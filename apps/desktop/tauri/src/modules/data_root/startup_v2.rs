//! Resolve only the current user's v2 location; never consult legacy bindings.
use super::{bootstrap::StartupResolution, DataRootRuntimeConfig};
use crate::{errors::AppError, modules::instance::AppInstanceLock};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Location {
    schema_version: u32,
    active_root: PathBuf,
}
fn failure(detail: impl Into<String>) -> AppError {
    AppError::FileSystem {
        operation: "resolve v2 user directory".into(),
        detail: detail.into(),
    }
}
/// Normalize existing ancestors (including macOS /var and /tmp aliases) before
/// comparing boundaries. Missing descendants are appended without creating them.
pub(crate) fn checked_path(path: &Path) -> Result<PathBuf, AppError> {
    if !path.is_absolute()
        || path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(failure(
            "absolute directory without parent traversal required",
        ));
    }
    let mut ancestor = path;
    let mut missing = Vec::new();
    loop {
        match std::fs::symlink_metadata(ancestor) {
            Ok(_) => break,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                missing.push(
                    ancestor
                        .file_name()
                        .ok_or_else(|| failure("invalid path"))?,
                );
                ancestor = ancestor
                    .parent()
                    .ok_or_else(|| failure("invalid ancestor"))?;
            }
            Err(e) => return Err(failure(e.to_string())),
        }
    }
    let mut result = ancestor
        .canonicalize()
        .map_err(|e| failure(e.to_string()))?;
    if !missing.is_empty() && !result.is_dir() {
        return Err(failure("path ancestor is not a directory"));
    }
    for name in missing.into_iter().rev() {
        result.push(name);
    }
    Ok(result)
}

pub(crate) fn check_owner(path: &Path) -> Result<(), AppError> {
    #[cfg(unix)]
    match std::fs::metadata(path) {
        Ok(metadata) => {
            use std::os::unix::fs::MetadataExt;
            if metadata.uid() != unsafe { libc::geteuid() } {
                return Err(failure("directory belongs to another user"));
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(failure(e.to_string())),
    }
    #[cfg(windows)]
    check_windows_owner(path)?;
    let _ = path;
    Ok(())
}

#[cfg(windows)]
fn check_windows_owner(path: &Path) -> Result<(), AppError> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        Security::{
            EqualSid, GetFileSecurityW, GetSecurityDescriptorOwner, GetTokenInformation, TokenUser,
            OWNER_SECURITY_INFORMATION, TOKEN_QUERY, TOKEN_USER,
        },
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    };
    match std::fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(failure(e.to_string())),
        Ok(_) => {}
    }
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut needed = 0;
    // SAFETY: first call requests only the security descriptor buffer size.
    unsafe {
        GetFileSecurityW(
            wide.as_ptr(),
            OWNER_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            0,
            &mut needed,
        );
    }
    if needed == 0 || needed > 65536 {
        return Err(failure("file owner cannot be verified"));
    }
    let mut descriptor = vec![0usize; (needed as usize).div_ceil(std::mem::size_of::<usize>())];
    // SAFETY: aligned buffer has at least the requested byte capacity.
    if unsafe {
        GetFileSecurityW(
            wide.as_ptr(),
            OWNER_SECURITY_INFORMATION,
            descriptor.as_mut_ptr().cast(),
            needed,
            &mut needed,
        )
    } == 0
    {
        return Err(failure("file owner cannot be read"));
    }
    let mut owner = std::ptr::null_mut();
    let mut defaulted = 0;
    // SAFETY: the descriptor was returned by GetFileSecurityW and stays alive.
    if unsafe {
        GetSecurityDescriptorOwner(descriptor.as_mut_ptr().cast(), &mut owner, &mut defaulted)
    } == 0
        || owner.is_null()
    {
        return Err(failure("file owner SID is unavailable"));
    }
    let mut token = std::ptr::null_mut();
    // SAFETY: query-only current-process token handle is closed below.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(failure("current user SID is unavailable"));
    }
    let result = (|| {
        let mut required = 0;
        // SAFETY: size query writes only required bytes count.
        unsafe {
            GetTokenInformation(token, TokenUser, std::ptr::null_mut(), 0, &mut required);
        }
        if required == 0 || required > 65536 {
            return Err(failure("invalid user SID buffer size"));
        }
        let mut user = vec![0usize; (required as usize).div_ceil(std::mem::size_of::<usize>())];
        // SAFETY: aligned buffer covers the requested TOKEN_USER output.
        if unsafe {
            GetTokenInformation(
                token,
                TokenUser,
                user.as_mut_ptr().cast(),
                required,
                &mut required,
            )
        } == 0
        {
            return Err(failure("current user SID cannot be read"));
        }
        // SAFETY: both SIDs are valid while their owner buffers remain alive.
        let sid = unsafe { (*user.as_ptr().cast::<TOKEN_USER>()).User.Sid };
        if unsafe { EqualSid(owner, sid) } == 0 {
            return Err(failure("directory belongs to another user"));
        }
        Ok(())
    })();
    // SAFETY: this function owns the token handle.
    unsafe {
        CloseHandle(token);
    }
    result
}

