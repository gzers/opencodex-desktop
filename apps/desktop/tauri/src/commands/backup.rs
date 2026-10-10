use crate::commands::event_delivery;
use crate::errors::{AppError, AppResult};
use crate::modules::backup::browser::{self, BackupFiles};
use crate::modules::notifications::registry::{self, Channel, Job, Trigger};
use crate::state::SharedDataRoot;
use tauri::Manager;

#[tauri::command]
pub async fn backup_files(root: tauri::State<'_, SharedDataRoot>) -> AppResult<BackupFiles> {
    registry::validate_signal(
        "preferences-backup-list",
        Job::BackupList,
        Trigger::User,
        Channel::Local,
    )
    .map_err(|_| AppError::NotConfigured)?;
    let root = root.0.clone();
    tauri::async_runtime::spawn_blocking(move || browser::list(&root))
        .await
        .map_err(|_| AppError::NotConfigured)?
}

#[tauri::command]
pub async fn open_backup_file(root: tauri::State<'_, SharedDataRoot>, id: String) -> AppResult<()> {
    registry::validate_signal(
        "preferences-backup-open",
        Job::BackupList,
        Trigger::User,
        Channel::Local,
    )
    .map_err(|_| AppError::NotConfigured)?;
    let root = root.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let path = browser::resolve_open(&root, &id)?;
        crate::commands::workspace::open_path(&path)
    })
    .await
    .map_err(|_| AppError::NotConfigured)?
}

// W2: lib.rs mounting is owned by the coordinating main thread.
use crate::modules::backup::manager;
use crate::types::upgrade::{
    BackupCleanupPreviewDto, PreferencesBackupDto, PreferencesBackupResultDto,
    PreferencesRestoreResultDto,
};

#[tauri::command]
pub async fn create_preferences_backup(
    root: tauri::State<'_, SharedDataRoot>,
    app: tauri::AppHandle,
) -> AppResult<PreferencesBackupResultDto> {
    let root = root.0.clone();
    crate::commands::run_blocking("create preferences backup", move || {
        manager::create_with_saved_policy_and_observers(
            &root,
            chrono::Utc::now(),
            |payload| {
                event_delivery::prepare(
                    &root,
                    "preferences-backup-failed",
                    Channel::Local,
                    &[
                        crate::modules::backup::BackupAction::ManualPreferences
                            .as_str()
                            .as_bytes(),
                        b"\0",
                        payload,
                    ]
                    .concat(),
                )
            },
            |identity, result| {
                event_delivery::publish(
                    &app,
                    &root,
                    if result.is_ok() {
                        "preferences-backup-succeeded"
                    } else {
                        "preferences-backup-failed"
                    },
                    identity,
                    Trigger::User,
                )
            },
            &mut event_delivery::BackupEvents {
                app: &app,
                root: &root,
                trigger: Trigger::User,
            },
        )
    })
    .await
}

#[tauri::command]
pub async fn list_preferences_backups(
    root: tauri::State<'_, SharedDataRoot>,
) -> AppResult<Vec<PreferencesBackupDto>> {
    registry::validate_signal(
        "preferences-backup-list",
        Job::BackupList,
        Trigger::User,
        Channel::Local,
    )
    .map_err(|_| AppError::NotConfigured)?;
    let root = root.0.clone();
    // Even reads may create the cooperative transaction lock on disk.
    crate::commands::run_blocking("list preferences backups", move || manager::list(&root)).await
}

#[tauri::command]
pub async fn set_preferences_backup_pinned(
    root: tauri::State<'_, SharedDataRoot>,
    id: String,
    pinned: bool,
    app: tauri::AppHandle,
) -> AppResult<()> {
    registry::validate_signal(
        "preferences-backup-pin",
        Job::BackupPin,
        Trigger::User,
        Channel::Local,
    )
    .map_err(|_| AppError::NotConfigured)?;
    let root = root.0.clone();
    crate::commands::run_blocking("pin preferences backup", move || {
        manager::set_pinned_observed(&root, &id, pinned, |candidate, succeeded| {
            let identity = event_delivery::prepare(
                &root,
                "preferences-backup-pin-failed",
                Channel::Local,
                candidate,
            );
            event_delivery::publish(
                &app,
                &root,
                if succeeded {
                    "preferences-backup-pin-succeeded"
                } else {
                    "preferences-backup-pin-failed"
                },
                identity,
                Trigger::User,
            );
        })
    })
    .await
}

