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
fn main() {
    let exe = env::current_exe().unwrap();
    let tools = exe.parent().unwrap();
    let args: Vec<_> = env::args().skip(1).collect();
    writeln!(fs::OpenOptions::new().create(true).append(true).open(tools.join("calls.txt")).unwrap(), "{}", args.join("|")).unwrap();
    let mode = fs::read_to_string(tools.join("mode")).unwrap_or_default();
    if mode == "flood" { print!("{}", "x".repeat(1024 * 1024)); return; }
    if mode == "stall" { thread::sleep(Duration::from_secs(60)); }
    if args[0] == "show" {
        if mode == "source-failed" { process::exit(1); }
        println!("Fixture only: shinchiro.mpv 0.41.0\nSource: winget\nNo system changes");
        return;
    }
    match mode.as_str() {
        "cancel" => process::exit(0x800704C7u32 as i32),
        "failed" => process::exit(1),
        "elevation" => process::exit(0x8A150019u32 as i32),
        "no-update" => process::exit(0x8A15002Bu32 as i32),
        "false-success" => return,
        _ => copy_tree(&tools.join("core-source"), &tools.parent().unwrap().join("system-mpv")),
    }
}
