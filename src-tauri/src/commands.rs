use crate::{
    error::{AppError, Result},
    launch::{self, LaunchPlan, LaunchPreview, LaunchResult},
    platform,
};
use std::sync::Mutex;
use tauri::State;

#[derive(Default)]
pub struct PendingLaunch(pub Mutex<Option<LaunchPlan>>);

#[tauri::command]
pub async fn preview_launch(
    media: String,
    state: State<'_, PendingLaunch>,
) -> Result<LaunchPreview> {
    let root = platform::app_root()?;
    let plan = tauri::async_runtime::spawn_blocking(move || launch::prepare_managed(&root, &media))
        .await
        .map_err(|_| AppError::new("worker_failed", "检查任务异常结束，请重试。", "inspect"))??;
    let preview = plan.preview.clone();
    *state.0.lock().map_err(|_| {
        AppError::new(
            "state_unavailable",
            "会话状态不可用，请重启应用。",
            "preview",
        )
    })? = Some(plan);
    Ok(preview)
}

#[tauri::command]
pub async fn launch_mpv(
    preview_id: String,
    state: State<'_, PendingLaunch>,
) -> Result<LaunchResult> {
    let plan = {
        let mut pending = state.0.lock().map_err(|_| {
            AppError::new(
                "state_unavailable",
                "会话状态不可用，请重启应用。",
                "launch",
            )
        })?;
        if pending
            .as_ref()
            .is_none_or(|plan| plan.preview.id != preview_id)
        {
            return Err(AppError::new(
                "stale_preview",
                "没有匹配的启动预览，请重新检查。",
                "launch",
            ));
        }
        pending.take().expect("checked pending plan")
    };
    tauri::async_runtime::spawn_blocking(move || launch::execute(plan))
        .await
        .map_err(|_| {
            AppError::new(
                "worker_failed",
                "启动任务异常结束，请检查播放器窗口。",
                "launch",
            )
        })?
}

enum PendingOperation {
    Deployment(Box<crate::setup::ChangePlan>),
    Recovery(Box<crate::deploy::RecoveryPlan>),
}

impl PendingOperation {
    fn preview(&self) -> crate::setup::ChangePreview {
        match self {
            Self::Deployment(plan) => plan.preview(),
            Self::Recovery(plan) => crate::setup::ChangePreview {
                files: plan.preview.clone(),
                integration: None,
                retained: vec![],
            },
        }
    }
}

#[derive(Default)]
pub struct PendingDeployment(Mutex<Option<PendingOperation>>);

#[tauri::command]
pub async fn deployment_status() -> Result<crate::deploy::DeploymentStatus> {
    let root = platform::app_root()?;
    tauri::async_runtime::spawn_blocking(move || crate::deploy::status(&root))
        .await
        .map_err(|_| AppError::new("worker_failed", "读取安装状态异常结束。", "deploy"))?
}

#[tauri::command]
pub async fn preview_maintenance(
    action: String,
    state: State<'_, PendingDeployment>,
) -> Result<crate::setup::ChangePreview> {
    let root = platform::app_root()?;
    let operation = tauri::async_runtime::spawn_blocking(move || match action.as_str() {
        "reset-config" => crate::config::portable::reset(&root)
            .and_then(|p| {
                crate::setup::with_integration(
                    p,
                    crate::config::portable::read_state(&root)?.integration_enabled,
                )
            })
            .map(Box::new)
            .map(PendingOperation::Deployment),
        "uninstall" => crate::setup::prepare_uninstall(&root)
            .map(Box::new)
            .map(PendingOperation::Deployment),
        "recover" => crate::deploy::prepare_recovery(&root)
            .map(Box::new)
            .map(PendingOperation::Recovery),
        _ => Err(AppError::new(
            "invalid_action",
            "不支持的维护操作。",
            "deploy",
        )),
    })
    .await
    .map_err(|_| AppError::new("worker_failed", "维护预览异常结束。", "deploy"))??;
    let preview = operation.preview().clone();
    *state.0.lock().map_err(|_| {
        AppError::new(
            "state_unavailable",
            "操作状态不可用，请重启应用。",
            "deploy",
        )
    })? = Some(operation);
    Ok(preview)
}

