//! Existing authenticated FileOwner and canonical Store records, not routing mocks.
use super::*;
use crate::modules::model_config::{
    file_transfer_v2::FileRecoveryKeysV2,
    objects_overlay_v2::{ManagerSelectionV2, SelectionV2, UsageV2},
    provider_auth::PortableAuthorization,
    saved_objects_capture_v2::{CaptureCredentialsV2, CaptureRequestV2, SelectedKeySourceV2},
    schema_v2::digest,
    sync_v2::{
        store::{plan_digest, ReceiveRecord, Store},
        types::*,
    },
};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use zeroize::Zeroizing;
const O: &str = "11111111-1111-4111-8111-111111111111";
const OTHER: &str = "22222222-2222-4222-8222-222222222222";
struct NoSecrets;
impl CaptureCredentialsV2 for NoSecrets {
    fn channel_key(&self, _: &Path, _: &SelectedKeySourceV2<'_>) -> ModelResult<Zeroizing<String>> {
        panic!("unexpected credential access")
    }
    fn authorization(
        &self,
        _: &Path,
        _: &str,
        _: &serde_json::Value,
        _: &serde_json::Value,
    ) -> ModelResult<PortableAuthorization> {
        panic!("unexpected authorization access")
    }
    fn webdav_password(&self, _: &Path, _: &str) -> ModelResult<Zeroizing<String>> {
        panic!("unexpected password access")
    }
}
struct MemoryKey(Arc<AtomicBool>);
impl FileRecoveryKeysV2 for MemoryKey {
    fn key(&self, _: &Path, _: &Path, create: bool) -> ModelResult<Zeroizing<[u8; 32]>> {
        assert!(
            !create || !self.0.load(Ordering::SeqCst),
            "routing tried to create a recovery key"
        );
        Ok(Zeroizing::new([83; 32]))
    }
}
struct Fixture {
    _temp: tempfile::TempDir,
    d: PathBuf,
    h: PathBuf,
    file: FileTransferOwnerV2,
    service: SyncServiceV2,
    store: Store,
    readonly: Arc<AtomicBool>,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let d = temp.path().join("D");
        let h = temp.path().join("H");
        std::fs::create_dir_all(d.join("manager-state")).unwrap();
        std::fs::create_dir(&h).unwrap();
        std::fs::write(h.join("config.json"), br#"{"providers":{}}"#).unwrap();
        let preferences = crate::modules::preferences::document_from_preferences(
            &crate::modules::preferences::Preferences::default(),
        );
        std::fs::write(
            d.join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH),
            serde_json::to_vec(&preferences).unwrap(),
        )
        .unwrap();
        let readonly = Arc::new(AtomicBool::new(false));
        let file = FileTransferOwnerV2::new(d.clone(), h.clone(), Arc::new(NoSecrets))
            .unwrap()
            .with_recovery_keys(Arc::new(MemoryKey(readonly.clone())));
        // Constructs production owners/transport but issues no network request,
        // selects no endpoint and reads no credential or payload.
        let service = SyncServiceV2::production(d.clone(), h.clone()).unwrap();
        let store = Store::open(&d.join("manager-state/sync-v2-transactions")).unwrap();
        Self {
            _temp: temp,
            d,
            h,
            file,
            service,
            store,
            readonly,
        }
    }
    fn resolve(&mut self, id: &str) -> ModelResult<Option<ResultOwnerV2>> {
        self.readonly.store(true, Ordering::SeqCst);
        resolve(&self.d, &self.h, id, &mut self.file, &self.service)
    }
    fn export(&mut self) -> String {
        self.file
            .prepare_export(
                CaptureRequestV2 {
                    selection: SelectionV2 {
                        usage: UsageV2::LocalFile,
                        manager: ManagerSelectionV2 {
                            preferences: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    mcp_clients: Default::default(),
                    assets_reviewed: false,
                },
                None,
            )
            .unwrap()
            .id
    }
    fn receive(&self, id: &str, recovery: bool) {
        let mut record = ReceiveRecord {
            download: None,
            recovery: None,
            payload_handle: OTHER.into(),
            payload_schema: crate::modules::model_config::file_objects_v2::supported_schema(),
            root_revision: digest(b"root"),
            connection_revision: digest(b"connection"),
            fact: None,
            receipt: None,
        };
        if recovery {
            let mut plan = RecoveryPlan {
                dto_version: 1,
                recovery_plan_id: id.into(),
                plan_digest: String::new(),
                kind: RecoveryPlanKind::RecoveryDownload,
                source_binding_id: digest(b"binding"),
                generation: GenerationRef {
                    generation_id: OTHER.into(),
                    digest: digest(b"generation"),
                },
                head: CommitRef {
                    operation_id: OTHER.into(),
                    digest: digest(b"head"),
                },
                payload_digest: digest(b"payload"),
                payload_schema: record.payload_schema.clone(),
                scope_digest: digest(b"scope"),
                target_digest: Some(digest(b"target")),
                expected_view_digest: digest(b"view"),
                backup_before_update: true,
                prepared_at: "2026-11-07T00:00:00Z".into(),
                expires_at: "2030-11-07T00:00:00Z".into(),
            };
            plan.plan_digest = plan_digest(&plan).unwrap();
            record.recovery = Some(plan);
        } else {
            let mut plan = DownloadPlan {
                dto_version: 1,
                operation_id: id.into(),
                plan_digest: String::new(),
                binding_id: digest(b"binding"),
                generation: GenerationRef {
                    generation_id: OTHER.into(),
                    digest: digest(b"generation"),
                },
                head: CommitRef {
                    operation_id: OTHER.into(),
                    digest: digest(b"head"),
                },
                payload_digest: digest(b"payload"),
                scope_digest: digest(b"scope"),
                target_digest: digest(b"target"),
                expected_view_digest: digest(b"view"),
                backup_before_update: true,
                prepared_at: "2026-11-07T00:00:00Z".into(),
                expires_at: "2030-11-07T00:00:00Z".into(),
            };
            plan.plan_digest = plan_digest(&plan).unwrap();
            record.download = Some(plan);
        }
        self.store.save_receive(id, &record).unwrap();
    }
}
fn tree(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn walk(base: &Path, at: &Path, out: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        for entry in std::fs::read_dir(at).unwrap() {
            let p = entry.unwrap().path();
            let m = std::fs::symlink_metadata(&p).unwrap();
            let key = p.strip_prefix(base).unwrap().to_owned();
            if m.is_dir() {
                out.insert(key, None);
                walk(base, &p, out);
            } else {
                assert!(!m.file_type().is_symlink());
                out.insert(key, Some(std::fs::read(p).unwrap()));
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}
#[test]
fn all_actual_stores_positively_missing_is_none_and_readonly() {
    let mut f = Fixture::new();
    let before = tree(f._temp.path());
    assert_eq!(f.resolve(O).unwrap(), None);
    assert_eq!(tree(f._temp.path()), before);
    assert!(f.resolve("not-a-domain-or-uuid").is_err());
}
#[test]
fn authenticated_file_record_owns_original_id_without_reopen_or_key_creation() {
    let mut f = Fixture::new();
    let id = f.export();
    let before = tree(f._temp.path());
    assert_eq!(
        f.resolve(&id).unwrap().unwrap(),
        ResultOwnerV2 {
            operation_id: id.clone(),
            kind: ResultOwnerKindV2::File
        }
    );
    assert_eq!(tree(f._temp.path()), before);
    let text = serde_json::to_string(&f.resolve(&id).unwrap().unwrap()).unwrap();
    assert_eq!(text, format!(r#"{{"operationId":"{id}","kind":"file"}}"#));
}
#[test]
fn actual_download_and_recovery_records_route_without_reading_payload_or_network() {
    let mut f = Fixture::new();
    f.receive(O, false);
    f.receive(OTHER, true);
    let before = tree(f._temp.path());
    assert_eq!(f.resolve(O).unwrap().unwrap().kind, ResultOwnerKindV2::Sync);
    assert_eq!(
        f.resolve(OTHER).unwrap().unwrap().kind,
        ResultOwnerKindV2::Recovery
    );
    assert_eq!(tree(f._temp.path()), before);
}
#[test]
fn valid_uuid_owned_by_two_domains_is_ambiguous_not_first_match() {
    let mut f = Fixture::new();
    let id = f.export();
    f.receive(&id, false);
    let before = tree(f._temp.path());
    assert_eq!(f.resolve(&id).unwrap_err().code, "result_owner_ambiguous");
    assert_eq!(tree(f._temp.path()), before);
}
#[test]
fn corrupted_file_or_sync_record_never_falls_back_to_another_owner() {
    let mut f = Fixture::new();
    let id = f.export();
    f.receive(&id, true);
    let sealed =
        f.d.join("manager-state/file-transfer-v2")
            .join(format!("{id}.sealed"));
    std::fs::write(&sealed, b"corrupt authenticated record").unwrap();
    let before = tree(f._temp.path());
    assert!(f.resolve(&id).is_err());
    assert_eq!(tree(f._temp.path()), before);
    let mut g = Fixture::new();
    let id = g.export();
    std::fs::write(
        g.d.join("manager-state/sync-v2-transactions")
            .join(format!("{id}.upload.json")),
        b"bad canonical journal",
    )
    .unwrap();
    assert_eq!(g.resolve(&id).unwrap_err().code, "result_record_invalid");
}
#[test]
fn actual_record_wrong_id_and_plan_digest_are_errors_not_missing() {
    let mut f = Fixture::new();
    f.receive(O, false);
    let mut r = f.store.receive(O).unwrap();
    r.download.as_mut().unwrap().operation_id = OTHER.into();
    f.store.save_receive(O, &r).unwrap();
    assert!(f.resolve(O).is_err());
    let mut g = Fixture::new();
    g.receive(O, true);
    let mut r = g.store.receive(O).unwrap();
    r.recovery.as_mut().unwrap().plan_digest = digest(b"wrong");
    g.store.save_receive(O, &r).unwrap();
    assert!(g.resolve(O).is_err());
}
#[test]
fn corrupt_unrelated_restore_and_record_io_are_not_positive_absence() {
    let mut f = Fixture::new();
    let id = f.export();
    let plans = f.d.join("manager-state/transfer-scope-v2-restores");
    std::fs::create_dir_all(&plans).unwrap();
    std::fs::write(
        plans.join(format!("{OTHER}.plan.json")),
        b"corrupt scope plan",
    )
    .unwrap();
    let before = tree(f._temp.path());
    assert!(f.resolve(&id).is_err());
    assert_eq!(tree(f._temp.path()), before);
    let mut g = Fixture::new();
    std::fs::create_dir(
        g.d.join("manager-state/sync-v2-transactions")
            .join(format!("{O}.upload.json")),
    )
    .unwrap();
    assert!(g.resolve(O).is_err());
}
#[test]
fn wrong_fixed_home_is_rejected_instead_of_routing_another_context() {
    let mut f = Fixture::new();
    let id = f.export();
    let wrong = f._temp.path().join("wrongH");
    std::fs::create_dir(&wrong).unwrap();
    assert_eq!(
        resolve(&f.d, &wrong, &id, &mut f.file, &f.service)
            .unwrap_err()
            .code,
        "result_owner_context"
    );
}

#[test]
fn actual_preferences_import_issues_scope_restore_owner_without_result_side_effects() {
    use crate::modules::model_config::{file_objects_v2, file_transfer_v2::ImportBindingsV2};
    let mut f = Fixture::new();
    let export_id = f.export();
    let bytes = f.file.bytes(&export_id).unwrap().to_vec();
    let mut captured =
        file_objects_v2::decode(&bytes, None, &file_objects_v2::supported_schema(), None)
            .unwrap()
            .objects
            .into_captured();
    captured.manager.preferences.as_mut().unwrap().theme = "dark".into();
    let objects = file_objects_v2::SavedObjects::capture(captured, None).unwrap();
    let bytes = file_objects_v2::encode(&objects, &export_id, "2026-11-07T00:00:00Z", None, None)
        .unwrap()
        .into_bytes()
        .to_vec();
    let selection = SelectionV2 {
        usage: UsageV2::LocalFile,
        manager: ManagerSelectionV2 {
            preferences: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let import = f
        .file
        .prepare_import(bytes, None, Some(selection), ImportBindingsV2::default())
        .unwrap();
    assert!(import.can_confirm, "{:?}", import.reason_code);
    f.file
        .confirm_import(&import.operation_id, &import.plan_digest, Some(true))
        .unwrap();
    let restore =
        restore::preview(&f.d, &f.h, &import.operation_id, &["preferences".into()]).unwrap();
    let before = tree(f._temp.path());
    let found = f.resolve(&restore.restore_id).unwrap().unwrap();
    assert_eq!(found.operation_id, restore.restore_id);
    assert_eq!(found.kind, ResultOwnerKindV2::ScopeRestore);
    assert_eq!(
        serde_json::to_value(found).unwrap()["kind"],
        "scope_restore"
    );
    assert_eq!(tree(f._temp.path()), before);
}
