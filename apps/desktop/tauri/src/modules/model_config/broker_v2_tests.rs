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
