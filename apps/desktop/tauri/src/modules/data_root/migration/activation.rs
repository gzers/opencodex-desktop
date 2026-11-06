//! No legacy binding activation or receipt recovery in v2.
use super::{unsupported, VerifiedCopy};
use crate::{errors::AppResult, modules::instance::AppInstanceLock};
use std::path::Path;

// Kept as an opaque, unconstructible refusal adapter for internal callers.
pub struct PendingActivation(());
impl VerifiedCopy {
    pub fn prepare_activation(self, _anchor: &Path) -> AppResult<PendingActivation> {
        Err(unsupported())
    }
}
impl PendingActivation {
    pub fn commit(self) -> AppResult<()> {
        Err(unsupported())
    }
}

pub(in crate::modules::data_root) struct StartupPermit;
pub(in crate::modules::data_root) fn admit_startup(root: &Path) -> AppResult<StartupPermit> {
    super::require_published(root)?;
    Ok(StartupPermit)
}
pub(in crate::modules::data_root) fn complete_startup(
    root: &Path,
    _lock: &AppInstanceLock,
    _permit: StartupPermit,
) -> AppResult<()> {
    super::require_published(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::data_root;
    use std::fs;
    #[test]
    fn committed_looking_legacy_marker_is_never_an_activation_license() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("desktop");
        data_root::initialize(&root).unwrap();
        let marker = root.join(super::super::UNPUBLISHED_MARKER);
        let bytes = br#"{"schema_version":1,"phase":"awaiting_binding_commit","anchor":"/old","target":"/other"}"#;
        fs::write(&marker, bytes).unwrap();
        fs::remove_dir(root.join("logs")).unwrap();
        assert!(admit_startup(&root).is_err());
        assert!(data_root::bootstrap::resolve(&root).is_err());
        assert_eq!(fs::read(marker).unwrap(), bytes);
        assert!(!root.join("logs").exists());
        assert!(!root.join("manager-state/app.lock").exists());
    }
    #[test]
    fn existing_v2_root_is_checked_without_creating_receipts_or_pointers() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("desktop");
        data_root::initialize(&root).unwrap();
        admit_startup(&root).unwrap();
        assert!(!root.join(super::super::RECEIPT_PATH).exists());
        assert!(!root.join("manager-location.json").exists());
        assert!(!root.join("manager-state/app.lock").exists());
    }
}
