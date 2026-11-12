use super::*;
use std::fs;
use std::os::unix::fs::{symlink, DirBuilderExt, FileTypeExt, PermissionsExt};

struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    data: PathBuf,
    client: PathBuf,
    context: ProcessContext,
}
#[test]
#[ignore = "explicit retained owned fixture; read-only completion integrity observation, no service/new baseline"]
fn inspect_owned_closed_index_without_starting_or_repairing() {
    let root = PathBuf::from(std::env::var_os("OCXD024_CLOSED_INDEX_ROOT").expect("owned fixture"));
    assert!(
        root.is_absolute()
            && root.canonicalize().unwrap() == root
            && root.starts_with("/private/tmp")
            && root.ancestors().any(|p| p.file_name().is_some_and(|n| n
                .to_string_lossy()
                .starts_with("ocxd024-versioned-lifecycle-")))
    );
    let profile = root.join("profile");
    let data = profile.join(".codex-desktop/desktop");
    let context = ProcessContext {
        runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(),
        working_directory: profile.clone(),
        opencodex_home: profile.join(".codex-desktop/opencodex/home"),
    };
    let result = inspect(&context, &data);
    let expected = std::env::var("OCXD024_CLOSED_INDEX_EXPECT").expect("expected observation");
    match result {
        Ok(Some(head)) if expected == "restored" => {
            assert_eq!(head.state, State::Restored);
            println!(
                "{}",
                serde_json::json!({"observed":"restored","revision":head.revision,"receiptSha256":head.completion_sha256,"readOnly":true})
            );
        }
        Err(error) if expected == "refused" => println!(
            "{}",
            serde_json::json!({"observed":"refused","kind":format!("{:?}",error.kind()),"readOnly":true})
        ),
        other => panic!("unexpected completion integrity observation: {other:?}"),
    }
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::Builder::new()
            .prefix("ocxd024-index-fixture-")
            .tempdir()
            .unwrap();
        let root = temp.path().canonicalize().unwrap();
        let data = root.join("desktop");
        let profile = root.join("profile");
        let client = profile.join(".codex");
        let home = profile.join("home");
        for path in [&data, &profile, &client, &home] {
            fs::DirBuilder::new().mode(0o700).create(path).unwrap();
        }
        let context = ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(
            ),
            working_directory: profile,
            opencodex_home: home,
        };
        Self {
            _temp: temp,
            root,
            data,
            client,
            context,
        }
    }
    fn open(&self) -> io::Result<Index> {
        Index::open(&self.context, &self.data, &self.client)
    }
}
fn refs(n: usize) -> References {
    References {
        attempt: "e61c1ed4-d2b9-4c6e-8699-1b9c269a3f6f".into(),
        original_sha256: "a".repeat(64),
        scope_sha256: "b".repeat(64),
        phases: [Phase::Ready, Phase::PreStop, Phase::Stopped]
            .into_iter()
            .take(n)
            .map(|p| (p, "c".repeat(64)))
            .collect(),
        live_sha256: if n > 0 { Some("d".repeat(64)) } else { None },
    }
}

