//! Queue and subtitle checks, called only by the explicitly enabled Wayland test.
use super::*;
use serde_json::json;

pub(super) fn verify(root: &Path, media: &Path) {
    let directory = root.join("媒体 test");
    fs::create_dir(&directory).unwrap();
    let first = directory.join("第1集.mp4");
    let second = directory.join("第2集.mp4");
    let last = directory.join("第10集.wav");
    for path in [&first, &second] {
        fs::copy(media, path).unwrap();
    }
    assert!(
        platform::command(Path::new("/usr/bin/ffmpeg"))
            .args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:sample_rate=8000",
                "-t",
                "2",
                "-c:a",
                "pcm_s16le"
            ])
            .arg(&last)
            .status()
            .unwrap()
            .success()
    );
    fs::create_dir(directory.join("nested")).unwrap();
    fs::copy(media, directory.join("nested/第3集.mp4")).unwrap();
    fs::copy(media, directory.join(".hidden.mp4")).unwrap();
    for name in ["cover.png", "list.m3u", "notes.txt", "archive.zip"] {
        fs::write(directory.join(name), "excluded").unwrap();
    }
    let srt = "1\n00:00:00,000 --> 00:00:15,000\n字幕 Subtitle\n";
    let mut subtitles = Vec::new();
    for base in ["sub", "subs", "subtitle", "subtitles"] {
        let capitalized = format!("{}{}", base[..1].to_uppercase(), &base[1..]);
        for folder in [base.to_string(), capitalized, base.to_uppercase()] {
            fs::create_dir(directory.join(&folder)).unwrap();
            let file = directory.join(&folder).join("第2集.zh-Hans.srt");
            fs::write(&file, srt).unwrap();
            subtitles.push(file);
            fs::write(directory.join(&folder).join("第20集.en.srt"), srt).unwrap();
        }
    }
    for suffix in ["", ".zh-Hant", ".chs", ".en.forced", ".en.sdh"] {
        let file = directory.join(format!("第2集{suffix}.srt"));
        fs::write(&file, srt).unwrap();
        subtitles.push(file);
    }
    let socket = root.join("queue.sock");
    let get = |property: &str| request(&socket, json!(["get_property", property]));
    let play = |files: &[&Path], auto_playlist: bool, keep_open: bool| {
        let _ = fs::remove_file(&socket);
        let child = platform::command(Path::new("/usr/bin/mpv"))
            .env_remove("MPV_HOME")
            .env("HOME", platform::default_root().unwrap().join("home"))
            .env(
                "XDG_CONFIG_HOME",
                platform::default_root().unwrap().join("config"),
            )
            .env(
                "XDG_CACHE_HOME",
                platform::default_root().unwrap().join("cache"),
            )
            .env(
                "XDG_STATE_HOME",
                platform::default_root().unwrap().join("state"),
            )
            .args([
                "--player-operation-mode=pseudo-gui",
                "--ao=null",
                "--pause=yes",
                "--resume-playback=no",
                "--save-position-on-quit=no",
            ])
            .arg(format!(
                "--autocreate-playlist={}",
                if auto_playlist { "filter" } else { "no" }
            ))
            .arg(format!(
                "--keep-open={}",
                if keep_open { "yes" } else { "no" }
            ))
            .arg(format!("--input-ipc-server={}", socket.display()))
            .arg(format!("--log-file={}", root.join("queue.log").display()))
            .arg("--")
            .args(files)
            .spawn()
            .unwrap();
        wait(|| socket.exists());
        wait(|| get("time-pos").is_number());
        child
    };
    let finish_file = || {
        let end = get("duration").as_f64().unwrap() - 0.2;
        request(&socket, json!(["seek", end, "absolute+exact"]));
        request(&socket, json!(["set_property", "pause", false]));
    };
    let _guard = PlaybackGuard(socket.clone());
    let mut child = play(&[&second], true, true);
    wait(|| get("scale").is_string());
    let queue = get("playlist");
    assert_eq!(
        queue
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                Path::new(v["filename"].as_str().unwrap())
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
            })
            .collect::<Vec<_>>(),
        ["第1集.mp4", "第2集.mp4", "第10集.wav"]
    );
    assert_eq!(get("playlist-pos"), 1);
    let tracks = get("track-list");
    let mut actual: Vec<_> = tracks
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["type"] == "sub" && v["external"] == true)
        .map(|v| PathBuf::from(v["external-filename"].as_str().unwrap()))
        .collect();
    actual.sort();
    subtitles.sort();
    assert_eq!(
        actual, subtitles,
        "all twelve case-sensitive folders, without unrelated subtitles"
    );
    assert!(
        tracks
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["forced"] == true)
    );
    assert!(
        tracks
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["hearing-impaired"] == true)
    );
    for property in ["loop-file", "loop-playlist"] {
        request(&socket, json!(["set_property", property, "inf"]));
        assert_eq!(get(property), "inf");
        request(&socket, json!(["set_property", property, "no"]));
        assert_eq!(get(property), false);
    }
    request(&socket, json!(["script-binding", "uosc/shuffle"]));
    request(&socket, json!(["script-binding", "uosc/next"]));
    wait(|| get("playlist-pos") != 1);
    assert_eq!(
        get("playlist")
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["filename"].clone())
            .collect::<Vec<_>>(),
        queue
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["filename"].clone())
            .collect::<Vec<_>>()
    );
    request(&socket, json!(["script-binding", "uosc/shuffle"]));
    request(&socket, json!(["set_property", "playlist-pos", 1]));
    wait(|| get("playlist-pos") == 1 && get("time-pos").is_number());
    request(&socket, json!(["keypress", "`"]));
    wait(|| get("user-data")["mpv"]["console"]["open"] == true);
    let paused = get("pause");
    request(&socket, json!(["keypress", "SPACE"]));
    assert_eq!(get("pause"), paused);
    request(&socket, json!(["keypress", "ESC"]));
    wait(|| get("user-data")["mpv"]["console"]["open"] == false);
    request(&socket, json!(["keypress", "SPACE"]));
    assert_eq!(get("pause"), false);
    request(&socket, json!(["set_property", "pause", true]));
    assert_eq!(get("autofit"), "75%x75%");
    assert_eq!(get("auto-window-resize"), false);
    request(
        &socket,
        json!([
            "screenshot-to-file",
            root.join("playback-evidence.png"),
            "window"
        ]),
    );
    fs::write(
        root.join("queue-and-tracks.json"),
        serde_json::to_vec_pretty(&json!({
            "queue":queue,"tracks":tracks,
            "osdDimensions":get("osd-dimensions"),"modes":true,"console":true
        }))
        .unwrap(),
    )
    .unwrap();
    finish_file();
    wait(|| get("playlist-pos") == 2 && get("time-pos").as_f64().is_some_and(|t| t < 1.0));
    finish_file();
    wait(|| get("pause") == true && child.try_wait().unwrap().is_none());
    request(&socket, json!(["quit"]));
    assert!(child.wait().unwrap().success());
    child = play(&[&second], false, true);
    assert_eq!(get("playlist-count"), 1);
    finish_file();
    wait(|| get("eof-reached") == true && get("pause") == true);
    assert_eq!(get("playlist-count"), 1);
    request(&socket, json!(["quit"]));
    assert!(child.wait().unwrap().success());
    child = play(&[&first, &second], false, true);
    assert_eq!(get("playlist-count"), 2);
    finish_file();
    wait(|| get("playlist-pos") == 1);
    request(&socket, json!(["quit"]));
    assert!(child.wait().unwrap().success());
    child = play(&[&second], true, false);
    finish_file();
    wait(|| get("playlist-pos") == 2 && get("time-pos").as_f64().is_some_and(|t| t < 1.0));
    finish_file();
    wait(|| child.try_wait().unwrap().is_some());
    assert!(child.wait().unwrap().success());
    deploy::execute(config::portable::reset(root).unwrap()).unwrap();
    assert_eq!(config::read(root).unwrap(), config::Settings::default());
    for name in ["staging", "recovery", "pending.json", "backups"] {
        assert!(!root.join(".mpvkit").join(name).exists());
    }
}
