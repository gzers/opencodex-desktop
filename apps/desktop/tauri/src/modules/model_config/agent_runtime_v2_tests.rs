use super::*;
use crate::{
    infrastructure::runtime_executable::FixedRuntimeExecutable,
    modules::model_config::{
        schema::Field,
        store_v2::{DRAFT_PATH, LIBRARY_PATH},
    },
};
use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
#[derive(Clone)]
struct ServerState {
    pid: u32,
    port: u16,
    secret: String,
    response: Arc<Mutex<Value>>,
    calls: Arc<Mutex<Vec<Value>>>,
    delay: Arc<std::sync::atomic::AtomicBool>,
}
async fn health(State(state): State<ServerState>, headers: HeaderMap) -> (HeaderMap, Json<Value>) {
    let challenge = headers
        .get("x-opencodex-attestation-challenge")
        .unwrap()
        .to_str()
        .unwrap();
    let mut hmac = Hmac::<Sha256>::new_from_slice(state.secret.as_bytes()).unwrap();
    hmac.update(
        format!(
            "opencodex-local-management-v1\n{challenge}\n{}\n{}",
            state.pid, state.port
        )
        .as_bytes(),
    );
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-opencodex-attestation-proof",
        HeaderValue::from_str(&URL_SAFE_NO_PAD.encode(hmac.finalize().into_bytes())).unwrap(),
    );
    (
        headers,
        Json(json!({"service":"opencodex","pid":state.pid})),
    )
}
async fn response(State(state): State<ServerState>, Json(body): Json<Value>) -> Json<Value> {
    state.calls.lock().unwrap().push(body);
    if state.delay.load(std::sync::atomic::Ordering::SeqCst) {
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
    Json(state.response.lock().unwrap().clone())
}
struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    context: ProcessContext,
    state: ServerState,
    server: tokio::task::JoinHandle<()>,
    reference: RefV2,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}
