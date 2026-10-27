//! All paths are temporary. The credential integration case runs in its own
//! process with an in-memory keyring, so it never accesses the user's keystore.
use super::*;
use std::collections::BTreeMap;

struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    home: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("manager");
        let home = temp.path().join("official");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        let config = json!({
            "providers": {
                "a": {"adapter":"openai-chat", "baseUrl":"https://a.example.test/v1",
                    "apiKey":"fixture-a", "apiKeyPool":[{"key":"fixture-a","id":"old","future":7}],
                    "models":["vendor/a:8b","Mixed_Case/V2"],
                    "selectedModels":["vendor/a:8b","Mixed_Case/V2"],
                    "futureField":{"private":"fixture-opaque"},
                    "modelReasoningEfforts":{"vendor/a:8b":["low","high"]}},
                "b": {"adapter":"openai-chat", "baseUrl":"https://b.example.test/v1",
                    "apiKey":"fixture-b", "models":["b1","b2"], "selectedModels":["b1","b2"],
                    "futureField":{"keep":true}}
            },
            "modelPickerOrder":["native-head","a/vendor-a:8b","b/b1","opaque-middle","a/Mixed_Case-V2","b/b2","native-tail"],
            "disabledModels":["native-unknown","other/keep"],
            "defaultProvider":"b", "futureRoot":{"keep":42}
        });
        atomic_write(
            &home.join("config.json"),
            &serde_json::to_vec(&config).unwrap(),
            0o600,
        )
        .unwrap();
        let fixture = Self {
            _temp: temp,
            root,
            home,
        };
        let mut current = fixture.snapshot().policy;
        for c in &mut current.channels {
            c.label = format!("saved-{}", c.connection.provider_id);
        }
        record_layout(&fixture.root, &current);
        fixture
    }
    fn snapshot(&self) -> Snapshot {
        snapshot(&self.root, &self.home).unwrap()
    }
    fn bytes(&self) -> Vec<u8> {
        std::fs::read(self.home.join("config.json")).unwrap()
    }
    fn raw(&self) -> Value {
        serde_json::from_slice(&self.bytes()).unwrap()
    }
    fn layout(&self) -> Policy {
        serde_json::from_slice(&std::fs::read(self.root.join(METADATA)).unwrap()).unwrap()
    }
    fn request(&self, policy: Policy, id: &str) -> SaveRequest {
        SaveRequest {
            expected_file: self.snapshot().policy_digest,
            policy,
            credentials: vec![],
            scope: Some(SaveScope {
                channel_id: id.into(),
            }),
        }
    }
    fn save(&self, request: SaveRequest) -> ModelResult<SaveResult> {
        save(&self.root, &self.home, request)
    }
    fn backups(&self) -> Vec<PathBuf> {
        std::fs::read_dir(self.root.join("backups/model-config"))
            .into_iter()
            .flatten()
            .map(|e| e.unwrap().path().join("config.json"))
            .collect()
    }
}
fn channel<'a>(policy: &'a Policy, provider: &str) -> &'a ChannelPolicy {
    policy
        .channels
        .iter()
        .find(|c| c.connection.provider_id == provider)
        .unwrap()
}
fn channel_mut<'a>(policy: &'a mut Policy, provider: &str) -> &'a mut ChannelPolicy {
    policy
        .channels
        .iter_mut()
        .find(|c| c.connection.provider_id == provider)
        .unwrap()
}
fn reference() -> TemplateRef {
    TemplateRef {
        template_id: uuid::Uuid::new_v4().to_string(),
        revision_id: uuid::Uuid::new_v4().to_string(),
    }
}
fn unmatched_reference(root: &Path) -> TemplateRef {
    let store = ModelStore::new(root);
    let current = store.load_library().unwrap();
    let mut content = TemplateContent::default();
    content.matching.include = vec![Pattern {
        kind: MatchKind::Exact,
        value: "fixture-no-match".into(),
    }];
    let saved = store
        .save(SaveTemplate {
            expected_library: current.library_digest,
            template_id: None,
            base_revision: None,
            label: "Fixture metadata adoption".into(),
            content,
        })
        .unwrap();
    TemplateRef {
        template_id: saved.template_id,
        revision_id: saved.revision_id,
    }
}
fn new_channel(source: &ChannelPolicy, provider: &str) -> ChannelPolicy {
    let mut c = source.clone();
    c.id = uuid::Uuid::new_v4().to_string();
    c.label = provider.into();
    c.connection.provider_id = provider.into();
    c
}
fn inventory(c: &mut ChannelPolicy, count: usize) {
    c.discovered = (0..count)
        .map(|i| DiscoveredModel {
            id: format!("exact/vendor-{i}:8b"),
            label: format!("exact/vendor-{i}:8b"),
            rules: ModelRules::default(),
        })
        .collect();
    c.model_order = c.discovered.iter().map(|m| m.id.clone()).collect();
    c.selection = ModelSelection::All;
    c.overrides.clear();
}

