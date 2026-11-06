//! Real v6 portable/schema/source semantics; fixtures are memory-only.
use super::super::{
    portable_v2::{ModelPayloadV2, PortableConnectionV2, PortablePolicyV2},
    schema::{Field, ModelNaming, ServiceNode},
    source_v2::source_from_provider_v2,
};
use super::*;
use serde_json::json;
const A: &str = "11111111-1111-4111-8111-111111111111";
const B: &str = "22222222-2222-4222-8222-222222222222";
const C: &str = "33333333-3333-4333-8333-333333333333";
const NODE: &str = "44444444-4444-4444-8444-444444444444";
const REV: &str = "55555555-5555-4555-8555-555555555555";
fn library() -> LibraryV2 {
    LibraryV2 {
        schema_version: 2,
        templates: vec![],
        submission_receipts: BTreeMap::new(),
    }
}
fn policy() -> PolicyV2 {
    PolicyV2 {
        schema_version: 2,
        channels: vec![],
        template_refs: vec![],
        default_channel_id: None,
        pending_removals: vec![],
        recovery_id: None,
    }
}
fn binding(mode: ReceiveBindingModeV2) -> ReceiveBindingV2 {
    ReceiveBindingV2 {
        portable_channel_id: A.into(),
        local_channel_id: B.into(),
        provider_id: "local-provider".into(),
        mode,
    }
}
fn scope() -> SelectionV2 {
    SelectionV2 {
        channels: vec![ChannelSelectionV2 {
            portable_channel_id: A.into(),
            model_ids: None,
            family_ids: None,
            include_connection: true,
            include_key: false,
            include_authorization: false,
        }],
        ..Default::default()
    }
}
fn model(raw: &str) -> ModelConfigV2 {
    ModelConfigV2 {
        raw_model_id: raw.into(),
        label: format!("label:{raw}"),
        observed: true,
        source: source_from_provider_v2(
            "original/provider",
            raw,
            &json!({"adapter":"openai-chat","modelContextWindows":{raw:32000}}),
            &digest(b"original bytes"),
            SourceKindV2::ImportedFile,
        )
        .unwrap(),
        explicit_overrides: ModelRules::default(),
        binding: None,
    }
}
fn channel(ids: &[&str]) -> PortableChannelV2 {
    PortableChannelV2 {
        id: A.into(),
        label: "incoming".into(),
        connection: Some(PortableConnectionV2 {
            nodes: vec![ServiceNode {
                id: NODE.into(),
                name: "node".into(),
                url: "https://example.test/v1".into(),
            }],
            active_node_id: NODE.into(),
            enabled: true,
            allow_private_network: true,
            auth_mode: Some("key".into()),
            provider: None,
        }),
        model_subset: None,
        family_scope: None,
        models: ids.iter().map(|id| model(id)).collect(),
        template_refs: vec![],
        selection: ModelSelection::Whitelist(ids.iter().map(|id| (*id).into()).collect()),
        model_order: ids.iter().map(|id| (*id).into()).collect(),
        defaults: ModelRules::default(),
        family_overrides: BTreeMap::new(),
        naming: BTreeMap::new(),
        namespace: None,
    }
}
fn package(c: PortableChannelV2) -> PortableModelsV2 {
    let payload = ModelPayloadV2 {
        templates: vec![],
        policy: Some(PortablePolicyV2 {
            template_refs: vec![],
            channel_order: vec![c.id.clone()],
            channels: vec![c],
            default_channel_id: Some(A.into()),
        }),
    };
    PortableModelsV2 {
        format: portable_v2::PORTABLE_FORMAT.into(),
        format_version: 6,
        schema_version: 2,
        created_at: "2026-11-06T12:00:00Z".into(),
        digest: digest(&canonical_bytes(&payload).unwrap()),
        payload,
    }
}
fn rehash(p: &mut PortableModelsV2) {
    p.digest = digest(&canonical_bytes(&p.payload).unwrap());
}
fn pc(p: &mut PortableModelsV2) -> &mut PortableChannelV2 {
    &mut p.payload.policy.as_mut().unwrap().channels[0]
}
fn local(ids: &[&str]) -> PolicyV2 {
    let mut c = empty_channel(&binding(ReceiveBindingModeV2::Existing), "local");
    let p = channel(ids);
    let connection = p.connection.unwrap();
    c.connection.nodes = connection.nodes;
    c.connection.active_node_id = connection.active_node_id;
    c.connection.auth_mode = connection.auth_mode;
    c.connection.credential_ref = Some("local-credential".into());
    c.connection.enabled = true;
    c.connection.source_read_id = Some("local-read".into());
    c.models = p.models;
    c.model_order = p.model_order;
    c.selection = p.selection;
    let mut out = policy();
    out.channels.push(c);
    out.default_channel_id = Some(B.into());
    out
}
fn template() -> TemplateV2 {
    let mut t = seed_library_v2().unwrap().templates.remove(0);
    let mut r = t.revisions.remove(0);
    r.content.common_rules = ModelRules::default();
    r.content.common_rules.protocol = Field::Custom("google".into());
    r.content.condition_overrides.clear();
    r.content.locked_fields.clear();
    r.digest = content_digest(&r.saved_label, &r.content).unwrap();
    t.default_revision = Some(r.id.clone());
    t.revisions = vec![r];
    t
}
fn reference(t: &TemplateV2) -> RefV2 {
    let r = &t.revisions[0];
    RefV2 {
        template_id: t.id.clone(),
        revision_id: r.id.clone(),
        digest: r.digest.clone(),
    }
}
fn run(p: &PortableModelsV2) -> ReceiveModelsPlanV2 {
    plan(
        &library(),
        &policy(),
        p,
        &scope(),
        &[binding(ReceiveBindingModeV2::New)],
    )
    .unwrap()
}
fn code(r: Result<ReceiveModelsPlanV2>) -> String {
    r.unwrap_err().code
}
#[test]
fn fresh_identity_remap_preserves_original_source_evidence_and_inputs() {
    let p = package(channel(&["vendor/model", "model-b"]));
    let l = library();
    let saved = policy();
    let s = scope();
    let bs = vec![binding(ReceiveBindingModeV2::New)];
    let before = (l.clone(), saved.clone(), p.clone(), s.clone(), bs.clone());
    let result = plan(&l, &saved, &p, &s, &bs).unwrap();
    assert_eq!(before, (l, saved, p.clone(), s, bs));
    assert_eq!(result.source_to_target[A], B);
    assert_eq!(result.changed_channel_ids, vec![B]);
    let c = &result.policy.channels[0];
    assert_eq!(c.id, B);
    assert_eq!(c.connection.provider_id, "local-provider");
    assert!(!c.connection.enabled);
    assert!(!c.connection.allow_private_network);
    assert!(c.connection.credential_ref.is_none());
    assert!(c.connection.source_read_id.is_none());
    assert!(c.connection.imported_config.is_none());
    assert!(c.connection.auth_mode.is_none());
    let old = &p.payload.policy.as_ref().unwrap().channels[0].models[0];
    for (key, field) in &c.models[0].source.fields {
        let mut expected = old.source.fields[key].clone();
        expected.evidence.kind = SourceKindV2::PortableImport;
        assert_eq!(field, &expected);
    }
    assert_eq!(c.models[0].raw_model_id, "vendor/model");
    assert_eq!(result, run(&p));
}
#[test]
fn manual_and_auto_refs_stay_exact_and_provider_adapter_never_overrides_protocol() {
    let t = template();
    let r = reference(&t);
    let mut p = package(channel(&["manual", "auto"]));
    p.payload.templates.push(t);
    pc(&mut p).models[0].binding = Some(ModelBindingV2::Manual {
        reference: r.clone(),
    });
    pc(&mut p).models[1].binding = Some(ModelBindingV2::AutoRead { reference: r });
    pc(&mut p).models[0].explicit_overrides.protocol = Field::Custom("anthropic".into());
    pc(&mut p).connection.as_mut().unwrap().provider = Some(json!({"adapter":"openai-responses"}));
    rehash(&mut p);
    let result = run(&p);
    let c = &result.policy.channels[0];
    assert_eq!(
        c.models[0].binding,
        p.payload.policy.as_ref().unwrap().channels[0].models[0].binding
    );
    assert!(matches!(
        c.models[1].binding,
        Some(ModelBindingV2::AutoRead { .. })
    ));
    let a = resolve_effective_model_v2(&result.library, c, &c.models[0]).unwrap();
    let b = resolve_effective_model_v2(&result.library, c, &c.models[1]).unwrap();
    assert!(matches!(
        a.protocol,
        ProtocolStateV2::Resolved {
            value: ProtocolV2::Anthropic,
            ..
        }
    ));
    assert!(matches!(
        b.protocol,
        ProtocolStateV2::Resolved {
            value: ProtocolV2::Google,
            ..
        }
    ));
    assert_eq!(result.public_sources[B]["adapter"], "openai-responses");
}
#[test]
fn partial_preserves_unselected_facts_selection_order_refs_and_connection() {
    let mut saved = local(&["a", "keep", "b", "other"]);
    saved.channels[0].models[1]
        .explicit_overrides
        .context_window = Field::Custom(64000);
    saved.channels[0].naming.insert(
        "keep".into(),
        ModelNaming {
            alias: Some("keep-alias".into()),
            display_name: None,
        },
    );
    saved.channels[0].naming.insert(
        "a".into(),
        ModelNaming {
            alias: Some("obsolete".into()),
            display_name: None,
        },
    );
    let mut c = channel(&["a", "b"]);
    c.model_subset = Some(vec!["a".into(), "b".into()]);
    c.model_order = vec!["b".into(), "a".into()];
    c.selection = ModelSelection::Whitelist(vec!["b".into()]);
    let p = package(c);
    let mut s = scope();
    s.channels[0].model_ids = Some(vec!["a".into(), "b".into()]);
    s.channels[0].include_connection = false;
    let result = plan(
        &library(),
        &saved,
        &p,
        &s,
        &[binding(ReceiveBindingModeV2::Existing)],
    )
    .unwrap();
    let c = &result.policy.channels[0];
    assert_eq!(c.connection, saved.channels[0].connection);
    assert_eq!(c.models[1], saved.channels[0].models[1]);
    assert_eq!(c.models[3], saved.channels[0].models[3]);
    assert_eq!(c.model_order, vec!["b", "keep", "a", "other"]);
    assert_eq!(
        c.selection,
        ModelSelection::Whitelist(vec!["keep".into(), "b".into(), "other".into()])
    );
    assert_eq!(c.naming.get("keep"), saved.channels[0].naming.get("keep"));
    assert!(!c.naming.contains_key("a"));
    assert_eq!(c.label, "local");
    assert_eq!(result.policy.default_channel_id, saved.default_channel_id);
}
#[test]
fn partial_all_cannot_silently_deselect_or_expand_other_models() {
    let mut saved = local(&["a", "keep"]);
    saved.channels[0].selection = ModelSelection::All;
    let mut p = package(channel(&["a"]));
    pc(&mut p).model_subset = Some(vec!["a".into()]);
    pc(&mut p).selection = ModelSelection::None;
    rehash(&mut p);
    let mut s = scope();
    s.channels[0].model_ids = Some(vec!["a".into()]);
    s.channels[0].include_connection = false;
    assert_eq!(
        code(plan(
            &library(),
            &saved,
            &p,
            &s,
            &[binding(ReceiveBindingModeV2::Existing)]
        )),
        "overlay_model_selection_conflict"
    );
}
#[test]
fn selection_none_is_not_all() {
    let mut p = package(channel(&["a"]));
    pc(&mut p).selection = ModelSelection::None;
    rehash(&mut p);
    assert_eq!(run(&p).policy.channels[0].selection, ModelSelection::None);
}
#[test]
fn explicit_model_subset_of_whole_package_does_not_import_extra_models() {
    let p = package(channel(&["a", "b"]));
    let mut s = scope();
    s.channels[0].model_ids = Some(vec!["a".into()]);
    let r = plan(
        &library(),
        &policy(),
        &p,
        &s,
        &[binding(ReceiveBindingModeV2::New)],
    )
    .unwrap();
    assert_eq!(r.policy.channels[0].models.len(), 1);
    assert_eq!(r.policy.channels[0].model_order, vec!["a"]);
    assert_eq!(
        r.policy.channels[0].selection,
        ModelSelection::Whitelist(vec!["a".into()])
    );
}
#[test]
fn selection_missing_ids_empty_and_whole_on_subset_are_rejected() {
    let mut p = package(channel(&["a"]));
    pc(&mut p).model_subset = Some(vec!["a".into()]);
    rehash(&mut p);
    for ids in [
        None,
        Some(vec![]),
        Some(vec!["a".into(), "missing".into()]),
        Some(vec!["a".into(), "a".into()]),
    ] {
        let mut s = scope();
        s.channels[0].model_ids = ids;
        assert!(plan(
            &library(),
            &policy(),
            &p,
            &s,
            &[binding(ReceiveBindingModeV2::New)]
        )
        .is_err());
    }
}
#[test]
fn family_fragment_is_rules_only_and_cannot_create_connection() {
    let t = template();
    let r = reference(&t);
    let id = t.id.clone();
    let mut c = channel(&["a"]);
    c.connection = None;
    c.family_scope = Some(vec![id.clone()]);
    c.models[0].binding = Some(ModelBindingV2::AutoRead { reference: r });
    c.family_overrides.insert(id.clone(), ModelRules::default());
    let mut p = package(c);
    p.payload.templates.push(t);
    rehash(&mut p);
    let mut s = scope();
    s.channels[0].family_ids = Some(vec![id]);
    s.channels[0].include_connection = false;
    let r = plan(
        &library(),
        &policy(),
        &p,
        &s,
        &[binding(ReceiveBindingModeV2::New)],
    )
    .unwrap();
    let c = &r.policy.channels[0];
    assert!(c.connection.nodes.is_empty());
    assert!(!c.connection.enabled);
    assert!(r.public_sources.is_empty());
    s.channels[0].include_connection = true;
    assert!(plan(
        &library(),
        &policy(),
        &p,
        &s,
        &[binding(ReceiveBindingModeV2::New)]
    )
    .is_err());
}
#[test]
fn new_nonfamily_without_connection_selection_is_rejected() {
    let p = package(channel(&["a"]));
    let mut s = scope();
    s.channels[0].include_connection = false;
    assert_eq!(
        code(plan(
            &library(),
            &policy(),
            &p,
            &s,
            &[binding(ReceiveBindingModeV2::New)]
        )),
        "overlay_connection_required"
    );
}
#[test]
fn exact_library_merge_accepts_new_revision_but_rejects_immutable_collision() {
    let t = template();
    let mut l = library();
    l.templates.push(t.clone());
    let mut p = package(channel(&["a"]));
    let mut incoming = t.clone();
    let mut r = t.revisions[0].clone();
    r.id = REV.into();
    r.saved_label = "new".into();
    r.digest = content_digest(&r.saved_label, &r.content).unwrap();
    let mut s = scope();
    s.template_refs = vec![
        reference(&t),
        RefV2 {
            template_id: t.id.clone(),
            revision_id: r.id.clone(),
            digest: r.digest.clone(),
        },
    ];
    incoming.default_revision = Some(r.id.clone());
    incoming.revisions.push(r);
    p.payload.templates.push(incoming);
    rehash(&mut p);
    let result = plan(&l, &policy(), &p, &s, &[binding(ReceiveBindingModeV2::New)]).unwrap();
    assert_eq!(result.library.templates[0].revisions.len(), 2);
    assert_eq!(
        result.library.templates[0].default_revision,
        t.default_revision
    );
    assert_eq!(result.changed_template_refs.len(), 1);
    p.payload.templates[0].revisions[0].created_at = "2026-11-05T00:00:00Z".into();
    rehash(&mut p);
    assert_eq!(
        code(plan(
            &l,
            &policy(),
            &p,
            &s,
            &[binding(ReceiveBindingModeV2::New)]
        )),
        "model_revision_digest_mismatch"
    );
}
#[test]
fn defaults_require_exact_explicit_incoming_saved_default() {
    let t = template();
    let mut p = package(channel(&["a"]));
    p.payload.templates.push(t.clone());
    rehash(&mut p);
    let mut selected = scope();
    selected.template_refs = vec![reference(&t)];
    assert!(plan(
        &library(),
        &policy(),
        &p,
        &selected,
        &[binding(ReceiveBindingModeV2::New)]
    )
    .unwrap()
    .library
    .templates[0]
        .default_revision
        .is_none());
    let mut s = scope();
    s.adopt_template_defaults = vec![reference(&t)];
    let r = plan(
        &library(),
        &policy(),
        &p,
        &s,
        &[binding(ReceiveBindingModeV2::New)],
    )
    .unwrap();
    assert_eq!(r.library.templates[0].default_revision, t.default_revision);
    let mut l = library();
    let mut old = t.clone();
    old.default_revision = None;
    l.templates.push(old);
    assert!(plan(
        &l,
        &policy(),
        &p,
        &scope(),
        &[binding(ReceiveBindingModeV2::New)]
    )
    .unwrap()
    .library
    .templates[0]
        .default_revision
        .is_none());
    p.payload.templates[0].default_revision = None;
    rehash(&mut p);
    assert_eq!(
        code(plan(
            &l,
            &policy(),
            &p,
            &s,
            &[binding(ReceiveBindingModeV2::New)]
        )),
        "overlay_default_source_changed"
    );
}
#[test]
fn namespace_is_rejected_even_when_channel_unselected() {
    let mut p = package(channel(&["a"]));
    pc(&mut p).namespace = Some(NamespaceV2 {
        channel_id: A.into(),
        remove_leading_prefixes: vec!["prefix/".into()],
    });
    rehash(&mut p);
    assert_eq!(
        code(plan(
            &library(),
            &policy(),
            &p,
            &SelectionV2::default(),
            &[]
        )),
        "model_receive_namespace_adoption_required"
    );
}
#[test]
fn endpoint_change_requires_explicit_key_or_authorization_and_returns_only_mapping() {
    let saved = local(&["a"]);
    let mut p = package(channel(&["a"]));
    pc(&mut p).connection.as_mut().unwrap().nodes[0].url = "https://different.test".into();
    rehash(&mut p);
    assert_eq!(
        code(plan(
            &library(),
            &saved,
            &p,
            &scope(),
            &[binding(ReceiveBindingModeV2::Existing)]
        )),
        "model_receive_credentials_direction"
    );
    for kind in [
        ReceiveCredentialKindV2::Key,
        ReceiveCredentialKindV2::Authorization,
    ] {
        let mut s = scope();
        s.channels[0].include_key = kind == ReceiveCredentialKindV2::Key;
        s.channels[0].include_authorization = kind == ReceiveCredentialKindV2::Authorization;
        let r = plan(
            &library(),
            &saved,
            &p,
            &s,
            &[binding(ReceiveBindingModeV2::Existing)],
        )
        .unwrap();
        assert_eq!(
            r.credential_actions,
            vec![ReceiveCredentialActionV2 {
                portable_channel_id: A.into(),
                local_channel_id: B.into(),
                provider_id: "local-provider".into(),
                kind
            }]
        );
        assert!(r.policy.channels[0].connection.credential_ref.is_none());
        assert!(r.policy.channels[0].connection.auth_mode.is_none());
    }
}
#[test]
fn key_auth_invalid_combinations_and_missing_connection_are_rejected() {
    let p = package(channel(&["a"]));
    for (connection, key, auth) in [
        (false, true, false),
        (true, true, true),
        (false, false, true),
    ] {
        let mut s = scope();
        s.channels[0].include_connection = connection;
        s.channels[0].include_key = key;
        s.channels[0].include_authorization = auth;
        assert!(plan(
            &library(),
            &policy(),
            &p,
            &s,
            &[binding(ReceiveBindingModeV2::New)]
        )
        .is_err());
    }
    let mut p = p;
    pc(&mut p).connection = None;
    rehash(&mut p);
    assert_eq!(
        code(plan(
            &library(),
            &policy(),
            &p,
            &scope(),
            &[binding(ReceiveBindingModeV2::New)]
        )),
        "overlay_connection_required"
    );
}
#[test]
fn full_public_attachment_keeps_all_public_values_including_nested_extensions() {
    let mut p = package(channel(&["vendor/model"]));
    let public = json!({"adapter":"google","baseUrl":"https://example.test/v1","modelCapabilities":{"vendor/model":{"inputModalities":["text"],"extension":{"opaque":true}}},"responseTierAuthoritative":true,"note":"public material","modelContextTiers":{"vendor/model":[1,2]},"apiKeyTransport":"header"});
    pc(&mut p).connection.as_mut().unwrap().provider = Some(public.clone());
    rehash(&mut p);
    assert_eq!(run(&p).public_sources[B], public);
}
#[test]
fn subset_public_uses_l1_scope_and_exact_slash_ids_without_connection_authority() {
    let mut p = package(channel(&["vendor/model", "other"]));
    pc(&mut p).connection.as_mut().unwrap().provider = Some(
        json!({"adapter":"google","baseUrl":"https://example.test/v1","models":["vendor/model","other"],"modelCapabilities":{"vendor/model":{"inputModalities":["text"],"opaque":true},"other":{"opaque":false}},"modelAliases":{"vendor/model":"alias","other":"no"},"autoReviewModelOverrides":{"vendor/model":"review","vendor-model":"compat","other":"no"},"note":"shared"}),
    );
    rehash(&mut p);
    let mut s = scope();
    s.channels[0].model_ids = Some(vec!["vendor/model".into()]);
    s.channels[0].include_connection = false;
    let saved = local(&["vendor/model", "other"]);
    let r = plan(
        &library(),
        &saved,
        &p,
        &s,
        &[binding(ReceiveBindingModeV2::Existing)],
    )
    .unwrap();
    let public = &r.public_sources[B];
    assert!(public.get("adapter").is_none());
    assert!(public.get("baseUrl").is_none());
    assert!(public.get("note").is_none());
    assert_eq!(public["models"], json!(["vendor/model"]));
    assert_eq!(public["modelCapabilities"]["vendor/model"]["opaque"], true);
    assert!(public["modelAliases"].get("other").is_none());
    assert_eq!(public["autoReviewModelOverrides"]["vendor/model"], "review");
    assert_eq!(public["autoReviewModelOverrides"]["vendor-model"], "compat");
    assert_eq!(
        r.policy.channels[0].connection,
        saved.channels[0].connection
    );
}
#[test]
fn unknown_schema_format_digest_and_missing_exact_refs_are_rejected() {
    let p = package(channel(&["a"]));
    for version in [1, 5, 7] {
        let mut p = p.clone();
        p.format_version = version;
        assert_eq!(
            code(plan(
                &library(),
                &policy(),
                &p,
                &scope(),
                &[binding(ReceiveBindingModeV2::New)]
            )),
            "model_schema_unsupported"
        );
    }
    let mut wrong = p.clone();
    wrong.schema_version = 99;
    assert_eq!(
        code(plan(
            &library(),
            &policy(),
            &wrong,
            &scope(),
            &[binding(ReceiveBindingModeV2::New)]
        )),
        "model_schema_unsupported"
    );
    let mut wrong = p.clone();
    wrong.digest = digest(b"wrong");
    assert_eq!(
        code(plan(
            &library(),
            &policy(),
            &wrong,
            &scope(),
            &[binding(ReceiveBindingModeV2::New)]
        )),
        "model_revision_digest_mismatch"
    );
    let mut wrong = p;
    pc(&mut wrong).models[0].binding = Some(ModelBindingV2::Manual {
        reference: RefV2 {
            template_id: C.into(),
            revision_id: REV.into(),
            digest: digest(b"missing"),
        },
    });
    rehash(&mut wrong);
    assert_eq!(
        code(plan(
            &library(),
            &policy(),
            &wrong,
            &scope(),
            &[binding(ReceiveBindingModeV2::New)]
        )),
        "model_dependency_missing"
    );
}
#[test]
fn validation_covers_unselected_channels_and_public_unknown_fields() {
    let mut p = package(channel(&["a"]));
    pc(&mut p).connection.as_mut().unwrap().provider = Some(json!({"unknownField":true}));
    rehash(&mut p);
    assert!(plan(&library(), &policy(), &p, &SelectionV2::default(), &[]).is_err());
}
#[test]
fn binding_uuid_uniqueness_target_existence_provider_and_label_collisions() {
    let p = package(channel(&["a"]));
    let saved = local(&["a"]);
    for mut b in [
        binding(ReceiveBindingModeV2::Existing),
        binding(ReceiveBindingModeV2::New),
    ] {
        b.local_channel_id = C.into();
        assert!(plan(&library(), &saved, &p, &scope(), &[b]).is_err());
    }
    let mut b = binding(ReceiveBindingModeV2::New);
    b.local_channel_id = "not-a-uuid".into();
    assert!(plan(&library(), &policy(), &p, &scope(), &[b]).is_err());
    assert!(plan(
        &library(),
        &policy(),
        &p,
        &scope(),
        &[
            binding(ReceiveBindingModeV2::New),
            binding(ReceiveBindingModeV2::New)
        ]
    )
    .is_err());
    let mut saved = saved;
    saved.channels[0].label = "incoming".into();
    let mut b = binding(ReceiveBindingModeV2::New);
    b.local_channel_id = C.into();
    b.provider_id = "fresh".into();
    assert_eq!(
        code(plan(&library(), &saved, &p, &scope(), &[b])),
        "model_receive_identity_conflict"
    );
    let mut b = binding(ReceiveBindingModeV2::Existing);
    b.provider_id = "wrong".into();
    assert!(plan(&library(), &saved, &p, &scope(), &[b]).is_err());
}
#[test]
fn global_refs_order_and_default_change_only_with_explicit_selection() {
    let t = template();
    let r = reference(&t);
    let mut l = library();
    l.templates.push(t.clone());
    let mut saved = local(&["a"]);
    let mut second = saved.channels[0].clone();
    second.id = C.into();
    second.label = "other".into();
    second.connection.provider_id = "other".into();
    saved.channels.push(second);
    saved.default_channel_id = Some(C.into());
    let mut p = package(channel(&["a"]));
    p.payload.templates.push(t);
    p.payload.policy.as_mut().unwrap().template_refs = vec![r.clone()];
    rehash(&mut p);
    let mut s = scope();
    s.channels[0].include_connection = false;
    let out = plan(
        &l,
        &saved,
        &p,
        &s,
        &[binding(ReceiveBindingModeV2::Existing)],
    )
    .unwrap();
    assert!(out.policy.template_refs.is_empty());
    assert_eq!(out.policy.default_channel_id, Some(C.into()));
    s.include_global_template_refs = true;
    let out = plan(
        &l,
        &saved,
        &p,
        &s,
        &[binding(ReceiveBindingModeV2::Existing)],
    )
    .unwrap();
    assert_eq!(out.policy.template_refs, vec![r]);
    assert_eq!(out.policy.default_channel_id, Some(B.into()));
    assert_eq!(out.policy.channels[1], saved.channels[1]);
}
#[test]
fn input_values_remain_unchanged_after_late_error() {
    let l = library();
    let saved = local(&["a"]);
    let mut p = package(channel(&["a"]));
    pc(&mut p).connection.as_mut().unwrap().nodes[0].url = "https://changed.test".into();
    rehash(&mut p);
    let s = scope();
    let bs = vec![binding(ReceiveBindingModeV2::Existing)];
    let before = (l.clone(), saved.clone(), p.clone(), s.clone(), bs.clone());
    assert!(plan(&l, &saved, &p, &s, &bs).is_err());
    assert_eq!(before, (l, saved, p, s, bs));
}

