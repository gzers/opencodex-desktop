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
#[cfg(test)]
#[path = "native_config_import_v2_tests.rs"]
mod tests;

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
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
    pub backup_id: Option<String>,
    pub port: u16,
    pub changed: bool,
    pub external_auth: bool,
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

fn pending_admission(root: &Path) -> Result<Vec<TargetFileLock>> {
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
    let raw = candidate(&bytes, port.port.unwrap())?;
    let projected = Zeroizing::new(canonical_bytes(&raw)?);
    projected_policy(&raw, &digest(&projected))?;
    drop(locked);
    drop(_config);
    drop(admission);
    validate(&bytes)
        .and_then(|_| validate(&projected))
        .map_err(|_| error("native_import_config_invalid"))?;
    Ok(Selection {
        requested_path: path.to_path_buf(),
        source,
        source_digest: digest(&bytes),
        candidate_digest: digest(&projected),
        root_revision: revision,
        port_revision: port.revision,
        store_digests: state.digests,
    })
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
    let (source, bytes) =
        source_bytes(&selection.requested_path).map_err(|_| error("native_import_stale"))?;
    let store = StoreV2::new(root);
    let locked = store.lock()?;
    let state = locked.snapshot_unlocked()?;
    check_draft(&state)?;
    if source != selection.source
        || digest(&bytes) != selection.source_digest
        || current_port.revision != selection.port_revision
        || root_revision(root, home)? != selection.root_revision
        || state.digests != selection.store_digests
    {
        return Err(error("native_import_stale"));
    }
    let local_port = current_port.port.unwrap();
    let raw = candidate(&bytes, local_port)?;
    let after = canonical_bytes(&raw)?;
    if digest(&after) != selection.candidate_digest {
        return Err(error("native_import_stale"));
    }
    let (current, before) = read_raw(home)?;
    let has_external_auth = external_auth(&raw);
    if current == raw {
        return Ok(Outcome {
            backup_id: None,
            port: local_port,
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
        Some("OpenCodex 原生配置导入前完整备份".into()),
    )
    .map_err(|_| error("native_import_backup_failed"))?;
    if !backup::verify_backup(&record).map_err(|_| error("native_import_backup_failed"))? {
        return Err(error("native_import_backup_failed"));
    }
    require_stopped().map_err(|_| error("native_import_runtime_not_stopped"))?;
    let (resolved, rechecked) =
        source_bytes(&selection.requested_path).map_err(|_| error("native_import_stale"))?;
    if resolved != selection.source
        || digest(&rechecked) != selection.source_digest
        || port_snapshot(root, home)?.revision != selection.port_revision
    {
        return Err(error("native_import_stale"));
    }
    writes.insert(
        0,
        WriteV2 {
            target: TargetV2::OfficialConfig,
            before: Some(before.to_vec()),
            after: Some(after),
        },
    );
    model_transaction_v2::commit(root, home, writes)?;
    Ok(Outcome {
        backup_id: Some(record.manifest.backup_id),
        port: local_port,
        changed: true,
        external_auth: has_external_auth,
    })
}
