//! Version 1 of the 0.2.15 layout upgrade. This module never opens config.json,
//! calls the credential store, replays a journal, or promotes an old draft.
//!
//! Integration: replace the ordinary policy fallback with `load_layout(root)?`.
//! Call `migrate(root)` at upgrade/startup when its per-run info diagnostics and
//! cleanup report are wanted. Layout writers should share the layout file lock.
//! Publication also uses create-if-absent semantics, so even an uncooperative
//! layout writer cannot be overwritten by this upgrade. No completion/status
//! document is needed: the layout and content-addressed archives are repeatable.
//!
//! Only channel/provider identities, labels, node identities/names/addresses,
//! the active node *slot*, and exact template adoptions cross the boundary.
//! Protocol, enabled/auth state, credentials/import references, selections,
//! rules, model naming/order, default-provider intent and recovery/removal state
//! are excluded. `config_file_mapping` supplies those facts from config.json.

use super::schema::*;
use crate::{
    infrastructure::{
        atomic_write::atomic_write,
        locking::{lock_path_for, TargetFileLock},
    },
    modules::backup::safety,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::ErrorKind,
    path::{Path, PathBuf},
    time::Duration,
};
use zeroize::Zeroizing;

pub const MIGRATION_VERSION: u32 = 1;
pub const LAYOUT_PATH: &str = "manager-state/model-channel-layout.json";
pub const ARCHIVE_DIR: &str = "manager-state/legacy-model-config/v1";
const IDS_PATH: &str = "manager-state/model-execution-source-ids.json";
const RETIRED_PATH: &str = "manager-state/model-client-state.json";
const JOIN_OPERATIONS: &str = "manager-state/new-channel-operations";
const JOIN_ADMISSION: &str = "manager-state/new-channel-admission";
const JOIN_LOCK: &str = "manager-state/new-channel-admission.lock";
const STAGING_DIR: &str = "manager-state/.model-layout-upgrade-v1";
const STAGING_FILE: &str = "layout.json";
const MAX_DOCUMENT: u64 = 16 * 1024 * 1024;
const MAX_ENTRIES: usize = 10_000;
const MAX_JOIN_CANDIDATES: usize = 256;
const MAX_JOIN_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    ArchiveAndRetain,
    ArchiveAndRetireTerminal,
    RetainHistory,
    RetainActive,
    RemoveDeadFile,
    RemoveOwnedUnreferencedTemp,
}

/// Consumer evidence is intentionally specific. A future consumer removal is
/// not authorization to delete a whole namespace or its credential references.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsumerDisposition {
    pub path: &'static str,
    pub disposition: Disposition,
    pub consumers: &'static str,
}
pub const DISPOSITIONS: &[ConsumerDisposition] = &[
    ConsumerDisposition { path: POLICY_PATH, disposition: Disposition::ArchiveAndRetain, consumers: "store::ModelStore historical load/publish/recover_pending; runtime explicit historical restoration. config_file ordinary metadata reads only load_layout." },
    ConsumerDisposition { path: DRAFT_PATH, disposition: Disposition::ArchiveAndRetain, consumers: "store::ModelStore workspace/stage; runtime/recovery explicit restoration. Candidate only, never current layout input." },
    ConsumerDisposition { path: IDS_PATH, disposition: Disposition::ArchiveAndRetain, consumers: "execution_directory::stable_ids compatibility registry; preserve saved Agent selection identities during layout upgrade. Policy identity wins for the same provider." },
    ConsumerDisposition { path: "manager-state/model-source-reads", disposition: Disposition::RetainHistory, consumers: "discovery_paths and channel_actions load exact source reads; runtime source/slug audit." },
    ConsumerDisposition { path: "manager-state/model-save-plans", disposition: Disposition::RetainHistory, consumers: "runtime plan loading, success history, compensation and explicit restoration snapshots." },
    ConsumerDisposition { path: "manager-state/model-recovery-plans", disposition: Disposition::RetainHistory, consumers: "recovery loads explicit restoration candidates and source baselines." },
    ConsumerDisposition { path: "manager-state/model-runtime-ownership.json", disposition: Disposition::RetainActive, consumers: "runtime historical save/restore ownership checks." },
    ConsumerDisposition { path: "manager-state/model-config-journal.json", disposition: Disposition::RetainActive, consumers: "store::ModelStore::recover_pending replays a validated interrupted transaction." },
    ConsumerDisposition { path: "manager-state/model-config-transaction.lock", disposition: Disposition::RetainActive, consumers: "store::ModelStore transaction lock; removing a lock file could split live lock ownership." },
    ConsumerDisposition { path: "manager-state/model-file-incoming", disposition: Disposition::RetainActive, consumers: "file_objects incoming-plan adoption/cancellation, including protected authorization candidates." },
    ConsumerDisposition { path: "manager-state/imported-provider-configs", disposition: Disposition::RetainActive, consumers: "provider_import exact imported_config references; may still contain required authorization facts." },
    ConsumerDisposition { path: JOIN_OPERATIONS, disposition: Disposition::ArchiveAndRetireTerminal, consumers: "No ordinary reader/writer: new_channel add/result/latest and their IPC wrappers were removed. Baseline 56c11220 latest_unlocked/reconcile treats only committed/failed as terminal; conflict, cleanup_pending and noWriteProven candidates stay. Retire terminal records only under the nonblocking old admission lock after exact archive verification and unchanged source readback. Never traverse or delete Key/import references." },
    ConsumerDisposition { path: JOIN_LOCK, disposition: Disposition::ArchiveAndRetain, consumers: "Retired new_channel admission lock; an older process may still hold its inode. Archive only after nonblocking acquisition; create a missing lock only for a populated operations directory. Never unlink the lock inode." },
    ConsumerDisposition { path: "manager-state/model-template-tasks", disposition: Disposition::RetainActive, consumers: "tasks prepare/load/submission receipt recovery; library submission_receipts also remain untouched." },
    ConsumerDisposition { path: "backups", disposition: Disposition::RetainHistory, consumers: "backup browser/restore and runtime protected backups, including model-config-draft.json.bak-* candidates." },
    ConsumerDisposition { path: RETIRED_PATH, disposition: Disposition::RemoveDeadFile, consumers: "No reader/writer remains; commands::retired_files::RETIRED_FILES already declares this exact file dead." },
    ConsumerDisposition { path: STAGING_DIR, disposition: Disposition::RemoveOwnedUnreferencedTemp, consumers: "Only this module writes layout.json and its exact atomic_write temporary siblings, under the layout lock. No history/journal consumer reads them. Other temporary namespaces are retained." },
];

/// These are info diagnostics for this invocation, not a persistent UI state.
/// Messages/codes never interpolate legacy JSON, credentials or absolute paths.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub level: &'static str,
    pub code: &'static str,
    pub path: &'static str,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationReport {
    pub version: u32,
    pub layout: Policy,
    pub created_layout: bool,
    /// Relative paths of byte-for-byte copies, including already present copies.
    pub archives: Vec<String>,
    pub removed: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
    pub dispositions: &'static [ConsumerDisposition],
}
impl MigrationReport {
    fn new(layout: Policy) -> Self {
        Self {
            version: MIGRATION_VERSION,
            layout,
            created_layout: false,
            archives: vec![],
            removed: vec![],
            diagnostics: vec![],
            dispositions: DISPOSITIONS,
        }
    }
    fn info(&mut self, code: &'static str, path: &'static str) {
        self.diagnostics.push(Diagnostic {
            level: "info",
            code,
            path,
        });
    }
}

