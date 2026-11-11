//! Native config.json import. Desktop metadata is rebuilt from the imported
//! configuration without applying templates or copying external credentials.
use super::*;
use crate::{
    errors::AppResult,
    modules::{
        backup::{self, BackupAction},
        runtime::port,
    },
};
use std::io::Read;
#[cfg(test)]
#[path = "native_config_import_v2_tests.rs"]
mod tests;

const IMPORT_BACKUP_NOTE: &str = "OpenCodex 原生配置导入前完整备份";
const ROLLBACK_BACKUP_NOTE: &str = "OpenCodex 原生配置回退前完整备份";

/// Server-owned selection. No source bytes, credentials or write path travel
/// through IPC. A controller may issue an opaque, single-use selection ID.
pub struct Selection {
    requested_path: std::path::PathBuf,
    source: std::path::PathBuf,
    source_digest: String,
    candidate_digest: String,
    root_revision: String,
    port_revision: String,
    store_digests: store_v2::StoreDigestsV2,
    rollback: Option<backup::BackupRecord>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
    pub backup_id: Option<String>,
    pub port: u16,
    pub changed: bool,
    pub external_auth: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSummary {
    pub id: String,
    pub created_at: String,
    pub bytes: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RollbackPreview {
    pub current_port: u16,
    pub backup_port: u16,
    pub target_path: std::path::PathBuf,
}

impl Selection {
    pub fn is_rollback(&self) -> bool {
        self.rollback.is_some()
    }
}

fn native_backup(record: &backup::BackupRecord, home: &Path) -> bool {
    let m = &record.manifest;
    m.action == BackupAction::Import
        && m.restorable
        && m.management.is_none()
        && matches!(
            m.note.as_deref(),
            Some(IMPORT_BACKUP_NOTE | ROLLBACK_BACKUP_NOTE)
        )
        && Path::new(&m.target_path) == home.join("config.json")
}

pub fn list_backups(root: &Path, home: &Path) -> Result<Vec<BackupSummary>> {
    let mut rows = safety::scan(&root.join("backups"))
        .map_err(|_| error("native_rollback_backup_invalid"))?
        .into_iter()
        .filter(|record| native_backup(record, home))
        .map(|record| BackupSummary {
            id: record.manifest.backup_id,
            created_at: record.manifest.created_at,
            bytes: record.manifest.bytes,
        })
        .collect::<Vec<_>>();
    rows.sort_unstable_by(|a, b| {
        b.created_at
            .cmp(&a.created_at)
            .then_with(|| b.id.cmp(&a.id))
    });
    Ok(rows)
}

fn backup_source(
    root: &Path,
    home: &Path,
    id: &str,
) -> Result<(backup::BackupRecord, std::path::PathBuf, Zeroizing<Vec<u8>>)> {
    let invalid = || error("native_rollback_backup_invalid");
    let (stamp, suffix) = id
        .strip_prefix("bk_")
        .and_then(|v| v.split_once('_'))
        .ok_or_else(invalid)?;
    if stamp.len() != 14
        || !stamp.bytes().all(|b| b.is_ascii_digit())
        || suffix.len() != 8
        || !suffix.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(invalid());
    }
    chrono::NaiveDateTime::parse_from_str(stamp, "%Y%m%d%H%M%S").map_err(|_| invalid())?;
    let dir = root
        .join("backups")
        .join(&stamp[..4])
        .join(&stamp[4..6])
        .join(BackupAction::Import.as_str())
        .join(id);
    safety::check_path(root, &dir, false).map_err(|_| invalid())?;
    let record = safety::load_record(&dir).map_err(|_| invalid())?;
    if !native_backup(&record, home) || record.manifest.bytes > safety::MAX_PAYLOAD_BYTES {
        return Err(invalid());
    }
    let path = safety::payload_path(&dir).map_err(|_| invalid())?;
    if path.file_name().is_none_or(|name| name != "config.json") {
        return Err(invalid());
    }
    let file = safety::open_file(&path, false).map_err(|_| invalid())?;
    safety::require_single_link(&file).map_err(|_| invalid())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let metadata = file.metadata().map_err(|_| invalid())?;
        // SAFETY: geteuid has no arguments and returns the current process identity.
        if metadata.uid() != unsafe { libc::geteuid() } || metadata.mode() & 0o077 != 0 {
            return Err(invalid());
        }
    }
    #[cfg(windows)]
    crate::infrastructure::windows_private_create::verify_private_handle(&file, false)
        .map_err(|_| invalid())?;
    let mut bytes = Zeroizing::new(Vec::new());
    file.take(safety::MAX_PAYLOAD_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| invalid())?;
    if bytes.len() as u64 != record.manifest.bytes
        || crate::infrastructure::hash::sha256_hex(&bytes) != record.manifest.sha256
    {
        return Err(invalid());
    }
    Ok((record, path, bytes))
}

