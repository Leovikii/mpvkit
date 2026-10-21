use crate::{
    deploy,
    error::{AppError, Result},
    mpv, platform,
};
use serde::Serialize;
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub schema_version: u32,
    pub application: &'static str,
    pub version: &'static str,
    pub installed: bool,
    pub components: std::collections::BTreeMap<String, String>,
    pub actual_player_version: Option<String>,
    pub player_issue: Option<&'static str>,
    pub config_source: &'static str,
    pub user_config_present: bool,
    pub managed_files: usize,
    pub changed_or_unavailable_files: usize,
    pub recovery_required: bool,
    pub previous_available: bool,
    pub privacy: &'static str,
}
pub struct ExportPlan {
    pub id: String,
    pub report: Report,
    pub destination: PathBuf,
    created: Instant,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPreview {
    pub id: String,
    pub report: Report,
    pub destination: PathBuf,
}
impl ExportPlan {
    pub fn preview(&self) -> ExportPreview {
        ExportPreview {
            id: self.id.clone(),
            report: self.report.clone(),
            destination: self.destination.clone(),
        }
    }
}
fn error(code: &'static str, message: &'static str) -> AppError {
    AppError::new(code, message, "diagnostics")
}
fn safe_version(version: &str) -> Option<String> {
    if version.len() <= 128
        && version.starts_with("mpv ")
        && version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || " ._+-()".contains(c))
    {
        Some(version.into())
    } else {
        None
    }
}
pub fn prepare(root: &Path, destination: &Path) -> Result<ExportPlan> {
    platform::validate_local_path(destination)?;
    if !destination
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("json"))
    {
        return Err(error("json_required", "请选择新的 .json 文件。"));
    }
    if destination.exists() {
        return Err(error(
            "export_exists",
            "导出目标已存在，请选择新文件；不会覆盖。",
        ));
    }
    let status = deploy::status(root)?;
    let mut components = Default::default();
    let mut version = None;
    let mut player_issue = None;
    let mut managed_files = 0;
    let mut changed = 0;
    if let Some(manifest) = &status.manifest {
        components = manifest
            .components
            .iter()
            .filter(|(k, v)| {
                ["mpv", "uosc", "thumbfast"].contains(&k.as_str())
                    && v.len() <= 128
                    && v.chars()
                        .all(|c| c.is_ascii_alphanumeric() || ".-_".contains(c))
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        managed_files = manifest.files.len();
        for file in &manifest.files {
            deploy::validate_relative(&file.path)?;
            if deploy::file_hash(&root.join(&file.path))
                .ok()
                .flatten()
                .as_deref()
                != Some(&file.sha256)
            {
                changed += 1;
            }
        }
        match mpv::managed_executable(root, manifest)
            .and_then(|executable| mpv::inspect(&executable))
        {
            Ok(info) => {
                version = safe_version(&info.version);
                if version.is_none() {
                    player_issue = Some("unrecognized_version_redacted");
                }
            }
            Err(e) => player_issue = Some(e.code),
        }
    }
    let user = crate::config::directory(root)?.join("user.conf");
    platform::validate_local_path(&user)?;
    let report = Report {
        schema_version: 1,
        application: "MpvKit",
        version: env!("CARGO_PKG_VERSION"),
        installed: status.manifest.is_some(),
        components,
        actual_player_version: version,
        player_issue,
        config_source: if status.manifest.is_some() {
            "managed-config"
        } else {
            "none"
        },
        user_config_present: user.is_file(),
        managed_files,
        changed_or_unavailable_files: changed,
        recovery_required: status.recovery_required,
        previous_available: status.previous_available,
        privacy: "仅含固定组件版本、检查代码和文件计数；未包含用户名、绝对路径、媒体/URL、原始配置或日志。仅保存本地，不上传。",
    };
    Ok(ExportPlan {
        id: deploy::operation_id(),
        report,
        destination: destination.into(),
        created: Instant::now(),
    })
}
pub fn execute(plan: ExportPlan) -> Result<PathBuf> {
    if plan.created.elapsed() > Duration::from_secs(300) {
        return Err(error("stale_preview", "诊断预览已过期。"));
    }
    platform::validate_local_path(&plan.destination)?;
    let bytes = serde_json::to_vec_pretty(&plan.report)
        .map_err(|_| error("export_encode", "诊断信息编码失败。"))?;
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&plan.destination)
        .map_err(|_| error("export_exists", "导出目标已存在或不可写；未覆盖。"))?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| {
            error(
                "export_write",
                "导出写入失败，可能留下不完整 JSON，请检查目标文件。",
            )
        })?;
    Ok(plan.destination)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preview_is_private_read_only_and_export_cannot_overwrite() {
        let root =
            std::env::temp_dir().join(format!("mpvkit-diagnostics-{}", deploy::operation_id()));
        std::fs::create_dir(&root).unwrap();
        let destination = root.join("report.json");
        let plan = prepare(&root.join("missing"), &destination).unwrap();
        assert!(!destination.exists());
        let text = serde_json::to_string(&plan.report).unwrap();
        assert!(!text.contains(&root.to_string_lossy().to_string()));
        execute(plan).unwrap();
        let bytes = std::fs::read(&destination).unwrap();
        assert!(prepare(&root, &destination).is_err());
        let other = root.join("race.json");
        let plan = prepare(&root.join("missing"), &other).unwrap();
        std::fs::write(&other, "user file").unwrap();
        assert!(execute(plan).is_err());
        assert_eq!(std::fs::read_to_string(&other).unwrap(), "user file");
        assert_eq!(std::fs::read(&destination).unwrap(), bytes);
        assert!(safe_version("mpv v1 https://user:token@host/private").is_none());
        assert!(safe_version("mpv C:\\Users\\name\\file").is_none());
        assert!(safe_version("mpv v0.41.0-1224-g6a262d11f").is_some());
        std::fs::remove_dir_all(root).unwrap();
    }
}
