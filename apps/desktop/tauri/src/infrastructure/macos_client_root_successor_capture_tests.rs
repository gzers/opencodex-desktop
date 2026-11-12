//! Storage-only successor capture tests. No runtime, normal Stop certificate,
//! service permission or replacement of the original generation is simulated.
use super::*;
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    time::Duration,
};

fn fixture() -> (tempfile::TempDir, ProcessContext, Index, Head) {
    let (temp, context, index, closed) =
        super::super::super::root_retirement::fixture_closed_generation();
    let (index, preparing) = index
        .create_successor_root(&context, &closed, Instant::now() + Duration::from_secs(10))
        .unwrap();
    (temp, context, index, preparing)
}
fn synthetic_scope(originals: &Originals, context: &ProcessContext) -> References {
    let scope = serde_json::json!({
        "schemaVersion":1,"attempt":originals.attempt(),
        "originalSha256":originals.original_sha256(),"uid":unsafe{libc::geteuid()},
        "data":originals.data.path(),"dataIdentity":originals.data.identity(),
        "profile":originals.profile.path(),"profileIdentity":originals.profile.identity(),
        "home":context.opencodex_home,
        "homeIdentity":PinnedDirectory::open(&context.opencodex_home).unwrap().identity(),
        "client":originals.client.path(),"clientIdentity":originals.client.identity(),
        "programRoot":originals.data.path().parent().unwrap(),
        "selection":{"schemaVersion":1,"version":"2.99.0","platform":"darwin-arm64",
                     "imageSha256":"a".repeat(64)},
        "capturedAt":"2026-11-13T00:00:00Z","declaredRoutes":[]
    });
    let raw = serde_json::to_vec(&scope).unwrap();
    super::super::super::create(&originals.directory, "startup-scope.json", &raw).unwrap();
    References {
        attempt: originals.attempt().into(),
        original_sha256: originals.original_sha256().into(),
        scope_sha256: digest(&raw),
        phases: vec![],
        live_sha256: None,
    }
}
fn captured(context: &ProcessContext, index: &Index, preparing: &Head) -> (Head, Originals) {
    let reserved = index.reserve_successor_root_capture(preparing).unwrap();
    assert!(reserved.root_generation.is_none());
    let permit = index.capture_reservation(&reserved).unwrap();
    let originals = Originals::begin_reserved_profile(
        index.data_root(),
        &context.working_directory.join(".codex"),
        &context.working_directory,
        &permit,
    )
    .unwrap();
    let refs = synthetic_scope(&originals, context);
    let head = index
        .record_successor_root_originals(context, &reserved, refs)
        .unwrap();
    (head, originals)
}
fn binding(index: &Index) -> Vec<u8> {
    index
        .connection()
        .unwrap()
        .query_row("SELECT document FROM binding WHERE singleton=1", [], |r| {
            r.get(0)
        })
        .unwrap()
}
fn old_vault(index: &Index, preparing: &Head) -> PathBuf {
    let history = vault_history::read(index.connection().unwrap(), preparing).unwrap();
    index
        .data_root()
        .join("manager-state/client-originals")
        .join(&history[0].ticket)
}

