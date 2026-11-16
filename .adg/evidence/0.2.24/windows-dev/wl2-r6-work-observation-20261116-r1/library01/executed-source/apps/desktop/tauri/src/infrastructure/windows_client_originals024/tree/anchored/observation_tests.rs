use super::super::{
    segments::release_trees,
    tests::{
        bun_claim, compete,
        driver::{setup, setup_artifacts},
    },
};
use super::*;
use crate::infrastructure::windows_client_originals024::{
    coordination::NativeLease,
    v3::{FileAction, Journal},
};
fn work(
    trees: &mut [AnchoredTree; 2],
    journal: &mut Journal,
    lease: &mut NativeLease,
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
}
#[test]
fn wl2_r6_work_observation_original_handles_allow_writes_then_exact_unchanged_restore() {
    let (mut f, _data, paths, held, mut trees, mut journal, end) = setup();
    let mut lease = Some(held);
    work(&mut trees, &mut journal, lease.as_mut().unwrap(), end);
    let root = trees[0].root.clone();
    let path = root.join("config.toml");
    let bytes = std::fs::read(&path).unwrap();
    assert!(
        std::fs::write(&path, &bytes).is_err(),
        "creation handle denies external writes before canonical open"
    );
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    open_trees(&mut trees, &mut journal, &mut claim, end).unwrap();
    drop(claim);
    assert!(trees
        .iter()
        .flat_map(|t| &t.nodes)
        .filter(|n| n.kind == Kind::File)
        .all(|n| n.work_open));
    release_trees(&mut trees, &mut journal, &mut lease, end).unwrap();
    bun_claim(&paths[0], end, false);
    std::fs::write(&path, &bytes).unwrap(); // Same generation/content, synthetic ordinary writer only.
    reacquire_for_observation(&mut trees, &mut journal, &mut lease, None, end).unwrap();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    observe_trees(&mut trees, &mut journal, &mut claim, end).unwrap();
    assert_eq!(journal.work_mode_for_test(), "Observed");
    assert!(
        std::fs::write(&path, &bytes).is_err(),
        "native frozen frame denies a later writer"
    );
    accept_unchanged_trees(&mut trees, &mut journal, &mut claim, end).unwrap();
    assert_eq!(journal.work_mode_for_test(), "Sealed");
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
    assert!(lease.is_none());
    journal.check(end).unwrap();
    println!("r6-work-observation unchanged generation: native observer permits actual writes only after canonical open/release; full frozen frames deny subsequent writes; exact unchanged whole-tree proof re-enables canonical native retirement/restore; original custody+anchors/all8 intact, no production Ready/Closed");
    drop(journal);
    drop(trees);
    f.retain();
}
#[test]
fn wl2_r6_work_observation_changed_artifact_is_data_not_adoption_or_restore_permission() {
    let (mut f, _data, _paths, held, mut trees, mut journal, end) = setup_artifacts();
    let mut lease = Some(held);
    work(&mut trees, &mut journal, lease.as_mut().unwrap(), end);
    let root = trees[0].root.clone();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    open_trees(&mut trees, &mut journal, &mut claim, end).unwrap();
    drop(claim);
    release_trees(&mut trees, &mut journal, &mut lease, end).unwrap();
    let path = root.join("opencodex-catalog.json");
    std::fs::write(&path, b"{\"models\":[{\"slug\":\"synthetic-model\"}]}").unwrap();
    reacquire_for_observation(&mut trees, &mut journal, &mut lease, None, end).unwrap();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    observe_trees(&mut trees, &mut journal, &mut claim, end).unwrap();
    assert_eq!(journal.work_mode_for_test(), "Observed");
    let frame = trees[0].observed.as_mut().unwrap();
    frame.check(end).unwrap();
    frame.artifact_frame().unwrap().require_root(&root).unwrap();
    println!("r6-work-observed-native={}", frame.metadata());
    assert!(accept_unchanged_trees(&mut trees, &mut journal, &mut claim, end).is_err());
    assert!(trees.iter().all(|t| t.poisoned));
    assert!(trees[0].observed.is_some());
    assert_eq!(
        std::fs::read(&path).unwrap(),
        b"{\"models\":[{\"slug\":\"synthetic-model\"}]}"
    );
    drop(claim);
    lease.take().unwrap().release(end).unwrap();
    drop(journal);
    drop(trees);
    f.retain();
}
#[test]
fn wl2_r6_work_observation_unknown_node_empty_dir_and_nonartifact_edit_refuse() {
    for case in 0..3 {
        let (mut f, _data, _paths, held, mut trees, mut journal, end) = setup();
        let mut lease = Some(held);
        work(&mut trees, &mut journal, lease.as_mut().unwrap(), end);
        let root = trees[0].root.clone();
        let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
        open_trees(&mut trees, &mut journal, &mut claim, end).unwrap();
        drop(claim);
        release_trees(&mut trees, &mut journal, &mut lease, end).unwrap();
        match case {
            0 => std::fs::write(root.join("config.toml"), b"synthetic unauthorized edit").unwrap(),
            1 => {
                drop(create_private_directory(&root.join("foreign-empty")).unwrap());
            }
            _ => {
                drop(PrivateRecord::create(&root.join("foreign.txt"), b"unknown", 1024).unwrap());
            }
        }
        reacquire_for_observation(&mut trees, &mut journal, &mut lease, None, end).unwrap();
        let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
        assert!(observe_trees(&mut trees, &mut journal, &mut claim, end).is_err());
        assert!(trees[0].observed.is_some());
        assert!(journal.check(end).is_err());
        println!("r6-work-observation unknown case={case} complete native frame retained before veto; originals remain at exact custody, no overwrite/adoption; children={:?}",names(&root));
        drop(claim);
        lease.take().unwrap().release(end).unwrap();
        drop(journal);
        drop(trees);
        f.retain();
    }
}
#[test]
fn wl2_r6_work_observation_intent_native_result_and_partial_current_faults_refuse() {
    for fault in [13, 14, 15, 16] {
        let (mut f, _data, paths, held, mut trees, mut journal, end) = setup();
        let mut lease = Some(held);
        work(&mut trees, &mut journal, lease.as_mut().unwrap(), end);
        let root = trees[0].root.clone();
        let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
        journal.inject_driver_fault(fault);
        assert!(open_trees(&mut trees, &mut journal, &mut claim, end).is_err());
        assert!(trees.iter().all(|t| t.poisoned));
        assert!(journal.check(end).is_err());
        assert_eq!(journal.work_mode_for_test(), "Opening");
        compete(&paths[0], true);
        println!("r6-work-open-fault={fault} nativeModes={:?} canonical=Opening exact handles retained, no callback continuation",trees.each_ref().map(|t|t.work_mode));
        drop(claim);
        lease.take().unwrap().release(end).unwrap();
        drop(journal);
        drop(trees);
        f.retain();
    }
}