#[tauri::command]
pub async fn apply_deployment(
    preview_id: String,
    state: State<'_, PendingDeployment>,
) -> Result<crate::deploy::DeploymentStatus> {
    let operation = {
        let mut pending = state.0.lock().map_err(|_| {
            AppError::new(
                "state_unavailable",
                "操作状态不可用，请重启应用。",
                "deploy",
            )
        })?;
        if pending
            .as_ref()
            .is_none_or(|op| op.preview().files.id != preview_id)
        {
            return Err(AppError::new(
                "stale_preview",
                "没有匹配的部署预览，请重新预览。",
                "deploy",
            ));
        }
        pending.take().expect("checked operation")
    };
    tauri::async_runtime::spawn_blocking(move || match operation {
        PendingOperation::Deployment(plan) => crate::setup::execute(*plan),
        PendingOperation::Recovery(plan) => crate::deploy::recover(*plan),
    })
    .await
    .map_err(|_| {
        AppError::new(
            "worker_failed",
            "维护任务异常结束，请重新检查恢复状态。",
            "deploy",
        )
    })?
}

#[derive(Default)]
pub struct ComponentJob(Mutex<Option<std::sync::Arc<std::sync::atomic::AtomicBool>>>);

#[tauri::command]
pub fn component_sources() -> &'static [crate::components::Asset] {
    crate::components::ASSETS
}

