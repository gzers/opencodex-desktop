//! Synthetic sources only, all D/H and logs isolated under /tmp.
use super::super::sync_v2::types::LocalApplication;
use super::super::{
    file_objects_v2::SecretValues,
    objects_overlay_v2::{ChannelSelectionV2, SelectionV2},
    portable_v2::{
        ModelPayloadV2, PortableChannelV2, PortableConnectionV2, PortableModelsV2, PortablePolicyV2,
    },
    schema::{ModelRules, ModelSelection, ServiceNode},
    schema_v2::{ModelConfigV2, SourceKindV2},
    source_v2::source_from_provider_v2,
    sync_local_v2::ModelApplyV2,
};
use super::super::{sync_host_v2 as host, sync_local_v2 as local};
use super::*;
use host::CaptureProviderV2;
use serde_json::Value;
use tempfile::TempDir;
const CLOUD: &str = "11111111-1111-4111-8111-111111111111";
const OTHER: &str = "22222222-2222-4222-8222-222222222222";
const OP: &str = "33333333-3333-4333-8333-333333333333";
const NODE: &str = "44444444-4444-4444-8444-444444444444";
const SECRET: &str = "synthetic-identity-only-secret";
struct Credentials;
impl CaptureCredentialsV2 for Credentials {}
struct Fixture {
    _temp: TempDir,
    root: PathBuf,
    home: PathBuf,
    owner: Arc<SyncIdentityOwnerV2>,
}
impl Fixture {
    fn new(populated: bool) -> Self {
        let temp = tempfile::Builder::new()
            .prefix("ocxd023-sync-identity-fixture-")
            .tempdir_in("/tmp")
            .unwrap();
        let root = temp.path().join("D");
        let home = temp.path().join("H");
        std::fs::create_dir_all(root.join("manager-state")).unwrap();
        std::fs::create_dir(&home).unwrap();
        let raw = if populated {
            json!({"providers":{"fixture":{"adapter":"openai-chat","baseUrl":"https://fixture.invalid/v1","authMode":"key","apiKey":SECRET,"models":["vendor/raw/slash"],"selectedModels":["vendor/raw/slash"],"modelPreset":{"mode":"custom"}}},"defaultProvider":"fixture"})
        } else {
            json!({"providers":{}})
        };
        std::fs::write(home.join("config.json"), serde_json::to_vec(&raw).unwrap()).unwrap();
        let owner = Arc::new(
            SyncIdentityOwnerV2::new(
                root.clone(),
                home.clone(),
                schema_v2::digest(b"exact endpoint binding"),
                Arc::new(Credentials),
            )
            .unwrap(),
        );
        Self {
            _temp: temp,
            root,
            home,
            owner,
        }
    }
    fn local_id(&self) -> String {
        self.owner.observe().unwrap().policy.channels[0].id.clone()
    }
    fn mapping(&self, portable: &str, local: &str) {
        let doc = MappingDocument {
            schema_version: 2,
            bindings: vec![MappingRow {
                endpoint_binding_id: self.owner.endpoint_binding_id.clone(),
                portable_id: portable.into(),
                local_id: local.into(),
                adopted_operation_id: OP.into(),
            }],
        };
        std::fs::write(
            self.root.join(MAPPING_PATH),
            schema_v2::canonical_bytes(&doc).unwrap(),
        )
        .unwrap();
    }
}
fn request(id: &str) -> CaptureRequestV2 {
    CaptureRequestV2 {
        selection: SelectionV2 {
            channels: vec![ChannelSelectionV2 {
                portable_channel_id: id.into(),
                model_ids: None,
                family_ids: None,
                include_connection: true,
                include_key: false,
                include_authorization: false,
            }],
            ..Default::default()
        },
        mcp_clients: BTreeMap::new(),
        assets_reviewed: false,
    }
}
fn cloud(id: &str, label: &str) -> SavedObjects {
    let provider = json!({"adapter":"openai-chat","baseUrl":"https://cloud.invalid/v1","models":["vendor/raw/slash"],"modelContextWindows":{"vendor/raw/slash":32000}});
    let channel = PortableChannelV2 {
        id: id.into(),
        label: label.into(),
        connection: Some(PortableConnectionV2 {
            nodes: vec![ServiceNode {
                id: NODE.into(),
                name: "node".into(),
                url: "https://cloud.invalid/v1".into(),
            }],
            active_node_id: NODE.into(),
            enabled: true,
            allow_private_network: false,
            auth_mode: Some("key".into()),
            provider: Some(provider.clone()),
        }),
        model_subset: None,
        family_scope: None,
        models: vec![ModelConfigV2 {
            raw_model_id: "vendor/raw/slash".into(),
            label: "raw".into(),
            observed: true,
            source: source_from_provider_v2(
                "cloud-source",
                "vendor/raw/slash",
                &provider,
                &schema_v2::digest(b"cloud source"),
                SourceKindV2::ImportedFile,
            )
            .unwrap(),
            explicit_overrides: ModelRules::default(),
            binding: None,
        }],
        template_refs: vec![],
        selection: ModelSelection::All,
        model_order: vec!["vendor/raw/slash".into()],
        defaults: ModelRules::default(),
        family_overrides: BTreeMap::new(),
        naming: BTreeMap::new(),
        namespace: None,
    };
    let payload = ModelPayloadV2 {
        templates: vec![],
        policy: Some(PortablePolicyV2 {
            template_refs: vec![],
            channels: vec![channel],
            channel_order: vec![id.into()],
            default_channel_id: None,
        }),
    };
    SavedObjects::capture(
        CapturedObjects {
            models: PortableModelsV2 {
                format: portable_v2::PORTABLE_FORMAT.into(),
                format_version: 6,
                schema_version: 2,
                created_at: "2026-11-06T10:00:00Z".into(),
                digest: hash(&payload).unwrap(),
                payload,
            },
            manager: Default::default(),
            assets: Default::default(),
            channel_keys: SecretValues::new(),
            channel_authorizations: BTreeMap::new(),
            webdav_passwords: SecretValues::new(),
            content_versions: vec![],
        },
        Some(&PayloadAuthorizationV2),
    )
    .unwrap()
}
fn full_cloud(f: &Fixture) -> (SavedObjects, CaptureRequestV2) {
    let mut value = cloud(CLOUD, "cloud").into_captured();
    value.manager.preferences = Some(crate::modules::preferences::Preferences::default());
    value.manager.extensions = Some(extensions::ExtensionConfig::default());
    value.manager.accounts = vec![portable_accounts::AccountInfo {
        id: OP.into(),
        provider: "chatgpt".into(),
        subject: Some("synthetic-cloud-subject".into()),
        label: "cloud account".into(),
        email: None,
        origin: None,
    }];
    let source = f._temp.path().join("cloud-skill-input");
    std::fs::create_dir(&source).unwrap();
    std::fs::write(source.join("SKILL.md"), "# Synthetic selected skill").unwrap();
    value.assets.skills =
        vec![portable_assets::snapshot_skill(&source, "fixture-skill", false, true).unwrap()];
    let mcp = portable_assets::split_mcp(
        discovery::RawServerDefinition {
            name: "fixture".into(),
            value: json!({"command":"synthetic-cloud-command"}),
        },
        false,
        true,
    )
    .unwrap();
    let mut r = request(CLOUD);
    r.selection.manager.preferences = true;
    r.selection.manager.extensions = true;
    r.selection.manager.account_ids = vec![OP.into()];
    r.selection.skill_names = vec!["fixture-skill".into()];
    r.selection.mcp_sources = vec![super::super::objects_overlay_v2::McpSourceSelectionV2 {
        name: mcp.name.clone(),
        definition_digest: super::super::objects_overlay_v2::mcp_source_digest(&mcp).unwrap(),
    }];
    r.mcp_clients.insert("fixture".into(), "managed".into());
    r.assets_reviewed = true;
    value.assets.mcps = vec![mcp];
    (
        SavedObjects::capture(value, Some(&PayloadAuthorizationV2)).unwrap(),
        r,
    )
}
fn code<T>(result: ModelResult<T>) -> String {
    match result {
        Ok(_) => panic!("expected refusal"),
        Err(e) => e.code,
    }
}
#[test]
fn new_device_absence_cancel_and_exact_same_transaction_adoption_reopen() {
    let f = Fixture::new(false);
    let c = cloud(CLOUD, "cloud");
    let raw_before = std::fs::read(f.home.join("config.json")).unwrap();
    let p = f
        .owner
        .freeze_preview(DirectionV2::Download, request(CLOUD), Some(&c))
        .unwrap();
    assert!(matches!(
        p.binding().slots[0].expected,
        TargetExpectationV2::ExpectedAbsent(_)
    ));
    assert!(p.local_capture_request().selection.channels.is_empty());
    assert_eq!(p.binding().mapping_revision, schema_v2::digest(b""));
    assert_eq!(p.receive_bindings()[0].local_channel_id, CLOUD);
    let frozen = p.binding().digest.clone();
    drop(p);
    assert!(!f.root.join(MAPPING_PATH).exists());
    assert_eq!(
        std::fs::read(f.home.join("config.json")).unwrap(),
        raw_before
    );
    let p = f
        .owner
        .freeze_preview(DirectionV2::Download, request(CLOUD), Some(&c))
        .unwrap();
    assert_eq!(p.binding().digest, frozen);
    let write = p.after_adoption(OP).unwrap();
    let expected_revision = p.expected_adoption_revision(OP).unwrap();
    let exact_after = write.after.clone();
    assert_eq!(expected_revision, schema_v2::digest(&exact_after));
    assert!(p.verify_adopted(OP).is_err());
    assert!(p.expected_adoption_revision(OTHER).is_err());
    assert!(write.before.is_none());
    let rows = document(Some(&write.after)).unwrap();
    assert_eq!(rows.bindings[0].adopted_operation_id, OP);
    assert!(!f.root.join(MAPPING_PATH).exists());
    let host = config::receive::ReceiveFileHostV2 {
        root: f.root.clone(),
        home: f.home.clone(),
        bindings: p.receive_bindings().to_vec(),
        trusted_mapping_write: Some(write),
    };
    let handle = host
        .prepare(
            OP,
            &c,
            &p.binding().remote_scope,
            &schema_v2::digest(b"selected target"),
        )
        .unwrap();
    assert!(!f.root.join(MAPPING_PATH).exists());
    host.protect(OP, &handle, false).unwrap();
    assert!(p.verify_adopted(OP).is_err());
    assert!(matches!(
        host.status(OP, &handle).unwrap(),
        super::super::sync_local_v2::ModelApplyStatus::Prepared
    ));
    host.apply_once(OP, &c, &handle, false).unwrap();
    // Service checks the own participant receipt separately from mapping bytes.
    assert!(matches!(host.status(OP, &handle).unwrap(),
        super::super::sync_local_v2::ModelApplyStatus::Applied(r) if r.operation_id == OP && r.plan == handle));
    assert!(f.owner.verify(p.binding()).is_err());
    p.verify_adopted(OP).unwrap();
    assert_eq!(p.expected_adoption_revision(OP).unwrap(), expected_revision);
    assert!(p.verify_adopted(OTHER).is_err());
    assert_eq!(
        document(Some(&std::fs::read(f.root.join(MAPPING_PATH)).unwrap()))
            .unwrap()
            .bindings[0]
            .local_id,
        CLOUD
    );
    let reopened = Arc::new(
        SyncIdentityOwnerV2::new(
            f.root.clone(),
            f.home.clone(),
            f.owner.endpoint_binding_id.clone(),
            Arc::new(Credentials),
        )
        .unwrap(),
    );
    let next = reopened
        .freeze_preview(DirectionV2::Download, request(CLOUD), Some(&c))
        .unwrap();
    assert_eq!(
        next.receive_bindings()[0].mode,
        ReceiveBindingModeV2::Existing
    );
    assert_eq!(next.receive_bindings()[0].local_channel_id, CLOUD);
    // Same semantics with different raw bytes are NOT this operation's image.
    let mut changed = exact_after.clone();
    changed.push(10);
    std::fs::write(f.root.join(MAPPING_PATH), changed).unwrap();
    assert_eq!(code(p.verify_adopted(OP)), "sync_identity_adoption_changed");
    let mut spoof = document(Some(&exact_after)).unwrap();
    spoof.bindings[0].adopted_operation_id = OTHER.into();
    std::fs::write(
        f.root.join(MAPPING_PATH),
        schema_v2::canonical_bytes(&spoof).unwrap(),
    )
    .unwrap();
    assert!(p.verify_adopted(OP).is_err());
    assert!(p.verify_adopted(OTHER).is_err());
    std::fs::write(f.root.join(MAPPING_PATH), exact_after).unwrap();
    p.verify_adopted(OP).unwrap();
}
#[test]
fn first_empty_device_full_remote_scope_keeps_only_actual_local_capture_sources() {
    let f = Fixture::new(false);
    let (cloud, request) = full_cloud(&f);
    let remote = request.selection.clone();
    let p = f
        .owner
        .freeze_preview(DirectionV2::Download, request, Some(&cloud))
        .unwrap();
    assert_eq!(p.binding().remote_scope, remote);
    let local = p.local_capture_request();
    assert!(local.selection.channels.is_empty());
    assert!(!local.selection.manager.preferences);
    assert!(!local.selection.manager.extensions);
    assert!(local.selection.manager.account_ids.is_empty());
    assert!(local.selection.skill_names.is_empty());
    assert!(local.selection.mcp_sources.is_empty());
    assert!(local.mcp_clients.is_empty());
    assert!(local.assets_reviewed);
    let captured = f.owner.capture(&local).unwrap();
    assert_eq!(p.source_guard(), &captured.source_guard);
    f.owner.verify(p.binding()).unwrap();
    let host = config::receive::ReceiveFileHostV2 {
        root: f.root.clone(),
        home: f.home.clone(),
        bindings: p.receive_bindings().to_vec(),
        trusted_mapping_write: Some(p.after_adoption(OP).unwrap()),
    };
    host.prepare(
        OP,
        &cloud,
        &remote,
        &schema_v2::digest(b"full remote selected target"),
    )
    .unwrap();
    for relative in [
        MAPPING_PATH,
        crate::modules::preferences::PREFERENCES_RELATIVE_PATH,
        projection::CONFIG_RELATIVE_PATH,
        portable_accounts::IMPORTED,
        "manager-state/account-catalog-ids.json",
    ] {
        assert!(!f.root.join(relative).exists(), "{relative}");
    }
}

