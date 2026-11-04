//! Safe restart-commit guard on an isolated COPY, not a real desktop restart.
//! Calls the production sync_cache_after_save with synthetic attested HTTP IO.
//! The original, actually prepared pending intent remains unacknowledged.
use axum::{body::Bytes, extract::State, http::{HeaderMap, Method, StatusCode, Uri}, response::{IntoResponse, Response}, Json, Router};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use opencodex_desktop_lib::{infrastructure::runtime_executable::FixedRuntimeExecutable,
    modules::model_config::cache_refresh, state::ProcessContext};
use serde_json::{json, Value};
use sha2::Sha256;
use std::{path::{Path, PathBuf}, sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}}};

const PENDING: &str = "manager-state/provider-client-renames.json";
#[derive(Clone)]
struct MockOwner {
    pid: u32, port: u16, secret: String, public: Value, simulated_home: PathBuf,
    allow_commit: Arc<AtomicBool>, restart_calls: Arc<AtomicUsize>,
}

async fn handler(State(owner): State<MockOwner>, method: Method, uri: Uri, headers: HeaderMap, body: Bytes) -> Response {
    if method == Method::GET && uri.path() == "/healthz" {
        assert!(headers.get("authorization").is_none());
        let challenge = headers["x-opencodex-attestation-challenge"].to_str().unwrap();
        let mut mac = Hmac::<Sha256>::new_from_slice(owner.secret.as_bytes()).unwrap();
        mac.update(format!("opencodex-local-management-v1\n{challenge}\n{}\n{}",owner.pid,owner.port).as_bytes());
        let mut output = HeaderMap::new();
        output.insert("x-opencodex-attestation-proof", URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes()).parse().unwrap());
        return (output, Json(json!({"service":"opencodex","pid":owner.pid,"port":owner.port}))).into_response();
    }
    if method == Method::POST && uri.path() == "/api/providers/reload" {
        assert!(headers.get("authorization").is_none());
        assert!(body.is_empty());
        assert!(headers.get("x-opencodex-provider-reload-capability").is_some());
        return Json(json!({"success":true})).into_response();
    }
    assert_eq!(headers.get("authorization").unwrap().to_str().unwrap(), format!("Bearer ocx_admin_{}", "t".repeat(43)));
    match (method, uri.path()) {
        (Method::GET, "/api/settings") => Json(json!({"codexRuntime":{}})).into_response(),
        (Method::GET, "/api/client-integrations/codex" | "/api/native-integrations/codex") => {
            // Binding deliberately absent only in this commit-only transport
            // fixture. The real original prepare already exercised its owner.
            Json(json!({"clientId":"codex","installed":false,"state":"not_installed",
                "configPath":owner.simulated_home.join("no-client/config.toml")})).into_response()
        },
        (Method::GET, "/api/native-integrations") => Json(json!({"clients":[{
            "clientId":"codex","installed":false,"state":"absent",
            "configPath":owner.simulated_home.join("no-client/config.toml")
        }]})).into_response(),
        (Method::GET, "/api/config") => Json(owner.public).into_response(),
        (Method::PUT, "/api/providers") => {
            let request: Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(request["baseline"],request["next"]);
            assert_eq!(request["next"]["defaultProvider"], owner.public["defaultProvider"]);
            Json(json!({"success":true,"catalogRefresh":{"status":"committed","changed":false,
                "degraded":false,"notices":[]}})).into_response()
        },
        (Method::POST, "/api/system/codex-restart") => {
            assert!(body.is_empty());
            assert_eq!(headers["content-length"],"0");
            owner.restart_calls.fetch_add(1,Ordering::SeqCst);
            let desktop = if owner.allow_commit.load(Ordering::SeqCst) {
                // These are synthetic response IDs only. No process is queried
                // or signalled, and no GUI is launched by this HTTP handler.
                json!({"attempted":true,"stopped":[301],"surviving":[],"relaunch":"started"})
            } else {
                json!({"attempted":false,"stopped":[],"surviving":[],"relaunch":"skipped","reason":"test_environment"})
            };
            Json(json!({"success":true,"stateBefore":"not_running","synced":false,
                "requested":[],"stopped":[],"surviving":[],"failed":[],"code":"nothing_running",
                "desktopApp":desktop})).into_response()
        },
        _ => (StatusCode::NOT_FOUND, Json(json!({"error":"unexpected simulation route"}))).into_response(),
    }
}

fn clone_tree(source: &Path, destination: &Path) -> Result<(), String> {
    std::fs::create_dir_all(destination).map_err(|e|e.to_string())?;
    for entry in std::fs::read_dir(source).map_err(|e|e.to_string())? {
        let entry = entry.map_err(|e|e.to_string())?;
        let kind = entry.file_type().map_err(|e|e.to_string())?;
        if kind.is_symlink() { return Err("simulation refuses symlinked manager state".into()); }
        if kind.is_dir() { clone_tree(&entry.path(),&destination.join(entry.file_name()))?; }
        else if kind.is_file() { std::fs::copy(entry.path(),destination.join(entry.file_name())).map_err(|e|e.to_string())?; }
    }
    Ok(())
}

