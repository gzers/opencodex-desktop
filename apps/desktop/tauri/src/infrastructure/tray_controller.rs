//! Infrastructure：Tauri 托盘资源接线。
//!
//! 该边界把状态快照转换为托盘菜单状态、提示与动作门控；应用编排和
//! MOD 层不直接绑定平台托盘 API。

use crate::modules::tray::{
    enabled_actions, runtime_address, runtime_label, TrayAction, TrayState,
    NATIVE_MENU_LOGS_DATA_DIR, NATIVE_MENU_LOGS_OPEN, NATIVE_MENU_PROCESS_RESTART,
    NATIVE_MENU_PROCESS_START, NATIVE_MENU_PROCESS_STOP, NATIVE_MENU_PROCESS_SUBMENU,
    NATIVE_MENU_VIEW_MAIN, NATIVE_MENU_VIEW_PANEL, NATIVE_MENU_VIEW_RELOAD,
    NATIVE_MENU_VIEW_SETTINGS, TRAY_ID, TRAY_MENU_DOCTOR, TRAY_MENU_OPEN_DATA_DIR,
    TRAY_MENU_OPEN_LOGS, TRAY_MENU_OPEN_MAIN, TRAY_MENU_OPEN_PANEL, TRAY_MENU_OPEN_SETTINGS,
    TRAY_MENU_QUIT, TRAY_MENU_RESTART, TRAY_MENU_START, TRAY_MENU_STOP, TRAY_STATUS_HEADER,
};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};

/// 托盘 UI 更新边界；隔离 Tauri 资源，便于测试状态投影。
pub type SharedTrayPresenter = std::sync::Arc<dyn TrayPresenter>;

pub trait TrayPresenter: Send + Sync {
    fn update(&self, state: &TrayState);
}

/// Tauri 托盘控制器；在事件循环线程安全地更新菜单头与动作门控。
pub struct TauriTrayController<R: tauri::Runtime> {
    app: tauri::AppHandle<R>,
    menu: tauri::menu::Menu<R>,
    /// 顶部应用菜单（进程/视图/日志）。它和托盘菜单是两份独立资源，
    /// 必须一并门控，否则应用菜单里的进程项会永远停在创建时的禁用态。
    native_menu: Option<tauri::menu::Menu<R>>,
    last: std::sync::Mutex<Option<(String, [bool; 3])>>,
}

impl<R: tauri::Runtime> TauriTrayController<R> {
    pub fn new(
        app: tauri::AppHandle<R>,
        menu: tauri::menu::Menu<R>,
        native_menu: Option<tauri::menu::Menu<R>>,
    ) -> Self {
        Self {
            app,
            menu,
            native_menu,
            last: std::sync::Mutex::new(None),
        }
    }

    /// 构建 Tauri 托盘菜单；菜单项 ID 是 MOD-02 冻结契约。
    pub fn build_menu(app: &tauri::AppHandle<R>) -> tauri::Result<tauri::menu::Menu<R>> {
        let header =
            MenuItem::with_id(app, TRAY_STATUS_HEADER, "状态读取中", false, None::<String>)?;
        let start = MenuItem::with_id(
            app,
            TRAY_MENU_START,
            "启动 OpenCodex",
            false,
            None::<String>,
        )?;
        let stop = MenuItem::with_id(app, TRAY_MENU_STOP, "停止 OpenCodex", false, None::<String>)?;
        let restart = MenuItem::with_id(
            app,
            TRAY_MENU_RESTART,
            "重启 OpenCodex",
            false,
            None::<String>,
        )?;
        let open_main =
            MenuItem::with_id(app, TRAY_MENU_OPEN_MAIN, "打开主界面", true, None::<String>)?;
        let open_panel = MenuItem::with_id(
            app,
            TRAY_MENU_OPEN_PANEL,
            "打开扩展面板",
            true,
            None::<String>,
        )?;
        let open_logs = MenuItem::with_id(
            app,
            TRAY_MENU_OPEN_LOGS,
            "打开诊断中心",
            true,
            None::<String>,
        )?;
        let open_data_dir = MenuItem::with_id(
            app,
            TRAY_MENU_OPEN_DATA_DIR,
            "打开数据目录",
            true,
            None::<String>,
        )?;
        let doctor = MenuItem::with_id(app, TRAY_MENU_DOCTOR, "运行 Doctor", true, None::<String>)?;
        let open_settings = MenuItem::with_id(
            app,
            TRAY_MENU_OPEN_SETTINGS,
            "快速设置",
            true,
            None::<String>,
        )?;
        let quit = MenuItem::with_id(app, TRAY_MENU_QUIT, "退出", true, None::<String>)?;
        let menu = Menu::with_items(
            app,
            &[
                &header,
                &PredefinedMenuItem::separator(app)?,
                &start,
                &stop,
                &restart,
                &PredefinedMenuItem::separator(app)?,
                &open_main,
                &open_panel,
                &open_logs,
                &open_data_dir,
                &doctor,
                &open_settings,
                &PredefinedMenuItem::separator(app)?,
                &quit,
            ],
        )?;
        // F-06 must remain reachable without a functioning WebView after the
        // Windows application menu row is removed. Reuse its existing native
        // action ID and direct reload path; do not add a new proxy action.
        #[cfg(windows)]
        menu.insert(
            &MenuItem::with_id(
                app,
                NATIVE_MENU_VIEW_RELOAD,
                "重载主界面",
                true,
                None::<String>,
            )?,
            7,
        )?;
        Ok(menu)
    }
}

