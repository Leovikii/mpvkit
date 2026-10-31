use crate::error::{AppError, Result};
use std::{
    fs,
    io::{self, Read},
    os::fd::AsRawFd,
    os::unix::fs::PermissionsExt,
    path::{Component, Path, PathBuf},
    process::Command,
};

pub fn xdg_directory(variable: &str, fallback: &str) -> Result<PathBuf> {
    let value = std::env::var_os(variable).filter(|v| !v.is_empty());
    let path = match value {
        Some(v) if Path::new(&v).is_absolute() => PathBuf::from(v),
        _ => PathBuf::from(
            std::env::var_os("HOME")
                .ok_or_else(|| AppError::new("user_path", "无法获取用户目录。", "paths"))?,
        )
        .join(fallback),
    };
    validate_local_path(&path)?;
    Ok(path)
}
pub fn default_root() -> Result<PathBuf> {
    #[cfg(debug_assertions)]
    if std::env::var_os("MPVKIT_TEST_MODE").is_some_and(|v| v == "1") {
        let root = PathBuf::from(std::env::var_os("MPVKIT_TEST_ROOT").ok_or_else(|| {
            AppError::new("test_root_required", "测试模式必须提供隔离目录。", "paths")
        })?);
        validate_local_path(&root)?;
        return Ok(root);
    }
    Ok(xdg_directory("XDG_DATA_HOME", ".local/share")?.join("io.github.mpvkit"))
}
pub fn app_root() -> Result<PathBuf> {
    Ok(mpv_config_location()?.directory)
}

pub struct ConfigLocation {
    pub directory: PathBuf,
    pub source: &'static str,
}

fn resolve_mpv_config(
    home: &Path,
    mpv_home: Option<&std::ffi::OsStr>,
    xdg_config_home: Option<&std::ffi::OsStr>,
) -> Result<ConfigLocation> {
    let (directory, source) = if let Some(value) = mpv_home.filter(|s| !s.is_empty()) {
        (PathBuf::from(value), "MPV_HOME")
    } else {
        validate_local_path(home)?;
        let (base, source) = match xdg_config_home.filter(|s| !s.is_empty()) {
            Some(value) => {
                let directory = PathBuf::from(value);
                validate_local_path(&directory)?;
                (directory, "XDG_CONFIG_HOME")
            }
            _ => (home.join(".config"), "HOME"),
        };
        let standard = base.join("mpv");
        let legacy = home.join(".mpv");
        if standard.is_dir()
            && ["mpv.conf", "input.conf", "scripts", "script-opts", "fonts"]
                .iter()
                .any(|name| legacy.join(name).exists())
        {
            return Err(AppError::new(
                "mpv_config_ambiguous",
                "同时发现标准 MPV 目录和 ~/.mpv 中的配置。MPV 会读取多个目录，请先整理后重试；文件未修改。",
                "paths",
            ));
        }
        if !standard.exists() && legacy.is_dir() {
            (legacy, "HOME/.mpv")
        } else {
            (standard, source)
        }
    };
    validate_local_path(&directory)?;
    if directory.parent().is_none() || directory.exists() && !directory.is_dir() {
        return Err(AppError::new(
            "mpv_config_directory",
            "MPV 配置位置不是可管理的用户目录，请检查 MPV_HOME 和 XDG_CONFIG_HOME。",
            "paths",
        ));
    }
    Ok(ConfigLocation { directory, source })
}

