//! 官方升级引导前的备份命令层。只做数据根编排和 DTO 投影，不执行更新。

use crate::modules::backup;
use crate::modules::sync::config::{SyncConfig, SyncConfigStore};
use crate::state::SharedDataRoot;
use crate::types::runtime_status::RestoreGuidanceDto;
use crate::types::upgrade::RestoreRiskSummaryDto;
use crate::types::upgrade::UpgradeBackupResult;

#[tauri::command]
pub async fn create_upgrade_backup(
    data_root: tauri::State<'_, SharedDataRoot>,
    app: tauri::AppHandle,
) -> crate::errors::AppResult<UpgradeBackupResult> {
    let root = data_root.0.clone();
    crate::commands::run_blocking("create upgrade backup", move || {
        backup::manager::create_upgrade_observed(
            &root,
            &mut crate::commands::event_delivery::BackupEvents {
                app: &app,
                root: &root,
                trigger: crate::modules::notifications::registry::Trigger::User,
            },
        )
    })
    .await
}

#[tauri::command]
pub async fn create_restore_backup(
    data_root: tauri::State<'_, SharedDataRoot>,
    app: tauri::AppHandle,
) -> crate::errors::AppResult<UpgradeBackupResult> {
    let root = data_root.0.clone();
    crate::commands::run_blocking("create restore backup", move || {
        let guard = backup::manager::begin_preferences_protection_observed(
            &root,
            true,
            &mut crate::commands::event_delivery::BackupEvents {
                app: &app,
                root: &root,
                trigger: crate::modules::notifications::registry::Trigger::User,
            },
        )?;
        guard
            .backup
            .clone()
            .ok_or(crate::errors::AppError::NotConfigured)
    })
    .await
}

#[tauri::command]
pub fn restore_risk_summary(
    data_root: tauri::State<'_, SharedDataRoot>,
    guidance: tauri::State<'_, crate::state::SharedStatusCollector>,
) -> crate::errors::AppResult<RestoreRiskSummaryDto> {
    let opencodex_home = crate::modules::data_root::load_runtime_config(&data_root.0)
        .map(|config| crate::modules::data_root::resolve_opencodex_home(&config))
        .map(|path| path.to_string_lossy().into_owned())
        .ok();
    let snapshot = {
        let mut collector = guidance
            .lock()
            .map_err(|_| crate::errors::AppError::NotConfigured)?;
        crate::commands::status_snapshot_from_collector(&mut collector)?
    };
    Ok(RestoreRiskSummaryDto {
        guidance: RestoreGuidanceDto::from_snapshot(&snapshot),
        preferences_configured: data_root
            .0
            .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH)
            .is_file(),
        extension_configured: data_root
            .0
            .join(crate::modules::extensions::projection::CONFIG_RELATIVE_PATH)
            .is_file(),
        sync_endpoint_configured: SyncConfigStore::new(&data_root.0)
            .load()
            .map(|config: SyncConfig| !config.endpoints.is_empty())
            .unwrap_or(false),
        opencodex_home,
    })
}

pub fn create_upgrade_backup_with_root(
    data_root: &std::path::Path,
) -> Result<UpgradeBackupResult, crate::errors::AppError> {
    if !data_root.is_absolute() {
        return Err(crate::errors::AppError::NotConfigured);
    }
    crate::modules::data_root::validate_structure(data_root)
        .map_err(|_| crate::errors::AppError::NotConfigured)?;
    // Preserve the historical explicit missing-preferences error.
    let target = data_root.join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH);
    if std::fs::symlink_metadata(&target).is_err() {
        return Err(crate::errors::AppError::NotConfigured);
    }
    backup::manager::create_upgrade(data_root)
}

