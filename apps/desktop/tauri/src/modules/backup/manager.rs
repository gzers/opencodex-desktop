//! W2 preferences-only backup management.
//!
//! Reliable protection model: standalone manual and reconciled restore records may expire.
//! All legacy transaction records, unknown metadata, restore guards and their source
//! references remain protected. Free-form notes are never used as transaction links.
//! Guards are durable before restore validation. Only a verified disk commit releases
//! new restore guards; legacy, failed and interrupted transactions remain protected.
//! W2 operations serialize across processes; legacy creators only produce protected
//! records. Persisted cleanup defaults to manual; no startup cleanup hook exists.
//! Manual/upgrade/protection entry points may trigger cleanup only after a verified
//! backup. Protection links are durable first; cleanup errors never invalidate it.
use super::{
    policy::{self, CleanupMode, CleanupPolicy},
    safety::{self, fail},
    BackupAction, BackupRecord, MAX_PER_ACTION,
};
use crate::{
    errors::AppResult,
    infrastructure::hash::sha256_hex,
    modules::preferences::{self, PREFERENCES_RELATIVE_PATH},
    types::upgrade::*,
};
use chrono::{DateTime, Duration, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs::File, path::Path};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct Management {
    pub version: u32,
    pub kind: String,
    pub pinned: bool,
    pub related_backup_id: Option<String>,
    /// v2 restore / v3 preferences reconciliation, independent of the user pin. Missing/unknown
    /// states fail closed; v1 transaction records are never retrospectively released.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_state: Option<String>,
}

/// Called under the transaction lock with captured bytes or an exact cleanup
/// inventory. Admission failures have no lock or readable source and use the
/// unavailable sentinel. Implementations must never re-enter backup APIs.
pub trait MaintenanceObserver {
    fn backup_result(&mut self, _action: BackupAction, _payload: &[u8], _succeeded: bool) {}
    fn cleanup_result(&mut self, _preview: Option<&BackupCleanupPreviewDto>, _succeeded: bool) {}
}
impl MaintenanceObserver for () {}
const SOURCE_UNAVAILABLE: &[u8] = b"preferences-source-unavailable";

fn observed_lock(
    root: &Path,
    action: BackupAction,
    observer: &mut dyn MaintenanceObserver,
) -> AppResult<File> {
    lock(root).inspect_err(|_| observer.backup_result(action, SOURCE_UNAVAILABLE, false))
}

fn lock(root: &Path) -> AppResult<File> {
    let path = root.join(".backup-w2.lock");
    safety::check_path(root, &path, true)?;
    let file = safety::open_file(&path, true)?;
    file.lock().map_err(|e| fail(e.to_string()))?;
    Ok(file)
}
/// Cooperative cross-process guard. Keep alive through the entire mutation.
/// Uses the same lock as W2 restore, pin and cleanup. Lock release never deletes backups.
pub struct PreferencesProtectionGuard {
    _lock: File,
    pub backup: Option<UpgradeBackupResult>,
    /// Best-effort maintenance failure after a successful protection backup.
    /// The backup and mutation lock remain valid. Also emitted to diagnostics.
    pub cleanup_error: Option<String>,
}

/// Exact durable backup association, never a free-form path or note. The payload
/// digest prevents a stale handoff from releasing a replaced protection record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferencesProtectionRef {
    pub backup_id: String,
    pub sha256: String,
}
impl PreferencesProtectionRef {
    pub fn validate(&self) -> AppResult<()> {
        if self.backup_id.is_empty()
            || self.backup_id.len() > 128
            || !self
                .backup_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
            || self.sha256.len() != 64
            || !self
                .sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(fail("invalid preferences protection reference"));
        }
        Ok(())
    }
}
impl PreferencesProtectionGuard {
    /// Exact panel association, with this guard already holding the shared W2 lock.
    /// Verification must be durably recorded before the protection can expire.
    pub(crate) fn reconcile_panel_recorded(
        &self,
        root: &Path,
        reference: &PreferencesProtectionRef,
        previously_verified: bool,
        persist_verification: impl FnOnce() -> AppResult<()>,
    ) -> AppResult<()> {
        reference.validate()?;
        if previously_verified
            && !records(root)?
                .iter()
                .any(|r| r.manifest.backup_id == reference.backup_id)
        {
            return persist_verification();
        }
        let record = resolve(root, &reference.backup_id)?;
        validate_preferences_protection(&record)?;
        if record.manifest.sha256 != reference.sha256
            || !matches!(
                record
                    .manifest
                    .management
                    .as_ref()
                    .unwrap()
                    .transaction_state
                    .as_deref(),
                Some("active" | "committed")
            )
        {
            return Err(fail("panel protection association mismatch"));
        }
        persist_verification()?;
        transition_preferences_protection(root, reference, "active", "committed")
    }
    /// Keep the guard alive: this method already owns the backup lock.
    pub fn protection_ref(&self, root: &Path) -> AppResult<Option<PreferencesProtectionRef>> {
        self.backup
            .as_ref()
            .map(|b| {
                let record = resolve(root, &b.backup_id)?;
                validate_preferences_protection(&record)?;
                Ok(PreferencesProtectionRef {
                    backup_id: b.backup_id.clone(),
                    sha256: record.manifest.sha256,
                })
            })
            .transpose()
    }
    /// Called only after a verified runtime installation, never on Drop / error.
    pub fn commit_verified(&self, root: &Path) -> AppResult<()> {
        if let Some(reference) = self.protection_ref(root)? {
            transition_preferences_protection(root, &reference, "active", "committed")?;
        }
        Ok(())
    }
    /// Persist before OS handoff. A crash retains this state until startup proves
    /// the exact candidate is running. Holding a handle is not completion evidence.
    pub fn await_restart(&self, root: &Path) -> AppResult<Option<PreferencesProtectionRef>> {
        let reference = self.protection_ref(root)?;
        if let Some(reference) = &reference {
            transition_preferences_protection(root, reference, "active", "awaiting_restart")?;
        }
        Ok(reference)
    }
}
fn validate_preferences_protection(record: &BackupRecord) -> AppResult<()> {
    let meta = record
        .manifest
        .management
        .as_ref()
        .ok_or_else(|| fail("missing protection metadata"))?;
    if !candidate(record)
        || record.manifest.action != BackupAction::PreferencesProtection
        || meta.version != 3
        || meta.kind != "preferences-protection"
        || meta.related_backup_id.is_some()
        || !matches!(
            meta.transaction_state.as_deref(),
            Some("active" | "awaiting_restart" | "committed")
        )
        || !super::verify_backup(record)?
    {
        return Err(fail("preferences protection failed verification"));
    }
    Ok(())
}
fn transition_preferences_protection(
    root: &Path,
    reference: &PreferencesProtectionRef,
    from: &str,
    to: &str,
) -> AppResult<()> {
    reference.validate()?;
    let record = resolve(root, &reference.backup_id)?;
    validate_preferences_protection(&record)?;
    if record.manifest.sha256 != reference.sha256 {
        return Err(fail("preferences protection reference changed"));
    }
    let state = record
        .manifest
        .management
        .as_ref()
        .unwrap()
        .transaction_state
        .as_deref();
    if state == Some(to) {
        return Ok(());
    } // Retrying an already verified commit is safe.
    if state != Some(from) {
        return Err(fail("preferences protection state mismatch"));
    }
    let mut committed = record.manifest.clone();
    committed.management.as_mut().unwrap().transaction_state = Some(to.into());
    safety::check_path(root, &record.directory, false)?;
    super::write_manifest(&record.directory, &committed)?;
    safety::sync_directories(root, &record.directory)?;
    let stored = resolve(root, &reference.backup_id)?;
    if stored.manifest != committed {
        return Err(fail("preferences protection readback mismatch"));
    }
    validate_preferences_protection(&stored)
}
/// Caller must first verify the running manager version against its signed
/// handoff. This function cannot infer success from a pending-restart record.
pub(crate) fn reconcile_preferences_restart_recorded(
    root: &Path,
    reference: &PreferencesProtectionRef,
    previously_verified: bool,
    persist_verification: impl FnOnce() -> AppResult<()>,
) -> AppResult<()> {
    let _lock = lock(root)?;
    reference.validate()?;
    // A verified committed record may already have expired before notification
    // delivery retries. Malformed inventories or replacement records still fail.
    if previously_verified
        && !records(root)?
            .iter()
            .any(|r| r.manifest.backup_id == reference.backup_id)
    {
        return Ok(());
    }
    let record = resolve(root, &reference.backup_id)?;
    validate_preferences_protection(&record)?;
    if record.manifest.sha256 != reference.sha256
        || !matches!(
            record
                .manifest
                .management
                .as_ref()
                .unwrap()
                .transaction_state
                .as_deref(),
            Some("awaiting_restart" | "committed")
        )
    {
        return Err(fail("restart protection association mismatch"));
    }
    // Persist a verified receipt while cleanup cannot acquire the backup lock.
    // If that write fails the record remains protected. If the following commit
    // fails the receipt permits a verified retry, never a different association.
    persist_verification()?;
    transition_preferences_protection(root, reference, "awaiting_restart", "committed")
}
#[cfg(test)]
fn reconcile_preferences_restart(
    root: &Path,
    reference: &PreferencesProtectionRef,
) -> AppResult<()> {
    reconcile_preferences_restart_recorded(root, reference, false, || Ok(()))
}

/// Always acquire this guard before a preferences mutation, including when the
/// caller's optional protection checkbox is off. Do not re-enter W2 APIs while held.
pub fn acquire_preferences_transaction(root: &Path) -> AppResult<PreferencesProtectionGuard> {
    Ok(PreferencesProtectionGuard {
        _lock: lock(root)?,
        backup: None,
        cleanup_error: None,
    })
}

/// Missing preferences may be skipped only when required=false. Any other read,
/// boundary, link or backup failure is always an error. Current bytes are saved raw
/// so corrupt preferences can still be protected before a repair/install mutation.
/// A verified new backup may trigger saved automatic cleanup before mutation;
/// all protection/transaction records remain retained. Maintenance failures are
/// reported in cleanup_error without invalidating the backup or releasing the lock.
pub fn begin_preferences_protection(
    root: &Path,
    required: bool,
) -> AppResult<PreferencesProtectionGuard> {
    begin_preferences_protection_observed(root, required, &mut ())
}
pub fn begin_preferences_protection_observed(
    root: &Path,
    required: bool,
    observer: &mut dyn MaintenanceObserver,
) -> AppResult<PreferencesProtectionGuard> {
    let mut guard = PreferencesProtectionGuard {
        _lock: observed_lock(root, BackupAction::PreferencesProtection, observer)?,
        backup: None,
        cleanup_error: None,
    };
    let now = Utc::now();
    let target = root.join(PREFERENCES_RELATIVE_PATH);
    safety::check_path(root, &target, true).inspect_err(|_| {
        observer.backup_result(
            BackupAction::PreferencesProtection,
            SOURCE_UNAVAILABLE,
            false,
        );
    })?;
    if !required
        && std::fs::symlink_metadata(&target)
            .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
    {
        return Ok(guard); // No backup attempt: no invented success event.
    }
    let bytes = active_payload(root);
    let outcome = bytes
        .as_ref()
        .map_err(|e| fail(e.to_string()))
        .and_then(|bytes| {
            managed_backup(root, BackupAction::PreferencesProtection, bytes, now, None)
        });
    observer.backup_result(
        BackupAction::PreferencesProtection,
        bytes
            .as_deref()
            .unwrap_or(b"preferences-source-unavailable"),
        outcome.is_ok(),
    );
    guard.backup = Some(result(&outcome?));
    guard.cleanup_error = best_effort_saved_cleanup(root, now, observer);
    Ok(guard)
}