#[test]
fn scope_api_is_optional_camel_case_and_closed() {
    let payload = json!({"expectedFile":"sha256:fixture", "policy":Policy::default()});
    assert!(serde_json::from_value::<SaveRequest>(payload.clone())
        .unwrap()
        .scope
        .is_none());
    for scope in [
        json!({"channelId":"a","extra":true}),
        json!({"channel_id":"a"}),
        json!({}),
    ] {
        let mut invalid = payload.clone();
        invalid["scope"] = scope;
        assert!(serde_json::from_value::<SaveRequest>(invalid).is_err());
    }
    let mut valid = payload;
    valid["scope"] = json!({"channelId":"a"});
    assert_eq!(
        serde_json::from_value::<SaveRequest>(valid)
            .unwrap()
            .scope
            .unwrap()
            .channel_id,
        "a"
    );
}

#[test]
fn saves_only_a_with_exact_rules_key_projection_and_exact_backup() {
    let f = Fixture::new();
    let saved = f.snapshot().policy;
    let original = f.bytes();
    let old = f.raw();
    let mut draft = saved.clone();
    let a = channel_mut(&mut draft, "a");
    let id = a.id.clone();
    a.connection.enabled = false;
    a.overrides
        .entry("vendor/a:8b".into())
        .or_default()
        .context_window = Field::Custom(128000);
    a.discovered
        .iter_mut()
        .find(|m| m.id == "Mixed_Case/V2")
        .unwrap()
        .label = "Exact display".into();
    let b = channel_mut(&mut draft, "b");
    b.label = "unsaved B".into();
    b.connection.nodes.clear(); // Invalid unrelated drafts must not be compiled.
    b.template_refs = vec![reference()];
    draft.template_refs = vec![reference()];
    draft.default_channel_id = Some(id.clone());
    draft.pending_removals.push(SourceRemoval {
        read_id: "unsaved".into(),
        channel_id: channel(&saved, "b").id.clone(),
        provider_id: "b".into(),
        credential_ref: None,
    });
    draft.channels.reverse();
    let mut request = f.request(draft, &id);
    request.credentials.push(CredentialEdit {
        channel_id: id,
        action: "replace".into(),
        value: Some("fixture-a-new".into()),
    });
    let result = f.save(request).unwrap();
    assert!(result.changed);
    let raw = f.raw();
    assert!(raw["providers"]["b"] == old["providers"]["b"]);
    for key in [
        "defaultProvider",
        "disabledModels",
        "modelPickerOrder",
        "futureRoot",
    ] {
        assert!(raw[key] == old[key]);
    }
    let a = &raw["providers"]["a"];
    assert!(a["apiKey"] == "fixture-a-new" && a["apiKeyPool"][0]["key"] == "fixture-a-new");
    assert_eq!(a["apiKeyPool"][0]["future"], 7);
    assert_eq!(a["futureField"], old["providers"]["a"]["futureField"]);
    assert_eq!(a["modelContextWindows"]["vendor/a:8b"], 128000);
    assert_eq!(a["modelDisplayNames"]["Mixed_Case/V2"], "Exact display");
    assert_eq!(a["models"], old["providers"]["a"]["models"]);
    assert_eq!(result.snapshot.policy.template_refs, saved.template_refs);
    assert_eq!(channel(&result.snapshot.policy, "b"), channel(&saved, "b"));
    assert!(f.layout().pending_removals.is_empty());
    let backups = f.backups();
    assert_eq!(backups.len(), 1);
    assert!(std::fs::read(&backups[0]).unwrap() == original);
}

