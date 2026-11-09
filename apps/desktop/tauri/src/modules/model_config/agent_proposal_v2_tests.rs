use super::*;
const NEW_TEMPLATE: &str = "11111111-1111-4111-8111-111111111111";
const NEW_REVISION: &str = "22222222-2222-4222-8222-222222222222";
const TASK: &str = "33333333-3333-4333-8333-333333333333";
fn fixture() -> (LibraryV2, AgentProposalV2, AgentProposalContextV2) {
    let library = seed_library_v2().unwrap();
    let t = &library.templates[0];
    let old = &t.revisions[0];
    let target = RefV2 {
        template_id: t.id.clone(),
        revision_id: old.id.clone(),
        digest: old.digest.clone(),
    };
    let mut content = old.content.clone();
    content.description = "reviewed v2 proposal".into();
    let proposal = AgentProposalV2 {
        contract_version: 2,
        expected_library: digest(&canonical_bytes(&library).unwrap()),
        proposals: vec![ContentProposalV2 {
            target: Some(target.clone()),
            label: t.label.clone(),
            content_version: 2,
            content,
        }],
    };
    let context = AgentProposalContextV2 {
        expected_library: proposal.expected_library.clone(),
        allowed_targets: vec![target],
        allow_create: false,
        origin: TrustedAgentOriginV2::InternalAgent {
            task_id: TASK.into(),
        },
        assignments: vec![AssignedRevisionV2 {
            template_id: t.id.clone(),
            revision_id: NEW_REVISION.into(),
            created_at: "2026-11-06T16:00:00Z".into(),
        }],
    };
    (library, proposal, context)
}
fn wire(proposal: &AgentProposalV2) -> Vec<u8> {
    canonical_bytes(proposal).unwrap()
}
fn lock_fixture() -> (LibraryV2, AgentProposalV2, AgentProposalContextV2) {
    let (mut library, mut proposal, mut context) = fixture();
    library.templates[0].revisions[0]
        .content
        .locked_fields
        .insert("description".into());
    library.templates[0].revisions[0].digest = content_digest(
        &library.templates[0].label,
        &library.templates[0].revisions[0].content,
    )
    .unwrap();
    let old = &library.templates[0].revisions[0];
    let target = RefV2 {
        template_id: library.templates[0].id.clone(),
        revision_id: old.id.clone(),
        digest: old.digest.clone(),
    };
    context.expected_library = digest(&canonical_bytes(&library).unwrap());
    context.allowed_targets = vec![target.clone()];
    proposal.expected_library = context.expected_library.clone();
    proposal.proposals[0].target = Some(target);
    proposal.proposals[0].content.locked_fields = old.content.locked_fields.clone();
    (library, proposal, context)
}
#[test]
fn strict_v2_proposal_produces_exact_diff_dependencies_and_trusted_save_origin() {
    let (library, proposal, context) = fixture();
    let original = library.clone();
    let review = parse_validate_proposal_v2(&library, &wire(&proposal), &context).unwrap();
    assert!(review.valid);
    assert_eq!(review.proposals[0].dependencies, context.allowed_targets);
    assert!(review.proposals[0]
        .differences
        .iter()
        .any(|d| d.field_path == "/content/description"));
    let saved = plan_agent_save_v2(&library, &wire(&proposal), &context).unwrap();
    assert_eq!(saved.review, review);
    assert_eq!(
        saved.library.templates[0].revisions[0],
        original.templates[0].revisions[0]
    );
    assert_eq!(
        saved.library.templates[0].default_revision,
        original.templates[0].default_revision
    );
    let new = saved.library.templates[0].revisions.last().unwrap();
    assert_eq!(new.id, NEW_REVISION);
    assert_eq!(new.created_at, "2026-11-06T16:00:00Z");
    assert_eq!(new.origin, SaveOrigin::InternalAgent(TASK.into()));
    assert_eq!(library, original);
}
#[test]
fn backend_external_origin_and_creation_are_explicit_and_never_chosen_by_response() {
    let (library, mut proposal, mut context) = fixture();
    proposal.proposals[0].target = None;
    proposal.proposals[0].label = "New template".into();
    context.allowed_targets.clear();
    context.assignments[0].template_id = NEW_TEMPLATE.into();
    context.origin = TrustedAgentOriginV2::ExternalTask {
        task_id: TASK.into(),
    };
    assert!(
        !parse_validate_proposal_v2(&library, &wire(&proposal), &context)
            .unwrap()
            .valid
    );
    context.allow_create = true;
    let plan = plan_agent_save_v2(&library, &wire(&proposal), &context).unwrap();
    let t = plan
        .library
        .templates
        .iter()
        .find(|t| t.id == NEW_TEMPLATE)
        .unwrap();
    assert_eq!(t.revisions[0].origin, SaveOrigin::ExternalTask(TASK.into()));
    assert!(t.default_revision.is_none());
    assert_eq!(plan.library.templates.len(), library.templates.len() + 1);
}
#[test]
fn response_cannot_smuggle_origin_identity_time_or_unknown_fields() {
    let (library, proposal, context) = fixture();
    let value = serde_json::to_value(&proposal).unwrap();
    for key in [
        "origin",
        "revisionId",
        "createdAt",
        "templateId",
        "makeDefault",
        "unknown",
    ] {
        let mut top = value.clone();
        top[key] = serde_json::json!("attacker-controlled");
        assert_eq!(
            parse_validate_proposal_v2(&library, &serde_json::to_vec(&top).unwrap(), &context)
                .unwrap_err()
                .code,
            "model_unknown_field"
        );
        let mut nested = value.clone();
        nested["proposals"][0][key] = serde_json::json!("attacker-controlled");
        assert_eq!(
            parse_validate_proposal_v2(&library, &serde_json::to_vec(&nested).unwrap(), &context)
                .unwrap_err()
                .code,
            "model_unknown_field"
        );
    }
    let mut content = value;
    content["proposals"][0]["content"]["hiddenUnknown"] = serde_json::json!(false);
    assert_eq!(
        parse_validate_proposal_v2(&library, &serde_json::to_vec(&content).unwrap(), &context)
            .unwrap_err()
            .code,
        "model_unknown_field"
    );
}
#[test]
fn suffix_v1_bad_contract_duplicate_keys_and_stale_expected_library_are_rejected() {
    let (library, proposal, context) = fixture();
    let mut value = serde_json::to_value(&proposal).unwrap();
    value["contractVersion"] = serde_json::json!(1);
    assert_eq!(
        parse_validate_proposal_v2(&library, &serde_json::to_vec(&value).unwrap(), &context)
            .unwrap_err()
            .code,
        "model_schema_unsupported"
    );
    value = serde_json::to_value(&proposal).unwrap();
    value["proposals"][0]["content"]["variants"] =
        serde_json::json!([{ "suffix":"-legacy","rules":{} }]);
    assert!(
        parse_validate_proposal_v2(&library, &serde_json::to_vec(&value).unwrap(), &context)
            .is_err()
    );
    let duplicate = String::from_utf8(wire(&proposal)).unwrap().replacen(
        "\"contractVersion\":2",
        "\"contractVersion\":1,\"contractVersion\":2",
        1,
    );
    assert_eq!(
        parse_validate_proposal_v2(&library, duplicate.as_bytes(), &context)
            .unwrap_err()
            .code,
        "model_json_duplicate_key"
    );
    let mut stale = proposal;
    stale.expected_library = digest(b"stale");
    assert_eq!(
        parse_validate_proposal_v2(&library, &wire(&stale), &context)
            .unwrap_err()
            .code,
        "model_save_stale"
    );
}
#[test]
fn exact_reference_scope_and_digest_cannot_be_substituted() {
    let (library, mut proposal, mut context) = fixture();
    context.allowed_targets.clear();
    let review = parse_validate_proposal_v2(&library, &wire(&proposal), &context).unwrap();
    assert!(!review.valid);
    assert_eq!(
        review.proposals[0].issues[0].code,
        "model_agent_target_scope"
    );
    let (_, _, context) = fixture();
    proposal.proposals[0].target.as_mut().unwrap().digest = digest(b"wrong");
    let review = parse_validate_proposal_v2(&library, &wire(&proposal), &context).unwrap();
    assert!(!review.valid);
    assert_eq!(review.issues[0].code, "model_revision_digest_mismatch");
    assert!(plan_agent_save_v2(&library, &wire(&proposal), &context).is_err());
}
#[test]
fn real_save_plan_enforces_root_and_condition_locks_with_no_partial_library() {
    let (library, proposal, context) = lock_fixture();
    let review = parse_validate_proposal_v2(&library, &wire(&proposal), &context).unwrap();
    assert!(!review.valid);
    assert_eq!(review.issues[0].code, "model_locked_field");
    assert_eq!(
        plan_agent_save_v2(&library, &wire(&proposal), &context)
            .unwrap_err()
            .code,
        "model_agent_proposal_invalid"
    );
    let (mut library, mut proposal, mut context) = fixture();
    library.templates[0].revisions[0]
        .content
        .condition_overrides[0]
        .locked_fields
        .insert("contextWindow".into());
    library.templates[0].revisions[0].digest = content_digest(
        &library.templates[0].label,
        &library.templates[0].revisions[0].content,
    )
    .unwrap();
    let old = &library.templates[0].revisions[0];
    context.allowed_targets[0].digest = old.digest.clone();
    proposal.proposals[0].target = Some(context.allowed_targets[0].clone());
    context.expected_library = digest(&canonical_bytes(&library).unwrap());
    proposal.expected_library = context.expected_library.clone();
    proposal.proposals[0].content = old.content.clone();
    proposal.proposals[0].content.condition_overrides[0]
        .rules
        .context_window = Field::Custom(2048);
    assert_eq!(
        parse_validate_proposal_v2(&library, &wire(&proposal), &context)
            .unwrap()
            .issues[0]
            .code,
        "model_locked_field"
    );
}
#[test]
fn invalid_real_selector_produces_issues_and_a_mixed_batch_never_returns_partial_save() {
    let (library, mut proposal, mut context) = fixture();
    let mut new = proposal.proposals[0].clone();
    new.target = None;
    new.label = "Second".into();
    new.content.name_layouts[0].numeric_joiners.push("_".into());
    proposal.proposals.push(new);
    context.allow_create = true;
    context.assignments.push(AssignedRevisionV2 {
        template_id: NEW_TEMPLATE.into(),
        revision_id: "55555555-5555-4555-8555-555555555555".into(),
        created_at: "2026-11-06T16:00:00Z".into(),
    });
    let original = library.clone();
    let review = parse_validate_proposal_v2(&library, &wire(&proposal), &context).unwrap();
    assert!(!review.valid);
    assert!(review.proposals[0].planned_revision.is_some());
    assert!(review.proposals[1].planned_revision.is_none());
    assert_eq!(review.proposals[1].issues[0].code, "model_selector_invalid");
    assert!(plan_agent_save_v2(&library, &wire(&proposal), &context).is_err());
    assert_eq!(library, original);
}
