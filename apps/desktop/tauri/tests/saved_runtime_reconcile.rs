//! Real Management HTTP requests against a synthetic attested loopback owner.
//! No installed runtime, user configuration, credential backend, or restart.
use opencodex_desktop_lib::modules::model_config::cache_refresh::saved_runtime_reconcile;
use opencodex_desktop_lib::{modules, state};

use axum::{
    body::Bytes,
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use modules::model_config::management::Management;
use serde_json::{json, Value};
use sha2::Sha256;
use state::ProcessContext;
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

const SECRET: &str = "synthetic-provider-key-never-transferred";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fault {
    None,
    BadAttestation,
    ReloadUnavailable,
    ReloadMissingRow,
    ReloadDrift,
    ConfigDrift,
    BatchConflict,
    BatchUnavailable,
    BatchUnconfirmed,
    BatchMalformed,
    BatchDrift,
    BatchErrorDrift,
    KeepOld,
    MissingNew,
    WrongDefault,
    WrongRow,
    ReadbackDrift,
    ReadbackUnavailable,
}

#[derive(Clone, Debug)]
struct Call {
    operation: String,
    body: Value,
}

#[derive(Clone)]
struct Owner {
    root: PathBuf,
    pid: u32,
    port: u16,
    secret: String,
    fault: Fault,
    editor: Value,
    live: Arc<Mutex<Value>>,
    calls: Arc<Mutex<Vec<Call>>>,
    reads: Arc<AtomicUsize>,
    unexpected: Arc<AtomicUsize>,
    extra_field: Arc<Mutex<Option<(String, Value)>>>,
    catalog: Arc<Mutex<Option<Value>>>,
}

fn expected_editor() -> Value {
    json!({
        "defaultProvider":"Krill",
        "providers": {
            "Krill": {
                "adapter":"openai-responses","baseUrl":"https://example.test/v1",
                "models":["vendor/a:8b"],"selectedModels":["vendor/a:8b"],
                "modelCosts":{"vendor/a:8b":{"input":0.25,"output":0.5}},
                "codexAccountMode":"main","note":"Official registry note",
                "modelSupportsServiceTier":{"vendor/a:8b":true},
                "modelSuppressSyntheticMax":{"vendor/a:8b":true}
            },
            "xai": {
                "adapter":"openai-responses","baseUrl":"https://example.test/xai",
                "disabled":true,"models":["fixture-model"],"liveModels":false
            }
        }
    })
}

fn public_editor(editor: &Value) -> Value {
    let mut public = editor.clone();
    public["port"] = json!(1456);
    public["hostname"] = json!("127.0.0.1");
    public["defaultModelAliases"] = json!({"fast":"vendor/a:8b"});
    for row in public["providers"].as_object_mut().unwrap().values_mut() {
        row["hasApiKey"] = json!(true);
        row["hasHeaders"] = json!(true);
        row["initialModelSelection"] = json!({"kind":"static","models":["derived"]});
    }
    if let Some(row) = public["providers"].get_mut("xai") {
        row["xaiResponsesOptInState"] = json!("not-opted-in");
    }
    public
}

fn rejected(status: StatusCode) -> Response {
    (
        status,
        Json(json!({"error":SECRET,"path":"/synthetic/secret/path"})),
    )
        .into_response()
}

fn assert_admin(headers: &HeaderMap) {
    assert_eq!(
        headers[header::AUTHORIZATION],
        format!("Bearer ocx_admin_{}", "t".repeat(43))
    );
}

fn record(owner: &Owner, operation: &str, body: Value) {
    owner.calls.lock().unwrap().push(Call {
        operation: operation.into(),
        body,
    });
}

fn drift(owner: &Owner) {
    use std::io::Write;
    std::fs::OpenOptions::new()
        .append(true)
        .open(owner.root.join("config.json"))
        .unwrap()
        .write_all(b"\n")
        .unwrap();
}

async fn health(State(owner): State<Owner>, headers: HeaderMap) -> Response {
    assert!(headers.get(header::AUTHORIZATION).is_none());
    let challenge = headers["x-opencodex-attestation-challenge"]
        .to_str()
        .unwrap();
    let secret = if owner.fault == Fault::BadAttestation {
        b"wrong-owner".as_slice()
    } else {
        owner.secret.as_bytes()
    };
    let mut mac = Hmac::<Sha256>::new_from_slice(secret).unwrap();
    mac.update(
        format!(
            "opencodex-local-management-v1\n{challenge}\n{}\n{}",
            owner.pid, owner.port
        )
        .as_bytes(),
    );
    let mut response = Json(json!({"service":"opencodex","pid":owner.pid})).into_response();
    response.headers_mut().insert(
        "x-opencodex-attestation-proof",
        URL_SAFE_NO_PAD
            .encode(mac.finalize().into_bytes())
            .parse()
            .unwrap(),
    );
    response
}

async fn reload(State(owner): State<Owner>, headers: HeaderMap, body: Bytes) -> Response {
    assert!(headers.get(header::AUTHORIZATION).is_none());
    assert!(body.is_empty());
    assert_eq!(headers[header::CONTENT_LENGTH], "0");
    let name = headers["x-opencodex-provider-reload-name"]
        .to_str()
        .unwrap();
    assert_eq!(
        headers["x-opencodex-provider-reload-expected-pid"],
        owner.pid.to_string()
    );
    let nonce = headers["x-opencodex-provider-reload-nonce"]
        .to_str()
        .unwrap();
    assert_eq!(URL_SAFE_NO_PAD.decode(nonce).unwrap().len(), 32);
    let expires = headers["x-opencodex-provider-reload-expires-at"]
        .to_str()
        .unwrap();
    assert!((chrono::Utc::now().timestamp_millis()
        ..=chrono::Utc::now().timestamp_millis() + 10_000)
        .contains(&expires.parse::<i64>().unwrap()));
    let mut mac = Hmac::<Sha256>::new_from_slice(owner.secret.as_bytes()).unwrap();
    mac.update(format!("opencodex-local-provider-reload-v1\n{nonce}\nPOST\n/api/providers/reload\n{name}\n{}\n{}\n{expires}", owner.pid, owner.port).as_bytes());
    mac.verify_slice(
        &URL_SAFE_NO_PAD
            .decode(headers["x-opencodex-provider-reload-capability"].as_bytes())
            .unwrap(),
    )
    .unwrap();
    record(&owner, &format!("reload:{name}"), Value::Null);
    if owner.fault == Fault::ReloadUnavailable {
        return rejected(StatusCode::NOT_FOUND);
    }
    if owner.fault != Fault::ReloadMissingRow || name != "xai" {
        owner.live.lock().unwrap()["providers"][name] =
            public_editor(&owner.editor)["providers"][name].clone();
    }
    if owner.fault == Fault::ReloadDrift {
        drift(&owner);
    }
    Json(json!({"success":true,"name":name})).into_response()
}

async fn config(State(owner): State<Owner>, headers: HeaderMap, body: Bytes) -> Response {
    assert_admin(&headers);
    assert!(body.is_empty());
    record(&owner, "config", Value::Null);
    let read = owner.reads.fetch_add(1, Ordering::SeqCst);
    if read > 0 && owner.fault == Fault::ReadbackUnavailable {
        return rejected(StatusCode::NOT_FOUND);
    }
    if (read == 0 && owner.fault == Fault::ConfigDrift)
        || (read > 0 && owner.fault == Fault::ReadbackDrift)
    {
        drift(&owner);
    }
    let mut live = owner.live.lock().unwrap().clone();
    if let Some((key, value)) = &*owner.extra_field.lock().unwrap() {
        live["providers"]["xai"][key] = value.clone();
    }
    Json(live).into_response()
}

async fn batch(
    State(owner): State<Owner>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    assert_admin(&headers);
    record(&owner, "batch", body.clone());
    assert!(!body.to_string().contains(SECRET));
    if owner.fault == Fault::BatchUnavailable {
        return rejected(StatusCode::METHOD_NOT_ALLOWED);
    }
    if body != json!({"baseline":owner.editor,"next":owner.editor})
        || owner.fault == Fault::BatchConflict
    {
        return rejected(StatusCode::CONFLICT);
    }
    if owner.fault == Fault::BatchErrorDrift {
        drift(&owner);
        return rejected(StatusCode::CONFLICT);
    }
    let mut live = public_editor(&owner.editor);
    match owner.fault {
        Fault::KeepOld => live["providers"]["Krill-1"] = live["providers"]["Krill"].clone(),
        Fault::MissingNew => {
            live["providers"].as_object_mut().unwrap().remove("Krill");
        }
        Fault::WrongDefault => live["defaultProvider"] = json!("Krill-1"),
        Fault::WrongRow => live["providers"]["Krill"]["models"] = json!(["wrong"]),
        Fault::BatchDrift => drift(&owner),
        _ => {}
    }
    *owner.live.lock().unwrap() = live;
    if owner.fault == Fault::BatchMalformed {
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            "invalid-json",
        )
            .into_response();
    }
    let mut response = json!({"success":owner.fault != Fault::BatchUnconfirmed,"synced":false});
    if let Some(catalog) = &*owner.catalog.lock().unwrap() {
        response["catalogRefresh"] = catalog.clone();
    }
    Json(response).into_response()
}

