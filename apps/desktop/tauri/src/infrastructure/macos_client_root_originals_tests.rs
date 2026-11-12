use super::*;
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    time::{Duration, Instant},
};
struct Fixture {
    _temp: tempfile::TempDir,
    data: PathBuf,
    profile: PathBuf,
    context: crate::state::ProcessContext,
    index: session_index::Index,
    head: session_index::Head,
    origin: root_creation::References,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let data = root.join("desktop");
        let profile = root.join("profile");
        let home = profile.join("home");
        for p in [&data, &profile, &home] {
            fs::DirBuilder::new().mode(0o700).create(p).unwrap();
        }
        let context = crate::state::ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(
            ),
            working_directory: profile.clone(),
            opencodex_home: home,
        };
        let published =
            root_creation::reserve(&data, &profile, Instant::now() + Duration::from_secs(10))
                .unwrap()
                .publish()
                .unwrap();
        let origin = published.references().unwrap();
        let (index, head) =
            session_index::Index::register_first_root(&context, &data, published).unwrap();
        Self {
            _temp: temp,
            data,
            profile,
            context,
            index,
            head,
            origin,
        }
    }
    fn client(&self) -> PathBuf {
        self.profile.join(".codex")
    }
}
#[test]
fn root_absence_original_chain_keeps_created_execution_tree_distinct_from_original_user_state() {
    let f = Fixture::new();
    let reserved = f.index.reserve_first_root_capture(&f.head).unwrap();
    assert_eq!(reserved.state, session_index::State::CaptureReserved);
    assert_eq!(reserved.revision, 2);
    assert!(!f.data.join("manager-state/client-originals").exists());
    let permit = f.index.capture_reservation(&reserved).unwrap();
    let originals =
        Originals::begin_reserved_profile(&f.data, &f.client(), &f.profile, &permit).unwrap();
    let original = originals.frame(Phase::Original).unwrap();
    assert_eq!(original.0.schema_version, 5);
    assert!(original.0.original_root_absence.as_ref() == Some(&f.origin));
    assert!(original.0.tree.first_root_execution_baseline());
    assert!(original
        .0
        .capture
        .images
        .iter()
        .all(|i| *i == Image::Absent));
    let path = f.client().join("config.toml");
    fs::write(&path, b"model_provider='opencodex'\n").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let ready = originals.seal(Phase::Ready).unwrap();
    let pre = originals.seal(Phase::PreStop).unwrap();
    fs::write(&path, b"\n").unwrap();
    let stopped = originals.seal(Phase::Stopped).unwrap();
    let phases = vec![
        (Phase::Ready, ready),
        (Phase::PreStop, pre),
        (Phase::Stopped, stopped),
    ];
    let reopened = permit
        .reopen_root_originals(originals.original_sha256(), &phases)
        .unwrap();
    let frames = reopened.frames_through(Phase::Stopped).unwrap();
    assert_eq!(frames.len(), 4);
    assert!(frames.iter().all(|(frame, _)| frame.schema_version == 5
        && frame.original_root_absence.as_ref() == Some(&f.origin)));
    assert_eq!(frames[0].1, original.1);
    assert!(Originals::open_for_profile(
        &f.data,
        &f.client(),
        &f.profile,
        originals.attempt(),
        originals.original_sha256(),
        &phases
    )
    .is_err());
    assert_eq!(f.index.head().unwrap(), Some(reserved));
}
#[test]
fn root_absence_ordinary_constructor_and_nonempty_first_root_cannot_create_an_existing_root_baseline(
) {
    let f = Fixture::new();
    assert!(Originals::begin_for_profile(&f.data, &f.client(), &f.profile).is_err());
    assert!(!f.data.join("manager-state/client-originals").exists());
    fs::write(f.client().join("auth.json"), b"synthetic outside data").unwrap();
    assert!(f.index.reserve_first_root_capture(&f.head).is_err());
    assert_eq!(f.index.head().unwrap(), Some(f.head.clone()));
    let f = Fixture::new();
    let reserved = f.index.reserve_first_root_capture(&f.head).unwrap();
    let permit = f.index.capture_reservation(&reserved).unwrap();
    fs::write(f.client().join("auth.json"), b"do not adopt").unwrap();
    assert!(Originals::begin_reserved_profile(&f.data, &f.client(), &f.profile, &permit).is_err());
    assert_eq!(
        fs::read(f.client().join("auth.json")).unwrap(),
        b"do not adopt"
    );
    assert!(!f.data.join("manager-state/client-originals").exists());
}
#[test]
fn root_absence_execution_original_rejects_a_preexisting_service_owner_object() {
    let f = Fixture::new();
    let reserved = f.index.reserve_first_root_capture(&f.head).unwrap();
    let permit = f.index.capture_reservation(&reserved).unwrap();
    let path = f.client().join(".opencodex-native-main.owner.sqlite");
    fs::write(&path, b"unadmitted owner data").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(Originals::begin_reserved_profile(&f.data, &f.client(), &f.profile, &permit).is_err());
    assert!(path.exists());
    assert!(!f.data.join("manager-state/client-originals").exists());
}
#[test]
fn root_absence_ordinary_open_cannot_accept_a_downgraded_or_rebound_frame_even_with_updated_digest()
{
    for drop_absence in [true, false] {
        let f = Fixture::new();
        let reserved = f.index.reserve_first_root_capture(&f.head).unwrap();
        let permit = f.index.capture_reservation(&reserved).unwrap();
        let originals =
            Originals::begin_reserved_profile(&f.data, &f.client(), &f.profile, &permit).unwrap();
        let ready = originals.seal(Phase::Ready).unwrap();
        let path = originals.directory.path().join("ready.json");
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        if drop_absence {
            value["schemaVersion"] = serde_json::json!(4);
            value.as_object_mut().unwrap().remove("originalRootAbsence");
        } else {
            value["originalRootAbsence"]["intentSha256"] = serde_json::json!("a".repeat(64));
        }
        let bytes = serde_json::to_vec(&value).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, &bytes).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        originals.known.lock().unwrap()[Phase::Ready.index()] = Some(digest(&bytes));
        assert!(originals.frame(Phase::Ready).is_err());
        assert!(permit
            .reopen_root_originals(
                originals.original_sha256(),
                &[(Phase::Ready, digest(&bytes))]
            )
            .is_err());
        assert_ne!(ready, digest(&bytes));
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
}
#[test]
fn root_absence_cannot_mint_whole_root_completion_from_an_execution_directory_readback() {
    let f = Fixture::new();
    let reserved = f.index.reserve_first_root_capture(&f.head).unwrap();
    let permit = f.index.capture_reservation(&reserved).unwrap();
    let originals =
        Originals::begin_reserved_profile(&f.data, &f.client(), &f.profile, &permit).unwrap();
    let result = originals.complete_readback(
        session::References {
            attempt: originals.attempt().into(),
            original_sha256: originals.original_sha256().into(),
            scope_sha256: "a".repeat(64),
            phases: vec![],
            live_sha256: None,
        },
        session_index::RestorePlan {
            id: uuid::Uuid::new_v4().to_string(),
            sha256: "b".repeat(64),
        },
        Instant::now() + Duration::from_secs(10),
    );
    let error = result
        .err()
        .expect("an existing initialized directory is not original absence");
    assert!(error.to_string().contains("whole-root restoration proof"));
    assert!(!originals
        .directory
        .path()
        .join("completion-readback.json")
        .exists());
    assert!(f.client().exists());
}
#[test]
fn root_absence_reopened_index_facts_do_not_reconstruct_the_original_capture_custody() {
    let f = Fixture::new();
    let reserved = f.index.reserve_first_root_capture(&f.head).unwrap();
    drop(f.index);
    let index = session_index::Index::open_existing(&f.context, &f.data, &f.profile.join(".codex"))
        .unwrap();
    assert!(index.reserve_first_root_capture(&reserved).is_err());
    let permit = index.capture_reservation(&reserved).unwrap();
    assert!(Originals::begin_reserved_profile(
        &f.data,
        &f.profile.join(".codex"),
        &f.profile,
        &permit
    )
    .is_err());
    assert!(!f.data.join("manager-state/client-originals").exists());
}

