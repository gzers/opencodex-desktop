//! Explicit manager-data removal for the Windows uninstaller.
//! Default uninstall and updater runs never enter this path.
use super::{load_runtime_config, OpenCodexHomeMode, StructureValidation, PARTITIONS};
use crate::errors::{AppError, AppResult};
use crate::modules::instance::AppInstanceLock;
use std::path::{Component, Path, PathBuf};

fn failure(detail: &str) -> AppError {
    AppError::FileSystem {
        operation: "remove manager data".into(),
        detail: detail.into(),
    }
}

fn is_link(metadata: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    metadata.file_type().is_symlink()
}

fn checked_directory(path: &Path) -> AppResult<PathBuf> {
    if !path.is_absolute() || path.components().any(|part| part == Component::ParentDir) {
        return Err(failure("an absolute dedicated data directory is required"));
    }
    #[cfg(windows)]
    if !matches!(path.components().next(), Some(Component::Prefix(prefix))
        if matches!(prefix.kind(), std::path::Prefix::Disk(_) | std::path::Prefix::VerbatimDisk(_)))
    {
        return Err(failure("only local data directories may be removed"));
    }
    for ancestor in path.ancestors() {
        let metadata = std::fs::symlink_metadata(ancestor)
            .map_err(|_| failure("data directory ancestry cannot be inspected"))?;
        if !metadata.is_dir() || is_link(&metadata) {
            return Err(failure("data directory aliases are not permitted"));
        }
    }
    path.canonicalize()
        .map_err(|_| failure("data directory cannot be resolved"))
}

fn checked_external_home(path: &Path) -> AppResult<PathBuf> {
    if !path.is_absolute() || path.components().any(|part| part == Component::ParentDir) {
        return Err(failure("external HOME must remain an absolute directory"));
    }
    let mut ancestor = path;
    let mut missing = Vec::new();
    loop {
        match std::fs::symlink_metadata(ancestor) {
            Ok(_) => break,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                missing.push(
                    ancestor
                        .file_name()
                        .ok_or_else(|| failure("external HOME path is invalid"))?
                        .to_os_string(),
                );
                ancestor = ancestor
                    .parent()
                    .ok_or_else(|| failure("external HOME parent is unavailable"))?;
            }
            Err(_) => return Err(failure("external HOME cannot be inspected")),
        }
    }
    let mut resolved = checked_directory(ancestor)?;
    for part in missing.into_iter().rev() {
        resolved.push(part);
    }
    Ok(resolved)
}

fn inspect_tree(root: &Path) -> AppResult<()> {
    let mut pending = vec![root.to_path_buf()];
    let mut count = 0;
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory)
            .map_err(|_| failure("data directory inventory failed"))?
        {
            let path = entry
                .map_err(|_| failure("data directory entry cannot be read"))?
                .path();
            count += 1;
            if count > 100_000 {
                return Err(failure("data inventory exceeds the supported bound"));
            }
            let metadata = std::fs::symlink_metadata(&path)
                .map_err(|_| failure("data directory entry cannot be inspected"))?;
            if is_link(&metadata) {
                return Err(failure("linked data must be resolved before removal"));
            }
            if metadata.is_dir() {
                pending.push(path);
            } else if !metadata.is_file() {
                return Err(failure("unsupported data directory entry"));
            }
        }
    }
    Ok(())
}

/// A plan contains only the verified startup-anchor chain. It never includes
/// arbitrary caller paths, sibling directories, or an external OpenCodex HOME.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RemovalPlan {
    roots: Vec<PathBuf>,
    external_homes: Vec<PathBuf>,
}

