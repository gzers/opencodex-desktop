//! WP-76 exercises the real HTTP adapter against an attested loopback runtime.
//! All records, credentials, provider/cache sentinels and CLI stubs are temporary.
use super::*;
use axum::{
    body::Bytes,
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::Sha256;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
};

#[derive(Clone)]
struct RuntimeFixture {
    pid: u32,
    port: u16,
    secret: String,
    root: PathBuf,
    status: StatusCode,
    body: Vec<u8>,
    valid_proof: Arc<AtomicBool>,
    change_binding: Arc<AtomicBool>,
    health_calls: Arc<AtomicUsize>,
    restart_calls: Arc<AtomicUsize>,
    unexpected_calls: Arc<AtomicUsize>,
}

async fn health(
    State(state): State<RuntimeFixture>,
    headers: HeaderMap,
) -> (HeaderMap, Json<Value>) {
    assert!(headers.get(header::AUTHORIZATION).is_none());
    state.health_calls.fetch_add(1, Ordering::SeqCst);
    let challenge = headers["x-opencodex-attestation-challenge"]
        .to_str()
        .unwrap();
    let secret = if state.valid_proof.load(Ordering::SeqCst) {
        state.secret.as_bytes()
    } else {
        b"untrusted-listener"
    };
    let mut mac = Hmac::<Sha256>::new_from_slice(secret).unwrap();
    mac.update(
        format!(
            "opencodex-local-management-v1\n{challenge}\n{}\n{}",
            state.pid, state.port
        )
        .as_bytes(),
    );
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-opencodex-attestation-proof",
        URL_SAFE_NO_PAD
            .encode(mac.finalize().into_bytes())
            .parse()
            .unwrap(),
    );
    (
        headers,
        Json(json!({"service":"opencodex","pid":state.pid})),
    )
}

fn write_record(state: &RuntimeFixture, pid: u32) {
    crate::infrastructure::atomic_write::atomic_write(
        &state.root.join("runtime-port.json"),
        &serde_json::to_vec(&json!({
            "pid":pid,"port":state.port,"attestationSecret":state.secret
        }))
        .unwrap(),
        0o600,
    )
    .unwrap();
}

async fn restart(State(state): State<RuntimeFixture>, headers: HeaderMap, body: Bytes) -> Response {
    assert_eq!(
        headers[header::AUTHORIZATION],
        format!("Bearer ocx_admin_{}", "t".repeat(43))
    );
    assert_eq!(headers[header::CONTENT_LENGTH], "0");
    assert!(headers.get(header::TRANSFER_ENCODING).is_none());
    assert!(
        body.is_empty(),
        "restart must not forward provider/configuration data"
    );
    state.restart_calls.fetch_add(1, Ordering::SeqCst);
    if state.change_binding.load(Ordering::SeqCst) {
        write_record(&state, state.pid + 1);
    }
    let mut response = (state.status, state.body).into_response();
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
    response
        .headers_mut()
        .insert(header::LOCATION, "/unexpected".parse().unwrap());
    response
}

async fn unexpected(State(state): State<RuntimeFixture>) -> StatusCode {
    state.unexpected_calls.fetch_add(1, Ordering::SeqCst);
    StatusCode::NOT_FOUND
}

struct Fixture {
    root: tempfile::TempDir,
    context: ProcessContext,
    state: RuntimeFixture,
    server: tokio::task::JoinHandle<()>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}

impl Fixture {
    async fn new(body: Value) -> Self {
        Self::raw(StatusCode::OK, serde_json::to_vec(&body).unwrap()).await
    }

    async fn raw(status: StatusCode, body: Vec<u8>) -> Self {
        let root = tempfile::tempdir().unwrap();
        let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let state = RuntimeFixture {
            pid: std::process::id(),
            port: listener.local_addr().unwrap().port(),
            secret: "s".repeat(43),
            root: root.path().into(),
            status,
            body,
            valid_proof: Arc::new(AtomicBool::new(true)),
            change_binding: Arc::new(AtomicBool::new(false)),
            health_calls: Arc::new(AtomicUsize::new(0)),
            restart_calls: Arc::new(AtomicUsize::new(0)),
            unexpected_calls: Arc::new(AtomicUsize::new(0)),
        };
        write_record(&state, state.pid);
        crate::infrastructure::atomic_write::atomic_write(
            &root.path().join("admin-api-token"),
            format!("ocx_admin_{}", "t".repeat(43)).as_bytes(),
            0o600,
        )
        .unwrap();
        std::fs::write(root.path().join("config.json"), b"provider-sentinel").unwrap();
        std::fs::write(root.path().join("models_cache.json"), b"cache-sentinel").unwrap();
        let context = ProcessContext {
            // An explicit restart must work without resolving or spawning a CLI.
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(
            ),
            working_directory: root.path().into(),
            opencodex_home: root.path().into(),
        };
        let router = Router::new()
            .route("/healthz", get(health))
            .route("/api/system/codex-restart", post(restart))
            .fallback(unexpected)
            .with_state(state.clone());
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        Self {
            root,
            context,
            state,
            server,
        }
    }

