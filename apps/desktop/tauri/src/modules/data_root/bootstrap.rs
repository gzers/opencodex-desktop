//! Resolve the binding before any stateful service is created. No data is moved.
use super::*;
use std::collections::HashSet;

pub const MAX_BINDING_DEPTH: usize = 8;

/// Existing Windows data wins, including malformed data: never silently abandon it.
pub fn default_anchor(
    platform_root: &Path,
    executable: &Path,
    windows: bool,
) -> Result<PathBuf, AppError> {
    if !windows
        || platform_root
            .try_exists()
            .map_err(|e| failure(&e.to_string()))?
    {
        return Ok(platform_root.to_path_buf());
    }
    executable
        .parent()
        .filter(|parent| parent.is_absolute())
        .map(|parent| parent.join("data"))
        .ok_or_else(|| failure("installation directory is unavailable"))
}

pub fn resolve(anchor: &Path) -> Result<DataRootRuntimeConfig, AppError> {
    resolve_with_boundary(anchor, None)
}

pub fn resolve_with_boundary(
    anchor: &Path,
    boundary: Option<&Path>,
) -> Result<DataRootRuntimeConfig, AppError> {
    resolve_locked_with_boundary(anchor, boundary).map(|(config, _locks)| config)
}

/// Retain every binding lock for the process lifetime. Initialization and
/// partition repair happen only after owning that root, including the anchor.
pub fn resolve_locked_with_boundary(
    anchor: &Path,
    boundary: Option<&Path>,
) -> Result<
    (
        DataRootRuntimeConfig,
        Vec<crate::modules::instance::AppInstanceLock>,
    ),
    AppError,
> {
    let mut locks = vec![crate::modules::instance::AppInstanceLock::acquire(anchor)?];
    initialize(anchor)?;
    let boundary = boundary
        .map(Path::canonicalize)
        .transpose()
        .map_err(|e| failure(&e.to_string()))?;
    let mut current = anchor.to_path_buf();
    let mut seen = HashSet::new();
    let mut external_home = None;
    for _ in 0..MAX_BINDING_DEPTH {
        let canonical = current
            .canonicalize()
            .map_err(|e| failure(&e.to_string()))?;
        if !seen.insert(canonical.clone()) {
            return Err(failure("data root binding cycle"));
        }
        let mut config = load_runtime_config(&current)?;
        if external_home.is_none() && config.opencodex_home_mode == OpenCodexHomeMode::External {
            external_home = config.opencodex_home_path.clone();
        }
        if let (Some(root), Some(home)) = (&boundary, &external_home) {
            // HOME can be a not-yet-created directory. Resolve the nearest existing
            // ancestor so symlinks cannot conceal an escape from the sandbox.
            let mut ancestor = home.as_path();
            while !ancestor.try_exists().map_err(|e| failure(&e.to_string()))? {
                ancestor = ancestor
                    .parent()
                    .ok_or_else(|| failure("invalid external HOME"))?;
            }
            if !ancestor
                .canonicalize()
                .map_err(|e| failure(&e.to_string()))?
                .starts_with(root)
                || home
                    .components()
                    .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                return Err(failure("external HOME escapes the isolated root"));
            }
        }
        let target = config
            .active_data_root
            .canonicalize()
            .map_err(|e| failure(&e.to_string()))?;
        if boundary
            .as_ref()
            .is_some_and(|root| !target.starts_with(root))
        {
            return Err(failure("binding escapes the isolated root"));
        }
        if canonical == target {
            // Supplement partitions only after validating all bindings.
            initialize(&target)?;
            config.active_data_root = target;
            if let Some(home) = external_home {
                config.opencodex_home_mode = OpenCodexHomeMode::External;
                config.opencodex_home_path = Some(home);
            }
            return Ok((config, locks));
        }
        if validate_structure(&target)? != StructureValidation::Valid {
            return Err(failure(
                "referenced data root is corrupted or newer than supported",
            ));
        }
        if seen.contains(&target) {
            return Err(failure("data root binding cycle"));
        }
        locks.push(crate::modules::instance::AppInstanceLock::acquire(&target)?);
        current = target;
    }
    Err(failure("data root binding exceeds depth limit"))
}

