use super::super::super::tests::{compete, driver::setup};
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
    journal
        .component_original_history_for_test(borrow_history(&mut trees), end)
        .unwrap();
    compete(&paths[0], false);
    let config = root.join("config.toml");
    let original = std::fs::read(&config).unwrap();
    journal.inject_driver_fault(fault);
    let result = journal.component_close_original_handles_for_test(trees, end);
    if let Err(error) = &result {
        println!(
            "r6-original-handle-handoff-attempt={}",
            serde_json::json!({"fault":fault,"error":error.to_string(),"durable":journal.component_handle_trace_for_test().unwrap()})
        );
    }
    let trace = journal.component_handle_trace_for_test().unwrap();
    let inventory = trace["originalNativeHandoff"]["handles"]["rows"]
        .as_array()
        .unwrap();
    assert!(!inventory.is_empty());
    let held_count = inventory.iter().filter(|n| n["state"] == "Held").count();
    let closed_count = inventory
        .iter()
        .filter(|n| n["state"] == "ConfirmedClosed")
        .count();
    let unknown_count = inventory.iter().filter(|n| n["state"] == "Unknown").count();
    let events = trace["immutableHandleEvents"].as_array().unwrap();
    assert!(!events.is_empty());
    let last = events.last().unwrap();
    let sequence = trace["sequence"].as_u64().unwrap();
    assert_eq!(
        trace["nativeLeaseCompletionHistories"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(trace["originalNativeHandoff"]["historyRoots"], 2);
    if fault == 0 || fault == 86 {
        result.unwrap();
        assert_eq!(closed_count, inventory.len());
        assert_eq!(unknown_count, 0);
        assert_eq!(held_count, 0);
        assert_eq!(trace["originalHandleSummary"]["released"], true);
        assert_eq!(last["operation"], "OriginalHandlesReleasedVerified");
        journal
            .component_check_handoff_history_for_test(end)
            .unwrap();
        // The old original is no longer held by any capture/work/lease clone.
        // Allow a synthetic later-generation writer, but NEVER adopt those new
        // live bytes as an old-history baseline.
        std::fs::write(&config, b"synthetic successor live configuration").unwrap();
        journal
            .component_check_handoff_history_for_test(end)
            .unwrap();
        if fault == 86 {
            let archive = trace["immutableHistoryEvents"][1]["references"][0]["directory"]
                .as_str()
                .unwrap();
            let foreign = Path::new(archive).join("synthetic-handoff-foreign");
            drop(create_private_directory(&foreign).unwrap());
            assert!(journal
                .component_check_handoff_history_for_test(end)
                .is_err());
            assert!(foreign.is_dir());
            let after = journal.component_handle_trace_for_test().unwrap();
            assert_eq!(after["currentActual"], trace["currentActual"]);
            assert_eq!(
                after["immutableHandleEvents"],
                trace["immutableHandleEvents"]
            );
        }
    } else {
        let error = result.unwrap_err();
        println!("r6 original-handle-close fault={fault} error={error}");
        assert_eq!(trace["poisoned"], true);
        assert_eq!(trace["originalHandleSummary"]["released"], false);
        assert_eq!(
            closed_count,
            match fault {
                80 | 85 => 0,
                81 => 1,
                _ => inventory.len(),
            }
        );
        assert_eq!(unknown_count, usize::from(fault == 85));
        assert_eq!(held_count, inventory.len() - closed_count - unknown_count);
        assert_eq!(
            last["operation"],
            if matches!(fault, 83 | 84) {
                "OriginalHandlesReleasedVerified"
            } else {
                "OriginalHandlesReleaseIntent"
            }
        );
        assert_eq!(
            last["sequence"].as_u64(),
            Some(sequence + u64::from(matches!(fault, 83 | 84)))
        );
        assert_eq!(trace["currentActualBytes"].as_u64() == Some(1), fault == 84);
        if fault == 85 {
            assert!(inventory
                .iter()
                .any(|n| n["state"] == "Unknown" && n["rawError"].is_number()));
        }
        assert!(journal.check(end).is_err());
        assert_eq!(
            journal.component_handle_trace_for_test().unwrap(),
            trace,
            "no retry/catch-up or CloseHandle from serialized data"
        );
        assert_eq!(std::fs::read(&config).unwrap(), original);
    }
    println!(
        "r6-original-handle-handoff-native={}",
        serde_json::json!({"fault":fault,"durable":trace,"held":held_count,"closed":closed_count,"unknown":unknown_count,"Closed":false})
    );
    drop(journal);
    f.retain();
}
#[test]
fn wl2_r6_handles_handoff_native_exact_ownership_close_then_history_ignores_successor_live_tree() {
    run(0);
}
#[test]
fn wl2_r6_handles_handoff_native_close_and_current_faults_preserve_held_closed_unknown_prefix() {
    for fault in [80, 81, 82, 83, 84, 85] {
        run(fault);
    }
}
#[test]
fn wl2_r6_handles_handoff_native_history_foreign_membership_refuses_without_rewriting_past() {
    run(86);
}