#[test]
fn existing_nonmodel_sources_keep_actual_local_mcp_digest_and_explicit_client_guard() {
    let f = Fixture::new(false);
    let (cloud, mut request) = full_cloud(&f);
    crate::modules::preferences::PreferencesStore::new(&f.root)
        .save(&crate::modules::preferences::Preferences::default())
        .unwrap();
    let extension = extensions::ExtensionConfig::default();
    let saved = projection::ExtensionProjection {
        fingerprint: extensions::ProjectionFingerprint::create(
            extension.fingerprint(),
            chrono::DateTime::parse_from_rfc3339("2026-11-07T00:00:00Z")
                .unwrap()
                .to_utc(),
        ),
        config: extension,
    };
    std::fs::write(
        f.root.join(projection::CONFIG_RELATIVE_PATH),
        serde_json::to_vec(&saved).unwrap(),
    )
    .unwrap();
    std::fs::write(
        f.root.join(portable_accounts::IMPORTED),
        serde_json::to_vec(&cloud.partitions().manager.accounts).unwrap(),
    )
    .unwrap();
    let skill = discovery::agents_skills_dir(&f.home).join("fixture-skill");
    std::fs::create_dir_all(&skill).unwrap();
    std::fs::write(skill.join("SKILL.md"), "# Actual local selected skill").unwrap();
    std::fs::create_dir(f.home.join(".codex")).unwrap();
    let source = ["[mcp_servers.fixture]", "command='synthetic-local-command'"]
        .join(&char::from(10).to_string());
    std::fs::write(f.home.join(".codex/config.toml"), source).unwrap();
    std::fs::create_dir(f.home.join(".hermes")).unwrap();
    std::fs::write(
        f.home.join(".hermes/config.yaml"),
        "malformed unselected source: [",
    )
    .unwrap();
    request.mcp_clients.insert("fixture".into(), "codex".into());
    let remote = request.selection.clone();
    let p = f
        .owner
        .freeze_preview(DirectionV2::Download, request, Some(&cloud))
        .unwrap();
    assert_eq!(p.binding().remote_scope, remote);
    let local = p.local_capture_request();
    assert!(local.selection.manager.preferences && local.selection.manager.extensions);
    assert_eq!(local.selection.manager.account_ids, vec![OP.to_owned()]);
    assert_eq!(local.selection.skill_names, remote.skill_names);
    assert_eq!(local.mcp_clients["fixture"], "codex");
    assert_ne!(
        local.selection.mcp_sources[0].definition_digest,
        remote.mcp_sources[0].definition_digest
    );
    assert_eq!(
        p.source_guard(),
        &f.owner.capture(&local).unwrap().source_guard
    );
    f.owner.verify(p.binding()).unwrap();
    std::fs::write(
        f.home.join(".codex/config.toml"),
        "broken selected source [",
    )
    .unwrap();
    assert!(f.owner.verify(p.binding()).is_err());
}

