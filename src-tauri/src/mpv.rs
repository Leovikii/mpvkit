use crate::{
    error::{AppError, Result},
    platform,
};
use serde::Serialize;
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime},
};

#[derive(Clone, Debug, PartialEq)]
pub struct FileStamp {
    length: u64,
    modified: SystemTime,
}

// ponytail: 启动预览仅核对长度/修改时间；M2 改写部署必须另做哈希与互斥。
pub fn file_stamp(path: &Path) -> Result<FileStamp> {
    platform::validate_local_path(path)?;
    let metadata = fs::metadata(path)
        .map_err(|_| AppError::new("file_unavailable", "文件不存在或无法读取。", "inspect"))?;
    if !metadata.is_file() {
        return Err(AppError::new(
            "file_required",
            "请选择文件，而不是文件夹。",
            "inspect",
        ));
    }
    Ok(FileStamp {
        length: metadata.len(),
        modified: metadata
            .modified()
            .map_err(|_| AppError::new("file_state", "无法读取文件修改时间。", "inspect"))?,
    })
}

#[cfg(all(test, target_os = "windows"))]
pub fn executable_path(value: &str) -> Result<PathBuf> {
    let path = PathBuf::from(value);
    if !path
        .file_name()
        .is_some_and(|name| name.eq_ignore_ascii_case("mpv.exe"))
    {
        return Err(AppError::new(
            "mpv_required",
            "播放器名称无效；检查版本会执行该文件。",
            "inspect",
        ));
    }
    file_stamp(&path)?;
    Ok(path)
}

/// Probe the system core without taking ownership of package-manager files.
pub fn managed_executable(
    root: &Path,
    manifest: &crate::deploy::InstallManifest,
) -> Result<PathBuf> {
    if !manifest.components.contains_key("mpv") {
        return Err(AppError::new(
            "not_installed",
            "请先安装播放增强。",
            "inspect",
        ));
    }
    let player = crate::config::player_path(root)?;
    inspect(&player)?;
    Ok(player)
}

pub fn capture(executable: &Path, args: &[OsString], timeout: Duration) -> Result<String> {
    let mut command = platform::command(executable);
    command.args(args);
    capture_command(command, timeout)
}

pub(crate) fn capture_command(command: Command, timeout: Duration) -> Result<String> {
    let (status, output) = capture_status(command, timeout)?;
    if !status.success() {
        return Err(AppError::new(
            "probe_failed",
            "程序查询失败，请检查安装状态。",
            "inspect",
        ));
    }
    Ok(output)
}

pub(crate) fn capture_status(
    command: Command,
    timeout: Duration,
) -> Result<(std::process::ExitStatus, String)> {
    capture_output(command, timeout).map(|(status, output, _)| (status, output))
}

pub(crate) fn capture_output(
    mut command: Command,
    timeout: Duration,
) -> Result<(std::process::ExitStatus, String, String)> {
    let error = || {
        AppError::new(
            "probe_failed",
            "MPV 查询失败或输出过多，请检查所选播放器。",
            "inspect",
        )
    };
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| error())?;
    let mut stdout = child.stdout.take().expect("piped stdout");
    let mut stderr = child.stderr.take().expect("piped stderr");
    let (mut output, mut errors) = (Vec::new(), Vec::new());
    let deadline = Instant::now() + timeout;
    let result = loop {
        if platform::drain_pipe(&mut stdout, &mut output).is_err()
            || platform::drain_pipe(&mut stderr, &mut errors).is_err()
        {
            break Err(error());
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                if platform::drain_pipe(&mut stdout, &mut output).is_err()
                    || platform::drain_pipe(&mut stderr, &mut errors).is_err()
                {
                    break Err(error());
                }
                break Ok((
                    status,
                    String::from_utf8_lossy(&output).into_owned(),
                    String::from_utf8_lossy(&errors).into_owned(),
                ));
            }
            Ok(None) => {}
            Err(_) => break Err(error()),
        }
        if Instant::now() >= deadline {
            break Err(AppError::new(
                "probe_timeout",
                "MPV 查询超时，已停止查询进程。",
                "inspect",
            ));
        }
        thread::sleep(Duration::from_millis(10));
    };
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MpvInfo {
    pub executable: PathBuf,
    pub version: String,
}

