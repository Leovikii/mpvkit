use crate::error::{AppError, Result};
use std::{
    ffi::OsString,
    fs,
    io::{self, Read},
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
        fs::MetadataExt,
        io::AsRawHandle,
        process::CommandExt,
    },
    path::{Component, Path, PathBuf, Prefix},
    process::Command,
};
use windows_sys::Win32::{
    Foundation::{ERROR_BROKEN_PIPE, HANDLE},
    Storage::FileSystem::GetDriveTypeW,
    System::{Com::CoTaskMemFree, Pipes::PeekNamedPipe},
    UI::Shell::{
        FOLDERID_LocalAppData, FOLDERID_Pictures, FOLDERID_Profile, FOLDERID_ProgramFiles,
        KF_FLAG_DONT_VERIFY, SHGetKnownFolderPath,
    },
};

fn known_folder(id: &windows_sys::core::GUID) -> Result<PathBuf> {
    let mut raw = std::ptr::null_mut();
    // SAFETY: Windows allocates a NUL-terminated UTF-16 string on success.
    let result = unsafe {
        SHGetKnownFolderPath(
            id,
            KF_FLAG_DONT_VERIFY as u32,
            std::ptr::null_mut(),
            &mut raw,
        )
    };
    if result < 0 {
        return Err(AppError::new(
            "user_path",
            "无法获取本地应用数据目录。",
            "paths",
        ));
    }
    let path = unsafe {
        let mut length = 0;
        while *raw.add(length) != 0 {
            length += 1;
        }
        let value = OsString::from_wide(std::slice::from_raw_parts(raw, length));
        CoTaskMemFree(raw.cast());
        PathBuf::from(value)
    };
    Ok(path)
}

pub fn default_root() -> Result<PathBuf> {
    #[cfg(debug_assertions)]
    if std::env::var_os("MPVKIT_TEST_MODE").is_some_and(|v| v == "1") {
        let root = std::env::var_os("MPVKIT_TEST_ROOT").ok_or_else(|| {
            AppError::new(
                "test_root_required",
                "测试模式必须显式提供本地隔离目录。",
                "paths",
            )
        })?;
        let root = PathBuf::from(root);
        validate_local_path(&root)?;
        return Ok(root);
    }
    let root = known_folder(&FOLDERID_LocalAppData)?.join("MpvKit");
    validate_local_path(&root)?;
    Ok(root)
}

pub fn pictures_directory() -> Result<PathBuf> {
    #[cfg(debug_assertions)]
    if std::env::var_os("MPVKIT_TEST_MODE").is_some_and(|v| v == "1") {
        return Ok(default_root()?.join("pictures"));
    }
    Ok(known_folder(&FOLDERID_Pictures)?.join("MpvKit"))
}

pub fn app_root() -> Result<PathBuf> {
    Ok(crate::targets::read(&default_root()?)?.root)
}

// Check the drive before touching file metadata. Never enumerate mounted volumes.
pub fn validate_local_path(path: &Path) -> Result<()> {
    let invalid = || {
        AppError::new(
            "local_path_required",
            "请使用本地磁盘上的绝对路径；首个开发版本不访问网络盘或目录联接。",
            "paths",
        )
    };
    let mut components = path.components();
    let drive = match components.next() {
        Some(Component::Prefix(prefix)) => match prefix.kind() {
            Prefix::Disk(drive) => drive,
            _ => return Err(invalid()),
        },
        _ => return Err(invalid()),
    };
    if components.next() != Some(Component::RootDir) {
        return Err(invalid());
    }
    let root = [drive as u16, b':' as u16, b'\\' as u16, 0];
    // SAFETY: root is a NUL-terminated drive root; this does not probe a share.
    if !matches!(unsafe { GetDriveTypeW(root.as_ptr()) }, 2 | 3) {
        return Err(invalid());
    }
    let mut current = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => return Err(invalid()),
            Component::Normal(name) if name.to_string_lossy().contains([':', '\0']) => {
                return Err(invalid());
            }
            _ => {}
        }
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_attributes() & 0x400 != 0 => return Err(invalid()),
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(_) => {
                return Err(AppError::new(
                    "path_unavailable",
                    "本地路径不可访问，请检查路径和权限。",
                    "paths",
                ));
            }
        }
    }
    Ok(())
}

pub fn command(executable: &Path) -> Command {
    let mut command = Command::new(executable);
    // CREATE_NO_WINDOW affects console allocation, not MPV's playback window.
    command.creation_flags(0x0800_0000 | 0x0000_0200);
    command
}

