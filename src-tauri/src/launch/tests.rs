use super::*;
use std::sync::OnceLock;

pub(super) struct Sandbox(PathBuf);
impl Sandbox {
    pub(super) fn new() -> Self {
        let value = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "mpvkit-test-{value}-{}",
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
    pub(super) fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
    pub(super) fn player(&self, mode: &str) -> PathBuf {
        static FIXTURE: OnceLock<Vec<u8>> = OnceLock::new();
        let source = FIXTURE.get_or_init(|| {
            let directory = Sandbox::new();
            let source = directory.path("fixture.rs");
            fs::write(&source, include_str!("../../tests/fixtures/mpv.rs")).unwrap();
            let binary = directory.path("mpv.exe");
            let result = std::process::Command::new("rustc")
                .arg("--edition=2024")
                .arg("--crate-name=mpv_fixture")
                .arg(source)
                .arg("-o")
                .arg(&binary)
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            fs::read(binary).unwrap()
        });
        let binary = self.path("mpv.exe");
        fs::write(&binary, source).unwrap();
        fs::write(self.path("mode"), mode).unwrap();
        binary
    }
    fn media(&self) -> PathBuf {
        let media = self.path("中文 空格 & $ 影片.mkv");
        fs::write(&media, "fixture media").unwrap();
        media
    }
    fn plan(&self) -> LaunchPlan {
        let binary = self.player("");
        let media = self.media();
        prepare(
            &self.path("managed"),
            binary.to_str().unwrap(),
            media.to_str().unwrap(),
        )
        .unwrap()
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn preview_is_read_only_and_preserves_unicode_arguments() {
    let sandbox = Sandbox::new();
    let plan = sandbox.plan();
    assert!(!sandbox.path("managed").exists());
    assert_eq!(plan.preview.mpv.version, "mpv 0.40.0-fixture");
    assert_eq!(plan.preview.mpv.ownership, "external");
    assert_eq!(
        plan.preview.arguments.last().unwrap(),
        sandbox.media().to_str().unwrap()
    );
    assert_eq!(plan.preview.arguments[4], "--");
}

#[test]
fn changed_media_invalidates_preview_without_writes() {
    let sandbox = Sandbox::new();
    let plan = sandbox.plan();
    fs::write(&plan.preview.media, "changed length after preview").unwrap();
    assert_eq!(execute(plan).unwrap_err().code, "stale_preview");
    assert!(!sandbox.path("managed").exists());
}

#[test]
fn changed_executable_invalidates_preview() {
    let sandbox = Sandbox::new();
    let plan = sandbox.plan();
    OpenOptions::new()
        .append(true)
        .open(&plan.preview.mpv.executable)
        .unwrap()
        .write_all(b"changed")
        .unwrap();
    assert_eq!(execute(plan).unwrap_err().code, "stale_preview");
    assert!(!sandbox.path("managed").exists());
}

#[test]
fn expired_preview_does_not_create_session() {
    let sandbox = Sandbox::new();
    let mut plan = sandbox.plan();
    plan.created = Instant::now() - Duration::from_secs(301);
    assert_eq!(execute(plan).unwrap_err().code, "stale_preview");
    assert!(!sandbox.path("managed").exists());
}

#[test]
fn existing_session_is_never_overwritten() {
    let sandbox = Sandbox::new();
    let plan = sandbox.plan();
    fs::create_dir_all(&plan.preview.config_directory).unwrap();
    let marker = plan.preview.config_directory.join("user.txt");
    fs::write(&marker, "keep me").unwrap();
    assert_eq!(execute(plan).unwrap_err().code, "session_write");
    assert_eq!(fs::read_to_string(marker).unwrap(), "keep me");
}

#[test]
fn confirmed_launch_keeps_arguments_separate_and_records_ownership() {
    let sandbox = Sandbox::new();
    let plan = sandbox.plan();
    let directory = plan.preview.config_directory.clone();
    let arguments = plan.preview.arguments.clone();
    let start = Instant::now();
    let result = execute(plan).unwrap();
    assert!(result.process_id > 0);
    assert!(start.elapsed() < Duration::from_secs(2));
    assert_eq!(
        fs::read_to_string(sandbox.path("argv.txt"))
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        arguments
    );
    let record: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("session.json")).unwrap()).unwrap();
    assert_eq!(record["schemaVersion"], 1);
    assert_eq!(record["owner"], "MpvKit");
    assert!(
        fs::read_to_string(directory.join("session.log"))
            .unwrap()
            .contains("status=started")
    );
    thread::sleep(Duration::from_secs(1));
}

#[test]
fn missing_capabilities_are_rejected() {
    let sandbox = Sandbox::new();
    assert_eq!(
        mpv::inspect(&sandbox.player("missing-capability"))
            .unwrap_err()
            .code,
        "unsupported_mpv"
    );
}

#[test]
fn query_timeout_is_bounded() {
    let sandbox = Sandbox::new();
    let binary = sandbox.player("timeout");
    let start = Instant::now();
    let error = mpv::capture(&binary, &[], Duration::from_millis(100)).unwrap_err();
    assert_eq!(error.code, "probe_timeout");
    assert!(start.elapsed() < Duration::from_secs(2));
}

#[test]
fn excessive_output_and_failed_exit_are_rejected() {
    let sandbox = Sandbox::new();
    let binary = sandbox.player("flood");
    assert_eq!(
        mpv::capture(&binary, &[], Duration::from_secs(3))
            .unwrap_err()
            .code,
        "probe_failed"
    );
    fs::write(sandbox.path("mode"), "failure").unwrap();
    assert_eq!(
        mpv::capture(&binary, &[], Duration::from_secs(3))
            .unwrap_err()
            .code,
        "probe_failed"
    );
}

#[test]
fn network_device_relative_and_parent_paths_are_rejected_before_io() {
    for path in [
        r"\\server\RDM-X\mpv.exe",
        r"\\?\C:\mpv.exe",
        r"\\.\pipe\mpv",
        r"C:mpv.exe",
        r"mpv.exe",
        r"C:\folder\..\mpv.exe",
        r"C:\file:stream",
    ] {
        assert!(
            platform::validate_local_path(Path::new(path)).is_err(),
            "{path}"
        );
    }
}

#[test]
fn directories_and_other_executables_are_rejected() {
    let sandbox = Sandbox::new();
    assert_eq!(
        mpv::file_stamp(&sandbox.0).unwrap_err().code,
        "file_required"
    );
    assert_eq!(
        mpv::executable_path(r"C:\Windows\System32\cmd.exe")
            .unwrap_err()
            .code,
        "mpv_required"
    );
}

#[test]
fn managed_launch_checks_owned_files_and_user_additions_before_execution() {
    let sandbox = Sandbox::new();
    let player = sandbox.player("");
    let root = sandbox.path("managed");
    let files =
        crate::config::generate(&root, &player, &crate::config::Settings::default()).unwrap();
    crate::deploy::execute(
        crate::deploy::prepare_install(
            &root,
            files,
            false,
            Some(player),
            std::collections::BTreeMap::new(),
        )
        .unwrap(),
    )
    .unwrap();
    let media = sandbox.media();
    fs::write(root.join("config/user.conf"), "volume=17\n").unwrap();
    let plan = prepare_managed(&root, media.to_str().unwrap()).unwrap();
    assert!(plan.preview.arguments.iter().any(|a| a == "--config=yes"));
    assert!(
        plan.preview
            .arguments
            .iter()
            .any(|a| a == "--load-scripts=yes")
    );
    assert!(
        plan.preview
            .arguments
            .iter()
            .any(|a| a.starts_with("--include="))
    );
    assert_eq!(plan.preview.config_directory, root.join("config"));
    fs::write(root.join("config/user.conf"), "changed").unwrap();
    assert_eq!(execute(plan).unwrap_err().code, "stale_preview");
    fs::write(root.join("config/mpv.conf"), "manual").unwrap();
    assert_eq!(
        prepare_managed(&root, media.to_str().unwrap())
            .err()
            .unwrap()
            .code,
        "file_conflict"
    );
}

#[test]
fn saved_profile_is_applied_before_user_config_and_old_generation_is_ignored() {
    let sandbox = Sandbox::new();
    let player = sandbox.player("");
    let root = sandbox.path("managed");
    let files =
        crate::config::generate(&root, &player, &crate::config::Settings::default()).unwrap();
    crate::deploy::execute(
        crate::deploy::prepare_install(&root, files, false, Some(player), Default::default())
            .unwrap(),
    )
    .unwrap();
    let generation = crate::deploy::status(&root)
        .unwrap()
        .manifest
        .unwrap()
        .installation_id;
    crate::preferences::save(&root, crate::config::Profile::HighQuality, 0, &generation).unwrap();
    fs::write(root.join("config/user.conf"), "scale=bilinear\n").unwrap();
    let media = sandbox.media();
    let plan = prepare_managed(&root, media.to_str().unwrap()).unwrap();
    assert_eq!(plan.generation.as_deref(), Some(generation.as_str()));
    let saved = plan
        .preview
        .arguments
        .iter()
        .position(|a| a == "--scale=ewa_lanczossharp")
        .unwrap();
    let user = plan
        .preview
        .arguments
        .iter()
        .position(|a| a.starts_with("--include="))
        .unwrap();
    assert!(saved < user);
    crate::preferences::save(&root, crate::config::Profile::Balanced, 1, &generation).unwrap();
    assert_eq!(execute(plan).unwrap_err().code, "stale_preview");
    crate::deploy::execute(
        crate::config::prepare(
            &root,
            crate::config::Settings {
                profile: crate::config::Profile::PowerSaving,
                ..Default::default()
            },
        )
        .unwrap(),
    )
    .unwrap();
    let plan = prepare_managed(&root, media.to_str().unwrap()).unwrap();
    assert!(
        !plan
            .preview
            .arguments
            .iter()
            .any(|a| a.starts_with("--scale=")),
        "old generation preference must not override GUI setting"
    );
}

#[test]
fn missing_optional_components_fall_back_without_loading_scripts_but_modified_components_stop() {
    let sandbox = Sandbox::new();
    let player = sandbox.player("");
    let root = sandbox.path("managed");
    let files = crate::config::generate_at(&root, &player, &Default::default(), ".").unwrap();
    crate::deploy::execute(
        crate::deploy::prepare_install_at(
            &root,
            files,
            false,
            Some(player),
            Default::default(),
            ".",
        )
        .unwrap(),
    )
    .unwrap();
    let media = sandbox.media();
    fs::remove_file(root.join("scripts/mpvkit.lua")).unwrap();
    let plan = prepare_managed(&root, media.to_str().unwrap()).unwrap();
    assert_eq!(plan.preview.config_directory, root.join("."));
    assert!(plan.preview.arguments.contains(&"--load-scripts=no".into()));
    assert!(
        plan.preview
            .changes
            .iter()
            .any(|s| s.contains("组件文件缺失"))
    );
    fs::write(root.join("scripts/mpvkit.lua"), "modified").unwrap();
    assert_eq!(execute(plan).unwrap_err().code, "stale_preview");
    assert_eq!(
        prepare_managed(&root, media.to_str().unwrap())
            .err()
            .unwrap()
            .code,
        "file_conflict"
    );
}

#[test]
fn legacy_player_can_launch_native_but_cannot_deploy_or_load_unsupported_managed_configuration() {
    let sandbox = Sandbox::new();
    let player = sandbox.player("missing-managed-capability");
    let media = sandbox.media();
    let root = sandbox.path("managed");
    assert!(prepare(&root, player.to_str().unwrap(), media.to_str().unwrap()).is_ok());
    let result = crate::components::prepare_install(
        &root,
        Some(player.to_str().unwrap()),
        false,
        &std::sync::atomic::AtomicBool::new(false),
        &mut |_| {},
    );
    assert_eq!(result.err().unwrap().code, "managed_mpv_unsupported");
    assert!(
        !root.exists(),
        "reject unsupported options before download or deployment"
    );
}

#[test]
fn managed_runtime_is_verified_before_launch_and_diagnostic_probes() {
    for changed in ["player", "library", "missing-library", "unowned-player"] {
        let sandbox = Sandbox::new();
        let player_bytes = fs::read(sandbox.player("")).unwrap();
        let root = sandbox.path("managed");
        let files = vec![
            crate::deploy::Payload {
                path: "runtime/mpv.exe".into(),
                bytes: player_bytes.clone(),
            },
            crate::deploy::Payload {
                path: "runtime/unused.dll".into(),
                bytes: b"original dependency".to_vec(),
            },
            crate::deploy::Payload {
                path: "config/mpv.conf".into(),
                bytes: b"volume=40\n".to_vec(),
            },
        ];
        crate::deploy::execute(
            crate::deploy::prepare_install(&root, files, false, None, Default::default()).unwrap(),
        )
        .unwrap();
        fs::write(root.join("runtime/mode"), "record-probes").unwrap();
        let witness = root.join("runtime/probes.txt");
        let media = sandbox.media();
        prepare_managed(&root, media.to_str().unwrap()).unwrap();
        assert!(
            witness.exists(),
            "valid runtime must permit capability queries"
        );
        fs::remove_file(&witness).unwrap();
        let baseline = crate::diagnostics::prepare(&root, &sandbox.path("baseline.json")).unwrap();
        assert_eq!(
            baseline.report.actual_player_version.as_deref(),
            Some("mpv 0.40.0-fixture")
        );
        assert!(
            witness.exists(),
            "valid runtime must permit diagnostic queries"
        );
        fs::remove_file(&witness).unwrap();
        match changed {
            "player" => {
                let mut changed = player_bytes;
                changed.push(0);
                fs::write(root.join("runtime/mpv.exe"), changed).unwrap();
            }
            "library" => {
                fs::write(root.join("runtime/unused.dll"), b"changed dependency!").unwrap()
            }
            "missing-library" => fs::remove_file(root.join("runtime/unused.dll")).unwrap(),
            "unowned-player" => {
                let path = root.join(".mpvkit/install.json");
                let mut manifest = crate::deploy::status(&root).unwrap().manifest.unwrap();
                manifest.files.retain(|file| file.path != "runtime/mpv.exe");
                fs::write(path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
            }
            _ => unreachable!(),
        }
        let launch = prepare_managed(&root, media.to_str().unwrap());
        let diagnosis = crate::diagnostics::prepare(&root, &sandbox.path("blocked.json")).unwrap();
        assert!(
            !witness.exists(),
            "{changed}: no player process may run before integrity/ownership checks"
        );
        let expected = if changed == "unowned-player" {
            "managed_player_unowned"
        } else {
            "file_conflict"
        };
        assert_eq!(launch.err().unwrap().code, expected);
        assert_eq!(diagnosis.report.player_issue, Some(expected));
        assert!(diagnosis.report.actual_player_version.is_none());
        assert_eq!(
            diagnosis.report.changed_or_unavailable_files,
            usize::from(changed != "unowned-player")
        );
        assert!(
            !sandbox.path("blocked.json").exists(),
            "diagnostic preview remains read-only"
        );
    }
}
