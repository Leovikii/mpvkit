use crate::{
    deploy,
    error::{AppError, Result},
    platform,
};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tauri::State;
use tauri_plugin_updater::UpdaterExt;

const PREFIX: &str = "https://github.com/Leovikii/mpvkit/releases/download/";
const API: &str = "https://api.github.com/repos/Leovikii/mpvkit/releases";
fn error(code: &'static str) -> AppError {
    AppError::new(
        code,
        "无法完成更新操作，请检查网络、发布状态或当前任务后重试。",
        "update",
    )
}
#[derive(Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<ReleaseAsset>,
}
#[derive(Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
    digest: Option<String>,
}
#[derive(Deserialize)]
struct Manifest {
    version: Version,
    notes: Option<String>,
    pub_date: Option<String>,
    platforms: std::collections::BTreeMap<String, Asset>,
}
#[derive(Deserialize)]
struct Asset {
    url: String,
    signature: String,
}
#[derive(Clone, Serialize)]
pub struct Available {
    id: String,
    version: String,
    notes: String,
    date: Option<String>,
    method: &'static str,
}
#[derive(Serialize)]
pub struct Check {
    status: &'static str,
    update: Option<Available>,
}
#[derive(Serialize)]
pub struct Prepared {
    id: String,
    changes: Vec<String>,
}
#[derive(Clone, Serialize)]
pub struct Progress {
    phase: &'static str,
    percent: Option<u64>,
}
struct Candidate {
    public: Available,
    preview: bool,
    signature: String,
    manifest: String,
    asset: String,
    digest: String,
    created: Instant,
    #[cfg(target_os = "linux")]
    package: Option<platform::app_update::Plan>,
    #[cfg(target_os = "windows")]
    installer: Option<(tauri_plugin_updater::Update, Vec<u8>)>,
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
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url
            .path_segments()
            .is_none_or(|mut p| p.any(|v| v == ".." || v.contains('%')))
        || version.is_some_and(|v| !value.starts_with(&format!("{PREFIX}v{v}/")))
    {
        return Err(error("update_metadata"));
    }
    Ok(())
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn digest(asset: &ReleaseAsset) -> Result<&str> {
    asset
        .digest
        .as_deref()
        .and_then(|d| d.strip_prefix("sha256:"))
        .filter(|d| {
            d.len() == 64
                && d.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        })
        .ok_or_else(|| error("update_metadata"))
}
fn files(version: &Version) -> [String; 6] {
    let v = version.to_string();
    let pkg = v
        .replace("-alpha.", "alpha")
        .replace("-beta.", "beta")
        .replace("-rc.", "rc");
    let win = format!("MpvKit_{v}_x64-setup.exe");
    let linux = format!("mpvkit-bin-{pkg}-1-x86_64.pkg.tar.zst");
    [
        format!("{win}.sig"),
        format!("{linux}.tauri.sig"),
        win,
        linux,
        "SHA256SUMS".into(),
        "latest.json".into(),
    ]
}
fn select(releases: Vec<Release>, preview: bool) -> Result<Option<(Version, Release)>> {
    let mut selected: Option<(Version, Release)> = None;
    for release in releases {
        if release.draft || (!preview && release.prerelease) {
            continue;
        }
        let Some(version) = release
            .tag_name
            .strip_prefix('v')
            .and_then(|v| Version::parse(v).ok())
        else {
            continue;
        };
        if release.prerelease == version.pre.is_empty() {
            return Err(error("update_metadata"));
        }
        if !preview && !version.pre.is_empty() {
            continue;
        }
        let names = files(&version);
        if !names
            .iter()
            .all(|name| release.assets.iter().any(|a| &a.name == name))
        {
            continue;
        }
        for name in &names {
            let assets: Vec<_> = release.assets.iter().filter(|a| &a.name == name).collect();
            if assets.len() != 1 {
                return Err(error("update_metadata"));
            }
            if assets[0].browser_download_url != format!("{PREFIX}{}/{name}", release.tag_name) {
                return Err(error("update_metadata"));
            }
            digest(assets[0])?;
        }
        if selected
            .as_ref()
            .is_none_or(|(v, _)| version.cmp_precedence(v).is_gt())
        {
            selected = Some((version, release));
        }
    }
    Ok(selected)
}
fn fetch(url: &str) -> Result<Vec<u8>> {
    let page_prefix = format!("{API}?per_page=30&page=");
    let page = url.strip_prefix(&page_prefix);
    if page.is_none() {
        release_url(url, None)?;
    }
    if page.is_some_and(|p| p.parse::<u32>().is_err()) {
        return Err(error("update_metadata"));
    }
    #[cfg(debug_assertions)]
    if std::env::var_os("MPVKIT_TEST_MODE").is_some_and(|v| v == "1") {
        let name = if let Some(page) = page {
            format!("releases-{page}.json")
        } else {
            let relative = url
                .strip_prefix(PREFIX)
                .ok_or_else(|| error("update_metadata"))?;
            let (tag, file) = relative
                .split_once('/')
                .ok_or_else(|| error("update_metadata"))?;
            if file == "latest.json" {
                format!("{tag}.json")
            } else {
                format!("{tag}-{file}")
            }
        };
        let path = platform::default_root()?
            .join("test-tools/updates")
            .join(name);
        platform::validate_local_path(&path)?;
        let bytes = std::fs::read(path).map_err(|_| error("update_network"))?;
        if bytes.len() > 2 * 1024 * 1024 {
            return Err(error("update_metadata"));
        }
        return Ok(bytes);
    }
    let response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent(concat!("MpvKit/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|_| error("update_network"))?
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(|_| error("update_network"))?;
    let mut bytes = Vec::new();
    response
        .take(2 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| error("update_network"))?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err(error("update_metadata"));
    }
    Ok(bytes)
}
fn releases() -> Result<Vec<Release>> {
    let mut all = Vec::new();
    // ponytail: bound discovery to 600 releases; use a server feed if this repository outgrows it.
    for page in 1..=20 {
        let batch: Vec<Release> =
            serde_json::from_slice(&fetch(&format!("{API}?per_page=30&page={page}"))?)
                .map_err(|_| error("update_metadata"))?;
        let last = batch.len() < 30;
        all.extend(batch);
        if last {
            return Ok(all);
        }
    }
    Err(error("update_metadata"))
}
fn inspect(preview: bool) -> Result<(Check, Option<Candidate>)> {
    let Some((version, release)) = select(releases()?, preview)? else {
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
    if version.cmp_precedence(&current).is_le() {
        return Ok((
            Check {
                status: "current",
                update: None,
            },
            None,
        ));
    }
    let metadata = release
        .assets
        .iter()
        .find(|a| a.name == "latest.json")
        .ok_or_else(|| error("update_metadata"))?;
    let bytes = fetch(&metadata.browser_download_url)?;
    if hash(&bytes) != digest(metadata)? {
        return Err(error("update_metadata"));
    }
    let sums = release
        .assets
        .iter()
        .find(|a| a.name == "SHA256SUMS")
        .ok_or_else(|| error("update_metadata"))?;
    let checksums = fetch(&sums.browser_download_url)?;
    if hash(&checksums) != digest(sums)? {
        return Err(error("update_metadata"));
    }
    verify_checksums(&checksums, &release)?;
    let manifest: Manifest =
        serde_json::from_slice(&bytes).map_err(|_| error("update_metadata"))?;
    let notes = manifest.notes.unwrap_or_default();
    if manifest.version != version || notes.len() > 128 * 1024 {
        return Err(error("update_metadata"));
    }
    let target = if cfg!(target_os = "windows") {
        "windows-x86_64"
    } else {
        "linux-x86_64"
    };
    let asset = manifest
        .platforms
        .get(target)
        .ok_or_else(|| error("update_metadata"))?;
    release_url(&asset.url, Some(&version))?;
    let file = release
        .assets
        .iter()
        .find(|a| a.browser_download_url == asset.url)
        .ok_or_else(|| error("update_metadata"))?;
    let names = files(&version);
    if file.name != names[if cfg!(target_os = "windows") { 2 } else { 3 }]
        || asset.signature.trim().is_empty()
        || asset.signature.len() > 4096
    {
        return Err(error("update_metadata"));
    }
    let public = Available {
        id: deploy::operation_id(),
        version: version.to_string(),
        notes,
        date: manifest.pub_date,
        method: if cfg!(target_os = "windows") {
            "installer"
        } else {
            "pacman"
        },
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
            manifest: metadata.browser_download_url.clone(),
            asset: asset.url.clone(),
            digest: digest(file)?.into(),
            created: Instant::now(),
            #[cfg(target_os = "linux")]
            package: None,
            #[cfg(target_os = "windows")]
            installer: None,
        }),
    ))
}
fn verify_checksums(bytes: &[u8], release: &Release) -> Result<()> {
    let text = std::str::from_utf8(bytes).map_err(|_| error("update_metadata"))?;
    let mut seen = std::collections::BTreeSet::new();
    for line in text.lines() {
        let (sum, name) = line
            .split_once("  ")
            .ok_or_else(|| error("update_metadata"))?;
        let asset = release
            .assets
            .iter()
            .find(|a| a.name == name && name != "SHA256SUMS")
            .ok_or_else(|| error("update_metadata"))?;
        if !seen.insert(name) || sum != digest(asset)? {
            return Err(error("update_metadata"));
        }
    }
    if seen.len() != 5 {
        return Err(error("update_metadata"));
    }
    Ok(())
}
#[tauri::command]
pub async fn check_app_update(preview: bool, state: State<'_, Pending>) -> Result<Check> {
    let generation = state.generation.fetch_add(1, Ordering::SeqCst) + 1;
    *state
        .candidate
        .lock()
        .map_err(|_| error("state_unavailable"))? = None;
    let (result, candidate) = tauri::async_runtime::spawn_blocking(move || {
        let result = inspect(preview);
        if let Err(e) = &result {
            crate::logging::problem(e.code, "update-check");
        }
        result
    })
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
fn take(state: &Pending, id: &str) -> Result<Candidate> {
    let candidate = state
        .candidate
        .lock()
        .map_err(|_| error("state_unavailable"))?
        .take()
        .ok_or_else(|| error("stale_preview"))?;
    if candidate.public.id != id
        || candidate.created.elapsed() > Duration::from_secs(1800)
        || crate::app_settings::load(&platform::default_root()?)?
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
    Ok(candidate)
}
async fn download(
    candidate: &Candidate,
    app: &tauri::AppHandle,
    progress: tauri::ipc::Channel<Progress>,
) -> Result<(tauri_plugin_updater::Update, Vec<u8>)> {
    let _ = progress.send(Progress {
        phase: "download",
        percent: None,
    });
    let mut update = app
        .updater_builder()
        .endpoints(vec![
            candidate
                .manifest
                .parse()
                .map_err(|_| error("update_metadata"))?,
        ])
        .map_err(|_| error("update_metadata"))?
        .timeout(Duration::from_secs(180))
        .build()
        .map_err(|_| error("update_signing_missing"))?
        .check()
        .await
        .map_err(|_| error("update_network"))?
        .ok_or_else(|| error("stale_preview"))?;
    // The plugin's check timeout is not propagated to its download request.
    update.timeout = Some(Duration::from_secs(180));
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
                let _ = progress.send(Progress {
                    phase: "download",
                    percent: total
                        .filter(|v| *v > 0)
                        .map(|v| (received.saturating_mul(100) / v).min(100)),
                });
            },
            || {
                let _ = progress.send(Progress {
                    phase: "verify",
                    percent: None,
                });
            },
        )
        .await
        .map_err(|_| error("update_download"))?;
    if hash(&bytes) != candidate.digest {
        return Err(error("update_metadata"));
    }
    Ok((update, bytes))
}
#[tauri::command]
pub async fn prepare_app_update(
    id: String,
    app: tauri::AppHandle,
    state: State<'_, Pending>,
    on_progress: tauri::ipc::Channel<Progress>,
) -> Result<Prepared> {
    let mut candidate = take(&state, &id)?;
    let generation = state.generation.load(Ordering::SeqCst);
    let (update, bytes) = download(&candidate, &app, on_progress.clone()).await?;
    let _ = on_progress.send(Progress {
        phase: "prepare",
        percent: None,
    });
    #[cfg(target_os = "linux")]
    let changes = {
        let _ = update;
        let version = candidate.public.version.clone();
        let digest = candidate.digest.clone();
        let plan = tauri::async_runtime::spawn_blocking(move || {
            platform::app_update::prepare(&version, &digest, &bytes)
        })
        .await
        .map_err(|_| error("worker_failed"))??;
        let changes = plan.changes.clone();
        candidate.package = Some(plan);
        changes
    };
    #[cfg(target_os = "windows")]
    let changes = {
        let changes = vec![format!("MpvKit {} (x64)", candidate.public.version)];
        candidate.installer = Some((update, bytes));
        changes
    };
    let mut pending = state
        .candidate
        .lock()
        .map_err(|_| error("state_unavailable"))?;
    if state.generation.load(Ordering::SeqCst) != generation {
        return Err(error("stale_preview"));
    }
    *pending = Some(candidate);
    Ok(Prepared { id, changes })
}
#[tauri::command]
pub fn discard_app_update(state: State<'_, Pending>) -> Result<()> {
    state.generation.fetch_add(1, Ordering::SeqCst);
    *state
        .candidate
        .lock()
        .map_err(|_| error("state_unavailable"))? = None;
    Ok(())
}
#[tauri::command]
pub async fn install_app_update(
    id: String,
    app: tauri::AppHandle,
    state: State<'_, Pending>,
    on_progress: tauri::ipc::Channel<Progress>,
) -> Result<()> {
    let candidate = take(&state, &id)?;
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
    let _ = on_progress.send(Progress {
        phase: "install",
        percent: None,
    });
    #[cfg(target_os = "windows")]
    {
        let (update, bytes) = candidate.installer.ok_or_else(|| error("stale_preview"))?;
        if hash(&bytes) != candidate.digest {
            return Err(error("stale_preview"));
        }
        let _ = app;
        tauri::async_runtime::spawn_blocking(move || {
            crate::logging::record("application", "update", "install", "started", None, None, 0);
            let result = update
                .restart_after_install(true)
                .install(bytes)
                .map_err(|_| error("update_install"));
            if let Err(e) = &result {
                crate::logging::problem(e.code, "update-install");
            }
            result
        })
        .await
        .map_err(|_| error("worker_failed"))?
    }
    #[cfg(target_os = "linux")]
    {
        let plan = candidate.package.ok_or_else(|| error("stale_preview"))?;
        tauri::async_runtime::spawn_blocking(move || {
            let started = std::time::Instant::now();
            crate::logging::record("application", "update", "install", "started", None, None, 0);
            let result = platform::app_update::install(plan);
            crate::logging::record(
                "application",
                "update",
                "install",
                if result.is_ok() { "success" } else { "failed" },
                result.as_ref().err().map(|e| e.code),
                None,
                started.elapsed().as_millis() as u64,
            );
            result
        })
        .await
        .map_err(|_| error("worker_failed"))??;
        // Release file locks before Tauri cleans up the single-instance service and relaunches.
        drop(_config_lock);
        drop(_data_lock);
        app.request_restart();
        Ok(())
    }
}
#[cfg(test)]
mod download_tests;
#[cfg(test)]
mod tests {
    use super::*;
    fn release(v: &str) -> Release {
        let version: Version = v.parse().unwrap();
        Release {
            tag_name: format!("v{v}"),
            draft: false,
            prerelease: !version.pre.is_empty(),
            assets: files(&version)
                .into_iter()
                .map(|name| ReleaseAsset {
                    browser_download_url: format!("{PREFIX}v{v}/{name}"),
                    name,
                    digest: Some(format!("sha256:{}", "a".repeat(64))),
                })
                .collect(),
        }
    }
    #[test]
    fn releases_filter_previews_and_compare_versions_not_dates() {
        assert!(
            select(vec![release("0.2.0-alpha.3")], false)
                .unwrap()
                .is_none()
        );
        let choice = |preview| {
            select(
                vec![
                    release("0.2.0"),
                    release("0.3.0-alpha.3"),
                    release("0.2.0-rc.1"),
                ],
                preview,
            )
            .unwrap()
            .unwrap()
            .0
            .to_string()
        };
        assert_eq!(choice(false), "0.2.0");
        assert_eq!(choice(true), "0.3.0-alpha.3");
        assert_eq!(
            select(vec![release("0.2.0-rc.1"), release("0.2.0")], true)
                .unwrap()
                .unwrap()
                .0
                .to_string(),
            "0.2.0"
        );
        let mut incomplete = release("9.0.0");
        incomplete.assets.pop();
        assert!(select(vec![incomplete], true).unwrap().is_none());
        let mut draft = release("9.0.0");
        draft.draft = true;
        assert!(select(vec![draft], true).unwrap().is_none());
        for kind in ["digest", "url", "duplicate", "channel"] {
            let mut r = release("0.2.0");
            match kind {
                "digest" => r.assets[0].digest = None,
                "url" => r.assets[0].browser_download_url = "https://evil.invalid/x".into(),
                "duplicate" => r.assets.push(ReleaseAsset {
                    name: "latest.json".into(),
                    browser_download_url: "bad".into(),
                    digest: None,
                }),
                _ => r.prerelease = true,
            }
            assert!(select(vec![r], true).is_err(), "{kind}");
        }
        for bad in [
            "http://github.com/Leovikii/mpvkit/releases/download/v0.2.0/x",
            "https://github.com/other/mpvkit/releases/download/v0.2.0/x",
            "https://github.com/Leovikii/mpvkit/releases/download/v0.2.0/%2e%2e/x",
        ] {
            assert!(release_url(bad, Some(&"0.2.0".parse().unwrap())).is_err());
        }
    }
    #[test]
    fn checksums_must_cover_every_asset_once_and_match_release_digests() {
        let release = release("0.2.0");
        let sums = release
            .assets
            .iter()
            .filter(|a| a.name != "SHA256SUMS")
            .map(|a| format!("{}  {}\n", digest(a).unwrap(), a.name))
            .collect::<String>();
        assert!(verify_checksums(sums.as_bytes(), &release).is_ok());
        assert!(
            verify_checksums(
                sums.replace(&"a".repeat(64), &"b".repeat(64)).as_bytes(),
                &release
            )
            .is_err()
        );
        assert!(verify_checksums(format!("{sums}{sums}").as_bytes(), &release).is_err());
        assert!(
            verify_checksums(
                sums.lines()
                    .skip(1)
                    .collect::<Vec<_>>()
                    .join("\n")
                    .as_bytes(),
                &release
            )
            .is_err()
        );
    }
}
