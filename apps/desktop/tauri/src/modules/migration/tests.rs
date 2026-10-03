//! 配置迁移回归：明文容器免口令、旧加密容器兼容读、失败不改变现有配置。

use super::*;
use crate::modules::extensions::{ClientId, CLIENT_IDS};
use crate::modules::preferences::{Preferences, PreferencesStore};
use crate::modules::sync::config::{CredentialRef, SyncConfig, SyncEndpointConfig};
use std::os::unix::fs::PermissionsExt;

const LEGACY_PASSPHRASE: &str = "correct horse battery staple";

fn data_root() -> tempfile::TempDir {
    let temp = tempfile::tempdir().expect("create temporary data root");
    crate::modules::data_root::initialize(temp.path()).expect("initialize fixture");
    temp
}

fn home_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("create temporary home")
}

fn write_extension_projection(root: &Path, skills: &[&str], servers: &[&str], codex_only: bool) {
    let enablement: std::collections::BTreeMap<_, _> = CLIENT_IDS
        .map(|client| {
            (
                client,
                if codex_only {
                    client == ClientId::Codex
                } else {
                    true
                },
            )
        })
        .into_iter()
        .collect();
    let payload = serde_json::json!({
        "config": {
            "skills": skills,
            "servers": servers,
            "enablement": enablement,
            "revision": 7
        },
        "fingerprint": {
            "schema_version": 1,
            "fingerprint": "3b12453c883300e4933381a4fd5f10710a68ac68a45cafff581ed8df3a35975b",
            "updated_at": "2026-09-17T00:00:00Z",
            "source": "opencodex-desktop"
        }
    });
    std::fs::write(
        root.join("manager-state/extension-config.json"),
        serde_json::to_vec_pretty(&payload).unwrap(),
    )
    .expect("write projection");
}

fn write_sync_endpoint(root: &Path) {
    let now = "2026-09-17T00:00:00Z";
    let ref_id = "cred_12345678-1234-1234-1234-123456789abc";
    let sync_config = SyncConfig {
        schema_version: 1,
        endpoints: vec![SyncEndpointConfig {
            endpoint_id: "default".to_string(),
            url: "https://dav.example.test".to_string(),
            remote_path: "/desktop-sync/current".to_string(),
            username: "user".to_string(),
            credential_ref: CredentialRef {
                ref_id: ref_id.to_string(),
                backend: "keychain".to_string(),
                service_name: crate::infrastructure::keychain::KEYCHAIN_SERVICE_NAME.to_string(),
                account_key: crate::infrastructure::keychain::account_key(
                    crate::infrastructure::keychain::WEBDAV_CREDENTIAL_PURPOSE,
                    ref_id,
                )
                .unwrap(),
                purpose: crate::infrastructure::keychain::WEBDAV_CREDENTIAL_PURPOSE.to_string(),
                created_at: now.to_string(),
                updated_at: now.to_string(),
            },
            tls_policy: "verify_required".to_string(),
            legacy_encryption: "forced".to_string(),
            conflict_policy: "ask".to_string(),
        }],
    };
    std::fs::write(
        root.join("manager-state/sync-endpoints.json"),
        serde_json::to_vec(&sync_config).unwrap(),
    )
    .expect("write sync");
}

/// 读取明文容器里的文档（只用于断言导出内容）。
fn read_plaintext_document(bytes: &[u8]) -> ContainerDocument {
    match parse_container(bytes).expect("parse plaintext container") {
        ParsedContainer::Plaintext(document) => document,
        ParsedContainer::Legacy { .. } => panic!("expected plaintext container"),
    }
}