#[test]
fn partial_a_list_neither_deletes_b_nor_saves_unrelated_new_channels() {
    for add_unsaved in [false, true] {
        let f = Fixture::new();
        let saved = f.snapshot().policy;
        let old = f.raw();
        let mut a = channel(&saved, "a").clone();
        a.connection.enabled = false;
        let id = a.id.clone();
        let mut draft = Policy {
            channels: vec![a],
            ..Policy::default()
        };
        if add_unsaved {
            let mut c = new_channel(channel(&saved, "b"), "unsaved");
            c.connection.imported_config = Some(ImportedProviderRef {
                id: uuid::Uuid::new_v4().to_string(),
                digest: "invalid-unrelated".into(),
            });
            c.connection.credential_ref = Some("invalid-unrelated-key".into());
            draft.channels.push(c);
        }
        f.save(f.request(draft, &id)).unwrap();
        assert!(f.raw()["providers"]["b"] == old["providers"]["b"]);
        assert!(f.raw()["providers"].get("unsaved").is_none());
        assert_eq!(f.snapshot().policy.channels.len(), 2);
    }
}

#[test]
fn new_a_inserts_after_nearest_saved_predecessor_without_sorting_other_rows() {
    for anchor in [None, Some("a"), Some("b")] {
        let f = Fixture::new();
        let saved = f.snapshot().policy;
        let old = f.raw();
        let new = new_channel(channel(&saved, "a"), "new-a");
        let id = new.id.clone();
        let mut draft = saved.clone();
        draft.channels.reverse();
        let insertion = anchor
            .map(|p| {
                draft
                    .channels
                    .iter()
                    .position(|c| c.connection.provider_id == p)
                    .unwrap()
                    + 1
            })
            .unwrap_or(0);
        draft.channels.insert(
            insertion,
            new_channel(channel(&saved, "a"), "unsaved-neighbor"),
        );
        draft.channels.insert(insertion + 1, new);
        let result = f.save(f.request(draft, &id)).unwrap();
        let providers: Vec<_> = result
            .snapshot
            .policy
            .channels
            .iter()
            .map(|c| c.connection.provider_id.as_str())
            .collect();
        let expected = match anchor {
            None => vec!["new-a", "a", "b"],
            Some("a") => vec!["a", "new-a", "b"],
            _ => vec!["a", "b", "new-a"],
        };
        assert_eq!(providers, expected);
        let raw = f.raw();
        let other: Vec<_> = raw["modelPickerOrder"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| !v.as_str().unwrap().starts_with("new-a/"))
            .cloned()
            .collect();
        assert_eq!(other, *old["modelPickerOrder"].as_array().unwrap());
        assert!(raw["providers"].get("unsaved-neighbor").is_none());
        assert!(
            raw["providers"]["a"] == old["providers"]["a"]
                && raw["providers"]["b"] == old["providers"]["b"]
        );
    }
}

#[test]
fn a_model_order_change_preserves_interleaved_b_and_opaque_rows() {
    let f = Fixture::new();
    let mut draft = f.snapshot().policy;
    let a = channel_mut(&mut draft, "a");
    let id = a.id.clone();
    a.model_order.reverse();
    draft.channels.reverse();
    f.save(f.request(draft, &id)).unwrap();
    assert_eq!(
        f.raw()["modelPickerOrder"],
        json!([
            "native-head",
            "a/Mixed_Case-V2",
            "a/vendor-a:8b",
            "b/b1",
            "opaque-middle",
            "b/b2",
            "native-tail"
        ])
    );
}

#[test]
fn metadata_only_false_records_only_a_and_preserves_latest_locked_layout() {
    let f = Fixture::new();
    let adopted = unmatched_reference(&f.root);
    let mut saved = f.snapshot().policy;
    saved.template_refs = vec![adopted.clone()];
    channel_mut(&mut saved, "b").template_refs = vec![adopted.clone()];
    record_layout(&f.root, &saved);
    let mut draft = f.snapshot().policy;
    // Metadata may change without changing the config-file digest. The save
    // must use the latest locked layout rather than its stale request baseline.
    let mut latest = draft.clone();
    channel_mut(&mut latest, "b").label = "newly saved B".into();
    record_layout(&f.root, &latest);
    let prior_layout = f.layout();
    let original = f.bytes();
    let a = channel_mut(&mut draft, "a");
    let id = a.id.clone();
    a.label = "saved metadata A".into();
    a.template_refs = vec![adopted];
    a.connection.nodes[0].name = "A node label".into();
    let b = channel_mut(&mut draft, "b");
    b.label = "unsaved B".into();
    b.model_refs.insert("b1".into(), reference());
    b.connection.nodes.clear();
    draft.template_refs.clear();
    draft.default_channel_id = Some(id.clone());
    draft.channels.reverse();
    let result = f.save(f.request(draft, &id)).unwrap();
    assert!(!result.changed);
    assert!(f.bytes() == original);
    assert!(f.backups().is_empty());
    let layout = f.layout();
    assert_eq!(channel(&layout, "a").label, "saved metadata A");
    assert_eq!(channel(&layout, "a").template_refs.len(), 1);
    assert_eq!(
        channel(&layout, "a").connection.nodes[0].name,
        "A node label"
    );
    assert_eq!(channel(&layout, "b"), channel(&prior_layout, "b"));
    assert_eq!(layout.template_refs, prior_layout.template_refs);
    assert_eq!(
        result.snapshot.policy.default_channel_id,
        latest.default_channel_id
    );
    assert_eq!(channel(&f.snapshot().policy, "b").label, "newly saved B");
}

