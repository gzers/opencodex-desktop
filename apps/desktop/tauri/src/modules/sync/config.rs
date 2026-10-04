//! WebDAV 端点与凭据引用的本地配置。
//!
//! 配置文件只保存账号和 `cred_...` 引用；真实 WebDAV 口令保存在 macOS
//! Keychain。读取时冻结 `tls_policy=verify_required`。
//!
//! 新流程的同步载荷为明文 + SHA-256 完整性校验，**不再**要求额外内容加密口令；
//! 旧配置里的 `encryption=forced` 字段作为历史字段读取后忽略，不再写出。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::errors::AppError;
use crate::infrastructure::keychain::{self, WEBDAV_CREDENTIAL_PURPOSE};

pub const SYNC_ENDPOINTS_RELATIVE_PATH: &str = "manager-state/sync-endpoints.json";
pub const TLS_POLICY_VERIFY_REQUIRED: &str = "verify_required";
pub const ENCRYPTION_FORCED: &str = "forced";
pub const CONFLICT_POLICY_ASK: &str = "ask";
pub const CONFLICT_POLICY_KEEP_LOCAL: &str = "keep-local";
pub const CONFLICT_POLICY_KEEP_REMOTE: &str = "keep-remote";
pub const CONFLICT_POLICY_KEEP_BOTH: &str = "keep-both";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct CredentialRef {
    pub ref_id: String,
    pub backend: String,
    pub service_name: String,
    pub account_key: String,
    pub purpose: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct SyncEndpointConfig {
    pub endpoint_id: String,
    pub url: String,
    pub remote_path: String,
    pub username: String,
    pub credential_ref: CredentialRef,
    pub tls_policy: String,
    /// 历史字段：旧版本用 `forced` 表示同步载荷曾由用户口令加密。只读兼容，不再写出。
    #[serde(default, rename = "encryption", skip_serializing)]
    pub legacy_encryption: String,
    pub conflict_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct SyncConfig {
    pub schema_version: u32,
    pub endpoints: Vec<SyncEndpointConfig>,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            endpoints: Vec::new(),
        }
    }
}

