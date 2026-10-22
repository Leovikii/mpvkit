use super::*;
use std::{fs, io::Write};

fn archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, content) in entries {
        writer
            .start_file(*name, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(content).unwrap();
    }
    writer.finish().unwrap().into_inner()
}
fn zip_asset(bytes: &[u8]) -> Asset {
    Asset {
        size: bytes.len() as u64,
        sha256: Box::leak(deploy::hash_bytes(bytes).into_boxed_str()),
        ..ASSETS[1].clone()
    }
}
fn code<T>(result: Result<T>) -> &'static str {
    result.err().expect("must reject").code
}

#[test]
fn downloads_are_bounded_verified_and_cancelled() {
    let cancel = AtomicBool::new(false);
    let mut count = 0;
    assert_eq!(
        receive(&mut Cursor::new(b"abc"), 3, &cancel, |n| count = n).unwrap(),
        b"abc"
    );
    assert_eq!(count, 3);
    assert_eq!(
        code(receive(&mut Cursor::new(b"abcd"), 3, &cancel, |_| {})),
        "asset_size"
    );
    assert_eq!(
        code(receive(&mut Cursor::new(b"ab"), 3, &cancel, |_| {})),
        "asset_size"
    );
    assert_eq!(
        code(receive(&mut Cursor::new(b"abc"), 3, &cancel, |_| cancel
            .store(true, Ordering::Relaxed))),
        "cancelled"
    );
    assert_eq!(
        code(download(&ASSETS[0], &cancel, &mut |_| panic!(
            "must not download"
        ))),
        "cancelled"
    );
    let bytes = archive(&[("a", b"data")]);
    let asset = zip_asset(&bytes);
    verify(&asset, &bytes).unwrap();
    let mut corrupted = bytes.clone();
    corrupted[10] ^= 1;
    assert_eq!(
        code(unpack(&asset, &corrupted, &AtomicBool::new(false))),
        "asset_hash"
    );
    for url in [
        "http://github.com/file",
        "https://github.com.evil.test/file",
        "https://user:secret@github.com/file",
        "https://github.com:444/file",
        "https://127.0.0.1/file",
    ] {
        assert!(!allowed_url(&reqwest::Url::parse(url).unwrap()));
    }
    for url in [
        ASSETS[0].url,
        ASSETS[1].url,
        ASSETS[2].url,
        "https://release-assets.githubusercontent.com/file",
    ] {
        assert!(allowed_url(&reqwest::Url::parse(url).unwrap()));
    }
}