#[test]
fn successor_capture_keeps_source_only_in_initial_event_and_registers_distinct_absence_originals() {
    let (_temp, context, index, preparing) = fixture();
    let original_binding = binding(&index);
    let vault = old_vault(&index, &preparing);
    let before = TreeSnapshot::read_mode(&PinnedDirectory::open(&vault).unwrap(), false).unwrap();
    let origin = index.root_registration.clone().unwrap();
    let (head, originals) = captured(&context, &index, &preparing);
    assert_eq!(head.revision, preparing.revision + 2);
    assert_eq!(head.state, State::Captured);
    assert!(head.root_generation.is_none());
    let frame = originals.frame(Phase::Original).unwrap().0;
    assert_eq!(frame.schema_version, 5);
    assert!(frame.original_root_absence.as_ref() == Some(&origin));
    assert!(frame.tree.first_root_execution_baseline());
    assert!(frame
        .capture
        .images
        .iter()
        .all(|image| *image == super::super::super::Image::Absent));
    assert_eq!(binding(&index), original_binding);
    assert!(
        TreeSnapshot::read_mode(&PinnedDirectory::open(&vault).unwrap(), false).unwrap() == before
    );
    assert!(index.reserve_capture(&preparing).is_err());
    assert!(index.reserve_first_root_capture(&preparing).is_err());
    assert!(index
        .advance(&head, State::Ready, head.references.clone())
        .is_err());
    assert!(index.reserve_start().is_err());
    let data = index.data_root().to_path_buf();
    drop(originals);
    drop(index);
    let reopened =
        Index::open_existing(&context, &data, &context.working_directory.join(".codex")).unwrap();
    assert_eq!(reopened.head().unwrap(), Some(head.clone()));
    assert!(
        reopened
            .registered_root_originals(&head)
            .unwrap()
            .original_root_absence
            .as_ref()
            == Some(&origin)
    );
    println!(
        "SUCCESSOR_CAPTURE_EVIDENCE {}",
        serde_json::json!({
            "accepted":false,"scope":"synthetic storage only; no service or Ready permission",
            "preparing":preparing,"captured":head,"origin":origin,
            "initialBindingUnchanged":true,"oldVaultBytesUnchanged":true,
            "originalFrame":frame,
        })
    );
}

#[test]
fn successor_capture_reopen_never_recreates_creation_custody_before_or_after_reservation() {
    for reserved in [false, true] {
        let (_temp, context, index, preparing) = fixture();
        let head = if reserved {
            index.reserve_successor_root_capture(&preparing).unwrap()
        } else {
            preparing
        };
        let data = index.data_root().to_path_buf();
        drop(index);
        let reopened =
            Index::open_existing(&context, &data, &context.working_directory.join(".codex"))
                .unwrap();
        assert_eq!(reopened.head().unwrap(), Some(head.clone()));
        assert!(reopened.reserve_successor_root_capture(&head).is_err());
        if reserved {
            let permit = reopened.capture_reservation(&head).unwrap();
            assert!(Originals::begin_reserved_profile(
                &data,
                &context.working_directory.join(".codex"),
                &context.working_directory,
                &permit
            )
            .is_err());
        }
        assert!(reopened.reserve_first_root_capture(&head).is_err());
        assert!(reopened.reserve_start().is_err());
    }
}

#[test]
fn successor_capture_refuses_stale_source_expired_custody_nonempty_root_or_substituted_fd() {
    for case in ["stale", "expired", "nonempty", "descriptor"] {
        let (_temp, context, mut index, preparing) = fixture();
        let mut expected = preparing.clone();
        match case {
            "stale" => expected.revision += 1,
            "expired" => index.root_creation_deadline = Some(Instant::now()),
            "nonempty" => fs::write(
                context.working_directory.join(".codex/user.txt"),
                b"keep user bytes",
            )
            .unwrap(),
            "descriptor" => {
                index.root_created_fd = Some(File::open(&context.opencodex_home).unwrap())
            }
            _ => unreachable!(),
        }
        assert!(
            index.reserve_successor_root_capture(&expected).is_err(),
            "{case}"
        );
        assert_eq!(
            read_head(index.connection().unwrap()).unwrap(),
            Some(preparing)
        );
        if case == "nonempty" {
            assert_eq!(
                fs::read(context.working_directory.join(".codex/user.txt")).unwrap(),
                b"keep user bytes"
            );
        }
    }
}