async fn unexpected(State(owner): State<Owner>) -> Response {
    owner.unexpected.fetch_add(1, Ordering::SeqCst);
    rejected(StatusCode::NOT_FOUND)
}

struct Fixture {
    root: tempfile::TempDir,
    context: ProcessContext,
    owner: Owner,
    server: tokio::task::JoinHandle<()>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}

impl Fixture {
    async fn new(fault: Fault) -> Self {
        Self::with_editor(fault, expected_editor()).await
    }

    async fn with_editor(fault: Fault, editor: Value) -> Self {
        let root = tempfile::tempdir().unwrap();
        let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let mut live = public_editor(&editor);
        let old = live["providers"]
            .as_object_mut()
            .unwrap()
            .remove("Krill")
            .unwrap();
        live["providers"]["Krill-1"] = old;
        live["providers"]["obsolete-unrelated"] =
            json!({"adapter":"openai-chat","baseUrl":"https://example.test/obsolete"});
        live["defaultProvider"] = json!("Krill-1");
        let owner = Owner {
            root: root.path().into(),
            pid: std::process::id(),
            port: listener.local_addr().unwrap().port(),
            secret: "s".repeat(43),
            fault,
            editor: editor.clone(),
            live: Arc::new(Mutex::new(live)),
            calls: Default::default(),
            reads: Default::default(),
            unexpected: Default::default(),
            extra_field: Default::default(),
            catalog: Arc::new(Mutex::new(Some(json!({
                "status":"committed","changed":true,"degraded":false,"notices":[]
            })))),
        };
        let mut disk = editor;
        disk["providers"]["Krill"]["apiKey"] = json!(SECRET);
        disk["providers"]["Krill"]["apiKeyPool"] = json!([SECRET]);
        disk["providers"]["Krill"]["headers"] = json!({"Authorization":SECRET});
        disk["providers"]["Krill"]["futureSecretBlock"] = json!({"secret":SECRET});
        // Official DTO has its own baseUrl/note/cost/account-mode projection.
        // The adapter must not reconstruct it from these disk values.
        disk["providers"]["Krill"]["baseUrl"] =
            json!("https://synthetic:password@example.test/v1?token=private");
        disk["providers"]["Krill"]["note"] = json!("Raw operator note");
        disk["providers"]["Krill"]["modelCosts"]["ignored-invalid-model"] =
            json!({"input":"invalid"});
        disk["providers"]["Krill"]
            .as_object_mut()
            .unwrap()
            .remove("codexAccountMode");
        disk["unrelatedSecret"] = json!(SECRET);
        std::fs::write(
            root.path().join("config.json"),
            serde_json::to_vec_pretty(&disk).unwrap(),
        )
        .unwrap();
        std::fs::create_dir(root.path().join("manager-state")).unwrap();
        std::fs::write(
            root.path()
                .join("manager-state/provider-client-renames.json"),
            b"durable-pending-sentinel",
        )
        .unwrap();
        std::fs::write(root.path().join("models_cache.json"), b"cache-sentinel").unwrap();
        std::fs::write(
            root.path().join("admin-api-token"),
            format!("ocx_admin_{}", "t".repeat(43)),
        )
        .unwrap();
        std::fs::write(
            root.path().join("runtime-port.json"),
            serde_json::to_vec(
                &json!({"pid":owner.pid,"port":owner.port,"attestationSecret":owner.secret}),
            )
            .unwrap(),
        )
        .unwrap();
        let context = ProcessContext {
            runtime: opencodex_desktop_lib::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(),
            working_directory: root.path().into(), opencodex_home: root.path().into(),
        };
        let router = Router::new()
            .route("/healthz", get(health))
            .route("/api/providers/reload", post(reload))
            .route("/api/config", get(config))
            .route("/api/providers", put(batch))
            .fallback(unexpected)
            .with_state(owner.clone());
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        Self {
            root,
            context,
            owner,
            server,
        }
    }

