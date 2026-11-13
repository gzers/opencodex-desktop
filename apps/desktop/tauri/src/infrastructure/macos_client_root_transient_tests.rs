//! Explicit synthetic writer markers exercise plan/storage semantics only.
//! No synthetic Image or callback is a production Guard or movement permit.
use super::*;
use std::{fs, os::unix::fs::PermissionsExt, time::Duration};
use tests::Fixture;
const MARKER: &str = "config.toml.ocx-write.lock";
fn marker(f: &Fixture) -> Image {
    let path = f.originals.client.path().join(MARKER);
    let bytes = serde_json::to_vec(
        &serde_json::json!({"pid":std::process::id(),"token":"1234567890abcdef"}),
    )
    .unwrap();
    fs::write(&path, bytes).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    f.originals.client.config_write_lock_image().unwrap()
}
fn plan<'a>(f: &'a Fixture, image: Image) -> Pending<'a> {
    Pending::prepare_with_marker(&f.context, &f.originals, &f.source, Some(image)).unwrap()
}
fn settle(f: &Fixture, p: &Pending<'_>) -> io::Result<Readback> {
    p.apply_checked_inner(
        Direction::Retire,
        &|| Ok(()),
        &|phase| {
            if phase == "settle_writer" {
                fs::remove_file(f.originals.directory.path().join(p.slot()).join(MARKER))?;
            }
            Ok(())
        },
        true,
    )
}
#[test]
fn transient_root_marker_is_exact_and_release_is_required_before_terminal_receipt() {
    let f = Fixture::new();
    let image = marker(&f);
    assert!(Pending::prepare(&f.context, &f.originals, &f.source).is_err());
    let p = plan(&f, image.clone());
    let loaded = p.load().unwrap();
    assert_eq!(loaded.schema_version, 2);
    assert!(loaded.write_guard == Some(image));
    assert!(!loaded.released_tree().unwrap().has_config_guard());
    let registered = f
        ._index
        .record_root_retirement(&f.source, p.id(), p.sha256())
        .unwrap();
    assert!(p
        .apply_checked(Direction::Retire, &|| Ok(()), &|_| Ok(()))
        .is_err());
    assert!(p
        .apply_checked_inner(Direction::Retire, &|| Ok(()), &|_| Ok(()), true)
        .is_err());
    assert!(!f.originals.profile.path().join(".codex").exists());
    assert!(f
        .originals
        .directory
        .path()
        .join(p.slot())
        .join(MARKER)
        .exists());
    assert!(!f
        .originals
        .directory
        .path()
        .join(format!("root-retirement-{}-Retire-readback.json", p.id()))
        .exists());
    // Storage retry uses the exact prior plan, never rebaselines current bytes.
    let readback = settle(&f, &p).unwrap();
    let proof = p
        .authorized_readback(readback, Instant::now() + Duration::from_secs(10))
        .unwrap();
    let closed = f._index.close_root_absence(&registered, &proof).unwrap();
    assert_eq!(closed.state, session_index::State::RootAbsent);
    p.verify_retire_readback(closed.completion_sha256.as_deref().unwrap())
        .unwrap();
    println!(
        "ROOT_TRANSIENT_EVIDENCE {}",
        serde_json::json!({"accepted":false,"source":f.source,"prepared":registered,"closed":closed,"storageOnlySyntheticMarker":true,"files":tests::storage_evidence(f.originals.data.path().parent().unwrap())})
    );
}
#[test]
fn transient_root_marker_does_not_hide_other_user_files_or_substituted_lock_inodes() {
    for case in ["extra-file", "same-bytes-new-inode", "wrong-marker", "mode"] {
        let f = Fixture::new();
        let image = marker(&f);
        let path = f.originals.client.path().join(MARKER);
        if case == "extra-file" {
            fs::write(f.originals.client.path().join("auth.json"), b"outside").unwrap();
        }
        if case == "same-bytes-new-inode" {
            let bytes = fs::read(&path).unwrap();
            fs::rename(&path, f.originals.profile.path().join("retained-marker")).unwrap();
            fs::write(&path, bytes).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        }
        if case == "wrong-marker" {
            fs::write(&path, b"different-token").unwrap();
        }
        if case == "mode" {
            fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        }
        assert!(
            Pending::prepare_with_marker(&f.context, &f.originals, &f.source, Some(image)).is_err()
        );
        assert!(path.exists());
        if case == "extra-file" {
            assert_eq!(
                fs::read(f.originals.client.path().join("auth.json")).unwrap(),
                b"outside"
            );
        }
    }
}
#[test]
fn transient_root_rewritten_private_plan_cannot_adopt_new_user_contents_before_move() {
    let f = Fixture::new();
    let image = marker(&f);
    let p = plan(&f, image);
    fs::write(
        f.originals.client.path().join("auth.json"),
        b"outside user file",
    )
    .unwrap();
    let mut rewritten = p.load().unwrap();
    rewritten.execution_tree = TreeSnapshot::read(&f.originals.client).unwrap();
    let raw = encode(&rewritten).unwrap();
    let path = f
        .originals
        .directory
        .path()
        .join(format!("root-retirement-{}.json", p.id()));
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&path, &raw).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
    let rewritten = Pending::open(&f.originals, p.id(), &digest(&raw)).unwrap();
    assert!(rewritten.inspect_live().is_err());
    assert!(f
        ._index
        .record_root_retirement(&f.source, rewritten.id(), rewritten.sha256())
        .is_err());
    assert_eq!(f._index.head().unwrap(), Some(f.source.clone()));
    assert_eq!(
        fs::read(f.originals.client.path().join("auth.json")).unwrap(),
        b"outside user file"
    );
}
#[test]
fn transient_root_archive_tolerates_only_exact_held_or_exact_released_tree_for_readonly_pending() {
    let f = Fixture::new();
    let image = marker(&f);
    let p = Pending::prepare_with_marker(&f.context, &f.originals, &f.source, Some(image)).unwrap();
    let registered = f
        ._index
        .record_root_retirement(&f.source, p.id(), p.sha256())
        .unwrap();
    assert!(p
        .apply_checked_inner(
            Direction::Retire,
            &|| Ok(()),
            &|phase| if phase == "settle_writer" {
                Err(denied("synthetic parent exit before release"))
            } else {
                Ok(())
            },
            true
        )
        .is_err());
    drop(f._index);
    assert_eq!(
        session_index::inspect(&f.context, f.originals.data.path()).unwrap(),
        Some(registered.clone())
    );
    let lock = f.originals.directory.path().join(p.slot()).join(MARKER);
    let bytes = fs::read(&lock).unwrap();
    fs::remove_file(&lock).unwrap();
    assert_eq!(
        session_index::inspect(&f.context, f.originals.data.path()).unwrap(),
        Some(registered)
    );
    let preview = session_index::preview::inspect_preview(&f.context, f.originals.data.path());
    assert_eq!(preview.status, session_index::preview::Status::Pending);
    assert_eq!(
        preview.completion_evidence,
        session_index::preview::Evidence::NotRecorded
    );
    assert!(!preview.can_finalize && !preview.can_continue);
    fs::write(&lock, bytes).unwrap();
    fs::set_permissions(&lock, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(
        session_index::inspect(&f.context, f.originals.data.path()).is_err(),
        "same bytes at a new inode are not the bound live marker"
    );
}
#[test]
fn transient_root_post_release_new_marker_or_user_content_refuses_without_overwrite() {
    for case in ["marker", "outside", "new-root"] {
        let f = Fixture::new();
        let image = marker(&f);
        let p = plan(&f, image);
        let archive = f.originals.directory.path().join(p.slot());
        assert!(p
            .apply_checked_inner(
                Direction::Retire,
                &|| Ok(()),
                &|phase| {
                    if phase == "settle_writer" {
                        fs::remove_file(archive.join(MARKER))?;
                    }
                    if phase == "readback" {
                        match case {
                            "marker" => fs::write(archive.join(MARKER), b"new holder")?,
                            "outside" => fs::write(archive.join("external"), b"outside")?,
                            "new-root" => {
                                fs::create_dir(f.originals.profile.path().join(".codex"))?;
                            }
                            _ => unreachable!(),
                        }
                    }
                    Ok(())
                },
                true
            )
            .is_err());
        assert!(archive.is_dir());
        if case == "outside" {
            assert_eq!(fs::read(archive.join("external")).unwrap(), b"outside");
        }
    }
}
#[test]
fn transient_root_late_foreign_move_is_recorded_before_reanchor_refusal_and_preserved() {
    let f = Fixture::new();
    let image = marker(&f);
    let p = plan(&f, image);
    let logical = f.originals.client.path();
    let retained = f.originals.profile.path().join("retained-original-root");
    assert!(p
        .apply_checked_inner(
            Direction::Retire,
            &|| if logical.exists() {
                Ok(())
            } else {
                Err(denied("current root authority unknown"))
            },
            &|phase| {
                if phase == "before_rename" {
                    fs::rename(logical, &retained)?;
                    fs::create_dir(logical)?;
                    fs::write(logical.join("outside"), b"external root")?;
                }
                if phase == "after_native" {
                    return Err(denied("successful reanchor must not run on foreign object"));
                }
                Ok(())
            },
            true
        )
        .is_err());
    let conflicts: Vec<_> = fs::read_dir(f.originals.directory.path())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().contains("-conflict-"))
        .collect();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(
        fs::read(f.originals.directory.path().join(p.slot()).join("outside")).unwrap(),
        b"external root"
    );
    assert!(retained.join(MARKER).exists());
    assert!(!logical.exists());
    assert!(!f
        .originals
        .directory
        .path()
        .join(format!("root-retirement-{}-Retire-readback.json", p.id()))
        .exists());
}
#[test]
fn transient_root_real_process_exit_preserves_marker_release_state_without_terminal_adoption() {
    const ENV: &str = "OCXD_ROOT_TRANSIENT_CHILD";
    if let Some(root) = std::env::var_os(ENV) {
        let f = Fixture::at(PathBuf::from(root));
        let image = marker(&f);
        let p = plan(&f, image);
        let registered = f
            ._index
            .record_root_retirement(&f.source, p.id(), p.sha256())
            .unwrap();
        fs::write(f.originals.data.path().parent().unwrap().join("known-root-move.json"),encode(&serde_json::json!({"head":registered,"source":f.source,"id":p.id(),"sha256":p.sha256()})).unwrap()).unwrap();
        let point = std::env::var("OCXD_ROOT_TRANSIENT_POINT").unwrap();
        if point == "registered" {
            std::process::exit(91);
        }
        p.apply_checked_inner(
            Direction::Retire,
            &|| Ok(()),
            &|phase| {
                if phase == "settle_writer" {
                    fs::remove_file(f.originals.directory.path().join(p.slot()).join(MARKER))?;
                    if point == "released" {
                        std::process::exit(91);
                    }
                }
                if phase == point {
                    std::process::exit(91);
                }
                Ok(())
            },
            true,
        )
        .unwrap();
        panic!("exit not reached");
    }
    for point in ["registered", "after_native", "released", "readback"] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let output=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::root_retirement::transient_tests::transient_root_real_process_exit_preserves_marker_release_state_without_terminal_adoption","--nocapture","--test-threads=1"]).env(ENV,&root).env("OCXD_ROOT_TRANSIENT_POINT",point).output().unwrap();
        assert_eq!(
            output.status.code(),
            Some(91),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let document: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("known-root-move.json")).unwrap()).unwrap();
        let expected: Head = serde_json::from_value(document["head"].clone()).unwrap();
        let context = crate::state::ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(
            ),
            working_directory: root.join("profile"),
            opencodex_home: root.join("profile/home"),
        };
        assert_eq!(
            session_index::inspect(&context, &root.join("desktop")).unwrap(),
            Some(expected)
        );
        let preview = session_index::preview::inspect_preview(&context, &root.join("desktop"));
        assert_eq!(preview.status, session_index::preview::Status::Pending);
        assert_eq!(
            preview.completion_evidence,
            session_index::preview::Evidence::NotRecorded
        );
        println!(
            "ROOT_TRANSIENT_EXIT_EVIDENCE {}",
            serde_json::json!({"accepted":false,"point":point,"actualWaitExitCode":91,"known":document,"storageOnlySyntheticMarker":true,"files":tests::storage_evidence(&root)})
        );
    }
}

