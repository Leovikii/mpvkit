use super::desktop;
use tauri::Emitter;
use windows_sys::Win32::{
    Foundation::HWND,
    Graphics::{
        Dwm::*,
        Gdi::{COLOR_WINDOW, GetSysColor},
    },
    UI::Controls::MARGINS,
};

// These calls run on Tauri's main thread (setup, native window events and sync commands).
fn set_attribute(hwnd: HWND, attribute: u32, value: u32) -> std::result::Result<(), i32> {
    let result =
        unsafe { DwmSetWindowAttribute(hwnd, attribute, (&value as *const u32).cast(), 4) };
    if result < 0 { Err(result) } else { Ok(()) }
}

fn backdrop(hwnd: HWND, enabled: bool) -> std::result::Result<(), i32> {
    let expected = if enabled {
        DWMSBT_MAINWINDOW
    } else {
        DWMSBT_NONE
    } as u32;
    set_attribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE as u32, expected)?;
    let mut actual = 0u32;
    let result = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE as u32,
            (&mut actual as *mut u32).cast(),
            4,
        )
    };
    if result < 0 {
        return Err(result);
    }
    if actual != expected {
        return Err(0x80004005u32 as i32);
    }
    Ok(())
}

fn native(hwnd: HWND, dark: bool, allowed: bool, high_contrast: bool) -> bool {
    let dark = dark && !high_contrast;
    let _ = set_attribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE as u32, dark as u32);
    // Restore system caption colors before enabling the system material.
    let default_color = DWMWA_COLOR_DEFAULT;
    let material = if allowed && !high_contrast {
        let result = (|| {
            set_attribute(hwnd, DWMWA_CAPTION_COLOR as u32, default_color)?;
            set_attribute(hwnd, DWMWA_TEXT_COLOR as u32, default_color)?;
            let margins = MARGINS {
                cxLeftWidth: -1,
                cxRightWidth: -1,
                cyTopHeight: -1,
                cyBottomHeight: -1,
            };
            let result = unsafe { DwmExtendFrameIntoClientArea(hwnd, &margins) };
            if result < 0 {
                return Err(result);
            }
            backdrop(hwnd, true)
        })();
        if let Err(code) = result {
            eprintln!("appearance: Mica rejected, HRESULT=0x{:08X}", code as u32);
        }
        result.is_ok()
    } else {
        false
    };
    if !material {
        // Unsupported older Windows may reject these attributes; its native titlebar remains usable.
        let _ = backdrop(hwnd, false);
        let margins = MARGINS {
            cxLeftWidth: 0,
            cxRightWidth: 0,
            cyTopHeight: 0,
            cyBottomHeight: 0,
        };
        unsafe {
            DwmExtendFrameIntoClientArea(hwnd, &margins);
        }
        let caption = if high_contrast {
            default_color
        } else if dark {
            0x202020
        } else {
            0xf3f3f3
        };
        let text = if high_contrast {
            default_color
        } else if dark {
            0xffffff
        } else {
            0x000000
        };
        if let Err(code) = set_attribute(hwnd, DWMWA_CAPTION_COLOR as u32, caption)
            .and_then(|_| set_attribute(hwnd, DWMWA_TEXT_COLOR as u32, text))
        {
            eprintln!(
                "appearance: native caption fallback unavailable, HRESULT=0x{:08X}",
                code as u32
            );
        }
    }
    material
}

pub fn apply(window: &tauri::WebviewWindow) -> bool {
    let environment = desktop::environment();
    // Invalid preference files must remain visible/recoverable, not prevent the window opening.
    let settings = super::default_root()
        .and_then(|root| crate::app_settings::read(&root))
        .unwrap_or_default();
    let dark = if settings.theme == "system" {
        environment.dark
    } else {
        settings.theme == "dark"
    };
    let theme = if environment.high_contrast {
        Some(tauri::Theme::Light)
    } else if settings.theme == "system" {
        None
    } else {
        Some(if dark {
            tauri::Theme::Dark
        } else {
            tauri::Theme::Light
        })
    };
    let _ = window.set_theme(theme);
    let material = window.hwnd().is_ok_and(|handle| {
        native(
            handle.0 as HWND,
            dark,
            environment.material_allowed,
            environment.high_contrast,
        )
    });
    let color = if material {
        tauri::window::Color(0, 0, 0, 0)
    } else if environment.high_contrast {
        let rgb = unsafe { GetSysColor(COLOR_WINDOW) };
        tauri::window::Color(rgb as u8, (rgb >> 8) as u8, (rgb >> 16) as u8, 255)
    } else if dark {
        tauri::window::Color(32, 32, 32, 255)
    } else {
        tauri::window::Color(243, 243, 243, 255)
    };
    let _ = window.set_background_color(Some(color));
    // True means DWM accepted and reports MAINWINDOW; OS may visually flatten it while inactive.
    let _ = window.emit("appearance-changed", material);
    material
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    #[test]
    fn hidden_native_window_supports_material_and_caption_fallback_without_focus() {
        if !desktop::environment().material_allowed {
            return;
        }
        let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                class.as_ptr(),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                320,
                240,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            )
        };
        assert!(!hwnd.is_null());
        let result = std::panic::catch_unwind(|| {
            for dark in [false, true] {
                assert!(!native(hwnd, dark, false, false));
                let mut actual = 0u32;
                assert!(
                    unsafe {
                        DwmGetWindowAttribute(
                            hwnd,
                            DWMWA_SYSTEMBACKDROP_TYPE as u32,
                            (&mut actual as *mut u32).cast(),
                            4,
                        )
                    } >= 0
                );
                assert_eq!(actual, DWMSBT_NONE as u32);
                // Caption colors are set-only attributes; validate their HRESULT, not a readback.
                assert!(
                    set_attribute(
                        hwnd,
                        DWMWA_CAPTION_COLOR as u32,
                        if dark { 0x202020 } else { 0xf3f3f3 }
                    )
                    .is_ok()
                );
                assert!(native(hwnd, dark, true, false));
                assert!(
                    unsafe {
                        DwmGetWindowAttribute(
                            hwnd,
                            DWMWA_SYSTEMBACKDROP_TYPE as u32,
                            (&mut actual as *mut u32).cast(),
                            4,
                        )
                    } >= 0
                );
                assert_eq!(actual, DWMSBT_MAINWINDOW as u32);
            }
            assert!(!native(hwnd, false, false, true));
        });
        unsafe {
            DestroyWindow(hwnd);
        }
        result.unwrap();
        assert!(backdrop(std::ptr::null_mut(), true).is_err());
    }
}
