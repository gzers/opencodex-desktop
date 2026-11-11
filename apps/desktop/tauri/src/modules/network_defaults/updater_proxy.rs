//! Pinned updater request hook. The plugin applies this same hook to check and
//! download builders; this module cannot change endpoints or signatures.
use crate::{
    errors::{AppError, AppResult},
    modules::preferences::ProxyPolicy,
};
#[derive(Clone)]
pub(crate) struct ClientPolicy {
    policy: Policy,
}
#[derive(Clone)]
enum Policy {
    Direct,
    System,
    Manual(Box<reqwest_updater::Proxy>),
}
impl ClientPolicy {
    pub(crate) fn new(policy: ProxyPolicy, bypass: String) -> AppResult<Self> {
        let policy = match policy {
            ProxyPolicy::None => Policy::Direct,
            ProxyPolicy::System => Policy::System,
            ProxyPolicy::Manual(url) => {
                let parsed =
                    reqwest_updater::Url::parse(&url).map_err(|_| AppError::NotConfigured)?;
                if !matches!(parsed.scheme(), "http" | "socks5h")
                    || parsed.host().is_none()
                    || !parsed.username().is_empty()
                    || parsed.password().is_some()
                    || parsed.query().is_some()
                    || parsed.fragment().is_some()
                    || !matches!(parsed.path(), "" | "/")
                {
                    return Err(AppError::NotConfigured);
                }
                let proxy = reqwest_updater::Proxy::all(parsed.as_str())
                    .map_err(|_| AppError::NotConfigured)?
                    .no_proxy(reqwest_updater::NoProxy::from_string(&bypass));
                Policy::Manual(Box::new(proxy))
            }
        };
        Ok(Self { policy })
    }
    pub(crate) fn apply(
        &self,
        builder: reqwest_updater::ClientBuilder,
    ) -> reqwest_updater::ClientBuilder {
        match &self.policy {
            Policy::Direct => builder.no_proxy(),
            Policy::System => builder,
            Policy::Manual(proxy) => builder.no_proxy().proxy((**proxy).clone()),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };
    async fn fixture() -> (String, tokio::task::JoinHandle<String>) {
        let server = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/fixture", server.local_addr().unwrap());
        let task = tokio::spawn(async move {
            let (mut socket, _) = server.accept().await.unwrap();
            let mut bytes = [0; 4096];
            let n = socket.read(&mut bytes).await.unwrap();
            socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}")
                .await
                .unwrap();
            String::from_utf8_lossy(&bytes[..n]).into_owned()
        });
        (url, task)
    }
    #[tokio::test]
    async fn metadata_and_artifact_requests_share_the_saved_bypass_and_actual_manual_route() {
        // The real plugin performs this exact setup before its first check;
        // transport-only fixtures do not run the Tauri plugin lifecycle.
        let _ = rustls::crypto::ring::default_provider().install_default();

        for route in ["metadata", "artifact"] {
            let (url, task) = fixture().await;
            let proxy = url.strip_suffix("/fixture").unwrap();
            let policy =
                ClientPolicy::new(ProxyPolicy::Manual(proxy.into()), String::new()).unwrap();
            let client = policy
                .apply(
                    reqwest_updater::Client::builder().timeout(std::time::Duration::from_secs(2)),
                )
                .build()
                .unwrap();
            let path = format!("http://unresolvable.invalid/{route}");
            assert!(client
                .get(&path)
                .send()
                .await
                .unwrap()
                .status()
                .is_success());
            assert!(task
                .await
                .unwrap()
                .starts_with(&format!("GET {path} HTTP/1.1")));
            let (url, task) = fixture().await;
            let policy = ClientPolicy::new(
                ProxyPolicy::Manual("http://127.0.0.1:9".into()),
                "127.0.0.1".into(),
            )
            .unwrap();
            let client = policy
                .apply(
                    reqwest_updater::Client::builder().timeout(std::time::Duration::from_secs(2)),
                )
                .build()
                .unwrap();
            assert!(client.get(url).send().await.unwrap().status().is_success());
            assert!(task.await.unwrap().starts_with("GET /fixture HTTP/1.1"));
        }
    }
    #[tokio::test]
    async fn direct_clears_prior_proxy_but_system_preserves_platform_client_selection() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let (url, task) = fixture().await;
        let direct = ClientPolicy::new(ProxyPolicy::None, String::new()).unwrap();
        let client = direct
            .apply(
                reqwest_updater::Client::builder()
                    .proxy(reqwest_updater::Proxy::all("http://127.0.0.1:9").unwrap())
                    .timeout(std::time::Duration::from_secs(2)),
            )
            .build()
            .unwrap();
        assert!(client.get(url).send().await.unwrap().status().is_success());
        assert!(task.await.unwrap().starts_with("GET /fixture HTTP/1.1"));
        let (url, task) = fixture().await;
        let system = ClientPolicy::new(ProxyPolicy::System, String::new()).unwrap();
        let client = system
            .apply(
                reqwest_updater::Client::builder()
                    .proxy(
                        reqwest_updater::Proxy::all(url.strip_suffix("/fixture").unwrap()).unwrap(),
                    )
                    .timeout(std::time::Duration::from_secs(2)),
            )
            .build()
            .unwrap();
        assert!(client
            .get("http://platform-resolved.invalid/fixture")
            .send()
            .await
            .unwrap()
            .status()
            .is_success());
        assert!(task
            .await
            .unwrap()
            .starts_with("GET http://platform-resolved.invalid/fixture HTTP/1.1"));
    }
    #[test]
    fn invalid_manual_proxy_is_rejected_instead_of_silently_using_direct_or_system() {
        for value in [
            "http://",
            "http://u:p@proxy:8080",
            "https://proxy:8080",
            "http://proxy:8080/path",
            "file:///tmp/proxy",
        ] {
            assert!(
                ClientPolicy::new(ProxyPolicy::Manual(value.into()), String::new()).is_err(),
                "{value}"
            )
        }
        assert!(ClientPolicy::new(
            ProxyPolicy::Manual("socks5h://127.0.0.1:7890".into()),
            String::new()
        )
        .is_ok());
        assert!(ClientPolicy::new(ProxyPolicy::None, String::new()).is_ok());
    }
}
