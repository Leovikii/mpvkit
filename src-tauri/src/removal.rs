use crate::{
    deploy,
    error::{AppError, Result},
    platform,
};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

fn error(code: &'static str, message: &'static str) -> AppError {
    AppError::new(code, message, "remove")
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    directory: bool,
    identity: String,
    hash: Option<String>,
}

fn identity(path: &Path) -> Result<String> {
    let m = fs::symlink_metadata(path).map_err(|_| error("remove_read", "无法读取待删除内容。"))?;
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(format!("{}:{}", m.dev(), m.ino()))
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
        use windows_sys::Win32::Storage::FileSystem::{
            BY_HANDLE_FILE_INFORMATION, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
            GetFileInformationByHandle,
        };
        let _ = m;
        let file = fs::OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)
            .map_err(|_| error("remove_read", "无法读取待删除内容。"))?;
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        // SAFETY: a live handle and initialized writable file-information structure.
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
            return Err(error("remove_read", "无法读取文件身份。"));
        }
        Ok(format!(
            "{}:{}:{}",
            info.dwVolumeSerialNumber, info.nFileIndexHigh, info.nFileIndexLow
        ))
    }
}

pub fn validate_root(root: &Path) -> Result<()> {
    platform::validate_local_path(root)?;
    if root.parent().is_none()
        || platform::default_root()?.starts_with(root)
        || std::env::current_exe().is_ok_and(|p| p.starts_with(root))
        || [
            "HOME",
            "USERPROFILE",
            "APPDATA",
            "LOCALAPPDATA",
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
        ]
        .iter()
        .filter_map(std::env::var_os)
        .any(|v| Path::new(&v) == root)
    {
        return Err(error(
            "unsafe_target_root",
            "此位置不是独立的 MPV 配置目录，无法完整删除。",
        ));
    }
    #[cfg(target_os = "linux")]
    if [
        "/usr", "/etc", "/proc", "/sys", "/dev", "/run", "/bin", "/lib", "/boot", "/var", "/opt",
        "/sbin", "/lib64",
    ]
    .iter()
    .any(|p| root.starts_with(p))
    {
        return Err(error("unsafe_target_root", "不能删除系统目录。"));
    }
    #[cfg(target_os = "linux")]
    if ["/tmp", "/mnt", "/media", "/srv"]
        .iter()
        .any(|p| root == Path::new(p))
        || std::env::var_os("HOME").is_some_and(|home| {
            [".config", ".cache", ".local/state"]
                .iter()
                .any(|p| root == Path::new(&home).join(p))
        })
    {
        return Err(error(
            "unsafe_target_root",
            "此位置不是独立的 MPV 配置目录。",
        ));
    }
    #[cfg(target_os = "windows")]
    if [
        "WINDIR",
        "SystemRoot",
        "ProgramFiles",
        "ProgramFiles(x86)",
        "ProgramData",
    ]
    .iter()
    .filter_map(std::env::var_os)
    .any(|p| root.starts_with(p))
    {
        return Err(error("unsafe_target_root", "不能删除系统目录。"));
    }
    Ok(())
}

fn entries(root: &Path) -> Result<BTreeMap<PathBuf, Entry>> {
    fn visit(
        root: &Path,
        path: &Path,
        depth: usize,
        result: &mut BTreeMap<PathBuf, Entry>,
    ) -> Result<()> {
        if depth > 32 || result.len() >= 10000 {
            return Err(error("remove_limit", "配置目录内容过多，请手动检查。"));
        }
        platform::validate_local_path(path)?;
        let m =
            fs::symlink_metadata(path).map_err(|_| error("remove_read", "无法读取配置目录。"))?;
        let entry = Entry {
            directory: m.is_dir(),
            identity: identity(path)?,
            hash: if m.is_file() {
                deploy::file_hash(path)?
            } else {
                None
            },
        };
        result.insert(
            path.strip_prefix(root).expect("inside root").to_path_buf(),
            entry,
        );
        if m.is_dir() {
            for child in
                fs::read_dir(path).map_err(|_| error("remove_read", "无法读取配置目录。"))?
            {
                visit(
                    root,
                    &child
                        .map_err(|_| error("remove_read", "无法读取配置目录。"))?
                        .path(),
                    depth + 1,
                    result,
                )?;
            }
        }
        Ok(())
    }
    let mut result = BTreeMap::new();
    platform::validate_local_path(root)?;
    if root.exists() {
        visit(root, root, 0, &mut result)?;
    }
    Ok(result)
}

