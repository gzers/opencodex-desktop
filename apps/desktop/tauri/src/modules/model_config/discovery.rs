//! Unsaved connection discovery. No provider registration or generation request.
use super::{agent, schema::*};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeSet, time::Duration};
use zeroize::Zeroizing;
#[path = "discovery_paths.rs"]
mod paths;

const PURPOSE: &str = "model_channel";
fn entry(reference: &str) -> ModelResult<keyring::Entry> {
    require_uuid(reference)?;
    keyring::Entry::new(
        crate::infrastructure::keychain::keychain_service_name(),
        &format!("{PURPOSE}:{reference}"),
    )
    .map_err(|_| invalid("credential_store", "系统凭据存储不可用"))
}
pub fn credential(reference: &str) -> ModelResult<Zeroizing<String>> {
    entry(reference)?
        .get_password()
        .map(Zeroizing::new)
        .map_err(|_| invalid("credential_store", "渠道密钥尚未配置或无法读取，请重新输入"))
}
pub(super) fn import_credential(reference: &str, value: &str) -> ModelResult<()> {
    if value.is_empty() || value.len() > 8192 || value.contains(['\r', '\n']) {
        return Err(invalid("credential", "导入密钥格式无效"));
    }
    let entry = entry(reference)?;
    match entry.get_password() {
        Ok(current) if current == value => return Ok(()),
        Ok(_) => {
            return Err(invalid(
                "credential_collision",
                "新凭据引用已有不同内容，未替换",
            ))
        }
        Err(keyring::Error::NoEntry) => {}
        Err(_) => return Err(invalid("credential_store", "系统凭据存储不可用")),
    }
    entry
        .set_password(value)
        .map_err(|_| invalid("credential_store", "导入密钥未安全保存"))?;
    if entry
        .get_password()
        .map_err(|_| invalid("credential_readback", "导入密钥尚未读回确认"))?
        != value
    {
        return Err(invalid("credential_readback", "导入密钥读回不一致"));
    }
    Ok(())
}
pub fn forget(reference: &str) -> ModelResult<()> {
    match entry(reference)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err(invalid("credential_store", "系统凭据未删除，请稍后重试")),
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiscoveryRequest {
    pub protocol: String,
    pub url: String,
    pub api_key: Option<String>,
    pub credential_ref: Option<String>,
    pub allow_private_network: bool,
    /// 渠道身份（官方 provider id）。给了才允许回落到 config.json 里该渠道已配置的 Key。
    #[serde(default)]
    pub provider_id: Option<String>,
    /// 官方配置根（不含 config.json 文件名）。由命令层注入，**不从界面收路径**。
    #[serde(skip)]
    pub home: Option<std::path::PathBuf>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryResult {
    pub models: Vec<DiscoveredModel>,
    pub credential_ref: Option<String>,
    pub elapsed_ms: u64,
}
pub async fn discover(request: DiscoveryRequest) -> ModelResult<DiscoveryResult> {
    discover_inner(request, true).await
}
pub async fn preview(request: DiscoveryRequest) -> ModelResult<DiscoveryResult> {
    discover_inner(request, false).await
}
pub(super) async fn readonly(request: DiscoveryRequest) -> ModelResult<DiscoveryResult> {
    discover_inner(request, false).await
}
async fn discover_inner(
    mut request: DiscoveryRequest,
    persist_credential: bool,
) -> ModelResult<DiscoveryResult> {
    let started = std::time::Instant::now();
    if ![
        "openai-chat",
        "openai-responses",
        "anthropic",
        "google",
        "ollama-native",
    ]
    .contains(&request.protocol.as_str())
    {
        return Err(invalid(
            "discovery_protocol",
            "此协议需使用专用连接方式，未发送请求",
        ));
    }
    let url = reqwest::Url::parse(&request.url).map_err(|_| invalid("node_url", "连接地址无效"))?;
    if !["http", "https"].contains(&url.scheme())
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(invalid("node_url", "服务地址不能包含凭据/查询参数/片段"));
    }
    let host = url
        .host_str()
        .ok_or_else(|| invalid("node_url", "地址缺少主机"))?;
    let port = url
        .port_or_known_default()
        .ok_or_else(|| invalid("node_url", "地址端口无效"))?;
    let addresses: Vec<_> = tokio::time::timeout(
        Duration::from_secs(5),
        tokio::net::lookup_host((host, port)),
    )
    .await
    .map_err(|_| invalid("discovery_dns", "地址解析超时"))?
    .map_err(|_| invalid("discovery_dns", "地址解析失败"))?
    .collect();
    if addresses.is_empty()
        || !request.allow_private_network && addresses.iter().any(|a| !agent::public_ip(a.ip()))
    {
        return Err(invalid(
            "discovery_private",
            "地址指向私有网络；如为可信本机服务，请显式允许私有网络",
        ));
    }
    let provided = request.api_key.take().map(Zeroizing::new);
    let stored = if provided.is_none() {
        request
            .credential_ref
            .as_ref()
            .map(|r| credential(r))
            .transpose()?
    } else {
        None
    };
    // 管理器的 credential_ref 在「读取现有渠道」导入时是 null，用户也不该为每次检测重填 Key。
    // 因此这里回落到官方 config.json 里该渠道已配置的 Key，但必须同源且协议一致。
    let from_official = if provided.is_none() && stored.is_none() {
        match (request.provider_id.as_deref(), request.home.as_deref()) {
            (Some(provider_id), Some(home)) => {
                let resolved = super::discovery_config::resolve(
                    home,
                    provider_id,
                    &request.url,
                    &request.protocol,
                );
                if resolved.missing {
                    return Err(invalid(
                        "credential_unconfirmed",
                        "该节点没有可确认的凭据；请先确认该渠道地址与协议，或在本页填写 Key 后再检测",
                    ));
                }
                resolved.secret
            }
            _ => None,
        }
    } else {
        None
    };
    let secret = provided
        .as_ref()
        .or(stored.as_ref())
        .or(from_official.as_ref())
        .map(|s| s.as_str());
    if secret.is_some_and(|s| s.len() > 8192 || s.contains(['\r', '\n'])) {
        return Err(invalid("credential", "密钥格式无效"));
    }
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(20))
        .resolve_to_addrs(host, &addresses)
        .build()
        .map_err(|_| invalid("discovery_client", "发现连接未就绪"))?;
    let (mut target, response) = read_first_page(&client, &url, &request.protocol, secret).await?;
    let mut first_page = Some(response);
    let mut next: Option<String> = None;
    let mut pages = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut models = vec![];
    for page in 0..20 {
        if let Some(value) = &next {
            if value.is_empty() || value.len() > 4096 || !pages.insert(value.clone()) {
                return Err(invalid(
                    "discovery_pagination",
                    "分页游标重复或无效，未报告完整成功",
                ));
            }
            target.set_query(None);
            target.query_pairs_mut().append_pair(
                if request.protocol == "google" {
                    "pageToken"
                } else if request.protocol == "anthropic" {
                    "after_id"
                } else {
                    "after"
                },
                value,
            );
        }
        let response = match first_page.take() {
            Some(response) => response,
            None => read_page(&client, &target, &request.protocol, secret).await?,
        };
        if response.get("error").is_some_and(|error| !error.is_null()) {
            return Err(invalid(
                "discovery_shape",
                "接口返回错误对象，未取得有效模型清单",
            ));
        }
        let entries = response
            .get("data")
            .or_else(|| response.get("models"))
            .and_then(Value::as_array)
            .ok_or_else(|| {
                invalid(
                    "discovery_shape",
                    "发现接口没有有效模型数组，可手动输入精确ID",
                )
            })?;
        for value in entries {
            let raw = value
                .get("id")
                .or_else(|| value.get("name"))
                .or_else(|| value.get("model"))
                .and_then(Value::as_str)
                .ok_or_else(|| invalid("discovery_shape", "模型条目缺少精确身份"))?;
            let id = if request.protocol == "google" {
                raw.strip_prefix("models/").unwrap_or(raw)
            } else {
                raw
            };
            if id.trim().is_empty() || id.len() > 2048 {
                return Err(invalid("discovery_identity", "模型身份为空或过长"));
            }
            if seen.insert(id.to_string()) {
                models.push(DiscoveredModel {
                    id: id.into(),
                    label: value
                        .get("displayName")
                        .or_else(|| value.get("display_name"))
                        .and_then(Value::as_str)
                        .unwrap_or(id)
                        .into(),
                    rules: Default::default(),
                });
            }
            if models.len() > 5000 {
                return Err(invalid("discovery_limit", "模型条目超过5000，未截断为成功"));
            }
        }
        if response["has_more"] == true
            && response.get("last_id").and_then(Value::as_str).is_none()
            && response
                .get("nextPageToken")
                .and_then(Value::as_str)
                .is_none()
        {
            return Err(invalid(
                "discovery_pagination",
                "接口表示仍有模型但缺少分页游标，未报告完整成功",
            ));
        }
        next = response
            .get("nextPageToken")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| {
                if response["has_more"] == true {
                    response
                        .get("last_id")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                } else {
                    None
                }
            });
        if next.is_none() {
            break;
        }
        if page == 19 {
            return Err(invalid(
                "discovery_pagination",
                "模型分页仍有未读取内容，未报告完整成功",
            ));
        }
    }
    let reference = if !persist_credential {
        request.credential_ref
    } else if let Some(provided) = provided {
        if provided.is_empty() {
            None
        } else {
            let reference = uuid::Uuid::new_v4().to_string();
            entry(&reference)?.set_password(&provided).map_err(|_| {
                invalid("credential_store", "发现成功但密钥无法安全暂存，未加入草稿")
            })?;
            Some(reference)
        }
    } else {
        request.credential_ref
    };
    Ok(DiscoveryResult {
        models,
        credential_ref: reference,
        elapsed_ms: started.elapsed().as_millis() as u64,
    })
}

