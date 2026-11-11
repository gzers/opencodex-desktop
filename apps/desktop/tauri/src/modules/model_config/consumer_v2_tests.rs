//! Real provider/config materialization and persistence on isolated D/H roots.
use super::schema::ModelRules;
use super::{
    compiler_v2::*,
    protocol_groups::{plan_mappings_v2, MappingFileV2, RuntimeMappingV2},
    provider_import::{read_policy_v2, ProviderBindingV2},
    schema_v2::*,
    source_v2::*,
    store_v2::*,
};
use serde_json::{json, Value};
use std::{collections::BTreeSet, path::Path};
const CHANNEL: &str = "11111111-1111-4111-8111-111111111111";
const DRAFT: &str = "22222222-2222-4222-8222-222222222222";
const READ: &str = "33333333-3333-4333-8333-333333333333";
fn config() -> Value {
    json!({"providers":{"vendor":{"adapter":"openai-chat","baseUrl":"https://fixture.invalid/v1","authMode":"key","apiKey":"synthetic-auth-do-not-export","models":["gpt-6-sol","vendor/custom"],"selectedModels":["gpt-6-sol","vendor/custom"],"modelContextWindows":{"vendor/custom":4096},"note":"original note","unknownVendor":{"keep":true}},"other":{"adapter":"openai-chat","baseUrl":"https://other.invalid","models":["untouched"],"apiKey":"other-synthetic"}},"modelPickerOrder":["vendor/vendor/custom","vendor/gpt-6-sol","other/untouched"],"disabledModels":["other/untouched"],"defaultProvider":"vendor","defaultModel":"vendor/gpt-6-sol","unrelated":{"keep":42}})
}
fn bindings() -> Vec<ProviderBindingV2> {
    vec![ProviderBindingV2 {
        channel_id: CHANNEL.into(),
        provider_id: "vendor".into(),
    }]
}
fn imported(raw: &Value) -> PolicyV2 {
    read_policy_v2(
        raw,
        &digest(&canonical_bytes(raw).unwrap()),
        &empty_policy_v2(),
        None,
        &MappingFileV2::default(),
        &bindings(),
    )
    .unwrap()
}
fn matched(library: &LibraryV2, policy: &PolicyV2) -> PolicyV2 {
    let base = ReadBaseV2 {
        dto_version: 2,
        draft_id: DRAFT.into(),
        draft_generation: 0,
        source_digest: digest(b"raw"),
        policy_digest: digest(b"policy"),
        library_digest: digest(&canonical_bytes(library).unwrap()),
        root_revision: "temp-root-v1".into(),
    };
    let observed = ReadChannelObservationV2 {
        read_id: READ.into(),
        base,
        channel_id: CHANNEL.into(),
        models: policy.channels[0]
            .models
            .iter()
            .map(|m| m.source.clone())
            .collect(),
    };
    match_read_to_draft_v2(library, policy, &observed)
        .unwrap()
        .next_policy
}
fn auth() -> Vec<ProtocolAuthBindingV2> {
    vec![ProtocolAuthBindingV2 {
        channel_id: CHANNEL.into(),
        allowed_protocols: [ProtocolV2::OpenaiChat, ProtocolV2::OpenaiResponses].into(),
    }]
}
fn mapping_for(library: &LibraryV2, before: &PolicyV2, intended: &PolicyV2) -> MappingFileV2 {
    let plan = plan_materialization_v2(library, before, intended).unwrap();
    let allocated = plan
        .groups
        .iter()
        .map(|g| RuntimeMappingV2 {
            channel_id: CHANNEL.into(),
            protocol: g.protocol.clone(),
            wire_profile_digest: g.wire_profile_digest.clone(),
            provider_id: if g.protocol == ProtocolV2::OpenaiChat {
                "vendor"
            } else {
                "vendor-responses"
            }
            .into(),
            anchor: g.protocol == ProtocolV2::OpenaiChat,
        })
        .collect::<Vec<_>>();
    plan_mappings_v2(&MappingFileV2::default(), &plan.groups, &allocated).unwrap()
}
fn commit_in_test(root: &Path, writes: Vec<PendingWriteV2>) {
    // Tests play L0's journal commit only inside a temp root, never StoreV2.
    for write in writes {
        assert!(!Path::new(&write.relative_path).is_absolute());
        let path = root.join(&write.relative_path);
        let before = std::fs::read(&path).ok();
        assert_eq!(before, write.before);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, write.after).unwrap();
    }
}
#[test]
fn config_read_keeps_exact_ids_selection_auth_out_of_dto_and_unknown_fields() {
    let raw = config();
    let policy = imported(&raw);
    let c = &policy.channels[0];
    assert_eq!(c.model_order, ["vendor/custom", "gpt-6-sol"]);
    assert_eq!(c.models.len(), 2);
    assert!(c
        .models
        .iter()
        .all(|m| m.explicit_overrides == Default::default()));
    assert!(!serde_json::to_string(&policy)
        .unwrap()
        .contains("synthetic-auth"));
    assert_eq!(c.connection.provider_id, "vendor");
    assert!(!serde_json::to_value(&c.connection)
        .unwrap()
        .as_object()
        .unwrap()
        .contains_key("protocol"));
    assert_eq!(raw, config());
}
#[test]
fn read_match_compile_prepare_commit_and_reopen_restore_source_layer() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("D");
    let home = temp.path().join("H");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&home).unwrap();
    let raw = config();
    std::fs::write(home.join("config.json"), canonical_bytes(&raw).unwrap()).unwrap();
    let store = StoreV2::new(&root);
    let snapshot = store.snapshot().unwrap();
    let before = imported(&raw);
    let intended = matched(&snapshot.library, &before);
    let mappings = mapping_for(&snapshot.library, &before, &intended);
    let compiled = compile_file_v2(
        &snapshot.library,
        &before,
        &intended,
        &raw,
        &mappings,
        &auth(),
    )
    .unwrap();
    assert_eq!(
        compiled.next_config["providers"]["other"],
        raw["providers"]["other"]
    );
    assert_eq!(
        compiled.next_config["providers"]["vendor"]["apiKey"],
        raw["providers"]["vendor"]["apiKey"]
    );
    assert_eq!(compiled.next_config["unrelated"], raw["unrelated"]);
    assert_eq!(
        compiled.next_config["providers"]["vendor"]["modelContextWindows"]["gpt-6-sol"],
        1050000
    );
    let locked = store.lock().unwrap();
    let writes = locked
        .prepare_save_unlocked(
            &snapshot.digests,
            &compiled.policy,
            &compiled.ledger,
            &compiled.mappings,
        )
        .unwrap();
    assert_eq!(writes.len(), 3);
    assert!(!root.join(POLICY_PATH).exists());
    assert_eq!(
        std::fs::read(home.join("config.json")).unwrap(),
        canonical_bytes(&raw).unwrap()
    );
    commit_in_test(&root, writes);
    std::fs::write(
        home.join("config.json"),
        canonical_bytes(&compiled.next_config).unwrap(),
    )
    .unwrap();
    drop(locked);
    let reopened = store.snapshot().unwrap();
    let actual: Value =
        serde_json::from_slice(&std::fs::read(home.join("config.json")).unwrap()).unwrap();
    let policy = read_policy_v2(
        &actual,
        &digest(&canonical_bytes(&actual).unwrap()),
        &reopened.policy,
        reopened.ledger.as_ref(),
        &reopened.mappings,
        &bindings(),
    )
    .unwrap();
    let model = policy.channels[0]
        .models
        .iter()
        .find(|m| m.raw_model_id == "gpt-6-sol")
        .unwrap();
    assert!(!model.source.fields.contains_key("contextWindow"));
    assert_eq!(model.explicit_overrides, ModelRules::default());
    assert!(matches!(
        model.binding,
        Some(ModelBindingV2::AutoRead { .. })
    ));
    let mut clear = policy.clone();
    clear.channels[0]
        .models
        .iter_mut()
        .find(|m| m.raw_model_id == "gpt-6-sol")
        .unwrap()
        .binding = None;
    let cleared = compile_file_v2(
        &reopened.library,
        &policy,
        &clear,
        &actual,
        &reopened.mappings,
        &auth(),
    )
    .unwrap();
    assert!(
        cleared.next_config["providers"]["vendor"]["modelContextWindows"]
            .get("gpt-6-sol")
            .is_none()
    );
    assert_eq!(
        cleared.next_config["providers"]["vendor"]["modelContextWindows"]["vendor/custom"],
        4096
    );
}
#[test]
fn protocol_split_is_real_and_recompiles_without_copied_manual_authority() {
    let library = seed_library_v2().unwrap();
    let mut raw = config();
    raw["providers"]["vendor"]["modelCapabilities"] =
        json!({"gpt-6-sol":{"inputModalities":["text"],"extension":"keep-across-groups"}});
    let before = imported(&raw);
    let matched = matched(&library, &before);
    let intended = apply_explicit_edits_v2(
        &matched,
        &[ModelEditV2::SetOverride {
            channel_id: CHANNEL.into(),
            raw_model_id: "gpt-6-sol".into(),
            field: "protocol".into(),
            value: json!("openai-responses"),
        }],
    )
    .unwrap();
    let mappings = mapping_for(&library, &before, &intended);
    let compiled = compile_file_v2(&library, &before, &intended, &raw, &mappings, &auth()).unwrap();
    assert_eq!(
        compiled.next_config["providers"]["vendor"]["models"],
        json!(["vendor/custom"])
    );
    assert_eq!(
        compiled.next_config["providers"]["vendor-responses"]["models"],
        json!(["gpt-6-sol"])
    );
    assert_eq!(
        compiled.next_config["providers"]["vendor-responses"]["adapter"],
        "openai-responses"
    );
    assert_eq!(
        compiled.next_config["defaultModel"],
        "vendor-responses/gpt-6-sol"
    );
    assert_eq!(
        compiled.next_config["modelPickerOrder"][0],
        "vendor/vendor/custom"
    );
    assert_eq!(
        compiled.next_config["modelPickerOrder"][1],
        "vendor-responses/gpt-6-sol"
    );
    let reopened = read_policy_v2(
        &compiled.next_config,
        &digest(b"reopen"),
        &compiled.policy,
        Some(&compiled.ledger),
        &compiled.mappings,
        &bindings(),
    )
    .unwrap();
    assert_eq!(reopened.channels[0].models.len(), 2);
    let again = compile_file_v2(
        &library,
        &reopened,
        &reopened,
        &compiled.next_config,
        &compiled.mappings,
        &auth(),
    )
    .unwrap();
    assert_eq!(again.next_config, compiled.next_config);
    let cleared = apply_explicit_edits_v2(
        &reopened,
        &[ModelEditV2::ClearOverride {
            channel_id: CHANNEL.into(),
            raw_model_id: "gpt-6-sol".into(),
            field: "protocol".into(),
        }],
    )
    .unwrap();
    let done = compile_file_v2(
        &library,
        &reopened,
        &cleared,
        &compiled.next_config,
        &compiled.mappings,
        &auth(),
    )
    .unwrap();
    assert_eq!(
        done.next_config["providers"]["vendor-responses"]["disabled"],
        true
    );
    assert_eq!(
        done.next_config["providers"]["vendor"]["modelCapabilities"]["gpt-6-sol"]["extension"],
        "keep-across-groups"
    );
    assert_eq!(
        done.next_config["providers"]["vendor-responses"]["models"],
        json!([])
    );
}
#[test]
fn rejected_mapping_collision_or_auth_has_zero_mutation() {
    let library = seed_library_v2().unwrap();
    let mut raw = config();
    let before = imported(&raw);
    let intended = apply_explicit_edits_v2(
        &matched(&library, &before),
        &[ModelEditV2::SetOverride {
            channel_id: CHANNEL.into(),
            raw_model_id: "gpt-6-sol".into(),
            field: "protocol".into(),
            value: json!("openai-responses"),
        }],
    )
    .unwrap();
    let mappings = mapping_for(&library, &before, &intended);
    raw["providers"]["vendor-responses"] =
        json!({"adapter":"openai-responses","apiKey":"foreign-synthetic"});
    let original = raw.clone();
    assert_eq!(
        compile_file_v2(&library, &before, &intended, &raw, &mappings, &auth())
            .err()
            .unwrap()
            .code,
        "model_runtime_mapping_conflict"
    );
    assert_eq!(raw, original);
    assert_eq!(
        compile_file_v2(
            &library,
            &before,
            &before,
            &config(),
            &mapping_for(&library, &before, &before),
            &[]
        )
        .err()
        .unwrap()
        .code,
        "model_protocol_auth_incompatible"
    );
}
#[test]
fn store_rejects_v1_unknown_fields_and_corrupt_pairs_without_rewriting() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(temp.path().join("manager-state")).unwrap();
    let old = br#"{"schemaVersion":1,"templates":[]}"#;
    std::fs::write(temp.path().join(LIBRARY_PATH), old).unwrap();
    let store = StoreV2::new(temp.path());
    assert_eq!(
        store.snapshot().unwrap_err().code,
        "model_schema_unsupported"
    );
    assert_eq!(std::fs::read(temp.path().join(LIBRARY_PATH)).unwrap(), old);
    std::fs::remove_file(temp.path().join(LIBRARY_PATH)).unwrap();
    std::fs::write(temp.path().join(MAPPING_PATH), b"[]").unwrap();
    assert_eq!(
        store.snapshot().unwrap_err().code,
        "model_schema_unsupported"
    );
    std::fs::remove_file(temp.path().join(MAPPING_PATH)).unwrap();
    let p = imported(&config());
    std::fs::write(temp.path().join(POLICY_PATH), canonical_bytes(&p).unwrap()).unwrap();
    assert_eq!(
        store.snapshot().unwrap_err().code,
        "model_source_provenance_unknown"
    );
}
#[test]
fn library_save_and_draft_cas_use_real_model_v2_lock_and_readback() {
    let temp = tempfile::tempdir().unwrap();
    let store = StoreV2::new(temp.path());
    let snapshot = store.snapshot().unwrap();
    assert!(!temp.path().join(LIBRARY_PATH).exists());
    let t = &snapshot.library.templates[0];
    let r = &t.revisions[0];
    let mut request = SaveRevisionRequestV2 {
        expected_library: snapshot.digests.library.clone(),
        template_id: t.id.clone(),
        base_revision: Some(r.id.clone()),
        revision_id: "44444444-4444-4444-8444-444444444444".into(),
        created_at: "2026-11-06T13:00:00Z".into(),
        label: t.label.clone(),
        content: r.content.clone(),
        make_default: false,
        origin: super::schema::SaveOrigin::Manual,
    };
    request.content.description.push_str(" test save");
    let saved = store.save_library(&request).unwrap();
    assert_eq!(saved.templates[0].revisions[0], *r);
    assert_eq!(
        store.save_library(&request).unwrap_err().code,
        "model_save_stale"
    );
    let snapshot = store.snapshot().unwrap();
    let base = ReadBaseV2 {
        dto_version: 2,
        draft_id: DRAFT.into(),
        draft_generation: 0,
        source_digest: digest(b"file"),
        policy_digest: digest(b"policy"),
        library_digest: snapshot.digests.library,
        root_revision: "fixture-root".into(),
    };
    let draft = DraftV2 {
        schema_version: 2,
        base: base.clone(),
        policy: empty_policy_v2(),
        observed_baseline: None,
        observed_config_digest: None,
        credential_edits: Default::default(),
    };
    store.stage_draft(None, draft.clone()).unwrap();
    let mut changed = draft.clone();
    changed.base.draft_generation = 1;
    store.stage_draft(Some(&base), changed).unwrap();
    assert_eq!(
        store.stage_draft(Some(&base), draft).unwrap_err().code,
        "model_read_stale"
    );
    let guard = store.lock().unwrap();
    assert!(
        crate::infrastructure::locking::TargetFileLock::try_lock_with_timeout(
            &temp.path().join("manager-state/model-v2"),
            std::time::Duration::ZERO
        )
        .is_err()
    );
    assert!(guard.snapshot_unlocked().is_ok());
    drop(guard);
}
#[test]
fn store_prepare_rejects_changed_baseline_without_writing_policy() {
    let temp = tempfile::tempdir().unwrap();
    let store = StoreV2::new(temp.path());
    let snapshot = store.snapshot().unwrap();
    let mut bad = snapshot.digests;
    bad.files.insert(DRAFT_PATH.into(), digest(b"changed"));
    let p = empty_policy_v2();
    let ledger = MaterializationLedgerV2 {
        source_policy_digest: policy_semantic_digest_v2(&p).unwrap(),
        written_file_digest: digest(b"{}"),
        fields: vec![],
    };
    let guard = store.lock().unwrap();
    assert_eq!(
        guard
            .prepare_save_unlocked(&bad, &p, &ledger, &MappingFileV2::default())
            .unwrap_err()
            .code,
        "model_save_stale"
    );
    assert!(!temp.path().join(POLICY_PATH).exists());
}
#[cfg(unix)]
#[test]
fn resolved_root_never_follows_a_symlinked_manager_state() {
    let temp = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(other.path(), temp.path().join("manager-state")).unwrap();
    assert!(StoreV2::new(temp.path()).snapshot().is_err());
    assert_eq!(std::fs::read_dir(other.path()).unwrap().count(), 0);
}
#[test]
fn channel_rename_uses_l0_supplied_identity_and_does_not_touch_other_channels() {
    let library = seed_library_v2().unwrap();
    let original = config();
    let before = imported(&original);
    let mut intended = matched(&library, &before);
    intended.channels[0].connection.provider_id = "renamed".into();
    intended.channels[0].label = "新渠道".into();
    let mut raw = original.clone();
    let source = raw["providers"]
        .as_object_mut()
        .unwrap()
        .remove("vendor")
        .unwrap();
    raw["providers"]["renamed"] = source;
    raw["defaultProvider"] = json!("renamed");
    raw["defaultModel"] = json!("renamed/gpt-6-sol");
    raw["modelPickerOrder"] = json!([
        "renamed/vendor/custom",
        "renamed/gpt-6-sol",
        "other/untouched"
    ]);
    let mut mappings = mapping_for(&library, &before, &intended);
    for m in &mut mappings.mappings {
        m.provider_id = "renamed".into();
    }
    let compiled = compile_file_v2(&library, &before, &intended, &raw, &mappings, &auth()).unwrap();
    assert!(compiled.next_config["providers"].get("vendor").is_none());
    assert_eq!(
        compiled.next_config["providers"]["other"],
        original["providers"]["other"]
    );
    assert_eq!(compiled.policy.channels[0].id, CHANNEL);
    assert_eq!(
        compiled.policy.channels[0]
            .models
            .iter()
            .map(|m| m.raw_model_id.as_str())
            .collect::<BTreeSet<_>>(),
        ["gpt-6-sol", "vendor/custom"].into()
    );
}

