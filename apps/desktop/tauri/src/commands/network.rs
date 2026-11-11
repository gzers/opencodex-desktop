//! One explicit diagnostic, two independent fixed identity-checked destinations.
//! No caller URL, version update, model request or configuration write is made.
use crate::{errors::AppResult, modules::preferences::ProxyPolicy};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeTarget {
    Github,
    Npm,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeState {
    Ready,
    Timeout,
    RateLimited,
    HttpError,
    Network,
    IdentityMismatch,
    TooLarge,
    InvalidProxy,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResult {
    pub target: ProbeTarget,
    pub state: ProbeState,
    pub http_status: Option<u16>,
    pub retry_after_seconds: Option<u64>,
    pub checked_at: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkProbeDto {
    pub ok: bool,
    pub detail: String,
    pub results: Vec<ProbeResult>,
}
#[tauri::command]
pub async fn check_network_proxy(app: tauri::AppHandle) -> AppResult<NetworkProbeDto> {
    let client = tauri::async_runtime::spawn_blocking(move || {
        let (policy, no_proxy) = crate::modules::preferences::http_options_for_app(&app);
        crate::modules::network_defaults::http_client(
            policy,
            &no_proxy,
            crate::modules::network_defaults::request_timeout(),
        )
    })
    .await
    .map_err(|_| crate::errors::AppError::NotConfigured)?;
    Ok(probe_client(client).await)
}
fn endpoints() -> [(ProbeTarget, String); 2] {
    [
        (
            ProbeTarget::Github,
            format!(
                "https://api.github.com/repos/{}",
                crate::modules::runtime_defaults::release_notes().manager_repository
            ),
        ),
        (
            ProbeTarget::Npm,
            "https://registry.npmjs.org/@bitkyc08%2fopencodex/latest".into(),
        ),
    ]
}
fn result(
    target: ProbeTarget,
    state: ProbeState,
    http_status: Option<u16>,
    retry_after_seconds: Option<u64>,
) -> ProbeResult {
    ProbeResult {
        target,
        state,
        http_status,
        retry_after_seconds,
        checked_at: chrono::Utc::now().to_rfc3339(),
    }
}
fn identity(target: ProbeTarget, bytes: &[u8]) -> bool {
    let Ok(v) = crate::modules::model_config::schema_v2::parse_strict_json_v2(bytes) else {
        return false;
    };
    let config = crate::modules::runtime_defaults::release_notes();
    match target {
        ProbeTarget::Github => {
            v["full_name"] == config.manager_repository
                && v["html_url"] == format!("https://github.com/{}", config.manager_repository)
        }
        ProbeTarget::Npm => {
            v["name"] == crate::modules::runtime::OFFICIAL_PACKAGE
                && v["version"]
                    .as_str()
                    .is_some_and(crate::modules::update::handoff::valid_version)
                && v["repository"]["url"]
                    == format!("git+https://github.com/{}.git", config.panel_repository)
        }
    }
}
async fn one(client: &reqwest::Client, target: ProbeTarget, url: &str) -> ProbeResult {
    let work = async {
        let mut response = match client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return result(
                    target,
                    if e.is_timeout() {
                        ProbeState::Timeout
                    } else {
                        ProbeState::Network
                    },
                    None,
                    None,
                )
            }
        };
        let status = response.status().as_u16();
        let retry = response
            .headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|v| *v <= 604800);
        let limited = status == 429
            || status == 403
                && response
                    .headers()
                    .get("x-ratelimit-remaining")
                    .is_some_and(|v| v == "0");
        if limited {
            return result(target, ProbeState::RateLimited, Some(status), retry);
        }
        if response.status().is_redirection() {
            return result(target, ProbeState::IdentityMismatch, Some(status), None);
        }
        if !response.status().is_success() {
            return result(target, ProbeState::HttpError, Some(status), None);
        }
        let limit = crate::modules::runtime_defaults::release_notes().response_max_bytes;
        if response.content_length().is_some_and(|n| n > limit) {
            return result(target, ProbeState::TooLarge, Some(status), None);
        }
        let mut bytes = Vec::new();
        loop {
            match response.chunk().await {
                Ok(Some(chunk)) => {
                    if bytes.len() as u64 + chunk.len() as u64 > limit {
                        return result(target, ProbeState::TooLarge, Some(status), None);
                    }
                    bytes.extend_from_slice(&chunk)
                }
                Ok(None) => break,
                Err(e) => {
                    return result(
                        target,
                        if e.is_timeout() {
                            ProbeState::Timeout
                        } else {
                            ProbeState::Network
                        },
                        Some(status),
                        None,
                    )
                }
            }
        }
        result(
            target,
            if identity(target, &bytes) {
                ProbeState::Ready
            } else {
                ProbeState::IdentityMismatch
            },
            Some(status),
            None,
        )
    };
    tokio::time::timeout(crate::modules::network_defaults::request_timeout(), work)
        .await
        .unwrap_or_else(|_| result(target, ProbeState::Timeout, None, None))
}
async fn probe_client(client: Result<reqwest::Client, reqwest::Error>) -> NetworkProbeDto {
    probe_pair(client, endpoints()).await
}
async fn probe_pair(
    client: Result<reqwest::Client, reqwest::Error>,
    pair: [(ProbeTarget, String); 2],
) -> NetworkProbeDto {
    let [(a, url_a), (b, url_b)] = pair;
    let results = match client {
        Ok(client) => {
            let (a, b) = tokio::join!(one(&client, a, &url_a), one(&client, b, &url_b));
            vec![a, b]
        }
        Err(_) => vec![
            result(a, ProbeState::InvalidProxy, None, None),
            result(b, ProbeState::InvalidProxy, None, None),
        ],
    };
    let ok = results.iter().all(|r| r.state == ProbeState::Ready);
    NetworkProbeDto {
        ok,
        detail: if ok {
            "GitHub 与 npm 连接及来源核验通过。"
        } else {
            "部分连接未通过，请分别查看结果。"
        }
        .into(),
        results,
    }
}
/// Explicit synchronous adapter for isolated diagnostics/tests, never a global
/// environment discovery or retry loop. Production IPC uses the async command.
pub fn probe(policy: ProxyPolicy) -> NetworkProbeDto {
    let client = crate::modules::network_defaults::http_client(
        policy,
        "",
        crate::modules::network_defaults::request_timeout(),
    );
    match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime.block_on(probe_client(client)),
        Err(_) => NetworkProbeDto {
            ok: false,
            detail: "无法启动连接检查。".into(),
            results: vec![],
        },
    }
}
#[cfg(test)]
#[path = "network_tests.rs"]
mod tests;
