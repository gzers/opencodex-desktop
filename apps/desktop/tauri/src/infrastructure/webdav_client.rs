//! 受控 WebDAV 网络客户端。
//!
//! reqwest 保持默认证书校验；客户端只提供 PROPFIND / GET / PUT 三个边界。
//! Basic 凭据只在请求头中短期使用，错误不携带 URL query、密码或 Cookie。
//!
//! 远端布局（**不新增 MKCOL 边界**）：所有对象都直接落在用户配置的远端路径下，
//! 不依赖任何中间集合：
//! - `latest.txt` — 指向当前有效快照 id；
//! - `<snapshot_id>.ocxd` — 该快照的清单；
//! - `<sha256>.ocxd` — 内容寻址的载荷（同一份内容在多快照间共享）。
//!
//! 此前把清单与载荷放在 `<snapshot_id>/payload/` 子集合下，但客户端只做 PUT、
//! 不建集合，标准 WebDAV 服务器（如 wsgidav / Apache mod_dav）会对「PUT 到不存在的
//! 集合」返回 409，导致真实同步上传 100% 失败。展平后只需要用户配置的远端目录存在，
//! 与冻结的 PROPFIND/GET/PUT 边界一致。

use async_trait::async_trait;
use reqwest::{Client, Method, StatusCode};
use std::time::Duration;

use crate::modules::sync::{connect_timeout, item_timeout, TlsFailureStage};

pub const MANIFEST_FILE_NAME: &str = "manifest.ocxd";
pub const LATEST_FILE_NAME: &str = "latest.txt";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebDavConfig {
    pub base_url: String,
    pub remote_path: String,
    pub username: String,
    pub password: String,
}

