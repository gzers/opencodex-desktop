//! MOD-08：FZ-30 / FZ-31 / FZ-32 / FZ-45 同步契约。
//!
//! 本模块只定义清单、载荷校验、防回放和重试策略；真实 HTTP 客户端
//! 由后续 infrastructure 接入。测试不访问真实 WebDAV 端点。

pub mod config;
pub mod engine;
pub mod runner;

use chrono::{DateTime, SecondsFormat, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::types::status::{ConnectionState, OperationState};

/// FZ-30 冻结 manifest 格式。
pub const MANIFEST_FORMAT: &str = "OCXDSYNC";
pub const TRANSPORT_PACKAGE_MAGIC: &str = "OCXDSYNCPKG";
pub const MANIFEST_FORMAT_VERSION: u32 = 1;
pub const COMPAT_VERSION: u32 = 1;

/// FZ-31 冻结传输与载荷上限。
pub const MANIFEST_MAX_BYTES: u64 = 16 * 1024 * 1024;
pub const PAYLOAD_MAX_BYTES: u64 = 256 * 1024 * 1024;
pub const SNAPSHOT_MAX_ITEMS: usize = 10_000;
pub const SNAPSHOT_MAX_BYTES: u64 = 1024 * 1024 * 1024;
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
pub const ITEM_TIMEOUT: Duration = Duration::from_secs(10 * 60);
pub const TOTAL_TIMEOUT: Duration = Duration::from_secs(30 * 60);
pub const RETRY_DELAYS: [Duration; 3] = [
    Duration::from_secs(2),
    Duration::from_secs(4),
    Duration::from_secs(8),
];

/// manifest 单件。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SyncArtifact {
    pub name: String,
    pub size: u64,
    pub sha256: String,
}

/// manifest 明文结构；AID 覆盖固定字段由调用方写入 AAD。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SyncManifest {
    pub format: String,
    pub version: u32,
    pub compat_version: u32,
    pub snapshot_id: String,
    pub created_at: String,
    pub device_name: String,
    pub artifacts: Vec<SyncArtifact>,
}

impl SyncManifest {
    pub fn new(
        snapshot_id: impl Into<String>,
        created_at: DateTime<Utc>,
        device_name: impl Into<String>,
        artifacts: Vec<SyncArtifact>,
    ) -> Result<Self, &'static str> {
        let manifest = Self {
            format: MANIFEST_FORMAT.to_string(),
            version: MANIFEST_FORMAT_VERSION,
            compat_version: COMPAT_VERSION,
            snapshot_id: snapshot_id.into(),
            created_at: created_at.to_rfc3339_opts(SecondsFormat::Secs, true),
            device_name: device_name.into(),
            artifacts,
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.format != MANIFEST_FORMAT {
            return Err("manifest format mismatch");
        }
        if self.version != MANIFEST_FORMAT_VERSION {
            return Err("manifest version mismatch");
        }
        if self.compat_version != COMPAT_VERSION {
            return Err("manifest compat version mismatch");
        }
        if !self.snapshot_id.starts_with("snap_") || self.snapshot_id.len() != 5 + 14 + 1 + 12 {
            return Err("snapshot id shape mismatch");
        }
        if self.created_at.trim().is_empty() {
            return Err("created_at is required");
        }
        if self.device_name.trim().is_empty() {
            return Err("device_name is required");
        }
        if self.artifacts.len() > SNAPSHOT_MAX_ITEMS {
            return Err("snapshot artifact count exceeds limit");
        }
        if self.artifacts.iter().map(|a| a.size).sum::<u64>() > SNAPSHOT_MAX_BYTES {
            return Err("snapshot size exceeds limit");
        }
        for artifact in &self.artifacts {
            if artifact.name.trim().is_empty() || artifact.name.contains("..") {
                return Err("artifact path is rejected");
            }
            if artifact.sha256.len() != 64
                || !artifact.sha256.chars().all(|c| c.is_ascii_hexdigit())
            {
                return Err("artifact sha256 shape mismatch");
            }
        }
        Ok(())
    }
}

/// manifest 传输包。这里只冻结字段，不做真实加密。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SyncManifestPackage {
    pub manifest_magic: String,
    pub manifest_format_version: u32,
    pub kdf: KdfHeader,
    pub aead: AeadHeader,
    pub aad: ManifestAad,
    pub manifest_ciphertext_b64: String,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ManifestAad {
    pub format: String,
    pub version: u32,
    pub compat_version: u32,
    pub device_name: String,
    pub schema_kind: String,
}

impl ManifestAad {
    pub fn for_manifest(manifest: &SyncManifest) -> Self {
        Self {
            format: MANIFEST_FORMAT.to_string(),
            version: MANIFEST_FORMAT_VERSION,
            compat_version: COMPAT_VERSION,
            device_name: manifest.device_name.clone(),
            schema_kind: "sync-manifest".to_string(),
        }
    }
}

