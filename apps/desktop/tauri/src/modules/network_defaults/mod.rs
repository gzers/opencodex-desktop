//! 网络请求行为策略（U-05b）：构建期嵌入、只读、暂不对用户开放。
//!
//! 单一事实源是 `config/network.defaults.json`，由 build.rs 校验存在与合法性；本模块把它
//! 解析为类型化访问器。用户偏好只决定「用不用代理、用哪个代理」，超时、重试、UA 与 TLS
//! 约束等安全行为固定在此，不随用户配置放宽。

use std::sync::OnceLock;
use std::time::Duration;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct NetworkDefaults {
    timeout: Timeouts,
    retry: Retry,
    user_agent: String,
    probe_url: String,
    tls_verify: bool,
    allow_proxy: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct Timeouts {
    connect_seconds: u64,
    request_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct Retry {
    max_attempts: u32,
    backoff_seconds: u64,
}

fn defaults() -> &'static NetworkDefaults {
    static DEFAULTS: OnceLock<NetworkDefaults> = OnceLock::new();
    DEFAULTS.get_or_init(|| {
        let raw = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/config/network.defaults.json"
        ));
        serde_json::from_str(raw).expect("frozen network defaults must parse")
    })
}

pub fn connect_timeout() -> Duration {
    Duration::from_secs(defaults().timeout.connect_seconds)
}
pub fn request_timeout() -> Duration {
    Duration::from_secs(defaults().timeout.request_seconds)
}
pub fn max_attempts() -> u32 {
    defaults().retry.max_attempts
}
pub fn retry_backoff() -> Duration {
    Duration::from_secs(defaults().retry.backoff_seconds)
}
pub fn user_agent() -> &'static str {
    defaults().user_agent.as_str()
}
/// 连通性探测的固定、受控地址（U-05「检查连接」使用；不改写任何状态）。
pub fn probe_url() -> &'static str {
    defaults().probe_url.as_str()
}
/// TLS 证书校验永远为 true；此处只暴露事实供诊断展示，不提供关闭入口。
pub fn tls_verify() -> bool {
    defaults().tls_verify
}
/// 是否允许使用代理：安全策略开关。
pub fn proxy_allowed() -> bool {
    defaults().allow_proxy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_network_defaults_match_contract() {
        assert_eq!(connect_timeout(), Duration::from_secs(15));
        assert_eq!(request_timeout(), Duration::from_secs(30));
        assert_eq!(max_attempts(), 2);
        assert_eq!(retry_backoff(), Duration::from_secs(1));
        assert!(user_agent().contains("OpenCodeX"));
        assert!(probe_url().starts_with("https://"));
        assert!(tls_verify());
        assert!(proxy_allowed());
    }
}