#[tauri::command]
pub fn cancel_component_download(state: State<'_, ComponentJob>) -> Result<()> {
    let job = state
        .0
        .lock()
        .map_err(|_| AppError::new("state_unavailable", "获取状态不可用。", "components"))?;
    if let Some(cancel) = job.as_ref() {
        cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    Ok(())
}

#[tauri::command]
pub async fn preview_install(
    integration_enabled: bool,
    replace_existing: bool,
    on_progress: tauri::ipc::Channel<crate::components::Progress>,
    job: State<'_, ComponentJob>,
    state: State<'_, PendingDeployment>,
) -> Result<crate::setup::ChangePreview> {
    let root = platform::app_root()?;
    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    {
        let mut current = job
            .0
            .lock()
            .map_err(|_| AppError::new("state_unavailable", "获取状态不可用。", "components"))?;
        if current.is_some() {
            return Err(AppError::new(
                "download_busy",
                "已有组件获取任务，请先取消或等待完成。",
                "components",
            ));
        }
        *current = Some(cancel.clone());
    }
    let result = tauri::async_runtime::spawn_blocking(move || {
        crate::components::prepare_install(
            &root,
            integration_enabled,
            replace_existing,
            &cancel,
            &mut |progress| {
                let _ = on_progress.send(progress);
            },
        )
    })
    .await
    .map_err(|_| {
        AppError::new(
            "worker_failed",
            "组件任务异常结束，安装目录未改变。",
            "components",
        )
    });
    *job.0
        .lock()
        .map_err(|_| AppError::new("state_unavailable", "获取状态不可用。", "components"))? = None;
    let files = result??;
    let plan = if crate::config::portable::ready(&files.preview.root)? {
        crate::setup::with_integration(files, integration_enabled)?
    } else {
        crate::setup::files_only(files)
    };
    let preview = plan.preview();
    *state
        .0
        .lock()
        .map_err(|_| AppError::new("state_unavailable", "部署状态不可用。", "components"))? =
        Some(PendingOperation::Deployment(Box::new(plan)));
    Ok(preview)
}

#[tauri::command]
pub async fn read_settings() -> Result<crate::config::Settings> {
    let root = platform::app_root()?;
    tauri::async_runtime::spawn_blocking(move || crate::config::read(&root))
        .await
        .map_err(|_| AppError::new("worker_failed", "读取设置异常结束。", "config"))?
}
#[tauri::command]
pub async fn preview_configuration(
    settings: crate::config::Settings,
    replace_manual: bool,
) -> Result<crate::config::portable::ConfigurationPreview> {
    let root = platform::app_root()?;
    tauri::async_runtime::spawn_blocking(move || {
        crate::config::portable::preview(&root, &settings, replace_manual)
    })
    .await
    .map_err(|_| AppError::new("worker_failed", "配置预览异常结束。", "config"))?
}
#[tauri::command]
pub async fn preview_settings(
    settings: crate::config::Settings,
    replace_manual: bool,
    integration_enabled: bool,
    expected_fingerprint: String,
    state: State<'_, PendingDeployment>,
) -> Result<crate::setup::ChangePreview> {
    let root = platform::app_root()?;
    let plan = tauri::async_runtime::spawn_blocking(move || {
        crate::setup::prepare_draft(
            &root,
            settings,
            replace_manual,
            integration_enabled,
            &expected_fingerprint,
        )
    })
    .await
    .map_err(|_| AppError::new("worker_failed", "设置预览异常结束。", "config"))??;
    let preview = plan.preview();
    *state
        .0
        .lock()
        .map_err(|_| AppError::new("state_unavailable", "部署状态不可用。", "config"))? =
        Some(PendingOperation::Deployment(Box::new(plan)));
    Ok(preview)
}

#[derive(Default)]
pub struct PendingExport(Mutex<Option<crate::diagnostics::ExportPlan>>);
#[tauri::command]
pub async fn preview_diagnostics(
    destination: String,
    state: State<'_, PendingExport>,
) -> Result<crate::diagnostics::ExportPreview> {
    let root = platform::app_root()?;
    let plan = tauri::async_runtime::spawn_blocking(move || {
        crate::diagnostics::prepare(&root, std::path::Path::new(&destination))
    })
    .await
    .map_err(|_| AppError::new("worker_failed", "诊断检查异常结束。", "diagnostics"))??;
    let preview = plan.preview();
    *state
        .0
        .lock()
        .map_err(|_| AppError::new("state_unavailable", "诊断状态不可用。", "diagnostics"))? =
        Some(plan);
    Ok(preview)
}
#[tauri::command]
pub async fn export_diagnostics(
    preview_id: String,
    state: State<'_, PendingExport>,
) -> Result<std::path::PathBuf> {
    let plan = {
        let mut pending = state
            .0
            .lock()
            .map_err(|_| AppError::new("state_unavailable", "诊断状态不可用。", "diagnostics"))?;
        if pending.as_ref().is_none_or(|p| p.id != preview_id) {
            return Err(AppError::new(
                "stale_preview",
                "诊断预览不匹配，请重新生成。",
                "diagnostics",
            ));
        }
        pending.take().expect("matched export")
    };
    tauri::async_runtime::spawn_blocking(move || crate::diagnostics::execute(plan))
        .await
        .map_err(|_| AppError::new("worker_failed", "诊断导出异常结束。", "diagnostics"))?
}

#[derive(Default)]
pub struct PendingIntegration(Mutex<Option<crate::integration::Plan>>);
#[tauri::command]
pub async fn integration_status() -> Result<crate::integration::Status> {
    let root = platform::app_root()?;
    tauri::async_runtime::spawn_blocking(move || crate::integration::status(&root))
        .await
        .map_err(|_| AppError::new("worker_failed", "系统集成状态读取异常。", "integration"))?
}
#[tauri::command]
pub async fn preview_integration(
    action: String,
    state: State<'_, PendingIntegration>,
) -> Result<crate::integration::Preview> {
    let root = platform::app_root()?;
    let plan =
        tauri::async_runtime::spawn_blocking(move || crate::integration::prepare(&root, &action))
            .await
            .map_err(|_| AppError::new("worker_failed", "系统集成预览异常。", "integration"))??;
    let preview = plan.preview.clone();
    *state
        .0
        .lock()
        .map_err(|_| AppError::new("state_unavailable", "系统集成状态不可用。", "integration"))? =
        Some(plan);
    Ok(preview)
}
#[tauri::command]
pub async fn apply_integration(
    preview_id: String,
    state: State<'_, PendingIntegration>,
) -> Result<crate::integration::Status> {
    let plan = {
        let mut pending = state.0.lock().map_err(|_| {
            AppError::new("state_unavailable", "系统集成状态不可用。", "integration")
        })?;
        if pending.as_ref().is_none_or(|p| p.preview.id != preview_id) {
            return Err(AppError::new(
                "stale_preview",
                "系统集成预览不匹配。",
                "integration",
            ));
        }
        pending.take().expect("matched integration")
    };
    tauri::async_runtime::spawn_blocking(move || crate::integration::execute(plan))
        .await
        .map_err(|_| {
            AppError::new(
                "worker_failed",
                "系统集成执行异常，请检查恢复状态。",
                "integration",
            )
        })?
}
#[tauri::command]
pub fn open_default_apps() -> Result<()> {
    platform::open_default_apps()
}
#[cfg(target_os = "windows")]
#[tauri::command]
pub fn open_app_installer() -> Result<()> {
    platform::open_app_installer()
}

#[tauri::command]
pub fn component_catalog() -> serde_json::Value {
    serde_json::from_str(crate::components::CATALOG).expect("embedded component catalog")
}

#[derive(Default)]
pub struct PendingLogCleanup(Mutex<Option<crate::logs::Plan>>);
#[tauri::command]
pub async fn preview_log_cleanup(
    state: State<'_, PendingLogCleanup>,
) -> Result<crate::logs::Preview> {
    let root = platform::app_root()?;
    let plan = tauri::async_runtime::spawn_blocking(move || crate::logs::prepare(&root))
        .await
        .map_err(|_| AppError::new("worker_failed", "日志预览异常结束。", "logs"))??;
    let preview = plan.preview.clone();
    *state
        .0
        .lock()
        .map_err(|_| AppError::new("state_unavailable", "日志状态不可用。", "logs"))? = Some(plan);
    Ok(preview)
}
#[tauri::command]
pub async fn clean_logs(preview_id: String, state: State<'_, PendingLogCleanup>) -> Result<usize> {
    let plan = {
        let mut pending = state
            .0
            .lock()
            .map_err(|_| AppError::new("state_unavailable", "日志状态不可用。", "logs"))?;
        if pending.as_ref().is_none_or(|p| p.preview.id != preview_id) {
            return Err(AppError::new(
                "stale_preview",
                "日志清理预览不匹配。",
                "logs",
            ));
        }
        pending.take().expect("matched logs")
    };
    tauri::async_runtime::spawn_blocking(move || crate::logs::execute(plan))
        .await
        .map_err(|_| AppError::new("worker_failed", "日志清理异常结束。", "logs"))?
}

#[tauri::command]
pub async fn setup_status() -> Result<crate::setup::Report> {
    let root = platform::app_root()?;
    tauri::async_runtime::spawn_blocking(move || crate::setup::status(&root))
        .await
        .map_err(|_| AppError::new("worker_failed", "无法检查安装状态。", "setup"))?
}
#[tauri::command]
pub async fn read_app_settings() -> Result<crate::app_settings::AppSettings> {
    tauri::async_runtime::spawn_blocking(|| crate::app_settings::read(&platform::default_root()?))
        .await
        .map_err(|_| AppError::new("worker_failed", "无法读取应用设置。", "settings"))?
}
#[tauri::command]
pub async fn save_app_settings(
    settings: crate::app_settings::AppSettings,
) -> Result<crate::app_settings::AppSettings> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::app_settings::save(&platform::default_root()?, settings)
    })
    .await
    .map_err(|_| AppError::new("worker_failed", "无法保存应用设置。", "settings"))?
}
#[tauri::command]
pub fn desktop_environment() -> platform::desktop::Environment {
    platform::desktop::environment()
}
#[tauri::command]
pub fn apply_appearance(window: tauri::WebviewWindow) -> Result<bool> {
    Ok(platform::appearance::apply(&window))
}
#[tauri::command]
pub async fn open_config_directory() -> Result<()> {
    tauri::async_runtime::spawn_blocking(|| {
        let root = platform::app_root()?;
        platform::desktop::open_directory(&crate::config::directory(&root)?)
    })
    .await
    .map_err(|_| AppError::new("worker_failed", "无法打开目录。", "paths"))?
}
#[tauri::command]
pub fn app_information() -> serde_json::Value {
    serde_json::json!({"version":env!("CARGO_PKG_VERSION"),"license":"GPL-3.0-or-later","repository":"https://github.com/Leovikii/mpvkit","releaseReady":false,"platform":std::env::consts::OS})
}
#[tauri::command]
pub fn license_text(kind: String) -> Result<&'static str> {
    match kind.as_str() {
        "project" => Ok(include_str!("../../COPYING.md")),
        "license" => Ok(include_str!("../../LICENSE")),
        "dependencies" => {
            #[cfg(target_os = "windows")]
            const NOTICES: &str = concat!(
                include_str!("../../resources/app-dependency-notices.txt"),
                "\n\nLXGW WenKai Screen v1.530 — unmodified subtitle font\n",
                include_str!("../../resources/licenses/LXGWWenKaiScreen-OFL.txt")
            );
            #[cfg(target_os = "linux")]
            const NOTICES: &str = concat!(
                include_str!("../../resources/app-dependency-notices-linux.txt"),
                "\n\nLXGW WenKai Screen v1.530 — unmodified subtitle font\n",
                include_str!("../../resources/licenses/LXGWWenKaiScreen-OFL.txt")
            );
            Ok(NOTICES)
        }
        _ => Err(AppError::new(
            "invalid_action",
            "不支持此声明类型。",
            "settings",
        )),
    }
}

