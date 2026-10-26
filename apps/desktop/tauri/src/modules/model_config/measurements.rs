//! Node measurements are model-list reads. No registration, generation,
//! credential persistence, automatic switching or retry is performed.
use super::{discovery, schema::*};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::PathBuf, sync::OnceLock};
static RUNS: OnceLock<super::operations::Registry> = OnceLock::new();
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub run_id: String,
    pub protocol: String,
    pub nodes: Vec<ServiceNode>,
    pub api_key: Option<String>,
    pub credential_ref: Option<String>,
    pub allow_private_network: bool,
    /// 渠道身份（官方 provider id）；用于回落到该渠道已配置的 Key。
    #[serde(default)]
    pub provider_id: Option<String>,
    /// 官方配置根。命令层注入，界面不能传路径。
    #[serde(skip)]
    pub home: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Measurement {
    pub node_id: String,
    pub elapsed_ms: Option<u64>,
    pub model_count: Option<usize>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultSet {
    pub run_id: String,
    pub stopped: bool,
    pub results: Vec<Measurement>,
}

pub fn stop(id: &str) -> ModelResult<()> {
    RUNS.get_or_init(Default::default).stop(id)
}
pub async fn measure(mut request: Request) -> ModelResult<ResultSet> {
    require_uuid(&request.run_id)?;
    if request.nodes.is_empty() || request.nodes.len() > 32 {
        return Err(invalid("node_scope", "请选择1至32个节点"));
    }
    let mut ids = BTreeSet::new();
    for node in &request.nodes {
        require_uuid(&node.id)?;
        if !ids.insert(&node.id) {
            return Err(invalid("node_scope", "检测节点身份重复"));
        }
    }
    let (mut cancel, _run) = RUNS.get_or_init(Default::default).start(&request.run_id)?;
    let key = request.api_key.take().map(zeroize::Zeroizing::new);
    let mut results = vec![];
    let mut stopped = false;
    for node in request.nodes {
        if *cancel.borrow() {
            stopped = true;
            break;
        }
        let operation = discovery::readonly(discovery::DiscoveryRequest {
            protocol: request.protocol.clone(),
            url: node.url,
            api_key: key.as_ref().map(|key| key.to_string()),
            credential_ref: request.credential_ref.clone(),
            allow_private_network: request.allow_private_network,
            provider_id: request.provider_id.clone(),
            home: request.home.clone(),
        });
        let result = tokio::select! {value=operation=>Some(value),_=cancel.changed()=>None};
        match result {
            None => {
                stopped = true;
                break;
            }
            Some(Ok(value)) => results.push(Measurement {
                node_id: node.id,
                elapsed_ms: Some(value.elapsed_ms),
                model_count: Some(value.models.len()),
                error: None,
            }),
            Some(Err(error)) => results.push(Measurement {
                node_id: node.id,
                elapsed_ms: None,
                model_count: None,
                error: Some(error.code),
            }),
        }
    }
    Ok(ResultSet {
        run_id: request.run_id,
        stopped,
        results,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{extract::State, http::HeaderMap, routing::get, Json, Router};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    #[tokio::test]
    async fn batch_uses_only_model_lists_and_never_persists_temporary_credentials() {
        let calls = Arc::new(AtomicUsize::new(0));
        let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let port = listener.local_addr().unwrap().port();
        let router = Router::new()
            .route(
                "/models",
                get(
                    |State(calls): State<Arc<AtomicUsize>>, headers: HeaderMap| async move {
                        assert_eq!(
                            headers.get("authorization").unwrap(),
                            "Bearer temporary-fixture-key"
                        );
                        calls.fetch_add(1, Ordering::SeqCst);
                        Json(serde_json::json!({"data":[{"id":"vendor/raw-id"}]}))
                    },
                ),
            )
            .with_state(calls.clone());
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        let ids = [
            uuid::Uuid::new_v4().to_string(),
            uuid::Uuid::new_v4().to_string(),
        ];
        let run_id = uuid::Uuid::new_v4().to_string();
        let response = measure(Request {
            run_id: run_id.clone(),
            protocol: "openai-chat".into(),
            nodes: ids
                .iter()
                .map(|id| ServiceNode {
                    id: id.clone(),
                    name: id.clone(),
                    url: format!("http://127.0.0.1:{port}"),
                })
                .collect(),
            api_key: Some("temporary-fixture-key".into()),
            credential_ref: None,
            allow_private_network: true,
            provider_id: None,
            home: None,
        })
        .await
        .unwrap();
        assert!(!response.stopped);
        assert_eq!(
            response
                .results
                .iter()
                .map(|r| r.node_id.clone())
                .collect::<Vec<_>>(),
            ids
        );
        assert!(response
            .results
            .iter()
            .all(|r| r.model_count == Some(1) && r.error.is_none()));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        server.abort();
    }
    #[tokio::test]
    async fn cancel_drops_the_pending_request_and_returns_only_completed_results() {
        let entered = Arc::new(tokio::sync::Notify::new());
        let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let port = listener.local_addr().unwrap().port();
        let router = Router::new()
            .route(
                "/models",
                get(
                    |State(entered): State<Arc<tokio::sync::Notify>>| async move {
                        entered.notify_one();
                        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                        Json(serde_json::json!({"data":[]}))
                    },
                ),
            )
            .with_state(entered.clone());
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        let run_id = uuid::Uuid::new_v4().to_string();
        let id = run_id.clone();
        let task = tokio::spawn(async move {
            measure(Request {
                run_id: id,
                protocol: "openai-chat".into(),
                nodes: vec![ServiceNode {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: "slow".into(),
                    url: format!("http://127.0.0.1:{port}"),
                }],
                api_key: None,
                credential_ref: None,
                allow_private_network: true,
                provider_id: None,
                home: None,
            })
            .await
        });
        tokio::time::timeout(std::time::Duration::from_secs(3), entered.notified())
            .await
            .unwrap();
        stop(&run_id).unwrap();
        let response = tokio::time::timeout(std::time::Duration::from_secs(2), task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(response.stopped);
        assert!(response.results.is_empty());
        server.abort();
    }

    /// 0.2.12：列表下拉的测速必须复用该渠道已配置的 Key。
    ///
    /// 修前这条链路恒定不带认证头（前端把取 Key 的函数写成返回空串，且导入渠道的
    /// credentialRef 是 null），对需要 Key 的服务必然 401 —— 用户看到的是「测速无效」。
    #[tokio::test]
    async fn measurement_reuses_the_configured_key_for_the_same_origin() {
        let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let port = listener.local_addr().unwrap().port();
        let router = Router::new().route(
            "/models",
            get(|headers: HeaderMap| async move {
                assert_eq!(
                    headers.get("authorization").unwrap(),
                    "Bearer sk-configured-SECRET",
                    "检测必须带上该渠道已配置的 Key"
                );
                Json(serde_json::json!({"data":[{"id":"demo/model"}]}))
            }),
        );
        let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });

        let home = tempfile::tempdir().unwrap();
        let base = format!("http://127.0.0.1:{port}");
        std::fs::write(
            home.path().join("config.json"),
            serde_json::to_vec(&serde_json::json!({
                "providers": {"demo": {
                    "adapter": "openai-chat",
                    "baseUrl": base,
                    "authMode": "key",
                    "apiKey": "sk-configured-SECRET"
                }}
            }))
            .unwrap(),
        )
        .unwrap();

        let response = measure(Request {
            run_id: uuid::Uuid::new_v4().to_string(),
            protocol: "openai-chat".into(),
            nodes: vec![ServiceNode {
                id: uuid::Uuid::new_v4().to_string(),
                name: "当前节点".into(),
                url: base,
            }],
            api_key: None,
            credential_ref: None,
            allow_private_network: true,
            provider_id: Some("demo".into()),
            home: Some(home.path().to_path_buf()),
        })
        .await
        .unwrap();
        assert_eq!(response.results[0].error, None);
        assert_eq!(response.results[0].model_count, Some(1));
        server.abort();
    }

    /// 节点地址被改到别处时不得携带 Key —— 这一条比「测速能用」更重要。
    #[tokio::test]
    async fn measurement_never_sends_the_key_to_a_different_origin() {
        let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let port = listener.local_addr().unwrap().port();
        let router = Router::new().route(
            "/models",
            get(|headers: HeaderMap| async move {
                assert!(headers.get("authorization").is_none(), "不同源不得携带 Key");
                Json(serde_json::json!({"data":[]}))
            }),
        );
        let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });

        let home = tempfile::tempdir().unwrap();
        std::fs::write(
            home.path().join("config.json"),
            serde_json::to_vec(&serde_json::json!({
                "providers": {"demo": {
                    "adapter": "openai-chat",
                    "baseUrl": "http://127.0.0.1:1",
                    "authMode": "key",
                    "apiKey": "sk-configured-SECRET"
                }}
            }))
            .unwrap(),
        )
        .unwrap();

        let response = measure(Request {
            run_id: uuid::Uuid::new_v4().to_string(),
            protocol: "openai-chat".into(),
            nodes: vec![ServiceNode {
                id: uuid::Uuid::new_v4().to_string(),
                name: "改过的节点".into(),
                url: format!("http://127.0.0.1:{port}"),
            }],
            api_key: None,
            credential_ref: None,
            allow_private_network: true,
            provider_id: Some("demo".into()),
            home: Some(home.path().to_path_buf()),
        })
        .await
        .unwrap();
        // 不同源 → 不携带 Key；按节点报「无可确认凭据」，而不是「认证失败」。
        // 单节点失败不整批失败，同批其它节点仍应各自给出结果。
        assert_eq!(response.results.len(), 1);
        assert_eq!(
            response.results[0].error.as_deref(),
            Some("credential_unconfirmed")
        );
        assert_eq!(
            response.results[0].elapsed_ms, None,
            "未发出请求时不得给出伪造耗时"
        );
        server.abort();
    }
}
