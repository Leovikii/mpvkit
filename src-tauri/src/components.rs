use crate::{
    deploy::{self, Payload},
    error::{AppError, Result},
};
use serde::Serialize;
use std::{
    collections::BTreeSet,
    io::{Cursor, Read},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

const MAX_ARCHIVE: u64 = 64 * 1024 * 1024;
const MAX_FILE: u64 = 256 * 1024 * 1024;
const MAX_EXPANDED: u64 = 512 * 1024 * 1024;
const MAX_ENTRIES: usize = 2048;

fn error(code: &'static str, message: &'static str) -> AppError {
    AppError::new(code, message, "components")
}
fn check_cancel(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        Err(error("cancelled", "获取已取消，安装目录未改变。"))
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    SevenZip,
    Zip,
    Raw,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub id: &'static str,
    pub version: &'static str,
    pub project: &'static str,
    pub url: &'static str,
    pub size: u64,
    pub sha256: &'static str,
    pub format: Format,
}

// A fixed research baseline, not a claim of release readiness. No remote catalog is loaded.
pub const ASSETS: &[Asset] = &[
    Asset {
        id: "mpv",
        version: "20261020-6a262d11f6",
        project: "https://github.com/shinchiro/mpv-winbuild-cmake",
        url: "https://github.com/shinchiro/mpv-winbuild-cmake/releases/download/20261020/mpv-x86_64-20261020-git-6a262d11f6.7z",
        size: 34_014_613,
        sha256: "cbeda25235a24e9145f6ab3e784052e437e6e5aac306b682afd2bbf0070ddea8",
        format: Format::SevenZip,
    },
    Asset {
        id: "uosc",
        version: "5.13.0",
        project: "https://github.com/tomasklaen/uosc",
        url: "https://github.com/tomasklaen/uosc/releases/download/5.13.0/uosc.zip",
        size: 8_163_987,
        sha256: "4be9da3289285300fa374496c3f1bfd7bb20ac08e890d25bd5a06b28eebe4882",
        format: Format::Zip,
    },
    Asset {
        id: "thumbfast",
        version: "0f711de3138c9bd6718209d819ac54022c23ded2",
        project: "https://github.com/po5/thumbfast",
        url: "https://raw.githubusercontent.com/po5/thumbfast/0f711de3138c9bd6718209d819ac54022c23ded2/thumbfast.lua",
        size: 32_495,
        sha256: "a3d08e71eae8b892f6cd39f9593ea219768e709312d176bca883841b156448bf",
        format: Format::Raw,
    },
    Asset {
        id: "subtitle-font",
        version: "1.530",
        project: "https://github.com/lxgw/LxgwWenKai-Screen",
        url: "https://github.com/lxgw/LxgwWenKai-Screen/releases/download/v1.530/LXGWWenKaiScreen.ttf",
        size: 26_095_590,
        sha256: "abd13364163e3cd84372b333f2fdcee9efb848634668f1c0ce5788dcd059ee61",
        format: Format::Raw,
    },
];

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub component: String,
    pub phase: &'static str,
    pub received: u64,
    pub total: u64,
}

fn allowed_url(url: &reqwest::Url) -> bool {
    url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.port_or_known_default() == Some(443)
        && matches!(
            url.host_str(),
            Some(
                "github.com"
                    | "release-assets.githubusercontent.com"
                    | "raw.githubusercontent.com"
                    | "objects.githubusercontent.com"
            )
        )
}

