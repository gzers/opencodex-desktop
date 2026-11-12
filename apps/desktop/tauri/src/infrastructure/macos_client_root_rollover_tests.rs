//! Multigeneration storage and real process-exit evidence only. No official
//! service, Ready/normal Stop, new Start/status rollout or power-loss claim.
use super::*;
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    time::Duration,
};
fn close_successor(context: &ProcessContext, index: Index, preparing: &Head) -> (Index, Head) {
    let (captured, originals) = super::capture_tests::captured(context, &index, preparing);
    let pending =
        super::super::super::root_retirement::Pending::prepare(context, &originals, &captured)
            .unwrap();
    let registered = index
        .record_root_retirement(&captured, pending.id(), pending.sha256())
        .unwrap();
    let proof = pending
        .fixture_apply_for_index(Instant::now() + Duration::from_secs(10))
        .unwrap();
    let closed = index.close_root_absence(&registered, &proof).unwrap();
    drop(pending);
    drop(originals);
    (index, closed)
}
fn fixture_at(root: PathBuf, count: usize) -> (ProcessContext, Index, Head) {
    let (context, mut index, mut closed) =
        super::super::super::root_retirement::fixture_closed_generation_at(root);
    for _ in 1..count {
        let (next, preparing) = index
            .create_successor_root(&context, &closed, Instant::now() + Duration::from_secs(30))
            .unwrap();
        (index, closed) = close_successor(&context, next, &preparing);
    }
    (context, index, closed)
}
fn fixture(count: usize) -> (tempfile::TempDir, ProcessContext, Index, Head) {
    let temp = tempfile::tempdir().unwrap();
    let (context, index, head) = fixture_at(temp.path().canonicalize().unwrap(), count);
    (temp, context, index, head)
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
fn snapshot(path: &Path) -> TreeSnapshot {
    TreeSnapshot::read_mode(&PinnedDirectory::open(path).unwrap(), false).unwrap()
}

#[test]
fn root_rollover_four_generations_preserve_every_origin_event_archive_and_namespace() {
    let (temp, context, mut index, mut closed) = fixture(1);
    let initial = binding(&index);
    let data = index.data_root().to_path_buf();
    let mut expected = Vec::new();
    for generation in 1..=4 {
        assert_eq!(closed.revision, generation * 5);
        let references = index.closed_root_history(&closed).unwrap();
        assert_eq!(references.len(), generation as usize);
        assert!(!context.working_directory.join(".codex").exists());
        let history = vault_history::read(index.connection().unwrap(), &closed).unwrap();
        let lineage = root_lineage::parse(&initial, &history).unwrap();
        assert_eq!(lineage.slots().len(), generation as usize - 1);
        assert_eq!(lineage.current().origin.id, closed.ticket);
        assert_eq!(binding(&index), initial);
        for (id, before) in &expected {
            assert!(snapshot(&data.join("manager-state/client-originals").join(id)) == *before);
        }
        for reference in &references {
            let original = super::super::super::root_retirement::open_closed_history(
                &data,
                &context.working_directory,
                reference,
            )
            .unwrap();
            assert_eq!(original.attempt(), reference.closed.ticket);
            root_binding::verify_initial_event(
                index.connection().unwrap(),
                &reference.original_root,
            )
            .unwrap();
        }
        expected.push((
            closed.ticket.clone(),
            snapshot(
                &data
                    .join("manager-state/client-originals")
                    .join(&closed.ticket),
            ),
        ));
        if generation < 4 {
            let prefix = history.clone();
            let (next, preparing) = index
                .create_successor_root(&context, &closed, Instant::now() + Duration::from_secs(30))
                .unwrap();
            assert_eq!(preparing.revision, closed.revision + 1);
            assert!(next.reserve_start().is_err());
            (index, closed) = close_successor(&context, next, &preparing);
            assert_eq!(
                &vault_history::read(index.connection().unwrap(), &closed).unwrap()[..prefix.len()],
                prefix.as_slice()
            );
        }
    }
    let final_refs = index.closed_root_history(&closed).unwrap();
    drop(index);
    let reopened =
        Index::open_existing(&context, &data, &context.working_directory.join(".codex")).unwrap();
    assert_eq!(reopened.head().unwrap(), Some(closed.clone()));
    assert_eq!(reopened.closed_root_history(&closed).unwrap().len(), 4);
    assert!(reopened.reserve_start().is_err());
    let mut files = Vec::new();
    raw_files(temp.path(), temp.path(), &mut files);
    println!(
        "ROOT_ROLLOVER_FOUR_EVIDENCE {}",
        serde_json::json!({"accepted":false,"head":closed,"generations":4,"oldBindingEventsAndArchivesUnchanged":true,"origins":final_refs.iter().map(|r|&r.original_root).collect::<Vec<_>>(),"productionStartStillRefused":true,"files":files})
    );
}

#[test]
fn root_rollover_namespace_facts_never_serialize_or_survive_as_an_unchecked_claim() {
    let (_temp, context, index, closed) = fixture(3);
    let data = index.data_root();
    for reference in index.closed_root_history(&closed).unwrap() {
        let raw = serde_json::to_vec(&reference).unwrap();
        let plain: super::super::super::root_retirement::ClosedRootReferences =
            serde_json::from_slice(&raw).unwrap();
        assert!(plain.original_root == reference.original_root);
        assert!(super::super::super::root_retirement::open_closed_history(
            data,
            &context.working_directory,
            &reference
        )
        .is_ok());
        assert!(super::super::super::root_retirement::open_closed_history(
            data,
            &context.working_directory,
            &plain
        )
        .is_err());
        let mut value = serde_json::to_value(&reference.original_root).unwrap();
        assert!(value.get("namespace").is_none());
        value["namespace"] = serde_json::json!(["arbitrary directory"]);
        if let Ok(untrusted) =
            serde_json::from_value::<super::super::super::root_creation::References>(value)
        {
            let mut poisoned = reference.clone();
            poisoned.original_root = untrusted;
            assert!(super::super::super::root_retirement::open_closed_history(
                data,
                &context.working_directory,
                &poisoned
            )
            .is_err());
        }
    }
}
#[test]
fn root_rollover_reclose_cannot_change_an_earlier_immutable_completion() {
    let (_temp, _context, index, closed) = fixture(1);
    let mut history = vault_history::read(index.connection().unwrap(), &closed).unwrap();
    let mut prepared = history[3].clone();
    prepared.revision = 6;
    let mut reclosed = closed.clone();
    reclosed.revision = 7;
    history.push(prepared);
    history.push(reclosed);
    let initial = binding(&index);
    assert!(root_lineage::parse(&initial, &history).is_ok());
    history[4].completion_sha256 = Some("b".repeat(64));
    assert!(root_lineage::parse(&initial, &history).is_err());
}

#[test]
fn root_rollover_parser_refuses_reused_identity_changed_context_phase_or_predecessor() {
    let (_temp, _context, index, closed) = fixture(3);
    let original = binding(&index);
    let history = vault_history::read(index.connection().unwrap(), &closed).unwrap();
    for case in [
        "digest",
        "skip-close",
        "context",
        "identity",
        "phase",
        "ticket",
        "copy-source",
    ] {
        let mut changed = history.clone();
        match case {
            "digest" => {
                changed[10]
                    .root_generation
                    .as_mut()
                    .unwrap()
                    .predecessor_sha256 = "b".repeat(64)
            }
            "skip-close" => {
                changed[9].state = State::RootRetirementPrepared;
                changed[9].completion_sha256 = None;
                let sha = digest(&encode(&changed[9]).unwrap());
                changed[10]
                    .root_generation
                    .as_mut()
                    .unwrap()
                    .predecessor_sha256 = sha;
            }
            "context" | "identity" => {
                let mut value: serde_json::Value =
                    serde_json::from_slice(&changed[10].root_generation.as_ref().unwrap().binding)
                        .unwrap();
                let mut scope: serde_json::Value = serde_json::from_slice(
                    &serde_json::from_value::<Vec<u8>>(value["scope"].clone()).unwrap(),
                )
                .unwrap();
                if case == "context" {
                    scope[5] = serde_json::json!("/different/HOME");
                } else {
                    let first: serde_json::Value =
                        serde_json::from_slice(&root_binding::scope_bytes(&original).unwrap())
                            .unwrap();
                    scope[8] = first[8].clone();
                }
                value["scope"] = serde_json::to_value(serde_json::to_vec(&scope).unwrap()).unwrap();
                changed[10].root_generation.as_mut().unwrap().binding =
                    serde_json::to_vec(&value).unwrap();
            }
            "phase" => {
                changed[11].state = State::Captured;
                changed[11].references = changed[12].references.clone();
            }
            "ticket" => changed[11].ticket = history[5].ticket.clone(),
            "copy-source" => changed[11].root_generation = changed[10].root_generation.clone(),
            _ => unreachable!(),
        }
        assert!(root_lineage::parse(&original, &changed).is_err(), "{case}");
    }
    let mut oversized = history;
    oversized.resize(4097, closed);
    assert!(root_lineage::parse(&original, &oversized).is_err());
}

#[test]
fn root_rollover_exact_slot_set_refuses_unknown_missing_linked_changed_or_unsafe_slots() {
    for case in ["unknown", "missing", "link", "intent", "permissions"] {
        let (_temp, context, index, closed) = fixture(2);
        let references = index.closed_root_history(&closed).unwrap();
        let generations = index
            .data_root()
            .join("manager-state/client-root-generations");
        let slot = generations.join(&closed.ticket);
        match case {
            "unknown" => fs::DirBuilder::new()
                .mode(0o700)
                .create(generations.join(uuid::Uuid::new_v4().to_string()))
                .unwrap(),
            "missing" => fs::rename(&slot, generations.join("retained-unindexed")).unwrap(),
            "link" => {
                fs::rename(&slot, generations.join("retained-unindexed")).unwrap();
                std::os::unix::fs::symlink(&context.opencodex_home, &slot).unwrap();
            }
            "intent" => {
                let p = slot.join("intent.json");
                fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
                fs::write(p, b"changed provenance").unwrap();
            }
            "permissions" => fs::set_permissions(&slot, fs::Permissions::from_mode(0o755)).unwrap(),
            _ => unreachable!(),
        }
        assert!(index.head().is_err(), "{case}");
        assert!(index.closed_root_history(&closed).is_err(), "{case}");
        // Every namespace mutation also refuses direct old-leaf consumption.
        if case != "intent" {
            assert!(
                super::super::super::root_retirement::open_closed_history(
                    index.data_root(),
                    &context.working_directory,
                    &references[0]
                )
                .is_err(),
                "{case}"
            );
        }
    }
}

#[test]
fn root_rollover_frame_prefix_rechecks_namespace_after_last_frame_observation() {
    let (_temp, context, index, closed) = fixture(2);
    let refs = index.closed_root_history(&closed).unwrap();
    let original = super::super::super::root_retirement::open_closed_history(
        index.data_root(),
        &context.working_directory,
        &refs[0],
    )
    .unwrap();
    let called = std::cell::Cell::new(false);
    let result = original.frames_through_checked(Phase::Original, &|_| {
        called.set(true);
        fs::DirBuilder::new()
            .mode(0o700)
            .create(
                index
                    .data_root()
                    .join("manager-state/client-root-generations")
                    .join(uuid::Uuid::new_v4().to_string()),
            )
            .unwrap();
        Ok(())
    });
    assert!(called.get());
    assert!(result.is_err());
    assert!(index.head().is_err());
}

#[test]
fn root_rollover_unknown_slot_at_pending_candidate_or_committed_boundary_never_becomes_registered()
{
    for point in [
        "generation_reserved",
        "generation_head_updated",
        "generation_committed",
    ] {
        let (_temp, context, index, closed) = fixture(2);
        let data = index.data_root().to_path_buf();
        let unknown = uuid::Uuid::new_v4().to_string();
        let result = index.create_successor_root_checked(
            &context,
            &closed,
            Instant::now() + Duration::from_secs(30),
            &|stage| {
                if stage == point {
                    fs::DirBuilder::new()
                        .mode(0o700)
                        .create(
                            data.join("manager-state/client-root-generations")
                                .join(&unknown),
                        )
                        .unwrap();
                }
                Ok(())
            },
        );
        assert!(result.is_err(), "{point}");
        assert!(data
            .join("manager-state/client-root-generations")
            .join(&unknown)
            .is_dir());
        assert!(
            Index::open_existing(&context, &data, &context.working_directory.join(".codex"))
                .is_err()
        );
        let uri = format!(
            "file:{}?immutable=1",
            data.join("manager-state/client-session-index.sqlite")
                .display()
        );
        let c = Connection::open_with_flags(
            uri,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        )
        .unwrap();
        let head = read_head(&c).unwrap().unwrap();
        assert_eq!(
            head.revision,
            if point == "generation_committed" {
                11
            } else {
                10
            }
        );
        let all = vault_history::read(&c, &head).unwrap();
        assert!(all.iter().all(|h| h.ticket != unknown));
        let count: u64 = c
            .query_row("SELECT count(*) FROM events", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, head.revision);
    }
}

fn raw_files(root: &Path, path: &Path, files: &mut Vec<serde_json::Value>) {
    for e in fs::read_dir(path).unwrap() {
        let p = e.unwrap().path();
        let meta = fs::symlink_metadata(&p).unwrap();
        assert!(!meta.file_type().is_symlink());
        if meta.is_dir() {
            raw_files(root, &p, files);
        } else {
            let raw = fs::read(&p).unwrap();
            files.push(serde_json::json!({"path":p.strip_prefix(root).unwrap(),"sha256":digest(&raw),"rawBase64":base64::Engine::encode(&base64::engine::general_purpose::STANDARD,raw)}));
        }
    }
}
#[test]
fn root_rollover_process_exit_preserves_third_generation_atomicity_without_recreating_custody() {
    const TEST:&str="infrastructure::macos_client_originals::session_index::generation::rollover_tests::root_rollover_process_exit_preserves_third_generation_atomicity_without_recreating_custody";
    if let (Some(root), Some(point)) = (
        std::env::var_os("OCXD024_ROLLOVER_ROOT"),
        std::env::var_os("OCXD024_ROLLOVER_POINT"),
    ) {
        let root = PathBuf::from(root);
        let point = point.to_str().unwrap();
        assert!(root.starts_with("/private/tmp"));
        assert!(root.ancestors().any(|p| p
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with("ocxd024-root-lineage-"))));
        let (context, index, closed) = fixture_at(root.clone(), 2);
        fs::write(root.join("predecessor.json"), encode(&closed).unwrap()).unwrap();
        let _ = index.create_successor_root_checked(
            &context,
            &closed,
            Instant::now() + Duration::from_secs(30),
            &|stage| {
                if stage == point {
                    unsafe { libc::_exit(91) }
                }
                Ok(())
            },
        );
        panic!("owned checkpoint was not reached");
    }
    for point in [
        "generation_published",
        "generation_head_updated",
        "generation_committed",
    ] {
        let temp = tempfile::Builder::new()
            .prefix("ocxd024-root-lineage-exit-")
            .tempdir()
            .unwrap();
        let root = temp.path().canonicalize().unwrap();
        let out = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", TEST, "--nocapture"])
            .env("OCXD024_ROLLOVER_ROOT", &root)
            .env("OCXD024_ROLLOVER_POINT", point)
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
        let data = root.join("desktop");
        let uri = format!(
            "file:{}?immutable=1",
            data.join("manager-state/client-session-index.sqlite")
                .display()
        );
        let c = Connection::open_with_flags(
            uri,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        )
        .unwrap();
        let head = read_head(&c).unwrap().unwrap();
        drop(c);
        let committed = point == "generation_committed";
        assert_eq!(head.revision, if committed { 11 } else { 10 });
        assert_eq!(
            head.state,
            if committed {
                State::Preparing
            } else {
                State::RootAbsent
            }
        );
        let context = ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(
            ),
            working_directory: root.join("profile"),
            opencodex_home: root.join("profile/home"),
        };
        let reopened = Index::open_existing(&context, &data, &root.join("profile/.codex"));
        if committed {
            let index = reopened.unwrap();
            assert_eq!(index.head().unwrap(), Some(head.clone()));
            assert!(index.reserve_successor_root_capture(&head).is_err());
            assert!(index.reserve_start().is_err());
        } else {
            assert!(reopened.is_err());
        }
        assert!(root.join("profile/.codex").is_dir());
        println!(
            "ROOT_ROLLOVER_EXIT_EVIDENCE {}",
            serde_json::json!({"accepted":false,"point":point,"exitCode":out.status.code(),"head":head,"committed":committed,"files":files,"rootRetained":true,"noRecreatedCustody":true})
        );
    }
}
