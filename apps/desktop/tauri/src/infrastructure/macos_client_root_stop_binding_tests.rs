//! Storage/source binding tests only. Synthetic Ready/Stopped frames and PID
//! values never construct RootRetirementAuthority or a production Stop proof.
use super::tests::Fixture;
use super::*;
use std::{fs, os::unix::fs::PermissionsExt, time::Duration};
struct RootFixture {
    f: Fixture,
    session: StartupSession,
    head: session_index::Head,
    _index: session_index::Index,
}
impl RootFixture {
    fn new() -> Self {
        let f = Fixture::new();
        fs::remove_dir(&f.client).unwrap();
        let published = root_creation::reserve(
            &f.data,
            &f.context.working_directory,
            Instant::now() + Duration::from_secs(10),
        )
        .unwrap()
        .publish()
        .unwrap();
        let (index, preparing) =
            session_index::Index::register_first_root(&f.context, &f.data, published).unwrap();
        let reserved = index.reserve_first_root_capture(&preparing).unwrap();
        let reservation = index.capture_reservation(&reserved).unwrap();
        let originals = Originals::begin_reserved_profile(
            &f.data,
            &f.client,
            &f.context.working_directory,
            &reservation,
        )
        .unwrap();
        let (program, selection) = source(&f.context).unwrap();
        let mut session =
            StartupSession::persist(&f.context, originals, program, selection).unwrap();
        let captured = index
            .record_first_root_originals(&f.context, &reserved, session.references().unwrap())
            .unwrap();
        drop(reservation);
        let ready = session.originals.seal(Phase::Ready).unwrap();
        session.originals.seal(Phase::PreStop).unwrap();
        session.originals.seal(Phase::Stopped).unwrap();
        let live = Live {
            schema_version: 1,
            scope_sha256: session.scope_sha256.clone(),
            ready_sha256: ready,
            record_sha256: "a".repeat(64),
            generation: [42, unsafe { libc::geteuid() }.into(), 1, 1],
            parent_generation: [41, unsafe { libc::geteuid() }.into(), 1, 1],
            port: 12345,
        };
        let raw = encode(&live).unwrap();
        create(&session.originals.directory, "startup-live.json", &raw).unwrap();
        session.live_sha256 = Some(digest(&raw));
        let head = session_index::Head {
            revision: captured.revision + 3,
            state: session_index::State::Stopped,
            references: Some(session.references().unwrap()),
            ..captured
        };
        Self {
            f,
            session,
            head,
            _index: index,
        }
    }
    fn verify(
        &self,
        plan: &session_index::RestorePlan,
        scope: &Scope,
        live: &Live,
    ) -> AppResult<()> {
        self.session
            .verify_root_stop_binding(&self.f.context, &self.head, plan, scope, live)
    }
}
fn synthetic_ready(second: bool) -> (Fixture, session_index::Index, session_index::Head) {
    let fixture = RootFixture::new();
    let RootFixture {
        f,
        session,
        head,
        _index: index,
    } = fixture;
    let current = index.head().unwrap().unwrap();
    let mut previous = current;
    for (state, count) in [
        (session_index::State::Ready, 1),
        (session_index::State::StopPrepared, 2),
        (session_index::State::Stopped, 3),
    ] {
        if !second && count > 1 {
            break;
        }
        let mut h = head.clone();
        h.revision = previous.revision + 1;
        h.state = state;
        h.references.as_mut().unwrap().phases.truncate(count);
        let c = index.fixture_existing_connection().unwrap();
        let raw = encode(&h).unwrap();
        c.execute(
            "INSERT INTO events(revision,document) VALUES(?1,?2)",
            rusqlite::params![h.revision, &raw],
        )
        .unwrap();
        c.execute(
            "UPDATE current_session SET revision=?1,document=?2 WHERE singleton=1",
            rusqlite::params![h.revision, &raw],
        )
        .unwrap();
        previous = h;
    }
    if !second {
        return (f, index, previous);
    }
    let pending =
        root_retirement::Pending::prepare(&f.context, &session.originals, &previous).unwrap();
    let prepared = index
        .record_root_retirement(&previous, pending.id(), pending.sha256())
        .unwrap();
    let proof = pending
        .fixture_apply_for_index(Instant::now() + Duration::from_secs(10))
        .unwrap();
    let closed = index.close_root_absence(&prepared, &proof).unwrap();
    drop(pending);
    drop(session);
    let (index, preparing) = index
        .create_successor_root(
            &f.context,
            &closed,
            Instant::now() + Duration::from_secs(30),
        )
        .unwrap();
    let reserved = index.reserve_successor_root_capture(&preparing).unwrap();
    let permit = index.capture_reservation(&reserved).unwrap();
    let originals = Originals::begin_reserved_profile(
        &f.data,
        &f.client,
        &f.context.working_directory,
        &permit,
    )
    .unwrap();
    let (program, selection) = source(&f.context).unwrap();
    let mut session = StartupSession::persist(&f.context, originals, program, selection).unwrap();
    let captured = index
        .record_successor_root_originals(&f.context, &reserved, session.references().unwrap())
        .unwrap();
    drop(permit);
    let ready = session.originals.seal(Phase::Ready).unwrap();
    let live = Live {
        schema_version: 1,
        scope_sha256: session.scope_sha256.clone(),
        ready_sha256: ready,
        record_sha256: "a".repeat(64),
        generation: [42, unsafe { libc::geteuid() }.into(), 1, 1],
        parent_generation: [41, unsafe { libc::geteuid() }.into(), 1, 1],
        port: 12345,
    };
    let raw = encode(&live).unwrap();
    create(&session.originals.directory, "startup-live.json", &raw).unwrap();
    session.live_sha256 = Some(digest(&raw));
    let head = session_index::Head {
        revision: captured.revision + 1,
        state: session_index::State::Ready,
        references: Some(session.references().unwrap()),
        ..captured
    };
    let raw = encode(&head).unwrap();
    let c = index.fixture_existing_connection().unwrap();
    c.execute(
        "INSERT INTO events(revision,document) VALUES(?1,?2)",
        rusqlite::params![head.revision, &raw],
    )
    .unwrap();
    c.execute(
        "UPDATE current_session SET revision=?1,document=?2 WHERE singleton=1",
        rusqlite::params![head.revision, &raw],
    )
    .unwrap();
    index.head().unwrap();
    (f, index, head)
}
#[test]
fn ready_acquisition_returns_same_authenticated_head_and_originals_without_repeated_historical_scans(
) {
    for second in [false, true] {
        let (f, index, head) = synthetic_ready(second);
        let oldrefs = index
            .registered_root_originals(&head)
            .unwrap()
            .original_sha256()
            .to_string();
        drop(index);
        let (opened, metrics) = observation_metrics::measure(|| {
            session_index::Index::open_ready_session(&f.context, &f.data, &f.client)
        });
        let (index, observed, session) = opened.unwrap();
        assert_eq!(observed, head);
        assert_eq!(session.originals.original_sha256(), oldrefs);
        assert!(session.references().unwrap() == head.references.clone().unwrap());
        assert!(session.live(&f.context).is_ok());
        assert!(index.reserve_start().is_err());
        drop(session);
        drop(index);
        let (reference, reference_metrics) = observation_metrics::measure(|| {
            let index = session_index::Index::open(&f.context, &f.data, &f.client).unwrap();
            let head = index.head().unwrap().unwrap();
            let session = StartupSession::open_indexed(&f.context, &index, &head).unwrap();
            (index, head, session)
        });
        assert_eq!(reference.1, head);
        drop(reference);
        let current = serde_json::to_value(metrics).unwrap();
        let before = serde_json::to_value(reference_metrics).unwrap();
        if second {
            assert_eq!(current["historical_begin"]["calls"], 2);
            assert_eq!(before["historical_begin"]["calls"], 4);
        }
        println!(
            "READY_ACQUISITION_COMPARISON {}",
            serde_json::json!({"accepted":false,"secondGeneration":second,"optimized":current,"reference":before,"head":head,"scope":"synthetic scope/live/frame binding only; no authenticated live OS service or Stop permission"})
        );
    }
}
#[test]
fn ready_acquisition_refuses_scope_live_frame_program_and_nonready_changes_without_repair() {
    for case in ["scope", "live", "frame", "program", "nonready"] {
        let (f, index, mut head) = synthetic_ready(false);
        let attempt = f
            .data
            .join("manager-state/client-originals")
            .join(&head.ticket);
        match case {
            "nonready" => {
                head.state = session_index::State::Failed;
                let raw = encode(&head).unwrap();
                let c = index.fixture_existing_connection().unwrap();
                c.execute(
                    "UPDATE events SET document=?1 WHERE revision=?2",
                    rusqlite::params![&raw, head.revision],
                )
                .unwrap();
                c.execute(
                    "UPDATE current_session SET document=?1 WHERE singleton=1",
                    [&raw],
                )
                .unwrap();
            }
            "program" => {
                let path = f.context.executable().unwrap();
                fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
                fs::write(path, b"changed executable").unwrap();
            }
            _ => {
                let name = match case {
                    "scope" => "startup-scope.json",
                    "live" => "startup-live.json",
                    _ => "ready.json",
                };
                let p = attempt.join(name);
                fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
                fs::write(&p, b"changed").unwrap();
                fs::set_permissions(&p, fs::Permissions::from_mode(0o400)).unwrap();
            }
        }
        drop(index);
        assert!(
            session_index::Index::open_ready_session(&f.context, &f.data, &f.client).is_err(),
            "{case}"
        );
    }
}
#[test]
fn ready_acquisition_final_audit_refuses_mutation_after_initial_transaction_admission() {
    for case in ["archive", "receipt", "binding", "event", "namespace"] {
        let (f, index, head) = synthetic_ready(true);
        let c = index.fixture_existing_connection().unwrap();
        let first_raw: Vec<u8> = c
            .query_row("SELECT document FROM events WHERE revision=1", [], |r| {
                r.get(0)
            })
            .unwrap();
        let first: session_index::Head = serde_json::from_slice(&first_raw).unwrap();
        let old = f
            .data
            .join("manager-state/client-originals")
            .join(&first.ticket);
        let closed = {
            let mut statement = c
                .prepare("SELECT document FROM events ORDER BY revision")
                .unwrap();
            let raw = statement
                .query_map([], |r| r.get::<_, Vec<u8>>(0))
                .unwrap()
                .map(|r| r.unwrap())
                .collect::<Vec<_>>();
            raw.into_iter()
                .map(|b| serde_json::from_slice::<session_index::Head>(&b).unwrap())
                .find(|h| h.state == session_index::State::RootAbsent)
                .unwrap()
        };
        drop(index);
        let reached = std::cell::Cell::new(false);
        let opened = session_index::Index::fixture_open_ready_checked(
            &f.context,
            &f.data,
            &f.client,
            &|stage| {
                if stage == "index_acquisition_committed" {
                    reached.set(true);
                    match case {
                        "archive" => {
                            let p = old.join("original.json");
                            fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
                            fs::write(&p, b"late old archive").unwrap();
                            fs::set_permissions(&p, fs::Permissions::from_mode(0o400)).unwrap();
                        }
                        "receipt" => {
                            let p = old.join(format!(
                                "root-retirement-{}-Retire-readback.json",
                                closed.root_retirement.as_ref().unwrap().plan.id
                            ));
                            fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
                            fs::write(&p, b"late old receipt").unwrap();
                            fs::set_permissions(&p, fs::Permissions::from_mode(0o400)).unwrap();
                        }
                        "namespace" => {
                            use std::os::unix::fs::DirBuilderExt;
                            fs::DirBuilder::new()
                                .mode(0o700)
                                .create(
                                    f.data
                                        .join("manager-state/client-root-generations")
                                        .join(uuid::Uuid::new_v4().to_string()),
                                )
                                .unwrap();
                        }
                        "binding" | "event" => {
                            // A different process may ignore the action flock; do
                            // not open/close another SQLite inode fd in the parent.
                            let python="import sqlite3,sys; c=sqlite3.connect(sys.argv[1]); c.execute(sys.argv[2]); c.commit(); c.close()";
                            let sql = if case == "binding" {
                                "UPDATE binding SET document=x'5b5d' WHERE singleton=1".to_string()
                            } else {
                                format!("DELETE FROM events WHERE revision={}", head.revision)
                            };
                            let output = std::process::Command::new("/usr/bin/python3")
                                .args(["-c", python])
                                .arg(f.data.join("manager-state/client-session-index.sqlite"))
                                .arg(sql)
                                .output()
                                .unwrap();
                            assert!(
                                output.status.success(),
                                "{}",
                                String::from_utf8_lossy(&output.stderr)
                            );
                        }
                        _ => unreachable!(),
                    }
                }
                Ok(())
            },
        );
        assert!(reached.get());
        assert!(opened.is_err(), "{case}");
    }
}
#[test]
fn ready_acquisition_does_not_turn_synthetic_live_or_prior_observation_into_current_authority() {
    let (f, index, head) = synthetic_ready(true);
    drop(index);
    let (index, observed, session) =
        session_index::Index::open_ready_session(&f.context, &f.data, &f.client).unwrap();
    assert_eq!(observed, head);
    assert!(
        session
            .verify_current_ready(&f.context, Instant::now() + Duration::from_secs(1))
            .is_err(),
        "persisted synthetic PID/live is not actual attested readiness"
    );
    let origin = head.ticket.clone();
    let p = f
        .data
        .join("manager-state/client-originals")
        .join(origin)
        .join("ready.json");
    fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&p, b"changed after load").unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(
        index.head().is_err(),
        "later operations must run full current-source checks"
    );
    assert!(session.live(&f.context).is_err());
}
#[test]
fn stop_step_preserves_expired_deadline_and_independent_refusal_results() {
    assert!(matches!(
        stop_step(Instant::now(), "test.expired", || time(Instant::now())),
        Err(AppError::Timeout)
    ));
    let future = Instant::now() + Duration::from_secs(1);
    assert!(
        matches!(stop_step(future,"test.binding",||Err::<(),_>(error(()))),Err(AppError::RuntimeManaged{code,..}) if code=="client_session_unconfirmed")
    );
    assert_eq!(stop_step(future, "test.success", || Ok(42)).unwrap(), 42);
}
#[test]
fn root_stop_binding_follows_only_original_plan_archive_and_preserves_logical_scope() {
    let f = RootFixture::new();
    let scope = f.session.scope(&f.f.context).unwrap();
    let live = f.session.live(&f.f.context).unwrap();
    let pending =
        root_retirement::Pending::prepare(&f.f.context, &f.session.originals, &f.head).unwrap();
    let plan = session_index::RestorePlan {
        id: pending.id().into(),
        sha256: pending.sha256().into(),
    };
    f.verify(&plan, &scope, &live).unwrap();
    pending
        .fixture_apply_for_index(Instant::now() + Duration::from_secs(10))
        .unwrap();
    assert!(!f.f.client.exists());
    f.verify(&plan, &scope, &live).unwrap();
    let raw = fs::read(
        f.session
            .originals
            .directory
            .path()
            .join("startup-scope.json"),
    )
    .unwrap();
    let stored: Scope = parse(&raw).unwrap();
    assert_eq!(stored.client, f.f.client);
    let mut wrong = plan.clone();
    wrong.sha256 = "b".repeat(64);
    assert!(f.verify(&wrong, &scope, &live).is_err());
    let mut wrong_scope = scope.clone();
    wrong_scope.selection.version = "2.99.1".into();
    assert!(f.verify(&plan, &wrong_scope, &live).is_err());
    let mut wrong_live = live.clone();
    wrong_live.record_sha256 = "b".repeat(64);
    assert!(f.verify(&plan, &scope, &wrong_live).is_err());
    fs::create_dir(&f.f.client).unwrap();
    fs::write(f.f.client.join("outside"), b"new user root").unwrap();
    assert!(f.verify(&plan, &scope, &live).is_err());
    assert_eq!(
        fs::read(f.f.client.join("outside")).unwrap(),
        b"new user root"
    );
    println!(
        "ROOT_STOP_BINDING_EVIDENCE {}",
        serde_json::json!({"accepted":false,"sourceAndFramesVerifiedBeforeAndAfterMove":true,"logicalScopeUnchanged":true,"wrongPlanScopeLiveAndNewRootRefused":true,"syntheticFramesNoStopPermission":true})
    );
}
#[test]
fn root_stop_binding_rejects_modified_historical_frames_scope_and_live_after_move() {
    let f = RootFixture::new();
    let scope = f.session.scope(&f.f.context).unwrap();
    let live = f.session.live(&f.f.context).unwrap();
    let pending =
        root_retirement::Pending::prepare(&f.f.context, &f.session.originals, &f.head).unwrap();
    let plan = session_index::RestorePlan {
        id: pending.id().into(),
        sha256: pending.sha256().into(),
    };
    pending
        .fixture_apply_for_index(Instant::now() + Duration::from_secs(10))
        .unwrap();
    for name in [
        "original.json",
        "ready.json",
        "pre-stop.json",
        "stopped.json",
        "startup-scope.json",
        "startup-live.json",
    ] {
        let path = f.session.originals.directory.path().join(name);
        // Phase names follow the real frame contract, never guess a fallback.
        let raw = fs::read(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, b"{}").unwrap();
        assert!(
            f.verify(&plan, &scope, &live).is_err(),
            "changed {name} accepted"
        );
        assert_eq!(fs::read(&path).unwrap(), b"{}");
        fs::write(&path, raw).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        f.verify(&plan, &scope, &live).unwrap();
    }
}
