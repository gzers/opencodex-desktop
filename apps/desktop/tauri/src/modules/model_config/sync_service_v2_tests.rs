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
    fn inventory(&self, _: &ConnectionHandle) -> ModelResult<InventoryV2> {
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
            key_channel_ids: Default::default(),
            authorization_channel_ids: Default::default(),
            credential_skill_names: Default::default(),
            credential_mcp_names: Default::default(),
        })
    }
    fn compose(
        &self,
        r: &PreviewRequestV2,
        _: &ConnectionHandle,
        _: Option<&SavedObjects>,
        _: Option<Zeroizing<String>>,
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
#[tokio::test]
async fn resume_preflight_returns_original_plan_request_not_a_new_preview_and_stored_does_not_capture(
) {
    let e = Env::new();
    let s = e.service();
    let p = s
        .preview(e.request(), ServiceSecretsV2::default())
        .await
        .unwrap();
    let ServicePlanV2::Upload(plan) = p.plan.clone() else {
        panic!("upload")
    };
    s.confirm(ConfirmRequestV2 {
        session_id: p.session_id,
        plan_digest: p.plan_digest,
    })
    .await
    .unwrap();
    let count = e.backend.calls.load(Ordering::SeqCst);
    std::fs::remove_file(e.home.join("config.json")).unwrap();
    let preflight = s
        .resume_preflight(plan.operation_id.clone(), None)
        .await
        .unwrap();
    assert!(!preflight.can_resume);
    assert_eq!(preflight.reason, "already_stored");
    assert_eq!(preflight.request.operation_id, plan.operation_id);
    assert_eq!(preflight.request.plan_digest, plan.plan_digest);
    assert_eq!(
        preflight.request.expected_view_digest,
        preflight.observation.view_digest
    );
    assert_eq!(count, e.backend.calls.load(Ordering::SeqCst));
}
#[test]
fn source_inventory_dtos_expose_exact_references_and_per_source_binding_without_secret_payloads() {
    let source = McpSourceEntryV2 {
        name: "fixture".into(),
        sources: vec![
            McpSourceOptionV2 {
                source: "managed".into(),
                definition_digest: store::digest(b"actual same-owner MCP source"),
            },
            McpSourceOptionV2 {
                source: "codex".into(),
                definition_digest: store::digest(b"another actual source"),
            },
        ],
        differing_fields: vec!["credentials".into(), "source".into()],
    };
    let value = serde_json::to_value(source).unwrap();
    assert_eq!(value["sources"][0]["source"], "managed");
    assert!(value["sources"][0]["definitionDigest"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    let mut request: CatalogRequestV2 =
        serde_json::from_value(json!({"readPassphrase":"synthetic-one-request-secret"})).unwrap();
    assert_eq!(
        request.take_secret().unwrap().as_str(),
        "synthetic-one-request-secret"
    );
    assert!(request.read_passphrase.is_none());
    assert!(serde_json::from_value::<CatalogRequestV2>(
        json!({"readPassphrase":null,"url":"not-authority"})
    )
    .is_err());
}
#[test]
fn inventory_dtos_keep_key_and_authorization_independent_for_cloud_and_default_old_fields() {
    let e = Env::new();
    let saved = capture::capture(
        &e.root,
        &e.home,
        &capture::CaptureRequestV2 {
            selection: e.backend.selection.clone(),
            mcp_clients: Default::default(),
            assets_reviewed: false,
        },
        &CatalogNeverCredentials,
    )
    .unwrap()
    .objects;
    let id = e.backend.selection.channels[0].portable_channel_id.clone();
    let row = entries_from(&saved, true)
        .into_iter()
        .find(|r| r.id == id)
        .unwrap();
    assert!(!row.key_available && !row.authorization_available && !row.secret_available);
    let mut old = serde_json::to_value(&row).unwrap();
    old.as_object_mut().unwrap().remove("keyAvailable");
    old.as_object_mut()
        .unwrap()
        .remove("authorizationAvailable");
    let old: ObjectEntryV2 = serde_json::from_value(old).unwrap();
    assert!(!old.key_available && !old.authorization_available);
    let mut parts = saved.into_captured();
    parts
        .channel_keys
        .insert(id.clone(), Zeroizing::new("synthetic-cloud-key".into()))
        .unwrap();
    let keyed = SavedObjects::capture(parts, Some(&host::PayloadAuthorizationV2)).unwrap();
    let row = entries_from(&keyed, false)
        .into_iter()
        .find(|r| r.id == id)
        .unwrap();
    assert!(
        row.available_cloud && !row.available_local && row.key_available && row.secret_available
    );
    assert!(!row.authorization_available);
    let mut parts = keyed.into_captured();
    parts.channel_keys = file_objects_v2::SecretValues::new();
    parts.channel_authorizations.insert(
        id.clone(),
        serde_json::from_value(json!({
            "activeId":null,"fields":{"headers":{"X-Fixture":"synthetic-cloud-authorization"}}
        }))
        .unwrap(),
    );
    let authorized = SavedObjects::capture(parts, Some(&host::PayloadAuthorizationV2)).unwrap();
    let row = entries_from(&authorized, false)
        .into_iter()
        .find(|r| r.id == id)
        .unwrap();
    assert!(row.authorization_available && row.secret_available);
    assert!(!row.key_available);
    let public = serde_json::to_string(&row).unwrap();
    assert!(!public.contains("synthetic-cloud-authorization"));
    assert!(!public.contains("synthetic-cloud-key"));
}
fn production_service(e: &Env) -> SyncServiceV2 {
    SyncServiceV2::new(
        e.root.clone(),
        Arc::new(
            ProductionServiceBackendV2::new(
                e.root.clone(),
                e.home.clone(),
                Arc::new(capture::SystemCaptureCredentialsV2),
            )
            .unwrap(),
        ),
        e.remote.clone(),
        Arc::new(CredentialOwner),
    )
    .unwrap()
}
#[tokio::test]
async fn production_backend_uploads_actual_saved_objects_and_first_device_downloads_with_real_model_receipt(
) {
    let source = Env::new();
    let upload = production_service(&source);
    let mut request = source.request();
    request.forced_direction = None;
    let (connection, _) = upload.active().await.unwrap();
    let backend = ProductionServiceBackendV2::new(
        source.root.clone(),
        source.home.clone(),
        Arc::new(capture::SystemCaptureCredentialsV2),
    )
    .unwrap();
    backend.compose(&request, &connection, None, None).unwrap();
    let preview = upload
        .preview(request, ServiceSecretsV2::default())
        .await
        .unwrap();
    let ServicePlanV2::Upload(_) = preview.plan else {
        panic!("first upload")
    };
    assert_eq!(source.remote.puts.load(Ordering::SeqCst), 0);
    upload
        .confirm(ConfirmRequestV2 {
            session_id: preview.session_id,
            plan_digest: preview.plan_digest,
        })
        .await
        .unwrap();
    let puts = source.remote.puts.load(Ordering::SeqCst);
    let mut target = Env::new();
    target.remote = source.remote.clone();
    std::fs::remove_file(target.home.join("config.json")).unwrap();
    for file in [
        "manager-state/model-v2-provider-identities.json",
        crate::modules::model_config::store_v2::DRAFT_PATH,
    ] {
        let p = target.root.join(file);
        if p.exists() {
            std::fs::remove_file(p).unwrap();
        }
    }
    let download = production_service(&target);
    let mut request = source.request();
    request.forced_direction = None;
    let before = download
        .preview(request, ServiceSecretsV2::default())
        .await
        .unwrap();
    assert_eq!(before.chosen_direction, DirectionV2::Download);
    assert!(before
        .entries
        .iter()
        .any(|e| matches!(e.status, DiffStatusV2::Add)));
    let ServicePlanV2::Download(plan) = &before.plan else {
        panic!("download")
    };
    let operation = plan.operation_id.clone();
    assert!(!target.home.join("config.json").exists());
    let result = download
        .confirm(ConfirmRequestV2 {
            session_id: before.session_id,
            plan_digest: before.plan_digest,
        })
        .await
        .unwrap();
    assert_eq!(result.local_apply_status, LocalApplyStatus::Applied);
    assert_eq!(source.remote.puts.load(Ordering::SeqCst), puts);
    let raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(target.home.join("config.json")).unwrap()).unwrap();
    assert!(raw["providers"].as_object().unwrap().values().any(|p| p
        .get("models")
        .and_then(|m| m.as_array())
        .is_some_and(|ids| ids.contains(&json!("fixture-model")))));
    let receipt = target
        .root
        .join("manager-state/model-receive-v2")
        .join(&operation)
        .join("receipt.json");
    assert!(receipt.is_file());
    let mapping: serde_json::Value = serde_json::from_slice(
        &std::fs::read(
            target
                .root
                .join("manager-state/sync-identity-mappings-v2.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(mapping["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m["adoptedOperationId"] == operation));
    assert_eq!(
        download.result(operation).await.unwrap().local_apply_status,
        LocalApplyStatus::Applied
    );
}
#[tokio::test]
async fn production_directory_mcp_source_options_are_exact_and_do_not_choose_conflicting_sources() {
    let e = Env::new();
    std::fs::create_dir_all(e.home.join(".codex")).unwrap();
    std::fs::create_dir_all(e.home.join(".hermes")).unwrap();
    std::fs::write(
        e.home.join(".codex/config.toml"),
        "[mcp_servers.fixture]\ncommand = 'codex-source'\n",
    )
    .unwrap();
    std::fs::write(
        e.home.join(".hermes/config.yaml"),
        "mcp_servers:\n  fixture:\n    command: hermes-source\n",
    )
    .unwrap();
    let service = production_service(&e);
    let catalog = service.catalog(None).await.unwrap();
    let mcp = catalog
        .mcp_sources
        .iter()
        .find(|m| m.name == "fixture")
        .unwrap();
    assert_eq!(mcp.sources.len(), 2);
    assert!(mcp
        .sources
        .iter()
        .all(|m| super::super::schema_v2::valid_digest(&m.definition_digest)));
    assert!(!mcp.differing_fields.is_empty());
    assert!(!catalog
        .selected
        .selection
        .mcp_sources
        .iter()
        .any(|m| m.name == "fixture"));
    assert!(catalog
        .entries
        .iter()
        .any(|e| e.kind == ObjectKindV2::Mcp && e.id == "fixture"));
}
fn add_full_saved_scope(e: &Env, request: &mut PreviewRequestV2) {
    let prefs = crate::modules::preferences::Preferences::default();
    crate::modules::preferences::PreferencesStore::new(&e.root)
        .save(&prefs)
        .unwrap();
    let config = crate::modules::extensions::ExtensionConfig::default();
    let projection = crate::modules::extensions::projection::ExtensionProjection {
        fingerprint: crate::modules::extensions::ProjectionFingerprint::create(
            config.fingerprint(),
            chrono::Utc::now(),
        ),
        config,
    };
    std::fs::write(
        e.root
            .join(crate::modules::extensions::projection::CONFIG_RELATIVE_PATH),
        serde_json::to_vec(&projection).unwrap(),
    )
    .unwrap();
    let skill = e.home.join(".agents/skills/service-fixture");
    std::fs::create_dir_all(&skill).unwrap();
    std::fs::write(
        skill.join("SKILL.md"),
        b"# Service fixture\nRead synthetic resource only.",
    )
    .unwrap();
    std::fs::write(skill.join("resource.txt"), b"fixture-resource").unwrap();
    std::fs::create_dir_all(e.home.join(".codex")).unwrap();
    std::fs::write(
        e.home.join(".codex/config.toml"),
        "[mcp_servers.service_fixture]\ncommand = 'service-fixture-command'\nargs = []\n",
    )
    .unwrap();
    let sources = BTreeMap::from([("service_fixture".into(), "codex".into())]);
    let (assets, conflicts) = crate::modules::sync::portable_assets::export_with_credential_scope(
        &e.root,
        &e.home,
        &[],
        &["service_fixture".into()],
        &sources,
        false,
        true,
        Some(&[]),
    )
    .unwrap();
    assert!(conflicts.is_empty());
    request.selection.manager.preferences = true;
    request.selection.manager.extensions = true;
    request.selection.skill_names = vec!["service-fixture".into()];
    request.selection.mcp_sources = vec![McpSourceSelectionV2 {
        name: "service_fixture".into(),
        definition_digest: super::super::objects_overlay_v2::mcp_source_digest(&assets.mcps[0])
            .unwrap(),
    }];
    request.mcp_clients = sources;
    request.assets_reviewed = true;
}
fn empty_existing_fixture(e: &Env) {
    if e.home.join("config.json").exists() {
        std::fs::remove_file(e.home.join("config.json")).unwrap();
    }
    for file in [
        "manager-state/model-v2-provider-identities.json",
        crate::modules::model_config::store_v2::DRAFT_PATH,
    ] {
        let p = e.root.join(file);
        if p.exists() {
            std::fs::remove_file(p).unwrap();
        }
    }
}
#[tokio::test]
async fn production_first_device_receives_model_mapping_receipt_preferences_extensions_and_assets_after_model_adoption(
) {
    let source = Env::new();
    let mut request = source.request();
    add_full_saved_scope(&source, &mut request);
    let upload = production_service(&source);
    let preview = upload
        .preview(request.clone(), ServiceSecretsV2::default())
        .await
        .unwrap();
    upload
        .confirm(ConfirmRequestV2 {
            session_id: preview.session_id,
            plan_digest: preview.plan_digest,
        })
        .await
        .unwrap();
    let puts = source.remote.puts.load(Ordering::SeqCst);
    let mut target = Env::new();
    target.remote = source.remote.clone();
    empty_existing_fixture(&target);
    let receive = production_service(&target);
    request.forced_direction = None;
    let preview = receive
        .preview(request, ServiceSecretsV2::default())
        .await
        .unwrap();
    assert_eq!(preview.chosen_direction, DirectionV2::Download);
    let ServicePlanV2::Download(plan) = preview.plan.clone() else {
        panic!("download")
    };
    assert!(preview
        .entries
        .iter()
        .filter(|e| matches!(
            e.kind,
            ObjectKindV2::Channel
                | ObjectKindV2::Preferences
                | ObjectKindV2::Extensions
                | ObjectKindV2::Skill
                | ObjectKindV2::Mcp
        ))
        .all(|e| matches!(e.status, DiffStatusV2::Add)));
    let result = receive
        .confirm(ConfirmRequestV2 {
            session_id: preview.session_id,
            plan_digest: preview.plan_digest,
        })
        .await
        .unwrap();
    assert_eq!(result.local_apply_status, LocalApplyStatus::Applied);
    assert_eq!(source.remote.puts.load(Ordering::SeqCst), puts);
    assert!(target
        .root
        .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH)
        .is_file());
    assert!(target
        .root
        .join(crate::modules::extensions::projection::CONFIG_RELATIVE_PATH)
        .is_file());
    assert!(
        target
            .home
            .join(".agents/skills/service-fixture/SKILL.md")
            .is_file()
            || target
                .root
                .join("manager-state/skills-store/service-fixture/SKILL.md")
                .is_file()
    );
    let definitions: serde_json::Value = serde_json::from_slice(
        &std::fs::read(target.root.join("manager-state/mcp-definitions.json")).unwrap(),
    )
    .unwrap();
    assert!(definitions
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m["name"] == "service_fixture"));
    assert!(target
        .root
        .join("manager-state/model-receive-v2")
        .join(plan.operation_id)
        .join("receipt.json")
        .exists());
}
#[tokio::test]
async fn production_encrypted_first_download_uses_only_request_read_secret_and_still_commits_actual_model_receipt(
) {
    let source = Env::new();
    let service = production_service(&source);
    let p = service
        .preview(
            source.request(),
            ServiceSecretsV2 {
                read_passphrase: None,
                write_passphrase: Some(Zeroizing::new("service-synthetic-protection".into())),
            },
        )
        .await
        .unwrap();
    service
        .confirm(ConfirmRequestV2 {
            session_id: p.session_id,
            plan_digest: p.plan_digest,
        })
        .await
        .unwrap();
    let mut target = Env::new();
    target.remote = source.remote.clone();
    empty_existing_fixture(&target);
    let service = production_service(&target);
    let mut request = source.request();
    request.forced_direction = None;
    assert_eq!(
        service
            .preview(request.clone(), ServiceSecretsV2::default())
            .await
            .err()
            .unwrap()
            .code,
        ErrorCode::SyncPassphraseRequired
    );
    let p = service
        .preview(
            request,
            ServiceSecretsV2 {
                read_passphrase: Some(Zeroizing::new("service-synthetic-protection".into())),
                write_passphrase: None,
            },
        )
        .await
        .unwrap();
    let ServicePlanV2::Download(plan) = p.plan.clone() else {
        panic!("download")
    };
    let result = service
        .confirm(ConfirmRequestV2 {
            session_id: p.session_id,
            plan_digest: p.plan_digest,
        })
        .await
        .unwrap();
    assert_eq!(result.local_apply_status, LocalApplyStatus::Applied);
    assert!(target
        .root
        .join("manager-state/model-receive-v2")
        .join(plan.operation_id)
        .join("receipt.json")
        .is_file());
    for file in std::fs::read_dir(target.root.join("manager-state/sync-service-v2")).unwrap() {
        let file = file.unwrap();
        if file.path().is_file() {
            assert!(
                !String::from_utf8_lossy(&std::fs::read(file.path()).unwrap())
                    .contains("service-synthetic-protection")
            );
        }
    }
}
#[tokio::test]
async fn production_initialization_ambiguity_exposes_readonly_candidates_and_can_explicitly_recover_a_chosen_branch(
) {
    let source = Env::new();
    let upload = production_service(&source);
    let p = upload
        .preview(source.request(), ServiceSecretsV2::default())
        .await
        .unwrap();
    upload
        .confirm(ConfirmRequestV2 {
            session_id: p.session_id,
            plan_digest: p.plan_digest,
        })
        .await
        .unwrap();
    let second = Generation {
        format: "OCXD_CONFIG_GENERATION".into(),
        format_version: 1,
        generation_id: "77777777-7777-4777-8777-777777777777".into(),
    };
    source.remote.files.lock().unwrap().insert(
        SyncRelativePath::generation(&second.generation_id)
            .unwrap()
            .as_str()
            .into(),
        store::canonical(&second).unwrap(),
    );
    let puts = source.remote.puts.load(Ordering::SeqCst);
    let mut target = Env::new();
    target.remote = source.remote.clone();
    empty_existing_fixture(&target);
    let service = production_service(&target);
    let catalog = service.catalog(None).await.unwrap();
    assert_eq!(catalog.cloud_state, RemoteState::InitializationAmbiguous);
    assert!(!catalog.initialized);
    assert!(!catalog.observation.recovery_candidates.is_empty());
    let selected = &catalog.observation.recovery_candidates[0];
    let request = source.request();
    let recovery = service
        .recovery_preview(
            RecoveryRequestV2 {
                selection: request.selection,
                mcp_clients: request.mcp_clients,
                assets_reviewed: request.assets_reviewed,
                generation: selected.generation.clone(),
                head: selected.head.clone(),
                mode: RecoveryMode::Download,
                backup_before_update: true,
            },
            None,
        )
        .await
        .unwrap();
    assert!(!target.home.join("config.json").exists());
    let fact = service
        .recovery_confirm(ConfirmRequestV2 {
            session_id: recovery.session_id,
            plan_digest: recovery.plan.plan_digest.clone(),
        })
        .await
        .unwrap();
    assert_eq!(fact.outcome, RecoveryOutcome::Applied);
    assert_eq!(source.remote.puts.load(Ordering::SeqCst), puts);
    assert!(target.home.join("config.json").is_file());
    assert_eq!(
        service
            .recovery_result(recovery.plan.recovery_plan_id)
            .await
            .unwrap()
            .outcome,
        RecoveryOutcome::Applied
    );
}
#[tokio::test]
async fn production_backup_failure_preserves_actual_target_and_retries_same_operation_after_protection_restored(
) {
    let source = Env::new();
    let mut request = source.request();
    crate::modules::preferences::PreferencesStore::new(&source.root)
        .save(&crate::modules::preferences::Preferences::default())
        .unwrap();
    request.selection.manager.preferences = true;
    let upload = production_service(&source);
    let p = upload
        .preview(request.clone(), ServiceSecretsV2::default())
        .await
        .unwrap();
    upload
        .confirm(ConfirmRequestV2 {
            session_id: p.session_id,
            plan_digest: p.plan_digest,
        })
        .await
        .unwrap();
    let mut target = Env::new();
    target.remote = source.remote.clone();
    empty_existing_fixture(&target);
    let prefs = crate::modules::preferences::Preferences {
        theme: "dark".into(),
        ..Default::default()
    };
    crate::modules::preferences::PreferencesStore::new(&target.root)
        .save(&prefs)
        .unwrap();
    let preference_path = target
        .root
        .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH);
    let before = std::fs::read(&preference_path).unwrap();
    let download = production_service(&target);
    request.forced_direction = Some(ServiceDirectionV2::Download);
    let p = download
        .preview(request, ServiceSecretsV2::default())
        .await
        .unwrap();
    let ServicePlanV2::Download(plan) = p.plan.clone() else {
        panic!("download")
    };
    std::fs::write(
        target.root.join("backups"),
        b"synthetic protection obstruction",
    )
    .unwrap();
    let confirmation = ConfirmRequestV2 {
        session_id: p.session_id.clone(),
        plan_digest: p.plan_digest.clone(),
    };
    let err = download.confirm(confirmation.clone()).await.err().unwrap();
    assert!(matches!(
        err.code,
        ErrorCode::SyncBackupFailed | ErrorCode::SyncLocalStateIo
    ));
    assert_eq!(std::fs::read(&preference_path).unwrap(), before);
    assert!(!target.home.join("config.json").exists());
    assert!(!target
        .root
        .join("manager-state/model-receive-v2")
        .join(&plan.operation_id)
        .join("receipt.json")
        .exists());
    std::fs::remove_file(target.root.join("backups")).unwrap();
    let result = download.confirm(confirmation).await.unwrap();
    assert_eq!(result.operation_id, plan.operation_id);
    assert_eq!(result.local_apply_status, LocalApplyStatus::Applied);
    assert!(target.home.join("config.json").exists());
}
#[tokio::test]
async fn production_recovery_export_reads_chosen_branch_and_preserves_all_business_targets_and_remote_files(
) {
    let source = Env::new();
    let upload = production_service(&source);
    let p = upload
        .preview(source.request(), ServiceSecretsV2::default())
        .await
        .unwrap();
    upload
        .confirm(ConfirmRequestV2 {
            session_id: p.session_id,
            plan_digest: p.plan_digest,
        })
        .await
        .unwrap();
    let marker = Generation {
        format: "OCXD_CONFIG_GENERATION".into(),
        format_version: 1,
        generation_id: "88888888-8888-4888-8888-888888888888".into(),
    };
    source.remote.files.lock().unwrap().insert(
        SyncRelativePath::generation(&marker.generation_id)
            .unwrap()
            .as_str()
            .into(),
        store::canonical(&marker).unwrap(),
    );
    let remote_before = source.remote.files.lock().unwrap().clone();
    let mut target = Env::new();
    target.remote = source.remote.clone();
    empty_existing_fixture(&target);
    let service = production_service(&target);
    let catalog = service.catalog(None).await.unwrap();
    let candidate = &catalog.observation.recovery_candidates[0];
    let request = source.request();
    let p = service
        .recovery_preview(
            RecoveryRequestV2 {
                selection: request.selection,
                mcp_clients: request.mcp_clients,
                assets_reviewed: request.assets_reviewed,
                generation: candidate.generation.clone(),
                head: candidate.head.clone(),
                mode: RecoveryMode::Export,
                backup_before_update: true,
            },
            None,
        )
        .await
        .unwrap();
    let fact = service
        .recovery_confirm(ConfirmRequestV2 {
            session_id: p.session_id,
            plan_digest: p.plan.plan_digest,
        })
        .await
        .unwrap();
    assert_eq!(fact.outcome, RecoveryOutcome::Exported);
    let export = target
        .root
        .join("exports")
        .join(format!("{}.ocxdconf", fact.artifact_id.unwrap()));
    let bytes = std::fs::read(export).unwrap();
    file_objects_v2::decode(
        &bytes,
        None,
        &file_objects_v2::supported_schema(),
        Some(&host::PayloadAuthorizationV2),
    )
    .unwrap();
    assert!(!target.home.join("config.json").exists());
    assert!(!target
        .root
        .join("manager-state/sync-identity-mappings-v2.json")
        .exists());
    assert_eq!(*source.remote.files.lock().unwrap(), remote_before);
}
#[test]
fn selected_key_and_mcp_secret_only_changes_are_visible_as_updates_without_secret_values_or_hashes()
{
    let e = Env::new();
    let h = ConnectionHandle {
        id: "fixture".into(),
        binding_id: store::digest(b"namespace"),
        revision: store::digest(b"revision"),
    };
    let base = e.backend.inventory(&h).unwrap().objects;
    let mut before = base.into_captured();
    let id = before.models.payload.policy.as_ref().unwrap().channels[0]
        .id
        .clone();
    before
        .channel_keys
        .insert(id.clone(), Zeroizing::new("first-synthetic-key".into()))
        .unwrap();
    let first = crate::modules::sync::portable_assets::split_mcp(
        crate::modules::extensions::discovery::RawServerDefinition {
            name: "secret-fixture".into(),
            value: json!({"command":"fixture","env":{"TOKEN":"first-synthetic-mcp"}}),
        },
        true,
        true,
    )
    .unwrap();
    before.assets.mcps = vec![first];
    let before = SavedObjects::capture(before, Some(&host::PayloadAuthorizationV2)).unwrap();
    let mut after = before.partitions().models.clone();
    after.created_at = "2026-11-06T16:00:00Z".into();
    let bytes =
        file_objects_v2::encode_document(&before, Some(&host::PayloadAuthorizationV2)).unwrap();
    let mut after_objects =
        file_objects_v2::decode_document(&bytes, Some(&host::PayloadAuthorizationV2))
            .unwrap()
            .into_captured();
    after_objects.models = after;
    let mut keys = super::super::file_objects_v2::SecretValues::new();
    keys.insert(id, Zeroizing::new("second-synthetic-key".into()))
        .unwrap();
    after_objects.channel_keys = keys;
    after_objects.assets.mcps[0]
        .secrets
        .0
        .insert("/env/TOKEN".into(), "second-synthetic-mcp".into());
    let after = SavedObjects::capture(after_objects, Some(&host::PayloadAuthorizationV2)).unwrap();
    let mut selection = e.backend.selection.clone();
    selection.channels[0].include_key = true;
    selection.mcp_sources = vec![McpSourceSelectionV2 {
        name: "secret-fixture".into(),
        definition_digest: super::super::objects_overlay_v2::mcp_source_digest(
            &after.partitions().assets.mcps[0],
        )
        .unwrap(),
    }];
    selection.extension_credentials.mcp_names = vec!["secret-fixture".into()];
    let diff = semantic_diff(&selection, &after, Some(&before), DirectionV2::Upload).unwrap();
    assert!(diff
        .iter()
        .filter(|d| matches!(d.kind, ObjectKindV2::Channel | ObjectKindV2::Mcp))
        .all(|d| matches!(d.status, DiffStatusV2::Update)));
    let public = serde_json::to_string(&diff).unwrap();
    for value in [
        "first-synthetic-key",
        "second-synthetic-key",
        "first-synthetic-mcp",
        "second-synthetic-mcp",
    ] {
        assert!(!public.contains(value));
        assert!(!public.contains(&store::digest(value.as_bytes())));
    }
}
#[tokio::test]
async fn auth_mode_none_first_receive_keeps_actual_public_direction_on_second_service_preview() {
    let source = Env::new();
    let raw_path = source.home.join("config.json");
    let mut raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&raw_path).unwrap()).unwrap();
    raw["providers"]["fixture"]["authMode"] = json!("none");
    std::fs::write(&raw_path, serde_json::to_vec(&raw).unwrap()).unwrap();
    let upload = production_service(&source);
    let p = upload
        .preview(source.request(), ServiceSecretsV2::default())
        .await
        .unwrap();
    upload
        .confirm(ConfirmRequestV2 {
            session_id: p.session_id,
            plan_digest: p.plan_digest,
        })
        .await
        .unwrap();
    let puts = source.remote.puts.load(Ordering::SeqCst);
    let mut target = Env::new();
    target.remote = source.remote.clone();
    empty_existing_fixture(&target);
    let service = production_service(&target);
    let mut request = source.request();
    request.forced_direction = Some(ServiceDirectionV2::Download);
    let p = service
        .preview(request.clone(), ServiceSecretsV2::default())
        .await
        .unwrap();
    service
        .confirm(ConfirmRequestV2 {
            session_id: p.session_id,
            plan_digest: p.plan_digest,
        })
        .await
        .unwrap();
    let after: serde_json::Value =
        serde_json::from_slice(&std::fs::read(target.home.join("config.json")).unwrap()).unwrap();
    let saved_mode = after["providers"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap()
        .get("authMode")
        .cloned();
    let backend = ProductionServiceBackendV2::new(
        target.root.clone(),
        target.home.clone(),
        Arc::new(capture::SystemCaptureCredentialsV2),
    )
    .unwrap();
    let (handle, _) = service.active().await.unwrap();
    let graph = service.graph(&handle).await.unwrap();
    let payload = service.cloud(&graph, None, &handle).await.unwrap().unwrap();
    let cloud = file_objects_v2::decode(
        &payload.bytes,
        None,
        &file_objects_v2::supported_schema(),
        Some(&host::PayloadAuthorizationV2),
    )
    .unwrap()
    .objects;
    let compose = backend.compose(&request, &handle, Some(&cloud), None);
    assert!(
        compose.is_ok(),
        "authMode none second preview: persisted authMode={saved_mode:?}, compose code={:?}",
        compose.err().map(|e| e.code)
    );
    let p = service
        .preview(request, ServiceSecretsV2::default())
        .await
        .unwrap();
    assert_eq!(source.remote.puts.load(Ordering::SeqCst), puts);
    assert!(p.no_change || matches!(p.plan, ServicePlanV2::Download(_)));
}
struct CatalogNeverCredentials;
impl capture::CaptureCredentialsV2 for CatalogNeverCredentials {
    fn channel_key(
        &self,
        _: &Path,
        _: &capture::SelectedKeySourceV2<'_>,
    ) -> ModelResult<Zeroizing<String>> {
        panic!("catalog must not read selected key")
    }
    fn authorization(
        &self,
        _: &Path,
        _: &str,
        _: &serde_json::Value,
        _: &serde_json::Value,
    ) -> ModelResult<super::super::provider_auth::PortableAuthorization> {
        panic!("catalog must not read authorization")
    }
    fn webdav_password(&self, _: &Path, _: &str) -> ModelResult<Zeroizing<String>> {
        panic!("catalog must not read endpoint password")
    }
}
#[test]
fn local_file_catalog_needs_no_endpoint_and_preserves_actual_saved_ids_models_and_exact_template_refs(
) {
    let e = Env::new();
    std::fs::remove_file(
        e.root
            .join(crate::modules::sync::config::SYNC_ENDPOINTS_RELATIVE_PATH),
    )
    .unwrap();
    let mut raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(e.home.join("config.json")).unwrap()).unwrap();
    raw["providers"]["fixture"]["apiKey"] = json!("synthetic-unselected-local-key");
    raw["codexAccounts"] = json!([{
        "id":"work-slot", "email":"metadata@example.test", "chatgptAccountId":"work-tenant"
    }]);
    let legacy_id = uuid::Uuid::new_v4().to_string();
    let blocked_id = uuid::Uuid::new_v4().to_string();
    std::fs::write(
        e.root.join(crate::modules::sync::portable_accounts::IMPORTED),
        serde_json::to_vec(&json!([
            {"id":legacy_id,"provider":"chatgpt","subject":null,"label":"Legacy","email":null},
            {"id":blocked_id,"provider":"muse","subject":"blocked-subject","label":"Foreign owner","email":null,
             "origin":{"version":1,"accountPlatform":"muse","owner":"platform-native","authKind":"oauth",
                "sourceNamespace":"oauth-account-pool","sourceAccountId":"foreign-slot","tenant":null}}
        ])).unwrap(),
    ).unwrap();
    std::fs::write(
        e.home.join("config.json"),
        serde_json::to_vec(&raw).unwrap(),
    )
    .unwrap();
    let before = std::fs::read(e.home.join("config.json")).unwrap();
    let backend = ProductionServiceBackendV2::new(
        e.root.clone(),
        e.home.clone(),
        Arc::new(CatalogNeverCredentials),
    )
    .unwrap();
    let catalog = backend.local_file_catalog().unwrap();
    assert_eq!(catalog.source, "saved");
    assert!(catalog.webdav_connections.is_empty());
    let channel = catalog
        .entries
        .iter()
        .find(|e| e.kind == ObjectKindV2::Channel)
        .unwrap();
    assert_eq!(
        channel.id,
        e.backend.selection.channels[0].portable_channel_id
    );
    assert!(channel.secret_available);
    assert!(channel.key_available);
    assert!(!channel.authorization_available);
    assert_eq!(catalog.accounts.len(), 3);
    let accounts = &catalog.accounts;
    let local = accounts.iter().find(|a| a.local_source).unwrap();
    assert!(local.information_supported);
    assert_eq!(local.account.provider, "chatgpt");
    assert_eq!(
        local.account.email.as_deref(),
        Some("metadata@example.test")
    );
    let origin = local.account.origin.as_ref().unwrap();
    assert_eq!(origin.account_platform, "chatgpt");
    assert_eq!(origin.owner, "opencodex");
    assert_eq!(origin.source_namespace, "codex-account-pool");
    assert_eq!(origin.source_account_id, "work-slot");
    assert_eq!(origin.tenant.as_deref(), Some("work-tenant"));
    assert_eq!(origin.auth_kind, "oauth");
    let legacy = accounts.iter().find(|a| a.account.id == legacy_id).unwrap();
    assert!(legacy.information_supported);
    assert!(legacy.account.origin.is_none());
    let blocked = accounts
        .iter()
        .find(|a| a.account.id == blocked_id)
        .unwrap();
    assert!(!blocked.information_supported);
    assert!(!catalog.entries.iter().any(|entry| entry.id == blocked_id));
    for row in accounts {
        assert_eq!(
            row.information_supported,
            crate::modules::sync::portable_accounts::information_supported(&row.account)
        );
    }
    assert_eq!(channel.models[0].raw_model_id, "fixture-model");
    assert!(channel.available_local);
    assert!(!channel.available_cloud);
    assert_eq!(std::fs::read(e.home.join("config.json")).unwrap(), before);
    assert_eq!(e.remote.gets.load(Ordering::SeqCst), 0);
    assert!(!e
        .root
        .join("manager-state/sync-identity-mappings-v2.json")
        .exists());
    for template in catalog
        .entries
        .iter()
        .filter(|e| e.kind == ObjectKindV2::Template)
    {
        assert!(template
            .revisions
            .iter()
            .all(|r| r.template_id == template.id
                && super::super::schema_v2::valid_digest(&r.digest)));
    }
    let public = serde_json::to_string(&catalog).unwrap();
    assert!(!public.contains("synthetic-unselected-local-key"));
    assert!(!public.contains("credentialRef"));
    assert!(!public.contains("accessToken"));
    assert!(!public.contains("refreshToken"));
    let value = serde_json::to_value(&catalog).unwrap();
    assert_eq!(
        value["accounts"][0]["informationSupported"],
        catalog.accounts[0].information_supported
    );
    assert!(value["entries"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["keyAvailable"] == true && entry["authorizationAvailable"] == false));
}
#[test]
fn local_file_catalog_lists_webdav_candidates_and_password_availability_without_reading_values() {
    let e = Env::new();
    let endpoint = crate::modules::sync::config::SyncConfigStore::new(&e.root)
        .load()
        .unwrap()
        .active()
        .unwrap()
        .clone();
    let account = crate::infrastructure::keychain::account_key(
        crate::infrastructure::keychain::WEBDAV_CREDENTIAL_PURPOSE,
        &endpoint.credential_ref.ref_id,
    )
    .unwrap();
    let dir = e.root.join("manager-state/credentials");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{account}.json"));
    std::fs::write(
        &path,
        b"deliberately not decoded unselected synthetic password file",
    )
    .unwrap();
    let backend = ProductionServiceBackendV2::new(
        e.root.clone(),
        e.home.clone(),
        Arc::new(CatalogNeverCredentials),
    )
    .unwrap();
    let catalog = backend.local_file_catalog().unwrap();
    assert_eq!(catalog.webdav_connections.len(), 1);
    let candidate = &catalog.webdav_connections[0];
    assert_eq!(candidate.id, "default");
    assert!(candidate.password_available);
    let channel = catalog
        .entries
        .iter()
        .find(|row| row.kind == ObjectKindV2::Channel)
        .unwrap();
    assert!(!channel.key_available);
    assert!(!channel.authorization_available);
    let mut raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(e.home.join("config.json")).unwrap()).unwrap();
    raw["providers"]["fixture"]["headers"] =
        json!({"X-Fixture":"synthetic-unselected-authorization"});
    std::fs::write(
        e.home.join("config.json"),
        serde_json::to_vec(&raw).unwrap(),
    )
    .unwrap();
    let catalog = backend.local_file_catalog().unwrap();
    let channel = catalog
        .entries
        .iter()
        .find(|row| row.kind == ObjectKindV2::Channel)
        .unwrap();
    assert!(channel.secret_available);
    assert!(!channel.key_available);
    assert!(channel.authorization_available);
    let public = serde_json::to_string(&catalog).unwrap();
    assert!(!public.contains("synthetic-unselected-authorization"));
    assert!(!public.contains("not decoded"));
    assert!(!public.contains(CREDS));
    assert!(!public.contains("accountKey"));
    assert_eq!(e.remote.gets.load(Ordering::SeqCst), 0);
    assert_eq!(
        std::fs::read(path).unwrap(),
        b"deliberately not decoded unselected synthetic password file"
    );
}
#[test]
fn local_file_catalog_keeps_conflicting_mcp_per_source_choices_without_endpoint_or_default_selection(
) {
    let e = Env::new();
    std::fs::remove_file(
        e.root
            .join(crate::modules::sync::config::SYNC_ENDPOINTS_RELATIVE_PATH),
    )
    .unwrap();
    std::fs::create_dir_all(e.home.join(".codex")).unwrap();
    std::fs::create_dir_all(e.home.join(".hermes")).unwrap();
    std::fs::write(
        e.home.join(".codex/config.toml"),
        "[mcp_servers.localfixture]\ncommand = 'codex-command'\n[mcp_servers.localfixture.env]\nTOKEN = 'synthetic-unselected-mcp-token'\n",
    )
    .unwrap();
    std::fs::write(
        e.home.join(".hermes/config.yaml"),
        "mcp_servers:\n  localfixture:\n    command: hermes-command\n",
    )
    .unwrap();
    let backend = ProductionServiceBackendV2::new(
        e.root.clone(),
        e.home.clone(),
        Arc::new(CatalogNeverCredentials),
    )
    .unwrap();
    let catalog = backend.local_file_catalog().unwrap();
    let mcp = catalog
        .mcp_sources
        .iter()
        .find(|m| m.name == "localfixture")
        .unwrap();
    assert_eq!(mcp.sources.len(), 2);
    assert!(mcp
        .sources
        .iter()
        .all(|s| super::super::schema_v2::valid_digest(&s.definition_digest)));
    assert!(!mcp.differing_fields.is_empty());
    let row = catalog
        .entries
        .iter()
        .find(|e| e.kind == ObjectKindV2::Mcp && e.id == "localfixture")
        .unwrap();
    assert!(row.secret_available);
    assert!(!row.key_available);
    assert!(!row.authorization_available);
    assert!(!serde_json::to_string(&catalog)
        .unwrap()
        .contains("synthetic-unselected-mcp-token"));
    assert_eq!(e.remote.gets.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn production_same_uuid_second_sync_is_semantic_noop_and_explicit_download_restores_existing_actual_source(
) {
    let source = Env::new();
    // Env's begin_draft exists ONLY to establish the source UUID registry.
    // This scenario explicitly has no user edit buffer to preserve; leave the
    // real receiver's unsaved-draft refusal unchanged for actual dirty drafts.
    std::fs::remove_file(
        source
            .root
            .join(crate::modules::model_config::store_v2::DRAFT_PATH),
    )
    .unwrap();

    let service = production_service(&source);
    let p = service
        .preview(source.request(), ServiceSecretsV2::default())
        .await
        .unwrap();
    service
        .confirm(ConfirmRequestV2 {
            session_id: p.session_id,
            plan_digest: p.plan_digest,
        })
        .await
        .unwrap();
    let puts = source.remote.puts.load(Ordering::SeqCst);
    let repeated = service
        .preview(source.request(), ServiceSecretsV2::default())
        .await
        .unwrap();
    assert!(repeated.no_change);
    assert!(matches!(repeated.plan, ServicePlanV2::NoChange));
    assert_eq!(source.remote.puts.load(Ordering::SeqCst), puts);
    let path = source.home.join("config.json");
    let mut raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    raw["providers"]["fixture"]["modelContextWindows"]["fixture-model"] = json!(16384);
    std::fs::write(&path, serde_json::to_vec(&raw).unwrap()).unwrap();
    let mut request = source.request();
    request.forced_direction = Some(ServiceDirectionV2::Download);
    let p = service
        .preview(request, ServiceSecretsV2::default())
        .await
        .unwrap();
    let ServicePlanV2::Download(plan) = p.plan.clone() else {
        panic!("existing exact same UUID must allow explicit receive preview")
    };
    let result = service
        .confirm(ConfirmRequestV2 {
            session_id: p.session_id,
            plan_digest: p.plan_digest,
        })
        .await
        .unwrap();
    assert_eq!(result.local_apply_status, LocalApplyStatus::Applied);
    assert_eq!(result.operation_id, plan.operation_id);
    assert_eq!(source.remote.puts.load(Ordering::SeqCst), puts);
    let raw: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(
        raw["providers"]["fixture"]["modelContextWindows"]["fixture-model"],
        8192
    );
}
#[tokio::test]
async fn real_model_applied_interruption_reopens_with_dual_proofs_and_only_finishes_original_nonmodel_participants(
) {
    let source = Env::new();
    let mut request = source.request();
    add_full_saved_scope(&source, &mut request);
    let upload = production_service(&source);
    let p = upload
        .preview(request.clone(), ServiceSecretsV2::default())
        .await
        .unwrap();
    upload
        .confirm(ConfirmRequestV2 {
            session_id: p.session_id,
            plan_digest: p.plan_digest,
        })
        .await
        .unwrap();
    let mut target = Env::new();
    target.remote = source.remote.clone();
    empty_existing_fixture(&target);
    let mut backend = ProductionServiceBackendV2::new(
        target.root.clone(),
        target.home.clone(),
        Arc::new(capture::SystemCaptureCredentialsV2),
    )
    .unwrap();
    let root = target.root.clone();
    let interrupted = Arc::new(AtomicBool::new(false));
    let observed = interrupted.clone();
    backend.after_model_fault = Some(Arc::new(move |id| {
        assert!(root
            .join("manager-state/model-receive-v2")
            .join(id)
            .join("receipt.json")
            .is_file());
        assert!(root
            .join("manager-state/sync-identity-mappings-v2.json")
            .is_file());
        observed.store(true, Ordering::SeqCst);
        Err(invalid(
            "test_remaining_interruption",
            "deterministic post-model participant fault",
        ))
    }));
    let service = SyncServiceV2::new(
        target.root.clone(),
        Arc::new(backend),
        target.remote.clone(),
        Arc::new(CredentialOwner),
    )
    .unwrap();
    request.forced_direction = None;
    let p = service
        .preview(request, ServiceSecretsV2::default())
        .await
        .unwrap();
    let ServicePlanV2::Download(plan) = p.plan.clone() else {
        panic!("download")
    };
    let operation = plan.operation_id.clone();
    assert!(service
        .confirm(ConfirmRequestV2 {
            session_id: p.session_id,
            plan_digest: p.plan_digest
        })
        .await
        .is_err());
    assert!(interrupted.load(Ordering::SeqCst));
    let receipt_path = target
        .root
        .join("manager-state/model-receive-v2")
        .join(&operation)
        .join("receipt.json");
    let model_receipt = std::fs::read(&receipt_path).unwrap();
    assert!(!target
        .root
        .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH)
        .exists());
    let original = service.result(operation.clone()).await.unwrap();
    assert_ne!(original.local_apply_status, LocalApplyStatus::Applied);
    drop(service);
    let external = b"{ deliberately invalid later source; remaining must not read/capture it";
    std::fs::write(target.home.join("config.json"), external).unwrap();
    let puts = target.remote.puts.load(Ordering::SeqCst);
    let gets = target.remote.gets.load(Ordering::SeqCst);
    let reopened = production_service(&target);
    let completed = reopened
        .resume_receive(operation.clone(), None)
        .await
        .unwrap();
    assert_eq!(completed.operation_id, operation);
    assert_eq!(completed.local_apply_status, LocalApplyStatus::Applied);
    assert_eq!(completed.relation.local, LocalRelation::Unknown);
    assert_eq!(gets, target.remote.gets.load(Ordering::SeqCst));
    assert_eq!(puts, target.remote.puts.load(Ordering::SeqCst));
    assert_eq!(model_receipt, std::fs::read(receipt_path).unwrap());
    assert_eq!(
        std::fs::read(target.home.join("config.json")).unwrap(),
        external
    );
    assert!(target
        .root
        .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH)
        .is_file());
    assert!(target
        .root
        .join("manager-state/mcp-definitions.json")
        .is_file());
    assert!(
        target
            .home
            .join(".agents/skills/service-fixture/SKILL.md")
            .exists()
            || target
                .root
                .join("manager-state/skills-store/service-fixture/SKILL.md")
                .exists()
    );
    assert_eq!(
        reopened
            .result(operation.clone())
            .await
            .unwrap()
            .local_apply_status,
        LocalApplyStatus::Applied
    );
    let context_name = format!("{operation}.download-context.json");
    let mut context: DownloadContextV2 = reopened.files.read(&context_name).unwrap().unwrap();
    let original_after = context.mapping_after.clone();
    context.mapping_after.push(b' ');
    reopened.files.write(&context_name, context).unwrap();
    assert!(reopened
        .resume_receive(operation.clone(), None)
        .await
        .is_err());
    let mut context: DownloadContextV2 = reopened.files.read(&context_name).unwrap().unwrap();
    context.mapping_after = original_after;
    reopened.files.write(&context_name, context).unwrap();
    let repeated = reopened.resume_receive(operation, None).await.unwrap();
    assert_eq!(repeated.local_apply_status, LocalApplyStatus::Applied);
    assert_eq!(gets, target.remote.gets.load(Ordering::SeqCst));
}