pub async fn run(container: &Path, manager: &Path, home: &Path, pending_id: &str) -> Value {
    match inner(container,manager,home,pending_id).await {
        Ok(value) => value,
        Err(error) => json!({"simulationPassed":false,"error":error,"realGlobalRestartCalled":false}),
    }
}

async fn inner(container: &Path, manager: &Path, home: &Path, pending_id: &str) -> Result<Value,String> {
    let original = std::fs::read(manager.join(PENDING)).map_err(|e|e.to_string())?;
    let mut intent: Value = serde_json::from_slice(&original).map_err(|e|e.to_string())?;
    if intent["id"].as_str() != Some(pending_id) { return Err("prepared pending identity mismatch".into()); }
    let fixture = container.join("safe-restart-commit-copy");
    if fixture.exists() { return Err("commit fixture already exists; refusing reuse".into()); }
    let copy_root = fixture.join("manager");
    let copy_home = fixture.join("official");
    clone_tree(manager,&copy_root)?;
    std::fs::create_dir_all(&copy_home).map_err(|e|e.to_string())?;
    std::fs::copy(home.join("config.json"),copy_home.join("config.json")).map_err(|e|e.to_string())?;
    // A separate synthetic identity is required because the pending copy is
    // bound to a different official home. Preserve the actual prepared ID/map.
    intent["homeBinding"] = json!(opencodex_desktop_lib::modules::model_config::store::sha256(
        &serde_json::to_vec(&copy_home).unwrap(),
    ));
    std::fs::write(copy_root.join(PENDING),serde_json::to_vec(&intent).unwrap()).map_err(|e|e.to_string())?;
    let public: Value = serde_json::from_slice(&std::fs::read(container.join("safe-public-state-for-commit.json")).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST,0)).await.map_err(|e|e.to_string())?;
    let owner = MockOwner { pid:std::process::id(), port:listener.local_addr().unwrap().port(), secret:"s".repeat(43),
        public, simulated_home:copy_home.clone(),allow_commit:Arc::new(AtomicBool::new(false)),restart_calls:Arc::new(AtomicUsize::new(0)) };
    std::fs::write(copy_home.join("runtime-port.json"),serde_json::to_vec(&json!({"pid":owner.pid,"port":owner.port,"attestationSecret":owner.secret})).unwrap()).map_err(|e|e.to_string())?;
    std::fs::write(copy_home.join("admin-api-token"),format!("ocx_admin_{}","t".repeat(43))).map_err(|e|e.to_string())?;
    let service = Router::new().fallback(handler).with_state(owner.clone());
    let server = tokio::spawn(async move { axum::serve(listener,service).await.unwrap(); });
    let context = ProcessContext { runtime:FixedRuntimeExecutable::resolved(container.join("ocx")),
        working_directory:container.join("work"),opencodex_home:copy_home };
    let result: Result<Value,String> = async {
        let skipped = cache_refresh::sync_cache_after_save(&copy_root,&context,true).await.map_err(|e|format!("mock incomplete branch: {e:?}"))?;
        if skipped.success || skipped.restarted || !copy_root.join(PENDING).is_file() {
            return Err("incomplete mock restart incorrectly acknowledged pending".into());
        }
        let kept: Value = serde_json::from_slice(&std::fs::read(copy_root.join(PENDING)).unwrap()).unwrap();
        if kept["id"].as_str()!=Some(pending_id) { return Err("copy pending changed after incomplete response".into()); }
        owner.allow_commit.store(true,Ordering::SeqCst);
        let completed = cache_refresh::sync_cache_after_save(&copy_root,&context,true).await.map_err(|e|format!("mock completed branch: {e:?}"))?;
        if !completed.success || !completed.restarted || copy_root.join(PENDING).exists() {
            return Err("successful mock restart did not CAS-acknowledge copy pending".into());
        }
        if std::fs::read(manager.join(PENDING)).unwrap()!=original { return Err("real pending was touched by simulation".into()); }
        Ok(json!({"simulationPassed":true,"productionEntry":"cache_refresh::sync_cache_after_save(true)",
            "actualPreparedPendingId":pending_id,"incompleteRestartRetainedPendingCopy":true,
            "successfulRestartAcknowledgedPendingCopy":true,"originalPreparedPendingRetained":true,
            "restartHttpCallsToSyntheticFixture":owner.restart_calls.load(Ordering::SeqCst),
            "realGlobalRestartCalled":false,"actualDesktopRestartVerified":false,
            "fixtureScope":"copied manager/home, mocked attested owner/exit response only; not the real routing acceptance"}))
    }.await;
    server.abort();
    let _ = server.await;
    result
}