fn path(root: &Path, relative: &str) -> ModelResult<PathBuf> {
    let path = root.join(relative);
    safety::check_path(root, &path, true)?;
    Ok(path)
}
fn read(root: &Path, relative: &str) -> ModelResult<Option<Zeroizing<Vec<u8>>>> {
    let path = path(root, relative)?;
    match std::fs::symlink_metadata(&path) {
        Ok(_) => Ok(Some(Zeroizing::new(safety::read_file(
            &path,
            MAX_DOCUMENT,
        )?))),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
fn text(value: &Value, limit: usize) -> Option<&str> {
    value
        .as_str()
        .filter(|s| s.len() <= limit && !s.chars().any(char::is_control))
}
fn identity(value: &Value) -> Option<&str> {
    value.as_str().filter(|s| require_uuid(s).is_ok())
}
fn provider(value: &Value) -> Option<&str> {
    text(value, 256).filter(|s| !s.trim().is_empty())
}
fn node_address(value: &Value) -> &str {
    let Some(value) = text(value, 8192) else {
        return "";
    };
    // Match the existing discovery URL contract without DNS/network access.
    // A damaged address must not copy embedded credentials into layout JSON;
    // keep the slot's UUID/name and recover its address from the current file.
    if reqwest::Url::parse(value).is_ok_and(|url| {
        ["http", "https"].contains(&url.scheme())
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none()
    }) {
        value
    } else {
        ""
    }
}
fn reference(value: &Value) -> Option<TemplateRef> {
    Some(TemplateRef {
        template_id: identity(&value["templateId"])?.into(),
        revision_id: identity(&value["revisionId"])?.into(),
    })
}
fn references(value: &Value) -> Vec<TemplateRef> {
    let mut seen = BTreeSet::new();
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(reference)
        .filter(|r| seen.insert(r.template_id.clone()))
        .take(MAX_ENTRIES)
        .collect()
}

/// Read only the metadata allowlist, so an invalid retired factual field cannot
/// prevent salvage of a valid UUID, node or adoption. The raw original is kept.
fn sanitize(bytes: &[u8]) -> Option<Policy> {
    let value: Value = serde_json::from_slice(bytes).ok()?;
    if value["schemaVersion"].as_u64()? != u64::from(SCHEMA_VERSION)
        || !value["channels"].is_array()
        || !value["templateRefs"].is_array()
    {
        return None;
    }
    let mut policy = Policy {
        template_refs: references(&value["templateRefs"]),
        ..Policy::default()
    };
    let mut providers = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for row in value["channels"].as_array()?.iter().take(MAX_ENTRIES) {
        let Some(provider) = provider(&row["connection"]["providerId"]) else {
            continue;
        };
        if !providers.insert(provider.to_owned()) {
            continue;
        }
        let id = identity(&row["id"])
            .filter(|id| ids.insert((*id).to_owned()))
            .unwrap_or("");
        let mut channel = empty_channel(provider, id);
        if let Some(label) = text(&row["label"], 256).filter(|s| !s.trim().is_empty()) {
            channel.label = label.into();
        }
        channel.template_refs = references(&row["templateRefs"]);
        if let Some(refs) = row["modelRefs"].as_object() {
            for (model, value) in refs.iter().take(MAX_ENTRIES) {
                if !model.is_empty() && model.len() <= 1024 && !model.chars().any(char::is_control)
                {
                    if let Some(reference) = reference(value) {
                        channel.model_refs.insert(model.clone(), reference);
                    }
                }
            }
        }
        let mut nodes = BTreeSet::new();
        for node in row["connection"]["nodes"]
            .as_array()
            .into_iter()
            .flatten()
            .take(MAX_ENTRIES)
        {
            if let (Some(id), Some(name)) = (identity(&node["id"]), text(&node["name"], 256)) {
                if nodes.insert(id.to_owned()) {
                    channel.connection.nodes.push(ServiceNode {
                        id: id.into(),
                        name: name.into(),
                        url: node_address(&node["url"]).into(),
                    });
                }
            }
        }
        if let Some(active) =
            identity(&row["connection"]["activeNodeId"]).filter(|id| nodes.contains(*id))
        {
            channel.connection.active_node_id = active.into();
        }
        policy.channels.push(channel);
    }
    Some(policy)
}
fn empty_channel(provider: &str, id: &str) -> ChannelPolicy {
    ChannelPolicy {
        id: id.into(),
        label: provider.into(),
        template_refs: vec![],
        model_refs: BTreeMap::new(),
        connection: ChannelConnection {
            provider_id: provider.into(),
            ..Default::default()
        },
        selection: ModelSelection::None,
        model_order: vec![],
        discovered: vec![],
        defaults: ModelRules::default(),
        overrides: BTreeMap::new(),
        family_overrides: BTreeMap::new(),
        naming: BTreeMap::new(),
    }
}
fn source_ids(bytes: &[u8]) -> Option<BTreeMap<String, String>> {
    let value: Value = serde_json::from_slice(bytes).ok()?;
    Some(
        value
            .as_object()?
            .iter()
            .take(MAX_ENTRIES)
            .filter_map(|(p, id)| {
                let p = Value::String(p.clone());
                Some((provider(&p)?.into(), identity(id)?.into()))
            })
            .collect(),
    )
}
fn merge_ids(layout: &mut Policy, source: BTreeMap<String, String>) {
    let mut used: BTreeSet<_> = layout
        .channels
        .iter()
        .filter(|c| !c.id.is_empty())
        .map(|c| c.id.clone())
        .collect();
    for (provider, id) in source {
        if let Some(channel) = layout
            .channels
            .iter_mut()
            .find(|c| c.connection.provider_id == provider)
        {
            if channel.id.is_empty() && used.insert(id.clone()) {
                channel.id = id;
            }
        } else if used.insert(id.clone()) {
            layout.channels.push(empty_channel(&provider, &id));
        }
    }
    // Empty/duplicate damaged ids remain absent metadata. The file mapper uses
    // its deterministic provider identity, rather than making a random new id.
}

fn archive(root: &Path, relative: &str, bytes: &[u8], quarantine: bool) -> ModelResult<String> {
    let name = Path::new(relative)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| invalid("layout_archive", "旧布局归档名称无效"))?;
    let relative = format!(
        "{ARCHIVE_DIR}/{}/{name}-{:x}.json",
        if quarantine { "quarantine" } else { "archive" },
        Sha256::digest(bytes)
    );
    match read(root, &relative)? {
        Some(existing) if existing.as_slice() == bytes => {}
        Some(_) => {
            return Err(invalid(
                "layout_archive",
                "旧布局归档已存在不同内容，原文件已保留",
            ))
        }
        None => {
            atomic_write(&path(root, &relative)?, bytes, 0o600)?;
            if read(root, &relative)?.as_deref().map(|v| v.as_slice()) != Some(bytes) {
                return Err(invalid(
                    "layout_archive",
                    "旧布局归档读回不同，原文件已保留",
                ));
            }
        }
    }
    Ok(relative)
}
fn preserve(
    root: &Path,
    relative: &'static str,
    bytes: &[u8],
    quarantine: bool,
    report: &mut MigrationReport,
) {
    match archive(root, relative, bytes, quarantine) {
        Ok(path) => {
            report.archives.push(path);
            if quarantine {
                report.info("legacy_quarantined_original_retained", relative);
            }
        }
        Err(_) => report.info("archive_deferred_original_retained", relative),
    }
}
fn legacy(
    root: &Path,
    relative: &'static str,
    report: &mut MigrationReport,
) -> Option<Zeroizing<Vec<u8>>> {
    match read(root, relative) {
        Ok(bytes) => bytes,
        Err(_) => {
            report.info("legacy_unreadable_original_retained", relative);
            None
        }
    }
}

fn current(root: &Path) -> ModelResult<Option<Policy>> {
    read(root, LAYOUT_PATH)?
        .map(|bytes| {
            sanitize(&bytes)
                .ok_or_else(|| invalid("layout_document", "当前布局元数据格式无效，原文件已保留"))
        })
        .transpose()
}

/// Ordinary reads use only the new layout once present, never legacy fallback.
/// A missing layout is initialized by the same repeatable upgrade as startup.
pub fn load_layout(root: &Path) -> ModelResult<Policy> {
    match current(root)? {
        Some(layout) => Ok(layout),
        None => Ok(migrate(root)?.layout),
    }
}

/// Archive candidates before publishing. Legacy read/parse/archive failures are
/// info diagnostics with originals retained; they cannot deny a fresh layout.
/// Errors are limited to the required new layout/root/lock/publication itself.
/// Existing layout bytes are never changed, including on explicit repeat runs.
pub fn migrate(root: &Path) -> ModelResult<MigrationReport> {
    let target = path(root, LAYOUT_PATH)?;
    path(root, &format!("{LAYOUT_PATH}.lock"))?;
    let _guard = TargetFileLock::lock(&target)?;
    safety::check_path(root, &lock_path_for(&target), false)?;
    let existing = current(root)?;
    let mut report = MigrationReport::new(existing.clone().unwrap_or_default());
    let policy = legacy(root, POLICY_PATH, &mut report);
    let old_layout = policy.as_deref().and_then(|b| sanitize(b));
    if let Some(bytes) = &policy {
        preserve(root, POLICY_PATH, bytes, old_layout.is_none(), &mut report);
    }
    let ids = legacy(root, IDS_PATH, &mut report);
    let old_ids = ids.as_deref().and_then(|b| source_ids(b));
    if let Some(bytes) = &ids {
        preserve(root, IDS_PATH, bytes, old_ids.is_none(), &mut report);
    }
    if let Some(bytes) = legacy(root, DRAFT_PATH, &mut report) {
        let valid = serde_json::from_slice::<PolicyDraft>(&bytes).is_ok();
        preserve(root, DRAFT_PATH, &bytes, !valid, &mut report);
    }
    preserve_join_candidates(root, &mut report);
    if existing.is_none() {
        let mut layout = old_layout.unwrap_or_default();
        merge_ids(&mut layout, old_ids.unwrap_or_default());
        let staging = path(root, &format!("{STAGING_DIR}/{STAGING_FILE}"))?;
        let bytes = serde_json::to_vec(&layout)?;
        atomic_write(&staging, &bytes, 0o600)?;
        // A hard link publishes one fully flushed file atomically without
        // replacing a layout created after our initial existence check. Both
        // names are inside the same data root; no external link is followed.
        safety::check_path(root, &target, true)?;
        match std::fs::hard_link(&staging, &target) {
            Ok(()) => {
                report.created_layout = true;
                report.layout = layout;
            }
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                report.layout = current(root)?
                    .ok_or_else(|| invalid("layout_changed", "布局在升级期间发生变化，请重试"))?;
                report.info("current_layout_won_race", LAYOUT_PATH);
            }
            Err(e) => return Err(e.into()),
        }
        if current(root)?.as_ref() != Some(&report.layout) {
            return Err(invalid("layout_readback", "布局写后读回不同，旧文件已保留"));
        }
    }
    cleanup(root, &mut report);
    Ok(report)
}

