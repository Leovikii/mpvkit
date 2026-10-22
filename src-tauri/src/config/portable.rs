use super::*;
use std::path::PathBuf;

const BEGIN: &str = "# >>> MpvKit playback settings\n";
const END: &str = "# <<< MpvKit playback settings\n";
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct State {
    pub schema_version: u32,
    pub owner: String,
    pub revision: u64,
    pub settings: Settings,
    pub confirmed: bool,
    pub completed: bool,
    pub manual_config: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            schema_version: 1,
            owner: "MpvKit".into(),
            revision: 0,
            settings: Settings::default(),
            confirmed: false,
            completed: false,
            manual_config: false,
        }
    }
}
pub fn read_state(root: &Path) -> Result<State> {
    let path = root.join(".mpvkit/settings.json");
    if deploy::file_hash(&path)?.is_none() {
        return Ok(State::default());
    }
    let bytes = deploy::read_bytes(&path)?;
    if bytes.len() > 65536 {
        return Err(error("settings_limit", "设置记录过大，已保留。"));
    }
    let state: State = serde_json::from_slice(&bytes)
        .map_err(|_| error("settings_invalid", "设置记录损坏，已保留。"))?;
    if state.schema_version != 1 || state.owner != "MpvKit" {
        return Err(error("settings_version", "设置记录版本或归属未知。"));
    }
    state.settings.validate()?;
    Ok(state)
}
pub fn player_path(root: &Path, manifest: &deploy::InstallManifest) -> PathBuf {
    manifest
        .external_mpv
        .clone()
        .unwrap_or_else(|| root.join("mpv.exe"))
}
pub fn helper_path(root: &Path) -> PathBuf {
    root.join(".mpvkit/launcher/mpvkit.exe")
}
pub fn editable(path: &str) -> bool {
    path.strip_prefix("portable_config/").is_some_and(|name| {
        ["mpv.conf", "input.conf", "fonts.conf"].contains(&name)
            || name.starts_with("script-opts/") && name.ends_with(".conf")
    })
}
fn text(bytes: &[u8]) -> Result<String> {
    if bytes.len() > 1024 * 1024 {
        return Err(error("config_limit", "配置过大，未自动修改。"));
    }
    String::from_utf8(bytes.to_vec())
        .map(|s| s.replace("\r\n", "\n"))
        .map_err(|_| error("config_encoding", "配置不是 UTF-8，未自动修改。"))
}
fn gui_line(line: &str) -> bool {
    let key = line.split('=').next().unwrap_or("");
    [
        "keep-open",
        "screenshot-directory",
        "screenshot-format",
        "hwdec",
        "target-colorspace-hint",
        "target-trc",
        "target-prim",
        "alang",
        "slang",
    ]
    .contains(&key)
}
fn block(raw: &str, settings: &Settings) -> String {
    let mut out = String::from(BEGIN);
    for (name, options) in profiles() {
        out.push_str(&format!("[mpvkit-{name}]\n"));
        for (k, v) in options {
            out.push_str(&format!("{k}={v}\n"));
        }
    }
    out.push_str(&format!(
        "[default]\nprofile=mpvkit-{}\n",
        settings.profile.id()
    ));
    for line in raw.lines().filter(|l| gui_line(l)) {
        out.push_str(line);
        out.push('\n');
    }
    out.push_str(&format!(
        "script-opts-append=mpvkit-profile={}\nscript-opts-append=mpvkit-screenshot_subtitles={}\n",
        settings.profile.id(),
        if settings.playback.screenshot_subtitles {
            "yes"
        } else {
            "no"
        }
    ));
    out.push_str(END);
    out
}
fn bounds(s: &str) -> Result<(usize, usize)> {
    if s.matches(BEGIN).count() != 1 || s.matches(END).count() != 1 {
        return Err(error(
            "config_structure",
            "配置管理标记已改变，未自动重写；请保留文件并恢复标记后重试。",
        ));
    }
    let start = s.find(BEGIN).unwrap();
    let end = s.find(END).unwrap() + END.len();
    if end <= start || s[..start].lines().any(|l| l.trim_start().starts_with('[')) {
        return Err(error(
            "config_structure",
            "配置段作用域已改变，未自动合并。",
        ));
    }
    Ok((start, end))
}
pub fn generate(root: &Path, player: &Path, settings: &Settings) -> Result<Vec<Payload>> {
    generate_inner(root, player, settings, false, false)
}
fn generate_inner(
    root: &Path,
    player: &Path,
    settings: &Settings,
    replace: bool,
    reset: bool,
) -> Result<Vec<Payload>> {
    let mut files = base_files(player, settings)?;
    let raw = text(
        &files
            .iter()
            .find(|p| p.path == "portable_config/mpv.conf")
            .unwrap()
            .bytes,
    )?;
    let next_block = block(&raw, settings);
    let existing = deploy::status(root)?.manifest;
    let conf = if let Some(manifest) = existing.as_ref().filter(|_| !reset) {
        let old = text(&deploy::read_bytes(&root.join("portable_config/mpv.conf"))?)?;
        let (start, end) = bounds(&old)?;
        let previous = read_state(root)?;
        let old_files = base_files(player, &previous.settings)?;
        let expected = block(
            &text(
                &old_files
                    .iter()
                    .find(|p| p.path == "portable_config/mpv.conf")
                    .unwrap()
                    .bytes,
            )?,
            &previous.settings,
        );
        if old[start..end] != expected && !replace {
            return Err(error(
                "config_manual_conflict",
                "播放设置已手动修改；请勾选高级选项中的替换手动设置，再查看差异。",
            ));
        }
        if !manifest
            .files
            .iter()
            .any(|f| f.path == "portable_config/mpv.conf")
        {
            return Err(error("config_owner", "主配置没有归属记录。"));
        }
        format!("{}{}{}", &old[..start], next_block, &old[end..])
    } else {
        let profile_keys = profiles()[settings.profile.id()]
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        let defaults = raw
            .lines()
            .filter(|l| {
                !l.starts_with('#')
                    && !gui_line(l)
                    && !profile_keys.contains(&l.split('=').next().unwrap_or("").to_string())
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("# MPV settings: edit this file directly.\n{defaults}\n\n{next_block}")
    };
    let mut other_manual = false;
    for p in &mut files {
        if p.path == "portable_config/mpv.conf" {
            p.bytes = conf.as_bytes().to_vec();
        }
        if existing.is_some()
            && editable(&p.path)
            && p.path != "portable_config/mpv.conf"
            && deploy::file_hash(&root.join(&p.path))?.is_some()
        {
            let retained = deploy::read_bytes(&root.join(&p.path))?;
            other_manual |= retained != p.bytes;
            p.bytes = retained;
        }
    }
    let mut state = read_state(root)?;
    state.manual_config = other_manual || !reset && (state.manual_config || modified(root)?);
    state.settings = settings.clone();
    state.revision = state
        .revision
        .checked_add(1)
        .ok_or_else(|| error("settings_limit", "设置修订数已达上限。"))?;
    files
        .iter_mut()
        .find(|p| p.path == ".mpvkit/settings.json")
        .unwrap()
        .bytes = serde_json::to_vec_pretty(&state)
        .map_err(|_| error("settings_encode", "无法保存设置。"))?;
    Ok(files)
}
pub fn modified(root: &Path) -> Result<bool> {
    if read_state(root)?.manual_config {
        return Ok(true);
    }
    let Some(m) = deploy::status(root)?.manifest else {
        return Ok(false);
    };
    for f in &m.files {
        if editable(&f.path)
            && deploy::file_hash(&root.join(&f.path))?.as_deref() != Some(&f.sha256)
        {
            return Ok(true);
        }
    }
    Ok(false)
}
fn retain_files(root: &Path, m: &deploy::InstallManifest, files: &mut Vec<Payload>) -> Result<()> {
    for f in &m.files {
        if files.iter().any(|p| p.path == f.path) {
            continue;
        }
        let bytes = deploy::read_bytes(&root.join(&f.path))?;
        if !editable(&f.path) && deploy::hash_bytes(&bytes) != f.sha256 {
            return Err(error("file_conflict", "受管理程序或脚本已修改，请先检查。"));
        }
        files.push(Payload {
            path: f.path.clone(),
            bytes,
        });
    }
    Ok(())
}
pub fn prepare_portable(
    root: &Path,
    settings: Settings,
    replace: bool,
) -> Result<deploy::DeploymentPlan> {
    prepare_portable_mode(root, settings, replace, false)
}
pub fn reset(root: &Path) -> Result<deploy::DeploymentPlan> {
    prepare_portable_mode(root, read(root)?, true, true)
}
fn prepare_portable_mode(
    root: &Path,
    settings: Settings,
    replace: bool,
    reset: bool,
) -> Result<deploy::DeploymentPlan> {
    settings.validate()?;
    validate_output(&settings)?;
    let m = deploy::status(root)?
        .manifest
        .ok_or_else(|| error("not_installed", "请先安装播放器。"))?;
    let watched = [
        root.join("portable_config/mpv.conf"),
        root.join(".mpvkit/settings.json"),
    ];
    let hashes = watched
        .iter()
        .map(|p| deploy::file_hash(p))
        .collect::<Result<Vec<_>>>()?;
    let mut files = generate_inner(root, &player_path(root, &m), &settings, replace, reset)?;
    for p in &files {
        if !m.files.iter().any(|f| f.path == p.path)
            && deploy::file_hash(&root.join(&p.path))?.is_some()
        {
            return Err(error(
                "file_conflict",
                "新增配置或启动文件已存在，未自动覆盖。",
            ));
        }
    }
    retain_files(root, &m, &mut files)?;
    let mut plan = deploy::prepare_install(root, files, true, m.external_mpv, m.components)?;
    for (path, hash) in watched.into_iter().zip(hashes) {
        plan.guard(path, hash);
    }
    Ok(plan)
}
pub fn save_state(root: &Path, state: State, generation: &str) -> Result<()> {
    let m = deploy::status(root)?
        .manifest
        .ok_or_else(|| error("not_installed", "请先安装播放器。"))?;
    if m.installation_id != generation {
        return Err(error("setup_changed", "安装状态已变化，请重新检查。"));
    }
    let mut files = vec![Payload {
        path: ".mpvkit/settings.json".into(),
        bytes: serde_json::to_vec_pretty(&state)
            .map_err(|_| error("settings_encode", "无法保存设置。"))?,
    }];
    retain_files(root, &m, &mut files)?;
    deploy::execute(deploy::prepare_install(
        root,
        files,
        false,
        m.external_mpv,
        m.components,
    )?)?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn install(root: &Path) {
        let mut files = generate(root, &root.join("mpv.exe"), &Settings::default()).unwrap();
        files.push(Payload {
            path: "mpv.exe".into(),
            bytes: b"fixture".to_vec(),
        });
        for p in [
            "scripts/uosc/main.lua",
            "scripts/thumbfast.lua",
            "fonts/LXGWWenKaiScreen.ttf",
        ] {
            files.push(Payload {
                path: format!("portable_config/{p}"),
                bytes: b"-- fixture".to_vec(),
            });
        }
        deploy::execute(
            deploy::prepare_install(root, files, false, None, Default::default()).unwrap(),
        )
        .unwrap();
    }
    #[test]
    fn portable_configuration_preserves_edits_and_previews_only_explicit_replacement() {
        let root = std::env::temp_dir().join(format!("mpvkit-portable-{}", deploy::operation_id()));
        install(&root);
        assert!(root.join("mpv.exe").exists());
        assert!(!root.join("state").exists());
        assert!(!root.join("portable_config/mpvkit-profiles.json").exists());
        let path = root.join("portable_config/mpv.conf");
        let before = std::fs::read_to_string(&path).unwrap();
        assert!(before.contains("[mpvkit-balanced]"));
        let manual = before.replace("save-position-on-quit=yes", "save-position-on-quit=no")
            + "\n# custom\nvolume=23\n";
        std::fs::write(&path, &manual).unwrap();
        std::fs::write(
            root.join("portable_config/input.conf"),
            "SPACE cycle pause\n",
        )
        .unwrap();
        let mut settings = read(&root).unwrap();
        settings.playback.audio_languages = vec!["ja".into()];
        let plan = prepare_portable(&root, settings.clone(), false).unwrap();
        assert!(plan.preview.conflicts.is_empty());
        assert!(!plan.preview.configuration_diffs.is_empty());
        deploy::execute(plan).unwrap();
        let after = std::fs::read_to_string(&path).unwrap();
        assert!(after.contains("save-position-on-quit=no"));
        assert!(after.ends_with("# custom\nvolume=23\n"));
        assert!(after.contains("alang=ja"));
        assert_eq!(
            std::fs::read_to_string(root.join("portable_config/input.conf")).unwrap(),
            "SPACE cycle pause\n"
        );
        std::fs::write(&path, after.replace("alang=ja", "alang=en")).unwrap();
        assert_eq!(
            prepare_portable(&root, settings.clone(), false)
                .err()
                .unwrap()
                .code,
            "config_manual_conflict"
        );
        let plan = prepare_portable(&root, settings, true).unwrap();
        assert!(plan.preview.conflicts.is_empty());
        std::fs::write(&path, "changed after preview").unwrap();
        assert!(deploy::execute(plan).is_err());
        let reset_plan = reset(&root).unwrap();
        assert!(reset_plan.preview.conflicts.is_empty());
        assert_eq!(
            reset_plan.preview.configuration_diffs[0].before,
            "changed after preview"
        );
        deploy::execute(reset_plan).unwrap();
        assert!(std::fs::read_to_string(&path).unwrap().contains(BEGIN));
        assert!(
            modified(&root).unwrap(),
            "retained input.conf still differs from defaults"
        );
        deploy::execute(deploy::prepare_restore(&root).unwrap()).unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "changed after preview"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn unified_profile_saves_are_generation_safe_and_restore_as_one_transaction() {
        let root = std::env::temp_dir().join(format!("mpvkit-portable-{}", deploy::operation_id()));
        install(&root);
        let m = deploy::status(&root).unwrap().manifest.unwrap();
        let state = read_state(&root).unwrap();
        let saved = crate::preferences::save(
            &root,
            Profile::HighQuality,
            state.revision,
            &m.installation_id,
        )
        .unwrap();
        assert_eq!(read(&root).unwrap().profile, Profile::HighQuality);
        assert!(!root.join(".mpvkit/playback.json").exists());
        assert!(
            std::fs::read_to_string(root.join("portable_config/mpv.conf"))
                .unwrap()
                .contains("profile=mpvkit-high-quality")
        );
        assert!(
            crate::preferences::save(&root, Profile::Balanced, state.revision, &m.installation_id)
                .is_err()
        );
        assert!(saved.revision > state.revision);
        deploy::execute(deploy::prepare_restore(&root).unwrap()).unwrap();
        assert_eq!(read(&root).unwrap().profile, Profile::Balanced);
        let m = deploy::status(&root).unwrap().manifest.unwrap();
        assert!(
            crate::setup::confirm(&root, &m.installation_id, false)
                .unwrap()
                .settings_ready
        );
        assert!(!root.join(".mpvkit/setup.json").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
