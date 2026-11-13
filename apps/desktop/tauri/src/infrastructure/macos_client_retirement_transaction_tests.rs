//! Registration/source transaction only. Synthetic Captured closure and plans
//! do not certify a normal service Stop or construct a native move permit.
use super::*;
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    time::Duration,
};
pub(super) fn fixture() -> (tempfile::TempDir, ProcessContext, Index, Head, Originals) {
    let (tmp, context, index, closed) = super::super::root_retirement::fixture_closed_generation();
    let (index, preparing) = index
        .create_successor_root(&context, &closed, Instant::now() + Duration::from_secs(30))
        .unwrap();
    let (head, originals) = generation::fixture_captured_successor(&context, &index, &preparing);
    (tmp, context, index, head, originals)
}
#[test]
fn retirement_registration_transaction_keeps_full_source_candidate_and_committed_checks() {
    let mut samples = Vec::new();
    for reference in [true, false, false, true] {
        let (_tmp, context, index, head, originals) = fixture();
        let pending =
            super::super::root_retirement::Pending::prepare(&context, &originals, &head).unwrap();
        let started = Instant::now();
        let (result, metrics) = observation_metrics::measure(|| {
            if reference {
                // Reproduce the pre-change initial source/plan admission plus
                // subsequent full registration observation in this fixture.
                let old = index.registered_root_originals(&head).unwrap();
                let plan = super::super::root_retirement::Pending::open(
                    &old,
                    pending.id(),
                    pending.sha256(),
                )
                .unwrap();
                plan.require_source(&head).unwrap();
                plan.inspect_live().unwrap();
                // Original implementation wrote this exact next Head with
                // write(), after its separate plan admission above. Calling
                // the new record method here would admit the plan twice.
                let mut prepared = head.clone();
                prepared.revision += 1;
                prepared.state = State::RootRetirementPrepared;
                prepared.root_retirement = Some(RootRetirement {
                    source_revision: head.revision,
                    plan: RestorePlan {
                        id: pending.id().into(),
                        sha256: pending.sha256().into(),
                    },
                });
                prepared.root_generation = None;
                index.write(Some(&head), &prepared).unwrap();
                let p = prepared.root_retirement.as_ref().unwrap().plan.clone();
                let proof = index
                    .root_retirement_registration(&head, &prepared, &p)
                    .unwrap();
                (prepared, proof)
            } else {
                index
                    .record_root_retirement_registered(&head, pending.id(), pending.sha256())
                    .unwrap()
            }
        });
        let elapsed = started.elapsed().as_nanos();
        let (prepared, registration) = result;
        assert_eq!(prepared.state, State::RootRetirementPrepared);
        assert_eq!(prepared.revision, head.revision + 1);
        registration
            .require(&head, registration.plan().unwrap())
            .unwrap();
        assert_eq!(index.head().unwrap(), Some(prepared));
        let metrics = serde_json::to_value(metrics).unwrap();
        assert_eq!(
            metrics["historical_begin"]["calls"],
            if reference { 5 } else { 3 }
        );
        samples.push(
            serde_json::json!({"reference":reference,"elapsedNanos":elapsed,"metrics":metrics}),
        );
    }
    println!(
        "RETIREMENT_TRANSACTION_COMPARISON {}",
        serde_json::json!({"accepted":false,"samples":samples,"scope":"synthetic secondgen/source+plan+SQL/currentreadback; not OS movement authority or original Stop acceptance"})
    );
}
#[test]
fn retirement_registration_transaction_rolls_back_changes_after_source_admission() {
    for mutation in ["plan", "old-archive", "root-content", "namespace"] {
        let (_tmp, context, index, head, originals) = fixture();
        let pending =
            super::super::root_retirement::Pending::prepare(&context, &originals, &head).unwrap();
        let reached = std::cell::Cell::new(false);
        let result = index.record_root_retirement_registered_checked(
            &head,
            pending.id(),
            pending.sha256(),
            &|stage| {
                if stage == "source_admitted" {
                    reached.set(true);
                    match mutation {
                        "plan" => {
                            let p = originals
                                .directory
                                .path()
                                .join(format!("root-retirement-{}.json", pending.id()));
                            fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
                            fs::write(&p, b"changedplan").unwrap();
                            fs::set_permissions(&p, fs::Permissions::from_mode(0o400)).unwrap();
                        }
                        "old-archive" => {
                            let c = index.connection().unwrap();
                            let raw: Vec<u8> = c
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
                            fs::write(&p, b"changedold").unwrap();
                            fs::set_permissions(&p, fs::Permissions::from_mode(0o400)).unwrap();
                        }
                        "root-content" => {
                            fs::write(
                                context.working_directory.join(".codex/outside"),
                                b"keepoutside",
                            )
                            .unwrap();
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
        assert!(result.is_err(), "{mutation}");
        assert_eq!(
            read_head(index.connection().unwrap()).unwrap(),
            Some(head.clone())
        );
        let events: u64 = index
            .connection()
            .unwrap()
            .query_row("SELECT count(*) FROM events", [], |r| r.get(0))
            .unwrap();
        assert_eq!(events, head.revision);
        assert!(context.working_directory.join(".codex").exists());
    }
}
#[test]
fn retirement_registration_transaction_rejects_invalid_source_plan_and_missing_originals_before_sql(
) {
    for mutation in [
        "stale-source",
        "wrong-plan",
        "missing-root-registration",
        "admission-error",
    ] {
        let (_tmp, context, mut index, head, originals) = fixture();
        let pending =
            super::super::root_retirement::Pending::prepare(&context, &originals, &head).unwrap();
        let mut source = head.clone();
        if mutation == "stale-source" {
            source.revision += 1;
        }
        let digest = if mutation == "wrong-plan" {
            "a".repeat(64)
        } else {
            pending.sha256().into()
        };
        let registration = if mutation == "missing-root-registration" {
            index.root_registration.take()
        } else {
            None
        };
        let admitted = std::cell::Cell::new(false);
        let sql_written = std::cell::Cell::new(false);
        let refused = index
            .record_root_retirement_registered_checked(&source, pending.id(), &digest, &|point| {
                if point == "source_admitted" {
                    admitted.set(true);
                    return Err(denied("injected admission interruption"));
                }
                if point == "references_sql_written" {
                    sql_written.set(true);
                }
                Ok(())
            })
            .is_err();
        if registration.is_some() {
            index.root_registration = registration;
        }
        assert!(refused, "{mutation}");
        assert_eq!(admitted.get(), mutation == "admission-error");
        assert!(!sql_written.get(), "{mutation}");
        assert_eq!(index.head().unwrap(), Some(head.clone()));
        let events: u64 = index
            .connection()
            .unwrap()
            .query_row("SELECT count(*) FROM events", [], |r| r.get(0))
            .unwrap();
        assert_eq!(events, head.revision);
        assert!(context.working_directory.join(".codex").exists());
    }
}
#[test]
fn retirement_registration_transaction_keeps_postcommit_failures_and_does_not_mint_a_proof() {
    for stage in ["references_sql_written", "references_committed"] {
        let (_tmp, context, index, head, originals) = fixture();
        let pending =
            super::super::root_retirement::Pending::prepare(&context, &originals, &head).unwrap();
        let reached = std::cell::Cell::new(false);
        let result = index.record_root_retirement_registered_checked(
            &head,
            pending.id(),
            pending.sha256(),
            &|point| {
                if point == stage {
                    reached.set(true);
                    fs::write(
                        context.working_directory.join(".codex/late-user"),
                        b"retained",
                    )
                    .unwrap();
                }
                Ok(())
            },
        );
        assert!(reached.get());
        assert!(result.is_err(), "{stage}");
        let current = read_head(index.connection().unwrap()).unwrap().unwrap();
        assert_eq!(
            current.state,
            if stage == "references_committed" {
                State::RootRetirementPrepared
            } else {
                State::Captured
            }
        );
        assert_eq!(
            current.revision,
            head.revision + u64::from(stage == "references_committed")
        );
        assert_eq!(
            fs::read(context.working_directory.join(".codex/late-user")).unwrap(),
            b"retained"
        );
    }
}
#[test]
fn retirement_registration_descendant_acquisition_keeps_full_audits_with_lower_directory_cost() {
    let mut samples = Vec::new();
    for reference in [true, false, false, true] {
        let (_tmp, context, index, head, originals) = fixture();
        let pending =
            super::super::root_retirement::Pending::prepare(&context, &originals, &head).unwrap();
        let started = Instant::now();
        let (result, metrics) = observation_metrics::measure(|| {
            let register =
                || index.record_root_retirement_registered(&head, pending.id(), pending.sha256());
            if reference {
                observation_metrics::descendant_reference(register)
            } else {
                register()
            }
        });
        let elapsed = started.elapsed().as_nanos();
        let (prepared, registration) = result.unwrap();
        registration
            .require(&head, registration.plan().unwrap())
            .unwrap();
        assert_eq!(index.head().unwrap(), Some(prepared));
        let metrics = serde_json::to_value(metrics).unwrap();
        assert_eq!(metrics["historical_begin"]["calls"], 3);
        assert_eq!(metrics["historical_full_pass"]["calls"], 6);
        samples.push(
            serde_json::json!({"reference":reference,"elapsedNanos":elapsed,"metrics":metrics}),
        );
    }
    let old = samples[0]["metrics"]["protection"]["calls"]
        .as_u64()
        .unwrap();
    let new = samples[1]["metrics"]["protection"]["calls"]
        .as_u64()
        .unwrap();
    assert!(new < old);
    assert_eq!(samples[2]["metrics"]["protection"]["calls"], new);
    assert_eq!(samples[3]["metrics"]["protection"]["calls"], old);
    println!(
        "RETIREMENT_DESCENDANT_COMPARISON {}",
        serde_json::json!({"accepted":false,"samples":samples,"scope":"synthetic second generation; complete source/candidate/postcommit audits retained; no original normalStop acceptance"})
    );
}