pub fn mpv_config_location() -> Result<ConfigLocation> {
    #[cfg(debug_assertions)]
    if std::env::var_os("MPVKIT_TEST_MODE").is_some_and(|v| v == "1") {
        return Ok(ConfigLocation {
            directory: default_root()?.join("config/mpv"),
            source: "isolated-test",
        });
    }
    let home = PathBuf::from(
        std::env::var_os("HOME")
            .ok_or_else(|| AppError::new("user_path", "无法获取用户目录。", "paths"))?,
    );
    resolve_mpv_config(
        &home,
        std::env::var_os("MPV_HOME").as_deref(),
        std::env::var_os("XDG_CONFIG_HOME").as_deref(),
    )
}
pub fn validate_local_path(path: &Path) -> Result<()> {
    let invalid = || {
        AppError::new(
            "local_path_required",
            "请使用本地绝对路径，不支持符号链接或父目录跳转。",
            "paths",
        )
    };
    if !path.is_absolute() || path.as_os_str().as_encoded_bytes().contains(&0) {
        return Err(invalid());
    }
    let mut current = PathBuf::new();
    for part in path.components() {
        if matches!(part, Component::ParentDir) {
            return Err(invalid());
        }
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(m) if m.file_type().is_symlink() || (!m.is_dir() && !m.is_file()) => {
                return Err(invalid());
            }
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(_) => return Err(AppError::new("path_unavailable", "路径不可访问。", "paths")),
        }
    }
    Ok(())
}
pub fn command(executable: &Path) -> Command {
    Command::new(executable)
}
pub fn drain_pipe<R: Read + AsRawFd>(pipe: &mut R, bytes: &mut Vec<u8>) -> io::Result<()> {
    loop {
        let mut available: libc::c_int = 0;
        // SAFETY: a valid child pipe fd and writable integer; does not consume data.
        if unsafe { libc::ioctl(pipe.as_raw_fd(), libc::FIONREAD, &mut available) } < 0 {
            return Err(io::Error::last_os_error());
        }
        if available <= 0 {
            return Ok(());
        }
        if bytes.len() + available as usize > 512 * 1024 {
            return Err(io::Error::other("probe output limit"));
        }
        let mut buffer = [0; 8192];
        let count = pipe.read(&mut buffer[..(available as usize).min(8192)])?;
        if count == 0 {
            return Ok(());
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
}
pub fn replace_file(source: &Path, destination: &Path) -> Result<()> {
    validate_local_path(source)?;
    validate_local_path(destination)?;
    let result = (|| {
        if destination.ends_with(super::UOSC_HELPER) {
            fs::set_permissions(source, fs::Permissions::from_mode(0o700))?;
        }
        fs::rename(source, destination)?;
        fs::File::open(destination.parent().expect("file parent"))?.sync_all()
    })();
    result.map_err(|_: io::Error| AppError::new("replace_failed", "文件替换或同步失败。", "deploy"))
}
pub fn playback_config(hardware_decoding: bool) -> &'static str {
    if hardware_decoding {
        "vo=gpu-next\ngpu-api=vulkan\ngpu-context=waylandvk\nhwdec=auto-safe\nsub-fonts-dir=~~/fonts\nosd-fonts-dir=~~/fonts\n"
    } else {
        "vo=gpu-next\ngpu-api=vulkan\ngpu-context=waylandvk\nhwdec=no\nsub-fonts-dir=~~/fonts\nosd-fonts-dir=~~/fonts\n"
    }
}
pub fn open_default_apps() -> Result<()> {
    if Path::new("/usr/bin/kcmshell6").is_file() {
        command(Path::new("/usr/bin/kcmshell6"))
            .arg("kcm_componentchooser")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|_| {
                AppError::new(
                    "settings_open",
                    "请手动进入系统设置的默认应用页面。",
                    "integration",
                )
            })?;
        Ok(())
    } else {
        Err(AppError::new(
            "settings_open",
            "请在桌面环境的系统设置中选择默认应用。",
            "integration",
        ))
    }
}
pub fn hdr_config(hdr: crate::config::Hdr) -> &'static str {
    match hdr {
        crate::config::Hdr::Auto => {
            "target-colorspace-hint=auto\ntarget-trc=auto\ntarget-prim=auto\n"
        }
        crate::config::Hdr::On => {
            "target-colorspace-hint=yes\ntarget-trc=pq\ntarget-prim=bt.2020\n"
        }
        crate::config::Hdr::Off => {
            "target-colorspace-hint=yes\ntarget-trc=srgb\ntarget-prim=bt.709\n"
        }
    }
}