#[test]
fn selected_runtime_slugs_resolve_to_existing_exact_raw_ids() {
    let mut raw = config();
    raw["providers"]["vendor"]["selectedModels"] =
        json!(["vendor/gpt-6-sol", "vendor/vendor/custom"]);
    let p = imported(&raw);
    let ids = p.channels[0]
        .models
        .iter()
        .map(|m| m.raw_model_id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(ids, ["gpt-6-sol", "vendor/custom"].into());
    assert!(
        matches!(&p.channels[0].selection,super::schema::ModelSelection::Whitelist(selected) if selected.len()==2 && selected.contains(&"vendor/custom".into()))
    );
}
#[test]
fn external_materialized_capability_edit_is_source_not_a_manual_override() {
    let library = seed_library_v2().unwrap();
    let mut raw = config();
    raw["providers"]["vendor"]["modelCapabilities"] =
        json!({"gpt-6-sol":{"inputModalities":["text"],"unknownCapability":"preserve"}});
    let before = imported(&raw);
    let intended = matched(&library, &before);
    let mappings = mapping_for(&library, &before, &intended);
    let compiled = compile_file_v2(&library, &before, &intended, &raw, &mappings, &auth()).unwrap();
    assert_eq!(
        compiled.next_config["providers"]["vendor"]["modelCapabilities"]["gpt-6-sol"]
            ["unknownCapability"],
        "preserve"
    );
    let mut modified = compiled.next_config.clone();
    modified["providers"]["vendor"]["modelCapabilities"]["gpt-6-sol"]["inputModalities"] =
        json!(["text", "audio"]);
    let p = read_policy_v2(
        &modified,
        &digest(b"external-edit"),
        &compiled.policy,
        Some(&compiled.ledger),
        &compiled.mappings,
        &bindings(),
    )
    .unwrap();
    let model = p.channels[0]
        .models
        .iter()
        .find(|m| m.raw_model_id == "gpt-6-sol")
        .unwrap();
    assert_eq!(model.explicit_overrides, ModelRules::default());
    assert_eq!(
        model.source.fields["inputModalities"].value,
        json!(["text", "audio"])
    );
    assert_eq!(
        model.source.fields["inputModalities"].evidence.kind,
        SourceKindV2::ExternalFileEdit
    );
}

#[test]
fn every_store_document_rejects_duplicate_json_without_rewriting_the_file() {
    for path in [
        LIBRARY_PATH,
        POLICY_PATH,
        DRAFT_PATH,
        LEDGER_PATH,
        MAPPING_PATH,
    ] {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("manager-state")).unwrap();
        let bytes = br#"{"schemaVersion":1,"schemaVersion":2}"#;
        let file = temp.path().join(path);
        std::fs::write(&file, bytes).unwrap();
        assert_eq!(
            StoreV2::new(temp.path()).snapshot().unwrap_err().code,
            "model_json_duplicate_key",
            "{path}"
        );
        assert_eq!(std::fs::read(file).unwrap(), bytes);
    }
    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(temp.path().join("manager-state")).unwrap();
    let p = imported(&config());
    let json = String::from_utf8(canonical_bytes(&p).unwrap()).unwrap();
    let duplicate = json.replacen("\"source\":", "\"source\":{},\"source\":", 1);
    assert_ne!(json, duplicate);
    let path = temp.path().join(POLICY_PATH);
    std::fs::write(&path, duplicate.as_bytes()).unwrap();
    assert_eq!(
        StoreV2::new(temp.path()).snapshot().unwrap_err().code,
        "model_json_duplicate_key"
    );
    assert_eq!(std::fs::read(path).unwrap(), duplicate.as_bytes());
}

