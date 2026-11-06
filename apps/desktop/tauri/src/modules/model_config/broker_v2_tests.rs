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
    #[cfg(unix)]
    let value = run_external_cli(&f, &bundle, &canonical_bytes(&report).unwrap(), "submit").await;
    #[cfg(not(unix))]
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
    #[cfg(not(unix))]
    let submission: Value = serde_json::from_slice(&body).unwrap();
    #[cfg(unix)]
    let before = run_external_cli(&f, &active, &body, "submit").await;
    #[cfg(not(unix))]
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
    #[cfg(not(unix))]
    let id = submission["submissionId"].as_str().unwrap();
    #[cfg(unix)]
    let recovered = run_external_cli(&f, &active, &body, "receipt").await;
    #[cfg(not(unix))]
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

// Executable standard-library reference consumer for the copied JSON block.
// Evidence exports this exact text; production does not depend on Python.
#[cfg(unix)]
const EXTERNAL_CLI_REFERENCE: &str = r###"
import hashlib, hmac, http.client, json, pathlib, re, sys, uuid

def require(condition):
    if not condition:
        raise ValueError('external task contract mismatch')

def main():
    args = json.load(sys.stdin)
    bundle = args['bundle']
    task, transport = bundle['task'], bundle['transport']
    task_id = task['taskId']
    require(str(uuid.UUID(task_id)) == task_id)
    bearer_header = transport['headers']['Authorization']
    require(bearer_header.startswith('Bearer '))
    bearer = bearer_header[7:]
    require(re.fullmatch('[0-9a-fA-F]{64}', bearer) is not None)
    descriptor_path = pathlib.Path(transport['discoveryFile'])
    body = pathlib.Path(args['submissionPath']).read_bytes()
    require(len(body) <= 4 * 1024 * 1024)
    submission = json.loads(body)
    require(submission['contractVersion'] == 2)
    submission_id = submission['submissionId']
    require(str(uuid.UUID(submission_id)) == submission_id)

    def request(method, suffix, data=None):
        descriptor_bytes = descriptor_path.read_bytes()
        descriptor = json.loads(descriptor_bytes)
        authority, pid = descriptor['authority'], descriptor['pid']
        require(descriptor['version'] == 1 and type(pid) is int and pid > 0)
        require(re.fullmatch(r'127\.0\.0\.1:[0-9]{1,5}', authority) is not None)
        port = int(authority.split(':')[1])
        require(0 < port < 65536)
        # Literal loopback only. HTTPConnection has no environment proxy or
        # automatic redirect/retry support. Keep the proven socket for bearer.
        conn = http.client.HTTPConnection('127.0.0.1', port, timeout=5)
        base = '/v1/tasks/' + task_id
        def response():
            reply = conn.getresponse()
            payload = reply.read(4 * 1024 * 1024 + 1)
            require(200 <= reply.status < 300 and len(payload) <= 4 * 1024 * 1024)
            return json.loads(payload)
        try:
            challenge = str(uuid.uuid4())
            conn.request('POST', base + '/proof', json.dumps({'challenge':challenge}).encode(), {'Content-Type':'application/json'})
            proof = response()
            require(proof['taskId'] == task_id and proof['challenge'] == challenge
                    and proof['authority'] == authority and proof['pid'] == pid)
            key = ('sha256:' + hashlib.sha256(bearer.encode()).hexdigest()).encode()
            message = f'ocxd-model-task-v1\n{task_id}\n{challenge}\n{authority}\n{pid}'.encode()
            require(hmac.compare_digest(proof['proof'], hmac.new(key, message, hashlib.sha256).hexdigest()))
            require(descriptor_path.read_bytes() == descriptor_bytes and conn.sock is not None)
            conn.request(method, base + suffix, data, {'Authorization':bearer_header,'Content-Type':'application/json'})
            return response()
        finally:
            conn.close()

    action = sys.argv[1]
    if action == 'submit':
        context = request('GET', '/context')
        contract = request('GET', '/contract')
        require(context == task and contract == task['contract'])
        require(contract['submission']['properties']['contractVersion']['type'] == 'integer')
        preflight = request('POST', '/preflight', body)
        require(preflight['dtoVersion'] == 2 and preflight['runId'] == task_id
                and preflight['expectedLibrary'] == task['expectedLibrary'])
        if task['scene'] == 'check_only':
            require(preflight['review'] is None and bool(preflight['report']['summary'].strip()))
        else:
            require(preflight['review']['valid'] is True)
        receipt = request('POST', '/submit', body)
        print('external-cli: proof/context/contract/preflight/submit verified', file=sys.stderr)
    elif action == 'receipt':
        # Recovery performs no POST /submit and uses the current descriptor,
        # not the stale exported baseUrl, so a moved port is recoverable.
        receipt = request('GET', '/receipts/' + submission_id)
        print('external-cli: proof/receipt recovered without submit', file=sys.stderr)
    else:
        raise ValueError('unsupported external CLI action')
    if receipt is not None:
        require(receipt['taskId'] == task_id and receipt['submissionId'] == submission_id)
        require(receipt['payloadDigest'] == 'sha256:' + hashlib.sha256(body).hexdigest())
        require(receipt['result']['phase'] in ('review', 'accepted'))
    json.dump(receipt, sys.stdout)

try:
    main()
except Exception as error:
    # No bearer, ticket or full request in diagnostics.
    print('external-cli failure: ' + type(error).__name__, file=sys.stderr)
    sys.exit(1)
"###;

#[cfg(unix)]
async fn run_external_cli(
    f: &Fixture,
    bundle: &ExternalBrokerBundleV2,
    body: &[u8],
    action: &str,
) -> Value {
    use tokio::io::AsyncWriteExt;
    let payload = f
        ._temp
        .path()
        .join(format!("{}-request.json", bundle.task_id));
    crate::infrastructure::atomic_write::atomic_write(&payload, body, 0o600).unwrap();
    let mut child = tokio::process::Command::new("/usr/bin/python3")
        .args(["-c", EXTERNAL_CLI_REFERENCE, action])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let pid = child.id().unwrap();
    let input =
        serde_json::to_vec(&json!({"bundle":f.copied(bundle),"submissionPath":payload})).unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(&input).await.unwrap();
    drop(stdin);
    let output = tokio::time::timeout(Duration::from_secs(20), child.wait_with_output())
        .await
        .unwrap()
        .unwrap();
    eprintln!(
        "external-cli action={action} pid={pid} exit={:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr).trim()
    );
    assert!(output.status.success());
    serde_json::from_slice(&output.stdout).unwrap()
}
