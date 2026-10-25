#[cfg(target_os = "linux")]
use crate::platform::desktop_entry::{Data as RegistryData, Store as Registry};
#[cfg(target_os = "windows")]
use crate::platform::registry::{Registry, RegistryData};
use crate::{
    deploy,
    error::{AppError, Result},
};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
const RECORD: &str = ".mpvkit/associations.json";
const PENDING: &str = ".mpvkit/association-pending.json";
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Value {
    pub key: String,
    pub name: String,
    pub before: Option<RegistryData>,
    pub after: Option<RegistryData>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    schema_version: u32,
    owner: String,
    values: Vec<Value>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Journal {
    schema_version: u32,
    owner: String,
    old_record: Option<Vec<u8>>,
    new_record: Option<Vec<u8>>,
    changes: Vec<Value>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub id: String,
    pub values: Vec<Value>,
    pub conflicts: Vec<String>,
    pub skipped: Vec<String>,
    pub action: String,
}
pub struct Plan {
    pub preview: Preview,
    pub(crate) root: PathBuf,
    created: Instant,
    before_hash: Option<String>,
    pending_hash: Option<String>,
    journal: Journal,
    registry: Registry,
    guarded_files: Vec<(PathBuf, Option<String>)>,
    observed_values: Vec<(String, String, Option<RegistryData>)>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub registered: bool,
    pub valid: bool,
    pub issue: Option<String>,
    pub recovery_required: bool,
}
fn error(code: &'static str, message: &'static str) -> AppError {
    AppError::new(code, message, "integration")
}
fn bytes(root: &Path, name: &str) -> Result<Option<Vec<u8>>> {
    let path = root.join(name);
    crate::platform::validate_local_path(&path)?;
    if !path.exists() {
        return Ok(None);
    }
    let bytes = deploy::read_bytes(&path)?;
    if bytes.len() > 1024 * 1024 {
        return Err(error("integration_state", "系统集成记录过大，已保留。"));
    }
    Ok(Some(bytes))
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|_| error("integration_state", "系统集成记录损坏，已保留。"))?;
    if value["schemaVersion"] != 1 || value["owner"] != "MpvKit" {
        return Err(error(
            "integration_state",
            "系统集成记录版本或归属未知，已保留。",
        ));
    }
    serde_json::from_value(value)
        .map_err(|_| error("integration_state", "系统集成记录字段无效，已保留。"))
}
pub fn status(root: &Path) -> Result<Status> {
    status_with(root, &Registry::current(root))
}
fn status_with(root: &Path, registry: &Registry) -> Result<Status> {
    deploy::status(root)?;
    let record = bytes(root, RECORD)?;
    if let Some(data) = &record {
        let _: Record = decode(data)?;
    }
    let pending = bytes(root, PENDING)?;
    if let Some(data) = &pending {
        let _: Journal = decode(data)?;
    }
    let mut valid = record.is_some();
    if let Some(data) = &record {
        let saved: Record = decode(data)?;
        valid = !saved.values.is_empty();
        for value in &saved.values {
            valid &= registry.read(&value.key, &value.name)? == value.after;
        }
    }
    if valid {
        let manifest = deploy::status(root)?.manifest;
        let launcher = manifest
            .as_ref()
            .and_then(|m| m.files.iter().find(|f| f.path == crate::platform::HELPER));
        if let Some(file) = launcher {
            valid &= deploy::file_hash(&root.join(&file.path))?.as_deref() == Some(&file.sha256);
            let saved: Record = decode(record.as_ref().expect("record exists"))?;
            for (key, name, value) in desired(root)? {
                valid &= saved
                    .values
                    .iter()
                    .any(|v| v.key == key && v.name == name && v.after.as_ref() == Some(&value));
            }
        } else {
            valid = false;
        }
    }
    Ok(Status {
        valid: valid && pending.is_none(),
        issue: (record.is_some() && !valid).then(|| "integration_changed".into()),
        registered: record.is_some(),
        recovery_required: pending.is_some(),
    })
}
#[cfg(target_os = "windows")]
fn desired(root: &Path) -> Result<Vec<(String, String, RegistryData)>> {
    let executable = crate::config::helper_path(root);
    crate::platform::validate_local_path(&executable)?;
    let executable = executable
        .to_str()
        .filter(|s| !s.contains('"'))
        .ok_or_else(|| error("helper_path", "启动入口路径无效。"))?;
    let mut values = vec![
        (
            "Software\\Classes\\MpvKit.Media".into(),
            "".into(),
            RegistryData::string("MpvKit 媒体文件"),
        ),
        (
            "Software\\Classes\\MpvKit.Media\\shell\\open\\command".into(),
            "".into(),
            RegistryData::string(&format!("\"{executable}\" --open \"%1\"")),
        ),
        (
            "Software\\MpvKit\\Capabilities".into(),
            "ApplicationName".into(),
            RegistryData::string("MpvKit"),
        ),
        (
            "Software\\MpvKit\\Capabilities".into(),
            "ApplicationDescription".into(),
            RegistryData::string("使用 MpvKit 受管理 MPV 环境播放媒体"),
        ),
        (
            "Software\\RegisteredApplications".into(),
            "MpvKit".into(),
            RegistryData::string("Software\\MpvKit\\Capabilities"),
        ),
    ];
    for extension in [
        ".mkv", ".mp4", ".webm", ".avi", ".mov", ".mp3", ".flac", ".wav", ".m4v", ".ts",
    ] {
        values.push((
            "Software\\MpvKit\\Capabilities\\FileAssociations".into(),
            extension.into(),
            RegistryData::string("MpvKit.Media"),
        ));
    }
    Ok(values)
}
#[cfg(target_os = "linux")]
fn desired(root: &Path) -> Result<Vec<(String, String, RegistryData)>> {
    Ok(vec![(
        "io.github.mpvkit.desktop".into(),
        String::new(),
        RegistryData::string(&crate::platform::desktop_entry::content(root)?),
    )])
}
pub fn prepare(root: &Path, action: &str) -> Result<Plan> {
    let mut plan = prepare_with(root, action, Registry::current(root))?;
    if action == "register" {
        let manifest = deploy::status(root)?
            .manifest
            .ok_or_else(|| error("not_installed", "请先安装播放器。"))?;
        crate::mpv::managed_executable(root, &manifest)?;
        if !crate::config::portable::ready(root)? {
            return Err(error("settings_required", "请先保存播放设置。"));
        }
        let manifest = deploy::status(root)?
            .manifest
            .ok_or_else(|| error("not_installed", "请先安装播放器。"))?;
        if !manifest
            .files
            .iter()
            .any(|f| f.path == crate::platform::HELPER)
        {
            return Err(error(
                "helper_missing",
                "请先保存播放设置以准备稳定启动入口。",
            ));
        }
        for file in &manifest.files {
            let path = root.join(&file.path);
            let hash = deploy::file_hash(&path)?;
            if file.path == crate::platform::HELPER && hash.as_deref() != Some(&file.sha256) {
                return Err(error("helper_changed", "启动入口已改变，请先修复组件。"));
            }
            plan.guarded_files.push((path, hash));
        }
        for name in [".mpvkit/install.json", ".mpvkit/settings.json"] {
            let path = root.join(name);
            plan.guarded_files
                .push((path.clone(), deploy::file_hash(&path)?));
        }
    }
    Ok(plan)
}
pub fn prepare_after_files(files: &deploy::DeploymentPlan) -> Result<Plan> {
    let root = &files.preview.root;
    let mut plan = prepare_with(root, "register", Registry::current(root))?;
    plan.guarded_files = files.expected_files()?;
    let helper = crate::config::helper_path(root);
    if !plan
        .guarded_files
        .iter()
        .any(|(p, h)| p == &helper && h.is_some())
    {
        return Err(error("helper_missing", "启动入口缺失。"));
    }
    Ok(plan)
}
pub fn validate_pending(plan: &Plan) -> Result<()> {
    if plan.created.elapsed() > Duration::from_secs(300)
        || !plan.preview.conflicts.is_empty()
        || deploy::file_hash(&plan.root.join(RECORD))? != plan.before_hash
        || deploy::file_hash(&plan.root.join(PENDING))? != plan.pending_hash
    {
        return Err(error("stale_preview", "系统集成预览已变化，请重新预览。"));
    }
    for (key, name, before) in &plan.observed_values {
        if &plan.registry.read(key, name)? != before {
            return Err(error("stale_preview", "预览后系统集成值已改变。"));
        }
    }
    Ok(())
}
fn prepare_with(root: &Path, action: &str, registry: Registry) -> Result<Plan> {
    let deployment = deploy::status(root)?;
    if deployment.recovery_required {
        return Err(error("recovery_required", "请先恢复文件部署。"));
    }
    let old_record = bytes(root, RECORD)?;
    let pending = bytes(root, PENDING)?;
    let mut conflicts = Vec::new();
    let mut skipped = Vec::new();
    let mut observed_values = Vec::new();
    let journal = if action == "recover" {
        let data = pending
            .as_ref()
            .ok_or_else(|| error("no_recovery", "没有未完成系统集成操作。"))?;
        let journal: Journal = decode(data)?;
        for change in &journal.changes {
            let current = registry.read(&change.key, &change.name)?;
            if current != change.before && current != change.after {
                conflicts.push(format!("{} / {}", change.key, change.name));
            }
        }
        if old_record != journal.old_record && old_record != journal.new_record {
            conflicts.push("系统集成归属记录".into());
        }
        journal
    } else {
        if pending.is_some() {
            return Err(error("recovery_required", "请先恢复未完成系统集成操作。"));
        }
        let old: Option<Record> = old_record.as_deref().map(decode).transpose()?;
        let mut changes = Vec::new();
        let new_record = if action == "register" {
            if deployment.manifest.is_none() {
                return Err(error("not_installed", "请先安装受管理播放环境。"));
            }
            let mut owned = Vec::new();
            for (key, name, data) in desired(root)? {
                let before = registry.read(&key, &name)?;
                observed_values.push((key.clone(), name.clone(), before.clone()));
                let previous = old
                    .as_ref()
                    .and_then(|r| r.values.iter().find(|v| v.key == key && v.name == name));
                if before.is_some() && previous.is_none_or(|p| before != p.after) {
                    conflicts.push(format!("{key} / {name}"));
                }
                let after = Some(data);
                owned.push(Value {
                    key: key.clone(),
                    name: name.clone(),
                    before: previous
                        .map(|v| v.before.clone())
                        .unwrap_or_else(|| before.clone()),
                    after: after.clone(),
                });
                if before != after {
                    changes.push(Value {
                        key,
                        name,
                        before,
                        after,
                    });
                }
            }
            Some(
                serde_json::to_vec(&Record {
                    schema_version: 1,
                    owner: "MpvKit".into(),
                    values: owned,
                })
                .map_err(|_| error("integration_state", "无法生成系统集成记录。"))?,
            )
        } else if action == "unregister" {
            let old =
                old.ok_or_else(|| error("not_registered", "没有本应用的系统集成归属记录。"))?;
            for value in old.values {
                let current = registry.read(&value.key, &value.name)?;
                observed_values.push((value.key.clone(), value.name.clone(), current.clone()));
                if current == value.after {
                    changes.push(Value {
                        key: value.key,
                        name: value.name,
                        before: current,
                        after: value.before,
                    });
                } else {
                    skipped.push(format!(
                        "{} / {}：已由其他来源改变，保留",
                        value.key, value.name
                    ));
                }
            }
            None
        } else {
            return Err(error("invalid_action", "系统集成操作无效。"));
        };
        Journal {
            schema_version: 1,
            owner: "MpvKit".into(),
            old_record: old_record.clone(),
            new_record,
            changes,
        }
    };
    let preview = Preview {
        id: deploy::operation_id(),
        values: journal.changes.clone(),
        conflicts,
        skipped,
        action: action.into(),
    };
    Ok(Plan {
        preview,
        root: root.into(),
        created: Instant::now(),
        before_hash: old_record.as_ref().map(|b| deploy::hash_bytes(b)),
        pending_hash: pending.as_ref().map(|b| deploy::hash_bytes(b)),
        journal,
        registry,
        guarded_files: Vec::new(),
        observed_values,
    })
}
fn record(root: &Path, content: Option<&[u8]>) -> Result<()> {
    if let Some(bytes) = content {
        deploy::atomic_write(&root.join(RECORD), bytes)
    } else {
        match std::fs::remove_file(root.join(RECORD)) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err(error("integration_state", "无法清理系统集成记录。")),
        }
    }
}
fn rollback(plan: &Plan) -> Result<()> {
    for change in plan.journal.changes.iter().rev() {
        let current = plan.registry.read(&change.key, &change.name)?;
        if current == change.before {
            continue;
        }
        if current != change.after {
            return Err(error(
                "integration_conflict",
                "恢复时发现其他来源修改，已保留记录并停止。",
            ));
        }
        plan.registry
            .write(&change.key, &change.name, change.before.as_ref())?;
    }
    record(&plan.root, plan.journal.old_record.as_deref())?;
    std::fs::remove_file(plan.root.join(PENDING))
        .map_err(|_| error("integration_state", "恢复完成但记录未能清理，请重新检查。"))
}
pub fn execute(plan: Plan) -> Result<Status> {
    if plan.created.elapsed() > Duration::from_secs(300) || !plan.preview.conflicts.is_empty() {
        return Err(error("stale_preview", "预览已过期或含冲突，请重新检查。"));
    }
    if plan.preview.action != "recover" {
        validate_pending(&plan)?;
    }
    let _lock = deploy::acquire(&plan.root)?;
    if deploy::status(&plan.root)?.recovery_required
        || deploy::file_hash(&plan.root.join(RECORD))? != plan.before_hash
        || deploy::file_hash(&plan.root.join(PENDING))? != plan.pending_hash
    {
        return Err(error("stale_preview", "执行前系统集成/部署记录已变化。"));
    }
    for (path, expected) in &plan.guarded_files {
        if &deploy::file_hash(path)? != expected {
            return Err(error(
                "stale_preview",
                "启动入口或安装文件已改变，请重新预览。",
            ));
        }
    }
    if plan.preview.action == "recover" {
        rollback(&plan)?;
        return status_with(&plan.root, &plan.registry);
    }
    for change in &plan.journal.changes {
        if plan.registry.read(&change.key, &change.name)? != change.before {
            return Err(error("stale_preview", "预览后系统集成值已改变。"));
        }
    }
    validate_pending(&plan)?;
    deploy::atomic_write(
        &plan.root.join(PENDING),
        &serde_json::to_vec(&plan.journal)
            .map_err(|_| error("integration_state", "无法生成恢复记录。"))?,
    )?;
    let applied = (|| {
        for change in &plan.journal.changes {
            if plan.registry.read(&change.key, &change.name)? != change.before {
                return Err(error("stale_preview", "写入前注册项已改变。"));
            }
            plan.registry
                .write(&change.key, &change.name, change.after.as_ref())?;
            if plan.registry.read(&change.key, &change.name)? != change.after {
                return Err(error("registry_verify", "注册项写入验证失败。"));
            }
        }
        record(&plan.root, plan.journal.new_record.as_deref())?;
        std::fs::remove_file(plan.root.join(PENDING))
            .map_err(|_| error("integration_state", "系统集成恢复记录未能清理。"))?;
        Ok(())
    })();
    if let Err(e) = applied {
        if rollback(&plan).is_err() {
            return Err(error(
                "integration_recovery",
                "系统集成失败且恢复未完成，请使用恢复入口。",
            ));
        }
        return Err(e);
    }
    status_with(&plan.root, &plan.registry)
}
#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;
    #[test]
    fn association_preview_stale_values_external_changes_and_recovery_are_isolated() {
        let root = std::env::temp_dir().join(format!("mpvkit-registry-{}", deploy::operation_id()));
        deploy::execute(deploy::prepare_install(&root, vec![], false, Default::default()).unwrap())
            .unwrap();
        exercise(&root, Registry::test(&root));
        std::fs::remove_dir_all(root).unwrap();
    }
    fn exercise(root: &Path, registry: Registry) {
        let plan = prepare_with(root, "register", registry.clone()).unwrap();
        assert_eq!(plan.preview.values.len(), 15);
        assert!(!root.join(RECORD).exists());
        execute(plan).unwrap();
        assert!(status(root).unwrap().registered);
        let key = "Software\\RegisteredApplications";
        let unchanged = prepare_with(root, "register", registry.clone()).unwrap();
        assert!(unchanged.preview.values.is_empty());
        let stale = prepare_with(root, "unregister", registry.clone()).unwrap();
        registry
            .write(key, "MpvKit", Some(&RegistryData::string("other app")))
            .unwrap();
        assert_eq!(execute(unchanged).err().unwrap().code, "stale_preview");
        assert_eq!(execute(stale).err().unwrap().code, "stale_preview");
        let plan = prepare_with(root, "unregister", registry.clone()).unwrap();
        assert_eq!(plan.preview.skipped.len(), 1);
        execute(plan).unwrap();
        assert_eq!(
            registry.read(key, "MpvKit").unwrap(),
            Some(RegistryData::string("other app"))
        );
        assert!(!status(root).unwrap().registered);
        let plan = prepare_with(root, "register", registry.clone()).unwrap();
        assert_eq!(plan.preview.conflicts.len(), 1);
        registry.write(key, "MpvKit", None).unwrap();
        let plan = prepare_with(root, "register", registry.clone()).unwrap();
        deploy::atomic_write(
            &root.join(PENDING),
            &serde_json::to_vec(&plan.journal).unwrap(),
        )
        .unwrap();
        let change = &plan.journal.changes[0];
        registry
            .write(&change.key, &change.name, change.after.as_ref())
            .unwrap();
        assert!(status(root).unwrap().recovery_required);
        execute(prepare_with(root, "recover", registry.clone()).unwrap()).unwrap();
        assert_eq!(registry.read(&change.key, &change.name).unwrap(), None);
        assert!(!status(root).unwrap().recovery_required);
    }
    #[test]
    #[ignore = "requires Windows Offreg; uses only a private file-backed application hive"]
    fn native_registry_api_against_private_hive_preserves_changes_and_recovers() {
        let root =
            std::env::temp_dir().join(format!("mpvkit-native-hive-{}", deploy::operation_id()));
        deploy::execute(deploy::prepare_install(&root, vec![], false, Default::default()).unwrap())
            .unwrap();
        let registry = Registry::offline(&root).unwrap();
        exercise(&root, registry.clone());
        assert!(
            registry
                .write("Software\\Classes\\Unrelated", "", None)
                .is_err()
        );
        drop(registry);
        let reopened = Registry::offline(&root).unwrap();
        assert_eq!(
            reopened
                .read("Software\\RegisteredApplications", "MpvKit")
                .unwrap(),
            None
        );
        drop(reopened);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(all(test, target_os = "linux"))]
pub(super) fn test_interrupt(plan: &Plan) {
    deploy::atomic_write(
        &plan.root.join(PENDING),
        &serde_json::to_vec(&plan.journal).unwrap(),
    )
    .unwrap();
    for change in &plan.journal.changes {
        plan.registry
            .write(&change.key, &change.name, change.after.as_ref())
            .unwrap();
    }
}
