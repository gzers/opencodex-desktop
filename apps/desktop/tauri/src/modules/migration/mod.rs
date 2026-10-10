//! MOD-05：配置导出与导入。
//!
//! 导出和导入都作用于显式数据根；真实凭据仍在 Keychain，容器只保留引用。
//! 当前格式（v2）是**不含额外口令的明文容器**：用 `INTEGRITY_ALGORITHM` 检测损坏，
//! 不声称端到端加密。旧版 v1 是 Argon2id + AES-256-GCM 加密容器，仅保留兼容读取。

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::Argon2;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use zeroize::Zeroizing;

use crate::errors::AppError;
use crate::infrastructure::hash::sha256_hex;
use crate::infrastructure::locking::TargetFileLock;
use crate::modules::backup;
use crate::modules::container::{
    self, ContainerDocument, ContainerHeader, AEAD_NONCE_BYTES, KDF_SALT_BYTES,
};
use crate::modules::extensions::projection::ExtensionProjection;
use crate::modules::extensions::projection::CONFIG_RELATIVE_PATH as EXTENSION_CONFIG_RELATIVE_PATH;
use crate::modules::extensions::{ClientTarget, CLIENT_IDS};
use crate::modules::preferences::Preferences;
use crate::modules::sync::config::{SyncConfig, SYNC_ENDPOINTS_RELATIVE_PATH};

pub const EXPORT_FILE_NAME: &str = "opencodex-config.ocxdconf";
pub const EXPORTS_RELATIVE_PATH: &str = "exports";
pub const PREFERENCES_RELATIVE_PATH: &str = "manager-state/preferences.json";

