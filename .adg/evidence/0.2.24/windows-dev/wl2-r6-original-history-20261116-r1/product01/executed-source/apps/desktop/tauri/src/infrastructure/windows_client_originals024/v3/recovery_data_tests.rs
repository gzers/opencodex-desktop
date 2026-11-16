use super::super::{
    snapshots::tests::{observed, refs},
    tests::plan,
};
use super::*;
use serde_json::{json, Value};

// Synthetic DATA derived from real captured native original plans. Deliberately
// no native Stopped/Owner/process/Source authority is constructed by this helper.
fn stopped(p: &StoragePlan) -> (Progress, Value) {
    let mut g = observed(p);
    g.lifecycle = LifecyclePhase::Stopped;
    g.action_kind = actions::ActionKind::Stop;
    let mut frames = Vec::new();
    for (index, r) in p.roots.iter().enumerate() {
        let mut rows = Vec::new();
        for n in &r.nodes {
            let file = g.nodes[&n.node_id].1.as_ref().unwrap_or(&n.file_id);
            let mut security = serde_json::to_value(&n.security).unwrap();
            security["fileId"] = file.as_str().into();
            rows.push(
                json!({"relative":n.relative,"directory":n.kind==Kind::DirectoryAnchor,
                "volume":n.volume,"fileId":file,"security":security,"bytes":n.length,
                "sha256":n.content_digest,"children":n.children}),
            );
        }
        if index == 0 {
            for (i, name) in [
                "models_cache.json",
                "opencodex-catalog-audit.jsonl",
                "opencodex-catalog.json",
            ]
            .iter()
            .enumerate()
            {
                let id = (u64::MAX - 1000 - i as u64).to_string();
                let mut security = serde_json::to_value(&r.nodes[0].security).unwrap();
                security["fileId"] = id.clone().into();
                rows.push(json!({"relative":name,"directory":false,"volume":r.custody_volume,
                    "fileId":id,"security":security,"bytes":0,"sha256":hash::sha256_hex(b""),"children":[]}));
            }
        }
        rows.sort_by(|a, b| a["relative"].as_str().cmp(&b["relative"].as_str()));
        for i in 0..rows.len() {
            if rows[i]["directory"] == true {
                let path = Path::new(rows[i]["relative"].as_str().unwrap());
                let children = rows
                    .iter()
                    .filter(|n| {
                        n["relative"] != ""
                            && Path::new(n["relative"].as_str().unwrap()).parent() == Some(path)
                    })
                    .map(|n| {
                        Path::new(n["relative"].as_str().unwrap())
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string()
                    })
                    .collect::<Vec<_>>();
                rows[i]["children"] = json!(children);
            }
        }
        let mut parent_security = serde_json::to_value(&r.nodes[0].security).unwrap();
        parent_security["fileId"] = r.nodes[0].parent_id.clone().into();
        frames.push(
            json!({"root":r.root,"parentVolume":r.custody_volume,"parentId":r.nodes[0].parent_id,
            "parentSecurity":parent_security,"nodes":rows}),
        );
    }
    g.observed_frames = Some([
        hash::sha256_hex(&encoded(&frames[0]).unwrap()),
        hash::sha256_hex(&encoded(&frames[1]).unwrap()),
    ]);
    g.linked_frames
        .insert(g.observed_sequence, g.observed_frames.clone().unwrap());
    let references = refs(p, &g, SnapshotPurpose::Stopped, g.observed_sequence);
    g.snapshot_links.insert(
        SnapshotPurpose::Stopped,
        references
            .each_ref()
            .map(|r| hash::sha256_hex(&encoded(r).unwrap())),
    );
    let roots: Vec<Value> = p.roots.iter().enumerate().map(|(index,r)| {
        let frame = &frames[index];
        let retirements: Vec<Value> = frame["nodes"].as_array().unwrap().iter().filter(|n| {
            n["directory"] == false && !(index == 0 && ANCHORED_CODEX_STABLE_FILES.contains(&n["relative"].as_str().unwrap()))
        }).map(|n| {
            let path = n["relative"].as_str().unwrap();
            let old = r.nodes.iter().find(|n| n.relative == path);
            let parent = r.nodes.iter().find(|n| n.kind == Kind::DirectoryAnchor && Path::new(path).parent()==Some(Path::new(&n.relative))).unwrap();
            json!({"relative":path,"currentVolume":n["volume"],"currentFileId":n["fileId"],
                "target":Path::new(&r.custody).join(format!("retired-current-{}-{}",key(&r.nodes[0].root_id,path),n["fileId"].as_str().unwrap())),
                "targetParentId":r.custody_id,"sourceParentId":parent.file_id,
                "originalFileId":old.map(|n|&n.file_id),"originalAbsent":old.is_none(),"bytes":n["bytes"],
                "sha256":n["sha256"],"security":n["security"]})
        }).collect();
            json!({"schema":"windows-anchored-native-retirement-data-v3","storageStrategy":ANCHORED_STORAGE_STRATEGY,
            "originalTree":r,"currentTree":frame,"currentSnapshot":references[index],"runtimeAuthority":false,"retirements":retirements,"absentCurrent":[]})
    }).collect();
    (g, json!({"roots":roots}))
}
fn decode(v: &Value) -> RecoveryPlan {
    serde_json::from_value(v.clone()).unwrap()
}
fn rebind(g: &mut Progress, v: &mut Value) {
    let frames: [String; 2] =
        std::array::from_fn(|i| hash::sha256_hex(&encoded(&v["roots"][i]["currentTree"]).unwrap()));
    g.observed_frames = Some(frames.clone());
    g.linked_frames.insert(g.observed_sequence, frames.clone());
    let hashes = std::array::from_fn(|i| {
        v["roots"][i]["currentSnapshot"]["frameDigest"] = frames[i].clone().into();
        v["roots"][i]["currentSnapshot"]["context"]["frameDigests"] = json!(frames);
        let r: SnapshotReference =
            serde_json::from_value(v["roots"][i]["currentSnapshot"].clone()).unwrap();
        hash::sha256_hex(&encoded(&r).unwrap())
    });
    g.snapshot_links.insert(SnapshotPurpose::Stopped, hashes);
}
#[test]
fn wl2_r6_recovery_data_self_consistent_changed_nonartifact_missing_anchor_and_unknown_nodes_refuse(
) {
    let (mut f, p, _, _) = plan();
    let (mut g, good) = stopped(&p);
    let config = good["roots"][0]["currentTree"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|n| n["relative"] == "config.toml")
        .unwrap();
    let row = good["roots"][0]["retirements"]
        .as_array()
        .unwrap()
        .iter()
        .position(|n| n["relative"] == "config.toml")
        .unwrap();
    let mut changed = good.clone();
    changed["roots"][0]["currentTree"]["nodes"][config]["sha256"] =
        hash::sha256_hex(b"foreign config").into();
    changed["roots"][0]["retirements"][row]["sha256"] = hash::sha256_hex(b"foreign config").into();
    rebind(&mut g, &mut changed);
    assert!(
        decode(&changed).validate(&p, &g).is_err(),
        "fresh consistent digest is not nonartifact permission"
    );
    let mut replacement = good.clone();
    let n = &mut replacement["roots"][0]["currentTree"]["nodes"][config];
    n["fileId"] = "123456789".into();
    n["security"]["fileId"] = "123456789".into();
    let n = &mut replacement["roots"][0]["retirements"][row];
    n["currentFileId"] = "123456789".into();
    n["security"]["fileId"] = "123456789".into();
    n["target"] = Path::new(&p.roots[0].custody)
        .join(format!(
            "retired-current-{}-123456789",
            key(".codex", "config.toml")
        ))
        .to_str()
        .unwrap()
        .into();
    rebind(&mut g, &mut replacement);
    assert!(
        decode(&replacement).validate(&p, &g).is_err(),
        "same config bytes/all8 under new ID still unknown"
    );
    let mut missing = good.clone();
    missing["roots"][0]["currentTree"]["nodes"]
        .as_array_mut()
        .unwrap()
        .remove(config);
    missing["roots"][0]["retirements"]
        .as_array_mut()
        .unwrap()
        .remove(row);
    missing["roots"][0]["currentTree"]["nodes"][0]["children"]
        .as_array_mut()
        .unwrap()
        .retain(|n| n != "config.toml");
    let old = p.roots[0]
        .nodes
        .iter()
        .find(|n| n.relative == "config.toml")
        .unwrap();
    missing["roots"][0]["absentCurrent"] =
        json!([{"relative":"config.toml","originalFileId":old.file_id,"parentId":old.parent_id}]);
    rebind(&mut g, &mut missing);
    assert!(
        decode(&missing).validate(&p, &g).is_err(),
        "nonartifact absence cannot become allowed by complete row/digests"
    );
    let mut unknown = good.clone();
    let mut n = unknown["roots"][0]["currentTree"]["nodes"][config].clone();
    n["relative"] = "unknown-empty".into();
    n["directory"] = true.into();
    n["bytes"] = 0.into();
    n["sha256"] = hash::sha256_hex(b"").into();
    n["fileId"] = "123456789".into();
    n["security"]["fileId"] = "123456789".into();
    let nodes = unknown["roots"][0]["currentTree"]["nodes"]
        .as_array_mut()
        .unwrap();
    nodes.push(n);
    nodes.sort_by(|a, b| a["relative"].as_str().cmp(&b["relative"].as_str()));
    let children = nodes[0]["children"].as_array_mut().unwrap();
    children.push("unknown-empty".into());
    children.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
    rebind(&mut g, &mut unknown);
    assert!(
        decode(&unknown).validate(&p, &g).is_err(),
        "self-consistent unknown empty directory refuses"
    );
    println!("r6 recovery DATA self-consistent rewritten frame/archive/context digests do not allow changed/replaced/missing nonartifact or unknown directory; no content policy minted");
    f.retain();
}
fn event(
    g: &Progress,
    op: RecoveryOperation,
    plan: &RecoveryPlan,
    node: Option<&str>,
) -> RecoveryEvent {
    let id = node.map(str::to_string);
    let node_digest = id.as_ref().map(|id| {
        if matches!(
            op,
            RecoveryOperation::WorkRetireIntent | RecoveryOperation::WorkRetiredVerified
        ) {
            let (r, n) = plan
                .roots
                .iter()
                .flat_map(|r| r.retirements.iter().map(move |n| (r, n)))
                .find(|(r, n)| key(&r.original_tree.nodes[0].root_id, &n.relative) == *id)
                .unwrap();
            let _ = r;
            hash::sha256_hex(&encoded(n).unwrap())
        } else {
            let n = plan
                .roots
                .iter()
                .flat_map(|r| &r.original_tree.nodes)
                .find(|n| n.node_id == *id)
                .unwrap();
            hash::sha256_hex(&encoded(n).unwrap())
        }
    });
    RecoveryEvent {
        schema: "windows-client-originals-v3".into(),
        document_kind: "stopped-recovery-event".into(),
        storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
        event_id: uuid::Uuid::new_v4().to_string(),
        sequence: g.sequence + 1,
        session_id: g.session.clone(),
        operation: op,
        plan: if op == RecoveryOperation::RecoveryPlanIntent {
            Some(decode(&serde_json::to_value(plan).unwrap()))
        } else {
            None
        },
        recovery_plan_digest: plan.digest().unwrap(),
        node_id: id,
        root_id: node.map(|id| plan.node_root(id).unwrap()),
        node_digest,
        movement: node.map(|id| {
            if op == RecoveryOperation::ReleaseVerified {
                plan.anchor_release(id).unwrap().1
            } else {
                plan.movement(
                    id,
                    matches!(
                        op,
                        RecoveryOperation::WorkRetireIntent
                            | RecoveryOperation::WorkRetiredVerified
                    ),
                )
                .unwrap()
                .1
            }
        }),
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
fn wl2_r6_recovery_data_full_plan_missing_duplicate_unknown_cross_scope_and_generation_refuse() {
    let (mut f, p, _, _) = plan();
    let (g, good) = stopped(&p);
    decode(&good).validate(&p, &g).unwrap();
    let mut cases = Vec::new();
    for (pointer, replacement) in [
        ("/roots/0/runtimeAuthority", json!(true)),
        ("/roots/0/schema", json!("windows-client-originals-v2")),
        ("/roots/0/retirements/0/currentFileId", json!("999")),
        ("/roots/0/retirements/0/sourceParentId", json!("999")),
        ("/roots/0/retirements/0/targetParentId", json!("999")),
        ("/roots/0/retirements/0/target", json!("D:/foreign")),
        ("/roots/0/retirements/0/originalAbsent", json!(true)),
        ("/roots/0/retirements/0/originalFileId", Value::Null),
        ("/roots/0/currentTree/nodes/0/fileId", json!("999")),
        ("/roots/0/currentTree/nodes/0/children", json!([])),
        ("/roots/0/currentTree/parentId", json!("999")),
        (
            "/roots/0/currentTree/nodes/1/security/fullSecurityProven",
            json!(true),
        ),
        (
            "/roots/0/currentSnapshot/context/deadlineScope",
            json!(uuid::Uuid::new_v4().to_string()),
        ),
        (
            "/roots/0/currentSnapshot/frameDigest",
            json!("f".repeat(64)),
        ),
        ("/roots/0/currentSnapshot/runtimeAuthority", json!(true)),
    ] {
        let mut v = good.clone();
        *v.pointer_mut(pointer).unwrap() = replacement;
        cases.push(v);
    }
    let mut v = good.clone();
    v["roots"][0]["retirements"].as_array_mut().unwrap().pop();
    cases.push(v);
    let mut v = good.clone();
    let n = v["roots"][0]["retirements"][0].clone();
    v["roots"][0]["retirements"].as_array_mut().unwrap().push(n);
    cases.push(v);
    let mut v = good.clone();
    v["roots"][0]["currentTree"]["extraAuthority"] = true.into();
    cases.push(v);
    let mut v = good.clone();
    v["roots"].as_array_mut().unwrap().swap(0, 1);
    cases.push(v);
    for (i, v) in cases.iter().enumerate() {
        assert!(
            serde_json::from_value::<RecoveryPlan>(v.clone())
                .map(|candidate| candidate.validate(&p, &g).is_err())
                .unwrap_or(true),
            "malformed {i}"
        );
    }
    println!("r6 recovery DATA full plan: {} malformed forms rejected; original/current/archive/retirement generations separated, no native authority", cases.len());
    f.retain();
}

#[test]
fn wl2_r6_recovery_data_chain_all_prefixes_pending_pairing_and_legacy_bypass_refuse() {
    use RecoveryOperation::*;
    let (mut f, p, _, _) = plan();
    let (mut g, data) = stopped(&p);
    let recovery = decode(&data);
    assert!(g
        .apply_recovery(&p, &event(&g, RecoveryPlanVerified, &recovery, None))
        .is_err());
    g.apply_entry(
        &p,
        &segments::ChainEntry::Recovery(event(&g, RecoveryPlanIntent, &recovery, None)),
    )
    .unwrap();
    assert!(!g.recovery_summary().unwrap().unwrap().plan_verified);
    assert!(g
        .apply_work(
            &p,
            &super::super::observations::WorkEvent {
                schema: String::new(),
                document_kind: String::new(),
                storage_strategy: String::new(),
                event_id: String::new(),
                sequence: 0,
                session_id: String::new(),
                operation: observations::WorkOperation::WorkOpenIntent,
                frame_digests: None,
                lease_epoch: String::new(),
                claim_key: String::new(),
                deadline_scope: String::new(),
                previous_event_digest: String::new(),
                storage_plan_digest: String::new(),
                security_policy_digest: String::new(),
                manifest_digest: String::new(),
                binding_digest: String::new()
            }
        )
        .is_err());
    g.apply_recovery(&p, &event(&g, RecoveryPlanVerified, &recovery, None))
        .unwrap();
    let work: Vec<_> = recovery.retirement_keys().into_keys().collect();
    let originals: Vec<_> = recovery.restore_keys().into_keys().collect();
    assert!(g
        .apply_recovery(
            &p,
            &event(&g, OriginalRestoreIntent, &recovery, Some(&originals[0]))
        )
        .is_err());
    for (i, id) in work.iter().enumerate() {
        assert!(g
            .apply_recovery(&p, &event(&g, WorkRetiredVerified, &recovery, Some(id)))
            .is_err());
        g.apply_recovery(&p, &event(&g, WorkRetireIntent, &recovery, Some(id)))
            .unwrap();
        assert_eq!(
            g.recovery_summary().unwrap().unwrap().pending,
            Some((WorkRetireIntent, id.clone()))
        );
        assert!(g
            .apply_recovery(&p, &event(&g, WorkRetireIntent, &recovery, Some(id)))
            .is_err());
        if let Some(other) = work.iter().find(|x| *x != id) {
            assert!(g
                .apply_recovery(&p, &event(&g, WorkRetiredVerified, &recovery, Some(other)))
                .is_err());
        }
        g.apply_recovery(&p, &event(&g, WorkRetiredVerified, &recovery, Some(id)))
            .unwrap();
        assert_eq!(g.recovery_summary().unwrap().unwrap().retired_count, i + 1);
        assert!(!g.complete_files(&p));
    }
    for (i, id) in originals.iter().enumerate() {
        g.apply_recovery(&p, &event(&g, OriginalRestoreIntent, &recovery, Some(id)))
            .unwrap();
        g.apply_recovery(
            &p,
            &event(&g, OriginalRestoredVerified, &recovery, Some(id)),
        )
        .unwrap();
        assert_eq!(g.nodes[id].0, 8);
        assert_eq!(g.recovery_summary().unwrap().unwrap().restored_count, i + 1);
    }
    assert!(
        !g.complete_files(&p),
        "all row data still lacks anchor release/native final proof"
    );
    let n = &p.roots[0].nodes[0];
    assert!(g
        .apply(
            &p,
            &super::super::tests::event(&p, &g, n, Operation::ReleaseVerified, "999")
        )
        .is_err());
    println!("r6 recovery DATA plan intent/result + all {} current retirement and {} original restore intent/result prefixes share sequence/tip; skips/cross-node/reentry/early restore/legacy anchor release refuse, no Closed",work.len(),originals.len());
    f.retain();
}

#[test]
fn wl2_r6_recovery_data_context_drift_unheld_and_unknown_events_never_advance() {
    use RecoveryOperation::*;
    let (mut f, p, _, _) = plan();
    let (mut g, data) = stopped(&p);
    let recovery = decode(&data);
    let good = serde_json::to_value(event(&g, RecoveryPlanIntent, &recovery, None)).unwrap();
    for field in [
        "sessionId",
        "eventId",
        "sequence",
        "storagePlanDigest",
        "manifestDigest",
        "bindingDigest",
        "securityPolicyDigest",
        "leaseEpoch",
        "claimKey",
        "deadlineScope",
        "previousEventDigest",
        "recoveryPlanDigest",
        "nodeId",
        "nodeDigest",
    ] {
        let mut bad = good.clone();
        bad[field] = if field == "sequence" {
            json!(99999)
        } else {
            json!("foreign")
        };
        let sequence = g.sequence;
        let tip = g.tip.clone();
        assert!(
            g.apply_recovery(&p, &serde_json::from_value(bad).unwrap())
                .is_err(),
            "{field}"
        );
        assert_eq!(g.sequence, sequence);
        assert_eq!(g.tip, tip);
        assert!(g.recovery.is_none());
    }
    let mut bad = good.clone();
    bad["Closed"] = true.into();
    assert!(serde_json::from_value::<segments::ChainEntry>(bad).is_err());
    g.lease_state = segments::LeaseState::Released;
    assert!(g
        .apply_recovery(&p, &event(&g, RecoveryPlanIntent, &recovery, None))
        .is_err());
    g.lease_state = segments::LeaseState::Active;
    g.lifecycle = LifecyclePhase::TeardownPrepared;
    assert!(g
        .apply_recovery(&p, &event(&g, RecoveryPlanIntent, &recovery, None))
        .is_err());
    println!("r6 recovery DATA context drift/unknown/unheld/not-stopped reject before progress writes; no replacement deadline or text authority");
    f.retain();
}
#[test]
fn wl2_r6_recovery_data_exact_native_location_identity_parent_and_expected_states_reject_drift() {
    use RecoveryOperation::*;
    let (mut f, p, _, _) = plan();
    let (mut g, data) = stopped(&p);
    let recovery = decode(&data);
    g.apply_recovery(&p, &event(&g, RecoveryPlanIntent, &recovery, None))
        .unwrap();
    g.apply_recovery(&p, &event(&g, RecoveryPlanVerified, &recovery, None))
        .unwrap();
    let id = recovery.retirement_keys().into_keys().next().unwrap();
    let good = serde_json::to_value(event(&g, WorkRetireIntent, &recovery, Some(&id))).unwrap();
    let before = serde_json::to_value(g.recovery_summary().unwrap()).unwrap();
    for field in [
        "from",
        "to",
        "nativeId",
        "parentId",
        "expectedBefore",
        "expectedAfter",
        "volume",
    ] {
        let mut bad = good.clone();
        bad["movement"][field] = if field == "volume" {
            json!(0)
        } else {
            json!("foreign")
        };
        assert!(
            g.apply_recovery(&p, &serde_json::from_value(bad).unwrap())
                .is_err(),
            "{field}"
        );
        assert_eq!(
            serde_json::to_value(g.recovery_summary().unwrap()).unwrap(),
            before
        );
    }
    let duplicate =
        String::from_utf8(encoded(&event(&g, WorkRetireIntent, &recovery, Some(&id))).unwrap())
            .unwrap()
            .replacen("\"nodeId\":", "\"nodeId\":null,\"nodeId\":", 1);
    assert!(serde_json::from_str::<segments::ChainEntry>(&duplicate).is_err());
    println!("r6 recovery DATA exact native from/to/volume/fileID/destination-parent and expected state digests bound to immutable whole-plan row;7 location drifts/duplicate fields refuse without progress change");
    f.retain();
}
fn completed_nodes(p: &StoragePlan) -> (Progress, RecoveryPlan) {
    use RecoveryOperation::*;
    let (mut g, data) = stopped(p);
    let r = decode(&data);
    for op in [RecoveryPlanIntent, RecoveryPlanVerified] {
        g.apply_recovery(p, &event(&g, op, &r, None)).unwrap();
    }
    for id in r.retirement_keys().into_keys() {
        for op in [WorkRetireIntent, WorkRetiredVerified] {
            g.apply_recovery(p, &event(&g, op, &r, Some(&id))).unwrap();
        }
    }
    for id in r.restore_keys().into_keys() {
        for op in [OriginalRestoreIntent, OriginalRestoredVerified] {
            g.apply_recovery(p, &event(&g, op, &r, Some(&id))).unwrap();
        }
    }
    (g, r)
}
fn release_event(g: &Progress, operation: segments::LeaseOperation) -> segments::LeaseEvent {
    segments::LeaseEvent {
        schema: "windows-client-originals-v3".into(),
        document_kind: "lease-event".into(),
        storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
        event_id: uuid::Uuid::new_v4().to_string(),
        sequence: g.sequence + 1,
        session_id: g.session.clone(),
        operation,
        lease_epoch: g.epoch.clone(),
        next_epoch: None,
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
pub(in crate::infrastructure::windows_client_originals024::v3) fn released(
    p: &StoragePlan,
) -> Progress {
    use segments::LeaseOperation::*;
    use RecoveryOperation::*;
    let (mut g, r) = completed_nodes(p);
    for op in [WholeTreeRestoreIntent, WholeTreeRestoredVerified] {
        g.apply_recovery(p, &event(&g, op, &r, None)).unwrap();
    }
    g.apply_lease(p, &release_event(&g, LeaseReleasePrepared))
        .unwrap();
    for n in p
        .roots
        .iter()
        .flat_map(|r| &r.nodes)
        .filter(|n| n.kind != Kind::CustodiedFile)
    {
        g.apply_recovery(p, &event(&g, ReleaseVerified, &r, Some(&n.node_id)))
            .unwrap();
    }
    g.apply_lease(p, &release_event(&g, LeaseReleasedVerified))
        .unwrap();
    g
}
#[test]
fn wl2_r6_history_root_locator_exact_current_original_anchor_and_aggregate_scope_refuse() {
    use RecoveryOperation::*;
    let (mut f, p, _, _) = plan();
    let (mut g, data) = stopped(&p);
    let r = decode(&data);
    let mut aggregate = event(&g, RecoveryPlanIntent, &r, None);
    aggregate.root_id = Some(".codex".into());
    assert!(g.apply_recovery(&p, &aggregate).is_err());
    g.apply_recovery(&p, &event(&g, RecoveryPlanIntent, &r, None))
        .unwrap();
    g.apply_recovery(&p, &event(&g, RecoveryPlanVerified, &r, None))
        .unwrap();
    let id = r.retirement_keys().into_keys().next().unwrap();
    let good = event(&g, WorkRetireIntent, &r, Some(&id));
    assert!(matches!(good.root_id.as_deref(), Some(".codex" | ".grok")));
    for root in [
        None,
        Some("foreign".into()),
        Some(
            if good.root_id.as_deref() == Some(".codex") {
                ".grok"
            } else {
                ".codex"
            }
            .into(),
        ),
    ] {
        let mut bad = event(&g, WorkRetireIntent, &r, Some(&id));
        bad.root_id = root;
        assert!(g.apply_recovery(&p, &bad).is_err());
    }
    let mut old = serde_json::to_value(&good).unwrap();
    old.as_object_mut().unwrap().remove("rootId");
    let old: RecoveryEvent = serde_json::from_value(old).unwrap();
    assert!(g.apply_recovery(&p, &old).is_err());
    let duplicate = String::from_utf8(encoded(&good).unwrap())
        .unwrap()
        .replacen("\"rootId\":", "\"rootId\":null,\"rootId\":", 1);
    assert!(serde_json::from_str::<segments::ChainEntry>(&duplicate).is_err());
    println!("r6 history explicit recovery rootId binds exact node/current/original/anchor root; missing/wrong/cross-root/aggregate/duplicate roots refuse without old-record conversion");
    f.retain();
}
#[test]
fn wl2_r6_final_release_data_requires_full_tree_result_and_exact_release_pair_no_reacquire_or_text_closed(
) {
    use segments::LeaseOperation::*;
    use RecoveryOperation::*;
    let (mut f, p, _, _) = plan();
    let (mut g, r) = completed_nodes(&p);
    assert!(!g.complete_files(&p));
    assert!(g
        .apply_lease(&p, &release_event(&g, LeaseReleasePrepared))
        .is_err());
    assert!(g
        .apply_recovery(&p, &event(&g, WholeTreeRestoredVerified, &r, None))
        .is_err());
    g.apply_recovery(&p, &event(&g, WholeTreeRestoreIntent, &r, None))
        .unwrap();
    assert_eq!(g.recovery_summary().unwrap().unwrap().whole_tree, 1);
    assert!(g
        .apply_lease(&p, &release_event(&g, LeaseReleasePrepared))
        .is_err());
    assert!(g
        .apply_recovery(&p, &event(&g, WholeTreeRestoreIntent, &r, None))
        .is_err());
    g.apply_recovery(&p, &event(&g, WholeTreeRestoredVerified, &r, None))
        .unwrap();
    assert_eq!(g.recovery_summary().unwrap().unwrap().whole_tree, 2);
    assert!(g.recovery_tree_ready_for_release(&p));
    assert!(!g.complete_files(&p));
    assert!(g
        .apply_lease(&p, &release_event(&g, LeaseReleasedVerified))
        .is_err());
    g.apply_lease(&p, &release_event(&g, LeaseReleasePrepared))
        .unwrap();
    assert!(!g.complete_files(&p));
    assert!(g
        .apply_lease(&p, &release_event(&g, LeaseReleasePrepared))
        .is_err());
    assert!(
        g.apply_lease(&p, &release_event(&g, LeaseReleasedVerified))
            .is_err(),
        "all exact anchors must have release events"
    );
    let mut anchor_count = 0;
    for n in p
        .roots
        .iter()
        .flat_map(|r| &r.nodes)
        .filter(|n| n.kind != Kind::CustodiedFile)
    {
        let old_summary = g.recovery_summary().unwrap().unwrap();
        g.apply_recovery(
            &p,
            &event(&g, RecoveryOperation::ReleaseVerified, &r, Some(&n.node_id)),
        )
        .unwrap();
        anchor_count += 1;
        let summary = g.recovery_summary().unwrap().unwrap();
        assert_eq!(summary.anchor_released_count, anchor_count);
        assert_ne!(summary.progress_digest, old_summary.progress_digest);
        assert!(g
            .apply_recovery(
                &p,
                &event(&g, RecoveryOperation::ReleaseVerified, &r, Some(&n.node_id))
            )
            .is_err());
        assert!(!g.complete_files(&p));
    }
    g.apply_lease(&p, &release_event(&g, LeaseReleasedVerified))
        .unwrap();
    assert!(
        g.complete_files(&p),
        "necessary DATA only; no actual receipt/Closed factory"
    );
    assert!(g
        .apply_lease(&p, &release_event(&g, LeaseAcquirePrepared))
        .is_err());
    assert!(g
        .apply_lease(&p, &release_event(&g, LeaseReleasedVerified))
        .is_err());
    assert!(g
        .apply_recovery(&p, &event(&g, WholeTreeRestoredVerified, &r, None))
        .is_err());
    let mut text = serde_json::to_value(release_event(&g, LeaseReleasedVerified)).unwrap();
    text["Closed"] = true.into();
    assert!(serde_json::from_value::<segments::ChainEntry>(text).is_err());
    println!("r6 final-release DATA requires every original/current node, whole-tree intent/result, exact Active->ReleasePrepared->Released; anchors data completes only after release result, no reacquire/repeat/textClosed or native capability");
    f.retain();
}
#[test]
fn wl2_r6_final_release_data_full_tree_and_release_scope_drift_and_incomplete_nodes_refuse() {
    use segments::LeaseOperation::*;
    use RecoveryOperation::*;
    let (mut f, p, _, _) = plan();
    let (mut g, r) = completed_nodes(&p);
    let original = p.roots[0]
        .nodes
        .iter()
        .find(|n| n.kind == Kind::CustodiedFile)
        .unwrap()
        .node_id
        .clone();
    g.nodes.get_mut(&original).unwrap().0 = 7;
    assert!(g
        .apply_recovery(&p, &event(&g, WholeTreeRestoreIntent, &r, None))
        .is_err());
    g.nodes.get_mut(&original).unwrap().0 = 8;
    g.apply_recovery(&p, &event(&g, WholeTreeRestoreIntent, &r, None))
        .unwrap();
    let good = serde_json::to_value(event(&g, WholeTreeRestoredVerified, &r, None)).unwrap();
    for field in [
        "claimKey",
        "leaseEpoch",
        "deadlineScope",
        "sessionId",
        "previousEventDigest",
        "recoveryPlanDigest",
        "nodeId",
        "nodeDigest",
    ] {
        let mut bad = good.clone();
        bad[field] = "foreign".into();
        let tip = g.tip.clone();
        assert!(
            g.apply_recovery(&p, &serde_json::from_value(bad).unwrap())
                .is_err(),
            "{field}"
        );
        assert_eq!(g.tip, tip);
        assert_eq!(g.recovery_summary().unwrap().unwrap().whole_tree, 1);
    }
    g.apply_recovery(&p, &event(&g, WholeTreeRestoredVerified, &r, None))
        .unwrap();
    let good = serde_json::to_value(release_event(&g, LeaseReleasePrepared)).unwrap();
    for field in [
        "claimKey",
        "leaseEpoch",
        "deadlineScope",
        "sessionId",
        "previousEventDigest",
        "objectsDigest",
    ] {
        let mut bad = good.clone();
        bad[field] = "foreign".into();
        let tip = g.tip.clone();
        assert!(
            g.apply_lease(&p, &serde_json::from_value(bad).unwrap())
                .is_err(),
            "{field}"
        );
        assert_eq!(g.tip, tip);
        assert_eq!(g.lease_state, segments::LeaseState::Active);
    }
    println!("r6 final-release DATA incomplete original node,8 whole-tree and6 lease context drifts refuse without sequence/state laundering; original deadline not reset");
    f.retain();
}
