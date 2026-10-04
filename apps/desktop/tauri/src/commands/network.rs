//! 网络连通性检查命令（U-05「检查连接」）。
//!
//! 用当前代理偏好对一个固定、受控的探测地址发一次带超时请求，只回报成功/失败与原因，
//! **不改写任何本地状态**。TLS 证书校验不可关闭（由 rustls 与系统根证书保证）。

use serde::{Deserialize, Serialize};

use crate::errors::AppResult;
use crate::modules::preferences::ProxyPolicy;

/// 连通性检查结果；`detail` 已脱敏，不含代理地址之外的任何凭据。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkProbeDto {
    pub ok: bool,
    pub detail: String,
}

#[tauri::command]
pub async fn check_network_proxy(app: tauri::AppHandle) -> AppResult<NetworkProbeDto> {
    let policy = crate::modules::preferences::proxy_policy_for_app(&app);
    let result = tauri::async_runtime::spawn_blocking(move || probe(policy))
        .await
        .map_err(|_| crate::errors::AppError::NotConfigured)?;
    Ok(result)
}

/// 执行一次受控探测；网络行为约束（超时、TLS）来自固化网络策略。
pub fn probe(policy: ProxyPolicy) -> NetworkProbeDto {
    let timeout = crate::modules::network_defaults::request_timeout();
    let mut builder = reqwest::Client::builder()
        .timeout(timeout)
        .connect_timeout(crate::modules::network_defaults::connect_timeout())
        .user_agent(crate::modules::network_defaults::user_agent());
    match &policy {
        ProxyPolicy::None => {
            // 明确禁用代理，避免环境变量外泄到「无代理」模式。
            builder = builder.no_proxy();
        }
        ProxyPolicy::System => {
            // 「自动」沿用平台/环境默认代理（reqwest 默认行为）。
        }
        ProxyPolicy::Manual(url) => match reqwest::Proxy::all(url.as_str()) {
            Ok(proxy) => builder = builder.proxy(proxy),
            Err(_) => {
                return NetworkProbeDto {
                    ok: false,
                    detail: "代理地址无效，请检查主机名与端口。".to_string(),
                }
            }
        },
    }
    let client = match builder.build() {
        Ok(client) => client,
        Err(_) => {
            return NetworkProbeDto {
                ok: false,
                detail: "无法构建网络客户端；请检查代理配置。".to_string(),
            }
        }
    };
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(_) => {
            return NetworkProbeDto {
                ok: false,
                detail: "无法启动连通性检查。".to_string(),
            }
        }
    };
    let url = crate::modules::network_defaults::probe_url();
    runtime.block_on(async {
        match client.get(url).send().await {
            Ok(response) if response.status().is_success() => NetworkProbeDto {
                ok: true,
                detail: format!("连接正常（HTTP {}）。", response.status().as_u16()),
            },
            Ok(response) => NetworkProbeDto {
                ok: false,
                detail: format!("连接可达但返回 HTTP {}。", response.status().as_u16()),
            },
            Err(_) => NetworkProbeDto {
                ok: false,
                detail: "连接失败；请检查网络或代理设置。".to_string(),
            },
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_manual_proxy_is_reported_without_panicking() {
        let result = probe(ProxyPolicy::Manual("http://".to_string()));
        assert!(!result.ok);
        assert!(!result.detail.is_empty());
    }

    /// 真实出网运行检查（U-05）：默认忽略，按需 `cargo test -- --ignored` 运行，
    /// 用生产 HTTP 客户端走一次带超时的受控探测，验证 TLS 与连通性链路。
    #[test]
    #[ignore = "hits the network; run explicitly"]
    fn live_no_proxy_probe_reaches_controlled_url() {
        let result = probe(ProxyPolicy::None);
        assert!(result.ok, "probe failed: {}", result.detail);
    }
}