#[test]
fn existing_split_groups_follow_the_logical_connection_credential_rotation() {
    let library = seed_library_v2().unwrap();
    let raw = config();
    let before = imported(&raw);
    let intended = apply_explicit_edits_v2(
        &matched(&library, &before),
        &[ModelEditV2::SetOverride {
            channel_id: CHANNEL.into(),
            raw_model_id: "gpt-6-sol".into(),
            field: "protocol".into(),
            value: json!("openai-responses"),
        }],
    )
    .unwrap();
    let mappings = mapping_for(&library, &before, &intended);
    let first = compile_file_v2(&library, &before, &intended, &raw, &mappings, &auth()).unwrap();
    let mut rotated = first.next_config.clone();
    rotated["providers"]["vendor"]["apiKey"] = json!("rotated-synthetic");
    rotated["providers"]["vendor"]["headers"] = json!({"x-fixture":"rotated"});
    let next =
        compile_file_v2(&library, &intended, &intended, &rotated, &mappings, &auth()).unwrap();
    assert_eq!(
        next.next_config["providers"]["vendor-responses"]["apiKey"],
        "rotated-synthetic"
    );
    assert_eq!(
        next.next_config["providers"]["vendor-responses"]["headers"],
        json!({"x-fixture":"rotated"})
    );
    assert_eq!(
        next.next_config["providers"]["other"],
        raw["providers"]["other"]
    );
    let mut cleared = next.next_config.clone();
    cleared["providers"]["vendor"]
        .as_object_mut()
        .unwrap()
        .remove("apiKey");
    cleared["providers"]["vendor"]
        .as_object_mut()
        .unwrap()
        .remove("headers");
    let no_auth =
        compile_file_v2(&library, &intended, &intended, &cleared, &mappings, &auth()).unwrap();
    assert!(no_auth.next_config["providers"]["vendor-responses"]
        .get("apiKey")
        .is_none());
    assert!(no_auth.next_config["providers"]["vendor-responses"]
        .get("headers")
        .is_none());
}

