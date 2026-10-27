//! Real loopback HTTP vectors for the supported read-only discovery contract.
use super::*;
use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::json;
use std::sync::{Arc, Mutex};

fn request(port: u16, protocol: &str, path: &str, key: Option<&str>) -> DiscoveryRequest {
    DiscoveryRequest {
        protocol: protocol.into(),
        url: format!("http://127.0.0.1:{port}{path}"),
        api_key: key.map(str::to_owned),
        credential_ref: None,
        allow_private_network: true,
        provider_id: None,
        home: None,
    }
}

#[tokio::test]
async fn supported_protocol_auth_paths_and_complete_pagination_preserve_raw_ids_without_inventing_capabilities(
) {
    for (protocol, path, total) in [
        ("openai-chat", "/v1", 104usize),
        ("openai-responses", "/v1", 30),
        ("anthropic", "/v1", 108),
        ("google", "/v1beta", 101),
        ("ollama-native", "", 3),
    ] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let observed = Arc::new(Mutex::new(Vec::new()));
        let seen = observed.clone();
        let route = if protocol == "ollama-native" {
            "/api/tags"
        } else if protocol == "google" {
            "/v1beta/models"
        } else {
            "/v1/models"
        };
        let app=Router::new().route(route,get(move |headers:HeaderMap,Query(query):Query<std::collections::BTreeMap<String,String>>|{let seen=seen.clone();async move {
   let key=match protocol{"anthropic"=>"x-api-key","google"=>"x-goog-api-key",_=>"authorization"};
   assert_eq!(headers.get(key).unwrap().to_str().unwrap(),if key=="authorization"{"Bearer fixture-secret"}else{"fixture-secret"});
   if protocol=="anthropic"{assert_eq!(headers.get("anthropic-version").unwrap(),"2023-06-01");}
   if key!="authorization"{assert!(headers.get("authorization").is_none());}
   let cursor=if protocol=="google"{"pageToken"}else if protocol=="anthropic"{"after_id"}else{"after"};
   let paged=protocol!="ollama-native";let second=!query.is_empty();if second{assert_eq!(query.get(cursor).unwrap(),"page /+ second");assert_eq!(query.len(),1);}
   seen.lock().unwrap().push(query);
   let end=if paged&&!second{total/2}else{total};let start=if second{total/2}else{0};
   let items=(start..end).map(|i|{let raw=format!("vendor/raw-{i:03}");match protocol{"google"=>json!({"name":format!("models/{raw}"),"displayName":format!("Display {i}")}),"ollama-native"=>json!({"model":raw}),_=>json!({"id":raw})}}).collect::<Vec<_>>();
   let mut body=if ["google","ollama-native"].contains(&protocol){json!({"models":items})}else{json!({"data":items})};
   if paged&&!second{if protocol=="google"{body["nextPageToken"]=json!("page /+ second")}else{body["has_more"]=json!(true);body["last_id"]=json!("page /+ second")}}
   Json(body)
  }}));
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let result = preview(request(port, protocol, path, Some("fixture-secret")))
            .await
            .unwrap();
        server.abort();
        assert_eq!(result.models.len(), total, "{protocol}");
        assert!(result.credential_ref.is_none());
        for (i, model) in result.models.iter().enumerate() {
            assert_eq!(model.id, format!("vendor/raw-{i:03}"));
            assert_eq!(model.rules, Default::default());
        }
        assert_eq!(
            observed.lock().unwrap().len(),
            if protocol == "ollama-native" { 1 } else { 2 }
        );
    }
}

#[tokio::test]
async fn anthropic_auth_free_relay_keeps_protocol_version_without_adding_a_key() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let app = Router::new().route(
        "/v1/models",
        get(|headers: HeaderMap| async move {
            assert_eq!(headers.get("anthropic-version").unwrap(), "2023-06-01");
            assert!(headers.get("x-api-key").is_none());
            assert!(headers.get("authorization").is_none());
            Json(json!({"data":[]}))
        }),
    );
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let result = preview(request(port, "anthropic", "/v1", None))
        .await
        .unwrap();
    assert!(result.models.is_empty());
    assert!(result.credential_ref.is_none());
    server.abort();
}

#[tokio::test]
async fn missing_or_repeated_cursor_never_reports_a_partial_list_as_complete() {
    for repeated in [false, true] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let observed = calls.clone();
        let app = Router::new().route(
            "/v1/models",
            get(move || {
                let observed = observed.clone();
                async move {
                    observed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let mut body = json!({"data":[{"id":"partial"}],"has_more":true});
                    if repeated {
                        body["last_id"] = json!("same");
                    }
                    Json(body)
                }
            }),
        );
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let error = preview(request(port, "openai-chat", "/v1", None))
            .await
            .unwrap_err();
        server.abort();
        assert_eq!(error.code, "discovery_pagination");
        assert_eq!(
            calls.load(std::sync::atomic::Ordering::SeqCst),
            if repeated { 2 } else { 1 }
        );
    }
}

