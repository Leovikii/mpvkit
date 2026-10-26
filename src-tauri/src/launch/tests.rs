use super::*;
pub(super) fn prepare(root: &Path, executable: &str, media: &str) -> Result<LaunchPlan> {
    let mut plan = new_plan(
        root,
        mpv::inspect(&mpv::executable_path(executable)?)?,
        media,
    )?;
    plan.preview.arguments = mpv::launch_args(&plan.session_directory, &plan.preview.media)
        .into_iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
    Ok(plan)
}
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
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut file = loop {
        match OpenOptions::new()
            .append(true)
            .open(&plan.preview.mpv.executable)
        {
            Ok(file) => break file,
            Err(e) if e.raw_os_error() == Some(32) && Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(20))
            }
            Err(e) => panic!("fixture could not be changed: {e}"),
        }
    };
    file.write_all(b"changed").unwrap();
    drop(file);
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
    drop(crate::deploy::acquire(&plan.root).unwrap());
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
    assert!(crate::logs::prepare(&sandbox.path("managed")).is_ok());
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
    let files = crate::config::generate(
        &root,
        &root.join("mpv.exe"),
        &crate::config::Settings::default(),
    )
    .unwrap();
    fs::create_dir_all(&root).unwrap();
    fs::copy(&player, root.join("mpv.exe")).unwrap();
    crate::deploy::execute(
        crate::deploy::prepare_install(
            &root,
            files,
            false,
            std::collections::BTreeMap::from([("mpv".into(), "fixture".into())]),
        )
        .unwrap(),
    )
    .unwrap();
    let media = sandbox.media();
    fs::write(root.join("mpv.conf"), "volume=17\n").unwrap();
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
            .all(|a| !a.starts_with("--include=") && !a.starts_with("--scale="))
    );
    assert_eq!(plan.preview.config_directory, root.clone());
    fs::write(root.join("mpv.conf"), "changed").unwrap();
    assert_eq!(execute(plan).unwrap_err().code, "stale_preview");
    fs::write(root.join("scripts/mpvkit.lua"), "modified").unwrap();
    assert_eq!(
        prepare_managed(&root, media.to_str().unwrap())
            .err()
            .unwrap()
            .code,
        "file_conflict"
    );
}

#[test]
fn missing_optional_components_fall_back_without_loading_scripts_but_modified_components_stop() {
    let sandbox = Sandbox::new();
    let player = sandbox.player("");
    let root = sandbox.path("managed");
    let files = crate::config::generate(&root, &root.join("mpv.exe"), &Default::default()).unwrap();
    fs::create_dir_all(&root).unwrap();
    fs::copy(&player, root.join("mpv.exe")).unwrap();
    crate::deploy::execute(
        crate::deploy::prepare_install(
            &root,
            files,
            false,
            std::collections::BTreeMap::from([("mpv".into(), "fixture".into())]),
        )
        .unwrap(),
    )
    .unwrap();
    let media = sandbox.media();
    fs::remove_file(root.join("scripts/mpvkit.lua")).unwrap();
    let plan = prepare_managed(&root, media.to_str().unwrap()).unwrap();
    assert_eq!(plan.preview.config_directory, root.clone());
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
fn system_core_is_not_owned_and_package_updates_invalidate_pending_launches() {
    let sandbox = Sandbox::new();
    let root = sandbox.path("config");
    let player = mpv::fixture_player(&root);
    let files = crate::config::generate(&root, &player, &Default::default()).unwrap();
    crate::deploy::execute(
        crate::deploy::prepare_install(
            &root,
            files,
            false,
            std::collections::BTreeMap::from([("mpv".into(), "fixture".into())]),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(
        !crate::deploy::status(&root)
            .unwrap()
            .manifest
            .unwrap()
            .files
            .iter()
            .any(|f| f.path == "mpv.exe")
    );
    let media = sandbox.media();
    let plan = prepare_managed(&root, media.to_str().unwrap()).unwrap();
    let mut bytes = fs::read(&player).unwrap();
    bytes.push(0);
    fs::write(&player, bytes).unwrap();
    assert_eq!(execute(plan).unwrap_err().code, "stale_preview");
    prepare_managed(&root, media.to_str().unwrap()).unwrap();
    let diagnosis = crate::diagnostics::prepare(&root, &sandbox.path("report.json")).unwrap();
    assert_eq!(
        diagnosis.report.actual_player_version.as_deref(),
        Some("mpv 0.40.0-fixture")
    );
    assert_eq!(diagnosis.report.changed_or_unavailable_files, 0);
}
