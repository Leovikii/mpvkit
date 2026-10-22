use crate::{
    config, deploy,
    error::{AppError, Result},
    integration, mpv,
};
use serde::Serialize;
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
    pub configuration_modified: bool,
    pub completed: bool,
    pub next_step: &'static str,
    pub recovery_required: bool,
    pub integration: integration::Status,
    pub issues: Vec<String>,
}
fn error(code: &'static str) -> AppError {
    AppError::new(code, "安装状态已变化，请重新检查。", "setup")
}
pub fn status(root: &Path) -> Result<Report> {
    let deployment = deploy::status(root)?;
    let integration = integration::status(root)?;
    let mut report = Report {
        root: root.into(),
        config_directory: root.join("portable_config"),
        external_mpv: None,
        generation: None,
        components: vec![],
        player_ready: false,
        components_ready: false,
        settings_ready: false,
        configuration_modified: false,
        completed: false,
        next_step: "components",
        recovery_required: deployment.recovery_required || integration.recovery_required,
        integration,
        issues: vec![],
    };
    if let Some(m) = deployment.manifest {
        report.configuration_modified = config::modified(root)?;
        report.generation = Some(m.installation_id.clone());
        report.external_mpv = m.external_mpv.clone();
        report.config_directory = root.join(&m.config_directory);
        let mut invalid = Vec::new();
        for file in &m.files {
            deploy::validate_relative(&file.path)?;
            if (!config::editable(&file.path)
                || deploy::file_hash(&root.join(&file.path))?.is_none())
                && deploy::file_hash(&root.join(&file.path))?.as_deref() != Some(&file.sha256)
            {
                invalid.push(file.path.clone());
            }
        }
        report.player_ready = mpv::managed_executable(root, &m).is_ok();
        for (id, suffix) in [
            ("mpv", "mpv.exe"),
            ("uosc", "scripts/uosc/main.lua"),
            ("thumbfast", "scripts/thumbfast.lua"),
            ("subtitle-font", "fonts/LXGWWenKaiScreen.ttf"),
        ] {
            let required = if id == "mpv" {
                suffix.into()
            } else {
                format!("portable_config/{suffix}")
            };
            let issues: Vec<_> = invalid
                .iter()
                .filter(|p| {
                    if id == "mpv" {
                        p.as_str() == "mpv.exe" || p.as_str() == "d3dcompiler_43.dll"
                    } else if id == "uosc" {
                        p.contains("uosc")
                    } else if id == "subtitle-font" {
                        p.ends_with("fonts/LXGWWenKaiScreen.ttf")
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
        let config_owned = ["mpv.conf", "scripts/mpvkit.lua"].iter().all(|p| {
            let relative = format!("portable_config/{p}");
            m.files.iter().any(|f| f.path == relative)
        });
        report.components_ready =
            report.components.iter().all(|c| c.ready) && config_owned && invalid.is_empty();
        let state = config::portable::read_state(root)?;
        report.settings_ready = state.confirmed;
        report.completed = state.completed;
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
pub fn require_components(root: &Path) -> Result<()> {
    let state = status(root)?;
    if state.recovery_required || !state.components_ready {
        return Err(error("components_required"));
    }
    Ok(())
}
pub fn confirm(root: &Path, generation: &str, finish: bool) -> Result<Report> {
    let report = status(root)?;
    if report.generation.as_deref() != Some(generation)
        || !report.components_ready
        || report.recovery_required
    {
        return Err(error("setup_changed"));
    }
    if finish && (!report.settings_ready || !report.integration.valid) {
        return Err(error("integration_required"));
    }
    let mut state = config::portable::read_state(root)?;
    state.confirmed = true;
    state.completed |= finish;
    config::portable::save_state(root, state, generation)?;
    status(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    fn fixture(root: &Path) {
        let mut files = config::generate(root, &root.join("mpv.exe"), &Default::default()).unwrap();
        files.extend(
            [
                "mpv.exe",
                "portable_config/scripts/uosc/main.lua",
                "portable_config/scripts/thumbfast.lua",
                "portable_config/fonts/LXGWWenKaiScreen.ttf",
            ]
            .into_iter()
            .map(|p| deploy::Payload {
                path: p.into(),
                bytes: b"fixture".to_vec(),
            }),
        );
        deploy::execute(
            deploy::prepare_install(root, files, false, None, Default::default()).unwrap(),
        )
        .unwrap();
    }
    #[test]
    fn health_and_progress_never_unlock_missing_components_or_stale_generations() {
        let root = std::env::temp_dir().join(format!("mpvkit-setup-{}", deploy::operation_id()));
        let empty = status(&root).unwrap();
        assert_eq!(empty.next_step, "components");
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
        fs::remove_file(root.join("portable_config/scripts/thumbfast.lua")).unwrap();
        let broken = status(&root).unwrap();
        assert!(broken.player_ready);
        assert!(!broken.components_ready);
        assert_eq!(broken.next_step, "components");
        assert!(confirm(&root, state.generation.as_ref().unwrap(), false).is_err());
        let unknown = config::portable::State {
            schema_version: 99,
            ..Default::default()
        };
        fs::write(
            root.join(".mpvkit/settings.json"),
            serde_json::to_vec(&unknown).unwrap(),
        )
        .unwrap();
        assert!(status(&root).is_err());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn subtitle_font_is_required_and_its_integrity_is_checked() {
        let root = std::env::temp_dir().join(format!("mpvkit-font-{}", deploy::operation_id()));
        fixture(&root);
        assert!(status(&root).unwrap().components_ready);
        let manifest = deploy::status(&root).unwrap().manifest.unwrap();
        let mut files: Vec<_> = manifest
            .files
            .iter()
            .map(|f| deploy::Payload {
                path: f.path.clone(),
                bytes: fs::read(root.join(&f.path)).unwrap(),
            })
            .collect();
        let font = "portable_config/fonts/LXGWWenKaiScreen.ttf";
        files.iter_mut().find(|f| f.path == font).unwrap().bytes = b"font".to_vec();
        let versions = [("subtitle-font".into(), "1.530".into())].into();
        deploy::execute(deploy::prepare_install(&root, files, false, None, versions).unwrap())
            .unwrap();
        assert!(status(&root).unwrap().components_ready);
        fs::remove_file(root.join(font)).unwrap();
        let broken = status(&root).unwrap();
        assert!(!broken.components_ready);
        assert!(
            !broken
                .components
                .iter()
                .find(|c| c.id == "subtitle-font")
                .unwrap()
                .ready
        );
        assert!(
            broken
                .components
                .iter()
                .find(|c| c.id == "uosc")
                .unwrap()
                .ready
        );
        fs::write(root.join(font), b"font").unwrap();
        assert!(status(&root).unwrap().components_ready);
        fs::write(root.join(font), b"changed").unwrap();
        assert!(!status(&root).unwrap().components_ready);
        fs::remove_dir_all(root).unwrap();
    }
}
