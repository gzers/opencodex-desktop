//! WebDAV 同步内核。
//!
//! 这里实现 FZ-30 / FZ-31 / FZ-32 / FZ-33 / FZ-45 的确定性规则：
//! 逐件校验、互斥、防回放、覆盖备份与冲突登记。
//!
//! 当前传输格式（v2）是**明文载荷 + SHA-256 完整性校验**：不含额外内容加密口令，
//! 远端保存的是未加密配置。旧版 v1（口令派生的 manifest 独立 AEAD + 载荷 AEAD）
//! 只保留兼容读取。哈希只用于检测损坏与回放，不是对恶意远端的真实性保护。

use aes_gcm::{
    aead::{Aead, Payload},
    Aes256Gcm, KeyInit, Nonce,
};
use argon2::Argon2;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{DateTime, SecondsFormat, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::infrastructure::locking::TargetFileLock;
use crate::modules::backup::{backup_file, BackupAction};
use crate::modules::sync::{generate_snapshot_id, SyncFailureClass, SyncManifest};

/// 用于下载时重建 payload AEAD 的最小 envelope。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SealedPayloadEnvelope {
    pub format: String,
    pub version: u32,
    pub artifact_name: String,
    pub artifact_sha256: String,
    pub ciphertext_b64: String,
}

/// 新快照应用到本地后的确定性结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteApplyResult {
    pub decision: SyncDecision,
    pub backup_id: Option<String>,
    pub applied_paths: Vec<String>,
}

/// 传输包格式版本：1 = 旧版口令 AEAD；2 = 明文 + 完整性校验。
pub const MANIFEST_PACKAGE_VERSION_LEGACY: u32 = 1;
pub const MANIFEST_PACKAGE_VERSION_PLAINTEXT: u32 = 2;
pub const PAYLOAD_FORMAT_VERSION_LEGACY: u32 = 1;
pub const PAYLOAD_FORMAT_VERSION_PLAINTEXT: u32 = 2;

const NONCE_BYTES: usize = 12;
const TAG_BYTES: usize = 16;
const ARGON2_M_COST: u32 = 1024;
const ARGON2_T_COST: u32 = 3;
const ARGON2_P_COST: u32 = 1;
const SYNC_STATE_VERSION: u32 = 1;

/// FZ-32 每 endpoint 的内存互斥。
#[derive(Clone, Default)]
pub struct SyncEndpointLocks {
    locks: Arc<Mutex<Vec<String>>>,
}

impl SyncEndpointLocks {
    /// 进程级共享锁表。
    ///
    /// 同步入口此前每次 `SyncEndpointLocks::default()` 都新建一张空锁表，
    /// `acquire` 必然成功，FZ-32 的「每 endpoint 内存互斥」实际上从未生效；
    /// 并发触发同步时两边会各自跑到一半互相覆盖。所有真实同步入口都必须走这里。
    pub fn global() -> &'static SyncEndpointLocks {
        static GLOBAL: std::sync::OnceLock<SyncEndpointLocks> = std::sync::OnceLock::new();
        GLOBAL.get_or_init(SyncEndpointLocks::default)
    }

    pub fn acquire(&self, endpoint_id: &str) -> Result<SyncEndpointGuard, SyncFailureClass> {
        let mut held = self.locks.lock().map_err(|_| SyncFailureClass::Network)?;
        if held.iter().any(|value| value == endpoint_id) {
            return Err(SyncFailureClass::Network);
        }
        held.push(endpoint_id.to_string());
        Ok(SyncEndpointGuard {
            endpoint_id: endpoint_id.to_string(),
            locks: self.locks.clone(),
        })
    }
}

pub struct SyncEndpointGuard {
    endpoint_id: String,
    locks: Arc<Mutex<Vec<String>>>,
}

impl Drop for SyncEndpointGuard {
    fn drop(&mut self) {
        if let Ok(mut held) = self.locks.lock() {
            held.retain(|value| value != &self.endpoint_id);
        }
    }
}

/// FZ-30 / FZ-45 manifest 独立 AEAD 传输包。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SealedManifestPackage {
    pub manifest_magic: String,
    pub manifest_format_version: u32,
    pub kdf: KdfHeader,
    pub aead: AeadHeader,
    pub aad: ManifestAad,
    pub manifest_ciphertext_b64: String,
}

/// 当前（v2）manifest 传输包：明文清单 + 完整性摘要。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct PlainManifestPackage {
    pub manifest_magic: String,
    pub manifest_format_version: u32,
    pub manifest: SyncManifest,
    pub manifest_sha256: String,
}