#[test]
fn persistent_anchor_can_park_and_return_without_reassigning_any_group_identity() {
    let library = seed_library_v2().unwrap();
    let raw = config();
    let before = imported(&raw);
    let split = apply_explicit_edits_v2(
        &matched(&library, &before),
        &[ModelEditV2::SetOverride {
            channel_id: CHANNEL.into(),
            raw_model_id: "gpt-6-sol".into(),
            field: "protocol".into(),
            value: json!("openai-responses"),
        }],
    )
    .unwrap();
    let mapping = mapping_for(&library, &before, &split);
    let first = compile_file_v2(&library, &before, &split, &raw, &mapping, &auth()).unwrap();
    let only_responses = apply_explicit_edits_v2(
        &split,
        &[ModelEditV2::SetOverride {
            channel_id: CHANNEL.into(),
            raw_model_id: "vendor/custom".into(),
            field: "protocol".into(),
            value: json!("openai-responses"),
        }],
    )
    .unwrap();
    let parked = compile_file_v2(
        &library,
        &split,
        &only_responses,
        &first.next_config,
        &mapping,
        &auth(),
    )
    .unwrap();
    assert_eq!(parked.mappings, first.mappings);
    assert_eq!(parked.next_config["providers"]["vendor"]["disabled"], true);
    assert_eq!(
        parked.next_config["providers"]["vendor"]["models"],
        json!([])
    );
    assert_eq!(
        parked.next_config["providers"]["vendor"]["selectedModels"],
        json!([])
    );
    assert_eq!(
        parked.next_config["providers"]["vendor"]["note"],
        "original note"
    );
    assert_eq!(
        parked.next_config["providers"]["vendor"]["apiKey"],
        raw["providers"]["vendor"]["apiKey"]
    );
    assert_eq!(
        parked.next_config["providers"]["vendor"]["unknownVendor"],
        raw["providers"]["vendor"]["unknownVendor"]
    );
    assert_eq!(parked.next_config["defaultProvider"], "vendor-responses");
    assert_eq!(
        parked.next_config["defaultModel"],
        "vendor-responses/gpt-6-sol"
    );
    let reopened = read_policy_v2(
        &parked.next_config,
        &digest(b"parked"),
        &parked.policy,
        Some(&parked.ledger),
        &mapping,
        &bindings(),
    )
    .unwrap();
    assert!(reopened.channels[0].connection.enabled);
    assert_eq!(reopened.channels[0].models.len(), 2);
    let same = compile_file_v2(
        &library,
        &reopened,
        &reopened,
        &parked.next_config,
        &mapping,
        &auth(),
    )
    .unwrap();
    assert_eq!(same.next_config, parked.next_config);
    let back = apply_explicit_edits_v2(
        &reopened,
        &[
            ModelEditV2::ClearOverride {
                channel_id: CHANNEL.into(),
                raw_model_id: "vendor/custom".into(),
                field: "protocol".into(),
            },
            ModelEditV2::ClearOverride {
                channel_id: CHANNEL.into(),
                raw_model_id: "gpt-6-sol".into(),
                field: "protocol".into(),
            },
        ],
    )
    .unwrap();
    let returned = compile_file_v2(
        &library,
        &reopened,
        &back,
        &parked.next_config,
        &mapping,
        &auth(),
    )
    .unwrap();
    assert_eq!(returned.mappings, first.mappings);
    assert_eq!(returned.next_config["defaultProvider"], "vendor");
    assert_eq!(returned.next_config["defaultModel"], "vendor/gpt-6-sol");
    assert_eq!(
        returned.next_config["providers"]["vendor"]["disabled"],
        false
    );
    assert_eq!(
        returned.next_config["providers"]["vendor-responses"]["disabled"],
        true
    );
    let reopened = read_policy_v2(
        &returned.next_config,
        &digest(b"returned"),
        &returned.policy,
        Some(&returned.ledger),
        &mapping,
        &bindings(),
    )
    .unwrap();
    assert!(reopened.channels[0].connection.enabled);
    assert_eq!(reopened.channels[0].models.len(), 2);
    assert!(reopened.channels[0]
        .models
        .iter()
        .all(|m| m.explicit_overrides.protocol == super::schema::Field::Inherit));
}

