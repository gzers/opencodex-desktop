use super::*;
#[test]
fn diagnostic_writer_refusal_preserves_original_permission_kind_and_message() {
    for error in [
        crate::errors::AppError::Timeout,
        crate::errors::AppError::RuntimeManaged {
            code: "client_native_coordination_expired".into(),
            detail: "never print this detail".into(),
        },
    ] {
        let mapped = writer_refusal(error, "test.guard", "retained Codex writer changed");
        assert_eq!(mapped.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(mapped.to_string(), "retained Codex writer changed");
    }
}
use serde_json::json;
use std::{
    cell::Cell,
    fs,
    os::unix::fs::{symlink, DirBuilderExt, PermissionsExt},
};

struct Fixture {
    _temp: Option<tempfile::TempDir>,
    originals: Originals,
    evidence: Evidence,
}
fn wire(value: serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(&value).unwrap()
}
fn event(writer: &str, target: &str, intent: &str) -> Vec<u8> {
    let mut b = wire(
        json!({"at":"2026-11-11T10:00:01Z","pid":101,"ppid":100,"command":writer,"opencodexHome":"synthetic-captured-home","target":target,"outcome":"written","intent":intent,"writer":writer,"configSource":"file","routedBefore":0,"routedAfter":1}),
    );
    b.push(b'\n');
    b
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let mut f = Self::at(root);
        f._temp = Some(temp);
        f
    }
    fn at(root: PathBuf) -> Self {
        Self::at_with(root, |_| {})
    }
    fn at_with(root: PathBuf, decorate: impl FnOnce(&Path)) -> Self {
        let data = root.join("desktop");
        let client = root.join("codex 中文 space");
        for p in [&data, &client] {
            fs::DirBuilder::new().mode(0o700).create(p).unwrap();
        }
        fs::write(
            client.join(NAMES[0]),
            wire(json!({"models":[{"slug":"native-5.4","display_name":"original native"}]})),
        )
        .unwrap();
        fs::set_permissions(client.join(NAMES[0]), fs::Permissions::from_mode(0o640)).unwrap();
        decorate(&client.join(NAMES[0]));
        let originals = Originals::begin(&data, &client).unwrap();
        let ready = json!({"models":[{"slug":"native-5.4","display_name":"native projection"},{"slug":"channel/model","display_name":"routed"}]});
        fs::write(client.join(NAMES[0]), wire(ready.clone())).unwrap();
        fs::write(client.join(NAMES[1]),wire(json!({"fetched_at":"2000-01-01T00:00:00Z","client_version":"0.0.0","models":ready["models"]}))).unwrap();
        fs::write(
            client.join(NAMES[2]),
            event("startup-cache", "cache", "cache"),
        )
        .unwrap();
        for n in &NAMES[1..] {
            fs::set_permissions(client.join(n), fs::Permissions::from_mode(0o600)).unwrap();
        }
        originals.seal(Phase::Ready).unwrap();
        originals.seal(Phase::PreStop).unwrap();
        fs::write(client.join(NAMES[0]), wire(json!({"models":[]}))).unwrap();
        let mut audit = fs::read(client.join(NAMES[2])).unwrap();
        audit.extend(event("codex-restore", "catalog", "restore"));
        fs::write(client.join(NAMES[2]), audit).unwrap();
        originals.seal(Phase::Stopped).unwrap();
        let evidence = Evidence {
            declared_routes: Some(vec!["channel/model".into()]),
            retained_pids: vec![100, 101],
            audit_home: "synthetic-captured-home".into(),
            not_before: chrono::DateTime::parse_from_rfc3339("2026-11-11T10:00:00Z")
                .unwrap()
                .into(),
            not_after: chrono::DateTime::parse_from_rfc3339("2026-11-11T10:00:02Z")
                .unwrap()
                .into(),
        };
        Self {
            _temp: None,
            originals,
            evidence,
        }
    }
    fn prepare(&self) -> Pending<'_> {
        Pending::prepare_checked(&self.originals, self.evidence.clone(), &|| Ok(())).unwrap()
    }
    fn matches(&self, phase: Phase) -> bool {
        let actual = Capture::read(&self.originals.client).unwrap();
        let expected = self.originals.read(phase).unwrap();
        actual
            .images
            .iter()
            .zip(&expected.images)
            .all(|(a, b)| original_content(a, b))
    }
    fn no_stages(&self) -> bool {
        fs::read_dir(self.originals.client.path())
            .unwrap()
            .all(|p| {
                !p.unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".ocxd-restore-")
            })
    }
}

#[test]
fn complete_restores_real_original_bytes_permissions_and_absence_without_discarding_postimages() {
    let f = Fixture::new();
    let stopped = f.originals.read(Phase::Stopped).unwrap();
    let pending = f.prepare();
    assert!(f.matches(Phase::Stopped));
    pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|_| Ok(()))
        .unwrap();
    assert!(f.matches(Phase::Original));
    assert!(f.no_stages());
    for i in 0..3 {
        let archived = capture_file(
            &f.originals.directory,
            &format!("restore-{}-old-{i}", pending.id()),
            MAX_CONTENT,
        )
        .unwrap();
        assert!(same(&archived, &stopped.images[i], false));
    }
    pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|_| Ok(()))
        .unwrap();
    assert!(f.matches(Phase::Original));
    assert!(pending
        .apply_checked(Decision::Rollback, &|| Ok(()), &|_| Ok(()))
        .is_err());
    assert!(f.matches(Phase::Original));
}
#[test]
fn completed_inspection_never_adopts_partial_rollback_or_unknown_postimages() {
    let f = Fixture::new();
    let pending = f.prepare();
    assert!(!pending
        .review_recovery()
        .map(|r| r.terminal == Some(Decision::Complete))
        .unwrap());
    pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|_| Ok(()))
        .unwrap();
    assert!(pending
        .review_recovery()
        .map(|r| r.terminal == Some(Decision::Complete))
        .unwrap());
    let path = f.originals.client.path().join(NAMES[0]);
    let before = fs::read(&path).unwrap();
    fs::write(&path, b"outside edit after publication").unwrap();
    assert!(
        pending
            .review_recovery()
            .map(|r| r.terminal == Some(Decision::Complete))
            .is_err()
            || !pending
                .review_recovery()
                .map(|r| r.terminal == Some(Decision::Complete))
                .unwrap()
    );
    assert_eq!(fs::read(&path).unwrap(), b"outside edit after publication");
    assert!(!before.is_empty());
    let f = Fixture::new();
    let pending = f.prepare();
    pending
        .apply_checked(Decision::Rollback, &|| Ok(()), &|_| Ok(()))
        .unwrap();
    assert!(!pending
        .review_recovery()
        .map(|r| r.terminal == Some(Decision::Complete))
        .unwrap());
}

#[test]
fn recovery_review_binds_the_whole_batch_without_changing_any_object() {
    let f = Fixture::new();
    let pending = f.prepare();
    let before = Capture::read(&f.originals.client).unwrap();
    let review = pending.review_recovery().unwrap();
    assert_eq!(review.terminal, None);
    assert_eq!(
        pending.review_recovery().unwrap().fingerprint,
        review.fingerprint
    );
    assert_eq!(Capture::read(&f.originals.client).unwrap(), before);
    assert!(!f
        .originals
        .directory
        .path()
        .join(format!("restore-{}-intent.json", pending.id()))
        .exists());
    let stopped = f.originals.frame(Phase::Stopped).unwrap().1;
    let ready = f.originals.frame(Phase::Ready).unwrap().1;
    let path = f.originals.client.path().join(NAMES[1]);
    fs::write(&path, b"outside cache edit").unwrap();
    assert!(pending.review_recovery().is_err());
    assert_eq!(fs::read(&path).unwrap(), b"outside cache edit");
    assert_eq!(f.originals.frame(Phase::Stopped).unwrap().1, stopped);
    assert_eq!(f.originals.frame(Phase::Ready).unwrap().1, ready);
}
#[test]
fn missing_plan_proposal_reuses_only_original_four_frames_and_exact_stopped_postimages() {
    let f = Fixture::new();
    let before = Capture::read(&f.originals.client).unwrap();
    let digest = Pending::preparation_fingerprint(&f.originals, &f.evidence).unwrap();
    assert_eq!(
        digest,
        Pending::preparation_fingerprint(&f.originals, &f.evidence).unwrap()
    );
    assert_eq!(Capture::read(&f.originals.client).unwrap(), before);
    let names = fs::read_dir(f.originals.directory.path())
        .unwrap()
        .map(|p| p.unwrap().file_name())
        .collect::<Vec<_>>();
    assert!(!names
        .iter()
        .any(|s| s.to_string_lossy().starts_with("restore-")));
    let cache = f.originals.client.path().join(NAMES[1]);
    let bytes = fs::read(&cache).unwrap();
    fs::write(&cache, b"outside post-stop cache").unwrap();
    assert!(Pending::preparation_fingerprint(&f.originals, &f.evidence).is_err());
    assert_eq!(fs::read(&cache).unwrap(), b"outside post-stop cache");
    fs::write(&cache, bytes).unwrap();
    // Returning bytes does not recreate the exact captured change metadata;
    // an outside write must not become a replacement stopped baseline.
    assert!(Pending::preparation_fingerprint(&f.originals, &f.evidence).is_err());
    let f = Fixture::new();
    let mut unknown = f.evidence.clone();
    unknown.declared_routes = None;
    assert!(Pending::preparation_fingerprint(&f.originals, &unknown).is_err());
    assert!(Pending::preparation_fingerprint(&f.originals, &f.evidence).is_ok());
}