#[test]
fn new_empty_channel_retains_insertion_in_saved_layout_and_reopened_snapshot() {
    let f = Fixture::new();
    let mut draft = f.snapshot().policy;
    let mut new = new_channel(channel(&draft, "a"), "empty");
    new.discovered.clear();
    new.model_order.clear();
    new.overrides.clear();
    new.selection = ModelSelection::None;
    let id = new.id.clone();
    draft.channels.insert(1, new);
    f.save(f.request(draft, &id)).unwrap();
    let providers = |policy: Policy| {
        policy
            .channels
            .into_iter()
            .map(|c| c.connection.provider_id)
            .collect::<Vec<_>>()
    };
    assert_eq!(providers(f.layout()), ["a", "empty", "b"]);
    assert_eq!(providers(f.snapshot().policy), ["a", "empty", "b"]);
}

#[test]
fn scope_credentials_recovery_and_missing_or_duplicate_target_fail_without_writing() {
    for mode in ["credential", "recovery", "missing", "duplicate", "provider"] {
        let f = Fixture::new();
        let mut draft = f.snapshot().policy;
        let original = f.bytes();
        let layout = std::fs::read(f.root.join(METADATA)).unwrap();
        let id = channel(&draft, "a").id.clone();
        channel_mut(&mut draft, "a").connection.enabled = false;
        let mut credentials = vec![];
        let expected = match mode {
            "credential" => {
                credentials.push(CredentialEdit {
                    channel_id: channel(&draft, "b").id.clone(),
                    action: "clear".into(),
                    value: None,
                });
                "credential_scope"
            }
            "recovery" => {
                draft.recovery_id = Some(uuid::Uuid::new_v4().to_string());
                "save_scope"
            }
            "missing" => {
                draft.channels.retain(|c| c.id != id);
                "save_scope"
            }
            "duplicate" => {
                draft.channels.push(channel(&draft, "a").clone());
                "save_scope"
            }
            _ => {
                channel_mut(&mut draft, "a").connection.provider_id = "b".into();
                "provider_identity"
            }
        };
        let mut request = f.request(draft, &id);
        request.credentials = credentials;
        assert_eq!(f.save(request).err().unwrap().code, expected);
        assert!(f.bytes() == original);
        assert!(std::fs::read(f.root.join(METADATA)).unwrap() == layout);
        assert!(f.backups().is_empty());
    }
}

#[test]
fn budget_compiles_only_full_merged_candidate_at_100_and_101() {
    for (count, allowed) in [(98, true), (99, false)] {
        let f = Fixture::new();
        let mut draft = f.snapshot().policy;
        let original = f.bytes();
        let a = channel_mut(&mut draft, "a");
        let id = a.id.clone();
        inventory(a, count);
        // If the request is compiled, this unrelated invalid/oversized B fails.
        let b = channel_mut(&mut draft, "b");
        inventory(b, 150);
        b.template_refs = vec![reference()];
        draft.template_refs = vec![reference()];
        let result = f.save(f.request(draft, &id));
        if allowed {
            let saved = result.unwrap();
            assert!(saved.changed);
            assert_eq!(channel(&saved.snapshot.policy, "a").discovered.len(), 98);
            assert_eq!(channel(&saved.snapshot.policy, "b").discovered.len(), 2);
        } else {
            assert_eq!(result.err().unwrap().code, "catalog_budget");
            assert!(f.bytes() == original);
            assert!(f.backups().is_empty());
        }
    }
}

