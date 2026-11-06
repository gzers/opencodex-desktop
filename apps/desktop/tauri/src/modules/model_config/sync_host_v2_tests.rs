//! Real SavedObjects/v6 model/overlay/protection types. Only the capture owner
//! and remote transport boundaries are injected; no model/protection substitute.
use super::super::content_versions::Version;
use super::super::{
    file_objects_v2::{CapturedObjects, SecretValues},
    portable_v2::{self, ModelPayloadV2, PortableChannelV2, PortableModelsV2, PortablePolicyV2},
    schema::{ModelRules, ModelSelection},
    schema_v2::*,
    source_v2::source_from_provider_v2,
};
use super::digest;
use super::*;
use serde_json::json;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
};
const A: &str = "11111111-1111-4111-8111-111111111111";
const B: &str = "22222222-2222-4222-8222-222222222222";
const O: &str = "33333333-3333-4333-8333-333333333333";
const TIME: &str = "2026-11-06T10:00:00Z";
fn channel(id: &str, window: u64) -> PortableChannelV2 {
    PortableChannelV2 {
        id: id.into(),
        label: id.into(),
        connection: Some(portable_v2::PortableConnectionV2 {
            nodes: vec![],
            active_node_id: String::new(),
            enabled: true,
            allow_private_network: false,
            auth_mode: Some("key".into()),
            provider: None,
        }),
        model_subset: None,
        family_scope: None,
        models: vec![ModelConfigV2 {
            raw_model_id: "fixture-model".into(),
            label: "fixture-model".into(),
            observed: true,
            source: source_from_provider_v2(
                "fixture",
                "fixture-model",
                &json!({"adapter":"openai-chat"}),
                &digest(b"source"),
                SourceKindV2::ImportedFile,
            )
            .unwrap(),
            explicit_overrides: ModelRules {
                context_window: Field::Custom(window),
                ..Default::default()
            },
            binding: None,
        }],
        template_refs: vec![],
        selection: ModelSelection::All,
        model_order: vec!["fixture-model".into()],
        defaults: ModelRules::default(),
        family_overrides: BTreeMap::new(),
        naming: BTreeMap::new(),
        namespace: None,
    }
}
fn objects(window: u64, other: bool) -> SavedObjects {
    let mut channels = vec![channel(A, window)];
    if other {
        channels.push(channel(B, 4096));
    }
    let payload = ModelPayloadV2 {
        templates: vec![],
        policy: Some(PortablePolicyV2 {
            template_refs: vec![],
            channel_order: channels.iter().map(|c| c.id.clone()).collect(),
            default_channel_id: None,
            channels,
        }),
    };
    SavedObjects::capture(
        CapturedObjects {
            models: PortableModelsV2 {
                format: portable_v2::PORTABLE_FORMAT.into(),
                format_version: 6,
                schema_version: 2,
                created_at: TIME.into(),
                digest: super::super::schema_v2::digest(&canonical_bytes(&payload).unwrap()),
                payload,
            },
            manager: Default::default(),
            assets: Default::default(),
            channel_keys: SecretValues::new(),
            channel_authorizations: BTreeMap::new(),
            webdav_passwords: SecretValues::new(),
            content_versions: vec![],
        },
        None,
    )
    .unwrap()
}
fn selection() -> SelectionV2 {
    let value = json!({"channels":[{"portableChannelId":A,"modelIds":null,"familyIds":null,"includeConnection":true,"includeKey":false,"includeAuthorization":false}],"usage":"remote_sync"});
    serde_json::from_value(value).unwrap()
}
fn stamped(objects: SavedObjects, when: &str) -> SavedObjects {
    let atoms = file_objects_v2::save_state(&objects).unwrap();
    let mut captured = objects.into_captured();
    captured.content_versions = atoms
        .into_iter()
        .map(|(scope_key, content_digest)| Version {
            scope_key,
            content_digest,
            revision: 1,
            saved_at: when.into(),
            time_kind: TimeKind::Saved,
            trusted_floor: Some(when.into()),
            last_event: Some(O.into()),
        })
        .collect();
    SavedObjects::capture(captured, None).unwrap()
}
struct Capture {
    document: Mutex<Zeroizing<Vec<u8>>>,
    calls: AtomicUsize,
    error: Mutex<Option<&'static str>>,
    mcp: Mutex<Digest>,
    root: Mutex<Digest>,
}
impl Capture {
    fn new(objects: SavedObjects) -> Self {
        Self {
            document: Mutex::new(file_objects_v2::encode_document(&objects, None).unwrap()),
            calls: AtomicUsize::new(0),
            error: Mutex::new(None),
            mcp: Mutex::new(digest(b"chosen-client-and-definition")),
            root: Mutex::new(digest(b"root")),
        }
    }
    fn replace(&self, objects: SavedObjects) {
        *self.document.lock().unwrap() = file_objects_v2::encode_document(&objects, None).unwrap();
    }
}
impl CaptureProviderV2 for Capture {
    fn capture_saved(&self, selection: &SelectionV2) -> ModelResult<HostCapturedV2> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if let Some(code) = *self.error.lock().unwrap() {
            return Err(super::super::schema::invalid(
                code,
                "NEVER EXPOSE /private/fixture or synthetic-secret",
            ));
        }
        let objects = file_objects_v2::decode_document(&self.document.lock().unwrap(), None)?;
        let saved_source_digest = file_objects_v2::semantic_digest(&objects)?;
        Ok(HostCapturedV2 {
            objects,
            source_guard: SourceGuard {
                saved_source_digest,
                scope_digest: digest(&super::super::schema_v2::canonical_bytes(selection).unwrap()),
                mcp_sources_digest: self.mcp.lock().unwrap().clone(),
            },
            root_revision: self.root.lock().unwrap().clone(),
        })
    }
}
fn baseline(objects: SavedObjects, passphrase: Option<&str>) -> PayloadInput {
    let encoded = file_objects_v2::encode(&objects, O, TIME, passphrase, None).unwrap();
    PayloadInput {
        head: CommitRef {
            operation_id: O.into(),
            digest: digest(b"exact-verified-commit-bytes"),
        },
        bytes: encoded.bytes().to_vec(),
    }
}
fn upload_preview(provider: &dyn CaptureProviderV2, cloud: Option<&PayloadInput>) -> HostPreviewV2 {
    preview_semantics(
        provider,
        selection(),
        cloud
            .map(CloudMaterialV2::Valid)
            .unwrap_or(CloudMaterialV2::Empty),
        &SessionSecretsV2::default(),
        Some(DirectionV2::Upload),
    )
    .unwrap()
}
#[test]
fn host_codec_uses_actual_overlay_preserves_unselected_and_freezes_one_supported_schema() {
    let provider = Arc::new(Capture::new(objects(8192, false)));
    let cloud = baseline(objects(4096, true), None);
    let preview = upload_preview(provider.as_ref(), Some(&cloud));
    let guard = preview.source_guard.clone();
    let codec = HostPayloadCodecV2::new(provider, preview, SessionSecretsV2::default());
    assert_eq!(
        codec.supported_schemas(),
        vec![file_objects_v2::supported_schema()]
    );
    let frozen = codec.freeze_selected(&guard, &[cloud], None).unwrap();
    let decoded = file_objects_v2::decode(&frozen.bytes, None, &frozen.schema, None).unwrap();
    let channels = &decoded
        .objects
        .models()
        .payload
        .policy
        .as_ref()
        .unwrap()
        .channels;
    assert_eq!(channels.len(), 2);
    assert_eq!(
        channels[0].models[0].explicit_overrides.context_window,
        Field::Custom(8192)
    );
    assert_eq!(channels[1].id, B);
    assert_eq!(
        channels[1].models[0].explicit_overrides.context_window,
        Field::Custom(4096)
    );
}
#[test]
fn original_payload_inspection_never_recaptures_todays_source_or_mcp() {
    let provider = Arc::new(Capture::new(objects(8192, false)));
    let cloud = baseline(objects(4096, false), None);
    let preview = upload_preview(provider.as_ref(), Some(&cloud));
    let codec = HostPayloadCodecV2::new(provider.clone(), preview, SessionSecretsV2::default());
    let before = provider.calls.load(Ordering::SeqCst);
    *provider.error.lock().unwrap() = Some("capture_stale");
    *provider.mcp.lock().unwrap() = digest(b"new-client");
    codec
        .inspect_and_validate(&cloud.bytes, &file_objects_v2::supported_schema())
        .unwrap();
    assert_eq!(before, provider.calls.load(Ordering::SeqCst));
}
#[test]
fn source_or_scope_or_root_change_blocks_new_freeze_with_specific_stable_error() {
    let provider = Arc::new(Capture::new(objects(8192, false)));
    let cloud = baseline(objects(4096, false), None);
    let preview = upload_preview(provider.as_ref(), Some(&cloud));
    let guard = preview.source_guard.clone();
    let codec = HostPayloadCodecV2::new(provider.clone(), preview, SessionSecretsV2::default());
    provider.replace(objects(16384, false));
    assert_eq!(
        codec
            .freeze_selected(&guard, &[cloud], None)
            .err()
            .unwrap()
            .code,
        ErrorCode::SyncSourceChanged
    );
    let provider = Arc::new(Capture::new(objects(8192, false)));
    let preview = upload_preview(provider.as_ref(), None);
    let guard = preview.source_guard.clone();
    let codec = HostPayloadCodecV2::new(provider.clone(), preview, SessionSecretsV2::default());
    *provider.mcp.lock().unwrap() = digest(b"new selected MCP source");
    assert_eq!(
        codec.freeze_selected(&guard, &[], None).err().unwrap().code,
        ErrorCode::SyncScopeChanged
    );
    *provider.mcp.lock().unwrap() = digest(b"chosen-client-and-definition");
    *provider.root.lock().unwrap() = digest(b"new root");
    assert_eq!(
        codec.freeze_selected(&guard, &[], None).err().unwrap().code,
        ErrorCode::SyncPlanChanged
    );
}
#[test]
fn multi_baseline_and_automatic_merged_resolution_are_rejected_not_silently_combined() {
    let provider = Arc::new(Capture::new(objects(8192, false)));
    let first = baseline(objects(4096, false), None);
    let preview = upload_preview(provider.as_ref(), Some(&first));
    let guard = preview.source_guard.clone();
    let codec = HostPayloadCodecV2::new(provider, preview, SessionSecretsV2::default());
    let second = baseline(objects(2048, false), None);
    assert_eq!(
        codec
            .freeze_selected(&guard, &[first, second], None)
            .err()
            .unwrap()
            .code,
        ErrorCode::SyncRemoteConflict
    );
    let r = Resolution {
        selection_mode: SelectionMode::Merged,
        selected_head: None,
        scope_digest: guard.scope_digest.clone(),
        preview_digest: digest(b"not an auto-merge"),
    };
    assert_eq!(
        codec
            .freeze_selected(&guard, &[], Some(&r))
            .err()
            .unwrap()
            .code,
        ErrorCode::SyncRemoteConflict
    );
}
#[test]
fn preview_head_and_payload_binding_rejects_relabelled_or_replaced_bytes() {
    let provider = Arc::new(Capture::new(objects(8192, false)));
    let mut cloud = baseline(objects(4096, false), None);
    let preview = upload_preview(provider.as_ref(), Some(&cloud));
    let guard = preview.source_guard.clone();
    let codec = HostPayloadCodecV2::new(provider, preview, SessionSecretsV2::default());
    cloud.bytes = baseline(objects(2048, false), None).bytes;
    assert_eq!(
        codec
            .freeze_selected(&guard, &[cloud], None)
            .err()
            .unwrap()
            .code,
        ErrorCode::SyncPlanChanged
    );
}
#[test]
fn reliable_complete_save_atoms_choose_saved_later_unknown_time_defaults_cloud() {
    let provider = Capture::new(stamped(objects(8192, false), "2026-11-06T12:00:00Z"));
    let cloud = baseline(stamped(objects(4096, false), "2026-11-06T11:00:00Z"), None);
    let preview = preview_semantics(
        &provider,
        selection(),
        CloudMaterialV2::Valid(&cloud),
        &SessionSecretsV2::default(),
        None,
    )
    .unwrap();
    assert_eq!(preview.decision.chosen_direction, DirectionV2::Upload);
    assert_eq!(
        preview.decision.automatic_reason,
        AutomaticReasonV2::LocalSavedLater
    );
    assert_eq!(
        preview.decision.local_saved_time.as_deref(),
        Some("2026-11-06T12:00:00Z")
    );
    let provider = Capture::new(objects(8192, false));
    let p = preview_semantics(
        &provider,
        selection(),
        CloudMaterialV2::Valid(&cloud),
        &SessionSecretsV2::default(),
        None,
    )
    .unwrap();
    assert_eq!(p.decision.chosen_direction, DirectionV2::Download);
    assert_eq!(
        p.decision.automatic_reason,
        AutomaticReasonV2::UnreliableSaveTime
    );
    assert!(p.decision.local_saved_time.is_none());
}
#[test]
fn confirmed_empty_prefers_upload_but_bad_cloud_never_becomes_empty() {
    let provider = Capture::new(objects(8192, false));
    let p = preview_semantics(
        &provider,
        selection(),
        CloudMaterialV2::Empty,
        &SessionSecretsV2::default(),
        None,
    )
    .unwrap();
    assert_eq!(p.decision.chosen_direction, DirectionV2::Upload);
    assert!(!p.decision.can_download);
    assert!(preview_semantics(
        &provider,
        selection(),
        CloudMaterialV2::Empty,
        &SessionSecretsV2::default(),
        Some(DirectionV2::Download)
    )
    .is_err());
    let bad = PayloadInput {
        head: CommitRef {
            operation_id: O.into(),
            digest: digest(b"commit"),
        },
        bytes: b"{broken".to_vec(),
    };
    assert!(preview_semantics(
        &provider,
        selection(),
        CloudMaterialV2::Valid(&bad),
        &SessionSecretsV2::default(),
        None
    )
    .is_err());
    let auth = failure(ErrorCode::SyncAuthRequired, Step::Observe);
    assert_eq!(
        preview_semantics(
            &provider,
            selection(),
            CloudMaterialV2::Unavailable(auth),
            &SessionSecretsV2::default(),
            None
        )
        .err()
        .unwrap()
        .code,
        ErrorCode::SyncAuthRequired
    );
}
#[test]
fn no_change_is_semantic_and_does_not_reencrypt_or_allow_prepare_freeze() {
    let provider = Arc::new(Capture::new(objects(4096, false)));
    let cloud = baseline(
        objects(4096, false),
        Some("synthetic protection passphrase"),
    );
    let secrets = SessionSecretsV2 {
        read_passphrase: Some(Zeroizing::new("synthetic protection passphrase".into())),
        write_passphrase: Some(Zeroizing::new("different synthetic encryption".into())),
    };
    let preview = preview_semantics(
        provider.as_ref(),
        selection(),
        CloudMaterialV2::Valid(&cloud),
        &secrets,
        None,
    )
    .unwrap();
    assert!(preview.decision.no_change);
    assert_eq!(preview.decision.chosen_direction, DirectionV2::NoChange);
    let guard = preview.source_guard.clone();
    let codec = HostPayloadCodecV2::new(provider, preview, secrets);
    assert_eq!(
        codec
            .freeze_selected(&guard, &[cloud], None)
            .err()
            .unwrap()
            .code,
        ErrorCode::SyncPlanChanged
    );
}
#[test]
fn current_source_facts_preserve_real_capture_errors_without_secret_paths() {
    let provider = Arc::new(Capture::new(objects(8192, false)));
    let current =
        CurrentSourceProviderV2::new(provider.clone(), selection(), digest(b"root")).unwrap();
    current.current_source().unwrap();
    *provider.error.lock().unwrap() = Some("capture_stale");
    let error = current.current_source().unwrap_err();
    assert_eq!(error.code, ErrorCode::SyncSourceChanged);
    let public = serde_json::to_string(&error).unwrap();
    assert!(!public.contains("synthetic-secret"));
    assert!(!public.contains("/private/fixture"));
}
#[test]
fn projected_scope_does_not_borrow_complete_cloud_global_save_time() {
    let provider = Capture::new(stamped(objects(8192, false), "2026-11-06T12:00:00Z"));
    let cloud = baseline(stamped(objects(4096, true), "2026-11-06T11:00:00Z"), None);
    let preview = preview_semantics(
        &provider,
        selection(),
        CloudMaterialV2::Valid(&cloud),
        &SessionSecretsV2::default(),
        None,
    )
    .unwrap();
    assert_eq!(preview.decision.chosen_direction, DirectionV2::Download);
    assert!(preview.decision.cloud_saved_time.is_none());
    assert_eq!(
        preview.decision.automatic_reason,
        AutomaticReasonV2::UnreliableSaveTime
    );
}
const CRED: &str = "cred_44444444-4444-4444-8444-444444444444";
const CRED2: &str = "cred_55555555-5555-4555-8555-555555555555";
fn endpoint(
    id: &str,
    reference: &str,
    url: &str,
) -> crate::modules::sync::config::SyncEndpointConfig {
    use crate::{infrastructure::keychain, modules::sync::config::*};
    SyncEndpointConfig {
        endpoint_id: id.into(),
        url: url.into(),
        remote_path: "/fixture/scope/".into(),
        username: "fixture-user".into(),
        credential_ref: CredentialRef {
            ref_id: reference.into(),
            backend: "keychain".into(),
            service_name: keychain::keychain_service_name().into(),
            account_key: keychain::account_key(keychain::WEBDAV_CREDENTIAL_PURPOSE, reference)
                .unwrap(),
            purpose: keychain::WEBDAV_CREDENTIAL_PURPOSE.into(),
            created_at: TIME.into(),
            updated_at: TIME.into(),
        },
        tls_policy: TLS_POLICY_VERIFY_REQUIRED.into(),
        legacy_encryption: String::new(),
        conflict_policy: CONFLICT_POLICY_ASK.into(),
    }
}
fn save_endpoint(root: &std::path::Path, e: crate::modules::sync::config::SyncEndpointConfig) {
    std::fs::create_dir_all(root.join("manager-state")).unwrap();
    crate::modules::sync::config::SyncConfigStore::new(root)
        .save_config(&crate::modules::sync::config::SyncConfig {
            schema_version: 1,
            endpoints: vec![e],
        })
        .unwrap();
}
struct EndpointVault {
    root: std::path::PathBuf,
    values: Mutex<BTreeMap<String, String>>,
    reads: Mutex<Vec<String>>,
}
impl EndpointVault {
    fn new(root: &std::path::Path) -> Self {
        Self {
            root: root.into(),
            values: Mutex::new(BTreeMap::from([
                (CRED.into(), "synthetic-original-password".into()),
                (CRED2.into(), "synthetic-second-password".into()),
            ])),
            reads: Mutex::new(vec![]),
        }
    }
}
impl EndpointCredentialsV2 for EndpointVault {
    fn read_password(
        &self,
        root: &std::path::Path,
        endpoint: &crate::modules::sync::config::SyncEndpointConfig,
    ) -> ModelResult<Zeroizing<String>> {
        assert_eq!(root, self.root);
        assert_ne!(endpoint.endpoint_id, endpoint.credential_ref.ref_id);
        self.reads
            .lock()
            .unwrap()
            .push(endpoint.credential_ref.ref_id.clone());
        self.values
            .lock()
            .unwrap()
            .get(&endpoint.credential_ref.ref_id)
            .map(|s| Zeroizing::new(s.clone()))
            .ok_or_else(|| {
                super::super::schema::invalid("capture_credential_missing", "hidden root/secret")
            })
    }
}
#[test]
fn original_endpoint_binding_does_not_follow_current_active_connection_and_rejects_changed_password(
) {
    let root = tempfile::tempdir().unwrap();
    let vault = Arc::new(EndpointVault::new(root.path()));
    save_endpoint(
        root.path(),
        endpoint(A, CRED, "https://fixture.example.test/base"),
    );
    let resolver = EndpointResolverV2::new(root.path().into(), vault.clone()).unwrap();
    let original = resolver.resolve_active().unwrap();
    let handle = original.handle().clone();
    save_endpoint(
        root.path(),
        endpoint(B, CRED2, "https://other.example.test/base"),
    );
    let active = resolver.resolve_active().unwrap();
    assert_ne!(active.handle(), &handle);
    let before = vault.reads.lock().unwrap().len();
    let restored = resolver.resolve_original(&handle).unwrap();
    assert_eq!(restored.handle(), &handle);
    assert_eq!(&vault.reads.lock().unwrap()[before..], &[CRED.to_string()]);
    vault
        .values
        .lock()
        .unwrap()
        .insert(CRED.into(), "changed-original-password".into());
    assert_eq!(
        resolver.resolve_original(&handle).err().unwrap().code,
        ErrorCode::SyncEndpointChanged
    );
    let public = serde_json::to_string(
        &json!({"id":handle.id,"bindingId":handle.binding_id,"revision":handle.revision}),
    )
    .unwrap();
    assert!(!public.contains("password"));
    assert!(!public.contains("fixture.example.test"));
}
#[test]
fn endpoint_revision_edit_rejects_before_reading_replacement_reference_and_never_guesses_on_restart(
) {
    let root = tempfile::tempdir().unwrap();
    let vault = Arc::new(EndpointVault::new(root.path()));
    let saved = endpoint(A, CRED, "https://fixture.example.test");
    save_endpoint(root.path(), saved.clone());
    let resolver = EndpointResolverV2::new(root.path().into(), vault.clone()).unwrap();
    let handle = resolver.resolve_active().unwrap().handle().clone();
    let count = vault.reads.lock().unwrap().len();
    save_endpoint(
        root.path(),
        endpoint(A, CRED2, "https://fixture.example.test"),
    );
    assert_eq!(
        resolver.resolve_original(&handle).err().unwrap().code,
        ErrorCode::SyncEndpointChanged
    );
    assert_eq!(vault.reads.lock().unwrap().len(), count);
    let reopened = EndpointResolverV2::new(root.path().into(), vault.clone()).unwrap();
    assert_eq!(
        reopened.resolve_original(&handle).err().unwrap().code,
        ErrorCode::SyncEndpointChanged
    );
    save_endpoint(
        root.path(),
        endpoint(B, CRED2, "https://other.example.test"),
    );
    reopened.restore_original(saved, &handle).unwrap();
    assert_eq!(
        reopened.resolve_original(&handle).unwrap().handle(),
        &handle
    );
}
#[test]
fn endpoint_canonical_binding_normalizes_equivalent_paths_and_rejects_traversal_before_url_normalization(
) {
    let e = endpoint(A, CRED, "https://FIXTURE.example.test:443/base/");
    let n = normalize_endpoint(&e).unwrap();
    let first = endpoint_handle(&e, &n, "synthetic").unwrap();
    let mut other = e.clone();
    other.url = "https://fixture.example.test/base".into();
    other.remote_path = "fixture/scope".into();
    let next = endpoint_handle(&other, &normalize_endpoint(&other).unwrap(), "synthetic").unwrap();
    assert_eq!(first, next);
    other.url = "https://fixture.example.test/base/../hidden".into();
    assert!(normalize_endpoint(&other).is_err());
    other.url = "https://fixture.example.test".into();
    other.remote_path = "fixture/%2fhidden".into();
    assert!(normalize_endpoint(&other).is_err());
}
#[derive(Default)]
struct MemoryRemote {
    files: Mutex<BTreeMap<String, Vec<u8>>>,
    calls: AtomicUsize,
    writes: AtomicUsize,
}
#[async_trait::async_trait]
impl ConfigSyncTransport for MemoryRemote {
    async fn list_children(
        &self,
        _: &ConnectionHandle,
        _: ListLimits,
    ) -> std::result::Result<ChildListing, SyncTransportError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(ChildListing {
            entries: self
                .files
                .lock()
                .unwrap()
                .iter()
                .map(|(name, b)| ChildEntry {
                    name: name.clone(),
                    is_collection: false,
                    content_length: Some(b.len() as u64),
                })
                .collect(),
            http_status: 207,
            response_bytes: 100,
        })
    }
    async fn get_file(
        &self,
        _: &ConnectionHandle,
        p: &SyncRelativePath,
        _: ReadLimits,
    ) -> std::result::Result<FileRead, SyncTransportError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.files
            .lock()
            .unwrap()
            .get(p.as_str())
            .cloned()
            .map(|body| FileRead {
                body,
                http_status: 200,
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
        body: &[u8],
        _: WriteLimits,
    ) -> std::result::Result<PutResponse, SyncTransportError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.writes.fetch_add(1, Ordering::SeqCst);
        self.files
            .lock()
            .unwrap()
            .insert(p.as_str().into(), body.to_vec());
        Ok(PutResponse { http_status: 201 })
    }
}
struct SourceOnlyLocal(CurrentSourceProviderV2);
impl LocalApplication for SourceOnlyLocal {
    fn root_revision(&self) -> SyncResult<Digest> {
        self.0.root_revision()
    }
    fn current_source(&self) -> SyncResult<SourceGuard> {
        self.0.current_source()
    }
    fn target_digest(&self, _: &str) -> SyncResult<Digest> {
        Ok(digest(b"synthetic-current-local-target"))
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
        panic!("not a local apply test")
    }
    fn export_once(&self, _: &str, _: &[u8], _: &PayloadSchemaRef, _: &str) -> SyncResult<Uuid> {
        panic!("not an export test")
    }
}
fn empty_observation(handle: &ConnectionHandle) -> RemoteObservation {
    RemoteObservation {
        dto_version: 1,
        binding_id: handle.binding_id.clone(),
        state: RemoteState::Empty,
        generation: None,
        generations: vec![],
        heads: vec![],
        orphan_count: 0,
        stale_count: 0,
        view_digest: digest(b"placeholder superseded by actual graph observation"),
        observed_at: TIME.into(),
        problems: vec![],
        recovery_candidates: vec![],
        recovery_actions: vec![],
    }
}
#[tokio::test]
async fn reusable_session_prepares_and_publishes_real_core_then_original_verify_does_not_capture() {
    let root = tempfile::tempdir().unwrap();
    let tx = tempfile::tempdir().unwrap();
    let store = super::super::sync_v2::store::Store::open(tx.path()).unwrap();
    let vault = Arc::new(EndpointVault::new(root.path()));
    save_endpoint(
        root.path(),
        endpoint(A, CRED, "https://fixture.example.test"),
    );
    let resolver = EndpointResolverV2::new(root.path().into(), vault).unwrap();
    let bound = resolver.resolve_active().unwrap();
    let provider = Arc::new(Capture::new(objects(8192, false)));
    let preview = upload_preview(provider.as_ref(), None);
    let codec = HostPayloadCodecV2::new(provider.clone(), preview, SessionSecretsV2::default());
    let local = SourceOnlyLocal(codec.current_source_provider().unwrap());
    let remote = MemoryRemote::default();
    let observation = super::super::sync_v2::SyncContext::new(
        &store,
        &remote,
        &codec,
        &local,
        bound.handle().clone(),
    )
    .unwrap()
    .observe_remote(ObserveRequest {
        connection_id: bound.handle().id.clone(),
    })
    .await
    .unwrap();
    let session = HostSessionV2::new(codec, &bound, observation).unwrap();
    let HostPreparationV2::Upload(plan) = session
        .prepare(&resolver, &store, &remote, &local)
        .await
        .unwrap()
    else {
        panic!("upload expected")
    };
    assert_eq!(remote.writes.load(Ordering::SeqCst), 0);
    let original = session
        .publish_upload(
            ConfirmUploadRequest {
                operation_id: plan.operation_id.clone(),
                plan_digest: plan.plan_digest.clone(),
            },
            &resolver,
            &store,
            &remote,
            &local,
        )
        .await
        .unwrap();
    assert_eq!(original.storage_status, StorageStatus::Stored);
    let before = provider.calls.load(Ordering::SeqCst);
    let puts = remote.writes.load(Ordering::SeqCst);
    *provider.error.lock().unwrap() = Some("capture_stale");
    let repeated = session
        .publish_upload(
            ConfirmUploadRequest {
                operation_id: plan.operation_id.clone(),
                plan_digest: plan.plan_digest.clone(),
            },
            &resolver,
            &store,
            &remote,
            &local,
        )
        .await
        .unwrap();
    assert_eq!(repeated.storage_status, StorageStatus::Stored);
    let resumed = session
        .resume_original_upload(
            ResumeUploadRequest {
                operation_id: plan.operation_id.clone(),
                plan_digest: plan.plan_digest.clone(),
                expected_view_digest: digest(b"ignored for already stored verification"),
            },
            &resolver,
            &store,
            &remote,
            &local,
        )
        .await
        .unwrap();
    assert_eq!(resumed.storage_status, StorageStatus::Stored);
    let checked = session
        .verify_original_upload(
            OperationRequest {
                operation_id: plan.operation_id,
            },
            &resolver,
            &store,
            &remote,
            &local,
        )
        .await
        .unwrap();
    assert_eq!(checked.storage_status, StorageStatus::Stored);
    assert_eq!(before, provider.calls.load(Ordering::SeqCst));
    assert_eq!(puts, remote.writes.load(Ordering::SeqCst));
}
#[tokio::test]
async fn no_change_session_never_calls_core_prepare_or_remote_and_bad_observation_cannot_be_empty()
{
    let root = tempfile::tempdir().unwrap();
    let tx = tempfile::tempdir().unwrap();
    let store = super::super::sync_v2::store::Store::open(tx.path()).unwrap();
    let vault = Arc::new(EndpointVault::new(root.path()));
    save_endpoint(
        root.path(),
        endpoint(A, CRED, "https://fixture.example.test"),
    );
    let resolver = EndpointResolverV2::new(root.path().into(), vault).unwrap();
    let bound = resolver.resolve_active().unwrap();
    let provider = Arc::new(Capture::new(objects(4096, false)));
    let cloud = baseline(objects(4096, false), None);
    let preview = preview_semantics(
        provider.as_ref(),
        selection(),
        CloudMaterialV2::Valid(&cloud),
        &SessionSecretsV2::default(),
        None,
    )
    .unwrap();
    let codec = HostPayloadCodecV2::new(provider, preview, SessionSecretsV2::default());
    let local = SourceOnlyLocal(codec.current_source_provider().unwrap());
    let remote = MemoryRemote::default();
    let mut observation = empty_observation(bound.handle());
    observation.state = RemoteState::Ready;
    observation.heads = vec![cloud.head.clone()];
    let generation = GenerationRef {
        generation_id: O.into(),
        digest: digest(b"synthetic verified generation"),
    };
    observation.generation = Some(generation.clone());
    observation.generations = vec![generation];
    let session = HostSessionV2::new(codec, &bound, observation).unwrap();
    assert!(matches!(
        session
            .prepare(&resolver, &store, &remote, &local)
            .await
            .unwrap(),
        HostPreparationV2::NoChange(_)
    ));
    assert_eq!(remote.calls.load(Ordering::SeqCst), 0);
    assert_eq!(remote.writes.load(Ordering::SeqCst), 0);
    assert_eq!(std::fs::read_dir(tx.path()).unwrap().count(), 0);
    let preview = upload_preview(session.codec.provider.as_ref(), None);
    let codec = HostPayloadCodecV2::new(
        session.codec.provider.clone(),
        preview,
        SessionSecretsV2::default(),
    );
    let mut invalid = empty_observation(bound.handle());
    invalid.state = RemoteState::Incomplete;
    assert!(HostSessionV2::new(codec, &bound, invalid).is_err());
}
fn actual_capture_fixture(
    include_key: bool,
) -> (
    tempfile::TempDir,
    std::path::PathBuf,
    std::path::PathBuf,
    super::super::saved_objects_capture_v2::CaptureRequestV2,
) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("D");
    let home = temp.path().join("H");
    std::fs::create_dir_all(root.join("manager-state")).unwrap();
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(home.join("config.json"),serde_json::to_vec(&json!({"providers":{"fixture":{"adapter":"openai-chat","baseUrl":"https://fixture.invalid/v1","authMode":"key","apiKey":"synthetic-actual-saved-inline-key","models":["fixture-model"],"selectedModels":["fixture-model"],"responsesPath":"/fixture/responses","modelContextWindows":{"fixture-model":8192}}},"defaultProvider":"fixture","defaultModel":"fixture/fixture-model"})).unwrap()).unwrap();
    let snapshot = crate::modules::model_config::config_file::v2::snapshot(&root, &home).unwrap();
    // Setup uses the actual public Read/binding owner to establish a stable
    // source UUID. Capture later reads the saved raw file/registry, not this
    // editable draft (the test deliberately corrupts that draft afterwards).
    let observed = crate::modules::model_config::config_file::v2::begin_draft(
        &root,
        &home,
        crate::modules::model_config::config_file::v2::ReadRequestV2 {
            expected_source: snapshot.source_digest,
            expected_library: snapshot.library_digest,
            expected_draft: snapshot.draft.as_ref().map(|d| d.base.clone()),
        },
    )
    .unwrap();
    let id = observed.draft.as_ref().unwrap().policy.channels[0]
        .id
        .clone();
    let request = super::super::saved_objects_capture_v2::CaptureRequestV2 {
        selection: SelectionV2 {
            channels: vec![objects_overlay_v2::ChannelSelectionV2 {
                portable_channel_id: id,
                model_ids: None,
                family_ids: None,
                include_connection: true,
                include_key,
                include_authorization: false,
            }],
            ..Default::default()
        },
        mcp_clients: Default::default(),
        assets_reviewed: false,
    };
    (temp, root, home, request)
}
#[test]
fn production_capture_provider_calls_registered_saved_owner_and_never_uses_dirty_draft() {
    let (_temp, root, home, request) = actual_capture_fixture(false);
    let selection = request.selection.clone();
    let config_before = std::fs::read(home.join("config.json")).unwrap();
    // Deliberately invalid draft proves the production path is saved-only, not
    // a UI draft parser or a copied reimplementation of the capture engine.
    std::fs::write(
        root.join(crate::modules::model_config::store_v2::DRAFT_PATH),
        b"{broken unsaved draft",
    )
    .unwrap();
    let provider = SavedCaptureProviderV2::new(
        root.clone(),
        home.clone(),
        request,
        Arc::new(super::super::saved_objects_capture_v2::SystemCaptureCredentialsV2),
    )
    .unwrap();
    let a = provider.capture_saved(&selection).unwrap();
    let b = provider.capture_saved(&selection).unwrap();
    assert_eq!(a.source_guard, b.source_guard);
    assert!(a.objects.partitions().channel_keys.is_empty());
    assert_eq!(a.objects.models().format_version, 6);
    let public = a.objects.models().payload.policy.as_ref().unwrap().channels[0]
        .connection
        .as_ref()
        .unwrap()
        .provider
        .as_ref()
        .unwrap();
    assert_eq!(public["responsesPath"], "/fixture/responses");
    assert!(public.get("apiKey").is_none());
    assert_eq!(
        std::fs::read(home.join("config.json")).unwrap(),
        config_before
    );
    let mut other = selection;
    other.channels.clear();
    assert!(provider.capture_saved(&other).is_err());
}
#[test]
fn production_codec_recaptures_actual_inline_saved_key_then_inspects_frozen_bytes_without_source_file(
) {
    let (_temp, root, home, request) = actual_capture_fixture(true);
    let selection = request.selection.clone();
    let provider = Arc::new(
        SavedCaptureProviderV2::new(
            root,
            home.clone(),
            request,
            Arc::new(super::super::saved_objects_capture_v2::SystemCaptureCredentialsV2),
        )
        .unwrap(),
    );
    let preview = preview_semantics(
        provider.as_ref(),
        selection,
        CloudMaterialV2::Empty,
        &SessionSecretsV2::default(),
        None,
    )
    .unwrap();
    let guard = preview.source_guard.clone();
    let codec = HostPayloadCodecV2::new(provider, preview, SessionSecretsV2::default());
    let frozen = codec.freeze_selected(&guard, &[], None).unwrap();
    let decoded = file_objects_v2::decode(
        &frozen.bytes,
        None,
        &frozen.schema,
        Some(&PayloadAuthorizationV2),
    )
    .unwrap();
    let id = &decoded
        .objects
        .models()
        .payload
        .policy
        .as_ref()
        .unwrap()
        .channels[0]
        .id;
    assert_eq!(
        decoded.objects.partitions().channel_keys.get(id),
        Some("synthetic-actual-saved-inline-key")
    );
    std::fs::remove_file(home.join("config.json")).unwrap();
    codec
        .inspect_and_validate(&frozen.bytes, &frozen.schema)
        .unwrap();
}
#[cfg(not(windows))]
#[test]
fn production_endpoint_credentials_read_exact_ref_in_injected_root_without_global_configure() {
    use crate::infrastructure::keychain;
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    let saved = endpoint(A, CRED, "https://fixture.example.test");
    for (root, value) in [
        (a.path(), "root-a-synthetic-password"),
        (b.path(), "root-b-synthetic-password"),
    ] {
        save_endpoint(root, saved.clone());
        let account = keychain::account_key(keychain::WEBDAV_CREDENTIAL_PURPOSE, CRED).unwrap();
        let directory = root.join("manager-state/credentials");
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join(format!("{account}.json")),
            serde_json::to_vec(&json!({"account":account,"value":value,"protection":"plain"}))
                .unwrap(),
        )
        .unwrap();
        // An endpoint-id-named decoy cannot become the credential reference.
        let wrong =
            keychain::account_key(keychain::WEBDAV_CREDENTIAL_PURPOSE, &format!("cred_{A}"))
                .unwrap();
        std::fs::write(
            directory.join(format!("{wrong}.json")),
            serde_json::to_vec(
                &json!({"account":wrong,"value":"wrong-reference-secret","protection":"plain"}),
            )
            .unwrap(),
        )
        .unwrap();
    }
    let owner = SystemEndpointCredentialsV2;
    assert_eq!(
        owner.read_password(a.path(), &saved).unwrap().as_str(),
        "root-a-synthetic-password"
    );
    assert_eq!(
        owner.read_password(b.path(), &saved).unwrap().as_str(),
        "root-b-synthetic-password"
    );
    let resolver =
        EndpointResolverV2::new(a.path().into(), Arc::new(SystemEndpointCredentialsV2)).unwrap();
    resolver.resolve_active().unwrap();
    let missing = tempfile::tempdir().unwrap();
    assert_eq!(
        map_host_error(owner.read_password(missing.path(), &saved).err().unwrap()).code,
        ErrorCode::SyncAuthRequired
    );
}
#[test]
fn newly_frozen_protected_payload_uses_write_passphrase_for_core_inspection_without_recapture() {
    let provider = Arc::new(Capture::new(objects(8192, false)));
    let preview = upload_preview(provider.as_ref(), None);
    let guard = preview.source_guard.clone();
    let codec = HostPayloadCodecV2::new(
        provider.clone(),
        preview,
        SessionSecretsV2 {
            read_passphrase: None,
            write_passphrase: Some(Zeroizing::new("synthetic-new-protection".into())),
        },
    );
    let frozen = codec.freeze_selected(&guard, &[], None).unwrap();
    assert_eq!(frozen.protection_mode, ProtectionMode::Protected);
    let count = provider.calls.load(Ordering::SeqCst);
    *provider.error.lock().unwrap() = Some("capture_stale");
    codec
        .inspect_and_validate(&frozen.bytes, &frozen.schema)
        .unwrap();
    assert_eq!(count, provider.calls.load(Ordering::SeqCst));
    assert!(file_objects_v2::decode(&frozen.bytes, None, &frozen.schema, None).is_err());
    file_objects_v2::decode(
        &frozen.bytes,
        Some("synthetic-new-protection"),
        &frozen.schema,
        None,
    )
    .unwrap();
}
#[tokio::test]
async fn original_upload_owner_reopens_existing_journal_without_preview_or_any_current_capture() {
    struct NoCurrentLocal;
    impl LocalApplication for NoCurrentLocal {
        fn root_revision(&self) -> SyncResult<Digest> {
            panic!("original verify must not inspect current root")
        }
        fn current_source(&self) -> SyncResult<SourceGuard> {
            panic!("original verify must not capture source")
        }
        fn target_digest(&self, _: &str) -> SyncResult<Digest> {
            panic!("no local write")
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
            panic!("no local write")
        }
        fn export_once(
            &self,
            _: &str,
            _: &[u8],
            _: &PayloadSchemaRef,
            _: &str,
        ) -> SyncResult<Uuid> {
            panic!("no export")
        }
    }
    let root = tempfile::tempdir().unwrap();
    let tx = tempfile::tempdir().unwrap();
    let store = super::super::sync_v2::store::Store::open(tx.path()).unwrap();
    let vault = Arc::new(EndpointVault::new(root.path()));
    let saved = endpoint(A, CRED, "https://fixture.example.test");
    save_endpoint(root.path(), saved.clone());
    let resolver = EndpointResolverV2::new(root.path().into(), vault.clone()).unwrap();
    let bound = resolver.resolve_active().unwrap();
    let provider = Arc::new(Capture::new(objects(8192, false)));
    let preview = upload_preview(provider.as_ref(), None);
    let codec = HostPayloadCodecV2::new(provider.clone(), preview, SessionSecretsV2::default());
    let local = SourceOnlyLocal(codec.current_source_provider().unwrap());
    let remote = MemoryRemote::default();
    let observation = super::super::sync_v2::SyncContext::new(
        &store,
        &remote,
        &codec,
        &local,
        bound.handle().clone(),
    )
    .unwrap()
    .observe_remote(ObserveRequest {
        connection_id: bound.handle().id.clone(),
    })
    .await
    .unwrap();
    let session = HostSessionV2::new(codec, &bound, observation).unwrap();
    let HostPreparationV2::Upload(plan) = session
        .prepare(&resolver, &store, &remote, &local)
        .await
        .unwrap()
    else {
        panic!("upload")
    };
    session
        .publish_upload(
            ConfirmUploadRequest {
                operation_id: plan.operation_id.clone(),
                plan_digest: plan.plan_digest,
            },
            &resolver,
            &store,
            &remote,
            &local,
        )
        .await
        .unwrap();
    drop(session);
    *provider.error.lock().unwrap() = Some("capture_stale");
    let captures = provider.calls.load(Ordering::SeqCst);
    save_endpoint(
        root.path(),
        endpoint(B, CRED2, "https://other.example.test"),
    );
    let reopened = EndpointResolverV2::new(root.path().into(), vault).unwrap();
    let restored = reopened.restore_original(saved, bound.handle()).unwrap();
    let original = OriginalUploadOwnerV2::new(&restored, None);
    let store = super::super::sync_v2::store::Store::open(tx.path()).unwrap();
    let puts = remote.writes.load(Ordering::SeqCst);
    let read_before = remote.calls.load(Ordering::SeqCst);
    original
        .read_result(
            &store,
            OperationRequest {
                operation_id: plan.operation_id.clone(),
            },
        )
        .unwrap();
    assert_eq!(read_before, remote.calls.load(Ordering::SeqCst));
    let fact = original
        .verify(
            OperationRequest {
                operation_id: plan.operation_id,
            },
            &reopened,
            &store,
            &remote,
            &NoCurrentLocal,
        )
        .await
        .unwrap();
    assert_eq!(fact.storage_status, StorageStatus::Stored);
    assert_eq!(captures, provider.calls.load(Ordering::SeqCst));
    assert_eq!(puts, remote.writes.load(Ordering::SeqCst));
}
struct ChannelMapping {
    revision: Digest,
    valid: std::sync::atomic::AtomicBool,
    checks: AtomicUsize,
}
impl TrustedMappingOwnerV2 for ChannelMapping {
    fn verify(&self, binding: &LocalTargetBindingV2) -> ModelResult<()> {
        self.checks.fetch_add(1, Ordering::SeqCst);
        if !self.valid.load(Ordering::SeqCst) || binding.mapping_revision != self.revision {
            return Err(super::super::schema::invalid(
                "host_mapping_changed",
                "target absent/present fact changed",
            ));
        }
        Ok(())
    }
    fn project_saved_to_remote(
        &self,
        objects: SavedObjects,
        binding: &LocalTargetBindingV2,
    ) -> ModelResult<SavedObjects> {
        self.verify(binding)?;
        let mut parts = objects.into_captured();
        if let Some(policy) = &mut parts.models.payload.policy {
            for ch in &mut policy.channels {
                let slot = binding
                    .slots
                    .iter()
                    .find(|s| s.local_id == ch.id)
                    .ok_or_else(|| {
                        super::super::schema::invalid(
                            "host_mapping_required",
                            "no exact channel pair",
                        )
                    })?;
                ch.id = slot.portable_id.clone();
            }
            for id in &mut policy.channel_order {
                if let Some(pair) = binding.slots.iter().find(|s| s.local_id == *id) {
                    *id = pair.portable_id.clone();
                }
            }
            if let Some(id) = &mut policy.default_channel_id {
                if let Some(pair) = binding.slots.iter().find(|s| s.local_id == *id) {
                    *id = pair.portable_id.clone();
                }
            }
        }
        parts.models.digest =
            super::super::schema_v2::digest(&canonical_bytes(&parts.models.payload).unwrap());
        SavedObjects::capture(parts, None)
    }
}
struct MappedApplyLocal {
    source: CurrentSourceProviderV2,
    binding: LocalTargetBindingV2,
    owner: Arc<ChannelMapping>,
    applied: std::sync::atomic::AtomicBool,
}
impl LocalApplication for MappedApplyLocal {
    fn root_revision(&self) -> SyncResult<Digest> {
        self.source.root_revision()
    }
    fn current_source(&self) -> SyncResult<SourceGuard> {
        self.source.current_source()
    }
    fn target_digest(&self, _: &str) -> SyncResult<Digest> {
        self.owner.verify(&self.binding).map_err(map_host_error)?;
        Ok(self.binding.digest.clone())
    }
    fn apply_once(
        &self,
        _: &str,
        bytes: &[u8],
        schema: &PayloadSchemaRef,
        _: &str,
        expected: &str,
        backup: bool,
    ) -> SyncResult<()> {
        self.owner.verify(&self.binding).map_err(map_host_error)?;
        assert!(backup);
        assert_eq!(expected, self.binding.digest);
        let incoming =
            file_objects_v2::decode(bytes, None, schema, None).map_err(map_host_error)?;
        assert_eq!(
            incoming
                .objects
                .models()
                .payload
                .policy
                .as_ref()
                .unwrap()
                .channels[0]
                .id,
            self.binding.slots[0].portable_id
        );
        assert_eq!(
            self.binding.slots[0].portable_id,
            self.binding.slots[0].local_id
        );
        self.applied.store(true, Ordering::SeqCst);
        Ok(())
    }
    fn export_once(&self, _: &str, _: &[u8], _: &PayloadSchemaRef, _: &str) -> SyncResult<Uuid> {
        panic!("not exporting")
    }
}
fn seed_cloud(remote: &MemoryRemote, input: &PayloadInput) -> GenerationRef {
    let g = Generation {
        format: "OCXD_CONFIG_GENERATION".into(),
        format_version: 1,
        generation_id: B.into(),
    };
    let gb = super::super::sync_v2::store::canonical(&g).unwrap();
    let gr = GenerationRef {
        generation_id: B.into(),
        digest: digest(&gb),
    };
    let c = Commit {
        format: "OCXD_CONFIG_COMMIT".into(),
        format_version: 1,
        generation_id: B.into(),
        operation_id: input.head.operation_id.clone(),
        parents: vec![],
        payload_digest: digest(&input.bytes),
        payload_bytes: input.bytes.len() as u64,
        payload_schema: file_objects_v2::supported_schema(),
        intent: Intent::Update,
        resolution: None,
    };
    let cb = super::super::sync_v2::store::canonical(&c).unwrap();
    let mut files = remote.files.lock().unwrap();
    files.insert(SyncRelativePath::generation(B).unwrap().as_str().into(), gb);
    files.insert(
        SyncRelativePath::data(B, &input.head.operation_id)
            .unwrap()
            .as_str()
            .into(),
        input.bytes.clone(),
    );
    files.insert(
        SyncRelativePath::commit(B, &input.head.operation_id)
            .unwrap()
            .as_str()
            .into(),
        cb,
    );
    gr
}
#[tokio::test]
async fn first_install_remote_channel_local_explicit_empty_defaults_download_and_applies_through_mapping(
) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("D");
    let home = temp.path().join("H");
    let tx = temp.path().join("T");
    std::fs::create_dir_all(root.join("manager-state")).unwrap();
    std::fs::create_dir_all(&home).unwrap();
    let remote_scope = selection();
    let local_scope = SelectionV2::default();
    let local_request = super::super::saved_objects_capture_v2::CaptureRequestV2 {
        selection: local_scope.clone(),
        mcp_clients: Default::default(),
        assets_reviewed: false,
    };
    let actual = Arc::new(
        SavedCaptureProviderV2::new(
            root.clone(),
            home,
            local_request,
            Arc::new(super::super::saved_objects_capture_v2::SystemCaptureCredentialsV2),
        )
        .unwrap(),
    );
    let mapping = Arc::new(ChannelMapping {
        revision: digest(b"mapping-revision"),
        valid: std::sync::atomic::AtomicBool::new(true),
        checks: AtomicUsize::new(0),
    });
    let binding = LocalTargetBindingV2::freeze(
        mapping.revision.clone(),
        local_scope,
        remote_scope.clone(),
        DirectionV2::Download,
        vec![TargetSlotV2 {
            kind: "channel".into(),
            portable_id: A.into(),
            local_id: A.into(),
            expected: TargetExpectationV2::ExpectedAbsent(digest(
                b"exact locked absent channel slot",
            )),
        }],
    )
    .unwrap();
    let mapped =
        Arc::new(MappedCaptureProviderV2::new(actual, mapping.clone(), binding.clone()).unwrap());
    let remote = MemoryRemote::default();
    let mut cloud = baseline(objects(4096, false), None);
    seed_cloud(&remote, &cloud);
    let commit_bytes = remote
        .files
        .lock()
        .unwrap()
        .get(SyncRelativePath::commit(B, O).unwrap().as_str())
        .unwrap()
        .clone();
    cloud.head.digest = digest(&commit_bytes);
    let preview = preview_semantics(
        mapped.as_ref(),
        remote_scope,
        CloudMaterialV2::Valid(&cloud),
        &SessionSecretsV2::default(),
        None,
    )
    .unwrap();
    assert_eq!(preview.decision.chosen_direction, DirectionV2::Download);
    assert_eq!(
        preview.decision.automatic_reason,
        AutomaticReasonV2::LocalEmpty
    );
    assert!(preview.decision.can_download);
    assert!(!preview.decision.can_upload);
    assert!(!preview.decision.no_change);
    assert_eq!(preview.target_binding().unwrap().digest, binding.digest);
    save_endpoint(&root, endpoint(A, CRED, "https://fixture.example.test"));
    let resolver =
        EndpointResolverV2::new(root.clone(), Arc::new(EndpointVault::new(&root))).unwrap();
    let bound = resolver.resolve_active().unwrap();
    let store = super::super::sync_v2::store::Store::open(&tx).unwrap();
    let codec = HostPayloadCodecV2::new(mapped, preview, SessionSecretsV2::default());
    let local = MappedApplyLocal {
        source: codec.current_source_provider().unwrap(),
        binding,
        owner: mapping,
        applied: std::sync::atomic::AtomicBool::new(false),
    };
    let observation = super::super::sync_v2::SyncContext::new(
        &store,
        &remote,
        &codec,
        &local,
        bound.handle().clone(),
    )
    .unwrap()
    .observe_remote(ObserveRequest {
        connection_id: bound.handle().id.clone(),
    })
    .await
    .unwrap();
    let session = HostSessionV2::new(codec, &bound, observation).unwrap();
    let HostPreparationV2::Download(plan) = session
        .prepare(&resolver, &store, &remote, &local)
        .await
        .unwrap()
    else {
        panic!("download must not be disabled for absent local channel")
    };
    assert_eq!(remote.writes.load(Ordering::SeqCst), 0);
    assert!(!local.applied.load(Ordering::SeqCst));
    let result = session
        .apply_download(
            ConfirmDownloadRequest {
                operation_id: plan.operation_id.clone(),
                plan_digest: plan.plan_digest.clone(),
            },
            &resolver,
            &store,
            &remote,
            &local,
        )
        .await
        .unwrap();
    assert_eq!(result.local_apply_status, LocalApplyStatus::Applied);
    assert!(local.applied.load(Ordering::SeqCst));
    assert_eq!(remote.writes.load(Ordering::SeqCst), 0);
    local.owner.valid.store(false, Ordering::SeqCst);
    struct AppliedReceiptLocal(Digest);
    impl LocalApplication for AppliedReceiptLocal {
        fn root_revision(&self) -> SyncResult<Digest> {
            Ok(self.0.clone())
        }
        fn current_source(&self) -> SyncResult<SourceGuard> {
            panic!("no recapture after applied")
        }
        fn target_digest(&self, _: &str) -> SyncResult<Digest> {
            panic!("no old absent check")
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
            panic!("no reapply")
        }
        fn export_once(
            &self,
            _: &str,
            _: &[u8],
            _: &PayloadSchemaRef,
            _: &str,
        ) -> SyncResult<Uuid> {
            panic!("no export")
        }
    }
    let applied = AppliedReceiptLocal(store.receive(&plan.operation_id).unwrap().root_revision);
    let receipt = session
        .apply_download(
            ConfirmDownloadRequest {
                operation_id: plan.operation_id,
                plan_digest: plan.plan_digest,
            },
            &resolver,
            &store,
            &remote,
            &applied,
        )
        .await
        .unwrap();
    assert_eq!(receipt.local_apply_status, LocalApplyStatus::Applied);
}
#[test]
fn expected_absent_mapping_unknown_or_changed_target_does_not_turn_into_empty_capture() {
    let capture = Arc::new(Capture::new(objects(8192, false)));
    let owner = Arc::new(ChannelMapping {
        revision: digest(b"map"),
        valid: std::sync::atomic::AtomicBool::new(true),
        checks: AtomicUsize::new(0),
    });
    let binding = LocalTargetBindingV2::freeze(
        owner.revision.clone(),
        selection(),
        selection(),
        DirectionV2::Upload,
        vec![TargetSlotV2 {
            kind: "channel".into(),
            portable_id: A.into(),
            local_id: A.into(),
            expected: TargetExpectationV2::Present(digest(b"present atom")),
        }],
    )
    .unwrap();
    let mapped = MappedCaptureProviderV2::new(capture.clone(), owner.clone(), binding).unwrap();
    owner.valid.store(false, Ordering::SeqCst);
    let before = capture.calls.load(Ordering::SeqCst);
    assert_eq!(
        preview_semantics(
            &mapped,
            selection(),
            CloudMaterialV2::Empty,
            &SessionSecretsV2::default(),
            None
        )
        .err()
        .unwrap()
        .code,
        ErrorCode::SyncScopeChanged
    );
    assert_eq!(before, capture.calls.load(Ordering::SeqCst));
    let invalid = LocalTargetBindingV2::freeze(
        digest(b"map"),
        SelectionV2::default(),
        selection(),
        DirectionV2::Download,
        vec![TargetSlotV2 {
            kind: "channel".into(),
            portable_id: A.into(),
            local_id: A.into(),
            expected: TargetExpectationV2::Present(digest(b"not actually selected local present")),
        }],
    );
    assert!(invalid.is_err());
}

