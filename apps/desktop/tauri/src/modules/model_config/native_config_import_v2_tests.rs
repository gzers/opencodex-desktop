use super::*;
use std::fs;

struct Fixture {
    _temp: tempfile::TempDir,
    root: std::path::PathBuf,
    home: std::path::PathBuf,
    source: std::path::PathBuf,
    original: Vec<u8>,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let root = base.join("desktop");
        let home = base.join("home");
        fs::create_dir_all(root.join("manager-state")).unwrap();
        fs::create_dir_all(&home).unwrap();
        let original = br#"{ "port": 41234, "oldGlobal": true, "providers": {"Old": {"adapter":"openai-chat","baseUrl":"https://old.invalid/v1","models":["old"]}} }"#.to_vec();
        fs::write(home.join("config.json"), &original).unwrap();
        let source = base.join("import.json");
        fs::write(&source, br#"{"port":51234,"defaultProvider":"New","providers":{"New":{"adapter":"openai-chat","baseUrl":"https://new.invalid/v1","models":["new"],"apiKey":"fixture-only","apiKeyPool":[{"id":"one","key":"fixture-first"},{"id":"two","key":"fixture-second"}],"headers":{"X-Auth":"fixture-header"}}},"globalFromSource":{"keep":true}}"#).unwrap();
        Self {
            _temp: temp,
            root,
            home,
            source,
            original,
        }
    }
    fn select(&self) -> Selection {
        prepare(&self.root, &self.home, &self.source, |_| Ok(())).unwrap()
    }
    fn unchanged(&self) {
        assert_eq!(
            fs::read(self.home.join("config.json")).unwrap(),
            self.original
        );
    }
    fn no_backups(&self) {
        assert!(!self.root.join("backups").exists());
    }
}

#[test]
fn complete_replace_preserves_port_and_mode_and_exact_original_backup() {
    let f = Fixture::new();
    let settings = f.root.join("manager-state/runtime-port-settings.json");
    let mode =
        serde_json::to_vec(&json!({"schemaVersion":1,"home":f.home,"mode":"auto","port":41234}))
            .unwrap();
    fs::write(&settings, &mode).unwrap();
    let initial = snapshot(&f.root, &f.home).unwrap();
    begin_draft(
        &f.root,
        &f.home,
        ReadRequestV2 {
            expected_source: initial.source_digest,
            expected_library: initial.library_digest,
            expected_draft: None,
        },
    )
    .unwrap();
    let plan = prepare(&f.root, &f.home, &f.source, |bytes| {
        let raw: Value = serde_json::from_slice(bytes).unwrap();
        assert!(matches!(raw["port"].as_u64(), Some(51234 | 41234)));
        Ok(())
    })
    .unwrap();
    f.unchanged();
    f.no_backups();
    let outcome = apply(&f.root, &f.home, plan, || Ok(())).unwrap();
    assert!(outcome.changed);
    assert_eq!(outcome.port, 41234);
    assert!(!outcome.external_auth);
    assert_eq!(fs::read(settings).unwrap(), mode);
    let mut expected: Value = serde_json::from_slice(&fs::read(&f.source).unwrap()).unwrap();
    expected["port"] = json!(41234);
    assert_eq!(read_raw(&f.home).unwrap().0, expected);
    let backups = safety::scan(&f.root.join("backups")).unwrap();
    let backup = backups
        .iter()
        .find(|b| Some(&b.manifest.backup_id) == outcome.backup_id.as_ref())
        .unwrap();
    assert_eq!(backup.manifest.action, BackupAction::Import);
    assert!(backup::verify_backup(backup).unwrap());
    assert_eq!(
        fs::read(backup.directory.join("config.json")).unwrap(),
        f.original
    );
    let state = snapshot(&f.root, &f.home).unwrap();
    assert!(state.draft.is_none());
    assert_eq!(state.saved_policy.channels[0].label, "New");
    assert!(!serde_json::to_string(&state)
        .unwrap()
        .contains("fixture-first"));
}