fn remove_regular(root: &Path, relative: &str) -> ModelResult<bool> {
    let path = path(root, relative)?;
    match std::fs::symlink_metadata(&path) {
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
        Ok(_) => {
            let file = safety::open_file(&path, false)?;
            // Unlink only the checked name. In particular the committed layout
            // and our interrupted staging file may temporarily share an inode;
            // removing the staging name never modifies the committed bytes.
            drop(file);
            std::fs::remove_file(path)?;
            Ok(true)
        }
    }
}

/// Exact retired Record shape from 56c11220:new_channel.rs. Requiring the known
/// shape keeps partial/future/corrupt documents recoverable at their old path.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RetiredJoinRecord {
    id: String,
    created_at: String,
    expected_policy: String,
    expected_workspace: String,
    request_digest: String,
    channel: ChannelPolicy,
    #[serde(default)]
    replace: Option<ChannelPolicy>,
    phase: String,
    #[serde(rename = "error")]
    _error: Option<ModelError>,
    // This was true on prepared and cleanup_pending records too, so it proves
    // no terminal outcome. In particular a conflict is still recoverable.
    #[serde(default, rename = "noWriteProven")]
    _no_write_proven: bool,
}
fn terminal_join_record(relative: &str, bytes: &[u8]) -> bool {
    if bytes.len() > 2 * 1024 * 1024 {
        return false;
    }
    let Ok(record) = serde_json::from_slice::<RetiredJoinRecord>(bytes) else {
        return false;
    };
    let digest = |s: &str| {
        s.strip_prefix("sha256:").is_some_and(|s| {
            s.len() == 64
                && s.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        })
    };
    ["committed", "failed"].contains(&record.phase.as_str())
        && require_uuid(&record.id).is_ok()
        && Path::new(relative).file_stem().and_then(|s| s.to_str()) == Some(record.id.as_str())
        && chrono::DateTime::parse_from_rfc3339(&record.created_at).is_ok()
        && digest(&record.expected_policy)
        && digest(&record.expected_workspace)
        && digest(&record.request_digest)
        && super::compiler::validate_channel(&record.channel).is_ok()
        && record
            .replace
            .as_ref()
            .is_none_or(|c| super::compiler::validate_channel(c).is_ok())
}

/// Caller holds the old admission lock throughout archive, verification and
/// unlink. Re-read both names immediately before removal; drift retains source.
fn retire_archived_join(
    root: &Path,
    relative: &str,
    archived: &str,
    bytes: &[u8],
) -> ModelResult<bool> {
    if read(root, archived)?.as_ref().map(|b| b.as_slice()) != Some(bytes) {
        return Err(invalid(
            "candidate_archive_changed",
            "旧加入记录归档读回不同，原文件已保留",
        ));
    }
    if read(root, relative)?.as_ref().map(|b| b.as_slice()) != Some(bytes) {
        return Ok(false);
    }
    remove_regular(root, relative)
}

