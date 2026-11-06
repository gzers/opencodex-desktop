//! Frozen public model IDs and synthetic observations only; no filesystem/network tests.
use super::{
    matching_v2::*,
    portable_v2::*,
    schema::{Field, ModelRules, ModelSelection},
    schema_v2::*,
    source_v2::*,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
const CHANNEL: &str = "11111111-1111-4111-8111-111111111111";
const DRAFT: &str = "22222222-2222-4222-8222-222222222222";
const READ: &str = "33333333-3333-4333-8333-333333333333";
fn source(id: &str, protocol: &str) -> SourceModelV2 {
    source_from_provider_v2(
        "fixture",
        id,
        &json!({"adapter":protocol}),
        &digest(b"fixture"),
        SourceKindV2::ImportedFile,
    )
    .unwrap()
}
fn channel(ids: &[&str]) -> ChannelV2 {
    ChannelV2 {
        id: CHANNEL.into(),
        label: "fixture".into(),
        connection: ConnectionV2 {
            provider_id: "fixture".into(),
            nodes: vec![],
            active_node_id: String::new(),
            credential_ref: None,
            enabled: true,
            allow_private_network: false,
            auth_mode: Some("key".into()),
            source_read_id: None,
            imported_config: None,
        },
        selection: ModelSelection::Whitelist(ids.iter().map(|s| (*s).into()).collect()),
        model_order: ids.iter().map(|s| (*s).into()).collect(),
        models: ids
            .iter()
            .map(|id| ModelConfigV2 {
                raw_model_id: (*id).into(),
                label: (*id).into(),
                observed: true,
                source: source(id, "openai-chat"),
                explicit_overrides: ModelRules::default(),
                binding: None,
            })
            .collect(),
        template_refs: vec![],
        defaults: ModelRules::default(),
        family_overrides: BTreeMap::new(),
        naming: BTreeMap::new(),
        namespace: None,
    }
}
fn policy(ids: &[&str]) -> PolicyV2 {
    PolicyV2 {
        schema_version: 2,
        channels: vec![channel(ids)],
        template_refs: vec![],
        default_channel_id: Some(CHANNEL.into()),
        pending_removals: vec![],
        recovery_id: None,
    }
}
fn base(library: &LibraryV2) -> ReadBaseV2 {
    ReadBaseV2 {
        dto_version: 2,
        draft_id: DRAFT.into(),
        draft_generation: 7,
        source_digest: digest(b"source"),
        policy_digest: digest(b"policy"),
        library_digest: digest(&canonical_bytes(library).unwrap()),
        root_revision: "fixture-root-r1".into(),
    }
}
fn observation(library: &LibraryV2, ids: &[&str]) -> ReadChannelObservationV2 {
    ReadChannelObservationV2 {
        read_id: READ.into(),
        base: base(library),
        channel_id: CHANNEL.into(),
        models: ids.iter().map(|id| source(id, "openai-chat")).collect(),
    }
}
fn pin(library: &LibraryV2, policy: &mut PolicyV2, id: &str, manual: bool) {
    let c = &policy.channels[0];
    let reference = select_reference_v2(library, policy, c, id)
        .unwrap()
        .unwrap();
    policy.channels[0]
        .models
        .iter_mut()
        .find(|m| m.raw_model_id == id)
        .unwrap()
        .binding = Some(if manual {
        ModelBindingV2::Manual { reference }
    } else {
        ModelBindingV2::AutoRead { reference }
    });
}
fn refresh_revision(library: &mut LibraryV2, index: usize) {
    let r = &mut library.templates[index].revisions[0];
    r.digest = content_digest(&r.saved_label, &r.content).unwrap();
}
#[test]
fn seed_is_real_v2_and_history_is_not_relabelled() {
    let _: LibraryV2 = serde_json::from_str(include_str!("seed_v2.json")).unwrap();
    let library = seed_library_v2().unwrap();
    assert_eq!(library.templates.len(), 17);
    assert_eq!(
        library
            .templates
            .iter()
            .map(|t| t.revisions[0].content.condition_overrides.len())
            .sum::<usize>(),
        58
    );
    let mut wrong = library.clone();
    wrong.schema_version = 1;
    assert_eq!(
        validate_library_v2(&wrong).unwrap_err().code,
        "model_schema_unsupported"
    );
    let mut tamper = library.clone();
    tamper.templates[0].revisions[0]
        .content
        .description
        .push('x');
    assert_eq!(
        validate_library_v2(&tamper).unwrap_err().code,
        "model_revision_digest_mismatch"
    );
}
#[test]
fn frozen_246_rows_keep_133_ids_and_79_known_hits() {
    let library = seed_library_v2().unwrap();
    let data: Value = serde_json::from_str(FROZEN_ROWS).unwrap();
    let keys: Value = serde_json::from_str(FAMILY_KEYS).unwrap();
    let rows = data.as_array().unwrap();
    assert_eq!(rows.len(), 246);
    let mut unique = BTreeSet::new();
    let mut known = 0;
    for row in rows {
        let id = row["rawModelId"].as_str().unwrap();
        let mut policy = policy(&[id]);
        if row["provider"] == "GZGG-NewApi" {
            policy.channels[0].namespace = Some(NamespaceV2 {
                channel_id: CHANNEL.into(),
                remove_leading_prefixes: serde_json::from_str(NAMESPACE_PREFIXES).unwrap(),
            });
        }
        let c = &policy.channels[0];
        let reference = select_reference_v2(&library, &policy, c, id).unwrap();
        let family = reference
            .as_ref()
            .and_then(|r| keys.get(&r.template_id))
            .and_then(Value::as_str);
        assert_eq!(family, row["family"].as_str(), "{id}");
        assert_eq!(
            matching_view_v2(c, id).unwrap(),
            row["matchingView"].as_str().unwrap()
        );
        if let Some(reference) = reference {
            unique.insert(id);
            let r = revision(&library, &reference).unwrap();
            let condition = condition_v2(&r.content, &matching_view_v2(c, id).unwrap()).unwrap();
            assert_eq!(
                condition.map(|c| c.id.as_str()),
                row["knownOverrideId"].as_str(),
                "{id}"
            );
            known += usize::from(condition.is_some());
        }
    }
    assert_eq!(unique.len(), 133);
    assert_eq!(known, 79);
}
#[test]
fn all_58_pinned_parameter_groups_and_locks_are_unchanged() {
    let library = seed_library_v2().unwrap();
    let data: Value = serde_json::from_str(PINNED_PARAMETERS).unwrap();
    let mut n = 0;
    for row in data.as_array().unwrap() {
        let template = library
            .templates
            .iter()
            .find(|t| t.id == row["templateId"])
            .unwrap();
        let c = &template.revisions[0].content;
        let condition = condition_v2(c, row["sourceExactId"].as_str().unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(condition.id, row["id"]);
        let rules: ModelRules = serde_json::from_value(row["rules"].clone()).unwrap();
        assert_eq!(condition.rules, rules);
        assert_eq!(
            serde_json::to_value(&condition.locked_fields).unwrap(),
            row["lockedFields"]
        );
        n += 1;
    }
    assert_eq!(n, 58);
}
#[test]
fn frozen_separator_and_family_boundaries() {
    let library = seed_library_v2().unwrap();
    let data: Value = serde_json::from_str(FROZEN_BOUNDARIES).unwrap();
    fn canonical(library: &LibraryV2, id: &str) -> Option<(String, String, Vec<String>, String)> {
        let p = policy(&[id]);
        let r = select_reference_v2(library, &p, &p.channels[0], id).unwrap()?;
        let c = &revision(library, &r).unwrap().content;
        let hits: Vec<_> = c
            .name_layouts
            .iter()
            .filter_map(|l| {
                parse_name_v2(l, id, c.matching.case_sensitive)
                    .map(|p| (l.literal_prefix.to_lowercase(), p))
            })
            .collect();
        (hits.len() == 1).then(|| {
            (
                r.template_id,
                hits[0].0.clone(),
                hits[0].1.components.clone(),
                hits[0].1.tail.clone(),
            )
        })
    }
    for row in data["separator"].as_array().unwrap() {
        let a = canonical(&library, row["a"].as_str().unwrap());
        let b = canonical(&library, row["b"].as_str().unwrap());
        assert_eq!(
            a.is_some() && a == b,
            row["expected"].as_bool().unwrap(),
            "{}",
            row["case"]
        );
    }
    let keys: Value = serde_json::from_str(FAMILY_KEYS).unwrap();
    for row in data["family_boundaries"].as_array().unwrap() {
        let id = row["model"].as_str().unwrap();
        let mut p = policy(&[id]);
        if row["provider"] == "GZGG-NewApi" {
            p.channels[0].namespace = Some(NamespaceV2 {
                channel_id: CHANNEL.into(),
                remove_leading_prefixes: serde_json::from_str(NAMESPACE_PREFIXES).unwrap(),
            });
        }
        let r = select_reference_v2(&library, &p, &p.channels[0], id).unwrap();
        assert_eq!(
            r.as_ref()
                .and_then(|r| keys.get(&r.template_id))
                .and_then(Value::as_str),
            row["expected"].as_str(),
            "{id}"
        );
    }
    assert_eq!(
        data["separator"].as_array().unwrap().len()
            + data["family_boundaries"].as_array().unwrap().len(),
        57
    );
}
#[test]
fn eight_name_position_and_four_capability_spillover_cases() {
    let library = seed_library_v2().unwrap();
    let cases = [
        (
            "Claude",
            "claude-opus-5-5",
            "opus",
            LiteralPositionV2::Any,
            true,
        ),
        (
            "Claude",
            "claude-opus-5-5",
            "opus",
            LiteralPositionV2::BeforeVersion,
            true,
        ),
        (
            "Claude",
            "claude-opus-5-5",
            "opus",
            LiteralPositionV2::AfterVersion,
            false,
        ),
        ("GPT", "gpt-6-sol", "sol", LiteralPositionV2::Any, true),
        (
            "GPT",
            "gpt-6-sol",
            "sol",
            LiteralPositionV2::AfterVersion,
            true,
        ),
        (
            "GPT",
            "gpt-6-sol",
            "sol",
            LiteralPositionV2::BeforeVersion,
            false,
        ),
        ("GPT", "gpt-6-solar", "sol", LiteralPositionV2::Any, false),
        (
            "Claude",
            "claude-opus-5-5",
            "oups",
            LiteralPositionV2::Any,
            false,
        ),
    ];
    for (family, id, word, position, wanted) in cases {
        let c = &library
            .templates
            .iter()
            .find(|t| t.label == family)
            .unwrap()
            .revisions[0]
            .content;
        let (layout, parsed) = c
            .name_layouts
            .iter()
            .find_map(|l| parse_name_v2(l, id, false).map(|p| (l, p)))
            .unwrap();
        let selector = SelectorV2 {
            layout_id: Some(layout.id.clone()),
            literal_tail: Some(parsed.tail),
            all: vec![
                SelectorAtomV2::NumericVersion {
                    components: parsed.components,
                },
                SelectorAtomV2::LiteralSegment {
                    value: word.into(),
                    position,
                    boundary: "hyphen_or_string_edge".into(),
                },
            ],
        };
        assert_eq!(selector_matches_v2(c, &selector, id), wanted, "{id}:{word}");
    }
    let c = &library
        .templates
        .iter()
        .find(|t| t.label == "GPT")
        .unwrap()
        .revisions[0]
        .content;
    for id in [
        "gpt-5.5-mini",
        "gpt-99-sol",
        "gpt-6-sol-mini",
        "gpt-6-sol-20261106",
    ] {
        assert!(condition_v2(c, id).unwrap().is_none(), "{id}");
    }
}
#[test]
fn bad_conditions_unknown_versions_and_overlaps_are_rejected() {
    let library = seed_library_v2().unwrap();
    let mut c = library.templates[0].revisions[0].content.clone();
    c.name_layouts[0].numeric_joiners.push("_".into());
    assert_eq!(
        validate_content_v2(&c).unwrap_err().code,
        "model_selector_invalid"
    );
    let mut c = library.templates[0].revisions[0].content.clone();
    c.condition_overrides[1].selector = c.condition_overrides[0].selector.clone();
    assert_eq!(
        validate_content_v2(&c).unwrap_err().code,
        "model_selector_duplicate"
    );
    let mut c = library.templates[0].revisions[0].content.clone();
    let mut duplicate = c.condition_overrides[0].clone();
    duplicate.id = "44444444-4444-4444-8444-444444444444".into();
    duplicate.selector = SelectorV2 {
        all: vec![SelectorAtomV2::ExactId {
            value: duplicate.source_exact_id.clone(),
        }],
        layout_id: None,
        literal_tail: None,
    };
    c.condition_overrides.push(duplicate);
    assert_eq!(
        condition_v2(&c, "gpt-6-astra").unwrap_err().code,
        "model_condition_overlap"
    );
}
#[test]
fn provider_adapter_is_evidence_and_exact_protocol_never_falls_back() {
    let provider = json!({"adapter":"openai-chat","modelAdapters":{"vendor/a":"openai-responses","other":"future-protocol"},"apiKey":"MUST_NOT_LEAVE_SOURCE"});
    let source = source_from_provider_v2(
        "fixture",
        "vendor/a",
        &provider,
        &digest(b"raw"),
        SourceKindV2::ImportedFile,
    )
    .unwrap();
    assert_eq!(source.fields["protocol"].value, "openai-responses");
    assert!(source.fields["protocol"]
        .evidence
        .pointer
        .ends_with("vendor~1a"));
    assert!(!serde_json::to_string(&source)
        .unwrap()
        .contains("MUST_NOT_LEAVE_SOURCE"));
    let other = source_from_provider_v2(
        "fixture",
        "other",
        &provider,
        &digest(b"raw"),
        SourceKindV2::ImportedFile,
    )
    .unwrap();
    assert_eq!(other.fields["protocol"].value, "future-protocol");
    let fallback = source_from_provider_v2(
        "fixture",
        "unlisted",
        &provider,
        &digest(b"raw"),
        SourceKindV2::ImportedFile,
    )
    .unwrap();
    assert_eq!(
        fallback.fields["protocol"].evidence.pointer,
        "/providers/fixture/adapter"
    );
}
#[test]
fn actual_source_mapping_keeps_negative_and_colon_rules() {
    let s=source_from_provider_v2("fixture","model:tag",&json!({"adapter":"openai-chat","contextWindow":10,"modelContextWindows":{"model":20},"fastEnabled":false,"modelSupportsServiceTier":{"model:tag":true},"noReasoningModels":["model:tag"]}),&digest(b"raw"),SourceKindV2::ImportedFile).unwrap();
    assert_eq!(s.fields["contextWindow"].value, 20);
    assert_eq!(s.fields["fast"].value, false);
    assert_eq!(s.fields["noReasoning"].value, true);
}
#[test]
fn read_matches_into_draft_and_does_not_change_manual_values_or_selection() {
    let library = seed_library_v2().unwrap();
    let mut p = policy(&["gpt-6-sol", "claude-opus-5-5", "unmatched"]);
    pin(&library, &mut p, "claude-opus-5-5", true);
    p.channels[0].models[0].explicit_overrides.fast = Field::Custom(false);
    let original = p.clone();
    let plan = match_read_to_draft_v2(
        &library,
        &p,
        &observation(&library, &["gpt-6-sol", "claude-opus-5-5", "unmatched"]),
    )
    .unwrap();
    assert_eq!(p, original);
    assert_eq!(plan.dto.counts.matched, 1);
    assert_eq!(plan.dto.counts.manual_preserved, 1);
    assert_eq!(plan.dto.counts.unmatched, 1);
    assert_eq!(
        plan.next_policy.channels[0].selection,
        p.channels[0].selection
    );
    assert_eq!(
        plan.next_policy.channels[0].model_order,
        p.channels[0].model_order
    );
    assert_eq!(
        plan.next_policy.channels[0].connection,
        p.channels[0].connection
    );
    assert!(matches!(
        plan.next_policy.channels[0].models[0].binding,
        Some(ModelBindingV2::AutoRead { .. })
    ));
    assert_eq!(
        plan.dto.models[0].effective.rules.fast,
        Field::Custom(false)
    );
    assert_eq!(
        plan.dto.models[0].effective.field_origins["fast"],
        FieldOriginV2::ExplicitModel
    );
    let again = match_read_to_draft_v2(
        &library,
        &plan.next_policy,
        &observation(&library, &["gpt-6-sol", "claude-opus-5-5", "unmatched"]),
    )
    .unwrap();
    assert!(!again.dto.changed);
    assert_eq!(again.dto.counts.changed, 0);
}
#[test]
fn stale_read_empty_and_wrong_library_do_not_apply() {
    let library = seed_library_v2().unwrap();
    let p = policy(&["gpt-6-sol"]);
    let o = observation(&library, &["gpt-6-sol"]);
    let plan = match_read_to_draft_v2(&library, &p, &o).unwrap();
    let mut b = o.base.clone();
    b.draft_generation += 1;
    assert_eq!(
        apply_read_plan_v2(plan, &b, READ).unwrap_err().code,
        "model_read_stale"
    );
    let plan = match_read_to_draft_v2(&library, &p, &o).unwrap();
    assert!(apply_read_plan_v2(plan, &o.base, DRAFT).is_err());
    let mut empty = o.clone();
    empty.models.clear();
    assert_eq!(
        match_read_to_draft_v2(&library, &p, &empty)
            .unwrap_err()
            .code,
        "model_read_empty"
    );
    let mut drift = o;
    drift.base.library_digest = digest(b"changed");
    assert_eq!(
        match_read_to_draft_v2(&library, &p, &drift)
            .unwrap_err()
            .code,
        "model_read_stale"
    );
}
#[test]
fn manual_pin_survives_default_change_and_auto_updates_only_on_read() {
    let mut library = seed_library_v2().unwrap();
    let mut p = policy(&["gpt-6-sol"]);
    pin(&library, &mut p, "gpt-6-sol", true);
    let old = p.channels[0].models[0].binding.clone();
    library.templates[0].default_revision = None;
    assert!(resolve_effective_model_v2(&library, &p.channels[0], &p.channels[0].models[0]).is_ok());
    let plan =
        match_read_to_draft_v2(&library, &p, &observation(&library, &["gpt-6-sol"])).unwrap();
    assert_eq!(plan.next_policy.channels[0].models[0].binding, old);
    let no_default = policy(&["gpt-6-sol"]);
    let plan = match_read_to_draft_v2(
        &library,
        &no_default,
        &observation(&library, &["gpt-6-sol"]),
    )
    .unwrap();
    assert_eq!(plan.dto.counts.missing_default, 1);
    assert!(plan.next_policy.channels[0].models[0].binding.is_none());
}
#[test]
fn equal_priority_conflict_preserves_old_model_and_unrelated_success() {
    let mut library = seed_library_v2().unwrap();
    let mut copy = library.templates[0].clone();
    copy.id = "55555555-5555-4555-8555-555555555555".into();
    copy.revisions[0].id = "66666666-6666-4666-8666-666666666666".into();
    copy.default_revision = Some(copy.revisions[0].id.clone());
    library.templates.push(copy);
    let p = policy(&["gpt-6-sol", "claude-opus-5-5"]);
    let plan = match_read_to_draft_v2(
        &library,
        &p,
        &observation(&library, &["gpt-6-sol", "claude-opus-5-5"]),
    )
    .unwrap();
    assert_eq!(plan.dto.counts.conflict, 1);
    assert_eq!(plan.dto.counts.matched, 1);
    assert_eq!(
        plan.next_policy.channels[0].models[0],
        p.channels[0].models[0]
    );
}
#[test]
fn explicit_intent_equal_value_and_clear_are_not_inferred_from_source() {
    let library = seed_library_v2().unwrap();
    let p = policy(&["unmatched"]);
    let edits = [ModelEditV2::SetOverride {
        channel_id: CHANNEL.into(),
        raw_model_id: "unmatched".into(),
        field: "protocol".into(),
        value: json!("openai-chat"),
    }];
    let custom = apply_explicit_edits_v2(&p, &edits).unwrap();
    assert_ne!(
        policy_semantic_digest_v2(&p).unwrap(),
        policy_semantic_digest_v2(&custom).unwrap()
    );
    let clear = apply_explicit_edits_v2(
        &custom,
        &[ModelEditV2::ClearOverride {
            channel_id: CHANNEL.into(),
            raw_model_id: "unmatched".into(),
            field: "protocol".into(),
        }],
    )
    .unwrap();
    assert_eq!(clear, p);
    let mat = plan_materialization_v2(&library, &p, &custom).unwrap();
    assert!(mat.writes.is_empty());
    assert_eq!(
        custom.channels[0].models[0].source,
        p.channels[0].models[0].source
    );
}
#[test]
fn materialization_clear_restores_source_and_does_not_write_manual_fields() {
    let library = seed_library_v2().unwrap();
    let p = policy(&["gpt-6-sol"]);
    let plan =
        match_read_to_draft_v2(&library, &p, &observation(&library, &["gpt-6-sol"])).unwrap();
    let mat = plan_materialization_v2(&library, &p, &plan.next_policy).unwrap();
    assert!(mat.writes.iter().any(|w| w.field == "contextWindow"));
    assert_eq!(
        plan.next_policy.channels[0].models[0].explicit_overrides,
        ModelRules::default()
    );
    let mut clear = plan.next_policy.clone();
    clear.channels[0].models[0].binding = None;
    let restore = plan_materialization_v2(&library, &plan.next_policy, &clear).unwrap();
    assert!(restore
        .writes
        .iter()
        .any(|w| w.field == "contextWindow" && w.after.is_none()));
}
#[test]
fn ledger_prevents_materialized_protocol_becoming_source() {
    let saved = source("model", "openai-chat");
    let raw = json!({"providers":{"fixture":{"adapter":"openai-chat","modelAdapters":{"model":"openai-responses"}}}});
    let imported = source_from_provider_v2(
        "fixture",
        "model",
        &raw["providers"]["fixture"],
        &digest(b"new"),
        SourceKindV2::ImportedFile,
    )
    .unwrap();
    let ledger = MaterializationLedgerV2 {
        source_policy_digest: digest(b"policy"),
        written_file_digest: digest(b"file"),
        fields: vec![MaterializedFieldV2 {
            channel_id: CHANNEL.into(),
            raw_model_id: "model".into(),
            field: "protocol".into(),
            target_pointer: "/providers/fixture/modelAdapters/model".into(),
            value_digest: digest(&canonical_bytes(&json!("openai-responses")).unwrap()),
            present: true,
        }],
    };
    assert_eq!(
        reconcile_source_v2(
            CHANNEL,
            &saved,
            &imported,
            Some(&ledger),
            &digest(b"policy"),
            &raw
        )
        .unwrap(),
        saved
    );
    assert_eq!(
        reconcile_source_v2(CHANNEL, &saved, &imported, None, &digest(b"policy"), &raw)
            .unwrap_err()
            .code,
        "model_source_provenance_unknown"
    );
    let changed = json!({"providers":{"fixture":{"adapter":"anthropic"}}});
    let imported = source_from_provider_v2(
        "fixture",
        "model",
        &changed["providers"]["fixture"],
        &digest(b"edit"),
        SourceKindV2::ImportedFile,
    )
    .unwrap();
    let s = reconcile_source_v2(
        CHANNEL,
        &saved,
        &imported,
        Some(&ledger),
        &digest(b"policy"),
        &changed,
    )
    .unwrap();
    assert_eq!(s.fields["protocol"].value, "anthropic");
    assert_eq!(
        s.fields["protocol"].evidence.kind,
        SourceKindV2::ExternalFileEdit
    );
}
#[test]
fn groups_keep_chat_responses_distinct_and_pending_selected_blocks() {
    let library = seed_library_v2().unwrap();
    let mut p = policy(&["a", "b"]);
    p.channels[0].models[1].source = source("b", "openai-responses");
    let rows = p.channels[0]
        .models
        .iter()
        .map(|m| resolve_effective_model_v2(&library, &p.channels[0], m).unwrap())
        .collect::<Vec<_>>();
    let groups = plan_protocol_groups_v2(&rows).unwrap();
    assert_eq!(groups.len(), 2);
    assert_ne!(groups[0].protocol, groups[1].protocol);
    p.channels[0].models[0].source.fields.clear();
    let m = resolve_effective_model_v2(&library, &p.channels[0], &p.channels[0].models[0]).unwrap();
    assert_eq!(
        plan_protocol_groups_v2(&[m]).unwrap_err().code,
        "model_protocol_missing"
    );
}
#[test]
fn channel_protocol_custom_is_rejected_not_dropped() {
    let mut p = policy(&["a"]);
    p.channels[0].defaults.protocol = Field::Custom("openai-chat".into());
    assert_eq!(
        validate_policy_v2(&p).unwrap_err().code,
        "model_channel_protocol_forbidden"
    );
    let mut connection = serde_json::to_value(&p.channels[0].connection).unwrap();
    connection["protocol"] = json!("openai-chat");
    assert!(serde_json::from_value::<ConnectionV2>(connection).is_err());
}
#[test]
fn portable_v6_round_trip_is_lossless_and_has_no_local_authority() {
    let library = seed_library_v2().unwrap();
    let mut p = policy(&["gpt-6-sol", "unmatched"]);
    pin(&library, &mut p, "gpt-6-sol", false);
    p.channels[0].connection.credential_ref = Some("LOCAL_SECRET_REFERENCE".into());
    p.channels[0].connection.source_read_id = Some("LOCAL_READ".into());
    let scope = ModelExportScopeV2 {
        channel_ids: [CHANNEL.into()].into(),
        ..Default::default()
    };
    let package = export_portable_models_v2(&library, &p, &scope).unwrap();
    let bytes = encode_portable_models_v2(&package).unwrap();
    assert_eq!(decode_portable_models_v2(&bytes).unwrap(), package);
    assert_eq!(encode_portable_models_v2(&package).unwrap(), bytes);
    let text = String::from_utf8(bytes).unwrap();
    assert!(!text.contains("LOCAL_SECRET_REFERENCE"));
    assert!(!text.contains("LOCAL_READ"));
    assert!(text.contains("auto_read"));
}
#[test]
fn portable_subset_is_exact_and_cannot_smuggle_others_or_versions() {
    let library = seed_library_v2().unwrap();
    let mut p = policy(&["gpt-6-sol", "unmatched"]);
    pin(&library, &mut p, "gpt-6-sol", true);
    let scope = ModelExportScopeV2 {
        channel_models: [(CHANNEL.into(), ["gpt-6-sol".into()].into())].into(),
        ..Default::default()
    };
    let package = export_portable_models_v2(&library, &p, &scope).unwrap();
    assert_eq!(
        package.payload.policy.as_ref().unwrap().channels[0]
            .models
            .len(),
        1
    );
    let mut wrong = serde_json::to_value(&package).unwrap();
    wrong["formatVersion"] = json!(5);
    assert_eq!(
        decode_portable_models_v2(&serde_json::to_vec(&wrong).unwrap())
            .unwrap_err()
            .code,
        "model_schema_unsupported"
    );
    wrong = serde_json::to_value(&package).unwrap();
    wrong["unknown"] = json!(true);
    assert_eq!(
        decode_portable_models_v2(&serde_json::to_vec(&wrong).unwrap())
            .unwrap_err()
            .code,
        "model_unknown_field"
    );
    let mut bad = package.clone();
    bad.payload.templates.clear();
    bad.digest = digest(&canonical_bytes(&bad.payload).unwrap());
    assert_eq!(
        validate_portable_models_v2(&bad).unwrap_err().code,
        "model_dependency_missing"
    );
}
#[test]
fn portable_merge_rejects_same_revision_different_bytes_and_keeps_default() {
    let library = seed_library_v2().unwrap();
    let mut incoming = library.clone();
    incoming.templates[0].revisions[0].created_at = "2026-11-06T00:00:00Z".into();
    assert_eq!(
        merge_library_v2(&library, &incoming).unwrap_err().code,
        "model_revision_digest_mismatch"
    );
    let mut incoming = library.clone();
    incoming.templates[0].default_revision = None;
    assert_eq!(
        merge_library_v2(&library, &incoming).unwrap().templates[0].default_revision,
        library.templates[0].default_revision
    );
}

const FROZEN_ROWS: &str = r#"[{"provider":"A6api","rawModelId":"claude-fable-5","family":"Claude","matchingView":"claude-fable-5","knownOverrideId":null},{"provider":"A6api","rawModelId":"claude-fable-5-1","family":"Claude","matchingView":"claude-fable-5-1","knownOverrideId":"e81e03ce-ca23-4b8b-a124-57394f58bab3"},{"provider":"A6api","rawModelId":"claude-fable-5.1","family":"Claude","matchingView":"claude-fable-5.1","knownOverrideId":"e81e03ce-ca23-4b8b-a124-57394f58bab3"},{"provider":"A6api","rawModelId":"claude-haiku-4-5","family":"Claude","matchingView":"claude-haiku-4-5","knownOverrideId":null},{"provider":"A6api","rawModelId":"claude-haiku-4-5-20251001","family":"Claude","matchingView":"claude-haiku-4-5-20251001","knownOverrideId":null},{"provider":"A6api","rawModelId":"claude-haiku-5-5","family":"Claude","matchingView":"claude-haiku-5-5","knownOverrideId":"f66dd527-dd74-4cb6-8021-6fa8e3d6b913"},{"provider":"A6api","rawModelId":"claude-mythos-5","family":"Claude","matchingView":"claude-mythos-5","knownOverrideId":null},{"provider":"A6api","rawModelId":"claude-opus-4-6","family":"Claude","matchingView":"claude-opus-4-6","knownOverrideId":null},{"provider":"A6api","rawModelId":"claude-opus-4-7","family":"Claude","matchingView":"claude-opus-4-7","knownOverrideId":null},{"provider":"A6api","rawModelId":"claude-opus-4-8","family":"Claude","matchingView":"claude-opus-4-8","knownOverrideId":null},{"provider":"A6api","rawModelId":"claude-opus-5","family":"Claude","matchingView":"claude-opus-5","knownOverrideId":null},{"provider":"A6api","rawModelId":"claude-opus-5-5","family":"Claude","matchingView":"claude-opus-5-5","knownOverrideId":"3d3ca373-350d-4ea7-8dac-fddacfbfd729"},{"provider":"A6api","rawModelId":"claude-sonnet-4-6","family":"Claude","matchingView":"claude-sonnet-4-6","knownOverrideId":null},{"provider":"A6api","rawModelId":"claude-sonnet-5","family":"Claude","matchingView":"claude-sonnet-5","knownOverrideId":null},{"provider":"A6api","rawModelId":"claude-sonnet-5-5","family":"Claude","matchingView":"claude-sonnet-5-5","knownOverrideId":"48dd6394-b195-49e2-b0f1-f58aee3cb33a"},{"provider":"A6api","rawModelId":"codex-auto-review","family":null,"matchingView":"codex-auto-review","knownOverrideId":null},{"provider":"A6api","rawModelId":"deepseek-v3.2","family":"DeepSeek","matchingView":"deepseek-v3.2","knownOverrideId":null},{"provider":"A6api","rawModelId":"deepseek-v4-flash","family":"DeepSeek","matchingView":"deepseek-v4-flash","knownOverrideId":"ab1ab4e2-d784-42aa-9153-4a71823b5e65"},{"provider":"A6api","rawModelId":"deepseek-v4-flash-0731","family":"DeepSeek","matchingView":"deepseek-v4-flash-0731","knownOverrideId":null},{"provider":"A6api","rawModelId":"DeepSeek-V4-Flash-0731","family":"DeepSeek","matchingView":"DeepSeek-V4-Flash-0731","knownOverrideId":null},{"provider":"A6api","rawModelId":"deepseek-v4-flash-vision-exp","family":"DeepSeek","matchingView":"deepseek-v4-flash-vision-exp","knownOverrideId":"99600e39-5ccd-4ebf-86dd-3b6709728b1e"},{"provider":"A6api","rawModelId":"deepseek-v4-pro","family":"DeepSeek","matchingView":"deepseek-v4-pro","knownOverrideId":"2d237fce-d5a2-4169-89bf-a9ee77cb81d5"},{"provider":"A6api","rawModelId":"deepseek-v4-pro-0813","family":"DeepSeek","matchingView":"deepseek-v4-pro-0813","knownOverrideId":null},{"provider":"A6api","rawModelId":"deepseek-v4.1-flash","family":"DeepSeek","matchingView":"deepseek-v4.1-flash","knownOverrideId":null},{"provider":"A6api","rawModelId":"deepseek-v4.1-flash-1017","family":"DeepSeek","matchingView":"deepseek-v4.1-flash-1017","knownOverrideId":null},{"provider":"A6api","rawModelId":"deepseek-v4.1-flash-expires-on-0910","family":"DeepSeek","matchingView":"deepseek-v4.1-flash-expires-on-0910","knownOverrideId":null},{"provider":"A6api","rawModelId":"gemini-2.5-flash","family":"Gemini","matchingView":"gemini-2.5-flash","knownOverrideId":null},{"provider":"A6api","rawModelId":"gemini-2.5-pro","family":"Gemini","matchingView":"gemini-2.5-pro","knownOverrideId":null},{"provider":"A6api","rawModelId":"gemini-3-flash-agent","family":"Gemini","matchingView":"gemini-3-flash-agent","knownOverrideId":null},{"provider":"A6api","rawModelId":"gemini-3-flash-preview","family":"Gemini","matchingView":"gemini-3-flash-preview","knownOverrideId":null},{"provider":"A6api","rawModelId":"gemini-3-pro-image-preview","family":"Gemini","matchingView":"gemini-3-pro-image-preview","knownOverrideId":null},{"provider":"A6api","rawModelId":"gemini-3-pro-preview","family":"Gemini","matchingView":"gemini-3-pro-preview","knownOverrideId":null},{"provider":"A6api","rawModelId":"gemini-3.1-flash-image","family":"Gemini","matchingView":"gemini-3.1-flash-image","knownOverrideId":null},{"provider":"A6api","rawModelId":"gemini-3.1-flash-lite-preview","family":"Gemini","matchingView":"gemini-3.1-flash-lite-preview","knownOverrideId":null},{"provider":"A6api","rawModelId":"gemini-3.1-pro-preview","family":"Gemini","matchingView":"gemini-3.1-pro-preview","knownOverrideId":null},{"provider":"A6api","rawModelId":"gemini-3.1-pro-preview-low","family":"Gemini","matchingView":"gemini-3.1-pro-preview-low","knownOverrideId":null},{"provider":"A6api","rawModelId":"gemini-3.5-flash","family":"Gemini","matchingView":"gemini-3.5-flash","knownOverrideId":"11b12da7-2747-4442-a3f7-db1aefa03950"},{"provider":"A6api","rawModelId":"gemini-3.5-flash-extra-low","family":"Gemini","matchingView":"gemini-3.5-flash-extra-low","knownOverrideId":null},{"provider":"A6api","rawModelId":"gemini-3.5-flash-high","family":"Gemini","matchingView":"gemini-3.5-flash-high","knownOverrideId":null},{"provider":"A6api","rawModelId":"gemini-3.5-flash-lite","family":"Gemini","matchingView":"gemini-3.5-flash-lite","knownOverrideId":"0490058a-caa9-44d4-a4be-d080e0bd90dd"},{"provider":"A6api","rawModelId":"gemini-3.6-flash","family":"Gemini","matchingView":"gemini-3.6-flash","knownOverrideId":"011d7c2f-0a7e-4d6d-bef9-5de00b1ac6d3"},{"provider":"A6api","rawModelId":"gemini-3.7-flash","family":"Gemini","matchingView":"gemini-3.7-flash","knownOverrideId":"22dd7f7b-d3d0-4d50-9079-62a9250ef576"},{"provider":"A6api","rawModelId":"gemini-3.8-flash","family":"Gemini","matchingView":"gemini-3.8-flash","knownOverrideId":"368f5f5a-9197-46ba-bc50-f4df200a6bc5"},{"provider":"A6api","rawModelId":"gemini-4-argon","family":"Gemini","matchingView":"gemini-4-argon","knownOverrideId":null},{"provider":"A6api","rawModelId":"gemini-embedding-2-preview","family":"Gemini","matchingView":"gemini-embedding-2-preview","knownOverrideId":null},{"provider":"A6api","rawModelId":"gemini-omni-1.1-flash","family":"Gemini","matchingView":"gemini-omni-1.1-flash","knownOverrideId":null},{"provider":"A6api","rawModelId":"glm-5","family":"GLM","matchingView":"glm-5","knownOverrideId":"0d721c00-c448-421e-89c2-77d7fc83d637"},{"provider":"A6api","rawModelId":"glm-5.1","family":"GLM","matchingView":"glm-5.1","knownOverrideId":null},{"provider":"A6api","rawModelId":"glm-5.2","family":"GLM","matchingView":"glm-5.2","knownOverrideId":"bf970c87-2e16-47d7-90df-7620da22730e"},{"provider":"A6api","rawModelId":"glm-5.3","family":"GLM","matchingView":"glm-5.3","knownOverrideId":"43358f88-e076-434d-b953-cef5eacf3378"},{"provider":"A6api","rawModelId":"glm-5.3-flash","family":"GLM","matchingView":"glm-5.3-flash","knownOverrideId":"7dd338eb-23fc-4a03-a5a7-51d8745f91e0"},{"provider":"A6api","rawModelId":"glm-5.3-flashx","family":"GLM","matchingView":"glm-5.3-flashx","knownOverrideId":"0b89159c-24cc-41e3-95b5-24f5cb8cd5c5"},{"provider":"A6api","rawModelId":"gpt-4o","family":"GPT","matchingView":"gpt-4o","knownOverrideId":null},{"provider":"A6api","rawModelId":"gpt-4o-mini","family":"GPT","matchingView":"gpt-4o-mini","knownOverrideId":null},{"provider":"A6api","rawModelId":"gpt-5.2","family":"GPT","matchingView":"gpt-5.2","knownOverrideId":null},{"provider":"A6api","rawModelId":"gpt-5.2-chat-latest","family":"GPT","matchingView":"gpt-5.2-chat-latest","knownOverrideId":null},{"provider":"A6api","rawModelId":"gpt-5.3-codex","family":"GPT","matchingView":"gpt-5.3-codex","knownOverrideId":null},{"provider":"A6api","rawModelId":"gpt-5.4","family":"GPT","matchingView":"gpt-5.4","knownOverrideId":null},{"provider":"A6api","rawModelId":"gpt-5.4-mini","family":"GPT","matchingView":"gpt-5.4-mini","knownOverrideId":null},{"provider":"A6api","rawModelId":"gpt-5.5","family":"GPT","matchingView":"gpt-5.5","knownOverrideId":null},{"provider":"A6api","rawModelId":"gpt-5.6","family":"GPT","matchingView":"gpt-5.6","knownOverrideId":null},{"provider":"A6api","rawModelId":"gpt-5.6-luna","family":"GPT","matchingView":"gpt-5.6-luna","knownOverrideId":null},{"provider":"A6api","rawModelId":"gpt-5.6-sol","family":"GPT","matchingView":"gpt-5.6-sol","knownOverrideId":null},{"provider":"A6api","rawModelId":"gpt-5.6-terra","family":"GPT","matchingView":"gpt-5.6-terra","knownOverrideId":null},{"provider":"A6api","rawModelId":"gpt-6-astra","family":"GPT","matchingView":"gpt-6-astra","knownOverrideId":"2660fb7c-dcb3-43c4-bbfe-f5fd2a3e6c9b"},{"provider":"A6api","rawModelId":"gpt-6-luna","family":"GPT","matchingView":"gpt-6-luna","knownOverrideId":"8112514a-ce72-47b8-bb40-c2d6bc22d3a7"},{"provider":"A6api","rawModelId":"gpt-6-sol","family":"GPT","matchingView":"gpt-6-sol","knownOverrideId":"fdcb0aa3-7513-4f79-a5d4-247e233adde5"},{"provider":"A6api","rawModelId":"gpt-6.1-sol","family":"GPT","matchingView":"gpt-6.1-sol","knownOverrideId":"549f12b2-2f58-434d-bf98-814a4935bf20"},{"provider":"A6api","rawModelId":"grok-4.20-0309-non-reasoning","family":"Grok","matchingView":"grok-4.20-0309-non-reasoning","knownOverrideId":null},{"provider":"A6api","rawModelId":"grok-4.20-0309-reasoning","family":"Grok","matchingView":"grok-4.20-0309-reasoning","knownOverrideId":null},{"provider":"A6api","rawModelId":"grok-4.20-multi-agent-0309","family":"Grok","matchingView":"grok-4.20-multi-agent-0309","knownOverrideId":null},{"provider":"A6api","rawModelId":"grok-4.3","family":"Grok","matchingView":"grok-4.3","knownOverrideId":null},{"provider":"A6api","rawModelId":"grok-4.5","family":"Grok","matchingView":"grok-4.5","knownOverrideId":"481e0d93-bbd7-445d-a0e8-27b531793c40"},{"provider":"A6api","rawModelId":"grok-4.6","family":"Grok","matchingView":"grok-4.6","knownOverrideId":"5c274d72-a05c-4ac8-8921-789709869c42"},{"provider":"A6api","rawModelId":"grok-4.7","family":"Grok","matchingView":"grok-4.7","knownOverrideId":"11d1a67d-e5b7-47cb-b829-b829ff1629e1"},{"provider":"A6api","rawModelId":"grok-4.7-build-fast","family":"Grok","matchingView":"grok-4.7-build-fast","knownOverrideId":null},{"provider":"A6api","rawModelId":"grok-build-0.1","family":"Grok","matchingView":"grok-build-0.1","knownOverrideId":null},{"provider":"A6api","rawModelId":"happyhorse-1.1","family":null,"matchingView":"happyhorse-1.1","knownOverrideId":null},{"provider":"A6api","rawModelId":"hy3","family":"HY","matchingView":"hy3","knownOverrideId":"3454b88e-1c52-4283-8348-17a1efce6fec"},{"provider":"A6api","rawModelId":"hy4-preview","family":"HY","matchingView":"hy4-preview","knownOverrideId":"9c25cbe3-f3df-4709-b5da-7ebe32c2a0fe"},{"provider":"A6api","rawModelId":"kimi-k2.5","family":"Kimi","matchingView":"kimi-k2.5","knownOverrideId":null},{"provider":"A6api","rawModelId":"kimi-k2.6","family":"Kimi","matchingView":"kimi-k2.6","knownOverrideId":"2cfc0f2b-68e0-4866-a098-c18a187e9ba9"},{"provider":"A6api","rawModelId":"kimi-k2.7-code","family":"Kimi","matchingView":"kimi-k2.7-code","knownOverrideId":"9b9d91aa-9ec6-429e-8a79-4e5c7cada5fa"},{"provider":"A6api","rawModelId":"kimi-k2.8","family":"Kimi","matchingView":"kimi-k2.8","knownOverrideId":null},{"provider":"A6api","rawModelId":"kimi-k3","family":"Kimi","matchingView":"kimi-k3","knownOverrideId":"40ab2e8a-1089-449a-8401-5d023be6bbbb"},{"provider":"A6api","rawModelId":"kimi-k3.1-pro","family":"Kimi","matchingView":"kimi-k3.1-pro","knownOverrideId":null},{"provider":"A6api","rawModelId":"LongCat-2.0","family":null,"matchingView":"LongCat-2.0","knownOverrideId":null},{"provider":"A6api","rawModelId":"longcat-2.5-preview","family":null,"matchingView":"longcat-2.5-preview","knownOverrideId":null},{"provider":"A6api","rawModelId":"mimo-v2.5","family":"MiMo","matchingView":"mimo-v2.5","knownOverrideId":null},{"provider":"A6api","rawModelId":"mimo-v2.5-pro","family":"MiMo","matchingView":"mimo-v2.5-pro","knownOverrideId":"bf958042-1b3e-4517-a6b9-97916ba1cdaa"},{"provider":"A6api","rawModelId":"mimo-v2.6-flash","family":"MiMo","matchingView":"mimo-v2.6-flash","knownOverrideId":null},{"provider":"A6api","rawModelId":"mimo-v2.6-pro","family":"MiMo","matchingView":"mimo-v2.6-pro","knownOverrideId":null},{"provider":"A6api","rawModelId":"minimax-h3","family":"MiniMax","matchingView":"minimax-h3","knownOverrideId":null},{"provider":"A6api","rawModelId":"minimax-m2.5","family":"MiniMax","matchingView":"minimax-m2.5","knownOverrideId":"01afae6b-4787-4061-bf2f-e96537f62457"},{"provider":"A6api","rawModelId":"minimax-m2.7","family":"MiniMax","matchingView":"minimax-m2.7","knownOverrideId":"b67de1dc-0776-44a2-b16b-48253a120078"},{"provider":"A6api","rawModelId":"minimax-m3","family":"MiniMax","matchingView":"minimax-m3","knownOverrideId":"9218ac88-e227-45f9-babe-36bd736e3771"},{"provider":"A6api","rawModelId":"minimax-m3.1-flash","family":"MiniMax","matchingView":"minimax-m3.1-flash","knownOverrideId":null},{"provider":"A6api","rawModelId":"muse-spark-1.3","family":null,"matchingView":"muse-spark-1.3","knownOverrideId":null},{"provider":"A6api","rawModelId":"muse-wonder","family":null,"matchingView":"muse-wonder","knownOverrideId":null},{"provider":"A6api","rawModelId":"nano-banana","family":null,"matchingView":"nano-banana","knownOverrideId":null},{"provider":"A6api","rawModelId":"nano-banana-2","family":null,"matchingView":"nano-banana-2","knownOverrideId":null},{"provider":"A6api","rawModelId":"nano-banana-2.1","family":null,"matchingView":"nano-banana-2.1","knownOverrideId":null},{"provider":"A6api","rawModelId":"nano-banana-pro","family":null,"matchingView":"nano-banana-pro","knownOverrideId":null},{"provider":"A6api","rawModelId":"qwen3.6-27b","family":"Qwen","matchingView":"qwen3.6-27b","knownOverrideId":null},{"provider":"A6api","rawModelId":"qwen3.7-max","family":"Qwen","matchingView":"qwen3.7-max","knownOverrideId":"942c4345-9565-4b26-b35a-a4620d535ae3"},{"provider":"A6api","rawModelId":"qwen3.7-plus","family":"Qwen","matchingView":"qwen3.7-plus","knownOverrideId":"2e3443a0-69b5-43b7-99f0-7d7a3af734d5"},{"provider":"A6api","rawModelId":"qwen3.8-flash","family":"Qwen","matchingView":"qwen3.8-flash","knownOverrideId":"a73f2d36-7780-449c-8747-d34d6384445a"},{"provider":"A6api","rawModelId":"qwen3.8-max","family":"Qwen","matchingView":"qwen3.8-max","knownOverrideId":"79ee17cc-6320-48ee-a76f-5cc8710ee0dd"},{"provider":"A6api","rawModelId":"qwen3.8-max-preview","family":"Qwen","matchingView":"qwen3.8-max-preview","knownOverrideId":null},{"provider":"A6api","rawModelId":"qwen3.8-omni-flash","family":"Qwen","matchingView":"qwen3.8-omni-flash","knownOverrideId":null},{"provider":"A6api","rawModelId":"step-5-preview","family":"Step","matchingView":"step-5-preview","knownOverrideId":null},{"provider":"A6api","rawModelId":"text-embedding-3-large","family":null,"matchingView":"text-embedding-3-large","knownOverrideId":null},{"provider":"A6api","rawModelId":"text-embedding-3-small","family":null,"matchingView":"text-embedding-3-small","knownOverrideId":null},{"provider":"A6api","rawModelId":"text-embedding-ada-002","family":null,"matchingView":"text-embedding-ada-002","knownOverrideId":null},{"provider":"A6api","rawModelId":"viduq3-pro","family":null,"matchingView":"viduq3-pro","knownOverrideId":null},{"provider":"A6api","rawModelId":"viduq3-turbo","family":null,"matchingView":"viduq3-turbo","knownOverrideId":null},{"provider":"A6api","rawModelId":"doubao-seedream-5","family":null,"matchingView":"doubao-seedream-5","knownOverrideId":null},{"provider":"A6api","rawModelId":"flux.2-max","family":null,"matchingView":"flux.2-max","knownOverrideId":null},{"provider":"A6api","rawModelId":"ideogram-4","family":null,"matchingView":"ideogram-4","knownOverrideId":null},{"provider":"Ciii-1","rawModelId":"codex-auto-review","family":null,"matchingView":"codex-auto-review","knownOverrideId":null},{"provider":"Ciii-1","rawModelId":"gpt-5.5","family":"GPT","matchingView":"gpt-5.5","knownOverrideId":null},{"provider":"Ciii-1","rawModelId":"gpt-5.6","family":"GPT","matchingView":"gpt-5.6","knownOverrideId":null},{"provider":"Ciii-1","rawModelId":"gpt-5.6-luna","family":"GPT","matchingView":"gpt-5.6-luna","knownOverrideId":null},{"provider":"Ciii-1","rawModelId":"gpt-5.6-sol","family":"GPT","matchingView":"gpt-5.6-sol","knownOverrideId":null},{"provider":"Ciii-1","rawModelId":"gpt-5.6-terra","family":"GPT","matchingView":"gpt-5.6-terra","knownOverrideId":null},{"provider":"Ciii-1","rawModelId":"gpt-6-astra","family":"GPT","matchingView":"gpt-6-astra","knownOverrideId":"2660fb7c-dcb3-43c4-bbfe-f5fd2a3e6c9b"},{"provider":"Ciii-1","rawModelId":"gpt-6-sol","family":"GPT","matchingView":"gpt-6-sol","knownOverrideId":"fdcb0aa3-7513-4f79-a5d4-247e233adde5"},{"provider":"Ciii-1","rawModelId":"gpt-6.1-sol","family":"GPT","matchingView":"gpt-6.1-sol","knownOverrideId":"549f12b2-2f58-434d-bf98-814a4935bf20"},{"provider":"Ciii-1","rawModelId":"gpt-reserve","family":"GPT","matchingView":"gpt-reserve","knownOverrideId":null},{"provider":"Ciii-2","rawModelId":"codex-auto-review","family":null,"matchingView":"codex-auto-review","knownOverrideId":null},{"provider":"Ciii-2","rawModelId":"gpt-5.5","family":"GPT","matchingView":"gpt-5.5","knownOverrideId":null},{"provider":"Ciii-2","rawModelId":"gpt-5.6","family":"GPT","matchingView":"gpt-5.6","knownOverrideId":null},{"provider":"Ciii-2","rawModelId":"gpt-5.6-sol","family":"GPT","matchingView":"gpt-5.6-sol","knownOverrideId":null},{"provider":"Ciii-2","rawModelId":"gpt-5.6-terra","family":"GPT","matchingView":"gpt-5.6-terra","knownOverrideId":null},{"provider":"Ciii-2","rawModelId":"gpt-6-astra","family":"GPT","matchingView":"gpt-6-astra","knownOverrideId":"2660fb7c-dcb3-43c4-bbfe-f5fd2a3e6c9b"},{"provider":"Ciii-2","rawModelId":"gpt-6-sol","family":"GPT","matchingView":"gpt-6-sol","knownOverrideId":"fdcb0aa3-7513-4f79-a5d4-247e233adde5"},{"provider":"Ciii-2","rawModelId":"gpt-6.1-sol","family":"GPT","matchingView":"gpt-6.1-sol","knownOverrideId":"549f12b2-2f58-434d-bf98-814a4935bf20"},{"provider":"Ciii-2","rawModelId":"gpt-reserve","family":"GPT","matchingView":"gpt-reserve","knownOverrideId":null},{"provider":"Ciii-2","rawModelId":"grok-4.6","family":"Grok","matchingView":"grok-4.6","knownOverrideId":"5c274d72-a05c-4ac8-8921-789709869c42"},{"provider":"CodeKey","rawModelId":"gpt-6-astra","family":"GPT","matchingView":"gpt-6-astra","knownOverrideId":"2660fb7c-dcb3-43c4-bbfe-f5fd2a3e6c9b"},{"provider":"CodeKey","rawModelId":"gpt-6-sol","family":"GPT","matchingView":"gpt-6-sol","knownOverrideId":"fdcb0aa3-7513-4f79-a5d4-247e233adde5"},{"provider":"CodeKey","rawModelId":"gpt-6.1-sol","family":"GPT","matchingView":"gpt-6.1-sol","knownOverrideId":"549f12b2-2f58-434d-bf98-814a4935bf20"},{"provider":"FeiMao","rawModelId":"gpt-5.5","family":"GPT","matchingView":"gpt-5.5","knownOverrideId":null},{"provider":"FeiMao","rawModelId":"gpt-5.6-sol","family":"GPT","matchingView":"gpt-5.6-sol","knownOverrideId":null},{"provider":"FeiMao","rawModelId":"gpt-5.6-terra","family":"GPT","matchingView":"gpt-5.6-terra","knownOverrideId":null},{"provider":"FeiMao","rawModelId":"gpt-6-astra","family":"GPT","matchingView":"gpt-6-astra","knownOverrideId":"2660fb7c-dcb3-43c4-bbfe-f5fd2a3e6c9b"},{"provider":"FeiMao","rawModelId":"gpt-6-sol","family":"GPT","matchingView":"gpt-6-sol","knownOverrideId":"fdcb0aa3-7513-4f79-a5d4-247e233adde5"},{"provider":"FeiMao","rawModelId":"gpt-6.1-sol","family":"GPT","matchingView":"gpt-6.1-sol","knownOverrideId":"549f12b2-2f58-434d-bf98-814a4935bf20"},{"provider":"FeiMao","rawModelId":"codex-auto-review","family":null,"matchingView":"codex-auto-review","knownOverrideId":null},{"provider":"FeiMao-CNM","rawModelId":"deepseek-v4.1-flash","family":"DeepSeek","matchingView":"deepseek-v4.1-flash","knownOverrideId":null},{"provider":"FeiMao-CNM","rawModelId":"deepseek-v4-flash","family":"DeepSeek","matchingView":"deepseek-v4-flash","knownOverrideId":"ab1ab4e2-d784-42aa-9153-4a71823b5e65"},{"provider":"FeiMao-CNM","rawModelId":"deepseek-v4-pro","family":"DeepSeek","matchingView":"deepseek-v4-pro","knownOverrideId":"2d237fce-d5a2-4169-89bf-a9ee77cb81d5"},{"provider":"FeiMao-CNM","rawModelId":"deepseek-v4.1-flash-expires-on-0910","family":"DeepSeek","matchingView":"deepseek-v4.1-flash-expires-on-0910","knownOverrideId":null},{"provider":"FeiMao-Claude","rawModelId":"claude-fable-5","family":"Claude","matchingView":"claude-fable-5","knownOverrideId":null},{"provider":"FeiMao-Claude","rawModelId":"claude-fable-5-1","family":"Claude","matchingView":"claude-fable-5-1","knownOverrideId":"e81e03ce-ca23-4b8b-a124-57394f58bab3"},{"provider":"FeiMao-Claude","rawModelId":"claude-opus-4-6","family":"Claude","matchingView":"claude-opus-4-6","knownOverrideId":null},{"provider":"FeiMao-Claude","rawModelId":"claude-opus-4-7","family":"Claude","matchingView":"claude-opus-4-7","knownOverrideId":null},{"provider":"FeiMao-Claude","rawModelId":"claude-opus-4-8","family":"Claude","matchingView":"claude-opus-4-8","knownOverrideId":null},{"provider":"FeiMao-Claude","rawModelId":"claude-opus-5","family":"Claude","matchingView":"claude-opus-5","knownOverrideId":null},{"provider":"FeiMao-Claude","rawModelId":"claude-opus-5-5","family":"Claude","matchingView":"claude-opus-5-5","knownOverrideId":"3d3ca373-350d-4ea7-8dac-fddacfbfd729"},{"provider":"FeiMao-Claude","rawModelId":"claude-sonnet-4-6","family":"Claude","matchingView":"claude-sonnet-4-6","knownOverrideId":null},{"provider":"FeiMao-Claude","rawModelId":"claude-sonnet-5","family":"Claude","matchingView":"claude-sonnet-5","knownOverrideId":null},{"provider":"FeiMao-Claude","rawModelId":"claude-sonnet-5-5","family":"Claude","matchingView":"claude-sonnet-5-5","knownOverrideId":"48dd6394-b195-49e2-b0f1-f58aee3cb33a"},{"provider":"GZGG-NewApi","rawModelId":"aixj-gpt-5.4","family":"GPT","matchingView":"gpt-5.4","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"aixj-gpt-5.5","family":"GPT","matchingView":"gpt-5.5","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"btka-kimi-for-coding","family":"Kimi","matchingView":"kimi-for-coding","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"btka-kimi-for-coding-highspeed","family":"Kimi","matchingView":"kimi-for-coding-highspeed","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"btka-kimi-k3","family":"Kimi","matchingView":"kimi-k3","knownOverrideId":"40ab2e8a-1089-449a-8401-5d023be6bbbb"},{"provider":"GZGG-NewApi","rawModelId":"btka-kimi-k3-256k","family":"Kimi","matchingView":"kimi-k3-256k","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"ciii-gpt-5.5","family":"GPT","matchingView":"gpt-5.5","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"ciii-gpt-5.6-luna","family":"GPT","matchingView":"gpt-5.6-luna","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"ciii-gpt-5.6-sol","family":"GPT","matchingView":"gpt-5.6-sol","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"ciii-gpt-gpt-5.6-terra","family":null,"matchingView":"gpt-gpt-5.6-terra","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"ciii-grok-4.5","family":"Grok","matchingView":"grok-4.5","knownOverrideId":"481e0d93-bbd7-445d-a0e8-27b531793c40"},{"provider":"GZGG-NewApi","rawModelId":"gpt-5.4","family":"GPT","matchingView":"gpt-5.4","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"gpt-5.5","family":"GPT","matchingView":"gpt-5.5","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"gxlq-bailian-deepseek-flash","family":"DeepSeek","matchingView":"deepseek-flash","knownOverrideId":"a8618b91-f8c2-42bc-ab0e-1b31ea46106f"},{"provider":"GZGG-NewApi","rawModelId":"gxlq-bailian-deepseek-pro","family":"DeepSeek","matchingView":"deepseek-pro","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"gxlq-bailian-glm","family":"GLM","matchingView":"glm","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"gxlq-bailian-kimi-code","family":"Kimi","matchingView":"kimi-code","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"gxlq-bailian-qwen-flash","family":"Qwen","matchingView":"qwen-flash","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"gxlq-bailian-qwen-max","family":"Qwen","matchingView":"qwen-max","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"gxlq-local-glm-flash","family":"GLM","matchingView":"glm-flash","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"gxlq-local-qwen-flash-next","family":"Qwen","matchingView":"qwen-flash-next","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"gzgg-local-ezio-itx-qwen3.8-27b-q6","family":"Qwen","matchingView":"qwen3.8-27b-q6","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"gzgg-local-ezio-pc-qwen3.8-27b","family":"Qwen","matchingView":"qwen3.8-27b","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"gzgg-local-strata-qwen-flash","family":"Qwen","matchingView":"qwen-flash","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"input-glm-5.2","family":"GLM","matchingView":"glm-5.2","knownOverrideId":"bf970c87-2e16-47d7-90df-7620da22730e"},{"provider":"GZGG-NewApi","rawModelId":"input-grok-4.5","family":"Grok","matchingView":"grok-4.5","knownOverrideId":"481e0d93-bbd7-445d-a0e8-27b531793c40"},{"provider":"GZGG-NewApi","rawModelId":"siliconflow-qwen-2.5-7b-instruct","family":"Qwen","matchingView":"qwen-2.5-7b-instruct","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"siliconflow-qwen-qvq-72b-preview","family":"Qwen","matchingView":"qwen-qvq-72b-preview","knownOverrideId":null},{"provider":"GZGG-NewApi","rawModelId":"gxlq-local-deepseek-flash","family":"DeepSeek","matchingView":"deepseek-flash","knownOverrideId":"a8618b91-f8c2-42bc-ab0e-1b31ea46106f"},{"provider":"Input-1","rawModelId":"codex-auto-review","family":null,"matchingView":"codex-auto-review","knownOverrideId":null},{"provider":"Input-1","rawModelId":"gpt-5.2","family":"GPT","matchingView":"gpt-5.2","knownOverrideId":null},{"provider":"Input-1","rawModelId":"gpt-5.3-codex-spark","family":"GPT","matchingView":"gpt-5.3-codex-spark","knownOverrideId":null},{"provider":"Input-1","rawModelId":"gpt-5.4","family":"GPT","matchingView":"gpt-5.4","knownOverrideId":null},{"provider":"Input-1","rawModelId":"gpt-5.4-mini","family":"GPT","matchingView":"gpt-5.4-mini","knownOverrideId":null},{"provider":"Input-1","rawModelId":"gpt-5.5","family":"GPT","matchingView":"gpt-5.5","knownOverrideId":null},{"provider":"Input-1","rawModelId":"gpt-5.6","family":"GPT","matchingView":"gpt-5.6","knownOverrideId":null},{"provider":"Input-1","rawModelId":"gpt-5.6-luna","family":"GPT","matchingView":"gpt-5.6-luna","knownOverrideId":null},{"provider":"Input-1","rawModelId":"gpt-5.6-sol","family":"GPT","matchingView":"gpt-5.6-sol","knownOverrideId":null},{"provider":"Input-1","rawModelId":"gpt-5.6-terra","family":"GPT","matchingView":"gpt-5.6-terra","knownOverrideId":null},{"provider":"Input-1","rawModelId":"gpt-6","family":"GPT","matchingView":"gpt-6","knownOverrideId":null},{"provider":"Input-1","rawModelId":"gpt-6-astra","family":"GPT","matchingView":"gpt-6-astra","knownOverrideId":"2660fb7c-dcb3-43c4-bbfe-f5fd2a3e6c9b"},{"provider":"Input-1","rawModelId":"gpt-6-luna","family":"GPT","matchingView":"gpt-6-luna","knownOverrideId":"8112514a-ce72-47b8-bb40-c2d6bc22d3a7"},{"provider":"Input-1","rawModelId":"gpt-6-sol","family":"GPT","matchingView":"gpt-6-sol","knownOverrideId":"fdcb0aa3-7513-4f79-a5d4-247e233adde5"},{"provider":"Input-1","rawModelId":"gpt-6.1-sol","family":"GPT","matchingView":"gpt-6.1-sol","knownOverrideId":"549f12b2-2f58-434d-bf98-814a4935bf20"},{"provider":"Input-2","rawModelId":"codex-auto-review","family":null,"matchingView":"codex-auto-review","knownOverrideId":null},{"provider":"Input-2","rawModelId":"gpt-5.2","family":"GPT","matchingView":"gpt-5.2","knownOverrideId":null},{"provider":"Input-2","rawModelId":"gpt-5.3-codex-spark","family":"GPT","matchingView":"gpt-5.3-codex-spark","knownOverrideId":null},{"provider":"Input-2","rawModelId":"gpt-5.4","family":"GPT","matchingView":"gpt-5.4","knownOverrideId":null},{"provider":"Input-2","rawModelId":"gpt-5.4-mini","family":"GPT","matchingView":"gpt-5.4-mini","knownOverrideId":null},{"provider":"Input-2","rawModelId":"gpt-5.5","family":"GPT","matchingView":"gpt-5.5","knownOverrideId":null},{"provider":"Input-2","rawModelId":"gpt-5.6","family":"GPT","matchingView":"gpt-5.6","knownOverrideId":null},{"provider":"Input-2","rawModelId":"gpt-5.6-luna","family":"GPT","matchingView":"gpt-5.6-luna","knownOverrideId":null},{"provider":"Input-2","rawModelId":"gpt-5.6-sol","family":"GPT","matchingView":"gpt-5.6-sol","knownOverrideId":null},{"provider":"Input-2","rawModelId":"gpt-5.6-terra","family":"GPT","matchingView":"gpt-5.6-terra","knownOverrideId":null},{"provider":"Input-2","rawModelId":"gpt-6","family":"GPT","matchingView":"gpt-6","knownOverrideId":null},{"provider":"Input-2","rawModelId":"gpt-6-astra","family":"GPT","matchingView":"gpt-6-astra","knownOverrideId":"2660fb7c-dcb3-43c4-bbfe-f5fd2a3e6c9b"},{"provider":"Input-2","rawModelId":"gpt-6-luna","family":"GPT","matchingView":"gpt-6-luna","knownOverrideId":"8112514a-ce72-47b8-bb40-c2d6bc22d3a7"},{"provider":"Input-2","rawModelId":"gpt-6-sol","family":"GPT","matchingView":"gpt-6-sol","knownOverrideId":"fdcb0aa3-7513-4f79-a5d4-247e233adde5"},{"provider":"Input-2","rawModelId":"gpt-6.1-sol","family":"GPT","matchingView":"gpt-6.1-sol","knownOverrideId":"549f12b2-2f58-434d-bf98-814a4935bf20"},{"provider":"JustDoWoker-1","rawModelId":"claude-opus-4-8-thinking","family":"Claude","matchingView":"claude-opus-4-8-thinking","knownOverrideId":null},{"provider":"JustDoWoker-1","rawModelId":"claude-opus-5","family":"Claude","matchingView":"claude-opus-5","knownOverrideId":null},{"provider":"JustDoWoker-1","rawModelId":"claude-opus-5-thinking","family":"Claude","matchingView":"claude-opus-5-thinking","knownOverrideId":null},{"provider":"JustDoWoker-2","rawModelId":"claude-opus-4-8-thinking","family":"Claude","matchingView":"claude-opus-4-8-thinking","knownOverrideId":null},{"provider":"JustDoWoker-2","rawModelId":"claude-opus-5","family":"Claude","matchingView":"claude-opus-5","knownOverrideId":null},{"provider":"JustDoWoker-2","rawModelId":"claude-opus-5-thinking","family":"Claude","matchingView":"claude-opus-5-thinking","knownOverrideId":null},{"provider":"Krill-1","rawModelId":"codex-auto-review","family":null,"matchingView":"codex-auto-review","knownOverrideId":null},{"provider":"Krill-1","rawModelId":"gpt-5.6-luna","family":"GPT","matchingView":"gpt-5.6-luna","knownOverrideId":null},{"provider":"Krill-1","rawModelId":"gpt-5.6-sol","family":"GPT","matchingView":"gpt-5.6-sol","knownOverrideId":null},{"provider":"Krill-1","rawModelId":"gpt-5.6-terra","family":"GPT","matchingView":"gpt-5.6-terra","knownOverrideId":null},{"provider":"Krill-1","rawModelId":"gpt-6-astra","family":"GPT","matchingView":"gpt-6-astra","knownOverrideId":"2660fb7c-dcb3-43c4-bbfe-f5fd2a3e6c9b"},{"provider":"Krill-1","rawModelId":"gpt-6-sol","family":"GPT","matchingView":"gpt-6-sol","knownOverrideId":"fdcb0aa3-7513-4f79-a5d4-247e233adde5"},{"provider":"Krill-1","rawModelId":"gpt-6.1-sol","family":"GPT","matchingView":"gpt-6.1-sol","knownOverrideId":"549f12b2-2f58-434d-bf98-814a4935bf20"},{"provider":"Krill-1","rawModelId":"gpt-6.1-terra","family":"GPT","matchingView":"gpt-6.1-terra","knownOverrideId":null},{"provider":"Krill-2","rawModelId":"codex-auto-review","family":null,"matchingView":"codex-auto-review","knownOverrideId":null},{"provider":"Krill-2","rawModelId":"gpt-5.6-luna","family":"GPT","matchingView":"gpt-5.6-luna","knownOverrideId":null},{"provider":"Krill-2","rawModelId":"gpt-5.6-sol","family":"GPT","matchingView":"gpt-5.6-sol","knownOverrideId":null},{"provider":"Krill-2","rawModelId":"gpt-5.6-terra","family":"GPT","matchingView":"gpt-5.6-terra","knownOverrideId":null},{"provider":"Krill-2","rawModelId":"gpt-6-astra","family":"GPT","matchingView":"gpt-6-astra","knownOverrideId":"2660fb7c-dcb3-43c4-bbfe-f5fd2a3e6c9b"},{"provider":"Krill-2","rawModelId":"gpt-6-sol","family":"GPT","matchingView":"gpt-6-sol","knownOverrideId":"fdcb0aa3-7513-4f79-a5d4-247e233adde5"},{"provider":"Krill-2","rawModelId":"gpt-6.1-sol","family":"GPT","matchingView":"gpt-6.1-sol","knownOverrideId":"549f12b2-2f58-434d-bf98-814a4935bf20"},{"provider":"MDJK","rawModelId":"gpt-6-astra","family":"GPT","matchingView":"gpt-6-astra","knownOverrideId":"2660fb7c-dcb3-43c4-bbfe-f5fd2a3e6c9b"},{"provider":"MDJK","rawModelId":"gpt-6-sol","family":"GPT","matchingView":"gpt-6-sol","knownOverrideId":"fdcb0aa3-7513-4f79-a5d4-247e233adde5"},{"provider":"MDJK","rawModelId":"gpt-6.1-sol","family":"GPT","matchingView":"gpt-6.1-sol","knownOverrideId":"549f12b2-2f58-434d-bf98-814a4935bf20"}]"#;

const PINNED_PARAMETERS: &str = r#"[{"templateId":"837c8133-76e9-4bdf-a68c-9637331c4d43","id":"2660fb7c-dcb3-43c4-bbfe-f5fd2a3e6c9b","sourceExactId":"gpt-6-astra","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"custom","value":1050000},"inputModalities":{"mode":"custom","value":["text","image"]},"reasoningEfforts":{"mode":"custom","value":["low","medium","high","xhigh","max"]},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"837c8133-76e9-4bdf-a68c-9637331c4d43","id":"549f12b2-2f58-434d-bf98-814a4935bf20","sourceExactId":"gpt-6.1-sol","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"custom","value":1050000},"inputModalities":{"mode":"custom","value":["text","image"]},"reasoningEfforts":{"mode":"custom","value":["low","medium","high","xhigh","max"]},"defaultEffort":{"mode":"custom","value":"medium"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"837c8133-76e9-4bdf-a68c-9637331c4d43","id":"8112514a-ce72-47b8-bb40-c2d6bc22d3a7","sourceExactId":"gpt-6-luna","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"custom","value":1050000},"inputModalities":{"mode":"custom","value":["text","image"]},"reasoningEfforts":{"mode":"custom","value":["none","low","medium","high","xhigh","max"]},"defaultEffort":{"mode":"custom","value":"medium"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"837c8133-76e9-4bdf-a68c-9637331c4d43","id":"fdcb0aa3-7513-4f79-a5d4-247e233adde5","sourceExactId":"gpt-6-sol","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"custom","value":1050000},"inputModalities":{"mode":"custom","value":["text","image"]},"reasoningEfforts":{"mode":"custom","value":["none","low","medium","high","xhigh","max"]},"defaultEffort":{"mode":"custom","value":"medium"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"961df482-3cbe-421d-a268-caa768de2fc6","id":"e81e03ce-ca23-4b8b-a124-57394f58bab3","sourceExactId":"claude-fable-5-1","rules":{"protocol":{"mode":"custom","value":"anthropic"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"custom","value":["text","image"]},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"custom","value":"high"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"961df482-3cbe-421d-a268-caa768de2fc6","id":"3d3ca373-350d-4ea7-8dac-fddacfbfd729","sourceExactId":"claude-opus-5-5","rules":{"protocol":{"mode":"custom","value":"anthropic"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"custom","value":["text","image"]},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"custom","value":"medium"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"961df482-3cbe-421d-a268-caa768de2fc6","id":"48dd6394-b195-49e2-b0f1-f58aee3cb33a","sourceExactId":"claude-sonnet-5-5","rules":{"protocol":{"mode":"custom","value":"anthropic"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"custom","value":["text","image"]},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"custom","value":"high"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"961df482-3cbe-421d-a268-caa768de2fc6","id":"f66dd527-dd74-4cb6-8021-6fa8e3d6b913","sourceExactId":"claude-haiku-5-5","rules":{"protocol":{"mode":"custom","value":"anthropic"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"custom","value":["text","image"]},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"custom","value":"medium"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"e4182850-f6b6-4597-b06a-935a23fc19de","id":"368f5f5a-9197-46ba-bc50-f4df200a6bc5","sourceExactId":"gemini-3.8-flash","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"custom","value":["low","medium","high"]},"defaultEffort":{"mode":"custom","value":"medium"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"e4182850-f6b6-4597-b06a-935a23fc19de","id":"11b12da7-2747-4442-a3f7-db1aefa03950","sourceExactId":"gemini-3.5-flash","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"e4182850-f6b6-4597-b06a-935a23fc19de","id":"011d7c2f-0a7e-4d6d-bef9-5de00b1ac6d3","sourceExactId":"gemini-3.6-flash","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"e4182850-f6b6-4597-b06a-935a23fc19de","id":"22dd7f7b-d3d0-4d50-9079-62a9250ef576","sourceExactId":"gemini-3.7-flash","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"e4182850-f6b6-4597-b06a-935a23fc19de","id":"0490058a-caa9-44d4-a4be-d080e0bd90dd","sourceExactId":"gemini-3.5-flash-lite","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"e93e0572-8a01-499e-b9c1-7f44bbe512ea","id":"43358f88-e076-434d-b953-cef5eacf3378","sourceExactId":"glm-5.3","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"custom","value":["text"]},"reasoningEfforts":{"mode":"custom","value":["low","high","max"]},"defaultEffort":{"mode":"custom","value":"max"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"e93e0572-8a01-499e-b9c1-7f44bbe512ea","id":"7dd338eb-23fc-4a03-a5a7-51d8745f91e0","sourceExactId":"glm-5.3-flash","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"custom","value":["low","high","max"]},"defaultEffort":{"mode":"custom","value":"max"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"e93e0572-8a01-499e-b9c1-7f44bbe512ea","id":"0b89159c-24cc-41e3-95b5-24f5cb8cd5c5","sourceExactId":"glm-5.3-flashx","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"custom","value":["low","high","max"]},"defaultEffort":{"mode":"custom","value":"max"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"e93e0572-8a01-499e-b9c1-7f44bbe512ea","id":"bf970c87-2e16-47d7-90df-7620da22730e","sourceExactId":"glm-5.2","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"custom","value":["text"]},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"e93e0572-8a01-499e-b9c1-7f44bbe512ea","id":"0d721c00-c448-421e-89c2-77d7fc83d637","sourceExactId":"glm-5","rules":{"protocol":{"mode":"custom","value":"openai-chat"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"custom","value":["text"]},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"f42bcbe3-9758-4512-a0e1-fc7809b505b4","id":"3454b88e-1c52-4283-8348-17a1efce6fec","sourceExactId":"hy3","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"custom","value":["text"]},"reasoningEfforts":{"mode":"custom","value":["none","low","high"]},"defaultEffort":{"mode":"custom","value":"high"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"f42bcbe3-9758-4512-a0e1-fc7809b505b4","id":"9c25cbe3-f3df-4709-b5da-7ebe32c2a0fe","sourceExactId":"hy4-preview","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"custom","value":["text"]},"reasoningEfforts":{"mode":"custom","value":["none","high"]},"defaultEffort":{"mode":"custom","value":"high"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"061c63d1-8cfe-4eaa-bceb-37ca808cd7c8","id":"0e768678-4bd2-4a1d-afe7-7f9d42af44a2","sourceExactId":"doubao-seed-2-1-pro-260915","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"061c63d1-8cfe-4eaa-bceb-37ca808cd7c8","id":"c112d90a-263d-47ba-b472-ab2ec50667c9","sourceExactId":"doubao-seed-2-1-lite-260915","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"061c63d1-8cfe-4eaa-bceb-37ca808cd7c8","id":"52868202-0f9c-442e-8e4d-6c08910cd755","sourceExactId":"doubao-seed-2-1-turbo-260628","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"061c63d1-8cfe-4eaa-bceb-37ca808cd7c8","id":"8e888356-a8e3-4145-84b5-4411adc0c6a4","sourceExactId":"doubao-seed-evolving","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"061c63d1-8cfe-4eaa-bceb-37ca808cd7c8","id":"041ae2ab-3cdb-408b-b438-bbe60b3443b1","sourceExactId":"doubao-seed-2-0-lite-260428","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"061c63d1-8cfe-4eaa-bceb-37ca808cd7c8","id":"ed7a3523-71e4-471b-85e8-5c4170d06d43","sourceExactId":"doubao-seed-2-0-mini-260428","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"061c63d1-8cfe-4eaa-bceb-37ca808cd7c8","id":"c80014e9-9ba1-401c-816b-e727d5cd0a1f","sourceExactId":"doubao-seed-2-0-pro-260215","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"061c63d1-8cfe-4eaa-bceb-37ca808cd7c8","id":"976b79f7-1b3f-4863-8d09-03fa41c5426b","sourceExactId":"doubao-seed-2-0-code-preview-260215","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"48b7e309-2ecb-42af-b8c1-e65a883764ab","id":"79ee17cc-6320-48ee-a76f-5cc8710ee0dd","sourceExactId":"qwen3.8-max","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"48b7e309-2ecb-42af-b8c1-e65a883764ab","id":"a73f2d36-7780-449c-8747-d34d6384445a","sourceExactId":"qwen3.8-flash","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"48b7e309-2ecb-42af-b8c1-e65a883764ab","id":"2e3443a0-69b5-43b7-99f0-7d7a3af734d5","sourceExactId":"qwen3.7-plus","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"48b7e309-2ecb-42af-b8c1-e65a883764ab","id":"afb07420-ecdc-4cbc-b212-a3249cdc516a","sourceExactId":"qwen3.7-flash","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"48b7e309-2ecb-42af-b8c1-e65a883764ab","id":"942c4345-9565-4b26-b35a-a4620d535ae3","sourceExactId":"qwen3.7-max","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"48b7e309-2ecb-42af-b8c1-e65a883764ab","id":"8ae29a70-3b2a-4dca-bb51-eb4ac243b9b0","sourceExactId":"qwen3.6-plus","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"48b7e309-2ecb-42af-b8c1-e65a883764ab","id":"706b776d-47bc-49d1-be7a-833c9976ab95","sourceExactId":"qwen3.6-flash","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"48b7e309-2ecb-42af-b8c1-e65a883764ab","id":"9440d40d-c430-4341-acd9-69646feeaf01","sourceExactId":"qwen3.5-plus","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"48b7e309-2ecb-42af-b8c1-e65a883764ab","id":"95264799-64df-49ee-9ba2-374df73d3045","sourceExactId":"qwen3.5-flash","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"48b7e309-2ecb-42af-b8c1-e65a883764ab","id":"f2c007df-f61d-4b6d-b8b0-bab9fcd43e61","sourceExactId":"qwen3-coder-flash","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"ec0b21a6-e038-437c-816e-ce2567c3a264","id":"a8618b91-f8c2-42bc-ab0e-1b31ea46106f","sourceExactId":"deepseek-flash","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"custom","value":["text","image"]},"reasoningEfforts":{"mode":"custom","value":["low","high","max"]},"defaultEffort":{"mode":"custom","value":"high"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"ec0b21a6-e038-437c-816e-ce2567c3a264","id":"2d237fce-d5a2-4169-89bf-a9ee77cb81d5","sourceExactId":"deepseek-v4-pro","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"custom","value":["text"]},"reasoningEfforts":{"mode":"custom","value":["low","high","max"]},"defaultEffort":{"mode":"custom","value":"high"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"ec0b21a6-e038-437c-816e-ce2567c3a264","id":"ab1ab4e2-d784-42aa-9153-4a71823b5e65","sourceExactId":"deepseek-v4-flash","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"ec0b21a6-e038-437c-816e-ce2567c3a264","id":"99600e39-5ccd-4ebf-86dd-3b6709728b1e","sourceExactId":"deepseek-v4-flash-vision-exp","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"3528cd7b-89e4-4b8d-8609-fd294ef7958a","id":"40ab2e8a-1089-449a-8401-5d023be6bbbb","sourceExactId":"kimi-k3","rules":{"protocol":{"mode":"custom","value":"openai-chat"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"custom","value":["text","image"]},"reasoningEfforts":{"mode":"custom","value":["low","high","max"]},"defaultEffort":{"mode":"custom","value":"max"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"3528cd7b-89e4-4b8d-8609-fd294ef7958a","id":"2cfc0f2b-68e0-4866-a098-c18a187e9ba9","sourceExactId":"kimi-k2.6","rules":{"protocol":{"mode":"custom","value":"openai-chat"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"custom","value":["text","image","video"]},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"3528cd7b-89e4-4b8d-8609-fd294ef7958a","id":"9b9d91aa-9ec6-429e-8a79-4e5c7cada5fa","sourceExactId":"kimi-k2.7-code","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"4d612ef5-e45b-4379-b03f-8c5a81954c7e","id":"ba89d514-f427-4464-be5c-7ebf88804536","sourceExactId":"MiniMax-M3.1-Flash-Preview","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"custom","value":1000000},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"custom","value":["low","medium","high","xhigh","max"]},"defaultEffort":{"mode":"custom","value":"max"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"4d612ef5-e45b-4379-b03f-8c5a81954c7e","id":"9218ac88-e227-45f9-babe-36bd736e3771","sourceExactId":"MiniMax-M3","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"custom","value":1000000},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"4d612ef5-e45b-4379-b03f-8c5a81954c7e","id":"b67de1dc-0776-44a2-b16b-48253a120078","sourceExactId":"MiniMax-M2.7","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"custom","value":204800},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"4d612ef5-e45b-4379-b03f-8c5a81954c7e","id":"f3747e24-ef66-4908-ba97-78bd45b5f94f","sourceExactId":"MiniMax-M2.7-highspeed","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"custom","value":204800},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"4d612ef5-e45b-4379-b03f-8c5a81954c7e","id":"01afae6b-4787-4061-bf2f-e96537f62457","sourceExactId":"MiniMax-M2.5","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"custom","value":204800},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"4d612ef5-e45b-4379-b03f-8c5a81954c7e","id":"cc41c98f-53b7-4bd0-847e-01d256a1ec78","sourceExactId":"MiniMax-M2.5-highspeed","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"custom","value":204800},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"c2efe090-d091-418b-aa1e-92c5c5f7eabf","id":"85ea2573-645d-429e-bdfc-f349f807b49b","sourceExactId":"ernie-5.0","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"c2efe090-d091-418b-aa1e-92c5c5f7eabf","id":"aef1641f-d5e3-49dd-a590-1f0f6c44ffd8","sourceExactId":"ernie-5.0-thinking-preview","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"04018546-d99e-430a-afd3-92d8a151472c","id":"82c076c8-e460-4223-bf0f-73964f37bec0","sourceExactId":"step-3.5-flash","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"0bf7408b-2197-4a98-9a9e-5dd218953594","id":"bf958042-1b3e-4517-a6b9-97916ba1cdaa","sourceExactId":"mimo-v2.5-pro","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"ec0d785e-989b-458c-9f2b-35e15874b863","id":"11d1a67d-e5b7-47cb-b829-b829ff1629e1","sourceExactId":"grok-4.7","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"custom","value":500000},"inputModalities":{"mode":"custom","value":["text","image"]},"reasoningEfforts":{"mode":"custom","value":["low","medium","high","xhigh"]},"defaultEffort":{"mode":"custom","value":"high"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"ec0d785e-989b-458c-9f2b-35e15874b863","id":"5c274d72-a05c-4ac8-8921-789709869c42","sourceExactId":"grok-4.6","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]},{"templateId":"ec0d785e-989b-458c-9f2b-35e15874b863","id":"481e0d93-bbd7-445d-a0e8-27b531793c40","sourceExactId":"grok-4.5","rules":{"protocol":{"mode":"inherit"},"contextWindow":{"mode":"inherit"},"inputModalities":{"mode":"inherit"},"reasoningEfforts":{"mode":"inherit"},"defaultEffort":{"mode":"inherit"},"fast":{"mode":"inherit"}},"lockedFields":[]}]"#;

const FROZEN_BOUNDARIES: &str = r#"{"separator":[{"case":"dot_dash_gpt","a":"gpt-5.5","b":"gpt-5-5","expected":true,"actual":true},{"case":"dot_dash_claude","a":"claude-fable-5.1","b":"claude-fable-5-1","expected":true,"actual":true},{"case":"same_spec","a":"gpt-5.5-mini","b":"gpt-5-5-mini","expected":true,"actual":true},{"case":"three_components","a":"gpt-5.5.1","b":"gpt-5-5-1","expected":true,"actual":true},{"case":"same_dated_build","a":"claude-haiku-4.5-20251001","b":"claude-haiku-4-5-20251001","expected":true,"actual":true},{"case":"gpt_minor","a":"gpt-6.1-sol","b":"gpt-6-1-sol","expected":true,"actual":true},{"case":"qwen_version_before_size","a":"qwen3.8-27b","b":"qwen3-8-27b","expected":true,"actual":true},{"case":"deepseek_version","a":"deepseek-v4.1-flash","b":"deepseek-v4-1-flash","expected":true,"actual":true},{"case":"gemini_version","a":"gemini-3.5-flash","b":"gemini-3-5-flash","expected":true,"actual":true},{"case":"glm_version","a":"glm-5.3-flash","b":"glm-5-3-flash","expected":true,"actual":true},{"case":"kimi_version","a":"kimi-k2.7-code","b":"kimi-k2-7-code","expected":true,"actual":true},{"case":"minimax_version","a":"MiniMax-M3.1-Flash","b":"minimax-m3-1-flash","expected":true,"actual":true},{"case":"different_major","a":"gpt-5.5","b":"gpt-15.5","expected":false,"actual":false},{"case":"different_minor","a":"gpt-5.5","b":"gpt-5.50","expected":false,"actual":false},{"case":"patch_not_minor","a":"gpt-5.5","b":"gpt-5.5.1","expected":false,"actual":false},{"case":"dashed_patch_not_minor","a":"gpt-5.5","b":"gpt-5-5-1","expected":false,"actual":false},{"case":"missing_separator","a":"gpt-5.5","b":"gpt-55","expected":false,"actual":false},{"case":"underscore_not_declared","a":"gpt-5.5","b":"gpt-5_5","expected":false,"actual":false},{"case":"space_not_declared","a":"gpt-5.5","b":"gpt-5 5","expected":false,"actual":false},{"case":"any_char_not_separator","a":"gpt-5.5","b":"gpt-5x5","expected":false,"actual":false},{"case":"different_spec","a":"gpt-5.5-mini","b":"gpt-5-5-pro","expected":false,"actual":false},{"case":"suffix_is_not_version","a":"gpt-5.5-mini","b":"gpt-5.5-mini.1","expected":false,"actual":false},{"case":"family_not_substring","a":"gpt-5.5","b":"notgpt-5-5","expected":false,"actual":false},{"case":"channel_prefix_not_implicit","a":"gpt-5.5","b":"vendor-gpt-5-5","expected":false,"actual":false},{"case":"series_not_same","a":"claude-opus-5.5","b":"claude-sonnet-5-5","expected":false,"actual":false},{"case":"family_not_same","a":"gpt-5.5","b":"glm-5-5","expected":false,"actual":false},{"case":"build_not_ignored","a":"claude-haiku-4.5","b":"claude-haiku-4-5-20251001","expected":false,"actual":false},{"case":"date_not_equated","a":"claude-haiku-4.5-20251001","b":"claude-haiku-4-5-20251002","expected":false,"actual":false},{"case":"build_date_delimiters_not_global","a":"claude-haiku-4.5-20251001","b":"claude-haiku-4-5-2025-10-01","expected":false,"actual":false},{"case":"invalid_date_not_build","a":"claude-haiku-4.5","b":"claude-haiku-4-5-20251301","expected":false,"actual":false},{"case":"leading_zero_not_folded","a":"gpt-5.5","b":"gpt-05-5","expected":false,"actual":false},{"case":"version_zero_not_folded","a":"gpt-5.5","b":"gpt-5-05","expected":false,"actual":false},{"case":"date_suffix_ambiguous","a":"grok-4.20","b":"grok-4.20-0309","expected":false,"actual":false},{"case":"ambiguous_date_slots_not_equated","a":"grok-4.20-0309","b":"grok-4-20-0309","expected":false,"actual":false},{"case":"ambiguous_six_digit_build","a":"gpt-5.5-260915","b":"gpt-5-5-260915","expected":false,"actual":false},{"case":"rolling_alias_not_proven","a":"gpt-5.5","b":"gpt-5.5-latest","expected":false,"actual":false},{"case":"tag_not_punctuation_normalized","a":"gpt-5.5-code-mini","b":"gpt-5-5-code.mini","expected":false,"actual":false},{"case":"non_ascii_separator","a":"gpt-5.5","b":"gpt-5–5","expected":false,"actual":false},{"case":"qualified_number","a":"gpt-5.5","b":"gpt-5.5b","expected":false,"actual":false},{"case":"date_in_other_name","a":"gpt-5.5","b":"model-2026-05-05","expected":false,"actual":false},{"case":"review_task_not_model","a":"gpt-5.5","b":"codex-auto-review","expected":false,"actual":false},{"case":"retain_quantization","a":"qwen3.8-27b-q6","b":"qwen3-8-27b-q8","expected":false,"actual":false},{"case":"retain_model_size","a":"qwen3.8-27b","b":"qwen3-8-72b","expected":false,"actual":false},{"case":"unparsed_symbolic_id","a":"gpt-reserve","b":"gpt-reserve","expected":false,"actual":false}],"family_boundaries":[{"provider":"other","model":"notgpt-5.5","expected":null,"actual":null},{"provider":"other","model":"backup-gpt-5.5","expected":null,"actual":null},{"provider":"other","model":"qwenish-3.8","expected":null,"actual":null},{"provider":"other","model":"hypothesis-3","expected":null,"actual":null},{"provider":"other","model":"doubao-seedream-5","expected":null,"actual":null},{"provider":"other","model":"text-embedding-3-large","expected":null,"actual":null},{"provider":"other","model":"claudette-5.5","expected":null,"actual":null},{"provider":"other","model":"aixj-gpt-5.5","expected":null,"actual":null},{"provider":"other","model":"qwen５.５","expected":null,"actual":null},{"provider":"GZGG-NewApi","model":"aixj-gpt-5.5","expected":"GPT","actual":"GPT"},{"provider":"GZGG-NewApi","model":"ciii-gpt-gpt-5.6-terra","expected":null,"actual":null},{"provider":"other","model":"mygemini-3.8","expected":null,"actual":null},{"provider":"other","model":"gemini-embedding-2-preview","expected":"Gemini","actual":"Gemini"}]}"#;

const NAMESPACE_PREFIXES: &str = r#"["aixj-","btka-","ciii-","input-","siliconflow-","gxlq-bailian-","gxlq-local-","gzgg-local-ezio-itx-","gzgg-local-ezio-pc-","gzgg-local-strata-"]"#;

const FAMILY_KEYS: &str = r#"{"837c8133-76e9-4bdf-a68c-9637331c4d43":"GPT","961df482-3cbe-421d-a268-caa768de2fc6":"Claude","e4182850-f6b6-4597-b06a-935a23fc19de":"Gemini","e93e0572-8a01-499e-b9c1-7f44bbe512ea":"GLM","f42bcbe3-9758-4512-a0e1-fc7809b505b4":"HY","061c63d1-8cfe-4eaa-bceb-37ca808cd7c8":"DoubaoSeed","48b7e309-2ecb-42af-b8c1-e65a883764ab":"Qwen","ec0b21a6-e038-437c-816e-ce2567c3a264":"DeepSeek","3528cd7b-89e4-4b8d-8609-fd294ef7958a":"Kimi","4d612ef5-e45b-4379-b03f-8c5a81954c7e":"MiniMax","c2efe090-d091-418b-aa1e-92c5c5f7eabf":"ERNIE","04018546-d99e-430a-afd3-92d8a151472c":"Step","0bf7408b-2197-4a98-9a9e-5dd218953594":"MiMo","ec0d785e-989b-458c-9f2b-35e15874b863":"Grok","5910381f-9f57-4c6c-abc7-5162139fa200":"Mistral","85f3649e-6532-403b-b573-8bf39cc3c9ab":"Gemma","626989ec-73e3-41a8-90e3-810dfb200b73":"Llama"}"#;

const FROZEN_SELECTED: &str = r#"[{"provider":"A6api","ids":["gpt-6.1-sol","gpt-6-sol","gpt-6-luna","gpt-6-astra"]},{"provider":"AgentRouter-1","ids":[]},{"provider":"AgentRouter-2","ids":[]},{"provider":"AgentRouter-3","ids":[]},{"provider":"Ciii-1","ids":["gpt-5.6-luna","gpt-6-astra","gpt-6-sol","gpt-6.1-sol","gpt-reserve"]},{"provider":"Ciii-2","ids":["gpt-6-astra","gpt-6-sol","gpt-6.1-sol","gpt-reserve","grok-4.6"]},{"provider":"CodeKey","ids":[]},{"provider":"FeiMao","ids":["gpt-6-astra","gpt-6-sol","gpt-6.1-sol"]},{"provider":"FeiMao-CNM","ids":["deepseek-v4.1-flash"]},{"provider":"FeiMao-Claude","ids":["claude-fable-5","claude-opus-5-5","claude-sonnet-5-5"]},{"provider":"GZGG-NewApi","ids":["gxlq-local-deepseek-flash","gxlq-local-glm-flash","gxlq-bailian-deepseek-flash","gxlq-bailian-deepseek-pro","gxlq-bailian-qwen-flash","gxlq-bailian-qwen-max","gxlq-bailian-glm","gxlq-bailian-kimi-code","btka-kimi-for-coding","btka-kimi-for-coding-highspeed","btka-kimi-k3","btka-kimi-k3-256k","gzgg-local-strata-qwen-flash","gzgg-local-ezio-itx-qwen3.8-27b-q6","gzgg-local-ezio-pc-qwen3.8-27b"]},{"provider":"Input-1","ids":["gpt-6","gpt-6-astra","gpt-6-luna","gpt-6-sol","gpt-6.1-sol"]},{"provider":"Input-2","ids":["gpt-6","gpt-6-astra","gpt-6-luna","gpt-6-sol","gpt-6.1-sol"]},{"provider":"JustDoWoker-1","ids":[]},{"provider":"JustDoWoker-2","ids":[]},{"provider":"Krill-1","ids":["gpt-6-astra","gpt-6-sol","gpt-6.1-sol","gpt-6.1-terra"]},{"provider":"Krill-2","ids":["gpt-6-astra","gpt-6-sol","gpt-6.1-sol"]},{"provider":"MDJK","ids":[]},{"provider":"Pite","ids":[]},{"provider":"SeekAi-1","ids":[]},{"provider":"SeekAi-2","ids":[]},{"provider":"SeekAi-3","ids":[]},{"provider":"TokEasy","ids":[]},{"provider":"Vsllm","ids":[]},{"provider":"YunShu","ids":[]},{"provider":"openai","ids":[]}]"#;

#[test]
fn immutable_revision_append_rejects_stale_base_and_agent_lock_changes() {
    let library = seed_library_v2().unwrap();
    let t = &library.templates[0];
    let old = &t.revisions[0];
    let mut request = SaveRevisionRequestV2 {
        expected_library: digest(&canonical_bytes(&library).unwrap()),
        template_id: t.id.clone(),
        base_revision: Some(old.id.clone()),
        revision_id: "77777777-7777-4777-8777-777777777777".into(),
        created_at: "2026-11-06T13:00:00Z".into(),
        label: t.label.clone(),
        content: old.content.clone(),
        make_default: false,
        origin: super::schema::SaveOrigin::Manual,
    };
    request.content.description.push_str(" reviewed");
    let next = save_revision_plan_v2(&library, &request).unwrap();
    assert_eq!(next.templates[0].revisions[0], *old);
    assert_eq!(next.templates[0].default_revision, t.default_revision);
    assert_eq!(next.templates[0].revisions.len(), 2);
    assert_eq!(next.templates[0].revisions[1].changes.len(), 1);
    assert_eq!(
        save_revision_plan_v2(&next, &request).unwrap_err().code,
        "model_save_stale"
    );
    let mut locked = library.clone();
    locked.templates[0].revisions[0]
        .content
        .locked_fields
        .insert("description".into());
    refresh_revision(&mut locked, 0);
    request.expected_library = digest(&canonical_bytes(&locked).unwrap());
    request.origin = super::schema::SaveOrigin::InternalAgent("fixture-task".into());
    assert_eq!(
        save_revision_plan_v2(&locked, &request).unwrap_err().code,
        "model_locked_field"
    );
}
#[test]
fn preserved_selected_28_ids_remain_matchable_without_inventory_changes() {
    let library = seed_library_v2().unwrap();
    let fixtures: Value = serde_json::from_str(FROZEN_SELECTED).unwrap();
    let mut selected = BTreeSet::new();
    for provider in fixtures.as_array().unwrap() {
        for id in provider["ids"].as_array().unwrap() {
            let id = id.as_str().unwrap();
            let mut p = policy(&[id]);
            if provider["provider"] == "GZGG-NewApi" {
                p.channels[0].namespace = Some(NamespaceV2 {
                    channel_id: CHANNEL.into(),
                    remove_leading_prefixes: serde_json::from_str(NAMESPACE_PREFIXES).unwrap(),
                });
            }
            assert!(
                select_reference_v2(&library, &p, &p.channels[0], id)
                    .unwrap()
                    .is_some(),
                "{id}"
            );
            selected.insert(id);
        }
    }
    assert_eq!(selected.len(), 28);
}
#[test]
fn newly_observed_unmatched_id_is_counted_changed_without_selecting_it() {
    let library = seed_library_v2().unwrap();
    let p = policy(&["existing"]);
    let plan =
        match_read_to_draft_v2(&library, &p, &observation(&library, &["new-unmatched"])).unwrap();
    assert_eq!(plan.dto.counts.changed, 1);
    assert_eq!(plan.dto.counts.unmatched, 1);
    assert!(!plan.dto.models[0].effective.selected);
    assert_eq!(
        plan.next_policy.channels[0].selection,
        p.channels[0].selection
    );
}
#[test]
fn source_capability_precedence_and_bad_protocol_types_match_original_fields() {
    let s=source_from_provider_v2("fixture","model:tag",&json!({"adapter":"openai-chat","modelInputModalities":{"model:tag":["text"]},"modelCapabilities":{"model:tag":{"inputModalities":["text","image"]}},"noReasoningModels":["model"]}),&digest(b"fixture"),SourceKindV2::ImportedFile).unwrap();
    assert_eq!(s.fields["inputModalities"].value, json!(["text", "image"]));
    assert_eq!(s.fields["noReasoning"].value, true);
    assert!(source_from_provider_v2(
        "fixture",
        "model",
        &json!({"adapter":"openai-chat","modelAdapters":{"model":false}}),
        &digest(b"fixture"),
        SourceKindV2::ImportedFile
    )
    .is_err());
}
#[test]
fn portable_native_and_family_boundary_validation_is_enforced() {
    let library = seed_library_v2().unwrap();
    let mut p = policy(&["gpt-6-sol"]);
    pin(&library, &mut p, "gpt-6-sol", false);
    let family = p.channels[0].models[0]
        .binding
        .as_ref()
        .unwrap()
        .reference()
        .template_id
        .clone();
    let scope = ModelExportScopeV2 {
        channel_families: [(CHANNEL.into(), [family].into())].into(),
        ..Default::default()
    };
    let package = export_portable_models_v2(&library, &p, &scope).unwrap();
    let c = &package.payload.policy.as_ref().unwrap().channels[0];
    assert!(c.connection.is_none());
    assert_eq!(c.defaults, ModelRules::default());
    let mut bad = package.clone();
    bad.payload.policy.as_mut().unwrap().channels[0]
        .defaults
        .fast = Field::Custom(true);
    bad.digest = digest(&canonical_bytes(&bad.payload).unwrap());
    assert!(validate_portable_models_v2(&bad).is_err());
    p.channels[0].connection.provider_id = "openai".into();
    let scope = ModelExportScopeV2 {
        channel_ids: [CHANNEL.into()].into(),
        ..Default::default()
    };
    assert_eq!(
        export_portable_models_v2(&library, &p, &scope)
            .unwrap_err()
            .code,
        "model_native_channel_not_portable"
    );
}
#[test]
fn ledger_pairing_and_connection_credentials_in_urls_are_rejected() {
    let saved = source("model", "openai-chat");
    let ledger = MaterializationLedgerV2 {
        source_policy_digest: digest(b"old"),
        written_file_digest: digest(b"file"),
        fields: vec![],
    };
    assert_eq!(
        reconcile_source_v2(
            CHANNEL,
            &saved,
            &saved,
            Some(&ledger),
            &digest(b"new"),
            &json!({})
        )
        .unwrap_err()
        .code,
        "model_source_provenance_unknown"
    );
    let mut p = policy(&["model"]);
    p.channels[0]
        .connection
        .nodes
        .push(super::schema::ServiceNode {
            id: READ.into(),
            name: "bad".into(),
            url: "https://user:secret@example.invalid".into(),
        });
    p.channels[0].connection.active_node_id = READ.into();
    assert!(validate_policy_v2(&p).is_err());
}
#[test]
fn clearing_invalid_manual_pin_can_plan_repair_without_reinterpreting_old_effective_values() {
    let library = seed_library_v2().unwrap();
    let mut p = policy(&["gpt-6-sol"]);
    pin(&library, &mut p, "gpt-6-sol", true);
    if let Some(ModelBindingV2::Manual { reference }) = &mut p.channels[0].models[0].binding {
        reference.revision_id = READ.into();
    }
    let next = apply_explicit_edits_v2(
        &p,
        &[ModelEditV2::ClearManualBinding {
            channel_id: CHANNEL.into(),
            raw_model_id: "gpt-6-sol".into(),
        }],
    )
    .unwrap();
    let plan = plan_materialization_v2(&library, &p, &next).unwrap();
    assert!(plan.writes.iter().all(|w| !w.before_known));
    assert_eq!(plan.writes.len(), 11);
}

#[test]
fn ledger_finalizer_covers_all_fields_and_auth_binding_cannot_guess() {
    let p = policy(&["model"]);
    let raw = json!({"providers":{"fixture":{"adapter":"openai-chat"}}});
    let mappings = [
        ("protocol", "modelAdapters"),
        ("contextWindow", "modelContextWindows"),
        ("inputModalities", "modelInputModalities"),
        ("reasoningEfforts", "modelReasoningEfforts"),
        ("defaultEffort", "modelDefaultReasoningEfforts"),
        ("fast", "modelSupportsServiceTier"),
        ("pinnedEffort", "modelPinnedReasoningEfforts"),
        ("effortMap", "modelReasoningEffortMap"),
    ];
    let mut targets = mappings
        .iter()
        .map(|(field, map)| MaterializedTargetV2 {
            channel_id: CHANNEL.into(),
            raw_model_id: "model".into(),
            field: (*field).into(),
            provider_id: "fixture".into(),
            pointer: format!("/providers/fixture/{map}/model"),
        })
        .collect::<Vec<_>>();
    for (field, path) in [
        ("noReasoning", "noReasoningModels"),
        (
            "reasoningAuthoritative",
            "modelReasoningEffortsAuthoritative",
        ),
        ("fastWire", "fastWire"),
    ] {
        targets.push(MaterializedTargetV2 {
            channel_id: CHANNEL.into(),
            raw_model_id: "model".into(),
            field: field.into(),
            provider_id: "fixture".into(),
            pointer: format!("/providers/fixture/{path}"),
        });
    }
    let ledger = finalize_materialization_ledger_v2(&p, &raw, &targets).unwrap();
    assert_eq!(ledger.fields.len(), 11);
    let saved = &p.channels[0].models[0].source;
    assert_eq!(
        reconcile_source_v2(
            CHANNEL,
            saved,
            saved,
            Some(&ledger),
            &policy_semantic_digest_v2(&p).unwrap(),
            &raw
        )
        .unwrap(),
        *saved
    );
    let mut malicious = targets.clone();
    malicious[0].pointer = "/providers/fixture/apiKey".into();
    assert!(finalize_materialization_ledger_v2(&p, &raw, &malicious).is_err());
    targets.pop();
    assert_eq!(
        finalize_materialization_ledger_v2(&p, &raw, &targets)
            .unwrap_err()
            .code,
        "model_source_provenance_unknown"
    );
    let groups = vec![ProtocolGroupPlanV2 {
        channel_id: CHANNEL.into(),
        protocol: ProtocolV2::OpenaiResponses,
        wire_profile_digest: digest(b"wire"),
        raw_model_ids: vec!["model".into()],
    }];
    assert_eq!(
        validate_protocol_group_auth_v2(&groups, &[])
            .unwrap_err()
            .code,
        "model_protocol_auth_incompatible"
    );
    assert!(validate_protocol_group_auth_v2(
        &groups,
        &[ProtocolAuthBindingV2 {
            channel_id: CHANNEL.into(),
            allowed_protocols: [ProtocolV2::OpenaiChat].into()
        }]
    )
    .is_err());
    validate_protocol_group_auth_v2(
        &groups,
        &[ProtocolAuthBindingV2 {
            channel_id: CHANNEL.into(),
            allowed_protocols: [ProtocolV2::OpenaiResponses].into(),
        }],
    )
    .unwrap();
}

#[test]
fn frozen_nineteen_unmatched_models_keep_original_configuration() {
    let library = seed_library_v2().unwrap();
    let rows: Value = serde_json::from_str(FROZEN_ROWS).unwrap();
    let mut ids = BTreeSet::new();
    for row in rows
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["family"].is_null())
    {
        let id = row["rawModelId"].as_str().unwrap();
        let mut p = policy(&[id]);
        if row["provider"] == "GZGG-NewApi" {
            p.channels[0].namespace = Some(NamespaceV2 {
                channel_id: CHANNEL.into(),
                remove_leading_prefixes: serde_json::from_str(NAMESPACE_PREFIXES).unwrap(),
            });
        }
        let result = match_read_to_draft_v2(&library, &p, &observation(&library, &[id])).unwrap();
        assert_eq!(result.next_policy, p, "{id}");
        assert!(!result.dto.changed);
        assert_eq!(result.dto.counts.unmatched, 1);
        ids.insert(id);
    }
    assert_eq!(ids.len(), 19);
}