#[test]
fn recovery_review_rejects_missing_intent_damaged_terminal_and_replaced_archive() {
    for mode in ["intent", "outcome", "archive"] {
        let f = Fixture::new();
        let pending = f.prepare();
        let initial = pending.review_recovery().unwrap().fingerprint;
        pending
            .apply_checked(Decision::Complete, &|| Ok(()), &|_| Ok(()))
            .unwrap();
        let complete = pending.review_recovery().unwrap();
        assert_eq!(complete.terminal, Some(Decision::Complete));
        assert_ne!(complete.fingerprint, initial);
        let directory = f.originals.directory.path();
        match mode {
            "intent" => {
                fs::remove_file(directory.join(format!("restore-{}-intent.json", pending.id())))
                    .unwrap()
            }
            "outcome" => {
                let p = directory.join(format!("restore-{}-outcome.json", pending.id()));
                fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
                fs::write(p, b"{}").unwrap();
            }
            "archive" => {
                let p = directory.join(format!("restore-{}-old-0", pending.id()));
                let bytes = fs::read(&p).unwrap();
                let replacement = directory.join("replacement");
                fs::write(&replacement, &bytes).unwrap();
                fs::rename(replacement, p).unwrap();
            }
            _ => unreachable!(),
        }
        let before = Capture::read(&f.originals.client).unwrap();
        assert!(pending.review_recovery().is_err(), "{mode}");
        assert_eq!(Capture::read(&f.originals.client).unwrap(), before);
    }
}

#[test]
fn a_second_object_external_change_refuses_the_whole_batch_before_any_original_publish() {
    let f = Fixture::new();
    let pending = f.prepare();
    let before = fs::read(f.originals.client.path().join(NAMES[0])).unwrap();
    fs::write(
        f.originals.client.path().join(NAMES[1]),
        b"external cache write",
    )
    .unwrap();
    assert!(pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|_| Ok(()))
        .is_err());
    assert_eq!(
        fs::read(f.originals.client.path().join(NAMES[0])).unwrap(),
        before
    );
    assert_eq!(
        fs::read(f.originals.client.path().join(NAMES[1])).unwrap(),
        b"external cache write"
    );
}

#[test]
fn same_bytes_replacement_inode_and_stage_symlink_are_not_owned() {
    for swap in [false, true] {
        let f = Fixture::new();
        let pending = f.prepare();
        let stage = format!(".ocxd-restore-{}-0", pending.id());
        let target = f
            .originals
            .client
            .path()
            .join(if swap { NAMES[1] } else { &stage });
        let data = fs::read(&target).unwrap();
        fs::remove_file(&target).unwrap();
        if swap {
            fs::write(&target, data).unwrap();
        } else {
            symlink(f.originals.client.path().join(NAMES[0]), &target).unwrap();
        }
        assert!(pending
            .apply_checked(Decision::Complete, &|| Ok(()), &|_| Ok(()))
            .is_err());
        assert!(f.originals.client.path().join(NAMES[0]).is_file());
    }
}

#[test]
fn failure_after_native_exchange_can_explicitly_roll_back_to_the_frozen_stopped_images() {
    let f = Fixture::new();
    let pending = f.prepare();
    let once = Cell::new(false);
    assert!(pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|point| {
            if point == "after_native" && !once.replace(true) {
                Err(denied("injected observer failure"))
            } else {
                Ok(())
            }
        })
        .is_err());
    pending
        .apply_checked(Decision::Rollback, &|| Ok(()), &|_| Ok(()))
        .unwrap();
    assert!(f.matches(Phase::Stopped));
    assert!(f.no_stages());
    pending
        .apply_checked(Decision::Rollback, &|| Ok(()), &|_| Ok(()))
        .unwrap();
    assert!(pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|_| Ok(()))
        .is_err());
}

#[test]
fn failure_during_archive_retains_known_objects_and_can_continue_the_same_plan() {
    let f = Fixture::new();
    let pending = f.prepare();
    let once = Cell::new(false);
    assert!(pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|point| {
            if point == "archive" && !once.replace(true) {
                Err(denied("injected archive observation failure"))
            } else {
                Ok(())
            }
        })
        .is_err());
    pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|_| Ok(()))
        .unwrap();
    assert!(f.matches(Phase::Original));
    assert!(f.no_stages());
}

#[test]
fn partial_archive_can_roll_back_without_learning_new_file_identities() {
    let f = Fixture::new();
    let pending = f.prepare();
    let once = Cell::new(false);
    assert!(pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|point| {
            if point == "archive" && !once.replace(true) {
                Err(denied("injected archive observation failure"))
            } else {
                Ok(())
            }
        })
        .is_err());
    pending
        .apply_checked(Decision::Rollback, &|| Ok(()), &|_| Ok(()))
        .unwrap();
    assert!(f.matches(Phase::Stopped));
    assert!(f.no_stages());
}

#[test]
fn rearmed_logical_plan_keeps_exact_native_objects_and_immutable_rollback_history() {
    let f = Fixture::new();
    let parent = f.prepare();
    assert!(parent
        .apply_fixture(Decision::Complete, &|p| if p == "after_native" {
            Err(denied("interrupt"))
        } else {
            Ok(())
        })
        .is_err());
    parent
        .apply_fixture(Decision::Rollback, &|_| Ok(()))
        .unwrap();
    let terminal = f
        .originals
        .directory
        .path()
        .join(format!("restore-{}-outcome.json", parent.id()));
    let original_terminal = fs::read(&terminal).unwrap();
    let before = Capture::read(&f.originals.client).unwrap();
    let orphan = parent.rearm_rolled_back().unwrap();
    let next = parent.rearm_rolled_back().unwrap();
    assert_ne!(next.id(), parent.id());
    assert_ne!(next.id(), orphan.id());
    assert_eq!(next.object_id(), parent.id());
    assert_eq!(Capture::read(&f.originals.client).unwrap(), before);
    assert!(next.review_recovery().unwrap().terminal.is_none());
    next.require_parent(&session_index::RestorePlan {
        id: parent.id().into(),
        sha256: parent.sha256().into(),
    })
    .unwrap();
    assert!(next
        .require_parent(&session_index::RestorePlan {
            id: orphan.id().into(),
            sha256: orphan.sha256().into()
        })
        .is_err());
    next.apply_fixture(Decision::Complete, &|_| Ok(())).unwrap();
    assert!(f.matches(Phase::Original));
    assert_eq!(fs::read(&terminal).unwrap(), original_terminal);
    assert!(next.rearm_rolled_back().is_err());
    assert!(
        orphan.review_recovery().is_err(),
        "unindexed plan is never adopted after objects change"
    );
}

#[test]
fn expiration_or_native_authority_loss_never_creates_a_terminal_success() {
    let f = Fixture::new();
    let pending = f.prepare();
    let revoked = Cell::new(false);
    assert!(pending
        .apply_checked(
            Decision::Complete,
            &|| if revoked.get() {
                Err(denied("coordinator gone"))
            } else {
                Ok(())
            },
            &|p| {
                if p == "after_native" {
                    revoked.set(true);
                }
                Ok(())
            }
        )
        .is_err());
    assert!(!f
        .originals
        .directory
        .path()
        .join(format!("restore-{}-outcome.json", pending.id()))
        .exists());
    pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|_| Ok(()))
        .unwrap();
    assert!(f.matches(Phase::Original));
}

#[test]
fn external_edit_between_preflight_and_native_step_is_preserved() {
    let f = Fixture::new();
    let pending = f.prepare();
    let changed = Cell::new(false);
    assert!(pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|point| {
            if point == "before_native" && !changed.replace(true) {
                fs::write(
                    f.originals.client.path().join(NAMES[0]),
                    b"user replacement",
                )?;
            }
            Ok(())
        })
        .is_err());
    assert_eq!(
        fs::read(f.originals.client.path().join(NAMES[0])).unwrap(),
        b"user replacement"
    );
}

#[test]
fn independent_plan_digest_and_all_phase_digests_are_required_on_reopen() {
    let f = Fixture::new();
    let pending = f.prepare();
    assert!(Pending::open(&f.originals, pending.id(), &"a".repeat(64)).is_err());
    let reopened = Pending::open(&f.originals, pending.id(), pending.sha256()).unwrap();
    reopened
        .apply_checked(Decision::Complete, &|| Ok(()), &|_| Ok(()))
        .unwrap();
    assert!(f.matches(Phase::Original));
}