#[test]
fn observation_is_read_only_and_does_not_initialize_a_missing_session() {
    let f = Fixture::new();
    assert!(inspect(&f.context, &f.data).unwrap().is_none());
    assert!(!f.data.join("manager-state").exists());
    let index = f.open().unwrap();
    drop(index);
    let path = f.data.join("manager-state").join(FILE);
    let before = fs::read(&path).unwrap();
    let entries = fs::read_dir(path.parent().unwrap()).unwrap().count();
    assert!(inspect(&f.context, &f.data).unwrap().is_none());
    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(
        fs::read_dir(path.parent().unwrap()).unwrap().count(),
        entries
    );
}
#[test]
fn recovery_open_existing_never_initializes_missing_or_empty_storage() {
    let f = Fixture::new();
    assert!(Index::open_existing(&f.context, &f.data, &f.client).is_err());
    assert!(!f.data.join("manager-state").exists());
    let index = f.open().unwrap();
    drop(index);
    let manager = f.data.join("manager-state");
    fs::remove_file(manager.join("client-session-action.lock")).unwrap();
    assert!(Index::open_existing(&f.context, &f.data, &f.client).is_err());
    assert!(!manager.join("client-session-action.lock").exists());
    let file = crate::infrastructure::platform::create_new_private_file(
        &manager.join("client-session-action.lock"),
        0o600,
    )
    .unwrap();
    drop(file);
    fs::remove_file(manager.join(FILE)).unwrap();
    assert!(Index::open_existing(&f.context, &f.data, &f.client).is_err());
    assert!(!manager.join(FILE).exists());
    let file = crate::infrastructure::platform::create_new_private_file(&manager.join(FILE), 0o600)
        .unwrap();
    drop(file);
    assert!(Index::open_existing(&f.context, &f.data, &f.client).is_err());
    assert!(fs::read(manager.join(FILE)).unwrap().is_empty());
}
#[test]
fn evidence_projection_keeps_shared_action_lock_for_the_entire_read_callback() {
    let f = Fixture::new();
    let index = f.open().unwrap();
    let original = index.reserve_start().unwrap();
    drop(index);
    let action = f.data.join("manager-state/client-session-action.lock");
    let before = fs::read(f.data.join("manager-state").join(FILE)).unwrap();
    let revision = observation::inspect_map(&f.context, &f.data, |head| {
        assert_eq!(head, Some(&original));
        let competing = File::open(&action)?;
        assert!(
            competing.try_lock().is_err(),
            "evidence callback must still hold its shared action lease"
        );
        Ok(head.unwrap().revision)
    })
    .unwrap();
    assert_eq!(revision, 1);
    assert_eq!(
        fs::read(f.data.join("manager-state").join(FILE)).unwrap(),
        before
    );
    let released = File::open(action).unwrap();
    released.try_lock().unwrap();
    released.unlock().unwrap();
}

#[test]
fn retained_originals_without_current_references_are_unknown_and_cannot_initialize_or_reserve() {
    let f = Fixture::new();
    let index = f.open().unwrap();
    let manager = f.data.join("manager-state");
    fs::create_dir(manager.join("client-originals")).unwrap();
    fs::write(
        manager.join("client-originals/retained-proof"),
        b"owned original must remain",
    )
    .unwrap();
    assert!(index.reserve_start().is_err());
    drop(index);
    assert!(inspect(&f.context, &f.data).is_err());
    fs::remove_file(manager.join(FILE)).unwrap();
    fs::remove_file(manager.join("client-session-action.lock")).unwrap();
    assert!(inspect(&f.context, &f.data).is_err());
    assert!(f.open().is_err());
    assert!(!manager.join(FILE).exists() && !manager.join("client-session-action.lock").exists());
    assert_eq!(
        fs::read(manager.join("client-originals/retained-proof")).unwrap(),
        b"owned original must remain"
    );
}

#[test]
fn observation_uses_the_exact_unicode_path_instead_of_interpreting_uri_metacharacters() {
    let mut f = Fixture::new();
    let path = f.root.join("desktop 中文 ?#%");
    fs::rename(&f.data, &path).unwrap();
    f.data = path;
    let index = f.open().unwrap();
    let head = index.reserve_start().unwrap();
    drop(index);
    assert_eq!(inspect(&f.context, &f.data).unwrap(), Some(head));
}

#[test]
fn stopped_plan_is_observed_after_reopen_without_rebasing_original_references() {
    let f = Fixture::new();
    let index = f.open().unwrap();
    let mut head = index.reserve_start().unwrap();
    for (state, n) in [
        (State::Captured, 0),
        (State::Ready, 1),
        (State::StopPrepared, 2),
        (State::Stopped, 3),
    ] {
        head = index.advance(&head, state, Some(refs(n))).unwrap();
    }
    head = index
        .record_restore_plan(
            &head,
            "77f2af67-cdcc-42da-b5e4-e218416063a9",
            &"e".repeat(64),
        )
        .unwrap();
    drop(index);
    let path = f.data.join("manager-state").join(FILE);
    let before = fs::read(&path).unwrap();
    assert_eq!(inspect(&f.context, &f.data).unwrap(), Some(head));
    assert_eq!(fs::read(&path).unwrap(), before);
}

