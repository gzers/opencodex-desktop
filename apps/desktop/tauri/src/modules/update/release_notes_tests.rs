use super::*;
use serde_json::json;
use std::sync::Mutex;

#[tokio::test]
async fn bounded_http_transport_rejects_oversize_redirect_and_error_instead_of_missing() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut limits = crate::modules::runtime_defaults::release_notes().clone();
    limits.response_max_bytes = 8;
    let source = HttpSource::new(crate::modules::preferences::ProxyPolicy::None, &limits).unwrap();
    for (status, body, headers, expected) in [
        ("200 OK", "123456789", "", Err(Error::Size)),
        ("404 Not Found", "", "", Ok(None)),
        ("500 Internal Server Error", "", "", Err(Error::Network)),
        (
            "302 Found",
            "",
            "Location: https://evil.example/\r\n",
            Err(Error::Network),
        ),
        ("200 OK", "{}", "", Ok(Some(b"{}".to_vec()))),
    ] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let response = format!(
            "HTTP/1.1 {status}\r\n{headers}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0; 4096];
            let bytes_read = stream.read(&mut request).await.unwrap();
            assert!(bytes_read > 0);
            stream.write_all(response.as_bytes()).await.unwrap();
        });
        assert_eq!(
            source.get(&format!("http://{address}/fixture")).await,
            expected
        );
        server.await.unwrap();
    }
}

#[tokio::test]
async fn total_deadline_stops_a_hung_notes_source_without_fast_retries_or_cache() {
    struct Hung;
    #[async_trait]
    impl Source for Hung {
        async fn get(&self, _: &str) -> Result<Option<Vec<u8>>, Error> {
            std::future::pending().await
        }
    }
    let root = tempfile::tempdir().unwrap();
    let mut limits = crate::modules::runtime_defaults::release_notes().clone();
    limits.total_seconds = 1;
    let started = std::time::Instant::now();
    let result = query(
        &Hung,
        root.path(),
        Request {
            target: Target::Manager,
            channel: Channel::Stable,
            version: "0.1.10",
            updater_notes: None,
            force: false,
            now: 100,
        },
        &limits,
    )
    .await
    .unwrap();
    assert_eq!(result.notes_state, NotesState::Failed);
    assert!(started.elapsed() < Duration::from_secs(2));
    assert!(!root.path().join("cache").exists());
}

struct Fixture {
    calls: Mutex<Vec<String>>,
    responses: Mutex<std::collections::VecDeque<Result<Option<Vec<u8>>, Error>>>,
}
impl Fixture {
    fn new(items: Vec<Result<Option<Vec<u8>>, Error>>) -> Self {
        Self {
            calls: Mutex::new(vec![]),
            responses: Mutex::new(items.into()),
        }
    }
    fn count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }
}
#[async_trait]
impl Source for Fixture {
    async fn get(&self, url: &str) -> Result<Option<Vec<u8>>, Error> {
        self.calls.lock().unwrap().push(url.into());
        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected network request")
    }
}
fn package(version: &str) -> Result<Option<Vec<u8>>, Error> {
    Ok(Some(serde_json::to_vec(&json!({"name":"@bitkyc08/opencodex","version":version,"repository":{"url":"git+https://github.com/lidge-jun/opencodex.git"}})).unwrap()))
}
fn release(repo: &str, version: &str, body: serde_json::Value) -> Result<Option<Vec<u8>>, Error> {
    Ok(Some(serde_json::to_vec(&json!({"tag_name":format!("v{version}"),"html_url":format!("https://github.com/{repo}/releases/tag/v{version}"),"draft":false,"prerelease":false,"published_at":"2026-10-14T00:00:00Z","body":body})).unwrap()))
}