#[test]
fn actual_acl_and_opaque_xattrs_are_restored_with_original_content() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let f = Fixture::at_with(root, |path| {
        let file = File::open(path).unwrap();
        let key = CString::new("user.ocxd024-restore").unwrap();
        let value = b"opaque original attribute";
        assert_eq!(
            unsafe {
                libc::fsetxattr(
                    file.as_raw_fd(),
                    key.as_ptr(),
                    value.as_ptr().cast(),
                    value.len(),
                    0,
                    0,
                )
            },
            0
        );
        let user = unsafe { libc::getpwuid(libc::geteuid()) };
        assert!(!user.is_null());
        let username = unsafe { std::ffi::CStr::from_ptr((*user).pw_name) }
            .to_str()
            .unwrap();
        let output = std::process::Command::new("/bin/chmod")
            .args(["+a", &format!("user:{username} allow read")])
            .arg(path)
            .output()
            .unwrap();
        assert!(output.status.success());
    });
    let original = f.originals.read(Phase::Original).unwrap();
    let Image::File { protection, .. } = &original.images[0] else {
        panic!("file missing")
    };
    assert!(protection.acl.is_some());
    assert!(protection.xattrs.iter().any(
        |(key, value)| key == b"user.ocxd024-restore" && value == b"opaque original attribute"
    ));
    let pending =
        Pending::prepare_layout(&f.originals, f.evidence.clone(), true, &|| Ok(()), &|_| {
            Ok(())
        })
        .unwrap();
    pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|_| Ok(()))
        .unwrap();
    assert!(f.matches(Phase::Original));
    assert!(f.no_stages());
}

#[test]
fn damaged_or_third_party_archived_postimage_blocks_continuation() {
    let f = Fixture::new();
    let pending = f.prepare();
    let once = Cell::new(false);
    assert!(pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|p| {
            if p == "archive" && !once.replace(true) {
                Err(denied("injected"))
            } else {
                Ok(())
            }
        })
        .is_err());
    let archive = f
        .originals
        .directory
        .path()
        .join(format!("restore-{}-old-0", pending.id()));
    fs::write(&archive, b"unrecognized archive edit").unwrap();
    assert!(pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|_| Ok(()))
        .is_err());
    assert!(pending
        .apply_checked(Decision::Rollback, &|| Ok(()), &|_| Ok(()))
        .is_err());
    assert_eq!(fs::read(&archive).unwrap(), b"unrecognized archive edit");
}

#[test]
fn unknown_preexisting_archive_destination_refuses_before_any_target_mutation() {
    let f = Fixture::new();
    let pending = f.prepare();
    let archive = f
        .originals
        .directory
        .path()
        .join(format!("restore-{}-old-2", pending.id()));
    fs::write(&archive, b"third-party object").unwrap();
    assert!(pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|_| Ok(()))
        .is_err());
    assert!(f.matches(Phase::Stopped));
    assert_eq!(fs::read(&archive).unwrap(), b"third-party object");
}

#[test]
fn late_catalog_replacement_after_final_check_is_preserved_at_original_path_and_refused() {
    let f = Fixture::new();
    let pending = f.prepare();
    let once = Cell::new(false);
    let target = f.originals.client.path().join(NAMES[0]);
    assert!(pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|p| {
            if p == "native_gap" && !once.replace(true) {
                let saved = target.with_extension("pre-race");
                fs::rename(&target, &saved)?;
                fs::write(&target, b"late user catalog")?;
            }
            Ok(())
        })
        .is_err());
    assert_eq!(fs::read(&target).unwrap(), b"late user catalog");
    assert!(!f
        .originals
        .directory
        .path()
        .join(format!("restore-{}-outcome.json", pending.id()))
        .exists());
}

#[test]
fn late_cache_edit_in_remove_gap_is_returned_without_overwriting_a_new_destination() {
    let f = Fixture::new();
    let pending = f.prepare();
    let number = Cell::new(0);
    let target = f.originals.client.path().join(NAMES[1]);
    assert!(pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|p| {
            if p == "native_gap" {
                number.set(number.get() + 1);
                if number.get() == 2 {
                    fs::write(&target, b"late user cache")?;
                }
            }
            Ok(())
        })
        .is_err());
    assert_eq!(fs::read(&target).unwrap(), b"late user cache");
}

#[test]
fn late_archive_source_replacement_is_returned_to_stage_and_never_committed_as_original() {
    let f = Fixture::new();
    let pending = f.prepare();
    let once = Cell::new(false);
    let stage = f
        .originals
        .client
        .path()
        .join(format!(".ocxd-restore-{}-0", pending.id()));
    assert!(pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|p| {
            if p == "archive_gap" && !once.replace(true) {
                let saved = stage.with_extension("pre-race");
                fs::rename(&stage, &saved)?;
                fs::write(&stage, b"late user archive-stage")?;
            }
            Ok(())
        })
        .is_err());
    assert_eq!(fs::read(&stage).unwrap(), b"late user archive-stage");
}

#[test]
fn a_second_writer_in_compensation_gap_keeps_both_payloads_and_no_terminal_result() {
    let f = Fixture::new();
    let pending = f.prepare();
    let number = Cell::new(0);
    let target = f.originals.client.path().join(NAMES[1]);
    assert!(pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|p| {
            if p == "native_gap" {
                number.set(number.get() + 1);
                if number.get() == 2 {
                    fs::write(&target, b"first late cache")?;
                }
            }
            if p == "compensation_gap" {
                fs::write(&target, b"second independent cache")?;
            }
            Ok(())
        })
        .is_err());
    assert_eq!(fs::read(&target).unwrap(), b"second independent cache");
    let staged = f
        .originals
        .client
        .path()
        .join(format!(".ocxd-restore-{}-1", pending.id()));
    assert_eq!(fs::read(staged).unwrap(), b"first late cache");
    assert!(!f
        .originals
        .directory
        .path()
        .join(format!("restore-{}-outcome.json", pending.id()))
        .exists());
}

#[test]
fn actual_external_process_replacement_in_native_gap_is_returned_and_conflict_is_durable() {
    const CHILD: &str = "OCXD_RESTORE_LATE_WRITER";
    if let Some(path) = std::env::var_os(CHILD) {
        let path = PathBuf::from(path);
        assert!(path.is_absolute() && path.starts_with("/private/tmp"));
        let original = path.with_extension("external-displaced");
        fs::rename(&path, original).unwrap();
        fs::write(path, b"other OS process catalog").unwrap();
        std::process::exit(0);
    }
    let f = Fixture::new();
    let pending = f.prepare();
    let target = f.originals.client.path().join(NAMES[0]);
    let once = Cell::new(false);
    assert!(pending.apply_checked(Decision::Complete,&||Ok(()),&|p|{
        if p=="native_gap"&&!once.replace(true){let output=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::restore::tests::actual_external_process_replacement_in_native_gap_is_returned_and_conflict_is_durable"]).env(CHILD,&target).output()?;assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));}Ok(())
    }).is_err());
    assert_eq!(fs::read(&target).unwrap(), b"other OS process catalog");
    let entries = fs::read_dir(f.originals.directory.path())
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .contains("-conflict-")
        })
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 1);
    let bytes = fs::read(&entries[0]).unwrap();
    assert_eq!(
        fs::metadata(&entries[0]).unwrap().permissions().mode() & 0o777,
        0o400
    );
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["planSha256"], pending.sha256());
    assert_eq!(value["artifact"], 0);
    let stage = &value["stage"]["bytes"];
    let actual = stage
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b.as_u64().unwrap() as u8)
        .collect::<Vec<_>>();
    assert_eq!(actual, b"other OS process catalog");
    println!(
        "SYNTHETIC_RACE_CONFLICT_RECEIPT {}",
        serde_json::to_string(&value).unwrap()
    );
    assert!(pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|_| Ok(()))
        .is_err());
}

#[test]
fn late_stage_replacement_does_not_reverse_over_an_unknown_published_object() {
    let f = Fixture::new();
    let pending = f.prepare();
    let target = f.originals.client.path().join(NAMES[0]);
    let stage = f
        .originals
        .client
        .path()
        .join(format!(".ocxd-restore-{}-0", pending.id()));
    let once = Cell::new(false);
    assert!(pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|p| {
            if p == "native_gap" && !once.replace(true) {
                let saved = stage.with_extension("owned-original");
                fs::rename(&stage, saved)?;
                fs::write(&stage, b"outside stage payload")?;
            }
            Ok(())
        })
        .is_err());
    assert_eq!(fs::read(&target).unwrap(), b"outside stage payload");
    assert!(stage.exists());
    assert!(!f
        .originals
        .directory
        .path()
        .join(format!("restore-{}-outcome.json", pending.id()))
        .exists());
}

