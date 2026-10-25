use tauri::Emitter;
pub fn apply(window: &tauri::WebviewWindow) -> bool {
    let settings = super::default_root()
        .and_then(|root| crate::app_settings::read(&root))
        .unwrap_or_default();
    let dark = if settings.theme == "system" {
        super::desktop::system_dark()
    } else {
        settings.theme == "dark"
    };
    let theme = if dark {
        tauri::Theme::Dark
    } else {
        tauri::Theme::Light
    };
    if window.theme().ok() != Some(theme) {
        let _ = window.set_theme(Some(theme));
    }
    let color = if dark {
        tauri::window::Color(32, 32, 32, 255)
    } else {
        tauri::window::Color(243, 243, 243, 255)
    };
    let _ = window.set_background_color(Some(color));
    let _ = window.emit("appearance-changed", false);
    false
}
