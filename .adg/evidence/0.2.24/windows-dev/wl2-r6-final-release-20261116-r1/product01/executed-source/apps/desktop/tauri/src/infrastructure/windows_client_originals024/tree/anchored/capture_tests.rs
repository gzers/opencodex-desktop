use super::super::tests::{compete, driver::setup_unsnapshotted};
use super::*;
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
    let original = core.trees[0]
        .nodes
        .iter()
        .position(|n| n.original.relative == "config.toml")
        .unwrap();
    let custody = core.trees[0].original_location(original);
    let captured = std::fs::read(&custody).unwrap();
    assert_eq!(captured, b"synthetic original config");
    std::fs::write(&config, &captured).unwrap();
    assert_eq!(std::fs::read(&custody).unwrap(), captured);
    super::super::segments::reacquire_trees(
        &mut core.trees,
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
        assert!(core.trees.iter().all(|t| t.poisoned));
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
        &mut core.trees,
        &mut core.journal,
        &mut lease,
        None,
        end
    )
    .is_err());
    assert!(lease.is_none());
    assert!(core.trees.iter().all(|t| t.poisoned));
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
        assert!(core.trees.iter().all(|t| t.poisoned));
        let moved = core
            .trees
            .iter()
            .flat_map(|t| &t.nodes)
            .filter(|n| n.state == State::Moved)
            .count();
        assert_eq!(moved, if fault == 3 { 1 } else { 0 });
        if fault == 3 {
            let tree = &core.trees[0];
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
