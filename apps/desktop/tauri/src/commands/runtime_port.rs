use crate::{
    errors::AppResult,
    modules::runtime::port::{self, SaveRequest, Snapshot},
    state::{SharedDataRoot, SharedProcessContext, SharedRuntimeInstall},
};
use tauri::Manager;

#[tauri::command]
pub async fn runtime_port_load(
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
) -> AppResult<Snapshot> {
    let root = data_root.0.clone();
    let home = context.opencodex_home.clone();
    super::run_readonly("read runtime port", move || port::load(&root, &home)).await
}
#[tauri::command]
pub async fn runtime_port_save(
    request: SaveRequest,
    app: tauri::AppHandle,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
) -> AppResult<Snapshot> {
    let root = data_root.0.clone();
    let home = context.opencodex_home.clone();
    let lease = app
        .state::<SharedRuntimeInstall>()
        .acquire()
        .ok_or(crate::errors::AppError::NotConfigured)?;
    super::run_blocking("save runtime port", move || {
        let _lease = lease;
        port::save(&root, &home, &request)
    })
    .await
}