#[tokio::test]
async fn panel_repository_and_exact_version_are_verified_before_release_and_cache_hit_has_zero_requests(
) {
    let root = tempfile::tempdir().unwrap();
    let source = Fixture::new(vec![
        package("2.84.0"),
        release("lidge-jun/opencodex", "2.84.0", json!("actual notes")),
    ]);
    let result = query(
        &source,
        root.path(),
        Request {
            target: Target::Official,
            channel: Channel::Official,
            version: "2.84.0",
            updater_notes: None,
            force: false,
            now: 100,
        },
        crate::modules::runtime_defaults::release_notes(),
    )
    .await
    .unwrap();
    assert_eq!(result.notes_state, NotesState::Ready);
    assert_eq!(result.notes.as_deref(), Some("actual notes"));
    assert_eq!(
        result.release_url.as_deref(),
        Some("https://github.com/lidge-jun/opencodex/releases/tag/v2.84.0")
    );
    assert_eq!(
        *source.calls.lock().unwrap(),
        vec![
            "https://registry.npmjs.org/@bitkyc08%2fopencodex/2.84.0",
            "https://api.github.com/repos/lidge-jun/opencodex/releases/tags/v2.84.0"
        ]
    );
    let cached = query(
        &source,
        root.path(),
        Request {
            target: Target::Official,
            channel: Channel::Official,
            version: "2.84.0",
            updater_notes: None,
            force: false,
            now: 101,
        },
        crate::modules::runtime_defaults::release_notes(),
    )
    .await
    .unwrap();
    assert!(cached.cached);
    assert_eq!(source.count(), 2);
}

#[tokio::test]
async fn wrong_package_identity_repository_or_version_never_contacts_a_supplied_repository() {
    for mutation in [
        json!({"name":"other"}),
        json!({"version":"2.85.0"}),
        json!({"repository":{"url":"git+https://evil.example/x.git"}}),
    ] {
        let root = tempfile::tempdir().unwrap();
        let mut raw: serde_json::Value =
            serde_json::from_slice(&package("2.84.0").unwrap().unwrap()).unwrap();
        for (key, value) in mutation.as_object().unwrap() {
            raw[key] = value.clone();
        }
        let source = Fixture::new(vec![Ok(Some(serde_json::to_vec(&raw).unwrap()))]);
        let result = query(
            &source,
            root.path(),
            Request {
                target: Target::Official,
                channel: Channel::Official,
                version: "2.84.0",
                updater_notes: None,
                force: false,
                now: 100,
            },
            crate::modules::runtime_defaults::release_notes(),
        )
        .await
        .unwrap();
        assert_eq!(result.notes_state, NotesState::Failed);
        assert!(result.release_url.is_none());
        assert_eq!(source.count(), 1);
        assert!(!root.path().join("cache").exists());
    }
}

