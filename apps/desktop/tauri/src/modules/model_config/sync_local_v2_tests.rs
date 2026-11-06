//! Real wrapper/capture/manager/assets in isolated D/H. Only the model
//! participant is a fake file transaction; this does not certify model receive.
use super::super::saved_objects_capture_v2::SelectedKeySourceV2;
use super::*;
use crate::modules::{
    extensions::discovery,
    preferences::{self, Preferences},
};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst};
use tempfile::TempDir;
const OP: &str = "22222222-2222-4222-8222-222222222222";
const EXPORT: &str = "33333333-3333-4333-8333-333333333333";
const SECRET: &str = "synthetic-local-only-secret-023";
const PASSWORD: &str = "synthetic-memory-passphrase-023";
struct NoCredentials;
impl CaptureCredentialsV2 for NoCredentials {
    fn channel_key(&self, _: &Path, _: &SelectedKeySourceV2<'_>) -> ModelResult<Zeroizing<String>> {
        panic!("unselected credential access")
    }
}
// Durable model receipt survives a NEW host instance, including crash after
// the model transaction but before the host can persist ModelsApplied.
struct Models {
    d: PathBuf,
    prepares: AtomicUsize,
    applies: AtomicUsize,
    fail_protect: AtomicBool,
    cut_after_apply: AtomicBool,
    drift_manager: AtomicBool,
    drift_asset: AtomicBool,
    fail_asset_write: AtomicBool,
}
impl Models {
    fn plan(&self, id: &str) -> PathBuf {
        self.d.join(format!("model-{id}.plan"))
    }
    fn receipt(&self, id: &str) -> PathBuf {
        self.d.join(format!("model-{id}.receipt"))
    }
    fn business(&self) -> PathBuf {
        self.d.join("model-business.json")
    }
}
impl ModelApplyV2 for Models {
    fn prepare(
        &self,
        id: &str,
        objects: &SavedObjects,
        _: &SelectionV2,
        _: &str,
    ) -> ModelResult<ModelApplyPlanHandle> {
        self.prepares.fetch_add(1, SeqCst);
        let token = hash(&super::super::file_objects_v2::encode_document(
            objects,
            Some(&ActualAuthorization),
        )?);
        let h = ModelApplyPlanHandle::new(token)?;
        if self.plan(id).exists() {
            if std::fs::read(self.plan(id))? != h.token().as_bytes() {
                return Err(invalid("test_plan_collision", "collision"));
            }
        } else {
            atomic_write(&self.plan(id), h.token().as_bytes(), 0o600)?;
        }
        Ok(h)
    }
    fn protect(&self, _: &str, _: &ModelApplyPlanHandle, _: bool) -> ModelResult<()> {
        if self.fail_protect.load(SeqCst) {
            return Err(invalid("test_backup", SECRET));
        }
        Ok(())
    }
    fn apply_once(
        &self,
        id: &str,
        _: &SavedObjects,
        h: &ModelApplyPlanHandle,
        _: bool,
    ) -> ModelResult<ModelApplyReceipt> {
        assert!(!self.fail_protect.load(SeqCst));
        self.applies.fetch_add(1, SeqCst);
        atomic_write(&self.business(), b"model-applied", 0o600)?;
        atomic_write(&self.receipt(id), h.token().as_bytes(), 0o600)?;
        if self.drift_manager.swap(false, SeqCst) {
            atomic_write(
                &self.d.join(preferences::PREFERENCES_RELATIVE_PATH),
                b"{\"theme\":\"light\"}",
                0o600,
            )?;
        }
        if self.drift_asset.swap(false, SeqCst) {
            atomic_write(
                &discovery::skills_store_dir(&self.d).join("fixture/SKILL.md"),
                b"external asset edit",
                0o600,
            )?;
        }
        #[cfg(unix)]
        if self.fail_asset_write.swap(false, SeqCst) {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                discovery::skills_store_dir(&self.d),
                std::fs::Permissions::from_mode(0o500),
            )?;
        }
        if self.cut_after_apply.swap(false, SeqCst) {
            return Err(invalid("test_crash", SECRET));
        }
        Ok(ModelApplyReceipt {
            operation_id: id.into(),
            plan: h.clone(),
        })
    }
    fn status(&self, id: &str, h: &ModelApplyPlanHandle) -> ModelResult<ModelApplyStatus> {
        if self.receipt(id).exists() {
            if std::fs::read(self.receipt(id))? != h.token().as_bytes() {
                return Ok(ModelApplyStatus::Unknown);
            }
            Ok(ModelApplyStatus::Applied(ModelApplyReceipt {
                operation_id: id.into(),
                plan: h.clone(),
            }))
        } else if self.plan(id).exists() {
            Ok(ModelApplyStatus::Prepared)
        } else {
            Ok(ModelApplyStatus::Unknown)
        }
    }
}
struct Fixture {
    _temp: TempDir,
    d: PathBuf,
    h: PathBuf,
    selection: SelectionV2,
    capture_json: Vec<u8>,
    scope: String,
    expected: String,
    payload: Zeroizing<Vec<u8>>,
    models: Arc<Models>,
}
impl Fixture {
    fn new(secret_skill: bool) -> Self {
        let t = tempfile::Builder::new()
            .prefix("ocxd023-local-apply-")
            .tempdir_in("/tmp")
            .unwrap();
        let d = t.path().join("D");
        let h = t.path().join("H");
        std::fs::create_dir_all(d.join("manager-state")).unwrap();
        std::fs::create_dir_all(&h).unwrap();
        atomic_write(&h.join("config.json"), br#"{"providers":{"fixture":{"adapter":"openai-chat","models":["fixture-model"],"authMode":"none"}}}"#, 0o600).unwrap();
        let prefs = preferences::document_from_preferences(&Preferences::default());
        atomic_write(
            &d.join(preferences::PREFERENCES_RELATIVE_PATH),
            &json(&prefs).unwrap(),
            0o600,
        )
        .unwrap();
        let store = discovery::skills_store_dir(&d).join("fixture");
        std::fs::create_dir_all(&store).unwrap();
        atomic_write(
            &store.join("SKILL.md"),
            b"# Fixture\nOld instruction.\n",
            0o600,
        )
        .unwrap();
        if secret_skill {
            atomic_write(&store.join(".env"), b"synthetic-old-secret", 0o600).unwrap();
        }
        let selection = SelectionV2 {
            manager: objects_overlay_v2::ManagerSelectionV2 {
                preferences: true,
                ..Default::default()
            },
            skill_names: vec!["fixture".into()],
            extension_credentials: objects_overlay_v2::ExtensionCredentialsV2 {
                skill_names: if secret_skill {
                    vec!["fixture".into()]
                } else {
                    vec![]
                },
                mcp_names: vec![],
            },
            ..Default::default()
        };
        let request = CaptureRequestV2 {
            selection: selection.clone(),
            mcp_clients: BTreeMap::new(),
            assets_reviewed: true,
        };
        let snapshot = saved_objects_capture_v2::capture(&d, &h, &request, &NoCredentials).unwrap();
        let scope = snapshot.source_guard.scope_digest.clone();
        let expected = snapshot.source_guard.saved_source_digest.clone();
        let mut incoming = snapshot.objects.into_captured();
        incoming.manager.preferences.as_mut().unwrap().theme = "dark".into();
        let incoming_dir = t.path().join("incoming");
        std::fs::create_dir_all(&incoming_dir).unwrap();
        std::fs::write(
            incoming_dir.join("SKILL.md"),
            b"# Fixture\nNew instruction.\n",
        )
        .unwrap();
        if secret_skill {
            std::fs::write(incoming_dir.join(".env"), SECRET).unwrap();
        }
        incoming.assets.skills =
            vec![
                portable_assets::snapshot_skill(&incoming_dir, "fixture", secret_skill, true)
                    .unwrap(),
            ];
        incoming.content_versions.clear();
        let saved = SavedObjects::capture(incoming, Some(&ActualAuthorization)).unwrap();
        let payload = file_objects_v2::encode(
            &saved,
            OP,
            "2026-11-06T10:00:00Z",
            Some(PASSWORD),
            Some(&ActualAuthorization),
        )
        .unwrap()
        .into_bytes();
        let mut request = request;
        // Same explicit credential selection for receive and target observation.
        request.selection = selection.clone();
        let models = Arc::new(Models {
            d: d.clone(),
            prepares: AtomicUsize::new(0),
            applies: AtomicUsize::new(0),
            fail_protect: AtomicBool::new(false),
            cut_after_apply: AtomicBool::new(false),
            drift_manager: AtomicBool::new(false),
            drift_asset: AtomicBool::new(false),
            fail_asset_write: AtomicBool::new(false),
        });
        Self {
            _temp: t,
            d,
            h,
            selection,
            capture_json: json(&request).unwrap(),
            scope,
            expected,
            payload,
            models,
        }
    }
    fn host(&self) -> LocalApplicationV2 {
        LocalApplicationV2::new(
            self.d.clone(),
            self.h.clone(),
            self.selection.clone(),
            serde_json::from_slice(&self.capture_json).unwrap(),
            self.scope.clone(),
            Arc::new(SavedSourceSnapshotProviderV2(NoCredentials)),
            self.models.clone(),
            Some(Zeroizing::new(PASSWORD.into())),
        )
        .unwrap()
    }
    fn apply(&self, host: &LocalApplicationV2) -> SyncResult<()> {
        host.apply_once(
            OP,
            &self.payload,
            &file_objects_v2::supported_schema(),
            &self.scope,
            &self.expected,
            true,
        )
    }
    fn theme(&self) -> String {
        let document = serde_json::from_slice::<Value>(
            &std::fs::read(self.d.join(preferences::PREFERENCES_RELATIVE_PATH)).unwrap(),
        )
        .unwrap();
        preferences::preferences_from_document(&document)
            .unwrap()
            .theme
    }
}

#[test]
fn real_participants_apply_and_reopened_retries_preserve_later_edits() {
    let f = Fixture::new(false);
    let host = f.host();
    f.apply(&host).unwrap();
    assert_eq!(f.theme(), "dark");
    assert_eq!(host.progress(OP).unwrap(), Some(LocalProgressV2::Applied));
    let skill = portable_assets::snapshot_skill(
        &discovery::skills_store_dir(&f.d).join("fixture"),
        "fixture",
        false,
        true,
    )
    .unwrap();
    assert_eq!(skill.files.len(), 1);
    std::fs::write(f.models.business(), b"later model edit").unwrap();
    atomic_write(
        &f.d.join(preferences::PREFERENCES_RELATIVE_PATH),
        br#"{"theme":"light"}"#,
        0o600,
    )
    .unwrap();
    std::fs::write(
        discovery::skills_store_dir(&f.d).join("fixture/SKILL.md"),
        "later asset edit",
    )
    .unwrap();
    let journal = std::fs::read(host.path(OP).unwrap()).unwrap();
    f.apply(&f.host()).unwrap();
    assert_eq!(std::fs::read(host.path(OP).unwrap()).unwrap(), journal);
    assert_eq!(
        std::fs::read(f.models.business()).unwrap(),
        b"later model edit"
    );
    assert_eq!(f.theme(), "light");
    assert_eq!(f.models.prepares.load(SeqCst), 1);
    assert_eq!(f.models.applies.load(SeqCst), 1);
    let records = backup::list_action_records(&f.d.join("backups"), "import").unwrap();
    assert!(records.len() >= 2);
    assert!(records.iter().all(|r| backup::verify_backup(r).unwrap()));
}
#[test]
fn actual_backup_failure_leaves_all_business_files_untouched_and_plan_reusable() {
    let f = Fixture::new(false);
    let host = f.host();
    let prefs = std::fs::read(f.d.join(preferences::PREFERENCES_RELATIVE_PATH)).unwrap();
    let skill = std::fs::read(discovery::skills_store_dir(&f.d).join("fixture/SKILL.md")).unwrap();
    std::fs::write(f.d.join("backups"), b"obstruct backup directory").unwrap();
    assert_eq!(
        f.apply(&host).unwrap_err().code,
        ErrorCode::SyncBackupFailed
    );
    assert_eq!(f.models.applies.load(SeqCst), 0);
    assert_eq!(
        std::fs::read(f.d.join(preferences::PREFERENCES_RELATIVE_PATH)).unwrap(),
        prefs
    );
    assert_eq!(
        std::fs::read(discovery::skills_store_dir(&f.d).join("fixture/SKILL.md")).unwrap(),
        skill
    );
    let manager = std::fs::read(host.manager_path(OP)).unwrap();
    let asset = std::fs::read(host.asset_path(OP)).unwrap();
    std::fs::remove_file(f.d.join("backups")).unwrap();
    f.apply(&f.host()).unwrap();
    assert_eq!(f.models.prepares.load(SeqCst), 1);
    // Compare immutable plan facts, not mutable participant progress.
    let a: Value = serde_json::from_slice(&manager).unwrap();
    let b: Value = serde_json::from_slice(&std::fs::read(host.manager_path(OP)).unwrap()).unwrap();
    assert_eq!(a["files"], b["files"]);
    let a: Value = serde_json::from_slice(&asset).unwrap();
    let b: Value = serde_json::from_slice(&std::fs::read(host.asset_path(OP)).unwrap()).unwrap();
    assert_eq!(a["skillsBefore"], b["skillsBefore"]);
}
#[test]
fn model_backup_failure_is_a_barrier_and_error_has_no_secret() {
    let f = Fixture::new(false);
    f.models.fail_protect.store(true, SeqCst);
    let before = f.theme();
    let e = f.apply(&f.host()).unwrap_err();
    assert_eq!(e.code, ErrorCode::SyncBackupFailed);
    assert!(!format!("{e:?}").contains(SECRET));
    assert_eq!(f.theme(), before);
    assert_eq!(f.models.applies.load(SeqCst), 0);
}
#[test]
fn crash_after_model_receipt_reopens_same_operation_and_continues_other_participants() {
    let f = Fixture::new(false);
    f.models.cut_after_apply.store(true, SeqCst);
    let host = f.host();
    assert_eq!(
        f.apply(&host).unwrap_err().code,
        ErrorCode::SyncOperationUnknown
    );
    assert_eq!(host.progress(OP).unwrap(), Some(LocalProgressV2::Protected));
    std::fs::write(f.models.business(), b"user changed model after crash").unwrap();
    f.apply(&f.host()).unwrap();
    assert_eq!(f.theme(), "dark");
    assert_eq!(
        std::fs::read(f.models.business()).unwrap(),
        b"user changed model after crash"
    );
    assert_eq!(f.models.applies.load(SeqCst), 1);
    assert_eq!(f.models.prepares.load(SeqCst), 1);
}
#[test]
fn external_manager_edit_after_model_is_rejected_without_rebasing_before() {
    let f = Fixture::new(false);
    f.models.drift_manager.store(true, SeqCst);
    let host = f.host();
    assert_eq!(
        f.apply(&host).unwrap_err().code,
        ErrorCode::SyncLocalChanged
    );
    let guard = host.load(OP).unwrap().unwrap().manager_guard;
    assert_eq!(
        f.apply(&f.host()).unwrap_err().code,
        ErrorCode::SyncLocalChanged
    );
    assert_eq!(host.load(OP).unwrap().unwrap().manager_guard, guard);
    assert_eq!(f.theme(), "light");
    assert_eq!(f.models.applies.load(SeqCst), 1);
}
#[test]
fn asset_failure_retains_manager_progress_and_original_guard() {
    let f = Fixture::new(false);
    f.models.drift_asset.store(true, SeqCst);
    let host = f.host();
    assert_eq!(
        f.apply(&host).unwrap_err().code,
        ErrorCode::SyncLocalChanged
    );
    assert_eq!(
        host.progress(OP).unwrap(),
        Some(LocalProgressV2::ManagerApplied)
    );
    let asset = std::fs::read(host.asset_path(OP)).unwrap();
    assert_eq!(
        f.apply(&f.host()).unwrap_err().code,
        ErrorCode::SyncLocalChanged
    );
    assert_eq!(std::fs::read(host.asset_path(OP)).unwrap(), asset);
    assert_eq!(f.models.applies.load(SeqCst), 1);
    assert_eq!(f.theme(), "dark");
}
#[test]
fn exact_payload_intent_scope_root_and_uuid_are_bound() {
    let f = Fixture::new(false);
    let host = f.host();
    f.models.cut_after_apply.store(true, SeqCst);
    assert!(f.apply(&host).is_err());
    let decoded = file_objects_v2::decode(
        &f.payload,
        Some(PASSWORD),
        &file_objects_v2::supported_schema(),
        Some(&ActualAuthorization),
    )
    .unwrap();
    let other = file_objects_v2::encode(
        &decoded.objects,
        OP,
        "2026-11-06T10:00:00Z",
        Some(PASSWORD),
        Some(&ActualAuthorization),
    )
    .unwrap();
    assert_eq!(
        host.apply_once(
            OP,
            other.bytes(),
            &other.schema,
            &f.scope,
            &f.expected,
            true
        )
        .unwrap_err()
        .code,
        ErrorCode::SyncOperationCollision
    );
    assert_eq!(
        host.apply_once(OP, &f.payload, &other.schema, &f.scope, &f.expected, false)
            .unwrap_err()
            .code,
        ErrorCode::SyncOperationCollision
    );
    assert_eq!(
        host.apply_once(
            OP,
            &f.payload,
            &other.schema,
            &hash(b"wrong scope"),
            &f.expected,
            true
        )
        .unwrap_err()
        .code,
        ErrorCode::SyncScopeChanged
    );
    assert!(host
        .apply_once(
            "22222222222242228222222222222222",
            &f.payload,
            &other.schema,
            &f.scope,
            &f.expected,
            true
        )
        .is_err());
    let different_home = f._temp.path().join("H2");
    std::fs::create_dir_all(&different_home).unwrap();
    let changed = LocalApplicationV2::new(
        f.d.clone(),
        different_home,
        f.selection.clone(),
        serde_json::from_slice(&f.capture_json).unwrap(),
        f.scope.clone(),
        Arc::new(SavedSourceSnapshotProviderV2(NoCredentials)),
        f.models.clone(),
        Some(Zeroizing::new(PASSWORD.into())),
    )
    .unwrap();
    assert_eq!(
        f.apply(&changed).unwrap_err().code,
        ErrorCode::SyncSourceChanged
    );
}
#[test]
fn expected_target_mismatch_is_not_an_own_receipt() {
    let f = Fixture::new(false);
    let host = f.host();
    assert_eq!(
        host.apply_once(
            OP,
            &f.payload,
            &file_objects_v2::supported_schema(),
            &f.scope,
            &hash(b"not previewed"),
            true
        )
        .unwrap_err()
        .code,
        ErrorCode::SyncLocalChanged
    );
    assert!(host.progress(OP).unwrap().is_none());
    assert_eq!(f.models.applies.load(SeqCst), 0);
}
#[test]
fn journal_checksum_duplicates_permissions_and_plan_tampering_fail_closed() {
    let f = Fixture::new(false);
    let host = f.host();
    f.models.cut_after_apply.store(true, SeqCst);
    assert!(f.apply(&host).is_err());
    let p = host.path(OP).unwrap();
    let bytes = std::fs::read(&p).unwrap();
    let mut v: Value = serde_json::from_slice(&bytes).unwrap();
    v["journal"]["backup"] = json!(false);
    atomic_write(&p, &json(&v).unwrap(), 0o600).unwrap();
    assert_eq!(
        f.apply(&host).unwrap_err().code,
        ErrorCode::SyncJournalInvalid
    );
    let duplicate =
        String::from_utf8(bytes.clone())
            .unwrap()
            .replacen("{", "{\"checksum\":\"duplicate\",", 1);
    atomic_write(&p, duplicate.as_bytes(), 0o600).unwrap();
    assert_eq!(
        f.apply(&host).unwrap_err().code,
        ErrorCode::SyncJournalInvalid
    );
    atomic_write(&p, &bytes, 0o600).unwrap();
    let manager = host.manager_path(OP);
    let mut v: Value = serde_json::from_slice(&std::fs::read(&manager).unwrap()).unwrap();
    v["files"][0]["after"] = json!([1, 2, 3]);
    atomic_write(&manager, &json(&v).unwrap(), 0o600).unwrap();
    assert_eq!(
        f.apply(&host).unwrap_err().code,
        ErrorCode::SyncJournalInvalid
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            host.progress(OP).unwrap_err().code,
            ErrorCode::SyncJournalInvalid
        );
    }
}
#[test]
fn export_same_id_is_exact_durable_and_never_replaces_external_content() {
    let f = Fixture::new(false);
    let host = f.host();
    assert_eq!(
        host.export_once(
            EXPORT,
            &f.payload,
            &file_objects_v2::supported_schema(),
            &f.scope
        )
        .unwrap(),
        EXPORT
    );
    let path = f.d.join("exports").join(format!("{EXPORT}.ocxdconf"));
    let stamp = std::fs::metadata(&path).unwrap().modified().unwrap();
    let receipt = std::fs::read(host.path(EXPORT).unwrap()).unwrap();
    assert_eq!(
        f.host()
            .export_once(
                EXPORT,
                &f.payload,
                &file_objects_v2::supported_schema(),
                &f.scope
            )
            .unwrap(),
        EXPORT
    );
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), stamp);
    assert_eq!(std::fs::read(host.path(EXPORT).unwrap()).unwrap(), receipt);
    assert_eq!(std::fs::read(&path).unwrap(), f.payload.as_slice());
    assert_eq!(f.models.prepares.load(SeqCst), 0);
    std::fs::write(&path, b"external edit").unwrap();
    assert_eq!(
        host.export_once(
            EXPORT,
            &f.payload,
            &file_objects_v2::supported_schema(),
            &f.scope
        )
        .unwrap_err()
        .code,
        ErrorCode::SyncOperationCollision
    );
    assert_eq!(std::fs::read(&path).unwrap(), b"external edit");
    std::fs::remove_file(&path).unwrap();
    assert!(host
        .export_once(
            EXPORT,
            &f.payload,
            &file_objects_v2::supported_schema(),
            &f.scope
        )
        .is_err());
    assert!(!path.exists());
}
#[test]
fn invalid_wrapper_schema_and_missing_selection_do_not_prepare() {
    let f = Fixture::new(false);
    let host = f.host();
    assert_eq!(
        host.apply_once(
            OP,
            b"{}",
            &file_objects_v2::supported_schema(),
            &f.scope,
            &f.expected,
            true
        )
        .unwrap_err()
        .code,
        ErrorCode::SyncPayloadInvalid
    );
    let mut schema = file_objects_v2::supported_schema();
    schema.document.version += 1;
    assert_eq!(
        host.apply_once(OP, &f.payload, &schema, &f.scope, &f.expected, true)
            .unwrap_err()
            .code,
        ErrorCode::SyncPayloadSchemaUnsupported
    );
    assert_eq!(f.models.prepares.load(SeqCst), 0);
    assert!(host.progress(OP).unwrap().is_none());
}
#[test]
fn journal_and_export_receipt_contain_no_payload_passphrase_or_secret() {
    let f = Fixture::new(true);
    let host = f.host();
    f.apply(&host).unwrap();
    assert_eq!(
        std::fs::read(discovery::skills_store_dir(&f.d).join("fixture/.env")).unwrap(),
        SECRET.as_bytes()
    );
    host.export_once(
        EXPORT,
        &f.payload,
        &file_objects_v2::supported_schema(),
        &f.scope,
    )
    .unwrap();
    for id in [OP, EXPORT] {
        let s = std::fs::read_to_string(host.path(id).unwrap()).unwrap();
        assert!(!s.contains(PASSWORD));
        assert!(!s.contains(SECRET));
        assert!(!s.contains("New instruction"));
    }
}