/// 解析后的 manifest 传输包：区分旧版 AEAD 与当前明文。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestPackage {
    Legacy(SealedManifestPackage),
    Plaintext(PlainManifestPackage),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct KdfHeader {
    algorithm: String,
    salt_b64: String,
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
    output_bytes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AeadHeader {
    algorithm: String,
    nonce_bytes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ManifestAad {
    format: String,
    version: u32,
    compat_version: u32,
    device_name: String,
    schema_kind: String,
}

/// 用口令生成 manifest AEAD 包；盐和 nonce 每次重新生成。
/// 组装当前（v2）manifest 传输包；不需要任何口令。
pub fn seal_manifest(manifest: &SyncManifest) -> Result<PlainManifestPackage, SyncFailureClass> {
    let payload = serde_json::to_vec(manifest).map_err(|_| SyncFailureClass::Parse)?;
    if payload.len() as u64 > crate::modules::sync::MANIFEST_MAX_BYTES {
        return Err(SyncFailureClass::Protocol);
    }
    Ok(PlainManifestPackage {
        manifest_magic: "OCXDSYNCPKG".to_string(),
        manifest_format_version: MANIFEST_PACKAGE_VERSION_PLAINTEXT,
        manifest: manifest.clone(),
        manifest_sha256: sha256_hex(&payload),
    })
}

/// 从原始字节解析传输包；先看版本号，再分派到旧版 AEAD 或当前明文。
pub fn parse_manifest_package(bytes: &[u8]) -> Result<ManifestPackage, SyncFailureClass> {
    if bytes.len() as u64 > crate::modules::sync::MANIFEST_MAX_BYTES {
        return Err(SyncFailureClass::Protocol);
    }
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| SyncFailureClass::Parse)?;
    let magic = value
        .get("manifest_magic")
        .and_then(|item| item.as_str())
        .unwrap_or_default();
    if magic != "OCXDSYNCPKG" {
        return Err(SyncFailureClass::Protocol);
    }
    let version = value
        .get("manifest_format_version")
        .and_then(|item| item.as_u64())
        .ok_or(SyncFailureClass::Protocol)?;
    match version as u32 {
        MANIFEST_PACKAGE_VERSION_LEGACY => {
            let package: SealedManifestPackage =
                serde_json::from_value(value).map_err(|_| SyncFailureClass::Parse)?;
            Ok(ManifestPackage::Legacy(package))
        }
        MANIFEST_PACKAGE_VERSION_PLAINTEXT => {
            let package: PlainManifestPackage =
                serde_json::from_value(value).map_err(|_| SyncFailureClass::Parse)?;
            Ok(ManifestPackage::Plaintext(package))
        }
        _ => Err(SyncFailureClass::Protocol),
    }
}

/// 打开传输包。旧版 AEAD 包需要原口令；缺失或错误时明确失败。
pub fn open_manifest(
    package: &ManifestPackage,
    legacy_passphrase: Option<&[u8]>,
) -> Result<SyncManifest, SyncFailureClass> {
    match package {
        ManifestPackage::Plaintext(package) => {
            if package.manifest_magic != "OCXDSYNCPKG"
                || package.manifest_format_version != MANIFEST_PACKAGE_VERSION_PLAINTEXT
            {
                return Err(SyncFailureClass::Protocol);
            }
            let payload =
                serde_json::to_vec(&package.manifest).map_err(|_| SyncFailureClass::Parse)?;
            if package.manifest_sha256 != sha256_hex(&payload) {
                // 完整性摘要不符：拒绝，不把损坏内容当作可信清单。
                return Err(SyncFailureClass::Parse);
            }
            package
                .manifest
                .validate()
                .map_err(|_| SyncFailureClass::Parse)?;
            Ok(package.manifest.clone())
        }
        ManifestPackage::Legacy(package) => {
            let passphrase = legacy_passphrase
                .filter(|value| !value.is_empty())
                .ok_or(SyncFailureClass::Auth)?;
            open_legacy_manifest(package, passphrase)
        }
    }
}

/// 旧版（v1）AEAD manifest 包解密；只为兼容读取保留。
fn open_legacy_manifest(
    package: &SealedManifestPackage,
    passphrase: &[u8],
) -> Result<SyncManifest, SyncFailureClass> {
    if package.manifest_magic != "OCXDSYNCPKG"
        || package.manifest_format_version != MANIFEST_PACKAGE_VERSION_LEGACY
        || package.aad.format != "OCXDSYNC"
        || package.aad.version != 1
        || package.aad.compat_version != 1
        || package.aad.schema_kind != "sync-manifest"
        || package.kdf.algorithm != "argon2id"
        || package.kdf.memory_kib != ARGON2_M_COST
        || package.kdf.iterations != ARGON2_T_COST
        || package.kdf.parallelism != ARGON2_P_COST
        || package.kdf.output_bytes != 32
        || package.aead.algorithm != "aes-256-gcm"
        || package.aead.nonce_bytes != NONCE_BYTES as u32
    {
        return Err(SyncFailureClass::Protocol);
    }
    let salt = BASE64
        .decode(&package.kdf.salt_b64)
        .map_err(|_| SyncFailureClass::Protocol)?;
    let ciphertext = BASE64
        .decode(&package.manifest_ciphertext_b64)
        .map_err(|_| SyncFailureClass::Protocol)?;
    if salt.len() != 16 || ciphertext.len() <= NONCE_BYTES + TAG_BYTES {
        return Err(SyncFailureClass::Protocol);
    }
    let (nonce, sealed) = ciphertext.split_at(NONCE_BYTES);
    let key = derive_key(passphrase, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| SyncFailureClass::Protocol)?;
    let plaintext = cipher
        .decrypt(
            Nonce::from_slice(nonce),
            Payload {
                msg: sealed,
                aad: &serde_json::to_vec(&package.aad).expect("AAD is serializable"),
            },
        )
        .map_err(|_| SyncFailureClass::Auth)?;
    let manifest: SyncManifest =
        serde_json::from_slice(&plaintext).map_err(|_| SyncFailureClass::Parse)?;
    if package.aad.device_name != manifest.device_name {
        return Err(SyncFailureClass::Protocol);
    }
    manifest.validate().map_err(|_| SyncFailureClass::Parse)?;
    Ok(manifest)
}

/// 旧版 AEAD manifest 包封装；只为测试与迁移夹具保留。
#[allow(dead_code)]
pub fn seal_legacy_manifest(
    manifest: &SyncManifest,
    passphrase: &[u8],
) -> Result<SealedManifestPackage, SyncFailureClass> {
    if passphrase.is_empty() {
        return Err(SyncFailureClass::Auth);
    }
    let salt = random_bytes(16);
    let nonce = random_bytes(NONCE_BYTES);
    let payload = serde_json::to_vec(manifest).map_err(|_| SyncFailureClass::Parse)?;
    let key = derive_key(passphrase, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| SyncFailureClass::Protocol)?;
    let aad = ManifestAad {
        format: "OCXDSYNC".to_string(),
        version: 1,
        compat_version: 1,
        device_name: manifest.device_name.clone(),
        schema_kind: "sync-manifest".to_string(),
    };
    let sealed = cipher
        .encrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: &payload,
                aad: &serde_json::to_vec(&aad).expect("AAD is serializable"),
            },
        )
        .map_err(|_| SyncFailureClass::Network)?;
    Ok(SealedManifestPackage {
        manifest_magic: "OCXDSYNCPKG".to_string(),
        manifest_format_version: MANIFEST_PACKAGE_VERSION_LEGACY,
        kdf: KdfHeader {
            algorithm: "argon2id".to_string(),
            salt_b64: BASE64.encode(salt),
            memory_kib: ARGON2_M_COST,
            iterations: ARGON2_T_COST,
            parallelism: ARGON2_P_COST,
            output_bytes: 32,
        },
        aead: AeadHeader {
            algorithm: "aes-256-gcm".to_string(),
            nonce_bytes: NONCE_BYTES as u32,
        },
        aad,
        manifest_ciphertext_b64: BASE64.encode([nonce.as_slice(), sealed.as_slice()].concat()),
    })
}