#[tauri::command]
pub async fn preview_preferences_backup_cleanup(
    root: tauri::State<'_, SharedDataRoot>,
) -> AppResult<BackupCleanupPreviewDto> {
    registry::validate_signal(
        "preferences-backup-preview",
        Job::BackupCleanup,
        Trigger::User,
        Channel::Local,
    )
    .map_err(|_| AppError::NotConfigured)?;
    let root = root.0.clone();
    crate::commands::run_blocking("preview preferences cleanup", move || {
        manager::preview(&root, chrono::Utc::now())
    })
    .await
}

#[tauri::command]
pub async fn cleanup_preferences_backups(
    root: tauri::State<'_, SharedDataRoot>,
    preview: BackupCleanupPreviewDto,
    app: tauri::AppHandle,
) -> AppResult<Vec<String>> {
    let root = root.0.clone();
    crate::commands::run_blocking("cleanup preferences backups", move || {
        // The preview token already commits to the exact inventory and policy.
        let identity = event_delivery::prepare(
            &root,
            "preferences-backup-cleanup-failed",
            Channel::Local,
            preview.token.as_bytes(),
        );
        let result = manager::execute(&root, &preview, chrono::Utc::now());
        event_delivery::publish(
            &app,
            &root,
            if result.is_ok() {
                "preferences-backup-cleanup-succeeded"
            } else {
                "preferences-backup-cleanup-failed"
            },
            identity,
            Trigger::User,
        );
        result
    })
    .await
}

#[tauri::command]
pub async fn restore_preferences_backup(
    root: tauri::State<'_, SharedDataRoot>,
    id: String,
    app: tauri::AppHandle,
) -> AppResult<PreferencesRestoreResultDto> {
    let root = root.0.clone();
    let status = app
        .state::<crate::commands::update::SharedUpdateStatus>()
        .inner()
        .clone();
    let runtime = app
        .state::<crate::state::SharedRuntimeInstall>()
        .inner()
        .clone();
    let worker_app = app.clone();
    // The reservation lives in the worker, not the dismissible IPC observer.
    let result = crate::commands::run_blocking("restore preferences backup", move || {
        let _reservation = {
            let state = status.lock().map_err(|_| AppError::NotConfigured)?;
            if state.installing || state.pending_restart.is_some() {
                return Err(AppError::NotConfigured);
            }
            runtime.acquire().ok_or(AppError::NotConfigured)?
        };
        let identity = event_delivery::prepare(
            &root,
            "preferences-restore-failed",
            Channel::Local,
            id.as_bytes(),
        );
        let result = manager::restore_observed(
            &root,
            &id,
            chrono::Utc::now(),
            &mut event_delivery::BackupEvents {
                app: &worker_app,
                root: &root,
                trigger: Trigger::User,
            },
        );
        event_delivery::publish(
            &worker_app,
            &root,
            if result.is_ok() {
                "preferences-restore-succeeded"
            } else {
                "preferences-restore-failed"
            },
            identity,
            Trigger::User,
        );
        let mut restored = result?;
        // Reflect the latest committed preferences. If a save happened immediately
        // after restore, this reread still matches disk rather than a stale snapshot.
        let projection = (|| {
            let _transaction = manager::acquire_preferences_transaction(&root)?;
            let preferences = crate::commands::preferences::load_preferences_with_path(&root)?;
            let channel =
                crate::modules::update::UpdateChannel::parse(&preferences.app_update_channel)
                    .ok_or(AppError::NotConfigured)?;
            let mut state = status.lock().map_err(|_| AppError::NotConfigured)?;
            if !state.switch_channel(channel) {
                return Err(AppError::NotConfigured);
            }
            Ok::<_, AppError>(())
        })();
        // Disk commit is already verified; an in-memory refresh failure is separate.
        restored.refresh_required = projection.is_err();
        Ok(restored)
    })
    .await?;
    crate::infrastructure::window_appearance::refresh(&app);
    let _ = crate::commands::event_delivery::emit_signal(
        &app,
        "preferences-restored",
        crate::modules::notifications::registry::Job::PreferencesRestore,
        crate::modules::notifications::registry::Trigger::User,
        crate::modules::notifications::registry::Channel::Local,
        (),
    );
    Ok(result)
}