impl<R: tauri::Runtime> TrayPresenter for TauriTrayController<R> {
    fn update(&self, state: &TrayState) {
        let label = runtime_label(state.runtime);
        let address = runtime_address(state);
        let headline = address.as_deref().map_or_else(
            || label.to_string(),
            |address| format!("{label} · {address}"),
        );
        let semantic = (
            headline.clone(),
            process_menu_gates(state).map(|(_, _, enabled)| enabled),
        );
        let Ok(mut last) = self.last.lock() else {
            return;
        };
        if last.as_ref() == Some(&semantic) {
            return;
        }
        let mut succeeded = true;
        // 托盘只放「运行标签 · 地址」这一小段：风险结论与成因说明留在应用内
        // （概览 / 通知中心）。此前把说明句拼进头部与 tooltip，托盘菜单项被撑得很宽。
        let header_text = headline;
        let tooltip = format!("OpenCodeX Desktop · {header_text}");

        if let Some(tray) = self.app.tray_by_id(TRAY_ID) {
            succeeded &= tray.set_tooltip(Some(tooltip)).is_ok();
        }

        if let Some(tauri::menu::MenuItemKind::MenuItem(header)) = self.menu.get(TRAY_STATUS_HEADER)
        {
            succeeded &= header.set_text(header_text).is_ok();
        }
        for (tray_id, native_id, enabled) in process_menu_gates(state) {
            if let Some(tauri::menu::MenuItemKind::MenuItem(item)) = self.menu.get(tray_id) {
                succeeded &= item.set_enabled(enabled).is_ok();
            }
            if let Some(native_menu) = self.native_menu.as_ref() {
                // 进程项在「进程」子菜单里，`Menu::get` 只查直接子项取不到，
                // 必须下钻一层；否则应用菜单里的进程项会永远停在创建时的禁用态。
                if let Some(item) = native_process_item(native_menu, native_id) {
                    succeeded &= item.set_enabled(enabled).is_ok();
                }
            }
        }
        if succeeded {
            *last = Some(semantic);
        }
    }
}

/// 一次状态刷新要写入的全部进程动作门控：`(托盘菜单项, 顶部应用菜单项, 是否可用)`。
///
/// 托盘菜单与顶部应用菜单是两份独立资源，进程动作在这两处都有入口；
/// 门控必须同时覆盖，否则应用菜单里的进程项会永远停在创建时的禁用态。
pub fn process_menu_gates(state: &TrayState) -> [(&'static str, &'static str, bool); 3] {
    enabled_actions(state).map(|(tray_id, enabled)| {
        (
            tray_id,
            native_menu_id(tray_id).expect("被门控的托盘动作都应有对应的应用菜单项"),
            enabled,
        )
    })
}

/// 取应用菜单里「进程」子菜单下的菜单项。
///
/// `Menu::get` 只查该菜单的直接子项，而进程动作在 `NATIVE_MENU_PROCESS_SUBMENU`
/// 子菜单里，因此必须下钻一层，否则永远取不到、也就永远门控不到。
pub fn native_process_item<R: tauri::Runtime>(
    menu: &tauri::menu::Menu<R>,
    id: &str,
) -> Option<tauri::menu::MenuItem<R>> {
    match menu.get(NATIVE_MENU_PROCESS_SUBMENU) {
        Some(tauri::menu::MenuItemKind::Submenu(submenu)) => match submenu.get(id) {
            Some(tauri::menu::MenuItemKind::MenuItem(item)) => Some(item),
            _ => None,
        },
        _ => None,
    }
}

