use super::{validate_local_path, windows::known_folder};
use crate::{
    error::{AppError, Result},
    mpv,
};
use serde::Serialize;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    time::{Duration, Instant},
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

pub struct CorePlan {
    pub preview: CorePreview,
    winget: PathBuf,
    player: PathBuf,
    stamp: Option<mpv::FileStamp>,
    created: Instant,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CorePreview {
    pub id: String,
    pub action: String,
    pub package: &'static str,
    pub source: &'static str,
    pub executable: PathBuf,
    pub config_directory: PathBuf,
    pub details: String,
}
fn args(action: &str) -> Vec<&str> {
    let mut args = vec![
        action,
        "--id",
        PACKAGE,
        "--exact",
        "--source",
        "winget",
        "--disable-interactivity",
        "--accept-source-agreements",
        "--silent",
    ];
    if action != "uninstall" {
        args.push("--accept-package-agreements");
    }
    args
}
fn run(
    winget: &Path,
    args: &[&str],
    timeout: Duration,
) -> Result<(std::process::ExitStatus, String)> {
    let mut command = super::command(winget);
    command.args(args);
    mpv::capture_status(command, timeout).map_err(|e| {
        if e.code == "probe_timeout" {
            error(
                "winget_timeout",
                "WinGet 超时。安装程序可能仍在运行，请检查系统安装状态后重试；配置未修改。",
            )
        } else {
            error(
                "winget_failed",
                "WinGet 无法运行，请检查应用安装程序、源和网络后重试。",
            )
        }
    })
}
pub fn prepare(action: &str) -> Result<CorePlan> {
    if !matches!(action, "install" | "upgrade" | "uninstall") {
        return Err(error("invalid_action", "不支持此核心操作。"));
    }
    let root = super::app_root()?;
    let player = player_path(&root)?;
    let stamp = if player.exists() {
        Some(mpv::file_stamp(&player)?)
    } else {
        None
    };
    if action == "install" && stamp.is_some() {
        return Err(error(
            "core_already_installed",
            "MPV 核心已安装，请检查更新。",
        ));
    }
    if action == "uninstall" && stamp.is_none() {
        return Err(error("not_installed", "MPV 核心未安装。"));
    }
    let winget = winget_path()?;
    let (status, details) = run(
        &winget,
        &[
            if action == "uninstall" {
                "list"
            } else {
                "show"
            },
            "--id",
            PACKAGE,
            "--exact",
            "--source",
            "winget",
            "--disable-interactivity",
            "--accept-source-agreements",
        ],
        Duration::from_secs(45),
    )?;
    if !status.success() {
        return Err(error(
            "winget_source",
            "WinGet 未能查询 shinchiro.mpv，请检查 winget 源和网络后重试。",
        ));
    }
    Ok(CorePlan {
        preview: CorePreview {
            id: crate::deploy::operation_id(),
            action: action.into(),
            package: PACKAGE,
            source: "winget",
            executable: player.clone(),
            config_directory: root,
            details,
        },
        winget,
        player,
        stamp,
        created: Instant::now(),
    })
}
pub fn check_update() -> Result<bool> {
    let player = player_path(&super::app_root()?)?;
    if !player.is_file() {
        return Err(error("not_installed", "MPV 核心未安装。"));
    }
    let (status, output) = run(
        &winget_path()?,
        &[
            "list",
            "--id",
            PACKAGE,
            "--exact",
            "--source",
            "winget",
            "--upgrade-available",
            "--disable-interactivity",
            "--accept-source-agreements",
        ],
        Duration::from_secs(45),
    )?;
    query_update(status.code().map(|code| code as u32), &output)
}
fn query_update(code: Option<u32>, output: &str) -> Result<bool> {
    match code {
        Some(0) => Ok(output
            .lines()
            .any(|line| line.split_whitespace().any(|field| field == PACKAGE))),
        Some(0x8A150014) => Ok(false), // Exact package has no matching upgrade row.
        _ => Err(error(
            "winget_source",
            "WinGet 未能检查 MPV 更新，请检查源和网络后重试。",
        )),
    }
}
fn check_exit(code: u32) -> Result<()> {
    match code {
        0 | 0x8A15002B => Ok(()), // APPINSTALLER_CLI_ERROR_UPDATE_NOT_APPLICABLE
        0x800704C7 | 0x8A150005 => {
            Err(error("winget_cancelled", "WinGet 安装已取消，配置未修改。"))
        }
        0x8A150019 => Err(error(
            "winget_elevation",
            "安装需要管理员授权，请允许系统授权提示后重试。",
        )),
        _ => Err(error(
            "winget_failed",
            "WinGet 安装或更新失败，请检查系统授权、源和网络后重试；已安装的核心会保留。",
        )),
    }
}
pub fn execute(plan: CorePlan) -> Result<Option<mpv::MpvInfo>> {
    let _lock = crate::deploy::acquire(&super::default_root()?)?;
    let root = super::app_root()?;
    let player = player_path(&root)?;
    let stamp = if player.exists() {
        Some(mpv::file_stamp(&player)?)
    } else {
        None
    };
    if plan.created.elapsed() > Duration::from_secs(300)
        || root != plan.preview.config_directory
        || player != plan.player
        || stamp != plan.stamp
        || winget_path()? != plan.winget
    {
        return Err(error("stale_preview", "核心或配置位置已改变，请重新预览。"));
    }
    let (status, _) = run(
        &plan.winget,
        &args(&plan.preview.action),
        Duration::from_secs(900),
    )?;
    if plan.preview.action == "uninstall" && !status.success() {
        return Err(error(
            "winget_failed",
            "WinGet 卸载未完成，请检查系统授权或包状态；配置未清除。",
        ));
    }
    check_exit(status.code().unwrap_or(-1) as u32)?;
    let root = super::app_root()?;
    let player = player_path(&root)?;
    if plan.preview.action == "uninstall" {
        if player.exists() {
            return Err(error(
                "mpv_uninstall_incomplete",
                "WinGet 已结束，但 MPV 核心仍存在，请检查包状态。",
            ));
        }
        return Ok(None);
    }
    let info = mpv::inspect(&player).map_err(|_| {
        error(
            "mpv_install_incomplete",
            "WinGet 已结束，但未检测到可用 MPV；请检查系统安装状态后重试。",
        )
    })?;
    mpv::ensure_managed_options(&player)?;
    Ok(Some(info))
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
    fn exact_package_arguments_and_exit_codes() {
        assert!(query_update(Some(0), "mpv shinchiro.mpv 0.40 0.41 winget").unwrap());
        assert!(!query_update(Some(0), "No installed package found.").unwrap());
        assert!(!query_update(Some(0x8A150014), "").unwrap());
        assert!(query_update(Some(1), "").is_err());
        assert_eq!(
            args("install")[..7],
            [
                "install",
                "--id",
                PACKAGE,
                "--exact",
                "--source",
                "winget",
                "--disable-interactivity"
            ]
        );
        assert!(!args("upgrade").contains(&"--include-unknown"));
        assert!(!args("uninstall").contains(&"--accept-package-agreements"));
        assert!(args("uninstall").contains(&"--silent"));
        assert!(!args("uninstall").contains(&"--purge"));
        for code in [0, 0x8A15002B] {
            check_exit(code).unwrap();
        }
        for code in [1, 0x800704C7, 0x8A150005, 0x8A150019] {
            assert!(check_exit(code).is_err());
        }
        assert!(select_player(vec![PathBuf::from("relative/mpv.exe")], PathBuf::new()).is_err());
    }
}
