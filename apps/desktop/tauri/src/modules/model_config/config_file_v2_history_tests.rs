use super::*;
use std::path::PathBuf;
struct Fixture {
    _temp: tempfile::TempDir,
    d: PathBuf,
    h: PathBuf,
    source: String,
    expected: ReadBaseV2,
    channel: String,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let d = temp.path().join("D");
        let h = temp.path().join("H");
        std::fs::create_dir_all(d.join("manager-state")).unwrap();
        std::fs::create_dir_all(&h).unwrap();
        std::fs::write(h.join("config.json"),canonical_bytes(&json!({"providers":{"source":{"name":"source","adapter":"openai-chat","baseUrl":"https://source.invalid/v1","apiKey":"SYNTHETIC-HISTORY-SECRET","models":["vendor/raw"],"selectedModels":["vendor/raw"],"modelContextWindows":{"vendor/raw":8192}},"unselected":{"name":"unselected","adapter":"openai-chat","baseUrl":"https://unselected.invalid/v1","models":["other"],"selectedModels":["other"]}},"keep":{"unchanged":true}})).unwrap()).unwrap();
        let s = snapshot(&d, &h).unwrap();
        let started = begin_draft(
            &d,
            &h,
            ReadRequestV2 {
                expected_source: s.source_digest,
                expected_library: s.library_digest,
                expected_draft: None,
            },
        )
        .unwrap();
        let draft = started.draft.unwrap();
        let channel = draft.policy.channels[0].id.clone();
        save(
            &d,
            &h,
            SaveRequestV2 {
                expected: draft.base,
                edits: vec![ModelEditV2::SetOverride {
                    channel_id: channel.clone(),
                    raw_model_id: "vendor/raw".into(),
                    field: "contextWindow".into(),
                    value: json!(4096),
                }],
                channel_intents: vec![],
                scope_channel_id: None,
            },
        )
        .unwrap();
        let source = images::list(&d, &h)
            .unwrap()
            .into_iter()
            .find(|s| s.has_official_config)
            .unwrap()
            .id;
        let s = snapshot(&d, &h).unwrap();
        let begun = begin_draft(
            &d,
            &h,
            ReadRequestV2 {
                expected_source: s.source_digest,
                expected_library: s.library_digest,
                expected_draft: None,
            },
        )
        .unwrap();
        Self {
            _temp: temp,
            d,
            h,
            source,
            expected: begun.draft.unwrap().base,
            channel,
        }
    }
    fn request(&self) -> HistoryPrepareV2 {
        HistoryPrepareV2 {
            source_id: self.source.clone(),
            side: SideV2::Before,
            scope: ScopeV2::Affected,
            restore_defaults: false,
            expected: self.expected.clone(),
        }
    }
    fn business(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut files = BTreeMap::new();
        for p in [
            self.h.join("config.json"),
            self.d.join(store_v2::POLICY_PATH),
            self.d.join(store_v2::LIBRARY_PATH),
            self.d.join(store_v2::LEDGER_PATH),
            self.d.join(store_v2::MAPPING_PATH),
            self.d
                .join(crate::modules::model_config::content_versions::SAVED_VERSIONS_PATH),
        ] {
            if p.exists() {
                files.insert(p.clone(), std::fs::read(p).unwrap());
            }
        }
        files
    }
}
#[test]
fn real_history_before_side_prepares_accepts_only_draft_then_saves_with_same_transaction() {
    let f = Fixture::new();
    let before = f.business();
    let entries = list(&f.d, &f.h).unwrap();
    let row = entries.iter().find(|e| e.id == f.source).unwrap();
    assert!(row.recoverable);
    assert!(row.created_at.is_none());
    assert_eq!(row.phase, "committed");
    let draft_before = std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap();
    let plan = prepare(&f.d, &f.h, f.request()).unwrap();
    assert_eq!(before, f.business());
    assert_eq!(
        draft_before,
        std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap()
    );
    assert!(!serde_json::to_string(&plan)
        .unwrap()
        .contains("SYNTHETIC-HISTORY-SECRET"));
    let again = prepare(&f.d, &f.h, f.request()).unwrap();
    assert_eq!(again.recovery_id, plan.recovery_id);
    assert_eq!(again.target_digest, plan.target_digest);
    let accepted = accept(&f.d, &f.h, &plan.recovery_id).unwrap();
    let draft = accepted.draft.unwrap();
    assert_eq!(
        draft.policy.recovery_id.as_deref(),
        Some(plan.recovery_id.as_str())
    );
    assert_eq!(before, f.business());
    let replay = accept(&f.d, &f.h, &plan.recovery_id).unwrap();
    assert_eq!(replay.draft.as_ref().unwrap().base, draft.base);
    let saved = save(
        &f.d,
        &f.h,
        SaveRequestV2 {
            expected: draft.base,
            edits: vec![],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )
    .unwrap();
    assert!(saved.draft.is_none());
    assert!(saved.saved_policy.recovery_id.is_none());
    assert_eq!(
        read_raw(&f.h).unwrap().0["providers"]["source"]["modelContextWindows"]["vendor/raw"],
        8192
    );
    assert_eq!(
        read_raw(&f.h).unwrap().0["providers"]["source"]["apiKey"],
        "SYNTHETIC-HISTORY-SECRET"
    );
}
#[test]
fn missing_defaults_stale_guard_and_mixed_model_or_key_intents_keep_originals() {
    let f = Fixture::new();
    let before = f.business();
    let mut request = f.request();
    request.restore_defaults = true;
    assert_eq!(
        prepare(&f.d, &f.h, request).err().unwrap().code,
        "model_history_defaults_missing"
    );
    let mut request = f.request();
    request.expected.draft_generation += 1;
    assert!(prepare(&f.d, &f.h, request).is_err());
    assert_eq!(before, f.business());
    let plan = prepare(&f.d, &f.h, f.request()).unwrap();
    let accepted = accept(&f.d, &f.h, &plan.recovery_id).unwrap();
    let draft = accepted.draft.unwrap();
    let draft_bytes = std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap();
    let result = save(
        &f.d,
        &f.h,
        SaveRequestV2 {
            expected: draft.base.clone(),
            edits: vec![ModelEditV2::SetOverride {
                channel_id: f.channel.clone(),
                raw_model_id: "vendor/raw".into(),
                field: "contextWindow".into(),
                value: json!(1234),
            }],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    );
    assert_eq!(
        result.err().unwrap().code,
        "model_history_save_mixed_intent"
    );
    assert_eq!(before, f.business());
    assert_eq!(
        draft_bytes,
        std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap()
    );
    let mut modified = draft.clone();
    modified.base.draft_generation += 1;
    modified
        .credential_edits
        .insert(f.channel.clone(), DraftCredentialEditV2::Clear);
    modified.policy.channels[0].connection.credential_ref = None;
    let staged = StoreV2::new(&f.d)
        .stage_draft(Some(&draft.base), modified)
        .unwrap();
    assert!(save(
        &f.d,
        &f.h,
        SaveRequestV2 {
            expected: staged.base,
            edits: vec![],
            channel_intents: vec![],
            scope_channel_id: None
        }
    )
    .is_err());
    assert_eq!(before, f.business());
}
#[test]
fn corrupt_or_missing_exact_source_images_are_never_recovered_or_overwritten() {
    let f = Fixture::new();
    let before = f.business();
    let source = images::read(&f.d, &f.h, &f.source).unwrap();
    let journal =
        f.d.join("manager-state/model-transactions-v2")
            .join(&f.source)
            .join("journal.json");
    let bytes = std::fs::read(&journal).unwrap();
    let mut value: Value = serde_json::from_slice(&bytes).unwrap();
    value["checksum"] = json!(digest(b"bad"));
    std::fs::write(&journal, canonical_bytes(&value).unwrap()).unwrap();
    assert!(list(&f.d, &f.h).is_err());
    assert!(prepare(&f.d, &f.h, f.request()).is_err());
    assert_eq!(before, f.business());
    std::fs::write(&journal, bytes).unwrap();
    let original = images::read(&f.d, &f.h, &f.source).unwrap();
    assert_eq!(original.source_digest, source.source_digest);
    let image =
        f.d.join("manager-state/model-transactions-v2")
            .join(&f.source)
            .join("0.before");
    if image.exists() {
        std::fs::write(&image, b"corrupt").unwrap();
        assert!(prepare(&f.d, &f.h, f.request()).is_err());
        assert_eq!(before, f.business());
    }
}
#[test]
fn changed_source_after_prepare_and_later_draft_edits_refuse_accept_replay() {
    let f = Fixture::new();
    let plan = prepare(&f.d, &f.h, f.request()).unwrap();
    let mut draft = StoreV2::new(&f.d).snapshot().unwrap().draft.unwrap();
    let expected = draft.base.clone();
    draft.base.draft_generation += 1;
    draft.policy.channels[0].defaults.context_window =
        crate::modules::model_config::schema::Field::Custom(3333);
    StoreV2::new(&f.d)
        .stage_draft(Some(&expected), draft)
        .unwrap();
    let before = f.business();
    let kept = std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap();
    assert!(accept(&f.d, &f.h, &plan.recovery_id).is_err());
    assert_eq!(before, f.business());
    assert_eq!(kept, std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap());
}

#[test]
fn accepted_recovery_cannot_be_rematched_or_mixed_through_other_draft_entrypoints() {
    let f = Fixture::new();
    let plan = prepare(&f.d, &f.h, f.request()).unwrap();
    let accepted = accept(&f.d, &f.h, &plan.recovery_id).unwrap();
    let base = accepted.draft.unwrap().base;
    let before = f.business();
    let bytes = std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap();
    let request = ReadRequestV2 {
        expected_source: base.source_digest.clone(),
        expected_library: base.library_digest.clone(),
        expected_draft: Some(base.clone()),
    };
    assert_eq!(
        read_existing(&f.d, &f.h, request).err().unwrap().code,
        "model_history_draft_active"
    );
    assert_eq!(
        copy_channel(
            &f.d,
            &f.h,
            CopyChannelV2 {
                expected: base.clone(),
                channel_id: f.channel.clone(),
                name: "blocked".into(),
                reuse_source_key: false
            }
        )
        .err()
        .unwrap()
        .code,
        "model_history_draft_active"
    );
    assert_eq!(
        stage_credential(
            &f.d,
            &f.h,
            StageCredentialV2 {
                expected: base.clone(),
                channel_id: f.channel.clone(),
                action: "clear".into(),
                value: None
            }
        )
        .err()
        .unwrap()
        .code,
        "model_history_draft_active"
    );
    let projected = super::super::export::project(
        &f.d,
        &f.h,
        SaveRequestV2 {
            expected: base,
            edits: vec![ModelEditV2::ClearManualBinding {
                channel_id: f.channel.clone(),
                raw_model_id: "vendor/raw".into(),
            }],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    );
    assert!(projected.is_err());
    assert_eq!(before, f.business());
    assert_eq!(
        bytes,
        std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap()
    );
}
#[test]
fn accept_retry_repairs_interrupted_private_reply_without_new_draft_generation() {
    let f = Fixture::new();
    let plan = prepare(&f.d, &f.h, f.request()).unwrap();
    let first = accept(&f.d, &f.h, &plan.recovery_id)
        .unwrap()
        .draft
        .unwrap();
    let mut record = read_record(&f.d, &plan.recovery_id).unwrap();
    record.accepted_base = None;
    persist(&f.d, &record).unwrap();
    let replay = accept(&f.d, &f.h, &plan.recovery_id)
        .unwrap()
        .draft
        .unwrap();
    assert_eq!(first, replay);
}
#[test]
fn exact_library_defaults_are_staged_readonly_then_written_in_the_one_save_transaction() {
    use crate::modules::model_config::model_transaction_v2::{self, WriteV2};
    let mut f = Fixture::new();
    let mut before = StoreV2::new(&f.d).snapshot().unwrap().library;
    before.templates[0].default_revision = None;
    let mut after = before.clone();
    after.templates[0].default_revision = Some(after.templates[0].revisions[0].id.clone());
    std::fs::write(
        f.d.join(store_v2::LIBRARY_PATH),
        canonical_bytes(&before).unwrap(),
    )
    .unwrap();
    let mut writes = vec![WriteV2 {
        target: TargetV2::Manager(store_v2::LIBRARY_PATH.into()),
        before: Some(canonical_bytes(&before).unwrap()),
        after: Some(canonical_bytes(&after).unwrap()),
    }];
    let config = std::fs::read(f.h.join("config.json")).unwrap();
    writes.push(WriteV2 {
        target: TargetV2::OfficialConfig,
        before: Some(config.clone()),
        after: Some(config),
    });
    for relative in [
        store_v2::POLICY_PATH,
        store_v2::LEDGER_PATH,
        store_v2::MAPPING_PATH,
    ] {
        let bytes = std::fs::read(f.d.join(relative)).unwrap();
        writes.push(WriteV2 {
            target: TargetV2::Manager(relative.into()),
            before: Some(bytes.clone()),
            after: Some(bytes),
        });
    }
    let existing = images::list(&f.d, &f.h)
        .unwrap()
        .into_iter()
        .map(|r| r.id)
        .collect::<BTreeSet<_>>();
    model_transaction_v2::commit(&f.d, &f.h, writes).unwrap();
    f.source = images::list(&f.d, &f.h)
        .unwrap()
        .into_iter()
        .find(|r| !existing.contains(&r.id))
        .unwrap()
        .id;
    std::fs::remove_file(f.d.join(store_v2::DRAFT_PATH)).unwrap();
    let s = snapshot(&f.d, &f.h).unwrap();
    f.expected = begin_draft(
        &f.d,
        &f.h,
        ReadRequestV2 {
            expected_source: s.source_digest,
            expected_library: s.library_digest,
            expected_draft: None,
        },
    )
    .unwrap()
    .draft
    .unwrap()
    .base;
    let mut request = f.request();
    request.restore_defaults = true;
    let original = f.business();
    let plan = prepare(&f.d, &f.h, request).unwrap();
    assert_eq!(plan.default_changes.len(), 1);
    assert_eq!(original, f.business());
    let accepted = accept(&f.d, &f.h, &plan.recovery_id).unwrap();
    assert_eq!(original, f.business());
    let saved = save(
        &f.d,
        &f.h,
        SaveRequestV2 {
            expected: accepted.draft.unwrap().base,
            edits: vec![],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )
    .unwrap();
    assert!(saved.library.templates[0].default_revision.is_none());
    assert!(saved.saved_policy.recovery_id.is_none());
    assert_eq!(StoreV2::new(&f.d).snapshot_saved().unwrap().library, before);
}
#[cfg(unix)]
#[test]
fn transaction_history_reader_refuses_symlink_roots_and_modified_source_images() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let plan = prepare(&f.d, &f.h, f.request()).unwrap();
    let dir =
        f.d.join("manager-state/model-transactions-v2")
            .join(&f.source);
    let source = dir.join("0.before");
    let old = std::fs::read(&source).unwrap();
    let outside = f._temp.path().join("outside");
    std::fs::write(&outside, &old).unwrap();
    std::fs::remove_file(&source).unwrap();
    symlink(&outside, &source).unwrap();
    let before = f.business();
    assert!(accept(&f.d, &f.h, &plan.recovery_id).is_err());
    assert_eq!(old, std::fs::read(outside).unwrap());
    assert_eq!(before, f.business());
}

#[test]
fn affected_history_ignores_source_evidence_churn_and_preserves_unrelated_draft_edits() {
    let f = Fixture::new();
    let mut draft = StoreV2::new(&f.d).snapshot().unwrap().draft.unwrap();
    let old = draft.base.clone();
    let unselected = draft
        .policy
        .channels
        .iter_mut()
        .find(|c| c.label == "unselected")
        .unwrap();
    unselected.defaults.context_window = crate::modules::model_config::schema::Field::Custom(2222);
    let keep = unselected.clone();
    draft.base.draft_generation += 1;
    let staged = StoreV2::new(&f.d).stage_draft(Some(&old), draft).unwrap();
    let mut request = f.request();
    request.expected = staged.base;
    let plan = prepare(&f.d, &f.h, request).unwrap();
    let accepted = accept(&f.d, &f.h, &plan.recovery_id).unwrap();
    assert_eq!(
        accepted
            .draft
            .unwrap()
            .policy
            .channels
            .iter()
            .find(|c| c.id == keep.id),
        Some(&keep)
    );
    let accepted_order = StoreV2::new(&f.d)
        .snapshot()
        .unwrap()
        .draft
        .unwrap()
        .policy
        .channels
        .into_iter()
        .map(|c| c.id)
        .collect::<Vec<_>>();
    let observed_order = read_record(&f.d, &plan.recovery_id)
        .unwrap()
        .request
        .expected;
    assert_eq!(observed_order.draft_generation, old.draft_generation + 1);
    assert_eq!(accepted_order[0], f.channel);
}
#[test]
fn restoring_before_a_new_channel_uses_the_same_guarded_delete_transform_on_save() {
    let mut f = Fixture::new();
    let s = stage_connection(
        &f.d,
        &f.h,
        StageConnectionV2 {
            expected: Some(f.expected.clone()),
            expected_source: f.expected.source_digest.clone(),
            expected_library: f.expected.library_digest.clone(),
            channel_id: None,
            name: "created".into(),
            url: "https://created.invalid/v1".into(),
            nodes: None,
            active_node_id: None,
            auth_mode: "none".into(),
            api_key: None,
            raw_model_ids: vec!["new/raw".into()],
            selected_model_ids: vec!["new/raw".into()],
        },
    )
    .unwrap();
    let draft = s.draft.unwrap();
    let id = draft
        .policy
        .channels
        .iter()
        .find(|c| c.label == "created")
        .unwrap()
        .id
        .clone();
    let old = images::list(&f.d, &f.h)
        .unwrap()
        .into_iter()
        .map(|e| e.id)
        .collect::<BTreeSet<_>>();
    let saved = save(
        &f.d,
        &f.h,
        SaveRequestV2 {
            expected: draft.base,
            edits: vec![ModelEditV2::SetOverride {
                channel_id: id.clone(),
                raw_model_id: "new/raw".into(),
                field: "protocol".into(),
                value: json!("openai-chat"),
            }],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )
    .unwrap();
    assert!(saved.saved_policy.channels.iter().any(|c| c.id == id));
    f.source = images::list(&f.d, &f.h)
        .unwrap()
        .into_iter()
        .find(|e| !old.contains(&e.id) && e.has_official_config)
        .unwrap()
        .id;
    f.expected = begin_draft(
        &f.d,
        &f.h,
        ReadRequestV2 {
            expected_source: saved.source_digest,
            expected_library: saved.library_digest,
            expected_draft: None,
        },
    )
    .unwrap()
    .draft
    .unwrap()
    .base;
    let plan = prepare(&f.d, &f.h, f.request()).unwrap();
    let draft = accept(&f.d, &f.h, &plan.recovery_id)
        .unwrap()
        .draft
        .unwrap();
    assert!(!draft.policy.channels.iter().any(|c| c.id == id));
    assert!(read_raw(&f.h).unwrap().0["providers"]
        .get("created")
        .is_some());
    save(
        &f.d,
        &f.h,
        SaveRequestV2 {
            expected: draft.base,
            edits: vec![],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )
    .unwrap();
    assert!(read_raw(&f.h).unwrap().0["providers"]
        .get("created")
        .is_none());
}

#[test]
fn cross_endpoint_history_never_forwards_current_or_historical_key_to_another_direction() {
    let mut f = Fixture::new();
    let (mut raw, _) = read_raw(&f.h).unwrap();
    raw["providers"]["source"]["baseUrl"] = json!("https://different.invalid/v1");
    std::fs::write(f.h.join("config.json"), canonical_bytes(&raw).unwrap()).unwrap();
    std::fs::remove_file(f.d.join(store_v2::DRAFT_PATH)).unwrap();
    let current = snapshot(&f.d, &f.h).unwrap();
    f.expected = begin_draft(
        &f.d,
        &f.h,
        ReadRequestV2 {
            expected_source: current.source_digest,
            expected_library: current.library_digest,
            expected_draft: None,
        },
    )
    .unwrap()
    .draft
    .unwrap()
    .base;
    let before = f.business();
    assert_eq!(
        prepare(&f.d, &f.h, f.request()).err().unwrap().code,
        "model_history_credential_direction_review"
    );
    assert_eq!(before, f.business());
    assert_eq!(
        read_raw(&f.h).unwrap().0["providers"]["source"]["apiKey"],
        "SYNTHETIC-HISTORY-SECRET"
    );
}
#[test]
fn history_exact_dependencies_and_bounded_enumeration_do_not_guess_missing_revisions() {
    let f = Fixture::new();
    let state = StoreV2::new(&f.d).snapshot().unwrap();
    let reference = RefV2 {
        template_id: state.library.templates[0].id.clone(),
        revision_id: state.library.templates[0].revisions[0].id.clone(),
        digest: state.library.templates[0].revisions[0].digest.clone(),
    };
    let old = images::list(&f.d, &f.h)
        .unwrap()
        .into_iter()
        .map(|r| r.id)
        .collect::<BTreeSet<_>>();
    save(
        &f.d,
        &f.h,
        SaveRequestV2 {
            expected: f.expected.clone(),
            edits: vec![ModelEditV2::SetManualBinding {
                channel_id: f.channel.clone(),
                raw_model_id: "vendor/raw".into(),
                reference: reference.clone(),
            }],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )
    .unwrap();
    let adopted = images::list(&f.d, &f.h)
        .unwrap()
        .into_iter()
        .find(|r| !old.contains(&r.id) && r.has_official_config)
        .unwrap();
    let source = images::read(&f.d, &f.h, &adopted.id).unwrap();
    let mut missing = state.library.clone();
    missing.templates.retain(|t| t.id != reference.template_id);
    assert_eq!(
        side_policy(&source, SideV2::After, &missing)
            .err()
            .unwrap()
            .code,
        "model_dependency_missing"
    );
    let directory = f.d.join("manager-state/model-transactions-v2");
    for _ in 0..256 {
        std::fs::create_dir(directory.join(uuid::Uuid::new_v4().to_string())).unwrap();
    }
    assert_eq!(images::list(&f.d, &f.h).err().unwrap().code, "model_limit");
}