#[test]
fn no_resolved_active_group_parks_existing_identity_without_inventing_models() {
    let library = seed_library_v2().unwrap();
    let raw = config();
    let before = imported(&raw);
    let mapping = mapping_for(&library, &before, &before);
    let first = compile_file_v2(&library, &before, &before, &raw, &mapping, &auth()).unwrap();
    let mut pending = before.clone();
    pending.channels[0].selection = super::schema::ModelSelection::None;
    for model in &mut pending.channels[0].models {
        model.source.fields.clear();
        model.binding = None;
    }
    let parked = compile_file_v2(
        &library,
        &before,
        &pending,
        &first.next_config,
        &mapping,
        &auth(),
    )
    .unwrap();
    assert!(parked.plan.groups.is_empty());
    assert_eq!(parked.mappings, first.mappings);
    assert_eq!(parked.next_config["defaultProvider"], "vendor");
    assert!(parked.next_config.get("defaultModel").is_none());
    assert_eq!(parked.next_config["providers"]["vendor"]["disabled"], true);
    assert_eq!(
        parked.next_config["providers"]["vendor"]["models"],
        json!([])
    );
    assert_eq!(
        parked.next_config["providers"]["vendor"]["note"],
        "original note"
    );
    assert_eq!(
        parked.next_config["providers"]["vendor"]["apiKey"],
        raw["providers"]["vendor"]["apiKey"]
    );
}