    fn renames() -> BTreeMap<String, String> {
        BTreeMap::from([("Krill-1".into(), "Krill".into())])
    }

    async fn reconcile(&self) -> Result<(), modules::model_config::schema::ModelError> {
        let management = Management::bind(&self.context).await?;
        saved_runtime_reconcile::reconcile(&management, &self.context, &Self::renames()).await
    }

    fn operations(&self) -> Vec<String> {
        self.owner
            .calls
            .lock()
            .unwrap()
            .iter()
            .map(|call| call.operation.clone())
            .collect()
    }

    fn assert_sentinels(&self) {
        assert_eq!(
            std::fs::read(
                self.root
                    .path()
                    .join("manager-state/provider-client-renames.json")
            )
            .unwrap(),
            b"durable-pending-sentinel"
        );
        assert_eq!(
            std::fs::read(self.root.path().join("models_cache.json")).unwrap(),
            b"cache-sentinel"
        );
        assert_eq!(
            self.owner.unexpected.load(Ordering::SeqCst),
            0,
            "no restart, compensating POST/delete, or private API"
        );
    }
}

fn fingerprint(fixture: &Fixture) -> (Vec<u8>, std::time::SystemTime, u64) {
    let path = fixture.root.path().join("config.json");
    let metadata = std::fs::metadata(&path).unwrap();
    #[cfg(unix)]
    let identity = {
        use std::os::unix::fs::MetadataExt;
        metadata.ino()
    };
    #[cfg(not(unix))]
    let identity = metadata.len();
    (
        std::fs::read(path).unwrap(),
        metadata.modified().unwrap(),
        identity,
    )
}

