//! Read-only protection of the OpenCodex-owned native config.
//!
//! Desktop may read and back up the native configuration before it replaces
//! its own runtime program. It never writes, repairs, locks or restores that
//! file; OpenCodex owns every mutation of its configuration.

use crate::{
    infrastructure::locking::TargetFileLock,
    modules::backup::{self, safety, BackupAction},
    modules::model_config::schema_v2::{self, MatchErrorV2, Result},
};
use std::{
    io::Read,
    path::{Path, PathBuf},
};
use zeroize::Zeroizing;

const MANUAL_BACKUP_NOTE: &str = "OpenCodex 原生配置手动完整备份";
const UPGRADE_BACKUP_NOTE: &str = "OpenCodex 原生配置升级前完整备份";

#[derive(Clone)]
pub struct UpgradeRequest {
    pub home: PathBuf,
    pub program_source: Option<backup::NativeConfigSource>,
}

/// Holds Desktop's pending-transaction admission and a read lock on the native
/// file while the backup is verified. No native mutation lock is created.
pub struct Protection {
    _admission: Vec<TargetFileLock>,
    _target: TargetFileLock,
    home: PathBuf,
    source: Option<String>,
    backup: Option<backup::BackupRecord>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSummary {
    pub id: String,
    pub created_at: String,
    pub bytes: u64,
    pub purpose: String,
    pub runtime_version: Option<String>,
}

fn fault() -> MatchErrorV2 {
    schema_v2::error("native_config_backup_failed")
}

pub fn read(home: &Path) -> Result<Option<Zeroizing<Vec<u8>>>> {
    let path = home.join("config.json");
    safety::check_path(home, &path, true).map_err(|_| fault())?;
    match std::fs::symlink_metadata(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(fault()),
        Ok(_) => {
            let file = safety::open_file(&path, false).map_err(|_| fault())?;
            safety::require_single_link(&file).map_err(|_| fault())?;
            let mut bytes = Zeroizing::new(Vec::new());
            file.take(safety::MAX_PAYLOAD_BYTES + 1)
                .read_to_end(&mut bytes)
                .map_err(|_| fault())?;
            if bytes.len() as u64 > safety::MAX_PAYLOAD_BYTES {
                return Err(fault());
            }
            Ok(Some(bytes))
        }
    }
}

pub(crate) fn admission(root: &Path) -> Result<Vec<TargetFileLock>> {
    let names = [
        "saved-file-save-pending.json",
        "provider-rename-pending.json",
        "model-transaction-v2.pending.json",
        "credential-references-v2.pending.json",
    ];
    let mut locks = Vec::new();
    for name in names {
        let path = root.join("manager-state").join(name);
        safety::check_path(root, &path, true).map_err(|_| fault())?;
        safety::check_path(
            root,
            &crate::infrastructure::locking::lock_path_for(&path),
            true,
        )
        .map_err(|_| fault())?;
        locks.push(TargetFileLock::lock(&path).map_err(|_| fault())?);
    }
    for name in names {
        if std::fs::symlink_metadata(root.join("manager-state").join(name)).is_ok() {
            return Err(schema_v2::error("model_transaction_pending"));
        }
    }
    Ok(locks)
}

fn protect(root: &Path, request: &UpgradeRequest, upgrade: bool) -> Result<Protection> {
    let admission = admission(root)?;
    let target = TargetFileLock::lock(&request.home.join("config.json")).map_err(|_| fault())?;
    let bytes = read(&request.home)?;
    if bytes.is_none() && !upgrade {
        return Err(schema_v2::error("native_config_missing"));
    }
    let source = bytes.as_ref().map(|bytes| schema_v2::digest(bytes));
    let record = bytes
        .as_ref()
        .map(|bytes| {
            let record = backup::backup_file_with_source(
                root,
                if upgrade {
                    BackupAction::Upgrade
                } else {
                    BackupAction::ModelConfig
                },
                &request.home.join("config.json"),
                bytes,
                chrono::Utc::now(),
                Some(
                    if upgrade {
                        UPGRADE_BACKUP_NOTE
                    } else {
                        MANUAL_BACKUP_NOTE
                    }
                    .into(),
                ),
                request.program_source.clone(),
            )
            .map_err(|_| fault())?;
            let stored =
                std::fs::read(record.directory.join("config.json")).map_err(|_| fault())?;
            if stored != bytes.as_slice()
                || crate::infrastructure::hash::sha256_hex(&stored) != record.manifest.sha256
            {
                return Err(fault());
            }
            Ok(record)
        })
        .transpose()?;
    let protection = Protection {
        _admission: admission,
        _target: target,
        home: request.home.clone(),
        source,
        backup: record,
    };
    protection.verify()?;
    Ok(protection)
}

impl Protection {
    /// Never repairs drift, changes config, deletes a backup or claims success.
    pub fn verify(&self) -> Result<()> {
        let current = read(&self.home)?.map(|bytes| schema_v2::digest(&bytes));
        if current != self.source {
            return Err(schema_v2::error("native_config_backup_stale"));
        }
        if let Some(expected) = &self.backup {
            let stored =
                std::fs::read(expected.directory.join("config.json")).map_err(|_| fault())?;
            if crate::infrastructure::hash::sha256_hex(&stored) != expected.manifest.sha256 {
                return Err(fault());
            }
        }
        Ok(())
    }
}

pub fn protect_upgrade(root: &Path, request: &UpgradeRequest) -> Result<Protection> {
    protect(root, request, true)
}

pub fn create_manual(root: &Path, request: &UpgradeRequest) -> Result<BackupSummary> {
    let guard = protect(root, request, false)?;
    let record = guard
        .backup
        .as_ref()
        .ok_or_else(|| schema_v2::error("native_config_missing"))?;
    Ok(BackupSummary {
        id: record.manifest.backup_id.clone(),
        created_at: record.manifest.created_at.clone(),
        bytes: record.manifest.bytes,
        purpose: MANUAL_BACKUP_NOTE.into(),
        runtime_version: record
            .manifest
            .native_config
            .as_ref()
            .map(|s| s.runtime_version.clone()),
    })
}

/// Count Desktop's own manual native-config backups without reading secrets.
pub fn count_manual(root: &Path) -> Result<usize> {
    let records = safety::scan(&root.join("backups")).map_err(|_| fault())?;
    Ok(records
        .into_iter()
        .filter(|record| {
            record.manifest.action == BackupAction::ModelConfig
                && record.manifest.note.as_deref() == Some(MANUAL_BACKUP_NOTE)
        })
        .count())
}

/// Program provenance recorded with a native-config backup. It never grants a
/// write to the config being backed up.
pub fn program_source(
    runtime: &crate::modules::runtime::program_selection::SelectedProgram,
) -> backup::NativeConfigSource {
    backup::NativeConfigSource {
        runtime_version: runtime.selection.version.clone(),
        runtime_image_sha256: runtime.selection.image_sha256.clone(),
    }
}

/// Count Desktop's own pre-upgrade native-config backups.
pub fn count_upgrades(root: &Path) -> Result<usize> {
    let records = safety::scan(&root.join("backups")).map_err(|_| fault())?;
    Ok(records
        .into_iter()
        .filter(|record| {
            record.manifest.action == BackupAction::Upgrade
                && record.manifest.note.as_deref() == Some(UPGRADE_BACKUP_NOTE)
        })
        .count())
}
