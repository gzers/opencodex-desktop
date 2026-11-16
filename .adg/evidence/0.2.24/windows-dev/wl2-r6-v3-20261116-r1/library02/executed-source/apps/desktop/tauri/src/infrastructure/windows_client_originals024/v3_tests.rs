use super::*;
pub(super) fn plan() -> (
    super::super::tree::tests::Fixture,
    StoragePlan,
    std::path::PathBuf,
    FileIdentity,
) {
    let (f, roots, profile, parent) = super::super::tree::r6_test_plan();
    let p = StoragePlan::from_native_data(roots, &profile, parent).unwrap();
    (f, p, profile, parent)
}
#[test]
fn wl2_r6_v3_full_native_plan_rejects_missing_duplicate_extra_anchor_wrong_parent_and_unknown_fields(
) {
    let (mut f, p, profile, parent) = plan();
    let original = serde_json::to_value(&p).unwrap();
    let mut cases = Vec::new();
    let mut v = original.clone();
    v["storageStrategy"] = "windows-client-originals-v2".into();
    cases.push(v);
    let mut v = original.clone();
    v["roots"][0]["nodes"].as_array_mut().unwrap().pop();
    cases.push(v);
    let mut v = original.clone();
    let n = v["roots"][0]["nodes"][1].clone();
    v["roots"][0]["nodes"].as_array_mut().unwrap().push(n);
    cases.push(v);
    let mut v = original.clone();
    v["roots"][1]["nodes"][1]["kind"] = "CoordinatorAnchor".into();
    v["roots"][1]["nodes"][1]["target"] = serde_json::Value::Null;
    cases.push(v);
    let mut v = original.clone();
    v["roots"][0]["nodes"][0]["parentId"] = "999".into();
    cases.push(v);
    let mut v = original.clone();
    v["roots"][0]["nodes"][0]["children"]
        .as_array_mut()
        .unwrap()
        .push("foreign-empty".into());
    cases.push(v);
    let mut v = original.clone();
    v["roots"][0]["nodes"][0]["security"]["fullSecurityProven"] = true.into();
    cases.push(v);
    let mut v = original.clone();
    v["roots"][0]["nodes"][0]["relative"] = "../escape".into();
    cases.push(v);
    let mut v = original.clone();
    v["roots"][0]["nodes"][0]["extraAuthority"] = true.into();
    cases.push(v);
    let mut v = original.clone();
    v["roots"][0]["custodyVolume"] = (parent.volume.wrapping_add(1)).into();
    cases.push(v);
    for (i, v) in cases.iter().enumerate() {
        let rejected = serde_json::from_value::<StoragePlan>(v.clone())
            .map(|p| p.validate(&profile, parent).is_err())
            .unwrap_or(true);
        assert!(rejected, "bad plan {i} must refuse");
    }
    let duplicate = String::from_utf8(encoded(&p).unwrap()).unwrap().replacen(
        "\"storageStrategy\":",
        "\"storageStrategy\":\"wrong\",\"storageStrategy\":",
        1,
    );
    assert!(serde_json::from_str::<StoragePlan>(&duplicate).is_err());
    println!("r6 full native-derived v3 plan data validates two complete trees/actual IDs/all8/parent and same-volume custody;10 malformed plans plus duplicate fields refuse; parsing never admits native authority");
    f.retain();
}
fn progress(p: &StoragePlan) -> Progress {
    Progress::new(
        p,
        &uuid::Uuid::new_v4().to_string(),
        &"a".repeat(64),
        &"b".repeat(64),
        &uuid::Uuid::new_v4().to_string(),
        &"c".repeat(64),
        &uuid::Uuid::new_v4().to_string(),
    )
    .unwrap()
}
pub(super) fn event(p: &StoragePlan, g: &Progress, n: &Node, op: Operation, work: &str) -> Event {
    let root = p
        .roots
        .iter()
        .find(|r| r.root.ends_with(&n.root_id))
        .unwrap();
    let path = if n.relative.is_empty() {
        n.root_id.clone()
    } else {
        format!("{}/{}", n.root_id, n.relative)
    };
    let custody = format!("custody/{}/original-{}", n.root_id, n.node_id);
    let retired = format!("retired/{}/retired-{}", n.root_id, n.node_id);
    let (from, to, parent) = match op {
        Operation::NodeMoveIntent | Operation::NodeMovedVerified => {
            (path.clone(), custody.clone(), root.custody_id.clone())
        }
        Operation::WorkCreateIntent | Operation::WorkCreatedVerified => {
            ("Absent".into(), path.clone(), n.parent_id.clone())
        }
        Operation::WorkRetireIntent | Operation::WorkRetiredVerified => {
            (path.clone(), retired.clone(), root.custody_id.clone())
        }
        Operation::OriginalRestoreIntent | Operation::OriginalRestoredVerified => {
            (custody.clone(), path.clone(), n.parent_id.clone())
        }
        _ => (path.clone(), path, n.parent_id.clone()),
    };
    Event {
        schema:
            crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3
                .into(),
        document_kind: "node-event".into(),
        storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
        event_id: uuid::Uuid::new_v4().to_string(),
        sequence: g.sequence + 1,
        session_id: g.session.clone(),
        node_id: n.node_id.clone(),
        root_id: n.root_id.clone(),
        operation: op,
        from,
        to,
        native_id: if op == Operation::WorkCreateIntent {
            None
        } else if matches!(
            op,
            Operation::WorkCreatedVerified
                | Operation::WorkRetireIntent
                | Operation::WorkRetiredVerified
        ) {
            Some(work.into())
        } else {
            Some(n.file_id.clone())
        },
        parent_id: parent,
        expected_before: "d".repeat(64),
        expected_after: "e".repeat(64),
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
fn wl2_r6_v3_node_events_reject_skip_reverse_context_drift_cross_node_and_text_closed() {
    let (mut f, p, _, _) = plan();
    let n = p.roots[0]
        .nodes
        .iter()
        .find(|n| n.kind == Kind::CustodiedFile)
        .unwrap();
    let mut g = progress(&p);
    assert!(!g.complete_files(&p));
    assert!(g
        .apply(&p, &event(&p, &g, n, Operation::NodeMovedVerified, "999"))
        .is_err());
    let good = event(&p, &g, n, Operation::NodeMoveIntent, "999");
    for field in [
        "sequence",
        "sessionId",
        "claimKey",
        "leaseEpoch",
        "deadlineScope",
        "parentId",
        "previousEventDigest",
        "storagePlanDigest",
        "securityPolicyDigest",
        "manifestDigest",
        "bindingDigest",
        "from",
        "to",
        "rootId",
    ] {
        let mut v = serde_json::to_value(&good).unwrap();
        v[field] = if field == "sequence" {
            99.into()
        } else {
            "invalid".into()
        };
        let bad: Event = serde_json::from_value(v).unwrap();
        assert!(g.apply(&p, &bad).is_err(), "drift {field}");
    }
    let mut v = serde_json::to_value(&good).unwrap();
    v["operation"] = "Closed".into();
    assert!(serde_json::from_value::<Event>(v).is_err());
    g.apply(&p, &good).unwrap();
    assert!(!g.complete_files(&p));
    assert!(g.apply(&p, &good).is_err());
    let other = p.roots[0]
        .nodes
        .iter()
        .find(|x| x.kind == Kind::CustodiedFile && x.node_id != n.node_id)
        .unwrap();
    assert!(g
        .apply(&p, &event(&p, &g, other, Operation::NodeMoveIntent, "998"))
        .is_err());
    let mut bad = event(&p, &g, n, Operation::NodeMovedVerified, "999");
    bad.expected_after = "f".repeat(64);
    assert!(g.apply(&p, &bad).is_err());
    g.apply(&p, &event(&p, &g, n, Operation::NodeMovedVerified, "999"))
        .unwrap();
    assert!(
        !g.complete_files(&p),
        "moved originals cannot count as restored anchors"
    );
    println!("r6 v3 pending blocks cross-node, skip/reverse/replay/context/claim/deadline/expected digest drift and text Closed; data has no native resume factory");
    f.retain();
}
#[test]
fn wl2_r6_v3_every_node_requires_full_order_and_anchors_require_release() {
    let (mut f, p, _, _) = plan();
    let mut g = progress(&p);
    for n in p.roots.iter().flat_map(|r| &r.nodes) {
        if n.kind != Kind::CustodiedFile {
            g.apply(&p, &event(&p, &g, n, Operation::SnapshotVerified, "999"))
                .unwrap();
        }
    }
    assert!(!g.complete_files(&p));
    for (i, n) in p
        .roots
        .iter()
        .flat_map(|r| &r.nodes)
        .filter(|n| n.kind == Kind::CustodiedFile)
        .enumerate()
    {
        let work = (u64::MAX - i as u64).to_string();
        for op in [
            Operation::NodeMoveIntent,
            Operation::NodeMovedVerified,
            Operation::WorkCreateIntent,
            Operation::WorkCreatedVerified,
            Operation::WorkRetireIntent,
            Operation::WorkRetiredVerified,
            Operation::OriginalRestoreIntent,
            Operation::OriginalRestoredVerified,
        ] {
            let e = event(&p, &g, n, op, &work);
            g.apply(&p, &e).unwrap();
            assert!(!g.complete_files(&p));
        }
    }
    for n in p
        .roots
        .iter()
        .flat_map(|r| &r.nodes)
        .filter(|n| n.kind != Kind::CustodiedFile)
    {
        g.apply(&p, &event(&p, &g, n, Operation::ReleaseVerified, "999"))
            .unwrap();
    }
    assert!(g.complete_files(&p));
    println!("r6 v3 complete DATA requires all8 ordered events per file plus every anchor snapshot/release; no native Restored or Closed is minted");
    f.retain();
}
