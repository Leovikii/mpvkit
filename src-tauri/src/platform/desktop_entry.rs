//! A single user-level desktop entry, backed by the existing integration transaction.
use crate::{
    deploy,
    error::{AppError, Result},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};
pub const ENTRY: &str = "io.github.mpvkit.player.desktop";
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Data {
    pub text: String,
}
impl Data {
    pub fn string(text: &str) -> Self {
        Self { text: text.into() }
    }
}
fn error() -> AppError {
    AppError::new(
        "desktop_entry_unavailable",
        "用户级打开方式文件不可访问，或 MIME 缓存更新失败。",
        "integration",
    )
}
#[derive(Clone)]
pub struct Store {
    directory: Option<PathBuf>,
}
impl Store {
    pub fn current(root: &Path) -> Self {
        #[cfg(debug_assertions)]
        if cfg!(test) || std::env::var_os("MPVKIT_TEST_MODE").is_some_and(|v| v == "1") {
            return Self {
                directory: Some(root.join(".mpvkit/test-applications")),
            };
        }
        let _ = root;
        Self {
            directory: super::xdg_directory("XDG_DATA_HOME", ".local/share")
                .ok()
                .map(|p| p.join("applications")),
        }
    }
    fn path(&self, key: &str, name: &str) -> Result<PathBuf> {
        if key != ENTRY || !name.is_empty() {
            return Err(error());
        }
        let path = self.directory.as_ref().ok_or_else(error)?.join(ENTRY);
        super::validate_local_path(&path)?;
        Ok(path)
    }
    pub fn read(&self, key: &str, name: &str) -> Result<Option<Data>> {
        let path = self.path(key, name)?;
        if deploy::file_hash(&path)?.is_none() {
            return Ok(None);
        }
        let bytes = deploy::read_bytes(&path)?;
        if bytes.len() > 65536 {
            return Err(error());
        }
        Ok(Some(Data {
            text: String::from_utf8(bytes).map_err(|_| error())?,
        }))
    }
    pub fn write(&self, key: &str, name: &str, data: Option<&Data>) -> Result<()> {
        let path = self.path(key, name)?;
        match data {
            Some(data) if data.text.len() <= 65536 => {
                deploy::atomic_write(&path, data.text.as_bytes())?
            }
            Some(_) => return Err(error()),
            None => match fs::remove_file(&path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => return Err(error()),
            },
        }
        // This tool rebuilds a derived index only; it never changes mimeapps.list/defaults.
        if Path::new("/usr/bin/update-desktop-database").is_file() {
            crate::mpv::capture(
                Path::new("/usr/bin/update-desktop-database"),
                &[path.parent().expect("entry parent").as_os_str().to_owned()],
                Duration::from_secs(5),
            )?;
        }
        Ok(())
    }
}
pub(super) fn exec_argument(value: &str) -> Result<String> {
    if value.chars().any(char::is_control) {
        return Err(error());
    }
    // Exec quoting first, desktop-entry string escaping second (no shell involved).
    let mut quoted = String::from("\"");
    for c in value.chars() {
        match c {
            '\\' | '"' | '`' | '$' => {
                quoted.push('\\');
                quoted.push(c);
            }
            '%' => quoted.push_str("%%"),
            _ => quoted.push(c),
        }
    }
    quoted.push('"');
    Ok(quoted.replace('\\', "\\\\"))
}
pub fn content(root: &Path) -> Result<String> {
    let helper = crate::config::helper_path(root);
    super::validate_local_path(&helper)?;
    let executable = exec_argument(helper.to_str().ok_or_else(error)?)?;
    let icon = root.join(super::DESKTOP_ICON);
    super::validate_local_path(&icon)?;
    let icon = icon.to_str().ok_or_else(error)?.replace('\\', "\\\\");
    if icon.chars().any(char::is_control) {
        return Err(error());
    }
    Ok(format!(
        "[Desktop Entry]\nType=Application\nName=MPV (MpvKit)\nComment=Play media with MpvKit\nExec={executable} --open %f\nIcon={icon}\nTerminal=false\nNoDisplay=true\nCategories=AudioVideo;Player;\nMimeType=video/x-matroska;video/mp4;video/webm;video/x-msvideo;video/quicktime;audio/mpeg;audio/flac;audio/x-wav;video/mp2t;\n"
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::integration;
    #[test]
    fn quoting_preserves_spaces_field_codes_and_special_characters() {
        assert_eq!(exec_argument("/tmp/a b%f").unwrap(), "\"/tmp/a b%%f\"");
        assert_eq!(
            exec_argument("a\"$`\\").unwrap(),
            "\"a\\\\\"\\\\$\\\\`\\\\\\\\\""
        );
        assert!(exec_argument("/tmp/a\nExec=bad").is_err());
    }
    #[test]
    fn registration_stale_preview_external_edits_and_recovery_are_isolated() {
        let root = std::env::temp_dir().join(format!("mpvkit 桌面-%f-{}", deploy::operation_id()));
        let mut payloads = crate::config::runtime_files().unwrap();
        payloads.extend(
            crate::config::generate(&root, Path::new("/usr/bin/mpv"), &Default::default())
                .unwrap()
                .into_iter()
                .filter(|p| !payloads_contains_runtime(&p.path)),
        );
        deploy::execute(
            deploy::prepare_install(
                &root,
                payloads,
                false,
                std::collections::BTreeMap::from([("mpv".into(), "fixture".into())]),
            )
            .unwrap(),
        )
        .unwrap();
        let store = Store::current(&root);
        let plan = integration::prepare(&root, "register").unwrap();
        assert!(store.read(ENTRY, "").unwrap().is_none());
        integration::execute(plan).unwrap();
        assert!(integration::status(&root).unwrap().valid);
        let icon = root.join(crate::platform::DESKTOP_ICON);
        assert_eq!(
            fs::read(&icon).unwrap(),
            include_bytes!("../../icons/icon.png")
        );
        assert!(
            store
                .read(ENTRY, "")
                .unwrap()
                .unwrap()
                .text
                .contains(&format!("\nIcon={}\n", icon.display()))
        );
        fs::write(&icon, b"external edit").unwrap();
        assert!(integration::prepare(&root, "register").is_err());
        fs::write(&icon, include_bytes!("../../icons/icon.png")).unwrap();
        assert_eq!(
            fs::metadata(crate::config::helper_path(&root))
                .unwrap()
                .permissions()
                .mode()
                & 0o700,
            0o700
        );
        let plan = integration::prepare(&root, "unregister").unwrap();
        store
            .write(ENTRY, "", Some(&Data::string("external")))
            .unwrap();
        assert_eq!(
            integration::execute(plan).err().unwrap().code,
            "stale_preview"
        );
        assert!(!integration::status(&root).unwrap().valid);
        integration::execute(integration::prepare(&root, "unregister").unwrap()).unwrap();
        assert_eq!(store.read(ENTRY, "").unwrap().unwrap().text, "external");
        assert!(
            !integration::prepare(&root, "register")
                .unwrap()
                .preview
                .conflicts
                .is_empty()
        );
        assert!(store.write("../outside", "", None).is_err());
        store.write(ENTRY, "", None).unwrap();
        // Interrupt after the external file was written; recovery restores the pre-operation state.
        let plan = integration::prepare(&root, "register").unwrap();
        integration::test_interrupt(&plan);
        assert!(integration::status(&root).unwrap().recovery_required);
        integration::execute(integration::prepare(&root, "recover").unwrap()).unwrap();
        assert!(store.read(ENTRY, "").unwrap().is_none());
        assert!(!integration::status(&root).unwrap().recovery_required);
        fs::remove_dir_all(root).unwrap();
    }
    use std::os::unix::fs::PermissionsExt;
    fn payloads_contains_runtime(path: &str) -> bool {
        path.starts_with(".mpvkit/launcher/")
            || path == crate::config::relative!("scripts/mpvkit.lua")
    }
}
