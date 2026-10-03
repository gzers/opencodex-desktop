use opencodex_desktop_lib::commands::preferences::{
    load_preferences_with_path, restore_preferences_with_path, save_preferences_with_path,
};
use opencodex_desktop_lib::modules::data_root::initialize;
use opencodex_desktop_lib::modules::preferences::{Preferences, PreferencesError};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[test]
fn preferences_load_returns_frozen_defaults_for_valid_data_root() {
    let temp = tempfile::tempdir().expect("create temporary data root");
    initialize(temp.path()).expect("initialize fixture");

    let loaded = load_preferences_with_path(temp.path()).expect("load default preferences");
    assert_eq!(loaded, Preferences::default());
}

#[test]
fn preferences_save_is_atomic_private_and_reloadable() {
    let temp = tempfile::tempdir().expect("create temporary data root");
    initialize(temp.path()).expect("initialize fixture");

    let value = Preferences {
        interface_scale: 175,
        launch_main: false,
        panel_mode: "browser".to_string(),
        backup_retention: "20".to_string(),
        cli_enabled: true,
        ..Default::default()
    };

    let saved = save_preferences_with_path(temp.path(), &value).expect("save preferences");
    assert_eq!(saved, value);

    let path = temp.path().join("manager-state/preferences.json");
    let metadata = std::fs::metadata(&path).expect("preferences metadata");
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
    assert_eq!(
        load_preferences_with_path(temp.path()).expect("reload"),
        value
    );
}

#[test]
fn preferences_save_rejects_invalid_contract_before_write() {
    let temp = tempfile::tempdir().expect("create temporary data root");
    initialize(temp.path()).expect("initialize fixture");

    let value = Preferences {
        interface_scale: 250,
        ..Default::default()
    };
    assert!(save_preferences_with_path(temp.path(), &value).is_err());

    let value = Preferences {
        sync_conflict_policy: "overwrite".to_string(),
        ..Default::default()
    };
    assert!(save_preferences_with_path(temp.path(), &value).is_err());

    let path = temp.path().join("manager-state/preferences.json");
    assert!(!path.exists(), "invalid preferences must not be written");
}

#[test]
fn preferences_corrupted_file_fails_closed_and_preserves_target() {
    let temp = tempfile::tempdir().expect("create temporary data root");
    initialize(temp.path()).expect("initialize fixture");
    let path = temp.path().join("manager-state/preferences.json");
    std::fs::write(&path, b"{broken").expect("write corrupted fixture");

    let error = load_preferences_with_path(temp.path()).expect_err("corrupted should fail");
    assert!(matches!(
        error,
        opencodex_desktop_lib::errors::AppError::FileSystem { .. }
    ));

    let observed = std::fs::read(&path).expect("preserve corrupted target");
    assert_eq!(observed, b"{broken");
}

#[test]
fn preferences_restore_writes_default_contract() {
    let temp = tempfile::tempdir().expect("create temporary data root");
    initialize(temp.path()).expect("initialize fixture");
    let value = Preferences {
        interface_scale: 200,
        cli_enabled: true,
        ..Default::default()
    };
    save_preferences_with_path(temp.path(), &value).expect("save changed preferences");

    let restored = restore_preferences_with_path(temp.path()).expect("restore defaults");
    assert_eq!(restored, Preferences::default());
    assert_eq!(
        load_preferences_with_path(temp.path()).expect("reload restored"),
        Preferences::default()
    );
}

#[test]
fn preferences_unconfigured_path_is_not_configured() {
    let error = load_preferences_with_path(Path::new("")).expect_err("unconfigured");
    assert!(matches!(
        error,
        opencodex_desktop_lib::errors::AppError::NotConfigured
    ));

    let error = save_preferences_with_path(Path::new(""), &Preferences::default())
        .expect_err("unconfigured save");
    assert!(matches!(
        error,
        opencodex_desktop_lib::errors::AppError::NotConfigured
    ));
}

#[test]
fn preferences_error_maps_to_app_error_contract() {
    let error: opencodex_desktop_lib::errors::AppError = PreferencesError::Corrupted.into();
    assert!(error.to_string().contains("corrupted"));
}
