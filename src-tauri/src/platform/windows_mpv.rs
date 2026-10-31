use super::{validate_local_path, windows::known_folder};
use crate::error::{AppError, Result};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};
use windows_sys::Win32::System::Registry::*;
use windows_sys::Win32::UI::Shell::FOLDERID_LocalAppData;

pub const PACKAGE: &str = "shinchiro.mpv";
fn error(code: &'static str, message: &'static str) -> AppError {
    AppError::new(code, message, "system-mpv")
}

// Only the selected package's exact uninstall identity is read; never scan drives.
fn registered_locations() -> Result<Vec<PathBuf>> {
    let key: Vec<u16> =
        r"Software\Microsoft\Windows\CurrentVersion\Uninstall\MPV_Player_Automated_is1"
            .encode_utf16()
            .chain(Some(0))
            .collect();
    let name: Vec<u16> = "InstallLocation".encode_utf16().chain(Some(0)).collect();
    let mut paths = Vec::new();
    for hive in [HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER] {
        for view in [RRF_SUBKEY_WOW6464KEY, RRF_SUBKEY_WOW6432KEY] {
            let mut buffer = vec![0u16; 32768];
            let mut length = (buffer.len() * 2) as u32;
            let status = unsafe {
                RegGetValueW(
                    hive,
                    key.as_ptr(),
                    name.as_ptr(),
                    RRF_RT_REG_SZ | view,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr().cast(),
                    &mut length,
                )
            };
            match status {
                0 => {
                    let text = String::from_utf16(&buffer[..length as usize / 2])
                        .map_err(|_| error("mpv_registry", "MPV 安装路径不是有效文本。"))?;
                    let text = text.trim_end_matches('\0');
                    if !text.is_empty() {
                        paths.push(PathBuf::from(text).join("mpv.exe"));
                    }
                }
                2 | 3 => {}
                _ => return Err(error("mpv_registry", "无法读取 MPV 安装位置，请检查权限。")),
            }
        }
    }
    Ok(paths)
}

fn select_player(paths: Vec<PathBuf>, fallback: PathBuf) -> Result<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    for path in paths {
        validate_local_path(&path)?;
        if path.is_file()
            && !found.iter().any(|p| {
                p.components()
                    .map(|c| c.as_os_str().to_ascii_lowercase())
                    .eq(path
                        .components()
                        .map(|c| c.as_os_str().to_ascii_lowercase()))
            })
        {
            found.push(path);
        }
    }
    if found.len() > 1 {
        return Err(AppError::new(
            "mpv_ambiguous",
            format!(
                "发现多个 MPV 安装位置，请核对后重试；文件未修改。\n{}",
                found
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            "system-mpv",
        ));
    }
    Ok(found.pop().unwrap_or(fallback))
}

pub fn player_path(root: &Path) -> Result<PathBuf> {
    #[cfg(test)]
    return Ok(root.join("mpv.exe"));
    #[cfg(not(test))]
    {
        let _ = root;
        #[cfg(debug_assertions)]
        if std::env::var_os("MPVKIT_TEST_MODE").is_some_and(|v| v == "1") {
            return Ok(super::default_root()?.join("system-mpv/mpv.exe"));
        }
        let fallback = known_folder(&windows_sys::Win32::UI::Shell::FOLDERID_ProgramFiles)?
            .join("MPV Player/mpv.exe");
        let mut paths = registered_locations()?;
        paths.push(fallback.clone());
        select_player(paths, fallback)
    }
}

pub struct ConfigLocation {
    pub directory: PathBuf,
    pub source: &'static str,
}
fn resolve_config(
    player: &Path,
    roaming: &Path,
    mpv_home: Option<OsString>,
) -> Result<ConfigLocation> {
    let parent = player
        .parent()
        .ok_or_else(|| error("mpv_path", "MPV 安装位置无效。"))?;
    let portable = parent.join("portable_config");
    let (directory, source) = if let Some(home) = mpv_home.filter(|v| !v.is_empty()) {
        (PathBuf::from(home), "MPV_HOME")
    } else if portable.exists() {
        (portable, "portable_config")
    } else {
        // MPV also reads exe-dir and exe-dir/mpv. Do not silently combine those
        // configurations with the user directory we are about to manage.
        for dir in [parent.to_path_buf(), parent.join("mpv")] {
            validate_local_path(&dir)?;
            if ["mpv.conf", "input.conf", "scripts", "script-opts", "fonts"]
                .iter()
                .any(|p| dir.join(p).exists())
            {
                return Err(error(
                    "mpv_config_ambiguous",
                    "MPV 安装目录含额外配置，请整理或设置绝对 MPV_HOME 后重试；文件未修改。",
                ));
            }
        }
        (roaming.join("mpv"), "APPDATA")
    };
    validate_local_path(&directory)?;
    if directory.parent().is_none() || directory.exists() && !directory.is_dir() {
        return Err(error(
            "mpv_config_directory",
            "MPV 配置位置不是可管理的目录。",
        ));
    }
    Ok(ConfigLocation { directory, source })
}
pub fn config_location() -> Result<ConfigLocation> {
    #[cfg(test)]
    return Ok(ConfigLocation {
        directory: std::env::temp_dir().join("mpvkit-unrouted-unit"),
        source: "isolated-unit",
    });
    #[cfg(not(test))]
    {
        #[cfg(debug_assertions)]
        if std::env::var_os("MPVKIT_TEST_MODE").is_some_and(|v| v == "1") {
            return Ok(ConfigLocation {
                directory: super::default_root()?.join("config").join("mpv"),
                source: "isolated-test",
            });
        }
        resolve_config(
            &player_path(Path::new(""))?,
            &known_folder(&windows_sys::Win32::UI::Shell::FOLDERID_RoamingAppData)?,
            std::env::var_os("MPV_HOME"),
        )
    }
}

