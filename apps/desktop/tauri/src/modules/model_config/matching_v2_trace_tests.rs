use super::super::schema::Field;
use super::*;
const CHANNEL: &str = "33333333-3333-4333-8333-333333333333";
fn content() -> TemplateContentV2 {
    TemplateContentV2 {
        description: "controlled trace".into(),
        matching: MatchRulesV2 {
            channel_mode: ChannelMode::All,
            channels: BTreeSet::new(),
            case_sensitive: false,
            priority: 0,
            include: vec![PatternV2 {
                kind: PatternKindV2::Prefix,
                value: "lab-".into(),
            }],
            exclude: vec![],
            reject_repeated_literal_prefix: true,
        },
        name_layouts: vec![NameLayoutV2 {
            id: "11111111-1111-4111-8111-111111111111".into(),
            literal_prefix: "lab-".into(),
            version_capture: "complete_numeric_components".into(),
            numeric_joiners: vec![".".into(), "-".into()],
            tail_capture: "literal_remainder".into(),
            trailing_date_policy: "retain_valid_YYYYMMDD_after_hyphen".into(),
            ambiguous_numeric_tail: "no_equivalence".into(),
        }],
        common_rules: ModelRules::default(),
        condition_overrides: vec![ConditionOverrideV2 {
            id: "22222222-2222-4222-8222-222222222222".into(),
            name: "known sol".into(),
            selector: SelectorV2 {
                layout_id: Some("11111111-1111-4111-8111-111111111111".into()),
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
            },
            rules: ModelRules {
                context_window: Field::Custom(4096),
                ..Default::default()
            },
            locked_fields: BTreeSet::new(),
            source_exact_id: "lab-06.1-sol".into(),
            scope: "known_identity_only".into(),
        }],
        locked_fields: BTreeSet::new(),
        sources: vec![],
    }
}
fn channel() -> ChannelV2 {
    ChannelV2 {
        id: CHANNEL.into(),
        label: "fixture".into(),
        connection: ConnectionV2 {
            provider_id: "fixture".into(),
            nodes: vec![],
            active_node_id: String::new(),
            credential_ref: None,
            enabled: false,
            allow_private_network: false,
            auth_mode: None,
            source_read_id: None,
            imported_config: None,
        },
        selection: ModelSelection::None,
        models: vec![],
        model_order: vec![],
        template_refs: vec![],
        defaults: ModelRules::default(),
        family_overrides: BTreeMap::new(),
        naming: BTreeMap::new(),
        namespace: None,
    }
}
#[test]
fn traces_match_the_real_selector_for_complete_digits_dates_arity_and_tail_failures() {
    let content = content();
    validate_content_v2(&content).unwrap();
    for raw in [
        "lab-06.1-sol",
        "lab-06-1-sol",
        "lab-6.1-sol",
        "lab-06.10-sol",
        "lab-06.1.0-sol",
        "lab-06.1-solx",
        "lab-06.1-sol-20251001",
        "lab-06.1-0731-sol",
        "lab-06.1.-sol",
        "lab-missing-sol",
        "LAB-06.1-SOL",
    ] {
        let trace = trace_matching_v2(&content, &channel(), raw).unwrap();
        assert_eq!(trace.matching_view, raw);
        assert_eq!(
            trace.family.matched,
            family_matches_v2(&content, CHANNEL, raw)
        );
        assert_eq!(
            trace.conditions[0].matched,
            Some(selector_matches_v2(
                &content,
                &content.condition_overrides[0].selector,
                raw
            )),
            "{raw}"
        );
        assert_eq!(
            trace.conditions[0].applied,
            condition_v2(&content, raw).unwrap().is_some(),
            "{raw}"
        );
    }
    let failed = trace_matching_v2(&content, &channel(), "lab-6.1-sol").unwrap();
    assert_eq!(
        failed.conditions[0].reason_code,
        MatchReasonV2::ConditionAndFailed
    );
    assert_eq!(
        failed.conditions[0].atoms[0].reason_code,
        MatchReasonV2::NumericComponentsMismatch
    );
    assert_eq!(failed.conditions[0].atoms[0].matched, Some(false));
    assert_eq!(failed.conditions[0].atoms[1].matched, Some(true));
}
#[test]
fn layout_and_tail_prerequisites_fail_before_atoms_and_do_not_invent_atom_failures() {
    let mut c = content();
    for (raw, reason) in [
        ("lab-x-sol", MatchReasonV2::LayoutParseFailed),
        ("lab-06.1-other", MatchReasonV2::LiteralTailMismatch),
    ] {
        let trace = trace_matching_v2(&c, &channel(), raw).unwrap();
        let condition = &trace.conditions[0];
        assert!(condition.applicable);
        assert_eq!(condition.matched, Some(false));
        assert!(!condition.applied);
        assert_eq!(condition.reason_code, reason);
        assert_eq!(condition.preconditions.len(), 2);
        assert!(condition
            .atoms
            .iter()
            .all(|a| a.matched.is_none()
                && a.reason_code == MatchReasonV2::SelectorPreconditionFailed));
    }
    c.name_layouts.clear();
    let trace = trace_matching_v2(&c, &channel(), "lab-06.1-sol").unwrap();
    assert_eq!(
        trace.conditions[0].preconditions[0].reason_code,
        MatchReasonV2::LayoutMissing
    );
    assert_eq!(trace.conditions[0].preconditions[1].matched, None);
    assert!(!selector_matches_v2(
        &c,
        &c.condition_overrides[0].selector,
        "lab-06.1-sol"
    ));
}
#[test]
fn rejected_family_gates_mark_every_condition_unapplied_not_false_or_successful() {
    let original = content();
    for reason in [
        MatchReasonV2::ChannelScopeMismatch,
        MatchReasonV2::RepeatedLiteralPrefix,
        MatchReasonV2::ExcludedPattern,
        MatchReasonV2::NoIncludedPattern,
    ] {
        let mut c = original.clone();
        let raw = match reason {
            MatchReasonV2::ChannelScopeMismatch => {
                c.matching.channel_mode = ChannelMode::Whitelist;
                "lab-06.1-sol"
            }
            MatchReasonV2::RepeatedLiteralPrefix => "lab-lab-06.1-sol",
            MatchReasonV2::ExcludedPattern => {
                c.matching.exclude.push(PatternV2 {
                    kind: PatternKindV2::Exact,
                    value: "lab-06.1-sol".into(),
                });
                "lab-06.1-sol"
            }
            _ => "other-06.1-sol",
        };
        let trace = trace_matching_v2(&c, &channel(), raw).unwrap();
        assert!(!trace.family.matched);
        assert_eq!(trace.family.reason_code, reason);
        for condition in trace.conditions {
            assert!(!condition.applicable);
            assert_eq!(condition.matched, None);
            assert!(!condition.applied);
            assert!(condition.atoms.iter().all(|a| a.matched.is_none()));
            assert!(condition.preconditions.iter().all(|a| a.matched.is_none()));
        }
    }
}
#[test]
fn complete_literal_boundaries_positions_and_missing_layout_have_distinct_real_reasons() {
    let mut c = content();
    c.condition_overrides[0].selector = SelectorV2 {
        layout_id: None,
        literal_tail: None,
        all: vec![SelectorAtomV2::LiteralSegment {
            value: "sol".into(),
            position: LiteralPositionV2::Any,
            boundary: "hyphen_or_string_edge".into(),
        }],
    };
    for (raw, expected) in [
        ("lab-06.1-sol", MatchReasonV2::LiteralSegmentMatched),
        ("lab-06.1-solx", MatchReasonV2::SegmentBoundaryMismatch),
        ("lab-06.1-other", MatchReasonV2::LiteralSegmentMissing),
    ] {
        let t = trace_matching_v2(&c, &channel(), raw).unwrap();
        assert_eq!(t.conditions[0].atoms[0].reason_code, expected);
    }
    let SelectorAtomV2::LiteralSegment { position, .. } =
        &mut c.condition_overrides[0].selector.all[0]
    else {
        unreachable!()
    };
    *position = LiteralPositionV2::BeforeVersion;
    assert_eq!(
        trace_matching_v2(&c, &channel(), "lab-sol-06.1")
            .unwrap()
            .conditions[0]
            .atoms[0]
            .reason_code,
        MatchReasonV2::PositionRequiresLayout
    );
    c = content();
    let SelectorAtomV2::LiteralSegment { position, .. } =
        &mut c.condition_overrides[0].selector.all[1]
    else {
        unreachable!()
    };
    *position = LiteralPositionV2::BeforeVersion;
    assert_eq!(
        trace_matching_v2(&c, &channel(), "lab-06.1-sol")
            .unwrap()
            .conditions[0]
            .atoms[1]
            .reason_code,
        MatchReasonV2::SegmentPositionMismatch
    );
}
#[test]
fn every_and_atom_and_all_conditions_are_traced_without_changing_overlap_errors() {
    let mut c = content();
    c.condition_overrides[0]
        .selector
        .all
        .push(SelectorAtomV2::ExactId {
            value: "lab-06.1-sol".into(),
        });
    let mut other = c.condition_overrides[0].clone();
    other.id = "44444444-4444-4444-8444-444444444444".into();
    other.selector.all = vec![SelectorAtomV2::ExactId {
        value: "lab-06.1-other".into(),
    }];
    c.condition_overrides.push(other);
    let before = c.clone();
    let trace = trace_matching_v2(&c, &channel(), "lab-6.1-sol").unwrap();
    assert_eq!(trace.conditions.len(), 2);
    assert_eq!(
        trace.conditions[0]
            .atoms
            .iter()
            .map(|a| a.matched)
            .collect::<Vec<_>>(),
        vec![Some(false), Some(true), Some(false)]
    );
    assert_eq!(c, before);
    c.condition_overrides[1].selector = c.condition_overrides[0].selector.clone();
    let plain = condition_v2(&c, "lab-06.1-sol").unwrap_err();
    let traced = trace_matching_v2(&c, &channel(), "lab-06.1-sol").unwrap_err();
    assert_eq!(plain, traced);
}
#[test]
fn matching_view_is_actual_explicit_namespace_while_raw_input_and_content_stay_intact() {
    let c = content();
    let mut ch = channel();
    ch.namespace = Some(NamespaceV2 {
        channel_id: ch.id.clone(),
        remove_leading_prefixes: vec!["vendor/".into()],
    });
    let raw = "vendor/lab-06.1-sol";
    let trace = trace_matching_v2(&c, &ch, raw).unwrap();
    assert_eq!(trace.matching_view, "lab-06.1-sol");
    assert_eq!(raw, "vendor/lab-06.1-sol");
    assert_eq!(trace.conditions[0].matched, Some(true));
    assert_eq!(c, content());
}

