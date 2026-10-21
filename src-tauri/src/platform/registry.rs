use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use windows_sys::Win32::System::Registry::*;
#[cfg(test)]
mod offline;
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegistryData {
    pub kind: u32,
    pub bytes: Vec<u8>,
}
impl RegistryData {
    pub fn string(value: &str) -> Self {
        Self {
            kind: REG_SZ,
            bytes: value
                .encode_utf16()
                .chain(Some(0))
                .flat_map(u16::to_le_bytes)
                .collect(),
        }
    }
}
fn error() -> AppError {
    AppError::new(
        "registry_unavailable",
        "用户级打开方式注册项不可读取或写入。",
        "integration",
    )
}
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
fn allowed(key: &str, name: &str) -> bool {
    if name.contains('\0') {
        return false;
    }
    match key {
        "Software\\Classes\\MpvKit.Media"
        | "Software\\Classes\\MpvKit.Media\\shell\\open\\command" => name.is_empty(),
        "Software\\MpvKit\\Capabilities" => {
            matches!(name, "ApplicationName" | "ApplicationDescription")
        }
        "Software\\MpvKit\\Capabilities\\FileAssociations" => [
            ".mkv", ".mp4", ".webm", ".avi", ".mov", ".mp3", ".flac", ".wav", ".m4v", ".ts",
        ]
        .contains(&name),
        "Software\\RegisteredApplications" => name == "MpvKit",
        _ => false,
    }
}
#[derive(Clone)]
pub struct Registry {
    sandbox: Option<PathBuf>,
    #[cfg(test)]
    hive: Option<std::sync::Arc<offline::Hive>>,
}
impl Registry {
    pub fn current(root: &Path) -> Self {
        #[cfg(debug_assertions)]
        if std::env::var_os("MPVKIT_TEST_MODE").is_some_and(|v| v == "1") {
            return Self::isolated(root);
        }
        let _ = root;
        Self {
            sandbox: None,
            #[cfg(test)]
            hive: None,
        }
    }
    #[cfg(any(debug_assertions, test))]
    fn isolated(root: &Path) -> Self {
        Self {
            sandbox: Some(root.join(".mpvkit/test-registry.json")),
            #[cfg(test)]
            hive: None,
        }
    }
    #[cfg(test)]
    pub fn test(root: &Path) -> Self {
        Self::isolated(root)
    }
    #[cfg(test)]
    pub fn offline(root: &Path) -> Result<Self> {
        Ok(Self {
            sandbox: None,
            hive: Some(std::sync::Arc::new(offline::Hive::open(
                &root.join(".mpvkit/test-registry.hiv"),
            )?)),
        })
    }
    fn handle(&self) -> HKEY {
        #[cfg(test)]
        if let Some(hive) = &self.hive {
            return hive.handle();
        }
        HKEY_CURRENT_USER
    }
    fn map(&self) -> Result<BTreeMap<String, RegistryData>> {
        let path = self.sandbox.as_ref().expect("isolated registry");
        crate::platform::validate_local_path(path)?;
        if !path.exists() {
            return Ok(Default::default());
        }
        let bytes = crate::deploy::read_bytes(path)?;
        if bytes.len() > 1024 * 1024 {
            return Err(error());
        }
        serde_json::from_slice(&bytes).map_err(|_| error())
    }
    pub fn read(&self, key: &str, name: &str) -> Result<Option<RegistryData>> {
        if !allowed(key, name) {
            return Err(error());
        }
        if self.sandbox.is_some() {
            return Ok(self.map()?.get(&format!("{key}|{name}")).cloned());
        }
        let mut handle = std::ptr::null_mut();
        let result =
            unsafe { RegOpenKeyExW(self.handle(), wide(key).as_ptr(), 0, KEY_READ, &mut handle) };
        if result == 2 {
            return Ok(None);
        }
        if result != 0 {
            return Err(error());
        }
        let mut kind = 0;
        let mut size = 0;
        let result = unsafe {
            RegQueryValueExW(
                handle,
                wide(name).as_ptr(),
                std::ptr::null(),
                &mut kind,
                std::ptr::null_mut(),
                &mut size,
            )
        };
        if result == 2 {
            unsafe {
                RegCloseKey(handle);
            }
            return Ok(None);
        }
        if result != 0 || size > 65536 {
            unsafe {
                RegCloseKey(handle);
            }
            return Err(error());
        }
        let mut bytes = vec![0; size as usize];
        let result = unsafe {
            RegQueryValueExW(
                handle,
                wide(name).as_ptr(),
                std::ptr::null(),
                &mut kind,
                bytes.as_mut_ptr(),
                &mut size,
            )
        };
        unsafe {
            RegCloseKey(handle);
        }
        if result != 0 || size as usize > bytes.len() {
            return Err(error());
        }
        bytes.truncate(size as usize);
        Ok(Some(RegistryData { kind, bytes }))
    }
    pub fn write(&self, key: &str, name: &str, data: Option<&RegistryData>) -> Result<()> {
        if !allowed(key, name) || data.is_some_and(|d| d.bytes.len() > 65536) {
            return Err(error());
        }
        if let Some(path) = &self.sandbox {
            let mut map = self.map()?;
            let id = format!("{key}|{name}");
            if let Some(data) = data {
                map.insert(id, data.clone());
            } else {
                map.remove(&id);
            }
            return crate::deploy::atomic_write(
                path,
                &serde_json::to_vec_pretty(&map).map_err(|_| error())?,
            );
        }
        let mut handle = std::ptr::null_mut();
        let result = if data.is_some() {
            unsafe {
                RegCreateKeyExW(
                    self.handle(),
                    wide(key).as_ptr(),
                    0,
                    std::ptr::null(),
                    0,
                    KEY_READ | KEY_WRITE,
                    std::ptr::null(),
                    &mut handle,
                    std::ptr::null_mut(),
                )
            }
        } else {
            unsafe { RegOpenKeyExW(self.handle(), wide(key).as_ptr(), 0, KEY_WRITE, &mut handle) }
        };
        if result == 2 && data.is_none() {
            return Ok(());
        }
        if result != 0 {
            return Err(error());
        }
        let result = if let Some(data) = data {
            unsafe {
                RegSetValueExW(
                    handle,
                    wide(name).as_ptr(),
                    0,
                    data.kind,
                    data.bytes.as_ptr(),
                    data.bytes.len() as u32,
                )
            }
        } else {
            unsafe { RegDeleteValueW(handle, wide(name).as_ptr()) }
        };
        unsafe {
            RegCloseKey(handle);
        }
        if result == 0 || (result == 2 && data.is_none()) {
            Ok(())
        } else {
            Err(error())
        }
    }
}