    fn assert_calls(&self, expected: usize) {
        assert_eq!(self.state.restart_calls.load(Ordering::SeqCst), expected);
        assert_eq!(self.state.unexpected_calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            std::fs::read(self.root.path().join("config.json")).unwrap(),
            b"provider-sentinel"
        );
        assert_eq!(
            std::fs::read(self.root.path().join("models_cache.json")).unwrap(),
            b"cache-sentinel"
        );
    }
}

fn completed_desktop() -> Value {
    json!({"attempted":true,"stopped":[301],"surviving":[],"relaunch":"started"})
}

fn response(desktop: Value) -> Value {
    json!({
        "success":true,"stateBefore":"stale","synced":true,
        "requested":[101],"stopped":[101],"surviving":[],"failed":[],
        "code":"stopped","desktopApp":desktop
    })
}

fn no_app_servers(desktop: Value) -> Value {
    json!({
        "success":true,"stateBefore":"not_running","synced":false,
        "requested":[],"stopped":[],"surviving":[],"failed":[],
        "code":"nothing_running","desktopApp":desktop
    })
}

fn assert_incomplete(result: &CacheRefresh) {
    assert!(!result.success, "{result:?}");
    assert!(!result.restarted, "{result:?}");
    assert!(!result.summary.contains("已重启"));
    assert!(!result.summary.contains("已重建"));
}

#[tokio::test]
async fn explicit_restart_posts_even_when_cache_is_unchanged_and_needs_no_cli() {
    for synced in [true, false] {
        let mut body = response(completed_desktop());
        body["synced"] = json!(synced);
        let fixture = Fixture::new(body).await;
        let result = sync_cache(&fixture.context, true).await.unwrap();
        assert!(result.success && result.restarted);
        assert!(result.summary.contains("Codex 已重启"));
        assert_eq!(result.summary.contains("缓存已刷新"), synced);
        assert_eq!(result.reason, None);
        assert_eq!(fixture.state.health_calls.load(Ordering::SeqCst), 1);
        fixture.assert_calls(1);
        let dto = serde_json::to_value(result).unwrap();
        assert_eq!(dto.as_object().unwrap().len(), 3);
        assert_eq!(dto["success"], true);
        assert_eq!(dto["restarted"], true);
        assert!(dto["summary"].is_string());
    }
}

#[tokio::test]
async fn no_app_servers_does_not_prevent_a_verified_desktop_restart() {
    let fixture = Fixture::new(no_app_servers(completed_desktop())).await;
    let result = sync_cache(&fixture.context, true).await.unwrap();
    assert!(result.success && result.restarted);
    fixture.assert_calls(1);
}

#[tokio::test]
async fn desktop_refused_blocked_not_found_and_other_skips_are_not_restarts() {
    for reason in [
        "self_ancestry",
        "restart_in_flight",
        "package_discovery_failed",
        "no_targets",
        "process_probe_failed",
        "unsupported_platform",
        "test_environment",
        "handoff_started",
    ] {
        let fixture = Fixture::new(response(json!({
            "attempted":false,"stopped":[],"surviving":[],"relaunch":"skipped","reason":reason
        })))
        .await;
        let result = sync_cache(&fixture.context, true).await.unwrap();
        assert_incomplete(&result);
        assert_eq!(result.reason.as_deref(), Some(reason));
        fixture.assert_calls(1);
    }
}

#[tokio::test]
async fn stopped_some_desktop_processes_or_failed_relaunch_is_incomplete() {
    for desktop in [
        json!({"attempted":true,"stopped":[301],"surviving":[302],"relaunch":"skipped","reason":"targets_survived"}),
        json!({"attempted":true,"stopped":[301],"surviving":[],"relaunch":"skipped","reason":"relaunch_failed"}),
    ] {
        let expected = desktop["reason"].as_str().unwrap().to_owned();
        let fixture = Fixture::new(response(desktop)).await;
        let result = sync_cache(&fixture.context, true).await.unwrap();
        assert_incomplete(&result);
        assert_eq!(result.reason.as_deref(), Some(expected.as_str()));
        fixture.assert_calls(1);
    }
}

