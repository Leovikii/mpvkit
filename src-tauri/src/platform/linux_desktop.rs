use crate::error::{AppError, Result};
use gtk::prelude::*;
use gtk::{gio, glib};
use serde::Serialize;
use std::path::Path;
use std::time::Duration;
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub languages: Vec<String>,
    pub dark: bool,
    pub high_contrast: bool,
    pub material_allowed: bool,
}
pub fn initialize_identity() -> Result<()> {
    // GTK's Wayland fallback uses the program name; set it before GTK initializes.
    gtk::glib::set_prgname(Some("io.github.mpvkit"));
    // The single-instance plugin cannot enforce exclusivity without the session bus.
    gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).map_err(|_| {
        AppError::new(
            "session_bus_unavailable",
            "无法连接桌面会话总线。",
            "startup",
        )
    })?;
    Ok(())
}
fn locale_languages<'a>(preferences: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    for value in preferences {
        let languages: Vec<_> = value
            .split(':')
            .filter_map(|s| {
                let s = s.trim().split(['.', '@']).next().unwrap_or_default();
                match s {
                    "" => None,
                    "C" | "POSIX" => Some("en".into()),
                    _ => Some(s.replace('_', "-")),
                }
            })
            .collect();
        if !languages.is_empty() {
            return languages;
        }
    }
    vec!["en".into()]
}
fn system_languages() -> Vec<String> {
    let mut preferences = vec![];
    if std::env::var("XDG_CURRENT_DESKTOP").is_ok_and(|s| s.split(':').any(|s| s == "KDE")) {
        // KDE's desktop language is independent of a terminal/launcher locale override.
        for (group, key) in [("Translations", "LANGUAGE"), ("Formats", "LANG")] {
            if let Ok(value) = crate::mpv::capture(
                Path::new("/usr/bin/kreadconfig6"),
                &[
                    "--file".into(),
                    "plasma-localerc".into(),
                    "--group".into(),
                    group.into(),
                    "--key".into(),
                    key.into(),
                ],
                Duration::from_secs(1),
            ) {
                preferences.push(value);
            }
        }
    }
    preferences.extend(
        ["LANGUAGE", "LC_ALL", "LC_MESSAGES", "LANG"]
            .into_iter()
            .filter_map(|key| std::env::var(key).ok()),
    );
    locale_languages(preferences.iter().map(String::as_str))
}
fn portal_dark(reply: &glib::Variant) -> Option<bool> {
    let (value,) = reply.get::<(glib::Variant,)>()?;
    match value.get::<u32>()? {
        1 => Some(true),
        2 => Some(false),
        _ => None,
    }
}
fn portal_color_scheme() -> Option<bool> {
    let connection = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).ok()?;
    let reply = connection
        .call_sync(
            Some("org.freedesktop.portal.Desktop"),
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Settings",
            "ReadOne",
            Some(&("org.freedesktop.appearance", "color-scheme").to_variant()),
            Some(glib::VariantTy::new("(v)").expect("portal reply type")),
            gio::DBusCallFlags::NO_AUTO_START,
            500,
            gio::Cancellable::NONE,
        )
        .ok()?;
    portal_dark(&reply)
}
pub fn system_dark() -> bool {
    portal_color_scheme().unwrap_or_else(|| {
        let settings = gtk::Settings::default();
        settings
            .as_ref()
            .is_some_and(|s| s.is_gtk_application_prefer_dark_theme())
            || settings
                .and_then(|s| s.gtk_theme_name())
                .is_some_and(|name| name.to_lowercase().contains("dark"))
    })
}
pub fn environment() -> Environment {
    let settings = gtk::Settings::default();
    let theme = settings
        .as_ref()
        .and_then(|s| s.gtk_theme_name())
        .unwrap_or_default();
    Environment {
        languages: system_languages(),
        dark: system_dark(),
        high_contrast: theme.to_lowercase().contains("highcontrast"),
        material_allowed: false,
    }
}

pub fn open_directory(path: &Path) -> Result<()> {
    super::validate_local_path(path)?;
    if !path.is_dir() {
        return Err(AppError::new(
            "directory_missing",
            "目录尚未创建。",
            "paths",
        ));
    }
    crate::mpv::capture(
        Path::new("/usr/bin/xdg-open"),
        &[path.as_os_str().to_owned()],
        std::time::Duration::from_secs(5),
    )
    .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn desktop_language_precedes_launcher_locale_and_empty_values_are_skipped() {
        assert_eq!(
            locale_languages(["", " \n", "zh_CN.UTF-8", "C.UTF-8"]),
            ["zh-CN"]
        );
        assert_eq!(
            locale_languages(["en_US:zh_CN", "zh_CN.UTF-8"]),
            ["en-US", "zh-CN"]
        );
        assert_eq!(locale_languages(["::", "zh_SG.UTF-8@modifier"]), ["zh-SG"]);
        assert_eq!(locale_languages(["C.UTF-8", "zh_CN.UTF-8"]), ["en"]);
        assert_eq!(locale_languages([]), ["en"]);
    }
    #[test]
    fn portal_handles_dark_light_and_unknown_preferences() {
        for (scheme, expected) in [(1u32, Some(true)), (2, Some(false)), (0, None), (9, None)] {
            let value = scheme.to_variant();
            assert_eq!(portal_dark(&(value,).to_variant()), expected);
        }
        assert_eq!(portal_dark(&("invalid".to_variant(),).to_variant()), None);
    }
    #[test]
    #[ignore = "requires the current KDE session bus and desktop configuration"]
    fn kde_session_preferences_are_available() {
        gtk::init().expect("Wayland GTK initialization");
        let dark = portal_color_scheme().expect("Settings portal color scheme");
        let settings = gtk::Settings::default().unwrap();
        // Change only this process, deliberately disagreeing with the desktop portal.
        settings.set_gtk_application_prefer_dark_theme(!dark);
        let environment = environment();
        assert_eq!(environment.dark, dark);
        assert!(!environment.languages.is_empty());
        println!(
            "KDE system preferences: {}",
            serde_json::to_string(&environment).unwrap()
        );
    }
}
