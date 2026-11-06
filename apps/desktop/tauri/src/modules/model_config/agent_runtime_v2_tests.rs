//! Unix official-CLI fixture; native Windows execution remains L4-owned.
#![cfg(unix)]
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
    let context = proposal_context(&record).unwrap();
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

fn external_wire(proposal: &AgentProposalV2, submission_id: &str) -> Vec<u8> {
    canonical_bytes(&ExternalSubmissionV2 {
        contract_version: 2,
        submission_id: submission_id.into(),
        response: serde_json::to_value(proposal).unwrap(),
    })
    .unwrap()
}
#[cfg(unix)]
#[tokio::test]
async fn external_facade_is_data_only_idempotent_and_uses_trusted_external_origin() {
    let f = Fixture::new().await;
    // External tasks have no dependency on internal model preferences or a
    // reachable inference service. The existing broker auth is L0-owned.
    std::fs::remove_file(
        f.root
            .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH),
    )
    .unwrap();
    f.server.abort();
    let proposal = f.proposal();
    let task = prepare_external(&f.root, &f.context, f.intent(Scene::UpdateModels)).unwrap();
    assert_eq!(task.dto_version, 2);
    assert_eq!(task.targets[0].reference, f.reference);
    assert_eq!(task.contract["contractVersion"], 2);
    let record = read_record(&f.root, &task.task_id).unwrap();
    assert!(record.settings.is_none());
    assert_eq!(record.executor, ExecutorV2::External);
    let context_bytes = canonical_bytes(&task).unwrap();
    assert!(!String::from_utf8(context_bytes)
        .unwrap()
        .contains("SOURCE-KEY-MUST-NOT-ENTER-PROMPT"));
    let submission_id = uuid::Uuid::new_v4().to_string();
    let bytes = external_wire(&proposal, &submission_id);
    let before = std::fs::read(record_path(&f.root, &task.task_id).unwrap()).unwrap();
    let preflight = preflight_external(&f.root, &f.context, &task.task_id, &bytes).unwrap();
    assert!(preflight.review.unwrap().valid);
    assert_eq!(
        before,
        std::fs::read(record_path(&f.root, &task.task_id).unwrap()).unwrap()
    );
    assert!(receipt_external(&f.root, &task.task_id, &submission_id)
        .unwrap()
        .is_none());
    let receipt = submit_external_review(&f.root, &f.context, &task.task_id, &bytes).unwrap();
    assert_eq!(receipt.result.phase, RunPhaseV2::Review);
    assert!(!f.root.join(LIBRARY_PATH).exists());
    assert!(!f.root.join(DRAFT_PATH).exists());
    let retry = submit_external_review(&f.root, &f.context, &task.task_id, &bytes).unwrap();
    assert_eq!(
        canonical_bytes(&receipt).unwrap(),
        canonical_bytes(&retry).unwrap()
    );
    assert_eq!(
        receipt_external(&f.root, &task.task_id, &submission_id)
            .unwrap()
            .unwrap()
            .submission_digest,
        receipt.submission_digest
    );
    let mut changed = proposal.clone();
    changed.proposals[0].content.description.push_str("changed");
    assert_eq!(
        submit_external_review(
            &f.root,
            &f.context,
            &task.task_id,
            &external_wire(&changed, &submission_id)
        )
        .unwrap_err()
        .code,
        "model_agent_submission_conflict"
    );
    let plan = prepare_accept(
        &f.root,
        &f.context,
        &task.task_id,
        &task.expected_library,
        &canonical_bytes(&proposal).unwrap(),
    )
    .unwrap();
    let new = plan.review.proposals[0].planned_revision.as_ref().unwrap();
    assert!(
        matches!(&revision(&plan.library, new).unwrap().origin, super::super::schema::SaveOrigin::ExternalTask(id) if id == &task.task_id)
    );
    assert!(f.state.calls.lock().unwrap().is_empty());
    revoke_external(&f.root, &task.task_id).unwrap();
    assert_eq!(
        submit_external_review(&f.root, &f.context, &task.task_id, &bytes)
            .unwrap_err()
            .code,
        "model_agent_run_state"
    );
    assert!(prepare_accept(
        &f.root,
        &f.context,
        &task.task_id,
        &task.expected_library,
        &canonical_bytes(&proposal).unwrap()
    )
    .is_err());
}
#[cfg(unix)]
#[tokio::test]
async fn external_report_expiry_stale_state_and_unknown_fields_are_enforced() {
    let f = Fixture::new().await;
    let task = prepare_external(&f.root, &f.context, f.intent(Scene::CheckOnly)).unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let value = json!({"contractVersion":2,"submissionId":id,"response":{"contractVersion":2,"expectedLibrary":task.expected_library,"report":{"summary":"external checked","issues":[]}}});
    let bytes = canonical_bytes(&value).unwrap();
    let receipt = submit_external_review(&f.root, &f.context, &task.task_id, &bytes).unwrap();
    assert!(receipt.result.review.is_none());
    assert_eq!(receipt.result.report.unwrap().summary, "external checked");
    assert!(prepare_accept(
        &f.root,
        &f.context,
        &task.task_id,
        &task.expected_library,
        &bytes
    )
    .is_err());
    let mut bad = value.clone();
    bad["origin"] = json!("internal_agent");
    assert!(external_submission(&canonical_bytes(&bad).unwrap()).is_err());
    assert!(external_submission(
        br#"{"contractVersion":2,"contractVersion":2,"submissionId":"x","response":{}}"#
    )
    .is_err());
    let mut record = read_record(&f.root, &task.task_id).unwrap();
    record.expires_at = "2000-01-01T00:00:00Z".into();
    persist(&f.root, &record).unwrap();
    assert_eq!(
        receipt_external(&f.root, &task.task_id, &id)
            .unwrap_err()
            .code,
        "model_agent_expired"
    );
    assert_eq!(
        external_context(&f.root, &task.task_id).unwrap_err().code,
        "model_agent_expired"
    );
    let task = prepare_external(&f.root, &f.context, f.intent(Scene::UpdateModels)).unwrap();
    let mut proposal = f.proposal();
    let id = uuid::Uuid::new_v4().to_string();
    // A preflight never silently replaces a stale exact target.
    proposal.proposals[0].target.as_mut().unwrap().digest = digest(b"wrong ref");
    assert!(preflight_external(
        &f.root,
        &f.context,
        &task.task_id,
        &external_wire(&proposal, &id),
    )
    .is_err());
    assert!(submit_external_review(
        &f.root,
        &f.context,
        &task.task_id,
        &external_wire(&proposal, &id)
    )
    .is_err());
    assert_eq!(
        read_record(&f.root, &task.task_id).unwrap().phase,
        RunPhaseV2::Prepared
    );
    let path = f.context.opencodex_home.join("config.json");
    let mut source: Value = parse_strict_json_v2(&std::fs::read(&path).unwrap()).unwrap();
    source["providers"]["fixture"]["baseUrl"] = json!("https://changed.invalid");
    std::fs::write(path, canonical_bytes(&source).unwrap()).unwrap();
    assert_eq!(
        preflight_external(
            &f.root,
            &f.context,
            &task.task_id,
            &external_wire(&proposal, &id)
        )
        .unwrap_err()
        .code,
        "model_save_stale"
    );
    assert!(!f.root.join(LIBRARY_PATH).exists());
    assert!(f.state.calls.lock().unwrap().is_empty());
}
#[test]
fn saved_to_buffer_cannot_remove_locks_change_protected_values_or_delete_locked_conditions() {
    let library = seed_library_v2().unwrap();
    let mut old = library.templates[0].revisions[0].content.clone();
    old.locked_fields.insert("description".into());
    old.condition_overrides[0]
        .locked_fields
        .insert("contextWindow".into());
    for mutation in 0..4 {
        let mut buffer = old.clone();
        match mutation {
            0 => {
                buffer.locked_fields.clear();
            }
            1 => {
                buffer.description.push_str("changed");
            }
            2 => {
                buffer.condition_overrides[0].locked_fields.clear();
            }
            _ => {
                buffer.condition_overrides.remove(0);
            }
        }
        assert_eq!(
            validate_editing_buffer_v2(Some(&old), &buffer)
                .unwrap_err()
                .code,
            "model_locked_field"
        );
    }
}

