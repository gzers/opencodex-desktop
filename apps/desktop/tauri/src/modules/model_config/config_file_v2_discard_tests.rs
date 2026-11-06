use super::*;
use std::{path::PathBuf, sync::Mutex};
#[derive(Default)]
struct Vault {
    slots: Mutex<BTreeMap<String, String>>,
    removed: Mutex<Vec<String>>,
    fail: Mutex<bool>,
}
impl DraftCredentialStoreV2 for Vault {
    fn put(&self, id: &str, key: &str) -> Result<()> {
        self.slots.lock().unwrap().insert(id.into(), key.into());
        Ok(())
    }
    fn get(&self, id: &str) -> Result<Zeroizing<String>> {
        self.slots
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .map(Zeroizing::new)
            .ok_or_else(|| error("model_credential_missing"))
    }
    fn remove(&self, id: &str) -> Result<()> {
        self.removed.lock().unwrap().push(id.into());
        if *self.fail.lock().unwrap() {
            return Err(error("model_credential_store"));
        }
        self.slots.lock().unwrap().remove(id);
        Ok(())
    }
}
struct Fixture {
    _t: tempfile::TempDir,
    d: PathBuf,
    h: PathBuf,
    base: ReadBaseV2,
    id: String,
    vault: Vault,
}
impl Fixture {
    fn new() -> Self {
        let t = tempfile::tempdir().unwrap();
        let d = t.path().join("D");
        let h = t.path().join("H");
        std::fs::create_dir_all(d.join("manager-state")).unwrap();
        std::fs::create_dir_all(&h).unwrap();
        std::fs::write(h.join("config.json"),canonical_bytes(&json!({"providers":{"source":{"name":"source","adapter":"openai-chat","authMode":"key","baseUrl":"https://source.invalid/v1","apiKey":"SYNTHETIC-EXISTING-SAVED-KEY","models":["raw"],"selectedModels":["raw"]}}})).unwrap()).unwrap();
        let s = snapshot(&d, &h).unwrap();
        let s = begin_draft(
            &d,
            &h,
            ReadRequestV2 {
                expected_source: s.source_digest,
                expected_library: s.library_digest,
                expected_draft: None,
            },
        )
        .unwrap();
        let draft = s.draft.unwrap();
        Self {
            _t: t,
            d,
            h,
            id: draft.policy.channels[0].id.clone(),
            base: draft.base,
            vault: Vault::default(),
        }
    }
    fn replace(&mut self) -> String {
        let s = stage_credential_with_store(
            &self.d,
            &self.h,
            StageCredentialV2 {
                expected: self.base.clone(),
                channel_id: self.id.clone(),
                action: "replace".into(),
                value: Some("SYNTHETIC-PENDING-ONLY".into()),
            },
            &self.vault,
        )
        .unwrap();
        let draft = s.draft.unwrap();
        self.base = draft.base;
        draft.policy.channels[0]
            .connection
            .credential_ref
            .clone()
            .unwrap()
    }
    fn copy(&mut self) -> (String, std::path::PathBuf) {
        let s = copy_channel(
            &self.d,
            &self.h,
            CopyChannelV2 {
                expected: self.base.clone(),
                channel_id: self.id.clone(),
                name: "discard-copy".into(),
                reuse_source_key: true,
            },
        )
        .unwrap();
        let draft = s.draft.unwrap();
        self.base = draft.base;
        let c = draft
            .policy
            .channels
            .iter()
            .find(|c| c.id != self.id)
            .unwrap();
        (
            c.id.clone(),
            copy_path(&self.d, &c.connection.imported_config.as_ref().unwrap().id).unwrap(),
        )
    }
    fn business(&self) -> BTreeMap<PathBuf, String> {
        let mut out = BTreeMap::new();
        for path in [
            self.h.join("config.json"),
            self.d.join(store_v2::LIBRARY_PATH),
            self.d.join(store_v2::POLICY_PATH),
            self.d.join(store_v2::MAPPING_PATH),
            self.d.join(store_v2::LEDGER_PATH),
            self.d
                .join("manager-state/model-v2-provider-identities.json"),
            self.d
                .join(crate::modules::model_config::content_versions::SAVED_VERSIONS_PATH),
        ] {
            if path.exists() {
                out.insert(path.clone(), digest(&std::fs::read(path).unwrap()));
            }
        }
        out
    }
}
#[test]
fn explicit_discard_clears_only_the_guarded_draft_and_its_proven_private_material() {
    let mut f = Fixture::new();
    let reference = f.replace();
    let (_, copy) = f.copy();
    let before = f.business();
    let result = discard_with_store(&f.d, &f.h, f.base.clone(), &f.vault).unwrap();
    assert!(result.draft.is_none());
    assert_eq!(result.saved_policy.channels.len(), 1);
    assert_eq!(
        result.saved_policy.channels[0].connection.provider_id,
        "source"
    );
    assert!(result.saved_policy.channels[0].models[0].binding.is_none());
    assert_eq!(before, f.business());
    assert!(!copy.exists());
    assert!(!f.vault.slots.lock().unwrap().contains_key(&reference));
    assert_eq!(f.vault.removed.lock().unwrap().as_slice(), [reference]);
    assert!(!f.d.join(store_v2::DRAFT_PATH).exists());
}
#[test]
fn stale_source_generation_root_library_or_policy_never_discards_or_cleans_anything() {
    let mut f = Fixture::new();
    let reference = f.replace();
    let (_, copy) = f.copy();
    let before = f.business();
    let draft = std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap();
    for n in 0..5 {
        let mut expected = f.base.clone();
        match n {
            0 => expected.draft_generation += 1,
            1 => expected.root_revision = digest(b"wrong-root"),
            2 => expected.library_digest = digest(b"wrong-lib"),
            3 => expected.policy_digest = digest(b"wrong-policy"),
            _ => expected.source_digest = digest(b"wrong-source"),
        };
        assert!(discard_with_store(&f.d, &f.h, expected, &f.vault).is_err());
        assert_eq!(
            draft,
            std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap()
        );
        assert!(copy.exists());
        assert!(f.vault.slots.lock().unwrap().contains_key(&reference));
        assert!(f.vault.removed.lock().unwrap().is_empty());
        assert_eq!(before, f.business());
        assert!(!f.d.join(DISCARDS).exists());
    }
}
#[test]
fn discard_never_deletes_a_pending_slot_already_referenced_by_saved_channel() {
    let mut f = Fixture::new();
    let reference = f.replace();
    let (copy_id, copy) = f.copy();
    let saved = save_with_credentials(
        &f.d,
        &f.h,
        SaveRequestV2 {
            expected: f.base.clone(),
            edits: vec![],
            channel_intents: vec![],
            scope_channel_id: Some(copy_id),
        },
        &f.vault,
    )
    .unwrap();
    f.base = saved.draft.unwrap().base;
    let before = f.business();
    discard_with_store(&f.d, &f.h, f.base.clone(), &f.vault).unwrap();
    assert!(f.vault.slots.lock().unwrap().contains_key(&reference));
    assert!(f.vault.removed.lock().unwrap().is_empty());
    assert!(copy.exists());
    assert_eq!(before, f.business());
}
#[test]
fn cleanup_failure_leaves_only_attributed_private_tombstone_after_draft_invalidation() {
    let mut f = Fixture::new();
    let reference = f.replace();
    *f.vault.fail.lock().unwrap() = true;
    let before = f.business();
    let result = discard_with_store(&f.d, &f.h, f.base.clone(), &f.vault).unwrap();
    assert!(result.draft.is_none());
    assert_eq!(before, f.business());
    assert!(f.vault.slots.lock().unwrap().contains_key(&reference));
    let tombstones = std::fs::read_dir(f.d.join(DISCARDS))
        .unwrap()
        .map(|v| v.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(tombstones.len(), 1);
    let bytes = std::fs::read(&tombstones[0]).unwrap();
    let captured: DraftV2 = serde_json::from_value(parse_strict_json_v2(&bytes).unwrap()).unwrap();
    assert_eq!(captured.base, f.base);
    assert!(!String::from_utf8_lossy(&bytes).contains("SYNTHETIC-PENDING-ONLY"));
    assert!(!f.d.join(store_v2::DRAFT_PATH).exists());
}
#[test]
fn pending_journal_is_rejected_without_recovery_and_newer_generation_is_preserved() {
    let f = Fixture::new();
    let original = std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap();
    let pending = f.d.join("manager-state/model-transaction-v2.pending.json");
    std::fs::write(&pending, b"invalid journal preserved").unwrap();
    assert!(discard_with_store(&f.d, &f.h, f.base.clone(), &f.vault).is_err());
    assert_eq!(
        original,
        std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap()
    );
    assert_eq!(
        std::fs::read(&pending).unwrap(),
        b"invalid journal preserved"
    );
    std::fs::remove_file(pending).unwrap();
    let mut draft = StoreV2::new(&f.d).snapshot().unwrap().draft.unwrap();
    draft.base.draft_generation += 1;
    draft.policy.channels[0].label = "later generation".into();
    StoreV2::new(&f.d)
        .stage_draft(Some(&f.base), draft)
        .unwrap();
    let later = std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap();
    assert!(discard_with_store(&f.d, &f.h, f.base.clone(), &f.vault).is_err());
    assert_eq!(
        later,
        std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap()
    );
}
#[test]
fn discard_recovery_then_begin_new_generation_allows_ordinary_edit_and_save() {
    let mut f = Fixture::new();
    let saved = save(
        &f.d,
        &f.h,
        SaveRequestV2 {
            expected: f.base.clone(),
            edits: vec![ModelEditV2::SetOverride {
                channel_id: f.id.clone(),
                raw_model_id: "raw".into(),
                field: "contextWindow".into(),
                value: json!(999),
            }],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )
    .unwrap();
    let source = crate::modules::model_config::model_transaction_v2::history::list(&f.d, &f.h)
        .unwrap()
        .into_iter()
        .find(|e| e.has_official_config)
        .unwrap()
        .id;
    let begun = begin_draft(
        &f.d,
        &f.h,
        ReadRequestV2 {
            expected_source: saved.source_digest,
            expected_library: saved.library_digest,
            expected_draft: None,
        },
    )
    .unwrap();
    f.base = begun.draft.unwrap().base;
    let plan = history::prepare(
        &f.d,
        &f.h,
        history::HistoryPrepareV2 {
            source_id: source,
            side: history::SideV2::Before,
            scope: history::ScopeV2::Affected,
            restore_defaults: false,
            expected: f.base.clone(),
        },
    )
    .unwrap();
    f.base = history::accept(&f.d, &f.h, &plan.recovery_id)
        .unwrap()
        .draft
        .unwrap()
        .base;
    let before = f.business();
    let discarded = discard_with_store(&f.d, &f.h, f.base.clone(), &f.vault).unwrap();
    assert!(discarded.draft.is_none());
    assert_eq!(before, f.business());
    let fresh = begin_draft(
        &f.d,
        &f.h,
        ReadRequestV2 {
            expected_source: discarded.source_digest,
            expected_library: discarded.library_digest,
            expected_draft: None,
        },
    )
    .unwrap()
    .draft
    .unwrap();
    assert_ne!(fresh.base.draft_id, f.base.draft_id);
    assert!(fresh.policy.recovery_id.is_none());
    let fresh_bytes = std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap();
    assert!(history::accept(&f.d, &f.h, &plan.recovery_id).is_err());
    assert_eq!(
        fresh_bytes,
        std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap()
    );
    save(
        &f.d,
        &f.h,
        SaveRequestV2 {
            expected: fresh.base,
            edits: vec![ModelEditV2::SetOverride {
                channel_id: f.id,
                raw_model_id: "raw".into(),
                field: "contextWindow".into(),
                value: json!(777),
            }],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )
    .unwrap();
    assert_eq!(
        read_raw(&f.h).unwrap().0["providers"]["source"]["modelContextWindows"]["raw"],
        777
    );
}

#[test]
fn actual_source_change_after_edit_is_not_silently_discarded_or_cleaned() {
    let mut f = Fixture::new();
    let reference = f.replace();
    let (_, copy) = f.copy();
    let original = std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap();
    let (mut raw, _) = read_raw(&f.h).unwrap();
    raw["providers"]["source"]["baseUrl"] = json!("https://external-change.invalid/v1");
    std::fs::write(f.h.join("config.json"), canonical_bytes(&raw).unwrap()).unwrap();
    let actual = std::fs::read(f.h.join("config.json")).unwrap();
    assert!(discard_with_store(&f.d, &f.h, f.base.clone(), &f.vault).is_err());
    assert_eq!(
        original,
        std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap()
    );
    assert_eq!(actual, std::fs::read(f.h.join("config.json")).unwrap());
    assert!(copy.exists());
    assert!(f.vault.slots.lock().unwrap().contains_key(&reference));
    assert!(f.vault.removed.lock().unwrap().is_empty());
    assert!(!f.d.join(DISCARDS).exists());
}
