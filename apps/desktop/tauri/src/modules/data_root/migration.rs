//! Legacy root copying is unsupported by the v2 directory contract.
//! Retained signatures let existing internal callers receive an explicit refusal.
pub mod activation;
use super::DataRootRuntimeConfig;
use crate::{
    errors::{AppError, AppResult},
    modules::instance::AppInstanceLock,
};
use serde::Serialize;
use std::path::{Path, PathBuf};

pub const RECEIPT_PATH: &str = "manager-state/data-root-migration.json";
pub const UNPUBLISHED_MARKER: &str = ".data-root-migration-incomplete";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Entry {
    Directory {
        path: PathBuf,
    },
    File {
        path: PathBuf,
        bytes: u64,
        sha256: String,
    },
    Link {
        path: PathBuf,
        target: PathBuf,
        directory: bool,
    },
}

#[derive(Debug, Serialize)]
pub struct CopyReceipt {
    pub schema_version: u32,
    /// "verified_copy" is not a committed binding or successful restart.
    pub phase: &'static str,
    pub source: PathBuf,
    pub target: PathBuf,
    pub excluded: Vec<&'static str>,
    pub bytes: u64,
    pub entries: Vec<Entry>,
    /// Inventory after rewriting only the target self-binding, excluding this
    /// receipt and the unpublished marker. Runtime relocation is still pending.
    pub prepared_entries: Vec<Entry>,
}

pub struct VerifiedCopy {
    pub receipt: CopyReceipt,
    /// Prevent another instance from repairing or starting the unpublished copy.
    pub target_lock: AppInstanceLock,
}

impl VerifiedCopy {
    pub fn prepare_runtime(&mut self) -> AppResult<()> {
        Err(unsupported())
    }
}

pub(crate) fn unsupported() -> AppError {
    AppError::ConfigMigration { detail: "unsupported: v2 supports explicit root selection only; legacy migration and root copying are disabled".into() }
}

/// Refuse before inspecting either source or target, including linked paths.
pub fn copy_verified(_config: &DataRootRuntimeConfig, _target: &Path) -> AppResult<VerifiedCopy> {
    Err(unsupported())
}

/// Unpublished legacy copies remain quarantined; v2 never resumes or repairs them.
pub fn require_published(root: &Path) -> AppResult<()> {
    match std::fs::symlink_metadata(root.join(UNPUBLISHED_MARKER)) {
        Ok(_) => Err(unsupported()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(AppError::FileSystem {
            operation: "inspect directory publication".into(),
            detail: e.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn root_copy_is_unsupported_before_any_source_or_target_io() {
        let t = tempfile::tempdir().unwrap();
        let source = t.path().join("missing-source/desktop");
        let target = t.path().join("missing-target");
        let config = DataRootRuntimeConfig {
            active_data_root: source.clone(),
            opencodex_home_mode: super::super::OpenCodexHomeMode::Inside,
            opencodex_home_path: None,
        };
        let error = copy_verified(&config, &target).err().unwrap();
        assert!(error.to_string().contains("unsupported"));
        assert!(!source.exists() && !target.exists());
        assert_eq!(fs::read_dir(t.path()).unwrap().count(), 0);
    }

    #[test]
    fn refusal_preserves_existing_v2_metadata_pointer_and_home() {
        let t = tempfile::tempdir().unwrap();
        let startup = super::super::startup_v2::resolve(&t.path().join("user"), None).unwrap();
        let source = &startup.runtime_config.active_data_root;
        let home = super::super::resolve_opencodex_home(&startup.runtime_config);
        fs::write(home.join("config.json"), b"private native config").unwrap();
        let metadata = fs::read(super::super::runtime_config_path(source)).unwrap();
        let pointer = fs::read(startup.anchor.join("manager-location.json")).unwrap();
        let target = t.path().join("empty-target");
        fs::create_dir(&target).unwrap();
        assert!(copy_verified(&startup.runtime_config, &target).is_err());
        assert_eq!(
            fs::read(super::super::runtime_config_path(source)).unwrap(),
            metadata
        );
        assert_eq!(
            fs::read(startup.anchor.join("manager-location.json")).unwrap(),
            pointer
        );
        assert_eq!(
            fs::read(home.join("config.json")).unwrap(),
            b"private native config"
        );
        assert_eq!(fs::read_dir(&target).unwrap().count(), 0);
    }

    #[test]
    fn marker_cannot_trigger_resume_or_partition_repair() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("desktop");
        super::super::initialize(&root).unwrap();
        fs::remove_dir(root.join("cache")).unwrap();
        fs::write(root.join(UNPUBLISHED_MARKER), b"legacy receipt").unwrap();
        let metadata = fs::read(super::super::runtime_config_path(&root)).unwrap();
        assert!(super::super::initialize(&root).is_err());
        assert!(super::super::ensure_partitions(&root).is_err());
        assert!(super::super::bootstrap::resolve(&root).is_err());
        assert!(!root.join("cache").exists());
        assert_eq!(
            fs::read(super::super::runtime_config_path(&root)).unwrap(),
            metadata
        );
        assert_eq!(
            fs::read(root.join(UNPUBLISHED_MARKER)).unwrap(),
            b"legacy receipt"
        );
    }

    #[cfg(unix)]
    #[test]
    fn linked_source_and_target_are_neither_followed_nor_replaced() {
        let t = tempfile::tempdir().unwrap();
        let outside = t.path().join("outside");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("keep"), b"keep").unwrap();
        let source = t.path().join("source");
        let target = t.path().join("target");
        std::os::unix::fs::symlink(&outside, &source).unwrap();
        std::os::unix::fs::symlink(&outside, &target).unwrap();
        let config = DataRootRuntimeConfig {
            active_data_root: source,
            opencodex_home_mode: super::super::OpenCodexHomeMode::Inside,
            opencodex_home_path: None,
        };
        assert!(copy_verified(&config, &target).is_err());
        assert!(fs::symlink_metadata(&target)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(fs::read(outside.join("keep")).unwrap(), b"keep");
        assert_eq!(fs::read_dir(&outside).unwrap().count(), 1);
    }
}