const CONTAINER_FILE_MAGIC: &str = "OCXDCONF";
/// 当前写出格式：不含额外口令的明文容器。
const CONTAINER_FILE_FORMAT_VERSION: u32 = 2;
/// 旧版：口令保护的加密容器。只保留兼容读取。
const CONTAINER_FILE_FORMAT_VERSION_LEGACY: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportResult {
    pub path: std::path::PathBuf,
    pub backup_id: Option<String>,
    pub document_sha256: String,
    /// 写出的容器格式版本。
    pub format_version: u32,
    /// 本次导出实际包含的区段名。
    pub sections: Vec<String>,
    /// 容器明确不包含的内容（与 `container::EXPORT_EXCLUSIONS` 一致）。
    pub excluded: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportResult {
    pub backup_id: String,
    pub document_sha256: String,
    /// 读入的容器格式版本（1 = 旧加密容器，2 = 明文容器）。
    pub format_version: u32,
    /// 本次导入实际应用的区段名。
    pub applied_sections: Vec<String>,
    /// 容器里存在但本次**未应用**的区段名与原因。
    pub skipped_sections: Vec<ImportSectionSkip>,
    /// 容器明确不包含的内容（与 `container::EXPORT_EXCLUSIONS` 一致）。
    pub excluded: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSectionSkip {
    pub section: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
/// 旧版（v1）加密容器文件外壳。
struct ContainerFile {
    container_magic: String,
    container_format_version: u32,
    header: ContainerHeader,
    payload_b64: String,
}

/// 当前（v2）明文容器文件外壳。载荷直接内联，`integrity` 只用于检测损坏。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
struct ContainerFilePlaintext {
    container_magic: String,
    container_format_version: u32,
    created_at: String,
    app_version: String,
    structure_version: u32,
    integrity: IntegrityHeader,
    document: ContainerDocument,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
struct IntegrityHeader {
    algorithm: String,
    digest: String,
}

/// 解析后的容器：区分旧加密载荷与当前明文载荷。
enum ParsedContainer {
    Legacy {
        header: ContainerHeader,
        payload_b64: String,
    },
    Plaintext(ContainerDocument),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationError {
    NotConfigured,
    LimitExceeded,
    ContainerFormat,
    /// 旧版加密容器需要口令，但本次没有提供。
    PassphraseRequired,
    /// 容器版本高于当前支持范围。
    UnsupportedVersion,
    Authentication,
    CorruptedDocument,
    Preferences,
    Backup,
    AtomicWrite,
}

impl MigrationError {
    fn as_app_error(self) -> AppError {
        let (operation, detail) = match self {
            Self::NotConfigured => ("validate data root", "data root is not configured"),
            Self::LimitExceeded => ("validate migration payload", "payload exceeds frozen limit"),
            Self::ContainerFormat => ("validate migration container", "container is invalid"),
            // 旧口令缺失有独立错误码，前端据此提示输入原口令。
            Self::PassphraseRequired => return AppError::PassphraseRequired,
            Self::UnsupportedVersion => (
                "validate migration container",
                "container format version is newer than supported",
            ),
            Self::Authentication => ("decrypt migration container", "authentication failed"),
            Self::CorruptedDocument => (
                "parse migration document",
                "document is invalid or corrupted",
            ),
            Self::Preferences => (
                "serialize preferences",
                "preferences could not be serialized",
            ),
            Self::Backup => ("backup before migration", "backup creation failed"),
            Self::AtomicWrite => ("replace migration document", "atomic write failed"),
        };
        AppError::FileSystem {
            operation: operation.to_string(),
            detail: detail.to_string(),
        }
    }
}

impl From<MigrationError> for AppError {
    fn from(value: MigrationError) -> Self {
        value.as_app_error()
    }
}

fn export_path(data_root: &Path) -> std::path::PathBuf {
    data_root.join(EXPORTS_RELATIVE_PATH).join(EXPORT_FILE_NAME)
}

fn validate_data_root(data_root: &Path) -> Result<(), MigrationError> {
    if !data_root.is_absolute() {
        return Err(MigrationError::NotConfigured);
    }
    if !data_root.is_dir() {
        return Err(MigrationError::NotConfigured);
    }
    Ok(())
}

fn validate_structure(data_root: &Path) -> Result<(), MigrationError> {
    match crate::modules::data_root::validate_structure(data_root) {
        Ok(crate::modules::data_root::StructureValidation::Valid) => Ok(()),
        Ok(_) => Err(MigrationError::ContainerFormat),
        Err(_) => Err(MigrationError::ContainerFormat),
    }
}

#[doc(hidden)]
pub fn derive_key_for_test(
    passphrase: &[u8],
    salt: &[u8],
) -> Result<Zeroizing<[u8; 32]>, MigrationError> {
    let salt = argon2::password_hash::SaltString::encode_b64(salt)
        .map_err(|_| MigrationError::ContainerFormat)?;
    let argon = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(
            crate::modules::container::KDF_MEMORY_KIB,
            crate::modules::container::KDF_ITERATIONS,
            crate::modules::container::KDF_PARALLELISM,
            Some(crate::modules::container::KDF_OUTPUT_BYTES as usize),
        )
        .map_err(|_| MigrationError::ContainerFormat)?,
    );
    let mut key = Zeroizing::new([0_u8; 32]);
    argon
        .hash_password_into(passphrase, salt.as_str().as_bytes(), key.as_mut())
        .map_err(|_| MigrationError::ContainerFormat)?;
    Ok(key)
}

/// 旧版容器的 AAD：绑定容器头，避免头部被替换。
fn aad(header: &ContainerHeader) -> Result<Vec<u8>, MigrationError> {
    serde_json::to_vec(header).map_err(|_| MigrationError::ContainerFormat)
}

#[doc(hidden)]
pub fn open_document_for_test(
    payload_b64: &str,
    header: &ContainerHeader,
    key: &Zeroizing<[u8; 32]>,
) -> Result<ContainerDocument, MigrationError> {
    let payload = BASE64_STANDARD
        .decode(payload_b64)
        .map_err(|_| MigrationError::ContainerFormat)?;
    if payload.len() as u64 > container::CIPHERTEXT_MAX_BYTES {
        return Err(MigrationError::LimitExceeded);
    }
    if payload.len() < AEAD_NONCE_BYTES {
        return Err(MigrationError::ContainerFormat);
    }
    let (nonce, ciphertext) = payload.split_at(AEAD_NONCE_BYTES);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key.as_slice()));
    let plaintext = cipher
        .decrypt(
            Nonce::from_slice(nonce),
            Payload {
                msg: ciphertext,
                aad: aad(header)?.as_ref(),
            },
        )
        .map_err(|_| MigrationError::Authentication)?;
    if plaintext.len() as u64 > container::PLAINTEXT_MAX_BYTES {
        return Err(MigrationError::LimitExceeded);
    }
    let document: ContainerDocument =
        serde_json::from_slice(&plaintext).map_err(|_| MigrationError::CorruptedDocument)?;
    container::validate_document(&document).map_err(map_container_error)?;
    Ok(document)
}

fn map_container_error(error: container::ContainerError) -> MigrationError {
    match error {
        container::ContainerError::LimitExceeded => MigrationError::LimitExceeded,
        container::ContainerError::HeaderFutureVersion => MigrationError::UnsupportedVersion,
        _ => MigrationError::CorruptedDocument,
    }
}

struct BuiltDocument {
    document: ContainerDocument,
    sections: Vec<String>,
}

/// 从偏好文件读出原始取值与容器用映射。
fn read_preferences_map(
    data_root: &Path,
) -> Result<
    (
        BTreeMap<String, container::PreferenceValue>,
        serde_json::Value,
    ),
    MigrationError,
> {
    // C 阶段：磁盘可能已是分域结构；容器 preferences 段按分域结构导出（含 schema_version），
    // 同时返回扁平视图供范围开关读取。
    let preferences = crate::modules::preferences::PreferencesStore::new(data_root)
        .load()
        .map_err(|_| MigrationError::Preferences)?;
    let flat: serde_json::Value =
        serde_json::to_value(&preferences).map_err(|_| MigrationError::Preferences)?;
    let payload = crate::modules::preferences::document_from_preferences(&preferences);
    let map = payload
        .as_object()
        .ok_or(MigrationError::Preferences)?
        .iter()
        .map(|(key, value)| {
            let value = match value {
                // schema_version 是整数；分域段是嵌套对象，按 JSON 文本承载，导入端再解析。
                serde_json::Value::Bool(value) => container::PreferenceValue::Boolean(*value),
                serde_json::Value::Number(value) => container::PreferenceValue::Integer(
                    value.as_i64().ok_or(MigrationError::Preferences)?,
                ),
                serde_json::Value::String(value) => container::PreferenceValue::Text(value.clone()),
                serde_json::Value::Object(_) => {
                    let text =
                        serde_json::to_string(value).map_err(|_| MigrationError::Preferences)?;
                    container::PreferenceValue::Text(text)
                }
                _ => return Err(MigrationError::Preferences),
            };
            Ok((key.clone(), value))
        })
        .collect::<Result<_, MigrationError>>()?;
    Ok((map, flat))
}

fn preference_flag(payload: &serde_json::Value, key: &str, default: bool) -> bool {
    payload
        .get(key)
        .and_then(|value| value.as_bool())
        .unwrap_or(default)
}

fn build_document(data_root: &Path) -> Result<BuiltDocument, MigrationError> {
    let (preference_map, preference_payload) = read_preferences_map(data_root)?;
    // FZ-19 范围开关：导出内容必须跟随用户设置，而不是固定全量。
    let include_skills = preference_flag(&preference_payload, "export_include_skills", true);
    let include_mcp = preference_flag(&preference_payload, "export_include_mcp", true);

    let extension_payload = match std::fs::read(data_root.join(EXTENSION_CONFIG_RELATIVE_PATH)) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(_) => return Err(MigrationError::Preferences),
    };
    let extension: crate::modules::extensions::ExtensionConfig = if extension_payload.is_empty() {
        crate::modules::extensions::ExtensionConfig::default()
    } else {
        let projection: ExtensionProjection =
            serde_json::from_slice(&extension_payload).map_err(|_| MigrationError::Preferences)?;
        projection.config
    };
    let sync_payload = match std::fs::read(data_root.join(SYNC_ENDPOINTS_RELATIVE_PATH)) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(_) => return Err(MigrationError::Preferences),
    };
    let sync: SyncConfig = if sync_payload.is_empty() {
        SyncConfig::default()
    } else {
        serde_json::from_slice(&sync_payload).map_err(|_| MigrationError::Preferences)?
    };
    if extension.skills.len() > container::ARRAY_MAX_ITEMS
        || extension.servers.len() > container::ARRAY_MAX_ITEMS
        || sync.endpoints.len() > container::ARRAY_MAX_ITEMS
    {
        return Err(MigrationError::LimitExceeded);
    }

    let skills = if include_skills {
        extension
            .skills
            .iter()
            .map(
                |name| -> Result<container::ContainerSkill, MigrationError> {
                    let source_path = container::ImportPath::parse(&format!(
                        "{}/{}/",
                        crate::modules::extensions::discovery::skills_store_relative_path(),
                        name
                    ))
                    .map_err(|_| MigrationError::CorruptedDocument)?;
                    let client_ids = extension
                        .enablement
                        .iter()
                        .filter(|(_, enabled)| **enabled)
                        .map(|(client, _)| *client)
                        .collect();
                    Ok(container::ContainerSkill {
                        name: name.clone(),
                        source_path,
                        client_ids,
                    })
                },
            )
            .collect::<Result<Vec<_>, MigrationError>>()?
    } else {
        Vec::new()
    };
    let server_names = if include_mcp {
        extension.servers.clone()
    } else {
        Vec::new()
    };

    let mut sections = vec!["preferences".to_string()];
    if include_skills {
        sections.push("extension_config.skills".to_string());
    }
    if include_mcp {
        sections.push("extension_config.mcp".to_string());
    }
    if !extension.enablement.is_empty() {
        sections.push("extension_config.enablement".to_string());
    }
    if !sync.endpoints.is_empty() {
        sections.push("sync_endpoints".to_string());
    }
    sections.sort();

    Ok(BuiltDocument {
        document: ContainerDocument {
            container: container::ContainerInfo {
                structure_version: container::STRUCTURE_VERSION,
            },
            extension_config: Some(container::ExtensionSection {
                skills,
                servers: Vec::new(),
                server_names,
                source_store: extension.source_store.clone(),
                sync_method: Some(extension.sync_method_normalized().to_string()),
                enablement: extension.enablement.clone(),
            }),
            sync_endpoints: Some(
                sync.endpoints
                    .iter()
                    .map(
                        |endpoint| -> Result<container::SyncEndpoint, MigrationError> {
                            Ok(container::SyncEndpoint {
                                name: endpoint.endpoint_id.clone(),
                                credential: container::SecretReference::parse(&format!(
                                    "keychain://{}",
                                    endpoint.credential_ref.ref_id
                                ))
                                .map_err(|_| MigrationError::CorruptedDocument)?,
                                url: Some(endpoint.url.clone()),
                                remote_path: Some(endpoint.remote_path.clone()),
                                username: Some(endpoint.username.clone()),
                                tls_policy: Some(endpoint.tls_policy.clone()),
                                conflict_policy: Some(endpoint.conflict_policy.clone()),
                            })
                        },
                    )
                    .collect::<Result<Vec<_>, MigrationError>>()?,
            ),
            preferences: Some(preference_map),
        },
        sections,
    })
}

