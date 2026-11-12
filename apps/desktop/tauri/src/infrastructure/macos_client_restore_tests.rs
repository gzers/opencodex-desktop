use super::*;
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
    let pending = f.prepare();
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
        let pending = f.prepare();
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