#[test]
fn nonmodel_read_failure_is_not_filtered_as_absence() {
    for source in ["preferences", "extensions", "mcp"] {
        let f = Fixture::new(false);
        let (cloud, request) = full_cloud(&f);
        let path = f.root.join(match source {
            "preferences" => crate::modules::preferences::PREFERENCES_RELATIVE_PATH,
            "extensions" => projection::CONFIG_RELATIVE_PATH,
            _ => "manager-state/mcp-definitions.json",
        });
        std::fs::write(path, "{broken").unwrap();
        assert!(
            f.owner
                .freeze_preview(DirectionV2::Download, request, Some(&cloud))
                .is_err(),
            "{source}"
        );
    }
}

#[test]
fn known_different_uuid_pair_and_missing_known_target_keep_fixed_identity() {
    let f = Fixture::new(true);
    let local = f.local_id();
    f.mapping(CLOUD, &local);
    let c = cloud(CLOUD, "different label");
    let mut r = request(CLOUD);
    r.selection.channels[0].include_connection = false;
    let p = f
        .owner
        .freeze_preview(DirectionV2::Download, r, Some(&c))
        .unwrap();
    assert_eq!(p.receive_bindings()[0].local_channel_id, local);
    assert_eq!(p.receive_bindings()[0].mode, ReceiveBindingModeV2::Existing);
    let empty = Fixture::new(false);
    empty.mapping(CLOUD, OTHER);
    let p = empty
        .owner
        .freeze_preview(DirectionV2::Download, request(CLOUD), Some(&c))
        .unwrap();
    assert_eq!(p.receive_bindings()[0].local_channel_id, OTHER);
    assert!(p.local_capture_request().selection.channels.is_empty());
    assert!(matches!(
        p.binding().slots[0].expected,
        TargetExpectationV2::ExpectedAbsent(_)
    ));
}

#[test]
fn known_mapping_noop_keeps_raw_revision_and_does_not_prove_model_applied() {
    let f = Fixture::new(false);
    f.mapping(CLOUD, OTHER);
    let doc = document(Some(&std::fs::read(f.root.join(MAPPING_PATH)).unwrap())).unwrap();
    let before = serde_json::to_vec_pretty(&doc).unwrap();
    std::fs::write(f.root.join(MAPPING_PATH), &before).unwrap();
    let c = cloud(CLOUD, "cloud");
    let p = f
        .owner
        .freeze_preview(DirectionV2::Download, request(CLOUD), Some(&c))
        .unwrap();
    assert!(p.verify_adopted(NODE).is_err());
    assert_eq!(
        p.expected_adoption_revision(NODE).unwrap(),
        p.binding().mapping_revision
    );
    let write = p.after_adoption(NODE).unwrap();
    assert_eq!(write.before.as_deref(), Some(before.as_slice()));
    assert_eq!(write.after, before);
    assert_eq!(
        document(Some(&write.after)).unwrap().bindings[0].adopted_operation_id,
        OP
    );
    // Matching no-op bytes are not a ModelApply receipt or new write authority.
    p.verify_adopted(NODE).unwrap();
    assert!(!f
        .root
        .join("manager-state/model-receive-v2")
        .join(NODE)
        .join("receipt.json")
        .exists());
    assert!(p.verify_adopted(OP).is_err());
}

