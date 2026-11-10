//! Resolve v2 roots before creating stateful services; no legacy discovery.
use super::*;

pub(crate) struct StartupResolution {
    pub anchor: PathBuf,
    pub sandbox_boundary: Option<PathBuf>,
    pub runtime_config: DataRootRuntimeConfig,
    pub locks: Vec<crate::modules::instance::AppInstanceLock>,
}

pub(crate) fn resolve_default_startup(
    platform_root: &Path,
    executable: &Path,
) -> Result<StartupResolution, AppError> {
    let home = crate::infrastructure::platform::home_dir().ok_or(AppError::NotConfigured)?;
    let sandbox = if crate::modules::test_sandbox::enabled() {
        Some(
            crate::modules::test_sandbox::resolve_data_root(platform_root)
                .map_err(|e| failure(&e))?,
        )
    } else {
        None
    };
    let platform = if cfg!(windows) {
        layout::Platform::Windows
    } else if cfg!(target_os = "macos") {
        layout::Platform::Macos
    } else {
        layout::Platform::Unix
    };
    let install = match &sandbox {
        Some(root) => root.join("program"),
        None => layout::Layout::installation_root_for_user(executable, &home, platform)?,
    };
    startup_v2::resolve_with_install(&home, sandbox.as_deref(), Some(&install))
}

/// Old Windows callers must use the current-user v2 resolver, never scan installs.
pub fn default_anchor(
    _platform_root: &Path,
    _executable: &Path,
    _windows: bool,
) -> Result<PathBuf, AppError> {
    Err(failure(
        "legacy startup selection disabled; use current-user v2 location",
    ))
}

/// Resolve one explicit D for internal copy/verification callers. No pointer chase.
pub fn resolve(root: &Path) -> Result<DataRootRuntimeConfig, AppError> {
    resolve_with_boundary(root, None)
}
pub fn resolve_with_boundary(
    root: &Path,
    boundary: Option<&Path>,
) -> Result<DataRootRuntimeConfig, AppError> {
    resolve_locked_with_boundary(root, boundary).map(|(config, _locks)| config)
}
pub fn resolve_locked_with_boundary(
    root: &Path,
    boundary: Option<&Path>,
) -> Result<
    (
        DataRootRuntimeConfig,
        Vec<crate::modules::instance::AppInstanceLock>,
    ),
    AppError,
> {
    let root = startup_v2::checked_path(root)?;
    let boundary = boundary.map(startup_v2::checked_path).transpose()?;
    let check = |path: &Path| -> Result<(), AppError> {
        if boundary
            .as_ref()
            .is_some_and(|b| startup_v2::checked_path(path).map_or(true, |p| !p.starts_with(b)))
        {
            return Err(failure(
                "directory or external HOME escapes the isolated root",
            ));
        }
        startup_v2::check_owner(path)
    };
    check(&root)?;
    check_partitions(&root)?;
    check(&root.join("manager-state"))?;
    check(&root.join("manager-state/app.lock"))?;
    if root.join(METADATA_RELATIVE_PATH).exists() {
        check(&resolve_opencodex_home(&load_runtime_config(&root)?))?;
    }
    let admission = migration::activation::admit_startup(&root)?;
    let locks = vec![crate::modules::instance::AppInstanceLock::acquire(&root)?];
    migration::activation::complete_startup(&root, &locks[0], admission)?;
    initialize(&root)?;
    let config = load_runtime_config(&root)?;
    check(&resolve_opencodex_home(&config))?;
    Ok((config, locks))
}
fn failure(detail: &str) -> AppError {
    AppError::FileSystem {
        operation: "resolve v2 data root".into(),
        detail: detail.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn losing_startup_does_not_repair_partitions_before_owning_root() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("desktop");
        let (_config, locks) = resolve_locked_with_boundary(&root, None).unwrap();
        fs::remove_dir(root.join("cache")).unwrap();
        assert!(matches!(
            resolve_locked_with_boundary(&root, None),
            Err(AppError::InstanceLockConflict)
        ));
        assert!(!root.join("cache").exists());
        drop(locks);
        let (_config, _locks) = resolve_locked_with_boundary(&root, None).unwrap();
        assert!(root.join("cache").is_dir());
    }
    #[test]
    fn sandbox_rejects_external_home_before_creating_or_repairing_it() {
        let t = tempfile::tempdir().unwrap();
        let aggregate = t.path().join("sandbox");
        let root = aggregate.join("desktop");
        initialize(&root).unwrap();
        let home = t.path().join("daily/not-created");
        set_opencodex_home(&root, OpenCodexHomeMode::External, Some(&home)).unwrap();
        fs::remove_dir(root.join("cache")).unwrap();
        assert!(resolve_with_boundary(&root, Some(&aggregate)).is_err());
        assert!(!home.exists());
        assert!(!root.join("cache").exists());
    }
    #[test]
    fn old_windows_anchor_api_never_scans_or_selects_installation_data() {
        let t = tempfile::tempdir().unwrap();
        assert!(default_anchor(
            &t.path().join("legacy"),
            &t.path().join("install/app.exe"),
            true
        )
        .is_err());
        assert_eq!(fs::read_dir(t.path()).unwrap().count(), 0);
    }
}