#[cfg(not(windows))]
#[test]
fn actual_default_endpoint_id_resolves_d_bound_password_and_binds_production_transport() {
    use crate::infrastructure::{keychain, webdav_config_v2::FileWebDavTransport};
    let root = tempfile::tempdir().unwrap();
    // This is the exact current SyncConfigStore::save_endpoint ID/domain,
    // with its real persisted config type and D-bound credential file format.
    let original = endpoint("default", CRED, "https://fixture.example.test/base");
    save_endpoint(root.path(), original.clone());
    let reread = crate::modules::sync::config::SyncConfigStore::new(root.path())
        .load()
        .unwrap();
    assert_eq!(reread.active().unwrap().endpoint_id, "default");
    let account = keychain::account_key(keychain::WEBDAV_CREDENTIAL_PURPOSE, CRED).unwrap();
    let dir = root.path().join("manager-state/credentials");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(format!("{account}.json")),serde_json::to_vec(&json!({"account":account,"value":"default-endpoint-synthetic-password","protection":"plain"})).unwrap()).unwrap();
    let resolver =
        EndpointResolverV2::new(root.path().into(), Arc::new(SystemEndpointCredentialsV2)).unwrap();
    let bound = resolver.resolve_active().unwrap();
    assert!(bound.handle().id.starts_with("default:"));
    let transport = FileWebDavTransport::new().unwrap();
    bound.bind_file_transport(&transport).unwrap();
    let handle = bound.handle().clone();
    // Switching the active saved endpoint cannot substitute its ID/password for
    // the original default binding. No second password is needed to verify it.
    save_endpoint(
        root.path(),
        endpoint(
            "another-saved-endpoint",
            CRED2,
            "https://other.example.test",
        ),
    );
    let restored = resolver.resolve_original(&handle).unwrap();
    assert_eq!(restored.handle(), &handle);
    restored.bind_file_transport(&transport).unwrap();
    let mut changed = original;
    changed.url = "https://changed.example.test".into();
    save_endpoint(root.path(), changed);
    assert_eq!(
        resolver.resolve_original(&handle).err().unwrap().code,
        ErrorCode::SyncEndpointChanged
    );
}
#[test]
fn endpoint_ids_are_bounded_opaque_strings_not_operation_uuids() {
    for id in [
        "default",
        "saved-endpoint",
        "用户连接",
        "x:y",
        &"x".repeat(256),
    ] {
        let e = endpoint(id, CRED, "https://fixture.example.test");
        let normalized = normalize_endpoint(&e).unwrap();
        let handle = endpoint_handle(&e, &normalized, "synthetic-password").unwrap();
        assert!(handle.id.starts_with(&format!("{id}:")));
    }
    for id in [
        "",
        " ",
        "default\n",
        "default\r",
        "default\t",
        "default\0",
        &"x".repeat(257),
    ] {
        let e = endpoint(id, CRED, "https://fixture.example.test");
        assert_eq!(
            normalize_endpoint(&e).err().unwrap().code,
            ErrorCode::SyncEndpointChanged
        );
    }
    // Only the endpoint domain changed: operation/generation UUID and the
    // credential reference/account-key owner checks remain strict.
    assert!(require_uuid("default").is_err());
    assert!(SyncRelativePath::generation("default").is_err());
    assert!(crate::infrastructure::keychain::account_key(
        crate::infrastructure::keychain::WEBDAV_CREDENTIAL_PURPOSE,
        "default"
    )
    .is_err());
}
#[tokio::test]
async fn download_prepare_freezes_explicit_backup_preference_without_publishing() {
    let root = tempfile::tempdir().unwrap();
    let tx = tempfile::tempdir().unwrap();
    let store = super::super::sync_v2::store::Store::open(tx.path()).unwrap();
    save_endpoint(
        root.path(),
        endpoint("default", CRED, "https://fixture.example.test"),
    );
    let resolver = EndpointResolverV2::new(
        root.path().into(),
        Arc::new(EndpointVault::new(root.path())),
    )
    .unwrap();
    let bound = resolver.resolve_active().unwrap();
    let provider = Arc::new(Capture::new(objects(8192, false)));
    let mut cloud = baseline(objects(4096, false), None);
    let remote = MemoryRemote::default();
    seed_cloud(&remote, &cloud);
    cloud.head.digest = digest(
        remote
            .files
            .lock()
            .unwrap()
            .get(SyncRelativePath::commit(B, O).unwrap().as_str())
            .unwrap(),
    );
    let preview = preview_semantics(
        provider.as_ref(),
        selection(),
        CloudMaterialV2::Valid(&cloud),
        &SessionSecretsV2::default(),
        Some(DirectionV2::Download),
    )
    .unwrap();
    let codec = HostPayloadCodecV2::new(provider, preview, SessionSecretsV2::default());
    let local = SourceOnlyLocal(codec.current_source_provider().unwrap());
    let graph = super::super::sync_v2::SyncContext::new(
        &store,
        &remote,
        &codec,
        &local,
        bound.handle().clone(),
    )
    .unwrap()
    .observe_remote(ObserveRequest {
        connection_id: bound.handle().id.clone(),
    })
    .await
    .unwrap();
    let session = HostSessionV2::new(codec, &bound, graph).unwrap();
    let HostPreparationV2::Download(plan) = session
        .prepare_with_backup(&resolver, &store, &remote, &local, false)
        .await
        .unwrap()
    else {
        panic!("download")
    };
    assert!(!plan.backup_before_update);
    assert_eq!(remote.writes.load(Ordering::SeqCst), 0);
}
#[test]
fn mapping_direction_bridge_reuses_saved_atom_time_without_borrowing_unknown_or_other_domains() {
    let local = stamped(objects(8192, false), "2026-11-06T12:00:00Z");
    let old = stamped(objects(4096, false), "2026-11-06T11:00:00Z");
    assert_eq!(
        automatic_direction_from_saved(&local, Some(&old), &selection()).unwrap(),
        DirectionV2::Upload
    );
    assert_eq!(
        automatic_direction_from_saved(&objects(8192, false), Some(&old), &selection()).unwrap(),
        DirectionV2::Download
    );
    let other = stamped(objects(4096, true), "2026-11-06T11:00:00Z");
    assert_eq!(
        automatic_direction_from_saved(&local, Some(&other), &selection()).unwrap(),
        DirectionV2::Download
    );
    assert_eq!(
        automatic_direction_from_saved(&local, None, &selection()).unwrap(),
        DirectionV2::Upload
    );
}