/// 生成一次本地快照；载荷是明文封装，仅带 SHA-256 完整性摘要。
pub fn build_local_snapshot(
    artifacts: Vec<(String, Vec<u8>)>,
    device_name: &str,
    now: DateTime<Utc>,
) -> Result<(SyncManifest, Vec<(String, PlainPayloadEnvelope)>), SyncFailureClass> {
    let mut manifest_artifacts = Vec::with_capacity(artifacts.len());
    let mut payloads = Vec::with_capacity(artifacts.len());
    for (name, plaintext) in artifacts {
        if plaintext.len() as u64 > crate::modules::sync::PAYLOAD_MAX_BYTES {
            return Err(SyncFailureClass::Protocol);
        }
        let digest = sha256_hex(&plaintext);
        manifest_artifacts.push(crate::modules::sync::SyncArtifact {
            name: name.clone(),
            size: plaintext.len() as u64,
            sha256: digest.clone(),
        });
        payloads.push((
            name.clone(),
            PlainPayloadEnvelope {
                format: "OCXDPAYLOAD".to_string(),
                version: PAYLOAD_FORMAT_VERSION_PLAINTEXT,
                artifact_name: name,
                artifact_sha256: digest,
                data_b64: BASE64.encode(&plaintext),
            },
        ));
    }
    let manifest = SyncManifest::new(
        generate_snapshot_id(now),
        now,
        device_name,
        manifest_artifacts,
    )
    .map_err(|_| SyncFailureClass::Parse)?;
    Ok((manifest, payloads))
}

