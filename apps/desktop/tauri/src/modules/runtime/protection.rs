//! Durable panel-update backup association. No networking or process execution.
//! Callers own writer admission, the runtime mutation lease, and the shared W2 guard.
use super::{install, MANAGED_ENTRY_RELATIVE, MANIFEST_FILENAME, OFFICIAL_PACKAGE};
use crate::{
    errors::{AppError, AppResult},
    modules::{
        backup::{
            manager::{PreferencesProtectionGuard, PreferencesProtectionRef},
            safety,
        },
        notifications::registry::{self, PathId},
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

fn fail() -> AppError {
    AppError::RuntimeManaged {
        code: "protection_reconciliation_pending".into(),
        detail: "上一次面板更新的保护备份尚未完成核验；已保留备份，请核对安装记录后重试".into(),
    }
}
fn state_path(root: &Path) -> PathBuf {
    root.join(
        registry::registry()
            .expect("validated event registry")
            .path(PathId::RuntimeProtection),
    )
}
fn read_owned(path: &Path, limit: u64) -> AppResult<Vec<u8>> {
    let file = safety::open_file(path, false)?;
    safety::require_single_link(&file)?;
    if file.metadata().map_err(|_| fail())?.len() > limit {
        return Err(fail());
    }
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| fail())?;
    if bytes.len() as u64 > limit {
        return Err(fail());
    }
    Ok(bytes)
}
fn digest(root: &Path, path: &Path, limit: u64) -> AppResult<String> {
    safety::check_path(root, path, false)?;
    Ok(format!("{:x}", Sha256::digest(read_owned(path, limit)?)))
}
fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn valid_absolute(path: &Path) -> bool {
    path.is_absolute()
        && !path
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
}
fn valid_bin(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 1024
        && !path.contains(['\\', ':'])
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}

/// Written to the installed manifest only after package probe and final activation.
/// Hashes detect changed metadata/launcher/script; this is not a package signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActivationProof {
    attempt_id: String,
    bin: String,
    entry_sha256: String,
    package_sha256: String,
    bin_sha256: String,
}
impl ActivationProof {
    fn validate(&self) -> AppResult<()> {
        if uuid::Uuid::parse_str(&self.attempt_id).is_err()
            || !valid_bin(&self.bin)
            || !valid_digest(&self.entry_sha256)
            || !valid_digest(&self.package_sha256)
            || !valid_digest(&self.bin_sha256)
        {
            return Err(fail());
        }
        Ok(())
    }
}
pub(super) fn activation_proof(
    root: &Path,
    prefix: &Path,
    attempt_id: &str,
    bin: &str,
) -> AppResult<ActivationProof> {
    if !valid_bin(bin) {
        return Err(fail());
    }
    let package = prefix.join(install::PACKAGE_SUBPATH);
    let proof = ActivationProof {
        attempt_id: attempt_id.into(),
        bin: bin.into(),
        entry_sha256: digest(root, &root.join(MANAGED_ENTRY_RELATIVE), 64 * 1024)?,
        package_sha256: digest(prefix, &package.join("package.json"), 64 * 1024)?,
        bin_sha256: digest(prefix, &package.join(bin), 16 * 1024 * 1024)?,
    };
    proof.validate()?;
    Ok(proof)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PanelProtection {
    schema: u32,
    attempt_id: String,
    protection: PreferencesProtectionRef,
    prefix: PathBuf,
    version: String,
    node: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    verified: Option<ActivationProof>,
}
impl PanelProtection {
    /// The guard serializes load/CAS/write with cleanup, restore and other updates.
    pub(crate) fn arm(
        root: &Path,
        guard: &PreferencesProtectionGuard,
        prefix: &Path,
        version: &str,
        node: Option<&Path>,
    ) -> AppResult<Option<Self>> {
        reconcile_pending(root, guard)?;
        let Some(protection) = guard.protection_ref(root)? else {
            return Ok(None);
        };
        let record = Self {
            schema: 1,
            attempt_id: uuid::Uuid::new_v4().to_string(),
            protection,
            prefix: prefix.into(),
            version: version.into(),
            node: node.map(Path::to_path_buf),
            verified: None,
        };
        record.validate()?;
        record.save(root)?;
        Ok(Some(record))
    }
    pub(crate) fn attempt_id(&self) -> String {
        self.attempt_id.clone()
    }
    fn validate(&self) -> AppResult<()> {
        self.protection.validate()?;
        if self.schema != 1
            || uuid::Uuid::parse_str(&self.attempt_id).is_err()
            || !valid_absolute(&self.prefix)
            || self.prefix.as_os_str().len() > 4096
            || !crate::modules::update::handoff::valid_version(&self.version)
            || self
                .node
                .as_ref()
                .is_some_and(|p| !valid_absolute(p) || p.as_os_str().len() > 4096)
        {
            return Err(fail());
        }
        if let Some(proof) = &self.verified {
            proof.validate()?;
            if proof.attempt_id != self.attempt_id {
                return Err(fail());
            }
        }
        Ok(())
    }
    pub(crate) fn load(root: &Path) -> AppResult<Option<Self>> {
        let path = state_path(root);
        safety::check_path(root, &path, true)?;
        match std::fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(fail()),
            Ok(_) => {}
        }
        let record: Self =
            serde_json::from_slice(&read_owned(&path, 16 * 1024)?).map_err(|_| fail())?;
        record.validate()?;
        Ok(Some(record))
    }
    fn save(&self, root: &Path) -> AppResult<()> {
        self.validate()?;
        let path = state_path(root);
        safety::check_path(root, &path, true)?;
        let bytes = serde_json::to_vec(self).map_err(|_| fail())?;
        if bytes.len() > 16 * 1024 {
            return Err(fail());
        }
        crate::infrastructure::atomic_write::atomic_write(&path, &bytes, 0o600)?;
        safety::sync_directories(root, path.parent().ok_or_else(fail)?)?;
        if Self::load(root)?.as_ref() != Some(self) {
            return Err(fail());
        }
        Ok(())
    }
    fn activation(&self, root: &Path) -> AppResult<ActivationProof> {
        // Same prefix lock as the installer, without following a link at the lock path.
        let parent = self.prefix.parent().ok_or_else(fail)?;
        let lock_path = crate::infrastructure::locking::lock_path_for(&self.prefix);
        safety::check_path(parent, &lock_path, true)?;
        let lock = safety::open_file(&lock_path, true)?;
        safety::require_single_link(&lock)?;
        lock.try_lock().map_err(|_| fail())?;
        safety::check_path(parent, &self.prefix, false)?;
        let path = self.prefix.join(MANIFEST_FILENAME);
        safety::check_path(&self.prefix, &path, false)?;
        let manifest: install::RuntimeManifest =
            serde_json::from_slice(&read_owned(&path, 64 * 1024)?).map_err(|_| fail())?;
        let proof = manifest.activation.ok_or_else(fail)?;
        proof.validate()?;
        if manifest.package != OFFICIAL_PACKAGE
            || manifest.version != self.version
            || manifest.source != install::InstallSourceKind::Registry
            || proof.attempt_id != self.attempt_id
            || activation_proof(root, &self.prefix, &self.attempt_id, &proof.bin)? != proof
        {
            return Err(fail());
        }
        let bin = self.prefix.join(install::PACKAGE_SUBPATH).join(&proof.bin);
        // The package bin is a Node script, not an .exe/.cmd on Windows.
        // Validate the runnable launcher; the script was bounded/read/hashed above.
        if crate::modules::runtime::paths::validate_executable(&root.join(MANAGED_ENTRY_RELATIVE))
            .is_err()
        {
            return Err(fail());
        }
        let package: serde_json::Value = serde_json::from_slice(&read_owned(
            &self
                .prefix
                .join(install::PACKAGE_SUBPATH)
                .join("package.json"),
            64 * 1024,
        )?)
        .map_err(|_| fail())?;
        if package["name"].as_str() != Some(OFFICIAL_PACKAGE)
            || package["version"].as_str() != Some(self.version.as_str())
            || install::entry_script(self.node.as_deref(), &bin).as_bytes()
                != read_owned(&root.join(MANAGED_ENTRY_RELATIVE), 64 * 1024)?
        {
            return Err(fail());
        }
        Ok(proof)
    }
    /// Persist verification before releasing this exact backup; repeated calls are safe.
    pub(crate) fn reconcile(
        &mut self,
        root: &Path,
        guard: &PreferencesProtectionGuard,
    ) -> AppResult<()> {
        self.reconcile_with(root, guard, |record| record.save(root))
    }
    fn reconcile_with(
        &mut self,
        root: &Path,
        guard: &PreferencesProtectionGuard,
        persist: impl FnOnce(&Self) -> AppResult<()>,
    ) -> AppResult<()> {
        if Self::load(root)?.as_ref() != Some(self) {
            return Err(fail());
        }
        let mut verified = self.clone();
        if verified.verified.is_none() {
            verified.verified = Some(self.activation(root)?);
        }
        guard.reconcile_panel_recorded(root, &self.protection, self.verified.is_some(), || {
            if Self::load(root)?.as_ref() != Some(self) {
                return Err(fail());
            }
            persist(&verified)
        })?;
        *self = verified;
        self.discard(root)
    }
    /// Terminal installation failure can forget the attempt, never release its backup.
    pub(crate) fn discard(&self, root: &Path) -> AppResult<()> {
        if Self::load(root)?.as_ref() != Some(self) {
            return Err(fail());
        }
        let path = state_path(root);
        std::fs::remove_file(&path).map_err(|_| fail())?;
        safety::sync_directories(root, path.parent().ok_or_else(fail)?)
    }
}
pub(crate) fn reconcile_pending(root: &Path, guard: &PreferencesProtectionGuard) -> AppResult<()> {
    if let Some(mut record) = PanelProtection::load(root)? {
        record.reconcile(root, guard)?;
    }
    Ok(())
}
/// A bounded read projection, never an assertion that activation has been verified.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtectionStatus {
    pub state: ProtectionState,
    pub version: Option<String>,
    pub backup_id: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtectionState {
    None,
    Pending,
    VerifiedPending,
    Unreadable,
}
pub(crate) fn status(root: &Path) -> ProtectionStatus {
    match PanelProtection::load(root) {
        Ok(Some(record)) => ProtectionStatus {
            state: if record.verified.is_some() {
                ProtectionState::VerifiedPending
            } else {
                ProtectionState::Pending
            },
            version: Some(record.version),
            backup_id: Some(record.protection.backup_id),
        },
        result => ProtectionStatus {
            state: if result.is_ok() {
                ProtectionState::None
            } else {
                ProtectionState::Unreadable
            },
            version: None,
            backup_id: None,
        },
    }
}
/// Caller owns storage admission and the runtime mutation lease. No package command
/// or network request; malformed receipts are rejected before acquiring a write guard.
pub(crate) fn retry(
    root: &Path,
    mut terminal: impl FnMut(&[u8], bool),
) -> AppResult<ProtectionStatus> {
    if let Some(mut record) = PanelProtection::load(root)? {
        let guard = crate::modules::backup::manager::acquire_preferences_transaction(root)?;
        // Stable across verification persistence and retries. No raw path or
        // payload enters notification history; the event adapter hashes this.
        let candidate =
            serde_json::to_vec(&(&record.attempt_id, &record.version, &record.protection))
                .map_err(|_| fail())?;
        let result = record.reconcile(root, &guard);
        terminal(&candidate, result.is_ok());
        result?;
    }
    Ok(status(root))
}
/// Binding changes cannot carry an unresolved absolute installation association.
/// This is read-only: startup/install retries perform reconciliation under their guard.
pub(crate) fn ensure_binding_ready(root: &Path) -> AppResult<()> {
    if PanelProtection::load(root)?.is_some() {
        return Err(fail());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::{
        backup::{self, manager, BackupAction, BackupRecord},
        preferences::{Preferences, PreferencesStore},
    };

    fn fixture() -> (
        tempfile::TempDir,
        PreferencesProtectionGuard,
        PanelProtection,
    ) {
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        PreferencesStore::new(root.path())
            .save(&Preferences::default())
            .unwrap();
        let guard = manager::begin_preferences_protection(root.path(), true).unwrap();
        let receipt = PanelProtection::arm(
            root.path(),
            &guard,
            &root.path().join(super::super::MANAGED_PREFIX_RELATIVE),
            "0.3.1",
            None,
        )
        .unwrap()
        .unwrap();
        (root, guard, receipt)
    }
    // Synthetic activation bytes test reconciliation; real installer has a separate probe test.
    fn activate(root: &Path, receipt: &PanelProtection, attempt: &str) {
        let package = receipt.prefix.join(install::PACKAGE_SUBPATH);
        std::fs::create_dir_all(package.join("bin")).unwrap();
        std::fs::write(
            package.join("package.json"),
            serde_json::to_vec(&serde_json::json!({
                "name": OFFICIAL_PACKAGE, "version": receipt.version, "bin": {"ocx": "bin/ocx.js"},
            }))
            .unwrap(),
        )
        .unwrap();
        let bin = package.join("bin/ocx.js");
        std::fs::write(&bin, b"console.log('fixture');\n").unwrap();
        let entry = root.join(MANAGED_ENTRY_RELATIVE);
        std::fs::create_dir_all(entry.parent().unwrap()).unwrap();
        std::fs::write(&entry, install::entry_script(receipt.node.as_deref(), &bin)).unwrap();
        crate::infrastructure::platform::set_mode(&entry, 0o755).unwrap();
        let manifest = install::RuntimeManifest {
            activation: Some(
                activation_proof(root, &receipt.prefix, attempt, "bin/ocx.js").unwrap(),
            ),
            package: OFFICIAL_PACKAGE.into(),
            version: receipt.version.clone(),
            tarball_sha256: "a".repeat(64),
            installed_at: "2026-10-10T00:00:00Z".into(),
            npm_path: String::new(),
            scripts_enabled: false,
            source: install::InstallSourceKind::Registry,
        };
        std::fs::write(
            receipt.prefix.join(MANIFEST_FILENAME),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
    }
    fn backup(root: &Path, receipt: &PanelProtection) -> BackupRecord {
        backup::list_action_records(
            &root.join("backups"),
            BackupAction::PreferencesProtection.as_str(),
        )
        .unwrap()
        .into_iter()
        .find(|r| r.manifest.backup_id == receipt.protection.backup_id)
        .unwrap()
    }
    fn assert_active(root: &Path, receipt: &PanelProtection) {
        assert_eq!(
            backup(root, receipt)
                .manifest
                .management
                .unwrap()
                .transaction_state
                .as_deref(),
            Some("active")
        );
    }
    fn assert_committed(root: &Path, receipt: &PanelProtection) {
        assert_eq!(
            backup(root, receipt)
                .manifest
                .management
                .unwrap()
                .transaction_state
                .as_deref(),
            Some("committed")
        );
    }

    #[test]
    fn read_status_does_not_verify_or_release_valid_receipt() {
        let (root, _guard, receipt) = fixture();
        let original = std::fs::read(state_path(root.path())).unwrap();
        let pending = status(root.path());
        assert_eq!(pending.state, ProtectionState::Pending);
        assert!(ensure_binding_ready(root.path()).is_err());
        assert_eq!(pending.version.as_deref(), Some("0.3.1"));
        assert_eq!(
            pending.backup_id,
            Some(receipt.protection.backup_id.clone())
        );
        assert_eq!(std::fs::read(state_path(root.path())).unwrap(), original);
        assert_active(root.path(), &receipt);
        let mut verified = receipt.clone();
        activate(root.path(), &verified, &verified.attempt_id);
        verified.verified = Some(verified.activation(root.path()).unwrap());
        verified.save(root.path()).unwrap();
        assert_eq!(status(root.path()).state, ProtectionState::VerifiedPending);
        let verified_bytes = std::fs::read(state_path(root.path())).unwrap();
        assert!(ensure_binding_ready(root.path()).is_err());
        assert_eq!(
            std::fs::read(state_path(root.path())).unwrap(),
            verified_bytes
        );
        assert_active(root.path(), &receipt);
    }

    #[test]
    fn corrupt_receipt_is_reported_without_exposing_fields_or_overwriting_bytes() {
        let (root, guard, receipt) = fixture();
        drop(guard);
        let original = br#"{"version":"secret","prefix":"untrusted"}"#;
        std::fs::write(state_path(root.path()), original).unwrap();
        let reported = status(root.path());
        assert_eq!(reported.state, ProtectionState::Unreadable);
        assert!(reported.version.is_none() && reported.backup_id.is_none());
        assert!(ensure_binding_ready(root.path()).is_err());
        assert!(retry(root.path(), |_, _| panic!("corruption is pre-admission")).is_err());
        assert_eq!(std::fs::read(state_path(root.path())).unwrap(), original);
        assert_active(root.path(), &receipt);
    }

    #[test]
    fn explicit_retry_retains_inconclusive_receipt_then_releases_only_after_activation() {
        let (root, guard, receipt) = fixture();
        drop(guard);
        let original = std::fs::read(state_path(root.path())).unwrap();
        let mut events = Vec::new();
        assert!(retry(root.path(), |candidate, succeeded| {
            events.push((candidate.to_vec(), succeeded));
        })
        .is_err());
        assert_eq!(std::fs::read(state_path(root.path())).unwrap(), original);
        assert_active(root.path(), &receipt);
        activate(root.path(), &receipt, &receipt.attempt_id);
        assert_eq!(
            retry(root.path(), |candidate, succeeded| {
                events.push((candidate.to_vec(), succeeded));
            })
            .unwrap()
            .state,
            ProtectionState::None
        );
        assert_committed(root.path(), &receipt);
        assert_eq!(events.len(), 2);
        assert!(!events[0].1 && events[1].1);
        assert_eq!(events[0].0, events[1].0);
        assert_eq!(
            retry(root.path(), |_, _| panic!("no receipt has no event"))
                .unwrap()
                .state,
            ProtectionState::None
        );
    }

    #[test]
    fn absent_receipt_retry_does_not_initialize_storage() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(status(root.path()).state, ProtectionState::None);
        assert!(ensure_binding_ready(root.path()).is_ok());
        assert_eq!(
            retry(root.path(), |_, _| panic!("no receipt has no event"))
                .unwrap()
                .state,
            ProtectionState::None
        );
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    }

    #[test]
    fn verified_activation_releases_exact_backup_and_preserves_independent_pin() {
        let (root, guard, mut receipt) = fixture();
        activate(root.path(), &receipt, &receipt.attempt_id);
        let mut record = backup(root.path(), &receipt);
        record.manifest.management.as_mut().unwrap().pinned = true;
        std::fs::write(
            record.directory.join(backup::MANIFEST_NAME),
            serde_json::to_vec(&record.manifest).unwrap(),
        )
        .unwrap();
        receipt.reconcile(root.path(), &guard).unwrap();
        assert_committed(root.path(), &receipt);
        assert!(
            backup(root.path(), &receipt)
                .manifest
                .management
                .unwrap()
                .pinned
        );
        assert!(PanelProtection::load(root.path()).unwrap().is_none());
        reconcile_pending(root.path(), &guard).unwrap();
    }

    #[test]
    fn verification_write_failure_keeps_receipt_and_backup_for_restart_retry() {
        let (root, guard, mut receipt) = fixture();
        activate(root.path(), &receipt, &receipt.attempt_id);
        assert!(receipt
            .reconcile_with(root.path(), &guard, |_| Err(fail()))
            .is_err());
        assert_active(root.path(), &receipt);
        assert_eq!(PanelProtection::load(root.path()).unwrap(), Some(receipt));
        drop(guard);
        let guard = manager::acquire_preferences_transaction(root.path()).unwrap();
        reconcile_pending(root.path(), &guard).unwrap();
        assert!(PanelProtection::load(root.path()).unwrap().is_none());
    }

    #[test]
    fn persisted_verification_survives_backup_commit_failure_and_retries_without_package() {
        let (root, guard, mut receipt) = fixture();
        activate(root.path(), &receipt, &receipt.attempt_id);
        let manifest_path = backup(root.path(), &receipt)
            .directory
            .join(backup::MANIFEST_NAME);
        let original = std::fs::read(&manifest_path).unwrap();
        assert!(receipt
            .reconcile_with(root.path(), &guard, |verified| {
                verified.save(root.path())?;
                // Actual filesystem failure after the durable proof, before manifest commit.
                std::fs::remove_file(&manifest_path).unwrap();
                std::fs::create_dir(&manifest_path).unwrap();
                Ok(())
            })
            .is_err());
        assert!(PanelProtection::load(root.path())
            .unwrap()
            .unwrap()
            .verified
            .is_some());
        std::fs::remove_dir(&manifest_path).unwrap();
        std::fs::write(&manifest_path, original).unwrap();
        assert_active(root.path(), &receipt);
        std::fs::remove_dir_all(&receipt.prefix).unwrap();
        drop(guard);
        let guard = manager::acquire_preferences_transaction(root.path()).unwrap();
        reconcile_pending(root.path(), &guard).unwrap();
        assert_committed(root.path(), &receipt);
    }

    #[test]
    fn interrupted_receipt_removal_is_idempotent_after_commit_and_after_expiry() {
        for expire in [false, true] {
            let (root, guard, mut receipt) = fixture();
            activate(root.path(), &receipt, &receipt.attempt_id);
            receipt.verified = Some(receipt.activation(root.path()).unwrap());
            receipt.save(root.path()).unwrap();
            guard
                .reconcile_panel_recorded(root.path(), &receipt.protection, true, || Ok(()))
                .unwrap();
            assert_committed(root.path(), &receipt);
            if expire {
                std::fs::remove_dir_all(backup(root.path(), &receipt).directory).unwrap();
            }
            receipt.reconcile(root.path(), &guard).unwrap();
            assert!(PanelProtection::load(root.path()).unwrap().is_none());
        }
    }

    #[test]
    fn missing_final_activation_blocks_retry_overwrite_and_preserves_backup() {
        let (root, guard, mut receipt) = fixture();
        activate(root.path(), &receipt, &receipt.attempt_id);
        let path = receipt.prefix.join(MANIFEST_FILENAME);
        let mut manifest: install::RuntimeManifest =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        manifest.activation = None;
        std::fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        assert!(receipt.reconcile(root.path(), &guard).is_err());
        assert!(
            PanelProtection::arm(root.path(), &guard, &receipt.prefix, &receipt.version, None)
                .is_err()
        );
        assert_eq!(
            PanelProtection::load(root.path()).unwrap(),
            Some(receipt.clone())
        );
        assert_active(root.path(), &receipt);
    }

    #[test]
    fn same_version_different_attempt_cannot_release_previous_backup() {
        let (root, guard, mut receipt) = fixture();
        activate(root.path(), &receipt, &uuid::Uuid::new_v4().to_string());
        assert!(receipt.reconcile(root.path(), &guard).is_err());
        assert_active(root.path(), &receipt);
        activate(root.path(), &receipt, &receipt.attempt_id);
        receipt.reconcile(root.path(), &guard).unwrap();
    }

    #[test]
    fn disappeared_receipt_cannot_release_the_captured_backup() {
        let (root, guard, mut receipt) = fixture();
        activate(root.path(), &receipt, &receipt.attempt_id);
        std::fs::remove_file(state_path(root.path())).unwrap();
        assert!(receipt.reconcile(root.path(), &guard).is_err());
        assert_active(root.path(), &receipt);
    }

    #[test]
    fn replaced_receipt_and_backup_digest_cannot_release_protection() {
        let (root, guard, mut receipt) = fixture();
        activate(root.path(), &receipt, &receipt.attempt_id);
        let mut replacement = receipt.clone();
        replacement.attempt_id = uuid::Uuid::new_v4().to_string();
        replacement.save(root.path()).unwrap();
        assert!(receipt.reconcile(root.path(), &guard).is_err());
        assert!(receipt.discard(root.path()).is_err());
        receipt.save(root.path()).unwrap();
        receipt.protection.sha256 = "b".repeat(64);
        receipt.save(root.path()).unwrap();
        assert!(receipt.reconcile(root.path(), &guard).is_err());
        assert_active(root.path(), &receipt);
    }

    #[test]
    fn changed_activation_inputs_never_release_protection() {
        for changed in [
            "package.json",
            "bin/ocx.js",
            "launcher",
            "version",
            "source",
        ] {
            let (root, guard, mut receipt) = fixture();
            activate(root.path(), &receipt, &receipt.attempt_id);
            match changed {
                "launcher" => {
                    std::fs::write(root.path().join(MANAGED_ENTRY_RELATIVE), b"replaced").unwrap()
                }
                "version" | "source" => {
                    let path = receipt.prefix.join(MANIFEST_FILENAME);
                    let mut manifest: install::RuntimeManifest =
                        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
                    if changed == "version" {
                        manifest.version = "0.3.2".into();
                    } else {
                        manifest.source = install::InstallSourceKind::Offline;
                    }
                    std::fs::write(path, serde_json::to_vec(&manifest).unwrap()).unwrap();
                }
                name => std::fs::write(
                    receipt.prefix.join(install::PACKAGE_SUBPATH).join(name),
                    b"replaced",
                )
                .unwrap(),
            }
            assert!(receipt.reconcile(root.path(), &guard).is_err(), "{changed}");
            assert_active(root.path(), &receipt);
        }
    }

    #[test]
    fn strict_receipt_validation_rejects_unknown_corrupt_and_unbounded_fields() {
        let (root, _guard, receipt) = fixture();
        let good = serde_json::to_value(&receipt).unwrap();
        for bad in [
            serde_json::json!({"schema": 99}),
            {
                let mut v = good.clone();
                v["extra"] = true.into();
                v
            },
            {
                let mut v = good.clone();
                v["prefix"] = "relative".into();
                v
            },
            {
                let mut v = good.clone();
                v["version"] = "latest".into();
                v
            },
            {
                let mut v = good.clone();
                v["attempt_id"] = "bad".into();
                v
            },
        ] {
            std::fs::write(state_path(root.path()), serde_json::to_vec(&bad).unwrap()).unwrap();
            assert!(PanelProtection::load(root.path()).is_err());
            assert_active(root.path(), &receipt);
        }
        std::fs::write(state_path(root.path()), vec![b' '; 16 * 1024 + 1]).unwrap();
        assert!(PanelProtection::load(root.path()).is_err());
    }

    #[test]
    fn terminal_failure_forgets_attempt_but_never_releases_backup() {
        let (root, _guard, receipt) = fixture();
        receipt.discard(root.path()).unwrap();
        assert!(PanelProtection::load(root.path()).unwrap().is_none());
        assert_active(root.path(), &receipt);
    }

    #[cfg(unix)]
    #[test]
    fn linked_receipt_or_package_script_is_rejected_without_touching_link_target() {
        for (linked_receipt, hard) in [(true, false), (true, true), (false, false), (false, true)] {
            let (root, guard, mut receipt) = fixture();
            activate(root.path(), &receipt, &receipt.attempt_id);
            let path = if linked_receipt {
                state_path(root.path())
            } else {
                receipt
                    .prefix
                    .join(install::PACKAGE_SUBPATH)
                    .join("bin/ocx.js")
            };
            let target = root.path().join("untouched");
            let original = std::fs::read(&path).unwrap();
            std::fs::rename(&path, &target).unwrap();
            if hard {
                std::fs::hard_link(&target, &path).unwrap();
            } else {
                std::os::unix::fs::symlink(&target, &path).unwrap();
            }
            assert!(receipt.reconcile(root.path(), &guard).is_err());
            assert_eq!(std::fs::read(target).unwrap(), original);
            assert_active(root.path(), &receipt);
        }
    }
}