#[test]
fn abrupt_exit_after_conflict_receipt_preserves_unknown_bytes_and_cannot_resume_as_success() {
    const CHILD: &str = "OCXD_RESTORE_CONFLICT_CRASH";
    if let Some(root) = std::env::var_os(CHILD) {
        let root = PathBuf::from(root);
        let f = Fixture::at(root.clone());
        let pending = f.prepare();
        let pending = if std::env::var_os("OCXD024_RESTORE_REARM").is_some() {
            assert!(pending
                .apply_fixture(Decision::Complete, &|p| if p == "after_native" {
                    Err(denied("first interruption"))
                } else {
                    Ok(())
                })
                .is_err());
            pending
                .apply_fixture(Decision::Rollback, &|_| Ok(()))
                .unwrap();
            pending.rearm_rolled_back().unwrap()
        } else {
            pending
        };
        let phases = f.originals.known.lock().unwrap().clone();
        let note = json!({"attempt":f.originals.attempt(),"original":f.originals.original_sha256(),"phases":phases,"plan":pending.id(),"planSha256":pending.sha256()});
        let mut receipt = File::create(root.join("independent-receipt.json")).unwrap();
        receipt.write_all(&wire(note)).unwrap();
        receipt.sync_all().unwrap();
        let target = f.originals.client.path().join(NAMES[0]);
        let once = Cell::new(false);
        let _ = pending.apply_checked(Decision::Complete, &|| Ok(()), &|point| {
            if point == "native_gap" && !once.replace(true) {
                fs::write(&target, b"unknown before abrupt conflict exit")?;
            }
            if point == "compensation_gap" {
                std::process::exit(91);
            }
            Ok(())
        });
        panic!("conflict exit boundary not reached");
    }
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let output=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::restore::tests::abrupt_exit_after_conflict_receipt_preserves_unknown_bytes_and_cannot_resume_as_success","--nocapture"]).env(CHILD,&root).output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(91),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let n: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("independent-receipt.json")).unwrap()).unwrap();
    let phases = [Phase::Ready, Phase::PreStop, Phase::Stopped]
        .into_iter()
        .enumerate()
        .map(|(i, p)| (p, n["phases"][i + 1].as_str().unwrap().to_owned()))
        .collect::<Vec<_>>();
    let originals = Originals::open(
        &root.join("desktop"),
        &root.join("codex 中文 space"),
        n["attempt"].as_str().unwrap(),
        n["original"].as_str().unwrap(),
        &phases,
    )
    .unwrap();
    let pending = Pending::open(
        &originals,
        n["plan"].as_str().unwrap(),
        n["planSha256"].as_str().unwrap(),
    )
    .unwrap();
    assert!(pending
        .apply_checked(Decision::Complete, &|| Ok(()), &|_| Ok(()))
        .is_err());
    assert!(pending
        .apply_checked(Decision::Rollback, &|| Ok(()), &|_| Ok(()))
        .is_err());
    let stage = originals
        .client
        .path()
        .join(format!(".ocxd-restore-{}-0", pending.id()));
    assert_eq!(
        fs::read(&stage).unwrap(),
        b"unknown before abrupt conflict exit"
    );
    let conflicts = fs::read_dir(originals.directory.path())
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .contains("-conflict-")
        })
        .collect::<Vec<_>>();
    assert_eq!(conflicts.len(), 1);
    let raw = fs::read(&conflicts[0]).unwrap();
    let conflict: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(conflict["planSha256"], pending.sha256());
    println!(
        "SYNTHETIC_CRASH_CONFLICT_RECEIPT {}",
        serde_json::to_string(&conflict).unwrap()
    );
    assert!(!originals
        .directory
        .path()
        .join(format!("restore-{}-outcome.json", pending.id()))
        .exists());
}

#[test]
fn actual_subprocess_exit_at_every_publication_boundary_preserves_resumable_known_generations() {
    const CHILD: &str = "OCXD024_RESTORE_CRASH";
    if let Some(root) = std::env::var_os(CHILD) {
        let root = PathBuf::from(root);
        let f = Fixture::at(root.clone());
        let pending = if std::env::var_os("OCXD024_JOURNAL_LAYOUT").is_some() {
            Pending::prepare_layout(&f.originals, f.evidence.clone(), true, &|| Ok(()), &|_| {
                Ok(())
            })
            .unwrap()
        } else {
            f.prepare()
        };
        let pending = if std::env::var_os("OCXD024_RESTORE_REARM").is_some() {
            assert!(pending
                .apply_fixture(Decision::Complete, &|p| if p == "after_native" {
                    Err(denied("injected interrupted publication"))
                } else {
                    Ok(())
                })
                .is_err());
            pending
                .apply_fixture(Decision::Rollback, &|_| Ok(()))
                .unwrap();
            pending.rearm_rolled_back().unwrap()
        } else {
            pending
        };
        let point = std::env::var("OCXD024_RESTORE_POINT").unwrap();
        let nth: usize = std::env::var("OCXD024_RESTORE_NTH")
            .unwrap()
            .parse()
            .unwrap();
        let phases = f.originals.known.lock().unwrap().clone();
        let note = json!({"attempt":f.originals.attempt(),"original":f.originals.original_sha256(),"phases":phases,"plan":pending.id(),"planSha256":pending.sha256()});
        let mut file = File::create(root.join("independent-receipt.json")).unwrap();
        file.write_all(&wire(note)).unwrap();
        file.sync_all().unwrap();
        let seen = Cell::new(0usize);
        let _ = pending.apply_checked(Decision::Complete, &|| Ok(()), &|p| {
            if p == point {
                seen.set(seen.get() + 1);
                if seen.get() == nth {
                    std::process::exit(91);
                }
            }
            Ok(())
        });
        panic!("crash boundary was not reached");
    }
    for (point, nth) in [
        ("intent", 1),
        ("after_native", 1),
        ("after_native", 2),
        ("after_native", 3),
        ("step", 1),
        ("step", 2),
        ("step", 3),
        ("archive", 1),
        ("archive", 2),
        ("archive", 3),
        ("terminal", 1),
    ] {
        for rearmed in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().canonicalize().unwrap();
            let mut command = std::process::Command::new(std::env::current_exe().unwrap());
            command.args(["--exact","infrastructure::macos_client_originals::restore::tests::actual_subprocess_exit_at_every_publication_boundary_preserves_resumable_known_generations","--nocapture"]).env(CHILD,&root).env("OCXD024_RESTORE_POINT",point).env("OCXD024_RESTORE_NTH",nth.to_string());
            if rearmed {
                command.env("OCXD024_RESTORE_REARM", "1");
            }
            let child = command.output().unwrap();
            assert_eq!(
                child.status.code(),
                Some(91),
                "{point}/{nth}/rearmed={rearmed}: {}",
                String::from_utf8_lossy(&child.stderr)
            );
            let n: serde_json::Value =
                serde_json::from_slice(&fs::read(root.join("independent-receipt.json")).unwrap())
                    .unwrap();
            let phases = [Phase::Ready, Phase::PreStop, Phase::Stopped]
                .into_iter()
                .enumerate()
                .map(|(i, p)| (p, n["phases"][i + 1].as_str().unwrap().to_owned()))
                .collect::<Vec<_>>();
            let originals = Originals::open(
                &root.join("desktop"),
                &root.join("codex 中文 space"),
                n["attempt"].as_str().unwrap(),
                n["original"].as_str().unwrap(),
                &phases,
            )
            .unwrap();
            let pending = Pending::open(
                &originals,
                n["plan"].as_str().unwrap(),
                n["planSha256"].as_str().unwrap(),
            )
            .unwrap();
            let review = pending.review_recovery().unwrap();
            assert_eq!(
                review.terminal,
                (point == "terminal").then_some(Decision::Complete)
            );
            assert_eq!(
                pending.review_recovery().unwrap().fingerprint,
                review.fingerprint
            );
            pending
                .apply_checked(Decision::Complete, &|| Ok(()), &|_| Ok(()))
                .unwrap();
            let expected = originals.read(Phase::Original).unwrap();
            let actual = Capture::read(&originals.client).unwrap();
            assert!(
                actual
                    .images
                    .iter()
                    .zip(&expected.images)
                    .all(|(a, b)| original_content(a, b)),
                "{point}/{nth}"
            );
        }
    }
}