/// 计算明文载荷的完整性摘要（只用于检测损坏）。
fn document_integrity(document: &ContainerDocument) -> Result<String, MigrationError> {
    let bytes = serde_json::to_vec(document).map_err(|_| MigrationError::CorruptedDocument)?;
    Ok(sha256_hex(&bytes))
}

fn build_container_file(
    document: &ContainerDocument,
    app_version: &str,
) -> Result<Vec<u8>, MigrationError> {
    let file = ContainerFilePlaintext {
        container_magic: CONTAINER_FILE_MAGIC.to_string(),
        container_format_version: CONTAINER_FILE_FORMAT_VERSION,
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        app_version: app_version.to_string(),
        structure_version: container::STRUCTURE_VERSION,
        integrity: IntegrityHeader {
            algorithm: container::INTEGRITY_ALGORITHM.to_string(),
            digest: document_integrity(document)?,
        },
        document: document.clone(),
    };
    let bytes = serde_json::to_vec_pretty(&file).map_err(|_| MigrationError::ContainerFormat)?;
    if bytes.len() as u64 > container::PLAINTEXT_MAX_BYTES {
        return Err(MigrationError::LimitExceeded);
    }
    Ok(bytes)
}

fn read_file_limited(path: &Path, limit: u64) -> Result<Vec<u8>, MigrationError> {
    let file = std::fs::File::open(path).map_err(|_| MigrationError::ContainerFormat)?;
    let size = file
        .metadata()
        .map_err(|_| MigrationError::ContainerFormat)?
        .len();
    if size > limit {
        return Err(MigrationError::LimitExceeded);
    }
    let mut reader = file.take(limit);
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|_| MigrationError::ContainerFormat)?;
    Ok(bytes)
}

