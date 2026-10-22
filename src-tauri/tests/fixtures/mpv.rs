use std::{env, fs, path::Path, thread, time::Duration};

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    let executable = env::current_exe().unwrap();
    let root = executable.parent().unwrap();
    let mode = fs::read_to_string(root.join("mode")).unwrap_or_default();
    if mode == "record-probes" {
        use std::io::Write;
        let mut witness = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(root.join("probes.txt"))
            .unwrap();
        writeln!(witness, "{}", args.join(" ")).unwrap();
    }
    if mode == "timeout" {
        thread::sleep(Duration::from_secs(30));
    }
    if mode == "flood" {
        print!("{}", "x".repeat(2 * 1024 * 1024));
        return;
    }
    if mode == "failure" {
        std::process::exit(7);
    }
    if args.iter().any(|arg| arg == "--version") {
        println!("mpv 0.40.0-fixture\nfixture only, not a real player");
    } else if args.iter().any(|arg| arg == "--list-options") {
        if mode == "missing-capability" {
            println!(" --terminal Flag");
        } else if mode == "missing-managed-capability" {
            println!(" --config-dir String\n --load-scripts Flag\n --terminal Flag");
        } else {
            println!(" --config-dir String\n --load-scripts Flag\n --terminal Flag");
            for key in ["scale","dscale","scale-antiring","correct-downscaling","linear-downscaling","sigmoid-upscaling","hdr-peak-percentile","hdr-contrast-recovery","vo","gpu-api","hwdec","sub-font-size","osd-font-size","volume","osc","input-builtin-bindings","video-sync","interpolation","sub-font","save-position-on-quit","watch-later-options","watch-later-directory","resume-playback","keep-open","screenshot-directory","screenshot-format"] { println!(" --{key} String"); }
        }
    } else if mode == "contract"
        && args
            .iter()
            .any(|arg| arg.starts_with("--term-playing-msg="))
    {
        let config = args
            .iter()
            .find_map(|arg| arg.strip_prefix("--config-dir="))
            .unwrap();
        let volume = if args
            .iter()
            .rev()
            .find(|arg| arg.as_str() == "--no-config" || arg.as_str() == "--config=yes")
            .map(String::as_str)
            == Some("--config=yes")
        {
            fs::read_to_string(Path::new(config).join("mpv.conf"))
                .unwrap()
                .trim()
                .strip_prefix("volume=")
                .unwrap()
                .to_owned()
        } else {
            "100".into()
        };
        println!("MpvKitProbe volume={volume} rate=8000");
        if args.iter().any(|arg| arg == "--load-scripts=yes") {
            println!("MpvKitAutoScriptWitness");
        }
    } else {
        fs::write(root.join("argv.txt"), args.join("\n")).unwrap();
        // Prove GUI-side execute does not wait for playback to finish.
        thread::sleep(if mode == "contract" {
            Duration::from_secs(30)
        } else {
            Duration::from_millis(700)
        });
        let config = args
            .iter()
            .find_map(|arg| arg.strip_prefix("--config-dir="))
            .unwrap();
        assert!(Path::new(config).join("session.json").is_file());
    }
}
