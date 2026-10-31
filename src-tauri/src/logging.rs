//! Small, local task logs. Only allowlisted identifiers are accepted, never raw errors or output.
use crate::{deploy, platform};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
const FILE_LIMIT: u64 = 1024 * 1024;
const RETENTION: Duration = Duration::from_secs(7 * 86400);
static ROOT: OnceLock<PathBuf> = OnceLock::new();
static DISABLED: AtomicBool = AtomicBool::new(false);
static WRITER: Mutex<()> = Mutex::new(());
static LAST_PROBLEM: Mutex<Option<(String, String)>> = Mutex::new(None);

pub fn initialize() {
    if let Ok(root) = platform::default_root() {
        let _ = ROOT.set(root.join("logs"));
        std::thread::spawn(|| {
            record(
                "application",
                "startup",
                "session",
                "started",
                None,
                None,
                0,
            )
        });
    }
}
pub fn directory() -> crate::error::Result<PathBuf> {
    let root = ROOT.get().cloned().ok_or_else(|| {
        crate::error::AppError::new("log_unavailable", "日志目录不可用。", "logs")
    })?;
    prepare(&root).map_err(|_| {
        crate::error::AppError::new("log_unavailable", "无法创建日志目录。", "logs")
    })?;
    Ok(root)
}
pub fn available() -> bool {
    !DISABLED.load(Ordering::Relaxed)
}
fn identifier(text: &str) -> String {
    if text.len() <= 80
        && !text.is_empty()
        && text
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_,".contains(&b))
    {
        text.into()
    } else {
        "redacted".into()
    }
}
fn prepare(root: &Path) -> io::Result<()> {
    platform::validate_local_path(root).map_err(|_| io::Error::other("unsafe log directory"))?;
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(root)
}
fn file(root: &Path, number: usize) -> PathBuf {
    root.join(if number == 0 {
        "mpvkit.log".into()
    } else {
        format!("mpvkit.{number}.log")
    })
}
fn checked(path: &Path) -> io::Result<()> {
    platform::validate_local_path(path).map_err(|_| io::Error::other("unsafe log file"))
}
fn append(root: &Path, record: &[u8], now: SystemTime) -> io::Result<()> {
    prepare(root)?;
    let _lock = deploy::lock_target(root).map_err(|e| {
        if e.code == "target_locked" {
            io::Error::from(io::ErrorKind::WouldBlock)
        } else {
            io::Error::other("log unavailable")
        }
    })?;
    for n in 0..4 {
        let path = file(root, n);
        checked(&path)?;
        if let Ok(m) = fs::metadata(&path) {
            if !m.is_file() {
                return Err(io::Error::other("invalid log file"));
            }
            if now.duration_since(m.modified()?).unwrap_or_default() > RETENTION
                || m.len() > FILE_LIMIT
            {
                fs::remove_file(path)?;
            }
        }
    }
    let current = file(root, 0);
    if fs::metadata(&current).map(|m| m.len()).unwrap_or(0) + record.len() as u64 > FILE_LIMIT {
        if file(root, 3).exists() {
            fs::remove_file(file(root, 3))?;
        }
        for n in (0..3).rev() {
            let source = file(root, n);
            if source.exists() {
                fs::rename(source, file(root, n + 1))?;
            }
        }
    }
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    options.open(current)?.write_all(record)
}
pub fn record(
    task: &str,
    operation: &str,
    phase: &str,
    result: &str,
    code: Option<&str>,
    exit_code: Option<i32>,
    elapsed_ms: u64,
) {
    if !available() {
        return;
    }
    let Some(root) = ROOT.get() else {
        return;
    };
    let Ok(_guard) = WRITER.lock() else {
        return;
    };
    if !available() {
        return;
    }
    let now = SystemTime::now();
    let value = serde_json::json!({"time":now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(), "version":env!("CARGO_PKG_VERSION"), "os":std::env::consts::OS, "arch":std::env::consts::ARCH, "task":identifier(task), "operation":identifier(operation), "phase":identifier(phase), "result":identifier(result), "code":code.map(identifier), "exitCode":exit_code, "elapsedMs":elapsed_ms});
    let Ok(mut bytes) = serde_json::to_vec(&value) else {
        return;
    };
    bytes.truncate(8191);
    bytes.push(b'\n');
    if let Err(error) = append(root, &bytes, now)
        && error.kind() != io::ErrorKind::WouldBlock
    {
        DISABLED.store(true, Ordering::Relaxed);
    }
}
pub fn problem(code: &str, phase: &str) {
    let problem = (identifier(code), identifier(phase));
    if let Ok(mut previous) = LAST_PROBLEM.lock() {
        if previous.as_ref() == Some(&problem) {
            return;
        }
        *previous = Some(problem.clone());
    }
    record(
        "application",
        "check",
        &problem.1,
        "failed",
        Some(&problem.0),
        None,
        0,
    );
}
pub fn recovered() {
    if let Ok(mut previous) = LAST_PROBLEM.lock()
        && let Some((_, phase)) = previous.take()
    {
        record("application", "check", &phase, "recovered", None, None, 0);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_rotation_retention_and_private_fields() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("logs");
        let block = vec![b'x'; 8192];
        for _ in 0..600 {
            append(&root, &block, SystemTime::now()).unwrap();
        }
        let total: u64 = (0..4)
            .map(|n| fs::metadata(file(&root, n)).unwrap().len())
            .sum();
        assert!(total <= 4 * FILE_LIMIT);
        fs::write(root.join("user.txt"), "keep").unwrap();
        append(
            &root,
            b"new\n",
            SystemTime::now() + RETENTION + Duration::from_secs(5),
        )
        .unwrap();
        assert_eq!(fs::read(file(&root, 0)).unwrap(), b"new\n");
        assert!(root.join("user.txt").exists());
        for raw in [
            "/home/person/movie.mkv",
            "https://user:secret@host",
            "script error\npassword",
        ] {
            assert_eq!(identifier(raw), "redacted");
        }
        assert_eq!(identifier("package_failed"), "package_failed");
    }
    #[test]
    fn contention_and_invalid_storage_leave_other_files_intact() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("logs");
        prepare(&root).unwrap();
        let lock = deploy::lock_target(&root).unwrap();
        assert_eq!(
            append(&root, b"entry\n", SystemTime::now())
                .unwrap_err()
                .kind(),
            io::ErrorKind::WouldBlock
        );
        drop(lock);
        append(&root, b"entry\n", SystemTime::now()).unwrap();
        fs::remove_file(file(&root, 0)).unwrap();
        fs::create_dir(file(&root, 0)).unwrap();
        fs::write(root.join("private"), "keep").unwrap();
        assert!(append(&root, b"entry\n", SystemTime::now()).is_err());
        assert_eq!(fs::read_to_string(root.join("private")).unwrap(), "keep");
        fs::remove_dir(file(&root, 0)).unwrap();
        let key = root.to_string_lossy();
        #[cfg(target_os = "windows")]
        let key = key.to_lowercase();
        let lock_path = dir.path().join(format!(
            ".mpvkit-{}.lock",
            deploy::hash_bytes(key.as_bytes())
        ));
        fs::remove_file(&lock_path).unwrap();
        fs::create_dir(lock_path).unwrap();
        assert_ne!(
            append(&root, b"entry\n", SystemTime::now())
                .unwrap_err()
                .kind(),
            io::ErrorKind::WouldBlock
        );
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn log_cleanup_never_follows_links() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("logs");
        fs::create_dir(&root).unwrap();
        let target = dir.path().join("private");
        fs::write(&target, "keep").unwrap();
        std::os::unix::fs::symlink(&target, file(&root, 0)).unwrap();
        assert!(append(&root, b"failed", SystemTime::now()).is_err());
        assert_eq!(fs::read_to_string(target).unwrap(), "keep");
    }
}
