//! Original v2 LocalApplication scope backups and explicit guarded undo.
//! UI supplies only listed groups and opaque IDs. Images never cross IPC.
use super::*;
use crate::modules::model_config::{
    content_versions::{self as versions, SaveState},
    model_transaction_v2::{self as transactions, TargetV2, WriteV2},
    schema::{ModelError, ModelResult},
    schema_v2, source_v2, store_v2,
    transfer_backup::{self as images, Image},
};
use std::collections::BTreeSet;
use zeroize::Zeroize;

const DIRECTORY: &str = "backups/transfer-scope-v2";
const PLANS: &str = "manager-state/transfer-scope-v2-restores";
const LIMIT: u64 = 128 * 1024 * 1024;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupStatusV2 {
    pub id: String,
    pub enabled: bool,
    pub ready: bool,
    pub groups: Vec<String>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePreviewV2 {
    pub restore_id: String,
    pub plan_digest: String,
    pub groups: Vec<String>,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RestoreStatusV2 {
    Applied,
    Partial,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RestoreReceiptV2 {
    pub restore_id: String,
    pub original_operation_id: String,
    pub groups: Vec<String>,
    pub status: RestoreStatusV2,
    pub completed_groups: Vec<String>,
    pub refresh_file_snapshot: bool,
    #[serde(default)]
    pub refresh_sync_scope: bool,
    pub failure_code: Option<String>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreOperationV2 {
    pub restore_id: String,
    pub original_operation_id: String,
    pub plan_digest: String,
    pub groups: Vec<String>,
    pub receipt: Option<RestoreReceiptV2>,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Address {
    group: String,
    anchor: PathBuf,
    path: PathBuf,
    model: Option<TargetV2>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    address: Address,
    before: Image,
    expected_after: Option<Image>,
    before_tree: Option<BTreeMap<PathBuf, String>>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    version: u32,
    id: String,
    root: PathBuf,
    home: PathBuf,
    enabled: bool,
    groups: Vec<String>,
    selection: SelectionV2,
    admission: String,
    model_supported: bool,
    rows: Vec<Row>,
    after: Option<Vec<Image>>,
    after_guards: Option<Vec<Option<String>>>,
}
impl Drop for Record {
    fn drop(&mut self) {
        for row in &mut self.rows {
            erase_image(&mut row.before);
            if let Some(image) = &mut row.expected_after {
                erase_image(image);
            }
        }
        if let Some(after) = &mut self.after {
            for image in after {
                erase_image(image);
            }
        }
    }
}
fn erase_image(image: &mut Image) {
    for bytes in image.files.values_mut() {
        bytes.zeroize();
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Plan {
    version: u32,
    id: String,
    original: String,
    root: PathBuf,
    home: PathBuf,
    source_digest: String,
    groups: Vec<String>,
    guards: Vec<String>,
    #[serde(default)]
    scope_edit: Option<super::super::sync_service_v2::scope_restore::ScopeEditV2>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Progress {
    plan_digest: String,
    protected: bool,
    backups: Vec<BackupRef>,
    completed: BTreeSet<usize>,
    receipt: Option<RestoreReceiptV2>,
    #[serde(default)]
    clock_time: Option<String>,
    #[serde(default)]
    deltas: BTreeMap<String, StateDelta>,
    #[serde(default)]
    clocks: Vec<ClockStep>,
    #[serde(default)]
    versioned: BTreeSet<String>,
    #[serde(default)]
    scope_updated: bool,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StateDelta {
    before: SaveState,
    after: SaveState,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ClockStep {
    group: String,
    ledger_before: Option<Vec<u8>>,
    ledger_after: Option<Vec<u8>>,
    applied: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope<T> {
    value: T,
    checksum: String,
}
fn bad(code: &str) -> ModelError {
    invalid(code, "原范围备份未通过核对，未覆盖当前内容")
}
fn local_error(_: SyncFailure) -> ModelError {
    bad("scope_backup_local_journal")
}
fn encode<T: Serialize>(value: &T) -> ModelResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(Into::into)
}
fn digest<T: Serialize>(value: &T) -> ModelResult<String> {
    Ok(hash(&encode(value)?))
}
fn roots(d: &Path, h: &Path) -> ModelResult<(PathBuf, PathBuf)> {
    for root in [d, h] {
        if !root.is_absolute() || !safety::inspect(root)?.is_dir() {
            return Err(bad("scope_backup_root"));
        }
    }
    Ok((d.canonicalize()?, h.canonicalize()?))
}
fn id(id: &str) -> ModelResult<()> {
    canonical_id(id).map_err(|_| bad("scope_backup_id"))
}
fn record_path(d: &Path, original: &str) -> ModelResult<PathBuf> {
    id(original)?;
    let path = d.join(DIRECTORY).join(original).join("record.json");
    safety::check_path(d, &path, true)?;
    Ok(path)
}
fn plan_path(d: &Path, restore: &str, suffix: &str) -> ModelResult<PathBuf> {
    id(restore)?;
    let path = d.join(PLANS).join(format!("{restore}.{suffix}.json"));
    safety::check_path(d, &path, true)?;
    Ok(path)
}
fn save<T: Serialize>(d: &Path, path: &Path, value: &T) -> ModelResult<()> {
    safety::check_path(d, path, true)?;
    let bytes = Zeroizing::new(encode(&Envelope {
        value,
        checksum: digest(value)?,
    })?);
    if bytes.len() as u64 > LIMIT {
        return Err(bad("scope_backup_limit"));
    }
    atomic_write(path, &bytes, 0o600)?;
    if safety::read_file(path, LIMIT)?.as_slice() != bytes.as_slice() {
        return Err(bad("scope_backup_readback"));
    }
    safety::sync_directories(d, path.parent().ok_or_else(|| bad("scope_backup_path"))?)?;
    Ok(())
}
fn read<T: serde::de::DeserializeOwned + Serialize>(path: &Path) -> ModelResult<T> {
    private(path).map_err(local_error)?;
    let bytes = Zeroizing::new(safety::read_file(path, LIMIT)?);
    let envelope: Envelope<T> = serde_json::from_value(
        schema_v2::parse_strict_json_v2(&bytes).map_err(|_| bad("scope_backup_corrupt"))?,
    )?;
    if envelope.checksum != digest(&envelope.value)? {
        return Err(bad("scope_backup_corrupt"));
    }
    Ok(envelope.value)
}
fn file_image(bytes: Option<&[u8]>) -> Image {
    Image {
        exists: bytes.is_some(),
        directory: false,
        files: bytes
            .map(|b| BTreeMap::from([(String::new(), b.to_vec())]))
            .unwrap_or_default(),
        executable: BTreeSet::new(),
    }
}
fn bytes(image: &Image) -> ModelResult<Option<Vec<u8>>> {
    if !image.exists && image.files.is_empty() && !image.directory {
        return Ok(None);
    }
    if image.directory || image.files.len() != 1 || !image.files.contains_key("") {
        return Err(bad("scope_backup_model_image"));
    }
    Ok(image.files.get("").cloned())
}
fn model_path(d: &Path, h: &Path, target: &TargetV2) -> ModelResult<(PathBuf, PathBuf)> {
    match target {
        TargetV2::OfficialConfig => Ok((h.into(), h.join("config.json"))),
        TargetV2::Manager(relative)
            if [
                store_v2::LIBRARY_PATH,
                store_v2::POLICY_PATH,
                store_v2::LEDGER_PATH,
                store_v2::MAPPING_PATH,
                "manager-state/saved-content-versions.json",
                "manager-state/sync-identity-mappings-v2.json",
            ]
            .contains(&relative.as_str()) =>
        {
            Ok((d.into(), d.join(relative)))
        }
        _ => Err(bad("scope_backup_model_target")),
    }
}
fn model_group(selection: &SelectionV2, target: &TargetV2) -> &'static str {
    if (matches!(target, TargetV2::Manager(p) if p == store_v2::LIBRARY_PATH)
        && !selection.template_refs.is_empty())
        || selection.channels.is_empty()
    {
        "templates"
    } else {
        "channels"
    }
}
fn participant_addresses(
    d: &Path,
    h: &Path,
    original: &str,
    s: &SelectionV2,
) -> ModelResult<Vec<Address>> {
    let mut addresses = vec![];
    for (selected, group, relative) in [
        (
            s.manager.preferences,
            "preferences",
            crate::modules::preferences::PREFERENCES_RELATIVE_PATH,
        ),
        (
            s.manager.extensions,
            "extensions",
            crate::modules::extensions::projection::CONFIG_RELATIVE_PATH,
        ),
        (
            !s.manager.webdav_ids.is_empty(),
            "webdav",
            "manager-state/sync-imported-endpoints.json",
        ),
        (
            !s.manager.account_ids.is_empty(),
            "accounts",
            crate::modules::sync::portable_accounts::IMPORTED,
        ),
    ] {
        if selected {
            addresses.push(Address {
                group: group.into(),
                anchor: d.into(),
                path: d.join(relative),
                model: None,
            });
        }
    }
    for (group, anchor, path) in portable_assets::backup_targets(d, original)? {
        if group == "mcp" && s.mcp_sources.is_empty() {
            continue;
        }
        if group
            .strip_prefix("skill:")
            .is_some_and(|name| !s.skill_names.iter().any(|n| n == name))
        {
            return Err(bad("scope_backup_scope"));
        }
        let anchor = if path.starts_with(d) {
            d.into()
        } else if path.starts_with(h) {
            h.into()
        } else {
            anchor
        };
        addresses.push(Address {
            group,
            anchor,
            path,
            model: None,
        });
    }
    addresses.sort_by(|a, b| (&a.group, &a.path).cmp(&(&b.group, &b.path)));
    Ok(addresses)
}
fn groups(s: &SelectionV2) -> Vec<String> {
    let mut g = BTreeSet::new();
    if !s.channels.is_empty() {
        g.insert("channels".into());
    }
    if !s.template_refs.is_empty() {
        g.insert("templates".into());
    }
    for (yes, name) in [
        (s.manager.preferences, "preferences"),
        (s.manager.extensions, "extensions"),
        (!s.manager.webdav_ids.is_empty(), "webdav"),
        (!s.manager.account_ids.is_empty(), "accounts"),
        (!s.mcp_sources.is_empty(), "mcp"),
    ] {
        if yes {
            g.insert(name.into());
        }
    }
    g.extend(s.skill_names.iter().map(|name| format!("skill:{name}")));
    g.into_iter().collect()
}
fn validate_record(d: &Path, h: &Path, r: &Record) -> ModelResult<Journal> {
    validate_record_at(d, h, r, true)
}
fn validate_record_at(
    d: &Path,
    h: &Path,
    r: &Record,
    current_targets: bool,
) -> ModelResult<Journal> {
    if r.version != 2 || r.root != d || r.home != h || r.groups != groups(&r.selection) {
        return Err(bad("scope_backup_binding"));
    }
    let j = read_local_journal(d, h, &r.id)
        .map_err(local_error)?
        .ok_or_else(|| bad("scope_backup_local_journal"))?;
    if j.export
        || j.backup != r.enabled
        || hash(&journal_admission(&j).map_err(local_error)?) != r.admission
        || r.after
            .as_ref()
            .is_some_and(|after| after.len() != r.rows.len())
    {
        return Err(bad("scope_backup_binding"));
    }
    if !r.enabled {
        if !r.rows.is_empty() || r.after.is_some() {
            return Err(bad("scope_backup_disabled"));
        }
        return Ok(j);
    }
    if current_targets {
        let addresses = participant_addresses(d, h, &r.id, &r.selection)?;
        let participant: Vec<_> = r
            .rows
            .iter()
            .filter(|row| row.address.model.is_none())
            .map(|row| row.address.clone())
            .collect();
        if participant != addresses {
            return Err(bad("scope_backup_target_changed"));
        }
    }
    let mut seen = BTreeSet::new();
    for row in &r.rows {
        if !r.groups.contains(&row.address.group) || !seen.insert(&row.address.path) {
            return Err(bad("scope_backup_scope"));
        }
        if let Some(target) = &row.address.model {
            let (anchor, path) = model_path(d, h, target)?;
            if anchor != row.address.anchor
                || path != row.address.path
                || row.address.group != model_group(&r.selection, target)
            {
                return Err(bad("scope_backup_model_target"));
            }
        }
        if current_targets {
            safety::check_path(&row.address.anchor, &row.address.path, true)?;
        }
        if row.address.model.is_none() {
            verify_before_tree(row, &j)?;
        }
    }
    if r.after.is_some() && r.model_supported {
        verify_original_model_images(d, h, r, &j)?;
    }
    Ok(j)
}
fn verify_before_tree(row: &Row, j: &Journal) -> ModelResult<()> {
    let t = row
        .before_tree
        .as_ref()
        .ok_or_else(|| bad("scope_backup_before_proof"))?;
    let own = j
        .targets
        .as_ref()
        .and_then(|targets| {
            targets.iter().find(|target| {
                target.anchor == row.address.anchor && target.path == row.address.path
            })
        })
        .ok_or_else(|| bad("scope_backup_before_proof"))?;
    if digest(t)? != own.image {
        return Err(bad("scope_backup_before_proof"));
    }
    let root = t
        .get(&row.address.path)
        .ok_or_else(|| bad("scope_backup_before_proof"))?;
    if row.before.exists != (root != "absent") || row.before.directory != (root == "directory") {
        return Err(bad("scope_backup_before_proof"));
    }
    let mut files = BTreeSet::new();
    for (path, value) in t {
        if value == "absent" || value == "directory" {
            continue;
        }
        let name = path
            .strip_prefix(&row.address.path)
            .map_err(|_| bad("scope_backup_before_proof"))?
            .to_str()
            .ok_or_else(|| bad("scope_backup_before_proof"))?
            .replace('\\', "/");
        let (expected, mode) = value
            .rsplit_once(':')
            .ok_or_else(|| bad("scope_backup_before_proof"))?;
        let mode: u32 = mode.parse().map_err(|_| bad("scope_backup_before_proof"))?;
        if row
            .before
            .files
            .get(&name)
            .is_none_or(|bytes| hash(bytes) != expected)
            || row.before.executable.contains(&name) != (mode & 0o111 != 0)
        {
            return Err(bad("scope_backup_before_proof"));
        }
        files.insert(name);
    }
    if files != row.before.files.keys().cloned().collect() {
        return Err(bad("scope_backup_before_proof"));
    }
    Ok(())
}
fn verify_original_model_images(d: &Path, h: &Path, r: &Record, j: &Journal) -> ModelResult<()> {
    let handle = j
        .model
        .as_ref()
        .ok_or_else(|| bad("scope_backup_model_plan"))?;
    let receipt = d
        .join("manager-state/model-receive-v2")
        .join(&r.id)
        .join("receipt.json");
    safety::check_path(d, &receipt, false)?;
    private(&receipt).map_err(local_error)?;
    let receipt_bytes = Zeroizing::new(safety::read_file(&receipt, 32 * 1024)?);
    let value: serde_json::Value = serde_json::from_slice(&receipt_bytes)?;
    if value["value"]["operationId"] != r.id || value["value"]["planHash"] != handle.token() {
        return Err(bad("scope_backup_model_receipt"));
    }
    // The typed engine's checked, root-bound transaction images independently
    // witness the exact O receipt. Never pick a latest transaction or parse v1.
    let mut found = false;
    for entry in transactions::history::list(d, h).map_err(|_| bad("scope_backup_model_history"))? {
        if entry.phase != "committed" {
            continue;
        }
        let history = transactions::history::read(d, h, &entry.id)
            .map_err(|_| bad("scope_backup_model_history"))?;
        if !history.images.iter().any(|w| {
            w.target == TargetV2::ManagerOperationReceipt(r.id.clone())
                && w.after.as_deref() == Some(receipt_bytes.as_slice())
        }) {
            continue;
        }
        if found {
            return Err(bad("scope_backup_model_history"));
        }
        found = true;
        let model_rows: Vec<_> = r
            .rows
            .iter()
            .filter(|row| row.address.model.is_some())
            .collect();
        if (!r.selection.channels.is_empty() || !r.selection.template_refs.is_empty())
            && model_rows.len() + 1 != history.images.len()
        {
            return Err(bad("scope_backup_model_history"));
        }
        for row in model_rows {
            let write = history
                .images
                .iter()
                .find(|w| Some(&w.target) == row.address.model.as_ref())
                .ok_or_else(|| bad("scope_backup_model_history"))?;
            let index = r
                .rows
                .iter()
                .position(|candidate| std::ptr::eq(candidate, row))
                .ok_or_else(|| bad("scope_backup_model_history"))?;
            if file_image(write.before.as_deref()) != row.before
                || Some(file_image(write.after.as_deref())) != row.expected_after
                || r.after.as_ref().and_then(|after| after.get(index))
                    != row.expected_after.as_ref()
            {
                return Err(bad("scope_backup_model_history"));
            }
        }
    }
    if !found {
        return Err(bad("scope_backup_model_history"));
    }
    Ok(())
}
fn record(d: &Path, h: &Path, original: &str) -> ModelResult<Record> {
    let r: Record = read(&record_path(d, original)?)?;
    if r.id != original {
        return Err(bad("scope_backup_binding"));
    }
    validate_record(d, h, &r)?;
    Ok(r)
}
fn hook(result: ModelResult<()>) -> SyncResult<()> {
    result.map_err(|_| failure(ErrorCode::SyncBackupFailed))
}
pub(super) fn freeze_before(host: &LocalApplicationV2, j: &Journal) -> SyncResult<()> {
    hook((|| {
        let path = record_path(&host.d, &j.id)?;
        if path.exists() {
            let _ = record(&host.d, &host.h, &j.id)?;
            return Ok(());
        }
        if j.progress > LocalProgressV2::Protected {
            return Ok(());
        }
        if j.progress != LocalProgressV2::Protected || !j.protected {
            return Err(bad("scope_backup_before_missing"));
        }
        let mut r = Record {
            version: 2,
            id: j.id.clone(),
            root: host.d.clone(),
            home: host.h.clone(),
            enabled: j.backup,
            groups: groups(&host.selection),
            selection: host.selection.clone(),
            admission: hash(&journal_admission(j).map_err(local_error)?),
            model_supported: false,
            rows: vec![],
            after: None,
            after_guards: None,
        };
        // No protection was requested; retain a disabled entry without private images.
        if !j.backup {
            save(&host.d, &path, &r)?;
            return Ok(());
        }
        host.verify_backups(j).map_err(local_error)?;
        let handle = j
            .model
            .as_ref()
            .ok_or_else(|| bad("scope_backup_model_plan"))?;
        if host.model_applied(&j.id, handle).map_err(local_error)? {
            return Err(bad("scope_backup_before_missing"));
        }
        if let Some(writes) = host.models.scope_backup_images(&j.id, handle)? {
            r.model_supported = true;
            for w in &writes {
                if host.selection.channels.is_empty() && host.selection.template_refs.is_empty() {
                    continue;
                }
                let (anchor, path) = model_path(&host.d, &host.h, &w.target)?;
                let before = file_image(w.before.as_deref());
                // Real typed before images plus the actual protected target, never a synthetic empty JSON.
                if images::image(&anchor, &path)? != before {
                    return Err(bad("scope_backup_source_stale"));
                }
                r.rows.push(Row {
                    address: Address {
                        group: model_group(&host.selection, &w.target).into(),
                        anchor,
                        path,
                        model: Some(w.target.clone()),
                    },
                    before,
                    expected_after: Some(file_image(w.after.as_deref())),
                    before_tree: None,
                });
            }
        }
        for address in participant_addresses(&host.d, &host.h, &j.id, &host.selection)? {
            if !j.targets.as_ref().is_some_and(|targets| {
                targets.iter().any(|t| {
                    t.anchor == address.anchor
                        && t.path == address.path
                        && image(&t.anchor, &t.path).ok().as_ref() == Some(&t.image)
                })
            }) {
                return Err(bad("scope_backup_source_stale"));
            }
            let before = images::image(&address.anchor, &address.path)?;
            let mut before_tree = BTreeMap::new();
            tree(
                &address.anchor,
                &address.path,
                &mut before_tree,
                &mut (64 * 1024 * 1024),
            )
            .map_err(local_error)?;
            r.rows.push(Row {
                address,
                before,
                expected_after: None,
                before_tree: Some(before_tree),
            });
        }
        save(&host.d, &path, &r)
    })())
}
pub(super) fn finish(host: &LocalApplicationV2, j: &Journal) -> SyncResult<()> {
    hook((|| {
        let path = record_path(&host.d, &j.id)?;
        if !path.exists() {
            return Ok(());
        } // Earlier operations never invent a before snapshot.
        let mut r = record(&host.d, &host.h, &j.id)?;
        if !r.enabled || r.after.is_some() || !r.model_supported {
            return Ok(());
        }
        let handle = j
            .model
            .as_ref()
            .ok_or_else(|| bad("scope_backup_model_plan"))?;
        if j.progress != LocalProgressV2::Applied
            || !host.complete(&j.id, handle).map_err(local_error)?
        {
            return Err(bad("scope_backup_incomplete"));
        }
        let writes = host
            .models
            .scope_backup_images(&j.id, handle)?
            .ok_or_else(|| bad("scope_backup_model_plan"))?;
        let manager = portable_manager::applied_files(&host.d, &j.id)?;
        let mut after = vec![];
        for row in &r.rows {
            let a = &row.address;
            let next = if let Some(target) = &a.model {
                let w = writes
                    .iter()
                    .find(|w| &w.target == target)
                    .ok_or_else(|| bad("scope_backup_model_plan"))?;
                if file_image(w.before.as_deref()) != row.before
                    || Some(file_image(w.after.as_deref())) != row.expected_after
                {
                    return Err(bad("scope_backup_model_plan"));
                }
                row.expected_after
                    .clone()
                    .ok_or_else(|| bad("scope_backup_model_plan"))?
            } else if let Some((_, bytes)) = manager
                .iter()
                .find(|(relative, _)| host.d.join(relative) == a.path)
            {
                file_image(Some(bytes))
            } else if a.group == "mcp" {
                file_image(Some(
                    &portable_assets::completed_mcp_bytes(&host.d, &j.id)?
                        .ok_or_else(|| bad("scope_backup_asset_receipt"))?,
                ))
            } else if let Some(name) = a.group.strip_prefix("skill:") {
                if let Some((_, bytes)) = portable_assets::completed_modes(&host.d, &j.id, name)?
                    .filter(|(p, _)| p == &a.path)
                {
                    file_image(Some(&bytes))
                } else if portable_assets::completed_skill_target(&host.d, &j.id, name)?.as_ref()
                    == Some(&a.path)
                {
                    // The actual participant checks its receipted Skill digest before this read.
                    images::image(&a.anchor, &a.path)?
                } else {
                    if images::image(&a.anchor, &a.path)? != row.before {
                        return Err(bad("scope_backup_asset_receipt"));
                    }
                    row.before.clone()
                }
            } else {
                return Err(bad("scope_backup_participant_receipt"));
            };
            after.push(next);
        }
        let mut guards = Vec::new();
        for (row, next) in r.rows.iter().zip(&after) {
            // File after bytes come from the original typed/manager/MCP receipts.
            // Do not consult today's model config during remaining-only recovery.
            // Skills additionally need their receipted live directory tree guard.
            if next.directory {
                if images::image(&row.address.anchor, &row.address.path)? != *next {
                    return Err(bad("scope_backup_after_changed"));
                }
                guards.push(Some(
                    image(&row.address.anchor, &row.address.path).map_err(local_error)?,
                ));
            } else {
                guards.push(None);
            }
        }
        r.after = Some(after);
        r.after_guards = Some(guards);
        save(&host.d, &path, &r)
    })())
}
pub fn list(d: &Path, h: &Path) -> ModelResult<Vec<BackupStatusV2>> {
    let (d, h) = roots(d, h)?;
    let dir = d.join(DIRECTORY);
    safety::check_path(&d, &dir, true)?;
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut result = vec![];
    for entry in std::fs::read_dir(dir)? {
        if result.len() >= 256 {
            return Err(bad("scope_backup_limit"));
        }
        let entry = entry?;
        let original = entry
            .file_name()
            .into_string()
            .map_err(|_| bad("scope_backup_id"))?;
        id(&original)?;
        safety::check_path(&d, &entry.path(), false)?;
        // A failed freeze before record publication is preserved but cannot become ready.
        if !record_path(&d, &original)?.exists() {
            continue;
        }
        let r = record(&d, &h, &original)?;
        let j = validate_record(&d, &h, &r)?;
        result.push(BackupStatusV2 {
            id: original,
            enabled: r.enabled,
            ready: r.enabled
                && r.model_supported
                && r.after.is_some()
                && j.progress == LocalProgressV2::Applied,
            groups: r.groups.clone(),
        });
    }
    result.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(result)
}
fn selected(r: &Record, groups: &[String]) -> ModelResult<Vec<usize>> {
    if groups.is_empty()
        || groups.iter().collect::<BTreeSet<_>>().len() != groups.len()
        || groups.iter().any(|g| !r.groups.contains(g))
    {
        return Err(bad("scope_restore_scope"));
    }
    if !r.enabled || !r.model_supported || r.after.is_none() {
        return Err(bad("scope_restore_not_ready"));
    }
    Ok(r.rows
        .iter()
        .enumerate()
        .filter(|(_, row)| groups.contains(&row.address.group))
        .map(|(i, _)| i)
        .collect())
}
fn model_check(d: &Path, h: &Path, r: &Record, selected: &[usize]) -> ModelResult<()> {
    if selected.iter().any(|i| r.rows[*i].address.model.is_some()) {
        model_state(d, h, r, selected, false)?;
    }
    Ok(())
}
fn model_state(
    d: &Path,
    h: &Path,
    r: &Record,
    selected: &[usize],
    original_after: bool,
) -> ModelResult<SaveState> {
    let intended = |target: &TargetV2| -> ModelResult<Option<Vec<u8>>> {
        if let Some(i) = selected
            .iter()
            .find(|i| r.rows[**i].address.model.as_ref() == Some(target))
        {
            let selected_image = if original_after {
                r.after
                    .as_ref()
                    .and_then(|images| images.get(*i))
                    .ok_or_else(|| bad("scope_restore_not_ready"))?
            } else {
                &r.rows[*i].before
            };
            return bytes(selected_image);
        }
        let (anchor, path) = model_path(d, h, target)?;
        bytes(&images::image(&anchor, &path)?)
    };
    let parse =
        |b: &[u8]| schema_v2::parse_strict_json_v2(b).map_err(|_| bad("scope_restore_dependency"));
    let library = match intended(&TargetV2::Manager(store_v2::LIBRARY_PATH.into()))? {
        Some(bytes) => serde_json::from_value::<schema_v2::LibraryV2>(parse(&bytes)?)?,
        None => schema_v2::seed_library_v2().map_err(|_| bad("scope_restore_dependency"))?,
    };
    let policy = intended(&TargetV2::Manager(store_v2::POLICY_PATH.into()))?
        .as_deref()
        .map(|b| serde_json::from_value::<schema_v2::PolicyV2>(parse(b)?).map_err(ModelError::from))
        .transpose()?
        .unwrap_or_else(store_v2::empty_policy_v2);
    source_v2::plan_materialization_v2(&library, &policy, &policy)
        .map_err(|_| bad("scope_restore_dependency"))?;
    let raw = intended(&TargetV2::OfficialConfig)?
        .as_deref()
        .map(parse)
        .transpose()?
        .unwrap_or_else(|| serde_json::json!({}));
    if !raw.is_object() {
        return Err(bad("scope_restore_dependency"));
    }
    let mut source_ledger = None;
    if let Some(b) = intended(&TargetV2::Manager(store_v2::LEDGER_PATH.into()))? {
        let ledger: store_v2::LedgerFileV2 = serde_json::from_value(parse(&b)?)?;
        if ledger.schema_version != 2
            || ledger.ledger.source_policy_digest
                != source_v2::policy_semantic_digest_v2(&policy)
                    .map_err(|_| bad("scope_restore_dependency"))?
            || ledger.ledger.written_file_digest
                != schema_v2::digest(
                    &schema_v2::canonical_bytes(&raw)
                        .map_err(|_| bad("scope_restore_dependency"))?,
                )
        {
            return Err(bad("scope_restore_dependency"));
        }
        source_ledger = Some(ledger.ledger);
    } else if !policy.channels.is_empty() {
        return Err(bad("scope_restore_dependency"));
    }
    let mappings = if let Some(b) = intended(&TargetV2::Manager(store_v2::MAPPING_PATH.into()))? {
        let mappings = serde_json::from_value(parse(&b)?)?;
        crate::modules::model_config::protocol_groups::validate_mappings_v2(&mappings)
            .map_err(|_| bad("scope_restore_dependency"))?;
        mappings
    } else {
        crate::modules::model_config::protocol_groups::MappingFileV2::default()
    };
    let bindings = policy
        .channels
        .iter()
        .map(
            |channel| crate::modules::model_config::provider_import::ProviderBindingV2 {
                channel_id: channel.id.clone(),
                provider_id: channel.connection.provider_id.clone(),
            },
        )
        .collect::<Vec<_>>();
    let observed = crate::modules::model_config::provider_import::read_policy_v2(
        &raw,
        &schema_v2::digest(
            &schema_v2::canonical_bytes(&raw).map_err(|_| bad("scope_restore_save_state"))?,
        ),
        &policy,
        source_ledger.as_ref(),
        &mappings,
        &bindings,
    )
    .map_err(|_| bad("scope_restore_save_state"))?;
    super::super::config_file::v2::saved_state(&library, &observed, &raw)
        .map_err(|_| bad("scope_restore_save_state"))
}
// Derive only identities that will actually cease to exist in the restored
// frozen images. Model save atoms retain local UUIDs; endpoint mappings bridge
// the exact portable UUID without inferring identity from a display label.
fn restored_absences(
    d: &Path,
    h: &Path,
    r: &Record,
    indices: &[usize],
) -> ModelResult<BTreeSet<String>> {
    let mut removed = BTreeSet::new();
    let models = if indices.iter().any(|i| r.rows[*i].address.model.is_some()) {
        Some(model_state(d, h, r, indices, false)?)
    } else {
        None
    };
    if let Some(before) = &models {
        let mut ids: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (i, row) in r.rows.iter().enumerate() {
            if matches!(&row.address.model, Some(TargetV2::Manager(p)) if p=="manager-state/sync-identity-mappings-v2.json")
            {
                if let Some(raw) = bytes(
                    r.after
                        .as_ref()
                        .and_then(|a| a.get(i))
                        .ok_or_else(|| bad("scope_restore_not_ready"))?,
                )? {
                    let value = schema_v2::parse_strict_json_v2(&raw)
                        .map_err(|_| bad("scope_restore_scope_identity"))?;
                    for binding in value["bindings"]
                        .as_array()
                        .ok_or_else(|| bad("scope_restore_scope_identity"))?
                    {
                        let portable = binding["portableId"]
                            .as_str()
                            .ok_or_else(|| bad("scope_restore_scope_identity"))?;
                        let local = binding["localId"]
                            .as_str()
                            .ok_or_else(|| bad("scope_restore_scope_identity"))?;
                        ids.entry(portable.into()).or_default().insert(local.into());
                    }
                }
            }
        }
        let mut removed_local = BTreeSet::new();
        for i in indices {
            if matches!(&r.rows[*i].address.model, Some(TargetV2::Manager(p)) if p==store_v2::POLICY_PATH)
            {
                if let Some(raw) = bytes(
                    r.after
                        .as_ref()
                        .and_then(|a| a.get(*i))
                        .ok_or_else(|| bad("scope_restore_not_ready"))?,
                )? {
                    let policy: schema_v2::PolicyV2 = serde_json::from_value(
                        schema_v2::parse_strict_json_v2(&raw)
                            .map_err(|_| bad("scope_restore_scope_identity"))?,
                    )?;
                    for channel in policy.channels {
                        if !before.contains_key(&versions::key(&serde_json::json!([
                            "v2-channel",
                            channel.id,
                            null,
                            null
                        ]))?) {
                            removed.insert(format!("channel:{}", channel.id));
                            removed_local.insert(channel.id);
                        }
                    }
                }
            }
        }
        let mut originally_empty = false;
        for i in indices {
            if r.rows[*i].address.model == Some(TargetV2::OfficialConfig) {
                originally_empty = match bytes(&r.rows[*i].before)? {
                    None => true,
                    Some(raw) => schema_v2::parse_strict_json_v2(&raw)
                        .map_err(|_| bad("scope_restore_scope_identity"))?["providers"]
                        .as_object()
                        .is_none_or(|providers| providers.is_empty()),
                };
            }
        }
        for channel in &r.selection.channels {
            let id = &channel.portable_channel_id;
            let mut candidates = ids.get(id).cloned().unwrap_or_default();
            candidates.insert(id.clone());
            let present = candidates.iter().any(|id| {
                versions::key(&serde_json::json!(["v2-channel", id, null, null]))
                    .is_ok_and(|key| before.contains_key(&key))
            });
            if !present
                && (removed_local.contains(id)
                    || ids
                        .get(id)
                        .is_some_and(|locals| locals.iter().any(|id| removed_local.contains(id)))
                    || originally_empty)
            {
                removed.insert(format!("channel:{id}"));
            }
        }
        for reference in &r.selection.template_refs {
            let key = versions::key(&serde_json::json!([
                "v2-template",
                reference.template_id,
                reference.revision_id
            ]))?;
            if !before.contains_key(&key) {
                removed.insert(format!(
                    "template:{}:{}",
                    reference.template_id, reference.revision_id
                ));
            }
        }
    }
    for group in groups(&r.selection) {
        if !indices
            .iter()
            .any(|i| r.rows[*i].address.group == group && r.rows[*i].address.model.is_none())
        {
            continue;
        }
        let before = group_state(d, r, &group, false)?;
        let missing = |identity: serde_json::Value| -> ModelResult<bool> {
            Ok(!before.contains_key(&versions::key(&identity)?))
        };
        match group.as_str() {
            "preferences" | "extensions" => {
                if missing(serde_json::json!([format!("v2-{group}")]))? {
                    removed.insert(group);
                }
            }
            "mcp" => {
                for mcp in &r.selection.mcp_sources {
                    if missing(serde_json::json!(["v2-mcps", mcp.name]))? {
                        removed.insert(format!("mcp:{}", mcp.name));
                    }
                }
            }
            "accounts" => {
                let mut present = BTreeSet::new();
                for row in r.rows.iter().filter(|row| row.address.group == group) {
                    if let Some(raw) = bytes(&row.before)? {
                        for a in serde_json::from_slice::<
                            Vec<crate::modules::sync::portable_accounts::AccountInfo>,
                        >(&raw)?
                        {
                            present.insert(a.id);
                        }
                    }
                }
                for id in &r.selection.manager.account_ids {
                    if !present.contains(id) {
                        removed.insert(format!("account:{id}"));
                    }
                }
            }
            "webdav" => {
                for id in &r.selection.manager.webdav_ids {
                    if missing(serde_json::json!(["v2-webdav", id]))? {
                        removed.insert(format!("webdav:{id}"));
                    }
                }
            }
            _ => {
                if let Some(name) = group.strip_prefix("skill:") {
                    if missing(serde_json::json!(["v2-skills", name]))? {
                        removed.insert(group);
                    }
                }
            }
        }
    }
    Ok(removed)
}
pub fn preview(
    d: &Path,
    h: &Path,
    original: &str,
    groups: &[String],
) -> ModelResult<RestorePreviewV2> {
    let _storage = crate::infrastructure::storage_writers::global().admit()?;
    let (d, h) = roots(d, h)?;
    let r = record(&d, &h, original)?;
    let indices = selected(&r, groups)?;
    if validate_record(&d, &h, &r)?.progress != LocalProgressV2::Applied {
        return Err(bad("scope_restore_not_ready"));
    }
    let after = r
        .after
        .as_ref()
        .ok_or_else(|| bad("scope_restore_not_ready"))?;
    let original_guards = r
        .after_guards
        .as_ref()
        .ok_or_else(|| bad("scope_restore_not_ready"))?;
    if original_guards.len() != r.rows.len() {
        return Err(bad("scope_backup_corrupt"));
    }
    let mut guards = vec![];
    for i in &indices {
        let a = &r.rows[*i].address;
        let current = images::image(&a.anchor, &a.path)?;
        // Explicit conservative policy: any later target edit requires a fresh user decision.
        let guard = image(&a.anchor, &a.path).map_err(local_error)?;
        if (!matches!(&a.model,Some(TargetV2::Manager(relative)) if relative==versions::SAVED_VERSIONS_PATH)
            && current != after[*i])
            || original_guards[*i]
                .as_ref()
                .is_some_and(|original| &guard != original)
        {
            return Err(bad("scope_restore_stale"));
        }
        guards.push(guard);
    }
    model_check(&d, &h, &r, &indices)?;
    let mut groups = groups.to_vec();
    groups.sort();
    let p = Plan {
        version: 2,
        id: uuid::Uuid::new_v4().to_string(),
        original: original.into(),
        root: d.clone(),
        home: h.clone(),
        source_digest: digest(&r)?,
        groups: groups.clone(),
        guards,
        scope_edit: super::super::sync_service_v2::scope_restore::freeze(
            &d,
            restored_absences(&d, &h, &r, &indices)?,
        )
        .map_err(local_error)?,
    };
    let plan_digest = digest(&p)?;
    save(&d, &plan_path(&d, &p.id, "plan")?, &p)?;
    Ok(RestorePreviewV2 {
        restore_id: p.id,
        plan_digest,
        groups,
    })
}
/// Owned v2 restore journals only. Historical receipts never inspect today's
/// business configuration, allocate a replacement plan, or grant new targets.
pub fn operations(
    d: &Path,
    h: &Path,
    original: Option<&str>,
) -> ModelResult<Vec<RestoreOperationV2>> {
    let (d, h) = roots(d, h)?;
    if let Some(original) = original {
        id(original)?;
    }
    let directory = d.join(PLANS);
    safety::check_path(&d, &directory, true)?;
    if !directory.exists() {
        return Ok(vec![]);
    }
    let mut names = BTreeSet::new();
    let mut count = 0;
    for entry in std::fs::read_dir(&directory)? {
        count += 1;
        if count > 1024 {
            return Err(bad("scope_restore_limit"));
        }
        let entry = entry?;
        safety::check_path(&d, &entry.path(), false)?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| bad("scope_restore_journal"))?;
        let base = name.strip_suffix(".lock").unwrap_or(&name);
        let (restore, suffix) = if let Some(restore) = base.strip_suffix(".plan.json") {
            (restore, "plan")
        } else if let Some(restore) = base.strip_suffix(".progress.json") {
            (restore, "progress")
        } else {
            return Err(bad("scope_restore_journal"));
        };
        id(restore)?;
        if suffix == "progress" && !plan_path(&d, restore, "plan")?.exists() {
            return Err(bad("scope_restore_journal"));
        }
        if suffix == "plan" && !name.ends_with(".lock") {
            names.insert(restore.to_string());
        }
    }
    if names.len() > 256 {
        return Err(bad("scope_restore_limit"));
    }
    let mut result = vec![];
    for restore_id in names {
        let p: Plan = read(&plan_path(&d, &restore_id, "plan")?)?;
        if p.version != 2 || p.id != restore_id || p.root != d || p.home != h {
            return Err(bad("scope_restore_plan"));
        }
        let r: Record = read(&record_path(&d, &p.original)?)?;
        if r.id != p.original || digest(&r)? != p.source_digest {
            return Err(bad("scope_restore_source_stale"));
        }
        validate_record_at(&d, &h, &r, false)?;
        let indices = selected(&r, &p.groups)?;
        if indices.len() != p.guards.len() {
            return Err(bad("scope_restore_plan"));
        }
        let plan_digest = digest(&p)?;
        let path = plan_path(&d, &restore_id, "progress")?;
        let receipt = if path.exists() {
            let progress: Progress = read(&path)?;
            if progress.plan_digest != plan_digest
                || progress.completed.iter().any(|i| !indices.contains(i))
            {
                return Err(bad("scope_restore_journal"));
            }
            if let Some(stored) = &progress.receipt {
                let expected = receipt(&p, &indices, &r, &progress, stored.failure_code.clone());
                if encode(stored)? != encode(&expected)? {
                    return Err(bad("scope_restore_receipt"));
                }
            }
            progress.receipt
        } else {
            None
        };
        if original.is_none_or(|original| original == p.original) {
            result.push(RestoreOperationV2 {
                restore_id,
                original_operation_id: p.original,
                plan_digest,
                groups: p.groups,
                receipt,
            });
        }
    }
    Ok(result)
}
fn ledger_bytes(d: &Path) -> ModelResult<Option<Vec<u8>>> {
    let path = d.join(versions::SAVED_VERSIONS_PATH);
    safety::check_path(d, &path, true)?;
    match std::fs::symlink_metadata(&path) {
        Ok(_) => Ok(Some(safety::read_file(&path, 8 * 1024 * 1024)?)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
fn group_state(d: &Path, r: &Record, group: &str, original_after: bool) -> ModelResult<SaveState> {
    // Rehydrate only this backend-frozen group's private images. Reuse real
    // portable owners and the v2 codec's atom rules, never derive keys by name.
    let stage_parent = d.join(DIRECTORY).join(&r.id);
    let stage = tempfile::Builder::new()
        .prefix("save-state-")
        .tempdir_in(&stage_parent)?;
    let root = stage.path().join("D");
    let home = stage.path().join("H");
    crate::infrastructure::platform::create_private_dir_all(&root, 0o700)?;
    crate::infrastructure::platform::create_private_dir_all(&home, 0o700)?;
    let mut settings = portable_manager::Settings::default();
    let mut assets = portable_assets::Assets::default();
    for (i, row) in r
        .rows
        .iter()
        .enumerate()
        .filter(|(_, row)| row.address.group == group && row.address.model.is_none())
    {
        let material = if original_after {
            r.after
                .as_ref()
                .and_then(|a| a.get(i))
                .ok_or_else(|| bad("scope_restore_not_ready"))?
        } else {
            &row.before
        };
        if let Some(name) = group.strip_prefix("skill:") {
            if material.directory && material.exists {
                let path = root.join("skill");
                images::write_image(&root, &path, &Image::default(), material)?;
                // The participant's actual source target is canonical. When
                // both copies exist they must describe the same saved object.
                let skill = portable_assets::snapshot_skill(&path, name, true, true)?;
                if assets.skills.is_empty() {
                    assets.skills.push(skill);
                } else if assets.skills[0].digest != skill.digest {
                    return Err(bad("scope_restore_asset_state"));
                }
                std::fs::remove_dir_all(path)?;
            }
            continue;
        }
        let Some(raw) = bytes(material)? else {
            continue;
        };
        match group {
            "preferences" => {
                settings.preferences = Some(
                    crate::modules::preferences::preferences_from_document(
                        &serde_json::from_slice(&raw)?,
                    )
                    .map_err(|_| bad("scope_restore_save_state"))?,
                )
            }
            "extensions" => {
                let projection: crate::modules::extensions::projection::ExtensionProjection =
                    serde_json::from_slice(&raw)?;
                if projection.fingerprint.fingerprint != projection.config.fingerprint() {
                    return Err(bad("scope_restore_save_state"));
                }
                let path = root.join(crate::modules::extensions::projection::CONFIG_RELATIVE_PATH);
                atomic_write(&path, &raw, 0o600)?;
                settings.extensions = portable_manager::export(
                    &root,
                    &portable_manager::Scope {
                        extensions: true,
                        ..Default::default()
                    },
                )?
                .extensions;
            }
            "accounts" => settings.accounts = serde_json::from_slice(&raw)?,
            "webdav" => {
                let candidates: Vec<portable_manager::Candidate> = serde_json::from_slice(&raw)?;
                settings.webdav = candidates.into_iter().map(|c| c.endpoint).collect();
                settings.webdav.sort_by(|a, b| a.id.cmp(&b.id));
                settings.webdav.dedup_by(|a, b| a == b);
            }
            "mcp" => {
                let relative = row
                    .address
                    .path
                    .strip_prefix(&r.root)
                    .map_err(|_| bad("scope_restore_asset_state"))?;
                atomic_write(&root.join(relative), &raw, 0o600)?;
                let selected = r
                    .selection
                    .mcp_sources
                    .iter()
                    .map(|m| m.name.clone())
                    .collect::<Vec<_>>();
                let sources = selected
                    .iter()
                    .map(|n| (n.clone(), "managed".into()))
                    .collect();
                assets = portable_assets::export_with_credential_scope(
                    &root,
                    &home,
                    &[],
                    &selected,
                    &sources,
                    false,
                    true,
                    Some(&[]),
                )?
                .0;
            }
            _ => return Err(bad("scope_restore_save_state")),
        }
    }
    let payload = super::super::portable_v2::ModelPayloadV2 {
        templates: vec![],
        policy: None,
    };
    let objects = SavedObjects::capture(
        file_objects_v2::CapturedObjects {
            models: super::super::portable_v2::PortableModelsV2 {
                format: "OCXD_MODEL_PACKAGE".into(),
                format_version: 6,
                schema_version: 2,
                created_at: "2026-11-07T00:00:00Z".into(),
                digest: schema_v2::digest(
                    &schema_v2::canonical_bytes(&payload)
                        .map_err(|_| bad("scope_restore_save_state"))?,
                ),
                payload,
            },
            manager: settings,
            assets,
            channel_keys: file_objects_v2::SecretValues::new(),
            channel_authorizations: BTreeMap::new(),
            webdav_passwords: file_objects_v2::SecretValues::new(),
            content_versions: vec![],
        },
        Some(&ActualAuthorization),
    )?;
    file_objects_v2::save_state(&objects)
}
fn freeze_clocks(
    d: &Path,
    h: &Path,
    r: &Record,
    p: &Plan,
    indices: &[usize],
    progress: &mut Progress,
) -> ModelResult<()> {
    let mut expected = BTreeMap::new();
    if indices.iter().any(|i| r.rows[*i].address.model.is_some()) {
        expected.insert(
            "@models".into(),
            StateDelta {
                before: model_state(d, h, r, indices, true)?,
                after: model_state(d, h, r, indices, false)?,
            },
        );
    }
    for group in &p.groups {
        if indices
            .iter()
            .any(|i| r.rows[*i].address.group == *group && r.rows[*i].address.model.is_none())
        {
            expected.insert(
                group.clone(),
                StateDelta {
                    before: group_state(d, r, group, true)?,
                    after: group_state(d, r, group, false)?,
                },
            );
        }
    }
    if progress.clock_time.is_some() {
        if encode(&progress.deltas)? != encode(&expected)? {
            return Err(bad("scope_restore_clock_state"));
        }
    } else {
        progress.clock_time =
            Some(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true));
        progress.deltas = expected;
    }
    Ok(())
}
fn clock_step(d: &Path, p: &Plan, group: &str, progress: &mut Progress) -> ModelResult<ClockStep> {
    if let Some(step) = progress.clocks.iter().find(|s| s.group == group) {
        return Ok(step.clone());
    }
    let delta = progress
        .deltas
        .get(group)
        .ok_or_else(|| bad("scope_restore_clock_state"))?;
    let planned = versions::plan_save_at(
        d,
        &delta.before,
        &delta.after,
        progress
            .clock_time
            .as_deref()
            .ok_or_else(|| bad("scope_restore_clock_state"))?,
        &p.id,
    )?;
    let after = if delta.before == delta.after {
        planned.before.clone()
    } else {
        Some(planned.after)
    };
    let step = ClockStep {
        group: group.into(),
        ledger_before: planned.before,
        ledger_after: after,
        applied: false,
    };
    progress.clocks.push(step.clone());
    Ok(step)
}
fn write_clock(
    d: &Path,
    h: &Path,
    p: &Plan,
    group: &str,
    progress: &mut Progress,
) -> ModelResult<()> {
    let step = clock_step(d, p, group, progress)?;
    let current = ledger_bytes(d)?;
    if current != step.ledger_after {
        if current != step.ledger_before {
            return Err(bad("scope_restore_clock_stale"));
        }
        transactions::commit_protected_restore(
            d,
            h,
            vec![WriteV2 {
                target: TargetV2::Manager(versions::SAVED_VERSIONS_PATH.into()),
                before: step.ledger_before,
                after: step.ledger_after.clone(),
            }],
        )
        .map_err(|e| bad(&e.code))?;
    }
    if ledger_bytes(d)? != step.ledger_after {
        return Err(bad("scope_restore_clock_readback"));
    }
    if let Some(stored) = progress.clocks.iter_mut().find(|s| s.group == group) {
        stored.applied = true;
    }
    progress.versioned.insert(group.into());
    Ok(())
}
fn verify_protection(d: &Path, progress: &Progress) -> ModelResult<()> {
    for proof in &progress.backups {
        if !proof.directory.starts_with(d.join("backups")) {
            return Err(bad("scope_restore_protection"));
        }
        safety::check_path(d, &proof.directory, false)?;
        let manifest: backup::BackupManifest = serde_json::from_slice(&safety::read_file(
            &proof.directory.join(backup::MANIFEST_NAME),
            64 * 1024,
        )?)?;
        if hash(&encode(&manifest)?) != proof.manifest_hash
            || !backup::verify_backup(&backup::BackupRecord {
                manifest,
                directory: proof.directory.clone(),
            })?
        {
            return Err(bad("scope_restore_protection"));
        }
    }
    Ok(())
}
fn receipt(
    p: &Plan,
    indices: &[usize],
    r: &Record,
    progress: &Progress,
    failure_code: Option<String>,
) -> RestoreReceiptV2 {
    let completed_groups =
        p.groups
            .iter()
            .filter(|group| {
                indices
                    .iter()
                    .filter(|i| &r.rows[**i].address.group == *group)
                    .all(|i| progress.completed.contains(i))
                    && (progress.clock_time.is_none()
                        || progress.versioned.contains(*group)
                        || (r.rows.iter().any(|row| {
                            &row.address.group == *group && row.address.model.is_some()
                        }) && progress.versioned.contains("@models")))
            })
            .cloned()
            .collect();
    let applied = progress.completed.len() == indices.len()
        && (p.scope_edit.is_none() || progress.scope_updated)
        && failure_code.is_none();
    RestoreReceiptV2 {
        restore_id: p.id.clone(),
        original_operation_id: p.original.clone(),
        groups: p.groups.clone(),
        status: if applied {
            RestoreStatusV2::Applied
        } else {
            RestoreStatusV2::Partial
        },
        completed_groups,
        refresh_file_snapshot: applied,
        refresh_sync_scope: progress.scope_updated,
        failure_code,
    }
}
fn with_locks<T>(
    d: &Path,
    h: &Path,
    r: &Record,
    run: impl FnOnce() -> ModelResult<T>,
) -> ModelResult<T> {
    // Match actual receiver and manager/asset writers. The model engine itself takes no target locks.
    let _admission = super::super::content_versions::file_saves::admit(d, h)?;
    let mut pending_guards = vec![];
    for relative in [
        "manager-state/provider-rename-pending.json",
        "manager-state/model-transaction-v2.pending.json",
        "manager-state/credential-references-v2.pending.json",
    ] {
        let path = d.join(relative);
        checked(d, &path).map_err(local_error)?;
        pending_guards.push(TargetFileLock::lock(&path)?);
        if path.exists() {
            return Err(bad("scope_restore_model_recovery_required"));
        }
    }
    let _config = TargetFileLock::lock(&h.join("config.json"))?;
    let store = store_v2::StoreV2::new(d);
    let _model = store.lock().map_err(|_| bad("scope_restore_model_lock"))?;
    let _preferences = backup::manager::acquire_preferences_transaction(d)?;
    let asset_path = d.join("manager-state/extension-assets-transaction");
    checked(d, &asset_path).map_err(local_error)?;
    let _assets = TargetFileLock::lock(&asset_path)?;
    let mut paths: Vec<_> = r
        .rows
        .iter()
        .filter(|row| row.address.model.is_none())
        .map(|row| (&row.address.anchor, &row.address.path))
        .collect();
    paths.sort();
    paths.dedup();
    let _guards = paths
        .iter()
        .map(|(anchor, path)| {
            checked(anchor, &lock_path_for(path)).map_err(local_error)?;
            TargetFileLock::lock(path).map_err(Into::into)
        })
        .collect::<ModelResult<Vec<_>>>()?;
    let _versions = versions::lock_versions(d)?;
    run()
}
pub fn restore(
    d: &Path,
    h: &Path,
    restore_id: &str,
    plan_digest: &str,
) -> ModelResult<RestoreReceiptV2> {
    let _storage = crate::infrastructure::storage_writers::global().admit()?;
    let (d, h) = roots(d, h)?;
    let path = plan_path(&d, restore_id, "plan")?;
    let progress_path = plan_path(&d, restore_id, "progress")?;
    let _guard = TargetFileLock::lock(&path)?;
    let p: Plan = read(&path)?;
    if p.version != 2
        || p.id != restore_id
        || p.root != d
        || p.home != h
        || digest(&p)? != plan_digest
    {
        return Err(bad("scope_restore_plan"));
    }
    let mut progress = if progress_path.exists() {
        let existing: Progress = read(&progress_path)?;
        if existing.plan_digest != plan_digest {
            return Err(bad("scope_restore_plan"));
        }
        // A completed undo is a historical fact. Repeating the same ID never
        // re-checks or overwrites edits made after that successful restore.
        if let Some(receipt) = &existing.receipt {
            if receipt.status == RestoreStatusV2::Applied {
                return Ok(receipt.clone());
            }
        }
        existing
    } else {
        Progress {
            plan_digest: plan_digest.into(),
            protected: false,
            backups: vec![],
            completed: BTreeSet::new(),
            receipt: None,
            clock_time: None,
            deltas: BTreeMap::new(),
            clocks: vec![],
            versioned: BTreeSet::new(),
            scope_updated: false,
        }
    };
    let r = record(&d, &h, &p.original)?;
    if digest(&r)? != p.source_digest {
        return Err(bad("scope_restore_source_stale"));
    }
    let indices = selected(&r, &p.groups)?;
    if p.guards.len() != indices.len() || progress.completed.iter().any(|i| !indices.contains(i)) {
        return Err(bad("scope_restore_plan"));
    }
    with_locks(&d, &h, &r, || {
        let removed = restored_absences(&d, &h, &r, &indices)?;
        super::super::sync_service_v2::scope_restore::validate(&d, p.scope_edit.as_ref(), &removed)
            .map_err(local_error)?;
        let _scope_guard = if p.scope_edit.is_some() {
            Some(super::super::sync_service_v2::scope_restore::lock(&d).map_err(local_error)?)
        } else {
            None
        };
        if let Some(edit) = &p.scope_edit {
            super::super::sync_service_v2::scope_restore::check(
                &d,
                edit,
                progress.protected,
                progress.scope_updated,
            )
            .map_err(local_error)?;
        }
        // Readonly plans never open or classify a draft. Only a write that
        // touches models requires the strict absence witness under real locks.
        if indices.iter().any(|i| r.rows[*i].address.model.is_some()) {
            let draft = d.join(store_v2::DRAFT_PATH);
            safety::check_path(&d, &draft, true)?;
            match std::fs::symlink_metadata(draft) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
                _ => {
                    return Err(invalid(
                        "scope_restore_unsaved_draft",
                        "模型存在未结束的编辑会话，请先保存或明确还原后再恢复备份",
                    ))
                }
            }
        }
        let current_record = record(&d, &h, &p.original)?;
        if digest(&current_record)? != p.source_digest {
            return Err(bad("scope_restore_source_stale"));
        }
        let after = r
            .after
            .as_ref()
            .ok_or_else(|| bad("scope_restore_not_ready"))?;
        for (slot, i) in indices.iter().enumerate() {
            if matches!(&r.rows[*i].address.model,Some(TargetV2::Manager(relative)) if relative==versions::SAVED_VERSIONS_PATH)
            {
                if !progress.protected
                    && image(&d, &r.rows[*i].address.path).map_err(local_error)? != p.guards[slot]
                {
                    return Err(bad("scope_restore_stale"));
                }
                continue;
            }
            let a = &r.rows[*i].address;
            let current = images::image(&a.anchor, &a.path)?;
            if progress.completed.contains(i) {
                if current != r.rows[*i].before {
                    return Err(bad("scope_restore_stale"));
                }
            } else if progress.protected && current == r.rows[*i].before {
                // Crash after a real read-back but before progress publication.
                progress.completed.insert(*i);
            } else if current != after[*i]
                || image(&a.anchor, &a.path).map_err(local_error)? != p.guards[slot]
            {
                return Err(bad("scope_restore_stale"));
            }
        }
        model_check(&d, &h, &r, &indices)?;
        freeze_clocks(&d, &h, &r, &p, &indices, &mut progress)?;
        save(&d, &progress_path, &progress)?;
        if !progress.protected {
            // Protect and verify ALL currently selected targets before the first business write.
            for i in &indices {
                let a = &r.rows[*i].address;
                progress
                    .backups
                    .extend(protect_tree(&d, &a.anchor, &a.path).map_err(local_error)?);
            }
            // Every restore also updates the existing version ledger. Protect
            // that derived target even for manager-only or templates-only scope.
            progress.backups.extend(
                protect_tree(&d, &d, &d.join(versions::SAVED_VERSIONS_PATH))
                    .map_err(local_error)?,
            );
            if p.scope_edit.is_some() {
                let path = super::super::sync_service_v2::scope_restore::path(&d);
                progress
                    .backups
                    .extend(protect_tree(&d, &d, &path).map_err(local_error)?);
            }
            verify_protection(&d, &progress)?;
            for (slot, i) in indices.iter().enumerate() {
                let a = &r.rows[*i].address;
                if image(&a.anchor, &a.path).map_err(local_error)? != p.guards[slot] {
                    return Err(bad("scope_restore_stale"));
                }
            }
            progress.protected = true;
            save(&d, &progress_path, &progress)?;
        }
        verify_protection(&d, &progress)?;
        // Deselect before removing objects: an interrupted undo must never leave
        // automatic sync authorized to immediately receive the removed scope.
        if let Some(edit) = &p.scope_edit {
            if let Err(error) = super::super::sync_service_v2::scope_restore::apply(&d, edit) {
                let result = receipt(
                    &p,
                    &indices,
                    &r,
                    &progress,
                    Some(format!("{:?}", error.code)),
                );
                progress.receipt = Some(result.clone());
                save(&d, &progress_path, &progress)?;
                return Ok(result);
            }
            progress.scope_updated = true;
            save(&d, &progress_path, &progress)?;
        }
        // Model/config/Policy/ledger/mappings/versions use the existing typed
        // exact-image engine, including its durable compensation direction.
        let model_indices: Vec<_> = indices
            .iter()
            .copied()
            .filter(|i| r.rows[*i].address.model.is_some())
            .collect();
        let unfinished: Vec<_> = model_indices
            .iter()
            .copied()
            .filter(|i| !progress.completed.contains(i))
            .collect();
        if !unfinished.is_empty() {
            let clock = clock_step(&d, &p, "@models", &mut progress)?;
            save(&d, &progress_path, &progress)?;
            let mut writes = unfinished
                .iter()
                .filter(|i|!matches!(&r.rows[**i].address.model,Some(TargetV2::Manager(relative)) if relative==versions::SAVED_VERSIONS_PATH))
                .map(|i| {
                    Ok(WriteV2 {
                        target: r.rows[*i]
                            .address
                            .model
                            .clone()
                            .ok_or_else(|| bad("scope_restore_model_target"))?,
                        before: bytes(&after[*i])?,
                        after: bytes(&r.rows[*i].before)?,
                    })
                })
                .collect::<ModelResult<Vec<_>>>()?;
            writes.push(WriteV2 {
                target: TargetV2::Manager(versions::SAVED_VERSIONS_PATH.into()),
                before: clock.ledger_before.clone(),
                after: clock.ledger_after.clone(),
            });
            if let Err(error) = transactions::commit_protected_restore(&d, &h, writes) {
                let result = receipt(&p, &indices, &r, &progress, Some(error.code));
                progress.receipt = Some(result.clone());
                save(&d, &progress_path, &progress)?;
                return Ok(result);
            }
            for i in unfinished {
                let a = &r.rows[i].address;
                if matches!(&a.model,Some(TargetV2::Manager(relative)) if relative==versions::SAVED_VERSIONS_PATH)
                {
                    if ledger_bytes(&d)? != clock.ledger_after {
                        return Err(bad("scope_restore_clock_readback"));
                    }
                } else if images::image(&a.anchor, &a.path)? != r.rows[i].before {
                    return Err(bad("scope_restore_model_readback"));
                }
                progress.completed.insert(i);
            }
            if let Some(step) = progress.clocks.iter_mut().find(|s| s.group == "@models") {
                step.applied = true;
            }
            progress.versioned.insert("@models".into());
            save(&d, &progress_path, &progress)?;
        }
        for i in &indices {
            if progress.completed.contains(i) {
                continue;
            }
            let a = &r.rows[*i].address;
            let current = images::image(&a.anchor, &a.path)?;
            if current != after[*i] {
                return Err(bad("scope_restore_stale"));
            }
            if let Err(error) =
                images::write_image(&a.anchor, &a.path, &current, &r.rows[*i].before)
            {
                // Compensate this one helper-owned tree only. Prior completed
                // participants retain exact durable progress and can continue.
                let live = images::image(&a.anchor, &a.path)?;
                let compensated = live == current
                    || (images::owned_intermediate(&live, &current, &r.rows[*i].before)
                        && images::write_image(&a.anchor, &a.path, &live, &current).is_ok());
                let result = receipt(
                    &p,
                    &indices,
                    &r,
                    &progress,
                    Some(if compensated {
                        error.code
                    } else {
                        "scope_restore_partial_compensation".into()
                    }),
                );
                progress.receipt = Some(result.clone());
                save(&d, &progress_path, &progress)?;
                return Ok(result);
            }
            progress.completed.insert(*i);
            save(&d, &progress_path, &progress)?;
            let group = &r.rows[*i].address.group;
            if indices
                .iter()
                .filter(|index| r.rows[**index].address.group == *group)
                .all(|index| progress.completed.contains(index))
            {
                let _ = clock_step(&d, &p, group, &mut progress)?;
                save(&d, &progress_path, &progress)?;
                if let Err(error) = write_clock(&d, &h, &p, group, &mut progress) {
                    let partial = receipt(&p, &indices, &r, &progress, Some(error.code));
                    progress.receipt = Some(partial.clone());
                    save(&d, &progress_path, &progress)?;
                    return Ok(partial);
                }
                save(&d, &progress_path, &progress)?;
            }
        }
        for group in progress.deltas.keys().cloned().collect::<Vec<_>>() {
            if !progress.versioned.contains(&group) {
                let _ = clock_step(&d, &p, &group, &mut progress)?;
                save(&d, &progress_path, &progress)?;
                if let Err(error) = write_clock(&d, &h, &p, &group, &mut progress) {
                    let partial = receipt(&p, &indices, &r, &progress, Some(error.code));
                    progress.receipt = Some(partial.clone());
                    save(&d, &progress_path, &progress)?;
                    return Ok(partial);
                }
                save(&d, &progress_path, &progress)?;
            }
        }
        let result = receipt(&p, &indices, &r, &progress, None);
        progress.receipt = Some(result.clone());
        save(&d, &progress_path, &progress)?;
        Ok(result)
    })
}
#[cfg(test)]
#[path = "sync_local_v2_restore_tests.rs"]
mod tests;