/// 解析容器文件：先看格式版本，再分派到旧加密载荷或当前明文载荷。
fn parse_container(bytes: &[u8]) -> Result<ParsedContainer, MigrationError> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| MigrationError::ContainerFormat)?;
    let magic = value
        .get("container_magic")
        .and_then(|item| item.as_str())
        .unwrap_or_default();
    if magic != CONTAINER_FILE_MAGIC {
        return Err(MigrationError::ContainerFormat);
    }
    let version = value
        .get("container_format_version")
        .and_then(|item| item.as_u64())
        .ok_or(MigrationError::ContainerFormat)?;
    match version as u32 {
        CONTAINER_FILE_FORMAT_VERSION_LEGACY => {
            let file: ContainerFile =
                serde_json::from_value(value).map_err(|_| MigrationError::ContainerFormat)?;
            container::validate_header(&file.header).map_err(map_container_error)?;
            Ok(ParsedContainer::Legacy {
                header: file.header,
                payload_b64: file.payload_b64,
            })
        }
        CONTAINER_FILE_FORMAT_VERSION => {
            let file: ContainerFilePlaintext =
                serde_json::from_value(value).map_err(|_| MigrationError::ContainerFormat)?;
            if file.structure_version != container::STRUCTURE_VERSION
                || file.integrity.algorithm != container::INTEGRITY_ALGORITHM
                || file.app_version.trim().is_empty()
                || DateTime::parse_from_rfc3339(&file.created_at).is_err()
            {
                return Err(MigrationError::ContainerFormat);
            }
            if file.integrity.digest != document_integrity(&file.document)? {
                return Err(MigrationError::CorruptedDocument);
            }
            container::validate_document(&file.document).map_err(map_container_error)?;
            Ok(ParsedContainer::Plaintext(file.document))
        }
        _ => Err(MigrationError::UnsupportedVersion),
    }
}