fn active_payload(root: &Path) -> AppResult<Vec<u8>> {
    let target = root.join(PREFERENCES_RELATIVE_PATH);
    safety::check_path(root, &target, false)?;
    safety::read_file(&target, safety::MAX_PAYLOAD_BYTES)
}
/// Use the existing preferences parser, but reject malformed schema declarations
/// before its legacy-default conversion, and reject unrelated JSON documents.
fn validate_preferences(payload: &[u8]) -> AppResult<()> {
    let value: serde_json::Value =
        serde_json::from_slice(payload).map_err(|_| fail("invalid preferences JSON"))?;
    let object = value
        .as_object()
        .ok_or_else(|| fail("preferences must be an object"))?;
    if let Some(schema) = object.get("schema_version") {
        let schema = schema
            .as_u64()
            .ok_or_else(|| fail("invalid preferences schema type"))?;
        if schema > u64::from(crate::modules::config_migration::PREFERENCES_CURRENT_SCHEMA) {
            return Err(fail("unsupported preferences schema"));
        }
    }
    if !(object.contains_key("interface_scale")
        || object
            .get("appearance")
            .and_then(serde_json::Value::as_object)
            .is_some_and(|g| g.contains_key("interface_scale")))
    {
        return Err(fail("payload is not identifiable manager preferences"));
    }
    for section in [
        "appearance",
        "shell",
        "backup",
        "extensions",
        "maintenance",
        "sync",
        "updates",
        "network",
        "cli",
    ] {
        if object.get(section).is_some_and(|g| !g.is_object()) {
            return Err(fail("invalid preferences section type"));
        }
    }
    preferences::preferences_from_document(&value)
        .map_err(|_| fail("invalid manager preferences"))?;
    Ok(())
}
fn managed_backup(
    root: &Path,
    action: BackupAction,
    bytes: &[u8],
    now: DateTime<Utc>,
    related: Option<String>,
) -> AppResult<BackupRecord> {
    let mut record = super::backup_file(
        root,
        action,
        &root.join(PREFERENCES_RELATIVE_PATH),
        bytes,
        now,
        None,
    )?;
    record.manifest.management = Some(Management {
        version: if action == BackupAction::PreferencesProtection {
            3
        } else if action == BackupAction::RestoreProtection {
            2
        } else {
            1
        },
        kind: if action == BackupAction::ManualPreferences {
            "standalone"
        } else if action == BackupAction::PreferencesProtection {
            "preferences-protection"
        } else {
            "restore-protection"
        }
        .into(),
        pinned: !matches!(
            action,
            BackupAction::ManualPreferences
                | BackupAction::RestoreProtection
                | BackupAction::PreferencesProtection
        ),
        related_backup_id: related,
        transaction_state: matches!(
            action,
            BackupAction::RestoreProtection | BackupAction::PreferencesProtection
        )
        .then(|| "active".into()),
    });
    super::write_manifest(&record.directory, &record.manifest)?;
    safety::sync_directories(root, &record.directory)?;
    if !super::verify_backup(&record)? {
        return Err(fail("new backup failed verification"));
    }
    Ok(record)
}
fn result(record: &BackupRecord) -> UpgradeBackupResult {
    UpgradeBackupResult {
        backup_id: record.manifest.backup_id.clone(),
        directory: record.directory.to_string_lossy().into(),
        target_path: record.manifest.target_path.clone(),
    }
}
pub fn create(
    root: &Path,
    now: DateTime<Utc>,
    policy: BackupCleanupPolicy,
) -> AppResult<PreferencesBackupResultDto> {
    let _lock = lock(root)?;
    create_inner(root, now, policy)
}

/// Read policy under the shared transaction lock. Do not call while holding a guard.
pub fn read_policy(root: &Path) -> AppResult<CleanupPolicy> {
    let _lock = lock(root)?;
    policy::load(root)
}

/// Save policy under the shared lock; changing the mode never runs cleanup.
/// Do not call while holding a preferences transaction guard.
pub fn save_policy(root: &Path, policy: &CleanupPolicy) -> AppResult<()> {
    save_policy_observed(root, policy, |_, _| {})
}

/// The observer runs under the admitted mutation lock after validation. Only
/// exact policy contents may resolve an earlier write failure. No cleanup runs.
/// Lock/admission refusals remain immediate feedback, not terminal write facts.
pub fn save_policy_observed(
    root: &Path,
    policy: &CleanupPolicy,
    observer: impl FnOnce(&[u8], bool),
) -> AppResult<()> {
    policy.validate()?;
    let _lock = lock(root)?;
    let candidate = serde_json::to_vec(policy).map_err(|e| fail(e.to_string()))?;
    let result = policy::save(root, policy);
    observer(&candidate, result.is_ok());
    result
}

/// Read persisted mode and create under one lock, preventing policy-change races.
/// Only a successfully verified backup can trigger automatic preview+execute.
/// Do not call while holding a preferences transaction guard.
pub fn create_with_saved_policy(
    root: &Path,
    now: DateTime<Utc>,
) -> AppResult<PreferencesBackupResultDto> {
    create_with_saved_policy_observed(root, now, |_| (), |_, _| {})
}

/// Capture the exact source under the mutation lock. Observers must not re-enter
/// any backup API; their failure must never invalidate the committed backup.
pub fn create_with_saved_policy_observed<T>(
    root: &Path,
    now: DateTime<Utc>,
    prepare: impl FnOnce(&[u8]) -> T,
    terminal: impl FnOnce(T, &AppResult<PreferencesBackupResultDto>),
) -> AppResult<PreferencesBackupResultDto> {
    create_with_saved_policy_and_observers(root, now, prepare, terminal, &mut ())
}

pub fn create_with_saved_policy_and_observers<T>(
    root: &Path,
    now: DateTime<Utc>,
    prepare: impl FnOnce(&[u8]) -> T,
    terminal: impl FnOnce(T, &AppResult<PreferencesBackupResultDto>),
    observer: &mut dyn MaintenanceObserver,
) -> AppResult<PreferencesBackupResultDto> {
    let _lock = match lock(root) {
        Ok(lock) => lock,
        Err(error) => {
            let identity = prepare(SOURCE_UNAVAILABLE);
            let outcome = Err(error);
            terminal(identity, &outcome);
            return outcome;
        }
    };
    let payload = active_payload(root);
    let identity = prepare(
        payload
            .as_deref()
            .unwrap_or(b"preferences-source-unavailable"),
    );
    let result = payload.and_then(|bytes| {
        let saved = policy::load(root)?;
        let mode = match saved.mode {
            CleanupMode::Manual => BackupCleanupPolicy::Manual,
            CleanupMode::Automatic => BackupCleanupPolicy::AfterSuccessfulBackup,
        };
        create_payload_inner(root, now, mode, &bytes, observer)
    });
    terminal(identity, &result);
    result
}

fn create_inner(
    root: &Path,
    now: DateTime<Utc>,
    policy: BackupCleanupPolicy,
) -> AppResult<PreferencesBackupResultDto> {
    let bytes = active_payload(root)?;
    create_payload_inner(root, now, policy, &bytes, &mut ())
}
fn create_payload_inner(
    root: &Path,
    now: DateTime<Utc>,
    policy: BackupCleanupPolicy,
    bytes: &[u8],
    observer: &mut dyn MaintenanceObserver,
) -> AppResult<PreferencesBackupResultDto> {
    validate_preferences(bytes)?;
    let record = managed_backup(root, BackupAction::ManualPreferences, bytes, now, None)?;
    let mut outcome = PreferencesBackupResultDto {
        backup: result(&record),
        removed_ids: Vec::new(),
        cleanup_error: None,
    };
    if policy == BackupCleanupPolicy::AfterSuccessfulBackup {
        match cleanup_inner(root, now, observer) {
            Ok(ids) => outcome.removed_ids = ids,
            Err(error) => outcome.cleanup_error = Some(error.to_string()),
        }
    }
    Ok(outcome)
}

/// Lock-free internals: every caller already owns the shared transaction lock.
/// Keep this hook outside backup_file/managed_backup: protection metadata and
/// source links must be durable and verified before any cleanup can start.
fn cleanup_inner(
    root: &Path,
    now: DateTime<Utc>,
    observer: &mut dyn MaintenanceObserver,
) -> AppResult<Vec<String>> {
    let preview = preview_inner(root, now, MAX_PER_ACTION);
    let outcome = preview
        .as_ref()
        .map_err(|e| fail(e.to_string()))
        .and_then(|p| execute_inner(root, p, now));
    observer.cleanup_result(preview.as_ref().ok(), outcome.is_ok());
    outcome
}
fn best_effort_saved_cleanup(
    root: &Path,
    now: DateTime<Utc>,
    observer: &mut dyn MaintenanceObserver,
) -> Option<String> {
    let cleanup = (|| {
        let saved = policy::load(root).inspect_err(|_| observer.cleanup_result(None, false))?;
        if saved.mode == CleanupMode::Automatic {
            cleanup_inner(root, now, observer)?;
        }
        Ok::<_, crate::errors::AppError>(())
    })();
    cleanup.err().map(|error| {
        let detail = error.to_string();
        eprintln!("Backup succeeded; automatic cleanup skipped or failed: {detail}");
        detail
    })
}
/// Upgrade uses the same bounded, validated source but preserves the existing DTO/action.
/// Cleanup errors are diagnostic only; a successfully verified backup is returned.
pub fn create_upgrade(root: &Path) -> AppResult<UpgradeBackupResult> {
    create_upgrade_observed(root, &mut ())
}
pub fn create_upgrade_observed(
    root: &Path,
    observer: &mut dyn MaintenanceObserver,
) -> AppResult<UpgradeBackupResult> {
    let _lock = observed_lock(root, BackupAction::Upgrade, observer)?;
    let bytes = active_payload(root);
    let now = Utc::now();
    let outcome = bytes
        .as_ref()
        .map_err(|e| fail(e.to_string()))
        .and_then(|bytes| {
            validate_preferences(bytes)?;
            let record = super::backup_file(
                root,
                BackupAction::Upgrade,
                &root.join(PREFERENCES_RELATIVE_PATH),
                bytes,
                now,
                Some("ui-requested".into()),
            )?;
            if !super::verify_backup(&record)? {
                return Err(fail("upgrade backup verification failed"));
            }
            Ok(record)
        });
    observer.backup_result(
        BackupAction::Upgrade,
        bytes
            .as_deref()
            .unwrap_or(b"preferences-source-unavailable"),
        outcome.is_ok(),
    );
    let record = outcome?;
    let _ = best_effort_saved_cleanup(root, now, observer);
    Ok(result(&record))
}
fn records(root: &Path) -> AppResult<Vec<BackupRecord>> {
    safety::scan(&root.join("backups"))
}
fn related(records: &[BackupRecord]) -> BTreeSet<String> {
    records
        .iter()
        .filter_map(|r| {
            r.manifest
                .management
                .as_ref()
                .filter(|m| !reconciled_restore(r, m))
                .and_then(|m| m.related_backup_id.clone())
        })
        .collect()
}
fn reconciled_restore(record: &BackupRecord, meta: &Management) -> bool {
    record.manifest.action == BackupAction::RestoreProtection
        && meta.version == 2
        && meta.kind == "restore-protection"
        && meta.related_backup_id.is_some()
        && meta.transaction_state.as_deref() == Some("committed")
}
fn reconciled_preferences(record: &BackupRecord, meta: &Management) -> bool {
    record.manifest.action == BackupAction::PreferencesProtection
        && meta.version == 3
        && meta.kind == "preferences-protection"
        && meta.related_backup_id.is_none()
        && meta.transaction_state.as_deref() == Some("committed")
}
fn protection(record: &BackupRecord, related: &BTreeSet<String>) -> Option<String> {
    let m = &record.manifest;
    if related.contains(&m.backup_id) {
        return Some("referenced-by-protection".into());
    }
    if !candidate(record) {
        return Some("unrecognized-preferences-scope".into());
    }
    let Some(meta) = &m.management else {
        return Some("legacy-transaction-unresolved".into());
    };
    if meta.pinned {
        return Some("pinned".into());
    }
    if reconciled_restore(record, meta) || reconciled_preferences(record, meta) {
        return None;
    }
    if m.action != BackupAction::ManualPreferences
        || meta.version != 1
        || meta.kind != "standalone"
        || meta.related_backup_id.is_some()
        || meta.transaction_state.is_some()
    {
        return Some("transaction-or-unknown-management".into());
    }
    None
}
fn candidate(record: &BackupRecord) -> bool {
    // target_path is evidence of scope, never a restore destination.
    Path::new(&record.manifest.target_path).ends_with(PREFERENCES_RELATIVE_PATH)
        && safety::payload_path(&record.directory)
            .ok()
            .is_some_and(|p| p.file_name().is_some_and(|n| n == "preferences.json"))
        && record.manifest.restorable
}
/// Complete bounded metadata snapshot for the CLI's existing snake-case DTO.
/// Like GUI listing this may create the cooperative lock, so callers must own
/// storage admission. No hash verification or cleanup happens during listing.
pub(crate) fn list_records(root: &Path) -> AppResult<Vec<BackupRecord>> {
    let _lock = lock(root)?;
    records(root)
}