#[test]
fn stale_file_rejects_before_credentials_layout_or_backup() {
    let f = Fixture::new();
    let draft = f.snapshot().policy;
    let id = channel(&draft, "a").id.clone();
    let request = f.request(draft, &id);
    let layout = std::fs::read(f.root.join(METADATA)).unwrap();
    let mut external = f.raw();
    external["external"] = json!(true);
    let bytes = serde_json::to_vec(&external).unwrap();
    atomic_write(&f.home.join("config.json"), &bytes, 0o600).unwrap();
    assert_eq!(f.save(request).err().unwrap().code, "file_changed");
    assert!(f.bytes() == bytes);
    assert!(std::fs::read(f.root.join(METADATA)).unwrap() == layout);
    assert!(f.backups().is_empty());
}

fn named_draft(f: &Fixture, raw_change: bool) -> (Policy, String) {
    let mut draft = f.snapshot().policy;
    let a = channel_mut(&mut draft, "a");
    a.label = "name awaiting durable layout".into();
    if raw_change {
        a.connection.enabled = !a.connection.enabled;
    }
    let id = a.id.clone();
    (draft, id)
}

#[test]
fn actual_layout_lock_timeout_never_acknowledges_metadata_or_raw_and_is_retryable() {
    for raw_change in [false, true] {
        let f = Fixture::new();
        let (draft, id) = named_draft(&f, raw_change);
        let request = f.request(draft.clone(), &id);
        let before = f.bytes();
        let layout_before = std::fs::read(f.root.join(METADATA)).unwrap();
        let guard = TargetFileLock::lock(&f.root.join(METADATA)).unwrap();
        assert_eq!(f.save(request).err().unwrap().code, "layout_lock");
        assert!(f.bytes() == before);
        assert!(std::fs::read(f.root.join(METADATA)).unwrap() == layout_before);
        assert!(f.backups().is_empty());
        drop(guard);
        let result = f.save(f.request(draft, &id)).unwrap();
        assert_eq!(result.changed, raw_change);
        assert_eq!(
            channel(&result.snapshot.policy, "a").label,
            "name awaiting durable layout"
        );
        assert_eq!(result.snapshot.policy, f.snapshot().policy);
    }
}

