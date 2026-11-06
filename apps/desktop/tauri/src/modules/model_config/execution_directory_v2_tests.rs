use super::*;
use crate::{
    infrastructure::runtime_executable::FixedRuntimeExecutable,
    modules::model_config::{
        compiler_v2,
        protocol_groups::RuntimeMappingV2,
        source_v2::{plan_materialization_v2, source_from_provider_v2, ProtocolAuthBindingV2},
        store_v2::{
            LedgerFileV2, DRAFT_PATH, LEDGER_PATH, LIBRARY_PATH, MAPPING_PATH, POLICY_PATH,
        },
    },
};
use serde_json::json;
use std::path::PathBuf;
const CHANNEL: &str = "11111111-1111-4111-8111-111111111111";
const NODE: &str = "22222222-2222-4222-8222-222222222222";
struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    context: ProcessContext,
    library: LibraryV2,
    policy: PolicyV2,
    mappings: MappingFileV2,
    raw: Value,
    live: UpstreamSnapshot,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("D");
        let home = temp.path().join("H");
        std::fs::create_dir_all(root.join("manager-state")).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        let executable = temp.path().join("fixture-ocx");
        std::fs::write(&executable, b"fixture").unwrap();
        let context = ProcessContext {
            runtime: FixedRuntimeExecutable::resolved(executable),
            working_directory: temp.path().to_owned(),
            opencodex_home: home,
        };
        let library = seed_library_v2().unwrap();
        let model = ModelConfigV2 {
            raw_model_id: "model/a".into(),
            label: "model/a".into(),
            observed: true,
            source: source_from_provider_v2(
                "primary",
                "model/a",
                &json!({"adapter":"openai-chat"}),
                &digest(b"source"),
                SourceKindV2::ImportedFile,
            )
            .unwrap(),
            explicit_overrides: super::super::schema::ModelRules {
                protocol: Field::Custom("openai-responses".into()),
                ..Default::default()
            },
            binding: None,
        };
        let policy = PolicyV2 {
            schema_version: 2,
            channels: vec![ChannelV2 {
                id: CHANNEL.into(),
                label: "真实渠道".into(),
                connection: ConnectionV2 {
                    provider_id: "primary".into(),
                    nodes: vec![super::super::schema::ServiceNode {
                        id: NODE.into(),
                        name: "node".into(),
                        url: "https://fixture.invalid/v1".into(),
                    }],
                    active_node_id: NODE.into(),
                    credential_ref: None,
                    enabled: true,
                    allow_private_network: false,
                    auth_mode: Some("key".into()),
                    source_read_id: None,
                    imported_config: None,
                },
                selection: super::super::schema::ModelSelection::Whitelist(vec!["model/a".into()]),
                model_order: vec!["model/a".into()],
                models: vec![model],
                template_refs: vec![],
                defaults: ModelRules::default(),
                family_overrides: BTreeMap::new(),
                naming: BTreeMap::new(),
                namespace: None,
            }],
            template_refs: vec![],
            default_channel_id: None,
            pending_removals: vec![],
            recovery_id: None,
        };
        let group = plan_materialization_v2(&library, &policy, &policy)
            .unwrap()
            .groups
            .remove(0);
        let mappings = MappingFileV2 {
            schema_version: 2,
            mappings: vec![RuntimeMappingV2 {
                channel_id: CHANNEL.into(),
                protocol: group.protocol.clone(),
                wire_profile_digest: group.wire_profile_digest.clone(),
                provider_id: "responses-group".into(),
                anchor: false,
            }],
        };
        let raw = json!({"providers":{"primary":{"adapter":"openai-chat","baseUrl":"https://fixture.invalid/v1","authMode":"key","apiKey":"DO_NOT_LEAK_CAPTURE_SECRET","models":["model/a"]}},"modelPickerOrder":["responses-group/model/a"]});
        let auth = vec![ProtocolAuthBindingV2 {
            channel_id: CHANNEL.into(),
            allowed_protocols: [ProtocolV2::OpenaiChat, ProtocolV2::OpenaiResponses].into(),
        }];
        let compiled =
            compiler_v2::compile_file_v2(&library, &policy, &policy, &raw, &mappings, &auth)
                .unwrap();
        // A parked primary identity stays present but owns no duplicate live row.
        let mut raw = compiled.next_config;
        raw["providers"]["primary"]["models"] = json!([]);
        raw["providers"]["primary"]["selectedModels"] = json!([]);
        raw["providers"]["primary"]["disabled"] = json!(true);
        let mut public = raw.clone();
        for row in public["providers"].as_object_mut().unwrap().values_mut() {
            row.as_object_mut().unwrap().remove("apiKey");
        }
        let live = UpstreamSnapshot {
            version: "2.97.0-fixture".into(),
            providers: public,
            models: json!([{"provider":"responses-group","id":"model/a","namespaced":"actual-route/selector-with-slash","reasoningEfforts":["none","low"],"defaultReasoningEffort":"none","fastRowAvailable":true,"contextWindow":4096}]),
            order: json!({"pickerOrder":["actual-route/selector-with-slash"]}),
            catalog: json!({}),
        };
        // Generate ledger against the final actual JSON bytes, not a draft.
        let ledger = super::super::source_v2::finalize_materialization_ledger_v2(
            &policy,
            &raw,
            &compiled.targets,
        )
        .unwrap();
        for (path, bytes) in [
            (LIBRARY_PATH, canonical_bytes(&library).unwrap()),
            (POLICY_PATH, canonical_bytes(&policy).unwrap()),
            (
                LEDGER_PATH,
                canonical_bytes(&LedgerFileV2 {
                    schema_version: 2,
                    ledger,
                })
                .unwrap(),
            ),
            (MAPPING_PATH, canonical_bytes(&mappings).unwrap()),
        ] {
            std::fs::write(root.join(path), bytes).unwrap();
        }
        std::fs::write(
            context.opencodex_home.join("config.json"),
            canonical_bytes(&raw).unwrap(),
        )
        .unwrap();
        Self {
            _temp: temp,
            root,
            context,
            library,
            policy,
            mappings,
            raw,
            live,
        }
    }
    fn saved(&self) -> SavedExecutionV2 {
        from_saved_facts(
            &self.library,
            &self.policy,
            &self.mappings,
            &self.raw,
            &digest(b"source-file"),
            &digest(b"root"),
            &super::super::runtime::runtime_binding(&self.context).unwrap(),
        )
        .unwrap()
    }
    fn settings(&self) -> AgentSettings {
        AgentSettings {
            provider_id: "responses-group".into(),
            model_id: "model/a".into(),
            reasoning_effort: Some("none".into()),
            fast: Some(true),
            channel_id: None,
            connection_digest: None,
        }
    }
}
#[test]
fn nonanchor_catalog_joins_logical_uuid_and_never_constructs_a_selector() {
    let f = Fixture::new();
    let directory = build(&f.saved(), &f.live).unwrap();
    assert_eq!(directory.rows.len(), 1);
    let row = &directory.rows[0];
    assert_eq!(row.channel_id, CHANNEL);
    assert_eq!(row.label, "真实渠道/model/a");
    assert_eq!(
        row.actual_selector.as_deref(),
        Some("actual-route/selector-with-slash")
    );
    assert!(row.picker_visible);
    assert!(row.connection_digest.is_some());
    assert!(!serde_json::to_string(&directory)
        .unwrap()
        .contains("DO_NOT_LEAK_CAPTURE_SECRET"));
    let bound = bind(&f.saved(), &f.live, f.settings()).unwrap();
    assert_eq!(bound.selector(), "actual-route/selector-with-slash");
    assert_eq!(bound.settings().channel_id.as_deref(), Some(CHANNEL));
    validate_binding(&f.context, &f.saved(), &f.live, &bound).unwrap();
}
#[test]
fn direction_root_source_protocol_and_selection_changes_refuse_the_cached_binding() {
    let f = Fixture::new();
    let saved = f.saved();
    let bound = bind(&saved, &f.live, f.settings()).unwrap();
    let mut changed = f.live.clone();
    changed.providers["providers"]["responses-group"]["baseUrl"] = json!("https://changed.invalid");
    assert!(validate_binding(&f.context, &saved, &changed, &bound).is_err());
    let mut changed = f.saved();
    changed.source_digest = digest(b"changed-source");
    assert!(validate_binding(&f.context, &changed, &f.live, &bound).is_err());
    let mut changed = f.saved();
    changed.root_revision = digest(b"other-root");
    assert!(validate_binding(&f.context, &changed, &f.live, &bound).is_err());
    let mut changed = f.saved();
    changed.policy.channels[0].selection = super::super::schema::ModelSelection::None;
    assert!(validate_binding(&f.context, &changed, &f.live, &bound).is_err());
    let mut changed = f.live.clone();
    changed.providers["providers"]["responses-group"]["adapter"] = json!("openai-chat");
    assert!(bind(&saved, &changed, f.settings()).is_err());
    let mut settings = f.settings();
    settings.channel_id = Some(NODE.into());
    assert!(bind(&saved, &f.live, settings).is_err());
}
#[test]
fn actual_capabilities_and_saved_denials_do_not_invent_fast_or_reasoning() {
    let f = Fixture::new();
    let mut saved = f.saved();
    saved.policy.channels[0].models[0].explicit_overrides.fast = Field::Custom(false);
    let directory = build(&saved, &f.live).unwrap();
    assert_eq!(directory.rows[0].rules.fast, Field::Custom(false));
    assert_eq!(
        bind(&saved, &f.live, f.settings()).unwrap_err().code,
        "model_execution_fast_unavailable"
    );
    let mut settings = f.settings();
    settings.fast = None;
    settings.reasoning_effort = Some("max".into());
    assert_eq!(
        bind(&f.saved(), &f.live, settings).unwrap_err().code,
        "model_execution_reasoning_unavailable"
    );
    let mut hidden = f.live.clone();
    hidden.order = json!({"pickerOrder":[]});
    assert!(!build(&f.saved(), &hidden).unwrap().rows[0].picker_visible);
}
#[test]
fn missing_or_native_rows_remain_explanatory_and_never_become_executable() {
    let f = Fixture::new();
    let mut missing = f.live.clone();
    missing.models = json!([]);
    let directory = build(&f.saved(), &missing).unwrap();
    assert_eq!(
        directory.rows[0].unavailable_reason.as_deref(),
        Some("model_execution_not_live")
    );
    assert!(bind(&f.saved(), &missing, f.settings()).is_err());
    let mut native = missing;
    native.models =
        json!([{"provider":"openai","id":"native-a","native":true,"namespaced":"native-a"}]);
    let directory = build(&f.saved(), &native).unwrap();
    assert!(directory
        .rows
        .iter()
        .any(|r| r.native_model_id.is_some() && !r.picker_visible));
    let mut duplicate = f.live.clone();
    duplicate
        .models
        .as_array_mut()
        .unwrap()
        .push(f.live.models[0].clone());
    assert_eq!(
        build(&f.saved(), &duplicate).unwrap_err().code,
        "model_execution_identity_collision"
    );
}
#[test]
fn production_saved_capture_ignores_bad_draft_and_releases_locks_before_runtime_work() {
    let f = Fixture::new();
    std::fs::write(f.root.join(DRAFT_PATH), b"{bad draft").unwrap();
    let source = std::fs::read(f.context.opencodex_home.join("config.json")).unwrap();
    let saved = capture_saved(&f.root, &f.context).unwrap();
    assert_eq!(saved.policy.channels.len(), 1);
    assert_eq!(saved.policy.channels[0].connection.enabled, true);
    assert!(!format!("{saved:?}").contains("DO_NOT_LEAK_CAPTURE_SECRET"));
    assert_eq!(
        source,
        std::fs::read(f.context.opencodex_home.join("config.json")).unwrap()
    );
    let config_lock = crate::infrastructure::locking::TargetFileLock::try_lock_with_timeout(
        &f.context.opencodex_home.join("config.json"),
        std::time::Duration::ZERO,
    )
    .unwrap();
    drop(config_lock);
    std::fs::write(
        f.root
            .join("manager-state/model-transaction-v2.pending.json"),
        b"pending must not recover",
    )
    .unwrap();
    assert_eq!(
        capture_saved(&f.root, &f.context).unwrap_err().code,
        "model_transaction_pending"
    );
    assert_eq!(
        std::fs::read(
            f.root
                .join("manager-state/model-transaction-v2.pending.json")
        )
        .unwrap(),
        b"pending must not recover"
    );
}
#[cfg(unix)]
#[tokio::test]
async fn actual_snapshot_runs_only_the_temporary_cli_and_binds_its_returned_selector() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    let parent = f.context.working_directory.clone();
    for (name, value) in [
        ("providers", f.live.providers.clone()),
        ("models", f.live.models.clone()),
        ("order", f.live.order.clone()),
        ("catalog", f.live.catalog.clone()),
    ] {
        std::fs::write(
            parent.join(format!("{name}.json")),
            canonical_bytes(&value).unwrap(),
        )
        .unwrap();
    }
    let script=format!("#!/bin/sh\ncase \"$1 $2\" in\n'--version ') echo '2.97.0-fixture';;\n'provider snapshot') exec /bin/cat '{}';;\n'models live') exec /bin/cat '{}';;\n'models order') exec /bin/cat '{}';;\n'inspect catalog') exec /bin/cat '{}';;\n*) exit 9;;\nesac\n",parent.join("providers.json").display(),parent.join("models.json").display(),parent.join("order.json").display(),parent.join("catalog.json").display());
    let executable = f.context.executable().unwrap();
    std::fs::write(&executable, script).unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
    let saved = capture_saved(&f.root, &f.context).unwrap();
    let live = super::super::upstream::snapshot(&f.context).await.unwrap();
    let bound = bind(&saved, &live, f.settings()).unwrap();
    assert_eq!(bound.selector(), "actual-route/selector-with-slash");
    validate_binding(
        &f.context,
        &capture_saved(&f.root, &f.context).unwrap(),
        &live,
        &bound,
    )
    .unwrap();
}
