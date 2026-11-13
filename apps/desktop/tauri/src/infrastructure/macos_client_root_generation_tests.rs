//! Second-generation creation/index primitives only. No service is started and
//! current ordinary capture/Start remains closed until the next integration.
use super::*;
use std::{fs, os::unix::fs::DirBuilderExt, time::Duration};
fn fixture() -> (tempfile::TempDir, ProcessContext, Index, Head) {
    super::super::super::root_retirement::fixture_closed_generation()
}
fn bytes(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, path: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for e in fs::read_dir(path).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(root, &p, out)
            } else {
                out.insert(
                    p.strip_prefix(root).unwrap().to_str().unwrap().into(),
                    fs::read(p).unwrap(),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}
#[test]
fn successor_root_registers_new_origin_and_preparing_atomically_without_replacing_old_evidence() {
    let (_tmp, context, index, closed) = fixture();
    let data = index.data_root().to_path_buf();
    let manager = data.join("manager-state");
    let old_binding: Vec<u8> = index
        .connection()
        .unwrap()
        .query_row("SELECT document FROM binding WHERE singleton=1", [], |r| {
            r.get(0)
        })
        .unwrap();
    let oldvault = bytes(&manager.join("client-originals"));
    let oldroot = index.client.identity().clone();
    let (index, preparing) = index
        .create_successor_root(&context, &closed, Instant::now() + Duration::from_secs(10))
        .unwrap();
    assert_eq!(preparing.state, State::Preparing);
    assert_eq!(preparing.revision, closed.revision + 1);
    assert_ne!(preparing.ticket, closed.ticket);
    assert!(preparing.references.is_none());
    let generation = preparing.root_generation.as_ref().unwrap();
    assert_eq!(generation.predecessor_revision, closed.revision);
    assert_eq!(
        generation.predecessor_sha256,
        digest(&encode(&closed).unwrap())
    );
    let origin = root_binding::original_root(&generation.binding).unwrap();
    assert_eq!(origin.id, preparing.ticket);
    assert_eq!(
        origin.generation_slot.as_deref(),
        Some(preparing.ticket.as_str())
    );
    assert_ne!(index.client.identity(), &oldroot);
    assert!(fs::read_dir(context.working_directory.join(".codex"))
        .unwrap()
        .next()
        .is_none());
    assert_eq!(index.head().unwrap(), Some(preparing.clone()));
    assert_eq!(bytes(&manager.join("client-originals")), oldvault);
    let binding: Vec<u8> = index
        .connection()
        .unwrap()
        .query_row("SELECT document FROM binding WHERE singleton=1", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(binding, old_binding);
    let raw: Vec<u8> = index
        .connection()
        .unwrap()
        .query_row(
            "SELECT document FROM events WHERE revision=?1",
            [preparing.revision],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(raw, encode(&preparing).unwrap());
    assert!(index.reserve_start().is_err());
    assert!(index.reserve_capture(&preparing).is_err());
    assert!(
        index.reserve_first_root_capture(&preparing).is_err(),
        "atomic creation alone must not open unintegrated later capture"
    );
    let retained = preparing.clone();
    drop(index);
    let reopened =
        Index::open_existing(&context, &data, &context.working_directory.join(".codex")).unwrap();
    assert_eq!(reopened.head().unwrap(), Some(retained));
    println!(
        "SUCCESSOR_ROOT_EVIDENCE {}",
        serde_json::json!({"accepted":false,"predecessor":closed,"preparing":preparing,"originalBindingUnchanged":true,"oldVaultBytesUnchanged":true,"newRootIdentity":reopened.client.identity(),"noCaptureOrServicePermission":true})
    );
}
#[test]
fn successor_root_refuses_new_user_directory_unsafe_paths_and_expired_or_stale_predecessor() {
    for case in ["new-root", "expired", "stale", "extra-vault"] {
        let (_tmp, context, index, mut closed) = fixture();
        let data = index.data_root().to_path_buf();
        let root = context.working_directory.join(".codex");
        match case {
            "new-root" => {
                fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
                fs::write(root.join("outside"), b"new user").unwrap();
            }
            "stale" => closed.revision += 1,
            "extra-vault" => {
                fs::create_dir(data.join("manager-state/client-originals/unregistered")).unwrap();
            }
            _ => {}
        }
        let deadline = if case == "expired" {
            Instant::now()
        } else {
            Instant::now() + Duration::from_secs(10)
        };
        assert!(index
            .create_successor_root(&context, &closed, deadline)
            .is_err());
        assert!(!data.join("manager-state/client-root-generations").exists());
        if case == "new-root" {
            assert_eq!(fs::read(root.join("outside")).unwrap(), b"new user");
        }
    }
}
#[test]
fn successor_generation_missing_or_changed_registration_cannot_be_adopted_on_reopen() {
    let (_tmp, context, index, closed) = fixture();
    let data = index.data_root().to_path_buf();
    let (index, head) = index
        .create_successor_root(&context, &closed, Instant::now() + Duration::from_secs(10))
        .unwrap();
    let mut wrong = head.clone();
    wrong.root_generation.as_mut().unwrap().predecessor_sha256 = "a".repeat(64);
    let raw = encode(&wrong).unwrap();
    let c = index.connection().unwrap();
    c.execute(
        "UPDATE events SET document=?1 WHERE revision=?2",
        params![&raw, head.revision],
    )
    .unwrap();
    c.execute(
        "UPDATE current_session SET document=?1 WHERE revision=?2",
        params![&raw, head.revision],
    )
    .unwrap();
    assert!(index.head().is_err());
    drop(index);
    assert!(
        Index::open_existing(&context, &data, &context.working_directory.join(".codex")).is_err()
    );
    assert!(context.working_directory.join(".codex").is_dir());
}
#[test]
fn successor_generation_precommit_failure_retains_old_index_and_unadoptable_new_root() {
    let (_tmp, context, index, closed) = fixture();
    let data = index.data_root().to_path_buf();
    let before = bytes(&data.join("manager-state/client-originals"));
    let result = index.create_successor_root_checked(
        &context,
        &closed,
        Instant::now() + Duration::from_secs(10),
        &|phase| {
            if phase == "before_generation_commit" {
                Err(denied("owned precommit interruption"))
            } else {
                Ok(())
            }
        },
    );
    assert!(result.is_err());
    assert!(context.working_directory.join(".codex").is_dir());
    assert_eq!(bytes(&data.join("manager-state/client-originals")), before);
    // Read the terminal fixture after all original Index fds closed; no live
    // SQLite lock is invalidated by this supplemental immutable evidence open.
    let c = Connection::open_with_flags(
        data.join("manager-state/client-session-index.sqlite"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let head = read_head(&c).unwrap().unwrap();
    assert_eq!(head, closed);
    let count: u64 = c
        .query_row("SELECT count(*) FROM events", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, closed.revision);
    c.close().unwrap();
    assert!(
        Index::open_existing(&context, &data, &context.working_directory.join(".codex")).is_err()
    );
}

#[test]
fn successor_generation_unknown_slot_or_late_user_data_cannot_be_committed_or_cleaned_implicitly() {
    for case in ["slot", "file"] {
        let (_tmp, context, index, closed) = fixture();
        let data = index.data_root().to_path_buf();
        let profile = context.working_directory.clone();
        let touched = std::cell::Cell::new(false);
        assert!(index
            .create_successor_root_checked(
                &context,
                &closed,
                Instant::now() + Duration::from_secs(10),
                &|phase| {
                    if phase == "before_generation_commit" {
                        if case == "slot" {
                            let p = data.join("manager-state/client-root-generations/unknown");
                            fs::create_dir(&p).unwrap();
                            fs::write(p.join("outside"), b"unknown reservation").unwrap();
                        } else {
                            fs::write(profile.join(".codex/outside"), b"new user object").unwrap();
                        }
                        touched.set(true);
                    }
                    Ok(())
                }
            )
            .is_err());
        assert!(touched.get());
        if case == "slot" {
            assert_eq!(
                fs::read(data.join("manager-state/client-root-generations/unknown/outside"))
                    .unwrap(),
                b"unknown reservation"
            );
        } else {
            assert_eq!(
                fs::read(profile.join(".codex/outside")).unwrap(),
                b"new user object"
            );
        }
    }
}
#[test]
fn successor_generation_creation_keeps_same_action_flock_until_index_owner_exits() {
    const PROBE: &str = "OCXD_SUCCESSOR_ACTION_PROBE";
    if let Some(path) = std::env::var_os(PROBE) {
        let f = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let locked = std::env::var("OCXD_SUCCESSOR_ACTION_BUSY").unwrap() == "1";
        assert_eq!(f.try_lock().is_err(), locked);
        return;
    }
    let (_tmp, context, index, closed) = fixture();
    let action = index
        .action_parent
        .path()
        .join("client-session-action.lock");
    let identity = index.action_identity.clone();
    let probe = |busy: bool| {
        let result=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::session_index::generation::tests::successor_generation_creation_keeps_same_action_flock_until_index_owner_exits","--nocapture","--test-threads=1"]).env(PROBE,&action).env("OCXD_SUCCESSOR_ACTION_BUSY",if busy{"1"}else{"0"}).output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    };
    probe(true);
    let (index, _) = index
        .create_successor_root(&context, &closed, Instant::now() + Duration::from_secs(10))
        .unwrap();
    assert_eq!(index.action_identity, identity);
    probe(true);
    drop(index);
    probe(false);
}
fn raw_evidence(root: &Path) -> Vec<serde_json::Value> {
    bytes(root).into_iter().map(|(path,raw)|serde_json::json!({"path":path,"bytes":raw.len(),"sha256":digest(&raw),"rawBase64":base64::Engine::encode(&base64::engine::general_purpose::STANDARD,raw)})).collect()
}
#[test]
fn successor_generation_actual_process_exit_keeps_predecessor_or_whole_atomic_preparing() {
    const CHILD: &str = "OCXD_SUCCESSOR_GENERATION_EXIT_ROOT";
    if let Some(root) = std::env::var_os(CHILD) {
        let root = PathBuf::from(root);
        let (context, index, closed) =
            super::super::super::root_retirement::fixture_closed_generation_at(root.clone());
        fs::write(root.join("original-closed.json"), encode(&closed).unwrap()).unwrap();
        let point = std::env::var("OCXD_SUCCESSOR_GENERATION_POINT").unwrap();
        index
            .create_successor_root_checked(
                &context,
                &closed,
                Instant::now() + Duration::from_secs(10),
                &|phase| {
                    if phase == point {
                        std::process::exit(91);
                    }
                    Ok(())
                },
            )
            .unwrap();
        panic!("exit boundary not reached");
    }
    for point in [
        "generation_reserved",
        "generation_published",
        "generation_event_inserted",
        "generation_head_updated",
        "generation_committed",
    ] {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let output=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::session_index::generation::tests::successor_generation_actual_process_exit_keeps_predecessor_or_whole_atomic_preparing","--nocapture","--test-threads=1"]).env(CHILD,&root).env("OCXD_SUCCESSOR_GENERATION_POINT",point).output().unwrap();
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
        let data = root.join("desktop");
        let connection = Connection::open_with_flags(
            format!(
                "file:{}?mode=ro&immutable=1",
                data.join("manager-state/client-session-index.sqlite")
                    .display()
            ),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        )
        .unwrap();
        let head = read_head(&connection).unwrap().unwrap();
        let closed: Head =
            serde_json::from_slice(&fs::read(root.join("original-closed.json")).unwrap()).unwrap();
        connection.close().unwrap();
        if point == "generation_committed" {
            assert_eq!(head.state, State::Preparing);
            assert_eq!(head.revision, closed.revision + 1);
            assert!(head.root_generation.is_some());
            let index =
                Index::open_existing(&context, &data, &context.working_directory.join(".codex"))
                    .unwrap();
            assert_eq!(index.head().unwrap(), Some(head.clone()));
            assert!(
                index.reserve_first_root_capture(&head).is_err(),
                "reopened files cannot mint original live creation custody"
            );
            drop(index);
        } else {
            assert_eq!(head, closed);
            if point != "generation_reserved" {
                assert!(Index::open_existing(
                    &context,
                    &data,
                    &context.working_directory.join(".codex")
                )
                .is_err());
            }
        }
        println!(
            "SUCCESSOR_GENERATION_EXIT_EVIDENCE {}",
            serde_json::json!({"accepted":false,"point":point,"actualWaitExitCode":91,"head":head,"predecessor":closed,"rootExists":context.working_directory.join(".codex").exists(),"noServiceOrRecapturedCustody":true,"files":raw_evidence(&root)})
        );
    }
}

#[test]
fn successor_generation_transaction_tail_rechecks_old_archive_and_empty_created_root() {
    for phase in [
        "before_generation_commit",
        "generation_event_inserted",
        "generation_head_updated",
        "generation_committed",
    ] {
        for what in ["user-file", "old-archive"] {
            let (_tmp, context, index, closed) = fixture();
            let data = index.data_root().to_path_buf();
            let profile = context.working_directory.clone();
            let old = closed.clone();
            let archive = data
                .join("manager-state/client-originals")
                .join(&closed.ticket)
                .join(format!(
                    "root-retired-{}",
                    closed.root_retirement.as_ref().unwrap().plan.id
                ));
            let touched = std::cell::Cell::new(false);
            let result = index.create_successor_root_checked(
                &context,
                &closed,
                Instant::now() + Duration::from_secs(10),
                &|actual| {
                    if actual == phase {
                        if what == "user-file" {
                            fs::write(profile.join(".codex/outside"), b"late user bytes").unwrap();
                        } else {
                            fs::write(archive.join("outside"), b"late archive bytes").unwrap();
                        }
                        touched.set(true);
                    }
                    Ok(())
                },
            );
            assert!(
                result.is_err(),
                "{phase}/{what} cannot return valid custody"
            );
            assert!(touched.get());
            let connection = Connection::open_with_flags(
                data.join("manager-state/client-session-index.sqlite"),
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            )
            .unwrap();
            let head = read_head(&connection).unwrap().unwrap();
            connection.close().unwrap();
            if phase == "generation_committed" {
                assert_eq!(head.state, State::Preparing);
                assert_eq!(head.revision, old.revision + 1);
            } else {
                assert_eq!(
                    head, old,
                    "precommit mutation must rollback BOTH generation event and current Head"
                );
            }
            if what == "user-file" {
                assert_eq!(
                    fs::read(profile.join(".codex/outside")).unwrap(),
                    b"late user bytes"
                );
            } else {
                assert_eq!(
                    fs::read(archive.join("outside")).unwrap(),
                    b"late archive bytes"
                );
            }
        }
    }
}
