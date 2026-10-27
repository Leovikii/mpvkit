const DEFAULT_LANGUAGES: &str = "zh-Hans-CN,zh-Hans-SG,zh-Hant-TW,zh-Hant-HK,zh-Hant-MO,zh-Hans,zh-Hant,zh-CN,zh-SG,zh-TW,zh-HK,zh-MO,zh,cmn-Hans-CN,cmn-Hant-TW,cmn-Hans,cmn-Hant,cmn-CN,cmn-TW,cmn,yue-Hant-HK,yue-Hant,yue-HK,yue,chs,cht,ja-JP,ja,en-US,en-GB,en-AU,en-CA,en,es-ES,es-MX,es-419,es-AR,es";
use crate::{
    deploy::{self, Payload},
    error::{AppError, Result},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

macro_rules! relative {
    ($name:literal) => {
        $name
    };
}
pub(crate) use relative;

pub fn component_path(name: &str) -> String {
    format!("{}{name}", relative!(""))
}

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
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Playback {
    pub hdr: Hdr,
    pub keep_open: bool,
    pub auto_playlist: bool,
}
impl Default for Playback {
    fn default() -> Self {
        Self {
            hdr: Hdr::Auto,
            keep_open: false,
            auto_playlist: true,
        }
    }
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
pub use portable::{editable, modified, player_path, prepare_portable};
#[cfg(test)]
pub fn generate(root: &Path, player: &Path, settings: &Settings) -> Result<Vec<Payload>> {
    portable::generate(root, player, settings)
}
fn base_files(player: &Path, settings: &Settings) -> Result<Vec<Payload>> {
    settings.validate()?;
    let player = player
        .to_str()
        .filter(|p| !p.chars().any(char::is_control))
        .ok_or_else(|| error("config_path", "路径无法用于组件配置。"))?;
    let mut conf =
        String::from("osc=no\nload-scripts=yes\nautofit=75%x75%\nauto-window-resize=no\n");
    for (key, value) in &profiles()[settings.profile.id()] {
        conf.push_str(&format!("{key}={value}\n"));
    }
    conf.push_str("input-builtin-bindings=no\nvideo-sync=audio\ninterpolation=no\nsub-font=LXGW WenKai Screen\nsave-position-on-quit=yes\nwatch-later-options=start\nresume-playback=yes\nwatch-later-directory=~~/watch_later\n");
    conf.push_str("directory-filter-types=video,audio\ndirectory-mode=ignore\nsub-auto=exact\n");
    // Append individual paths so the same config works with Windows and Unix separators.
    for directory in ["sub", "subs", "subtitle", "subtitles"] {
        conf.push_str(&format!("sub-file-paths-append={directory}\n"));
    }
    // Windows resolves these case-insensitively; do not scan the same folder repeatedly.
    #[cfg(target_os = "linux")]
    for directory in [
        "Sub",
        "Subs",
        "Subtitle",
        "Subtitles",
        "SUB",
        "SUBS",
        "SUBTITLE",
        "SUBTITLES",
    ] {
        conf.push_str(&format!("sub-file-paths-append={directory}\n"));
    }
    conf.push_str(if settings.playback.auto_playlist {
        "autocreate-playlist=filter\n"
    } else {
        "autocreate-playlist=no\n"
    });
    conf.push_str(if settings.playback.keep_open {
        "keep-open=yes\n"
    } else {
        "keep-open=no\n"
    });
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
    conf.push_str(crate::platform::hdr_config(settings.playback.hdr));
    conf.push_str(&format!(
        "alang={DEFAULT_LANGUAGES}\nslang={DEFAULT_LANGUAGES}\n"
    ));
    let mut files = vec![
        Payload {
            path: crate::config::relative!("input.conf").into(),
            bytes: include_bytes!("../../resources/input.conf").to_vec(),
        },
        Payload {
            path: crate::config::relative!("mpv.conf").into(),
            bytes: conf.into_bytes(),
        },
        Payload {
            path: crate::config::relative!("script-opts/thumbfast.conf").into(),
            bytes: format!(
                "mpv_path={}\nnetwork=no\naudio=no\nhwdec=no\n",
                player.replace('\\', "/")
            )
            .into_bytes(),
        },
        Payload {
            path: crate::config::relative!("script-opts/uosc.conf").into(),
            bytes: include_bytes!("../../resources/uosc.conf").to_vec(),
        },
        Payload {
            path: ".mpvkit/settings.json".into(),
            bytes: Vec::new(),
        },
    ];
    files.extend(runtime_files()?);
    Ok(files)
}

pub fn runtime_files() -> Result<Vec<Payload>> {
    let executable =
        std::env::current_exe().map_err(|_| error("helper_path", "无法取得管理器位置。"))?;
    // Native Rust tests run in a harness, while playback must invoke the separately built manager.
    #[cfg(test)]
    let executable = std::env::var_os("MPVKIT_TEST_EXECUTABLE")
        .map(std::path::PathBuf::from)
        .unwrap_or(executable);
    crate::platform::validate_local_path(&executable)?;
    let path = executable
        .to_str()
        .ok_or_else(|| error("helper_path", "管理器路径不是 UTF-8。"))?;
    if path.chars().any(char::is_control) {
        return Err(error("helper_path", "管理器路径含不支持的控制字符。"));
    }
    // Without control characters JSON and Lua share quote/backslash escaping.
    let literal =
        serde_json::to_string(path).map_err(|_| error("helper_path", "无法编码管理器路径。"))?;
    Ok(vec![Payload {
        path: crate::config::relative!("scripts/mpvkit.lua").into(),
        bytes: include_str!("../../resources/scripts/mpvkit.lua")
            .replace("__MPVKIT_HELPER__", &literal)
            .into_bytes(),
    }])
}

pub fn directory(root: &Path) -> Result<std::path::PathBuf> {
    Ok(root.into())
}
pub fn read(root: &Path) -> Result<Settings> {
    portable::read_state(root).map(|state| state.settings)
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
                .find(|p| p.path == crate::config::relative!("mpv.conf"))
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
                    .find(|p| p.path == crate::config::relative!("script-opts/thumbfast.conf"))
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
                    .find(|p| p.path == crate::config::relative!("mpv.conf"))
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
    fn native_playback_defaults_are_generated() {
        let root = std::env::temp_dir().join(format!("mpvkit-defaults-{}", deploy::operation_id()));
        let settings = Settings::default();
        let files = generate(&root, &root.join("mpv.exe"), &settings).unwrap();
        let conf = String::from_utf8(
            files
                .iter()
                .find(|f| f.path == crate::config::relative!("mpv.conf"))
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
            "autofit=75%x75%\n",
            "auto-window-resize=no\n",
            "autocreate-playlist=filter\n",
            "directory-filter-types=video,audio\n",
            "directory-mode=ignore\n",
            "sub-auto=exact\n",
            "sub-file-paths-append=sub\n",
            "sub-file-paths-append=subs\n",
            "sub-file-paths-append=subtitle\n",
            "sub-file-paths-append=subtitles\n",
        ] {
            assert!(conf.contains(expected), "{expected}");
        }
        for file in &files {
            if file.path.ends_with(".conf") || file.path.ends_with(".lua") {
                assert!(!String::from_utf8_lossy(&file.bytes).contains("screenshot"));
            }
        }
        assert!(!root.exists());
        let mut settings = settings;
        settings.playback.auto_playlist = false;
        settings.playback.keep_open = true;
        let files = generate(&root, &root.join("mpv.exe"), &settings).unwrap();
        let conf = &files.iter().find(|f| f.path == "mpv.conf").unwrap().bytes;
        let conf = String::from_utf8_lossy(conf);
        assert!(conf.contains("autocreate-playlist=no\n"));
        assert!(conf.contains("keep-open=yes\n"));
    }
    #[test]
    fn fixed_languages_and_hdr_parameters_are_generated() {
        let root = std::env::temp_dir().join(format!("mpvkit-config-{}", deploy::operation_id()));
        let mut edited = Settings::default();
        for (hdr, expected) in [
            (Hdr::Auto, "target-trc=auto"),
            (Hdr::On, "target-trc=pq"),
            (
                Hdr::Off,
                if cfg!(target_os = "linux") {
                    "target-trc=srgb"
                } else {
                    "target-trc=bt.1886"
                },
            ),
        ] {
            edited.playback.hdr = hdr;
            let files = generate(&root, &root.join("mpv.exe"), &edited).unwrap();
            let conf = String::from_utf8(
                files
                    .into_iter()
                    .find(|f| f.path == crate::config::relative!("mpv.conf"))
                    .unwrap()
                    .bytes,
            )
            .unwrap();
            assert!(conf.contains(expected));
            assert!(conf.contains(&format!("alang={DEFAULT_LANGUAGES}\n")));
            assert!(conf.contains(&format!("slang={DEFAULT_LANGUAGES}\n")));
        }
        let mut invalid = serde_json::to_value(&edited).unwrap();
        invalid["playback"]["hdr"] = "invalid".into();
        assert!(serde_json::from_value::<Settings>(invalid).is_err());
    }
}
