//! Optional integration checks against the installed Arch MPV and a real Wayland session.
use crate::{components, config, deploy, launch, platform, setup};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
    time::Duration,
};

#[test]
fn linux_configuration_has_no_windows_runtime_paths() {
    let root = std::env::temp_dir().join(format!("mpvkit-linux-config-{}", deploy::operation_id()));
    let files = config::generate(&root, Path::new("/usr/bin/mpv"), &Default::default()).unwrap();
    let content = |path: &str| {
        String::from_utf8(files.iter().find(|p| p.path == path).unwrap().bytes.clone()).unwrap()
    };
    let conf = content(crate::config::relative!("mpv.conf"));
    for line in [
        "gpu-api=vulkan\n",
        "gpu-context=waylandvk\n",
        "hwdec=auto-safe\n",
        "sub-fonts-dir=~~/fonts\n",
        "osd-fonts-dir=~~/fonts\n",
        "autofit=75%x75%\n",
        "auto-window-resize=no\n",
        "autocreate-playlist=filter\n",
        "directory-filter-types=video,audio\n",
        "directory-mode=ignore\n",
        "sub-auto=exact\n",
        "sub-file-paths-append=sub\n",
        "sub-file-paths-append=Subtitles\n",
        "sub-file-paths-append=SUBS\n",
    ] {
        assert!(conf.contains(line), "{line}");
    }
    assert!(!conf.contains("d3d") && !conf.contains("C:\\"));
    assert_eq!(
        content("script-opts/uosc.conf"),
        include_str!("../../resources/uosc.conf")
    );
    assert!(
        content(crate::config::relative!("script-opts/thumbfast.conf"))
            .contains("mpv_path=/usr/bin/mpv\n")
    );
    assert!(
        content(crate::config::relative!("scripts/mpvkit.lua"))
            .contains(".mpvkit/launcher/mpvkit'")
    );
    assert!(!files.iter().any(|p| p.path.ends_with(".exe")
        || p.path.ends_with(".dll")
        || p.path.ends_with("fonts.conf")));
}

