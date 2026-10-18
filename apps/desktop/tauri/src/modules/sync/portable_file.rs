//! Versioned portable file envelope shared by file/sync objects. A checksum
//! detects damage; password protection authenticates the complete manifest.
use crate::modules::{
    container, migration,
    model_config::{schema::*, store::sha256},
};
use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;
pub const DOCUMENT_LIMIT: usize = 32 * 1024 * 1024;
pub const FILE_LIMIT: usize = 64 * 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Manifest {
    pub id: String,
    pub created_at: String,
    pub sections: Vec<String>,
    pub secret_objects: usize,
    pub document_digest: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Envelope {
    format: String,
    format_version: u32,
    manifest: Manifest,
    payload: Content,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Content {
    Plain {
        document_b64: String,
    },
    Protected {
        kdf: String,
        memory_kib: u32,
        iterations: u32,
        parallelism: u32,
        aead: String,
        salt_b64: String,
        nonce_b64: String,
        ciphertext_b64: String,
    },
}
pub struct Opened {
    pub manifest: Manifest,
    pub protected: bool,
    pub document: Zeroizing<Vec<u8>>,
}
fn aad(manifest: &Manifest) -> ModelResult<Vec<u8>> {
    Ok(serde_json::to_vec(
        &serde_json::json!({"format":"OCXDCONF","formatVersion":3,"manifest":manifest}),
    )?)
}
fn validate(manifest: &Manifest) -> ModelResult<()> {
    require_uuid(&manifest.id)?;
    if chrono::DateTime::parse_from_rfc3339(&manifest.created_at).is_err()
        || manifest.sections.is_empty()
        || manifest.sections.len() > 32
        || manifest
            .sections
            .iter()
            .any(|s| s.is_empty() || s.len() > 128)
        || manifest
            .sections
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != manifest.sections.len()
        || manifest.secret_objects > 20000
    {
        return Err(invalid(
            "portable_manifest",
            "配置清单的身份、范围或数量无效",
        ));
    }
    let digest = manifest
        .document_digest
        .strip_prefix("sha256:")
        .ok_or_else(|| invalid("portable_digest", "配置清单摘要格式无效"))?;
    if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(invalid("portable_digest", "配置清单摘要格式无效"));
    }
    Ok(())
}
pub fn seal(
    mut manifest: Manifest,
    document: &[u8],
    passphrase: Option<&str>,
) -> ModelResult<Vec<u8>> {
    if document.len() > DOCUMENT_LIMIT {
        return Err(invalid("portable_limit", "配置文档超过32MiB"));
    }
    manifest.document_digest = sha256(document);
    validate(&manifest)?;
    let payload = match passphrase {
        None if manifest.secret_objects > 0 => {
            return Err(invalid(
                "passphrase_required",
                "携带凭据时请设置独立的文件保护口令",
            ))
        }
        None => Content::Plain {
            document_b64: STANDARD.encode(document),
        },
        Some(password) => {
            if password.is_empty() || password.len() > 8192 {
                return Err(invalid(
                    "portable_passphrase",
                    "文件保护口令不能为空或超过上限",
                ));
            }
            let mut salt = [0u8; container::KDF_SALT_BYTES];
            let mut nonce = [0u8; container::AEAD_NONCE_BYTES];
            rand::rng().fill_bytes(&mut salt);
            rand::rng().fill_bytes(&mut nonce);
            let key = migration::derive_portable_key(password.as_bytes(), &salt)
                .map_err(|_| invalid("portable_kdf", "文件保护密钥派生失败"))?;
            let cipher = Aes256Gcm::new_from_slice(key.as_ref())
                .map_err(|_| invalid("portable_cipher", "文件保护未就绪"))?;
            let encrypted = cipher
                .encrypt(
                    Nonce::from_slice(&nonce),
                    Payload {
                        msg: document,
                        aad: &aad(&manifest)?,
                    },
                )
                .map_err(|_| invalid("portable_encrypt", "配置文件保护失败"))?;
            Content::Protected {
                kdf: container::KDF_ALGORITHM.into(),
                memory_kib: container::KDF_MEMORY_KIB,
                iterations: container::KDF_ITERATIONS,
                parallelism: container::KDF_PARALLELISM,
                aead: container::AEAD_ALGORITHM.into(),
                salt_b64: STANDARD.encode(salt),
                nonce_b64: STANDARD.encode(nonce),
                ciphertext_b64: STANDARD.encode(encrypted),
            }
        }
    };
    let bytes = serde_json::to_vec(&Envelope {
        format: "OCXDCONF".into(),
        format_version: 3,
        manifest,
        payload,
    })?;
    if bytes.len() > FILE_LIMIT {
        return Err(invalid("portable_limit", "配置文件超过64MiB"));
    }
    Ok(bytes)
}
pub fn open(bytes: &[u8], passphrase: Option<&str>) -> ModelResult<Opened> {
    if bytes.len() > FILE_LIMIT {
        return Err(invalid("portable_limit", "配置文件超过64MiB"));
    }
    let envelope: Envelope = serde_json::from_slice(bytes)
        .map_err(|_| invalid("portable_format", "配置文件结构无效或含未知字段"))?;
    if envelope.format != "OCXDCONF" || envelope.format_version != 3 {
        return Err(invalid("portable_version", "不支持此配置文件版本"));
    }
    validate(&envelope.manifest)?;
    let (protected, document) = match envelope.payload {
        Content::Plain { document_b64 } => {
            if envelope.manifest.secret_objects > 0 {
                return Err(invalid(
                    "portable_protection",
                    "携带凭据的配置文件必须受保护",
                ));
            }
            (
                false,
                STANDARD
                    .decode(document_b64)
                    .map_err(|_| invalid("portable_payload", "配置载荷编码无效"))?,
            )
        }
        Content::Protected {
            kdf,
            memory_kib,
            iterations,
            parallelism,
            aead,
            salt_b64,
            nonce_b64,
            ciphertext_b64,
        } => {
            if kdf != container::KDF_ALGORITHM
                || memory_kib != container::KDF_MEMORY_KIB
                || iterations != container::KDF_ITERATIONS
                || parallelism != container::KDF_PARALLELISM
                || aead != container::AEAD_ALGORITHM
            {
                return Err(invalid(
                    "portable_algorithm",
                    "文件保护参数不受支持，未开始派生",
                ));
            }
            let salt = STANDARD
                .decode(salt_b64)
                .map_err(|_| invalid("portable_payload", "文件保护盐编码无效"))?;
            let nonce = STANDARD
                .decode(nonce_b64)
                .map_err(|_| invalid("portable_payload", "文件保护随机数编码无效"))?;
            let ciphertext = STANDARD
                .decode(ciphertext_b64)
                .map_err(|_| invalid("portable_payload", "文件密文编码无效"))?;
            if salt.len() != container::KDF_SALT_BYTES
                || nonce.len() != container::AEAD_NONCE_BYTES
                || ciphertext.len() < 16
                || ciphertext.len() > DOCUMENT_LIMIT + 16
            {
                return Err(invalid("portable_limit", "文件保护载荷大小无效"));
            }
            let password = passphrase
                .filter(|p| !p.is_empty() && p.len() <= 8192)
                .ok_or_else(|| invalid("passphrase_required", "请输入此配置文件的保护口令"))?;
            let key = migration::derive_portable_key(password.as_bytes(), &salt)
                .map_err(|_| invalid("portable_kdf", "文件保护密钥派生失败"))?;
            let cipher = Aes256Gcm::new_from_slice(key.as_ref())
                .map_err(|_| invalid("portable_cipher", "文件保护未就绪"))?;
            let decrypted = cipher
                .decrypt(
                    Nonce::from_slice(&nonce),
                    Payload {
                        msg: &ciphertext,
                        aad: &aad(&envelope.manifest)?,
                    },
                )
                .map_err(|_| {
                    invalid(
                        "portable_authentication",
                        "口令不正确或受保护配置文件已损坏",
                    )
                })?;
            (true, decrypted)
        }
    };
    let document = Zeroizing::new(document);
    if document.len() > DOCUMENT_LIMIT || sha256(&document) != envelope.manifest.document_digest {
        return Err(invalid("portable_digest", "配置载荷与清单摘要不一致"));
    }
    Ok(Opened {
        manifest: envelope.manifest,
        protected,
        document,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn manifest(secret_objects: usize) -> Manifest {
        Manifest {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            sections: vec!["models".into()],
            secret_objects,
            document_digest: String::new(),
        }
    }
    #[test]
    fn plaintext_checks_damage_and_never_accepts_a_declared_secret_payload() {
        let bytes = seal(manifest(0), b"fixture-config", None).unwrap();
        assert_eq!(
            open(&bytes, None).unwrap().document.as_slice(),
            b"fixture-config"
        );
        assert_eq!(
            seal(manifest(1), b"fixture-key", None).unwrap_err().code,
            "passphrase_required"
        );
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        value["manifest"]["secretObjects"] = serde_json::json!(1);
        assert!(open(&serde_json::to_vec(&value).unwrap(), None).is_err());
    }
    #[test]
    fn password_protects_payload_and_authenticates_scope_while_rejecting_algorithm_changes() {
        let bytes = seal(
            manifest(1),
            b"fixture-only-key",
            Some("fixture-file-passphrase"),
        )
        .unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains("fixture-only-key"));
        let opened = open(&bytes, Some("fixture-file-passphrase")).unwrap();
        assert!(opened.protected);
        assert_eq!(opened.document.as_slice(), b"fixture-only-key");
        assert!(matches!(open(&bytes,None),Err(error) if error.code=="passphrase_required"));
        assert!(
            matches!(open(&bytes,Some("wrong-passphrase")),Err(error) if error.code=="portable_authentication")
        );
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        value["manifest"]["sections"] = serde_json::json!(["accounts"]);
        assert!(
            matches!(open(&serde_json::to_vec(&value).unwrap(),Some("fixture-file-passphrase")),Err(error) if error.code=="portable_authentication")
        );
        value["payload"]["memory_kib"] = serde_json::json!(1);
        assert!(
            matches!(open(&serde_json::to_vec(&value).unwrap(),Some("fixture-file-passphrase")),Err(error) if error.code=="portable_algorithm")
        );
    }
}