#[test]
fn observation_does_not_close_a_sqlite_inode_while_a_same_process_writer_holds_it() {
    const CHILD: &str = "OCXD024_INDEX_READONLY_SQLITE_PROBE";
    const TEST:&str="infrastructure::macos_client_originals::session_index::tests::observation_does_not_close_a_sqlite_inode_while_a_same_process_writer_holds_it";
    if let Some(path) = std::env::var_os(CHILD) {
        let path = PathBuf::from(path);
        assert!(path.starts_with("/private/tmp"));
        let connection = Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .unwrap();
        connection.busy_timeout(std::time::Duration::ZERO).unwrap();
        assert!(
            matches!(connection.execute_batch("BEGIN IMMEDIATE"),Err(rusqlite::Error::SqliteFailure(error,_)) if error.code == rusqlite::ErrorCode::DatabaseBusy),
            "observer must not release the parent's POSIX writer lock"
        );
        println!("SQLITE_PARENT_LOCK_RETAINED");
        return;
    }
    let f = Fixture::new();
    let index = f.open().unwrap();
    index
        .connection()
        .unwrap()
        .execute_batch("BEGIN IMMEDIATE")
        .unwrap();
    assert!(inspect(&f.context, &f.data).is_err());
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--nocapture"])
        .env(CHILD, f.data.join("manager-state").join(FILE))
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{} {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).contains("SQLITE_PARENT_LOCK_RETAINED"));
    println!("{}", String::from_utf8_lossy(&result.stdout));
    index
        .connection()
        .unwrap()
        .execute_batch("ROLLBACK")
        .unwrap();
}

#[test]
fn observation_refuses_unsafe_permissions_redirected_context_and_incomplete_initializers() {
    let f = Fixture::new();
    drop(f.open().unwrap());
    let path = f.data.join("manager-state").join(FILE);
    let original = fs::read(&path).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(inspect(&f.context, &f.data).is_err());
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o644
    );
    assert_eq!(fs::read(&path).unwrap(), original);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let mut changed = f.context.clone();
    changed.opencodex_home = f.root.join("another-home");
    fs::create_dir(&changed.opencodex_home).unwrap();
    assert!(inspect(&changed, &f.data).is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
    fs::remove_file(&path).unwrap();
    assert!(inspect(&f.context, &f.data).is_err());
    assert!(
        !path.exists(),
        "read-only observation must not recreate an abandoned initializer"
    );
}

#[test]
fn observation_refuses_sidecars_old_schema_and_orphaned_or_newer_events_without_repair() {
    for mutation in ["sidecar", "schema", "view", "orphan", "ahead", "wal"] {
        let f = Fixture::new();
        let index = f.open().unwrap();
        index.reserve_start().unwrap();
        drop(index);
        let path = f.data.join("manager-state").join(FILE);
        if mutation == "sidecar" {
            fs::write(
                path.with_file_name(format!("{FILE}-journal")),
                b"unconfirmed original journal",
            )
            .unwrap();
        } else {
            let connection = Connection::open(&path).unwrap();
            connection.execute_batch(match mutation {
                "schema"=>"PRAGMA user_version=1",
                "view"=>"CREATE VIEW unfamiliar AS SELECT * FROM events",
                "orphan"=>"DELETE FROM current_session",
                "ahead"=>"INSERT INTO events(revision,document) SELECT 2,document FROM events WHERE revision=1",
                "wal"=>"PRAGMA journal_mode=WAL",_=>unreachable!(),
            }).unwrap();
            connection.close().unwrap();
        }
        let before = fs::read(&path).unwrap();
        assert!(inspect(&f.context, &f.data).is_err(), "{mutation}");
        assert_eq!(fs::read(&path).unwrap(), before, "{mutation}");
        if mutation == "sidecar" {
            assert_eq!(
                fs::read(path.with_file_name(format!("{FILE}-journal"))).unwrap(),
                b"unconfirmed original journal"
            );
        }
        if mutation == "wal" {
            assert!(!path.with_file_name(format!("{FILE}-shm")).exists());
        }
    }
}

