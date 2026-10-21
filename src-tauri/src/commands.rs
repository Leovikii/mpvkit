use crate::{
    detect::{self, DiscoveryReport},
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
    executable: String,
    media: String,
    managed: Option<bool>,
    state: State<'_, PendingLaunch>,
) -> Result<LaunchPreview> {
    let root = platform::app_root()?;
    let plan = tauri::async_runtime::spawn_blocking(move || {
        if managed.unwrap_or(false) {
            launch::prepare_managed(&root, &media)
        } else {
            launch::prepare(&root, &executable, &media)
        }
    })
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
    tauri::async_runtime::spawn_blocking(move || {
        let _selection = crate::targets::active(&plan.root)?;
        launch::execute(plan)
    })
    .await
    .map_err(|_| {
        AppError::new(
            "worker_failed",
            "启动任务异常结束，请检查播放器窗口。",
            "launch",
        )
    })?
}

#[tauri::command]
pub async fn discover_mpv() -> Result<DiscoveryReport> {
    tauri::async_runtime::spawn_blocking(detect::discover)
        .await
        .map_err(|_| {
            AppError::new(
                "worker_failed",
                "查找任务异常结束，请重试或手动填写路径。",
                "discovery",
            )
        })
}

enum PendingOperation {
    Deployment(crate::deploy::DeploymentPlan),
    Recovery(crate::deploy::RecoveryPlan),
}