#[tokio::test]
async fn draft_wrong_tag_wrong_url_or_stable_prerelease_cannot_supply_notes_or_links() {
    for mutation in [
        json!({"draft":true}),
        json!({"tag_name":"v0.1.11"}),
        json!({"html_url":"https://evil.example/v0.1.10"}),
        json!({"prerelease":true}),
        json!({"prerelease":null}),
    ] {
        let root = tempfile::tempdir().unwrap();
        let mut raw: serde_json::Value = serde_json::from_slice(
            &release("gzers/opencodex-desktop", "0.1.10", json!("untrusted"))
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        for (key, value) in mutation.as_object().unwrap() {
            raw[key] = value.clone();
        }
        let source = Fixture::new(vec![Ok(Some(serde_json::to_vec(&raw).unwrap()))]);
        let result = query(
            &source,
            root.path(),
            Request {
                target: Target::Manager,
                channel: Channel::Stable,
                version: "0.1.10",
                updater_notes: None,
                force: false,
                now: 100,
            },
            crate::modules::runtime_defaults::release_notes(),
        )
        .await
        .unwrap();
        assert_eq!(result.notes_state, NotesState::Failed);
        assert!(result.notes.is_none() && result.release_url.is_none());
    }
}

#[tokio::test]
async fn updater_notes_take_priority_but_a_404_never_generates_a_guessed_release_link() {
    let root = tempfile::tempdir().unwrap();
    let source = Fixture::new(vec![Ok(None)]);
    let result = query(
        &source,
        root.path(),
        Request {
            target: Target::Manager,
            channel: Channel::Beta,
            version: "0.1.10-beta.1",
            updater_notes: Some("updater source"),
            force: false,
            now: 100,
        },
        crate::modules::runtime_defaults::release_notes(),
    )
    .await
    .unwrap();
    assert_eq!(result.notes_state, NotesState::Ready);
    assert_eq!(result.notes.as_deref(), Some("updater source"));
    assert_eq!(result.notes_source.as_deref(), Some("updater"));
    assert!(result.release_url.is_none());
}

#[tokio::test]
async fn missing_notes_and_network_failure_are_distinct_and_failed_notes_retry_independently() {
    let root = tempfile::tempdir().unwrap();
    let source = Fixture::new(vec![
        Err(Error::Network),
        release("gzers/opencodex-desktop", "0.1.10", json!("recovered")),
    ]);
    let failed = query(
        &source,
        root.path(),
        Request {
            target: Target::Manager,
            channel: Channel::Stable,
            version: "0.1.10",
            updater_notes: None,
            force: false,
            now: 100,
        },
        crate::modules::runtime_defaults::release_notes(),
    )
    .await
    .unwrap();
    assert_eq!(failed.notes_state, NotesState::Failed);
    assert!(!root.path().join("cache").exists());
    let recovered = query(
        &source,
        root.path(),
        Request {
            target: Target::Manager,
            channel: Channel::Stable,
            version: "0.1.10",
            updater_notes: None,
            force: false,
            now: 101,
        },
        crate::modules::runtime_defaults::release_notes(),
    )
    .await
    .unwrap();
    assert_eq!(recovered.notes_state, NotesState::Ready);
    let source = Fixture::new(vec![release(
        "gzers/opencodex-desktop",
        "0.1.11",
        json!(null),
    )]);
    let missing = query(
        &source,
        root.path(),
        Request {
            target: Target::Manager,
            channel: Channel::Stable,
            version: "0.1.11",
            updater_notes: None,
            force: false,
            now: 102,
        },
        crate::modules::runtime_defaults::release_notes(),
    )
    .await
    .unwrap();
    assert_eq!(missing.notes_state, NotesState::Unavailable);
    assert!(missing.release_url.is_some());
}

#[tokio::test]
async fn channel_version_and_root_do_not_share_details_and_force_only_refreshes_notes() {
    let root = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    let source = Fixture::new(
        (0..5)
            .map(|_| release("gzers/opencodex-desktop", "0.1.10", json!("notes")))
            .collect(),
    );
    for (r, c, f) in [
        (root.path(), Channel::Stable, false),
        (root.path(), Channel::Beta, false),
        (other.path(), Channel::Stable, false),
        (root.path(), Channel::Stable, true),
    ] {
        assert!(
            !query(
                &source,
                r,
                Request {
                    target: Target::Manager,
                    channel: c,
                    version: "0.1.10",
                    updater_notes: None,
                    force: f,
                    now: 100
                },
                crate::modules::runtime_defaults::release_notes()
            )
            .await
            .unwrap()
            .cached
        );
    }
    assert_eq!(source.count(), 4);
    assert!(root
        .path()
        .join("cache/updates/release-notes/manager-stable.json")
        .is_file());
    assert!(root
        .path()
        .join("cache/updates/release-notes/manager-beta.json")
        .is_file());
    assert!(!root
        .path()
        .join(super::super::schedule::STATE_PATH)
        .exists());
}

#[tokio::test]
async fn expired_future_corrupt_and_wrong_version_cache_are_refetched() {
    let root = tempfile::tempdir().unwrap();
    let limits = crate::modules::runtime_defaults::release_notes();
    let source = Fixture::new(
        (0..6)
            .map(|_| release("gzers/opencodex-desktop", "0.1.10", json!("notes")))
            .collect(),
    );
    query(
        &source,
        root.path(),
        Request {
            target: Target::Manager,
            channel: Channel::Stable,
            version: "0.1.10",
            updater_notes: None,
            force: false,
            now: 100,
        },
        limits,
    )
    .await
    .unwrap();
    assert!(
        !query(
            &source,
            root.path(),
            Request {
                target: Target::Manager,
                channel: Channel::Stable,
                version: "0.1.10",
                updater_notes: None,
                force: false,
                now: 100 + limits.cache_seconds
            },
            limits
        )
        .await
        .unwrap()
        .cached
    );
    assert!(
        !query(
            &source,
            root.path(),
            Request {
                target: Target::Manager,
                channel: Channel::Stable,
                version: "0.1.10",
                updater_notes: None,
                force: false,
                now: 99
            },
            limits
        )
        .await
        .unwrap()
        .cached
    );
    let path = root
        .path()
        .join("cache/updates/release-notes/manager-stable.json");
    std::fs::write(&path, b"invalid").unwrap();
    query(
        &source,
        root.path(),
        Request {
            target: Target::Manager,
            channel: Channel::Stable,
            version: "0.1.10",
            updater_notes: None,
            force: false,
            now: 100,
        },
        limits,
    )
    .await
    .unwrap();
    let mut cache: Cache = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    cache.details.candidate_version = "0.1.11".into();
    std::fs::write(&path, serde_json::to_vec(&cache).unwrap()).unwrap();
    query(
        &source,
        root.path(),
        Request {
            target: Target::Manager,
            channel: Channel::Stable,
            version: "0.1.10",
            updater_notes: None,
            force: false,
            now: 101,
        },
        limits,
    )
    .await
    .unwrap();
    assert_eq!(source.count(), 5);
}

#[tokio::test]
async fn invalid_request_is_rejected_before_any_network_or_filesystem_mutation() {
    let root = tempfile::tempdir().unwrap();
    let source = Fixture::new(vec![]);
    for (t, c, v) in [
        (Target::Manager, Channel::Official, "0.1.10"),
        (Target::Official, Channel::Stable, "2.84.0"),
        (Target::Manager, Channel::Stable, "1.0/../x"),
        (Target::Manager, Channel::Stable, "1.0?x"),
    ] {
        assert_eq!(
            query(
                &source,
                root.path(),
                Request {
                    target: t,
                    channel: c,
                    version: v,
                    updater_notes: None,
                    force: false,
                    now: 100
                },
                crate::modules::runtime_defaults::release_notes()
            )
            .await,
            Err(Error::Source)
        );
    }
    assert_eq!(source.count(), 0);
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
}

#[test]
fn utf8_notes_are_bounded_and_remote_html_stays_literal_text() {
    let value = "改进🦀".repeat(30000);
    let result = bounded_text(Some(&value), 65536).unwrap();
    assert!(result.len() <= 65536 && value.starts_with(&result));
    assert_eq!(bounded_text(Some("  "), 65536), None);
    assert_eq!(
        bounded_text(Some("<img onerror=evil()>"), 65536).as_deref(),
        Some("<img onerror=evil()>")
    );
}

#[cfg(unix)]
#[tokio::test]
async fn cache_link_escape_does_not_write_external_directory() {
    let root = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(external.path(), root.path().join("cache")).unwrap();
    let source = Fixture::new(vec![release(
        "gzers/opencodex-desktop",
        "0.1.10",
        json!("notes"),
    )]);
    assert_eq!(
        query(
            &source,
            root.path(),
            Request {
                target: Target::Manager,
                channel: Channel::Stable,
                version: "0.1.10",
                updater_notes: None,
                force: false,
                now: 100
            },
            crate::modules::runtime_defaults::release_notes()
        )
        .await,
        Err(Error::Cache)
    );
    assert_eq!(std::fs::read_dir(external.path()).unwrap().count(), 0);
}
