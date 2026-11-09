use super::*;
use std::{path::PathBuf, sync::Mutex};
#[derive(Default)]
struct Vault(Mutex<BTreeMap<String, String>>);
impl DraftCredentialStoreV2 for Vault {
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
        let raw = json!({"providers":{"source":{"name":"source","adapter":"openai-chat","authMode":"key","baseUrl":"https://source.invalid/v1","apiKey":"$COPY_TEST_SOURCE_KEY","apiKeyPool":[{"id":"pool","key":"keychain:source/pool"}],"headers":{"Authorization":"keychain:source/auth"},"proxy":"https://private.invalid","mcpServers":{"hidden":{"key":"private"}},"desktopExecutor":{"hidden":true},"apiKeySelectionRevision":7,"_apiKeyAttempt":1,"models":["vendor/model"],"selectedModels":["vendor/model"],"unknown":{"keep":true}}}});
        std::fs::write(h.join("config.json"), canonical_bytes(&raw).unwrap()).unwrap();
        let snapshot = snapshot(&d, &h).unwrap();
        let state = begin_draft(
            &d,
            &h,
            ReadRequestV2 {
                expected_source: snapshot.source_digest,
                expected_library: snapshot.library_digest,
                expected_draft: None,
            },
        )
        .unwrap();
        let draft = state.draft.unwrap();
        Self {
            _t: t,
            d,
            h,
            base: draft.base,
            id: draft.policy.channels[0].id.clone(),
            vault: Vault::default(),
        }
    }
    fn copy(&mut self, reuse: bool) -> FileSnapshotV2 {
        let result = copy_channel(
            &self.d,
            &self.h,
            CopyChannelV2 {
                expected: self.base.clone(),
                channel_id: self.id.clone(),
                name: "copy".into(),
                reuse_source_key: reuse,
            },
        )
        .unwrap();
        self.base = result.draft.as_ref().unwrap().base.clone();
        result
    }
    fn pending(&mut self, clear: bool) {
        let result = stage_credential_with_store(
            &self.d,
            &self.h,
            StageCredentialV2 {
                expected: self.base.clone(),
                channel_id: self.id.clone(),
                action: if clear { "clear" } else { "replace" }.into(),
                value: (!clear).then(|| "new-pending-key".into()),
            },
            &self.vault,
        )
        .unwrap();
        self.base = result.draft.unwrap().base;
    }
    fn save_copy(&self, copy_id: &str) -> Result<FileSnapshotV2> {
        save_with_credentials(
            &self.d,
            &self.h,
            SaveRequestV2 {
                expected: self.base.clone(),
                edits: vec![],
                channel_intents: vec![],
                scope_channel_id: Some(copy_id.into()),
            },
            &self.vault,
        )
    }
}
#[test]
fn omitted_copy_key_is_opt_in_false_and_ui_never_supplies_key_material() {
    let f = Fixture::new();
    let value = json!({"expected":f.base,"channelId":f.id,"name":"copy"});
    let request: CopyChannelV2 = serde_json::from_value(value.clone()).unwrap();
    assert!(!request.reuse_source_key);
    let mut injected = value;
    injected["apiKey"] = json!("injected");
    assert!(serde_json::from_value::<CopyChannelV2>(injected).is_err());
}
#[test]
fn nonreuse_copy_strips_all_seed_credentials_and_preserves_source_and_unknown_public_bytes() {
    let mut f = Fixture::new();
    let original = std::fs::read(f.h.join("config.json")).unwrap();
    let copied = f.copy(false);
    assert_eq!(std::fs::read(f.h.join("config.json")).unwrap(), original);
    let draft = copied.draft.unwrap();
    let c = draft.policy.channels.iter().find(|c| c.id != f.id).unwrap();
    assert!(!c.connection.enabled);
    assert!(c.connection.credential_ref.is_none());
    assert_eq!(
        draft.credential_edits.get(&c.id),
        Some(&DraftCredentialEditV2::Clear)
    );
    let (raw, _) = read_raw(&f.h).unwrap();
    let seed = copied_seed(&f.d, &raw, c, &f.base.source_digest)
        .unwrap()
        .unwrap();
    for field in crate::modules::model_config::provider_fields::PRIVATE {
        assert!(seed.get(*field).is_none());
    }
    assert!(seed.get("apiKeySelectionRevision").is_none());
    assert_eq!(seed["unknown"], json!({"keep":true}));
    f.save_copy(&c.id).unwrap();
    let (saved, _) = read_raw(&f.h).unwrap();
    assert_eq!(saved["providers"]["source"], raw["providers"]["source"]);
    for field in crate::modules::model_config::provider_fields::PRIVATE {
        assert!(saved["providers"]["copy"].get(*field).is_none());
    }
    assert_eq!(saved["providers"]["copy"]["unknown"], json!({"keep":true}));
}
#[test]
fn pending_replace_reuse_uses_immutable_intent_without_carrying_old_pool_or_private_refs() {
    let mut f = Fixture::new();
    f.pending(false);
    let source_draft = StoreV2::new(&f.d).snapshot().unwrap().draft.unwrap();
    let copied = f.copy(true);
    let draft = copied.draft.unwrap();
    let c = draft.policy.channels.iter().find(|c| c.id != f.id).unwrap();
    assert_eq!(
        draft.credential_edits[&c.id],
        source_draft.credential_edits[&f.id]
    );
    assert_eq!(
        draft.policy.channels.iter().find(|c| c.id == f.id).unwrap(),
        source_draft
            .policy
            .channels
            .iter()
            .find(|c| c.id == f.id)
            .unwrap()
    );
    f.save_copy(&c.id).unwrap();
    let (saved, _) = read_raw(&f.h).unwrap();
    assert_eq!(saved["providers"]["copy"]["apiKey"], "new-pending-key");
    for key in [
        "apiKeyPool",
        "headers",
        "proxy",
        "mcpServers",
        "desktopExecutor",
    ] {
        assert!(saved["providers"]["copy"].get(key).is_none());
    }
    assert_eq!(
        saved["providers"]["source"]["apiKey"],
        "$COPY_TEST_SOURCE_KEY"
    );
    let retained = StoreV2::new(&f.d).snapshot().unwrap().draft.unwrap();
    assert_eq!(
        retained.credential_edits[&f.id],
        source_draft.credential_edits[&f.id]
    );
}
#[test]
fn pending_clear_and_nonreuse_replacement_never_borrow_actual_or_pending_credentials() {
    for (clear, reuse) in [(true, true), (false, false)] {
        let mut f = Fixture::new();
        f.pending(clear);
        let source = StoreV2::new(&f.d).snapshot().unwrap().draft.unwrap();
        let copied = f.copy(reuse);
        let c = copied
            .draft
            .as_ref()
            .unwrap()
            .policy
            .channels
            .iter()
            .find(|c| c.id != f.id)
            .unwrap();
        assert!(c.connection.credential_ref.is_none());
        if !reuse {
            let reference = c.connection.imported_config.as_ref().unwrap();
            let value = std::fs::read(copy_path(&f.d, &reference.id).unwrap()).unwrap();
            let record: CopySourceV2 =
                serde_json::from_value(parse_strict_json_v2(&value).unwrap()).unwrap();
            assert!(record.source_credential_edit.is_none());
        }
        f.save_copy(&c.id).unwrap();
        let (saved, _) = read_raw(&f.h).unwrap();
        for field in crate::modules::model_config::provider_fields::PRIVATE {
            assert!(saved["providers"]["copy"].get(*field).is_none());
        }
        assert_eq!(
            saved["providers"]["source"]["apiKey"],
            "$COPY_TEST_SOURCE_KEY"
        );
        assert_eq!(
            StoreV2::new(&f.d)
                .snapshot()
                .unwrap()
                .draft
                .unwrap()
                .credential_edits[&f.id],
            source.credential_edits[&f.id]
        );
    }
}
#[test]
fn failed_save_keeps_copy_reference_and_pending_replacement_for_exact_retry() {
    let mut f = Fixture::new();
    f.pending(false);
    let copied = f.copy(true);
    let c = copied
        .draft
        .as_ref()
        .unwrap()
        .policy
        .channels
        .iter()
        .find(|c| c.id != f.id)
        .unwrap();
    let copy_id = c.id.clone();
    let reference = c.connection.credential_ref.as_ref().unwrap().clone();
    let key = f.vault.0.lock().unwrap().remove(&reference).unwrap();
    let config = std::fs::read(f.h.join("config.json")).unwrap();
    let before = std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap();
    assert_eq!(
        f.save_copy(&copy_id).err().unwrap().code,
        "model_credential_missing"
    );
    assert_eq!(
        std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap(),
        before
    );
    assert_eq!(std::fs::read(f.h.join("config.json")).unwrap(), config);
    f.vault.put(&reference, &key).unwrap();
    f.save_copy(&copy_id).unwrap();
    assert_eq!(
        read_raw(&f.h).unwrap().0["providers"]["copy"]["apiKey"],
        "new-pending-key"
    );
}
#[test]
fn stale_copy_request_and_source_change_do_not_leave_copy_sources_or_draft_effects() {
    let f = Fixture::new();
    let before = std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap();
    let mut expected = f.base.clone();
    expected.draft_generation += 1;
    assert!(copy_channel(
        &f.d,
        &f.h,
        CopyChannelV2 {
            expected,
            channel_id: f.id.clone(),
            name: "copy".into(),
            reuse_source_key: true
        }
    )
    .is_err());
    assert_eq!(
        std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap(),
        before
    );
    assert!(!f.d.join("manager-state/model-v2-copy-sources").exists());
    let (mut raw, _) = read_raw(&f.h).unwrap();
    raw["providers"]["source"]["apiKey"] = json!("changed-source");
    std::fs::write(f.h.join("config.json"), canonical_bytes(&raw).unwrap()).unwrap();
    assert!(copy_channel(
        &f.d,
        &f.h,
        CopyChannelV2 {
            expected: f.base,
            channel_id: f.id,
            name: "copy".into(),
            reuse_source_key: false
        }
    )
    .is_err());
    assert!(!f.d.join("manager-state/model-v2-copy-sources").exists());
    assert_eq!(
        std::fs::read(f.d.join(store_v2::DRAFT_PATH)).unwrap(),
        before
    );
}