pub fn download(
    asset: &Asset,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<Vec<u8>> {
    check_cancel(cancel)?;
    let url =
        reqwest::Url::parse(asset.url).map_err(|_| error("source_invalid", "组件来源无效。"))?;
    if !allowed_url(&url) || asset.size > MAX_ARCHIVE {
        return Err(error("source_invalid", "组件来源或大小不受支持。"));
    }
    let client = client_builder(Duration::from_secs(600), Duration::from_secs(60))
        .build()
        .map_err(|_| error("network_unavailable", "无法初始化安全下载。"))?;
    let response = client
        .get(url)
        .send()
        .map_err(|_| error("download_failed", "下载失败或超时，请检查网络后重试。"))?;
    check_cancel(cancel)?;
    let mut response = response
        .error_for_status()
        .map_err(|_| error("download_status", "固定来源返回下载错误。"))?;
    if response.content_length().is_some_and(|s| s != asset.size) {
        return Err(error("asset_size", "下载长度与固定清单不符。"));
    }
    let bytes = receive(&mut response, asset.size, cancel, |received| {
        progress(Progress {
            component: asset.id.into(),
            phase: "download",
            received,
            total: asset.size,
        })
    })?;
    verify(asset, &bytes)?;
    Ok(bytes)
}

fn client_builder(total: Duration, stall: Duration) -> reqwest::blocking::ClientBuilder {
    // The async builder bounds the whole body; the blocking timeout bounds each read.
    reqwest::blocking::ClientBuilder::from(
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(total)
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                if attempt.previous().len() >= 5 || !allowed_url(attempt.url()) {
                    attempt.error("untrusted redirect")
                } else {
                    attempt.follow()
                }
            }))
            .user_agent(concat!("MpvKit/", env!("CARGO_PKG_VERSION"))),
    )
    .timeout(stall)
}

fn receive(
    reader: &mut dyn Read,
    expected: u64,
    cancel: &AtomicBool,
    mut progress: impl FnMut(u64),
) -> Result<Vec<u8>> {
    if expected > MAX_ARCHIVE {
        return Err(error("asset_size", "下载大小超过限制。"));
    }
    let mut bytes = Vec::new();
    let mut buffer = [0; 65536];
    loop {
        check_cancel(cancel)?;
        let n = reader
            .read(&mut buffer)
            .map_err(|_| error("download_failed", "下载中断或超时，安装目录未改变。"))?;
        if n == 0 {
            break;
        }
        if bytes.len() as u64 + n as u64 > expected {
            return Err(error("asset_size", "下载超过固定大小限制。"));
        }
        bytes.extend_from_slice(&buffer[..n]);
        progress(bytes.len() as u64);
    }
    check_cancel(cancel)?;
    if bytes.len() as u64 != expected {
        return Err(error("asset_size", "下载不完整，安装目录未改变。"));
    }
    Ok(bytes)
}

pub fn verify(asset: &Asset, bytes: &[u8]) -> Result<()> {
    if bytes.len() as u64 != asset.size || asset.size > MAX_ARCHIVE {
        return Err(error("asset_size", "组件大小与固定清单不符。"));
    }
    if deploy::hash_bytes(bytes) != asset.sha256 {
        return Err(error("asset_hash", "组件 SHA-256 不匹配，已拒绝使用。"));
    }
    Ok(())
}

struct Boundary {
    paths: BTreeSet<String>,
    files: BTreeSet<String>,
    expanded: u64,
}
impl Boundary {
    fn new() -> Self {
        Self {
            paths: BTreeSet::new(),
            files: BTreeSet::new(),
            expanded: 0,
        }
    }
    fn entry(&mut self, name: &str, directory: bool, size: u64, special: bool) -> Result<()> {
        let path = if directory {
            name.strip_suffix('/').unwrap_or(name)
        } else {
            name
        };
        deploy::validate_relative(path)?;
        let key = path.to_lowercase();
        if special || key.split('/').any(|p| p == ".mpvkit") {
            return Err(error("archive_link", "归档含链接、特殊文件或保留路径。"));
        }
        if !self.paths.insert(key.clone()) {
            return Err(error("archive_duplicate", "归档含重复或大小写冲突路径。"));
        }
        for (position, _) in key.match_indices('/') {
            if self.files.contains(&key[..position]) {
                return Err(error("archive_collision", "归档文件与目录路径冲突。"));
            }
        }
        if !directory {
            if self
                .paths
                .iter()
                .any(|p| p.starts_with(&(key.clone() + "/")))
            {
                return Err(error("archive_collision", "归档文件与目录路径冲突。"));
            }
            self.files.insert(key);
        }
        self.expanded = self
            .expanded
            .checked_add(size)
            .ok_or_else(|| error("archive_limit", "归档展开大小超过限制。"))?;
        if self.paths.len() > MAX_ENTRIES
            || size > MAX_FILE
            || self.expanded > MAX_EXPANDED
            || (directory && size != 0)
        {
            return Err(error("archive_limit", "归档条目数或展开大小超过限制。"));
        }
        Ok(())
    }
}

