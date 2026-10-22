use crate::{
    config::Profile,
    deploy,
    error::{AppError, Result},
};
use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaybackPreference {
    pub schema_version: u32,
    pub owner: String,
    pub revision: u64,
    pub generation: String,
    pub profile: Profile,
}
fn error(code: &'static str, message: &'static str) -> AppError {
    AppError::new(code, message, "preferences")
}
pub fn read(root: &Path) -> Result<Option<PlaybackPreference>> {
    let state = crate::config::portable::read_state(root)?;
    let m = deploy::status(root)?
        .manifest
        .ok_or_else(|| error("not_installed", "环境尚未安装。"))?;
    Ok(Some(PlaybackPreference {
        schema_version: 1,
        owner: "MpvKit".into(),
        revision: state.revision,
        generation: m.installation_id,
        profile: state.settings.profile,
    }))
}
pub fn save(
    root: &Path,
    profile: Profile,
    expected_revision: u64,
    generation: &str,
) -> Result<PlaybackPreference> {
    if deploy::status(root)?.manifest.is_none() {
        return Err(error("not_installed", "环境尚未安装。"));
    }
    let saved = read(root)?.ok_or_else(|| error("not_installed", "环境尚未安装。"))?;
    if saved.generation != generation || saved.revision != expected_revision {
        return Err(error(
            "preference_stale",
            "设置已变化，请重新打开播放器后保存。",
        ));
    }
    let mut settings = crate::config::read(root)?;
    settings.profile = profile;
    let plan = crate::config::prepare_portable(root, settings, false)?;
    if deploy::status(root)?
        .manifest
        .is_none_or(|m| m.installation_id != generation)
    {
        return Err(error(
            "preference_stale",
            "安装或设置已变化，请重新打开播放器。",
        ));
    }
    deploy::execute(plan)?;
    read(root)?.ok_or_else(|| error("preference_invalid", "设置记录不可用。"))
}

pub fn helper(args: &[String]) -> i32 {
    let result = (|| {
        if args.len() != 6 {
            return Err(error("helper_arguments", "播放偏好保存参数无效。"));
        }
        let profile: Profile = serde_json::from_value(serde_json::Value::String(args[3].clone()))
            .map_err(|_| error("helper_arguments", "档位无效。"))?;
        let revision = args[4]
            .parse::<u64>()
            .map_err(|_| error("helper_arguments", "修订数无效。"))?;
        let root = crate::platform::app_root()?;
        if Path::new(&args[2]) != root {
            return Err(error("fixed_root", "只能保存当前播放器配置。"));
        }
        save(&root, profile, revision, &args[5])
    })();
    match result {
        Ok(value) => {
            println!(
                "{}",
                serde_json::to_string(&value).expect("preference JSON")
            );
            0
        }
        Err(e) => {
            println!("{}", serde_json::to_string(&e).expect("error JSON"));
            1
        }
    }
}
