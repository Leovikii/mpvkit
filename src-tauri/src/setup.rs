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
    pub player_directory: PathBuf,
    pub config_directory: PathBuf,
    pub config_directory_source: &'static str,
    pub root_exists: bool,
    pub config_directory_exists: bool,
    pub generation: Option<String>,
    pub components: Vec<ComponentHealth>,
    pub player_ready: bool,
    pub player_installed: bool,
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
    let config_directory = config::directory(root)?;
    crate::platform::validate_local_path(&config_directory)?;
    let mut report = Report {
        root: root.into(),
        player_directory: config::player_path(root)?.parent().unwrap().into(),
        root_exists: root.is_dir(),
        config_directory_exists: config_directory.is_dir(),
        config_directory,
        config_directory_source: {
            #[cfg(target_os = "linux")]
            {
                crate::platform::mpv_config_location()?.source
            }
            #[cfg(target_os = "windows")]
            {
                crate::platform::windows_mpv::config_location()?.source
            }
        },
        generation: None,
        components: vec![],
        player_ready: false,
        player_installed: config::player_path(root)?.is_file(),
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
    {
        let player = mpv::inspect(&config::player_path(root)?)
            .and_then(|info| {
                mpv::ensure_managed_options(&info.executable)?;
                Ok(info)
            })
            .ok();
        report.components.push(ComponentHealth {
            id: "mpv".into(),
            ready: player.is_some(),
            version: player.map(|p| p.version),
            issues: vec![],
        });
        report.player_ready = report.components[0].ready;
    }
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
        report.player_ready = report.components[0].ready;
        for id in ["uosc", "thumbfast", "subtitle-font"] {
            let issues: Vec<_> = report
                .issues
                .iter()
                .filter(|p| match id {
                    "uosc" => p.contains("uosc"),
                    "thumbfast" => p.as_str() == crate::config::relative!("scripts/thumbfast.lua"),
                    _ => p.ends_with("fonts/LXGWWenKaiScreen.ttf"),
                })
                .cloned()
                .collect();
            let ready = issues.is_empty()
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
        report.configuration_installed = m
            .files
            .iter()
            .any(|f| f.path == crate::config::relative!("mpv.conf"));
        report.settings_ready = config::portable::ready(root)?;
        report.configuration_modified = config::modified(root)?;
        // Actual main-file mtime; never substitutes a cached save/inspection time.
        report.config_modified_at =
            config::portable::modified_at(&root.join(crate::config::relative!("mpv.conf")));
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
    #[cfg(target_os = "linux")]
    #[test]
    fn system_mpv_is_visible_without_mpvkit_installation() {
        let root =
            std::env::temp_dir().join(format!("mpvkit-system-status-{}", deploy::operation_id()));
        let path = config::player_path(&root).unwrap();
        let expected = mpv::inspect(&path)
            .ok()
            .filter(|_| mpv::ensure_managed_options(&path).is_ok());
        let report = status(&root).unwrap();
        let player = report.components.iter().find(|c| c.id == "mpv").unwrap();
        assert_eq!(player.ready, expected.is_some());
        assert_eq!(player.version, expected.map(|p| p.version));
        assert_eq!(report.player_ready, player.ready);
        assert!(!report.components_ready && !report.settings_ready);
        assert!(report.generation.is_none());
        assert!(!root.exists());
    }
    #[test]
    fn core_and_configuration_have_independent_health_and_real_mtime() {
        let root = std::env::temp_dir().join(format!("mpvkit-health-{}", deploy::operation_id()));
        let path = config::player_path(&root).unwrap();
        assert_eq!(
            status(&root).unwrap().player_ready,
            mpv::inspect(&path).is_ok() && mpv::ensure_managed_options(&path).is_ok()
        );
        assert!(!root.exists());
        #[cfg(target_os = "windows")]
        mpv::fixture_player(&root);
        let mut files = Vec::new();
        for name in crate::components::ASSETS
            .iter()
            .flat_map(|a| crate::components::required_paths(a.id))
        {
            files.push(deploy::Payload {
                path: (*name).into(),
                bytes: b"fixture".to_vec(),
            });
        }
        deploy::execute(
            deploy::prepare_install(
                &root,
                files,
                false,
                std::collections::BTreeMap::from([("mpv".into(), "fixture".into())]),
            )
            .unwrap(),
        )
        .unwrap();
        let core = status(&root).unwrap();
        assert_eq!(
            core.player_ready,
            config::player_path(&root).unwrap().is_file()
        );
        assert!(!core.settings_ready);
        assert!(!root.join(crate::config::relative!("mpv.conf")).exists());
        deploy::execute(config::prepare_portable(&root, Default::default(), false).unwrap())
            .unwrap();
        let configured = status(&root).unwrap();
        assert!(configured.settings_ready);
        assert!(configured.config_modified_at.is_some());
        let path = root.join(crate::config::relative!("mpv.conf"));
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
        #[cfg(target_os = "windows")]
        {
            std::fs::remove_file(root.join("mpv.exe")).unwrap();
            assert!(config::prepare_portable(&root, Default::default(), false).is_err());
        }
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
#[cfg(all(test, target_os = "windows"))]
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
    let m = deploy::status(root)?
        .manifest
        .ok_or_else(|| AppError::new("not_installed", "尚未安装播放增强。", "setup"))?;
    // The integration record is removed first by execute(); the plan itself remains read-only.
    let files = deploy::prepare_install(root, vec![], false, Default::default())?;
    let mut files = files.for_uninstall();
    files.remove_generated_logs(crate::logs::uninstall_files(root)?)?;
    let mut plan = with_integration(files, false)?;
    for file in &m.files {
        if deploy::file_hash(&root.join(&file.path))?.is_some_and(|hash| hash != file.sha256) {
            plan.retained.push(file.path.clone());
        }
    }
    plan.retained
        .push(crate::config::relative!("watch_later/").into());
    Ok(plan)
}
pub fn execute(plan: ChangePlan) -> Result<deploy::DeploymentStatus> {
    #[cfg(not(test))]
    if crate::platform::app_root()? != plan.files.preview.root {
        return Err(AppError::new(
            "stale_preview",
            "配置位置已改变，请重新预览。",
            "setup",
        ));
    }
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
        #[cfg(target_os = "windows")]
        mpv::fixture_player(root);
        files.push(deploy::Payload {
            path: ".mpvkit/settings.json".into(),
            bytes: serde_json::to_vec(&config::portable::State::default()).unwrap(),
        });
        for name in crate::components::ASSETS
            .iter()
            .flat_map(|a| crate::components::required_paths(a.id))
        {
            files.push(deploy::Payload {
                path: (*name).into(),
                bytes: b"fixture".to_vec(),
            });
        }
        deploy::execute(
            deploy::prepare_install(
                root,
                files,
                false,
                std::collections::BTreeMap::from([("mpv".into(), "fixture".into())]),
            )
            .unwrap(),
        )
        .unwrap();
    }
    #[test]
    fn draft_preview_is_read_only_and_apply_requires_components_and_fresh_files() {
        let root = std::env::temp_dir().join(format!("mpvkit-draft-{}", deploy::operation_id()));
        let settings = config::Settings::default();
        let preview = config::portable::preview(&root, &settings, false).unwrap();
        assert_eq!(preview.files.len(), 4);
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
        std::fs::remove_file(root.join(crate::config::relative!("scripts/thumbfast.lua"))).unwrap();
        assert!(execute(plan).is_err());
        assert_eq!(
            prepare_draft(&root, settings.clone(), false, false, &preview.fingerprint)
                .err()
                .unwrap()
                .code,
            "components_required"
        );
        std::fs::write(
            root.join(crate::config::relative!("input.conf")),
            "SPACE ignore\n",
        )
        .unwrap();
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
    #[cfg(target_os = "windows")]
    fn registration_failure_keeps_configuration_and_uninstall_failure_keeps_helper() {
        let root = std::env::temp_dir().join(format!("mpvkit-combined-{}", deploy::operation_id()));
        core(&root);
        let plan = prepare_configuration(&root, Default::default(), false).unwrap();
        assert_eq!(plan.preview().integration.unwrap().values.len(), 15);
        assert!(!root.join(crate::config::relative!("mpv.conf")).exists());
        crate::platform::registry::FAIL_WRITE.with(|f| f.set(true));
        assert_eq!(execute(plan).err().unwrap().code, "integration_incomplete");
        assert!(status(&root).unwrap().settings_ready);
        assert!(!status(&root).unwrap().integration.registered);
        let conf = deploy::read_bytes(&root.join(crate::config::relative!("mpv.conf"))).unwrap();
        execute(prepare_intent(&root, true).unwrap()).unwrap();
        assert!(status(&root).unwrap().integration.valid);
        assert_eq!(
            deploy::read_bytes(&root.join(crate::config::relative!("mpv.conf"))).unwrap(),
            conf
        );
        let removal = prepare_uninstall(&root).unwrap();
        crate::platform::registry::FAIL_WRITE.with(|f| f.set(true));
        assert!(execute(removal).is_err());
        assert!(root.join(".mpvkit/launcher/mpvkit.exe").exists());
        assert!(status(&root).unwrap().integration.valid);
        std::fs::write(root.join(crate::config::relative!("private.txt")), "mine").unwrap();
        std::fs::create_dir(root.join(crate::config::relative!("watch_later"))).unwrap();
        std::fs::write(
            root.join(crate::config::relative!("watch_later/position")),
            "start=10",
        )
        .unwrap();
        let removal = prepare_uninstall(&root).unwrap();
        assert!(!removal.preview().retained.is_empty());
        execute(removal).unwrap();
        assert!(root.join("mpv.exe").exists());
        assert!(!status(&root).unwrap().integration.registered);
        assert!(!root.join(crate::config::relative!("mpv.conf")).exists());
        assert!(root.join(crate::config::relative!("private.txt")).exists());
        assert!(
            root.join(crate::config::relative!("watch_later/position"))
                .exists()
        );
        assert!(deploy::status(&root).unwrap().manifest.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[cfg(target_os = "windows")]
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
    #[cfg(target_os = "windows")]
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
        assert!(!root.join(crate::config::relative!("mpv.conf")).exists());
        registry
            .write("Software\\RegisteredApplications", "MpvKit", None)
            .unwrap();
        execute(prepare_configuration(&root, Default::default(), false).unwrap()).unwrap();
        let plan = prepare_uninstall(&root).unwrap();
        std::fs::write(config::helper_path(&root), "changed").unwrap();
        assert!(execute(plan).is_err());
        assert!(status(&root).unwrap().integration.registered);
        std::fs::remove_dir_all(root).unwrap();
    }
}