#[tokio::test]
async fn explicit_download_updates_an_existing_different_mcp_definition_in_managed_store_only() {
    use crate::modules::sync::portable_assets;
    let source = Env::new();
    let mut upload_request = source.request();
    add_full_saved_scope(&source, &mut upload_request);
    upload_request.selection = SelectionV2 {
        mcp_sources: upload_request.selection.mcp_sources.clone(),
        ..Default::default()
    };
    let upload = production_service(&source);
    let preview = upload
        .preview(upload_request, ServiceSecretsV2::default())
        .await
        .unwrap();
    upload
        .confirm(ConfirmRequestV2 {
            session_id: preview.session_id,
            plan_digest: preview.plan_digest,
        })
        .await
        .unwrap();
    let mut target = Env::new();
    target.remote = source.remote.clone();
    std::fs::create_dir_all(target.home.join(".codex")).unwrap();
    let config = target.home.join(".codex/config.toml");
    std::fs::write(
        &config,
        "[mcp_servers.service_fixture]\ncommand = 'different-local-command'\nargs = []\n",
    )
    .unwrap();
    let unselected = target.home.join(".claude.json");
    let unselected_bytes =
        br#"{"mcpServers":{"service_fixture":{"command":"unselected-client-command","args":[]}}}"#;
    std::fs::write(&unselected, unselected_bytes).unwrap();
    std::fs::remove_file(
        target
            .root
            .join(crate::modules::model_config::store_v2::DRAFT_PATH),
    )
    .unwrap();
    let sources = BTreeMap::from([("service_fixture".into(), "codex".into())]);
    let (assets, conflicts) = portable_assets::export_with_credential_scope(
        &target.root,
        &target.home,
        &[],
        &["service_fixture".into()],
        &sources,
        false,
        false,
        Some(&[]),
    )
    .unwrap();
    assert!(conflicts.is_empty());
    let request = PreviewRequestV2 {
        selection: SelectionV2 {
            mcp_sources: vec![McpSourceSelectionV2 {
                name: "service_fixture".into(),
                definition_digest: super::super::objects_overlay_v2::mcp_source_digest(
                    &assets.mcps[0],
                )
                .unwrap(),
            }],
            ..Default::default()
        },
        mcp_clients: sources,
        assets_reviewed: false,
        selected_head: None,
        forced_direction: Some(ServiceDirectionV2::Download),
        backup_before_update: true,
    };
    let client = production_service(&target);
    let before = std::fs::read(&config).unwrap();
    let puts = source.remote.puts.load(Ordering::SeqCst);
    let preview = client
        .preview(request, ServiceSecretsV2::default())
        .await
        .unwrap();
    assert!(matches!(preview.chosen_direction, DirectionV2::Download));
    assert_eq!(std::fs::read(&config).unwrap(), before);
    let actual = client
        .confirm(ConfirmRequestV2 {
            session_id: preview.session_id,
            plan_digest: preview.plan_digest,
        })
        .await
        .unwrap();
    assert!(actual.local_apply_status == LocalApplyStatus::Applied);
    let after = std::fs::read_to_string(config).unwrap();
    assert!(after.contains("different-local-command"));
    let managed =
        std::fs::read_to_string(target.root.join("manager-state/mcp-definitions.json")).unwrap();
    assert!(managed.contains("service-fixture-command"));
    assert_eq!(source.remote.puts.load(Ordering::SeqCst), puts);
    assert_eq!(std::fs::read(unselected).unwrap(), unselected_bytes);
}
