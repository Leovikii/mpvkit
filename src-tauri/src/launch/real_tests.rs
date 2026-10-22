//! Opt-in tests only: no download, registry, user config or global env changes.
use super::tests::prepare;
use super::*;

const TEST_NAME: &str = "launch::real_tests::real_mpv_contract";

fn write_new(path: &Path, bytes: &[u8]) {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
}

// Thirty seconds of mono PCM silence. Long enough for the parent-exit check;
// no private media, codecs, speakers or GPU needed for the bounded probe.
fn silent_wav() -> Vec<u8> {
    let data_length = 8000u32 * 2 * 30;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_length).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&8000u32.to_le_bytes());
    bytes.extend_from_slice(&16000u32.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_length.to_le_bytes());
    bytes.resize(44 + data_length as usize, 0);
    bytes
}

fn probe(
    player: &Path,
    media: &Path,
    config: &Path,
    home: &Path,
    load: bool,
    scripts: bool,
) -> String {
    let mut arguments = mpv::launch_args(config, media);
    let boundary = arguments.iter().position(|arg| arg == "--").unwrap();
    arguments.splice(
        boundary..boundary,
        [
            "--ao=null".into(),
            "--vo=null".into(),
            "--audio-display=no".into(),
            "--length=0.1".into(),
            "--terminal=yes".into(),
            "--term-playing-msg=MpvKitProbe volume=${volume} rate=${audio-params/samplerate}"
                .into(),
        ],
    );
    if load {
        arguments[0] = "--config=yes".into();
    }
    if scripts {
        let boundary = arguments.iter().position(|arg| arg == "--").unwrap();
        arguments.insert(boundary, "--load-scripts=yes".into());
    }
    let mut command = platform::command(player);
    command
        .args(arguments)
        .env("MPV_HOME", home)
        .current_dir(config);
    mpv::capture_command(command, Duration::from_secs(5)).unwrap()
}

#[test]
#[ignore = "显式设置 MPVKIT_REAL_MPV 与 MPVKIT_REAL_TEST_ROOT 后单独运行；会执行可信播放器"]
fn real_mpv_contract() {
    // Re-enter this same test in a short-lived launcher process. This exercises
    // production prepare/execute without adding a test command to the application.
    if let Some(run) = std::env::var_os("MPVKIT_REAL_CHILD_RUN") {
        let run = PathBuf::from(run);
        platform::validate_local_path(&run).unwrap();
        let plan = prepare(
            &run.join("managed"),
            run.join("player/mpv.exe").to_str().unwrap(),
            run.join("中文 空格 & $ 声音.wav").to_str().unwrap(),
        )
        .unwrap();
        let result = execute(plan).unwrap();
        write_new(
            &run.join("launch-result.json"),
            &serde_json::to_vec(&result).unwrap(),
        );
        return;
    }

    let root = PathBuf::from(
        std::env::var_os("MPVKIT_REAL_TEST_ROOT")
            .expect("必须明确指定本地测试目录 MPVKIT_REAL_TEST_ROOT"),
    );
    platform::validate_local_path(&root).unwrap();
    assert!(
        root.is_dir(),
        "测试根目录必须已存在，不自动创建或复用真实配置目录"
    );
    let source = mpv::executable_path(
        &std::env::var("MPVKIT_REAL_MPV").expect("必须指定可信 MPVKIT_REAL_MPV"),
    )
    .unwrap();
    run_contract(&root, &source, true);
}

#[test]
fn verification_harness_exercises_launch_and_cleanup_with_fixture() {
    let sandbox = super::tests::Sandbox::new();
    let source = sandbox.player("contract");
    let root = sandbox.path("runs");
    fs::create_dir(&root).unwrap();
    run_contract(&root, &source, false);
}

