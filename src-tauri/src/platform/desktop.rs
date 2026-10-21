use crate::error::{AppError, Result};
use serde::Serialize;
use std::{
    ffi::OsString,
    os::windows::ffi::OsStringExt,
    path::{Path, PathBuf},
};
use windows_sys::Win32::{
    Globalization::*,
    System::{Com::*, Registry::*},
    UI::{Accessibility::*, Controls::Dialogs::*, Shell::*, WindowsAndMessaging::*},
};

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn text(s: &[u16]) -> String {
    String::from_utf16_lossy(&s[..s.iter().position(|x| *x == 0).unwrap_or(s.len())])
}
fn dword(key: &str, name: &str) -> Option<u32> {
    let mut value = 0u32;
    let mut size = 4;
    let ok = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            wide(key).as_ptr(),
            wide(name).as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            (&mut value as *mut u32).cast(),
            &mut size,
        )
    };
    (ok == 0).then_some(value)
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub languages: Vec<String>,
    pub dark: bool,
    pub accent: String,
    pub high_contrast: bool,
    pub material_allowed: bool,
}
pub fn environment() -> Environment {
    let (mut count, mut length) = (0, 0);
    unsafe {
        GetUserPreferredUILanguages(
            MUI_LANGUAGE_NAME,
            &mut count,
            std::ptr::null_mut(),
            &mut length,
        );
    }
    let mut languages = Vec::new();
    if length > 0 && length <= 65536 {
        let mut buffer = vec![0u16; length as usize];
        if unsafe {
            GetUserPreferredUILanguages(
                MUI_LANGUAGE_NAME,
                &mut count,
                buffer.as_mut_ptr(),
                &mut length,
            )
        } != 0
        {
            languages = buffer
                .split(|x| *x == 0)
                .filter(|s| !s.is_empty())
                .map(String::from_utf16_lossy)
                .collect();
        }
    }
    let mut contrast = HIGHCONTRASTW {
        cbSize: std::mem::size_of::<HIGHCONTRASTW>() as u32,
        ..Default::default()
    };
    unsafe {
        SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            contrast.cbSize,
            (&mut contrast as *mut HIGHCONTRASTW).cast(),
            0,
        );
    }
    let high_contrast = contrast.dwFlags & HCF_HIGHCONTRASTON != 0;
    let personalize = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
    let dark = dword(personalize, "AppsUseLightTheme") == Some(0);
    let color = dword(r"Software\Microsoft\Windows\DWM", "ColorizationColor").unwrap_or(0xff0078d4);
    let rgb = [
        ((color >> 16) & 255) as i32,
        ((color >> 8) & 255) as i32,
        (color & 255) as i32,
    ];
    let palette = [
        ("blue", [34, 139, 230]),
        ("teal", [18, 184, 134]),
        ("violet", [121, 80, 242]),
        ("orange", [253, 126, 20]),
    ];
    let accent = palette
        .iter()
        .min_by_key(|(_, c)| c.iter().zip(rgb).map(|(a, b)| (a - b).pow(2)).sum::<i32>())
        .unwrap()
        .0
        .into();
    let mut power = windows_sys::Win32::System::Power::SYSTEM_POWER_STATUS::default();
    let power_ok =
        unsafe { windows_sys::Win32::System::Power::GetSystemPowerStatus(&mut power) } != 0;
    let mut build_buffer = [0u16; 32];
    let mut build_size = 64u32;
    let build_ok = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            wide(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion").as_ptr(),
            wide("CurrentBuildNumber").as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            build_buffer.as_mut_ptr().cast(),
            &mut build_size,
        )
    } == 0;
    let supported_build = build_ok && text(&build_buffer).parse::<u32>().is_ok_and(|b| b >= 22621);
    Environment {
        languages,
        dark,
        accent,
        high_contrast,
        material_allowed: supported_build
            && !high_contrast
            && dword(personalize, "EnableTransparency") == Some(1)
            && power_ok
            && power.SystemStatusFlag == 0
            && unsafe { GetSystemMetrics(SM_REMOTESESSION) } == 0,
    }
}
pub fn choose_path(folder: bool, owner: isize) -> Result<Option<PathBuf>> {
    // This function is called on a dedicated blocking worker; balance COM initialization.
    let initialized =
        unsafe { CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32) } >= 0;
    let result = if folder {
        let title = wide("MpvKit");
        let info = BROWSEINFOW {
            hwndOwner: owner as _,
            lpszTitle: title.as_ptr(),
            ulFlags: BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE,
            ..Default::default()
        };
        let item = unsafe { SHBrowseForFolderW(&info) };
        if item.is_null() {
            Ok(None)
        } else {
            let mut buffer = vec![0u16; 32768];
            let ok =
                unsafe { SHGetPathFromIDListEx(item, buffer.as_mut_ptr(), buffer.len() as u32, 0) };
            unsafe {
                CoTaskMemFree(item.cast());
            }
            if ok == 0 {
                Err(AppError::new("dialog_path", "无法读取所选目录。", "paths"))
            } else {
                Ok(Some(PathBuf::from(OsString::from_wide(
                    &buffer[..buffer.iter().position(|n| *n == 0).unwrap_or(buffer.len())],
                ))))
            }
        }
    } else {
        let mut buffer = vec![0u16; 32768];
        let mut info = OPENFILENAMEW {
            lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
            hwndOwner: owner as _,
            lpstrFile: buffer.as_mut_ptr(),
            nMaxFile: buffer.len() as u32,
            Flags: OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR,
            ..Default::default()
        };
        if unsafe { GetOpenFileNameW(&mut info) } != 0 {
            Ok(Some(PathBuf::from(text(&buffer))))
        } else if unsafe { CommDlgExtendedError() } == 0 {
            Ok(None)
        } else {
            Err(AppError::new(
                "dialog_failed",
                "无法打开文件选择器。",
                "paths",
            ))
        }
    };
    if initialized {
        unsafe {
            CoUninitialize();
        }
    }
    if let Ok(Some(path)) = &result {
        super::validate_local_path(path)?;
    }
    result
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
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            wide("open").as_ptr(),
            wide(&path.to_string_lossy()).as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };
    if result as isize <= 32 {
        return Err(AppError::new("open_failed", "无法打开目录。", "paths"));
    }
    Ok(())
}