#[test]
fn root_absence_real_capture_crash_keeps_partial_records_without_making_them_a_registered_original()
{
    use std::collections::BTreeMap;
    fn bytes(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        fn walk(root: &Path, at: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
            for e in fs::read_dir(at).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    walk(root, &p, out);
                } else {
                    out.insert(p.strip_prefix(root).unwrap().into(), fs::read(p).unwrap());
                }
            }
        }
        let mut out = BTreeMap::new();
        walk(root, root, &mut out);
        out
    }
    const CHILD: &str = "OCXD024_ROOT_ORIGINAL_CRASH";
    if let Some(root) = std::env::var_os(CHILD) {
        let root = PathBuf::from(root);
        let data = root.join("desktop");
        let profile = root.join("profile");
        let home = profile.join("home");
        for p in [&data, &profile, &home] {
            fs::DirBuilder::new().mode(0o700).create(p).unwrap();
        }
        let context = crate::state::ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(
            ),
            working_directory: profile.clone(),
            opencodex_home: home,
        };
        let published =
            root_creation::reserve(&data, &profile, Instant::now() + Duration::from_secs(10))
                .unwrap()
                .publish()
                .unwrap();
        let (index, preparing) =
            session_index::Index::register_first_root(&context, &data, published).unwrap();
        let reserved = index.reserve_first_root_capture(&preparing).unwrap();
        let authority = index.capture_reservation(&reserved).unwrap();
        let point = std::env::var("OCXD024_ROOT_ORIGINAL_POINT").unwrap();
        let _ = Originals::begin_profile_attempt(
            &data,
            &profile.join(".codex"),
            &profile,
            authority.attempt(),
            authority.root_absence().unwrap(),
            &|p| {
                authority.require(&data, &profile.join(".codex"), &profile)?;
                if p == point {
                    std::process::exit(91);
                }
                Ok(())
            },
        );
        panic!("capture boundary missed");
    }
    for point in [
        "directory_created",
        "codex_archived",
        "grok_archived",
        "original_published",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let out=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::root_originals_tests::root_absence_real_capture_crash_keeps_partial_records_without_making_them_a_registered_original","--nocapture"]).env(CHILD,&root).env("OCXD024_ROOT_ORIGINAL_POINT",point).output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(91),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let data = root.join("desktop");
        let profile = root.join("profile");
        let client = profile.join(".codex");
        let context = crate::state::ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(
            ),
            working_directory: profile.clone(),
            opencodex_home: profile.join("home"),
        };
        let before = bytes(&root);
        let head = session_index::inspect(&context, &data).unwrap().unwrap();
        assert_eq!(head.state, session_index::State::CaptureReserved);
        assert_eq!(head.revision, 2);
        assert!(head.references.is_none());
        let path = data
            .join("manager-state/client-originals")
            .join(&head.ticket)
            .join("original.json");
        assert_eq!(path.exists(), point == "original_published");
        let sha = if path.exists() {
            digest(&fs::read(&path).unwrap())
        } else {
            "a".repeat(64)
        };
        let index = session_index::Index::open_existing(&context, &data, &client).unwrap();
        let authority = index.capture_reservation(&head).unwrap();
        assert!(authority.reopen_root_originals(&sha, &[]).is_err());
        assert!(Originals::begin_reserved_profile(&data, &client, &profile, &authority).is_err());
        assert!(
            Originals::open_for_profile(&data, &client, &profile, &head.ticket, &sha, &[]).is_err()
        );
        drop(index);
        assert!(bytes(&root) == before);
        println!(
            "ROOT_ORIGINAL_CRASH_EVIDENCE {}",
            serde_json::json!({"accepted":false,"point":point,"childExitCode":out.status.code(),"indexState":head.state,"indexRevision":head.revision,"originalIndependentlyRegistered":false,"originalFilePresent":path.exists(),"files":before.into_iter().map(|(p,b)|serde_json::json!({"path":p,"bytes":b.len(),"sha256":digest(&b),"rawBase64":base64::Engine::encode(&base64::engine::general_purpose::STANDARD,b)})).collect::<Vec<_>>()})
        );
    }
}

