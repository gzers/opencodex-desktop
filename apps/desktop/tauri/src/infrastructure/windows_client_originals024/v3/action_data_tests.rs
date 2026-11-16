use super::super::{
    observations::{WorkEvent, WorkOperation},
    segments::{LeaseEvent, LeaseOperation},
    snapshots::{
        tests::{observed, refs, snapshot_event},
        LifecyclePhase, SnapshotEvent, SnapshotOperation, SnapshotPurpose,
    },
    tests::plan,
};
use super::*;
fn work(g: &Progress, operation: WorkOperation) -> WorkEvent {
    WorkEvent {
        schema: "windows-client-originals-v3".into(),
        document_kind: "work-interval-event".into(),
        storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
        event_id: uuid::Uuid::new_v4().to_string(),
        sequence: g.sequence + 1,
        session_id: g.session.clone(),
        operation,
        frame_digests: if operation == WorkOperation::WorkSealIntent {
            None
        } else {
            g.work_frames.clone()
        },
        lease_epoch: g.epoch.clone(),
        claim_key: g.claim.clone(),
        deadline_scope: g.deadline.clone(),
        previous_event_digest: g.tip.clone(),
        storage_plan_digest: g.plan.clone(),
        security_policy_digest: Policy::fixed().digest().unwrap(),
        manifest_digest: g.manifest.clone(),
        binding_digest: g.binding.clone(),
    }
}
fn lease(g: &Progress, operation: LeaseOperation, next: Option<String>) -> LeaseEvent {
    LeaseEvent {
        schema: "windows-client-originals-v3".into(),
        document_kind: "lease-event".into(),
        storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
        event_id: uuid::Uuid::new_v4().to_string(),
        sequence: g.sequence + 1,
        session_id: g.session.clone(),
        operation,
        lease_epoch: g.epoch.clone(),
        next_epoch: next,
        claim_key: g.claim.clone(),
        objects_digest: "9".repeat(64),
        deadline_scope: g.deadline.clone(),
        previous_event_digest: g.tip.clone(),
        storage_plan_digest: g.plan.clone(),
        security_policy_digest: Policy::fixed().digest().unwrap(),
        manifest_digest: g.manifest.clone(),
        binding_digest: g.binding.clone(),
    }
}
fn ready(p: &StoragePlan) -> Progress {
    let mut g = observed(p);
    let seq = g.sequence;
    g.apply_snapshot(
        p,
        &snapshot_event(
            &g,
            SnapshotOperation::SnapshotLinkIntent,
            SnapshotPurpose::Ready,
            seq,
            None,
        ),
    )
    .unwrap();
    let values = refs(p, &g, SnapshotPurpose::Ready, seq);
    g.apply_snapshot(
        p,
        &snapshot_event(
            &g,
            SnapshotOperation::SnapshotLinkedVerified,
            SnapshotPurpose::Ready,
            seq,
            Some(values),
        ),
    )
    .unwrap();
    let mut e = snapshot_event(
        &g,
        SnapshotOperation::ReadyVerified,
        SnapshotPurpose::Ready,
        seq,
        Some(refs(p, &g, SnapshotPurpose::Ready, seq)),
    );
    e.ready = Some((123, 456, "7".repeat(64)));
    g.apply_snapshot(p, &e).unwrap();
    g.apply_work(p, &work(&g, WorkOperation::WorkUnchangedVerified))
        .unwrap();
    g.apply_work(p, &work(&g, WorkOperation::WorkOpenIntent))
        .unwrap();
    g.apply_work(p, &work(&g, WorkOperation::WorkOpenedVerified))
        .unwrap();
    g.apply_lease(p, &lease(&g, LeaseOperation::LeaseReleasePrepared, None))
        .unwrap();
    g.apply_lease(p, &lease(&g, LeaseOperation::LeaseReleasedVerified, None))
        .unwrap();
    g
}
fn action(g: &Progress, operation: ActionOperation, next: String) -> ActionEvent {
    ActionEvent {
        schema: "windows-client-originals-v3".into(),
        document_kind: "original-action-event".into(),
        storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
        event_id: uuid::Uuid::new_v4().to_string(),
        sequence: g.sequence + 1,
        session_id: g.session.clone(),
        operation,
        next_action_scope: next,
        action_kind: ActionKind::Restart,
        ready: g.ready_generation.clone().unwrap(),
        lease_objects_digest: g.lease_objects.clone().unwrap(),
        lease_epoch: g.epoch.clone(),
        claim_key: g.claim.clone(),
        deadline_scope: g.deadline.clone(),
        previous_event_digest: g.tip.clone(),
        storage_plan_digest: g.plan.clone(),
        security_policy_digest: Policy::fixed().digest().unwrap(),
        manifest_digest: g.manifest.clone(),
        binding_digest: g.binding.clone(),
    }
}
#[test]
fn wl2_r6_stop_action_data_exact_ready_context_order_no_replay_or_deadline_laundering() {
    let (mut f, p, _, _) = plan();
    let mut g = ready(&p);
    let old = g.deadline.clone();
    let next = uuid::Uuid::new_v4().to_string();
    assert!(g
        .apply_action(
            &p,
            &action(&g, ActionOperation::StopActionBoundVerified, next.clone())
        )
        .is_err());
    let good = action(&g, ActionOperation::StopActionIntent, next.clone());
    for field in [
        "schema",
        "documentKind",
        "sessionId",
        "leaseEpoch",
        "claimKey",
        "deadlineScope",
        "previousEventDigest",
        "storagePlanDigest",
        "securityPolicyDigest",
        "manifestDigest",
        "bindingDigest",
        "leaseObjectsDigest",
        "nextActionScope",
    ] {
        let mut value = serde_json::to_value(&good).unwrap();
        value[field] = "invalid".into();
        let e: ActionEvent = serde_json::from_value(value).unwrap();
        assert!(g.apply_action(&p, &e).is_err(), "drift {field}");
    }
    let mut value = serde_json::to_value(&good).unwrap();
    value["ready"][0] = 999.into();
    assert!(g
        .apply_action(&p, &serde_json::from_value(value).unwrap())
        .is_err());
    let mut value = serde_json::to_value(&good).unwrap();
    value["deadlineQpc"] = true.into();
    assert!(serde_json::from_value::<ActionEvent>(value).is_err());
    let mut value = serde_json::to_value(&good).unwrap();
    value["nextActionScope"] = old.clone().into();
    assert!(g
        .apply_action(&p, &serde_json::from_value(value).unwrap())
        .is_err());
    g.apply_action(&p, &good).unwrap();
    assert_eq!(g.deadline, old);
    assert!(g.action_pending.is_some());
    assert!(g
        .apply_lease(&p, &lease(&g, LeaseOperation::LeaseAcquirePrepared, None))
        .is_err());
    assert!(g
        .apply_action(
            &p,
            &action(
                &g,
                ActionOperation::StopActionBoundVerified,
                uuid::Uuid::new_v4().to_string()
            )
        )
        .is_err());
    g.apply_action(
        &p,
        &action(&g, ActionOperation::StopActionBoundVerified, next.clone()),
    )
    .unwrap();
    assert_eq!(g.deadline, next);
    assert_eq!(g.action_kind, ActionKind::Restart);
    assert!(g.action_pending.is_none());
    assert!(g
        .apply_action(
            &p,
            &action(
                &g,
                ActionOperation::StopActionIntent,
                uuid::Uuid::new_v4().to_string()
            )
        )
        .is_err());
    g.apply_lease(&p, &lease(&g, LeaseOperation::LeaseAcquirePrepared, None))
        .unwrap();
    g.apply_lease(
        &p,
        &lease(
            &g,
            LeaseOperation::LeaseAcquiredVerified,
            Some(uuid::Uuid::new_v4().to_string()),
        ),
    )
    .unwrap();
    println!("r6 new-action DATA preserves Ready generation and old scope until verified binding, rejects13 context drifts/unknown deadline/replay/skip and lease during pending; accepts no actual NativeAction, remains data only");
    f.retain();
}
#[test]
fn wl2_r6_stop_phase_data_exact_prestop_teardown_stopped_and_ready_generation() {
    let (mut f, p, _, _) = plan();
    let mut g = ready(&p);
    let next = uuid::Uuid::new_v4().to_string();
    g.apply_action(
        &p,
        &action(&g, ActionOperation::StopActionIntent, next.clone()),
    )
    .unwrap();
    g.apply_action(
        &p,
        &action(&g, ActionOperation::StopActionBoundVerified, next),
    )
    .unwrap();
    g.apply_lease(&p, &lease(&g, LeaseOperation::LeaseAcquirePrepared, None))
        .unwrap();
    g.apply_lease(
        &p,
        &lease(
            &g,
            LeaseOperation::LeaseAcquiredVerified,
            Some(uuid::Uuid::new_v4().to_string()),
        ),
    )
    .unwrap();
    // Native frame capture is deliberately not simulated as authority. These
    // are progression DATA; real factories require native typed witnesses.
    g.apply_work(&p, &work(&g, WorkOperation::WorkSealIntent))
        .unwrap();
    g.apply_work(&p, &work(&g, WorkOperation::WorkSealedVerified))
        .unwrap();
    for (purpose, operation, phase) in [
        (
            SnapshotPurpose::PreStop,
            SnapshotOperation::PreStopVerified,
            LifecyclePhase::PreStop,
        ),
        (
            SnapshotPurpose::Stopped,
            SnapshotOperation::StoppedVerified,
            LifecyclePhase::Stopped,
        ),
    ] {
        let seq = g.sequence;
        g.apply_snapshot(
            &p,
            &snapshot_event(
                &g,
                SnapshotOperation::SnapshotLinkIntent,
                purpose,
                seq,
                None,
            ),
        )
        .unwrap();
        g.apply_snapshot(
            &p,
            &snapshot_event(
                &g,
                SnapshotOperation::SnapshotLinkedVerified,
                purpose,
                seq,
                Some(refs(&p, &g, purpose, seq)),
            ),
        )
        .unwrap();
        let mut e = snapshot_event(
            &g,
            operation,
            purpose,
            seq,
            Some(refs(&p, &g, purpose, seq)),
        );
        e.ready = g.ready_generation.clone();
        let mut bad: SnapshotEvent = serde_json::from_slice(&encoded(&e).unwrap()).unwrap();
        bad.ready.as_mut().unwrap().0 += 1;
        assert!(g.apply_snapshot(&p, &bad).is_err());
        g.apply_snapshot(&p, &e).unwrap();
        assert_eq!(g.lifecycle, phase);
        if purpose == SnapshotPurpose::PreStop {
            let mut e = snapshot_event(&g, SnapshotOperation::TeardownPrepared, purpose, seq, None);
            e.ready = g.ready_generation.clone();
            g.apply_snapshot(&p, &e).unwrap();
            assert_eq!(g.lifecycle, LifecyclePhase::TeardownPrepared);
            g.apply_work(&p, &work(&g, WorkOperation::WorkResumeIntent))
                .unwrap();
            g.apply_work(&p, &work(&g, WorkOperation::WorkResumedVerified))
                .unwrap();
            g.apply_work(&p, &work(&g, WorkOperation::WorkSealIntent))
                .unwrap();
            g.apply_work(&p, &work(&g, WorkOperation::WorkSealedVerified))
                .unwrap();
        }
    }
    assert!(!g.complete_files(&p));
    println!("r6 PreStop/teardown/Stopped phase DATA keeps exact Ready generation/ref chains and rejects generation drift, no Closed eligibility or actual Stop proof is minted");
    f.retain();
}