#[test]
fn complete_baseline_does_not_apply_unselected_manager_or_assets() {
    let f = Fixture::new(false);
    let host = f.host();
    let decoded = file_objects_v2::decode(
        &f.payload,
        Some(PASSWORD),
        &file_objects_v2::supported_schema(),
        Some(&ActualAuthorization),
    )
    .unwrap();
    let mut incoming = decoded.objects.into_captured();
    incoming.manager.extensions = Some(Default::default());
    let mut extra = incoming.assets.skills[0].clone();
    extra.name = "unselected".into();
    incoming.assets.skills.push(extra);
    let saved = SavedObjects::capture(incoming, Some(&ActualAuthorization)).unwrap();
    let payload = file_objects_v2::encode(
        &saved,
        OP,
        "2026-11-06T10:00:00Z",
        Some(PASSWORD),
        Some(&ActualAuthorization),
    )
    .unwrap();
    host.apply_once(
        OP,
        payload.bytes(),
        &payload.schema,
        &f.scope,
        &f.expected,
        true,
    )
    .unwrap();
    assert!(!discovery::skills_store_dir(&f.d)
        .join("unselected")
        .exists());
    assert!(!f
        .d
        .join(crate::modules::extensions::projection::CONFIG_RELATIVE_PATH)
        .exists());
    assert_eq!(f.theme(), "dark");
}