async fn read_first_page(
    client: &reqwest::Client,
    base: &reqwest::Url,
    protocol: &str,
    secret: Option<&str>,
) -> ModelResult<(reqwest::Url, Value)> {
    let mut candidates = paths::candidates(base, protocol).into_iter().peekable();
    while let Some(target) = candidates.next() {
        match read_page(client, &target, protocol, secret).await {
            // Select an endpoint only before accepting its first page. Never mix pages
            // from different paths, or retry authentication/transport failures.
            Err(error)
                if matches!(error.code.as_str(), "discovery_html" | "discovery_path")
                    && candidates.peek().is_some() => {}
            result => return result.map(|response| (target, response)),
        }
    }
    Err(invalid(
        "discovery_protocol",
        "此协议没有可用的模型清单路径",
    ))
}

async fn read_page(
    client: &reqwest::Client,
    target: &reqwest::Url,
    protocol: &str,
    secret: Option<&str>,
) -> ModelResult<Value> {
    let mut query = client
        .get(target.clone())
        .header(reqwest::header::ACCEPT, "application/json");
    // Protocol version is required even for an auth-free Anthropic relay.
    if protocol == "anthropic" {
        query = query.header("anthropic-version", "2023-06-01");
    }
    if let Some(secret) = secret {
        query = match protocol {
            "anthropic" => query.header("x-api-key", secret),
            "google" => query.header("x-goog-api-key", secret),
            _ => query.bearer_auth(secret),
        };
    }
    let mut response = query.send().await.map_err(transport_error)?;
    let status = response.status().as_u16();
    if !response.status().is_success() {
        return Err(invalid(
            match status {
                401 | 403 => "discovery_auth",
                404 | 405 | 410 => "discovery_path",
                _ => "discovery_failed",
            },
            format!("模型清单接口返回 HTTP{status}，未取得有效清单"),
        ));
    }
    let html_content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .is_some_and(|value| {
            value.trim().eq_ignore_ascii_case("text/html")
                || value.trim().eq_ignore_ascii_case("application/xhtml+xml")
        });
    let mut bytes = vec![];
    while let Some(chunk) = response.chunk().await.map_err(transport_error)? {
        if bytes.len() + chunk.len() > 8 * 1024 * 1024 {
            return Err(invalid("discovery_limit", "发现响应超过8MiB"));
        }
        bytes.extend_from_slice(&chunk);
    }
    // Accept a valid JSON list even when a relay mislabels Content-Type. Never
    // include upstream bytes or parser diagnostics in public errors (they can echo a Key).
    serde_json::from_slice(&bytes).map_err(|_| {
        let prefix = String::from_utf8_lossy(&bytes[..bytes.len().min(512)]);
        let prefix = prefix
            .trim_start_matches('\u{feff}')
            .trim_start()
            .to_ascii_lowercase();
        if html_content_type
            || ["<!doctype html", "<html", "<head", "<body"]
                .iter()
                .any(|tag| prefix.starts_with(tag))
        {
            invalid(
                "discovery_html",
                "模型清单接口返回 HTML 页面，请检查服务地址及代理路径",
            )
        } else {
            invalid(
                "discovery_json",
                "模型清单响应不是有效 JSON，请检查接口及代理响应",
            )
        }
    })
}

