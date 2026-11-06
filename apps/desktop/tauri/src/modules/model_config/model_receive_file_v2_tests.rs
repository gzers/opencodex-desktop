//! Actual temporary D/H, v6 codec, compiler, backup and transaction engine.
use super::*;
use crate::modules::model_config::{
    file_objects_v2::{CapturedObjects, SecretValues},
    objects_overlay_v2::ChannelSelectionV2,
    portable_v2::{ModelPayloadV2, PortableConnectionV2, PortableModelsV2, PortablePolicyV2},
    schema::{Field, ModelNaming},
};
use tempfile::TempDir;
const OP: &str = "99999999-9999-4999-8999-999999999999";
const OP2: &str = "88888888-8888-4888-8888-888888888888";
const SOURCE: &str = "11111111-1111-4111-8111-111111111111";
const LOCAL: &str = "22222222-2222-4222-8222-222222222222";
const NODE: &str = "33333333-3333-4333-8333-333333333333";
const TEMPLATE: &str = "44444444-4444-4444-8444-444444444444";
const REVISION: &str = "55555555-5555-4555-8555-555555555555";
const SECRET: &str = "synthetic-receive-secret-never-in-journal";
struct Fixture {
    _temp: TempDir,
    host: ReceiveFileHostV2,
}
impl Fixture {
    fn new() -> Self {
        let t = tempfile::Builder::new()
            .prefix("ocxd023-model-receive-file-fixture-")
            .tempdir_in("/tmp")
            .unwrap();
        let root = t.path().join("D");
        let home = t.path().join("H");
        std::fs::create_dir_all(root.join("manager-state")).unwrap();
        std::fs::create_dir(&home).unwrap();
        std::fs::write(
            home.join("config.json"),
            b"{\"providers\":{},\"untouched\":{\"opaque\":true}}",
        )
        .unwrap();
        Self {
            _temp: t,
            host: ReceiveFileHostV2 {
                root,
                home,
                trusted_mapping_write: None,
                bindings: vec![ReceiveBindingV2 {
                    portable_channel_id: SOURCE.into(),
                    local_channel_id: LOCAL.into(),
                    provider_id: "received".into(),
                    mode: ReceiveBindingModeV2::New,
                }],
            },
        }
    }
    fn prepare(&self, id: &str, o: &SavedObjects, s: &SelectionV2) -> ModelApplyPlanHandle {
        self.host
            .prepare(id, o, s, &digest(b"outer selected target"))
            .unwrap_or_else(|e| panic!("prepare: {}", e.code))
    }
    fn apply(
        &self,
        id: &str,
        o: &SavedObjects,
        s: &SelectionV2,
        backup: bool,
    ) -> ModelApplyPlanHandle {
        let h = self.prepare(id, o, s);
        self.host
            .protect(id, &h, backup)
            .unwrap_or_else(|e| panic!("protect: {}", e.code));
        self.host
            .apply_once(id, o, &h, backup)
            .unwrap_or_else(|e| panic!("apply: {}", e.code));
        h
    }
    fn raw(&self) -> Value {
        read_raw(&self.host.home).unwrap().0
    }
    fn business(&self) -> Vec<Option<Vec<u8>>> {
        std::iter::once(self.host.home.join("config.json"))
            .chain(METADATA.iter().map(|p| self.host.root.join(p)))
            .map(|p| std::fs::read(p).ok())
            .collect()
    }
    fn existing(&mut self) {
        self.host.bindings[0].mode = ReceiveBindingModeV2::Existing;
    }
    fn saved(&self) -> StoreSnapshotV2 {
        StoreV2::new(&self.host.root).snapshot_saved().unwrap()
    }
}
fn selection() -> SelectionV2 {
    SelectionV2 {
        channels: vec![ChannelSelectionV2 {
            portable_channel_id: SOURCE.into(),
            model_ids: None,
            family_ids: None,
            include_connection: true,
            include_key: false,
            include_authorization: false,
        }],
        ..Default::default()
    }
}
fn channel(ids: &[&str]) -> PortableChannelV2 {
    PortableChannelV2 {
        id: SOURCE.into(),
        label: "Received channel".into(),
        connection: Some(PortableConnectionV2 {
            nodes: vec![ServiceNode {
                id: NODE.into(),
                name: "public node".into(),
                url: "https://receive.invalid/v1".into(),
            }],
            active_node_id: NODE.into(),
            enabled: true,
            allow_private_network: false,
            auth_mode: Some("key".into()),
            provider: Some(
                json!({"adapter":"openai-responses","baseUrl":"https://receive.invalid/v1","models":ids,"modelCapabilities":{},"note":"captured source"}),
            ),
        }),
        model_subset: None,
        family_scope: None,
        models: ids
            .iter()
            .map(|id| ModelConfigV2 {
                raw_model_id: (*id).into(),
                label: (*id).into(),
                observed: true,
                source: source_from_provider_v2(
                    "original-source",
                    id,
                    &json!({"adapter":"google","contextWindow":32000}),
                    &digest(b"original source bytes"),
                    SourceKindV2::UpstreamRead,
                )
                .unwrap(),
                explicit_overrides: ModelRules::default(),
                binding: None,
            })
            .collect(),
        template_refs: vec![],
        selection: ModelSelection::Whitelist(ids.iter().map(|s| (*s).into()).collect()),
        model_order: ids.iter().map(|s| (*s).into()).collect(),
        defaults: ModelRules::default(),
        family_overrides: BTreeMap::new(),
        naming: BTreeMap::new(),
        namespace: None,
    }
}
fn objects(
    c: PortableChannelV2,
    templates: Vec<TemplateV2>,
    key: bool,
    authorization: Option<PortableAuthorization>,
) -> SavedObjects {
    let mut c = c;
    let captured_public = if c.model_subset.is_some() {
        c.connection.as_mut().and_then(|c| c.provider.take())
    } else {
        None
    };
    let payload = ModelPayloadV2 {
        templates,
        policy: Some(PortablePolicyV2 {
            template_refs: vec![],
            channels: vec![c],
            channel_order: vec![SOURCE.into()],
            default_channel_id: None,
        }),
    };
    let models = PortableModelsV2 {
        format: "OCXD_MODEL_PACKAGE".into(),
        format_version: 6,
        schema_version: 2,
        created_at: "2026-11-06T12:00:00Z".into(),
        digest: digest(&canonical_bytes(&payload).unwrap()),
        payload,
    };
    let models = if let Some(public) = captured_public {
        crate::modules::model_config::portable_v2::attach_public_sources_v2(
            &models,
            &BTreeMap::from([(SOURCE.into(), public)]),
        )
        .unwrap()
    } else {
        models
    };
    let mut keys = SecretValues::new();
    if key {
        keys.insert(SOURCE.into(), Zeroizing::new(SECRET.into()))
            .unwrap();
    }
    let mut auth = BTreeMap::new();
    if let Some(a) = authorization {
        auth.insert(SOURCE.into(), a);
    }
    SavedObjects::capture(
        CapturedObjects {
            models,
            manager: Default::default(),
            assets: Default::default(),
            channel_keys: keys,
            channel_authorizations: auth,
            webdav_passwords: SecretValues::new(),
            content_versions: vec![],
        },
        Some(&ActualAuthorization),
    )
    .unwrap_or_else(|e| panic!("objects: {}", e.code))
}
fn simple(ids: &[&str]) -> SavedObjects {
    objects(channel(ids), vec![], false, None)
}
fn template() -> TemplateV2 {
    let mut t = seed_library_v2().unwrap().templates.remove(0);
    t.id = TEMPLATE.into();
    let mut r = t.revisions.remove(0);
    r.id = REVISION.into();
    r.base_revision = None;
    r.content.common_rules = ModelRules {
        protocol: Field::Custom("anthropic".into()),
        ..Default::default()
    };
    r.content.condition_overrides.clear();
    r.content.locked_fields.clear();
    r.digest = content_digest(&r.saved_label, &r.content).unwrap();
    t.default_revision = Some(r.id.clone());
    t.revisions = vec![r];
    t
}
fn reference(t: &TemplateV2) -> RefV2 {
    RefV2 {
        template_id: t.id.clone(),
        revision_id: t.revisions[0].id.clone(),
        digest: t.revisions[0].digest.clone(),
    }
}
fn code<T>(r: ModelResult<T>) -> String {
    match r {
        Err(e) => e.code,
        Ok(_) => panic!("expected error"),
    }
}
fn assert_applied(status: ModelResult<ModelApplyStatus>) {
    assert!(matches!(status.unwrap(), ModelApplyStatus::Applied(_)));
}
#[test]
fn receive_new_writes_real_config_library_policy_ledger_mappings_versions_and_atomic_receipt() {
    let f = Fixture::new();
    let t = template();
    let r = reference(&t);
    let mut c = channel(&["vendor/model", "second"]);
    c.models[0].binding = Some(ModelBindingV2::Manual {
        reference: r.clone(),
    });
    c.models[0].explicit_overrides.context_window = Field::Custom(64000);
    c.models[1].binding = Some(ModelBindingV2::AutoRead {
        reference: r.clone(),
    });
    c.models[1].explicit_overrides.protocol = Field::Custom("google".into());
    c.naming.insert(
        "vendor/model".into(),
        ModelNaming {
            alias: Some("exact-alias".into()),
            display_name: Some("display".into()),
        },
    );
    let o = objects(c, vec![t], true, None);
    let mut s = selection();
    s.channels[0].include_key = true;
    s.adopt_template_defaults = vec![r.clone()];
    let before = f.business();
    let h = f.prepare(OP, &o, &s);
    assert_eq!(f.business(), before);
    assert!(matches!(
        f.host.status(OP, &h).unwrap(),
        ModelApplyStatus::Prepared
    ));
    f.host.protect(OP, &h, true).unwrap();
    assert_eq!(f.business(), before);
    let receipt = f.host.apply_once(OP, &o, &h, true).unwrap();
    assert_eq!(receipt.operation_id, OP);
    assert!(receipt.plan == h);
    let raw = f.raw();
    assert_eq!(raw["untouched"]["opaque"], true);
    assert_eq!(raw["providers"]["received"]["apiKey"], SECRET);
    let state = f.saved();
    assert!(state.draft.is_none());
    assert_eq!(state.policy.channels[0].id, LOCAL);
    assert_eq!(state.policy.channels[0].connection.provider_id, "received");
    let t = state
        .library
        .templates
        .iter()
        .find(|t| t.id == TEMPLATE)
        .unwrap();
    assert_eq!(t.default_revision.as_deref(), Some(REVISION));
    let c = &state.policy.channels[0];
    assert_eq!(
        c.models[0].binding,
        o.models().payload.policy.as_ref().unwrap().channels[0].models[0].binding
    );
    for m in &c.models {
        for field in m.source.fields.values() {
            assert_eq!(field.evidence.kind, SourceKindV2::PortableImport);
            assert_eq!(field.evidence.provider_id, "original-source");
            assert_eq!(
                field.evidence.source_digest,
                digest(b"original source bytes")
            );
        }
    }
    let plan = plan_materialization_v2(&state.library, &state.policy, &state.policy).unwrap();
    assert_eq!(plan.groups.len(), 2);
    for group in &plan.groups {
        let map = protocol_groups::mapping_for_group_v2(&state.mappings, group).unwrap();
        assert_eq!(
            raw["providers"][&map.provider_id]["adapter"],
            group.protocol.as_str()
        );
    }
    assert_eq!(
        state.ledger.unwrap().written_file_digest,
        digest(&std::fs::read(f.host.home.join("config.json")).unwrap())
    );
    for p in METADATA {
        assert!(f.host.root.join(p).is_file());
    }
    assert!(f.host.dir(OP).unwrap().join("receipt.json").is_file());
    assert_applied(f.host.status(OP, &h));
}
#[test]
fn selected_partial_existing_preserves_unselected_raw_fields_models_credentials_and_order() {
    let mut f = Fixture::new();
    let initial = objects(channel(&["a", "keep", "b"]), vec![], true, None);
    let mut s = selection();
    s.channels[0].include_key = true;
    f.apply(OP, &initial, &s, false);
    f.existing();
    let mut raw = f.raw();
    raw["providers"]["received"]["unknownExtension"] = json!({"keep":true});
    raw["providers"]["received"]["modelCapabilities"] =
        json!({"a":{"unknownNested":true},"keep":{"unselected":true}});
    std::fs::write(
        f.host.home.join("config.json"),
        canonical_bytes(&raw).unwrap(),
    )
    .unwrap();
    let mut c = channel(&["a", "b"]);
    c.model_subset = Some(vec!["a".into(), "b".into()]);
    c.model_order = vec!["b".into(), "a".into()];
    c.selection = ModelSelection::Whitelist(vec!["b".into()]);
    c.connection.as_mut().unwrap().provider = Some(
        json!({"baseUrl":"https://receive.invalid/v1","adapter":"openai-chat","modelCapabilities":{"a":{"incomingNested":true}},"models":["a","b"],"liveModels":false,"newModelPolicy":"manual"}),
    );
    let o = objects(c, vec![], false, None);
    let mut s = selection();
    s.channels[0].model_ids = Some(vec!["a".into(), "b".into()]);
    s.channels[0].include_connection = false;
    let saved = f.saved();
    f.apply(OP2, &o, &s, false);
    let out = f.raw();
    assert_eq!(out["providers"]["received"]["apiKey"], SECRET);
    assert_eq!(
        out["providers"]["received"]["unknownExtension"],
        raw["providers"]["received"]["unknownExtension"]
    );
    assert_eq!(
        out["providers"]["received"]["modelCapabilities"]["keep"],
        raw["providers"]["received"]["modelCapabilities"]["keep"]
    );
    assert_eq!(
        out["providers"]["received"]["modelCapabilities"]["a"]["unknownNested"],
        true
    );
    assert_eq!(
        out["providers"]["received"]["modelCapabilities"]["a"]["incomingNested"],
        true
    );
    let state = f.saved();
    assert_eq!(
        state.policy.channels[0].models[1],
        saved.policy.channels[0].models[1]
    );
    assert_eq!(state.policy.channels[0].model_order, vec!["b", "keep", "a"]);
}
#[test]
fn key_replace_preserves_unselected_pool_slots() {
    let mut f = Fixture::new();
    let o = objects(channel(&["a"]), vec![], true, None);
    let mut s = selection();
    s.channels[0].include_key = true;
    f.apply(OP, &o, &s, false);
    f.existing();
    let mut raw = f.raw();
    raw["providers"]["received"]["apiKey"] = json!("old-active");
    raw["providers"]["received"]["apiKeyPool"] =
        json!([{"id":"old","key":"old-active"},{"id":"keep","key":"keep-pool-slot"}]);
    std::fs::write(
        f.host.home.join("config.json"),
        canonical_bytes(&raw).unwrap(),
    )
    .unwrap();
    f.apply(OP2, &o, &s, false);
    let raw = f.raw();
    assert_eq!(raw["providers"]["received"]["apiKeyPool"][0]["key"], SECRET);
    assert_eq!(
        raw["providers"]["received"]["apiKeyPool"][1]["key"],
        "keep-pool-slot"
    );
}
#[test]
fn portable_authorization_is_materialized_into_file_without_system_credentials() {
    let f = Fixture::new();
    let auth: PortableAuthorization = serde_json::from_value(
        json!({"activeId":null,"fields":{"headers":{"Authorization":SECRET},"apiKeyPool":[]}}),
    )
    .unwrap();
    let o = objects(channel(&["a"]), vec![], false, Some(auth));
    let mut s = selection();
    s.channels[0].include_authorization = true;
    let h = f.apply(OP, &o, &s, false);
    assert_eq!(
        f.raw()["providers"]["received"]["headers"]["Authorization"],
        SECRET
    );
    assert_applied(f.host.status(OP, &h));
}
#[test]
fn backup_failure_has_zero_business_writes_and_cannot_claim_protection() {
    let mut f = Fixture::new();
    let o = simple(&["a"]);
    f.apply(OP, &o, &selection(), false);
    f.existing();
    let h = f.prepare(OP2, &o, &selection());
    let before = f.business();
    std::fs::write(f.host.root.join("backups"), b"blocked").unwrap();
    assert!(f.host.protect(OP2, &h, true).is_err());
    assert_eq!(f.business(), before);
    assert_eq!(
        code(f.host.apply_once(OP2, &o, &h, true)),
        "model_receive_protection_required"
    );
    assert_eq!(f.business(), before);
}
#[test]
fn protect_covers_all_existing_images_and_rechecks_backup_bytes() {
    let mut f = Fixture::new();
    let o = simple(&["a"]);
    f.apply(OP, &o, &selection(), false);
    f.existing();
    let h = f.prepare(OP2, &o, &selection());
    let before = f.business();
    f.host.protect(OP2, &h, true).unwrap();
    assert_eq!(f.business(), before);
    let bytes = f
        .host
        .private_read(
            &f.host.dir(OP2).unwrap().join("protection.json"),
            MAX_MANIFEST,
        )
        .unwrap()
        .unwrap();
    let p: Protection = decoded(&bytes).unwrap();
    assert_eq!(p.proofs.len(), 6);
    std::fs::write(f.host.root.join(&p.proofs[0].payload), b"corrupt backup").unwrap();
    assert_eq!(
        code(f.host.apply_once(OP2, &o, &h, true)),
        "model_receive_backup_invalid"
    );
    assert_eq!(f.business(), before);
}
#[test]
fn external_config_or_each_metadata_edit_after_freeze_is_rejected() {
    for changed in 0..6 {
        let mut f = Fixture::new();
        let o = simple(&["a"]);
        f.apply(OP, &o, &selection(), false);
        f.existing();
        let h = f.prepare(OP2, &o, &selection());
        f.host.protect(OP2, &h, false).unwrap();
        let path = if changed == 0 {
            f.host.home.join("config.json")
        } else {
            f.host.root.join(METADATA[changed - 1])
        };
        std::fs::write(&path, b"external-value").unwrap();
        let before = f.business();
        assert_eq!(
            code(f.host.apply_once(OP2, &o, &h, false)),
            "model_receive_stale"
        );
        assert_eq!(f.business(), before);
    }
}
#[test]
fn applied_is_historical_and_same_id_never_overwrites_later_user_changes() {
    let f = Fixture::new();
    let o = simple(&["a"]);
    let h = f.apply(OP, &o, &selection(), false);
    std::fs::write(f.host.home.join("config.json"), b"{\"userChanged\":true}").unwrap();
    let before = f.business();
    assert_applied(f.host.status(OP, &h));
    f.host.apply_once(OP, &o, &h, false).unwrap();
    assert_eq!(f.business(), before);
    let reopened = f.prepare(OP, &o, &selection());
    assert!(reopened == h);
    assert_eq!(f.business(), before);
}
#[test]
fn same_id_payload_scope_target_bindings_or_handle_change_cannot_rebuild() {
    let mut f = Fixture::new();
    let o = simple(&["a"]);
    let s = selection();
    let h = f.prepare(OP, &o, &s);
    let before = f.business();
    assert_eq!(
        code(f.host.prepare(
            OP,
            &simple(&["different"]),
            &s,
            &digest(b"outer selected target")
        )),
        "model_receive_operation_collision"
    );
    let mut changed = s.clone();
    changed.channels[0].include_connection = false;
    assert_eq!(
        code(
            f.host
                .prepare(OP, &o, &changed, &digest(b"outer selected target"))
        ),
        "model_receive_operation_collision"
    );
    assert_eq!(
        code(f.host.prepare(OP, &o, &s, &digest(b"other target"))),
        "model_receive_operation_collision"
    );
    let wrong = ModelApplyPlanHandle::new(digest(b"wrong handle")).unwrap();
    assert_eq!(
        code(f.host.protect(OP, &wrong, false)),
        "model_receive_handle_mismatch"
    );
    assert_eq!(
        code(f.host.apply_once(OP, &o, &wrong, false)),
        "model_receive_handle_mismatch"
    );
    f.host.bindings[0].provider_id = "different".into();
    assert_eq!(
        code(
            f.host
                .prepare(OP, &o, &s, &digest(b"outer selected target"))
        ),
        "model_receive_plan_binding"
    );
    assert_eq!(f.business(), before);
    assert_ne!(h.token(), wrong.token());
}
#[test]
fn frozen_image_corruption_and_plan_schema_checksum_fail_closed() {
    let f = Fixture::new();
    let o = simple(&["a"]);
    let h = f.prepare(OP, &o, &selection());
    let before = f.business();
    let dir = f.host.dir(OP).unwrap();
    std::fs::write(dir.join("0-after.bin"), b"corrupt").unwrap();
    assert_eq!(
        code(f.host.protect(OP, &h, false)),
        "model_receive_image_corrupt"
    );
    assert_eq!(f.business(), before);
    let bytes = std::fs::read(dir.join("plan.json")).unwrap();
    let mut plan: Frozen = decoded(&bytes).unwrap();
    plan.schema_version = 99;
    std::fs::write(dir.join("plan.json"), checked(&plan).unwrap()).unwrap();
    assert_eq!(code(f.host.status(OP, &h)), "model_schema_unsupported");
    std::fs::write(
        dir.join("plan.json"),
        b"{\"value\":{},\"checksum\":\"bad\"}",
    )
    .unwrap();
    assert!(f.host.status(OP, &h).is_err());
}
#[test]
fn draft_existence_is_protected_without_reading_or_clearing_it() {
    let f = Fixture::new();
    let o = simple(&["a"]);
    let draft = f.host.root.join(store_v2::DRAFT_PATH);
    std::fs::write(&draft, b"unknown unsaved draft schema").unwrap();
    let before = f.business();
    assert_eq!(
        code(f.host.prepare(OP, &o, &selection(), &digest(b"target"))),
        "model_receive_unsaved_draft"
    );
    assert_eq!(
        std::fs::read(draft).unwrap(),
        b"unknown unsaved draft schema"
    );
    assert_eq!(f.business(), before);
}
#[test]
fn namespace_and_credential_direction_errors_leave_originals_untouched() {
    let mut f = Fixture::new();
    let mut c = channel(&["a"]);
    c.namespace = Some(NamespaceV2 {
        channel_id: SOURCE.into(),
        remove_leading_prefixes: vec!["prefix/".into()],
    });
    let o = objects(c, vec![], false, None);
    let before = f.business();
    assert_eq!(
        code(f.host.prepare(OP, &o, &selection(), &digest(b"target"))),
        "model_receive_namespace_adoption_required"
    );
    assert_eq!(f.business(), before);
    let o = simple(&["a"]);
    f.apply(OP, &o, &selection(), false);
    f.existing();
    let mut c = channel(&["a"]);
    c.connection.as_mut().unwrap().nodes[0].url = "https://changed.invalid/v1".into();
    c.connection.as_mut().unwrap().provider.as_mut().unwrap()["baseUrl"] =
        json!("https://changed.invalid/v1");
    let o = objects(c, vec![], false, None);
    let before = f.business();
    assert_eq!(
        code(f.host.prepare(OP2, &o, &selection(), &digest(b"target"))),
        "model_receive_credentials_direction"
    );
    assert_eq!(f.business(), before);
}
#[test]
fn secrets_are_only_in_private_images_and_actual_files_not_manifests_receipts_or_dtos() {
    let f = Fixture::new();
    let o = objects(channel(&["a"]), vec![], true, None);
    let mut s = selection();
    s.channels[0].include_key = true;
    let h = f.apply(OP, &o, &s, true);
    let dir = f.host.dir(OP).unwrap();
    for name in ["plan.json", "protection.json", "receipt.json"] {
        assert!(!String::from_utf8_lossy(&std::fs::read(dir.join(name)).unwrap()).contains(SECRET));
    }
    assert!(!serde_json::to_string(&h).unwrap().contains(SECRET));
    for entry in std::fs::read_dir(f.host.root.join("manager-state/model-transactions-v2")).unwrap()
    {
        let p = entry.unwrap().path().join("journal.json");
        assert!(!String::from_utf8_lossy(&std::fs::read(p).unwrap()).contains(SECRET));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(dir.join("0-after.bin"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
}
#[test]
fn real_transaction_pending_resumes_same_frozen_images_and_commits_receipt_last() {
    let f = Fixture::new();
    let o = simple(&["a"]);
    let h = f.apply(OP, &o, &selection(), false);
    let applied = f.business();
    let dir = f.host.dir(OP).unwrap();
    let transaction = std::fs::read_dir(f.host.root.join("manager-state/model-transactions-v2"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let journal_path = transaction.join("journal.json");
    let mut journal: Value =
        serde_json::from_slice(&std::fs::read(&journal_path).unwrap()).unwrap();
    let plan = journal["value"]["plan"].clone();
    // Reconstruct the real engine's crash boundary: config written, metadata
    // still before, pending marker published, receipt has not been committed.
    let frozen = f.host.load(OP, &h).unwrap();
    for (i, e) in frozen.entries.iter().enumerate().skip(1) {
        let (_, path) = f.host.target_path(&e.target).unwrap();
        if let Some(b) = &e.before {
            let bytes = f.host.image(OP, i, true, b).unwrap();
            std::fs::write(path, &bytes).unwrap();
        } else {
            std::fs::remove_file(path).unwrap();
        }
    }
    std::fs::remove_file(dir.join("receipt.json")).unwrap();
    journal["value"]["phase"] = json!("applying");
    journal["checksum"] = json!(digest(&canonical_bytes(&journal["value"]).unwrap()));
    std::fs::write(&journal_path, canonical_bytes(&journal).unwrap()).unwrap();
    let marker = json!({"version":2,"id":plan["id"],"binding":plan["binding"],"planDigest":digest(&canonical_bytes(&plan).unwrap())});
    let marker = json!({"checksum":digest(&canonical_bytes(&marker).unwrap()),"value":marker});
    std::fs::write(
        f.host
            .root
            .join("manager-state/model-transaction-v2.pending.json"),
        canonical_bytes(&marker).unwrap(),
    )
    .unwrap();
    restrict(
        &f.host
            .root
            .join("manager-state/model-transaction-v2.pending.json"),
        false,
    )
    .unwrap();
    f.host
        .apply_once(OP, &o, &h, false)
        .unwrap_or_else(|e| panic!("recover: {}", e.code));
    assert_eq!(f.business(), applied);
    assert_applied(f.host.status(OP, &h));
    assert!(!f
        .host
        .root
        .join("manager-state/model-transaction-v2.pending.json")
        .exists());
}
#[test]
fn applied_id_still_rejects_different_payload_or_handle() {
    let f = Fixture::new();
    let o = simple(&["a"]);
    let h = f.apply(OP, &o, &selection(), false);
    let before = f.business();
    assert_eq!(
        code(f.host.apply_once(OP, &simple(&["different"]), &h, false)),
        "model_receive_operation_collision"
    );
    let wrong = ModelApplyPlanHandle::new(digest(b"wrong")).unwrap();
    assert!(f.host.status(OP, &wrong).is_err());
    assert_eq!(f.business(), before);
}

#[test]
fn first_device_identity_can_use_the_same_portable_uuid_without_allocation() {
    let mut f = Fixture::new();
    f.host.bindings[0].local_channel_id = SOURCE.into();
    let o = simple(&["a"]);
    f.apply(OP, &o, &selection(), false);
    assert_eq!(f.saved().policy.channels[0].id, SOURCE);
}
#[test]
fn apply_requires_explicit_protection_even_when_backup_is_disabled() {
    let f = Fixture::new();
    let o = simple(&["a"]);
    let h = f.prepare(OP, &o, &selection());
    let before = f.business();
    assert_eq!(
        code(f.host.apply_once(OP, &o, &h, false)),
        "model_receive_protection_required"
    );
    assert_eq!(f.business(), before);
    f.host.protect(OP, &h, false).unwrap();
    let p: Protection =
        decoded(&std::fs::read(f.host.dir(OP).unwrap().join("protection.json")).unwrap()).unwrap();
    assert!(!p.backup);
    assert!(p.proofs.is_empty());
    assert_eq!(
        code(f.host.apply_once(OP, &o, &h, true)),
        "model_receive_protection_mismatch"
    );
    assert_eq!(f.business(), before);
}
#[test]
fn selected_missing_key_or_authorization_fails_before_freezing_business_images() {
    for authorization in [false, true] {
        let f = Fixture::new();
        let o = simple(&["a"]);
        let mut s = selection();
        s.channels[0].include_key = !authorization;
        s.channels[0].include_authorization = authorization;
        let before = f.business();
        assert!(f.host.prepare(OP, &o, &s, &digest(b"target")).is_err());
        assert_eq!(f.business(), before);
        assert!(!f.host.dir(OP).unwrap().join("plan.json").exists());
    }
}
#[test]
fn unbound_actual_provider_is_not_assigned_a_guessed_identity_or_overwritten() {
    let f = Fixture::new();
    std::fs::write(f.host.home.join("config.json"),canonical_bytes(&json!({"providers":{"unbound":{"adapter":"google","baseUrl":"https://other.invalid","models":["a"]}}})).unwrap()).unwrap();
    let before = f.business();
    assert_eq!(
        code(
            f.host
                .prepare(OP, &simple(&["a"]), &selection(), &digest(b"target"))
        ),
        "model_receive_unbound_source"
    );
    assert_eq!(f.business(), before);
    assert!(!f.host.root.join(IDENTITY).exists());
}
#[test]
fn identity_registry_change_after_prepare_is_also_a_cas_conflict() {
    let f = Fixture::new();
    let o = simple(&["a"]);
    let h = f.prepare(OP, &o, &selection());
    f.host.protect(OP, &h, false).unwrap();
    std::fs::write(f.host.root.join(IDENTITY), b"external identity registry").unwrap();
    let before = f.business();
    assert_eq!(
        code(f.host.apply_once(OP, &o, &h, false)),
        "model_receive_stale"
    );
    assert_eq!(f.business(), before);
}
#[test]
fn draft_created_after_freeze_cannot_be_cleared_by_apply() {
    let f = Fixture::new();
    let o = simple(&["a"]);
    let h = f.prepare(OP, &o, &selection());
    f.host.protect(OP, &h, false).unwrap();
    std::fs::write(f.host.root.join(store_v2::DRAFT_PATH), b"unsaved draft").unwrap();
    let before = f.business();
    assert_eq!(
        code(f.host.apply_once(OP, &o, &h, false)),
        "model_receive_unsaved_draft"
    );
    assert_eq!(f.business(), before);
}
#[test]
fn real_v6_decoder_rejects_unknown_schema_and_missing_exact_dependencies() {
    let f = Fixture::new();
    let o = simple(&["a"]);
    let bytes = file_objects_v2::encode_document(&o, None).unwrap();
    let original: Value = serde_json::from_slice(&bytes).unwrap();
    let before = f.business();
    let mut v = original.clone();
    v["models"]["schemaVersion"] = json!(99);
    assert!(file_objects_v2::decode_document(&canonical_bytes(&v).unwrap(), None).is_err());
    let mut v = original;
    v["models"]["payload"]["policy"]["channels"][0]["models"][0]["binding"] = json!({"kind":"manual","reference":{"templateId":TEMPLATE,"revisionId":REVISION,"digest":digest(b"missing")}});
    v["models"]["digest"] = json!(digest(&canonical_bytes(&v["models"]["payload"]).unwrap()));
    assert!(file_objects_v2::decode_document(&canonical_bytes(&v).unwrap(), None).is_err());
    assert_eq!(f.business(), before);
}
#[test]
fn family_fragment_updates_existing_rules_without_new_connection_or_credentials() {
    let mut f = Fixture::new();
    let o = objects(channel(&["a", "keep"]), vec![], true, None);
    let mut s = selection();
    s.channels[0].include_key = true;
    f.apply(OP, &o, &s, false);
    f.existing();
    let before = f.raw();
    let t = template();
    let mut c = channel(&["a"]);
    c.connection = None;
    c.family_scope = Some(vec![t.id.clone()]);
    c.models[0].binding = Some(ModelBindingV2::AutoRead {
        reference: reference(&t),
    });
    c.family_overrides.insert(
        t.id.clone(),
        ModelRules {
            context_window: Field::Custom(64000),
            ..Default::default()
        },
    );
    let o = objects(c, vec![t.clone()], false, None);
    let mut s = selection();
    s.channels[0].family_ids = Some(vec![t.id]);
    s.channels[0].include_connection = false;
    f.apply(OP2, &o, &s, false);
    let out = f.raw();
    assert_eq!(
        out["providers"]["received"]["baseUrl"],
        before["providers"]["received"]["baseUrl"]
    );
    assert_eq!(out["providers"]["received"]["apiKey"], SECRET);
    assert!(f.saved().policy.channels[0]
        .models
        .iter()
        .any(|m| m.raw_model_id == "keep"));
}
#[test]
fn family_fragment_cannot_manufacture_a_new_file_connection() {
    let f = Fixture::new();
    let t = template();
    let mut c = channel(&["a"]);
    c.connection = None;
    c.family_scope = Some(vec![t.id.clone()]);
    c.models[0].binding = Some(ModelBindingV2::Manual {
        reference: reference(&t),
    });
    let o = objects(c, vec![t.clone()], false, None);
    let mut s = selection();
    s.channels[0].family_ids = Some(vec![t.id]);
    s.channels[0].include_connection = false;
    let before = f.business();
    assert_eq!(
        code(f.host.prepare(OP, &o, &s, &digest(b"target"))),
        "model_receive_family_target_required"
    );
    assert_eq!(f.business(), before);
}
#[test]
fn unselected_owned_keychain_reference_is_not_read_or_copied_to_a_new_group() {
    let mut f = Fixture::new();
    let o = simple(&["a", "keep"]);
    f.apply(OP, &o, &selection(), false);
    f.existing();
    let mut raw = f.raw();
    raw["providers"]["received"]["apiKey"] = json!("keychain:received");
    std::fs::write(
        f.host.home.join("config.json"),
        canonical_bytes(&raw).unwrap(),
    )
    .unwrap();
    let mut c = channel(&["a"]);
    c.models[0].explicit_overrides.protocol = Field::Custom("anthropic".into());
    let o = objects(c, vec![], false, None);
    let mut s = selection();
    s.channels[0].model_ids = Some(vec!["a".into()]);
    s.channels[0].include_connection = false;
    let before = f.business();
    assert_eq!(
        code(f.host.prepare(OP2, &o, &s, &digest(b"target"))),
        "model_receive_reference_moves_required"
    );
    assert_eq!(f.business(), before);
}
#[test]
fn new_endpoint_with_selected_key_cannot_carry_old_unselected_header_or_pool_secrets() {
    for pool in [false, true] {
        let mut f = Fixture::new();
        let o = objects(channel(&["a"]), vec![], true, None);
        let mut s = selection();
        s.channels[0].include_key = true;
        f.apply(OP, &o, &s, false);
        f.existing();
        let mut raw = f.raw();
        if pool {
            raw["providers"]["received"]["apiKeyPool"] =
                json!([{"id":"unselected","key":"old endpoint slot"}]);
        } else {
            raw["providers"]["received"]["headers"] =
                json!({"Authorization":"old endpoint header"});
        }
        std::fs::write(
            f.host.home.join("config.json"),
            canonical_bytes(&raw).unwrap(),
        )
        .unwrap();
        let mut c = channel(&["a"]);
        let conn = c.connection.as_mut().unwrap();
        conn.nodes[0].url = "https://changed.invalid/v1".into();
        conn.provider.as_mut().unwrap()["baseUrl"] = json!("https://changed.invalid/v1");
        let o = objects(c, vec![], true, None);
        let before = f.business();
        assert_eq!(
            code(f.host.prepare(OP2, &o, &s, &digest(b"target"))),
            "model_receive_credentials_direction"
        );
        assert_eq!(f.business(), before);
    }
}
#[test]
fn explicit_none_selection_is_materialized_without_enabling_any_model() {
    let f = Fixture::new();
    let mut c = channel(&["a"]);
    c.selection = ModelSelection::None;
    let o = objects(c, vec![], false, None);
    f.apply(OP, &o, &selection(), false);
    let raw = f.raw();
    assert_eq!(raw["providers"]["received"]["selectedModels"], json!([]));
    assert_eq!(raw["providers"]["received"]["liveModels"], false);
    assert_eq!(f.saved().policy.channels[0].selection, ModelSelection::None);
}
#[test]
fn absent_plan_unknown_status_and_wrong_operation_id_have_no_business_effect() {
    let f = Fixture::new();
    let h = ModelApplyPlanHandle::new(digest(b"unknown")).unwrap();
    let before = f.business();
    assert!(matches!(
        f.host.status(OP, &h).unwrap(),
        ModelApplyStatus::Unknown
    ));
    assert!(f.host.status("../bad", &h).is_err());
    assert_eq!(f.business(), before);
}

fn mapping_row(endpoint: &str, portable: &str, local: &str, op: &str) -> Value {
    json!({"endpointBindingId":digest(endpoint.as_bytes()),"portableId":portable,"localId":local,"adoptedOperationId":op})
}
fn mapping_bytes(rows: Vec<Value>) -> Vec<u8> {
    canonical_bytes(&json!({"schemaVersion":2,"bindings":rows})).unwrap()
}
#[test]
fn identity_mapping_is_frozen_and_committed_as_the_eighth_transaction_target() {
    let mut f = Fixture::new();
    let after = mapping_bytes(vec![mapping_row("endpoint", SOURCE, LOCAL, OP)]);
    f.host.trusted_mapping_write = Some(IdentityMappingWriteV2 {
        before: None,
        after: after.clone(),
    });
    let o = simple(&["a"]);
    let h = f.prepare(OP, &o, &selection());
    assert!(!f.host.root.join(SYNC_IDENTITIES).exists());
    let frozen = f.host.load(OP, &h).unwrap();
    assert_eq!(frozen.entries.len(), 7);
    assert_eq!(f.host.writes(OP, &frozen, &h).unwrap().len(), 8);
    f.host.protect(OP, &h, true).unwrap();
    f.host.apply_once(OP, &o, &h, true).unwrap();
    assert_eq!(
        std::fs::read(f.host.root.join(SYNC_IDENTITIES)).unwrap(),
        after
    );
    assert_applied(f.host.status(OP, &h));
}
#[test]
fn mapping_write_must_match_current_bindings_operation_and_strict_schema() {
    for after in [mapping_bytes(vec![mapping_row("endpoint",SOURCE,LOCAL,OP2)]),mapping_bytes(vec![mapping_row("endpoint",SOURCE,NODE,OP)]),canonical_bytes(&json!({"schemaVersion":99,"bindings":[]})).unwrap(),canonical_bytes(&json!({"schemaVersion":2,"bindings":[],"extra":true})).unwrap(),b"{\"schemaVersion\":2,\"schemaVersion\":2,\"bindings\":[]}".to_vec(),canonical_bytes(&json!({"schemaVersion":2,"bindings":[{"endpointBindingId":"bad","portableId":SOURCE,"localId":LOCAL,"adoptedOperationId":OP}]})).unwrap()] {
        let mut f=Fixture::new();f.host.trusted_mapping_write=Some(IdentityMappingWriteV2 { before:None,after });let before=f.business();assert!(f.host.prepare(OP,&simple(&["a"]),&selection(),&digest(b"target")).is_err());assert_eq!(f.business(),before);assert!(!f.host.root.join(SYNC_IDENTITIES).exists());
    }
}
#[test]
fn mapping_cannot_drop_or_overwrite_other_endpoint_identities() {
    let before = mapping_bytes(vec![mapping_row("other endpoint", NODE, REVISION, OP2)]);
    for rows in [
        vec![],
        vec![mapping_row("other endpoint", NODE, LOCAL, OP)],
        vec![mapping_row("other endpoint", NODE, REVISION, OP)],
    ] {
        let mut f = Fixture::new();
        std::fs::write(f.host.root.join(SYNC_IDENTITIES), &before).unwrap();
        f.host.trusted_mapping_write = Some(IdentityMappingWriteV2 {
            before: Some(before.clone()),
            after: mapping_bytes(rows),
        });
        let original = f.business();
        assert_eq!(
            code(
                f.host
                    .prepare(OP, &simple(&["a"]), &selection(), &digest(b"target"))
            ),
            "model_receive_identity_overwrite"
        );
        assert_eq!(f.business(), original);
        assert_eq!(
            std::fs::read(f.host.root.join(SYNC_IDENTITIES)).unwrap(),
            before
        );
    }
}
#[test]
fn mapping_preserves_old_pairs_and_backups_existing_identity_metadata() {
    let mut f = Fixture::new();
    let old = mapping_row("old endpoint", NODE, REVISION, OP2);
    let before = mapping_bytes(vec![old.clone()]);
    std::fs::write(f.host.root.join(SYNC_IDENTITIES), &before).unwrap();
    let after = mapping_bytes(vec![old, mapping_row("new endpoint", SOURCE, LOCAL, OP)]);
    f.host.trusted_mapping_write = Some(IdentityMappingWriteV2 {
        before: Some(before.clone()),
        after: after.clone(),
    });
    let o = simple(&["a"]);
    let h = f.prepare(OP, &o, &selection());
    f.host.protect(OP, &h, true).unwrap();
    let p: Protection =
        decoded(&std::fs::read(f.host.dir(OP).unwrap().join("protection.json")).unwrap()).unwrap();
    let proof = p.proofs.iter().find(|p| p.target_index == 6).unwrap();
    assert_eq!(
        std::fs::read(f.host.root.join(&proof.payload)).unwrap(),
        before
    );
    f.host.apply_once(OP, &o, &h, true).unwrap();
    assert_eq!(
        std::fs::read(f.host.root.join(SYNC_IDENTITIES)).unwrap(),
        after
    );
}
#[test]
fn mapping_material_changes_invalidate_handle_and_external_mapping_edit_fails_cas() {
    let mut f = Fixture::new();
    let after = mapping_bytes(vec![mapping_row("endpoint", SOURCE, LOCAL, OP)]);
    f.host.trusted_mapping_write = Some(IdentityMappingWriteV2 {
        before: None,
        after: after.clone(),
    });
    let o = simple(&["a"]);
    let h = f.prepare(OP, &o, &selection());
    f.host.protect(OP, &h, false).unwrap();
    std::fs::write(f.host.root.join(SYNC_IDENTITIES), b"external mapping").unwrap();
    let before = f.business();
    assert_eq!(
        code(f.host.apply_once(OP, &o, &h, false)),
        "model_receive_stale"
    );
    assert_eq!(f.business(), before);
    assert_eq!(
        std::fs::read(f.host.root.join(SYNC_IDENTITIES)).unwrap(),
        b"external mapping"
    );
    f.host.trusted_mapping_write.as_mut().unwrap().after =
        mapping_bytes(vec![mapping_row("different endpoint", SOURCE, LOCAL, OP)]);
    assert_eq!(
        code(f.host.protect(OP, &h, false)),
        "model_receive_plan_binding"
    );
}
#[test]
fn no_mapping_change_does_not_touch_existing_identity_metadata() {
    let f = Fixture::new();
    let before = mapping_bytes(vec![mapping_row("unrelated", NODE, REVISION, OP2)]);
    std::fs::write(f.host.root.join(SYNC_IDENTITIES), &before).unwrap();
    let o = simple(&["a"]);
    f.apply(OP, &o, &selection(), false);
    assert_eq!(
        std::fs::read(f.host.root.join(SYNC_IDENTITIES)).unwrap(),
        before
    );
}

#[test]
fn unselected_provider_object_remains_exact_after_complete_compiler_validation() {
    let mut f = Fixture::new();
    let o = simple(&["a"]);
    f.apply(OP, &o, &selection(), false);
    f.host.bindings[0].local_channel_id = NODE.into();
    f.host.bindings[0].provider_id = "other-provider".into();
    let mut c = channel(&["untouched-model"]);
    c.label = "Other channel".into();
    let o = objects(c, vec![], false, None);
    f.apply(OP2, &o, &selection(), false);
    let mut raw = f.raw();
    raw["providers"]["other-provider"]["unknownExtension"] = json!({"opaque":"retain exactly"});
    raw["providers"]["other-provider"]
        .as_object_mut()
        .unwrap()
        .remove("liveModels");
    std::fs::write(
        f.host.home.join("config.json"),
        canonical_bytes(&raw).unwrap(),
    )
    .unwrap();
    let original = raw["providers"]["other-provider"].clone();
    f.host.bindings[0].local_channel_id = LOCAL.into();
    f.host.bindings[0].provider_id = "received".into();
    f.existing();
    let o = simple(&["a"]);
    let mut s = selection();
    s.channels[0].include_connection = false;
    f.apply("77777777-7777-4777-8777-777777777777", &o, &s, false);
    assert_eq!(f.raw()["providers"]["other-provider"], original);
}
#[test]
fn copied_receipt_from_another_root_cannot_claim_local_application() {
    let a = Fixture::new();
    let o = simple(&["a"]);
    let h = a.apply(OP, &o, &selection(), false);
    let b = Fixture::new();
    let dir = b.host.dir(OP).unwrap();
    std::fs::create_dir_all(&dir).unwrap();
    restrict(&dir, true).unwrap();
    publish(
        &b.host.root,
        &dir.join("receipt.json"),
        &std::fs::read(a.host.dir(OP).unwrap().join("receipt.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        code(b.host.status(OP, &h)),
        "model_receive_receipt_mismatch"
    );
}
#[test]
fn eight_target_transaction_recovery_finishes_mapping_and_receipt_together() {
    let mut f = Fixture::new();
    let after = mapping_bytes(vec![mapping_row("endpoint", SOURCE, LOCAL, OP)]);
    f.host.trusted_mapping_write = Some(IdentityMappingWriteV2 {
        before: None,
        after: after.clone(),
    });
    let o = simple(&["a"]);
    let h = f.apply(OP, &o, &selection(), false);
    let transaction = std::fs::read_dir(f.host.root.join("manager-state/model-transactions-v2"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let path = transaction.join("journal.json");
    let mut journal: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let plan = journal["value"]["plan"].clone();
    assert_eq!(plan["entries"].as_array().unwrap().len(), 8);
    // Crash after the first six business files, before identity adoption/receipt.
    std::fs::remove_file(f.host.root.join(SYNC_IDENTITIES)).unwrap();
    std::fs::remove_file(f.host.dir(OP).unwrap().join("receipt.json")).unwrap();
    journal["value"]["phase"] = json!("applying");
    journal["checksum"] = json!(digest(&canonical_bytes(&journal["value"]).unwrap()));
    std::fs::write(path, canonical_bytes(&journal).unwrap()).unwrap();
    let marker = json!({"version":2,"id":plan["id"],"binding":plan["binding"],"planDigest":digest(&canonical_bytes(&plan).unwrap())});
    let marker = json!({"checksum":digest(&canonical_bytes(&marker).unwrap()),"value":marker});
    let pending = f
        .host
        .root
        .join("manager-state/model-transaction-v2.pending.json");
    std::fs::write(&pending, canonical_bytes(&marker).unwrap()).unwrap();
    restrict(&pending, false).unwrap();
    f.host.apply_once(OP, &o, &h, false).unwrap();
    assert_eq!(
        std::fs::read(f.host.root.join(SYNC_IDENTITIES)).unwrap(),
        after
    );
    assert_applied(f.host.status(OP, &h));
}
#[test]
fn duplicate_endpoint_source_or_target_mapping_is_rejected() {
    for rows in [
        vec![
            mapping_row("endpoint", SOURCE, LOCAL, OP),
            mapping_row("endpoint", SOURCE, NODE, OP),
        ],
        vec![
            mapping_row("endpoint", SOURCE, LOCAL, OP),
            mapping_row("endpoint", NODE, LOCAL, OP),
        ],
    ] {
        let mut f = Fixture::new();
        f.host.trusted_mapping_write = Some(IdentityMappingWriteV2 {
            before: None,
            after: mapping_bytes(rows),
        });
        assert_eq!(
            code(
                f.host
                    .prepare(OP, &simple(&["a"]), &selection(), &digest(b"target"))
            ),
            "model_receive_identity_invalid"
        );
    }
}
#[test]
fn pending_recovery_refuses_unknown_external_target_value_without_overwrite() {
    let f = Fixture::new();
    let o = simple(&["a"]);
    let h = f.apply(OP, &o, &selection(), false);
    let transaction = std::fs::read_dir(f.host.root.join("manager-state/model-transactions-v2"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let path = transaction.join("journal.json");
    let mut journal: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let plan = journal["value"]["plan"].clone();
    std::fs::remove_file(f.host.dir(OP).unwrap().join("receipt.json")).unwrap();
    journal["value"]["phase"] = json!("applying");
    journal["checksum"] = json!(digest(&canonical_bytes(&journal["value"]).unwrap()));
    std::fs::write(path, canonical_bytes(&journal).unwrap()).unwrap();
    let marker = json!({"version":2,"id":plan["id"],"binding":plan["binding"],"planDigest":digest(&canonical_bytes(&plan).unwrap())});
    let marker = json!({"checksum":digest(&canonical_bytes(&marker).unwrap()),"value":marker});
    let pending = f
        .host
        .root
        .join("manager-state/model-transaction-v2.pending.json");
    std::fs::write(&pending, canonical_bytes(&marker).unwrap()).unwrap();
    restrict(&pending, false).unwrap();
    std::fs::write(
        f.host.home.join("config.json"),
        b"{\"external\":\"unknown value\"}",
    )
    .unwrap();
    let before = f.business();
    assert!(f.host.apply_once(OP, &o, &h, false).is_err());
    assert_eq!(f.business(), before);
    assert!(pending.exists());
}

#[test]
fn status_and_applied_replay_do_not_enter_an_unrelated_pending_save_gate() {
    let f = Fixture::new();
    let o = simple(&["a"]);
    let s = selection();
    let h = f.apply(OP, &o, &s, false);
    let pending = f
        .host
        .root
        .join("manager-state/saved-file-save-pending.json");
    std::fs::write(
        &pending,
        b"unrelated pending save must not be recovered by status",
    )
    .unwrap();
    std::fs::write(
        f.host.home.join("config.json"),
        b"{\"laterUserValue\":true}",
    )
    .unwrap();
    let before = f.business();
    assert_applied(f.host.status(OP, &h));
    f.host.protect(OP, &h, false).unwrap();
    f.host.apply_once(OP, &o, &h, false).unwrap();
    assert!(f.prepare(OP, &o, &s) == h);
    assert_eq!(f.business(), before);
    assert_eq!(
        std::fs::read(&pending).unwrap(),
        b"unrelated pending save must not be recovered by status"
    );
}

#[test]
fn namespace_file_receiver_freezes_adoption_and_applies_compiler_and_receipt_in_one_transaction() {
    use crate::modules::model_config::file_transfer_v2::NamespaceReceiveV2;
    let f = Fixture::new();
    let mut c = channel(&["vendor/model"]);
    c.namespace = Some(NamespaceV2 {
        channel_id: SOURCE.into(),
        remove_leading_prefixes: vec!["vendor/".into()],
    });
    c.models[0].explicit_overrides.context_window = Field::Custom(64000);
    let o = objects(c, vec![], true, None);
    let mut scope = selection();
    scope.usage = crate::modules::model_config::objects_overlay_v2::UsageV2::LocalFile;
    scope.channels[0].include_key = true;
    let expected = digest(b"outer selected target");
    let before = f.business();
    let helper = SystemNamespaceReceiveV2;
    let plan = helper
        .plan(
            &f.saved().library,
            &f.saved().policy,
            &o,
            &scope,
            &f.host.bindings,
            &[SOURCE.into()],
        )
        .unwrap();
    assert_eq!(
        plan.policy.channels[0]
            .namespace
            .as_ref()
            .unwrap()
            .channel_id,
        LOCAL
    );
    assert_eq!(f.business(), before); // Preview/cancel has no business writes.
    assert!(!f.host.root.join(RECEIVES).exists());
    assert!(f.host.prepare(OP, &o, &scope, &expected).is_err());
    let consumer = helper
        .consumer(
            &f.host.root,
            &f.host.home,
            f.host.bindings.clone(),
            &[SOURCE.into()],
        )
        .unwrap();
    let handle = consumer.prepare(OP, &o, &scope, &expected).unwrap();
    assert_eq!(f.business(), before);
    let frozen: Frozen =
        decoded(&std::fs::read(f.host.dir(OP).unwrap().join("plan.json")).unwrap()).unwrap();
    assert_eq!(frozen.namespace_adoption, vec![SOURCE.to_string()]);
    assert_eq!(frozen.objects_hash, objects_hash(&o).unwrap());
    assert!(matches!(
        consumer.status(OP, &handle).unwrap(),
        ModelApplyStatus::Prepared
    ));
    assert!(f.host.status(OP, &handle).is_err());
    assert!(f.host.protect(OP, &handle, false).is_err());
    drop(consumer);
    let reopened = helper
        .consumer(
            &f.host.root,
            &f.host.home,
            f.host.bindings.clone(),
            &[SOURCE.into()],
        )
        .unwrap();
    assert!(reopened.prepare(OP, &o, &scope, &expected).unwrap() == handle);
    reopened.protect(OP, &handle, false).unwrap();
    assert_eq!(f.business(), before);
    let receipt = reopened.apply_once(OP, &o, &handle, false).unwrap();
    assert_eq!(receipt.operation_id, OP);
    assert!(receipt.plan == handle);
    let saved = f.saved();
    let channel = &saved.policy.channels[0];
    assert_eq!(channel.id, LOCAL);
    assert_eq!(channel.namespace.as_ref().unwrap().channel_id, LOCAL);
    assert_eq!(
        channel.namespace.as_ref().unwrap().remove_leading_prefixes,
        vec!["vendor/"]
    );
    assert_eq!(channel.models[0].raw_model_id, "vendor/model");
    assert_eq!(channel.models[0].source.raw_model_id, "vendor/model");
    assert_eq!(
        f.raw()["providers"]["received"]["models"],
        json!(["vendor/model"])
    );
    assert_eq!(
        f.raw()["providers"]["received"]["modelContextWindows"]["vendor/model"],
        64000
    );
    assert_eq!(f.raw()["providers"]["received"]["apiKey"], SECRET);
    assert_eq!(
        saved.ledger.unwrap().written_file_digest,
        digest(&std::fs::read(f.host.home.join("config.json")).unwrap())
    );
    let own_receipt: Receipt =
        decoded(&std::fs::read(f.host.dir(OP).unwrap().join("receipt.json")).unwrap()).unwrap();
    assert_eq!(own_receipt.namespace_adoption, vec![SOURCE.to_string()]);
    assert_eq!(own_receipt.plan_hash, handle.token());
    assert!(f.host.status(OP, &handle).is_err());
    assert!(f.host.applied_mapping_proof(OP, &handle).is_err()); // No remote-proof authority.
    let after = f.business();
    let reopened = helper
        .consumer(
            &f.host.root,
            &f.host.home,
            f.host.bindings.clone(),
            &[SOURCE.into()],
        )
        .unwrap();
    assert!(matches!(
        reopened.status(OP, &handle).unwrap(),
        ModelApplyStatus::Applied(_)
    ));
    assert!(reopened.prepare(OP, &o, &scope, &expected).unwrap() == handle);
    reopened.apply_once(OP, &o, &handle, false).unwrap();
    assert_eq!(f.business(), after);
    let mut changed_bindings = f.host.bindings.clone();
    changed_bindings[0].provider_id = "changed-provider".into();
    let changed = helper
        .consumer(
            &f.host.root,
            &f.host.home,
            changed_bindings,
            &[SOURCE.into()],
        )
        .unwrap();
    assert!(changed.status(OP, &handle).is_err());
    assert!(changed.apply_once(OP, &o, &handle, false).is_err());
    let mut captured = crate::modules::model_config::file_objects_v2::decode_document(
        &crate::modules::model_config::file_objects_v2::encode_document(
            &o,
            Some(&ActualAuthorization),
        )
        .unwrap(),
        Some(&ActualAuthorization),
    )
    .unwrap()
    .into_captured();
    captured.models.payload.policy.as_mut().unwrap().channels[0]
        .namespace
        .as_mut()
        .unwrap()
        .remove_leading_prefixes = vec!["other/".into()];
    captured.models.digest = digest(&canonical_bytes(&captured.models.payload).unwrap());
    let changed_objects = SavedObjects::capture(captured, Some(&ActualAuthorization)).unwrap();
    assert!(reopened
        .prepare(OP, &changed_objects, &scope, &expected)
        .is_err());
    assert!(reopened
        .apply_once(OP, &changed_objects, &handle, false)
        .is_err());
    assert_eq!(f.business(), after);
}

#[test]
fn namespace_file_receiver_rejects_partial_selection_and_invalid_authority_without_business_writes()
{
    use crate::modules::model_config::file_transfer_v2::NamespaceReceiveV2;
    let f = Fixture::new();
    let mut c = channel(&["vendor/model"]);
    c.namespace = Some(NamespaceV2 {
        channel_id: SOURCE.into(),
        remove_leading_prefixes: vec!["vendor/".into()],
    });
    let objects = objects(c, vec![], false, None);
    let helper = SystemNamespaceReceiveV2;
    let before = f.business();
    for ids in [vec![SOURCE.into(), SOURCE.into()], vec![LOCAL.into()]] {
        assert!(helper
            .consumer(&f.host.root, &f.host.home, f.host.bindings.clone(), &ids)
            .is_err());
    }
    let mixed = ReceiveFileHostV2 {
        root: f.host.root.clone(),
        home: f.host.home.clone(),
        bindings: f.host.bindings.clone(),
        trusted_mapping_write: Some(IdentityMappingWriteV2 {
            before: None,
            after: b"untrusted mapping".to_vec(),
        }),
    };
    assert!(mixed.with_namespace_adoption(vec![SOURCE.into()]).is_err());
    let host = helper
        .consumer(
            &f.host.root,
            &f.host.home,
            f.host.bindings.clone(),
            &[SOURCE.into()],
        )
        .unwrap();
    let remote = selection();
    assert!(host
        .prepare(OP, &objects, &remote, &digest(b"outer selected target"))
        .is_err());
    assert!(helper
        .plan(
            &f.saved().library,
            &f.saved().policy,
            &objects,
            &remote,
            &f.host.bindings,
            &[SOURCE.into()]
        )
        .is_err());
    let mut partial = selection();
    partial.usage = crate::modules::model_config::objects_overlay_v2::UsageV2::LocalFile;
    partial.channels[0].model_ids = Some(vec!["vendor/model".into()]);
    assert!(host
        .prepare(OP, &objects, &partial, &digest(b"outer selected target"))
        .is_err());
    assert_eq!(f.business(), before);
    assert!(!f.host.root.join(RECEIVES).exists());
}

#[test]
fn namespace_localfile_owner_explicit_injection_cancel_and_reopen_use_original_plan_and_receipt() {
    use crate::modules::model_config::{
        file_transfer_v2::{
            ChannelBindingChoiceV2, FileRecoveryKeysV2, FileTransferOwnerV2, ImportBindingsV2,
        },
        objects_overlay_v2::UsageV2,
        saved_objects_capture_v2::{CaptureCredentialsV2, SelectedKeySourceV2},
    };
    use std::sync::Arc;
    struct NoSecrets;
    impl CaptureCredentialsV2 for NoSecrets {
        fn channel_key(
            &self,
            _: &Path,
            _: &SelectedKeySourceV2<'_>,
        ) -> ModelResult<Zeroizing<String>> {
            panic!("unselected key access")
        }
        fn authorization(
            &self,
            _: &Path,
            _: &str,
            _: &Value,
            _: &Value,
        ) -> ModelResult<PortableAuthorization> {
            panic!("unselected authorization access")
        }
        fn webdav_password(&self, _: &Path, _: &str) -> ModelResult<Zeroizing<String>> {
            panic!("unselected password access")
        }
    }
    struct MemoryKey;
    impl FileRecoveryKeysV2 for MemoryKey {
        fn key(&self, _: &Path, _: &Path, _: bool) -> ModelResult<Zeroizing<[u8; 32]>> {
            Ok(Zeroizing::new([81; 32]))
        }
    }
    let f = Fixture::new();
    let owner = || {
        FileTransferOwnerV2::new(
            f.host.root.clone(),
            f.host.home.clone(),
            Arc::new(NoSecrets),
        )
        .unwrap()
        .with_recovery_keys(Arc::new(MemoryKey))
        .with_namespace_receive(Arc::new(SystemNamespaceReceiveV2))
    };
    let mut c = channel(&["vendor/model"]);
    c.namespace = Some(NamespaceV2 {
        channel_id: SOURCE.into(),
        remove_leading_prefixes: vec!["vendor/".into()],
    });
    let o = objects(c, vec![], false, None);
    let bytes = file_objects_v2::encode(
        &o,
        OP,
        "2026-11-07T00:00:00Z",
        None,
        Some(&ActualAuthorization),
    )
    .unwrap()
    .into_bytes()
    .to_vec();
    let mut scope = selection();
    scope.usage = UsageV2::LocalFile;
    let choices = ImportBindingsV2 {
        channels: vec![ChannelBindingChoiceV2::New {
            portable_channel_id: SOURCE.into(),
        }],
        adopt_namespace_channel_ids: vec![SOURCE.into()],
        ..Default::default()
    };
    let before = f.business();
    let mut original = owner();
    let preview = original
        .prepare_import(bytes.clone(), None, Some(scope.clone()), choices.clone())
        .unwrap();
    assert!(preview.can_confirm, "{:?}", preview.reason_code);
    assert_eq!(f.business(), before);
    assert!(!f.host.root.join(RECEIVES).exists());
    assert!(original.cancel_import(&preview.operation_id).unwrap());
    assert_eq!(f.business(), before);
    let preview = original
        .prepare_import(bytes, None, Some(scope), choices)
        .unwrap();
    let operation = preview.operation_id.clone();
    let digest = preview.plan_digest.clone();
    drop(original);
    let mut resumed = owner();
    let reopened = resumed.reopen_import(&operation).unwrap();
    assert_eq!(reopened.plan_digest, digest);
    assert!(reopened.can_confirm, "{:?}", reopened.reason_code);
    let receipt = resumed
        .confirm_import(&operation, &digest, Some(false))
        .unwrap();
    assert_eq!(receipt.operation_id, operation);
    assert_eq!(receipt.plan_digest, digest);
    let saved = f.saved();
    let channel = &saved.policy.channels[0];
    assert_eq!(channel.id, SOURCE); // LocalFile New preserves the explicit available UUID.
    assert_eq!(channel.namespace.as_ref().unwrap().channel_id, channel.id);
    assert_eq!(
        channel.namespace.as_ref().unwrap().remove_leading_prefixes,
        vec!["vendor/"]
    );
    assert_eq!(channel.models[0].raw_model_id, "vendor/model");
    assert!(f
        .host
        .dir(&operation)
        .unwrap()
        .join("receipt.json")
        .is_file());
    let after = f.business();
    drop(resumed);
    let mut again = owner();
    let after_reopen = again.reopen_import(&operation).unwrap();
    assert_eq!(after_reopen.plan_digest, digest);
    let replay = again
        .confirm_import(&operation, &digest, Some(false))
        .unwrap();
    assert_eq!(replay.operation_id, operation);
    assert_eq!(f.business(), after);
}

#[test]
fn applied_mapping_proof_requires_original_manifest_and_receipt_even_after_later_edits() {
    let mut f = Fixture::new();
    let after = mapping_bytes(vec![mapping_row("endpoint", SOURCE, LOCAL, OP)]);
    f.host.trusted_mapping_write = Some(IdentityMappingWriteV2 {
        before: None,
        after: after.clone(),
    });
    let objects = simple(&["a"]);
    let handle = f.prepare(OP, &objects, &selection());
    assert!(f.host.applied_mapping_proof(OP, &handle).is_err());
    f.host.protect(OP, &handle, false).unwrap();
    f.host.apply_once(OP, &objects, &handle, false).unwrap();
    // Historical proof is not an assertion that today's config remains equal.
    std::fs::write(
        f.host.home.join("config.json"),
        b"{\"later_user_edit\":true}",
    )
    .unwrap();
    let before = f.business();
    let proof = f.host.applied_mapping_proof(OP, &handle).unwrap();
    assert_eq!(proof.operation_id(), OP);
    assert!(proof.handle() == &handle);
    assert!(proof.mapping_before().is_none());
    assert_eq!(proof.mapping_after(), after);
    assert_eq!(proof.expected_target(), digest(b"outer selected target"));
    assert_eq!(f.business(), before);
    let manifest_path = f.host.dir(OP).unwrap().join("plan.json");
    let original = std::fs::read(&manifest_path).unwrap();
    std::fs::write(&manifest_path, b"{}").unwrap();
    assert!(matches!(
        f.host.status(OP, &handle).unwrap(),
        ModelApplyStatus::Applied(_)
    ));
    assert!(f.host.applied_mapping_proof(OP, &handle).is_err());
    std::fs::write(&manifest_path, original).unwrap();
    std::fs::write(f.host.dir(OP).unwrap().join("6-after.bin"), b"corrupt").unwrap();
    assert!(f.host.applied_mapping_proof(OP, &handle).is_err());
    assert_eq!(f.business(), before);
}

#[test]
fn first_receive_preserves_explicit_no_secret_auth_mode_and_allows_second_receive() {
    for mode in ["none", "local"] {
        let mut f = Fixture::new();
        let mut c = channel(&["a"]);
        c.connection.as_mut().unwrap().auth_mode = Some(mode.into());
        let objects = objects(c, vec![], false, None);
        f.apply(OP, &objects, &selection(), false);
        assert_eq!(f.raw()["providers"]["received"]["authMode"], mode);
        assert!(f.raw()["providers"]["received"].get("apiKey").is_none());
        f.existing();
        f.apply(OP2, &objects, &selection(), false);
        assert_eq!(f.raw()["providers"]["received"]["authMode"], mode);
        assert!(f.raw()["providers"]["received"].get("apiKey").is_none());
    }
}

#[test]
fn backup_images_require_actual_protection_and_keep_original_before_after_without_receipt_targets()
{
    let f = Fixture::new();
    let objects = simple(&["a"]);
    let handle = f.prepare(OP, &objects, &selection());
    assert!(f.host.protected_images(OP, &handle).is_err());
    f.host.protect(OP, &handle, true).unwrap();
    let expected = f.host.protected_images(OP, &handle).unwrap();
    assert_eq!(expected.len(), 6);
    assert!(matches!(expected[0].target, TargetV2::OfficialConfig));
    assert!(expected
        .iter()
        .all(|w| !matches!(w.target, TargetV2::ManagerOperationReceipt(_))));
    f.host.apply_once(OP, &objects, &handle, true).unwrap();
    let before = std::fs::read(f.host.home.join("config.json")).unwrap();
    let images = f.host.scope_backup_images(OP, &handle).unwrap().unwrap();
    assert_eq!(images.len(), expected.len());
    for (a, b) in images.iter().zip(&expected) {
        assert!(a.target == b.target);
        assert_eq!(a.before, b.before);
        assert_eq!(a.after, b.after);
    }
    assert_eq!(
        std::fs::read(f.host.home.join("config.json")).unwrap(),
        before
    );
    let other = ModelApplyPlanHandle::new(digest(b"wrong handle")).unwrap();
    assert!(f.host.protected_images(OP, &other).is_err());
}
