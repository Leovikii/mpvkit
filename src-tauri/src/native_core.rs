use crate::{
    error::{AppError, Result},
    platform,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Outcome {
    pub exit_code: Option<i32>,
    pub output: String,
}
impl Outcome {
    pub fn success(&self) -> bool {
        self.exit_code == Some(0)
    }
}

// Keep terminal interaction native while retaining bounded output for the task dialog.
pub fn wait_command(command: &mut std::process::Command) -> Result<Outcome> {
    use std::{
        io::{Read, Write},
        process::Stdio,
    };
    fn relay(mut input: impl Read, mut output: impl Write) -> String {
        let mut tail = Vec::new();
        let mut buffer = [0; 4096];
        while let Ok(size) = input.read(&mut buffer) {
            if size == 0 {
                break;
            }
            let _ = output.write_all(&buffer[..size]);
            let _ = output.flush();
            tail.extend_from_slice(&buffer[..size]);
            if tail.len() > 8192 {
                tail.drain(..tail.len() - 8192);
            }
        }
        String::from_utf8_lossy(&tail).into_owned()
    }
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| AppError::new("package_failed", "无法启动包管理器。", "system-mpv"))?;
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    #[cfg(target_os = "linux")]
    let (out, err) = (
        std::thread::spawn(move || relay(stdout, std::io::stdout())),
        std::thread::spawn(move || relay(stderr, std::io::stderr())),
    );
    #[cfg(target_os = "windows")]
    let (out, err) = (
        std::thread::spawn(move || {
            relay(
                stdout,
                std::fs::OpenOptions::new()
                    .write(true)
                    .open("CONOUT$")
                    .map(|f| Box::new(f) as Box<dyn Write>)
                    .unwrap_or_else(|_| Box::new(std::io::sink())),
            )
        }),
        std::thread::spawn(move || {
            relay(
                stderr,
                std::fs::OpenOptions::new()
                    .write(true)
                    .open("CONOUT$")
                    .map(|f| Box::new(f) as Box<dyn Write>)
                    .unwrap_or_else(|_| Box::new(std::io::sink())),
            )
        }),
    );
    let status = child
        .wait()
        .map_err(|_| AppError::new("package_failed", "无法等待包管理器结果。", "system-mpv"))?;
    Ok(Outcome {
        exit_code: status.code(),
        output: format!(
            "{}{}",
            out.join().unwrap_or_default(),
            err.join().unwrap_or_default()
        ),
    })
}

fn task_directory(id: &str) -> Result<std::path::PathBuf> {
    if id.is_empty() || id.len() > 80 || !id.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-') {
        return Err(AppError::new(
            "invalid_action",
            "核心任务无效。",
            "system-mpv",
        ));
    }
    let path = platform::default_root()?.join("core-tasks").join(id);
    platform::validate_local_path(&path)?;
    Ok(path)
}

pub fn run(action: &str) -> Result<Outcome> {
    use std::fs;
    #[cfg(target_os = "linux")]
    use std::os::unix::fs::DirBuilderExt;
    #[cfg(target_os = "linux")]
    if cfg!(debug_assertions) && std::env::var_os("MPVKIT_TEST_MODE").is_some_and(|v| v == "1") {
        return Err(AppError::new(
            "package_test_blocked",
            "隔离模式不运行系统包管理器。",
            "system-mpv",
        ));
    }
    platform::core::check(action)?;
    let id = crate::deploy::operation_id();
    let directory = task_directory(&id)?;
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(target_os = "linux")]
    builder.mode(0o700);
    builder
        .create(&directory)
        .map_err(|_| AppError::new("package_failed", "无法准备包管理器任务。", "system-mpv"))?;
    let result = (|| {
        crate::deploy::write_new(&directory.join("request"), action.as_bytes())?;
        let executable = std::env::current_exe()
            .map_err(|_| AppError::new("package_failed", "无法找到管理器。", "system-mpv"))?;
        let mut terminal = platform::core::terminal(&executable, action, &id);
        // Only the helper's package-manager result counts, never the terminal's exit status.
        terminal
            .status()
            .map_err(|_| AppError::new("package_failed", "无法打开终端。", "system-mpv"))?;
        let bytes = crate::deploy::read_bytes(&directory.join("result"))?;
        if bytes.len() > 128 * 1024 {
            return Err(AppError::new(
                "package_failed",
                "包管理器结果无效。",
                "system-mpv",
            ));
        }
        serde_json::from_slice(&bytes)
            .map_err(|_| AppError::new("package_failed", "未收到包管理器结果。", "system-mpv"))
    })();
    for name in ["request", "running", "result"] {
        let _ = fs::remove_file(directory.join(name));
    }
    let _ = fs::remove_dir(directory);
    result
}

pub fn helper(args: &[String]) -> i32 {
    let result = (|| -> Result<Outcome> {
        if args.len() != 4 {
            return Err(AppError::new(
                "invalid_action",
                "核心任务参数无效。",
                "system-mpv",
            ));
        }
        let directory = task_directory(&args[3])?;
        if crate::deploy::read_bytes(&directory.join("request"))? != args[2].as_bytes() {
            return Err(AppError::new(
                "invalid_action",
                "核心任务不匹配。",
                "system-mpv",
            ));
        }
        crate::deploy::write_new(&directory.join("running"), b"running")?;
        let outcome = platform::core::launch(&args[2]).unwrap_or_else(|e| Outcome {
            exit_code: None,
            output: e.message,
        });
        crate::deploy::write_new(
            &directory.join("result"),
            &serde_json::to_vec(&outcome).expect("outcome JSON"),
        )?;
        Ok(outcome)
    })();
    match result {
        Ok(r) => {
            if r.success() {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("{}", e.message);
            1
        }
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    #[test]
    fn waits_for_actual_command_result_and_output() {
        let result =
            wait_command(std::process::Command::new("/usr/bin/printf").arg("native result"))
                .unwrap();
        assert!(result.success());
        assert_eq!(result.output, "native result");
        let result = wait_command(&mut std::process::Command::new("/usr/bin/false")).unwrap();
        assert_eq!(result.exit_code, Some(1));
        let flood =
            wait_command(std::process::Command::new("/usr/bin/printf").args(["%20000s", "tail"]))
                .unwrap();
        assert_eq!(flood.output.len(), 8192);
        assert!(flood.output.ends_with("tail"));
        assert!(task_directory("../escape").is_err());
    }
}
