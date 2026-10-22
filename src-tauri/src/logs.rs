use crate::{
    deploy,
    error::{AppError, Result},
    platform,
};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime},
};
const KEEP: usize = 20;
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub id: String,
    pub files: Vec<PathBuf>,
    pub bytes: u64,
    pub skipped: usize,
}
pub struct Plan {
    pub preview: Preview,
    pub(crate) root: PathBuf,
    hashes: Vec<(PathBuf, String, String)>,
    created: Instant,
}
fn error(code: &'static str, message: &'static str) -> AppError {
    AppError::new(code, message, "logs")
}
pub fn directory(root: &Path) -> Result<PathBuf> {
    Ok(root.join(".mpvkit/cache/sessions"))
}
fn active(log: &str) -> bool {
    log.lines()
        .filter_map(|l| l.strip_prefix("phase=launch status=started pid="))
        .filter_map(|p| p.parse::<u32>().ok())
        .any(platform::process_active)
}
fn sessions(root: &Path) -> Result<Vec<PathBuf>> {
    let directory = directory(root)?;
    platform::validate_local_path(&directory)?;
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(_) => return Err(error("logs_unavailable", "会话目录不可读取。")),
    };
    let mut paths = Vec::new();
    for entry in entries.take(8193) {
        paths.push(
            entry
                .map_err(|_| error("logs_unavailable", "会话目录项不可读取。"))?
                .path(),
        );
    }
    if paths.len() > 8192 {
        return Err(error(
            "logs_limit",
            "会话目录过多，请检查管理目录；未执行清理。",
        ));
    }
    Ok(paths)
}
pub fn check_capacity(root: &Path) -> Result<()> {
    let mut count = 0;
    for directory in sessions(root)? {
        let path = directory.join("session.log");
        platform::validate_local_path(&path)?;
        if path.is_file() {
            count += 1;
        }
    }
    if count >= 1024 {
        return Err(error(
            "logs_full",
            "会话日志达到1024份上限，请先预览并清理旧日志。",
        ));
    }
    Ok(())
}
pub fn prepare(root: &Path) -> Result<Plan> {
    deploy::status(root)?;
    let mut candidates = Vec::new();
    let mut skipped = 0;
    for directory in sessions(root)? {
        if platform::validate_local_path(&directory).is_err() {
            skipped += 1;
            continue;
        }
        let owner = directory.join("session.json");
        let log = directory.join("session.log");
        let Ok(owner_bytes) = deploy::read_bytes(&owner) else {
            skipped += 1;
            continue;
        };
        if owner_bytes.len() > 4096 {
            skipped += 1;
            continue;
        }
        let Ok(record) = serde_json::from_slice::<serde_json::Value>(&owner_bytes) else {
            skipped += 1;
            continue;
        };
        if record["schemaVersion"] != 1
            || record["owner"] != "MpvKit"
            || record["sessionId"].as_str() != directory.file_name().and_then(|p| p.to_str())
        {
            skipped += 1;
            continue;
        }
        let Ok(bytes) = deploy::read_bytes(&log) else {
            skipped += 1;
            continue;
        };
        let Ok(text) = std::str::from_utf8(&bytes) else {
            skipped += 1;
            continue;
        };
        let valid = text.lines().count() <= 3
            && text.lines().all(|l| {
                [
                    "phase=launch status=prepared",
                    "phase=launch status=spawn_failed",
                    "phase=launch status=early_exit",
                ]
                .contains(&l)
                    || l.strip_prefix("phase=launch status=started pid=")
                        .is_some_and(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
            });
        if !valid || bytes.len() > 4096 || active(text) {
            skipped += 1;
            continue;
        }
        let modified = fs::metadata(&log)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::now());
        candidates.push((
            modified,
            log,
            deploy::hash_bytes(&bytes),
            deploy::hash_bytes(&owner_bytes),
            bytes.len() as u64,
        ));
    }
    candidates.sort_by_key(|v| std::cmp::Reverse(v.0));
    let old: Vec<_> = candidates.into_iter().skip(KEEP).collect();
    let preview = Preview {
        id: deploy::operation_id(),
        files: old.iter().map(|v| v.1.clone()).collect(),
        bytes: old.iter().map(|v| v.4).sum(),
        skipped,
    };
    Ok(Plan {
        preview,
        root: root.into(),
        hashes: old
            .into_iter()
            .map(|(_, p, hash, owner, _)| (p, hash, owner))
            .collect(),
        created: Instant::now(),
    })
}
pub fn execute(plan: Plan) -> Result<usize> {
    if plan.created.elapsed() > Duration::from_secs(300) {
        return Err(error("stale_preview", "日志清理预览已过期。"));
    }
    let _lock = deploy::acquire(&plan.root)?;
    for (path, hash, owner_hash) in &plan.hashes {
        if deploy::file_hash(path)?.as_deref() != Some(hash)
            || deploy::file_hash(&path.with_file_name("session.json"))?.as_deref()
                != Some(owner_hash)
        {
            return Err(error(
                "stale_preview",
                "日志或归属在预览后改变，未继续清理。",
            ));
        }
    }
    let mut removed = 0;
    for (path, hash, owner_hash) in plan.hashes {
        if deploy::file_hash(&path)?.as_deref() != Some(&hash) {
            return Err(error(
                "logs_changed",
                "清理时日志已改变，已停止；此前日志可能已移除。",
            ));
        }
        if active(&String::from_utf8_lossy(&deploy::read_bytes(&path)?)) {
            continue;
        }
        fs::remove_file(&path)
            .map_err(|_| error("logs_remove", "日志被占用或不可移除；此前日志可能已清理。"))?;
        let directory = path.parent().expect("session log parent");
        let entries = fs::read_dir(directory)
            .map_err(|_| error("logs_unavailable", "会话目录不可读。"))?
            .collect::<std::io::Result<Vec<_>>>()
            .map_err(|_| error("logs_unavailable", "会话目录不可读。"))?;
        if entries.len() == 1 && entries[0].file_name() == "session.json" {
            let owner = directory.join("session.json");
            if deploy::file_hash(&owner)?.as_deref() == Some(&owner_hash) {
                fs::remove_file(owner).map_err(|_| error("logs_remove", "无法清理旧会话记录。"))?;
                // Non-recursive: a newly added unknown file is always preserved.
                let _ = fs::remove_dir(directory);
            }
        }
        removed += 1;
    }
    Ok(removed)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_verified_old_logs_are_removed_after_fresh_preview() {
        let root = std::env::temp_dir().join(format!("mpvkit-logs-{}", deploy::operation_id()));
        drop(deploy::acquire(&root).unwrap());
        for n in 0..25 {
            let dir = root.join(format!(".mpvkit/cache/sessions/session-{n}"));
            fs::create_dir_all(&dir).unwrap();
            fs::write(
                dir.join("session.json"),
                format!(r#"{{"schemaVersion":1,"owner":"MpvKit","sessionId":"session-{n}"}}"#),
            )
            .unwrap();
            fs::write(dir.join("session.log"), "phase=launch status=prepared\n").unwrap();
            fs::write(dir.join("user.txt"), "keep").unwrap();
        }
        let plan = prepare(&root).unwrap();
        assert_eq!(plan.preview.files.len(), 5);
        let target = plan.preview.files[0].clone();
        fs::write(&target, "user changed").unwrap();
        assert_eq!(execute(plan).unwrap_err().code, "stale_preview");
        assert_eq!(execute(prepare(&root).unwrap()).unwrap(), 4);
        assert_eq!(fs::read_to_string(target).unwrap(), "user changed");
        assert_eq!(
            fs::read_to_string(root.join(".mpvkit/cache/sessions/session-0/user.txt")).unwrap(),
            "keep"
        );
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn cleanup_removes_owned_inactive_session_directories_and_preserves_running_processes() {
        let root = std::env::temp_dir().join(format!("mpvkit-logs-{}", deploy::operation_id()));
        drop(deploy::acquire(&root).unwrap());
        for n in 0..25 {
            let dir = root.join(format!(".mpvkit/cache/sessions/session-{n}"));
            fs::create_dir_all(&dir).unwrap();
            fs::write(
                dir.join("session.json"),
                format!(r#"{{"schemaVersion":1,"owner":"MpvKit","sessionId":"session-{n}"}}"#),
            )
            .unwrap();
            fs::write(
                dir.join("session.log"),
                if n == 0 {
                    format!("phase=launch status=started pid={}\n", std::process::id())
                } else {
                    "phase=launch status=prepared\n".into()
                },
            )
            .unwrap();
        }
        let plan = prepare(&root).unwrap();
        assert_eq!(plan.preview.files.len(), 4);
        let removed = plan
            .preview
            .files
            .iter()
            .map(|p| p.parent().unwrap().to_owned())
            .collect::<Vec<_>>();
        execute(plan).unwrap();
        assert!(removed.iter().all(|p| !p.exists()));
        assert!(
            root.join(".mpvkit/cache/sessions/session-0/session.json")
                .exists()
        );
        fs::remove_dir_all(root).unwrap();
    }
}
