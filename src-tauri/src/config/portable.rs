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
    pub integration_enabled: bool,
    pub manual_config: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            schema_version: 1,
            owner: "MpvKit".into(),
            revision: 0,
            settings: Settings::default(),
            integration_enabled: cfg!(target_os = "windows"),
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
pub fn player_path(root: &Path) -> PathBuf {
    {
        #[cfg(target_os = "windows")]
        {
            root.join("mpv.exe")
        }
        #[cfg(target_os = "linux")]
        {
            let _ = root;
            PathBuf::from("/usr/bin/mpv")
        }
    }
}
pub fn helper_path(root: &Path) -> PathBuf {
    root.join(crate::platform::HELPER)
}
pub fn editable(path: &str) -> bool {
    path.strip_prefix(crate::config::relative!(""))
        .is_some_and(|name| {
            ["mpv.conf", "input.conf", "fonts.conf"].contains(&name)
                || name.starts_with("script-opts/") && name.ends_with(".conf")
        })
}
const PREVIEW_FILES: [&str; 3] = [
    crate::config::relative!("mpv.conf"),
    crate::config::relative!("input.conf"),
    crate::config::relative!("script-opts/thumbfast.conf"),
];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationFile {
    pub path: String,
    pub before: String,
    pub after: String,
    pub modified_at: Option<u64>,
    pub retained: bool,
    pub exists: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationPreview {
    pub fingerprint: String,
    pub files: Vec<ConfigurationFile>,
}
pub fn modified_at(path: &Path) -> Option<u64> {
    std::fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis() as u64)
}
pub fn fingerprint(root: &Path) -> Result<String> {
    let mut bytes = serde_json::to_vec(&read_state(root)?.settings)
        .map_err(|_| error("settings_encode", "无法读取设置。"))?;
    for name in PREVIEW_FILES {
        bytes.extend_from_slice(
            serde_json::to_string(&deploy::file_hash(&root.join(name))?)
                .map_err(|_| error("settings_encode", "无法读取配置。"))?
                .as_bytes(),
        );
    }
    Ok(deploy::hash_bytes(&bytes))
}
// Read-only rendering uses exactly the same merger as the deployment plan.
pub fn preview(root: &Path, settings: &Settings, replace: bool) -> Result<ConfigurationPreview> {
    let fingerprint = fingerprint(root)?;
    let payloads = generate_inner(root, &player_path(root), settings, replace, false)?;
    let manifest = deploy::status(root)?.manifest;
    let mut files = Vec::new();
    for name in PREVIEW_FILES {
        let path = root.join(name);
        let exists = deploy::file_hash(&path)?.is_some();
        if exists
            && !manifest
                .as_ref()
                .is_some_and(|m| m.files.iter().any(|f| f.path == name))
        {
            return Err(error(
                "file_conflict",
                "配置文件已存在且未受管理，未自动覆盖。",
            ));
        }
        let before = if exists {
            text(&deploy::read_bytes(&path)?)?
        } else {
            String::new()
        };
        let after = text(&payloads.iter().find(|p| p.path == name).unwrap().bytes)?;
        files.push(ConfigurationFile {
            path: name.into(),
            retained: exists && before == after,
            exists,
            before,
            after,
            modified_at: modified_at(&path),
        });
    }
    if fingerprint != self::fingerprint(root)? {
        return Err(error("stale_preview", "配置已变化，请重新预览。"));
    }
    Ok(ConfigurationPreview { fingerprint, files })
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
#[cfg(test)]
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
            .find(|p| p.path == crate::config::relative!("mpv.conf"))
            .unwrap()
            .bytes,
    )?;
    let next_block = block(&raw, settings);
    let existing = deploy::status(root)?.manifest;
    let conf = if let Some(manifest) = existing.as_ref().filter(|m| {
        !reset
            && m.files
                .iter()
                .any(|f| f.path == crate::config::relative!("mpv.conf"))
            && root.join(crate::config::relative!("mpv.conf")).exists()
    }) {
        let old = text(&deploy::read_bytes(
            &root.join(crate::config::relative!("mpv.conf")),
        )?)?;
        let (start, end) = bounds(&old)?;
        let previous = read_state(root)?;
        let old_files = base_files(player, &previous.settings)?;
        let expected = block(
            &text(
                &old_files
                    .iter()
                    .find(|p| p.path == crate::config::relative!("mpv.conf"))
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
            .any(|f| f.path == crate::config::relative!("mpv.conf"))
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
        if p.path == crate::config::relative!("mpv.conf") {
            p.bytes = conf.as_bytes().to_vec();
        }
        if existing.is_some()
            && editable(&p.path)
            && p.path != crate::config::relative!("mpv.conf")
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
pub fn ready(root: &Path) -> Result<bool> {
    let Some(m) = deploy::status(root)?.manifest else {
        return Ok(false);
    };
    for name in [
        crate::config::relative!("mpv.conf"),
        crate::config::relative!("input.conf"),
        crate::config::relative!("script-opts/thumbfast.conf"),
        ".mpvkit/settings.json",
    ] {
        if !m.files.iter().any(|f| f.path == name) || deploy::file_hash(&root.join(name))?.is_none()
        {
            return Ok(false);
        }
        if name.ends_with(".conf") {
            text(&deploy::read_bytes(&root.join(name))?)?;
        }
    }
    read_state(root)?;
    Ok(true)
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
pub(crate) fn retain_files(
    root: &Path,
    m: &deploy::InstallManifest,
    files: &mut Vec<Payload>,
) -> Result<()> {
    for f in &m.files {
        if files.iter().any(|p| p.path == f.path) {
            continue;
        }
        if editable(&f.path) && deploy::file_hash(&root.join(&f.path))?.is_none() {
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
    prepare_portable_mode(root, settings, replace, false, None)
}
pub fn reset(root: &Path) -> Result<deploy::DeploymentPlan> {
    prepare_portable_mode(root, read(root)?, true, true, None)
}
pub fn prepare_draft(
    root: &Path,
    settings: Settings,
    replace: bool,
    integration_enabled: bool,
) -> Result<deploy::DeploymentPlan> {
    prepare_portable_mode(root, settings, replace, false, Some(integration_enabled))
}
fn prepare_portable_mode(
    root: &Path,
    settings: Settings,
    replace: bool,
    reset: bool,
    integration_enabled: Option<bool>,
) -> Result<deploy::DeploymentPlan> {
    crate::setup::require_components(root)?;
    settings.validate()?;
    let m = deploy::status(root)?
        .manifest
        .ok_or_else(|| error("not_installed", "请先安装播放器。"))?;
    let mut watched = vec![
        root.join(crate::config::relative!("mpv.conf")),
        root.join(".mpvkit/settings.json"),
    ];
    // Protect retained components and configuration too, including unchanged files.
    watched.extend(m.files.iter().map(|f| root.join(&f.path)));
    let hashes = watched
        .iter()
        .map(|p| deploy::file_hash(p))
        .collect::<Result<Vec<_>>>()?;
    let mut files = generate_inner(root, &player_path(root), &settings, replace, reset)?;
    if let Some(enabled) = integration_enabled {
        let payload = files
            .iter_mut()
            .find(|p| p.path == ".mpvkit/settings.json")
            .unwrap();
        let mut state: State = serde_json::from_slice(&payload.bytes)
            .map_err(|_| error("settings_invalid", "设置记录无效。"))?;
        state.integration_enabled = enabled;
        payload.bytes = serde_json::to_vec_pretty(&state)
            .map_err(|_| error("settings_encode", "无法保存设置。"))?;
    }
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
    let mut plan = deploy::prepare_install(root, files, true, m.components)?;
    for (path, hash) in watched.into_iter().zip(hashes) {
        plan.guard(path, hash);
    }
    Ok(plan)
}
pub fn prepare_intent(root: &Path, enabled: bool) -> Result<deploy::DeploymentPlan> {
    let state_path = root.join(".mpvkit/settings.json");
    let state_hash = deploy::file_hash(&state_path)?;
    let m = deploy::status(root)?
        .manifest
        .ok_or_else(|| error("not_installed", "请先安装播放器。"))?;
    let mut state = read_state(root)?;
    state.integration_enabled = enabled;
    state.manual_config |= modified(root)?;
    let mut files = vec![Payload {
        path: ".mpvkit/settings.json".into(),
        bytes: serde_json::to_vec_pretty(&state)
            .map_err(|_| error("settings_encode", "无法保存设置。"))?,
    }];
    retain_files(root, &m, &mut files)?;
    let mut plan = deploy::prepare_install(root, files, false, m.components)?;
    plan.guard(state_path, state_hash);
    Ok(plan)
}
pub fn prepare_remove_player(root: &Path) -> Result<deploy::DeploymentPlan> {
    let m = deploy::status(root)?
        .manifest
        .ok_or_else(|| error("not_installed", "播放器未安装。"))?;
    let mut files = Vec::new();
    for file in m.files {
        if (editable(&file.path) || file.path == ".mpvkit/settings.json")
            && deploy::file_hash(&root.join(&file.path))?.is_some()
        {
            files.push(Payload {
                bytes: deploy::read_bytes(&root.join(&file.path))?,
                path: file.path,
            });
        }
    }
    deploy::prepare_install(root, files, false, Default::default())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn install(root: &Path) {
        let mut files = generate(root, &player_path(root), &Settings::default()).unwrap();
        files.push(Payload {
            path: "mpv.exe".into(),
            bytes: b"fixture".to_vec(),
        });
        for p in crate::components::ASSETS
            .iter()
            .flat_map(|a| crate::components::required_paths(a.id))
            .filter(|p| **p != "mpv.exe")
        {
            files.push(Payload {
                path: (*p).into(),
                bytes: b"fixture".to_vec(),
            });
        }
        deploy::execute(
            deploy::prepare_install(
                root,
                files,
                false,
                BTreeMap::from([("mpv".into(), "fixture".into())]),
            )
            .unwrap(),
        )
        .unwrap();
    }
    #[test]
    fn portable_configuration_preserves_edits_and_previews_only_explicit_replacement() {
        let root = std::env::temp_dir().join(format!("mpvkit-portable-{}", deploy::operation_id()));
        install(&root);
        assert!(player_path(&root).exists());
        assert!(!root.join("state").exists());
        assert!(
            !root
                .join(crate::config::relative!("mpvkit-profiles.json"))
                .exists()
        );
        let path = root.join(crate::config::relative!("mpv.conf"));
        let before = std::fs::read_to_string(&path).unwrap();
        assert!(before.contains("[mpvkit-balanced]"));
        let manual = before
            .replace("save-position-on-quit=yes", "save-position-on-quit=no")
            .replace(&format!("alang={DEFAULT_LANGUAGES}"), "alang=ja")
            .replace(&format!("slang={DEFAULT_LANGUAGES}\n"), "")
            + "\n# custom\nvolume=23\n";
        std::fs::write(&path, &manual).unwrap();
        std::fs::write(
            root.join(crate::config::relative!("input.conf")),
            "SPACE cycle pause\n",
        )
        .unwrap();
        let mut settings = read(&root).unwrap();
        settings.profile = Profile::HighQuality;
        let plan = prepare_portable(&root, settings.clone(), false).unwrap();
        assert!(plan.preview.conflicts.is_empty());
        assert!(!plan.preview.configuration_diffs.is_empty());
        deploy::execute(plan).unwrap();
        let after = std::fs::read_to_string(&path).unwrap();
        assert!(after.contains("save-position-on-quit=no"));
        assert!(after.ends_with("# custom\nvolume=23\n"));
        assert!(after.contains("alang=ja"));
        assert!(!after.contains("slang="));
        assert_eq!(
            std::fs::read_to_string(root.join(crate::config::relative!("input.conf"))).unwrap(),
            "SPACE cycle pause\n"
        );
        std::fs::write(&path, after.replace("target-trc=auto", "target-trc=pq")).unwrap();
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
            std::fs::read_to_string(root.join(crate::config::relative!("mpv.conf")))
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
        assert!(crate::setup::status(&root).unwrap().settings_ready);
        assert!(!root.join(".mpvkit/setup.json").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
