//! Fixed Arch package operations. pkexec owns authentication; no shell or sudo input.
use crate::{
    deploy,
    error::{AppError, Result},
    mpv,
};
use serde::Serialize;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

fn error(message: &'static str) -> AppError {
    AppError::new("package_failed", message, "system-mpv")
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
pub struct CorePlan {
    pub preview: CorePreview,
    stamp: Option<mpv::FileStamp>,
    created: Instant,
}
fn stamp() -> Result<Option<mpv::FileStamp>> {
    let path = std::path::Path::new("/usr/bin/mpv");
    if path.exists() {
        mpv::file_stamp(path).map(Some)
    } else {
        Ok(None)
    }
}
pub fn prepare(action: &str) -> Result<CorePlan> {
    if !matches!(action, "install" | "upgrade" | "uninstall") {
        return Err(error("不支持此核心操作。"));
    }
    let os =
        std::fs::read_to_string("/etc/os-release").map_err(|_| error("无法识别系统发行版。"))?;
    if !supported_os(&os) {
        return Err(error("当前仅支持 Arch/CachyOS 的 pacman 核心操作。"));
    }
    for path in ["/usr/bin/pacman", "/usr/bin/pkexec"] {
        if !std::path::Path::new(path).is_file() {
            return Err(error("未找到 pacman 或 pkexec，请使用系统包管理器处理。"));
        }
    }
    let state = stamp()?;
    if action == "uninstall" && state.is_none() {
        return Err(error("MPV 核心未安装。"));
    }
    Ok(CorePlan { preview: CorePreview {
        id: deploy::operation_id(), action: action.into(), package: "mpv", source: "pacman",
        executable: "/usr/bin/mpv".into(), config_directory: super::app_root()?,
        details: "通过系统授权执行固定 mpv 包操作；不绕过依赖检查，不移除依赖或用户配置。安装/更新使用当前同步数据库，不单独刷新数据库。".into(),
    }, stamp: state, created: Instant::now() })
}
pub fn execute(plan: CorePlan) -> Result<Option<mpv::MpvInfo>> {
    // Debug test mode never reaches the real package manager.
    if cfg!(debug_assertions) && std::env::var_os("MPVKIT_TEST_MODE").is_some_and(|v| v == "1") {
        return Err(error("Linux 隔离模式禁止调用真实包管理器。"));
    }
    let _lock = deploy::acquire(&super::default_root()?)?;
    if plan.created.elapsed() > Duration::from_secs(300)
        || plan.stamp != stamp()?
        || plan.preview.config_directory != super::app_root()?
    {
        return Err(AppError::new(
            "stale_preview",
            "核心或配置位置已变化，请重新预览。",
            "system-mpv",
        ));
    }
    let mut command = super::command(std::path::Path::new("/usr/bin/pkexec"));
    command.args(package_args(&plan.preview.action));
    let (status, _) = mpv::capture_status(command, Duration::from_secs(900))?;
    if !status.success() {
        return Err(error(
            "包管理器操作未完成；可能取消授权或存在依赖/网络冲突，请检查系统状态。",
        ));
    }
    if plan.preview.action == "uninstall" {
        if stamp()?.is_some() {
            return Err(error("包管理器已结束，但 MPV 核心仍存在。"));
        }
        return Ok(None);
    }
    let info = mpv::inspect(std::path::Path::new("/usr/bin/mpv"))?;
    mpv::ensure_managed_options(std::path::Path::new("/usr/bin/mpv"))?;
    Ok(Some(info))
}

fn supported_os(os: &str) -> bool {
    os.lines()
        .filter_map(|line| line.trim().split_once('='))
        .any(|(key, value)| {
            key == "ID" && matches!(value.trim().trim_matches('"'), "arch" | "cachyos")
        })
}
fn package_args(action: &str) -> [&'static str; 5] {
    [
        "--disable-internal-agent",
        "/usr/bin/pacman",
        if action == "uninstall" { "-R" } else { "-S" },
        "--noconfirm",
        "mpv",
    ]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distro_and_native_arguments_keep_dependency_checks() {
        for text in ["ID=arch\n", "NAME=Other\nID=\"cachyos\"\n"] {
            assert!(supported_os(text));
        }
        for text in ["ID_LIKE=arch\nID=manjaro", "ID=debian", "ID=archlinux"] {
            assert!(!supported_os(text));
        }
        assert_eq!(
            package_args("uninstall"),
            [
                "--disable-internal-agent",
                "/usr/bin/pacman",
                "-R",
                "--noconfirm",
                "mpv"
            ]
        );
        assert_eq!(package_args("upgrade")[2], "-S");
        assert_eq!(package_args("install")[2], "-S");
        // Compiling on Windows checks shared types and both implementations without executing pacman.
        let _prepare: fn(&str) -> Result<CorePlan> = prepare;
        let _execute: fn(CorePlan) -> Result<Option<mpv::MpvInfo>> = execute;
    }
}
