//! WebDAV 同步的真实执行体。
//!
//! GUI 命令（`commands::sync::run_sync_now`）与默认关闭的 CLI/IPC（`IpcCommand::SyncRun`）
//! 共用这一段：两边读取同一份端点配置与 Keychain 口令，走同样的上传、下载与回放判定，
//! 从而保证「CLI 与 GUI 状态一致」。此前 IPC 侧只读同步状态文件并返回一个伪造的成功结果，
//! 从不接触网络——这正是 IMP-02 D 阶段发现的 Q2-12。

use std::path::Path;

use serde_json::Value;

use crate::infrastructure::keychain;
use crate::infrastructure::webdav_client::{
    WebDavClient, WebDavConfig, WebDavError, WebDavOperation, WebDavTransport,
};
use crate::modules::extensions::{projection, ClientTarget, CLIENT_IDS};
use crate::modules::sync::config::SyncEndpointConfig;
use crate::modules::sync::engine::{
    apply_remote_snapshot_batch, build_local_snapshot, open_manifest, open_payload,
    parse_manifest_package, parse_payload_package, seal_manifest, ManifestPackage, SyncDecision,
    SyncEndpointLocks, SyncState,
};

pub const SYNCED_ARTIFACTS: [&str; 2] = [
    crate::modules::preferences::PREFERENCES_RELATIVE_PATH,
    crate::modules::extensions::projection::CONFIG_RELATIVE_PATH,
];

/// 一个待应用的远端文件：目标路径、可选的迁移后偏好字节（仅偏好需要迁移时）与将要写入的字节。
type StagedArtifact = (std::path::PathBuf, Option<Vec<u8>>, Vec<u8>, String);

/// C 阶段：接收端按偏好 schema 校验/迁移远端载荷。
///
/// 仅对偏好文件生效；已是最新 schema 返回 `None`（原样写入），需要迁移返回分域字节，
/// 过新或损坏则显式失败（不得写入本地）。
fn migrate_incoming_preferences(
    name: &str,
    payload: &[u8],
) -> Result<Option<Vec<u8>>, SyncRunFailure> {
    if name != crate::modules::preferences::PREFERENCES_RELATIVE_PATH {
        return Ok(None);
    }
    let value: serde_json::Value =
        serde_json::from_slice(payload).map_err(|_| SyncRunFailure::NotConfigured)?;
    let declared = value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as u32;
    if declared > crate::modules::config_migration::PREFERENCES_CURRENT_SCHEMA {
        // 混合版本客户端：过新载荷不写入。
        return Err(SyncRunFailure::NotConfigured);
    }
    let preferences = crate::modules::preferences::preferences_from_document(&value)
        .map_err(|_| SyncRunFailure::NotConfigured)?;
    if declared == crate::modules::config_migration::PREFERENCES_CURRENT_SCHEMA {
        // 已当前 schema：原样写入，不改变远端字节。
        return Ok(None);
    }
    let document = crate::modules::preferences::document_from_preferences(&preferences);
    let bytes = serde_json::to_vec_pretty(&document).map_err(|_| SyncRunFailure::NotConfigured)?;
    Ok(Some(bytes))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncOutcome {
    pub snapshot_id: String,
    pub uploaded: Vec<String>,
    pub applied: Vec<String>,
    /// 本次同步是否因冲突（回放/旧快照）暂停了远端覆盖。
    pub conflicted: bool,
    pub etag: Option<String>,
}

impl SyncOutcome {
    /// 用户可见结果：如实说明本次上传了什么、是否应用了远端内容。
    pub fn message(&self) -> String {
        if self.conflicted {
            return "检测到本地与远端冲突；已暂停覆盖并保留双方历史，请确认后重试。".to_string();
        }
        if self.applied.is_empty() {
            "WebDAV 同步已上传。".to_string()
        } else {
            format!(
                "WebDAV 同步完成：已应用远端 {}，并上传本机快照。",
                self.applied.join("、")
            )
        }
    }
}

/// 同步失败分类：保留「哪个阶段、什么错误」，供 GUI 与 CLI 各自映射。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncRunFailure {
    /// 未配置端点，或本地内容无效。
    NotConfigured,
    /// 冷同步已开启，本次同步被跳过（不是失败，是策略）。
    ColdSync,
    /// 同一端点已有同步在跑。
    TargetLocked,
    /// 真实网络阶段失败（探测/下载/上传）。
    WebDav(WebDavOperation, WebDavError),
}

