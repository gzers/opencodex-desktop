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
/// （历史上就是这么坏的）。本机临时 TCP 端点只记录 TLS 握手首字节再关闭，
/// 不依赖 Windows 对关闭端口的超时行为或本地化错误文本，也不访问外网。
#[tokio::test]
async fn https_scheme_reaches_the_transport_layer() {
    use std::io::Read;
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let server = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    // macOS 的 accepted socket 会继承 listener 的非阻塞模式；
                    // 显式恢复阻塞，确保 read_timeout 能等待尚未到达的握手字节。
                    stream.set_nonblocking(false).unwrap();
                    stream
                        .set_read_timeout(Some(std::time::Duration::from_secs(1)))
                        .unwrap();
                    let mut first = [0; 1];
                    stream.read_exact(&mut first).ok()?;
                    return Some(first[0]);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if std::time::Instant::now() >= deadline {
                        return None;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(_) => return None,
            }
        }
    });
    let client = reqwest::Client::builder()
        .no_proxy()
        .connect_timeout(std::time::Duration::from_millis(800))
        .timeout(std::time::Duration::from_millis(1500))
        .build()
        .expect("build client");
    let error = client
        .get(format!("https://{address}/"))
        .send()
        .await
        .expect_err("endpoint closes before completing TLS");
    let chain = error_chain(&error);
    assert!(
        !chain
            .iter()
            .any(|message| message.contains("scheme is not http")),
        "reqwest 没有可用的 TLS 后端，https 请求在 scheme 阶段就被拒绝：{chain:?}"
    );
    assert_eq!(
        server.join().unwrap(),
        Some(0x16),
        "TLS handshake must reach loopback transport"
    );
}
