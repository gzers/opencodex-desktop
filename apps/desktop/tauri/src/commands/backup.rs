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
        manager::set_pinned(&root, &id, pinned)
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
) -> AppResult<()> {
    let root = root.0.clone();
    crate::commands::run_blocking("save backup policy", move || {
        manager::save_policy(&root, &policy)
    })
    .await
}
