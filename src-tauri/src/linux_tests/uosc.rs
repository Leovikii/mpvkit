//! Real native layout, clicks and startup sizing; all writes stay in a private copy.
use super::*;
use serde_json::json;

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &to.join(entry.file_name()));
        } else {
            fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
        }
    }
}

pub(super) fn verify(root: &Path, media: &Path) {
    let output = root.join("uosc-native");
    fs::create_dir(&output).unwrap();
    let config = output.join("config");
    fs::create_dir(&config).unwrap();
    for name in ["mpv.conf", "input.conf"] {
        fs::copy(root.join(name), config.join(name)).unwrap();
    }
    for name in ["scripts", "script-opts", "fonts", "shaders"] {
        if root.join(name).is_dir() {
            copy_tree(&root.join(name), &config.join(name));
        }
    }
    let main = config.join("scripts/uosc/main.lua");
    let mut script = fs::read(&main).unwrap();
    script.extend_from_slice(include_bytes!("uosc-inspect.lua"));
    fs::write(main, script).unwrap();
    let socket = output.join("ipc.sock");
    let get = |key: &str| request(&socket, json!(["get_property", key]));
    let inspect = || {
        request(
            &socket,
            json!(["script-message-to", "uosc", "mpvkit-test-inspect"]),
        );
        std::thread::sleep(Duration::from_millis(60));
        get("user-data/mpvkit-test")
    };
    let launch = |name: &str, extra: &[&str], source: &Path| {
        let _ = fs::remove_file(&socket);
        let child = platform::command(Path::new("/usr/bin/mpv"))
            .env("MPV_HOME", &config)
            .args([
                "--ao=null",
                "--pause=yes",
                "--keep-open=yes",
                "--resume-playback=no",
                "--save-position-on-quit=no",
            ])
            .arg(format!("--config-dir={}", config.display()))
            .arg(format!("--input-ipc-server={}", socket.display()))
            .arg(format!(
                "--log-file={}",
                output.join(format!("{name}.log")).display()
            ))
            .args(extra)
            .arg("--")
            .arg(source)
            .spawn()
            .unwrap();
        wait(|| socket.exists());
        wait(|| {
            get("user-data")["mpvkit-test-loaded"]
                .as_u64()
                .is_some_and(|n| n > 0)
        });
        wait(|| get("user-data/mpvkit-playback-mode") == "sequential");
        std::thread::sleep(Duration::from_millis(400));
        child
    };
    let click = |e: &serde_json::Value| {
        wait(|| inspect()["curtain"] == 0);
        let x = (e["ax"].as_f64().unwrap() + e["bx"].as_f64().unwrap()) / 2.0;
        let y = (e["ay"].as_f64().unwrap() + e["by"].as_f64().unwrap()) / 2.0;
        request(
            &socket,
            json!(["script-message-to", "uosc", "mpvkit-test-pointer", "1", "1"]),
        );
        std::thread::sleep(Duration::from_millis(100));
        request(
            &socket,
            json!(["mouse", x.round() as i64, y.round() as i64]),
        );
        request(
            &socket,
            json!([
                "script-message-to",
                "uosc",
                "mpvkit-test-pointer",
                (x.round() as i64).to_string(),
                (y.round() as i64).to_string()
            ]),
        );
        std::thread::sleep(Duration::from_millis(350));
        request(&socket, json!(["keydown", "MBTN_LEFT"]));
        request(&socket, json!(["keyup", "MBTN_LEFT"]));
        std::thread::sleep(Duration::from_millis(200));
    };
    let _guard = PlaybackGuard(socket.clone());
    let mut child = launch("startup", &[], media);
    let startup_size = get("osd-dimensions");
    let startup_outputs = get("display-names");
    let actual_output = startup_outputs[0].as_str().unwrap().to_owned();
    let startup_log = fs::read_to_string(output.join("startup.log")).unwrap();
    assert!(
        startup_log.contains("Reapplied startup autofit")
            || startup_log.contains("window spans multiple")
    );
    let mut layouts = Vec::new();
    // Widths are in uosc logical pixels, including a high-DPI narrow window.
    for width in [1200, 900, 640, 480, 320] {
        let scale = inspect()["scale"].as_f64().unwrap();
        let physical = (f64::from(width) * scale).round() as u32;
        request(
            &socket,
            json!([
                "set_property",
                "geometry",
                format!("{physical}x{}", physical * 9 / 16)
            ]),
        );
        std::thread::sleep(Duration::from_millis(250));
        let data = inspect();
        let controls = &data["controls"];
        for id in ["1", "2", "3", "13", "14"] {
            assert_eq!(
                controls[id]["enabled"], true,
                "essential {id} at {width}: {data}"
            );
        }
        assert!(controls["2"]["bx"].as_f64().unwrap() < data["width"].as_f64().unwrap() / 2.0);
        assert_eq!(controls["1"]["icon"], "skip_previous");
        assert_eq!(controls["3"]["icon"], "skip_next");
        assert_eq!(controls["8"]["kind"], "button");
        let mut visible: Vec<_> = controls
            .as_object()
            .unwrap()
            .values()
            .filter(|e| e["enabled"] == true)
            .collect();
        visible.sort_by(|a, b| {
            a["ax"]
                .as_f64()
                .unwrap()
                .total_cmp(&b["ax"].as_f64().unwrap())
        });
        for pair in visible.windows(2) {
            assert!(
                pair[0]["bx"].as_f64().unwrap() <= pair[1]["ax"].as_f64().unwrap() + 1.0,
                "overlap: {data}"
            );
        }
        assert!(visible.last().unwrap()["bx"].as_f64().unwrap() <= data["width"].as_f64().unwrap());
        layouts.push(data);
    }
    request(&socket, json!(["script-binding", "uosc/menu"]));
    let menu = inspect();
    for label in [
        "Playback speed",
        "Playback mode",
        "Playback info",
        "Audio tracks",
        "Subtitles",
    ] {
        assert!(
            menu["menu"]["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v["title"].as_str().unwrap().contains(label))
        );
    }
    request(&socket, json!(["keypress", "ESC"]));
    request(&socket, json!(["set_property", "geometry", "1800x1012"]));
    std::thread::sleep(Duration::from_millis(250));
    let data = inspect();
    click(&data["controls"]["2"]);
    assert_eq!(get("pause"), false);
    request(&socket, json!(["keypress", "SPACE"]));
    assert_eq!(get("pause"), true);
    click(&inspect()["controls"]["8"]);
    let data = inspect();
    assert_eq!(data["menu"]["type"], "mpvkit-speed");
    assert_eq!(data["menu"]["items"].as_array().unwrap().len(), 6);
    request(&socket, json!(["keypress", "HOME"]));
    request(&socket, json!(["keypress", "DOWN"]));
    request(&socket, json!(["keypress", "ENTER"]));
    wait(|| get("speed") == 0.75);
    request(&socket, json!(["set_property", "speed", 1.1]));
    wait(|| inspect()["controls"]["8"]["text"] == "1.1×");
    click(&inspect()["controls"]["8"]);
    assert_eq!(
        inspect()["menu"]["type"],
        "mpvkit-speed",
        "reopened by native click"
    );
    request(&socket, json!(["set_property", "speed", 1.5]));
    wait(|| {
        inspect()["menu"]["items"][4]["hint"]
            .as_str()
            .is_some_and(|s| s.contains("Current"))
    });
    request(&socket, json!(["keypress", "ESC"]));
    assert_eq!(get("pause"), true);
    click(&inspect()["controls"]["9"]);
    assert_eq!(inspect()["menu"]["type"], "mpvkit-playback");
    request(&socket, json!(["keypress", "ESC"]));
    // Fullscreen round trips and file changes must retain the chosen window size.
    let before = get("osd-dimensions");
    request(&socket, json!(["set_property", "fullscreen", true]));
    std::thread::sleep(Duration::from_millis(250));
    request(&socket, json!(["set_property", "fullscreen", false]));
    std::thread::sleep(Duration::from_millis(250));
    assert_eq!(get("osd-dimensions")["w"], before["w"]);
    let loaded = get("user-data")["mpvkit-test-loaded"].as_u64().unwrap();
    request(&socket, json!(["loadfile", media, "replace"]));
    wait(|| {
        get("user-data")["mpvkit-test-loaded"]
            .as_u64()
            .is_some_and(|n| n > loaded)
    });
    assert_eq!(get("osd-dimensions")["w"], before["w"]);
    request(&socket, json!(["quit"]));
    assert!(child.wait().unwrap().success());
    // Reference target comes from MPV itself with the actual output hint, never a product screen constant.
    child = launch(
        "output-reference",
        &[&format!("--screen-name={actual_output}")],
        media,
    );
    let fit_bounds = get("osd-dimensions");
    let reference_outputs = get("display-names");
    if startup_outputs.as_array().unwrap().len() == 1 && reference_outputs == startup_outputs {
        assert_eq!(fit_bounds["w"], startup_size["w"]);
        assert_eq!(fit_bounds["h"], startup_size["h"]);
    }
    request(&socket, json!(["quit"]));
    assert!(child.wait().unwrap().success());
    let mut startup_cases = Vec::new();
    for (name, size) in [
        ("1080p", "1920x1080"),
        ("4k", "3840x2160"),
        ("portrait", "720x1280"),
        ("ultrawide", "1280x540"),
    ] {
        let sample = output.join(format!("{name}.mp4"));
        assert!(
            platform::command(Path::new("/usr/bin/ffmpeg"))
                .args([
                    "-v",
                    "error",
                    "-f",
                    "lavfi",
                    "-i",
                    &format!("color=white:size={size}:rate=1"),
                    "-frames:v",
                    "1",
                    "-c:v",
                    "libx264",
                    "-preset",
                    "ultrafast"
                ])
                .arg(&sample)
                .status()
                .unwrap()
                .success()
        );
        child = launch(name, &[], &sample);
        let dimensions = get("osd-dimensions");
        let w = dimensions["w"].as_f64().unwrap();
        let h = dimensions["h"].as_f64().unwrap();
        let outputs = get("display-names");
        let checked = outputs.as_array().unwrap().len() == 1 && outputs == reference_outputs;
        if checked {
            let max_w = fit_bounds["w"].as_f64().unwrap();
            let max_h = fit_bounds["h"].as_f64().unwrap();
            assert!(w <= max_w + 2.0 && h <= max_h + 2.0);
            assert!((w - max_w).abs() <= 2.0 || (h - max_h).abs() <= 2.0);
        } else {
            assert!(
                fs::read_to_string(output.join(format!("{name}.log")))
                    .unwrap()
                    .contains("window spans multiple")
            );
        }
        startup_cases
            .push(json!({"case":name,"dimensions":dimensions,"outputs":outputs,"checked":checked}));
        request(&socket, json!(["quit"]));
        assert!(child.wait().unwrap().success());
    }
    child = launch("custom-fit", &["--autofit=60%x60%"], media);
    let custom = get("osd-dimensions");
    let custom_outputs = get("display-names");
    if custom_outputs.as_array().unwrap().len() == 1 && custom_outputs == reference_outputs {
        assert!(
            (custom["w"].as_f64().unwrap() - fit_bounds["w"].as_f64().unwrap() * 0.8).abs() <= 2.0
        );
    } else {
        assert!(
            fs::read_to_string(output.join("custom-fit.log"))
                .unwrap()
                .contains("window spans multiple")
        );
    }
    startup_cases.push(json!({"case":"custom-fit","dimensions":custom}));
    request(&socket, json!(["quit"]));
    assert!(child.wait().unwrap().success());
    child = launch("audio-first", &[], &root.join("媒体 test/第10集.wav"));
    assert!(
        !fs::read_to_string(output.join("audio-first.log"))
            .unwrap()
            .contains("Reapplied startup autofit")
    );
    request(&socket, json!(["loadfile", media, "replace"]));
    wait(|| get("user-data")["mpvkit-test-loaded"] == 2);
    wait(|| {
        let log = fs::read_to_string(output.join("audio-first.log")).unwrap();
        log.contains("Reapplied startup autofit") || log.contains("window spans multiple")
    });
    if get("display-names") == reference_outputs {
        assert_eq!(get("osd-dimensions")["w"], fit_bounds["w"]);
    }
    request(&socket, json!(["quit"]));
    assert!(child.wait().unwrap().success());
    for (name, extra) in [
        ("geometry", vec!["--geometry=900x506"]),
        ("scale", vec!["--window-scale=0.5"]),
        ("maximized", vec!["--window-maximized=yes"]),
        ("fullscreen", vec!["--fullscreen=yes"]),
    ] {
        child = launch(name, &extra, media);
        assert!(
            !fs::read_to_string(output.join(format!("{name}.log")))
                .unwrap()
                .contains("Reapplied startup autofit")
        );
        request(&socket, json!(["quit"]));
        assert!(child.wait().unwrap().success());
    }
    fs::write(
        output.join("layouts.json"),
        serde_json::to_vec_pretty(
            &json!({"startup":startup_size,"outputs":startup_outputs,"reference":fit_bounds,"startupCases":startup_cases,"layouts":layouts}),
        )
        .unwrap(),
    )
    .unwrap();
    for entry in fs::read_dir(&output).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "log") {
            let log = fs::read_to_string(path).unwrap();
            assert!(
                !log.contains("Lua error") && !log.contains("stack traceback"),
                "{log}"
            );
        }
    }
}
