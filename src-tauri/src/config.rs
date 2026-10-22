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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Playback {
    pub display_id: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub manual_profile: bool,
    pub hdr: Hdr,
    pub audio_languages: Vec<String>,
    pub subtitle_languages: Vec<String>,
    pub keep_open: bool,
    pub screenshot_directory: Option<String>,
    pub screenshot_format: ScreenshotFormat,
    pub screenshot_subtitles: bool,
}
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScreenshotFormat {
    #[default]
    Png,
    Jpeg,
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
    pub playback: Playback,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: 3,
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
        if self.schema_version != 3 {
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
        if let Some(directory) = &p.screenshot_directory {
            if directory.is_empty()
                || directory.len() > 4096
                || directory.chars().any(char::is_control)
            {
                return Err(error("screenshot_path", "截图目录无效。"));
            }
            crate::platform::validate_local_path(Path::new(directory))?;
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

pub(crate) mod portable;
pub use portable::{editable, helper_path, modified, player_path, prepare_portable};
pub fn generate(root: &Path, player: &Path, settings: &Settings) -> Result<Vec<Payload>> {
    portable::generate(root, player, settings)
}
fn base_files(player: &Path, settings: &Settings) -> Result<Vec<Payload>> {
    settings.validate()?;
    let player = player
        .to_str()
        .filter(|p| !p.chars().any(char::is_control))
        .ok_or_else(|| error("config_path", "路径无法用于组件配置。"))?;
    let mut conf = String::from("osc=no\nload-scripts=yes\n");
    for (key, value) in &profiles()[settings.profile.id()] {
        conf.push_str(&format!("{key}={value}\n"));
    }
    conf.push_str("input-builtin-bindings=no\nvideo-sync=audio\ninterpolation=no\nsub-font=LXGW WenKai Screen\nsave-position-on-quit=yes\nwatch-later-options=start\nresume-playback=yes\nwatch-later-directory=~~/watch_later\n");
    conf.push_str(if settings.playback.keep_open {
        "keep-open=yes\n"
    } else {
        "keep-open=no\n"
    });
    let screenshots = settings
        .playback
        .screenshot_directory
        .as_ref()
        .map(std::path::PathBuf::from)
        .map(Ok)
        .unwrap_or_else(crate::platform::pictures_directory)?;
    crate::platform::validate_local_path(&screenshots)?;
    let screenshots = screenshots
        .to_str()
        .ok_or_else(|| error("screenshot_path", "截图目录无效。"))?;
    conf.push_str(&format!(
        "screenshot-directory=%{}%{}\nscreenshot-format={}\n",
        screenshots.len(),
        screenshots,
        match settings.playback.screenshot_format {
            ScreenshotFormat::Png => "png",
            ScreenshotFormat::Jpeg => "jpg",
        }
    ));
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
    let mut files = vec![
        Payload {
            path: "portable_config/scripts/mpvkit.lua".into(),
            bytes: include_bytes!("../../resources/scripts/mpvkit.lua").to_vec(),
        },
        Payload {
            path: "portable_config/input.conf".into(),
            bytes: include_bytes!("../../resources/input.conf").to_vec(),
        },
        Payload {
            path: "portable_config/mpv.conf".into(),
            bytes: conf.into_bytes(),
        },
        Payload {
            path: "portable_config/script-opts/thumbfast.conf".into(),
            bytes: format!(
                "mpv_path={}\nnetwork=no\naudio=no\nhwdec=no\n",
                player.replace('\\', "/")
            )
            .into_bytes(),
        },
        Payload {
            path: ".mpvkit/settings.json".into(),
            bytes: Vec::new(),
        },
    ];
    let executable =
        std::env::current_exe().map_err(|_| error("helper_path", "无法取得启动入口。"))?;
    files.push(Payload {
        path: ".mpvkit/launcher/mpvkit.exe".into(),
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
            path: format!(".mpvkit/launcher/{name}"),
            bytes: bytes.to_vec(),
        });
    }
    Ok(files)
}

pub fn directory(root: &Path) -> Result<std::path::PathBuf> {
    Ok(root.join("portable_config"))
}
pub fn read(root: &Path) -> Result<Settings> {
    portable::read_state(root).map(|state| state.settings)
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
    prepare_portable(root, settings, false)
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
                .find(|p| p.path == "portable_config/mpv.conf")
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
                    .find(|p| p.path == "portable_config/script-opts/thumbfast.conf")
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
                    .find(|p| p.path == "portable_config/mpv.conf")
                    .unwrap()
                    .bytes
            )
            .unwrap()
            .contains("sub-font-size=38\n")
        );
        settings.schema_version = 99;
        assert!(settings.validate().is_err());
        settings.schema_version = 3;
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
    fn playback_defaults_and_screenshot_paths_cannot_inject_options() {
        let root = std::env::temp_dir().join(format!("mpvkit-defaults-{}", deploy::operation_id()));
        let mut settings = Settings::default();
        settings.playback.screenshot_directory =
            Some(root.join("截图 # 空格").to_string_lossy().into_owned());
        let files = generate(&root, &root.join("mpv.exe"), &settings).unwrap();
        let conf = String::from_utf8(
            files
                .iter()
                .find(|f| f.path == "portable_config/mpv.conf")
                .unwrap()
                .bytes
                .clone(),
        )
        .unwrap();
        for expected in [
            "save-position-on-quit=yes\n",
            "watch-later-options=start\n",
            "video-sync=audio\n",
            "interpolation=no\n",
            "input-builtin-bindings=no\n",
            "sub-font=LXGW WenKai Screen\n",
        ] {
            assert!(conf.contains(expected), "{expected}");
        }
        assert!(!root.exists());
        settings.playback.screenshot_directory = Some("C:/pictures\nvolume=0".into());
        assert!(settings.validate().is_err());
        settings.playback.screenshot_directory = Some("../pictures".into());
        assert!(settings.validate().is_err());
    }
    #[test]
    fn languages_and_hdr_parameters_are_validated() {
        let root = std::env::temp_dir().join(format!("mpvkit-config-{}", deploy::operation_id()));
        let mut edited = Settings::default();
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
            let files = generate(&root, &root.join("mpv.exe"), &edited).unwrap();
            let conf = String::from_utf8(
                files
                    .into_iter()
                    .find(|f| f.path == "portable_config/mpv.conf")
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
    }
}
