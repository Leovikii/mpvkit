use crate::{
    deploy,
    error::{AppError, Result},
    platform,
};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{
    io::Read,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tauri::State;

const PREFIX: &str = "https://github.com/Leovikii/mpvkit/releases/download/";
const INDEX: &str = "https://github.com/Leovikii/mpvkit/releases/download/updates/channels.json";
fn error(code: &'static str) -> AppError {
    AppError::new(
        code,
        "无法完成更新操作，请检查网络、发布状态或当前任务后重试。",
        "update",
    )
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Release {
    version: Version,
    manifest: String,
    arch_package: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Index {
    schema_version: u32,
    stable: Option<Release>,
    preview: Option<Release>,
}
#[derive(Deserialize)]
struct Manifest {
    version: Version,
    notes: String,
    pub_date: Option<String>,
    platforms: std::collections::BTreeMap<String, Asset>,
}
#[derive(Deserialize)]
struct Asset {
    url: String,
    signature: String,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Available {
    id: String,
    version: String,
    notes: String,
    date: Option<String>,
    method: &'static str,
    package: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    status: &'static str,
    update: Option<Available>,
}
struct Candidate {
    public: Available,
    preview: bool,
    signature: String,
    manifest: String,
    asset: String,
    created: Instant,
}
#[derive(Default)]
pub struct Pending {
    generation: AtomicU64,
    candidate: Mutex<Option<Candidate>>,
}

fn release_url(value: &str, version: Option<&Version>) -> Result<()> {
    let url = reqwest::Url::parse(value).map_err(|_| error("update_metadata"))?;
    if !value.starts_with(PREFIX)
        || value.contains('%')
        || url.as_str() != value
        || url.query().is_some()
        || url.fragment().is_some()
        || url.username() != ""
        || url.password().is_some()
        || url.port().is_some()
        || url
            .path_segments()
            .is_none_or(|mut parts| parts.any(|p| p == ".." || p.contains('%')))
        || version.is_some_and(|v| !value.starts_with(&format!("{PREFIX}v{v}/")))
    {
        return Err(error("update_metadata"));
    }
    Ok(())
}
fn select(index: Index, preview: bool) -> Result<Option<Release>> {
    if index.schema_version != 1
        || index
            .stable
            .as_ref()
            .is_some_and(|r| !r.version.pre.is_empty())
    {
        return Err(error("update_metadata"));
    }
    let candidates = [index.stable, if preview { index.preview } else { None }];
    for release in candidates.iter().flatten() {
        release_url(&release.manifest, Some(&release.version))?;
    }
    Ok(candidates
        .into_iter()
        .flatten()
        .max_by(|a, b| a.version.cmp_precedence(&b.version)))
}
fn fetch(url: &str) -> Result<Vec<u8>> {
    release_url(url, None)?;
    #[cfg(debug_assertions)]
    if std::env::var_os("MPVKIT_TEST_MODE").is_some_and(|v| v == "1") {
        // Fixed isolated files only; a test never contacts the public channel or runs an installer.
        let name = if url == INDEX {
            "channels.json".into()
        } else {
            let relative = url
                .strip_prefix(PREFIX)
                .ok_or_else(|| error("update_metadata"))?;
            format!("{}.json", relative.split('/').next().unwrap_or_default())
        };
        let path = platform::default_root()?
            .join("test-tools/updates")
            .join(name);
        platform::validate_local_path(&path)?;
        let bytes = std::fs::read(path).map_err(|_| error("update_network"))?;
        if bytes.len() > 512 * 1024 {
            return Err(error("update_metadata"));
        }
        return Ok(bytes);
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(8))
        .user_agent(concat!("MpvKit/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|_| error("update_network"))?;
    let response = client
        .get(url)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(|_| error("update_network"))?;
    let mut bytes = Vec::new();
    response
        .take(512 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| error("update_network"))?;
    if bytes.len() > 512 * 1024 {
        return Err(error("update_metadata"));
    }
    Ok(bytes)
}
fn inspect(preview: bool) -> Result<(Check, Option<Candidate>)> {
    let index = serde_json::from_slice(&fetch(INDEX)?).map_err(|_| error("update_metadata"))?;
    let Some(release) = select(index, preview)? else {
        return Ok((
            Check {
                status: "noRelease",
                update: None,
            },
            None,
        ));
    };
    let current =
        Version::parse(env!("CARGO_PKG_VERSION")).map_err(|_| error("update_metadata"))?;
    if release.version.cmp_precedence(&current).is_le() {
        return Ok((
            Check {
                status: "current",
                update: None,
            },
            None,
        ));
    }
    let manifest: Manifest =
        serde_json::from_slice(&fetch(&release.manifest)?).map_err(|_| error("update_metadata"))?;
    if manifest.version != release.version || manifest.notes.len() > 128 * 1024 {
        return Err(error("update_metadata"));
    }
    let package = release.arch_package;
    if package
        .as_ref()
        .is_some_and(|p| !["mpvkit-bin", "mpvkit-preview-bin"].contains(&p.as_str()))
    {
        return Err(error("update_metadata"));
    }
    let asset = manifest
        .platforms
        .get("windows-x86_64")
        .ok_or_else(|| error("update_metadata"))?;
    release_url(&asset.url, Some(&release.version))?;
    if asset.signature.trim().is_empty() || asset.signature.len() > 4096 {
        return Err(error("update_metadata"));
    }
    let public = Available {
        id: deploy::operation_id(),
        version: release.version.to_string(),
        notes: manifest.notes,
        date: manifest.pub_date,
        method: if cfg!(target_os = "windows") {
            "installer"
        } else {
            "aur"
        },
        package,
    };
    Ok((
        Check {
            status: "available",
            update: Some(public.clone()),
        },
        Some(Candidate {
            public,
            preview,
            signature: asset.signature.clone(),
            manifest: release.manifest,
            asset: asset.url.clone(),
            created: Instant::now(),
        }),
    ))
}
#[tauri::command]
pub async fn check_app_update(preview: bool, state: State<'_, Pending>) -> Result<Check> {
    let generation = state.generation.fetch_add(1, Ordering::SeqCst) + 1;
    *state
        .candidate
        .lock()
        .map_err(|_| error("state_unavailable"))? = None;
    let (result, candidate) = tauri::async_runtime::spawn_blocking(move || inspect(preview))
        .await
        .map_err(|_| error("worker_failed"))??;
    let mut pending = state
        .candidate
        .lock()
        .map_err(|_| error("state_unavailable"))?;
    if state.generation.load(Ordering::SeqCst) != generation {
        return Err(error("stale_preview"));
    }
    *pending = candidate;
    Ok(result)
}
#[tauri::command]
pub async fn install_app_update(
    id: String,
    app: tauri::AppHandle,
    state: State<'_, Pending>,
    on_progress: tauri::ipc::Channel<u64>,
) -> Result<()> {
    let candidate = state
        .candidate
        .lock()
        .map_err(|_| error("state_unavailable"))?
        .take()
        .ok_or_else(|| error("stale_preview"))?;
    if candidate.public.id != id || candidate.created.elapsed() > Duration::from_secs(1800) {
        return Err(error("stale_preview"));
    }
    if crate::app_settings::load(&platform::default_root()?)?
        .settings
        .preview_updates
        != candidate.preview
    {
        return Err(error("stale_preview"));
    }
    #[cfg(debug_assertions)]
    if std::env::var_os("MPVKIT_TEST_MODE").is_some_and(|v| v == "1") {
        return Err(error("update_test_install_blocked"));
    }
    #[cfg(target_os = "windows")]
    {
        use tauri_plugin_updater::UpdaterExt;
        let key = app
            .config()
            .plugins
            .0
            .get("updater")
            .and_then(|config| config.get("pubkey"))
            .and_then(serde_json::Value::as_str)
            .filter(|key| !key.trim().is_empty())
            .ok_or_else(|| error("update_signing_missing"))?;
        let update = app
            .updater_builder()
            .pubkey(key)
            .endpoints(vec![
                candidate
                    .manifest
                    .parse()
                    .map_err(|_| error("update_metadata"))?,
            ])
            .map_err(|_| error("update_metadata"))?
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|_| error("update_metadata"))?
            .check()
            .await
            .map_err(|_| error("update_network"))?
            .ok_or_else(|| error("stale_preview"))?;
        if update.version != candidate.public.version
            || update.download_url.as_str() != candidate.asset
            || update.signature != candidate.signature
        {
            return Err(error("update_metadata"));
        }
        let mut received = 0u64;
        let bytes = update
            .download(
                |size, total| {
                    received += size as u64;
                    let _ = on_progress.send(
                        total
                            .filter(|v| *v > 0)
                            .map(|v| received.saturating_mul(100) / v)
                            .unwrap_or(0)
                            .min(100),
                    );
                },
                || {},
            )
            .await
            .map_err(|_| error("update_download"))?;
        // Acquire both roots before handing control to the native installer. This blocks helpers,
        // other manager instances and package operations until this process exits.
        let data = platform::default_root()?;
        let config = platform::app_root()?;
        let _data_lock = deploy::acquire(&data)?;
        let _config_lock = if config != data {
            Some(deploy::acquire(&config)?)
        } else {
            None
        };
        if deploy::status(&data)?.recovery_required || deploy::status(&config)?.recovery_required {
            return Err(error("recovery_required"));
        }
        update.install(bytes).map_err(|_| error("update_install"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (
            candidate.signature,
            candidate.manifest,
            candidate.asset,
            app,
            on_progress,
        );
        Err(error("update_package_managed"))
    }
}

#[cfg(all(test, target_os = "windows"))]
mod download_tests;

#[cfg(test)]
mod tests {
    use super::*;
    fn release(version: &str) -> Release {
        Release {
            version: version.parse().unwrap(),
            manifest: format!("{PREFIX}v{version}/updater.json"),
            arch_package: Some("mpvkit-bin".into()),
        }
    }
    #[test]
    fn channels_never_offer_preview_when_disabled_and_compare_semver() {
        let index = || Index {
            schema_version: 1,
            stable: Some(release("0.2.0")),
            preview: Some(release("0.3.0-alpha.2")),
        };
        assert_eq!(
            select(index(), false).unwrap().unwrap().version.to_string(),
            "0.2.0"
        );
        assert_eq!(
            select(index(), true).unwrap().unwrap().version.to_string(),
            "0.3.0-alpha.2"
        );
        assert!(
            select(
                Index {
                    schema_version: 1,
                    stable: None,
                    preview: Some(release("0.2.0-alpha.2"))
                },
                false
            )
            .unwrap()
            .is_none()
        );
        assert!(
            select(
                Index {
                    schema_version: 1,
                    stable: Some(release("0.2.0-alpha.1")),
                    preview: None
                },
                false
            )
            .is_err()
        );
        assert_eq!(
            select(
                Index {
                    schema_version: 1,
                    stable: Some(release("0.2.0")),
                    preview: Some(release("0.2.0-rc.1"))
                },
                true
            )
            .unwrap()
            .unwrap()
            .version
            .to_string(),
            "0.2.0"
        );
        for bad in [
            "http://github.com/Leovikii/mpvkit/releases/download/v0.2.0/x",
            "https://github.com/other/mpvkit/releases/download/v0.2.0/x",
            "https://github.com/Leovikii/mpvkit/releases/download/v0.2.0/%2e%2e/x",
        ] {
            assert!(release_url(bad, Some(&"0.2.0".parse().unwrap())).is_err());
        }
    }
}