#[cfg(unix)]
struct ReadOnlyDirectory {
    path: PathBuf,
    permissions: std::fs::Permissions,
}
#[cfg(unix)]
impl ReadOnlyDirectory {
    fn new(path: &Path) -> Self {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::metadata(path).unwrap().permissions();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o500)).unwrap();
        let guard = Self {
            path: path.into(),
            permissions,
        };
        // Prove this fixture really denies atomic writes; never claim a fault
        // test passed merely because a privileged process bypassed mode bits.
        assert!(atomic_write(&path.join("fixture-write-probe"), b"probe", 0o600).is_err());
        guard
    }
}
#[cfg(unix)]
impl Drop for ReadOnlyDirectory {
    fn drop(&mut self) {
        std::fs::set_permissions(&self.path, self.permissions.clone()).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn actual_layout_write_failure_rolls_back_raw_and_metadata_only_never_succeeds() {
    for raw_change in [false, true] {
        let f = Fixture::new();
        let (draft, id) = named_draft(&f, raw_change);
        let request = f.request(draft.clone(), &id);
        let before = f.bytes();
        let layout_before = std::fs::read(f.root.join(METADATA)).unwrap();
        let denied = ReadOnlyDirectory::new(&f.root.join("manager-state"));
        assert_eq!(f.save(request).err().unwrap().code, "layout_write");
        assert!(f.bytes() == before);
        assert!(std::fs::read(f.root.join(METADATA)).unwrap() == layout_before);
        let backups = f.backups();
        assert_eq!(backups.len(), usize::from(raw_change));
        if raw_change {
            assert!(std::fs::read(&backups[0]).unwrap() == before);
        }
        drop(denied);
        let result = f.save(f.request(draft, &id)).unwrap();
        assert_eq!(result.changed, raw_change);
        assert_eq!(result.snapshot.policy, f.snapshot().policy);
        assert_eq!(
            channel(&result.snapshot.policy, "a").label,
            "name awaiting durable layout"
        );
    }
}

#[cfg(unix)]
#[test]
fn actual_raw_write_failure_preserves_layout_and_allows_same_baseline_retry() {
    let f = Fixture::new();
    let (draft, id) = named_draft(&f, true);
    let request = f.request(draft.clone(), &id);
    let before = f.bytes();
    let layout_before = std::fs::read(f.root.join(METADATA)).unwrap();
    let denied = ReadOnlyDirectory::new(&f.home);
    assert_eq!(f.save(request).err().unwrap().code, "storage");
    assert!(f.bytes() == before);
    assert!(std::fs::read(f.root.join(METADATA)).unwrap() == layout_before);
    drop(denied);
    assert!(f.save(f.request(draft, &id)).unwrap().changed);
}

#[cfg(windows)]
#[test]
fn windows_layout_replace_denied_preserves_metadata_and_rolls_back_raw_then_retries() {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::{FILE_SHARE_READ, FILE_SHARE_WRITE};
    for raw_change in [false, true] {
        let f = Fixture::new();
        let (draft, id) = named_draft(&f, raw_change);
        let request = f.request(draft.clone(), &id);
        let before = f.bytes();
        let layout_path = f.root.join(METADATA);
        let layout_before = std::fs::read(&layout_path).unwrap();
        // Retain read/write sharing so baseline reads and the layout lock work.
        // Omit FILE_SHARE_DELETE to make the real atomic rename fail on Windows.
        let deny_replace = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .open(&layout_path)
            .unwrap();
        assert!(
            atomic_write(&layout_path, b"replace must fail", 0o600).is_err(),
            "fixture must actually prevent atomic layout replacement"
        );
        assert!(std::fs::read(&layout_path).unwrap() == layout_before);
        assert_eq!(f.save(request).err().unwrap().code, "layout_write");
        assert!(f.bytes() == before);
        assert!(std::fs::read(&layout_path).unwrap() == layout_before);
        let backups = f.backups();
        assert_eq!(backups.len(), usize::from(raw_change));
        if raw_change {
            assert!(std::fs::read(&backups[0]).unwrap() == before);
        }
        drop(deny_replace);
        let result = f.save(f.request(draft, &id)).unwrap();
        assert_eq!(result.changed, raw_change);
        assert_eq!(result.snapshot.policy, f.snapshot().policy);
        assert_eq!(
            channel(&result.snapshot.policy, "a").label,
            "name awaiting durable layout"
        );
    }
}

#[cfg(unix)]
#[test]
fn layout_write_failure_restores_absent_raw_file_as_absent() {
    let f = Fixture::new();
    let mut incoming = channel(&f.snapshot().policy, "a").clone();
    incoming.label = "new unsaved name".into();
    let id = incoming.id.clone();
    std::fs::remove_file(f.home.join("config.json")).unwrap();
    let draft = Policy {
        channels: vec![incoming],
        ..Policy::default()
    };
    let request = f.request(draft.clone(), &id);
    let layout_before = std::fs::read(f.root.join(METADATA)).unwrap();
    let denied = ReadOnlyDirectory::new(&f.root.join("manager-state"));
    assert_eq!(f.save(request).err().unwrap().code, "layout_write");
    assert!(!f.home.join("config.json").exists());
    assert!(std::fs::read(f.root.join(METADATA)).unwrap() == layout_before);
    drop(denied);
    assert!(f.save(f.request(draft, &id)).unwrap().changed);
}

thread_local! {
    static AFTER_LAYOUT_WRITE: std::cell::RefCell<Option<Box<dyn FnOnce()>>> = const { std::cell::RefCell::new(None) };
}
pub(super) fn after_layout_write() {
    if let Some(callback) = AFTER_LAYOUT_WRITE.with(|slot| slot.borrow_mut().take()) {
        callback();
    }
}
struct AfterLayoutWrite;
impl AfterLayoutWrite {
    fn install(callback: impl FnOnce() + 'static) -> Self {
        AFTER_LAYOUT_WRITE.with(|slot| *slot.borrow_mut() = Some(Box::new(callback)));
        Self
    }
}
impl Drop for AfterLayoutWrite {
    fn drop(&mut self) {
        AFTER_LAYOUT_WRITE.with(|slot| slot.borrow_mut().take());
    }
}

#[test]
fn snapshot_read_failure_after_both_commits_rolls_back_exact_bytes_and_allows_retry() {
    for raw_change in [false, true] {
        let f = Fixture::new();
        unmatched_reference(&f.root); // Materialize an isolated library file.
        let (draft, id) = named_draft(&f, raw_change);
        let request = f.request(draft.clone(), &id);
        let before = f.bytes();
        let layout_before = std::fs::read(f.root.join(METADATA)).unwrap();
        let library_path = f.root.join(LIBRARY_PATH);
        let library_before = std::fs::read(&library_path).unwrap();
        let fault_path = library_path.clone();
        // This is real invalid on-disk input, not a mocked success/error result.
        let hook = AfterLayoutWrite::install(move || {
            std::fs::write(fault_path, b"broken snapshot library").unwrap()
        });
        assert_eq!(f.save(request).err().unwrap().code, "invalid_document");
        assert!(f.bytes() == before);
        assert!(std::fs::read(f.root.join(METADATA)).unwrap() == layout_before);
        std::fs::write(library_path, library_before).unwrap();
        drop(hook);
        let result = f.save(f.request(draft, &id)).unwrap();
        assert_eq!(result.changed, raw_change);
        assert_eq!(result.snapshot.policy, f.snapshot().policy);
    }
}

#[test]
fn external_raw_after_commit_is_not_overwritten_by_compensation() {
    let f = Fixture::new();
    let (draft, id) = named_draft(&f, true);
    let request = f.request(draft, &id);
    let layout_before = std::fs::read(f.root.join(METADATA)).unwrap();
    let mut external = f.raw();
    external["external-writer"] = json!(true);
    let external_bytes = serde_json::to_vec(&external).unwrap();
    let expected_external = external_bytes.clone();
    let path = f.home.join("config.json");
    let _hook =
        AfterLayoutWrite::install(move || atomic_write(&path, &external_bytes, 0o600).unwrap());
    assert_eq!(f.save(request).err().unwrap().code, "save_rollback_failed");
    assert!(f.bytes() == expected_external);
    assert!(std::fs::read(f.root.join(METADATA)).unwrap() == layout_before);
    assert_eq!(f.backups().len(), 1);
}

#[test]
fn success_snapshot_reads_persisted_layout_instead_of_using_unsaved_request_fields() {
    let f = Fixture::new();
    let mut draft = f.snapshot().policy;
    channel_mut(&mut draft, "a").label.clear();
    // Legacy global saves accept this shape, but the layout reader drops the
    // empty display label. A success response must agree with reopening it.
    let result = f
        .save(SaveRequest {
            expected_file: f.snapshot().policy_digest,
            policy: draft,
            credentials: vec![],
            scope: None,
        })
        .unwrap();
    assert!(!result.changed);
    assert_eq!(result.snapshot.policy, f.snapshot().policy);
    assert_eq!(channel(&result.snapshot.policy, "a").label, "a");
}

#[test]
fn imported_public_fields_preserve_local_key_and_other_channels() {
    let f = Fixture::new();
    let old = f.raw();
    let mut draft = f.snapshot().policy;
    let a = channel_mut(&mut draft, "a");
    let id = a.id.clone();
    let public = json!({"adapter":"openai-chat","baseUrl":"https://a.example.test/v1",
        "models":["vendor/a:8b","Mixed_Case/V2"],"selectedModels":["vendor/a:8b","Mixed_Case/V2"],
        "note":"imported exact public", "retryOn429":false});
    a.connection.imported_config =
        Some(super::super::provider_import::stage(&f.root, a, &public, None).unwrap());
    channel_mut(&mut draft, "b").connection.imported_config = Some(ImportedProviderRef {
        id: uuid::Uuid::new_v4().to_string(),
        digest: "unread".into(),
    });
    f.save(f.request(draft, &id)).unwrap();
    let raw = f.raw();
    let a = &raw["providers"]["a"];
    assert_eq!(a["note"], "imported exact public");
    assert_eq!(a["retryOn429"], false);
    assert!(
        a["apiKey"] == old["providers"]["a"]["apiKey"]
            && a["apiKeyPool"] == old["providers"]["a"]["apiKeyPool"]
    );
    assert_eq!(a["futureField"], old["providers"]["a"]["futureField"]);
    assert!(raw["providers"]["b"] == old["providers"]["b"]);
    assert!(channel(&f.snapshot().policy, "a")
        .connection
        .imported_config
        .is_none());
}

#[test]
fn absent_scope_keeps_global_save_behavior() {
    let f = Fixture::new();
    let mut draft = f.snapshot().policy;
    for c in &mut draft.channels {
        c.connection.enabled = false;
    }
    let result = f
        .save(SaveRequest {
            expected_file: f.snapshot().policy_digest,
            policy: draft,
            credentials: vec![],
            scope: None,
        })
        .unwrap();
    assert!(result.changed);
    assert!(result
        .snapshot
        .policy
        .channels
        .iter()
        .all(|c| !c.connection.enabled));
}

// Process isolation prevents the keyring builder from changing the behavior of
// other parallel tests. No environment mutation or real OS keychain is needed.
#[test]
fn imported_credentials_and_legacy_single_key_are_scoped_without_os_keychain() {
    const CHILD: &str = "OCX_SCOPED_KEYRING_FIXTURE";
    if std::env::var_os(CHILD).is_none() {
        let status=std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact","modules::model_config::config_file::scoped_tests::imported_credentials_and_legacy_single_key_are_scoped_without_os_keychain","--nocapture"])
            .env(CHILD,"1").status().unwrap();
        assert!(status.success());
        return;
    }
    keyring::set_default_credential_builder(Box::new(memory_keyring::Builder::default()));
    for legacy in [false, true] {
        let f = Fixture::new();
        let old = f.raw();
        let mut draft = f.snapshot().policy;
        let mut incoming = new_channel(channel(&draft, "a"), "imported");
        incoming.connection.enabled = false;
        let id = incoming.id.clone();
        if legacy {
            let reference = uuid::Uuid::new_v4().to_string();
            super::super::discovery::import_credential(&reference, "fixture-legacy").unwrap();
            incoming.connection.credential_ref = Some(reference);
        } else {
            let public = json!({"adapter":"openai-chat","authMode":"key","baseUrl":"https://a.example.test/v1",
                "models":["vendor/a:8b","Mixed_Case/V2"],"selectedModels":["vendor/a:8b","Mixed_Case/V2"],"note":"complete incoming"});
            incoming.connection.auth_mode = Some("key".into());
            let slot = sha256(b"fixture-imported").trim_start_matches("sha256:")[..8].to_string();
            let auth=serde_json::from_value(json!({"activeId":slot,"fields":{"apiKeyPool":[{"id":slot,"key":"fixture-imported","label":"Imported"}],"headers":{"X-Fixture":"fixture-header"}}})).unwrap();
            incoming.connection.imported_config = Some(
                super::super::provider_import::stage(&f.root, &incoming, &public, Some(&auth))
                    .unwrap(),
            );
        }
        // A second unselected channel carries a key ref that must not be read.
        let mut ignored = new_channel(channel(&draft, "b"), "ignored");
        ignored.connection.credential_ref = Some("invalid-not-read".into());
        draft.channels.extend([incoming, ignored]);
        let result = f.save(f.request(draft, &id)).unwrap();
        assert!(result.changed);
        let raw = f.raw();
        let imported = &raw["providers"]["imported"];
        assert!(
            imported["apiKey"]
                == if legacy {
                    "fixture-legacy"
                } else {
                    "fixture-imported"
                }
        );
        if !legacy {
            assert!(imported["apiKeyPool"][0]["key"] == "fixture-imported");
            assert_eq!(imported["note"], "complete incoming");
        }
        assert!(raw["providers"].get("ignored").is_none());
        assert!(
            raw["providers"]["a"] == old["providers"]["a"]
                && raw["providers"]["b"] == old["providers"]["b"]
        );
        assert!(result.snapshot.policy.channels.iter().all(|c| c
            .connection
            .credential_ref
            .is_none()
            && c.connection.imported_config.is_none()));
    }
}

mod memory_keyring {
    use super::BTreeMap;
    use keyring::credential::{Credential, CredentialApi, CredentialBuilderApi};
    use std::{
        any::Any,
        sync::{Arc, Mutex},
    };
    type Secrets = Arc<Mutex<BTreeMap<(String, String), Vec<u8>>>>;
    #[derive(Default)]
    pub struct Builder(Secrets);
    struct Entry {
        values: Secrets,
        identity: (String, String),
    }
    impl CredentialBuilderApi for Builder {
        fn build(
            &self,
            _: Option<&str>,
            service: &str,
            user: &str,
        ) -> keyring::Result<Box<Credential>> {
            Ok(Box::new(Entry {
                values: self.0.clone(),
                identity: (service.into(), user.into()),
            }))
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    impl CredentialApi for Entry {
        fn set_secret(&self, value: &[u8]) -> keyring::Result<()> {
            self.values
                .lock()
                .unwrap()
                .insert(self.identity.clone(), value.to_vec());
            Ok(())
        }
        fn get_secret(&self) -> keyring::Result<Vec<u8>> {
            self.values
                .lock()
                .unwrap()
                .get(&self.identity)
                .cloned()
                .ok_or(keyring::Error::NoEntry)
        }
        fn delete_credential(&self) -> keyring::Result<()> {
            self.values
                .lock()
                .unwrap()
                .remove(&self.identity)
                .map(|_| ())
                .ok_or(keyring::Error::NoEntry)
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
}
