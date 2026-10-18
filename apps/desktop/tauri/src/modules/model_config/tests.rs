use super::{matching::*, schema::*, store::*};
use std::collections::BTreeMap;

fn content() -> TemplateContent {
    let mut value = TemplateContent::default();
    value.matching.include = vec![Pattern {
        kind: MatchKind::Prefix,
        value: "gpt-".into(),
    }];
    value
}
fn request(snapshot: &Snapshot) -> SaveTemplate {
    SaveTemplate {
        expected_library: snapshot.library_digest.clone(),
        template_id: None,
        base_revision: None,
        label: "我的编码模型".into(),
        content: content(),
    }
}
fn reference(commit: &Commit) -> TemplateRef {
    TemplateRef {
        template_id: commit.template_id.clone(),
        revision_id: commit.revision_id.clone(),
    }
}

#[test]
fn initial_read_has_trusted_seed_but_does_not_write_working_files() {
    let root = tempfile::tempdir().unwrap();
    let store = ModelStore::new(root.path());
    let snapshot = store.load().unwrap();
    assert_eq!(snapshot.library, seed_library());
    assert!(!root.path().join(LIBRARY_PATH).exists());
    assert!(!root.path().join(POLICY_PATH).exists());
}
#[test]
fn saved_versions_are_complete_immutable_and_default_is_independent() {
    let root = tempfile::tempdir().unwrap();
    let store = ModelStore::new(root.path());
    let first = store.save(request(&store.load().unwrap())).unwrap();
    let old = matching_revision(&first);
    let mut change = request(&first.snapshot);
    change.template_id = Some(first.template_id.clone());
    change.base_revision = Some(first.revision_id.clone());
    change.content.rules.fast = Field::Custom(false);
    let next = store.save(change).unwrap();
    assert_ne!(first.revision_id, next.revision_id);
    assert_eq!(
        revision(&next.snapshot.library, &reference(&first)).unwrap(),
        &old
    );
    assert_eq!(
        revision(&next.snapshot.library, &reference(&next))
            .unwrap()
            .content
            .rules
            .fast,
        Field::Custom(false)
    );
    assert!(next
        .snapshot
        .library
        .templates
        .iter()
        .find(|t| t.id == first.template_id)
        .unwrap()
        .default_revision
        .is_none());
    assert_eq!(store.load().unwrap(), next.snapshot);
    assert!(!root.path().join(POLICY_PATH).exists());
}
fn matching_revision(commit: &Commit) -> Revision {
    revision(&commit.snapshot.library, &reference(commit))
        .unwrap()
        .clone()
}
#[test]
fn semantic_set_reorder_is_no_change_and_never_adds_a_version() {
    let root = tempfile::tempdir().unwrap();
    let store = ModelStore::new(root.path());
    let initial = store.load().unwrap();
    let mut req = request(&initial);
    req.content.rules.reasoning_efforts = Field::Custom(vec!["high".into(), "low".into()]);
    let first = store.save(req.clone()).unwrap();
    req.expected_library = first.snapshot.library_digest.clone();
    req.template_id = Some(first.template_id.clone());
    req.base_revision = Some(first.revision_id.clone());
    req.content.rules.reasoning_efforts =
        Field::Custom(vec!["low".into(), "high".into(), "high".into()]);
    req.content
        .matching
        .channels
        .insert("ignored-in-all".into());
    let again = store.save(req).unwrap();
    assert!(!again.changed);
    assert_eq!(again.snapshot, first.snapshot);
}
#[test]
fn stale_save_and_stale_delete_never_overwrite_newer_content() {
    let root = tempfile::tempdir().unwrap();
    let store = ModelStore::new(root.path());
    let original = store.load().unwrap();
    let first = store.save(request(&original)).unwrap();
    assert_eq!(
        store.save(request(&original)).unwrap_err().code,
        "stale_baseline"
    );
    assert_eq!(
        store
            .delete(
                &original.library_digest,
                &original.policy_digest,
                &original.workspace_digest,
                &first.template_id,
                std::slice::from_ref(&first.revision_id)
            )
            .unwrap_err()
            .code,
        "stale_baseline"
    );
    assert_eq!(store.load().unwrap(), first.snapshot);
}
#[test]
fn defaults_baselines_and_batch_protection_are_atomic() {
    let root = tempfile::tempdir().unwrap();
    let store = ModelStore::new(root.path());
    let first = store.save(request(&store.load().unwrap())).unwrap();
    let current = store
        .set_default(
            &first.snapshot.library_digest,
            &first.template_id,
            &first.revision_id,
        )
        .unwrap();
    assert_eq!(
        store
            .delete(
                &current.library_digest,
                &current.policy_digest,
                &current.workspace_digest,
                &first.template_id,
                std::slice::from_ref(&first.revision_id)
            )
            .unwrap_err()
            .code,
        "protected_revision"
    );
    let builtin = &current.library.templates[0];
    assert_eq!(
        store
            .delete(
                &current.library_digest,
                &current.policy_digest,
                &current.workspace_digest,
                &builtin.id,
                &[builtin.revisions[0].id.clone()]
            )
            .unwrap_err()
            .code,
        "protected_revision"
    );
    assert_eq!(store.load().unwrap(), current);
}
#[test]
fn saved_policy_reference_protects_version_and_policy_is_not_changed() {
    let root = tempfile::tempdir().unwrap();
    let store = ModelStore::new(root.path());
    let first = store.save(request(&store.load().unwrap())).unwrap();
    let policy = Policy {
        template_refs: vec![reference(&first)],
        ..Default::default()
    };
    let bytes = serde_json::to_vec(&policy).unwrap();
    std::fs::write(root.path().join(POLICY_PATH), &bytes).unwrap();
    let current = store.load().unwrap();
    assert_eq!(
        store
            .delete(
                &current.library_digest,
                &current.policy_digest,
                &current.workspace_digest,
                &first.template_id,
                std::slice::from_ref(&first.revision_id)
            )
            .unwrap_err()
            .code,
        "protected_revision"
    );
    let after = store
        .set_default(
            &current.library_digest,
            &first.template_id,
            &first.revision_id,
        )
        .unwrap();
    assert_eq!(after.policy_digest, current.policy_digest);
    assert_eq!(std::fs::read(root.path().join(POLICY_PATH)).unwrap(), bytes);
}
#[test]
fn malformed_or_tampered_library_fails_without_reinitializing() {
    let root = tempfile::tempdir().unwrap();
    let store = ModelStore::new(root.path());
    let first = store.save(request(&store.load().unwrap())).unwrap();
    let path = root.path().join(LIBRARY_PATH);
    let mut library = first.snapshot.library;
    library.templates.last_mut().unwrap().revisions[0]
        .content
        .description = "tampered".into();
    let bytes = serde_json::to_vec(&library).unwrap();
    std::fs::write(&path, &bytes).unwrap();
    assert_eq!(store.load().unwrap_err().code, "digest_mismatch");
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    std::fs::write(&path, b"not json").unwrap();
    assert_eq!(store.load().unwrap_err().code, "invalid_document");
    assert_eq!(std::fs::read(path).unwrap(), b"not json");
}
#[test]
fn backup_failure_stops_a_write() {
    let root = tempfile::tempdir().unwrap();
    let store = ModelStore::new(root.path());
    let first = store.save(request(&store.load().unwrap())).unwrap();
    std::fs::write(root.path().join("backups"), b"blocked").unwrap();
    let before = std::fs::read(root.path().join(LIBRARY_PATH)).unwrap();
    assert!(store
        .set_default(
            &first.snapshot.library_digest,
            &first.template_id,
            &first.revision_id
        )
        .is_err());
    assert_eq!(
        std::fs::read(root.path().join(LIBRARY_PATH)).unwrap(),
        before
    );
}
#[cfg(unix)]
#[test]
fn library_link_outside_root_is_rejected() {
    let root = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("manager-state")).unwrap();
    let target = external.path().join("secret");
    std::fs::write(&target, b"keep").unwrap();
    std::os::unix::fs::symlink(&target, root.path().join(LIBRARY_PATH)).unwrap();
    assert!(ModelStore::new(root.path()).load().is_err());
    assert_eq!(std::fs::read(target).unwrap(), b"keep");
}
#[test]
fn glob_is_literal_and_channels_exclusions_and_suffixes_are_explained() {
    assert!(glob("模型-?*", "模型-甲5"));
    assert!(!glob("gpt.[0-9]", "gpt.5"));
    let mut value = content();
    value.matching.channel_mode = ChannelMode::Whitelist;
    assert!(!preview(&value, "channel", "gpt-x").unwrap().matched);
    value.matching.channel_mode = ChannelMode::Blacklist;
    value.matching.exclude.push(Pattern {
        kind: MatchKind::Glob,
        value: "*-image".into(),
    });
    assert!(!preview(&value, "channel", "gpt-x-image").unwrap().matched);
    let low = ModelRules {
        fast: Field::Custom(false),
        ..Default::default()
    };
    let high = ModelRules {
        fast: Field::Custom(true),
        ..Default::default()
    };
    value.variants = vec![
        Variant {
            id: uuid::Uuid::new_v4().to_string(),
            name: "短后缀".into(),
            suffix: "5".into(),
            rules: low,
            locked_fields: Default::default(),
        },
        Variant {
            id: uuid::Uuid::new_v4().to_string(),
            name: "长后缀".into(),
            suffix: "5.5".into(),
            rules: high,
            locked_fields: Default::default(),
        },
    ];
    assert_eq!(
        preview(&value, "channel", "GPT-5.5").unwrap().rules.fast,
        Field::Custom(true)
    );
}
#[test]
fn resolver_uses_only_adopted_versions_and_rejects_ties() {
    let root = tempfile::tempdir().unwrap();
    let store = ModelStore::new(root.path());
    let first = store.save(request(&store.load().unwrap())).unwrap();
    let second = store.save(request(&first.snapshot)).unwrap();
    let mut policy = Policy::default();
    assert!(
        resolve(&second.snapshot.library, &policy, "channel", "gpt-5")
            .unwrap()
            .is_none()
    );
    policy.template_refs = vec![reference(&first), reference(&second)];
    assert_eq!(
        resolve(&second.snapshot.library, &policy, "channel", "gpt-5")
            .unwrap_err()
            .code,
        "matching_conflict"
    );
    policy.channels.push(ChannelPolicy {
        id: "channel".into(),
        label: "测试".into(),
        template_refs: vec![],
        model_refs: BTreeMap::from([("gpt-5".into(), reference(&first))]),
        connection: Default::default(),
        selection: Default::default(),
        model_order: vec![],
        discovered: vec![],
        defaults: Default::default(),
        overrides: BTreeMap::new(),
        family_overrides: BTreeMap::new(),
    });
    assert_eq!(
        resolve(&second.snapshot.library, &policy, "channel", "gpt-5")
            .unwrap()
            .unwrap()
            .0,
        reference(&first)
    );
}
#[test]
fn malformed_fields_unknown_modes_and_fake_builtin_are_rejected() {
    assert!(
        serde_json::from_str::<ModelRules>(r#"{"fast":{"mode":"custom","value":null}}"#).is_err()
    );
    assert!(serde_json::from_str::<ModelRules>(r#"{"apiKey":"secret"}"#).is_err());
    let mut library = seed_library();
    library.templates[0].id = uuid::Uuid::new_v4().to_string();
    assert_eq!(validate_library(&library).unwrap_err().code, "source");
    let mut value = content();
    value.rules.context_window = Field::Custom(0);
    assert!(value.validate().is_err());
}

fn channel(id: &str) -> ChannelPolicy {
    let node = ServiceNode {
        id: uuid::Uuid::new_v4().to_string(),
        name: "主节点".into(),
        url: "https://example.test/v1".into(),
    };
    ChannelPolicy {
        id: id.into(),
        label: "测试渠道".into(),
        template_refs: vec![],
        model_refs: BTreeMap::new(),
        connection: ChannelConnection {
            provider_id: format!("provider-{}", &id[..4]),
            protocol: "openai-chat".into(),
            nodes: vec![node.clone()],
            active_node_id: node.id,
            credential_ref: None,
            enabled: true,
            allow_private_network: false,
        },
        selection: ModelSelection::None,
        model_order: vec![],
        discovered: vec![],
        defaults: Default::default(),
        overrides: BTreeMap::new(),
        family_overrides: BTreeMap::new(),
    }
}
#[test]
fn projection_keeps_channel_group_order_and_missing_selection_without_claiming_catalog_precision() {
    let a = uuid::Uuid::new_v4().to_string();
    let b = uuid::Uuid::new_v4().to_string();
    let mut first = channel(&a);
    first.model_order = vec!["gpt-b".into(), "gpt-a".into()];
    first.discovered = ["gpt-a", "gpt-b"]
        .iter()
        .map(|id| DiscoveredModel {
            id: (*id).into(),
            label: (*id).into(),
            rules: Default::default(),
        })
        .collect();
    first.selection = ModelSelection::Whitelist(vec!["gpt-a".into(), "not-found".into()]);
    let mut second = channel(&b);
    second.discovered = first.discovered.clone();
    second.selection = ModelSelection::All;
    let policy = Policy {
        channels: vec![first, second],
        ..Default::default()
    };
    let result = super::compiler::project(&seed_library(), &policy).unwrap();
    assert_eq!(
        result
            .rows
            .iter()
            .map(|r| r.model_id.as_str())
            .collect::<Vec<_>>(),
        vec!["gpt-b", "gpt-a", "not-found", "gpt-a", "gpt-b"]
    );
    assert_eq!(result.known_active_models, 3);
    assert_eq!(result.exact_catalog_count, None);
    assert!(result.rows[2].selected && !result.rows[2].discovered);
}
#[test]
fn capability_and_protocol_priorities_differ_and_explicit_false_survives() {
    let mut library = seed_library();
    let template = &mut library.templates[0];
    template.revisions[0].content.matching.include = vec![Pattern {
        kind: MatchKind::Exact,
        value: "gpt-a".into(),
    }];
    template.revisions[0].content.rules.protocol = Field::Custom("openai-responses".into());
    template.revisions[0].content.rules.context_window = Field::Custom(8000);
    template.revisions[0].content.rules.fast = Field::Custom(true);
    let mut c = channel(&uuid::Uuid::new_v4().to_string());
    c.template_refs.push(TemplateRef {
        template_id: template.id.clone(),
        revision_id: template.revisions[0].id.clone(),
    });
    c.defaults.context_window = Field::Custom(4000);
    c.defaults.fast = Field::Custom(false);
    c.defaults.protocol = Field::Custom("openai-chat".into());
    c.discovered.push(DiscoveredModel {
        id: "gpt-a".into(),
        label: "A".into(),
        rules: Default::default(),
    });
    c.selection = ModelSelection::All;
    let mut policy = Policy {
        channels: vec![c],
        ..Default::default()
    };
    let result = super::compiler::project(&library, &policy).unwrap();
    assert_eq!(
        result.rows[0].rules.protocol,
        Field::Custom("openai-responses".into())
    );
    assert_eq!(result.rows[0].rules.context_window, Field::Custom(4000));
    assert_eq!(result.rows[0].rules.fast, Field::Custom(false));
    let single = ModelRules {
        protocol: Field::Custom("openai-chat".into()),
        ..Default::default()
    };
    policy.channels[0].overrides.insert("gpt-a".into(), single);
    assert_eq!(
        super::compiler::project(&library, &policy).unwrap().rows[0]
            .rules
            .protocol,
        Field::Custom("openai-chat".into())
    );
}
#[test]
fn node_validation_and_zero_selection_never_turn_into_all() {
    let mut c = channel(&uuid::Uuid::new_v4().to_string());
    c.discovered.push(DiscoveredModel {
        id: "gpt-new".into(),
        label: "new".into(),
        rules: Default::default(),
    });
    assert!(
        !super::compiler::project(
            &seed_library(),
            &Policy {
                channels: vec![c.clone()],
                ..Default::default()
            }
        )
        .unwrap()
        .rows[0]
            .selected
    );
    c.selection = ModelSelection::Whitelist(vec![]);
    assert_eq!(
        super::compiler::validate_channel(&c).unwrap_err().code,
        "selection"
    );
    c.selection = ModelSelection::None;
    c.connection.nodes[0].url = "https://user:password@example.test/v1".into();
    assert_eq!(
        super::compiler::validate_channel(&c).unwrap_err().code,
        "node_url"
    );
}

#[test]
fn batch_save_is_all_or_nothing_and_uses_the_single_template_service() {
    let root = tempfile::tempdir().unwrap();
    let store = ModelStore::new(root.path());
    let initial = store.load().unwrap();
    let first = request(&initial);
    let mut bad = request(&initial);
    bad.content.matching.include.clear();
    assert!(store
        .save_batch(&initial.library_digest, vec![first.clone(), bad])
        .is_err());
    assert!(!root.path().join(LIBRARY_PATH).exists());
    let mut second = first.clone();
    second.label = "另一个模版".into();
    let result = store
        .save_batch(&initial.library_digest, vec![first, second])
        .unwrap();
    assert_eq!(result.outcomes.len(), 2);
    assert_ne!(
        result.outcomes[0].template_id,
        result.outcomes[1].template_id
    );
    assert_eq!(store.load().unwrap(), result.snapshot);
}
#[test]
fn staged_channel_references_survive_reload_and_protect_versions_without_publishing() {
    let root = tempfile::tempdir().unwrap();
    let store = ModelStore::new(root.path());
    let first = store.save(request(&store.load().unwrap())).unwrap();
    let policy = Policy {
        template_refs: vec![reference(&first)],
        ..Default::default()
    };
    let staged = store
        .stage_policy(
            &first.snapshot.policy_digest,
            &first.snapshot.workspace_digest,
            policy.clone(),
        )
        .unwrap();
    assert_eq!(store.load().unwrap().workspace.unwrap().policy, policy);
    assert!(!root.path().join(POLICY_PATH).exists());
    assert_eq!(
        store
            .delete(
                &staged.library_digest,
                &staged.policy_digest,
                &staged.workspace_digest,
                &first.template_id,
                std::slice::from_ref(&first.revision_id)
            )
            .unwrap_err()
            .code,
        "protected_revision"
    );
    assert_eq!(
        store
            .stage_policy(
                &staged.policy_digest,
                &first.snapshot.workspace_digest,
                Policy::default()
            )
            .unwrap_err()
            .code,
        "stale_workspace"
    );
    let restored = store
        .stage_policy(
            &staged.policy_digest,
            &staged.workspace_digest,
            Policy::default(),
        )
        .unwrap();
    store
        .delete(
            &restored.library_digest,
            &restored.policy_digest,
            &restored.workspace_digest,
            &first.template_id,
            &[first.revision_id],
        )
        .unwrap();
}

#[test]
fn external_field_contract_is_generated_from_the_actual_save_content_type() {
    let schema = serde_json::to_value(schemars::schema_for!(TemplateContent)).unwrap();
    assert_eq!(schema["additionalProperties"], false);
    assert!(schema["properties"].get("matching").is_some());
    assert!(schema["properties"].get("variants").is_some());
    assert!(schema["properties"].get("apiKey").is_none());
}
