//! Complete current-HOME config protection. This never exports secrets through IPC.
use super::*;

#[derive(Clone)]
pub struct UpgradeRequest {
    pub home: std::path::PathBuf,
    pub program_source: Option<backup::NativeConfigSource>,
}

/// Retains the official mutation lock and Desktop target lock through program
/// publication. A writer ignoring either lock still fails the exact-byte recheck.
pub struct Protection {
    _admission: Vec<TargetFileLock>,
    _official: port::ConfigMutation,
    _target: TargetFileLock,
    home: std::path::PathBuf,
    source: Option<String>,
    backup: Option<backup::BackupRecord>,
    root: std::path::PathBuf,
}

pub(super) fn read(home: &Path) -> Result<Option<Zeroizing<Vec<u8>>>> {
    let path = home.join("config.json");
    safety::check_path(home, &path, true).map_err(|_| error("native_config_backup_failed"))?;
    match std::fs::symlink_metadata(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(error("native_config_backup_failed")),
        Ok(_) => {
            let file = safety::open_file(&path, false)
                .map_err(|_| error("native_config_backup_failed"))?;
            safety::require_single_link(&file).map_err(|_| error("native_config_backup_failed"))?;
            let mut bytes = Zeroizing::new(Vec::new());
            file.take(safety::MAX_PAYLOAD_BYTES + 1)
                .read_to_end(&mut bytes)
                .map_err(|_| error("native_config_backup_failed"))?;
            if bytes.len() as u64 > safety::MAX_PAYLOAD_BYTES {
                return Err(error("native_config_backup_failed"));
            }
            Ok(Some(bytes))
        }
    }
}

fn protect(root: &Path, request: &UpgradeRequest, upgrade: bool) -> Result<Protection> {
    root_revision(root, &request.home)?;
    let admission = pending_admission(root)?;
    let official = port::ConfigMutation::acquire(&request.home)
        .map_err(|_| error("native_config_backup_failed"))?;
    safety::check_path(&request.home, &request.home.join("config.json.lock"), true)
        .map_err(|_| error("native_config_backup_failed"))?;
    let target = TargetFileLock::lock(&request.home.join("config.json"))
        .map_err(|_| error("native_config_backup_failed"))?;
    let bytes = read(&request.home)?;
    if bytes.is_none() && !upgrade {
        return Err(error("native_config_missing"));
    }
    let source = bytes.as_ref().map(|bytes| digest(bytes));
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
            .map_err(|_| error("native_config_backup_failed"))?;
            let (verified, _, stored) =
                backup_source(root, &request.home, &record.manifest.backup_id)
                    .map_err(|_| error("native_config_backup_failed"))?;
            if record != verified || stored.as_slice() != bytes.as_slice() {
                return Err(error("native_config_backup_failed"));
            }
            Ok(record)
        })
        .transpose()?;
    let protection = Protection {
        _admission: admission,
        _official: official,
        _target: target,
        home: request.home.clone(),
        source,
        backup: record,
        root: root.into(),
    };
    protection.verify()?;
    Ok(protection)
}

