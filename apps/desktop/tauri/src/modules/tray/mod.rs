//! MOD-02：原生托盘状态菜单的编排契约。
//!
//! 本层只维护菜单事件、运行态标签、地址展示和原生菜单项的启用矩阵；
//! Tauri 平台资源由 infrastructure / 应用接线注入。UI 不直接访问托盘。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::modules::status::HealthState;
use crate::types::status::{runtime_can_start, runtime_can_stop, RuntimeState};

/// 托盘菜单动作；动作限定在管理器自有域与界面导航，导航/设置动作在托盘域扩展。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrayAction {
    Start,
    Stop,
    Restart,
    OpenMain,
    OpenPanel,
    OpenLogs,
    OpenDataDir,
    RunDoctor,
    OpenSettings,
    /// 原生恢复入口：macOS 应用菜单／Windows 托盘直接重载主 WebView，
    /// 不经前端轮询（F-06）。
    ReloadMain,
    Quit,
}

/// 原生托盘菜单项标识；仅集中定义，不在菜单处理处重复字符串。
pub const TRAY_ID: &str = "opencodex-main";
pub const TRAY_MENU_START: &str = "tray-start";
pub const TRAY_MENU_STOP: &str = "tray-stop";
pub const TRAY_MENU_RESTART: &str = "tray-restart";
pub const TRAY_MENU_OPEN_MAIN: &str = "tray-open-main";
pub const TRAY_MENU_OPEN_PANEL: &str = "tray-open-panel";
pub const TRAY_MENU_OPEN_LOGS: &str = "tray-open-logs";
pub const TRAY_MENU_OPEN_DATA_DIR: &str = "tray-open-data-dir";
pub const TRAY_MENU_DOCTOR: &str = "tray-doctor";
pub const TRAY_MENU_OPEN_SETTINGS: &str = "tray-open-settings";
pub const NATIVE_MENU_PROCESS_SUBMENU: &str = "menu-process";
pub const NATIVE_MENU_PROCESS_START: &str = "menu-process-start";
pub const NATIVE_MENU_PROCESS_STOP: &str = "menu-process-stop";
pub const NATIVE_MENU_PROCESS_RESTART: &str = "menu-process-restart";
pub const NATIVE_MENU_VIEW_MAIN: &str = "menu-view-main";
pub const NATIVE_MENU_VIEW_RELOAD: &str = "menu-view-reload";
pub const NATIVE_MENU_VIEW_PANEL: &str = "menu-view-panel";
pub const NATIVE_MENU_VIEW_SETTINGS: &str = "menu-view-settings";
pub const NATIVE_MENU_LOGS_OPEN: &str = "menu-logs-open";
pub const NATIVE_MENU_LOGS_DATA_DIR: &str = "menu-logs-data-dir";
pub const TRAY_MENU_QUIT: &str = "tray-quit";
pub const TRAY_STATUS_HEADER: &str = "tray-status";

/// 托盘状态与动作快照。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayState {
    pub runtime: RuntimeState,
    pub health: HealthState,
    pub port: Option<u16>,
    pub pid: Option<String>,
    pub data_root: PathBuf,
}

/// 运行态在菜单顶部的展示文案。
///
/// **托盘只讲进程事实，不承载风险结论**（2026-09-24 用户决策）：`at_risk` 由
/// 「代理未在运行 + 官方 startup at-risk」折叠而来，托盘这一层能如实说的是「未运行」；
/// 「存在风险」的结论与成因留在应用内（概览 / 通知中心）展示。
/// 托盘菜单项宽度会被长文案撑开，风险说明行不再进入托盘头部与 tooltip。
pub fn runtime_label(runtime: RuntimeState) -> &'static str {
    match runtime {
        RuntimeState::Loading => "加载中",
        RuntimeState::StartingFailed => "启动失败",
        RuntimeState::ExternalTakeover => "外部接管",
        RuntimeState::AtRisk => "未运行",
        RuntimeState::Unreachable => "不可达",
        RuntimeState::Pending => "待就绪",
        RuntimeState::Starting => "启动中",
        RuntimeState::Stopping => "停止中",
        RuntimeState::Running => "运行中",
        RuntimeState::Stopped => "未运行",
        RuntimeState::NotFound => "未发现",
    }
}

/// 运行地址只在真实运行态展示；端口缺失时不猜测默认端口。
pub fn runtime_address(state: &TrayState) -> Option<String> {
    if state.runtime != RuntimeState::Running {
        return None;
    }
    state.port.map(|port| format!("本地 · 127.0.0.1:{port}"))
}

