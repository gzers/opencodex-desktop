use super::*;
use crate::infrastructure::windows_official_runtime024::create_private_directory;
use std::time::Duration;
struct CrossData(PathBuf);
#[test]
fn wl2_r6_content_chain_bounded_current_retains_all_original_summary_fields() {
    let (mut f, _data, journal) = setup();
    let mut v: serde_json::Value = serde_json::from_slice(&journal.current_raw).unwrap();
    let h = "f".repeat(64);
    for name in ["workFrames", "observedFrames"] {
        v[name] = serde_json::json!([h, h]);
    }
    v["leaseObjectsDigest"] = h.clone().into();
    v["sequence"] = 4096.into();
    v["observedSequence"] = 4.into();
    v["readyGeneration"] = serde_json::json!([u32::MAX, u16::MAX, h]);
    v["actionKind"] = "Restart".into();
    v["lifecycle"] = "Stopped".into();
    v["linkedFrames"] = serde_json::json!({"1":[h,h],"2":[h,h],"3":[h,h],"4":[h,h]});
    v["snapshotLinks"] =
        serde_json::json!({"Observation":[h,h],"Ready":[h,h],"PreStop":[h,h],"Stopped":[h,h]});
    v["recovery"] = serde_json::json!({"planDigest":h,"progressDigest":h,"planVerified":true,
        "retiredCount":512,"restoredCount":512,"pending":null,"wholeTree":2,"anchorReleasedCount":512});
    v["history"] =
        serde_json::json!({"contextDigest":h,"intentSequence":4095,"linkedReferences":[h,h]});
    v["handles"] = serde_json::json!({"inventoryDigest":h,"count":65536,"released":true});
    v["content"] = serde_json::json!({"digest":h,"verified":true});
    v["closed"] = serde_json::json!({"digest":h,"verified":true,"closeSequence":4094,"closeEventDigest":h,"nativeCloseDigest":h,"routeCloseDigest":h});
    v["routeClose"] = serde_json::json!({"digest":h,"verified":true});
    let typed: Current = serde_json::from_value(v).unwrap();
    let raw = encoded(&typed).unwrap();
    println!(
        "r6 original current full four-stage/recovery/history/handles/content summary bytes={}",
        raw.len()
    );
    assert!(
        raw.len() <= 4096,
        "new content qualification must preserve original current bound"
    );
    let parsed: Current = serde_json::from_slice(&raw).unwrap();
    assert!(parsed == typed);
    drop(journal);
    f.retain();
}
impl Drop for CrossData {
    fn drop(&mut self) {
        println!("r4-retained-synthetic-fixture={}", self.0.display());
    }
}
fn setup() -> (
    super::super::super::tree::tests::Fixture,
    CrossData,
    Journal,
) {
    let (f, plan, profile, parent) = super::super::tests::plan();
    let path = PathBuf::from(
        "D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/test/windows024-native/target",
    )
    .join(format!("r6-v3-{}", uuid::Uuid::new_v4()));
    let data = create_private_directory(&path).unwrap();
    let retained = CrossData(path);
    assert_ne!(
        data.identity().volume,
        parent.volume,
        "test requires actual distinct native volumes; no letter-only claim"
    );
    let pins = [
        PinnedDirectory::open_private(&profile).unwrap(),
        data,
        create_private_directory(&f.path().join("H")).unwrap(),
        create_private_directory(&f.path().join("I")).unwrap(),
    ];
    let directory = create_private_directory(&pins[1].path().join("session")).unwrap();
    let peer = RetainedPeer::current().unwrap();
    let binding = Binding::from_native(
        &peer,
        &pins,
        hash::sha256_hex(b"synthetic source, not official admission"),
        hash::sha256_hex(b"synthetic current"),
    )
    .unwrap();
    let end = Instant::now() + Duration::from_secs(10);
    let journal = Journal::create(
        directory,
        pins,
        peer,
        binding,
        plan,
        &uuid::Uuid::new_v4().to_string(),
        &"c".repeat(64),
        &uuid::Uuid::new_v4().to_string(),
        end,
    )
    .unwrap();
    (f, retained, journal)
}
#[test]
fn wl2_r6_batch_typed_immutable_event_replay_keeps_exact_native_record_bytes_and_original_rules() {
    let (mut f, _data, mut journal) = setup();
    let end = journal.deadline;
    let node = journal
        .manifest
        .storage_plan
        .roots
        .iter()
        .flat_map(|r| &r.nodes)
        .find(|n| n.kind != Kind::CustodiedFile)
        .unwrap();
    let event = super::super::tests::event(
        &journal.manifest.storage_plan,
        &journal.progress,
        node,
        Operation::SnapshotVerified,
        "999",
    );
    journal.append(event, end).unwrap();
    assert_eq!(journal.events.len(), 1);
    let record = &journal.events[0];
    assert_eq!(encoded(record.entry.as_ref().unwrap()).unwrap(), record.raw);
    let decoded: ChainEntry = serde_json::from_slice(&record.raw).unwrap();
    assert_eq!(encoded(&decoded).unwrap(), record.raw);
    journal.check(end).unwrap();
    let mut replay = Progress::new(
        &journal.manifest.storage_plan,
        &journal.progress.session,
        &journal.progress.manifest,
        &journal.progress.binding,
        &journal.manifest.initial_lease_epoch,
        &journal.manifest.claim_key,
        &journal.manifest.deadline_scope,
    )
    .unwrap();
    replay
        .apply_entry(
            &journal.manifest.storage_plan,
            journal.events[0].entry.as_ref().unwrap(),
        )
        .unwrap();
    assert_eq!(replay.sequence, journal.progress.sequence);
    assert_eq!(replay.tip, journal.progress.tip);
    let mut other = OpenOptions::new()
        .access_mode(0x40000000)
        .share_mode(1)
        .custom_flags(0x200000)
        .open(&journal.events[0].path);
    assert!(
        other.is_err(),
        "retained immutable native record still denies other writer"
    );
    assert!(other.as_mut().unwrap_err().raw_os_error().is_some());
    println!("r6 typed immutable record equals exact serialized native bytes, fresh native full record checks retained and original independent progression replay agrees; other writer refused, no data cache grants native identity or new deadline");
    drop(journal);
    f.retain();
}
#[test]
fn wl2_r7_journal_replay_cost_tracks_native_prefix_and_pure_data_separately() {
    let (mut fixture, _data, mut journal) = setup();
    let end = journal.deadline;
    let nodes: Vec<_> = journal
        .manifest
        .storage_plan
        .roots
        .iter()
        .flat_map(|root| &root.nodes)
        .map(|node| (node.root_id.clone(), node.node_id.clone(), node.kind == Kind::CustodiedFile))
        .collect();
    {
        let _timing = super::super::super::recovery_timing::Capture::begin();
        super::super::super::recovery_timing::phase("synthetic-node-prefix");
        for (root_id, node_id, custodied) in nodes {
            let operations: &[_] = if custodied {
                &[Operation::NodeMoveIntent, Operation::NodeMovedVerified]
            } else {
                &[Operation::SnapshotVerified]
            };
            for &operation in operations {
                let plan = &journal.manifest.storage_plan;
                let node = plan
                    .roots
                    .iter()
                    .flat_map(|root| &root.nodes)
                    .find(|node| node.node_id == node_id && node.root_id == root_id)
                    .unwrap();
                let event =
                    super::super::tests::event(plan, &journal.progress, node, operation, "999");
                journal.append(event, end).unwrap();
            }
        }
        assert!(journal.events.len() >= 8);
        let tip = journal.checked_current(end).unwrap();
        assert_eq!(tip.sequence as usize, journal.events.len());
        assert_eq!(tip.event_digest, journal.progress.tip);
    }
    let event = journal.events.last().unwrap().entry.as_ref().unwrap();
    let bytes = encoded(event).unwrap();
    assert_eq!(hash::sha256_hex(&bytes), journal.progress.tip);
    fn measure(iterations: usize, mut operation: impl FnMut() -> String) -> u128 {
        let started = Instant::now();
        for _ in 0..iterations {
            std::hint::black_box(operation());
        }
        started.elapsed().as_micros()
    }
    let iters = 8192;
    let policy_us = measure(iters, || Policy::fixed().digest().unwrap());
    let event_us = measure(iters, || {
        hash::sha256_hex(&encoded(std::hint::black_box(event)).unwrap())
    });
    let data = serde_json::json!({
        "iterations": iters,
        "policyDigestUs": policy_us,
        "eventEncodeAndDigestUs": event_us,
        "eventBytes": bytes.len(),
    });
    println!("r7-journal-pure-data={data}");
    drop(journal);
    fixture.retain();
}
#[test]
fn wl2_r6_capture_canonical_data_namespace_binds_native_session_leaf_and_rejects_other_descendants()
{
    let (mut f, _data, journal) = setup();
    let end = journal.deadline;
    let data = journal.pins[1].path();
    let official = create_private_directory(&data.join("official024")).unwrap();
    let windows = create_private_directory(&official.path().join("windows")).unwrap();
    let session = uuid::Uuid::new_v4().to_string();
    let directory = create_private_directory(&windows.path().join(&session)).unwrap();
    assert_eq!(
        session_namespace(&directory, &journal.pins[1]).unwrap(),
        Some(session.clone())
    );
    let peer = RetainedPeer::current().unwrap();
    let pins = [
        PinnedDirectory::open(journal.pins[0].path()).unwrap(),
        PinnedDirectory::open_private(data).unwrap(),
        PinnedDirectory::open(journal.pins[2].path()).unwrap(),
        PinnedDirectory::open(journal.pins[3].path()).unwrap(),
    ];
    let binding = Binding::from_native(
        &peer,
        &pins,
        hash::sha256_hex(b"synthetic namespace source, not production admission"),
        hash::sha256_hex(b"synthetic namespace current"),
    )
    .unwrap();
    let plan: StoragePlan =
        serde_json::from_slice(&encoded(&journal.manifest.storage_plan).unwrap()).unwrap();
    let mut canonical = Journal::create(
        directory,
        pins,
        peer,
        binding,
        plan,
        &uuid::Uuid::new_v4().to_string(),
        &"c".repeat(64),
        &uuid::Uuid::new_v4().to_string(),
        end,
    )
    .unwrap();
    canonical.check(end).unwrap();
    assert_eq!(canonical.manifest.session_id, session);
    assert_eq!(canonical.progress.session, session);
    let wrong = create_private_directory(&data.join("wrong-parent")).unwrap();
    let leaf =
        create_private_directory(&wrong.path().join(uuid::Uuid::new_v4().to_string())).unwrap();
    assert!(session_namespace(&leaf, &journal.pins[1]).is_err());
    let invalid = create_private_directory(&windows.path().join("not-a-session")).unwrap();
    assert!(session_namespace(&invalid, &journal.pins[1]).is_err());
    println!("r6 actual native D/official024/windows/canonicalUUID journal uses same UUID in immutable manifest/current; wrong descendant/nonUUID refuse, direct D/session remains component-only, actual Source create factory still requires production namespace and original Owner");
    drop(canonical);
    drop(invalid);
    drop(leaf);
    drop(wrong);
    drop(windows);
    drop(official);
    drop(journal);
    f.retain();
}
#[test]
fn wl2_r6_v3_native_journal_manifest_event_current_and_other_writer_refusal() {
    let (mut f, _data, mut journal) = setup();
    let end = journal.deadline;
    let node = journal.manifest.storage_plan.roots[0]
        .nodes
        .iter()
        .find(|n| n.kind == Kind::CustodiedFile)
        .unwrap();
    let event = super::super::tests::event(
        &journal.manifest.storage_plan,
        &journal.progress,
        node,
        Operation::NodeMoveIntent,
        "999",
    );
    let old_id = journal.current_id;
    let directory = journal.directory.path().to_path_buf();
    journal.append(event, end).unwrap();
    journal.check(end).unwrap();
    assert_eq!(journal.progress.sequence, 1);
    assert_eq!(journal.current_id, old_id);
    let current: Current =
        serde_json::from_slice(&std::fs::read(directory.join("current.json")).unwrap()).unwrap();
    assert_eq!(current.schema, ANCHORED_CURRENT_SCHEMA_V3);
    assert_eq!(current.sequence, 1);
    let e = OpenOptions::new()
        .read(true)
        .write(true)
        .open(directory.join("current.json"))
        .err()
        .unwrap();
    assert_eq!(e.raw_os_error(), Some(32));
    assert!(directory.join("event-000001.json").exists());
    assert!(!journal
        .progress
        .complete_files(&journal.manifest.storage_plan));
    println!("r6 native manifest/event immutable sync/readback and current same-ID CAS pass; actual D separate volume; second current writer raw32. This is data journal only, original tree not mutated, no recovery capability.");
    drop(journal);
    f.retain();
}
#[test]
fn wl2_r6_v3_native_journal_event_or_partial_current_failure_poison_without_catchup() {
    for fault in [1, 2] {
        let (mut f, _data, mut journal) = setup();
        let end = journal.deadline;
        let directory = journal.directory.path().to_path_buf();
        let node = journal.manifest.storage_plan.roots[0]
            .nodes
            .iter()
            .find(|n| n.kind == Kind::CustodiedFile)
            .unwrap();
        let event = super::super::tests::event(
            &journal.manifest.storage_plan,
            &journal.progress,
            node,
            Operation::NodeMoveIntent,
            "999",
        );
        journal.fault = fault;
        let error = journal.append(event, end).err().unwrap();
        println!("r6 durable injected fault={fault}: {error}");
        assert!(journal.poisoned);
        assert_eq!(journal.progress.sequence, 0);
        assert!(directory.join("event-000001.json").exists());
        assert!(journal.check(end).is_err());
        journal.fault = 0;
        assert!(
            journal.check(end).is_err(),
            "no automatic repair of partial current"
        );
        let bytes = std::fs::read(directory.join("current.json")).unwrap();
        if fault == 1 {
            let c: Current = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(c.sequence, 0);
        } else {
            assert_eq!(bytes, b"{");
        }
        drop(journal);
        f.retain();
    }
}
#[test]
fn wl2_r6_v3_native_journal_unknown_node_and_late_deadline_poison() {
    for foreign in [false, true] {
        let (mut f, _data, mut journal) = setup();
        let end = journal.deadline;
        if foreign {
            std::fs::create_dir(journal.directory.path().join("unknown-empty")).unwrap();
            assert!(journal.check(end).is_err());
        } else {
            assert!(journal.check(end + Duration::from_millis(1)).is_err());
        }
        assert!(journal.poisoned);
        assert!(journal.check(end).is_err());
        assert_eq!(journal.progress.sequence, 0);
        drop(journal);
        f.retain();
    }
}
