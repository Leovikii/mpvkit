use crate::{
    deploy,
    error::{AppError, Result},
    platform,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Target {
    schema_version: u32,
    owner: String,
    pub root: PathBuf,
    pub config_directory: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    #[serde(flatten)]
    pub target: Target,
    pub default_root: PathBuf,
}
pub fn status(default: &Path) -> Result<Status> {
    Ok(Status {
        target: read_saved(default)?,
        default_root: default.into(),
    })
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub id: String,
    pub before: Target,
    pub after: Target,
    pub before_unavailable: bool,
}
pub struct Plan {
    pub preview: Preview,
    default: PathBuf,
    before_hash: Option<String>,
    created: Instant,
}
fn error(code: &'static str, message: &'static str) -> AppError {
    AppError::new(code, message, "target")
}
fn record(default: &Path) -> PathBuf {
    default.join(".mpvkit/target.json")
}
fn validate(value: &Target) -> Result<()> {
    if value.schema_version != 1
        || value.owner != "MpvKit"
        || !matches!(value.config_directory.as_str(), "config" | ".")
    {
        return Err(error(
            "target_state",
            "目标记录版本或归属不受支持，已保留。",
        ));
    }

    if value.root.parent().is_none() || value.root.file_name().is_none() {
        return Err(error(
            "target_root",
            "请选择具体的本地目录，不能使用磁盘根目录。",
        ));
    }
    Ok(())
}
pub fn read_saved(default: &Path) -> Result<Target> {
    deploy::status(default)?;
    let path = record(default);
    let value = if deploy::file_hash(&path)?.is_some() {
        let bytes = deploy::read_bytes(&path)?;
        if bytes.len() > 65536 {
            return Err(error("target_state", "目标记录过大，已保留。"));
        }
        serde_json::from_slice(&bytes)
            .map_err(|_| error("target_state", "目标记录损坏或含未知字段，已保留。"))?
    } else {
        Target {
            schema_version: 1,
            owner: "MpvKit".into(),
            root: default.into(),
            config_directory: "config".into(),
        }
    };
    validate(&value)?;
    Ok(value)
}
pub fn read(default: &Path) -> Result<Target> {
    let value = read_saved(default)?;
    platform::validate_local_path(&value.root)?;
    Ok(value)
}
fn movable(root: &Path) -> Result<()> {
    let state = deploy::status(root)?;
    let integration = crate::integration::status(root)?;
    if state.recovery_required || integration.registered || integration.recovery_required {
        return Err(error(
            "target_in_use",
            "切换目标前请恢复未完成操作，并移除当前打开方式登记。",
        ));
    }
    Ok(())
}
pub fn prepare(default: &Path, root: PathBuf) -> Result<Plan> {
    let before = read_saved(default)?;
    let before_unavailable = platform::validate_local_path(&before.root).is_err();
    if !before_unavailable {
        movable(&before.root)?;
    }
    platform::validate_local_path(&root)?;
    let config_directory = match deploy::status(&root)?.manifest {
        Some(manifest) => manifest.config_directory,
        None if root == before.root => before.config_directory.clone(),
        None => "config".into(),
    };
    let after = Target {
        schema_version: 1,
        owner: "MpvKit".into(),
        root,
        config_directory,
    };
    validate(&after)?;
    platform::validate_local_path(&after.root)?;
    if let Some(manifest) = deploy::status(&after.root)?.manifest
        && manifest.config_directory != after.config_directory
    {
        return Err(error(
            "layout_changed",
            "目标已有不同布局的管理环境，请先移除它或选择原布局。",
        ));
    }
    Ok(Plan {
        preview: Preview {
            id: deploy::operation_id(),
            before,
            after,
            before_unavailable,
        },
        default: default.into(),
        before_hash: deploy::file_hash(&record(default))?,
        created: Instant::now(),
    })
}
fn selection_lock(default: &Path) -> Result<File> {
    // Reuse ownership validation before opening a separate selection lock.
    let _ownership = deploy::acquire(default)?;
    let path = default.join(".mpvkit/selection.lock");
    platform::validate_local_path(&path)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(|_| error("target_locked", "无法打开目标选择锁。"))?;
    file.try_lock()
        .map_err(|_| error("target_locked", "另一个进程正在切换或使用当前目标。"))?;
    Ok(file)
}
pub fn active(root: &Path) -> Result<File> {
    let default = platform::default_root()?;
    let lock = selection_lock(&default)?;
    if read(&default)?.root != root {
        return Err(error("target_changed", "目标已切换，请重新预览。"));
    }
    Ok(lock)
}
pub fn execute(plan: Plan) -> Result<Target> {
    let _selection = selection_lock(&plan.default)?;
    if plan.created.elapsed() > Duration::from_secs(300)
        || deploy::file_hash(&record(&plan.default))? != plan.before_hash
        || read_saved(&plan.default)? != plan.preview.before
    {
        return Err(error("stale_preview", "目标预览已失效，请重新预览。"));
    }
    // An offline old root is never opened or changed. Existing registry values remain
    // untouched and will conflict with a new registration until explicitly removed.
    let unavailable = platform::validate_local_path(&plan.preview.before.root).is_err();
    if unavailable != plan.preview.before_unavailable {
        return Err(error("stale_preview", "原目录可用性已变化，请重新预览。"));
    }
    let _old = if unavailable {
        None
    } else {
        Some(deploy::acquire(&plan.preview.before.root)?)
    };
    if !unavailable {
        movable(&plan.preview.before.root)?;
    }
    validate(&plan.preview.after)?;
    platform::validate_local_path(&plan.preview.after.root)?;
    if let Some(manifest) = deploy::status(&plan.preview.after.root)?.manifest
        && manifest.config_directory != plan.preview.after.config_directory
    {
        return Err(error("layout_changed", "目标布局已改变，请重新预览。"));
    }
    let bytes = serde_json::to_vec_pretty(&plan.preview.after)
        .map_err(|_| error("target_state", "目标记录无法编码。"))?;
    deploy::atomic_write(&record(&plan.default), &bytes)?;
    Ok(plan.preview.after)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_target_is_explicit_stale_safe_and_preserves_existing_data() {
        let base = std::env::temp_dir().join(format!("mpvkit-target-{}", deploy::operation_id()));
        let default = base.join("default");
        let custom = base.join("中文 空格/portable_config");
        std::fs::create_dir_all(&custom).unwrap();
        std::fs::write(custom.join("mpv.conf"), "volume=22\n").unwrap();
        let plan = prepare(&default, custom.clone()).unwrap();
        assert!(!default.exists());
        assert_eq!(status(&default).unwrap().default_root, default);
        assert_eq!(plan.preview.after.config_directory, "config");
        let stale = prepare(&default, base.join("other")).unwrap();
        execute(plan).unwrap();
        assert_eq!(read(&default).unwrap().root, custom);
        let mut legacy = read_saved(&default).unwrap();
        legacy.config_directory = ".".into();
        deploy::atomic_write(&record(&default), &serde_json::to_vec(&legacy).unwrap()).unwrap();
        assert_eq!(
            prepare(&default, custom.clone())
                .unwrap()
                .preview
                .after
                .config_directory,
            "."
        );
        assert!(execute(stale).is_err());
        assert_eq!(
            std::fs::read_to_string(custom.join("mpv.conf")).unwrap(),
            "volume=22\n"
        );
        assert!(prepare(&default, PathBuf::from(r"C:\")).is_err());
        std::fs::create_dir_all(custom.join(".mpvkit")).unwrap();
        std::fs::write(
            custom.join(".mpvkit/owner.json"),
            br#"{"schemaVersion":1,"owner":"MpvKit"}"#,
        )
        .unwrap();
        std::fs::write(
            custom.join(".mpvkit/associations.json"),
            br#"{"schemaVersion":1,"owner":"MpvKit","values":[]}"#,
        )
        .unwrap();
        assert_eq!(
            prepare(&default, default.clone()).err().unwrap().code,
            "target_in_use"
        );
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn unavailable_saved_target_can_be_reselected_without_touching_old_path() {
        let base = std::env::temp_dir().join(format!("mpvkit-target-{}", deploy::operation_id()));
        let default = base.join("default");
        let _lock = deploy::acquire(&default).unwrap();
        let old = Target {
            schema_version: 1,
            owner: "MpvKit".into(),
            root: PathBuf::from("//unavailable.example/share/mpv"),
            config_directory: "config".into(),
        };
        deploy::atomic_write(&record(&default), &serde_json::to_vec(&old).unwrap()).unwrap();
        drop(_lock);
        assert!(read(&default).is_err());
        assert_eq!(read_saved(&default).unwrap(), old);
        let plan = prepare(&default, base.join("new")).unwrap();
        assert!(plan.preview.before_unavailable);
        execute(plan).unwrap();
        assert_eq!(read(&default).unwrap().root, base.join("new"));
        assert!(!base.join("new").exists());
        std::fs::remove_dir_all(base).unwrap();
    }
}
