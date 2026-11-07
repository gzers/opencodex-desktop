//! All sources and credentials are synthetic and isolated under /tmp.
use super::super::{config_file::v2 as config, schema::Field, source_v2::ModelEditV2, store_v2};
use super::*;
use std::{cell::Cell, path::PathBuf};
use tempfile::TempDir;
const OP: &str = "22222222-2222-4222-8222-222222222222";
const TIME: &str = "2026-11-06T10:00:00Z";
struct NoCredentials;
impl CaptureCredentialsV2 for NoCredentials {
    fn channel_key(&self, _: &Path, _: &SelectedKeySourceV2<'_>) -> ModelResult<Zeroizing<String>> {
        panic!("unselected vault access");
    }
    fn authorization(
        &self,
        _: &Path,
        _: &str,
        _: &Value,
        _: &Value,
    ) -> ModelResult<PortableAuthorization> {
        panic!("unselected authorization access");
    }
    fn webdav_password(&self, _: &Path, _: &str) -> ModelResult<Zeroizing<String>> {
        panic!("unselected sync credential access");
    }
}
fn fixture() -> (TempDir, PathBuf, PathBuf, CaptureRequestV2) {
    let temp = tempfile::Builder::new()
        .prefix("ocxd023-capture-fixture-")
        .tempdir_in(crate::modules::test_sandbox::temporary_parent())
        .unwrap();
    let root = temp.path().join("D");
    let home = temp.path().join("H");
    std::fs::create_dir_all(root.join("manager-state")).unwrap();
    std::fs::create_dir_all(&home).unwrap();
    write_raw(
        &home,
        &json!({"providers":{"fixture":{"adapter":"openai-chat","baseUrl":"https://fixture.invalid/v1","authMode":"key","apiKey":"synthetic-unselected-only","models":["gpt-6-sol","vendor/custom"],"selectedModels":["gpt-6-sol","vendor/custom"],"modelContextWindows":{"vendor/custom":4096},"modelCosts":{"vendor/custom":{"input":1,"futureNested":{"keep":[1,2]}}},"modelPreset":{"mode":"custom"}}},"defaultProvider":"fixture","defaultModel":"fixture/vendor/custom"}),
    );
    let id = config::with_saved_config(&root, &home, |_, p, _, _, _, _| {
        Ok(p.channels[0].id.clone())
    })
    .unwrap();
    let request = CaptureRequestV2 {
        selection: SelectionV2 {
            channels: vec![objects_overlay_v2::ChannelSelectionV2 {
                portable_channel_id: id,
                model_ids: None,
                family_ids: None,
                include_connection: true,
                include_key: false,
                include_authorization: false,
            }],
            ..Default::default()
        },
        mcp_clients: BTreeMap::new(),
        assets_reviewed: false,
    };
    (temp, root, home, request)
}
fn write_raw(home: &Path, raw: &Value) {
    std::fs::write(home.join("config.json"), serde_json::to_vec(raw).unwrap()).unwrap();
}
fn raw(home: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(home.join("config.json")).unwrap()).unwrap()
}
fn code<T>(r: ModelResult<T>) -> String {
    match r {
        Ok(_) => panic!("expected refusal"),
        Err(e) => e.code,
    }
}
fn first(s: &CapturedSnapshotV2) -> &PortableChannelV2 {
    &s.objects.models().payload.policy.as_ref().unwrap().channels[0]
}
fn read_request(s: &config::FileSnapshotV2) -> config::ReadRequestV2 {
    config::ReadRequestV2 {
        expected_source: s.source_digest.clone(),
        expected_library: s.library_digest.clone(),
        expected_draft: s.draft.as_ref().map(|d| d.base.clone()),
    }
}
#[test]
fn actual_temp_source_becomes_v6_protected_objects_and_has_stable_semantics() {
    let (_t, d, h, r) = fixture();
    let before = std::fs::read(h.join("config.json")).unwrap();
    let a = capture(&d, &h, &r, &NoCredentials).unwrap();
    let b = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert_eq!(a.objects.models().format_version, 6);
    assert_eq!(a.source_guard, b.source_guard);
    assert_eq!(a.semantic_digest, b.semantic_digest);
    assert_eq!(a.save_state, b.save_state);
    assert!(a.objects.partitions().content_versions.is_empty());
    assert_eq!(a.summary.channel_key_count, 0);
    assert_eq!(
        first(&a)
            .connection
            .as_ref()
            .unwrap()
            .provider
            .as_ref()
            .unwrap()["modelCosts"]["vendor/custom"]["futureNested"],
        json!({"keep":[1,2]})
    );
    let bytes = file_objects_v2::encode(
        &a.objects,
        OP,
        TIME,
        Some("synthetic-protection-only"),
        None,
    )
    .unwrap();
    let decoded = file_objects_v2::decode(
        bytes.bytes(),
        Some("synthetic-protection-only"),
        &file_objects_v2::supported_schema(),
        None,
    )
    .unwrap();
    assert!(decoded.protected);
    assert_eq!(
        file_objects_v2::semantic_digest(&decoded.objects).unwrap(),
        a.semantic_digest
    );
    assert_eq!(std::fs::read(h.join("config.json")).unwrap(), before);
}
#[test]
fn slash_id_subset_preserves_extra_map_and_drops_other_models() {
    let (_t, d, h, mut r) = fixture();
    r.selection.channels[0].model_ids = Some(vec!["vendor/custom".into()]);
    let s = capture(&d, &h, &r, &NoCredentials).unwrap();
    let c = first(&s);
    assert_eq!(c.models.len(), 1);
    assert_eq!(c.models[0].raw_model_id, "vendor/custom");
    let p = c.connection.as_ref().unwrap().provider.as_ref().unwrap();
    assert_eq!(p["models"], json!(["vendor/custom"]));
    assert_eq!(
        p["modelCosts"]["vendor/custom"]["futureNested"]["keep"],
        json!([1, 2])
    );
    assert_eq!(p["modelPreset"]["mode"], "custom");
    assert!(p.get("apiKey").is_none());
}
#[test]
fn valid_dirty_draft_does_not_become_capture_authority() {
    let (_t, d, h, r) = fixture();
    let before = capture(&d, &h, &r, &NoCredentials).unwrap();
    let s = config::snapshot(&d, &h).unwrap();
    let s = config::begin_draft(&d, &h, read_request(&s)).unwrap();
    let mut draft = s.draft.unwrap();
    draft.policy.channels[0].models[0]
        .explicit_overrides
        .context_window = Field::Custom(12345);
    draft.policy.channels[0].label = "unsaved-draft-label".into();
    std::fs::write(
        d.join(store_v2::DRAFT_PATH),
        serde_json::to_vec(&draft).unwrap(),
    )
    .unwrap();
    let after = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert_eq!(before.semantic_digest, after.semantic_digest);
    assert_eq!(before.source_guard, after.source_guard);
    assert_ne!(first(&after).label, "unsaved-draft-label");
}
#[test]
fn saved_auto_reference_and_manual_override_survive_with_exact_revisions() {
    let (_t, d, h, r) = fixture();
    let s = config::snapshot(&d, &h).unwrap();
    let s = config::read_existing(&d, &h, read_request(&s))
        .unwrap()
        .snapshot;
    let id = r.selection.channels[0].portable_channel_id.clone();
    let saved = config::save(
        &d,
        &h,
        config::SaveRequestV2 {
            expected: s.draft.unwrap().base,
            edits: vec![ModelEditV2::SetOverride {
                channel_id: id,
                raw_model_id: "vendor/custom".into(),
                field: "contextWindow".into(),
                value: json!(8192),
            }],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )
    .unwrap();
    let snapshot = capture(&d, &h, &r, &NoCredentials).unwrap();
    let c = first(&snapshot);
    assert_eq!(c.models, saved.saved_policy.channels[0].models);
    assert!(c
        .models
        .iter()
        .any(|m| matches!(m.binding, Some(schema_v2::ModelBindingV2::AutoRead { .. }))));
    assert_eq!(
        c.models
            .iter()
            .find(|m| m.raw_model_id == "vendor/custom")
            .unwrap()
            .explicit_overrides
            .context_window,
        Field::Custom(8192)
    );
    for t in &snapshot.objects.models().payload.templates {
        for rev in &t.revisions {
            assert!(c.models.iter().any(|m| m.binding.as_ref().is_some_and(|b| b
                .reference()
                .template_id
                == t.id
                && b.reference().revision_id == rev.id)));
        }
    }
}
#[test]
fn unknown_original_field_refuses_export_but_capture_does_not_modify_source() {
    let (_t, d, h, r) = fixture();
    let mut value = raw(&h);
    value["providers"]["fixture"]["futurePublicTop"] = json!({"x":true});
    write_raw(&h, &value);
    assert_eq!(code(capture(&d, &h, &r, &NoCredentials)), "provider_fields");
    assert_eq!(raw(&h), value);
}
#[test]
fn explicit_empty_and_missing_selection_do_not_expand_scope() {
    let (_t, d, h, mut r) = fixture();
    r.selection.channels[0].model_ids = Some(vec![]);
    assert_eq!(
        code(capture(&d, &h, &r, &NoCredentials)),
        "capture_scope_invalid"
    );
    r.selection.channels.clear();
    let s = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert!(s.objects.models().payload.policy.is_none());
    r.selection
        .channels
        .push(objects_overlay_v2::ChannelSelectionV2 {
            portable_channel_id: OP.into(),
            model_ids: None,
            family_ids: None,
            include_connection: true,
            include_key: false,
            include_authorization: false,
        });
    assert!(capture(&d, &h, &r, &NoCredentials).is_err());
}
#[test]
fn capture_request_requires_explicit_nested_choices_and_rejects_unknown_fields() {
    let (_t, _, _, r) = fixture();
    let value = serde_json::to_value(&r).unwrap();
    assert!(serde_json::from_value::<CaptureRequestV2>(value.clone()).is_ok());
    let mut missing = value.clone();
    missing["selection"]["manager"]
        .as_object_mut()
        .unwrap()
        .remove("accountIds");
    assert!(serde_json::from_value::<CaptureRequestV2>(missing).is_err());
    let mut missing = value.clone();
    missing["selection"]
        .as_object_mut()
        .unwrap()
        .remove("skillNames");
    assert!(serde_json::from_value::<CaptureRequestV2>(missing).is_err());
    let mut extra = value;
    extra["public"] = json!({"apiKey":"synthetic"});
    assert!(serde_json::from_value::<CaptureRequestV2>(extra).is_err());
}
#[test]
fn changed_actual_source_changes_guard_but_unselected_secret_does_not() {
    let (_t, d, h, r) = fixture();
    let a = capture(&d, &h, &r, &NoCredentials).unwrap();
    let mut value = raw(&h);
    value["providers"]["fixture"]["apiKey"] = json!("another-unselected-synthetic-key");
    write_raw(&h, &value);
    let b = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert_eq!(a.source_guard, b.source_guard);
    value["providers"]["fixture"]["modelContextWindows"]["vendor/custom"] = json!(16384);
    write_raw(&h, &value);
    let c = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert_ne!(
        b.source_guard.saved_source_digest,
        c.source_guard.saved_source_digest
    );
}
struct ChangingKey(Cell<u32>);
impl CaptureCredentialsV2 for ChangingKey {
    fn channel_key(&self, _: &Path, _: &SelectedKeySourceV2<'_>) -> ModelResult<Zeroizing<String>> {
        let n = self.0.get() + 1;
        self.0.set(n);
        Ok(Zeroizing::new(format!("synthetic-key-{n}")))
    }
}
#[test]
fn repeated_secret_changes_refuse_stale_after_three_observations() {
    let (_t, d, h, mut r) = fixture();
    r.selection.channels[0].include_key = true;
    let credentials = ChangingKey(Cell::new(0));
    assert_eq!(code(capture(&d, &h, &r, &credentials)), "capture_stale");
    assert_eq!(credentials.0.get(), 3);
}
fn mcps(h: &Path, equal: bool) {
    std::fs::create_dir_all(h.join(".codex")).unwrap();
    std::fs::create_dir_all(h.join(".hermes")).unwrap();
    std::fs::write(h.join(".codex/config.toml"),"[mcp_servers.fixture]\ncommand='synthetic-command'\n[mcp_servers.fixture.env]\nAPI_KEY='synthetic-inline-secret'\n").unwrap();
    std::fs::write(h.join(".hermes/config.yaml"),format!("mcp_servers:\n  fixture:\n    command: {}\n    env:\n      API_KEY: synthetic-inline-secret\n",if equal {"synthetic-command"}else{"synthetic-other-command"})).unwrap();
}
fn choose_mcp(d: &Path, h: &Path, r: &mut CaptureRequestV2, client: &str) {
    let sources = BTreeMap::from([("fixture".into(), client.into())]);
    let (assets, conflicts) = portable_assets::export_with_credential_scope(
        d,
        h,
        &[],
        &["fixture".into()],
        &sources,
        false,
        true,
        Some(&[]),
    )
    .unwrap();
    assert!(conflicts.is_empty());
    r.selection.mcp_sources = vec![objects_overlay_v2::McpSourceSelectionV2 {
        name: "fixture".into(),
        definition_digest: objects_overlay_v2::mcp_source_digest(&assets.mcps[0]).unwrap(),
    }];
    r.mcp_clients = sources;
}
#[test]
fn same_named_mcp_requires_client_and_client_identity_changes_guard_even_if_equal() {
    let (_t, d, h, mut r) = fixture();
    mcps(&h, true);
    choose_mcp(&d, &h, &mut r, "codex");
    let a = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert_eq!(a.summary.extension_secret_count, 0);
    r.mcp_clients.clear();
    assert_eq!(
        code(capture(&d, &h, &r, &NoCredentials)),
        "capture_mcp_source_required"
    );
    choose_mcp(&d, &h, &mut r, "hermes");
    let b = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert_eq!(a.semantic_digest, b.semantic_digest);
    assert_ne!(
        a.source_guard.mcp_sources_digest,
        b.source_guard.mcp_sources_digest
    );
    mcps(&h, false);
    assert_eq!(
        code(capture(&d, &h, &r, &NoCredentials)),
        "capture_mcp_source_changed"
    );
}
#[test]
fn mcp_secret_selection_is_independent_of_definition_selection() {
    let (_t, d, h, mut r) = fixture();
    mcps(&h, true);
    choose_mcp(&d, &h, &mut r, "codex");
    let public = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert!(public.objects.partitions().assets.mcps[0]
        .secrets
        .0
        .is_empty());
    r.selection.extension_credentials.mcp_names = vec!["fixture".into()];
    r.assets_reviewed = true;
    let private = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert_eq!(
        private.objects.partitions().assets.mcps[0].secrets.0["/env/API_KEY"],
        "synthetic-inline-secret"
    );
    assert_ne!(
        public.source_guard.saved_source_digest,
        private.source_guard.saved_source_digest
    );
    assert_eq!(
        public.source_guard.mcp_sources_digest,
        private.source_guard.mcp_sources_digest
    );
}
#[test]
fn prefs_extensions_and_account_metadata_use_saved_whitelists() {
    use crate::modules::{
        extensions::{projection::ExtensionProjection, ExtensionConfig, ProjectionFingerprint},
        preferences::{Preferences, PreferencesStore},
        sync::portable_accounts::{self, AccountInfo},
    };
    let (_t, d, h, mut r) = fixture();
    PreferencesStore::new(&d)
        .save(&Preferences {
            interface_scale: 125,
            ..Default::default()
        })
        .unwrap();
    let config = ExtensionConfig {
        source_store: Some("/synthetic/local-only-path".into()),
        skills: vec!["synthetic-skill".into()],
        ..Default::default()
    };
    let projection = ExtensionProjection {
        fingerprint: ProjectionFingerprint::create(
            config.fingerprint(),
            chrono::DateTime::parse_from_rfc3339(TIME).unwrap().to_utc(),
        ),
        config,
    };
    std::fs::write(
        d.join(crate::modules::extensions::projection::CONFIG_RELATIVE_PATH),
        serde_json::to_vec(&projection).unwrap(),
    )
    .unwrap();
    let account = AccountInfo {
        id: OP.into(),
        provider: "chatgpt".into(),
        subject: Some("synthetic-subject".into()),
        label: "synthetic-label".into(),
        email: None,
        origin: None,
    };
    std::fs::write(
        d.join(portable_accounts::IMPORTED),
        serde_json::to_vec(&vec![account]).unwrap(),
    )
    .unwrap();
    r.selection.manager.preferences = true;
    r.selection.manager.extensions = true;
    r.selection.manager.account_ids = vec![OP.into()];
    let s = capture(&d, &h, &r, &NoCredentials).unwrap();
    let value = serde_json::to_value(s.objects.partitions().manager).unwrap();
    assert_eq!(value["preferences"].as_object().unwrap().len(), 4);
    assert_eq!(value["preferences"]["interface_scale"], 125);
    assert!(value["extensions"]["source_store"].is_null());
    assert_eq!(s.summary.account_count, 1);
    assert!(!serde_json::to_string(&value)
        .unwrap()
        .contains("/synthetic/local-only-path"));
}
#[test]
fn missing_or_corrupt_selected_saved_sources_are_errors() {
    let (_t, d, h, mut r) = fixture();
    r.selection.manager.preferences = true;
    assert!(capture(&d, &h, &r, &NoCredentials).is_err());
    r.selection.manager.preferences = false;
    r.selection.manager.extensions = true;
    assert!(capture(&d, &h, &r, &NoCredentials).is_err());
    r.selection.manager.extensions = false;
    r.selection.skill_names = vec!["missing-synthetic-skill".into()];
    assert!(capture(&d, &h, &r, &NoCredentials).is_err());
    r.selection.skill_names.clear();
    std::fs::write(h.join("config.json"), b"{broken").unwrap();
    assert!(capture(&d, &h, &r, &NoCredentials).is_err());
}
#[test]
fn content_times_require_exact_scope_and_digest_and_are_never_inferred() {
    let (_t, d, h, r) = fixture();
    let a = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert!(a.objects.partitions().content_versions.is_empty());
    let plan =
        content_versions::plan_save_at(&d, &SaveState::new(), &a.save_state, TIME, OP).unwrap();
    std::fs::write(d.join(content_versions::SAVED_VERSIONS_PATH), plan.after).unwrap();
    let b = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert!(!b.objects.partitions().content_versions.is_empty());
    for v in b.objects.partitions().content_versions {
        assert_eq!(v.saved_at, TIME);
        assert_eq!(b.save_state.get(&v.scope_key), Some(&v.content_digest));
    }
    let mut value = raw(&h);
    value["providers"]["fixture"]["alias"] = json!("changed-public");
    write_raw(&h, &value);
    let c = capture(&d, &h, &r, &NoCredentials).unwrap();
    for v in c.objects.partitions().content_versions {
        assert_eq!(c.save_state.get(&v.scope_key), Some(&v.content_digest));
    }
    assert!(!c
        .objects
        .partitions()
        .content_versions
        .iter()
        .any(|v| b.save_state.get(&v.scope_key) != c.save_state.get(&v.scope_key)));
}
#[test]
fn webdav_remote_self_scope_is_refused_before_any_credential_read() {
    let (_t, d, h, mut r) = fixture();
    r.selection.manager.webdav_ids = vec!["synthetic-endpoint".into()];
    r.selection.manager.webdav_password_ids = r.selection.manager.webdav_ids.clone();
    assert_eq!(code(capture(&d, &h, &r, &NoCredentials)), "sync_self_scope");
}
#[test]
fn multiple_protocol_groups_keep_original_public_model_maps_and_source_protocol() {
    use super::super::protocol_groups::{mapping_marker_v2, RuntimeMappingV2};
    let (_t, d, h, mut r) = fixture();
    let mapping = RuntimeMappingV2 {
        channel_id: r.selection.channels[0].portable_channel_id.clone(),
        protocol: schema_v2::ProtocolV2::Anthropic,
        wire_profile_digest: schema_v2::digest(b"synthetic-group"),
        provider_id: "actual-group".into(),
        anchor: false,
    };
    let s = config::snapshot(&d, &h).unwrap();
    let s = config::begin_draft(&d, &h, read_request(&s)).unwrap();
    config::save(
        &d,
        &h,
        config::SaveRequestV2 {
            expected: s.draft.unwrap().base,
            edits: vec![],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )
    .unwrap();
    std::fs::write(
        d.join(store_v2::MAPPING_PATH),
        serde_json::to_vec(&MappingFileV2 {
            schema_version: 2,
            mappings: vec![mapping.clone()],
        })
        .unwrap(),
    )
    .unwrap();
    let mut value = raw(&h);
    value["providers"]["actual-group"] = json!({"adapter":"anthropic","baseUrl":"https://fixture.invalid/v1","models":["group/raw"],"selectedModels":["group/raw"],"modelCosts":{"group/raw":{"input":7,"unknownNested":{"original":true}}},"note":mapping_marker_v2(&mapping)});
    write_raw(&h, &value);
    r.selection.channels[0].model_ids = Some(vec!["group/raw".into()]);
    let s = capture(&d, &h, &r, &NoCredentials).unwrap();
    let c = first(&s);
    assert_eq!(c.models[0].raw_model_id, "group/raw");
    assert_eq!(
        c.models[0].source.fields["protocol"].value,
        json!("anthropic")
    );
    let public = c.connection.as_ref().unwrap().provider.as_ref().unwrap();
    assert_eq!(public["adapter"], "openai-chat");
    assert_eq!(
        public["modelCosts"],
        json!({"group/raw":{"input":7,"unknownNested":{"original":true}}})
    );
    assert_eq!(public["models"], json!(["group/raw"]));
}
#[test]
fn family_and_template_ref_exports_use_saved_exact_references() {
    let (_t, d, h, mut r) = fixture();
    let s = config::snapshot(&d, &h).unwrap();
    let s = config::read_existing(&d, &h, read_request(&s))
        .unwrap()
        .snapshot;
    let saved = config::save(
        &d,
        &h,
        config::SaveRequestV2 {
            expected: s.draft.unwrap().base,
            edits: vec![],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )
    .unwrap();
    let reference = saved.saved_policy.channels[0]
        .models
        .iter()
        .find_map(|m| m.binding.as_ref().map(|b| b.reference().clone()))
        .unwrap();
    r.selection.channels[0].family_ids = Some(vec![reference.template_id.clone()]);
    r.selection.channels[0].include_connection = false;
    let family = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert!(first(&family).connection.is_none());
    assert!(first(&family).models.iter().all(|m| m
        .binding
        .as_ref()
        .unwrap()
        .reference()
        .template_id
        == reference.template_id));
    r.selection.channels.clear();
    r.selection.template_refs = vec![reference.clone()];
    let template = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert!(template.objects.models().payload.policy.is_none());
    assert_eq!(template.objects.models().payload.templates.len(), 1);
    assert_eq!(
        template.objects.models().payload.templates[0]
            .revisions
            .len(),
        1
    );
    assert_eq!(
        template.objects.models().payload.templates[0].revisions[0].id,
        reference.revision_id
    );
}
#[test]
fn skill_content_and_secret_files_have_independent_scope() {
    let (_t, d, h, mut r) = fixture();
    let source =
        crate::modules::extensions::source_dir::default_source_dir(&h).join("fixture-skill");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(
        source.join("SKILL.md"),
        "# Synthetic skill\nA harmless instruction.\n",
    )
    .unwrap();
    std::fs::write(source.join(".env"), "TOKEN=synthetic-skill-token\n").unwrap();
    r.selection.skill_names = vec!["fixture-skill".into()];
    let public = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert_eq!(public.objects.partitions().assets.skills[0].files.len(), 1);
    assert!(!public.objects.partitions().assets.skills[0].contains_secrets);
    r.selection.extension_credentials.skill_names = vec!["fixture-skill".into()];
    r.assets_reviewed = true;
    let private = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert_eq!(private.objects.partitions().assets.skills[0].files.len(), 2);
    assert!(private.objects.partitions().assets.skills[0].contains_secrets);
    assert_ne!(
        public.source_guard.saved_source_digest,
        private.source_guard.saved_source_digest
    );
}
struct MutateActualSource;
impl CaptureCredentialsV2 for MutateActualSource {
    fn channel_key(
        &self,
        home: &Path,
        _: &SelectedKeySourceV2<'_>,
    ) -> ModelResult<Zeroizing<String>> {
        let mut value = raw(home);
        value["providers"]["fixture"]["alias"] = json!("changed-during-observation");
        write_raw(home, &value);
        Ok(Zeroizing::new("synthetic-stable-key".into()))
    }
}
#[test]
fn final_model_recheck_refuses_source_changed_during_capture() {
    let (_t, d, h, mut r) = fixture();
    r.selection.channels[0].include_key = true;
    assert_eq!(
        code(capture(&d, &h, &r, &MutateActualSource)),
        "capture_stale"
    );
}
struct FixedKey(&'static str);
impl CaptureCredentialsV2 for FixedKey {
    fn channel_key(&self, _: &Path, _: &SelectedKeySourceV2<'_>) -> ModelResult<Zeroizing<String>> {
        Ok(Zeroizing::new(self.0.to_owned()))
    }
}
#[test]
fn selected_secret_hash_changes_saved_guard_without_changing_scope() {
    let (_t, d, h, mut r) = fixture();
    r.selection.channels[0].include_key = true;
    let a = capture(&d, &h, &r, &FixedKey("synthetic-first")).unwrap();
    let b = capture(&d, &h, &r, &FixedKey("synthetic-second")).unwrap();
    assert_ne!(
        a.source_guard.saved_source_digest,
        b.source_guard.saved_source_digest
    );
    assert_eq!(a.source_guard.scope_digest, b.source_guard.scope_digest);
    assert_eq!(a.summary.channel_key_count, 1);
}
#[test]
fn namespace_and_manual_binding_remain_actual_saved_values() {
    let (_t, d, h, r) = fixture();
    let s = config::snapshot(&d, &h).unwrap();
    let s = config::read_existing(&d, &h, read_request(&s))
        .unwrap()
        .snapshot;
    let reference = s.draft.as_ref().unwrap().policy.channels[0]
        .models
        .iter()
        .find_map(|m| m.binding.as_ref().map(|b| b.reference().clone()))
        .unwrap();
    let saved = config::save(
        &d,
        &h,
        config::SaveRequestV2 {
            expected: s.draft.unwrap().base,
            edits: vec![ModelEditV2::SetManualBinding {
                channel_id: r.selection.channels[0].portable_channel_id.clone(),
                raw_model_id: "vendor/custom".into(),
                reference,
            }],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )
    .unwrap();
    let mut policy = saved.saved_policy;
    policy.channels[0].namespace = Some(schema_v2::NamespaceV2 {
        channel_id: policy.channels[0].id.clone(),
        remove_leading_prefixes: vec!["vendor/".into()],
    });
    std::fs::write(
        d.join(store_v2::POLICY_PATH),
        serde_json::to_vec(&policy).unwrap(),
    )
    .unwrap();
    let mut ledger: store_v2::LedgerFileV2 =
        serde_json::from_slice(&std::fs::read(d.join(store_v2::LEDGER_PATH)).unwrap()).unwrap();
    ledger.ledger.source_policy_digest =
        super::super::source_v2::policy_semantic_digest_v2(&policy).unwrap();
    std::fs::write(
        d.join(store_v2::LEDGER_PATH),
        serde_json::to_vec(&ledger).unwrap(),
    )
    .unwrap();
    let s = capture(&d, &h, &r, &NoCredentials).unwrap();
    let c = first(&s);
    assert_eq!(c.namespace, policy.channels[0].namespace);
    assert!(matches!(
        c.models
            .iter()
            .find(|m| m.raw_model_id == "vendor/custom")
            .unwrap()
            .binding,
        Some(schema_v2::ModelBindingV2::Manual { .. })
    ));
}
struct FixtureAuthorization;
impl CaptureCredentialsV2 for FixtureAuthorization {
    fn authorization(
        &self,
        home: &Path,
        provider: &str,
        scoped: &Value,
        source: &Value,
    ) -> ModelResult<PortableAuthorization> {
        // This fixture contains only a literal synthetic key: no environment,
        // keychain or imported credential references are present.
        assert_eq!(
            raw(home)["providers"][provider]["apiKey"],
            "synthetic-unselected-only"
        );
        provider_auth::export_portable_scoped(home, provider, scoped, source)
    }
}
#[test]
fn selected_file_authorization_uses_actual_source_and_validates_scoped_public() {
    let (_t, d, h, mut r) = fixture();
    r.selection.channels[0].include_authorization = true;
    r.selection.channels[0].model_ids = Some(vec!["vendor/custom".into()]);
    let s = capture(&d, &h, &r, &FixtureAuthorization).unwrap();
    assert_eq!(s.summary.channel_authorization_count, 1);
    assert_eq!(s.summary.channel_key_count, 0);
    let bytes = file_objects_v2::encode(
        &s.objects,
        OP,
        TIME,
        Some("synthetic-auth-protection"),
        Some(&ActualAuthorization),
    )
    .unwrap();
    assert!(bytes.protected);
}
#[test]
fn mixed_managed_client_capture_honors_explicit_source_and_binds_guard() {
    let (_t, d, h, mut r) = fixture();
    mcps(&h, true);
    choose_mcp(&d, &h, &mut r, "codex");
    let managed = d.join("manager-state/mcp-definitions.json");
    std::fs::write(&managed,serde_json::to_vec(&json!([{"name":"fixture","definition":{"command":"synthetic-managed-command"},"secretFields":[],"references":{}}])).unwrap()).unwrap();
    let client = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert_eq!(
        client.objects.partitions().assets.mcps[0].definition["command"],
        "synthetic-command"
    );
    r.mcp_clients.clear();
    assert_eq!(
        code(capture(&d, &h, &r, &NoCredentials)),
        "capture_mcp_source_required"
    );
    choose_mcp(&d, &h, &mut r, "managed");
    let stored = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert_eq!(
        stored.objects.partitions().assets.mcps[0].definition["command"],
        "synthetic-managed-command"
    );
    assert_ne!(
        client.source_guard.mcp_sources_digest,
        stored.source_guard.mcp_sources_digest
    );
    let client_public = client.objects.partitions().assets.mcps[0].clone();
    std::fs::write(&managed,serde_json::to_vec(&json!([{"name":"fixture","definition":client_public.definition,"secretFields":client_public.secret_fields,"references":{}}])).unwrap()).unwrap();
    choose_mcp(&d, &h, &mut r, "managed");
    let equal = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert_eq!(client.semantic_digest, equal.semantic_digest);
    assert_ne!(
        client.source_guard.mcp_sources_digest,
        equal.source_guard.mcp_sources_digest
    );
    r.mcp_clients.clear();
    assert_eq!(
        code(capture(&d, &h, &r, &NoCredentials)),
        "capture_mcp_source_required"
    );
}
#[test]
fn mixed_explicit_client_ignores_broken_unselected_managed_source() {
    let (_t, d, h, mut r) = fixture();
    mcps(&h, true);
    choose_mcp(&d, &h, &mut r, "codex");
    std::fs::write(
        d.join("manager-state/mcp-definitions.json"),
        b"broken-unselected-managed-source",
    )
    .unwrap();
    let s = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert_eq!(
        s.objects.partitions().assets.mcps[0].definition["command"],
        "synthetic-command"
    );
    r.mcp_clients.insert("fixture".into(), "managed".into());
    assert!(capture(&d, &h, &r, &NoCredentials).is_err());
}
// Metadata refs are stale synthetic identities, never installed in a vault.
fn save_with_stale_metadata_ref(d: &Path, h: &Path) {
    let s = config::snapshot(d, h).unwrap();
    let s = config::begin_draft(d, h, read_request(&s)).unwrap();
    config::save(
        d,
        h,
        config::SaveRequestV2 {
            expected: s.draft.unwrap().base,
            edits: vec![],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )
    .unwrap();
    let mut policy: schema_v2::PolicyV2 =
        serde_json::from_slice(&std::fs::read(d.join(store_v2::POLICY_PATH)).unwrap()).unwrap();
    policy.channels[0].connection.credential_ref = Some(OP.into());
    std::fs::write(
        d.join(store_v2::POLICY_PATH),
        serde_json::to_vec(&policy).unwrap(),
    )
    .unwrap();
    let mut ledger: store_v2::LedgerFileV2 =
        serde_json::from_slice(&std::fs::read(d.join(store_v2::LEDGER_PATH)).unwrap()).unwrap();
    ledger.ledger.source_policy_digest =
        super::super::source_v2::policy_semantic_digest_v2(&policy).unwrap();
    std::fs::write(
        d.join(store_v2::LEDGER_PATH),
        serde_json::to_vec(&ledger).unwrap(),
    )
    .unwrap();
}
#[test]
fn system_key_uses_externally_changed_original_and_never_metadata_vault_fallback() {
    let (_t, d, h, mut r) = fixture();
    save_with_stale_metadata_ref(&d, &h);
    let mut value = raw(&h);
    value["providers"]["fixture"]["apiKey"] = json!("synthetic-current-original-key");
    value["providers"]["fixture"]
        .as_object_mut()
        .unwrap()
        .remove("apiKeyPool");
    write_raw(&h, &value);
    r.selection.channels[0].include_key = true;
    let s = capture(&d, &h, &r, &SystemCaptureCredentialsV2).unwrap();
    assert_eq!(
        s.objects
            .partitions()
            .channel_keys
            .get(&r.selection.channels[0].portable_channel_id),
        Some("synthetic-current-original-key")
    );
    value["providers"]["fixture"]
        .as_object_mut()
        .unwrap()
        .remove("apiKey");
    write_raw(&h, &value);
    assert_eq!(
        code(capture(&d, &h, &r, &SystemCaptureCredentialsV2)),
        "capture_credential_missing"
    );
}
struct SelectedReferenceMock {
    calls: Cell<usize>,
}
impl CaptureCredentialsV2 for SelectedReferenceMock {
    fn keychain_key(
        &self,
        home: &Path,
        source: &SelectedKeySourceV2<'_>,
        account: &str,
    ) -> ModelResult<Zeroizing<String>> {
        self.calls.set(self.calls.get() + 1);
        assert_eq!(source.provider(), "fixture");
        assert_eq!(account, "fixture/abcd1234");
        assert_eq!(source.value(), "keychain:fixture/abcd1234");
        assert!(schema_v2::valid_digest(source.raw_digest()));
        assert_eq!(
            source.actual_source_public()["baseUrl"],
            "https://fixture.invalid/v1"
        );
        assert_eq!(
            source.scoped_public()["baseUrl"],
            source.actual_source_public()["baseUrl"]
        );
        assert_eq!(
            source.scoped_public()["adapter"],
            source.actual_source_public()["adapter"]
        );
        assert_eq!(
            raw(home)["providers"][source.provider()]["apiKey"],
            source.value()
        );
        Ok(Zeroizing::new("synthetic-actual-keychain-value".into()))
    }
    fn environment_key(
        &self,
        _: &Path,
        source: &SelectedKeySourceV2<'_>,
        name: &str,
    ) -> ModelResult<Zeroizing<String>> {
        self.calls.set(self.calls.get() + 1);
        assert_eq!(name, "OCXD023_SYNTHETIC_CAPTURE_KEY");
        assert_eq!(source.provider(), "fixture");
        Ok(Zeroizing::new("synthetic-environment-value".into()))
    }
}
#[test]
fn keychain_reference_requires_actual_owner_direction_and_explicit_key_selection() {
    let (_t, d, h, mut r) = fixture();
    save_with_stale_metadata_ref(&d, &h);
    let mut value = raw(&h);
    value["providers"]["fixture"]["apiKey"] = json!("keychain:fixture/abcd1234");
    value["providers"]["fixture"]
        .as_object_mut()
        .unwrap()
        .remove("apiKeyPool");
    write_raw(&h, &value);
    let credentials = SelectedReferenceMock {
        calls: Cell::new(0),
    };
    capture(&d, &h, &r, &credentials).unwrap();
    assert_eq!(credentials.calls.get(), 0);
    r.selection.channels[0].include_key = true;
    let s = capture(&d, &h, &r, &credentials).unwrap();
    assert_eq!(credentials.calls.get(), 2);
    assert_eq!(s.summary.channel_key_count, 1);
    value["providers"]["fixture"]["apiKey"] = json!("keychain:other-provider/abcd1234");
    write_raw(&h, &value);
    assert_eq!(
        code(capture(&d, &h, &r, &credentials)),
        "provider_auth_reference"
    );
    assert_eq!(credentials.calls.get(), 2);
    value["providers"]["fixture"]
        .as_object_mut()
        .unwrap()
        .remove("apiKey");
    write_raw(&h, &value);
    assert_eq!(
        code(capture(&d, &h, &r, &credentials)),
        "capture_credential_missing"
    );
    assert_eq!(credentials.calls.get(), 2);
}
#[test]
fn original_environment_reference_is_explicit_and_resolved_only_after_selection() {
    let (_t, d, h, mut r) = fixture();
    let mut value = raw(&h);
    value["providers"]["fixture"]["apiKey"] =
        json!(concat!("$", "{OCXD023_SYNTHETIC_CAPTURE_KEY}"));
    write_raw(&h, &value);
    let credentials = SelectedReferenceMock {
        calls: Cell::new(0),
    };
    capture(&d, &h, &r, &credentials).unwrap();
    assert_eq!(credentials.calls.get(), 0);
    r.selection.channels[0].include_key = true;
    capture(&d, &h, &r, &credentials).unwrap();
    assert_eq!(credentials.calls.get(), 2);
    value["providers"]["fixture"]["apiKey"] = json!(concat!("$", "{BAD-NAME}"));
    write_raw(&h, &value);
    assert_eq!(
        code(capture(&d, &h, &r, &credentials)),
        "provider_auth_reference"
    );
    assert_eq!(credentials.calls.get(), 2);
}
fn slot(key: &str) -> Value {
    let hash = schema_v2::digest(key.as_bytes());
    json!({"id":&hash[7..15],"key":key,"label":null})
}
#[test]
fn simple_key_requires_confirmed_pool_item_and_never_guesses_multi_pool_active_key() {
    let (_t, d, h, mut r) = fixture();
    let mut value = raw(&h);
    let provider = &mut value["providers"]["fixture"];
    provider.as_object_mut().unwrap().remove("apiKey");
    provider["apiKeyPool"] = json!([slot("synthetic-only-pool-key")]);
    write_raw(&h, &value);
    r.selection.channels[0].include_key = true;
    let s = capture(&d, &h, &r, &SystemCaptureCredentialsV2).unwrap();
    assert_eq!(
        s.objects
            .partitions()
            .channel_keys
            .get(&r.selection.channels[0].portable_channel_id),
        Some("synthetic-only-pool-key")
    );
    value["providers"]["fixture"]["apiKeyPool"] = json!([
        slot("synthetic-only-pool-key"),
        slot("synthetic-second-pool-key")
    ]);
    write_raw(&h, &value);
    assert_eq!(
        code(capture(&d, &h, &r, &SystemCaptureCredentialsV2)),
        "capture_key_selection_unsupported"
    );
    value["providers"]["fixture"]["apiKey"] = json!("synthetic-second-pool-key");
    write_raw(&h, &value);
    let s = capture(&d, &h, &r, &SystemCaptureCredentialsV2).unwrap();
    assert_eq!(
        s.objects
            .partitions()
            .channel_keys
            .get(&r.selection.channels[0].portable_channel_id),
        Some("synthetic-second-pool-key")
    );
    value["providers"]["fixture"]["apiKey"] = json!("synthetic-not-in-pool");
    write_raw(&h, &value);
    assert_eq!(
        code(capture(&d, &h, &r, &SystemCaptureCredentialsV2)),
        "capture_key_source_invalid"
    );
}
#[test]
fn selected_key_evidence_rejects_public_direction_mismatch_before_resolver() {
    let (_t, _, h, _) = fixture();
    let value = raw(&h);
    let row = &value["providers"]["fixture"];
    let public = super::super::provider_fields::public_from_raw(row).unwrap();
    let mut changed = public.clone();
    changed["baseUrl"] = json!("https://different.invalid/v1");
    assert_eq!(
        code(selected_key_source(
            "fixture",
            row,
            &changed,
            &public,
            &schema_v2::digest(b"synthetic-source")
        )),
        "capture_credential_scope"
    );
}
#[test]
fn sole_owned_keychain_pool_item_is_confirmed_but_foreign_slot_identity_is_not() {
    let (_t, d, h, mut r) = fixture();
    let mut value = raw(&h);
    value["providers"]["fixture"]["apiKey"] = json!("keychain:fixture/abcd1234");
    value["providers"]["fixture"]["apiKeyPool"] =
        json!([{"id":"abcd1234","key":"keychain:fixture/abcd1234"}]);
    write_raw(&h, &value);
    r.selection.channels[0].include_key = true;
    let credentials = SelectedReferenceMock {
        calls: Cell::new(0),
    };
    capture(&d, &h, &r, &credentials).unwrap();
    assert_eq!(credentials.calls.get(), 2);
    value["providers"]["fixture"]["apiKeyPool"][0]["id"] = json!("ffff1234");
    write_raw(&h, &value);
    assert_eq!(
        code(capture(&d, &h, &r, &credentials)),
        "capture_key_source_invalid"
    );
    assert_eq!(credentials.calls.get(), 2);
}
#[test]
fn saved_only_boundary_ignores_corrupt_draft_and_refuses_pending_without_recovery() {
    let (_t, d, h, r) = fixture();
    let a = capture(&d, &h, &r, &NoCredentials).unwrap();
    let draft = d.join(store_v2::DRAFT_PATH);
    std::fs::write(&draft, b"corrupt-unsaved-draft").unwrap();
    let b = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert_eq!(a.source_guard, b.source_guard);
    let original = std::fs::read(h.join("config.json")).unwrap();
    for relative in [
        "manager-state/saved-file-save-pending.json",
        "manager-state/provider-rename-pending.json",
        "manager-state/model-transaction-v2.pending.json",
    ] {
        let pending = d.join(relative);
        std::fs::write(&pending, b"synthetic-pending-must-not-be-recovered").unwrap();
        assert_eq!(
            code(capture(&d, &h, &r, &NoCredentials)),
            "model_transaction_pending"
        );
        assert_eq!(
            std::fs::read(&pending).unwrap(),
            b"synthetic-pending-must-not-be-recovered"
        );
        assert_eq!(std::fs::read(h.join("config.json")).unwrap(), original);
        assert_eq!(std::fs::read(&draft).unwrap(), b"corrupt-unsaved-draft");
        std::fs::remove_file(pending).unwrap();
    }
}
const DAV_REF: &str = "cred_33333333-3333-4333-8333-333333333333";
const DAV_ID: &str = "selected-synthetic-endpoint";
fn saved_webdav(d: &Path) {
    use crate::{
        infrastructure::keychain,
        modules::sync::config::{
            self, CredentialRef, SyncConfig, SyncConfigStore, SyncEndpointConfig,
        },
    };
    SyncConfigStore::new(d)
        .save_config(&SyncConfig {
            schema_version: 1,
            endpoints: vec![SyncEndpointConfig {
                endpoint_id: DAV_ID.into(),
                url: "https://dav.fixture.invalid".into(),
                remote_path: "synthetic-scope".into(),
                username: "synthetic-user".into(),
                credential_ref: CredentialRef {
                    ref_id: DAV_REF.into(),
                    backend: "keychain".into(),
                    service_name: keychain::keychain_service_name().into(),
                    account_key: keychain::account_key(
                        keychain::WEBDAV_CREDENTIAL_PURPOSE,
                        DAV_REF,
                    )
                    .unwrap(),
                    purpose: keychain::WEBDAV_CREDENTIAL_PURPOSE.into(),
                    created_at: TIME.into(),
                    updated_at: TIME.into(),
                },
                tls_policy: config::TLS_POLICY_VERIFY_REQUIRED.into(),
                legacy_encryption: String::new(),
                conflict_policy: config::CONFLICT_POLICY_ASK.into(),
            }],
        })
        .unwrap();
}
// Only test fixtures know the non-Windows controlled plain file schema; the
// production capture always delegates to credentials::load_optional_at.
fn credential_fixture(d: &Path, account: &str, value: &str, protection: &str) -> PathBuf {
    let path = d
        .join("manager-state/credentials")
        .join(format!("{account}.json"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        serde_json::to_vec(&json!({"account":account,"value":value,"protection":protection}))
            .unwrap(),
    )
    .unwrap();
    path
}
fn dav_account() -> String {
    crate::infrastructure::keychain::account_key(
        crate::infrastructure::keychain::WEBDAV_CREDENTIAL_PURPOSE,
        DAV_REF,
    )
    .unwrap()
}
#[cfg(not(windows))]
#[test]
fn system_webdav_uses_exact_saved_ref_in_injected_root_without_global_cache() {
    let (_t, d, h, mut r) = fixture();
    saved_webdav(&d);
    let account = dav_account();
    let path = credential_fixture(&d, &account, "synthetic-first-dav-password", "plain");
    // A bogus endpoint-named file must never replace the saved ref identity.
    credential_fixture(
        &d,
        &format!("ocx.dav.{DAV_ID}"),
        "synthetic-wrong-endpoint-file",
        "plain",
    );
    r.selection.usage = UsageV2::LocalFile;
    r.selection.manager.webdav_ids = vec![DAV_ID.into()];
    r.selection.manager.webdav_password_ids = vec![DAV_ID.into()];
    let before = capture(&d, &h, &r, &SystemCaptureCredentialsV2).unwrap();
    assert_eq!(
        before.objects.partitions().webdav_passwords.get(DAV_ID),
        Some("synthetic-first-dav-password")
    );
    assert_eq!(before.summary.webdav_password_count, 1);
    credential_fixture(&d, &account, "synthetic-updated-dav-password", "plain");
    let after = capture(&d, &h, &r, &SystemCaptureCredentialsV2).unwrap();
    assert_eq!(
        after.objects.partitions().webdav_passwords.get(DAV_ID),
        Some("synthetic-updated-dav-password")
    );
    assert_ne!(
        before.source_guard.saved_source_digest,
        after.source_guard.saved_source_digest
    );
    let other = d.parent().unwrap().join("other-D");
    std::fs::create_dir_all(other.join("manager-state")).unwrap();
    saved_webdav(&other);
    credential_fixture(&other, &account, "synthetic-other-root-password", "plain");
    assert_eq!(
        &*SystemCaptureCredentialsV2
            .webdav_password(&other, DAV_ID)
            .unwrap(),
        "synthetic-other-root-password"
    );
    assert_eq!(
        &*SystemCaptureCredentialsV2
            .webdav_password(&d, DAV_ID)
            .unwrap(),
        "synthetic-updated-dav-password"
    );
    assert_eq!(
        code(SystemCaptureCredentialsV2.webdav_password(&d, DAV_REF)),
        "capture_selection_missing"
    );
    std::fs::remove_file(path).unwrap();
    assert_eq!(
        code(SystemCaptureCredentialsV2.webdav_password(&d, DAV_ID)),
        "capture_credential_missing"
    );
}
#[test]
fn system_webdav_missing_or_corrupt_source_never_falls_back() {
    let (_t, d, _, _) = fixture();
    assert_eq!(
        code(SystemCaptureCredentialsV2.webdav_password(&d, DAV_ID)),
        "capture_selection_missing"
    );
    saved_webdav(&d);
    assert_eq!(
        code(SystemCaptureCredentialsV2.webdav_password(&d, DAV_ID)),
        "capture_credential_missing"
    );
    let account = dav_account();
    let path = credential_fixture(&d, &account, "synthetic-invalid-protection", "unsupported");
    assert_eq!(
        code(SystemCaptureCredentialsV2.webdav_password(&d, DAV_ID)),
        "capture_credential_source_invalid"
    );
    std::fs::write(&path, b"broken-selected-credential").unwrap();
    assert_eq!(
        code(SystemCaptureCredentialsV2.webdav_password(&d, DAV_ID)),
        "capture_credential_source_invalid"
    );
    std::fs::write(
        &path,
        serde_json::to_vec(
            &json!({"account":"wrong-owner","value":"synthetic-wrong-owner","protection":"plain"}),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        code(SystemCaptureCredentialsV2.webdav_password(&d, DAV_ID)),
        "capture_credential_source_invalid"
    );
    std::fs::write(
        d.join(crate::modules::sync::config::SYNC_ENDPOINTS_RELATIVE_PATH),
        b"broken-selected-config",
    )
    .unwrap();
    assert_eq!(
        code(SystemCaptureCredentialsV2.webdav_password(&d, DAV_ID)),
        "capture_source_invalid"
    );
}
#[test]
fn unselected_webdav_password_does_not_read_bad_credential_and_remote_scope_still_refuses() {
    let (_t, d, h, mut r) = fixture();
    saved_webdav(&d);
    let path = credential_fixture(
        &d,
        &dav_account(),
        "synthetic-unreadable-definition",
        "unsupported",
    );
    r.selection.usage = UsageV2::LocalFile;
    r.selection.manager.webdav_ids = vec![DAV_ID.into()];
    let s = capture(&d, &h, &r, &NoCredentials).unwrap();
    assert_eq!(s.summary.webdav_password_count, 0);
    assert_eq!(s.summary.webdav_count, 1);
    r.selection.manager.webdav_password_ids = vec![DAV_ID.into()];
    assert_eq!(
        code(capture(&d, &h, &r, &SystemCaptureCredentialsV2)),
        "capture_credential_source_invalid"
    );
    r.selection.usage = UsageV2::RemoteSync;
    assert_eq!(code(capture(&d, &h, &r, &NoCredentials)), "sync_self_scope");
    assert!(path.is_file());
}
#[cfg(windows)]
#[test]
fn windows_system_webdav_refuses_plain_fixture_instead_of_bypassing_dpapi() {
    // Run on an actual Windows target. Production decode must require DPAPI;
    // its private encode helper is not duplicated or exposed by this module.
    let (_t, d, _, _) = fixture();
    saved_webdav(&d);
    credential_fixture(&d, &dav_account(), "synthetic-plain-not-dpapi", "plain");
    assert_eq!(
        code(SystemCaptureCredentialsV2.webdav_password(&d, DAV_ID)),
        "capture_credential_source_invalid"
    );
}
