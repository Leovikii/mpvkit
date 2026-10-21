use crate::platform;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MpvCandidate {
    pub executable: PathBuf,
    pub locations: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedLocation {
    pub source: String,
    pub code: &'static str,
    pub message: &'static str,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryReport {
    pub candidates: Vec<MpvCandidate>,
    pub skipped: Vec<SkippedLocation>,
    pub notes: Vec<String>,
    pub checked_locations: usize,
    pub path_truncated: bool,
}

pub fn discover() -> DiscoveryReport {
    find(platform::mpv_search_locations())
}

fn find(search: platform::MpvSearchLocations) -> DiscoveryReport {
    let mut report = DiscoveryReport {
        checked_locations: search.locations.len(),
        notes: search.notes,
        path_truncated: search.path_truncated,
        ..DiscoveryReport::default()
    };
    for (path, source) in search.locations {
        // This is discovery, not inspection: never execute a file found on PATH.
        match platform::validate_local_path(&path) {
            Ok(()) => {}
            Err(error) => {
                report.skipped.push(SkippedLocation {
                    source,
                    code: error.code,
                    message: error.message,
                });
                continue;
            }
        }
        match std::fs::metadata(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Ok(metadata) if metadata.is_file() => {}
            _ => {
                report.skipped.push(SkippedLocation {
                    source,
                    code: "candidate_unavailable",
                    message: "候选不是普通文件或无法读取，已跳过。",
                });
                continue;
            }
        }
        if let Some(existing) = report.candidates.iter_mut().find(|candidate| {
            candidate
                .executable
                .as_os_str()
                .eq_ignore_ascii_case(path.as_os_str())
        }) {
            if !existing.locations.contains(&source) {
                existing.locations.push(source);
            }
        } else {
            report.candidates.push(MpvCandidate {
                executable: path,
                locations: vec![source],
            });
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    struct Sandbox(PathBuf);
    impl Sandbox {
        fn new() -> Self {
            let id = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path =
                std::env::temp_dir().join(format!("mpvkit-discovery-{}-{id}", std::process::id()));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn path(&self, relative: &str) -> PathBuf {
            self.0.join(relative)
        }
        fn candidate(&self) -> PathBuf {
            let path = self.path("中文 空格/mpv.exe");
            fs::create_dir(path.parent().unwrap()).unwrap();
            // Intentionally not executable: discovery must still return a candidate.
            fs::write(&path, "not an executable").unwrap();
            path
        }
    }
    impl Drop for Sandbox {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn search(locations: Vec<(PathBuf, String)>) -> platform::MpvSearchLocations {
        platform::MpvSearchLocations {
            locations,
            notes: vec![],
            path_truncated: false,
        }
    }

    #[test]
    fn discovery_is_read_only_does_not_execute_or_scan_children() {
        let sandbox = Sandbox::new();
        let candidate = sandbox.candidate();
        let report = find(search(vec![
            (candidate.clone(), "已知位置".into()),
            (sandbox.path("mpv.exe"), "不会递归查找子目录".into()),
        ]));
        assert_eq!(report.candidates.len(), 1);
        assert_eq!(report.candidates[0].executable, candidate);
        assert_eq!(report.checked_locations, 2);
        assert!(report.skipped.is_empty());
        assert_eq!(
            fs::read_to_string(&report.candidates[0].executable).unwrap(),
            "not an executable"
        );
        assert_eq!(fs::read_dir(&sandbox.0).unwrap().count(), 1);
    }

    #[test]
    fn duplicate_ascii_case_paths_merge_location_labels() {
        let sandbox = Sandbox::new();
        let candidate = sandbox.candidate();
        let upper = PathBuf::from(candidate.to_str().unwrap().to_ascii_uppercase());
        let report = find(search(vec![
            (candidate.clone(), "已知位置".into()),
            (upper, "PATH 第 1 项".into()),
            (candidate, "已知位置".into()),
        ]));
        assert_eq!(report.candidates.len(), 1);
        assert_eq!(report.candidates[0].locations, ["已知位置", "PATH 第 1 项"]);
    }

    #[test]
    fn unsafe_locations_are_skipped_and_later_candidates_are_returned() {
        let sandbox = Sandbox::new();
        let candidate = sandbox.candidate();
        let report = find(search(vec![
            (PathBuf::from(r"\\server\RDM-X\mpv.exe"), "远程 PATH".into()),
            (PathBuf::from(r"relative\mpv.exe"), "相对 PATH".into()),
            (candidate, "后续本地位置".into()),
        ]));
        assert_eq!(report.candidates.len(), 1);
        assert_eq!(report.skipped.len(), 2);
        assert!(
            report
                .skipped
                .iter()
                .all(|entry| entry.code == "local_path_required")
        );
    }

    #[test]
    fn missing_files_are_normal_and_directory_candidates_are_reported() {
        let sandbox = Sandbox::new();
        let directory = sandbox.path("mpv.exe");
        fs::create_dir(&directory).unwrap();
        let report = find(search(vec![
            (directory, "错误候选".into()),
            (sandbox.path("missing/mpv.exe"), "未安装".into()),
        ]));
        assert!(report.candidates.is_empty());
        assert_eq!(report.skipped.len(), 1);
        assert_eq!(report.skipped[0].code, "candidate_unavailable");
    }
}
