//! Explicit v2 user-data removal. Planning and cancellation never write files.
//! Only the fixed current-user location file can select an additional aggregate.
use super::{startup_v2, OpenCodexHomeMode, StructureValidation, PARTITIONS};
use crate::errors::{AppError, AppResult};
use crate::modules::instance::AppInstanceLock;
use std::path::{Path, PathBuf};

const MAX_ENTRIES: usize = 1_000_000;
fn failure(detail: &str) -> AppError {
    AppError::FileSystem {
        operation: "remove v2 manager data".into(),
        detail: detail.into(),
    }
}
fn io(error: std::io::Error) -> AppError {
    AppError::FileSystem {
        operation: "remove v2 manager data".into(),
        detail: error.to_string(),
    }
}
fn is_link(metadata: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return true;
        }
    }
    metadata.file_type().is_symlink()
}

// Resolve OS aliases in the supplied profile first. Within a managed aggregate,
// reject links rather than deleting a different directory through an alias.
fn directory(path: &Path) -> AppResult<PathBuf> {
    let resolved = startup_v2::checked_path(path)?;
    let metadata = std::fs::symlink_metadata(path).map_err(io)?;
    if !metadata.is_dir() || is_link(&metadata) {
        return Err(failure("linked or non-directory removal target"));
    }
    startup_v2::check_owner(path)?;
    Ok(resolved)
}
fn regular(path: &Path) -> AppResult<Vec<u8>> {
    let metadata = std::fs::symlink_metadata(path).map_err(io)?;
    if !metadata.is_file() || is_link(&metadata) || metadata.len() > 65536 {
        return Err(failure("invalid removal metadata"));
    }
    startup_v2::check_owner(path)?;
    std::fs::read(path).map_err(io)
}
fn inspect_tree(root: &Path) -> AppResult<()> {
    let mut pending = vec![root.to_path_buf()];
    let mut count = 0usize;
    while let Some(path) = pending.pop() {
        startup_v2::check_owner(&path)?;
        for entry in std::fs::read_dir(&path).map_err(io)? {
            let path = entry.map_err(io)?.path();
            count += 1;
            if count > MAX_ENTRIES {
                return Err(failure("removal inventory exceeds limit"));
            }
            let metadata = std::fs::symlink_metadata(&path).map_err(io)?;
            startup_v2::check_owner(&path)?;
            if is_link(&metadata) {
                return Err(failure("linked data retained"));
            }
            if metadata.is_dir() {
                pending.push(path);
            } else if !metadata.is_file() {
                return Err(failure("unsupported file type retained"));
            }
        }
    }
    Ok(())
}
fn owned_root_file(name: &std::ffi::OsStr) -> bool {
    use crate::modules::backup::manager::LOCK_FILE_NAME;
    use crate::modules::ipc::{audit::AUDIT_LOG_FILE_NAME, AUDIT_LOG_ROTATIONS};
    let Some(name) = name.to_str() else {
        return false;
    };
    name == LOCK_FILE_NAME
        || name == AUDIT_LOG_FILE_NAME
        || (1..=AUDIT_LOG_ROTATIONS).any(|index| name == format!("{AUDIT_LOG_FILE_NAME}.{index}"))
}
fn verify_desktop(root: &Path, fixed: bool, control_only: bool) -> AppResult<()> {
    directory(root)?;
    // Removing the journal before its external PATH entry has been revoked
    // would orphan a current-user command. The installer has a separate,
    // explicit terminal cleanup step; raw data deletion never guesses it.
    crate::modules::runtime::terminal_cli::ensure_binding_change_allowed(root)?;
    if !control_only && super::validate_structure(root)? != StructureValidation::Valid {
        return Err(failure("only a current v2 directory may be removed"));
    }
    for entry in std::fs::read_dir(root).map_err(io)? {
        let entry = entry.map_err(io)?;
        let metadata = std::fs::symlink_metadata(entry.path()).map_err(io)?;
        let control = fixed
            && ["manager-location.json", "manager-startup.lock"]
                .iter()
                .any(|name| entry.file_name() == *name);
        let partition = !control_only
            && PARTITIONS
                .iter()
                .any(|(_, name)| entry.file_name() == *name);
        if is_link(&metadata)
            || !(control && metadata.is_file()
                || partition && metadata.is_dir()
                || !control_only && owned_root_file(&entry.file_name()) && metadata.is_file())
        {
            return Err(failure("unrecognized desktop content retained"));
        }
    }
    inspect_tree(root)
}

