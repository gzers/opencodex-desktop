//! Model operations publish registered facts; payloads contain no prompt/config.
use crate::modules::notifications::registry::{
    self, Action, Channel, EventIdentity, Job, ObjectKind, OpaqueId, Phase, Trigger,
};
use serde::Serialize;
use std::{path::Path, sync::OnceLock};
static APP: OnceLock<tauri::AppHandle> = OnceLock::new();
pub fn configure(app: tauri::AppHandle) {
    let _ = APP.set(app);
}
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    Generation,
    Research,
    ExternalSubmit,
    PolicySave,
    Transfer,
    NodeMeasurement,
}
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationPhase {
    Running,
    Review,
    Succeeded,
    Failed,
    Stopped,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Signal<'a> {
    id: &'a str,
    kind: OperationKind,
    phase: OperationPhase,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_id: Option<&'a str>,
}
pub fn progress(root: &Path, id: &str, kind: OperationKind, phase: OperationPhase) {
    progress_linked(root, id, kind, phase, None);
}
fn progress_linked(
    root: &Path,
    id: &str,
    kind: OperationKind,
    phase: OperationPhase,
    previous_id: Option<&str>,
) {
    let Some(app) = APP.get() else {
        return;
    };
    if OpaqueId::parse(id).is_err()
        || previous_id.is_some_and(|previous| OpaqueId::parse(previous).is_err())
    {
        return;
    }
    if crate::commands::event_delivery::emit_signal(
        app,
        "model-operation-progress",
        Job::ModelConfig,
        Trigger::Commit,
        Channel::Local,
        Signal {
            id,
            kind,
            phase,
            previous_id,
        },
    )
    .is_err()
    {
        diagnostic(root);
    }
}
pub fn external_terminal(root: &Path, task: &str, submission: &str, success: bool) {
    let Some(app) = APP.get() else {
        return;
    };
    let Ok(object_id) = OpaqueId::parse(task) else {
        return;
    };
    let Ok(candidate) = OpaqueId::parse(submission) else {
        return;
    };
    let identity = EventIdentity {
        object: ObjectKind::Configuration,
        object_id,
        action: Action::Save,
        phase: Phase::Execution,
        channel: Channel::Local,
        candidate,
    };
    crate::commands::event_delivery::publish(
        app,
        root,
        if success {
            "model-template-external-succeeded"
        } else {
            "model-template-external-failed"
        },
        Some(identity),
        Trigger::Commit,
    );
}
fn diagnostic(root: &Path) {
    let _ = crate::infrastructure::runtime_log::RuntimeLog::new(root)
        .append_event("registered model operation delivery failed");
}
pub fn policy_terminal(root: &Path, id: &str, success: bool) {
    let Some(app) = APP.get() else {
        return;
    };
    let Ok(object_id) = OpaqueId::parse(id) else {
        return;
    };
    let identity = EventIdentity {
        object: ObjectKind::Configuration,
        object_id: object_id.clone(),
        action: Action::Save,
        phase: Phase::Execution,
        channel: Channel::Local,
        candidate: object_id,
    };
    crate::commands::event_delivery::publish(
        app,
        root,
        if success {
            "model-policy-save-succeeded"
        } else {
            "model-policy-save-failed"
        },
        Some(identity),
        Trigger::Commit,
    );
}

pub use crate::modules::notifications::registry::BlockedReason as SaveBlockedReason;
/// 每次确认都产生一次明确事实；同一原因按固定 dedupe 更新同一条通知。
pub fn save_blocked(app: &tauri::AppHandle, root: &Path, reason: SaveBlockedReason) {
    let _ = crate::infrastructure::runtime_log::RuntimeLog::new(root)
        .append_event(&format!("model policy save blocked: {}", reason.text()));
    let identity = crate::commands::event_delivery::prepare(
        root,
        "model-policy-save-blocked",
        Channel::Local,
        &[reason as u8],
    );
    crate::commands::event_delivery::publish_detail(
        app,
        root,
        "model-policy-save-blocked",
        identity,
        Trigger::User,
        Some(crate::modules::notifications::registry::Detail::ModelPolicyBlocked(reason)),
    );
}
pub fn registered() -> bool {
    registry::validate_signal(
        "model-operation-progress",
        Job::ModelConfig,
        Trigger::Commit,
        Channel::Local,
    )
    .is_ok()
}
pub fn transfer<T>(
    root: &Path,
    id: &str,
    review: bool,
    action: impl FnOnce() -> super::schema::ModelResult<T>,
) -> super::schema::ModelResult<T> {
    progress(root, id, OperationKind::Transfer, OperationPhase::Running);
    let result = action();
    progress(
        root,
        id,
        OperationKind::Transfer,
        if result.is_err() {
            OperationPhase::Failed
        } else if review {
            OperationPhase::Review
        } else {
            OperationPhase::Succeeded
        },
    );
    result
}