/// 动作门槛；直接复用 FZ-06 契约的运行态判定，不再自持一份白名单。
///
/// 托盘菜单与应用菜单共用本函数，任何一方私自加白名单都会让两处入口再次分叉。
pub fn enabled_actions(state: &TrayState) -> [(&'static str, bool); 3] {
    [
        (TRAY_MENU_START, runtime_can_start(state.runtime)),
        (TRAY_MENU_STOP, runtime_can_stop(state.runtime)),
        (TRAY_MENU_RESTART, runtime_can_stop(state.runtime)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(runtime: RuntimeState, port: Option<u16>) -> TrayState {
        TrayState {
            runtime,
            health: HealthState::Unknown,
            port,
            pid: None,
            data_root: PathBuf::from("/fixtures/data-root"),
        }
    }

    #[test]
    fn labels_cover_all_frozen_runtime_states() {
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
            RuntimeState::Stopped,
            RuntimeState::NotFound,
        ] {
            assert!(!runtime_label(runtime).is_empty());
        }
    }

    #[test]
    fn address_is_only_visible_when_running_with_port() {
        assert_eq!(
            runtime_address(&state(RuntimeState::Running, Some(10100))),
            Some("本地 · 127.0.0.1:10100".to_string())
        );
        assert_eq!(runtime_address(&state(RuntimeState::Running, None)), None);
        assert_eq!(
            runtime_address(&state(RuntimeState::Stopped, Some(10100))),
            None
        );
    }

    #[test]
    fn action_gates_follow_frozen_matrix() {
        assert_eq!(
            enabled_actions(&state(RuntimeState::Stopped, None))
                .into_iter()
                .collect::<Vec<_>>(),
            [
                (TRAY_MENU_START, true),
                (TRAY_MENU_STOP, false),
                (TRAY_MENU_RESTART, false)
            ]
        );
        assert_eq!(
            enabled_actions(&state(RuntimeState::Running, None))
                .into_iter()
                .collect::<Vec<_>>(),
            [
                (TRAY_MENU_START, false),
                (TRAY_MENU_STOP, true),
                (TRAY_MENU_RESTART, true)
            ]
        );
        assert_eq!(
            enabled_actions(&state(RuntimeState::Starting, None))
                .into_iter()
                .collect::<Vec<_>>(),
            [
                (TRAY_MENU_START, false),
                (TRAY_MENU_STOP, false),
                (TRAY_MENU_RESTART, false)
            ]
        );
    }

    #[test]
    fn action_gates_allow_start_when_at_risk() {
        // 回归：托盘此前手写白名单漏掉 AtRisk，代理没在跑却把「启动 OpenCodex」置灰。
        assert_eq!(
            enabled_actions(&state(RuntimeState::AtRisk, None))
                .into_iter()
                .collect::<Vec<_>>(),
            [
                (TRAY_MENU_START, true),
                (TRAY_MENU_STOP, false),
                (TRAY_MENU_RESTART, false)
            ]
        );
        // 外部接管仍不可启动：契约如此，避免代改被接管的配置。
        assert_eq!(
            enabled_actions(&state(RuntimeState::ExternalTakeover, None))
                .into_iter()
                .collect::<Vec<_>>(),
            [
                (TRAY_MENU_START, false),
                (TRAY_MENU_STOP, false),
                (TRAY_MENU_RESTART, false)
            ]
        );
    }

    #[test]
    fn tray_labels_never_carry_the_risk_verdict() {
        // 2026-09-24 用户决策：托盘只讲进程事实。at_risk 在托盘是「未运行」，
        // 「存在风险」这个结论不写进托盘（托盘菜单项会被长文案撑宽）。
        assert_eq!(runtime_label(RuntimeState::AtRisk), "未运行");
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
            RuntimeState::Stopped,
            RuntimeState::NotFound,
        ] {
            let label = runtime_label(runtime);
            assert!(
                !label.contains("风险"),
                "{runtime:?} 的托盘标签出现了风险结论"
            );
            assert!(
                !label.contains('；') && !label.contains('，'),
                "{runtime:?} 的托盘标签应是一小段标题，不带说明句"
            );
        }
    }

    #[test]
    fn menu_ids_are_frozen() {
        assert_eq!(TRAY_ID, "opencodex-main");
        assert_eq!(TRAY_MENU_QUIT, "tray-quit");
        assert_eq!(TRAY_MENU_OPEN_DATA_DIR, "tray-open-data-dir");
        assert_eq!(TRAY_MENU_OPEN_SETTINGS, "tray-open-settings");
        assert_eq!(NATIVE_MENU_PROCESS_START, "menu-process-start");
        assert_eq!(NATIVE_MENU_LOGS_DATA_DIR, "menu-logs-data-dir");
        assert_eq!(TRAY_STATUS_HEADER, "tray-status");
    }
}
