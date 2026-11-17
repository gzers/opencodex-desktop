use super::super::super::{
    segments::release_trees,
    tests::{bun_claim, driver::setup},
};
use super::*;
use crate::infrastructure::windows_client_originals024::v3::FileAction;
fn prepare_observed(
    trees: &mut [AnchoredTree; 2],
    journal: &mut Journal,
    lease: &mut crate::infrastructure::windows_client_originals024::coordination::NativeLease,
    end: Instant,
) {
    let root = trees[0].root.clone();
    let mut claim = lease.anchored_claim(&root, end).unwrap();
    let files = trees
        .iter()
        .enumerate()
        .flat_map(|(r, t)| {
            t.nodes
                .iter()
                .enumerate()
                .filter_map(move |(i, n)| (n.kind == Kind::File).then_some((r, i)))
        })
        .collect::<Vec<_>>();
    for action in [FileAction::MoveOriginal, FileAction::CreateWork] {
        for (r, i) in &files {
            trees[*r]
                .journal_step(*i, action, journal, &mut claim, end)
                .unwrap();
        }
    }
    open_trees(trees, journal, &mut claim, end).unwrap();
    observe_trees(trees, journal, &mut claim, end).unwrap();
}
#[test]
fn wl2_r6_snapshot_links_both_native_archives_canonical_current_and_retained_history() {
    let (mut f, _data, paths, held, mut trees, mut journal, end) = setup();
    let mut lease = Some(held);
    prepare_observed(&mut trees, &mut journal, lease.as_mut().unwrap(), end);
    let root = trees[0].root.clone();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    assert_eq!(
        journal.snapshot_state_for_test(),
        (false, 0, "Preparing".into())
    );
    link_observation(&mut trees, &mut journal, &mut claim, false, end).unwrap();
    assert_eq!(journal.native_progress_for_test(), (24, false));
    assert_eq!(
        journal.snapshot_state_for_test(),
        (false, 1, "Preparing".into())
    );
    let refs = witness(&mut trees, end).unwrap().references(end).unwrap();
    for (i, r) in refs.iter().enumerate() {
        assert_eq!(r["context"]["purpose"], "Observation");
        assert_eq!(r["context"]["observationSequence"], 22);
        assert_eq!(
            r["volume"].as_u64(),
            Some(trees[i].parent.identity().volume as u64)
        );
    }
    println!(
        "r6-linked-native-snapshot-references={}",
        serde_json::json!(refs)
    );
    accept_unchanged_trees(&mut trees, &mut journal, &mut claim, end).unwrap();
    assert!(trees.iter().all(|t| t.snapshot_history.len() == 1));
    let files = trees
        .iter()
        .enumerate()
        .flat_map(|(r, t)| {
            t.nodes
                .iter()
                .enumerate()
                .filter_map(move |(i, n)| (n.kind == Kind::File).then_some((r, i)))
        })
        .collect::<Vec<_>>();
    for action in [FileAction::RetireWork, FileAction::RestoreOriginal] {
        for (r, i) in &files {
            trees[*r]
                .journal_step(*i, action, &mut journal, &mut claim, end)
                .unwrap();
        }
    }
    drop(claim);
    release_trees(&mut trees, &mut journal, &mut lease, end).unwrap();
    for tree in &mut trees {
        tree.verify_after_release(end).unwrap();
    }
    assert_eq!(journal.native_progress_for_test(), (39, false));
    assert_eq!(
        journal.snapshot_state_for_test(),
        (false, 1, "Preparing".into())
    );
    journal.check(end).unwrap();
    bun_claim(&paths[0], end, false);
    println!("r6-snapshot-links canonical intent precedes both actual archive writes; exact references linked, current link digest retains immutable histories after native source handles return; full original tree restored+explicit lease release, lifecycle still Preparing, never native Ready/Closed");
    drop(journal);
    drop(trees);
    f.retain();
}
#[test]
fn wl2_r6_snapshot_links_durable_intent_archive_result_and_partial_current_faults_keep_pending() {
    for fault in [17, 18, 19, 20] {
        let (mut f, _data, _paths, mut lease, mut trees, mut journal, end) = setup();
        prepare_observed(&mut trees, &mut journal, &mut lease, end);
        let root = trees[0].root.clone();
        let mut claim = lease.anchored_claim(&root, end).unwrap();
        journal.inject_driver_fault(fault);
        let error = link_observation(&mut trees, &mut journal, &mut claim, false, end)
            .err()
            .unwrap();
        assert_eq!(journal.native_progress_for_test(), (23, true));
        assert_eq!(
            journal.snapshot_state_for_test(),
            (true, 0, "Preparing".into())
        );
        assert!(journal.check(end).is_err());
        let archives = trees
            .iter()
            .filter(|t| t.observed.as_ref().is_some_and(|f| f.archive.is_some()))
            .count();
        assert_eq!(archives, if fault == 17 { 0 } else { 2 });
        println!("r6-snapshot-link-fault={fault} archives={archives} actualPendingSnapshot=true sequence23 previousFrame22 error={error}; no repaired result/current or text Ready");
        drop(claim);
        lease.release(end).unwrap();
        drop(journal);
        drop(trees);
        f.retain();
    }
}
#[test]
fn wl2_r6_snapshot_links_partial_archive_and_missing_second_root_never_publish_completion() {
    for missing in [false, true] {
        let (mut f, _data, _paths, mut lease, mut trees, mut journal, end) = setup();
        prepare_observed(&mut trees, &mut journal, &mut lease, end);
        let root = trees[0].root.clone();
        let mut claim = lease.anchored_claim(&root, end).unwrap();
        let context = journal
            .begin_observation_snapshot_link(&mut claim, end)
            .unwrap();
        if missing {
            trees[0]
                .observed
                .as_mut()
                .unwrap()
                .archive(root.parent().unwrap(), &context, false, end)
                .unwrap();
            let error = journal
                .finish_snapshot_link(witness(&mut trees, end).unwrap(), &mut claim, end)
                .err()
                .unwrap();
            println!("r6-snapshot-link-missing-second error={error}");
        } else {
            assert!(
                create_snapshots(&mut trees, &mut journal, &context, &mut claim, true, end)
                    .is_err()
            );
        }
        assert_eq!(
            journal.snapshot_state_for_test(),
            (true, 0, "Preparing".into())
        );
        assert!(journal.check(end).is_err());
        assert!(trees[0].observed.as_ref().unwrap().archive.is_some());
        drop(claim);
        lease.release(end).unwrap();
        drop(journal);
        drop(trees);
        f.retain();
    }
}
#[test]
fn wl2_r6_snapshot_links_duplicate_purpose_and_unknown_archive_refuse_without_new_authority() {
    for foreign in [false, true] {
        let (mut f, _data, _paths, mut lease, mut trees, mut journal, end) = setup();
        prepare_observed(&mut trees, &mut journal, &mut lease, end);
        let root = trees[0].root.clone();
        let mut claim = lease.anchored_claim(&root, end).unwrap();
        link_observation(&mut trees, &mut journal, &mut claim, false, end).unwrap();
        if foreign {
            let reference = trees[0]
                .observed
                .as_mut()
                .unwrap()
                .archive
                .as_mut()
                .unwrap()
                .reference(end)
                .unwrap();
            let directory = PathBuf::from(reference["directory"].as_str().unwrap());
            drop(create_private_directory(&directory.join("foreign-empty")).unwrap());
            assert!(witness(&mut trees, end).is_err());
        } else {
            assert!(journal
                .begin_observation_snapshot_link(&mut claim, end)
                .is_err());
            assert_eq!(
                journal.snapshot_state_for_test(),
                (false, 1, "Preparing".into())
            );
        }
        println!("r6-snapshot-unknown-or-duplicate foreign={foreign} exact existing snapshot retained; no new link/Ready/Closed from strings");
        drop(claim);
        lease.release(end).unwrap();
        drop(journal);
        drop(trees);
        f.retain();
    }
}