pub(crate) fn load_location(path: &Path, default: &Path) -> Result<PathBuf, AppError> {
    use std::io::Read;
    checked_path(path)?;
    if std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink() || !m.is_file()) {
        return Err(failure("location must be a regular fixed-user file"));
    }
    check_owner(path)?;
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(default.into()),
        Err(e) => return Err(failure(e.to_string())),
    };
    let mut bytes = Vec::new();
    file.take(65537)
        .read_to_end(&mut bytes)
        .map_err(|e| failure(e.to_string()))?;
    if bytes.len() > 65536 {
        return Err(failure("location file exceeds limit"));
    }
    let value: Location = serde_json::from_slice(&bytes)
        .map_err(|_| failure("invalid location file; no fallback"))?;
    if value.schema_version != 1 {
        return Err(failure("unsupported location schema"));
    }
    checked_path(&value.active_root)
}
pub(crate) fn set_location(anchor: &Path, active: &Path) -> Result<(), AppError> {
    let active = checked_path(active)?;
    checked_path(anchor)?;
    checked_path(&anchor.join("manager-location.json"))?;
    let payload = serde_json::to_vec_pretty(&Location {
        schema_version: 1,
        active_root: active.clone(),
    })
    .map_err(|_| failure("location serialization"))?;
    let path = anchor.join("manager-location.json");
    crate::infrastructure::atomic_write::atomic_write(&path, &payload, 0o600)?;
    sync_parent(&path)?;
    if !path.is_file() || load_location(&path, &active)? != active {
        return Err(failure("location readback changed after replacement"));
    }
    Ok(())
}

pub(crate) fn sync_parent(path: &Path) -> Result<(), AppError> {
    #[cfg(unix)]
    std::fs::File::open(path.parent().ok_or_else(|| failure("missing parent"))?)
        .and_then(|directory| directory.sync_all())
        .map_err(|e| failure(e.to_string()))?;
    let _ = path;
    Ok(())
}

pub(crate) fn selected_config(anchor: &Path) -> Result<DataRootRuntimeConfig, AppError> {
    if !anchor.join("manager-location.json").is_file() {
        return Err(failure(
            "fixed current-user location file is required; active D is not an anchor",
        ));
    }
    let aggregate = anchor
        .parent()
        .ok_or_else(|| failure("fixed D has no aggregate parent"))?;
    let active = load_location(&anchor.join("manager-location.json"), aggregate)?;
    super::load_runtime_config(&active.join("desktop"))
}

#[cfg(test)]
pub(crate) fn resolve(
    user_home: &Path,
    sandbox: Option<&Path>,
) -> Result<StartupResolution, AppError> {
    resolve_with_install(user_home, sandbox, None)
}