#[test]
fn interrupted_journal_preparation_never_changes_clients_and_retry_preserves_unindexed_files() {
    const CHILD: &str = "OCXD024_JOURNAL_PREP_CHILD";
    if let Some(root) = std::env::var_os(CHILD) {
        let root = PathBuf::from(root);
        let f = Fixture::at(root.clone());
        let note = json!({"attempt":f.originals.attempt(),"original":f.originals.original_sha256(),"phases":f.originals.known.lock().unwrap().clone(),"evidence":f.evidence});
        let mut receipt = File::create(root.join("independent-receipt.json")).unwrap();
        receipt.write_all(&wire(note)).unwrap();
        receipt.sync_all().unwrap();
        let point = std::env::var("OCXD024_JOURNAL_PREP_POINT").unwrap();
        let _ = Pending::prepare_layout(&f.originals, f.evidence.clone(), true, &|| Ok(()), &|p| {
            if p == point {
                std::process::exit(91);
            }
            Ok(())
        });
        panic!("preparation interruption boundary not reached");
    }
    for point in [
        "preparing",
        "stage_created",
        "stage_bytes",
        "stage_protected",
        "stage_synced",
        "staged",
        "plan_published",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let child=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::restore::tests::interrupted_journal_preparation_never_changes_clients_and_retry_preserves_unindexed_files","--nocapture"]).env(CHILD,&root).env("OCXD024_JOURNAL_PREP_POINT",point).output().unwrap();
        assert_eq!(
            child.status.code(),
            Some(91),
            "{point}: {}",
            String::from_utf8_lossy(&child.stderr)
        );
        let note: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("independent-receipt.json")).unwrap())
                .unwrap();
        let phases = [Phase::Ready, Phase::PreStop, Phase::Stopped]
            .into_iter()
            .enumerate()
            .map(|(i, p)| (p, note["phases"][i + 1].as_str().unwrap().to_string()))
            .collect::<Vec<_>>();
        let originals = Originals::open(
            &root.join("desktop"),
            &root.join("codex 中文 space"),
            note["attempt"].as_str().unwrap(),
            note["original"].as_str().unwrap(),
            &phases,
        )
        .unwrap();
        let before = Capture::read(&originals.client).unwrap();
        assert_eq!(before, originals.read(Phase::Stopped).unwrap());
        assert!(fs::read_dir(originals.client.path()).unwrap().all(|p| !p
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".ocxd-restore-")));
        let retained = fs::read_dir(originals.directory.path())
            .unwrap()
            .map(|p| {
                let n = p.unwrap().file_name().to_string_lossy().into_owned();
                let image = capture_file(&originals.directory, &n, MAX_FRAME).unwrap();
                (n, image)
            })
            .collect::<Vec<_>>();
        let evidence: Evidence = serde_json::from_value(note["evidence"].clone()).unwrap();
        let next =
            Pending::prepare_layout(&originals, evidence, true, &|| Ok(()), &|_| Ok(())).unwrap();
        assert!(next.stages_in_journal());
        assert_eq!(Capture::read(&originals.client).unwrap(), before);
        for (n, image) in &retained {
            assert_eq!(
                capture_file(&originals.directory, n, MAX_FRAME).unwrap(),
                *image,
                "unindexed evidence changed at {point}: {n}"
            );
        }
        next.apply_fixture(Decision::Complete, &|_| Ok(())).unwrap();
        let actual = Capture::read(&originals.client).unwrap();
        let original = originals.read(Phase::Original).unwrap();
        assert!(actual
            .images
            .iter()
            .zip(&original.images)
            .all(|(a, b)| original_content(a, b)));
        for (n, image) in &retained {
            assert_eq!(
                capture_file(&originals.directory, n, MAX_FRAME).unwrap(),
                *image,
                "orphan stage/plan adopted or overwritten"
            );
        }
    }
}
#[test]
fn journal_staging_requires_exact_plan_bound_objects_and_preserves_external_edits() {
    for unknown in ["stage", "target"] {
        let f = Fixture::new();
        let pending =
            Pending::prepare_layout(&f.originals, f.evidence.clone(), true, &|| Ok(()), &|_| {
                Ok(())
            })
            .unwrap();
        let path = if unknown == "stage" {
            f.originals
                .directory
                .path()
                .join(format!(".ocxd-restore-{}-0", pending.id()))
        } else {
            f.originals.client.path().join(NAMES[1])
        };
        let before = fs::read(&path).unwrap();
        let replacement = path.with_extension("replacement");
        fs::write(&replacement, &before).unwrap();
        fs::rename(replacement, &path).unwrap();
        let current = Capture::read(&f.originals.client).unwrap();
        assert!(pending.review_recovery().is_err());
        assert!(pending
            .apply_fixture(Decision::Complete, &|_| Ok(()))
            .is_err());
        assert_eq!(Capture::read(&f.originals.client).unwrap(), current);
        assert_eq!(fs::read(&path).unwrap(), before);
    }
}
#[test]
fn journal_native_gap_returns_late_external_object_and_never_claims_success() {
    let f = Fixture::new();
    let pending =
        Pending::prepare_layout(&f.originals, f.evidence.clone(), true, &|| Ok(()), &|_| {
            Ok(())
        })
        .unwrap();
    let once = Cell::new(false);
    let error = pending.apply_fixture(Decision::Complete, &|point| {
        if point == "native_gap" && !once.replace(true) {
            let path = f.originals.client.path().join(NAMES[0]);
            let replacement = path.with_extension("outside");
            fs::write(&replacement, b"late external catalog")?;
            fs::rename(replacement, path)?;
        }
        Ok(())
    });
    assert!(error.is_err());
    assert_eq!(
        fs::read(f.originals.client.path().join(NAMES[0])).unwrap(),
        b"late external catalog"
    );
    assert!(pending.review_recovery().is_err());
    assert!(pending
        .apply_fixture(Decision::Rollback, &|_| Ok(()))
        .is_err());
}
#[test]
fn journal_staging_survives_every_registered_publication_boundary_and_rearm() {
    let output=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::restore::tests::actual_subprocess_exit_at_every_publication_boundary_preserves_resumable_known_generations","--nocapture","--test-threads=1"]).env("OCXD024_JOURNAL_LAYOUT","1").output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn new_stage_readback_cannot_adopt_a_same_bytes_replacement_before_plan_binding() {
    let f = Fixture::new();
    let original = f.originals.read(Phase::Original).unwrap();
    let stage_name = "synthetic-created-stage";
    let path = f.originals.directory.path().join(stage_name);
    let result = stage(
        &f.originals.directory,
        stage_name,
        &original.images[0],
        &|point| {
            if point == "stage_synced" {
                let replacement = path.with_extension("other");
                let Image::File {
                    protection, bytes, ..
                } = &original.images[0]
                else {
                    panic!("original absent");
                };
                let mut file = File::create(&replacement)?;
                file.write_all(bytes)?;
                metadata(&file, protection)?;
                file.sync_all()?;
                fs::rename(replacement, &path)?;
            }
            Ok(())
        },
    );
    assert!(result.is_err());
    let Image::File { bytes, .. } = &original.images[0] else {
        panic!("original absent");
    };
    assert_eq!(fs::read(path).unwrap(), *bytes);
    assert!(f.matches(Phase::Stopped));
}

#[test]
fn current_recovery_observation_cannot_manufacture_or_adopt_a_missing_historical_stopped_frame() {
    for prefix_count in [1, 2] {
        let f = Fixture::new();
        let references = [Phase::Ready, Phase::PreStop]
            .into_iter()
            .take(prefix_count)
            .map(|phase| {
                (
                    phase,
                    digest(&fs::read(f.originals.directory.path().join(phase.filename())).unwrap()),
                )
            })
            .collect::<Vec<_>>();
        let originals = Originals::open(
            &f.originals.data.path,
            &f.originals.client.path,
            f.originals.attempt(),
            f.originals.original_sha256(),
            &references,
        )
        .unwrap();
        let refs = session::References {
            attempt: originals.attempt().into(),
            original_sha256: originals.original_sha256().into(),
            scope_sha256: "b".repeat(64),
            phases: references.clone(),
            live_sha256: Some("c".repeat(64)),
        };
        let source = session_index::Head {
            revision: 5,
            ticket: uuid::Uuid::new_v4().to_string(),
            state: if prefix_count == 1 {
                session_index::State::Ready
            } else {
                session_index::State::StopPrepared
            },
            references: Some(refs.clone()),
            restore_plan: None,
            completion_sha256: None,
            root_retirement: None,
            root_generation: None,
        };
        let path = originals.directory.path().join("stopped.json");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, b"unindexed historical bytes must never be adopted").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        let pre = originals.directory.path().join("pre-stop.json");
        if prefix_count == 1 {
            fs::set_permissions(&pre, fs::Permissions::from_mode(0o600)).unwrap();
            fs::write(&pre, b"unindexed pre-stop must never be adopted").unwrap();
            fs::set_permissions(&pre, fs::Permissions::from_mode(0o400)).unwrap();
        }
        let fingerprint =
            Pending::observed_preparation_fingerprint(&originals, &source, &f.evidence).unwrap();
        assert_eq!(
            fingerprint,
            Pending::observed_preparation_fingerprint(&originals, &source, &f.evidence).unwrap()
        );
        let current = Capture::read(&originals.client).unwrap();
        let pending =
            Pending::prepare_observed_fixture(&originals, &source, f.evidence.clone()).unwrap();
        pending.require_recovery_source(&source).unwrap();
        pending.require_indexed_references(&refs).unwrap();
        let mut wrong = source.clone();
        wrong.revision += 1;
        assert!(pending.require_recovery_source(&wrong).is_err());
        let mut wrong = refs.clone();
        wrong.phases.push((Phase::Stopped, "d".repeat(64)));
        assert!(pending.require_indexed_references(&wrong).is_err());
        assert_eq!(Capture::read(&originals.client).unwrap(), current);
        assert!(originals.frame(Phase::Stopped).is_err());
        pending
            .apply_fixture(Decision::Complete, &|_| Ok(()))
            .unwrap();
        let final_capture = Capture::read(&originals.client).unwrap();
        let original = originals.read(Phase::Original).unwrap();
        assert!(final_capture
            .images
            .iter()
            .zip(original.images.iter())
            .all(|(a, b)| original_content(a, b)));
        assert_eq!(
            fs::read(&path).unwrap(),
            b"unindexed historical bytes must never be adopted"
        );
        assert!(originals.frame(Phase::Stopped).is_err());
        let raw = fs::read(
            originals
                .directory
                .path()
                .join(format!("restore-{}.json", pending.id())),
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(value["schemaVersion"], 4);
        assert_eq!(
            value["recoveryObservation"]["sourceHead"]["state"],
            if prefix_count == 1 {
                "ready"
            } else {
                "stop_prepared"
            }
        );
        assert_eq!(
            value["recoveryObservation"]["sourceHead"]["references"]["phases"]
                .as_array()
                .unwrap()
                .len(),
            prefix_count
        );
        assert_eq!(value["phases"].as_array().unwrap().len(), prefix_count + 2);
        if prefix_count == 1 {
            assert!(originals.frame(Phase::PreStop).is_err());
            assert_eq!(
                fs::read(&pre).unwrap(),
                b"unindexed pre-stop must never be adopted"
            );
        }
    }
}

