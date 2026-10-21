use super::*;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Receipt {
    schema_version: u32,
    owner: String,
    id: String,
    files: BTreeMap<String, String>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub id: String,
    pub root: PathBuf,
    pub files: Vec<String>,
    pub bytes: u64,
    pub skipped: usize,
}
pub struct Plan {
    pub preview: Preview,
    created: Instant,
    state: Vec<(PathBuf, Option<String>)>,
    files: Vec<(PathBuf, String)>,
}

pub(super) fn record(root: &Path, journal: &Journal) -> Result<()> {
    validate_journal(journal)?;
    let mut files = BTreeMap::new();
    for (index, change) in journal.changes.iter().enumerate() {
        if let (Some(path), Some(hash)) = (&change.backup, &change.before) {
            files.insert(path.clone(), hash.clone());
        }
        if let Some(hash) = &change.after {
            files.insert(format!("staging/{}/{index}", journal.id), hash.clone());
        }
    }
    if let Some(hash) = &journal.previous_before {
        files.insert(
            format!("backups/{}/previous.json", journal.id),
            hash.clone(),
        );
    }
    let receipt = Receipt {
        schema_version: 1,
        owner: OWNER.into(),
        id: journal.id.clone(),
        files,
    };
    let bytes = serde_json::to_vec_pretty(&receipt)
        .map_err(|_| error("state_write", "无法生成暂存与备份归属。"))?;
    write_new(
        &control(root, &format!("receipts/{}.json", journal.id))?,
        &bytes,
    )
}
fn references(manifest: &InstallManifest, protected: &mut BTreeSet<String>) {
    protected.extend(
        manifest
            .files
            .iter()
            .filter_map(|f| f.original_backup.clone()),
    );
}
fn ready(root: &Path) -> Result<()> {
    if status(root)?.recovery_required
        || file_hash(&control(root, "association-pending.json")?)?.is_some()
    {
        return Err(error(
            "recovery_required",
            "请先恢复未完成操作，再清理暂存与旧备份。",
        ));
    }
    Ok(())
}
fn owned_path(receipt: &Receipt, name: &str, hash: &str) -> bool {
    if !valid_hash(hash) || validate_relative(name).is_err() {
        return false;
    }
    let parts: Vec<_> = name.split('/').collect();
    parts.len() == 3
        && ["backups", "staging"].contains(&parts[0])
        && parts[1] == receipt.id
        && (parts[2].parse::<usize>().is_ok()
            || (parts[0] == "backups" && parts[2] == "previous.json"))
}
pub fn prepare(root: &Path) -> Result<Plan> {
    ready(root)?;
    let mut protected = BTreeSet::new();
    let mut latest = None;
    if let Some(manifest) = status(root)?.manifest {
        references(&manifest, &mut protected);
    }
    let previous = read_optional(&control(root, "previous.json")?)?;
    if let Some(text) = previous {
        let journal: Journal = decode(&text)?;
        validate_journal(&journal)?;
        latest = Some(journal.id);
        for text in [journal.old_manifest, journal.new_manifest]
            .into_iter()
            .flatten()
        {
            references(&decode(&text)?, &mut protected);
        }
    }
    let mut state = Vec::new();
    for name in [
        "install.json",
        "previous.json",
        "pending.json",
        "association-pending.json",
    ] {
        let path = control(root, name)?;
        state.push((path.clone(), file_hash(&path)?));
    }
    let directory = control(root, "receipts")?;
    let entries = match fs::read_dir(directory) {
        Ok(entries) => Some(entries),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(_) => return Err(error("cleanup_state", "归属目录不可读取。")),
    };
    let mut files = Vec::new();
    let mut receipts = Vec::new();
    let mut skipped = 0;
    let mut total = 0;
    if let Some(entries) = entries {
        for (index, entry) in entries.enumerate() {
            if index >= 8192 {
                return Err(error(
                    "cleanup_limit",
                    "归属记录超过8192项，已停止预览；请保留并检查。",
                ));
            }
            let path = entry
                .map_err(|_| error("cleanup_state", "归属目录项无法读取。"))?
                .path();
            platform::validate_local_path(&path)?;
            let Some(text) = read_optional(&path)? else {
                continue;
            };
            let receipt: Receipt = match decode(&text) {
                Ok(receipt) => receipt,
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            };
            if receipt.id.is_empty()
                || !receipt
                    .id
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() || b == b'-')
                || receipt.files.len() > MAX_FILES * 2 + 1
                || path.file_name().and_then(|s| s.to_str())
                    != Some(&format!("{}.json", receipt.id))
                || receipt
                    .files
                    .iter()
                    .any(|(p, h)| !owned_path(&receipt, p, h))
            {
                skipped += 1;
                continue;
            }
            let mut can_remove = true;
            for (relative, hash) in &receipt.files {
                if protected.contains(relative)
                    || (latest.as_ref() == Some(&receipt.id) && relative.starts_with("backups/"))
                {
                    can_remove = false;
                    continue;
                }
                let path = control(root, relative)?;
                match file_hash(&path)? {
                    None => {}
                    Some(current) if &current == hash => {
                        total += fs::metadata(&path)
                            .map_err(|_| error("cleanup_state", "清理目标状态不可读取。"))?
                            .len();
                        files.push((path, hash.clone()));
                    }
                    Some(_) => {
                        can_remove = false;
                        skipped += 1;
                    }
                }
            }
            let receipt_hash = hash_bytes(text.as_bytes());
            state.push((path.clone(), Some(receipt_hash.clone())));
            if can_remove {
                receipts.push((path, receipt_hash));
            }
        }
    }
    // Receipts are last: interruption never discards remaining file ownership.
    files.extend(receipts);
    let preview = Preview {
        id: operation_id(),
        root: root.into(),
        files: files
            .iter()
            .map(|(p, _)| {
                p.strip_prefix(root)
                    .expect("owned path")
                    .to_string_lossy()
                    .into_owned()
            })
            .collect(),
        bytes: total,
        skipped,
    };
    Ok(Plan {
        preview,
        created: Instant::now(),
        state,
        files,
    })
}
pub fn execute(plan: Plan) -> Result<usize> {
    let root = &plan.preview.root;
    let _lock = acquire(root)?;
    ready(root)?;
    if plan.created.elapsed() > Duration::from_secs(300) {
        return Err(error("stale_preview", "清理预览已过期。"));
    }
    for (path, hash) in &plan.state {
        if &file_hash(path)? != hash {
            return Err(error("stale_preview", "恢复引用或归属已改变，请重新预览。"));
        }
    }
    for (path, hash) in &plan.files {
        if file_hash(path)?.as_ref() != Some(hash) {
            return Err(error("stale_preview", "清理目标已改变，未继续删除。"));
        }
    }
    let mut removed = 0;
    for (path, hash) in plan.files {
        if file_hash(&path)?.as_ref() != Some(&hash) {
            return Err(error(
                "cleanup_changed",
                "清理时文件改变，已停止；之前的文件可能已清理。",
            ));
        }
        fs::remove_file(path).map_err(|_| {
            error(
                "cleanup_failed",
                "文件被占用或不可清理，已停止；归属仍可用于再次预览。",
            )
        })?;
        removed += 1;
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cleanup_preserves_original_previous_modified_unknown_and_pending_assets() {
        let root = std::env::temp_dir().join(format!("mpvkit-cleanup-{}", operation_id()));
        write_new(&root.join("config/mpv.conf"), b"original").unwrap();
        for value in ["first", "second", "third"] {
            execute_deploy(&root, value);
        }
        let original = status(&root).unwrap().manifest.unwrap().files[0]
            .original_backup
            .clone()
            .unwrap();
        write_new(
            &control(&root, "staging/unknown/user.txt").unwrap(),
            b"keep",
        )
        .unwrap();
        let plan = prepare(&root).unwrap();
        assert!(!plan.preview.files.is_empty());
        assert!(
            !plan
                .files
                .iter()
                .any(|(p, _)| p == &control(&root, &original).unwrap())
        );
        let changed = plan.files[0].0.clone();
        fs::write(&changed, b"modified").unwrap();
        assert_eq!(execute(plan).unwrap_err().code, "stale_preview");
        let plan = prepare(&root).unwrap();
        assert!(plan.preview.skipped > 0);
        execute(plan).unwrap();
        assert_eq!(read_bytes(&changed).unwrap(), b"modified");
        assert_eq!(
            read_bytes(&control(&root, "staging/unknown/user.txt").unwrap()).unwrap(),
            b"keep"
        );
        assert_eq!(read_backup(&root, &original).unwrap(), b"original");
        super::super::execute(prepare_restore(&root).unwrap()).unwrap();
        assert_eq!(
            read_bytes(&root.join("config/mpv.conf")).unwrap(),
            b"second"
        );
        super::super::execute(prepare_uninstall(&root).unwrap()).unwrap();
        assert_eq!(
            read_bytes(&root.join("config/mpv.conf")).unwrap(),
            b"original"
        );
        write_new(&control(&root, "pending.json").unwrap(), b"pending").unwrap();
        assert!(prepare(&root).is_err());
        fs::remove_dir_all(root).unwrap();
    }
    fn execute_deploy(root: &Path, value: &str) {
        super::super::execute(
            prepare_install(
                root,
                vec![Payload {
                    path: "config/mpv.conf".into(),
                    bytes: value.as_bytes().into(),
                }],
                true,
                None,
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap();
    }
}
