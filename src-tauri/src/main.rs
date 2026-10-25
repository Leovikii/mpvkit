#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(all(target_os = "windows", target_arch = "x86_64")))]
compile_error!("MpvKit v0.1.1 currently supports Windows x64 only.");

mod app_settings;
mod commands;
pub mod components;
mod config;
pub mod deploy;
mod diagnostics;
mod error;
mod integration;
mod launch;
mod logs;
mod mpv;
mod platform;
mod preferences;
mod setup;

use tauri::Manager;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).is_some_and(|a| a == "--save-playback") {
        std::process::exit(preferences::helper(&args));
    }
    if args.get(1).is_some_and(|a| a == "--open") {
        let result = (|| {
            let media = args.get(2).filter(|_| args.len() == 3).ok_or_else(|| {
                error::AppError::new("media_required", "请选择一个本地媒体文件。", "launch")
            })?;
            let root = platform::app_root()?;
            launch::execute(launch::prepare_managed(&root, media)?)
        })();
        if let Err(e) = result {
            platform::show_error(e.message);
            std::process::exit(1);
        }
        return;
    }
    tauri::Builder::default()
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                platform::appearance::apply(&window);
                window.show()?;
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if matches!(
                event,
                tauri::WindowEvent::Focused(_) | tauri::WindowEvent::ThemeChanged(_)
            ) && let Some(webview) = window.get_webview_window(window.label())
            {
                platform::appearance::apply(&webview);
            }
        })
        .manage(commands::PendingAssetCleanup::default())
        .manage(commands::PendingLaunch::default())
        .manage(commands::PendingDeployment::default())
        .manage(commands::ComponentJob::default())
        .manage(commands::PendingExport::default())
        .manage(commands::PendingIntegration::default())
        .manage(commands::PendingLogCleanup::default())
        .invoke_handler(tauri::generate_handler![
            commands::setup_status,
            commands::read_app_settings,
            commands::save_app_settings,
            commands::desktop_environment,
            commands::apply_appearance,
            commands::choose_path,
            commands::open_config_directory,
            commands::open_player_directory,
            commands::preview_integration_intent,
            commands::app_information,
            commands::license_text,
            commands::preview_asset_cleanup,
            commands::clean_assets,
            commands::preview_log_cleanup,
            commands::clean_logs,
            commands::integration_status,
            commands::preview_integration,
            commands::apply_integration,
            commands::open_default_apps,
            commands::preview_diagnostics,
            commands::export_diagnostics,
            commands::read_settings,
            commands::preview_settings,
            commands::preview_configuration,
            commands::component_sources,
            commands::component_catalog,
            commands::cancel_component_download,
            commands::preview_install,
            commands::deployment_status,
            commands::preview_maintenance,
            commands::apply_deployment,
            commands::preview_launch,
            commands::launch_mpv
        ])
        .run(tauri::generate_context!())
        .expect("MpvKit 无法启动桌面窗口");
}