#[tauri::command]
pub async fn preferences_backup_policy(
    root: tauri::State<'_, SharedDataRoot>,
) -> AppResult<crate::modules::backup::policy::CleanupPolicy> {
    let root = root.0.clone();
    crate::commands::run_blocking("read backup policy", move || manager::read_policy(&root)).await
}

#[tauri::command]
pub async fn save_preferences_backup_policy(
    root: tauri::State<'_, SharedDataRoot>,
    policy: crate::modules::backup::policy::CleanupPolicy,
    app: tauri::AppHandle,
) -> AppResult<()> {
    let root = root.0.clone();
    crate::commands::run_blocking("save backup policy", move || {
        manager::save_policy_observed(&root, &policy, |candidate, succeeded| {
            let identity = event_delivery::prepare(
                &root,
                "preferences-backup-policy-failed",
                Channel::Local,
                candidate,
            );
            event_delivery::publish(
                &app,
                &root,
                if succeeded {
                    "preferences-backup-policy-succeeded"
                } else {
                    "preferences-backup-policy-failed"
                },
                identity,
                Trigger::User,
            );
        })
    })
    .await
}

#[cfg(test)]
mod policy_event_tests {
    use super::*;
    use crate::modules::backup::policy::{CleanupMode, CleanupPolicy, POLICY_RELATIVE_PATH};
    use crate::modules::notifications::{
        persistence::{load_notifications, notifications_path},
        registry::{Delivery, Evidence},
        NotificationStore,
    };
    use std::{
        path::Path,
        sync::{Arc, Mutex},
    };

    fn save_observed(
        root: &Path,
        policy: &CleanupPolicy,
        store: &Arc<Mutex<NotificationStore>>,
    ) -> AppResult<()> {
        manager::save_policy_observed(root, policy, |candidate, succeeded| {
            let identity = event_delivery::prepare(
                root,
                "preferences-backup-policy-failed",
                Channel::Local,
                candidate,
            )
            .unwrap();
            let event = if succeeded {
                "preferences-backup-policy-succeeded"
            } else {
                "preferences-backup-policy-failed"
            };
            let evidence = if succeeded {
                Evidence::Success {
                    candidate: identity.candidate.clone(),
                    verified: true,
                }
            } else {
                Evidence::Failure
            };
            let delivery = Delivery {
                event,
                job: Job::BackupPolicySave,
                trigger: Trigger::User,
                identity,
                evidence,
                occurred_at: chrono::Utc::now(),
            };
            let _ = crate::commands::notifications::NotificationPublisher {
                store,
                data_root: root,
            }
            .publish_event(&delivery);
        })
    }