fn source_bytes(path: &Path) -> Result<(std::path::PathBuf, Zeroizing<Vec<u8>>)> {
    if !path.is_absolute()
        || path
            .extension()
            .and_then(|v| v.to_str())
            .is_none_or(|v| !v.eq_ignore_ascii_case("json"))
        || path.components().any(|part| {
            matches!(
                part,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
    {
        return Err(error("native_import_source_invalid"));
    }
    safety::inspect(path).map_err(|_| error("native_import_source_invalid"))?;
    let resolved = path
        .canonicalize()
        .map_err(|_| error("native_import_source_invalid"))?;
    let parent = resolved
        .parent()
        .ok_or_else(|| error("native_import_source_invalid"))?;
    safety::check_path(parent, &resolved, false)
        .map_err(|_| error("native_import_source_invalid"))?;
    let bytes = Zeroizing::new(
        safety::read_file(&resolved, safety::MAX_PAYLOAD_BYTES)
            .map_err(|_| error("native_import_source_invalid"))?,
    );
    Ok((resolved, bytes))
}

fn port_snapshot(root: &Path, home: &Path) -> Result<port::Snapshot> {
    let snapshot = port::load(root, home).map_err(|_| error("native_import_port_required"))?;
    if snapshot.reconciliation_required || snapshot.port.is_none() {
        return Err(error("native_import_port_required"));
    }
    Ok(snapshot)
}

fn candidate(bytes: &[u8], local_port: u16) -> Result<Value> {
    let mut raw = parse_strict_json_v2(bytes).map_err(|_| error("native_import_config_invalid"))?;
    let map = raw
        .as_object_mut()
        .ok_or_else(|| error("native_import_config_invalid"))?;
    map.insert("port".into(), json!(local_port));
    Ok(raw)
}

fn effective_payload(
    bytes: &[u8],
    local_port: u16,
    rollback: bool,
) -> Result<(Value, Zeroizing<Vec<u8>>, u16)> {
    if rollback {
        let raw = parse_strict_json_v2(bytes).map_err(|_| error("native_import_config_invalid"))?;
        let port = raw
            .get("port")
            .and_then(Value::as_u64)
            .and_then(|n| u16::try_from(n).ok())
            .filter(|p| *p != 0)
            .ok_or_else(|| error("native_rollback_backup_invalid"))?;
        return Ok((raw, Zeroizing::new(bytes.to_vec()), port));
    }
    let raw = candidate(bytes, local_port)?;
    let bytes = Zeroizing::new(canonical_bytes(&raw)?);
    Ok((raw, bytes, local_port))
}

fn projected_policy(raw: &Value, source: &str) -> Result<PolicyV2> {
    let mappings = MappingFileV2::default();
    let providers = raw
        .get("providers")
        .map(|v| {
            v.as_object()
                .ok_or_else(|| error("native_import_config_invalid"))
        })
        .transpose()?;
    let bindings = providers
        .into_iter()
        .flat_map(|v| v.keys())
        .map(|provider_id| ProviderBindingV2 {
            channel_id: uuid::Uuid::new_v4().to_string(),
            provider_id: provider_id.clone(),
        })
        .collect::<Vec<_>>();
    provider_import::read_policy_v2(
        raw,
        source,
        &store_v2::empty_policy_v2(),
        None,
        &mappings,
        &bindings,
    )
    .map_err(|_| error("native_import_config_invalid"))
}

pub(super) fn pending_admission(root: &Path) -> Result<Vec<TargetFileLock>> {
    let names = [
        "saved-file-save-pending.json",
        "provider-rename-pending.json",
        "model-transaction-v2.pending.json",
        "credential-references-v2.pending.json",
    ];
    let mut locks = Vec::new();
    for name in names {
        let path = root.join("manager-state").join(name);
        safety::check_path(root, &path, true).map_err(io)?;
        safety::check_path(
            root,
            &crate::infrastructure::locking::lock_path_for(&path),
            true,
        )
        .map_err(io)?;
        locks.push(TargetFileLock::lock(&path).map_err(io)?);
    }
    // Selection only observes. It never recovers another business transaction.
    for name in names {
        let path = root.join("manager-state").join(name);
        if !matches!(std::fs::symlink_metadata(path), Err(e) if e.kind() == std::io::ErrorKind::NotFound)
        {
            return Err(error("model_transaction_pending"));
        }
    }
    Ok(locks)
}

fn check_draft(state: &StoreSnapshotV2) -> Result<()> {
    if state.draft.as_ref().is_some_and(|d| {
        d.policy.recovery_id.is_some()
            || !d.credential_edits.is_empty()
            || d.observed_baseline.as_ref() != Some(&d.policy)
    }) {
        return Err(error("native_import_draft_active"));
    }
    Ok(())
}

/// Validate the original document and effective candidate on the selected
/// official runtime. Validation is a read-only capability of the caller.
pub fn prepare(
    root: &Path,
    home: &Path,
    path: &Path,
    validate: impl Fn(&[u8]) -> AppResult<()>,
) -> Result<Selection> {
    let (source, bytes) = source_bytes(path)?;
    prepare_source(root, home, path, source, &bytes, None, validate).map(|(selection, _)| selection)
}

pub fn prepare_rollback(
    root: &Path,
    home: &Path,
    id: &str,
    validate: impl Fn(&[u8]) -> AppResult<()>,
) -> Result<(Selection, RollbackPreview)> {
    let (record, path, bytes) = backup_source(root, home, id)?;
    prepare_source(
        root,
        home,
        &path,
        path.clone(),
        &bytes,
        Some(record),
        validate,
    )
}

fn prepare_source(
    root: &Path,
    home: &Path,
    path: &Path,
    source: std::path::PathBuf,
    bytes: &[u8],
    rollback: Option<backup::BackupRecord>,
    validate: impl Fn(&[u8]) -> AppResult<()>,
) -> Result<(Selection, RollbackPreview)> {
    let revision = root_revision(root, home)?;
    if source == home.canonicalize().map_err(io)?.join("config.json") {
        return Err(error("native_import_source_is_target"));
    }
    let admission = pending_admission(root)?;
    safety::check_path(home, &home.join("config.json.lock"), true).map_err(io)?;
    let _config = TargetFileLock::lock(&home.join("config.json")).map_err(io)?;
    let port = port_snapshot(root, home)?;
    let store = StoreV2::new(root);
    let locked = store.lock()?;
    let state = locked.snapshot_unlocked()?;
    check_draft(&state)?;
    let current_port = port.port.unwrap();
    let (raw, projected, backup_port) = effective_payload(bytes, current_port, rollback.is_some())?;
    projected_policy(&raw, &digest(&projected))?;
    drop(locked);
    drop(_config);
    drop(admission);
    validate(bytes).map_err(|_| error("native_import_config_invalid"))?;
    if projected.as_slice() != bytes {
        validate(&projected).map_err(|_| error("native_import_config_invalid"))?;
    }
    Ok((
        Selection {
            requested_path: path.to_path_buf(),
            source,
            source_digest: digest(bytes),
            candidate_digest: digest(&projected),
            root_revision: revision,
            port_revision: port.revision,
            store_digests: state.digests,
            rollback,
        },
        RollbackPreview {
            current_port,
            backup_port,
            target_path: home.join("config.json"),
        },
    ))
}

fn recheck_source(root: &Path, home: &Path, selection: &Selection) -> Result<Zeroizing<Vec<u8>>> {
    let stale = || error("native_import_stale");
    let (resolved, bytes) = if let Some(expected) = &selection.rollback {
        let (record, resolved, bytes) =
            backup_source(root, home, &expected.manifest.backup_id).map_err(|_| stale())?;
        if &record != expected {
            return Err(stale());
        }
        (resolved, bytes)
    } else {
        source_bytes(&selection.requested_path).map_err(|_| stale())?
    };
    if resolved != selection.source || digest(&bytes) != selection.source_digest {
        return Err(stale());
    }
    Ok(bytes)
}

fn external_auth(raw: &Value) -> bool {
    fn references(value: &Value) -> bool {
        match value {
            Value::String(s) => s.starts_with("keychain:") || s.starts_with('$'),
            Value::Object(m) => m.values().any(references),
            Value::Array(a) => a.iter().any(references),
            _ => false,
        }
    }
    raw["providers"].as_object().is_some_and(|p| {
        p.values().any(|v| {
            matches!(v["authMode"].as_str(), Some("oauth" | "forward"))
                || ["apiKey", "apiKeyPool", "headers"]
                    .iter()
                    .any(|field| references(&v[field]))
        })
    })
}

/// Caller holds the lifecycle reservation. Recheck a proved stopped service
/// before committing; an unknown owner must never be stopped as a side effect.
pub fn apply(
    root: &Path,
    home: &Path,
    selection: Selection,
    require_stopped: impl Fn() -> AppResult<()>,
) -> Result<Outcome> {
    use crate::modules::model_config::model_transaction_v2::{self, TargetV2, WriteV2};
    let _admission = pending_admission(root)?;
    let _official_lock = port::ConfigMutation::acquire(home).map_err(io)?;
    safety::check_path(home, &home.join("config.json.lock"), true).map_err(io)?;
    let _config = TargetFileLock::lock(&home.join("config.json")).map_err(io)?;
    let current_port = port_snapshot(root, home)?;
    let bytes = recheck_source(root, home, &selection)?;
    let store = StoreV2::new(root);
    let locked = store.lock()?;
    let state = locked.snapshot_unlocked()?;
    check_draft(&state)?;
    if current_port.revision != selection.port_revision
        || root_revision(root, home)? != selection.root_revision
        || state.digests != selection.store_digests
    {
        return Err(error("native_import_stale"));
    }
    let rollback = selection.rollback.is_some();
    let (raw, after, result_port) =
        effective_payload(&bytes, current_port.port.unwrap(), rollback)?;
    if digest(&after) != selection.candidate_digest {
        return Err(error("native_import_stale"));
    }
    let (current, before) = read_raw(home)?;
    let has_external_auth = external_auth(&raw);
    if if rollback {
        before
            .as_ref()
            .is_some_and(|value| value.as_slice() == after.as_slice())
    } else {
        current == raw
    } {
        return Ok(Outcome {
            backup_id: None,
            port: result_port,
            changed: false,
            external_auth: has_external_auth,
        });
    }
    let before = before.ok_or_else(|| error("native_import_stale"))?;
    if before.len() as u64 > safety::MAX_PAYLOAD_BYTES {
        return Err(error("native_import_backup_failed"));
    }
    let policy = projected_policy(&raw, &digest(&after))?;
    // Record observed pointers from the imported document. Running the save
    // compiler here would regroup providers and rewrite the user's native JSON.
    let mut targets = Vec::new();
    for row in effective(&state.library, &policy)? {
        let channel = policy
            .channels
            .iter()
            .find(|c| c.id == row.channel_id)
            .ok_or_else(|| error("native_import_config_invalid"))?;
        targets.extend(compiler_v2::targets_for(
            &row,
            &channel.connection.provider_id,
        ));
    }
    let ledger = finalize_materialization_ledger_v2(&policy, &raw, &targets)?;
    let mappings = MappingFileV2::default();
    let mut writes = locked
        .prepare_save_unlocked(&state.digests, &policy, &ledger, &mappings)?
        .into_iter()
        .map(|w| WriteV2 {
            target: TargetV2::Manager(w.relative_path),
            before: w.before,
            after: Some(w.after),
        })
        .collect::<Vec<_>>();
    if state.draft.is_some() {
        let path = root.join(store_v2::DRAFT_PATH);
        let before = safety::read_file(&path, 32 * 1024 * 1024).map_err(io)?;
        if digest(&before) != state.digests.files[store_v2::DRAFT_PATH] {
            return Err(error("native_import_stale"));
        }
        writes.push(WriteV2 {
            target: TargetV2::Manager(store_v2::DRAFT_PATH.into()),
            before: Some(before),
            after: None,
        });
    }
    require_stopped().map_err(|_| error("native_import_runtime_not_stopped"))?;
    let record = backup::backup_file(
        root,
        BackupAction::Import,
        &home.join("config.json"),
        &before,
        chrono::Utc::now(),
        Some(
            if rollback {
                ROLLBACK_BACKUP_NOTE
            } else {
                IMPORT_BACKUP_NOTE
            }
            .into(),
        ),
    )
    .map_err(|_| error("native_import_backup_failed"))?;
    let (verified, _, protected) = backup_source(root, home, &record.manifest.backup_id)
        .map_err(|_| error("native_import_backup_failed"))?;
    if verified != record || protected.as_slice() != before.as_slice() {
        return Err(error("native_import_backup_failed"));
    }
    require_stopped().map_err(|_| error("native_import_runtime_not_stopped"))?;
    recheck_source(root, home, &selection)?;
    if port_snapshot(root, home)?.revision != selection.port_revision {
        return Err(error("native_import_stale"));
    }
    writes.insert(
        0,
        WriteV2 {
            target: TargetV2::OfficialConfig,
            before: Some(before.to_vec()),
            after: Some(after.to_vec()),
        },
    );
    model_transaction_v2::commit(root, home, writes)?;
    Ok(Outcome {
        backup_id: Some(record.manifest.backup_id),
        port: result_port,
        changed: true,
        external_auth: has_external_auth,
    })
}
