//! Registered feedback for backend-owned v2 operations. Mounted as feedback::v2.
//! This is an internal receipt projection, not another IPC DTO or toast engine.
//! Workers must keep Admission until the domain result AND this delivery settle.
use super::{OperationKind as ProgressKind, OperationPhase};
use crate::modules::model_config::{
    file_transfer_v2::{ExportOutputV2, FileTransferReceiptV2},
    schema::ModelResult,
    sync_local_v2::restore::{RestoreReceiptV2, RestoreStatusV2},
    sync_v2::types::{
        require_uuid, ErrorCode, LocalApplyStatus, OperationFact, OperationKind, Phase,
        RecoveryOutcome, RecoveryPlanKind, RecoveryReceipt, StorageStatus, SyncResult, WriteStep,
    },
};
use std::path::Path;

/// Only original UUID and fixed registry code cross into registered feedback.
/// The four sync facts stay in OperationFact: Stored/Applied is not a claim that
/// local content is unchanged or that the current remote head is still equal.
#[derive(Clone, Debug)]
pub(crate) struct EventFactV2 {
    event: &'static str,
    operation_id: String,
    progress: OperationPhase,
}
impl EventFactV2 {
    pub(crate) fn event(&self) -> &'static str {
        self.event
    }
    pub(crate) fn operation_id(&self) -> &str {
        &self.operation_id
    }
    pub(crate) fn progress(&self) -> OperationPhase {
        self.progress
    }
}
fn fact(event: &'static str, id: &str, progress: OperationPhase) -> Option<EventFactV2> {
    require_uuid(id).ok()?;
    super::OpaqueId::parse(id).ok()?;
    Some(EventFactV2 {
        event,
        operation_id: id.into(),
        progress,
    })
}

/// Pure interpretation of the real core enum. No Result::is_ok success shortcut,
/// relation rewrite, new ID, IO, clock, retry or result lookup is performed here.
pub(crate) fn classify_operation(operation: &OperationFact) -> Option<EventFactV2> {
    if operation.dto_version != 1 || operation.phase == Phase::CancelledBeforeWrite {
        return None;
    }
    // A prepared snapshot/catalog and a prepared download with a remote Stored
    // fact are not an executed upload or local application.
    if operation.phase == Phase::Prepared && operation.failure.is_none() {
        return None;
    }
    let applied = match operation.kind {
        OperationKind::Upload => operation.storage_status == StorageStatus::Stored,
        OperationKind::Download => operation.local_apply_status == LocalApplyStatus::Applied,
    };
    if applied {
        // Stored survives remote advancement/conflict/observation errors. The
        // generic registered body promises processing only, never "latest".
        return fact(
            "config-snapshot-sync-succeeded",
            &operation.operation_id,
            OperationPhase::Succeeded,
        );
    }
    if operation
        .failure
        .as_ref()
        .is_some_and(|f| f.code == ErrorCode::SyncCancelled)
    {
        return None;
    }
    let pending = match operation.kind {
        OperationKind::Upload => {
            operation.storage_status == StorageStatus::Unknown
                || operation.last_write_step != WriteStep::None
                    && operation.storage_status != StorageStatus::Invalid
        }
        OperationKind::Download => operation.local_apply_status == LocalApplyStatus::Unknown,
    };
    if pending {
        // Existing snapshot_sync_failure is a persistent warning with the
        // transfer_result action. Review keeps the original O pending; it is
        // deliberately not the successful progress terminal.
        return fact(
            "config-snapshot-sync-failed",
            &operation.operation_id,
            OperationPhase::Review,
        );
    }
    if operation.failure.is_some()
        || operation.storage_status == StorageStatus::Invalid
        || operation.local_apply_status == LocalApplyStatus::Invalid
    {
        return fact(
            "config-snapshot-sync-failed",
            &operation.operation_id,
            OperationPhase::Failed,
        );
    }
    None
}
fn classify_sync_result(
    result: &SyncResult<OperationFact>,
    original_operation_id: Option<&str>,
) -> Option<EventFactV2> {
    match result {
        Ok(operation) => {
            // A caller holding original O cannot accidentally announce another
            // operation returned by a stale observer or a different connection.
            if original_operation_id.is_some_and(|id| id != operation.operation_id) {
                return None;
            }
            classify_operation(operation)
        }
        Err(error) => {
            if error.code == ErrorCode::SyncCancelled {
                return None;
            }
            let id = original_operation_id?;
            fact(
                "config-snapshot-sync-failed",
                id,
                if error.code == ErrorCode::SyncOperationUnknown {
                    OperationPhase::Review
                } else {
                    OperationPhase::Failed
                },
            )
        }
    }
}
fn receipt_matches(receipt: &FileTransferReceiptV2, original: &str) -> bool {
    receipt.operation_id == original
        && receipt.length > 0
        && crate::modules::model_config::schema_v2::valid_digest(&receipt.plan_digest)
        && crate::modules::model_config::schema_v2::valid_digest(&receipt.file_digest)
}
fn classify_file_export(
    id: &str,
    result: &ModelResult<Option<ExportOutputV2>>,
) -> Option<EventFactV2> {
    match result {
        Ok(None) => None, // Native folder cancel: no execution, no terminal event.
        Ok(Some(output)) if output.operation_id == id && receipt_matches(&output.receipt, id) => {
            fact(
                "config-snapshot-export-succeeded",
                id,
                OperationPhase::Succeeded,
            )
        }
        Ok(Some(_)) => None, // An incoherent/foreign receipt grants no success.
        Err(_) => fact("config-snapshot-export-failed", id, OperationPhase::Failed),
    }
}
fn classify_import(id: &str, result: &ModelResult<FileTransferReceiptV2>) -> Option<EventFactV2> {
    match result {
        Ok(receipt) if receipt_matches(receipt, id) => fact(
            "config-snapshot-import-succeeded",
            id,
            OperationPhase::Succeeded,
        ),
        Ok(_) => None,
        Err(_) => fact("config-snapshot-import-failed", id, OperationPhase::Failed),
    }
}
fn emit(root: &Path, classified: Option<EventFactV2>) {
    let Some(event) = classified else { return };
    // Child access reuses the real configured AppHandle and parent's registered
    // publisher. Candidate=original O resolves only its own previous failure.
    super::receipt_event(
        root,
        event.event(),
        event.operation_id(),
        event.operation_id(),
    );
    super::progress(
        root,
        event.operation_id(),
        ProgressKind::Transfer,
        event.progress(),
    );
}

