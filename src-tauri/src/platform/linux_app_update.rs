//! Native Arch application updates. The GUI and downloads never run as root.
use crate::{
    error::{AppError, Result},
    mpv,
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    process::Stdio,
    time::{Duration, Instant},
};

fn error(code: &'static str) -> AppError {
    AppError::new(
        code,
        "应用更新未完成，请检查系统授权、包数据库和依赖后重试。",
        "update",
    )
}
pub struct Plan {
    _directory: tempfile::TempDir,
    file: PathBuf,
    digest: String,
    version: String,
    installed: String,
    transaction: String,
    pub changes: Vec<String>,
    created: Instant,
}
fn query(args: &[&str]) -> Result<String> {
    let mut command = super::command(Path::new("/usr/bin/pacman"));
    command.env("LC_ALL", "C").args(args);
    let (status, output, errors) = mpv::capture_output(command, Duration::from_secs(30))?;
    if !status.success() || !errors.trim().is_empty() {
        return Err(error("update_package_query"));
    }
    Ok(output.trim().into())
}
fn installed() -> Result<String> {
    if cfg!(debug_assertions) && std::env::var_os("MPVKIT_TEST_MODE").is_some_and(|v| v == "1") {
        return Err(error("update_test_install_blocked"));
    }
    let os =
        fs::read_to_string("/etc/os-release").map_err(|_| error("update_package_unsupported"))?;
    if !os
        .lines()
        .filter_map(|l| l.split_once('='))
        .any(|(k, v)| k == "ID" && matches!(v.trim_matches('"'), "arch" | "cachyos"))
    {
        return Err(error("update_package_unsupported"));
    }
    if std::env::current_exe().ok().as_deref() != Some(Path::new("/usr/bin/mpvkit"))
        || query(&["-Qoq", "--", "/usr/bin/mpvkit"])? != "mpvkit-bin"
    {
        return Err(error("update_package_unmanaged"));
    }
    for tool in ["/usr/bin/pacman", "/usr/bin/pkexec"] {
        if !Path::new(tool).is_file() {
            return Err(error("update_package_unsupported"));
        }
    }
    let version = query(&["-Q", "--", "mpvkit-bin"])?;
    if version != format!("mpvkit-bin {}", package_version(env!("CARGO_PKG_VERSION"))) {
        return Err(error("stale_preview"));
    }
    Ok(version)
}
fn transaction(file: &str, version: &str) -> Result<(String, Vec<String>)> {
    let output = query(&[
        "-Up",
        "--noconfirm",
        "--print-format",
        "%n\t%v\t%a\t%l",
        "--",
        file,
    ])?;
    let changes = parse_transaction(&output, version)?;
    Ok((output, changes))
}
fn parse_transaction(output: &str, version: &str) -> Result<Vec<String>> {
    let mut target = false;
    let mut changes = Vec::new();
    for line in output.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 4
            || fields.iter().any(|s| s.is_empty())
            || !["x86_64", "any"].contains(&fields[2])
        {
            return Err(error("update_package_query"));
        }
        if fields[0] == "mpvkit-bin" {
            if target || fields[1] != version || fields[2] != "x86_64" {
                return Err(error("update_metadata"));
            }
            target = true;
        }
        changes.push(format!("{} {} ({})", fields[0], fields[1], fields[2]));
    }
    if !target {
        return Err(error("update_metadata"));
    }
    Ok(changes)
}
fn package_version(version: &str) -> String {
    format!(
        "{}-1",
        version
            .replace("-alpha.", "alpha")
            .replace("-beta.", "beta")
            .replace("-rc.", "rc")
    )
}
pub fn prepare(version: &str, digest: &str, bytes: &[u8]) -> Result<Plan> {
    let installed = installed()?;
    if format!("{:x}", Sha256::digest(bytes)) != digest {
        return Err(error("update_metadata"));
    }
    let directory = tempfile::Builder::new()
        .prefix("mpvkit-update-")
        .tempdir()
        .map_err(|_| error("update_package_temp"))?;
    let file = directory.path().join("mpvkit.pkg.tar.zst");
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&file)
        .map_err(|_| error("update_package_temp"))?;
    output
        .write_all(bytes)
        .and_then(|_| output.sync_all())
        .map_err(|_| error("update_package_temp"))?;
    let version = package_version(version);
    if query(&[
        "-Qp",
        "--",
        file.to_str().ok_or_else(|| error("update_package_temp"))?,
    ])? != format!("mpvkit-bin {version}")
    {
        return Err(error("update_metadata"));
    }
    let (transaction, changes) = transaction(
        file.to_str().ok_or_else(|| error("update_package_temp"))?,
        &version,
    )?;
    Ok(Plan {
        _directory: directory,
        file,
        digest: digest.into(),
        version,
        installed,
        transaction,
        changes,
        created: Instant::now(),
    })
}
pub fn install(plan: Plan) -> Result<()> {
    if plan.created.elapsed() > Duration::from_secs(1800) || installed()? != plan.installed {
        return Err(error("stale_preview"));
    }
    super::validate_local_path(&plan.file)?;
    let bytes = fs::read(&plan.file).map_err(|_| error("update_package_temp"))?;
    if format!("{:x}", Sha256::digest(&bytes)) != plan.digest {
        return Err(error("stale_preview"));
    }
    let path = plan
        .file
        .to_str()
        .ok_or_else(|| error("update_package_temp"))?;
    if transaction(path, &plan.version)?.0 != plan.transaction {
        return Err(error("stale_preview"));
    }
    let log = tempfile::tempfile().map_err(|_| error("update_package_temp"))?;
    let mut command = super::command(Path::new("/usr/bin/pkexec"));
    command.args([
        "--disable-internal-agent",
        "/usr/bin/pacman",
        "-U",
        "--noconfirm",
        "--",
        path,
    ]);
    // Pacman owns its transaction. Never apply probe/download timeouts or kill it on output limits.
    let status = command
        .stdin(Stdio::null())
        .stdout(log.try_clone().map_err(|_| error("update_package_temp"))?)
        .stderr(log)
        .status()
        .map_err(|_| error("update_package_install"))?;
    match status.code() {
        Some(126) => return Err(error("update_authorization_cancelled")),
        Some(127) => return Err(error("update_authorization_failed")),
        _ => {}
    }
    if !status.success() {
        return Err(error("update_package_install"));
    }
    if query(&["-Q", "--", "mpvkit-bin"])? != format!("mpvkit-bin {}", plan.version)
        || query(&["-Qoq", "--", "/usr/bin/mpvkit"])? != "mpvkit-bin"
    {
        return Err(error("update_package_incomplete"));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires an explicitly selected locally built Arch package and pacman"]
    fn packaged_metadata_matches_pacman() {
        let file = std::env::var("MPVKIT_TEST_PACKAGE").expect("select the CI-built package");
        let version = package_version(env!("CARGO_PKG_VERSION"));
        assert_eq!(
            query(&["-Qp", "--", &file]).unwrap(),
            format!("mpvkit-bin {version}")
        );
        let (_, changes) = transaction(&file, &version).unwrap();
        assert!(changes.iter().any(|line| line.starts_with("mpvkit-bin ")));
    }
    #[test]
    fn transaction_checks_target_architecture_and_package_version() {
        let line = "mpvkit-bin\t0.2.0alpha3-1\tx86_64\t/tmp/mpvkit.pkg.tar.zst";
        assert_eq!(parse_transaction(line, "0.2.0alpha3-1").unwrap().len(), 1);
        assert!(parse_transaction(line, "0.2.0alpha4-1").is_err());
        assert!(parse_transaction(&line.replace("x86_64", "aarch64"), "0.2.0alpha3-1").is_err());
        assert!(parse_transaction("unexpected prompt", "0.2.0alpha3-1").is_err());
        assert!(parse_transaction("other\t1\tany\t/repo/other", "0.2.0alpha3-1").is_err());
        assert_eq!(package_version("0.2.0-rc.2"), "0.2.0rc2-1");
    }
}