#[test]
fn post_apply_bytes_cannot_issue_an_unfrozen_operation() {
    let f = Fixture::new(false);
    let c = cloud(CLOUD, "cloud");
    let p = f
        .owner
        .freeze_preview(DirectionV2::Download, request(CLOUD), Some(&c))
        .unwrap();
    // Even exactly plausible rows for OP cannot replace pre-apply issuance.
    f.mapping(CLOUD, CLOUD);
    assert_eq!(
        code(p.verify_adopted(OP)),
        "sync_identity_adoption_unfrozen"
    );
    assert!(p.expected_adoption_revision(OP).is_err());
    assert!(p.after_adoption(OP).is_err());
}
#[test]
fn uuid_occupied_without_pair_requires_explicit_mapping_same_name_never_guesses() {
    let f = Fixture::new(true);
    let local = f.local_id();
    let c = cloud(&local, "fixture");
    assert_eq!(
        code(
            f.owner
                .freeze_preview(DirectionV2::Download, request(&local), Some(&c))
        ),
        "sync_identity_explicit_mapping_required"
    );
    let c = cloud(CLOUD, "fixture");
    assert_eq!(
        code(
            f.owner
                .freeze_preview(DirectionV2::Download, request(CLOUD), Some(&c))
        ),
        "model_receive_identity_conflict"
    );
    assert!(!f.root.join(MAPPING_PATH).exists());
}

#[test]
fn explicit_same_uuid_download_freezes_present_then_adopts_only_in_receive_transaction() {
    let f = Fixture::new(true);
    let id = f.local_id();
    let c = cloud(&id, "different remote label");
    let mut r = request(&id);
    // Preserve the actual local connection and credentials in this model-only
    // selection. Explicit identity adoption confers no connection authority.
    r.selection.channels[0].include_connection = false;
    assert_eq!(
        code(
            f.owner
                .freeze_preview(DirectionV2::Download, request_copy(&r), Some(&c))
        ),
        "sync_identity_explicit_mapping_required"
    );
    let p = f
        .owner
        .freeze_preview_adopting_same_ids(request_copy(&r), &c)
        .unwrap();
    assert_eq!(p.binding().direction, DirectionV2::Download);
    assert_eq!(p.binding().slots[0].portable_id, id);
    assert_eq!(p.binding().slots[0].local_id, id);
    assert_eq!(p.receive_bindings()[0].mode, ReceiveBindingModeV2::Existing);
    let captured = f.owner.capture(&p.local_capture_request()).unwrap();
    let atoms = file_objects_v2::save_state(&captured.objects).unwrap();
    let atom = channel_atom(&captured.objects, &atoms, &id).unwrap();
    assert!(
        matches!(&p.binding().slots[0].expected, TargetExpectationV2::Present(d) if d == &atom)
    );
    f.owner.verify(p.binding()).unwrap();
    assert!(!f.root.join(MAPPING_PATH).exists());
    // Issuing an explicit preview never unlocks the original strict API.
    assert!(f
        .owner
        .freeze_preview(DirectionV2::Download, request_copy(&r), Some(&c))
        .is_err());
    let write = p.after_adoption(OP).unwrap();
    assert!(write.before.is_none());
    let doc = document(Some(&write.after)).unwrap();
    assert_eq!(doc.bindings[0].portable_id, id);
    assert_eq!(doc.bindings[0].local_id, id);
    assert_eq!(doc.bindings[0].adopted_operation_id, OP);
    let host = config::receive::ReceiveFileHostV2 {
        root: f.root.clone(),
        home: f.home.clone(),
        bindings: p.receive_bindings().to_vec(),
        trusted_mapping_write: Some(write),
    };
    let handle = host
        .prepare(
            OP,
            &c,
            &r.selection,
            &schema_v2::digest(b"explicit same uuid target"),
        )
        .unwrap();
    assert!(!f.root.join(MAPPING_PATH).exists());
    host.protect(OP, &handle, false).unwrap();
    host.apply_once(OP, &c, &handle, false).unwrap();
    assert!(matches!(host.status(OP, &handle).unwrap(),
        super::super::sync_local_v2::ModelApplyStatus::Applied(receipt) if receipt.operation_id == OP && receipt.plan == handle));
    p.verify_adopted(OP).unwrap();
}

#[test]
fn explicit_same_uuid_preview_cannot_steal_a_pair_or_guess_a_different_uuid_by_name() {
    let f = Fixture::new(true);
    let local = f.local_id();
    f.mapping(CLOUD, &local);
    let before = std::fs::read(f.root.join(MAPPING_PATH)).unwrap();
    let c = cloud(&local, "fixture");
    let mut r = request(&local);
    r.selection.channels[0].include_connection = false;
    assert_eq!(
        code(f.owner.freeze_preview_adopting_same_ids(r, &c)),
        "sync_identity_explicit_mapping_required"
    );
    assert_eq!(std::fs::read(f.root.join(MAPPING_PATH)).unwrap(), before);
    let f = Fixture::new(true);
    let c = cloud(CLOUD, "fixture");
    assert_eq!(
        code(f.owner.freeze_preview_adopting_same_ids(request(CLOUD), &c)),
        "model_receive_identity_conflict"
    );
    assert!(!f.root.join(MAPPING_PATH).exists());
}

