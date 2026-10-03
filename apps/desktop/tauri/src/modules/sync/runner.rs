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
    WebDavClient, WebDavConfig, WebDavError, WebDavOperation,
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
            Self::NotConfigured => "同步未配置或内容无效；本地内容未修改。".to_string(),
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
    let client = WebDavClient::production();
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
        let mut staged: Vec<(std::path::PathBuf, &Vec<u8>, String)> = Vec::new();
        for (name, payload) in &downloaded {
            if !payload_is_json(payload) {
                return Err(SyncRunFailure::NotConfigured);
            }
            staged.push((target_path(data_root, name)?, payload, name.clone()));
        }
        let batch: Vec<(&std::path::Path, &[u8])> = staged
            .iter()
            .map(|(path, payload, _)| (path.as_path(), payload.as_slice()))
            .collect();
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
        // 只有真的写入本地时才声称「已应用」；回放/旧快照只登记冲突，不改动本地。
        if matches!(decision, SyncDecision::Applied { .. }) {
            applied_paths = staged
                .iter()
                .map(|(_, _, name)| name.clone())
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
}
