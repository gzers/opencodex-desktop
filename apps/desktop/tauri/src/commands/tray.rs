//! 托盘命令边界；只读取共享状态并投影 DTO，不创建平台资源。

use serde::Serialize;

use crate::errors::{AppError, AppResult};
use crate::modules::status::{StatusCollector, StatusSource};
use crate::modules::tray::{enabled_actions, runtime_address, runtime_label, TrayState};
use crate::state::{SharedDataRoot, SharedStatusCollector};
use crate::types::runtime_status::{snapshot_from_matrix, StatusSource as DtoStatusSource};

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
    let snapshot = snapshot_from_matrix(
        collector.matrix(),
        collector.facts(),
        collector.port(),
        collector.pid(),
        DtoStatusSource::Live,
    );
    let state = TrayState {
        runtime: snapshot.matrix.runtime,
        health: snapshot.facts.health,
        port: snapshot.port,
        pid: snapshot.pid,
        data_root: data_root.to_path_buf(),
    };
    let actions = enabled_actions(&state);
    Ok(TrayDto {
        runtime_label: runtime_label(state.runtime).to_string(),
        address: runtime_address(&state),
        can_start: actions[0].1,
        can_stop: actions[1].1,
        can_restart: actions[2].1,
    })
}