impl RemovalPlan {
    pub(crate) fn prepare(anchor: &Path, install: &Path) -> AppResult<Self> {
        let install = checked_directory(install)?;
        let mut current = anchor.to_path_buf();
        let mut roots = Vec::new();
        let mut external_homes = Vec::new();
        for _ in 0..super::bootstrap::MAX_BINDING_DEPTH {
            let root = checked_directory(&current)?;
            if root.parent().is_none() || roots.contains(&root) || install.starts_with(&root) {
                return Err(failure("cyclic or containing data directory"));
            }
            if super::validate_structure(&root)? != StructureValidation::Valid {
                return Err(failure("only a current manager data root may be removed"));
            }
            for entry in std::fs::read_dir(&root)
                .map_err(|_| failure("data root inventory failed"))?
            {
                let entry = entry.map_err(|_| failure("data root entry cannot be read"))?;
                if !PARTITIONS
                    .iter()
                    .any(|(_, name)| entry.file_name() == *name)
                {
                    return Err(failure("unrecognized content in data root; retain all data"));
                }
            }
            inspect_tree(&root)?;
            let config = load_runtime_config(&root)?;
            if config.opencodex_home_mode == OpenCodexHomeMode::External {
                let home = config
                    .opencodex_home_path
                    .as_deref()
                    .ok_or_else(|| failure("external HOME binding is incomplete"))?;
                external_homes.push(checked_external_home(home)?);
            }
            let next = checked_directory(&config.active_data_root)?;
            roots.push(root.clone());
            if next == root {
                for (index, left) in roots.iter().enumerate() {
                    if roots
                        .iter()
                        .skip(index + 1)
                        .any(|right| left.starts_with(right) || right.starts_with(left))
                    {
                        return Err(failure("nested data roots cannot be removed"));
                    }
                }
                if external_homes.iter().any(|home| {
                    roots
                        .iter()
                        .any(|root| home.starts_with(root) || root.starts_with(home))
                }) {
                    return Err(failure("external HOME overlaps manager data"));
                }
                return Ok(Self {
                    roots,
                    external_homes,
                });
            }
            current = next;
        }
        Err(failure("data root binding chain exceeds the supported depth"))
    }

    pub(crate) fn remove(self, anchor: &Path, install: &Path) -> AppResult<()> {
        // Obtain every root lock before deleting any file. An active app or
        // competing operation retains its files; never stop a running process.
        let _locks = self
            .roots
            .iter()
            .map(|root| AppInstanceLock::acquire(root))
            .collect::<AppResult<Vec<_>>>()?;
        if Self::prepare(anchor, install)? != self {
            return Err(failure("data root binding changed before removal"));
        }
        for root in self.roots.iter().rev() {
            std::fs::remove_dir_all(root)
                .map_err(|_| failure("data removal incomplete; uninstall must stop"))?;
        }
        Ok(())
    }
}