#[test]
fn prepared_current_observation_must_match_the_reviewed_content_not_a_later_valid_subset() {
    for prefix_count in [1, 2] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let f = Fixture::at_with(root, |path| {
            for name in [
                ".opencodex-native-main.claim.sqlite",
                ".opencodex-native-main.owner.sqlite",
            ] {
                let p = path.parent().unwrap().join(name);
                fs::write(&p, b"").unwrap();
                fs::set_permissions(p, fs::Permissions::from_mode(0o600)).unwrap();
            }
        });
        let phases = [Phase::Ready, Phase::PreStop]
            .into_iter()
            .take(prefix_count)
            .map(|p| {
                (
                    p,
                    digest(&fs::read(f.originals.directory.path().join(p.filename())).unwrap()),
                )
            })
            .collect::<Vec<_>>();
        let originals = Originals::open(
            &f.originals.data.path,
            &f.originals.client.path,
            f.originals.attempt(),
            f.originals.original_sha256(),
            &phases,
        )
        .unwrap();
        let head = session_index::Head {
            revision: 5,
            ticket: uuid::Uuid::new_v4().to_string(),
            state: if prefix_count == 1 {
                session_index::State::Ready
            } else {
                session_index::State::StopPrepared
            },
            references: Some(session::References {
                attempt: originals.attempt().into(),
                original_sha256: originals.original_sha256().into(),
                scope_sha256: "b".repeat(64),
                phases,
                live_sha256: Some("c".repeat(64)),
            }),
            restore_plan: None,
            completion_sha256: None,
            root_retirement: None,
            root_generation: None,
        };
        let decision =
            Pending::observed_preparation_fingerprint(&originals, &head, &f.evidence).unwrap();
        let confirmed = digest(
            &encode(&(
                &head,
                decision,
                originals.data.identity(),
                originals.client.identity(),
                originals.profile.identity(),
            ))
            .unwrap(),
        );
        let first =
            Pending::prepare_observed_fixture(&originals, &head, f.evidence.clone()).unwrap();
        assert_eq!(
            first.prepared_observation_confirmation(&head).unwrap(),
            confirmed
        );
        let original = originals.read(Phase::Original).unwrap();
        let Image::File { bytes, .. } = &original.images[0] else {
            panic!("original missing");
        };
        fs::write(originals.client.path().join(NAMES[0]), bytes).unwrap();
        let second =
            Pending::prepare_observed_fixture(&originals, &head, f.evidence.clone()).unwrap();
        assert_ne!(
            second.prepared_observation_confirmation(&head).unwrap(),
            confirmed,
            "a later valid content projection must still require a NEW confirmation"
        );
        assert_eq!(
            fs::read(originals.client.path().join(NAMES[0])).unwrap(),
            *bytes
        );
    }
}

fn first_use_config_at(root: PathBuf, prefix_count: usize) -> Fixture {
    first_use_config_initial(root, prefix_count, false, true)
}
fn first_use_config_initial(
    root: PathBuf,
    prefix_count: usize,
    originally_present: bool,
    ready_present: bool,
) -> Fixture {
    let data = root.join("desktop");
    let client = root.join("codex 中文 space");
    for p in [&data, &client] {
        fs::DirBuilder::new().mode(0o700).create(p).unwrap();
    }
    for name in [
        ".opencodex-native-main.claim.sqlite",
        ".opencodex-native-main.owner.sqlite",
    ] {
        fs::write(client.join(name), b"").unwrap();
        fs::set_permissions(client.join(name), fs::Permissions::from_mode(0o600)).unwrap();
    }
    if originally_present {
        fs::write(client.join("config.toml"), b"\n").unwrap();
        fs::set_permissions(
            client.join("config.toml"),
            fs::Permissions::from_mode(0o600),
        )
        .unwrap();
    }
    let originals = Originals::begin(&data, &client).unwrap();
    if ready_present {
        fs::write(
            client.join("config.toml"),
            b"model_provider = 'opencodex'\n",
        )
        .unwrap();
        fs::set_permissions(
            client.join("config.toml"),
            fs::Permissions::from_mode(0o600),
        )
        .unwrap();
    }
    originals.seal(Phase::Ready).unwrap();
    if prefix_count >= 2 {
        originals.seal(Phase::PreStop).unwrap();
    }
    fs::write(client.join("config.toml"), b"\n").unwrap();
    fs::set_permissions(
        client.join("config.toml"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    if prefix_count == 3 {
        originals.seal(Phase::Stopped).unwrap();
    }
    Fixture {
        _temp: None,
        originals,
        evidence: Evidence {
            declared_routes: Some(vec!["channel/model".into()]),
            retained_pids: vec![100, 101],
            audit_home: "synthetic-captured-home".into(),
            not_before: chrono::DateTime::parse_from_rfc3339("2026-11-11T10:00:00Z")
                .unwrap()
                .into(),
            not_after: chrono::DateTime::parse_from_rfc3339("2026-11-11T10:00:02Z")
                .unwrap()
                .into(),
        },
    }
}
fn first_use_config(prefix_count: usize) -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let mut f = first_use_config_at(temp.path().canonicalize().unwrap(), prefix_count);
    f._temp = Some(temp);
    f
}
fn first_use_source(f: &Fixture, prefix_count: usize) -> session_index::Head {
    let phases = [Phase::Ready, Phase::PreStop, Phase::Stopped]
        .into_iter()
        .take(prefix_count)
        .map(|p| (p, digest(&f.originals.frame(p).unwrap().1)))
        .collect();
    session_index::Head {
        revision: 4 + prefix_count as u64,
        ticket: uuid::Uuid::new_v4().to_string(),
        state: if prefix_count == 1 {
            session_index::State::Ready
        } else if prefix_count == 2 {
            session_index::State::StopPrepared
        } else {
            session_index::State::Stopped
        },
        references: Some(session::References {
            attempt: f.originals.attempt().into(),
            original_sha256: f.originals.original_sha256().into(),
            scope_sha256: "b".repeat(64),
            phases,
            live_sha256: Some("c".repeat(64)),
        }),
        restore_plan: None,
        completion_sha256: None,
        root_retirement: None,
        root_generation: None,
    }
}

#[test]
fn first_use_empty_config_uses_the_same_reversible_engine_and_keeps_original_absence() {
    for prefix in [3, 2, 1] {
        let f = first_use_config(prefix);
        let original_bytes = f.originals.frame(Phase::Original).unwrap().1;
        let source = first_use_source(&f, prefix);
        let pending = if prefix == 3 {
            Pending::prepare_fixture(&f.originals, f.evidence.clone()).unwrap()
        } else {
            Pending::prepare_observed_fixture(&f.originals, &source, f.evidence.clone()).unwrap()
        };
        let path = f.originals.client.path().join("config.toml");
        assert_eq!(fs::read(&path).unwrap(), b"\n");
        let (plan, _) = pending.load().unwrap();
        assert_eq!(plan.schema_version, 5);
        assert_eq!(plan.count(), 4);
        assert!(plan.empty_config.is_some());
        assert!(pending
            .apply_fixture(Decision::Complete, &|point| if point == "after_native" {
                Err(denied("fourth-artifact interruption"))
            } else {
                Ok(())
            })
            .is_err());
        assert!(!path.exists());
        assert_eq!(pending.review_recovery().unwrap().terminal, None);
        pending
            .apply_fixture(Decision::Rollback, &|_| Ok(()))
            .unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"\n");
        assert_eq!(
            pending.review_recovery().unwrap().terminal,
            Some(Decision::Rollback)
        );
        let next = pending.rearm_rolled_back().unwrap();
        next.apply_fixture(Decision::Complete, &|_| Ok(())).unwrap();
        assert!(!path.exists());
        assert_eq!(
            next.review_recovery().unwrap().terminal,
            Some(Decision::Complete)
        );
        assert_eq!(next.object_id(), pending.object_id());
        assert_eq!(
            fs::read(
                f.originals
                    .directory
                    .path()
                    .join(format!("restore-{}-old-3", pending.object_id()))
            )
            .unwrap(),
            b"\n"
        );
        assert_eq!(
            f.originals.frame(Phase::Original).unwrap().1,
            original_bytes
        );
        let original = f.originals.frame(Phase::Original).unwrap().0;
        assert!(original
            .tree
            .restored_matches(&TreeSnapshot::read(&f.originals.client).unwrap(), true));
        if prefix < 3 {
            assert!(!f.originals.directory.path().join("stopped.json").exists());
        }
        if prefix == 1 {
            assert!(!f.originals.directory.path().join("pre-stop.json").exists());
        }
    }
}

#[test]
fn first_use_empty_config_refuses_new_content_replacement_and_missing_ready_provenance() {
    for kind in [0, 1, 2] {
        let f = first_use_config(3);
        let path = f.originals.client.path().join("config.toml");
        if kind == 0 {
            fs::write(&path, b"model='outside'\n").unwrap();
        }
        if kind == 1 {
            fs::rename(
                &path,
                f.originals
                    .client
                    .path()
                    .parent()
                    .unwrap()
                    .join("displaced-empty-config"),
            )
            .unwrap();
            fs::write(&path, b"\n").unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        }
        if kind == 2 {
            fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        }
        let actual = capture_file(&f.originals.client, "config.toml", 4096).unwrap();
        assert!(Pending::prepare_fixture(&f.originals, f.evidence.clone()).is_err());
        assert_eq!(
            capture_file(&f.originals.client, "config.toml", 4096).unwrap(),
            actual
        );
    }
    let f = Fixture::new(); // No config existed at either Original or Ready.
    fs::write(f.originals.client.path().join("config.toml"), b"\n").unwrap();
    // Already independently captured Stopped did not include it: never learn it.
    assert!(Pending::prepare_fixture(&f.originals, f.evidence.clone()).is_err());
    assert!(!f
        .originals
        .frame(Phase::Original)
        .unwrap()
        .0
        .tree
        .restored_matches(&TreeSnapshot::read(&f.originals.client).unwrap(), true));
    assert_eq!(
        fs::read(f.originals.client.path().join("config.toml")).unwrap(),
        b"\n"
    );
}

