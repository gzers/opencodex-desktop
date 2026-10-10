//! Persisted manager-preferences cleanup policy. Retention is N10 OR D30:
//! pinned/legacy records are extra and remain governed by manager protection rules.
//! These helpers never acquire the transaction lock. Callers that write, or read
//! for a subsequent mutation, must hold the shared preferences transaction guard.
use super::safety::{self, fail};
use crate::{errors::AppResult, infrastructure::atomic_write::atomic_write};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const POLICY_RELATIVE_PATH: &str = "manager-state/backup-cleanup-policy.json";
pub const SCHEMA_VERSION: u32 = 1;
pub const KEEP_RECENT: usize = super::MAX_PER_ACTION;
pub const KEEP_DAYS: u32 = 30;
pub const MAX_POLICY_BYTES: u64 = 4096;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CleanupMode {
    #[default]
    Manual,
    Automatic,
}

/// Fixed retention contract; every field is required in persisted schema 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct CleanupPolicy {
    pub schema_version: u32,
    pub mode: CleanupMode,
    pub keep_recent: usize,
    pub keep_days: u32,
}
impl Default for CleanupPolicy {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            mode: CleanupMode::Manual,
            keep_recent: KEEP_RECENT,
            keep_days: KEEP_DAYS,
        }
    }
}
impl CleanupPolicy {
    fn validate(&self) -> AppResult<()> {
        if self.schema_version != SCHEMA_VERSION
            || self.keep_recent != KEEP_RECENT
            || self.keep_days != KEEP_DAYS
        {
            return Err(fail("unsupported cleanup policy schema or retention"));
        }
        Ok(())
    }
}

/// Missing policy returns manual without creating files. Invalid existing data
/// fails closed; it never silently opts into cleanup or resets the policy.
pub fn load(root: &Path) -> AppResult<CleanupPolicy> {
    let path = root.join(POLICY_RELATIVE_PATH);
    safety::check_path(root, &path, true)?;
    match std::fs::symlink_metadata(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(CleanupPolicy::default()),
        Err(e) => Err(fail(e.to_string())),
        Ok(_) => {
            let bytes = safety::read_file(&path, MAX_POLICY_BYTES)?;
            let policy: CleanupPolicy =
                serde_json::from_slice(&bytes).map_err(|_| fail("invalid cleanup policy JSON"))?;
            policy.validate()?;
            Ok(policy)
        }
    }
}