#[tokio::test]
async fn pending_auth_failure_and_real_empty_are_distinct_and_server_secrets_do_not_escape() {
    for (status, code) in [
        (StatusCode::CONFLICT, "discovery_failed"),
        (StatusCode::UNAUTHORIZED, "discovery_auth"),
        (StatusCode::FORBIDDEN, "discovery_auth"),
        (StatusCode::INTERNAL_SERVER_ERROR, "discovery_failed"),
    ] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let app = Router::new().route(
            "/v1/models",
            get(move || async move { (status, "fixture-secret echoed by hostile upstream") }),
        );
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let error = preview(request(
            port,
            "openai-responses",
            "/v1",
            Some("fixture-secret"),
        ))
        .await
        .unwrap_err();
        server.abort();
        assert_eq!(error.code, code);
        assert!(error.message.contains(&status.as_u16().to_string()));
        assert!(!error.message.contains("fixture-secret"));
    }
}

struct Fixture {
    port: u16,
    calls: Arc<Mutex<Vec<String>>>,
    server: tokio::task::JoinHandle<()>,
}

impl Fixture {
    fn paths(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}

async fn fixture(
    respond: impl Fn(&Uri, &HeaderMap) -> Response + Send + Sync + 'static,
) -> Fixture {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let observed = calls.clone();
    let respond = Arc::new(respond);
    let app = Router::new().fallback(get(move |uri: Uri, headers: HeaderMap| {
        let observed = observed.clone();
        let respond = respond.clone();
        async move {
            observed.lock().unwrap().push(uri.to_string());
            respond(&uri, &headers)
        }
    }));
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    Fixture {
        port,
        calls,
        server,
    }
}

fn assert_fixture_auth(protocol: &str, headers: &HeaderMap) {
    assert_eq!(headers.get("accept").unwrap(), "application/json");
    match protocol {
        "anthropic" => {
            assert_eq!(headers.get("x-api-key").unwrap(), "fixture-secret");
            assert_eq!(headers.get("anthropic-version").unwrap(), "2023-06-01");
            assert!(headers.get("authorization").is_none());
        }
        "google" => {
            assert_eq!(headers.get("x-goog-api-key").unwrap(), "fixture-secret");
            assert!(headers.get("authorization").is_none());
        }
        _ => assert_eq!(
            headers.get("authorization").unwrap(),
            "Bearer fixture-secret"
        ),
    }
}

#[tokio::test]
async fn root_version_full_url_and_proxy_prefix_use_the_expected_first_request() {
    for protocol in ["openai-chat", "openai-responses", "anthropic"] {
        for (base, expected) in [
            ("", "/v1/models"),
            ("/", "/v1/models"),
            ("/v1/", "/v1/models"),
            ("/v1/models/", "/v1/models"),
            ("/models", "/models"),
            ("/proxy/v1/", "/proxy/v1/models"),
            ("/proxy/v1/models", "/proxy/v1/models"),
            ("/proxy", "/proxy/models"),
            ("/proxy/models", "/proxy/models"),
        ] {
            let fixture = fixture(move |uri, headers| {
                assert_fixture_auth(protocol, headers);
                if uri.path() == expected {
                    Json(json!({"data":[{"id":"vendor/verified"}]})).into_response()
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            })
            .await;
            let result = preview(request(
                fixture.port,
                protocol,
                base,
                Some("fixture-secret"),
            ))
            .await
            .unwrap();
            assert_eq!(result.models[0].id, "vendor/verified");
            assert!(result.credential_ref.is_none());
            assert_eq!(fixture.paths(), [expected], "{protocol} {base}");
        }
    }
    for (protocol, base, expected, body) in [
        (
            "google",
            "",
            "/v1beta/models",
            json!({"models":[{"name":"models/raw/google"}]}),
        ),
        (
            "google",
            "/proxy/v1/models/",
            "/proxy/v1/models",
            json!({"models":[{"name":"models/raw/google"}]}),
        ),
        (
            "ollama-native",
            "/proxy/api/",
            "/proxy/api/tags",
            json!({"models":[{"model":"raw/ollama"}]}),
        ),
        (
            "ollama-native",
            "/proxy/api/tags/",
            "/proxy/api/tags",
            json!({"models":[{"model":"raw/ollama"}]}),
        ),
    ] {
        let fixture = fixture(move |uri, headers| {
            assert_fixture_auth(protocol, headers);
            assert_eq!(uri.path(), expected);
            Json(body.clone()).into_response()
        })
        .await;
        let result = preview(request(
            fixture.port,
            protocol,
            base,
            Some("fixture-secret"),
        ))
        .await
        .unwrap();
        assert_eq!(result.models.len(), 1);
        assert_eq!(fixture.paths(), [expected]);
    }
}

#[tokio::test]
async fn path_errors_try_only_the_same_origin_compatibility_path_and_keep_proxy_prefixes() {
    for status in [
        StatusCode::NOT_FOUND,
        StatusCode::METHOD_NOT_ALLOWED,
        StatusCode::GONE,
    ] {
        for (protocol, base, first, second) in [
            ("openai-chat", "", "/v1/models", "/models"),
            ("openai-responses", "/models", "/models", "/v1/models"),
            ("anthropic", "/proxy", "/proxy/models", "/proxy/v1/models"),
            (
                "anthropic",
                "/proxy/v1/",
                "/proxy/v1/models",
                "/proxy/models",
            ),
            (
                "google",
                "/proxy/v1beta/models",
                "/proxy/v1beta/models",
                "/proxy/models",
            ),
        ] {
            let fixture = fixture(move |uri, headers| {
                assert_fixture_auth(protocol, headers);
                if uri.path() == first {
                    (status, "fixture-secret echoed by upstream").into_response()
                } else if uri.path() == second {
                    Json(json!({"data":[{"id":"verified-after-fallback"}]})).into_response()
                } else {
                    panic!("unexpected path: {uri}");
                }
            })
            .await;
            let result = readonly(request(
                fixture.port,
                protocol,
                base,
                Some("fixture-secret"),
            ))
            .await
            .unwrap();
            assert_eq!(result.models[0].id, "verified-after-fallback");
            assert!(result.credential_ref.is_none());
            assert_eq!(fixture.paths(), [first, second]);
        }
    }
}

#[tokio::test]
async fn html_homepage_detection_handles_mime_types_body_sniffing_and_bom() {
    for (content_type, body) in [
        ("text/html; charset=utf-8", "<html>fixture-secret</html>"),
        ("application/xhtml+xml", "<?xml version='1.0'?><html/>"),
        ("text/plain", "\u{feff} \r\n<!DoCtYpE HTML><html/>"),
        ("application/json", " \n<HTML>fixture-secret</HTML>"),
    ] {
        let fixture = fixture(move |uri, headers| {
            assert_fixture_auth("anthropic", headers);
            match uri.path() {
                "/proxy/models" => ([("content-type", content_type)], body).into_response(),
                "/proxy/v1/models" => Json(json!({"data":[{"id":"actual-list"}]})).into_response(),
                _ => panic!("proxy prefix was lost: {uri}"),
            }
        })
        .await;
        let result = preview(request(
            fixture.port,
            "anthropic",
            "/proxy/models",
            Some("fixture-secret"),
        ))
        .await
        .unwrap();
        assert_eq!(result.models[0].id, "actual-list");
        assert_eq!(fixture.paths(), ["/proxy/models", "/proxy/v1/models"]);
    }
}

#[tokio::test]
async fn exhausted_html_and_missing_paths_are_specific_errors_with_a_two_request_limit() {
    for (status, body, expected) in [
        (
            StatusCode::OK,
            "<html>fixture-secret</html>",
            "discovery_html",
        ),
        (StatusCode::NOT_FOUND, "fixture-secret", "discovery_path"),
    ] {
        let fixture = fixture(move |_, _| (status, body).into_response()).await;
        let error = preview(request(
            fixture.port,
            "openai-chat",
            "",
            Some("fixture-secret"),
        ))
        .await
        .unwrap_err();
        assert_eq!(error.code, expected);
        assert!(!serde_json::to_string(&error)
            .unwrap()
            .contains("fixture-secret"));
        assert_eq!(fixture.paths(), ["/v1/models", "/models"]);
    }
}

#[tokio::test]
async fn authentication_rate_limits_and_server_errors_do_not_probe_other_paths() {
    for (status, expected) in [
        (StatusCode::BAD_REQUEST, "discovery_failed"),
        (StatusCode::UNAUTHORIZED, "discovery_auth"),
        (StatusCode::FORBIDDEN, "discovery_auth"),
        (StatusCode::CONFLICT, "discovery_failed"),
        (StatusCode::TOO_MANY_REQUESTS, "discovery_failed"),
        (StatusCode::INTERNAL_SERVER_ERROR, "discovery_failed"),
        (StatusCode::SERVICE_UNAVAILABLE, "discovery_failed"),
    ] {
        let fixture =
            fixture(move |_, _| (status, "<html>fixture-secret</html>").into_response()).await;
        let error = readonly(request(
            fixture.port,
            "anthropic",
            "",
            Some("fixture-secret"),
        ))
        .await
        .unwrap_err();
        assert_eq!(error.code, expected);
        assert!(error.message.contains(&status.as_u16().to_string()));
        assert!(!error.message.contains("fixture-secret"));
        assert_eq!(fixture.paths(), ["/v1/models"]);
    }
}

#[tokio::test]
async fn redirects_never_forward_credentials_to_another_origin() {
    let destination =
        fixture(|_, _| panic!("redirect destination must never receive a request")).await;
    let location = format!("http://127.0.0.1:{}/v1/models", destination.port);
    let source =
        fixture(move |_, _| (StatusCode::FOUND, [("location", location.clone())]).into_response())
            .await;
    let error = preview(request(
        source.port,
        "openai-chat",
        "",
        Some("fixture-secret"),
    ))
    .await
    .unwrap_err();
    assert_eq!(error.code, "discovery_failed");
    assert!(error.message.contains("302"));
    assert_eq!(source.paths(), ["/v1/models"]);
    assert!(destination.paths().is_empty());
}

#[tokio::test]
async fn invalid_json_and_invalid_lists_are_not_success_and_do_not_trigger_path_probing() {
    for (body, expected) in [
        ("fixture-secret invalid json", "discovery_json"),
        ("{}", "discovery_shape"),
        (
            r#"{"error":{"message":"fixture-secret"}}"#,
            "discovery_shape",
        ),
        (r#"{"data":[],"error":"fixture-secret"}"#, "discovery_shape"),
        (r#"{"data":{}}"#, "discovery_shape"),
        (r#"{"data":[{}]}"#, "discovery_shape"),
        (r#"{"data":[{"id":""}]}"#, "discovery_identity"),
        (r#"{"data":[{"id":"   "}]}"#, "discovery_identity"),
    ] {
        let fixture =
            fixture(move |_, _| ([("content-type", "application/json")], body).into_response())
                .await;
        let error = preview(request(
            fixture.port,
            "openai-responses",
            "",
            Some("fixture-secret"),
        ))
        .await
        .unwrap_err();
        assert_eq!(error.code, expected);
        assert!(!serde_json::to_string(&error)
            .unwrap()
            .contains("fixture-secret"));
        assert_eq!(fixture.paths(), ["/v1/models"]);
    }
}

#[tokio::test]
async fn valid_json_lists_remain_valid_even_if_the_upstream_mislabels_the_content_type() {
    for body in [r#"{"data":[]}"#, r#"{"data":[{"id":"real-model"}]}"#] {
        let fixture =
            fixture(move |_, _| ([("content-type", "text/html")], body).into_response()).await;
        let result = preview(request(fixture.port, "openai-chat", "", None))
            .await
            .unwrap();
        assert_eq!(
            result.models.len(),
            usize::from(body.contains("real-model"))
        );
        assert_eq!(fixture.paths(), ["/v1/models"]);
    }
}

#[tokio::test]
async fn pagination_after_fallback_stays_on_the_selected_endpoint_and_keeps_auth() {
    for protocol in ["openai-chat", "openai-responses", "anthropic", "google"] {
        let first = if protocol == "google" {
            "/v1beta/models"
        } else {
            "/v1/models"
        };
        let cursor = if protocol == "google" {
            "pageToken"
        } else if protocol == "anthropic" {
            "after_id"
        } else {
            "after"
        };
        let fixture = fixture(move |uri, headers| {
            assert_fixture_auth(protocol, headers);
            if uri.path() == first {
                return StatusCode::NOT_FOUND.into_response();
            }
            assert_eq!(uri.path(), "/models");
            let second = uri.query().is_some();
            let mut body = if protocol == "google" {
                json!({"models":[{"name":if second {"models/second"} else {"models/first"}}]})
            } else {
                json!({"data":[{"id":if second {"second"} else {"first"}}]})
            };
            if !second {
                if protocol == "google" {
                    body["nextPageToken"] = json!("cursor-2");
                } else {
                    body["has_more"] = json!(true);
                    body["last_id"] = json!("cursor-2");
                }
            }
            Json(body).into_response()
        })
        .await;
        let result = readonly(request(fixture.port, protocol, "", Some("fixture-secret")))
            .await
            .unwrap();
        assert_eq!(
            result
                .models
                .iter()
                .map(|model| model.id.as_str())
                .collect::<Vec<_>>(),
            ["first", "second"]
        );
        assert!(result.credential_ref.is_none());
        assert_eq!(
            fixture.paths(),
            [
                first.to_owned(),
                "/models".into(),
                format!("/models?{cursor}=cursor-2")
            ]
        );
    }
}

#[tokio::test]
async fn a_later_page_path_error_does_not_fall_back_or_report_a_partial_success() {
    for (status, expected) in [
        (StatusCode::OK, "discovery_html"),
        (StatusCode::NOT_FOUND, "discovery_path"),
    ] {
        let fixture = fixture(move |uri, _| {
            assert_eq!(uri.path(), "/v1/models");
            if uri.query().is_none() {
                Json(json!({"data":[{"id":"partial"}],"has_more":true,"last_id":"cursor-2"}))
                    .into_response()
            } else {
                (status, "<html>not a second page</html>").into_response()
            }
        })
        .await;
        let error = preview(request(fixture.port, "anthropic", "", None))
            .await
            .unwrap_err();
        assert_eq!(error.code, expected);
        assert_eq!(
            fixture.paths(),
            ["/v1/models", "/v1/models?after_id=cursor-2"]
        );
    }
}

#[tokio::test]
async fn connection_refusal_and_tls_failures_do_not_try_a_compatibility_connection() {
    use tokio::io::AsyncWriteExt;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let error = preview(request(port, "openai-chat", "", None))
        .await
        .unwrap_err();
    assert_eq!(error.code, "discovery_failed");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let observed = calls.clone();
    let server = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            observed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let _ = stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                .await;
            let _ = stream.shutdown().await;
        }
    });
    let mut input = request(port, "anthropic", "", Some("fixture-secret"));
    input.url = format!("https://127.0.0.1:{port}");
    let error = preview(input).await.unwrap_err();
    server.abort();
    assert_eq!(error.code, "discovery_failed");
    assert!(!error.message.contains("fixture-secret"));
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[tokio::test]
async fn timeout_ends_endpoint_selection_without_trying_the_next_path() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let observed = calls.clone();
    let app = Router::new().fallback(get(move |uri: Uri| {
        observed.lock().unwrap().push(uri.to_string());
        std::future::pending::<Response>()
    }));
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_millis(100))
        .build()
        .unwrap();
    let base = reqwest::Url::parse(&format!("http://127.0.0.1:{port}")).unwrap();
    let error = read_first_page(&client, &base, "openai-chat", Some("fixture-secret"))
        .await
        .unwrap_err();
    server.abort();
    assert_eq!(error.code, "discovery_timeout");
    assert_eq!(*calls.lock().unwrap(), ["/v1/models"]);
}

#[tokio::test]
async fn canceling_measurement_during_fallback_drops_the_request_and_keeps_temporary_keys_in_memory(
) {
    use crate::modules::model_config::measurements;
    let home = tempfile::tempdir().unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let observed = calls.clone();
    let waiting = Arc::new(tokio::sync::Notify::new());
    let reached = waiting.clone();
    let app = Router::new().fallback(get(move |uri: Uri, headers: HeaderMap| {
        let reached = reached.clone();
        let observed = observed.clone();
        async move {
            assert_fixture_auth("anthropic", &headers);
            observed.lock().unwrap().push(uri.to_string());
            if uri.path() == "/v1/models" {
                return axum::response::Html("<html>homepage</html>").into_response();
            }
            assert_eq!(uri.path(), "/models");
            reached.notify_one();
            std::future::pending::<Response>().await
        }
    }));
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let run_id = uuid::Uuid::new_v4().to_string();
    let operation = tokio::spawn(measurements::measure(measurements::Request {
        run_id: run_id.clone(),
        protocol: "anthropic".into(),
        nodes: vec![ServiceNode {
            id: uuid::Uuid::new_v4().to_string(),
            name: "fixture".into(),
            url: format!("http://127.0.0.1:{port}"),
        }],
        api_key: Some("fixture-secret".into()),
        credential_ref: None,
        allow_private_network: true,
        provider_id: None,
        home: Some(home.path().to_owned()),
    }));
    tokio::time::timeout(Duration::from_secs(2), waiting.notified())
        .await
        .unwrap();
    measurements::stop(&run_id).unwrap();
    let result = tokio::time::timeout(Duration::from_secs(2), operation)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    server.abort();
    assert!(result.stopped);
    assert!(result.results.is_empty());
    assert_eq!(*calls.lock().unwrap(), ["/v1/models", "/models"]);
    assert_eq!(std::fs::read_dir(home.path()).unwrap().count(), 0);
}
