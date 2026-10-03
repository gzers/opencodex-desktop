//! MOD-05：FZ-16 ~ FZ-20 加密容器头、明文白名单与导入上限契约。
//!
//! 本模块只冻结解析与校验边界，不执行真实加密解密、不访问真实
//! 数据根、Keychain、WebDAV 或用户配置。测试只使用内存结构。

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// FZ-16 容器头与 FZ-17/18 冻结参数。
pub const CONTAINER_MAGIC: &str = "OCXDCONF";
/// 旧版：口令保护的加密容器（Argon2id + AES-256-GCM）。仅兼容读取。
pub const CONTAINER_FORMAT_VERSION: u32 = 1;
/// 新版：不含额外口令的明文配置容器；用 `INTEGRITY_ALGORITHM` 检测损坏。
pub const CONTAINER_FORMAT_VERSION_PLAINTEXT: u32 = 2;
/// 明文容器的完整性校验算法（只检测损坏，不是防篡改身份认证）。
pub const INTEGRITY_ALGORITHM: &str = "sha-256";

/// 导出容器明确**不包含**的内容；UI、CLI 与规范必须与之一致，不得暗示端到端加密。
pub const EXPORT_EXCLUSIONS: [&str; 4] = [
    "系统钥匙串中的口令密文（WebDAV 口令、旧版加密口令）",
    "MCP 环境变量值的本体（只导出 keychain 引用）",
    "Skills 目录内的文件内容（只导出名称清单）",
    "官方 OpenCodex 运行配置（提供方、路由、模型映射）",
];
pub const STRUCTURE_VERSION: u32 = 1;
pub const KDF_ALGORITHM: &str = "argon2id";
pub const KDF_SALT_BYTES: usize = 16;
pub const KDF_MEMORY_KIB: u32 = 256 * 1024;
pub const KDF_ITERATIONS: u32 = 3;
pub const KDF_PARALLELISM: u32 = 2;
pub const KDF_OUTPUT_BYTES: u32 = 32;
pub const AEAD_ALGORITHM: &str = "aes-256-gcm";
pub const AEAD_NONCE_BYTES: usize = 12;

/// FZ-20 导入硬上限。
pub const CIPHERTEXT_MAX_BYTES: u64 = 512 * 1024 * 1024;
pub const PLAINTEXT_MAX_BYTES: u64 = 128 * 1024 * 1024;
pub const JSON_PARSE_BUFFER_MAX_BYTES: u64 = 256 * 1024 * 1024;
pub const ARRAY_MAX_ITEMS: usize = 20_000;
pub const MAX_DEPTH: usize = 64;
pub const SINGLE_FILE_MAX_BYTES: u64 = 32 * 1024 * 1024;
pub const ARCHIVE_ENTRY_MAX_ITEMS: usize = 50_000;
pub const ARCHIVE_TOTAL_MAX_BYTES: u64 = 1024 * 1024 * 1024;
pub const SINGLE_THREAD_MEMORY_MAX_BYTES: u64 = 512 * 1024 * 1024;

