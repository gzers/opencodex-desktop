//! 托盘命令边界；只读取共享状态并投影 DTO，不创建平台资源。

use serde::Serialize;

use crate::errors::{AppError, AppResult};
use crate::infrastructure::tray_controller::tray_state_from_snapshot;
use crate::modules::status::{StatusCollector, StatusSource};
use crate::modules::tray::{runtime_address, runtime_label};
use crate::state::{SharedDataRoot, SharedStatusCollector};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayDto {
    pub runtime_label: String,
    pub address: Option<String>,
    pub can_start: bool,
    pub can_stop: bool,
    pub can_restart: bool,
}

#[tauri::command]
pub fn tray_state(
    collector: tauri::State<'_, SharedStatusCollector>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> AppResult<TrayDto> {
    let collector = collector.lock().map_err(|_| AppError::NotConfigured)?;
    tray_state_from_parts(&collector, &data_root.0)
}

pub fn tray_state_from_parts<S>(
    collector: &StatusCollector<S>,
    data_root: &std::path::Path,
) -> AppResult<TrayDto>
where
    S: StatusSource,
{
    let state = tray_state_from_snapshot(&collector.snapshot(), data_root.to_path_buf());
    Ok(TrayDto {
        runtime_label: runtime_label(state.runtime).to_string(),
        address: runtime_address(&state),
        can_start: state.can_start,
        can_stop: state.can_stop,
        can_restart: state.can_restart,
    })
}
