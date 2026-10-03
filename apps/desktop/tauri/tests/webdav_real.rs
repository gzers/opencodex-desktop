//! 真实 WebDAV 远端链路（需要一台真实 TLS WebDAV 服务器）。
//!
//! 与其他测试不同，这里**不注入传输替身**：直接使用生产实现
//! `WebDavClient::production()`（reqwest + rustls），对一台真实服务器做
//! PROPFIND / PUT / GET 往返。用于证明「同步上传在标准 WebDAV 服务器上真的可用」，
//! 而不是只对替身成立。
//!
//! 环境缺失时用例会打印跳过原因并返回（视为未验证，不计入通过）：
//! - `OCX_TEST_WEBDAV_URL`：形如 `https://localhost:8443` 的根地址；
//! - 可选 `OCX_TEST_WEBDAV_PATH`（默认 `ocx-test`）、`OCX_TEST_WEBDAV_USER`
//!   （默认 `davuser`）、`OCX_TEST_WEBDAV_PASS`（默认 `dav-pass-123`）。
//!
//! 自签证书时用 `SSL_CERT_FILE` 指向 CA（rustls-native-certs 会读取）。

use opencodex_desktop_lib::infrastructure::webdav_client::{
    manifest_remote_path, payload_remote_path, WebDavClient, WebDavConfig, WebDavError,
    WebDavOperation,
};

fn endpoint() -> Option<WebDavConfig> {
    let base_url = std::env::var("OCX_TEST_WEBDAV_URL").ok()?;
    Some(WebDavConfig {
        base_url,
        remote_path: std::env::var("OCX_TEST_WEBDAV_PATH").unwrap_or_else(|_| "ocx-test".into()),
        username: std::env::var("OCX_TEST_WEBDAV_USER").unwrap_or_else(|_| "davuser".into()),
        password: std::env::var("OCX_TEST_WEBDAV_PASS").unwrap_or_else(|_| "dav-pass-123".into()),
    })
}

fn skip(name: &str) -> bool {
    eprintln!(
        "[webdav_real] 跳过 {name}：未设置 OCX_TEST_WEBDAV_URL（需要真实 TLS WebDAV 服务器）"
    );
    true
}

fn snapshot_id(seed: u32) -> String {
    format!("snap_202609200301{:02}_abcdef{:06x}", seed % 60, seed)
}

/// 探活：对真实服务器发起 PROPFIND。
#[tokio::test]
async fn probe_reaches_real_server() {
    let Some(config) = endpoint() else {
        skip("probe_reaches_real_server");
        return;
    };
    WebDavClient::production()
        .test_connection(&config)
        .await
        .expect("生产客户端应能连上真实 WebDAV 服务器");
}

/// 上传后必须能在服务端真实读回：latest 指针、manifest、payload 三者一致。
/// 这条用例会暴露「PUT 到不存在的集合」这一标准服务器行为（409）。
#[tokio::test]
async fn upload_then_download_round_trip() {
    let Some(config) = endpoint() else {
        skip("upload_then_download_round_trip");
        return;
    };
    let client = WebDavClient::production();
    let id = snapshot_id(0xA11CE);
    let artifacts: Vec<(String, String, Vec<u8>)> = vec![
        (
            "manager-state/preferences.json".to_string(),
            b"{\"interface_scale\":100}".to_vec(),
        ),
        (
            "manager-state/extension-config.json".to_string(),
            b"{\"skills\":[]}".to_vec(),
        ),
    ]
    .into_iter()
    .map(|(name, bytes)| {
        let digest = opencodex_desktop_lib::infrastructure::hash::sha256_hex(&bytes);
        (name, digest, bytes)
    })
    .collect();
    let manifest = b"{\"snapshot_id\":\"placeholder\"}".to_vec();

    client
        .upload(&config, &id, &artifacts, &manifest)
        .await
        .expect("上传到真实 WebDAV 服务器应成功");

    let latest = client
        .download_latest_pointer(&config)
        .await
        .expect("读取 latest 指针");
    assert_eq!(latest.as_deref(), Some(id.as_str()));

    let read_manifest = client.download(&config, &id).await.expect("读回 manifest");
    assert_eq!(read_manifest, manifest);

    for (_, digest, payload) in &artifacts {
        let read_back = client
            .download_payload(&config, &id, digest)
            .await
            .expect("读回 payload");
        assert_eq!(&read_back, payload);
    }

    // 布局自检：manifest 与 payload 直接落在远端目录下，不依赖中间集合。
    assert_eq!(manifest_remote_path(&id), format!("{id}.ocxd"));
    let sample_digest = "aa".repeat(32);
    assert_eq!(
        payload_remote_path(&sample_digest),
        format!("{sample_digest}.ocxd")
    );
}