impl Protection {
    /// Never repairs drift, changes config, deletes a backup or claims success.
    pub fn verify(&self) -> Result<()> {
        if read(&self.home)?.as_ref().map(|bytes| digest(bytes)) != self.source {
            return Err(error("native_config_backup_stale"));
        }
        if let Some(expected) = &self.backup {
            let (record, _, _) =
                backup_source(&self.root, &self.home, &expected.manifest.backup_id)
                    .map_err(|_| error("native_config_backup_failed"))?;
            if &record != expected {
                return Err(error("native_config_backup_failed"));
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
        .ok_or_else(|| error("native_config_missing"))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture {
        _temp: tempfile::TempDir,
        root: std::path::PathBuf,
        request: UpgradeRequest,
        original: Vec<u8>,
    }
    impl Fixture {
        fn new() -> Self {
            let temp = tempfile::tempdir().unwrap();
            let base = temp.path().canonicalize().unwrap();
            let root = base.join("desktop");
            let home = base.join("opencodex/home");
            std::fs::create_dir_all(root.join("manager-state")).unwrap();
            std::fs::create_dir_all(&home).unwrap();
            let original = br#"{ "port": 41234, "providers": {"Source":{"adapter":"openai-chat","baseUrl":"https://fixture.invalid","models":["m"],"apiKey":"fixture-only","apiKeyPool":[{"id":"one","key":"fixture-one"}],"headers":{"X-Test":"fixture-header"}}}, "global":true }"#.to_vec();
            std::fs::write(home.join("config.json"), &original).unwrap();
            Self {
                _temp: temp,
                root,
                request: UpgradeRequest {
                    home,
                    program_source: Some(backup::NativeConfigSource {
                        runtime_version: "2.99.0".into(),
                        runtime_image_sha256: "a".repeat(64),
                    }),
                },
                original,
            }
        }
    }
    #[test]
    fn manual_and_upgrade_protect_exact_full_bytes_with_observed_version_and_rollback() {
        for upgrade in [false, true] {
            let f = Fixture::new();
            let record = if upgrade {
                let protected = protect_upgrade(&f.root, &f.request).unwrap();
                protected.verify().unwrap();
                protected.backup.as_ref().unwrap().clone()
            } else {
                let summary = create_manual(&f.root, &f.request).unwrap();
                assert_eq!(summary.runtime_version.as_deref(), Some("2.99.0"));
                assert_eq!(summary.bytes, f.original.len() as u64);
                assert!(!serde_json::to_string(&summary)
                    .unwrap()
                    .contains("fixture-one"));
                backup_source(&f.root, &f.request.home, &summary.id)
                    .unwrap()
                    .0
            };
            assert_eq!(record.manifest.native_config, f.request.program_source);
            assert_eq!(
                record.manifest.action,
                if upgrade {
                    BackupAction::Upgrade
                } else {
                    BackupAction::ModelConfig
                }
            );
            assert_eq!(
                std::fs::read(record.directory.join("config.json")).unwrap(),
                f.original
            );
            assert_eq!(list_backups(&f.root, &f.request.home).unwrap().len(), 1);
            std::fs::write(
                f.request.home.join("config.json"),
                br#"{"port":43456,"providers":{}}"#,
            )
            .unwrap();
            let (selection, preview) =
                prepare_rollback(&f.root, &f.request.home, &record.manifest.backup_id, |_| {
                    Ok(())
                })
                .unwrap();
            assert_eq!(preview.backup_port, 41234);
            apply(&f.root, &f.request.home, selection, || Ok(())).unwrap();
            assert_eq!(
                std::fs::read(f.request.home.join("config.json")).unwrap(),
                f.original
            );
        }
    }
    #[test]
    fn protection_detects_ignored_lock_writers_and_retains_the_original_backup() {
        let f = Fixture::new();
        let guard = protect_upgrade(&f.root, &f.request).unwrap();
        let record = guard.backup.as_ref().unwrap();
        std::fs::write(f.request.home.join("config.json"), br#"{"port":43456}"#).unwrap();
        assert_eq!(
            guard.verify().err().unwrap().code,
            "native_config_backup_stale"
        );
        assert_eq!(
            std::fs::read(record.directory.join("config.json")).unwrap(),
            f.original
        );
        assert_eq!(
            std::fs::read(f.request.home.join("config.json")).unwrap(),
            br#"{"port":43456}"#
        );
    }
    #[test]
    fn backup_failure_and_pending_transactions_never_modify_original_config() {
        let f = Fixture::new();
        std::fs::write(f.root.join("backups"), b"block backup").unwrap();
        assert_eq!(
            create_manual(&f.root, &f.request).err().unwrap().code,
            "native_config_backup_failed"
        );
        assert_eq!(
            std::fs::read(f.request.home.join("config.json")).unwrap(),
            f.original
        );
        let f = Fixture::new();
        let pending = f.root.join("manager-state/saved-file-save-pending.json");
        std::fs::write(&pending, b"retain pending").unwrap();
        assert_eq!(
            create_manual(&f.root, &f.request).err().unwrap().code,
            "model_transaction_pending"
        );
        assert_eq!(std::fs::read(pending).unwrap(), b"retain pending");
        assert!(!f.root.join("backups").exists());
    }
    #[test]
    fn absent_config_is_an_explicit_upgrade_fact_and_not_a_fabricated_backup() {
        let f = Fixture::new();
        let config = f.request.home.join("config.json");
        std::fs::remove_file(&config).unwrap();
        assert_eq!(
            create_manual(&f.root, &f.request).err().unwrap().code,
            "native_config_missing"
        );
        let guard = protect_upgrade(&f.root, &f.request).unwrap();
        assert!(guard.backup.is_none());
        assert!(!f.root.join("backups").exists());
        std::fs::write(config, b"new config after absence was observed").unwrap();
        assert_eq!(
            guard.verify().err().unwrap().code,
            "native_config_backup_stale"
        );
    }
    #[test]
    fn malformed_config_can_be_protected_without_pretending_it_is_restorable() {
        let f = Fixture::new();
        let bad = b"{invalid original file";
        std::fs::write(f.request.home.join("config.json"), bad).unwrap();
        let summary = create_manual(&f.root, &f.request).unwrap();
        let (_, _, stored) = backup_source(&f.root, &f.request.home, &summary.id).unwrap();
        assert_eq!(stored.as_slice(), bad);
        assert!(
            prepare_rollback(&f.root, &f.request.home, &summary.id, |_| panic!(
                "invalid JSON cannot pass validation"
            ))
            .is_err()
        );
    }
    #[test]
    fn full_rollback_recovers_missing_or_malformed_targets_and_preserves_unspecified_port() {
        for (absent, unspecified) in [(true, false), (false, false), (false, true)] {
            let f = Fixture::new();
            let target = f.request.home.join("config.json");
            let original = if unspecified {
                br#"{ "providers":{} }"#.to_vec()
            } else {
                f.original.clone()
            };
            std::fs::write(&target, &original).unwrap();
            let backup = create_manual(&f.root, &f.request).unwrap();
            let broken = b"{malformed configuration";
            if absent {
                std::fs::remove_file(&target).unwrap();
            } else {
                std::fs::write(&target, broken).unwrap();
            }
            let (selected, preview) =
                prepare_rollback(&f.root, &f.request.home, &backup.id, |_| Ok(())).unwrap();
            assert_eq!(preview.current_port, None);
            assert_eq!(preview.backup_port, if unspecified { 0 } else { 41234 });
            let restored = apply(&f.root, &f.request.home, selected, || Ok(())).unwrap();
            assert_eq!(std::fs::read(&target).unwrap(), original);
            assert_eq!(restored.backup_id.is_none(), absent);
            if let Some(id) = restored.backup_id {
                assert_eq!(
                    backup_source(&f.root, &f.request.home, &id)
                        .unwrap()
                        .2
                        .as_slice(),
                    broken
                );
            }
        }
    }
}
