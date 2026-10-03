//! 数据根命令：只做显式路径编排，不推导用户目录。

use crate::errors::AppResult;
use crate::modules::data_root::{
    initialize as initialize_module, load_runtime_config, resolve_opencodex_home,
    set_opencodex_home, switch_reference, validate_structure, DataRootRuntimeConfig,
    DataRootSwitchBlocked, DataRootSwitchMode, OpenCodexHomeMode, StructureValidation,
};
use serde::Serialize;

use crate::errors::AppError;
use crate::state::SharedDataRoot;

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
    pub active_data_root: String,
    pub opencodex_home_mode: OpenCodexHomeMode,
    pub opencodex_home: String,
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
) -> AppResult<DataRootConfigDto> {
    data_root_config_with_paths(&data_root.0, &data_root.0)
}

#[tauri::command]
pub async fn switch_data_root(
    request: DataRootSwitchRequest,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
    collector: tauri::State<'_, crate::state::SharedStatusCollector>,
) -> AppResult<DataRootSwitchResult> {
    let root = data_root.0.clone();
    let target = std::path::PathBuf::from(&request.target_path);
    let mode = request.mode;
    let result = crate::commands::run_blocking("switch data root", move || {
        switch_data_root_with_paths(&root, &target, mode)
    })
    .await?;
    if result.status == DataRootSwitchStatus::RestartRequired {
        reload_process_environment(&data_root.0, &mut context.inner().clone(), &collector)?;
    }
    Ok(result)
}

#[tauri::command]
pub async fn set_opencodex_home_config(
    request: OpenCodexHomeRequest,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> AppResult<DataRootSwitchResult> {
    let root = data_root.0.clone();
    let external_path = request.external_path.map(std::path::PathBuf::from);
    let mode = request.mode;
    crate::commands::run_blocking("set opencodex home", move || {
        set_opencodex_home_with_paths(&root, mode, external_path.as_deref())
    })
    .await
}

pub fn data_root_config_with_paths(
    data_root: &std::path::Path,
    runtime_root: &std::path::Path,
) -> AppResult<DataRootConfigDto> {
    let config = load_runtime_config(data_root)?;
    let opencodex_home = resolve_opencodex_home(&config);
    Ok(DataRootConfigDto {
        active_data_root: config.active_data_root.display().to_string(),
        opencodex_home_mode: config.opencodex_home_mode,
        opencodex_home: opencodex_home.display().to_string(),
        runtime_active: config.active_data_root == runtime_root,
    })
}

pub fn switch_data_root_with_paths(
    current: &std::path::Path,
    target: &std::path::Path,
    mode: DataRootSwitchMode,
) -> AppResult<DataRootSwitchResult> {
    apply_config_or_blocked(
        switch_reference(current, target, mode),
        DataRootSwitchBlocked::Corrupted,
    )
}

pub fn set_opencodex_home_with_paths(
    current: &std::path::Path,
    mode: OpenCodexHomeMode,
    external_path: Option<&std::path::Path>,
) -> AppResult<DataRootSwitchResult> {
    apply_config_or_blocked(
        set_opencodex_home(current, mode, external_path),
        DataRootSwitchBlocked::Nested,
    )
}

pub fn reload_process_environment(
    current: &std::path::Path,
    context: &mut crate::state::ProcessContext,
    collector: &crate::state::SharedStatusCollector,
) -> AppResult<bool> {
    let config = load_runtime_config(current)?;
    let opencodex_home = resolve_opencodex_home(&config);
    let runtime_active = config.active_data_root == current;
    if runtime_active {
        let mut guard = collector.lock().map_err(|_| AppError::NotConfigured)?;
        guard.source.set_environment(opencodex_home.clone());
    }
    let updated = crate::state::ProcessContext {
        opencodex_home,
        ..context.clone()
    };
    *context = updated;
    Ok(runtime_active)
}

fn apply_config_or_blocked(
    result: Result<DataRootRuntimeConfig, crate::errors::AppError>,
    blocked: DataRootSwitchBlocked,
) -> AppResult<DataRootSwitchResult> {
    match result {
        Ok(config) => Ok(DataRootSwitchResult {
            status: DataRootSwitchStatus::RestartRequired,
            blocked: None,
            config: Some(DataRootConfigDto {
                active_data_root: config.active_data_root.display().to_string(),
                opencodex_home_mode: config.opencodex_home_mode,
                opencodex_home: resolve_opencodex_home(&config).display().to_string(),
                runtime_active: false,
            }),
        }),
        Err(_) => Ok(DataRootSwitchResult {
            status: DataRootSwitchStatus::Blocked,
            blocked: Some(blocked),
            config: None,
        }),
    }
}
