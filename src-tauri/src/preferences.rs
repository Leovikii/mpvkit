use crate::{
    config::{Hdr, Profile, Settings, WhenFinished},
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
    pub hdr: Hdr,
    pub autoplay: bool,
    pub when_finished: WhenFinished,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(
    tag = "item",
    content = "value",
    rename_all = "kebab-case",
    deny_unknown_fields
)]
pub enum Update {
    Profile(Profile),
    Hdr(Hdr),
    Autoplay(bool),
    WhenFinished(WhenFinished),
}
impl Update {
    fn apply(self, settings: &mut Settings) {
        match self {
            Self::Profile(value) => settings.profile = value,
            Self::Hdr(value) => settings.hdr = value,
            Self::Autoplay(value) => settings.autoplay = value,
            Self::WhenFinished(value) => settings.when_finished = value,
        }
    }
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
        hdr: state.settings.hdr,
        autoplay: state.settings.autoplay,
        when_finished: state.settings.when_finished,
    }))
}
pub fn save(
    root: &Path,
    update: Update,
    expected_revision: u64,
    generation: &str,
) -> Result<PlaybackPreference> {
    let state_path = root.join(".mpvkit/settings.json");
    let manifest_path = root.join(".mpvkit/install.json");
    let state_hash = deploy::file_hash(&state_path)?;
    let manifest_hash = deploy::file_hash(&manifest_path)?;
    let saved = read(root)?.ok_or_else(|| error("not_installed", "环境尚未安装。"))?;
    if saved.generation != generation || saved.revision != expected_revision {
        return Err(error(
            "preference_stale",
            "设置已变化，请重新打开播放器后保存。",
        ));
    }
    let mut settings = crate::config::read(root)?;
    update.apply(&mut settings);
    let mut plan = crate::config::prepare_portable(root, settings, false)?;
    plan.guard(state_path, state_hash);
    plan.guard(manifest_path, manifest_hash);
    if deploy::status(root)?
        .manifest
        .is_none_or(|m| m.installation_id != generation)
    {
        return Err(error(
            "preference_stale",
            "安装或设置已变化，请重新打开播放器。",
        ));
    }
    let applied = deploy::execute(plan)?;
    let result = read(root)?.ok_or_else(|| error("preference_invalid", "设置记录不可用。"))?;
    if applied
        .manifest
        .is_none_or(|m| m.installation_id != result.generation)
    {
        return Err(error("preference_stale", "设置已变化，请重新打开播放器。"));
    }
    Ok(result)
}

pub fn helper(args: &[String]) -> i32 {
    let result = (|| {
        if args.len() != 6 {
            return Err(error("helper_arguments", "播放偏好保存参数无效。"));
        }
        let update: Update = serde_json::from_str(&args[3])
            .map_err(|_| error("helper_arguments", "播放设置项目或值无效。"))?;
        let revision = args[4]
            .parse::<u64>()
            .map_err(|_| error("helper_arguments", "修订数无效。"))?;
        let root = crate::platform::app_root()?;
        if Path::new(&args[2]) != root {
            return Err(error("fixed_root", "只能保存当前播放器配置。"));
        }
        save(&root, update, revision, &args[5])
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn updates_only_accept_controlled_items_and_values() {
        for input in [
            r#"{"item":"volume","value":50}"#,
            r#"{"item":"profile","value":"injected"}"#,
            r#"{"item":"hdr","value":"pq"}"#,
            r#"{"item":"autoplay","value":"yes"}"#,
            r#"{"item":"when-finished","value":"always"}"#,
            r#"{"item":"hdr","value":"on","path":"mpv.conf"}"#,
        ] {
            assert!(serde_json::from_str::<Update>(input).is_err(), "{input}");
        }
    }
}
