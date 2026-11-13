//! Synthetic Stopped/source publication only; no normal Stop or apply permit.
use super::*;
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    time::Instant,
};
fn fixture(second: bool) -> (tempfile::TempDir, ProcessContext, Index, Head, Originals) {
    let (temp, context, index, captured, originals) = if second {
        super::retirement_transaction_tests::fixture()
    } else {
        super::super::root_retirement::fixture_captured_generation()
    };
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
fn evidence() -> restore::Evidence {
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
