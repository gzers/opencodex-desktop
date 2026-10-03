//! 旧版（v1）加密远端的真实读取链路。
//!
//! C 阶段保留了「按版本号识别旧容器/旧远端并兼容读取」的路径，但只做过单测，
//! 没有在真实远端上演练过。这里把一份**旧格式**的清单与载荷真实 PUT 到 TLS 服务器，
//! 再用生产客户端按新流程读取：
//! - 缺口令必须失败（前端据此提示「输入原口令」）；
//! - 带上原口令必须能读出原始内容。
//!
//! 未设置 `OCX_TEST_WEBDAV_URL` 时打印跳过原因并返回。

use opencodex_desktop_lib::infrastructure::webdav_client::{
    manifest_remote_path, payload_remote_path, WebDavClient, WebDavConfig, LATEST_FILE_NAME,
};
use opencodex_desktop_lib::modules::sync::engine::{
    open_manifest, open_payload, parse_manifest_package, parse_payload_package,
    seal_legacy_manifest, seal_legacy_payload, ManifestPackage,
};
use opencodex_desktop_lib::modules::sync::{SyncArtifact, SyncManifest};

const PASSPHRASE: &[u8] = b"legacy-passphrase";
const ARTIFACT_NAME: &str = "manager-state/preferences.json";

struct Env {
    base_url: String,
    remote_path: String,
    username: String,
    password: String,
}

fn env() -> Option<Env> {
    Some(Env {
        base_url: std::env::var("OCX_TEST_WEBDAV_URL").ok()?,
        remote_path: std::env::var("OCX_TEST_WEBDAV_PATH").unwrap_or_else(|_| "ocx-test".into()),
        username: std::env::var("OCX_TEST_WEBDAV_USER").unwrap_or_else(|_| "davuser".into()),
        password: std::env::var("OCX_TEST_WEBDAV_PASS").unwrap_or_else(|_| "dav-pass-123".into()),
    })
}

impl Env {
    fn config(&self) -> WebDavConfig {
        WebDavConfig {
            base_url: self.base_url.clone(),
            remote_path: self.remote_path.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
        }
    }

    /// 直接把给定内容 PUT 到远端（模拟旧版本客户端写下的对象）。
    async fn put_raw(&self, name: &str, body: Vec<u8>) {
        let url = format!(
            "{}/{}/{}",
            self.base_url.trim_end_matches('/'),
            self.remote_path.trim_matches('/'),
            name
        );
        let client = reqwest::Client::new();
        let response = client
            .put(&url)
            .basic_auth(&self.username, Some(&self.password))
            .body(body)
            .send()
            .await
            .expect("PUT to real server");
        assert!(
            response.status().is_success(),
            "PUT {name} returned {}",
            response.status()
        );
    }
}

#[tokio::test]
async fn legacy_encrypted_remote_is_read_with_the_original_passphrase() {
    let Some(env) = env() else {
        eprintln!("[webdav_legacy_real] 跳过：未设置 OCX_TEST_WEBDAV_URL");
        return;
    };
    let plaintext = b"{\"interface_scale\":144}";
    let digest = opencodex_desktop_lib::infrastructure::hash::sha256_hex(plaintext);
    let manifest = SyncManifest::new(
        "snap_20260920030100_abcdef123456",
        chrono::Utc::now(),
        "Legacy Device",
        vec![SyncArtifact {
            name: ARTIFACT_NAME.to_string(),
            size: plaintext.len() as u64,
            sha256: digest.clone(),
        }],
    )
    .expect("legacy manifest");
    let sealed_manifest =
        seal_legacy_manifest(&manifest, PASSPHRASE).expect("seal legacy manifest");
    let sealed_payload = seal_legacy_payload(plaintext, &digest).expect("seal legacy payload");

    let manifest_bytes = serde_json::to_vec(&sealed_manifest).expect("manifest json");
    let payload_bytes = serde_json::to_vec(&sealed_payload).expect("payload json");
    env.put_raw(
        &manifest_remote_path(&manifest.snapshot_id),
        manifest_bytes.clone(),
    )
    .await;
    env.put_raw(&payload_remote_path(&digest), payload_bytes.clone())
        .await;
    env.put_raw(
        LATEST_FILE_NAME,
        format!("{}\n", manifest.snapshot_id).into_bytes(),
    )
    .await;

    // 新流程读旧远端：必须识别为旧版。
    let client = WebDavClient::production();
    let config = env.config();
    let latest = client
        .download_latest_pointer(&config)
        .await
        .expect("latest pointer")
        .expect("pointer present");
    assert_eq!(latest, manifest.snapshot_id);
    let downloaded = client.download(&config, &latest).await.expect("manifest");
    assert_eq!(downloaded, manifest_bytes);
    let package = parse_manifest_package(&downloaded).expect("parse legacy manifest");
    assert!(
        matches!(package, ManifestPackage::Legacy(_)),
        "旧格式必须按版本号识别为 Legacy"
    );

    // 缺口令：必须失败，而不是伪装成功。
    assert!(
        open_manifest(&package, None).is_err(),
        "旧远端缺口令不得读出内容"
    );
    // 原口令：可以读出。
    let decoded = open_manifest(&package, Some(PASSPHRASE)).expect("open with passphrase");
    assert_eq!(decoded.snapshot_id, manifest.snapshot_id);
    assert_eq!(decoded.artifacts.len(), 1);

    let payload = client
        .download_payload(&config, &latest, &digest)
        .await
        .expect("legacy payload");
    assert_eq!(payload, payload_bytes);
    let envelope = parse_payload_package(&payload).expect("parse legacy payload package");
    let recovered = open_payload(&envelope, &digest).expect("open legacy payload");
    assert_eq!(recovered, plaintext);
    assert_eq!(
        opencodex_desktop_lib::infrastructure::hash::sha256_hex(&recovered),
        digest
    );
}
