//! One actual path projection for Settings. Existence never certifies installation.
use crate::{
    errors::{AppError, AppResult},
    state::{SharedDataRoot, SharedDataRootAnchor, SharedProcessContext},
};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PathStatusV2 {
    Valid,
    Missing,
    Unreadable,
    Corrupted,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathValueV2 {
    path: String,
    status: PathStatusV2,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramStateV2 {
    package_version: Option<String>,
    manifest_version: Option<String>,
    source_version: Option<String>,
    consistency: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathsSnapshotV2 {
    schema_version: u32,
    aggregate_root: PathValueV2,
    desktop_root: PathValueV2,
    opencodex_home: PathValueV2,
    config_path: PathValueV2,
    installation_root: PathValueV2,
    runtime_prefix: PathValueV2,
    runtime_entry: PathValueV2,
    manager_executable: PathValueV2,
    program: ProgramStateV2,
    current_aggregate_root: String,
    pending_restart: bool,
}
fn path_value(path: &Path, file: bool) -> PathValueV2 {
    let status = match std::fs::symlink_metadata(path) {
        Ok(m) if m.file_type().is_symlink() => PathStatusV2::Corrupted,
        Ok(m) if file && m.is_file() || !file && m.is_dir() => PathStatusV2::Valid,
        Ok(_) => PathStatusV2::Corrupted,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => PathStatusV2::Missing,
        Err(_) => PathStatusV2::Unreadable,
    };
    PathValueV2 {
        path: path.to_string_lossy().into_owned(),
        status,
    }
}
fn version(path: &Path) -> Option<String> {
    let bytes = crate::modules::backup::safety::read_file(path, 128 * 1024).ok()?;
    serde_json::from_slice::<serde_json::Value>(&bytes)
        .ok()?
        .get("version")?
        .as_str()
        .map(String::from)
}
pub fn snapshot_with_paths(
    config: &super::data_root::DataRootConfigDto,
    install: &Path,
    manager: &Path,
) -> AppResult<PathsSnapshotV2> {
    let aggregate = PathBuf::from(&config.aggregate_root);
    let desktop = PathBuf::from(&config.active_data_root);
    let home = PathBuf::from(&config.opencodex_home);
    let prefix = install.join(crate::modules::runtime::MANAGED_PREFIX_RELATIVE);
    let entry = install.join(crate::modules::runtime::MANAGED_ENTRY_RELATIVE);
    let package_version = version(
        &prefix
            .join(crate::modules::runtime::install::PACKAGE_SUBPATH)
            .join("package.json"),
    );
    let manifest_version = version(&prefix.join(crate::modules::runtime::MANIFEST_FILENAME));
    let source =
        crate::modules::runtime::RuntimeStore::new(&PathBuf::from(&config.current_data_root))
            .load();
    let source_version = source.resolved_version;
    let (node, _) = crate::modules::runtime::install::managed_dependency_paths(install);
    let consistency =
        if crate::modules::runtime::install::validate_managed_installation(install, Some(&node))
            .is_ok()
        {
            "consistent"
        } else if !prefix.exists() && !entry.exists() {
            "missing"
        } else if package_version.is_some() || manifest_version.is_some() {
            "mismatch"
        } else {
            "unreadable"
        };
    Ok(PathsSnapshotV2 {
        schema_version: 1,
        aggregate_root: path_value(&aggregate, false),
        desktop_root: path_value(&desktop, false),
        opencodex_home: path_value(&home, false),
        config_path: path_value(&home.join("config.json"), true),
        installation_root: path_value(install, false),
        runtime_prefix: path_value(&prefix, false),
        runtime_entry: path_value(&entry, true),
        manager_executable: path_value(manager, true),
        program: ProgramStateV2 {
            package_version,
            manifest_version,
            source_version,
            consistency: consistency.into(),
        },
        current_aggregate_root: PathBuf::from(&config.current_data_root)
            .parent()
            .ok_or(AppError::NotConfigured)?
            .to_string_lossy()
            .into_owned(),
        pending_restart: !config.runtime_active,
    })
}
#[tauri::command]
pub async fn model_paths_v2_snapshot(
    data_root: tauri::State<'_, SharedDataRoot>,
    anchor: tauri::State<'_, SharedDataRootAnchor>,
    context: tauri::State<'_, SharedProcessContext>,
    runtime: tauri::State<'_, std::sync::Arc<crate::modules::runtime::RuntimeHandle>>,
) -> AppResult<PathsSnapshotV2> {
    let current = data_root.0.clone();
    let fixed = anchor.0.clone();
    let home = context.opencodex_home.clone();
    let install = runtime.installation_root().to_owned();
    crate::commands::run_readonly("read v2 directory paths", move || {
        let config = super::data_root::data_root_config_with_paths(&fixed, &current, &home)?;
        snapshot_with_paths(
            &config,
            &install,
            &std::env::current_exe().map_err(|_| AppError::NotConfigured)?,
        )
    })
    .await
}