fn baseline_draft_for_test(library_digest: String) -> DraftV2 {
    let observed = imported(&config());
    DraftV2 {
        schema_version: 2,
        base: ReadBaseV2 {
            dto_version: 2,
            draft_id: DRAFT.into(),
            draft_generation: 0,
            source_digest: digest(b"observed-config"),
            policy_digest: policy_semantic_digest_v2(&empty_policy_v2()).unwrap(),
            library_digest,
            root_revision: "baseline-fixture-root".into(),
        },
        policy: observed.clone(),
        observed_baseline: Some(observed),
        observed_config_digest: None,
        credential_edits: Default::default(),
    }
}
#[test]
fn draft_observation_baseline_round_trips_without_turning_source_into_manual_edits() {
    let temp = tempfile::tempdir().unwrap();
    let store = StoreV2::new(temp.path());
    let mut draft = baseline_draft_for_test(store.snapshot().unwrap().digests.library);
    let observation = draft.observed_baseline.clone().unwrap();
    draft.policy.channels[0].label = "explicit draft name".into();
    draft.policy.channels[0].selection = super::schema::ModelSelection::None;
    draft.policy.channels[0].model_order.reverse();
    draft.policy.channels[0].connection.nodes[0].url = "https://changed.invalid/v1".into();
    for model in &mut draft.policy.channels[0].models {
        model.source.fields.clear();
    }
    let mut new_channel = draft.policy.channels[0].clone();
    new_channel.id = READ.into();
    new_channel.connection.provider_id = "explicit-new".into();
    draft.policy.channels.push(new_channel);
    store.stage_draft(None, draft.clone()).unwrap();
    let reloaded = store.snapshot().unwrap().draft.unwrap();
    assert_eq!(reloaded, draft);
    assert_eq!(reloaded.observed_baseline.unwrap(), observation);
    let wire = std::fs::read(temp.path().join(DRAFT_PATH)).unwrap();
    assert!(!String::from_utf8(wire.clone())
        .unwrap()
        .contains("synthetic-auth"));
    let value = parse_strict_json_v2(&wire).unwrap();
    assert!(value.get("observedBaseline").is_some());
    assert!(!temp.path().join(POLICY_PATH).exists());
    assert!(!temp.path().join(LEDGER_PATH).exists());
    // Unreleased v2 fixtures may omit the field; schema-v1 is still rejected.
    let mut fixture = serde_json::to_value(&draft).unwrap();
    fixture.as_object_mut().unwrap().remove("observedBaseline");
    assert!(serde_json::from_value::<DraftV2>(fixture)
        .unwrap()
        .observed_baseline
        .is_none());
}
#[test]
fn draft_observation_baseline_schema_source_and_management_identity_are_validated() {
    for kind in ["schema", "source-id", "management-id"] {
        let temp = tempfile::tempdir().unwrap();
        let store = StoreV2::new(temp.path());
        let mut draft = baseline_draft_for_test(store.snapshot().unwrap().digests.library);
        let observed = draft.observed_baseline.as_mut().unwrap();
        match kind {
            "schema" => observed.schema_version = 1,
            "source-id" => observed.channels[0].models[0].source.raw_model_id = "wrong-id".into(),
            _ => observed.channels[0].id = READ.into(),
        }
        assert!(store.stage_draft(None, draft).is_err(), "{kind}");
        assert!(!temp.path().join(DRAFT_PATH).exists());
    }
    let temp = tempfile::tempdir().unwrap();
    let store = StoreV2::new(temp.path());
    let draft = baseline_draft_for_test(store.snapshot().unwrap().digests.library);
    let mut value = serde_json::to_value(&draft).unwrap();
    value["observedBaseline"]["apiKey"] = json!("never-accepted");
    assert!(serde_json::from_value::<DraftV2>(value).is_err());
}
#[test]
fn baseline_only_update_is_covered_by_the_existing_whole_draft_cas() {
    let temp = tempfile::tempdir().unwrap();
    let store = StoreV2::new(temp.path());
    let draft = baseline_draft_for_test(store.snapshot().unwrap().digests.library);
    store.stage_draft(None, draft.clone()).unwrap();
    let original = std::fs::read(temp.path().join(DRAFT_PATH)).unwrap();
    let mut next = draft.clone();
    next.observed_baseline.as_mut().unwrap().channels[0].label = "new observed name".into();
    assert_eq!(
        store
            .stage_draft(Some(&draft.base), next.clone())
            .unwrap_err()
            .code,
        "model_read_stale"
    );
    assert_eq!(
        std::fs::read(temp.path().join(DRAFT_PATH)).unwrap(),
        original
    );
    next.base.draft_generation += 1;
    store.stage_draft(Some(&draft.base), next.clone()).unwrap();
    assert_eq!(store.snapshot().unwrap().draft, Some(next.clone()));
    assert_eq!(next.policy, draft.policy);
    assert_eq!(
        store
            .stage_draft(Some(&draft.base), draft.clone())
            .unwrap_err()
            .code,
        "model_read_stale"
    );
    assert!(!temp.path().join(POLICY_PATH).exists());
    assert!(!temp.path().join(LEDGER_PATH).exists());
}