/// 只用于测试夹具：生成旧版（v1）加密容器字节。
fn legacy_container_bytes(
    document: &ContainerDocument,
    passphrase: &str,
    app_version: &str,
) -> Vec<u8> {
    use aes_gcm::aead::{Aead, KeyInit, Payload};
    use aes_gcm::{Aes256Gcm, Key, Nonce};
    use base64::engine::general_purpose::STANDARD as B64;
    use base64::Engine;
    use rand::Rng;

    let mut salt = [0_u8; KDF_SALT_BYTES];
    rand::rng().fill(&mut salt);
    let header = container::new_header(chrono::Utc::now(), app_version, B64.encode(salt));
    container::validate_header(&header).expect("legacy header");
    let key = derive_key_for_test(passphrase.as_bytes(), &salt).expect("derive key");
    let plaintext = serde_json::to_vec(document).expect("serialize document");
    let mut nonce = [0_u8; AEAD_NONCE_BYTES];
    rand::rng().fill(&mut nonce);
    let ciphertext = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key.as_slice()))
        .encrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: plaintext.as_ref(),
                aad: serde_json::to_vec(&header).expect("aad").as_ref(),
            },
        )
        .expect("seal legacy document");
    let mut payload = Vec::with_capacity(nonce.len() + ciphertext.len());
    payload.extend_from_slice(&nonce);
    payload.extend_from_slice(&ciphertext);
    let file = serde_json::json!({
        "container_magic": CONTAINER_FILE_MAGIC,
        "container_format_version": CONTAINER_FILE_FORMAT_VERSION_LEGACY,
        "header": header,
        "payload_b64": B64.encode(payload),
    });
    serde_json::to_vec_pretty(&file).expect("legacy container json")
}