impl PendingOperation {
    fn preview(&self) -> &crate::deploy::DeploymentPreview {
        match self {
            Self::Deployment(plan) => &plan.preview,
            Self::Recovery(plan) => &plan.preview,
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
) -> Result<crate::deploy::DeploymentPreview> {
    let root = platform::app_root()?;
    let operation = tauri::async_runtime::spawn_blocking(move || match action.as_str() {
        "uninstall" => crate::deploy::prepare_uninstall(&root).map(PendingOperation::Deployment),
        "restore" => crate::deploy::prepare_restore(&root).map(PendingOperation::Deployment),
        "recover" => crate::deploy::prepare_recovery(&root).map(PendingOperation::Recovery),
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
            .is_none_or(|op| op.preview().id != preview_id)
        {
            return Err(AppError::new(
                "stale_preview",
                "没有匹配的部署预览，请重新预览。",
                "deploy",
            ));
        }
        pending.take().expect("checked operation")
    };
    tauri::async_runtime::spawn_blocking(move || {
        let _selection = crate::targets::active(&operation.preview().root)?;
        match operation {
            PendingOperation::Deployment(plan) => {
                let target = crate::targets::read(&platform::default_root()?)?;
                if target.config_directory != plan.config_directory()? {
                    return Err(AppError::new(
                        "target_changed",
                        "目标布局已切换，请重新预览。",
                        "deploy",
                    ));
                }
                crate::setup::remember_completed_legacy(&plan.preview.root)?;
                crate::deploy::execute(plan)
            }
            PendingOperation::Recovery(plan) => crate::deploy::recover(plan),
        }
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
    external_mpv: Option<String>,
    replace_existing: bool,
    on_progress: tauri::ipc::Channel<crate::components::Progress>,
    job: State<'_, ComponentJob>,
    state: State<'_, PendingDeployment>,
) -> Result<crate::deploy::DeploymentPreview> {
    let target = crate::targets::read(&platform::default_root()?)?;
    let root = target.root;
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
        crate::components::prepare_install_at(
            &root,
            external_mpv.as_deref(),
            replace_existing,
            &cancel,
            &mut |progress| {
                let _ = on_progress.send(progress);
            },
            &target.config_directory,
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
    let plan = result??;
    let preview = plan.preview.clone();
    *state
        .0
        .lock()
        .map_err(|_| AppError::new("state_unavailable", "部署状态不可用。", "components"))? =
        Some(PendingOperation::Deployment(plan));
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
pub async fn preview_settings(
    settings: crate::config::Settings,
    state: State<'_, PendingDeployment>,
) -> Result<crate::deploy::DeploymentPreview> {
    let root = platform::app_root()?;
    let plan = tauri::async_runtime::spawn_blocking(move || {
        crate::setup::require_components(&root)?;
        crate::config::prepare(&root, settings)
    })
    .await
    .map_err(|_| AppError::new("worker_failed", "设置预览异常结束。", "config"))??;
    let preview = plan.preview.clone();
    *state
        .0
        .lock()
        .map_err(|_| AppError::new("state_unavailable", "部署状态不可用。", "config"))? =
        Some(PendingOperation::Deployment(plan));
    Ok(preview)
}

#[tauri::command]
pub async fn detect_device(window: tauri::WebviewWindow) -> Result<crate::device::DeviceReport> {
    let handle = window.hwnd().map(|h| h.0 as isize).unwrap_or(0);
    tauri::async_runtime::spawn_blocking(move || platform::detect_device_for_window(handle))
        .await
        .map_err(|_| AppError::new("worker_failed", "设备检测异常结束。", "device"))
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
    let plan = tauri::async_runtime::spawn_blocking(move || {
        if action == "register" {
            crate::setup::require_components(&root)?;
        }
        crate::integration::prepare(&root, &action)
    })
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
    tauri::async_runtime::spawn_blocking(move || {
        let _selection = crate::targets::active(&plan.root)?;
        crate::integration::execute(plan)
    })
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

#[tauri::command]
pub fn component_catalog() -> serde_json::Value {
    serde_json::from_str(include_str!("../../resources/catalog.json"))
        .expect("embedded component catalog")
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
    tauri::async_runtime::spawn_blocking(move || {
        let _selection = crate::targets::active(&plan.root)?;
        crate::logs::execute(plan)
    })
    .await
    .map_err(|_| AppError::new("worker_failed", "日志清理异常结束。", "logs"))?
}

#[derive(Default)]
pub struct PendingTarget(Mutex<Option<crate::targets::Plan>>);
#[tauri::command]
pub async fn target_status() -> Result<crate::targets::Status> {
    tauri::async_runtime::spawn_blocking(|| crate::targets::status(&platform::default_root()?))
        .await
        .map_err(|_| AppError::new("worker_failed", "目标状态读取异常。", "target"))?
}
#[tauri::command]
pub async fn preview_target(
    root: String,
    state: State<'_, PendingTarget>,
) -> Result<crate::targets::Preview> {
    let plan = tauri::async_runtime::spawn_blocking(move || {
        crate::targets::prepare(&platform::default_root()?, root.into())
    })
    .await
    .map_err(|_| AppError::new("worker_failed", "目标预览异常。", "target"))??;
    let preview = plan.preview.clone();
    *state
        .0
        .lock()
        .map_err(|_| AppError::new("state_unavailable", "目标状态不可用。", "target"))? =
        Some(plan);
    Ok(preview)
}
#[tauri::command]
pub async fn apply_target(
    preview_id: String,
    state: State<'_, PendingTarget>,
) -> Result<crate::targets::Target> {
    let plan = {
        let mut pending = state
            .0
            .lock()
            .map_err(|_| AppError::new("state_unavailable", "目标状态不可用。", "target"))?;
        if pending.as_ref().is_none_or(|p| p.preview.id != preview_id) {
            return Err(AppError::new("stale_preview", "目标预览不匹配。", "target"));
        }
        pending.take().expect("matched target")
    };
    tauri::async_runtime::spawn_blocking(move || crate::targets::execute(plan))
        .await
        .map_err(|_| AppError::new("worker_failed", "目标切换异常。", "target"))?
}

#[derive(Default)]
pub struct PendingAssetCleanup(Mutex<Option<crate::deploy::cleanup::Plan>>);
#[tauri::command]
pub async fn preview_asset_cleanup(
    state: State<'_, PendingAssetCleanup>,
) -> Result<crate::deploy::cleanup::Preview> {
    let root = platform::app_root()?;
    let plan = tauri::async_runtime::spawn_blocking(move || crate::deploy::cleanup::prepare(&root))
        .await
        .map_err(|_| AppError::new("worker_failed", "清理预览异常。", "cleanup"))??;
    let preview = plan.preview.clone();
    *state
        .0
        .lock()
        .map_err(|_| AppError::new("state_unavailable", "清理状态不可用。", "cleanup"))? =
        Some(plan);
    Ok(preview)
}
#[tauri::command]
pub async fn clean_assets(
    preview_id: String,
    state: State<'_, PendingAssetCleanup>,
) -> Result<usize> {
    let plan = {
        let mut pending = state
            .0
            .lock()
            .map_err(|_| AppError::new("state_unavailable", "清理状态不可用。", "cleanup"))?;
        if pending.as_ref().is_none_or(|p| p.preview.id != preview_id) {
            return Err(AppError::new(
                "stale_preview",
                "清理预览不匹配。",
                "cleanup",
            ));
        }
        pending.take().expect("matched cleanup")
    };
    tauri::async_runtime::spawn_blocking(move || {
        let _selection = crate::targets::active(&plan.preview.root)?;
        crate::deploy::cleanup::execute(plan)
    })
    .await
    .map_err(|_| AppError::new("worker_failed", "清理异常结束，请重新预览。", "cleanup"))?
}

#[tauri::command]
pub async fn setup_status() -> Result<crate::setup::Report> {
    let root = platform::app_root()?;
    tauri::async_runtime::spawn_blocking(move || crate::setup::status(&root))
        .await
        .map_err(|_| AppError::new("worker_failed", "无法检查安装状态。", "setup"))?
}
#[tauri::command]
pub async fn confirm_setup(
    root: String,
    generation: String,
    finish: bool,
) -> Result<crate::setup::Report> {
    tauri::async_runtime::spawn_blocking(move || {
        let root = std::path::PathBuf::from(root);
        let _selection = crate::targets::active(&root)?;
        crate::setup::confirm(&root, &generation, finish)
    })
    .await
    .map_err(|_| AppError::new("worker_failed", "无法保存引导进度。", "setup"))?
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
pub async fn choose_path(
    folder: bool,
    window: tauri::WebviewWindow,
) -> Result<Option<std::path::PathBuf>> {
    let owner = window
        .hwnd()
        .map_err(|_| AppError::new("window_unavailable", "窗口不可用。", "paths"))?
        .0 as isize;
    tauri::async_runtime::spawn_blocking(move || platform::desktop::choose_path(folder, owner))
        .await
        .map_err(|_| AppError::new("worker_failed", "选择器异常结束。", "paths"))?
}
#[tauri::command]
pub async fn open_config_directory() -> Result<()> {
    tauri::async_runtime::spawn_blocking(|| {
        let root = platform::app_root()?;
        let _selection = crate::targets::active(&root)?;
        platform::desktop::open_directory(&crate::config::directory(&root)?)
    })
    .await
    .map_err(|_| AppError::new("worker_failed", "无法打开目录。", "paths"))?
}
#[tauri::command]
pub fn app_information() -> serde_json::Value {
    serde_json::json!({"version":env!("CARGO_PKG_VERSION"),"license":"GPL-3.0-or-later","repository":null,"releaseReady":false})
}
#[tauri::command]
pub fn license_text(kind: String) -> Result<&'static str> {
    match kind.as_str() {
        "project" => Ok(include_str!("../../COPYING.md")),
        "license" => Ok(include_str!("../../LICENSE")),
        "dependencies" => Ok(include_str!("../../resources/app-dependency-notices.txt")),
        _ => Err(AppError::new(
            "invalid_action",
            "不支持此声明类型。",
            "settings",
        )),
    }
}