impl SyncRunFailure {
    pub fn user_message(self) -> String {
        match self {
            Self::NotConfigured => {
                "同步未配置或内容无效；请检查同步状态与本地内容后重试。".to_string()
            }
            Self::ColdSync => "冷同步已开启，已跳过本次同步。".to_string(),
            Self::TargetLocked => "同步目标正被占用，请稍后重试。".to_string(),
            Self::WebDav(operation, error) => error.user_message(operation).to_string(),
        }
    }
}

/// 读取可选同步文件：不存在或为空时返回 `None`（不作为快照内容）。
///
/// 此前缺失文件会被当成「空载荷」上传，对端取回后无法当作 JSON 校验，
/// 直接让整次同步以 `NotConfigured` 失败——首次运行（还没有扩展配置）就会中招。
fn read_optional_file(path: &Path, operation: &str) -> Result<Option<Vec<u8>>, SyncRunFailure> {
    match std::fs::read(path) {
        Ok(value) if !value.is_empty() => Ok(Some(value)),
        Ok(_) => Ok(None),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => {
            let _ = operation;
            Err(SyncRunFailure::NotConfigured)
        }
    }
}

fn target_path(data_root: &Path, name: &str) -> Result<std::path::PathBuf, SyncRunFailure> {
    if !SYNCED_ARTIFACTS.contains(&name) {
        return Err(SyncRunFailure::NotConfigured);
    }
    let target = data_root.join(name);
    if !target.starts_with(data_root) || target == data_root {
        return Err(SyncRunFailure::NotConfigured);
    }
    Ok(target)
}

/// 投影文件必须是 JSON 文本。
///
/// 载荷与清单的一致性已由 `open_payload` 按**明文原始字节**的 SHA-256 校验（准确且充分）。
/// 这里此前又拿「解析成 `serde_json::Value` 再 pretty 打印」的字节去比对清单摘要——
/// 对象键顺序会变，合法内容也会被判为不一致，导致远端内容永远无法应用。
fn payload_is_json(payload: &[u8]) -> bool {
    serde_json::from_slice::<Value>(payload).is_ok()
}

/// 受控 WebDAV 端点配置：地址 + 账号 + 已从 Keychain 取出的口令。
pub fn webdav_config(endpoint: &SyncEndpointConfig) -> Result<WebDavConfig, SyncRunFailure> {
    let password = keychain::load_webdav_password(&endpoint.credential_ref.ref_id)
        .map_err(|_| SyncRunFailure::NotConfigured)?;
    Ok(WebDavConfig {
        base_url: endpoint.url.clone(),
        remote_path: endpoint.remote_path.clone(),
        username: endpoint.username.clone(),
        password,
    })
}

/// 真正执行一次同步：先应用远端快照（若有），再上传本机快照。
pub async fn run_sync(
    data_root: &Path,
    home: &Path,
    endpoint: &SyncEndpointConfig,
    webdav: &WebDavConfig,
) -> Result<SyncOutcome, SyncRunFailure> {
    run_sync_observed(data_root, home, endpoint, webdav, |_, _| {}).await
}

