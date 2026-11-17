use super::super::tests::{event, plan, progress};
use super::*;
fn rows(
    p: &StoragePlan,
    g: &Progress,
    kind: BatchKind,
    completion: bool,
    start: usize,
    count: usize,
) -> Vec<BatchRow> {
    let nodes = g.batch_expected_nodes(p, kind).unwrap();
    nodes
        .into_iter()
        .enumerate()
        .skip(start)
        .take(count)
        .map(|(ordinal, (_, id))| {
            let n = p
                .roots
                .iter()
                .flat_map(|r| &r.nodes)
                .find(|n| n.node_id == id)
                .unwrap();
            let (intent, result) = kind.ordinary().unwrap();
            BatchRow {
                ordinal,
                event: RowEvent::Original(event(
                    p,
                    g,
                    n,
                    if completion { result } else { intent },
                    &(u64::MAX - 10000 - ordinal as u64).to_string(),
                )),
            }
        })
        .collect()
}
fn anchors(p: &StoragePlan, g: &mut Progress) {
    let members = rows(p, g, BatchKind::AnchorSnapshot, true, 0, 512);
    let phase = g
        .batch_event(
            BatchKind::AnchorSnapshot,
            BatchOperation::AnchorsVerified,
            members,
        )
        .unwrap();
    g.apply_entry(p, &segments::ChainEntry::Batch(phase))
        .unwrap();
}
#[test]
fn wl2_r6_batch_data_digest_reuse_only_immutable_plan_bytes_never_serialized_native_facts() {
    use super::super::recovery::RecoveryPlan;
    let (mut f, p, _, _) = plan();
    let first = p.digest().unwrap();
    assert_eq!(p.digest().unwrap(), first);
    let json = serde_json::to_value(&p).unwrap();
    assert!(json.get("dataDigest").is_none());
    let restored: StoragePlan = serde_json::from_value(json.clone()).unwrap();
    assert!(restored.data_digest.get().is_none());
    assert_eq!(restored.digest().unwrap(), first);
    let mut changed = json.clone();
    changed["roots"][0]["nodes"][0]["contentDigest"] = "e".repeat(64).into();
    let changed: StoragePlan = serde_json::from_value(changed).unwrap();
    assert_ne!(changed.digest().unwrap(), first);
    let mut spoof = json;
    spoof["dataDigest"] = first.clone().into();
    assert!(serde_json::from_value::<StoragePlan>(spoof).is_err());
    let (_, data) = super::super::recovery::tests::stopped(&p);
    let recovery: RecoveryPlan = serde_json::from_value(data.clone()).unwrap();
    let digest = recovery.digest().unwrap();
    assert_eq!(recovery.digest().unwrap(), digest);
    let serialized = serde_json::to_value(&recovery).unwrap();
    assert!(serialized.get("dataDigest").is_none());
    let parsed: RecoveryPlan = serde_json::from_value(serialized.clone()).unwrap();
    assert_eq!(parsed.digest().unwrap(), digest);
    let mut changed = serialized.clone();
    changed["roots"][0]["retirements"][0]["sha256"] = "e".repeat(64).into();
    assert_ne!(
        serde_json::from_value::<RecoveryPlan>(changed)
            .unwrap()
            .digest()
            .unwrap(),
        digest
    );
    let mut spoof = serialized;
    spoof["dataDigest"] = digest.into();
    assert!(serde_json::from_value::<RecoveryPlan>(spoof).is_err());
    println!("r6 pure immutable plan digest reuse changes no serialized bytes/native proof: new deserialization starts empty, changed DATA hashes differently, spoofed persisted cache refuses; no native ID/security/content/lease observation is cached");
    f.retain();
}
#[test]
fn wl2_r6_batch_data_current_retirement_before_original_restore_and_exact_recovery_prefixes() {
    use super::super::recovery::{RecoveryOperation, RecoveryPlan};
    let (mut f, p, _, _) = plan();
    let (mut g, data) = super::super::recovery::tests::stopped(&p);
    let recovery: RecoveryPlan = serde_json::from_value(data).unwrap();
    for op in [
        RecoveryOperation::RecoveryPlanIntent,
        RecoveryOperation::RecoveryPlanVerified,
    ] {
        g.apply_recovery(
            &p,
            &g.recovery_event(
                op,
                if op == RecoveryOperation::RecoveryPlanIntent {
                    Some(&recovery)
                } else {
                    None
                },
                None,
            )
            .unwrap(),
        )
        .unwrap();
    }
    let make = |g: &Progress, kind: BatchKind, completion: bool| {
        let nodes = g.batch_expected_nodes(&p, kind).unwrap();
        let ops = kind.recovery().unwrap();
        nodes
            .into_iter()
            .enumerate()
            .map(|(ordinal, (_, id))| BatchRow {
                ordinal,
                event: RowEvent::Recovery(
                    g.recovery_event(if completion { ops.1 } else { ops.0 }, None, Some(&id))
                        .unwrap(),
                ),
            })
            .collect::<Vec<_>>()
    };
    let premature = g
        .batch_event(
            BatchKind::OriginalRestore,
            BatchOperation::Intent,
            make(&g, BatchKind::OriginalRestore, false),
        )
        .unwrap();
    let before = encoded(&g).unwrap();
    assert!(g.apply_batch(&p, &premature).is_err());
    assert_eq!(encoded(&g).unwrap(), before);
    for kind in [BatchKind::CurrentRetire, BatchKind::OriginalRestore] {
        let event = g
            .batch_event(kind, BatchOperation::Intent, make(&g, kind, false))
            .unwrap();
        g.apply_batch(&p, &event).unwrap();
        assert_eq!(g.batch_summary().unwrap().prefix, 0);
        let mut too_many = g
            .batch_event(kind, BatchOperation::PrefixVerified, make(&g, kind, true))
            .unwrap();
        too_many.rows.pop();
        let before = encoded(&g).unwrap();
        assert!(g.apply_batch(&p, &too_many).is_err());
        assert_eq!(encoded(&g).unwrap(), before);
        let event = g
            .batch_event(kind, BatchOperation::PrefixVerified, make(&g, kind, true))
            .unwrap();
        g.apply_batch(&p, &event).unwrap();
        assert!(g.batch.is_none());
        assert!(!g.complete_files(&p));
    }
    for n in p
        .roots
        .iter()
        .flat_map(|r| &r.nodes)
        .filter(|n| n.kind == Kind::CustodiedFile)
    {
        assert_eq!(g.nodes[&n.node_id].0, 8);
    }
    println!("r6 batch recovery DATA uses exact stopped current plan and original rows; early restore/truncated count claims refuse atomically, exact full retired then original prefixes advance without anchor/Closed grant");
    f.retain();
}
#[test]
fn wl2_r6_batch_data_one_phase_complete_intents_ordered_result_prefix_and_old_bypass_refuse() {
    let (mut f, p, _, _) = plan();
    let mut g = progress(&p);
    anchors(&p, &mut g);
    assert_eq!(g.sequence, 1);
    let count = g
        .batch_expected_nodes(&p, BatchKind::OriginalMove)
        .unwrap()
        .len();
    assert!(count >= 2);
    let members = rows(&p, &g, BatchKind::OriginalMove, false, 0, count);
    let phase = g
        .batch_event(BatchKind::OriginalMove, BatchOperation::Intent, members)
        .unwrap();
    g.apply_batch(&p, &phase).unwrap();
    assert_eq!(g.sequence, 2);
    assert!(g.batch_summary().is_some());
    assert!(!g.complete_files(&p));
    let n = p
        .roots
        .iter()
        .flat_map(|r| &r.nodes)
        .find(|n| n.kind == Kind::CustodiedFile)
        .unwrap();
    assert!(g
        .apply(&p, &event(&p, &g, n, Operation::NodeMovedVerified, "999"))
        .is_err());
    let pending = g.batch_summary().unwrap();
    assert_eq!(pending.prefix, 0);
    let first = rows(&p, &g, BatchKind::OriginalMove, true, 0, 1);
    let phase = g
        .batch_event(
            BatchKind::OriginalMove,
            BatchOperation::PrefixVerified,
            first,
        )
        .unwrap();
    g.apply_batch(&p, &phase).unwrap();
    assert_eq!(g.sequence, 3);
    assert_eq!(g.batch_summary().unwrap().prefix, 1);
    let mut skip = rows(&p, &g, BatchKind::OriginalMove, true, 1, count - 1);
    skip[0].ordinal = 0;
    let phase = g
        .batch_event(
            BatchKind::OriginalMove,
            BatchOperation::PrefixVerified,
            skip,
        )
        .unwrap();
    let old = encoded(&g).unwrap();
    assert!(g.apply_batch(&p, &phase).is_err());
    assert_eq!(encoded(&g).unwrap(), old);
    let remaining = rows(&p, &g, BatchKind::OriginalMove, true, 1, count - 1);
    let phase = g
        .batch_event(
            BatchKind::OriginalMove,
            BatchOperation::PrefixVerified,
            remaining,
        )
        .unwrap();
    g.apply_batch(&p, &phase).unwrap();
    assert_eq!(g.sequence, 4);
    assert!(g.batch.is_none());
    for n in p
        .roots
        .iter()
        .flat_map(|r| &r.nodes)
        .filter(|n| n.kind == Kind::CustodiedFile)
    {
        assert_eq!(g.nodes[&n.node_id].0, 2);
    }
    let members = rows(&p, &g, BatchKind::WorkCreate, false, 0, count);
    let phase = g
        .batch_event(BatchKind::WorkCreate, BatchOperation::Intent, members)
        .unwrap();
    g.apply_batch(&p, &phase).unwrap();
    let mut alias = rows(&p, &g, BatchKind::WorkCreate, true, 0, count);
    let RowEvent::Original(first) = &mut alias[0].event else {
        panic!()
    };
    let identity = p.roots[0].custody_id.clone();
    first.native_id = Some(identity.clone());
    let mut security = serde_json::to_value(first.native_security.as_ref().unwrap()).unwrap();
    security["fileId"] = identity.into();
    first.native_security = Some(serde_json::from_value(security).unwrap());
    let alias = g
        .batch_event(BatchKind::WorkCreate, BatchOperation::PrefixVerified, alias)
        .unwrap();
    let before = encoded(&g).unwrap();
    assert!(
        g.apply_batch(&p, &alias).is_err(),
        "work generation must not alias original/custody/parent/snapshot identities"
    );
    assert_eq!(encoded(&g).unwrap(), before);
    let members = rows(&p, &g, BatchKind::WorkCreate, true, 0, count);
    let phase = g
        .batch_event(
            BatchKind::WorkCreate,
            BatchOperation::PrefixVerified,
            members,
        )
        .unwrap();
    g.apply_batch(&p, &phase).unwrap();
    assert_eq!(g.sequence, 6);
    assert!(g.batch.is_none());
    for n in p
        .roots
        .iter()
        .flat_map(|r| &r.nodes)
        .filter(|n| n.kind == Kind::CustodiedFile)
    {
        assert_eq!(g.nodes[&n.node_id].0, 4);
    }
    println!("r6 bounded batch DATA exactly one sequence per envelope, complete anchor/original/work rows, strict ordered prefix and pending blocks old node path; pure projection uses unchanged validators, no native authority");
    f.retain();
}
#[test]
fn wl2_r6_batch_data_complete_plan_context_ordinal_event_drift_duplicate_and_nested_rows_refuse_atomically(
) {
    let (mut f, p, _, _) = plan();
    let mut g = progress(&p);
    anchors(&p, &mut g);
    let members = rows(&p, &g, BatchKind::OriginalMove, false, 0, 512);
    let phase = g
        .batch_event(BatchKind::OriginalMove, BatchOperation::Intent, members)
        .unwrap();
    let good = serde_json::to_value(&phase).unwrap();
    let original = encoded(&g).unwrap();
    let mut cases = Vec::new();
    for field in [
        "sequence",
        "sessionId",
        "previousEventDigest",
        "batchId",
        "planDigest",
        "leaseEpoch",
        "claimKey",
        "deadlineScope",
        "manifestDigest",
        "bindingDigest",
        "storagePlanDigest",
        "securityPolicyDigest",
        "eventId",
        "count",
        "prefix",
    ] {
        let mut bad = good.clone();
        bad[field] = if matches!(field, "sequence" | "count" | "prefix") {
            9999.into()
        } else {
            "foreign".into()
        };
        cases.push(bad);
    }
    for pointer in [
        "/rows/0/ordinal",
        "/rows/0/event/nodeId",
        "/rows/0/event/rootId",
        "/rows/0/event/previousEventDigest",
        "/rows/0/event/sequence",
        "/rows/0/event/claimKey",
    ] {
        let mut bad = good.clone();
        *bad.pointer_mut(pointer).unwrap() =
            if pointer.ends_with("ordinal") || pointer.ends_with("sequence") {
                9999.into()
            } else {
                "foreign".into()
            };
        cases.push(bad);
    }
    let mut bad = good.clone();
    bad["rows"].as_array_mut().unwrap().pop();
    cases.push(bad);
    let mut bad = good.clone();
    bad["rows"].as_array_mut().unwrap().swap(0, 1);
    cases.push(bad);
    let mut bad = good.clone();
    bad["rows"][1]["event"]["eventId"] = bad["rows"][0]["event"]["eventId"].clone();
    cases.push(bad);
    let mut bad = good.clone();
    bad["rows"][0]["event"] = good.clone();
    cases.push(bad);
    let mut bad = good.clone();
    bad["Closed"] = true.into();
    cases.push(bad);
    for (i, value) in cases.into_iter().enumerate() {
        let refused = serde_json::from_value::<BatchEvent>(value)
            .map(|e| g.apply_batch(&p, &e).is_err())
            .unwrap_or(true);
        assert!(refused, "malformed {i}");
        assert_eq!(
            encoded(&g).unwrap(),
            original,
            "phase validation must be atomic {i}"
        );
    }
    // Recompute batch digest for a structurally self-consistent wrong late row;
    // original single-node native ID/security/parent validator must still reject.
    let mut late: BatchEvent = serde_json::from_value(good).unwrap();
    let RowEvent::Original(last) = &mut late.rows.last_mut().unwrap().event else {
        panic!()
    };
    last.parent_id = "999".into();
    late.plan_digest = hash::sha256_hex(&encoded(&late.rows).unwrap());
    assert!(g.apply_batch(&p, &late).is_err());
    assert_eq!(encoded(&g).unwrap(), original);
    println!("r6 bounded batch DATA full plan/context/row ordinal/node-root/duplicate/nesting/late-row native parent drift all reject atomically, including recomputed plan digest; no skip or textClosed");
    f.retain();
}
