use std::fs;
use std::path::PathBuf;

use opencodex_desktop_lib::commands::logs::read_logs_with_root;
use opencodex_desktop_lib::modules::data_root::{self, DataRootRuntimeConfig, OpenCodexHomeMode};
use opencodex_desktop_lib::types::logs::LogKind;

fn temp_data_root(name: &str) -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().expect("create temp root");
    let logs_dir = root.path().join("logs");
    fs::create_dir_all(&logs_dir).expect("create logs dir");
    let path = PathBuf::from(name);
    (root, logs_dir.join(path))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn read_recent_returns_200_tail_and_sanitizes_frozen_rules() {
    let (_root, path) = temp_data_root("app.log");
    let mut payload = String::new();
    for index in 0..220 {
        payload.push_str(&format!("line-{index:03} safe\n"));
    }
    payload.push_str("s3://private-bucket/object\n");
    payload.push_str("https://user:token@example.com/v1?api_key=secret\n");
    fs::write(&path, payload).expect("write fixture log");

    let result = read_logs_with_root(LogKind::App, _root.path()).expect("read logs");
    assert_eq!(result.lines.len(), 200);
    assert_eq!(
        result.lines.first().map(String::as_str),
        Some("line-022 safe")
    );
    assert_eq!(
        result.lines.last().map(String::as_str),
        Some("https://example.com/v1")
    );
    assert!(result.lines.iter().any(|line| line.contains("[REDACTED]")));
    assert!(!result
        .lines
        .iter()
        .any(|line| line.contains("private-bucket")));
    assert!(!result
        .lines
        .iter()
        .any(|line| line.contains("api_key=secret")));
    assert!(result.truncated);
    assert!(!result.file_missing);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn missing_log_file_is_explicit_not_an_error() {
    let (root, _path) = temp_data_root("agent.log");
    let result = read_logs_with_root(LogKind::Agent, root.path()).expect("read missing log");
    assert!(result.lines.is_empty());
    assert!(result.file_missing);
    assert!(!result.truncated);
}

// 回归：状态机在「存在风险 / 启动失败」时会要 `agent.log`，而它目前没有写入方。
// 若此时直接显示 0 行，就会出现「磁盘上 app.log 有内容、界面却是空的」。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn missing_agent_log_falls_back_to_the_app_log() {
    let (root, _path) = temp_data_root("agent.log");
    fs::write(
        root.path().join("logs").join("app.log"),
        "2026-09-21T06:01:46.438Z [manager] start: start confirmed\n",
    )
    .expect("write app log");

    let result = read_logs_with_root(LogKind::Agent, root.path()).expect("read logs");
    assert_eq!(
        result.lines,
        vec!["2026-09-21T06:01:46.438Z [manager] start: start confirmed".to_string()]
    );
    assert!(!result.file_missing);
    assert_eq!(result.fallback_file.as_deref(), Some("app.log"));
}

// 两个文件都不存在时仍然如实报「没有日志文件」，不拿空内容冒充成功读取。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fallback_keeps_missing_state_when_no_log_file_exists() {
    let (root, _path) = temp_data_root("agent.log");
    let result = read_logs_with_root(LogKind::Agent, root.path()).expect("read logs");
    assert!(result.file_missing);
    assert!(result.lines.is_empty());
    assert_eq!(result.fallback_file, None);
}

// 回归：审计写入器把 `audit.log` 写在**活跃数据根**的根目录（`AuditStore::new(active_data_root)`），
// 不是 `<数据根>/logs/` 分区。此前「调用日志」分类会因此永远读不到内容。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn call_log_is_read_from_the_active_data_root_not_the_logs_partition() {
    let temp = tempfile::tempdir().expect("create temp root");
    let manager = temp.path().join("manager");
    let active = temp.path().join("active-home");
    fs::create_dir_all(&active).expect("create active root");
    data_root::initialize(&manager).expect("initialize data root");

    let mut config: DataRootRuntimeConfig =
        data_root::load_runtime_config(&manager).expect("load runtime config");
    config.active_data_root = active.clone();
    config.opencodex_home_mode = OpenCodexHomeMode::External;
    config.opencodex_home_path = Some(active.clone());
    data_root::save_runtime_config(&manager, &config).expect("save runtime config");

    fs::write(
        active.join("audit.log"),
        "{\"call\":\"status\",\"result\":\"ok\"}\n",
    )
    .expect("write audit log");
    // 分区里的同名文件是干扰项：分类日志必须读到活跃数据根那一份。
    fs::create_dir_all(manager.join("logs")).expect("create logs dir");
    fs::write(manager.join("logs/audit.log"), "WRONG-SOURCE\n").expect("write decoy");

    let result = read_logs_with_root(LogKind::Audit, &manager).expect("read call log");
    assert_eq!(
        result.lines,
        vec!["{\"call\":\"status\",\"result\":\"ok\"}".to_string()]
    );
    assert!(!result.file_missing);
    assert_eq!(result.fallback_file, None);
}

// 分类日志缺失时如实报「不存在」，不回退到 app.log（否则会把应用日志冒充成调用日志）。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn missing_call_log_does_not_fall_back_to_application_log() {
    let (root, _path) = temp_data_root("app.log");
    fs::write(
        root.path().join("logs").join("app.log"),
        "2026-09-23T00:00:00Z [manager] start\n",
    )
    .expect("write app log");

    let result = read_logs_with_root(LogKind::Audit, root.path()).expect("read call log");
    assert!(result.file_missing);
    assert!(result.lines.is_empty());
    assert_eq!(result.fallback_file, None);
}