/// Explicitly save a validated policy by same-directory atomic replacement,
/// syncing the file and directories. Does not trigger cleanup. Caller holds lock.
pub fn save(root: &Path, policy: &CleanupPolicy) -> AppResult<()> {
    policy.validate()?;
    let path = root.join(POLICY_RELATIVE_PATH);
    safety::check_path(root, &path, true)?;
    let bytes = serde_json::to_vec_pretty(policy).map_err(|e| fail(e.to_string()))?;
    if bytes.len() as u64 > MAX_POLICY_BYTES {
        return Err(fail("cleanup policy exceeds write limit"));
    }
    atomic_write(&path, &bytes, 0o600)?;
    safety::sync_directories(root, path.parent().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_policy_is_manual_without_creating_manager_state() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(load(root.path()).unwrap(), CleanupPolicy::default());
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    }

    #[test]
    fn atomic_roundtrip_both_modes_and_invalid_save_preserves_previous_bytes() {
        let root = tempfile::tempdir().unwrap();
        let automatic = CleanupPolicy {
            mode: CleanupMode::Automatic,
            ..Default::default()
        };
        save(root.path(), &automatic).unwrap();
        assert_eq!(load(root.path()).unwrap(), automatic);
        let path = root.path().join(POLICY_RELATIVE_PATH);
        let before = std::fs::read(&path).unwrap();
        for invalid in [
            CleanupPolicy {
                schema_version: 2,
                ..automatic
            },
            CleanupPolicy {
                keep_recent: 0,
                ..automatic
            },
            CleanupPolicy {
                keep_days: 31,
                ..automatic
            },
        ] {
            assert!(save(root.path(), &invalid).is_err());
            assert_eq!(std::fs::read(&path).unwrap(), before);
        }
        save(root.path(), &CleanupPolicy::default()).unwrap();
        assert_eq!(load(root.path()).unwrap(), CleanupPolicy::default());
        assert_eq!(
            std::fs::read_dir(path.parent().unwrap()).unwrap().count(),
            1
        );
        #[cfg(unix)]
        assert_eq!(
            crate::infrastructure::platform::mode_of(&path).unwrap(),
            0o600
        );
    }

    #[test]
    fn malformed_schema_types_retention_and_oversize_fail_closed() {
        let root = tempfile::tempdir().unwrap();
        save(root.path(), &CleanupPolicy::default()).unwrap();
        let path = root.path().join(POLICY_RELATIVE_PATH);
        let valid = serde_json::to_value(CleanupPolicy::default()).unwrap();
        for (key, value) in [
            ("schema_version", serde_json::json!(2)),
            ("schema_version", serde_json::json!("1")),
            ("schema_version", serde_json::json!(null)),
            ("mode", serde_json::json!("after-successful-backup")),
            ("mode", serde_json::json!(true)),
            ("keep_recent", serde_json::json!(11)),
            ("keep_recent", serde_json::json!(-1)),
            ("keep_recent", serde_json::json!(10.0)),
            ("keep_days", serde_json::json!(29)),
            ("unknown", serde_json::json!(true)),
        ] {
            let mut invalid = valid.clone();
            invalid[key] = value;
            std::fs::write(&path, serde_json::to_vec(&invalid).unwrap()).unwrap();
            assert!(load(root.path()).is_err(), "{invalid}");
        }
        for key in ["schema_version", "mode", "keep_recent", "keep_days"] {
            let mut invalid = valid.clone();
            invalid.as_object_mut().unwrap().remove(key);
            std::fs::write(&path, serde_json::to_vec(&invalid).unwrap()).unwrap();
            assert!(load(root.path()).is_err());
        }
        for bytes in [b"{".as_slice(), b"[]".as_slice(), b"null".as_slice()] {
            std::fs::write(&path, bytes).unwrap();
            assert!(load(root.path()).is_err());
        }
        std::fs::File::create(&path)
            .unwrap()
            .set_len(MAX_POLICY_BYTES + 1)
            .unwrap();
        assert!(load(root.path()).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_file_directory_and_nonregular_paths_are_rejected() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let outside_file = outside.path().join("policy.json");
        std::fs::write(&outside_file, b"outside").unwrap();
        let parent = root.path().join("manager-state");
        std::fs::create_dir(&parent).unwrap();
        let path = root.path().join(POLICY_RELATIVE_PATH);
        symlink(&outside_file, &path).unwrap();
        assert!(load(root.path()).is_err());
        assert!(save(root.path(), &CleanupPolicy::default()).is_err());
        assert_eq!(std::fs::read(&outside_file).unwrap(), b"outside");
        std::fs::remove_file(&path).unwrap();
        symlink(outside.path().join("missing"), &path).unwrap();
        assert!(load(root.path()).is_err());
        assert!(save(root.path(), &CleanupPolicy::default()).is_err());
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        assert!(load(root.path()).is_err());
        assert!(save(root.path(), &CleanupPolicy::default()).is_err());
        std::fs::remove_dir_all(&parent).unwrap();
        symlink(outside.path(), &parent).unwrap();
        assert!(load(root.path()).is_err());
        assert!(save(root.path(), &CleanupPolicy::default()).is_err());
        assert!(!outside.path().join("backup-cleanup-policy.json").exists());
        std::fs::remove_file(&parent).unwrap();
        std::fs::write(&parent, b"not a directory").unwrap();
        assert!(load(root.path()).is_err());
        assert!(save(root.path(), &CleanupPolicy::default()).is_err());
        assert!(load(Path::new("relative-root")).is_err());
    }
}