fn destination(id: &str, name: &str) -> Option<String> {
    match id {
        "mpv" if matches!(name, "mpv.exe" | "d3dcompiler_43.dll") => Some(name.into()),
        "mpv" if name == "mpv/fonts.conf" => Some("portable_config/fonts.conf".into()),
        "uosc"
            if name.starts_with("scripts/uosc/")
                && (!name.starts_with("scripts/uosc/bin/")
                    || name == "scripts/uosc/bin/ziggy-windows.exe") =>
        {
            Some(format!("portable_config/{name}"))
        }
        "uosc" if matches!(name, "fonts/uosc_icons.otf" | "fonts/uosc_textures.ttf") => {
            Some(format!("portable_config/{name}"))
        }
        "subtitle-font" => Some("portable_config/fonts/LXGWWenKaiScreen.ttf".into()),
        "thumbfast" => Some("portable_config/scripts/thumbfast.lua".into()),
        _ => None,
    }
}

fn read_entry(
    reader: &mut dyn Read,
    size: u64,
    selected: bool,
    cancel: &AtomicBool,
) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut buffer = [0; 65536];
    let mut count = 0u64;
    loop {
        check_cancel(cancel)?;
        let n = reader
            .read(&mut buffer)
            .map_err(|_| error("archive_corrupt", "归档数据损坏或校验失败。"))?;
        if n == 0 {
            break;
        }
        count += n as u64;
        if count > size || count > MAX_FILE {
            return Err(error("archive_limit", "归档实际展开大小超过声明。"));
        }
        if selected {
            bytes.extend_from_slice(&buffer[..n]);
        }
    }
    if count != size {
        return Err(error("archive_corrupt", "归档内容长度不符。"));
    }
    Ok(bytes)
}

pub fn unpack(asset: &Asset, bytes: &[u8], cancel: &AtomicBool) -> Result<Vec<Payload>> {
    verify(asset, bytes)?; // Authenticate the exact fixed bytes before any parser allocates decoder state.
    check_cancel(cancel)?;
    let mut payloads = Vec::new();
    let mut boundary = Boundary::new();
    match asset.format {
        Format::Raw => payloads.push(Payload {
            path: destination(asset.id, "")
                .ok_or_else(|| error("component_unknown", "未知组件。"))?,
            bytes: bytes.into(),
        }),
        Format::Zip => {
            let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
                .map_err(|_| error("archive_corrupt", "ZIP 归档无效。"))?;
            if archive.len() > MAX_ENTRIES {
                return Err(error("archive_limit", "归档条目过多。"));
            }
            for index in 0..archive.len() {
                let file = archive
                    .by_index(index)
                    .map_err(|_| error("archive_corrupt", "ZIP 条目不可读取。"))?;
                let kind = file.unix_mode().unwrap_or(0) & 0o170000;
                boundary.entry(
                    file.name(),
                    file.is_dir(),
                    file.size(),
                    !matches!(kind, 0 | 0o100000 | 0o040000) || file.encrypted(),
                )?;
            }
            for index in 0..archive.len() {
                check_cancel(cancel)?;
                let mut file = archive
                    .by_index(index)
                    .map_err(|_| error("archive_corrupt", "ZIP 条目不可读取。"))?;
                if file.is_dir() {
                    continue;
                }
                let path = destination(asset.id, file.name());
                let size = file.size();
                let data = read_entry(&mut file, size, path.is_some(), cancel)?;
                if let Some(path) = path {
                    payloads.push(Payload { path, bytes: data });
                }
            }
        }
        Format::SevenZip => {
            let mut archive = sevenz_rust2::ArchiveReader::new(
                Cursor::new(bytes),
                sevenz_rust2::Password::empty(),
            )
            .map_err(|_| error("archive_corrupt", "7z 归档无效。"))?;
            for file in &archive.archive().files {
                let attributes = file.windows_attributes;
                let kind = (attributes >> 16) & 0o170000;
                boundary.entry(
                    &file.name,
                    file.is_directory,
                    file.size,
                    file.is_anti_item
                        || (file.has_windows_attributes
                            && (attributes & 0x400 != 0
                                || !matches!(kind, 0 | 0o100000 | 0o040000))),
                )?;
            }
            let mut failure = None;
            let decoded = archive.for_each_entries(|file, reader| {
                // Solid archives must consume even unselected entries to reach later files.
                let path = (!file.is_directory)
                    .then(|| destination(asset.id, &file.name))
                    .flatten();
                match read_entry(reader, file.size, path.is_some(), cancel) {
                    Ok(data) => {
                        if let Some(path) = path {
                            payloads.push(Payload { path, bytes: data });
                        }
                        Ok(true)
                    }
                    Err(e) => {
                        failure = Some(e);
                        Ok(false)
                    }
                }
            });
            if let Some(e) = failure {
                return Err(e);
            }
            decoded.map_err(|_| error("archive_corrupt", "7z 解码或校验失败。"))?;
        }
    }
    check_cancel(cancel)?;
    let required: &[&str] = match asset.id {
        "mpv" => &[
            "mpv.exe",
            "d3dcompiler_43.dll",
            "portable_config/fonts.conf",
        ],
        "uosc" => &[
            "portable_config/scripts/uosc/main.lua",
            "portable_config/scripts/uosc/bin/ziggy-windows.exe",
            "portable_config/fonts/uosc_icons.otf",
            "portable_config/fonts/uosc_textures.ttf",
        ],
        "thumbfast" => &["portable_config/scripts/thumbfast.lua"],
        "subtitle-font" => &["portable_config/fonts/LXGWWenKaiScreen.ttf"],
        _ => return Err(error("component_unknown", "未知组件。")),
    };
    if required
        .iter()
        .any(|name| !payloads.iter().any(|p| p.path == *name))
    {
        return Err(error("component_incomplete", "固定组件缺少必要文件。"));
    }
    Ok(payloads)
}