#[test]
fn explicit_same_uuid_present_verification_rejects_actual_source_and_roster_changes() {
    for change in ["model", "roster", "key"] {
        let f = Fixture::new(true);
        let local = f.local_id();
        let c = cloud(&local, "cloud");
        let mut r = request(&local);
        r.selection.channels[0].include_connection = false;
        let p = f.owner.freeze_preview_adopting_same_ids(r, &c).unwrap();
        let path = f.home.join("config.json");
        let mut raw: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        match change {
            "model" => {
                raw["providers"]["fixture"]["modelContextWindows"] =
                    json!({"vendor/raw/slash":7777})
            }
            "key" => raw["providers"]["fixture"]["apiKey"] = json!("synthetic-changed-key"),
            _ => raw["providers"]["another"] = json!({"adapter":"openai-chat","models":[]}),
        }
        std::fs::write(path, serde_json::to_vec(&raw).unwrap()).unwrap();
        assert!(f.owner.verify(p.binding()).is_err(), "{change}");
        assert!(!f.root.join(MAPPING_PATH).exists());
    }
}
#[test]
fn upload_projects_only_channel_identity_and_preserves_source_models_refs() {
    let f = Fixture::new(true);
    let path = f.home.join("config.json");
    let mut raw: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    raw["providers"]["fixture"]["models"] = json!(["vendor/raw/slash", "gpt-6-sol"]);
    raw["providers"]["fixture"]["selectedModels"] = json!(["vendor/raw/slash", "gpt-6-sol"]);
    std::fs::write(path, serde_json::to_vec(&raw).unwrap()).unwrap();
    let local = f.local_id();
    let saved = config::snapshot(&f.root, &f.home).unwrap();
    let imported = config::read_existing(
        &f.root,
        &f.home,
        config::ReadRequestV2 {
            expected_source: saved.source_digest,
            expected_library: saved.library_digest,
            expected_draft: saved.draft.map(|d| d.base),
        },
    )
    .unwrap()
    .snapshot;
    config::save(
        &f.root,
        &f.home,
        config::SaveRequestV2 {
            expected: imported.draft.unwrap().base,
            edits: vec![super::super::source_v2::ModelEditV2::SetOverride {
                channel_id: local.clone(),
                raw_model_id: "vendor/raw/slash".into(),
                field: "contextWindow".into(),
                value: json!(8192),
            }],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )
    .unwrap();
    f.mapping(CLOUD, &local);
    let mut r = request(&local);
    r.selection.channels[0].include_key = true;
    let p = f
        .owner
        .freeze_preview(DirectionV2::Upload, r, None)
        .unwrap();
    assert_eq!(
        p.binding().remote_scope.channels[0].portable_channel_id,
        CLOUD
    );
    let captured = f.owner.capture(&p.local_capture_request()).unwrap();
    let source = captured
        .objects
        .models()
        .payload
        .policy
        .as_ref()
        .unwrap()
        .channels[0]
        .clone();
    let templates = captured.objects.models().payload.templates.clone();
    assert!(!templates.is_empty());
    assert!(source.models.iter().any(|m| m.binding.is_some()));
    let projected = f
        .owner
        .project_saved_to_remote(captured.objects, p.binding())
        .unwrap();
    let channel = &projected.models().payload.policy.as_ref().unwrap().channels[0];
    assert_eq!(channel.id, CLOUD);
    assert_eq!(channel.models, source.models);
    assert_eq!(channel.template_refs, source.template_refs);
    assert_eq!(projected.models().payload.templates, templates);
    assert!(channel
        .models
        .iter()
        .any(|m| m.raw_model_id == "vendor/raw/slash"));
    assert_eq!(projected.partitions().channel_keys.get(CLOUD), Some(SECRET));
    assert!(projected.partitions().channel_keys.get(&local).is_none());
    let binding_json = serde_json::to_string(p.binding()).unwrap();
    assert!(!binding_json.contains(SECRET));
    assert!(
        !String::from_utf8(p.after_adoption(OP).unwrap().after.clone())
            .unwrap()
            .contains(SECRET)
    );
    portable_v2::validate_portable_models_v2(projected.models()).unwrap();
}
#[test]
fn stable_local_uuid_upload_never_randomly_allocates() {
    let f = Fixture::new(true);
    let id = f.local_id();
    let p = f
        .owner
        .freeze_preview(DirectionV2::Upload, request(&id), None)
        .unwrap();
    assert_eq!(p.binding().slots[0].portable_id, id);
    assert_eq!(p.binding().slots[0].local_id, id);
}

#[test]
fn same_uuid_and_known_pair_key_maps_move_without_cloning() {
    let f = Fixture::new(true);
    let local = f.local_id();
    let mut r = request(&local);
    r.selection.channels[0].include_key = true;
    let p = f
        .owner
        .freeze_preview(DirectionV2::Upload, request_copy(&r), None)
        .unwrap();
    let objects = f.owner.capture(&p.local_capture_request()).unwrap().objects;
    let projected = f
        .owner
        .project_saved_to_remote(objects, p.binding())
        .unwrap();
    assert_eq!(
        projected.partitions().channel_keys.get(&local),
        Some(SECRET)
    );
    f.mapping(CLOUD, &local);
    let p = f
        .owner
        .freeze_preview(DirectionV2::Upload, r, None)
        .unwrap();
    let objects = f.owner.capture(&p.local_capture_request()).unwrap().objects;
    let projected = f
        .owner
        .project_saved_to_remote(objects, p.binding())
        .unwrap();
    assert_eq!(projected.partitions().channel_keys.get(CLOUD), Some(SECRET));
    assert!(projected.partitions().channel_keys.get(&local).is_none());
    let mut captured = f
        .owner
        .capture(&p.local_capture_request())
        .unwrap()
        .objects
        .into_captured();
    captured
        .channel_keys
        .insert(OTHER.into(), zeroize::Zeroizing::new(SECRET.into()))
        .unwrap();
    assert!(SavedObjects::capture(captured, Some(&PayloadAuthorizationV2)).is_err());
    let mut forged = p.binding().clone();
    forged.slots[0].local_id = OTHER.into();
    let objects = f.owner.capture(&p.local_capture_request()).unwrap().objects;
    assert!(f.owner.project_saved_to_remote(objects, &forged).is_err());
}
#[test]
fn source_key_model_and_roster_changes_refuse_verification() {
    for mutation in ["key", "model", "source", "roster"] {
        let f = Fixture::new(true);
        let id = f.local_id();
        let mut r = request(&id);
        r.selection.channels[0].include_key = true;
        let p = f
            .owner
            .freeze_preview(DirectionV2::Upload, r, None)
            .unwrap();
        let path = f.home.join("config.json");
        let mut raw: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        match mutation {
            "key" => raw["providers"]["fixture"]["apiKey"] = json!("synthetic-changed"),
            "model" => {
                raw["providers"]["fixture"]["modelContextWindows"] =
                    json!({"vendor/raw/slash":7777})
            }
            "source" => {
                raw["providers"]["fixture"]["baseUrl"] = json!("https://changed.invalid/v1")
            }
            _ => raw["providers"]["other"] = json!({"adapter":"openai-chat","models":[]}),
        }
        std::fs::write(path, serde_json::to_vec(&raw).unwrap()).unwrap();
        assert!(f.owner.verify(p.binding()).is_err(), "{mutation}");
    }
}
#[test]
fn read_failure_never_becomes_absent_and_raw_byte_map_change_invalidates() {
    let f = Fixture::new(false);
    let c = cloud(CLOUD, "cloud");
    std::fs::write(f.home.join("config.json"), b"invalid").unwrap();
    assert!(f
        .owner
        .freeze_preview(DirectionV2::Download, request(CLOUD), Some(&c))
        .is_err());
    std::fs::write(f.home.join("config.json"), b"{\"providers\":{}}").unwrap();
    std::fs::create_dir(f.root.join(MAPPING_PATH)).unwrap();
    assert_eq!(
        code(
            f.owner
                .freeze_preview(DirectionV2::Download, request(CLOUD), Some(&c))
        ),
        "sync_identity_read"
    );
    std::fs::remove_dir(f.root.join(MAPPING_PATH)).unwrap();
    let p = f
        .owner
        .freeze_preview(DirectionV2::Download, request(CLOUD), Some(&c))
        .unwrap();
    std::fs::write(
        f.root.join(MAPPING_PATH),
        b"{\"schemaVersion\":2,\"bindings\":[]}",
    )
    .unwrap();
    assert!(f.owner.verify(p.binding()).is_err());
}
#[test]
fn strict_schema_duplicates_and_budgets_are_rejected() {
    assert_eq!(
        code(document(Some(b"{\"schemaVersion\":1,\"bindings\":[]}"))),
        "model_schema_unsupported"
    );
    assert!(document(Some(
        b"{\"schemaVersion\":2,\"schemaVersion\":2,\"bindings\":[]}"
    ))
    .is_err());
    assert!(document(Some(
        b"{\"schemaVersion\":2,\"bindings\":[],\"unknown\":true}"
    ))
    .is_err());
    assert_eq!(
        code(document(Some(&vec![b' '; MAX_MAP_BYTES as usize + 1]))),
        "sync_identity_limit"
    );
    let f = Fixture::new(false);
    f.mapping(CLOUD, OTHER);
    let mut doc = document(Some(&std::fs::read(f.root.join(MAPPING_PATH)).unwrap())).unwrap();
    doc.bindings.push(doc.bindings[0].clone());
    assert_eq!(
        code(document(Some(&schema_v2::canonical_bytes(&doc).unwrap()))),
        "sync_identity_conflict"
    );
    doc.bindings[1].portable_id = OP.into();
    assert!(document(Some(&schema_v2::canonical_bytes(&doc).unwrap())).is_err());
    doc.bindings = vec![doc.bindings[0].clone(); MAX_BINDINGS + 1];
    assert_eq!(
        code(document(Some(&schema_v2::canonical_bytes(&doc).unwrap()))),
        "sync_identity_limit"
    );
}
#[test]
fn unissued_or_modified_binding_has_no_authority() {
    let f = Fixture::new(false);
    let c = cloud(CLOUD, "cloud");
    let p = f
        .owner
        .freeze_preview(DirectionV2::Download, request(CLOUD), Some(&c))
        .unwrap();
    let mut forged = p.binding().clone();
    forged.slots[0].local_id = OTHER.into();
    assert!(f.owner.verify(&forged).is_err());
    let binding = p.binding().clone();
    drop(p);
    assert_eq!(
        code(f.owner.verify(&binding)),
        "sync_identity_untrusted_preview"
    );
}
#[cfg(unix)]
#[test]
fn owned_mapping_parent_leaf_and_root_symlinks_refuse() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new(false);
    let c = cloud(CLOUD, "cloud");
    let outside = f._temp.path().join("outside");
    std::fs::create_dir(&outside).unwrap();
    symlink(outside.join("missing"), f.root.join(MAPPING_PATH)).unwrap();
    assert!(f
        .owner
        .freeze_preview(DirectionV2::Download, request(CLOUD), Some(&c))
        .is_err());
    std::fs::remove_file(f.root.join(MAPPING_PATH)).unwrap();
    std::fs::remove_dir(f.root.join("manager-state")).unwrap();
    symlink(&outside, f.root.join("manager-state")).unwrap();
    assert!(f
        .owner
        .freeze_preview(DirectionV2::Download, request(CLOUD), Some(&c))
        .is_err());
    let link = f._temp.path().join("linked-D");
    symlink(&f.root, &link).unwrap();
    assert!(SyncIdentityOwnerV2::new(
        link,
        f.home.clone(),
        f.owner.endpoint_binding_id.clone(),
        Arc::new(Credentials)
    )
    .is_err());
}

