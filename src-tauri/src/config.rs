use crate::{
    deploy::{self, Payload},
    error::{AppError, Result},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    PowerSaving,
    #[default]
    Balanced,
    HighQuality,
}
impl Profile {
    pub fn id(self) -> &'static str {
        match self {
            Self::PowerSaving => "power-saving",
            Self::Balanced => "balanced",
            Self::HighQuality => "high-quality",
        }
    }
}
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Overrides {
    pub subtitle_size: Option<u16>,
    pub osd_size: Option<u16>,
    pub volume: Option<u16>,
    pub hardware_decoding: Option<bool>,
}
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct Playback {
    pub display_id: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub manual_profile: bool,
    pub hdr: Hdr,
    pub audio_languages: Vec<String>,
    pub subtitle_languages: Vec<String>,
}
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Hdr {
    #[default]
    Auto,
    On,
    Off,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
    pub schema_version: u32,
    pub profile: Profile,
    pub overrides: Overrides,
    #[serde(default)]
    pub playback: Playback,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: 2,
            profile: Profile::Balanced,
            overrides: Overrides::default(),
            playback: Playback::default(),
        }
    }
}
fn error(code: &'static str, message: &'static str) -> AppError {
    AppError::new(code, message, "config")
}
impl Settings {
    pub fn validate(&self) -> Result<()> {
        if !matches!(self.schema_version, 1 | 2) {
            return Err(error(
                "settings_version",
                "设置版本不受支持，已保留原文件。",
            ));
        }
        if self
            .overrides
            .subtitle_size
            .is_some_and(|n| !(12..=72).contains(&n))
            || self
                .overrides
                .osd_size
                .is_some_and(|n| !(12..=72).contains(&n))
            || self.overrides.volume.is_some_and(|n| n > 100)
        {
            return Err(error(
                "settings_range",
                "字幕/界面大小需为 12–72，音量需为 0–100。",
            ));
        }
        let p = &self.playback;
        if p.width.is_some() != p.height.is_some()
            || p.width.is_some_and(|n| !(320..=16384).contains(&n))
            || p.height.is_some_and(|n| !(200..=16384).contains(&n))
            || p.display_id
                .as_ref()
                .is_some_and(|id| id.len() > 512 || id.chars().any(char::is_control))
        {
            return Err(error("display_range", "请填写有效的参考分辨率。"));
        }
        for languages in [&p.audio_languages, &p.subtitle_languages] {
            let mut seen = std::collections::BTreeSet::new();
            if languages.len() > 8 {
                return Err(error("language_limit", "最多选择八种语言。"));
            }
            for lang in languages {
                let parts: Vec<_> = lang.split('-').collect();
                if lang.len() > 35
                    || !(2..=3).contains(&parts[0].len())
                    || !parts[0].bytes().all(|b| b.is_ascii_alphabetic())
                    || parts.iter().skip(1).any(|p| {
                        p.is_empty() || p.len() > 8 || !p.bytes().all(|b| b.is_ascii_alphanumeric())
                    })
                    || !seen.insert(lang.to_ascii_lowercase())
                {
                    return Err(error("language_invalid", "语言代码无效或重复。"));
                }
            }
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Profiles {
    schema_version: u32,
    profiles: BTreeMap<String, BTreeMap<String, String>>,
}
pub fn profiles() -> BTreeMap<String, BTreeMap<String, String>> {
    let profiles: Profiles = serde_json::from_str(include_str!("../../resources/profiles.json"))
        .expect("embedded profiles");
    assert_eq!(profiles.schema_version, 1);
    profiles.profiles
}

pub fn generate(root: &Path, player: &Path, settings: &Settings) -> Result<Vec<Payload>> {
    let layout = deploy::status(root)?
        .manifest
        .map(|m| m.config_directory)
        .unwrap_or_else(deploy::default_config_directory);
    generate_at(root, player, settings, &layout)
}

pub fn generate_at(
    root: &Path,
    player: &Path,
    settings: &Settings,
    layout: &str,
) -> Result<Vec<Payload>> {
    settings.validate()?;
    let mut migrated = settings.clone();
    migrated.schema_version = 2;
    let settings = &migrated;
    crate::platform::validate_local_path(root)?;
    crate::platform::validate_local_path(player)?;
    let player = player
        .to_str()
        .filter(|p| !p.chars().any(char::is_control))
        .ok_or_else(|| error("config_path", "路径无法用于组件配置。"))?;
    let mut conf = String::from(
        "# Generated by MpvKit. Use user.conf in the selected configuration directory for optional additions.\nosc=no\nload-scripts=yes\n",
    );
    for (key, value) in &profiles()[settings.profile.id()] {
        conf.push_str(&format!("{key}={value}\n"));
    }
    let overrides = &settings.overrides;
    conf.push_str(&format!(
        "sub-font-size={}\nosd-font-size={}\nvolume={}\n",
        overrides.subtitle_size.unwrap_or(38),
        overrides.osd_size.unwrap_or(30),
        overrides.volume.unwrap_or(100)
    ));
    conf.push_str(crate::platform::playback_config(
        overrides.hardware_decoding.unwrap_or(true),
    ));
    match settings.playback.hdr {
        Hdr::Auto => {
            conf.push_str("target-colorspace-hint=auto\ntarget-trc=auto\ntarget-prim=auto\n")
        }
        Hdr::On => {
            conf.push_str("target-colorspace-hint=yes\ntarget-trc=pq\ntarget-prim=bt.2020\n")
        }
        Hdr::Off => {
            conf.push_str("target-colorspace-hint=no\ntarget-trc=bt.1886\ntarget-prim=bt.709\n")
        }
    }
    for (key, list) in [
        ("alang", &settings.playback.audio_languages),
        ("slang", &settings.playback.subtitle_languages),
    ] {
        if !list.is_empty() {
            conf.push_str(&format!("{key}={}\n", list.join(",")));
        }
    }
    let settings = serde_json::to_vec_pretty(settings)
        .map_err(|_| error("settings_encode", "设置无法编码。"))?;
    let context = serde_json::to_vec_pretty(&serde_json::json!({ "schemaVersion": 1, "root": root, "helper": root.join("launcher/mpvkit.exe"), "profile": serde_json::from_slice::<Settings>(&settings).expect("encoded settings").profile })).map_err(|_| error("config_encode", "无法生成播放上下文。"))?;
    let mut files = vec![
        Payload {
            path: "config/scripts/mpvkit.lua".into(),
            bytes: include_bytes!("../../resources/scripts/mpvkit.lua").to_vec(),
        },
        Payload {
            path: "config/mpvkit-context.json".into(),
            bytes: context,
        },
        Payload {
            path: "config/input.conf".into(),
            bytes: b"# MpvKit shortcuts are registered by scripts/mpvkit.lua. Use user.conf for trusted additions.\n".to_vec(),
        },
        Payload {
            path: "config/mpv.conf".into(),
            bytes: conf.into_bytes(),
        },
        Payload {
            path: "config/script-opts/thumbfast.conf".into(),
            bytes: format!(
                "mpv_path={}\nnetwork=no\naudio=no\nhwdec=no\n",
                player.replace('\\', "/")
            )
            .into_bytes(),
        },
        Payload {
            path: "state/settings.json".into(),
            bytes: settings,
        },
        Payload {
            path: "config/mpvkit-profiles.json".into(),
            bytes: include_bytes!("../../resources/profiles.json").to_vec(),
        },
    ];
    let executable =
        std::env::current_exe().map_err(|_| error("helper_path", "无法取得启动入口。"))?;
    files.push(Payload {
        path: "launcher/mpvkit.exe".into(),
        bytes: deploy::read_bytes(&executable)?,
    });
    for (name, bytes) in [
        ("LICENSE.txt", include_bytes!("../../LICENSE").as_slice()),
        (
            "PROJECT-LICENSE.md",
            include_bytes!("../../COPYING.md").as_slice(),
        ),
        (
            "third-party/APP-DEPENDENCY-NOTICES.txt",
            include_bytes!("../../resources/app-dependency-notices.txt").as_slice(),
        ),
        (
            "third-party/app-dependency-inventory.json",
            include_bytes!("../../resources/app-dependency-inventory.json").as_slice(),
        ),
        (
            "third-party/app-distribution-materials.json",
            include_bytes!("../../resources/app-distribution-materials.json").as_slice(),
        ),
        (
            "third-party/RUST-COPYRIGHT-library.html",
            include_bytes!("../../resources/licenses/rust-library-COPYRIGHT.txt").as_slice(),
        ),
        (
            "third-party/WEBVIEW2-SDK-LICENSE.txt",
            include_bytes!("../../resources/licenses/webview2-sdk-LICENSE.txt").as_slice(),
        ),
        (
            "third-party/WEBVIEW2-SDK-NOTICE.txt",
            include_bytes!("../../resources/licenses/webview2-sdk-NOTICE.txt").as_slice(),
        ),
    ] {
        files.push(Payload {
            path: format!("launcher/{name}"),
            bytes: bytes.to_vec(),
        });
    }
    map_layout(&mut files, layout)?;
    Ok(files)
}

pub fn map_layout(files: &mut [Payload], layout: &str) -> Result<()> {
    if !matches!(layout, "config" | ".") {
        return Err(error("invalid_layout", "配置目录布局不受支持。"));
    }
    if layout == "." {
        for file in files {
            if let Some(path) = file.path.strip_prefix("config/") {
                file.path = path.into();
            }
        }
    }
    Ok(())
}

pub fn directory(root: &Path) -> Result<std::path::PathBuf> {
    let layout = deploy::status(root)?
        .manifest
        .map(|m| m.config_directory)
        .unwrap_or_else(deploy::default_config_directory);
    Ok(root.join(layout))
}

pub fn read(root: &Path) -> Result<Settings> {
    let path = root.join("state/settings.json");
    crate::platform::validate_local_path(&path)?;
    if !path.exists() {
        return Ok(Settings::default());
    }
    let bytes = deploy::read_bytes(&path)?;
    if bytes.len() > 64 * 1024 {
        return Err(error("settings_limit", "设置文件过大，已保留。"));
    }
    let mut value: Settings = serde_json::from_slice(&bytes)
        .map_err(|_| error("settings_invalid", "设置损坏或包含未知字段，已保留。"))?;
    value.validate()?;
    if value.schema_version == 1 {
        value.playback.manual_profile = true;
    }
    Ok(value)
}

pub fn validate_output(settings: &Settings) -> Result<()> {
    if settings.playback.hdr == Hdr::On {
        let device = crate::platform::detect_device();
        let selected = settings
            .playback
            .display_id
            .as_ref()
            .and_then(|id| device.displays.iter().find(|d| &d.id == id))
            .or_else(|| {
                settings
                    .playback
                    .display_id
                    .is_none()
                    .then(|| {
                        device
                            .displays
                            .iter()
                            .find(|d| d.primary)
                            .or_else(|| device.displays.first())
                    })
                    .flatten()
            });
        if !selected.is_some_and(|d| d.hdr_supported == Some(true) && d.hdr_enabled == Some(true)) {
            return Err(error(
                "hdr_unavailable",
                "请先启用目标屏幕的系统 HDR，或选择自动模式。",
            ));
        }
    }
    Ok(())
}

pub fn prepare(root: &Path, settings: Settings) -> Result<deploy::DeploymentPlan> {
    settings.validate()?;
    validate_output(&settings)?;
    let status = deploy::status(root)?;
    let manifest = status
        .manifest
        .ok_or_else(|| error("not_installed", "请先部署受管理环境。"))?;
    let player = manifest
        .external_mpv
        .clone()
        .unwrap_or_else(|| root.join("runtime/mpv.exe"));
    let mut payloads = generate(root, &player, &settings)?;
    for file in &manifest.files {
        if payloads.iter().any(|p| p.path == file.path) {
            continue;
        }
        let path = root.join(&file.path);
        deploy::validate_relative(&file.path)?;
        let bytes = deploy::read_bytes(&path)?;
        if deploy::hash_bytes(&bytes) != file.sha256 {
            return Err(error("file_conflict", "受管理文件已被修改，请保留并检查。"));
        }
        payloads.push(Payload {
            path: file.path.clone(),
            bytes,
        });
    }
    deploy::prepare_install(
        root,
        payloads,
        false,
        manifest.external_mpv,
        manifest.components,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn profiles_reset_all_shared_keys_and_overrides_take_precedence() {
        let root = std::env::temp_dir().join(format!("mpvkit-config-{}", deploy::operation_id()));
        let player = root.join("中文 空格/mpv.exe");
        let profiles = profiles();
        let keys: Vec<_> = profiles["balanced"].keys().collect();
        assert_eq!(profiles.len(), 3);
        for values in profiles.values() {
            assert_eq!(values.keys().collect::<Vec<_>>(), keys);
        }
        let mut settings = Settings::default();
        settings.overrides.subtitle_size = Some(45);
        settings.overrides.hardware_decoding = Some(false);
        let files = generate(&root, &player, &settings).unwrap();
        let conf = String::from_utf8(
            files
                .iter()
                .find(|p| p.path == "config/mpv.conf")
                .unwrap()
                .bytes
                .clone(),
        )
        .unwrap();
        assert!(conf.contains("sub-font-size=45\n"));
        assert!(conf.contains("volume=100\n"));
        assert!(conf.contains("hwdec=no\n"));
        assert!(
            String::from_utf8(
                files
                    .iter()
                    .find(|p| p.path == "config/script-opts/thumbfast.conf")
                    .unwrap()
                    .bytes
                    .clone()
            )
            .unwrap()
            .contains("中文 空格/mpv.exe\n")
        );
        assert!(!root.exists());
        settings.overrides.subtitle_size = None;
        assert!(
            String::from_utf8(
                generate(&root, &player, &settings)
                    .unwrap()
                    .into_iter()
                    .find(|p| p.path == "config/mpv.conf")
                    .unwrap()
                    .bytes
            )
            .unwrap()
            .contains("sub-font-size=38\n")
        );
        settings.schema_version = 99;
        assert!(settings.validate().is_err());
        settings.schema_version = 1;
        settings.overrides.volume = Some(101);
        assert!(settings.validate().is_err());
        assert!(
            serde_json::from_str::<Settings>(
                r#"{"schemaVersion":1,"profile":"balanced","overrides":{},"injected":true}"#
            )
            .is_err()
        );
    }
    #[test]
    fn settings_updates_use_transaction_and_preserve_modified_or_user_files() {
        let root = std::env::temp_dir().join(format!("mpvkit-config-{}", deploy::operation_id()));
        let files = generate(&root, &root.join("runtime/mpv.exe"), &Settings::default()).unwrap();
        deploy::execute(
            deploy::prepare_install(&root, files, false, None, BTreeMap::new()).unwrap(),
        )
        .unwrap();
        std::fs::write(root.join("config/user.conf"), "user setting").unwrap();
        let mut edited = read(&root).unwrap();
        edited.profile = Profile::HighQuality;
        edited.overrides.volume = Some(25);
        let plan = prepare(&root, edited.clone()).unwrap();
        assert_eq!(read(&root).unwrap(), Settings::default());
        deploy::execute(plan).unwrap();
        assert_eq!(read(&root).unwrap(), edited);
        assert_eq!(
            std::fs::read_to_string(root.join("config/user.conf")).unwrap(),
            "user setting"
        );
        deploy::execute(deploy::prepare_restore(&root).unwrap()).unwrap();
        assert_eq!(read(&root).unwrap(), Settings::default());
        std::fs::write(root.join("config/mpv.conf"), "manual").unwrap();
        assert!(!prepare(&root, edited).unwrap().preview.conflicts.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn v1_migration_preserves_overrides_and_v2_rejects_invalid_preferences() {
        let root = std::env::temp_dir().join(format!("mpvkit-migrate-{}", deploy::operation_id()));
        let old = br#"{"schemaVersion":1,"profile":"high-quality","overrides":{"subtitleSize":44,"volume":23}}"#;
        let settings: Settings = serde_json::from_slice(old).unwrap();
        let mut files = generate(&root, &root.join("runtime/mpv.exe"), &settings).unwrap();
        files
            .iter_mut()
            .find(|p| p.path == "state/settings.json")
            .unwrap()
            .bytes = old.to_vec();
        deploy::execute(
            deploy::prepare_install(&root, files, false, None, BTreeMap::new()).unwrap(),
        )
        .unwrap();
        let loaded = read(&root).unwrap();
        assert_eq!(loaded.schema_version, 1);
        assert!(loaded.playback.manual_profile);
        assert_eq!(loaded.overrides.volume, Some(23));
        let plan = prepare(&root, loaded).unwrap();
        assert_eq!(
            std::fs::read(root.join("state/settings.json")).unwrap(),
            old
        );
        deploy::execute(plan).unwrap();
        let migrated = read(&root).unwrap();
        assert_eq!(migrated.schema_version, 2);
        assert_eq!(migrated.profile, Profile::HighQuality);
        assert_eq!(migrated.overrides.subtitle_size, Some(44));
        deploy::execute(deploy::prepare_restore(&root).unwrap()).unwrap();
        assert_eq!(
            std::fs::read(root.join("state/settings.json")).unwrap(),
            old
        );
        let mut edited = migrated;
        for languages in [
            vec!["en", "EN"],
            vec!["en,ja"],
            vec!["en--US"],
            vec!["en\nvolume=0"],
        ] {
            edited.playback.audio_languages = languages.into_iter().map(String::from).collect();
            assert!(edited.validate().is_err());
        }
        edited.playback.audio_languages = vec!["ja".into(), "en-US".into()];
        edited.playback.subtitle_languages = vec!["zh-Hans".into()];
        for (hdr, expected) in [
            (Hdr::Auto, "target-trc=auto"),
            (Hdr::On, "target-trc=pq"),
            (Hdr::Off, "target-trc=bt.1886"),
        ] {
            edited.playback.hdr = hdr;
            let files = generate(&root, &root.join("runtime/mpv.exe"), &edited).unwrap();
            let conf = String::from_utf8(
                files
                    .into_iter()
                    .find(|f| f.path == "config/mpv.conf")
                    .unwrap()
                    .bytes,
            )
            .unwrap();
            assert!(conf.contains(expected));
            assert!(conf.contains("alang=ja,en-US"));
            assert!(conf.contains("slang=zh-Hans"));
        }
        edited.playback.width = Some(1);
        assert!(edited.validate().is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