#[test]
fn first_use_empty_config_publication_race_returns_outside_bytes_and_refuses_success() {
    let f = first_use_config(3);
    let pending = Pending::prepare_fixture(&f.originals, f.evidence.clone()).unwrap();
    let path = f.originals.client.path().join("config.toml");
    let once = Cell::new(false);
    assert!(pending
        .apply_fixture(Decision::Complete, &|point| {
            if point == "native_gap" && !once.replace(true) {
                fs::rename(
                    &path,
                    f.originals
                        .client
                        .path()
                        .parent()
                        .unwrap()
                        .join("displaced-owned-config"),
                )?;
                fs::write(&path, b"outside-user-configuration")?;
            }
            Ok(())
        })
        .is_err());
    assert_eq!(fs::read(&path).unwrap(), b"outside-user-configuration");
    assert!(pending.review_recovery().is_err());
    assert!(!f
        .originals
        .directory
        .path()
        .join(format!("restore-{}-outcome.json", pending.id()))
        .exists());
}

#[test]
fn first_use_empty_config_real_crash_reopens_only_the_original_known_objects() {
    const CHILD: &str = "OCXD024_EMPTY_CONFIG_CRASH";
    if let Some(root) = std::env::var_os(CHILD) {
        let root = PathBuf::from(root);
        let f = first_use_config_at(root.clone(), 3);
        let pending = Pending::prepare_fixture(&f.originals, f.evidence.clone()).unwrap();
        let refs = [Phase::Ready, Phase::PreStop, Phase::Stopped]
            .into_iter()
            .map(|p| (p, digest(&f.originals.frame(p).unwrap().1)))
            .collect::<Vec<_>>();
        fs::write(root.join("independent-receipt.json"),wire(json!({"attempt":f.originals.attempt(),"original":f.originals.original_sha256(),"phases":refs,"id":pending.id(),"sha256":pending.sha256()}))).unwrap();
        let wanted = std::env::var("OCXD024_EMPTY_CONFIG_BOUNDARY").unwrap();
        pending
            .apply_fixture(Decision::Complete, &|point| {
                if point == wanted
                    && (point != "archive"
                        || f.originals
                            .directory
                            .path()
                            .join(format!("restore-{}-old-3", pending.object_id()))
                            .exists())
                {
                    std::process::exit(91);
                }
                Ok(())
            })
            .unwrap();
        panic!("boundary not reached");
    }
    for boundary in ["after_native", "archive_gap", "archive", "terminal"] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let out=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::restore::tests::first_use_empty_config_real_crash_reopens_only_the_original_known_objects","--nocapture"]).env(CHILD,&root).env("OCXD024_EMPTY_CONFIG_BOUNDARY",boundary).output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(91),
            "{} {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let value: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("independent-receipt.json")).unwrap())
                .unwrap();
        let phases: Vec<(Phase, String)> = serde_json::from_value(value["phases"].clone()).unwrap();
        let originals = Originals::open(
            &root.join("desktop"),
            &root.join("codex 中文 space"),
            value["attempt"].as_str().unwrap(),
            value["original"].as_str().unwrap(),
            &phases,
        )
        .unwrap();
        let pending = Pending::open(
            &originals,
            value["id"].as_str().unwrap(),
            value["sha256"].as_str().unwrap(),
        )
        .unwrap();
        pending
            .apply_fixture(Decision::Complete, &|_| Ok(()))
            .unwrap();
        assert!(!originals.client.path().join("config.toml").exists());
        assert_eq!(
            pending.review_recovery().unwrap().terminal,
            Some(Decision::Complete)
        );
    }
}

#[test]
fn first_use_empty_config_never_removes_existing_original_or_unrecorded_ready_content() {
    let temp = tempfile::tempdir().unwrap();
    let f = first_use_config_initial(temp.path().canonicalize().unwrap(), 3, true, true);
    let pending = Pending::prepare_fixture(&f.originals, f.evidence.clone()).unwrap();
    assert!(!pending.empty_config);
    assert_eq!(pending.load().unwrap().0.count(), 3);
    pending
        .apply_fixture(Decision::Complete, &|_| Ok(()))
        .unwrap();
    assert_eq!(
        fs::read(f.originals.client.path().join("config.toml")).unwrap(),
        b"\n"
    );
    let temp = tempfile::tempdir().unwrap();
    let f = first_use_config_initial(temp.path().canonicalize().unwrap(), 3, false, false);
    assert!(Pending::prepare_fixture(&f.originals, f.evidence.clone()).is_err());
    assert_eq!(
        fs::read(f.originals.client.path().join("config.toml")).unwrap(),
        b"\n"
    );
    let f = first_use_config(1);
    fs::write(
        f.originals.client.path().join("config.toml"),
        b"model='outside-before-stop'\n",
    )
    .unwrap();
    f.originals.seal(Phase::PreStop).unwrap();
    fs::write(f.originals.client.path().join("config.toml"), b"\n").unwrap();
    f.originals.seal(Phase::Stopped).unwrap();
    assert!(Pending::prepare_fixture(&f.originals, f.evidence.clone()).is_err());
}

#[test]
fn first_use_empty_config_plan_cannot_be_downgraded_to_omit_the_fourth_target() {
    let f = first_use_config(3);
    let pending = Pending::prepare_fixture(&f.originals, f.evidence.clone()).unwrap();
    let bytes = fs::read(
        f.originals
            .directory
            .path()
            .join(format!("restore-{}.json", pending.id())),
    )
    .unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    value["id"] = json!(id);
    value["schemaVersion"] = json!(3);
    value.as_object_mut().unwrap().remove("emptyConfig");
    let bytes = wire(value);
    create(
        &f.originals.directory,
        &format!("restore-{id}.json"),
        &bytes,
    )
    .unwrap();
    assert!(Pending::open(&f.originals, &id, &digest(&bytes)).is_err());
    assert_eq!(
        fs::read(f.originals.client.path().join("config.toml")).unwrap(),
        b"\n"
    );
}

