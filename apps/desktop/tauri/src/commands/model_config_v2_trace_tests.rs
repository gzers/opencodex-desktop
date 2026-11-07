use super::*;
use crate::modules::model_config::matching_v2::{MatchReasonV2 as R, PreconditionKindV2};
const CHANNEL: &str = "33333333-3333-4333-8333-333333333333";
fn content() -> TemplateContentV2 {
    let mut c = seed_library_v2().unwrap().templates[0].revisions[0]
        .content
        .clone();
    c.matching.include = vec![PatternV2 {
        kind: PatternKindV2::Prefix,
        value: "lab-".into(),
    }];
    c.matching.exclude.clear();
    c.matching.channels.clear();
    c.matching.channel_mode = crate::modules::model_config::schema::ChannelMode::All;
    c.name_layouts = vec![NameLayoutV2 {
        id: "11111111-1111-4111-8111-111111111111".into(),
        literal_prefix: "lab-".into(),
        version_capture: "complete_numeric_components".into(),
        numeric_joiners: vec![".".into(), "-".into()],
        tail_capture: "literal_remainder".into(),
        trailing_date_policy: "retain_valid_YYYYMMDD_after_hyphen".into(),
        ambiguous_numeric_tail: "no_equivalence".into(),
    }];
    let mut condition = c.condition_overrides[0].clone();
    condition.source_exact_id = "lab-06.1-sol".into();
    condition.selector = SelectorV2 {
        layout_id: Some(c.name_layouts[0].id.clone()),
        literal_tail: Some("-sol".into()),
        all: vec![
            SelectorAtomV2::NumericVersion {
                components: vec!["06".into(), "1".into()],
            },
            SelectorAtomV2::LiteralSegment {
                value: "sol".into(),
                position: LiteralPositionV2::AfterVersion,
                boundary: "hyphen_or_string_edge".into(),
            },
        ],
    };
    c.condition_overrides = vec![condition];
    c
}
fn preview(content: TemplateContentV2, ids: &[&str]) -> Result<ConditionPreview> {
    preview_conditions(ConditionPreviewRequest {
        content,
        raw_model_ids: ids.iter().map(|id| (*id).into()).collect(),
        channel_id: CHANNEL.into(),
    })
}
#[test]
fn all_real_seed_conditions_trace_the_same_applied_revision_rules_as_resolution() {
    let library = seed_library_v2().unwrap();
    let before = library.clone();
    let mut count = 0;
    for template in &library.templates {
        let content = &template.revisions[0].content;
        let ids = content
            .condition_overrides
            .iter()
            .map(|c| c.source_exact_id.as_str())
            .collect::<Vec<_>>();
        if ids.is_empty() {
            continue;
        }
        let result = preview(content.clone(), &ids).unwrap();
        for row in result.rows {
            let expected = matching_v2::condition_v2(content, &row.matching_view)
                .unwrap()
                .unwrap();
            assert!(row.family_matched);
            assert_eq!(row.family.matched, row.family_matched);
            assert_eq!(
                row.matched_condition_id.as_deref(),
                Some(expected.id.as_str())
            );
            assert_eq!(row.conditions.len(), content.condition_overrides.len());
            assert_eq!(row.conditions.iter().filter(|c| c.applied).count(), 1);
            let applied = row.conditions.iter().find(|c| c.applied).unwrap();
            assert_eq!(applied.condition_id, expected.id);
            assert_eq!(applied.matched, Some(true));
            assert!(applied.atoms.iter().all(|a| a.matched == Some(true)));
            count += 1;
        }
    }
    assert_eq!(count, 58);
    assert_eq!(library, before);
}
#[test]
fn readonly_preview_preserves_raw_ids_and_returns_truthful_and_tail_and_family_diagnostics() {
    let c = content();
    let before = c.clone();
    let result = preview(
        c.clone(),
        &[
            "lab-06-1-sol",
            "lab-6.1-sol",
            "lab-06.1-other",
            "unrelated/raw",
        ],
    )
    .unwrap();
    assert!(result.preview_only);
    assert_eq!(c, before);
    let matched = &result.rows[0];
    assert_eq!(matched.raw_model_id, "lab-06-1-sol");
    assert_eq!(matched.matching_view, matched.raw_model_id);
    assert_eq!(matched.reason_code, R::ConditionMatched);
    let and_failed = &result.rows[1];
    assert!(and_failed.family_matched);
    assert!(and_failed.matched_condition_id.is_none());
    assert_eq!(and_failed.conditions[0].reason_code, R::ConditionAndFailed);
    assert_eq!(and_failed.conditions[0].atoms[0].matched, Some(false));
    assert_eq!(and_failed.conditions[0].atoms[1].matched, Some(true));
    let tail = &result.rows[2];
    assert_eq!(
        tail.conditions[0].preconditions[1].kind,
        PreconditionKindV2::LiteralTail
    );
    assert_eq!(tail.conditions[0].reason_code, R::LiteralTailMismatch);
    assert!(tail.conditions[0].atoms.iter().all(|a| a.matched.is_none()));
    let unrelated = &result.rows[3];
    assert_eq!(unrelated.raw_model_id, "unrelated/raw");
    assert_eq!(unrelated.matching_view, "unrelated/raw");
    assert_eq!(unrelated.reason_code, R::NoIncludedPattern);
    assert_eq!(unrelated.conditions[0].matched, None);
    assert!(!unrelated.conditions[0].applied);
    assert!(unrelated.field_origins.is_empty());
    assert!(matches!(unrelated.protocol, ProtocolStateV2::Missing));
    let serialized = serde_json::to_value(&result).unwrap();
    assert_eq!(
        serialized["rows"][0]["conditions"][0]["atoms"][0]["reasonCode"],
        "numeric_components_matched"
    );
    assert!(serialized["rows"][3]["conditions"][0]["atoms"][0]["matched"].is_null());
    println!(
        "CONDITION_PREVIEW_EXAMPLE {}",
        serde_json::to_string(&result).unwrap()
    );
}
#[test]
fn exclusion_and_repeated_prefix_do_not_report_a_condition_as_adopted() {
    let mut c = content();
    c.matching.exclude.push(PatternV2 {
        kind: PatternKindV2::Exact,
        value: "lab-06.1-sol".into(),
    });
    let result = preview(c, &["lab-06.1-sol", "lab-lab-06.1-sol"]).unwrap();
    for (row, reason) in result
        .rows
        .into_iter()
        .zip([R::ExcludedPattern, R::RepeatedLiteralPrefix])
    {
        assert_eq!(row.family.reason_code, reason);
        assert!(!row.family_matched);
        assert!(row.matched_condition_id.is_none());
        assert!(row
            .conditions
            .iter()
            .all(|c| !c.applicable && c.matched.is_none() && !c.applied));
    }
}
#[test]
fn preview_keeps_duplicate_and_overlap_rejection_and_never_serializes_preview_authority_as_input() {
    let mut c = content();
    let mut other = c.condition_overrides[0].clone();
    other.id = "44444444-4444-4444-8444-444444444444".into();
    c.condition_overrides.push(other);
    assert_eq!(
        preview(c, &["lab-06.1-sol"]).err().unwrap().code,
        "model_selector_duplicate"
    );
    let c = content();
    let mut input = serde_json::to_value(&c).unwrap();
    input["matchingView"] = serde_json::json!("caller-forged");
    assert!(serde_json::from_value::<TemplateContentV2>(input).is_err());
}
#[test]
fn expanded_trace_budget_refuses_large_valid_batches_before_allocating_results() {
    let mut c = content();
    c.name_layouts.clear();
    let prototype = c.condition_overrides[0].clone();
    c.condition_overrides = (0..256)
        .map(|index| {
            let mut condition = prototype.clone();
            condition.id = uuid::Uuid::new_v4().to_string();
            condition.source_exact_id = format!("lab-{index}");
            condition.selector = SelectorV2 {
                layout_id: None,
                literal_tail: None,
                all: std::iter::once(SelectorAtomV2::ExactId {
                    value: condition.source_exact_id.clone(),
                })
                .chain((0..31).map(|_| SelectorAtomV2::LiteralSegment {
                    value: "lab".into(),
                    position: LiteralPositionV2::Any,
                    boundary: "hyphen_or_string_edge".into(),
                }))
                .collect(),
            };
            condition
        })
        .collect();
    matching_v2::validate_content_v2(&c).unwrap();
    assert_eq!(preview(c, &["lab-0"; 8]).err().unwrap().code, "model_limit");
}
#[test]
fn output_json_schema_freezes_nullable_skip_and_trace_reason_enums() {
    let schema = wire_output_schema();
    let definitions = schema
        .get("$defs")
        .or_else(|| schema.get("definitions"))
        .unwrap();
    assert!(definitions["ConditionPreviewRow"]["properties"]
        .get("matchingView")
        .is_some());
    assert!(definitions["ConditionMatchTraceV2"]["properties"]
        .get("applicable")
        .is_some());
    let reasons = definitions["MatchReasonV2"]["enum"].as_array().unwrap();
    assert!(reasons.contains(&serde_json::json!("selector_precondition_failed")));
    assert!(reasons.contains(&serde_json::json!("family_not_applicable")));
    println!(
        "CONDITION_PREVIEW_SCHEMA {}",
        serde_json::to_string(&schema).unwrap()
    );
    println!(
        "CONDITION_PREVIEW_ERROR_SCHEMA {}",
        serde_json::to_string(&schemars::schema_for!(MatchErrorV2)).unwrap()
    );
}