fn open_legacy_document(
    header: &ContainerHeader,
    payload_b64: &str,
    passphrase: Option<&str>,
) -> Result<ContainerDocument, MigrationError> {
    let passphrase = passphrase
        .filter(|value| !value.is_empty())
        .ok_or(MigrationError::PassphraseRequired)?;
    let salt = BASE64_STANDARD
        .decode(header.kdf.salt_b64.as_bytes())
        .map_err(|_| MigrationError::ContainerFormat)?;
    if salt.len() != KDF_SALT_BYTES {
        return Err(MigrationError::ContainerFormat);
    }
    let key = derive_key_for_test(passphrase.as_bytes(), &salt)?;
    open_document_for_test(payload_b64, header, &key)
}

/// Captured export ownership, separate from the planned generic migration job.
/// Begin runs after the target lock is acquired; completed runs before it drops.
pub trait ExportObserver {
    fn begin(&mut self, document_sha256: &str, target: &Path);
    fn completed(&mut self, succeeded: bool);
}
struct NoExportObserver;
impl ExportObserver for NoExportObserver {
    fn begin(&mut self, _: &str, _: &Path) {}
    fn completed(&mut self, _: bool) {}
}

fn verify_export_file(target: &Path, expected: &[u8]) -> Result<(), AppError> {
    let actual = read_file_limited(target, container::CIPHERTEXT_MAX_BYTES)
        .map_err(|_| MigrationError::Preferences)?;
    if actual != expected {
        return Err(MigrationError::CorruptedDocument.into());
    }
    Ok(())
}

fn write_export(
    data_root: &Path,
    target: &Path,
    app_version: &str,
    observer: &mut dyn ExportObserver,
) -> Result<ExportResult, AppError> {
    write_export_verified(data_root, target, app_version, observer, verify_export_file)
}