#[tokio::test]
async fn complete_noop_editor_adopts_roster_default_without_disk_or_secret_roundtrip() {
    let fixture = Fixture::new(Fault::None).await;
    let before = fingerprint(&fixture);
    fixture.reconcile().await.unwrap();
    assert_eq!(
        fixture.operations(),
        ["reload:Krill", "reload:xai", "config", "batch", "config"]
    );
    let calls = fixture.owner.calls.lock().unwrap();
    assert_eq!(
        calls[3].body,
        json!({"baseline":expected_editor(),"next":expected_editor()})
    );
    assert_eq!(
        fixture.owner.live.lock().unwrap()["providers"]
            .as_object()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        fingerprint(&fixture),
        before,
        "config bytes, mtime and inode stay identical"
    );
    fixture.assert_sentinels();
}

#[tokio::test]
async fn retry_after_live_already_adopted_is_idempotent_and_does_not_clear_pending() {
    let fixture = Fixture::new(Fault::None).await;
    let before = fingerprint(&fixture);
    fixture.reconcile().await.unwrap();
    fixture.reconcile().await.unwrap();
    assert_eq!(
        fixture
            .operations()
            .iter()
            .filter(|op| *op == "batch")
            .count(),
        2
    );
    assert_eq!(fingerprint(&fixture), before);
    fixture.assert_sentinels();
}