/// Call inside save's owned worker only after export_plan(original O) succeeded.
/// A native FolderCancel is Ok(None). Preview/auth/invalid-plan errors stay in
/// the dialog: do not call this function when no backend original O was issued.
pub(crate) fn emit_file_export(
    root: &Path,
    id: &str,
    result: &ModelResult<Option<ExportOutputV2>>,
) {
    emit(root, classify_file_export(id, result));
}
/// Call for actual confirm/resume of the original prepared import, never for
/// inventory, preview, no-change, cancellation, or a failed plan lookup.
pub(crate) fn emit_import(root: &Path, id: &str, result: &ModelResult<FileTransferReceiptV2>) {
    emit(root, classify_import(id, result));
}
/// fallback must be the backend-issued original O, not a session/UI request ID.
/// None on preview/no-change/auth errors means no operation failure is published.
/// Verify/resume may settle the original fact, but result/catalog reads must not
/// call this function; nothing here starts another operation or sends Running.
pub(crate) fn emit_sync(
    root: &Path,
    result: &SyncResult<OperationFact>,
    fallback_operation_id: Option<&str>,
) {
    emit(root, classify_sync_result(result, fallback_operation_id));
}
/// Emit only after the real open-directory call returned true for the original
/// export. No new candidate, export success or transfer progress is invented.
pub(crate) fn emit_export_directory_opened(root: &Path, id: &str, result: &ModelResult<bool>) {
    if let Some(event) = classify_directory_opened(id, result) {
        super::receipt_event(
            root,
            event.event(),
            event.operation_id(),
            event.operation_id(),
        );
    }
}
fn classify_directory_opened(id: &str, result: &ModelResult<bool>) -> Option<EventFactV2> {
    if !matches!(result, Ok(true)) {
        return None;
    }
    fact(
        "config-export-directory-opened",
        id,
        OperationPhase::Succeeded,
    )
}

/// Dedicated recovery settles its own recoveryPlanId. Neither the source head O,
/// artifact ID nor a session/UI ID can replace this original backend authority.
pub(crate) fn emit_recovery(
    root: &Path,
    result: &SyncResult<RecoveryReceipt>,
    original_recovery_plan_id: &str,
) {
    emit(root, classify_recovery(result, original_recovery_plan_id));
}
fn classify_recovery(result: &SyncResult<RecoveryReceipt>, original: &str) -> Option<EventFactV2> {
    match result {
        Ok(receipt) => {
            if receipt.dto_version != 1 || receipt.recovery_plan_id != original {
                return None;
            }
            let (event, phase) = match (receipt.kind, receipt.outcome) {
                (RecoveryPlanKind::RecoveryDownload, RecoveryOutcome::Applied)
                | (RecoveryPlanKind::RecoveryExport, RecoveryOutcome::Exported) => {
                    ("config-snapshot-sync-succeeded", OperationPhase::Succeeded)
                }
                (_, RecoveryOutcome::Unknown) => {
                    ("config-snapshot-sync-failed", OperationPhase::Review)
                }
                _ => ("config-snapshot-sync-failed", OperationPhase::Failed),
            };
            fact(event, original, phase)
        }
        Err(error) if error.code == ErrorCode::SyncCancelled => None,
        Err(error) => fact(
            "config-snapshot-sync-failed",
            original,
            if error.code == ErrorCode::SyncOperationUnknown {
                OperationPhase::Review
            } else {
                OperationPhase::Failed
            },
        ),
    }
}
/// Scoped backup restoration has no registered dedicated success policy. Do not
/// announce "configuration imported" for a restore: only settle the existing
/// restore progress card, retaining Partial as Review under original restore O.
/// Call only after the backend restore plan resolved, never on preview/no-plan.
pub(crate) fn emit_scope_restore(
    root: &Path,
    result: &ModelResult<RestoreReceiptV2>,
    original_restore_id: &str,
) {
    if let Some(phase) = scope_restore_phase(result, original_restore_id) {
        super::progress(root, original_restore_id, ProgressKind::Transfer, phase);
    }
}
fn scope_restore_phase(
    result: &ModelResult<RestoreReceiptV2>,
    original: &str,
) -> Option<OperationPhase> {
    require_uuid(original).ok()?;
    match result {
        Ok(receipt) if receipt.restore_id != original => None,
        Ok(receipt) => Some(match receipt.status {
            RestoreStatusV2::Applied => OperationPhase::Succeeded,
            RestoreStatusV2::Partial => OperationPhase::Review,
        }),
        Err(_) => Some(OperationPhase::Failed),
    }
}

#[cfg(test)]
#[path = "feedback_v2_tests.rs"]
mod tests;