/// Durable protection; retained independently of the generic upgrade history.
pub fn create_restore_backup_with_root(
    data_root: &std::path::Path,
) -> crate::errors::AppResult<UpgradeBackupResult> {
    crate::modules::data_root::validate_structure(data_root)
        .map_err(|_| crate::errors::AppError::NotConfigured)?;
    let guard = backup::manager::begin_preferences_protection(data_root, true)?;
    guard
        .backup
        .clone()
        .ok_or(crate::errors::AppError::NotConfigured)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::modules::backup::{self};
    use crate::modules::preferences::{Preferences, PreferencesStore};
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};

    #[test]
    fn creates_upgrade_backup_from_real_preferences() {
        let root = tempfile::tempdir().expect("temporary data root");
        crate::modules::data_root::initialize(root.path()).expect("initialize data root");
        let preferences = Preferences {
            interface_scale: 150,
            ..Default::default()
        };
        PreferencesStore::new(root.path())
            .save(&preferences)
            .expect("save preferences");

        let result = create_upgrade_backup_with_root(root.path()).expect("create upgrade backup");
        let expected_target = root
            .path()
            .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH);
        assert!(result.backup_id.starts_with("bk_"));
        assert_eq!(result.target_path, expected_target.to_string_lossy());
        let directory = PathBuf::from(&result.directory);
        assert!(directory.starts_with(root.path()));
        assert!(directory.to_string_lossy().contains("/upgrade/"));

        let stored =
            std::fs::read(directory.join("preferences.json")).expect("read stored payload");
        // C 阶段：备份的是分域磁盘结构，按统一读取规则还原为扁平偏好。
        let stored_document: serde_json::Value =
            serde_json::from_slice(&stored).expect("restore preferences document");
        let restored = crate::modules::preferences::preferences_from_document(&stored_document)
            .expect("restore preferences");
        assert_eq!(restored, preferences);
        let mode = std::fs::metadata(directory.join("preferences.json"))
            .expect("stored metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);

        let records = backup::list_action_records(
            &root.path().join("backups"),
            backup::BackupAction::Upgrade.as_str(),
        )
        .expect("list upgrade records");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].manifest.backup_id, result.backup_id);
    }

    #[test]
    fn missing_preferences_is_explicit_not_invented() {
        let root = tempfile::tempdir().expect("temporary data root");
        crate::modules::data_root::initialize(root.path()).expect("initialize data root");
        let error = create_upgrade_backup_with_root(root.path())
            .expect_err("missing preferences must fail");
        assert!(matches!(error, crate::errors::AppError::NotConfigured));
    }

    #[test]
    fn create_restore_backup_is_permanently_protected() {
        let root = tempfile::tempdir().expect("temporary data root");
        crate::modules::data_root::initialize(root.path()).expect("initialize data root");
        let preferences = Preferences::default();
        PreferencesStore::new(root.path())
            .save(&preferences)
            .expect("save preferences");

        let result = create_restore_backup_with_root(root.path()).expect("create backup");
        assert!(result.directory.contains("/preferences-protection/"));
        let directory = PathBuf::from(&result.directory);
        let stored = std::fs::read(directory.join("preferences.json")).expect("read backup");
        let document: serde_json::Value = serde_json::from_slice(&stored).unwrap();
        let restored = crate::modules::preferences::preferences_from_document(&document).unwrap();
        assert_eq!(restored, preferences);
        let records = backup::list_action_records(
            &root.path().join("backups"),
            backup::BackupAction::PreferencesProtection.as_str(),
        )
        .expect("list upgrade records");
        assert_eq!(records.len(), 1);
        assert!(records[0].manifest.management.as_ref().unwrap().pinned);
    }

    #[test]
    fn relative_or_invalid_data_root_is_rejected() {
        let root = tempfile::tempdir().expect("temporary data root");
        crate::modules::data_root::initialize(root.path()).expect("initialize data root");
        assert!(create_upgrade_backup_with_root(Path::new("relative")).is_err());

        let nested = tempfile::tempdir().expect("nested root");
        assert!(create_upgrade_backup_with_root(nested.path()).is_err());
    }
}