pub fn list(root: &Path) -> AppResult<Vec<PreferencesBackupDto>> {
    let _lock = lock(root)?;
    let mut records = records(root)?;
    records.sort_by_key(|r| {
        (
            std::cmp::Reverse(r.manifest.created_at.clone()),
            r.manifest.backup_id.clone(),
        )
    });
    let links = related(&records);
    Ok(records
        .iter()
        .map(|r| {
            let reason = protection(r, &links);
            PreferencesBackupDto {
                id: r.manifest.backup_id.clone(),
                action: r.manifest.action.as_str().into(),
                created_at: r.manifest.created_at.clone(),
                bytes: r.manifest.bytes,
                pinned: r.manifest.management.as_ref().is_some_and(|m| m.pinned),
                protected: reason.is_some(),
                protection_reason: reason,
                preferences_candidate: candidate(r),
                integrity_verified: false,
            }
        })
        .collect())
}
fn resolve(root: &Path, id: &str) -> AppResult<BackupRecord> {
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return Err(fail("invalid backup id"));
    }
    let found: Vec<_> = records(root)?
        .into_iter()
        .filter(|r| r.manifest.backup_id == id)
        .collect();
    if found.len() != 1 {
        return Err(fail("backup id missing or ambiguous"));
    }
    Ok(found.into_iter().next().unwrap())
}
pub fn set_pinned(root: &Path, id: &str, pinned: bool) -> AppResult<()> {
    set_pinned_observed(root, id, pinned, |_, _| {})
}

/// Observe only an admitted manifest mutation, after durable write and readback.
/// Failed admission or protection reconciliation never produces a write fact.
pub fn set_pinned_observed(
    root: &Path,
    id: &str,
    pinned: bool,
    observer: impl FnOnce(&[u8], bool),
) -> AppResult<()> {
    set_pinned_with_writer(root, id, pinned, persist_pin, observer)
}

fn persist_pin(root: &Path, record: &BackupRecord) -> AppResult<()> {
    super::write_manifest(&record.directory, &record.manifest)?;
    safety::sync_directories(root, &record.directory)
}

