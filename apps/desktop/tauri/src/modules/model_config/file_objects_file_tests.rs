use super::super::{config_file, portable_channels};
use super::*;
use serde_json::json;
use std::cell::RefCell;

struct Device {
    _temp: tempfile::TempDir,
    root: PathBuf,
    home: PathBuf,
}
impl Device {
    fn new(provider: &str) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("manager");
        let home = temp.path().join("official");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        let device = Self {
            _temp: temp,
            root,
            home,
        };
        device.write(json!({"providers": {provider: {
            "adapter":"openai-chat", "baseUrl":format!("https://{provider}.example.test/v1"),
            "authMode":"key", "models":["vendor/custom/a","vendor/custom-b"],
            "selectedModels":["vendor/custom-b"], "note":"saved file",
            "modelAliases":{"vendor/custom/a":"native-route"},
            "apiKey":"fixture-current-file-SECRET", "headers":{"X-Auth":"fixture-header-SECRET"}
        }}, "unrelated":{"keep":true}}));
        device
    }
    fn write(&self, value: Value) {
        atomic_write(
            &self.home.join("config.json"),
            &serde_json::to_vec(&value).unwrap(),
            0o600,
        )
        .unwrap();
    }
    fn raw(&self) -> Value {
        serde_json::from_slice(&std::fs::read(self.home.join("config.json")).unwrap()).unwrap()
    }
    fn snapshot(&self) -> Snapshot {
        config_file::snapshot(&self.root, &self.home).unwrap()
    }
    fn retired_history(&self) {
        std::fs::create_dir_all(self.root.join("manager-state")).unwrap();
        std::fs::write(self.root.join(POLICY_PATH), b"broken retired policy").unwrap();
        std::fs::write(
            self.root.join(DRAFT_PATH),
            b"broken retired draft to preserve",
        )
        .unwrap();
    }
    fn history_unchanged(&self) {
        assert_eq!(
            std::fs::read(self.root.join(POLICY_PATH)).unwrap(),
            b"broken retired policy"
        );
        assert_eq!(
            std::fs::read(self.root.join(DRAFT_PATH)).unwrap(),
            b"broken retired draft to preserve"
        );
    }
    fn scope(&self) -> transfer::ExportScope {
        transfer::ExportScope {
            channel_families: Default::default(),
            channel_models: Default::default(),
            include_policy: true,
            channel_ids: self
                .snapshot()
                .policy
                .channels
                .iter()
                .map(|c| c.id.clone())
                .collect(),
            template_refs: vec![],
            include_global: true,
        }
    }
    fn export(&self, scope: transfer::ExportScope, keys: bool) -> transfer::FileExportPlan {
        transfer::prepare_file_scope(
            &self.root,
            scope,
            None,
            keys,
            &ManagerScope {
                upstream_root: Some(self.home.clone()),
                ..Default::default()
            },
        )
        .unwrap()
    }
    fn custom_template(&self) -> TemplateRef {
        let store = ModelStore::new(&self.root);
        let local = store.load_library().unwrap();
        let content = local.library.templates[0].revisions[0].content.clone();
        let saved = store
            .save(SaveTemplate {
                expected_library: local.library_digest,
                template_id: None,
                base_revision: None,
                label: "incoming custom family".into(),
                content,
            })
            .unwrap();
        TemplateRef {
            template_id: saved.template_id,
            revision_id: saved.revision_id,
        }
    }
}
#[derive(Default)]
struct Keys(RefCell<BTreeMap<String, String>>);
impl Credentials for Keys {
    fn read(&self, reference: &str) -> ModelResult<Zeroizing<String>> {
        self.0
            .borrow()
            .get(reference)
            .cloned()
            .map(Zeroizing::new)
            .ok_or_else(|| invalid("credential_store", "fixture key unavailable"))
    }
    fn install(&self, reference: &str, value: &str) -> ModelResult<()> {
        self.0.borrow_mut().insert(reference.into(), value.into());
        Ok(())
    }
    fn remove(&self, reference: &str) -> ModelResult<()> {
        self.0.borrow_mut().remove(reference);
        Ok(())
    }
}

