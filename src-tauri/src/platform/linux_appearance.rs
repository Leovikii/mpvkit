use gtk::{glib, prelude::*};
use std::{cell::RefCell, time::Duration};
use tauri::{Emitter, Manager};

#[path = "blur.rs"]
mod blur;

thread_local! {
    static BLUR: RefCell<Option<blur::Blur>> = const { RefCell::new(None) };
}

pub fn initialize(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    let native = window.gtk_window()?;
    // Let KWin provide the user's window decoration, buttons and shadow.
    native.set_titlebar(None::<&gtk::Widget>);
    native.set_decorated(true);
    native.style_context().add_class("mpvkit-window");
    if let (Some(content), Some(config)) = (
        native.child(),
        window
            .app_handle()
            .config()
            .app
            .windows
            .iter()
            .find(|c| c.label == window.label()),
    ) {
        // Keep the content minimum independent of compositor decoration dimensions.
        content.set_size_request(
            config.min_width.map_or(-1, |v| v as i32),
            config.min_height.map_or(-1, |v| v as i32),
        );
    }
    let handle = window.clone();
    native.connect_realize(move |native| {
        BLUR.with(|state| *state.borrow_mut() = blur::Blur::new(native));
        apply(&handle);
    });
    let handle = window.clone();
    // GTK reads the shared connection; dispatch only our pending events, without blocking.
    glib::timeout_add_local(Duration::from_millis(250), move || {
        if handle.gtk_window().is_err() {
            BLUR.with(|state| state.borrow_mut().take());
            return glib::ControlFlow::Break;
        }
        let changed = BLUR.with(|state| state.borrow_mut().as_mut().is_some_and(blur::Blur::poll));
        if changed {
            apply(&handle);
        }
        glib::ControlFlow::Continue
    });
    Ok(())
}

pub fn apply(window: &tauri::WebviewWindow) -> bool {
    let environment = super::desktop::environment();
    let settings = super::default_root()
        .and_then(|root| crate::app_settings::read(&root))
        .unwrap_or_default();
    let dark = !environment.high_contrast
        && if settings.theme == "system" {
            environment.dark
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
    let material = BLUR.with(|state| {
        state
            .borrow_mut()
            .as_mut()
            .is_some_and(|blur| blur.apply(!environment.high_contrast))
    });
    let rgb = if dark { "32,32,32" } else { "243,243,243" };
    let background = format!("rgba({rgb},{})", if material { "0.82" } else { "1" });
    let css = format!("window.mpvkit-window {{ background-color: {background}; }}");
    if let Ok(native) = window.gtk_window() {
        let provider = gtk::CssProvider::new();
        if provider.load_from_data(css.as_bytes()).is_ok() {
            replace_provider(&native.style_context(), &provider);
        }
        if let Some(surface) = native.window() {
            surface.set_opaque_region(None);
        }
    }
    let color = if material {
        tauri::window::Color(0, 0, 0, 0)
    } else if dark {
        tauri::window::Color(32, 32, 32, 255)
    } else {
        tauri::window::Color(243, 243, 243, 255)
    };
    // WebviewWindow also clears GTK's background; only WebKit should be transparent.
    let webview: &tauri::Webview = window.as_ref();
    let _ = webview.set_background_color(Some(color));
    let _ = window.emit("appearance-changed", material);
    material
}

fn replace_provider(context: &gtk::StyleContext, provider: &gtk::CssProvider) {
    // The context owns this process-local provider for precisely its widget's lifetime.
    unsafe {
        if let Some(old) = context.steal_data::<gtk::CssProvider>("mpvkit-appearance") {
            context.remove_provider(&old);
        }
        context.add_provider(provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        context.set_data("mpvkit-appearance", provider.clone());
    }
}
