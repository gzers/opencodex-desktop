//! Real core facts plus the real registry, publisher, persistence and store.
use super::*;
use crate::{
    commands::notifications::NotificationPublisher,
    modules::{
        model_config::{schema::invalid, schema_v2::digest, sync_v2::types::*},
        notifications::{
            persistence::{load_notifications, notifications_path},
            registry::{self, Channel, Delivery, EventIdentity, Evidence, OpaqueId, Trigger},
            NotificationAction, NotificationLevel, NotificationStore,
        },
    },
};
use std::sync::{Arc, Mutex};
const O: &str = "11111111-1111-4111-8111-111111111111";
const OTHER: &str = "22222222-2222-4222-8222-222222222222";
fn operation(kind: OperationKind) -> OperationFact {
    OperationFact {
        dto_version: 1,
        operation_id: O.into(),
        kind,
        phase: Phase::Prepared,
        last_write_step: WriteStep::None,
        storage_status: if kind == OperationKind::Download {
            StorageStatus::Stored
        } else {
            StorageStatus::NotAttempted
        },
        local_apply_status: if kind == OperationKind::Download {
            LocalApplyStatus::NotAttempted
        } else {
            LocalApplyStatus::NotApplicable
        },
        relation: Relation {
            local: LocalRelation::NotCompared,
            remote: RemoteRelation::Unknown,
        },
        generation: None,
        payload_digest: "sensitive-payload-digest-not-for-notifications".into(),
        commit: None,
        stored_at: None,
        observation: None,
        failure: None,
        recovery_actions: vec![],
    }
}
fn stored() -> OperationFact {
    let mut f = operation(OperationKind::Upload);
    f.phase = Phase::Settled;
    f.storage_status = StorageStatus::Stored;
    f.last_write_step = WriteStep::Commit;
    f
}
fn applied() -> OperationFact {
    let mut f = operation(OperationKind::Download);
    f.phase = Phase::Settled;
    f.local_apply_status = LocalApplyStatus::Applied;
    f
}
fn unknown() -> OperationFact {
    let mut f = operation(OperationKind::Upload);
    f.phase = Phase::CommitMayBeWritten;
    f.storage_status = StorageStatus::Unknown;
    f.last_write_step = WriteStep::Commit;
    f
}
fn progress_is(event: &EventFactV2, expected: OperationPhase) {
    assert_eq!(
        serde_json::to_value(event.progress()).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
}
fn receipt() -> FileTransferReceiptV2 {
    FileTransferReceiptV2 {
        operation_id: O.into(),
        plan_digest: digest(b"plan"),
        file_digest: digest(b"file"),
        length: 4,
    }
}
fn output() -> ExportOutputV2 {
    ExportOutputV2 {
        operation_id: O.into(),
        receipt: receipt(),
        path: "/sensitive/secret-name.ocxdconf".into(),
        directory: "/sensitive".into(),
        filename: "secret-name.ocxdconf".into(),
    }
}
fn store() -> crate::state::SharedNotificationStore {
    Arc::new(Mutex::new(NotificationStore::new()))
}
// Opt-in handoff evidence copies the actual persisted notification bytes after
// real publisher/readback assertions; it never replaces the store or publisher.
fn persisted(root: &Path, label: &str) -> NotificationStore {
    let path = notifications_path(root);
    let disk = load_notifications(&path).unwrap();
    if let Some(directory) = std::env::var_os("OCXD_FEEDBACK_V2_EVIDENCE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        assert!(directory.is_absolute());
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::copy(path, directory.join(format!("{label}.json"))).unwrap();
    }
    disk
}
fn publish(
    root: &Path,
    store: &crate::state::SharedNotificationStore,
    fact: &EventFactV2,
    second: i64,
) -> registry::DeliveryOutcome {
    let definition = registry::lookup(fact.event()).unwrap();
    let id = OpaqueId::parse(fact.operation_id()).unwrap();
    let identity = EventIdentity {
        object: definition.object,
        object_id: id.clone(),
        action: definition.action,
        phase: definition.phase,
        channel: Channel::Local,
        candidate: id.clone(),
    };
    let evidence = match definition.fact {
        registry::Fact::Success => Evidence::Success {
            candidate: id,
            verified: true,
        },
        registry::Fact::Failure => Evidence::Failure,
        registry::Fact::Risk => Evidence::Risk,
        registry::Fact::Signal => Evidence::Signal,
    };
    NotificationPublisher {
        store,
        data_root: root,
    }
    .publish_event(&Delivery {
        event: fact.event(),
        job: definition.job,
        trigger: Trigger::User,
        identity,
        evidence,
        occurred_at: chrono::DateTime::from_timestamp(1_800_000_000 + second, 0).unwrap(),
        detail: None,
    })
    .unwrap()
}
#[test]
fn actual_upload_stored_is_success_and_pending_is_review_not_success() {
    let success = classify_operation(&stored()).unwrap();
    assert_eq!(success.event(), "config-snapshot-sync-succeeded");
    progress_is(&success, OperationPhase::Succeeded);
    let pending = classify_operation(&unknown()).unwrap();
    assert_eq!(pending.event(), "config-snapshot-sync-failed");
    progress_is(&pending, OperationPhase::Review);
}
#[test]
fn actual_download_remote_stored_is_not_local_applied() {
    assert_eq!(
        classify_operation(&applied()).unwrap().event(),
        "config-snapshot-sync-succeeded"
    );
    let mut f = operation(OperationKind::Download);
    assert!(classify_operation(&f).is_none());
    f.phase = Phase::Settled;
    assert!(classify_operation(&f).is_none());
    f.local_apply_status = LocalApplyStatus::Unknown;
    let event = classify_operation(&f).unwrap();
    assert_eq!(event.event(), "config-snapshot-sync-failed");
    progress_is(&event, OperationPhase::Review);
    f.local_apply_status = LocalApplyStatus::Applied;
    assert_eq!(
        classify_operation(&f).unwrap().event(),
        "config-snapshot-sync-succeeded"
    );
}
#[test]
fn upload_cannot_borrow_download_apply_or_download_borrow_upload_store() {
    let mut f = operation(OperationKind::Upload);
    f.phase = Phase::Settled;
    f.local_apply_status = LocalApplyStatus::Applied;
    assert!(classify_operation(&f).is_none());
    let mut f = operation(OperationKind::Download);
    f.phase = Phase::Settled;
    f.storage_status = StorageStatus::Stored;
    assert!(classify_operation(&f).is_none());
}
#[test]
fn four_facts_remain_independent_under_remote_advance_conflict_and_current_unknown() {
    for remote in [
        RemoteRelation::Unknown,
        RemoteRelation::ObservedEqual,
        RemoteRelation::Descendant,
        RemoteRelation::Conflict,
        RemoteRelation::GenerationChanged,
        RemoteRelation::InitializationAmbiguous,
    ] {
        for local in [
            LocalRelation::NotCompared,
            LocalRelation::ObservedEqual,
            LocalRelation::Changed,
            LocalRelation::Unknown,
        ] {
            let mut f = stored();
            f.relation = Relation { local, remote };
            f.failure = Some(failure(ErrorCode::SyncNetwork, Step::Observe));
            let before = f.clone();
            let event = classify_operation(&f).unwrap();
            assert_eq!(event.event(), "config-snapshot-sync-succeeded");
            assert_eq!(f, before);
        }
    }
}
#[test]
fn cancellation_and_prepared_nochange_have_no_operation_receipt_event() {
    for kind in [OperationKind::Upload, OperationKind::Download] {
        let mut f = operation(kind);
        assert!(classify_operation(&f).is_none());
        f.phase = Phase::CancelledBeforeWrite;
        f.failure = Some(failure(ErrorCode::SyncCancelled, Step::SourceGuard));
        assert!(classify_operation(&f).is_none());
    }
    assert!(classify_sync_result(
        &Err(failure(ErrorCode::SyncCancelled, Step::SourceGuard)),
        Some(O)
    )
    .is_none());
}
#[test]
fn upload_possible_generation_or_payload_write_is_never_success_without_stored() {
    for step in [WriteStep::Generation, WriteStep::Payload, WriteStep::Commit] {
        let mut f = operation(OperationKind::Upload);
        f.last_write_step = step;
        f.phase = Phase::PayloadMayBeWritten;
        let e = classify_operation(&f).unwrap();
        assert_eq!(e.event(), "config-snapshot-sync-failed");
        progress_is(&e, OperationPhase::Review);
    }
}
#[test]
fn actual_invalid_status_is_terminal_failure_not_pending_or_success() {
    let mut f = operation(OperationKind::Upload);
    f.phase = Phase::Settled;
    f.storage_status = StorageStatus::Invalid;
    progress_is(&classify_operation(&f).unwrap(), OperationPhase::Failed);
    let mut f = operation(OperationKind::Download);
    f.phase = Phase::Settled;
    f.local_apply_status = LocalApplyStatus::Invalid;
    progress_is(&classify_operation(&f).unwrap(), OperationPhase::Failed);
}
#[test]
fn malformed_or_future_fact_cannot_publish_owned_identity() {
    for id in [
        "/sensitive/path",
        "https://secret.invalid",
        "nil",
        "00000000-0000-0000-0000-000000000000",
    ] {
        let mut f = stored();
        f.operation_id = id.into();
        assert!(classify_operation(&f).is_none());
    }
    let mut f = stored();
    f.dto_version = 2;
    assert!(classify_operation(&f).is_none());
}
#[test]
fn raw_failure_without_backend_original_operation_remains_in_dialog() {
    for code in [
        ErrorCode::SyncAuthRequired,
        ErrorCode::SyncPassphraseInvalid,
        ErrorCode::SyncPlanChanged,
        ErrorCode::SyncNetwork,
    ] {
        let r = Err(failure(code, Step::Decode));
        assert!(classify_sync_result(&r, None).is_none());
    }
    assert!(classify_sync_result(&Ok(stored()), Some(OTHER)).is_none());
    assert!(classify_sync_result(
        &Err(failure(ErrorCode::SyncNetwork, Step::Observe)),
        Some("request-id")
    )
    .is_none());
}
#[test]
fn original_failure_uses_original_uuid_and_unknown_result_is_review() {
    let r = Err(failure(ErrorCode::SyncNetwork, Step::CommitPut));
    let e = classify_sync_result(&r, Some(O)).unwrap();
    assert_eq!(e.operation_id(), O);
    progress_is(&e, OperationPhase::Failed);
    let e = classify_sync_result(
        &Err(failure(ErrorCode::SyncOperationUnknown, Step::Verify)),
        Some(O),
    )
    .unwrap();
    progress_is(&e, OperationPhase::Review);
}
#[test]
fn file_success_requires_actual_matching_receipt_and_folder_cancel_is_silent() {
    assert!(classify_file_export(O, &Ok(None)).is_none());
    let o = output();
    assert_eq!(
        classify_file_export(O, &Ok(Some(o.clone())))
            .unwrap()
            .event(),
        "config-snapshot-export-succeeded"
    );
    let mut bad = o.clone();
    bad.receipt.operation_id = OTHER.into();
    assert!(classify_file_export(O, &Ok(Some(bad))).is_none());
    let mut bad = o;
    bad.operation_id = OTHER.into();
    assert!(classify_file_export(O, &Ok(Some(bad))).is_none());
    let mut bad = receipt();
    bad.file_digest = "not receipt evidence".into();
    assert!(classify_import(O, &Ok(bad)).is_none());
    assert_eq!(
        classify_import(O, &Ok(receipt())).unwrap().event(),
        "config-snapshot-import-succeeded"
    );
}
#[test]
fn registry_pending_warning_persists_original_result_action_and_dedupes_same_candidate() {
    let root = tempfile::tempdir().unwrap();
    let s = store();
    let pending = classify_operation(&unknown()).unwrap();
    publish(root.path(), &s, &pending, 0);
    publish(root.path(), &s, &pending, 1);
    let disk = persisted(root.path(), "pending-warning-deduped");
    assert_eq!(disk.all().len(), 1);
    let n = &disk.all()[0];
    assert_eq!(n.level, NotificationLevel::Warning);
    assert_eq!(n.action_ref, Some(NotificationAction::TransferResult));
    assert!(!n.resolved);
    assert_eq!(
        n.event_identity.as_ref().unwrap().object_id,
        OpaqueId::parse(O).unwrap()
    );
    assert_eq!(
        n.event_identity.as_ref().unwrap().candidate,
        OpaqueId::parse(O).unwrap()
    );
    assert!(n.notification_id.contains(O));
}
#[test]
fn registry_same_operation_success_resolves_only_its_own_failure_not_other_o() {
    let root = tempfile::tempdir().unwrap();
    let s = store();
    let pending = classify_operation(&unknown()).unwrap();
    publish(root.path(), &s, &pending, 0);
    let mut other = unknown();
    other.operation_id = OTHER.into();
    publish(root.path(), &s, &classify_operation(&other).unwrap(), 1);
    let result = publish(root.path(), &s, &classify_operation(&stored()).unwrap(), 2);
    assert_eq!(result.resolved, 1);
    let disk = persisted(root.path(), "same-o-resolved-other-o-retained");
    assert!(disk.all().iter().any(|n| n
        .notification_id
        .starts_with("config-snapshot-sync-failed:")
        && n.notification_id.contains(O)
        && n.resolved));
    assert!(disk
        .all()
        .iter()
        .any(|n| n.notification_id.contains(OTHER) && !n.resolved));
    assert!(disk.all().iter().any(|n| n
        .notification_id
        .starts_with("config-snapshot-sync-succeeded:")
        && n.notification_id.contains(O)));
}
#[test]
fn registry_export_import_failure_then_receipt_success_resolves_same_candidate() {
    let root = tempfile::tempdir().unwrap();
    let s = store();
    let export_failed = classify_file_export(
        O,
        &Err(invalid(
            "raw-secret-code",
            "secret URL https://u:p@host/path",
        )),
    )
    .unwrap();
    publish(root.path(), &s, &export_failed, 0);
    publish(root.path(), &s, &export_failed, 1);
    assert_eq!(
        publish(
            root.path(),
            &s,
            &classify_file_export(O, &Ok(Some(output()))).unwrap(),
            2
        )
        .resolved,
        1
    );
    let import_failed = classify_import(O, &Err(invalid("secret-path", "secret-payload"))).unwrap();
    publish(root.path(), &s, &import_failed, 3);
    assert_eq!(
        publish(
            root.path(),
            &s,
            &classify_import(O, &Ok(receipt())).unwrap(),
            4
        )
        .resolved,
        1
    );
    let disk = persisted(root.path(), "file-export-import-receipts-resolved");
    assert!(disk
        .all()
        .iter()
        .filter(|n| n.notification_id.contains("failed:"))
        .all(|n| n.resolved));
    let text = std::fs::read_to_string(notifications_path(root.path())).unwrap();
    for secret in [
        "secret",
        "https://",
        "/sensitive",
        "raw-secret-code",
        "payload-digest",
    ] {
        assert!(!text.contains(secret));
    }
}
#[test]
fn registry_success_body_never_claims_latest_or_equal_for_unknown_current_relation() {
    let root = tempfile::tempdir().unwrap();
    let s = store();
    let mut f = stored();
    f.relation.remote = RemoteRelation::Unknown;
    f.relation.local = LocalRelation::Changed;
    publish(root.path(), &s, &classify_operation(&f).unwrap(), 0);
    let disk = persisted(root.path(), "stored-current-unknown");
    let n = &disk.all()[0];
    assert_eq!(
        n.body,
        registry::registered_notification("config-snapshot-sync-succeeded", chrono::Utc::now())
            .unwrap()
            .body
    );
    for claim in [
        "最新",
        "一致",
        "latest",
        "observed_equal",
        "sensitive-payload",
    ] {
        assert!(!n.body.contains(claim));
    }
}
#[test]
fn readonly_or_nonowned_inputs_do_not_manufacture_event_or_touch_store() {
    let root = tempfile::tempdir().unwrap();
    let s = store();
    for input in [
        Ok(operation(OperationKind::Upload)),
        Ok(operation(OperationKind::Download)),
        Err(failure(ErrorCode::SyncPlanChanged, Step::SourceGuard)),
    ] {
        assert!(classify_sync_result(&input, None).is_none());
    }
    assert!(s.lock().unwrap().all().is_empty());
    assert!(!notifications_path(root.path()).exists());
    // No AppHandle configured: production entry points must remain harmless,
    // not create a second persistence or emit a fake task to test-only sinks.
    emit_file_export(root.path(), O, &Ok(None));
    emit_sync(root.path(), &Ok(stored()), Some(O));
    emit_import(root.path(), O, &Ok(receipt()));
    emit_recovery(
        root.path(),
        &Ok(recovery(
            RecoveryPlanKind::RecoveryDownload,
            RecoveryOutcome::Applied,
        )),
        O,
    );
    emit_export_directory_opened(root.path(), O, &Ok(true));
    assert!(!notifications_path(root.path()).exists());
}
#[test]
fn registered_directory_open_event_is_separate_from_export_result_and_same_original_o() {
    let root = tempfile::tempdir().unwrap();
    let s = store();
    assert!(classify_directory_opened(O, &Ok(false)).is_none());
    assert!(classify_directory_opened(O, &Err(invalid("secret", "raw secret"))).is_none());
    let event = classify_directory_opened(O, &Ok(true)).unwrap();
    publish(root.path(), &s, &event, 0);
    publish(root.path(), &s, &event, 1);
    let disk = persisted(root.path(), "directory-open-original-o");
    assert_eq!(disk.all().len(), 1);
    assert!(disk.all()[0]
        .notification_id
        .starts_with("config-export-directory-opened:"));
    assert_eq!(
        disk.all()[0].event_identity.as_ref().unwrap().candidate,
        OpaqueId::parse(O).unwrap()
    );
}

fn recovery(kind: RecoveryPlanKind, outcome: RecoveryOutcome) -> RecoveryReceipt {
    RecoveryReceipt {
        dto_version: 1,
        recovery_plan_id: O.into(),
        kind,
        outcome,
        source_binding_id: "sensitive-source-binding-digest".into(),
        generation: GenerationRef {
            generation_id: OTHER.into(),
            digest: "sensitive-generation-digest".into(),
        },
        head: CommitRef {
            operation_id: OTHER.into(),
            digest: "sensitive-head-digest".into(),
        },
        payload_digest: "sensitive-payload-digest".into(),
        artifact_id: Some(OTHER.into()),
        finished_at: "2026-11-07T00:00:00Z".into(),
        failure: None,
        recovery_actions: vec![],
    }
}
#[test]
fn dedicated_recovery_real_outcome_and_kind_settle_only_original_plan_id() {
    for (kind, outcome) in [
        (RecoveryPlanKind::RecoveryDownload, RecoveryOutcome::Applied),
        (RecoveryPlanKind::RecoveryExport, RecoveryOutcome::Exported),
    ] {
        let result = Ok(recovery(kind, outcome));
        let classified = classify_recovery(&result, O).unwrap();
        assert_eq!(classified.event(), "config-snapshot-sync-succeeded");
        assert_eq!(classified.operation_id(), O);
        progress_is(&classified, OperationPhase::Succeeded);
        assert!(classify_recovery(&result, OTHER).is_none());
        assert!(classify_recovery(&result, "ui-session-id").is_none());
    }
    for (kind, outcome) in [
        (
            RecoveryPlanKind::RecoveryDownload,
            RecoveryOutcome::Exported,
        ),
        (RecoveryPlanKind::RecoveryExport, RecoveryOutcome::Applied),
    ] {
        let classified = classify_recovery(&Ok(recovery(kind, outcome)), O).unwrap();
        assert_eq!(classified.event(), "config-snapshot-sync-failed");
        progress_is(&classified, OperationPhase::Failed);
    }
}
#[test]
fn dedicated_recovery_unknown_warning_then_real_receipt_resolves_original_bell_without_digest_leak()
{
    let root = tempfile::tempdir().unwrap();
    let s = store();
    let unknown_receipt = Ok(recovery(
        RecoveryPlanKind::RecoveryDownload,
        RecoveryOutcome::Unknown,
    ));
    let pending = classify_recovery(&unknown_receipt, O).unwrap();
    progress_is(&pending, OperationPhase::Review);
    publish(root.path(), &s, &pending, 0);
    publish(root.path(), &s, &pending, 1);
    let disk = persisted(root.path(), "recovery-unknown-warning");
    assert_eq!(disk.all().len(), 1);
    assert_eq!(disk.all()[0].level, NotificationLevel::Warning);
    assert!(!disk.all()[0].resolved);
    let failed = classify_recovery(
        &Ok(recovery(
            RecoveryPlanKind::RecoveryDownload,
            RecoveryOutcome::Failed,
        )),
        O,
    )
    .unwrap();
    progress_is(&failed, OperationPhase::Failed);
    assert_eq!(failed.event(), "config-snapshot-sync-failed");
    let success = classify_recovery(
        &Ok(recovery(
            RecoveryPlanKind::RecoveryDownload,
            RecoveryOutcome::Applied,
        )),
        O,
    )
    .unwrap();
    assert_eq!(publish(root.path(), &s, &success, 2).resolved, 1);
    let disk = persisted(root.path(), "recovery-original-plan-resolved");
    assert!(disk
        .all()
        .iter()
        .all(|n| n.event_identity.as_ref().unwrap().candidate == OpaqueId::parse(O).unwrap()));
    let bytes = std::fs::read_to_string(notifications_path(root.path())).unwrap();
    assert!(!bytes.contains("sensitive"));
    assert!(!bytes.contains(OTHER));
    assert!(!bytes.contains("最新"));
    assert!(!bytes.contains("latest"));
    let pending_error = classify_recovery(
        &Err(failure(ErrorCode::SyncOperationUnknown, Step::Recovery)),
        O,
    )
    .unwrap();
    progress_is(&pending_error, OperationPhase::Review);
}
#[test]
fn scoped_restore_real_status_preserves_original_restore_progress_without_import_title_or_new_notice(
) {
    let root = tempfile::tempdir().unwrap();
    let receipt = RestoreReceiptV2 {
        restore_id: O.into(),
        original_operation_id: OTHER.into(),
        groups: vec!["secret-private-path".into()],
        status: RestoreStatusV2::Partial,
        completed_groups: vec![],
        refresh_file_snapshot: false,
        refresh_sync_scope: false,
        failure_code: Some("raw-secret-error".into()),
    };
    assert_eq!(
        serde_json::to_value(scope_restore_phase(&Ok(receipt.clone()), O).unwrap()).unwrap(),
        serde_json::to_value(OperationPhase::Review).unwrap()
    );
    assert!(scope_restore_phase(&Ok(receipt.clone()), OTHER).is_none());
    emit_scope_restore(root.path(), &Ok(receipt.clone()), O);
    let mut applied = receipt;
    applied.status = RestoreStatusV2::Applied;
    applied.failure_code = None;
    assert_eq!(
        serde_json::to_value(scope_restore_phase(&Ok(applied.clone()), O).unwrap()).unwrap(),
        serde_json::to_value(OperationPhase::Succeeded).unwrap()
    );
    emit_scope_restore(root.path(), &Ok(applied), O);
    assert_eq!(
        serde_json::to_value(scope_restore_phase(&Err(invalid("secret", "secret")), O).unwrap())
            .unwrap(),
        serde_json::to_value(OperationPhase::Failed).unwrap()
    );
    assert!(scope_restore_phase(&Err(invalid("secret", "secret")), "ui-id").is_none());
    assert!(!notifications_path(root.path()).exists());
}