#[test]
fn traced_preview_results_and_actual_effective_resolution_are_equivalent_without_side_effects() {
    let c = content();
    let label = "trace equivalence";
    let r = RevisionV2 {
        id: "55555555-5555-4555-8555-555555555555".into(),
        created_at: "2026-11-07T00:00:00Z".into(),
        saved_label: label.into(),
        content_version: 2,
        content: c.clone(),
        digest: content_digest(label, &c).unwrap(),
        base_revision: None,
        changes: vec![],
        origin: super::super::schema::SaveOrigin::Manual,
    };
    let reference = RefV2 {
        template_id: "66666666-6666-4666-8666-666666666666".into(),
        revision_id: r.id.clone(),
        digest: r.digest.clone(),
    };
    let library = LibraryV2 {
        schema_version: 2,
        templates: vec![TemplateV2 {
            id: reference.template_id.clone(),
            label: label.into(),
            source: super::super::schema::TemplateSource::User,
            default_revision: Some(r.id.clone()),
            revisions: vec![r],
        }],
        submission_receipts: BTreeMap::new(),
    };
    let old = library.clone();
    let channel = channel();
    for raw in [
        "lab-06.1-sol",
        "lab-06-1-sol",
        "lab-6.1-sol",
        "lab-06.1-other",
        "unrelated/raw",
    ] {
        let trace = trace_matching_v2(&c, &channel, raw).unwrap();
        let model = ModelConfigV2 {
            raw_model_id: raw.into(),
            label: raw.into(),
            observed: true,
            source: SourceModelV2 {
                raw_model_id: raw.into(),
                fields: BTreeMap::new(),
            },
            explicit_overrides: ModelRules::default(),
            binding: trace.family.matched.then(|| ModelBindingV2::Manual {
                reference: reference.clone(),
            }),
        };
        let original = model.clone();
        let effective = resolve_effective_model_v2(&library, &channel, &model).unwrap();
        let applied = trace
            .conditions
            .iter()
            .filter(|c| c.applied)
            .map(|c| c.condition_id.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            effective
                .matched_condition_id
                .into_iter()
                .collect::<Vec<_>>(),
            applied,
            "{raw}"
        );
        assert_eq!(
            effective.rules.context_window,
            if applied.is_empty() {
                Field::Inherit
            } else {
                Field::Custom(4096)
            }
        );
        assert_eq!(model, original);
    }
    assert_eq!(library, old);
}