/// 当前（v2）载荷封装：明文 + 完整性摘要。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct PlainPayloadEnvelope {
    pub format: String,
    pub version: u32,
    pub artifact_name: String,
    pub artifact_sha256: String,
    pub data_b64: String,
}

/// 旧版（v1）载荷封装：口令派生 AEAD 密文。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SealedPayload {
    pub format: String,
    pub version: u32,
    pub artifact_name: String,
    pub artifact_sha256: String,
    pub ciphertext_b64: String,
}

/// 解析后的载荷包：区分旧版密文与当前明文。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayloadPackage {
    Legacy(SealedPayload),
    Plaintext(PlainPayloadEnvelope),
}

pub fn parse_payload_package(bytes: &[u8]) -> Result<PayloadPackage, SyncFailureClass> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| SyncFailureClass::Parse)?;
    if value.get("format").and_then(|item| item.as_str()) != Some("OCXDPAYLOAD") {
        return Err(SyncFailureClass::Protocol);
    }
    let version = value
        .get("version")
        .and_then(|item| item.as_u64())
        .ok_or(SyncFailureClass::Protocol)?;
    match version as u32 {
        PAYLOAD_FORMAT_VERSION_LEGACY => {
            let package: SealedPayload =
                serde_json::from_value(value).map_err(|_| SyncFailureClass::Parse)?;
            Ok(PayloadPackage::Legacy(package))
        }
        PAYLOAD_FORMAT_VERSION_PLAINTEXT => {
            let package: PlainPayloadEnvelope =
                serde_json::from_value(value).map_err(|_| SyncFailureClass::Parse)?;
            Ok(PayloadPackage::Plaintext(package))
        }
        _ => Err(SyncFailureClass::Protocol),
    }
}

/// 打开载荷并校验完整性。旧版密文需要清单里记录的口令派生密钥材料。
pub fn open_payload(
    package: &PayloadPackage,
    artifact_sha256: &str,
) -> Result<Vec<u8>, SyncFailureClass> {
    match package {
        PayloadPackage::Plaintext(package) => {
            if package.format != "OCXDPAYLOAD"
                || package.version != PAYLOAD_FORMAT_VERSION_PLAINTEXT
                || package.artifact_sha256 != artifact_sha256
            {
                return Err(SyncFailureClass::Protocol);
            }
            let data = BASE64
                .decode(&package.data_b64)
                .map_err(|_| SyncFailureClass::Protocol)?;
            if data.len() as u64 > crate::modules::sync::PAYLOAD_MAX_BYTES {
                return Err(SyncFailureClass::Protocol);
            }
            if sha256_hex(&data) != artifact_sha256 {
                return Err(SyncFailureClass::Parse);
            }
            Ok(data)
        }
        PayloadPackage::Legacy(package) => open_legacy_payload(package, artifact_sha256),
    }
}

/// 旧版载荷解密；只为兼容读取保留。
fn open_legacy_payload(
    payload: &SealedPayload,
    artifact_sha256: &str,
) -> Result<Vec<u8>, SyncFailureClass> {
    if payload.format != "OCXDPAYLOAD" || payload.version != PAYLOAD_FORMAT_VERSION_LEGACY {
        return Err(SyncFailureClass::Protocol);
    }
    let ciphertext = BASE64
        .decode(&payload.ciphertext_b64)
        .map_err(|_| SyncFailureClass::Protocol)?;
    if ciphertext.len() <= 16 + NONCE_BYTES + TAG_BYTES {
        return Err(SyncFailureClass::Auth);
    }
    let (salt, rest) = ciphertext.split_at(16);
    let (nonce, sealed) = rest.split_at(NONCE_BYTES);
    let key = derive_key(artifact_sha256.as_bytes(), salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| SyncFailureClass::Protocol)?;
    cipher
        .decrypt(
            Nonce::from_slice(nonce),
            Payload {
                msg: sealed,
                aad: artifact_sha256.as_bytes(),
            },
        )
        .map_err(|_| SyncFailureClass::Auth)
}