/// 托盘动作 ID 到应用菜单动作 ID 的映射；两者是同一冻结动作域的不同入口。
pub fn native_menu_id(tray_id: &str) -> Option<&'static str> {
    match tray_id {
        TRAY_MENU_START => Some(NATIVE_MENU_PROCESS_START),
        TRAY_MENU_STOP => Some(NATIVE_MENU_PROCESS_STOP),
        TRAY_MENU_RESTART => Some(NATIVE_MENU_PROCESS_RESTART),
        _ => None,
    }
}

/// 把状态采集投影为托盘契约；不读取平台资源。
pub fn tray_state_from_snapshot(
    snapshot: &crate::types::runtime_status::StatusSnapshotDto,
    data_root: std::path::PathBuf,
) -> TrayState {
    TrayState {
        runtime: snapshot.matrix.runtime,
        health: snapshot.facts.health,
        port: snapshot.port,
        pid: snapshot.pid.clone(),
        data_root,
    }
}

/// 冻结托盘动作集合；菜单事件映射不得引入第十一项。
pub const TRAY_ACTIONS: [TrayAction; 10] = [
    TrayAction::Start,
    TrayAction::Stop,
    TrayAction::Restart,
    TrayAction::OpenMain,
    TrayAction::OpenPanel,
    TrayAction::OpenLogs,
    TrayAction::OpenDataDir,
    TrayAction::RunDoctor,
    TrayAction::OpenSettings,
    TrayAction::Quit,
];

