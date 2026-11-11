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
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
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
        assert!(root.starts_with("/private/tmp"));
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
