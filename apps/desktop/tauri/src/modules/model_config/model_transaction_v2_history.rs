//! Read-only access to this version's checksummed, root-bound exact images.
//! No recovery, legacy fallback, path discovery or image writes.
use super::*;
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntryV2 {
    pub id: String,
    pub created_at: Option<String>,
    pub phase: String,
    pub has_official_config: bool,
    pub image_bytes: usize,
}
/// Secret-bearing exact images stay backend-only, never Debug/Serialize.
pub struct HistoryImagesV2 {
    pub entry: HistoryEntryV2,
    pub source_digest: String,
    pub images: Vec<WriteV2>,
}
fn summary(j: &Journal) -> Result<HistoryEntryV2> {
    let image_bytes = j
        .plan
        .entries
        .iter()
        .flat_map(|e| [&e.before, &e.after])
        .flatten()
        .try_fold(0usize, |n, i| {
            n.checked_add(i.bytes)
                .filter(|v| *v <= MAX_TOTAL)
                .ok_or_else(|| error("model_limit"))
        })?;
    Ok(HistoryEntryV2 {
        id: j.plan.id.clone(),
        created_at: None,
        phase: match j.phase {
            Phase::Applying => "applying",
            Phase::RollingBack => "rolling_back",
            Phase::Committed => "committed",
            Phase::RolledBack => "rolled_back",
        }
        .into(),
        has_official_config: j
            .plan
            .entries
            .iter()
            .any(|e| e.target == TargetV2::OfficialConfig),
        image_bytes,
    })
}
fn journal(c: &Context, transactions: &Dir, id: &str) -> Result<(Dir, Journal, String)> {
    if !uuid4(id) {
        return Err(fault("model_transaction_path_invalid"));
    }
    c.live()?;
    transactions.live()?;
    let dir = transactions
        .child(id, false, true)?
        .ok_or_else(|| fault("model_history_missing"))?;
    let bytes = dir
        .read(JOURNAL, MAX_JOURNAL, true)?
        .ok_or_else(|| fault("model_transaction_journal_invalid"))?;
    let j: Journal = decode(&bytes)?;
    let binding = c.binding()?;
    let marker = Marker {
        version: 2,
        id: id.into(),
        binding: Binding {
            root: binding.root.clone(),
            home: binding.home.clone(),
        },
        plan_digest: checksum(&j.plan)?,
    };
    validate_journal(&j, &marker, &binding)?;
    let source_digest = schema_v2::digest(&bytes);
    Ok((dir, j, source_digest))
}
pub fn list(root: &Path, home: &Path) -> Result<Vec<HistoryEntryV2>> {
    let c = Context::open(root, home)?;
    let Some(manager) = &c.manager else {
        return Ok(vec![]);
    };
    let Some(transactions) = manager.child(TRANSACTIONS, false, true)? else {
        return Ok(vec![]);
    };
    let mut names = vec![];
    for entry in io(std::fs::read_dir(&transactions.path))? {
        if names.len() >= MAX_RETAINED_TRANSACTIONS {
            return Err(error("model_limit"));
        }
        let entry = io(entry)?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| fault("model_transaction_path_invalid"))?;
        let m = io(std::fs::symlink_metadata(entry.path()))?;
        supported(&m)?;
        if !uuid4(&name) || !m.is_dir() {
            return Err(fault("model_transaction_path_invalid"));
        }
        names.push(name);
    }
    names.sort_unstable();
    let mut total = 0usize;
    let mut result = vec![];
    for id in names {
        let (_dir, j, _) = journal(&c, &transactions, &id)?;
        let entry = summary(&j)?;
        total = total
            .checked_add(entry.image_bytes)
            .filter(|v| *v <= MAX_TOTAL)
            .ok_or_else(|| error("model_limit"))?;
        result.push(entry);
    }
    c.live()?;
    transactions.live()?;
    Ok(result)
}
pub fn read(root: &Path, home: &Path, id: &str) -> Result<HistoryImagesV2> {
    let c = Context::open(root, home)?;
    let transactions = c
        .manager()?
        .child(TRANSACTIONS, false, true)?
        .ok_or_else(|| fault("model_history_missing"))?;
    let (dir, j, source_digest) = journal(&c, &transactions, id)?;
    let entry = summary(&j)?;
    let loaded = load_images(&dir, &j)?;
    let images = loaded
        .into_iter()
        .map(|mut row| WriteV2 {
            target: row.target.clone(),
            before: row.before.take().map(|v| v.to_vec()),
            after: row.after.take().map(|v| v.to_vec()),
        })
        .collect();
    // A concurrent journal/image rewrite cannot become a silently mixed source.
    let (_, _, current) = journal(&c, &transactions, id)?;
    if current != source_digest {
        return Err(fault("model_history_source_stale"));
    }
    c.live()?;
    dir.live()?;
    transactions.live()?;
    Ok(HistoryImagesV2 {
        entry,
        source_digest,
        images,
    })
}