impl Fixture {
    #[cfg(unix)]
    async fn new() -> Self {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("D");
        let home = temp.path().join("H");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        let executable = temp.path().join("official-fixture");
        let context = ProcessContext {
            runtime: FixedRuntimeExecutable::resolved(executable.clone()),
            working_directory: temp.path().to_owned(),
            opencodex_home: home.clone(),
        };
        let raw = json!({"providers":{"fixture":{"adapter":"openai-chat","baseUrl":"https://models.invalid/v1","authMode":"key","apiKey":"SOURCE-KEY-MUST-NOT-ENTER-PROMPT","models":["model/a"],"selectedModels":["model/a"]}},"modelPickerOrder":["fixture/model/a"],"defaultProvider":"fixture"});
        std::fs::write(home.join("config.json"), canonical_bytes(&raw).unwrap()).unwrap();
        let mut providers = raw;
        providers["providers"]["fixture"]
            .as_object_mut()
            .unwrap()
            .remove("apiKey");
        let data = [
            ("providers", providers),
            (
                "models",
                json!([{"provider":"fixture","id":"model/a","namespaced":"verified-fixture/route-a","reasoningEfforts":["none","low"],"fastRowAvailable":false}]),
            ),
            ("order", json!({"pickerOrder":["verified-fixture/route-a"]})),
            ("catalog", json!({})),
        ];
        for (key, value) in data {
            std::fs::write(
                temp.path().join(format!("{key}.json")),
                canonical_bytes(&value).unwrap(),
            )
            .unwrap();
        }
        let script=format!("#!/bin/sh\ncase \"$1 $2\" in\n'--version ') echo '2.97.0-fixture';;\n'provider snapshot') exec /bin/cat '{}';;\n'models live') exec /bin/cat '{}';;\n'models order') exec /bin/cat '{}';;\n'inspect catalog') exec /bin/cat '{}';;\n*) exit 7;;\nesac\n",temp.path().join("providers.json").display(),temp.path().join("models.json").display(),temp.path().join("order.json").display(),temp.path().join("catalog.json").display());
        std::fs::write(&executable, script).unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address: SocketAddr = listener.local_addr().unwrap();
        let state = ServerState {
            pid: std::process::id(),
            port: address.port(),
            secret: URL_SAFE_NO_PAD.encode([7u8; 32]),
            response: Arc::new(Mutex::new(json!({}))),
            calls: Arc::new(Mutex::new(vec![])),
            delay: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        };
        let app = Router::new()
            .route("/healthz", get(health))
            .route("/v1/responses", post(response))
            .with_state(state.clone());
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        std::fs::write(
            home.join("runtime-port.json"),
            canonical_bytes(
                &json!({"pid":state.pid,"port":state.port,"attestationSecret":state.secret}),
            )
            .unwrap(),
        )
        .unwrap();
        let saved = directory::capture_saved(&root, &context).unwrap();
        let live = super::super::upstream::snapshot(&context).await.unwrap();
        let settings = AgentSettings {
            provider_id: "fixture".into(),
            model_id: "model/a".into(),
            reasoning_effort: Some("none".into()),
            fast: Some(false),
            channel_id: None,
            connection_digest: None,
        };
        let bound = directory::bind(&saved, &live, settings).unwrap();
        super::super::agent::save_settings(&root, bound.settings().clone()).unwrap();
        let t = &saved.library().templates[0];
        let r = &t.revisions[0];
        let reference = RefV2 {
            template_id: t.id.clone(),
            revision_id: r.id.clone(),
            digest: r.digest.clone(),
        };
        Self {
            _temp: temp,
            root,
            context,
            state,
            server,
            reference,
        }
    }
    fn intent(&self, scene: Scene) -> GenerateIntentV2 {
        GenerateIntentV2 {
            scene,
            targets: vec![self.reference.clone()],
            allow_new: false,
            expected_library: directory::capture_saved(&self.root, &self.context)
                .unwrap()
                .library_digest()
                .unwrap(),
            requirements: "Please adjust the description using the v2 contract".into(),
            history: vec![],
            sources: vec![],
            buffer: None,
        }
    }
    fn proposal(&self) -> AgentProposalV2 {
        let saved = directory::capture_saved(&self.root, &self.context).unwrap();
        let old = revision(saved.library(), &self.reference).unwrap();
        let mut content = old.content.clone();
        content.description = "strict runtime-reviewed proposal".into();
        AgentProposalV2 {
            contract_version: 2,
            expected_library: saved.library_digest().unwrap(),
            proposals: vec![ContentProposalV2 {
                target: Some(self.reference.clone()),
                label: old.saved_label.clone(),
                content_version: 2,
                content,
            }],
        }
    }
    fn output(&self, value: Value) {
        *self.state.response.lock().unwrap() =
            json!({"output_text":String::from_utf8(canonical_bytes(&value).unwrap()).unwrap()});
    }
}
#[cfg(unix)]
#[tokio::test]
async fn verified_cli_and_runtime_execute_v2_prompt_then_review_and_backend_accept_plan() {
    let f = Fixture::new().await;
    let proposal = f.proposal();
    f.output(serde_json::to_value(&proposal).unwrap());
    let ticket = prepare_run(&f.root, &f.context, f.intent(Scene::UpdateModels)).unwrap();
    assert!(valid_id(&ticket.run_id));
    let before = std::fs::read(f.context.opencodex_home.join("config.json")).unwrap();
    let generated = generate(&f.root, &f.context, &ticket.run_id).await.unwrap();
    assert_eq!(generated.phase, RunPhaseV2::Review);
    assert!(generated.review.as_ref().unwrap().valid);
    assert!(!f.root.join(LIBRARY_PATH).exists());
    assert_eq!(
        before,
        std::fs::read(f.context.opencodex_home.join("config.json")).unwrap()
    );
    let calls = f.state.calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0]["model"], "verified-fixture/route-a");
    let prompt = calls[0]["input"][1]["content"].as_str().unwrap();
    assert!(!prompt.contains("SOURCE-KEY-MUST-NOT-ENTER-PROMPT"));
    assert!(!prompt.contains("\"templates\":"));
    assert!(prompt.contains("\"contractVersion\":2"));
    assert!(prompt.contains("\"nameLayouts\""));
    drop(calls);
    assert_eq!(
        latest(&f.root, Some(&f.reference.template_id))
            .unwrap()
            .unwrap()
            .run_id,
        ticket.run_id
    );
    let plan = prepare_accept(
        &f.root,
        &f.context,
        &ticket.run_id,
        &proposal.expected_library,
        &canonical_bytes(&proposal).unwrap(),
    )
    .unwrap();
    assert!(plan.operation_id.starts_with("agent-v2/"));
    let new = plan.review.proposals[0].planned_revision.as_ref().unwrap();
    assert_ne!(new.revision_id, f.reference.revision_id);
    assert!(
        matches!(revision(&plan.library,new).unwrap().origin,super::super::schema::SaveOrigin::InternalAgent(ref id) if id==&ticket.run_id)
    );
    assert_eq!(
        mark_accepted(&f.root, &f.context, &ticket.run_id, &plan.response_digest)
            .unwrap_err()
            .code,
        "model_agent_commit_unconfirmed"
    );
    // Simulate only L0's library commit inside this temp root. Runtime never
    // writes it; real multi-object transaction coverage remains L0-owned.
    std::fs::write(
        f.root.join(LIBRARY_PATH),
        canonical_bytes(&plan.library).unwrap(),
    )
    .unwrap();
    assert_eq!(
        mark_accepted(&f.root, &f.context, &ticket.run_id, &plan.response_digest)
            .unwrap()
            .phase,
        RunPhaseV2::Accepted
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(record_path(&f.root, &ticket.run_id).unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
#[cfg(unix)]
#[tokio::test]
async fn check_only_uses_a_strict_report_and_cannot_produce_a_save_plan() {
    let f = Fixture::new().await;
    let intent = f.intent(Scene::CheckOnly);
    f.output(json!({"contractVersion":2,"expectedLibrary":intent.expected_library,"report":{"summary":"Checked exact v2 rules","issues":[]}}));
    let ticket = prepare_run(&f.root, &f.context, intent).unwrap();
    let output = generate(&f.root, &f.context, &ticket.run_id).await.unwrap();
    assert!(output.review.is_none());
    assert_eq!(output.report.unwrap().summary, "Checked exact v2 rules");
    assert!(!f.root.join(LIBRARY_PATH).exists());
    assert!(!f.root.join(DRAFT_PATH).exists());
    let bytes = canonical_bytes(&f.proposal()).unwrap();
    assert_eq!(
        prepare_accept(
            &f.root,
            &f.context,
            &ticket.run_id,
            &ticket.expected_library,
            &bytes
        )
        .unwrap_err()
        .code,
        "model_agent_run_state"
    );
    for value in [
        json!({"contractVersion":2,"expectedLibrary":ticket.expected_library,"report":{"summary":"x","issues":[]},"origin":"builtin"}),
        json!({"contractVersion":2,"expectedLibrary":ticket.expected_library,"report":{"summary":"x","issues":[],"command":"shell"}}),
    ] {
        assert!(check_report(&canonical_bytes(&value).unwrap(), &ticket.expected_library).is_err());
    }
}
#[cfg(unix)]
#[tokio::test]
async fn cancel_before_send_and_invalid_response_do_not_save_the_library() {
    let f = Fixture::new().await;
    let ticket = prepare_run(&f.root, &f.context, f.intent(Scene::UpdateModels)).unwrap();
    stop(&ticket.run_id).unwrap();
    assert!(generate(&f.root, &f.context, &ticket.run_id).await.is_err());
    assert!(f.state.calls.lock().unwrap().is_empty());
    f.output(json!({"contractVersion":1,"proposals":[]}));
    let ticket = prepare_run(&f.root, &f.context, f.intent(Scene::UpdateModels)).unwrap();
    assert_eq!(
        generate(&f.root, &f.context, &ticket.run_id)
            .await
            .unwrap_err()
            .code,
        "model_schema_unsupported"
    );
    assert_eq!(
        latest(&f.root, None).unwrap().unwrap().phase,
        RunPhaseV2::Failed
    );
    assert!(!f.root.join(LIBRARY_PATH).exists());
}
#[cfg(unix)]
#[tokio::test]
async fn buffer_added_root_and_condition_locks_use_the_same_real_lock_checker() {
    let f = Fixture::new().await;
    let saved = directory::capture_saved(&f.root, &f.context).unwrap();
    let original = revision(saved.library(), &f.reference).unwrap();
    let mut content = original.content.clone();
    content.locked_fields.insert("description".into());
    content.condition_overrides[0]
        .locked_fields
        .insert("contextWindow".into());
    let mut buffer = EditingBufferV2 {
        target: Some(f.reference.clone()),
        label: original.saved_label.clone(),
        content,
        digest: String::new(),
        generation: 4,
    };
    buffer.digest = buffer_digest(&buffer).unwrap();
    let mut intent = f.intent(Scene::UpdateModels);
    intent.buffer = Some(buffer.clone());
    validate_buffer(saved.library(), &intent).unwrap();
    let ticket = prepare_run(&f.root, &f.context, intent).unwrap();
    let record = read_record(&f.root, &ticket.run_id).unwrap();
    let mut proposal = f.proposal();
    proposal.proposals[0].content = buffer.content.clone();
    proposal.proposals[0].content.description = "would break buffer lock".into();
    assert_eq!(
        validate_response_buffer(&record, &canonical_bytes(&proposal).unwrap())
            .unwrap_err()
            .code,
        "model_locked_field"
    );
    proposal.proposals[0].content = buffer.content.clone();
    proposal.proposals[0].content.condition_overrides[0]
        .rules
        .context_window = Field::Custom(1);
    assert_eq!(
        validate_response_buffer(&record, &canonical_bytes(&proposal).unwrap())
            .unwrap_err()
            .code,
        "model_locked_field"
    );
    let mut invalid = f.intent(Scene::UpdateModels);
    buffer.digest = digest(b"wrong buffer");
    invalid.buffer = Some(buffer);
    assert_eq!(
        prepare_run(&f.root, &f.context, invalid).unwrap_err().code,
        "model_agent_buffer_stale"
    );
    assert_eq!(saved.library().templates[0].revisions.len(), 1);
    assert!(!f.root.join(LIBRARY_PATH).exists());
}
#[cfg(unix)]
#[tokio::test]
async fn stale_source_or_saved_library_refuses_execution_and_review_identity_injection() {
    let f = Fixture::new().await;
    let proposal = f.proposal();
    f.output(serde_json::to_value(&proposal).unwrap());
    let ticket = prepare_run(&f.root, &f.context, f.intent(Scene::UpdateModels)).unwrap();
    let path = f.context.opencodex_home.join("config.json");
    let mut changed: Value = parse_strict_json_v2(&std::fs::read(&path).unwrap()).unwrap();
    changed["providers"]["fixture"]["baseUrl"] = json!("https://changed.invalid");
    std::fs::write(path, canonical_bytes(&changed).unwrap()).unwrap();
    assert_eq!(
        generate(&f.root, &f.context, &ticket.run_id)
            .await
            .unwrap_err()
            .code,
        "model_execution_binding_stale"
    );
    assert!(f.state.calls.lock().unwrap().is_empty());
    let mut value = serde_json::to_value(f.intent(Scene::UpdateModels)).unwrap();
    value["origin"] = json!("builtin");
    assert!(serde_json::from_value::<GenerateIntentV2>(value).is_err());
}

#[cfg(unix)]
#[tokio::test]
async fn trusted_buffer_preserves_new_locks_in_one_real_append_and_rejects_forgery() {
    let f = Fixture::new().await;
    let saved = directory::capture_saved(&f.root, &f.context).unwrap();
    let old = revision(saved.library(), &f.reference).unwrap();
    let mut content = old.content.clone();
    content.description = "human unsaved locked description".into();
    content.locked_fields.insert("description".into());
    content.condition_overrides[0]
        .locked_fields
        .insert("contextWindow".into());
    let mut added = content.condition_overrides[0].clone();
    added.id = uuid::Uuid::new_v4().to_string();
    added.source_exact_id = "gpt-999-buffer".into();
    added.selector = SelectorV2 {
        all: vec![SelectorAtomV2::ExactId {
            value: added.source_exact_id.clone(),
        }],
        layout_id: None,
        literal_tail: None,
    };
    content.condition_overrides.push(added);
    let mut buffer = EditingBufferV2 {
        target: Some(f.reference.clone()),
        label: old.saved_label.clone(),
        content: content.clone(),
        digest: String::new(),
        generation: 12,
    };
    buffer.digest = buffer_digest(&buffer).unwrap();
    let mut intent = f.intent(Scene::UpdateModels);
    intent.buffer = Some(buffer.clone());
    let ticket = prepare_run(&f.root, &f.context, intent.clone()).unwrap();
    let mut proposal = f.proposal();
    proposal.proposals[0].content = content;
    f.output(serde_json::to_value(&proposal).unwrap());
    let output = generate(&f.root, &f.context, &ticket.run_id).await.unwrap();
    assert!(output.review.unwrap().valid);
    let bytes = canonical_bytes(&proposal).unwrap();
    let plan = prepare_accept(
        &f.root,
        &f.context,
        &ticket.run_id,
        &ticket.expected_library,
        &bytes,
    )
    .unwrap();
    let reference = plan.review.proposals[0].planned_revision.as_ref().unwrap();
    let new = revision(&plan.library, reference).unwrap();
    assert_eq!(
        new.base_revision.as_deref(),
        Some(f.reference.revision_id.as_str())
    );
    assert_eq!(new.content, buffer.content);
    assert_eq!(plan.library.templates[0].revisions.len(), 2);
    assert_eq!(
        plan.library.templates[0].revisions[0],
        saved.library().templates[0].revisions[0]
    );
    assert!(!f.root.join(LIBRARY_PATH).exists());
    let record = read_record(&f.root, &ticket.run_id).unwrap();
    let context = proposal_context(&record, false).unwrap();
    assert!(
        !parse_validate_proposal_v2(saved.library(), &bytes, &context)
            .unwrap()
            .valid
    );
    for mutation in 0..4 {
        let mut bad = proposal.clone();
        match mutation {
            0 => {
                bad.proposals[0].content.locked_fields.clear();
            }
            1 => {
                bad.proposals[0].content.description = "model changed lock".into();
            }
            2 => {
                bad.proposals[0].content.condition_overrides.pop();
            }
            _ => {
                bad.proposals[0].content.condition_overrides[0]
                    .locked_fields
                    .insert("fast".into());
            }
        }
        assert!(prepare_accept(
            &f.root,
            &f.context,
            &ticket.run_id,
            &ticket.expected_library,
            &canonical_bytes(&bad).unwrap()
        )
        .is_err());
    }
    let mut bad = trusted_buffer(&intent).unwrap();
    bad.generation += 1;
    assert_eq!(
        bad.validate(saved.library(), &context).unwrap_err().code,
        "model_agent_buffer_stale"
    );
    bad = trusted_buffer(&intent).unwrap();
    bad.target.as_mut().unwrap().digest = digest(b"stale target");
    bad.digest = digest(
        &canonical_bytes(&(2, &bad.target, &bad.label, &bad.content, bad.generation)).unwrap(),
    );
    assert!(bad.validate(saved.library(), &context).is_err());
    let mut changed = plan.library.clone();
    changed.templates[0].label = "concurrent library edit".into();
    std::fs::write(
        f.root.join(LIBRARY_PATH),
        canonical_bytes(&changed).unwrap(),
    )
    .unwrap();
    assert_eq!(
        prepare_accept(
            &f.root,
            &f.context,
            &ticket.run_id,
            &ticket.expected_library,
            &bytes
        )
        .unwrap_err()
        .code,
        "model_save_stale"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn cancellation_during_response_never_delivers_review_or_save_authority() {
    let f = Fixture::new().await;
    f.output(serde_json::to_value(f.proposal()).unwrap());
    f.state
        .delay
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let ticket = prepare_run(&f.root, &f.context, f.intent(Scene::UpdateModels)).unwrap();
    let execute = generate(&f.root, &f.context, &ticket.run_id);
    let cancel = async {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if !f.state.calls.lock().unwrap().is_empty() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
        stop(&ticket.run_id).unwrap();
    };
    let (result, _) = tokio::join!(execute, cancel);
    assert_eq!(result.unwrap_err().code, "model_agent_stopped");
    let record = read_record(&f.root, &ticket.run_id).unwrap();
    assert_eq!(record.phase, RunPhaseV2::Stopped);
    assert!(record.review.is_none());
    assert!(record.accepted_digest.is_none());
    assert!(!f.root.join(LIBRARY_PATH).exists());
}

#[cfg(unix)]
#[tokio::test]
async fn invalid_attestation_and_oversized_envelope_fail_closed() {
    let f = Fixture::new().await;
    let path = f.context.opencodex_home.join("runtime-port.json");
    std::fs::write(&path, canonical_bytes(&json!({"pid":f.state.pid,"port":f.state.port,"attestationSecret":URL_SAFE_NO_PAD.encode([9u8;32])})).unwrap()).unwrap();
    let ticket = prepare_run(&f.root, &f.context, f.intent(Scene::UpdateModels)).unwrap();
    assert_eq!(
        generate(&f.root, &f.context, &ticket.run_id)
            .await
            .unwrap_err()
            .code,
        "runtime_attestation"
    );
    assert!(f.state.calls.lock().unwrap().is_empty());
    std::fs::write(
        &path,
        canonical_bytes(
            &json!({"pid":f.state.pid,"port":f.state.port,"attestationSecret":f.state.secret}),
        )
        .unwrap(),
    )
    .unwrap();
    *f.state.response.lock().unwrap() = json!({"output_text":"x".repeat(4*1024*1024)});
    let ticket = prepare_run(&f.root, &f.context, f.intent(Scene::UpdateModels)).unwrap();
    assert_eq!(
        generate(&f.root, &f.context, &ticket.run_id)
            .await
            .unwrap_err()
            .code,
        "model_limit"
    );
    assert!(!f.root.join(LIBRARY_PATH).exists());
}
