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
    pub root_exists: bool,
    pub config_directory_exists: bool,
    pub generation: Option<String>,
    pub components: Vec<ComponentHealth>,
    pub player_ready: bool,
    pub components_ready: bool,
    pub settings_ready: bool,
    pub configuration_installed: bool,
    pub configuration_modified: bool,
    pub config_modified_at: Option<u64>,
    pub integration_enabled: bool,
    pub recovery_required: bool,
    pub integration: integration::Status,
    pub issues: Vec<String>,
}
pub fn status(root: &Path) -> Result<Report> {
    let deployment = deploy::status(root)?;
    let integration = integration::status(root)?;
    let state = config::portable::read_state(root)?;
    let config_directory = root.join("portable_config");
    crate::platform::validate_local_path(&config_directory)?;
    let mut report = Report {
        root: root.into(),
        root_exists: root.is_dir(),
        config_directory_exists: config_directory.is_dir(),
        config_directory,
        generation: None,
        components: vec![],
        player_ready: false,
        components_ready: false,
        settings_ready: false,
        configuration_installed: false,
        configuration_modified: false,
        config_modified_at: None,
        integration_enabled: state.integration_enabled,
        recovery_required: deployment.recovery_required || integration.recovery_required,
        integration,
        issues: vec![],
    };
    if let Some(m) = deployment.manifest {
        report.generation = Some(m.installation_id.clone());
        for file in &m.files {
            deploy::validate_relative(&file.path)?;
            let hash = deploy::file_hash(&root.join(&file.path))?;
            if (!config::editable(&file.path) || hash.is_none())
                && hash.as_deref() != Some(&file.sha256)
            {
                report.issues.push(file.path.clone());
            }
        }
        report.player_ready = mpv::managed_executable(root, &m).is_ok();
        for id in ["mpv", "uosc", "thumbfast", "subtitle-font"] {
            let issues: Vec<_> = report
                .issues
                .iter()
                .filter(|p| match id {
                    "mpv" => p.as_str() == "mpv.exe" || p.as_str() == "d3dcompiler_43.dll",
                    "uosc" => p.contains("uosc"),
                    "thumbfast" => p.as_str() == "portable_config/scripts/thumbfast.lua",
                    _ => p.ends_with("fonts/LXGWWenKaiScreen.ttf"),
                })
                .cloned()
                .collect();
            let ready = issues.is_empty()
                && (id != "mpv" || report.player_ready)
                && crate::components::required_paths(id)
                    .iter()
                    .all(|required| {
                        m.files.iter().any(|f| f.path == *required) && root.join(required).is_file()
                    });
            report.components.push(ComponentHealth {
                id: id.into(),
                version: m.components.get(id).cloned(),
                ready,
                issues,
            });
        }
        report.components_ready = report.components.iter().all(|c| c.ready);
        report.configuration_installed =
            m.files.iter().any(|f| f.path == "portable_config/mpv.conf");
        report.settings_ready = config::portable::ready(root)?;
        report.configuration_modified = config::modified(root)?;
        // Actual main-file mtime; never substitutes a cached save/inspection time.
        report.config_modified_at =
            config::portable::modified_at(&root.join("portable_config/mpv.conf"));
    }
    Ok(report)
}
pub fn require_components(root: &Path) -> Result<()> {
    let report = status(root)?;
    if report.recovery_required || !report.components_ready {
        return Err(AppError::new(
            "components_required",
            "请先安装或修复播放器。",
            "setup",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn core_and_configuration_have_independent_health_and_real_mtime() {
        let root = std::env::temp_dir().join(format!("mpvkit-health-{}", deploy::operation_id()));
        assert!(!status(&root).unwrap().player_ready);
        assert!(!root.exists());
        let mut files = vec![deploy::Payload {
            path: "mpv.exe".into(),
            bytes: b"fixture".to_vec(),
        }];
        for name in crate::components::ASSETS
            .iter()
            .flat_map(|a| crate::components::required_paths(a.id))
            .filter(|p| **p != "mpv.exe")
        {
            files.push(deploy::Payload {
                path: (*name).into(),
                bytes: b"fixture".to_vec(),
            });
        }
        deploy::execute(deploy::prepare_install(&root, files, false, Default::default()).unwrap())
            .unwrap();
        let core = status(&root).unwrap();
        assert!(core.player_ready);
        assert!(!core.settings_ready);
        assert!(!root.join("portable_config/mpv.conf").exists());
        deploy::execute(config::prepare_portable(&root, Default::default(), false).unwrap())
            .unwrap();
        let configured = status(&root).unwrap();
        assert!(configured.settings_ready);
        assert!(configured.config_modified_at.is_some());
        let path = root.join("portable_config/mpv.conf");
        let mut bytes = std::fs::read(&path).unwrap();
        bytes.extend(
            b"
volume=20
",
        );
        std::fs::write(&path, bytes).unwrap();
        let changed = status(&root).unwrap();
        assert!(changed.settings_ready && changed.configuration_modified);
        assert_eq!(
            changed.config_modified_at,
            Some(
                std::fs::metadata(&path)
                    .unwrap()
                    .modified()
                    .unwrap()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64
            )
        );
        std::fs::remove_file(&path).unwrap();
        assert!(!status(&root).unwrap().settings_ready);
        assert_eq!(status(&root).unwrap().config_modified_at, None);
        deploy::execute(config::prepare_portable(&root, Default::default(), false).unwrap())
            .unwrap();
        std::fs::remove_file(root.join("mpv.exe")).unwrap();
        assert!(config::prepare_portable(&root, Default::default(), false).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePreview {
    #[serde(flatten)]
    pub files: deploy::DeploymentPreview,
    pub integration: Option<integration::Preview>,
    pub retained: Vec<String>,
}
pub struct ChangePlan {
    pub files: deploy::DeploymentPlan,
    pub integration: Option<integration::Plan>,
    pub unregister_first: bool,
    pub retained: Vec<String>,
}
impl ChangePlan {
    pub fn preview(&self) -> ChangePreview {
        ChangePreview {
            files: self.files.preview.clone(),
            integration: self.integration.as_ref().map(|p| p.preview.clone()),
            retained: self.retained.clone(),
        }
    }
}
pub fn with_integration(files: deploy::DeploymentPlan, enabled: bool) -> Result<ChangePlan> {
    let root = &files.preview.root;
    let current = integration::status(root)?;
    let registration = if enabled {
        Some(integration::prepare_after_files(&files)?)
    } else if current.registered {
        Some(integration::prepare(root, "unregister")?)
    } else {
        None
    };
    Ok(ChangePlan {
        files,
        integration: registration,
        unregister_first: !enabled,
        retained: vec![],
    })
}
pub fn files_only(files: deploy::DeploymentPlan) -> ChangePlan {
    ChangePlan {
        files,
        integration: None,
        unregister_first: false,
        retained: vec![],
    }
}
#[cfg(test)]
pub fn prepare_configuration(
    root: &Path,
    settings: config::Settings,
    replace: bool,
) -> Result<ChangePlan> {
    let files = config::prepare_portable(root, settings, replace)?;
    with_integration(
        files,
        config::portable::read_state(root)?.integration_enabled,
    )
}
pub fn prepare_draft(
    root: &Path,
    settings: config::Settings,
    replace: bool,
    enabled: bool,
    expected: &str,
) -> Result<ChangePlan> {
    if config::portable::fingerprint(root)? != expected {
        return Err(AppError::new(
            "stale_preview",
            "配置在预览后改变，请重新预览。",
            "config",
        ));
    }
    let files = config::portable::prepare_draft(root, settings, replace, enabled)?;
    if config::portable::fingerprint(root)? != expected {
        return Err(AppError::new(
            "stale_preview",
            "配置在预览后改变，请重新预览。",
            "config",
        ));
    }
    with_integration(files, enabled)
}
pub fn prepare_intent(root: &Path, enabled: bool) -> Result<ChangePlan> {
    let files = config::portable::prepare_intent(root, enabled)?;
    if config::portable::ready(root)? || !enabled {
        if enabled {
            require_components(root)?;
        }
        with_integration(files, enabled)
    } else {
        Ok(files_only(files))
    }
}
pub fn prepare_uninstall(root: &Path) -> Result<ChangePlan> {
    let mut plan = with_integration(config::portable::prepare_remove_player(root)?, false)?;
    let m = deploy::status(root)?.manifest.unwrap();
    plan.retained = m
        .files
        .iter()
        .filter(|f| config::editable(&f.path) || f.path == ".mpvkit/settings.json")
        .map(|f| f.path.clone())
        .collect();
    plan.retained.push("portable_config/watch_later/".into());
    Ok(plan)
}
pub fn execute(plan: ChangePlan) -> Result<deploy::DeploymentStatus> {
    plan.files.validate_before_integration()?;
    if let Some(registration) = &plan.integration {
        integration::validate_pending(registration)?;
    }
    if plan.unregister_first {
        if let Some(registration) = plan.integration {
            integration::execute(registration)?;
        }
        deploy::execute(plan.files)
    } else {
        let result = deploy::execute(plan.files)?;
        if let Some(registration) = plan.integration {
            integration::execute(registration).map_err(|_| {
                AppError::new(
                    "integration_incomplete",
                    "文件已保存，系统打开方式未完成。请重试登记或恢复。",
                    "integration",
                )
            })?;
        }
        Ok(result)
    }
}

#[cfg(test)]
mod transaction_tests {
    use super::*;
    fn core(root: &Path) {
        let mut files = config::runtime_files().unwrap();
        files.push(deploy::Payload {
            path: "mpv.exe".into(),
            bytes: b"fixture".to_vec(),
        });
        files.push(deploy::Payload {
            path: ".mpvkit/settings.json".into(),
            bytes: serde_json::to_vec(&config::portable::State::default()).unwrap(),
        });
        for name in crate::components::ASSETS
            .iter()
            .flat_map(|a| crate::components::required_paths(a.id))
            .filter(|p| **p != "mpv.exe")
        {
            files.push(deploy::Payload {
                path: (*name).into(),
                bytes: b"fixture".to_vec(),
            });
        }
        deploy::execute(deploy::prepare_install(root, files, false, Default::default()).unwrap())
            .unwrap();
    }
    #[test]
    fn draft_preview_is_read_only_and_apply_requires_components_and_fresh_files() {
        let root = std::env::temp_dir().join(format!("mpvkit-draft-{}", deploy::operation_id()));
        let settings = config::Settings::default();
        let preview = config::portable::preview(&root, &settings, false).unwrap();
        assert_eq!(preview.files.len(), 3);
        assert!(
            preview
                .files
                .iter()
                .all(|f| f.before.is_empty() && f.modified_at.is_none())
        );
        assert!(!root.exists());
        assert_eq!(
            prepare_draft(&root, settings.clone(), false, false, &preview.fingerprint)
                .err()
                .unwrap()
                .code,
            "components_required"
        );
        core(&root);
        let plan =
            prepare_draft(&root, settings.clone(), false, false, &preview.fingerprint).unwrap();
        execute(plan).unwrap();
        let preview = config::portable::preview(&root, &settings, false).unwrap();
        assert!(
            preview
                .files
                .iter()
                .all(|f| f.retained && f.modified_at.is_some())
        );
        let plan =
            prepare_draft(&root, settings.clone(), false, false, &preview.fingerprint).unwrap();
        std::fs::remove_file(root.join("portable_config/scripts/thumbfast.lua")).unwrap();
        assert!(execute(plan).is_err());
        assert_eq!(
            prepare_draft(&root, settings.clone(), false, false, &preview.fingerprint)
                .err()
                .unwrap()
                .code,
            "components_required"
        );
        std::fs::write(root.join("portable_config/input.conf"), "SPACE ignore\n").unwrap();
        assert_eq!(
            prepare_draft(&root, settings, false, false, &preview.fingerprint)
                .err()
                .unwrap()
                .code,
            "stale_preview"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn registration_failure_keeps_configuration_and_uninstall_failure_keeps_helper() {
        let root = std::env::temp_dir().join(format!("mpvkit-combined-{}", deploy::operation_id()));
        core(&root);
        let plan = prepare_configuration(&root, Default::default(), false).unwrap();
        assert_eq!(plan.preview().integration.unwrap().values.len(), 15);
        assert!(!root.join("portable_config/mpv.conf").exists());
        crate::platform::registry::FAIL_WRITE.with(|f| f.set(true));
        assert_eq!(execute(plan).err().unwrap().code, "integration_incomplete");
        assert!(status(&root).unwrap().settings_ready);
        assert!(!status(&root).unwrap().integration.registered);
        let conf = deploy::read_bytes(&root.join("portable_config/mpv.conf")).unwrap();
        execute(prepare_intent(&root, true).unwrap()).unwrap();
        assert!(status(&root).unwrap().integration.valid);
        assert_eq!(
            deploy::read_bytes(&root.join("portable_config/mpv.conf")).unwrap(),
            conf
        );
        let removal = prepare_uninstall(&root).unwrap();
        crate::platform::registry::FAIL_WRITE.with(|f| f.set(true));
        assert!(execute(removal).is_err());
        assert!(root.join(".mpvkit/launcher/mpvkit.exe").exists());
        assert!(status(&root).unwrap().integration.valid);
        std::fs::write(root.join("portable_config/private.txt"), "mine").unwrap();
        std::fs::create_dir(root.join("portable_config/watch_later")).unwrap();
        std::fs::write(
            root.join("portable_config/watch_later/position"),
            "start=10",
        )
        .unwrap();
        let removal = prepare_uninstall(&root).unwrap();
        assert!(!removal.preview().retained.is_empty());
        execute(removal).unwrap();
        assert!(!root.join("mpv.exe").exists());
        assert!(!status(&root).unwrap().integration.registered);
        assert_eq!(
            deploy::read_bytes(&root.join("portable_config/mpv.conf")).unwrap(),
            conf
        );
        assert!(root.join("portable_config/private.txt").exists());
        assert!(root.join("portable_config/watch_later/position").exists());
        assert!(deploy::status(&root).unwrap().manifest.is_some());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn standalone_registration_rejects_a_modified_helper() {
        let root = std::env::temp_dir().join(format!("mpvkit-helper-{}", deploy::operation_id()));
        core(&root);
        execute(prepare_configuration(&root, Default::default(), false).unwrap()).unwrap();
        execute(prepare_intent(&root, false).unwrap()).unwrap();
        std::fs::write(config::helper_path(&root), "changed helper").unwrap();
        assert_eq!(
            integration::prepare(&root, "register").err().unwrap().code,
            "helper_changed"
        );
        assert!(!status(&root).unwrap().integration.registered);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn stale_registration_preview_cannot_write_files_and_stale_uninstall_cannot_unregister() {
        let root =
            std::env::temp_dir().join(format!("mpvkit-stale-combined-{}", deploy::operation_id()));
        core(&root);
        let plan = prepare_configuration(&root, Default::default(), false).unwrap();
        let registry = crate::platform::registry::Registry::test(&root);
        registry
            .write(
                "Software\\RegisteredApplications",
                "MpvKit",
                Some(&crate::platform::registry::RegistryData::string("other")),
            )
            .unwrap();
        assert!(execute(plan).is_err());
        assert!(!root.join("portable_config/mpv.conf").exists());
        registry
            .write("Software\\RegisteredApplications", "MpvKit", None)
            .unwrap();
        execute(prepare_configuration(&root, Default::default(), false).unwrap()).unwrap();
        let plan = prepare_uninstall(&root).unwrap();
        std::fs::write(root.join("mpv.exe"), "changed").unwrap();
        assert!(execute(plan).is_err());
        assert!(status(&root).unwrap().integration.registered);
        std::fs::remove_dir_all(root).unwrap();
    }
}