#[tokio::test]
async fn empty_rename_map_still_adopts_saved_full_roster_and_default() {
    let fixture = Fixture::new(Fault::None).await;
    *fixture.owner.live.lock().unwrap() = public_editor(&fixture.owner.editor);
    fixture.owner.live.lock().unwrap()["providers"]["obsolete"] = json!({});
    fixture.owner.live.lock().unwrap()["defaultProvider"] = json!("obsolete");
    fixture.owner.live.lock().unwrap()["providers"]["xai"]["models"] = json!(["old-saved-model"]);
    let management = Management::bind(&fixture.context).await.unwrap();
    saved_runtime_reconcile::reconcile(&management, &fixture.context, &BTreeMap::new())
        .await
        .unwrap();
    assert_eq!(
        fixture.operations(),
        ["reload:Krill", "reload:xai", "config", "batch", "config"]
    );
    fixture.assert_sentinels();
}

#[tokio::test]
async fn multiple_old_prefixes_reload_shared_target_once() {
    let fixture = Fixture::new(Fault::None).await;
    let management = Management::bind(&fixture.context).await.unwrap();
    let mut renames = Fixture::renames();
    renames.insert("Krill-2".into(), "Krill".into());
    saved_runtime_reconcile::reconcile(&management, &fixture.context, &renames)
        .await
        .unwrap();
    assert_eq!(
        fixture.operations(),
        ["reload:Krill", "reload:xai", "config", "batch", "config"]
    );
}

#[tokio::test]
async fn all_distinct_rename_targets_reload_before_one_complete_batch() {
    let fixture = Fixture::new(Fault::None).await;
    {
        let mut live = fixture.owner.live.lock().unwrap();
        let old = live["providers"]
            .as_object_mut()
            .unwrap()
            .remove("xai")
            .unwrap();
        live["providers"]["xai-1"] = old;
    }
    let before = fingerprint(&fixture);
    let management = Management::bind(&fixture.context).await.unwrap();
    let mut renames = Fixture::renames();
    renames.insert("xai-1".into(), "xai".into());
    saved_runtime_reconcile::reconcile(&management, &fixture.context, &renames)
        .await
        .unwrap();
    assert_eq!(
        fixture.operations(),
        ["reload:Krill", "reload:xai", "config", "batch", "config"]
    );
    assert_eq!(fingerprint(&fixture), before);
    fixture.assert_sentinels();
}

#[tokio::test]
async fn stale_or_unsupported_owner_never_uses_repair_writes() {
    for (fault, code, operations) in [
        (
            Fault::ReloadUnavailable,
            "saved_runtime_unsupported",
            vec!["reload:Krill"],
        ),
        (
            Fault::BatchUnavailable,
            "saved_runtime_unsupported",
            vec!["reload:Krill", "reload:xai", "config", "batch"],
        ),
        (
            Fault::BatchConflict,
            "saved_runtime_baseline",
            vec!["reload:Krill", "reload:xai", "config", "batch"],
        ),
    ] {
        let fixture = Fixture::new(fault).await;
        let before = fingerprint(&fixture);
        let error = fixture.reconcile().await.unwrap_err();
        assert_eq!(error.code, code, "{fault:?}");
        assert!(!error.message.contains(SECRET) && !error.message.contains("/synthetic"));
        assert_eq!(fixture.operations(), operations);
        assert_eq!(fingerprint(&fixture), before);
        fixture.assert_sentinels();
    }
}

#[tokio::test]
async fn success_flag_and_complete_live_readback_are_required() {
    for (fault, code, reads) in [
        (Fault::BatchUnconfirmed, "saved_runtime_response", 1),
        (Fault::BatchMalformed, "saved_runtime_owner", 1),
        (Fault::KeepOld, "saved_runtime_readback", 2),
        (Fault::MissingNew, "saved_runtime_readback", 2),
        (Fault::WrongDefault, "saved_runtime_readback", 2),
        (Fault::WrongRow, "saved_runtime_readback", 2),
        (Fault::ReadbackUnavailable, "saved_runtime_unsupported", 2),
    ] {
        let fixture = Fixture::new(fault).await;
        let before = fingerprint(&fixture);
        assert_eq!(
            fixture.reconcile().await.unwrap_err().code,
            code,
            "{fault:?}"
        );
        assert_eq!(fixture.owner.reads.load(Ordering::SeqCst), reads);
        assert_eq!(
            fixture
                .operations()
                .iter()
                .filter(|op| *op == "batch")
                .count(),
            1
        );
        assert_eq!(fingerprint(&fixture), before);
        fixture.assert_sentinels();
    }
}

