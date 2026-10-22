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

pub fn executable_path(value: &str) -> Result<PathBuf> {
    let path = PathBuf::from(value);
    if !path
        .file_name()
        .is_some_and(|name| name.eq_ignore_ascii_case("mpv.exe"))
    {
        return Err(AppError::new(
            "mpv_required",
            "请选择可信来源的 mpv.exe；检查版本会执行该文件。",
            "inspect",
        ));
    }
    file_stamp(&path)?;
    Ok(path)
}

/// Resolve an installed player without running it; managed runtime bytes must still match ownership.
pub fn managed_executable(
    root: &Path,
    manifest: &crate::deploy::InstallManifest,
) -> Result<PathBuf> {
    let player = if let Some(external) = &manifest.external_mpv {
        external.clone()
    } else {
        if !manifest
            .files
            .iter()
            .any(|file| file.path.eq_ignore_ascii_case("mpv.exe"))
        {
            return Err(AppError::new(
                "managed_player_unowned",
                "播放器缺少受管理归属记录；已停止查询，请检查或重新部署。",
                "inspect",
            ));
        }
        for file in &manifest.files {
            if !(file.path == "mpv.exe" || file.path == "d3dcompiler_43.dll") {
                continue;
            }
            crate::deploy::validate_relative(&file.path)?;
            if crate::deploy::file_hash(&root.join(&file.path))?.as_deref() != Some(&file.sha256) {
                return Err(AppError::new(
                    "file_conflict",
                    "受管理播放器或运行依赖缺失或已改变；已停止查询，请检查或恢复。",
                    "inspect",
                ));
            }
        }
        crate::config::player_path(root, manifest)
    };
    executable_path(
        player
            .to_str()
            .ok_or_else(|| AppError::new("invalid_path", "播放器路径不是有效文本。", "inspect"))?,
    )
}

pub fn capture(executable: &Path, args: &[OsString], timeout: Duration) -> Result<String> {
    let mut command = platform::command(executable);
    command.args(args);
    capture_command(command, timeout)
}

pub(crate) fn capture_command(mut command: Command, timeout: Duration) -> Result<String> {
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
                    || !status.success()
                {
                    break Err(error());
                }
                break Ok(String::from_utf8_lossy(&output).into_owned());
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
    pub ownership: &'static str,
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
        ownership: "external",
    })
}

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
        "osc",
        "input-builtin-bindings",
        "video-sync",
        "interpolation",
        "sub-font",
        "save-position-on-quit",
        "watch-later-options",
        "watch-later-directory",
        "resume-playback",
        "keep-open",
        "screenshot-directory",
        "screenshot-format",
    ]);
    for key in keys {
        if !available.contains(format!("--{key}").as_str()) {
            return Err(AppError::new(
                "managed_mpv_unsupported",
                "此MPV缺少受管理配置所需参数。请使用固定清单播放器；仍可按原生默认值手动启动外部MPV。",
                "inspect",
            ));
        }
    }
    Ok(())
}
