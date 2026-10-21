//! Opt-in loopback broker for one-copy external tasks. Preparing a task starts
//! the transport; only copying activates its scoped submission capability.
use super::{
    schema::*,
    tasks::{CreateTask, Submission, Task, Tasks},
};
use axum::{
    extract::{DefaultBodyLimit, Path, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    path::PathBuf,
    sync::{Arc, OnceLock},
    time::Duration,
};
use tokio::sync::Mutex;

#[derive(Clone)]
struct BrokerState {
    root: PathBuf,
    authority: String,
}
struct Broker {
    root: PathBuf,
    endpoint: String,
    _lock: crate::infrastructure::locking::TargetFileLock,
    server: tauri::async_runtime::JoinHandle<()>,
}
impl Drop for Broker {
    fn drop(&mut self) {
        self.server.abort();
    }
}
const DESCRIPTOR: &str = "manager-state/model-task-broker.json";
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Descriptor {
    version: u32,
    authority: String,
    pid: u32,
}
static BROKER: OnceLock<Mutex<Option<Broker>>> = OnceLock::new();
static APP: OnceLock<tauri::AppHandle> = OnceLock::new();
pub fn configure(app: tauri::AppHandle) {
    let _ = APP.set(app);
}
pub fn resume(root: PathBuf) {
    // Resume a previously prepared transport only; this never activates a task.
    if !root.join(DESCRIPTOR).exists() {
        return;
    }
    tauri::async_runtime::spawn(async move {
        if ensure(root.clone()).await.is_err() {
            let _ = crate::infrastructure::runtime_log::RuntimeLog::new(&root)
                .append_event("external model task transport resume failed");
        }
    });
}
pub fn announce(root: &std::path::Path) {
    if let Some(app) = APP.get() {
        use crate::modules::notifications::registry::{Channel, Job, Trigger};
        if crate::commands::event_delivery::emit_signal(
            app,
            "model-config-changed",
            Job::ModelConfig,
            Trigger::Commit,
            Channel::Local,
            (),
        )
        .is_err()
        {
            let _ = crate::infrastructure::runtime_log::RuntimeLog::new(root)
                .append_event("model config change signal delivery failed");
        }
    }
}

async fn ensure(root: PathBuf) -> ModelResult<String> {
    let mut current = BROKER.get_or_init(|| Mutex::new(None)).lock().await;
    if let Some(broker) = current.as_ref() {
        if broker.root != root {
            return Err(invalid(
                "storage_binding",
                "数据目录已变化，请重启管理器后重新复制任务",
            ));
        }
        return Ok(broker.endpoint.clone());
    }
    let broker = start(root).await?;
    let endpoint = broker.endpoint.clone();
    *current = Some(broker);
    Ok(endpoint)
}
async fn start(root: PathBuf) -> ModelResult<Broker> {
    use crate::modules::backup::safety;
    let descriptor = root.join(DESCRIPTOR);
    safety::check_path(&root, &descriptor, true)?;
    safety::check_path(
        &root,
        &crate::infrastructure::locking::lock_path_for(&descriptor),
        true,
    )?;
    let lock = crate::infrastructure::locking::TargetFileLock::lock(&descriptor)?;
    let preferred = if descriptor.exists() {
        let previous: Descriptor = serde_json::from_slice(&safety::read_file(&descriptor, 4096)?)?;
        if previous.version != 1 {
            return Err(invalid("task_transport", "任务入口版本不受支持"));
        }
        let address: std::net::SocketAddr = previous
            .authority
            .parse()
            .map_err(|_| invalid("task_transport", "任务入口地址无效"))?;
        if !address.ip().is_loopback() {
            return Err(invalid("task_transport", "任务入口只能绑定回环地址"));
        }
        address.port()
    } else {
        0
    };
    let listener =
        match tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, preferred)).await {
            Ok(listener) => listener,
            Err(_) => tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
                .await
                .map_err(|_| invalid("task_transport", "无法启动本机任务入口"))?,
        };
    let address = listener
        .local_addr()
        .map_err(|_| invalid("task_transport", "无法读取任务入口"))?;
    let authority = address.to_string();
    let state = Arc::new(BrokerState {
        root: root.clone(),
        authority: authority.clone(),
    });
    let app = Router::new()
        .route("/v1/tasks/{id}/proof", post(proof))
        .route("/v1/tasks/{id}/context", get(context))
        .route("/v1/tasks/{id}/contract", get(contract))
        .route("/v1/tasks/{id}/preflight", post(preflight))
        .route("/v1/tasks/{id}/submit", post(submit))
        .route("/v1/tasks/{id}/receipts/{submission}", get(receipt))
        .layer(DefaultBodyLimit::max(4 * 1024 * 1024))
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(20),
        ))
        .layer(middleware::from_fn_with_state(state.clone(), local_only))
        .with_state(state);
    let server = tauri::async_runtime::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    let endpoint = format!("http://{authority}");
    if let Err(error) = crate::infrastructure::atomic_write::atomic_write(
        &descriptor,
        &serde_json::to_vec(&Descriptor {
            version: 1,
            authority,
            pid: std::process::id(),
        })?,
        0o600,
    ) {
        server.abort();
        return Err(error.into());
    }
    Ok(Broker {
        root,
        endpoint,
        _lock: lock,
        server,
    })
}
async fn local_only(
    State(state): State<Arc<BrokerState>>,
    request: Request,
    next: Next,
) -> Response {
    let host = request.headers().get("host").and_then(|v| v.to_str().ok());
    if host != Some(state.authority.as_str()) || request.headers().contains_key("origin") {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"code":"task_origin","message":"本机任务入口不接受浏览器跨源请求"})),
        )
            .into_response();
    }
    next.run(request).await
}
fn token(headers: &HeaderMap) -> ModelResult<String> {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .filter(|v| v.len() == 64 && v.bytes().all(|c| c.is_ascii_hexdigit()))
        .map(str::to_owned)
        .ok_or_else(|| invalid("task_authorization", "缺少有效的任务能力"))
}
struct ApiError(ModelError);
impl From<ModelError> for ApiError {
    fn from(error: ModelError) -> Self {
        Self(error)
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let code = match self.0.code.as_str() {
            "task_authorization" => StatusCode::FORBIDDEN,
            "stale_baseline" | "submission_conflict" => StatusCode::CONFLICT,
            _ => StatusCode::BAD_REQUEST,
        };
        (code, Json(self.0)).into_response()
    }
}
async fn work<T: Send + 'static>(
    state: Arc<BrokerState>,
    action: impl FnOnce(Tasks) -> ModelResult<T> + Send + 'static,
) -> Result<Json<T>, ApiError> {
    let admission = crate::infrastructure::storage_writers::global()
        .admit()
        .map_err(ModelError::from)?;
    let value = tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        action(Tasks::new(&state.root))
    })
    .await
    .map_err(|_| ApiError(invalid("task_worker", "任务操作中断，请先查询原回执")))??;
    Ok(Json(value))
}
async fn context(
    State(state): State<Arc<BrokerState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Task>, ApiError> {
    let token = token(&headers)?;
    work(state, move |tasks| tasks.context(&id, &token)).await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Challenge {
    challenge: String,
}
async fn proof(
    State(state): State<Arc<BrokerState>>,
    Path(id): Path<String>,
    Json(request): Json<Challenge>,
) -> Result<Json<Value>, ApiError> {
    let authority = state.authority.clone();
    work(state, move |tasks| {
        let pid = std::process::id();
        let proof = tasks.broker_proof(&id, &request.challenge, &authority, pid)?;
        Ok(json!({"taskId":id,"challenge":request.challenge,"authority":authority,"pid":pid,"proof":proof}))
    }).await
}
async fn contract(
    State(state): State<Arc<BrokerState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let token = token(&headers)?;
    work(state, move |tasks| tasks.contract(&id, &token)).await
}
async fn preflight(
    State(state): State<Arc<BrokerState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(submission): Json<Submission>,
) -> Result<Json<Value>, ApiError> {
    let token = token(&headers)?;
    work(state, move |tasks| {
        tasks.preflight(&id, &token, &submission)
    })
    .await
}
async fn submit(
    State(state): State<Arc<BrokerState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(submission): Json<Submission>,
) -> Result<Json<super::tasks::Receipt>, ApiError> {
    let token = token(&headers)?;
    let root = state.root.clone();
    let operation_root = root.clone();
    let result = work(state, move |tasks| {
        tasks.context(&id, &token)?;
        use super::feedback::{self, OperationKind, OperationPhase};
        feedback::progress(
            &operation_root,
            &submission.submission_id,
            OperationKind::ExternalSubmit,
            OperationPhase::Running,
        );
        let result = tasks.submit(&id, &token, submission.clone());
        feedback::progress(
            &operation_root,
            &submission.submission_id,
            OperationKind::ExternalSubmit,
            if result.is_ok() {
                OperationPhase::Succeeded
            } else {
                OperationPhase::Failed
            },
        );
        feedback::external_terminal(
            &operation_root,
            &id,
            &submission.submission_id,
            result.is_ok(),
        );
        result
    })
    .await;
    if result.as_ref().is_ok_and(|value| value.0.state == "saved") {
        announce(&root);
    }
    result
}
async fn receipt(
    State(state): State<Arc<BrokerState>>,
    Path((id, submission)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Option<super::tasks::Receipt>>, ApiError> {
    let token = token(&headers)?;
    work(state, move |tasks| tasks.receipt(&id, &token, &submission)).await
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskBundle {
    pub task_id: String,
    pub content: String,
    pub grant: super::tasks::TaskGrant,
}
/// The copied content contains instruction + exact targets + generated schema once.
pub async fn create_bundle(root: PathBuf, request: CreateTask) -> ModelResult<TaskBundle> {
    let endpoint = ensure(root.clone()).await?;
    let discovery_file = root.join(DESCRIPTOR).to_string_lossy().into_owned();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    let grant = tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        Tasks::new(&root).prepare(request)
    })
    .await
    .map_err(|_| invalid("task_worker", "准备任务中断"))??;
    let contract = super::tasks::contract_definition();
    let base = format!("{endpoint}/v1/tasks/{}", grant.task.id);
    let instructions = "按本任务范围制作模型模版。只使用随附契约字段，记录资料来源和未核实项，保留人工锁；不要修改旧版本、默认指针、采用关系、凭据或运行文件。完整结果先POST /preflight，确认valid后以同一submissionId POST /submit；软件分配版本。断连先GET /receipts/{submissionId}再决定重试，同ID不得换内容。仅检查场景只能提交报告，proposals必须为空。不要直接写正式库。外部执行者自身的其他文件权限由其宿主控制；任务能力仅授权此服务范围。";
    let transport = json!({"baseUrl":base,"headers":{"Authorization":format!("Bearer {}",grant.token),"Content-Type":"application/json"},"discoveryFile":discovery_file,"proof":{"method":"POST /proof","request":{"challenge":"新生成的UUIDv4"},"key":"UTF8('sha256:'+hex(SHA256(UTF8(bearer token))))","payload":"ocxd-model-task-v1\n{taskId}\n{challenge}\n{authority}\n{pid}","algorithm":"HMAC-SHA256，返回hex；先常量时间比较验证，通过后才转发Bearer。每次重新连接先读discoveryFile中的version=1、authority、pid，只允许127.0.0.1:port；地址、pid、taskId、challenge须逐项相等。禁用代理与重定向。"},"methods":{"context":"GET /context","contract":"GET /contract","preflight":"POST /preflight","submit":"POST /submit","receipt":"GET /receipts/{submissionId}"},"submissionId":"本次提交生成一个完整UUIDv4并在重试时保持","availability":"同一台机器且管理器运行；重启后从discoveryFile重新定位并核验入口。不可达时保留结果，不改写任何库文件"});
    let content = format!(
        "{instructions}\n\n{}",
        serde_json::to_string_pretty(
            &json!({"task":grant.task,"contract":contract,"transport":transport})
        )?
    );
    Ok(TaskBundle {
        task_id: grant.task.id.clone(),
        content,
        grant,
    })
}

#[cfg(test)]
mod tests {
    use super::super::tasks::{Proposal, Scene, TaskGrant};
    use super::*;
    #[tokio::test]
    async fn restart_relocates_the_listener_and_proves_identity_before_receipt_recovery() {
        use hmac::{Hmac, Mac};
        let root = tempfile::tempdir().unwrap();
        let tasks = Tasks::new(root.path());
        let grant = tasks
            .create(CreateTask {
                scene: Scene::FromSources,
                targets: vec![],
                allow_new: true,
                requirements: "新建测试模版".into(),
                expected_library: None,
                buffers: vec![],
            })
            .unwrap();
        let broker = start(root.path().into()).await.unwrap();
        let original: std::net::SocketAddr = broker
            .endpoint
            .trim_start_matches("http://")
            .parse()
            .unwrap();
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        let submission_id = uuid::Uuid::new_v4().to_string();
        let mut content = TemplateContent::default();
        content.matching.include.push(Pattern {
            kind: MatchKind::Prefix,
            value: "proof-test-".into(),
        });
        let saved: super::super::tasks::Receipt = client
            .post(format!(
                "{}/v1/tasks/{}/submit",
                broker.endpoint, grant.task.id
            ))
            .bearer_auth(&grant.token)
            .json(&Submission {
                submission_id: submission_id.clone(),
                contract_version: 1,
                proposals: vec![Proposal {
                    template_id: None,
                    base_revision: None,
                    label: "入口恢复测试".into(),
                    content,
                }],
                report: None,
            })
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(saved.state, "saved");
        let before = super::super::ModelStore::new(root.path()).load().unwrap();
        drop(broker);
        // Reproduce a different process occupying the old ephemeral port.
        tokio::task::yield_now().await;
        let mut occupied = None;
        for _ in 0..20 {
            if let Ok(listener) = tokio::net::TcpListener::bind(original).await {
                occupied = Some(listener);
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        let _occupied = occupied.expect("aborted listener should release its port");
        let resumed = start(root.path().into()).await.unwrap();
        let descriptor: Descriptor = serde_json::from_slice(
            &crate::modules::backup::safety::read_file(&root.path().join(DESCRIPTOR), 4096)
                .unwrap(),
        )
        .unwrap();
        assert_ne!(descriptor.authority, original.to_string());
        assert_eq!(resumed.endpoint, format!("http://{}", descriptor.authority));
        let challenge = uuid::Uuid::new_v4().to_string();
        let value: Value = client
            .post(format!(
                "{}/v1/tasks/{}/proof",
                resumed.endpoint, grant.task.id
            ))
            .json(&json!({"challenge":challenge}))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .await
            .unwrap();
        let mut mac = Hmac::<sha2::Sha256>::new_from_slice(
            super::super::store::sha256(grant.token.as_bytes()).as_bytes(),
        )
        .unwrap();
        mac.update(
            format!(
                "ocxd-model-task-v1\n{}\n{challenge}\n{}\n{}",
                grant.task.id, descriptor.authority, descriptor.pid
            )
            .as_bytes(),
        );
        assert_eq!(value["proof"], format!("{:x}", mac.finalize().into_bytes()));
        assert_eq!(value["challenge"], challenge);
        assert_eq!(value["taskId"], grant.task.id);
        let recovered: Option<super::super::tasks::Receipt> = client
            .get(format!(
                "{}/v1/tasks/{}/receipts/{submission_id}",
                resumed.endpoint, grant.task.id
            ))
            .bearer_auth(&grant.token)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(recovered.unwrap(), saved);
        assert_eq!(
            super::super::ModelStore::new(root.path()).load().unwrap(),
            before
        );
        tasks.revoke(&grant.task.id).unwrap();
        assert!(client
            .post(format!(
                "{}/v1/tasks/{}/proof",
                resumed.endpoint, grant.task.id
            ))
            .json(&json!({"challenge":uuid::Uuid::new_v4().to_string()}))
            .send()
            .await
            .unwrap()
            .status()
            .is_client_error());
    }
    #[tokio::test]
    async fn loopback_task_requires_activation_token_scope_and_real_service_receipts() {
        let root = tempfile::tempdir().unwrap();
        let request = CreateTask {
            scene: Scene::FromSources,
            targets: vec![],
            allow_new: true,
            requirements: "仅做公开资料模版".into(),
            expected_library: None,
            buffers: vec![],
        };
        let bundle = create_bundle(root.path().into(), request).await.unwrap();
        assert!(!root
            .path()
            .join("manager-state/model-template-tasks")
            .join(format!("{}.json", bundle.task_id))
            .exists());
        let base = ensure(root.path().into()).await.unwrap();
        let endpoint = format!("{base}/v1/tasks/{}", bundle.task_id);
        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        assert_eq!(
            client
                .get(format!("{endpoint}/context"))
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN
        );
        let grant: TaskGrant = bundle.grant;
        Tasks::new(root.path()).activate(&grant).unwrap();
        let auth = format!("Bearer {}", grant.token);
        assert_eq!(
            client
                .get(format!("{endpoint}/context"))
                .header("Authorization", &auth)
                .header("Origin", "https://untrusted.test")
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            client
                .get(format!("{endpoint}/context"))
                .header("Authorization", &auth)
                .header("Host", "rebound.test")
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN
        );
        let context: Value = client
            .get(format!("{endpoint}/context"))
            .header("Authorization", &auth)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(context["id"], bundle.task_id);
        assert!(context.get("token").is_none());
        let mut content = TemplateContent::default();
        content.matching.include.push(Pattern {
            kind: MatchKind::Prefix,
            value: "test-".into(),
        });
        let submission = Submission {
            submission_id: uuid::Uuid::new_v4().to_string(),
            contract_version: 1,
            proposals: vec![Proposal {
                template_id: None,
                base_revision: None,
                label: "网络提交模版".into(),
                content,
            }],
            report: None,
        };
        let response = client
            .post(format!("{endpoint}/preflight"))
            .header("Authorization", &auth)
            .json(&submission)
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        assert!(!root.path().join(LIBRARY_PATH).exists());
        let response = client
            .post(format!("{endpoint}/submit"))
            .header("Authorization", &auth)
            .json(&submission)
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let receipt: Value = response.json().await.unwrap();
        assert_eq!(receipt["state"], "saved");
        let replay: Value = client
            .post(format!("{endpoint}/submit"))
            .header("Authorization", &auth)
            .json(&submission)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(receipt, replay);
        let query: Value = client
            .get(format!("{endpoint}/receipts/{}", submission.submission_id))
            .header("Authorization", &auth)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(receipt, query);
        Tasks::new(root.path()).revoke(&grant.task.id).unwrap();
        assert_eq!(
            client
                .get(format!("{endpoint}/context"))
                .header("Authorization", &auth)
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN
        );
    }
}