fn write_export_verified(
    data_root: &Path,
    target: &Path,
    app_version: &str,
    observer: &mut dyn ExportObserver,
    verify: impl FnOnce(&Path, &[u8]) -> Result<(), AppError>,
) -> Result<ExportResult, AppError> {
    let built = build_document(data_root)?;
    let container_bytes = build_container_file(&built.document, app_version)?;
    let document_bytes =
        serde_json::to_vec(&built.document).map_err(|_| MigrationError::Preferences)?;
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|error| AppError::FileSystem {
            operation: "create exports directory".to_string(),
            detail: error.to_string(),
        })?;
    }
    // Serialize capture of the previous export with replacement and readback.
    // Admission/lock rejection is not an executed export terminal.
    let _lock = TargetFileLock::lock(target).map_err(|_| MigrationError::AtomicWrite)?;
    let document_sha256 = sha256_hex(&document_bytes);
    observer.begin(&document_sha256, target);
    let result = (|| {
        let backup_id = if target.exists() {
            let previous = read_file_limited(target, container::CIPHERTEXT_MAX_BYTES)
                .map_err(|_| MigrationError::Preferences)?;
            let record = backup::backup_file(
                data_root,
                backup::BackupAction::Upgrade,
                target,
                &previous,
                chrono::Utc::now(),
                Some("pre-export".to_string()),
            )?;
            Some(record.manifest.backup_id)
        } else {
            None
        };
        crate::infrastructure::atomic_write::atomic_write(target, &container_bytes, 0o600)?;
        verify(target, &container_bytes)?;
        Ok(ExportResult {
            path: target.to_path_buf(),
            backup_id,
            document_sha256,
            format_version: CONTAINER_FILE_FORMAT_VERSION,
            sections: built.sections,
            excluded: container::EXPORT_EXCLUSIONS
                .iter()
                .map(|value| value.to_string())
                .collect(),
        })
    })();
    // A failed readback may leave a changed file. Never claim atomic rollback.
    observer.completed(result.is_ok());
    result
}

pub fn export_with_paths(data_root: &Path, app_version: &str) -> Result<ExportResult, AppError> {
    validate_data_root(data_root)?;
    validate_structure(data_root)?;
    if !data_root.join("manager-state").is_dir() {
        return Err(MigrationError::Preferences.into());
    }
    let target = export_path(data_root);
    write_export(data_root, &target, app_version, &mut NoExportObserver)
}

pub fn export_with_container_file(
    data_root: &Path,
    output_path: &Path,
    app_version: &str,
) -> Result<ExportResult, AppError> {
    export_with_container_file_observed(data_root, output_path, app_version, &mut NoExportObserver)
}

pub fn export_with_container_file_observed(
    data_root: &Path,
    output_path: &Path,
    app_version: &str,
    observer: &mut dyn ExportObserver,
) -> Result<ExportResult, AppError> {
    validate_data_root(data_root)?;
    validate_structure(data_root)?;
    if !data_root.join("manager-state").is_dir() {
        return Err(MigrationError::Preferences.into());
    }
    write_export(data_root, output_path, app_version, observer)
}

/// 准备写入的一个文件（用于受控提交与回滚）。
struct AppliedSection {
    changed_paths: Vec<std::path::PathBuf>,
    document: Vec<u8>,
}

fn import_preferences(
    data_root: &Path,
    document: &ContainerDocument,
) -> Result<AppliedSection, MigrationError> {
    let preference_values = document
        .preferences
        .clone()
        .ok_or(MigrationError::CorruptedDocument)?;
    let preferences =
        Preferences::try_from(preference_values).map_err(|_| MigrationError::CorruptedDocument)?;
    crate::modules::preferences::validate(&preferences)
        .map_err(|_| MigrationError::CorruptedDocument)?;
    // C 阶段：导入落盘为分域结构（与偏好存储一致），而不是写回 legacy 扁平文件。
    let document = crate::modules::preferences::document_from_preferences(&preferences);
    Ok(AppliedSection {
        changed_paths: vec![data_root.join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH)],
        document: serde_json::to_vec_pretty(&document).map_err(|_| MigrationError::Preferences)?,
    })
}

/// 只做校验与目标配置构造，不写盘（写盘在受控提交阶段）。
fn prepare_extension_config(
    data_root: &Path,
    document: &ContainerDocument,
) -> Result<Option<crate::modules::extensions::ExtensionConfig>, MigrationError> {
    let Some(extension) = document.extension_config.as_ref() else {
        return Ok(None);
    };
    let mut current = crate::modules::extensions::projection::load_config_lenient(data_root);
    let mut changed = false;
    if !extension.enablement.is_empty() && extension.enablement != current.enablement {
        current.enablement = extension.enablement.clone();
        changed = true;
    }
    if let Some(method) = extension.sync_method.as_deref() {
        if current.sync_method_normalized() != method {
            current.sync_method = method.to_string();
            changed = true;
        }
    }
    if !changed {
        return Ok(None);
    }
    Ok(Some(current))
}

