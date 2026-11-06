use super::*;
use tempfile::TempDir;
fn fixture(raw: Value) -> (TempDir, std::path::PathBuf, std::path::PathBuf) {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("user/desktop");
    let home = t.path().join("user/opencodex/home");
    std::fs::create_dir_all(root.join("manager-state")).unwrap();
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(home.join("config.json"), serde_json::to_vec(&raw).unwrap()).unwrap();
    (t, root, home)
}
fn raw() -> Value {
    json!({"providers":{"Krill-1":{"name":"Krill","adapter":"openai-chat","authMode":"key","apiKey":"synthetic-only-never-return","baseUrl":"https://fixture.invalid/v1","models":["gpt-6-sol","vendor/custom"],"selectedModels":["gpt-6-sol","vendor/custom"],"modelContextWindows":{"vendor/custom":4096},"unknown":{"keep":true}}},"defaultProvider":"Krill-1","defaultModel":"Krill-1/vendor/custom","modelPickerOrder":["Krill-1/gpt-6-sol","Krill-1/vendor/custom"],"unrelated":{"keep":true}})
}
fn request(s: &FileSnapshotV2) -> ReadRequestV2 {
    ReadRequestV2 {
        expected_source: s.source_digest.clone(),
        expected_library: s.library_digest.clone(),
        expected_draft: s.draft.as_ref().map(|d| d.base.clone()),
    }
}
fn begin(root: &Path, home: &Path) -> FileSnapshotV2 {
    let s = snapshot(root, home).unwrap();
    begin_draft(root, home, request(&s)).unwrap()
}
#[test]
fn name_is_actual_provider_authority_even_when_display_label_was_already_changed() {
    let (_t, root, home) = fixture(raw());
    let original = std::fs::read(home.join("config.json")).unwrap();
    let started = begin(&root, &home);
    assert_eq!(std::fs::read(home.join("config.json")).unwrap(), original);
    assert!(!serde_json::to_string(&started)
        .unwrap()
        .contains("synthetic-only-never-return"));
    let channel = started.draft.as_ref().unwrap().policy.channels[0].clone();
    assert!(channel.models.iter().all(|m| m.binding.is_none())); // begin never auto-selects
    let result = save(
        &root,
        &home,
        SaveRequestV2 {
            expected: started.draft.unwrap().base,
            edits: vec![],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )
    .unwrap();
    assert!(result.draft.is_none());
    let (raw, _) = read_raw(&home).unwrap();
    assert!(raw["providers"].get("Krill-1").is_none());
    assert!(raw["providers"].get("Krill").is_some());
    assert_eq!(raw["providers"]["Krill"]["name"], "Krill");
    assert_eq!(
        raw["providers"]["Krill"]["apiKey"],
        "synthetic-only-never-return"
    );
    assert_eq!(raw["defaultModel"], "Krill/vendor/custom");
    assert_eq!(raw["unrelated"], json!({"keep":true}));
    assert_eq!(result.saved_policy.channels[0].id, channel.id);
    let reopened = snapshot(&root, &home).unwrap();
    assert_eq!(reopened.saved_policy, result.saved_policy);
    let next = begin_draft(&root, &home, request(&reopened)).unwrap();
    assert_eq!(
        next.draft.as_ref().unwrap().policy.channels[0].id,
        channel.id
    );
    assert!(next.draft.as_ref().unwrap().policy.channels[0]
        .models
        .iter()
        .all(|m| m.binding.is_none()));
}
#[test]
fn explicit_read_matches_and_repeat_is_idempotent_save_reopen_preserves_source() {
    let (_t, root, home) = fixture(raw());
    let s = snapshot(&root, &home).unwrap();
    let read = read_existing(&root, &home, request(&s)).unwrap();
    assert!(!read.results.is_empty());
    let d = read.snapshot.draft.as_ref().unwrap();
    assert!(d.policy.channels[0]
        .models
        .iter()
        .any(|m| matches!(m.binding, Some(ModelBindingV2::AutoRead { .. }))));
    let before = std::fs::read(root.join(store_v2::DRAFT_PATH)).unwrap();
    let repeated = read_existing(&root, &home, request(&read.snapshot)).unwrap();
    assert_eq!(
        repeated
            .snapshot
            .draft
            .as_ref()
            .unwrap()
            .base
            .draft_generation,
        d.base.draft_generation
    );
    assert_eq!(
        std::fs::read(root.join(store_v2::DRAFT_PATH)).unwrap(),
        before
    );
    let saved = save(
        &root,
        &home,
        SaveRequestV2 {
            expected: repeated.snapshot.draft.unwrap().base,
            edits: vec![],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )
    .unwrap();
    let reopened = begin_draft(&root, &home, request(&saved)).unwrap();
    let p = &reopened.draft.as_ref().unwrap().policy;
    assert_eq!(
        p.channels[0]
            .models
            .iter()
            .find(|m| m.raw_model_id == "vendor/custom")
            .unwrap()
            .source
            .fields["contextWindow"]
            .value,
        json!(4096)
    );
    assert!(p.channels[0]
        .models
        .iter()
        .all(|m| m.explicit_overrides == ModelRules::default()));
}
#[test]
fn stale_source_and_duplicate_raw_json_never_modify_saved_or_draft() {
    let (_t, root, home) = fixture(raw());
    let s = begin(&root, &home);
    let draft = std::fs::read(root.join(store_v2::DRAFT_PATH)).unwrap();
    std::fs::write(
        home.join("config.json"),
        br#"{"providers":{},"providers":{}}"#,
    )
    .unwrap();
    assert!(
        matches!(save(&root,&home,SaveRequestV2{expected:s.draft.unwrap().base,edits:vec![],channel_intents:vec![],scope_channel_id:None}),Err(e) if e.code=="model_json_duplicate_key")
    );
    assert_eq!(
        std::fs::read(root.join(store_v2::DRAFT_PATH)).unwrap(),
        draft
    );
    assert_eq!(
        std::fs::read(home.join("config.json")).unwrap(),
        br#"{"providers":{},"providers":{}}"#
    );
}
#[test]
fn staging_new_no_auth_channel_is_draft_only_and_read_preserves_it() {
    let (_t, root, home) = fixture(raw());
    let s = begin(&root, &home);
    let before = std::fs::read(home.join("config.json")).unwrap();
    let staged = stage_connection(
        &root,
        &home,
        StageConnectionV2 {
            expected: s.draft.as_ref().map(|d| d.base.clone()),
            expected_source: s.source_digest,
            expected_library: s.library_digest,
            channel_id: None,
            name: "新渠道".into(),
            url: "https://new.fixture.invalid/v1".into(),
            nodes: None,
            active_node_id: None,
            auth_mode: "none".into(),
            api_key: None,
            raw_model_ids: vec!["gpt-6-sol".into()],
            selected_model_ids: vec!["gpt-6-sol".into()],
        },
    )
    .unwrap();
    assert_eq!(std::fs::read(home.join("config.json")).unwrap(), before);
    let again = read_existing(&root, &home, request(&staged))
        .unwrap()
        .snapshot;
    assert!(again
        .draft
        .as_ref()
        .unwrap()
        .policy
        .channels
        .iter()
        .any(|c| c.label == "新渠道"));
    let saved = save(
        &root,
        &home,
        SaveRequestV2 {
            expected: again.draft.unwrap().base,
            edits: vec![],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )
    .unwrap();
    let c = saved
        .saved_policy
        .channels
        .iter()
        .find(|c| c.label == "新渠道")
        .unwrap();
    assert!(c.connection.provider_id.starts_with("channel-"));
    let (raw, _) = read_raw(&home).unwrap();
    assert_eq!(
        raw["providers"][&c.connection.provider_id]["name"],
        "新渠道"
    );
}

#[test]
fn fresh_read_does_not_turn_old_connection_source_into_manual_override() {
    let (_t, root, home) = fixture(raw());
    let first = begin(&root, &home);
    let mut next = raw();
    next["providers"]["Krill-1"]["baseUrl"] = json!("https://external.fixture.invalid/v2");
    next["providers"]["Krill-1"]["selectedModels"] = json!(["vendor/custom"]);
    std::fs::write(home.join("config.json"), serde_json::to_vec(&next).unwrap()).unwrap();
    let observed = snapshot(&root, &home).unwrap();
    let read = read_existing(&root, &home, request(&observed)).unwrap();
    let channel = &read.snapshot.draft.as_ref().unwrap().policy.channels[0];
    assert_eq!(
        channel.connection.nodes[0].url,
        "https://external.fixture.invalid/v2"
    );
    assert_eq!(
        channel.selection,
        ModelSelection::Whitelist(vec!["vendor/custom".into()])
    );
    assert_eq!(
        first.draft.as_ref().unwrap().policy.channels[0]
            .connection
            .nodes[0]
            .url,
        "https://fixture.invalid/v1"
    );
}
#[test]
fn copied_channel_keeps_actual_key_and_unknown_fields_only_on_explicit_save() {
    let (_t, root, home) = fixture(raw());
    let started = begin(&root, &home);
    let channel = started.draft.as_ref().unwrap().policy.channels[0].clone();
    let before = std::fs::read(home.join("config.json")).unwrap();
    let copied = copy_channel(
        &root,
        &home,
        CopyChannelV2 {
            expected: started.draft.unwrap().base,
            channel_id: channel.id.clone(),
            name: "Krill-copy".into(),
        },
    )
    .unwrap();
    assert_eq!(std::fs::read(home.join("config.json")).unwrap(), before);
    let new_id = copied
        .draft
        .as_ref()
        .unwrap()
        .policy
        .channels
        .iter()
        .find(|c| c.label == "Krill-copy")
        .unwrap()
        .id
        .clone();
    assert_ne!(new_id, channel.id);
    let saved = save(
        &root,
        &home,
        SaveRequestV2 {
            expected: copied.draft.unwrap().base,
            edits: vec![],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )
    .unwrap();
    let (raw, _) = read_raw(&home).unwrap();
    assert_eq!(
        raw["providers"]["Krill-copy"]["apiKey"],
        "synthetic-only-never-return"
    );
    assert_eq!(
        raw["providers"]["Krill-copy"]["unknown"],
        json!({"keep":true})
    );
    assert!(saved.saved_policy.channels.iter().any(|c| c.id == new_id));
}
#[test]
fn single_channel_save_keeps_other_staged_work_and_rebases_its_guard() {
    let (_t, root, home) = fixture(raw());
    let s = begin(&root, &home);
    let original_id = s.draft.as_ref().unwrap().policy.channels[0].id.clone();
    let staged = stage_connection(
        &root,
        &home,
        StageConnectionV2 {
            expected: s.draft.as_ref().map(|d| d.base.clone()),
            expected_source: s.source_digest,
            expected_library: s.library_digest,
            channel_id: None,
            name: "later".into(),
            url: "https://later.fixture.invalid/v1".into(),
            nodes: None,
            active_node_id: None,
            auth_mode: "none".into(),
            api_key: None,
            raw_model_ids: vec!["gpt-6-sol".into()],
            selected_model_ids: vec!["gpt-6-sol".into()],
        },
    )
    .unwrap();
    let before = staged.draft.as_ref().unwrap().base.clone();
    let partial = save(
        &root,
        &home,
        SaveRequestV2 {
            expected: before.clone(),
            edits: vec![],
            channel_intents: vec![ChannelIntentV2::Rename {
                channel_id: original_id.clone(),
                name: "chosen".into(),
            }],
            scope_channel_id: Some(original_id),
        },
    )
    .unwrap();
    let (raw, _) = read_raw(&home).unwrap();
    assert!(raw["providers"].get("chosen").is_some());
    assert!(raw["providers"].get("later").is_none());
    let draft = partial.draft.as_ref().unwrap();
    assert!(draft.policy.channels.iter().any(|c| c.label == "later"));
    assert_ne!(draft.base.source_digest, before.source_digest);
    assert!(draft.base.draft_generation > before.draft_generation);
    let completed = save(
        &root,
        &home,
        SaveRequestV2 {
            expected: draft.base.clone(),
            edits: vec![],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )
    .unwrap();
    assert!(completed.draft.is_none());
    assert!(read_raw(&home).unwrap().0["providers"]
        .get("later")
        .is_some());
}
#[test]
fn saved_times_are_bound_to_exact_semantics_and_committed_with_models() {
    let (_t, root, home) = fixture(raw());
    let s = begin(&root, &home);
    let channel = s.draft.as_ref().unwrap().policy.channels[0].clone();
    let result = save(
        &root,
        &home,
        SaveRequestV2 {
            expected: s.draft.unwrap().base,
            edits: vec![ModelEditV2::SetOverride {
                channel_id: channel.id,
                raw_model_id: "vendor/custom".into(),
                field: "contextWindow".into(),
                value: json!(8192),
            }],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )
    .unwrap();
    let (raw, _) = read_raw(&home).unwrap();
    let state = saved_state(&result.library, &result.saved_policy, &raw).unwrap();
    let versions = crate::modules::model_config::content_versions::observe(&root, &state).unwrap();
    assert!(versions
        .iter()
        .any(|v| v.time_kind == crate::modules::model_config::content_versions::TimeKind::Saved));
    assert!(root
        .join(crate::modules::model_config::content_versions::SAVED_VERSIONS_PATH)
        .exists());
    assert!(!root
        .join("manager-state/model-transaction-v2.pending.json")
        .exists());
}

#[test]
fn library_revision_and_time_commit_together_without_rematching_channel_draft() {
    let (_t, root, home) = fixture(raw());
    let started = begin(&root, &home);
    let original = std::fs::read(home.join("config.json")).unwrap();
    let old_draft = std::fs::read(root.join(store_v2::DRAFT_PATH)).unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let revision_id = uuid::Uuid::new_v4().to_string();
    let next = save_revision(
        &root,
        &home,
        &SaveRevisionRequestV2 {
            expected_library: started.library_digest,
            template_id: id.clone(),
            base_revision: None,
            revision_id: revision_id.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            label: "custom template".into(),
            content: started.library.templates[0].revisions[0].content.clone(),
            make_default: true,
            origin: crate::modules::model_config::schema::SaveOrigin::Manual,
        },
    )
    .unwrap();
    assert_eq!(
        next.templates
            .iter()
            .find(|t| t.id == id)
            .unwrap()
            .default_revision
            .as_deref(),
        Some(revision_id.as_str())
    );
    assert_eq!(std::fs::read(home.join("config.json")).unwrap(), original);
    assert_eq!(
        std::fs::read(root.join(store_v2::DRAFT_PATH)).unwrap(),
        old_draft
    );
    let atoms = saved_state(&next, &store_v2::empty_policy_v2(), &json!({})).unwrap();
    assert!(crate::modules::model_config::content_versions::observe(&root,&atoms).unwrap().iter().any(|v|v.time_kind==crate::modules::model_config::content_versions::TimeKind::Saved));
}
#[test]
fn explicit_delete_removes_only_owned_channel_routes_and_can_keep_other_draft() {
    let (_t, root, home) = fixture(raw());
    let mut file = raw();
    file["providers"]["unmanaged"] = json!({"adapter":"openai-chat","baseUrl":"https://other.fixture.invalid","models":["x"],"apiKey":"other-synthetic","opaque":9});
    std::fs::write(home.join("config.json"), serde_json::to_vec(&file).unwrap()).unwrap();
    let s = begin(&root, &home);
    let id = s
        .draft
        .as_ref()
        .unwrap()
        .policy
        .channels
        .iter()
        .find(|c| c.connection.provider_id == "Krill-1")
        .unwrap()
        .id
        .clone();
    let saved = save(
        &root,
        &home,
        SaveRequestV2 {
            expected: s.draft.unwrap().base,
            edits: vec![],
            channel_intents: vec![ChannelIntentV2::Delete {
                channel_id: id.clone(),
            }],
            scope_channel_id: Some(id),
        },
    )
    .unwrap();
    let (after, _) = read_raw(&home).unwrap();
    assert!(after["providers"].get("Krill-1").is_none());
    assert_eq!(
        after["providers"]["unmanaged"],
        file["providers"]["unmanaged"]
    );
    assert!(saved
        .draft
        .as_ref()
        .unwrap()
        .policy
        .channels
        .iter()
        .any(|c| c.connection.provider_id == "unmanaged"));
}

#[derive(Default)]
struct FakeDraftCredentials(std::sync::Mutex<BTreeMap<String, String>>);
impl DraftCredentialStoreV2 for FakeDraftCredentials {
    fn put(&self, id: &str, value: &str) -> Result<()> {
        self.0.lock().unwrap().insert(id.into(), value.into());
        Ok(())
    }
    fn get(&self, id: &str) -> Result<Zeroizing<String>> {
        self.0
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .map(Zeroizing::new)
            .ok_or_else(|| error("model_credential_missing"))
    }
    fn remove(&self, id: &str) -> Result<()> {
        self.0.lock().unwrap().remove(id);
        Ok(())
    }
}
#[test]
fn explicit_key_only_replace_preserves_other_pool_slots_and_clear_keeps_auth_mode() {
    let mut file = raw();
    file["providers"]["Krill-1"]["apiKeyPool"] = json!([{"id":"first","key":"synthetic-only-never-return","label":"active","future":7},{"id":"other","key":"other-synthetic","future":8}]);
    let (_t, root, home) = fixture(file.clone());
    let s = begin(&root, &home);
    let id = s.draft.as_ref().unwrap().policy.channels[0].id.clone();
    let vault = FakeDraftCredentials::default();
    let staged = stage_credential_with_store(
        &root,
        &home,
        StageCredentialV2 {
            expected: s.draft.unwrap().base,
            channel_id: id.clone(),
            action: "replace".into(),
            value: Some("new-synthetic".into()),
        },
        &vault,
    )
    .unwrap();
    assert_eq!(read_raw(&home).unwrap().0, file);
    assert!(!serde_json::to_string(&staged)
        .unwrap()
        .contains("new-synthetic"));
    let view = credential_view_with_store(&root, &home, &id, &vault).unwrap();
    assert_eq!(view.value, "new-synthetic");
    let saved = save_with_credentials(
        &root,
        &home,
        SaveRequestV2 {
            expected: staged.draft.unwrap().base,
            edits: vec![],
            channel_intents: vec![],
            scope_channel_id: None,
        },
        &vault,
    )
    .unwrap();
    let (raw, _) = read_raw(&home).unwrap();
    assert_eq!(raw["providers"]["Krill"]["apiKey"], "new-synthetic");
    assert_eq!(
        raw["providers"]["Krill"]["apiKeyPool"][0]["key"],
        "new-synthetic"
    );
    assert_eq!(raw["providers"]["Krill"]["apiKeyPool"][0]["future"], 7);
    assert_eq!(
        raw["providers"]["Krill"]["apiKeyPool"][1],
        file["providers"]["Krill-1"]["apiKeyPool"][1]
    );
    let next = begin_draft(&root, &home, request(&saved)).unwrap();
    let cleared = stage_credential_with_store(
        &root,
        &home,
        StageCredentialV2 {
            expected: next.draft.unwrap().base,
            channel_id: id.clone(),
            action: "clear".into(),
            value: None,
        },
        &vault,
    )
    .unwrap();
    assert_eq!(
        cleared.draft.as_ref().unwrap().policy.channels[0]
            .connection
            .auth_mode
            .as_deref(),
        Some("key")
    );
    assert_eq!(
        read_raw(&home).unwrap().0["providers"]["Krill"]["apiKey"],
        "new-synthetic"
    );
    let again = read_existing(&root, &home, request(&cleared))
        .unwrap()
        .snapshot;
    assert!(matches!(
        again.draft.as_ref().unwrap().credential_edits.get(&id),
        Some(DraftCredentialEditV2::Clear)
    ));
    save_with_credentials(
        &root,
        &home,
        SaveRequestV2 {
            expected: again.draft.unwrap().base,
            edits: vec![],
            channel_intents: vec![],
            scope_channel_id: None,
        },
        &vault,
    )
    .unwrap();
    let (raw, _) = read_raw(&home).unwrap();
    assert!(raw["providers"]["Krill"].get("apiKey").is_none());
    assert!(raw["providers"]["Krill"].get("apiKeyPool").is_none());
    assert_eq!(raw["providers"]["Krill"]["authMode"], "key");
}
#[test]
fn saved_capture_source_identity_is_stable_and_unaffected_by_unsaved_channel_name() {
    let (_t, root, home) = fixture(raw());
    let source =
        || with_saved_config(&root, &home, |_, policy, _, _, _, _| Ok(policy.clone())).unwrap();
    let first = source();
    assert_eq!(source(), first);
    let draft = begin(&root, &home);
    let saved_id = first.channels[0].id.clone();
    assert_eq!(
        draft.draft.as_ref().unwrap().policy.channels[0].id,
        saved_id
    );
    let mut staged = draft.draft.unwrap();
    staged.policy.channels[0].label = "unsaved display".into();
    let expected = staged.base.clone();
    staged.base.draft_generation += 1;
    StoreV2::new(&root)
        .stage_draft(Some(&expected), staged)
        .unwrap();
    assert_eq!(source().channels[0].label, "Krill");
    assert_eq!(source().channels[0].id, saved_id);
}

#[test]
fn saved_capture_ignores_corrupt_draft_and_never_recovers_pending_business_writes() {
    let (_t, root, home) = fixture(raw());
    let before = std::fs::read(home.join("config.json")).unwrap();
    std::fs::write(
        root.join(store_v2::DRAFT_PATH),
        b"invalid uncommitted draft",
    )
    .unwrap();
    let capture = with_saved_config(&root, &home, |_, p, _, _, _, _| Ok(p.clone())).unwrap();
    assert_eq!(capture.channels[0].label, "Krill");
    assert_eq!(
        std::fs::read(root.join(store_v2::DRAFT_PATH)).unwrap(),
        b"invalid uncommitted draft"
    );
    std::fs::write(
        root.join("manager-state/model-transaction-v2.pending.json"),
        b"unconfirmed frozen operation",
    )
    .unwrap();
    assert!(
        matches!(with_saved_config(&root,&home,|_,_,_,_,_,_|Ok(())),Err(e)if e.code=="model_transaction_pending")
    );
    assert_eq!(std::fs::read(home.join("config.json")).unwrap(), before);
    assert_eq!(
        std::fs::read(root.join("manager-state/model-transaction-v2.pending.json")).unwrap(),
        b"unconfirmed frozen operation"
    );
}
#[test]
fn node_measurement_uses_explicit_kind_and_only_acknowledged_node_ids() {
    let (_t, root, home) = fixture(raw());
    let started = begin(&root, &home);
    let d = started.draft.as_ref().unwrap();
    let c = &d.policy.channels[0];
    let request = MeasurementIntentV2 {
        expected: d.base.clone(),
        channel_id: c.id.clone(),
        node_ids: vec![c.connection.active_node_id.clone()],
        discovery_kind: "google_list".into(),
        run_id: uuid::Uuid::new_v4().to_string(),
    };
    let captured =
        prepare_measurements_with_store(&root, &home, request, &FakeDraftCredentials::default())
            .unwrap();
    assert_eq!(captured.protocol, "google");
    assert_eq!(
        captured.api_key.as_deref(),
        Some("synthetic-only-never-return")
    );
    assert!(captured.provider_id.is_none());
    assert!(captured.home.is_none());
    let request = MeasurementIntentV2 {
        expected: d.base.clone(),
        channel_id: c.id.clone(),
        node_ids: vec![uuid::Uuid::new_v4().to_string()],
        discovery_kind: "openai_list".into(),
        run_id: uuid::Uuid::new_v4().to_string(),
    };
    assert!(
        matches!(prepare_measurements_with_store(&root,&home,request,&FakeDraftCredentials::default()),Err(e)if e.code=="model_node_missing")
    );
    let mut changed = d.clone();
    let expected = changed.base.clone();
    changed.base.draft_generation += 1;
    changed.policy.channels[0].connection.nodes[0].url = "https://other.fixture.invalid".into();
    StoreV2::new(&root)
        .stage_draft(Some(&expected), changed.clone())
        .unwrap();
    let request = MeasurementIntentV2 {
        expected: changed.base,
        channel_id: c.id.clone(),
        node_ids: vec![c.connection.active_node_id.clone()],
        discovery_kind: "openai_list".into(),
        run_id: uuid::Uuid::new_v4().to_string(),
    };
    assert!(
        matches!(prepare_measurements_with_store(&root,&home,request,&FakeDraftCredentials::default()),Err(e)if e.code=="model_node_credential_unconfirmed")
    );
}