/// 最后接受的远端快照事实。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastAcceptedSnapshot {
    pub snapshot_id: String,
    pub accepted_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub manifest_hash: String,
}

/// 下载校验判定。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayDecision {
    Accept,
    Conflict,
    Reject,
}

/// 回放判定：旧 snapshot_id 或 created_at 不晚于本地接受时间都进入冲突。
pub fn replay_decision(
    manifest: &SyncManifest,
    last_accepted: Option<&LastAcceptedSnapshot>,
) -> ReplayDecision {
    let created = match DateTime::parse_from_rfc3339(&manifest.created_at) {
        Ok(value) => value.with_timezone(&Utc),
        Err(_) => return ReplayDecision::Reject,
    };
    let Some(last) = last_accepted else {
        return ReplayDecision::Accept;
    };
    if manifest.snapshot_id == last.snapshot_id {
        return ReplayDecision::Conflict;
    }
    if created <= last.created_at {
        return ReplayDecision::Conflict;
    }
    ReplayDecision::Accept
}

/// 下载逐件校验；缺失、多载荷、越权和哈希不一致都拒绝。
pub fn validate_download(
    manifest: &SyncManifest,
    payloads: &[(String, Vec<u8>)],
) -> Result<(), &'static str> {
    if payloads.len() != manifest.artifacts.len() {
        return Err("payload count does not match manifest");
    }
    for artifact in &manifest.artifacts {
        let Some((_, payload)) = payloads.iter().find(|(name, _)| name == &artifact.name) else {
            return Err("manifest payload is missing");
        };
        if payload.len() as u64 != artifact.size {
            return Err("payload size mismatch");
        }
        if sha256_hex(payload) != artifact.sha256 {
            return Err("payload sha256 mismatch");
        }
    }
    for (name, _) in payloads {
        if !manifest
            .artifacts
            .iter()
            .any(|artifact| &artifact.name == name)
        {
            return Err("payload is not declared in manifest");
        }
    }
    Ok(())
}

/// 生成 `snap_<UTC14>_<12位随机hex>`。
pub fn generate_snapshot_id(now: DateTime<Utc>) -> String {
    let mut random = [0_u8; 6];
    rand::rng().fill_bytes(&mut random);
    format!(
        "snap_{}_{}",
        now.format("%Y%m%d%H%M%S"),
        random
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}

/// 重试策略：认证、回放、解析类失败不重试。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncFailureClass {
    Network,
    Protocol,
    Replay,
    Auth,
    Parse,
}

pub fn retry_plan(failure: SyncFailureClass) -> &'static [Duration] {
    match failure {
        SyncFailureClass::Network => &RETRY_DELAYS,
        SyncFailureClass::Protocol
        | SyncFailureClass::Replay
        | SyncFailureClass::Auth
        | SyncFailureClass::Parse => &[],
    }
}

/// TLS 失败文案；不显示 URL query、凭据或 cookie。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsFailureStage {
    Connect,
    Download,
    Upload,
}

pub fn tls_failure_message(stage: TlsFailureStage) -> &'static str {
    match stage {
        TlsFailureStage::Connect => {
            "TLS 证书校验失败，已停止连接。请确认服务器证书、系统时间与代理设置。"
        }
        TlsFailureStage::Download => "TLS 证书校验失败，已取消下载；本地内容未修改。",
        TlsFailureStage::Upload => "TLS 证书校验失败，已取消上传；远端内容未修改。",
    }
}

/// 内存中一次同步运行事实；真实 HTTP 后续由基础设施执行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncRun {
    pub run_id: String,
    pub direction: SyncDirection,
    pub connection_state: ConnectionState,
    pub operation_state: OperationState,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub items_total: usize,
    pub items_done: usize,
    pub failure_reason: Option<String>,
    pub snapshot_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDirection {
    Upload,
    Download,
}

