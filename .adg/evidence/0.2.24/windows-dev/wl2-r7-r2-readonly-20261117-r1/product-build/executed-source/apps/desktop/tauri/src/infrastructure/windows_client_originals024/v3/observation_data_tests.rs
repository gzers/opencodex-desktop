use super::super::tests::{event, plan, progress};
use super::*;
fn ready(p: &StoragePlan, g: &mut Progress) {
    for n in p.roots.iter().flat_map(|r| &r.nodes) {
        if n.kind != Kind::CustodiedFile {
            g.apply(p, &event(p, g, n, Operation::SnapshotVerified, "999"))
                .unwrap();
        }
    }
    for (i, n) in p
        .roots
        .iter()
        .flat_map(|r| &r.nodes)
        .filter(|n| n.kind == Kind::CustodiedFile)
        .enumerate()
    {
        for op in [
            Operation::NodeMoveIntent,
            Operation::NodeMovedVerified,
            Operation::WorkCreateIntent,
            Operation::WorkCreatedVerified,
        ] {
            g.apply(p, &event(p, g, n, op, &(u64::MAX - i as u64).to_string()))
                .unwrap();
        }
    }
}
fn work_event(g: &Progress, operation: WorkOperation, frames: Option<[String; 2]>) -> WorkEvent {
    WorkEvent {
        schema:
            crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3
                .into(),
        document_kind: "work-interval-event".into(),
        storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
        event_id: uuid::Uuid::new_v4().to_string(),
        sequence: g.sequence + 1,
        session_id: g.session.clone(),
        operation,
        frame_digests: frames,
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
fn wl2_r6_work_observation_data_rejects_incomplete_context_skip_changed_reaccept_and_text_permission(
) {
    let (mut f, p, _, _) = plan();
    let mut g = progress(&p);
    let frames = ["1".repeat(64), "2".repeat(64)];
    assert!(g
        .apply_work(
            &p,
            &work_event(&g, WorkOperation::WorkOpenIntent, Some(frames.clone()))
        )
        .is_err());
    ready(&p, &mut g);
    assert!(g
        .apply_work(
            &p,
            &work_event(&g, WorkOperation::WorkOpenedVerified, Some(frames.clone()))
        )
        .is_err());
    let good = work_event(&g, WorkOperation::WorkOpenIntent, Some(frames.clone()));
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
    ] {
        let mut v = serde_json::to_value(&good).unwrap();
        v[field] = "wrong".into();
        let bad: WorkEvent = serde_json::from_value(v).unwrap();
        assert!(g.apply_work(&p, &bad).is_err(), "drift {field}");
    }
    let mut v = serde_json::to_value(&good).unwrap();
    v["restoreAuthority"] = true.into();
    assert!(serde_json::from_value::<WorkEvent>(v).is_err());
    g.apply_work(&p, &good).unwrap();
    let n = p.roots[0]
        .nodes
        .iter()
        .find(|n| n.kind == Kind::CustodiedFile)
        .unwrap();
    assert!(g
        .apply(
            &p,
            &event(
                &p,
                &g,
                n,
                Operation::WorkRetireIntent,
                &u64::MAX.to_string()
            )
        )
        .is_err());
    let e:super::super::segments::LeaseEvent=serde_json::from_value(serde_json::json!({"schema":"windows-client-originals-v3","documentKind":"lease-event","storageStrategy":ANCHORED_STORAGE_STRATEGY,"eventId":uuid::Uuid::new_v4().to_string(),"sequence":g.sequence+1,"sessionId":g.session,"operation":"LeaseReleasePrepared","leaseEpoch":g.epoch,"nextEpoch":null,"claimKey":g.claim,"objectsDigest":"9".repeat(64),"deadlineScope":g.deadline,"previousEventDigest":g.tip,"storagePlanDigest":g.plan,"securityPolicyDigest":Policy::fixed().digest().unwrap(),"manifestDigest":g.manifest,"bindingDigest":g.binding})).unwrap();
    assert!(
        g.apply_entry(&p, &super::super::segments::ChainEntry::Lease(e))
            .is_err(),
        "incomplete open cannot release"
    );
    g.apply_work(
        &p,
        &work_event(&g, WorkOperation::WorkOpenedVerified, Some(frames.clone())),
    )
    .unwrap();
    g.apply_work(&p, &work_event(&g, WorkOperation::WorkSealIntent, None))
        .unwrap();
    g.apply_work(
        &p,
        &work_event(
            &g,
            WorkOperation::WorkSealedVerified,
            Some(["3".repeat(64), "2".repeat(64)]),
        ),
    )
    .unwrap();
    assert!(
        g.apply_work(
            &p,
            &work_event(&g, WorkOperation::WorkUnchangedVerified, Some(frames))
        )
        .is_err(),
        "observed changed generation/content digest cannot be re-labelled with older baseline"
    );
    assert_eq!(g.work_mode, WorkMode::Observed);
    assert!(!g.complete_files(&p));
    println!("r6 work interval same-chain data refuses incomplete tree,11 context drifts, skip/text authority/native operations while open and release before open result; observed digest persists and cannot be laundered as unchanged");
    f.retain();
}