fn winget_path() -> Result<PathBuf> {
    #[cfg(debug_assertions)]
    if std::env::var_os("MPVKIT_TEST_MODE").is_some_and(|v| v == "1") {
        // Test mode must never reach the real package manager or registry.
        let path = super::default_root()?.join("test-tools/winget.exe");
        validate_local_path(&path)?;
        return path.is_file().then_some(path).ok_or_else(|| {
            error(
                "winget_missing",
                "未找到 WinGet，请安装或更新 Microsoft 应用安装程序后重试。",
            )
        });
    }
    let path = known_folder(&FOLDERID_LocalAppData)?.join("Microsoft/WindowsApps/winget.exe");
    // App Execution Aliases are reparse points; only this OS-known alias may
    // bypass the ordinary file reparse prohibition. It is never a payload path.
    if !path.exists() {
        return Err(error(
            "winget_missing",
            "未找到 WinGet，请安装或更新 Microsoft 应用安装程序后重试。",
        ));
    }
    Ok(path)
}

fn args(action: &str) -> Result<Vec<&str>> {
    if !matches!(action, "install" | "upgrade" | "uninstall") {
        return Err(error("invalid_action", "不支持此核心操作。"));
    }
    Ok(vec![
        action,
        "--id",
        PACKAGE,
        "--exact",
        "--source",
        "winget",
        "--interactive",
    ])
}

pub fn check(action: &str) -> Result<()> {
    args(action)?;
    winget_path()?;
    Ok(())
}
pub fn terminal(executable: &Path, action: &str, id: &str) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    let mut command = std::process::Command::new(executable);
    command
        .args(["--core-command", action, id])
        .creation_flags(0x0000_0010);
    command
}
pub fn launch(action: &str) -> Result<crate::native_core::Outcome> {
    // The helper owns this console; WinGet keeps its prompts and a real console input handle.
    unsafe {
        windows_sys::Win32::System::Console::AllocConsole();
    }
    let input =
        std::fs::File::open("CONIN$").map_err(|_| error("winget_failed", "无法打开终端输入。"))?;
    crate::native_core::wait_command(
        std::process::Command::new(winget_path()?)
            .args(args(action)?)
            .stdin(input),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "read-only check of the explicitly selected Windows host registry and MPV file"]
    fn local_registered_mpv_paths_are_unambiguous() {
        let fallback = known_folder(&windows_sys::Win32::UI::Shell::FOLDERID_ProgramFiles)
            .unwrap()
            .join("MPV Player/mpv.exe");
        let mut paths = registered_locations().unwrap();
        paths.push(fallback.clone());
        let installed = paths.iter().any(|path| path.is_file());
        let selected = select_player(paths, fallback.clone()).unwrap();
        eprintln!(
            "Registered MPV installed: {installed}; selected: {}",
            selected.display()
        );
        if installed {
            assert!(selected.is_file());
        } else {
            assert_eq!(selected, fallback);
        }
    }
    #[test]
    fn configuration_precedence_and_ambiguity() {
        let base = std::env::temp_dir().join(format!(
            "mpvkit-win-paths-{}",
            crate::deploy::operation_id()
        ));
        let player = base.join("bin/mpv.exe");
        let roaming = base.join("roaming");
        std::fs::create_dir_all(player.parent().unwrap()).unwrap();
        assert_eq!(
            resolve_config(&player, &roaming, None).unwrap().directory,
            roaming.join("mpv")
        );
        std::fs::write(player.parent().unwrap().join("mpv.conf"), "volume=1").unwrap();
        assert!(resolve_config(&player, &roaming, None).is_err());
        let home = base.join("home");
        assert_eq!(
            resolve_config(&player, &roaming, Some(home.clone().into()))
                .unwrap()
                .directory,
            home
        );
        std::fs::create_dir(player.parent().unwrap().join("portable_config")).unwrap();
        assert_eq!(
            resolve_config(&player, &roaming, None).unwrap().source,
            "portable_config"
        );
        assert!(resolve_config(&player, &roaming, Some("relative".into())).is_err());
        std::fs::write(&player, b"first").unwrap();
        let other = base.join("second.exe");
        std::fs::write(&other, b"second").unwrap();
        assert_eq!(
            select_player(vec![player.clone(), player.clone()], other.clone()).unwrap(),
            player
        );
        let mixed = PathBuf::from(player.to_string_lossy().replace('\\', "/"));
        assert_eq!(
            select_player(vec![player.clone(), mixed], other.clone()).unwrap(),
            player
        );
        let upper = PathBuf::from(player.to_string_lossy().to_ascii_uppercase());
        assert_eq!(
            select_player(vec![player.clone(), upper], other.clone()).unwrap(),
            player
        );
        let ambiguity =
            select_player(vec![player.clone(), other.clone()], player.clone()).unwrap_err();
        assert_eq!(ambiguity.code, "mpv_ambiguous");
        assert!(ambiguity.message.contains(&player.display().to_string()));
        assert!(ambiguity.message.contains(&other.display().to_string()));
        std::fs::remove_dir_all(base).unwrap();
    }
    #[test]
    fn exact_package_arguments_preserve_native_interaction() {
        for action in ["install", "upgrade", "uninstall"] {
            assert_eq!(
                args(action).unwrap(),
                [
                    action,
                    "--id",
                    PACKAGE,
                    "--exact",
                    "--source",
                    "winget",
                    "--interactive"
                ]
            );
        }
        assert!(args("uninstall --purge").is_err());
        assert!(select_player(vec![PathBuf::from("relative/mpv.exe")], PathBuf::new()).is_err());
    }
}