#[test]
fn draft_credential_actions_round_trip_without_secret_values_or_auth_mode_changes() {
    let temp = tempfile::tempdir().unwrap();
    let store = StoreV2::new(temp.path());
    let mut draft = baseline_draft_for_test(store.snapshot().unwrap().digests.library);
    draft.policy.channels[0].connection.credential_ref = Some(READ.into());
    draft.policy.channels[0].connection.auth_mode = Some("key".into());
    draft.credential_edits.insert(
        CHANNEL.into(),
        DraftCredentialEditV2::Replace {
            credential_ref: READ.into(),
        },
    );
    store.stage_draft(None, draft.clone()).unwrap();
    let saved = store.snapshot().unwrap().draft.unwrap();
    assert_eq!(saved, draft);
    let wire = serde_json::to_value(&saved).unwrap();
    assert_eq!(
        wire["credentialEdits"][CHANNEL],
        json!({"kind":"replace","credentialRef":READ})
    );
    assert!(!serde_json::to_string(&saved)
        .unwrap()
        .contains("synthetic-auth"));
    assert!(!temp.path().join(POLICY_PATH).exists());
    assert!(!temp.path().join(LEDGER_PATH).exists());
    let mut clear = saved.clone();
    clear.base.draft_generation += 1;
    clear.policy.channels[0].connection.credential_ref = None;
    clear
        .credential_edits
        .insert(CHANNEL.into(), DraftCredentialEditV2::Clear);
    store.stage_draft(Some(&saved.base), clear.clone()).unwrap();
    let reloaded = store.snapshot().unwrap().draft.unwrap();
    assert_eq!(
        reloaded.policy.channels[0].connection.auth_mode.as_deref(),
        Some("key")
    );
    assert_eq!(
        reloaded.credential_edits[CHANNEL],
        DraftCredentialEditV2::Clear
    );
    assert_eq!(
        serde_json::to_value(&reloaded).unwrap()["credentialEdits"][CHANNEL],
        json!({"kind":"clear"})
    );
    assert!(!temp.path().join(POLICY_PATH).exists());
    assert!(!temp.path().join(LEDGER_PATH).exists());
}
#[test]
fn draft_credential_actions_validate_exact_management_and_reference_identities() {
    for kind in [
        "wrong-channel",
        "noncanonical-channel",
        "wrong-ref",
        "noncanonical-ref",
        "clear-with-ref",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let store = StoreV2::new(temp.path());
        let mut draft = baseline_draft_for_test(store.snapshot().unwrap().digests.library);
        draft.policy.channels[0].connection.credential_ref = Some(READ.into());
        let (channel, edit) = match kind {
            "wrong-channel" => (
                DRAFT.into(),
                DraftCredentialEditV2::Replace {
                    credential_ref: READ.into(),
                },
            ),
            "noncanonical-channel" => (
                "not-a-uuid".into(),
                DraftCredentialEditV2::Replace {
                    credential_ref: READ.into(),
                },
            ),
            "wrong-ref" => (
                CHANNEL.into(),
                DraftCredentialEditV2::Replace {
                    credential_ref: DRAFT.into(),
                },
            ),
            "noncanonical-ref" => (
                CHANNEL.into(),
                DraftCredentialEditV2::Replace {
                    credential_ref: "sk-raw-secret-is-not-a-reference".into(),
                },
            ),
            _ => (CHANNEL.into(), DraftCredentialEditV2::Clear),
        };
        draft.credential_edits.insert(channel, edit);
        assert!(store.stage_draft(None, draft).is_err(), "{kind}");
        assert!(!temp.path().join(DRAFT_PATH).exists());
    }
    let secret_refs = [
        json!({"kind":"replace","credentialRef":READ,"apiKey":"raw-fixture"}),
        json!({"kind":"replace","credentialRef":READ,"secret":"raw-fixture"}),
        json!({"kind":"clear","value":null}),
        json!({"kind":"clear","apiKey":"raw-fixture"}),
        json!({"kind":"clear","credentialRef":READ}),
        json!({"kind":"replace","credentialRef":null}),
    ];
    for value in secret_refs {
        assert!(serde_json::from_value::<DraftCredentialEditV2>(value).is_err());
    }
}
#[test]
fn credential_action_only_changes_are_covered_by_the_whole_draft_cas() {
    let temp = tempfile::tempdir().unwrap();
    let store = StoreV2::new(temp.path());
    let mut draft = baseline_draft_for_test(store.snapshot().unwrap().digests.library);
    draft.policy.channels[0].connection.credential_ref = Some(READ.into());
    store.stage_draft(None, draft.clone()).unwrap();
    let original = std::fs::read(temp.path().join(DRAFT_PATH)).unwrap();
    let mut next = draft.clone();
    next.credential_edits.insert(
        CHANNEL.into(),
        DraftCredentialEditV2::Replace {
            credential_ref: READ.into(),
        },
    );
    assert_eq!(
        store
            .stage_draft(Some(&draft.base), next.clone())
            .unwrap_err()
            .code,
        "model_read_stale"
    );
    assert_eq!(
        std::fs::read(temp.path().join(DRAFT_PATH)).unwrap(),
        original
    );
    next.base.draft_generation += 1;
    store.stage_draft(Some(&draft.base), next.clone()).unwrap();
    assert_eq!(store.snapshot().unwrap().draft, Some(next.clone()));
    assert_eq!(next.policy, draft.policy);
    assert_eq!(next.observed_baseline, draft.observed_baseline);
    assert!(!temp.path().join(POLICY_PATH).exists());
    assert!(!temp.path().join(LEDGER_PATH).exists());
    let mut old_fixture = serde_json::to_value(&draft).unwrap();
    old_fixture
        .as_object_mut()
        .unwrap()
        .remove("credentialEdits");
    assert!(serde_json::from_value::<DraftV2>(old_fixture)
        .unwrap()
        .credential_edits
        .is_empty());
}