#[tokio::test]
async fn exact_file_drift_fails_even_after_rejection_or_successful_live_adoption() {
    for (fault, operations) in [
        (Fault::ReloadDrift, vec!["reload:Krill"]),
        (
            Fault::ConfigDrift,
            vec!["reload:Krill", "reload:xai", "config"],
        ),
        (
            Fault::BatchDrift,
            vec!["reload:Krill", "reload:xai", "config", "batch"],
        ),
        (
            Fault::BatchErrorDrift,
            vec!["reload:Krill", "reload:xai", "config", "batch"],
        ),
        (
            Fault::ReadbackDrift,
            vec!["reload:Krill", "reload:xai", "config", "batch", "config"],
        ),
    ] {
        let fixture = Fixture::new(fault).await;
        let before = fingerprint(&fixture).0;
        assert_eq!(
            fixture.reconcile().await.unwrap_err().code,
            "saved_runtime_drift",
            "{fault:?}"
        );
        assert_eq!(fixture.operations(), operations);
        let mut expected = before;
        expected.push(b'\n');
        assert_eq!(
            fingerprint(&fixture).0,
            expected,
            "concurrent bytes must never be restored/overwritten"
        );
        fixture.assert_sentinels();
    }
}

#[tokio::test]
async fn unknown_secret_and_unexpected_runtime_dto_fields_fail_before_batch() {
    for (key, value) in [
        ("futureEditorField", json!({"secret":SECRET})),
        ("apiKey", json!(SECRET)),
        ("apiKeyPool", json!([{"key":SECRET}])),
        ("headers", json!({"Authorization":SECRET})),
        ("proxy", json!("https://user:password@example.test")),
        ("mcpServers", json!({"fixture":{"env":{"TOKEN":SECRET}}})),
        ("desktopExecutor", json!({"env":{"TOKEN":SECRET}})),
        ("modelMaxInputTokens", json!({"model":123})),
        ("modelCosts", json!({"password":SECRET})),
        ("baseUrl", json!("https://user:password@example.test/v1")),
    ] {
        let fixture = Fixture::new(Fault::None).await;
        let before = fingerprint(&fixture);
        *fixture.owner.extra_field.lock().unwrap() = Some((key.into(), value));
        let error = fixture.reconcile().await.unwrap_err();
        assert_eq!(error.code, "saved_runtime_dto", "{key}");
        assert!(!error.message.contains(SECRET));
        assert_eq!(
            fixture.operations(),
            ["reload:Krill", "reload:xai", "config"]
        );
        assert_eq!(fingerprint(&fixture), before);
        fixture.assert_sentinels();
    }
}

#[tokio::test]
async fn missing_saved_public_row_is_not_invented_from_secret_disk_data() {
    let fixture = Fixture::new(Fault::ReloadMissingRow).await;
    fixture.owner.live.lock().unwrap()["providers"]
        .as_object_mut()
        .unwrap()
        .remove("xai");
    assert_eq!(
        fixture.reconcile().await.unwrap_err().code,
        "saved_runtime_baseline"
    );
    assert_eq!(
        fixture.operations(),
        ["reload:Krill", "reload:xai", "config"]
    );
    fixture.assert_sentinels();
}

