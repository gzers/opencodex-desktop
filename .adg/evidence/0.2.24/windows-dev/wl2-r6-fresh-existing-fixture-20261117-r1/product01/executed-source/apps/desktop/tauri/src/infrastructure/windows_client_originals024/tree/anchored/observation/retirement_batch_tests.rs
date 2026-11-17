use super::super::super::{
    batches::initial_batch_witness,
    tests::{bun_claim, compete, driver::setup_unsnapshotted},
};
use super::*;
use crate::infrastructure::windows_client_originals024::{coordination::NativeLease, v3::Journal};
pub(super) fn restore_batched(
    trees: &mut [AnchoredTree; 2],
    journal: &mut Journal,
    lease: &mut Option<NativeLease>,
    end: Instant,
) {
    let root = trees[0].root.clone();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    journal
        .prepare_initial_batches(
            initial_batch_witness(trees, &mut claim, end).unwrap(),
            &mut claim,
            end,
        )
        .unwrap();
    drop(claim);
    super::chain_tests::sample_phases(trees, journal, lease, end);
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    journal
        .component_prepare_recovery_for_test(
            recovery_witness(trees, &mut claim, end).unwrap(),
            &mut claim,
            end,
        )
        .unwrap();
    for retire in [true, false] {
        journal
            .component_recover_batch_for_test(
                recovery_witness(trees, &mut claim, end).unwrap(),
                retire,
                &mut claim,
                end,
            )
            .unwrap();
    }
    recovery_witness(trees, &mut claim, end)
        .unwrap()
        .restored(&mut claim, end)
        .unwrap();
}
fn initial_positions(trees: &[AnchoredTree; 2]) -> serde_json::Value {
    let mut rows = Vec::new();
    for t in trees {
        for (i, n) in t.nodes.iter().enumerate() {
            let path = t.original_location(i);
            let original = open_node(&path, n.kind, true).unwrap();
            assert_eq!(
                id(&original, n.kind == Kind::Directory).unwrap(),
                n.original.id
            );
            if n.kind != Kind::Directory {
                assert_eq!(std::fs::read(&path).unwrap(), n.original.bytes);
            }
            rows.push(serde_json::json!({"role":"original","root":t.root,"relative":n.original.relative,"path":path,"fileId":n.original.id.index.to_string(),
            "state":format!("{:?}",n.state),"security":n.original.security,"sha256":hash::sha256_hex(&n.original.bytes)}));
            if let Some((_, entry)) = &n.work {
                let path = t.work_location(i);
                let file = open_node(&path, Kind::File, true).unwrap();
                assert_eq!(id(&file, false).unwrap(), entry.id);
                assert_eq!(std::fs::read(&path).unwrap(), entry.bytes);
                rows.push(serde_json::json!({"role":"work","relative":entry.relative,"path":path,"fileId":entry.id.index.to_string(),"security":entry.security,"sha256":hash::sha256_hex(&entry.bytes)}));
            }
        }
    }
    serde_json::json!({"postFailureForensicOnly":true,"rows":rows})
}
#[test]
fn wl2_r6_batch_native_complete_initial_and_recovery_phases_keep_exact_originals_and_claim() {
    let (mut f, _data, paths, held, mut trees, mut journal, end) = setup_unsnapshotted();
    let mut lease = Some(held);
    let original = trees.each_ref().map(|t| t.plan());
    restore_batched(&mut trees, &mut journal, &mut lease, end);
    let root = trees[0].root.clone();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    compete(&paths[0], true);
    bun_claim(&paths[0], end, true);
    assert_eq!(trees.each_ref().map(|t| t.plan()), original);
    let batch = journal.component_batch_trace_for_test().unwrap();
    assert!(batch["pending"].is_null());
    assert_eq!(batch["physicalEventFiles"], 47);
    let stored = batch["retainedNativeBatches"].as_array().unwrap();
    assert_eq!(stored.len(), 5);
    assert!(stored.iter().all(|n| n["committed"] == true));
    assert_eq!(
        stored
            .iter()
            .map(|n| n["actualVerifiedPrefix"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![6, 3, 3, 6, 3]
    );
    let release = recovery_witness(&mut trees, &mut claim, end).unwrap();
    drop(claim);
    journal
        .component_release_recovery_for_test(release, &mut lease, end)
        .unwrap();
    assert!(lease.is_none());
    compete(&paths[0], false);
    journal
        .component_original_history_for_test(borrow_history(&mut trees), end)
        .unwrap();
    journal
        .component_close_original_handles_for_test(trees, end)
        .unwrap();
    let trace = journal.component_handle_trace_for_test().unwrap();
    let batch = journal.component_batch_trace_for_test().unwrap();
    assert_eq!(batch["physicalEventFiles"], 56);
    assert_eq!(trace["originalHandleSummary"]["released"], true);
    assert_eq!(
        journal.component_completion_for_test()["completedInOriginalWindow"],
        true
    );
    println!(
        "r6-batch-native-complete={}",
        serde_json::json!({"batch":batch,"durable":trace,"completion":journal.component_completion_for_test(),"typedClosed":false})
    );
    drop(journal);
    f.retain();
}
fn initial_fault(fault: u8) {
    let (mut f, _data, _paths, mut lease, mut trees, mut journal, end) = setup_unsnapshotted();
    let root = trees[0].root.clone();
    let mut claim = lease.anchored_claim(&root, end).unwrap();
    journal.inject_driver_fault(fault);
    assert!(journal
        .prepare_initial_batches(
            initial_batch_witness(&mut trees, &mut claim, end).unwrap(),
            &mut claim,
            end
        )
        .is_err());
    assert!(trees.iter().all(|t| t.poisoned));
    assert!(journal.native_progress_for_test().1);
    let moved = trees
        .iter()
        .flat_map(|t| &t.nodes)
        .filter(|n| n.kind == Kind::File && n.state == State::Moved)
        .count();
    let working = trees
        .iter()
        .flat_map(|t| &t.nodes)
        .filter(|n| n.kind == Kind::File && n.state == State::Working)
        .count();
    let expected = match fault {
        100 | 140 => 0,
        112 | 152 => 1,
        101..=103 => (fault - 100) as usize,
        141..=143 => (fault - 140) as usize,
        _ => 3,
    };
    assert_eq!(if fault < 140 { moved } else { working }, expected);
    if fault >= 140 {
        assert_eq!(moved + working, 3);
    }
    let batch = journal.component_batch_trace_for_test().unwrap();
    assert!(batch["pending"].is_object());
    assert_eq!(
        batch["pending"]["prefix"],
        if matches!(fault, 112 | 152) { 1 } else { 0 }
    );
    let durable = journal.component_recovery_trace_for_test().unwrap();
    assert_eq!(
        durable["currentActualBytes"].as_u64() == Some(1),
        matches!(fault, 111 | 151)
    );
    assert!(journal.check(end).is_err());
    println!(
        "r6-batch-native-initial-fault={}",
        serde_json::json!({"fault":fault,"moved":moved,"working":working,"batch":batch,"durable":durable,"positions":initial_positions(&trees)})
    );
    drop(claim);
    lease.release(end).unwrap();
    drop(journal);
    drop(trees);
    f.retain();
}
#[test]
fn wl2_r6_batch_native_initial_every_original_move_and_work_create_prefix_pending_and_current_failures(
) {
    for fault in [
        100, 101, 102, 103, 110, 111, 112, 140, 141, 142, 143, 150, 151, 152,
    ] {
        initial_fault(fault);
    }
}
fn recovery_fault(fault: u8, retire: bool) {
    let (mut f, _data, _paths, held, mut trees, mut journal, end) = setup_unsnapshotted();
    let mut lease = Some(held);
    let root = trees[0].root.clone();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    journal
        .prepare_initial_batches(
            initial_batch_witness(&mut trees, &mut claim, end).unwrap(),
            &mut claim,
            end,
        )
        .unwrap();
    drop(claim);
    super::chain_tests::sample_phases(&mut trees, &mut journal, &mut lease, end);
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    journal
        .component_prepare_recovery_for_test(
            recovery_witness(&mut trees, &mut claim, end).unwrap(),
            &mut claim,
            end,
        )
        .unwrap();
    if !retire {
        journal
            .component_recover_batch_for_test(
                recovery_witness(&mut trees, &mut claim, end).unwrap(),
                true,
                &mut claim,
                end,
            )
            .unwrap();
    }
    journal.inject_driver_fault(fault);
    assert!(journal
        .component_recover_batch_for_test(
            recovery_witness(&mut trees, &mut claim, end).unwrap(),
            retire,
            &mut claim,
            end
        )
        .is_err());
    let expected = match fault {
        120 => 0,
        132 => 1,
        121..=126 => (fault - 120) as usize,
        _ => {
            if retire {
                6
            } else {
                3
            }
        }
    };
    let current = trees
        .iter()
        .map(|t| {
            t.retirement
                .as_ref()
                .unwrap()
                .items
                .iter()
                .filter(|i| i.retired)
                .count()
        })
        .sum::<usize>();
    let original = trees
        .iter()
        .flat_map(|t| &t.nodes)
        .filter(|n| n.kind == Kind::File && n.state == State::Restored)
        .count();
    assert_eq!(if retire { current } else { original }, expected);
    if !retire {
        assert_eq!(current, 6);
    }
    assert!(trees.iter().all(|t| t.poisoned));
    let batch = journal.component_batch_trace_for_test().unwrap();
    assert!(batch["pending"].is_object());
    let durable = journal.component_recovery_trace_for_test().unwrap();
    assert_eq!(
        durable["currentActualBytes"].as_u64() == Some(1),
        fault == 131
    );
    assert!(journal.check(end).is_err());
    println!(
        "r6-batch-native-recovery-fault={}",
        serde_json::json!({"fault":fault,"retire":retire,"currentRetired":current,"originalRestored":original,
        "batch":batch,"durable":durable,"positions":super::chain_tests::positions(&trees)})
    );
    drop(claim);
    lease.take().unwrap().release(end).unwrap();
    drop(journal);
    drop(trees);
    f.retain();
}
#[test]
fn wl2_r6_batch_native_recovery_all_retire_restore_prefixes_and_immutable_or_partial_current_failures(
) {
    for fault in [120, 121, 122, 123, 124, 125, 126, 130, 131, 132] {
        recovery_fault(fault, true);
    }
    for fault in [120, 121, 122, 123, 130, 131, 132] {
        recovery_fault(fault, false);
    }
}
