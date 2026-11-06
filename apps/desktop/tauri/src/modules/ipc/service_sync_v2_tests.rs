//! Real IPC validation/admission/audit + Service/Host/capture/receive/backup.
//! Only HTTP transport and endpoint credential retrieval are synthetic.
use super::*;
use crate::modules::model_config::{
    objects_overlay_v2::SelectionV2,
    saved_objects_capture_v2::SystemCaptureCredentialsV2,
    schema::ModelResult,
    sync_host_v2::EndpointCredentialsV2,
    sync_service_v2::{ProductionServiceBackendV2, ScopeRequestV2},
    sync_v2::types::*,
};
use crate::modules::sync::config::*;
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst},
};
use zeroize::Zeroizing;
const CREDS: &str = "cred_11111111-1111-4111-8111-111111111111";
const SECRET: &str = "synthetic-cli-v2-endpoint-secret";
const TIME: &str = "2026-11-07T00:00:00Z";

#[derive(Default)]
struct Remote {
    files: Mutex<BTreeMap<String, Vec<u8>>>,
    reads: AtomicUsize,
    writes: AtomicUsize,
    unauthorized: AtomicBool,
    fail_payload_readback: AtomicBool,
    hold_listing: AtomicBool,
    entered: tokio::sync::Notify,
    release: tokio::sync::Notify,
}
fn transport_error(kind: TransportKind) -> SyncTransportError {
    SyncTransportError {
        kind,
        http_status: None,
        delivery: Delivery::ResponseReceived,
    }
}
#[async_trait::async_trait]
impl ConfigSyncTransport for Remote {
    async fn list_children(
        &self,
        _: &ConnectionHandle,
        _: ListLimits,
    ) -> Result<ChildListing, SyncTransportError> {
        self.reads.fetch_add(1, SeqCst);
        if self.hold_listing.swap(false, SeqCst) {
            self.entered.notify_one();
            self.release.notified().await;
        }
        if self.unauthorized.load(SeqCst) {
            return Err(transport_error(TransportKind::Unauthorized));
        }
        let entries = self
            .files
            .lock()
            .unwrap()
            .iter()
            .map(|(name, b)| ChildEntry {
                name: name.clone(),
                is_collection: false,
                content_length: Some(b.len() as u64),
            })
            .collect();
        Ok(ChildListing {
            entries,
            http_status: 207,
            response_bytes: 100,
        })
    }
    async fn get_file(
        &self,
        _: &ConnectionHandle,
        p: &SyncRelativePath,
        _: ReadLimits,
    ) -> Result<FileRead, SyncTransportError> {
        self.reads.fetch_add(1, SeqCst);
        if self.fail_payload_readback.load(SeqCst) && p.as_str().ends_with(".ocxdconf") {
            return Ok(FileRead {
                body: b"bad readback".to_vec(),
                http_status: 200,
            });
        }
        self.files
            .lock()
            .unwrap()
            .get(p.as_str())
            .map(|b| FileRead {
                body: b.clone(),
                http_status: 200,
            })
            .ok_or_else(|| transport_error(TransportKind::NotFound))
    }
    async fn put_file(
        &self,
        _: &ConnectionHandle,
        p: &SyncRelativePath,
        b: &[u8],
        _: WriteLimits,
    ) -> Result<PutResponse, SyncTransportError> {
        assert!(
            p.as_str().starts_with("config-v2-"),
            "CLI must never call the v1 engine"
        );
        self.writes.fetch_add(1, SeqCst);
        self.files
            .lock()
            .unwrap()
            .insert(p.as_str().into(), b.to_vec());
        Ok(PutResponse { http_status: 201 })
    }
}
struct Credentials {
    d: PathBuf,
}
impl EndpointCredentialsV2 for Credentials {
    fn read_password(
        &self,
        d: &Path,
        endpoint: &SyncEndpointConfig,
    ) -> ModelResult<Zeroizing<String>> {
        assert_eq!(d, self.d);
        assert_eq!(endpoint.credential_ref.ref_id, CREDS);
        Ok(Zeroizing::new(SECRET.into()))
    }
}
struct Env {
    _temp: tempfile::TempDir,
    d: PathBuf,
    h: PathBuf,
    remote: Arc<Remote>,
    sync: Arc<SyncServiceV2>,
}
impl Env {
    fn new(remote: Arc<Remote>, model: bool) -> Self {
        let temp = tempfile::Builder::new()
            .prefix("ocxd023-cli-sync-v2-")
            .tempdir_in("/tmp")
            .unwrap();
        let d = temp.path().join("D");
        let h = temp.path().join("actual-H");
        std::fs::create_dir_all(d.join("manager-state")).unwrap();
        std::fs::create_dir_all(&h).unwrap();
        // A real valid empty saved library keeps this IPC integration fixture
        // bounded; repeated source checks need not rebuild the bundled catalog.
        std::fs::write(
            d.join(crate::modules::model_config::store_v2::LIBRARY_PATH),
            serde_json::to_vec(&crate::modules::model_config::schema_v2::LibraryV2 {
                schema_version: 2,
                templates: vec![],
                submission_receipts: BTreeMap::new(),
            })
            .unwrap(),
        )
        .unwrap();
        if model {
            std::fs::write(h.join("config.json"), br#"{"providers":{"cli-fixture":{"adapter":"openai-chat","baseUrl":"https://fixture.invalid/v1","authMode":"key","models":["cli-model"],"selectedModels":["cli-model"]}}}"#).unwrap();
        }
        let keychain = &crate::infrastructure::keychain::keychain_service_name();
        SyncConfigStore::new(&d)
            .save_config(&SyncConfig {
                schema_version: 1,
                endpoints: vec![SyncEndpointConfig {
                    endpoint_id: "default".into(),
                    url: "https://fixture.invalid".into(),
                    remote_path: "cli-v2".into(),
                    username: "fixture".into(),
                    credential_ref: CredentialRef {
                        ref_id: CREDS.into(),
                        backend: "keychain".into(),
                        service_name: keychain.to_string(),
                        account_key: crate::infrastructure::keychain::account_key(
                            crate::infrastructure::keychain::WEBDAV_CREDENTIAL_PURPOSE,
                            CREDS,
                        )
                        .unwrap(),
                        purpose: crate::infrastructure::keychain::WEBDAV_CREDENTIAL_PURPOSE.into(),
                        created_at: TIME.into(),
                        updated_at: TIME.into(),
                    },
                    tls_policy: TLS_POLICY_VERIFY_REQUIRED.into(),
                    legacy_encryption: String::new(),
                    conflict_policy: CONFLICT_POLICY_ASK.into(),
                }],
            })
            .unwrap();
        let sync = Arc::new(
            SyncServiceV2::new(
                d.clone(),
                Arc::new(
                    ProductionServiceBackendV2::new(
                        d.clone(),
                        h.clone(),
                        Arc::new(SystemCaptureCredentialsV2),
                    )
                    .unwrap(),
                ),
                remote.clone(),
                Arc::new(Credentials { d: d.clone() }),
            )
            .unwrap(),
        );
        Self {
            _temp: temp,
            d,
            h,
            remote,
            sync,
        }
    }
    fn ipc(&self) -> IpcService {
        let mut ipc = IpcService::for_tests_with(self.d.clone(), self.d.clone());
        // These intentionally differ: only the formal process H is valid.
        ipc.dependencies.home = self._temp.path().join("never-user-home");
        ipc.dependencies.opencodex_home = self._temp.path().join("never-legacy-home");
        ipc.dependencies.process_context.opencodex_home = self.h.clone();
        *ipc.sync_v2.lock().unwrap() = Some(self.sync.clone());
        ipc
    }
    fn audit(&self, _: &IpcService) -> Vec<super::super::AuditRecord> {
        match std::fs::read_to_string(self.d.join("audit.log")) {
            Ok(text) => text
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => vec![],
            Err(e) => panic!("synthetic audit read failed: {e}"),
        }
    }
    fn no_v1(&self) {
        assert!(self
            .remote
            .files
            .lock()
            .unwrap()
            .keys()
            .all(|p| p.starts_with("config-v2-")));
        assert!(!self.d.join("sync-state.json").exists());
        assert!(!self.d.join("manager-state/sync-state.json").exists());
    }
}
fn request(confirm: bool) -> IpcRequest {
    IpcRequest {
        request_id: format!("req_{}", uuid::Uuid::new_v4()),
        command: IpcCommand::SyncRun,
        args: BTreeMap::new(),
        confirm,
        contract_version: 1,
        secret: None,
    }
}
async fn cli(env: &Env) -> serde_json::Value {
    let response = env.ipc().execute(request(true)).await;
    assert!(response.ok, "{:?}", response.error);
    response.data.unwrap()
}
fn failed(f: ErrorCode) -> SyncFailure {
    SyncFailure {
        code: f,
        step: Step::Verify,
        retryability: Retryability::AfterRefresh,
        cause: None,
        http_status: None,
        recovery_actions: vec![],
    }
}

#[tokio::test]
async fn cli_real_v2_initial_catalog_uploads_verified_wrapper_and_audits_original_request() {
    let e = Env::new(Arc::new(Remote::default()), true);
    let mut ipc = e.ipc();
    let r = request(true);
    let response = ipc.execute(r.clone()).await;
    assert!(response.ok, "{:?}", response.error);
    assert_eq!(response.request_id, r.request_id);
    let data = response.data.unwrap();
    assert_eq!(data["status"], "stored");
    assert_eq!(data["direction"], "upload");
    let id = data["snapshot_id"].as_str().unwrap();
    assert!(uuid::Uuid::parse_str(id).is_ok());
    assert!(!id.starts_with("snap_"));
    let fact = e.sync.result(id.into()).await.unwrap();
    assert_eq!(data["operation"], serde_json::to_value(&fact).unwrap());
    assert_eq!(fact.storage_status, StorageStatus::Stored);
    assert_eq!(fact.local_apply_status, LocalApplyStatus::NotApplicable);
    let scope = e.sync.read_scope().await.unwrap().unwrap();
    assert_eq!(scope.selection.channels.len(), 1);
    let data_file = e
        .remote
        .files
        .lock()
        .unwrap()
        .iter()
        .find(|(p, _)| p.ends_with(".ocxdconf"))
        .unwrap()
        .1
        .clone();
    let decoded = crate::modules::model_config::file_objects_v2::decode(
        &data_file,
        None,
        &crate::modules::model_config::file_objects_v2::supported_schema(),
        Some(&crate::modules::model_config::sync_host_v2::PayloadAuthorizationV2),
    )
    .unwrap();
    assert_eq!(
        decoded
            .objects
            .models()
            .payload
            .policy
            .as_ref()
            .unwrap()
            .channels[0]
            .models[0]
            .raw_model_id,
        "cli-model"
    );
    let audit = e.audit(&ipc);
    assert_eq!(audit.len(), 1);
    assert_eq!(audit[0].request_id, r.request_id);
    assert_eq!(audit[0].result, super::super::AuditResult::Succeeded);
    assert!(audit[0].confirm);
    let text = std::fs::read_to_string(e.d.join("audit.log")).unwrap();
    assert!(!text.contains(SECRET));
    assert!(!text.contains("actual-H"));
    e.no_v1();
}

#[tokio::test]
async fn cli_repeated_models_follow_actual_backend_plan_without_inventing_semantic_equality() {
    let source = Env::new(Arc::new(Remote::default()), true);
    cli(&source).await;
    let e = Env::new(source.remote.clone(), false);
    cli(&e).await;
    let scope = e.sync.read_scope().await.unwrap().unwrap();
    let preview = e
        .sync
        .preview(
            PreviewRequestV2 {
                selection: scope.selection,
                mcp_clients: scope.mcp_clients,
                assets_reviewed: scope.assets_reviewed,
                selected_head: None,
                forced_direction: None,
                backup_before_update: true,
            },
            ServiceSecretsV2::default(),
        )
        .await
        .unwrap();
    // The model compiler may retain local defaults absent from the cloud.
    // The CLI must use the real semantic plan, not compare paths/raw files.
    let no_change = matches!(preview.plan, ServicePlanV2::NoChange);
    assert!(no_change || matches!(preview.plan, ServicePlanV2::Download(_)));
    e.sync.cancel(preview.session_id).await.unwrap();
    let writes = e.remote.writes.load(SeqCst);
    let data = cli(&e).await;
    if no_change {
        assert_eq!(data["direction"], "no_change");
        assert_eq!(data["status"], "no_change");
        assert_eq!(data["snapshot_id"], "");
        assert_eq!(data["artifacts"], 0);
        assert!(data.get("operation").is_none());
    } else {
        assert_eq!(data["direction"], "download");
        assert_eq!(data["status"], "download_applied");
        assert_eq!(data["operation"]["localApplyStatus"], "applied");
        assert_eq!(data["snapshot_id"], data["operation"]["operationId"]);
    }
    assert_eq!(e.remote.writes.load(SeqCst), writes);
    e.no_v1();
}

#[tokio::test]
async fn cli_real_saved_preferences_no_change_has_no_fabricated_snapshot_or_confirm_write() {
    let e = Env::new(Arc::new(Remote::default()), false);
    crate::modules::preferences::PreferencesStore::new(&e.d)
        .save(&crate::modules::preferences::Preferences::default())
        .unwrap();
    assert_eq!(cli(&e).await["status"], "stored");
    let writes = e.remote.writes.load(SeqCst);
    let data = cli(&e).await;
    assert_eq!(data["status"], "no_change");
    assert_eq!(data["snapshot_id"], "");
    assert_eq!(data["artifacts"], 0);
    assert_eq!(e.remote.writes.load(SeqCst), writes);
    e.no_v1();
}

#[tokio::test]
async fn cli_first_device_download_uses_actual_model_transaction_receipt_and_default_backup() {
    let source = Env::new(Arc::new(Remote::default()), true);
    cli(&source).await;
    let target = Env::new(source.remote.clone(), false);
    let puts = source.remote.writes.load(SeqCst);
    let data = cli(&target).await;
    assert_eq!(data["status"], "download_applied");
    assert_eq!(data["direction"], "download");
    let id = data["snapshot_id"].as_str().unwrap();
    let fact = target.sync.result(id.into()).await.unwrap();
    assert_eq!(fact.local_apply_status, LocalApplyStatus::Applied);
    assert!(target
        .d
        .join("manager-state/model-receive-v2")
        .join(id)
        .join("receipt.json")
        .is_file());
    let journal: serde_json::Value = serde_json::from_slice(
        &std::fs::read(
            target
                .d
                .join("manager-state/sync-local-v2")
                .join(format!("{id}.json")),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(journal["journal"]["backup"], true);
    assert_eq!(journal["journal"]["progress"], "Applied");
    let raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(target.h.join("config.json")).unwrap()).unwrap();
    assert!(raw["providers"]
        .as_object()
        .unwrap()
        .values()
        .any(|p| p["models"]
            .as_array()
            .is_some_and(|m| m.contains(&serde_json::json!("cli-model")))));
    assert_eq!(source.remote.writes.load(SeqCst), puts);
    target.no_v1();
}

#[tokio::test]
async fn confirmation_and_invalid_args_reject_before_scope_network_or_audit() {
    let e = Env::new(Arc::new(Remote::default()), true);
    let mut ipc = e.ipc();
    let r = ipc.execute(request(false)).await;
    assert_eq!(r.error.unwrap().code, IpcErrorCode::RequireConfirm);
    let mut r = request(true);
    r.args.insert("head".into(), "untrusted".into());
    assert_eq!(
        ipc.execute(r).await.error.unwrap().code,
        IpcErrorCode::ValidationFailed
    );
    let mut r = request(true);
    r.secret = Some("not-a-sync-argument".into());
    assert_eq!(
        ipc.execute(r).await.error.unwrap().code,
        IpcErrorCode::ValidationFailed
    );
    assert!(e.sync.read_scope().await.unwrap().is_none());
    assert_eq!(e.remote.reads.load(SeqCst), 0);
    assert!(e.audit(&ipc).is_empty());
}

#[tokio::test]
async fn real_v2_cli_busy_or_frozen_rejects_before_catalog_credentials_and_audit() {
    let e = Env::new(Arc::new(Remote::default()), true);
    let mut ipc = e.ipc();
    let busy = ipc.sync_operations.clone().try_lock_owned().unwrap();
    assert_eq!(
        ipc.execute(request(true)).await.error.unwrap().code,
        IpcErrorCode::Timeout
    );
    drop(busy);
    let frozen = ipc.writers.freeze().unwrap().unwrap();
    assert_eq!(
        ipc.execute(request(true)).await.error.unwrap().code,
        IpcErrorCode::TargetStateConflict
    );
    drop(frozen);
    assert_eq!(e.remote.reads.load(SeqCst), 0);
    assert_eq!(e.remote.writes.load(SeqCst), 0);
    assert!(e.sync.read_scope().await.unwrap().is_none());
    assert!(e.audit(&ipc).is_empty());
}

#[tokio::test]
async fn saved_explicit_empty_scope_is_not_replaced_by_catalog_default_all() {
    let e = Env::new(Arc::new(Remote::default()), true);
    e.sync
        .save_scope(ScopeRequestV2 {
            selection: SelectionV2::default(),
            mcp_clients: BTreeMap::new(),
            assets_reviewed: false,
        })
        .await
        .unwrap();
    let response = e.ipc().execute(request(true)).await;
    // Both compared scopes empty is an explicit no-op, not a model upload.
    assert!(response.ok, "{:?}", response.error);
    assert_eq!(response.data.unwrap()["status"], "no_change");
    assert!(e
        .sync
        .read_scope()
        .await
        .unwrap()
        .unwrap()
        .selection
        .channels
        .is_empty());
    assert_eq!(e.remote.writes.load(SeqCst), 0);
}

#[tokio::test]
async fn protected_cloud_without_passphrase_returns_typed_failure_without_plaintext_fallback() {
    let source = Env::new(Arc::new(Remote::default()), true);
    let c = source.sync.catalog(None).await.unwrap();
    let s = c.selected;
    let p = source
        .sync
        .preview(
            PreviewRequestV2 {
                selection: s.selection,
                mcp_clients: s.mcp_clients,
                assets_reviewed: s.assets_reviewed,
                selected_head: None,
                forced_direction: None,
                backup_before_update: true,
            },
            ServiceSecretsV2 {
                read_passphrase: None,
                write_passphrase: Some(Zeroizing::new("synthetic-payload-passphrase".into())),
            },
        )
        .await
        .unwrap();
    source
        .sync
        .confirm(ConfirmRequestV2 {
            session_id: p.session_id,
            plan_digest: p.plan_digest,
        })
        .await
        .unwrap();
    let target = Env::new(source.remote.clone(), false);
    let puts = source.remote.writes.load(SeqCst);
    let mut ipc = target.ipc();
    let response = ipc.execute(request(true)).await;
    assert!(!response.ok);
    let error = response.error.unwrap();
    assert_eq!(error.code, IpcErrorCode::PassphraseRequired);
    assert!(error.message.contains("Protected v2"));
    assert!(!error.message.contains("Legacy container"));
    assert!(!target.h.join("config.json").exists());
    assert!(target.sync.read_scope().await.unwrap().is_none());
    assert_eq!(source.remote.writes.load(SeqCst), puts);
    assert_eq!(
        target.audit(&ipc)[0].error_code,
        Some(IpcErrorCode::PassphraseRequired)
    );
    target.no_v1();
}

#[tokio::test]
async fn differing_mcp_sources_require_gui_choice_and_never_silently_sync_a_partial_default() {
    let e = Env::new(Arc::new(Remote::default()), true);
    std::fs::create_dir_all(e.h.join(".codex")).unwrap();
    std::fs::create_dir_all(e.h.join(".hermes")).unwrap();
    std::fs::write(
        e.h.join(".codex/config.toml"),
        "[mcp_servers.fixture]\ncommand='source-one'\n",
    )
    .unwrap();
    std::fs::write(
        e.h.join(".hermes/config.yaml"),
        "mcp_servers:\n  fixture:\n    command: source-two\n",
    )
    .unwrap();
    for _ in 0..2 {
        let r = e.ipc().execute(request(true)).await;
        assert_eq!(r.error.unwrap().code, IpcErrorCode::TargetStateConflict);
    }
    assert_eq!(e.remote.writes.load(SeqCst), 0);
    assert!(!e
        .d
        .join("manager-state/sync-identity-mappings-v2.json")
        .exists());
    let c = e.sync.catalog(None).await.unwrap();
    let source = &c.mcp_sources[0];
    let option = &source.sources[0];
    let mut selection = c.selected.selection;
    selection.mcp_sources.push(
        crate::modules::model_config::objects_overlay_v2::McpSourceSelectionV2 {
            name: source.name.clone(),
            definition_digest: option.definition_digest.clone(),
        },
    );
    e.sync
        .save_scope(ScopeRequestV2 {
            selection,
            mcp_clients: BTreeMap::from([(source.name.clone(), option.source.clone())]),
            assets_reviewed: true,
        })
        .await
        .unwrap();
    assert_eq!(cli(&e).await["status"], "stored");
    e.no_v1();
}

#[tokio::test]
async fn bad_payload_readback_and_unauthorized_remote_never_project_success() {
    for unauthorized in [true, false] {
        let e = Env::new(Arc::new(Remote::default()), true);
        e.remote.unauthorized.store(unauthorized, SeqCst);
        e.remote.fail_payload_readback.store(!unauthorized, SeqCst);
        let mut ipc = e.ipc();
        let r = ipc.execute(request(true)).await;
        assert!(!r.ok);
        assert!(r.data.is_none());
        assert_eq!(
            r.error.unwrap().code,
            if unauthorized {
                IpcErrorCode::AccessDenied
            } else {
                IpcErrorCode::TargetStateConflict
            }
        );
        assert_eq!(e.audit(&ipc)[0].result, super::super::AuditResult::Failed);
        e.no_v1();
    }
}

#[tokio::test]
async fn cancelled_cli_observer_keeps_admissions_until_real_v2_terminal_and_audit() {
    let e = Env::new(Arc::new(Remote::default()), true);
    e.remote.hold_listing.store(true, SeqCst);
    let mut ipc = e.ipc();
    let writers = ipc.writers.clone();
    let gate = ipc.sync_operations.clone();
    let observer = tokio::spawn(async move { ipc.execute(request(true)).await });
    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        e.remote.entered.notified(),
    )
    .await
    .unwrap();
    observer.abort();
    let _ = observer.await;
    assert!(writers.freeze().unwrap().is_none());
    assert!(gate.try_lock().is_err());
    e.remote.release.notify_one();
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            if e.d.join("audit.log").is_file() && gate.try_lock().is_ok() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let ipc = e.ipc();
    let audit = e.audit(&ipc);
    assert_eq!(audit.len(), 1);
    assert_eq!(audit[0].result, super::super::AuditResult::Succeeded);
    assert!(writers.freeze().unwrap().is_some());
    e.no_v1();
}

#[test]
fn production_cli_source_selects_process_context_home_and_has_no_legacy_engine_edge() {
    let source = include_str!("service.rs");
    let body = source
        .split("    async fn sync_run(")
        .nth(1)
        .unwrap()
        .split("    async fn run_lifecycle(")
        .next()
        .unwrap();
    assert!(body.contains("dependencies.process_context.opencodex_home"));
    assert!(!body.contains("execute_sync_now"));
    assert!(body.contains("run_ipc_sync_owned"));
    assert!(body.contains("execute_ipc_sync_v2"));
    let all = source
        .split("#[cfg(test)]\nfn project_ipc_sync(")
        .next()
        .unwrap();
    assert!(!all.contains("crate::commands::sync::execute_sync_now("));
}

#[tokio::test]
async fn lazy_production_constructor_is_shared_and_uses_explicit_existing_home() {
    let t = tempfile::Builder::new()
        .prefix("ocxd023-cli-sync-v2-constructor-")
        .tempdir_in("/tmp")
        .unwrap();
    let d = t.path().join("D");
    let h = t.path().join("H");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::create_dir_all(&h).unwrap();
    let slot = Mutex::new(None);
    let a = ipc_sync_service_v2(&slot, d.clone(), h.clone()).unwrap();
    let b = ipc_sync_service_v2(&slot, d.clone(), h).unwrap();
    assert!(Arc::ptr_eq(&a, &b));
    assert!(a.read_scope().await.unwrap().is_none());
    assert!(d.join("manager-state/sync-v2-transactions").is_dir());
    assert!(ipc_sync_service_v2(&Mutex::new(None), d, t.path().join("missing-H")).is_err());
}

#[tokio::test]
async fn projection_requires_matching_operation_and_independent_terminal_proofs() {
    let e = Env::new(Arc::new(Remote::default()), true);
    let data = cli(&e).await;
    let id = data["snapshot_id"].as_str().unwrap();
    let fact = e.sync.result(id.into()).await.unwrap();
    for status in [
        StorageStatus::NotAttempted,
        StorageStatus::Unknown,
        StorageStatus::Invalid,
    ] {
        let mut f = fact.clone();
        f.storage_status = status;
        assert_eq!(
            project_ipc_sync_v2(id, OperationKind::Upload, &f).unwrap_err(),
            IpcErrorCode::ExecutionFailed
        );
    }
    let mut changed = fact.clone();
    changed.relation.local = LocalRelation::Changed;
    let projected = project_ipc_sync_v2(id, OperationKind::Upload, &changed).unwrap();
    assert_eq!(projected["operation"]["relation"]["local"], "changed"); // Stored is not local equality.
    for relation in [
        RemoteRelation::Conflict,
        RemoteRelation::InitializationAmbiguous,
        RemoteRelation::GenerationChanged,
    ] {
        let mut f = fact.clone();
        f.relation.remote = relation;
        assert_eq!(
            project_ipc_sync_v2(id, OperationKind::Upload, &f).unwrap_err(),
            IpcErrorCode::TargetStateConflict
        );
    }
    let mut f = fact.clone();
    f.failure = Some(failed(ErrorCode::SyncNetwork));
    assert!(project_ipc_sync_v2(id, OperationKind::Upload, &f).is_err());
    assert!(project_ipc_sync_v2(
        &uuid::Uuid::new_v4().to_string(),
        OperationKind::Upload,
        &fact
    )
    .is_err());
    let mut receive = fact;
    receive.kind = OperationKind::Download;
    for status in [
        LocalApplyStatus::NotAttempted,
        LocalApplyStatus::Unknown,
        LocalApplyStatus::Invalid,
        LocalApplyStatus::NotApplicable,
    ] {
        receive.local_apply_status = status;
        assert!(project_ipc_sync_v2(id, OperationKind::Download, &receive).is_err());
    }
}

#[tokio::test]
async fn ambiguous_generations_are_not_implicitly_selected_or_written_by_cli() {
    let e = Env::new(Arc::new(Remote::default()), true);
    cli(&e).await;
    let id = uuid::Uuid::new_v4().to_string();
    // Clone the actual generation's format/version instead of inventing a wire format.
    let mut wire: serde_json::Value = {
        let files = e.remote.files.lock().unwrap();
        serde_json::from_slice(
            files
                .iter()
                .find(|(p, _)| p.starts_with("config-v2-generation-"))
                .unwrap()
                .1,
        )
        .unwrap()
    };
    wire["generationId"] = serde_json::json!(id);
    e.remote.files.lock().unwrap().insert(
        SyncRelativePath::generation(&id).unwrap().as_str().into(),
        serde_json::to_vec(&wire).unwrap(),
    );
    let writes = e.remote.writes.load(SeqCst);
    let mut ipc = e.ipc();
    let r = ipc.execute(request(true)).await;
    assert!(!r.ok);
    assert_eq!(r.error.unwrap().code, IpcErrorCode::TargetStateConflict);
    assert_eq!(e.remote.writes.load(SeqCst), writes);
    assert_eq!(
        e.audit(&ipc).last().unwrap().result,
        super::super::AuditResult::Failed
    );
}

#[test]
fn v2_failures_keep_cli_error_classes_and_never_expose_provider_details() {
    for (code, expected) in [
        (
            ErrorCode::SyncPassphraseRequired,
            IpcErrorCode::PassphraseRequired,
        ),
        (ErrorCode::SyncAuthRequired, IpcErrorCode::AccessDenied),
        (
            ErrorCode::SyncCollectionMissing,
            IpcErrorCode::TargetNotFound,
        ),
        (
            ErrorCode::SyncRemoteConflict,
            IpcErrorCode::TargetStateConflict,
        ),
        (
            ErrorCode::SyncOperationUnknown,
            IpcErrorCode::ExecutionFailed,
        ),
        (ErrorCode::SyncBackupFailed, IpcErrorCode::ExecutionFailed),
        (ErrorCode::SyncCancelled, IpcErrorCode::Cancelled),
    ] {
        assert_eq!(map_sync_v2_failure(failed(code)), expected);
    }
    let mut failure = failed(ErrorCode::SyncNetwork);
    failure.cause = Some(Cause::Timeout);
    assert_eq!(map_sync_v2_failure(failure), IpcErrorCode::Timeout);
}

#[test]
fn terminal_notification_candidate_stays_bound_to_the_original_public_endpoint() {
    let e = Env::new(Arc::new(Remote::default()), true);
    let frozen = ipc_sync_notification_candidate(&e.d).unwrap();
    assert!(!String::from_utf8(frozen.clone()).unwrap().contains(SECRET));
    assert_eq!(
        ipc_sync_notification_candidate(&e.d).as_deref(),
        Some(frozen.as_slice())
    );
    let store = SyncConfigStore::new(&e.d);
    let mut config = store.load().unwrap();
    config.endpoints[0].username = "changed-owner".into();
    store.save_config(&config).unwrap();
    assert_ne!(
        ipc_sync_notification_candidate(&e.d).as_deref(),
        Some(frozen.as_slice())
    );
    assert_eq!(e.remote.reads.load(SeqCst), 0);
    assert_eq!(e.remote.writes.load(SeqCst), 0);
}
