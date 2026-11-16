use super::super::super::tests::driver::setup_unsnapshotted as setup;
use super::batch_tests::restore_batched as restore;
use super::*;
use std::time::Duration;
fn run(fault: u8) {
    let (mut f, _data, _paths, held, mut trees, mut journal, original_end) = setup();
    let mut lease = Some(held);
    restore(&mut trees, &mut journal, &mut lease, original_end);
    let root = trees[0].root.clone();
    let mut claim = lease
        .as_mut()
        .unwrap()
        .anchored_claim(&root, original_end)
        .unwrap();
    let witness = recovery_witness(&mut trees, &mut claim, original_end).unwrap();
    drop(claim);
    journal
        .component_release_recovery_for_test(witness, &mut lease, original_end)
        .unwrap();
    journal
        .component_original_history_for_test(borrow_history(&mut trees), original_end)
        .unwrap();
    let close_fault = if matches!(fault, 80 | 81 | 83 | 84 | 85 | 87) {
        fault
    } else {
        0
    };
    journal.inject_driver_fault(close_fault);
    let closed = journal.component_close_original_handles_for_test(trees, original_end);
    let receipt = journal.component_completion_for_test();
    if let Err(error) = &closed {
        println!(
            "r6-completed-history-attempt={}",
            serde_json::json!({"case":fault,"error":error.to_string(),"completion":receipt,
        "durable":journal.component_handle_trace_for_test().unwrap()})
        );
    }
    if close_fault != 0 {
        let error = closed.unwrap_err();
        assert!(receipt.is_null());
        let before = journal.component_handle_trace_for_test().unwrap();
        let rows = before["originalNativeHandoff"]["handles"]["rows"]
            .as_array()
            .unwrap();
        assert!(
            !rows.is_empty(),
            "requested negative must reach actual owned batch"
        );
        let actual_closed = rows
            .iter()
            .filter(|n| n["state"] == "ConfirmedClosed")
            .count();
        let actual_unknown = rows.iter().filter(|n| n["state"] == "Unknown").count();
        assert_eq!(
            actual_closed,
            match fault {
                80 | 85 => 0,
                81 => 1,
                _ => rows.len(),
            },
            "upstream timeout cannot replace requested native prefix"
        );
        assert_eq!(actual_unknown, usize::from(fault == 85));
        if fault != 87 {
            assert_ne!(
                error.kind(),
                io::ErrorKind::TimedOut,
                "upstream timeout is NOT the requested negative"
            );
        }
        match fault {
            80 => assert!(error
                .to_string()
                .contains("durable exact handle-close intent")),
            81 => assert!(error.to_string().contains("first confirmed native")),
            83 => assert!(error
                .to_string()
                .contains("immutable event sync/readback before current CAS")),
            84 => {
                assert!(error.to_string().contains("partial current write"));
                assert_eq!(before["currentActualBytes"], 1);
            }
            85 => assert!(rows
                .iter()
                .any(|n| n["state"] == "Unknown" && n["rawError"].is_number())),
            87 => {
                assert_eq!(before["originalHandleSummary"]["released"], true);
                assert_eq!(error.kind(), io::ErrorKind::TimedOut);
            }
            _ => panic!(),
        }
        let when = Instant::now();
        let end = when + Duration::from_secs(5);
        assert!(journal
            .component_completed_history_at_for_test(when, end)
            .is_err());
        let after = journal.component_handle_trace_for_test().unwrap();
        assert_eq!(before["currentActualSha256"], after["currentActualSha256"]);
        assert_eq!(
            before["immutableHandleEvents"],
            after["immutableHandleEvents"]
        );
        println!(
            "r6-completed-history-native={}",
            serde_json::json!({"case":fault,"closeError":error.to_string(),"completion":receipt,"freshReadRefused":true,"durable":after,"typedClosed":false})
        );
        drop(journal);
        f.retain();
        return;
    }
    closed.unwrap();
    assert_eq!(receipt["completedInOriginalWindow"], true);
    let sealed = journal.component_handle_trace_for_test().unwrap();
    // This wait creates no mutation budget. The original transaction completed
    // before its unchanged cutoff. A NEW caller-owned read window comes later.
    while Instant::now() < original_end {
        std::thread::sleep(Duration::from_millis(5));
    }
    let when = Instant::now();
    let end = when + Duration::from_secs(5);
    assert_eq!(
        journal.component_completion_for_test()["originalDeadlineExpiredNow"],
        true
    );
    std::fs::write(
        root.join("config.toml"),
        b"synthetic next action live config after original cutoff",
    )
    .unwrap();
    if fault == 0 {
        journal
            .component_completed_history_at_for_test(when, end)
            .unwrap();
        let read = journal.component_handle_trace_for_test().unwrap();
        assert_eq!(read["currentActualSha256"], sealed["currentActualSha256"]);
        assert_eq!(
            read["immutableHandleEvents"],
            sealed["immutableHandleEvents"]
        );
        // Actual append still passes the ORIGINAL guard and times out before
        // event replay/write. A fresh successful historical read cannot renew it.
        let error = journal
            .component_expired_append_for_test(original_end)
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        let refused = journal.component_handle_trace_for_test().unwrap();
        assert_eq!(
            refused["currentActualSha256"],
            sealed["currentActualSha256"]
        );
        println!(
            "r6-completed-history-native={}",
            serde_json::json!({"case":fault,"completion":receipt,"originalDeadlineExpired":true,
            "freshReadSucceeded":true,"lateMutationRefused":true,"lateMutationError":error.to_string(),"durable":refused,"typedClosed":false})
        );
    } else {
        match fault {
            90 => journal.component_drift_completion_context_for_test(),
            91 => {
                let dir = sealed["immutableHistoryEvents"][1]["references"][0]["directory"]
                    .as_str()
                    .unwrap();
                drop(
                    create_private_directory(
                        &Path::new(dir).join("synthetic-completed-history-foreign"),
                    )
                    .unwrap(),
                );
            }
            95 => journal
                .component_corrupt_completed_current_for_test()
                .unwrap(),
            _ => {}
        }
        let (read_when, read_end) = match fault {
            92 => (when, when + Duration::from_secs(6)),
            93 => (
                when + Duration::from_secs(60),
                when + Duration::from_secs(65),
            ),
            94 => (original_end - Duration::from_secs(5), original_end),
            _ => (when, end),
        };
        if fault == 97 {
            let window = crate::modules::runtime::official_lifecycle024::ActionWindow::begin(
                crate::modules::process::LifecycleAction::Start,
                when,
                when + Duration::from_secs(30),
            )
            .unwrap();
            let error = journal
                .completed_stop_history_for_action(&window)
                .unwrap_err();
            assert_eq!(
                error.kind(),
                io::ErrorKind::InvalidData,
                "fresh original action read cannot mint final Stop from old handle close"
            );
        } else if fault == 96 {
            assert!(journal.completed_stop_history_at(read_when,read_end).is_err(),
                "timely original handle close receipt alone is not an actual final typed Stop completion");
        } else {
            assert!(journal
                .component_completed_history_at_for_test(read_when, read_end)
                .is_err());
        }
        let after = journal.component_handle_trace_for_test().unwrap();
        if fault == 95 {
            assert_eq!(after["currentActualBytes"], 1);
        } else {
            assert_eq!(after["currentActualSha256"], sealed["currentActualSha256"]);
        }
        assert_eq!(
            after["immutableHandleEvents"],
            sealed["immutableHandleEvents"]
        );
        assert_eq!(after["poisoned"], true);
        println!(
            "r6-completed-history-native={}",
            serde_json::json!({"case":fault,"completion":receipt,"originalDeadlineExpired":true,
            "freshReadRefused":true,"durable":after,"typedClosed":false})
        );
    }
    drop(journal);
    f.retain();
}
#[test]
fn wl2_r6_completed_history_native_original_expiry_allows_fresh_read_but_never_append() {
    run(0);
}
#[test]
fn wl2_r6_completed_history_native_original_close_alone_cannot_sign_final_stop() {
    run(96);
}
#[test]
fn wl2_r6_completed_history_native_new_action_read_cannot_convert_old_close_to_final_stop() {
    run(97);
}
#[test]
fn wl2_r6_completed_history_native_missing_partial_late_unknown_close_never_mint_receipt() {
    for fault in [80, 81, 83, 84, 85, 87] {
        run(fault);
    }
}
#[test]
fn wl2_r6_completed_history_native_drift_foreign_and_wrong_read_windows_keep_exact_prefix() {
    for fault in [90, 91, 92, 93, 94, 95] {
        run(fault);
    }
}