#[test]
fn empty_receive_scope_does_not_expand_to_package_templates_or_channels() {
    let mut p = package(channel(&["a"]));
    p.payload.templates.push(template());
    rehash(&mut p);
    let result = plan(&library(), &policy(), &p, &SelectionV2::default(), &[]).unwrap();
    assert_eq!(result.library, library());
    assert_eq!(result.policy, policy());
    assert!(result.source_to_target.is_empty());
    assert!(result.changed_channel_ids.is_empty());
    assert!(result.changed_template_refs.is_empty());
}
#[test]
fn rule_only_missing_family_dependency_is_rejected_outside_selection() {
    let mut p = package(channel(&["a"]));
    pc(&mut p)
        .family_overrides
        .insert(C.into(), ModelRules::default());
    rehash(&mut p);
    assert_eq!(
        code(plan(
            &library(),
            &policy(),
            &p,
            &SelectionV2::default(),
            &[]
        )),
        "model_dependency_missing"
    );
}
#[test]
fn full_attachment_preserves_the_entire_public_field_contract() {
    let fields = super::super::provider_fields::PUBLIC;
    let mut public = Value::Object(fields.iter().map(|k| ((*k).into(), Value::Null)).collect());
    public["adapter"] = json!("openai-responses");
    public["baseUrl"] = json!("https://example.test/v1");
    let mut p = package(channel(&["a"]));
    pc(&mut p).connection.as_mut().unwrap().provider = Some(public.clone());
    rehash(&mut p);
    let result = run(&p);
    assert_eq!(result.public_sources[B], public);
    assert_eq!(
        result.public_sources[B].as_object().unwrap().len(),
        fields.len()
    );
}
#[test]
fn partial_shared_rules_conflict_is_explicit_and_unselected_refs_survive() {
    let t = template();
    let r = reference(&t);
    let mut l = library();
    l.templates.push(t.clone());
    let mut saved = local(&["a", "keep"]);
    saved.channels[0].template_refs.push(r.clone());
    saved.channels[0]
        .family_overrides
        .insert(t.id.clone(), ModelRules::default());
    let p = package(channel(&["a"]));
    let mut s = scope();
    s.channels[0].model_ids = Some(vec!["a".into()]);
    s.channels[0].include_connection = false;
    let out = plan(
        &l,
        &saved,
        &p,
        &s,
        &[binding(ReceiveBindingModeV2::Existing)],
    )
    .unwrap();
    assert_eq!(out.policy.channels[0].template_refs, vec![r]);
    assert_eq!(
        out.policy.channels[0].family_overrides,
        saved.channels[0].family_overrides
    );
    let mut p = p;
    pc(&mut p).defaults.context_window = Field::Custom(64000);
    rehash(&mut p);
    assert_eq!(
        code(plan(
            &l,
            &saved,
            &p,
            &s,
            &[binding(ReceiveBindingModeV2::Existing)]
        )),
        "overlay_shared_rules_conflict"
    );
}
#[test]
fn family_merge_preserves_unselected_models_defaults_and_connection() {
    let t = template();
    let r = reference(&t);
    let mut l = library();
    l.templates.push(t.clone());
    let mut saved = local(&["a", "keep"]);
    saved.channels[0].defaults.context_window = Field::Custom(64000);
    let mut c = channel(&["a"]);
    c.connection = None;
    c.family_scope = Some(vec![t.id.clone()]);
    c.models[0].binding = Some(ModelBindingV2::Manual { reference: r });
    c.family_overrides.insert(
        t.id.clone(),
        ModelRules {
            fast: Field::Custom(true),
            ..Default::default()
        },
    );
    let mut p = package(c);
    p.payload.templates.push(t.clone());
    rehash(&mut p);
    let mut s = scope();
    s.channels[0].family_ids = Some(vec![t.id]);
    s.channels[0].include_connection = false;
    let out = plan(
        &l,
        &saved,
        &p,
        &s,
        &[binding(ReceiveBindingModeV2::Existing)],
    )
    .unwrap();
    assert_eq!(
        out.policy.channels[0].models[1],
        saved.channels[0].models[1]
    );
    assert_eq!(
        out.policy.channels[0].connection,
        saved.channels[0].connection
    );
    assert_eq!(out.policy.channels[0].defaults, saved.channels[0].defaults);
    assert!(out.public_sources.is_empty());
}
#[test]
fn channel_order_uses_remapped_ids_and_keeps_unselected_slots() {
    let mut saved = local(&["a"]);
    let mut keep = saved.channels[0].clone();
    keep.id = NODE.into();
    keep.label = "keep".into();
    keep.connection.provider_id = "keep-provider".into();
    let mut last = saved.channels[0].clone();
    last.id = C.into();
    last.label = "last".into();
    last.connection.provider_id = "last-provider".into();
    saved.channels.extend([keep.clone(), last]);
    let mut p = package(channel(&["a"]));
    let mut other = channel(&["b"]);
    other.id = REV.into();
    other.label = "last incoming".into();
    let pp = p.payload.policy.as_mut().unwrap();
    pp.channels.push(other);
    pp.channel_order = vec![REV.into(), A.into()];
    rehash(&mut p);
    let mut s = scope();
    s.channels[0].include_connection = false;
    let mut other = s.channels[0].clone();
    other.portable_channel_id = REV.into();
    s.channels.push(other);
    let bs = vec![
        binding(ReceiveBindingModeV2::Existing),
        ReceiveBindingV2 {
            portable_channel_id: REV.into(),
            local_channel_id: C.into(),
            provider_id: "last-provider".into(),
            mode: ReceiveBindingModeV2::Existing,
        },
    ];
    let out = plan(&library(), &saved, &p, &s, &bs).unwrap();
    assert_eq!(
        out.policy
            .channels
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>(),
        vec![B, NODE, C]
    );
    s.include_global_template_refs = true;
    let out = plan(&library(), &saved, &p, &s, &bs).unwrap();
    assert_eq!(
        out.policy
            .channels
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>(),
        vec![C, NODE, B]
    );
    assert_eq!(out.policy.channels[1], keep);
}
#[test]
fn exact_binding_digest_and_nondefault_adoption_are_rejected() {
    let t = template();
    let mut p = package(channel(&["a"]));
    p.payload.templates.push(t.clone());
    let mut r = reference(&t);
    r.digest = digest(b"incorrect");
    pc(&mut p).models[0].binding = Some(ModelBindingV2::AutoRead { reference: r });
    rehash(&mut p);
    assert_eq!(
        code(plan(
            &library(),
            &policy(),
            &p,
            &scope(),
            &[binding(ReceiveBindingModeV2::New)]
        )),
        "model_revision_digest_mismatch"
    );
    pc(&mut p).models[0].binding = None;
    let mut alternate = t.revisions[0].clone();
    alternate.id = REV.into();
    p.payload.templates[0].revisions.push(alternate.clone());
    rehash(&mut p);
    let mut s = scope();
    s.adopt_template_defaults = vec![RefV2 {
        template_id: t.id,
        revision_id: alternate.id,
        digest: alternate.digest,
    }];
    assert_eq!(
        code(plan(
            &library(),
            &policy(),
            &p,
            &s,
            &[binding(ReceiveBindingModeV2::New)]
        )),
        "overlay_default_source_changed"
    );
}