#[test]
fn pure_receive_accepts_existing_channel_without_connection_adoption() {
    let f = Fixture::new(false);
    let id = super::super::config_file::v2::with_saved_config(&f.d, &f.h, |_, p, _, _, _, _| {
        Ok(p.channels[0].id.clone())
    })
    .unwrap();
    let mut request: CaptureRequestV2 = serde_json::from_slice(&f.capture_json).unwrap();
    request
        .selection
        .channels
        .push(objects_overlay_v2::ChannelSelectionV2 {
            portable_channel_id: id,
            model_ids: None,
            family_ids: None,
            include_connection: false,
            include_key: false,
            include_authorization: false,
        });
    let snapshot = saved_objects_capture_v2::capture(&f.d, &f.h, &request, &NoCredentials).unwrap();
    let prefs = std::fs::read(f.d.join(preferences::PREFERENCES_RELATIVE_PATH)).unwrap();
    let objects = select_received_objects_v2(snapshot.objects, &request.selection).unwrap();
    assert!(
        objects.models().payload.policy.as_ref().unwrap().channels[0]
            .connection
            .is_none()
    );
    assert_eq!(
        std::fs::read(f.d.join(preferences::PREFERENCES_RELATIVE_PATH)).unwrap(),
        prefs
    );
    assert!(!f.d.join("manager-state/portable-manager-plans").exists());
}