fn sha256_hex(payload: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(payload);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn artifact(name: &str, payload: &[u8]) -> SyncArtifact {
        SyncArtifact {
            name: name.to_string(),
            size: payload.len() as u64,
            sha256: sha256_hex(payload),
        }
    }

    fn manifest(now: DateTime<Utc>, artifacts: Vec<SyncArtifact>) -> SyncManifest {
        SyncManifest::new(generate_snapshot_id(now), now, "fixture-device", artifacts).unwrap()
    }

    #[test]
    fn snapshot_id_matches_frozen_shape() {
        let now = Utc.with_ymd_and_hms(2026, 9, 15, 1, 2, 3).unwrap();
        let id = generate_snapshot_id(now);
        assert_eq!(id.len(), 5 + 14 + 1 + 12);
        assert!(id.starts_with("snap_20260915010203_"));
        assert!(id
            .rsplit('_')
            .next()
            .unwrap()
            .chars()
            .all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn manifest_validates_frozen_fields_and_limits() {
        let now = Utc::now();
        let value = manifest(now, vec![artifact("manager-state/config.json", b"payload")]);
        assert_eq!(value.format, "OCXDSYNC");
        assert_eq!(value.version, 1);
        assert_eq!(value.compat_version, 1);
        assert!(value.validate().is_ok());

        let mut oversized = value.clone();
        oversized.artifacts[0].size = SNAPSHOT_MAX_BYTES + 1;
        assert_eq!(oversized.validate(), Err("snapshot size exceeds limit"));
    }

    #[test]
    fn aad_is_bound_to_format_version_device_and_schema() {
        let now = Utc::now();
        let value = manifest(now, Vec::new());
        let aad = ManifestAad::for_manifest(&value);
        assert_eq!(aad.format, "OCXDSYNC");
        assert_eq!(aad.version, 1);
        assert_eq!(aad.compat_version, 1);
        assert_eq!(aad.device_name, "fixture-device");
        assert_eq!(aad.schema_kind, "sync-manifest");
    }

    #[test]
    fn replay_older_snapshot_enters_conflict() {
        let now = Utc.with_ymd_and_hms(2026, 9, 15, 0, 0, 0).unwrap();
        let old = manifest(now - chrono::Duration::hours(1), Vec::new());
        let fresh = manifest(now + chrono::Duration::hours(1), Vec::new());
        let last = LastAcceptedSnapshot {
            snapshot_id: "snap_20260915000000_aaaaaaaaaaaa".to_string(),
            accepted_at: now,
            created_at: now,
            manifest_hash: "hash".to_string(),
        };
        assert_eq!(replay_decision(&fresh, Some(&last)), ReplayDecision::Accept);
        assert_eq!(replay_decision(&old, Some(&last)), ReplayDecision::Conflict);
        assert_eq!(replay_decision(&fresh, None), ReplayDecision::Accept);
    }

    #[test]
    fn download_rejects_missing_duplicate_or_bad_payloads() {
        let now = Utc::now();
        let a = b"one";
        let b = b"two";
        let value = manifest(now, vec![artifact("a", a), artifact("b", b)]);
        assert!(validate_download(
            &value,
            &[("a".into(), a.to_vec()), ("b".into(), b.to_vec())]
        )
        .is_ok());

        assert_eq!(
            validate_download(&value, &[("a".into(), a.to_vec())]),
            Err("payload count does not match manifest")
        );
        assert_eq!(
            validate_download(
                &value,
                &[
                    ("a".into(), a.to_vec()),
                    ("b".into(), b.to_vec()),
                    ("c".into(), b.to_vec())
                ]
            ),
            Err("payload count does not match manifest")
        );
        assert_eq!(
            validate_download(
                &value,
                &[("a".into(), a.to_vec()), ("x".into(), b.to_vec())]
            ),
            Err("manifest payload is missing")
        );
        assert_eq!(
            validate_download(
                &value,
                &[("a".into(), b.to_vec()), ("b".into(), b.to_vec())]
            ),
            Err("payload sha256 mismatch")
        );
    }

    #[test]
    fn only_network_failures_retry() {
        assert_eq!(retry_plan(SyncFailureClass::Network), &RETRY_DELAYS);
        assert!(retry_plan(SyncFailureClass::Auth).is_empty());
        assert!(retry_plan(SyncFailureClass::Replay).is_empty());
        assert!(retry_plan(SyncFailureClass::Parse).is_empty());
    }

    #[test]
    fn frozen_timeouts_and_limits_match_contract() {
        assert_eq!(CONNECT_TIMEOUT, Duration::from_secs(15));
        assert_eq!(ITEM_TIMEOUT, Duration::from_secs(600));
        assert_eq!(TOTAL_TIMEOUT, Duration::from_secs(1800));
        assert_eq!(MANIFEST_MAX_BYTES, 16 * 1024 * 1024);
        assert_eq!(PAYLOAD_MAX_BYTES, 256 * 1024 * 1024);
        assert_eq!(SNAPSHOT_MAX_ITEMS, 10_000);
    }

    #[test]
    fn tls_messages_do_not_leak_endpoint_details() {
        assert!(tls_failure_message(TlsFailureStage::Connect).contains("TLS 证书校验失败"));
        assert!(tls_failure_message(TlsFailureStage::Download).contains("本地内容未修改"));
        assert!(tls_failure_message(TlsFailureStage::Upload).contains("远端内容未修改"));
        assert!(!tls_failure_message(TlsFailureStage::Connect).contains("http"));
    }

    #[test]
    fn sync_run_tracks_progress_and_failure() {
        let now = Utc::now();
        let run = SyncRun {
            run_id: "sync-1".to_string(),
            direction: SyncDirection::Upload,
            connection_state: ConnectionState::Syncing,
            operation_state: OperationState::Applying,
            started_at: now,
            finished_at: None,
            items_total: 2,
            items_done: 1,
            failure_reason: None,
            snapshot_id: None,
        };
        assert_eq!(run.items_done, 1);
        assert!(run.finished_at.is_none());
    }
}