#[test]
fn partial_present_uses_exact_v2_atom_and_preserves_full_mcp_guard() {
    let f = Fixture::new(true);
    let id = f.local_id();
    let mut r = request(&id);
    r.selection.channels[0].model_ids = Some(vec!["vendor/raw/slash".into()]);
    let p = f
        .owner
        .freeze_preview(DirectionV2::Upload, r, None)
        .unwrap();
    let captured = f.owner.capture(&p.local_capture_request()).unwrap();
    let atoms = file_objects_v2::save_state(&captured.objects).unwrap();
    let actual = channel_atom(&captured.objects, &atoms, &id).unwrap();
    assert!(
        matches!(&p.binding().slots[0].expected,TargetExpectationV2::Present(d) if d == &actual)
    );
    assert_eq!(p.source_guard(), &captured.source_guard);
    f.owner.verify(p.binding()).unwrap();
}

#[test]
fn endpoint_namespace_is_exact_and_old_path_is_not_consulted() {
    let f = Fixture::new(true);
    let local = f.local_id();
    f.mapping(CLOUD, &local);
    let other_owner = Arc::new(
        SyncIdentityOwnerV2::new(
            f.root.clone(),
            f.home.clone(),
            schema_v2::digest(b"another endpoint"),
            Arc::new(Credentials),
        )
        .unwrap(),
    );
    let c = cloud(CLOUD, "cloud");
    let p = other_owner
        .freeze_preview(DirectionV2::Download, request(CLOUD), Some(&c))
        .unwrap();
    assert_eq!(p.receive_bindings()[0].local_channel_id, CLOUD);
    assert_eq!(p.receive_bindings()[0].mode, ReceiveBindingModeV2::New);
    std::fs::remove_file(f.root.join(MAPPING_PATH)).unwrap();
    std::fs::write(
        f.root.join("manager-state/sync-identity-mappings.json"),
        b"invalid old metadata must not be read",
    )
    .unwrap();
    let p = f
        .owner
        .freeze_preview(DirectionV2::Upload, request(&local), None)
        .unwrap();
    assert_eq!(p.binding().slots[0].portable_id, local);
}

struct AuthorizationCredentials;
impl CaptureCredentialsV2 for AuthorizationCredentials {
    fn authorization(
        &self,
        home: &Path,
        provider: &str,
        scoped: &Value,
        source: &Value,
    ) -> ModelResult<super::super::provider_auth::PortableAuthorization> {
        super::super::provider_auth::export_portable_scoped(home, provider, scoped, source)
    }
}
#[test]
fn authorization_map_moves_through_actual_provider_validation() {
    let f = Fixture::new(true);
    let local = f.local_id();
    f.mapping(CLOUD, &local);
    let owner = Arc::new(
        SyncIdentityOwnerV2::new(
            f.root.clone(),
            f.home.clone(),
            f.owner.endpoint_binding_id.clone(),
            Arc::new(AuthorizationCredentials),
        )
        .unwrap(),
    );
    let mut r = request(&local);
    r.selection.channels[0].include_authorization = true;
    let p = owner.freeze_preview(DirectionV2::Upload, r, None).unwrap();
    let captured = owner.capture(&p.local_capture_request()).unwrap();
    let projected = owner
        .project_saved_to_remote(captured.objects, p.binding())
        .unwrap();
    assert!(projected
        .partitions()
        .channel_authorizations
        .contains_key(CLOUD));
    assert!(!projected
        .partitions()
        .channel_authorizations
        .contains_key(&local));
    assert!(file_objects_v2::encode_document(&projected, Some(&PayloadAuthorizationV2)).is_ok());
    assert!(!serde_json::to_string(p.binding()).unwrap().contains(SECRET));
}

