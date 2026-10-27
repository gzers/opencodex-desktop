//! Loopback-only E2E fixture. The browser uses production config_file/project
//! functions, with synthetic credentials and an isolated temp configuration.
use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue},
    routing::{get, post},
    Json, Router,
};
use opencodex_desktop_lib::modules::model_config::{
    compiler, config_file, file_objects, portable_channels, schema::ModelError, transfer,
};
use serde_json::{json, Value};
use std::{path::PathBuf, sync::Arc};
struct Fixture {
    root: PathBuf,
    home: PathBuf,
    stop: tokio::sync::Notify,
}
fn cors() -> HeaderMap {
    let mut h = HeaderMap::new();
    h.insert(
        "access-control-allow-origin",
        HeaderValue::from_static("http://127.0.0.1:5216"),
    );
    h.insert(
        "access-control-allow-methods",
        HeaderValue::from_static("GET,POST,OPTIONS"),
    );
    h.insert(
        "access-control-allow-headers",
        HeaderValue::from_static("content-type"),
    );
    h
}
async fn rpc(State(s): State<Arc<Fixture>>, Json(body): Json<Value>) -> (HeaderMap, Json<Value>) {
    let result = (|| -> Result<Value, ModelError> {
        let args = &body["args"];
        match body["command"].as_str().unwrap_or("") {
            "snapshot" => {
                serde_json::to_value(config_file::snapshot(&s.root, &s.home)?).map_err(Into::into)
            }
            "credential" => serde_json::to_value(config_file::credential(
                &s.home,
                args["providerId"].as_str().unwrap_or(""),
            )?)
            .map_err(Into::into),
            "project" => {
                let snap = config_file::snapshot(&s.root, &s.home)?;
                serde_json::to_value(compiler::project(
                    &snap.library,
                    &serde_json::from_value(args["policy"].clone())?,
                )?)
                .map_err(Into::into)
            }
            "save" => serde_json::to_value(config_file::save(
                &s.root,
                &s.home,
                serde_json::from_value(args.clone())?,
            )?)
            .map_err(Into::into),
            "prepareTransfer" => {
                let source = s.root.join("fixture-source");
                let home = s.root.join("fixture-source-home");
                std::fs::create_dir_all(&source).unwrap();
                std::fs::create_dir_all(&home).unwrap();
                std::fs::create_dir_all(source.join("manager-state")).unwrap();
                std::fs::write(
                    source.join("manager-state/model-policy.json"),
                    b"broken old copy",
                )
                .unwrap();
                std::fs::write(
                    source.join("manager-state/model-config-draft.json"),
                    b"broken old draft",
                )
                .unwrap();
                std::fs::write(home.join("config.json"), serde_json::to_vec(&json!({"providers":{"import-fixture":{"adapter":"openai-chat","authMode":"local","baseUrl":"https://source.example.test/v1","models":["site/exact-import"],"selectedModels":["site/exact-import"],"note":"current-file-source","retryOn429":false}}})).unwrap()).unwrap();
                let snap = config_file::snapshot(&source, &home)?;
                let scope = transfer::ExportScope {
                    channel_families: Default::default(),
                    channel_models: Default::default(),
                    include_policy: true,
                    include_global: false,
                    template_refs: vec![],
                    channel_ids: snap.policy.channels.iter().map(|c| c.id.clone()).collect(),
                };
                let manager = opencodex_desktop_lib::modules::sync::portable_manager::Scope {
                    upstream_root: Some(home),
                    ..Default::default()
                };
                let prepared = transfer::prepare_file_scope(&source, scope, None, false, &manager)?;
                let bytes = transfer::file_bytes(&source, &prepared.id)?;
                std::fs::write(s.root.join("fixture-transfer.ocxdconf"), bytes).unwrap();
                Ok(
                    json!({"sourceModels":prepared.package.payload.policy.unwrap().channels[0].model_order}),
                )
            }
            "receive" => {
                let bytes = std::fs::read(s.root.join("fixture-transfer.ocxdconf")).unwrap();
                serde_json::to_value(file_objects::receive_context_file(
                    &s.root, &bytes, None, None, &s.home,
                )?)
                .map_err(Into::into)
            }
            "select" => serde_json::to_value(transfer::select_received_context(
                &s.root,
                args["id"].as_str().unwrap(),
                serde_json::from_value(args["scope"].clone())?,
                None,
            )?)
            .map_err(Into::into),
            "accept" => serde_json::to_value(file_objects::accept_extended(
                &s.root,
                args["id"].as_str().unwrap(),
                false,
                false,
                false,
                None,
            )?)
            .map_err(Into::into),
            "catalog" => serde_json::to_value(portable_channels::catalog_file(&s.root, &s.home)?)
                .map_err(Into::into),
            "disk" => {
                let raw: Value =
                    serde_json::from_slice(&std::fs::read(s.home.join("config.json")).unwrap())?;
                Ok(
                    json!({"key":raw["providers"]["demo"]["apiKey"],"url":raw["providers"]["demo"]["baseUrl"],"unknown":raw["providers"]["demo"]["unknown"],"models":raw["providers"]["demo"]["models"],"providerCount":raw["providers"].as_object().unwrap().len(),"imported":raw["providers"].as_object().unwrap().values().find(|r|r["note"]=="current-file-source"),"backups":std::fs::read_dir(s.root.join("backups/model-config")).map(|i|i.count()).unwrap_or(0)}),
                )
            }
            "finish" => {
                s.stop.notify_one();
                Ok(Value::Null)
            }
            _ => Err(opencodex_desktop_lib::modules::model_config::invalid(
                "test_scope",
                "unsupported test request",
            )),
        }
    })();
    let payload = match result {
        Ok(value) => json!({"value":value}),
        Err(e) => json!({"error":{"code":e.code,"message":e.message}}),
    };
    (cors(), Json(payload))
}
#[tokio::test]
#[ignore = "explicit loopback browser fixture; see ui/qa/file-save-browser.mjs"]
async fn file_browser_fixture() {
    assert_eq!(
        std::env::var("OCX_FILE_BROWSER_FIXTURE").as_deref(),
        Ok("1")
    );
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("manager");
    let home = temp.path().join("official");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(home.join("config.json"),serde_json::to_vec(&json!({"providers":{"demo":{"adapter":"openai-chat","baseUrl":"https://example.test","authMode":"key","apiKey":"fixture-original-key","models":["site/custom-id"],"selectedModels":["site/custom-id"],"modelContextWindows":{"site/custom-id":64000},"unknown":{"keep":42}}}})).unwrap()).unwrap();
    let state = Arc::new(Fixture {
        root,
        home,
        stop: tokio::sync::Notify::new(),
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:5223")
        .await
        .unwrap();
    let app = Router::new()
        .route("/rpc", post(rpc).options(|| async { (cors(), "") }))
        .route("/ready", get(|| async { (cors(), "ready") }))
        .with_state(state.clone());
    axum::serve(listener,app).with_graceful_shutdown(async move {tokio::select!{_=state.stop.notified()=>{},_=tokio::time::sleep(std::time::Duration::from_secs(180))=>{}}}).await.unwrap();
}
