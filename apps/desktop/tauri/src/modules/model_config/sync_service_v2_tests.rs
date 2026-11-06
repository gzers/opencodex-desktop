//! Real capture/v6 objects/overlay/Host/core; deterministic in-memory transport.
//! The apply-only fixture is intentionally unavailable until L0's actual
//! receiver is composed, rather than pretending LocalApplied is a download test.
use super::super::schema::invalid;
use super::super::{saved_objects_capture_v2 as capture, sync_host_v2 as host};
use super::*;
use serde_json::json;
use std::sync::atomic::AtomicUsize;
const CREDS: &str = "cred_11111111-1111-4111-8111-111111111111";
const TIME: &str = "2026-11-06T15:00:00Z";
#[derive(Default)]
struct Remote {
    files: Mutex<BTreeMap<String, Vec<u8>>>,
    gets: AtomicUsize,
    puts: AtomicUsize,
    fail: AtomicBool,
}
#[async_trait::async_trait]
impl ConfigSyncTransport for Remote {
    async fn list_children(
        &self,
        _: &ConnectionHandle,
        _: ListLimits,
    ) -> Result<ChildListing, SyncTransportError> {
        self.gets.fetch_add(1, Ordering::SeqCst);
        if self.fail.load(Ordering::SeqCst) {
            return Err(SyncTransportError {
                kind: TransportKind::Unauthorized,
                http_status: Some(401),
                delivery: Delivery::NotSent,
            });
        }
        Ok(ChildListing {
            http_status: 207,
            response_bytes: 100,
            entries: self
                .files
                .lock()
                .unwrap()
                .iter()
                .map(|(name, v)| ChildEntry {
                    name: name.clone(),
                    is_collection: false,
                    content_length: Some(v.len() as u64),
                })
                .collect(),
        })
    }
    async fn get_file(
        &self,
        _: &ConnectionHandle,
        p: &SyncRelativePath,
        _: ReadLimits,
    ) -> Result<FileRead, SyncTransportError> {
        self.gets.fetch_add(1, Ordering::SeqCst);
        self.files
            .lock()
            .unwrap()
            .get(p.as_str())
            .cloned()
            .map(|body| FileRead {
                http_status: 200,
                body,
            })
            .ok_or(SyncTransportError {
                kind: TransportKind::NotFound,
                http_status: Some(404),
                delivery: Delivery::ResponseReceived,
            })
    }
    async fn put_file(
        &self,
        _: &ConnectionHandle,
        p: &SyncRelativePath,
        b: &[u8],
        _: WriteLimits,
    ) -> Result<PutResponse, SyncTransportError> {
        self.puts.fetch_add(1, Ordering::SeqCst);
        self.files
            .lock()
            .unwrap()
            .insert(p.as_str().into(), b.to_vec());
        Ok(PutResponse { http_status: 201 })
    }
}
struct CredentialOwner;
impl EndpointCredentialsV2 for CredentialOwner {
    fn read_password(
        &self,
        _: &Path,
        e: &crate::modules::sync::config::SyncEndpointConfig,
    ) -> ModelResult<Zeroizing<String>> {
        assert_eq!(e.credential_ref.ref_id, CREDS);
        Ok(Zeroizing::new("synthetic-service-password".into()))
    }
}
struct GuardLocal {
    provider: host::CurrentSourceProviderV2,
}
impl LocalApplication for GuardLocal {
    fn root_revision(&self) -> SyncResult<Digest> {
        self.provider.root_revision()
    }
    fn current_source(&self) -> SyncResult<SourceGuard> {
        self.provider.current_source()
    }
    fn target_digest(&self, _: &str) -> SyncResult<Digest> {
        Ok(self.provider.current_source()?.saved_source_digest)
    }
    fn apply_once(
        &self,
        _: &str,
        _: &[u8],
        _: &PayloadSchemaRef,
        _: &str,
        _: &str,
        _: bool,
    ) -> SyncResult<()> {
        panic!("actual receiver required for a download test")
    }
    fn export_once(&self, _: &str, _: &[u8], _: &PayloadSchemaRef, _: &str) -> SyncResult<String> {
        panic!("actual export owner required")
    }
}
struct CaptureBackend {
    root: PathBuf,
    home: PathBuf,
    selection: SelectionV2,
    calls: AtomicUsize,
}
impl ServiceBackendV2 for CaptureBackend {
    fn inventory(&self) -> ModelResult<InventoryV2> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let r = capture::CaptureRequestV2 {
            selection: self.selection.clone(),
            mcp_clients: Default::default(),
            assets_reviewed: false,
        };
        Ok(InventoryV2 {
            objects: capture::capture(
                &self.root,
                &self.home,
                &r,
                &capture::SystemCaptureCredentialsV2,
            )?
            .objects,
            mcp_sources: vec![],
        })
    }
    fn compose(
        &self,
        r: &PreviewRequestV2,
        _: &ConnectionHandle,
        _: Option<&SavedObjects>,
    ) -> ModelResult<CompositionV2> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let provider = Arc::new(
            host::SavedCaptureProviderV2::new(
                self.root.clone(),
                self.home.clone(),
                capture::CaptureRequestV2 {
                    selection: r.selection.clone(),
                    mcp_clients: r.mcp_clients.clone(),
                    assets_reviewed: r.assets_reviewed,
                },
                Arc::new(capture::SystemCaptureCredentialsV2),
            )
            .map_err(|_| invalid("capture_scope_invalid", "invalid"))?,
        );
        let captured = provider.capture_saved(&r.selection)?;
        let source = host::CurrentSourceProviderV2::new(
            provider.clone(),
            r.selection.clone(),
            captured.root_revision,
        )
        .map_err(|_| invalid("capture_scope_invalid", "invalid"))?;
        Ok(CompositionV2 {
            provider,
            local: Arc::new(GuardLocal { provider: source }),
            remote_scope: r.selection.clone(),
            diff: vec![],
        })
    }
}
struct Env {
    _temp: tempfile::TempDir,
    root: PathBuf,
    home: PathBuf,
    remote: Arc<Remote>,
    backend: Arc<CaptureBackend>,
}
impl Env {
    fn new() -> Self {
        use crate::infrastructure::keychain;
        use crate::modules::sync::config::*;
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("D");
        let home = temp.path().join("H");
        std::fs::create_dir_all(root.join("manager-state")).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(home.join("config.json"),serde_json::to_vec(&json!({"providers":{"fixture":{"adapter":"openai-chat","baseUrl":"https://fixture.invalid/v1","models":["fixture-model"],"selectedModels":["fixture-model"],"modelContextWindows":{"fixture-model":8192}}}})).unwrap()).unwrap();
        let s = crate::modules::model_config::config_file::v2::snapshot(&root, &home).unwrap();
        let observed = crate::modules::model_config::config_file::v2::begin_draft(
            &root,
            &home,
            crate::modules::model_config::config_file::v2::ReadRequestV2 {
                expected_source: s.source_digest,
                expected_library: s.library_digest,
                expected_draft: None,
            },
        )
        .unwrap();
        let id = observed.draft.unwrap().policy.channels[0].id.clone();
        let selection = SelectionV2 {
            channels: vec![ChannelSelectionV2 {
                portable_channel_id: id,
                model_ids: None,
                family_ids: None,
                include_connection: true,
                include_key: false,
                include_authorization: false,
            }],
            ..Default::default()
        };
        SyncConfigStore::new(&root)
            .save_config(&SyncConfig {
                schema_version: 1,
                endpoints: vec![SyncEndpointConfig {
                    endpoint_id: "default".into(),
                    url: "https://fixture.example.test".into(),
                    remote_path: "fixture".into(),
                    username: "fixture".into(),
                    credential_ref: CredentialRef {
                        ref_id: CREDS.into(),
                        backend: "keychain".into(),
                        service_name: keychain::keychain_service_name().into(),
                        account_key: keychain::account_key(
                            keychain::WEBDAV_CREDENTIAL_PURPOSE,
                            CREDS,
                        )
                        .unwrap(),
                        purpose: keychain::WEBDAV_CREDENTIAL_PURPOSE.into(),
                        created_at: TIME.into(),
                        updated_at: TIME.into(),
                    },
                    tls_policy: TLS_POLICY_VERIFY_REQUIRED.into(),
                    legacy_encryption: String::new(),
                    conflict_policy: CONFLICT_POLICY_ASK.into(),
                }],
            })
            .unwrap();
        let backend = Arc::new(CaptureBackend {
            root: root.clone(),
            home: home.clone(),
            selection,
            calls: AtomicUsize::new(0),
        });
        Self {
            _temp: temp,
            root,
            home,
            remote: Arc::new(Remote::default()),
            backend,
        }
    }
    fn service(&self) -> SyncServiceV2 {
        SyncServiceV2::new(
            self.root.clone(),
            self.backend.clone(),
            self.remote.clone(),
            Arc::new(CredentialOwner),
        )
        .unwrap()
    }
    fn request(&self) -> PreviewRequestV2 {
        PreviewRequestV2 {
            selection: self.backend.selection.clone(),
            mcp_clients: Default::default(),
            assets_reviewed: false,
            selected_head: None,
            forced_direction: Some(ServiceDirectionV2::Upload),
            backup_before_update: true,
        }
    }
}
#[tokio::test]
async fn actual_saved_upload_preview_is_read_only_confirmation_publishes_and_reopened_result_verify_is_original_only(
) {
    let e = Env::new();
    let service = e.service();
    let preview = service
        .preview(e.request(), ServiceSecretsV2::default())
        .await
        .unwrap();
    let ServicePlanV2::Upload(plan) = &preview.plan else {
        panic!("upload")
    };
    let id = plan.operation_id.clone();
    let original_plan_digest = plan.plan_digest.clone();
    assert_eq!(e.remote.puts.load(Ordering::SeqCst), 0);
    let fact = service
        .confirm(ConfirmRequestV2 {
            session_id: preview.session_id,
            plan_digest: preview.plan_digest,
        })
        .await
        .unwrap();
    assert_eq!(fact.storage_status, StorageStatus::Stored);
    let puts = e.remote.puts.load(Ordering::SeqCst);
    let calls = e.remote.gets.load(Ordering::SeqCst);
    let sources = e.backend.calls.load(Ordering::SeqCst);
    drop(service);
    std::fs::remove_file(e.home.join("config.json")).unwrap();
    let reopened = e.service();
    assert_eq!(
        reopened.result(id.clone()).await.unwrap().storage_status,
        StorageStatus::Stored
    );
    assert_eq!(calls, e.remote.gets.load(Ordering::SeqCst));
    let recovered = reopened
        .resume_original(
            ResumeUploadRequest {
                operation_id: id.clone(),
                plan_digest: original_plan_digest,
                expected_view_digest: store::digest(b"stored original needs only verification"),
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(recovered.storage_status, StorageStatus::Stored);
    let original = reopened.verify(id, None).await.unwrap();
    assert_eq!(original.storage_status, StorageStatus::Stored);
    assert_eq!(puts, e.remote.puts.load(Ordering::SeqCst));
    assert_eq!(sources, e.backend.calls.load(Ordering::SeqCst));
}
#[tokio::test]
async fn persisted_scope_is_explicit_empty_and_bad_cloud_never_initializes_default_all() {
    let e = Env::new();
    let service = e.service();
    assert!(service.read_scope().await.unwrap().is_none());
    e.remote.fail.store(true, Ordering::SeqCst);
    assert_eq!(
        service.catalog(None).await.err().unwrap().code,
        ErrorCode::SyncAuthRequired
    );
    assert!(service.read_scope().await.unwrap().is_none());
    e.remote.fail.store(false, Ordering::SeqCst);
    let catalog = service.catalog(None).await.unwrap();
    assert!(catalog.initialized);
    assert_eq!(catalog.selected.selection.channels.len(), 1);
    service
        .save_scope(ScopeRequestV2 {
            selection: SelectionV2::default(),
            mcp_clients: Default::default(),
            assets_reviewed: false,
        })
        .await
        .unwrap();
    let reopened = e.service();
    let catalog = reopened.catalog(None).await.unwrap();
    assert!(catalog.initialized);
    assert!(catalog.selected.selection.channels.is_empty());
    assert_eq!(e.remote.puts.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn no_op_is_semantic_and_never_prepares_or_publishes_a_second_operation() {
    let e = Env::new();
    let s = e.service();
    let p = s
        .preview(e.request(), ServiceSecretsV2::default())
        .await
        .unwrap();
    s.confirm(ConfirmRequestV2 {
        session_id: p.session_id,
        plan_digest: p.plan_digest,
    })
    .await
    .unwrap();
    let before = e.remote.puts.load(Ordering::SeqCst);
    let p = s
        .preview(e.request(), ServiceSecretsV2::default())
        .await
        .unwrap();
    assert!(p.no_change);
    assert!(matches!(p.plan, ServicePlanV2::NoChange));
    assert_eq!(e.remote.puts.load(Ordering::SeqCst), before);
}
#[tokio::test]
async fn current_saved_change_and_cancel_block_unconfirmed_publication() {
    let e = Env::new();
    let s = e.service();
    let p = s
        .preview(e.request(), ServiceSecretsV2::default())
        .await
        .unwrap();
    s.cancel(p.session_id.clone()).await.unwrap();
    assert!(s
        .confirm(ConfirmRequestV2 {
            session_id: p.session_id,
            plan_digest: p.plan_digest
        })
        .await
        .is_err());
    assert_eq!(e.remote.puts.load(Ordering::SeqCst), 0);
    let p = s
        .preview(e.request(), ServiceSecretsV2::default())
        .await
        .unwrap();
    let mut raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(e.home.join("config.json")).unwrap()).unwrap();
    raw["providers"]["fixture"]["modelContextWindows"]["fixture-model"] = json!(16384);
    std::fs::write(
        e.home.join("config.json"),
        serde_json::to_vec(&raw).unwrap(),
    )
    .unwrap();
    assert!(s
        .confirm(ConfirmRequestV2 {
            session_id: p.session_id,
            plan_digest: p.plan_digest
        })
        .await
        .is_err());
    assert_eq!(e.remote.puts.load(Ordering::SeqCst), 0);
}
#[test]
fn request_dtos_reject_unknown_connection_authority_and_secrets_are_not_serializable() {
    let e = Env::new();
    let mut value = serde_json::to_value(e.request()).unwrap();
    value["url"] = json!("https://never-authority.example.test");
    assert!(serde_json::from_value::<PreviewRequestV2>(value).is_err());
    let value = json!({"sessionId":"not-uuid","planDigest":"x","password":"never-in-dto"});
    assert!(serde_json::from_value::<ConfirmRequestV2>(value).is_err());
}
#[tokio::test]
async fn catalog_keeps_exact_saved_revisions_and_unions_cloud_only_raw_models_without_expanding_existing_scope(
) {
    let e = Env::new();
    let s = e.service();
    let first = s.catalog(None).await.unwrap();
    assert_eq!(first.selected.selection.channels.len(), 1);
    let p = s
        .preview(e.request(), ServiceSecretsV2::default())
        .await
        .unwrap();
    s.confirm(ConfirmRequestV2 {
        session_id: p.session_id,
        plan_digest: p.plan_digest,
    })
    .await
    .unwrap();
    {
        let mut files = e.remote.files.lock().unwrap();
        let commit_path = files
            .keys()
            .find(|p| p.starts_with("config-v2-commit-"))
            .unwrap()
            .clone();
        let mut commit: Commit = store::decode(&files[&commit_path]).unwrap();
        let payload_path = SyncRelativePath::data(&commit.generation_id, &commit.operation_id)
            .unwrap()
            .as_str()
            .to_owned();
        let decoded = file_objects_v2::decode(
            &files[&payload_path],
            None,
            &file_objects_v2::supported_schema(),
            Some(&host::PayloadAuthorizationV2),
        )
        .unwrap();
        let mut objects = decoded.objects.into_captured();
        let ch = &mut objects.models.payload.policy.as_mut().unwrap().channels[0];
        let mut added = ch.models[0].clone();
        added.raw_model_id = "cloud-only-model".into();
        added.source.raw_model_id = "cloud-only-model".into();
        added.label = "Cloud only".into();
        added.observed = false;
        ch.models.push(added);
        ch.model_order.push("cloud-only-model".into());
        objects.models.digest = super::super::schema_v2::digest(
            &super::super::schema_v2::canonical_bytes(&objects.models.payload).unwrap(),
        );
        let objects = SavedObjects::capture(objects, Some(&host::PayloadAuthorizationV2)).unwrap();
        let encoded = file_objects_v2::encode(
            &objects,
            &commit.operation_id,
            TIME,
            None,
            Some(&host::PayloadAuthorizationV2),
        )
        .unwrap();
        commit.payload_digest = encoded.file_digest.clone();
        commit.payload_bytes = encoded.bytes().len() as u64;
        // Simulate an independently observed, valid descendant; never mutate a
        // previously seen commit ID, which the graph correctly rejects.
        let parent = CommitRef {
            operation_id: commit.operation_id.clone(),
            digest: store::digest(&files[&commit_path]),
        };
        commit.operation_id = uuid::Uuid::new_v4().to_string();
        commit.parents = vec![parent];
        let new_data = SyncRelativePath::data(&commit.generation_id, &commit.operation_id)
            .unwrap()
            .as_str()
            .to_owned();
        let new_commit = SyncRelativePath::commit(&commit.generation_id, &commit.operation_id)
            .unwrap()
            .as_str()
            .to_owned();
        files.insert(new_data, encoded.bytes().to_vec());
        files.insert(new_commit, store::canonical(&commit).unwrap());
    }
    let catalog = s.catalog(None).await.unwrap();
    let channel = catalog
        .entries
        .iter()
        .find(|o| o.kind == ObjectKindV2::Channel)
        .unwrap();
    let extra = channel
        .models
        .iter()
        .find(|m| m.raw_model_id == "cloud-only-model")
        .unwrap();
    assert!(!extra.available_local);
    assert!(extra.available_cloud);
    assert!(!extra.observed);
    assert!(channel.model_ids.contains(&"cloud-only-model".to_string()));
    assert_eq!(catalog.selected.selection, first.selected.selection);
}
#[tokio::test]
async fn original_private_record_has_no_password_and_bad_record_never_uses_new_active_endpoint() {
    let e = Env::new();
    let s = e.service();
    let p = s
        .preview(e.request(), ServiceSecretsV2::default())
        .await
        .unwrap();
    let ServicePlanV2::Upload(plan) = p.plan else {
        panic!("upload")
    };
    let path = e
        .root
        .join("manager-state/sync-service-v2")
        .join(format!("{}.original.json", plan.operation_id));
    let bytes = std::fs::read(&path).unwrap();
    assert!(!String::from_utf8_lossy(&bytes).contains("synthetic-service-password"));
    std::fs::write(&path, b"{corrupt").unwrap();
    let reads = e.remote.gets.load(Ordering::SeqCst);
    assert_eq!(
        s.verify(plan.operation_id, None).await.err().unwrap().code,
        ErrorCode::SyncJournalInvalid
    );
    assert_eq!(reads, e.remote.gets.load(Ordering::SeqCst));
}
