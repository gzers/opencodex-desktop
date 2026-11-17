use super::super::tests::{compete, driver::setup_unsnapshotted};
use super::*;
#[test]
fn wl2_r6_capture_owned_tree_transfer_retains_native_files_before_fallible_close_and_refuses_mutation(
) {
    let (mut f, _data, _paths, mut lease, trees, journal, end) = setup_unsnapshotted();
    let config = trees[0].root.join("config.toml");
    let original = std::fs::read(&config).unwrap();
    let mut core = CapturedTrees::retain(trees, journal);
    core.journal
        .retain_original_handle_ownership(&mut core.trees, end)
        .unwrap();
    assert!(core.trees.is_none());
    let trace = core.journal.component_handle_trace_for_test().unwrap();
    assert_eq!(trace["originalNativeHandoff"]["treesPending"], true);
    assert_eq!(trace["originalNativeHandoff"]["historyRoots"], 0);
    assert!(trace["originalHandleSummary"].is_null());
    let error =
        std::fs::write(&config, b"must remain blocked by same original handle").unwrap_err();
    assert_eq!(error.raw_os_error(), Some(32));
    assert!(core.prepare_working_batched(&mut lease, end).is_err());
    assert!(core.failed);
    let trace = core.journal.component_handle_trace_for_test().unwrap();
    assert_eq!(trace["originalNativeHandoff"]["treesPending"], true);
    assert_eq!(trace["originalNativeHandoff"]["poisoned"], true);
    assert_eq!(std::fs::read(&config).unwrap(), original);
    assert_eq!(
        std::fs::write(&config, b"no Drop during failed mutation")
            .unwrap_err()
            .raw_os_error(),
        Some(32)
    );
    lease.release(end).unwrap();
    println!("r6 exact owned tree transfer: original files remain native deny-write before any fallible close; later mutation on absent local slot refuses and poisons retained journal handoff, same bytes/raw32 retained; no close/Closed/Drop proof or actual production guard constructed");
    drop(core);
    f.retain();
}
#[test]
fn wl2_r6_capture_owned_tree_duplicate_transfer_never_replaces_or_drops_either_original_set() {
    let (mut f1, _data1, _paths1, lease1, trees1, journal1, end1) = setup_unsnapshotted();
    let (mut f2, _data2, _paths2, lease2, trees2, journal2, end2) = setup_unsnapshotted();
    let config1 = trees1[0].root.join("config.toml");
    let config2 = trees2[0].root.join("config.toml");
    let mut first = Some(trees1);
    let mut second = Some(trees2);
    let mut journal = journal1;
    journal
        .retain_original_handle_ownership(&mut first, end1)
        .unwrap();
    assert!(journal
        .retain_original_handle_ownership(&mut second, end2)
        .is_err());
    assert!(first.is_none() && second.is_some());
    for config in [&config1, &config2] {
        assert_eq!(
            std::fs::write(config, b"duplicate must not drop either set")
                .unwrap_err()
                .raw_os_error(),
            Some(32)
        );
    }
    assert!(journal.component_handle_trace_for_test().unwrap()["originalHandleSummary"].is_null());
    lease1.release(end1).unwrap();
    lease2.release(end2).unwrap();
    println!("r6 duplicate original-tree ownership transfer rejects before take; first journal tree set and second caller tree set both retain exact actual files/raw32; no replacement, native close or new action deadline");
    drop(journal);
    drop(journal2);
    drop(second);
    f1.retain();
    f2.retain();
}
#[test]
fn wl2_r6_capture_stop_prepared_boundary_requires_exact_original_generation_and_deadline() {
    use crate::infrastructure::windows_official_lifecycle024::OriginalActionKind;
    let started = Instant::now();
    let deadline = started + std::time::Duration::from_secs(1);
    let digest = "a".repeat(64);
    let stop = PreparedStop::admit(123, 456, &digest, OriginalActionKind::Stop, deadline).unwrap();
    assert!(stop.check(123, 456, &digest, deadline).is_ok());
    for (pid, port, record, end) in [
        (124, 456, digest.clone(), deadline),
        (123, 457, digest.clone(), deadline),
        (123, 456, "b".repeat(64), deadline),
        (
            123,
            456,
            digest.clone(),
            deadline + std::time::Duration::from_nanos(1),
        ),
        (
            123,
            456,
            digest.clone(),
            deadline - std::time::Duration::from_nanos(1),
        ),
    ] {
        assert!(stop.check(pid, port, &record, end).is_err());
    }
    assert!(PreparedStop::admit(123, 456, &digest, OriginalActionKind::Start, deadline).is_err());
    assert!(PreparedStop::admit(0, 456, &digest, OriginalActionKind::Stop, deadline).is_err());
    assert!(PreparedStop::admit(123, 0, &digest, OriginalActionKind::Restart, deadline).is_err());
    assert!(PreparedStop::admit(123, 456, "bad", OriginalActionKind::Stop, deadline).is_err());
    assert!(PreparedStop::admit(
        123,
        456,
        &digest,
        OriginalActionKind::Stop,
        started - std::time::Duration::from_nanos(1)
    )
    .is_err());
    let historical = PreparedStop {
        pid: 123,
        port: 456,
        record: digest.clone(),
        deadline: started - std::time::Duration::from_nanos(1),
    };
    assert!(historical
        .check(123, 456, &digest, historical.deadline)
        .is_err());
    println!("r6 Stop boundary locator tests only: exact original pid/port/record/end and no renewed/substituted timer. Data cannot authorize Source/Owner/native phase; actual factories remain separate, no fake Ready/NormalExit/Closed");
}
#[test]
fn wl2_r6_capture_ready_initial_open_precedes_release_actual_writer_and_preserves_originals() {
    let (mut f, _data, paths, held, trees, journal, end) = setup_unsnapshotted();
    let mut lease = Some(held);
    let root = trees[0].root.clone();
    let config = root.join("config.toml");
    let mut core = CapturedTrees::retain(trees, journal);
    core.prepare_working(lease.as_mut().unwrap(), end).unwrap();
    assert!(std::fs::write(&config, b"blocked-before-open").is_err());
    core.open_prepared_working(lease.as_mut().unwrap(), end)
        .unwrap();
    assert_eq!(core.journal.native_progress_for_test(), (20, false));
    core.check_working(lease.as_mut().unwrap(), end).unwrap();
    core.release_prepared(&mut lease, end).unwrap();
    assert!(lease.is_none());
    assert_eq!(core.journal.native_progress_for_test(), (22, false));
    compete(&paths[0], false);
    let original = core.trees.as_ref().unwrap()[0]
        .nodes
        .iter()
        .position(|n| n.original.relative == "config.toml")
        .unwrap();
    let custody = core.trees.as_ref().unwrap()[0].original_location(original);
    let captured = std::fs::read(&custody).unwrap();
    assert_eq!(captured, b"synthetic original config");
    std::fs::write(&config, &captured).unwrap();
    assert_eq!(std::fs::read(&custody).unwrap(), captured);
    super::super::segments::reacquire_trees(
        core.trees.as_mut().unwrap(),
        &mut core.journal,
        &mut lease,
        None,
        end,
    )
    .unwrap();
    core.check_working(lease.as_mut().unwrap(), end).unwrap();
    core.release_prepared(&mut lease, end).unwrap();
    assert!(lease.is_none());
    assert!(core.opened);
    assert!(!core.ready);
    assert_eq!(core.journal.native_progress_for_test(), (26, false));
    println!("r6 capture-ready core original18 prepare/20 canonicalOpen/22explicitRelease, actual same-byte writer then full unchanged preResume-like recheck/26release; exact original custody remains; test does not mint actual Source/Owner/typedReady or invoke official creator");
    drop(core);
    f.retain();
}
#[test]
fn wl2_r6_capture_ready_open_faults_keep_installed_core_and_no_successful_publication() {
    for fault in [13, 14, 15, 16] {
        let (mut f, _data, _paths, mut lease, trees, journal, end) = setup_unsnapshotted();
        let mut core = CapturedTrees::retain(trees, journal);
        core.prepare_working(&mut lease, end).unwrap();
        core.journal.inject_driver_fault(fault);
        assert!(core.open_prepared_working(&mut lease, end).is_err());
        assert!(core.failed);
        assert!(!core.opened);
        assert!(!core.ready);
        assert_eq!(core.journal.work_mode_for_test(), "Opening");
        assert!(core.trees.as_ref().unwrap().iter().all(|t| t.poisoned));
        assert!(core.open_prepared_working(&mut lease, end).is_err());
        assert!(core.journal.check(end).is_err());
        println!("r6-capture-ready-open-fault={fault} installed core keeps exact original/work handles and Opening/current pending; no retry or typedReady state");
        lease.release(end).unwrap();
        drop(core);
        f.retain();
    }
}
#[test]
fn wl2_r6_capture_ready_changed_initial_work_before_official_creator_runs_refuses_recheck() {
    let (mut f, _data, _paths, held, trees, journal, end) = setup_unsnapshotted();
    let mut lease = Some(held);
    let root = trees[0].root.clone();
    let mut core = CapturedTrees::retain(trees, journal);
    core.prepare_working(lease.as_mut().unwrap(), end).unwrap();
    core.open_prepared_working(lease.as_mut().unwrap(), end)
        .unwrap();
    core.release_prepared(&mut lease, end).unwrap();
    std::fs::write(root.join("config.toml"), b"unexpected pre-creator mutation").unwrap();
    assert!(super::super::segments::reacquire_trees(
        core.trees.as_mut().unwrap(),
        &mut core.journal,
        &mut lease,
        None,
        end
    )
    .is_err());
    assert!(lease.is_none());
    assert!(core.trees.as_ref().unwrap().iter().all(|t| t.poisoned));
    assert!(!core.ready);
    println!("r6 capture-ready unexpected work edit before actual creator would Resume is not an official mutation, refuses unchanged preResume check; original custody and external edit both retained");
    drop(core);
    f.retain();
}
#[test]
fn wl2_r6_capture_installed_core_precedes_native_moves_and_released_store_survives_thread_handoff()
{
    let (mut f, _data, paths, held, trees, journal, end) = setup_unsnapshotted();
    let mut lease = Some(held);
    assert!(trees
        .iter()
        .flat_map(|t| &t.nodes)
        .all(|n| n.state == State::Original));
    assert_eq!(journal.native_progress_for_test(), (0, false));
    // Same native core used by production CaptureStore; Source/route authority
    // is not faked here. The component is retained before the first file move.
    let mut retained = Some(CapturedTrees::retain(trees, journal));
    let core = retained.as_mut().unwrap();
    core.prepare_working(lease.as_mut().unwrap(), end).unwrap();
    core.check_working(lease.as_mut().unwrap(), end).unwrap();
    assert_eq!(core.journal.native_progress_for_test(), (18, false));
    assert_eq!(
        core.native_prefix(),
        (true, false, vec!["Working".into(); 3])
    );
    compete(&paths[0], true);
    core.release_prepared(&mut lease, end).unwrap();
    assert!(lease.is_none());
    compete(&paths[0], false);
    let core = retained.take().unwrap();
    let mut core = std::thread::spawn(move || {
        let mut core = core;
        core.journal.check(end).unwrap();
        core
    })
    .join()
    .unwrap();
    core.journal.check(end).unwrap();
    assert_eq!(core.journal.native_progress_for_test(), (20, false));
    assert_eq!(core.journal.lease_state_for_test(), "Released");
    println!("r6 retained native core installed at seq0 before first mutation;18 exact anchor/file events prepare all3 work files, explicit20 release; only released historical file/pin objects+full original trees+same journal move across worker and back; no live lease/ranges or production Source/Ready/Closed granted");
    drop(core);
    f.retain();
}
#[test]
fn wl2_r6_capture_faults_keep_installed_original_tree_and_exact_native_prefix_without_retry() {
    for fault in [1, 2, 3] {
        let (mut f, _data, _paths, mut lease, trees, mut journal, end) = setup_unsnapshotted();
        journal.inject_driver_fault(fault);
        let mut retained = Some(CapturedTrees::retain(trees, journal));
        let core = retained.as_mut().unwrap();
        let error = core.prepare_working(&mut lease, end).err().unwrap();
        assert!(retained.is_some());
        let core = retained.as_mut().unwrap();
        assert!(core.failed);
        assert!(!core.prepared);
        assert!(core.trees.as_ref().unwrap().iter().all(|t| t.poisoned));
        let moved = core
            .trees
            .as_ref()
            .unwrap()
            .iter()
            .flat_map(|t| &t.nodes)
            .filter(|n| n.state == State::Moved)
            .count();
        assert_eq!(moved, if fault == 3 { 1 } else { 0 });
        if fault == 3 {
            let tree = &core.trees.as_ref().unwrap()[0];
            let i = tree
                .nodes
                .iter()
                .position(|n| n.state == State::Moved)
                .unwrap();
            assert!(tree.original_location(i).exists());
            assert!(!tree.root.join(&tree.nodes[i].original.relative).exists());
        }
        assert!(core.prepare_working(&mut lease, end).is_err());
        assert!(core.journal.check(end).is_err());
        println!("r6 capture-installed-fault={fault} movedOriginals={moved} exact native parents/handles/journal retained before error={error}; no slot replace, catchup or automatic retry");
        lease.release(end).unwrap();
        drop(retained);
        f.retain();
    }
}
