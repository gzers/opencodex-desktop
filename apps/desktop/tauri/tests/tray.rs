//! 托盘状态契约测试；不创建原生托盘，不读取真实用户数据根。

use opencodex_desktop_lib::commands::tray::tray_state_from_parts;
use opencodex_desktop_lib::modules::status::{StatusCollector, VirtualStatusSource};
use serde_json::json;

fn collector(payload: serde_json::Value) -> StatusCollector<VirtualStatusSource> {
    let mut collector = StatusCollector::new(VirtualStatusSource::new(Ok(payload)));
    collector
        .refresh()
        .expect("fixture status should map without error");
    collector
}

#[test]
fn tray_state_reports_running_address_and_action_gates() {
    let collector = collector(json!({
        "status": "running",
        "ready": true,
        "port": 10100,
        "pid": "39421",
        "dataRoot": "/fixtures/data-root",
        "startup": {"protection": "none", "rebootSafe": false}
    }));
    let dto = tray_state_from_parts(&collector, std::path::Path::new("/fixtures/data-root"))
        .expect("tray dto");
    assert_eq!(dto.runtime_label, "运行中");
    assert_eq!(dto.address.as_deref(), Some("本地 · 127.0.0.1:10100"));
    assert!(!dto.can_start);
    assert!(dto.can_stop);
    assert!(dto.can_restart);
}

#[test]
fn tray_state_hides_address_until_live_running_port_exists() {
    let collector = collector(json!({
        "status": "stopped",
        "ready": false,
        "port": 10100,
        "dataRoot": "/fixtures/data-root",
        "startup": {"protection": "none", "rebootSafe": false}
    }));
    let dto = tray_state_from_parts(&collector, std::path::Path::new("/fixtures/data-root"))
        .expect("tray dto");
    assert_eq!(dto.runtime_label, "未运行");
    assert_eq!(dto.address, None);
    assert!(dto.can_start);
    assert!(!dto.can_stop);
    assert!(!dto.can_restart);
}

#[test]
fn tray_state_allows_start_at_risk_without_writing_the_risk_verdict_to_the_tray() {
    // 官方真实形状：代理未运行 + startup at-risk（端口只是配置值，不代表在监听）。
    let collector = collector(json!({
        "proxy": {"running": false},
        "listen": {"port": 10100},
        "dataRoot": "/fixtures/data-root",
        "startup": {"status": "at-risk", "protection": "none", "rebootSafe": false},
        "versionSkew": {"cliVersion": "2.50.0"}
    }));
    let dto = tray_state_from_parts(&collector, std::path::Path::new("/fixtures/data-root"))
        .expect("tray dto");
    // 2026-09-24 用户决策：托盘只讲进程事实，风险结论与说明句都不写进托盘。
    assert_eq!(dto.runtime_label, "未运行");
    assert_eq!(dto.address, None);
    // 回归：此前这里 can_start=false，托盘/应用菜单把「启动 OpenCodex」置灰。
    assert!(dto.can_start, "存在风险时代理没在跑，必须允许启动");
    assert!(!dto.can_stop);
    assert!(!dto.can_restart);
}

#[test]
fn tray_state_uses_frozen_labels_for_failure_states() {
    let collector = collector(json!({
        "status": "unreachable",
        "ready": false,
        "port": 10100,
        "dataRoot": "/fixtures/data-root",
        "startup": {"protection": "none", "rebootSafe": false}
    }));
    let dto = tray_state_from_parts(&collector, std::path::Path::new("/fixtures/data-root"))
        .expect("tray dto");
    assert_eq!(dto.runtime_label, "不可达");
    assert_eq!(dto.address, None);
    assert!(dto.can_start);
    assert!(dto.can_stop);
    assert!(dto.can_restart);
}