// Read only bytes already buffered in an anonymous pipe. No reader thread can
// remain blocked if an unexpected descendant inherits the pipe's write handle.
pub fn drain_pipe<R: Read + AsRawHandle>(pipe: &mut R, bytes: &mut Vec<u8>) -> io::Result<()> {
    loop {
        let mut available = 0;
        let ok = unsafe {
            PeekNamedPipe(
                pipe.as_raw_handle() as HANDLE,
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                &mut available,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            let error = io::Error::last_os_error();
            return if error.raw_os_error() == Some(ERROR_BROKEN_PIPE as i32) {
                Ok(())
            } else {
                Err(error)
            };
        }
        if available == 0 {
            return Ok(());
        }
        if bytes.len() + available as usize > 512 * 1024 {
            return Err(io::Error::other("probe output limit"));
        }
        let mut buffer = [0; 8192];
        let count = pipe.read(&mut buffer[..(available as usize).min(8192)])?;
        if count == 0 {
            return Ok(());
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
}

pub struct MpvSearchLocations {
    pub locations: Vec<(PathBuf, String)>,
    pub notes: Vec<String>,
    pub path_truncated: bool,
}

fn path_mpv_locations(value: &std::ffi::OsStr) -> (Vec<(PathBuf, String)>, bool) {
    // ponytail: 最多检查 PATH 前 64 项；其它位置由手动路径入口处理。
    let entries: Vec<_> = std::env::split_paths(value).take(65).collect();
    let truncated = entries.len() > 64;
    let locations = entries
        .into_iter()
        .take(64)
        .enumerate()
        // Empty PATH entries must not turn into a search of the working directory.
        .filter(|(_, entry)| !entry.as_os_str().is_empty())
        .map(|(index, directory)| {
            (
                directory.join("mpv.exe"),
                format!("PATH 第 {} 项", index + 1),
            )
        })
        .collect();
    (locations, truncated)
}

pub fn mpv_search_locations() -> MpvSearchLocations {
    let mut search = MpvSearchLocations {
        locations: vec![],
        notes: vec![],
        path_truncated: false,
    };
    for (id, relative, source) in [
        (
            &FOLDERID_LocalAppData,
            "MpvKit/player/mpv.exe",
            "MpvKit 播放器位置",
        ),
        (
            &FOLDERID_LocalAppData,
            "Programs/mpv/mpv.exe",
            "用户 Programs 位置",
        ),
        (&FOLDERID_ProgramFiles, "mpv/mpv.exe", "Program Files 位置"),
        (
            &FOLDERID_Profile,
            "scoop/apps/mpv/current/mpv.exe",
            "用户 Scoop 默认位置",
        ),
    ] {
        match known_folder(id) {
            Ok(root) => search.locations.push((root.join(relative), source.into())),
            Err(_) => search.notes.push(format!("无法获取{source}，已跳过。")),
        }
    }
    match std::env::current_exe() {
        Ok(path) => {
            if let Some(parent) = path.parent() {
                search
                    .locations
                    .push((parent.join("mpv.exe"), "MpvKit 程序同级位置".into()));
            }
        }
        Err(_) => search
            .notes
            .push("无法获取 MpvKit 程序同级位置，已跳过。".into()),
    }
    if let Some(value) = std::env::var_os("PATH") {
        let (locations, truncated) = path_mpv_locations(&value);
        search.locations.extend(locations);
        search.path_truncated = truncated;
    } else {
        search.notes.push("PATH 未设置，已跳过 PATH 查找。".into());
    }
    search
}

pub fn replace_file(source: &Path, destination: &Path) -> Result<()> {
    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };
    validate_local_path(source)?;
    validate_local_path(destination)?;
    let source: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let destination: Vec<u16> = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let ok = unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if ok == 0 {
        return Err(AppError::new(
            "replace_failed",
            "文件替换失败，可能被占用或权限不足。",
            "deploy",
        ));
    }
    Ok(())
}

// Only playback contract tests use these handles. Verify the image before
// accepting ownership, so cleanup cannot terminate an unrelated reused PID.
#[cfg(test)]
pub struct TestPlayer {
    handle: HANDLE,
    verified: bool,
}

#[cfg(test)]
impl TestPlayer {
    pub fn open(process_id: u32, executable: &Path) -> Self {
        use windows_sys::Win32::System::Threading::{
            OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
            QueryFullProcessImageNameW,
        };
        let handle = unsafe {
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_TERMINATE | 0x0010_0000,
                0,
                process_id,
            )
        };
        assert!(!handle.is_null(), "无法打开测试播放器进程");
        let mut process = Self {
            handle,
            verified: false,
        };
        let mut buffer = vec![0u16; 32768];
        let mut length = buffer.len() as u32;
        assert_ne!(
            unsafe { QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut length) },
            0,
            "无法核对测试播放器路径"
        );
        let actual = OsString::from_wide(&buffer[..length as usize]);
        assert!(
            actual
                .to_string_lossy()
                .eq_ignore_ascii_case(&executable.to_string_lossy()),
            "PID 对应的程序不是本次测试播放器"
        );
        process.verified = true;
        process
    }

    pub fn stop(self) {
        use windows_sys::Win32::{
            Foundation::WAIT_OBJECT_0,
            System::Threading::{TerminateProcess, WaitForSingleObject},
        };
        if self.running() {
            assert_ne!(
                unsafe { TerminateProcess(self.handle, 0) },
                0,
                "无法结束测试播放器"
            );
        }
        assert_eq!(
            unsafe { WaitForSingleObject(self.handle, 1000) },
            WAIT_OBJECT_0,
            "测试播放器未在期限内退出"
        );
    }

    pub fn running(&self) -> bool {
        use windows_sys::Win32::{
            Foundation::WAIT_TIMEOUT, System::Threading::WaitForSingleObject,
        };
        unsafe { WaitForSingleObject(self.handle, 0) == WAIT_TIMEOUT }
    }
}