fn installed_saved_fixture(root: &Path) -> StoreSnapshotV2 {
    let library = seed_library_v2().unwrap();
    let raw = config();
    let before = imported(&raw);
    let intended = matched(&library, &before);
    let mappings = mapping_for(&library, &before, &intended);
    let compiled = compile_file_v2(&library, &before, &intended, &raw, &mappings, &auth()).unwrap();
    std::fs::create_dir_all(root.join("manager-state")).unwrap();
    for (path, bytes) in [
        (LIBRARY_PATH, canonical_bytes(&library).unwrap()),
        (POLICY_PATH, canonical_bytes(&compiled.policy).unwrap()),
        (
            LEDGER_PATH,
            canonical_bytes(&LedgerFileV2 {
                schema_version: 2,
                ledger: compiled.ledger,
            })
            .unwrap(),
        ),
        (MAPPING_PATH, canonical_bytes(&compiled.mappings).unwrap()),
    ] {
        std::fs::write(root.join(path), bytes).unwrap();
    }
    StoreV2::new(root).snapshot_saved().unwrap()
}
#[test]
fn saved_snapshot_omits_dirty_draft_and_preserves_the_actual_four_saved_objects() {
    let temp = tempfile::tempdir().unwrap();
    let baseline = installed_saved_fixture(temp.path());
    let mut draft = baseline_draft_for_test(baseline.digests.library.clone());
    draft.policy.channels[0].label = "UNSAVED-MUST-NOT-EXPORT".into();
    let bytes = canonical_bytes(&draft).unwrap();
    std::fs::write(temp.path().join(DRAFT_PATH), &bytes).unwrap();
    let store = StoreV2::new(temp.path());
    let full = store.snapshot().unwrap();
    assert_eq!(full.draft, Some(draft));
    let saved = store.snapshot_saved().unwrap();
    assert!(saved.draft.is_none());
    assert_eq!(saved.policy, baseline.policy);
    assert_eq!(saved.library, baseline.library);
    assert_eq!(saved.ledger, baseline.ledger);
    assert_eq!(saved.mappings, baseline.mappings);
    assert_eq!(saved.digests.files[DRAFT_PATH], digest(b""));
    assert_eq!(full.digests.files[DRAFT_PATH], digest(&bytes));
    for path in [LIBRARY_PATH, POLICY_PATH, LEDGER_PATH, MAPPING_PATH] {
        assert_eq!(saved.digests.files[path], baseline.digests.files[path]);
    }
    assert!(!serde_json::to_string(&saved)
        .unwrap()
        .contains("UNSAVED-MUST-NOT-EXPORT"));
    assert_eq!(std::fs::read(temp.path().join(DRAFT_PATH)).unwrap(), bytes);
    let guard = store.lock().unwrap();
    assert!(guard.snapshot_saved_unlocked().unwrap().draft.is_none());
    assert!(guard.snapshot_unlocked().unwrap().draft.is_some());
}
#[test]
fn bad_or_newer_draft_cannot_block_saved_reads_but_still_blocks_complete_snapshot() {
    for bad in [
        b"{".as_slice(),
        br#"{"schemaVersion":999,"notAReleasedDraft":true}"#.as_slice(),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let baseline = installed_saved_fixture(temp.path());
        let draft = temp.path().join(DRAFT_PATH);
        std::fs::write(&draft, bad).unwrap();
        let store = StoreV2::new(temp.path());
        let saved = store.snapshot_saved().unwrap();
        assert!(saved.draft.is_none());
        assert_eq!(saved.policy, baseline.policy);
        assert_eq!(saved.digests.files[DRAFT_PATH], digest(b""));
        assert!(store.snapshot().is_err());
        assert_eq!(std::fs::read(&draft).unwrap(), bad);
    }
}
#[cfg(unix)]
#[test]
fn saved_snapshot_never_checks_or_follows_a_symlinked_or_dangling_draft_target() {
    for dangling in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let baseline = installed_saved_fixture(temp.path());
        let target = external.path().join("do-not-open.json");
        if !dangling {
            std::fs::write(&target, b"not even JSON").unwrap();
        }
        let draft = temp.path().join(DRAFT_PATH);
        std::os::unix::fs::symlink(&target, &draft).unwrap();
        let store = StoreV2::new(temp.path());
        let saved = store.snapshot_saved().unwrap();
        assert_eq!(saved.policy, baseline.policy);
        assert!(saved.draft.is_none());
        assert!(store.snapshot().is_err());
        assert_eq!(std::fs::read_link(&draft).unwrap(), target);
        assert_eq!(target.exists(), !dangling);
        if !dangling {
            assert_eq!(std::fs::read(&target).unwrap(), b"not even JSON");
        }
    }
}
#[test]
fn saved_snapshot_keeps_strict_policy_ledger_mapping_and_library_checks() {
    for path in [LIBRARY_PATH, POLICY_PATH, LEDGER_PATH, MAPPING_PATH] {
        let temp = tempfile::tempdir().unwrap();
        installed_saved_fixture(temp.path());
        std::fs::write(temp.path().join(DRAFT_PATH), b"{").unwrap();
        let target = temp.path().join(path);
        let corrupt = if path == LEDGER_PATH {
            br#"{"schemaVersion":2,"ledger":{"sourcePolicyDigest":"sha256:0000000000000000000000000000000000000000000000000000000000000000","writtenFileDigest":"sha256:0000000000000000000000000000000000000000000000000000000000000000","fields":[]}}"#.as_slice()
        } else {
            br#"{"schemaVersion":1}"#.as_slice()
        };
        std::fs::write(&target, corrupt).unwrap();
        assert!(
            StoreV2::new(temp.path()).snapshot_saved().is_err(),
            "{path}"
        );
        assert_eq!(std::fs::read(target).unwrap(), corrupt);
    }
}
#[test]
fn saved_digest_placeholder_cannot_bypass_full_draft_reads_in_any_mutation() {
    let temp = tempfile::tempdir().unwrap();
    let saved = installed_saved_fixture(temp.path());
    let store = StoreV2::new(temp.path());
    let path = temp.path().join(DRAFT_PATH);
    std::fs::write(&path, b"{").unwrap();
    let current = store.snapshot_saved().unwrap();
    let guard = store.lock().unwrap();
    assert!(guard
        .prepare_save_unlocked(
            &current.digests,
            &current.policy,
            current.ledger.as_ref().unwrap(),
            &current.mappings
        )
        .is_err());
    assert!(guard
        .stage_draft_unlocked(None, baseline_draft_for_test(saved.digests.library.clone()))
        .is_err());
    let template = &saved.library.templates[0];
    let revision = &template.revisions[0];
    let request = SaveRevisionRequestV2 {
        expected_library: saved.digests.library.clone(),
        template_id: template.id.clone(),
        base_revision: Some(revision.id.clone()),
        revision_id: READ.into(),
        created_at: "2026-11-06T00:00:00Z".into(),
        label: template.label.clone(),
        content: revision.content.clone(),
        make_default: false,
        origin: super::schema::SaveOrigin::Manual,
    };
    assert!(guard.save_library_unlocked(&request).is_err());
    assert_eq!(std::fs::read(path).unwrap(), b"{");
    assert_eq!(
        std::fs::read(temp.path().join(LIBRARY_PATH)).unwrap(),
        canonical_bytes(&saved.library).unwrap()
    );
}
