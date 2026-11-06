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
    let raw = config();
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