fn transport_error(error: reqwest::Error) -> ModelError {
    if error.is_timeout() {
        invalid("discovery_timeout", "读取模型清单超时，请检查节点连接")
    } else {
        invalid(
            "discovery_failed",
            "模型清单连接失败或响应中断，请检查网络、端口和 TLS 证书",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn new_channel_preview_with_a_key_never_stores_a_credential_reference() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let app = axum::Router::new().route(
            "/v1/models",
            axum::routing::get(|headers: axum::http::HeaderMap| async move {
                assert_eq!(
                    headers.get("authorization").unwrap(),
                    "Bearer fixture-preview-only-SECRET"
                );
                axum::Json(serde_json::json!({"data":[{"id":"raw/preview"}]}))
            }),
        );
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let result = preview(DiscoveryRequest {
            protocol: "openai-chat".into(),
            url: format!("http://127.0.0.1:{port}/v1"),
            api_key: Some("fixture-preview-only-SECRET".into()),
            credential_ref: None,
            allow_private_network: true,
            provider_id: None,
            home: None,
        })
        .await
        .unwrap();
        assert!(result.credential_ref.is_none());
        assert_eq!(result.models[0].id, "raw/preview");
        server.abort();
    }
    #[tokio::test]
    async fn unsaved_models_discovery_never_registers_or_calls_generation() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let observed = calls.clone();
        let app=axum::Router::new().route("/v1/models",axum::routing::get(move|headers:axum::http::HeaderMap|{let observed=observed.clone();async move{assert!(headers.get("authorization").is_none());observed.fetch_add(1,std::sync::atomic::Ordering::SeqCst);axum::Json(serde_json::json!({"data":[{"id":"vendor/model-1"},{"id":"vendor/model-1"},{"id":"model-2"}]}))}}));
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let result = discover(DiscoveryRequest {
            protocol: "openai-responses".into(),
            url: format!("http://127.0.0.1:{port}/v1"),
            api_key: None,
            credential_ref: None,
            allow_private_network: true,
            provider_id: None,
            home: None,
        })
        .await
        .unwrap();
        assert_eq!(
            result
                .models
                .iter()
                .map(|m| m.id.as_str())
                .collect::<Vec<_>>(),
            vec!["vendor/model-1", "model-2"]
        );
        assert_eq!(result.credential_ref, None);
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
        server.abort();
    }
    #[tokio::test]
    async fn unapproved_private_and_credential_in_url_are_refused() {
        let error = discover(DiscoveryRequest {
            protocol: "openai-chat".into(),
            url: "http://127.0.0.1:9/v1".into(),
            api_key: None,
            credential_ref: None,
            allow_private_network: false,
            provider_id: None,
            home: None,
        })
        .await
        .unwrap_err();
        assert_eq!(error.code, "discovery_private");
        assert_eq!(
            discover(DiscoveryRequest {
                protocol: "openai-chat".into(),
                url: "https://user:password@example.test/v1".into(),
                api_key: None,
                credential_ref: None,
                allow_private_network: false,
                provider_id: None,
                home: None,
            })
            .await
            .unwrap_err()
            .code,
            "node_url"
        );
    }
}

#[cfg(test)]
#[path = "discovery_contract_tests.rs"]
mod contract_tests;