#[test]
fn actual_index_recovery_fact_survives_a_failed_service_observation_and_blocks_restart_on_reopen() {
    use crate::modules::status::{ClientRecoveryState as Recovery, StatusCollector, StatusSource};
    #[derive(Debug)]
    struct Managed(PathBuf);
    impl crate::infrastructure::runtime_executable::RuntimeExecutableProvider for Managed {
        fn executable(&self) -> Option<PathBuf> {
            Some(self.0.join("not-executed"))
        }
        fn managed_installation_root(&self) -> Option<PathBuf> {
            Some(self.0.clone())
        }
    }
    let f = Fixture::new();
    let index = f.open().unwrap();
    let head = index.reserve_start().unwrap();
    drop(index);
    let build = || {
        crate::infrastructure::status_source::OfficialStatusSource::new(
            std::sync::Arc::new(Managed(f.root.clone())),
            f.context.working_directory.clone(),
            crate::modules::process::EnvironmentPolicy {
                opencodex_home: f.context.opencodex_home.clone(),
                ..Default::default()
            },
        )
    };
    assert_eq!(
        build().client_recovery(),
        Recovery::Unknown,
        "missing explicit D must not guess a recovery root"
    );
    let source = build().with_client_recovery_root(f.data.clone());
    assert_eq!(source.client_recovery(), Recovery::Required);
    let mut collector = StatusCollector::new(source);
    assert!(collector.refresh().is_err()); // no runtime/CLI is executed here
    assert_eq!(collector.facts().client_recovery, Recovery::Required);
    let snapshot = crate::types::runtime_status::snapshot_from_matrix(
        collector.matrix(),
        collector.facts(),
        None,
        None,
        crate::types::runtime_status::StatusSource::Fixture,
    );
    assert!(!snapshot.can_start && !snapshot.can_restart);
    let tray = crate::commands::tray::tray_state_from_parts(&collector, &f.data).unwrap();
    assert!(!tray.can_start && !tray.can_stop && !tray.can_restart);
    assert_eq!(inspect(&f.context, &f.data).unwrap(), Some(head));
}

#[test]
fn references_are_durable_sequential_and_an_unfinished_session_cannot_be_overwritten() {
    let f = Fixture::new();
    let store = f.open().unwrap();
    assert!(store.head().unwrap().is_none());
    let first = store.reserve_start().unwrap();
    assert!(store.reserve_start().is_err());
    let captured = store
        .advance(&first, State::Captured, Some(refs(0)))
        .unwrap();
    let ready = store
        .advance(&captured, State::Ready, Some(refs(1)))
        .unwrap();
    drop(store);
    let reopened = f.open().unwrap();
    assert_eq!(reopened.head().unwrap(), Some(ready.clone()));
    let prep = reopened
        .advance(&ready, State::StopPrepared, Some(refs(2)))
        .unwrap();
    let stopped = reopened
        .advance(&prep, State::Stopped, Some(refs(3)))
        .unwrap();
    assert_eq!(reopened.head().unwrap(), Some(stopped));
    assert!(reopened.reserve_start().is_err());
    assert!(reopened
        .advance(&ready, State::Failed, Some(refs(1)))
        .is_err());
}

#[test]
fn stale_expected_revision_rebase_and_missing_phase_are_rejected_without_changes() {
    let f = Fixture::new();
    let index = f.open().unwrap();
    let first = index.reserve_start().unwrap();
    let captured = index
        .advance(&first, State::Captured, Some(refs(0)))
        .unwrap();
    assert!(index
        .advance(&first, State::Captured, Some(refs(0)))
        .is_err());
    assert!(index
        .advance(&captured, State::Ready, Some(refs(0)))
        .is_err());
    let mut other = refs(1);
    other.original_sha256 = "e".repeat(64);
    assert!(index.advance(&captured, State::Ready, Some(other)).is_err());
    assert!(index
        .advance(&captured, State::Stopped, Some(refs(3)))
        .is_err());
    assert_eq!(index.head().unwrap(), Some(captured));
}