#[derive(Debug, PartialEq, Eq)]
struct DesktopRemoval {
    path: PathBuf,
    metadata: Option<Vec<u8>>,
}

/// A plan covers the fixed control D and the selected v2 D/H, with no old chain.
/// Dropping it is cancellation. External HOME and unselected aggregates stay intact.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RemovalPlan {
    anchor: PathBuf,
    installation: PathBuf,
    pointer: Option<Vec<u8>>,
    desktops: Vec<DesktopRemoval>,
    homes: Vec<PathBuf>,
}
impl RemovalPlan {
    pub(crate) fn prepare_for_user(user_home: &Path, install: &Path) -> AppResult<Self> {
        let profile = startup_v2::checked_path(user_home)?;
        Self::prepare(&profile.join(".codex-desktop/desktop"), install)
    }

    /// The anchor must be the fixed profile/.codex-desktop/desktop location.
    /// The Windows adapter retains this signature while switching to v2 admission.
    pub(crate) fn prepare(anchor: &Path, install: &Path) -> AppResult<Self> {
        if anchor.file_name().is_none_or(|name| name != "desktop")
            || anchor
                .parent()
                .and_then(Path::file_name)
                .is_none_or(|name| name != ".codex-desktop")
        {
            return Err(failure("fixed current-user v2 anchor required"));
        }
        let installation = directory(install)?;
        // Normalize only the OS profile prefix, retaining managed path components
        // so a linked .codex-desktop or desktop cannot be hidden by normalization.
        let profile = startup_v2::checked_path(
            anchor
                .parent()
                .and_then(Path::parent)
                .ok_or_else(|| failure("missing user profile"))?,
        )?;
        let anchor = profile.join(".codex-desktop/desktop");
        let default = anchor
            .parent()
            .ok_or_else(|| failure("missing aggregate parent"))?
            .to_path_buf();
        super::layout::Layout::with_aggregate_root(
            &default,
            &installation,
            super::layout::Platform::Unix,
        )?;
        let mut plan = Self {
            anchor: anchor.clone(),
            installation,
            pointer: None,
            desktops: vec![],
            homes: vec![],
        };
        // Never initialize data for a never-opened application.
        match std::fs::symlink_metadata(&anchor) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(plan),
            Err(error) => return Err(io(error)),
            Ok(_) => {}
        }
        directory(&default)?;
        directory(&anchor)?;
        let pointer_path = anchor.join("manager-location.json");
        let active = match std::fs::symlink_metadata(&pointer_path) {
            Ok(_) => {
                plan.pointer = Some(regular(&pointer_path)?);
                startup_v2::load_location(&pointer_path, &default)?
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => default.clone(),
            Err(error) => return Err(io(error)),
        };
        if active != default && super::is_nested(&active, &default) {
            return Err(failure("nested aggregate selection retained"));
        }
        let mut aggregates = vec![active];
        if aggregates[0] != default {
            aggregates.push(default);
        }
        let mut external_homes = vec![];
        for aggregate in aggregates {
            super::layout::Layout::with_aggregate_root(
                &aggregate,
                &plan.installation,
                super::layout::Platform::Unix,
            )?;
            let aggregate = directory(&aggregate)?;
            let desktop = aggregate.join("desktop");
            let fixed = desktop == anchor;
            let control_only = fixed && !desktop.join(super::METADATA_RELATIVE_PATH).exists();
            if control_only && plan.pointer.is_none() {
                // An empty/partial unknown root cannot prove ownership for deletion.
                return Err(failure(
                    "missing v2 metadata and fixed pointer; retain directory",
                ));
            }
            verify_desktop(&desktop, fixed, control_only)?;
            let metadata = if control_only {
                None
            } else {
                Some(regular(&desktop.join(super::METADATA_RELATIVE_PATH))?)
            };
            if !control_only {
                let config = super::load_runtime_config(&desktop)?;
                let home = super::resolve_opencodex_home(&config);
                if config.opencodex_home_mode == OpenCodexHomeMode::External {
                    external_homes.push(startup_v2::checked_path(&home)?);
                } else {
                    // Do not traverse a linked intermediate opencodex directory.
                    match std::fs::symlink_metadata(aggregate.join("opencodex")) {
                        Ok(_) => {
                            directory(&aggregate.join("opencodex"))?;
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                        Err(e) => return Err(io(e)),
                    }
                    match std::fs::symlink_metadata(&home) {
                        Ok(_) => {
                            let home = directory(&home)?;
                            inspect_tree(&home)?;
                            plan.homes.push(home);
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                        Err(e) => return Err(io(e)),
                    }
                }
            }
            plan.desktops.push(DesktopRemoval {
                path: desktop,
                metadata,
            });
        }
        let deletions: Vec<_> = plan
            .desktops
            .iter()
            .map(|root| &root.path)
            .chain(plan.homes.iter())
            .collect();
        for (index, left) in deletions.iter().enumerate() {
            if deletions
                .iter()
                .skip(index + 1)
                .any(|right| super::is_nested(left, right))
                || external_homes
                    .iter()
                    .any(|home| super::is_nested(left, home))
            {
                return Err(failure("overlapping data or external HOME retained"));
            }
        }
        Ok(plan)
    }

    pub(crate) fn remove(self, anchor: &Path, install: &Path) -> AppResult<()> {
        // Recheck before even creating lock files, then again under all leases.
        if Self::prepare(anchor, install)? != self {
            return Err(failure("removal selection changed"));
        }
        if self.desktops.is_empty() {
            return Ok(());
        }
        let removal = super::removal_gate::RemovalLeaseSet::try_exclusive(
            self.desktops.iter().map(|root| root.path.clone()),
        )?;
        let startup = AppInstanceLock::acquire_for_removal(&self.anchor, true, &removal)?;
        let locks = self
            .desktops
            .iter()
            .filter(|root| root.metadata.is_some())
            .map(|root| AppInstanceLock::acquire_for_removal(&root.path, false, &removal))
            .collect::<AppResult<Vec<_>>>()?;
        let backups = self
            .desktops
            .iter()
            .filter(|root| root.metadata.is_some())
            .map(|root| crate::modules::backup::manager::try_lock_for_removal(&root.path, &removal))
            .collect::<AppResult<Vec<_>>>()?;
        if Self::prepare(anchor, install)? != self {
            return Err(failure("removal selection changed while locking"));
        }
        // The outer exclusive gate survives these handles and all deletions.
        drop(backups);
        drop(locks);
        drop(startup);
        for home in &self.homes {
            std::fs::remove_dir_all(home).map_err(io)?;
        }
        // The fixed pointer is removed last, after the selected root.
        for root in &self.desktops {
            std::fs::remove_dir_all(&root.path).map_err(io)?;
        }
        // Only empty predefined parent directories are pruned; unrelated files remain.
        for root in &self.desktops {
            if let Some(aggregate) = root.path.parent() {
                let _ = std::fs::remove_dir(aggregate.join("opencodex"));
                let _ = std::fs::remove_dir(aggregate);
            }
        }
        Ok(())
    }
}

pub(crate) fn remove_for_user(
    user_home: &Path,
    install: &Path,
    confirmed: bool,
) -> AppResult<bool> {
    if !confirmed {
        return Ok(false);
    }
    let plan = RemovalPlan::prepare_for_user(user_home, install)?;
    let anchor = plan.anchor.clone();
    plan.remove(&anchor, install)?;
    Ok(true)
}

/// ocxd calls this only after its explicit --uninstall-manager-data --confirm.
/// No legacy HKCU/AppData selection or registry cleanup is performed here.
#[cfg(windows)]
pub fn remove_from_windows_installer() -> AppResult<()> {
    if crate::modules::test_sandbox::enabled() {
        return Err(failure("installer cleanup cannot use sandbox identity"));
    }
    let home = crate::infrastructure::platform::home_dir().ok_or(AppError::NotConfigured)?;
    let executable = std::env::current_exe().map_err(io)?;
    let install =
        super::layout::Layout::installation_root(&executable, super::layout::Platform::Windows)?;
    remove_for_user(&home, &install, true)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    struct Fixture {
        _temp: tempfile::TempDir,
        user: PathBuf,
        anchor: PathBuf,
        home: PathBuf,
        install: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let temp = tempfile::tempdir().unwrap();
            let base = temp.path().canonicalize().unwrap();
            let user = base.join("alice");
            let startup = startup_v2::resolve(&user, None).unwrap();
            let anchor = startup.anchor.clone();
            let home = super::super::resolve_opencodex_home(&startup.runtime_config);
            drop(startup);
            let install = base.join("program");
            fs::create_dir(&install).unwrap();
            fs::write(install.join("keep-program"), b"program").unwrap();
            fs::write(
                anchor.join("manager-state/preferences.json"),
                b"preferences",
            )
            .unwrap();
            fs::write(home.join("config.json"), b"native config").unwrap();
            Self {
                _temp: temp,
                user,
                anchor,
                home,
                install,
            }
        }
        fn plan(&self) -> AppResult<RemovalPlan> {
            RemovalPlan::prepare_for_user(&self.user, &self.install)
        }
        fn assert_retained(&self) {
            assert_eq!(
                fs::read(self.anchor.join("manager-state/preferences.json")).unwrap(),
                b"preferences"
            );
            assert_eq!(
                fs::read(self.home.join("config.json")).unwrap(),
                b"native config"
            );
            assert_eq!(
                fs::read(self.install.join("keep-program")).unwrap(),
                b"program"
            );
        }
    }
    #[test]
    fn confirmed_removal_deletes_real_v2_data_and_home_preserving_program_and_other_user() {
        let f = Fixture::new();
        let bob = f._temp.path().join("bob");
        let other = startup_v2::resolve(&bob, None).unwrap();
        let other_home = super::super::resolve_opencodex_home(&other.runtime_config);
        fs::write(other_home.join("keep"), b"bob").unwrap();
        let other_d = other.runtime_config.active_data_root.clone();
        drop(other);
        assert!(remove_for_user(&f.user, &f.install, true).unwrap());
        assert!(!f.anchor.exists() && !f.home.exists());
        assert!(other_d.join(super::super::METADATA_RELATIVE_PATH).is_file());
        assert_eq!(fs::read(other_home.join("keep")).unwrap(), b"bob");
        assert_eq!(
            fs::read(f.install.join("keep-program")).unwrap(),
            b"program"
        );
    }
    #[test]
    fn prepare_and_cancel_do_not_delete_rewrite_or_create_locks() {
        let f = Fixture::new();
        let pointer = fs::read(f.anchor.join("manager-location.json")).unwrap();
        fs::remove_file(f.anchor.join("manager-state/app.lock")).unwrap();
        drop(f.plan().unwrap());
        assert!(!remove_for_user(&f.user, &f.install, false).unwrap());
        assert!(!f.anchor.join("manager-state/app.lock").exists());
        assert!(!f
            .anchor
            .join(crate::modules::backup::manager::LOCK_FILE_NAME)
            .exists());
        assert_eq!(
            fs::read(f.anchor.join("manager-location.json")).unwrap(),
            pointer
        );
        f.assert_retained();
        // Cancellation does not even inspect an invalid/nonexistent identity.
        assert!(!remove_for_user(Path::new("relative"), Path::new("missing"), false).unwrap());
    }
    #[test]
    fn selected_v2_root_is_removed_without_following_any_local_pointer_or_old_chain() {
        let f = Fixture::new();
        let selected = f._temp.path().canonicalize().unwrap().join("selected");
        let external = f._temp.path().join("external");
        fs::create_dir(&external).unwrap();
        fs::write(external.join("keep"), b"external").unwrap();
        super::super::initialize(&selected.join("desktop")).unwrap();
        super::super::set_opencodex_home(
            &selected.join("desktop"),
            OpenCodexHomeMode::External,
            Some(&external),
        )
        .unwrap();
        super::super::switch_reference(
            &f.anchor,
            &selected,
            super::super::DataRootSwitchMode::ReferenceOnly,
        )
        .unwrap();
        // An unselected valid aggregate is not discovered or removed.
        let unused = f._temp.path().join("unselected/desktop");
        super::super::initialize(&unused).unwrap();
        assert!(remove_for_user(&f.user, &f.install, true).unwrap());
        assert!(!selected.join("desktop").exists() && !f.anchor.exists());
        assert!(unused.join(super::super::METADATA_RELATIVE_PATH).is_file());
        assert_eq!(fs::read(external.join("keep")).unwrap(), b"external");
    }
    #[test]
    fn corrupted_future_or_old_pointer_and_metadata_are_retained_without_new_locks() {
        for payload in [
            b"invalid".as_slice(),
            br#"{"schemaVersion":2,"activeRoot":"/unknown"}"#,
            br#"{"schemaVersion":1,"activeRoot":"relative"}"#,
        ] {
            let f = Fixture::new();
            let pointer = f.anchor.join("manager-location.json");
            fs::write(&pointer, payload).unwrap();
            fs::remove_file(f.anchor.join("manager-state/app.lock")).unwrap();
            assert!(remove_for_user(&f.user, &f.install, true).is_err());
            assert_eq!(fs::read(pointer).unwrap(), payload);
            assert!(!f.anchor.join("manager-state/app.lock").exists());
            f.assert_retained();
        }
        for payload in [b"bad".as_slice(), br#"{"structure_version":"3","opencodex_home_mode":"inside"}"#,
            br#"{"structure_version":"1","active_data_root":"/another-user"}"#,
            br#"{"structure_version":"2","opencodex_home_mode":"inside","active_data_root":"/another-user"}"#] {
            let f = Fixture::new();
            let metadata = f.anchor.join(super::super::METADATA_RELATIVE_PATH);
            fs::write(&metadata, payload).unwrap();
            assert!(f.plan().is_err());
            assert_eq!(fs::read(metadata).unwrap(), payload);
            f.assert_retained();
        }
    }
    #[test]
    fn changed_pointer_or_metadata_invalidates_reviewed_plan_before_deletion() {
        for change_pointer in [false, true] {
            let f = Fixture::new();
            let plan = f.plan().unwrap();
            if change_pointer {
                let selected = f._temp.path().join("selected");
                super::super::initialize(&selected.join("desktop")).unwrap();
                super::super::switch_reference(
                    &f.anchor,
                    &selected,
                    super::super::DataRootSwitchMode::ReferenceOnly,
                )
                .unwrap();
            } else {
                let metadata = f.anchor.join(super::super::METADATA_RELATIVE_PATH);
                let mut bytes = fs::read(&metadata).unwrap();
                bytes.push(b'\n');
                fs::write(&metadata, bytes).unwrap();
            }
            assert!(plan.remove(&f.anchor, &f.install).is_err());
            f.assert_retained();
        }
    }
    #[test]
    fn running_startup_service_and_backup_locks_prevent_any_deletion() {
        for kind in [0, 1, 2] {
            let f = Fixture::new();
            let plan = f.plan().unwrap();
            if kind == 0 {
                let _lock =
                    AppInstanceLock::acquire_named(&f.anchor, "manager-startup.lock").unwrap();
                assert!(matches!(
                    plan.remove(&f.anchor, &f.install),
                    Err(AppError::InstanceLockConflict)
                ));
            } else if kind == 1 {
                let _lock = AppInstanceLock::acquire(&f.anchor).unwrap();
                assert!(matches!(
                    plan.remove(&f.anchor, &f.install),
                    Err(AppError::InstanceLockConflict)
                ));
            } else {
                let _lock =
                    crate::modules::backup::manager::begin_preferences_protection(&f.anchor, true)
                        .unwrap();
                assert!(plan.remove(&f.anchor, &f.install).is_err());
            }
            f.assert_retained();
        }
    }
    #[test]
    fn unknown_content_and_nonfixed_roots_are_not_removal_authority() {
        let f = Fixture::new();
        fs::write(f.anchor.join("user-document"), b"retain").unwrap();
        assert!(f.plan().is_err());
        assert!(RemovalPlan::prepare(&f.install, &f.install).is_err());
        assert!(RemovalPlan::prepare(&f._temp.path().join("other/desktop"), &f.install).is_err());
        f.assert_retained();
    }
    #[test]
    fn missing_external_home_is_never_created_or_removed() {
        let f = Fixture::new();
        let external = f._temp.path().join("missing/home");
        super::super::set_opencodex_home(&f.anchor, OpenCodexHomeMode::External, Some(&external))
            .unwrap();
        remove_for_user(&f.user, &f.install, true).unwrap();
        assert!(!f.anchor.exists() && !external.exists());
        // The dormant internal HOME is retained when metadata selects external.
        assert_eq!(
            fs::read(f.home.join("config.json")).unwrap(),
            b"native config"
        );
    }
    #[test]
    fn never_opened_app_has_no_data_to_initialize_or_remove() {
        let t = tempfile::tempdir().unwrap();
        let install = t.path().join("program");
        fs::create_dir(&install).unwrap();
        let user = t.path().join("never-opened");
        remove_for_user(&user, &install, true).unwrap();
        assert!(!user.exists());
    }
    #[cfg(unix)]
    #[test]
    fn linked_partition_pointer_or_home_cannot_delete_external_data() {
        for kind in [0, 1, 2] {
            let f = Fixture::new();
            let outside = f._temp.path().join("outside");
            fs::create_dir(&outside).unwrap();
            fs::write(outside.join("keep"), b"private").unwrap();
            let link = match kind {
                0 => {
                    fs::remove_dir(f.anchor.join("cache")).unwrap();
                    f.anchor.join("cache")
                }
                1 => {
                    fs::remove_file(f.anchor.join("manager-location.json")).unwrap();
                    f.anchor.join("manager-location.json")
                }
                _ => {
                    fs::remove_dir_all(&f.home).unwrap();
                    f.home.clone()
                }
            };
            std::os::unix::fs::symlink(&outside, &link).unwrap();
            assert!(f.plan().is_err());
            assert_eq!(fs::read(outside.join("keep")).unwrap(), b"private");
            assert!(f.anchor.join("manager-state/preferences.json").is_file());
        }
    }
    #[test]
    fn fixed_control_only_anchor_can_remove_selected_v2_data() {
        let f = Fixture::new();
        let selected = f._temp.path().join("selected");
        super::super::initialize(&selected.join("desktop")).unwrap();
        super::super::switch_reference(
            &f.anchor,
            &selected,
            super::super::DataRootSwitchMode::ReferenceOnly,
        )
        .unwrap();
        // First-launch custom selection may leave only the fixed pointer and lease.
        for (_, name) in PARTITIONS {
            fs::remove_dir_all(f.anchor.join(name)).unwrap();
        }
        remove_for_user(&f.user, &f.install, true).unwrap();
        assert!(!f.anchor.exists() && !selected.join("desktop").exists());
    }

    #[cfg(unix)]
    #[test]
    fn linked_fixed_aggregate_does_not_authorize_deleting_its_target() {
        let f = Fixture::new();
        let aggregate = f.anchor.parent().unwrap();
        let outside = f._temp.path().join("outside-managed-data");
        fs::rename(aggregate, &outside).unwrap();
        std::os::unix::fs::symlink(&outside, aggregate).unwrap();
        assert!(f.plan().is_err());
        assert_eq!(
            fs::read(outside.join("desktop/manager-state/preferences.json")).unwrap(),
            b"preferences"
        );
        assert_eq!(
            fs::read(outside.join("opencodex/home/config.json")).unwrap(),
            b"native config"
        );
    }

    #[test]
    fn terminal_registration_or_pending_refuses_data_deletion_before_external_revoke() {
        for name in ["terminal-cli.json", "terminal-cli.pending.json"] {
            let f = Fixture::new();
            let target = f.anchor.join("manager-state").join(name);
            fs::write(&target, b"retained terminal authority").unwrap();
            assert!(f.plan().is_err());
            assert_eq!(fs::read(&target).unwrap(), b"retained terminal authority");
            f.assert_retained();
        }
    }
}