#[test]
fn reject_conflicting_source_or_target_without_backup_or_overwrite() {
    for source_changed in [false, true] {
        let f = Fixture::new();
        let plan = f.select();
        let target = f.home.join("config.json");
        let changed = if source_changed { &f.source } else { &target };
        let mut raw: Value = serde_json::from_slice(&fs::read(changed).unwrap()).unwrap();
        raw["changed"] = json!(true);
        fs::write(changed, serde_json::to_vec(&raw).unwrap()).unwrap();
        let before = fs::read(&target).unwrap();
        assert_eq!(
            apply(&f.root, &f.home, plan, || Ok(())).err().unwrap().code,
            "native_import_stale"
        );
        assert_eq!(fs::read(target).unwrap(), before);
        f.no_backups();
    }
}

#[test]
fn invalid_selection_or_unproved_stopped_service_leaves_config_and_backups_alone() {
    let f = Fixture::new();
    assert_eq!(
        prepare(&f.root, &f.home, &f.source, |_| Err(
            crate::errors::AppError::NotConfigured
        ))
        .err()
        .unwrap()
        .code,
        "native_import_config_invalid"
    );
    let plan = f.select();
    assert_eq!(
        apply(&f.root, &f.home, plan, || Err(
            crate::errors::AppError::NotConfigured
        ))
        .err()
        .unwrap()
        .code,
        "native_import_runtime_not_stopped"
    );
    f.unchanged();
    f.no_backups();
    fs::write(&f.source, br#"{"port":1,"port":2}"#).unwrap();
    assert_eq!(
        prepare(&f.root, &f.home, &f.source, |_| panic!(
            "invalid JSON must not reach runtime validator"
        ))
        .err()
        .unwrap()
        .code,
        "native_import_config_invalid"
    );
}

#[test]
fn backup_failure_and_unfinished_transaction_cannot_overwrite_config() {
    let f = Fixture::new();
    let plan = f.select();
    fs::write(f.root.join("backups"), b"not a directory").unwrap();
    assert_eq!(
        apply(&f.root, &f.home, plan, || Ok(())).err().unwrap().code,
        "native_import_backup_failed"
    );
    f.unchanged();
    let marker = f.root.join("manager-state/saved-file-save-pending.json");
    fs::write(&marker, b"must not recover during selection").unwrap();
    assert_eq!(
        prepare(&f.root, &f.home, &f.source, |_| Ok(()))
            .err()
            .unwrap()
            .code,
        "model_transaction_pending"
    );
    assert_eq!(
        fs::read(marker).unwrap(),
        b"must not recover during selection"
    );
    f.unchanged();
}

#[test]
fn source_change_during_last_service_check_cannot_publish_candidate() {
    let f = Fixture::new();
    let plan = f.select();
    let probes = std::cell::Cell::new(0);
    let err = apply(&f.root, &f.home, plan, || {
        probes.set(probes.get() + 1);
        if probes.get() == 2 {
            fs::write(&f.source, b"changed while checking service").unwrap();
        }
        Ok(())
    })
    .err()
    .unwrap();
    assert_eq!(err.code, "native_import_stale");
    f.unchanged();
    let records = safety::scan(&f.root.join("backups")).unwrap();
    assert_eq!(records.len(), 1);
    assert!(backup::verify_backup(&records[0]).unwrap());
}

#[test]
fn no_change_or_cancel_never_creates_a_backup() {
    let f = Fixture::new();
    drop(f.select());
    f.unchanged();
    f.no_backups();
    fs::write(&f.source, &f.original).unwrap();
    let outcome = apply(&f.root, &f.home, f.select(), || {
        panic!("no-op needs no stopped service check")
    })
    .unwrap();
    assert!(!outcome.changed && outcome.backup_id.is_none());
    f.unchanged();
    f.no_backups();
}

impl Fixture {
    fn import(&self) -> String {
        apply(&self.root, &self.home, self.select(), || Ok(()))
            .unwrap()
            .backup_id
            .unwrap()
    }
    fn select_backup(&self, id: &str) -> (Selection, RollbackPreview) {
        prepare_rollback(&self.root, &self.home, id, |bytes| {
            assert_eq!(bytes, self.original);
            Ok(())
        })
        .unwrap()
    }
}

#[test]
fn rollback_restores_exact_bytes_and_port_and_protects_current_file() {
    let f = Fixture::new();
    let id = f.import();
    let target = f.home.join("config.json");
    let mut changed = read_raw(&f.home).unwrap().0;
    changed["port"] = json!(43456);
    let current = serde_json::to_vec(&changed).unwrap();
    fs::write(&target, &current).unwrap();
    let settings = f.root.join("manager-state/runtime-port-settings.json");
    let mode =
        serde_json::to_vec(&json!({"schemaVersion":1,"home":f.home,"mode":"auto","port":43456}))
            .unwrap();
    fs::write(&settings, &mode).unwrap();
    let (selected, preview) = f.select_backup(&id);
    assert!(selected.is_rollback());
    assert_eq!((preview.current_port, preview.backup_port), (43456, 41234));
    assert_eq!(preview.target_path, target);
    assert_eq!(fs::read(&target).unwrap(), current);
    assert_eq!(list_backups(&f.root, &f.home).unwrap().len(), 1);
    let out = apply(&f.root, &f.home, selected, || Ok(())).unwrap();
    assert!(out.changed);
    f.unchanged();
    assert_eq!(out.port, 41234);
    assert_eq!(fs::read(settings).unwrap(), mode);
    assert_eq!(
        port_snapshot(&f.root, &f.home).unwrap().mode,
        port::Mode::Fixed
    );
    let new_id = out.backup_id.unwrap();
    let (_, _, protection) = backup_source(&f.root, &f.home, &new_id).unwrap();
    assert_eq!(protection.as_slice(), current);
    let listed = list_backups(&f.root, &f.home).unwrap();
    assert_eq!(listed.len(), 2);
    assert!(listed
        .iter()
        .any(|b| b.id == new_id && b.bytes == current.len() as u64));
    let state = snapshot(&f.root, &f.home).unwrap();
    assert_eq!(state.saved_policy.channels[0].label, "Old");
}

#[test]
fn rollback_rejects_other_home_and_stale_preview_on_either_side() {
    for tamper_backup in [false, true] {
        let f = Fixture::new();
        let id = f.import();
        let other = f.home.with_file_name("other-home");
        fs::create_dir(&other).unwrap();
        assert!(list_backups(&f.root, &other).unwrap().is_empty());
        for (home, requested) in [(&other, id.as_str()), (&f.home, "../import")] {
            assert_eq!(
                prepare_rollback(&f.root, home, requested, |_| panic!("invalid source"))
                    .err()
                    .unwrap()
                    .code,
                "native_rollback_backup_invalid"
            );
        }
        let (selected, _) = f.select_backup(&id);
        let target = f.home.join("config.json");
        if tamper_backup {
            let (_, path, _) = backup_source(&f.root, &f.home, &id).unwrap();
            fs::write(path, vec![b'x'; f.original.len()]).unwrap();
        } else {
            let mut bytes = fs::read(&target).unwrap();
            bytes.push(b' ');
            fs::write(&target, bytes).unwrap();
        }
        let expected = fs::read(&target).unwrap();
        assert_eq!(
            apply(&f.root, &f.home, selected, || panic!("stale preview"))
                .err()
                .unwrap()
                .code,
            "native_import_stale"
        );
        assert_eq!(fs::read(&target).unwrap(), expected);
        assert_eq!(list_backups(&f.root, &f.home).unwrap().len(), 1);
    }
}

#[test]
fn rollback_preview_cancel_and_identical_bytes_do_not_write() {
    let f = Fixture::new();
    let id = f.import();
    let target = f.home.join("config.json");
    let current = fs::read(&target).unwrap();
    drop(f.select_backup(&id));
    assert_eq!(fs::read(&target).unwrap(), current);
    fs::write(&target, &f.original).unwrap();
    let (selected, _) = f.select_backup(&id);
    let outcome = apply(&f.root, &f.home, selected, || panic!("unchanged bytes")).unwrap();
    assert!(!outcome.changed && outcome.backup_id.is_none());
    f.unchanged();
    assert_eq!(list_backups(&f.root, &f.home).unwrap().len(), 1);
}

#[cfg(unix)]
#[test]
fn rollback_rejects_backup_with_broadened_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    let id = f.import();
    let (_, path, _) = backup_source(&f.root, &f.home, &id).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(
        prepare_rollback(&f.root, &f.home, &id, |_| panic!("unprotected source"))
            .err()
            .unwrap()
            .code,
        "native_rollback_backup_invalid"
    );
}
