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
        export_migration_with_root(&root, version).map(Into::into)
    })
    .await
}

#[tauri::command]
pub async fn import_migration(
    request: MigrationImportRequest,
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, SharedHomeDir>,
) -> crate::errors::AppResult<MigrationImportResult> {
    let root = data_root.0.clone();
    let home = home.0.clone();
    crate::commands::run_blocking("import migration container", move || {
        import_migration_with_root(&root, &home, request.passphrase.as_deref()).map(Into::into)
    })
    .await
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

#[cfg(test)]
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