/// Actual official Guard + NativeLease composed with explicit synthetic storage
/// lifecycle permission. This does not prove normal service Stop or Start.
pub(crate) fn official_round(root: &Path, program: &Path) {
    use crate::infrastructure::official_client_coordination::{
        native::with_native_relocation_coordination, Context,
    };
    let fixtures = root.join("transient");
    fs::create_dir(&fixtures).unwrap();
    let mut f = Fixture::at(fixtures.clone());
    f.context.runtime = crate::modules::runtime::RuntimeHandle::initialize_versioned(
        f.originals.data.path(),
        program,
    );
    let control = fixtures.join("native-control");
    fs::create_dir(&control).unwrap();
    fs::set_permissions(&control, fs::Permissions::from_mode(0o700)).unwrap();
    let context = Context {
        runtime: f.context.runtime.clone(),
        user_profile: f.context.working_directory.clone(),
        codex_home: f.originals.client.path().to_path_buf(),
        opencodex_home: f.context.opencodex_home.clone(),
        control_root: control,
    };
    let original_bytes = f.originals.frame(Phase::Original).unwrap().1;
    let started = Instant::now();
    let deadline = started + Duration::from_secs(10);
    let late = std::env::var_os("OCXD_NATIVE_ROOT_TRANSIENT_LATE").is_some();
    let result = with_native_relocation_coordination(&context, deadline, |native| {
        let mut guard = native.config_write_guard()?;
        let pending =
            Pending::prepare_guarded(&f.context, &f.originals, &f.source, native, &mut guard)
            .map_err(|e| {
                let detail=serde_json::json!({"accepted":false,"scope":"owned synthetic transient fixture only","phase":"prepare-plan","kind":format!("{:?}",e.kind()),"fixedReason":e.to_string(),"originalTree":f.originals.frame(Phase::Original).ok().map(|v|v.0.tree),"currentTree":TreeSnapshot::read(&f.originals.client).ok()});
                fs::write(root.join("root-transient-failure.json"),encode(&detail).unwrap()).unwrap();
                eprintln!("ROOT_TRANSIENT_REFUSAL {detail}");
                crate::errors::AppError::RuntimeManaged {
                    code: "test_transient_plan".into(),
                    detail: "synthetic transient plan refused".into(),
            }})?;
        let prepared = f
            ._index
            .record_root_retirement(&f.source, pending.id(), pending.sha256())
            .unwrap();
        // Explicit test-only authority, never exported as a production factory.
        let check = || time(deadline);
        let permit = MutationPermit {
            check: &check,
            deadline,
        };
        if late {
            let retained = f.originals.profile.path().join("retained-original-root");
            let moved = pending.apply_guarded_checked(&permit, native, guard, deadline, &|phase| {
                if phase == "before_rename" {
                    fs::rename(f.originals.client.path(), &retained)?;
                    fs::create_dir(f.originals.client.path())?;
                    fs::write(
                        f.originals.client.path().join("external"),
                        b"late external root",
                    )?;
                }
                Ok(())
            });
            assert!(moved.is_err());
            assert!(native.check().is_err());
            assert!(f._index.head().is_err());
            assert_eq!(
                fs::read(
                    f.originals
                        .directory
                        .path()
                        .join(pending.slot())
                        .join("external")
                )
                .unwrap(),
                b"late external root"
            );
            assert!(retained.join(MARKER).exists());
            let conflicts: Vec<_> = fs::read_dir(f.originals.directory.path())
                .unwrap()
                .filter_map(Result::ok)
                .filter(|e| e.file_name().to_string_lossy().contains("-conflict-"))
                .collect();
            assert_eq!(conflicts.len(), 1);
            assert!(!f
                .originals
                .directory
                .path()
                .join(format!(
                    "root-retirement-{}-Retire-readback.json",
                    pending.id()
                ))
                .exists());
            fs::write(root.join("root-transient-references.json"),encode(&serde_json::json!({"source":f.source,"prepared":prepared,"closed":null,"originalRoot":f.originals.original_root_absence,"planId":pending.id(),"planSha256":pending.sha256(),"expectedRefusal":true,"conflictRecordedBeforeReanchor":true})).unwrap()).unwrap();
            return Err(crate::errors::AppError::RuntimeManaged {
                code: "test_expected_late_refusal".into(),
                detail: "late replacement preserved with unknown authority".into(),
            });
        }
        let proof = pending
            .apply_guarded(&permit, native, guard, deadline)
            .map_err(|e| {
                let actual=if f.originals.client.path().exists(){f.originals.client.path().to_path_buf()}else{f.originals.directory.path().join(pending.slot())};
                let detail=serde_json::json!({"accepted":false,"scope":"owned synthetic transient fixture only","phase":"apply-guarded","kind":format!("{:?}",e.kind()),"fixedReason":e.to_string(),"currentTree":PinnedDirectory::open(&actual).ok().and_then(|p|TreeSnapshot::read(&p).ok())});
                fs::write(root.join("root-transient-failure.json"),encode(&detail).unwrap()).unwrap();eprintln!("ROOT_TRANSIENT_REFUSAL {detail}");
                crate::errors::AppError::RuntimeManaged {
                code: "test_transient_apply".into(),
                detail: "synthetic transient apply refused".into(),
            }})?;
        let closed = f._index.close_root_absence(&prepared, &proof).unwrap();
        assert_eq!(closed.state, session_index::State::RootAbsent);
        native.check()?;
        let archive = pending.archived_originals().unwrap();
        assert!(!archive.client.path().join(MARKER).exists());
        assert_eq!(archive.frame(Phase::Original).unwrap().1, original_bytes);
        pending
            .verify_retire_readback(closed.completion_sha256.as_deref().unwrap())
            .unwrap();
        fs::write(root.join("root-transient-references.json"),encode(&serde_json::json!({"source":f.source,"prepared":prepared,"closed":closed,"originalRoot":f.originals.original_root_absence,"planId":pending.id(),"planSha256":pending.sha256()})).unwrap()).unwrap();
        Ok(())
    });
    let report = serde_json::json!({"accepted":false,"result":if result.is_ok()!=late {"passed"}else{"failed"},"expectedLateRefusal":late,"elapsedMs":started.elapsed().as_millis(),"budgetMs":10000,"scope":"actual unchanged official Guard and NativeLease with synthetic storage Original/lifecycle permit; no service or production root authority"});
    fs::write(
        root.join("native-summary.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    if late {
        assert!(result.is_err());
    } else {
        result.unwrap();
    }
    assert!(started.elapsed() < Duration::from_secs(10));
    drop(f._index);
    if late {
        assert!(session_index::inspect(&f.context, f.originals.data.path()).is_err());
        fs::write(
            root.join("root-transient-original-bytes.json"),
            encode(
                &serde_json::json!({"accepted":false,"files":tests::storage_evidence(&fixtures)}),
            )
            .unwrap(),
        )
        .unwrap();
        println!("OFFICIAL_ROOT_TRANSIENT_EVIDENCE {report}");
        return;
    }
    let head = session_index::inspect(&f.context, f.originals.data.path())
        .unwrap()
        .unwrap();
    assert_eq!(head.state, session_index::State::RootAbsent);
    let preview = session_index::preview::inspect_preview(&f.context, f.originals.data.path());
    assert_eq!(preview.status, session_index::preview::Status::Pending);
    assert!(!preview.can_continue && !preview.can_finalize);
    let files = tests::storage_evidence(&fixtures);
    fs::write(
        root.join("root-transient-original-bytes.json"),
        encode(&serde_json::json!({"accepted":false,"files":files})).unwrap(),
    )
    .unwrap();
    println!("OFFICIAL_ROOT_TRANSIENT_EVIDENCE {report}");
}