/// Only captured execution is terminal. Admission, cold-sync and local capture
/// refusal happen before this observer and cannot recover earlier failures.
pub(crate) async fn run_sync_observed(
    data_root: &Path,
    home: &Path,
    endpoint: &SyncEndpointConfig,
    webdav: &WebDavConfig,
    observer: impl FnOnce(&[u8], SyncTerminal),
) -> Result<SyncOutcome, SyncRunFailure> {
    let _admission = crate::infrastructure::storage_writers::global()
        .admit()
        .map_err(|_| SyncRunFailure::TargetLocked)?;
    // 每 endpoint 的进程级互斥：并发同步必须被挡住，而不是各自跑到一半互相覆盖。
    let _locks = SyncEndpointLocks::global()
        .acquire(&endpoint.endpoint_id)
        .map_err(|_| SyncRunFailure::TargetLocked)?;
    let preferences = crate::modules::preferences::PreferencesStore::new(data_root)
        .load()
        .map_err(|_| SyncRunFailure::NotConfigured)?;
    if preferences.cold_sync {
        return Err(SyncRunFailure::ColdSync);
    }
    let now = chrono::Utc::now();
    let mut local_payloads: Vec<(String, Vec<u8>)> = Vec::new();
    if let Some(preference_payload) = read_optional_file(
        &data_root.join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH),
        "read preferences for sync",
    )? {
        local_payloads.push((
            crate::modules::preferences::PREFERENCES_RELATIVE_PATH.to_string(),
            preference_payload,
        ));
    }
    if let Some(extension_payload) = read_optional_file(
        &crate::modules::extensions::projection::config_path(data_root),
        "read extension projection for sync",
    )? {
        local_payloads.push((
            crate::modules::extensions::projection::CONFIG_RELATIVE_PATH.to_string(),
            extension_payload,
        ));
    }
    if local_payloads.is_empty() {
        return Err(SyncRunFailure::NotConfigured);
    }
    let candidate = execution_candidate(endpoint, webdav, &local_payloads)?;
    let client = WebDavClient::production();
    observe_execution(
        &candidate,
        run_prepared_sync(
            data_root,
            home,
            endpoint,
            webdav,
            local_payloads,
            now,
            &client,
        ),
        observer,
    )
    .await
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SyncTerminal {
    Failed,
    Succeeded,
    Conflict,
}

fn execution_candidate(
    endpoint: &SyncEndpointConfig,
    webdav: &WebDavConfig,
    artifacts: &[(String, Vec<u8>)],
) -> Result<Vec<u8>, SyncRunFailure> {
    // Captured bytes, not generated snapshot IDs/timestamps, identify retries.
    // Candidate bytes stay local; notification delivery persists only their hash.
    serde_json::to_vec(&(
        endpoint,
        &webdav.base_url,
        &webdav.remote_path,
        &webdav.username,
        &webdav.password,
        artifacts,
    ))
    .map_err(|_| SyncRunFailure::NotConfigured)
}

async fn observe_execution(
    candidate: &[u8],
    task: impl std::future::Future<Output = Result<SyncOutcome, SyncRunFailure>>,
    observer: impl FnOnce(&[u8], SyncTerminal),
) -> Result<SyncOutcome, SyncRunFailure> {
    let result = task.await;
    let terminal = match &result {
        Ok(outcome) if outcome.conflicted => SyncTerminal::Conflict,
        Ok(_) => SyncTerminal::Succeeded,
        Err(SyncRunFailure::ColdSync | SyncRunFailure::TargetLocked) => return result,
        Err(_) => SyncTerminal::Failed,
    };
    observer(candidate, terminal);
    result
}