fn prepare_sync_endpoints(
    data_root: &Path,
    document: &ContainerDocument,
) -> Result<Option<AppliedSection>, MigrationError> {
    let Some(endpoints) = document.sync_endpoints.as_ref() else {
        return Ok(None);
    };
    let Some(endpoint) = endpoints.first() else {
        return Ok(None);
    };
    let Some(url) = endpoint.url.clone() else {
        // 旧版容器没有可用端点配置，不应用。
        return Ok(None);
    };
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let existing = crate::modules::sync::config::SyncConfigStore::new(data_root)
        .load()
        .map_err(|_| MigrationError::Preferences)?;
    // 同机导入沿用原引用（口令仍在钥匙串）；异机导入新建本机引用，需重新认证。
    let (credential, username) = match existing.active() {
        Some(active) if active.credential_ref.ref_id == endpoint.credential.ref_id => {
            (active.credential_ref.clone(), active.username.clone())
        }
        _ => {
            let ref_id = crate::infrastructure::keychain::new_ref_id();
            let account_key = crate::infrastructure::keychain::account_key(
                crate::infrastructure::keychain::WEBDAV_CREDENTIAL_PURPOSE,
                &ref_id,
            )
            .map_err(|_| MigrationError::Preferences)?;
            (
                crate::modules::sync::config::CredentialRef {
                    ref_id,
                    backend: "keychain".to_string(),
                    service_name: crate::infrastructure::keychain::keychain_service_name()
                        .to_string(),
                    account_key,
                    purpose: crate::infrastructure::keychain::WEBDAV_CREDENTIAL_PURPOSE.to_string(),
                    created_at: now.clone(),
                    updated_at: now.clone(),
                },
                endpoint.username.clone().unwrap_or_default(),
            )
        }
    };
    let config = crate::modules::sync::config::SyncConfig {
        schema_version: 1,
        endpoints: vec![crate::modules::sync::config::SyncEndpointConfig {
            endpoint_id: endpoint.name.clone(),
            url: url.trim_end_matches('/').to_string(),
            remote_path: endpoint
                .remote_path
                .clone()
                .unwrap_or_default()
                .trim_matches('/')
                .to_string(),
            username,
            credential_ref: credential,
            tls_policy: crate::modules::sync::config::TLS_POLICY_VERIFY_REQUIRED.to_string(),
            legacy_encryption: String::new(),
            conflict_policy: endpoint
                .conflict_policy
                .clone()
                .unwrap_or_else(|| crate::modules::sync::config::CONFLICT_POLICY_ASK.to_string()),
        }],
    };
    crate::modules::sync::config::validate_config(&config)
        .map_err(|_| MigrationError::Preferences)?;
    Ok(Some(AppliedSection {
        changed_paths: vec![data_root.join(SYNC_ENDPOINTS_RELATIVE_PATH)],
        document: serde_json::to_vec_pretty(&config).map_err(|_| MigrationError::Preferences)?,
    }))
}

