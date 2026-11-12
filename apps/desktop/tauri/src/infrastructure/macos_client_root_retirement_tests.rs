use super::*;
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    time::Duration,
};
pub(super) struct Fixture {
    _temp: Option<tempfile::TempDir>,
    pub(super) originals: Originals,
    pub(super) source: Head,
    pub(super) context: crate::state::ProcessContext,
    pub(super) _index: session_index::Index,
}
impl Fixture {
    pub(super) fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let mut value = Self::at(temp.path().canonicalize().unwrap());
        value._temp = Some(temp);
        value
    }
    pub(super) fn at(root: PathBuf) -> Self {
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
            opencodex_home: home.clone(),
        };
        let published =
            root_creation::reserve(&data, &profile, Instant::now() + Duration::from_secs(10))
                .unwrap()
                .publish()
                .unwrap();
        let (index, preparing) =
            session_index::Index::register_first_root(&context, &data, published).unwrap();
        let reserved = index.reserve_first_root_capture(&preparing).unwrap();
        let permit = index.capture_reservation(&reserved).unwrap();
        let originals =
            Originals::begin_reserved_profile(&data, &profile.join(".codex"), &profile, &permit)
                .unwrap();
        let scope = serde_json::json!({"schemaVersion":1,"attempt":originals.attempt(),"originalSha256":originals.original_sha256(),"uid":unsafe{libc::geteuid()},"data":data,"dataIdentity":originals.data.identity(),"profile":profile,"profileIdentity":originals.profile.identity(),"home":home,"homeIdentity":PinnedDirectory::open(&home).unwrap().identity(),"client":profile.join(".codex"),"clientIdentity":originals.client.identity(),"programRoot":root,"selection":{"schemaVersion":1,"version":"2.99.0","platform":"darwin-arm64","imageSha256":"a".repeat(64)},"capturedAt":"2026-11-13T00:00:00Z","declaredRoutes":[]});
        let bytes = serde_json::to_vec(&scope).unwrap();
        create(&originals.directory, "startup-scope.json", &bytes).unwrap();
        let refs = session::References {
            attempt: originals.attempt().into(),
            original_sha256: originals.original_sha256().into(),
            scope_sha256: digest(&bytes),
            phases: vec![],
            live_sha256: None,
        };
        let source = index
            .record_first_root_originals(&context, &reserved, refs)
            .unwrap();
        drop(permit);
        Self {
            _temp: None,
            originals,
            source,
            context,
            _index: index,
        }
    }
    pub(super) fn prepare(&self) -> Pending<'_> {
        Pending::prepare(&self.context, &self.originals, &self.source).unwrap()
    }
}
pub(super) fn storage_evidence(root: &Path) -> Vec<serde_json::Value> {
    fn walk(root: &Path, path: &Path, out: &mut Vec<serde_json::Value>) {
        for e in fs::read_dir(path).unwrap() {
            let p = e.unwrap().path();
            let meta = fs::symlink_metadata(&p).unwrap();
            if meta.is_dir() {
                walk(root, &p, out);
            } else {
                let file = File::open(&p).unwrap();
                let info = stat(&file).unwrap();
                let raw = fs::read(&p).unwrap();
                out.push(serde_json::json!({"path":p.strip_prefix(root).unwrap(),"identity":identity(&info),"mode":info.st_mode,"bytes":raw.len(),"sha256":digest(&raw),"rawBase64":base64::Engine::encode(&base64::engine::general_purpose::STANDARD,raw)}));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out
}
#[test]
fn whole_root_retirement_preserves_same_directory_and_logical_scope_and_can_restore_execution_root()
{
    let f = Fixture::new();
    let pending = f.prepare();
    let original = f.originals.frame(Phase::Original).unwrap().1;
    let identity = f.originals.client.identity().clone();
    let proof = pending
        .apply_checked(Direction::Retire, &|| Ok(()), &|_| Ok(()))
        .unwrap();
    assert_eq!(proof.plan_sha256(), pending.sha256());
    assert_eq!(proof.direction(), Direction::Retire);
    assert!(!proof.receipt_sha256().is_empty());
    assert!(!f.originals.profile.path().join(".codex").exists());
    pending.inspect_retired().unwrap();
    let reference = ArchiveReferences {
        source: f.source.clone(),
        original_root: f.originals.original_root_absence.clone().unwrap(),
        plan_id: pending.id().into(),
        plan_sha256: pending.sha256().into(),
    };
    let archive = open_archived_originals(
        f.originals.data.path(),
        f.originals.profile.path(),
        &reference,
    )
    .unwrap();
    let mut bad = reference.clone();
    bad.source.revision += 1;
    assert!(
        open_archived_originals(f.originals.data.path(), f.originals.profile.path(), &bad).is_err()
    );
    bad = reference.clone();
    bad.plan_sha256 = "a".repeat(64);
    assert!(
        open_archived_originals(f.originals.data.path(), f.originals.profile.path(), &bad).is_err()
    );
    let foreign = tempfile::tempdir().unwrap();
    assert!(open_archived_originals(
        f.originals.data.path(),
        &foreign.path().canonicalize().unwrap(),
        &reference
    )
    .is_err());
    assert_eq!(archive.client.identity(), &identity);
    assert_eq!(
        archive.logical_client(),
        f.originals.profile.path().join(".codex")
    );
    assert_eq!(archive.frame(Phase::Original).unwrap().1, original);
    session::verify_index_scope(
        &archive,
        f.source.references.as_ref().unwrap(),
        &f.context.opencodex_home,
    )
    .unwrap();
    assert!(archive.seal(Phase::Ready).is_err());
    let rolled = pending
        .apply_checked(Direction::RestoreExecutionRoot, &|| Ok(()), &|_| Ok(()))
        .unwrap();
    assert_eq!(rolled.direction(), Direction::RestoreExecutionRoot);
    assert_eq!(
        PinnedDirectory::open(&f.originals.profile.path().join(".codex"))
            .unwrap()
            .identity(),
        &identity
    );
    assert!(f.originals.frame(Phase::Original).is_ok());
    assert_eq!(f._index.head().unwrap(), Some(f.source.clone()));
    pending
        .apply_checked(Direction::Retire, &|| Ok(()), &|_| Ok(()))
        .unwrap();
    pending.inspect_retired().unwrap();
    println!(
        "ROOT_RETIRE_POSITIVE_EVIDENCE {}",
        serde_json::json!({"accepted":false,"direction":"retire","rootAbsent":true,"archivedRootIdentity":identity,"sourceHead":f.source,"planSha256":pending.sha256(),"files":storage_evidence(f.originals.data.path().parent().unwrap())})
    );
}
#[test]
fn whole_root_retirement_never_replaces_current_root_or_accepts_new_contents() {
    let f = Fixture::new();
    let pending = f.prepare();
    fs::write(
        f.originals.client.path().join("auth.json"),
        b"external synthetic user data",
    )
    .unwrap();
    assert!(pending
        .apply_checked(Direction::Retire, &|| Ok(()), &|_| Ok(()))
        .is_err());
    assert_eq!(
        fs::read(f.originals.client.path().join("auth.json")).unwrap(),
        b"external synthetic user data"
    );
    let f = Fixture::new();
    let pending = f.prepare();
    pending
        .apply_checked(Direction::Retire, &|| Ok(()), &|_| Ok(()))
        .unwrap();
    fs::DirBuilder::new()
        .mode(0o700)
        .create(f.originals.profile.path().join(".codex"))
        .unwrap();
    fs::write(
        f.originals.profile.path().join(".codex/new-user"),
        b"keep me",
    )
    .unwrap();
    assert!(pending.inspect_retired().is_err());
    assert!(pending
        .apply_checked(Direction::RestoreExecutionRoot, &|| Ok(()), &|_| Ok(()))
        .is_err());
    assert_eq!(
        fs::read(f.originals.profile.path().join(".codex/new-user")).unwrap(),
        b"keep me"
    );
    assert!(f.originals.directory.path().join(pending.slot()).is_dir());
}
#[test]
fn whole_root_retirement_native_gap_returns_external_replacement_without_terminal_success() {
    let f = Fixture::new();
    let pending = f.prepare();
    let once = std::cell::Cell::new(false);
    let path = f.originals.profile.path().join(".codex");
    assert!(pending
        .apply_checked(Direction::Retire, &|| Ok(()), &|p| {
            if p == "native_gap" && !once.replace(true) {
                fs::rename(
                    &path,
                    f.originals.profile.path().join("displaced-created-root"),
                )?;
                fs::DirBuilder::new().mode(0o700).create(&path)?;
                fs::write(path.join("outside"), b"external directory")?;
            }
            Ok(())
        })
        .is_err());
    assert_eq!(
        fs::read(path.join("outside")).unwrap(),
        b"external directory"
    );
    assert!(f
        .originals
        .profile
        .path()
        .join("displaced-created-root")
        .is_dir());
    assert!(!f.originals.directory.path().join(pending.slot()).exists());
    assert!(!f
        .originals
        .directory
        .path()
        .join(format!(
            "root-retirement-{}-Retire-readback.json",
            pending.id()
        ))
        .exists());
}
#[test]
fn whole_root_retirement_checks_authority_before_and_after_move_without_claiming_cleanup_success() {
    let f = Fixture::new();
    let pending = f.prepare();
    let count = std::cell::Cell::new(0);
    assert!(pending
        .apply_checked(
            Direction::Retire,
            &|| {
                count.set(count.get() + 1);
                if count.get() == 4 {
                    Err(denied("expired after move"))
                } else {
                    Ok(())
                }
            },
            &|_| Ok(())
        )
        .is_err());
    assert!(!f.originals.profile.path().join(".codex").exists());
    assert!(f.originals.directory.path().join(pending.slot()).is_dir());
    assert!(!f
        .originals
        .directory
        .path()
        .join(format!(
            "root-retirement-{}-Retire-readback.json",
            pending.id()
        ))
        .exists());
    pending
        .apply_checked(Direction::Retire, &|| Ok(()), &|_| Ok(()))
        .unwrap();
    let permit = MutationPermit {
        check: &|| Ok(()),
        deadline: Instant::now() + Duration::from_secs(10),
    };
    assert!(pending
        .apply(Direction::Retire, &permit, Instant::now())
        .is_err());
}
#[test]
fn whole_root_retirement_archive_changes_and_unbound_plan_digest_refuse() {
    let f = Fixture::new();
    let pending = f.prepare();
    pending
        .apply_checked(Direction::Retire, &|| Ok(()), &|_| Ok(()))
        .unwrap();
    fs::write(
        f.originals
            .directory
            .path()
            .join(pending.slot())
            .join("outside-file"),
        b"keep archive addition",
    )
    .unwrap();
    assert!(pending.inspect_retired().is_err());
    assert!(pending
        .apply_checked(Direction::RestoreExecutionRoot, &|| Ok(()), &|_| Ok(()))
        .is_err());
    assert!(Pending::open(&f.originals, pending.id(), &"a".repeat(64)).is_err());
    let path = f
        .originals
        .directory
        .path()
        .join(format!("root-retirement-{}.json", pending.id()));
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&path, b"{}").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(pending.inspect_retired().is_err());
    assert_eq!(fs::read(path).unwrap(), b"{}");
}
#[test]
fn whole_root_retirement_real_process_exit_preserves_resumable_same_root_without_rebaselining() {
    const CHILD: &str = "OCXD024_ROOT_RETIRE_CRASH";
    if let Some(path) = std::env::var_os(CHILD) {
        let root = PathBuf::from(path);
        let f = Fixture::at(root.clone());
        let pending = f.prepare();
        fs::write(root.join("independent-move.json"),encode(&serde_json::json!({"id":pending.id(),"sha256":pending.sha256(),"head":f.source,"origin":f.originals.original_root_absence})).unwrap()).unwrap();
        let point = std::env::var("OCXD024_ROOT_RETIRE_POINT").unwrap();
        pending
            .apply_checked(Direction::Retire, &|| Ok(()), &|p| {
                if p == point {
                    std::process::exit(91);
                }
                Ok(())
            })
            .unwrap();
        panic!("exit boundary missed");
    }
    for point in ["intent", "after_native", "before_readback", "readback"] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let out=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::root_retirement::tests::whole_root_retirement_real_process_exit_preserves_resumable_same_root_without_rebaselining","--nocapture"]).env(CHILD,&root).env("OCXD024_ROOT_RETIRE_POINT",point).output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(91),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let value: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("independent-move.json")).unwrap()).unwrap();
        let head: Head = serde_json::from_value(value["head"].clone()).unwrap();
        let origin: root_creation::References =
            serde_json::from_value(value["origin"].clone()).unwrap();
        let refs = head.references.as_ref().unwrap();
        let data = PinnedDirectory::open(&root.join("desktop")).unwrap();
        let profile = PinnedDirectory::open(&root.join("profile")).unwrap();
        let directory = data
            .child_private("manager-state", false)
            .unwrap()
            .child_private("client-originals", false)
            .unwrap()
            .child_private(&refs.attempt, false)
            .unwrap();
        let id = value["id"].as_str().unwrap();
        let retired = point != "intent";
        let slot = format!("root-retired-{id}");
        let client = PinnedDirectory::open(&if retired {
            directory.path().join(&slot)
        } else {
            profile.path().join(".codex")
        })
        .unwrap();
        let originals = Originals {
            profile,
            data,
            client,
            directory,
            attempt: refs.attempt.clone(),
            original_sha256: refs.original_sha256.clone(),
            original_root_absence: Some(origin),
            retired_root_slot: if retired { Some(slot) } else { None },
            known: std::sync::Mutex::new([Some(refs.original_sha256.clone()), None, None, None]),
        };
        let pending = Pending::open(&originals, id, value["sha256"].as_str().unwrap()).unwrap();
        pending
            .apply_checked(Direction::Retire, &|| Ok(()), &|_| Ok(()))
            .unwrap();
        pending.inspect_retired().unwrap();
        assert!(!root.join("profile/.codex").exists());
        // A newly opened archival view must also support reversal. This is
        // storage-only replay from independent source/plan, never native permit.
        pending
            .apply_checked(Direction::RestoreExecutionRoot, &|| Ok(()), &|_| Ok(()))
            .unwrap();
        assert!(root.join("profile/.codex").is_dir());
        pending
            .apply_checked(Direction::Retire, &|| Ok(()), &|_| Ok(()))
            .unwrap();
        assert!(!root.join("profile/.codex").exists());

        println!(
            "ROOT_RETIRE_CRASH_EVIDENCE {}",
            serde_json::json!({"accepted":false,"point":point,"actualChildExitCode":91,"sameRootPreserved":true,"rootAbsentAfterResume":true,"sourceHead":head,"planId":id,"planSha256":value["sha256"],"scope":"synthetic storage move, no production lifecycle authority","files":storage_evidence(&root)})
        );
    }
}

#[test]
fn whole_root_retirement_cannot_return_success_if_absence_changes_after_receipt() {
    let f = Fixture::new();
    let pending = f.prepare();
    let path = f.originals.profile.path().join(".codex");
    assert!(pending
        .apply_checked(Direction::Retire, &|| Ok(()), &|point| {
            if point == "readback" {
                fs::DirBuilder::new().mode(0o700).create(&path)?;
                fs::write(path.join("outside"), b"created after receipt")?;
            }
            Ok(())
        })
        .is_err());
    assert_eq!(
        fs::read(path.join("outside")).unwrap(),
        b"created after receipt"
    );
    assert!(pending.inspect_retired().is_err());
    assert!(f.originals.directory.path().join(pending.slot()).is_dir());
}
#[test]
fn whole_root_retirement_foreign_archive_destination_is_preserved_and_missing_intent_is_not_adopted(
) {
    let f = Fixture::new();
    let pending = f.prepare();
    let slot = f.originals.directory.path().join(pending.slot());
    fs::DirBuilder::new().mode(0o700).create(&slot).unwrap();
    fs::write(slot.join("outside"), b"foreign archive").unwrap();
    assert!(pending
        .apply_checked(Direction::Retire, &|| Ok(()), &|_| Ok(()))
        .is_err());
    assert!(f.originals.profile.path().join(".codex").exists());
    assert_eq!(fs::read(slot.join("outside")).unwrap(), b"foreign archive");
    let f = Fixture::new();
    let pending = f.prepare();
    let src = name(OsStr::new(".codex")).unwrap();
    let dst = name(OsStr::new(&pending.slot())).unwrap();
    assert_eq!(
        unsafe {
            libc::renameatx_np(
                f.originals.profile.file().as_raw_fd(),
                src.as_ptr(),
                f.originals.directory.file().as_raw_fd(),
                dst.as_ptr(),
                libc::RENAME_EXCL,
            )
        },
        0
    );
    assert!(pending
        .apply_checked(Direction::Retire, &|| Ok(()), &|_| Ok(()))
        .is_err());
    assert!(f.originals.directory.path().join(pending.slot()).is_dir());
    assert!(!f
        .originals
        .directory
        .path()
        .join(format!(
            "root-retirement-{}-Retire-readback.json",
            pending.id()
        ))
        .exists());
}

#[test]
fn whole_root_retirement_caller_cannot_extend_the_owning_native_permit_deadline() {
    let f = Fixture::new();
    let pending = f.prepare();
    let permit = MutationPermit {
        check: &|| Ok(()),
        deadline: Instant::now(),
    };
    assert!(pending
        .apply(
            Direction::Retire,
            &permit,
            Instant::now() + Duration::from_secs(30)
        )
        .is_err());
    assert!(f.originals.profile.path().join(".codex").exists());
    assert!(!f.originals.directory.path().join(pending.slot()).exists());
}