#[cfg(all(test, target_os = "windows"))]
pub fn fixture_player(root: &Path) -> PathBuf {
    static BINARY: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    let bytes = BINARY.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!(
            "mpvkit-probe-fixture-{}",
            crate::deploy::operation_id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let source = dir.join("fixture.rs");
        let target = dir.join("mpv.exe");
        fs::write(&source, include_str!("../tests/fixtures/mpv.rs")).unwrap();
        let status = Command::new("rustc")
            .args(["--edition=2024", "--crate-name=mpv_fixture"])
            .arg(&source)
            .arg("-o")
            .arg(&target)
            .status()
            .unwrap();
        assert!(status.success());
        let bytes = fs::read(target).unwrap();
        fs::remove_dir_all(dir).unwrap();
        bytes
    });
    fs::create_dir_all(root).unwrap();
    let player = root.join("mpv.exe");
    fs::write(&player, bytes).unwrap();
    player
}

pub fn inspect(executable: &Path) -> Result<MpvInfo> {
    let query = |option: &str| {
        capture(
            executable,
            &[
                "--no-config".into(),
                "--load-scripts=no".into(),
                option.into(),
            ],
            Duration::from_secs(3),
        )
    };
    let version = query("--version")?;
    let version = version
        .lines()
        .find(|line| line.starts_with("mpv ") && line.len() < 512)
        .ok_or_else(|| {
            AppError::new(
                "invalid_version",
                "所选文件未返回可识别的 MPV 版本。",
                "inspect",
            )
        })?
        .to_owned();
    let options = query("--list-options")?;
    for required in ["--config-dir", "--load-scripts", "--terminal"] {
        if !options
            .lines()
            .any(|line| line.split_whitespace().next() == Some(required))
        {
            return Err(AppError::new(
                "unsupported_mpv",
                "此 MPV 未报告隔离启动所需的参数，暂不能使用。",
                "inspect",
            ));
        }
    }
    Ok(MpvInfo {
        executable: executable.to_path_buf(),
        version,
    })
}

#[cfg(all(test, target_os = "windows"))]
pub fn launch_args(config: &Path, media: &Path) -> Vec<OsString> {
    let mut config_arg = OsString::from("--config-dir=");
    config_arg.push(config);
    vec![
        "--no-config".into(),
        config_arg,
        "--load-scripts=no".into(),
        "--no-terminal".into(),
        "--".into(),
        media.as_os_str().to_owned(),
    ]
}

pub fn ensure_managed_options(executable: &Path) -> Result<()> {
    let output = capture(
        executable,
        &[
            "--no-config".into(),
            "--load-scripts=no".into(),
            "--list-options".into(),
        ],
        Duration::from_secs(3),
    )?;
    let available: std::collections::BTreeSet<_> = output
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    let profiles = crate::config::profiles();
    let keys = profiles["balanced"].keys().map(String::as_str).chain([
        "vo",
        "gpu-api",
        "hwdec",
        "sub-font-size",
        "osd-font-size",
        "volume",
        "sub-fonts-dir",
        "osd-fonts-dir",
        "osc",
        "input-builtin-bindings",
        "video-sync",
        "interpolation",
        "autofit",
        "auto-window-resize",
        "sub-font",
        "save-position-on-quit",
        "watch-later-options",
        "watch-later-directory",
        "resume-playback",
        "keep-open",
        "autocreate-playlist",
        "directory-filter-types",
        "directory-mode",
        "sub-file-paths",
        "sub-auto",
        "target-colorspace-hint",
        "target-trc",
        "target-prim",
    ]);
    #[cfg(target_os = "linux")]
    {
        for key in ["--gpu-context", "--sub-fonts-dir", "--osd-fonts-dir"] {
            if !available.contains(key) {
                return Err(AppError::new(
                    "managed_mpv_unsupported",
                    "系统 MPV 缺少 Linux 配置所需参数，请使用系统包管理器更新。",
                    "inspect",
                ));
            }
        }
        let contexts = capture(
            executable,
            &["--no-config".into(), "--gpu-context=help".into()],
            Duration::from_secs(3),
        )?;
        if !contexts
            .lines()
            .any(|l| l.split_whitespace().next() == Some("waylandvk"))
        {
            return Err(AppError::new(
                "wayland_required",
                "系统 MPV 不支持 Wayland/Vulkan，请检查系统 MPV 构建。",
                "inspect",
            ));
        }
    }
    for key in keys {
        if !available.contains(format!("--{key}").as_str()) {
            return Err(AppError::new(
                "managed_mpv_unsupported",
                "此MPV缺少受管理配置所需参数。请使用系统包管理器更新 MPV。",
                "inspect",
            ));
        }
    }
    Ok(())
}