fn preserve_join_candidates(root: &Path, report: &mut MigrationReport) {
    let mut candidates = Vec::new();
    let Ok(directory) = path(root, JOIN_OPERATIONS) else {
        report.info("candidate_archive_deferred", JOIN_OPERATIONS);
        return;
    };
    match std::fs::read_dir(directory) {
        Ok(entries) => {
            for (index, entry) in entries.take(MAX_JOIN_CANDIDATES + 1).enumerate() {
                if index == MAX_JOIN_CANDIDATES {
                    report.info(
                        "candidate_archive_limit_originals_retained",
                        JOIN_OPERATIONS,
                    );
                    break;
                }
                let Ok(entry) = entry else {
                    report.info("candidate_archive_deferred", JOIN_OPERATIONS);
                    continue;
                };
                let name = entry.file_name();
                if let Some(name) = name.to_str().filter(|name| {
                    name.strip_suffix(".json")
                        .is_some_and(|id| require_uuid(id).is_ok())
                }) {
                    candidates.push(format!("{JOIN_OPERATIONS}/{name}"));
                }
            }
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(_) => {
            report.info("candidate_archive_deferred", JOIN_OPERATIONS);
            return;
        }
    }
    let Ok(lock_path) = path(root, JOIN_LOCK) else {
        report.info("admission_archive_deferred_lock_retained", JOIN_LOCK);
        return;
    };
    match std::fs::symlink_metadata(&lock_path) {
        Ok(m) if m.is_file() => {}
        // A lock may be created only for a populated retired operations dir.
        Err(e) if e.kind() == ErrorKind::NotFound => {
            if candidates.is_empty() {
                return;
            }
        }
        _ => {
            report.info("admission_archive_deferred_lock_retained", JOIN_LOCK);
            return;
        }
    }
    let Ok(target) = path(root, JOIN_ADMISSION) else {
        report.info("admission_archive_deferred_lock_retained", JOIN_LOCK);
        return;
    };
    let Ok(guard) = TargetFileLock::try_lock_with_timeout(&target, Duration::ZERO) else {
        report.info("admission_archive_deferred_lock_retained", JOIN_LOCK);
        return;
    };
    if safety::check_path(root, &lock_path, false).is_err() {
        report.info("admission_archive_deferred_lock_retained", JOIN_LOCK);
        return;
    }
    // Never unlink this inode: an older process can be waiting for the same
    // lock. Busy admission skips both archival and cleanup of its records.
    match guard.read_metadata(MAX_DOCUMENT) {
        Ok(bytes) => preserve(root, JOIN_LOCK, &bytes, false, report),
        Err(_) => report.info(
            "admission_metadata_archive_deferred_lock_retained",
            JOIN_LOCK,
        ),
    }
    let mut remaining = MAX_JOIN_BYTES;
    for relative in candidates {
        let bytes = match read(root, &relative) {
            Ok(Some(b)) => b,
            Ok(None) => continue,
            Err(_) => {
                report.info("candidate_archive_deferred", JOIN_OPERATIONS);
                continue;
            }
        };
        let Some(budget) = remaining.checked_sub(bytes.len() as u64) else {
            report.info(
                "candidate_archive_limit_originals_retained",
                JOIN_OPERATIONS,
            );
            break;
        };
        remaining = budget;
        let archived = match archive(
            root,
            &relative,
            &bytes,
            serde_json::from_slice::<Value>(&bytes).is_err(),
        ) {
            Ok(p) => p,
            Err(_) => {
                report.info("candidate_archive_deferred", JOIN_OPERATIONS);
                continue;
            }
        };
        report.archives.push(archived.clone());
        if terminal_join_record(&relative, &bytes) {
            match retire_archived_join(root, &relative, &archived, &bytes) {
                Ok(true) => report.removed.push(relative),
                Ok(false) => report.info("candidate_changed_original_retained", JOIN_OPERATIONS),
                Err(_) => report.info(
                    "candidate_cleanup_deferred_original_retained",
                    JOIN_OPERATIONS,
                ),
            }
        }
    }
}
fn owned_temp(name: &str) -> bool {
    if name == STAGING_FILE {
        return true;
    }
    let Some(tail) = name.strip_prefix(".layout.json.tmp-") else {
        return false;
    };
    let parts: Vec<_> = tail.split('-').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
        && parts[0].parse::<u32>().is_ok()
        && parts[1].parse::<u8>().is_ok_and(|n| n < 16)
        && parts[2].parse::<u128>().is_ok()
}
fn cleanup(root: &Path, report: &mut MigrationReport) {
    match remove_regular(root, RETIRED_PATH) {
        Ok(true) => report.removed.push(RETIRED_PATH.into()),
        Ok(false) => {}
        Err(_) => report.info("retired_cleanup_deferred", RETIRED_PATH),
    }
    let directory = match path(root, STAGING_DIR) {
        Ok(p) => p,
        Err(_) => {
            report.info("temp_cleanup_deferred", STAGING_DIR);
            return;
        }
    };
    let entries = match std::fs::read_dir(directory) {
        Ok(e) => e,
        Err(e) if e.kind() == ErrorKind::NotFound => return,
        Err(_) => {
            report.info("temp_cleanup_deferred", STAGING_DIR);
            return;
        }
    };
    // No recursive walk and no assumption that a .tmp file elsewhere is dead.
    for entry in entries.take(MAX_ENTRIES) {
        let Ok(entry) = entry else {
            report.info("temp_cleanup_deferred", STAGING_DIR);
            continue;
        };
        let name = entry.file_name();
        let Some(name) = name.to_str().filter(|s| owned_temp(s)) else {
            continue;
        };
        let relative = format!("{STAGING_DIR}/{name}");
        match remove_regular(root, &relative) {
            Ok(true) => report.removed.push(relative),
            Ok(false) => {}
            Err(_) => report.info("temp_cleanup_deferred", STAGING_DIR),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{config_file, config_file_mapping};
    use super::*;
    use serde_json::json;

    const CHANNEL: &str = "11111111-1111-4111-8111-111111111111";
    const SOURCE: &str = "22222222-2222-4222-8222-222222222222";
    const NODE: &str = "33333333-3333-4333-8333-333333333333";
    const BACKUP_NODE: &str = "44444444-4444-4444-8444-444444444444";
    const TEMPLATE: &str = "55555555-5555-4555-8555-555555555555";
    const REVISION: &str = "66666666-6666-4666-8666-666666666666";
    const CREDENTIAL: &str = "77777777-7777-4777-8777-777777777777";
    const OPERATION: &str = "88888888-8888-4888-8888-888888888888";

    fn put(root: &Path, relative: &str, bytes: &[u8]) {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }
    fn put_json(root: &Path, relative: &str, value: &Value) -> Vec<u8> {
        let bytes = serde_json::to_vec_pretty(value).unwrap();
        put(root, relative, &bytes);
        bytes
    }
    fn join_record(id: &str, phase: &str, no_write_proven: bool) -> Value {
        let mut channel = empty_channel("saved", CHANNEL);
        channel.connection.protocol = "openai-chat".into();
        channel.connection.nodes.push(ServiceNode {
            id: NODE.into(),
            name: "Saved node".into(),
            url: "https://fixture.example.test/v1".into(),
        });
        channel.connection.active_node_id = NODE.into();
        channel.connection.credential_ref = Some(CREDENTIAL.into());
        json!({
            "id": id, "createdAt": "2026-10-27T00:00:00Z",
            "expectedPolicy": super::super::store::sha256(b"policy"),
            "expectedWorkspace": super::super::store::sha256(b"workspace"),
            "requestDigest": super::super::store::sha256(b"request"),
            "channel": channel, "replace": null, "phase": phase,
            "error": null, "noWriteProven": no_write_proven,
        })
    }
    fn policy() -> Value {
        let mut value = serde_json::to_value(Policy::default()).unwrap();
        let adoption = json!({"templateId": TEMPLATE, "revisionId": REVISION});
        value["templateRefs"] = json!([adoption]);
        value["defaultChannelId"] = json!(CHANNEL);
        value["recoveryId"] = json!(OPERATION);
        value["pendingRemovals"] = json!([{"readId": SOURCE, "channelId": CHANNEL, "providerId": "obsolete", "credentialRef": CREDENTIAL}]);
        let mut channel = serde_json::to_value(empty_channel("saved", CHANNEL)).unwrap();
        channel["label"] = json!("My saved channel");
        channel["templateRefs"] = json!([adoption]);
        channel["modelRefs"] = json!({"exact/model": adoption});
        channel["connection"] = json!({
            "providerId": "saved", "protocol": "stale-protocol", "enabled": true,
            "allowPrivateNetwork": true, "authMode": "forward",
            "credentialRef": CREDENTIAL, "sourceReadId": SOURCE,
            "importedConfig": {"id": OPERATION, "digest": "sensitive-import"},
            "nodes": [
                {"id": NODE, "name": "Main node", "url": "https://stale.example.test/v1"},
                {"id": BACKUP_NODE, "name": "Backup node", "url": "https://backup.example.test/v1"}
            ], "activeNodeId": NODE, "apiKey": "never-copy-this-key"
        });
        channel["selection"] = json!({"mode": "whitelist", "models": ["stale-model"]});
        channel["modelOrder"] = json!(["stale-model"]);
        channel["defaults"] = json!({"contextWindow": {"mode": "custom", "value": 1234}});
        channel["overrides"] = json!({"stale-model": {"fast": {"mode": "custom", "value": true}}});
        channel["familyOverrides"] = channel["overrides"].clone();
        channel["naming"] =
            json!({"stale-model": {"alias": "stale-alias", "displayName": "Stale display"}});
        channel["discovered"] = json!([{"id": "stale-model", "label": "Stale model", "rules": {}}]);
        value["channels"] = json!([channel]);
        value["unknownSecret"] = json!("never-copy-this-key");
        value
    }
    fn has_archive(root: &Path, report: &MigrationReport, bytes: &[u8], quarantined: bool) -> bool {
        report.archives.iter().any(|relative| {
            relative.contains(if quarantined {
                "/quarantine/"
            } else {
                "/archive/"
            }) && std::fs::read(root.join(relative)).unwrap() == bytes
        })
    }
    fn assert_metadata(layout: &Policy) {
        assert_eq!(layout.channels.len(), 1);
        let channel = &layout.channels[0];
        assert_eq!(channel.id, CHANNEL);
        assert_eq!(channel.label, "My saved channel");
        assert_eq!(channel.connection.provider_id, "saved");
        assert_eq!(channel.connection.active_node_id, NODE);
        assert_eq!(channel.connection.nodes[1].id, BACKUP_NODE);
        assert_eq!(channel.connection.nodes[1].name, "Backup node");
        assert_eq!(
            channel.connection.nodes[1].url,
            "https://backup.example.test/v1"
        );
        assert_eq!(channel.template_refs, layout.template_refs);
        assert_eq!(channel.model_refs["exact/model"].revision_id, REVISION);
        assert_eq!(channel.selection, ModelSelection::None);
        assert_eq!(channel.defaults, ModelRules::default());
        assert!(channel.overrides.is_empty());
        assert!(channel.family_overrides.is_empty());
        assert!(channel.model_order.is_empty());
        assert!(channel.discovered.is_empty());
        assert!(channel.naming.is_empty());
        assert_eq!(channel.connection.protocol, "");
        assert!(!channel.connection.enabled);
        assert!(!channel.connection.allow_private_network);
        assert!(channel.connection.auth_mode.is_none());
        assert!(channel.connection.credential_ref.is_none());
        assert!(channel.connection.source_read_id.is_none());
        assert!(channel.connection.imported_config.is_none());
        assert!(layout.default_channel_id.is_none());
        assert!(layout.pending_removals.is_empty());
        assert!(layout.recovery_id.is_none());
        let encoded = serde_json::to_string(layout).unwrap();
        for excluded in [
            "never-copy-this-key",
            "sensitive-import",
            "stale-protocol",
            "stale-model",
            "stale-alias",
            CREDENTIAL,
        ] {
            assert!(
                !encoded.contains(excluded),
                "copied excluded field: {excluded}"
            );
        }
    }

    #[test]
    fn new_install_creates_only_empty_layout_and_repeats_without_rewriting() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(load_layout(root.path()).unwrap(), Policy::default());
        let before = std::fs::read(root.path().join(LAYOUT_PATH)).unwrap();
        let modified = std::fs::metadata(root.path().join(LAYOUT_PATH))
            .unwrap()
            .modified()
            .unwrap();
        let repeat = migrate(root.path()).unwrap();
        assert_eq!(repeat.version, MIGRATION_VERSION);
        assert!(!repeat.created_layout);
        assert!(repeat.archives.is_empty());
        assert!(repeat.removed.is_empty());
        assert_eq!(
            std::fs::read(root.path().join(LAYOUT_PATH)).unwrap(),
            before
        );
        assert_eq!(
            std::fs::metadata(root.path().join(LAYOUT_PATH))
                .unwrap()
                .modified()
                .unwrap(),
            modified
        );
        for absent in [
            POLICY_PATH,
            DRAFT_PATH,
            IDS_PATH,
            JOIN_OPERATIONS,
            JOIN_LOCK,
            "config.json",
        ] {
            assert!(!root.path().join(absent).exists(), "created {absent}");
        }
    }

    #[test]
    fn upgrade_salvages_metadata_and_archives_exact_unsanitized_bytes() {
        let root = tempfile::tempdir().unwrap();
        let old = put_json(root.path(), POLICY_PATH, &policy());
        let draft = b"unfinished candidate with secret references, not a current configuration\x00";
        put(root.path(), DRAFT_PATH, draft);
        let report = migrate(root.path()).unwrap();
        assert!(report.created_layout);
        assert_metadata(&report.layout);
        assert!(has_archive(root.path(), &report, &old, false));
        assert!(has_archive(root.path(), &report, draft, true));
        #[cfg(unix)]
        for relative in report
            .archives
            .iter()
            .map(String::as_str)
            .chain([LAYOUT_PATH])
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(root.path().join(relative))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
        assert_eq!(std::fs::read(root.path().join(POLICY_PATH)).unwrap(), old);
        assert_eq!(std::fs::read(root.path().join(DRAFT_PATH)).unwrap(), draft);
        let repeat = migrate(root.path()).unwrap();
        assert!(!repeat.created_layout);
        assert_eq!(repeat.layout, report.layout);
        assert_eq!(repeat.archives, report.archives);
        assert!(repeat.removed.is_empty());
        assert_eq!(load_layout(root.path()).unwrap(), report.layout);
    }

    #[test]
    fn policy_identity_wins_and_source_only_identity_survives_for_saved_agent_preferences() {
        let root = tempfile::tempdir().unwrap();
        put_json(root.path(), POLICY_PATH, &policy());
        let old_ids = put_json(
            root.path(),
            IDS_PATH,
            &json!({"saved": OPERATION, "other": SOURCE, "collision": CHANNEL, "invalid": "short-id"}),
        );
        let report = migrate(root.path()).unwrap();
        assert_eq!(report.layout.channels.len(), 2);
        assert_eq!(report.layout.channels[0].id, CHANNEL);
        let other = &report.layout.channels[1];
        assert_eq!(other.connection.provider_id, "other");
        assert_eq!(other.id, SOURCE);
        assert!(has_archive(root.path(), &report, &old_ids, false));
        let actual = config_file_mapping::read_policy(
            &json!({"providers": {"other": {"models": ["exact/model"]}}}),
            &report.layout,
        )
        .unwrap();
        assert_eq!(actual.channels.len(), 1);
        assert_eq!(actual.channels[0].id, SOURCE);
        assert_eq!(std::fs::read(root.path().join(IDS_PATH)).unwrap(), old_ids);
    }

    #[test]
    fn malformed_retired_facts_do_not_discard_valid_metadata() {
        let root = tempfile::tempdir().unwrap();
        let mut old = policy();
        old["channels"][0]["selection"] = json!(42);
        old["channels"][0]["connection"]["protocol"] = json!(["broken"]);
        old["channels"][0]["defaults"] = json!(false);
        put_json(root.path(), POLICY_PATH, &old);
        assert_metadata(&load_layout(root.path()).unwrap());
    }

    #[test]
    fn damaged_identity_and_sensitive_node_address_keep_valid_slot_and_adoption_metadata() {
        let root = tempfile::tempdir().unwrap();
        let mut old = policy();
        old["channels"][0]["id"] = json!("damaged-id");
        old["channels"][0]["connection"]["nodes"][0]["url"] = json!(
            "https://user:never-copy-this-key@old.example.test/v1?apiKey=never-copy-this-key"
        );
        let valid_ref = old["channels"][0]["templateRefs"][0].clone();
        old["channels"][0]["templateRefs"]
            .as_array_mut()
            .unwrap()
            .extend([
                valid_ref,
                json!({"templateId": "broken", "revisionId": REVISION}),
            ]);
        put_json(root.path(), POLICY_PATH, &old);
        put_json(root.path(), IDS_PATH, &json!({"saved": CHANNEL}));
        let report = migrate(root.path()).unwrap();
        assert_metadata(&report.layout);
        assert_eq!(report.layout.channels[0].connection.nodes[0].id, NODE);
        assert_eq!(
            report.layout.channels[0].connection.nodes[0].name,
            "Main node"
        );
        assert_eq!(report.layout.channels[0].connection.nodes[0].url, "");
        assert_eq!(report.layout.channels[0].template_refs.len(), 1);
        let actual = config_file_mapping::read_policy(
            &json!({"providers": {"saved": {"baseUrl": "https://current.example.test/v1"}}}),
            &report.layout,
        )
        .unwrap();
        assert_eq!(actual.channels[0].connection.active_node_id, NODE);
        assert_eq!(
            actual.channels[0].connection.nodes[0].url,
            "https://current.example.test/v1"
        );
    }

    #[test]
    fn current_layout_wins_and_ordinary_reads_never_reopen_legacy() {
        let root = tempfile::tempdir().unwrap();
        let mut current = policy();
        current["channels"][0]["id"] = json!(SOURCE);
        current["channels"][0]["label"] = json!("New layout label");
        let bytes = put_json(root.path(), LAYOUT_PATH, &current);
        put(root.path(), POLICY_PATH, b"broken obsolete policy");
        put_json(
            root.path(),
            IDS_PATH,
            &json!({"saved": CHANNEL, "must-not-add": OPERATION}),
        );
        let layout = load_layout(root.path()).unwrap();
        assert_eq!(layout.channels.len(), 1);
        assert_eq!(layout.channels[0].id, SOURCE);
        assert_eq!(layout.channels[0].label, "New layout label");
        assert!(!root.path().join(ARCHIVE_DIR).exists());
        let report = migrate(root.path()).unwrap();
        assert!(!report.created_layout);
        assert_eq!(report.layout, layout);
        assert_eq!(std::fs::read(root.path().join(LAYOUT_PATH)).unwrap(), bytes);
    }

    #[test]
    fn current_file_facts_and_exact_bytes_win_over_legacy_metadata() {
        let root = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        put_json(root.path(), POLICY_PATH, &policy());
        let config = json!({
            "providers": {"saved": {"baseUrl": "https://current.example.test/v2",
                "adapter": "responses", "apiKey": "current-fixture-key", "disabled": true,
                "models": ["exact/model", "manual-model"], "selectedModels": ["exact/model"],
                "modelContextWindows": {"exact/model": 32000}, "modelDisplayNames": {"exact/model": "Current display"}}},
            "modelPickerOrder": ["saved/manual-model", "saved/exact/model"],
            "defaultProvider": "saved", "unrecognized": {"preserve": true}
        });
        let bytes = put_json(home.path(), "config.json", &config);
        let report = migrate(root.path()).unwrap();
        assert_metadata(&report.layout);
        let actual = config_file::snapshot(root.path(), home.path()).unwrap();
        let channel = &actual.policy.channels[0];
        assert_eq!(channel.id, CHANNEL);
        assert_eq!(channel.connection.protocol, "responses");
        assert_eq!(channel.connection.active_node_id, NODE);
        assert_eq!(
            channel.connection.nodes[0].url,
            "https://current.example.test/v2"
        );
        assert!(!channel.connection.enabled);
        assert_eq!(channel.model_order, ["manual-model", "exact/model"]);
        assert_eq!(
            channel.selection,
            ModelSelection::Whitelist(vec!["exact/model".into()])
        );
        assert_eq!(
            channel.naming["exact/model"].display_name.as_deref(),
            Some("Current display")
        );
        assert_eq!(
            channel.overrides["exact/model"].context_window,
            Field::Custom(32000)
        );
        assert_eq!(actual.policy.default_channel_id.as_deref(), Some(CHANNEL));
        assert_eq!(
            std::fs::read(home.path().join("config.json")).unwrap(),
            bytes
        );
        assert!(!std::fs::read_to_string(root.path().join(LAYOUT_PATH))
            .unwrap()
            .contains("current-fixture-key"));
    }

    #[test]
    fn corrupt_legacy_documents_do_not_block_new_file_snapshot_or_save() {
        let root = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        put(
            root.path(),
            POLICY_PATH,
            b"{corrupt policy with fixture-secret",
        );
        put(root.path(), IDS_PATH, b"not JSON");
        put(root.path(), DRAFT_PATH, b"\xff\x00unfinished draft");
        let before = put_json(
            home.path(),
            "config.json",
            &json!({"providers": {"saved": {"models": ["unverified-exact-id"], "apiKey": "keep-fixture-key"}}}),
        );
        let report = migrate(root.path()).unwrap();
        assert_eq!(report.layout, Policy::default());
        assert_eq!(report.archives.len(), 3);
        assert!(report.archives.iter().all(|p| p.contains("/quarantine/")));
        assert!(report.diagnostics.iter().all(|d| d.level == "info"));
        assert!(!serde_json::to_string(&report.diagnostics)
            .unwrap()
            .contains("fixture-secret"));
        let snapshot = config_file::snapshot(root.path(), home.path()).unwrap();
        let result = config_file::save(
            root.path(),
            home.path(),
            config_file::SaveRequest {
                expected_file: snapshot.policy_digest,
                policy: snapshot.policy,
                credentials: vec![],
            },
        )
        .unwrap();
        assert!(!result.changed);
        assert_eq!(
            std::fs::read(home.path().join("config.json")).unwrap(),
            before
        );
        assert_eq!(
            std::fs::read(root.path().join(DRAFT_PATH)).unwrap(),
            b"\xff\x00unfinished draft"
        );
    }

    #[test]
    fn draft_candidate_is_archived_but_never_used_as_current_metadata() {
        let root = tempfile::tempdir().unwrap();
        let draft = PolicyDraft {
            policy: serde_json::from_value({
                let mut value = policy();
                value["channels"][0]["connection"]
                    .as_object_mut()
                    .unwrap()
                    .remove("apiKey");
                value.as_object_mut().unwrap().remove("unknownSecret");
                value
            })
            .unwrap(),
            base_policy: "saved-base".into(),
        };
        let bytes = serde_json::to_vec_pretty(&draft).unwrap();
        put(root.path(), DRAFT_PATH, &bytes);
        let report = migrate(root.path()).unwrap();
        assert_eq!(report.layout, Policy::default());
        assert!(has_archive(root.path(), &report, &bytes, false));
        assert_eq!(std::fs::read(root.path().join(DRAFT_PATH)).unwrap(), bytes);
    }

    #[test]
    fn interrupted_before_and_after_publication_is_repeatable_and_keeps_history_bytes() {
        let root = tempfile::tempdir().unwrap();
        let old = put_json(root.path(), POLICY_PATH, &policy());
        let archived = archive(root.path(), POLICY_PATH, &old, false).unwrap();
        put(
            root.path(),
            &format!("{STAGING_DIR}/{STAGING_FILE}"),
            b"interrupted incomplete staging",
        );
        let temporary = format!("{STAGING_DIR}/.layout.json.tmp-123-0-456");
        put(root.path(), &temporary, b"abandoned writer");
        let history_paths = [
            "manager-state/model-save-plans/history-providers-before.json",
            "manager-state/model-source-reads/history.json",
            "manager-state/model-recovery-plans/history.json",
            "manager-state/model-config-journal.json",
            "manager-state/model-config-draft.json.bak-previous",
            "manager-state/model-template-tasks/receipt.json",
            "manager-state/imported-provider-configs/authorization.json",
            "manager-state/model-file-incoming/pending.json",
            "backups/model-config/exact/config.json",
        ];
        for relative in history_paths {
            put(
                root.path(),
                relative,
                b"exact historical bytes with Key references\x00\xff",
            );
        }
        let first = migrate(root.path()).unwrap();
        assert_metadata(&first.layout);
        assert_eq!(first.archives, [archived]);
        assert!(!root.path().join(&temporary).exists());
        let layout_bytes = std::fs::read(root.path().join(LAYOUT_PATH)).unwrap();
        // Crash after link publication, before unlinking our staging name.
        let staging = root.path().join(format!("{STAGING_DIR}/{STAGING_FILE}"));
        std::fs::hard_link(root.path().join(LAYOUT_PATH), &staging).unwrap();
        let repeat = migrate(root.path()).unwrap();
        assert!(!repeat.created_layout);
        assert_eq!(repeat.archives, first.archives);
        assert!(!staging.exists());
        assert_eq!(
            std::fs::read(root.path().join(LAYOUT_PATH)).unwrap(),
            layout_bytes
        );
        for relative in history_paths {
            assert_eq!(
                std::fs::read(root.path().join(relative)).unwrap(),
                b"exact historical bytes with Key references\x00\xff"
            );
        }
    }

    #[test]
    fn archive_failure_preserves_source_and_still_creates_required_layout() {
        let root = tempfile::tempdir().unwrap();
        let old = put_json(root.path(), POLICY_PATH, &policy());
        put(
            root.path(),
            &format!("{ARCHIVE_DIR}/archive"),
            b"obstructing non-directory",
        );
        let report = migrate(root.path()).unwrap();
        assert_metadata(&report.layout);
        assert!(report.archives.is_empty());
        assert!(report
            .diagnostics
            .iter()
            .any(|d| d.code == "archive_deferred_original_retained"));
        assert_eq!(std::fs::read(root.path().join(POLICY_PATH)).unwrap(), old);
        std::fs::remove_file(root.path().join(format!("{ARCHIVE_DIR}/archive"))).unwrap();
        let repeat = migrate(root.path()).unwrap();
        assert!(!repeat.created_layout);
        assert!(has_archive(root.path(), &repeat, &old, false));
    }

    #[test]
    fn failed_quarantine_reports_retention_without_claiming_an_archive_exists() {
        let root = tempfile::tempdir().unwrap();
        put(root.path(), POLICY_PATH, b"broken old policy");
        put(
            root.path(),
            &format!("{ARCHIVE_DIR}/quarantine"),
            b"obstructing non-directory",
        );
        let report = migrate(root.path()).unwrap();
        assert!(report.created_layout);
        assert_eq!(report.layout, Policy::default());
        assert!(report.archives.is_empty());
        assert!(report
            .diagnostics
            .iter()
            .any(|d| d.code == "archive_deferred_original_retained"));
        assert!(!report
            .diagnostics
            .iter()
            .any(|d| d.code == "legacy_quarantined_original_retained"));
        assert_eq!(
            std::fs::read(root.path().join(POLICY_PATH)).unwrap(),
            b"broken old policy"
        );
    }

    #[test]
    fn cleanup_removes_only_dead_fixed_file_and_owned_unreferenced_temporary_names() {
        let root = tempfile::tempdir().unwrap();
        put(root.path(), RETIRED_PATH, b"retired");
        let owned = format!("{STAGING_DIR}/.layout.json.tmp-123-1-456");
        put(root.path(), &owned, b"orphan");
        let keep = [
            "manager-state/.model-channel-layout.json.tmp-123-1-456",
            "manager-state/.model-policy.json.tmp-123-1-456",
            "manager-state/model-config-transaction.lock",
            "manager-state/unknown.tmp",
            "manager-state/.model-layout-upgrade-v1/.layout.json.tmp-invalid",
            "manager-state/.model-layout-upgrade-v1/.layout.json.tmp-123-16-456",
            "manager-state/.model-layout-upgrade-v1/user-notes",
            "manager-state/.model-layout-upgrade-v1/subdir/keep",
        ];
        for relative in keep {
            put(root.path(), relative, b"keep exact bytes");
        }
        let report = migrate(root.path()).unwrap();
        assert!(report.removed.contains(&RETIRED_PATH.to_owned()));
        assert!(report.removed.contains(&owned));
        assert!(!root.path().join(RETIRED_PATH).exists());
        assert!(!root.path().join(&owned).exists());
        for relative in keep {
            assert_eq!(
                std::fs::read(root.path().join(relative)).unwrap(),
                b"keep exact bytes"
            );
        }
    }

    #[test]
    fn retired_join_candidates_and_unlocked_admission_are_archived_without_deleting_references() {
        let root = tempfile::tempdir().unwrap();
        let relative = format!("{JOIN_OPERATIONS}/{OPERATION}.json");
        let candidate = put_json(
            root.path(),
            &relative,
            &json!({"credentialRef": CREDENTIAL, "importedConfig": {"id": SOURCE}, "rawFixtureSecret": "retain-in-archive"}),
        );
        put(root.path(), JOIN_LOCK, b"old admission bytes");
        let reference = "manager-state/imported-provider-configs/reference.json";
        put(root.path(), reference, b"protected fixture authorization");
        let report = migrate(root.path()).unwrap();
        assert!(has_archive(root.path(), &report, &candidate, false));
        assert!(has_archive(
            root.path(),
            &report,
            b"old admission bytes",
            false
        ));
        assert_eq!(
            std::fs::read(root.path().join(&relative)).unwrap(),
            candidate
        );
        assert_eq!(
            std::fs::read(root.path().join(JOIN_LOCK)).unwrap(),
            b"old admission bytes"
        );
        assert_eq!(
            std::fs::read(root.path().join(reference)).unwrap(),
            b"protected fixture authorization"
        );
        assert_eq!(report.layout, Policy::default());
    }

    #[test]
    fn terminal_join_records_are_archived_retired_and_repeat_without_new_archives() {
        let root = tempfile::tempdir().unwrap();
        let records = [(OPERATION, "committed", false), (SOURCE, "failed", true)].map(
            |(id, phase, proof)| {
                let relative = format!("{JOIN_OPERATIONS}/{id}.json");
                let bytes = put_json(root.path(), &relative, &join_record(id, phase, proof));
                (relative, bytes)
            },
        );
        put(root.path(), JOIN_LOCK, b"original admission inode");
        #[cfg(unix)]
        let inode = {
            use std::os::unix::fs::MetadataExt;
            std::fs::metadata(root.path().join(JOIN_LOCK))
                .unwrap()
                .ino()
        };
        let protected = [
            "manager-state/imported-provider-configs/authorization.json",
            "manager-state/model-source-reads/credential-reference.json",
        ];
        for relative in protected {
            put(
                root.path(),
                relative,
                b"preserve credential/import references exactly",
            );
        }
        let report = migrate(root.path()).unwrap();
        for (relative, bytes) in &records {
            assert!(has_archive(root.path(), &report, bytes, false));
            assert!(report.removed.contains(relative));
            assert!(!root.path().join(relative).exists());
        }
        let archives: BTreeSet<_> = report.archives.iter().cloned().collect();
        let repeat = migrate(root.path()).unwrap();
        assert!(!repeat.created_layout);
        assert!(repeat.removed.is_empty());
        assert!(repeat.archives.iter().all(|p| archives.contains(p)));
        assert_eq!(
            std::fs::read_dir(root.path().join(format!("{ARCHIVE_DIR}/archive")))
                .unwrap()
                .count(),
            archives.len()
        );
        assert_eq!(
            std::fs::read(root.path().join(JOIN_LOCK)).unwrap(),
            b"original admission inode"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            assert_eq!(
                std::fs::metadata(root.path().join(JOIN_LOCK))
                    .unwrap()
                    .ino(),
                inode
            );
        }
        for relative in protected {
            assert_eq!(
                std::fs::read(root.path().join(relative)).unwrap(),
                b"preserve credential/import references exactly"
            );
        }
    }

    #[test]
    fn incomplete_conflicting_unknown_and_corrupt_join_records_keep_originals() {
        let root = tempfile::tempdir().unwrap();
        let states = [
            ("prepared", true),
            ("staging", false),
            ("staging", true),
            ("conflict", false),
            ("conflict", true),
            ("cleanup_pending", true),
            ("no_write_proven", true),
            ("future-phase", false),
        ];
        let mut originals = Vec::new();
        for (index, (phase, proof)) in states.into_iter().enumerate() {
            let id = format!("{index:08x}-9999-4999-8999-999999999999");
            let relative = format!("{JOIN_OPERATIONS}/{id}.json");
            let bytes = put_json(root.path(), &relative, &join_record(&id, phase, proof));
            originals.push((relative, bytes));
        }
        for (index, mut record) in [
            join_record(OPERATION, "committed", false),
            join_record(OPERATION, "failed", true),
            join_record(OPERATION, "committed", false),
            json!({"id": OPERATION, "phase": "committed"}),
        ]
        .into_iter()
        .enumerate()
        {
            let id = format!("{index:08x}-aaaa-4aaa-8aaa-aaaaaaaaaaaa");
            record["id"] = json!(id);
            match index {
                0 => record["channel"]["id"] = json!("broken-identity"),
                1 => record["futureField"] = json!(true),
                2 => record["id"] = json!(SOURCE),
                _ => {}
            }
            let relative = format!("{JOIN_OPERATIONS}/{id}.json");
            let bytes = put_json(root.path(), &relative, &record);
            originals.push((relative, bytes));
        }
        let corrupt = format!("{JOIN_OPERATIONS}/{OPERATION}.json");
        put(root.path(), &corrupt, b"{corrupt join record\x00\xff");
        originals.push((corrupt, b"{corrupt join record\x00\xff".to_vec()));
        let report = migrate(root.path()).unwrap();
        assert!(!report
            .removed
            .iter()
            .any(|p| p.starts_with(JOIN_OPERATIONS)));
        for (relative, bytes) in originals {
            assert_eq!(std::fs::read(root.path().join(relative)).unwrap(), bytes);
            assert!(has_archive(
                root.path(),
                &report,
                &bytes,
                serde_json::from_slice::<Value>(&bytes).is_err()
            ));
        }
    }

    #[test]
    fn empty_or_unrecognized_join_directory_never_creates_an_admission_lock() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join(JOIN_OPERATIONS)).unwrap();
        migrate(root.path()).unwrap();
        assert!(!root.path().join(JOIN_LOCK).exists());
        put(
            root.path(),
            &format!("{JOIN_OPERATIONS}/user-notes.json"),
            b"unknown user file",
        );
        migrate(root.path()).unwrap();
        assert!(!root.path().join(JOIN_LOCK).exists());
        // An actual retired record justifies creating the admission lock even
        // when an earlier partial installation left no lock file behind.
        let relative = format!("{JOIN_OPERATIONS}/{OPERATION}.json");
        let bytes = put_json(
            root.path(),
            &relative,
            &join_record(OPERATION, "committed", false),
        );
        let report = migrate(root.path()).unwrap();
        assert!(root.path().join(JOIN_LOCK).is_file());
        assert!(report.removed.contains(&relative));
        assert!(has_archive(root.path(), &report, &bytes, false));
    }

    #[test]
    fn terminal_join_record_is_retained_when_archive_cannot_be_verified() {
        let root = tempfile::tempdir().unwrap();
        let relative = format!("{JOIN_OPERATIONS}/{OPERATION}.json");
        let bytes = put_json(
            root.path(),
            &relative,
            &join_record(OPERATION, "committed", false),
        );
        put(
            root.path(),
            &format!("{ARCHIVE_DIR}/archive"),
            b"blocked archive",
        );
        let report = migrate(root.path()).unwrap();
        assert!(!report.removed.contains(&relative));
        assert_eq!(std::fs::read(root.path().join(&relative)).unwrap(), bytes);
        assert!(report
            .diagnostics
            .iter()
            .any(|d| d.code == "candidate_archive_deferred"));
        std::fs::remove_file(root.path().join(format!("{ARCHIVE_DIR}/archive"))).unwrap();
        let repeat = migrate(root.path()).unwrap();
        assert!(repeat.removed.contains(&relative));
        assert!(has_archive(root.path(), &repeat, &bytes, false));
    }

    #[test]
    fn changed_source_or_archive_prevents_terminal_record_unlink() {
        let root = tempfile::tempdir().unwrap();
        let relative = format!("{JOIN_OPERATIONS}/{OPERATION}.json");
        let bytes = put_json(
            root.path(),
            &relative,
            &join_record(OPERATION, "committed", false),
        );
        let _guard = TargetFileLock::lock(&root.path().join(JOIN_ADMISSION)).unwrap();
        let archived = archive(root.path(), &relative, &bytes, false).unwrap();
        let later = put_json(
            root.path(),
            &relative,
            &join_record(OPERATION, "conflict", false),
        );
        assert!(!retire_archived_join(root.path(), &relative, &archived, &bytes).unwrap());
        assert_eq!(std::fs::read(root.path().join(&relative)).unwrap(), later);
        put(root.path(), &relative, &bytes);
        put(
            root.path(),
            &archived,
            b"archive changed after initial verification",
        );
        assert!(retire_archived_join(root.path(), &relative, &archived, &bytes).is_err());
        assert_eq!(std::fs::read(root.path().join(&relative)).unwrap(), bytes);
    }

    #[test]
    fn busy_retired_admission_lock_is_never_replaced_removed_or_archived() {
        let root = tempfile::tempdir().unwrap();
        let relative = format!("{JOIN_OPERATIONS}/{OPERATION}.json");
        let bytes = put_json(
            root.path(),
            &relative,
            &join_record(OPERATION, "committed", false),
        );
        put(root.path(), JOIN_LOCK, b"held lock bytes");
        let guard = TargetFileLock::lock(&root.path().join(JOIN_ADMISSION)).unwrap();
        let report = migrate(root.path()).unwrap();
        assert!(report.created_layout);
        assert!(report
            .diagnostics
            .iter()
            .any(|d| d.code == "admission_archive_deferred_lock_retained"));
        assert!(!report.archives.iter().any(|p| p.contains("admission")));
        assert!(!report.removed.contains(&relative));
        assert!(!has_archive(root.path(), &report, &bytes, false));
        assert_eq!(std::fs::read(root.path().join(&relative)).unwrap(), bytes);
        assert_eq!(
            guard.read_metadata(MAX_DOCUMENT).unwrap(),
            b"held lock bytes"
        );
        assert!(TargetFileLock::try_lock_with_timeout(
            &root.path().join(JOIN_ADMISSION),
            Duration::ZERO
        )
        .is_err());
        drop(guard);
        let repeat = migrate(root.path()).unwrap();
        assert!(repeat.removed.contains(&relative));
        assert!(has_archive(root.path(), &repeat, &bytes, false));
        assert!(has_archive(root.path(), &repeat, b"held lock bytes", false));
    }

    #[test]
    fn concurrent_upgrade_publishes_once_and_never_duplicates_metadata_or_archives() {
        let root = tempfile::tempdir().unwrap();
        put_json(root.path(), POLICY_PATH, &policy());
        let reports = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..4)
                .map(|_| scope.spawn(|| migrate(root.path()).unwrap()))
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().unwrap())
                .collect::<Vec<_>>()
        });
        assert_eq!(reports.iter().filter(|r| r.created_layout).count(), 1);
        for report in &reports {
            assert_metadata(&report.layout);
            assert_eq!(report.archives, reports[0].archives);
        }
        assert_eq!(
            std::fs::read_dir(root.path().join(format!("{ARCHIVE_DIR}/archive")))
                .unwrap()
                .count(),
            1
        );
    }

    #[cfg(unix)]
    #[test]
    fn links_and_directories_are_preserved_without_reading_or_removing_outside_files() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        put(outside.path(), "sensitive.json", b"outside fixture bytes");
        std::fs::create_dir_all(root.path().join("manager-state")).unwrap();
        for relative in [POLICY_PATH, RETIRED_PATH] {
            std::os::unix::fs::symlink(
                outside.path().join("sensitive.json"),
                root.path().join(relative),
            )
            .unwrap();
        }
        std::fs::create_dir_all(root.path().join(DRAFT_PATH)).unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join(JOIN_OPERATIONS)).unwrap();
        let report = migrate(root.path()).unwrap();
        assert_eq!(report.layout, Policy::default());
        assert!(report.archives.is_empty());
        assert!(std::fs::symlink_metadata(root.path().join(RETIRED_PATH))
            .unwrap()
            .file_type()
            .is_symlink());
        assert!(root.path().join(DRAFT_PATH).is_dir());
        assert_eq!(
            std::fs::read(outside.path().join("sensitive.json")).unwrap(),
            b"outside fixture bytes"
        );
    }
}