#[test]
fn root_absence_captured_originals_are_independently_registered_with_their_scope_before_ready() {
    let f = Fixture::new();
    let reserved = f.index.reserve_first_root_capture(&f.head).unwrap();
    let permit = f.index.capture_reservation(&reserved).unwrap();
    let originals =
        Originals::begin_reserved_profile(&f.data, &f.client(), &f.profile, &permit).unwrap();
    let scope = serde_json::json!({"schemaVersion":1,"attempt":originals.attempt(),"originalSha256":originals.original_sha256(),"uid":unsafe{libc::geteuid()},"data":f.data,"dataIdentity":originals.data.identity(),"profile":f.profile,"profileIdentity":originals.profile.identity(),"home":f.context.opencodex_home,"homeIdentity":PinnedDirectory::open(&f.context.opencodex_home).unwrap().identity(),"client":f.client(),"clientIdentity":originals.client.identity(),"programRoot":f.data.parent().unwrap(),"selection":{"schemaVersion":1,"version":"2.99.0","platform":"darwin-arm64","imageSha256":"a".repeat(64)},"capturedAt":"2026-11-13T00:00:00Z","declaredRoutes":[]});
    let raw = serde_json::to_vec(&scope).unwrap();
    create(&originals.directory, "startup-scope.json", &raw).unwrap();
    let refs = session::References {
        attempt: originals.attempt().into(),
        original_sha256: originals.original_sha256().into(),
        scope_sha256: digest(&raw),
        phases: vec![],
        live_sha256: None,
    };
    let mut bad = refs.clone();
    bad.scope_sha256 = "b".repeat(64);
    assert!(f
        .index
        .record_first_root_originals(&f.context, &reserved, bad)
        .is_err());
    assert_eq!(f.index.head().unwrap(), Some(reserved.clone()));
    let captured = f
        .index
        .record_first_root_originals(&f.context, &reserved, refs.clone())
        .unwrap();
    println!(
        "ROOT_CAPTURE_REGISTRATION_EVIDENCE {}",
        serde_json::json!({"accepted":false,"scope":"storage/reference-only synthetic profile; no service permission", "head":captured,"rootOrigin":f.origin,"originalFrameRawBase64":base64::Engine::encode(&base64::engine::general_purpose::STANDARD,fs::read(originals.directory.path().join("original.json")).unwrap()),"scopeRawBase64":base64::Engine::encode(&base64::engine::general_purpose::STANDARD,&raw)})
    );

    assert_eq!(captured.state, session_index::State::Captured);
    assert_eq!(captured.revision, 3);
    assert!(captured.references.as_ref() == Some(&refs));
    assert!(f
        .index
        .advance(&captured, session_index::State::Ready, Some(refs.clone()))
        .is_err());
    drop(permit);
    drop(f.index);
    let index = session_index::Index::open_existing(&f.context, &f.data, &f.profile.join(".codex"))
        .unwrap();
    assert_eq!(index.head().unwrap(), Some(captured.clone()));
    let reopened = index.registered_root_originals(&captured).unwrap();
    assert!(reopened.original_root_absence.as_ref() == Some(&f.origin));

    assert!(index.reserve_start().is_err());
    drop(index);
    assert_eq!(
        session_index::inspect(&f.context, &f.data).unwrap(),
        Some(captured.clone())
    );
    let preview = session_index::preview::inspect_preview(&f.context, &f.data);
    assert_eq!(preview.status, session_index::preview::Status::Pending);
    assert_eq!(
        preview.original_evidence,
        session_index::preview::Evidence::Verified
    );
    assert_eq!(preview.phase, Some(session_index::State::Captured));
    assert!(
        !preview.can_finish_unchanged
            && !preview.can_cancel_preparation
            && !preview.can_prepare_plan
            && !preview.can_finalize
            && !preview.can_continue
            && !preview.can_rollback
    );

    let scope_path = originals.directory.path().join("startup-scope.json");
    fs::set_permissions(&scope_path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&scope_path, b"{}").unwrap();
    fs::set_permissions(&scope_path, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(session_index::inspect(&f.context, &f.data).is_err());
    assert!(
        session_index::Index::open_existing(&f.context, &f.data, &f.profile.join(".codex"))
            .is_err()
    );
    assert_eq!(fs::read(scope_path).unwrap(), b"{}");
}

#[test]
fn root_absence_execution_snapshot_accepts_only_actual_initial_claim_transaction_metadata() {
    let f = Fixture::new();
    let reserved = f.index.reserve_first_root_capture(&f.head).unwrap();
    let permit = f.index.capture_reservation(&reserved).unwrap();
    let claim = f.client().join(".opencodex-native-main.claim.sqlite");
    let file = crate::infrastructure::platform::create_new_private_file(&claim, 0o600).unwrap();
    drop(file);
    let connection =
        rusqlite::Connection::open_with_flags(&claim, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)
            .unwrap();
    connection
        .execute_batch("BEGIN EXCLUSIVE; CREATE TABLE synthetic_claim_only(v INTEGER)")
        .unwrap();
    let journal = f
        .client()
        .join(".opencodex-native-main.claim.sqlite-journal");
    assert!(journal.exists());
    let originals =
        Originals::begin_reserved_profile(&f.data, &f.client(), &f.profile, &permit).unwrap();
    let frame = originals.frame(Phase::Original).unwrap().0;
    assert!(frame.tree.first_root_execution_baseline());
    assert!(frame.original_root_absence.as_ref() == Some(&f.origin));
    let value = serde_json::to_value(&frame.tree).unwrap();
    let paths = value["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["path"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        vec![
            ".opencodex-native-main.claim.sqlite",
            ".opencodex-native-main.claim.sqlite-journal"
        ]
    );
    assert!(value["entries"]
        .as_array()
        .unwrap()
        .iter()
        .all(|e| e["kind"] == "coordination"));
    assert!(!connection.is_autocommit());
    connection.execute_batch("ROLLBACK").unwrap();
    connection.close().unwrap();
    assert!(!journal.exists());
}