/// 构建顶部应用菜单，提供进程、视图与日志入口；动作仍走托盘请求桥。
pub fn build_app_menu<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> tauri::Result<tauri::menu::Menu<R>> {
    let process = Submenu::with_id_and_items(
        app,
        NATIVE_MENU_PROCESS_SUBMENU,
        "进程",
        true,
        &[
            &MenuItem::with_id(
                app,
                NATIVE_MENU_PROCESS_START,
                "启动 OpenCodex",
                false,
                None::<String>,
            )?,
            &MenuItem::with_id(
                app,
                NATIVE_MENU_PROCESS_STOP,
                "停止 OpenCodex",
                false,
                None::<String>,
            )?,
            &MenuItem::with_id(
                app,
                NATIVE_MENU_PROCESS_RESTART,
                "重启 OpenCodex",
                false,
                None::<String>,
            )?,
        ],
    )?;
    let view = Submenu::with_id_and_items(
        app,
        "menu-view",
        "视图",
        true,
        &[
            &MenuItem::with_id(
                app,
                NATIVE_MENU_VIEW_MAIN,
                "打开主界面",
                true,
                None::<String>,
            )?,
            &MenuItem::with_id(
                app,
                NATIVE_MENU_VIEW_RELOAD,
                "重载主界面",
                true,
                None::<String>,
            )?,
            &MenuItem::with_id(
                app,
                NATIVE_MENU_VIEW_PANEL,
                "打开扩展面板",
                true,
                None::<String>,
            )?,
            &MenuItem::with_id(
                app,
                NATIVE_MENU_VIEW_SETTINGS,
                "快速设置",
                true,
                None::<String>,
            )?,
        ],
    )?;
    let logs = Submenu::with_id_and_items(
        app,
        "menu-logs",
        "诊断",
        true,
        &[
            &MenuItem::with_id(
                app,
                NATIVE_MENU_LOGS_OPEN,
                "打开诊断中心",
                true,
                None::<String>,
            )?,
            &MenuItem::with_id(
                app,
                NATIVE_MENU_LOGS_DATA_DIR,
                "打开数据目录",
                true,
                None::<String>,
            )?,
        ],
    )?;
    // 编辑子菜单是必需的：macOS 的 ⌘C / ⌘V / ⌘X / ⌘A 由应用菜单的 key equivalent
    // 经 responder chain 转发给 WKWebView。此前应用菜单只有进程 / 视图 / 日志，
    // 顶掉了默认菜单，导致所有输入框都无法用键盘复制粘贴。这些项走系统预定义
    // 行为，不产生菜单事件，也不属于冻结的托盘动作域。
    let edit = Submenu::with_id_and_items(
        app,
        "menu-edit",
        "编辑",
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;
    Menu::with_items(app, &[&process, &edit, &view, &logs])
}

/// 把原生菜单事件映射为同一冻结动作域；ID 不在事件处理处重复字符串。
pub fn native_menu_action(id: &str) -> Option<TrayAction> {
    match id {
        NATIVE_MENU_PROCESS_START => Some(TrayAction::Start),
        NATIVE_MENU_PROCESS_STOP => Some(TrayAction::Stop),
        NATIVE_MENU_PROCESS_RESTART => Some(TrayAction::Restart),
        NATIVE_MENU_VIEW_MAIN => Some(TrayAction::OpenMain),
        NATIVE_MENU_VIEW_RELOAD => Some(TrayAction::ReloadMain),
        NATIVE_MENU_VIEW_PANEL => Some(TrayAction::OpenPanel),
        NATIVE_MENU_VIEW_SETTINGS => Some(TrayAction::OpenSettings),
        NATIVE_MENU_LOGS_OPEN => Some(TrayAction::OpenLogs),
        NATIVE_MENU_LOGS_DATA_DIR => Some(TrayAction::OpenDataDir),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::runtime_status::unconfigured_snapshot;

    #[test]
    fn projects_runtime_address_and_running_state() {
        let mut snapshot = unconfigured_snapshot();
        snapshot.matrix.runtime = crate::types::status::RuntimeState::Running;
        snapshot.port = Some(10100);
        let state =
            tray_state_from_snapshot(&snapshot, std::path::PathBuf::from("/fixtures/data-root"));
        assert_eq!(state.runtime, crate::types::status::RuntimeState::Running);
        assert_eq!(
            runtime_address(&state).as_deref(),
            Some("本地 · 127.0.0.1:10100")
        );
    }

    #[test]
    fn action_contract_covers_all_menu_targets() {
        assert_eq!(TRAY_ACTIONS.len(), 10);
        assert!(TRAY_ACTIONS.contains(&TrayAction::Quit));
        assert_eq!(
            native_menu_action(NATIVE_MENU_PROCESS_START),
            Some(TrayAction::Start)
        );
        assert_eq!(
            native_menu_action(NATIVE_MENU_LOGS_DATA_DIR),
            Some(TrayAction::OpenDataDir)
        );
        // 回归（F-06）：原生「重载主界面」必须映射为独立动作，且不进入前端请求域。
        assert_eq!(
            native_menu_action(NATIVE_MENU_VIEW_RELOAD),
            Some(TrayAction::ReloadMain)
        );
        assert!(!TRAY_ACTIONS.contains(&TrayAction::ReloadMain));
    }

    // 回归：应用菜单的进程项与托盘菜单是两份资源。若门控只覆盖托盘菜单，
    // 顶部菜单里的「启动/停止/重启 OpenCodex」会永远停在创建时的禁用态。
    #[test]
    fn process_gates_cover_both_the_tray_and_the_app_menu() {
        assert_eq!(
            native_menu_id(TRAY_MENU_START),
            Some(NATIVE_MENU_PROCESS_START)
        );
        assert_eq!(
            native_menu_id(TRAY_MENU_STOP),
            Some(NATIVE_MENU_PROCESS_STOP)
        );
        assert_eq!(
            native_menu_id(TRAY_MENU_RESTART),
            Some(NATIVE_MENU_PROCESS_RESTART)
        );
        assert_eq!(native_menu_id(TRAY_MENU_QUIT), None);

        let mut state = tray_state_from_snapshot(
            &crate::types::runtime_status::unconfigured_snapshot(),
            std::path::PathBuf::from("/fixtures/data-root"),
        );
        // 未运行：只允许启动，且该结论必须同时落到托盘菜单与应用菜单。
        state.runtime = crate::types::status::RuntimeState::Stopped;
        assert_eq!(
            process_menu_gates(&state),
            [
                (TRAY_MENU_START, NATIVE_MENU_PROCESS_START, true),
                (TRAY_MENU_STOP, NATIVE_MENU_PROCESS_STOP, false),
                (TRAY_MENU_RESTART, NATIVE_MENU_PROCESS_RESTART, false),
            ]
        );

        // 运行中：可停止/重启，不可再启动。
        state.runtime = crate::types::status::RuntimeState::Running;
        assert_eq!(
            process_menu_gates(&state),
            [
                (TRAY_MENU_START, NATIVE_MENU_PROCESS_START, false),
                (TRAY_MENU_STOP, NATIVE_MENU_PROCESS_STOP, true),
                (TRAY_MENU_RESTART, NATIVE_MENU_PROCESS_RESTART, true),
            ]
        );
    }
}
