//! Registered legacy migration commands are explicit unsupported adapters.
//! GUI and IPC share these functions; neither probes legacy data or credentials.
use crate::modules::migration;
use crate::state::{SharedDataRoot, SharedHomeDir};
use crate::types::migration::{
    MigrationExportResult, MigrationImportRequest, MigrationImportResult,
};

fn unsupported() -> crate::errors::AppError {
    crate::errors::AppError::ConfigMigration {
        detail:
            "unsupported: legacy migration containers are disabled by the v2 user-root contract"
                .into(),
    }
}

#[tauri::command]
pub async fn export_migration(
    app: tauri::AppHandle,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> crate::errors::AppResult<MigrationExportResult> {
    let _ = (app, data_root);
    Err(unsupported())
}
#[tauri::command]
pub async fn import_migration(
    app: tauri::AppHandle,
    request: MigrationImportRequest,
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, SharedHomeDir>,
) -> crate::errors::AppResult<MigrationImportResult> {
    let _ = (app, request, data_root, home);
    Err(unsupported())
}

#[cfg(test)]
pub(crate) fn export_container_registered(
    _root: &std::path::Path,
    _target: &std::path::Path,
    _version: &str,
    _app: Option<&tauri::AppHandle>,
) -> crate::errors::AppResult<migration::ExportResult> {
    Err(unsupported())
}
#[cfg(test)]
pub(crate) fn import_container_registered(
    _root: &std::path::Path,
    _source: &std::path::Path,
    _home: &std::path::Path,
    _passphrase: Option<&str>,
    _app: Option<&tauri::AppHandle>,
) -> crate::errors::AppResult<migration::ImportResult> {
    Err(unsupported())
}
pub fn export_migration_with_root(
    _data_root: &std::path::Path,
    _app_version: String,
) -> crate::errors::AppResult<migration::ExportResult> {
    Err(unsupported())
}
pub fn import_migration_with_root(
    _data_root: &std::path::Path,
    _home: &std::path::Path,
    _passphrase: Option<&str>,
) -> crate::errors::AppResult<migration::ImportResult> {
    Err(unsupported())
}

/// Lossless path bytes are transient hash input, never notification fields.
#[cfg(all(test, unix))]
pub(crate) fn export_candidate(document_sha256: &str, target: &std::path::Path) -> Vec<u8> {
    [
        document_sha256.as_bytes(),
        b"\0",
        target.as_os_str().as_encoded_bytes(),
    ]
    .concat()
}
#[cfg(all(test, unix))]
pub(crate) fn import_candidate(document_sha256: &str, home: &std::path::Path) -> Vec<u8> {
    [
        document_sha256.as_bytes(),
        b"\0",
        home.as_os_str().as_encoded_bytes(),
    ]
    .concat()
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn all_gui_and_ipc_helpers_refuse_without_creating_missing_paths() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("missing-root");
        let home = t.path().join("missing-home");
        let target = t.path().join("missing-export/container.json");
        for error in [
            export_container_registered(&root, &target, "2", None).unwrap_err(),
            import_container_registered(&root, &target, &home, Some("unused secret"), None)
                .unwrap_err(),
            export_migration_with_root(&root, "2".into()).unwrap_err(),
            import_migration_with_root(&root, &home, None).unwrap_err(),
        ] {
            assert!(error.to_string().contains("unsupported"));
        }
        assert_eq!(fs::read_dir(t.path()).unwrap().count(), 0);
    }
    #[test]
    fn existing_legacy_containers_native_config_and_metadata_remain_unchanged() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("desktop");
        let home = t.path().join("home");
        fs::create_dir(&root).unwrap();
        fs::create_dir(&home).unwrap();
        let source = root.join("legacy-export.json");
        fs::write(&source, b"legacy encrypted or invalid payload").unwrap();
        fs::write(home.join("config.json"), b"native config").unwrap();
        assert!(export_container_registered(&root, &source, "2", None).is_err());
        assert!(import_container_registered(&root, &source, &home, None, None).is_err());
        assert_eq!(
            fs::read(&source).unwrap(),
            b"legacy encrypted or invalid payload"
        );
        assert_eq!(
            fs::read(home.join("config.json")).unwrap(),
            b"native config"
        );
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    }
}
