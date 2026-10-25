use super::*;
#[cfg(target_os = "windows")]
use std::os::windows::fs::OpenOptionsExt;

struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("mpvkit-deploy-{}", operation_id()));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
    fn root(&self) -> PathBuf {
        self.0.join("target")
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn payload(path: &str, content: &str) -> Payload {
    Payload {
        path: path.into(),
        bytes: content.as_bytes().into(),
    }
}
fn plan(root: &Path, content: &str) -> DeploymentPlan {
    prepare_install(
        root,
        vec![payload(crate::config::relative!("mpv.conf"), content)],
        false,
        BTreeMap::new(),
    )
    .unwrap()
}
fn install(root: &Path, content: &str) {
    execute(plan(root, content)).unwrap();
}
fn read(root: &Path, path: &str) -> String {
    fs::read_to_string(root.join(path)).unwrap()
}

#[test]
fn install_is_read_only_until_confirmed_idempotent_and_uninstall_preserves_unknowns() {
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    let first = plan(&root, "volume=40");
    assert!(!root.exists());
    execute(first).unwrap();
    write_new(
        &root.join(crate::config::relative!("user.conf")),
        b"keep user",
    )
    .unwrap();
    let previous = read_optional(&control(&root, "previous.json").unwrap()).unwrap();
    let repeat = plan(&root, "volume=40");
    assert!(repeat.preview.no_changes);
    execute(repeat).unwrap();
    assert_eq!(
        read_optional(&control(&root, "previous.json").unwrap()).unwrap(),
        previous
    );
    execute(prepare_uninstall(&root).unwrap()).unwrap();
    assert!(!root.join(crate::config::relative!("mpv.conf")).exists());
    assert_eq!(
        read(&root, crate::config::relative!("user.conf")),
        "keep user"
    );
    assert!(status(&root).unwrap().manifest.is_none());
}

#[test]
fn unknown_file_requires_explicit_replace_and_original_is_restored() {
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    write_new(
        &root.join(crate::config::relative!("mpv.conf")),
        b"original",
    )
    .unwrap();
    let blocked = plan(&root, "managed");
    assert_eq!(
        blocked.preview.conflicts,
        [crate::config::relative!("mpv.conf")]
    );
    assert_eq!(execute(blocked).unwrap_err().code, "file_conflict");
    let replacement = prepare_install(
        &root,
        vec![payload(crate::config::relative!("mpv.conf"), "managed")],
        true,
        BTreeMap::new(),
    )
    .unwrap();
    execute(replacement).unwrap();
    execute(prepare_uninstall(&root).unwrap()).unwrap();
    assert_eq!(
        read(&root, crate::config::relative!("mpv.conf")),
        "original"
    );
}

#[test]
fn stale_file_manifest_previous_record_and_expired_plans_are_rejected() {
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    let first = plan(&root, "managed");
    write_new(
        &root.join(crate::config::relative!("mpv.conf")),
        b"user appeared",
    )
    .unwrap();
    assert_eq!(execute(first).unwrap_err().code, "stale_preview");
    let other = sandbox.0.join("other");
    install(&other, "one");
    let stale = plan(&other, "two");
    install(&other, "three");
    assert_eq!(execute(stale).unwrap_err().code, "stale_preview");
    let mut expired = plan(&other, "four");
    expired.created = Instant::now() - Duration::from_secs(301);
    assert_eq!(execute(expired).unwrap_err().code, "stale_preview");
    let stale = plan(&other, "four");
    atomic_write(&control(&other, "previous.json").unwrap(), b"changed").unwrap();
    assert_eq!(execute(stale).unwrap_err().code, "stale_preview");
    assert_eq!(read(&other, crate::config::relative!("mpv.conf")), "three");
}

#[test]
fn modified_managed_files_block_update_restore_and_uninstall() {
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    install(&root, "one");
    install(&root, "two");
    fs::write(
        root.join(crate::config::relative!("mpv.conf")),
        "user modification",
    )
    .unwrap();
    assert!(!plan(&root, "three").preview.conflicts.is_empty());
    assert!(
        !prepare_uninstall(&root)
            .unwrap()
            .preview
            .conflicts
            .is_empty()
    );
    assert!(!prepare_restore(&root).unwrap().preview.conflicts.is_empty());
    assert_eq!(
        execute(prepare_uninstall(&root).unwrap()).unwrap_err().code,
        "file_conflict"
    );
    assert_eq!(
        read(&root, crate::config::relative!("mpv.conf")),
        "user modification"
    );
}

#[test]
fn previous_success_can_be_restored_and_unknown_files_are_preserved() {
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    let external = sandbox.0.join("mpv.exe");
    write_new(&external, b"external player").unwrap();
    let first = prepare_install(
        &root,
        vec![payload(crate::config::relative!("mpv.conf"), "one")],
        false,
        BTreeMap::new(),
    )
    .unwrap();
    execute(first).unwrap();
    let second = prepare_install(
        &root,
        vec![payload(crate::config::relative!("mpv.conf"), "two")],
        false,
        BTreeMap::new(),
    )
    .unwrap();
    execute(second).unwrap();
    execute(prepare_restore(&root).unwrap()).unwrap();
    assert_eq!(read(&root, crate::config::relative!("mpv.conf")), "one");
    execute(prepare_uninstall(&root).unwrap()).unwrap();
    assert_eq!(fs::read(external).unwrap(), b"external player");
}

#[test]
fn corrupt_original_or_previous_backup_never_replaces_current_files() {
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    write_new(
        &root.join(crate::config::relative!("mpv.conf")),
        b"original",
    )
    .unwrap();
    execute(
        prepare_install(
            &root,
            vec![payload(crate::config::relative!("mpv.conf"), "one")],
            true,
            BTreeMap::new(),
        )
        .unwrap(),
    )
    .unwrap();
    let entry = status(&root).unwrap().manifest.unwrap().files.remove(0);
    fs::write(
        control(&root, &entry.original_backup.unwrap()).unwrap(),
        b"corrupted",
    )
    .unwrap();
    assert_eq!(
        prepare_uninstall(&root).err().unwrap().code,
        "backup_changed"
    );
    assert_eq!(read(&root, crate::config::relative!("mpv.conf")), "one");
    install(&root, "two");
    let journal: Journal = decode(
        &read_optional(&control(&root, "previous.json").unwrap())
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    fs::write(
        control(&root, journal.changes[0].backup.as_ref().unwrap()).unwrap(),
        b"corrupt",
    )
    .unwrap();
    assert_eq!(prepare_restore(&root).err().unwrap().code, "backup_changed");
    assert_eq!(read(&root, crate::config::relative!("mpv.conf")), "two");
}

#[cfg(target_os = "windows")]
#[test]
fn occupied_file_rolls_back_prior_replacements_and_keeps_last_success() {
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    let initial = vec![payload("a.conf", "old a"), payload("b.conf", "old b")];
    execute(prepare_install(&root, initial, false, BTreeMap::new()).unwrap()).unwrap();
    let manifest = read_optional(&control(&root, "install.json").unwrap()).unwrap();
    let previous = read_optional(&control(&root, "previous.json").unwrap()).unwrap();
    let update = prepare_install(
        &root,
        vec![payload("a.conf", "new a"), payload("b.conf", "new b")],
        false,
        BTreeMap::new(),
    )
    .unwrap();
    let _occupied = OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(root.join("b.conf"))
        .unwrap();
    assert_eq!(execute(update).unwrap_err().code, "replace_failed");
    assert_eq!(read(&root, "a.conf"), "old a");
    assert_eq!(read(&root, "b.conf"), "old b");
    assert_eq!(
        read_optional(&control(&root, "install.json").unwrap()).unwrap(),
        manifest
    );
    assert_eq!(
        read_optional(&control(&root, "previous.json").unwrap()).unwrap(),
        previous
    );
    assert!(!status(&root).unwrap().recovery_required);
}

#[test]
fn interruption_after_manifest_commit_restores_files_and_previous_success_journal() {
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    install(&root, "one");
    install(&root, "two");
    let previous = read_optional(&control(&root, "previous.json").unwrap()).unwrap();
    let update = plan(&root, "three");
    {
        let _lock = acquire(&root).unwrap();
        begin(&update).unwrap();
        apply_change(&root, &update.journal, 0).unwrap();
        set_content(
            &control(&root, "install.json").unwrap(),
            update.journal.new_manifest.as_deref().map(str::as_bytes),
        )
        .unwrap();
        atomic_write(
            &control(&root, "previous.json").unwrap(),
            &serde_json::to_vec_pretty(&update.journal).unwrap(),
        )
        .unwrap();
        // Stop before removing pending.json, just as after process interruption.
    }
    assert!(status(&root).unwrap().recovery_required);
    assert_eq!(
        prepare_uninstall(&root).err().unwrap().code,
        "recovery_required"
    );
    recover(prepare_recovery(&root).unwrap()).unwrap();
    assert_eq!(read(&root, crate::config::relative!("mpv.conf")), "two");
    assert_eq!(
        read_optional(&control(&root, "previous.json").unwrap()).unwrap(),
        previous
    );
    execute(prepare_restore(&root).unwrap()).unwrap();
    assert_eq!(read(&root, crate::config::relative!("mpv.conf")), "one");
}

#[test]
fn recovery_stops_for_user_changes_and_stale_preview_then_can_resume() {
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    install(&root, "one");
    let update = plan(&root, "two");
    {
        let _lock = acquire(&root).unwrap();
        begin(&update).unwrap();
        apply_change(&root, &update.journal, 0).unwrap();
    }
    let recovery = prepare_recovery(&root).unwrap();
    fs::write(
        root.join(crate::config::relative!("mpv.conf")),
        "user change",
    )
    .unwrap();
    assert_eq!(recover(recovery).unwrap_err().code, "stale_preview");
    let blocked = prepare_recovery(&root).unwrap();
    assert_eq!(
        blocked.preview.conflicts,
        [crate::config::relative!("mpv.conf")]
    );
    assert_eq!(recover(blocked).unwrap_err().code, "recovery_conflict");
    assert_eq!(
        read(&root, crate::config::relative!("mpv.conf")),
        "user change"
    );
    assert!(status(&root).unwrap().recovery_required);
    fs::write(root.join(crate::config::relative!("mpv.conf")), "two").unwrap();
    recover(prepare_recovery(&root).unwrap()).unwrap();
    assert_eq!(read(&root, crate::config::relative!("mpv.conf")), "one");
}

#[test]
fn lock_excludes_another_process() {
    const NAME: &str = "deploy::tests::lock_excludes_another_process";
    if let Some(root) = std::env::var_os("MPVKIT_LOCK_TEST_ROOT") {
        assert_eq!(acquire(Path::new(&root)).unwrap_err().code, "target_locked");
        return;
    }
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    let _lock = acquire(&root).unwrap();
    let mut command = platform::command(&std::env::current_exe().unwrap());
    command
        .args(["--exact", NAME, "--nocapture"])
        .env("MPVKIT_LOCK_TEST_ROOT", &root);
    crate::mpv::capture_command(command, Duration::from_secs(5)).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn inherited_descriptor_cannot_extend_completed_operation_lock() {
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    let lock = acquire(&root).unwrap();
    // A duplicate has the same flock lifetime as the descriptor inherited before exec.
    let inherited = lock.0.try_clone().unwrap();
    assert_eq!(acquire(&root).unwrap_err().code, "target_locked");
    drop(lock);
    let next = acquire(&root).unwrap();
    drop(inherited);
    assert_eq!(acquire(&root).unwrap_err().code, "target_locked");
    drop(next);
    assert!(acquire(&root).is_ok());
}

#[test]
fn unknown_schema_unknown_owner_and_unowned_control_directory_are_preserved() {
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    fs::create_dir_all(root.join(CONTROL)).unwrap();
    assert_eq!(status(&root).err().unwrap().code, "ownership_unknown");
    write_new(
        &root.join(CONTROL).join("owner.json"),
        br#"{"owner":"MpvKit","schemaVersion":99}"#,
    )
    .unwrap();
    assert_eq!(status(&root).err().unwrap().code, "incompatible_state");
    assert_eq!(
        read(&root, ".mpvkit/owner.json"),
        r#"{"owner":"MpvKit","schemaVersion":99}"#
    );
    fs::write(
        root.join(CONTROL).join("owner.json"),
        br#"{"owner":"Other","schemaVersion":1}"#,
    )
    .unwrap();
    assert_eq!(status(&root).err().unwrap().code, "incompatible_state");
}

#[test]
fn path_boundaries_and_case_collisions_are_rejected() {
    for path in [
        "../outside",
        "/absolute",
        "C:/outside",
        "foo\\bar",
        "a//b",
        "a/../b",
        "a:stream",
        "NUL.txt",
        "COM1",
        "LPT¹.txt",
        "a.",
        "a ",
        ".",
    ] {
        assert!(validate_relative(path).is_err(), "{path}");
    }
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    assert_eq!(
        prepare_install(
            &root,
            vec![payload(".mpvkit/install.json", "bad")],
            false,
            BTreeMap::new()
        )
        .err()
        .unwrap()
        .code,
        "reserved_path"
    );
    assert_eq!(
        prepare_install(
            &root,
            vec![payload("a.conf", "one"), payload("A.conf", "two")],
            false,
            BTreeMap::new()
        )
        .err()
        .unwrap()
        .code,
        "duplicate_path"
    );
    assert!(!root.exists());
}

#[cfg(target_os = "windows")]
#[test]
fn junction_target_cannot_redirect_deployment() {
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    let outside = sandbox.0.join("outside");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir(&outside).unwrap();
    let link = root.join("portable_config");
    let mut command = platform::command(Path::new("powershell.exe"));
    command.args(["-NoProfile","-NonInteractive","-Command",
        "New-Item -ItemType Junction -Path $env:MPVKIT_TEST_LINK -Target $env:MPVKIT_TEST_OUTSIDE -ErrorAction Stop | Out-Null"])
        .env("MPVKIT_TEST_LINK",&link).env("MPVKIT_TEST_OUTSIDE",&outside);
    crate::mpv::capture_command(command, Duration::from_secs(10)).unwrap();
    assert_eq!(
        prepare_install(
            &root,
            vec![payload(crate::config::relative!("mpv.conf"), "never")],
            false,
            BTreeMap::new()
        )
        .err()
        .unwrap()
        .code,
        "local_path_required"
    );
    assert!(!outside.join("mpv.conf").exists());
    // Remove only the junction itself; never recurse into its target.
    fs::remove_dir(link).unwrap();
}

#[test]
fn configuration_replacement_checks_scripts_repairs_missing_files_and_restores_original() {
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    write_new(
        &root.join(crate::config::relative!("mpv.conf")),
        b"original",
    )
    .unwrap();
    write_new(
        &root.join(crate::config::relative!("notes.txt")),
        b"keep user",
    )
    .unwrap();
    let versions = BTreeMap::from([("uosc".into(), "test".into())]);
    let files = || {
        vec![
            payload(crate::config::relative!("mpv.conf"), "managed"),
            payload(crate::config::relative!("scripts/mpvkit.lua"), "script"),
        ]
    };
    let blocked = prepare_install(&root, files(), false, versions.clone()).unwrap();
    assert!(
        blocked
            .preview
            .conflicts
            .contains(&crate::config::relative!("mpv.conf").into())
    );
    write_new(
        &root.join(crate::config::relative!("scripts/old-osc.lua")),
        b"keep old",
    )
    .unwrap();
    let blocked = prepare_install(&root, files(), true, versions.clone()).unwrap();
    assert!(
        blocked
            .preview
            .conflicts
            .iter()
            .any(|s| s.contains("old-osc"))
    );
    fs::remove_file(root.join(crate::config::relative!("scripts/old-osc.lua"))).unwrap();
    let stale = prepare_install(&root, files(), true, versions.clone()).unwrap();
    write_new(
        &root.join(crate::config::relative!("scripts/late.lua")),
        b"late",
    )
    .unwrap();
    assert_eq!(execute(stale).unwrap_err().code, "script_conflict");
    fs::remove_file(root.join(crate::config::relative!("scripts/late.lua"))).unwrap();
    execute(prepare_install(&root, files(), true, versions.clone()).unwrap()).unwrap();
    assert_eq!(
        status(&root).unwrap().manifest.unwrap().config_directory,
        default_config_directory()
    );
    fs::remove_file(root.join(crate::config::relative!("scripts/mpvkit.lua"))).unwrap();
    execute(prepare_install(&root, files(), false, versions).unwrap()).unwrap();
    assert_eq!(
        read(&root, crate::config::relative!("scripts/mpvkit.lua")),
        "script"
    );
    execute(prepare_uninstall(&root).unwrap()).unwrap();
    assert_eq!(
        read(&root, crate::config::relative!("mpv.conf")),
        "original"
    );
    assert_eq!(
        read(&root, crate::config::relative!("notes.txt")),
        "keep user"
    );
}

// Thread-local and test-only: write a real prefix, then emulate the OS write error.
// Ordinary tests keep their normal filesystem; no volume is filled or ACL changed.
struct WriteFault {
    directory: PathBuf,
    filename: Option<String>,
    skip: usize,
    remaining: usize,
    os_error: i32,
}
thread_local! {
    static WRITE_FAULT: std::cell::RefCell<Option<WriteFault>> = const { std::cell::RefCell::new(None) };
}
struct WriteFaultGuard;
impl WriteFaultGuard {
    fn new(
        directory: PathBuf,
        filename: Option<&str>,
        skip: usize,
        remaining: usize,
        os_error: i32,
    ) -> Self {
        WRITE_FAULT.with(|slot| {
            assert!(slot.borrow().is_none());
            *slot.borrow_mut() = Some(WriteFault {
                directory,
                filename: filename.map(str::to_owned),
                skip,
                remaining,
                os_error,
            });
        });
        Self
    }
    fn assert_consumed(&self) {
        WRITE_FAULT.with(|slot| assert_eq!(slot.borrow().as_ref().unwrap().remaining, 0));
    }
}
impl Drop for WriteFaultGuard {
    fn drop(&mut self) {
        WRITE_FAULT.with(|slot| {
            slot.borrow_mut().take();
        });
    }
}
pub(super) fn inject_write_failure(
    path: &Path,
    file: &mut File,
    bytes: &[u8],
) -> std::io::Result<()> {
    let os_error = WRITE_FAULT.with(|slot| {
        let mut slot = slot.borrow_mut();
        let fault = slot.as_mut()?;
        if fault.remaining == 0
            || path.parent() != Some(fault.directory.as_path())
            || fault
                .filename
                .as_ref()
                .is_some_and(|name| path.file_name() != Some(std::ffi::OsStr::new(name)))
        {
            return None;
        }
        if fault.skip > 0 {
            fault.skip -= 1;
            return None;
        }
        fault.remaining -= 1;
        Some(fault.os_error)
    });
    if let Some(code) = os_error {
        file.write_all(&bytes[..bytes.len().min(3)])?;
        return Err(std::io::Error::from_raw_os_error(code));
    }
    Ok(())
}

#[test]
fn partial_writes_remove_only_created_files_and_preserve_existing_targets() {
    for os_error in [112, 5] {
        // ERROR_DISK_FULL / ERROR_ACCESS_DENIED
        let sandbox = Sandbox::new();
        let root = sandbox.root();
        fs::create_dir(&root).unwrap();
        let existing = root.join("existing");
        write_new(&existing, b"keep existing").unwrap();
        assert_eq!(
            write_new(&existing, b"new").unwrap_err().code,
            "write_failed"
        );
        assert_eq!(fs::read(&existing).unwrap(), b"keep existing");
        let failure = WriteFaultGuard::new(root.clone(), None, 0, 1, os_error);
        assert_eq!(
            write_new(&root.join("created"), b"unfinished")
                .unwrap_err()
                .code,
            "write_failed"
        );
        failure.assert_consumed();
        assert!(!root.join("created").exists());
        drop(failure);
        let failure = WriteFaultGuard::new(root.clone(), None, 0, 1, os_error);
        assert_eq!(
            atomic_write(&existing, b"replacement").unwrap_err().code,
            "write_failed"
        );
        failure.assert_consumed();
        assert_eq!(fs::read(&existing).unwrap(), b"keep existing");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    }
}

#[test]
fn partial_staging_backup_and_commit_writes_preserve_success_and_allow_recovery() {
    for os_error in [112, 5] {
        for phase in ["staging", "backup", "commit-and-rollback"] {
            let sandbox = Sandbox::new();
            let root = sandbox.root();
            install(&root, "first");
            install(&root, "last success");
            let old_manifest = read(&root, ".mpvkit/install.json");
            let old_previous = read(&root, ".mpvkit/previous.json");
            let update = plan(&root, "new content");
            let id = &update.journal.id;
            let (directory, filename, skip, failures) = match phase {
                "staging" => (
                    control(&root, &format!("staging/{id}")).unwrap(),
                    Some("0"),
                    0,
                    1,
                ),
                "backup" => (
                    control(&root, &format!("backups/{id}")).unwrap(),
                    Some("0"),
                    0,
                    1,
                ),
                // pending.json succeeds, install.json and rollback's manifest write fail.
                _ => (root.join(CONTROL), None, 1, 2),
            };
            let failure = WriteFaultGuard::new(directory, filename, skip, failures, os_error);
            let error = execute(update).unwrap_err();
            failure.assert_consumed();
            drop(failure);
            assert_eq!(
                read(&root, crate::config::relative!("mpv.conf")),
                "last success",
                "{phase}"
            );
            assert_eq!(read(&root, ".mpvkit/install.json"), old_manifest, "{phase}");
            assert_eq!(
                read(&root, ".mpvkit/previous.json"),
                old_previous,
                "{phase}"
            );
            if phase == "commit-and-rollback" {
                assert_eq!(error.code, "recovery_required");
                assert!(status(&root).unwrap().recovery_required);
                recover(prepare_recovery(&root).unwrap()).unwrap();
            } else {
                assert_eq!(error.code, "write_failed");
            }
            assert!(!status(&root).unwrap().recovery_required);
            execute(prepare_restore(&root).unwrap()).unwrap();
            assert_eq!(read(&root, crate::config::relative!("mpv.conf")), "first");
        }
    }
}

#[test]
fn partial_owner_record_write_can_retry_without_taking_over_unknown_directories() {
    for os_error in [112, 5] {
        let sandbox = Sandbox::new();
        let root = sandbox.root();
        let failure = WriteFaultGuard::new(root.join(CONTROL), Some("owner.json"), 0, 1, os_error);
        assert_eq!(acquire(&root).unwrap_err().code, "write_failed");
        failure.assert_consumed();
        drop(failure);
        assert!(!root.join(CONTROL).exists());
        let ownership = acquire(&root).unwrap();
        assert!(status(&root).unwrap().manifest.is_none());
        drop(ownership);

        let unknown = sandbox.0.join("unknown");
        fs::create_dir_all(unknown.join(CONTROL)).unwrap();
        let marker = unknown.join(CONTROL).join("user.txt");
        fs::write(&marker, b"keep").unwrap();
        assert_eq!(acquire(&unknown).unwrap_err().code, "ownership_unknown");
        assert_eq!(fs::read(marker).unwrap(), b"keep");
        assert!(unknown.join(CONTROL).is_dir());
    }
}
