//! FZ-04 / FZ-21 备份存储、清单、复验与保留清理。
//!
//! 模块只作用于显式数据根；测试使用临时目录。
//! 不访问真实用户备份、不连接远端、不访问 Keychain。

use chrono::{DateTime, SecondsFormat, Utc};
pub mod browser;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::errors::AppError;
use crate::infrastructure::hash::{sha256_file, sha256_hex};

pub const MANIFEST_NAME: &str = "backup-manifest.json";
pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_PER_ACTION: usize = 20;
pub fn retention_days() -> i64 {
    crate::modules::runtime_defaults::backup_max_age_days()
}

pub const ACTIONS: [&str; 6] = [
    "upgrade",
    "import",
    "sync-overwrite",
    "data-root-move",
    "extension-write",
    "runtime-uninstall",
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
    std::fs::create_dir_all(&directory).map_err(|error| AppError::FileSystem {
        operation: "create backup directory".to_string(),
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
    };
    write_manifest(&directory, &manifest)?;
    Ok(BackupRecord {
        manifest,
        directory,
    })
}

/// 校验备份仍可通过 SHA-256 与清单复验。
pub fn verify_backup(record: &BackupRecord) -> Result<bool, AppError> {
    let stored_path = find_stored_payload(&record.directory)?;
    let actual = sha256_file(&stored_path).map_err(|error| AppError::FileSystem {
        operation: "verify backup payload".to_string(),
        detail: error.to_string(),
    })?;
    Ok(actual == record.manifest.sha256
        && stored_path.metadata().map(|m| m.len()).unwrap_or(0) == record.manifest.bytes)
}

/// 保留每类最近 `max_per_action` 份与 30 天内记录；只删除目录，不输出内容摘要。
///
/// `max_per_action` 由调用方传入（当前来自「备份保留策略」偏好）；传 `MAX_PER_ACTION`
/// 即保持历史默认行为。
pub fn cleanup_retention(
    data_root: &Path,
    now: DateTime<Utc>,
    max_per_action: usize,
) -> Result<Vec<String>, AppError> {
    let backups = data_root.join("backups");
    let mut removed = Vec::new();
    if !backups.is_dir() {
        return Ok(removed);
    }

    for action in ACTIONS {
        let mut records = list_action_records(&backups, action)?;
        records.sort_by(|a, b| b.manifest.created_at.cmp(&a.manifest.created_at));
        for (index, record) in records.into_iter().enumerate() {
            let created = DateTime::parse_from_rfc3339(&record.manifest.created_at)
                .map_err(|_| AppError::FileSystem {
                    operation: "parse backup timestamp".to_string(),
                    detail: record.manifest.backup_id.clone(),
                })?
                .with_timezone(&Utc);
            let within_days = now.signed_duration_since(created).num_days() < retention_days();
            if index < max_per_action || within_days {
                continue;
            }
            std::fs::remove_dir_all(&record.directory).map_err(|error| AppError::FileSystem {
                operation: "cleanup expired backup".to_string(),
                detail: error.to_string(),
            })?;
            removed.push(record.manifest.backup_id);
        }
    }
    Ok(removed)
}

pub fn list_action_records(
    backups_root: &Path,
    action: &str,
) -> Result<Vec<BackupRecord>, AppError> {
    let mut records = Vec::new();
    for year in read_dirs(backups_root)? {
        for month in read_dirs(&year)? {
            let action_dir = month.join(action);
            if !action_dir.is_dir() {
                continue;
            }
            for directory in read_dirs(&action_dir)? {
                let manifest_path = directory.join(MANIFEST_NAME);
                if !manifest_path.is_file() {
                    continue;
                }
                let bytes =
                    std::fs::read(&manifest_path).map_err(|error| AppError::FileSystem {
                        operation: "read backup manifest".to_string(),
                        detail: error.to_string(),
                    })?;
                let manifest: BackupManifest =
                    serde_json::from_slice(&bytes).map_err(|_| AppError::FileSystem {
                        operation: "parse backup manifest".to_string(),
                        detail: manifest_path.display().to_string(),
                    })?;
                let mut record = BackupRecord {
                    manifest,
                    directory,
                };
                // 清单里的 `restorable` 是创建时写下的乐观值。列表必须按当前
                // 内容复验，否则被改坏的备份仍会显示「可恢复」（实测如此）。
                record.manifest.restorable = verify_backup(&record).unwrap_or(false);
                records.push(record);
            }
        }
    }
    Ok(records)
}

fn find_stored_payload(directory: &Path) -> Result<PathBuf, AppError> {
    for entry in std::fs::read_dir(directory).map_err(|error| AppError::FileSystem {
        operation: "read backup directory".to_string(),
        detail: error.to_string(),
    })? {
        let path = entry
            .map_err(|error| AppError::FileSystem {
                operation: "read backup entry".to_string(),
                detail: error.to_string(),
            })?
            .path();
        if path.is_file()
            && path
                .file_name()
                .map(|name| name != MANIFEST_NAME)
                .unwrap_or(false)
        {
            return Ok(path);
        }
    }
    Err(AppError::FileSystem {
        operation: "verify backup payload".to_string(),
        detail: "backup payload is missing".to_string(),
    })
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
    if target.as_os_str().is_empty() {
        return Err(AppError::FileSystem {
            operation: "validate backup target".to_string(),
            detail: "target is empty".to_string(),
        });
    }
    Ok(())
}

fn read_dirs(path: &Path) -> Result<Vec<PathBuf>, AppError> {
    let mut directories = Vec::new();
    if !path.is_dir() {
        return Ok(directories);
    }
    for entry in std::fs::read_dir(path).map_err(|error| AppError::FileSystem {
        operation: "read backup tree".to_string(),
        detail: error.to_string(),
    })? {
        let path = entry
            .map_err(|error| AppError::FileSystem {
                operation: "read backup entry".to_string(),
                detail: error.to_string(),
            })?
            .path();
        if path.is_dir() {
            directories.push(path);
        }
    }
    Ok(directories)
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

    fn create_backup(
        data_root: &Path,
        action: BackupAction,
        created: DateTime<Utc>,
        content: &[u8],
    ) -> BackupRecord {
        let target = data_root.join("target.txt");
        backup_file(data_root, action, &target, content, created, None).expect("create backup")
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

    /// 回归：列表必须按当前内容复验 `restorable`。
    /// 此前它直接回传清单里的乐观值，被改坏的备份仍显示「可恢复」。
    #[test]
    fn listing_reports_corrupted_backups_as_not_restorable() {
        let temp = tempfile::tempdir().expect("temporary data root");
        let target = temp.path().join("config.json");
        std::fs::write(&target, b"payload").expect("write target");
        backup_file(
            temp.path(),
            BackupAction::Upgrade,
            &target,
            b"payload",
            now(),
            None,
        )
        .expect("create backup");

        let listed = list_action_records(&temp.path().join("backups"), "upgrade").expect("list");
        assert_eq!(listed.len(), 1);
        assert!(listed[0].manifest.restorable, "未损坏的备份应仍显示可恢复");

        let payload = find_stored_payload(&listed[0].directory).expect("payload");
        std::fs::write(payload, b"changed").expect("corrupt payload");

        let listed = list_action_records(&temp.path().join("backups"), "upgrade").expect("list");
        assert_eq!(listed.len(), 1);
        assert!(!listed[0].manifest.restorable, "损坏的备份不得再显示可恢复");
    }

    #[test]
    fn cleanup_keeps_recent_20_and_30_days() {
        let temp = tempfile::tempdir().expect("temporary data root");
        for i in 0..24 {
            // 20 records inside the day window; 4 only survive by recent-count.
            let age = if i < 20 { i } else { i + 11 };
            let record = create_backup(
                temp.path(),
                BackupAction::Upgrade,
                old(age),
                format!("payload-{i}").as_bytes(),
            );
            assert!(record.directory.exists());
        }

        let removed = cleanup_retention(temp.path(), now(), MAX_PER_ACTION).expect("cleanup");
        assert_eq!(removed.len(), 4);
        let records = list_action_records(&temp.path().join("backups"), "upgrade").expect("list");
        assert_eq!(records.len(), 20);
        assert!(records.iter().all(|record| record.manifest.created_at
            >= old(20).to_rfc3339_opts(SecondsFormat::Secs, true)));
    }

    #[test]
    fn cleanup_applies_each_action_independently() {
        let temp = tempfile::tempdir().expect("temporary data root");
        for i in 0..22 {
            create_backup(temp.path(), BackupAction::Upgrade, old(i + 31), b"upgrade");
            create_backup(temp.path(), BackupAction::Import, old(i), b"import");
        }

        let removed = cleanup_retention(temp.path(), now(), MAX_PER_ACTION).expect("cleanup");
        let upgrades =
            list_action_records(&temp.path().join("backups"), "upgrade").expect("list upgrades");
        let imports =
            list_action_records(&temp.path().join("backups"), "import").expect("list imports");
        assert_eq!(upgrades.len(), 20);
        // Imports are entirely inside the 30-day window, so count alone does not limit them.
        assert_eq!(imports.len(), 22);
        assert_eq!(removed.len(), 2);
    }

    // 回归：「备份保留策略」偏好会覆盖默认保留份数，而不是被写死在模块常量里。
    #[test]
    fn cleanup_honors_caller_supplied_retention_count() {
        let temp = tempfile::tempdir().expect("temporary data root");
        for i in 0..8 {
            create_backup(
                temp.path(),
                BackupAction::Upgrade,
                old(i + 31),
                format!("payload-{i}").as_bytes(),
            );
        }

        let removed = cleanup_retention(temp.path(), now(), 5).expect("cleanup");
        assert_eq!(removed.len(), 3);
        let records = list_action_records(&temp.path().join("backups"), "upgrade").expect("list");
        assert_eq!(records.len(), 5);
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
