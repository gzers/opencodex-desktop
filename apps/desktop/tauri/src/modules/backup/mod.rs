//! FZ-04 / FZ-21 备份存储、清单、复验与保留清理。
//!
//! 模块只作用于显式数据根；测试使用临时目录。
//! 不访问真实用户备份、不连接远端、不访问 Keychain。

use chrono::{DateTime, SecondsFormat, Utc};
pub mod browser;
pub mod manager;
pub mod policy;
pub(crate) mod safety;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::errors::AppError;
use crate::infrastructure::hash::{sha256_file, sha256_hex};

pub const MANIFEST_NAME: &str = "backup-manifest.json";
pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_PER_ACTION: usize = 10;
pub fn retention_days() -> i64 {
    30
}

pub const ACTIONS: [&str; 9] = [
    "upgrade",
    "import",
    "sync-overwrite",
    "data-root-move",
    "extension-write",
    "runtime-uninstall",
    "manual-preferences",
    "restore-protection",
    "preferences-protection",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackupAction {
    Upgrade,
    Import,
    SyncOverwrite,
    DataRootMove,
    ExtensionWrite,
    /// 托管卸载前的备份（IMP `FZ-51`）。
    RuntimeUninstall,
    ManualPreferences,
    RestoreProtection,
    PreferencesProtection,
}

impl BackupAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Upgrade => "upgrade",
            Self::Import => "import",
            Self::SyncOverwrite => "sync-overwrite",
            Self::DataRootMove => "data-root-move",
            Self::ExtensionWrite => "extension-write",
            Self::RuntimeUninstall => "runtime-uninstall",
            Self::ManualPreferences => "manual-preferences",
            Self::RestoreProtection => "restore-protection",
            Self::PreferencesProtection => "preferences-protection",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "upgrade" => Some(Self::Upgrade),
            "import" => Some(Self::Import),
            "sync-overwrite" => Some(Self::SyncOverwrite),
            "data-root-move" => Some(Self::DataRootMove),
            "extension-write" => Some(Self::ExtensionWrite),
            "runtime-uninstall" => Some(Self::RuntimeUninstall),
            "manual-preferences" => Some(Self::ManualPreferences),
            "restore-protection" => Some(Self::RestoreProtection),
            "preferences-protection" => Some(Self::PreferencesProtection),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BackupManifest {
    pub schema_version: u32,
    pub backup_id: String,
    pub action: BackupAction,
    pub target_path: String,
    pub stored_at: String,
    pub created_at: String,
    pub sha256: String,
    pub bytes: u64,
    pub file_count: u64,
    pub restorable: bool,
    pub note: Option<String>,
    /// Absent on legacy records: never infer transaction ownership from note/path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub management: Option<manager::Management>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupRecord {
    pub manifest: BackupManifest,
    pub directory: PathBuf,
}

/// 生成 `bk_<utc14>_<8位随机hex>`。
pub fn generate_backup_id(now: DateTime<Utc>) -> String {
    let mut random = [0_u8; 4];
    rand::rng().fill_bytes(&mut random);
    format!(
        "bk_{}_{}",
        now.format("%Y%m%d%H%M%S"),
        random
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

/// 创建文件备份；payload 是源文件字节副本。
pub fn backup_file(
    data_root: &Path,
    action: BackupAction,
    target_path: &Path,
    payload: &[u8],
    now: DateTime<Utc>,
    note: Option<String>,
) -> Result<BackupRecord, AppError> {
    validate_target(target_path)?;
    let backup_id = generate_backup_id(now);
    let directory = data_root
        .join("backups")
        .join(now.format("%Y").to_string())
        .join(now.format("%m").to_string())
        .join(action.as_str())
        .join(&backup_id);
    safety::check_path(data_root, &directory, true)?;
    std::fs::create_dir_all(directory.parent().expect("backup parent")).map_err(|error| {
        AppError::FileSystem {
            operation: "create backup directory".to_string(),
            detail: error.to_string(),
        }
    })?;

    safety::check_path(data_root, directory.parent().expect("backup parent"), false)?;
    std::fs::create_dir(&directory).map_err(|error| AppError::FileSystem {
        operation: "reserve backup id".into(),
        detail: error.to_string(),
    })?;

    let stored_path =
        directory.join(
            target_path
                .file_name()
                .ok_or_else(|| AppError::FileSystem {
                    operation: "resolve backup filename".to_string(),
                    detail: "target has no file name".to_string(),
                })?,
        );
    write_and_verify(&stored_path, payload, 0o600)?;
    let actual_hash = sha256_file(&stored_path).map_err(|error| AppError::FileSystem {
        operation: "verify backup file".to_string(),
        detail: error.to_string(),
    })?;
    let metadata = std::fs::metadata(&stored_path).map_err(|error| AppError::FileSystem {
        operation: "inspect backup file".to_string(),
        detail: error.to_string(),
    })?;

    let manifest = BackupManifest {
        schema_version: SCHEMA_VERSION,
        backup_id: backup_id.clone(),
        action,
        target_path: target_path.to_string_lossy().into_owned(),
        stored_at: now.to_rfc3339_opts(SecondsFormat::Secs, true),
        created_at: now.to_rfc3339_opts(SecondsFormat::Secs, true),
        sha256: actual_hash,
        bytes: metadata.len(),
        file_count: 1,
        restorable: true,
        note,
        management: None,
    };
    write_manifest(&directory, &manifest)?;
    safety::sync_directories(data_root, &directory)?;
    Ok(BackupRecord {
        manifest,
        directory,
    })
}

/// 校验备份仍可通过 SHA-256 与清单复验。
pub fn verify_backup(record: &BackupRecord) -> Result<bool, AppError> {
    safety::validate_record(record)?;
    let stored_path = find_stored_payload(&record.directory)?;
    let actual = sha256_hex(&safety::read_file(&stored_path, safety::MAX_PAYLOAD_BYTES)?);
    Ok(actual == record.manifest.sha256
        && stored_path.metadata().map(|m| m.len()).unwrap_or(0) == record.manifest.bytes)
}

/// Compatibility entry point for explicit cleanup. Legacy transaction records are
/// protected; only explicitly managed standalone preferences backups are eligible.
pub fn cleanup_retention(
    data_root: &Path,
    now: DateTime<Utc>,
    max_per_action: usize,
) -> Result<Vec<String>, AppError> {
    manager::cleanup_with_count(data_root, now, max_per_action)
}

/// Bounded metadata-only listing; restorable is a claim, not an integrity verdict.
pub fn list_action_records(
    backups_root: &Path,
    action: &str,
) -> Result<Vec<BackupRecord>, AppError> {
    safety::scan(backups_root).map(|records| {
        records
            .into_iter()
            .filter(|record| record.manifest.action.as_str() == action)
            .collect()
    })
}

fn find_stored_payload(directory: &Path) -> Result<PathBuf, AppError> {
    safety::payload_path(directory)
}

fn write_manifest(directory: &Path, manifest: &BackupManifest) -> Result<(), AppError> {
    let payload = serde_json::to_vec_pretty(manifest).map_err(|error| AppError::AtomicWrite {
        reason: error.to_string(),
    })?;
    crate::infrastructure::atomic_write::atomic_write(
        &directory.join(MANIFEST_NAME),
        &payload,
        0o600,
    )
}

fn write_and_verify(path: &Path, payload: &[u8], permissions: u32) -> Result<(), AppError> {
    crate::infrastructure::atomic_write::atomic_write(path, payload, permissions)?;
    let actual = sha256_file(path).map_err(|error| AppError::FileSystem {
        operation: "hash backup payload".to_string(),
        detail: error.to_string(),
    })?;
    if actual != sha256_hex(payload) {
        return Err(AppError::FileSystem {
            operation: "verify backup payload".to_string(),
            detail: "post-write SHA-256 mismatch".to_string(),
        });
    }
    Ok(())
}

fn validate_target(target: &Path) -> Result<(), AppError> {
    if target.as_os_str().is_empty() || target.file_name().is_some_and(|name| name == MANIFEST_NAME)
    {
        return Err(AppError::FileSystem {
            operation: "validate backup target".to_string(),
            detail: "target is empty or uses reserved manifest name".to_string(),
        });
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::os::unix::fs::PermissionsExt;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 9, 15, 0, 0, 0).unwrap()
    }

    fn old(days: i64) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 9, 15, 0, 0, 0).unwrap() - chrono::Duration::days(days)
    }

    #[test]
    fn backup_id_matches_frozen_shape() {
        let id = generate_backup_id(now());
        assert!(id.starts_with("bk_20260915000000_"));
        let suffix = id.rsplit('_').next().expect("random suffix");
        assert_eq!(suffix.len(), 8);
        assert!(suffix.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn creates_private_file_backup_and_verified_manifest() {
        let temp = tempfile::tempdir().expect("temporary data root");
        let target = temp.path().join("config.json");
        std::fs::write(&target, b"payload").expect("write target");

        let record = backup_file(
            temp.path(),
            BackupAction::Upgrade,
            &target,
            b"payload",
            now(),
            Some("test".to_string()),
        )
        .expect("create backup");
        assert!(record
            .directory
            .starts_with(temp.path().join("backups/2026/09/upgrade/")));
        assert!(record.manifest.restorable);
        assert_eq!(record.manifest.sha256, sha256_hex(b"payload"));
        assert_eq!(record.manifest.bytes, 7);
        assert_eq!(record.manifest.file_count, 1);
        assert!(verify_backup(&record).expect("verify"));

        let manifest_path = record.directory.join(MANIFEST_NAME);
        let mode = std::fs::metadata(&manifest_path)
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn corrupted_payload_fails_verification() {
        let temp = tempfile::tempdir().expect("temporary data root");
        let target = temp.path().join("config.json");
        std::fs::write(&target, b"payload").expect("write target");
        let mut record = backup_file(
            temp.path(),
            BackupAction::Import,
            &target,
            b"payload",
            now(),
            None,
        )
        .expect("create backup");
        let payload = find_stored_payload(&record.directory).expect("payload");
        std::fs::write(payload, b"changed").expect("corrupt payload");
        assert!(!verify_backup(&record).expect("verify"));
        record.manifest.restorable = false;
        assert!(!record.manifest.restorable);
    }

    #[test]
    fn listing_reads_metadata_and_explicit_verification_detects_same_size_corruption() {
        let temp = tempfile::tempdir().unwrap();
        let record = backup_file(
            temp.path(),
            BackupAction::Upgrade,
            &temp.path().join("config.json"),
            b"payload",
            now(),
            None,
        )
        .unwrap();
        std::fs::write(record.directory.join("config.json"), b"changed").unwrap();
        let listed = list_action_records(&temp.path().join("backups"), "upgrade").unwrap();
        assert_eq!(listed.len(), 1);
        assert!(listed[0].manifest.restorable); // creation claim, no payload hash on list
        assert!(!verify_backup(&listed[0]).unwrap());
    }

    #[test]
    fn legacy_transactions_are_retained_even_without_known_links() {
        let temp = tempfile::tempdir().unwrap();
        for action in [
            BackupAction::Upgrade,
            BackupAction::Import,
            BackupAction::RuntimeUninstall,
        ] {
            for i in 0..22 {
                backup_file(
                    temp.path(),
                    action,
                    &temp.path().join("target.txt"),
                    b"before",
                    old(60 + i),
                    Some("unstructured transaction reference".into()),
                )
                .unwrap();
            }
        }
        assert!(cleanup_retention(temp.path(), now(), 5).unwrap().is_empty());
        assert_eq!(
            safety::scan(&temp.path().join("backups")).unwrap().len(),
            66
        );
    }

    #[test]
    fn empty_target_is_rejected() {
        let temp = tempfile::tempdir().expect("temporary data root");
        let result = backup_file(
            temp.path(),
            BackupAction::Upgrade,
            Path::new(""),
            b"payload",
            now(),
            None,
        );
        assert!(result.is_err());
    }
}
