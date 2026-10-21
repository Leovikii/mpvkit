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
    // Legacy field retained for existing v1 records; material selection is automatic.
    pub background: String,
}
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: 1,
            language: "system".into(),
            theme: "system".into(),
            accent: "system".into(),
            background: "auto".into(),
        }
    }
}
impl AppSettings {
    fn validate(&self) -> Result<()> {
        if self.schema_version != 1
            || !["system", "zh-Hans", "en"].contains(&self.language.as_str())
            || !["system", "light", "dark"].contains(&self.theme.as_str())
            || !["system", "blue", "teal", "violet", "orange"].contains(&self.accent.as_str())
            || !["auto", "solid"].contains(&self.background.as_str())
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
pub fn read(root: &Path) -> Result<AppSettings> {
    deploy::status(root)?;
    let path = root.join(".mpvkit/app-settings.json");
    if deploy::file_hash(&path)?.is_none() {
        return Ok(AppSettings::default());
    }
    let bytes = deploy::read_bytes(&path)?;
    if bytes.len() > 65536 {
        return Err(AppError::new(
            "app_settings_invalid",
            "应用设置文件过大。",
            "settings",
        ));
    }
    let mut settings: AppSettings = serde_json::from_slice(&bytes)
        .map_err(|_| AppError::new("app_settings_invalid", "应用设置损坏，已保留。", "settings"))?;
    settings.validate()?;
    settings.background = "auto".into();
    Ok(settings)
}
pub fn save(root: &Path, mut settings: AppSettings) -> Result<AppSettings> {
    settings.validate()?;
    settings.background = "auto".into();
    let _lock = deploy::acquire(root)?;
    read(root)?;
    deploy::atomic_write(
        &root.join(".mpvkit/app-settings.json"),
        &serde_json::to_vec_pretty(&settings)
            .map_err(|_| AppError::new("settings_encode", "无法保存应用设置。", "settings"))?,
    )?;
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
        let mut legacy = settings.clone();
        legacy.background = "solid".into();
        std::fs::write(
            root.join(".mpvkit/app-settings.json"),
            serde_json::to_vec(&legacy).unwrap(),
        )
        .unwrap();
        assert_eq!(read(&root).unwrap().background, "auto");
        assert_eq!(save(&root, legacy).unwrap().background, "auto");
        let bytes = br#"{"schemaVersion":999,"language":"en","theme":"dark","accent":"blue","background":"auto"}"#;
        std::fs::write(root.join(".mpvkit/app-settings.json"), bytes).unwrap();
        assert!(save(&root, AppSettings::default()).is_err());
        assert_eq!(
            std::fs::read(root.join(".mpvkit/app-settings.json")).unwrap(),
            bytes
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
