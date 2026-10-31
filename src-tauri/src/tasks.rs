use crate::{
    components, config, deploy,
    error::{AppError, Result},
    logging, native_core, platform, removal, setup,
};
use serde::Serialize;
use std::{path::PathBuf, sync::atomic::AtomicBool, time::Instant};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    #[serde(flatten)]
    pub files: setup::ChangePreview,
    pub needs_core: bool,
    pub kind: &'static str,
    pub core_action: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stage {
    pub name: &'static str,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub output: String,
    pub error: Option<AppError>,
    pub detected: Option<bool>,
}
#[derive(Serialize)]
pub struct Outcome {
    pub stages: Vec<Stage>,
}
impl Stage {
    fn files(result: Result<()>) -> Self {
        Self {
            name: "configuration",
            success: result.is_ok(),
            exit_code: None,
            output: String::new(),
            error: result.err(),
            detected: None,
        }
    }
}
pub enum Plan {
    Install {
        files: Box<deploy::DeploymentPlan>,
        player: PathBuf,
        needs_core: bool,
    },
    Remove {
        id: String,
        files: removal::Plan,
    },
    Core {
        id: String,
        root: PathBuf,
        action: String,
        created: Instant,
    },
}
fn empty(id: String, root: PathBuf) -> setup::ChangePreview {
    setup::ChangePreview {
        files: deploy::DeploymentPreview {
            id,
            root,
            changes: vec![],
            conflicts: vec![],
            configuration_diffs: vec![],
            no_changes: false,
        },
        retained: vec![],
    }
}
impl Plan {
    pub fn preview(&self) -> Preview {
        match self {
            Self::Install {
                files, needs_core, ..
            } => Preview {
                files: setup::files_only_preview(files),
                needs_core: *needs_core,
                kind: "install",
                core_action: needs_core.then(|| "install".into()),
            },
            Self::Remove { id, files } => {
                let mut preview = empty(id.clone(), files.root.clone());
                preview.files.changes = files
                    .paths()
                    .into_iter()
                    .map(|path| deploy::Change {
                        path,
                        before: Some("reviewed".into()),
                        after: None,
                        backup: None,
                    })
                    .collect();
                Preview {
                    files: preview,
                    needs_core: false,
                    kind: "remove",
                    core_action: None,
                }
            }
            Self::Core {
                id, root, action, ..
            } => Preview {
                files: empty(id.clone(), root.clone()),
                needs_core: false,
                kind: "core",
                core_action: Some(action.clone()),
            },
        }
    }
}
pub fn prepare_install(
    root: &std::path::Path,
    settings: config::Settings,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(components::Progress),
) -> Result<Plan> {
    settings.validate()?;
    let report = setup::status(root)?;
    if report.recovery_required {
        return Err(AppError::new(
            "recovery_required",
            "请先恢复未完成操作。",
            "install",
        ));
    }
    if report.player_installed && !report.player_ready {
        return Err(AppError::new(
            "core_unusable",
            "请先修复 MPV 核心。",
            "install",
        ));
    }
    let player = config::player_path(root)?;
    let configuration = config::portable::preview(root, &settings, true)?;
    let selected: Vec<_> = components::ASSETS
        .iter()
        .filter(|asset| {
            !report
                .components
                .iter()
                .any(|h| h.id == asset.id && h.ready && h.version.as_deref() == Some(asset.version))
        })
        .map(|a| a.id.into())
        .collect();
    let (mut payloads, versions) = components::selected_payloads(&selected, cancel, progress)?;
    payloads.extend(config::portable::installation_files(root, &settings)?);
    let mut files = deploy::prepare_update(root, payloads, true, versions)?;
    files.preview.configuration_diffs = configuration
        .files
        .into_iter()
        .filter(|f| f.before != f.after)
        .map(|f| deploy::ConfigurationDiff {
            path: f.path,
            before: f.before,
            after: f.after,
        })
        .collect();
    if platform::app_root()? != root
        || config::portable::fingerprint(root)? != configuration.fingerprint
    {
        return Err(AppError::new(
            "stale_preview",
            "配置位置或内容已变化。",
            "install",
        ));
    }
    if report.player_ready {
        files.guard(player.clone(), deploy::file_hash(&player)?);
    }
    Ok(Plan::Install {
        files: Box::new(files),
        player,
        needs_core: !report.player_ready,
    })
}
pub fn prepare_remove(root: &std::path::Path) -> Result<Plan> {
    Ok(Plan::Remove {
        id: deploy::operation_id(),
        files: removal::prepare(root)?,
    })
}
pub fn prepare_core(root: &std::path::Path, action: String) -> Result<Plan> {
    if !matches!(action.as_str(), "install" | "upgrade" | "uninstall") {
        return Err(AppError::new(
            "invalid_action",
            "不支持此核心操作。",
            "system-mpv",
        ));
    }
    Ok(Plan::Core {
        id: deploy::operation_id(),
        root: root.into(),
        action,
        created: Instant::now(),
    })
}
fn core_stage(action: &str, task: &str) -> Stage {
    let started = Instant::now();
    let result = native_core::run(action);
    let detected = platform::app_root()
        .and_then(|p| config::player_path(&p))
        .ok()
        .map(|p| p.is_file());
    let (exit_code, output, error) = match result {
        Ok(outcome) => (outcome.exit_code, outcome.output, None),
        Err(e) => (None, String::new(), Some(e)),
    };
    logging::record(
        task,
        action,
        "core",
        if exit_code == Some(0) && error.is_none() {
            "success"
        } else {
            "failed"
        },
        error.as_ref().map(|e| e.code),
        exit_code,
        started.elapsed().as_millis() as u64,
    );
    Stage {
        name: "core",
        success: exit_code == Some(0) && error.is_none(),
        exit_code,
        output,
        error,
        detected,
    }
}
pub fn execute(
    plan: Plan,
    remove_core: bool,
    progress: &mut impl FnMut(components::Progress),
) -> Result<Outcome> {
    let preview = plan.preview();
    let _application_lock = deploy::acquire(&platform::default_root()?)?;
    if platform::app_root()? != preview.files.files.root {
        return Err(AppError::new("stale_preview", "配置位置已变化。", "setup"));
    }
    execute_steps(plan, remove_core, progress, core_stage)
}
fn execute_steps(
    plan: Plan,
    remove_core: bool,
    progress: &mut impl FnMut(components::Progress),
    mut core: impl FnMut(&str, &str) -> Stage,
) -> Result<Outcome> {
    let preview = plan.preview();
    let task = preview.files.files.id.clone();
    let started = Instant::now();
    logging::record(&task, preview.kind, "task", "started", None, None, 0);
    let mut stages = vec![];
    let event = |phase| components::Progress {
        component: String::new(),
        phase,
        received: 0,
        total: 0,
    };
    match plan {
        Plan::Install {
            mut files,
            player,
            needs_core,
        } => {
            files.validate()?;
            if needs_core {
                progress(event("package-manager"));
                let stage = core("install", &task);
                let success = stage.success;
                stages.push(stage);
                if !success {
                    return Ok(Outcome { stages });
                }
            }
            progress(event("install-components"));
            let result = (|| {
                if platform::app_root()? != files.preview.root
                    || config::player_path(&files.preview.root)? != player
                {
                    return Err(AppError::new(
                        "stale_preview",
                        "安装位置已变化，请重新确认。",
                        "install",
                    ));
                }
                crate::mpv::inspect(&player)?;
                crate::mpv::ensure_managed_options(&player)?;
                if needs_core {
                    files.guard(player.clone(), deploy::file_hash(&player)?);
                }
                let root = files.preview.root.clone();
                deploy::execute(*files)?;
                let report = setup::status(&root)?;
                if !report.components_ready || !report.settings_ready {
                    return Err(AppError::new(
                        "installation_incomplete",
                        "配置安装后检查未通过。",
                        "install",
                    ));
                }
                Ok(())
            })();
            stages.push(Stage::files(result));
        }
        Plan::Remove { files, .. } => {
            let _target_lock = deploy::lock_target(&files.root)?;
            files.validate()?;
            if remove_core {
                progress(event("package-manager"));
                stages.push(core("uninstall", &task));
            }
            progress(event("remove-configuration"));
            stages.push(Stage::files(removal::execute(&files)));
        }
        Plan::Core {
            action, created, ..
        } => {
            if created.elapsed().as_secs() > 300 {
                return Err(AppError::new(
                    "stale_preview",
                    "操作预览已过期。",
                    "system-mpv",
                ));
            }
            progress(event("package-manager"));
            stages.push(core(&action, &task));
        }
    }
    for stage in &stages {
        logging::record(
            &task,
            preview.kind,
            stage.name,
            if stage.success { "success" } else { "failed" },
            stage.error.as_ref().map(|e| e.code),
            stage.exit_code,
            started.elapsed().as_millis() as u64,
        );
    }
    Ok(Outcome { stages })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn failed_core(_: &str, _: &str) -> Stage {
        Stage {
            name: "core",
            success: false,
            exit_code: Some(1),
            output: "native failure".into(),
            error: None,
            detected: Some(true),
        }
    }
    #[test]
    fn failed_core_install_does_not_write_configuration() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("mpv");
        let files = deploy::prepare_update(
            &root,
            vec![deploy::Payload {
                path: "mpv.conf".into(),
                bytes: b"new".to_vec(),
            }],
            true,
            Default::default(),
        )
        .unwrap();
        let result = execute_steps(
            Plan::Install {
                files: Box::new(files),
                player: dir.path().join("missing"),
                needs_core: true,
            },
            false,
            &mut |_| {},
            failed_core,
        )
        .unwrap();
        assert_eq!(result.stages.len(), 1);
        assert!(!result.stages[0].success);
        assert!(!root.exists());
    }
    #[test]
    fn failed_core_uninstall_still_removes_all_reviewed_configuration() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("mpv");
        std::fs::create_dir_all(root.join("watch_later")).unwrap();
        std::fs::write(root.join("private.conf"), "manual").unwrap();
        std::fs::write(root.join("watch_later/resume"), "position").unwrap();
        let result = execute_steps(
            prepare_remove(&root).unwrap(),
            true,
            &mut |_| {},
            failed_core,
        )
        .unwrap();
        assert!(!result.stages[0].success);
        assert_eq!(result.stages[0].exit_code, Some(1));
        assert!(result.stages[1].success);
        assert!(!root.exists());
    }
}
