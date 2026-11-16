use super::super::tests::{event, plan, progress};
use super::*;
fn lease_event(g: &Progress, operation: LeaseOperation, next: Option<String>) -> LeaseEvent {
    LeaseEvent {
        schema:
            crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3
                .into(),
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
fn working(p: &StoragePlan, g: &mut Progress) {
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
#[test]
fn wl2_r6_lease_data_same_chain_rejects_bad_state_epoch_scope_digest_and_node_escape() {
    let (mut f, p, _, _) = plan();
    let mut g = progress(&p);
    assert!(g
        .apply_lease(
            &p,
            &lease_event(&g, LeaseOperation::LeaseReleasePrepared, None)
        )
        .is_err());
    working(&p, &mut g);
    let ready = lease_event(&g, LeaseOperation::LeaseReleasePrepared, None);
    for field in [
        "schema",
        "documentKind",
        "sessionId",
        "storageStrategy",
        "leaseEpoch",
        "claimKey",
        "deadlineScope",
        "previousEventDigest",
        "storagePlanDigest",
        "securityPolicyDigest",
        "manifestDigest",
        "bindingDigest",
        "objectsDigest",
    ] {
        let mut v = serde_json::to_value(&ready).unwrap();
        v[field] = "invalid".into();
        let bad: LeaseEvent = serde_json::from_value(v).unwrap();
        assert!(g.apply_lease(&p, &bad).is_err(), "lease drift {field}");
    }
    g.apply_lease(&p, &ready).unwrap();
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
    assert!(g
        .apply_lease(
            &p,
            &lease_event(&g, LeaseOperation::LeaseAcquirePrepared, None)
        )
        .is_err());
    g.apply_lease(
        &p,
        &lease_event(&g, LeaseOperation::LeaseReleasedVerified, None),
    )
    .unwrap();
    assert!(g
        .apply_lease(
            &p,
            &lease_event(
                &g,
                LeaseOperation::LeaseAcquiredVerified,
                Some(uuid::Uuid::new_v4().to_string())
            )
        )
        .is_err());
    let mut changed = lease_event(&g, LeaseOperation::LeaseAcquirePrepared, None);
    changed.objects_digest = "8".repeat(64);
    assert!(g.apply_lease(&p, &changed).is_err());
    g.apply_lease(
        &p,
        &lease_event(&g, LeaseOperation::LeaseAcquirePrepared, None),
    )
    .unwrap();
    assert!(g
        .apply_lease(
            &p,
            &lease_event(
                &g,
                LeaseOperation::LeaseAcquiredVerified,
                Some(g.epoch.clone())
            )
        )
        .is_err());
    let old = g.epoch.clone();
    let new = uuid::Uuid::new_v4().to_string();
    g.apply_lease(
        &p,
        &lease_event(&g, LeaseOperation::LeaseAcquiredVerified, Some(new.clone())),
    )
    .unwrap();
    assert_eq!(g.epoch, new);
    assert_ne!(g.epoch, old);
    assert_eq!(g.lease_state, LeaseState::Active);
    assert!(!g.complete_files(&p));
    g.apply_lease(
        &p,
        &lease_event(&g, LeaseOperation::LeaseReleasePrepared, None),
    )
    .unwrap();
    g.apply_lease(
        &p,
        &lease_event(&g, LeaseOperation::LeaseReleasedVerified, None),
    )
    .unwrap();
    g.apply_lease(
        &p,
        &lease_event(&g, LeaseOperation::LeaseAcquirePrepared, None),
    )
    .unwrap();
    assert!(
        g.apply_lease(
            &p,
            &lease_event(&g, LeaseOperation::LeaseAcquiredVerified, Some(old))
        )
        .is_err(),
        "older historical epoch must never reappear"
    );
    let raw = encoded(&lease_event(&g, LeaseOperation::LeaseReleasePrepared, None)).unwrap();
    let mut v: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    v["nativeReleaseAuthority"] = true.into();
    assert!(serde_json::from_value::<ChainEntry>(v).is_err());
    println!("r6 same-chain lease data rejects13 context drifts, skip/order/old epoch/object digest and node mutation while prepared/released; data transition never constructs release receipt/native authority");
    f.retain();
}