impl WebDavConfig {
    pub fn validate(&self) -> Result<(), WebDavError> {
        if self.base_url.trim().is_empty() || !self.base_url.starts_with("https://") {
            return Err(WebDavError::InvalidConfig);
        }
        let remote = normalize_remote_path(&self.remote_path)?;
        if remote.is_empty() {
            return Err(WebDavError::InvalidConfig);
        }
        if self.username.trim().is_empty() || self.password.is_empty() {
            return Err(WebDavError::InvalidConfig);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDavOperation {
    Probe,
    Download,
    Upload,
}

impl WebDavOperation {
    fn tls_stage(self) -> TlsFailureStage {
        match self {
            Self::Probe => TlsFailureStage::Connect,
            Self::Download => TlsFailureStage::Download,
            Self::Upload => TlsFailureStage::Upload,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDavError {
    InvalidConfig,
    Network,
    Unauthorized,
    NotFound,
    Protocol,
    Tls,
}

impl WebDavError {
    pub fn user_message(self, operation: WebDavOperation) -> &'static str {
        match self {
            Self::Tls => crate::modules::sync::tls_failure_message(operation.tls_stage()),
            Self::InvalidConfig => "WebDAV 端点配置不完整；已停止请求。",
            Self::Unauthorized => "WebDAV 认证失败；本地内容未修改。",
            Self::NotFound => "WebDAV 资源不存在；请确认远端目录。",
            Self::Protocol => "WebDAV 服务响应不符合协议；本地内容未修改。",
            Self::Network => "WebDAV 网络请求失败；本地内容未修改。",
        }
    }

    pub fn retryable(self) -> bool {
        matches!(self, Self::Network)
    }
}

fn normalize_remote_path(path: &str) -> Result<String, WebDavError> {
    let trimmed = path.trim().trim_matches('/');
    if trimmed.is_empty()
        || trimmed
            .split('/')
            .any(|part| part == ".." || part.is_empty())
    {
        return Err(WebDavError::InvalidConfig);
    }
    for byte in trimmed.bytes() {
        if !(byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/')) {
            return Err(WebDavError::InvalidConfig);
        }
    }
    Ok(trimmed.to_string())
}

fn encoded_remote_path(path: &str) -> String {
    path.split('/')
        .map(percent_encode_segment)
        .collect::<Vec<_>>()
        .join("/")
}

fn percent_encode_segment(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn trim_base_url(base_url: &str) -> Result<&str, WebDavError> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() || !trimmed.starts_with("https://") {
        return Err(WebDavError::InvalidConfig);
    }
    Ok(trimmed)
}

fn map_status(status: StatusCode) -> WebDavError {
    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => WebDavError::Unauthorized,
        // 404：目标不存在；409：父集合不存在（WebDAV 语义为「中间集合缺失」）。
        // 两者对用户都是「远端目录/资源不存在」，而非协议错误或无意义的通用失败。
        StatusCode::NOT_FOUND | StatusCode::CONFLICT => WebDavError::NotFound,
        status if status.is_client_error() || status.is_server_error() => WebDavError::Protocol,
        _ => WebDavError::Protocol,
    }
}

fn map_request_error(error: &reqwest::Error, operation: WebDavOperation) -> WebDavError {
    if error.is_builder() {
        WebDavError::InvalidConfig
    } else if error.is_decode() {
        WebDavError::Protocol
    } else if error.is_connect() && is_certificate_error(error) {
        WebDavError::Tls
    } else {
        let _ = operation;
        WebDavError::Network
    }
}

/// 只有确实来自 **TLS 证书校验** 的连接失败才归 `Tls`。
///
/// 此前把**所有** `is_connect()` 失败都报成「TLS 证书校验失败；请确认服务器证书…」，
/// 真机制品验证中把「测试服务未启动（连接被拒）」误报成证书问题，把排查方向带偏。
/// 判据取错误链上是否出现证书/链路校验字样；连接被拒、不可达等传输错误留给 `Network`
/// （其文案与 `retryable()` 语义都已就位）。
fn is_certificate_error(error: &reqwest::Error) -> bool {
    const MARKERS: [&str; 4] = [
        "invalid peer certificate",
        "UnknownIssuer",
        "CertificateError",
        "certificate",
    ];
    let mut source = std::error::Error::source(error);
    while let Some(cause) = source {
        let text = cause.to_string();
        if MARKERS.iter().any(|marker| text.contains(marker)) {
            return true;
        }
        source = cause.source();
    }
    false
}

/// 传输抽象：生产实现封装 reqwest；测试可注入确定性 HTTP 语义。
#[async_trait]
pub trait WebDavTransport: Send + Sync {
    async fn request(
        &self,
        config: &WebDavConfig,
        method: WebDavMethod,
        path: &str,
        body: Option<Vec<u8>>,
    ) -> Result<WebDavResponse, WebDavError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDavMethod {
    Propfind,
    Get,
    Put,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebDavResponse {
    pub status: u16,
    pub body: Vec<u8>,
    pub etag: Option<String>,
}

pub struct ReqwestWebDavTransport {
    client: Client,
}

impl ReqwestWebDavTransport {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .connect_timeout(connect_timeout())
                .timeout(item_timeout())
                .build()
                .expect("WebDAV client should use frozen timeouts"),
        }
    }
}

impl Default for ReqwestWebDavTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WebDavTransport for ReqwestWebDavTransport {
    async fn request(
        &self,
        config: &WebDavConfig,
        method: WebDavMethod,
        path: &str,
        body: Option<Vec<u8>>,
    ) -> Result<WebDavResponse, WebDavError> {
        let remote = normalize_remote_path(&config.remote_path)?;
        let base = trim_base_url(&config.base_url)?;
        let remote_segment = encoded_remote_path(&remote);
        let url = match path {
            "" => format!("{base}/{remote_segment}/"),
            _ => format!("{base}/{remote_segment}/{}", encoded_remote_path(path)),
        };
        let request = match method {
            WebDavMethod::Propfind => self
                .client
                .request(
                    Method::from_bytes(b"PROPFIND").expect("PROPFIND is valid"),
                    &url,
                )
                .header("Depth", "0")
                .header("Content-Type", "application/xml"),
            WebDavMethod::Get => self.client.get(&url),
            WebDavMethod::Put => {
                let request = self.client.put(&url);
                match body.as_deref() {
                    Some(payload) => request.body(payload.to_vec()),
                    None => request,
                }
            }
        };
        let response = request
            .basic_auth(config.username.trim(), Some(&config.password))
            .send()
            .await
            .map_err(|error| map_request_error(&error, WebDavOperation::Probe))?;
        let status = response.status();
        let etag = response
            .headers()
            .get("etag")
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());
        let payload = response
            .bytes()
            .await
            .map_err(|error| map_request_error(&error, WebDavOperation::Probe))?
            .to_vec();
        if !status.is_success() {
            return Err(map_status(status));
        }
        Ok(WebDavResponse {
            status: status.as_u16(),
            body: payload,
            etag,
        })
    }
}

/// 顺序化 WebDAV 操作：探测目录 → 逐件上传 → manifest 最后上传。
pub struct WebDavClient<T = ReqwestWebDavTransport>
where
    T: WebDavTransport,
{
    transport: T,
    total_timeout: Duration,
}

impl WebDavClient<ReqwestWebDavTransport> {
    pub fn production() -> Self {
        Self::new(ReqwestWebDavTransport::new())
    }
}

impl<T> WebDavClient<T>
where
    T: WebDavTransport,
{
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            total_timeout: crate::modules::sync::total_timeout(),
        }
    }

    pub fn total_timeout(&self) -> Duration {
        self.total_timeout
    }

    pub async fn test_connection(&self, config: &WebDavConfig) -> Result<(), WebDavError> {
        config.validate()?;
        self.transport
            .request(config, WebDavMethod::Propfind, "", None)
            .await
            .map(|_| ())
    }

    /// 读取固定 latest 指针；404 表示当前远端尚无有效快照。
    pub async fn download_latest_pointer(
        &self,
        config: &WebDavConfig,
    ) -> Result<Option<String>, WebDavError> {
        config.validate()?;
        match self
            .transport
            .request(config, WebDavMethod::Get, LATEST_FILE_NAME, None)
            .await
        {
            Ok(response) => {
                let value = String::from_utf8(response.body).map_err(|_| WebDavError::Protocol)?;
                let value = value.trim();
                if value.is_empty() {
                    return Ok(None);
                }
                validate_snapshot_id(value).map_err(|_| WebDavError::Protocol)?;
                Ok(Some(value.to_string()))
            }
            Err(WebDavError::NotFound) => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub async fn download(
        &self,
        config: &WebDavConfig,
        snapshot_id: &str,
    ) -> Result<Vec<u8>, WebDavError> {
        config.validate()?;
        validate_snapshot_id(snapshot_id)?;
        let manifest_path = manifest_remote_path(snapshot_id);
        let response = self
            .transport
            .request(config, WebDavMethod::Get, &manifest_path, None)
            .await?;
        Ok(response.body)
    }

    pub async fn download_payload(
        &self,
        config: &WebDavConfig,
        snapshot_id: &str,
        artifact_sha256: &str,
    ) -> Result<Vec<u8>, WebDavError> {
        config.validate()?;
        validate_snapshot_id(snapshot_id)?;
        validate_sha256(artifact_sha256)?;
        let path = payload_remote_path(artifact_sha256);
        let response = self
            .transport
            .request(config, WebDavMethod::Get, &path, None)
            .await?;
        Ok(response.body)
    }

    pub async fn upload(
        &self,
        config: &WebDavConfig,
        snapshot_id: &str,
        artifacts: &[(String, String, Vec<u8>)],
        manifest: &[u8],
    ) -> Result<Option<String>, WebDavError> {
        config.validate()?;
        validate_snapshot_id(snapshot_id)?;
        for (name, digest, payload) in artifacts {
            if name.trim().is_empty() || name.contains("..") {
                return Err(WebDavError::InvalidConfig);
            }
            validate_sha256(digest)?;
            if payload.len() as u64 > crate::modules::sync::PAYLOAD_MAX_BYTES {
                return Err(WebDavError::InvalidConfig);
            }
        }
        if manifest.len() as u64 > crate::modules::sync::MANIFEST_MAX_BYTES {
            return Err(WebDavError::InvalidConfig);
        }
        // 载荷文件名必须与下载端使用的清单 `sha256` 一致（明文摘要），
        // 而不是传输封装字节的摘要——否则下载端会按另一个名字去取，永远 404，
        // 远端内容永远无法应用。
        for (_name, digest, payload) in artifacts {
            let path = payload_remote_path(digest);
            let response = self
                .transport
                .request(config, WebDavMethod::Put, &path, Some(payload.clone()))
                .await?;
            if response.status != 201 && response.status != 204 {
                return Err(WebDavError::Protocol);
            }
        }
        let response = self
            .transport
            .request(
                config,
                WebDavMethod::Put,
                &manifest_remote_path(snapshot_id),
                Some(manifest.to_vec()),
            )
            .await?;
        if response.status != 201 && response.status != 204 {
            return Err(WebDavError::Protocol);
        }
        let pointer = format!("{}\n", snapshot_id);
        self.transport
            .request(
                config,
                WebDavMethod::Put,
                LATEST_FILE_NAME,
                Some(pointer.into_bytes()),
            )
            .await?;
        Ok(response.etag)
    }
}

pub fn manifest_remote_path(snapshot_id: &str) -> String {
    format!("{snapshot_id}.ocxd")
}

/// 载荷按内容寻址，直接放在远端目录下，避免依赖中间集合。
pub fn payload_remote_path(artifact_sha256: &str) -> String {
    format!("{artifact_sha256}.ocxd")
}

fn validate_snapshot_id(value: &str) -> Result<(), WebDavError> {
    if value.len() == 5 + 14 + 1 + 12
        && value.starts_with("snap_")
        && value[5..]
            .chars()
            .all(|c| c.is_ascii_digit() || c == '_' || c.is_ascii_hexdigit())
    {
        Ok(())
    } else {
        Err(WebDavError::InvalidConfig)
    }
}

fn validate_sha256(value: &str) -> Result<(), WebDavError> {
    if value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(WebDavError::InvalidConfig)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct FixtureTransport {
        requests: Arc<Mutex<Vec<(WebDavMethod, String, usize)>>>,
        unauthorized: bool,
    }

    #[async_trait]
    impl WebDavTransport for FixtureTransport {
        async fn request(
            &self,
            _config: &WebDavConfig,
            method: WebDavMethod,
            path: &str,
            body: Option<Vec<u8>>,
        ) -> Result<WebDavResponse, WebDavError> {
            self.requests.lock().unwrap().push((
                method,
                path.to_string(),
                body.map(|v| v.len()).unwrap_or_default(),
            ));
            if self.unauthorized {
                return Err(WebDavError::Unauthorized);
            }
            Ok(WebDavResponse {
                status: if method == WebDavMethod::Put {
                    201
                } else {
                    200
                },
                body: b"fixture".to_vec(),
                etag: method
                    .eq(&WebDavMethod::Put)
                    .then(|| "\"etag-1\"".to_string()),
            })
        }
    }

    fn config() -> WebDavConfig {
        WebDavConfig {
            base_url: "https://dav.example.test".to_string(),
            remote_path: "/desktop-sync/current".to_string(),
            username: "user@example".to_string(),
            password: "secret-password".to_string(),
        }
    }

    #[tokio::test]
    async fn test_connection_uses_propfind_with_basic_auth() {
        let transport = FixtureTransport::default();
        let requests = transport.requests.clone();
        let client = WebDavClient::new(transport);
        client.test_connection(&config()).await.unwrap();
        let requests = requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].0, WebDavMethod::Propfind);
        assert_eq!(requests[0].1, "");
    }

    #[tokio::test]
    async fn upload_puts_payloads_before_manifest() {
        let transport = FixtureTransport::default();
        let requests = transport.requests.clone();
        let client = WebDavClient::new(transport);
        let etag = client
            .upload(
                &config(),
                "snap_20260917000000_aaaaaaaaaaaa",
                &[(
                    "a".to_string(),
                    crate::infrastructure::hash::sha256_hex(b"one"),
                    b"one".to_vec(),
                )],
                b"manifest",
            )
            .await
            .unwrap();
        assert_eq!(etag.as_deref(), Some("\"etag-1\""));
        let requests = requests.lock().unwrap();
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].0, WebDavMethod::Put);
        // 载荷必须直接落在远端目录下（内容寻址），不得依赖任何中间集合。
        assert_eq!(
            requests[0].1,
            format!("{}.ocxd", crate::infrastructure::hash::sha256_hex(b"one"))
        );
        assert!(!requests[0].1.contains('/'));
        assert_eq!(requests[1].1, "snap_20260917000000_aaaaaaaaaaaa.ocxd");
        assert_eq!(requests[2].1, "latest.txt");
    }

    /// 回归：载荷文件名必须来自清单里的明文 `sha256`，而不是传输封装字节的摘要。
    /// 此前上传按封装字节摘要命名、下载按清单摘要取，两者不一致，远端内容永远 404。
    #[tokio::test]
    async fn payload_file_name_follows_manifest_digest_not_transport_bytes() {
        let transport = FixtureTransport::default();
        let requests = transport.requests.clone();
        let client = WebDavClient::new(transport);
        let manifest_digest = "ab".repeat(32);
        assert_ne!(
            manifest_digest,
            crate::infrastructure::hash::sha256_hex(b"one"),
            "测试前提：清单摘要与封装字节摘要必须不同"
        );
        client
            .upload(
                &config(),
                "snap_20260917000000_aaaaaaaaaaaa",
                &[("a".to_string(), manifest_digest.clone(), b"one".to_vec())],
                b"manifest",
            )
            .await
            .unwrap();
        let requests = requests.lock().unwrap();
        assert_eq!(requests[0].1, format!("{manifest_digest}.ocxd"));
    }

    #[tokio::test]
    async fn invalid_remote_path_is_rejected_before_network() {
        let transport = FixtureTransport::default();
        let requests = transport.requests.clone();
        let client = WebDavClient::new(transport);
        let mut invalid = config();
        invalid.base_url = "http://insecure.example.test".to_string();
        assert_eq!(
            client.test_connection(&invalid).await,
            Err(WebDavError::InvalidConfig)
        );
        assert!(requests.lock().unwrap().is_empty());
    }

    #[test]
    fn tls_error_has_frozen_message() {
        assert_eq!(
            WebDavError::Tls.user_message(WebDavOperation::Upload),
            "TLS 证书校验失败，已取消上传；远端内容未修改。"
        );
    }

    #[test]
    fn missing_parent_collection_is_reported_as_not_found() {
        // 标准 WebDAV 服务器对「PUT/GET 到不存在的集合」返回 409；
        // 这必须是「远端不存在」而不是伪装成协议错误或成功。
        assert_eq!(map_status(StatusCode::CONFLICT), WebDavError::NotFound);
        assert_eq!(map_status(StatusCode::NOT_FOUND), WebDavError::NotFound);
        assert_eq!(
            map_status(StatusCode::UNAUTHORIZED),
            WebDavError::Unauthorized
        );
    }

    #[test]
    fn remote_layout_has_no_intermediate_collections() {
        let digest = "ab".repeat(32);
        assert_eq!(
            manifest_remote_path("snap_20260920030100_abcdef123456"),
            "snap_20260920030100_abcdef123456.ocxd"
        );
        let payload = payload_remote_path(&digest);
        assert_eq!(payload, format!("{digest}.ocxd"));
        assert!(!payload.contains('/'), "载荷路径不得依赖中间集合");
    }

    #[test]
    fn remote_path_rejects_escape_and_empty_segments() {
        assert_eq!(
            normalize_remote_path("/a/../b"),
            Err(WebDavError::InvalidConfig)
        );
        assert_eq!(
            normalize_remote_path("a//b"),
            Err(WebDavError::InvalidConfig)
        );
        assert_eq!(
            normalize_remote_path("/desktop-sync/current"),
            Ok("desktop-sync/current".to_string())
        );
    }

    /// 回归（2026-10-01 真机制品验证）：连接被拒曾被报成「TLS 证书校验失败」。
    /// 127.0.0.1:1 上不会有服务监听，必须归 `Network`（可重试），而不是 `Tls`。
    #[test]
    fn refused_connection_is_network_not_tls() {
        let transport = ReqwestWebDavTransport::new();
        let config = WebDavConfig {
            base_url: "https://127.0.0.1:1".to_string(),
            remote_path: "ocx-smoke".to_string(),
            username: "u".to_string(),
            password: "p".to_string(),
        };
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let error = runtime
            .block_on(transport.request(&config, WebDavMethod::Propfind, "", None))
            .expect_err("connection must fail");
        assert_eq!(error, WebDavError::Network);
        assert!(error.retryable());
        assert!(error
            .user_message(WebDavOperation::Probe)
            .contains("网络请求失败"));
    }
}
