//! Authenticated official management requests bind to the protected runtime record.
//! Reusable admin credentials are read only after the process proves the record key.
use super::schema::*;
use crate::{modules::backup::safety, state::ProcessContext};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;
use std::time::Duration;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KeyState {
    pub active_id: Option<String>,
    pub ids: std::collections::BTreeSet<String>,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeRecord {
    pid: u32,
    port: u16,
    attestation_secret: Option<String>,
}

fn read_record(context: &ProcessContext) -> ModelResult<RuntimeRecord> {
    let path = context.opencodex_home.join("runtime-port.json");
    safety::check_path(&context.opencodex_home, &path, false)?;
    let record: RuntimeRecord = serde_json::from_slice(&safety::read_file(&path, 4096)?)?;
    if record.pid == 0 || record.port == 0 {
        return Err(invalid("runtime_identity", "官方运行身份无效"));
    }
    Ok(record)
}
pub struct Management {
    client: reqwest::Client,
    base: String,
    context: ProcessContext,
    record: RuntimeRecord,
}
impl Management {
    pub fn base_url(&self) -> &str {
        &self.base
    }
    pub fn verify_binding(&self) -> ModelResult<()> {
        self.unchanged()
    }
    pub async fn bind(context: &ProcessContext) -> ModelResult<Self> {
        let record = read_record(context)?;
        let secret = record
            .attestation_secret
            .as_ref()
            .filter(|s| s.len() == 43)
            .ok_or_else(|| {
                invalid(
                    "runtime_attestation",
                    "当前官方版本缺少本机运行证明，未发送管理凭据",
                )
            })?;
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|_| invalid("management_client", "管理连接未就绪"))?;
        let base = format!("http://127.0.0.1:{}", record.port);
        let challenge = URL_SAFE_NO_PAD.encode(
            [
                uuid::Uuid::new_v4().as_bytes().as_slice(),
                uuid::Uuid::new_v4().as_bytes().as_slice(),
            ]
            .concat(),
        );
        let response = client
            .get(format!("{base}/healthz"))
            .header("x-opencodex-attestation-challenge", &challenge)
            .send()
            .await
            .map_err(|_| invalid("runtime_attestation", "运行身份验证不可达"))?;
        let proof = response
            .headers()
            .get("x-opencodex-attestation-proof")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| URL_SAFE_NO_PAD.decode(v).ok())
            .ok_or_else(|| invalid("runtime_attestation", "运行端口没有有效证明，未发送凭据"))?;
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
            .map_err(|_| invalid("runtime_attestation", "运行证明密钥无效"))?;
        mac.update(
            format!(
                "opencodex-local-management-v1\n{challenge}\n{}\n{}",
                record.pid, record.port
            )
            .as_bytes(),
        );
        mac.verify_slice(&proof)
            .map_err(|_| invalid("runtime_attestation", "运行端口证明不匹配，未发送凭据"))?;
        let health: Value = response
            .json()
            .await
            .map_err(|_| invalid("runtime_identity", "运行身份响应无效"))?;
        if health["service"] != "opencodex" || health["pid"].as_u64() != Some(record.pid as u64) {
            return Err(invalid("runtime_identity", "运行身份已变化"));
        }
        Ok(Self {
            client,
            base,
            context: context.clone(),
            record,
        })
    }
    fn unchanged(&self) -> ModelResult<()> {
        let now = read_record(&self.context)?;
        if (now.pid, now.port, now.attestation_secret)
            != (
                self.record.pid,
                self.record.port,
                self.record.attestation_secret.clone(),
            )
        {
            return Err(invalid("stale_runtime", "运行实例已变化，未发送管理请求"));
        }
        Ok(())
    }
    pub async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
    ) -> ModelResult<Value> {
        if ![
            "/api/providers/keys",
            "/api/providers/keys/active",
            "/api/providers",
            "/api/disabled-models",
            "/api/subagent-models",
        ]
        .contains(&path.split('?').next().unwrap_or(""))
        {
            return Err(invalid("management_scope", "未注册的管理操作"));
        }
        self.unchanged()?;
        let token_path = self.context.opencodex_home.join("admin-api-token");
        safety::check_path(&self.context.opencodex_home, &token_path, false)?;
        let bytes = safety::read_file(&token_path, 512)?;
        let token = std::str::from_utf8(&bytes)
            .map_err(|_| invalid("management_auth", "管理令牌编码无效"))?
            .trim();
        if !token.starts_with("ocx_admin_")
            || token.len() != 53
            || !token[10..]
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        {
            return Err(invalid("management_auth", "管理令牌格式无效"));
        }
        let mut request = self
            .client
            .request(method, format!("{}{path}", self.base))
            .header("Authorization", format!("Bearer {token}"));
        if let Some(body) = body {
            request = request.json(&body);
        }
        let mut response = request.send().await.map_err(|_| {
            invalid(
                "management_unknown",
                "管理请求未完成，可能已有副作用，请核对原计划",
            )
        })?;
        let status = response.status();
        let mut bytes = vec![];
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| invalid("management_unknown", "管理结果读取中断，须核对"))?
        {
            if bytes.len() + chunk.len() > 4 * 1024 * 1024 {
                return Err(invalid("management_limit", "管理响应过大"));
            }
            bytes.extend_from_slice(&chunk);
        }
        self.unchanged()?;
        if !status.is_success() {
            return Err(invalid(
                "management_rejected",
                format!("官方管理接口返回HTTP{}；错误正文未回显", status.as_u16()),
            ));
        }
        let value: Value = serde_json::from_slice(&bytes)?;
        if value.get("apiKey").is_some() {
            return Err(invalid(
                "management_redaction",
                "管理接口意外返回秘密字段，已拒绝回显",
            ));
        }
        Ok(value)
    }
    pub async fn add_key(&self, provider: &str, key: &str) -> ModelResult<()> {
        self.request(
            reqwest::Method::POST,
            "/api/providers/keys",
            Some(serde_json::json!({"name":provider,"key":key})),
        )
        .await?;
        Ok(())
    }
    pub async fn key_state(&self, provider: &str) -> ModelResult<KeyState> {
        let mut url = reqwest::Url::parse("http://127.0.0.1/api/providers/keys")
            .map_err(|_| invalid("key_state", "密钥状态请求无效"))?;
        url.query_pairs_mut().append_pair("name", provider);
        let path = format!("{}?{}", url.path(), url.query().unwrap_or(""));
        let value = self.request(reqwest::Method::GET, &path, None).await?;
        let active_id = match &value["activeId"] {
            Value::Null => None,
            Value::String(id) => Some(id.clone()),
            _ => return Err(invalid("key_state", "官方活动密钥身份无效")),
        };
        let rows = value["keys"]
            .as_array()
            .ok_or_else(|| invalid("key_state", "官方密钥集合无效"))?;
        let ids: std::collections::BTreeSet<String> = rows
            .iter()
            .map(|row| {
                row["id"]
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| invalid("key_state", "官方密钥身份缺失"))
            })
            .collect::<ModelResult<_>>()?;
        if ids.len() != rows.len()
            || ids
                .iter()
                .any(|id| id.len() != 8 || !id.bytes().all(|c| c.is_ascii_hexdigit()))
            || active_id.as_ref().is_some_and(|id| !ids.contains(id))
        {
            return Err(invalid("key_state", "官方密钥身份不能核对"));
        }
        Ok(KeyState { active_id, ids })
    }
    pub async fn activate_key(&self, provider: &str, id: &str) -> ModelResult<()> {
        self.request(
            reqwest::Method::PUT,
            "/api/providers/keys/active",
            Some(serde_json::json!({"name":provider,"id":id})),
        )
        .await?;
        Ok(())
    }
    pub async fn remove_key(&self, provider: &str, id: &str) -> ModelResult<()> {
        self.request(
            reqwest::Method::DELETE,
            "/api/providers/keys",
            Some(serde_json::json!({"name":provider,"id":id})),
        )
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{extract::State, http::HeaderMap, routing::get, Json, Router};
    use std::{
        collections::BTreeSet,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc, Mutex,
        },
    };
    #[derive(Clone)]
    struct Fixture {
        pid: u32,
        port: u16,
        secret: String,
        valid: bool,
        pool: Arc<Mutex<KeyState>>,
        calls: Arc<AtomicUsize>,
    }
    async fn health(State(state): State<Fixture>, headers: HeaderMap) -> (HeaderMap, Json<Value>) {
        let challenge = headers
            .get("x-opencodex-attestation-challenge")
            .unwrap()
            .to_str()
            .unwrap();
        let mut mac = Hmac::<Sha256>::new_from_slice(if state.valid {
            state.secret.as_bytes()
        } else {
            b"wrong-proof"
        })
        .unwrap();
        mac.update(
            format!(
                "opencodex-local-management-v1\n{challenge}\n{}\n{}",
                state.pid, state.port
            )
            .as_bytes(),
        );
        let mut response = HeaderMap::new();
        response.insert(
            "x-opencodex-attestation-proof",
            URL_SAFE_NO_PAD
                .encode(mac.finalize().into_bytes())
                .parse()
                .unwrap(),
        );
        (
            response,
            Json(serde_json::json!({"service":"opencodex","pid":state.pid})),
        )
    }
    fn admission(state: &Fixture, headers: &HeaderMap) {
        assert_eq!(
            headers.get("authorization").unwrap().to_str().unwrap(),
            format!("Bearer ocx_admin_{}", "t".repeat(43))
        );
        state.calls.fetch_add(1, Ordering::SeqCst);
    }
    async fn keys(State(state): State<Fixture>, headers: HeaderMap) -> Json<Value> {
        admission(&state, &headers);
        let pool = state.pool.lock().unwrap();
        Json(
            serde_json::json!({"activeId":pool.active_id,"keys":pool.ids.iter().map(|id|serde_json::json!({"id":id,"masked":"do-not-return-to-webview"})).collect::<Vec<_>>()}),
        )
    }
    async fn add(
        State(state): State<Fixture>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> Json<Value> {
        admission(&state, &headers);
        let key = body["key"].as_str().unwrap();
        let id = super::super::store::sha256(key.as_bytes()).trim_start_matches("sha256:")[..8]
            .to_string();
        let mut pool = state.pool.lock().unwrap();
        pool.ids.insert(id.clone());
        pool.active_id = Some(id.clone());
        Json(serde_json::json!({"ok":true,"id":id}))
    }
    async fn activate(
        State(state): State<Fixture>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> Json<Value> {
        admission(&state, &headers);
        let id = body["id"].as_str().unwrap();
        let mut pool = state.pool.lock().unwrap();
        assert!(pool.ids.contains(id));
        pool.active_id = Some(id.into());
        Json(serde_json::json!({"ok":true}))
    }
    async fn remove(
        State(state): State<Fixture>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> Json<Value> {
        admission(&state, &headers);
        let id = body["id"].as_str().unwrap();
        let mut pool = state.pool.lock().unwrap();
        pool.ids.remove(id);
        if pool.active_id.as_deref() == Some(id) {
            pool.active_id = pool.ids.iter().next().cloned();
        }
        Json(serde_json::json!({"ok":true}))
    }
    async fn fixture(
        root: &std::path::Path,
        valid: bool,
    ) -> (ProcessContext, Fixture, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let port = listener.local_addr().unwrap().port();
        let state = Fixture {
            pid: std::process::id(),
            port,
            secret: "s".repeat(43),
            valid,
            pool: Arc::new(Mutex::new(KeyState {
                active_id: Some("aabbccdd".into()),
                ids: BTreeSet::from(["aabbccdd".into()]),
            })),
            calls: Arc::new(AtomicUsize::new(0)),
        };
        let context = ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(
                root.join("unused-ocx"),
            ),
            working_directory: root.into(),
            opencodex_home: root.into(),
        };
        crate::infrastructure::atomic_write::atomic_write(
            &root.join("runtime-port.json"),
            &serde_json::to_vec(
                &serde_json::json!({"pid":state.pid,"port":port,"attestationSecret":state.secret}),
            )
            .unwrap(),
            0o600,
        )
        .unwrap();
        crate::infrastructure::atomic_write::atomic_write(
            &root.join("admin-api-token"),
            format!("ocx_admin_{}", "t".repeat(43)).as_bytes(),
            0o600,
        )
        .unwrap();
        let router = Router::new()
            .route("/healthz", get(health))
            .route("/api/providers/keys", get(keys).post(add).delete(remove))
            .route("/api/providers/keys/active", axum::routing::put(activate))
            .with_state(state.clone());
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        (context, state, server)
    }
    #[tokio::test]
    async fn official_key_pool_selection_can_be_verified_and_restored_without_exporting_secrets() {
        let root = tempfile::tempdir().unwrap();
        let (context, state, server) = fixture(root.path(), true).await;
        let manager = Management::bind(&context).await.unwrap();
        let before = manager.key_state("test").await.unwrap();
        manager
            .add_key("test", "replacement-key-fixture")
            .await
            .unwrap();
        let after = manager.key_state("test").await.unwrap();
        assert_eq!(after.ids.len(), 2);
        assert_ne!(after.active_id, before.active_id);
        manager
            .activate_key("test", before.active_id.as_deref().unwrap())
            .await
            .unwrap();
        manager
            .remove_key("test", after.active_id.as_deref().unwrap())
            .await
            .unwrap();
        assert_eq!(manager.key_state("test").await.unwrap(), before);
        let public = serde_json::to_string(&after).unwrap();
        assert!(!public.contains("replacement-key") && !public.contains("masked"));
        assert!(state.calls.load(Ordering::SeqCst) > 0);
        server.abort();
    }
    #[tokio::test]
    async fn wrong_listener_proof_and_changed_runtime_record_never_receive_admin_credentials() {
        let root = tempfile::tempdir().unwrap();
        let (context, state, server) = fixture(root.path(), false).await;
        assert!(Management::bind(&context).await.is_err());
        assert_eq!(state.calls.load(Ordering::SeqCst), 0);
        server.abort();
        let second = tempfile::tempdir().unwrap();
        let (context, state, server) = fixture(second.path(), true).await;
        let manager = Management::bind(&context).await.unwrap();
        crate::infrastructure::atomic_write::atomic_write(&second.path().join("runtime-port.json"),&serde_json::to_vec(&serde_json::json!({"pid":state.pid+1,"port":state.port,"attestationSecret":state.secret})).unwrap(),0o600).unwrap();
        assert!(manager.key_state("test").await.is_err());
        assert_eq!(state.calls.load(Ordering::SeqCst), 0);
        server.abort();
    }
}