/// The installer supplies --confirm only after its data-removal checkbox was
/// selected. This does not open a desktop, connect to IPC, or run official code.
#[cfg(windows)]
pub fn remove_from_windows_installer() -> AppResult<()> {
    crate::infrastructure::windows_startup_root::remove_for_uninstall()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let install = base.join("install with spaces");
        std::fs::create_dir(&install).unwrap();
        let root = install.join("data");
        super::super::initialize(&root).unwrap();
        (temp, install, root)
    }

    #[test]
    fn explicit_removal_deletes_install_data_but_preserves_install_and_sibling() {
        let (_temp, install, root) = fixture();
        let sibling = install.join("unrelated.txt");
        std::fs::write(&sibling, b"keep").unwrap();
        std::fs::write(root.join("manager-state/preferences.json"), b"owned").unwrap();
        RemovalPlan::prepare(&root, &install)
            .unwrap()
            .remove(&root, &install)
            .unwrap();
        assert!(!root.exists());
        assert_eq!(std::fs::read(sibling).unwrap(), b"keep");
        assert!(install.is_dir());
    }

    #[test]
    fn planning_does_not_delete_or_rewrite_data() {
        let (_temp, install, root) = fixture();
        let before = std::fs::read(root.join(super::super::METADATA_RELATIVE_PATH)).unwrap();
        let _plan = RemovalPlan::prepare(&root, &install).unwrap();
        assert_eq!(
            std::fs::read(root.join(super::super::METADATA_RELATIVE_PATH)).unwrap(),
            before
        );
        assert!(!root.join("manager-state/app.lock").exists());
    }

    #[test]
    fn custom_active_root_and_anchor_are_removed_external_home_is_retained() {
        let (temp, install, anchor) = fixture();
        let base = temp.path().canonicalize().unwrap();
        let active = base.join("custom-root");
        let home = base.join("external-home");
        std::fs::create_dir(&home).unwrap();
        std::fs::write(home.join("keep"), b"outside").unwrap();
        super::super::initialize(&active).unwrap();
        super::super::save_runtime_config(
            &anchor,
            &super::super::DataRootRuntimeConfig {
                active_data_root: active.clone(),
                opencodex_home_mode: OpenCodexHomeMode::External,
                opencodex_home_path: Some(home.clone()),
            },
        )
        .unwrap();
        RemovalPlan::prepare(&anchor, &install)
            .unwrap()
            .remove(&anchor, &install)
            .unwrap();
        assert!(!anchor.exists() && !active.exists());
        assert_eq!(std::fs::read(home.join("keep")).unwrap(), b"outside");
    }

    #[test]
    fn busy_root_prevents_any_data_removal() {
        let (_temp, install, root) = fixture();
        let plan = RemovalPlan::prepare(&root, &install).unwrap();
        let _running = AppInstanceLock::acquire(&root).unwrap();
        assert!(matches!(
            plan.remove(&root, &install),
            Err(AppError::InstanceLockConflict)
        ));
        assert!(root.join(super::super::METADATA_RELATIVE_PATH).is_file());
    }

    #[test]
    fn future_corrupt_and_unowned_roots_are_retained() {
        for payload in [br#"{"structure_version":"2"}"#.as_slice(), b"invalid"] {
            let (_temp, install, root) = fixture();
            let metadata = root.join(super::super::METADATA_RELATIVE_PATH);
            std::fs::write(&metadata, payload).unwrap();
            assert!(RemovalPlan::prepare(&root, &install).is_err());
            assert_eq!(std::fs::read(metadata).unwrap(), payload);
        }
        let (_temp, install, root) = fixture();
        let sentinel = root.join("user-document");
        std::fs::write(&sentinel, b"keep").unwrap();
        assert!(RemovalPlan::prepare(&root, &install).is_err());
        assert_eq!(std::fs::read(sentinel).unwrap(), b"keep");
        assert!(RemovalPlan::prepare(&install, &install).is_err());
    }

    #[test]
    fn changed_binding_is_rejected_before_deleting_either_root() {
        let (temp, install, anchor) = fixture();
        let plan = RemovalPlan::prepare(&anchor, &install).unwrap();
        let active = temp.path().canonicalize().unwrap().join("new-choice");
        super::super::initialize(&active).unwrap();
        super::super::switch_reference(
            &anchor,
            &active,
            super::super::DataRootSwitchMode::ReferenceOnly,
        )
        .unwrap();
        assert!(plan.remove(&anchor, &install).is_err());
        assert!(anchor.is_dir() && active.is_dir());
    }

    #[test]
    fn missing_external_home_is_not_created_or_removed() {
        let (temp, install, root) = fixture();
        let home = temp.path().canonicalize().unwrap().join("missing/home");
        super::super::save_runtime_config(
            &root,
            &super::super::DataRootRuntimeConfig {
                active_data_root: root.clone(),
                opencodex_home_mode: OpenCodexHomeMode::External,
                opencodex_home_path: Some(home.clone()),
            },
        )
        .unwrap();
        RemovalPlan::prepare(&root, &install)
            .unwrap()
            .remove(&root, &install)
            .unwrap();
        assert!(!root.exists() && !home.exists());
        assert!(!temp.path().join("missing").exists());
    }

    #[cfg(unix)]
    #[test]
    fn linked_partition_never_removes_its_external_target() {
        let (temp, install, root) = fixture();
        let outside = temp.path().canonicalize().unwrap().join("outside");
        std::fs::create_dir(&outside).unwrap();
        std::fs::write(outside.join("keep"), b"external").unwrap();
        std::fs::remove_dir(root.join("cache")).unwrap();
        std::os::unix::fs::symlink(&outside, root.join("cache")).unwrap();
        assert!(RemovalPlan::prepare(&root, &install).is_err());
        assert_eq!(std::fs::read(outside.join("keep")).unwrap(), b"external");
    }
}