#[tokio::test]
async fn previously_added_or_modified_saved_sibling_is_reloaded_before_rename_batch() {
    for previously_present in [false, true] {
        let mut editor = expected_editor();
        editor["providers"]["C"] = json!({
            "adapter":"openai-chat","baseUrl":"https://example.test/new-c",
            "models":["saved-c-model"],"liveModels":false
        });
        let fixture = Fixture::with_editor(Fault::None, editor.clone()).await;
        {
            let mut live = fixture.owner.live.lock().unwrap();
            if previously_present {
                live["providers"]["C"]["baseUrl"] = json!("https://example.test/old-c");
                live["providers"]["C"]["models"] = json!(["old-c-model"]);
            } else {
                live["providers"].as_object_mut().unwrap().remove("C");
            }
        }
        let before = fingerprint(&fixture);
        fixture.reconcile().await.unwrap();
        assert_eq!(
            fixture.operations(),
            [
                "reload:C",
                "reload:Krill",
                "reload:xai",
                "config",
                "batch",
                "config"
            ]
        );
        let calls = fixture.owner.calls.lock().unwrap();
        assert_eq!(calls[4].body, json!({"baseline":editor,"next":editor}));
        assert_eq!(fingerprint(&fixture), before);
        fixture.assert_sentinels();
    }
}

#[tokio::test]
async fn canonical_provider_is_included_in_full_saved_reload_order() {
    let mut editor = expected_editor();
    editor["providers"]["openai"] = json!({
        "adapter":"openai-responses","baseUrl":"https://chatgpt.com/backend-api/codex",
        "authMode":"forward"
    });
    let fixture = Fixture::with_editor(Fault::None, editor).await;
    let before = fingerprint(&fixture);
    fixture.reconcile().await.unwrap();
    assert_eq!(
        fixture.operations(),
        [
            "reload:Krill",
            "reload:openai",
            "reload:xai",
            "config",
            "batch",
            "config"
        ]
    );
    assert_eq!(fingerprint(&fixture), before);
    fixture.assert_sentinels();
}

#[tokio::test]
async fn catalog_failed_skipped_missing_or_malformed_never_finishes_the_action() {
    for catalog in [
        Some(
            json!({"status":"failed","reason":"disk","phase":"commit","retryable":true,"partialWrite":true}),
        ),
        Some(json!({"status":"skipped","reason":"busy","retryable":true})),
        None,
        Some(Value::Null),
        Some(json!({"status":"committed"})),
        Some(json!({"status":"committed","changed":"true","degraded":false,"notices":[]})),
        Some(json!({"status":"committed","changed":false,"degraded":false,"notices":[SECRET]})),
        Some(
            json!({"status":"committed","changed":false,"degraded":false,"notices":["fallback","fallback","fallback","fallback"]}),
        ),
    ] {
        let fixture = Fixture::new(Fault::None).await;
        let before = fingerprint(&fixture);
        *fixture.owner.catalog.lock().unwrap() = catalog;
        let error = fixture.reconcile().await.unwrap_err();
        assert_eq!(error.code, "saved_runtime_catalog");
        assert!(!error.message.contains(SECRET));
        assert_eq!(
            fixture.operations(),
            ["reload:Krill", "reload:xai", "config", "batch"]
        );
        assert_eq!(fixture.owner.reads.load(Ordering::SeqCst), 1);
        assert_eq!(fingerprint(&fixture), before);
        // Roster mutation may already have happened. That is partial progress,
        // never permission to clear pending or continue into a restart.
        assert_eq!(
            fixture.owner.live.lock().unwrap()["defaultProvider"],
            "Krill"
        );
        fixture.assert_sentinels();
    }
}

#[tokio::test]
async fn committed_catalog_accepts_no_change_or_degradation_without_synced_heuristics() {
    for catalog in [
        json!({"status":"committed","changed":false,"degraded":false,"notices":[]}),
        json!({"status":"committed","changed":true,"degraded":true,"notices":["provider-network","fallback"]}),
    ] {
        let fixture = Fixture::new(Fault::None).await;
        let before = fingerprint(&fixture);
        *fixture.owner.catalog.lock().unwrap() = Some(catalog);
        fixture.reconcile().await.unwrap();
        assert_eq!(
            fixture.operations(),
            ["reload:Krill", "reload:xai", "config", "batch", "config"]
        );
        assert_eq!(fingerprint(&fixture), before);
        fixture.assert_sentinels();
    }
}

