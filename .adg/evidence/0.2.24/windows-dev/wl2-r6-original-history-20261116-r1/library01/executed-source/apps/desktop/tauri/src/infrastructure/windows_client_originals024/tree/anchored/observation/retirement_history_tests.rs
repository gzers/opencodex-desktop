use super::super::super::tests::{compete, driver::setup};
use super::chain_tests::positions;
use super::release_tests::restore;
use super::*;
fn run(fault: u8) {
    let (mut f, _data, paths, held, mut trees, mut journal, end) = setup();
    let mut lease = Some(held);
    restore(&mut trees, &mut journal, &mut lease, end);
    let root = trees[0].root.clone();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    let witness = recovery_witness(&mut trees, &mut claim, end).unwrap();
    drop(claim);
    journal
        .component_release_recovery_for_test(witness, &mut lease, end)
        .unwrap();
    assert!(lease.is_none());
    compete(&paths[0], false);
    journal.inject_driver_fault(fault);
    let result = journal.component_original_history_for_test(
        released_history_witness(&mut trees, end).unwrap(),
        end,
    );
    let trace = journal.component_history_trace_for_test().unwrap();
    assert_eq!(trace["actualReleaseReceiptRetained"], true);
    assert_eq!(trace["leaseState"], "Released");
    let events = trace["immutableHistoryEvents"].as_array().unwrap();
    assert!(!events.is_empty());
    let last = events.last().unwrap();
    let summary = &trace["historySummary"];
    let actual_archives = trees
        .iter()
        .filter(|t| {
            t.original_history
                .as_ref()
                .is_some_and(|f| f.archive.is_some())
        })
        .count();
    if fault == 0 {
        result.unwrap();
        assert_eq!(last["operation"], "OriginalHistoryLinkedVerified");
        assert_eq!(events.len(), 2);
        assert!(summary["linkedReferences"].is_array());
        assert_eq!(actual_archives, 2);
        assert_eq!(trace["poisoned"], false);
        released_history_witness(&mut trees, end)
            .unwrap()
            .history_references(end)
            .unwrap();
        journal.check(end).unwrap();
    } else {
        let error = result.unwrap_err();
        println!("r6 original-history fault={fault} error={error}");
        assert_eq!(trace["poisoned"], true);
        assert!(trees.iter().all(|t| t.poisoned));
        assert!(summary["linkedReferences"].is_null());
        assert_eq!(
            last["operation"],
            if matches!(fault, 74 | 75) {
                "OriginalHistoryLinkedVerified"
            } else {
                "OriginalHistoryIntent"
            }
        );
        assert_eq!(
            actual_archives,
            match fault {
                70 => 0,
                71 | 72 => 1,
                _ => 2,
            }
        );
        assert_eq!(trace["currentActualBytes"].as_u64() == Some(1), fault == 75);
        let sequence = trace["sequence"].as_u64().unwrap();
        assert_eq!(
            last["sequence"].as_u64(),
            Some(sequence + u64::from(matches!(fault, 74 | 75)))
        );
        assert!(journal.check(end).is_err());
        assert_eq!(
            journal.component_history_trace_for_test().unwrap(),
            trace,
            "no catch-up"
        );
    }
    if fault != 75 {
        assert_eq!(trace["currentActual"]["history"], trace["historySummary"]);
    }
    println!(
        "r6-original-history-native={}",
        serde_json::json!({"fault":fault,"durable":trace,"positions":positions(&trees),"actualHistoryArchives":actual_archives,"Closed":false})
    );
    drop(journal);
    drop(trees);
    f.retain();
}
#[test]
fn wl2_r6_history_native_originals_and_archive_copy_identities_remain_distinct() {
    run(0);
}
#[test]
fn wl2_r6_history_native_archive_copy_and_canonical_link_failures_preserve_precise_prefix() {
    for fault in [70, 71, 72, 73, 74, 75] {
        run(fault);
    }
}