#[test]
fn file_export_ignores_stale_policy_and_cli_and_carries_current_raw_authorization() {
    let source = Device::new("source");
    let mut raw = source.raw();
    let key_hash = sha256(b"fixture-current-file-SECRET");
    raw["providers"]["source"]["apiKeyPool"] = json!([{
        "id":&key_hash.trim_start_matches("sha256:")[..8],
        "key":"fixture-current-file-SECRET", "label":"current pool", "addedAt":123
    }]);
    raw["providers"]["source"]["contextWindow"] = json!(64000);
    raw["providers"]["unselected"] = json!({
        "adapter":"openai-chat", "baseUrl":"https://unselected.example.test/v1",
        "models":["unselected-model"], "apiKey":"fixture-UNSELECTED-secret"
    });
    source.write(raw);
    let mut saved = source.snapshot();
    let id = saved.policy.channels[0].id.clone();
    let reference = source.custom_template();
    saved.policy.channels[0].template_refs = vec![reference.clone()];
    saved.policy.channels[0]
        .model_refs
        .insert("vendor/custom/a".into(), reference.clone());
    config_file::record_layout(&source.root, &saved.policy);
    // A valid but stale policy used to trigger the provider_source_drift gate.
    saved.policy.channels[0].connection.nodes[0].url = "https://retired.example.test".into();
    saved.policy.channels[0].connection.credential_ref = Some(uuid::Uuid::new_v4().to_string());
    atomic_write(
        &source.root.join(POLICY_PATH),
        &serde_json::to_vec(&saved.policy).unwrap(),
        0o600,
    )
    .unwrap();
    std::fs::write(source.root.join(DRAFT_PATH), b"unreadable retired draft").unwrap();
    let mut scope = source.scope();
    scope.channel_ids = vec![id.clone()];
    let plan = transfer::prepare_file_scope(
        &source.root,
        scope.clone(),
        None,
        true,
        &ManagerScope {
            upstream_root: Some(source.home.clone()),
            upstream_snapshot: Some(super::super::upstream::UpstreamSnapshot {
                version: "unusable-cli".into(),
                providers: Value::Null,
                models: Value::Null,
                order: Value::Null,
                catalog: Value::Null,
            }),
            ..Default::default()
        },
    )
    .unwrap();
    let bytes = transfer::file_bytes(&source.root, &plan.id).unwrap();
    let (objects, protected) = remote_objects(&bytes, None).unwrap();
    assert!(!protected);
    assert!(objects.channel_keys.0.is_empty());
    assert_eq!(
        objects
            .channel_authorizations
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        vec![id.clone()]
    );
    let authorization = serde_json::to_value(&objects.channel_authorizations[&id]).unwrap();
    assert!(authorization
        .to_string()
        .contains("fixture-current-file-SECRET"));
    assert_eq!(
        authorization["fields"]["headers"]["X-Auth"],
        "fixture-header-SECRET"
    );
    assert_eq!(
        authorization["fields"]["apiKeyPool"][0]["label"],
        "current pool"
    );
    assert!(!String::from_utf8_lossy(&bytes).contains("fixture-UNSELECTED-secret"));
    let channel = &plan.package.payload.policy.as_ref().unwrap().channels[0];
    assert_eq!(
        channel.selection,
        source.snapshot().policy.channels[0].selection
    );
    assert_eq!(
        channel.model_order,
        source.snapshot().policy.channels[0].model_order
    );
    assert_eq!(channel.template_refs, vec![reference.clone()]);
    assert_eq!(
        channel
            .connection
            .as_ref()
            .unwrap()
            .provider
            .as_ref()
            .unwrap()["note"],
        "saved file"
    );
    assert!(!serde_json::to_string(&plan.package)
        .unwrap()
        .contains("SECRET"));
    // A file-only edit is the next export's authority, with no manager save.
    let mut raw = source.raw();
    raw["providers"]["source"]["note"] = json!("file-only edit");
    raw["providers"]["source"]["selectedModels"] = json!(["vendor/custom/a"]);
    source.write(raw);
    assert_eq!(
        seal_scope(
            &source.root,
            &uuid::Uuid::new_v4().to_string(),
            &plan.package,
            false,
            None,
            &ManagerScope {
                upstream_root: Some(source.home.clone()),
                ..Default::default()
            },
            &Keys::default()
        )
        .unwrap_err()
        .code,
        "provider_source_drift"
    );
    let next = source.export(scope.clone(), false);
    let next_channel = &next.package.payload.policy.as_ref().unwrap().channels[0];
    assert_eq!(
        next_channel.selection,
        ModelSelection::Whitelist(vec!["vendor/custom/a".into()])
    );
    assert_eq!(
        portable_channels::provider(&next.package, &id).unwrap()["note"],
        "file-only edit"
    );
    assert_eq!(next.credential_count, 0);
    let mut subset = scope.clone();
    subset
        .channel_models
        .insert(id.clone(), vec!["vendor/custom/a".into()]);
    let subset = source.export(subset, true);
    assert_eq!(
        portable_channels::provider(&subset.package, &id).unwrap()["models"],
        json!(["vendor/custom/a"])
    );
    let mut family = scope;
    family.channel_families.insert(
        id,
        transfer::FamilyScope {
            template_ids: vec![reference.template_id],
            include_channel_defaults: false,
        },
    );
    for shared in [false, true] {
        family
            .channel_families
            .values_mut()
            .next()
            .unwrap()
            .include_channel_defaults = shared;
        let family = source.export(family.clone(), true);
        assert_eq!(family.credential_count, 0);
        let channel = &family.package.payload.policy.as_ref().unwrap().channels[0];
        assert!(channel.connection.is_none());
        assert_eq!(
            channel.defaults.context_window,
            if shared {
                Field::Custom(64000)
            } else {
                Field::Inherit
            }
        );
    }
}

