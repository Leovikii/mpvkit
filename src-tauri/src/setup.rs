use crate::{
    config, deploy,
    error::{AppError, Result},
    mpv,
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
    pub recovery_required: bool,
    pub issues: Vec<String>,
}
pub fn status(root: &Path) -> Result<Report> {
    let deployment = deploy::status(root)?;
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
        recovery_required: deployment.recovery_required,
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
    pub retained: Vec<String>,
}
pub struct ChangePlan {
    pub files: deploy::DeploymentPlan,
    pub retained: Vec<String>,
}
impl ChangePlan {
    pub fn preview(&self) -> ChangePreview {
        ChangePreview {
            files: self.files.preview.clone(),
            retained: self.retained.clone(),
        }
    }
}
pub fn files_only(files: deploy::DeploymentPlan) -> ChangePlan {
    ChangePlan {
        files,
        retained: vec![],
    }
}
pub fn prepare_draft(
    root: &Path,
    settings: config::Settings,
    replace: bool,
    expected: &str,
) -> Result<ChangePlan> {
    if config::portable::fingerprint(root)? != expected {
        return Err(AppError::new(
            "stale_preview",
            "配置在预览后改变，请重新预览。",
            "config",
        ));
    }
    let files = config::prepare_portable(root, settings, replace)?;
    if config::portable::fingerprint(root)? != expected {
        return Err(AppError::new(
            "stale_preview",
            "配置在预览后改变，请重新预览。",
            "config",
        ));
    }
    Ok(files_only(files))
}
pub fn prepare_uninstall(root: &Path) -> Result<ChangePlan> {
    let m = deploy::status(root)?
        .manifest
        .ok_or_else(|| AppError::new("not_installed", "尚未安装播放增强。", "setup"))?;
    let files = deploy::prepare_install(root, vec![], false, Default::default())?;
    let files = files.for_uninstall();
    let mut plan = files_only(files);
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
    deploy::execute(plan.files)
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
            prepare_draft(&root, settings.clone(), false, &preview.fingerprint)
                .err()
                .unwrap()
                .code,
            "components_required"
        );
        core(&root);
        let plan = prepare_draft(&root, settings.clone(), false, &preview.fingerprint).unwrap();
        execute(plan).unwrap();
        let preview = config::portable::preview(&root, &settings, false).unwrap();
        assert!(
            preview
                .files
                .iter()
                .all(|f| f.retained && f.modified_at.is_some())
        );
        let plan = prepare_draft(&root, settings.clone(), false, &preview.fingerprint).unwrap();
        std::fs::remove_file(root.join(crate::config::relative!("scripts/thumbfast.lua"))).unwrap();
        assert!(execute(plan).is_err());
        assert_eq!(
            prepare_draft(&root, settings.clone(), false, &preview.fingerprint)
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
            prepare_draft(&root, settings, false, &preview.fingerprint)
                .err()
                .unwrap()
                .code,
            "stale_preview"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn uninstall_preserves_user_content_and_rejects_changed_script() {
        let root =
            std::env::temp_dir().join(format!("mpvkit-files-only-{}", deploy::operation_id()));
        core(&root);
        let preview = config::portable::preview(&root, &Default::default(), false).unwrap();
        execute(prepare_draft(&root, Default::default(), false, &preview.fingerprint).unwrap())
            .unwrap();
        let report = serde_json::to_value(status(&root).unwrap()).unwrap();
        assert!(report.get("integration").is_none());
        assert!(report.get("integrationEnabled").is_none());
        let state = serde_json::to_value(config::portable::read_state(&root).unwrap()).unwrap();
        assert!(state.get("integrationEnabled").is_none());
        let helper = root.join("scripts/mpvkit.lua");
        let original = std::fs::read(&helper).unwrap();
        let stale = prepare_uninstall(&root).unwrap();
        std::fs::write(&helper, "changed").unwrap();
        assert_eq!(execute(stale).unwrap_err().code, "stale_preview");
        std::fs::write(&helper, original).unwrap();
        std::fs::write(root.join("private.txt"), "mine").unwrap();
        std::fs::create_dir(root.join("watch_later")).unwrap();
        std::fs::write(root.join("watch_later/position"), "start=10").unwrap();
        execute(prepare_uninstall(&root).unwrap()).unwrap();
        assert!(!helper.exists());
        assert!(!root.join("mpv.conf").exists());
        assert_eq!(
            std::fs::read_to_string(root.join("private.txt")).unwrap(),
            "mine"
        );
        assert!(root.join("watch_later/position").is_file());
        assert!(deploy::status(&root).unwrap().manifest.is_none());
        #[cfg(target_os = "windows")]
        assert!(root.join("mpv.exe").is_file());
        std::fs::remove_dir_all(root).unwrap();
    }
}