#[tauri::command]
pub async fn preview_integration_intent(
    enabled: bool,
    state: State<'_, PendingDeployment>,
) -> Result<crate::setup::ChangePreview> {
    let root = platform::app_root()?;
    let plan =
        tauri::async_runtime::spawn_blocking(move || crate::setup::prepare_intent(&root, enabled))
            .await
            .map_err(|_| {
                AppError::new("worker_failed", "无法预览打开方式设置。", "integration")
            })??;
    let preview = plan.preview();
    *state
        .0
        .lock()
        .map_err(|_| AppError::new("state_unavailable", "操作状态不可用。", "integration"))? =
        Some(PendingOperation::Deployment(Box::new(plan)));
    Ok(preview)
}
#[tauri::command]
pub async fn open_player_directory() -> Result<()> {
    tauri::async_runtime::spawn_blocking(|| {
        let player = crate::config::player_path(&platform::app_root()?)?;
        platform::desktop::open_directory(
            player
                .parent()
                .ok_or_else(|| AppError::new("invalid_path", "无法获取播放器目录。", "paths"))?,
        )
    })
    .await
    .map_err(|_| AppError::new("worker_failed", "无法打开目录。", "paths"))?
}

#[derive(Default)]
pub struct PendingCore(Mutex<Option<platform::core::CorePlan>>);
#[tauri::command]
pub async fn preview_core(
    action: String,
    state: State<'_, PendingCore>,
) -> Result<platform::core::CorePreview> {
    let plan = tauri::async_runtime::spawn_blocking(move || platform::core::prepare(&action))
        .await
        .map_err(|_| AppError::new("worker_failed", "核心预览异常。", "system-mpv"))??;
    let preview = plan.preview.clone();
    *state
        .0
        .lock()
        .map_err(|_| AppError::new("state_unavailable", "核心状态不可用。", "system-mpv"))? =
        Some(plan);
    Ok(preview)
}
#[tauri::command]
pub async fn apply_core(
    preview_id: String,
    state: State<'_, PendingCore>,
) -> Result<Option<crate::mpv::MpvInfo>> {
    let plan = {
        let mut pending = state
            .0
            .lock()
            .map_err(|_| AppError::new("state_unavailable", "核心状态不可用。", "system-mpv"))?;
        if pending.as_ref().is_none_or(|p| p.preview.id != preview_id) {
            return Err(AppError::new(
                "stale_preview",
                "没有匹配的核心预览。",
                "system-mpv",
            ));
        }
        pending.take().expect("matched core plan")
    };
    tauri::async_runtime::spawn_blocking(move || platform::core::execute(plan))
        .await
        .map_err(|_| {
            AppError::new(
                "worker_failed",
                "核心任务异常，请检查系统安装状态。",
                "system-mpv",
            )
        })?
}
