const DEFAULT_LANGUAGES: &str = "zh-Hans-CN,zh-Hans-SG,zh-Hant-TW,zh-Hant-HK,zh-Hant-MO,zh-Hans,zh-Hant,zh-CN,zh-SG,zh-TW,zh-HK,zh-MO,zh,cmn-Hans-CN,cmn-Hant-TW,cmn-Hans,cmn-Hant,cmn-CN,cmn-TW,cmn,yue-Hant-HK,yue-Hant,yue-HK,yue,chs,cht,ja-JP,ja,en-US,en-GB,en-AU,en-CA,en,es-ES,es-MX,es-419,es-AR,es";
use crate::{
    deploy::{self, Payload},
    error::{AppError, Result},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

#[cfg(target_os = "windows")]
macro_rules! relative {
    ($name:literal) => {
        concat!("portable_config/", $name)
    };
}
#[cfg(target_os = "linux")]
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
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Playback {
    pub hdr: Hdr,
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
        if let Some(directory) = &p.screenshot_directory {
            if directory.is_empty()
                || directory.len() > 4096
                || directory.chars().any(char::is_control)
            {
                return Err(error("screenshot_path", "截图目录无效。"));
            }
            crate::platform::validate_local_path(Path::new(directory))?;
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
            path: ".mpvkit/settings.json".into(),
            bytes: Vec::new(),
        },
    ];
    files.extend(runtime_files()?);
    Ok(files)
}

pub fn runtime_files() -> Result<Vec<Payload>> {
    let mut files = vec![Payload {
        path: crate::config::relative!("scripts/mpvkit.lua").into(),
        bytes: include_str!("../../resources/scripts/mpvkit.lua")
            .replace(".mpvkit/launcher/mpvkit.exe", crate::platform::HELPER)
            .replace(
                "local root = utils.split_path(config_dir:gsub('[/\\\\]+$', ''))",
                if cfg!(target_os = "linux") {
                    "local root = config_dir:gsub('[/\\\\]+$', '')"
                } else {
                    "local root = utils.split_path(config_dir:gsub('[/\\\\]+$', ''))"
                },
            )
            .into_bytes(),
    }];
    let executable =
        std::env::current_exe().map_err(|_| error("helper_path", "无法取得启动入口。"))?;
    #[cfg(test)]
    let executable = std::env::var_os("MPVKIT_TEST_EXECUTABLE")
        .map(std::path::PathBuf::from)
        .unwrap_or(executable);
    files.push(Payload {
        path: crate::platform::HELPER.into(),
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
            crate::components::APP_NOTICES.as_bytes(),
        ),
        (
            "third-party/app-dependency-inventory.json",
            crate::components::APP_INVENTORY.as_bytes(),
        ),
        ("third-party/app-distribution-materials.json", {
            #[cfg(target_os = "windows")]
            {
                include_bytes!("../../resources/app-distribution-materials.json").as_slice()
            }
            #[cfg(target_os = "linux")]
            {
                include_bytes!("../../resources/app-distribution-materials-linux.json").as_slice()
            }
        }),
        (
            "third-party/RUST-COPYRIGHT-library.html",
            include_bytes!("../../resources/licenses/rust-library-COPYRIGHT.txt").as_slice(),
        ),
        #[cfg(target_os = "windows")]
        (
            "third-party/WEBVIEW2-SDK-LICENSE.txt",
            include_bytes!("../../resources/licenses/webview2-sdk-LICENSE.txt").as_slice(),
        ),
        #[cfg(target_os = "windows")]
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
    #[cfg(target_os = "windows")]
    return Ok(root.join("portable_config"));
    #[cfg(target_os = "linux")]
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
    fn playback_defaults_and_screenshot_paths_cannot_inject_options() {
        let root = std::env::temp_dir().join(format!("mpvkit-defaults-{}", deploy::operation_id()));
        let mut settings = Settings::default();
        settings.playback.screenshot_directory =
            Some(root.join("截图 # 空格").to_string_lossy().into_owned());
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