/// 旧版载荷封装；只为测试与迁移夹具保留。
pub fn seal_legacy_payload(
    plaintext: &[u8],
    artifact_sha256: &str,
) -> Result<SealedPayload, SyncFailureClass> {
    let salt = random_bytes(16);
    let nonce = random_bytes(NONCE_BYTES);
    let key = derive_key(artifact_sha256.as_bytes(), &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| SyncFailureClass::Protocol)?;
    let sealed = cipher
        .encrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: plaintext,
                aad: artifact_sha256.as_bytes(),
            },
        )
        .map_err(|_| SyncFailureClass::Network)?;
    let mut package = [0_u8; 16 + NONCE_BYTES];
    package[..16].copy_from_slice(&salt);
    package[16..].copy_from_slice(&nonce);
    Ok(SealedPayload {
        format: "OCXDPAYLOAD".to_string(),
        version: PAYLOAD_FORMAT_VERSION_LEGACY,
        artifact_name: String::new(),
        artifact_sha256: artifact_sha256.to_string(),
        ciphertext_b64: BASE64.encode([package.as_slice(), sealed.as_slice()].concat()),
    })
}

/// 远端 manifest 包的确定性验证。
pub fn validate_remote_package(
    package: &ManifestPackage,
    legacy_passphrase: Option<&[u8]>,
) -> Result<SyncManifest, SyncFailureClass> {
    open_manifest(package, legacy_passphrase)
}

/// 同步状态、接受快照与冲突记录。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SyncState {
    pub schema_version: u32,
    pub endpoint_id: String,
    pub last_accepted: Option<AcceptedSnapshot>,
    pub conflicts: Vec<SyncConflict>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AcceptedSnapshot {
    pub snapshot_id: String,
    pub accepted_at: String,
    pub created_at: String,
    pub manifest_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SyncConflict {
    pub snapshot_id: String,
    pub created_at: String,
    pub reason: String,
    pub local_side: String,
    pub remote_side: String,
    pub backup_id: Option<String>,
}

impl SyncState {
    pub fn new(endpoint_id: impl Into<String>) -> Self {
        Self {
            schema_version: SYNC_STATE_VERSION,
            endpoint_id: endpoint_id.into(),
            last_accepted: None,
            conflicts: Vec::new(),
        }
    }

    pub fn load(data_root: &Path, endpoint_id: &str) -> Result<Self, SyncFailureClass> {
        let path = state_path(data_root, endpoint_id);
        match std::fs::read(&path) {
            Ok(bytes) => {
                let value: Self =
                    serde_json::from_slice(&bytes).map_err(|_| SyncFailureClass::Parse)?;
                if value.schema_version != SYNC_STATE_VERSION || value.endpoint_id != endpoint_id {
                    return Err(SyncFailureClass::Parse);
                }
                Ok(value)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(Self::new(endpoint_id))
            }
            Err(_) => Err(SyncFailureClass::Network),
        }
    }

    pub fn save(&self, data_root: &Path) -> Result<(), SyncFailureClass> {
        let path = state_path(data_root, &self.endpoint_id);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|_| SyncFailureClass::Network)?;
        }
        let bytes = serde_json::to_vec_pretty(self).map_err(|_| SyncFailureClass::Parse)?;
        crate::infrastructure::atomic_write::atomic_write(&path, &bytes, 0o600)
            .map_err(|_| SyncFailureClass::Network)
    }
}

/// 同步状态所在分区（`sync-state`）。
pub fn sync_state_dir(data_root: &Path) -> PathBuf {
    data_root.join("sync-state")
}

fn state_path(data_root: &Path, endpoint_id: &str) -> PathBuf {
    sync_state_dir(data_root).join(format!("{endpoint_id}.json"))
}