#[test]
fn every_entry_is_checked_even_when_not_selected() {
    let cancel = AtomicBool::new(false);
    for name in [
        "../escape",
        "C:/escape",
        "a\\escape",
        "a:stream",
        "/escape",
        "a/CON.txt",
        "a/../escape",
        ".mpvkit/owner.json",
    ] {
        let bytes = archive(&[(name, b"unsafe")]);
        assert!(
            unpack(&zip_asset(&bytes), &bytes, &cancel).is_err(),
            "{name}"
        );
    }
    let bytes = archive(&[("a", b"1"), ("A", b"2")]);
    assert_eq!(
        code(unpack(&zip_asset(&bytes), &bytes, &cancel)),
        "archive_duplicate"
    );
    for names in [["a", "a/b"], ["a/b", "a"]] {
        let bytes = archive(&[(names[0], b"1"), (names[1], b"2")]);
        assert_eq!(
            code(unpack(&zip_asset(&bytes), &bytes, &cancel)),
            "archive_collision"
        );
    }
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .add_symlink(
            "link",
            "../outside",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
    let bytes = writer.finish().unwrap().into_inner();
    assert_eq!(
        code(unpack(&zip_asset(&bytes), &bytes, &cancel)),
        "archive_link"
    );
    let mut boundary = Boundary::new();
    assert_eq!(
        code(boundary.entry("huge", false, MAX_FILE + 1, false)),
        "archive_limit"
    );
    assert_eq!(
        code(Boundary::new().entry("junction", true, 0, true)),
        "archive_link"
    );
    assert_eq!(
        code(Boundary::new().entry("folder/", true, 1, false)),
        "archive_limit"
    );
    let mut boundary = Boundary::new();
    boundary.entry("a", false, MAX_FILE, false).unwrap();
    boundary.entry("b", false, MAX_FILE, false).unwrap();
    assert_eq!(code(boundary.entry("c", false, 1, false)), "archive_limit");
    let mut boundary = Boundary::new();
    for n in 0..MAX_ENTRIES {
        boundary.entry(&format!("{n}"), false, 0, false).unwrap();
    }
    assert_eq!(
        code(boundary.entry("overflow", false, 0, false)),
        "archive_limit"
    );
}

#[test]
fn complete_windows_components_are_selected_without_installers() {
    let entries: &[(&str, &[u8])] = &[
        ("scripts/uosc/main.lua", b"lua"),
        ("scripts/uosc/bin/ziggy-windows.exe", b"exe"),
        ("scripts/uosc/lib/std.lua", b"lib"),
        ("scripts/uosc/intl/zh-Hans.json", b"{}"),
        ("scripts/uosc/bin/ziggy-linux", b"omit"),
        ("fonts/uosc_icons.otf", b"font"),
        ("fonts/uosc_textures.ttf", b"font"),
        ("installer.ps1", b"omit"),
    ];
    let bytes = archive(entries);
    let files = unpack(&zip_asset(&bytes), &bytes, &AtomicBool::new(false)).unwrap();
    assert_eq!(files.len(), 6);
    assert!(files.iter().all(|p| p.path.starts_with("portable_config/")));
    assert!(
        !files
            .iter()
            .any(|p| p.path.contains("linux") || p.path.contains("installer"))
    );
    let bytes = archive(&entries[..2]);
    assert_eq!(
        code(unpack(&zip_asset(&bytes), &bytes, &AtomicBool::new(false))),
        "component_incomplete"
    );
    assert_eq!(destination("mpv", "installer/mpv-install.bat"), None);
}

#[test]
#[ignore = "requires explicitly selected local fixed research assets"]
fn fixed_archives_extract_and_deploy() {
    let directory =
        std::env::var_os("MPVKIT_COMPONENT_ASSETS").expect("set local assets directory");
    let directory = std::path::PathBuf::from(directory);
    crate::platform::validate_local_path(&directory).unwrap();
    let names = [
        "mpv-x86_64-20261020-git-6a262d11f6.7z",
        "uosc-5.13.0.zip",
        "thumbfast-thumbfast.lua",
        "LXGWWenKaiScreen.ttf",
    ];
    let mut files = Vec::new();
    for (asset, name) in ASSETS.iter().zip(names) {
        let bytes = fs::read(directory.join(name)).unwrap();
        files.extend(unpack(asset, &bytes, &AtomicBool::new(false)).unwrap());
    }
    let source: serde_json::Value =
        serde_json::from_str(include_str!("../../../resources/source-materials.json")).unwrap();
    let helper = files
        .iter()
        .find(|file| file.path == "portable_config/scripts/uosc/bin/ziggy-windows.exe")
        .unwrap();
    assert_eq!(
        deploy::hash_bytes(&helper.bytes),
        source["uoscHelper"]["sha256"].as_str().unwrap()
    );
    for font in source["uoscFonts"].as_array().unwrap() {
        let file = files
            .iter()
            .find(|file| {
                file.path == format!("portable_config/fonts/{}", font["file"].as_str().unwrap())
            })
            .unwrap();
        assert_eq!(
            deploy::hash_bytes(&file.bytes),
            font["sha256"].as_str().unwrap()
        );
    }
    files.extend(license_materials(true));
    let total: usize = files.iter().map(|p| p.bytes.len()).sum();
    let count = files.len();
    let root = std::env::temp_dir().join(format!("mpvkit-components-{}", deploy::operation_id()));
    let versions = ASSETS
        .iter()
        .map(|a| (a.id.into(), a.version.into()))
        .collect();
    let plan = deploy::prepare_install(&root, files, false, None, versions).unwrap();
    assert!(!root.exists());
    let status = deploy::execute(plan).unwrap();
    assert_eq!(status.manifest.unwrap().files.len(), count);
    for material in license_materials(true) {
        assert_eq!(fs::read(root.join(&material.path)).unwrap(), material.bytes);
    }
    deploy::execute(deploy::prepare_uninstall(&root).unwrap()).unwrap();
    for material in license_materials(true) {
        assert!(!root.join(&material.path).exists());
    }
    println!(
        "fixed assets: {count} files, {total} bytes; verified -> parsed -> deployed -> uninstalled; {root:?}"
    );
}

#[test]
#[ignore = "explicit network acceptance for fixed HTTPS assets"]
fn fixed_https_downloads() {
    assert_eq!(std::env::var("MPVKIT_TEST_NETWORK").as_deref(), Ok("1"));
    for asset in ASSETS {
        eprintln!("HTTPS starting: {}", asset.id);
        let started = std::time::Instant::now();
        let mut reported = 0;
        let bytes = download(asset, &AtomicBool::new(false), &mut |p| {
            if p.received / (4 * 1024 * 1024) > reported {
                reported = p.received / (4 * 1024 * 1024);
                eprintln!(
                    "HTTPS {}: {}/{} bytes in {:?}",
                    asset.id,
                    p.received,
                    p.total,
                    started.elapsed()
                );
            }
        })
        .unwrap_or_else(|error| panic!("{} after {:?}: {error:?}", asset.id, started.elapsed()));
        verify(asset, &bytes).unwrap();
        println!(
            "HTTPS verified: {} {} bytes {}",
            asset.id,
            bytes.len(),
            asset.sha256
        );
    }
}

#[test]
#[ignore = "requires local fixed assets and synthetic SDR sample; opens real video output"]
fn real_component_graphics() {
    let media =
        std::path::PathBuf::from(std::env::var_os("MPVKIT_TEST_VIDEO").expect("synthetic video"));
    exercise_graphics(&media, 1280, 720, None);
}

fn exercise_graphics(
    media: &std::path::Path,
    width: u64,
    height: u64,
    metadata: Option<&serde_json::Value>,
) -> std::path::PathBuf {
    let directory = std::path::PathBuf::from(
        std::env::var_os("MPVKIT_COMPONENT_ASSETS").expect("local fixed assets"),
    );
    crate::platform::validate_local_path(&directory).unwrap();
    crate::platform::validate_local_path(media).unwrap();
    let base = std::path::PathBuf::from(
        std::env::var_os("MPVKIT_REAL_TEST_ROOT").expect("isolated test root"),
    );
    crate::platform::validate_local_path(&base).unwrap();
    assert!(base.is_dir());
    let root = base.join(format!("components-video-{}", deploy::operation_id()));
    let mut files = Vec::new();
    for (asset, name) in ASSETS.iter().zip([
        "mpv-x86_64-20261020-git-6a262d11f6.7z",
        "uosc-5.13.0.zip",
        "thumbfast-thumbfast.lua",
        "LXGWWenKaiScreen.ttf",
    ]) {
        files.extend(
            unpack(
                asset,
                &fs::read(directory.join(name)).unwrap(),
                &AtomicBool::new(false),
            )
            .unwrap(),
        );
    }
    let player = root.join("mpv.exe");
    files.extend(
        crate::config::generate(&root, &player, &crate::config::Settings::default()).unwrap(),
    );
    deploy::execute(
        deploy::prepare_install(
            &root,
            files,
            false,
            None,
            ASSETS
                .iter()
                .map(|a| (a.id.into(), a.version.into()))
                .collect(),
        )
        .unwrap(),
    )
    .unwrap();
    let probe = root.join("mpvkit_probe.lua");
    fs::write(&probe, r#"local utils = require('mp.utils')
local result = {thumbnail=false}
local started = mp.get_time()
local reported = false
mp.register_script_message('thumbfast-render', function(json)
    local data = utils.parse_json(json)
    local file = io.open(data.thumbnail .. '.bgra', 'rb')
    if file then
        local size = file:seek('end'); file:close()
        result.thumbnail = size == data.width * data.height * 4
        result.thumbnail_seconds = mp.get_time() - started
        result.thumbnail_bytes = size
        result.thumbnail_width = data.width
        result.thumbnail_height = data.height
    end
end)
mp.add_periodic_timer(0.5, function() if not result.thumbnail then mp.commandv('script-message-to', 'thumbfast', 'thumb', '3', '', '', 'mpvkit_probe') end end)
local function report()
    if reported then return end
    reported = true
    for _, key in ipairs({'current-vo','hwdec-current','osd-dimensions','video-codec','video-params','video-out-params','display-names','display-fps','frame-drop-count','decoder-frame-drop-count','script-list','user-data/osc/margins','scale','dscale'}) do result[key] = mp.get_property_native(key) end
    print('MPVKIT_RESULT=' .. utils.format_json(result))
    mp.commandv('script-message-to', 'thumbfast', 'clear')
    mp.commandv('quit', '0')
end
mp.add_periodic_timer(0.1, function() if mp.get_time() - started >= 4.5 and result.thumbnail then report() end end)
mp.add_timeout(12, report)
"#).unwrap();
    for profile in [
        crate::config::Profile::Balanced,
        crate::config::Profile::PowerSaving,
        crate::config::Profile::HighQuality,
    ] {
        let settings = crate::config::Settings {
            profile,
            ..Default::default()
        };
        deploy::execute(crate::config::prepare(&root, settings).unwrap()).unwrap();
        let session = root.join(format!("probe-{}", profile.id()));
        fs::create_dir(&session).unwrap();
        let log = session.join("player.log");
        let mut command = crate::platform::command(&player);
        command.args([
            "--config=yes".into(),
            format!("--config-dir={}", root.join("config").display()),
            "--load-scripts=yes".into(),
            "--terminal=yes".into(),
            "--ao=null".into(),
            "--keep-open=yes".into(),
            format!("--script={}", probe.display()),
            format!("--log-file={}", log.display()),
            "--msg-level=all=warn,mpvkit_probe=info".into(),
            if width >= 1920 {
                "--geometry=1920x1080"
            } else {
                "--geometry=640x360"
            }
            .into(),
            "--".into(),
            media.to_string_lossy().into_owned(),
        ]);
        command
            .env("MPV_HOME", &session)
            .env("TEMP", &session)
            .env("TMP", &session)
            .current_dir(&session);
        let output =
            crate::mpv::capture_command(command, std::time::Duration::from_secs(15)).unwrap();
        fs::write(session.join("stdout.txt"), &output).unwrap();
        println!("{}: {}", profile.id(), output);
        let json = output
            .lines()
            .find_map(|l| l.split_once("MPVKIT_RESULT=").map(|(_, json)| json))
            .expect("probe callback");
        let result: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(result["current-vo"], "gpu-next");
        assert_eq!(
            result["thumbnail"], true,
            "real thumbnail generation required; see {log:?}"
        );
        assert_eq!(result["video-params"]["w"], width);
        assert_eq!(result["video-params"]["h"], height);
        if let Some(metadata) = metadata {
            for (key, value) in metadata.as_object().unwrap() {
                assert_eq!(&result["video-params"][key], value, "input color metadata");
            }
        }

        assert_eq!(
            result["scale"],
            crate::config::profiles()[profile.id()]["scale"]
        );
        let log = fs::read_to_string(log).unwrap();
        assert!(
            log.contains("args=\"uosc-version\", args=\"5.13.0\""),
            "uosc actual startup message"
        );
        assert!(
            !log.contains("Lua error") && !log.contains("Error parsing option"),
            "unexplained component/config error"
        );
        fs::write(
            session.join("passed.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .unwrap();
    }
    println!("Graphical component evidence: {root:?}");
    root
}

#[test]
fn embedded_provenance_matches_actual_download_contract_and_has_honest_release_gate() {
    let catalog: serde_json::Value =
        serde_json::from_str(include_str!("../../../resources/catalog.json")).unwrap();
    assert_eq!(catalog["schemaVersion"], 1);
    assert_eq!(catalog["releaseReady"], false);
    for asset in ASSETS {
        let entry = catalog["components"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["id"] == asset.id)
            .unwrap();
        assert_eq!(entry["version"], asset.version);
        assert_eq!(entry["url"], asset.url);
        assert_eq!(entry["size"], asset.size);
        assert_eq!(entry["sha256"], asset.sha256);
        assert!(!entry["license"]["materials"].as_array().unwrap().is_empty());
    }
}

#[test]
fn deployed_license_materials_preserve_fixed_bytes_and_external_player_ownership() {
    let index: serde_json::Value =
        serde_json::from_str(include_str!("../../../resources/source-materials.json")).unwrap();
    assert_eq!(index["releaseReady"], false);
    let files = license_materials(true);
    for material in index["materials"].as_array().unwrap() {
        let path = format!("licenses/{}", material["file"].as_str().unwrap());
        let file = files.iter().find(|f| f.path == path).unwrap();
        assert_eq!(file.bytes.len() as u64, material["size"].as_u64().unwrap());
        assert_eq!(
            deploy::hash_bytes(&file.bytes),
            material["sha256"].as_str().unwrap()
        );
    }
    assert!(
        !license_materials(false)
            .iter()
            .any(|f| f.path == "licenses/mpv-Copyright.txt")
    );
}

#[test]
#[ignore = "requires explicit synthetic media matrix, local fixed assets and graphical output"]
fn real_media_matrix() {
    let manifest = std::path::PathBuf::from(
        std::env::var_os("MPVKIT_MEDIA_MATRIX").expect("explicit matrix manifest"),
    );
    let bytes = deploy::read_bytes(&manifest).unwrap();
    assert!(bytes.len() <= 65536);
    let matrix: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(matrix["schemaVersion"], 1);
    let cases = matrix["cases"].as_array().unwrap();
    assert!(!cases.is_empty() && cases.len() <= 8);
    let mut evidence = Vec::new();
    for case in cases {
        let media = std::path::Path::new(case["file"].as_str().unwrap());
        assert_eq!(
            deploy::file_hash(media).unwrap().as_deref(),
            case["sha256"].as_str()
        );
        let width = case["width"].as_u64().unwrap();
        let height = case["height"].as_u64().unwrap();
        assert!(width <= 8192 && height <= 8192);
        let root = exercise_graphics(media, width, height, Some(&case["metadata"]));
        evidence
            .push(serde_json::json!({"id":case["id"],"root":root,"mediaSha256":case["sha256"]}));
    }
    let output = std::path::PathBuf::from(std::env::var_os("MPVKIT_REAL_TEST_ROOT").unwrap())
        .join(format!("media-matrix-{}.json", deploy::operation_id()));
    std::fs::write(&output, serde_json::to_vec_pretty(&evidence).unwrap()).unwrap();
    println!("Matrix evidence: {output:?}");
}

#[test]
fn continuous_and_stalled_responses_have_independent_deadlines() {
    use std::{net::TcpListener, time::Instant};
    for (total, stall, interval) in [
        (
            Duration::from_millis(200),
            Duration::from_secs(1),
            Duration::from_millis(20),
        ),
        (
            Duration::from_secs(2),
            Duration::from_millis(100),
            Duration::from_millis(400),
        ),
    ] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut buffer = [0; 4096];
            assert!(stream.read(&mut buffer).unwrap() > 0);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 30\r\nConnection: close\r\n\r\n")
                .unwrap();
            for _ in 0..30 {
                if stream.write_all(b"x").is_err() {
                    break;
                }
                std::thread::sleep(interval);
            }
        });
        let client = client_builder(total, stall).no_proxy().build().unwrap();
        let started = Instant::now();
        let mut response = client.get(format!("http://{address}/")).send().unwrap();
        let result = receive(&mut response, 30, &AtomicBool::new(false), |_| {});
        assert_eq!(
            code(result),
            "download_failed",
            "total or stall deadline must interrupt response"
        );
        assert!(started.elapsed() < Duration::from_secs(2));
        drop(response);
        server.join().unwrap();
    }
}
