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
}
impl InstallationHandoff {
    pub fn signed_candidate(version: String, identity: EventIdentity, now: i64) -> AppResult<Self> {
        let record = Self {
            schema: 1,
            version,
            identity,
            armed_at: now,
        };
        record.validate()?;
        Ok(record)
    }
    fn validate(&self) -> AppResult<()> {
        let id = &self.identity;
        if self.schema != 1
            || !valid_version(&self.version)
            || self.armed_at < 0
            || (id.object, id.action, id.phase)
                != (ObjectKind::Manager, Action::Install, Phase::Execution)
            || !matches!(id.channel, Channel::Stable | Channel::Beta)
        {
            return Err(AppError::NotConfigured);
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