#[test]
fn positively_missing_saved_config_is_empty_but_malformed_is_never_empty() {
    let f = Fixture::new(false);
    let c = cloud(CLOUD, "cloud");
    std::fs::remove_file(f.home.join("config.json")).unwrap();
    let p = f
        .owner
        .freeze_preview(DirectionV2::Download, request(CLOUD), Some(&c))
        .unwrap();
    assert!(matches!(
        p.binding().slots[0].expected,
        TargetExpectationV2::ExpectedAbsent(_)
    ));
    assert!(!f.home.join("config.json").exists());
}

#[cfg(unix)]
#[test]
fn invalid_draft_is_never_a_runtime_source() {
    let f = Fixture::new(true);
    let local = f.local_id();
    std::os::unix::fs::symlink(
        f._temp.path().join("missing-draft"),
        f.root.join(super::super::store_v2::DRAFT_PATH),
    )
    .unwrap();
    let p = f
        .owner
        .freeze_preview(DirectionV2::Upload, request(&local), None)
        .unwrap();
    f.owner.verify(p.binding()).unwrap();
}

struct OriginalMappedSource {
    current: host::CurrentSourceProviderV2,
    preview: Arc<MappingPreviewV2>,
}
impl local::SourceSnapshotProviderV2 for OriginalMappedSource {
    fn current(&self, _: &Path, _: &Path, _: &CaptureRequestV2) -> ModelResult<SourceGuard> {
        self.current
            .current_source()
            .map_err(|_| err("capture_stale"))
    }
    fn root(&self, _: &Path, _: &Path, _: &CaptureRequestV2) -> ModelResult<String> {
        self.current
            .root_revision()
            .map_err(|_| err("capture_stale"))
    }
    fn target(&self, d: &Path, h: &Path, r: &CaptureRequestV2) -> ModelResult<String> {
        Ok(self.current(d, h, r)?.saved_source_digest)
    }
    fn verify_applied_mapping(&self, id: &str, _: &LocalTargetBindingV2) -> ModelResult<()> {
        self.preview.verify_adopted(id)
    }
}
struct NoRecoveryCapture;
impl local::SourceSnapshotProviderV2 for NoRecoveryCapture {
    fn current(&self, _: &Path, _: &Path, _: &CaptureRequestV2) -> ModelResult<SourceGuard> {
        panic!("remaining recovery must not capture today's source")
    }
    fn root(&self, _: &Path, _: &Path, _: &CaptureRequestV2) -> ModelResult<String> {
        panic!("remaining recovery must not resolve today's root")
    }
    fn target(&self, _: &Path, _: &Path, _: &CaptureRequestV2) -> ModelResult<String> {
        panic!("remaining recovery must not create a baseline")
    }
}
struct CutAfterRealReceive {
    receiver: Arc<config::receive::ReceiveFileHostV2>,
    read_only: bool,
    remembered_handle: Mutex<Option<ModelApplyPlanHandle>>,
}
impl local::ModelApplyV2 for CutAfterRealReceive {
    fn prepare(
        &self,
        id: &str,
        o: &SavedObjects,
        s: &super::super::objects_overlay_v2::SelectionV2,
        e: &str,
    ) -> ModelResult<ModelApplyPlanHandle> {
        assert!(
            !self.read_only,
            "remaining recovery prepared a new model write"
        );
        let handle = self.receiver.prepare(id, o, s, e)?;
        *self.remembered_handle.lock().unwrap() = Some(handle.clone());
        Ok(handle)
    }
    fn protect(&self, id: &str, h: &ModelApplyPlanHandle, b: bool) -> ModelResult<()> {
        assert!(
            !self.read_only,
            "remaining recovery repeated model protection"
        );
        self.receiver.protect(id, h, b)
    }
    fn apply_once(
        &self,
        id: &str,
        o: &SavedObjects,
        h: &ModelApplyPlanHandle,
        b: bool,
    ) -> ModelResult<local::ModelApplyReceipt> {
        assert!(!self.read_only, "remaining recovery rewrote models");
        self.receiver.apply_once(id, o, h, b)?;
        Err(err("synthetic_crash_after_real_model_receipt"))
    }
    fn status(&self, id: &str, h: &ModelApplyPlanHandle) -> ModelResult<local::ModelApplyStatus> {
        self.receiver.status(id, h)
    }
}

