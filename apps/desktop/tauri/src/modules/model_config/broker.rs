//! Opt-in loopback broker for one-copy external tasks. Preparing a task starts
//! the transport; only copying activates its scoped submission capability.
use super::{
    agent_runtime_v2 as v2,
    schema::*,
    schema_v2::{canonical_bytes, digest, parse_strict_json_v2},
    tasks::CreateTask,
};
use axum::{
    body::Bytes,
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
    path::{Path as FsPath, PathBuf},
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
        let previous: Descriptor = serde_json::from_value(
            parse_strict_json_v2(&safety::read_file(&descriptor, 4096)?).map_err(v2_error)?,
        )?;
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
    let app = router(state);
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
fn router(state: Arc<BrokerState>) -> Router {
    Router::new()
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
        .with_state(state)
}
async fn local_only(
    State(state): State<Arc<BrokerState>>,
    request: Request,
    next: Next,
) -> Response {
    let host = request.headers().get("host").and_then(|v| v.to_str().ok());
    if host != Some(state.authority.as_str())
        || request.headers().get_all("host").iter().count() != 1
        || request.headers().contains_key("origin")
    {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"code":"task_origin","message":"本机任务入口不接受浏览器跨源请求"})),
        )
            .into_response();
    }
    next.run(request).await
}
fn token(headers: &HeaderMap) -> ModelResult<String> {
    if headers.get_all("authorization").iter().count() != 1 {
        return Err(invalid("task_authorization", "缺少唯一的任务能力"));
    }
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .filter(|v| is_token(v))
        .map(str::to_owned)
        .ok_or_else(|| invalid("task_authorization", "缺少有效的任务能力"))
}
fn is_token(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|c| c.is_ascii_hexdigit())
}
fn random_token() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}
fn equal_digest(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |v, (a, b)| v | (a ^ b)) == 0
}
fn v2_error(error: super::schema_v2::MatchErrorV2) -> ModelError {
    invalid(&error.code, "v2外部任务核对失败，未写入模型库")
}
fn unsupported() -> ModelError {
    invalid(
        "task_contract_unsupported",
        "旧版外部任务不再执行，请重新复制v2任务",
    )
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
            "task_authorization" | "task_expired" | "model_agent_expired" => StatusCode::FORBIDDEN,
            "model_save_stale"
            | "model_agent_submission_conflict"
            | "submission_conflict"
            | "task_binding" => StatusCode::CONFLICT,
            "model_limit" => StatusCode::PAYLOAD_TOO_LARGE,
            _ => StatusCode::BAD_REQUEST,
        };
        (code, Json(self.0)).into_response()
    }
}
async fn work<T: Send + 'static>(
    state: Arc<BrokerState>,
    action: impl FnOnce(&FsPath) -> ModelResult<T> + Send + 'static,
) -> Result<Json<T>, ApiError> {
    let admission = crate::infrastructure::storage_writers::global()
        .admit()
        .map_err(ModelError::from)?;
    let value = tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        action(&state.root)
    })
    .await
    .map_err(|_| ApiError(invalid("task_worker", "任务操作中断，请先查询原回执")))??;
    Ok(Json(value))
}
const GRANTS: &str = "manager-state/model-task-broker-v2";
const GRANT_LIMIT: u64 = 64 * 1024;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BoundContextV2 {
    executable: PathBuf,
    working_directory: PathBuf,
    opencodex_home: PathBuf,
}
impl BoundContextV2 {
    fn capture(context: &crate::state::ProcessContext) -> ModelResult<Self> {
        let result = Self {
            executable: context.executable()?,
            working_directory: context.working_directory.clone(),
            opencodex_home: context.opencodex_home.clone(),
        };
        if [
            &result.executable,
            &result.working_directory,
            &result.opencodex_home,
        ]
        .iter()
        .any(|p| !p.is_absolute())
        {
            return Err(invalid("task_binding", "任务运行上下文必须是实际绝对路径"));
        }
        Ok(result)
    }
    fn process(&self) -> crate::state::ProcessContext {
        crate::state::ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(
                self.executable.clone(),
            ),
            working_directory: self.working_directory.clone(),
            opencodex_home: self.opencodex_home.clone(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PayloadIdentityV2 {
    submission_id: String,
    body_digest: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GrantRecordV2 {
    schema_version: u32,
    task_id: String,
    run_id: String,
    root_binding: String,
    context: BoundContextV2,
    task_digest: String,
    contract_digest: String,
    source_digest: String,
    expires_at: String,
    token_digest: String,
    activation_digest: String,
    active: bool,
    revoked: bool,
    payload: Option<PayloadIdentityV2>,
}
fn root_binding(root: &FsPath) -> ModelResult<String> {
    Ok(super::store::sha256(&serde_json::to_vec(
        &std::fs::canonicalize(root)?,
    )?))
}
fn grant_path(root: &FsPath, id: &str) -> ModelResult<PathBuf> {
    require_uuid(id)?;
    let path = root.join(GRANTS).join(format!("{id}.json"));
    crate::modules::backup::safety::check_path(root, &path, true)?;
    Ok(path)
}
fn grant_lock(
    root: &FsPath,
    id: &str,
) -> ModelResult<crate::infrastructure::locking::TargetFileLock> {
    let path = grant_path(root, id)?;
    crate::modules::backup::safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&path),
        true,
    )?;
    Ok(crate::infrastructure::locking::TargetFileLock::lock(&path)?)
}
fn read_grant(root: &FsPath, id: &str) -> ModelResult<GrantRecordV2> {
    let path = grant_path(root, id)?;
    if !path.exists() {
        // Inspect only the private legacy record version. Never load a v1
        // library or reinterpret its content as a v2 task.
        let old = root
            .join("manager-state/model-template-tasks")
            .join(format!("{id}.json"));
        crate::modules::backup::safety::check_path(root, &old, true)?;
        if old.exists() {
            let value = parse_strict_json_v2(&crate::modules::backup::safety::read_file(
                &old,
                4 * 1024 * 1024,
            )?)
            .map_err(v2_error)?;
            if value["task"]["contractVersion"] == 1 {
                return Err(unsupported());
            }
        }
        return Err(invalid("task_authorization", "任务尚未登记或已失效"));
    }
    let value = parse_strict_json_v2(&crate::modules::backup::safety::read_file(
        &path,
        GRANT_LIMIT,
    )?)
    .map_err(v2_error)?;
    if value["schemaVersion"] != 2 {
        return Err(unsupported());
    }
    let record: GrantRecordV2 = serde_json::from_value(value)?;
    if record.task_id != id || record.run_id != id || record.root_binding != root_binding(root)? {
        return Err(invalid("task_binding", "任务来源身份不匹配"));
    }
    Ok(record)
}
fn persist_grant(root: &FsPath, record: &GrantRecordV2) -> ModelResult<()> {
    let bytes = canonical_bytes(record).map_err(v2_error)?;
    if bytes.len() > GRANT_LIMIT as usize {
        return Err(invalid("model_limit", "任务能力记录过大"));
    }
    let path = grant_path(root, &record.task_id)?;
    crate::infrastructure::atomic_write::atomic_write(&path, &bytes, 0o600)?;
    if crate::modules::backup::safety::read_file(&path, GRANT_LIMIT)? != bytes {
        return Err(invalid("task_readback", "任务能力记录回读失败"));
    }
    Ok(())
}
fn validate_grant(
    root: &FsPath,
    record: &GrantRecordV2,
    require_active: bool,
) -> ModelResult<v2::ExternalTaskV2> {
    if record.revoked || (require_active && !record.active) {
        return Err(invalid("task_authorization", "任务未激活或已撤销"));
    }
    if chrono::DateTime::parse_from_rfc3339(&record.expires_at)
        .map_err(|_| invalid("task_expired", "任务期限无效"))?
        < chrono::Utc::now()
    {
        return Err(invalid("task_expired", "任务已过期，请重新复制当前范围"));
    }
    let task = v2::external_context(root, &record.run_id).map_err(v2_error)?;
    if task.task_id != record.task_id
        || task.expires_at != record.expires_at
        || digest(&canonical_bytes(&task).map_err(v2_error)?) != record.task_digest
        || digest(&canonical_bytes(&task.contract).map_err(v2_error)?) != record.contract_digest
    {
        return Err(invalid("task_binding", "任务合同或上下文已变化"));
    }
    Ok(task)
}
fn authorize(
    root: &FsPath,
    record: &GrantRecordV2,
    bearer: &str,
) -> ModelResult<v2::ExternalTaskV2> {
    if !is_token(bearer)
        || !equal_digest(
            &super::store::sha256(bearer.as_bytes()),
            &record.token_digest,
        )
    {
        return Err(invalid("task_authorization", "任务能力不匹配"));
    }
    validate_grant(root, record, true)
}
/// Enter before config/model transaction locks. The same grant lock serializes
/// HTTP submission, activation, revocation and L0's final authorized commit.
pub fn with_external_authority<T>(
    root: &FsPath,
    context: &crate::state::ProcessContext,
    task_id: &str,
    action: impl FnOnce() -> ModelResult<T>,
) -> ModelResult<T> {
    let _lock = grant_lock(root, task_id)?;
    let record = read_grant(root, task_id)?;
    validate_grant(root, &record, true)?;
    if record.context != BoundContextV2::capture(context)? {
        return Err(invalid("task_binding", "任务运行来源已变化"));
    }
    action()
}
async fn context(
    State(state): State<Arc<BrokerState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let bearer = token(&headers)?;
    work(state, move |root| {
        let _lock = grant_lock(root, &id)?;
        let grant = read_grant(root, &id)?;
        Ok(serde_json::to_value(authorize(root, &grant, &bearer)?)?)
    })
    .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Challenge {
    challenge: String,
}
async fn proof(
    State(state): State<Arc<BrokerState>>,
    Path(id): Path<String>,
    body: Bytes,
) -> Result<Json<Value>, ApiError> {
    let authority = state.authority.clone();
    work(state, move |root| {
        let request: Challenge=serde_json::from_value(parse_strict_json_v2(&body).map_err(v2_error)?)?;
        require_uuid(&request.challenge)?;
        let _lock=grant_lock(root,&id)?;
        let grant=read_grant(root,&id)?;
        validate_grant(root,&grant,true)?;
        use hmac::{Hmac,Mac};
        let pid=std::process::id();
        let mut mac=Hmac::<sha2::Sha256>::new_from_slice(grant.token_digest.as_bytes()).map_err(|_| invalid("task_proof","任务证明不可用"))?;
        mac.update(format!("ocxd-model-task-v1\n{id}\n{}\n{authority}\n{pid}",request.challenge).as_bytes());
        Ok(json!({"taskId":id,"challenge":request.challenge,"authority":authority,"pid":pid,"proof":format!("{:x}",mac.finalize().into_bytes())}))
    }).await
}
async fn contract(
    State(state): State<Arc<BrokerState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let bearer = token(&headers)?;
    work(state, move |root| {
        let _lock = grant_lock(root, &id)?;
        let grant = read_grant(root, &id)?;
        Ok(authorize(root, &grant, &bearer)?.contract)
    })
    .await
}
fn parse_submission(bytes: &[u8]) -> ModelResult<(String, String)> {
    let value = parse_strict_json_v2(bytes).map_err(v2_error)?;
    if value["contractVersion"] != 2 {
        return Err(unsupported());
    }
    let submission: v2::ExternalSubmissionV2 = serde_json::from_value(value)?;
    require_uuid(&submission.submission_id)?;
    Ok((submission.submission_id, super::store::sha256(bytes)))
}
fn check_payload(record: &GrantRecordV2, id: &str, hash: &str) -> ModelResult<()> {
    if record
        .payload
        .as_ref()
        .is_some_and(|p| p.submission_id != id || p.body_digest != hash)
    {
        return Err(invalid(
            "submission_conflict",
            "该任务已固定另一份提交字节，请查询原回执",
        ));
    }
    Ok(())
}
fn receipt_value(root: &FsPath, record: &GrantRecordV2, submission_id: &str) -> ModelResult<Value> {
    let receipt = v2::receipt_external(root, &record.run_id, submission_id).map_err(v2_error)?;
    let Some(receipt) = receipt else {
        return Ok(Value::Null);
    };
    let mut value = serde_json::to_value(receipt)?;
    value["payloadDigest"] = json!(record.payload.as_ref().map(|p| &p.body_digest));
    Ok(value)
}
async fn preflight(
    State(state): State<Arc<BrokerState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, ApiError> {
    let bearer = token(&headers)?;
    work(state, move |root| {
        let _lock = grant_lock(root, &id)?;
        let record = read_grant(root, &id)?;
        authorize(root, &record, &bearer)?;
        let (submission_id, hash) = parse_submission(&body)?;
        check_payload(&record, &submission_id, &hash)?;
        if let Some(receipt) = v2::receipt_external(root, &id, &submission_id).map_err(v2_error)? {
            return Ok(serde_json::to_value(receipt.result)?);
        }
        Ok(serde_json::to_value(
            v2::preflight_external(root, &record.context.process(), &id, &body)
                .map_err(v2_error)?,
        )?)
    })
    .await
}
async fn submit(
    State(state): State<Arc<BrokerState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, ApiError> {
    let bearer = token(&headers)?;
    let root = state.root.clone();
    let result = work(state, move |root| {
        let _lock = grant_lock(root, &id)?;
        let mut record = read_grant(root, &id)?;
        authorize(root, &record, &bearer)?;
        let (submission_id, hash) = parse_submission(&body)?;
        check_payload(&record, &submission_id, &hash)?;
        if v2::receipt_external(root, &id, &submission_id)
            .map_err(v2_error)?
            .is_some()
        {
            return receipt_value(root, &record, &submission_id);
        }
        if record.payload.is_none() {
            let review = v2::preflight_external(root, &record.context.process(), &id, &body)
                .map_err(v2_error)?;
            if review.review.as_ref().is_some_and(|r| !r.valid) {
                return Err(invalid(
                    "model_agent_proposal_invalid",
                    "提案尚未通过v2范围与内容核对",
                ));
            }
            record.payload = Some(PayloadIdentityV2 {
                submission_id: submission_id.clone(),
                body_digest: hash,
            });
            persist_grant(root, &record)?;
        }
        v2::submit_external_review(root, &record.context.process(), &id, &body)
            .map_err(v2_error)?;
        receipt_value(root, &record, &submission_id)
    })
    .await;
    // This signal refreshes the persisted review. Save notifications and actual
    // library/version writes remain exclusively in L0's acceptance transaction.
    if result.is_ok() {
        announce(&root);
    }
    result
}
async fn receipt(
    State(state): State<Arc<BrokerState>>,
    Path((id, submission)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let bearer = token(&headers)?;
    work(state, move |root| {
        let _lock = grant_lock(root, &id)?;
        let record = read_grant(root, &id)?;
        authorize(root, &record, &bearer)?;
        require_uuid(&submission)?;
        receipt_value(root, &record, &submission)
    })
    .await
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalBrokerBundleV2 {
    pub task_id: String,
    pub content: String,
    pub prepared_ticket: String,
    pub expires_at: String,
}
pub async fn prepare_external_broker(
    root: PathBuf,
    context: crate::state::ProcessContext,
    intent: v2::GenerateIntentV2,
) -> ModelResult<ExternalBrokerBundleV2> {
    let endpoint = ensure(root.clone()).await?;
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        prepare_bundle_at(&root, &context, intent, &endpoint)
    })
    .await
    .map_err(|_| invalid("task_worker", "准备v2外部任务中断"))?
}
fn prepare_bundle_at(
    root: &FsPath,
    context: &crate::state::ProcessContext,
    intent: v2::GenerateIntentV2,
    endpoint: &str,
) -> ModelResult<ExternalBrokerBundleV2> {
    let bound = BoundContextV2::capture(context)?;
    let before = super::execution_directory_v2::capture_saved(root, context).map_err(v2_error)?;
    let task = v2::prepare_external(root, context, intent).map_err(v2_error)?;
    let saved = super::execution_directory_v2::capture_saved(root, context).map_err(v2_error)?;
    if before.source_digest != saved.source_digest
        || before.library_digest().map_err(v2_error)? != task.expected_library
    {
        return Err(invalid("task_binding", "准备期间保存来源已变化"));
    }
    let bearer = random_token();
    let ticket = random_token();
    let record = GrantRecordV2 {
        schema_version: 2,
        task_id: task.task_id.clone(),
        run_id: task.task_id.clone(),
        root_binding: root_binding(root)?,
        context: bound,
        task_digest: digest(&canonical_bytes(&task).map_err(v2_error)?),
        contract_digest: digest(&canonical_bytes(&task.contract).map_err(v2_error)?),
        source_digest: saved.source_digest,
        expires_at: task.expires_at.clone(),
        token_digest: super::store::sha256(bearer.as_bytes()),
        activation_digest: super::store::sha256(ticket.as_bytes()),
        active: false,
        revoked: false,
        payload: None,
    };
    let _lock = grant_lock(root, &task.task_id)?;
    if grant_path(root, &task.task_id)?.exists() {
        return Err(invalid("task_collision", "任务能力身份已登记"));
    }
    persist_grant(root, &record)?;
    let base = format!("{endpoint}/v1/tasks/{}", task.task_id);
    let discovery = root.join(DESCRIPTOR).to_string_lossy().into_owned();
    let transport = json!({"baseUrl":base,"headers":{"Authorization":format!("Bearer {bearer}"),"Content-Type":"application/json"},"discoveryFile":discovery,"proof":{"method":"POST /proof","request":{"challenge":"new UUIDv4"},"key":"UTF8('sha256:'+hex(SHA256(UTF8(bearer token))))","payload":"ocxd-model-task-v1\n{taskId}\n{challenge}\n{authority}\n{pid}","algorithm":"HMAC-SHA256 hex; compare in constant time before forwarding Bearer. On every reconnect read descriptor version=1, authority, pid; allow only 127.0.0.1:port and verify taskId/challenge/authority/pid. Disable proxies and redirects."},"methods":{"context":"GET /context","contract":"GET /contract","preflight":"POST /preflight","submit":"POST /submit","receipt":"GET /receipts/{submissionId}"}});
    let content=format!("仅按v2契约制作指定范围提案，保留全部人工锁。严格JSON先preflight，valid后同一submissionId与原始字节submit；断连先查询receipt，同ID不得换字节。提交仅进入待审，只有管理器人工确认后统一事务才可保存库。CheckOnly只能报告。不可修改凭据、运行配置、历史、默认或采用关系。先核验同一个本机broker proof，禁止代理和重定向。\n\n{}",serde_json::to_string_pretty(&json!({"task":task,"transport":transport}))?);
    Ok(ExternalBrokerBundleV2 {
        task_id: record.task_id,
        content,
        prepared_ticket: ticket,
        expires_at: record.expires_at,
    })
}
pub fn activate_external(
    root: &FsPath,
    context: &crate::state::ProcessContext,
    task_id: &str,
    prepared_ticket: &str,
) -> ModelResult<()> {
    let _lock = grant_lock(root, task_id)?;
    let mut record = read_grant(root, task_id)?;
    validate_grant(root, &record, false)?;
    if !is_token(prepared_ticket)
        || !equal_digest(
            &super::store::sha256(prepared_ticket.as_bytes()),
            &record.activation_digest,
        )
        || record.context != BoundContextV2::capture(context)?
    {
        return Err(invalid("task_authorization", "预备票据或运行来源不匹配"));
    }
    if record.active {
        return Ok(());
    }
    let saved = super::execution_directory_v2::capture_saved(root, context).map_err(v2_error)?;
    let task = v2::external_context(root, task_id).map_err(v2_error)?;
    if saved.source_digest != record.source_digest
        || saved.library_digest().map_err(v2_error)? != task.expected_library
    {
        return Err(invalid("task_binding", "复制前来源已变化，请重新准备任务"));
    }
    record.active = true;
    persist_grant(root, &record)
}
pub fn revoke_external(root: &FsPath, task_id: &str) -> ModelResult<()> {
    let _lock = grant_lock(root, task_id)?;
    let mut record = read_grant(root, task_id)?;
    if record.revoked {
        return Ok(());
    }
    record.revoked = true;
    persist_grant(root, &record)?;
    v2::revoke_external(root, task_id).map_err(v2_error)
}
/// Legacy command type stays source-compatible until L0 replaces its public
/// entry point. It never prepares or executes a v1 task or reads a v1 library.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskBundle {
    pub task_id: String,
    pub content: String,
    pub grant: super::tasks::TaskGrant,
}
pub async fn create_bundle(_root: PathBuf, _request: CreateTask) -> ModelResult<TaskBundle> {
    Err(unsupported())
}
#[cfg(test)]
#[path = "broker_v2_tests.rs"]
mod tests;