#[tokio::test]
async fn top_level_success_with_nothing_running_or_missing_desktop_never_claims_restart() {
    let mut missing = no_app_servers(completed_desktop());
    missing.as_object_mut().unwrap().remove("desktopApp");
    for body in [
        missing,
        no_app_servers(json!({
            "attempted":false,"stopped":[],"surviving":[],"relaunch":"skipped","reason":"no_targets"
        })),
    ] {
        let fixture = Fixture::new(body).await;
        let result = sync_cache(&fixture.context, true).await.unwrap();
        assert_incomplete(&result);
        fixture.assert_calls(1);
    }
}

#[tokio::test]
async fn app_server_partial_failure_prevents_overall_restart_success() {
    for field in ["surviving", "failed"] {
        let mut body = response(completed_desktop());
        body["success"] = json!(false);
        body["code"] = json!("partially_stopped");
        body[field] = json!([102]);
        let fixture = Fixture::new(body).await;
        let result = sync_cache(&fixture.context, true).await.unwrap();
        assert_incomplete(&result);
        assert_eq!(result.reason.as_deref(), Some("partially_stopped"));
        fixture.assert_calls(1);
    }
}

#[tokio::test]
async fn enumeration_failure_is_incomplete_even_with_top_level_success_and_desktop_relaunch() {
    let mut body = no_app_servers(completed_desktop());
    body["stateBefore"] = json!("unknown");
    body["code"] = json!("enumeration_unavailable");
    let fixture = Fixture::new(body).await;
    let result = sync_cache(&fixture.context, true).await.unwrap();
    assert_incomplete(&result);
    assert_eq!(result.reason.as_deref(), Some("enumeration_unavailable"));
    fixture.assert_calls(1);
}

#[tokio::test]
async fn malformed_or_contradictory_http_200_responses_are_rejected() {
    let mut bodies = vec![
        json!({"success":true}),
        json!({"desktopAppRestarted":true}),
        json!(null),
    ];
    for (pointer, value) in [
        ("/desktopApp/attempted", json!(false)),
        ("/desktopApp/attempted", json!("true")),
        ("/desktopApp/stopped", json!([])),
        ("/desktopApp/stopped", json!([0])),
        ("/desktopApp/stopped", json!([-1])),
        ("/desktopApp/stopped", json!([1.5])),
        ("/desktopApp/stopped", json!([9_007_199_254_740_992_u64])),
        ("/desktopApp/surviving", json!([302])),
        ("/desktopApp/relaunch", json!("queued")),
        ("/desktopApp/reason", json!("self_ancestry")),
        ("/desktopApp/reason", json!({"private":"response-secret"})),
        ("/success", json!(false)),
        ("/code", json!("partially_stopped")),
        ("/code", json!("nothing_running")),
        ("/code", json!("enumeration_unavailable")),
        ("/code", json!("unknown_code")),
        ("/stateBefore", json!("unknown_state")),
        ("/surviving", json!([102])),
        ("/failed", json!([102])),
        ("/requested", json!([0])),
    ] {
        let mut body = response(completed_desktop());
        if pointer == "/desktopApp/reason" {
            body["desktopApp"]["reason"] = value;
        } else {
            *body
                .pointer_mut(pointer)
                .unwrap_or_else(|| panic!("missing fixture field {pointer}")) = value;
        }
        bodies.push(body);
    }
    for body in bodies {
        let fixture = Fixture::new(body).await;
        let error = sync_cache(&fixture.context, true).await.unwrap_err();
        assert_eq!(error.code, "cache_restart_response");
        assert!(!error.message.contains("已重启"));
        assert!(!error.message.contains("response-secret"));
        fixture.assert_calls(1);
    }
}

#[tokio::test]
async fn unknown_or_missing_desktop_reason_has_a_fixed_redacted_fallback() {
    for reason in [Some("/private/response-secret/username"), None] {
        let mut desktop =
            json!({"attempted":false,"stopped":[],"surviving":[],"relaunch":"skipped"});
        if let Some(reason) = reason {
            desktop["reason"] = json!(reason);
        }
        let fixture = Fixture::new(response(desktop)).await;
        let result = sync_cache(&fixture.context, true).await.unwrap();
        assert_incomplete(&result);
        assert_eq!(result.reason.as_deref(), Some("desktop_restart_incomplete"));
        assert!(!serde_json::to_string(&result)
            .unwrap()
            .contains("response-secret"));
        fixture.assert_calls(1);
    }
}

#[tokio::test]
async fn http_errors_and_redirects_never_accept_even_a_successful_body_or_echo_error_text() {
    for status in [301, 302, 401, 403, 404, 409, 500, 503] {
        let mut body = response(completed_desktop());
        body["error"] = json!("/private/response-secret/username");
        let fixture = Fixture::raw(
            StatusCode::from_u16(status).unwrap(),
            serde_json::to_vec(&body).unwrap(),
        )
        .await;
        let error = sync_cache(&fixture.context, true).await.unwrap_err();
        assert_eq!(error.code, "management_rejected");
        assert!(error.message.contains(&format!("HTTP{status}")));
        assert!(!error.message.contains("response-secret"));
        assert!(!error.message.contains("已重启"));
        fixture.assert_calls(1);
    }
}