#[test]
fn restart_restores_original_absent_with_dual_real_proofs_and_finishes_only_old_participants() {
    let f = Fixture::new(false);
    let (cloud, request) = full_cloud(&f);
    let preview = Arc::new(
        f.owner
            .freeze_preview(DirectionV2::Download, request, Some(&cloud))
            .unwrap(),
    );
    let binding = preview.binding().clone();
    let original_bytes = serde_json::to_vec(&binding).unwrap();
    let request_bytes = serde_json::to_vec(&preview.local_capture_request()).unwrap();
    let receiver = Arc::new(config::receive::ReceiveFileHostV2 {
        root: f.root.clone(),
        home: f.home.clone(),
        bindings: preview.receive_bindings().to_vec(),
        trusted_mapping_write: Some(preview.after_adoption(OP).unwrap()),
    });
    let actual = Arc::new(
        host::SavedCaptureProviderV2::new(
            f.root.clone(),
            f.home.clone(),
            preview.local_capture_request(),
            Arc::new(Credentials),
        )
        .unwrap(),
    );
    let mapped = Arc::new(
        host::MappedCaptureProviderV2::new(actual, f.owner.clone(), binding.clone()).unwrap(),
    );
    let captured = mapped.capture_saved(&binding.remote_scope).unwrap();
    let scope = captured.source_guard.scope_digest.clone();
    let expected = captured.source_guard.saved_source_digest.clone();
    let source = Arc::new(OriginalMappedSource {
        current: host::CurrentSourceProviderV2::new(
            mapped,
            binding.remote_scope.clone(),
            captured.root_revision,
        )
        .unwrap(),
        preview: preview.clone(),
    });
    let payload = file_objects_v2::encode(
        &cloud,
        OP,
        "2026-11-07T01:00:00Z",
        None,
        Some(&PayloadAuthorizationV2),
    )
    .unwrap()
    .into_bytes();
    let model_host = Arc::new(CutAfterRealReceive {
        receiver: receiver.clone(),
        read_only: false,
        remembered_handle: Mutex::new(None),
    });
    let app = local::LocalApplicationV2::new_mapped(
        f.root.clone(),
        f.home.clone(),
        binding.remote_scope.clone(),
        preview.local_capture_request(),
        scope.clone(),
        source,
        model_host.clone(),
        None,
        binding.clone(),
        f.owner.clone(),
    )
    .unwrap();
    assert!(app
        .apply_once(
            OP,
            &payload,
            &file_objects_v2::supported_schema(),
            &scope,
            &expected,
            false
        )
        .is_err());
    assert_eq!(
        app.progress(OP).unwrap(),
        Some(local::LocalProgressV2::Protected)
    );
    // The Service retained the handle returned by the real participant. It
    // does not parse the mapper/local journal to reconstruct that context.
    let journal_path = f
        .root
        .join("manager-state/sync-local-v2")
        .join(format!("{OP}.json"));
    let handle = model_host
        .remembered_handle
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .clone();
    assert!(matches!(
        receiver.status(OP, &handle).unwrap(),
        local::ModelApplyStatus::Applied(_)
    ));
    let model_proof = receiver.applied_mapping_proof(OP, &handle).unwrap();
    assert!(f.owner.verify(&binding).is_err());
    drop(app);
    drop(preview);
    let model_config_after = std::fs::read(f.home.join("config.json")).unwrap();
    let map_after = std::fs::read(f.root.join(MAPPING_PATH)).unwrap();
    let owner = Arc::new(
        SyncIdentityOwnerV2::new(
            f.root.clone(),
            f.home.clone(),
            f.owner.endpoint_binding_id.clone(),
            Arc::new(Credentials),
        )
        .unwrap(),
    );
    let witness = || {
        local::LocalApplicationV2::local_model_journal_proof(&f.root, &f.home, OP, &handle).unwrap()
    };
    let mut forged = binding.clone();
    if let TargetExpectationV2::ExpectedAbsent(d) = &mut forged.slots[0].expected {
        *d = schema_v2::digest(b"forged absent");
    }
    let forged = LocalTargetBindingV2::freeze(
        forged.mapping_revision,
        forged.local_scope,
        forged.remote_scope,
        forged.direction,
        forged.slots,
    )
    .unwrap();
    assert!(owner
        .restore_adopted(
            &serde_json::to_vec(&forged).unwrap(),
            witness(),
            &model_proof
        )
        .is_err());
    let other_endpoint = Arc::new(
        SyncIdentityOwnerV2::new(
            f.root.clone(),
            f.home.clone(),
            schema_v2::digest(b"other endpoint"),
            Arc::new(Credentials),
        )
        .unwrap(),
    );
    assert!(other_endpoint
        .restore_adopted(&original_bytes, witness(), &model_proof)
        .is_err());
    let mut edited = map_after.clone();
    edited.push(10);
    std::fs::write(f.root.join(MAPPING_PATH), edited).unwrap();
    assert!(owner
        .restore_adopted(&original_bytes, witness(), &model_proof)
        .is_err());
    std::fs::write(f.root.join(MAPPING_PATH), &map_after).unwrap();
    let proof = owner
        .restore_adopted(&original_bytes, witness(), &model_proof)
        .unwrap();
    assert!(proof.binding() == &binding);
    assert!(matches!(
        proof.binding().slots[0].expected,
        TargetExpectationV2::ExpectedAbsent(_)
    ));
    assert!(owner.verify(proof.binding()).is_err());
    let capture = || serde_json::from_slice::<CaptureRequestV2>(&request_bytes).unwrap();
    let remaining = local::LocalApplicationV2::new_mapped_remaining(
        f.root.clone(),
        f.home.clone(),
        binding.remote_scope.clone(),
        capture(),
        scope.clone(),
        Arc::new(NoRecoveryCapture),
        Arc::new(CutAfterRealReceive {
            receiver: receiver.clone(),
            read_only: true,
            remembered_handle: Mutex::new(None),
        }),
        None,
        proof.clone(),
    )
    .unwrap();
    assert!(remaining.current_source().is_err());
    assert!(remaining.target_digest(&scope).is_err());
    assert!(remaining
        .export_once(OP, &payload, &file_objects_v2::supported_schema(), &scope)
        .is_err());
    assert!(remaining
        .apply_once(
            OTHER,
            &payload,
            &file_objects_v2::supported_schema(),
            &scope,
            &expected,
            false
        )
        .is_err());
    assert!(!f
        .root
        .join("manager-state/sync-local-v2")
        .join(format!("{OTHER}.json"))
        .exists());
    // Every call rechecks mapping and old context before remaining writes.
    let mut changed = map_after.clone();
    changed.push(10);
    std::fs::write(f.root.join(MAPPING_PATH), changed).unwrap();
    assert!(remaining
        .apply_once(
            OP,
            &payload,
            &file_objects_v2::supported_schema(),
            &scope,
            &expected,
            false
        )
        .is_err());
    std::fs::write(f.root.join(MAPPING_PATH), &map_after).unwrap();
    assert!(remaining
        .apply_once(
            OP,
            &payload,
            &file_objects_v2::supported_schema(),
            &scope,
            &expected,
            true
        )
        .is_err());
    remaining
        .apply_once(
            OP,
            &payload,
            &file_objects_v2::supported_schema(),
            &scope,
            &expected,
            false,
        )
        .unwrap();
    assert_eq!(
        remaining.progress(OP).unwrap(),
        Some(local::LocalProgressV2::Applied)
    );
    assert_eq!(
        std::fs::read(f.home.join("config.json")).unwrap(),
        model_config_after
    );
    assert_eq!(std::fs::read(f.root.join(MAPPING_PATH)).unwrap(), map_after);
    assert!(f
        .root
        .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH)
        .exists());
    assert!(discovery::skills_store_dir(&f.root)
        .join("fixture-skill/SKILL.md")
        .exists());
    assert_eq!(
        portable_assets::stored_public(&f.root).unwrap()[0].name,
        "fixture"
    );
    let before = std::fs::read(&journal_path).unwrap();
    remaining
        .apply_once(
            OP,
            &payload,
            &file_objects_v2::supported_schema(),
            &scope,
            &expected,
            false,
        )
        .unwrap();
    assert_eq!(std::fs::read(&journal_path).unwrap(), before);
    // Removal of the original journal is never permission to rebaseline O.
    std::fs::remove_file(&journal_path).unwrap();
    assert!(remaining
        .apply_once(
            OP,
            &payload,
            &file_objects_v2::supported_schema(),
            &scope,
            &expected,
            false
        )
        .is_err());
    assert!(!journal_path.exists());
}
