//! Real loopback HTTP vectors for the supported read-only discovery contract.
use super::*;
use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
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
