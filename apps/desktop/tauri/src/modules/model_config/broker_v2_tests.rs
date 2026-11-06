use super::super::{
    agent_proposal_v2::{AgentProposalV2, ContentProposalV2},
    execution_directory_v2,
    schema_v2::{revision, RefV2},
    store_v2::{DRAFT_PATH as V2_DRAFT, LIBRARY_PATH as V2_LIBRARY},
    tasks::Scene,
};
use super::*;
use crate::{infrastructure::runtime_executable::FixedRuntimeExecutable, state::ProcessContext};
struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    context: ProcessContext,
    broker: Option<Broker>,
    client: reqwest::Client,
    reference: RefV2,
}
impl Fixture {
    async fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("D");
        let home = temp.path().join("H");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        let executable = temp.path().join("official-cli-never-launched");
        std::fs::write(&executable, b"fixture-no-inference").unwrap();
        let context = ProcessContext {
            runtime: FixedRuntimeExecutable::resolved(executable),
            working_directory: temp.path().into(),
            opencodex_home: home.clone(),
        };
        std::fs::write(home.join("config.json"), serde_json::to_vec(&json!({"providers":{"fixture":{"adapter":"openai-chat","baseUrl":"https://fixture.invalid","apiKey":"SYNTHETIC-KEY-NOT-IN-BUNDLE","models":["model/a"],"selectedModels":["model/a"]}}})).unwrap()).unwrap();
        let saved = execution_directory_v2::capture_saved(&root, &context).unwrap();
        let template = &saved.library().templates[0];
        let old = &template.revisions[0];
        let reference = RefV2 {
            template_id: template.id.clone(),
            revision_id: old.id.clone(),
            digest: old.digest.clone(),
        };
        let broker = start(root.clone()).await.unwrap();
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        Self {
            _temp: temp,
            root,
            context,
            broker: Some(broker),
            client,
            reference,
        }
    }
    fn intent(&self, scene: Scene) -> v2::GenerateIntentV2 {
        v2::GenerateIntentV2 {
            scene,
            targets: vec![self.reference.clone()],
            allow_new: false,
            expected_library: execution_directory_v2::capture_saved(&self.root, &self.context)
                .unwrap()
                .library_digest()
                .unwrap(),
            requirements: "v2 scoped HTTP task".into(),
            history: vec![],
            sources: vec![],
            buffer: None,
        }
    }
    fn bundle(&self, scene: Scene) -> ExternalBrokerBundleV2 {
        prepare_bundle_at(
            &self.root,
            &self.context,
            self.intent(scene),
            &self.broker.as_ref().unwrap().endpoint,
        )
        .unwrap()
    }
    fn base(&self, bundle: &ExternalBrokerBundleV2) -> String {
        format!(
            "{}/v1/tasks/{}",
            self.broker.as_ref().unwrap().endpoint,
            bundle.task_id
        )
    }
    fn copied(&self, bundle: &ExternalBrokerBundleV2) -> Value {
        serde_json::from_str(bundle.content.split_once("\n\n").unwrap().1).unwrap()
    }
    fn bearer(&self, bundle: &ExternalBrokerBundleV2) -> String {
        self.copied(bundle)["transport"]["headers"]["Authorization"]
            .as_str()
            .unwrap()
            .strip_prefix("Bearer ")
            .unwrap()
            .into()
    }
    fn submission(&self) -> Vec<u8> {
        let saved = execution_directory_v2::capture_saved(&self.root, &self.context).unwrap();
        let old = revision(saved.library(), &self.reference).unwrap();
        let mut content = old.content.clone();
        content.description = "proposal through authenticated same listener".into();
        canonical_bytes(&v2::ExternalSubmissionV2 {
            contract_version: 2,
            submission_id: uuid::Uuid::new_v4().to_string(),
            response: serde_json::to_value(AgentProposalV2 {
                contract_version: 2,
                expected_library: saved.library_digest().unwrap(),
                proposals: vec![ContentProposalV2 {
                    target: Some(self.reference.clone()),
                    label: old.saved_label.clone(),
                    content_version: 2,
                    content,
                }],
            })
            .unwrap(),
        })
        .unwrap()
    }
    fn activate(&self, bundle: &ExternalBrokerBundleV2) {
        activate_external(
            &self.root,
            &self.context,
            &bundle.task_id,
            &bundle.prepared_ticket,
        )
        .unwrap();
    }
}
#[tokio::test]
async fn activation_proof_origin_host_and_scoped_bearer_use_the_existing_listener() {
    use hmac::{Hmac, Mac};
    let f = Fixture::new().await;
    let bundle = f.bundle(Scene::UpdateModels);
    let base = f.base(&bundle);
    let bearer = f.bearer(&bundle);
    assert!(!bundle.content.contains(&bundle.prepared_ticket));
    assert!(!bundle.content.contains("SYNTHETIC-KEY-NOT-IN-BUNDLE"));
    assert_eq!(
        f.client
            .post(format!("{base}/submit"))
            .bearer_auth(&bearer)
            .body(f.submission())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert!(activate_external(&f.root, &f.context, &bundle.task_id, &random_token()).is_err());
    assert!(!read_grant(&f.root, &bundle.task_id).unwrap().active);
    f.activate(&bundle);
    for bad in [
        f.client.get(format!("{base}/context")),
        f.client
            .get(format!("{base}/context"))
            .bearer_auth(random_token()),
        f.client
            .get(format!("{base}/context"))
            .bearer_auth(&bearer)
            .header("Origin", "https://untrusted.invalid"),
        f.client
            .get(format!("{base}/context"))
            .bearer_auth(&bearer)
            .header("Host", "rebound.invalid"),
    ] {
        assert_eq!(bad.send().await.unwrap().status(), StatusCode::FORBIDDEN);
    }
    let other = f.bundle(Scene::UpdateModels);
    f.activate(&other);
    assert_eq!(
        f.client
            .get(format!("{}/context", f.base(&other)))
            .bearer_auth(&bearer)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    let context: Value = f
        .client
        .get(format!("{base}/context"))
        .bearer_auth(&bearer)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(context["taskId"], bundle.task_id);
    assert_eq!(context["dtoVersion"], 2);
    assert!(context.get("token").is_none());
    let contract: Value = f
        .client
        .get(format!("{base}/contract"))
        .bearer_auth(&bearer)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(contract["contractVersion"], 2);
    let challenge = uuid::Uuid::new_v4().to_string();
    let proof: Value = f
        .client
        .post(format!("{base}/proof"))
        .json(&json!({"challenge":challenge}))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    let descriptor: Descriptor =
        serde_json::from_slice(&std::fs::read(f.root.join(DESCRIPTOR)).unwrap()).unwrap();
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(
        super::super::store::sha256(bearer.as_bytes()).as_bytes(),
    )
    .unwrap();
    mac.update(
        format!(
            "ocxd-model-task-v1\n{}\n{challenge}\n{}\n{}",
            bundle.task_id, descriptor.authority, descriptor.pid
        )
        .as_bytes(),
    );
    assert_eq!(proof["proof"], format!("{:x}", mac.finalize().into_bytes()));
    assert_eq!(proof["taskId"], bundle.task_id);
    assert_eq!(proof["challenge"], challenge);
    let bytes = std::fs::read(grant_path(&f.root, &bundle.task_id).unwrap()).unwrap();
    assert!(!String::from_utf8(bytes).unwrap().contains(&bearer));
    revoke_external(&f.root, &bundle.task_id).unwrap();
    assert_eq!(
        f.client
            .get(format!("{base}/context"))
            .bearer_auth(&bearer)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        f.client
            .post(format!("{base}/proof"))
            .json(&json!({"challenge":uuid::Uuid::new_v4().to_string()}))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert!(!f.root.join(V2_LIBRARY).exists());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(grant_path(&f.root, &bundle.task_id).unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
#[tokio::test]
async fn strict_raw_v2_submit_creates_review_receipts_and_preserves_same_id_bytes() {
    let f = Fixture::new().await;
    let bundle = f.bundle(Scene::UpdateModels);
    f.activate(&bundle);
    let base = f.base(&bundle);
    let bearer = f.bearer(&bundle);
    let bytes = f.submission();
    let config = std::fs::read(f.context.opencodex_home.join("config.json")).unwrap();
    let grant_before = std::fs::read(grant_path(&f.root, &bundle.task_id).unwrap()).unwrap();
    let preflight: Value = f
        .client
        .post(format!("{base}/preflight"))
        .bearer_auth(&bearer)
        .body(bytes.clone())
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(preflight["review"]["valid"], true);
    assert_eq!(
        grant_before,
        std::fs::read(grant_path(&f.root, &bundle.task_id).unwrap()).unwrap()
    );
    let receipt: Value = f
        .client
        .post(format!("{base}/submit"))
        .bearer_auth(&bearer)
        .body(bytes.clone())
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(receipt["result"]["phase"], "review");
    assert_eq!(
        receipt["payloadDigest"],
        super::super::store::sha256(&bytes)
    );
    assert!(!f.root.join(V2_LIBRARY).exists());
    assert!(!f.root.join(V2_DRAFT).exists());
    let replay: Value = f
        .client
        .post(format!("{base}/submit"))
        .bearer_auth(&bearer)
        .body(bytes.clone())
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(receipt, replay);
    let submission: Value = serde_json::from_slice(&bytes).unwrap();
    let id = submission["submissionId"].as_str().unwrap();
    let fetched: Value = f
        .client
        .get(format!("{base}/receipts/{id}"))
        .bearer_auth(&bearer)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(receipt, fetched);
    let mut whitespace = vec![b' '];
    whitespace.extend_from_slice(&bytes);
    assert_eq!(
        f.client
            .post(format!("{base}/submit"))
            .bearer_auth(&bearer)
            .body(whitespace)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::CONFLICT
    );
    let v1 = json!({"contractVersion":1,"submissionId":id,"proposals":[],"report":"legacy"});
    let rejected: Value = f
        .client
        .post(format!("{base}/submit"))
        .bearer_auth(&bearer)
        .json(&v1)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(rejected["code"], "task_contract_unsupported");
    assert_eq!(
        config,
        std::fs::read(f.context.opencodex_home.join("config.json")).unwrap()
    );
    assert!(!f.root.join(LIBRARY_PATH).exists());
}

#[tokio::test]
async fn reports_are_review_only_and_strict_json_scope_and_body_limits_fail_closed() {
    let f = Fixture::new().await;
    let bundle = f.bundle(Scene::CheckOnly);
    f.activate(&bundle);
    let base = f.base(&bundle);
    let bearer = f.bearer(&bundle);
    let library = f.intent(Scene::CheckOnly).expected_library;
    let report = json!({"contractVersion":2,"submissionId":uuid::Uuid::new_v4().to_string(),"response":{"contractVersion":2,"expectedLibrary":library,"report":{"summary":"bounded HTTP check","issues":[]}}});
    let mut unknown = report.clone();
    unknown["origin"] = json!("builtin");
    let mut executable = report.clone();
    executable["response"]["report"]["command"] = json!("shell");
    let duplicate=br#"{"contractVersion":2,"contractVersion":2,"submissionId":"11111111-1111-4111-8111-111111111111","response":{}}"#;
    for body in [
        canonical_bytes(&unknown).unwrap(),
        canonical_bytes(&executable).unwrap(),
        duplicate.to_vec(),
    ] {
        assert!(f
            .client
            .post(format!("{base}/submit"))
            .bearer_auth(&bearer)
            .body(body)
            .send()
            .await
            .unwrap()
            .status()
            .is_client_error());
        assert!(read_grant(&f.root, &bundle.task_id)
            .unwrap()
            .payload
            .is_none());
    }
    assert_eq!(
        f.client
            .post(format!("{base}/submit"))
            .bearer_auth(&bearer)
            .body(vec![b' '; 4 * 1024 * 1024 + 1])
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );
    let value: Value = f
        .client
        .post(format!("{base}/submit"))
        .bearer_auth(&bearer)
        .json(&report)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(value["result"]["review"].is_null());
    assert_eq!(value["result"]["report"]["summary"], "bounded HTTP check");
    assert!(
        with_external_authority(&f.root, &f.context, &bundle.task_id, || v2::prepare_accept(
            &f.root,
            &f.context,
            &bundle.task_id,
            &library,
            &canonical_bytes(&report).unwrap()
        )
        .map_err(v2_error))
        .is_err()
    );
    assert!(!f.root.join(V2_LIBRARY).exists());
    assert!(!f.root.join(V2_DRAFT).exists());
    let bundle = f.bundle(Scene::UpdateModels);
    f.activate(&bundle);
    let mut invalid: Value = serde_json::from_slice(&f.submission()).unwrap();
    invalid["response"]["proposals"][0]["target"]["digest"] = json!("sha256:wrong");
    assert!(f
        .client
        .post(format!("{}/submit", f.base(&bundle)))
        .bearer_auth(f.bearer(&bundle))
        .json(&invalid)
        .send()
        .await
        .unwrap()
        .status()
        .is_client_error());
    assert!(read_grant(&f.root, &bundle.task_id)
        .unwrap()
        .payload
        .is_none());
}
#[tokio::test]
async fn fixed_context_expiry_and_old_persisted_tasks_do_not_gain_v2_authority() {
    let f = Fixture::new().await;
    let bundle = f.bundle(Scene::UpdateModels);
    let mut wrong = f.context.clone();
    wrong.working_directory = f.root.clone();
    assert!(activate_external(&f.root, &wrong, &bundle.task_id, &bundle.prepared_ticket).is_err());
    assert!(!read_grant(&f.root, &bundle.task_id).unwrap().active);
    let config = f.context.opencodex_home.join("config.json");
    let before = std::fs::read(&config).unwrap();
    let mut changed: Value = serde_json::from_slice(&before).unwrap();
    changed["providers"]["fixture"]["baseUrl"] = json!("https://stale.invalid");
    std::fs::write(&config, canonical_bytes(&changed).unwrap()).unwrap();
    assert_eq!(
        activate_external(
            &f.root,
            &f.context,
            &bundle.task_id,
            &bundle.prepared_ticket
        )
        .unwrap_err()
        .code,
        "task_binding"
    );
    std::fs::write(&config, before).unwrap();
    f.activate(&bundle);
    let base = f.base(&bundle);
    let bearer = f.bearer(&bundle);
    let mut expired = read_grant(&f.root, &bundle.task_id).unwrap();
    expired.expires_at = "2000-01-01T00:00:00Z".into();
    persist_grant(&f.root, &expired).unwrap();
    for request in [
        f.client.get(format!("{base}/context")).bearer_auth(&bearer),
        f.client
            .get(format!("{base}/contract"))
            .bearer_auth(&bearer),
        f.client
            .post(format!("{base}/submit"))
            .bearer_auth(&bearer)
            .body(f.submission()),
        f.client
            .post(format!("{base}/proof"))
            .json(&json!({"challenge":uuid::Uuid::new_v4().to_string()})),
    ] {
        assert_eq!(
            request.send().await.unwrap().status(),
            StatusCode::FORBIDDEN
        );
    }
    let legacy_id = uuid::Uuid::new_v4().to_string();
    let legacy = f
        .root
        .join("manager-state/model-template-tasks")
        .join(format!("{legacy_id}.json"));
    std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    std::fs::write(
        &legacy,
        canonical_bytes(&json!({"task":{"contractVersion":1}})).unwrap(),
    )
    .unwrap();
    let old_base = format!(
        "{}/v1/tasks/{legacy_id}",
        f.broker.as_ref().unwrap().endpoint
    );
    let result: Value = f
        .client
        .get(format!("{old_base}/context"))
        .bearer_auth(random_token())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(result["code"], "task_contract_unsupported");
    assert_eq!(
        create_bundle(
            f.root.clone(),
            CreateTask {
                scene: Scene::FromSources,
                targets: vec![],
                allow_new: true,
                requirements: "legacy rejected".into(),
                expected_library: None,
                buffers: vec![]
            }
        )
        .await
        .err()
        .unwrap()
        .code,
        "task_contract_unsupported"
    );
    assert!(!f.root.join(LIBRARY_PATH).exists());
    assert!(!f.root.join(V2_LIBRARY).exists());
    let mut duplicate = reqwest::header::HeaderMap::new();
    duplicate.append("authorization", format!("Bearer {bearer}").parse().unwrap());
    duplicate.append(
        "authorization",
        format!("Bearer {}", random_token()).parse().unwrap(),
    );
    assert_eq!(
        f.client
            .get(format!("{base}/context"))
            .headers(duplicate)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
}
#[tokio::test]
async fn restart_reuses_one_descriptor_recovers_receipts_and_never_activates_prepared_tasks() {
    let mut f = Fixture::new().await;
    let prepared = f.bundle(Scene::UpdateModels);
    let active = f.bundle(Scene::UpdateModels);
    f.activate(&active);
    let bearer = f.bearer(&active);
    let unactivated = f.bearer(&prepared);
    let body = f.submission();
    let submission: Value = serde_json::from_slice(&body).unwrap();
    let before: Value = f
        .client
        .post(format!("{}/submit", f.base(&active)))
        .bearer_auth(&bearer)
        .body(body)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    let original = f
        .broker
        .as_ref()
        .unwrap()
        .endpoint
        .trim_start_matches("http://")
        .parse::<std::net::SocketAddr>()
        .unwrap();
    drop(f.broker.take());
    tokio::task::yield_now().await;
    let mut occupied = None;
    for _ in 0..20 {
        if let Ok(listener) = tokio::net::TcpListener::bind(original).await {
            occupied = Some(listener);
            break;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let _occupied = occupied.expect("old listener releases its port");
    f.broker = Some(start(f.root.clone()).await.unwrap());
    assert_ne!(
        f.broker.as_ref().unwrap().endpoint,
        format!("http://{original}")
    );
    // Locate the resumed listener and prove its identity before forwarding the
    // active task's bearer, using the original v1 transport HMAC domain.
    use hmac::{Hmac, Mac};
    let descriptor: Descriptor =
        serde_json::from_slice(&std::fs::read(f.root.join(DESCRIPTOR)).unwrap()).unwrap();
    let challenge = uuid::Uuid::new_v4().to_string();
    let proof: Value = f
        .client
        .post(format!("{}/proof", f.base(&active)))
        .json(&json!({"challenge":challenge}))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(
        super::super::store::sha256(bearer.as_bytes()).as_bytes(),
    )
    .unwrap();
    mac.update(
        format!(
            "ocxd-model-task-v1\n{}\n{challenge}\n{}\n{}",
            active.task_id, descriptor.authority, descriptor.pid
        )
        .as_bytes(),
    );
    assert_eq!(proof["proof"], format!("{:x}", mac.finalize().into_bytes()));
    assert_eq!(proof["authority"], descriptor.authority);
    assert_eq!(proof["pid"], descriptor.pid);
    assert_eq!(proof["taskId"], active.task_id);
    assert_eq!(proof["challenge"], challenge);
    assert_eq!(
        f.client
            .post(format!("{}/submit", f.base(&prepared)))
            .bearer_auth(unactivated)
            .body(f.submission())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert!(!read_grant(&f.root, &prepared.task_id).unwrap().active);
    let id = submission["submissionId"].as_str().unwrap();
    let recovered: Value = f
        .client
        .get(format!("{}/receipts/{id}", f.base(&active)))
        .bearer_auth(&bearer)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(before, recovered);
    let descriptor: Descriptor =
        serde_json::from_slice(&std::fs::read(f.root.join(DESCRIPTOR)).unwrap()).unwrap();
    assert_eq!(descriptor.version, 1);
    assert_eq!(
        f.broker.as_ref().unwrap().endpoint,
        format!("http://{}", descriptor.authority)
    );
    assert!(!f.root.join(V2_LIBRARY).exists());
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn final_authority_guard_serializes_revocation_with_acceptance() {
    let f = Fixture::new().await;
    let bundle = f.bundle(Scene::UpdateModels);
    f.activate(&bundle);
    let root = f.root.clone();
    let context = f.context.clone();
    let id = bundle.task_id.clone();
    let (entered_tx, entered_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let save = tokio::task::spawn_blocking(move || {
        with_external_authority(&root, &context, &id, || {
            entered_tx.send(()).unwrap();
            release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            Ok(())
        })
    });
    tokio::time::timeout(Duration::from_secs(5), entered_rx)
        .await
        .unwrap()
        .unwrap();
    let root = f.root.clone();
    let id = bundle.task_id.clone();
    let mut revoke = tokio::task::spawn_blocking(move || revoke_external(&root, &id));
    assert!(tokio::time::timeout(Duration::from_millis(30), &mut revoke)
        .await
        .is_err());
    release_tx.send(()).unwrap();
    save.await.unwrap().unwrap();
    revoke.await.unwrap().unwrap();
    assert!(with_external_authority(&f.root, &f.context, &bundle.task_id, || Ok(())).is_err());
    assert!(!f.root.join(V2_LIBRARY).exists());
}
#[cfg(unix)]
#[tokio::test]
async fn private_grants_reject_root_rebinding_unknown_schema_duplicates_and_symlinks() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new().await;
    let bundle = f.bundle(Scene::UpdateModels);
    let path = grant_path(&f.root, &bundle.task_id).unwrap();
    let original = std::fs::read(&path).unwrap();
    let mut value: Value = serde_json::from_slice(&original).unwrap();
    value["rootBinding"] = json!("sha256:other-root");
    std::fs::write(&path, canonical_bytes(&value).unwrap()).unwrap();
    assert_eq!(
        activate_external(
            &f.root,
            &f.context,
            &bundle.task_id,
            &bundle.prepared_ticket
        )
        .unwrap_err()
        .code,
        "task_binding"
    );
    value = serde_json::from_slice(&original).unwrap();
    value["schemaVersion"] = json!(99);
    std::fs::write(&path, canonical_bytes(&value).unwrap()).unwrap();
    assert_eq!(
        read_grant(&f.root, &bundle.task_id).unwrap_err().code,
        "task_contract_unsupported"
    );
    std::fs::write(&path, br#"{"schemaVersion":2,"schemaVersion":2}"#).unwrap();
    assert!(read_grant(&f.root, &bundle.task_id).is_err());
    std::fs::remove_file(&path).unwrap();
    let outside = f._temp.path().join("outside-grant.json");
    std::fs::write(&outside, &original).unwrap();
    symlink(&outside, &path).unwrap();
    assert!(read_grant(&f.root, &bundle.task_id).is_err());
    assert_eq!(std::fs::read(&outside).unwrap(), original);
    assert!(!f.root.join(V2_LIBRARY).exists());
}

#[tokio::test]
async fn public_bundle_and_resume_share_one_listener_without_implicit_activation() {
    let mut f = Fixture::new().await;
    drop(f.broker.take());
    let bundle = prepare_external_broker(
        f.root.clone(),
        f.context.clone(),
        f.intent(Scene::UpdateModels),
    )
    .await
    .unwrap();
    let endpoint = ensure(f.root.clone()).await.unwrap();
    let another = prepare_external_broker(
        f.root.clone(),
        f.context.clone(),
        f.intent(Scene::UpdateModels),
    )
    .await
    .unwrap();
    assert_eq!(
        f.copied(&bundle)["transport"]["baseUrl"],
        format!("{endpoint}/v1/tasks/{}", bundle.task_id)
    );
    assert_eq!(
        f.copied(&another)["transport"]["baseUrl"],
        format!("{endpoint}/v1/tasks/{}", another.task_id)
    );
    let bearer = f.bearer(&bundle);
    {
        let mut current = BROKER.get().unwrap().lock().await;
        drop(current.take());
    }
    resume(f.root.clone());
    let resumed = ensure(f.root.clone()).await.unwrap();
    assert!(!read_grant(&f.root, &bundle.task_id).unwrap().active);
    assert_eq!(
        f.client
            .post(format!("{resumed}/v1/tasks/{}/submit", bundle.task_id))
            .bearer_auth(bearer)
            .body(f.submission())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    let mut current = BROKER.get().unwrap().lock().await;
    drop(current.take());
}

#[tokio::test]
async fn revoking_an_accepted_task_withdraws_transport_without_undoing_the_revision() {
    let f = Fixture::new().await;
    let bundle = f.bundle(Scene::UpdateModels);
    f.activate(&bundle);
    let base = f.base(&bundle);
    let bearer = f.bearer(&bundle);
    let body = f.submission();
    let submission: Value = serde_json::from_slice(&body).unwrap();
    f.client
        .post(format!("{base}/submit"))
        .bearer_auth(&bearer)
        .body(body)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let response = canonical_bytes(&submission["response"]).unwrap();
    let expected = submission["response"]["expectedLibrary"].as_str().unwrap();
    // Simulate only the owner commit in this temp root. L0 owns the real
    // library/version transaction; HTTP submit above still only made Review.
    assert!(!f.root.join(V2_LIBRARY).exists());
    let accepted = with_external_authority(&f.root, &f.context, &bundle.task_id, || {
        let plan = v2::prepare_accept(&f.root, &f.context, &bundle.task_id, expected, &response)
            .map_err(v2_error)?;
        std::fs::write(
            f.root.join(V2_LIBRARY),
            canonical_bytes(&plan.library).map_err(v2_error)?,
        )?;
        v2::mark_accepted(&f.root, &f.context, &bundle.task_id, &plan.response_digest)
            .map_err(v2_error)
    })
    .unwrap();
    assert_eq!(accepted.phase, v2::RunPhaseV2::Accepted);
    let library = std::fs::read(f.root.join(V2_LIBRARY)).unwrap();
    revoke_external(&f.root, &bundle.task_id).unwrap();
    assert_eq!(
        f.client
            .get(format!("{base}/context"))
            .bearer_auth(&bearer)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        v2::latest(&f.root, None).unwrap().unwrap().phase,
        v2::RunPhaseV2::Accepted
    );
    assert_eq!(std::fs::read(f.root.join(V2_LIBRARY)).unwrap(), library);
}
