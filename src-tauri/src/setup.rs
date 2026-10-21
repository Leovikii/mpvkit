use crate::{
    config, deploy,
    error::{AppError, Result},
    integration, mpv,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentHealth {
    pub id: String,
    pub version: Option<String>,
    pub ready: bool,
    pub issues: Vec<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub root: PathBuf,
    pub config_directory: PathBuf,
    pub external_mpv: Option<PathBuf>,
    pub generation: Option<String>,
    pub components: Vec<ComponentHealth>,
    pub player_ready: bool,
    pub components_ready: bool,
    pub settings_ready: bool,
    pub completed: bool,
    pub next_step: &'static str,
    pub recovery_required: bool,
    pub integration: integration::Status,
    pub issues: Vec<String>,
}
#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Progress {
    schema_version: u32,
    owner: String,
    environment_id: String,
    settings_hash: Option<String>,
    completed: bool,
}
fn error(code: &'static str) -> AppError {
    AppError::new(code, "安装状态已变化，请重新检查。", "setup")
}
fn read(root: &Path) -> Result<Option<Progress>> {
    let path = root.join(".mpvkit/setup.json");
    if deploy::file_hash(&path)?.is_none() {
        return Ok(None);
    }
    let data = deploy::read_bytes(&path)?;
    if data.len() > 65536 {
        return Err(error("setup_state"));
    }
    let p: Progress = serde_json::from_slice(&data).map_err(|_| error("setup_state"))?;
    if p.schema_version != 1 || p.owner != "MpvKit" {
        return Err(error("setup_state"));
    }
    Ok(Some(p))
}
pub fn environment(manifest: &deploy::InstallManifest) -> &str {
    manifest
        .environment_id
        .as_deref()
        .unwrap_or(&manifest.installation_id)
}
pub fn status(root: &Path) -> Result<Report> {
    let deployment = deploy::status(root)?;
    let integration = integration::status(root)?;
    let progress = read(root)?;
    let mut report = Report {
        root: root.into(),
        config_directory: root.join("config"),
        external_mpv: None,
        generation: None,
        components: vec![],
        player_ready: false,
        components_ready: false,
        settings_ready: false,
        completed: false,
        next_step: "location",
        recovery_required: deployment.recovery_required || integration.recovery_required,
        integration,
        issues: vec![],
    };
    if let Some(m) = deployment.manifest {
        report.generation = Some(m.installation_id.clone());
        report.external_mpv = m.external_mpv.clone();
        report.config_directory = root.join(&m.config_directory);
        let mut invalid = Vec::new();
        for file in &m.files {
            deploy::validate_relative(&file.path)?;
            if deploy::file_hash(&root.join(&file.path))?.as_deref() != Some(&file.sha256) {
                invalid.push(file.path.clone());
            }
        }
        report.player_ready = mpv::managed_executable(root, &m).is_ok();
        for (id, suffix) in [
            ("mpv", "runtime/mpv.exe"),
            ("uosc", "scripts/uosc/main.lua"),
            ("thumbfast", "scripts/thumbfast.lua"),
        ] {
            let required = if id == "mpv" || m.config_directory == "." {
                suffix.into()
            } else {
                format!("config/{suffix}")
            };
            let issues: Vec<_> = invalid
                .iter()
                .filter(|p| {
                    if id == "mpv" {
                        p.starts_with("runtime/")
                    } else if id == "uosc" {
                        p.contains("uosc") || p.contains("fonts/")
                    } else {
                        p.contains("thumbfast")
                    }
                })
                .cloned()
                .collect();
            let present = m.files.iter().any(|p| p.path == required);
            let ready = if id == "mpv" {
                report.player_ready
            } else {
                present && issues.is_empty()
            };
            report.components.push(ComponentHealth {
                id: id.into(),
                version: m.components.get(id).cloned(),
                ready,
                issues,
            });
        }
        let config_owned = [
            "mpv.conf",
            "scripts/mpvkit.lua",
            "mpvkit-context.json",
            "mpvkit-profiles.json",
        ]
        .iter()
        .all(|p| {
            let relative = if m.config_directory == "." {
                p.to_string()
            } else {
                format!("config/{p}")
            };
            m.files.iter().any(|f| f.path == relative)
        });
        report.components_ready =
            report.components.iter().all(|c| c.ready) && config_owned && invalid.is_empty();
        let p = progress.filter(|p| p.environment_id == environment(&m));
        let settings_hash = deploy::file_hash(&root.join("state/settings.json"))?;
        let settings_valid = settings_hash.is_some() && config::read(root).is_ok();
        // Old complete installations enter management without any startup writes.
        let legacy = p.is_none() && config::read(root).is_ok_and(|s| s.schema_version == 1);
        report.settings_ready = settings_valid
            && (legacy || p.as_ref().is_some_and(|p| p.settings_hash == settings_hash));
        report.completed = p.as_ref().is_some_and(|p| p.completed)
            || (legacy && report.components_ready && report.integration.valid);
        report.issues = invalid;
        report.next_step = if report.completed {
            "status"
        } else if !report.components_ready {
            "components"
        } else if !report.settings_ready {
            "playback"
        } else {
            "integration"
        };
    }
    if report.recovery_required {
        report.next_step = "recovery";
    }
    Ok(report)
}
// Called only after the user confirms a file change, never during startup.
// Persist the already verified legacy completion before settings v1 is replaced.
pub fn remember_completed_legacy(root: &Path) -> Result<()> {
    if read(root)?.is_none() && config::read(root)?.schema_version == 1 {
        let state = status(root)?;
        if state.completed {
            confirm(
                root,
                state
                    .generation
                    .as_deref()
                    .ok_or_else(|| error("not_installed"))?,
                true,
            )?;
        }
    }
    Ok(())
}
pub fn require_components(root: &Path) -> Result<()> {
    let state = status(root)?;
    if state.recovery_required || !state.components_ready {
        return Err(error("components_required"));
    }
    Ok(())
}
pub fn confirm(root: &Path, generation: &str, finish: bool) -> Result<Report> {
    let _lock = deploy::acquire(root)?;
    let state = status(root)?;
    if state.generation.as_deref() != Some(generation)
        || !state.components_ready
        || state.recovery_required
    {
        return Err(error("setup_changed"));
    }
    if finish && (!state.settings_ready || !state.integration.valid) {
        return Err(error("integration_required"));
    }
    let m = deploy::status(root)?
        .manifest
        .ok_or_else(|| error("not_installed"))?;
    let existing = read(root)?;
    let progress = Progress {
        schema_version: 1,
        owner: "MpvKit".into(),
        environment_id: environment(&m).into(),
        settings_hash: deploy::file_hash(&root.join("state/settings.json"))?,
        completed: finish
            || existing.is_some_and(|p| p.environment_id == environment(&m) && p.completed),
    };
    deploy::atomic_write(
        &root.join(".mpvkit/setup.json"),
        &serde_json::to_vec_pretty(&progress).map_err(|_| error("setup_state"))?,
    )?;
    status(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    fn fixture(root: &Path) {
        let mut files: Vec<deploy::Payload> = [
            "runtime/mpv.exe",
            "config/mpv.conf",
            "config/scripts/mpvkit.lua",
            "config/mpvkit-context.json",
            "config/mpvkit-profiles.json",
            "config/scripts/uosc/main.lua",
            "config/scripts/thumbfast.lua",
        ]
        .into_iter()
        .map(|p| deploy::Payload {
            path: p.into(),
            bytes: b"fixture".to_vec(),
        })
        .collect();
        files.push(deploy::Payload {
            path: "state/settings.json".into(),
            bytes: serde_json::to_vec(&config::Settings::default()).unwrap(),
        });
        deploy::execute(
            deploy::prepare_install(root, files, false, None, Default::default()).unwrap(),
        )
        .unwrap();
    }
    #[test]
    fn health_and_progress_never_unlock_missing_components_or_stale_generations() {
        let root = std::env::temp_dir().join(format!("mpvkit-setup-{}", deploy::operation_id()));
        let empty = status(&root).unwrap();
        assert_eq!(empty.next_step, "location");
        assert!(!empty.player_ready);
        assert!(!root.exists());
        assert!(require_components(&root).is_err());
        fixture(&root);
        let state = status(&root).unwrap();
        assert!(state.components_ready);
        assert_eq!(state.next_step, "playback");
        assert!(confirm(&root, "stale", false).is_err());
        assert!(confirm(&root, state.generation.as_ref().unwrap(), true).is_err());
        let confirmed = confirm(&root, state.generation.as_ref().unwrap(), false).unwrap();
        assert!(confirmed.settings_ready);
        assert_eq!(confirmed.next_step, "integration");
        fs::remove_file(root.join("config/scripts/thumbfast.lua")).unwrap();
        let broken = status(&root).unwrap();
        assert!(broken.player_ready);
        assert!(!broken.components_ready);
        assert_eq!(broken.next_step, "components");
        assert!(confirm(&root, state.generation.as_ref().unwrap(), false).is_err());
        fs::write(root.join(".mpvkit/setup.json"),br#"{"schemaVersion":99,"owner":"MpvKit","environmentId":"x","settingsHash":null,"completed":true}"#).unwrap();
        assert!(status(&root).is_err());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn environment_identity_survives_updates_but_not_uninstall_reinstall() {
        let root = std::env::temp_dir().join(format!("mpvkit-setup-{}", deploy::operation_id()));
        fixture(&root);
        let before = deploy::status(&root).unwrap().manifest.unwrap();
        let settings = config::Settings {
            profile: config::Profile::HighQuality,
            ..Default::default()
        };
        deploy::execute(config::prepare(&root, settings).unwrap()).unwrap();
        let after = deploy::status(&root).unwrap().manifest.unwrap();
        assert_ne!(before.installation_id, after.installation_id);
        assert_eq!(environment(&before), environment(&after));
        deploy::execute(deploy::prepare_uninstall(&root).unwrap()).unwrap();
        fixture(&root);
        let fresh = deploy::status(&root).unwrap().manifest.unwrap();
        assert_ne!(environment(&before), environment(&fresh));
        fs::remove_dir_all(root).unwrap();
    }
}
