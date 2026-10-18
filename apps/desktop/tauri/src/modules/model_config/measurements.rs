//! Node measurements are model-list reads. No registration, generation,
//! credential persistence, automatic switching or retry is performed.
use super::{discovery, schema::*};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Mutex, OnceLock},
};
use tokio::sync::watch;
static RUNS: OnceLock<Mutex<BTreeMap<String, watch::Sender<bool>>>> = OnceLock::new();
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub run_id: String,
    pub protocol: String,
    pub nodes: Vec<ServiceNode>,
    pub api_key: Option<String>,
    pub credential_ref: Option<String>,
    pub allow_private_network: bool,
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
struct Run(String);
impl Drop for Run {
    fn drop(&mut self) {
        if let Ok(mut runs) = RUNS.get_or_init(Default::default).lock() {
            runs.remove(&self.0);
        }
    }
}
pub fn stop(id: &str) -> ModelResult<()> {
    require_uuid(id)?;
    let runs = RUNS
        .get_or_init(Default::default)
        .lock()
        .map_err(|_| invalid("node_run", "检测状态不可用"))?;
    if let Some(sender) = runs.get(id) {
        let _ = sender.send(true);
    }
    Ok(())
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
    let (sender, mut cancel) = watch::channel(false);
    {
        let mut runs = RUNS
            .get_or_init(Default::default)
            .lock()
            .map_err(|_| invalid("node_run", "检测状态不可用"))?;
        if runs.contains_key(&request.run_id) {
            return Err(invalid("node_run", "该检测已在运行"));
        }
        runs.insert(request.run_id.clone(), sender);
    }
    let _run = Run(request.run_id.clone());
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
        assert!(!RUNS.get().unwrap().lock().unwrap().contains_key(&run_id));
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
}