#[test]
fn failed_capture_is_not_an_empty_or_safe_new_baseline() {
    let f = Fixture::new();
    let index = f.open().unwrap();
    let first = index.reserve_start().unwrap();
    let failed = index.advance(&first, State::Failed, None).unwrap();
    drop(index);
    let reopened = f.open().unwrap();
    assert_eq!(reopened.head().unwrap(), Some(failed));
    assert!(reopened.reserve_start().is_err());
}

#[test]
fn a_real_second_process_is_excluded_for_entire_action_and_can_reopen_after_owner_exits() {
    const CHILD: &str = "OCXD024_SESSION_INDEX_PROBE";
    if let Some(root) = std::env::var_os(CHILD) {
        let root = PathBuf::from(root);
        assert!(
            root.is_absolute()
                && root.parent() == Some(std::env::temp_dir().canonicalize().unwrap().as_path())
                && root
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("ocxd024-index-fixture-")
        );
        let context = ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(
            ),
            working_directory: root.join("profile"),
            opencodex_home: root.join("profile/home"),
        };
        let blocked = std::env::var("OCXD024_SESSION_INDEX_BLOCKED").unwrap() == "1";
        assert_eq!(
            Index::open(
                &context,
                &root.join("desktop"),
                &root.join("profile/.codex")
            )
            .is_err(),
            blocked
        );
        return;
    }
    let f = Fixture::new();
    let index = f.open().unwrap();
    let first = index.reserve_start().unwrap();
    let probe = |blocked: bool| {
        let output=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::session_index::tests::a_real_second_process_is_excluded_for_entire_action_and_can_reopen_after_owner_exits","--nocapture"]).env(CHILD,&f.root).env("OCXD024_SESSION_INDEX_BLOCKED",if blocked{"1"}else{"0"}).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    probe(true);
    drop(index);
    probe(false);
    assert_eq!(f.open().unwrap().head().unwrap(), Some(first));
}

#[test]
fn abrupt_process_exit_keeps_captured_references_and_does_not_scan_for_new_ones() {
    const CHILD: &str = "OCXD024_SESSION_INDEX_CRASH";
    if let Some(root) = std::env::var_os(CHILD) {
        let root = PathBuf::from(root);
        let context = ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(
            ),
            working_directory: root.join("profile"),
            opencodex_home: root.join("profile/home"),
        };
        let store = Index::open(
            &context,
            &root.join("desktop"),
            &root.join("profile/.codex"),
        )
        .unwrap();
        let first = store.reserve_start().unwrap();
        store
            .advance(&first, State::Captured, Some(refs(0)))
            .unwrap();
        std::process::exit(91);
    }
    let f = Fixture::new();
    let output=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::session_index::tests::abrupt_process_exit_keeps_captured_references_and_does_not_scan_for_new_ones","--nocapture"]).env(CHILD,&f.root).output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(91),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let reopened = f.open().unwrap();
    let head = reopened.head().unwrap().unwrap();
    assert_eq!(head.state, State::Captured);
    assert!(head.references == Some(refs(0)));
    assert!(reopened.reserve_start().is_err());
}

#[test]
fn directory_generation_or_profile_home_change_refuses_prior_index() {
    let f = Fixture::new();
    let store = f.open().unwrap();
    store.reserve_start().unwrap();
    drop(store);
    let mut context = f.context.clone();
    let other = f.root.join("other-home");
    fs::DirBuilder::new().mode(0o700).create(&other).unwrap();
    context.opencodex_home = other;
    assert!(Index::open(&context, &f.data, &f.client).is_err());
    fs::rename(&f.client, f.client.with_extension("saved")).unwrap();
    fs::DirBuilder::new().mode(0o700).create(&f.client).unwrap();
    assert!(f.open().is_err());
}

