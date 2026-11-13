//! Storage close and action gates only; synthetic movement is not native Stop.
use super::*;
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    time::Duration,
};
fn fixture(
    second: bool,
) -> (
    tempfile::TempDir,
    ProcessContext,
    Index,
    Head,
    Originals,
    super::super::root_retirement::AuthorizedReadback,
) {
    let (temp, context, index, head, originals) = if second {
        super::restore_registration_tests::full_history_fixture()
    } else {
        super::super::root_retirement::fixture_captured_generation()
    };
    let head = if second {
        let pending = restore::Pending::prepare_fixture(
            &originals,
            super::restore_registration_tests::evidence(),
        )
        .unwrap();
        index
            .record_restore_plan(&head, pending.id(), pending.sha256())
            .unwrap()
    } else {
        head
    };
    let pending =
        super::super::root_retirement::Pending::prepare(&context, &originals, &head).unwrap();
    let prepared = index
        .record_root_retirement(&head, pending.id(), pending.sha256())
        .unwrap();
    let proof = pending
        .fixture_apply_for_index(Instant::now() + Duration::from_secs(10))
        .unwrap();
    drop(pending);
    (temp, context, index, prepared, originals, proof)
}
#[test]
fn stopped_close_eligibility_matches_status_without_another_complete_history_audit() {
    let mut rows = Vec::new();
    for second in [false, true] {
        for project in [false, true] {
            let (_temp, context, index, prepared, _originals, proof) = fixture(second);
            let (closed, metrics) = observation_metrics::measure(|| {
                if project {
                    let (closed, eligible) = index
                        .close_root_absence_with_eligibility(&prepared, &proof)
                        .unwrap();
                    assert!(eligible.can_start());
                    let action = crate::types::process_action::ProcessActionResult::from_state(
                        crate::modules::process::ProcessLifecycleState::Stopping,
                    )
                    .with_result(crate::modules::process::LifecycleResult::Stopped)
                    .with_closed_root_start(eligible);
                    assert!(
                        action.can_start
                            && !action.client_recovery_required
                            && action.failure.is_none()
                    );
                    assert_eq!(
                        action.result,
                        Some(crate::modules::process::LifecycleResult::Stopped)
                    );
                    closed
                } else {
                    index.close_root_absence(&prepared, &proof).unwrap()
                }
            });
            assert_eq!(index.head().unwrap(), Some(closed));
            let data = index.data_root().to_path_buf();
            drop(index);
            assert!(generation::inspect_successor_eligibility(&context, &data).unwrap());
            rows.push(serde_json::to_value(metrics).unwrap());
        }
    }
    for key in [
        "frame",
        "frame_archives",
        "historical_begin",
        "historical_finish",
        "historical_full_pass",
        "transaction_source",
        "transaction_candidate",
        "transaction_postcommit",
    ] {
        for pair in rows.chunks_exact(2) {
            assert_eq!(
                pair[0].get(key).map(|v| v["calls"].clone()),
                pair[1].get(key).map(|v| v["calls"].clone()),
                "{key}"
            );
        }
    }
    println!(
        "STOP_ELIGIBILITY_AUDIT_COUNTS {}",
        serde_json::json!({"accepted":false,"ordinary":rows[0],"projected":rows[1],"secondOrdinary":rows[2],"secondProjected":rows[3],"noAdditionalHistoricalFullAudit":true,"scope":"synthetic closed metadata, not normal Stop or native action time"})
    );
}
#[test]
fn stopped_close_eligibility_capacity_false_preserves_success_and_blocks_next_start() {
    let (_temp, context, index, prepared, _originals, proof) = fixture(false);
    let first = index.close_root_absence(&prepared, &proof).unwrap();
    let c = index.connection().unwrap();
    c.execute_batch("BEGIN IMMEDIATE").unwrap();
    let mut current = first.clone();
    for revision in 6..=4094 {
        let mut next = if revision % 2 == 0 {
            prepared.clone()
        } else {
            first.clone()
        };
        next.revision = revision;
        c.execute(
            "INSERT INTO events(revision,document) VALUES(?1,?2)",
            params![revision, encode(&next).unwrap()],
        )
        .unwrap();
        current = next;
    }
    c.execute(
        "UPDATE current_session SET revision=?1,document=?2 WHERE singleton=1",
        params![current.revision, encode(&current).unwrap()],
    )
    .unwrap();
    c.execute_batch("COMMIT").unwrap();
    let (closed, eligible) = index
        .close_root_absence_with_eligibility(&current, &proof)
        .unwrap();
    assert_eq!(closed.revision, 4095);
    assert_eq!(closed.state, State::RootAbsent);
    assert!(!eligible.can_start());
    let action = crate::types::process_action::ProcessActionResult::from_state(
        crate::modules::process::ProcessLifecycleState::Stopping,
    )
    .with_result(crate::modules::process::LifecycleResult::Stopped)
    .with_closed_root_start(eligible);
    assert_eq!(
        action.result,
        Some(crate::modules::process::LifecycleResult::Stopped)
    );
    assert!(action.failure.is_none() && action.client_recovery_required);
    assert!(!action.can_start && !action.can_stop && !action.can_restart);
    let data = index.data_root().to_path_buf();
    drop(index);
    assert_eq!(
        generation::inspect_client_recovery(&context, &data).unwrap(),
        crate::modules::status::ClientRecoveryState::Required
    );
    assert!(!generation::inspect_successor_eligibility(&context, &data).unwrap());
    assert!(Index::prepare_start_capture(
        &context,
        &data,
        Instant::now() + Duration::from_secs(10)
    )
    .is_err());
    assert!(!context.working_directory.join(".codex").exists());
}
#[test]
fn stopped_close_candidate_never_escapes_failed_final_authentication() {
    for mutation in [
        "frame",
        "plan",
        "receipt",
        "namespace",
        "root",
        "binding",
        "event",
        "projection-error",
    ] {
        let (_temp, context, index, prepared, originals, proof) = fixture(false);
        let reached = std::cell::Cell::new(false);
        let result = index.close_root_absence_with_eligibility_checked(&prepared, &proof, &|| {
            reached.set(true);
            let movement = prepared.root_retirement.as_ref().unwrap();
            let file = match mutation {
                "frame" => Some(originals.directory.path().join("original.json")),
                "plan" => Some(
                    originals
                        .directory
                        .path()
                        .join(format!("root-retirement-{}.json", movement.plan.id)),
                ),
                "receipt" => Some(originals.directory.path().join(format!(
                    "root-retirement-{}-Retire-readback.json",
                    movement.plan.id
                ))),
                _ => None,
            };
            if let Some(path) = file {
                fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
                fs::write(&path, b"kept-corruption").unwrap();
                fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
            }
            match mutation {
                "namespace" => {
                    let root = index
                        .data_root()
                        .join("manager-state/client-root-generations");
                    fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
                    fs::DirBuilder::new()
                        .mode(0o700)
                        .create(root.join(uuid::Uuid::new_v4().to_string()))
                        .unwrap();
                }
                "root" => {
                    fs::DirBuilder::new()
                        .mode(0o700)
                        .create(context.working_directory.join(".codex"))
                        .unwrap();
                }
                "binding" => {
                    index
                        .connection()
                        .unwrap()
                        .execute(
                            "UPDATE binding SET document=?1",
                            [b"changed-binding".as_slice()],
                        )
                        .unwrap();
                }
                "event" => {
                    index
                        .connection()
                        .unwrap()
                        .execute(
                            "UPDATE events SET document=?1 WHERE revision=1",
                            [b"changed-event".as_slice()],
                        )
                        .unwrap();
                }
                "projection-error" => return Err(denied("candidate refused")),
                _ => (),
            }
            Ok(())
        });
        assert!(reached.get());
        assert!(result.is_err(), "{mutation}");
        let head = read_head(index.connection().unwrap()).unwrap().unwrap();
        assert_eq!(head.state, State::RootAbsent);
        assert_eq!(head.revision, 5);
    }
}
#[test]
fn stopped_close_expired_movement_proof_never_returns_a_start_eligibility() {
    let (_temp, _context, index, head, _originals, mut proof) = fixture(false);
    proof.fixture_expire();
    assert!(index
        .close_root_absence_with_eligibility(&head, &proof)
        .is_err());
    assert_eq!(index.head().unwrap(), Some(head));
}
#[test]
fn stopped_close_eligibility_does_not_authorize_a_later_root_or_deadline() {
    let (_temp, context, index, head, _originals, proof) = fixture(false);
    let (_closed, eligible) = index
        .close_root_absence_with_eligibility(&head, &proof)
        .unwrap();
    assert!(eligible.can_start());
    let data = index.data_root().to_path_buf();
    drop(index);
    fs::DirBuilder::new()
        .mode(0o700)
        .create(context.working_directory.join(".codex"))
        .unwrap();
    fs::write(
        context.working_directory.join(".codex/user"),
        b"keep new user root",
    )
    .unwrap();
    assert!(Index::prepare_start_capture(
        &context,
        &data,
        Instant::now() + Duration::from_secs(10)
    )
    .is_err());
    assert_eq!(
        fs::read(context.working_directory.join(".codex/user")).unwrap(),
        b"keep new user root"
    );
}
#[test]
fn stopped_close_candidate_late_completion_keeps_original_pending_demotion() {
    let (_temp, _context, index, head, _originals, mut proof) = fixture(false);
    proof.fixture_limit_deadline(Instant::now() + Duration::from_secs(1));
    let reached = std::cell::Cell::new(false);
    let result = index.close_root_absence_with_eligibility_checked(&head, &proof, &|| {
        reached.set(true);
        std::thread::sleep(Duration::from_millis(1100));
        Ok(())
    });
    assert!(reached.get());
    assert!(result.is_err());
    let current = read_head(index.connection().unwrap()).unwrap().unwrap();
    assert_eq!(current.state, State::RootRetirementPrepared);
    assert_eq!(current.revision, head.revision + 2);
    assert!(current.completion_sha256.is_none());
    assert!(current.root_retirement == head.root_retirement);
}
