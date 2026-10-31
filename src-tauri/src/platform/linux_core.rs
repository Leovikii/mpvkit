//! Pacman owns all prompts and dependency decisions; the caller supplies a real terminal.
use crate::{
    error::{AppError, Result},
    native_core::Outcome,
};
use std::{path::Path, process::Command};
fn error(message: &'static str) -> AppError {
    AppError::new("package_failed", message, "system-mpv")
}
fn arguments(action: &str) -> Result<Vec<&'static str>> {
    match action {
        "install" | "upgrade" => Ok(vec!["-S", "--needed", "--", "mpv"]),
        "uninstall" => Ok(vec!["-R", "--", "mpv"]),
        _ => Err(error("不支持此核心操作。")),
    }
}
pub fn check(action: &str) -> Result<()> {
    arguments(action)?;
    if cfg!(debug_assertions) && std::env::var_os("MPVKIT_TEST_MODE").is_some_and(|v| v == "1") {
        return Err(error("隔离模式禁止调用真实包管理器。"));
    }
    let os = std::fs::read_to_string("/etc/os-release").map_err(|_| error("无法识别系统。"))?;
    if !os
        .lines()
        .filter_map(|s| s.split_once('='))
        .any(|(k, v)| k == "ID" && matches!(v.trim_matches('"'), "arch" | "cachyos"))
    {
        return Err(error("仅支持 Arch/CachyOS。"));
    }
    for tool in ["/usr/bin/konsole", "/usr/bin/pkexec", "/usr/bin/pacman"] {
        if !Path::new(tool).is_file() {
            return Err(error("需要 Konsole、pkexec 和 Pacman。"));
        }
    }
    Ok(())
}
pub fn terminal(executable: &Path, action: &str, id: &str) -> Command {
    let mut command = Command::new("/usr/bin/konsole");
    command
        .args(["--separate", "--nofork", "-e"])
        .arg(executable)
        .args(["--core-command", action, id]);
    command
}
pub fn launch(action: &str) -> Result<Outcome> {
    check(action)?;
    crate::native_core::wait_command(
        Command::new("/usr/bin/pkexec")
            .arg("/usr/bin/pacman")
            .args(arguments(action)?),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_scope_retains_native_prompts() {
        assert_eq!(
            arguments("install").unwrap(),
            ["-S", "--needed", "--", "mpv"]
        );
        assert_eq!(arguments("upgrade").unwrap(), arguments("install").unwrap());
        assert_eq!(arguments("uninstall").unwrap(), ["-R", "--", "mpv"]);
        assert!(arguments("uninstall --noconfirm").is_err());
        let command = terminal(Path::new("/tmp/test app"), "install", "abc-1");
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            [
                "--separate",
                "--nofork",
                "-e",
                "/tmp/test app",
                "--core-command",
                "install",
                "abc-1"
            ]
        );
    }
}
