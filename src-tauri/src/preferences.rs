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
    let path = root.join(".mpvkit/playback.json");
    crate::platform::validate_local_path(&path)?;
    if !path.exists() {
        return Ok(None);
    }
    let bytes = deploy::read_bytes(&path)?;
    if bytes.len() > 4096 {
        return Err(error("preference_invalid", "播放偏好记录过大，已保留。"));
    }
    let value: PlaybackPreference = serde_json::from_slice(&bytes)
        .map_err(|_| error("preference_invalid", "播放偏好记录损坏或字段未知，已保留。"))?;
    if value.schema_version != 1 || value.owner != "MpvKit" {
        return Err(error(
            "preference_invalid",
            "播放偏好版本或归属未知，已保留。",
        ));
    }
    Ok(Some(value))
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
    let _lock = deploy::acquire(root)?;
    let status = deploy::status(root)?;
    let manifest = status
        .manifest
        .ok_or_else(|| error("not_installed", "环境尚未安装。"))?;
    if status.recovery_required || manifest.installation_id != generation {
        return Err(error(
            "preference_stale",
            "安装或设置已变化，请重新启动播放器后保存。",
        ));
    }
    let previous = read(root)?;
    if previous.as_ref().map_or(0, |p| p.revision) != expected_revision {
        return Err(error(
            "preference_conflict",
            "其他播放器已保存偏好；本次选择仍生效，请重试保存。",
        ));
    }
    let revision = expected_revision
        .checked_add(1)
        .ok_or_else(|| error("preference_limit", "播放偏好修订数已达上限。"))?;
    let value = PlaybackPreference {
        schema_version: 1,
        owner: "MpvKit".into(),
        revision,
        generation: generation.into(),
        profile,
    };
    deploy::atomic_write(
        &root.join(".mpvkit/playback.json"),
        &serde_json::to_vec_pretty(&value)
            .map_err(|_| error("preference_encode", "无法编码播放偏好。"))?,
    )?;
    Ok(value)
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
        save(Path::new(&args[2]), profile, revision, &args[5])
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
    fn root() -> std::path::PathBuf {
        let root =
            std::env::temp_dir().join(format!("mpvkit-preference-{}", deploy::operation_id()));
        deploy::execute(
            deploy::prepare_install(&root, vec![], false, None, Default::default()).unwrap(),
        )
        .unwrap();
        root
    }
    #[test]
    fn revisions_generation_and_unknown_state_protect_concurrent_saves() {
        let root = root();
        let generation = deploy::status(&root)
            .unwrap()
            .manifest
            .unwrap()
            .installation_id;
        let first = save(&root, Profile::HighQuality, 0, &generation).unwrap();
        assert_eq!(first.revision, 1);
        assert_eq!(
            save(&root, Profile::Balanced, 0, &generation)
                .unwrap_err()
                .code,
            "preference_conflict"
        );
        assert_eq!(
            save(&root, Profile::Balanced, 1, "old").unwrap_err().code,
            "preference_stale"
        );
        save(&root, Profile::PowerSaving, 1, &generation).unwrap();
        let _lock = deploy::acquire(&root).unwrap();
        assert_eq!(
            save(&root, Profile::Balanced, 2, &generation)
                .unwrap_err()
                .code,
            "target_locked"
        );
        drop(_lock);
        std::fs::write(
            root.join(".mpvkit/playback.json"),
            br#"{"schemaVersion":99,"owner":"Other"}"#,
        )
        .unwrap();
        assert_eq!(
            save(&root, Profile::Balanced, 2, &generation)
                .unwrap_err()
                .code,
            "preference_invalid"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn two_real_processes_cannot_overwrite_the_same_preference_revision() {
        if let Ok(data) = std::env::var("MPVKIT_PREFERENCE_CHILD") {
            let data: serde_json::Value = serde_json::from_str(&data).unwrap();
            let result = save(
                Path::new(data["root"].as_str().unwrap()),
                Profile::HighQuality,
                0,
                data["generation"].as_str().unwrap(),
            );
            println!(
                "MpvKitSave={}",
                if result.is_ok() { "saved" } else { "rejected" }
            );
            return;
        }
        let root = root();
        let generation = deploy::status(&root)
            .unwrap()
            .manifest
            .unwrap()
            .installation_id;
        let data = serde_json::json!({"root": root, "generation": generation}).to_string();
        let executable = std::env::current_exe().unwrap();
        let start = || {
            crate::platform::command(&executable).args(["--exact", "preferences::tests::two_real_processes_cannot_overwrite_the_same_preference_revision", "--nocapture"]).env("MPVKIT_PREFERENCE_CHILD", &data).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn().unwrap()
        };
        let first = start();
        let second = start();
        let first = first.wait_with_output().unwrap();
        let second = second.wait_with_output().unwrap();
        assert!(first.status.success() && second.status.success());
        let saved = [first.stdout, second.stdout]
            .iter()
            .filter(|v| String::from_utf8_lossy(v).contains("MpvKitSave=saved"))
            .count();
        assert_eq!(saved, 1);
        assert_eq!(read(&root).unwrap().unwrap().revision, 1);
        std::fs::remove_dir_all(root).unwrap();
    }
}