#[test]
fn file_compare_accept_and_replay_keep_saved_file_and_retired_draft_separate_from_candidate() {
    let source = Device::new("source");
    let target = Device::new("local");
    let mut layout = source.snapshot().policy;
    layout.channels[0].connection.nodes.insert(
        0,
        ServiceNode {
            id: uuid::Uuid::new_v4().to_string(),
            name: "unused saved node".into(),
            url: "https://unused.example.test/v1".into(),
        },
    );
    config_file::record_layout(&source.root, &layout);
    source.retired_history();
    target.retired_history();
    let reference = source.custom_template();
    let mut scope = source.scope();
    scope.template_refs = vec![reference.clone()];
    let exported = source.export(scope, false);
    let bytes = transfer::file_bytes(&source.root, &exported.id).unwrap();
    let before = target.snapshot();
    let original_bytes = std::fs::read(target.home.join("config.json")).unwrap();
    let incoming = receive_context_file(&target.root, &bytes, None, None, &target.home).unwrap();
    assert_eq!(incoming.file_home, Some(target.home.clone()));
    assert_eq!(incoming.expected_policy, before.policy_digest);
    assert_eq!(target.snapshot(), before);
    let compared = recompare(&target.root, &incoming.id, None).unwrap();
    assert_eq!(compared.file_home, incoming.file_home);
    let selected = transfer::select_received(
        &target.root,
        &compared.id,
        transfer::ReceiveScope {
            channel_ids: exported
                .package
                .payload
                .policy
                .as_ref()
                .unwrap()
                .channels
                .iter()
                .map(|c| c.id.clone())
                .collect(),
            template_refs: vec![reference.clone()],
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(selected.file_home, incoming.file_home);
    let adopted = accept(&target.root, &selected.id, false, None).unwrap();
    assert_eq!(adopted.policy, before.policy);
    assert_eq!(adopted.policy_digest, before.policy_digest);
    let draft = adopted.workspace.as_ref().unwrap();
    assert_eq!(draft.base_policy, before.policy_digest);
    assert_eq!(draft.policy.channels.len(), 2);
    let channel = draft
        .policy
        .channels
        .iter()
        .find(|c| c.id == exported.package.payload.policy.as_ref().unwrap().channels[0].id)
        .unwrap();
    assert!(!channel.connection.enabled);
    assert_eq!(
        channel
            .connection
            .nodes
            .iter()
            .find(|node| node.id == channel.connection.active_node_id)
            .unwrap()
            .url,
        "https://source.example.test/v1"
    );
    let staged = super::super::provider_import::load(
        &target.root,
        channel,
        channel.connection.imported_config.as_ref().unwrap(),
    )
    .unwrap();
    assert_eq!(staged.public["note"], "saved file");
    assert!(staged.authorization.is_none());
    assert!(adopted
        .library
        .templates
        .iter()
        .any(|t| t.id == reference.template_id));
    assert_eq!(
        std::fs::read(target.home.join("config.json")).unwrap(),
        original_bytes
    );
    assert!(target.snapshot().workspace.is_none());
    target.history_unchanged();
    source.history_unchanged();
    let mut later = target.raw();
    later["unrelated"]["later"] = json!(42);
    target.write(later);
    let current = target.snapshot();
    let replay = accept(&target.root, &selected.id, false, None).unwrap();
    assert_eq!(replay.policy, current.policy);
    assert_eq!(replay.policy_digest, current.policy_digest);
    assert_eq!(replay.workspace, adopted.workspace);
    assert_eq!(target.snapshot(), current);
    target.history_unchanged();
}

#[test]
fn template_only_file_acceptance_and_legacy_recompare_never_create_a_channel_workspace() {
    let source = Device::new("source");
    let target = Device::new("local");
    let reference = source.custom_template();
    let plan = source.export(
        transfer::ExportScope {
            include_policy: false,
            channel_ids: vec![],
            template_refs: vec![reference.clone()],
            include_global: false,
            channel_families: Default::default(),
            channel_models: Default::default(),
        },
        false,
    );
    let bytes = transfer::file_bytes(&source.root, &plan.id).unwrap();
    let legacy = receive(&target.root, &bytes, None).unwrap();
    assert!(legacy.file_home.is_none());
    target.retired_history();
    let incoming = recompare_file(&target.root, &legacy.id, None, &target.home).unwrap();
    let before = target.snapshot();
    let result = accept(&target.root, &incoming.id, false, None).unwrap();
    assert_eq!(result.policy, before.policy);
    assert_eq!(result.policy_digest, before.policy_digest);
    assert!(result.workspace.is_none());
    assert!(result
        .library
        .templates
        .iter()
        .any(|t| t.id == reference.template_id));
    target.history_unchanged();
}

#[test]
fn file_acceptance_checks_file_digest_before_installing_plain_keys_and_mapping_keeps_local_connection(
) {
    let source = Device::new("source");
    let target = Device::new("local");
    let mut channel = source.snapshot().policy.channels[0].clone();
    let id = channel.id.clone();
    let key_ref = uuid::Uuid::new_v4().to_string();
    channel.connection.credential_ref = Some(key_ref.clone());
    atomic_write(
        &source.root.join(POLICY_PATH),
        &serde_json::to_vec(&Policy {
            channels: vec![channel],
            ..Default::default()
        })
        .unwrap(),
        0o600,
    )
    .unwrap();
    let keys = Keys::default();
    keys.0
        .borrow_mut()
        .insert(key_ref, "fixture-legacy-SECRET".into());
    let package = transfer::export(&source.root, source.scope()).unwrap();
    let (bytes, _) = seal_with(
        &source.root,
        &uuid::Uuid::new_v4().to_string(),
        &package,
        true,
        None,
        &keys,
    )
    .unwrap();
    target.retired_history();
    let incoming = receive_context_file(&target.root, &bytes, None, None, &target.home).unwrap();
    let receiver_keys = Keys::default();
    let mut raw = target.raw();
    raw["changed"] = json!(true);
    target.write(raw);
    assert_eq!(
        accept_all(
            &target.root,
            &incoming.id,
            true,
            false,
            None,
            &receiver_keys
        )
        .unwrap_err()
        .code,
        "stale_baseline"
    );
    assert!(receiver_keys.0.borrow().is_empty());
    let compared = recompare_file(&target.root, &incoming.id, None, &target.home).unwrap();
    let local = target.snapshot().policy.channels[0].clone();
    let mapped = transfer::select_received(
        &target.root,
        &compared.id,
        transfer::ReceiveScope {
            channel_ids: vec![id.clone()],
            channel_mappings: BTreeMap::from([(
                id.clone(),
                transfer::ChannelMapping::Local {
                    target_id: local.id.clone(),
                },
            )]),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(mapped.available_keys.is_empty());
    let kept = accept_all(&target.root, &mapped.id, false, false, None, &receiver_keys).unwrap();
    assert_eq!(
        kept.workspace.unwrap().policy.channels[0].connection,
        local.connection
    );
    assert!(receiver_keys.0.borrow().is_empty());
    let separated = transfer::select_received(
        &target.root,
        &compared.id,
        transfer::ReceiveScope {
            channel_ids: vec![id.clone()],
            channel_mappings: BTreeMap::from([(
                id,
                transfer::ChannelMapping::New {
                    label: "separate import".into(),
                },
            )]),
            ..Default::default()
        },
    )
    .unwrap();
    let saved = target.snapshot();
    let accepted = accept_all(
        &target.root,
        &separated.id,
        true,
        false,
        None,
        &receiver_keys,
    )
    .unwrap();
    assert_eq!(accepted.policy, saved.policy);
    let imported = accepted
        .workspace
        .as_ref()
        .unwrap()
        .policy
        .channels
        .iter()
        .find(|c| c.label == "separate import")
        .unwrap();
    assert_eq!(
        receiver_keys
            .read(imported.connection.credential_ref.as_deref().unwrap())
            .unwrap()
            .as_str(),
        "fixture-legacy-SECRET"
    );
    assert_eq!(target.snapshot(), saved);
    let replay = accept_all(
        &target.root,
        &separated.id,
        true,
        false,
        None,
        &receiver_keys,
    )
    .unwrap();
    assert_eq!(replay.workspace, accepted.workspace);
    assert_eq!(receiver_keys.0.borrow().len(), 1);
    target.history_unchanged();
}

#[test]
fn memory_export_freezes_supplied_policy_without_an_ordinary_disk_draft() {
    let device = Device::new("source");
    device.retired_history();
    let baseline = device.snapshot();
    let mut policy = baseline.policy.clone();
    policy.channels[0].label = "unsaved memory label".into();
    let expected = transfer::DraftCandidate {
        expected_library: baseline.library_digest.clone(),
        expected_policy: baseline.policy_digest.clone(),
        expected_workspace: baseline.workspace_digest.clone(),
    };
    let plan = transfer::prepare_memory_file(
        &device.root,
        &device.home,
        device.scope(),
        expected.clone(),
        policy,
        None,
    )
    .unwrap();
    assert_eq!(
        plan.package.payload.policy.as_ref().unwrap().channels[0].label,
        "unsaved memory label"
    );
    assert!(plan.package.payload.candidate.is_some());
    assert_eq!(device.snapshot(), baseline);
    device.history_unchanged();
    let mut raw = device.raw();
    raw["external"] = json!(true);
    device.write(raw);
    assert_eq!(
        transfer::prepare_memory_file(
            &device.root,
            &device.home,
            device.scope(),
            expected,
            baseline.policy,
            None
        )
        .unwrap_err()
        .code,
        "stale_baseline"
    );
}

#[test]
fn interrupted_file_library_commit_resumes_and_receipt_rejects_a_modified_candidate() {
    let source = Device::new("source");
    let target = Device::new("local");
    target.retired_history();
    let reference = source.custom_template();
    let mut scope = source.scope();
    scope.template_refs = vec![reference];
    let exported = source.export(scope, false);
    let bytes = transfer::file_bytes(&source.root, &exported.id).unwrap();
    let mut incoming =
        receive_context_file(&target.root, &bytes, None, None, &target.home).unwrap();
    let before = target.snapshot();
    let record = Adoption {
        id: incoming.id.clone(),
        keys: false,
        webdav: false,
        completed: false,
        extension_credentials: false,
    };
    atomic_write(
        &cache_path(&target.root, &incoming.id, "adoption.json").unwrap(),
        &serde_json::to_vec(&record).unwrap(),
        0o600,
    )
    .unwrap();
    stage_provider_materials(&target.root, &mut incoming, false, None).unwrap();
    let first = transfer::accept(&target.root, &incoming.id).unwrap();
    assert_ne!(first.library_digest, before.library_digest);
    let directory = target.root.join("manager-state/model-config-incoming");
    // Simulate interruption after the library readback and candidate write,
    // before the final acceptance receipt reaches disk.
    std::fs::remove_file(directory.join(format!("{}.accepted.json", incoming.id))).unwrap();
    let resumed = accept_all(
        &target.root,
        &incoming.id,
        false,
        false,
        None,
        &Keys::default(),
    )
    .unwrap();
    assert_eq!(resumed, first);
    assert_eq!(target.snapshot().policy, before.policy);
    target.history_unchanged();
    let path = directory.join(format!("{}.candidate.json", incoming.id));
    let mut candidate: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    candidate["workspace"]["policy"]["channels"][0]["label"] = json!("changed after acceptance");
    std::fs::write(path, serde_json::to_vec(&candidate).unwrap()).unwrap();
    assert_eq!(
        accept(&target.root, &incoming.id, false, None)
            .unwrap_err()
            .code,
        "import_replay_conflict"
    );
    assert_eq!(target.snapshot().policy, before.policy);
}
