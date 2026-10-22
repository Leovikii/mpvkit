use crate::{
    deploy,
    error::{AppError, Result},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppSettings {
    pub schema_version: u32,
    pub language: String,
    pub theme: String,
    pub accent: String,
}
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: 1,
            language: "system".into(),
            theme: "system".into(),
            accent: "system".into(),
        }
    }
}
impl AppSettings {
    fn validate(&self) -> Result<()> {
        if self.schema_version != 1
            || !["system", "zh-Hans", "en"].contains(&self.language.as_str())
            || !["system", "light", "dark"].contains(&self.theme.as_str())
            || !["system", "blue", "teal", "violet", "orange"].contains(&self.accent.as_str())
        {
            return Err(AppError::new(
                "app_settings_invalid",
                "应用设置无效或版本不受支持，已保留原文件。",
                "settings",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Record {
    schema_version: u32,
    owner: String,
    pub settings: AppSettings,
}
pub(crate) fn record_path(root: &Path) -> std::path::PathBuf {
    root.join("mpvkit.json")
}
pub(crate) fn load(root: &Path) -> Result<Record> {
    deploy::status(root)?;
    let path = record_path(root);
    if deploy::file_hash(&path)?.is_some() {
        let bytes = deploy::read_bytes(&path)?;
        if bytes.len() > 65536 {
            return Err(AppError::new(
                "app_settings_invalid",
                "软件设置过大。",
                "settings",
            ));
        }
        let r: Record = serde_json::from_slice(&bytes).map_err(|_| {
            AppError::new(
                "app_settings_invalid",
                "软件设置损坏或含未知字段。",
                "settings",
            )
        })?;
        if r.schema_version != 1 || r.owner != "MpvKit" {
            return Err(AppError::new(
                "app_settings_invalid",
                "软件设置版本或归属未知。",
                "settings",
            ));
        }
        r.settings.validate()?;
        return Ok(r);
    }
    Ok(Record {
        schema_version: 1,
        owner: "MpvKit".into(),
        settings: AppSettings::default(),
    })
}
pub(crate) fn write(root: &Path, record: &Record) -> Result<()> {
    deploy::atomic_write(
        &record_path(root),
        &serde_json::to_vec_pretty(record)
            .map_err(|_| AppError::new("settings_encode", "无法保存软件设置。", "settings"))?,
    )
}
pub fn read(root: &Path) -> Result<AppSettings> {
    Ok(load(root)?.settings)
}
pub fn save(root: &Path, settings: AppSettings) -> Result<AppSettings> {
    settings.validate()?;
    let _lock = deploy::acquire(root)?;
    let mut record = load(root)?;
    record.settings = settings.clone();
    write(root, &record)?;
    Ok(settings)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn app_preferences_preserve_unknown_state_and_are_independent_of_installation() {
        let root = std::env::temp_dir().join(format!("mpvkit-app-{}", deploy::operation_id()));
        assert_eq!(read(&root).unwrap(), AppSettings::default());
        assert!(!root.exists());
        let settings = AppSettings {
            language: "en".into(),
            theme: "dark".into(),
            ..Default::default()
        };
        save(&root, settings.clone()).unwrap();
        assert_eq!(read(&root).unwrap(), settings);
        assert!(deploy::status(&root).unwrap().manifest.is_none());
        let mut unknown = serde_json::to_value(load(&root).unwrap()).unwrap();
        unknown["schemaVersion"] = 999.into();
        let bytes = serde_json::to_vec(&unknown).unwrap();
        std::fs::write(record_path(&root), &bytes).unwrap();
        assert!(save(&root, AppSettings::default()).is_err());
        assert_eq!(std::fs::read(record_path(&root)).unwrap(), bytes);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn preferences_are_locked_across_processes() {
        let root = std::env::temp_dir().join(format!("mpvkit-global-{}", deploy::operation_id()));
        let lock = deploy::acquire(&root).unwrap();
        assert!(save(&root, AppSettings::default()).is_err());
        drop(lock);
        save(&root, AppSettings::default()).unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }
}