fn run_contract(root: &Path, source: &Path, real: bool) {
    platform::validate_local_path(root).unwrap();
    mpv::file_stamp(source).unwrap();
    let run = root.join(format!(
        "m1-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&run).unwrap();
    println!("测试记录保留于 {}", run.display());
    let player_dir = run.join("player");
    fs::create_dir(&player_dir).unwrap();
    let player = player_dir.join("mpv.exe");
    fs::copy(source, &player).unwrap();
    if !real {
        write_new(&player_dir.join("mode"), b"contract");
    }
    // The fixed shinchiro candidate may need this local rendering DLL. No scripts
    // or other binaries are copied or run; other builds can fail with a missing DLL.
    let dll = source.parent().unwrap().join("d3dcompiler_43.dll");
    platform::validate_local_path(&dll).unwrap();
    if dll.exists() {
        mpv::file_stamp(&dll).unwrap();
        fs::copy(dll, player_dir.join("d3dcompiler_43.dll")).unwrap();
    }
    let media = run.join("中文 空格 & $ 声音.wav");
    write_new(&media, &silent_wav());
    let portable = player_dir.join("portable_config");
    let home = run.join("mpv-home");
    let config = run.join("isolated");
    for (directory, volume) in [(&portable, 23), (&home, 37), (&config, 41)] {
        fs::create_dir(directory).unwrap();
        write_new(
            &directory.join("mpv.conf"),
            format!("volume={volume}\n").as_bytes(),
        );
        fs::create_dir(directory.join("scripts")).unwrap();
        write_new(
            &directory.join("scripts/witness.lua"),
            b"mp.msg.warn('MpvKitAutoScriptWitness')\n",
        );
    }
    let info = mpv::inspect(&player).unwrap();
    assert!(
        !real || !info.version.contains("fixture"),
        "此入口不能以模拟程序代替真实 MPV"
    );
    for (option, name) in [
        ("--version", "version.txt"),
        ("--list-options", "options.txt"),
    ] {
        let output = mpv::capture(
            &player,
            &[
                "--no-config".into(),
                "--load-scripts=no".into(),
                option.into(),
            ],
            Duration::from_secs(3),
        )
        .unwrap();
        write_new(&run.join(name), output.as_bytes());
    }

    // Positive controls ensure a silent/ignored property probe cannot pass.
    let loaded = probe(&player, &media, &config, &home, true, false);
    write_new(&run.join("explicit-config.txt"), loaded.as_bytes());
    assert!(
        loaded.contains("MpvKitProbe volume=41 rate=8000"),
        "{loaded}"
    );
    for (name, directory, volume) in [("portable", &portable, 23), ("home", &home, 37)] {
        let output = probe(&player, &media, directory, &home, true, false);
        write_new(&run.join(format!("{name}-control.txt")), output.as_bytes());
        assert!(
            output.contains(&format!("MpvKitProbe volume={volume} rate=8000")),
            "{output}"
        );
    }
    let with_script = probe(&player, &media, &config, &home, true, true);
    write_new(&run.join("script-control.txt"), with_script.as_bytes());
    assert!(
        with_script.contains("MpvKitAutoScriptWitness"),
        "{with_script}"
    );
    let isolated = probe(&player, &media, &config, &home, false, false);
    write_new(&run.join("isolated.txt"), isolated.as_bytes());
    assert!(
        isolated.contains("MpvKitProbe volume=100 rate=8000"),
        "{isolated}"
    );
    assert!(!isolated.contains("MpvKitAutoScriptWitness"), "{isolated}");
    // Probe keeps production load-scripts=no even for config=yes controls.
    assert!(!loaded.contains("MpvKitAutoScriptWitness"), "{loaded}");

    let mut launcher = platform::command(&std::env::current_exe().unwrap());
    launcher
        .args(["--exact", TEST_NAME, "--ignored", "--nocapture"])
        .env("MPVKIT_REAL_CHILD_RUN", &run)
        .env("MPV_HOME", &home);
    let output = mpv::capture_command(launcher, Duration::from_secs(10)).unwrap();
    write_new(&run.join("launcher.txt"), output.as_bytes());
    let result: serde_json::Value =
        serde_json::from_slice(&fs::read(run.join("launch-result.json")).unwrap()).unwrap();
    let process = platform::TestPlayer::open(result["processId"].as_u64().unwrap() as u32, &player);
    assert!(process.running(), "启动者退出后播放器已结束");
    thread::sleep(Duration::from_secs(2));
    assert!(process.running(), "启动者退出两秒后播放器未继续运行");
    process.stop(); // Only the verified handle for this run is terminated.
    write_new(
        &run.join("passed.json"),
        &serde_json::to_vec_pretty(&serde_json::json!({
            "schemaVersion": 1,
            "mpvVersion": info.version,
            "verificationMode": if real { "real-player" } else { "fixture" },
            "configIsolation": real,
            "pcmDecode": real,
            "launcherExitSurvivalSeconds": 2,
            "guiCloseVerified": false,
            "videoHdrPowerVerified": false
        }))
        .unwrap(),
    );
}