#[test]
fn pure_receive_enforces_exact_selected_mcp_source_and_missing_objects() {
    let f = Fixture::new(false);
    let decoded = file_objects_v2::decode(
        &f.payload,
        Some(PASSWORD),
        &file_objects_v2::supported_schema(),
        Some(&ActualAuthorization),
    )
    .unwrap();
    let mut incoming = decoded.objects.into_captured();
    let mcp = portable_assets::Mcp {
        name: "fixture-mcp".into(),
        definition: json!({"command":"fixture-command","args":["--fixture"]}),
        secret_fields: vec![],
        secrets: Default::default(),
        warnings: vec![],
    };
    let digest = objects_overlay_v2::mcp_source_digest(&mcp).unwrap();
    incoming.assets.mcps.push(mcp);
    let saved = SavedObjects::capture(incoming, Some(&ActualAuthorization)).unwrap();
    let document = file_objects_v2::encode_document(&saved, Some(&ActualAuthorization)).unwrap();
    let mut selection = f.selection.clone();
    selection
        .mcp_sources
        .push(objects_overlay_v2::McpSourceSelectionV2 {
            name: "fixture-mcp".into(),
            definition_digest: digest,
        });
    let selected = select_received_objects_v2(saved, &selection).unwrap();
    assert_eq!(selected.partitions().assets.mcps.len(), 1);
    selection.mcp_sources[0].definition_digest = hash(b"wrong source");
    assert!(select_received_objects_v2(
        file_objects_v2::decode_document(&document, Some(&ActualAuthorization)).unwrap(),
        &selection
    )
    .is_err());
    selection.mcp_sources.clear();
    selection.skill_names.push("missing".into());
    assert!(select_received_objects_v2(
        file_objects_v2::decode_document(&document, Some(&ActualAuthorization)).unwrap(),
        &selection
    )
    .is_err());
}