fn request(socket: &Path, command: serde_json::Value) -> serde_json::Value {
    let mut stream = UnixStream::connect(socket).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    writeln!(
        stream,
        "{}",
        serde_json::json!({"command":command,"request_id":1})
    )
    .unwrap();
    for line in BufReader::new(stream).lines() {
        let value: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        if value["request_id"] == 1 {
            assert_eq!(value["error"], "success", "{command}");
            return value["data"].clone();
        }
    }
    panic!("IPC response missing")
}
struct PlaybackGuard(PathBuf);
impl Drop for PlaybackGuard {
    fn drop(&mut self) {
        if let Ok(mut stream) = UnixStream::connect(&self.0) {
            let _ = writeln!(stream, "{}", serde_json::json!({"command":["quit"]}));
        }
    }
}
#[track_caller]
fn wait(mut condition: impl FnMut() -> bool) {
    for _ in 0..150 {
        if condition() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("condition timed out")
}
#[test]
#[ignore = "requires fixed local assets, built native executable, synthetic video and Wayland"]
fn arch_wayland_install_playback_and_uninstall() {
    assert_eq!(std::env::var("MPVKIT_TEST_MODE").as_deref(), Ok("1"));
    let root = platform::app_root().unwrap();
    assert!(!root.exists(), "use a fresh isolated MPVKIT_TEST_ROOT");
    let media =
        PathBuf::from(std::env::var_os("MPVKIT_TEST_VIDEO").expect("local synthetic video"));
    platform::validate_local_path(&media).unwrap();
    let player_hash = deploy::file_hash(Path::new("/usr/bin/mpv")).unwrap();
    let initial = setup::status(&root).unwrap();
    assert!(!initial.integration_enabled);
    let system_player = initial.components.iter().find(|c| c.id == "mpv").unwrap();
    assert!(system_player.ready);
    assert!(
        system_player
            .version
            .as_deref()
            .unwrap()
            .starts_with("mpv ")
    );
    assert!(initial.player_ready && !initial.components_ready && initial.generation.is_none());
    let draft = config::Settings::default();
    let preview = config::portable::preview(&root, &draft, false).unwrap();
    assert_eq!(preview.files.len(), 4);
    assert!(preview.files[0].after.contains("gpu-context=waylandvk"));
    assert_eq!(
        setup::prepare_draft(&root, draft.clone(), false, true, &preview.fingerprint)
            .err()
            .unwrap()
            .code,
        "components_required"
    );
    assert!(!root.exists());
    let plan =
        components::prepare_install(&root, false, false, &AtomicBool::new(false), &mut |_| {})
            .unwrap();
    assert!(!root.exists());
    deploy::execute(plan).unwrap();
    let core = setup::status(&root).unwrap();
    assert!(core.player_ready && core.components_ready && !core.settings_ready);
    assert!(!core.integration.registered);
    let helper = root.clone().join(platform::UOSC_HELPER);
    let held = helper.with_extension("held");
    fs::rename(&helper, &held).unwrap();
    assert_eq!(
        setup::prepare_draft(&root, draft.clone(), false, true, &preview.fingerprint)
            .err()
            .unwrap()
            .code,
        "components_required"
    );
    fs::rename(held, helper).unwrap();
    fs::write(root.join("notes.txt"), "unmanaged user file").unwrap();
    let plan = setup::prepare_draft(&root, draft, false, false, &preview.fingerprint).unwrap();
    assert!(plan.preview().integration.is_none());
    setup::execute(plan).unwrap();
    assert!(!setup::status(&root).unwrap().integration.registered);
    let entry = root.join(".mpvkit/test-applications/io.github.mpvkit.desktop");
    assert!(!entry.exists());
    let config_file = root.join(crate::config::relative!("mpv.conf"));
    let socket = root.join("mpv.sock");
    let text = fs::read_to_string(&config_file).unwrap();
    fs::write(
        &config_file,
        format!("{text}\ninput-ipc-server={}\n", socket.display()),
    )
    .unwrap();
    // Match the system desktop entry: no config-dir, wrapper or MPV_HOME override.
    let test_root = platform::default_root().unwrap();
    let mut player = platform::command(Path::new("/usr/bin/mpv"))
        .env_remove("MPV_HOME")
        .env("HOME", test_root.join("home"))
        .env("XDG_CONFIG_HOME", test_root.join("config"))
        .env("XDG_CACHE_HOME", test_root.join("cache"))
        .env("XDG_STATE_HOME", test_root.join("state"))
        .arg("--player-operation-mode=pseudo-gui")
        .arg("--no-terminal")
        .arg("--")
        .arg(&media)
        .spawn()
        .unwrap();
    wait(|| socket.exists());
    let _playback = PlaybackGuard(socket.clone());
    let pid = request(&socket, serde_json::json!(["get_property", "pid"]));
    assert_eq!(
        request(&socket, serde_json::json!(["get_property", "current-vo"])),
        "gpu-next"
    );
    assert_eq!(
        request(
            &socket,
            serde_json::json!(["get_property", "current-gpu-context"])
        ),
        "waylandvk"
    );
    request(&socket, serde_json::json!(["set_property", "volume", 50]));
    request(&socket, serde_json::json!(["keypress", "UP"]));
    assert_eq!(
        request(&socket, serde_json::json!(["get_property", "volume"])).as_f64(),
        Some(60.0)
    );
    let scripts = serde_json::json!({
        "uoscBindings":request(&socket,serde_json::json!(["get_property","input-bindings"])),
        "mpvkit":request(&socket,serde_json::json!(["get_property","user-data/mpvkit"]))
    });
    assert!(
        scripts["uoscBindings"]
            .to_string()
            .contains("uosc/__keybinding")
    );
    assert_eq!(scripts["mpvkit"]["profile"], "balanced");
    request(
        &socket,
        serde_json::json!(["script-message-to", "mpvkit", "profile", "high-quality"]),
    );
    wait(|| request(&socket, serde_json::json!(["get_property", "scale"])) == "ewa_lanczossharp");
    request(
        &socket,
        serde_json::json!(["script-message-to", "mpvkit", "save"]),
    );
    wait(|| config::read(&root).is_ok_and(|s| s.profile == config::Profile::HighQuality));
    request(
        &socket,
        serde_json::json!(["script-message-to", "thumbfast", "thumb", "2", "20", "20"]),
    );
    let thumbnail = PathBuf::from(format!("/tmp/thumbfast.out{}.bgra", pid.as_u64().unwrap()));
    wait(|| thumbnail.is_file());
    assert!(fs::metadata(&thumbnail).unwrap().len() > 0);
    let snapshot = request(
        &socket,
        serde_json::json!(["get_property", "video-out-params"]),
    );
    let evidence = serde_json::json!({"vo":"gpu-next","context":"waylandvk","scripts":scripts,"thumbnailBytes":fs::metadata(&thumbnail).unwrap().len(),"video":snapshot,"hwdec":request(&socket,serde_json::json!(["get_property","hwdec-current"]))});
    fs::write(
        root.join("playback-evidence.json"),
        serde_json::to_vec_pretty(&evidence).unwrap(),
    )
    .unwrap();
    request(&socket, serde_json::json!(["seek", 4, "absolute+exact"]));
    wait(|| {
        request(&socket, serde_json::json!(["get_property", "time-pos"]))
            .as_f64()
            .is_some_and(|t| t >= 4.0)
    });
    request(&socket, serde_json::json!(["quit"]));
    wait(|| UnixStream::connect(&socket).is_err());
    assert!(player.wait().unwrap().success());
    // Optional Open with registration is independent of native configuration loading.
    setup::execute(setup::prepare_intent(&root, true).unwrap()).unwrap();
    assert!(setup::status(&root).unwrap().integration.valid);
    assert!(
        platform::command(Path::new("/usr/bin/desktop-file-validate"))
            .arg(&entry)
            .status()
            .unwrap()
            .success()
    );
    let watch = root.join(crate::config::relative!("watch_later"));
    assert!(fs::read_dir(&watch).unwrap().next().is_some());
    let mut helper = platform::command(&config::helper_path(&root))
        .arg("--open")
        .arg(&media)
        .spawn()
        .unwrap();
    assert!(helper.wait().unwrap().success());
    wait(|| UnixStream::connect(&socket).is_ok());
    assert!(
        request(&socket, serde_json::json!(["get_property", "time-pos"]))
            .as_f64()
            .unwrap()
            >= 4.0
    );
    assert_eq!(
        request(&socket, serde_json::json!(["get_property", "volume"])).as_f64(),
        Some(100.0)
    );
    assert_eq!(
        request(&socket, serde_json::json!(["get_property", "scale"])),
        "ewa_lanczossharp"
    );
    request(&socket, serde_json::json!(["quit"]));
    wait(|| UnixStream::connect(&socket).is_err());
    // All profile/HDR modes must parse in the actual target MPV; this does not certify HDR hardware output.
    for profile in [
        config::Profile::PowerSaving,
        config::Profile::Balanced,
        config::Profile::HighQuality,
    ] {
        for hdr in [config::Hdr::Auto, config::Hdr::On, config::Hdr::Off] {
            let settings = config::Settings {
                profile,
                playback: config::Playback {
                    hdr,
                    ..Default::default()
                },
                ..Default::default()
            };
            let preview = config::portable::preview(&root, &settings, false).unwrap();
            setup::execute(
                setup::prepare_draft(&root, settings, false, true, &preview.fingerprint).unwrap(),
            )
            .unwrap();
            let args = vec![
                "--config=yes".into(),
                format!("--config-dir={}", root.clone().display()).into(),
                "--load-scripts=no".into(),
                "--frames=2".into(),
                "--audio=no".into(),
                media.as_os_str().to_owned(),
            ];
            crate::mpv::capture(Path::new("/usr/bin/mpv"), &args, Duration::from_secs(8)).unwrap();
        }
    }
    assert!(launch::prepare_managed(&root, media.to_str().unwrap()).is_ok());
    setup::execute(setup::prepare_uninstall(&root).unwrap()).unwrap();
    assert!(!config_file.exists());
    assert!(deploy::status(&root).unwrap().manifest.is_none());
    assert!(watch.exists());
    for name in ["staging", "recovery", "pending.json"] {
        assert!(!root.join(".mpvkit").join(name).exists());
    }
    let removed = setup::status(&root).unwrap();
    assert!(removed.player_ready && !removed.components_ready);
    let system_player_after = removed.components.iter().find(|c| c.id == "mpv").unwrap();
    assert!(system_player_after.ready);
    assert_eq!(system_player.version, system_player_after.version);
    assert!(!entry.exists());
    assert!(!config::helper_path(&root).exists());
    assert_eq!(
        fs::read_to_string(root.join("notes.txt")).unwrap(),
        "unmanaged user file"
    );
    assert_eq!(
        deploy::file_hash(Path::new("/usr/bin/mpv")).unwrap(),
        player_hash
    );
    println!("Linux integration evidence: {}", root.display());
}
