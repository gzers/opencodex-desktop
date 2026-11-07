//! Actual LocalApplication + ReceiveFileHost + typed transaction + manager/assets.
//! Only interruption points are injected; Applied always comes from the real receipt.
use super::*;
use crate::modules::model_config::source_v2::source_from_provider_v2;
use crate::modules::{
    model_config::{
        config_file::v2::receive::ReceiveFileHostV2,
        file_objects_v2::{CapturedObjects, SecretValues},
        objects_overlay_v2::{ChannelSelectionV2, ManagerSelectionV2, McpSourceSelectionV2},
        portable_v2::{ModelPayloadV2, PortableConnectionV2, PortableModelsV2, PortablePolicyV2},
        receive_models_v2::{ReceiveBindingModeV2, ReceiveBindingV2},
        schema::{Field, ModelRules, ModelSelection, ServiceNode},
        schema_v2::*,
    },
    preferences::{self, Preferences},
};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
const OP: &str = "99999999-9999-4999-8999-999999999999";
const SOURCE: &str = "11111111-1111-4111-8111-111111111111";
const LOCAL: &str = "22222222-2222-4222-8222-222222222222";
const TEMPLATE: &str = "44444444-4444-4444-8444-444444444444";
const REVISION: &str = "55555555-5555-4555-8555-555555555555";
struct ActualModels {
    receiver: ReceiveFileHostV2,
    interrupt_after_real_model: AtomicBool,
}
impl ModelApplyV2 for ActualModels {
    fn prepare(
        &self,
        id: &str,
        objects: &SavedObjects,
        s: &SelectionV2,
        target: &str,
    ) -> ModelResult<ModelApplyPlanHandle> {
        self.receiver.prepare(id, objects, s, target)
    }
    fn protect(&self, id: &str, h: &ModelApplyPlanHandle, backup: bool) -> ModelResult<()> {
        self.receiver.protect(id, h, backup)
    }
    fn apply_once(
        &self,
        id: &str,
        o: &SavedObjects,
        h: &ModelApplyPlanHandle,
        b: bool,
    ) -> ModelResult<ModelApplyReceipt> {
        let actual = self.receiver.apply_once(id, o, h, b)?;
        if self
            .interrupt_after_real_model
            .swap(false, Ordering::SeqCst)
        {
            return Err(invalid(
                "scope_test_after_real_model",
                "injected only after actual model receipt",
            ));
        }
        Ok(actual)
    }
    fn status(&self, id: &str, h: &ModelApplyPlanHandle) -> ModelResult<ModelApplyStatus> {
        self.receiver.status(id, h)
    }
    fn scope_backup_images(
        &self,
        id: &str,
        h: &ModelApplyPlanHandle,
    ) -> ModelResult<Option<Vec<WriteV2>>> {
        self.receiver.protected_images(id, h).map(Some)
    }
}
struct ActualTarget {
    scope: String,
}
impl SourceSnapshotProviderV2 for ActualTarget {
    fn current(&self, d: &Path, h: &Path, _: &CaptureRequestV2) -> ModelResult<SourceGuard> {
        Ok(SourceGuard {
            saved_source_digest: self.target(
                d,
                h,
                &CaptureRequestV2 {
                    selection: Default::default(),
                    mcp_clients: Default::default(),
                    assets_reviewed: false,
                },
            )?,
            scope_digest: self.scope.clone(),
            mcp_sources_digest: hash(b"no captured MCP authority"),
        })
    }
    fn root(&self, d: &Path, h: &Path, _: &CaptureRequestV2) -> ModelResult<String> {
        Ok(hash(&encode(&(d, h))?))
    }
    fn target(&self, d: &Path, h: &Path, _: &CaptureRequestV2) -> ModelResult<String> {
        super::digest(&(
            images::image(h, &h.join("config.json"))?,
            images::image(d, &d.join(preferences::PREFERENCES_RELATIVE_PATH))?,
        ))
    }
}
struct Fixture {
    _temp: tempfile::TempDir,
    d: PathBuf,
    h: PathBuf,
    selection: SelectionV2,
    payload: Zeroizing<Vec<u8>>,
    scope: String,
    expected: String,
    models: Arc<ActualModels>,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::Builder::new()
            .prefix("ocxd-l2-v2-scope-restore-")
            .tempdir_in(crate::modules::test_sandbox::temporary_parent())
            .unwrap();
        let d = temp.path().join("D");
        let h = temp.path().join("H");
        std::fs::create_dir_all(d.join("manager-state")).unwrap();
        std::fs::create_dir_all(&h).unwrap();
        let prefs = preferences::document_from_preferences(&Preferences::default());
        atomic_write(
            &d.join(preferences::PREFERENCES_RELATIVE_PATH),
            &encode(&prefs).unwrap(),
            0o600,
        )
        .unwrap();
        let mut template = seed_library_v2().unwrap().templates.remove(0);
        template.id = TEMPLATE.into();
        let mut revision = template.revisions.remove(0);
        revision.id = REVISION.into();
        revision.base_revision = None;
        revision.content.common_rules = ModelRules {
            protocol: Field::Custom("openai-chat".into()),
            context_window: Field::Custom(8192),
            ..Default::default()
        };
        revision.content.condition_overrides.clear();
        revision.content.locked_fields.clear();
        revision.digest = content_digest(&revision.saved_label, &revision.content).unwrap();
        template.default_revision = Some(revision.id.clone());
        template.revisions = vec![revision];
        let reference = RefV2 {
            template_id: TEMPLATE.into(),
            revision_id: REVISION.into(),
            digest: template.revisions[0].digest.clone(),
        };
        let channel = PortableChannelV2 {
            id: SOURCE.into(),
            label: "Actual restore channel".into(),
            connection: Some(PortableConnectionV2 {
                nodes: vec![ServiceNode {
                    id: "33333333-3333-4333-8333-333333333333".into(),
                    name: "Synthetic".into(),
                    url: "https://scope.invalid/v1".into(),
                }],
                active_node_id: "33333333-3333-4333-8333-333333333333".into(),
                enabled: true,
                allow_private_network: false,
                auth_mode: Some("none".into()),
                provider: Some(
                    json!({"adapter":"openai-chat","baseUrl":"https://scope.invalid/v1","models":["fixture"]}),
                ),
            }),
            model_subset: None,
            family_scope: None,
            models: vec![ModelConfigV2 {
                raw_model_id: "fixture".into(),
                label: "Fixture".into(),
                observed: true,
                source: source_from_provider_v2(
                    "source",
                    "fixture",
                    &json!({"adapter":"openai-chat"}),
                    &hash(b"synthetic source"),
                    SourceKindV2::UpstreamRead,
                )
                .unwrap(),
                explicit_overrides: ModelRules::default(),
                binding: Some(ModelBindingV2::Manual {
                    reference: reference.clone(),
                }),
            }],
            template_refs: vec![reference.clone()],
            selection: ModelSelection::All,
            model_order: vec!["fixture".into()],
            defaults: ModelRules::default(),
            family_overrides: BTreeMap::new(),
            naming: BTreeMap::new(),
            namespace: None,
        };
        let payload = ModelPayloadV2 {
            templates: vec![template],
            policy: Some(PortablePolicyV2 {
                template_refs: vec![reference.clone()],
                channels: vec![channel],
                channel_order: vec![SOURCE.into()],
                default_channel_id: None,
            }),
        };
        let source = temp.path().join("incoming-skill");
        std::fs::create_dir(&source).unwrap();
        std::fs::write(
            source.join("SKILL.md"),
            b"# Scope fixture\nNew instruction.\n",
        )
        .unwrap();
        let old = h.join(".agents/skills/scope-fixture");
        std::fs::create_dir_all(&old).unwrap();
        std::fs::write(
            old.join("SKILL.md"),
            b"# Scope fixture\nOriginal instruction.\n",
        )
        .unwrap();
        let skill = portable_assets::snapshot_skill(&source, "scope-fixture", false, true).unwrap();
        let mcp = portable_assets::split_mcp(
            crate::modules::extensions::discovery::RawServerDefinition {
                name: "scope_fixture".into(),
                value: json!({"command":"synthetic-scope-command"}),
            },
            false,
            false,
        )
        .unwrap();
        let assets = portable_assets::Assets {
            skills: vec![skill],
            mcps: vec![mcp],
        };
        let mcp_digest = objects_overlay_v2::mcp_source_digest(&assets.mcps[0]).unwrap();
        let settings = portable_manager::Settings {
            preferences: Some(Preferences {
                theme: "dark".into(),
                ..Default::default()
            }),
            extensions: Some(crate::modules::extensions::ExtensionConfig::default()),
            accounts: vec![crate::modules::sync::portable_accounts::AccountInfo {
                id: "66666666-6666-4666-8666-666666666666".into(),
                provider: "chatgpt".into(),
                subject: Some("synthetic-account-subject".into()),
                label: "Synthetic account".into(),
                email: None,
                origin: None,
            }],
            webdav: vec![portable_manager::Endpoint {
                id: "synthetic-connection".into(),
                url: "https://backup.invalid".into(),
                remote_path: "isolated".into(),
                username: "synthetic".into(),
                conflict_policy: "ask".into(),
            }],
        };
        let saved = SavedObjects::capture(
            CapturedObjects {
                models: PortableModelsV2 {
                    format: "OCXD_MODEL_PACKAGE".into(),
                    format_version: 6,
                    schema_version: 2,
                    created_at: "2026-11-07T00:00:00Z".into(),
                    digest: schema_v2::digest(&canonical_bytes(&payload).unwrap()),
                    payload,
                },
                manager: settings,
                assets,
                channel_keys: SecretValues::new(),
                channel_authorizations: BTreeMap::new(),
                webdav_passwords: SecretValues::new(),
                content_versions: vec![],
            },
            Some(&ActualAuthorization),
        )
        .unwrap();
        let selection = SelectionV2 {
            usage: UsageV2::LocalFile,
            channels: vec![ChannelSelectionV2 {
                portable_channel_id: SOURCE.into(),
                model_ids: None,
                family_ids: None,
                include_connection: true,
                include_key: false,
                include_authorization: false,
            }],
            template_refs: vec![reference],
            manager: ManagerSelectionV2 {
                preferences: true,
                extensions: true,
                account_ids: vec!["66666666-6666-4666-8666-666666666666".into()],
                webdav_ids: vec!["synthetic-connection".into()],
                webdav_password_ids: vec![],
            },
            skill_names: vec!["scope-fixture".into()],
            mcp_sources: vec![McpSourceSelectionV2 {
                name: "scope_fixture".into(),
                definition_digest: mcp_digest,
            }],
            ..Default::default()
        };
        let scope = super::digest(&selection).unwrap();
        let capture = CaptureRequestV2 {
            selection: selection.clone(),
            mcp_clients: BTreeMap::new(),
            assets_reviewed: true,
        };
        let expected = ActualTarget {
            scope: scope.clone(),
        }
        .target(&d, &h, &capture)
        .unwrap();
        let payload = file_objects_v2::encode(
            &saved,
            OP,
            "2026-11-07T00:00:00Z",
            None,
            Some(&ActualAuthorization),
        )
        .unwrap()
        .into_bytes();
        let models = Arc::new(ActualModels {
            receiver: ReceiveFileHostV2 {
                root: d.clone(),
                home: h.clone(),
                bindings: vec![ReceiveBindingV2 {
                    portable_channel_id: SOURCE.into(),
                    local_channel_id: LOCAL.into(),
                    provider_id: "scope_received".into(),
                    mode: ReceiveBindingModeV2::New,
                }],
                trusted_mapping_write: None,
            },
            interrupt_after_real_model: AtomicBool::new(false),
        });
        Self {
            _temp: temp,
            d,
            h,
            selection,
            payload,
            scope,
            expected,
            models,
        }
    }
    fn host(&self) -> LocalApplicationV2 {
        LocalApplicationV2::new(
            self.d.clone(),
            self.h.clone(),
            self.selection.clone(),
            CaptureRequestV2 {
                selection: self.selection.clone(),
                mcp_clients: BTreeMap::new(),
                assets_reviewed: true,
            },
            self.scope.clone(),
            Arc::new(ActualTarget {
                scope: self.scope.clone(),
            }),
            self.models.clone(),
            None,
        )
        .unwrap()
    }
    fn apply(&self) -> SyncResult<()> {
        self.host().apply_once(
            OP,
            &self.payload,
            &file_objects_v2::supported_schema(),
            &self.scope,
            &self.expected,
            true,
        )
    }
    fn all(&self) -> Vec<String> {
        list(&self.d, &self.h).unwrap()[0].groups.clone()
    }
}
fn err<T>(result: ModelResult<T>) -> String {
    match result {
        Err(e) => e.code,
        Ok(_) => panic!("expected explicit refusal"),
    }
}
#[test]
fn original_actual_receive_before_after_restore_all_and_repeat_receipt_preserves_later_edits() {
    let f = Fixture::new();
    assert!(!f.h.join("config.json").exists());
    let prefs = std::fs::read(f.d.join(preferences::PREFERENCES_RELATIVE_PATH)).unwrap();
    f.apply().unwrap();
    let status = list(&f.d, &f.h).unwrap();
    assert!(status[0].enabled && status[0].ready);
    assert_eq!(
        status[0].groups,
        vec![
            "accounts",
            "channels",
            "extensions",
            "mcp",
            "preferences",
            "skill:scope-fixture",
            "templates",
            "webdav"
        ]
    );
    assert!(f.h.join("config.json").exists());
    assert!(f
        .d
        .join("manager-state/model-receive-v2")
        .join(OP)
        .join("receipt.json")
        .exists());
    let preview = preview(&f.d, &f.h, OP, &f.all()).unwrap();
    let receipt = restore(&f.d, &f.h, &preview.restore_id, &preview.plan_digest).unwrap();
    assert!(receipt.status == RestoreStatusV2::Applied);
    assert!(!f.h.join("config.json").exists());
    for p in [
        store_v2::LIBRARY_PATH,
        store_v2::POLICY_PATH,
        store_v2::LEDGER_PATH,
        store_v2::MAPPING_PATH,
    ] {
        assert!(!f.d.join(p).exists(), "{p}");
    }
    assert_eq!(
        std::fs::read(f.d.join(preferences::PREFERENCES_RELATIVE_PATH)).unwrap(),
        prefs
    );
    assert_eq!(
        std::fs::read(f.h.join(".agents/skills/scope-fixture/SKILL.md")).unwrap(),
        b"# Scope fixture\nOriginal instruction.\n"
    );
    atomic_write(
        &f.h.join("config.json"),
        br#"{"afterRestore":"user edit"}"#,
        0o600,
    )
    .unwrap();
    let repeated = restore(&f.d, &f.h, &preview.restore_id, &preview.plan_digest).unwrap();
    assert!(repeated.status == RestoreStatusV2::Applied);
    assert_eq!(
        std::fs::read(f.h.join("config.json")).unwrap(),
        br#"{"afterRestore":"user edit"}"#
    );
}
#[test]
fn template_only_dependency_rejected_and_preview_cas_preserves_changed_targets() {
    let f = Fixture::new();
    f.apply().unwrap();
    assert_eq!(
        err(preview(&f.d, &f.h, OP, &["templates".into()])),
        "scope_restore_dependency"
    );
    assert_eq!(
        err(preview(&f.d, &f.h, OP, &["not-a-group".into()])),
        "scope_restore_scope"
    );
    let p = preview(&f.d, &f.h, OP, &["preferences".into()]).unwrap();
    let path = f.d.join(preferences::PREFERENCES_RELATIVE_PATH);
    atomic_write(&path, br#"{"theme":"light","outside":"later edit"}"#, 0o600).unwrap();
    assert_eq!(
        err(restore(&f.d, &f.h, &p.restore_id, &p.plan_digest)),
        "scope_restore_stale"
    );
    assert_eq!(
        std::fs::read(path).unwrap(),
        br#"{"theme":"light","outside":"later edit"}"#
    );
    assert!(f.h.join("config.json").exists());
}
#[test]
fn actual_model_applied_interruption_keeps_not_ready_then_reopens_same_before() {
    let f = Fixture::new();
    f.models
        .interrupt_after_real_model
        .store(true, Ordering::SeqCst);
    assert!(f.apply().is_err());
    let before = std::fs::read(record_path(&f.d, OP).unwrap()).unwrap();
    assert!(!list(&f.d, &f.h).unwrap()[0].ready);
    assert!(f
        .d
        .join("manager-state/model-receive-v2")
        .join(OP)
        .join("receipt.json")
        .exists());
    assert_eq!(
        err(preview(&f.d, &f.h, OP, &["preferences".into()])),
        "scope_restore_not_ready"
    );
    f.apply().unwrap();
    assert!(list(&f.d, &f.h).unwrap()[0].ready);
    let initial: Envelope<serde_json::Value> = serde_json::from_slice(&before).unwrap();
    let final_record: Envelope<serde_json::Value> =
        serde_json::from_slice(&std::fs::read(record_path(&f.d, OP).unwrap()).unwrap()).unwrap();
    assert_eq!(initial.value["rows"], final_record.value["rows"]);
}
#[test]
fn real_unsaved_draft_and_backup_protection_fail_closed_before_business_writes() {
    let f = Fixture::new();
    atomic_write(&f.d.join(store_v2::DRAFT_PATH), b"{}", 0o600).unwrap();
    assert!(f.apply().is_err());
    assert!(!f.h.join("config.json").exists());
    assert!(!f.d.join(DIRECTORY).exists());
    std::fs::remove_file(f.d.join(store_v2::DRAFT_PATH)).unwrap();
    atomic_write(&f.d.join("backups"), b"not a backup directory", 0o600).unwrap();
    assert!(f.apply().is_err());
    assert!(!f.h.join("config.json").exists());
}

#[cfg(unix)]
#[test]
fn real_participant_write_failure_returns_partial_receipt_and_resumes_without_rewriting_completed_groups(
) {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    f.apply().unwrap();
    let groups = vec!["preferences".into(), "skill:scope-fixture".into()];
    let p = preview(&f.d, &f.h, OP, &groups).unwrap();
    let skill = f.h.join(".agents/skills/scope-fixture");
    std::fs::set_permissions(&skill, std::fs::Permissions::from_mode(0o500)).unwrap();
    let result = restore(&f.d, &f.h, &p.restore_id, &p.plan_digest);
    std::fs::set_permissions(&skill, std::fs::Permissions::from_mode(0o700)).unwrap();
    let partial = result.unwrap();
    assert!(partial.status == RestoreStatusV2::Partial);
    assert!(!partial.refresh_file_snapshot);
    assert!(partial.completed_groups.contains(&"preferences".into()));
    assert!(!partial
        .completed_groups
        .contains(&"skill:scope-fixture".into()));
    assert!(partial.failure_code.is_some());
    let partial_progress: Progress =
        read(&plan_path(&f.d, &p.restore_id, "progress").unwrap()).unwrap();
    let frozen_time = partial_progress.clock_time.clone().unwrap();
    assert!(partial_progress.versioned.contains("preferences"));
    assert!(!partial_progress.versioned.contains("skill:scope-fixture"));
    let records: BTreeMap<String, versions::Version> =
        serde_json::from_slice(&ledger_bytes(&f.d).unwrap().unwrap()).unwrap();
    for (key, content) in &partial_progress.deltas["preferences"].after {
        if partial_progress.deltas["preferences"].before.get(key) != Some(content) {
            assert_eq!(records[key].saved_at, frozen_time);
            assert_eq!(records[key].content_digest, *content);
            assert_eq!(
                records[key].last_event.as_deref(),
                Some(p.restore_id.as_str())
            );
        }
    }
    let protected_preferences =
        std::fs::read(f.d.join(preferences::PREFERENCES_RELATIVE_PATH)).unwrap();
    let resumed = restore(&f.d, &f.h, &p.restore_id, &p.plan_digest).unwrap();
    assert!(resumed.status == RestoreStatusV2::Applied);
    assert_eq!(
        std::fs::read(f.d.join(preferences::PREFERENCES_RELATIVE_PATH)).unwrap(),
        protected_preferences
    );
    let final_progress: Progress =
        read(&plan_path(&f.d, &p.restore_id, "progress").unwrap()).unwrap();
    assert_eq!(
        final_progress.clock_time.as_deref(),
        Some(frozen_time.as_str())
    );
    assert!(final_progress.versioned.contains("skill:scope-fixture"));
}
#[test]
fn rewriting_backup_checksum_does_not_authorize_forged_model_or_participant_before_bytes() {
    let f = Fixture::new();
    f.apply().unwrap();
    let path = record_path(&f.d, OP).unwrap();
    let original = std::fs::read(&path).unwrap();
    let mut r: Record = read(&path).unwrap();
    let row = r
        .rows
        .iter_mut()
        .find(|row| row.address.model == Some(TargetV2::OfficialConfig))
        .unwrap();
    row.before = file_image(Some(br#"{"forged":"never restore"}"#));
    save(&f.d, &path, &r).unwrap();
    assert_eq!(
        err(preview(&f.d, &f.h, OP, &["channels".into()])),
        "scope_backup_model_history"
    );
    atomic_write(&path, &original, 0o600).unwrap();
    let mut r: Record = read(&path).unwrap();
    let row = r
        .rows
        .iter_mut()
        .find(|row| row.address.group == "preferences")
        .unwrap();
    row.before = file_image(Some(br#"{"theme":"forged"}"#));
    save(&f.d, &path, &r).unwrap();
    assert_eq!(
        err(preview(&f.d, &f.h, OP, &["preferences".into()])),
        "scope_backup_before_proof"
    );
}

#[test]
fn manager_only_restore_preserves_later_unselected_model_edits_and_restores_original_absent_metadata(
) {
    let f = Fixture::new();
    f.apply().unwrap();
    let unselected = br#"{"providers":{},"laterUnselectedModelEdit":true}"#;
    atomic_write(&f.h.join("config.json"), unselected, 0o600).unwrap();
    let groups = vec![
        "preferences".into(),
        "extensions".into(),
        "accounts".into(),
        "webdav".into(),
    ];
    let p = preview(&f.d, &f.h, OP, &groups).unwrap();
    assert_eq!(
        err(restore(&f.d, &f.h, &p.restore_id, &hash(b"wrong plan"))),
        "scope_restore_plan"
    );
    let receipt = restore(&f.d, &f.h, &p.restore_id, &p.plan_digest).unwrap();
    assert!(receipt.status == RestoreStatusV2::Applied);
    for path in [
        crate::modules::extensions::projection::CONFIG_RELATIVE_PATH,
        crate::modules::sync::portable_accounts::IMPORTED,
        portable_manager::CANDIDATES,
    ] {
        assert!(!f.d.join(path).exists(), "{path}");
    }
    assert_eq!(std::fs::read(f.h.join("config.json")).unwrap(), unselected);
}
#[test]
fn backup_false_is_listed_disabled_without_private_scope_images() {
    let f = Fixture::new();
    f.host()
        .apply_once(
            OP,
            &f.payload,
            &file_objects_v2::supported_schema(),
            &f.scope,
            &f.expected,
            false,
        )
        .unwrap();
    let statuses = list(&f.d, &f.h).unwrap();
    assert!(!statuses[0].enabled && !statuses[0].ready);
    let r: Record = read(&record_path(&f.d, OP).unwrap()).unwrap();
    assert!(r.rows.is_empty() && r.after.is_none());
    assert_eq!(
        err(preview(&f.d, &f.h, OP, &["preferences".into()])),
        "scope_restore_not_ready"
    );
}

#[test]
fn actual_receiver_and_namespace_production_traits_issue_ready_restorable_backups() {
    for namespace in [false, true] {
        let f = Fixture::new();
        let receiver = ReceiveFileHostV2 {
            root: f.d.clone(),
            home: f.h.clone(),
            bindings: f.models.receiver.bindings.clone(),
            trusted_mapping_write: None,
        };
        let models: Arc<dyn ModelApplyV2> = if namespace {
            Arc::new(receiver.with_namespace_adoption(vec![]).unwrap())
        } else {
            Arc::new(receiver)
        };
        let host = LocalApplicationV2::new(
            f.d.clone(),
            f.h.clone(),
            f.selection.clone(),
            CaptureRequestV2 {
                selection: f.selection.clone(),
                mcp_clients: BTreeMap::new(),
                assets_reviewed: true,
            },
            f.scope.clone(),
            Arc::new(ActualTarget {
                scope: f.scope.clone(),
            }),
            models,
            None,
        )
        .unwrap();
        host.apply_once(
            OP,
            &f.payload,
            &file_objects_v2::supported_schema(),
            &f.scope,
            &f.expected,
            true,
        )
        .unwrap();
        assert!(list(&f.d, &f.h).unwrap()[0].ready);
        let p = preview(&f.d, &f.h, OP, &f.all()).unwrap();
        assert!(
            restore(&f.d, &f.h, &p.restore_id, &p.plan_digest)
                .unwrap()
                .status
                == RestoreStatusV2::Applied
        );
    }
}
struct NeverCaptureSecrets;
impl CaptureCredentialsV2 for NeverCaptureSecrets {
    fn channel_key(
        &self,
        _: &Path,
        _: &saved_objects_capture_v2::SelectedKeySourceV2<'_>,
    ) -> ModelResult<Zeroizing<String>> {
        panic!("unselected capture secret")
    }
}
#[test]
fn production_service_backend_factory_mapping_receipt_protects_actual_scope_without_network() {
    use crate::modules::model_config::sync_service_v2::{
        PreviewRequestV2, ProductionServiceBackendV2, ServiceBackendV2, ServiceDirectionV2,
    };
    let f = Fixture::new();
    let mut parts = file_objects_v2::decode(
        &f.payload,
        None,
        &file_objects_v2::supported_schema(),
        Some(&ActualAuthorization),
    )
    .unwrap()
    .objects
    .into_captured();
    parts.manager.webdav.clear();
    let cloud = SavedObjects::capture(parts, Some(&ActualAuthorization)).unwrap();
    let mut selection = f.selection.clone();
    selection.usage = UsageV2::RemoteSync;
    selection.manager.webdav_ids.clear();
    let request = PreviewRequestV2 {
        selection,
        mcp_clients: BTreeMap::new(),
        assets_reviewed: true,
        selected_head: None,
        forced_direction: Some(ServiceDirectionV2::Download),
        backup_before_update: true,
    };
    let backend =
        ProductionServiceBackendV2::new(f.d.clone(), f.h.clone(), Arc::new(NeverCaptureSecrets))
            .unwrap();
    let connection = ConnectionHandle {
        id: "synthetic-host-endpoint".into(),
        binding_id: hash(b"isolated endpoint namespace"),
        revision: hash(b"isolated revision"),
    };
    let composition = backend
        .compose(&request, &connection, Some(&cloud), None)
        .unwrap();
    let guard = composition.local.current_source().unwrap();
    let target = composition
        .local
        .target_digest(&guard.scope_digest)
        .unwrap();
    let payload = file_objects_v2::encode(
        &cloud,
        OP,
        "2026-11-07T00:00:00Z",
        None,
        Some(&ActualAuthorization),
    )
    .unwrap()
    .into_bytes();
    composition
        .local
        .apply_once(
            OP,
            &payload,
            &file_objects_v2::supported_schema(),
            &guard.scope_digest,
            &target,
            true,
        )
        .unwrap();
    assert!(list(&f.d, &f.h).unwrap()[0].ready);
    assert!(f
        .d
        .join("manager-state/sync-identity-mappings-v2.json")
        .exists());
    let p = preview(&f.d, &f.h, OP, &list(&f.d, &f.h).unwrap()[0].groups).unwrap();
    assert!(
        restore(&f.d, &f.h, &p.restore_id, &p.plan_digest)
            .unwrap()
            .status
            == RestoreStatusV2::Applied
    );
    assert!(!f.h.join("config.json").exists());
}

#[test]
fn readonly_preview_ignores_unknown_draft_and_same_plan_restores_only_after_explicit_cas_discard() {
    use crate::modules::model_config::config_file::v2;
    let f = Fixture::new();
    f.apply().unwrap();
    let draft_path = f.d.join(store_v2::DRAFT_PATH);
    let unknown = br#"{"schemaVersion":999,"unsaved":"unknown draft generation"}"#;
    atomic_write(&draft_path, unknown, 0o600).unwrap();
    let before = f
        .all()
        .iter()
        .filter(|g| g.as_str() == "channels")
        .cloned()
        .collect::<Vec<_>>();
    let raw = std::fs::read(f.h.join("config.json")).unwrap();
    let p = preview(&f.d, &f.h, OP, &before).unwrap();
    assert_eq!(std::fs::read(&draft_path).unwrap(), unknown);
    assert_eq!(
        err(restore(&f.d, &f.h, &p.restore_id, &p.plan_digest)),
        "scope_restore_unsaved_draft"
    );
    assert_eq!(std::fs::read(f.h.join("config.json")).unwrap(), raw);
    assert_eq!(std::fs::read(&draft_path).unwrap(), unknown);
    // Explicit fixture actor removes only its own deliberately unknown draft;
    // backend preview/restore never parses or removes that file.
    std::fs::remove_file(&draft_path).unwrap();
    let saved = v2::snapshot(&f.d, &f.h).unwrap();
    let clean = v2::begin_draft(
        &f.d,
        &f.h,
        v2::ReadRequestV2 {
            expected_source: saved.source_digest,
            expected_library: saved.library_digest,
            expected_draft: None,
        },
    )
    .unwrap();
    let draft = clean.draft.unwrap();
    let draft_bytes = std::fs::read(&draft_path).unwrap();
    let clean_plan = preview(&f.d, &f.h, OP, &before).unwrap();
    assert_eq!(std::fs::read(&draft_path).unwrap(), draft_bytes);
    assert_eq!(
        err(restore(
            &f.d,
            &f.h,
            &clean_plan.restore_id,
            &clean_plan.plan_digest
        )),
        "scope_restore_unsaved_draft"
    );
    v2::discard(&f.d, &f.h, draft.base).unwrap();
    assert!(!draft_path.exists());
    let result = restore(&f.d, &f.h, &p.restore_id, &p.plan_digest).unwrap();
    assert!(result.status == RestoreStatusV2::Applied);
    assert!(!f.h.join("config.json").exists());
}

#[cfg(unix)]
#[test]
fn owned_operations_reopen_partial_identity_and_keep_completed_receipt_without_current_config() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    f.apply().unwrap();
    let p = preview(
        &f.d,
        &f.h,
        OP,
        &["preferences".into(), "skill:scope-fixture".into()],
    )
    .unwrap();
    let directory = f.h.join(".agents/skills/scope-fixture");
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o500)).unwrap();
    let partial = restore(&f.d, &f.h, &p.restore_id, &p.plan_digest);
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(partial.unwrap().status == RestoreStatusV2::Partial);
    let reopened = operations(&f.d, &f.h, Some(OP)).unwrap();
    assert_eq!(reopened.len(), 1);
    assert_eq!(reopened[0].restore_id, p.restore_id);
    assert_eq!(reopened[0].plan_digest, p.plan_digest);
    assert!(reopened[0].receipt.as_ref().unwrap().status == RestoreStatusV2::Partial);
    let actual = restore(
        &f.d,
        &f.h,
        &reopened[0].restore_id,
        &reopened[0].plan_digest,
    )
    .unwrap();
    assert!(actual.status == RestoreStatusV2::Applied);
    atomic_write(
        &f.h.join("config.json"),
        b"broken later model config",
        0o600,
    )
    .unwrap();
    atomic_write(
        &f.d.join(crate::modules::extensions::projection::CONFIG_RELATIVE_PATH),
        b"broken later asset source config",
        0o600,
    )
    .unwrap();
    let history = operations(&f.d, &f.h, None).unwrap();
    assert!(history[0].receipt.as_ref().unwrap().status == RestoreStatusV2::Applied);
    assert!(
        operations(&f.d, &f.h, Some("88888888-8888-4888-8888-888888888888"))
            .unwrap()
            .is_empty()
    );
    let path = plan_path(&f.d, &p.restore_id, "progress").unwrap();
    atomic_write(&path, b"broken owned receipt", 0o600).unwrap();
    assert!(operations(&f.d, &f.h, None).is_err());
}
#[test]
fn restoring_existing_model_content_is_a_new_frozen_save_and_actual_capture_uploads_over_old_cloud()
{
    use crate::modules::model_config::{config_file::v2, content_versions, sync_host_v2};
    const SECOND: &str = "88888888-8888-4888-8888-888888888888";
    let f = Fixture::new();
    f.apply().unwrap();
    let state = v2::with_saved_config(&f.d, &f.h, |library, policy, _, raw, _, _| {
        v2::saved_state(library, policy, raw)
    })
    .unwrap();
    // Exact content-bound prior save receipts; no mtime or network clock.
    let old = content_versions::plan_save_at(
        &f.d,
        &SaveState::new(),
        &state,
        &chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
        "77777777-7777-4777-8777-777777777777",
    )
    .unwrap();
    atomic_write(
        &f.d.join(content_versions::SAVED_VERSIONS_PATH),
        &old.after,
        0o600,
    )
    .unwrap();
    let mut incoming = file_objects_v2::decode(
        &f.payload,
        None,
        &file_objects_v2::supported_schema(),
        Some(&ActualAuthorization),
    )
    .unwrap()
    .objects
    .into_captured();
    incoming.models.payload.policy.as_mut().unwrap().channels[0].models[0]
        .explicit_overrides
        .context_window = Field::Custom(16384);
    incoming.models.digest = schema_v2::digest(&canonical_bytes(&incoming.models.payload).unwrap());
    let incoming = SavedObjects::capture(incoming, Some(&ActualAuthorization)).unwrap();
    let payload = file_objects_v2::encode(
        &incoming,
        SECOND,
        "2026-11-07T00:00:00Z",
        None,
        Some(&ActualAuthorization),
    )
    .unwrap()
    .into_bytes();
    let mut selection = f.selection.clone();
    selection.manager = Default::default();
    selection.skill_names.clear();
    selection.mcp_sources.clear();
    let capture = CaptureRequestV2 {
        selection: selection.clone(),
        mcp_clients: BTreeMap::new(),
        assets_reviewed: false,
    };
    let expected = ActualTarget {
        scope: f.scope.clone(),
    }
    .target(&f.d, &f.h, &capture)
    .unwrap();
    let model = Arc::new(ReceiveFileHostV2 {
        root: f.d.clone(),
        home: f.h.clone(),
        trusted_mapping_write: None,
        bindings: vec![ReceiveBindingV2 {
            portable_channel_id: SOURCE.into(),
            local_channel_id: LOCAL.into(),
            provider_id: "scope_received".into(),
            mode: ReceiveBindingModeV2::Existing,
        }],
    });
    let host = LocalApplicationV2::new(
        f.d.clone(),
        f.h.clone(),
        selection.clone(),
        capture,
        f.scope.clone(),
        Arc::new(ActualTarget {
            scope: f.scope.clone(),
        }),
        model,
        None,
    )
    .unwrap();
    host.apply_once(
        SECOND,
        &payload,
        &file_objects_v2::supported_schema(),
        &f.scope,
        &expected,
        true,
    )
    .unwrap();
    let mut actual_scope = selection.clone();
    actual_scope.channels[0].portable_channel_id = LOCAL.into();
    let request = CaptureRequestV2 {
        selection: actual_scope.clone(),
        mcp_clients: BTreeMap::new(),
        assets_reviewed: false,
    };
    let observed_cloud =
        saved_objects_capture_v2::capture(&f.d, &f.h, &request, &NeverCaptureSecrets).unwrap();
    // The cloud fixture is an actual explicitly saved snapshot with known
    // content-bound clocks, not a metadata-only receive with unknown source time.
    let cloud_saved = content_versions::plan_save_at(
        &f.d,
        &SaveState::new(),
        &observed_cloud.save_state,
        &chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
        "66666666-6666-4666-8666-666666666666",
    )
    .unwrap();
    atomic_write(
        &f.d.join(content_versions::SAVED_VERSIONS_PATH),
        &cloud_saved.after,
        0o600,
    )
    .unwrap();
    let cloud = saved_objects_capture_v2::capture(&f.d, &f.h, &request, &NeverCaptureSecrets)
        .unwrap()
        .objects;
    let versions_before = ledger_bytes(&f.d).unwrap();
    let p = preview(&f.d, &f.h, SECOND, &["channels".into()]).unwrap();
    let result = restore(&f.d, &f.h, &p.restore_id, &p.plan_digest).unwrap();
    assert!(result.status == RestoreStatusV2::Applied);
    let progress: Progress = read(&plan_path(&f.d, &p.restore_id, "progress").unwrap()).unwrap();
    let time = progress.clock_time.as_ref().unwrap();
    let delta = &progress.deltas["@models"];
    let records: BTreeMap<String, content_versions::Version> =
        serde_json::from_slice(&ledger_bytes(&f.d).unwrap().unwrap()).unwrap();
    let old_records: BTreeMap<String, content_versions::Version> =
        serde_json::from_slice(&versions_before.unwrap()).unwrap();
    for (key, value) in &delta.after {
        if delta.before.get(key) != Some(value) {
            assert_eq!(&records[key].saved_at, time);
            assert_eq!(
                records[key].last_event.as_deref(),
                Some(p.restore_id.as_str())
            );
            assert_eq!(records[key].content_digest, *value);
        } else {
            assert_eq!(records[key], old_records[key]);
        }
    }
    let actual = saved_objects_capture_v2::capture(&f.d, &f.h, &request, &NeverCaptureSecrets)
        .unwrap()
        .objects;
    let raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(f.h.join("config.json")).unwrap()).unwrap();
    assert_eq!(
        raw["providers"]["scope_received"]["modelContextWindows"]["fixture"],
        8192
    );
    assert!(matches!(
        sync_host_v2::automatic_direction_from_saved(&actual, Some(&cloud), &actual_scope).unwrap(),
        DirectionV2::Upload
    ));
    let ledger = ledger_bytes(&f.d).unwrap();
    assert!(
        restore(&f.d, &f.h, &p.restore_id, &p.plan_digest)
            .unwrap()
            .status
            == RestoreStatusV2::Applied
    );
    assert_eq!(ledger_bytes(&f.d).unwrap(), ledger);
    assert_eq!(
        read::<Progress>(&plan_path(&f.d, &p.restore_id, "progress").unwrap())
            .unwrap()
            .clock_time
            .as_ref(),
        Some(time)
    );
}

/// This transport has an immutable synthetic graph and rejects every PUT.
/// It cannot contact a server or confirm the prepared download.
struct AbsentPreviewRemote {
    files: BTreeMap<String, Vec<u8>>,
    reads: std::sync::atomic::AtomicUsize,
    writes: std::sync::atomic::AtomicUsize,
}
#[async_trait::async_trait]
impl ConfigSyncTransport for AbsentPreviewRemote {
    async fn list_children(
        &self,
        _: &ConnectionHandle,
        _: ListLimits,
    ) -> std::result::Result<ChildListing, SyncTransportError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        Ok(ChildListing {
            http_status: 207,
            response_bytes: 100,
            entries: self
                .files
                .iter()
                .map(|(name, bytes)| ChildEntry {
                    name: name.clone(),
                    is_collection: false,
                    content_length: Some(bytes.len() as u64),
                })
                .collect(),
        })
    }
    async fn get_file(
        &self,
        _: &ConnectionHandle,
        path: &SyncRelativePath,
        _: ReadLimits,
    ) -> std::result::Result<FileRead, SyncTransportError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.files
            .get(path.as_str())
            .map(|bytes| FileRead {
                http_status: 200,
                body: bytes.clone(),
            })
            .ok_or(SyncTransportError {
                kind: TransportKind::NotFound,
                http_status: Some(404),
                delivery: Delivery::ResponseReceived,
            })
    }
    async fn put_file(
        &self,
        _: &ConnectionHandle,
        _: &SyncRelativePath,
        _: &[u8],
        _: WriteLimits,
    ) -> std::result::Result<PutResponse, SyncTransportError> {
        self.writes.fetch_add(1, Ordering::SeqCst);
        panic!("read-only None restoration probe must not publish a remote file")
    }
}
struct AbsentPreviewEndpointCredentials;
impl crate::modules::model_config::sync_host_v2::EndpointCredentialsV2
    for AbsentPreviewEndpointCredentials
{
    fn read_password(
        &self,
        _: &Path,
        _: &crate::modules::sync::config::SyncEndpointConfig,
    ) -> ModelResult<Zeroizing<String>> {
        Ok(Zeroizing::new("synthetic-readonly-probe-password".into()))
    }
}
fn absent_probe_tree(d: &Path, h: &Path) -> BTreeMap<String, String> {
    let mut output = BTreeMap::new();
    for (label, root) in [("D", d), ("H", h)] {
        let mut state = BTreeMap::new();
        tree(root, root, &mut state, &mut (64 * 1024 * 1024)).unwrap();
        for (path, value) in state {
            output.insert(
                format!("{label}/{}", path.strip_prefix(root).unwrap().display()),
                value,
            );
        }
    }
    output
}
async fn original_none_probe(group: &str) {
    use crate::modules::model_config::{
        content_versions, sync_host_v2 as host,
        sync_service_v2::{self as service, ServiceBackendV2},
        sync_v2::store,
    };
    use crate::modules::sync::config::*;
    let f = Fixture::new();
    if group == "skill:scope-fixture" {
        std::fs::remove_dir_all(f.h.join(".agents/skills/scope-fixture")).unwrap();
    }
    // Actual source/receive material, with matching known Saved clocks for
    // every portable atom. No timestamp is derived from a local mtime/header.
    let mut parts = file_objects_v2::decode(
        &f.payload,
        None,
        &file_objects_v2::supported_schema(),
        Some(&ActualAuthorization),
    )
    .unwrap()
    .objects
    .into_captured();
    parts.manager.webdav.clear();
    let objects = SavedObjects::capture(parts, Some(&ActualAuthorization)).unwrap();
    let atoms = file_objects_v2::save_state(&objects).unwrap();
    let mut parts = objects.into_captured();
    parts.content_versions = atoms
        .into_iter()
        .map(|(scope_key, content_digest)| content_versions::Version {
            scope_key,
            content_digest,
            revision: 1,
            saved_at: "2026-11-01T00:00:00Z".into(),
            time_kind: content_versions::TimeKind::Saved,
            trusted_floor: Some("2026-11-01T00:00:00Z".into()),
            last_event: Some("77777777-7777-4777-8777-777777777777".into()),
        })
        .collect();
    let cloud = SavedObjects::capture(parts, Some(&ActualAuthorization)).unwrap();
    let cloud_atoms = file_objects_v2::save_state(&cloud).unwrap();
    assert!(cloud_atoms.iter().all(|(key, content)| cloud
        .partitions()
        .content_versions
        .iter()
        .any(|v| &v.scope_key == key && &v.content_digest == content && v.instant().is_some())));
    let credential_ref = "cred_aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
    let endpoint = SyncEndpointConfig {
        endpoint_id: "default".into(),
        url: "https://readonly-probe.invalid".into(),
        remote_path: "fixture".into(),
        username: "synthetic".into(),
        credential_ref: CredentialRef {
            ref_id: credential_ref.into(),
            backend: "keychain".into(),
            service_name: crate::infrastructure::keychain::keychain_service_name().into(),
            account_key: crate::infrastructure::keychain::account_key(
                crate::infrastructure::keychain::WEBDAV_CREDENTIAL_PURPOSE,
                credential_ref,
            )
            .unwrap(),
            purpose: crate::infrastructure::keychain::WEBDAV_CREDENTIAL_PURPOSE.into(),
            created_at: "2026-11-01T00:00:00Z".into(),
            updated_at: "2026-11-01T00:00:00Z".into(),
        },
        tls_policy: TLS_POLICY_VERIFY_REQUIRED.into(),
        legacy_encryption: String::new(),
        conflict_policy: CONFLICT_POLICY_ASK.into(),
    };
    SyncConfigStore::new(&f.d)
        .save_config(&SyncConfig {
            schema_version: 1,
            endpoints: vec![endpoint.clone()],
        })
        .unwrap();
    let resolver =
        host::EndpointResolverV2::new(f.d.clone(), Arc::new(AbsentPreviewEndpointCredentials))
            .unwrap();
    let connection = resolver.resolve_active().unwrap().handle().clone();
    let generation_id = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
    let cloud_operation_id = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
    let encoded = file_objects_v2::encode(
        &cloud,
        cloud_operation_id,
        "2026-11-01T00:00:00Z",
        None,
        Some(&ActualAuthorization),
    )
    .unwrap();
    let generation = Generation {
        format: "OCXD_CONFIG_GENERATION".into(),
        format_version: 1,
        generation_id: generation_id.into(),
    };
    let commit = Commit {
        format: "OCXD_CONFIG_COMMIT".into(),
        format_version: 1,
        generation_id: generation_id.into(),
        operation_id: cloud_operation_id.into(),
        parents: vec![],
        payload_digest: encoded.file_digest.clone(),
        payload_bytes: encoded.bytes().len() as u64,
        payload_schema: file_objects_v2::supported_schema(),
        intent: Intent::Update,
        resolution: None,
    };
    let commit_bytes = store::canonical(&commit).unwrap();
    let material = PayloadInput {
        head: CommitRef {
            operation_id: cloud_operation_id.into(),
            digest: store::digest(&commit_bytes),
        },
        bytes: encoded.bytes().to_vec(),
    };
    let remote = Arc::new(AbsentPreviewRemote {
        files: BTreeMap::from([
            (
                SyncRelativePath::generation(generation_id)
                    .unwrap()
                    .as_str()
                    .to_string(),
                store::canonical(&generation).unwrap(),
            ),
            (
                SyncRelativePath::data(generation_id, cloud_operation_id)
                    .unwrap()
                    .as_str()
                    .to_string(),
                encoded.bytes().to_vec(),
            ),
            (
                SyncRelativePath::commit(generation_id, cloud_operation_id)
                    .unwrap()
                    .as_str()
                    .to_string(),
                commit_bytes,
            ),
        ]),
        reads: Default::default(),
        writes: Default::default(),
    });
    let scope = if group == "channels" {
        SelectionV2 {
            usage: UsageV2::RemoteSync,
            channels: f.selection.channels.clone(),
            template_refs: f.selection.template_refs.clone(),
            ..Default::default()
        }
    } else if group == "mcp" {
        SelectionV2 {
            mcp_sources: f.selection.mcp_sources.clone(),
            ..Default::default()
        }
    } else if group == "skill:scope-fixture" {
        SelectionV2 {
            skill_names: vec!["scope-fixture".into()],
            ..Default::default()
        }
    } else {
        SelectionV2 {
            usage: UsageV2::RemoteSync,
            manager: ManagerSelectionV2 {
                extensions: true,
                ..Default::default()
            },
            ..Default::default()
        }
    };
    let request = service::PreviewRequestV2 {
        selection: scope.clone(),
        mcp_clients: if group == "mcp" {
            BTreeMap::from([("scope_fixture".into(), "managed".into())])
        } else {
            BTreeMap::new()
        },
        assets_reviewed: false,
        selected_head: None,
        forced_direction: Some(service::ServiceDirectionV2::Download),
        backup_before_update: true,
    };
    let backend = service::ProductionServiceBackendV2::new(
        f.d.clone(),
        f.h.clone(),
        Arc::new(NeverCaptureSecrets),
    )
    .unwrap();
    let composition = backend
        .compose(&request, &connection, Some(&cloud), None)
        .unwrap();
    let initial_binding = composition
        .provider
        .target_binding(&composition.remote_scope)
        .unwrap()
        .unwrap();
    let local_id = initial_binding
        .slots
        .first()
        .map(|slot| slot.local_id.clone());
    let source = composition.local.current_source().unwrap();
    let target = composition
        .local
        .target_digest(&source.scope_digest)
        .unwrap();
    composition
        .local
        .apply_once(
            OP,
            encoded.bytes(),
            &file_objects_v2::supported_schema(),
            &source.scope_digest,
            &target,
            true,
        )
        .unwrap();
    drop(composition);
    let restored_path = if group == "channels" {
        f.h.join("config.json")
    } else if group == "extensions" {
        f.d.join(crate::modules::extensions::projection::CONFIG_RELATIVE_PATH)
    } else {
        let r: Record = read(&record_path(&f.d, OP).unwrap()).unwrap();
        r.rows
            .iter()
            .find(|row| row.address.group == group && !row.before.exists)
            .unwrap()
            .address
            .path
            .clone()
    };
    assert!(restored_path.exists());
    let plan = preview(&f.d, &f.h, OP, &[group.to_string()]).unwrap();
    let result = restore(&f.d, &f.h, &plan.restore_id, &plan.plan_digest).unwrap();
    assert!(result.status == RestoreStatusV2::Applied);
    assert!(!restored_path.exists());
    let original: Record = read(&record_path(&f.d, OP).unwrap()).unwrap();
    let business_before = original
        .rows
        .iter()
        .map(|row| {
            (
                &row.address.path,
                images::image(&row.address.anchor, &row.address.path).unwrap(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let before = absent_probe_tree(&f.d, &f.h);
    let mut direct_scope = scope.clone();
    if let Some(local_id) = &local_id {
        direct_scope.channels[0].portable_channel_id = local_id.clone();
    }
    let capture_request = CaptureRequestV2 {
        selection: direct_scope,
        mcp_clients: request.mcp_clients.clone(),
        assets_reviewed: false,
    };
    let direct =
        match saved_objects_capture_v2::capture(&f.d, &f.h, &capture_request, &NeverCaptureSecrets)
        {
            Ok(_) => "ok".to_string(),
            Err(error) => error.code,
        };
    if group == "channels" || group == "extensions" {
        assert_eq!(
            direct,
            if group == "channels" {
                "model_source_invalid"
            } else {
                "capture_source_invalid"
            }
        );
    } else {
        assert_ne!(
            direct, "ok",
            "direct capture must not invent a restored missing resource"
        );
    }
    let mut automatic = request;
    automatic.forced_direction = None;
    let composition = backend
        .compose(&automatic, &connection, Some(&cloud), None)
        .unwrap();
    let mapped_capture = composition
        .provider
        .capture_saved(&composition.remote_scope)
        .unwrap();
    let local_objects = mapped_capture.objects.summary();
    let mapped_binding = composition
        .provider
        .target_binding(&composition.remote_scope)
        .unwrap()
        .unwrap();
    let mapping_direction = mapped_binding.direction.clone();
    let prepared = host::preview_semantics(
        composition.provider.as_ref(),
        composition.remote_scope.clone(),
        host::CloudMaterialV2::Valid(&material),
        &host::SessionSecretsV2::default(),
        None,
    )
    .unwrap();
    let host_decision = prepared.decision.clone();
    let client = service::SyncServiceV2::new(
        f.d.clone(),
        Arc::new(
            service::ProductionServiceBackendV2::new(
                f.d.clone(),
                f.h.clone(),
                Arc::new(NeverCaptureSecrets),
            )
            .unwrap(),
        ),
        remote.clone(),
        Arc::new(AbsentPreviewEndpointCredentials),
    )
    .unwrap();
    let catalog = client.catalog(None).await.unwrap();
    if group == "channels" {
        assert!(
            catalog.selected.selection.channels.is_empty(),
            "restored absent channel must not be reselected by first catalog"
        );
    } else if group == "mcp" {
        assert!(catalog.selected.selection.mcp_sources.is_empty());
        assert!(
            catalog
                .mcp_sources
                .iter()
                .any(|m| m.name == "scope_fixture" && !m.sources.is_empty()),
            "remote-only MCP must expose an exact source option for manual reselection"
        );
    } else if group == "skill:scope-fixture" {
        assert!(!catalog
            .selected
            .selection
            .skill_names
            .contains(&"scope-fixture".to_string()));
        assert!(catalog
            .entries
            .iter()
            .any(|e| e.kind == service::ObjectKindV2::Skill
                && e.id == "scope-fixture"
                && e.available_cloud));
    } else {
        assert!(
            !catalog.selected.selection.manager.extensions,
            "restored absent extensions must stay unchecked"
        );
    }
    if group == "mcp" {
        let row = catalog
            .entries
            .iter()
            .find(|entry| entry.kind == service::ObjectKindV2::Mcp && entry.id == "scope_fixture")
            .unwrap();
        assert!(!row.available_local && row.available_cloud);
    }
    // An explicit user action may select it again and download the retained cloud object.
    client
        .save_scope(service::ScopeRequestV2 {
            selection: scope.clone(),
            mcp_clients: BTreeMap::new(),
            assets_reviewed: false,
        })
        .await
        .unwrap();
    assert_eq!(client.read_scope().await.unwrap().unwrap().selection, scope);
    let historical = restore(&f.d, &f.h, &plan.restore_id, &plan.plan_digest).unwrap();
    assert!(historical.status == RestoreStatusV2::Applied);
    assert_eq!(
        client.read_scope().await.unwrap().unwrap().selection,
        scope,
        "historical retry must preserve later explicit reselection"
    );
    let service_preview = client
        .preview(automatic, service::ServiceSecretsV2::default())
        .await
        .unwrap();
    let prepared_operation = match &service_preview.plan {
        service::ServicePlanV2::Download(plan) => plan.operation_id.clone(),
        other => panic!("expected actual download preparation: {other:?}"),
    };
    let business_after = original
        .rows
        .iter()
        .map(|row| {
            (
                &row.address.path,
                images::image(&row.address.anchor, &row.address.path).unwrap(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert!(business_before == business_after);
    assert!(!restored_path.exists());
    assert_eq!(remote.writes.load(Ordering::SeqCst), 0);
    assert!(matches!(mapping_direction, DirectionV2::Download));
    assert!(matches!(
        host_decision.chosen_direction,
        DirectionV2::Download
    ));
    assert!(matches!(
        service_preview.chosen_direction,
        DirectionV2::Download
    ));
    let after = absent_probe_tree(&f.d, &f.h);
    let added = after
        .keys()
        .filter(|path| !before.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    let changed = before
        .iter()
        .filter(|(path, value)| after.get(*path) != Some(*value))
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    assert!(
        changed
            .iter()
            .all(|path| path == "D/manager-state/sync-service-v2/scope.json"
                || path == "D/manager-state/sync-service-v2/scope.json.lock"),
        "unexpected changed paths: {changed:?}"
    );
    assert!(
        added
            .iter()
            .all(|path| path.starts_with("D/manager-state/sync-service-v2/")
                || path == "D/manager-state/account-catalog-ids.json.lock"
                || path == "D/manager-state/preferences.json.lock"
                || path == "D/manager-state/sync-v2-transactions"
                || path.starts_with("D/manager-state/sync-v2-transactions/")),
        "unexpected added paths: {added:?}"
    );
    // The user explicitly selected the retained cloud object again. Verify the
    // complete actual confirmation/receipt path, still with zero remote writes.
    let applied = client
        .confirm(service::ConfirmRequestV2 {
            session_id: service_preview.session_id.clone(),
            plan_digest: service_preview.plan_digest.clone(),
        })
        .await
        .unwrap();
    assert!(applied.local_apply_status == LocalApplyStatus::Applied);
    assert!(
        restored_path.exists(),
        "explicitly reselected object must be received again"
    );
    assert_eq!(remote.writes.load(Ordering::SeqCst), 0);
    let fixture_root = f._temp.path().to_path_buf();
    println!("NONE_RESTORE_PROBE {}",serde_json::to_string(&json!({
        "group":group,"originalOperationId":OP,"restoreId":plan.restore_id,"restorePlanDigest":plan.plan_digest,
        "restoreStatus":"applied","originalTargetWasNone":true,"targetAbsentAfterRestore":true,
        "originalRemoteScope":scope,"directActualCapture":direct,"mappedCaptureSummary":local_objects,
        "mappingDirection":mapping_direction,"mappingSlots":mapped_binding.slots,"hostPreview":host_decision,
        "servicePreviewChosenDirection":service_preview.chosen_direction,"servicePreviewReason":service_preview.automatic_reason,
        "preparedDownloadOperationId":prepared_operation,"confirmationCalled":true,"explicitDownloadActuallyApplied":true,"businessImagesUnchangedDuringPreview":true,
        "remoteReads":remote.reads.load(Ordering::SeqCst),"remotePuts":remote.writes.load(Ordering::SeqCst),
        "previewAddedPaths":added,"previewChangedOrRemovedPaths":changed,
        "scopeAdjustedAfterRestore":true,"explicitUserReselection":true,"realNetworkUsed":false,"policyOrAcAcceptance":false,
    })).unwrap());
    drop(client);
    drop(composition);
    drop(backend);
    drop(resolver);
    drop(f);
    assert!(!fixture_root.exists());
    println!(
        "NONE_RESTORE_CLEANUP {}",
        serde_json::to_string(&json!({"group":group,"fixtureRoot":fixture_root,"removed":true}))
            .unwrap()
    );
}
#[tokio::test]
async fn original_none_channel_restore_deselects_scope_and_explicit_reselection_can_download() {
    original_none_probe("channels").await;
}
#[tokio::test]
async fn original_none_extension_restore_deselects_scope_and_explicit_reselection_can_download() {
    original_none_probe("extensions").await;
}

#[test]
fn changed_remembered_sync_scope_refuses_restore_before_any_business_write() {
    use crate::modules::model_config::sync_service_v2::{scope_restore, ScopeRequestV2};
    let f = Fixture::new();
    f.apply().unwrap();
    let mut selection = SelectionV2::default();
    selection.manager.extensions = true;
    selection.manager.preferences = true;
    scope_restore::save_manual(
        &f.d,
        ScopeRequestV2 {
            selection: selection.clone(),
            mcp_clients: BTreeMap::new(),
            assets_reviewed: false,
        },
    )
    .unwrap();
    let plan = preview(&f.d, &f.h, OP, &["extensions".into()]).unwrap();
    let path =
        f.d.join(crate::modules::extensions::projection::CONFIG_RELATIVE_PATH);
    let before = std::fs::read(&path).unwrap();
    selection.manager.preferences = false;
    scope_restore::save_manual(
        &f.d,
        ScopeRequestV2 {
            selection: selection.clone(),
            mcp_clients: BTreeMap::new(),
            assets_reviewed: false,
        },
    )
    .unwrap();
    assert!(restore(&f.d, &f.h, &plan.restore_id, &plan.plan_digest).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(
        scope_restore::snapshot(&f.d).unwrap().0.unwrap().selection,
        selection
    );
}

#[test]
fn local_file_restore_deselects_actual_local_uuid_and_keeps_unrelated_remembered_channels() {
    use crate::modules::model_config::sync_service_v2::{scope_restore, ScopeRequestV2};
    let f = Fixture::new();
    f.apply().unwrap();
    let mut selection = SelectionV2 {
        channels: [LOCAL, "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa"]
            .into_iter()
            .map(|id| ChannelSelectionV2 {
                portable_channel_id: id.into(),
                model_ids: None,
                family_ids: None,
                include_connection: true,
                include_key: false,
                include_authorization: false,
            })
            .collect(),
        ..Default::default()
    };
    selection.manager.preferences = true;
    scope_restore::save_manual(
        &f.d,
        ScopeRequestV2 {
            selection,
            mcp_clients: BTreeMap::new(),
            assets_reviewed: false,
        },
    )
    .unwrap();
    let plan = preview(&f.d, &f.h, OP, &["channels".into()]).unwrap();
    let result = restore(&f.d, &f.h, &plan.restore_id, &plan.plan_digest).unwrap();
    assert!(result.status == RestoreStatusV2::Applied && result.refresh_sync_scope);
    let saved = scope_restore::snapshot(&f.d).unwrap().0.unwrap();
    assert_eq!(saved.selection.channels.len(), 1);
    assert_eq!(
        saved.selection.channels[0].portable_channel_id,
        "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa"
    );
    assert!(saved.selection.manager.preferences);
}

#[tokio::test]
async fn original_none_skill_restore_deselects_scope_and_explicit_reselection_can_download() {
    original_none_probe("skill:scope-fixture").await;
}
#[tokio::test]
async fn original_none_mcp_restore_deselects_scope_and_exposes_manual_download_source() {
    original_none_probe("mcp").await;
}