fn set_pinned_with_writer(
    root: &Path,
    id: &str,
    pinned: bool,
    writer: impl FnOnce(&Path, &BackupRecord) -> AppResult<()>,
    observer: impl FnOnce(&[u8], bool),
) -> AppResult<()> {
    let _lock = lock(root)?;
    let mut r = resolve(root, id)?;
    let reconciled = r
        .manifest
        .management
        .as_ref()
        .is_some_and(|m| reconciled_restore(&r, m) || reconciled_preferences(&r, m));
    // Pinning legacy records is allowed, unpinning never releases their protection.
    let meta = r.manifest.management.get_or_insert(Management {
        version: 1,
        kind: "legacy-protected".into(),
        pinned: false,
        related_backup_id: None,
        transaction_state: None,
    });
    if meta.version != 1
        && !(meta.version == 3
            && r.manifest.action == BackupAction::PreferencesProtection
            && meta.kind == "preferences-protection"
            && meta.related_backup_id.is_none()
            && matches!(
                meta.transaction_state.as_deref(),
                Some("active" | "awaiting_restart" | "committed")
            ))
        && !(meta.version == 2
            && r.manifest.action == BackupAction::RestoreProtection
            && meta.kind == "restore-protection"
            && meta.related_backup_id.is_some()
            && matches!(
                meta.transaction_state.as_deref(),
                Some("active" | "committed")
            ))
    {
        return Err(fail("unknown management version"));
    }
    if matches!(
        r.manifest.action,
        BackupAction::RestoreProtection | BackupAction::PreferencesProtection
    ) && !pinned
        && !reconciled
    {
        return Err(fail(
            "restore guards cannot be released without transaction reconciliation",
        ));
    }
    meta.pinned = pinned;
    safety::check_path(root, &r.directory, false)?;
    // Hash exact backup identity and requested state, never paths or user content.
    // A different backup or an unpin success cannot resolve a pin failure.
    let candidate = serde_json::to_vec(&(
        &r.manifest.backup_id,
        r.manifest.action,
        &r.manifest.sha256,
        r.manifest.bytes,
        pinned,
    ))
    .map_err(|e| fail(e.to_string()))?;
    let result = writer(root, &r).and_then(|()| {
        let persisted = safety::load_record(&r.directory)?;
        if persisted.manifest != r.manifest {
            return Err(fail("pin manifest readback mismatch"));
        }
        Ok(())
    });
    observer(&candidate, result.is_ok());
    result
}
fn preview_inner(
    root: &Path,
    now: DateTime<Utc>,
    count: usize,
) -> AppResult<BackupCleanupPreviewDto> {
    let now = DateTime::parse_from_rfc3339(&now.to_rfc3339_opts(SecondsFormat::Secs, true))
        .map_err(|_| fail("invalid preview clock"))?
        .with_timezone(&Utc);
    if count == 0 || count > 1000 {
        return Err(fail("invalid retention count"));
    }
    let mut all = records(root)?;
    let links = related(&all);
    // Chronological sorting is by parsed time, not lexicographic timezone strings.
    all.sort_by_key(|r| {
        (
            std::cmp::Reverse(DateTime::parse_from_rfc3339(&r.manifest.created_at).unwrap()),
            r.manifest.backup_id.clone(),
        )
    });
    let mut unpinned_by_action = std::collections::BTreeMap::new();
    let mut ids = Vec::new();
    let mut bytes = 0_u64;
    for r in &all {
        if protection(r, &links).is_some() {
            continue;
        }
        let created = DateTime::parse_from_rfc3339(&r.manifest.created_at)
            .map_err(|_| fail("invalid time"))?;
        let unpinned_index = unpinned_by_action
            .entry(r.manifest.action.as_str())
            .or_insert(0);
        let keep =
            *unpinned_index < count || now.signed_duration_since(created) < Duration::days(30);
        *unpinned_index += 1;
        if !keep {
            ids.push(r.manifest.backup_id.clone());
            bytes = bytes
                .checked_add(r.manifest.bytes)
                .ok_or_else(|| fail("size overflow"))?;
        }
    }
    // Bind preview to the entire inventory (including links/pins), active root and policy.
    let inventory: Vec<_> = all.iter().map(|r| (&r.manifest, &r.directory)).collect();
    let snapshot = serde_json::to_vec(&(
        root.canonicalize().map_err(|e| fail(e.to_string()))?,
        count,
        &ids,
        inventory,
    ))
    .map_err(|e| fail(e.to_string()))?;
    Ok(BackupCleanupPreviewDto {
        token: sha256_hex(&snapshot),
        created_at: now.to_rfc3339_opts(SecondsFormat::Secs, true),
        candidate_ids: ids.clone(),
        candidate_bytes: bytes,
        retained_count: all.len() - ids.len(),
        keep_recent: count,
        keep_days: 30,
    })
}
pub fn preview(root: &Path, now: DateTime<Utc>) -> AppResult<BackupCleanupPreviewDto> {
    let _lock = lock(root)?;
    preview_inner(root, now, MAX_PER_ACTION)
}
fn execute_inner(
    root: &Path,
    preview: &BackupCleanupPreviewDto,
    now: DateTime<Utc>,
) -> AppResult<Vec<String>> {
    let at = DateTime::parse_from_rfc3339(&preview.created_at)
        .map_err(|_| fail("invalid preview time"))?
        .with_timezone(&Utc);
    if now < at || now.signed_duration_since(at) > Duration::minutes(15) {
        return Err(fail("cleanup preview expired"));
    }
    let fresh = preview_inner(root, at, preview.keep_recent)?;
    if fresh != *preview {
        return Err(fail("backup inventory changed; preview again"));
    }
    let mut verified = Vec::new();
    // Validate every candidate before the first deletion. Never hash retained payloads.
    for id in &fresh.candidate_ids {
        let r = resolve(root, id)?;
        safety::check_path(root, &r.directory, false)?;
        if !super::verify_backup(&r)? {
            return Err(fail("cleanup candidate SHA-256 mismatch"));
        }
        verified.push(r);
    }
    if preview_inner(root, at, preview.keep_recent)? != fresh {
        return Err(fail("backup inventory changed during verification"));
    }
    let mut removed = Vec::new();
    for r in verified {
        safety::check_path(root, &r.directory, false)?;
        if safety::load_record(&r.directory)? != r || !super::verify_backup(&r)? {
            return Err(fail("cleanup candidate changed"));
        }
        std::fs::remove_dir_all(&r.directory).map_err(|e| fail(e.to_string()))?;
        removed.push(r.manifest.backup_id);
    }
    Ok(removed)
}
pub fn execute(
    root: &Path,
    preview: &BackupCleanupPreviewDto,
    now: DateTime<Utc>,
) -> AppResult<Vec<String>> {
    let _lock = lock(root)?;
    if preview.keep_recent != MAX_PER_ACTION || preview.keep_days != 30 {
        return Err(fail("unsupported cleanup policy"));
    }
    execute_inner(root, preview, now)
}
pub(super) fn cleanup_with_count(
    root: &Path,
    now: DateTime<Utc>,
    count: usize,
) -> AppResult<Vec<String>> {
    let _lock = lock(root)?;
    let preview = preview_inner(root, now, count)?;
    execute_inner(root, &preview, now)
}
/// The linked protection backup can trigger saved automatic cleanup before source
/// validation. Both guard and source stay protected even if restore later fails.
/// Cleanup failures are diagnostic only and never prevent the guarded restore.
pub fn restore(
    root: &Path,
    id: &str,
    now: DateTime<Utc>,
) -> AppResult<PreferencesRestoreResultDto> {
    restore_observed(root, id, now, &mut ())
}
pub fn restore_observed(
    root: &Path,
    id: &str,
    now: DateTime<Utc>,
    observer: &mut dyn MaintenanceObserver,
) -> AppResult<PreferencesRestoreResultDto> {
    if id.is_empty()
        || id.len() > 128
        || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(fail("invalid backup id"));
    }
    let _lock = observed_lock(root, BackupAction::RestoreProtection, observer)?;
    let before = active_payload(root).inspect_err(|_| {
        observer.backup_result(BackupAction::RestoreProtection, SOURCE_UNAVAILABLE, false);
    })?;
    // Raw current bytes are protected even if the current preferences are corrupt.
    let protection = managed_backup(
        root,
        BackupAction::RestoreProtection,
        &before,
        now,
        Some(id.into()),
    );
    observer.backup_result(BackupAction::RestoreProtection, &before, protection.is_ok());
    let mut guard = protection?;
    // The active guard and its source reference must exist before cleanup. Even
    // when the later restore fails, neither record can become a cleanup candidate.
    let _ = best_effort_saved_cleanup(root, now, observer);
    let attempt = (|| {
        let r = resolve(root, id)?;
        if !candidate(&r) {
            return Err(fail("backup is not manager preferences"));
        }
        safety::check_path(root, &r.directory, false)?;
        let bytes = safety::read_file(
            &safety::payload_path(&r.directory)?,
            safety::MAX_PAYLOAD_BYTES,
        )?;
        if bytes.len() as u64 != r.manifest.bytes || sha256_hex(&bytes) != r.manifest.sha256 {
            return Err(fail("restore size/SHA-256 mismatch"));
        }
        validate_preferences(&bytes)?;
        let target = root.join(PREFERENCES_RELATIVE_PATH);
        safety::check_path(root, &target, false)?;
        if active_payload(root)? != before {
            return Err(fail("active preferences changed during restore"));
        }
        // Preserve the original document exactly, including browser mode and unknown fields.
        super::write_and_verify(&target, &bytes, 0o600)?;
        // Reconcile under the same lock, only after exact write and readback. No
        // Drop reconciliation: failed/crashed operations must retain their guard.
        // A metadata failure cannot turn an already committed restore into a failure.
        let protection_reconciliation_pending =
            release_restore_protection(root, &mut guard).is_err();
        Ok(PreferencesRestoreResultDto {
            refresh_required: false,
            protection_reconciliation_pending,
            backup_id: id.into(),
            protection_backup_id: guard.manifest.backup_id.clone(),
            target_path: target.to_string_lossy().into(),
        })
    })();
    attempt.map_err(|e| {
        fail(format!(
            "{e}; protection backup retained: {}",
            guard.manifest.backup_id
        ))
    })
}
fn release_restore_protection(root: &Path, guard: &mut BackupRecord) -> AppResult<()> {
    let meta = guard
        .manifest
        .management
        .as_mut()
        .ok_or_else(|| fail("missing restore protection"))?;
    if guard.manifest.action != BackupAction::RestoreProtection
        || meta.version != 2
        || meta.kind != "restore-protection"
        || meta.related_backup_id.is_none()
        || meta.transaction_state.as_deref() != Some("active")
    {
        return Err(fail("restore protection cannot be reconciled"));
    }
    safety::check_path(root, &guard.directory, false)?;
    // Verify the persisted active guard before releasing it. Update a copy so a
    // failed write never reports a committed state through the in-memory handle.
    let stored = resolve(root, &guard.manifest.backup_id)?;
    if stored.manifest != guard.manifest || !super::verify_backup(&stored)? {
        return Err(fail("active restore protection failed verification"));
    }
    let mut committed = guard.manifest.clone();
    committed.management.as_mut().unwrap().transaction_state = Some("committed".into());
    super::write_manifest(&guard.directory, &committed)?;
    safety::sync_directories(root, &guard.directory)?;
    let stored = resolve(root, &guard.manifest.backup_id)?;
    if stored.manifest != committed || !super::verify_backup(&stored)? {
        return Err(fail("restore reconciliation failed verification"));
    }
    guard.manifest = committed;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::preferences::{Preferences, PreferencesStore};
    use chrono::TimeZone;
    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 10, 12, 0, 0).unwrap()
    }
    fn root() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        PreferencesStore::new(root.path())
            .save(&Preferences::default())
            .unwrap();
        root
    }
    fn manual(root: &Path, age: i64) -> BackupRecord {
        managed_backup(
            root,
            BackupAction::ManualPreferences,
            &active_payload(root).unwrap(),
            now() - Duration::days(age),
            None,
        )
        .unwrap()
    }
    fn aged_protection(root: &Path) -> PreferencesProtectionGuard {
        let mut guard = acquire_preferences_transaction(root).unwrap();
        let record = managed_backup(
            root,
            BackupAction::PreferencesProtection,
            &active_payload(root).unwrap(),
            now() - Duration::days(100),
            None,
        )
        .unwrap();
        guard.backup = Some(result(&record));
        guard
    }
    #[test]
    fn interrupted_preferences_protection_is_not_a_user_pin_and_never_expires() {
        let root = root();
        let guard = aged_protection(root.path());
        let reference = guard.protection_ref(root.path()).unwrap().unwrap();
        drop(guard);
        let record = resolve(root.path(), &reference.backup_id).unwrap();
        assert!(!record.manifest.management.as_ref().unwrap().pinned);
        assert!(protection(&record, &BTreeSet::new()).is_some());
        assert!(set_pinned(root.path(), &reference.backup_id, false).is_err());
        assert!(reconcile_preferences_restart(root.path(), &reference).is_err());
        assert!(preview_inner(root.path(), now(), 1)
            .unwrap()
            .candidate_ids
            .is_empty());
    }
    #[test]
    fn verified_preferences_commit_allows_rotation_but_preserves_independent_pin() {
        let root = root();
        let guard = aged_protection(root.path());
        let reference = guard.protection_ref(root.path()).unwrap().unwrap();
        guard.commit_verified(root.path()).unwrap();
        guard.commit_verified(root.path()).unwrap();
        drop(guard);
        set_pinned(root.path(), &reference.backup_id, true).unwrap();
        let recent = begin_preferences_protection(root.path(), true).unwrap();
        recent.commit_verified(root.path()).unwrap();
        drop(recent);
        assert!(preview_inner(root.path(), now(), 1)
            .unwrap()
            .candidate_ids
            .is_empty());
        set_pinned(root.path(), &reference.backup_id, false).unwrap();
        assert_eq!(
            preview_inner(root.path(), now(), 1).unwrap().candidate_ids,
            vec![reference.backup_id]
        );
    }
    #[test]
    fn pending_restart_requires_exact_digest_and_is_idempotently_reconciled() {
        let root = root();
        let guard = aged_protection(root.path());
        let reference = guard.await_restart(root.path()).unwrap().unwrap();
        drop(guard);
        set_pinned(root.path(), &reference.backup_id, true).unwrap();
        let before = resolve(root.path(), &reference.backup_id).unwrap().manifest;
        let wrong = PreferencesProtectionRef {
            sha256: "0".repeat(64),
            ..reference.clone()
        };
        assert!(reconcile_preferences_restart(root.path(), &wrong).is_err());
        let wrong = PreferencesProtectionRef {
            backup_id: "absent_id".into(),
            ..reference.clone()
        };
        assert!(reconcile_preferences_restart(root.path(), &wrong).is_err());
        assert_eq!(
            resolve(root.path(), &reference.backup_id).unwrap().manifest,
            before
        );
        reconcile_preferences_restart(root.path(), &reference).unwrap();
        reconcile_preferences_restart(root.path(), &reference).unwrap();
        let after = resolve(root.path(), &reference.backup_id).unwrap();
        assert!(after.manifest.management.as_ref().unwrap().pinned);
        assert_eq!(
            after
                .manifest
                .management
                .as_ref()
                .unwrap()
                .transaction_state
                .as_deref(),
            Some("committed")
        );
    }
    #[test]
    fn failed_verification_persistence_keeps_restart_protection_active() {
        let root = root();
        let guard = aged_protection(root.path());
        let reference = guard.await_restart(root.path()).unwrap().unwrap();
        drop(guard);
        let before = resolve(root.path(), &reference.backup_id).unwrap().manifest;
        assert!(
            reconcile_preferences_restart_recorded(root.path(), &reference, false, || {
                let contender =
                    safety::open_file(&root.path().join(".backup-w2.lock"), true).unwrap();
                assert!(contender.try_lock().is_err());
                Err(fail("injected receipt write failure"))
            })
            .is_err()
        );
        assert_eq!(
            resolve(root.path(), &reference.backup_id).unwrap().manifest,
            before
        );
        assert!(set_pinned(root.path(), &reference.backup_id, false).is_err());
    }
    #[test]
    fn tampered_payload_and_legacy_or_unknown_metadata_cannot_release_protection() {
        let root = root();
        let guard = aged_protection(root.path());
        let reference = guard.await_restart(root.path()).unwrap().unwrap();
        drop(guard);
        let record = resolve(root.path(), &reference.backup_id).unwrap();
        let payload = safety::payload_path(&record.directory).unwrap();
        let bytes = std::fs::read(&payload).unwrap();
        std::fs::write(&payload, b"replaced").unwrap();
        assert!(reconcile_preferences_restart(root.path(), &reference).is_err());
        std::fs::write(&payload, bytes).unwrap();
        for (version, state) in [(1, "awaiting_restart"), (3, "unknown")] {
            let mut manifest = record.manifest.clone();
            let meta = manifest.management.as_mut().unwrap();
            meta.version = version;
            meta.transaction_state = Some(state.into());
            super::super::write_manifest(&record.directory, &manifest).unwrap();
            assert!(reconcile_preferences_restart(root.path(), &reference).is_err());
            assert_eq!(
                resolve(root.path(), &reference.backup_id).unwrap().manifest,
                manifest
            );
            assert!(protection(
                &resolve(root.path(), &reference.backup_id).unwrap(),
                &BTreeSet::new()
            )
            .is_some());
        }
    }
    struct Probe<'a> {
        root: &'a Path,
        events: Vec<(&'static str, bool)>,
        damage_cleanup: bool,
    }
    impl MaintenanceObserver for Probe<'_> {
        fn backup_result(&mut self, action: BackupAction, payload: &[u8], succeeded: bool) {
            let contender = safety::open_file(&self.root.join(".backup-w2.lock"), true).unwrap();
            assert!(
                contender.try_lock().is_err(),
                "callback must retain mutation ownership"
            );
            self.events.push((action.as_str(), succeeded));
            if succeeded {
                let found = records(self.root)
                    .unwrap()
                    .into_iter()
                    .find(|r| r.manifest.action == action)
                    .unwrap();
                assert!(super::super::verify_backup(&found).unwrap());
                assert_eq!(
                    std::fs::read(safety::payload_path(&found.directory).unwrap()).unwrap(),
                    payload
                );
                if self.damage_cleanup {
                    // Backup already verified; corrupt another record's manifest before cleanup.
                    let record = records(self.root)
                        .unwrap()
                        .into_iter()
                        .find(|r| r.manifest.action == BackupAction::ManualPreferences)
                        .unwrap();
                    std::fs::write(record.directory.join("backup-manifest.json"), b"broken")
                        .unwrap();
                }
            }
        }
        fn cleanup_result(&mut self, preview: Option<&BackupCleanupPreviewDto>, succeeded: bool) {
            self.events.push(("cleanup", succeeded));
            if self.damage_cleanup {
                assert!(preview.is_none());
                assert!(!succeeded);
            }
        }
    }
    #[test]
    fn protection_success_precedes_cleanup_failure_and_keeps_its_guard() {
        let root = root();
        manual(root.path(), 60);
        save_policy(
            root.path(),
            &CleanupPolicy {
                mode: CleanupMode::Automatic,
                ..Default::default()
            },
        )
        .unwrap();
        let mut probe = Probe {
            root: root.path(),
            events: vec![],
            damage_cleanup: true,
        };
        let guard = begin_preferences_protection_observed(root.path(), true, &mut probe).unwrap();
        assert_eq!(
            probe.events,
            vec![("preferences-protection", true), ("cleanup", false)]
        );
        assert!(guard.cleanup_error.is_some());
        let directory = std::path::PathBuf::from(&guard.backup.as_ref().unwrap().directory);
        let record = safety::load_record(&directory).unwrap();
        let meta = record.manifest.management.unwrap();
        assert!(!meta.pinned);
        assert_eq!(meta.transaction_state.as_deref(), Some("active"));
        let contender = safety::open_file(&root.path().join(".backup-w2.lock"), true).unwrap();
        assert!(
            contender.try_lock().is_err(),
            "cleanup failure must not release protection"
        );
    }
    #[test]
    fn upgrade_bad_payload_emits_failure_and_never_starts_cleanup() {
        let root = root();
        save_policy(
            root.path(),
            &CleanupPolicy {
                mode: CleanupMode::Automatic,
                ..Default::default()
            },
        )
        .unwrap();
        std::fs::write(root.path().join(PREFERENCES_RELATIVE_PATH), b"broken").unwrap();
        let mut probe = Probe {
            root: root.path(),
            events: vec![],
            damage_cleanup: false,
        };
        assert!(create_upgrade_observed(root.path(), &mut probe).is_err());
        assert_eq!(probe.events, vec![("upgrade", false)]);
        assert!(records(root.path()).unwrap().is_empty());
    }
    #[test]
    fn skipped_optional_protection_does_not_emit_success_or_cleanup() {
        let root = root();
        std::fs::remove_file(root.path().join(PREFERENCES_RELATIVE_PATH)).unwrap();
        let mut probe = Probe {
            root: root.path(),
            events: vec![],
            damage_cleanup: false,
        };
        let guard = begin_preferences_protection_observed(root.path(), false, &mut probe).unwrap();
        assert!(guard.backup.is_none());
        assert!(probe.events.is_empty());
    }
    #[derive(Default)]
    struct FailureProbe(Vec<(&'static str, Vec<u8>, bool)>);
    impl MaintenanceObserver for FailureProbe {
        fn backup_result(&mut self, action: BackupAction, payload: &[u8], succeeded: bool) {
            self.0.push((action.as_str(), payload.to_vec(), succeeded));
        }
        fn cleanup_result(&mut self, preview: Option<&BackupCleanupPreviewDto>, succeeded: bool) {
            assert!(preview.is_none());
            self.0.push(("cleanup", vec![], succeeded));
        }
    }
    #[test]
    fn admission_failures_report_once_without_source_reads_or_backup_writes() {
        let container = tempfile::tempdir().unwrap();
        let path = container.path().join("not-a-directory");
        std::fs::write(&path, b"unchanged").unwrap();
        for action in [
            BackupAction::Upgrade,
            BackupAction::PreferencesProtection,
            BackupAction::RestoreProtection,
        ] {
            let mut probe = FailureProbe::default();
            let failed = match action {
                BackupAction::Upgrade => create_upgrade_observed(&path, &mut probe).is_err(),
                BackupAction::PreferencesProtection => {
                    begin_preferences_protection_observed(&path, false, &mut probe).is_err()
                }
                _ => restore_observed(&path, "valid_id", now(), &mut probe).is_err(),
            };
            assert!(failed);
            assert_eq!(
                probe.0,
                vec![(action.as_str(), SOURCE_UNAVAILABLE.to_vec(), false)]
            );
        }
        let calls = std::cell::Cell::new(0);
        assert!(create_with_saved_policy_observed(
            &path,
            now(),
            |payload| {
                assert_eq!(payload, SOURCE_UNAVAILABLE);
                calls.set(calls.get() + 1);
            },
            |(), result| {
                assert!(result.is_err());
                calls.set(calls.get() + 1);
            }
        )
        .is_err());
        assert_eq!(calls.get(), 2);
        assert_eq!(std::fs::read(&path).unwrap(), b"unchanged");
    }
    #[test]
    fn optional_protection_invalid_source_is_a_failure_not_a_skip() {
        let root = root();
        let target = root.path().join(PREFERENCES_RELATIVE_PATH);
        std::fs::remove_file(&target).unwrap();
        std::fs::create_dir(&target).unwrap();
        let mut probe = FailureProbe::default();
        assert!(begin_preferences_protection_observed(root.path(), false, &mut probe).is_err());
        assert_eq!(
            probe.0,
            vec![("preferences-protection", SOURCE_UNAVAILABLE.to_vec(), false)]
        );
        assert!(records(root.path()).unwrap().is_empty());
    }
    #[test]
    fn restore_missing_source_reports_guard_failure_but_invalid_id_attempts_no_guard() {
        let root = root();
        std::fs::remove_file(root.path().join(PREFERENCES_RELATIVE_PATH)).unwrap();
        let mut probe = FailureProbe::default();
        assert!(restore_observed(root.path(), "../invalid", now(), &mut probe).is_err());
        assert!(probe.0.is_empty());
        assert!(restore_observed(root.path(), "valid_id", now(), &mut probe).is_err());
        assert_eq!(
            probe.0,
            vec![("restore-protection", SOURCE_UNAVAILABLE.to_vec(), false)]
        );
        assert!(records(root.path()).unwrap().is_empty());
    }
    #[test]
    fn unreadable_cleanup_policy_reports_failure_after_verified_backup_once() {
        let root = root();
        let path = root.path().join(policy::POLICY_RELATIVE_PATH);
        std::fs::write(path, b"broken").unwrap();
        let mut probe = FailureProbe::default();
        let guard = begin_preferences_protection_observed(root.path(), true, &mut probe).unwrap();
        assert!(guard.backup.is_some());
        assert!(guard.cleanup_error.is_some());
        assert_eq!(probe.0.len(), 2);
        assert_eq!(probe.0[0].0, "preferences-protection");
        assert!(probe.0[0].2);
        assert_eq!(probe.0[1], ("cleanup", vec![], false));
        assert_eq!(records(root.path()).unwrap().len(), 1);
    }
    #[test]
    fn cleanup_candidate_identity_survives_time_but_changes_with_inventory() {
        let root = root();
        for age in 60..72 {
            manual(root.path(), age);
        }
        let first = preview(root.path(), now()).unwrap();
        let retry = preview(root.path(), now() + Duration::minutes(1)).unwrap();
        assert_ne!(first.created_at, retry.created_at);
        assert_eq!(
            first.token, retry.token,
            "same inventory must allow matching recovery"
        );
        set_pinned(root.path(), &first.candidate_ids[0], true).unwrap();
        assert_ne!(first.token, preview(root.path(), now()).unwrap().token);
    }
    #[test]
    fn observed_backup_binds_terminal_to_the_captured_payload_under_the_same_lock() {
        let root = root();
        let expected = active_payload(root.path()).unwrap();
        let terminal = std::cell::Cell::new(false);
        let result = create_with_saved_policy_observed(
            root.path(),
            now(),
            |bytes| {
                assert_eq!(bytes, expected);
                let contender =
                    safety::open_file(&root.path().join(".backup-w2.lock"), true).unwrap();
                assert!(
                    contender.try_lock().is_err(),
                    "prepare must own mutation lock"
                );
                // A non-cooperative writer changing the source cannot alter the already captured payload.
                std::fs::write(root.path().join(PREFERENCES_RELATIVE_PATH), b"broken").unwrap();
                bytes.to_vec()
            },
            |captured, result| {
                let outcome = result.as_ref().unwrap();
                let record = resolve(root.path(), &outcome.backup.backup_id).unwrap();
                assert_eq!(
                    safety::read_file(
                        &safety::payload_path(&record.directory).unwrap(),
                        safety::MAX_PAYLOAD_BYTES
                    )
                    .unwrap(),
                    captured
                );
                let contender =
                    safety::open_file(&root.path().join(".backup-w2.lock"), true).unwrap();
                assert!(
                    contender.try_lock().is_err(),
                    "terminal must own mutation lock"
                );
                terminal.set(true);
            },
        )
        .unwrap();
        assert!(terminal.get());
        assert!(result.cleanup_error.is_none());
    }
    #[test]
    fn observed_backup_reports_invalid_source_failure_without_creating_a_record() {
        let root = root();
        std::fs::write(root.path().join(PREFERENCES_RELATIVE_PATH), b"broken").unwrap();
        let terminal = std::cell::Cell::new(false);
        let result = create_with_saved_policy_observed(
            root.path(),
            now(),
            |bytes| bytes.to_vec(),
            |captured, result| {
                assert_eq!(captured, b"broken");
                assert!(result.is_err());
                terminal.set(true);
            },
        );
        assert!(result.is_err());
        assert!(terminal.get());
        assert!(records(root.path()).unwrap().is_empty());
    }
    #[test]
    fn real_manual_backup_does_not_expand_scope_and_defaults_to_no_cleanup() {
        let root = root();
        for i in 0..12 {
            manual(root.path(), 60 + i);
        }
        let before = active_payload(root.path()).unwrap();
        let result = create(root.path(), now(), BackupCleanupPolicy::default()).unwrap();
        assert!(result.removed_ids.is_empty());
        assert!(result.cleanup_error.is_none());
        assert_eq!(records(root.path()).unwrap().len(), 13);
        let record = resolve(root.path(), &result.backup.backup_id).unwrap();
        assert_eq!(
            safety::read_file(
                &safety::payload_path(&record.directory).unwrap(),
                safety::MAX_PAYLOAD_BYTES
            )
            .unwrap(),
            before
        );
        assert_eq!(record.manifest.file_count, 1);
        assert!(!list(root.path()).unwrap()[0].integrity_verified);
    }
    #[test]
    fn n10_or_d30_with_pins_additional_and_guards_retained() {
        let root = root();
        for i in 0..12 {
            manual(root.path(), 60 + i);
        }
        let pinned = manual(root.path(), 90);
        set_pinned(root.path(), &pinned.manifest.backup_id, true).unwrap();
        let recent = manual(root.path(), 2);
        let legacy = super::super::backup_file(
            root.path(),
            BackupAction::Upgrade,
            &root.path().join("legacy"),
            b"raw",
            now() - Duration::days(365),
            None,
        )
        .unwrap();
        let p = preview(root.path(), now()).unwrap();
        assert_eq!(p.candidate_ids.len(), 3); // 13 unpinned, ten retained
        assert!(!p.candidate_ids.contains(&pinned.manifest.backup_id));
        assert!(!p.candidate_ids.contains(&recent.manifest.backup_id));
        assert!(!p.candidate_ids.contains(&legacy.manifest.backup_id));
        assert_eq!(execute(root.path(), &p, now()).unwrap().len(), 3);
        assert_eq!(records(root.path()).unwrap().len(), 12); // ten + pin + legacy
        for i in 0..12 {
            manual(root.path(), i);
        }
        let p = preview(root.path(), now()).unwrap();
        let ids: BTreeSet<_> = p.candidate_ids.iter().cloned().collect();
        assert!(records(root.path())
            .unwrap()
            .iter()
            .filter(
                |r| DateTime::parse_from_rfc3339(&r.manifest.created_at).unwrap()
                    > now() - Duration::days(30)
            )
            .all(|r| !ids.contains(&r.manifest.backup_id)));
    }
    #[test]
    fn preview_rechecks_pin_inventory_expiry_and_candidate_hash_before_any_delete() {
        let root = root();
        for i in 0..12 {
            manual(root.path(), 60 + i);
        }
        let p = preview(root.path(), now()).unwrap();
        set_pinned(root.path(), &p.candidate_ids[0], true).unwrap();
        assert!(execute(root.path(), &p, now()).is_err());
        let p = preview(root.path(), now()).unwrap();
        assert!(execute(root.path(), &p, now() + Duration::minutes(16)).is_err());
        let record = resolve(root.path(), &p.candidate_ids[0]).unwrap();
        let payload = safety::payload_path(&record.directory).unwrap();
        let mut bytes = std::fs::read(&payload).unwrap();
        bytes[0] ^= 1;
        std::fs::write(payload, bytes).unwrap();
        assert!(execute(root.path(), &p, now()).is_err());
        assert_eq!(records(root.path()).unwrap().len(), 12);
    }
    #[test]
    fn restore_legacy_schema1_single_payload_to_active_root_only_and_preserve_browser_mode() {
        let root = root();
        let preferences = Preferences {
            interface_scale: 150,
            panel_mode: "browser".into(),
            ..Default::default()
        };
        let bytes = serde_json::to_vec(&preferences).unwrap(); // legacy flat document
        let other_root = tempfile::tempdir().unwrap();
        let old_target = other_root.path().join(PREFERENCES_RELATIVE_PATH);
        let legacy = super::super::backup_file(
            root.path(),
            BackupAction::Upgrade,
            &old_target,
            &bytes,
            now() - Duration::days(1),
            None,
        )
        .unwrap();
        let result = restore(root.path(), &legacy.manifest.backup_id, now()).unwrap();
        assert_eq!(active_payload(root.path()).unwrap(), bytes);
        assert_eq!(
            PreferencesStore::new(root.path())
                .load()
                .unwrap()
                .panel_mode,
            "browser"
        );
        assert!(!old_target.exists());
        let guard = resolve(root.path(), &result.protection_backup_id).unwrap();
        let meta = guard.manifest.management.as_ref().unwrap();
        assert!(!meta.pinned);
        assert!(reconciled_restore(&guard, meta));
        assert!(!result.protection_reconciliation_pending);
        set_pinned(root.path(), &result.protection_backup_id, true).unwrap();
        set_pinned(root.path(), &result.protection_backup_id, false).unwrap();
        let p = preview(root.path(), now() + Duration::days(400)).unwrap();
        assert!(!p.candidate_ids.contains(&legacy.manifest.backup_id));
        assert!(!p.candidate_ids.contains(&result.protection_backup_id));
    }
    #[test]
    fn committed_restore_guards_rotate_by_action_without_consuming_manual_retention() {
        let root = root();
        let source = manual(root.path(), 400);
        for age in 101..114 {
            manual(root.path(), age);
        }
        let mut restored = Vec::new();
        for age in 101..114 {
            restored.push(
                restore(
                    root.path(),
                    &source.manifest.backup_id,
                    now() - Duration::days(age),
                )
                .unwrap(),
            );
        }
        let pinned = &restored.last().unwrap().protection_backup_id;
        set_pinned(root.path(), pinned, true).unwrap();
        let p = preview(root.path(), now()).unwrap();
        assert!(!p.candidate_ids.contains(pinned));
        let all = records(root.path()).unwrap();
        let action_count = |action| {
            all.iter()
                .filter(|r| {
                    r.manifest.action == action && p.candidate_ids.contains(&r.manifest.backup_id)
                })
                .count()
        };
        assert_eq!(action_count(BackupAction::RestoreProtection), 2);
        assert_eq!(action_count(BackupAction::ManualPreferences), 4);
        assert!(p.candidate_ids.contains(&source.manifest.backup_id));
        assert_eq!(execute(root.path(), &p, now()).unwrap().len(), 6);
        assert!(resolve(root.path(), pinned).is_ok());
    }
    #[test]
    fn active_and_legacy_guards_keep_sources_and_reconciliation_preserves_user_pin() {
        let root = root();
        let source = manual(root.path(), 400);
        for age in 101..114 {
            manual(root.path(), age);
        }
        let guard = managed_backup(
            root.path(),
            BackupAction::RestoreProtection,
            &active_payload(root.path()).unwrap(),
            now() - Duration::days(401),
            Some(source.manifest.backup_id.clone()),
        )
        .unwrap();
        set_pinned(root.path(), &guard.manifest.backup_id, true).unwrap();
        assert!(set_pinned(root.path(), &guard.manifest.backup_id, false).is_err());
        let p = preview(root.path(), now()).unwrap();
        assert!(!p.candidate_ids.contains(&source.manifest.backup_id));
        let mut guard = resolve(root.path(), &guard.manifest.backup_id).unwrap();
        release_restore_protection(root.path(), &mut guard).unwrap();
        let released = resolve(root.path(), &guard.manifest.backup_id).unwrap();
        assert!(released.manifest.management.as_ref().unwrap().pinned);
        assert!(reconciled_restore(
            &released,
            released.manifest.management.as_ref().unwrap()
        ));
        let p = preview(root.path(), now()).unwrap();
        assert!(p.candidate_ids.contains(&source.manifest.backup_id));
        assert!(!p.candidate_ids.contains(&released.manifest.backup_id));
        // An old pinned guard has no proof of a completed transaction.
        let mut legacy = managed_backup(
            root.path(),
            BackupAction::RestoreProtection,
            &active_payload(root.path()).unwrap(),
            now() - Duration::days(402),
            Some(source.manifest.backup_id.clone()),
        )
        .unwrap();
        let m = legacy.manifest.management.as_mut().unwrap();
        m.version = 1;
        m.pinned = true;
        m.transaction_state = None;
        super::super::write_manifest(&legacy.directory, &legacy.manifest).unwrap();
        assert!(release_restore_protection(root.path(), &mut legacy).is_err());
        assert!(set_pinned(root.path(), &legacy.manifest.backup_id, false).is_err());
        assert!(!preview(root.path(), now())
            .unwrap()
            .candidate_ids
            .contains(&source.manifest.backup_id));
    }
    #[test]
    fn unknown_restore_reconciliation_states_fail_closed() {
        for state in [None, Some("prepared"), Some("failed"), Some("future")] {
            let root = root();
            let source = manual(root.path(), 400);
            for age in 101..114 {
                manual(root.path(), age);
            }
            let mut guard = managed_backup(
                root.path(),
                BackupAction::RestoreProtection,
                &active_payload(root.path()).unwrap(),
                now() - Duration::days(401),
                Some(source.manifest.backup_id.clone()),
            )
            .unwrap();
            guard
                .manifest
                .management
                .as_mut()
                .unwrap()
                .transaction_state = state.map(str::to_owned);
            super::super::write_manifest(&guard.directory, &guard.manifest).unwrap();
            assert!(release_restore_protection(root.path(), &mut guard).is_err());
            let p = preview(root.path(), now()).unwrap();
            assert!(!p.candidate_ids.contains(&source.manifest.backup_id));
            assert!(!p.candidate_ids.contains(&guard.manifest.backup_id));
        }
    }
    #[test]
    fn corrupted_active_guard_is_never_reconciled() {
        let root = root();
        let source = manual(root.path(), 400);
        let mut guard = managed_backup(
            root.path(),
            BackupAction::RestoreProtection,
            &active_payload(root.path()).unwrap(),
            now(),
            Some(source.manifest.backup_id.clone()),
        )
        .unwrap();
        let payload = safety::payload_path(&guard.directory).unwrap();
        let mut bytes = std::fs::read(&payload).unwrap();
        bytes[0] ^= 1;
        std::fs::write(payload, bytes).unwrap();
        assert!(release_restore_protection(root.path(), &mut guard).is_err());
        let stored = resolve(root.path(), &guard.manifest.backup_id).unwrap();
        assert_eq!(
            stored
                .manifest
                .management
                .unwrap()
                .transaction_state
                .as_deref(),
            Some("active")
        );
        assert_eq!(
            guard
                .manifest
                .management
                .unwrap()
                .transaction_state
                .as_deref(),
            Some("active")
        );
    }
    #[test]
    fn failed_restore_keeps_current_bytes_and_protection_for_sha_size_schema_and_type() {
        let root = root();
        let before = active_payload(root.path()).unwrap();
        let mut record = manual(root.path(), 1);
        for payload in [
            br#"{"schema_version":4294967296,"interface_scale":120}"#.as_slice(),
            br#"{"schema_version":"1","interface_scale":120}"#.as_slice(),
            br#"{"interface_scale":"bad"}"#.as_slice(),
            br#"{"unrelated":true}"#.as_slice(),
        ] {
            let path = safety::payload_path(&record.directory).unwrap();
            std::fs::write(&path, payload).unwrap();
            record.manifest.bytes = payload.len() as u64;
            record.manifest.sha256 = sha256_hex(payload);
            super::super::write_manifest(&record.directory, &record.manifest).unwrap();
            let error = restore(root.path(), &record.manifest.backup_id, now())
                .unwrap_err()
                .to_string();
            assert!(error.contains("protection backup retained:"));
            assert_eq!(active_payload(root.path()).unwrap(), before);
        }
        let guards = records(root.path())
            .unwrap()
            .into_iter()
            .filter(|r| r.manifest.action == BackupAction::RestoreProtection)
            .count();
        assert_eq!(guards, 4);
        // Same-size corruption passes listing metadata; restore hashes and rejects it.
        let valid = manual(root.path(), 2);
        let path = safety::payload_path(&valid.directory).unwrap();
        let mut changed = std::fs::read(&path).unwrap();
        changed[0] ^= 1;
        std::fs::write(&path, changed).unwrap();
        assert!(restore(root.path(), &valid.manifest.backup_id, now()).is_err());
        assert_eq!(active_payload(root.path()).unwrap(), before);
        std::fs::write(path, b"short").unwrap();
        assert!(restore(root.path(), &valid.manifest.backup_id, now()).is_err());
        assert_eq!(active_payload(root.path()).unwrap(), before);
    }
    #[test]
    fn unknown_transaction_metadata_cannot_expire() {
        let root = root();
        let mut unknown = manual(root.path(), 300);
        unknown.manifest.management.as_mut().unwrap().version = 99;
        super::super::write_manifest(&unknown.directory, &unknown.manifest).unwrap();
        for i in 0..12 {
            manual(root.path(), 40 + i);
        }
        let p = preview(root.path(), now()).unwrap();
        assert!(!p.candidate_ids.contains(&unknown.manifest.backup_id));
        assert!(set_pinned(root.path(), &unknown.manifest.backup_id, false).is_err());
    }
    #[test]
    fn automatic_cleanup_is_opt_in_and_runs_only_after_success() {
        let root = root();
        for i in 0..12 {
            manual(root.path(), 60 + i);
        }
        std::fs::write(root.path().join(PREFERENCES_RELATIVE_PATH), b"invalid").unwrap();
        assert!(create(
            root.path(),
            now(),
            BackupCleanupPolicy::AfterSuccessfulBackup
        )
        .is_err());
        assert_eq!(records(root.path()).unwrap().len(), 12);
        PreferencesStore::new(root.path())
            .save(&Preferences::default())
            .unwrap();
        let result = create(
            root.path(),
            now(),
            BackupCleanupPolicy::AfterSuccessfulBackup,
        )
        .unwrap();
        assert_eq!(result.removed_ids.len(), 3);
        assert_eq!(records(root.path()).unwrap().len(), 10);
    }
    #[test]
    fn saved_policy_only_cleans_after_success_and_keeps_pins_legacy_and_recent() {
        let root = root();
        for i in 0..12 {
            manual(root.path(), 60 + i);
        }
        let pinned = manual(root.path(), 200);
        set_pinned(root.path(), &pinned.manifest.backup_id, true).unwrap();
        let recent = manual(root.path(), 2);
        let legacy = super::super::backup_file(
            root.path(),
            BackupAction::ManualPreferences,
            &root.path().join(PREFERENCES_RELATIVE_PATH),
            &active_payload(root.path()).unwrap(),
            now() - Duration::days(365),
            None,
        )
        .unwrap();
        assert_eq!(read_policy(root.path()).unwrap(), CleanupPolicy::default());
        assert!(!root.path().join(policy::POLICY_RELATIVE_PATH).exists());
        let manual_result = create_with_saved_policy(root.path(), now()).unwrap();
        assert!(manual_result.removed_ids.is_empty());
        assert_eq!(records(root.path()).unwrap().len(), 16);
        let automatic = CleanupPolicy {
            mode: CleanupMode::Automatic,
            ..Default::default()
        };
        save_policy(root.path(), &automatic).unwrap();
        assert_eq!(read_policy(root.path()).unwrap(), automatic);
        assert_eq!(records(root.path()).unwrap().len(), 16); // save/read never clean
        let result = create_with_saved_policy(root.path(), now()).unwrap();
        assert_eq!(result.removed_ids.len(), 5);
        assert!(result.cleanup_error.is_none());
        assert_eq!(records(root.path()).unwrap().len(), 12); // ten + pin + legacy
        for record in [pinned, recent, legacy] {
            assert!(record.directory.exists());
            assert!(!result.removed_ids.contains(&record.manifest.backup_id));
        }
        save_policy(root.path(), &CleanupPolicy::default()).unwrap();
        assert!(create_with_saved_policy(root.path(), now())
            .unwrap()
            .removed_ids
            .is_empty());
        assert_eq!(records(root.path()).unwrap().len(), 13);
    }
    #[test]
    fn persisted_automatic_fails_closed_for_source_policy_and_candidate_corruption() {
        let root = root();
        for i in 0..12 {
            manual(root.path(), 60 + i);
        }
        let automatic = CleanupPolicy {
            mode: CleanupMode::Automatic,
            ..Default::default()
        };
        save_policy(root.path(), &automatic).unwrap();
        std::fs::write(root.path().join(PREFERENCES_RELATIVE_PATH), b"invalid").unwrap();
        assert!(create_with_saved_policy(root.path(), now()).is_err());
        assert_eq!(records(root.path()).unwrap().len(), 12);
        PreferencesStore::new(root.path())
            .save(&Preferences::default())
            .unwrap();
        let p = preview(root.path(), now()).unwrap();
        let candidate = resolve(root.path(), &p.candidate_ids[0]).unwrap();
        let payload = safety::payload_path(&candidate.directory).unwrap();
        let mut bytes = std::fs::read(&payload).unwrap();
        bytes[0] ^= 1;
        std::fs::write(&payload, bytes).unwrap();
        let result = create_with_saved_policy(root.path(), now()).unwrap();
        assert!(result.removed_ids.is_empty());
        assert!(result.cleanup_error.unwrap().contains("SHA-256"));
        assert!(resolve(root.path(), &result.backup.backup_id).is_ok());
        assert_eq!(records(root.path()).unwrap().len(), 13);
        let path = root.path().join(policy::POLICY_RELATIVE_PATH);
        let bad = br#"{"schema_version":2,"mode":"automatic","keep_recent":10,"keep_days":30}"#;
        std::fs::write(&path, bad).unwrap();
        assert!(read_policy(root.path()).is_err());
        assert!(create_with_saved_policy(root.path(), now()).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bad);
        assert_eq!(records(root.path()).unwrap().len(), 13);
    }
    #[test]
    fn saved_policy_apis_share_transaction_lock_and_helpers_do_not_reenter() {
        use std::{sync::mpsc, time::Duration as Wait};
        let root = root();
        let guard = acquire_preferences_transaction(root.path()).unwrap();
        policy::save(root.path(), &CleanupPolicy::default()).unwrap();
        assert_eq!(policy::load(root.path()).unwrap(), CleanupPolicy::default());
        let (started_tx, started_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let mut workers = Vec::new();
        for operation in 0..3 {
            let path = root.path().to_path_buf();
            let started = started_tx.clone();
            let done = done_tx.clone();
            workers.push(std::thread::spawn(move || {
                started.send(()).unwrap();
                let result = match operation {
                    0 => read_policy(&path).map(|_| ()),
                    1 => save_policy(&path, &CleanupPolicy::default()),
                    _ => create_with_saved_policy(&path, now()).map(|_| ()),
                };
                done.send(result).unwrap();
            }));
        }
        for _ in 0..3 {
            started_rx.recv_timeout(Wait::from_secs(5)).unwrap();
        }
        let completed_while_held = done_rx.recv_timeout(Wait::from_millis(100));
        drop(guard);
        assert!(matches!(
            completed_while_held,
            Err(mpsc::RecvTimeoutError::Timeout)
        ));
        for _ in 0..3 {
            done_rx.recv_timeout(Wait::from_secs(5)).unwrap().unwrap();
        }
        for worker in workers {
            worker.join().unwrap();
        }
    }
    fn automatic_with_old_manuals(root: &Path) {
        for age in 60..72 {
            managed_backup(
                root,
                BackupAction::ManualPreferences,
                &active_payload(root).unwrap(),
                Utc::now() - Duration::days(age),
                None,
            )
            .unwrap();
        }
        save_policy(
            root,
            &CleanupPolicy {
                mode: CleanupMode::Automatic,
                ..Default::default()
            },
        )
        .unwrap();
    }
    #[test]
    fn transaction_automatic_upgrade_and_protection_preserve_all_related_snapshots() {
        for protection_backup in [false, true] {
            let root = root();
            automatic_with_old_manuals(root.path());
            let linked = manual(root.path(), 365);
            let pinned = manual(root.path(), 366);
            set_pinned(root.path(), &pinned.manifest.backup_id, true).unwrap();
            let previous_guard = managed_backup(
                root.path(),
                BackupAction::RestoreProtection,
                &active_payload(root.path()).unwrap(),
                now() - Duration::days(365),
                Some(linked.manifest.backup_id.clone()),
            )
            .unwrap();
            let legacy = super::super::backup_file(
                root.path(),
                BackupAction::Upgrade,
                &root.path().join(PREFERENCES_RELATIVE_PATH),
                &active_payload(root.path()).unwrap(),
                now() - Duration::days(365),
                None,
            )
            .unwrap();
            let new_backup = if protection_backup {
                let guard = begin_preferences_protection(root.path(), true).unwrap();
                assert!(guard.cleanup_error.is_none());
                let competitor =
                    safety::open_file(&root.path().join(".backup-w2.lock"), true).unwrap();
                assert!(competitor.try_lock().is_err());
                guard.backup.as_ref().unwrap().clone()
            } else {
                create_upgrade(root.path()).unwrap()
            };
            // Twelve unlinked standalone records reduce to ten; all four older
            // protected records plus the new upgrade/protection remain additional.
            assert_eq!(records(root.path()).unwrap().len(), 15);
            assert!(Path::new(&new_backup.directory).exists());
            for protected in [linked, pinned, previous_guard, legacy] {
                assert!(protected.directory.exists());
                assert!(resolve(root.path(), &protected.manifest.backup_id).is_ok());
            }
        }
    }
    #[test]
    fn transaction_default_manual_and_missing_optional_source_do_not_clean() {
        let root = root();
        automatic_with_old_manuals(root.path());
        std::fs::remove_file(root.path().join(policy::POLICY_RELATIVE_PATH)).unwrap();
        create_upgrade(root.path()).unwrap();
        let guard = begin_preferences_protection(root.path(), true).unwrap();
        assert!(guard.cleanup_error.is_none());
        drop(guard);
        assert_eq!(records(root.path()).unwrap().len(), 14);
        save_policy(root.path(), &CleanupPolicy::default()).unwrap();
        let guard = begin_preferences_protection(root.path(), true).unwrap();
        drop(guard);
        assert_eq!(records(root.path()).unwrap().len(), 15);
        save_policy(
            root.path(),
            &CleanupPolicy {
                mode: CleanupMode::Automatic,
                ..Default::default()
            },
        )
        .unwrap();
        std::fs::remove_file(root.path().join(PREFERENCES_RELATIVE_PATH)).unwrap();
        let guard = begin_preferences_protection(root.path(), false).unwrap();
        assert!(guard.backup.is_none());
        assert!(guard.cleanup_error.is_none());
        drop(guard);
        assert!(begin_preferences_protection(root.path(), true).is_err());
        assert!(create_upgrade(root.path()).is_err());
        assert_eq!(records(root.path()).unwrap().len(), 15);
    }
    #[test]
    fn transaction_automatic_failed_backup_never_cleans_and_cleanup_error_keeps_guard() {
        let root = root();
        automatic_with_old_manuals(root.path());
        let active = root.path().join(PREFERENCES_RELATIVE_PATH);
        std::fs::write(&active, b"invalid").unwrap();
        assert!(create_upgrade(root.path()).is_err());
        assert_eq!(records(root.path()).unwrap().len(), 12);
        std::fs::File::create(&active)
            .unwrap()
            .set_len(safety::MAX_PAYLOAD_BYTES + 1)
            .unwrap();
        assert!(begin_preferences_protection(root.path(), true).is_err());
        assert_eq!(records(root.path()).unwrap().len(), 12);
        // Raw invalid preferences still get a durable, active protection. A bad
        // candidate hash blocks the entire cleanup without invalidating that guard.
        std::fs::write(&active, b"invalid").unwrap();
        let candidate_id = preview(root.path(), Utc::now()).unwrap().candidate_ids[0].clone();
        let candidate = resolve(root.path(), &candidate_id).unwrap();
        let path = safety::payload_path(&candidate.directory).unwrap();
        let mut bytes = std::fs::read(&path).unwrap();
        bytes[0] ^= 1;
        std::fs::write(path, bytes).unwrap();
        let guard = begin_preferences_protection(root.path(), true).unwrap();
        assert!(guard.cleanup_error.as_ref().unwrap().contains("SHA-256"));
        assert_eq!(records(root.path()).unwrap().len(), 13);
        let protection = resolve(root.path(), &guard.backup.as_ref().unwrap().backup_id).unwrap();
        let meta = protection.manifest.management.as_ref().unwrap();
        assert!(!meta.pinned);
        assert_eq!(meta.transaction_state.as_deref(), Some("active"));
        assert_eq!(
            std::fs::read(safety::payload_path(&protection.directory).unwrap()).unwrap(),
            b"invalid"
        );
    }
    #[test]
    fn transaction_automatic_restore_guard_links_source_before_cleanup_on_success_and_failure() {
        for valid_source in [false, true] {
            let root = root();
            automatic_with_old_manuals(root.path());
            let before = active_payload(root.path()).unwrap();
            let source = managed_backup(
                root.path(),
                BackupAction::ManualPreferences,
                if valid_source {
                    &before
                } else {
                    br#"{"unrelated":true}"#
                },
                Utc::now() - Duration::days(365),
                None,
            )
            .unwrap();
            let result = restore(root.path(), &source.manifest.backup_id, Utc::now());
            assert_eq!(result.is_ok(), valid_source);
            if let Err(error) = result {
                assert!(error.to_string().contains("protection backup retained:"));
            }
            assert!(source.directory.exists());
            assert_eq!(active_payload(root.path()).unwrap(), before);
            let all = records(root.path()).unwrap();
            assert_eq!(all.len(), 12); // ten unrelated + linked source + guard
            let guard = all
                .iter()
                .find(|r| r.manifest.action == BackupAction::RestoreProtection)
                .unwrap();
            let management = guard.manifest.management.as_ref().unwrap();
            assert!(!management.pinned);
            assert_eq!(
                management.transaction_state.as_deref(),
                Some(if valid_source { "committed" } else { "active" })
            );
            assert_eq!(
                management.related_backup_id.as_deref(),
                Some(source.manifest.backup_id.as_str())
            );
            let p = preview(root.path(), Utc::now() + Duration::days(400)).unwrap();
            assert_eq!(
                p.candidate_ids.contains(&source.manifest.backup_id),
                valid_source
            );
            assert!(!p.candidate_ids.contains(&guard.manifest.backup_id));
        }
    }
    #[test]
    fn transaction_invalid_saved_policy_keeps_successful_upgrade_and_protection() {
        let root = root();
        automatic_with_old_manuals(root.path());
        std::fs::write(root.path().join(policy::POLICY_RELATIVE_PATH), b"invalid").unwrap();
        let upgraded = create_upgrade(root.path()).unwrap();
        assert!(Path::new(&upgraded.directory).exists());
        let guard = begin_preferences_protection(root.path(), true).unwrap();
        assert!(guard.cleanup_error.as_ref().unwrap().contains("policy"));
        assert!(Path::new(&guard.backup.as_ref().unwrap().directory).exists());
        assert_eq!(records(root.path()).unwrap().len(), 14);
    }
    #[test]
    fn protection_helper_keeps_raw_bytes_and_holds_shared_lock() {
        let root = root();
        std::fs::write(
            root.path().join(PREFERENCES_RELATIVE_PATH),
            b"corrupt-current",
        )
        .unwrap();
        let guard = begin_preferences_protection(root.path(), true).unwrap();
        let backup = guard.backup.as_ref().unwrap();
        assert_eq!(
            std::fs::read(Path::new(&backup.directory).join("preferences.json")).unwrap(),
            b"corrupt-current"
        );
        let competitor = safety::open_file(&root.path().join(".backup-w2.lock"), true).unwrap();
        assert!(competitor.try_lock().is_err());
        drop(guard);
        assert!(competitor.try_lock().is_ok());
        competitor.unlock().unwrap();
        std::fs::remove_file(root.path().join(PREFERENCES_RELATIVE_PATH)).unwrap();
        assert!(begin_preferences_protection(root.path(), true).is_err());
        assert!(begin_preferences_protection(root.path(), false)
            .unwrap()
            .backup
            .is_none());
    }
    #[cfg(unix)]
    #[test]
    fn symlinks_boundaries_and_multiple_payloads_are_rejected_without_touching_outside() {
        use std::os::unix::fs::symlink;
        let root = root();
        let outside = tempfile::tempdir().unwrap();
        let outside_file = outside.path().join("preferences.json");
        std::fs::write(&outside_file, b"secret").unwrap();
        let r = manual(root.path(), 100);
        let p = safety::payload_path(&r.directory).unwrap();
        std::fs::remove_file(&p).unwrap();
        symlink(&outside_file, &p).unwrap();
        assert!(list(root.path()).is_err());
        assert!(restore(root.path(), &r.manifest.backup_id, now()).is_err());
        std::fs::remove_file(&p).unwrap();
        std::fs::write(&p, active_payload(root.path()).unwrap()).unwrap();
        std::fs::write(r.directory.join("extra.json"), b"extra").unwrap();
        assert!(list(root.path()).is_err());
        std::fs::remove_file(r.directory.join("extra.json")).unwrap();
        let active = root.path().join(PREFERENCES_RELATIVE_PATH);
        std::fs::remove_file(&active).unwrap();
        symlink(&outside_file, &active).unwrap();
        assert!(create(root.path(), now(), BackupCleanupPolicy::Manual).is_err());
        assert!(restore(root.path(), &r.manifest.backup_id, now()).is_err());
        assert!(begin_preferences_protection(root.path(), true).is_err());
        assert_eq!(std::fs::read(outside_file).unwrap(), b"secret");
        assert!(safety::check_path(root.path(), &root.path().join("../outside"), true).is_err());
    }
    #[test]
    fn listing_and_restore_reads_are_bounded() {
        let root = root();
        let r = manual(root.path(), 1);
        std::fs::write(
            r.directory.join(super::super::MANIFEST_NAME),
            vec![b' '; 65537],
        )
        .unwrap();
        assert!(list(root.path()).is_err());
        let file = root.path().join(PREFERENCES_RELATIVE_PATH);
        std::fs::File::create(&file)
            .unwrap()
            .set_len(safety::MAX_PAYLOAD_BYTES + 1)
            .unwrap();
        assert!(begin_preferences_protection(root.path(), true).is_err());
        std::fs::remove_dir_all(root.path().join("backups")).unwrap();
        std::fs::create_dir(root.path().join("backups")).unwrap();
        for i in 0..=safety::MAX_ENTRIES {
            std::fs::write(root.path().join("backups").join(format!("file{i}")), b"").unwrap();
        }
        assert!(list(root.path()).is_err());
    }
}

#[cfg(test)]
mod pin_event_tests {
    use super::*;
    use crate::commands::event_delivery;
    use crate::modules::notifications::{
        persistence::{load_notifications, notifications_path},
        registry::{Channel, Delivery, Evidence, Job, Trigger},
        NotificationStore,
    };
    use std::sync::{Arc, Mutex};

    fn root() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        crate::commands::preferences::save_preferences_with_path(
            root.path(),
            &crate::modules::preferences::Preferences::default(),
        )
        .unwrap();
        root
    }
    fn record(root: &Path) -> BackupRecord {
        let dto = create(root, Utc::now(), Default::default()).unwrap();
        resolve(root, &dto.backup.backup_id).unwrap()
    }
    fn publish(root: &Path, store: &Arc<Mutex<NotificationStore>>, bytes: &[u8], succeeded: bool) {
        let identity =
            event_delivery::prepare(root, "preferences-backup-pin-failed", Channel::Local, bytes)
                .unwrap();
        let evidence = if succeeded {
            Evidence::Success {
                candidate: identity.candidate.clone(),
                verified: true,
            }
        } else {
            Evidence::Failure
        };
        let delivery = Delivery {
            event: if succeeded {
                "preferences-backup-pin-succeeded"
            } else {
                "preferences-backup-pin-failed"
            },
            job: Job::BackupPin,
            trigger: Trigger::User,
            identity,
            evidence,
            occurred_at: Utc::now(),
        };
        let _ = crate::commands::notifications::NotificationPublisher {
            store,
            data_root: root,
        }
        .publish_event(&delivery);
    }
    fn pin(
        root: &Path,
        id: &str,
        pinned: bool,
        store: &Arc<Mutex<NotificationStore>>,
    ) -> AppResult<()> {
        set_pinned_observed(root, id, pinned, |bytes, succeeded| {
            publish(root, store, bytes, succeeded)
        })
    }
    fn fail_write(root: &Path, record: &BackupRecord, store: &Arc<Mutex<NotificationStore>>) {
        let path = record.directory.join(super::super::MANIFEST_NAME);
        let original = std::fs::read(&path).unwrap();
        // Inject after bounded admission/resolve: actual write failure, not refusal.
        let result = set_pinned_with_writer(
            root,
            &record.manifest.backup_id,
            true,
            |root, candidate| {
                std::fs::remove_file(&path).unwrap();
                std::fs::create_dir(&path).unwrap();
                persist_pin(root, candidate)
            },
            |bytes, succeeded| publish(root, store, bytes, succeeded),
        );
        assert!(result.is_err());
        std::fs::remove_dir(&path).unwrap();
        std::fs::write(path, original).unwrap();
    }
    #[test]
    fn manifest_write_failure_and_exact_retry_recover_persisted_history() {
        let root = root();
        let record = record(root.path());
        let payload = std::fs::read(record.directory.join("preferences.json")).unwrap();
        let store = Arc::new(Mutex::new(NotificationStore::new()));
        fail_write(root.path(), &record, &store);
        let history = load_notifications(&notifications_path(root.path())).unwrap();
        assert_eq!(history.all().len(), 1);
        assert!(!history.all()[0].resolved);
        assert!(!history.all()[0]
            .body
            .contains(&root.path().to_string_lossy().to_string()));
        let reloaded = Arc::new(Mutex::new(history));
        pin(root.path(), &record.manifest.backup_id, true, &reloaded).unwrap();
        assert!(
            resolve(root.path(), &record.manifest.backup_id)
                .unwrap()
                .manifest
                .management
                .unwrap()
                .pinned
        );
        assert_eq!(
            std::fs::read(record.directory.join("preferences.json")).unwrap(),
            payload
        );
        let history = load_notifications(&notifications_path(root.path())).unwrap();
        assert_eq!(history.all().len(), 1);
        assert!(history.all()[0].resolved);
        assert!(!history.all()[0].read);
    }
    #[test]
    fn other_root_backup_and_unpin_cannot_clear_a_pin_failure() {
        let root = root();
        let other = self::root();
        let first = record(root.path());
        let second = record(root.path());
        let foreign = record(other.path());
        let store = Arc::new(Mutex::new(NotificationStore::new()));
        fail_write(root.path(), &first, &store);
        pin(other.path(), &foreign.manifest.backup_id, true, &store).unwrap();
        assert!(!store.lock().unwrap().all()[0].resolved);
        pin(root.path(), &second.manifest.backup_id, true, &store).unwrap();
        assert!(!store.lock().unwrap().all()[0].resolved);
        pin(root.path(), &first.manifest.backup_id, false, &store).unwrap();
        assert!(!store.lock().unwrap().all()[0].resolved);
        // Returning to original contents does not resurrect the old UUID.
        pin(root.path(), &first.manifest.backup_id, true, &store).unwrap();
        assert!(!store.lock().unwrap().all()[0].resolved);
    }
    #[test]
    fn refusal_never_emits_and_unpin_cannot_bypass_active_protection() {
        let root = root();
        let guard = begin_preferences_protection(root.path(), true).unwrap();
        let reference = guard.protection_ref(root.path()).unwrap().unwrap();
        drop(guard);
        set_pinned(root.path(), &reference.backup_id, true).unwrap();
        let before = resolve(root.path(), &reference.backup_id).unwrap();
        assert!(
            set_pinned_observed(root.path(), &reference.backup_id, false, |_, _| panic!(
                "active protection refused"
            ))
            .is_err()
        );
        assert_eq!(resolve(root.path(), &reference.backup_id).unwrap(), before);
        for id in ["../escape", "missing"] {
            assert!(set_pinned_observed(root.path(), id, true, |_, _| panic!(
                "invalid id delivery"
            ))
            .is_err());
        }
        // The cooperative lock waits for contention; test a genuine admission
        // refusal (invalid lock path), not recursive acquisition on this thread.
        let lock_path = root.path().join(".backup-w2.lock");
        std::fs::remove_file(&lock_path).unwrap();
        std::fs::create_dir(&lock_path).unwrap();
        assert!(
            set_pinned_observed(root.path(), &reference.backup_id, true, |_, _| panic!(
                "lock refused"
            ))
            .is_err()
        );
        assert!(!notifications_path(root.path()).exists());
    }
    #[test]
    fn nominal_writer_success_requires_exact_manifest_readback() {
        let root = root();
        let record = record(root.path());
        let mut observed = None;
        assert!(set_pinned_with_writer(
            root.path(),
            &record.manifest.backup_id,
            true,
            |_, _| Ok(()),
            |_, succeeded| observed = Some(succeeded)
        )
        .is_err());
        assert_eq!(observed, Some(false));
        assert!(
            !resolve(root.path(), &record.manifest.backup_id)
                .unwrap()
                .manifest
                .management
                .unwrap()
                .pinned
        );
    }
    #[test]
    fn notification_failure_does_not_undo_committed_pin() {
        let root = root();
        let record = record(root.path());
        let store = Arc::new(Mutex::new(NotificationStore::new()));
        fail_write(root.path(), &record, &store);
        let path = notifications_path(root.path());
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        pin(root.path(), &record.manifest.backup_id, true, &store).unwrap();
        assert!(
            resolve(root.path(), &record.manifest.backup_id)
                .unwrap()
                .manifest
                .management
                .unwrap()
                .pinned
        );
        assert!(!store.lock().unwrap().all()[0].resolved);
    }
}
