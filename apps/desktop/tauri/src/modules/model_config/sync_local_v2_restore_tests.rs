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
            .tempdir_in("/tmp")
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
        "manager-state/saved-content-versions.json",
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
    let protected_preferences =
        std::fs::read(f.d.join(preferences::PREFERENCES_RELATIVE_PATH)).unwrap();
    let resumed = restore(&f.d, &f.h, &p.restore_id, &p.plan_digest).unwrap();
    assert!(resumed.status == RestoreStatusV2::Applied);
    assert_eq!(
        std::fs::read(f.d.join(preferences::PREFERENCES_RELATIVE_PATH)).unwrap(),
        protected_preferences
    );
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