#[test]
fn successor_capture_only_projects_exact_reserved_vault_and_never_adopts_partial_originals() {
    for case in [
        "before-reservation",
        "unknown-sibling",
        "partial",
        "symlink",
    ] {
        let (_temp, context, index, preparing) = fixture();
        let vault = index.data_root().join("manager-state/client-originals");
        if case == "before-reservation" {
            fs::DirBuilder::new()
                .mode(0o700)
                .create(vault.join(&preparing.ticket))
                .unwrap();
            assert!(index.reserve_successor_root_capture(&preparing).is_err());
            continue;
        }
        let reserved = index.reserve_successor_root_capture(&preparing).unwrap();
        if case == "unknown-sibling" {
            fs::DirBuilder::new()
                .mode(0o700)
                .create(vault.join(uuid::Uuid::new_v4().to_string()))
                .unwrap();
            assert!(index.head().is_err());
        } else if case == "symlink" {
            std::os::unix::fs::symlink(&context.opencodex_home, vault.join(&reserved.ticket))
                .unwrap();
            assert!(index.head().is_err());
        } else {
            let partial = vault.join(&reserved.ticket);
            fs::DirBuilder::new().mode(0o700).create(&partial).unwrap();
            assert_eq!(index.head().unwrap(), Some(reserved.clone()));
            let permit = index.capture_reservation(&reserved).unwrap();
            assert!(Originals::begin_reserved_profile(
                index.data_root(),
                &context.working_directory.join(".codex"),
                &context.working_directory,
                &permit
            )
            .is_err());
            assert!(fs::read_dir(partial).unwrap().next().is_none());
            assert_eq!(index.head().unwrap(), Some(reserved));
        }
    }
}

#[test]
fn successor_capture_registration_requires_original_digest_scope_and_current_ticket() {
    for case in ["original", "scope", "attempt", "deadline"] {
        let (_temp, context, mut index, preparing) = fixture();
        let reserved = index.reserve_successor_root_capture(&preparing).unwrap();
        let permit = index.capture_reservation(&reserved).unwrap();
        let originals = Originals::begin_reserved_profile(
            index.data_root(),
            &context.working_directory.join(".codex"),
            &context.working_directory,
            &permit,
        )
        .unwrap();
        let mut refs = synthetic_scope(&originals, &context);
        drop(permit);
        match case {
            "original" => refs.original_sha256 = "b".repeat(64),
            "scope" => refs.scope_sha256 = "b".repeat(64),
            "attempt" => refs.attempt = uuid::Uuid::new_v4().to_string(),
            "deadline" => index.root_creation_deadline = Some(Instant::now()),
            _ => unreachable!(),
        }
        assert!(
            index
                .record_successor_root_originals(&context, &reserved, refs)
                .is_err(),
            "{case}"
        );
        assert_eq!(index.head().unwrap(), Some(reserved));
        assert!(originals.directory.path().join("original.json").exists());
    }
}

#[test]
fn successor_capture_history_rejects_skipped_phase_rebound_origin_and_erased_reservation() {
    for case in ["skip", "rebound", "copy-source"] {
        let (_temp, context, index, preparing) = fixture();
        let (head, _originals) = captured(&context, &index, &preparing);
        let c = index.connection().unwrap();
        let raw: Vec<u8> = c
            .query_row(
                "SELECT document FROM events WHERE revision=?1",
                [preparing.revision + 1],
                |r| r.get(0),
            )
            .unwrap();
        let mut changed: Head = decode(&raw).unwrap();
        match case {
            "skip" => {
                changed.state = State::Captured;
                changed.references = head.references.clone();
            }
            "rebound" => changed.ticket = uuid::Uuid::new_v4().to_string(),
            "copy-source" => changed.root_generation = preparing.root_generation.clone(),
            _ => unreachable!(),
        }
        c.execute(
            "UPDATE events SET document=?1 WHERE revision=?2",
            params![encode(&changed).unwrap(), changed.revision],
        )
        .unwrap();
        assert!(index.head().is_err(), "{case}");
    }
}

#[test]
fn successor_capture_never_hides_changed_or_missing_predecessor_archive() {
    for after in [false, true] {
        let (_temp, context, index, preparing) = fixture();
        let vault = old_vault(&index, &preparing);
        let expected = if after {
            captured(&context, &index, &preparing).0
        } else {
            preparing
        };
        let archived = vault.join("original.json");
        let raw = fs::read(&archived).unwrap();
        fs::set_permissions(&archived, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&archived, b"changed old evidence").unwrap();
        assert!(index.head().is_err());
        assert!(index.reserve_successor_root_capture(&expected).is_err());
        assert_eq!(fs::read(&archived).unwrap(), b"changed old evidence");
        fs::write(&archived, raw).unwrap();
        fs::remove_file(&archived).unwrap();
        assert!(index.head().is_err());
    }
}