#[tokio::test]
async fn invalid_pending_map_stops_before_management_operations() {
    for (from, to) in [
        ("xai", "Krill"),
        ("Krill-1", "absent"),
        ("Krill", "Krill"),
        ("bad/name", "Krill"),
    ] {
        let fixture = Fixture::new(Fault::None).await;
        let before = fingerprint(&fixture);
        let management = Management::bind(&fixture.context).await.unwrap();
        let renames = BTreeMap::from([(from.into(), to.into())]);
        assert_eq!(
            saved_runtime_reconcile::reconcile(&management, &fixture.context, &renames)
                .await
                .unwrap_err()
                .code,
            "saved_runtime_rename"
        );
        assert!(fixture.operations().is_empty());
        assert_eq!(fingerprint(&fixture), before);
        fixture.assert_sentinels();
    }
}

#[tokio::test]
async fn untrusted_listener_never_receives_admin_or_reload_capability() {
    let fixture = Fixture::new(Fault::BadAttestation).await;
    std::fs::remove_file(fixture.root.path().join("admin-api-token")).unwrap();
    assert_eq!(
        fixture.reconcile().await.unwrap_err().code,
        "runtime_attestation"
    );
    assert!(fixture.operations().is_empty());
    fixture.assert_sentinels();
}

#[tokio::test]
async fn changed_attested_record_stops_before_provider_operations() {
    let fixture = Fixture::new(Fault::None).await;
    let management = Management::bind(&fixture.context).await.unwrap();
    std::fs::write(fixture.root.path().join("runtime-port.json"),
        serde_json::to_vec(&json!({"pid":fixture.owner.pid+1,"port":fixture.owner.port,"attestationSecret":fixture.owner.secret})).unwrap()).unwrap();
    assert_eq!(
        saved_runtime_reconcile::reconcile(&management, &fixture.context, &Fixture::renames())
            .await
            .unwrap_err()
            .code,
        "stale_runtime"
    );
    assert!(fixture.operations().is_empty());
    fixture.assert_sentinels();
}

#[tokio::test]
async fn invalid_or_missing_config_does_not_fall_back_to_live_or_defaults() {
    for bytes in [
        b"invalid".as_slice(),
        b"{}",
        br#"{"providers":{},"defaultProvider":"Krill"}"#,
        br#"{"providers":{"Krill":{"disabled":true}},"defaultProvider":"Krill"}"#,
    ] {
        let fixture = Fixture::new(Fault::None).await;
        std::fs::write(fixture.root.path().join("config.json"), bytes).unwrap();
        assert_eq!(
            fixture.reconcile().await.unwrap_err().code,
            "saved_runtime_file"
        );
        assert!(fixture.operations().is_empty());
        assert_eq!(
            std::fs::read(fixture.root.path().join("config.json")).unwrap(),
            bytes
        );
        fixture.assert_sentinels();
    }
    let fixture = Fixture::new(Fault::None).await;
    std::fs::remove_file(fixture.root.path().join("config.json")).unwrap();
    assert_eq!(
        fixture.reconcile().await.unwrap_err().code,
        "saved_runtime_file"
    );
    assert!(fixture.operations().is_empty());
    assert!(!fixture.root.path().join("config.json").exists());
    fixture.assert_sentinels();
}

#[cfg(unix)]
#[tokio::test]
async fn linked_saved_config_is_rejected_without_reading_or_modifying_its_target() {
    let fixture = Fixture::new(Fault::None).await;
    let path = fixture.root.path().join("config.json");
    let target = fixture.root.path().join("private-config.json");
    std::fs::rename(&path, &target).unwrap();
    let original = std::fs::read(&target).unwrap();
    std::os::unix::fs::symlink(&target, &path).unwrap();
    assert_eq!(
        fixture.reconcile().await.unwrap_err().code,
        "saved_runtime_file"
    );
    assert!(fixture.operations().is_empty());
    assert_eq!(std::fs::read(&target).unwrap(), original);
    fixture.assert_sentinels();
}