pub fn open_repository() -> Result<()> {
    if cfg!(debug_assertions) && std::env::var_os("MPVKIT_TEST_MODE").is_some_and(|v| v == "1") {
        return Ok(());
    }
    command(Path::new("/usr/bin/xdg-open"))
        .arg("https://github.com/Leovikii/mpvkit")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|_| AppError::new("repository_open", "无法打开浏览器。", "repository"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detected_directory_is_loaded_by_the_system_mpv_without_config_dir() {
        let sandbox = std::env::temp_dir().join(format!(
            "mpvkit-native-search-{}",
            crate::deploy::operation_id()
        ));
        for case in ["default", "xdg", "explicit", "legacy"] {
            let home = sandbox.join(case);
            fs::create_dir_all(&home).unwrap();
            if case == "legacy" {
                fs::create_dir(home.join(".mpv")).unwrap();
            }
            let mpv_home = (case == "explicit").then(|| home.join("explicit"));
            let xdg = match case {
                "xdg" => Some(home.join("xdg")),
                _ => None,
            };
            let detected = resolve_mpv_config(
                &home,
                mpv_home.as_deref().map(Path::as_os_str),
                xdg.as_deref().map(Path::as_os_str),
            )
            .unwrap();
            fs::create_dir_all(&detected.directory).unwrap();
            fs::write(
                detected.directory.join("mpv.conf"),
                "[mpvkit-detector-check]\nvolume=37\n",
            )
            .unwrap();
            let mut query = command(Path::new("/usr/bin/mpv"));
            query
                .env("HOME", &home)
                .env_remove("MPV_HOME")
                .env_remove("XDG_CONFIG_HOME");
            if let Some(value) = mpv_home {
                query.env("MPV_HOME", value);
            }
            if let Some(value) = xdg {
                query.env("XDG_CONFIG_HOME", value);
            }
            query.args(["--load-scripts=no", "--show-profile=mpvkit-detector-check"]);
            let output =
                crate::mpv::capture_command(query, std::time::Duration::from_secs(3)).unwrap();
            assert!(output.contains("volume=37"), "{case}: {output}");
        }
        fs::remove_dir_all(sandbox).unwrap();
    }
    #[test]
    fn configuration_detection_obeys_mpv_precedence_and_rejects_unsafe_targets() {
        let root =
            std::env::temp_dir().join(format!("mpvkit-detect-{}", crate::deploy::operation_id()));
        fs::create_dir(&root).unwrap();
        let xdg = root.join("xdg");
        let explicit = root.join("explicit");
        let detect = |mpv_home: Option<&std::ffi::OsStr>, xdg_home: Option<&std::ffi::OsStr>| {
            resolve_mpv_config(&root, mpv_home, xdg_home)
        };
        assert_eq!(
            detect(None, None).unwrap().directory,
            root.join(".config/mpv")
        );
        assert_eq!(
            detect(None, Some(xdg.as_os_str())).unwrap().directory,
            xdg.join("mpv")
        );
        let selected = detect(Some(explicit.as_os_str()), Some(xdg.as_os_str())).unwrap();
        assert_eq!(selected.directory, explicit);
        assert_eq!(selected.source, "MPV_HOME");
        assert!(detect(None, Some(std::ffi::OsStr::new("relative"))).is_err());
        assert!(detect(Some(std::ffi::OsStr::new("relative")), None).is_err());
        assert!(detect(Some(std::ffi::OsStr::new("/")), None).is_err());
        fs::create_dir(root.join(".mpv")).unwrap();
        assert_eq!(detect(None, None).unwrap().directory, root.join(".mpv"));
        fs::create_dir_all(root.join(".config/mpv")).unwrap();
        assert_eq!(
            detect(None, None).unwrap().directory,
            root.join(".config/mpv")
        );
        fs::write(root.join(".mpv/mpv.conf"), "volume=11\n").unwrap();
        assert_eq!(
            detect(None, None).err().unwrap().code,
            "mpv_config_ambiguous"
        );
        fs::remove_file(root.join(".mpv/mpv.conf")).unwrap();
        let link = root.join("linked");
        std::os::unix::fs::symlink(&explicit, &link).unwrap();
        assert!(detect(Some(link.as_os_str()), None).is_err());
        fs::write(&explicit, "not a directory").unwrap();
        assert!(detect(Some(explicit.as_os_str()), None).is_err());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn symlinks_and_parent_paths_cannot_redirect_writes() {
        let root =
            std::env::temp_dir().join(format!("mpvkit-paths-{}", crate::deploy::operation_id()));
        fs::create_dir(&root).unwrap();
        std::os::unix::fs::symlink("/tmp", root.join("link")).unwrap();
        assert!(validate_local_path(&root.join("link/file")).is_err());
        assert!(validate_local_path(&root.join("../other")).is_err());
        assert!(validate_local_path(Path::new("relative")).is_err());
        assert!(validate_local_path(&root.join("new/file")).is_ok());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn probe_is_bounded_even_if_a_descendant_keeps_stdout_open() {
        let mut cmd = command(Path::new("/usr/bin/sh"));
        cmd.args(["-c", "sleep 1 & printf ok"]);
        assert_eq!(
            crate::mpv::capture_command(cmd, std::time::Duration::from_millis(500)).unwrap(),
            "ok"
        );
        let mut cmd = command(Path::new("/usr/bin/sh"));
        cmd.args(["-c", "sleep 1"]);
        assert_eq!(
            crate::mpv::capture_command(cmd, std::time::Duration::from_millis(20))
                .unwrap_err()
                .code,
            "probe_timeout"
        );
    }
}
