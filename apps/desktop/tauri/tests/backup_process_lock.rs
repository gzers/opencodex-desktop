//! Real subprocess verification of the cooperative W2 boundary, on temp roots only.
//! This does not establish non-cooperating writer safety or native performance.
use chrono::{Duration as Days, Utc};
use opencodex_desktop_lib::{
    modules::{
        backup::{manager, policy::CleanupPolicy, MAX_PER_ACTION},
        data_root,
        preferences::{Preferences, PreferencesStore, PREFERENCES_RELATIVE_PATH},
    },
    types::upgrade::BackupCleanupPolicy,
};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

struct Worker(Child);
impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
impl Worker {
    fn spawn(root: &Path, mode: &str, id: &str) -> Self {
        let log = fs::File::create(root.join("child.log")).unwrap();
        Self(
            Command::new(std::env::current_exe().unwrap())
                .args(["--ignored", "--exact", "w2_child", "--nocapture"])
                .env("OCX_W2_TEST_ROOT", root)
                .env("OCX_W2_TEST_MODE", mode)
                .env("OCX_W2_TEST_ID", id)
                .stdout(Stdio::from(log.try_clone().unwrap()))
                .stderr(Stdio::from(log))
                .spawn()
                .unwrap(),
        )
    }
    fn marker(&mut self, root: &Path, marker: &str) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !root.join(marker).exists() {
            assert!(
                self.0.try_wait().unwrap().is_none(),
                "child exited before {marker}: {}",
                fs::read_to_string(root.join("child.log")).unwrap()
            );
            assert!(Instant::now() < deadline, "child {marker} timed out");
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    fn success(&mut self, root: &Path) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = self.0.try_wait().unwrap() {
                assert!(
                    status.success(),
                    "child failed: {}",
                    fs::read_to_string(root.join("child.log")).unwrap()
                );
                assert!(root.join("done").exists());
                break;
            }
            assert!(Instant::now() < deadline, "child completion timed out");
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    data_root::initialize(root.path()).unwrap();
    PreferencesStore::new(root.path())
        .save(&Preferences::default())
        .unwrap();
    root
}
fn expired_manual_backups(root: &Path) {
    let now = Utc::now();
    for index in 0..=MAX_PER_ACTION {
        manager::create(
            root,
            now - Days::days(40 + index as i64),
            BackupCleanupPolicy::Manual,
        )
        .unwrap();
    }
}
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn collect(root: &Path, path: &Path, output: &mut BTreeMap<PathBuf, Vec<u8>>) {
        if !path.exists() {
            return;
        }
        if path.is_dir() {
            for entry in fs::read_dir(path).unwrap() {
                collect(root, &entry.unwrap().path(), output);
            }
        } else {
            output.insert(
                path.strip_prefix(root).unwrap().into(),
                fs::read(path).unwrap(),
            );
        }
    }
    let mut output = BTreeMap::new();
    for name in ["manager-state", "backups"] {
        collect(root, &root.join(name), &mut output);
    }
    output
}

// Only the parent tests explicitly execute this helper. The extra ignored count
// is a subprocess entry point, not an unverified regression exemption.
#[test]
#[ignore = "subprocess helper; run by both parent tests on explicit temporary roots"]
fn w2_child() {
    let root = PathBuf::from(std::env::var_os("OCX_W2_TEST_ROOT").unwrap());
    let mode = std::env::var("OCX_W2_TEST_MODE").unwrap();
    let id = std::env::var("OCX_W2_TEST_ID").unwrap();
    fs::write(root.join("started"), std::process::id().to_string()).unwrap();
    match mode.as_str() {
        "create" => {
            manager::create_with_saved_policy(&root, Utc::now()).unwrap();
        }
        "pin" => manager::set_pinned(&root, &id, true).unwrap(),
        "restore" => {
            manager::restore(&root, &id, Utc::now()).unwrap();
        }
        "cleanup" => {
            let now = Utc::now() + Days::days(100);
            let preview = manager::preview(&root, now).unwrap();
            let removed = manager::execute(&root, &preview, now).unwrap();
            fs::write(root.join("removed"), removed.len().to_string()).unwrap();
        }
        "policy" => manager::save_policy(&root, &CleanupPolicy::default()).unwrap(),
        "list" => {
            manager::list(&root).unwrap();
        }
        "protection" => {
            let guard = manager::begin_preferences_protection(&root, true).unwrap();
            fs::write(
                root.join("protected"),
                guard.backup.as_ref().unwrap().backup_id.as_bytes(),
            )
            .unwrap();
            // Parent forcibly terminates us; no commit_verified or reconciliation.
            std::thread::sleep(Duration::from_secs(10));
            panic!("parent did not terminate protection holder");
        }
        _ => panic!("unknown subprocess mode"),
    }
    fs::write(root.join("done"), mode).unwrap();
}

