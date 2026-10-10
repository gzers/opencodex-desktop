//! 数据根命令：只做显式路径编排，不推导用户目录。

use crate::errors::AppResult;
use crate::modules::data_root::{
    initialize as initialize_module, load_runtime_config, resolve_opencodex_home,
    set_opencodex_home, switch_reference, validate_structure, DataRootRuntimeConfig,
    DataRootSwitchBlocked, DataRootSwitchMode, OpenCodexHomeMode, StructureValidation,
};
use serde::Serialize;

use crate::errors::AppError;
use crate::state::{SharedDataRoot, SharedDataRootAnchor, SharedProcessContext};
use tauri::Manager;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataRootRequest {
    pub root_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataRootInitialization {
    pub created: bool,
    pub structure_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataRootSwitchRequest {
    pub target_path: String,
    pub mode: DataRootSwitchMode,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodexHomeRequest {
    pub mode: OpenCodexHomeMode,
    #[serde(default)]
    pub external_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataRootConfigDto {
    /// Saved binding, which can differ from immutable startup services.
    pub active_data_root: String,
    pub opencodex_home_mode: OpenCodexHomeMode,
    pub opencodex_home: String,
    pub current_data_root: String,
    pub current_opencodex_home: String,
    pub runtime_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataRootSwitchResult {
    pub status: DataRootSwitchStatus,
    pub blocked: Option<DataRootSwitchBlocked>,
    pub config: Option<DataRootConfigDto>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DataRootSwitchStatus {
    Applied,
    RestartRequired,
    Blocked,
}

#[tauri::command]
pub async fn initialize_data_root(request: DataRootRequest) -> AppResult<DataRootInitialization> {
    let root = std::path::PathBuf::from(request.root_path);
    crate::commands::run_blocking("initialize data root", move || {
        let created = initialize_module(&root)?;
        Ok(DataRootInitialization {
            created,
            structure_version: crate::modules::data_root::STRUCTURE_VERSION.to_string(),
        })
    })
    .await
}

#[tauri::command]
pub async fn validate_data_root_structure(root_path: String) -> AppResult<StructureValidation> {
    crate::commands::run_blocking("validate data root", move || {
        validate_structure(std::path::Path::new(&root_path))
    })
    .await
}

#[tauri::command]
pub fn get_data_root_config(
    data_root: tauri::State<'_, SharedDataRoot>,
    anchor: tauri::State<'_, SharedDataRootAnchor>,
    context: tauri::State<'_, SharedProcessContext>,
) -> AppResult<DataRootConfigDto> {
    data_root_config_with_paths(&anchor.0, &data_root.0, &context.opencodex_home)
}

#[tauri::command]
pub async fn switch_data_root(
    request: DataRootSwitchRequest,
    anchor: tauri::State<'_, SharedDataRootAnchor>,
    app: tauri::AppHandle,
) -> AppResult<DataRootSwitchResult> {
    let root = anchor.0.clone();
    let target = std::path::PathBuf::from(&request.target_path);
    let mode = request.mode;
    let result = crate::commands::run_blocking("switch data root", move || {
        change_binding(&app, || switch_data_root_with_paths(&root, &target, mode))
    })
    .await?;
    Ok(result)
}

#[tauri::command]
pub async fn set_opencodex_home_config(
    request: OpenCodexHomeRequest,
    anchor: tauri::State<'_, SharedDataRootAnchor>,
    app: tauri::AppHandle,
) -> AppResult<DataRootSwitchResult> {
    let root = anchor.0.clone();
    let external_path = request.external_path.map(std::path::PathBuf::from);
    let mode = request.mode;
    crate::commands::run_blocking("set opencodex home", move || {
        change_binding(&app, || {
            set_opencodex_home_with_paths(&root, mode, external_path.as_deref())
        })
    })
    .await
}

pub fn data_root_config_with_paths(
    data_root: &std::path::Path,
    runtime_root: &std::path::Path,
    runtime_home: &std::path::Path,
) -> AppResult<DataRootConfigDto> {
    let config = load_runtime_config(data_root)?;
    Ok(project_config(&config, runtime_root, runtime_home))
}

fn project_config(
    config: &DataRootRuntimeConfig,
    runtime_root: &std::path::Path,
    runtime_home: &std::path::Path,
) -> DataRootConfigDto {
    let opencodex_home = resolve_opencodex_home(config);
    DataRootConfigDto {
        active_data_root: config.active_data_root.display().to_string(),
        opencodex_home_mode: config.opencodex_home_mode,
        opencodex_home: opencodex_home.display().to_string(),
        current_data_root: runtime_root.display().to_string(),
        current_opencodex_home: runtime_home.display().to_string(),
        runtime_active: config.active_data_root == runtime_root && opencodex_home == runtime_home,
    }
}

pub fn switch_data_root_with_paths(
    current: &std::path::Path,
    target: &std::path::Path,
    mode: DataRootSwitchMode,
) -> AppResult<DataRootSwitchResult> {
    let before = load_runtime_config(current)?;
    apply_config_or_blocked(
        switch_reference(current, target, mode),
        DataRootSwitchBlocked::Corrupted,
        &before,
    )
}

pub fn set_opencodex_home_with_paths(
    current: &std::path::Path,
    mode: OpenCodexHomeMode,
    external_path: Option<&std::path::Path>,
) -> AppResult<DataRootSwitchResult> {
    let before = load_runtime_config(current)?;
    apply_config_or_blocked(
        set_opencodex_home(current, mode, external_path),
        DataRootSwitchBlocked::Nested,
        &before,
    )
}

// Excludes lifecycle/install operations while saving. This is not the full
// storage-writer quiescence required for migration or live rebinding.
fn change_binding(
    app: &tauri::AppHandle,
    change: impl FnOnce() -> AppResult<DataRootSwitchResult>,
) -> AppResult<DataRootSwitchResult> {
    let mutation = app.state::<crate::state::SharedRuntimeInstall>();
    let Some(_lease) = mutation.acquire() else {
        return Ok(blocked_running());
    };
    {
        let collector = app.state::<crate::state::SharedStatusCollector>();
        let mut collector = collector.lock().map_err(|_| AppError::NotConfigured)?;
        if collector.refresh().is_err() || !binding_runtime_is_idle(collector.matrix().runtime) {
            return Ok(blocked_running());
        }
    }
    let root = app.state::<SharedDataRoot>();
    // Same lock order as preference writes: filesystem transaction, status.
    let _transaction = crate::modules::backup::manager::acquire_preferences_transaction(&root.0)?;
    let status = app.state::<crate::commands::update::SharedUpdateStatus>();
    let status = status.lock().map_err(|_| AppError::NotConfigured)?;
    if status.installing || status.pending_restart.is_some() {
        return Ok(blocked_running());
    }
    let context = app.state::<SharedProcessContext>();
    let mut result = change()?;
    if let Some(config) = result.config.as_mut() {
        config.current_data_root = root.0.display().to_string();
        config.current_opencodex_home = context.opencodex_home.display().to_string();
        config.runtime_active = config.active_data_root == config.current_data_root
            && config.opencodex_home == config.current_opencodex_home;
    }
    Ok(result)
}

fn binding_runtime_is_idle(runtime: crate::types::status::RuntimeState) -> bool {
    matches!(
        runtime,
        crate::types::status::RuntimeState::Stopped | crate::types::status::RuntimeState::NotFound
    )
}

fn blocked_running() -> DataRootSwitchResult {
    DataRootSwitchResult {
        status: DataRootSwitchStatus::Blocked,
        blocked: Some(DataRootSwitchBlocked::Running),
        config: None,
    }
}

fn apply_config_or_blocked(
    result: Result<DataRootRuntimeConfig, crate::errors::AppError>,
    blocked: DataRootSwitchBlocked,
    before: &DataRootRuntimeConfig,
) -> AppResult<DataRootSwitchResult> {
    match result {
        Ok(config) => Ok(DataRootSwitchResult {
            status: DataRootSwitchStatus::RestartRequired,
            blocked: None,
            config: Some(project_config(
                &config,
                &before.active_data_root,
                &resolve_opencodex_home(before),
            )),
        }),
        Err(_) => Ok(DataRootSwitchResult {
            status: DataRootSwitchStatus::Blocked,
            blocked: Some(blocked),
            config: None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::status::RuntimeState;

    #[test]
    fn only_observed_idle_runtime_allows_binding_save() {
        for runtime in [
            RuntimeState::Loading,
            RuntimeState::StartingFailed,
            RuntimeState::ExternalTakeover,
            RuntimeState::AtRisk,
            RuntimeState::Unreachable,
            RuntimeState::Pending,
            RuntimeState::Starting,
            RuntimeState::Stopping,
            RuntimeState::Running,
        ] {
            assert!(!binding_runtime_is_idle(runtime), "{runtime:?}");
        }
        assert!(binding_runtime_is_idle(RuntimeState::Stopped));
        assert!(binding_runtime_is_idle(RuntimeState::NotFound));
        assert_eq!(
            blocked_running().blocked,
            Some(DataRootSwitchBlocked::Running)
        );
    }

    #[test]
    fn home_only_save_stays_pending_until_actual_startup_home_matches() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("data");
        initialize_module(&root).unwrap();
        let actual_home = root.join("opencodex-home");
        let external = temp.path().join("external-home");
        let result =
            set_opencodex_home_with_paths(&root, OpenCodexHomeMode::External, Some(&external))
                .unwrap();
        let config = result.config.unwrap();
        assert_eq!(result.status, DataRootSwitchStatus::RestartRequired);
        assert!(!config.runtime_active);
        assert_eq!(
            config.current_opencodex_home,
            actual_home.display().to_string()
        );
        let reread = data_root_config_with_paths(&root, &root, &actual_home).unwrap();
        assert!(!reread.runtime_active);
        assert_eq!(reread.opencodex_home, external.display().to_string());
        assert!(
            data_root_config_with_paths(&root, &root, &external)
                .unwrap()
                .runtime_active
        );
        let wire = serde_json::to_value(reread).unwrap();
        assert!(wire.get("currentDataRoot").is_some());
        assert!(wire.get("currentOpencodexHome").is_some());
    }

    #[test]
    fn reference_switch_preserves_external_home_without_copying() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("source");
        let target = temp.path().join("target");
        let external = temp.path().join("external-home");
        initialize_module(&root).unwrap();
        initialize_module(&target).unwrap();
        set_opencodex_home_with_paths(&root, OpenCodexHomeMode::External, Some(&external)).unwrap();
        std::fs::write(root.join("manager-state/source-only"), b"untouched").unwrap();
        let result =
            switch_data_root_with_paths(&root, &target, DataRootSwitchMode::ReferenceOnly).unwrap();
        let config = result.config.unwrap();
        assert_eq!(config.current_data_root, root.display().to_string());
        assert_eq!(config.active_data_root, target.display().to_string());
        assert_eq!(config.opencodex_home, external.display().to_string());
        assert!(!config.runtime_active);
        assert!(!target.join("manager-state/source-only").exists());
    }
}