impl SyncConfig {
    pub fn active(&self) -> Option<&SyncEndpointConfig> {
        self.endpoints.first()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncEndpointInput {
    pub base_url: String,
    pub remote_path: String,
    pub username: String,
    pub password: String,
    pub conflict_policy: String,
}

pub struct SyncConfigStore {
    data_root: PathBuf,
}

impl SyncConfigStore {
    pub fn new(data_root: &Path) -> Self {
        Self {
            data_root: data_root.to_path_buf(),
        }
    }

    pub fn path(&self) -> PathBuf {
        self.data_root.join(SYNC_ENDPOINTS_RELATIVE_PATH)
    }

    pub fn load(&self) -> Result<SyncConfig, AppError> {
        match std::fs::read(self.path()) {
            Ok(bytes) => {
                let value: SyncConfig =
                    serde_json::from_slice(&bytes).map_err(|_| AppError::NotConfigured)?;
                validate_config(&value)?;
                Ok(value)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(SyncConfig::default()),
            Err(_) => Err(AppError::FileSystem {
                operation: "read sync endpoint config".to_string(),
                detail: "configuration could not be read".to_string(),
            }),
        }
    }

    pub fn save_endpoint(&self, input: &SyncEndpointInput) -> Result<SyncEndpointConfig, AppError> {
        validate_input(input)?;
        let mut config = self.load()?;
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let endpoint_id = "default".to_string();
        let ref_id = config
            .active()
            .map(|endpoint| endpoint.credential_ref.ref_id.clone())
            .unwrap_or_else(keychain::new_ref_id);

        // 只保存 WebDAV 服务端认证口令；同步载荷不再使用额外加密口令。
        keychain::store_webdav_password(&ref_id, &input.password)?;

        let credential = CredentialRef {
            ref_id: ref_id.clone(),
            backend: "keychain".to_string(),
            service_name: keychain::keychain_service_name().to_string(),
            account_key: keychain::account_key(WEBDAV_CREDENTIAL_PURPOSE, &ref_id)?,
            purpose: WEBDAV_CREDENTIAL_PURPOSE.to_string(),
            created_at: config
                .active()
                .map(|endpoint| endpoint.credential_ref.created_at.clone())
                .unwrap_or_else(|| now.clone()),
            updated_at: now,
        };
        let endpoint = SyncEndpointConfig {
            endpoint_id,
            url: input.base_url.trim().trim_end_matches('/').to_string(),
            remote_path: input.remote_path.trim().trim_matches('/').to_string(),
            username: input.username.trim().to_string(),
            credential_ref: credential,
            tls_policy: TLS_POLICY_VERIFY_REQUIRED.to_string(),
            legacy_encryption: String::new(),
            conflict_policy: input.conflict_policy.clone(),
        };
        config.endpoints.clear();
        config.endpoints.push(endpoint.clone());
        let bytes = serde_json::to_vec_pretty(&config).map_err(|_| AppError::NotConfigured)?;
        crate::infrastructure::atomic_write::atomic_write(&self.path(), &bytes, 0o600)?;
        Ok(endpoint)
    }

    pub fn delete_endpoint(&self, delete_credentials: bool) -> Result<(), AppError> {
        let config = self.load()?;
        if let Some(endpoint) = config.active() {
            if delete_credentials {
                let _ = keychain::delete_webdav_password(&endpoint.credential_ref.ref_id);
                // 旧版本可能存有同步加密口令；不存在时删除会返回错误，这里按「尽力清理」处理。
                let _ = keychain::delete_encryption_password(&endpoint.credential_ref.ref_id);
            }
        }
        let bytes = serde_json::to_vec_pretty(&SyncConfig::default())
            .map_err(|_| AppError::NotConfigured)?;
        crate::infrastructure::atomic_write::atomic_write(&self.path(), &bytes, 0o600)
    }

    /// 用外部提供的整份配置替换本机端点配置（配置迁移导入路径）。
    ///
    /// 只写管理器自有配置文件；口令始终在钥匙串，调用方负责新建本机凭据引用。
    pub fn save_config(&self, config: &SyncConfig) -> Result<(), AppError> {
        validate_config(config)?;
        let bytes = serde_json::to_vec_pretty(config).map_err(|_| AppError::NotConfigured)?;
        crate::infrastructure::atomic_write::atomic_write(&self.path(), &bytes, 0o600)
    }
}

pub fn validate_config(config: &SyncConfig) -> Result<(), AppError> {
    if config.schema_version != 1 || config.endpoints.len() > 1 {
        return Err(AppError::NotConfigured);
    }
    for endpoint in &config.endpoints {
        if endpoint.tls_policy != TLS_POLICY_VERIFY_REQUIRED
            || !matches!(
                endpoint.conflict_policy.as_str(),
                CONFLICT_POLICY_ASK
                    | CONFLICT_POLICY_KEEP_LOCAL
                    | CONFLICT_POLICY_KEEP_REMOTE
                    | CONFLICT_POLICY_KEEP_BOTH
            )
        {
            return Err(AppError::NotConfigured);
        }
        if endpoint.endpoint_id.trim().is_empty()
            || endpoint.url.trim().is_empty()
            || endpoint.remote_path.trim().is_empty()
            || endpoint.username.trim().is_empty()
        {
            return Err(AppError::NotConfigured);
        }
        let credential = &endpoint.credential_ref;
        if credential.backend != "keychain"
            || credential.service_name != keychain::keychain_service_name()
            || credential.purpose != WEBDAV_CREDENTIAL_PURPOSE
        {
            return Err(AppError::NotConfigured);
        }
        keychain::account_key(WEBDAV_CREDENTIAL_PURPOSE, &credential.ref_id)?;
    }
    Ok(())
}

pub fn validate_input(input: &SyncEndpointInput) -> Result<(), AppError> {
    if !input.base_url.starts_with("https://") {
        return Err(AppError::NotConfigured);
    }
    if input.remote_path.trim().is_empty()
        || input.remote_path.contains("..")
        || input.username.trim().is_empty()
        || input.password.is_empty()
    {
        return Err(AppError::NotConfigured);
    }
    if !matches!(
        input.conflict_policy.as_str(),
        CONFLICT_POLICY_ASK
            | CONFLICT_POLICY_KEEP_LOCAL
            | CONFLICT_POLICY_KEEP_REMOTE
            | CONFLICT_POLICY_KEEP_BOTH
    ) {
        return Err(AppError::NotConfigured);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> SyncEndpointInput {
        SyncEndpointInput {
            base_url: "https://dav.example.test".to_string(),
            remote_path: "/desktop-sync/current".to_string(),
            username: "user".to_string(),
            password: "password".to_string(),
            conflict_policy: CONFLICT_POLICY_ASK.to_string(),
        }
    }

    #[test]
    fn input_requires_https_and_passwords() {
        assert!(validate_input(&input()).is_ok());
        let mut invalid = input();
        invalid.base_url = "http://dav.example.test".to_string();
        assert!(validate_input(&invalid).is_err());
        let mut missing = input();
        missing.password = String::new();
        assert!(validate_input(&missing).is_err());
    }

    /// 调用点护栏：保存端点只写 WebDAV 服务端认证口令，不再写同步加密口令。
    ///
    /// 会访问系统钥匙串，故默认忽略；改动凭据存储后手动执行：
    /// `cargo test --offline --lib -- --ignored --nocapture save_endpoint_stores`
    #[test]
    #[ignore]
    fn save_endpoint_stores_only_the_webdav_secret() {
        let root = tempfile::tempdir().expect("tempdir");
        let store = SyncConfigStore::new(root.path());
        let endpoint = store.save_endpoint(&input()).expect("save endpoint");
        let ref_id = endpoint.credential_ref.ref_id.clone();

        assert_eq!(
            keychain::load_webdav_password(&ref_id).expect("load webdav"),
            "password"
        );
        // 新流程不再写入同步加密口令位。
        assert!(keychain::load_encryption_password(&ref_id).is_err());

        let _ = keychain::delete_webdav_password(&ref_id);
        let _ = keychain::delete_encryption_password(&ref_id);
    }

    #[test]
    fn config_accepts_legacy_encryption_field_and_rejects_tls_override() {
        let mut config = SyncConfig::default();
        config.endpoints.push(SyncEndpointConfig {
            endpoint_id: "default".to_string(),
            url: "https://dav.example.test".to_string(),
            remote_path: "desktop-sync".to_string(),
            username: "user".to_string(),
            credential_ref: CredentialRef {
                ref_id: "cred_12345678-1234-1234-1234-123456789abc".to_string(),
                backend: "keychain".to_string(),
                service_name: keychain::KEYCHAIN_SERVICE_NAME.to_string(),
                account_key: "ocx.dav.cred_12345678-1234-1234-1234-123456789abc".to_string(),
                purpose: WEBDAV_CREDENTIAL_PURPOSE.to_string(),
                created_at: "2026-09-17T00:00:00Z".to_string(),
                updated_at: "2026-09-17T00:00:00Z".to_string(),
            },
            tls_policy: TLS_POLICY_VERIFY_REQUIRED.to_string(),
            legacy_encryption: ENCRYPTION_FORCED.to_string(),
            conflict_policy: CONFLICT_POLICY_ASK.to_string(),
        });
        // 旧字段只是历史信息，不再参与校验。
        assert!(validate_config(&config).is_ok());
        config.endpoints[0].tls_policy = "ignore".to_string();
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn legacy_encryption_field_is_read_and_not_written() {
        let legacy = r#"{
            "schema_version": 1,
            "endpoints": [{
                "endpoint_id": "default",
                "url": "https://dav.example.test",
                "remote_path": "desktop-sync",
                "username": "user",
                "credential_ref": {
                    "ref_id": "cred_12345678-1234-1234-1234-123456789abc",
                    "backend": "keychain",
                    "service_name": "OpenCodex Desktop",
                    "account_key": "ocx.dav.cred_12345678-1234-1234-1234-123456789abc",
                    "purpose": "webdav_credential",
                    "created_at": "2026-09-17T00:00:00Z",
                    "updated_at": "2026-09-17T00:00:00Z"
                },
                "tls_policy": "verify_required",
                "encryption": "forced",
                "conflict_policy": "ask"
            }]
        }"#;
        let parsed: SyncConfig = serde_json::from_str(legacy).expect("legacy config parses");
        assert_eq!(parsed.endpoints[0].legacy_encryption, "forced");
        assert!(validate_config(&parsed).is_ok());
        // 新写出不再包含该字段。
        let written = serde_json::to_string(&parsed).expect("serialize");
        assert!(!written.contains("forced"));
    }
}