#[cfg(test)]
mod tests;

pub fn prepare_install(
    root: &std::path::Path,
    integration_enabled: bool,
    replace_existing: bool,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<deploy::DeploymentPlan> {
    // Inspect ownership and interrupted operations before fetching anything.
    if crate::deploy::status(root)?.recovery_required {
        return Err(AppError::new(
            "recovery_required",
            "请先恢复未完成操作。",
            "components",
        ));
    }
    let mut payloads = Vec::new();
    let mut versions = std::collections::BTreeMap::new();
    for asset in crate::components::ASSETS {
        let bytes = acquire(asset, cancel, progress)?;
        progress(Progress {
            component: asset.id.into(),
            phase: "verify-extract",
            received: asset.size,
            total: asset.size,
        });
        payloads.extend(crate::components::unpack(asset, &bytes, cancel)?);
        versions.insert(asset.id.into(), asset.version.into());
    }
    payloads.extend(license_materials());
    payloads.extend(crate::config::runtime_files()?);
    let state_path = root.join(".mpvkit/settings.json");
    let state_hash = deploy::file_hash(&state_path)?;
    let mut state = crate::config::portable::read_state(root)?;
    state.integration_enabled = integration_enabled;
    state.manual_config |= crate::config::modified(root)?;
    payloads.push(Payload {
        path: ".mpvkit/settings.json".into(),
        bytes: serde_json::to_vec_pretty(&state)
            .map_err(|_| error("settings_encode", "无法生成设置。"))?,
    });
    if let Some(m) = deploy::status(root)?.manifest {
        for f in &m.files {
            if crate::config::editable(&f.path) {
                // Do not recreate missing user configuration during component updates.
                payloads.retain(|p| p.path != f.path);
                if deploy::file_hash(&root.join(&f.path))?.is_some() {
                    payloads.push(Payload {
                        path: f.path.clone(),
                        bytes: deploy::read_bytes(&root.join(&f.path))?,
                    });
                }
            }
        }
    }
    if cancel.load(std::sync::atomic::Ordering::Relaxed) {
        return Err(AppError::new(
            "cancelled",
            "获取已取消，安装目录未改变。",
            "components",
        ));
    }
    let mut plan = crate::deploy::prepare_install(root, payloads, replace_existing, versions)?;
    plan.guard(state_path, state_hash);
    Ok(plan)
}

fn acquire(
    asset: &Asset,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<Vec<u8>> {
    #[cfg(debug_assertions)]
    if std::env::var_os("MPVKIT_TEST_MODE").is_some_and(|v| v == "1")
        && let Some(directory) = std::env::var_os("MPVKIT_TEST_ASSETS")
    {
        let name = match asset.id {
            "mpv" => "mpv-x86_64-20261020-git-6a262d11f6.7z",
            "uosc" => "uosc-5.13.0.zip",
            "thumbfast" => "thumbfast-thumbfast.lua",
            "subtitle-font" => "LXGWWenKaiScreen.ttf",
            _ => return Err(error("component_unknown", "未知组件。")),
        };
        let path = std::path::PathBuf::from(directory).join(name);
        crate::platform::validate_local_path(&path)?;
        let mut file =
            std::fs::File::open(path).map_err(|_| error("asset_missing", "测试固定资产缺失。"))?;
        let bytes = receive(&mut file, asset.size, cancel, |received| {
            progress(Progress {
                component: asset.id.into(),
                phase: "download",
                received,
                total: asset.size,
            })
        })?;
        verify(asset, &bytes)?;
        return Ok(bytes);
    }
    download(asset, cancel, progress)
}

fn license_materials() -> Vec<Payload> {
    let mut files = vec![
        ("LXGWWenKaiScreen-OFL.txt",include_bytes!("../../resources/licenses/LXGWWenKaiScreen-OFL.txt").as_slice()),
        ("uosc-LGPL-2.1.txt",include_bytes!("../../resources/licenses/uosc-LGPL-2.1.txt").as_slice()),
        ("thumbfast-MPL-2.txt",include_bytes!("../../resources/licenses/thumbfast-MPL-2.txt").as_slice()),
        ("material-icons-Apache-2.txt",include_bytes!("../../resources/licenses/material-icons-Apache-2.txt").as_slice()),
        ("uosc-clipboard-BSD-3.txt",include_bytes!("../../resources/licenses/uosc-clipboard-BSD-3.txt").as_slice()),
        ("uosc-browser-BSD-2.txt",include_bytes!("../../resources/licenses/uosc-browser-BSD-2.txt").as_slice()),
        ("uosc-sys-BSD-3.txt",include_bytes!("../../resources/licenses/uosc-sys-BSD-3.txt").as_slice()),
        ("uosc-go-BSD-3.txt",include_bytes!("../../resources/licenses/uosc-go-BSD-3.txt").as_slice()),
        ("uosc-go-PATENTS.txt",include_bytes!("../../resources/licenses/uosc-go-PATENTS.txt").as_slice()),
        ("uosc-go-NOTICES.txt",include_bytes!("../../resources/licenses/uosc-go-NOTICES.txt").as_slice()),
        ("source-materials.json",include_bytes!("../../resources/source-materials.json").as_slice()),
        ("catalog.json",include_bytes!("../../resources/catalog.json").as_slice()),
        ("THIRD-PARTY.txt", "MpvKit 开发基线第三方材料\n\n这些文本保留固定组件已核对的上游许可与来源，不代表聚合分发材料齐备或发布已验收。\nMPV 核心 Copyright 声明 GPLv2 或以后；实际 FFmpeg 构建日志声明 GPLv3 或以后。\nuosc 上游附 LGPL2.1 文本，Go 辅助程序内嵌依赖版本及三份模块许可、Go1.23.1许可/PATENTS已保留；Go源码33份独立声明和14个已审阅版权/许可段落已保守保留；完整对应源码及链接适用仍需审阅。\nuosc_icons.otf 与 Google Material Icons Round 固定提交 1c575764826fa5752ddc2bdc6b31ef7cc3233371 原文件逐字节一致，未修改；Apache-2.0 许可原文已保留。Copyright 2019 Google LLC. All Rights Reserved. 纹理字体的完整许可映射仍未完成。\nthumbfast 固定源码及 MPL2 文本未修改。\n完整固定来源见 catalog.json / source-materials.json；releaseReady=false。\n".as_bytes()),
    ];
    files.extend([
        (
            "mpv-Copyright.txt",
            include_bytes!("../../resources/licenses/mpv-Copyright.txt").as_slice(),
        ),
        (
            "mpv-GPL-2.txt",
            include_bytes!("../../resources/licenses/mpv-GPL-2.txt").as_slice(),
        ),
        (
            "mpv-LGPL-2.1.txt",
            include_bytes!("../../resources/licenses/mpv-LGPL-2.1.txt").as_slice(),
        ),
        (
            "ffmpeg-GPL-3.txt",
            include_bytes!("../../resources/licenses/ffmpeg-GPL-3.txt").as_slice(),
        ),
    ]);
    files
        .into_iter()
        .map(|(name, bytes)| Payload {
            path: format!("licenses/{name}"),
            bytes: bytes.to_vec(),
        })
        .collect()
}
