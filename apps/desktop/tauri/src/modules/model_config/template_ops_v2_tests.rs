use super::*;
use crate::modules::model_config::{
    schema::{ModelRules, ModelSelection, SaveOrigin},
    source_v2::source_from_provider_v2,
    store_v2::empty_policy_v2,
};
use std::collections::BTreeMap;
const TEMPLATE: &str = "11111111-1111-4111-8111-111111111111";
const FIRST: &str = "22222222-2222-4222-8222-222222222222";
const SECOND: &str = "33333333-3333-4333-8333-333333333333";
const CHANNEL: &str = "44444444-4444-4444-8444-444444444444";
fn hash(library: &LibraryV2) -> String {
    digest(&canonical_bytes(library).unwrap())
}
fn user_library(two: bool) -> LibraryV2 {
    let seed = seed_library_v2().unwrap();
    let mut content = seed.templates[0].revisions[0].content.clone();
    content.description = "user revision 1".into();
    let request = SaveRevisionRequestV2 {
        expected_library: hash(&seed),
        template_id: TEMPLATE.into(),
        base_revision: None,
        revision_id: FIRST.into(),
        created_at: "2026-11-06T00:00:00Z".into(),
        label: "User".into(),
        content,
        make_default: false,
        origin: SaveOrigin::Manual,
    };
    let first = save_revision_plan_v2(&seed, &request).unwrap();
    if !two {
        return first;
    }
    let mut request = request;
    request.expected_library = hash(&first);
    request.base_revision = Some(FIRST.into());
    request.revision_id = SECOND.into();
    request.content.description = "user revision 2".into();
    save_revision_plan_v2(&first, &request).unwrap()
}
fn reference(library: &LibraryV2, id: &str) -> RefV2 {
    exact(library, TEMPLATE, id).unwrap()
}
fn policy_with(library: &LibraryV2, manual: bool) -> PolicyV2 {
    let reference = reference(library, FIRST);
    let model = "gpt-6-sol";
    let channel = ChannelV2 {
        id: CHANNEL.into(),
        label: "fixture".into(),
        connection: ConnectionV2 {
            provider_id: "fixture".into(),
            nodes: vec![],
            active_node_id: String::new(),
            credential_ref: None,
            enabled: true,
            allow_private_network: false,
            auth_mode: None,
            source_read_id: None,
            imported_config: None,
        },
        selection: ModelSelection::Whitelist(vec![model.into()]),
        model_order: vec![model.into()],
        models: vec![ModelConfigV2 {
            raw_model_id: model.into(),
            label: model.into(),
            observed: true,
            source: source_from_provider_v2(
                "fixture",
                model,
                &serde_json::json!({"adapter":"openai-chat"}),
                &digest(b"source"),
                SourceKindV2::ImportedFile,
            )
            .unwrap(),
            explicit_overrides: ModelRules::default(),
            binding: Some(if manual {
                ModelBindingV2::Manual { reference }
            } else {
                ModelBindingV2::AutoRead { reference }
            }),
        }],
        template_refs: vec![],
        defaults: ModelRules::default(),
        family_overrides: BTreeMap::new(),
        naming: BTreeMap::new(),
        namespace: None,
    };
    let mut p = empty_policy_v2();
    p.channels.push(channel);
    p
}
#[test]
fn exact_default_and_explicit_cancel_preserve_all_revision_bytes() {
    let lib = user_library(true);
    let first = reference(&lib, FIRST);
    let second = reference(&lib, SECOND);
    let initial = lib.clone();
    let selected = set_default(&lib, &hash(&lib), &first).unwrap();
    assert_eq!(
        selected
            .templates
            .last()
            .unwrap()
            .default_revision
            .as_deref(),
        Some(FIRST)
    );
    assert_eq!(
        selected.templates.last().unwrap().revisions,
        initial.templates.last().unwrap().revisions
    );
    assert_eq!(
        set_default(&selected, &hash(&selected), &first).unwrap(),
        selected
    );
    assert!(set_default(&selected, &hash(&lib), &second).is_err());
    assert_eq!(
        clear_default(&selected, &hash(&selected), &second)
            .unwrap_err()
            .code,
        "model_template_default_changed"
    );
    let cancelled = clear_default(&selected, &hash(&selected), &first).unwrap();
    assert!(cancelled
        .templates
        .last()
        .unwrap()
        .default_revision
        .is_none());
    assert_eq!(lib, initial);
    let mut invalid = first;
    invalid.digest = digest(b"wrong");
    assert_eq!(
        set_default(&lib, &hash(&lib), &invalid).unwrap_err().code,
        "model_revision_digest_mismatch"
    );
}
#[test]
fn name_match_is_not_an_exact_binding_and_manual_auto_draft_are_reported_separately() {
    let lib = user_library(false);
    let mut saved = policy_with(&lib, true);
    let mut draft = saved.clone();
    draft.channels[0].models[0].binding = Some(ModelBindingV2::AutoRead {
        reference: reference(&lib, FIRST),
    });
    draft.template_refs.push(reference(&lib, FIRST));
    draft.channels[0].template_refs.push(reference(&lib, FIRST));
    let report = usage(&lib, &saved, Some(&draft), TEMPLATE, FIRST, &[]).unwrap();
    assert_eq!(report.references.len(), 4);
    assert!(report
        .references
        .iter()
        .any(|r| r.state == UsageStateV2::Saved && r.kind == UsageKindV2::Manual));
    assert!(report
        .references
        .iter()
        .any(|r| r.state == UsageStateV2::Draft && r.kind == UsageKindV2::AutoRead));
    assert!(report
        .models
        .iter()
        .all(|m| m.matched && m.using_queried_revision));
    saved.channels[0].models[0].binding = None;
    let only_match = usage(&lib, &saved, None, TEMPLATE, FIRST, &[]).unwrap();
    assert!(only_match.references.is_empty());
    assert!(only_match.models[0].matched);
    assert!(!only_match.models[0].using_queried_revision);
}
#[test]
fn usage_includes_only_explicit_additional_refs_and_revision_lineage() {
    let lib = user_library(true);
    let p = empty_policy_v2();
    let reference = reference(&lib, FIRST);
    let report = usage(
        &lib,
        &p,
        None,
        TEMPLATE,
        FIRST,
        std::slice::from_ref(&reference),
    )
    .unwrap();
    assert_eq!(report.references.len(), 2);
    assert!(report
        .references
        .iter()
        .any(|r| r.kind == UsageKindV2::BaseRevision
            && r.dependent_revision
                .as_ref()
                .is_some_and(|r| r.revision_id == SECOND)));
    assert!(report
        .references
        .iter()
        .any(|r| r.kind == UsageKindV2::AdditionalExact));
    assert_eq!(
        usage(&lib, &p, None, TEMPLATE, SECOND, &[])
            .unwrap()
            .references
            .len(),
        0
    );
}
#[test]
fn deletion_rejects_used_saved_draft_history_and_surviving_children_atomically() {
    let lib = user_library(true);
    let empty = empty_policy_v2();
    let initial = lib.clone();
    for (saved, draft, additional) in [
        (policy_with(&lib, true), None, vec![]),
        (empty.clone(), Some(policy_with(&lib, false)), vec![]),
        (empty.clone(), None, vec![reference(&lib, FIRST)]),
    ] {
        assert_eq!(
            delete_revisions(
                &lib,
                &hash(&lib),
                TEMPLATE,
                &[FIRST.into(), SECOND.into()],
                &saved,
                draft.as_ref(),
                &additional
            )
            .unwrap_err()
            .code,
            "model_revision_in_use"
        );
        assert_eq!(lib, initial);
    }
    assert_eq!(
        delete_revisions(
            &lib,
            &hash(&lib),
            TEMPLATE,
            &[FIRST.into()],
            &empty,
            None,
            &[]
        )
        .unwrap_err()
        .code,
        "model_revision_in_use"
    );
    let removed = delete_revisions(
        &lib,
        &hash(&lib),
        TEMPLATE,
        &[FIRST.into(), SECOND.into()],
        &empty,
        None,
        &[],
    )
    .unwrap();
    assert!(!removed.templates.iter().any(|t| t.id == TEMPLATE));
    assert_eq!(
        removed.templates,
        initial.templates[..initial.templates.len() - 1]
    );
}
#[test]
fn builtin_initial_and_default_revisions_are_protected_without_auto_latest() {
    let lib = user_library(true);
    let empty = empty_policy_v2();
    let seed = &lib.templates[0];
    assert_eq!(
        delete_revisions(
            &lib,
            &hash(&lib),
            &seed.id,
            &[seed.revisions[0].id.clone()],
            &empty,
            None,
            &[]
        )
        .unwrap_err()
        .code,
        "model_revision_builtin_protected"
    );
    let selected = set_default(&lib, &hash(&lib), &reference(&lib, SECOND)).unwrap();
    assert_eq!(
        delete_revisions(
            &selected,
            &hash(&selected),
            TEMPLATE,
            &[SECOND.into()],
            &empty,
            None,
            &[]
        )
        .unwrap_err()
        .code,
        "model_revision_is_default"
    );
    let cancelled =
        clear_default(&selected, &hash(&selected), &reference(&selected, SECOND)).unwrap();
    let deleted = delete_revisions(
        &cancelled,
        &hash(&cancelled),
        TEMPLATE,
        &[SECOND.into()],
        &empty,
        None,
        &[],
    )
    .unwrap();
    let t = deleted.templates.iter().find(|t| t.id == TEMPLATE).unwrap();
    assert_eq!(t.revisions.len(), 1);
    assert_eq!(t.revisions[0].id, FIRST);
    assert!(t.default_revision.is_none());
}
#[test]
fn stale_invalid_or_partial_delete_selection_never_changes_the_library() {
    let lib = user_library(true);
    let empty = empty_policy_v2();
    let expected = hash(&lib);
    assert_eq!(
        delete_revisions(&lib, "stale", TEMPLATE, &[SECOND.into()], &empty, None, &[])
            .unwrap_err()
            .code,
        "model_save_stale"
    );
    for ids in [
        vec![],
        vec![SECOND.into(), SECOND.into()],
        vec![SECOND.into(), CHANNEL.into()],
    ] {
        assert!(delete_revisions(&lib, &expected, TEMPLATE, &ids, &empty, None, &[]).is_err());
    }
    assert_eq!(hash(&lib), expected);
    let mut invalid = reference(&lib, FIRST);
    invalid.digest = digest(b"wrong");
    assert_eq!(
        delete_revisions(
            &lib,
            &expected,
            TEMPLATE,
            &[FIRST.into(), SECOND.into()],
            &empty,
            None,
            &[invalid]
        )
        .unwrap_err()
        .code,
        "model_revision_digest_mismatch"
    );
}