#[test]
fn export_is_plaintext_and_needs_no_passphrase() {
    let root = data_root();
    PreferencesStore::new(root.path())
        .save(&Preferences::default())
        .expect("save defaults");
    write_extension_projection(root.path(), &["design-studio"], &["node_repl"], false);
    write_sync_endpoint(root.path());

    let exported = export_with_paths(root.path(), "0.1.0").expect("export");
    assert_eq!(exported.format_version, 2);
    let raw = std::fs::read_to_string(&exported.path).expect("read container");
    // 明文容器不得包含口令字段，也不得包含钥匙串密文。
    assert!(!raw.contains("passphrase"));
    assert!(!raw.contains("password"));
    assert!(!raw.contains("ciphertext"));
    assert!(!raw.contains("kdf"));
    assert_eq!(
        std::fs::metadata(&exported.path)
            .expect("metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert!(exported.sections.iter().any(|item| item == "preferences"));
    assert!(exported
        .sections
        .iter()
        .any(|item| item == "sync_endpoints"));
    assert!(!exported.excluded.is_empty());

    let document = read_plaintext_document(&std::fs::read(&exported.path).unwrap());
    let endpoints = document.sync_endpoints.expect("sync endpoints");
    assert_eq!(endpoints.len(), 1);
    assert_eq!(
        endpoints[0].url.as_deref(),
        Some("https://dav.example.test")
    );
    assert_eq!(
        endpoints[0].remote_path.as_deref(),
        Some("/desktop-sync/current")
    );
    assert_eq!(
        endpoints[0].credential.as_ref_id(),
        "cred_12345678-1234-1234-1234-123456789abc"
    );
    let extension = document.extension_config.expect("extension section");
    assert_eq!(extension.skills.len(), 1);
    assert_eq!(extension.skills[0].name, "design-studio");
    assert_eq!(extension.server_names, vec!["node_repl".to_string()]);
}

#[test]
fn export_honours_include_toggles() {
    let root = data_root();
    PreferencesStore::new(root.path())
        .save(&Preferences {
            export_include_skills: false,
            export_include_mcp: false,
            ..Default::default()
        })
        .expect("save preferences");
    write_extension_projection(root.path(), &["design-studio"], &["node_repl"], false);

    let exported = export_with_paths(root.path(), "0.1.0").expect("export");
    assert!(!exported.sections.iter().any(|item| item.contains("skills")));
    assert!(!exported.sections.iter().any(|item| item.contains("mcp")));
    let document = read_plaintext_document(&std::fs::read(&exported.path).unwrap());
    let extension = document.extension_config.expect("extension section");
    assert!(extension.skills.is_empty());
    assert!(extension.server_names.is_empty());
    // 总开关与分发方式与机器无关，仍会导出。
    assert!(!extension.enablement.is_empty());
}

#[test]
fn import_without_passphrase_applies_preferences_extension_and_sync() {
    let source = data_root();
    let home = home_dir();
    PreferencesStore::new(source.path())
        .save(&Preferences {
            interface_scale: 150,
            panel_mode: "browser".to_string(),
            ..Default::default()
        })
        .expect("save source preferences");
    write_extension_projection(source.path(), &["design-studio"], &["node_repl"], true);
    write_sync_endpoint(source.path());
    let exported = export_with_paths(source.path(), "0.1.0").expect("export");

    // 目标环境：另一份数据根 + 空配置。
    let target = data_root();
    let imported = import_with_container_file(target.path(), &exported.path, home.path(), None)
        .expect("import without passphrase");
    assert_eq!(imported.format_version, 2);
    assert!(imported
        .applied_sections
        .iter()
        .any(|item| item == "preferences"));
    assert!(imported
        .applied_sections
        .iter()
        .any(|item| item == "extension_config"));
    assert!(imported
        .applied_sections
        .iter()
        .any(|item| item == "sync_endpoints"));
    assert_eq!(imported.document_sha256, exported.document_sha256);

    let reloaded = PreferencesStore::new(target.path()).load().expect("reload");
    assert_eq!(reloaded.interface_scale, 150);
    assert_eq!(reloaded.panel_mode, "browser");

    let config = crate::modules::extensions::projection::load_config_lenient(target.path());
    assert!(config.enablement[&ClientId::Codex]);
    assert!(!config.enablement[&ClientId::Claude]);
    // 异机导入：凭据引用换成本机新引用，口令需重新认证。
    let sync = crate::modules::sync::config::SyncConfigStore::new(target.path())
        .load()
        .expect("load sync");
    let endpoint = sync.active().expect("endpoint");
    assert_eq!(endpoint.url, "https://dav.example.test");
    assert_ne!(
        endpoint.credential_ref.ref_id,
        "cred_12345678-1234-1234-1234-123456789abc"
    );
}

#[test]
fn legacy_container_needs_original_passphrase_and_leaves_config_untouched() {
    let root = data_root();
    let home = home_dir();
    let original = Preferences {
        interface_scale: 175,
        ..Default::default()
    };
    PreferencesStore::new(root.path())
        .save(&original)
        .expect("save preferences");
    let document = read_plaintext_document(
        &export_with_paths(root.path(), "0.1.0")
            .expect("export")
            .path
            .pipe_read(),
    );
    let legacy = legacy_container_bytes(&document, LEGACY_PASSPHRASE, "0.1.0");
    let legacy_path = root.path().join("exports/opencodex-config.ocxdconf");
    std::fs::write(&legacy_path, &legacy).expect("write legacy container");
    PreferencesStore::new(root.path())
        .save(&Preferences {
            interface_scale: 90,
            ..Default::default()
        })
        .expect("change current preferences");

    // 缺失口令：明确报「需要旧口令」，且不改动现有配置。
    let error = import_with_container_file(root.path(), &legacy_path, home.path(), None)
        .expect_err("legacy without passphrase");
    assert!(error.to_string().contains("legacy container requires"));
    assert_eq!(
        PreferencesStore::new(root.path())
            .load()
            .expect("reload")
            .interface_scale,
        90
    );

    // 错误口令：同样不改动现有配置。
    let error = import_with_container_file(
        root.path(),
        &legacy_path,
        home.path(),
        Some("wrong passphrase"),
    )
    .expect_err("legacy wrong passphrase");
    assert!(matches!(error, AppError::FileSystem { .. }));
    assert_eq!(
        PreferencesStore::new(root.path())
            .load()
            .expect("reload")
            .interface_scale,
        90
    );

    // 正确口令：按旧格式读入并应用。
    let imported = import_with_container_file(
        root.path(),
        &legacy_path,
        home.path(),
        Some(LEGACY_PASSPHRASE),
    )
    .expect("legacy import");
    assert_eq!(imported.format_version, 1);
    assert_eq!(
        PreferencesStore::new(root.path())
            .load()
            .expect("reload")
            .interface_scale,
        175
    );
}

trait ReadPathExt {
    fn pipe_read(self) -> Vec<u8>;
}

impl ReadPathExt for std::path::PathBuf {
    fn pipe_read(self) -> Vec<u8> {
        std::fs::read(self).expect("read container")
    }
}

#[test]
fn corrupted_or_unknown_container_version_does_not_touch_config() {
    let root = data_root();
    let home = home_dir();
    PreferencesStore::new(root.path())
        .save(&Preferences {
            interface_scale: 80,
            ..Default::default()
        })
        .expect("save preferences");
    export_with_paths(root.path(), "0.1.0").expect("export");
    let target = root.path().join("exports/opencodex-config.ocxdconf");

    // 损坏的完整性摘要：拒绝且不改动配置。
    let mut bytes = std::fs::read(&target).expect("read container");
    let last = bytes.len() - 2;
    bytes[last] ^= 1;
    std::fs::write(&target, &bytes).expect("tamper");
    assert!(import_with_container_file(root.path(), &target, home.path(), None).is_err());
    assert_eq!(
        PreferencesStore::new(root.path())
            .load()
            .expect("reload")
            .interface_scale,
        80
    );

    // 未知版本：拒绝且不改动配置。
    let unknown = serde_json::json!({
        "container_magic": CONTAINER_FILE_MAGIC,
        "container_format_version": 99,
    });
    std::fs::write(&target, serde_json::to_vec(&unknown).unwrap()).expect("write unknown");
    assert!(import_with_container_file(root.path(), &target, home.path(), None).is_err());
    assert_eq!(
        PreferencesStore::new(root.path())
            .load()
            .expect("reload")
            .interface_scale,
        80
    );
}

#[test]
fn export_and_import_require_valid_data_root_structure() {
    let temp = tempfile::tempdir().expect("temporary root");
    let home = home_dir();
    std::fs::create_dir_all(temp.path().join("manager-state")).expect("partial structure");
    let error = export_with_paths(temp.path(), "0.1.0").expect_err("partial root");
    assert!(matches!(error, AppError::FileSystem { .. }));
    assert!(!temp.path().join("exports").exists());

    let error = import_with_paths(temp.path(), home.path(), None).expect_err("partial import");
    assert!(matches!(error, AppError::FileSystem { .. }));
}

#[test]
fn re_export_backs_up_previous_container() {
    let root = data_root();
    PreferencesStore::new(root.path())
        .save(&Preferences::default())
        .expect("save defaults");
    export_with_paths(root.path(), "0.1.0").expect("first export");
    let second = export_with_paths(root.path(), "0.1.0").expect("second export");
    let backup_id = second.backup_id.expect("backup previous container");
    assert!(backup_id.starts_with("bk_"));
    let records =
        crate::modules::backup::list_action_records(&root.path().join("backups"), "upgrade")
            .expect("list backups");
    assert!(records
        .iter()
        .any(|record| record.manifest.backup_id == backup_id));
}

#[test]
fn import_creates_backup_before_overwriting_preferences() {
    let source = data_root();
    let target = data_root();
    let home = home_dir();
    PreferencesStore::new(source.path())
        .save(&Preferences {
            interface_scale: 130,
            ..Default::default()
        })
        .expect("save source");
    let exported = export_with_paths(source.path(), "0.1.0").expect("export");
    PreferencesStore::new(target.path())
        .save(&Preferences {
            interface_scale: 70,
            ..Default::default()
        })
        .expect("save target");

    let imported = import_with_container_file(target.path(), &exported.path, home.path(), None)
        .expect("import");
    assert!(imported.backup_id.starts_with("bk_"));
    assert_eq!(
        PreferencesStore::new(target.path())
            .load()
            .expect("reload")
            .interface_scale,
        130
    );
}
