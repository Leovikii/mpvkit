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
    let header = gtk::HeaderBar::new();
    header.set_title(Some("MpvKit"));
    header.set_show_close_button(true);
    if let Some(icon) = window.app_handle().default_window_icon() {
        let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_bytes(
            &glib::Bytes::from(icon.rgba()),
            gtk::gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            icon.width() as i32,
            icon.height() as i32,
            icon.width() as i32 * 4,
        );
        let image = gtk::Image::from_pixbuf(
            pixbuf
                .scale_simple(20, 20, gtk::gdk_pixbuf::InterpType::Bilinear)
                .as_ref(),
        );
        image.set_margin_start(4);
        image.set_margin_end(4);
        header.pack_start(&image);
    }
    header.style_context().add_class("mpvkit-titlebar");
    native.style_context().add_class("mpvkit-window");
    native.set_titlebar(Some(&header));
    header.show_all();
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
    let (rgb, text) = if dark {
        ("32,32,32", "#ffffff")
    } else {
        ("243,243,243", "#000000")
    };
    let background = format!("rgba({rgb},{})", if material { "0.82" } else { "1" });
    let css = format!(
        "window.mpvkit-window {{ background-color: {background}; }}
         window.mpvkit-window .mpvkit-titlebar {{ background: {background}; color: {text}; border: 0; box-shadow: none; min-height: 32px; padding: 0 8px; }}
         window.mpvkit-window .mpvkit-titlebar label,
         window.mpvkit-window .mpvkit-titlebar button {{ color: {text}; }}"
    );
    if let Ok(native) = window.gtk_window() {
        let provider = gtk::CssProvider::new();
        if provider.load_from_data(css.as_bytes()).is_ok() {
            replace_provider(&native.style_context(), &provider);
            if let Some(header) = native.titlebar() {
                replace_provider(&header.style_context(), &provider);
            }
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