#[test]
fn existing_action_symlink_and_unknown_index_are_not_adopted_or_repaired() {
    let f = Fixture::new();
    let manager = f.data.join("manager-state");
    fs::DirBuilder::new().mode(0o700).create(&manager).unwrap();
    let outside = f.root.join("outside");
    fs::write(&outside, b"sentinel").unwrap();
    symlink(&outside, manager.join("client-session-action.lock")).unwrap();
    assert!(f.open().is_err());
    assert_eq!(fs::read(outside).unwrap(), b"sentinel");
    fs::remove_file(manager.join("client-session-action.lock")).unwrap();
    fs::write(manager.join(FILE), b"unknown db bytes").unwrap();
    fs::set_permissions(manager.join(FILE), fs::Permissions::from_mode(0o600)).unwrap();
    assert!(f.open().is_err());
    assert_eq!(fs::read(manager.join(FILE)).unwrap(), b"unknown db bytes");
}

#[test]
fn index_and_event_document_tampering_cannot_supply_new_current_references() {
    let f = Fixture::new();
    let store = f.open().unwrap();
    let first = store.reserve_start().unwrap();
    let mut changed = first.clone();
    changed.ticket = uuid::Uuid::new_v4().to_string();
    store
        .connection()
        .unwrap()
        .execute(
            "UPDATE current_session SET document=?1 WHERE singleton=1",
            [encode(&changed).unwrap()],
        )
        .unwrap();
    assert!(store.head().is_err());
}

fn stopped(store: &Index) -> Head {
    let mut head = store.reserve_start().unwrap();
    for (state, n) in [
        (State::Captured, 0),
        (State::Ready, 1),
        (State::StopPrepared, 2),
        (State::Stopped, 3),
    ] {
        head = store.advance(&head, state, Some(refs(n))).unwrap();
    }
    head
}
#[test]
fn restore_plan_references_commit_once_with_same_originals_and_survive_reopen() {
    let f = Fixture::new();
    let store = f.open().unwrap();
    let stopped = stopped(&store);
    let id = "88fe6d0a-56fd-475f-bf92-c5f0f813779a";
    let sha = "e".repeat(64);
    let planned = store.record_restore_plan(&stopped, id, &sha).unwrap();
    assert_eq!(planned.state, State::RestorePrepared);
    assert_eq!(planned.revision, 6);
    assert!(planned.references == stopped.references);
    assert!(store
        .record_restore_plan(&stopped, id, &"f".repeat(64))
        .is_err());
    assert!(store.record_restore_plan(&planned, id, &sha).is_err());
    assert!(store.reserve_start().is_err());
    assert!(store.advance(&planned, State::Failed, None).is_err());
    drop(store);
    let reopened = f.open().unwrap();
    assert_eq!(reopened.head().unwrap(), Some(planned));
    assert!(reopened.reserve_start().is_err());
}

