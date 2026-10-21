use crate::{
    error::{AppError, Result},
    platform,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const OWNER: &str = "MpvKit";
const CONTROL: &str = ".mpvkit";
const MAX_FILES: usize = 2048;
const MAX_FILE: u64 = 512 * 1024 * 1024;
static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn error(code: &'static str, message: &'static str) -> AppError {
    AppError::new(code, message, "deploy")
}

pub fn operation_id() -> String {
    format!(
        "{:x}-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        std::process::id(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    )
}

pub fn validate_relative(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 220
        || value.contains(['\\', ':', '<', '>', '"', '|', '?', '*'])
        || value.chars().any(|c| c.is_control())
    {
        return Err(error("unsafe_relative_path", "文件相对路径不安全。"));
    }
    for part in value.split('/') {
        let stem = part
            .split('.')
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();
        let device = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || ["COM", "LPT"].iter().any(|prefix| {
                stem.strip_prefix(prefix).is_some_and(|s| {
                    ["1", "2", "3", "4", "5", "6", "7", "8", "9", "¹", "²", "³"].contains(&s)
                })
            });
        if part.is_empty() || part == "." || part == ".." || part.ends_with(['.', ' ']) || device {
            return Err(error("unsafe_relative_path", "文件相对路径不安全。"));
        }
    }
    Ok(())
}

fn target(root: &Path, relative: &str) -> Result<PathBuf> {
    validate_relative(relative)?;
    if relative
        .split('/')
        .next()
        .unwrap()
        .eq_ignore_ascii_case(CONTROL)
    {
        return Err(error("reserved_path", "组件不能写入内部操作记录目录。"));
    }
    let path = root.join(relative);
    platform::validate_local_path(&path)?;
    Ok(path)
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn file_hash(path: &Path) -> Result<Option<String>> {
    platform::validate_local_path(path)?;
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(error("file_unavailable", "文件不可读取或被占用。")),
    };
    let metadata = file
        .metadata()
        .map_err(|_| error("file_unavailable", "文件状态不可读取。"))?;
    if !metadata.is_file() || metadata.len() > MAX_FILE {
        return Err(error(
            "file_limit",
            "目标不是普通文件或超出单文件大小限制。",
        ));
    }
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    let mut count = 0u64;
    loop {
        let size = file
            .read(&mut buffer)
            .map_err(|_| error("file_unavailable", "读取文件失败。"))?;
        if size == 0 {
            break;
        }
        count += size as u64;
        if count > MAX_FILE {
            return Err(error("file_limit", "文件超出大小限制。"));
        }
        hash.update(&buffer[..size]);
    }
    Ok(Some(format!("{:x}", hash.finalize())))
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OwnedFile {
    pub path: String,
    pub sha256: String,
    pub original_backup: Option<String>,
    pub original_sha256: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstallManifest {
    pub schema_version: u32,
    pub owner: String,
    pub installation_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<String>,
    #[serde(default = "default_config_directory")]
    pub config_directory: String,
    pub files: Vec<OwnedFile>,
    pub external_mpv: Option<PathBuf>,
    pub components: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Journal {
    schema_version: u32,
    owner: String,
    id: String,
    old_manifest: Option<String>,
    new_manifest: Option<String>,
    previous_before: Option<String>,
    changes: Vec<Change>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Change {
    pub path: String,
    pub before: Option<String>,
    pub after: Option<String>,
    pub backup: Option<String>,
}

pub struct Payload {
    pub path: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentPreview {
    pub id: String,
    pub root: PathBuf,
    pub changes: Vec<Change>,
    pub conflicts: Vec<String>,
    pub no_changes: bool,
}

pub struct DeploymentPlan {
    pub preview: DeploymentPreview,
    created: Instant,
    journal: Journal,
    payloads: Vec<Option<Vec<u8>>>,
    manifest_before: Option<String>,
    previous_bytes: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentStatus {
    pub root: PathBuf,
    pub manifest: Option<InstallManifest>,
    pub recovery_required: bool,
    pub previous_available: bool,
}

fn control(root: &Path, name: &str) -> Result<PathBuf> {
    validate_relative(name)?;
    let path = root.join(CONTROL).join(name);
    platform::validate_local_path(&path)?;
    Ok(path)
}

fn read_optional(path: &Path) -> Result<Option<String>> {
    match file_hash(path)? {
        None => Ok(None),
        Some(_) => {
            let file = File::open(path).map_err(|_| error("state_read", "无法读取操作记录。"))?;
            let mut bytes = Vec::new();
            file.take(2 * 1024 * 1024 + 1)
                .read_to_end(&mut bytes)
                .map_err(|_| error("state_read", "无法读取操作记录。"))?;
            if bytes.len() > 2 * 1024 * 1024 {
                return Err(error("state_limit", "操作记录过大，已保留原文件。"));
            }
            String::from_utf8(bytes)
                .map(Some)
                .map_err(|_| error("invalid_state", "操作记录不是 UTF-8，已保留。"))
        }
    }
}

fn decode<T: serde::de::DeserializeOwned>(text: &str) -> Result<T> {
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|_| error("invalid_state", "操作记录损坏，已保留原文件。"))?;
    if value["schemaVersion"] != 1 || value["owner"] != OWNER {
        return Err(error(
            "incompatible_state",
            "未知版本或归属的记录，已保留原文件。",
        ));
    }
    serde_json::from_value(value)
        .map_err(|_| error("invalid_state", "操作记录字段不受支持，已保留。"))
}

fn check_owner(root: &Path) -> Result<()> {
    platform::validate_local_path(root)?;
    let directory = root.join(CONTROL);
    platform::validate_local_path(&directory)?;
    if directory.exists() {
        let value = read_optional(&control(root, "owner.json")?)?.ok_or_else(|| {
            error(
                "ownership_unknown",
                "内部目录没有所有权记录，请保留并检查。",
            )
        })?;
        let _: serde_json::Value = decode(&value)?;
    }
    Ok(())
}

fn manifest_text(root: &Path) -> Result<Option<String>> {
    check_owner(root)?;
    let text = read_optional(&control(root, "install.json")?)?;
    if let Some(text) = &text {
        validate_manifest(&decode(text)?)?;
    }
    Ok(text)
}

pub fn default_config_directory() -> String {
    "config".into()
}

fn validate_manifest(manifest: &InstallManifest) -> Result<()> {
    if !matches!(manifest.config_directory.as_str(), "config" | ".") {
        return Err(error("invalid_state", "配置目录布局不受支持。"));
    }
    if manifest.files.len() > MAX_FILES {
        return Err(error("state_limit", "安装记录包含过多文件。"));
    }
    let mut paths = BTreeSet::new();
    for entry in &manifest.files {
        validate_relative(&entry.path)?;
        if entry
            .path
            .split('/')
            .next()
            .unwrap()
            .eq_ignore_ascii_case(CONTROL)
            || !paths.insert(entry.path.to_ascii_lowercase())
            || !valid_hash(&entry.sha256)
        {
            return Err(error("invalid_state", "安装记录包含冲突路径或无效摘要。"));
        }
        match (&entry.original_backup, &entry.original_sha256) {
            (Some(path), Some(hash)) if valid_hash(hash) => validate_backup(path)?,
            (None, None) => {}
            _ => return Err(error("invalid_state", "原件备份缺少有效摘要。")),
        }
    }
    Ok(())
}

fn valid_hash(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn validate_backup(path: &str) -> Result<()> {
    validate_relative(path)?;
    let parts: Vec<_> = path.split('/').collect();
    if parts.len() != 3
        || parts[0] != "backups"
        || !parts[1].bytes().all(|c| c.is_ascii_hexdigit() || c == b'-')
        || parts[1].is_empty()
        || parts[2].parse::<usize>().is_err()
    {
        return Err(error("invalid_state", "备份引用越出已记录操作范围。"));
    }
    Ok(())
}

pub fn status(root: &Path) -> Result<DeploymentStatus> {
    let manifest = manifest_text(root)?.map(|s| decode(&s)).transpose()?;
    Ok(DeploymentStatus {
        root: root.into(),
        manifest,
        recovery_required: read_optional(&control(root, "pending.json")?)?.is_some(),
        previous_available: read_optional(&control(root, "previous.json")?)?.is_some(),
    })
}

pub fn prepare_install(
    root: &Path,
    payloads: Vec<Payload>,
    replace_existing: bool,
    external_mpv: Option<PathBuf>,
    components: BTreeMap<String, String>,
) -> Result<DeploymentPlan> {
    let layout = status(root)?
        .manifest
        .map(|m| m.config_directory)
        .unwrap_or_else(default_config_directory);
    prepare_install_at(
        root,
        payloads,
        replace_existing,
        external_mpv,
        components,
        &layout,
    )
}

pub fn prepare_install_at(
    root: &Path,
    payloads: Vec<Payload>,
    replace_existing: bool,
    external_mpv: Option<PathBuf>,
    components: BTreeMap<String, String>,
    config_directory: &str,
) -> Result<DeploymentPlan> {
    if !matches!(config_directory, "config" | ".") {
        return Err(error("invalid_layout", "配置目录布局不受支持。"));
    }
    let old_text = manifest_text(root)?;
    if read_optional(&control(root, "association-pending.json")?)?.is_some() {
        return Err(error("recovery_required", "请先恢复未完成的系统集成操作。"));
    }
    if status(root)?.recovery_required {
        return Err(error("recovery_required", "有未完成操作，请先预览恢复。"));
    }
    if payloads.len() > MAX_FILES {
        return Err(error("file_limit", "组件文件过多。"));
    }
    if let Some(path) = &external_mpv {
        platform::validate_local_path(path)?;
    }
    let old: Option<InstallManifest> = old_text.as_deref().map(decode).transpose()?;
    if old
        .as_ref()
        .is_some_and(|m| m.config_directory != config_directory)
    {
        return Err(error(
            "layout_changed",
            "请先移除当前环境，再改变目录布局。",
        ));
    }
    let owned: BTreeMap<_, _> = old
        .as_ref()
        .map(|m| {
            m.files
                .iter()
                .map(|f| (f.path.to_ascii_lowercase(), f))
                .collect()
        })
        .unwrap_or_default();
    let id = operation_id();
    let mut changes = Vec::new();
    let mut data = Vec::new();
    let mut files = Vec::new();
    let mut conflicts = Vec::new();
    let mut seen = BTreeSet::new();
    let mut total = 0u64;
    for payload in payloads {
        let path = target(root, &payload.path)?;
        if [
            external_mpv.as_ref(),
            old.as_ref().and_then(|m| m.external_mpv.as_ref()),
        ]
        .into_iter()
        .flatten()
        .any(|external| {
            external
                .to_string_lossy()
                .eq_ignore_ascii_case(&path.to_string_lossy())
        }) {
            return Err(error(
                "external_player_conflict",
                "外部播放器不能变成待覆盖或卸载的受管理文件。",
            ));
        }
        let key = payload.path.to_ascii_lowercase();
        if !seen.insert(key.clone()) {
            return Err(error("duplicate_path", "组件文件路径重复。"));
        }
        total += payload.bytes.len() as u64;
        if payload.bytes.len() as u64 > MAX_FILE || total > 1024 * 1024 * 1024 {
            return Err(error("file_limit", "组件展开大小超过限制。"));
        }
        let before = file_hash(&path)?;
        let after = hash_bytes(&payload.bytes);
        let previous = owned.get(&key).copied();
        if previous
            .is_some_and(|entry| before.is_some() && before.as_deref() != Some(&entry.sha256))
            || (previous.is_none() && before.is_some() && !replace_existing)
        {
            conflicts.push(payload.path.clone());
        }
        let backup = before
            .as_ref()
            .map(|_| format!("backups/{id}/{}", changes.len()));
        let original_backup = previous
            .map(|e| e.original_backup.clone())
            .unwrap_or_else(|| backup.clone());
        let original_sha256 = previous
            .map(|e| e.original_sha256.clone())
            .unwrap_or_else(|| before.clone());
        files.push(OwnedFile {
            path: payload.path.clone(),
            sha256: after.clone(),
            original_backup,
            original_sha256,
        });
        if before.as_deref() != Some(&after) || previous.is_none() {
            changes.push(Change {
                path: payload.path,
                before,
                after: Some(after),
                backup,
            });
            data.push(Some(payload.bytes));
        }
    }
    // A complete desired set: omitted managed files restore their original bytes.
    for previous in owned
        .values()
        .filter(|f| !seen.contains(&f.path.to_ascii_lowercase()))
    {
        let before = file_hash(&target(root, &previous.path)?)?;
        if before.as_deref() != Some(&previous.sha256) {
            conflicts.push(previous.path.clone());
        }
        let bytes = previous
            .original_backup
            .as_ref()
            .map(|b| read_backup(root, b))
            .transpose()?;
        let after = bytes.as_ref().map(|b| hash_bytes(b));
        if after != previous.original_sha256 {
            return Err(error("backup_changed", "原件备份摘要不匹配。"));
        }
        let backup = before
            .as_ref()
            .map(|_| format!("backups/{id}/{}", changes.len()));
        changes.push(Change {
            path: previous.path.clone(),
            before,
            after,
            backup,
        });
        data.push(bytes);
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let manifest = InstallManifest {
        schema_version: 1,
        owner: OWNER.into(),
        installation_id: id.clone(),
        environment_id: Some(
            old.as_ref()
                .map(|m| crate::setup::environment(m).to_string())
                .unwrap_or_else(|| id.clone()),
        ),
        files,
        config_directory: config_directory.into(),
        external_mpv,
        components,
    };
    conflicts.extend(script_conflicts(root, &manifest)?);
    let no_changes = changes.is_empty()
        && old.as_ref().is_some_and(|m| {
            m.files == manifest.files
                && m.external_mpv == manifest.external_mpv
                && m.components == manifest.components
        });
    let new_text = serde_json::to_string_pretty(&manifest)
        .map_err(|_| error("state_write", "无法生成安装记录。"))?;
    make_plan(
        root,
        Journal {
            schema_version: 1,
            owner: OWNER.into(),
            id,
            changes,
            old_manifest: old_text,
            new_manifest: Some(new_text),
            previous_before: None,
        },
        conflicts,
        data,
        no_changes,
    )
}

pub(crate) fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    platform::validate_local_path(path)?;
    let file = File::open(path).map_err(|_| error("file_unavailable", "文件无法读取。"))?;
    if !file
        .metadata()
        .map_err(|_| error("file_unavailable", "文件状态不可读取。"))?
        .is_file()
    {
        return Err(error("file_limit", "目标不是普通文件。"));
    }
    let mut bytes = Vec::new();
    file.take(MAX_FILE + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| error("file_unavailable", "文件读取失败。"))?;
    if bytes.len() as u64 > MAX_FILE {
        return Err(error("file_limit", "文件超出大小限制。"));
    }
    Ok(bytes)
}

fn read_backup(root: &Path, relative: &str) -> Result<Vec<u8>> {
    validate_backup(relative)?;
    let path = control(root, relative)?;
    let hash = file_hash(&path)?;
    if hash.is_none() {
        return Err(error(
            "backup_missing",
            "恢复所需备份缺失，已保留当前文件。",
        ));
    }
    read_bytes(&path)
}

fn make_plan(
    root: &Path,
    mut journal: Journal,
    conflicts: Vec<String>,
    payloads: Vec<Option<Vec<u8>>>,
    no_changes: bool,
) -> Result<DeploymentPlan> {
    let previous_bytes = read_optional(&control(root, "previous.json")?)?;
    if let Some(text) = &previous_bytes {
        validate_journal(&decode(text)?)?;
    }
    journal.previous_before = previous_bytes.as_ref().map(|s| hash_bytes(s.as_bytes()));
    Ok(DeploymentPlan {
        manifest_before: journal
            .old_manifest
            .as_ref()
            .map(|s| hash_bytes(s.as_bytes())),
        preview: DeploymentPreview {
            id: journal.id.clone(),
            root: root.into(),
            changes: journal.changes.clone(),
            conflicts,
            no_changes,
        },
        created: Instant::now(),
        journal,
        payloads,
        previous_bytes,
    })
}

pub fn prepare_uninstall(root: &Path) -> Result<DeploymentPlan> {
    if read_optional(&control(root, "associations.json")?)?.is_some() {
        return Err(error(
            "integration_registered",
            "请先在系统集成区预览并移除打开方式登记，再移除播放器环境。",
        ));
    }
    let mut plan = prepare_install(root, vec![], false, None, BTreeMap::new())?;
    plan.journal.new_manifest = None;
    plan.preview.no_changes = plan.journal.old_manifest.is_none();
    Ok(plan)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    platform::validate_local_path(path)?;
    let parent = path
        .parent()
        .ok_or_else(|| error("unsafe_path", "文件没有父目录。"))?;
    fs::create_dir_all(parent).map_err(|_| error("write_failed", "无法创建目标目录。"))?;
    platform::validate_local_path(path)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| error("write_failed", "无法新建文件，可能已存在或被占用。"))?;
    #[cfg(test)]
    let ready = tests::inject_write_failure(path, &mut file, bytes);
    #[cfg(not(test))]
    let ready: std::io::Result<()> = Ok(());
    let written = ready
        .and_then(|_| file.write_all(bytes))
        .and_then(|_| file.sync_all());
    if written.is_err() {
        // Only this create_new call owns the incomplete file; an open failure must not delete it.
        drop(file);
        let _ = fs::remove_file(path);
    }
    written.map_err(|_| error("write_failed", "文件写入失败，请检查空间和权限。"))
}

pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    platform::validate_local_path(path)?;
    let temporary = path.with_file_name(format!(".mpvkit-{}.tmp", operation_id()));
    write_new(&temporary, bytes)?;
    let result = platform::replace_file(&temporary, path);
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn set_content(path: &Path, bytes: Option<&[u8]>) -> Result<()> {
    platform::validate_local_path(path)?;
    match bytes {
        Some(bytes) => atomic_write(path, bytes),
        None => match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err(error("remove_failed", "文件无法移除，可能被占用。")),
        },
    }
}

pub(crate) fn acquire(root: &Path) -> Result<File> {
    check_owner(root)?;
    fs::create_dir_all(root).map_err(|_| error("write_failed", "无法创建安装目标。"))?;
    let directory = root.join(CONTROL);
    match fs::create_dir(&directory) {
        Ok(()) => {
            if let Err(failure) = write_new(
                &control(root, "owner.json")?,
                br#"{"schemaVersion":1,"owner":"MpvKit"}"#,
            ) {
                // This call created it; remove only while empty, never a pre-existing directory.
                let _ = fs::remove_dir(&directory);
                return Err(failure);
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => check_owner(root)?,
        Err(_) => return Err(error("write_failed", "无法创建操作目录。")),
    }
    let lock_path = control(root, "operation.lock")?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path)
        .map_err(|_| error("target_locked", "无法打开安装目标的操作锁。"))?;
    file.try_lock()
        .map_err(|_| error("target_locked", "另一个进程正在操作此安装目标。"))?;
    Ok(file)
}

fn validate_plan(plan: &DeploymentPlan) -> Result<()> {
    if let Some(text) = &plan.journal.new_manifest {
        let manifest: InstallManifest = decode(text)?;
        if !script_conflicts(&plan.preview.root, &manifest)?.is_empty() {
            return Err(error(
                "script_conflict",
                "自动脚本目录出现未管理内容，请保留并使用独立目录，或先手动移出冲突脚本。",
            ));
        }
    }
    if read_optional(&control(&plan.preview.root, "association-pending.json")?)?.is_some() {
        return Err(error("recovery_required", "请先恢复系统集成操作。"));
    }
    if plan.journal.new_manifest.is_none()
        && read_optional(&control(&plan.preview.root, "associations.json")?)?.is_some()
    {
        return Err(error(
            "integration_registered",
            "请先移除打开方式登记，再移除或恢复为空环境。",
        ));
    }
    if plan.created.elapsed() > Duration::from_secs(300) {
        return Err(error("stale_preview", "部署预览已过期，请重新预览。"));
    }
    if !plan.preview.conflicts.is_empty() {
        return Err(error(
            "file_conflict",
            "发现用户文件或修改，请保留并处理冲突。",
        ));
    }
    if file_hash(&control(&plan.preview.root, "install.json")?)? != plan.manifest_before {
        return Err(error("stale_preview", "安装记录已变化，请重新预览。"));
    }
    if file_hash(&control(&plan.preview.root, "previous.json")?)? != plan.journal.previous_before {
        return Err(error("stale_preview", "上一成功记录已变化，请重新预览。"));
    }
    for change in &plan.journal.changes {
        if file_hash(&target(&plan.preview.root, &change.path)?)? != change.before {
            return Err(error("stale_preview", "预览后目标文件已变化，请重新预览。"));
        }
    }
    Ok(())
}

fn begin(plan: &DeploymentPlan) -> Result<()> {
    let root = &plan.preview.root;
    let bytes = serde_json::to_vec_pretty(&plan.journal)
        .map_err(|_| error("state_write", "无法生成恢复记录。"))?;
    // The durable journal precedes staging, backups, and all target mutations.
    atomic_write(&control(root, "pending.json")?, &bytes)?;
    cleanup::record(root, &plan.journal)?;
    if let Some(previous) = &plan.previous_bytes {
        write_new(
            &control(root, &format!("backups/{}/previous.json", plan.journal.id))?,
            previous.as_bytes(),
        )?;
    }
    for (index, change) in plan.journal.changes.iter().enumerate() {
        if let Some(relative) = &change.backup {
            let source = target(root, &change.path)?;
            let backup = control(root, relative)?;
            let data = read_bytes(&source)?;
            if Some(hash_bytes(&data)) != change.before {
                return Err(error("stale_preview", "备份前文件已变化。"));
            }
            write_new(&backup, &data)?;
        }
        if let Some(bytes) = &plan.payloads[index] {
            let staged = control(root, &format!("staging/{}/{index}", plan.journal.id))?;
            write_new(&staged, bytes)?;
            if file_hash(&staged)? != change.after {
                return Err(error("stage_failed", "暂存文件校验失败。"));
            }
        }
    }
    Ok(())
}

fn apply_change(root: &Path, journal: &Journal, index: usize) -> Result<()> {
    let change = &journal.changes[index];
    if file_hash(&target(root, &change.path)?)? != change.before {
        return Err(error("stale_preview", "执行前文件状态已变化，已停止覆盖。"));
    }
    let bytes = change
        .after
        .as_ref()
        .map(|_| {
            let path = control(root, &format!("staging/{}/{index}", journal.id))?;
            let bytes = read_bytes(&path)?;
            if Some(hash_bytes(&bytes)) != change.after {
                return Err(error("stage_failed", "暂存文件已变化。"));
            }
            Ok(bytes)
        })
        .transpose()?;
    set_content(&target(root, &change.path)?, bytes.as_deref())?;
    if file_hash(&target(root, &change.path)?)? != change.after {
        return Err(error("verify_failed", "部署后文件校验失败。"));
    }
    Ok(())
}

impl DeploymentPlan {
    pub(crate) fn config_directory(&self) -> Result<String> {
        self.journal
            .new_manifest
            .as_ref()
            .or(self.journal.old_manifest.as_ref())
            .map(|text| decode::<InstallManifest>(text).map(|m| m.config_directory))
            .unwrap_or_else(|| Ok(default_config_directory()))
    }
}

pub fn execute(plan: DeploymentPlan) -> Result<DeploymentStatus> {
    validate_plan(&plan)?;
    let root = &plan.preview.root;
    let _lock = acquire(root)?;
    if status(root)?.recovery_required {
        return Err(error("recovery_required", "请先恢复未完成操作。"));
    }
    validate_plan(&plan)?;
    if plan.preview.no_changes {
        return status(root);
    }
    // Output availability may change after preview. Check before any transaction write.
    for (change, payload) in plan.journal.changes.iter().zip(&plan.payloads) {
        if change.path == "state/settings.json"
            && let Some(bytes) = payload
            && let Ok(settings) = serde_json::from_slice::<crate::config::Settings>(bytes)
        {
            crate::config::validate_output(&settings)?;
        }
    }
    let applied = (|| {
        begin(&plan)?;
        for index in 0..plan.journal.changes.len() {
            apply_change(root, &plan.journal, index)?;
        }
        set_content(
            &control(root, "install.json")?,
            plan.journal.new_manifest.as_deref().map(str::as_bytes),
        )?;
        let bytes = serde_json::to_vec_pretty(&plan.journal)
            .map_err(|_| error("state_write", "无法保留上一成功版本。"))?;
        atomic_write(&control(root, "previous.json")?, &bytes)?;
        fs::remove_file(control(root, "pending.json")?)
            .map_err(|_| error("state_write", "完成记录无法落盘，请恢复确认。"))?;
        status(root)
    })();
    match applied {
        Ok(status) => Ok(status),
        Err(original) => {
            if read_optional(&control(root, "pending.json")?)?.is_some()
                && rollback(root, &plan.journal).is_err()
            {
                return Err(error(
                    "recovery_required",
                    "操作失败且未能完整恢复，备份与记录已保留。",
                ));
            }
            Err(original)
        }
    }
}

fn validate_journal(journal: &Journal) -> Result<()> {
    if journal.changes.len() > MAX_FILES
        || journal.id.is_empty()
        || !journal
            .id
            .bytes()
            .all(|b| b.is_ascii_hexdigit() || b == b'-')
    {
        return Err(error("invalid_state", "恢复记录的操作范围无效。"));
    }
    let mut paths = BTreeSet::new();
    for (index, change) in journal.changes.iter().enumerate() {
        validate_relative(&change.path)?;
        if !paths.insert(change.path.to_ascii_lowercase())
            || change.before.as_ref().is_some_and(|h| !valid_hash(h))
            || change.after.as_ref().is_some_and(|h| !valid_hash(h))
            || change.backup
                != change
                    .before
                    .as_ref()
                    .map(|_| format!("backups/{}/{index}", journal.id))
        {
            return Err(error("invalid_state", "恢复记录的路径或摘要无效。"));
        }
    }
    for text in [&journal.old_manifest, &journal.new_manifest]
        .into_iter()
        .flatten()
    {
        validate_manifest(&decode(text)?)?;
    }
    Ok(())
}

fn recovery_conflicts(root: &Path, journal: &Journal) -> Result<Vec<String>> {
    validate_journal(journal)?;
    let mut conflicts = vec![];
    for change in &journal.changes {
        let current = file_hash(&target(root, &change.path)?)?;
        if current != change.before && current != change.after {
            conflicts.push(change.path.clone());
        }
        if current != change.before
            && let Some(relative) = &change.backup
            && Some(hash_bytes(&read_backup(root, relative)?)) != change.before
        {
            return Err(error("backup_changed", "备份摘要不匹配，已停止恢复。"));
        }
    }
    let previous_path = control(root, "previous.json")?;
    let current_previous = file_hash(&previous_path)?;
    let new_previous = Some(hash_bytes(
        &serde_json::to_vec_pretty(journal)
            .map_err(|_| error("invalid_state", "恢复记录无法编码。"))?,
    ));
    if current_previous != journal.previous_before && current_previous != new_previous {
        conflicts.push("上一成功记录".into());
    }
    if current_previous == new_previous && journal.previous_before.is_some() {
        let backup = control(root, &format!("backups/{}/previous.json", journal.id))?;
        if file_hash(&backup)? != journal.previous_before {
            return Err(error("backup_changed", "上一成功记录的备份缺失或变化。"));
        }
    }
    let current = read_optional(&control(root, "install.json")?)?;
    if current != journal.old_manifest && current != journal.new_manifest {
        conflicts.push("安装记录".into());
    }
    Ok(conflicts)
}

fn rollback(root: &Path, journal: &Journal) -> Result<()> {
    if !recovery_conflicts(root, journal)?.is_empty() {
        return Err(error(
            "recovery_conflict",
            "恢复目标有新的用户修改，未覆盖。",
        ));
    }
    for change in journal.changes.iter().rev() {
        let path = target(root, &change.path)?;
        if file_hash(&path)? == change.before {
            continue;
        }
        let bytes = change
            .backup
            .as_ref()
            .map(|b| read_backup(root, b))
            .transpose()?;
        // Recheck immediately before each restore; rollback is itself restartable.
        if file_hash(&path)? != change.after {
            return Err(error("recovery_conflict", "恢复时文件状态发生变化。"));
        }
        set_content(&path, bytes.as_deref())?;
    }
    set_content(
        &control(root, "install.json")?,
        journal.old_manifest.as_deref().map(str::as_bytes),
    )?;
    let previous_path = control(root, "previous.json")?;
    if file_hash(&previous_path)? != journal.previous_before {
        let backup = control(root, &format!("backups/{}/previous.json", journal.id))?;
        let bytes = if journal.previous_before.is_some() {
            read_optional(&backup)?
        } else {
            None
        };
        set_content(&previous_path, bytes.as_deref().map(str::as_bytes))?;
    }
    fs::remove_file(control(root, "pending.json")?)
        .map_err(|_| error("state_write", "恢复完成但记录无法清理。"))?;
    Ok(())
}

pub struct RecoveryPlan {
    pub preview: DeploymentPreview,
    journal: Journal,
    journal_hash: String,
    current: Vec<Option<String>>,
    manifest_hash: Option<String>,
    created: Instant,
}

pub fn prepare_recovery(root: &Path) -> Result<RecoveryPlan> {
    check_owner(root)?;
    let text = read_optional(&control(root, "pending.json")?)?
        .ok_or_else(|| error("nothing_to_recover", "没有待恢复操作。"))?;
    let journal: Journal = decode(&text)?;
    let conflicts = recovery_conflicts(root, &journal)?;
    let current = journal
        .changes
        .iter()
        .map(|c| file_hash(&target(root, &c.path)?))
        .collect::<Result<_>>()?;
    Ok(RecoveryPlan {
        preview: DeploymentPreview {
            id: operation_id(),
            root: root.into(),
            changes: journal
                .changes
                .iter()
                .map(|c| Change {
                    path: c.path.clone(),
                    before: c.after.clone(),
                    after: c.before.clone(),
                    backup: c.backup.clone(),
                })
                .collect(),
            conflicts,
            no_changes: false,
        },
        journal,
        journal_hash: hash_bytes(text.as_bytes()),
        current,
        manifest_hash: file_hash(&control(root, "install.json")?)?,
        created: Instant::now(),
    })
}

pub fn recover(plan: RecoveryPlan) -> Result<DeploymentStatus> {
    let root = &plan.preview.root;
    let _lock = acquire(root)?;
    if plan.created.elapsed() > Duration::from_secs(300)
        || file_hash(&control(root, "pending.json")?)? != Some(plan.journal_hash)
        || file_hash(&control(root, "install.json")?)? != plan.manifest_hash
    {
        return Err(error("stale_preview", "恢复预览已变化或过期。"));
    }
    for (change, expected) in plan.journal.changes.iter().zip(plan.current) {
        if file_hash(&target(root, &change.path)?)? != expected {
            return Err(error("stale_preview", "恢复预览后文件已变化。"));
        }
    }
    rollback(root, &plan.journal)?;
    status(root)
}

pub fn prepare_restore(root: &Path) -> Result<DeploymentPlan> {
    let text = read_optional(&control(root, "previous.json")?)?
        .ok_or_else(|| error("no_previous_install", "没有上一成功操作可恢复。"))?;
    let previous: Journal = decode(&text)?;
    validate_journal(&previous)?;
    let current = manifest_text(root)?;
    if current != previous.new_manifest || status(root)?.recovery_required {
        return Err(error(
            "restore_conflict",
            "当前安装与上一成功记录不一致，请先处理恢复。",
        ));
    }
    let id = operation_id();
    let mut changes = vec![];
    let mut data = vec![];
    let mut conflicts = vec![];
    for change in &previous.changes {
        let before = file_hash(&target(root, &change.path)?)?;
        if before != change.after {
            conflicts.push(change.path.clone());
        }
        let bytes = change
            .backup
            .as_ref()
            .map(|b| read_backup(root, b))
            .transpose()?;
        if bytes.as_ref().map(|b| hash_bytes(b)) != change.before {
            return Err(error("backup_changed", "上一版本备份已变化。"));
        }
        let backup = before
            .as_ref()
            .map(|_| format!("backups/{id}/{}", changes.len()));
        changes.push(Change {
            path: change.path.clone(),
            before,
            after: change.before.clone(),
            backup,
        });
        data.push(bytes);
    }
    make_plan(
        root,
        Journal {
            schema_version: 1,
            owner: OWNER.into(),
            id,
            changes,
            old_manifest: current,
            new_manifest: previous.old_manifest,
            previous_before: None,
        },
        conflicts,
        data,
        false,
    )
}

#[cfg(test)]
mod tests;

pub(crate) fn script_conflicts(root: &Path, manifest: &InstallManifest) -> Result<Vec<String>> {
    if !manifest.components.contains_key("uosc") {
        return Ok(vec![]);
    }
    let prefix = if manifest.config_directory == "." {
        "scripts/"
    } else {
        "config/scripts/"
    };
    let directory = root.join(&manifest.config_directory).join("scripts");
    platform::validate_local_path(&directory)?;
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(_) => return Err(error("script_conflict", "自动脚本目录不可读取。")),
    };
    let allowed: BTreeSet<String> = manifest
        .files
        .iter()
        .filter_map(|f| f.path.strip_prefix(prefix))
        .filter_map(|p| p.split('/').next())
        .map(str::to_ascii_lowercase)
        .collect();
    let mut conflicts = Vec::new();
    for (index, entry) in entries.enumerate() {
        if index >= MAX_FILES {
            return Err(error(
                "file_limit",
                "自动脚本目录条目过多，请使用独立目标。",
            ));
        }
        let entry = entry.map_err(|_| error("script_conflict", "自动脚本目录项无法读取。"))?;
        platform::validate_local_path(&entry.path())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !allowed.contains(&name.to_ascii_lowercase()) {
            conflicts.push(format!(
                "{prefix}{name}（未管理自动脚本，请先移出或使用独立目标）"
            ));
        } else if entry
            .file_type()
            .map_err(|_| error("script_conflict", "脚本类型无法读取。"))?
            .is_dir()
        {
            for main in ["main.lua", "main.js"] {
                let relative = format!("{prefix}{name}/{main}");
                if entry.path().join(main).exists()
                    && !manifest
                        .files
                        .iter()
                        .any(|f| f.path.eq_ignore_ascii_case(&relative))
                {
                    conflicts.push(relative);
                }
            }
        }
    }
    Ok(conflicts)
}

pub mod cleanup;
