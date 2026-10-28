#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(all(
    any(target_os = "windows", target_os = "linux"),
    target_arch = "x86_64"
)))]
compile_error!("MpvKit supports Windows x64 and Arch Linux x64 / Wayland.");

mod app_settings;
mod commands;
pub mod components;
mod config;
pub mod deploy;
mod diagnostics;
mod error;
#[cfg(all(test, target_os = "linux"))]
mod linux_tests;
mod mpv;
mod platform;
mod preferences;
mod setup;
mod updates;

use tauri::Manager;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).is_some_and(|a| a == "--save-playback") {
        std::process::exit(preferences::helper(&args));
    }
    #[cfg(target_os = "linux")]
    {
        platform::desktop::initialize_identity().expect("MpvKit 无法连接桌面会话总线");
    }
    let builder =
        tauri::Builder::default().plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }));
    let builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    let builder = builder.manage(commands::PendingCore::default());
    builder
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
        .manage(commands::PendingDeployment::default())
        .manage(commands::ComponentJob::default())
        .manage(commands::PendingExport::default())
        .manage(updates::Pending::default())
        .invoke_handler(tauri::generate_handler![
            commands::setup_status,
            commands::preview_core,
            commands::check_core_update,
            commands::apply_core,
            commands::read_app_settings,
            commands::save_app_settings,
            commands::desktop_environment,
            commands::apply_appearance,
            commands::open_config_directory,
            commands::open_player_directory,
            commands::app_information,
            commands::license_text,
            updates::check_app_update,
            updates::install_app_update,
            updates::prepare_app_update,
            commands::open_default_apps,
            #[cfg(target_os = "windows")]
            commands::open_app_installer,
            commands::preview_diagnostics,
            commands::export_diagnostics,
            commands::read_settings,
            commands::preview_settings,
            commands::preview_configuration,
            commands::component_sources,
            commands::cancel_component_download,
            commands::preview_install,
            commands::deployment_status,
            commands::preview_maintenance,
            commands::apply_deployment
        ])
        .run(tauri::generate_context!())
        .expect("MpvKit 无法启动桌面窗口");
}
