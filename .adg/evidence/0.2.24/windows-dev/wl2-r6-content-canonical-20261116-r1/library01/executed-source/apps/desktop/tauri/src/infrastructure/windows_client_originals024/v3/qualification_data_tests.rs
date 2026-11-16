use super::super::{
    recovery::{RecoveryOperation, RecoveryPlan},
    segments::ChainEntry,
};
use super::*;
use serde_json::{json, Value};
fn copy(g: &Progress) -> Progress {
    serde_json::from_slice(&encoded(g).unwrap()).unwrap()
}
fn setup(p: &StoragePlan) -> (Progress, ContentData, RecoveryPlan) {
    // Deliberately DATA-only stages. No actual native assessment/typed Stop
    // factory, native scope or restoration capability is constructed here.
    let (mut g, recovery) = super::super::recovery::tests::stopped(p);
    g.observed_sequence = 3;
    let frames: [[String; 2]; 3] = std::array::from_fn(|i| {
        if i == 2 {
            g.observed_frames.clone().unwrap()
        } else {
            [format!("{i}").repeat(64), format!("{}", i + 4).repeat(64)]
        }
    });
    g.linked_frames.clear();
    let start_scope = uuid::Uuid::new_v4().to_string();
    g.seen_actions.insert(start_scope.clone());
    let purposes = [
        SnapshotPurpose::Ready,
        SnapshotPurpose::PreStop,
        SnapshotPurpose::Stopped,
    ];
    let bindings = std::array::from_fn(|i| {
        json!({"sessionId":g.session,"storagePlanDigest":g.plan,
        "bindingDigest":g.binding,"manifestDigest":g.manifest,"deadlineScope":if i==0 {&start_scope} else {&g.deadline},
        "frameDigests":frames[i],"purpose":purposes[i],"observationSequence":i+1})
    });
    let mut recovery = recovery;
    for i in 0..2 {
        recovery["roots"][i]["currentSnapshot"]["context"] = bindings[2].clone();
    }
    g.snapshot_links.clear();
    for (i, purpose) in purposes.into_iter().enumerate() {
        g.linked_frames.insert((i + 1) as u32, frames[i].clone());
        g.snapshot_stages.insert(
            purpose,
            (
                (i + 1) as u32,
                hash::sha256_hex(&encoded(&bindings[i]).unwrap()),
            ),
        );
        let hashes = if i == 2 {
            std::array::from_fn(|j| {
                hash::sha256_hex(&encoded(&recovery["roots"][j]["currentSnapshot"]).unwrap())
            })
        } else {
            ["a".repeat(64), "b".repeat(64)]
        };
        g.snapshot_links.insert(purpose, hashes);
    }
    let series_digest = hash::sha256_hex(
        &encoded(&json!({"sequences":[1,2,3],"bindings":bindings,"frames":frames})).unwrap(),
    );
    let bindings = bindings.map(|v| serde_json::from_value(v).unwrap());
    let data = ContentData {
        scope_digest: "c".repeat(64),
        series_digest,
        sequences: [1, 2, 3],
        bindings,
        frames,
        actions: [
            ContentAction::AlreadyOriginal,
            ContentAction::RestoreOriginal,
            ContentAction::AlreadyOriginal,
        ],
    };
    (g, data, serde_json::from_value(recovery).unwrap())
}
fn event(g: &Progress, data: &ContentData, operation: ContentOperation) -> ContentEvent {
    ContentEvent {
        schema: "windows-client-originals-v3".into(),
        document_kind: "content-qualification-event".into(),
        storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
        event_id: uuid::Uuid::new_v4().to_string(),
        sequence: g.sequence + 1,
        session_id: g.session.clone(),
        operation,
        content: serde_json::from_slice(&encoded(data).unwrap()).unwrap(),
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
fn assert_rejected(p: &StoragePlan, g: &Progress, value: Value) {
    let mut trial = copy(g);
    let before = encoded(&trial).unwrap();
    match serde_json::from_value::<ChainEntry>(value) {
        Ok(e) => assert!(trial.apply_entry(p, &e).is_err()),
        Err(_) => {}
    }
    assert_eq!(
        encoded(&trial).unwrap(),
        before,
        "bad qualification must be atomic DATA refusal"
    );
}
#[test]
fn wl2_r6_content_chain_pending_blocks_other_operations_and_result_binds_exact_data() {
    let (mut f, p, _, _) = super::super::tests::plan();
    let (mut g, data, r) = setup(&p);
    let intent = event(&g, &data, ContentOperation::ContentQualificationIntent);
    let result_without_intent = event(&g, &data, ContentOperation::ContentQualifiedVerified);
    assert_rejected(
        &p,
        &g,
        serde_json::to_value(&result_without_intent).unwrap(),
    );
    // Establish the same recovery DATA event WOULD be valid without pending.
    let mut otherwise = copy(&g);
    let e = otherwise
        .recovery_event(RecoveryOperation::RecoveryPlanIntent, Some(&r), None)
        .unwrap();
    otherwise.apply_entry(&p, &ChainEntry::Recovery(e)).unwrap();
    g.apply_entry(&p, &ChainEntry::Content(intent)).unwrap();
    let pending = encoded(&g).unwrap();
    let e = g
        .recovery_event(RecoveryOperation::RecoveryPlanIntent, Some(&r), None)
        .unwrap();
    assert!(g.apply_entry(&p, &ChainEntry::Recovery(e)).is_err());
    assert_eq!(encoded(&g).unwrap(), pending);
    let mut wrong =
        serde_json::to_value(event(&g, &data, ContentOperation::ContentQualifiedVerified)).unwrap();
    wrong["content"]["actions"][1] = "AlreadyOriginal".into();
    assert_rejected(&p, &g, wrong);
    let e = event(&g, &data, ContentOperation::ContentQualifiedVerified);
    let exact = encoded(&e).unwrap();
    let parsed: ChainEntry = serde_json::from_slice(&exact).unwrap();
    assert_eq!(encoded(&parsed).unwrap(), exact);
    g.apply_entry(&p, &parsed).unwrap();
    assert_eq!(
        g.content,
        Some(ContentSummary {
            digest: data.digest().unwrap(),
            verified: true
        })
    );
    assert_rejected(
        &p,
        &g,
        serde_json::to_value(event(&g, &data, ContentOperation::ContentQualifiedVerified)).unwrap(),
    );
    let e = g
        .recovery_event(RecoveryOperation::RecoveryPlanIntent, Some(&r), None)
        .unwrap();
    g.apply_entry(&p, &ChainEntry::Recovery(e)).unwrap();
    println!("r6 content canonical DATA: intent blocks otherwise-valid recovery; result must match exact original content/actions; strict event roundtrip; verified recovery DATA accepted, no native restoration authority");
    f.retain();
}
#[test]
fn wl2_r6_content_chain_unknown_duplicate_and_cross_scope_stage_data_refuse_atomically() {
    let (mut f, p, _, _) = super::super::tests::plan();
    let (g, data, _) = setup(&p);
    let good = serde_json::to_value(event(
        &g,
        &data,
        ContentOperation::ContentQualificationIntent,
    ))
    .unwrap();
    let mut cases = Vec::new();
    for field in ["sessionId", "leaseEpoch", "deadlineScope"] {
        let mut v = good.clone();
        v[field] = uuid::Uuid::new_v4().to_string().into();
        cases.push(v);
    }
    for field in [
        "claimKey",
        "previousEventDigest",
        "storagePlanDigest",
        "securityPolicyDigest",
        "manifestDigest",
        "bindingDigest",
    ] {
        let mut v = good.clone();
        v[field] = "f".repeat(64).into();
        cases.push(v);
    }
    for field in ["scopeDigest", "seriesDigest"] {
        let mut v = good.clone();
        v["content"][field] = "unknown".into();
        cases.push(v);
    }
    let mut v = good.clone();
    v["content"]["actions"][0] = "Unknown".into();
    cases.push(v);
    let mut v = good.clone();
    v["content"]["runtimeAuthority"] = true.into();
    cases.push(v);
    let mut v = good.clone();
    v["typedClosed"] = true.into();
    cases.push(v);
    // Recompute self-consistent series hash, so rejection proves the existing
    // canonical phase linkage rather than a stale hash alone.
    for field in [
        "deadlineScope",
        "purpose",
        "observationSequence",
        "sessionId",
        "extra",
    ] {
        let mut v = good.clone();
        v["content"]["bindings"][0][field] = if field == "observationSequence" {
            json!(3)
        } else {
            json!("Stopped")
        };
        let c = &v["content"];
        v["content"]["seriesDigest"] = hash::sha256_hex(
            &encoded(
                &json!({"sequences":c["sequences"],"bindings":c["bindings"],"frames":c["frames"]}),
            )
            .unwrap(),
        )
        .into();
        cases.push(v);
    }
    for v in cases {
        assert_rejected(&p, &g, v);
    }
    let raw = String::from_utf8(encoded(&good).unwrap()).unwrap();
    let duplicate = raw.replacen(
        "\"scopeDigest\":",
        "\"scopeDigest\":\"unknown\",\"scopeDigest\":",
        1,
    );
    assert!(serde_json::from_str::<ChainEntry>(&duplicate).is_err());
    println!("r6 content DATA rejects nineteen unknown/cross-context/self-rehashed stage cases and duplicate fields without changing progress; no parsed native permit");
    f.retain();
}
#[test]
fn wl2_r6_content_chain_missing_stage_or_reordered_frame_and_incomplete_tree_refuse() {
    let (mut f, p, _, _) = super::super::tests::plan();
    let (g, data, _) = setup(&p);
    for purpose in [
        SnapshotPurpose::Ready,
        SnapshotPurpose::PreStop,
        SnapshotPurpose::Stopped,
    ] {
        let mut trial = copy(&g);
        trial.snapshot_stages.remove(&purpose);
        assert_rejected(
            &p,
            &trial,
            serde_json::to_value(event(
                &trial,
                &data,
                ContentOperation::ContentQualificationIntent,
            ))
            .unwrap(),
        );
    }
    for mutation in 0..5 {
        let mut trial = copy(&g);
        match mutation {
            0 => {
                trial.lifecycle = snapshots::LifecyclePhase::TeardownPrepared;
            }
            1 => {
                trial.lease_state = segments::LeaseState::Released;
            }
            2 => {
                trial.linked_frames.get_mut(&1).unwrap().swap(0, 1);
            }
            3 => {
                trial.action_kind = actions::ActionKind::Start;
            }
            _ => {
                trial.nodes.values_mut().next().unwrap().0 = 0;
            }
        }
        assert_rejected(
            &p,
            &trial,
            serde_json::to_value(event(
                &trial,
                &data,
                ContentOperation::ContentQualificationIntent,
            ))
            .unwrap(),
        );
    }
    println!("r6 content DATA requires all three original stage links/order/BOTH-frame digests, actual phase/lease locators and complete original tree progress; no missing subset admitted");
    f.retain();
}
