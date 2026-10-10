//! An armed signed installation is a receipt, never proof of a running new version.
use crate::{
    errors::{AppError, AppResult},
    modules::{
        backup::safety,
        notifications::registry::{
            self, Action, Channel, EventIdentity, ObjectKind, PathId, Phase,
        },
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

fn state_path() -> &'static str {
    registry::registry()
        .expect("validated event registry")
        .path(PathId::UpdateHandoff)
}
const MAX_BYTES: u64 = 4096;
const MAX_AGE_SECONDS: i64 = 7 * 86400;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallationHandoff {
    schema: u32,
    version: String,
    identity: EventIdentity,
    armed_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    protection: Option<crate::modules::backup::manager::PreferencesProtectionRef>,
    #[serde(default, skip_serializing_if = "is_false")]
    protection_verified: bool,
}
fn is_false(value: &bool) -> bool {
    !value
}
impl InstallationHandoff {
    pub fn signed_candidate(version: String, identity: EventIdentity, now: i64) -> AppResult<Self> {
        let record = Self {
            schema: 1,
            version,
            identity,
            armed_at: now,
            protection: None,
            protection_verified: false,
        };
        record.validate()?;
        Ok(record)
    }
    pub fn with_protection(
        mut self,
        protection: Option<crate::modules::backup::manager::PreferencesProtectionRef>,
    ) -> AppResult<Self> {
        self.protection = protection;
        self.protection_verified = false;
        self.validate()?;
        Ok(self)
    }
    fn validate(&self) -> AppResult<()> {
        let id = &self.identity;
        if self.schema != 1
            || !valid_version(&self.version)
            || self.armed_at < 0
            || (id.object, id.action, id.phase)
                != (ObjectKind::Manager, Action::Install, Phase::Execution)
            || !matches!(id.channel, Channel::Stable | Channel::Beta)
            || (self.protection_verified && self.protection.is_none())
        {
            return Err(AppError::NotConfigured);
        }
        if let Some(reference) = &self.protection {
            reference.validate()?;
        }
        Ok(())
    }
    /// Call only after signature validation, at the owned OS installer handoff.
    pub fn arm(&self, root: &Path) -> AppResult<()> {
        self.validate()?;
        if !registry::matches_candidate(
            root,
            &self.identity,
            Sha256::digest(self.version.as_bytes()).into(),
        )? {
            return Err(AppError::NotConfigured);
        }
        let path = root.join(state_path());
        safety::check_path(root, &path, true)?;
        crate::infrastructure::atomic_write::atomic_write(
            &path,
            &serde_json::to_vec(self).map_err(|_| AppError::NotConfigured)?,
            0o600,
        )
    }
    pub fn load(root: &Path) -> AppResult<Option<Self>> {
        let path = root.join(state_path());
        safety::check_path(root, &path, true)?;
        match std::fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(AppError::NotConfigured),
            Ok(_) => {}
        }
        let record: Self = serde_json::from_slice(&safety::read_file(&path, MAX_BYTES)?)
            .map_err(|_| AppError::NotConfigured)?;
        record.validate()?;
        Ok(Some(record))
    }
    pub fn running_identity(
        &self,
        root: &Path,
        version: &str,
        now: i64,
    ) -> AppResult<Option<EventIdentity>> {
        self.validate()?;
        // An old process, a different candidate, or a clock rollback is inconclusive.
        // In particular, do not resolve beta failures from a stable installation.
        if version != self.version
            || now < self.armed_at
            || now.saturating_sub(self.armed_at) > MAX_AGE_SECONDS
        {
            return Ok(None);
        }
        Ok(registry::matches_candidate(
            root,
            &self.identity,
            Sha256::digest(self.version.as_bytes()).into(),
        )?
        .then(|| self.identity.clone()))
    }
    pub fn identity(&self) -> EventIdentity {
        self.identity.clone()
    }
    pub fn version(&self) -> &str {
        &self.version
    }
    /// Only a verified running candidate can release the associated backup.
    /// Failure leaves the durable handoff intact so startup can retry safely.
    pub fn reconcile_protection(
        &mut self,
        root: &Path,
        version: &str,
        now: i64,
    ) -> AppResult<bool> {
        if self.running_identity(root, version, now)?.is_none() {
            return Ok(false);
        }
        if let Some(reference) = self.protection.clone() {
            let mut verified = self.clone();
            verified.protection_verified = true;
            crate::modules::backup::manager::reconcile_preferences_restart_recorded(
                root,
                &reference,
                self.protection_verified,
                || {
                    // A newer attempt may use the same candidate/version but a
                    // different backup. Never overwrite its durable association.
                    if Self::load(root)?.as_ref() != Some(self) {
                        return Err(AppError::NotConfigured);
                    }
                    verified.arm(root)?;
                    if Self::load(root)?.as_ref() != Some(&verified) {
                        return Err(AppError::NotConfigured);
                    }
                    Ok(())
                },
            )?;
            *self = verified;
        }
        Ok(true)
    }
    /// Do not remove another attempt. On delivery failure leave the receipt for retry.
    pub fn discard(&self, root: &Path) -> AppResult<()> {
        if Self::load(root)?.as_ref() == Some(self) {
            std::fs::remove_file(root.join(state_path())).map_err(|_| AppError::NotConfigured)?;
        }
        Ok(())
    }
}
pub fn valid_version(version: &str) -> bool {
    !version.is_empty()
        && version.len() <= 128
        && version.as_bytes()[0].is_ascii_digit()
        && version
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".+-".contains(&b))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn receipt(root: &Path, version: &str, channel: Channel) -> InstallationHandoff {
        let id = registry::candidate_identity(
            root,
            ObjectKind::Manager,
            Action::Install,
            Phase::Execution,
            channel,
            Sha256::digest(version.as_bytes()).into(),
        )
        .unwrap();
        InstallationHandoff::signed_candidate(version.into(), id, 100).unwrap()
    }
    fn protected_receipt(root: &Path, channel: Channel) -> InstallationHandoff {
        crate::modules::data_root::initialize(root).unwrap();
        crate::modules::preferences::PreferencesStore::new(root)
            .save(&crate::modules::preferences::Preferences::default())
            .unwrap();
        let guard =
            crate::modules::backup::manager::begin_preferences_protection(root, true).unwrap();
        let reference = guard.await_restart(root).unwrap();
        drop(guard);
        receipt(root, "0.1.10", channel)
            .with_protection(reference)
            .unwrap()
    }
    #[test]
    fn exact_running_candidate_releases_associated_backup_only_and_preserves_user_pin() {
        let root = tempfile::tempdir().unwrap();
        let mut record = protected_receipt(root.path(), Channel::Beta);
        let reference = record.protection.clone().unwrap();
        crate::modules::backup::manager::set_pinned(root.path(), &reference.backup_id, true)
            .unwrap();
        record.arm(root.path()).unwrap();
        assert!(!record
            .reconcile_protection(root.path(), "0.1.9", 101)
            .unwrap());
        assert!(record
            .reconcile_protection(root.path(), "0.1.10", 101)
            .unwrap());
        assert!(record
            .reconcile_protection(root.path(), "0.1.10", 101)
            .unwrap());
        // Pin survives reconciliation; it becomes removable only afterwards.
        crate::modules::backup::manager::set_pinned(root.path(), &reference.backup_id, false)
            .unwrap();
        assert_eq!(record.identity().channel, Channel::Beta);
        assert_eq!(
            InstallationHandoff::load(root.path()).unwrap(),
            Some(record)
        );
    }
    #[test]
    fn invalid_association_remains_retryable_and_another_candidate_never_releases_it() {
        let root = tempfile::tempdir().unwrap();
        let mut record = protected_receipt(root.path(), Channel::Stable);
        let reference = record.protection.clone().unwrap();
        let mut wrong = record
            .clone()
            .with_protection(Some(
                crate::modules::backup::manager::PreferencesProtectionRef {
                    sha256: "0".repeat(64),
                    ..reference.clone()
                },
            ))
            .unwrap();
        wrong.arm(root.path()).unwrap();
        assert!(wrong
            .reconcile_protection(root.path(), "0.1.10", 101)
            .is_err());
        assert_eq!(InstallationHandoff::load(root.path()).unwrap(), Some(wrong));
        assert!(crate::modules::backup::manager::set_pinned(
            root.path(),
            &reference.backup_id,
            false
        )
        .is_err());
        record.arm(root.path()).unwrap();
        let next = receipt(root.path(), "0.1.11", Channel::Stable);
        next.arm(root.path()).unwrap();
        assert!(!record
            .reconcile_protection(root.path(), "0.1.10", 101)
            .unwrap());
        assert!(crate::modules::backup::manager::set_pinned(
            root.path(),
            &reference.backup_id,
            false
        )
        .is_err());
        assert_eq!(InstallationHandoff::load(root.path()).unwrap(), Some(next));
    }
    #[test]
    fn stale_same_candidate_receipt_cannot_overwrite_a_new_backup_association() {
        let root = tempfile::tempdir().unwrap();
        let mut stale = protected_receipt(root.path(), Channel::Stable);
        stale.arm(root.path()).unwrap();
        let current = protected_receipt(root.path(), Channel::Stable);
        assert_eq!(stale.identity, current.identity);
        assert_ne!(stale.protection, current.protection);
        current.arm(root.path()).unwrap();
        assert!(stale
            .reconcile_protection(root.path(), "0.1.10", 101)
            .is_err());
        assert_eq!(
            InstallationHandoff::load(root.path()).unwrap(),
            Some(current)
        );
        assert!(crate::modules::backup::manager::set_pinned(
            root.path(),
            &stale.protection.as_ref().unwrap().backup_id,
            false
        )
        .is_err());
    }
    #[test]
    fn persisted_verification_survives_rotation_before_notification_delivery() {
        let root = tempfile::tempdir().unwrap();
        let mut record = protected_receipt(root.path(), Channel::Stable);
        record.arm(root.path()).unwrap();
        let reference = record.protection.clone().unwrap();
        let row = crate::modules::backup::list_action_records(
            &root.path().join("backups"),
            "preferences-protection",
        )
        .unwrap()
        .into_iter()
        .find(|r| r.manifest.backup_id == reference.backup_id)
        .unwrap();
        assert!(record
            .reconcile_protection(root.path(), "0.1.10", 101)
            .unwrap());
        let mut saved = InstallationHandoff::load(root.path()).unwrap().unwrap();
        assert!(saved.protection_verified);
        // Emulate expiry after release and before a failed notification retries.
        std::fs::remove_dir_all(row.directory).unwrap();
        assert!(saved
            .reconcile_protection(root.path(), "0.1.10", 102)
            .unwrap());
        assert!(!saved
            .reconcile_protection(root.path(), "0.1.9", 102)
            .unwrap());
        // An unverified handoff cannot infer success from a missing backup.
        saved.protection_verified = false;
        saved.arm(root.path()).unwrap();
        assert!(saved
            .reconcile_protection(root.path(), "0.1.10", 102)
            .is_err());
        assert!(
            !InstallationHandoff::load(root.path())
                .unwrap()
                .unwrap()
                .protection_verified
        );
    }
    #[test]
    fn new_running_version_required_and_exact_original_channel_preserved() {
        let root = tempfile::tempdir().unwrap();
        let record = receipt(root.path(), "0.1.10-beta.1", Channel::Beta);
        assert!(InstallationHandoff::load(root.path()).unwrap().is_none());
        record.arm(root.path()).unwrap();
        let saved = InstallationHandoff::load(root.path()).unwrap().unwrap();
        for version in ["0.1.9", "0.1.10", "0.1.11"] {
            assert!(saved
                .running_identity(root.path(), version, 101)
                .unwrap()
                .is_none());
        }
        assert_eq!(
            saved
                .running_identity(root.path(), record.version(), 101)
                .unwrap(),
            Some(record.identity())
        );
        assert!(saved
            .running_identity(root.path(), record.version(), 99)
            .unwrap()
            .is_none());
        assert!(saved
            .running_identity(root.path(), record.version(), 101 + MAX_AGE_SECONDS)
            .unwrap()
            .is_none());
        saved.discard(root.path()).unwrap();
        assert!(InstallationHandoff::load(root.path()).unwrap().is_none());
    }
    #[test]
    fn different_root_or_newer_candidate_never_resolved_or_overwritten() {
        let root = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        let old = receipt(root.path(), "0.1.10", Channel::Stable);
        old.arm(root.path()).unwrap();
        assert!(old
            .running_identity(other.path(), "0.1.10", 101)
            .unwrap()
            .is_none());
        assert!(!other.path().join("manager-state").exists());
        let next = receipt(root.path(), "0.1.11", Channel::Stable);
        next.arm(root.path()).unwrap();
        let before = std::fs::read(root.path().join("manager-state/event-scope.json")).unwrap();
        assert!(old
            .running_identity(root.path(), "0.1.10", 101)
            .unwrap()
            .is_none());
        old.discard(root.path()).unwrap();
        assert_eq!(InstallationHandoff::load(root.path()).unwrap(), Some(next));
        assert_eq!(
            before,
            std::fs::read(root.path().join("manager-state/event-scope.json")).unwrap()
        );
    }
    #[test]
    fn invalid_oversized_or_linked_receipt_rejected_without_following_it() {
        let root = tempfile::tempdir().unwrap();
        let record = receipt(root.path(), "0.1.10", Channel::Stable);
        record.arm(root.path()).unwrap();
        std::fs::write(
            root.path().join(state_path()),
            vec![b' '; MAX_BYTES as usize + 1],
        )
        .unwrap();
        assert!(InstallationHandoff::load(root.path()).is_err());
        #[cfg(unix)]
        {
            let outside = tempfile::tempdir().unwrap();
            let target = outside.path().join("receipt.json");
            std::fs::write(&target, b"preserve").unwrap();
            std::fs::remove_file(root.path().join(state_path())).unwrap();
            std::os::unix::fs::symlink(&target, root.path().join(state_path())).unwrap();
            assert!(record.arm(root.path()).is_err());
            assert!(InstallationHandoff::load(root.path()).is_err());
            assert_eq!(std::fs::read(target).unwrap(), b"preserve");
        }
    }
}
