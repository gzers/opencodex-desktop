//! Authenticated official management requests bind to the protected runtime record.
//! Reusable admin credentials are read only after the process proves the record key.
use super::schema::*;
use crate::{modules::backup::safety, state::ProcessContext};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::Value;
use sha2::Sha256;
use std::time::Duration;
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
            "/api/providers",
            "/api/disabled-models",
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
}