#[cfg(test)]
impl Drop for TestPlayer {
    fn drop(&mut self) {
        use windows_sys::Win32::{
            Foundation::CloseHandle,
            System::Threading::{TerminateProcess, WaitForSingleObject},
        };
        unsafe {
            if self.verified && self.running() {
                TerminateProcess(self.handle, 0);
                WaitForSingleObject(self.handle, 1000);
            }
            CloseHandle(self.handle);
        }
    }
}

pub fn playback_config(hardware_decoding: bool) -> &'static str {
    if hardware_decoding {
        "vo=gpu-next\ngpu-api=d3d11\nhwdec=auto-safe\n"
    } else {
        "vo=gpu-next\ngpu-api=d3d11\nhwdec=no\n"
    }
}

pub fn show_error(message: &str) {
    let message: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
    let title: Vec<u16> = "MpvKit".encode_utf16().chain(Some(0)).collect();
    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW(
            std::ptr::null_mut(),
            message.as_ptr(),
            title.as_ptr(),
            0x10,
        );
    }
}

pub fn open_default_apps() -> Result<()> {
    #[cfg(debug_assertions)]
    if std::env::var_os("MPVKIT_TEST_MODE").is_some_and(|v| v == "1") {
        return Ok(());
    }
    let verb: Vec<u16> = "open".encode_utf16().chain(Some(0)).collect();
    let target: Vec<u16> = "ms-settings:defaultapps"
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let result = unsafe {
        windows_sys::Win32::UI::Shell::ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            target.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1,
        )
    };
    if result as isize <= 32 {
        return Err(AppError::new(
            "settings_open",
            "无法打开系统设置，请手动进入设置 → 应用 → 默认应用。",
            "integration",
        ));
    }
    Ok(())
}

pub fn process_active(pid: u32) -> bool {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, STILL_ACTIVE},
        System::Threading::{GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
    };
    // A denied query is treated as active; cleanup must not guess that a process ended.
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return windows_sys::Win32::Foundation::GetLastError() != 87;
        }
        let mut code = 0;
        let ok = GetExitCodeProcess(handle, &mut code);
        CloseHandle(handle);
        ok == 0 || code == STILL_ACTIVE as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_search_is_bounded_and_preserves_order() {
        let paths: Vec<_> = (0..70)
            .map(|index| PathBuf::from(format!(r"C:\test\{index}")))
            .collect();
        let value = std::env::join_paths(&paths).unwrap();
        let (locations, truncated) = path_mpv_locations(&value);
        assert!(truncated);
        assert_eq!(locations.len(), 64);
        assert_eq!(locations[0].0, paths[0].join("mpv.exe"));
        assert_eq!(locations[63].0, paths[63].join("mpv.exe"));
    }

    #[test]
    fn quoted_unicode_path_is_preserved_and_empty_entries_are_not_cwd() {
        let (locations, truncated) =
            path_mpv_locations(std::ffi::OsStr::new(";\"C:\\播放器 空格\";;C:\\tools;"));
        assert!(!truncated);
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].0, PathBuf::from(r"C:\播放器 空格\mpv.exe"));
        assert_eq!(locations[0].1, "PATH 第 2 项");
    }
}
