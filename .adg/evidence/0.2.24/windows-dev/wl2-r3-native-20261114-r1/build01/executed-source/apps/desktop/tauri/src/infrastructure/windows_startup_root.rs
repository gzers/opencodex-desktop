//! Windows v2 user context. Admission belongs to data_root::startup_v2.
//! No registry choice, legacy scan, migration, or recovery writer lives here.
#[cfg(windows)]
#[path = "windows_user_scope.rs"]
pub(crate) mod user_scope;
#[cfg(windows)]
pub(crate) use user_scope::{current_user_profile, protect_current_user_path};

/// No WebView command may run with half-initialized business state.
#[cfg(windows)]
pub(crate) fn defer_main(
    config: &mut tauri::utils::config::Config,
) -> Option<tauri::utils::config::WindowConfig> {
    let main = config
        .app
        .windows
        .iter_mut()
        .find(|window| window.label == "main")?;
    let original = main.clone();
    main.create = false;
    Some(original)
}

#[cfg(test)]
mod tests {
    use crate::modules::data_root::{bootstrap, startup_v2};
    #[test]
    fn legacy_default_selection_is_rejected_without_creating_install_data() {
        let t = tempfile::tempdir().unwrap();
        let old = t.path().join("old-appdata");
        let exe = t.path().join("installation/opencodex-desktop.exe");
        assert!(bootstrap::default_anchor(&old, &exe, true).is_err());
        assert!(!old.exists());
        assert!(!exe.parent().unwrap().join("data").exists());
    }
    #[test]
    fn v2_partitions_preserve_unrelated_legacy_material() {
        let t = tempfile::tempdir().unwrap();
        let user = t.path().canonicalize().unwrap().join("用户 profile");
        let legacy = user.join("AppData/Roaming/com.gzers.opencodex.desktop/retain");
        std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        std::fs::write(&legacy, b"legacy bytes").unwrap();
        let first = startup_v2::resolve(&user, None).unwrap();
        assert_eq!(first.anchor, user.join(".codex-desktop/desktop"));
        assert_eq!(first.runtime_config.active_data_root, first.anchor);
        assert_eq!(
            crate::modules::data_root::resolve_opencodex_home(&first.runtime_config),
            user.join(".codex-desktop/opencodex/home")
        );
        assert_eq!(std::fs::read(legacy).unwrap(), b"legacy bytes");
        assert!(first.anchor.join("manager-location.json").is_file());
        assert!(!first.anchor.join("opencodex-home").exists());
    }
    #[test]
    fn malformed_fixed_location_fails_without_fallback_or_repair() {
        let t = tempfile::tempdir().unwrap();
        let user = t.path().canonicalize().unwrap().join("user");
        let fixed = user.join(".codex-desktop/desktop");
        std::fs::create_dir_all(&fixed).unwrap();
        let pointer = fixed.join("manager-location.json");
        std::fs::write(&pointer, b"malformed").unwrap();
        assert!(startup_v2::resolve(&user, None).is_err());
        assert_eq!(std::fs::read(pointer).unwrap(), b"malformed");
        assert!(!fixed.join("manager-state").exists());
        assert!(!user.join(".codex-desktop/opencodex/home").exists());
    }
    #[test]
    fn same_user_relaunch_cannot_create_second_writer_or_rewrite_pointer() {
        let t = tempfile::tempdir().unwrap();
        let user = t.path().canonicalize().unwrap().join("user");
        let first = startup_v2::resolve(&user, None).unwrap();
        let pointer = first.anchor.join("manager-location.json");
        let before = std::fs::read(&pointer).unwrap();
        assert!(matches!(
            startup_v2::resolve(&user, None),
            Err(crate::errors::AppError::InstanceLockConflict)
        ));
        assert_eq!(std::fs::read(&pointer).unwrap(), before);
        drop(first);
        let reopened = startup_v2::resolve(&user, None).unwrap();
        assert_eq!(
            std::fs::read(reopened.anchor.join("manager-location.json")).unwrap(),
            before
        );
    }
}