#[cfg(unix)]
#[test]
fn actual_asset_apply_failure_is_durable_and_retries_without_rewriting_model_or_manager() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new(false);
    let host = f.host();
    f.models.fail_asset_write.store(true, SeqCst);
    let result = f.apply(&host);
    // Restore permissions before assertions so TempDir can always clean up.
    std::fs::set_permissions(
        discovery::skills_store_dir(&f.d),
        std::fs::Permissions::from_mode(0o700),
    )
    .unwrap();
    assert_eq!(result.unwrap_err().code, ErrorCode::SyncOperationUnknown);
    assert_eq!(
        host.progress(OP).unwrap(),
        Some(LocalProgressV2::ManagerApplied)
    );
    assert_eq!(f.theme(), "dark");
    let manager = std::fs::read(host.manager_path(OP)).unwrap();
    std::fs::write(f.models.business(), b"later model edit").unwrap();
    f.apply(&f.host()).unwrap();
    assert_eq!(f.models.applies.load(SeqCst), 1);
    assert_eq!(
        std::fs::read(f.models.business()).unwrap(),
        b"later model edit"
    );
    assert_eq!(std::fs::read(host.manager_path(OP)).unwrap(), manager);
    assert_eq!(host.progress(OP).unwrap(), Some(LocalProgressV2::Applied));
}

#[test]
fn missing_frozen_backup_on_resume_blocks_remaining_business_writes() {
    let f = Fixture::new(false);
    let host = f.host();
    f.models.cut_after_apply.store(true, SeqCst);
    let theme = f.theme();
    assert!(f.apply(&host).is_err());
    let journal = host.load(OP).unwrap().unwrap();
    assert!(!journal.backups.is_empty());
    std::fs::remove_dir_all(&journal.backups[0].directory).unwrap();
    assert_eq!(
        f.apply(&f.host()).unwrap_err().code,
        ErrorCode::SyncBackupFailed
    );
    assert_eq!(f.theme(), theme);
    assert_eq!(f.models.applies.load(SeqCst), 1);
}

