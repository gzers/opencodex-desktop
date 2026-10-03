//! 主窗口关闭行为契约。
//!
//! 回归背景：`keep_proxy_on_close`（「关闭窗口后保持代理运行」，默认开启）此前没有任何消费方，
//! 点窗口关闭按钮会直接退出整个桌面壳，托盘图标一并消失，与设置里的说明不符。

use opencodex_desktop_lib::{close_action, dock_visibility, CloseAction, DockVisibility};

#[test]
fn window_close_only_hides_the_shell_when_keep_proxy_on_close_is_enabled() {
    assert_eq!(close_action(Some(true)), CloseAction::Hide);
    assert_eq!(close_action(Some(false)), CloseAction::Exit);
}

#[test]
fn unreadable_preferences_never_quit_the_shell() {
    // 读取失败（None）时按默认处理：宁可留在托盘，也不要把用户应用悄悄退掉。
    assert_eq!(close_action(None), CloseAction::Hide);
}

#[test]
fn default_preference_is_keep_proxy_on_close() {
    assert!(
        opencodex_desktop_lib::modules::preferences::Preferences::default().keep_proxy_on_close
    );
}

#[test]
fn lifecycle_notifications_preference_is_consumed() {
    use opencodex_desktop_lib::modules::preferences::{Preferences, PreferencesStore};
    let root = tempfile::tempdir().expect("tempdir");
    // 默认开启。
    assert!(opencodex_desktop_lib::commands::lifecycle_notifications_enabled(root.path()));
    PreferencesStore::new(root.path())
        .save(&Preferences {
            lifecycle_notifications: false,
            ..Default::default()
        })
        .expect("save");
    assert!(!opencodex_desktop_lib::commands::lifecycle_notifications_enabled(root.path()));
}

#[test]
fn notification_publisher_is_absent_when_lifecycle_notifications_are_off() {
    use opencodex_desktop_lib::commands::publisher_if_enabled;
    use std::sync::{Arc, Mutex};

    let root = tempfile::tempdir().expect("tempdir");
    let store = Arc::new(Mutex::new(
        opencodex_desktop_lib::modules::notifications::NotificationStore::new(),
    ));
    assert!(publisher_if_enabled(false, &store, root.path()).is_none());
    assert!(publisher_if_enabled(true, &store, root.path()).is_some());
}

// 关闭窗口只隐藏时：窗口不再可见 → Dock 图标应消失（只留托盘）；窗口回来 → Dock 图标恢复。
// 这里断言纯函数；真实 Dock 外观属 macOS 原生行为，需真机验收。
#[test]
fn hidden_window_releases_the_dock_icon_and_showing_it_restores_it() {
    assert_eq!(dock_visibility(true), DockVisibility::Regular);
    assert_eq!(dock_visibility(false), DockVisibility::Accessory);
}
