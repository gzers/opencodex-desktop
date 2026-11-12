use super::*;
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    time::{Duration, Instant},
};
fn setup(root: &Path) -> (PathBuf, PathBuf, ProcessContext) {
    let data = root.join("desktop");
    let profile = root.join("profile");
    let home = profile.join("home");
    for p in [&data, &profile, &home] {
        fs::DirBuilder::new().mode(0o700).create(p).unwrap();
    }
    let context = ProcessContext {
        runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(),
        working_directory: profile.clone(),
        opencodex_home: home,
    };
    (data, profile, context)
}
fn publish(data: &Path, profile: &Path) -> super::super::root_creation::PublishedRoot {
    super::super::root_creation::reserve(data, profile, Instant::now() + Duration::from_secs(10))
        .unwrap()
        .publish()
        .unwrap()
}
fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(root: &Path, current: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(current).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                walk(root, &p, out);
            } else {
                out.insert(p.strip_prefix(root).unwrap().into(), fs::read(&p).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}
#[test]
fn first_root_registration_atomically_binds_original_absence_and_preparing_without_enabling_ordinary_capture(
) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let (data, profile, context) = setup(&root);
    let published = publish(&data, &profile);
    let refs = published.references().unwrap();
    let expected_created = published.path().to_path_buf();
    let (index, head) = Index::register_first_root(&context, &data, published).unwrap();
    assert!(index.root_created_fd.is_some());
    assert_eq!(head.state, State::Preparing);
    assert_eq!(head.revision, 1);
    assert_eq!(head.ticket, refs.id);
    assert!(head.references.is_none());
    assert!(index.root_registration.as_ref() == Some(&refs));
    let stored: Vec<u8> = index
        .connection()
        .unwrap()
        .query_row("SELECT document FROM binding", [], |r| r.get(0))
        .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&stored).unwrap();
    assert_eq!(value["originalRoot"]["intentSha256"], refs.intent_sha256);
    assert_eq!(
        index
            .connection()
            .unwrap()
            .query_row("SELECT count(*) FROM events", [], |r| r.get::<_, u64>(0))
            .unwrap(),
        1
    );
    assert!(index.reserve_capture(&head).is_err());
    assert!(index.reserve_start().is_err());
    assert!(index.advance(&head, State::Captured, None).is_err());
    assert!(!data.join("manager-state/client-originals").exists());
    drop(index);
    let before = files(&root);
    let observed = inspect(&context, &data).unwrap().unwrap();
    assert_eq!(observed, head);
    assert!(files(&root) == before);
    let reopened = Index::open_existing(&context, &data, &expected_created).unwrap();
    assert!(reopened.root_created_fd.is_none());
    assert!(reopened.root_registration.as_ref() == Some(&refs));
    assert_eq!(reopened.head().unwrap(), Some(head.clone()));
    assert!(reopened.reserve_capture(&head).is_err());
    drop(reopened);
    let preview = preview::inspect_preview(&context, &data);
    assert_eq!(preview.status, preview::Status::Pending);
    assert!(
        !preview.can_cancel_preparation
            && !preview.can_finish_unchanged
            && !preview.can_prepare_plan
            && !preview.can_continue
    );
    assert_eq!(preview.phase, Some(State::Preparing));
}
#[test]
fn first_root_registration_never_releases_the_original_action_flock_at_transfer_or_commit() {
    const CHILD: &str = "OCXD024_ROOT_INDEX_LOCK";
    if let Some(path) = std::env::var_os(CHILD) {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(PathBuf::from(path))
            .unwrap();
        assert!(
            file.try_lock().is_err(),
            "creation-to-index lock was released"
        );
        return;
    }
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let (data, profile, context) = setup(&root);
    let published = publish(&data, &profile);
    let checks = std::cell::Cell::new(0);
    let checkpoint = |point: &str| -> io::Result<()> {
        checks.set(checks.get() + 1);
        assert!(matches!(
            point,
            "root_lock_transferred" | "before_root_index_commit" | "root_index_committed"
        ));
        let output=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::session_index::root_registration_tests::first_root_registration_never_releases_the_original_action_flock_at_transfer_or_commit","--nocapture"]).env(CHILD,data.join("manager-state/client-session-action.lock")).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    };
    let (index, _) = Index::register_first_root_checked(&context, &data, published, &|p| {
        checkpoint(p)?;
        Ok(())
    })
    .unwrap();
    assert_eq!(checks.get(), 3);
    drop(index);
}
#[test]
fn first_root_registration_rejects_changed_context_or_created_object_before_creating_index() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let (data, profile, mut context) = setup(&root);
    let published = publish(&data, &profile);
    let other = root.join("other-profile");
    fs::DirBuilder::new().mode(0o700).create(&other).unwrap();
    context.working_directory = other;
    assert!(Index::register_first_root(&context, &data, published).is_err());
    assert!(!data
        .join("manager-state/client-session-index.sqlite")
        .exists());
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let (data, profile, context) = setup(&root);
    let published = publish(&data, &profile);
    fs::rename(published.path(), profile.join("displaced-codex")).unwrap();
    fs::DirBuilder::new()
        .mode(0o700)
        .create(profile.join(".codex"))
        .unwrap();
    assert!(Index::register_first_root(&context, &data, published).is_err());
    assert!(!data
        .join("manager-state/client-session-index.sqlite")
        .exists());
}
#[test]
fn first_root_registration_cannot_be_downgraded_to_an_ordinary_existing_root_binding() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let (data, profile, context) = setup(&root);
    let (index, _) = Index::register_first_root(&context, &data, publish(&data, &profile)).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&index.binding).unwrap();
    let ordinary: Vec<u8> = serde_json::from_value(value["scope"].clone()).unwrap();
    index
        .connection()
        .unwrap()
        .execute(
            "UPDATE binding SET document=?1 WHERE singleton=1",
            [ordinary],
        )
        .unwrap();
    assert!(index.head().is_err());
    drop(index);
    assert!(inspect(&context, &data).is_err());
    assert!(Index::open_existing(&context, &data, &profile.join(".codex")).is_err());
}
#[test]
fn first_root_registration_requires_original_creation_bytes_and_the_first_event_ticket() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let (data, profile, context) = setup(&root);
    let (index, _) = Index::register_first_root(&context, &data, publish(&data, &profile)).unwrap();
    drop(index);
    let path = data.join("manager-state/client-root-reservation/created.json");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&path, b"{}").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(inspect(&context, &data).is_err());
    assert_eq!(fs::read(path).unwrap(), b"{}");
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let (data, profile, context) = setup(&root);
    let (index, mut head) =
        Index::register_first_root(&context, &data, publish(&data, &profile)).unwrap();
    head.ticket = uuid::Uuid::new_v4().to_string();
    let raw = encode(&head).unwrap();
    index
        .connection()
        .unwrap()
        .execute("UPDATE events SET document=?1 WHERE revision=1", [&raw])
        .unwrap();
    index
        .connection()
        .unwrap()
        .execute(
            "UPDATE current_session SET document=?1 WHERE revision=1",
            [&raw],
        )
        .unwrap();
    assert!(index.head().is_err());
    drop(index);
    assert!(inspect(&context, &data).is_err());
}
#[test]
fn first_root_registration_real_crash_never_commits_absence_without_preparing_or_adopts_uncommitted_origin(
) {
    const CHILD: &str = "OCXD024_ROOT_INDEX_CRASH";
    if let Some(root) = std::env::var_os(CHILD) {
        let root = PathBuf::from(root);
        let (data, profile, context) = setup(&root);
        let published = publish(&data, &profile);
        let wanted = std::env::var("OCXD024_ROOT_INDEX_BOUNDARY").unwrap();
        let _ = Index::register_first_root_checked(&context, &data, published, &|point| {
            if point == wanted {
                std::process::exit(91);
            }
            Ok(())
        });
        panic!("crash boundary not reached");
    }
    for boundary in [
        "root_lock_transferred",
        "before_root_index_commit",
        "root_index_committed",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let output=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::session_index::root_registration_tests::first_root_registration_real_crash_never_commits_absence_without_preparing_or_adopts_uncommitted_origin","--nocapture"]).env(CHILD,&root).env("OCXD024_ROOT_INDEX_BOUNDARY",boundary).output().unwrap();
        assert_eq!(
            output.status.code(),
            Some(91),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let context = ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(
            ),
            working_directory: root.join("profile"),
            opencodex_home: root.join("profile/home"),
        };
        let before = files(&root);
        let observed = inspect(&context, &root.join("desktop"));
        if boundary == "root_index_committed" {
            let head = observed.unwrap().unwrap();
            assert_eq!(head.state, State::Preparing);
            assert_eq!(head.revision, 1);
            assert!(head.references.is_none());
            let index = Index::open_existing(
                &context,
                &root.join("desktop"),
                &root.join("profile/.codex"),
            )
            .unwrap();
            assert!(index.reserve_capture(&head).is_err());
            drop(index);
        } else {
            assert!(observed.is_err());
            assert!(Index::open(
                &context,
                &root.join("desktop"),
                &root.join("profile/.codex")
            )
            .is_err());
        }
        assert!(files(&root) == before);
        assert!(!root.join("desktop/manager-state/client-originals").exists());
        println!(
            "ROOT_INDEX_CRASH_EVIDENCE {}",
            serde_json::json!({"accepted":false,"boundary":boundary,"childExitCode":output.status.code(),"registeredPreparing":boundary=="root_index_committed","noOriginalFrame":true,"files":before.into_iter().map(|(p,b)|serde_json::json!({"path":p,"bytes":b.len(),"sha256":digest(&b),"rawBase64":base64::Engine::encode(&base64::engine::general_purpose::STANDARD,b)})).collect::<Vec<_>>()})
        );
    }
}
