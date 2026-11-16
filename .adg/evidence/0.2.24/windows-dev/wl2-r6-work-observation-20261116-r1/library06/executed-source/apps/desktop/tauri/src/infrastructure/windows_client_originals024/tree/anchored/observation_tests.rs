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
    let context = journal.observation_context(&mut claim, end).unwrap();
    let reference = frame
        .archive(root.parent().unwrap(), &context, false, end)
        .unwrap();
    assert_eq!(
        reference["volume"].as_u64(),
        Some(frame.parent.volume as u64)
    );
    let directory = PathBuf::from(reference["directory"].as_str().unwrap());
    assert!(std::fs::write(directory.join("snapshot.json"), b"tamper").is_err());
    println!("r6-work-native-archive-reference={reference}");
    frame.check(end).unwrap();
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
fn wl2_r6_work_observation_partial_archive_keeps_native_frame_and_real_chunk_then_refuses() {
    let (mut f, _data, _paths, held, mut trees, mut journal, end) = setup();
    let mut lease = Some(held);
    work(&mut trees, &mut journal, lease.as_mut().unwrap(), end);
    let root = trees[0].root.clone();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    open_trees(&mut trees, &mut journal, &mut claim, end).unwrap();
    observe_trees(&mut trees, &mut journal, &mut claim, end).unwrap();
    let context = journal.observation_context(&mut claim, end).unwrap();
    let frame = trees[0].observed.as_mut().unwrap();
    let error = frame
        .archive(root.parent().unwrap(), &context, true, end)
        .err()
        .unwrap();
    assert!(frame.archive.is_some());
    assert!(frame.archive_failed);
    assert!(frame.check(end).is_err());
    println!("r6-work-partial-native-archive error={error}; actual firstchunk retained with exact private original ID/security; frozen frame remains, no SnapshotReference/Ready or unchanged acceptance");
    assert!(accept_unchanged_trees(&mut trees, &mut journal, &mut claim, end).is_err());
    drop(claim);
    lease.take().unwrap().release(end).unwrap();
    drop(journal);
    drop(trees);
    f.retain();
}
#[test]
fn wl2_r6_work_observation_atomic_generation_and_large_snapshot_parts_keep_exact_native_data() {
    let (mut f, _data, _paths, held, mut trees, mut journal, end) = setup_artifacts();
    let mut lease = Some(held);
    work(&mut trees, &mut journal, lease.as_mut().unwrap(), end);
    let root = trees[0].root.clone();
    let i = trees[0]
        .nodes
        .iter()
        .position(|n| n.original.relative == "opencodex-catalog.json")
        .unwrap();
    let old_id = trees[0].nodes[i].work.as_ref().unwrap().1.id;
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    open_trees(&mut trees, &mut journal, &mut claim, end).unwrap();
    drop(claim);
    release_trees(&mut trees, &mut journal, &mut lease, end).unwrap();
    let temporary = root.join("synthetic-catalog.tmp");
    drop(PrivateRecord::create(&temporary, b"{\"models\":[]}", 1024).unwrap());
    std::fs::rename(&temporary, root.join("opencodex-catalog.json")).unwrap();
    let large = vec![0x78; 1280 * 1024];
    std::fs::write(root.join("models_cache.json"), &large).unwrap();
    reacquire_for_observation(&mut trees, &mut journal, &mut lease, None, end).unwrap();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    observe_trees(&mut trees, &mut journal, &mut claim, end).unwrap();
    let frame = trees[0].observed.as_ref().unwrap();
    let changed = frame
        .nodes
        .iter()
        .find(|n| n.entry.relative == "opencodex-catalog.json")
        .unwrap();
    assert_ne!(changed.entry.id, old_id);
    assert_eq!(
        retained_work_identity(&trees[0].nodes[i].work.as_ref().unwrap().0, old_id).unwrap(),
        old_id,
        "original working generation retained despite external atomic replace"
    );
    let context = journal.observation_context(&mut claim, end).unwrap();
    let frame = trees[0].observed.as_mut().unwrap();
    let reference = frame
        .archive(root.parent().unwrap(), &context, false, end)
        .unwrap();
    let directory = PathBuf::from(reference["directory"].as_str().unwrap());
    let metadata: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("snapshot.json")).unwrap()).unwrap();
    let cache = frame
        .nodes
        .iter()
        .position(|n| n.entry.relative == "models_cache.json")
        .unwrap();
    let parts = metadata["parts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["node"].as_u64() == Some(cache as u64))
        .collect::<Vec<_>>();
    assert_eq!(parts.len(), 2);
    let reconstructed = parts
        .iter()
        .flat_map(|p| std::fs::read(directory.join(p["name"].as_str().unwrap())).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(reconstructed, large);
    frame.check(end).unwrap();
    drop(create_private_directory(&directory.join("foreign-empty")).unwrap());
    assert!(frame.check(end).is_err());
    println!("r6-work-atomic-observation oldFileId={} newFileId={} archive={} exact1280KiB/2parts reconstructed; old observer retained, current data frozen; deliberately invalid cache bytes not content-policy acceptance; unknown archive empty child refuses",old_id.index,frame.nodes.iter().find(|n|n.entry.relative=="opencodex-catalog.json").unwrap().entry.id.index,directory.display());
    drop(claim);
    lease.take().unwrap().release(end).unwrap();
    drop(journal);
    drop(trees);
    f.retain();
}
#[test]
fn wl2_r6_work_observation_ads_hardlink_and_artifact_directory_refuse() {
    for case in 0..3 {
        let (mut f, _data, _paths, held, mut trees, mut journal, end) = setup_artifacts();
        let mut lease = Some(held);
        work(&mut trees, &mut journal, lease.as_mut().unwrap(), end);
        let root = trees[0].root.clone();
        let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
        open_trees(&mut trees, &mut journal, &mut claim, end).unwrap();
        drop(claim);
        release_trees(&mut trees, &mut journal, &mut lease, end).unwrap();
        let path = root.join("opencodex-catalog.json");
        if case == 0 {
            std::fs::write(
                PathBuf::from(format!("{}:synthetic-extra", path.display())),
                b"synthetic ADS",
            )
            .unwrap();
        } else if case == 1 {
            std::fs::hard_link(
                &path,
                root.parent().unwrap().join("synthetic-external-hardlink"),
            )
            .unwrap();
        } else {
            // Keep the original work object as a still-open renamed generation;
            // an exact allowed artifact NAME still cannot denote a directory.
            std::fs::rename(
                &path,
                root.parent().unwrap().join("synthetic-moved-work-catalog"),
            )
            .unwrap();
            drop(create_private_directory(&path).unwrap());
        }
        reacquire_for_observation(&mut trees, &mut journal, &mut lease, None, end).unwrap();
        let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
        assert!(observe_trees(&mut trees, &mut journal, &mut claim, end).is_err());
        assert!(trees[0].observed.is_some());
        assert!(journal.check(end).is_err());
        println!("r6-work-special-refusal case={case} (ADS/hardlink/artifact-directory); exact original custody retained, partial native observation pending; no stream deletion/link removal or unknown object overwrite");
        drop(claim);
        lease.take().unwrap().release(end).unwrap();
        drop(journal);
        drop(trees);
        f.retain();
    }
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
