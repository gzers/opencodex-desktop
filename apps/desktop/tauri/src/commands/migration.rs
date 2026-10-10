//! 配置迁移命令层。只做数据根编排和 DTO 投影，不访问平台细节。

use crate::modules::migration;
use crate::state::{SharedDataRoot, SharedHomeDir};
use crate::types::migration::{
    MigrationExportResult, MigrationImportRequest, MigrationImportResult,
};

#[tauri::command]
pub async fn export_migration(
    app: tauri::AppHandle,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> crate::errors::AppResult<MigrationExportResult> {
    let root = data_root.0.clone();
    let version = app.package_info().version.to_string();
    crate::commands::run_blocking("export migration container", move || {
        let target = root
            .join(migration::EXPORTS_RELATIVE_PATH)
            .join(migration::EXPORT_FILE_NAME);
        export_container_registered(&root, &target, &version, Some(&app)).map(Into::into)
    })
    .await
}

/// Both GUI and CLI use this export terminal adapter in their owned workers.
/// No AppHandle (isolated library/test use) does not fabricate persistent delivery.
pub(crate) fn export_container_registered(
    root: &std::path::Path,
    target: &std::path::Path,
    version: &str,
    app: Option<&tauri::AppHandle>,
) -> crate::errors::AppResult<migration::ExportResult> {
    match app {
        Some(app) => migration::export_with_container_file_observed(
            root,
            target,
            version,
            &mut ExportEvents {
                app,
                root,
                identity: None,
            },
        ),
        None => migration::export_with_container_file(root, target, version),
    }
}

/// Lossless path bytes are transient hash input, never notification fields.
pub(crate) fn export_candidate(document_sha256: &str, target: &std::path::Path) -> Vec<u8> {
    [
        document_sha256.as_bytes(),
        b"\0",
        target.as_os_str().as_encoded_bytes(),
    ]
    .concat()
}
struct ExportEvents<'a> {
    app: &'a tauri::AppHandle,
    root: &'a std::path::Path,
    identity: Option<crate::modules::notifications::registry::EventIdentity>,
}
impl migration::ExportObserver for ExportEvents<'_> {
    fn begin(&mut self, document_sha256: &str, target: &std::path::Path) {
        self.identity = crate::commands::event_delivery::prepare(
            self.root,
            "config-export-failed",
            crate::modules::notifications::registry::Channel::Local,
            &export_candidate(document_sha256, target),
        );
    }
    fn completed(&mut self, succeeded: bool) {
        crate::commands::event_delivery::publish(
            self.app,
            self.root,
            if succeeded {
                "config-export-succeeded"
            } else {
                "config-export-failed"
            },
            self.identity.take(),
            crate::modules::notifications::registry::Trigger::User,
        );
    }
}

#[tauri::command]
pub async fn import_migration(
    app: tauri::AppHandle,
    request: MigrationImportRequest,
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, SharedHomeDir>,
) -> crate::errors::AppResult<MigrationImportResult> {
    let root = data_root.0.clone();
    let home = home.0.clone();
    crate::commands::run_blocking("import migration container", move || {
        let source = root
            .join(migration::EXPORTS_RELATIVE_PATH)
            .join(migration::EXPORT_FILE_NAME);
        import_container_registered(
            &root,
            &source,
            &home,
            request.passphrase.as_deref(),
            Some(&app),
        )
        .map(Into::into)
    })
    .await
}

/// GUI and IPC share the admitted import terminal; no AppHandle means library-only.
pub(crate) fn import_container_registered(
    root: &std::path::Path,
    source: &std::path::Path,
    home: &std::path::Path,
    passphrase: Option<&str>,
    app: Option<&tauri::AppHandle>,
) -> crate::errors::AppResult<migration::ImportResult> {
    match app {
        Some(app) => migration::import_with_container_file_observed(
            root,
            source,
            home,
            passphrase,
            &mut ImportEvents {
                app,
                root,
                identity: None,
            },
        ),
        None => migration::import_with_container_file(root, source, home, passphrase),
    }
}

pub(crate) fn import_candidate(document_sha256: &str, home: &std::path::Path) -> Vec<u8> {
    [
        document_sha256.as_bytes(),
        b"\0",
        home.as_os_str().as_encoded_bytes(),
    ]
    .concat()
}
struct ImportEvents<'a> {
    app: &'a tauri::AppHandle,
    root: &'a std::path::Path,
    identity: Option<crate::modules::notifications::registry::EventIdentity>,
}
impl migration::ImportObserver for ImportEvents<'_> {
    fn begin(&mut self, document_sha256: &str, home: &std::path::Path) {
        self.identity = crate::commands::event_delivery::prepare(
            self.root,
            "config-import-failed",
            crate::modules::notifications::registry::Channel::Local,
            &import_candidate(document_sha256, home),
        );
    }
    fn completed(&mut self, succeeded: bool) {
        crate::commands::event_delivery::publish(
            self.app,
            self.root,
            if succeeded {
                "config-import-succeeded"
            } else {
                "config-import-failed"
            },
            self.identity.take(),
            crate::modules::notifications::registry::Trigger::User,
        );
    }
}

pub fn export_migration_with_root(
    data_root: &std::path::Path,
    app_version: String,
) -> Result<migration::ExportResult, crate::errors::AppError> {
    migration::export_with_paths(data_root, &app_version)
}

pub fn import_migration_with_root(
    data_root: &std::path::Path,
    home: &std::path::Path,
    passphrase: Option<&str>,
) -> Result<migration::ImportResult, crate::errors::AppError> {
    migration::import_with_paths(data_root, home, passphrase)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::modules::preferences::{Preferences, PreferencesStore};
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn command_layer_round_trips_preferences_through_data_root() {
        let root = tempfile::tempdir().expect("temporary data root");
        let home = tempfile::tempdir().expect("temporary home");
        crate::modules::data_root::initialize(root.path()).expect("initialize data root");
        let original = Preferences {
            interface_scale: 175,
            ..Default::default()
        };
        PreferencesStore::new(root.path())
            .save(&original)
            .expect("save preferences");

        let exported =
            export_migration_with_root(root.path(), env!("CARGO_PKG_VERSION").to_string())
                .expect("export through command layer");
        // 新流程不索要额外口令。
        assert_eq!(exported.format_version, 2);
        assert!(exported.sections.iter().any(|item| item == "preferences"));
        assert!(!exported.excluded.is_empty());
        assert_eq!(
            std::fs::metadata(&exported.path)
                .expect("container metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        // 新容器是明文：文件里不应出现任何口令字段。
        let raw = std::fs::read_to_string(&exported.path).expect("read container");
        assert!(!raw.contains("password"));
        assert!(!raw.contains("passphrase"));

        PreferencesStore::new(root.path())
            .save(&Preferences::default())
            .expect("change current preferences");
        let imported = import_migration_with_root(root.path(), home.path(), None).expect("import");
        assert_eq!(imported.document_sha256, exported.document_sha256);
        assert_eq!(imported.format_version, 2);
        assert!(imported
            .applied_sections
            .iter()
            .any(|item| item == "preferences"));
        let reloaded = PreferencesStore::new(root.path())
            .load()
            .expect("reload preferences");
        assert_eq!(reloaded, original);
    }
}
