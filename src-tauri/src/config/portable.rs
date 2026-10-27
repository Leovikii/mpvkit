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
    pub manual_config: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            schema_version: 1,
            owner: "MpvKit".into(),
            revision: 0,
            settings: Settings::default(),
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
pub fn player_path(root: &Path) -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        crate::platform::windows_mpv::player_path(root)
    }
    #[cfg(target_os = "linux")]
    {
        let _ = root;
        Ok(PathBuf::from("/usr/bin/mpv"))
    }
}
pub fn editable(path: &str) -> bool {
    path.strip_prefix(crate::config::relative!(""))
        .is_some_and(|name| {
            ["mpv.conf", "input.conf", "fonts.conf"].contains(&name)
                || name.starts_with("script-opts/") && name.ends_with(".conf")
        })
}
const PREVIEW_FILES: [&str; 4] = [
    crate::config::relative!("mpv.conf"),
    crate::config::relative!("input.conf"),
    crate::config::relative!("script-opts/thumbfast.conf"),
    crate::config::relative!("script-opts/uosc.conf"),
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
    let payloads = generate_inner(root, &player_path(root)?, settings, replace, false)?;
    let manifest = deploy::status(root)?.manifest;
    let mut files = Vec::new();
    for name in PREVIEW_FILES {
        let path = root.join(name);
        let exists = deploy::file_hash(&path)?.is_some();
        if exists
            && !replace
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
        "autocreate-playlist",
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
        "script-opts-append=mpvkit-profile={}\n",
        settings.profile.id()
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
        if !reset
            && existing.is_some()
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
        crate::config::relative!("script-opts/uosc.conf"),
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
    prepare_portable_mode(root, settings, replace, false)
}
pub fn reset(root: &Path) -> Result<deploy::DeploymentPlan> {
    prepare_portable_mode(root, Settings::default(), true, true)
}
fn prepare_portable_mode(
    root: &Path,
    settings: Settings,
    replace: bool,
    reset: bool,
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
    // Observe, but never own or roll back, files maintained by the package manager.
    watched.push(player_path(root)?);
    let hashes = watched
        .iter()
        .map(|p| deploy::file_hash(p))
        .collect::<Result<Vec<_>>>()?;
    let mut files = generate_inner(root, &player_path(root)?, &settings, replace, reset)?;
    for p in &files {
        if !replace
            && !m.files.iter().any(|f| f.path == p.path)
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
#[cfg(test)]
mod tests {
    use super::*;
    fn install(root: &Path) {
        let mut files = generate(root, &player_path(root).unwrap(), &Settings::default()).unwrap();
        #[cfg(target_os = "windows")]
        crate::mpv::fixture_player(root);
        for p in crate::components::ASSETS
            .iter()
            .flat_map(|a| crate::components::required_paths(a.id))
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
        assert!(player_path(&root).unwrap().exists());
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
            .replace("autofit=75%x75%", "autofit=60%x60%")
            .replace("auto-window-resize=no", "auto-window-resize=yes")
            .replace(
                "sub-file-paths-append=subtitles",
                "sub-file-paths-append=custom-subs",
            )
            .replace("directory-mode=ignore", "directory-mode=recursive")
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
        assert!(after.contains("autofit=60%x60%\n"));
        assert!(after.contains("auto-window-resize=yes\n"));
        assert!(after.contains("sub-file-paths-append=custom-subs\n"));
        assert!(after.contains("directory-mode=recursive\n"));
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
            reset_plan
                .preview
                .configuration_diffs
                .iter()
                .find(|d| d.path == crate::config::relative!("mpv.conf"))
                .unwrap()
                .before,
            "changed after preview"
        );
        deploy::execute(reset_plan).unwrap();
        assert!(std::fs::read_to_string(&path).unwrap().contains(BEGIN));
        assert!(!modified(&root).unwrap());
        assert_eq!(read(&root).unwrap(), Settings::default());
        for name in [
            "input.conf",
            "script-opts/uosc.conf",
            "script-opts/thumbfast.conf",
        ] {
            let expected = preview(&root, &Settings::default(), false).unwrap();
            let file = expected.files.iter().find(|f| f.path == name).unwrap();
            assert_eq!(
                std::fs::read_to_string(root.join(name)).unwrap(),
                file.after
            );
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn unmanaged_configuration_requires_concrete_preview_and_fresh_approval() {
        let root =
            std::env::temp_dir().join(format!("mpvkit-reinstall-{}", deploy::operation_id()));
        install(&root);
        let manifest = deploy::status(&root).unwrap().manifest.unwrap();
        let files = manifest
            .files
            .iter()
            .filter(|f| !editable(&f.path))
            .map(|f| Payload {
                path: f.path.clone(),
                bytes: deploy::read_bytes(&root.join(&f.path)).unwrap(),
            })
            .collect();
        let path = root.join("mpv.conf");
        std::fs::write(&path, "volume=11\n").unwrap();
        deploy::execute(deploy::prepare_install(&root, files, false, manifest.components).unwrap())
            .unwrap();
        assert_eq!(
            preview(&root, &Settings::default(), false)
                .err()
                .unwrap()
                .code,
            "file_conflict"
        );
        let shown = preview(&root, &Settings::default(), true).unwrap();
        assert_eq!(
            shown
                .files
                .iter()
                .find(|f| f.path == "mpv.conf")
                .unwrap()
                .before,
            "volume=11\n"
        );
        let pending = prepare_portable(&root, Settings::default(), true).unwrap();
        std::fs::write(&path, "late edit\n").unwrap();
        assert_eq!(deploy::execute(pending).unwrap_err().code, "stale_preview");
        deploy::execute(prepare_portable(&root, Settings::default(), true).unwrap()).unwrap();
        assert!(std::fs::read_to_string(path).unwrap().contains(BEGIN));
        for name in ["recovery", "staging", "previous.json"] {
            assert!(!root.join(".mpvkit").join(name).exists());
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn uosc_configuration_is_previewed_guarded_and_preserved() {
        let root = std::env::temp_dir().join(format!("mpvkit-uosc-{}", deploy::operation_id()));
        install(&root);
        let name = crate::config::relative!("script-opts/uosc.conf");
        let path = root.join(name);
        let settings = Settings::default();
        let original = preview(&root, &settings, false).unwrap();
        let pending = prepare_portable(&root, settings.clone(), false).unwrap();
        let edited = "# My control layout\ncontrols=menu,space,play-pause,space,fullscreen\n";
        std::fs::write(&path, edited).unwrap();
        assert_ne!(original.fingerprint, fingerprint(&root).unwrap());
        assert_eq!(deploy::execute(pending).unwrap_err().code, "stale_preview");
        let retained = preview(&root, &settings, false).unwrap();
        let file = retained.files.iter().find(|f| f.path == name).unwrap();
        assert!(file.exists && file.retained);
        assert_eq!(file.after, edited);
        deploy::execute(prepare_portable(&root, settings, false).unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), edited);
        std::fs::remove_file(path).unwrap();
        assert!(!ready(&root).unwrap());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn unified_profile_saves_are_generation_safe_and_reset_to_defaults() {
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
        deploy::execute(reset(&root).unwrap()).unwrap();
        assert_eq!(read(&root).unwrap().profile, Profile::Balanced);
        assert!(crate::setup::status(&root).unwrap().settings_ready);
        assert!(!root.join(".mpvkit/setup.json").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