#[tokio::test]
async fn invalid_json_and_empty_http_200_are_errors() {
    for body in [Vec::new(), b"not-json-response-secret".to_vec()] {
        let fixture = Fixture::raw(StatusCode::OK, body).await;
        let error = sync_cache(&fixture.context, true).await.unwrap_err();
        assert_eq!(error.code, "invalid_document");
        assert!(!error.message.contains("response-secret"));
        fixture.assert_calls(1);
    }
}

#[tokio::test]
async fn untrusted_runtime_proof_never_reaches_restart_or_reads_admin_token() {
    let fixture = Fixture::new(response(completed_desktop())).await;
    fixture.state.valid_proof.store(false, Ordering::SeqCst);
    std::fs::remove_file(fixture.root.path().join("admin-api-token")).unwrap();
    let error = sync_cache(&fixture.context, true).await.unwrap_err();
    assert_eq!(error.code, "runtime_attestation");
    fixture.assert_calls(0);
}

#[tokio::test]
async fn dedicated_restart_checks_runtime_binding_and_does_not_expand_generic_request_scope() {
    let fixture = Fixture::new(response(completed_desktop())).await;
    let manager = Management::bind(&fixture.context).await.unwrap();
    let error = manager
        .request(reqwest::Method::POST, "/api/system/codex-restart", None)
        .await
        .unwrap_err();
    assert_eq!(error.code, "management_scope");
    write_record(&fixture.state, fixture.state.pid + 1);
    let error = manager.restart_codex().await.unwrap_err();
    assert_eq!(error.code, "stale_runtime");
    fixture.assert_calls(0);
}

#[tokio::test]
async fn changed_runtime_after_response_is_not_reported_as_a_restart() {
    let fixture = Fixture::new(response(completed_desktop())).await;
    fixture.state.change_binding.store(true, Ordering::SeqCst);
    let error = sync_cache(&fixture.context, true).await.unwrap_err();
    assert_eq!(error.code, "stale_runtime");
    fixture.assert_calls(1);
}

#[tokio::test]
async fn malformed_admin_token_prevents_restart() {
    let fixture = Fixture::new(response(completed_desktop())).await;
    std::fs::write(
        fixture.root.path().join("admin-api-token"),
        b"invalid-token",
    )
    .unwrap();
    let error = sync_cache(&fixture.context, true).await.unwrap_err();
    assert_eq!(error.code, "management_auth");
    fixture.assert_calls(0);
}

#[cfg(any(unix, windows))]
fn fake_cli(root: &Path, exit_code: u8) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let path = root.join("fake-ocx");
        std::fs::write(&path, format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$OPENCODEX_HOME/cli-args\"\nprintf '%s\\n' 'stdout-response-secret'\nprintf '%s\\n' 'stderr-response-secret' >&2\nexit {exit_code}\n"
        )).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        path
    }
    #[cfg(windows)]
    {
        let path = root.join("fake-ocx.cmd");
        std::fs::write(&path, format!(
            "@echo off\r\n>\"%OPENCODEX_HOME%\\cli-args\" echo %*\r\necho stdout-response-secret\r\necho stderr-response-secret 1>&2\r\nexit /b {exit_code}\r\n"
        )).unwrap();
        path
    }
}

#[cfg(any(unix, windows))]
#[tokio::test]
async fn cache_only_executes_cli_without_restart_or_management_binding_and_redacts_output() {
    for exit_code in [0, 1] {
        let mut fixture = Fixture::new(response(completed_desktop())).await;
        fixture.context.runtime =
            crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(fake_cli(
                fixture.root.path(),
                exit_code,
            ));
        std::fs::remove_file(fixture.root.path().join("runtime-port.json")).unwrap();
        std::fs::remove_file(fixture.root.path().join("admin-api-token")).unwrap();
        let result = sync_cache(&fixture.context, false).await.unwrap();
        assert_eq!(result.success, exit_code == 0);
        assert!(!result.restarted);
        assert!(!result.summary.contains("已重启"));
        assert!(!result.summary.contains("response-secret"));
        assert_eq!(
            std::fs::read_to_string(fixture.root.path().join("cli-args"))
                .unwrap()
                .trim(),
            "sync-cache"
        );
        assert_eq!(fixture.state.health_calls.load(Ordering::SeqCst), 0);
        fixture.assert_calls(0);
    }
}

#[test]
fn warning_states_that_live_conversations_end() {
    assert!(RESTART_WARNING.contains("对话"));
}