// Actual Host dual-domain composition, only mapping facts/model transaction fake.
use super::super::sync_host_v2::{
    self, CurrentSourceProviderV2, MappedCaptureProviderV2, SavedCaptureProviderV2, TargetSlotV2,
};
struct MappedOwner {
    revision: String,
    binding_digest: String,
    revision_valid: AtomicBool,
    absent_valid: AtomicBool,
    checks: AtomicUsize,
    revision_checks: AtomicUsize,
    receipt: PathBuf,
}
impl MappedOwner {
    fn verify_revision(&self, b: &LocalTargetBindingV2) -> ModelResult<()> {
        self.revision_checks.fetch_add(1, SeqCst);
        if !self.revision_valid.load(SeqCst) || b.mapping_revision != self.revision {
            return Err(invalid(
                "host_mapping_changed",
                "synthetic mapping revision changed",
            ));
        }
        Ok(())
    }
}
impl TrustedMappingOwnerV2 for MappedOwner {
    fn verify(&self, b: &LocalTargetBindingV2) -> ModelResult<()> {
        self.checks.fetch_add(1, SeqCst);
        self.verify_revision(b)?;
        if b.digest != self.binding_digest
            || !self.absent_valid.load(SeqCst)
            || self.receipt.exists()
        {
            return Err(invalid(
                "host_mapping_changed",
                "synthetic original absent fact changed",
            ));
        }
        Ok(())
    }
    fn project_saved_to_remote(
        &self,
        objects: SavedObjects,
        _: &LocalTargetBindingV2,
    ) -> ModelResult<SavedObjects> {
        assert!(objects
            .models()
            .payload
            .policy
            .as_ref()
            .is_none_or(|p| p.channels.is_empty()));
        Ok(objects)
    }
}
struct MappedSource {
    current: CurrentSourceProviderV2,
    owner: Arc<MappedOwner>,
    binding: LocalTargetBindingV2,
}
impl SourceSnapshotProviderV2 for MappedSource {
    fn current(&self, _: &Path, _: &Path, r: &CaptureRequestV2) -> ModelResult<SourceGuard> {
        assert!(r.selection == self.binding.local_scope);
        self.current
            .current_source()
            .map_err(|_| invalid("host_mapping_changed", "mapped current unavailable"))
    }
    fn root(&self, _: &Path, _: &Path, _: &CaptureRequestV2) -> ModelResult<String> {
        self.current
            .root_revision()
            .map_err(|_| invalid("host_mapping_changed", "mapped root unavailable"))
    }
    fn target(&self, _: &Path, _: &Path, _: &CaptureRequestV2) -> ModelResult<String> {
        Ok(self
            .current
            .current_source()
            .map_err(|_| invalid("host_mapping_changed", "mapped target unavailable"))?
            .saved_source_digest)
    }
    fn verify_applied_mapping(&self, id: &str, b: &LocalTargetBindingV2) -> ModelResult<()> {
        assert_eq!(id, OP);
        self.owner.verify_revision(b)
    }
}
struct MappedFixture {
    f: Fixture,
    binding: LocalTargetBindingV2,
    owner: Arc<MappedOwner>,
    source: Arc<MappedSource>,
}
impl MappedFixture {
    fn new(with_non_model: bool) -> Self {
        Self::build(with_non_model, false)
    }
    fn empty_local_full_remote() -> Self {
        Self::build(true, true)
    }
    fn build(with_non_model: bool, absent_non_model: bool) -> Self {
        let mut f = Fixture::new(false);
        // This really is a new device: explicit empty local channel selection,
        // not a capture failure converted to empty and not an ID/name fallback.
        atomic_write(&f.h.join("config.json"), b"{}", 0o600).unwrap();
        let cloud_home = f._temp.path().join("cloud-H");
        let cloud_root = f._temp.path().join("cloud-D");
        std::fs::create_dir_all(cloud_root.join("manager-state")).unwrap();
        if with_non_model {
            std::fs::copy(
                f.d.join(preferences::PREFERENCES_RELATIVE_PATH),
                cloud_root.join(preferences::PREFERENCES_RELATIVE_PATH),
            )
            .unwrap();
            let skill = discovery::skills_store_dir(&cloud_root).join("fixture");
            std::fs::create_dir_all(&skill).unwrap();
            std::fs::copy(
                discovery::skills_store_dir(&f.d).join("fixture/SKILL.md"),
                skill.join("SKILL.md"),
            )
            .unwrap();
        }
        if absent_non_model {
            std::fs::remove_file(f.d.join(preferences::PREFERENCES_RELATIVE_PATH)).unwrap();
            std::fs::remove_dir_all(discovery::skills_store_dir(&f.d)).unwrap();
        }
        std::fs::create_dir_all(&cloud_home).unwrap();
        atomic_write(&cloud_home.join("config.json"), br#"{"providers":{"cloudnewA":{"adapter":"openai-chat","models":["cloud-model"],"authMode":"none"}}}"#, 0o600).unwrap();
        let id = super::super::config_file::v2::with_saved_config(
            &cloud_root,
            &cloud_home,
            |_, p, _, _, _, _| Ok(p.channels[0].id.clone()),
        )
        .unwrap();
        let local = if with_non_model && !absent_non_model {
            f.selection.clone()
        } else {
            SelectionV2::default()
        };
        let mut remote = if absent_non_model {
            f.selection.clone()
        } else {
            local.clone()
        };
        let mcp = absent_non_model.then(|| portable_assets::Mcp {
            name: "cloud-only-mcp".into(),
            definition: json!({"command":"fixture-command","args":["--cloud"]}),
            secret_fields: vec![],
            secrets: Default::default(),
            warnings: vec![],
        });
        if let Some(mcp) = &mcp {
            remote.manager.extensions = true;
            remote
                .mcp_sources
                .push(objects_overlay_v2::McpSourceSelectionV2 {
                    name: mcp.name.clone(),
                    definition_digest: objects_overlay_v2::mcp_source_digest(mcp).unwrap(),
                });
        }
        remote
            .channels
            .push(objects_overlay_v2::ChannelSelectionV2 {
                portable_channel_id: id.clone(),
                model_ids: None,
                family_ids: None,
                include_connection: true,
                include_key: false,
                include_authorization: false,
            });
        let binding = LocalTargetBindingV2::freeze(
            hash(b"mapped revision"),
            local.clone(),
            remote.clone(),
            DirectionV2::Download,
            vec![TargetSlotV2 {
                kind: "channel".into(),
                portable_id: id,
                local_id: "44444444-4444-4444-8444-444444444444".into(),
                expected: TargetExpectationV2::ExpectedAbsent(hash(
                    b"frozen actual synthetic absent slot",
                )),
            }],
        )
        .unwrap();
        let owner = Arc::new(MappedOwner {
            revision: binding.mapping_revision.clone(),
            binding_digest: binding.digest.clone(),
            revision_valid: AtomicBool::new(true),
            absent_valid: AtomicBool::new(true),
            checks: AtomicUsize::new(0),
            revision_checks: AtomicUsize::new(0),
            receipt: f.models.receipt(OP),
        });
        let local_request = CaptureRequestV2 {
            selection: local.clone(),
            mcp_clients: BTreeMap::new(),
            assets_reviewed: with_non_model,
        };
        let root_revision =
            saved_objects_capture_v2::capture(&f.d, &f.h, &local_request, &NoCredentials)
                .unwrap()
                .root_revision;
        let actual = Arc::new(
            SavedCaptureProviderV2::new(
                f.d.clone(),
                f.h.clone(),
                serde_json::from_slice(&json(&local_request).unwrap()).unwrap(),
                Arc::new(NoCredentials),
            )
            .unwrap(),
        );
        let mapped =
            Arc::new(MappedCaptureProviderV2::new(actual, owner.clone(), binding.clone()).unwrap());
        // Exercise the actual Host preview and its dual-domain SourceGuard.
        let mut cloud_capture_scope = remote.clone();
        if absent_non_model {
            // Model/prefs/Skill use real cloud capture. The explicit public MCP
            // and extension settings below are validated by the real wrapper.
            cloud_capture_scope.manager.extensions = false;
            cloud_capture_scope.mcp_sources.clear();
        }
        let cloud_request = CaptureRequestV2 {
            selection: cloud_capture_scope,
            mcp_clients: BTreeMap::new(),
            assets_reviewed: with_non_model,
        };
        let mut cloud = saved_objects_capture_v2::capture(
            &cloud_root,
            &cloud_home,
            &cloud_request,
            &NoCredentials,
        )
        .unwrap()
        .objects
        .into_captured();
        if with_non_model {
            cloud.manager.preferences.as_mut().unwrap().theme = "dark".into();
        }
        if let Some(mcp) = mcp {
            cloud.manager.extensions = Some(Default::default());
            cloud.assets.mcps.push(mcp);
        }
        let cloud = SavedObjects::capture(cloud, Some(&ActualAuthorization)).unwrap();
        f.payload = file_objects_v2::encode(
            &cloud,
            OP,
            "2026-11-06T10:00:00Z",
            None,
            Some(&ActualAuthorization),
        )
        .unwrap()
        .into_bytes();
        let input = PayloadInput {
            head: CommitRef {
                operation_id: EXPORT.into(),
                digest: hash(b"cloud head"),
            },
            bytes: f.payload.to_vec(),
        };
        let preview = sync_host_v2::preview_semantics(
            mapped.as_ref(),
            remote.clone(),
            sync_host_v2::CloudMaterialV2::Valid(&input),
            &sync_host_v2::SessionSecretsV2::default(),
            None,
        )
        .unwrap();
        f.scope = preview.source_guard().scope_digest.clone();
        f.expected = preview.source_guard().saved_source_digest.clone();
        f.selection = remote.clone();
        f.capture_json = json(&local_request).unwrap();
        let current = CurrentSourceProviderV2::new(mapped, remote, root_revision).unwrap();
        let source = Arc::new(MappedSource {
            current,
            owner: owner.clone(),
            binding: binding.clone(),
        });
        Self {
            f,
            binding,
            owner,
            source,
        }
    }
    fn host(&self) -> SyncResult<LocalApplicationV2> {
        LocalApplicationV2::new_mapped(
            self.f.d.clone(),
            self.f.h.clone(),
            self.f.selection.clone(),
            serde_json::from_slice(&self.f.capture_json).unwrap(),
            self.f.scope.clone(),
            self.source.clone(),
            self.f.models.clone(),
            None,
            self.binding.clone(),
            self.owner.clone(),
        )
    }
}

#[test]
fn mapped_cloudnew_a_explicit_empty_local_applies_with_real_host_guard() {
    let m = MappedFixture::new(false);
    let host = m.host().unwrap();
    let request: CaptureRequestV2 = serde_json::from_slice(&m.f.capture_json).unwrap();
    assert!(request.selection.channels.is_empty());
    let raw = saved_objects_capture_v2::capture(&m.f.d, &m.f.h, &request, &NoCredentials).unwrap();
    assert_ne!(raw.source_guard.scope_digest, m.f.scope);
    assert_eq!(host.current_source().unwrap().scope_digest, m.f.scope);
    assert_eq!(host.target_digest(&m.f.scope).unwrap(), m.f.expected);
    m.f.apply(&host).unwrap();
    assert_eq!(host.progress(OP).unwrap(), Some(LocalProgressV2::Applied));
    let journal = host.load(OP).unwrap().unwrap();
    assert_eq!(
        journal.target_binding_digest.as_ref(),
        Some(&m.binding.digest)
    );
    assert!(!std::fs::read_to_string(host.path(OP).unwrap())
        .unwrap()
        .contains("remoteScope"));
}
#[test]
fn mapped_revision_or_absent_drift_before_apply_has_zero_business_writes() {
    for revision in [true, false] {
        let m = MappedFixture::new(false);
        let host = m.host().unwrap();
        if revision {
            m.owner.revision_valid.store(false, SeqCst);
        } else {
            m.owner.absent_valid.store(false, SeqCst);
        }
        assert_eq!(
            m.f.apply(&host).unwrap_err().code,
            ErrorCode::SyncPlanChanged
        );
        assert_eq!(m.f.models.applies.load(SeqCst), 0);
        assert!(!m.f.models.business().exists());
        assert_eq!(m.f.models.prepares.load(SeqCst), 0);
        assert!(host.progress(OP).unwrap().is_none());
    }
}
#[test]
fn mapped_completed_same_id_returns_history_without_current_or_absent_verification() {
    let m = MappedFixture::new(false);
    let host = m.host().unwrap();
    m.f.apply(&host).unwrap();
    let checks = m.owner.checks.load(SeqCst);
    let revisions = m.owner.revision_checks.load(SeqCst);
    let journal = std::fs::read(host.path(OP).unwrap()).unwrap();
    m.owner.revision_valid.store(false, SeqCst);
    m.owner.absent_valid.store(false, SeqCst);
    std::fs::write(m.f.models.business(), b"user edit after applied").unwrap();
    m.f.apply(&host).unwrap();
    assert_eq!(m.owner.checks.load(SeqCst), checks);
    assert_eq!(m.owner.revision_checks.load(SeqCst), revisions);
    assert_eq!(m.f.models.applies.load(SeqCst), 1);
    assert_eq!(std::fs::read(host.path(OP).unwrap()).unwrap(), journal);
    assert_eq!(
        std::fs::read(m.f.models.business()).unwrap(),
        b"user edit after applied"
    );
}
#[test]
fn mapped_after_model_receipt_continues_real_non_model_guards_without_original_absent() {
    let m = MappedFixture::new(true);
    let host = m.host().unwrap();
    m.f.models.cut_after_apply.store(true, SeqCst);
    assert!(m.f.apply(&host).is_err());
    let checks = m.owner.checks.load(SeqCst);
    m.f.apply(&host).unwrap();
    assert_eq!(m.f.theme(), "dark");
    assert_eq!(m.f.models.applies.load(SeqCst), 1);
    assert_eq!(m.owner.checks.load(SeqCst), checks);
    assert_eq!(host.progress(OP).unwrap(), Some(LocalProgressV2::Applied));
}
#[test]
fn mapped_changed_revision_after_model_receipt_retains_history_and_blocks_new_writes() {
    let m = MappedFixture::new(true);
    let host = m.host().unwrap();
    m.f.models.cut_after_apply.store(true, SeqCst);
    assert!(m.f.apply(&host).is_err());
    let theme = m.f.theme();
    let checks = m.owner.checks.load(SeqCst);
    let receipt = std::fs::read(m.f.models.receipt(OP)).unwrap();
    m.owner.revision_valid.store(false, SeqCst);
    assert_eq!(
        m.f.apply(&host).unwrap_err().code,
        ErrorCode::SyncPlanChanged
    );
    assert_eq!(m.f.theme(), theme);
    assert_eq!(m.f.models.applies.load(SeqCst), 1);
    assert_eq!(m.owner.checks.load(SeqCst), checks);
    assert_eq!(std::fs::read(m.f.models.receipt(OP)).unwrap(), receipt);
    assert_eq!(
        host.progress(OP).unwrap(),
        Some(LocalProgressV2::ModelsApplied)
    );
}
#[test]
fn mapped_constructor_checks_digest_domains_scope_and_old_constructor_remains_strict() {
    let m = MappedFixture::new(false);
    let create = |scope: String, binding: LocalTargetBindingV2| {
        LocalApplicationV2::new_mapped(
            m.f.d.clone(),
            m.f.h.clone(),
            m.f.selection.clone(),
            serde_json::from_slice(&m.f.capture_json).unwrap(),
            scope,
            m.source.clone(),
            m.f.models.clone(),
            None,
            binding,
            m.owner.clone(),
        )
    };
    assert!(create(hash(b"raw or wrong scope"), m.binding.clone()).is_err());
    let mut tampered = m.binding.clone();
    tampered.digest = hash(b"forged");
    assert!(create(m.f.scope.clone(), tampered).is_err());
    let mut local = m.binding.clone();
    local.local_scope.channels = local.remote_scope.channels.clone();
    assert!(create(m.f.scope.clone(), local).is_err());
    assert!(LocalApplicationV2::new(
        m.f.d.clone(),
        m.f.h.clone(),
        m.f.selection.clone(),
        serde_json::from_slice(&m.f.capture_json).unwrap(),
        m.f.scope.clone(),
        m.source.clone(),
        m.f.models.clone(),
        None
    )
    .is_err());
}
#[test]
fn mapped_same_id_changed_binding_digest_is_rejected_before_receipt_reuse() {
    let m = MappedFixture::new(false);
    let mut host = m.host().unwrap();
    m.f.apply(&host).unwrap();
    host.mapped.as_mut().unwrap().binding.digest = hash(b"another binding");
    assert_eq!(
        m.f.apply(&host).unwrap_err().code,
        ErrorCode::SyncOperationCollision
    );
    assert_eq!(m.f.models.applies.load(SeqCst), 1);
}

struct FrozenMappedSource {
    guard: SourceGuard,
    root: String,
}
impl SourceSnapshotProviderV2 for FrozenMappedSource {
    fn current(&self, _: &Path, _: &Path, _: &CaptureRequestV2) -> ModelResult<SourceGuard> {
        Ok(self.guard.clone())
    }
    fn root(&self, _: &Path, _: &Path, _: &CaptureRequestV2) -> ModelResult<String> {
        Ok(self.root.clone())
    }
    fn target(&self, _: &Path, _: &Path, _: &CaptureRequestV2) -> ModelResult<String> {
        Ok(self.guard.saved_source_digest.clone())
    }
}
struct DriftAfterPrepare {
    models: Arc<Models>,
    owner: Arc<MappedOwner>,
    revision: bool,
}
impl ModelApplyV2 for DriftAfterPrepare {
    fn prepare(
        &self,
        id: &str,
        objects: &SavedObjects,
        s: &SelectionV2,
        expected: &str,
    ) -> ModelResult<ModelApplyPlanHandle> {
        let h = self.models.prepare(id, objects, s, expected)?;
        if self.revision {
            self.owner.revision_valid.store(false, SeqCst);
        } else {
            self.owner.absent_valid.store(false, SeqCst);
        }
        Ok(h)
    }
    fn protect(&self, id: &str, h: &ModelApplyPlanHandle, b: bool) -> ModelResult<()> {
        self.models.protect(id, h, b)
    }
    fn apply_once(
        &self,
        id: &str,
        objects: &SavedObjects,
        h: &ModelApplyPlanHandle,
        b: bool,
    ) -> ModelResult<ModelApplyReceipt> {
        self.models.apply_once(id, objects, h, b)
    }
    fn status(&self, id: &str, h: &ModelApplyPlanHandle) -> ModelResult<ModelApplyStatus> {
        self.models.status(id, h)
    }
}
#[test]
fn mapped_owner_is_independent_of_capture_and_rechecked_before_protect() {
    for revision in [true, false] {
        let m = MappedFixture::new(true);
        let r: CaptureRequestV2 = serde_json::from_slice(&m.f.capture_json).unwrap();
        let source = Arc::new(FrozenMappedSource {
            guard: m.source.current(&m.f.d, &m.f.h, &r).unwrap(),
            root: m.source.root(&m.f.d, &m.f.h, &r).unwrap(),
        });
        let models = Arc::new(DriftAfterPrepare {
            models: m.f.models.clone(),
            owner: m.owner.clone(),
            revision,
        });
        let host = LocalApplicationV2::new_mapped(
            m.f.d.clone(),
            m.f.h.clone(),
            m.f.selection.clone(),
            r,
            m.f.scope.clone(),
            source,
            models,
            None,
            m.binding.clone(),
            m.owner.clone(),
        )
        .unwrap();
        let theme = m.f.theme();
        assert_eq!(
            m.f.apply(&host).unwrap_err().code,
            ErrorCode::SyncPlanChanged
        );
        assert_eq!(m.f.models.prepares.load(SeqCst), 1);
        assert_eq!(m.f.models.applies.load(SeqCst), 0);
        assert_eq!(m.f.theme(), theme);
        assert_eq!(host.progress(OP).unwrap(), Some(LocalProgressV2::Prepared));
    }
}

#[test]
fn mapped_empty_local_receives_full_remote_preferences_extensions_skill_and_mcp() {
    let m = MappedFixture::empty_local_full_remote();
    assert!(m.binding.local_scope == SelectionV2::default());
    assert!(m.binding.remote_scope.manager.preferences);
    assert!(m.binding.remote_scope.manager.extensions);
    assert_eq!(m.binding.remote_scope.skill_names, ["fixture"]);
    assert_eq!(m.binding.remote_scope.mcp_sources.len(), 1);
    let prefs = m.f.d.join(preferences::PREFERENCES_RELATIVE_PATH);
    let extensions =
        m.f.d
            .join(crate::modules::extensions::projection::CONFIG_RELATIVE_PATH);
    let skill = discovery::skills_store_dir(&m.f.d).join("fixture/SKILL.md");
    assert!(!prefs.exists());
    assert!(!extensions.exists());
    assert!(!skill.exists());
    assert!(portable_assets::stored_public(&m.f.d).unwrap().is_empty());
    // Ordinary same-domain construction still rejects this split authority.
    assert!(LocalApplicationV2::new(
        m.f.d.clone(),
        m.f.h.clone(),
        m.f.selection.clone(),
        serde_json::from_slice(&m.f.capture_json).unwrap(),
        m.f.scope.clone(),
        m.source.clone(),
        m.f.models.clone(),
        None
    )
    .is_err());
    let host = m.host().unwrap();
    m.f.apply(&host).unwrap();
    assert_eq!(host.progress(OP).unwrap(), Some(LocalProgressV2::Applied));
    assert_eq!(m.f.theme(), "dark");
    assert!(extensions.is_file());
    assert_eq!(
        std::fs::read(&skill).unwrap(),
        b"# Fixture\nOld instruction.\n"
    );
    let mcps = portable_assets::stored_public(&m.f.d).unwrap();
    assert_eq!(mcps.len(), 1);
    assert_eq!(mcps[0].name, "cloud-only-mcp");
    assert_eq!(
        mcps[0].value,
        json!({"command":"fixture-command","args":["--cloud"]})
    );
    let manager: Value =
        serde_json::from_slice(&std::fs::read(host.manager_path(OP)).unwrap()).unwrap();
    assert_eq!(manager["files"].as_array().unwrap().len(), 2);
    assert!(manager["files"]
        .as_array()
        .unwrap()
        .iter()
        .all(|f| f["before"].is_null()));
    let assets: Value =
        serde_json::from_slice(&std::fs::read(host.asset_path(OP)).unwrap()).unwrap();
    assert!(assets["skillsBefore"]["fixture"].is_null());
    assert_eq!(assets["storeBefore"], hash(b""));
    let journal = host.load(OP).unwrap().unwrap();
    assert_eq!(
        journal.target_binding_digest.as_ref(),
        Some(&m.binding.digest)
    );
    assert!(!journal.targets.unwrap().is_empty());
}

#[test]
fn mapped_filtered_non_model_scopes_need_exact_issued_binding_and_reviewed_context() {
    let m = MappedFixture::empty_local_full_remote();
    let create =
        |selection: SelectionV2, capture: CaptureRequestV2, binding: LocalTargetBindingV2| {
            LocalApplicationV2::new_mapped(
                m.f.d.clone(),
                m.f.h.clone(),
                selection,
                capture,
                m.f.scope.clone(),
                m.source.clone(),
                m.f.models.clone(),
                None,
                binding,
                m.owner.clone(),
            )
        };
    let request = || serde_json::from_slice::<CaptureRequestV2>(&m.f.capture_json).unwrap();
    let mut wider = m.f.selection.clone();
    wider.skill_names.push("not-issued".into());
    assert!(create(wider.clone(), request(), m.binding.clone()).is_err());
    let mut local = request();
    local.selection.manager.preferences = true;
    assert!(create(m.f.selection.clone(), local, m.binding.clone()).is_err());
    let mut unreviewed = request();
    unreviewed.assets_reviewed = false;
    assert!(create(m.f.selection.clone(), unreviewed, m.binding.clone()).is_err());
    // Even a freshly recomputed digest is not authority without owner issuance.
    let forged = LocalTargetBindingV2::freeze(
        m.binding.mapping_revision.clone(),
        m.binding.local_scope.clone(),
        wider.clone(),
        m.binding.direction.clone(),
        m.binding.slots.clone(),
    )
    .unwrap();
    let result = create(wider, request(), forged);
    assert_eq!(result.err().unwrap().code, ErrorCode::SyncPlanChanged);
    assert_eq!(m.f.models.prepares.load(SeqCst), 0);
    assert!(!m.f.models.business().exists());
}

struct CreatePreferencesAfterProtection {
    models: Arc<Models>,
    once: AtomicBool,
}
impl ModelApplyV2 for CreatePreferencesAfterProtection {
    fn prepare(
        &self,
        id: &str,
        objects: &SavedObjects,
        s: &SelectionV2,
        expected: &str,
    ) -> ModelResult<ModelApplyPlanHandle> {
        self.models.prepare(id, objects, s, expected)
    }
    fn protect(&self, id: &str, h: &ModelApplyPlanHandle, b: bool) -> ModelResult<()> {
        self.models.protect(id, h, b)?;
        if self.once.swap(false, SeqCst) {
            atomic_write(
                &self.models.d.join(preferences::PREFERENCES_RELATIVE_PATH),
                br#"{"theme":"light"}"#,
                0o600,
            )?;
        }
        Ok(())
    }
    fn apply_once(
        &self,
        id: &str,
        objects: &SavedObjects,
        h: &ModelApplyPlanHandle,
        b: bool,
    ) -> ModelResult<ModelApplyReceipt> {
        self.models.apply_once(id, objects, h, b)
    }
    fn status(&self, id: &str, h: &ModelApplyPlanHandle) -> ModelResult<ModelApplyStatus> {
        self.models.status(id, h)
    }
}
#[test]
fn mapped_non_model_absence_is_frozen_by_real_helper_not_inferred_from_empty_capture() {
    let m = MappedFixture::empty_local_full_remote();
    let models = Arc::new(CreatePreferencesAfterProtection {
        models: m.f.models.clone(),
        once: AtomicBool::new(true),
    });
    let host = LocalApplicationV2::new_mapped(
        m.f.d.clone(),
        m.f.h.clone(),
        m.f.selection.clone(),
        serde_json::from_slice(&m.f.capture_json).unwrap(),
        m.f.scope.clone(),
        m.source.clone(),
        models,
        None,
        m.binding.clone(),
        m.owner.clone(),
    )
    .unwrap();
    assert_eq!(
        m.f.apply(&host).unwrap_err().code,
        ErrorCode::SyncLocalChanged
    );
    assert_eq!(m.f.models.applies.load(SeqCst), 0);
    assert_eq!(m.f.theme(), "light");
    assert!(!discovery::skills_store_dir(&m.f.d).join("fixture").exists());
    assert!(portable_assets::stored_public(&m.f.d).unwrap().is_empty());
    assert_eq!(host.progress(OP).unwrap(), Some(LocalProgressV2::Protected));
    let manager = std::fs::read(host.manager_path(OP)).unwrap();
    assert_eq!(
        m.f.apply(&host).unwrap_err().code,
        ErrorCode::SyncLocalChanged
    );
    assert_eq!(std::fs::read(host.manager_path(OP)).unwrap(), manager);
    assert_eq!(m.f.models.applies.load(SeqCst), 0);
}
