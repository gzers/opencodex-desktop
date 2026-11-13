//! Historical facts only. Synthetic storage close does not certify a service,
//! create a new generation or grant Start/Stop/movement permission.
use super::*;
use std::{
    fs,
    os::unix::fs::{symlink, DirBuilderExt, PermissionsExt},
    time::Duration,
};
use tests::Fixture;
fn close(f: &Fixture) -> ClosedRootReferences {
    let pending = f.prepare();
    let prepared = f
        ._index
        .record_root_retirement(&f.source, pending.id(), pending.sha256())
        .unwrap();
    let proof = pending
        .fixture_apply_for_index(Instant::now() + Duration::from_secs(10))
        .unwrap();
    let closed = f._index.close_root_absence(&prepared, &proof).unwrap();
    ClosedRootReferences {
        source: f.source.clone(),
        closed,
        original_root: f.originals.original_root_absence.clone().unwrap(),
    }
}
fn observe(f: &Fixture, refs: &ClosedRootReferences) -> io::Result<Originals> {
    open_closed_history(f.originals.data.path(), f.originals.profile.path(), refs)
}
fn inventory(root: &Path) -> Vec<serde_json::Value> {
    tests::storage_evidence(root)
}
#[test]
fn closed_plan_observation_retains_exact_start_and_end_authentication_without_cross_call_cache() {
    let f = Fixture::new();
    let refs = close(&f);
    let (result, metrics) = observation_metrics::measure(|| observe(&f, &refs));
    result.unwrap();
    assert_eq!(
        serde_json::to_value(metrics).unwrap()["retirement_plan_load"]["calls"],
        2
    );
    let movement = refs.closed.root_retirement.as_ref().unwrap();
    let path = f
        .originals
        .directory
        .path()
        .join(format!("root-retirement-{}.json", movement.plan.id));
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&path, b"later plan mutation").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(observe(&f, &refs).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"later plan mutation");
}
#[test]
fn closed_plan_observation_cannot_return_partial_shell_or_adopt_mismatched_independent_source() {
    for case in [
        "source-scope",
        "source-revision",
        "digest",
        "invalid-id",
        "receipt",
    ] {
        let f = Fixture::new();
        let mut refs = close(&f);
        match case {
            "source-scope" => {
                refs.source.references.as_mut().unwrap().scope_sha256 = "f".repeat(64);
                refs.closed.references = refs.source.references.clone();
            }
            "source-revision" => {
                refs.source.revision -= 1;
                refs.closed
                    .root_retirement
                    .as_mut()
                    .unwrap()
                    .source_revision = refs.source.revision;
            }
            "digest" => refs.closed.root_retirement.as_mut().unwrap().plan.sha256 = "f".repeat(64),
            "invalid-id" => {
                refs.closed.root_retirement.as_mut().unwrap().plan.id = "../outside".into()
            }
            "receipt" => refs.closed.completion_sha256 = Some("f".repeat(64)),
            _ => unreachable!(),
        }
        let before = inventory(f.originals.data.path());
        let error = observe(&f, &refs)
            .err()
            .expect("mismatched independent source must refuse");
        if case.starts_with("source-") {
            assert_eq!(
                error.to_string(),
                "root archive source Head differs from independent reference"
            );
        }
        assert_eq!(inventory(f.originals.data.path()), before);
    }
}
#[test]
fn closed_plan_observation_reloads_plan_after_real_readback_checkpoint() {
    for case in ["bytes", "permission", "replacement-unsealed"] {
        let f = Fixture::new();
        let refs = close(&f);
        let archive = observe(&f, &refs).unwrap();
        let movement = refs.closed.root_retirement.as_ref().unwrap();
        let (pending, initial) =
            Pending::open_with_plan(&archive, &movement.plan.id, &movement.plan.sha256).unwrap();
        let path = archive
            .directory
            .path()
            .join(format!("root-retirement-{}.json", movement.plan.id));
        let reached = std::cell::Cell::new(false);
        let result = pending.verify_historical_close_from_initial(
            refs.closed.completion_sha256.as_deref().unwrap(),
            initial,
            &|point| {
                assert_eq!(point, "historical-readback");
                reached.set(true);
                match case {
                    "bytes" => {
                        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
                        fs::write(&path, b"changed during observation").unwrap();
                        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
                    }
                    "permission" => {
                        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap()
                    }
                    "replacement-unsealed" => {
                        let raw = fs::read(&path).unwrap();
                        fs::rename(&path, archive.directory.path().join("retained-plan")).unwrap();
                        fs::write(&path, raw).unwrap();
                        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
                    }
                    _ => unreachable!(),
                }
                Ok(())
            },
        );
        assert!(reached.get());
        assert!(result.is_err(), "{case}");
    }
}
#[test]
fn closed_root_history_uses_exact_archive_without_following_new_logical_root_or_current_grok() {
    let f = Fixture::new();
    let refs = close(&f);
    let original = observe(&f, &refs).unwrap();
    let archive = original.client.path().to_path_buf();
    let initial = inventory(f.originals.data.path());
    assert_eq!(f._index.closed_root_history(&refs.closed).unwrap().len(), 1);
    assert_eq!(inventory(f.originals.data.path()), initial);
    let live = f.originals.profile.path().join(".codex");
    fs::DirBuilder::new().mode(0o700).create(&live).unwrap();
    fs::write(live.join("auth.json"), b"new synthetic user root").unwrap();
    let grok = f.originals.profile.path().join(".grok");
    fs::DirBuilder::new().mode(0o700).create(&grok).unwrap();
    fs::write(grok.join("config.toml"), b"new current Grok setting").unwrap();
    let before = inventory(f.originals.data.path());
    let retained = observe(&f, &refs).unwrap();
    assert_eq!(retained.client.path(), archive);
    assert_eq!(retained.client.identity(), original.client.identity());
    assert_eq!(f._index.closed_root_history(&refs.closed).unwrap().len(), 1);
    assert!(
        open_archived_originals(
            f.originals.data.path(),
            f.originals.profile.path(),
            &ArchiveReferences {
                source: refs.source.clone(),
                original_root: refs.original_root.clone(),
                plan_id: refs
                    .closed
                    .root_retirement
                    .as_ref()
                    .unwrap()
                    .plan
                    .id
                    .clone(),
                plan_sha256: refs
                    .closed
                    .root_retirement
                    .as_ref()
                    .unwrap()
                    .plan
                    .sha256
                    .clone()
            }
        )
        .is_err(),
        "CURRENT verification still requires actual root absence"
    );
    assert!(f._index.head().is_err());
    assert!(f._index.reserve_start().is_err());
    assert_eq!(
        fs::read(live.join("auth.json")).unwrap(),
        b"new synthetic user root"
    );
    assert_eq!(
        fs::read(grok.join("config.toml")).unwrap(),
        b"new current Grok setting"
    );
    assert_eq!(inventory(f.originals.data.path()), before);
    println!(
        "CLOSED_ROOT_HISTORY_EVIDENCE {}",
        serde_json::json!({"accepted":false,"source":refs.source,"closed":refs.closed,"origin":refs.original_root,"sameArchiveIdentity":retained.client.identity(),"readonlyHistoryPassesWithNewCurrentRoots":true,"currentAbsenceAndStartStillRefuse":true,"storageOnlySyntheticClose":true,"files":inventory(f.originals.data.path().parent().unwrap())})
    );
}
#[test]
fn closed_root_history_checks_archived_grok_bytes_instead_of_adopting_later_grok_configuration() {
    let f = Fixture::new_with_grok(b"old synthetic Grok original");
    let refs = close(&f);
    let archive = observe(&f, &refs).unwrap();
    assert_eq!(
        &*archive
            .archived_grok_file(Phase::Original, "config.toml")
            .unwrap(),
        b"old synthetic Grok original"
    );
    let current = f.originals.profile.path().join(".grok/config.toml");
    fs::write(&current, b"later configuration").unwrap();
    observe(&f, &refs).unwrap();
    f._index.closed_root_history(&refs.closed).unwrap();
    let blob = archive.directory.path().join(format!(
        "tree-content-{}.blob",
        digest(b"old synthetic Grok original")
    ));
    let original = fs::read(&blob).unwrap();
    fs::set_permissions(&blob, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&blob, b"tampered historical source").unwrap();
    fs::set_permissions(&blob, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(observe(&f, &refs).is_err());
    assert!(f._index.closed_root_history(&refs.closed).is_err());
    assert_eq!(fs::read(&current).unwrap(), b"later configuration");
    assert_eq!(fs::read(&blob).unwrap(), b"tampered historical source");
    fs::set_permissions(&blob, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&blob, original).unwrap();
    fs::set_permissions(&blob, fs::Permissions::from_mode(0o400)).unwrap();
    observe(&f, &refs).unwrap();
}
#[test]
fn closed_root_history_never_treats_serialized_prepared_or_wrong_source_plan_completion_as_closed()
{
    let f = Fixture::new();
    let refs = close(&f);
    observe(&f, &refs).unwrap();
    for case in [
        "prepared",
        "source-revision",
        "source-state",
        "completion",
        "plan",
        "origin",
    ] {
        let mut wrong = refs.clone();
        match case {
            "prepared" => {
                wrong.closed.state = session_index::State::RootRetirementPrepared;
                wrong.closed.completion_sha256 = None;
            }
            "source-revision" => wrong.source.revision += 1,
            "source-state" => wrong.source.state = session_index::State::Stopped,
            "completion" => wrong.closed.completion_sha256 = Some("a".repeat(64)),
            "plan" => wrong.closed.root_retirement.as_mut().unwrap().plan.sha256 = "a".repeat(64),
            "origin" => wrong.original_root.intent_sha256 = "a".repeat(64),
            _ => unreachable!(),
        }
        assert!(
            observe(&f, &wrong).is_err(),
            "accepted wrong historical {case}"
        );
    }
    assert!(f
        ._index
        .closed_root_history(&{
            let mut old = refs.closed.clone();
            old.revision += 1;
            old
        })
        .is_err());
    assert!(
        f._index.reserve_start().is_err(),
        "read-only history never creates successor custody"
    );
}
#[test]
fn closed_root_history_refuses_archive_or_intent_readback_frame_changes_without_repair() {
    for target in ["archive-extra", "intent", "receipt", "original", "scope"] {
        let f = Fixture::new();
        let refs = close(&f);
        let archive = observe(&f, &refs).unwrap();
        let plan = &refs.closed.root_retirement.as_ref().unwrap().plan;
        if target == "archive-extra" {
            let path = archive.client.path().join("outside");
            fs::write(&path, b"preserve external archive addition").unwrap();
            assert!(observe(&f, &refs).is_err());
            assert_eq!(
                fs::read(path).unwrap(),
                b"preserve external archive addition"
            );
            continue;
        }
        let filename = match target {
            "intent" => format!("root-retirement-{}-Retire-intent.json", plan.id),
            "receipt" => format!("root-retirement-{}-Retire-readback.json", plan.id),
            "original" => "original.json".into(),
            "scope" => "startup-scope.json".into(),
            _ => unreachable!(),
        };
        let path = archive.directory.path().join(filename);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, b"{}").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        assert!(observe(&f, &refs).is_err());
        assert!(f._index.closed_root_history(&refs.closed).is_err());
        assert_eq!(fs::read(path).unwrap(), b"{}");
    }
}
#[test]
fn closed_root_history_index_audit_refuses_unindexed_vault_or_rewritten_source_event() {
    let f = Fixture::new();
    let refs = close(&f);
    f._index.closed_root_history(&refs.closed).unwrap();
    let unexpected = f
        .originals
        .data
        .path()
        .join("manager-state/client-originals/unregistered");
    fs::DirBuilder::new()
        .mode(0o700)
        .create(&unexpected)
        .unwrap();
    fs::write(unexpected.join("outside"), b"unindexed history").unwrap();
    assert!(f._index.closed_root_history(&refs.closed).is_err());
    assert_eq!(
        fs::read(unexpected.join("outside")).unwrap(),
        b"unindexed history"
    );
    fs::remove_file(unexpected.join("outside")).unwrap();
    fs::remove_dir(unexpected).unwrap();
    let connection = f._index.fixture_existing_connection().unwrap();
    let mut rewritten = refs.source.clone();
    rewritten.state = session_index::State::Stopped;
    let raw = encode(&rewritten).unwrap();
    connection
        .execute(
            "UPDATE events SET document=?1 WHERE revision=?2",
            rusqlite::params![raw, rewritten.revision],
        )
        .unwrap();
    assert!(f._index.closed_root_history(&refs.closed).is_err());
    let saved: Vec<u8> = connection
        .query_row(
            "SELECT document FROM events WHERE revision=?1",
            [rewritten.revision],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        saved, raw,
        "history audit cannot repair rewritten independent facts"
    );
}
#[test]
fn closed_root_history_readonly_audit_does_not_follow_a_new_user_symlink() {
    let f = Fixture::new();
    let refs = close(&f);
    let foreign = tempfile::tempdir().unwrap();
    fs::write(foreign.path().join("outside"), b"untouched foreign path").unwrap();
    let logical = f.originals.profile.path().join(".codex");
    symlink(foreign.path(), &logical).unwrap();
    observe(&f, &refs).unwrap();
    f._index.closed_root_history(&refs.closed).unwrap();
    assert!(f._index.head().is_err());
    assert!(f._index.reserve_start().is_err());
    assert!(fs::symlink_metadata(&logical)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(
        fs::read(foreign.path().join("outside")).unwrap(),
        b"untouched foreign path"
    );
}
#[test]
fn closed_root_history_rechecks_exact_archive_and_receipts_after_projection() {
    let f = Fixture::new();
    let refs = close(&f);
    let archive = observe(&f, &refs).unwrap();
    let movement = refs.closed.root_retirement.as_ref().unwrap();
    let pending = Pending::open(&archive, &movement.plan.id, &movement.plan.sha256).unwrap();
    let once = std::cell::Cell::new(false);
    let changed = archive.client.path().join("late-user-file");
    assert!(pending
        .verify_historical_close_checked(
            refs.closed.completion_sha256.as_deref().unwrap(),
            &|phase| {
                assert_eq!(phase, "historical-readback");
                assert!(!once.replace(true));
                fs::write(&changed, b"created during history audit").unwrap();
                Ok(())
            }
        )
        .is_err());
    assert!(once.get());
    assert_eq!(fs::read(&changed).unwrap(), b"created during history audit");
    assert!(f._index.closed_root_history(&refs.closed).is_err());
}
#[test]
fn closed_root_history_same_empty_tree_new_directory_inode_is_not_the_retained_original() {
    let f = Fixture::new();
    let refs = close(&f);
    let archive = observe(&f, &refs).unwrap();
    let path = archive.client.path().to_path_buf();
    assert!(
        fs::read_dir(&path).unwrap().next().is_none(),
        "fixture must isolate identity from content mismatch"
    );
    let retained = f.originals.profile.path().join("retained-original-archive");
    fs::rename(&path, &retained).unwrap();
    fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
    assert_ne!(
        PinnedDirectory::open(&path).unwrap().identity(),
        archive.client.identity()
    );
    assert!(observe(&f, &refs).is_err());
    assert!(f._index.closed_root_history(&refs.closed).is_err());
    assert!(retained.is_dir() && path.is_dir());
    assert!(fs::read_dir(&path).unwrap().next().is_none());
}
#[test]
fn closed_root_history_rejects_no_reference_unclosed_event_or_unregistered_generation() {
    for kind in [
        session_index::State::Preparing,
        session_index::State::CaptureReserved,
        session_index::State::Failed,
    ] {
        for new_ticket in [false, true] {
            let f = Fixture::new();
            let refs = close(&f);
            f._index.closed_root_history(&refs.closed).unwrap();
            let head = Head {
                revision: refs.closed.revision + 1,
                ticket: if new_ticket {
                    uuid::Uuid::new_v4().to_string()
                } else {
                    refs.closed.ticket.clone()
                },
                state: kind,
                references: None,
                restore_plan: None,
                completion_sha256: None,
                root_retirement: None,
                root_generation: None,
            };
            let raw = encode(&head).unwrap();
            let connection = f._index.fixture_existing_connection().unwrap();
            connection
                .execute(
                    "INSERT INTO events(revision,document) VALUES(?1,?2)",
                    rusqlite::params![head.revision, &raw],
                )
                .unwrap();
            connection
                .execute(
                    "UPDATE current_session SET revision=?1,document=?2",
                    rusqlite::params![head.revision, &raw],
                )
                .unwrap();
            assert!(
                f._index.closed_root_history(&head).is_err(),
                "unclosed reference-free event cannot disappear from history review"
            );
        }
    }
}
#[test]
fn closed_root_history_checks_initial_and_preclose_plan_registration_not_only_terminal_receipt() {
    let f = Fixture::new();
    let refs = close(&f);
    f._index.closed_root_history(&refs.closed).unwrap();
    let revision = refs.source.revision + 1;
    let connection = f._index.fixture_existing_connection().unwrap();
    let original: Vec<u8> = connection
        .query_row(
            "SELECT document FROM events WHERE revision=?1",
            [revision],
            |r| r.get(0),
        )
        .unwrap();
    let mut prepared: Head = serde_json::from_slice(&original).unwrap();
    prepared.root_retirement.as_mut().unwrap().plan.sha256 = "a".repeat(64);
    let raw = encode(&prepared).unwrap();
    connection
        .execute(
            "UPDATE events SET document=?1 WHERE revision=?2",
            rusqlite::params![raw, revision],
        )
        .unwrap();
    assert!(
        f._index.closed_root_history(&refs.closed).is_err(),
        "changed independent Prepared cannot be hidden behind original close+disk plan"
    );
    let retained: Vec<u8> = connection
        .query_row(
            "SELECT document FROM events WHERE revision=?1",
            [revision],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(retained, raw);
}

#[test]
fn closed_root_history_missing_provenance_plan_or_original_bytes_are_not_repaired() {
    for case in ["provenance", "plan", "original", "intent", "receipt"] {
        let f = Fixture::new();
        let refs = close(&f);
        let archive = observe(&f, &refs).unwrap();
        let id = &refs.closed.root_retirement.as_ref().unwrap().plan.id;
        let path = match case {
            "provenance" => f
                .originals
                .data
                .path()
                .join("manager-state/client-root-reservation/created.json"),
            "plan" => archive
                .directory
                .path()
                .join(format!("root-retirement-{id}.json")),
            "original" => archive.directory.path().join("original.json"),
            "intent" => archive
                .directory
                .path()
                .join(format!("root-retirement-{id}-Retire-intent.json")),
            "receipt" => archive
                .directory
                .path()
                .join(format!("root-retirement-{id}-Retire-readback.json")),
            _ => unreachable!(),
        };
        let retained = f.originals.profile.path().join("retained-evidence");
        fs::rename(&path, &retained).unwrap();
        assert!(observe(&f, &refs).is_err());
        assert!(f._index.closed_root_history(&refs.closed).is_err());
        assert!(!path.exists());
        assert!(retained.is_file());
    }
}
#[test]
fn closed_root_history_projection_refuses_receipt_bytes_or_access_changes() {
    for case in ["bytes", "mode"] {
        let f = Fixture::new();
        let refs = close(&f);
        let archive = observe(&f, &refs).unwrap();
        let plan = &refs.closed.root_retirement.as_ref().unwrap().plan;
        let pending = Pending::open(&archive, &plan.id, &plan.sha256).unwrap();
        let path = archive
            .directory
            .path()
            .join(format!("root-retirement-{}-Retire-readback.json", plan.id));
        let changed = std::cell::Cell::new(false);
        assert!(pending
            .verify_historical_close_checked(
                refs.closed.completion_sha256.as_deref().unwrap(),
                &|_| {
                    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
                    if case == "bytes" {
                        fs::write(&path, b"{}").unwrap();
                        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
                    }
                    changed.set(true);
                    Ok(())
                }
            )
            .is_err());
        assert!(changed.get());
        assert!(observe(&f, &refs).is_err());
        if case == "bytes" {
            assert_eq!(fs::read(&path).unwrap(), b"{}");
        } else {
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o7777,
                0o600
            );
        }
    }
}
#[test]
fn closed_root_history_index_projection_rechecks_binding_action_and_all_events() {
    for case in ["binding", "action", "event"] {
        let f = Fixture::new();
        let refs = close(&f);
        let changed = std::cell::Cell::new(false);
        assert!(f
            ._index
            .fixture_closed_root_history_checked(&refs.closed, &|| {
                match case {
                    "binding" => {
                        f._index
                            .fixture_existing_connection()
                            .unwrap()
                            .execute("UPDATE binding SET document=?1", [b"{}".as_slice()])
                            .unwrap();
                    }
                    "action" => {
                        let path = f
                            .originals
                            .data
                            .path()
                            .join("manager-state/client-session-action.lock");
                        fs::rename(&path, f.originals.profile.path().join("retained-action"))
                            .unwrap();
                        fs::write(&path, b"").unwrap();
                        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
                    }
                    "event" => {
                        let mut head = refs.source.clone();
                        head.ticket = uuid::Uuid::new_v4().to_string();
                        f._index
                            .fixture_existing_connection()
                            .unwrap()
                            .execute(
                                "UPDATE events SET document=?1 WHERE revision=?2",
                                rusqlite::params![encode(&head).unwrap(), head.revision],
                            )
                            .unwrap();
                    }
                    _ => unreachable!(),
                }
                changed.set(true);
                Ok(())
            })
            .is_err());
        assert!(changed.get());
    }
}

#[test]
fn closed_root_history_full_frame_live_and_released_marker_chain_stays_independently_bound() {
    // Explicit storage-only phase fixture: these PID/Ready values are NOT
    // authenticated runtime facts, and never enter the normal-Stop factory.
    let mut f = Fixture::new_with_grok(b"old Grok bytes for closed phase history");
    let ready = f.originals.seal(Phase::Ready).unwrap();
    let prestop = f.originals.seal(Phase::PreStop).unwrap();
    let stopped = f.originals.seal(Phase::Stopped).unwrap();
    let refs = f.source.references.as_ref().unwrap();
    let live=encode(&serde_json::json!({"schemaVersion":1,"scopeSha256":refs.scope_sha256,"readySha256":ready,"recordSha256":"a".repeat(64),"generation":[42,unsafe{libc::geteuid()},1,1],"parentGeneration":[41,unsafe{libc::geteuid()},1,1],"port":12345})).unwrap();
    create(&f.originals.directory, "startup-live.json", &live).unwrap();
    let live_sha = digest(&live);
    for (state, phases) in [
        (
            session_index::State::Ready,
            vec![(Phase::Ready, ready.clone())],
        ),
        (
            session_index::State::StopPrepared,
            vec![
                (Phase::Ready, ready.clone()),
                (Phase::PreStop, prestop.clone()),
            ],
        ),
        (
            session_index::State::Stopped,
            vec![
                (Phase::Ready, ready.clone()),
                (Phase::PreStop, prestop.clone()),
                (Phase::Stopped, stopped.clone()),
            ],
        ),
    ] {
        let mut head = f.source.clone();
        head.revision += 1;
        head.state = state;
        let refs = head.references.as_mut().unwrap();
        refs.phases = phases;
        refs.live_sha256 = Some(live_sha.clone());
        session_index::validate(&head).unwrap();
        let raw = encode(&head).unwrap();
        let c = f._index.fixture_existing_connection().unwrap();
        c.execute(
            "INSERT INTO events(revision,document) VALUES(?1,?2)",
            rusqlite::params![head.revision, &raw],
        )
        .unwrap();
        c.execute(
            "UPDATE current_session SET revision=?1,document=?2",
            rusqlite::params![head.revision, &raw],
        )
        .unwrap();
        f.source = head;
    }
    assert_eq!(f._index.head().unwrap(), Some(f.source.clone()));
    let lock = f.originals.client.path().join("config.toml.ocx-write.lock");
    fs::write(
        &lock,
        encode(&serde_json::json!({"pid":std::process::id(),"token":"1234567890abcdef"})).unwrap(),
    )
    .unwrap();
    fs::set_permissions(&lock, fs::Permissions::from_mode(0o600)).unwrap();
    let image = f.originals.client.config_write_lock_image().unwrap();
    let pending =
        Pending::prepare_with_marker(&f.context, &f.originals, &f.source, Some(image)).unwrap();
    let prepared = f
        ._index
        .record_root_retirement(&f.source, pending.id(), pending.sha256())
        .unwrap();
    let readback = pending
        .apply_checked_inner(
            Direction::Retire,
            &|| Ok(()),
            &|phase| {
                if phase == "settle_writer" {
                    fs::remove_file(
                        f.originals
                            .directory
                            .path()
                            .join(pending.slot())
                            .join("config.toml.ocx-write.lock"),
                    )
                    .unwrap();
                }
                Ok(())
            },
            true,
        )
        .unwrap();
    let proof = pending
        .authorized_readback(readback, Instant::now() + Duration::from_secs(10))
        .unwrap();
    let closed = f._index.close_root_absence(&prepared, &proof).unwrap();
    let known = ClosedRootReferences {
        source: f.source.clone(),
        closed: closed.clone(),
        original_root: f.originals.original_root_absence.clone().unwrap(),
    };
    let (observed, metrics) = observation_metrics::measure(|| observe(&f, &known));
    observed.unwrap();
    println!(
        "CLOSED_PREFIX_METRICS {}",
        serde_json::json!({"accepted":false,"scope":"synthetic full four-frame schema2 closed-history observation, no official runtime","metrics":metrics})
    );
    let mut comparison = Vec::new();
    for reference in [true, false, false, true] {
        let started = Instant::now();
        let (observed, metrics) = observation_metrics::measure(|| {
            if reference {
                observation_metrics::reference(|| observe(&f, &known))
            } else {
                observe(&f, &known)
            }
        });
        let returned = observed.unwrap();
        assert_eq!(returned.attempt(), known.original_root.id);
        let metrics = serde_json::to_value(metrics).unwrap();
        assert_eq!(metrics["frame"]["calls"], 8);
        assert_eq!(metrics["root_provenance"]["calls"], 10);
        comparison.push(serde_json::json!({"mode":if reference {"reference"}else{"scoped-union"},"elapsedNanos":started.elapsed().as_nanos(),"metrics":metrics}));
    }
    println!(
        "CLOSED_PREFIX_COMPARISON {}",
        serde_json::json!({"accepted":false,"samples":comparison,"scope":"same-fixture same-binary reference/scoped observations in reversed order; no OS service acceptance"})
    );
    let mut plans = Vec::new();
    for reference in [true, false, false, true] {
        let started = Instant::now();
        let (opened, metrics) = observation_metrics::measure(|| {
            if reference {
                observation_metrics::plan_reference(|| observe(&f, &known))
            } else {
                observe(&f, &known)
            }
        });
        opened.unwrap();
        let metrics = serde_json::to_value(metrics).unwrap();
        assert_eq!(
            metrics["retirement_plan_load"]["calls"],
            if reference { 6 } else { 2 }
        );
        assert_eq!(metrics["frame"]["calls"], 8);
        assert_eq!(metrics["root_provenance"]["calls"], 10);
        plans.push(serde_json::json!({"reference":reference,"elapsedNanos":started.elapsed().as_nanos(),"metrics":metrics}));
    }
    println!(
        "HISTORICAL_PLAN_COMPARISON {}",
        serde_json::json!({"accepted":false,"samples":plans,"scope":"same source/binary/current four-frame fixture; fresh start/end plans and unchanged eight frames; no service acceptance"})
    );
    f._index.closed_root_history(&closed).unwrap();
    fs::DirBuilder::new()
        .mode(0o700)
        .create(f.originals.profile.path().join(".codex"))
        .unwrap();
    fs::write(
        f.originals.profile.path().join(".grok/config.toml"),
        b"new Grok should not replace archived phase evidence",
    )
    .unwrap();
    observe(&f, &known).unwrap();
    assert_eq!(f._index.closed_root_history(&closed).unwrap().len(), 1);
    assert!(f._index.head().is_err());
    let raw = f.originals.directory.path().join("startup-live.json");
    fs::set_permissions(&raw, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&raw, b"{}").unwrap();
    fs::set_permissions(&raw, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(observe(&f, &known).is_err());
    assert!(f._index.closed_root_history(&closed).is_err());
}