pub struct Plan {
    pub root: PathBuf,
    before: BTreeMap<PathBuf, Entry>,
    created: Instant,
}
impl Plan {
    pub fn paths(&self) -> Vec<String> {
        self.before
            .keys()
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| p.to_string_lossy().into_owned())
            .collect()
    }
    pub fn validate(&self) -> Result<()> {
        validate_root(&self.root)?;
        if self.created.elapsed() > Duration::from_secs(300) || entries(&self.root)? != self.before
        {
            return Err(error("stale_preview", "目录内容已变化，请重新确认。"));
        }
        Ok(())
    }
}
pub fn prepare(root: &Path) -> Result<Plan> {
    validate_root(root)?;
    Ok(Plan {
        root: root.into(),
        before: entries(root)?,
        created: Instant::now(),
    })
}

// Delete only reviewed entries. New or changed content is reported, never recursively swept up.
pub fn execute(plan: &Plan) -> Result<()> {
    validate_root(&plan.root)?;
    if !plan.root.exists() {
        return Ok(());
    }
    let root_entry = plan
        .before
        .get(Path::new(""))
        .ok_or_else(|| error("stale_preview", "配置目录已变化。"))?;
    if identity(&plan.root)? != root_entry.identity {
        return Err(error("stale_preview", "配置目录已被替换，请重新确认。"));
    }
    let mut paths: Vec<_> = plan.before.iter().collect();
    paths.sort_by_key(|(p, _)| std::cmp::Reverse(p.components().count()));
    let mut failed = Vec::new();
    for (relative, before) in paths {
        let path = plan.root.join(relative);
        let removed = (|| -> Result<()> {
            platform::validate_local_path(&path)?;
            if !path.exists() {
                return Ok(());
            }
            if identity(&path)? != before.identity
                || (!before.directory && deploy::file_hash(&path)? != before.hash)
            {
                return Err(error("stale_preview", "文件已变化，未删除。"));
            }
            let result = if before.directory {
                fs::remove_dir(&path)
            } else {
                fs::remove_file(&path)
            };
            result.map_err(|_| error("remove_failed", "文件被占用或无权删除。"))
        })();
        if removed.is_err() {
            failed.push(if relative.as_os_str().is_empty() {
                ".".into()
            } else {
                relative.to_string_lossy().into_owned()
            });
        }
    }
    if failed.is_empty() {
        Ok(())
    } else {
        Err(AppError::new(
            "remove_partial",
            format!("配置清理未完成：{}", failed.join("、")),
            "remove",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_removal_keeps_stable_lock_and_rejects_stale_content() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("mpv");
        fs::create_dir_all(root.join("watch_later")).unwrap();
        fs::write(root.join("private.conf"), "hand edited").unwrap();
        fs::write(root.join("watch_later/resume"), "position").unwrap();
        let plan = prepare(&root).unwrap();
        fs::write(root.join("private.conf"), "changed").unwrap();
        assert!(plan.validate().is_err());
        let plan = prepare(&root).unwrap();
        let lock = deploy::lock_target(&root).unwrap();
        execute(&plan).unwrap();
        assert!(!root.exists());
        assert!(deploy::acquire(&root).is_err());
        drop(lock);
        assert!(deploy::acquire(&root).is_ok());
    }
    #[test]
    fn late_new_and_changed_content_is_preserved_and_reported() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("mpv");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("reviewed.conf"), "reviewed").unwrap();
        fs::write(root.join("changed.conf"), "before").unwrap();
        let plan = prepare(&root).unwrap();
        fs::write(root.join("changed.conf"), "after").unwrap();
        fs::write(root.join("new.conf"), "new").unwrap();
        let error = execute(&plan).unwrap_err();
        assert_eq!(error.code, "remove_partial");
        assert!(error.message.contains("changed.conf"));
        assert!(!root.join("reviewed.conf").exists());
        assert_eq!(
            fs::read_to_string(root.join("changed.conf")).unwrap(),
            "after"
        );
        assert_eq!(fs::read_to_string(root.join("new.conf")).unwrap(), "new");
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn links_and_replaced_roots_are_never_followed() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("mpv");
        fs::create_dir(&root).unwrap();
        std::os::unix::fs::symlink(dir.path(), root.join("outside")).unwrap();
        assert!(prepare(&root).is_err());
        fs::remove_file(root.join("outside")).unwrap();
        let plan = prepare(&root).unwrap();
        fs::rename(&root, dir.path().join("old")).unwrap();
        fs::create_dir(&root).unwrap();
        fs::write(root.join("new.conf"), "keep").unwrap();
        assert!(execute(&plan).is_err());
        assert!(root.join("new.conf").exists());
        assert!(validate_root(Path::new("/usr/share/mpv")).is_err());
    }
}
