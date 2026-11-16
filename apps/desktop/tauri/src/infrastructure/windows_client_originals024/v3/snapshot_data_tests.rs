use super::super::{
    observations::{WorkEvent, WorkMode, WorkOperation},
    tests::{event, plan, progress},
};
use super::*;
pub(in crate::infrastructure::windows_client_originals024::v3) fn observed(
    p: &StoragePlan,
) -> Progress {
    let mut g = progress(p);
    for n in p.roots.iter().flat_map(|r| &r.nodes) {
        if n.kind != Kind::CustodiedFile {
            g.apply(p, &event(p, &g, n, Operation::SnapshotVerified, "999"))
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
            g.apply(p, &event(p, &g, n, op, &(u64::MAX - i as u64).to_string()))
                .unwrap();
        }
    }
    let frames = ["1".repeat(64), "2".repeat(64)];
    for op in [
        WorkOperation::WorkOpenIntent,
        WorkOperation::WorkOpenedVerified,
        WorkOperation::WorkSealIntent,
        WorkOperation::WorkSealedVerified,
    ] {
        let e = WorkEvent {
            schema: "windows-client-originals-v3".into(),
            document_kind: "work-interval-event".into(),
            storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
            event_id: uuid::Uuid::new_v4().to_string(),
            sequence: g.sequence + 1,
            session_id: g.session.clone(),
            operation: op,
            frame_digests: if op == WorkOperation::WorkSealIntent {
                None
            } else {
                Some(frames.clone())
            },
            lease_epoch: g.epoch.clone(),
            claim_key: g.claim.clone(),
            deadline_scope: g.deadline.clone(),
            previous_event_digest: g.tip.clone(),
            storage_plan_digest: g.plan.clone(),
            security_policy_digest: Policy::fixed().digest().unwrap(),
            manifest_digest: g.manifest.clone(),
            binding_digest: g.binding.clone(),
        };
        g.apply_work(p, &e).unwrap();
    }
    assert_eq!(g.work_mode, WorkMode::Observed);
    g
}
pub(in crate::infrastructure::windows_client_originals024::v3) fn refs(
    p: &StoragePlan,
    g: &Progress,
    purpose: SnapshotPurpose,
    sequence: u32,
) -> [SnapshotReference; 2] {
    std::array::from_fn(|i| {
        let root = &p.roots[i];
        let original = &root.nodes[0].security;
        // Separate native objects for every actual phase/archive; previously
        // these synthetic locators falsely reused IDs across distinct stages.
        let phase = match purpose {
            SnapshotPurpose::Observation => 0,
            SnapshotPurpose::Ready => 1,
            SnapshotPurpose::PreStop => 2,
            SnapshotPurpose::Stopped => 3,
        };
        let archive_offset = u64::from(sequence) * 32 + phase * 4;
        let directory_id = (u64::MAX - 100 - archive_offset - i as u64 * 2).to_string();
        let metadata_id = (u64::MAX - 101 - archive_offset - i as u64 * 2).to_string();
        let security = |id: &str| {
            let mut v = serde_json::to_value(original).unwrap();
            v["fileId"] = id.into();
            serde_json::from_value(v).unwrap()
        };
        SnapshotReference {
            schema: "windows-anchored-work-observation-reference-v3".into(),
            directory: Path::new(&root.root)
                .parent()
                .unwrap()
                .join(format!(
                    ".ocxd024-observed-{}-{}-{}",
                    g.session,
                    purpose.leaf(),
                    if i == 0 { "codex" } else { "grok" }
                ))
                .to_str()
                .unwrap()
                .into(),
            volume: root.custody_volume,
            file_id: directory_id.clone(),
            security: security(&directory_id),
            metadata_id: metadata_id.clone(),
            metadata_security: security(&metadata_id),
            metadata_sha256: "3".repeat(64),
            frame_digest: g.observed_frames.as_ref().unwrap()[i].clone(),
            context: SnapshotContext {
                session_id: g.session.clone(),
                storage_plan_digest: g.plan.clone(),
                binding_digest: g.binding.clone(),
                manifest_digest: g.manifest.clone(),
                deadline_scope: g.deadline.clone(),
                frame_digests: g.observed_frames.clone().unwrap(),
                purpose,
                observation_sequence: sequence,
            },
            runtime_authority: false,
        }
    })
}
pub(in crate::infrastructure::windows_client_originals024::v3) fn snapshot_event(
    g: &Progress,
    operation: SnapshotOperation,
    purpose: SnapshotPurpose,
    sequence: u32,
    references: Option<[SnapshotReference; 2]>,
) -> SnapshotEvent {
    SnapshotEvent {
        schema: "windows-client-originals-v3".into(),
        document_kind: "snapshot-phase-event".into(),
        storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
        event_id: uuid::Uuid::new_v4().to_string(),
        sequence: g.sequence + 1,
        session_id: g.session.clone(),
        operation,
        purpose,
        observation_sequence: sequence,
        frame_digests: g.observed_frames.clone().unwrap(),
        references,
        ready: None,
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
fn wl2_r6_snapshot_data_strict_both_roots_context_and_reference_rejects_text_native_authority() {
    let (mut f, p, _, _) = plan();
    let mut g = observed(&p);
    let sequence = g.sequence;
    let bad = snapshot_event(
        &g,
        SnapshotOperation::SnapshotLinkedVerified,
        SnapshotPurpose::Ready,
        sequence,
        Some(refs(&p, &g, SnapshotPurpose::Ready, sequence)),
    );
    assert!(g.apply_snapshot(&p, &bad).is_err());
    let skip = snapshot_event(
        &g,
        SnapshotOperation::SnapshotLinkIntent,
        SnapshotPurpose::Stopped,
        sequence,
        None,
    );
    assert!(g.apply_snapshot(&p, &skip).is_err());
    g.apply_snapshot(
        &p,
        &snapshot_event(
            &g,
            SnapshotOperation::SnapshotLinkIntent,
            SnapshotPurpose::Ready,
            sequence,
            None,
        ),
    )
    .unwrap();
    let good = snapshot_event(
        &g,
        SnapshotOperation::SnapshotLinkedVerified,
        SnapshotPurpose::Ready,
        sequence,
        Some(refs(&p, &g, SnapshotPurpose::Ready, sequence)),
    );
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
        v[field] = "invalid".into();
        let e: SnapshotEvent = serde_json::from_value(v).unwrap();
        assert!(g.apply_snapshot(&p, &e).is_err(), "drift {field}");
    }
    for field in [
        "schema",
        "directory",
        "fileId",
        "metadataId",
        "metadataSha256",
        "frameDigest",
    ] {
        let mut v = serde_json::to_value(&good).unwrap();
        v["references"][0][field] = "invalid".into();
        let e: SnapshotEvent = serde_json::from_value(v).unwrap();
        assert!(g.apply_snapshot(&p, &e).is_err(), "ref drift {field}");
    }
    let mut v = serde_json::to_value(&good).unwrap();
    v["references"][0]["runtimeAuthority"] = true.into();
    assert!(g
        .apply_snapshot(&p, &serde_json::from_value(v).unwrap())
        .is_err());
    let mut v = serde_json::to_value(&good).unwrap();
    v["references"][0]["context"]["observationSequence"] = (sequence + 1).into();
    assert!(g
        .apply_snapshot(&p, &serde_json::from_value(v).unwrap())
        .is_err());
    let mut v = serde_json::to_value(&good).unwrap();
    v["references"].as_array_mut().unwrap().swap(0, 1);
    assert!(g
        .apply_snapshot(&p, &serde_json::from_value(v).unwrap())
        .is_err());
    let mut v = serde_json::to_value(&good).unwrap();
    v["references"].as_array_mut().unwrap().pop();
    assert!(serde_json::from_value::<SnapshotEvent>(v).is_err());
    let mut v = serde_json::to_value(&good).unwrap();
    v["references"][0]["unknownNativePermit"] = true.into();
    assert!(serde_json::from_value::<SnapshotEvent>(v).is_err());
    let mut v = serde_json::to_value(&good).unwrap();
    v["operation"] = "Closed".into();
    assert!(serde_json::from_value::<SnapshotEvent>(v).is_err());
    g.apply_snapshot(&p, &good).unwrap();
    assert!(g.snapshot_pending.is_none());
    assert_eq!(g.snapshot_links.len(), 1);
    assert_eq!(g.lifecycle, LifecyclePhase::Preparing);
    println!("r6 snapshot reference DATA cases reject skip,11 context+6 ref drifts, wrong roots/sequence/missing second/unknown fields/text authority; valid locator links remain data and do not construct actual SnapshotWitness/NativeAction/Ready");
    f.retain();
}
#[test]
fn wl2_r6_history_snapshot_archive_ids_cannot_alias_previously_linked_native_objects() {
    let (mut f, p, _, _) = plan();
    let mut g = observed(&p);
    let sequence = g.observed_sequence;
    let ready = refs(&p, &g, SnapshotPurpose::Ready, sequence);
    let later = refs(&p, &g, SnapshotPurpose::Stopped, sequence);
    assert_ne!(
        ready[0].object_ids(),
        later[0].object_ids(),
        "independent phases require independent object IDs"
    );
    for id in ready[0].object_ids() {
        g.snapshot_objects.insert(id);
    }
    g.apply_snapshot(
        &p,
        &snapshot_event(
            &g,
            SnapshotOperation::SnapshotLinkIntent,
            SnapshotPurpose::Ready,
            sequence,
            None,
        ),
    )
    .unwrap();
    let tip = g.tip.clone();
    let pending = g.snapshot_pending;
    assert!(g
        .apply_snapshot(
            &p,
            &snapshot_event(
                &g,
                SnapshotOperation::SnapshotLinkedVerified,
                SnapshotPurpose::Ready,
                sequence,
                Some(ready)
            )
        )
        .is_err());
    assert_eq!(g.tip, tip);
    assert_eq!(g.snapshot_pending, pending);
    assert!(!g.snapshot_links.contains_key(&SnapshotPurpose::Ready));
    println!("r6 history snapshot DATA reusing any previously linked native archive/metadata ID refuses; synthetic phase references now carry independent IDs without changing original assertions or native qualification");
    f.retain();
}