#[test]
fn successor_capture_matching_scope_hash_cannot_rebind_home_client_or_original() {
    for field in ["homeIdentity", "clientIdentity", "client", "originalSha256"] {
        let (_temp, context, index, preparing) = fixture();
        let reserved = index.reserve_successor_root_capture(&preparing).unwrap();
        let permit = index.capture_reservation(&reserved).unwrap();
        let originals = Originals::begin_reserved_profile(
            index.data_root(),
            &context.working_directory.join(".codex"),
            &context.working_directory,
            &permit,
        )
        .unwrap();
        let mut refs = synthetic_scope(&originals, &context);
        let path = originals.directory.path().join("startup-scope.json");
        let mut scope: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        scope[field] = match field {
            "homeIdentity" => serde_json::to_value(originals.client.identity()).unwrap(),
            "clientIdentity" => serde_json::to_value(originals.data.identity()).unwrap(),
            "client" => serde_json::to_value(&context.opencodex_home).unwrap(),
            "originalSha256" => serde_json::Value::String("b".repeat(64)),
            _ => unreachable!(),
        };
        let raw = serde_json::to_vec(&scope).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, &raw).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        refs.scope_sha256 = digest(&raw);
        assert!(
            index
                .record_successor_root_originals(&context, &reserved, refs)
                .is_err(),
            "{field}"
        );
        assert_eq!(index.head().unwrap(), Some(reserved));
        assert_eq!(fs::read(&path).unwrap(), raw);
    }
}

#[test]
fn successor_capture_partial_vault_uses_membership_projection_without_client_tree_budget_or_adoption(
) {
    let (_temp, context, index, preparing) = fixture();
    let reserved = index.reserve_successor_root_capture(&preparing).unwrap();
    let directory = index
        .data_root()
        .join("manager-state/client-originals")
        .join(&reserved.ticket);
    fs::DirBuilder::new()
        .mode(0o700)
        .create(&directory)
        .unwrap();
    let path = directory.join("partial-unadmitted-blob");
    let file = fs::File::create(&path).unwrap();
    file.set_len(17 * 1024 * 1024).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    for n in 0..16_384 {
        fs::write(directory.join(format!("partial-{n}")), b"").unwrap();
    }
    assert!(
        super::super::super::tree::names(PinnedDirectory::open(&directory).unwrap().file())
            .is_err()
    );
    assert_eq!(index.head().unwrap(), Some(reserved.clone()));
    let permit = index.capture_reservation(&reserved).unwrap();
    assert!(Originals::begin_reserved_profile(
        index.data_root(),
        &context.working_directory.join(".codex"),
        &context.working_directory,
        &permit
    )
    .is_err());
    assert!(index.registered_root_originals(&reserved).is_err());
    assert_eq!(fs::metadata(&path).unwrap().len(), 17 * 1024 * 1024);
    assert_eq!(index.head().unwrap(), Some(reserved));
}

