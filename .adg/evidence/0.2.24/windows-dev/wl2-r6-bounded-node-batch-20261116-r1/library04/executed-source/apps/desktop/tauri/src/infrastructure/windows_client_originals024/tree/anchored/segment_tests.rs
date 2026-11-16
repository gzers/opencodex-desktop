use super::super::tests::{bun_claim, compete, driver::setup};
use super::*;
use crate::infrastructure::windows_client_originals024::v3::FileAction;
fn files(trees: &[AnchoredTree; 2]) -> Vec<(usize, usize)> {
    trees
        .iter()
        .enumerate()
        .flat_map(|(r, t)| {
            t.nodes
                .iter()
                .enumerate()
                .filter_map(move |(i, n)| (n.kind == Kind::File).then_some((r, i)))
        })
        .collect()
}
fn work(
    trees: &mut [AnchoredTree; 2],
    journal: &mut Journal,
    lease: &mut NativeLease,
    end: Instant,
) {
    let root = trees[0].root.clone();
    let mut claim = lease.anchored_claim(&root, end).unwrap();
    for action in [FileAction::MoveOriginal, FileAction::CreateWork] {
        for (r, i) in files(trees) {
            trees[r]
                .journal_step(i, action, journal, &mut claim, end)
                .unwrap();
        }
    }
}
#[test]
fn wl2_r6_lease_segments_explicit_release_same_window_reacquire_and_restore() {
    let (mut f, _data, paths, lease, mut trees, mut journal, end) = setup();
    let mut lease = Some(lease);
    let root = trees[0].root.clone();
    work(&mut trees, &mut journal, lease.as_mut().unwrap(), end);
    let old = lease
        .as_mut()
        .unwrap()
        .anchored_claim(&root, end)
        .unwrap()
        .epoch()
        .to_string();
    bun_claim(&paths[0], end, true);
    release_trees(&mut trees, &mut journal, &mut lease, end).unwrap();
    assert!(lease.is_none());
    assert_eq!(journal.lease_state_for_test(), "Released");
    assert_eq!(journal.native_progress_for_test(), (20, false));
    bun_claim(&paths[0], end, false); // Only claim SQLite; no official imports/client writes.
    reacquire_trees(&mut trees, &mut journal, &mut lease, None, end).unwrap();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    assert_ne!(claim.epoch(), old);
    assert_eq!(journal.native_progress_for_test(), (22, false));
    assert_eq!(journal.lease_state_for_test(), "Active");
    compete(&paths[0], true);
    for action in [FileAction::RetireWork, FileAction::RestoreOriginal] {
        for (r, i) in files(&trees) {
            trees[r]
                .journal_step(i, action, &mut journal, &mut claim, end)
                .unwrap();
        }
    }
    drop(claim);
    release_trees(&mut trees, &mut journal, &mut lease, end).unwrap();
    assert!(lease.is_none());
    assert_eq!(journal.native_progress_for_test(), (36, false));
    assert_eq!(journal.lease_state_for_test(), "Released");
    for tree in &mut trees {
        tree.verify_after_release(end).unwrap();
    }
    journal.check(end).unwrap();
    bun_claim(&paths[0], end, false);
    println!("r6-segment-success original same deadline, all4 exact object/security/content digest, old epoch={old};36 same-chain node+lease events, explicit native receipts retained, full trees restored+stable proof. No official writable interval/new action or Ready/Closed authority.");
    drop(journal);
    drop(trees);
    f.retain();
}
#[test]
fn wl2_r6_lease_segments_native_release_and_acquire_result_failures_preserve_pending() {
    for fault in [7, 8, 9, 10, 11, 12] {
        let (mut f, _data, paths, lease, mut trees, mut journal, end) = setup();
        let mut lease = Some(lease);
        work(&mut trees, &mut journal, lease.as_mut().unwrap(), end);
        if fault <= 9 {
            journal.inject_driver_fault(fault);
            let e = release_trees(&mut trees, &mut journal, &mut lease, end)
                .err()
                .unwrap();
            assert!(lease.is_none());
            assert_eq!(journal.lease_state_for_test(), "ReleasePrepared");
            assert_eq!(journal.native_progress_for_test(), (19, true));
            compete(&paths[0], false);
            println!("r6-segment-fault={fault} explicitlyReleased=true pending=ReleasePrepared error={e}");
        } else {
            release_trees(&mut trees, &mut journal, &mut lease, end).unwrap();
            journal.inject_driver_fault(fault);
            let e = reacquire_trees(&mut trees, &mut journal, &mut lease, None, end)
                .err()
                .unwrap();
            assert!(lease.is_some());
            assert_eq!(journal.lease_state_for_test(), "AcquirePrepared");
            assert_eq!(journal.native_progress_for_test(), (21, true));
            compete(&paths[0], true);
            println!("r6-segment-fault={fault} actualNewLeaseHeld=true pending=AcquirePrepared error={e}");
        }
        assert!(trees.iter().all(|t| t.poisoned));
        assert!(journal.check(end).is_err());
        if let Some(held) = lease.take() {
            held.release(end).unwrap();
        }
        drop(journal);
        drop(trees);
        f.retain();
    }
}
#[test]
fn wl2_r6_lease_segments_prepared_write_failure_keeps_original_locks() {
    for fault in [1, 2] {
        let (mut f, _data, paths, lease, mut trees, mut journal, end) = setup();
        let mut lease = Some(lease);
        work(&mut trees, &mut journal, lease.as_mut().unwrap(), end);
        journal.inject_driver_fault(fault);
        assert!(release_trees(&mut trees, &mut journal, &mut lease, end).is_err());
        assert!(lease.is_some());
        assert_eq!(journal.native_progress_for_test(), (18, true));
        compete(&paths[0], true);
        lease.take().unwrap().release(end).unwrap();
        println!("r6-segment-prepare-fault={fault} no native unlock before durable prepared/current; own failure cleanup explicit, not success");
        drop(journal);
        drop(trees);
        f.retain();
    }
}
#[test]
fn wl2_r6_lease_segments_drop_without_receipt_partial_tree_and_deadline_extension_refuse() {
    for case in 0..3 {
        let (mut f, _data, _paths, lease, mut trees, mut journal, end) = setup();
        let mut lease = Some(lease);
        if case == 1 {
            let root = trees[0].root.clone();
            let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
            let (r, i) = files(&trees)[0];
            trees[r]
                .journal_step(i, FileAction::MoveOriginal, &mut journal, &mut claim, end)
                .unwrap();
            drop(claim);
            assert!(release_trees(&mut trees, &mut journal, &mut lease, end).is_err());
            assert!(lease.is_some());
        } else {
            work(&mut trees, &mut journal, lease.as_mut().unwrap(), end);
            if case == 0 {
                let root = trees[0].root.clone();
                let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
                let plans = trees.each_ref().map(|t| t.plan());
                journal
                    .prepare_lease_release(&plans, &mut claim, end)
                    .unwrap();
                drop(claim);
                drop(lease.take());
                assert!(journal.finish_lease_release(&plans, end).is_err());
                assert_eq!(journal.lease_state_for_test(), "ReleasePrepared");
            } else {
                release_trees(&mut trees, &mut journal, &mut lease, end).unwrap();
                assert!(reacquire_trees(
                    &mut trees,
                    &mut journal,
                    &mut lease,
                    None,
                    end + std::time::Duration::from_millis(1)
                )
                .is_err());
                assert!(lease.is_none());
            }
        }
        assert!(journal.check(end).is_err());
        if let Some(held) = lease.take() {
            held.release(end).unwrap();
        }
        println!("r6-segment-refusal case={case} (dropNoReceipt/partialTree/extendedDeadline), no successor permit");
        drop(journal);
        drop(trees);
        f.retain();
    }
}
#[test]
fn wl2_r6_lease_segments_foreign_coordination_bytes_and_foreign_empty_dir_refuse_reacquire() {
    for case in 0..2 {
        let (mut f, _data, paths, lease, mut trees, mut journal, end) = setup();
        let mut lease = Some(lease);
        work(&mut trees, &mut journal, lease.as_mut().unwrap(), end);
        release_trees(&mut trees, &mut journal, &mut lease, end).unwrap();
        if case == 0 {
            std::fs::write(
                &paths[1],
                b"synthetic unexpected foreign coordination content",
            )
            .unwrap();
        } else {
            std::fs::create_dir(trees[1].root.join("foreign-empty")).unwrap();
        }
        assert!(reacquire_trees(&mut trees, &mut journal, &mut lease, None, end).is_err());
        assert!(lease.is_none());
        assert!(journal.check(end).is_err());
        println!("r6-segment-foreign={case} preserve original custody/foreign object, actual reacquire not admitted, no delete or repair");
        drop(journal);
        drop(trees);
        f.retain();
    }
}
#[test]
fn wl2_r6_lease_segments_actual_busy_claim_refuses_without_release_retry_or_deadline_reset() {
    let (mut f, _data, paths, lease, mut trees, mut journal, end) =
        super::super::tests::driver::setup_initialized_claim();
    let mut lease = Some(lease);
    work(&mut trees, &mut journal, lease.as_mut().unwrap(), end);
    release_trees(&mut trees, &mut journal, &mut lease, end).unwrap();
    let c = rusqlite::Connection::open(&paths[0]).unwrap();
    c.busy_timeout(std::time::Duration::ZERO).unwrap();
    c.execute_batch("BEGIN EXCLUSIVE").unwrap();
    println!(
        "r6-busy-admission before-reacquire remainingMs={} codexChildren={:?}",
        end.saturating_duration_since(Instant::now()).as_millis(),
        names(&trees[0].root)
    );
    for (i, tree) in trees.iter_mut().enumerate() {
        println!(
            "r6-busy-tree-observation root={i} result={:?}",
            tree.verify_image(end)
        );
    }
    let error = reacquire_trees(&mut trees, &mut journal, &mut lease, None, end)
        .err()
        .unwrap();
    println!("r6-busy-admission after-reacquire remainingMs={} kind={:?} raw={:?} error={error} progress={:?} state={}",end.saturating_duration_since(Instant::now()).as_millis(),error.kind(),error.raw_os_error(),journal.native_progress_for_test(),journal.lease_state_for_test());
    assert_eq!(
        error.raw_os_error(),
        Some(33),
        "original busy claim assertion; no accepting an earlier refusal as the raw33 positive"
    );
    assert!(lease.is_none());
    assert_eq!(journal.native_progress_for_test(), (21, true));
    assert_eq!(journal.lease_state_for_test(), "AcquirePrepared");
    c.execute_batch("ROLLBACK").unwrap();
    c.close().unwrap();
    assert!(reacquire_trees(&mut trees, &mut journal, &mut lease, None, end).is_err());
    assert!(lease.is_none());
    println!("r6-segment-real-busy raw={:?} foreign claim transaction untouched; native acquire failure leaves canonical pending, no release-retry after foreign unlock, no renewed deadline",error.raw_os_error());
    drop(journal);
    drop(trees);
    f.retain();
}