fn import_with_parsed(
    data_root: &Path,
    home: &Path,
    parsed: ParsedContainer,
    passphrase: Option<&str>,
) -> Result<ImportResult, AppError> {
    let (document, format_version) = match parsed {
        ParsedContainer::Legacy {
            header,
            payload_b64,
        } => (
            open_legacy_document(&header, &payload_b64, passphrase)?,
            CONTAINER_FILE_FORMAT_VERSION_LEGACY,
        ),
        ParsedContainer::Plaintext(document) => (document, CONTAINER_FILE_FORMAT_VERSION),
    };
    container::validate_document(&document).map_err(map_container_error)?;

    // 1) 全量验证与准备：任何一段不合法都在写盘之前返回。
    let preferences_section = import_preferences(data_root, &document)?;
    let extension_config = prepare_extension_config(data_root, &document)?;
    let sync_section = prepare_sync_endpoints(data_root, &document)?;

    let mut applied = vec!["preferences".to_string()];
    let mut skipped: Vec<ImportSectionSkip> = Vec::new();
    if extension_config.is_some() {
        applied.push("extension_config".to_string());
    } else if document.extension_config.is_some() {
        skipped.push(ImportSectionSkip {
            section: "extension_config".to_string(),
            reason: "容器中的扩展配置与当前一致，无需写入。".to_string(),
        });
    }
    if sync_section.is_some() {
        applied.push("sync_endpoints".to_string());
    } else if document
        .sync_endpoints
        .as_ref()
        .is_some_and(|items| !items.is_empty())
    {
        skipped.push(ImportSectionSkip {
            section: "sync_endpoints".to_string(),
            reason: "旧版容器或缺少可用端点地址，未应用；请在设置里重新填写并认证。".to_string(),
        });
    }
    skipped.push(ImportSectionSkip {
        section: "asset_files".to_string(),
        reason:
            "Skills 文件内容与 MCP 环境变量值不在容器内；导入后请在扩展管理中重新落盘或重新输入。"
                .to_string(),
    });

    // 2) 受控提交：先备份将被覆盖的文件，任一写入失败即回滚。
    let preferences_path = data_root.join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH);
    let mut targets: Vec<(std::path::PathBuf, Vec<u8>, bool)> = vec![(
        preferences_path.clone(),
        std::fs::read(&preferences_path).unwrap_or_default(),
        preferences_path.exists(),
    )];
    if let Some(section) = sync_section.as_ref() {
        let path = &section.changed_paths[0];
        targets.push((
            path.clone(),
            std::fs::read(path).unwrap_or_default(),
            path.exists(),
        ));
    }
    let mut backup_id = "none".to_string();
    for (index, (path, payload, existed)) in targets.iter().enumerate() {
        if !existed {
            continue;
        }
        let record = backup::backup_file(
            data_root,
            backup::BackupAction::Import,
            path,
            payload,
            chrono::Utc::now(),
            Some("pre-import".to_string()),
        )?;
        if index == 0 {
            backup_id = record.manifest.backup_id.clone();
        }
    }

    let restore = |targets: &[(std::path::PathBuf, Vec<u8>, bool)], count: usize| {
        for (path, payload, existed) in targets.iter().take(count) {
            if *existed {
                let _ = crate::infrastructure::atomic_write::atomic_write(path, payload, 0o600);
            } else {
                let _ = std::fs::remove_file(path);
            }
        }
    };

    let mut committed = 0_usize;
    let write = |path: &Path, bytes: &[u8]| -> Result<(), MigrationError> {
        let lock = TargetFileLock::lock(path).map_err(|_| MigrationError::AtomicWrite)?;
        let result = crate::infrastructure::atomic_write::atomic_write(path, bytes, 0o600)
            .map_err(|_| MigrationError::AtomicWrite);
        drop(lock);
        result
    };

    if let Err(error) = write(&preferences_path, &preferences_section.document) {
        restore(&targets, committed);
        return Err(error.into());
    }
    committed += 1;
    if let Some(section) = sync_section.as_ref() {
        let path = &section.changed_paths[0];
        if let Err(error) = write(path, &section.document) {
            restore(&targets, committed);
            return Err(error.into());
        }
        committed += 1;
    }
    if let Some(mut config) = extension_config {
        // 扩展统一配置走既有校验/备份/原子写路径（该路径自带备份）。
        let client_targets: Vec<ClientTarget> = CLIENT_IDS
            .map(|client| ClientTarget::user_target(client, home, true, true))
            .to_vec();
        if crate::modules::extensions::projection::save_with_config(
            data_root,
            &mut config,
            &client_targets,
        )
        .is_err()
        {
            restore(&targets, committed);
            return Err(MigrationError::Preferences.into());
        }
    }

    let document_bytes =
        serde_json::to_vec(&document).map_err(|_| MigrationError::CorruptedDocument)?;
    Ok(ImportResult {
        backup_id,
        document_sha256: sha256_hex(&document_bytes),
        format_version,
        applied_sections: applied,
        skipped_sections: skipped,
        excluded: container::EXPORT_EXCLUSIONS
            .iter()
            .map(|value| value.to_string())
            .collect(),
    })
}

pub fn import_with_paths(
    data_root: &Path,
    home: &Path,
    passphrase: Option<&str>,
) -> Result<ImportResult, AppError> {
    validate_data_root(data_root)?;
    validate_structure(data_root)?;
    let target = export_path(data_root);
    import_with_container_file(data_root, &target, home, passphrase)
}

pub fn import_with_container_file(
    data_root: &Path,
    container_path: &Path,
    home: &Path,
    passphrase: Option<&str>,
) -> Result<ImportResult, AppError> {
    validate_data_root(data_root)?;
    validate_structure(data_root)?;
    let container_bytes = read_file_limited(container_path, container::CIPHERTEXT_MAX_BYTES)
        .map_err(|_| MigrationError::Preferences)?;
    let parsed = parse_container(&container_bytes)?;
    import_with_parsed(data_root, home, parsed, passphrase)
}

#[cfg(all(test, unix))]
mod tests;
