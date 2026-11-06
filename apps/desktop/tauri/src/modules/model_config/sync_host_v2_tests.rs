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