#[test]
fn w2_transactions_wait_across_processes_without_changing_files() {
    for mode in ["create", "pin", "restore", "cleanup", "policy", "list"] {
        let root = fixture();
        let original = fs::read(root.path().join(PREFERENCES_RELATIVE_PATH)).unwrap();
        let backup = manager::create(root.path(), Utc::now(), BackupCleanupPolicy::Manual)
            .unwrap()
            .backup;
        if mode == "cleanup" {
            expired_manual_backups(root.path());
        }
        if mode == "restore" {
            PreferencesStore::new(root.path())
                .save(&Preferences {
                    interface_scale: 175,
                    ..Default::default()
                })
                .unwrap();
            assert_ne!(
                fs::read(root.path().join(PREFERENCES_RELATIVE_PATH)).unwrap(),
                original
            );
        }
        let guard = manager::acquire_preferences_transaction(root.path()).unwrap();
        let before = snapshot(root.path());
        let mut worker = Worker::spawn(root.path(), mode, &backup.backup_id);
        worker.marker(root.path(), "started");
        let child_pid = fs::read_to_string(root.path().join("started")).unwrap();
        assert_ne!(child_pid, std::process::id().to_string());
        std::thread::sleep(Duration::from_millis(150));
        assert!(
            worker.0.try_wait().unwrap().is_none(),
            "{mode} bypassed lock"
        );
        assert!(
            !root.path().join("done").exists(),
            "{mode} completed while held"
        );
        assert_eq!(snapshot(root.path()), before, "{mode} mutated while held");
        drop(guard);
        worker.success(root.path());
        assert_eq!(
            fs::read(root.path().join(PREFERENCES_RELATIVE_PATH)).unwrap(),
            original
        );
        let items = manager::list(root.path()).unwrap();
        if mode == "pin" {
            assert!(
                items
                    .iter()
                    .find(|b| b.id == backup.backup_id)
                    .unwrap()
                    .pinned
            );
        }
        if mode == "create" {
            assert_eq!(items.len(), 2);
        }
        if mode == "cleanup" {
            assert_eq!(items.len(), MAX_PER_ACTION);
            assert_eq!(
                fs::read_to_string(root.path().join("removed")).unwrap(),
                "2"
            );
        }
    }
}

#[test]
fn killed_process_releases_lock_but_keeps_unresolved_protection() {
    let root = fixture();
    let original = fs::read(root.path().join(PREFERENCES_RELATIVE_PATH)).unwrap();
    expired_manual_backups(root.path());
    let mut holder = Worker::spawn(root.path(), "protection", "");
    holder.marker(root.path(), "protected");
    let id = fs::read_to_string(root.path().join("protected")).unwrap();
    holder.0.kill().unwrap();
    assert!(!holder.0.wait().unwrap().success());
    fs::remove_file(root.path().join("started")).unwrap();
    // A fresh process must acquire the OS-released lock within a bounded wait.
    let mut cleanup = Worker::spawn(root.path(), "cleanup", "");
    cleanup.success(root.path());
    assert_eq!(
        fs::read_to_string(root.path().join("removed")).unwrap(),
        "1"
    );
    let items = manager::list(root.path()).unwrap();
    let protection = items.iter().find(|b| b.id == id).unwrap();
    assert!(protection.protected);
    assert!(!protection.pinned);
    assert!(manager::set_pinned(root.path(), &id, false).is_err());
    assert_eq!(
        fs::read(root.path().join(PREFERENCES_RELATIVE_PATH)).unwrap(),
        original
    );
}
