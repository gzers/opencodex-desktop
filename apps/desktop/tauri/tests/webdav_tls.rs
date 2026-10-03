//! WebDAV 传输必须真的带 TLS 后端。
//!
//! 冻结契约要求 WebDAV 端点必须是 `https://`（`WebDavConfig::validate`），
//! 因此 HTTP 客户端一旦在编译期丢掉 TLS feature，整条同步链路就会 100% 不可用，
//! 而错误信息还会伪装成「TLS 证书校验失败」。这里用两条不依赖外网的护栏盯住它。

/// 收集 reqwest 错误的完整 source 链，用于区分「没有 TLS 后端」与「真的连不上」。
fn error_chain(error: &(dyn std::error::Error + 'static)) -> Vec<String> {
    let mut chain = Vec::new();
    let mut current = error.source();
    while let Some(inner) = current {
        chain.push(inner.to_string());
        current = inner.source();
    }
    chain
}

/// 编译期护栏：`use_rustls_tls()` 只在启用 rustls feature 时存在。
/// 一旦有人把 `Cargo.toml` 里的 TLS feature 去掉，本测试直接编译失败。
#[test]
fn tls_backend_is_compiled_in() {
    let _ = reqwest::Client::builder().use_rustls_tls();
}

/// 行为护栏：`https://` 必须能走到传输层。
/// 缺少 TLS 后端时 reqwest 会在构造请求时直接报 `invalid URL, scheme is not http`
/// （历史上就是这么坏的）；带 TLS 时同一请求只是连不上（本机 1 端口未监听）。
/// 该断言不依赖外网，也不依赖证书，离线可跑。
#[tokio::test]
async fn https_scheme_reaches_the_transport_layer() {
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_millis(800))
        .timeout(std::time::Duration::from_millis(1500))
        .build()
        .expect("build client");
    let error = client
        .get("https://127.0.0.1:1/")
        .send()
        .await
        .expect_err("closed port must not answer");
    let chain = error_chain(&error);
    assert!(
        !chain
            .iter()
            .any(|message| message.contains("scheme is not http")),
        "reqwest 没有可用的 TLS 后端，https 请求在 scheme 阶段就被拒绝：{chain:?}"
    );
    assert!(
        chain
            .iter()
            .any(|message| message.contains("onnection refused")),
        "预期本机 1 端口拒绝连接，实际错误链：{chain:?}"
    );
}
