use super::super::super::tests::{compete, driver::setup};
use super::chain_tests::{keys, positions, prepare};
use super::*;
use crate::infrastructure::windows_client_originals024::{coordination::NativeLease, v3::Journal};
pub(super) fn restore(
    trees: &mut [AnchoredTree; 2],
    journal: &mut Journal,
    lease: &mut Option<NativeLease>,
    end: Instant,
) {
    prepare(trees, journal, lease, end);
    let root = trees[0].root.clone();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    let (work, original) = keys(trees);
    journal
        .component_prepare_recovery_for_test(
            recovery_witness(trees, &mut claim, end).unwrap(),
            &mut claim,
            end,
        )
        .unwrap();
    for (retire, nodes) in [(true, &work), (false, &original)] {
        for node in nodes {
            journal
                .component_execute_recovery_for_test(
                    recovery_witness(trees, &mut claim, end).unwrap(),
                    node,
                    retire,
                    &mut claim,
                    end,
                )
                .unwrap();
        }
    }
    recovery_witness(trees, &mut claim, end)
        .unwrap()
        .restored(&mut claim, end)
        .unwrap();
}
fn run(fault: u8) {
    let (mut f, _data, paths, held, mut trees, mut journal, end) = setup();
    let mut lease = Some(held);
    restore(&mut trees, &mut journal, &mut lease, end);
    let original = trees.each_ref().map(|t| t.plan());
    let anchor_count = trees
        .iter()
        .flat_map(|t| &t.nodes)
        .filter(|n| n.kind != Kind::File)
        .count();
    let root = trees[0].root.clone();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    compete(&paths[0], true);
    let witness = recovery_witness(&mut trees, &mut claim, end).unwrap();
    drop(claim);
    journal.inject_driver_fault(fault);
    let result = journal.component_release_recovery_for_test(witness, &mut lease, end);
    let released = fault == 0 || fault >= 55;
    assert_eq!(lease.is_none(), released);
    compete(&paths[0], !released);
    let trace = journal.component_final_release_trace_for_test().unwrap();
    let recovery = trace["immutableRecoveryEvents"].as_array().unwrap();
    let last = recovery.last().unwrap();
    let leases = trace["immutableLeaseEvents"].as_array().unwrap();
    let last_lease = leases.last().unwrap();
    assert_eq!(trace["actualReleaseReceiptRetained"], released);
    assert_eq!(trees.each_ref().map(|t| t.plan()), original);
    assert_eq!(trace["summary"]["retiredCount"], 6);
    assert_eq!(trace["summary"]["restoredCount"], 3);
    if fault == 0 {
        result.unwrap();
        assert_eq!(trace["leaseState"], "Released");
        assert_eq!(trace["completeFilesDataOnly"], true);
        assert_eq!(last["operation"], "ReleaseVerified");
        assert_eq!(last_lease["operation"], "LeaseReleasedVerified");
        assert_eq!(trace["summary"]["wholeTree"], 2);
        assert_eq!(trace["summary"]["anchorReleasedCount"], anchor_count);
        assert!(!trees.iter().any(|t| t.poisoned));
        journal.check(end).unwrap();
    } else {
        let error = result.unwrap_err();
        println!("r6 final-release fault={fault} raw-error={error}");
        assert_eq!(trace["poisoned"], true);
        assert!(trees.iter().all(|t| t.poisoned));
        assert_eq!(trace["completeFilesDataOnly"], false);
        let whole_pending = fault <= 53;
        assert_eq!(
            trace["summary"]["wholeTree"],
            if whole_pending { 1 } else { 2 }
        );
        assert_eq!(trace["summary"]["pending"].is_null(), !whole_pending);
        assert_eq!(
            trace["leaseState"],
            if fault <= 53 {
                "Active"
            } else {
                "ReleasePrepared"
            }
        );
        assert_eq!(
            trace["currentActualBytes"].as_u64() == Some(1),
            matches!(fault, 53 | 58 | 60)
        );
        if fault <= 53 {
            assert_eq!(
                last["operation"],
                if fault <= 51 {
                    "WholeTreeRestoreIntent"
                } else {
                    "WholeTreeRestoredVerified"
                }
            );
        } else {
            assert_eq!(
                last["operation"],
                if fault <= 55 || fault == 61 {
                    "WholeTreeRestoredVerified"
                } else {
                    "ReleaseVerified"
                }
            );
            assert_eq!(
                last_lease["operation"],
                if !matches!(fault, 57 | 58) {
                    "LeaseReleasePrepared"
                } else {
                    "LeaseReleasedVerified"
                }
            );
        }
        let sequence = trace["sequence"].as_u64().unwrap();
        let last_sequence = last["sequence"]
            .as_u64()
            .unwrap()
            .max(last_lease["sequence"].as_u64().unwrap());
        assert_eq!(
            last_sequence,
            sequence + u64::from(matches!(fault, 52 | 53 | 57 | 58 | 59 | 60))
        );
        assert_eq!(
            trace["summary"]["anchorReleasedCount"],
            if matches!(fault, 56 | 57 | 58) {
                anchor_count
            } else {
                0
            }
        );
        if fault == 61 {
            assert!(trees[0]
                .custody
                .path()
                .join("synthetic-final-release-foreign")
                .is_dir());
        }
        if !matches!(fault, 53 | 58 | 60) {
            assert_eq!(trace["currentActual"]["sequence"].as_u64(), Some(sequence));
            assert_eq!(trace["currentActual"]["recovery"], trace["summary"]);
        }
        assert!(journal.check(end).is_err());
        assert_eq!(
            journal.component_final_release_trace_for_test().unwrap(),
            trace,
            "no catch-up"
        );
    }
    println!(
        "r6-final-release-native={}",
        serde_json::json!({"fault":fault,"actualLeaseReleased":released,"durable":trace,"positions":positions(&trees),"Closed":false})
    );
    if let Some(held) = lease.take() {
        held.release(end).unwrap();
    }
    drop(journal);
    drop(trees);
    f.retain();
}
#[test]
fn wl2_r6_final_release_native_full_tree_canonical_receipt_and_released_stable_ranges() {
    run(0);
}
#[test]
fn wl2_r6_final_release_native_whole_tree_result_faults_keep_lease_and_precise_current() {
    for fault in [50, 51, 52, 53] {
        run(fault);
    }
}
#[test]
fn wl2_r6_final_release_native_explicit_unlock_and_result_faults_retain_exact_receipt() {
    for fault in [54, 55, 56, 57, 58, 59, 60, 61] {
        run(fault);
    }
}
