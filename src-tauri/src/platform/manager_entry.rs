//! The manager owns its desktop launcher; media associations belong to the MPV package.
use crate::{
    deploy::{self, DeploymentPlan, Payload},
    error::{AppError, Result},
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

pub const APP_ID: &str = "io.github.mpvkit";
const ENTRY: &str = "io.github.mpvkit.desktop";
const ICON: &str = "desktop/mpvkit.png";
const SOURCE: &str = "desktop/io.github.mpvkit.desktop";

fn conflict() -> AppError {
    AppError::new(
        "manager_entry_conflict",
        "管理器桌面入口或图标存在手改/未知内容，已保留；请检查用户级应用目录。",
        "desktop",
    )
}

fn exec_argument(value: &str) -> Result<String> {
    if value.chars().any(char::is_control) {
        return Err(conflict());
    }
    // Exec quoting first, desktop-entry string escaping second (no shell involved).
    let mut quoted = String::from("\"");
    for c in value.chars() {
        match c {
            '\\' | '"' | '`' | '$' => {
                quoted.push('\\');
                quoted.push(c);
            }
            '%' => quoted.push_str("%%"),
            _ => quoted.push(c),
        }
    }
    quoted.push('"');
    Ok(quoted.replace('\\', "\\\\"))
}

struct Plan {
    files: DeploymentPlan,
    link: PathBuf,
    target: PathBuf,
    linked: bool,
}

fn linked(link: &Path, target: &Path) -> Result<bool> {
    super::validate_local_path(link.parent().expect("applications directory"))?;
    match fs::symlink_metadata(link) {
        Ok(metadata)
            if metadata.file_type().is_symlink()
                && fs::read_link(link).ok().as_deref() == Some(target) =>
        {
            Ok(true)
        }
        Ok(_) => Err(conflict()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(conflict()),
    }
}

fn prepare(root: &Path, applications: &Path, executable: &Path) -> Result<Plan> {
    super::validate_local_path(executable)?;
    if !executable.is_file() {
        return Err(conflict());
    }
    let link = applications.join(ENTRY);
    let target = root.join(SOURCE);
    let linked = linked(&link, &target)?;
    let executable = exec_argument(executable.to_str().ok_or_else(conflict)?)?;
    let icon = root.join(ICON);
    super::validate_local_path(&icon)?;
    let icon = icon.to_str().ok_or_else(conflict)?.replace('\\', "\\\\");
    if icon.chars().any(char::is_control) {
        return Err(conflict());
    }
    let entry = format!(
        "[Desktop Entry]\nType=Application\nName=MpvKit\nComment=Manage MPV installation and settings\nExec={executable}\nIcon={icon}\nTerminal=false\nCategories=AudioVideo;\nStartupWMClass={APP_ID}\n"
    );
    let files = deploy::prepare_install(
        root,
        vec![
            Payload {
                path: SOURCE.into(),
                bytes: entry.into_bytes(),
            },
            Payload {
                path: ICON.into(),
                bytes: include_bytes!("../../icons/icon.png").to_vec(),
            },
        ],
        false,
        std::collections::BTreeMap::from([("mpvkit".into(), env!("CARGO_PKG_VERSION").into())]),
    )?;
    if !files.preview.conflicts.is_empty() {
        return Err(conflict());
    }
    Ok(Plan {
        files,
        link,
        target,
        linked,
    })
}

fn execute(plan: Plan) -> Result<bool> {
    if linked(&plan.link, &plan.target)? != plan.linked {
        return Err(conflict());
    }
    let changed = !plan.files.preview.no_changes || !plan.linked;
    if changed {
        eprintln!(
            "MpvKit desktop preview: {} -> {}; {}",
            plan.link.display(),
            plan.target.display(),
            serde_json::to_string(&plan.files.preview).unwrap_or_default()
        );
    }
    // Reuse file ownership, fresh-plan checks and transaction recovery for both assets.
    deploy::execute(plan.files)?;
    if !plan.linked {
        super::validate_local_path(plan.link.parent().expect("applications directory"))?;
        fs::create_dir_all(plan.link.parent().expect("applications directory"))
            .map_err(|_| conflict())?;
        super::validate_local_path(plan.link.parent().expect("applications directory"))?;
        // create_new semantics: never replace a user file, even if it appeared during deployment.
        std::os::unix::fs::symlink(&plan.target, &plan.link).map_err(|_| conflict())?;
    }
    Ok(changed)
}

pub fn ensure() -> Result<()> {
    let root = super::default_root()?;
    if deploy::status(&root)?.recovery_required {
        let recovery = deploy::prepare_recovery(&root)?;
        if !recovery.preview.conflicts.is_empty() {
            return Err(conflict());
        }
        eprintln!(
            "MpvKit desktop recovery: {}",
            serde_json::to_string(&recovery.preview).unwrap_or_default()
        );
        deploy::recover(recovery)?;
    }
    #[cfg(debug_assertions)]
    let isolated =
        cfg!(test) || std::env::var_os("MPVKIT_TEST_MODE").is_some_and(|value| value == "1");
    #[cfg(not(debug_assertions))]
    let isolated = false;
    let applications = if isolated {
        root.join("applications")
    } else {
        super::xdg_directory("XDG_DATA_HOME", ".local/share")?.join("applications")
    };
    let executable = std::env::current_exe().map_err(|_| conflict())?;
    if execute(prepare(&root, &applications, &executable)?)?
        && !isolated
        && Path::new("/usr/bin/kbuildsycoca6").is_file()
    {
        crate::mpv::capture(
            Path::new("/usr/bin/kbuildsycoca6"),
            &["--noincremental".into()],
            Duration::from_secs(5),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoting_preserves_spaces_field_codes_and_special_characters() {
        assert_eq!(exec_argument("/tmp/a b%f").unwrap(), "\"/tmp/a b%%f\"");
        assert_eq!(
            exec_argument("a\"$`\\").unwrap(),
            "\"a\\\\\"\\\\$\\\\`\\\\\\\\\""
        );
        assert!(exec_argument("/tmp/a\nExec=bad").is_err());
    }

    #[test]
    fn manager_launcher_updates_owned_files_without_media_configuration() {
        let sandbox =
            std::env::temp_dir().join(format!("mpvkit manager-%f-{}", deploy::operation_id()));
        fs::create_dir(&sandbox).unwrap();
        let executable = sandbox.join("manager %f");
        fs::write(&executable, "fixture").unwrap();
        let root = sandbox.join("data");
        let applications = sandbox.join("applications");
        let plan = prepare(&root, &applications, &executable).unwrap();
        assert!(!root.exists() && !applications.exists());
        assert!(execute(plan).unwrap());
        assert_eq!(
            fs::read_link(applications.join(ENTRY)).unwrap(),
            root.join(SOURCE)
        );
        let content = fs::read_to_string(applications.join(ENTRY)).unwrap();
        assert!(!content.contains("--open") && !content.contains("MimeType="));
        assert!(content.contains("manager %%f\"\n"));
        assert_eq!(
            fs::read(root.join(ICON)).unwrap(),
            include_bytes!("../../icons/icon.png")
        );
        assert!(!execute(prepare(&root, &applications, &executable).unwrap()).unwrap());
        let other = sandbox.join("manager relocated");
        fs::write(&other, "fixture").unwrap();
        assert!(execute(prepare(&root, &applications, &other).unwrap()).unwrap());
        assert!(
            fs::read_to_string(applications.join(ENTRY))
                .unwrap()
                .contains("manager relocated")
        );
        fs::write(root.join(ICON), "manual icon").unwrap();
        assert!(prepare(&root, &applications, &other).is_err());
        assert_eq!(fs::read_to_string(root.join(ICON)).unwrap(), "manual icon");
        fs::remove_dir_all(sandbox).unwrap();
    }

    #[test]
    fn unknown_launchers_and_changed_plans_are_preserved() {
        let sandbox =
            std::env::temp_dir().join(format!("mpvkit-manager-guard-{}", deploy::operation_id()));
        fs::create_dir(&sandbox).unwrap();
        let root = sandbox.join("data");
        let executable = sandbox.join("manager");
        fs::write(&executable, "fixture").unwrap();
        let applications = sandbox.join("applications");
        let plan = prepare(&root, &applications, &executable).unwrap();
        fs::create_dir(&applications).unwrap();
        fs::write(applications.join(ENTRY), "manual launcher").unwrap();
        assert!(execute(plan).is_err());
        assert!(!root.exists());
        assert!(prepare(&root, &applications, &executable).is_err());
        assert_eq!(
            fs::read_to_string(applications.join(ENTRY)).unwrap(),
            "manual launcher"
        );
        fs::remove_file(applications.join(ENTRY)).unwrap();
        std::os::unix::fs::symlink("/tmp/unknown.desktop", applications.join(ENTRY)).unwrap();
        assert!(prepare(&root, &applications, &executable).is_err());
        fs::remove_dir_all(sandbox).unwrap();
    }
}