#[test]
fn restored_label_and_digest_cannot_forge_a_completed_session_or_new_start() {
    let f = Fixture::new();
    let store = f.open().unwrap();
    let current = stopped(&store);
    let planned = store
        .record_restore_plan(
            &current,
            "88fe6d0a-56fd-475f-bf92-c5f0f813779a",
            &"e".repeat(64),
        )
        .unwrap();
    assert!(store
        .advance(&planned, State::Restored, planned.references.clone())
        .is_err());
    let mut forged = planned.clone();
    forged.state = State::Restored;
    assert!(validate(&forged).is_err());
    forged.completion_sha256 = Some("f".repeat(64));
    forged.revision += 1;
    validate(&forged).unwrap();
    // An independently serialized digest/label still lacks original frames and
    // real bound receipt. Structural JSON validity cannot grant restart.
    assert!(completion::verify_registered(
        &f.data,
        &f.client,
        &f.context.working_directory,
        &forged
    )
    .is_err());
    store.write(Some(&planned), &forged).unwrap();
    assert!(store.reserve_start().is_err());
    drop(store);
    assert!(inspect(&f.context, &f.data).is_err());
    let reopened = f.open().unwrap();
    assert!(reopened.reserve_start().is_err());
}
#[test]
fn malformed_or_early_restore_plan_never_publishes_new_references() {
    let f = Fixture::new();
    let store = f.open().unwrap();
    let preparing = store.reserve_start().unwrap();
    assert!(store
        .record_restore_plan(
            &preparing,
            &uuid::Uuid::new_v4().to_string(),
            &"e".repeat(64)
        )
        .is_err());
    let mut captured = store
        .advance(&preparing, State::Captured, Some(refs(0)))
        .unwrap();
    for (state, n) in [
        (State::Ready, 1),
        (State::StopPrepared, 2),
        (State::Stopped, 3),
    ] {
        captured = store.advance(&captured, state, Some(refs(n))).unwrap();
    }
    for (id, sha) in [
        ("../other".to_string(), "a".repeat(64)),
        (uuid::Uuid::new_v4().to_string(), "invalid".into()),
    ] {
        assert!(store.record_restore_plan(&captured, &id, &sha).is_err());
    }
    assert_eq!(store.head().unwrap(), Some(captured));
}
#[test]
fn previous_index_schema_is_refused_without_compatibility_or_automatic_migration() {
    let f = Fixture::new();
    let index = f.open().unwrap();
    index.reserve_start().unwrap();
    index
        .connection()
        .unwrap()
        .execute_batch("PRAGMA user_version=1")
        .unwrap();
    drop(index);
    let path = f.data.join("manager-state").join(FILE);
    let before = fs::read(&path).unwrap();
    assert!(f.open().is_err());
    assert_eq!(fs::read(path).unwrap(), before);
}

#[test]
fn action_lock_special_file_or_changed_permissions_are_refused_without_repair() {
    let f = Fixture::new();
    let manager = f.data.join("manager-state");
    fs::DirBuilder::new().mode(0o700).create(&manager).unwrap();
    let path = manager.join("client-session-action.lock");
    let c = CString::new(path.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(c.as_ptr(), 0o600) }, 0);
    assert!(f.open().is_err());
    assert!(fs::symlink_metadata(&path).unwrap().file_type().is_fifo());
    fs::remove_file(&path).unwrap();
    let index = f.open().unwrap();
    index.reserve_start().unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(index.head().is_err());
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o644
    );
}

#[test]
#[ignore = "explicit retained own product fixture; reads existing Stopped index only, no service or new baseline"]
fn retained_product_index_reopens_with_final_lock_validation_and_preserves_original_references() {
    let root = PathBuf::from(
        std::env::var_os("OCXD_EXISTING_PRODUCT_SESSION").expect("explicit own product fixture"),
    );
    assert!(
        root.is_absolute()
            && root.canonicalize().unwrap() == root
            && root.starts_with("/private/tmp")
    );
    assert!(root
        .parent()
        .unwrap()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .starts_with("ocxd024-versioned-lifecycle-"));
    let profile = root.join("profile");
    let data = profile.join(".codex-desktop/desktop");
    let client = profile.join(".codex");
    assert!(data
        .join("manager-state/client-session-index.sqlite")
        .is_file());
    let context = ProcessContext {
        runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(),
        working_directory: profile.clone(),
        opencodex_home: profile.join(".codex-desktop/opencodex/home"),
    };
    let index = Index::open(&context, &data, &client).unwrap();
    let head = index.head().unwrap().unwrap();
    assert_eq!(head.state, State::Stopped);
    assert_eq!(head.revision, 5);
    let references = head.references.as_ref().unwrap();
    assert_eq!(references.phases.len(), 3);
    assert!(references.live_sha256.is_some());
    assert!(index.reserve_start().is_err());
    assert_eq!(index.head().unwrap(), Some(head.clone()));
    println!("RETAINED_INDEX_STOPPED revision={} phases={} original_sha256={} baseline_overwrite_refused=true",head.revision,references.phases.len(),references.original_sha256);
}