    #[test]
    fn real_policy_write_failure_and_exact_retry_persist_without_cleanup() {
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        crate::commands::preferences::save_preferences_with_path(
            root.path(),
            &crate::modules::preferences::Preferences::default(),
        )
        .unwrap();
        // More than N backups outside D days would be eligible if saving the
        // automatic mode accidentally ran rotation immediately.
        for days in 40..52 {
            manager::create(
                root.path(),
                chrono::Utc::now() - chrono::Duration::days(days),
                Default::default(),
            )
            .unwrap();
        }
        let inventory_before = manager::list(root.path()).unwrap();
        assert_eq!(inventory_before.len(), 12);
        let policy = CleanupPolicy {
            mode: CleanupMode::Automatic,
            ..Default::default()
        };
        let path = root.path().join(POLICY_RELATIVE_PATH);
        std::fs::create_dir(&path).unwrap();
        let store = Arc::new(Mutex::new(NotificationStore::new()));
        assert!(save_observed(root.path(), &policy, &store).is_err());
        let failed = load_notifications(&notifications_path(root.path())).unwrap();
        assert_eq!(failed.all().len(), 1);
        assert!(!failed.all()[0].resolved);
        assert!(!failed.all()[0]
            .body
            .contains(&root.path().to_string_lossy().to_string()));
        std::fs::remove_dir(&path).unwrap();
        let reloaded = Arc::new(Mutex::new(failed));
        save_observed(root.path(), &policy, &reloaded).unwrap();
        assert_eq!(manager::read_policy(root.path()).unwrap(), policy);
        let history = load_notifications(&notifications_path(root.path())).unwrap();
        assert_eq!(history.all().len(), 1);
        assert!(history.all()[0].resolved);
        assert!(!history.all()[0].read);
        let inventory_after = manager::list(root.path()).unwrap();
        assert_eq!(
            serde_json::to_value(inventory_before).unwrap(),
            serde_json::to_value(inventory_after).unwrap()
        );
    }

    #[test]
    fn changed_policy_and_other_root_do_not_clear_the_original_failure() {
        let root = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        for path in [root.path(), other.path()] {
            crate::modules::data_root::initialize(path).unwrap();
        }
        let policy = CleanupPolicy {
            mode: CleanupMode::Automatic,
            ..Default::default()
        };
        let path = root.path().join(POLICY_RELATIVE_PATH);
        std::fs::create_dir(&path).unwrap();
        let store = Arc::new(Mutex::new(NotificationStore::new()));
        assert!(save_observed(root.path(), &policy, &store).is_err());
        std::fs::remove_dir(&path).unwrap();
        save_observed(other.path(), &policy, &store).unwrap();
        assert!(!store.lock().unwrap().all()[0].resolved);
        save_observed(root.path(), &CleanupPolicy::default(), &store).unwrap();
        assert!(!store.lock().unwrap().all()[0].resolved);
        // Returning to old contents is a new candidate generation, not recovery.
        save_observed(root.path(), &policy, &store).unwrap();
        assert!(!store.lock().unwrap().all()[0].resolved);
    }

    #[test]
    fn validation_and_lock_refusals_do_not_call_terminal_observer() {
        let root = tempfile::tempdir().unwrap();
        let invalid = CleanupPolicy {
            keep_days: 31,
            ..Default::default()
        };
        assert!(
            manager::save_policy_observed(root.path(), &invalid, |_, _| panic!("invalid delivery"))
                .is_err()
        );
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
        std::fs::create_dir(root.path().join(".backup-w2.lock")).unwrap();
        assert!(manager::save_policy_observed(
            root.path(),
            &CleanupPolicy::default(),
            |_, _| panic!("lock refusal delivery")
        )
        .is_err());
        assert!(!root.path().join("manager-state").exists());
    }

    #[test]
    fn notification_persistence_failure_does_not_undo_saved_policy() {
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        let path = root.path().join(POLICY_RELATIVE_PATH);
        let policy = CleanupPolicy {
            mode: CleanupMode::Automatic,
            ..Default::default()
        };
        let store = Arc::new(Mutex::new(NotificationStore::new()));
        std::fs::create_dir(&path).unwrap();
        assert!(save_observed(root.path(), &policy, &store).is_err());
        std::fs::remove_dir(&path).unwrap();
        let notifications = notifications_path(root.path());
        std::fs::remove_file(&notifications).unwrap();
        std::fs::create_dir(&notifications).unwrap();
        save_observed(root.path(), &policy, &store).unwrap();
        assert_eq!(manager::read_policy(root.path()).unwrap(), policy);
        assert!(!store.lock().unwrap().all()[0].resolved);
    }
}
