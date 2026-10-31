// Isolated native process fixture: never calls a real package manager/installer.
use std::{env, fs, io::Write, path::Path, process, thread, time::Duration};
fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() { copy_tree(&entry.path(), &target.join(entry.file_name())); }
        else { fs::copy(entry.path(), target.join(entry.file_name())).unwrap(); }
    }
}
fn remove_copies(source: &Path, target: &Path) {
    for entry in fs::read_dir(source).unwrap() {
        let entry=entry.unwrap(); let path=target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() { remove_copies(&entry.path(), &path); let _=fs::remove_dir(&path); }
        else if path.exists() { assert_eq!(fs::read(entry.path()).unwrap(),fs::read(&path).unwrap()); fs::remove_file(path).unwrap(); }
    }
    let _=fs::remove_dir(target);
}
fn main() {
    let exe = env::current_exe().unwrap();
    let tools = exe.parent().unwrap();
    let args: Vec<_> = env::args().skip(1).collect();
    writeln!(fs::OpenOptions::new().create(true).append(true).open(tools.join("calls.txt")).unwrap(), "{}", args.join("|")).unwrap();
    let mode = fs::read_to_string(tools.join("mode")).unwrap_or_default();
    if mode == "flood" { print!("{}", "x".repeat(1024 * 1024)); return; }
    if mode == "stall" { thread::sleep(Duration::from_secs(60)); }
    if args[0] == "show" || args[0] == "list" {
        if mode == "source-failed" { process::exit(1); }
        if args.iter().any(|arg| arg == "--upgrade-available") {
            if mode == "no-update" { println!("No installed package found."); }
            else { println!("mpv shinchiro.mpv 0.40.0 0.41.0 winget"); }
            return;
        }
        println!("Fixture only: shinchiro.mpv 0.41.0\nSource: winget\nNo system changes");
        return;
    }
    #[cfg(windows)]
    if mode == "console" {
        #[link(name="kernel32")]
        unsafe extern "system" { fn GetStdHandle(kind: u32) -> *mut std::ffi::c_void; fn GetConsoleMode(handle: *mut std::ffi::c_void, mode: *mut u32) -> i32; }
        let mut console_mode=0;
        let interactive=unsafe { GetConsoleMode(GetStdHandle(-10i32 as u32), &mut console_mode) != 0 };
        fs::write(tools.join("console-input.txt"),interactive.to_string()).unwrap();
        return;
    }
    match mode.as_str() {
        "cancel" => process::exit(0x800704C7u32 as i32),
        "failed" => process::exit(1),
        "elevation" => process::exit(0x8A150019u32 as i32),
        "no-update" => process::exit(0x8A15002Bu32 as i32),
        "false-success" => return,
        _ if args[0] == "uninstall" => {
            // Delete only matching fixture copies, never an arbitrary tree.
            let source = tools.join("core-source");
            let target = tools.parent().unwrap().join("system-mpv");
            remove_copies(&source, &target);
        },
        _ => copy_tree(&tools.join("core-source"), &tools.parent().unwrap().join("system-mpv")),
    }
}