fn first_grok_backup_at(
    root: PathBuf,
    prefix: usize,
    codex_new: bool,
    existing_backup: bool,
) -> Fixture {
    let data = root.join("desktop");
    let client = root.join("codex 中文 space");
    let grok = root.join(".grok");
    for p in [&data, &client, &grok] {
        fs::DirBuilder::new().mode(0o700).create(p).unwrap();
    }
    let write = |p: &Path, bytes: &[u8]| {
        fs::write(p, bytes).unwrap();
        fs::set_permissions(p, fs::Permissions::from_mode(0o600)).unwrap();
    };
    write(&grok.join("config.toml"), b"model='grok-original'\n");
    write(&grok.join("AGENTS.md"), b"keep original guidance");
    for n in [
        ".opencodex-native-main.claim.sqlite",
        ".opencodex-native-main.owner.sqlite",
    ] {
        write(&client.join(n), b"");
    }
    if existing_backup {
        write(
            &grok.join("config.toml.bak-opencodex"),
            b"previous independent backup",
        );
    }
    if !codex_new {
        write(&client.join("config.toml"), b"codex='original'\n");
    }
    let originals = Originals::begin_for_profile(&data, &client, &root).unwrap();
    if !existing_backup {
        write(
            &grok.join("config.toml.bak-opencodex"),
            b"model='grok-original'\n",
        );
    }
    if codex_new {
        write(&client.join("config.toml"), b"model_provider='opencodex'\n");
    }
    originals.seal(Phase::Ready).unwrap();
    if prefix >= 2 {
        originals.seal(Phase::PreStop).unwrap();
    }
    if codex_new {
        write(&client.join("config.toml"), b"\n");
    }
    if prefix == 3 {
        originals.seal(Phase::Stopped).unwrap();
    }
    Fixture {
        _temp: None,
        originals,
        evidence: Evidence {
            declared_routes: Some(vec!["channel/model".into()]),
            retained_pids: vec![100, 101],
            audit_home: "synthetic-captured-home".into(),
            not_before: chrono::DateTime::parse_from_rfc3339("2026-11-11T10:00:00Z")
                .unwrap()
                .into(),
            not_after: chrono::DateTime::parse_from_rfc3339("2026-11-11T10:00:02Z")
                .unwrap()
                .into(),
        },
    }
}
fn first_grok_backup(prefix: usize, codex_new: bool, existing: bool) -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let mut f = first_grok_backup_at(
        temp.path().canonicalize().unwrap(),
        prefix,
        codex_new,
        existing,
    );
    f._temp = Some(temp);
    f
}
#[test]
fn first_grok_backup_and_optional_codex_config_share_reversible_sparse_targets_without_fake_history(
) {
    for prefix in [3, 2, 1] {
        for codex_new in [false, true] {
            let f = first_grok_backup(prefix, codex_new, false);
            let source = first_use_source(&f, prefix);
            let original = f.originals.frame(Phase::Original).unwrap().1;
            let pending = if prefix == 3 {
                Pending::prepare_fixture(&f.originals, f.evidence.clone()).unwrap()
            } else {
                Pending::prepare_observed_fixture(&f.originals, &source, f.evidence.clone())
                    .unwrap()
            };
            let (plan, _) = pending.load().unwrap();
            assert_eq!(plan.schema_version, 6);
            assert_eq!(plan.count(), 4 + usize::from(codex_new));
            assert_eq!(
                plan.indices().collect::<Vec<_>>(),
                if codex_new {
                    vec![0, 1, 2, 3, 4]
                } else {
                    vec![0, 1, 2, 4]
                }
            );
            let backup = f
                .originals
                .profile
                .path()
                .join(".grok/config.toml.bak-opencodex");
            pending
                .apply_fixture(Decision::Complete, &|_| Ok(()))
                .unwrap();
            assert!(!backup.exists());
            pending
                .apply_fixture(Decision::Rollback, &|_| Ok(()))
                .unwrap_err(); // terminal decision never flips in-place
                               // Separate fixture exercises interruption before common terminal.
            assert_eq!(
                pending.review_recovery().unwrap().terminal,
                Some(Decision::Complete)
            );
            assert_eq!(
                fs::read(
                    f.originals
                        .directory
                        .path()
                        .join(format!("restore-{}-old-4", pending.object_id()))
                )
                .unwrap(),
                b"model='grok-original'\n"
            );
            assert_eq!(f.originals.frame(Phase::Original).unwrap().1, original);
            assert!(f
                .originals
                .frame(Phase::Original)
                .unwrap()
                .0
                .grok
                .restored_matches(&GrokSnapshot::read(&f.originals.profile).unwrap()));
            if prefix < 3 {
                assert!(!f.originals.directory.path().join("stopped.json").exists());
            }
            if prefix == 1 {
                assert!(!f.originals.directory.path().join("pre-stop.json").exists());
            }
        }
    }
}
#[test]
fn first_grok_backup_rollback_rearm_restores_same_object_and_preserves_other_grok_files() {
    let f = first_grok_backup(3, false, false);
    let pending = Pending::prepare_fixture(&f.originals, f.evidence.clone()).unwrap();
    let backup = f
        .originals
        .profile
        .path()
        .join(".grok/config.toml.bak-opencodex");
    assert!(pending
        .apply_fixture(Decision::Complete, &|p| if p == "after_native" {
            Err(denied("backup interruption"))
        } else {
            Ok(())
        })
        .is_err());
    assert!(!backup.exists());
    pending
        .apply_fixture(Decision::Rollback, &|_| Ok(()))
        .unwrap();
    assert_eq!(fs::read(&backup).unwrap(), b"model='grok-original'\n");
    let next = pending.rearm_rolled_back().unwrap();
    next.apply_fixture(Decision::Complete, &|_| Ok(())).unwrap();
    assert!(!backup.exists());
    assert_eq!(
        fs::read(f.originals.profile.path().join(".grok/AGENTS.md")).unwrap(),
        b"keep original guidance"
    );
    assert_eq!(next.object_id(), pending.object_id());
    assert_eq!(
        next.review_recovery().unwrap().terminal,
        Some(Decision::Complete)
    );
}
#[test]
fn first_grok_backup_refuses_modified_or_replaced_backup_and_keeps_existing_original_backup() {
    for replacement in [false, true] {
        let f = first_grok_backup(3, false, false);
        let root = f.originals.profile.path().join(".grok");
        let path = root.join("config.toml.bak-opencodex");
        if replacement {
            fs::rename(
                &path,
                f.originals.profile.path().join("displaced-grok-backup"),
            )
            .unwrap();
            fs::write(&path, b"model='grok-original'\n").unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        } else {
            fs::write(&path, b"outside-user-backup").unwrap();
        }
        let bytes = fs::read(&path).unwrap();
        assert!(Pending::prepare_fixture(&f.originals, f.evidence.clone()).is_err());
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
    let f = first_grok_backup(3, false, true);
    let pending = Pending::prepare_fixture(&f.originals, f.evidence.clone()).unwrap();
    assert!(pending.grok_root.is_none());
    pending
        .apply_fixture(Decision::Complete, &|_| Ok(()))
        .unwrap();
    assert_eq!(
        fs::read(
            f.originals
                .profile
                .path()
                .join(".grok/config.toml.bak-opencodex")
        )
        .unwrap(),
        b"previous independent backup"
    );
}
#[test]
fn first_grok_backup_native_gap_preserves_a_replacement_in_its_original_grok_path() {
    let f = first_grok_backup(3, false, false);
    let pending = Pending::prepare_fixture(&f.originals, f.evidence.clone()).unwrap();
    let backup = f
        .originals
        .profile
        .path()
        .join(".grok/config.toml.bak-opencodex");
    let once = Cell::new(false);
    assert!(pending
        .apply_fixture(Decision::Complete, &|p| {
            if p == "native_gap" && !once.replace(true) {
                fs::rename(
                    &backup,
                    f.originals.profile.path().join("displaced-first-backup"),
                )?;
                fs::write(&backup, b"outside-grok-backup")?;
            }
            Ok(())
        })
        .is_err());
    assert_eq!(fs::read(&backup).unwrap(), b"outside-grok-backup");
    assert!(pending.review_recovery().is_err());
}

#[test]
fn first_grok_backup_real_crash_reopens_only_the_original_known_objects() {
    const CHILD: &str = "OCXD024_GROK_BACKUP_CRASH";
    if let Some(root) = std::env::var_os(CHILD) {
        let root = PathBuf::from(root);
        let f = first_grok_backup_at(root.clone(), 3, false, false);
        let pending = Pending::prepare_fixture(&f.originals, f.evidence.clone()).unwrap();
        let refs = [Phase::Ready, Phase::PreStop, Phase::Stopped]
            .into_iter()
            .map(|p| (p, digest(&f.originals.frame(p).unwrap().1)))
            .collect::<Vec<_>>();
        fs::write(root.join("independent-receipt.json"),wire(json!({"attempt":f.originals.attempt(),"original":f.originals.original_sha256(),"phases":refs,"id":pending.id(),"sha256":pending.sha256()}))).unwrap();
        let wanted = std::env::var("OCXD024_GROK_BACKUP_BOUNDARY").unwrap();
        pending
            .apply_fixture(Decision::Complete, &|point| {
                if point == wanted
                    && (point != "archive"
                        || f.originals
                            .directory
                            .path()
                            .join(format!("restore-{}-old-4", pending.object_id()))
                            .exists())
                {
                    std::process::exit(91);
                }
                Ok(())
            })
            .unwrap();
        panic!("boundary not reached");
    }
    for boundary in ["after_native", "archive_gap", "archive", "terminal"] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let out=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::restore::tests::first_grok_backup_real_crash_reopens_only_the_original_known_objects","--nocapture"]).env(CHILD,&root).env("OCXD024_GROK_BACKUP_BOUNDARY",boundary).output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(91),
            "{} {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let value: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("independent-receipt.json")).unwrap())
                .unwrap();
        let phases: Vec<(Phase, String)> = serde_json::from_value(value["phases"].clone()).unwrap();
        let originals = Originals::open(
            &root.join("desktop"),
            &root.join("codex 中文 space"),
            value["attempt"].as_str().unwrap(),
            value["original"].as_str().unwrap(),
            &phases,
        )
        .unwrap();
        let pending = Pending::open(
            &originals,
            value["id"].as_str().unwrap(),
            value["sha256"].as_str().unwrap(),
        )
        .unwrap();
        pending
            .apply_fixture(Decision::Complete, &|_| Ok(()))
            .unwrap();
        assert!(!originals
            .profile
            .path()
            .join(".grok/config.toml.bak-opencodex")
            .exists());
        assert_eq!(
            fs::read(originals.client.path().join("config.toml")).unwrap(),
            b"codex='original'\n"
        );
        assert_eq!(
            pending.review_recovery().unwrap().terminal,
            Some(Decision::Complete)
        );
    }
}

#[test]
fn first_grok_backup_root_replacement_or_pre_stop_change_cannot_be_adopted() {
    let f = first_grok_backup(3, false, false);
    let pending = Pending::prepare_fixture(&f.originals, f.evidence.clone()).unwrap();
    let path = f.originals.profile.path().join(".grok");
    fs::rename(
        &path,
        f.originals.profile.path().join("displaced-original-grok"),
    )
    .unwrap();
    fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
    fs::write(
        path.join("config.toml.bak-opencodex"),
        b"model='grok-original'\n",
    )
    .unwrap();
    assert!(pending
        .apply_fixture(Decision::Complete, &|_| Ok(()))
        .is_err());
    assert_eq!(
        fs::read(path.join("config.toml.bak-opencodex")).unwrap(),
        b"model='grok-original'\n"
    );
    let f = first_grok_backup(1, false, false);
    let backup = f
        .originals
        .profile
        .path()
        .join(".grok/config.toml.bak-opencodex");
    fs::write(&backup, b"external-pre-stop-change").unwrap();
    f.originals.seal(Phase::PreStop).unwrap();
    fs::write(&backup, b"model='grok-original'\n").unwrap();
    f.originals.seal(Phase::Stopped).unwrap();
    assert!(Pending::prepare_fixture(&f.originals, f.evidence.clone()).is_err());
}