async fn run_prepared_sync<T: WebDavTransport>(
    data_root: &Path,
    home: &Path,
    endpoint: &SyncEndpointConfig,
    webdav: &WebDavConfig,
    local_payloads: Vec<(String, Vec<u8>)>,
    now: chrono::DateTime<chrono::Utc>,
    client: &WebDavClient<T>,
) -> Result<SyncOutcome, SyncRunFailure> {
    let artifacts = local_payloads.clone();
    let (manifest, payloads) = build_local_snapshot(artifacts, "OpenCodeX Desktop", now)
        .map_err(|_| SyncRunFailure::NotConfigured)?;
    let uploaded_names: Vec<String> = local_payloads
        .iter()
        .map(|(name, _)| name.clone())
        .collect();
    // 新流程不索取额外内容口令：清单与载荷都是明文 + 完整性摘要。
    let sealed_manifest = seal_manifest(&manifest).map_err(|_| SyncRunFailure::NotConfigured)?;
    let manifest_snapshot = manifest.snapshot_id.clone();
    let remote_package = client
        .download_latest_pointer(webdav)
        .await
        .map_err(|error| SyncRunFailure::WebDav(WebDavOperation::Download, error))?;
    let remote_manifest = match remote_package {
        Some(snapshot_id) => {
            let manifest_bytes = client
                .download(webdav, &snapshot_id)
                .await
                .map_err(|error| SyncRunFailure::WebDav(WebDavOperation::Download, error))?;
            Some((
                snapshot_id,
                parse_manifest_package(&manifest_bytes)
                    .map_err(|_| SyncRunFailure::NotConfigured)?,
            ))
        }
        None => None,
    };
    let mut sync_state = SyncState::load(data_root, &endpoint.endpoint_id)
        .map_err(|_| SyncRunFailure::NotConfigured)?;
    let mut applied_paths: Vec<String> = Vec::new();
    let mut conflicted = false;
    if let Some((snapshot_id, remote)) = &remote_manifest {
        // 旧版（v1）远端仍需要原加密口令：只有这条兼容路径才读取钥匙串里的旧口令。
        let legacy_passphrase = match remote {
            ManifestPackage::Legacy(_) => {
                keychain::load_encryption_password(&endpoint.credential_ref.ref_id)
                    .ok()
                    .map(|value| value.into_bytes())
            }
            ManifestPackage::Plaintext(_) => None,
        };
        let remote_decoded = open_manifest(remote, legacy_passphrase.as_deref())
            .map_err(|_| SyncRunFailure::NotConfigured)?;
        if remote_decoded.snapshot_id != *snapshot_id {
            return Err(SyncRunFailure::NotConfigured);
        }
        let mut downloaded = Vec::new();
        for artifact in &remote_decoded.artifacts {
            let payload_bytes = client
                .download_payload(webdav, snapshot_id, &artifact.sha256)
                .await
                .map_err(|error| SyncRunFailure::WebDav(WebDavOperation::Download, error))?;
            let envelope =
                parse_payload_package(&payload_bytes).map_err(|_| SyncRunFailure::NotConfigured)?;
            let decrypted = open_payload(&envelope, &artifact.sha256)
                .map_err(|_| SyncRunFailure::NotConfigured)?;
            downloaded.push((artifact.name.clone(), decrypted));
        }
        let remote_manifest_hash = crate::infrastructure::hash::sha256_hex(
            &serde_json::to_vec(&remote_decoded).map_err(|_| SyncRunFailure::NotConfigured)?,
        );
        // 先把整批内容校验完，再一次性应用：回放判定只做一次，写入失败整体回滚。
        // C 阶段：远端分域偏好先按 schema 校验/迁移；过新载荷不得写入本地。
        let mut staged: Vec<StagedArtifact> = Vec::new();
        for (name, payload) in &downloaded {
            if !payload_is_json(payload) {
                return Err(SyncRunFailure::NotConfigured);
            }
            let migrated = migrate_incoming_preferences(name, payload)?;
            let write_bytes = migrated.clone().unwrap_or_else(|| payload.clone());
            staged.push((
                target_path(data_root, name)?,
                migrated,
                write_bytes,
                name.clone(),
            ));
        }
        let batch: Vec<(&std::path::Path, &[u8])> = staged
            .iter()
            .map(|(path, _, bytes, _)| (path.as_path(), bytes.as_slice()))
            .collect();
        // 整批应用：回放判定只做一次，写入失败整体回滚。
        let decision = apply_remote_snapshot_batch(
            &mut sync_state,
            data_root,
            &batch,
            &remote_decoded.snapshot_id,
            &remote_decoded.created_at,
            &remote_manifest_hash,
            now,
        )
        .map_err(|_| SyncRunFailure::NotConfigured)?;
        // 偏好需要迁移时，用与偏好存储一致的分域结构追写，使本地落盘即当前 schema。
        if matches!(decision, SyncDecision::Applied { .. }) {
            for (path, migrated, _, _) in &staged {
                if let Some(bytes) = migrated {
                    crate::infrastructure::atomic_write::atomic_write(path, bytes, 0o600)
                        .map_err(|_| SyncRunFailure::NotConfigured)?;
                }
            }
        }
        // 只有真的写入本地时才声称「已应用」；回放/旧快照只登记冲突，不改动本地。
        if matches!(decision, SyncDecision::Applied { .. }) {
            applied_paths = staged
                .iter()
                .map(|(_, _, _, name)| name.clone())
                .collect::<Vec<_>>();
        } else if matches!(decision, SyncDecision::Conflict) {
            conflicted = true;
        }
        sync_state
            .save(data_root)
            .map_err(|_| SyncRunFailure::NotConfigured)?;
        let targets = CLIENT_IDS
            .map(|client_id| ClientTarget::user_target(client_id, home, true, true))
            .to_vec();
        // 本机没有扩展统一配置时视为「没有需要投射的 Skills」，而不是同步失败。
        match std::fs::read(crate::modules::extensions::projection::config_path(
            data_root,
        )) {
            Ok(live) => {
                projection::apply_skill_links(data_root, &live, &targets)
                    .map_err(|_| SyncRunFailure::NotConfigured)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(SyncRunFailure::NotConfigured),
        }
    }
    // 载荷文件名必须用清单里的明文 `sha256`（下载端也按它取），不是封装字节的摘要。
    let upload_artifacts: Vec<(String, String, Vec<u8>)> = payloads
        .into_iter()
        .map(|(name, envelope)| {
            let digest = manifest
                .artifacts
                .iter()
                .find(|artifact| artifact.name == name)
                .map(|artifact| artifact.sha256.clone())
                .ok_or(SyncRunFailure::NotConfigured)?;
            let bytes = serde_json::to_vec(&envelope).map_err(|_| SyncRunFailure::NotConfigured)?;
            Ok((name, digest, bytes))
        })
        .collect::<Result<_, _>>()?;
    let manifest_bytes =
        serde_json::to_vec(&sealed_manifest).map_err(|_| SyncRunFailure::NotConfigured)?;
    let etag = client
        .upload(
            webdav,
            &manifest_snapshot,
            &upload_artifacts,
            &manifest_bytes,
        )
        .await
        .map_err(|error| SyncRunFailure::WebDav(WebDavOperation::Upload, error))?;
    Ok(SyncOutcome {
        snapshot_id: manifest_snapshot,
        uploaded: uploaded_names,
        applied: applied_paths,
        conflicted,
        etag,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn endpoint() -> SyncEndpointConfig {
        use crate::modules::sync::config::CredentialRef;
        SyncEndpointConfig {
            endpoint_id: uuid::Uuid::new_v4().to_string(),
            url: "https://dav.example.test".into(),
            remote_path: "sync".into(),
            username: "fixture".into(),
            credential_ref: CredentialRef {
                ref_id: "cred_12345678-1234-1234-1234-123456789abc".into(),
                backend: "keychain".into(),
                service_name: "OpenCodex Desktop".into(),
                account_key: "fixture".into(),
                purpose: "webdav_credential".into(),
                created_at: "2026-10-11T00:00:00Z".into(),
                updated_at: "2026-10-11T00:00:00Z".into(),
            },
            tls_policy: "verify_required".into(),
            legacy_encryption: "forced".into(),
            conflict_policy: "ask".into(),
        }
    }

    fn fixture_webdav(e: &SyncEndpointConfig) -> WebDavConfig {
        WebDavConfig {
            base_url: e.url.clone(),
            remote_path: e.remote_path.clone(),
            username: e.username.clone(),
            password: "fixture-secret".into(),
        }
    }

    #[test]
    fn execution_identity_is_stable_for_retry_but_scopes_endpoint_secret_and_content() {
        let e = endpoint();
        let w = fixture_webdav(&e);
        let content = vec![(SYNCED_ARTIFACTS[0].to_string(), b"{}".to_vec())];
        let original = execution_candidate(&e, &w, &content).unwrap();
        assert_eq!(original, execution_candidate(&e, &w, &content).unwrap());
        let mut other_endpoint = e.clone();
        other_endpoint.endpoint_id.push_str("-other");
        assert_ne!(
            original,
            execution_candidate(&other_endpoint, &w, &content).unwrap()
        );
        let mut other_secret = w.clone();
        other_secret.password.push_str("-other");
        assert_ne!(
            original,
            execution_candidate(&e, &other_secret, &content).unwrap()
        );
        let changed = vec![(
            SYNCED_ARTIFACTS[0].to_string(),
            b"{\"changed\":true}".to_vec(),
        )];
        assert_ne!(original, execution_candidate(&e, &w, &changed).unwrap());
    }

    #[tokio::test]
    async fn execution_observer_preserves_result_and_skips_policy_refusals() {
        let success = SyncOutcome {
            snapshot_id: "snap".into(),
            uploaded: vec!["preferences".into()],
            applied: vec![],
            conflicted: false,
            etag: Some("etag".into()),
        };
        for (result, terminal) in [
            (Ok(success.clone()), Some(SyncTerminal::Succeeded)),
            (
                Ok(SyncOutcome {
                    conflicted: true,
                    ..success
                }),
                Some(SyncTerminal::Conflict),
            ),
            (
                Err(SyncRunFailure::NotConfigured),
                Some(SyncTerminal::Failed),
            ),
            (
                Err(SyncRunFailure::WebDav(
                    WebDavOperation::Upload,
                    WebDavError::Network,
                )),
                Some(SyncTerminal::Failed),
            ),
            (Err(SyncRunFailure::ColdSync), None),
            (Err(SyncRunFailure::TargetLocked), None),
        ] {
            let mut seen = Vec::new();
            let expected = result.clone();
            let actual = observe_execution(b"captured", async { result }, |candidate, terminal| {
                assert_eq!(candidate, b"captured");
                seen.push(terminal);
            })
            .await;
            assert_eq!(actual, expected);
            assert_eq!(seen, terminal.into_iter().collect::<Vec<_>>());
        }
    }

    #[tokio::test]
    async fn cold_sync_missing_preferences_and_lock_are_not_execution_terminals() {
        let e = endpoint();
        let w = fixture_webdav(&e);
        let root = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        let store = crate::modules::preferences::PreferencesStore::new(root.path());
        store
            .save(&crate::modules::preferences::Preferences {
                cold_sync: true,
                ..Default::default()
            })
            .unwrap();
        let callback =
            |_: &[u8], _: SyncTerminal| panic!("refusals cannot publish execution terminals");
        assert_eq!(
            run_sync_observed(root.path(), home.path(), &e, &w, callback).await,
            Err(SyncRunFailure::ColdSync)
        );
        store
            .save(&crate::modules::preferences::Preferences {
                cold_sync: false,
                ..Default::default()
            })
            .unwrap();
        std::fs::remove_file(root.path().join(SYNCED_ARTIFACTS[0])).unwrap();
        assert_eq!(
            run_sync_observed(root.path(), home.path(), &e, &w, callback).await,
            // Missing preferences restore the conservative cold-sync default.
            Err(SyncRunFailure::ColdSync)
        );
        let _locked = SyncEndpointLocks::global().acquire(&e.endpoint_id).unwrap();
        assert_eq!(
            run_sync_observed(root.path(), home.path(), &e, &w, callback).await,
            Err(SyncRunFailure::TargetLocked)
        );
    }

    struct SyncTransport {
        pointer_status: u16,
        requests: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    }
    #[async_trait::async_trait]
    impl WebDavTransport for SyncTransport {
        async fn request(
            &self,
            _: &WebDavConfig,
            method: crate::infrastructure::webdav_client::WebDavMethod,
            path: &str,
            _: Option<Vec<u8>>,
        ) -> Result<crate::infrastructure::webdav_client::WebDavResponse, WebDavError> {
            use crate::infrastructure::webdav_client::{WebDavMethod, WebDavResponse};
            self.requests.lock().unwrap().push(path.to_string());
            if method == WebDavMethod::Get {
                return Err(WebDavError::NotFound);
            }
            assert_eq!(method, WebDavMethod::Put);
            Ok(WebDavResponse {
                status: if path == "latest.txt" {
                    self.pointer_status
                } else {
                    201
                },
                body: vec![],
                etag: Some("etag".into()),
            })
        }
    }

    #[tokio::test]
    async fn captured_sync_reports_failed_final_pointer_after_payload_and_manifest_uploads() {
        for pointer_status in [201, 204, 202] {
            let e = endpoint();
            let w = fixture_webdav(&e);
            let root = tempfile::tempdir().unwrap();
            let home = tempfile::tempdir().unwrap();
            crate::modules::data_root::initialize(root.path()).unwrap();
            let bytes =
                serde_json::to_vec(&crate::modules::preferences::document_from_preferences(
                    &crate::modules::preferences::Preferences::default(),
                ))
                .unwrap();
            std::fs::write(root.path().join(SYNCED_ARTIFACTS[0]), &bytes).unwrap();
            let payloads = vec![(SYNCED_ARTIFACTS[0].to_string(), bytes.clone())];
            let candidate = execution_candidate(&e, &w, &payloads).unwrap();
            let requests = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
            let client = WebDavClient::new(SyncTransport {
                pointer_status,
                requests: requests.clone(),
            });
            let mut terminals = vec![];
            let outcome = observe_execution(
                &candidate,
                run_prepared_sync(
                    root.path(),
                    home.path(),
                    &e,
                    &w,
                    payloads,
                    chrono::Utc::now(),
                    &client,
                ),
                |_, terminal| terminals.push(terminal),
            )
            .await;
            if pointer_status == 202 {
                assert_eq!(
                    outcome,
                    Err(SyncRunFailure::WebDav(
                        WebDavOperation::Upload,
                        WebDavError::Protocol
                    ))
                );
                assert_eq!(terminals, [SyncTerminal::Failed]);
            } else {
                assert_eq!(outcome.unwrap().uploaded, [SYNCED_ARTIFACTS[0]]);
                assert_eq!(terminals, [SyncTerminal::Succeeded]);
            }
            let requests = requests.lock().unwrap();
            assert_eq!(requests.len(), 4); // GET pointer, PUT payload, PUT manifest, PUT pointer.
            assert_eq!(requests.last().unwrap(), "latest.txt");
            assert_eq!(
                std::fs::read(root.path().join(SYNCED_ARTIFACTS[0])).unwrap(),
                bytes
            );
        }
    }

    #[test]
    fn synced_artifacts_are_whitelisted() {
        assert_eq!(
            SYNCED_ARTIFACTS,
            [
                "manager-state/preferences.json",
                "manager-state/extension-config.json"
            ]
        );
        assert!(target_path(std::path::Path::new("/tmp/data"), SYNCED_ARTIFACTS[0]).is_ok());
        assert_eq!(
            target_path(std::path::Path::new("/tmp/data"), "../../secret"),
            Err(SyncRunFailure::NotConfigured)
        );
    }

    #[test]
    fn failure_messages_are_user_facing() {
        assert!(SyncRunFailure::ColdSync.user_message().contains("冷同步"));
        assert_eq!(
            SyncRunFailure::WebDav(WebDavOperation::Upload, WebDavError::Tls).user_message(),
            WebDavError::Tls.user_message(WebDavOperation::Upload)
        );
    }

    // 回归：冲突结果此前只体现为「没有应用」，无法与「无变化」区分，冲突提醒也就无从触发。
    #[test]
    fn conflicted_outcome_reports_paused_overwrite() {
        let outcome = SyncOutcome {
            snapshot_id: "snap".to_string(),
            uploaded: vec!["manager-state/preferences.json".to_string()],
            applied: Vec::new(),
            conflicted: true,
            etag: None,
        };
        assert!(outcome.message().contains("冲突"));
        let quiet = SyncOutcome {
            conflicted: false,
            ..outcome
        };
        assert!(!quiet.message().contains("冲突"));
    }

    // C 阶段：接收端按偏好 schema 校验/迁移远端载荷。
    #[test]
    fn incoming_current_preferences_are_written_verbatim() {
        let current = crate::modules::preferences::document_from_preferences(
            &crate::modules::preferences::Preferences::default(),
        );
        let bytes = serde_json::to_vec(&current).unwrap();
        let migrated =
            migrate_incoming_preferences(SYNCED_ARTIFACTS[0], &bytes).expect("current incoming");
        assert!(migrated.is_none(), "当前 schema 原样写入，不做二次转换");
    }

    #[test]
    fn incoming_legacy_preferences_are_upgraded_to_sectioned_layout() {
        let legacy = br#"{"interface_scale":130,"app_update_channel":"beta-6h"}"#;
        let migrated =
            migrate_incoming_preferences(SYNCED_ARTIFACTS[0], legacy).expect("legacy incoming");
        let bytes = migrated.expect("legacy 需要迁移为分域结构");
        let document: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            document["schema_version"],
            crate::modules::config_migration::PREFERENCES_CURRENT_SCHEMA
        );
        assert_eq!(document["appearance"]["interface_scale"], 130);
        assert_eq!(document["updates"]["app_update_channel"], "beta");
    }

    #[test]
    fn incoming_future_preferences_and_non_preferences_are_handled() {
        // 过新载荷：显式失败，不得写入本地。
        let future = serde_json::json!({
            "schema_version": crate::modules::config_migration::PREFERENCES_CURRENT_SCHEMA + 1,
            "appearance": { "interface_scale": 130 },
        });
        let bytes = serde_json::to_vec(&future).unwrap();
        assert!(migrate_incoming_preferences(SYNCED_ARTIFACTS[0], &bytes).is_err());
        // 非偏好文件：不参与偏好迁移，原样交给整批应用。
        assert!(migrate_incoming_preferences(SYNCED_ARTIFACTS[1], b"{}")
            .expect("non-preferences")
            .is_none());
    }
}
