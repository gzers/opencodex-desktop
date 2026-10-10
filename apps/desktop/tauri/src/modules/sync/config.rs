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

/// Testable credential boundary. Missing and unavailable are distinct facts.
trait EndpointCredentials {
    fn store(&self, ref_id: &str, password: &str) -> Result<(), AppError>;
    fn load(&self, purpose: &str, ref_id: &str) -> Result<Option<String>, AppError>;
    fn remove(&self, purpose: &str, ref_id: &str) -> Result<(), AppError>;
}
struct SystemCredentials;
impl EndpointCredentials for SystemCredentials {
    fn store(&self, ref_id: &str, password: &str) -> Result<(), AppError> {
        keychain::store_webdav_password(ref_id, password)
    }
    fn load(&self, purpose: &str, ref_id: &str) -> Result<Option<String>, AppError> {
        keychain::load_optional_keychain_password(purpose, ref_id)
    }
    fn remove(&self, purpose: &str, ref_id: &str) -> Result<(), AppError> {
        keychain::remove_keychain_password(purpose, ref_id)
    }
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
        self.save_endpoint_observed(input, |_, _| {})
    }

    /// Caller holds storage/sync admission through write, verification and callback.
    /// Validation and unreadable existing configuration are preflight refusals.
    pub(crate) fn save_endpoint_observed(
        &self,
        input: &SyncEndpointInput,
        observer: impl FnOnce(&[u8], bool),
    ) -> Result<SyncEndpointConfig, AppError> {
        self.save_endpoint_with_credentials(input, &SystemCredentials, observer)
    }

    fn save_endpoint_with_credentials(
        &self,
        input: &SyncEndpointInput,
        credentials: &impl EndpointCredentials,
        observer: impl FnOnce(&[u8], bool),
    ) -> Result<SyncEndpointConfig, AppError> {
        validate_input(input)?;
        let mut config = self.load()?;
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let ref_id = config
            .active()
            .map(|endpoint| endpoint.credential_ref.ref_id.clone())
            .unwrap_or_else(keychain::new_ref_id);
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
            endpoint_id: "default".to_string(),
            url: input.base_url.trim().trim_end_matches('/').to_string(),
            remote_path: input.remote_path.trim().trim_matches('/').to_string(),
            username: input.username.trim().to_string(),
            credential_ref: credential,
            tls_policy: TLS_POLICY_VERIFY_REQUIRED.to_string(),
            legacy_encryption: String::new(),
            conflict_policy: input.conflict_policy.clone(),
        };
        // Full requested values, without random refs/timestamps, define retries.
        // Secret-bearing bytes stay local; event delivery persists only a digest.
        let candidate = serde_json::to_vec(&(
            &endpoint.url,
            &endpoint.remote_path,
            &endpoint.username,
            &input.password,
            &endpoint.conflict_policy,
        ))
        .map_err(|_| AppError::NotConfigured)?;
        config.endpoints = vec![endpoint.clone()];
        validate_config(&config)?;
        let bytes = serde_json::to_vec_pretty(&config).map_err(|_| AppError::NotConfigured)?;
        let result = (|| {
            credentials.store(&ref_id, &input.password)?;
            crate::infrastructure::atomic_write::atomic_write(&self.path(), &bytes, 0o600)?;
            if self.load()? != config
                || credentials
                    .load(WEBDAV_CREDENTIAL_PURPOSE, &ref_id)?
                    .as_deref()
                    != Some(input.password.as_str())
            {
                return Err(AppError::NotConfigured);
            }
            Ok(endpoint)
        })();
        observer(&candidate, result.is_ok());
        result
    }

    pub fn delete_endpoint(&self, delete_credentials: bool) -> Result<(), AppError> {
        self.delete_endpoint_observed(delete_credentials, |_, _| {})
    }

    pub(crate) fn delete_endpoint_observed(
        &self,
        delete_credentials: bool,
        observer: impl FnOnce(&[u8], bool),
    ) -> Result<(), AppError> {
        self.delete_endpoint_with_credentials(delete_credentials, &SystemCredentials, observer)
    }

    fn delete_endpoint_with_credentials(
        &self,
        delete_credentials: bool,
        credentials: &impl EndpointCredentials,
        observer: impl FnOnce(&[u8], bool),
    ) -> Result<(), AppError> {
        let config = self.load()?;
        // Keep the old reference on any credential failure, so a partial cleanup
        // can be retried. Keeping credentials and removing them are distinct intents.
        let candidate = serde_json::to_vec(&(&config, delete_credentials))
            .map_err(|_| AppError::NotConfigured)?;
        let empty = SyncConfig::default();
        let bytes = serde_json::to_vec_pretty(&empty).map_err(|_| AppError::NotConfigured)?;
        let result = (|| {
            if let Some(endpoint) = config.active() {
                if delete_credentials {
                    let ref_id = &endpoint.credential_ref.ref_id;
                    for purpose in [
                        WEBDAV_CREDENTIAL_PURPOSE,
                        keychain::ENCRYPTION_PASSWORD_PURPOSE,
                    ] {
                        credentials.remove(purpose, ref_id)?;
                        if credentials.load(purpose, ref_id)?.is_some() {
                            return Err(AppError::NotConfigured);
                        }
                    }
                }
            }
            crate::infrastructure::atomic_write::atomic_write(&self.path(), &bytes, 0o600)?;
            if self.load()? != empty {
                return Err(AppError::NotConfigured);
            }
            Ok(())
        })();
        observer(&candidate, result.is_ok());
        result
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

#[cfg(test)]
mod endpoint_mutation_tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;

    #[derive(Clone, Copy, PartialEq, Eq, Default)]
    enum Fault {
        #[default]
        None,
        StoreError,
        StoreNoop,
        ReadError,
        WrongRead,
        DeleteError,
        DeleteNoop,
        LegacyDeleteError,
    }
    #[derive(Default)]
    struct Credentials {
        values: RefCell<HashMap<(String, String), String>>,
        calls: RefCell<Vec<String>>,
        fault: Cell<Fault>,
        after_store: RefCell<Option<Box<dyn FnOnce()>>>,
        after_remove: RefCell<Option<Box<dyn FnOnce()>>>,
    }
    impl EndpointCredentials for Credentials {
        fn store(&self, ref_id: &str, password: &str) -> Result<(), AppError> {
            self.calls.borrow_mut().push("store".into());
            if self.fault.get() == Fault::StoreError {
                return Err(AppError::NotConfigured);
            }
            if self.fault.get() != Fault::StoreNoop {
                self.values.borrow_mut().insert(
                    (WEBDAV_CREDENTIAL_PURPOSE.into(), ref_id.into()),
                    password.into(),
                );
            }
            if let Some(hook) = self.after_store.borrow_mut().take() {
                hook();
            }
            Ok(())
        }
        fn load(&self, purpose: &str, ref_id: &str) -> Result<Option<String>, AppError> {
            self.calls.borrow_mut().push(format!("read:{purpose}"));
            match self.fault.get() {
                Fault::ReadError => Err(AppError::NotConfigured),
                Fault::WrongRead => Ok(Some("different-secret".into())),
                _ => Ok(self
                    .values
                    .borrow()
                    .get(&(purpose.into(), ref_id.into()))
                    .cloned()),
            }
        }
        fn remove(&self, purpose: &str, ref_id: &str) -> Result<(), AppError> {
            self.calls.borrow_mut().push(format!("delete:{purpose}"));
            if self.fault.get() == Fault::DeleteError
                || (self.fault.get() == Fault::LegacyDeleteError
                    && purpose == keychain::ENCRYPTION_PASSWORD_PURPOSE)
            {
                return Err(AppError::NotConfigured);
            }
            if self.fault.get() != Fault::DeleteNoop {
                self.values
                    .borrow_mut()
                    .remove(&(purpose.into(), ref_id.into()));
            }
            if let Some(hook) = self.after_remove.borrow_mut().take() {
                hook();
            }
            Ok(())
        }
    }
    fn input() -> SyncEndpointInput {
        SyncEndpointInput {
            base_url: "https://dav.example.test/".into(),
            remote_path: "/desktop-sync/current/".into(),
            username: " user ".into(),
            password: "private-password".into(),
            conflict_policy: CONFLICT_POLICY_ASK.into(),
        }
    }
    fn save(store: &SyncConfigStore, credentials: &Credentials) -> SyncEndpointConfig {
        store
            .save_endpoint_with_credentials(&input(), credentials, |_, ok| assert!(ok))
            .unwrap()
    }

    #[test]
    fn save_requires_exact_config_and_secret_readback() {
        let root = tempfile::tempdir().unwrap();
        let store = SyncConfigStore::new(root.path());
        let credentials = Credentials::default();
        let endpoint = save(&store, &credentials);
        assert_eq!(store.load().unwrap().active(), Some(&endpoint));
        assert_eq!(endpoint.url, "https://dav.example.test");
        assert_eq!(endpoint.remote_path, "desktop-sync/current");
        assert_eq!(endpoint.username, "user");
        assert_eq!(
            *credentials.calls.borrow(),
            ["store", "read:webdav_credential"]
        );
        assert!(!std::fs::read_to_string(store.path())
            .unwrap()
            .contains("private-password"));
    }

    #[test]
    fn save_retry_keeps_full_intent_and_observes_real_error() {
        let root = tempfile::tempdir().unwrap();
        let store = SyncConfigStore::new(root.path());
        let credentials = Credentials::default();
        let original = save(&store, &credentials);
        let mut changed = input();
        changed.password = "changed-password".into();
        credentials.fault.set(Fault::StoreError);
        let mut failed = Vec::new();
        assert!(store
            .save_endpoint_with_credentials(&changed, &credentials, |candidate, ok| {
                assert!(!ok);
                failed = candidate.to_vec();
            })
            .is_err());
        assert_eq!(store.load().unwrap().active(), Some(&original));
        credentials.fault.set(Fault::None);
        let retried = store
            .save_endpoint_with_credentials(&changed, &credentials, |candidate, ok| {
                assert!(ok);
                assert_eq!(candidate, failed);
            })
            .unwrap();
        assert_eq!(
            retried.credential_ref.ref_id,
            original.credential_ref.ref_id
        );
        store
            .save_endpoint_with_credentials(&input(), &credentials, |candidate, ok| {
                assert!(ok);
                assert_ne!(candidate, failed);
            })
            .unwrap();
    }

    #[test]
    fn save_missing_wrong_or_unavailable_readback_cannot_succeed() {
        for fault in [Fault::StoreNoop, Fault::WrongRead, Fault::ReadError] {
            let root = tempfile::tempdir().unwrap();
            let store = SyncConfigStore::new(root.path());
            let credentials = Credentials::default();
            credentials.fault.set(fault);
            let mut observed = false;
            assert!(store
                .save_endpoint_with_credentials(&input(), &credentials, |_, ok| {
                    observed = true;
                    assert!(!ok);
                })
                .is_err());
            assert!(observed);
            assert!(
                store.load().unwrap().active().is_some(),
                "config commit is not rolled back"
            );
        }
    }

    #[test]
    fn save_partial_credential_write_reports_failure() {
        let root = tempfile::tempdir().unwrap();
        let store = SyncConfigStore::new(root.path());
        let credentials = Credentials::default();
        let blocked = store.path();
        credentials.after_store.replace(Some(Box::new(move || {
            std::fs::create_dir_all(blocked).unwrap();
        })));
        let mut observed = false;
        assert!(store
            .save_endpoint_with_credentials(&input(), &credentials, |_, ok| {
                observed = true;
                assert!(!ok);
            })
            .is_err());
        assert!(observed);
        assert_eq!(
            credentials.values.borrow().len(),
            1,
            "partial secret commit is not hidden"
        );
    }

    #[test]
    fn preflight_refusals_do_not_write_or_observe_terminals() {
        let root = tempfile::tempdir().unwrap();
        let store = SyncConfigStore::new(root.path());
        let credentials = Credentials::default();
        for invalid in ["", "/"] {
            let mut value = input();
            value.remote_path = invalid.into();
            assert!(store
                .save_endpoint_with_credentials(&value, &credentials, |_, _| panic!("preflight"))
                .is_err());
        }
        std::fs::create_dir_all(store.path().parent().unwrap()).unwrap();
        std::fs::write(store.path(), b"corrupt").unwrap();
        assert!(store
            .save_endpoint_with_credentials(&input(), &credentials, |_, _| panic!("preflight"))
            .is_err());
        assert!(store
            .delete_endpoint_with_credentials(true, &credentials, |_, _| panic!("preflight"))
            .is_err());
        assert!(credentials.calls.borrow().is_empty());
        assert_eq!(std::fs::read(store.path()).unwrap(), b"corrupt");
    }

    #[test]
    fn delete_verifies_both_purposes_and_accepts_explicit_absence() {
        for legacy in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let store = SyncConfigStore::new(root.path());
            let credentials = Credentials::default();
            let endpoint = save(&store, &credentials);
            if legacy {
                credentials.values.borrow_mut().insert(
                    (
                        keychain::ENCRYPTION_PASSWORD_PURPOSE.into(),
                        endpoint.credential_ref.ref_id.clone(),
                    ),
                    "legacy-secret".into(),
                );
            }
            credentials.calls.borrow_mut().clear();
            store
                .delete_endpoint_with_credentials(true, &credentials, |_, ok| assert!(ok))
                .unwrap();
            assert_eq!(store.load().unwrap(), SyncConfig::default());
            assert!(credentials.values.borrow().is_empty());
            assert_eq!(
                *credentials.calls.borrow(),
                [
                    "delete:webdav_credential",
                    "read:webdav_credential",
                    "delete:encryption_password",
                    "read:encryption_password"
                ]
            );
            credentials.calls.borrow_mut().clear();
            store
                .delete_endpoint_with_credentials(true, &credentials, |_, ok| assert!(ok))
                .unwrap();
            assert!(credentials.calls.borrow().is_empty());
        }
    }

    #[test]
    fn failed_or_unverified_deletion_retains_reference_for_exact_retry() {
        for fault in [
            Fault::DeleteError,
            Fault::DeleteNoop,
            Fault::LegacyDeleteError,
            Fault::ReadError,
        ] {
            let root = tempfile::tempdir().unwrap();
            let store = SyncConfigStore::new(root.path());
            let credentials = Credentials::default();
            let endpoint = save(&store, &credentials);
            let before = std::fs::read(store.path()).unwrap();
            credentials.fault.set(fault);
            let mut failed = Vec::new();
            assert!(store
                .delete_endpoint_with_credentials(true, &credentials, |candidate, ok| {
                    assert!(!ok);
                    failed = candidate.to_vec();
                })
                .is_err());
            assert_eq!(std::fs::read(store.path()).unwrap(), before);
            assert_eq!(store.load().unwrap().active(), Some(&endpoint));
            credentials.fault.set(Fault::None);
            store
                .delete_endpoint_with_credentials(true, &credentials, |candidate, ok| {
                    assert!(ok);
                    assert_eq!(candidate, failed);
                })
                .unwrap();
        }
    }

    #[test]
    fn keeping_credentials_is_a_distinct_intent_and_never_accesses_them() {
        let root = tempfile::tempdir().unwrap();
        let store = SyncConfigStore::new(root.path());
        let credentials = Credentials::default();
        save(&store, &credentials);
        credentials.fault.set(Fault::DeleteError);
        let mut deleting = Vec::new();
        assert!(store
            .delete_endpoint_with_credentials(true, &credentials, |candidate, ok| {
                assert!(!ok);
                deleting = candidate.to_vec();
            })
            .is_err());
        credentials.calls.borrow_mut().clear();
        store
            .delete_endpoint_with_credentials(false, &credentials, |candidate, ok| {
                assert!(ok);
                assert_ne!(candidate, deleting);
            })
            .unwrap();
        assert!(credentials.calls.borrow().is_empty());
        assert_eq!(credentials.values.borrow().len(), 1);
        assert_eq!(store.load().unwrap(), SyncConfig::default());
    }

    #[test]
    fn delete_partial_disk_failure_keeps_reference_and_retry_is_idempotent() {
        let root = tempfile::tempdir().unwrap();
        let store = SyncConfigStore::new(root.path());
        let credentials = Credentials::default();
        save(&store, &credentials);
        let before = std::fs::read(store.path()).unwrap();
        let path = store.path();
        credentials.after_remove.replace(Some(Box::new(move || {
            std::fs::remove_file(&path).unwrap();
            std::fs::create_dir(&path).unwrap();
        })));
        let mut failed = Vec::new();
        assert!(store
            .delete_endpoint_with_credentials(true, &credentials, |candidate, ok| {
                assert!(!ok);
                failed = candidate.to_vec();
            })
            .is_err());
        assert!(credentials.values.borrow().is_empty());
        // Restore only the injected filesystem fault, not the already removed secrets.
        std::fs::remove_dir(store.path()).unwrap();
        std::fs::write(store.path(), before).unwrap();
        store
            .delete_endpoint_with_credentials(true, &credentials, |candidate, ok| {
                assert!(ok);
                assert_eq!(candidate, failed);
            })
            .unwrap();
    }
}
