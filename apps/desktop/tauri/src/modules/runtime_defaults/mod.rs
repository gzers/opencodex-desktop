//! 版本固化运行策略（H-01/H-04/H-05/H-08/H-09/H-27）。
//!
//! 单一事实源是构建期嵌入的 `config/runtime.defaults.json`；本模块把它解析为类型化访问器，
//! 供进程超时、状态轮询、同步超时、备份保留与日志读取等消费者读取，避免各处各写常量。
//! 未改变任何已冻结行为数值：文件里的取值与此前的常量一一对应。

use std::sync::OnceLock;
use std::time::Duration;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct RuntimeDefaults {
    process: ProcessDefaults,
    status: StatusDefaults,
    sync: SyncDefaults,
    retention: RetentionDefaults,
    diagnostics: DiagnosticsDefaults,
    install: InstallDefaults,
    extensions: ExtensionsDefaults,
    updates: UpdatesDefaults,
}

#[derive(Debug, Clone, Deserialize)]
struct ProcessDefaults {
    start_seconds: u64,
    stop_seconds: u64,
    restart_seconds: u64,
    runner_start_seconds: u64,
    reap_grace_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct StatusDefaults {
    foreground_ms: u64,
    foreground_backoff_ms: u64,
    background_ms: u64,
    background_backoff_ms: u64,
    debounce_ms: u64,
    sample_ms: u64,
    backoff_ms: u64,
    max_events_per_second: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct SyncDefaults {
    connect_seconds: u64,
    item_seconds: u64,
    total_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct RetentionDefaults {
    backup_max_age_days: i64,
}

#[derive(Debug, Clone, Deserialize)]
struct DiagnosticsDefaults {
    version_probe_seconds: u64,
    doctor_seconds: u64,
    shim_seconds: u64,
    recent_log_lines: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct UpdatesDefaults {
    desktop: DesktopUpdates,
}

#[derive(Debug, Clone, Deserialize)]
struct DesktopUpdates {
    channels: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ExtensionsDefaults {
    default_sync_method: String,
    default_client_enablement: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct InstallDefaults {
    default_tag: String,
    idle_notice_seconds: u64,
    post_install_probe_seconds: u64,
}

fn defaults() -> &'static RuntimeDefaults {
    static DEFAULTS: OnceLock<RuntimeDefaults> = OnceLock::new();
    DEFAULTS.get_or_init(|| {
        let raw = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/config/runtime.defaults.json"
        ));
        serde_json::from_str(raw).expect("frozen runtime defaults must parse")
    })
}

// —— 进程生命周期（H-04）：start / restart / stop 窗口 ——
pub fn process_start() -> Duration {
    Duration::from_secs(defaults().process.start_seconds)
}
pub fn process_stop() -> Duration {
    Duration::from_secs(defaults().process.stop_seconds)
}
pub fn process_restart() -> Duration {
    Duration::from_secs(defaults().process.restart_seconds)
}
/// 实际子进程等待窗口（H-04）：runner 对 Start/Restart 取 30 秒，与编排动作窗口不同义。
pub fn process_runner_start() -> Duration {
    Duration::from_secs(defaults().process.runner_start_seconds)
}
/// 发出温和信号后的回收宽限。
pub fn process_reap_grace() -> Duration {
    Duration::from_secs(defaults().process.reap_grace_seconds)
}

// —— 状态轮询（H-08）——
pub fn status_foreground() -> Duration {
    Duration::from_millis(defaults().status.foreground_ms)
}
pub fn status_connection_foreground() -> Duration {
    Duration::from_millis(defaults().status.foreground_backoff_ms)
}
pub fn status_background() -> Duration {
    Duration::from_millis(defaults().status.background_ms)
}
pub fn status_connection_background() -> Duration {
    Duration::from_millis(defaults().status.background_backoff_ms)
}
pub fn status_debounce() -> Duration {
    Duration::from_millis(defaults().status.debounce_ms)
}
pub fn status_sample_timeout() -> Duration {
    Duration::from_millis(defaults().status.sample_ms)
}
pub fn status_max_backoff() -> Duration {
    Duration::from_millis(defaults().status.backoff_ms)
}
pub fn status_max_events_per_second() -> u32 {
    defaults().status.max_events_per_second
}

// —— WebDAV 同步（H-09）——
pub fn sync_connect() -> Duration {
    Duration::from_secs(defaults().sync.connect_seconds)
}
pub fn sync_item() -> Duration {
    Duration::from_secs(defaults().sync.item_seconds)
}
pub fn sync_total() -> Duration {
    Duration::from_secs(defaults().sync.total_seconds)
}

// —— 备份保留（H-05）——
pub fn backup_max_age_days() -> i64 {
    defaults().retention.backup_max_age_days
}

// —— 诊断命令窗口（H-23）——
pub fn recent_log_lines() -> usize {
    defaults().diagnostics.recent_log_lines
}

pub fn version_probe_timeout() -> Duration {
    Duration::from_secs(defaults().diagnostics.version_probe_seconds)
}
pub fn doctor_timeout() -> Duration {
    Duration::from_secs(defaults().diagnostics.doctor_seconds)
}
pub fn shim_timeout() -> Duration {
    Duration::from_secs(defaults().diagnostics.shim_seconds)
}

// —— 安装（H-10）——
pub fn install_default_tag() -> &'static str {
    defaults().install.default_tag.as_str()
}
pub fn install_idle_notice() -> Duration {
    Duration::from_secs(defaults().install.idle_notice_seconds)
}
pub fn install_probe_timeout() -> Duration {
    Duration::from_secs(defaults().install.post_install_probe_seconds)
}

// —— 扩展初始策略（H-29）——
pub fn extensions_default_sync_method() -> &'static str {
    defaults().extensions.default_sync_method.as_str()
}
pub fn extensions_default_client_enablement() -> bool {
    defaults().extensions.default_client_enablement
}

// -- 桌面更新端点（U-05） --
/// 返回指定通道的端点；未知通道回退 stable，与领域层 parse 的收敛一致。
pub fn desktop_update_endpoint(channel: &str) -> &'static str {
    let channels = &defaults().updates.desktop.channels;
    channels
        .get(channel)
        .or_else(|| channels.get("stable"))
        .map(String::as_str)
        .unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_values_match_declared_contract() {
        assert_eq!(process_start(), Duration::from_secs(20));
        assert_eq!(process_runner_start(), Duration::from_secs(30));
        assert_eq!(process_reap_grace(), Duration::from_secs(5));
        assert_eq!(process_stop(), Duration::from_secs(10));
        assert_eq!(process_restart(), Duration::from_secs(30));
        assert_eq!(status_foreground(), Duration::from_secs(3));
        assert_eq!(status_connection_foreground(), Duration::from_secs(15));
        assert_eq!(status_background(), Duration::from_secs(10));
        assert_eq!(status_connection_background(), Duration::from_secs(30));
        assert_eq!(status_debounce(), Duration::from_secs(1));
        assert_eq!(status_sample_timeout(), Duration::from_secs(5));
        assert_eq!(status_max_backoff(), Duration::from_secs(60));
        assert_eq!(status_max_events_per_second(), 2);
        assert_eq!(sync_connect(), Duration::from_secs(15));
        assert_eq!(sync_item(), Duration::from_secs(600));
        assert_eq!(sync_total(), Duration::from_secs(1800));
        assert_eq!(backup_max_age_days(), 30);
        assert_eq!(recent_log_lines(), 200);
        assert_eq!(version_probe_timeout(), Duration::from_secs(2));
        assert_eq!(doctor_timeout(), Duration::from_secs(5));
        assert_eq!(shim_timeout(), Duration::from_secs(30));
        assert_eq!(install_default_tag(), "latest");
        assert_eq!(install_idle_notice(), Duration::from_secs(120));
        assert_eq!(install_probe_timeout(), Duration::from_secs(30));
        assert_eq!(extensions_default_sync_method(), "symlink");
        assert!(extensions_default_client_enablement());
        assert!(desktop_update_endpoint("stable").contains("stable"));
        assert!(desktop_update_endpoint("beta").contains("beta"));
        assert_eq!(
            desktop_update_endpoint("nightly"),
            desktop_update_endpoint("stable")
        );
    }
}