/// 口令错误必须映射为「认证失败」，且本地内容不变。
#[tokio::test]
async fn wrong_password_is_unauthorized() {
    let Some(mut config) = endpoint() else {
        skip("wrong_password_is_unauthorized");
        return;
    };
    config.password = "definitely-wrong".to_string();
    let error = WebDavClient::production()
        .test_connection(&config)
        .await
        .expect_err("错误口令不得成功");
    assert_eq!(error, WebDavError::Unauthorized);
}

/// 远端不存在的快照必须映射为「资源不存在」，而不是伪装成功或协议错误。
#[tokio::test]
async fn missing_snapshot_is_not_found() {
    let Some(config) = endpoint() else {
        skip("missing_snapshot_is_not_found");
        return;
    };
    let error = WebDavClient::production()
        .download(&config, &snapshot_id(0xBEEF))
        .await
        .expect_err("不存在的快照不得成功");
    assert_eq!(error, WebDavError::NotFound);
}

/// 连不上的端点必须如实报错（网络/TLS 类），不能伪装成功。
/// 不依赖 `OCX_TEST_WEBDAV_URL`：固定指向本机一个未监听端口。
#[tokio::test]
async fn closed_port_is_reported_as_failure() {
    let config = WebDavConfig {
        base_url: "https://127.0.0.1:1".to_string(),
        remote_path: "ocx-test".to_string(),
        username: "davuser".to_string(),
        password: "dav-pass-123".to_string(),
    };
    let error = WebDavClient::production()
        .test_connection(&config)
        .await
        .expect_err("未监听端口不得成功");
    assert!(
        matches!(error, WebDavError::Tls | WebDavError::Network),
        "预期传输层失败，实际 {error:?}"
    );
    // 只要不是成功，用户看到的都是「本地内容未修改」类文案。
    assert!(!error.user_message(WebDavOperation::Probe).is_empty());
}

/// 目标只接受 TCP 连接、从不回应 TLS 握手时，必须在**连接超时**后如实失败，
/// 而不是无限等待或伪装成功。（本机黑洞口，不依赖外网；约 15 秒，来自冻结的 CONNECT_TIMEOUT。）
#[tokio::test]
async fn black_hole_endpoint_times_out_instead_of_hanging() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind black hole");
    let port = listener.local_addr().expect("addr").port();
    // 接受连接后既不回应也不关闭，模拟半开/挂死的远端。
    std::thread::spawn(move || {
        let mut held = Vec::new();
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => held.push(stream),
                Err(_) => break,
            }
        }
        drop(held);
    });
    let config = WebDavConfig {
        base_url: format!("https://127.0.0.1:{port}"),
        remote_path: "ocx-test".to_string(),
        username: "davuser".to_string(),
        password: "dav-pass-123".to_string(),
    };
    let started = std::time::Instant::now();
    let error = WebDavClient::production()
        .test_connection(&config)
        .await
        .expect_err("只握手不应答的远端不得成功");
    assert!(
        matches!(error, WebDavError::Tls | WebDavError::Network),
        "预期传输层失败，实际 {error:?}"
    );
    assert!(
        started.elapsed() >= std::time::Duration::from_secs(5),
        "应在连接超时后才失败，实际仅 {:?}",
        started.elapsed()
    );
}