#[test]
fn real_preview_function_emits_reproducible_input_output_cases_for_the_consumer() {
    fn emit(name: &str, c: TemplateContentV2, ids: &[&str], expected_error: Option<&str>) {
        let request = serde_json::json!({"content":c,"rawModelIds":ids,"channelId":CHANNEL});
        let decoded: ConditionPreviewRequest = serde_json::from_value(request.clone()).unwrap();
        let result = preview_conditions(decoded);
        let (output, error) = match result {
            Ok(result) => {
                assert!(expected_error.is_none(), "{name}");
                (
                    serde_json::to_value(result).unwrap(),
                    serde_json::Value::Null,
                )
            }
            Err(error) => {
                assert_eq!(Some(error.code.as_str()), expected_error, "{name}");
                (
                    serde_json::Value::Null,
                    serde_json::to_value(error).unwrap(),
                )
            }
        };
        println!(
            "CONDITION_PREVIEW_CASE {}",
            serde_json::json!({"case":name,"request":request,"result":output,"error":error})
        );
    }
    emit(
        "numeric_joiners",
        content(),
        &["lab-06.1-sol", "lab-06-1-sol"],
        None,
    );
    emit(
        "complete_numeric_boundaries",
        content(),
        &[
            "lab-6.1-sol",
            "lab-06.10-sol",
            "lab-06.1.0-sol",
            "lab-06.1-0731-sol",
            "lab-06.1-sol-20251001",
        ],
        None,
    );
    let mut any = content();
    let SelectorAtomV2::LiteralSegment { position, .. } =
        &mut any.condition_overrides[0].selector.all[1]
    else {
        unreachable!()
    };
    *position = LiteralPositionV2::Any;
    emit("name_any", any, &["lab-06.1-sol"], None);
    let mut before = content();
    before.name_layouts[0].literal_prefix = "lab-sol-".into();
    before.condition_overrides[0].selector.literal_tail = Some(String::new());
    before.condition_overrides[0].source_exact_id = "lab-sol-06.1".into();
    let SelectorAtomV2::LiteralSegment { position, .. } =
        &mut before.condition_overrides[0].selector.all[1]
    else {
        unreachable!()
    };
    *position = LiteralPositionV2::BeforeVersion;
    emit(
        "name_before_version",
        before,
        &["lab-sol-06.1", "lab-sol-6.1"],
        None,
    );
    emit("name_after_version", content(), &["lab-06.1-sol"], None);
    let mut wrong_position = content();
    let SelectorAtomV2::LiteralSegment { position, .. } =
        &mut wrong_position.condition_overrides[0].selector.all[1]
    else {
        unreachable!()
    };
    *position = LiteralPositionV2::BeforeVersion;
    emit(
        "wrong_name_position",
        wrong_position,
        &["lab-06.1-sol"],
        None,
    );
    let mut and = content();
    and.condition_overrides[0]
        .selector
        .all
        .push(SelectorAtomV2::ExactId {
            value: "lab-06.1-sol".into(),
        });
    emit("all_and_atoms", and, &["lab-6.1-sol", "lab-06.1-sol"], None);
    emit(
        "layout_and_tail_prerequisites",
        content(),
        &["lab-x-sol", "lab-06.1-other"],
        None,
    );
    let mut boundary = content();
    boundary.name_layouts.clear();
    boundary.condition_overrides[0].selector = SelectorV2 {
        layout_id: None,
        literal_tail: None,
        all: vec![SelectorAtomV2::LiteralSegment {
            value: "sol".into(),
            position: LiteralPositionV2::Any,
            boundary: "hyphen_or_string_edge".into(),
        }],
    };
    emit(
        "literal_segment_boundary",
        boundary,
        &["lab-06.1-solx", "lab-06.1-other", "lab-06.1-sol"],
        None,
    );
    emit(
        "family_include_and_repeated_prefix",
        content(),
        &["other/raw", "lab-lab-06.1-sol"],
        None,
    );
    let mut excluded = content();
    excluded.matching.exclude.push(PatternV2 {
        kind: PatternKindV2::Exact,
        value: "lab-06.1-sol".into(),
    });
    emit("family_excluded", excluded, &["lab-06.1-sol"], None);
    let mut channel_scope = content();
    channel_scope.matching.channel_mode =
        crate::modules::model_config::schema::ChannelMode::Whitelist;
    channel_scope
        .matching
        .channels
        .insert("99999999-9999-4999-8999-999999999999".into());
    emit(
        "family_channel_scope",
        channel_scope,
        &["lab-06.1-sol"],
        None,
    );
    let mut missing_layout = content();
    missing_layout.name_layouts.clear();
    emit(
        "invalid_layout_reference",
        missing_layout,
        &["lab-06.1-sol"],
        Some("model_selector_invalid"),
    );
    let mut overlap = content();
    overlap.name_layouts.clear();
    overlap.condition_overrides[0].selector = SelectorV2 {
        layout_id: None,
        literal_tail: None,
        all: vec![SelectorAtomV2::LiteralSegment {
            value: "sol".into(),
            position: LiteralPositionV2::Any,
            boundary: "hyphen_or_string_edge".into(),
        }],
    };
    overlap.condition_overrides[0].source_exact_id = "lab-sol-a".into();
    let mut other = overlap.condition_overrides[0].clone();
    other.id = "44444444-4444-4444-8444-444444444444".into();
    other.source_exact_id = "lab-fast-b".into();
    other.selector.all = vec![SelectorAtomV2::LiteralSegment {
        value: "fast".into(),
        position: LiteralPositionV2::Any,
        boundary: "hyphen_or_string_edge".into(),
    }];
    overlap.condition_overrides.push(other);
    matching_v2::validate_content_v2(&overlap).unwrap();
    emit(
        "runtime_overlap_error",
        overlap,
        &["lab-sol-fast"],
        Some("model_condition_overlap"),
    );
}