/// Receiving creates its durable identity only after the file/remote document
/// validates. Rebind the transient observation to that exact receipt, so later
/// adoption/retries update it instead of leaving a permanent review card.
pub fn receive_transfer(
    root: &Path,
    action: impl FnOnce() -> super::schema::ModelResult<super::transfer::IncomingPlan>,
) -> super::schema::ModelResult<super::transfer::IncomingPlan> {
    let pending = uuid::Uuid::new_v4().to_string();
    progress(
        root,
        &pending,
        OperationKind::Transfer,
        OperationPhase::Running,
    );
    let result = action();
    match &result {
        Ok(receipt) => progress_linked(
            root,
            &receipt.id,
            OperationKind::Transfer,
            OperationPhase::Review,
            Some(&pending),
        ),
        Err(_) => progress(
            root,
            &pending,
            OperationKind::Transfer,
            OperationPhase::Failed,
        ),
    }
    result
}

/// Upgrade diagnostics use the existing bell. Only fixed classifications cross
/// this boundary; legacy bytes, credentials, paths and error messages never do.
fn migration_notice_codes(
    result: &super::schema::ModelResult<super::legacy_migration::MigrationReport>,
) -> Vec<&'static str> {
    let Ok(report) = result else {
        return vec!["layout_migration_failed"];
    };
    report
        .diagnostics
        .iter()
        .filter_map(|diagnostic| match diagnostic.code {
            "legacy_quarantined_original_retained"
            | "archive_deferred_original_retained"
            | "legacy_unreadable_original_retained"
            | "candidate_archive_deferred"
            | "candidate_changed_original_retained"
            | "candidate_cleanup_deferred_original_retained"
            | "admission_unavailable_originals_retained"
            | "admission_metadata_archive_deferred_lock_retained"
            | "retired_cleanup_deferred"
            | "temp_cleanup_deferred" => Some(diagnostic.code),
            // Normal contention, bounded batches and another writer winning
            // publication are expected retries, not new user-facing tasks.
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn publish_migration_notice(
    root: &Path,
    store: &crate::state::SharedNotificationStore,
    result: &super::schema::ModelResult<super::legacy_migration::MigrationReport>,
) -> crate::errors::AppResult<bool> {
    let codes = migration_notice_codes(result);
    if codes.is_empty() {
        return Ok(false);
    }
    let _ = crate::infrastructure::runtime_log::RuntimeLog::new(root).append_event(&format!(
        "model layout migration incomplete: {}",
        codes.join(",")
    ));
    let event = "model-config-migration-incomplete";
    // One stable version scope aggregates repeated startup failures. No
    // migration-pending state or new confirmation mechanism is introduced.
    let identity = crate::commands::event_delivery::prepare(
        root,
        event,
        Channel::Local,
        b"model-layout-upgrade-v1",
    )
    .ok_or(crate::errors::AppError::NotConfigured)?;
    let definition = registry::lookup(event)?;
    let delivery = registry::Delivery {
        event,
        job: definition.job,
        trigger: Trigger::Startup,
        identity,
        evidence: registry::Evidence::Risk,
        occurred_at: chrono::Utc::now(),
        detail: None,
    };
    let outcome = crate::commands::notifications::NotificationPublisher {
        store,
        data_root: root,
    }
    .publish_event(&delivery)?;
    Ok(outcome.changed)
}

/// Called after the normal notification store has been loaded, so startup
/// delivery cannot be overwritten by an earlier in-memory notifications copy.
pub fn migration_startup(
    app: &tauri::AppHandle,
    root: &Path,
    result: &super::schema::ModelResult<super::legacy_migration::MigrationReport>,
) {
    use tauri::Manager;
    let store = app.state::<crate::state::SharedNotificationStore>();
    match publish_migration_notice(root, store.inner(), result) {
        Ok(true) => {
            if crate::commands::event_delivery::emit_signal(
                app,
                "notifications-changed",
                Job::NotificationMutation,
                Trigger::Commit,
                Channel::Local,
                (),
            )
            .is_err()
            {
                diagnostic(root);
            }
        }
        Ok(false) => {}
        Err(_) => diagnostic(root),
    }
}

#[cfg(test)]
mod migration_notice_tests {
    use super::*;
    use crate::modules::{
        model_config::{legacy_migration, schema::*},
        notifications::{persistence::*, NotificationStore},
    };
    use std::sync::{Arc, Mutex};

    fn store() -> crate::state::SharedNotificationStore {
        Arc::new(Mutex::new(NotificationStore::new()))
    }
    fn put(root: &Path, relative: &str, bytes: &[u8]) {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }

    #[test]
    fn archive_failure_reaches_the_existing_persisted_bell_without_legacy_content() {
        let root = tempfile::tempdir().unwrap();
        put(root.path(), DRAFT_PATH, b"raw-secret-token-and-private-url");
        put(root.path(), "manager-state/legacy-model-config", b"blocked");
        put(
            root.path(),
            "config.json",
            b"actual model file stays byte exact",
        );
        let original = std::fs::read(root.path().join(DRAFT_PATH)).unwrap();
        let result = legacy_migration::migrate(root.path());
        assert!(result.is_ok());
        assert!(migration_notice_codes(&result).contains(&"archive_deferred_original_retained"));
        let store = store();
        assert!(publish_migration_notice(root.path(), &store, &result).unwrap());
        assert!(publish_migration_notice(root.path(), &store, &result).is_ok());
        let disk = load_notifications(&notifications_path(root.path())).unwrap();
        assert_eq!(disk.all().len(), 1);
        let notice = &disk.all()[0];
        assert_eq!(notice.title, "旧配置迁移未完成");
        assert!(!notice.body.contains("raw-secret"));
        assert!(!notice
            .body
            .contains(&root.path().to_string_lossy().to_string()));
        assert_eq!(
            std::fs::read(root.path().join(DRAFT_PATH)).unwrap(),
            original
        );
        assert_eq!(
            std::fs::read(root.path().join("config.json")).unwrap(),
            b"actual model file stays byte exact"
        );
    }

    #[test]
    fn required_layout_failure_is_reported_without_blocking_notification_startup() {
        let root = tempfile::tempdir().unwrap();
        put(
            root.path(),
            legacy_migration::LAYOUT_PATH,
            b"invalid-layout",
        );
        let result = legacy_migration::migrate(root.path());
        assert!(result.is_err());
        let store = store();
        assert!(publish_migration_notice(root.path(), &store, &result).unwrap());
        assert_eq!(store.lock().unwrap().all().len(), 1);
        assert_eq!(
            std::fs::read(root.path().join(legacy_migration::LAYOUT_PATH)).unwrap(),
            b"invalid-layout"
        );
    }

    #[test]
    fn successful_upgrade_and_busy_old_admission_do_not_create_notification_noise() {
        let root = tempfile::tempdir().unwrap();
        let store = store();
        let success = legacy_migration::migrate(root.path());
        assert!(success.is_ok());
        assert!(!publish_migration_notice(root.path(), &store, &success).unwrap());
        put(
            root.path(),
            "manager-state/new-channel-operations/00000000-0000-4000-8000-000000000001.json",
            b"{}",
        );
        let _guard = crate::infrastructure::locking::TargetFileLock::lock(
            &root.path().join("manager-state/new-channel-admission"),
        )
        .unwrap();
        let busy = legacy_migration::migrate(root.path());
        assert!(busy
            .as_ref()
            .unwrap()
            .diagnostics
            .iter()
            .any(|d| d.code == "admission_archive_deferred_lock_retained"));
        assert!(!publish_migration_notice(root.path(), &store, &busy).unwrap());
        assert!(store.lock().unwrap().all().is_empty());
    }

    #[test]
    fn unsafe_old_admission_is_reported_as_a_failure_instead_of_busy_contention() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("manager-state/new-channel-admission.lock"))
            .unwrap();
        let result = legacy_migration::migrate(root.path());
        assert!(
            migration_notice_codes(&result).contains(&"admission_unavailable_originals_retained")
        );
        assert!(publish_migration_notice(root.path(), &store(), &result).unwrap());
    }
}
