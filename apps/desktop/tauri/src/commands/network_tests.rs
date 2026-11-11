use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
fn body(target: ProbeTarget) -> String {
    match target{
 ProbeTarget::Github=>serde_json::json!({"full_name":"gzers/opencodex-desktop","html_url":"https://github.com/gzers/opencodex-desktop"}),
 ProbeTarget::Npm=>serde_json::json!({"name":"@bitkyc08/opencodex","version":"2.99.0","repository":{"url":"git+https://github.com/lidge-jun/opencodex.git"}}),
}.to_string()
}
async fn server(
    status: &str,
    headers: &str,
    body: String,
) -> (String, tokio::task::JoinHandle<String>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/fixture", listener.local_addr().unwrap());
    let response = format!(
        "HTTP/1.1 {status}\r\n{headers}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let task = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = [0; 4096];
        let n = stream.read(&mut request).await.unwrap();
        let _ = stream.write_all(response.as_bytes()).await;
        String::from_utf8_lossy(&request[..n]).to_string()
    });
    (url, task)
}
fn client() -> reqwest::Client {
    crate::modules::network_defaults::http_client(
        ProxyPolicy::None,
        "",
        std::time::Duration::from_secs(1),
    )
    .unwrap()
}
#[tokio::test]
async fn github_and_npm_results_remain_independent_and_one_failed_source_does_not_hide_the_other() {
    let (a, ta) = server(
        "429 Too Many Requests",
        "Retry-After: 12\r\n",
        String::new(),
    )
    .await;
    let (b, tb) = server("200 OK", "", body(ProbeTarget::Npm)).await;
    let result = probe_pair(
        Ok(client()),
        [(ProbeTarget::Github, a), (ProbeTarget::Npm, b)],
    )
    .await;
    assert!(!result.ok);
    assert_eq!(result.results[0].state, ProbeState::RateLimited);
    assert_eq!(result.results[0].retry_after_seconds, Some(12));
    assert_eq!(result.results[1].state, ProbeState::Ready);
    assert!(ta.await.unwrap().starts_with("GET /fixture "));
    assert!(tb.await.unwrap().starts_with("GET /fixture "));
    let encoded = serde_json::to_string(&result).unwrap();
    assert!(!encoded.contains("Retry-After"));
    assert!(!encoded.contains("127.0.0.1"));
}
#[tokio::test]
async fn http_identity_redirect_duplicate_json_and_oversize_fail_with_fixed_classifications() {
    for (status, headers, payload, expected) in [
        (
            "500 Internal Server Error",
            "",
            String::new(),
            ProbeState::HttpError,
        ),
        (
            "403 Forbidden",
            "X-RateLimit-Remaining: 0\r\n",
            String::new(),
            ProbeState::RateLimited,
        ),
        (
            "302 Found",
            "Location: https://untrusted.invalid/secret\r\n",
            String::new(),
            ProbeState::IdentityMismatch,
        ),
        (
            "200 OK",
            "",
            body(ProbeTarget::Npm),
            ProbeState::IdentityMismatch,
        ),
        (
            "200 OK",
            "",
            r#"{"full_name":"gzers/opencodex-desktop","full_name":"other"}"#.to_string(),
            ProbeState::IdentityMismatch,
        ),
        (
            "200 OK",
            "",
            "x".repeat(
                crate::modules::runtime_defaults::release_notes().response_max_bytes as usize + 1,
            ),
            ProbeState::TooLarge,
        ),
        ("200 OK", "", body(ProbeTarget::Github), ProbeState::Ready),
    ] {
        let (url, task) = server(status, headers, payload).await;
        let r = one(&client(), ProbeTarget::Github, &url).await;
        assert_eq!(r.state, expected);
        task.await.unwrap();
    }
}
#[tokio::test]
async fn manual_http_proxy_receives_only_the_fixed_request_and_saved_bypass_uses_direct_route() {
    let (url, task) = server("200 OK", "", body(ProbeTarget::Github)).await;
    let proxy = url.strip_suffix("/fixture").unwrap();
    let c = crate::modules::network_defaults::http_client(
        ProxyPolicy::Manual(proxy.into()),
        "",
        std::time::Duration::from_secs(1),
    )
    .unwrap();
    let r = one(
        &c,
        ProbeTarget::Github,
        "http://must-not-resolve.invalid/fixture",
    )
    .await;
    assert_eq!(r.state, ProbeState::Ready);
    let raw = task.await.unwrap();
    assert!(raw.starts_with("GET http://must-not-resolve.invalid/fixture "));
    let (url, task) = server("200 OK", "", body(ProbeTarget::Github)).await;
    let c = crate::modules::network_defaults::http_client(
        ProxyPolicy::Manual("http://127.0.0.1:9".into()),
        "127.0.0.1",
        std::time::Duration::from_secs(1),
    )
    .unwrap();
    assert_eq!(
        one(&c, ProbeTarget::Github, &url).await.state,
        ProbeState::Ready
    );
    assert!(task.await.unwrap().starts_with("GET /fixture "));
}
#[tokio::test]
async fn timeout_is_distinct_and_invalid_proxy_never_dispatches_a_probe() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        let (_stream, _) = listener.accept().await.unwrap();
        std::future::pending::<()>().await
    });
    let c = crate::modules::network_defaults::http_client(
        ProxyPolicy::None,
        "",
        std::time::Duration::from_millis(25),
    )
    .unwrap();
    assert_eq!(
        one(&c, ProbeTarget::Github, &url).await.state,
        ProbeState::Timeout
    );
    task.abort();
    let _ = task.await;
    let invalid = crate::modules::network_defaults::http_client(
        ProxyPolicy::Manual("http://".into()),
        "",
        std::time::Duration::from_secs(1),
    );
    let r = probe_client(invalid).await;
    assert!(!r.ok);
    assert!(r
        .results
        .iter()
        .all(|r| r.state == ProbeState::InvalidProxy));
}