/// 容器头字段按 FZ-16 冻结。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ContainerHeader {
    pub magic: String,
    pub format_version: u32,
    pub kdf: KdfHeader,
    pub aead: AeadHeader,
    pub created_at: String,
    pub app_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct KdfHeader {
    pub algorithm: String,
    pub salt_b64: String,
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
    pub output_bytes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AeadHeader {
    pub algorithm: String,
    pub nonce_bytes: u32,
}

/// 解密后 JSON 的顶层结构。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct ContainerDocument {
    pub container: ContainerInfo,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extension_config: Option<ExtensionSection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sync_endpoints: Option<Vec<SyncEndpoint>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferences: Option<BTreeMap<String, PreferenceValue>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct ContainerInfo {
    pub structure_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct ExtensionSection {
    #[serde(default)]
    pub skills: Vec<ContainerSkill>,
    /// 旧版：MCP 定义快照（含命令/参数/环境变量引用）。只保留兼容读取。
    #[serde(default)]
    pub servers: Vec<ContainerServer>,
    /// 管理器统一配置里的 MCP 服务器**名称**清单；定义仍由各客户端文件承载。
    #[serde(default)]
    pub server_names: Vec<String>,
    /// 自定义 Skills 源目录（机器相关路径，导入时不套用，仅用于对账）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_store: Option<String>,
    /// 分发方式（`symlink` / `copy`），与机器无关，可导入。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sync_method: Option<String>,
    /// 各客户端总开关；与机器无关，可导入。
    #[serde(default)]
    pub enablement: BTreeMap<crate::modules::extensions::ClientId, bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct ContainerSkill {
    pub name: String,
    pub source_path: ImportPath,
    pub client_ids: Vec<crate::modules::extensions::ClientId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct ContainerServer {
    pub name: String,
    /// 传输类型（官方客户端的 `transport` 字段原文，如 `stdio`）。仅作对账。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport: Option<String>,
    pub command: ServerCommand,
    /// 旧版字段：环境变量名 → 钥匙串引用。仅为兼容读取保留，新导出不再写入。
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, SecretReference>,
    /// 环境变量**名称**清单；值本体不在容器内（见 `EXPORT_EXCLUSIONS`）。
    #[serde(default)]
    pub env_keys: Vec<String>,
    pub client_ids: Vec<crate::modules::extensions::ClientId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerCommand {
    Single(String),
    WithArgs { command: String, args: Vec<String> },
}

impl ServerCommand {
    pub fn command_text(&self) -> &str {
        match self {
            Self::Single(command) => command,
            Self::WithArgs { command, .. } => command,
        }
    }

    pub fn args(&self) -> &[String] {
        match self {
            Self::Single(_) => &[],
            Self::WithArgs { args, .. } => args,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PreferenceValue {
    Boolean(bool),
    Integer(i64),
    Text(String),
}

pub type PreferenceMap = BTreeMap<String, PreferenceValue>;

/// FZ-19：只接受数据根相对路径，拒绝 `..`、绝对路径和组件逃逸。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct ImportPath(PathBuf);

impl ImportPath {
    pub fn parse(value: &str) -> Result<Self, ContainerError> {
        if value.is_empty()
            || value.starts_with('/')
            || value.contains('\\')
            || value.contains('\0')
        {
            return Err(ContainerError::PathRejected);
        }
        let path = PathBuf::from(value);
        let mut depth = 0usize;
        for component in path.components() {
            match component {
                std::path::Component::Normal(part) => {
                    if part.is_empty() || part == "." || part == ".." {
                        return Err(ContainerError::PathRejected);
                    }
                    depth += 1;
                    if depth > MAX_DEPTH {
                        return Err(ContainerError::DepthExceeded);
                    }
                }
                _ => return Err(ContainerError::PathRejected),
            }
        }
        Ok(Self(path))
    }

    pub fn as_path(&self) -> &std::path::Path {
        &self.0
    }
}

impl TryFrom<String> for ImportPath {
    type Error = ContainerError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

/// FZ-19：凭据只允许 `keychain://` 引用，不携带真实值。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "SecretReferenceValue")]
pub struct SecretReference {
    pub ref_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
enum SecretReferenceValue {
    String(String),
    Object { ref_id: String },
}

impl SecretReference {
    pub fn parse(value: &str) -> Result<Self, ContainerError> {
        let reference = value
            .strip_prefix("keychain://")
            .ok_or(ContainerError::SecretRejected)?;
        if reference.is_empty()
            || !reference
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(ContainerError::SecretRejected);
        }
        Ok(Self {
            ref_id: reference.to_string(),
        })
    }

    pub fn as_ref_id(&self) -> &str {
        &self.ref_id
    }
}

impl TryFrom<String> for SecretReference {
    type Error = ContainerError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<SecretReferenceValue> for SecretReference {
    fn from(value: SecretReferenceValue) -> Self {
        match value {
            SecretReferenceValue::String(reference) => Self::parse(&reference)
                .expect("secret reference parser should accept stored string"),
            SecretReferenceValue::Object { ref_id } => Self { ref_id },
        }
    }
}

/// 端点条目冻结可用配置与脱敏边界：口令始终只在系统钥匙串，容器只携带引用。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SyncEndpoint {
    pub name: String,
    pub credential: SecretReference,
    /// 服务端地址（`https://…`）；旧版容器没有该字段。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// 远端路径前缀。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_path: Option<String>,
    /// 服务端认证账号（不是秘密；口令始终只在钥匙串）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// TLS 策略；当前只允许校验必需。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_policy: Option<String>,
    /// 冲突策略。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conflict_policy: Option<String>,
}

/// 头部或明文校验失败分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ContainerError {
    #[error("container header is invalid or incompatible")]
    HeaderInvalid,
    #[error("container format version is newer than supported version 1")]
    HeaderFutureVersion,
    #[error("credential reference is rejected")]
    SecretRejected,
    #[error("import path is rejected")]
    PathRejected,
    #[error("import nesting depth exceeds limit")]
    DepthExceeded,
    #[error("import payload exceeds frozen limit")]
    LimitExceeded,
}

/// 校验容器头；不派生密钥。
pub fn validate_header(header: &ContainerHeader) -> Result<(), ContainerError> {
    if header.magic != CONTAINER_MAGIC {
        return Err(ContainerError::HeaderInvalid);
    }
    if header.format_version != CONTAINER_FORMAT_VERSION {
        return Err(ContainerError::HeaderFutureVersion);
    }
    if header.kdf.algorithm != KDF_ALGORITHM
        || header.kdf.memory_kib != KDF_MEMORY_KIB
        || header.kdf.iterations != KDF_ITERATIONS
        || header.kdf.parallelism != KDF_PARALLELISM
        || header.kdf.output_bytes != KDF_OUTPUT_BYTES
    {
        return Err(ContainerError::HeaderInvalid);
    }
    if header.aead.algorithm != AEAD_ALGORITHM
        || header.aead.nonce_bytes as usize != AEAD_NONCE_BYTES
    {
        return Err(ContainerError::HeaderInvalid);
    }
    if DateTime::parse_from_rfc3339(&header.created_at).is_err() {
        return Err(ContainerError::HeaderInvalid);
    }
    if header.app_version.trim().is_empty() {
        return Err(ContainerError::HeaderInvalid);
    }
    Ok(())
}

/// 校验解密明文的结构、数组数量与路径/凭据引用。
pub fn validate_document(document: &ContainerDocument) -> Result<(), ContainerError> {
    if document.container.structure_version != STRUCTURE_VERSION {
        return Err(ContainerError::HeaderFutureVersion);
    }
    if let Some(extension) = &document.extension_config {
        if extension.skills.len() > ARRAY_MAX_ITEMS || extension.servers.len() > ARRAY_MAX_ITEMS {
            return Err(ContainerError::LimitExceeded);
        }
        if extension.server_names.len() > ARRAY_MAX_ITEMS {
            return Err(ContainerError::LimitExceeded);
        }
        for name in &extension.server_names {
            if name.trim().is_empty() || name.len() > 200 {
                return Err(ContainerError::PathRejected);
            }
        }
        if let Some(method) = &extension.sync_method {
            if method != "symlink" && method != "copy" {
                return Err(ContainerError::PathRejected);
            }
        }
        if let Some(store) = &extension.source_store {
            // 源目录是机器相关路径；只做基本形态校验，导入时不套用。
            if store.trim().is_empty() || store.len() > 4096 || !store.starts_with('/') {
                return Err(ContainerError::PathRejected);
            }
        }
        for server in &extension.servers {
            if server.name.trim().is_empty() || server.name.len() > 200 {
                return Err(ContainerError::PathRejected);
            }
            if let Some(transport) = &server.transport {
                if transport.trim().is_empty() || transport.len() > 40 {
                    return Err(ContainerError::PathRejected);
                }
            }
            if server.command.command_text().trim().is_empty() {
                return Err(ContainerError::PathRejected);
            }
            if server.command.args().len() > 200 {
                return Err(ContainerError::LimitExceeded);
            }
            for key in &server.env_keys {
                if !is_env_key(key) {
                    return Err(ContainerError::PathRejected);
                }
            }
        }
    }
    if let Some(endpoints) = &document.sync_endpoints {
        if endpoints.len() > ARRAY_MAX_ITEMS {
            return Err(ContainerError::LimitExceeded);
        }
        for endpoint in endpoints {
            if endpoint.name.trim().is_empty() || endpoint.name.len() > 200 {
                return Err(ContainerError::PathRejected);
            }
            if let Some(url) = &endpoint.url {
                // 与同步端点配置同一口径：只接受 https，且不带内联凭据。
                if !url.starts_with("https://")
                    || url.contains('@')
                    || url.contains('?')
                    || url.contains('#')
                {
                    return Err(ContainerError::PathRejected);
                }
            }
            if let Some(remote_path) = &endpoint.remote_path {
                if remote_path.trim().is_empty() || remote_path.contains("..") {
                    return Err(ContainerError::PathRejected);
                }
            }
            if let Some(tls_policy) = &endpoint.tls_policy {
                if tls_policy != "verify_required" {
                    return Err(ContainerError::PathRejected);
                }
            }
            if let Some(policy) = &endpoint.conflict_policy {
                if !matches!(
                    policy.as_str(),
                    "ask" | "keep-local" | "keep-remote" | "keep-both"
                ) {
                    return Err(ContainerError::PathRejected);
                }
            }
        }
    }
    Ok(())
}

/// 环境变量名：`[A-Za-z_][A-Za-z0-9_]*`，最长 200。
fn is_env_key(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return false,
    }
    value.len() <= 200 && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// 使用冻结参数构造新容器头。
pub fn new_header(
    now: DateTime<Utc>,
    app_version: impl Into<String>,
    salt_b64: impl Into<String>,
) -> ContainerHeader {
    ContainerHeader {
        magic: CONTAINER_MAGIC.to_string(),
        format_version: CONTAINER_FORMAT_VERSION,
        kdf: KdfHeader {
            algorithm: KDF_ALGORITHM.to_string(),
            salt_b64: salt_b64.into(),
            memory_kib: KDF_MEMORY_KIB,
            iterations: KDF_ITERATIONS,
            parallelism: KDF_PARALLELISM,
            output_bytes: KDF_OUTPUT_BYTES,
        },
        aead: AeadHeader {
            algorithm: AEAD_ALGORITHM.to_string(),
            nonce_bytes: AEAD_NONCE_BYTES as u32,
        },
        created_at: now.to_rfc3339_opts(SecondsFormat::Secs, true),
        app_version: app_version.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::extensions::{ClientTarget, ExtensionConfig, TargetScope};
    use chrono::TimeZone;

    fn header() -> ContainerHeader {
        new_header(
            Utc.with_ymd_and_hms(2026, 9, 15, 0, 0, 0).unwrap(),
            "0.1.0",
            "aGVsbG8td29ybGQtMTIzNDU2",
        )
    }

    fn document() -> ContainerDocument {
        ContainerDocument {
            container: ContainerInfo {
                structure_version: 1,
            },
            extension_config: Some(ExtensionSection {
                skills: vec![ContainerSkill {
                    name: "demo".to_string(),
                    source_path: ImportPath::parse("exports/demo").unwrap(),
                    client_ids: vec![crate::modules::extensions::ClientId::Codex],
                }],
                server_names: vec!["demo".to_string()],
                source_store: None,
                sync_method: Some("symlink".to_string()),
                enablement: [(crate::modules::extensions::ClientId::Codex, true)]
                    .into_iter()
                    .collect(),
                servers: vec![ContainerServer {
                    name: "demo".to_string(),
                    transport: Some("stdio".to_string()),
                    command: ServerCommand::WithArgs {
                        command: "node".to_string(),
                        args: vec!["server.js".to_string()],
                    },
                    env: [(
                        "token".to_string(),
                        SecretReference::parse("keychain://ocx.dav.demo").unwrap(),
                    )]
                    .into_iter()
                    .collect(),
                    env_keys: vec!["token".to_string()],
                    client_ids: vec![crate::modules::extensions::ClientId::Codex],
                }],
            }),
            sync_endpoints: Some(vec![SyncEndpoint {
                name: "dav".to_string(),
                credential: SecretReference::parse("keychain://ocx.dav.demo").unwrap(),
                url: None,
                remote_path: None,
                username: None,
                tls_policy: None,
                conflict_policy: None,
            }]),
            preferences: Some(
                [(
                    "theme".to_string(),
                    PreferenceValue::Text("dark".to_string()),
                )]
                .into_iter()
                .collect(),
            ),
        }
    }

    #[test]
    fn header_fields_match_fz16_17_18() {
        let value = header();
        assert!(validate_header(&value).is_ok());
        assert_eq!(value.kdf.memory_kib, 262_144);
        assert_eq!(value.kdf.iterations, 3);
        assert_eq!(value.kdf.parallelism, 2);
        assert_eq!(value.kdf.output_bytes, 32);
        assert_eq!(value.aead.algorithm, "aes-256-gcm");
        assert_eq!(value.aead.nonce_bytes, 12);
    }

    #[test]
    fn header_rejects_future_or_invalid_contract() {
        let mut value = header();
        value.format_version = 2;
        assert_eq!(
            validate_header(&value),
            Err(ContainerError::HeaderFutureVersion)
        );
        value = header();
        value.kdf.algorithm = "pbkdf2".into();
        assert_eq!(validate_header(&value), Err(ContainerError::HeaderInvalid));
    }

    #[test]
    fn document_shape_matches_fz19() {
        let value = document();
        assert!(validate_document(&value).is_ok());
        assert_eq!(value.container.structure_version, 1);
    }

    #[test]
    fn unknown_top_level_fields_are_rejected() {
        let raw = r#"{"container":{"structure_version":1},"unknown":true}"#;
        let error = serde_json::from_str::<ContainerDocument>(raw).unwrap_err();
        assert!(error.to_string().contains("unknown field"));
    }

    #[test]
    fn path_and_secret_references_are_restricted() {
        assert!(ImportPath::parse("exports/demo").is_ok());
        assert_eq!(
            ImportPath::parse("/absolute/path"),
            Err(ContainerError::PathRejected)
        );
        assert_eq!(
            ImportPath::parse("exports/../secret"),
            Err(ContainerError::PathRejected)
        );
        assert_eq!(
            ImportPath::parse("C:\\temp"),
            Err(ContainerError::PathRejected)
        );
        assert!(SecretReference::parse("keychain://ocx.dav.demo").is_ok());
        assert_eq!(
            SecretReference::parse("https://example.test"),
            Err(ContainerError::SecretRejected)
        );
        assert_eq!(
            SecretReference::parse("keychain://ocx.dav.demo?password=1"),
            Err(ContainerError::SecretRejected)
        );
    }

    #[test]
    fn array_limits_are_frozen() {
        assert_eq!(ARRAY_MAX_ITEMS, 20_000);
        assert_eq!(MAX_DEPTH, 64);
        assert_eq!(SINGLE_FILE_MAX_BYTES, 32 * 1024 * 1024);
        assert_eq!(ARCHIVE_ENTRY_MAX_ITEMS, 50_000);
        assert_eq!(ARCHIVE_TOTAL_MAX_BYTES, 1024 * 1024 * 1024);
        assert_eq!(SINGLE_THREAD_MEMORY_MAX_BYTES, 512 * 1024 * 1024);
    }

    #[test]
    fn extension_types_are_reusable_not_duplicated() {
        let config = ExtensionConfig::default();
        assert_eq!(config.revision, 1);
        let target = ClientTarget::user_target(
            crate::modules::extensions::ClientId::Codex,
            std::path::Path::new("/fixtures/opencodex-home"),
            true,
            true,
        );
        assert_eq!(target.scope, TargetScope::User);
    }
}