#[test]
fn successor_capture_publication_rechecks_custody_and_deadline_after_sql_and_commit() {
    for point in ["references_sql_written", "references_committed"] {
        for change in ["deadline", "root-identity"] {
            let (_temp, context, index, preparing) = fixture();
            let reserved = index.reserve_successor_root_capture(&preparing).unwrap();
            let permit = index.capture_reservation(&reserved).unwrap();
            let originals = Originals::begin_reserved_profile(
                index.data_root(),
                &context.working_directory.join(".codex"),
                &context.working_directory,
                &permit,
            )
            .unwrap();
            let refs = synthetic_scope(&originals, &context);
            let mut next = reserved.clone();
            next.state = State::Captured;
            next.revision += 1;
            next.references = Some(refs);
            let reached = std::cell::Cell::new(false);
            let result = index.write_checked(Some(&reserved), &next, &|stage| {
                if stage == point {
                    reached.set(true);
                    if change == "deadline" {
                        let deadline = index.root_creation_deadline.unwrap();
                        std::thread::sleep(
                            deadline.saturating_duration_since(Instant::now())
                                + Duration::from_millis(1),
                        );
                    } else {
                        let logical = context.working_directory.join(".codex");
                        fs::rename(
                            &logical,
                            context.working_directory.join("external-preserved-root"),
                        )
                        .unwrap();
                        fs::DirBuilder::new().mode(0o700).create(logical).unwrap();
                    }
                }
                Ok(())
            });
            assert!(
                reached.get(),
                "{point}/{change}: checkpoint was not reached"
            );
            assert!(result.is_err(), "{point}/{change}");
            let committed = point == "references_committed";
            assert_eq!(
                read_head(index.connection().unwrap()).unwrap(),
                Some(if committed {
                    next.clone()
                } else {
                    reserved.clone()
                })
            );
            let count: u64 = index
                .connection()
                .unwrap()
                .query_row("SELECT count(*) FROM events", [], |r| r.get(0))
                .unwrap();
            assert_eq!(
                count,
                if committed {
                    next.revision
                } else {
                    reserved.revision
                }
            );
            assert!(originals.directory.path().join("original.json").exists());
            if change == "root-identity" {
                assert!(context
                    .working_directory
                    .join("external-preserved-root")
                    .exists());
            }
            println!(
                "SUCCESSOR_CAPTURE_PUBLICATION_EVIDENCE {}",
                serde_json::json!({"accepted":false,"point":point,"change":change,"returnedError":true,"committed":committed,"retainedRevision":count,"originalRetained":true})
            );
        }
    }
}

#[test]
fn successor_capture_storage_close_retains_both_archives_and_does_not_grant_third_start() {
    let (_temp, context, index, preparing) = fixture();
    let old = old_vault(&index, &preparing);
    let before = TreeSnapshot::read_mode(&PinnedDirectory::open(&old).unwrap(), false).unwrap();
    let (captured, originals) = captured(&context, &index, &preparing);
    let pending =
        super::super::super::root_retirement::Pending::prepare(&context, &originals, &captured)
            .unwrap();
    let registered = index
        .record_root_retirement(&captured, pending.id(), pending.sha256())
        .unwrap();
    let proof = pending
        .fixture_apply_for_index(Instant::now() + Duration::from_secs(10))
        .unwrap();
    let closed = index.close_root_absence(&registered, &proof).unwrap();
    assert_eq!(closed.state, State::RootAbsent);
    assert_eq!(closed.revision, preparing.revision + 4);
    assert!(!context.working_directory.join(".codex").exists());
    assert!(
        TreeSnapshot::read_mode(&PinnedDirectory::open(&old).unwrap(), false).unwrap() == before
    );
    let archive = index.archived_root_originals(&closed).unwrap();
    assert_eq!(archive.attempt(), preparing.ticket);
    assert_eq!(index.head().unwrap(), Some(closed.clone()));
    let data = index.data_root().to_path_buf();
    drop(archive);
    drop(pending);
    drop(originals);
    drop(index);
    let reopened =
        Index::open_existing(&context, &data, &context.working_directory.join(".codex")).unwrap();
    assert_eq!(reopened.head().unwrap(), Some(closed.clone()));
    assert!(reopened.reserve_start().is_err());
    assert!(reopened
        .create_successor_root(&context, &closed, Instant::now() + Duration::from_secs(10))
        .is_err());
    println!(
        "SUCCESSOR_CAPTURE_CLOSE_EVIDENCE {}",
        serde_json::json!({
            "accepted":false,"scope":"synthetic storage close, not normal service Stop",
            "closed":closed,"oldArchiveUnchanged":true,"thirdStartStillRefused":true
        })
    );
}

