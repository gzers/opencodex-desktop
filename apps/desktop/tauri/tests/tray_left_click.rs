//! 托盘左键交互契约（F-01）。
//!
//! 回归背景：`tauri.conf.json` 的 `showMenuOnLeftClick` 曾为 `false`，托盘图标左键
//! 始终不展开菜单（右键仍可用），与「托盘菜单入口可用」的验收口径不符。
//! 这里直接断言配置为开启，防止该交互再次被关掉。

#[test]
fn tray_shows_its_menu_on_left_click() {
    let config: serde_json::Value = serde_json::from_str(include_str!("../tauri.conf.json"))
        .expect("tauri.conf.json 应为合法 JSON");
    let tray = &config["app"]["trayIcon"];
    assert_eq!(
        tray["showMenuOnLeftClick"].as_bool(),
        Some(true),
        "托盘左键必须展开同一份状态菜单"
    );
    // 托盘资源仍由配置声明，应用退出后不残留（AC-12）。
    assert_eq!(tray["id"].as_str(), Some("opencodex-main"));
}
