//! Real v6 model/manager/assets fixtures; memory-only overlays and save atoms.
use super::super::{
    file_objects_v2,
    schema::{Field, ModelRules},
    schema_v2::*,
    source_v2::source_from_provider_v2,
};
use super::*;
use serde_json::json;
const A: &str = "11111111-1111-4111-8111-111111111111";
const B: &str = "22222222-2222-4222-8222-222222222222";
const NODE: &str = "33333333-3333-4333-8333-333333333333";
const TIME: &str = "2026-11-06T12:00:00Z";
fn c(id: &str, ids: &[&str]) -> PortableChannelV2 {
    PortableChannelV2 {
        id: id.into(),
        label: format!("channel-{id}"),
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
        models: ids
            .iter()
            .map(|raw| ModelConfigV2 {
                raw_model_id: (*raw).into(),
                label: (*raw).into(),
                observed: true,
                source: source_from_provider_v2(
                    id,
                    raw,
                    &json!({"adapter":"openai-chat"}),
                    &schema_v2::digest(b"source-a"),
                    SourceKindV2::ImportedFile,
                )
                .unwrap(),
                explicit_overrides: ModelRules::default(),
                binding: None,
            })
            .collect(),
        template_refs: vec![],
        selection: ModelSelection::Whitelist(ids.iter().map(|id| (*id).into()).collect()),
        model_order: ids.iter().map(|id| (*id).into()).collect(),
        defaults: ModelRules::default(),
        family_overrides: BTreeMap::new(),
        naming: BTreeMap::new(),
        namespace: None,
    }
}
fn input(channels: Vec<PortableChannelV2>) -> CapturedObjects {
    let order = channels.iter().map(|c| c.id.clone()).collect();
    let payload = ModelPayloadV2 {
        templates: vec![],
        policy: Some(PortablePolicyV2 {
            template_refs: vec![],
            channels,
            channel_order: order,
            default_channel_id: None,
        }),
    };
    CapturedObjects {
        models: PortableModelsV2 {
            format: portable_v2::PORTABLE_FORMAT.into(),
            format_version: 6,
            schema_version: 2,
            created_at: TIME.into(),
            digest: schema_v2::digest(&schema_v2::canonical_bytes(&payload).unwrap()),
            payload,
        },
        manager: Default::default(),
        assets: Default::default(),
        channel_keys: SecretValues::new(),
        channel_authorizations: BTreeMap::new(),
        webdav_passwords: SecretValues::new(),
        content_versions: vec![],
    }
}
fn saved(input: CapturedObjects) -> SavedObjects {
    SavedObjects::capture(input, None).unwrap()
}
fn rehash(input: &mut CapturedObjects) {
    input.models.digest =
        schema_v2::digest(&schema_v2::canonical_bytes(&input.models.payload).unwrap());
}
fn selection(id: &str) -> SelectionV2 {
    SelectionV2 {
        channels: vec![ChannelSelectionV2 {
            portable_channel_id: id.into(),
            model_ids: None,
            family_ids: None,
            include_connection: true,
            include_key: false,
            include_authorization: false,
        }],
        ..Default::default()
    }
}
fn code<T>(r: ModelResult<T>) -> String {
    match r {
        Ok(_) => panic!("expected error"),
        Err(e) => e.code,
    }
}
#[test]
fn save_state_v2_channel_scope_keys_exclude_pure_evidence_but_keep_business_values() {
    let original = saved(input(vec![c(A, &["a"])]));
    let first = file_objects_v2::save_state(&original).unwrap();
    let digest = file_objects_v2::semantic_digest(&original).unwrap();
    let key = super::super::content_versions::key(&json!(["v2-channel", A, null, null])).unwrap();
    assert!(first.contains_key(&key));
    assert!(!first.contains_key(
        &super::super::content_versions::key(&json!(["channel", A, null, null])).unwrap()
    ));
    let mut changed = original.into_captured();
    let field = changed.models.payload.policy.as_mut().unwrap().channels[0].models[0]
        .source
        .fields
        .get_mut("protocol")
        .unwrap();
    field.evidence.source_digest = schema_v2::digest(b"same-business-new-snapshot");
    field.evidence.kind = SourceKindV2::ExternalFileEdit;
    rehash(&mut changed);
    let newer = saved(changed);
    assert_eq!(first, file_objects_v2::save_state(&newer).unwrap());
    assert_eq!(digest, file_objects_v2::semantic_digest(&newer).unwrap());
    let mut changed = newer.into_captured();
    changed.models.payload.policy.as_mut().unwrap().channels[0].models[0]
        .explicit_overrides
        .context_window = Field::Custom(8192);
    rehash(&mut changed);
    let different = saved(changed);
    assert_ne!(
        first[&key],
        file_objects_v2::save_state(&different).unwrap()[&key]
    );
    assert_ne!(
        digest,
        file_objects_v2::semantic_digest(&different).unwrap()
    );
}
#[test]
fn save_state_v2_receipts_do_not_date_themselves_and_secret_values_are_only_hashes() {
    let mut value = input(vec![c(A, &["a"])]);
    value
        .channel_keys
        .insert(A.into(), Zeroizing::new("synthetic-sensitive-value".into()))
        .unwrap();
    let objects = saved(value);
    let first = file_objects_v2::save_state(&objects).unwrap();
    let mut value = objects.into_captured();
    value
        .content_versions
        .push(super::super::content_versions::Version::unknown(
            "fixture-scope",
            &schema_v2::digest(b"receipt"),
        ));
    assert_eq!(first, file_objects_v2::save_state(&saved(value)).unwrap());
    let json = serde_json::to_string(&first).unwrap();
    assert!(!json.contains("synthetic-sensitive-value"));
    assert!(first.values().all(|v| schema_v2::valid_digest(v)));
}
#[test]
fn empty_selection_preserves_complete_baseline_and_zero_baseline_does_not_select_local_extras() {
    let mut base = input(vec![c(A, &["a"]), c(B, &["b"])]);
    base.channel_keys
        .insert(B.into(), Zeroizing::new("base-secret".into()))
        .unwrap();
    let baseline = saved(base);
    let before = file_objects_v2::semantic_digest(&baseline).unwrap();
    let result = overlay_selected(
        Some(baseline),
        saved(input(vec![c(A, &["new"])])),
        SelectionV2::default(),
        None,
    )
    .unwrap();
    assert_eq!(before, file_objects_v2::semantic_digest(&result).unwrap());
    assert_eq!(result.partitions().channel_keys.get(B), Some("base-secret"));
    let result = overlay_selected(
        None,
        saved(input(vec![c(A, &["a"]), c(B, &["b"])])),
        selection(A),
        None,
    )
    .unwrap();
    let policy = result.models().payload.policy.as_ref().unwrap();
    assert_eq!(policy.channels.len(), 1);
    assert_eq!(policy.channels[0].id, A);
    assert!(result.partitions().manager.preferences.is_none());
}
#[test]
fn partial_channel_updates_only_exact_exported_ids_and_preserves_order_and_keys() {
    let mut baseline = input(vec![c(A, &["a", "b"]), c(B, &["c"])]);
    baseline
        .channel_keys
        .insert(A.into(), Zeroizing::new("base-key".into()))
        .unwrap();
    baseline.models.payload.policy.as_mut().unwrap().channels[0].models[1]
        .explicit_overrides
        .context_window = Field::Custom(4096);
    rehash(&mut baseline);
    let original_b = baseline.models.payload.policy.as_ref().unwrap().channels[0].models[1].clone();
    let original_other = baseline.models.payload.policy.as_ref().unwrap().channels[1].clone();
    let mut local = input(vec![c(A, &["a"])]);
    let incoming = &mut local.models.payload.policy.as_mut().unwrap().channels[0];
    incoming.model_subset = Some(vec!["a".into()]);
    incoming.models[0].explicit_overrides.context_window = Field::Custom(8192);
    local
        .channel_keys
        .insert(A.into(), Zeroizing::new("unselected-local-key".into()))
        .unwrap();
    rehash(&mut local);
    let mut scope = selection(A);
    scope.channels[0].model_ids = Some(vec!["a".into()]);
    scope.channels[0].include_connection = false;
    let result = overlay_selected(Some(saved(baseline)), saved(local), scope, None).unwrap();
    let policy = result.models().payload.policy.as_ref().unwrap();
    let first = &policy.channels[0];
    assert_eq!(first.models.len(), 2);
    assert_eq!(first.models[1], original_b);
    assert_eq!(policy.channels[1], original_other);
    assert_eq!(first.model_order, vec!["a", "b"]);
    assert_eq!(policy.channel_order, vec![A, B]);
    assert_eq!(result.partitions().channel_keys.get(A), Some("base-key"));
    assert!(first.model_subset.is_none());
}
#[test]
fn repeated_same_head_and_same_selection_are_semantic_noops() {
    let baseline = saved(input(vec![c(A, &["a", "b"])]));
    let digest = file_objects_v2::semantic_digest(&baseline).unwrap();
    let make_local = || {
        let mut local = input(vec![c(A, &["a"])]);
        local.models.payload.policy.as_mut().unwrap().channels[0].model_subset =
            Some(vec!["a".into()]);
        rehash(&mut local);
        saved(local)
    };
    let mut scope = selection(A);
    scope.channels[0].model_ids = Some(vec!["a".into()]);
    scope.channels[0].include_connection = false;
    let result = overlay_selected(Some(baseline), make_local(), scope.clone(), None).unwrap();
    assert_eq!(digest, file_objects_v2::semantic_digest(&result).unwrap());
    let again = overlay_selected(Some(result), make_local(), scope, None).unwrap();
    assert_eq!(digest, file_objects_v2::semantic_digest(&again).unwrap());
}
#[test]
fn missing_scope_duplicate_ids_and_capture_scope_expansion_are_rejected() {
    assert_eq!(
        code(overlay_selected(
            None,
            saved(input(vec![c(A, &["a"])])),
            selection(B),
            None
        )),
        "overlay_selection_missing"
    );
    let mut scope = selection(A);
    scope.channels.push(scope.channels[0].clone());
    assert_eq!(
        code(overlay_selected(
            None,
            saved(input(vec![c(A, &["a"])])),
            scope,
            None
        )),
        "overlay_scope_invalid"
    );
    let mut scope = selection(A);
    scope.channels[0].model_ids = Some(vec!["a".into()]);
    assert_eq!(
        code(overlay_selected(
            None,
            saved(input(vec![c(A, &["a", "b"])])),
            scope,
            None
        )),
        "overlay_channel_scope_conflict"
    );
    let mut scope = selection(A);
    scope.channels[0].include_connection = false;
    assert_eq!(
        code(overlay_selected(
            None,
            saved(input(vec![c(A, &["a"])])),
            scope,
            None
        )),
        "overlay_connection_required"
    );
}
#[test]
fn shared_rules_and_all_selection_cannot_silently_change_unselected_models() {
    let make = || {
        let mut local = input(vec![c(A, &["a"])]);
        let ch = &mut local.models.payload.policy.as_mut().unwrap().channels[0];
        ch.model_subset = Some(vec!["a".into()]);
        ch.defaults.context_window = Field::Custom(8192);
        rehash(&mut local);
        saved(local)
    };
    let mut scope = selection(A);
    scope.channels[0].model_ids = Some(vec!["a".into()]);
    scope.channels[0].include_connection = false;
    assert_eq!(
        code(overlay_selected(
            Some(saved(input(vec![c(A, &["a", "b"])]))),
            make(),
            scope.clone(),
            None
        )),
        "overlay_shared_rules_conflict"
    );
    let mut baseline = input(vec![c(A, &["a", "b"])]);
    baseline.models.payload.policy.as_mut().unwrap().channels[0].selection = ModelSelection::All;
    rehash(&mut baseline);
    let mut local = input(vec![c(A, &["a"])]);
    let ch = &mut local.models.payload.policy.as_mut().unwrap().channels[0];
    ch.model_subset = Some(vec!["a".into()]);
    ch.selection = ModelSelection::None;
    rehash(&mut local);
    assert_eq!(
        code(overlay_selected(
            Some(saved(baseline)),
            saved(local),
            scope,
            None
        )),
        "overlay_model_selection_conflict"
    );
}
#[test]
fn key_selection_is_independent_and_retained_credentials_never_follow_changed_connection() {
    let make_base = || {
        let mut v = input(vec![c(A, &["a"])]);
        v.channel_keys
            .insert(A.into(), Zeroizing::new("base-key".into()))
            .unwrap();
        saved(v)
    };
    let make_local = |changed: bool| {
        let mut v = input(vec![c(A, &["a"])]);
        v.channel_keys
            .insert(A.into(), Zeroizing::new("local-key".into()))
            .unwrap();
        if changed {
            v.models.payload.policy.as_mut().unwrap().channels[0]
                .connection
                .as_mut()
                .unwrap()
                .auth_mode = Some("none".into());
            rehash(&mut v);
        }
        saved(v)
    };
    let result =
        overlay_selected(Some(make_base()), make_local(false), selection(A), None).unwrap();
    assert_eq!(result.partitions().channel_keys.get(A), Some("base-key"));
    let mut scope = selection(A);
    scope.channels[0].include_key = true;
    let result =
        overlay_selected(Some(make_base()), make_local(false), scope.clone(), None).unwrap();
    assert_eq!(result.partitions().channel_keys.get(A), Some("local-key"));
    assert_eq!(
        code(overlay_selected(
            Some(make_base()),
            make_local(true),
            selection(A),
            None
        )),
        "overlay_unselected_credential_conflict"
    );
    scope.channels[0].include_connection = false;
    assert_eq!(
        code(overlay_selected(
            Some(make_base()),
            make_local(true),
            scope,
            None
        )),
        "overlay_unselected_credential_conflict"
    );
}
fn mcp(command: &str, secret: Option<&str>) -> Mcp {
    let mut secrets = portable_assets::Secrets::default();
    if let Some(secret) = secret {
        secrets.0.insert("/env/FIXTURE_TOKEN".into(), secret.into());
    }
    Mcp {
        name: "fixture".into(),
        definition: json!({"command":command,"args":[]}),
        secret_fields: vec!["/env/FIXTURE_TOKEN".into()],
        secrets,
        warnings: vec![],
    }
}
#[test]
fn mcp_source_digest_and_explicit_credential_selection_are_separate() {
    let mut base = input(vec![]);
    base.assets.mcps.push(mcp("base", Some("base-secret")));
    let mut local = input(vec![]);
    local.assets.mcps.push(mcp("base", Some("local-secret")));
    let h = mcp_source_digest(&local.assets.mcps[0]).unwrap();
    let scope = SelectionV2 {
        mcp_sources: vec![McpSourceSelectionV2 {
            name: "fixture".into(),
            definition_digest: h,
        }],
        ..Default::default()
    };
    let result = overlay_selected(Some(saved(base)), saved(local), scope, None).unwrap();
    assert_eq!(
        result.partitions().assets.mcps[0].secrets.0["/env/FIXTURE_TOKEN"],
        "base-secret"
    );
    let mut local = input(vec![]);
    local.assets.mcps.push(mcp("different-source", None));
    let scope = SelectionV2 {
        mcp_sources: vec![McpSourceSelectionV2 {
            name: "fixture".into(),
            definition_digest: schema_v2::digest(b"old source"),
        }],
        ..Default::default()
    };
    assert_eq!(
        code(overlay_selected(None, saved(local), scope, None)),
        "overlay_mcp_source_changed"
    );
    let mut base = input(vec![]);
    base.assets.mcps.push(mcp("base", Some("base-secret")));
    let mut local = input(vec![]);
    local
        .assets
        .mcps
        .push(mcp("new-service", Some("local-secret")));
    let h = mcp_source_digest(&local.assets.mcps[0]).unwrap();
    let mut scope = SelectionV2 {
        mcp_sources: vec![McpSourceSelectionV2 {
            name: "fixture".into(),
            definition_digest: h,
        }],
        ..Default::default()
    };
    scope.extension_credentials.mcp_names.push("fixture".into());
    let result = overlay_selected(Some(saved(base)), saved(local), scope, None).unwrap();
    assert_eq!(
        result.partitions().assets.mcps[0].secrets.0["/env/FIXTURE_TOKEN"],
        "local-secret"
    );
}
#[test]
fn changed_mcp_definition_cannot_reuse_unselected_secret_or_guess_case_collision() {
    let mut base = input(vec![]);
    base.assets.mcps.push(mcp("old", Some("old-secret")));
    let mut local = input(vec![]);
    local.assets.mcps.push(mcp("new", None));
    let h = mcp_source_digest(&local.assets.mcps[0]).unwrap();
    let scope = SelectionV2 {
        mcp_sources: vec![McpSourceSelectionV2 {
            name: "fixture".into(),
            definition_digest: h,
        }],
        ..Default::default()
    };
    assert_eq!(
        code(overlay_selected(
            Some(saved(base)),
            saved(local),
            scope,
            None
        )),
        "overlay_unselected_credential_conflict"
    );
    let mut base = input(vec![]);
    let mut old = mcp("old", None);
    old.name = "FIXTURE".into();
    base.assets.mcps.push(old);
    let mut local = input(vec![]);
    local.assets.mcps.push(mcp("new", None));
    let h = mcp_source_digest(&local.assets.mcps[0]).unwrap();
    let scope = SelectionV2 {
        mcp_sources: vec![McpSourceSelectionV2 {
            name: "fixture".into(),
            definition_digest: h,
        }],
        ..Default::default()
    };
    assert_eq!(
        code(overlay_selected(
            Some(saved(base)),
            saved(local),
            scope,
            None
        )),
        "overlay_asset_identity_conflict"
    );
}
#[test]
fn manager_extension_configuration_is_explicit_and_independent_of_assets() {
    let mut base = input(vec![]);
    let mut config = crate::modules::extensions::ExtensionConfig {
        skills: vec!["old".into()],
        ..Default::default()
    };
    base.manager.extensions = Some(config.clone());
    let mut local = input(vec![]);
    config.skills = vec!["new".into()];
    local.manager.extensions = Some(config.clone());
    local.assets.mcps.push(mcp("selected-asset", None));
    let h = mcp_source_digest(&local.assets.mcps[0]).unwrap();
    let scope = SelectionV2 {
        mcp_sources: vec![McpSourceSelectionV2 {
            name: "fixture".into(),
            definition_digest: h,
        }],
        ..Default::default()
    };
    let result = overlay_selected(Some(saved(base)), saved(local), scope, None).unwrap();
    assert_eq!(
        result
            .partitions()
            .manager
            .extensions
            .as_ref()
            .unwrap()
            .skills,
        vec!["old"]
    );
    let mut local = input(vec![]);
    local.manager.extensions = Some(config);
    let mut scope = SelectionV2::default();
    scope.manager.extensions = true;
    let result = overlay_selected(Some(result), saved(local), scope, None).unwrap();
    assert_eq!(
        result
            .partitions()
            .manager
            .extensions
            .as_ref()
            .unwrap()
            .skills,
        vec!["new"]
    );
}
#[test]
fn save_state_v2_mcp_public_and_secret_atoms_are_independent_and_templates_are_immutable_material()
{
    let mut first = input(vec![]);
    first.assets.mcps.push(mcp("same", Some("first-secret")));
    let first = file_objects_v2::save_state(&saved(first)).unwrap();
    let mut next = input(vec![]);
    next.assets.mcps.push(mcp("same", Some("second-secret")));
    let next = file_objects_v2::save_state(&saved(next)).unwrap();
    let public = super::super::content_versions::key(&json!(["v2-mcps", "fixture"])).unwrap();
    let secret = super::super::content_versions::key(&json!(["v2-mcpSecrets", "fixture"])).unwrap();
    assert_eq!(first[&public], next[&public]);
    assert_ne!(first[&secret], next[&secret]);
}
fn template_pair() -> (TemplateV2, TemplateV2, RefV2, RefV2) {
    let seed = seed_library_v2().unwrap();
    let mut old = seed.templates[0].clone();
    let r1 = old.revisions[0].clone();
    old.revisions = vec![r1.clone()];
    old.default_revision = Some(r1.id.clone());
    let mut r2 = r1.clone();
    r2.id = NODE.into();
    r2.created_at = "2026-11-06T12:01:00Z".into();
    r2.saved_label.push_str(" fixture-v2");
    r2.content.description.push_str(" fixture revision");
    r2.digest = content_digest(&r2.saved_label, &r2.content).unwrap();
    let first = RefV2 {
        template_id: old.id.clone(),
        revision_id: r1.id.clone(),
        digest: r1.digest.clone(),
    };
    let second = RefV2 {
        template_id: old.id.clone(),
        revision_id: r2.id.clone(),
        digest: r2.digest.clone(),
    };
    let mut newer = old.clone();
    newer.revisions.push(r2);
    newer.default_revision = Some(NODE.into());
    (old, newer, first, second)
}
#[test]
fn exact_template_dependencies_add_revisions_without_implicit_default_adoption() {
    let (old, newer, first, second) = template_pair();
    let make_base = || {
        let mut b = input(vec![]);
        b.models.payload.templates = vec![old.clone()];
        rehash(&mut b);
        saved(b)
    };
    let make_local = || {
        let mut l = input(vec![]);
        l.models.payload.templates = vec![newer.clone()];
        rehash(&mut l);
        saved(l)
    };
    let s = SelectionV2 {
        template_refs: vec![second.clone()],
        ..Default::default()
    };
    let result = overlay_selected(Some(make_base()), make_local(), s.clone(), None).unwrap();
    let t = &result.models().payload.templates[0];
    assert_eq!(t.revisions.len(), 2);
    assert_eq!(
        t.default_revision.as_deref(),
        Some(first.revision_id.as_str())
    );
    let mut s = s;
    s.adopt_template_defaults = vec![second.clone()];
    let result = overlay_selected(Some(make_base()), make_local(), s, None).unwrap();
    assert_eq!(
        result.models().payload.templates[0]
            .default_revision
            .as_deref(),
        Some(second.revision_id.as_str())
    );
    let s = SelectionV2 {
        template_refs: vec![second.clone()],
        ..Default::default()
    };
    let result = overlay_selected(None, make_local(), s, None).unwrap();
    let t = &result.models().payload.templates[0];
    assert_eq!(t.revisions.len(), 1);
    assert_eq!(t.revisions[0].id, second.revision_id);
    assert!(t.default_revision.is_none());
}
#[test]
fn immutable_revision_collision_and_stale_explicit_default_are_errors() {
    let (old, newer, first, _) = template_pair();
    let mut base = input(vec![]);
    base.models.payload.templates = vec![old.clone()];
    rehash(&mut base);
    let mut source = input(vec![]);
    let mut conflicting = old;
    conflicting.revisions[0].created_at = "2026-11-06T13:00:00Z".into();
    source.models.payload.templates = vec![conflicting];
    rehash(&mut source);
    let scope = SelectionV2 {
        template_refs: vec![first.clone()],
        ..Default::default()
    };
    assert_eq!(
        code(overlay_selected(
            Some(saved(base)),
            saved(source),
            scope,
            None
        )),
        "model_revision_digest_mismatch"
    );
    let mut source = input(vec![]);
    source.models.payload.templates = vec![newer];
    rehash(&mut source);
    let scope = SelectionV2 {
        adopt_template_defaults: vec![first],
        ..Default::default()
    };
    assert_eq!(
        code(overlay_selected(None, saved(source), scope, None)),
        "overlay_default_source_changed"
    );
}
#[test]
fn selected_model_reference_brings_exact_revision_and_keeps_unselected_versions() {
    let (old, newer, first, second) = template_pair();
    let mut base = input(vec![c(A, &["a", "b"])]);
    base.models.payload.templates = vec![old];
    base.models.payload.policy.as_mut().unwrap().channels[0].models[1].binding =
        Some(ModelBindingV2::Manual {
            reference: first.clone(),
        });
    rehash(&mut base);
    let mut source = input(vec![c(A, &["a"])]);
    source.models.payload.templates = vec![newer];
    let ch = &mut source.models.payload.policy.as_mut().unwrap().channels[0];
    ch.models[0].binding = Some(ModelBindingV2::Manual {
        reference: second.clone(),
    });
    ch.model_subset = Some(vec!["a".into()]);
    rehash(&mut source);
    let mut s = selection(A);
    s.channels[0].include_connection = false;
    s.channels[0].model_ids = Some(vec!["a".into()]);
    let result = overlay_selected(Some(saved(base)), saved(source), s, None).unwrap();
    let ch = &result.models().payload.policy.as_ref().unwrap().channels[0];
    assert_eq!(ch.models[0].binding.as_ref().unwrap().reference(), &second);
    assert_eq!(ch.models[1].binding.as_ref().unwrap().reference(), &first);
    assert_eq!(result.models().payload.templates[0].revisions.len(), 2);
}
fn skill(name: &str, body: &str, secret: Option<&str>) -> Skill {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let mut files = vec![portable_assets::SkillFile {
        path: "SKILL.md".into(),
        digest: schema_v2::digest(body.as_bytes()),
        data_b64: STANDARD.encode(body),
        executable: true,
        sensitive: false,
    }];
    if let Some(secret) = secret {
        files.push(portable_assets::SkillFile {
            path: ".env".into(),
            digest: schema_v2::digest(secret.as_bytes()),
            data_b64: STANDARD.encode(secret),
            executable: false,
            sensitive: true,
        });
    }
    Skill {
        name: name.into(),
        digest: schema_v2::digest(&serde_json::to_vec(&files).unwrap()),
        files,
        contains_secrets: secret.is_some(),
        warnings: vec![],
    }
}
#[test]
fn skills_content_and_secrets_overlay_independently_and_other_skills_are_preserved() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let make_base = || {
        let mut b = input(vec![]);
        b.assets.skills = vec![
            skill("selected", "# old", Some("TOKEN=old")),
            skill("untouched", "# keep", None),
        ];
        saved(b)
    };
    let make_local = || {
        let mut l = input(vec![]);
        l.assets.skills = vec![skill("selected", "# new", Some("TOKEN=new"))];
        saved(l)
    };
    let scope = SelectionV2 {
        skill_names: vec!["selected".into()],
        ..Default::default()
    };
    let result = overlay_selected(Some(make_base()), make_local(), scope.clone(), None).unwrap();
    assert_eq!(result.partitions().assets.skills.len(), 2);
    let selected = &result.partitions().assets.skills[0];
    assert_eq!(
        STANDARD
            .decode(
                &selected
                    .files
                    .iter()
                    .find(|f| f.path == ".env")
                    .unwrap()
                    .data_b64
            )
            .unwrap(),
        b"TOKEN=old"
    );
    assert_eq!(
        STANDARD.decode(&selected.files[0].data_b64).unwrap(),
        b"# new"
    );
    assert_eq!(result.partitions().assets.skills[1].name, "untouched");
    let mut scope = scope;
    scope.extension_credentials.skill_names = vec!["selected".into()];
    let result = overlay_selected(Some(make_base()), make_local(), scope, None).unwrap();
    let selected = &result.partitions().assets.skills[0];
    assert_eq!(
        STANDARD
            .decode(
                &selected
                    .files
                    .iter()
                    .find(|f| f.path == ".env")
                    .unwrap()
                    .data_b64
            )
            .unwrap(),
        b"TOKEN=new"
    );
}
#[test]
fn save_state_v2_full_revision_and_projected_channel_keys_never_collide() {
    let (old, _, first, _) = template_pair();
    let mut base = input(vec![c(A, &["a"])]);
    base.models.payload.templates = vec![old.clone()];
    rehash(&mut base);
    let original = saved(base);
    let full = file_objects_v2::save_state(&original).unwrap();
    let mut next = original.into_captured();
    next.models.payload.templates[0].revisions[0].created_at = "2026-11-06T14:00:00Z".into();
    next.models.payload.policy.as_mut().unwrap().channels[0].model_subset = Some(vec!["a".into()]);
    rehash(&mut next);
    let projected = file_objects_v2::save_state(&saved(next)).unwrap();
    let full_key =
        super::super::content_versions::key(&json!(["v2-channel", A, null, null])).unwrap();
    let projected_key =
        super::super::content_versions::key(&json!(["v2-channel", A, ["a"], null])).unwrap();
    assert!(full.contains_key(&full_key));
    assert!(!projected.contains_key(&full_key));
    assert!(projected.contains_key(&projected_key));
    let revision_key =
        super::super::content_versions::key(&json!(["v2-template", old.id, first.revision_id]))
            .unwrap();
    assert_ne!(full[&revision_key], projected[&revision_key]);
}
fn receipt(
    scope_key: String,
    content_digest: String,
    when: &str,
) -> super::super::content_versions::Version {
    super::super::content_versions::Version {
        scope_key,
        content_digest,
        revision: 3,
        saved_at: when.into(),
        time_kind: super::super::content_versions::TimeKind::Saved,
        trusted_floor: Some(when.into()),
        last_event: Some(NODE.into()),
    }
}
#[test]
fn receipts_are_inherited_only_for_exact_selected_final_atoms_and_never_redate_unselected_content()
{
    let base = saved(input(vec![c(A, &["a"]), c(B, &["b"])]));
    let atoms = file_objects_v2::save_state(&base).unwrap();
    let a = super::super::content_versions::key(&json!(["v2-channel", A, null, null])).unwrap();
    let b = super::super::content_versions::key(&json!(["v2-channel", B, null, null])).unwrap();
    let mut baseline = base.into_captured();
    baseline.content_versions = vec![
        receipt(a.clone(), atoms[&a].clone(), TIME),
        receipt(b.clone(), atoms[&b].clone(), TIME),
    ];
    let mut local = input(vec![c(A, &["a"]), c(B, &["b"])]);
    local.models.payload.policy.as_mut().unwrap().channels[0].models[0]
        .explicit_overrides
        .context_window = Field::Custom(8192);
    rehash(&mut local);
    let source = saved(local);
    let source_atoms = file_objects_v2::save_state(&source).unwrap();
    let mut local = source.into_captured();
    local.content_versions = vec![
        receipt(a.clone(), source_atoms[&a].clone(), "2026-11-06T15:00:00Z"),
        receipt(b.clone(), source_atoms[&b].clone(), "2026-11-06T16:00:00Z"),
    ];
    let result = overlay_selected(Some(saved(baseline)), saved(local), selection(A), None).unwrap();
    let versions = result.partitions().content_versions;
    assert_eq!(
        versions.iter().find(|v| v.scope_key == a).unwrap().saved_at,
        "2026-11-06T15:00:00Z"
    );
    assert_eq!(
        versions.iter().find(|v| v.scope_key == b).unwrap().saved_at,
        TIME
    );
}
#[test]
fn projected_local_receipt_cannot_date_a_new_combination_or_replace_known_same_content_time() {
    let base = saved(input(vec![c(A, &["a", "b"])]));
    let state = file_objects_v2::save_state(&base).unwrap();
    let full_key =
        super::super::content_versions::key(&json!(["v2-channel", A, null, null])).unwrap();
    let mut baseline = base.into_captured();
    baseline.content_versions = vec![receipt(full_key.clone(), state[&full_key].clone(), TIME)];
    let mut local = input(vec![c(A, &["a"])]);
    let ch = &mut local.models.payload.policy.as_mut().unwrap().channels[0];
    ch.model_subset = Some(vec!["a".into()]);
    ch.models[0].explicit_overrides.context_window = Field::Custom(8192);
    rehash(&mut local);
    let source = saved(local);
    let state = file_objects_v2::save_state(&source).unwrap();
    let projected =
        super::super::content_versions::key(&json!(["v2-channel", A, ["a"], null])).unwrap();
    let mut local = source.into_captured();
    local.content_versions = vec![receipt(
        projected,
        state.values().next().cloned().unwrap(),
        "2026-11-06T17:00:00Z",
    )];
    // Bind to its actual projection atom, never a selected arbitrary map entry.
    local.content_versions[0].content_digest = state[&local.content_versions[0].scope_key].clone();
    let mut scope = selection(A);
    scope.channels[0].include_connection = false;
    scope.channels[0].model_ids = Some(vec!["a".into()]);
    let result = overlay_selected(Some(saved(baseline)), saved(local), scope, None).unwrap();
    let final_atoms = file_objects_v2::save_state(&result).unwrap();
    let old = &result.partitions().content_versions[0];
    assert_eq!(old.saved_at, TIME);
    assert_ne!(old.content_digest, final_atoms[&full_key]);
    let original = saved(input(vec![c(A, &["a"])]));
    let state = file_objects_v2::save_state(&original).unwrap();
    let key = super::super::content_versions::key(&json!(["v2-channel", A, null, null])).unwrap();
    let mut b = original.into_captured();
    b.content_versions = vec![receipt(key.clone(), state[&key].clone(), TIME)];
    let mut l = input(vec![c(A, &["a"])]);
    l.content_versions = vec![receipt(
        key.clone(),
        state[&key].clone(),
        "2026-11-06T18:00:00Z",
    )];
    let result = overlay_selected(Some(saved(b)), saved(l), selection(A), None).unwrap();
    assert_eq!(result.partitions().content_versions[0].saved_at, TIME);
}
#[test]
fn webdav_local_file_selection_is_explicit_and_remote_self_sync_is_rejected() {
    let make = || {
        let mut v = input(vec![]);
        v.manager.webdav.push(portable_manager::Endpoint {
            id: "fixture".into(),
            url: "https://dav.example.test".into(),
            remote_path: "scope".into(),
            username: "fixture".into(),
            conflict_policy: "ask".into(),
        });
        v.webdav_passwords
            .insert(
                "fixture".into(),
                Zeroizing::new("synthetic-password".into()),
            )
            .unwrap();
        saved(v)
    };
    let mut s = SelectionV2::default();
    s.manager.webdav_ids = vec!["fixture".into()];
    s.manager.webdav_password_ids = vec!["fixture".into()];
    assert_eq!(
        code(overlay_selected(None, make(), s.clone(), None)),
        "sync_self_scope"
    );
    s.usage = UsageV2::LocalFile;
    let result = overlay_selected(None, make(), s, None).unwrap();
    assert_eq!(
        result.partitions().webdav_passwords.get("fixture"),
        Some("synthetic-password")
    );
    assert!(result.validate_remote_sync_scope().is_err());
}
#[test]
fn selected_channel_name_collision_with_unselected_channel_is_not_silently_renamed() {
    let baseline = saved(input(vec![c(A, &["a"]), c(B, &["b"])]));
    let mut local = input(vec![c(A, &["a"])]);
    local.models.payload.policy.as_mut().unwrap().channels[0].label = format!("channel-{B}");
    rehash(&mut local);
    assert_eq!(
        code(overlay_selected(
            Some(baseline),
            saved(local),
            selection(A),
            None
        )),
        "overlay_channel_name_conflict"
    );
}
#[test]
fn family_fragment_updates_only_declared_family_and_preserves_channel_wide_rules() {
    let (old, newer, first, second) = template_pair();
    let mut base = input(vec![c(A, &["a", "b"])]);
    base.models.payload.templates = vec![old];
    let base_channel = &mut base.models.payload.policy.as_mut().unwrap().channels[0];
    base_channel.defaults.context_window = Field::Custom(4096);
    for m in &mut base_channel.models {
        m.binding = Some(ModelBindingV2::Manual {
            reference: first.clone(),
        });
    }
    base_channel.template_refs = vec![first.clone()];
    let b = base_channel.models[1].clone();
    rehash(&mut base);
    let mut local = input(vec![c(A, &["a"])]);
    local.models.payload.templates = vec![newer];
    let ch = &mut local.models.payload.policy.as_mut().unwrap().channels[0];
    ch.connection = None;
    ch.family_scope = Some(vec![first.template_id.clone()]);
    ch.models[0].binding = Some(ModelBindingV2::Manual {
        reference: second.clone(),
    });
    ch.template_refs = vec![second.clone()];
    ch.family_overrides.insert(
        first.template_id.clone(),
        ModelRules {
            context_window: Field::Custom(8192),
            ..Default::default()
        },
    );
    rehash(&mut local);
    let mut scope = selection(A);
    scope.channels[0].include_connection = false;
    scope.channels[0].family_ids = Some(vec![first.template_id.clone()]);
    let result = overlay_selected(Some(saved(base)), saved(local), scope, None).unwrap();
    let ch = &result.models().payload.policy.as_ref().unwrap().channels[0];
    assert_eq!(ch.defaults.context_window, Field::Custom(4096));
    assert_eq!(ch.models[1], b);
    assert_eq!(ch.models[0].binding.as_ref().unwrap().reference(), &second);
    assert_eq!(
        ch.family_overrides[&first.template_id].context_window,
        Field::Custom(8192)
    );
    assert!(ch.family_scope.is_none());
}
struct HostAuthorization;
impl file_objects_v2::AuthorizationValidator for HostAuthorization {
    fn validate(
        &self,
        _: &PortableChannelV2,
        auth: &super::super::provider_auth::PortableAuthorization,
    ) -> ModelResult<()> {
        super::super::provider_auth::validate_portable(
            &json!({"baseUrl":"https://fixture.example.test/v1","adapter":"openai"}),
            auth,
        )
    }
}
fn with_auth(value: &str) -> CapturedObjects {
    let mut input = input(vec![c(A, &["a"])]);
    let authorization =
        serde_json::from_value(json!({"activeId":null,"fields":{"headers":{"X-Fixture":value}}}))
            .unwrap();
    input.channel_authorizations.insert(A.into(), authorization);
    input
}
#[test]
fn channel_authorization_is_independent_and_never_switches_forms_implicitly() {
    let base = SavedObjects::capture(with_auth("old-secret"), Some(&HostAuthorization)).unwrap();
    let local = SavedObjects::capture(with_auth("new-secret"), Some(&HostAuthorization)).unwrap();
    let result =
        overlay_selected(Some(base), local, selection(A), Some(&HostAuthorization)).unwrap();
    let auth = serde_json::to_value(&result.partitions().channel_authorizations[A]).unwrap();
    assert_eq!(auth["fields"]["headers"]["X-Fixture"], "old-secret");
    let base = SavedObjects::capture(with_auth("old-secret"), Some(&HostAuthorization)).unwrap();
    let local = SavedObjects::capture(with_auth("new-secret"), Some(&HostAuthorization)).unwrap();
    let mut scope = selection(A);
    scope.channels[0].include_authorization = true;
    let result = overlay_selected(Some(base), local, scope, Some(&HostAuthorization)).unwrap();
    let auth = serde_json::to_value(&result.partitions().channel_authorizations[A]).unwrap();
    assert_eq!(auth["fields"]["headers"]["X-Fixture"], "new-secret");
    let base = SavedObjects::capture(with_auth("old-secret"), Some(&HostAuthorization)).unwrap();
    let mut source = input(vec![c(A, &["a"])]);
    source
        .channel_keys
        .insert(A.into(), Zeroizing::new("local-key".into()))
        .unwrap();
    let mut scope = selection(A);
    scope.channels[0].include_key = true;
    assert_eq!(
        code(overlay_selected(
            Some(base),
            saved(source),
            scope,
            Some(&HostAuthorization)
        )),
        "overlay_credential_conflict"
    );
}
#[test]
fn explicit_but_missing_preferences_keys_credentials_and_sources_are_errors() {
    let mut scope = SelectionV2::default();
    scope.manager.preferences = true;
    assert_eq!(
        code(overlay_selected(None, saved(input(vec![])), scope, None)),
        "overlay_selection_missing"
    );
    let mut scope = SelectionV2::default();
    scope.manager.extensions = true;
    assert_eq!(
        code(overlay_selected(None, saved(input(vec![])), scope, None)),
        "overlay_selection_missing"
    );
    let mut scope = selection(A);
    scope.channels[0].include_key = true;
    assert_eq!(
        code(overlay_selected(
            None,
            saved(input(vec![c(A, &["a"])])),
            scope,
            None
        )),
        "overlay_selection_missing"
    );
    let mut local = input(vec![]);
    local.assets.mcps.push(mcp("fixture", None));
    let h = mcp_source_digest(&local.assets.mcps[0]).unwrap();
    let scope = SelectionV2 {
        mcp_sources: vec![McpSourceSelectionV2 {
            name: "fixture".into(),
            definition_digest: h,
        }],
        extension_credentials: ExtensionCredentialsV2 {
            mcp_names: vec!["fixture".into()],
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(
        code(overlay_selected(None, saved(local), scope, None)),
        "overlay_selection_missing"
    );
    let scope = SelectionV2 {
        extension_credentials: ExtensionCredentialsV2 {
            skill_names: vec!["unselected".into()],
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(
        code(overlay_selected(None, saved(input(vec![])), scope, None)),
        "overlay_scope_invalid"
    );
}
#[test]
fn explicit_global_references_preserve_unselected_channel_order_and_default() {
    let (old, newer, first, second) = template_pair();
    let mut base = input(vec![c(A, &["a"]), c(B, &["b"])]);
    base.models.payload.templates = vec![old];
    let policy = base.models.payload.policy.as_mut().unwrap();
    policy.template_refs = vec![first];
    policy.default_channel_id = Some(B.into());
    rehash(&mut base);
    let mut local = input(vec![c(A, &["a"])]);
    local.models.payload.templates = vec![newer];
    let policy = local.models.payload.policy.as_mut().unwrap();
    policy.template_refs = vec![second.clone()];
    policy.default_channel_id = Some(A.into());
    rehash(&mut local);
    let scope = SelectionV2 {
        include_global_template_refs: true,
        ..Default::default()
    };
    let result = overlay_selected(Some(saved(base)), saved(local), scope, None).unwrap();
    let policy = result.models().payload.policy.as_ref().unwrap();
    assert_eq!(policy.channel_order, vec![A, B]);
    assert_eq!(policy.default_channel_id.as_deref(), Some(B));
    assert_eq!(policy.template_refs, vec![second]);
    assert_eq!(policy.channels.len(), 2);
}
#[test]
fn save_state_v2_key_and_auth_bind_normalized_channel_without_evidence_noise() {
    let mut input = input(vec![c(A, &["a"])]);
    input
        .channel_keys
        .insert(A.into(), Zeroizing::new("fixture-secret".into()))
        .unwrap();
    let original = saved(input);
    let state = file_objects_v2::save_state(&original).unwrap();
    let key =
        super::super::content_versions::key(&json!(["v2-channelKeys", A, null, null])).unwrap();
    assert!(state.contains_key(&key));
    let mut newer = original.into_captured();
    newer.models.payload.policy.as_mut().unwrap().channels[0].models[0]
        .source
        .fields
        .get_mut("protocol")
        .unwrap()
        .evidence
        .source_digest = schema_v2::digest(b"fresh observation");
    rehash(&mut newer);
    let normalized = saved(newer);
    assert_eq!(state, file_objects_v2::save_state(&normalized).unwrap());
    let old = SavedObjects::capture(with_auth("first-secret"), Some(&HostAuthorization)).unwrap();
    let new = SavedObjects::capture(with_auth("second-secret"), Some(&HostAuthorization)).unwrap();
    let old_state = file_objects_v2::save_state(&old).unwrap();
    let new_state = file_objects_v2::save_state(&new).unwrap();
    let channel =
        super::super::content_versions::key(&json!(["v2-channel", A, null, null])).unwrap();
    let auth =
        super::super::content_versions::key(&json!(["v2-channelAuthorizations", A, null, null]))
            .unwrap();
    assert_eq!(old_state[&channel], new_state[&channel]);
    assert_ne!(old_state[&auth], new_state[&auth]);
}
#[test]
fn partial_scope_does_not_materialize_unselected_implicit_order_or_replace_shared_connection() {
    let mut b = input(vec![c(A, &["a", "b"])]);
    b.models.payload.policy.as_mut().unwrap().channels[0].model_order = vec!["a".into()];
    rehash(&mut b);
    let mut l = input(vec![c(A, &["a"])]);
    l.models.payload.policy.as_mut().unwrap().channels[0].model_subset = Some(vec!["a".into()]);
    rehash(&mut l);
    let mut s = selection(A);
    s.channels[0].model_ids = Some(vec!["a".into()]);
    s.channels[0].include_connection = false;
    let result = overlay_selected(Some(saved(b)), saved(l), s, None).unwrap();
    assert_eq!(
        result.models().payload.policy.as_ref().unwrap().channels[0].model_order,
        vec!["a"]
    );
    let b = input(vec![c(A, &["a", "b"])]);
    let mut l = input(vec![c(A, &["a"])]);
    let ch = &mut l.models.payload.policy.as_mut().unwrap().channels[0];
    ch.model_subset = Some(vec!["a".into()]);
    ch.connection.as_mut().unwrap().auth_mode = Some("none".into());
    rehash(&mut l);
    let mut s = selection(A);
    s.channels[0].model_ids = Some(vec!["a".into()]);
    assert_eq!(
        code(overlay_selected(Some(saved(b)), saved(l), s, None)),
        "overlay_shared_connection_conflict"
    );
}
fn provider_input(subset: bool, path: &str) -> CapturedObjects {
    let mut value = input(vec![c(A, if subset { &["a"] } else { &["a", "b"] })]);
    let channel = &mut value.models.payload.policy.as_mut().unwrap().channels[0];
    if subset {
        channel.model_subset = Some(vec!["a".into()]);
    }
    let connection = channel.connection.as_mut().unwrap();
    connection.nodes = vec![super::super::schema::ServiceNode {
        id: NODE.into(),
        name: "fixture".into(),
        url: "https://fixture.example.test/v1".into(),
    }];
    connection.active_node_id = NODE.into();
    rehash(&mut value);
    let public = json!({"baseUrl":"https://fixture.example.test/v1","adapter":"openai-chat","responsesPath":path,"statelessResponses":true,"models":["a","b"],"selectedModels":["a","b"],"modelAliases":{"a":"alias-a","b":"alias-b"}});
    value.models =
        portable_v2::attach_public_sources_v2(&value.models, &BTreeMap::from([(A.into(), public)]))
            .unwrap();
    value
}
#[test]
fn published_provider_material_is_preserved_in_whole_connection_and_never_spreads_from_subset() {
    let base = saved(provider_input(false, "/old/responses"));
    let local = saved(provider_input(false, "/new/responses"));
    let result = overlay_selected(Some(base), local, selection(A), None).unwrap();
    let public = result.models().payload.policy.as_ref().unwrap().channels[0]
        .connection
        .as_ref()
        .unwrap()
        .provider
        .as_ref()
        .unwrap();
    assert_eq!(public["responsesPath"], "/new/responses");
    assert_eq!(public["statelessResponses"], true);
    assert_eq!(public["modelAliases"]["b"], "alias-b");
    let source = provider_input(true, "/subset/responses");
    let provider = source.models.payload.policy.as_ref().unwrap().channels[0]
        .connection
        .as_ref()
        .unwrap()
        .provider
        .as_ref()
        .unwrap();
    assert!(provider["modelAliases"].get("b").is_none());
    let mut scope = selection(A);
    scope.channels[0].model_ids = Some(vec!["a".into()]);
    assert_eq!(
        code(overlay_selected(
            Some(saved(provider_input(false, "/old/responses"))),
            saved(source),
            scope.clone(),
            None
        )),
        "overlay_shared_connection_conflict"
    );
    scope.channels[0].include_connection = false;
    let result = overlay_selected(
        Some(saved(provider_input(false, "/old/responses"))),
        saved(provider_input(true, "/subset/responses")),
        scope,
        None,
    )
    .unwrap();
    let public = result.models().payload.policy.as_ref().unwrap().channels[0]
        .connection
        .as_ref()
        .unwrap()
        .provider
        .as_ref()
        .unwrap();
    assert_eq!(public["responsesPath"], "/old/responses");
    assert_eq!(public["modelAliases"]["b"], "alias-b");
    let connection = serde_json::to_value(
        &result.models().payload.policy.as_ref().unwrap().channels[0].connection,
    )
    .unwrap();
    assert!(connection.get("protocol").is_none());
}
#[test]
fn provider_gate_rejects_private_material_and_save_atom_binds_public_transport_fields() {
    let base = saved(provider_input(false, "/old"));
    let before = file_objects_v2::save_state(&base).unwrap();
    let changed = saved(provider_input(false, "/new"));
    let after = file_objects_v2::save_state(&changed).unwrap();
    let key = super::super::content_versions::key(&json!(["v2-channel", A, null, null])).unwrap();
    assert_ne!(before[&key], after[&key]);
    let mut private = provider_input(false, "/old");
    private.models.payload.policy.as_mut().unwrap().channels[0]
        .connection
        .as_mut()
        .unwrap()
        .provider
        .as_mut()
        .unwrap()["apiKey"] = json!("synthetic-never-public");
    rehash(&mut private);
    assert!(SavedObjects::capture(private, None).is_err());
}
fn account(id: &str, subject: Option<&str>, slot: &str) -> portable_accounts::AccountInfo {
    portable_accounts::AccountInfo {
        id: id.into(),
        provider: "chatgpt".into(),
        subject: subject.map(str::to_owned),
        label: "same-name".into(),
        email: Some("same@example.test".into()),
        origin: Some(portable_accounts::AccountOrigin {
            version: 1,
            account_platform: "chatgpt".into(),
            owner: "opencodex".into(),
            auth_kind: "oauth".into(),
            source_namespace: "codex-account-pool".into(),
            source_account_id: slot.into(),
            tenant: Some("fixture-tenant".into()),
        }),
    }
}
fn account_selection(id: &str) -> SelectionV2 {
    let mut scope = SelectionV2::default();
    scope.manager.account_ids = vec![id.into()];
    scope
}
#[test]
fn selected_account_information_updates_by_exact_source_subject_and_preserves_unselected_accounts()
{
    let mut b = input(vec![]);
    b.manager.accounts = vec![
        account(A, Some("subject-one"), "old-slot"),
        account(B, Some("subject-two"), "unselected-slot"),
    ];
    let untouched = b.manager.accounts[1].clone();
    let mut local = input(vec![]);
    let mut incoming = account(NODE, Some("subject-one"), "other-device-slot");
    incoming.label = "new-selected-name".into();
    incoming.email = Some("new@example.test".into());
    local.manager.accounts = vec![incoming];
    let result =
        overlay_selected(Some(saved(b)), saved(local), account_selection(NODE), None).unwrap();
    let accounts = &result.partitions().manager.accounts;
    assert_eq!(accounts.len(), 2);
    assert_eq!(accounts[0].id, A);
    assert_eq!(accounts[0].label, "new-selected-name");
    assert_eq!(
        accounts[0].origin.as_ref().unwrap().source_account_id,
        "old-slot"
    );
    assert_eq!(accounts[1], untouched);
    assert!(result.partitions().channel_keys.is_empty());
    assert!(result.partitions().channel_authorizations.is_empty());
    let json = String::from_utf8(
        file_objects_v2::encode_document(&result, None)
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    for key in ["accessToken", "refreshToken", "cookie", "credentialRef"] {
        assert!(!json.contains(&format!("\"{key}\"")));
    }
}
#[test]
fn account_only_selection_works_and_name_or_email_never_associates_different_subjects() {
    let mut b = input(vec![]);
    b.manager.accounts = vec![account(A, Some("first-subject"), "slot")];
    let mut local = input(vec![]);
    local.manager.accounts = vec![account(B, Some("second-subject"), "slot")];
    let result =
        overlay_selected(Some(saved(b)), saved(local), account_selection(B), None).unwrap();
    assert_eq!(result.partitions().manager.accounts.len(), 2);
    assert_eq!(
        result.partitions().manager.accounts[0].subject.as_deref(),
        Some("first-subject")
    );
    assert_eq!(
        result.partitions().manager.accounts[1].subject.as_deref(),
        Some("second-subject")
    );
    assert!(result.models().payload.policy.is_some());
    let mut local = input(vec![]);
    local.manager.accounts = vec![account(A, Some("new-subject"), "slot")];
    let result = overlay_selected(None, saved(local), account_selection(A), None).unwrap();
    assert!(result.models().payload.policy.is_none());
    assert_eq!(result.partitions().manager.accounts.len(), 1);
    assert!(result.partitions().assets.mcps.is_empty());
}
#[test]
fn account_ambiguity_and_legacy_slot_remapping_require_explicit_host_decision() {
    let mut b = input(vec![]);
    let mut legacy = account(B, Some("same-subject"), "legacy-slot");
    legacy.origin = None;
    b.manager.accounts = vec![account(A, Some("same-subject"), "known-slot"), legacy];
    let mut local = input(vec![]);
    local.manager.accounts = vec![account(B, Some("same-subject"), "new-slot")];
    assert_eq!(
        code(overlay_selected(
            Some(saved(b)),
            saved(local),
            account_selection(B),
            None
        )),
        "overlay_account_ambiguous"
    );
    let mut b = input(vec![]);
    b.manager.accounts = vec![account(A, None, "old-slot")];
    let mut local = input(vec![]);
    local.manager.accounts = vec![account(A, None, "new-slot")];
    assert_eq!(
        code(overlay_selected(
            Some(saved(b)),
            saved(local),
            account_selection(A),
            None
        )),
        "overlay_account_mapping_required"
    );
    let mut b = input(vec![]);
    b.manager.accounts = vec![account(A, None, "same-slot")];
    let mut local = input(vec![]);
    local.manager.accounts = vec![account(B, None, "same-slot")];
    let result =
        overlay_selected(Some(saved(b)), saved(local), account_selection(B), None).unwrap();
    assert_eq!(result.partitions().manager.accounts.len(), 2); // no bare-slot inference
}
#[test]
fn missing_unsupported_and_conflicting_account_identities_are_rejected_without_login_payloads() {
    assert_eq!(
        code(overlay_selected(
            None,
            saved(input(vec![])),
            account_selection(A),
            None
        )),
        "overlay_selection_missing"
    );
    let mut local = input(vec![]);
    let mut unsupported = account(A, Some("s"), "slot");
    unsupported.provider = "unsupported-fixture".into();
    unsupported.origin = None;
    local.manager.accounts = vec![unsupported];
    assert_eq!(
        code(overlay_selected(
            None,
            saved(local),
            account_selection(A),
            None
        )),
        "overlay_account_unsupported"
    );
    let mut b = input(vec![]);
    b.manager.accounts = vec![account(A, Some("old-subject"), "slot")];
    let mut local = input(vec![]);
    local.manager.accounts = vec![account(A, Some("different-subject"), "slot")];
    assert_eq!(
        code(overlay_selected(
            Some(saved(b)),
            saved(local),
            account_selection(A),
            None
        )),
        "overlay_account_identity_conflict"
    );
    let mut json = serde_json::to_value(account(A, Some("s"), "slot")).unwrap();
    json["accessToken"] = json!("not-account-info");
    assert!(serde_json::from_value::<portable_accounts::AccountInfo>(json).is_err());
}
#[test]
fn account_save_atoms_ignore_name_for_identity_and_ignore_device_slot_for_known_subject() {
    let mut first = input(vec![]);
    first.manager.accounts = vec![account(A, Some("subject"), "slot-a")];
    let first = saved(first);
    let old = file_objects_v2::save_state(&first).unwrap();
    let mut second = input(vec![]);
    second.manager.accounts = vec![account(B, Some("subject"), "slot-b")];
    let same = file_objects_v2::save_state(&saved(second)).unwrap();
    assert_eq!(old, same);
    let mut named = input(vec![]);
    let mut a = account(B, Some("subject"), "slot-b");
    a.label = "updated-name".into();
    named.manager.accounts = vec![a];
    let changed = file_objects_v2::save_state(&saved(named)).unwrap();
    assert_eq!(
        old.keys().collect::<Vec<_>>(),
        changed.keys().collect::<Vec<_>>()
    );
    assert_ne!(
        old.values().collect::<Vec<_>>(),
        changed.values().collect::<Vec<_>>()
    );
    let mut other = input(vec![]);
    other.manager.accounts = vec![account(B, Some("another-subject"), "slot-b")];
    let different = file_objects_v2::save_state(&saved(other)).unwrap();
    assert_ne!(
        old.keys().collect::<Vec<_>>(),
        different.keys().collect::<Vec<_>>()
    );
}
#[test]
fn selected_account_receipt_uses_same_source_identity_after_baseline_id_preservation() {
    let mut b = input(vec![]);
    b.manager.accounts = vec![account(A, Some("subject"), "cloud-slot")];
    let mut l = input(vec![]);
    let mut info = account(B, Some("subject"), "local-slot");
    info.label = "new-label".into();
    l.manager.accounts = vec![info.clone()];
    let source = saved(l);
    let atoms = file_objects_v2::save_state(&source).unwrap();
    let key = super::super::content_versions::key(&json!([
        "v2-accounts",
        account_source_identity(&info).unwrap()
    ]))
    .unwrap();
    let mut l = source.into_captured();
    l.content_versions = vec![receipt(key.clone(), atoms[&key].clone(), TIME)];
    let result = overlay_selected(Some(saved(b)), saved(l), account_selection(B), None).unwrap();
    assert_eq!(result.partitions().manager.accounts[0].id, A);
    assert_eq!(result.partitions().content_versions[0].scope_key, key);
    assert_eq!(result.partitions().content_versions[0].saved_at, TIME);
    assert_eq!(
        result.partitions().content_versions[0].content_digest,
        file_objects_v2::save_state(&result).unwrap()[&key]
    );
}
