use super::*;
#[path = "preparation/read_tests.rs"]
mod read_tests;
use crate::infrastructure::windows_client_originals024::{tree::tests::Fixture, v3::tests::plan};
use std::time::Duration;
struct Scene {
    fixture: Option<Fixture>,
    profile: PathBuf,
    data: PathBuf,
    home: PathBuf,
    installation: PathBuf,
    windows: PathBuf,
    plan_raw: Vec<u8>,
}
impl Scene {
    fn new() -> Self {
        let (fixture, plan, profile, _) = plan();
        let data = fixture.path().join("D");
        drop(native::create_private_directory(&data).unwrap());
        let home = fixture.path().join("H");
        drop(native::create_private_directory(&home).unwrap());
        let installation = fixture.path().join("I");
        drop(native::create_private_directory(&installation).unwrap());
        let official = data.join("official024");
        drop(native::create_private_directory(&official).unwrap());
        let windows = official.join("windows");
        drop(native::create_private_directory(&windows).unwrap());
        Self {
            fixture: Some(fixture),
            profile,
            data,
            home,
            installation,
            windows,
            plan_raw: encoded(&plan).unwrap(),
        }
    }
    fn inputs(
        &self,
    ) -> (
        PinnedDirectory,
        [PinnedDirectory; 4],
        RetainedPeer,
        Binding,
        StoragePlan,
    ) {
        let pins = [
            PinnedDirectory::open_private(&self.profile).unwrap(),
            PinnedDirectory::open_private(&self.data).unwrap(),
            PinnedDirectory::open_private(&self.home).unwrap(),
            PinnedDirectory::open_private(&self.installation).unwrap(),
        ];
        let directory =
            native::create_private_directory(&self.windows.join(uuid::Uuid::new_v4().to_string()))
                .unwrap();
        let peer = RetainedPeer::current().unwrap();
        let binding = Binding::from_native(
            &peer,
            &pins,
            hash::sha256_hex(b"synthetic source DATA, not original Owner"),
            hash::sha256_hex(b"synthetic current DATA"),
        )
        .unwrap();
        // This data plan is still checked against actual native Profile/tree IDs;
        // it does not create a NativeAction/Owner or OriginalSession capability.
        let plan: StoragePlan = serde_json::from_slice(&self.plan_raw).unwrap();
        (directory, pins, peer, binding, plan)
    }
    fn prefix(&self, fault: u8) -> (JournalPreparation, PathBuf) {
        let (directory, pins, peer, binding, plan) = self.inputs();
        let path = directory.path().to_path_buf();
        let mut prefix = JournalPreparation::default();
        prefix.fault = fault;
        prefix
            .enroll(directory, pins, peer, binding, plan, Vec::new(), None)
            .unwrap();
        (prefix, path)
    }
}
impl Drop for Scene {
    fn drop(&mut self) {
        if let Some(fixture) = self.fixture.take() {
            let path = fixture.path().to_path_buf();
            drop(fixture);
            if !std::thread::panicking() {
                assert!(
                    !path.exists(),
                    "exact successful native journal scene cleanup readback: {}",
                    path.display()
                );
                println!("r7-journal-cleanup-absent={}", path.display());
            }
        }
    }
}
#[test]
fn wl2_r7_journal_prefix_native_all_initializer_failures_keep_original_handles_and_no_retry() {
    for stage in [1, 2, 3, 4, 5, 6, 7, 8, 10, 11] {
        let scene = Scene::new();
        let (mut prefix, path) = scene.prefix(stage);
        let end = Instant::now() + Duration::from_secs(10);
        let epoch = uuid::Uuid::new_v4().to_string();
        let scope = uuid::Uuid::new_v4().to_string();
        let result = prefix.initialize(&epoch, &"c".repeat(64), &scope, end);
        assert!(result.is_err(), "actual injected stage{stage}");
        assert!(prefix.failed && !prefix.ready);
        assert!(
            prefix.take_complete().is_err(),
            "failed prefix cannot become complete Journal"
        );
        let manifest = path.join("manifest.json");
        let current = path.join("current.json");
        for file in [&manifest, &current].into_iter().filter(|p| p.exists()) {
            assert!(
                OpenOptions::new().write(true).open(file).is_err(),
                "external writer must be blocked stage{stage}"
            );
            assert!(
                std::fs::remove_file(file).is_err(),
                "original initializer object must deny delete stage{stage}"
            );
        }
        if stage == 2 {
            assert!(prefix.manifest_open.is_some());
            assert_eq!(std::fs::metadata(&manifest).unwrap().len(), 0);
        }
        if stage == 3 {
            assert!(prefix.manifest_open.is_some());
            assert_eq!(
                std::fs::read(&manifest).unwrap(),
                *prefix.manifest_raw.as_ref().unwrap()
            );
        }
        if stage == 4 {
            assert!(prefix.manifest_record.is_some() && prefix.current_open.is_none());
        }
        if (5..=7).contains(&stage) {
            assert!(prefix.manifest_record.is_some() && prefix.current_open.is_some());
        }
        if stage == 5 {
            assert_eq!(std::fs::metadata(&current).unwrap().len(), 0);
        }
        if stage == 6 || stage == 7 {
            assert_eq!(
                std::fs::read(&current).unwrap(),
                *prefix.current_expected.as_ref().unwrap()
            );
        }
        if stage == 8 {
            assert!(prefix.completed.as_ref().unwrap().poisoned);
        }
        if stage == 10 {
            let expected = prefix.manifest_raw.as_ref().unwrap();
            assert_eq!(
                std::fs::read(&manifest).unwrap(),
                expected[..expected.len() / 2]
            );
            assert!(prefix.manifest_open.is_some());
        }
        if stage == 11 {
            let expected = prefix.current_expected.as_ref().unwrap();
            assert_eq!(
                std::fs::read(&current).unwrap(),
                expected[..expected.len() / 2]
            );
            assert!(prefix.current_open.is_some() && prefix.manifest_record.is_some());
        }
        let membership = std::fs::read_dir(&path)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect::<Vec<_>>();
        assert!(prefix
            .initialize(&epoch, &"c".repeat(64), &scope, end)
            .is_err());
        assert_eq!(
            std::fs::read_dir(&path)
                .unwrap()
                .map(|e| e.unwrap().file_name())
                .collect::<Vec<_>>(),
            membership
        );
        assert!(std::fs::rename(&path, scene.windows.join("must-not-replace")).is_err());
        drop(prefix);
        assert_eq!(std::fs::read_dir(&path).unwrap().count(), membership.len());
        println!("r7 actual native initializer stage{stage}: exact original handles retained, created bytes/prefix preserved, writer/delete/replay refused, no NativeAction/restore/Closed grant");
        drop(scene);
    }
}
#[test]
fn wl2_r7_journal_prefix_native_v3_bytes_match_reference_and_records_keep_deny_write_delete() {
    let scene = Scene::new();
    let end = Instant::now() + Duration::from_secs(10);
    let epoch = uuid::Uuid::new_v4().to_string();
    let scope = uuid::Uuid::new_v4().to_string();
    let claim = "c".repeat(64);
    let (directory, pins, peer, binding, plan) = scene.inputs();
    let reference = Journal::create(
        directory, pins, peer, binding, plan, &epoch, &claim, &scope, end,
    )
    .unwrap();
    let (mut prefix, path) = scene.prefix(0);
    prefix.initialize(&epoch, &claim, &scope, end).unwrap();
    // A component initialization with no original Owner never publishes the
    // production take_complete grant; tests inspect only original native DATA.
    assert!(prefix.take_complete().is_err());
    let journal = prefix.completed.as_mut().unwrap();
    journal.check(end).unwrap();
    let mut expected: Manifest = serde_json::from_slice(&reference.manifest_record.raw).unwrap();
    expected.session_id = journal.manifest.session_id.clone();
    let expected_manifest = encoded(&expected).unwrap();
    assert_eq!(journal.manifest_record.raw, expected_manifest);
    let progress = Progress::new(
        &expected.storage_plan,
        &expected.session_id,
        &hash::sha256_hex(&expected_manifest),
        &reference.progress.binding,
        &epoch,
        &claim,
        &scope,
    )
    .unwrap();
    let mut expected: Current = serde_json::from_slice(&reference.current_raw).unwrap();
    expected.session_id = journal.manifest.session_id.clone();
    expected.manifest_digest = progress.manifest.clone();
    expected.event_digest = progress.tip.clone();
    expected.previous_event_digest = progress.tip;
    assert_eq!(journal.current_raw, encoded(&expected).unwrap());
    for file in [path.join("manifest.json"), path.join("current.json")] {
        assert!(OpenOptions::new().write(true).open(&file).is_err());
        assert!(std::fs::remove_file(&file).is_err());
    }
    assert!(matches!(
        journal.manifest_record.file,
        RecordFile::Original { .. }
    ));
    assert_eq!(
        journal.manifest_record.file.read().unwrap(),
        journal.manifest_record.raw
    );
    assert_eq!(journal.progress.sequence, 0);
    assert!(journal.progress.closed.is_none());
    drop(prefix);
    drop(reference);
    drop(scene);
}
#[test]
fn wl2_r7_journal_prefix_native_expired_before_create_and_existing_namespace_refuse_unchanged() {
    let scene = Scene::new();
    let (mut prefix, path) = scene.prefix(0);
    assert!(prefix
        .initialize(
            &uuid::Uuid::new_v4().to_string(),
            &"c".repeat(64),
            &uuid::Uuid::new_v4().to_string(),
            Instant::now()
        )
        .is_err());
    assert_eq!(std::fs::read_dir(&path).unwrap().count(), 0);
    assert!(prefix.directory.is_some());
    assert!(prefix.failed);
    drop(prefix);
    let (mut prefix, path) = scene.prefix(0);
    let old = path.join("old-current.json");
    let held = PrivateRecord::create(&old, b"preserve this existing namespace; no adoption", 1024)
        .unwrap();
    let end = Instant::now() + Duration::from_secs(10);
    assert!(prefix
        .initialize(
            &uuid::Uuid::new_v4().to_string(),
            &"c".repeat(64),
            &uuid::Uuid::new_v4().to_string(),
            end
        )
        .is_err());
    assert_eq!(
        std::fs::read(&old).unwrap(),
        b"preserve this existing namespace; no adoption"
    );
    assert!(!path.join("manifest.json").exists() && !path.join("current.json").exists());
    drop(held);
    drop(prefix);
    drop(scene);
}