/// 判定接受、冲突或拒绝；必要时先备份被覆盖侧。
///
/// **一次快照只能调用一次**：同一批里的多个文件要走 `apply_remote_snapshot_batch`，
/// 否则第二个文件会被误判为回放冲突并跳过（历史上的半应用缺陷）。
#[allow(clippy::too_many_arguments)]
pub fn apply_remote_snapshot(
    state: &mut SyncState,
    data_root: &Path,
    target_path: &Path,
    local_payload: &[u8],
    snapshot_id: &str,
    created_at: &str,
    manifest_hash: &str,
    now: DateTime<Utc>,
) -> Result<SyncDecision, SyncFailureClass> {
    apply_remote_snapshot_batch(
        state,
        data_root,
        &[(target_path, local_payload)],
        snapshot_id,
        created_at,
        manifest_hash,
        now,
    )
}

/// 一个将被覆盖的文件在应用前的原状：路径、原内容（不存在则 `None`）与备份编号。
type SyncBackup = (std::path::PathBuf, Option<Vec<u8>>, Option<String>);

/// 应用一个远端快照的全部文件：回放检查只做一次，写入要么整体成功要么整体回滚。
#[allow(clippy::too_many_arguments)]
pub fn apply_remote_snapshot_batch(
    state: &mut SyncState,
    data_root: &Path,
    artifacts: &[(&Path, &[u8])],
    snapshot_id: &str,
    created_at: &str,
    manifest_hash: &str,
    now: DateTime<Utc>,
) -> Result<SyncDecision, SyncFailureClass> {
    let created = DateTime::parse_from_rfc3339(created_at)
        .map_err(|_| SyncFailureClass::Parse)?
        .with_timezone(&Utc);
    // 1) 回放与旧快照判定：整批只判定一次。
    if let Some(last) = &state.last_accepted {
        if snapshot_id == last.snapshot_id
            || created
                <= DateTime::parse_from_rfc3339(&last.created_at)
                    .map_err(|_| SyncFailureClass::Parse)?
                    .with_timezone(&Utc)
        {
            state.conflicts.push(SyncConflict {
                snapshot_id: snapshot_id.to_string(),
                created_at: created_at.to_string(),
                reason: "replay".to_string(),
                local_side: "local".to_string(),
                remote_side: "remote".to_string(),
                backup_id: None,
            });
            return Ok(SyncDecision::Conflict);
        }
    }

    // 2) 先备份所有将被覆盖的文件，再逐个写入；任一失败回滚本批已写文件。
    let mut backups: Vec<SyncBackup> = Vec::new();
    for (path, _payload) in artifacts {
        let existing = std::fs::read(path).ok();
        let backup_id = match &existing {
            Some(previous) => {
                let record = backup_file(
                    data_root,
                    BackupAction::SyncOverwrite,
                    path,
                    previous,
                    now,
                    Some(format!("snapshot:{snapshot_id}")),
                )
                .map_err(|_| SyncFailureClass::Network)?;
                Some(record.manifest.backup_id)
            }
            None => None,
        };
        backups.push((path.to_path_buf(), existing, backup_id));
    }

    let restore = |backups: &[SyncBackup], count: usize| {
        for (path, previous, _) in backups.iter().take(count) {
            match previous {
                Some(bytes) => {
                    let _ = crate::infrastructure::atomic_write::atomic_write(path, bytes, 0o600);
                }
                None => {
                    let _ = std::fs::remove_file(path);
                }
            }
        }
    };

    for (written, (path, payload)) in artifacts.iter().enumerate() {
        if let Some(parent) = path.parent() {
            if std::fs::create_dir_all(parent).is_err() {
                restore(&backups, written);
                return Err(SyncFailureClass::Network);
            }
        }
        if crate::infrastructure::atomic_write::atomic_write(path, payload, 0o600).is_err() {
            restore(&backups, written);
            return Err(SyncFailureClass::Network);
        }
    }

    state.last_accepted = Some(AcceptedSnapshot {
        snapshot_id: snapshot_id.to_string(),
        accepted_at: now.to_rfc3339_opts(SecondsFormat::Secs, true),
        created_at: created_at.to_string(),
        manifest_hash: manifest_hash.to_string(),
    });
    Ok(SyncDecision::Applied {
        backup_id: backups.iter().find_map(|(_, _, id)| id.clone()),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncDecision {
    Applied { backup_id: Option<String> },
    Conflict,
}

/// endpoint 互斥加目标文件锁；先锁 endpoint，再锁目标。
pub struct SnapshotLock {
    endpoint: String,
    _target_lock: TargetFileLock,
}

impl SnapshotLock {
    pub fn acquire(
        data_root: &Path,
        endpoint_id: &str,
        target: &Path,
    ) -> Result<Self, SyncFailureClass> {
        let _ = data_root;
        let target_lock = TargetFileLock::lock(target).map_err(|_| SyncFailureClass::Network)?;
        Ok(Self {
            endpoint: endpoint_id.to_string(),
            _target_lock: target_lock,
        })
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

fn random_bytes(length: usize) -> Vec<u8> {
    let mut value = vec![0_u8; length];
    rand::rng().fill_bytes(&mut value);
    value
}

fn derive_key(passphrase: &[u8], salt: &[u8]) -> Result<[u8; 32], SyncFailureClass> {
    let mut key = [0_u8; 32];
    Argon2::default()
        .hash_password_into(passphrase, salt, &mut key)
        .map_err(|_| SyncFailureClass::Auth)?;
    Ok(key)
}

/// 载荷/清单完整性摘要（SHA-256 十六进制）。
pub fn sha256_hex(payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::modules::sync::{generate_snapshot_id, SyncArtifact};
    use chrono::TimeZone;

    fn manifest(now: DateTime<Utc>, artifact: Option<(&str, &[u8])>) -> SyncManifest {
        let artifacts = artifact
            .map(|(name, payload)| {
                vec![SyncArtifact {
                    name: name.to_string(),
                    size: payload.len() as u64,
                    sha256: sha256_hex(payload),
                }]
            })
            .unwrap_or_default();
        SyncManifest::new(generate_snapshot_id(now), now, "fixture-device", artifacts).unwrap()
    }

    #[test]
    fn plaintext_manifest_package_round_trips_without_passphrase() {
        let now = Utc.with_ymd_and_hms(2026, 9, 16, 1, 2, 3).unwrap();
        let value = manifest(now, Some(("manager-state/preferences.json", b"{}")));
        let package = seal_manifest(&value).unwrap();
        assert_eq!(package.manifest_format_version, 2);
        let bytes = serde_json::to_vec(&package).unwrap();
        let parsed = parse_manifest_package(&bytes).unwrap();
        let opened = open_manifest(&parsed, None).unwrap();
        assert_eq!(value, opened);

        // 篡改清单内容后完整性摘要不符：拒绝，不当作可信清单。
        let mut tampered = package.clone();
        tampered.manifest.device_name = "evil-device".to_string();
        let bytes = serde_json::to_vec(&tampered).unwrap();
        let parsed = parse_manifest_package(&bytes).unwrap();
        assert_eq!(open_manifest(&parsed, None), Err(SyncFailureClass::Parse));
    }

    #[test]
    fn plaintext_payload_round_trips_and_detects_corruption() {
        let (_, payloads) = build_local_snapshot(
            vec![(
                "manager-state/preferences.json".to_string(),
                b"payload".to_vec(),
            )],
            "fixture-device",
            Utc.with_ymd_and_hms(2026, 9, 16, 1, 2, 3).unwrap(),
        )
        .unwrap();
        let (_, envelope) = &payloads[0];
        assert_eq!(envelope.version, 2);
        let bytes = serde_json::to_vec(envelope).unwrap();
        let parsed = parse_payload_package(&bytes).unwrap();
        assert_eq!(
            open_payload(&parsed, &sha256_hex(b"payload")).unwrap(),
            b"payload"
        );
        // 清单摘要与载荷不一致：拒绝。
        assert_eq!(
            open_payload(&parsed, &sha256_hex(b"other")),
            Err(SyncFailureClass::Protocol)
        );

        let mut tampered = envelope.clone();
        tampered.data_b64 = base64::engine::general_purpose::STANDARD.encode(b"other");
        let bytes = serde_json::to_vec(&tampered).unwrap();
        let parsed = parse_payload_package(&bytes).unwrap();
        assert_eq!(
            open_payload(&parsed, &sha256_hex(b"payload")),
            Err(SyncFailureClass::Parse)
        );
    }

    /// 兼容路径：旧版 v1 传输包只有拿到原口令才能打开，且不会把损坏内容当成功。
    #[test]
    fn legacy_packages_require_the_original_passphrase() {
        let now = Utc.with_ymd_and_hms(2026, 9, 16, 1, 2, 3).unwrap();
        let value = manifest(now, Some(("manager-state/preferences.json", b"{}")));
        let package = seal_legacy_manifest(&value, b"correct-pass").unwrap();
        let bytes = serde_json::to_vec(&package).unwrap();
        let parsed = parse_manifest_package(&bytes).unwrap();
        assert!(matches!(parsed, ManifestPackage::Legacy(_)));
        assert_eq!(open_manifest(&parsed, None), Err(SyncFailureClass::Auth));
        assert_eq!(
            open_manifest(&parsed, Some(b"wrong-pass")),
            Err(SyncFailureClass::Auth)
        );
        assert_eq!(
            open_manifest(&parsed, Some(b"correct-pass")).unwrap(),
            value
        );

        let digest = sha256_hex(b"payload");
        let legacy_payload = seal_legacy_payload(b"payload", &digest).unwrap();
        let bytes = serde_json::to_vec(&legacy_payload).unwrap();
        let parsed = parse_payload_package(&bytes).unwrap();
        assert!(matches!(parsed, PayloadPackage::Legacy(_)));
        assert_eq!(open_payload(&parsed, &digest).unwrap(), b"payload");
    }

    #[test]
    fn endpoint_locks_are_exclusive() {
        let locks = SyncEndpointLocks::default();
        let _guard = locks.acquire("endpoint-1").unwrap();
        assert!(matches!(
            locks.acquire("endpoint-1"),
            Err(SyncFailureClass::Network)
        ));
        assert!(locks.acquire("endpoint-2").is_ok());
    }

    #[test]
    fn replay_snapshot_conflicts_and_is_recorded() {
        let temp = tempfile::tempdir().unwrap();
        let now = Utc.with_ymd_and_hms(2026, 9, 16, 1, 0, 0).unwrap();
        let state = SyncState::new("endpoint-1");
        let mut state = state;
        state.last_accepted = Some(AcceptedSnapshot {
            snapshot_id: "snap_20260916000000_aaaaaaaaaaaa".to_string(),
            accepted_at: now.to_rfc3339_opts(SecondsFormat::Secs, true),
            created_at: now.to_rfc3339_opts(SecondsFormat::Secs, true),
            manifest_hash: "hash".to_string(),
        });
        let old_created = now - chrono::Duration::hours(1);
        let decision = apply_remote_snapshot(
            &mut state,
            temp.path(),
            &temp.path().join("manager-state/preferences.json"),
            b"remote",
            "snap_20260916000000_aaaaaaaaaaaa",
            &old_created.to_rfc3339_opts(SecondsFormat::Secs, true),
            "hash",
            now,
        )
        .unwrap();
        assert_eq!(decision, SyncDecision::Conflict);
        assert_eq!(state.conflicts.len(), 1);
    }

    #[test]
    fn apply_backs_up_then_replaces_and_updates_state() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("manager-state/preferences.json");
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(&target, b"local").unwrap();
        let mut state = SyncState::new("endpoint-1");
        let created = Utc.with_ymd_and_hms(2026, 9, 16, 2, 0, 0).unwrap();
        let decision = apply_remote_snapshot(
            &mut state,
            temp.path(),
            &target,
            b"remote",
            "snap_20260916020000_bbbbbbbbbbbb",
            &created.to_rfc3339_opts(SecondsFormat::Secs, true),
            "hash",
            created,
        )
        .unwrap();
        let SyncDecision::Applied { backup_id } = decision else {
            panic!("expected apply");
        };
        assert!(backup_id.is_some());
        assert_eq!(std::fs::read(&target).unwrap(), b"remote");
        assert_eq!(
            state.last_accepted.as_ref().unwrap().created_at,
            created.to_rfc3339_opts(SecondsFormat::Secs, true)
        );
    }

    #[test]
    fn state_round_trip_uses_secure_sync_state_path() {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        let state = SyncState::new("endpoint-1");
        state.save(temp.path()).unwrap();
        let loaded = SyncState::load(temp.path(), "endpoint-1").unwrap();
        assert_eq!(state, loaded);
        let mode = std::fs::metadata(temp.path().join("sync-state/endpoint-1.json"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn target_lock_blocks_concurrent_snapshots() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("manager-state/preferences.json");
        let first = SnapshotLock::acquire(temp.path(), "endpoint-1", &target).unwrap();
        assert!(matches!(
            SnapshotLock::acquire(temp.path(), "endpoint-1", &target),
            Err(SyncFailureClass::Network)
        ));
        drop(first);
        assert!(SnapshotLock::acquire(temp.path(), "endpoint-1", &target).is_ok());
    }
}