#[cfg(unix)]
#[tokio::test]
async fn external_only_create_buffer_appends_one_new_template_with_user_locks() {
    let f = Fixture::new().await;
    let mut proposal = f.proposal();
    proposal.proposals[0].target = None;
    proposal.proposals[0].label = "New human buffer template".into();
    proposal.proposals[0]
        .content
        .locked_fields
        .insert("description".into());
    let mut buffer = EditingBufferV2 {
        target: None,
        label: proposal.proposals[0].label.clone(),
        content: proposal.proposals[0].content.clone(),
        digest: String::new(),
        generation: 1,
    };
    buffer.digest = buffer_digest(&buffer).unwrap();
    let mut intent = f.intent(Scene::PrivateModels);
    intent.targets.clear();
    intent.allow_new = true;
    intent.buffer = Some(buffer);
    let task = prepare_external(&f.root, &f.context, intent).unwrap();
    let bytes = external_wire(&proposal, &uuid::Uuid::new_v4().to_string());
    submit_external_review(&f.root, &f.context, &task.task_id, &bytes).unwrap();
    // Owner review may edit an unlocked label. Receipt identity still refers
    // to the original submission, while the displayed review digest follows
    // the exact accepted bytes.
    proposal.proposals[0].label.push_str(" reviewed by owner");
    let plan = prepare_accept(
        &f.root,
        &f.context,
        &task.task_id,
        &task.expected_library,
        &canonical_bytes(&proposal).unwrap(),
    )
    .unwrap();
    let new_ref = plan.review.proposals[0].planned_revision.as_ref().unwrap();
    let new = revision(&plan.library, new_ref).unwrap();
    assert!(new.base_revision.is_none());
    assert!(new.content.locked_fields.contains("description"));
    assert_ne!(new_ref.template_id, f.reference.template_id);
    let mut wrong = plan.library.clone();
    wrong
        .templates
        .iter_mut()
        .find(|t| t.id == new_ref.template_id)
        .unwrap()
        .revisions[0]
        .origin = super::super::schema::SaveOrigin::Manual;
    std::fs::write(f.root.join(LIBRARY_PATH), canonical_bytes(&wrong).unwrap()).unwrap();
    assert_eq!(
        mark_accepted(&f.root, &f.context, &task.task_id, &plan.response_digest)
            .unwrap_err()
            .code,
        "model_agent_commit_unconfirmed"
    );
    std::fs::write(
        f.root.join(LIBRARY_PATH),
        canonical_bytes(&plan.library).unwrap(),
    )
    .unwrap();
    assert_eq!(
        mark_accepted(&f.root, &f.context, &task.task_id, &plan.response_digest)
            .unwrap()
            .phase,
        RunPhaseV2::Accepted
    );
    let receipt = receipt_external(
        &f.root,
        &task.task_id,
        &serde_json::from_slice::<ExternalSubmissionV2>(&bytes)
            .unwrap()
            .submission_id,
    )
    .unwrap()
    .unwrap();
    assert_eq!(receipt.result.phase, RunPhaseV2::Accepted);
    assert_ne!(receipt.submission_digest, plan.response_digest);
    assert_eq!(
        receipt.result.response_digest.as_deref(),
        Some(plan.response_digest.as_str())
    );

    assert!(f.state.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn executor_routing_comes_from_private_record_and_external_cannot_generate_internally() {
    let f = Fixture::new().await;
    let intent = f.intent(Scene::UpdateModels);
    let internal = prepare_run(&f.root, &f.context, intent.clone()).unwrap();
    assert!(!requires_external_authority(&f.root, &internal.run_id).unwrap());
    let external = prepare_external(&f.root, &f.context, intent.clone()).unwrap();
    assert!(requires_external_authority(&f.root, &external.task_id).unwrap());
    assert_eq!(
        generate(&f.root, &f.context, &external.task_id)
            .await
            .unwrap_err()
            .code,
        "model_agent_run_state"
    );
    assert_eq!(
        read_record(&f.root, &external.task_id).unwrap().phase,
        RunPhaseV2::Prepared
    );
    assert!(f.state.calls.lock().unwrap().is_empty());
    assert!(requires_external_authority(&f.root, &uuid::Uuid::new_v4().to_string()).is_err());
    let mut injected = serde_json::to_value(intent).unwrap();
    injected["executor"] = json!("internal");
    assert!(serde_json::from_value::<GenerateIntentV2>(injected).is_err());
}
