use crate::{
    error::{AppError, Result},
    mpv::{self, FileStamp, MpvInfo},
    platform,
};
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Stdio,
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchPreview {
    pub id: String,
    pub mpv: MpvInfo,
    pub media: PathBuf,
    pub config_directory: PathBuf,
    pub arguments: Vec<String>,
    pub changes: Vec<String>,
}

pub struct LaunchPlan {
    pub preview: LaunchPreview,
    pub(crate) root: PathBuf,
    created: Instant,
    executable_stamp: FileStamp,
    media_stamp: FileStamp,
    session_directory: PathBuf,
    managed_files: Vec<(PathBuf, Option<String>)>,
    generation: Option<String>,
}

pub fn prepare(root: &Path, executable: &str, media: &str) -> Result<LaunchPlan> {
    platform::validate_local_path(root)?;
    crate::logs::check_capacity(root)?;
    let executable = mpv::executable_path(executable)?;
    let media = PathBuf::from(media);
    let executable_stamp = mpv::file_stamp(&executable)?;
    let media_stamp = mpv::file_stamp(&media)?;
    let info = mpv::inspect(&executable)?;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let id = format!(
        "{nanos:x}-{}-{}",
        std::process::id(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    );
    let config_directory = root.join("cache").join("sessions").join(&id);
    let arguments = mpv::launch_args(&config_directory, &media)
        .into_iter()
        .map(|value| value.to_string_lossy().into_owned())
        .collect();
    Ok(LaunchPlan {
        root: root.into(),
        preview: LaunchPreview {
            id,
            mpv: info,
            media,
            config_directory: config_directory.clone(),
            arguments,
            changes: vec![
                "创建本次专用空配置目录及 session.json、session.log；不覆盖现有文件。".into(),
                "执行所选外部 MPV，使用原生默认值播放所选本地文件。".into(),
            ],
        },
        created: Instant::now(),
        executable_stamp,
        media_stamp,
        session_directory: config_directory,
        managed_files: Vec::new(),
        generation: None,
    })
}

pub fn prepare_managed(root: &Path, media: &str) -> Result<LaunchPlan> {
    let status = crate::deploy::status(root)?;
    if status.recovery_required {
        return Err(AppError::new(
            "recovery_required",
            "请先恢复未完成操作。",
            "launch",
        ));
    }
    let manifest = status
        .manifest
        .ok_or_else(|| AppError::new("not_installed", "请先部署受管理环境。", "launch"))?;
    if !crate::deploy::script_conflicts(root, &manifest)?.is_empty() {
        return Err(AppError::new(
            "script_conflict",
            "存在未管理的自动脚本，请检查后再启动。",
            "launch",
        ));
    }
    let mut hashes = Vec::new();
    let mut degraded = false;
    for file in &manifest.files {
        crate::deploy::validate_relative(&file.path)?;
        let path = root.join(&file.path);
        let hash = crate::deploy::file_hash(&path)?;
        let component_file = file.path.starts_with(if manifest.config_directory == "." {
            "scripts/"
        } else {
            "config/scripts/"
        }) || file.path.starts_with(if manifest.config_directory == "." {
            "fonts/"
        } else {
            "config/fonts/"
        }) || file.path.starts_with(if manifest.config_directory == "." {
            "script-opts/"
        } else {
            "config/script-opts/"
        }) || file.path.ends_with("mpvkit-context.json")
            || file.path.ends_with("mpvkit-profiles.json");
        if hash.is_none() && component_file {
            degraded = true;
        } else if hash.as_deref() != Some(&file.sha256) {
            return Err(AppError::new(
                "file_conflict",
                "受管理文件已改变，请检查或恢复后启动。",
                "launch",
            ));
        }
        hashes.push((path, hash));
    }
    let player = mpv::managed_executable(root, &manifest)?;
    mpv::ensure_managed_options(&player)?;
    let manifest_path = root.join(".mpvkit/install.json");
    hashes.push((
        manifest_path.clone(),
        crate::deploy::file_hash(&manifest_path)?,
    ));
    let config_directory = root.join(&manifest.config_directory);
    let user = config_directory.join("user.conf");
    let user_hash = crate::deploy::file_hash(&user)?;
    let mut plan = prepare(
        root,
        player
            .to_str()
            .ok_or_else(|| AppError::new("invalid_path", "播放器路径不是有效文本。", "launch"))?,
        media,
    )?;
    plan.preview.mpv.ownership = if manifest.external_mpv.is_some() {
        "external"
    } else {
        "managed"
    };
    plan.preview.config_directory = config_directory;
    let mut args = vec![
        "--config=yes".into(),
        format!("--config-dir={}", plan.preview.config_directory.display()),
        if degraded {
            "--load-scripts=no"
        } else {
            "--load-scripts=yes"
        }
        .into(),
        "--no-terminal".into(),
    ];
    let preference_path = root.join(".mpvkit/playback.json");
    let preference_hash = crate::deploy::file_hash(&preference_path)?;
    if let Some(saved) = crate::preferences::read(root)?
        && saved.generation == manifest.installation_id
    {
        for (key, value) in &crate::config::profiles()[saved.profile.id()] {
            args.push(format!("--{key}={value}"));
        }
    }
    hashes.push((preference_path, preference_hash));
    if user_hash.is_some() {
        args.push(format!("--include={}", user.display()));
    }
    args.push("--".into());
    args.push(media.into());
    hashes.push((user, user_hash));
    plan.preview.arguments = args;
    plan.managed_files = hashes;
    plan.generation = Some(manifest.installation_id);
    plan.preview.changes = vec!["加载已核对归属与摘要的受管理配置、uosc 和 thumbfast。".into(), "所选配置目录存在 user.conf 时最后加载；它不纳入覆盖或卸载。高级配置可覆盖 GUI 设置并加载其它脚本，请仅保留可信内容。".into(), "创建独立会话状态目录；关闭管理界面后播放器继续运行。".into()];
    if degraded {
        plan.preview.changes.push("组件文件缺失：本次禁用全部自动脚本，使用基础渲染链；uosc、缩略图与播放期全局保存不可用。请在组件区预览修复。".into());
    }
    Ok(plan)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionRecord<'a> {
    schema_version: u32,
    owner: &'static str,
    session_id: &'a str,
    mpv_version: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchResult {
    pub process_id: u32,
    pub message: &'static str,
}

pub fn execute(plan: LaunchPlan) -> Result<LaunchResult> {
    let stale = || {
        AppError::new(
            "stale_preview",
            "预览已过期或文件状态已变化，请重新预览。",
            "launch",
        )
    };
    if plan.created.elapsed() > Duration::from_secs(300)
        || mpv::file_stamp(&plan.preview.mpv.executable)? != plan.executable_stamp
        || mpv::file_stamp(&plan.preview.media)? != plan.media_stamp
    {
        return Err(stale());
    }
    for (path, hash) in &plan.managed_files {
        if crate::deploy::file_hash(path)? != *hash {
            return Err(stale());
        }
    }
    if !plan.managed_files.is_empty()
        && let Some(manifest) = crate::deploy::status(&plan.root)?.manifest
        && !crate::deploy::script_conflicts(&plan.root, &manifest)?.is_empty()
    {
        return Err(stale());
    }
    let directory = &plan.session_directory;
    platform::validate_local_path(directory)?;
    let write_error = || {
        AppError::new(
            "session_write",
            "无法创建本次会话目录或记录，请检查权限和剩余空间。",
            "launch",
        )
    };
    fs::create_dir_all(directory.parent().expect("session parent")).map_err(|_| write_error())?;
    platform::validate_local_path(directory)?;
    // A new directory per confirmed launch. A collision never reuses old files.
    fs::create_dir(directory).map_err(|_| write_error())?;
    let mut record = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join("session.json"))
        .map_err(|_| write_error())?;
    let data = SessionRecord {
        schema_version: 1,
        owner: "MpvKit",
        session_id: &plan.preview.id,
        mpv_version: &plan.preview.mpv.version,
    };
    serde_json::to_writer_pretty(&mut record, &data).map_err(|_| write_error())?;
    record.sync_all().map_err(|_| write_error())?;
    let mut log = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join("session.log"))
        .map_err(|_| write_error())?;
    writeln!(log, "phase=launch status=prepared").map_err(|_| write_error())?;
    let child = platform::command(&plan.preview.mpv.executable)
        .args(&plan.preview.arguments)
        .current_dir(directory)
        .env("MPV_HOME", directory)
        .env(
            "MPVKIT_GENERATION",
            plan.generation.as_deref().unwrap_or_default(),
        )
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    let mut child = match child {
        Ok(child) => child,
        Err(_) => {
            let _ = writeln!(log, "phase=launch status=spawn_failed");
            return Err(AppError::new(
                "launch_failed",
                "播放器无法启动，本次会话记录已保留。",
                "launch",
            ));
        }
    };
    let process_id = child.id();
    // Report immediate startup failures; do not equate successful spawn with playback verification.
    thread::sleep(Duration::from_millis(200));
    match child.try_wait() {
        Ok(Some(status)) if !status.success() => {
            let _ = writeln!(log, "phase=launch status=early_exit");
            return Err(AppError::new(
                "player_exited",
                "MPV 启动后立即报错退出，请检查播放器与媒体文件。",
                "launch",
            ));
        }
        Err(_) => {
            thread::spawn(move || {
                let _ = child.wait();
            });
            return Err(AppError::new(
                "process_status",
                "播放器已创建，但无法确认进程状态。请查看播放器窗口。",
                "launch",
            ));
        }
        _ => {}
    }
    let logged = writeln!(log, "phase=launch status=started pid={process_id}").is_ok();
    // No GUI lifetime guard or kill-on-drop: closing MpvKit must leave MPV running.
    thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(LaunchResult {
        process_id,
        message: if logged {
            "已启动 MPV 进程；实际播放结果请在播放器窗口确认。关闭 MpvKit 可继续播放。"
        } else {
            "已启动 MPV 进程，但会话日志未能写入。实际播放结果请在播放器窗口确认。"
        },
    })
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod real_tests;