pub(crate) fn resolve_with_install(
    user_home: &Path,
    sandbox: Option<&Path>,
    install: Option<&Path>,
) -> Result<StartupResolution, AppError> {
    let boundary = sandbox.map(checked_path).transpose()?;
    let aggregate = checked_path(
        &boundary
            .as_ref()
            .map(|b| b.join("data"))
            .unwrap_or_else(|| user_home.join(".codex-desktop")),
    )?;
    let anchor = aggregate.join("desktop");
    let check = |path: &Path| -> Result<(), AppError> {
        let path = checked_path(path)?;
        if boundary.as_ref().is_some_and(|b| !path.starts_with(b)) {
            return Err(failure("sandbox location or HOME escapes boundary"));
        }
        if let Some(install) = install {
            super::layout::Layout::with_aggregate_root(
                &path,
                install,
                super::layout::Platform::Unix,
            )?;
        }
        check_owner(&path)
    };
    if let Some(install) = install {
        super::layout::Layout::with_aggregate_root(
            &aggregate,
            install,
            super::layout::Platform::Unix,
        )?;
    }
    check(&aggregate)?;
    check(&anchor)?;
    check(&anchor.join("manager-startup.lock"))?;
    let _admission = super::removal_gate::shared(&anchor)?;
    crate::infrastructure::platform::create_private_dir_all(&anchor, 0o700)
        .map_err(|e| failure(e.to_string()))?;
    let mut locks = vec![AppInstanceLock::acquire_named(
        &anchor,
        "manager-startup.lock",
    )?];
    check(&anchor.join("manager-location.json"))?;
    let active = load_location(&anchor.join("manager-location.json"), &aggregate)?;
    check(&active)?;
    if let Some(install) = install {
        super::layout::Layout::with_aggregate_root(
            &active,
            install,
            super::layout::Platform::Unix,
        )?;
    }
    let desktop = active.join("desktop");
    check(&desktop)?;
    super::check_partitions(&desktop)?;
    super::migration::require_published(&desktop)?;
    check(&desktop.join("manager-state"))?;
    check(&desktop.join("manager-state/app.lock"))?;
    if active != aggregate
        && super::validate_structure(&desktop)? != super::StructureValidation::Valid
    {
        return Err(failure("selected directory is not valid v2 structure"));
    }
    // Check saved HOME before acquiring a target lock or repairing partitions.
    let saved = if desktop.join(super::METADATA_RELATIVE_PATH).exists() {
        Some(super::load_runtime_config(&desktop)?)
    } else {
        None
    };
    if let Some(config) = &saved {
        let home = super::resolve_opencodex_home(config);
        if super::is_nested(&anchor, &home) {
            return Err(failure("HOME overlaps the fixed manager directory"));
        }
        check(&home)?;
    }
    locks.push(AppInstanceLock::acquire(&desktop)?);
    super::initialize(&desktop)?;
    let config = super::load_runtime_config(&desktop)?;
    let home = super::resolve_opencodex_home(&config);
    check(&home)?;
    crate::infrastructure::platform::create_private_dir_all(&home, 0o700)
        .map_err(|e| failure(e.to_string()))?;
    super::set_private_directory(&home)?;
    super::set_private_directory(&aggregate)?;
    super::set_private_directory(&anchor)?;
    if !anchor.join("manager-location.json").exists() {
        set_location(&anchor, &active)?;
    }
    Ok(StartupResolution {
        anchor,
        sandbox_boundary: boundary,
        runtime_config: config,
        locks,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ignores_legacy_and_keeps_same_user_single_instance() {
        let t = tempfile::tempdir().unwrap();
        let user = t.path().canonicalize().unwrap().join("user");
        std::fs::create_dir_all(&user).unwrap();
        std::fs::write(user.join("legacy"), b"keep").unwrap();
        let a = resolve(&user, None).unwrap();
        assert_eq!(
            a.runtime_config.active_data_root,
            user.join(".codex-desktop/desktop")
        );
        assert_eq!(
            super::super::resolve_opencodex_home(&a.runtime_config),
            user.join(".codex-desktop/opencodex/home")
        );
        assert!(matches!(
            resolve(&user, None),
            Err(AppError::InstanceLockConflict)
        ));
        drop(a);
        resolve(&user, None).unwrap();
        assert_eq!(std::fs::read(user.join("legacy")).unwrap(), b"keep");
    }
    #[test]
    fn corrupted_pointer_never_falls_back_to_empty_root() {
        let t = tempfile::tempdir().unwrap();
        let user = t.path().canonicalize().unwrap().join("user");
        let a = resolve(&user, None).unwrap();
        let pointer = a.anchor.join("manager-location.json");
        drop(a);
        std::fs::write(&pointer, b"bad").unwrap();
        assert!(resolve(&user, None).is_err());
        assert_eq!(std::fs::read(pointer).unwrap(), b"bad");
    }

    #[test]
    fn fixed_pointer_is_only_authority_across_repeated_switches_and_return() {
        let t = tempfile::tempdir().unwrap();
        let user = t.path().join("user");
        let initial = resolve(&user, None).unwrap();
        let anchor = initial.anchor.clone();
        let default_u = anchor.parent().unwrap().to_path_buf();
        drop(initial);
        let a = t.path().join("a");
        let b = t.path().join("b");
        for u in [&a, &b] {
            super::super::initialize(&u.join("desktop")).unwrap();
        }
        let a_meta = std::fs::read(a.join("desktop/manager-state/data-root.json")).unwrap();
        for target in [&a, &b, &default_u] {
            let saved = super::super::switch_reference(
                &anchor,
                target,
                super::super::DataRootSwitchMode::ReferenceOnly,
            )
            .unwrap();
            assert_eq!(
                saved.active_data_root,
                checked_path(&target.join("desktop")).unwrap()
            );
            // Loading D does not reread the pointer or redirect runtime services.
            assert_eq!(
                super::super::load_runtime_config(&anchor)
                    .unwrap()
                    .active_data_root,
                anchor
            );
            let restarted = resolve(&user, None).unwrap();
            assert_eq!(restarted.anchor, anchor);
            assert_eq!(restarted.runtime_config, saved);
            let pointer: serde_json::Value = serde_json::from_slice(
                &std::fs::read(anchor.join("manager-location.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(
                pointer,
                serde_json::json!({"schemaVersion":1, "activeRoot":checked_path(target).unwrap()})
            );
            assert!(!a.join("desktop/manager-location.json").exists());
            assert!(!b.join("desktop/manager-location.json").exists());
        }
        assert_eq!(
            std::fs::read(a.join("desktop/manager-state/data-root.json")).unwrap(),
            a_meta
        );
    }

    #[test]
    fn separate_users_cannot_run_same_selected_root_or_repair_loser_partitions() {
        let t = tempfile::tempdir().unwrap();
        let shared = t.path().join("shared");
        super::super::initialize(&shared.join("desktop")).unwrap();
        let user_a = t.path().join("alice");
        let user_b = t.path().join("bob");
        for user in [&user_a, &user_b] {
            let startup = resolve(user, None).unwrap();
            super::super::switch_reference(
                &startup.anchor,
                &shared,
                super::super::DataRootSwitchMode::ReferenceOnly,
            )
            .unwrap();
        }
        let first = resolve(&user_a, None).unwrap();
        std::fs::remove_dir(shared.join("desktop/cache")).unwrap();
        assert!(matches!(
            resolve(&user_b, None),
            Err(AppError::InstanceLockConflict)
        ));
        assert!(!shared.join("desktop/cache").exists());
        drop(first);
        let second = resolve(&user_b, None).unwrap();
        assert!(shared.join("desktop/cache").is_dir());
        assert_eq!(second.locks.len(), 2);
    }

    #[test]
    fn invalid_pointer_and_selected_metadata_never_fall_back_or_repair() {
        let t = tempfile::tempdir().unwrap();
        let user = t.path().join("user");
        let startup = resolve(&user, None).unwrap();
        let pointer = startup.anchor.join("manager-location.json");
        drop(startup);
        for payload in [
            serde_json::json!({"schemaVersion":2,"activeRoot":t.path()}),
            serde_json::json!({"schemaVersion":1,"activeRoot":"relative"}),
            serde_json::json!({"schemaVersion":1,"activeRoot":t.path(),"other":true}),
            serde_json::json!({"schemaVersion":1,"activeRoot":t.path().join("missing")}),
        ] {
            let bytes = serde_json::to_vec(&payload).unwrap();
            std::fs::write(&pointer, &bytes).unwrap();
            assert!(resolve(&user, None).is_err());
            assert_eq!(std::fs::read(&pointer).unwrap(), bytes);
        }
        assert!(!t.path().join("missing").exists());
    }

    #[test]
    fn sandbox_pointer_and_external_home_escape_never_touch_daily_state() {
        let t = tempfile::tempdir().unwrap();
        let daily_user = t.path().join("daily-user");
        let sandbox = t.path().join("sandbox");
        let startup = resolve(&daily_user, Some(&sandbox)).unwrap();
        let anchor = startup.anchor.clone();
        assert_eq!(anchor, checked_path(&sandbox.join("data/desktop")).unwrap());
        assert!(!daily_user.exists());
        drop(startup);
        let daily = t.path().join("daily-data");
        super::super::initialize(&daily.join("desktop")).unwrap();
        std::fs::remove_dir(daily.join("desktop/cache")).unwrap();
        set_location(&anchor, &daily).unwrap();
        assert!(resolve(&daily_user, Some(&sandbox)).is_err());
        assert!(!daily.join("desktop/cache").exists());
        set_location(&anchor, &sandbox.join("data")).unwrap();
        let home = t.path().join("daily-home/not-created");
        super::super::set_opencodex_home(
            &anchor,
            super::super::OpenCodexHomeMode::External,
            Some(&home),
        )
        .unwrap();
        std::fs::remove_dir(anchor.join("cache")).unwrap();
        assert!(resolve(&daily_user, Some(&sandbox)).is_err());
        assert!(!home.exists());
        assert!(!anchor.join("cache").exists());
        assert!(!daily_user.exists());
    }

    #[test]
    fn startup_rejects_nested_install_and_data_before_creation() {
        let t = tempfile::tempdir().unwrap();
        let user = t.path().join("user");
        let install = user.join(".codex-desktop/program");
        assert!(resolve_with_install(&user, None, Some(&install)).is_err());
        assert!(!user.exists());
        let sandbox = t.path().join("sandbox");
        let startup =
            resolve_with_install(&user, Some(&sandbox), Some(&sandbox.join("program"))).unwrap();
        assert_eq!(
            startup.runtime_config.active_data_root,
            checked_path(&sandbox.join("data/desktop")).unwrap()
        );
        assert!(!user.exists());
    }

    #[cfg(unix)]
    #[test]
    fn canonical_aliases_work_but_symlink_sandbox_escape_is_denied_before_write() {
        let t = tempfile::tempdir().unwrap();
        let alias = t.path().join("alias");
        let sandbox = t.path().join("sandbox");
        std::fs::create_dir(&sandbox).unwrap();
        std::os::unix::fs::symlink(&sandbox, &alias).unwrap();
        let startup = resolve(t.path(), Some(&alias)).unwrap();
        let anchor = startup.anchor.clone();
        drop(startup);
        let outside = t.path().join("outside");
        std::fs::create_dir(&outside).unwrap();
        std::os::unix::fs::symlink(&outside, sandbox.join("escape")).unwrap();
        let home = sandbox.join("escape/uncreated");
        super::super::set_opencodex_home(
            &anchor,
            super::super::OpenCodexHomeMode::External,
            Some(&home),
        )
        .unwrap();
        assert!(resolve(t.path(), Some(&alias)).is_err());
        assert!(!outside.join("uncreated").exists());
    }

    #[test]
    fn active_desktop_cannot_be_used_as_a_new_fixed_pointer_anchor() {
        let t = tempfile::tempdir().unwrap();
        let startup = resolve(&t.path().join("user"), None).unwrap();
        let selected = t.path().join("selected");
        let next = t.path().join("next");
        for root in [&selected, &next] {
            super::super::initialize(&root.join("desktop")).unwrap();
        }
        super::super::switch_reference(
            &startup.anchor,
            &selected,
            super::super::DataRootSwitchMode::ReferenceOnly,
        )
        .unwrap();
        assert!(super::super::switch_reference(
            &selected.join("desktop"),
            &next,
            super::super::DataRootSwitchMode::ReferenceOnly
        )
        .is_err());
        assert!(!selected.join("desktop/manager-location.json").exists());
        assert_eq!(
            selected_config(&startup.anchor).unwrap().active_data_root,
            checked_path(&selected.join("desktop")).unwrap()
        );
    }
}
