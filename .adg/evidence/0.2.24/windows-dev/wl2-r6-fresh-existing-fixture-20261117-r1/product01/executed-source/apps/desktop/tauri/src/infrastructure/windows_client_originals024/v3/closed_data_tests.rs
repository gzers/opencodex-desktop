use super::super::{
    handles::HandleOperation, history::tests::linked, segments::ChainEntry, tests::plan,
};
use super::*;
use crate::infrastructure::windows_client_originals024::handle_release::HandleRow;
use serde_json::{json, Value};
pub(in crate::infrastructure::windows_client_originals024::v3) fn setup(
    p: &StoragePlan,
) -> (Progress, ClosedData) {
    // Pure canonical DATA only. No fake Owner/Stop/NormalExit/Absent/native
    // assessment/completion constructor is used or made available to tests.
    let mut g = linked(p);
    g.content = Some(qualification::ContentSummary {
        digest: "e".repeat(64),
        verified: true,
    });
    g.ready_generation = Some((123, 456, "a".repeat(64)));
    let rows = p
        .roots
        .iter()
        .enumerate()
        .flat_map(|(root, tree)| {
            tree.nodes.iter().enumerate().flat_map(move |(i, n)| {
                ["original", "history-source"].map(move |role| HandleRow {
                    key: format!("tree/{root}/{role}/{i:04}"),
                    volume: n.volume,
                    file_id: n.file_id.clone(),
                    directory: n.kind == Kind::DirectoryAnchor,
                    historical_work: false,
                })
            })
        })
        .collect();
    let e = g
        .handle_event(HandleOperation::OriginalHandlesReleaseIntent, Some(rows))
        .unwrap();
    g.apply_entry(p, &ChainEntry::Handles(e)).unwrap();
    let e = g
        .handle_event(HandleOperation::OriginalHandlesReleasedVerified, None)
        .unwrap();
    g.apply_entry(p, &ChainEntry::Handles(e)).unwrap();
    let handles = g.handle_progress.as_ref().unwrap();
    let data = ClosedData {
        route_close_digest: super::super::route_close::tests::data(p).digest().unwrap(),
        close_sequence: g.sequence,
        close_event_digest: g.tip.clone(),
        close_current_digest: "c".repeat(64),
        native_close_digest: "b".repeat(64),
        content_digest: g.content.as_ref().unwrap().digest.clone(),
        original_history_digest: hash::sha256_hex(
            &encoded(&g.history_summary().unwrap().unwrap()).unwrap(),
        ),
        inventory_digest: handles.inventory_digest.clone(),
        handle_count: handles.count,
        ready: g.ready_generation.clone().unwrap(),
    };
    (g, data)
}
fn event(g: &Progress, d: &ClosedData, op: ClosedOperation) -> ClosedEvent {
    g.closed_event(op, serde_json::from_slice(&encoded(d).unwrap()).unwrap())
        .unwrap()
}
fn reject(p: &StoragePlan, g: &Progress, v: Value) {
    let before = encoded(g).unwrap();
    let mut trial: Progress = serde_json::from_slice(&before).unwrap();
    match serde_json::from_value::<ChainEntry>(v) {
        Ok(e) => assert!(trial.apply_entry(p, &e).is_err()),
        Err(_) => {}
    }
    assert_eq!(
        before,
        encoded(&trial).unwrap(),
        "complete DATA state must remain unchanged on invalid closure"
    );
}
#[test]
fn wl2_r6_closed_data_original_close_parent_and_exact_qualification_history_intent_result_required()
{
    let (mut f, p, _, _) = plan();
    let (mut g, d) = setup(&p);
    reject(
        &p,
        &g,
        serde_json::to_value(event(&g, &d, ClosedOperation::StopClosedVerified)).unwrap(),
    );
    let e = event(&g, &d, ClosedOperation::StopClosureIntent);
    g.apply_entry(&p, &ChainEntry::Closed(e)).unwrap();
    assert!(!g.closed.as_ref().unwrap().verified);
    reject(
        &p,
        &g,
        serde_json::to_value(event(&g, &d, ClosedOperation::StopClosedVerified)).unwrap(),
    );
    super::super::route_close::tests::qualify(&mut g, &p);
    for field in [
        "nativeCloseDigest",
        "closeCurrentDigest",
        "contentDigest",
        "inventoryDigest",
    ] {
        let mut v =
            serde_json::to_value(event(&g, &d, ClosedOperation::StopClosedVerified)).unwrap();
        v["closed"][field] = "f".repeat(64).into();
        reject(&p, &g, v);
    }
    let e = event(&g, &d, ClosedOperation::StopClosedVerified);
    let bytes = encoded(&e).unwrap();
    let decoded: ChainEntry = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(bytes, encoded(&decoded).unwrap());
    g.apply_entry(&p, &decoded).unwrap();
    assert!(g.closed.as_ref().unwrap().verified);
    assert!(g.closed.as_ref().unwrap().digest == d.digest().unwrap());
    reject(
        &p,
        &g,
        serde_json::to_value(event(&g, &d, ClosedOperation::StopClosedVerified)).unwrap(),
    );
    let lease = segments::LeaseEvent {
        schema: "windows-client-originals-v3".into(),
        document_kind: "lease-event".into(),
        storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
        event_id: uuid::Uuid::new_v4().to_string(),
        sequence: g.sequence + 1,
        session_id: g.session.clone(),
        operation: segments::LeaseOperation::LeaseAcquirePrepared,
        lease_epoch: g.epoch.clone(),
        next_epoch: None,
        claim_key: g.claim.clone(),
        objects_digest: g.lease_objects.clone().unwrap(),
        deadline_scope: g.deadline.clone(),
        previous_event_digest: g.tip.clone(),
        storage_plan_digest: g.plan.clone(),
        security_policy_digest: Policy::fixed().digest().unwrap(),
        manifest_digest: g.manifest.clone(),
        binding_digest: g.binding.clone(),
    };
    assert!(g.apply_entry(&p, &ChainEntry::Lease(lease)).is_err());
    println!("r6 final closure DATA: original actual-close locator parent required; verified cannot precede intent or substitute original current/native/content/history/handle digest, terminal verified chain cannot reopen; typed Stop/native completion NEVER constructed from DATA");
    f.retain();
}
#[test]
fn wl2_r6_closed_data_context_missing_stage_and_text_native_completion_refuse_atomically() {
    let (mut f, p, _, _) = plan();
    let (g, d) = setup(&p);
    let good = serde_json::to_value(event(&g, &d, ClosedOperation::StopClosureIntent)).unwrap();
    for field in [
        "sessionId",
        "eventId",
        "leaseEpoch",
        "deadlineScope",
        "claimKey",
        "manifestDigest",
        "bindingDigest",
        "storagePlanDigest",
        "securityPolicyDigest",
        "previousEventDigest",
    ] {
        let mut v = good.clone();
        v[field] = "foreign".into();
        reject(&p, &g, v);
    }
    for (pointer, value) in [
        ("/closed/closeSequence", json!(g.sequence - 1)),
        ("/closed/closeEventDigest", json!("f".repeat(64))),
        ("/closed/closeCurrentDigest", json!("unknown")),
        ("/closed/nativeCloseDigest", json!("unknown")),
        ("/closed/handleCount", json!(d.handle_count + 1)),
        ("/closed/ready/0", json!(124)),
        ("/closed/ready/1", json!(457)),
        ("/closed/ready/2", json!("f".repeat(64))),
        ("/closed/originalHistoryDigest", json!("f".repeat(64))),
    ] {
        let mut v = good.clone();
        *v.pointer_mut(pointer).unwrap() = value;
        reject(&p, &g, v);
    }
    for field in [
        "normalExit",
        "supervisorAbsent",
        "nativeAuthority",
        "typedClosed",
    ] {
        let mut v = good.clone();
        v["closed"][field] = true.into();
        reject(&p, &g, v);
    }
    for kind in 0..6 {
        let mut t: Progress = serde_json::from_slice(&encoded(&g).unwrap()).unwrap();
        match kind {
            0 => t.content = None,
            1 => t.content.as_mut().unwrap().verified = false,
            2 => t.handle_progress.as_mut().unwrap().released = false,
            3 => t.handle_close = None,
            4 => t.lifecycle = snapshots::LifecyclePhase::TeardownPrepared,
            _ => t.action_kind = actions::ActionKind::Start,
        }
        reject(
            &p,
            &t,
            serde_json::to_value(event(&t, &d, ClosedOperation::StopClosureIntent)).unwrap(),
        );
    }
    let raw = String::from_utf8(encoded(&good).unwrap()).unwrap();
    let duplicate = raw.replacen(
        "\"nativeCloseDigest\":",
        "\"nativeCloseDigest\":\"foreign\",\"nativeCloseDigest\":",
        1,
    );
    assert!(serde_json::from_str::<ChainEntry>(&duplicate).is_err());
    println!("r6 final closure DATA strict scopes/close parent/ready/history/count/qualification/completed handles/stopped phase required; unknown native flags and duplicate receipt digest reject without any progress change");
    f.retain();
}