// schemars 0.8.22 does not apply the existing serde rename_all_fields flag on
// FieldOriginV2. Keep this output-contract artifact faithful to the unchanged
// serializer; only names in properties/required are corrected, never variants.
fn wire_output_schema() -> serde_json::Value {
    let mut schema = serde_json::to_value(schemars::schema_for!(ConditionPreview)).unwrap();
    let definitions = schema.get_mut("definitions").unwrap();
    for variant in definitions["FieldOriginV2"]["oneOf"]
        .as_array_mut()
        .unwrap()
    {
        for (rust, wire) in [
            ("condition_id", "conditionId"),
            ("template_id", "templateId"),
        ] {
            if let Some(value) = variant["properties"].as_object_mut().unwrap().remove(rust) {
                variant["properties"][wire] = value;
            }
            if let Some(required) = variant["required"].as_array_mut() {
                for key in required {
                    if key.as_str() == Some(rust) {
                        *key = serde_json::json!(wire);
                    }
                }
            }
        }
    }
    schema
}
#[test]
fn output_schema_origin_names_follow_the_actual_existing_serde_wire_format() {
    let schema = wire_output_schema();
    let reference = RefV2 {
        template_id: "11111111-1111-4111-8111-111111111111".into(),
        revision_id: "22222222-2222-4222-8222-222222222222".into(),
        digest: digest(b"fixture"),
    };
    let origins = [
        FieldOriginV2::TemplateCondition {
            reference,
            condition_id: "condition".into(),
        },
        FieldOriginV2::ChannelFamily {
            template_id: "template".into(),
        },
    ];
    for origin in origins {
        let wire = serde_json::to_value(origin).unwrap();
        let variant = schema["definitions"]["FieldOriginV2"]["oneOf"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["properties"]["kind"]["enum"][0] == wire["kind"])
            .unwrap();
        assert_eq!(
            wire.as_object()
                .unwrap()
                .keys()
                .collect::<std::collections::BTreeSet<_>>(),
            variant["properties"].as_object().unwrap().keys().collect()
        );
        for key in variant["required"].as_array().unwrap() {
            assert!(wire.get(key.as_str().unwrap()).is_some());
        }
    }
}