#[test]
fn successor_capture_process_exit_retains_exact_phase_without_recreating_permission() {
    const TEST: &str = "infrastructure::macos_client_originals::session_index::generation::capture_tests::successor_capture_process_exit_retains_exact_phase_without_recreating_permission";
    if let (Some(root), Some(point)) = (
        std::env::var_os("OCXD024_SUCCESSOR_CAPTURE_ROOT"),
        std::env::var_os("OCXD024_SUCCESSOR_CAPTURE_POINT"),
    ) {
        let root = PathBuf::from(root);
        assert!(root.starts_with("/private/tmp"));
        assert!(root.ancestors().any(|p| p.file_name().is_some_and(|n| n
            .to_string_lossy()
            .starts_with("ocxd024-successor-capture-"))));
        let (context, index, closed) =
            super::super::super::root_retirement::fixture_closed_generation_at(root);
        let (index, preparing) = index
            .create_successor_root(&context, &closed, Instant::now() + Duration::from_secs(10))
            .unwrap();
        let reserved = index.reserve_successor_root_capture(&preparing).unwrap();
        if point == "reserved" {
            unsafe { libc::_exit(91) }
        }
        let permit = index.capture_reservation(&reserved).unwrap();
        let originals = Originals::begin_reserved_profile(
            index.data_root(),
            &context.working_directory.join(".codex"),
            &context.working_directory,
            &permit,
        )
        .unwrap();
        if point == "original-published" {
            unsafe { libc::_exit(91) }
        }
        assert_eq!(point, "captured");
        let refs = synthetic_scope(&originals, &context);
        index
            .record_successor_root_originals(&context, &reserved, refs)
            .unwrap();
        unsafe { libc::_exit(91) }
    }
    fn raw_files(root: &Path, path: &Path, files: &mut Vec<serde_json::Value>) {
        for e in fs::read_dir(path).unwrap() {
            let path = e.unwrap().path();
            if fs::symlink_metadata(&path).unwrap().is_dir() {
                raw_files(root, &path, files);
            } else {
                let raw = fs::read(&path).unwrap();
                files.push(serde_json::json!({"path":path.strip_prefix(root).unwrap(),"sha256":digest(&raw),"rawBase64":base64::Engine::encode(&base64::engine::general_purpose::STANDARD,raw)}));
            }
        }
    }
    for point in ["reserved", "original-published", "captured"] {
        let temp = tempfile::Builder::new()
            .prefix("ocxd024-successor-capture-exit-")
            .tempdir()
            .unwrap();
        let root = temp.path().canonicalize().unwrap();
        let out = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", TEST, "--nocapture"])
            .env("OCXD024_SUCCESSOR_CAPTURE_ROOT", &root)
            .env("OCXD024_SUCCESSOR_CAPTURE_POINT", point)
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(91),
            "{point}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let mut files = Vec::new();
        raw_files(&root, &root, &mut files);
        let context = ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(
            ),
            working_directory: root.join("profile"),
            opencodex_home: root.join("profile/home"),
        };
        let data = root.join("desktop");
        let index = Index::open_existing(&context, &data, &root.join("profile/.codex")).unwrap();
        let head = index.head().unwrap().unwrap();
        let is_captured = point == "captured";
        assert_eq!(
            head.state,
            if is_captured {
                State::Captured
            } else {
                State::CaptureReserved
            }
        );
        assert_eq!(head.revision, if is_captured { 8 } else { 7 });
        assert!(index.reserve_successor_root_capture(&head).is_err());
        assert!(index.reserve_first_root_capture(&head).is_err());
        assert!(index.reserve_start().is_err());
        if !is_captured {
            let permit = index.capture_reservation(&head).unwrap();
            assert!(Originals::begin_reserved_profile(
                &data,
                &root.join("profile/.codex"),
                &context.working_directory,
                &permit
            )
            .is_err());
            assert!(index.registered_root_originals(&head).is_err());
        } else {
            assert!(index.registered_root_originals(&head).is_ok());
        }
        println!(
            "SUCCESSOR_CAPTURE_EXIT_EVIDENCE {}",
            serde_json::json!({"accepted":false,"point":point,"exitCode":out.status.code(),"head":head,"files":files,"reopenedCaptureRefused":true})
        );
    }
}