fn failure(detail: &str) -> AppError {
    AppError::FileSystem {
        operation: "resolve data root binding".into(),
        detail: detail.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn losing_startup_does_not_repair_partitions_before_owning_root() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("data");
        let (_config, locks) = resolve_locked_with_boundary(&root, None).unwrap();
        std::fs::remove_dir(root.join("cache")).unwrap();
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
    fn distinct_anchors_cannot_initialize_or_run_the_same_active_root() {
        let temp = tempfile::tempdir().unwrap();
        let a = temp.path().join("a");
        let b = temp.path().join("b");
        let shared = temp.path().join("shared");
        for root in [&a, &b, &shared] {
            initialize(root).unwrap();
        }
        switch_reference(&a, &shared, DataRootSwitchMode::ReferenceOnly).unwrap();
        switch_reference(&b, &shared, DataRootSwitchMode::ReferenceOnly).unwrap();
        let (_config, locks) = resolve_locked_with_boundary(&a, None).unwrap();
        std::fs::remove_dir(shared.join("cache")).unwrap();
        assert!(matches!(
            resolve_locked_with_boundary(&b, None),
            Err(AppError::InstanceLockConflict)
        ));
        assert!(!shared.join("cache").exists());
        drop(locks);
        let (_config, _locks) = resolve_locked_with_boundary(&b, None).unwrap();
        assert!(shared.join("cache").is_dir());
    }

    #[test]
    fn windows_uses_installation_path_and_keeps_legacy_even_if_invalid() {
        let temp = tempfile::tempdir().unwrap();
        let legacy = temp.path().join("legacy");
        let exe = temp.path().join("chosen-install/opencodex.exe");
        assert_eq!(
            default_anchor(&legacy, &exe, true).unwrap(),
            temp.path().join("chosen-install/data")
        );
        assert_eq!(default_anchor(&legacy, &exe, false).unwrap(), legacy);
        std::fs::create_dir(&legacy).unwrap();
        std::fs::write(legacy.join("keep"), b"old").unwrap();
        assert_eq!(default_anchor(&legacy, &exe, true).unwrap(), legacy);
    }
    #[test]
    fn resolves_before_stores_and_preserves_external_home() {
        let temp = tempfile::tempdir().unwrap();
        let a = temp.path().join("a");
        let b = temp.path().join("b");
        initialize(&a).unwrap();
        initialize(&b).unwrap();
        let home = temp.path().join("external");
        set_opencodex_home(&a, OpenCodexHomeMode::External, Some(&home)).unwrap();
        switch_reference(&a, &b, DataRootSwitchMode::ReferenceOnly).unwrap();
        let config = resolve(&a).unwrap();
        assert_eq!(config.active_data_root, b.canonicalize().unwrap());
        assert_eq!(resolve_opencodex_home(&config), home);
        assert_eq!(
            load_runtime_config(&a).unwrap().opencodex_home_mode,
            OpenCodexHomeMode::External
        );
    }
    #[test]
    fn cycles_and_future_bindings_fail_without_initializing_target() {
        let temp = tempfile::tempdir().unwrap();
        let a = temp.path().join("a");
        let b = temp.path().join("b");
        initialize(&a).unwrap();
        initialize(&b).unwrap();
        switch_reference(&a, &b, DataRootSwitchMode::ReferenceOnly).unwrap();
        switch_reference(&b, &a, DataRootSwitchMode::ReferenceOnly).unwrap();
        assert!(resolve(&a).unwrap_err().to_string().contains("cycle"));
        let before = std::fs::read(b.join(METADATA_RELATIVE_PATH)).unwrap();
        let mut payload: serde_json::Value = serde_json::from_slice(&before).unwrap();
        payload["structure_version"] = serde_json::json!("2");
        let future = serde_json::to_vec(&payload).unwrap();
        std::fs::write(b.join(METADATA_RELATIVE_PATH), &future).unwrap();
        assert!(resolve(&a).is_err());
        assert_eq!(
            std::fs::read(b.join(METADATA_RELATIVE_PATH)).unwrap(),
            future
        );
    }
    #[test]
    fn setting_home_does_not_erase_selected_root() {
        let temp = tempfile::tempdir().unwrap();
        let a = temp.path().join("a");
        let b = temp.path().join("b");
        initialize(&a).unwrap();
        initialize(&b).unwrap();
        switch_reference(&a, &b, DataRootSwitchMode::ReferenceOnly).unwrap();
        set_opencodex_home(
            &a,
            OpenCodexHomeMode::External,
            Some(&temp.path().join("home")),
        )
        .unwrap();
        assert_eq!(load_runtime_config(&a).unwrap().active_data_root, b);
    }
    #[test]
    fn sandbox_rejects_escape_before_supplementing_partitions() {
        let temp = tempfile::tempdir().unwrap();
        let a = temp.path().join("sandbox");
        let b = temp.path().join("daily");
        initialize(&a).unwrap();
        initialize(&b).unwrap();
        std::fs::remove_dir(b.join("cache")).unwrap();
        switch_reference(&a, &b, DataRootSwitchMode::ReferenceOnly).unwrap();
        assert!(resolve_with_boundary(&a, Some(&a)).is_err());
        assert!(!b.join("cache").exists());
    }
    #[test]
    fn sandbox_rejects_external_home_before_creating_it() {
        let temp = tempfile::tempdir().unwrap();
        let a = temp.path().join("sandbox");
        initialize(&a).unwrap();
        let home = temp.path().join("daily/not-created");
        set_opencodex_home(&a, OpenCodexHomeMode::External, Some(&home)).unwrap();
        assert!(resolve_with_boundary(&a, Some(&a))
            .unwrap_err()
            .to_string()
            .contains("external HOME"));
        assert!(!home.exists());
    }
}
