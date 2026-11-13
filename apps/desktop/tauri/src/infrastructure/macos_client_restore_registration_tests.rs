//! Synthetic Stopped/source publication only; no normal Stop or apply permit.
use super::*;
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    time::Instant,
};
pub(super) fn fixture(second: bool) -> (tempfile::TempDir, ProcessContext, Index, Head, Originals) {
    let captured = if second {
        super::retirement_transaction_tests::fixture()
    } else {
        super::super::root_retirement::fixture_captured_generation()
    };
    complete_stopped(captured)
}
fn complete_stopped(
    captured: (tempfile::TempDir, ProcessContext, Index, Head, Originals),
) -> (tempfile::TempDir, ProcessContext, Index, Head, Originals) {
    let (temp, context, index, captured, originals) = captured;
    let ready = originals.seal(Phase::Ready).unwrap();
    let raw=encode(&serde_json::json!({"schemaVersion":1,"scopeSha256":captured.references.as_ref().unwrap().scope_sha256,"readySha256":ready,"recordSha256":"a".repeat(64),"generation":[42,unsafe{libc::geteuid()},1,1],"parentGeneration":[41,unsafe{libc::geteuid()},1,1],"port":12345})).unwrap();
    super::super::create(&originals.directory, "startup-live.json", &raw).unwrap();
    let pre = originals.seal(Phase::PreStop).unwrap();
    let stopped = originals.seal(Phase::Stopped).unwrap();
    let phases = [
        (Phase::Ready, ready),
        (Phase::PreStop, pre),
        (Phase::Stopped, stopped),
    ];
    let mut head = captured;
    for (i, state) in [State::Ready, State::StopPrepared, State::Stopped]
        .into_iter()
        .enumerate()
    {
        head.revision += 1;
        head.state = state;
        let refs = head.references.as_mut().unwrap();
        refs.live_sha256 = Some(digest(&raw));
        refs.phases = phases[..=i].to_vec();
        // Source records for this storage fixture only. No typed native phase proof
        // or service result is constructed from these synthetic phase frames.
        let c = index.connection().unwrap();
        let doc = encode(&head).unwrap();
        c.execute(
            "INSERT INTO events(revision,document) VALUES(?1,?2)",
            params![head.revision, &doc],
        )
        .unwrap();
        c.execute(
            "UPDATE current_session SET revision=?1,document=?2 WHERE singleton=1",
            params![head.revision, &doc],
        )
        .unwrap();
    }
    assert_eq!(index.head().unwrap(), Some(head.clone()));
    (temp, context, index, head, originals)
}
/// Storage fixture with the same complete four-frame prior generation and
/// nine-event source shape as the original-service test. Synthetic exit and
/// movement seams grant no production authority or normal Stop success.
pub(super) fn full_history_fixture() -> (tempfile::TempDir, ProcessContext, Index, Head, Originals)
{
    let (temp, context, index, stopped, originals) = fixture(false);
    let artifacts = restore::Pending::prepare_fixture(&originals, evidence()).unwrap();
    let restored = index
        .record_restore_plan(&stopped, artifacts.id(), artifacts.sha256())
        .unwrap();
    let pending =
        super::super::root_retirement::Pending::prepare(&context, &originals, &restored).unwrap();
    let prepared = index
        .record_root_retirement(&restored, pending.id(), pending.sha256())
        .unwrap();
    let proof = pending
        .fixture_apply_for_index(Instant::now() + std::time::Duration::from_secs(10))
        .unwrap();
    let closed = index.close_root_absence(&prepared, &proof).unwrap();
    assert_eq!(closed.revision, 9);
    drop(pending);
    drop(artifacts);
    drop(originals);
    let (index, preparing) = index
        .create_successor_root(
            &context,
            &closed,
            Instant::now() + std::time::Duration::from_secs(30),
        )
        .unwrap();
    let (head, originals) = generation::fixture_captured_successor(&context, &index, &preparing);
    let result = complete_stopped((temp, context, index, head, originals));
    assert_eq!(result.3.revision, 15);
    result
}
pub(super) fn evidence() -> restore::Evidence {
    restore::Evidence {
        declared_routes: Some(vec![]),
        retained_pids: vec![100, 101],
        audit_home: "synthetic-root-restore".into(),
        not_before: chrono::DateTime::parse_from_rfc3339("2026-11-13T00:00:00Z")
            .unwrap()
            .into(),
        not_after: chrono::DateTime::parse_from_rfc3339("2026-11-13T00:00:02Z")
            .unwrap()
            .into(),
    }
}
fn next(source: &Head, id: &str, sha: &str) -> Head {
    let mut v = source.clone();
    v.revision += 1;
    v.state = State::RestorePrepared;
    v.restore_plan = Some(RestorePlan {
        id: id.into(),
        sha256: sha.into(),
    });
    v
}
#[test]
fn root_restore_registration_same_transaction_reduces_history_without_reducing_plan_admission() {
    let mut samples = Vec::new();
    for reference in [true, false, false, true] {
        let (_temp, _context, index, head, originals) = fixture(true);
        let pending = restore::Pending::prepare_fixture(&originals, evidence()).unwrap();
        let start = Instant::now();
        let (result, metrics) = observation_metrics::measure(|| {
            if reference {
                let source = index.registered_root_originals(&head).unwrap();
                let plan = restore::Pending::open(&source, pending.id(), pending.sha256()).unwrap();
                plan.require_indexed_references(head.references.as_ref().unwrap())
                    .unwrap();
                let next = next(&head, pending.id(), pending.sha256());
                index.write(Some(&head), &next).unwrap();
                next
            } else {
                index
                    .record_restore_plan(&head, pending.id(), pending.sha256())
                    .unwrap()
            }
        });
        let elapsed = start.elapsed().as_nanos();
        assert_eq!(index.head().unwrap(), Some(result));
        let metrics = serde_json::to_value(metrics).unwrap();
        assert_eq!(
            metrics["historical_begin"]["calls"],
            if reference { 4 } else { 3 }
        );
        assert_eq!(
            metrics["historical_full_pass"]["calls"],
            if reference { 8 } else { 6 }
        );
        samples.push(
            serde_json::json!({"reference":reference,"elapsedNanos":elapsed,"metrics":metrics}),
        );
    }
    println!(
        "RESTORE_REGISTRATION_COMPARISON {}",
        serde_json::json!({"accepted":false,"samples":samples,"scope":"synthetic second-generation Stopped and plan reference publication; no service or apply permission"})
    );
}
#[test]
fn root_restore_registration_first_generation_and_invalid_source_or_plan_before_sql() {
    for mode in [
        "valid",
        "stale",
        "ready",
        "rebound-live",
        "partial-phases",
        "digest",
        "missing-plan",
        "admission-interrupted",
    ] {
        let (_temp, _context, index, head, originals) = fixture(false);
        let pending = restore::Pending::prepare_fixture(&originals, evidence()).unwrap();
        let mut source = head.clone();
        if mode == "stale" {
            source.revision += 1;
        }
        if mode == "ready" {
            source.state = State::Ready;
        }
        if mode == "rebound-live" {
            source.references.as_mut().unwrap().live_sha256 = Some("b".repeat(64));
        }
        if mode == "partial-phases" {
            source.references.as_mut().unwrap().phases.pop();
        }
        let sha = if mode == "digest" {
            "a".repeat(64)
        } else {
            pending.sha256().into()
        };
        if mode == "missing-plan" {
            fs::remove_file(
                originals
                    .directory
                    .path()
                    .join(format!("restore-{}.json", pending.id())),
            )
            .unwrap();
        }
        let admitted = std::cell::Cell::new(false);
        let sql = std::cell::Cell::new(false);
        let result = index.record_restore_plan_checked(&source, pending.id(), &sha, &|point| {
            if point == "source_admitted" {
                admitted.set(true);
                if mode == "admission-interrupted" {
                    return Err(denied("injected source interruption"));
                }
            }
            if point == "references_sql_written" {
                sql.set(true);
            }
            Ok(())
        });
        assert_eq!(result.is_ok(), mode == "valid", "{mode}");
        assert_eq!(sql.get(), mode == "valid");
        assert_eq!(
            admitted.get(),
            mode == "valid" || mode == "admission-interrupted"
        );
        let current = read_head(index.connection().unwrap()).unwrap().unwrap();
        assert_eq!(current.revision, head.revision + u64::from(mode == "valid"));
        assert_eq!(
            current.state,
            if mode == "valid" {
                State::RestorePrepared
            } else {
                State::Stopped
            }
        );
    }
}
#[test]
fn root_restore_registration_does_not_certify_later_plan_bytes_or_application() {
    let (_temp, _context, index, head, originals) = fixture(false);
    let pending = restore::Pending::prepare_fixture(&originals, evidence()).unwrap();
    let frames: Vec<_> = [
        Phase::Original,
        Phase::Ready,
        Phase::PreStop,
        Phase::Stopped,
    ]
    .into_iter()
    .map(|phase| {
        let path = originals.directory.path().join(phase.filename());
        let bytes = fs::read(&path).unwrap();
        (path, bytes)
    })
    .collect();
    let prepared = index
        .record_restore_plan(&head, pending.id(), pending.sha256())
        .unwrap();
    assert_eq!(prepared.state, State::RestorePrepared);
    assert!(
        prepared.restore_plan
            == Some(RestorePlan {
                id: pending.id().into(),
                sha256: pending.sha256().into(),
            })
    );
    let path = originals
        .directory
        .path()
        .join(format!("restore-{}.json", pending.id()));
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&path, b"changed after successful reference publication").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(restore::Pending::open(&originals, pending.id(), pending.sha256()).is_err());
    for (path, original) in frames {
        assert_eq!(fs::read(path).unwrap(), original);
    }
    assert_eq!(
        fs::read(path).unwrap(),
        b"changed after successful reference publication"
    );
}
#[test]
fn root_restore_registration_candidate_and_postcommit_source_failures_preserve_exact_phase() {
    for stage in [
        "source_admitted",
        "references_sql_written",
        "references_committed",
    ] {
        for mutation in ["old-frame", "current-frame", "namespace"] {
            let (_temp, _context, index, head, originals) = fixture(true);
            let pending = restore::Pending::prepare_fixture(&originals, evidence()).unwrap();
            let reached = std::cell::Cell::new(false);
            let result = index.record_restore_plan_checked(
                &head,
                pending.id(),
                pending.sha256(),
                &|point| {
                    if point == stage {
                        reached.set(true);
                        match mutation {
                            "current-frame" => {
                                let p = originals.directory.path().join(Phase::Ready.filename());
                                fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
                                fs::write(&p, b"changed current frame").unwrap();
                                fs::set_permissions(&p, fs::Permissions::from_mode(0o400)).unwrap();
                            }
                            "old-frame" => {
                                let raw: Vec<u8> = index
                                    .connection()
                                    .unwrap()
                                    .query_row(
                                        "SELECT document FROM events WHERE revision=1",
                                        [],
                                        |r| r.get(0),
                                    )
                                    .unwrap();
                                let old: Head = decode(&raw).unwrap();
                                let p = index
                                    .data_root()
                                    .join("manager-state/client-originals")
                                    .join(old.ticket)
                                    .join("original.json");
                                fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
                                fs::write(&p, b"changed old frame").unwrap();
                                fs::set_permissions(&p, fs::Permissions::from_mode(0o400)).unwrap();
                            }
                            "namespace" => {
                                fs::DirBuilder::new()
                                    .mode(0o700)
                                    .create(
                                        index
                                            .data_root()
                                            .join("manager-state/client-root-generations")
                                            .join(uuid::Uuid::new_v4().to_string()),
                                    )
                                    .unwrap();
                            }
                            _ => unreachable!(),
                        }
                    }
                    Ok(())
                },
            );
            assert!(reached.get());
            assert!(result.is_err(), "{stage}/{mutation}");
            let current = read_head(index.connection().unwrap()).unwrap().unwrap();
            assert_eq!(
                current.revision,
                head.revision + u64::from(stage == "references_committed")
            );
            assert_eq!(
                current.state,
                if stage == "references_committed" {
                    State::RestorePrepared
                } else {
                    State::Stopped
                }
            );
        }
    }
}
#[test]
fn root_metadata_registration_keeps_every_dynamic_source_and_archive_check() {
    let mut samples = Vec::new();
    for reference in [true, false, false, true] {
        let (_temp, _context, index, head, originals) = fixture(true);
        let pending = restore::Pending::prepare_fixture(&originals, evidence()).unwrap();
        let start = Instant::now();
        let (result, metrics) = observation_metrics::measure(|| {
            let action = || index.record_restore_plan(&head, pending.id(), pending.sha256());
            if reference {
                observation_metrics::root_metadata_reference(action)
            } else {
                action()
            }
        });
        let prepared = result.unwrap();
        let elapsed = start.elapsed().as_nanos();
        assert_eq!(index.head().unwrap(), Some(prepared));
        samples.push(
            serde_json::json!({"reference":reference,"elapsedNanos":elapsed,"metrics":metrics}),
        );
    }
    for metric in [
        "frame",
        "frame_archives",
        "capture_file",
        "root_provenance",
        "root_namespace",
        "historical_begin",
        "historical_finish",
        "historical_full_pass",
    ] {
        let expected = &samples[0]["metrics"][metric]["calls"];
        assert!(expected.as_u64().unwrap() > 0);
        for sample in &samples {
            assert_eq!(&sample["metrics"][metric]["calls"], expected, "{metric}");
        }
    }
    let old = samples[0]["metrics"]["protection"]["calls"]
        .as_u64()
        .unwrap();
    let new = samples[1]["metrics"]["protection"]["calls"]
        .as_u64()
        .unwrap();
    assert!(new < old);
    println!(
        "ROOT_METADATA_REGISTRATION {}",
        serde_json::json!({"accepted":false,"samples":samples,"scope":"synthetic same second generation; no provenance/namespace/frame/archive observations removed; no native service"})
    );
}
#[test]
fn bounded_xattr_full_history_registration_keeps_48_frame_authentications() {
    let mut samples = Vec::new();
    for reference in [true, false, false, true] {
        let (_temp, context, index, stopped, originals) = full_history_fixture();
        let old: Vec<u8> = index
            .connection()
            .unwrap()
            .query_row("SELECT document FROM events WHERE revision=9", [], |r| {
                r.get(0)
            })
            .unwrap();
        let closed: Head = decode(&old).unwrap();
        assert_eq!(closed.state, State::RootAbsent);
        assert_eq!(closed.references.as_ref().unwrap().phases.len(), 3);
        let artifacts = restore::Pending::prepare_fixture(&originals, evidence()).unwrap();
        let head = index
            .record_restore_plan(&stopped, artifacts.id(), artifacts.sha256())
            .unwrap();
        assert_eq!(head.revision, 16);
        let pending =
            super::super::root_retirement::Pending::prepare(&context, &originals, &head).unwrap();
        let start = Instant::now();
        let (result, metrics) = observation_metrics::measure(|| {
            let register =
                || index.record_root_retirement_registered(&head, pending.id(), pending.sha256());
            if reference {
                observation_metrics::xattr_reference(register)
            } else {
                register()
            }
        });
        let elapsed = start.elapsed().as_nanos();
        let (prepared, registration) = result.unwrap();
        assert_eq!(prepared.revision, 17);
        registration
            .require(&head, registration.plan().unwrap())
            .unwrap();
        assert_eq!(index.head().unwrap(), Some(prepared));
        let metrics = serde_json::to_value(metrics).unwrap();
        assert_eq!(metrics["frame"]["calls"], 48);
        assert_eq!(metrics["frame_archives"]["calls"], 48);
        assert_eq!(metrics["historical_begin"]["calls"], 3);
        assert_eq!(metrics["historical_finish"]["calls"], 3);
        assert_eq!(metrics["historical_full_pass"]["calls"], 6);
        samples.push(
            serde_json::json!({"reference":reference,"elapsedNanos":elapsed,"metrics":metrics}),
        );
    }
    for key in [
        "frame",
        "frame_archives",
        "capture_file",
        "root_provenance",
        "root_namespace",
        "protection",
        "historical_begin",
        "historical_finish",
        "historical_full_pass",
    ] {
        for sample in &samples {
            assert_eq!(
                sample["metrics"][key]["calls"], samples[0]["metrics"][key]["calls"],
                "{key}"
            );
        }
    }
    assert!(
        samples[1]["metrics"]["xattr_native_call"]["calls"]
            .as_u64()
            .unwrap()
            < samples[0]["metrics"]["xattr_native_call"]["calls"]
                .as_u64()
                .unwrap()
    );
    println!(
        "XATTR_FULL_HISTORY_COMPARISON {}",
        serde_json::json!({"accepted":false,"samples":samples,"scope":"complete four-frame first closed history; exact 9/16/17 index shapes and 48 frame checks; synthetic trees, no original runtime or Stop budget success"})
    );
}
#[test]
fn private_children_generation_scan_keeps_all_complete_source_checks() {
    let mut rows = Vec::new();
    for reference in [true, false, false, true] {
        let (_tmp, context, index, stopped, originals) = full_history_fixture();
        let artifact = restore::Pending::prepare_fixture(&originals, evidence()).unwrap();
        let source = index
            .record_restore_plan(&stopped, artifact.id(), artifact.sha256())
            .unwrap();
        let pending =
            super::super::root_retirement::Pending::prepare(&context, &originals, &source).unwrap();
        let start = Instant::now();
        let (result, metrics) = observation_metrics::measure(|| {
            let register =
                || index.record_root_retirement_registered(&source, pending.id(), pending.sha256());
            if reference {
                observation_metrics::generation_directories_reference(register)
            } else {
                register()
            }
        });
        let elapsed = start.elapsed().as_nanos();
        let (head, proof) = result.unwrap();
        proof.require(&source, proof.plan().unwrap()).unwrap();
        assert_eq!(index.head().unwrap(), Some(head));
        let metrics = serde_json::to_value(metrics).unwrap();
        assert_eq!(metrics["frame"]["calls"], 48);
        assert_eq!(metrics["frame_archives"]["calls"], 48);
        assert_eq!(metrics["historical_begin"]["calls"], 3);
        assert_eq!(metrics["historical_finish"]["calls"], 3);
        assert_eq!(metrics["historical_full_pass"]["calls"], 6);
        rows.push(
            serde_json::json!({"reference":reference,"elapsedNanos":elapsed,"metrics":metrics}),
        );
    }
    for key in [
        "frame",
        "frame_archives",
        "capture_file",
        "root_provenance",
        "root_namespace",
        "historical_begin",
        "historical_finish",
        "historical_full_pass",
    ] {
        for row in &rows {
            assert_eq!(
                row["metrics"][key]["calls"], rows[0]["metrics"][key]["calls"],
                "{key}"
            );
        }
    }
    assert!(
        rows[1]["metrics"]["protection"]["calls"].as_u64().unwrap()
            < rows[0]["metrics"]["protection"]["calls"].as_u64().unwrap()
    );
    for key in ["protection", "xattr_native_call"] {
        assert_eq!(
            rows[0]["metrics"][key]["calls"],
            rows[3]["metrics"][key]["calls"]
        );
        assert_eq!(
            rows[1]["metrics"][key]["calls"],
            rows[2]["metrics"][key]["calls"]
        );
    }
    println!(
        "PRIVATE_CHILDREN_FULL_HISTORY_COMPARISON {}",
        serde_json::json!({"accepted":false,"samples":rows,"scope":"complete four-frame prior generation and all original48frame/source gates; synthetic namespace metadata only, no official service"})
    );
}
