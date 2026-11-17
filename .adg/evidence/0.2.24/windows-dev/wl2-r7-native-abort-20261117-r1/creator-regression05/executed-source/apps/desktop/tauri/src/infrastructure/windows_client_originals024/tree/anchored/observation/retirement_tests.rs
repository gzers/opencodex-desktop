use super::super::super::tests::driver::setup_artifacts;
use super::super::super::{
    segments::release_trees,
    tests::{bun_claim, compete, driver::setup},
};
use super::*;
use crate::infrastructure::windows_client_originals024::{
    coordination::NativeLease,
    v3::{FileAction, Journal},
};
fn current(
    trees: &mut [AnchoredTree; 2],
    journal: &mut Journal,
    lease: &mut Option<NativeLease>,
    end: Instant,
) {
    current_variant(trees, journal, lease, end, false)
}
fn current_variant(
    trees: &mut [AnchoredTree; 2],
    journal: &mut Journal,
    lease: &mut Option<NativeLease>,
    end: Instant,
    atomic: bool,
) {
    current_variant_with_absent(trees, journal, lease, end, atomic, false)
}
fn current_variant_with_absent(
    trees: &mut [AnchoredTree; 2],
    journal: &mut Journal,
    lease: &mut Option<NativeLease>,
    end: Instant,
    atomic: bool,
    absent: bool,
) {
    let root = trees[0].root.clone();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
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
    drop(claim);
    release_trees(trees, journal, lease, end).unwrap();
    // Exact3 originally absent known artifacts, plus an atomic replacement of
    // nonartifact config with IDENTICAL bytes/security. Scanner must still veto
    // the changed nonartifact generation, so that variant is tested separately.
    for (name, bytes) in [
        ("opencodex-catalog.json", b"{\"models\":[]}".as_slice()),
        ("models_cache.json", b"synthetic observed cache".as_slice()),
        (
            "opencodex-catalog-audit.jsonl",
            b"synthetic observed audit\n".as_slice(),
        ),
    ] {
        let path = root.join(name);
        if path.exists() {
            std::fs::write(&path, bytes).unwrap();
        } else {
            drop(PrivateRecord::create(&path, bytes, 1024).unwrap());
        }
    }
    if atomic {
        let path = root.join("synthetic-catalog-atomic.tmp");
        drop(
            PrivateRecord::create(
                &path,
                b"{\"models\":[{\"slug\":\"synthetic-observed\"}]}",
                1024,
            )
            .unwrap(),
        );
        std::fs::rename(&path, root.join("opencodex-catalog.json")).unwrap();
    }
    if absent {
        // Exact newly-created synthetic WORK only. Original artifact remains
        // under its original custody handle; historical denied cleanup untouched.
        std::fs::remove_file(root.join("models_cache.json")).unwrap();
    }
    reacquire_for_observation(trees, journal, lease, None, end).unwrap();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    observe_trees(trees, journal, &mut claim, end).unwrap();
    super::super::snapshots::link_observation(trees, journal, &mut claim, false, end).unwrap();
}
#[test]
fn wl2_r6_recovery_native_complete_frozen_plan_validates_and_absent_current_is_explicit() {
    let (mut f, _data, _paths, held, mut trees, mut journal, end) = setup_artifacts();
    let mut lease = Some(held);
    current_variant_with_absent(&mut trees, &mut journal, &mut lease, end, true, true);
    let root = trees[0].root.clone();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    let plans = [
        trees[0].capture_retirement(&mut claim, end).unwrap(),
        trees[1].capture_retirement(&mut claim, end).unwrap(),
    ];
    assert_eq!(plans[0]["absentCurrent"].as_array().unwrap().len(), 1);
    assert_eq!(
        plans[0]["absentCurrent"][0]["relative"],
        "models_cache.json"
    );
    journal
        .inspect_recovery_observation_for_test(plans.clone(), &mut claim, end)
        .unwrap();
    let mut missing = plans.clone();
    missing[0]["absentCurrent"] = serde_json::json!([]);
    assert!(journal
        .inspect_recovery_observation_for_test(missing, &mut claim, end)
        .is_err());
    let mut absent = plans.clone();
    absent[0]["absentCurrent"][0]["originalFileId"] = "999".into();
    assert!(journal
        .inspect_recovery_observation_for_test(absent, &mut claim, end)
        .is_err());
    let mut target = plans.clone();
    target[0]["retirements"][0]["targetParentId"] = "999".into();
    assert!(journal
        .inspect_recovery_observation_for_test(target, &mut claim, end)
        .is_err());
    println!("r6-recovery-native-plans={}", serde_json::json!(plans));
    println!("r6 recovery complete DATA validated against actual retained both-tree frozen frames and immutable native archives, ordinary work/original/current replacement distinct; absent current original artifact explicit and malformed absence/target refused; Observation component only, not actual Stopped or content-policy authority; no native recovery mutation");
    drop(claim);
    lease.take().unwrap().release(end).unwrap();
    drop(journal);
    drop(trees);
    f.retain();
}
#[test]
fn wl2_r6_retirement_existing_artifacts_changed_and_atomically_replaced_work_restore_original_objects(
) {
    let (mut f, _data, _paths, held, mut trees, mut journal, end) = setup_artifacts();
    let mut lease = Some(held);
    current_variant(&mut trees, &mut journal, &mut lease, end, true);
    let root = trees[0].root.clone();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    let original = trees.each_ref().map(|t| t.plan());
    for tree in &mut trees {
        tree.capture_retirement(&mut claim, end).unwrap();
    }
    let replacement = trees[0]
        .retirement
        .as_ref()
        .unwrap()
        .frame
        .nodes
        .iter()
        .find(|n| n.entry.relative == "opencodex-catalog.json")
        .unwrap()
        .entry
        .id;
    let original_node = trees[0]
        .nodes
        .iter()
        .find(|n| n.original.relative == "opencodex-catalog.json")
        .unwrap();
    assert_ne!(replacement, original_node.original.id);
    assert_ne!(replacement, original_node.work.as_ref().unwrap().1.id);
    assert!(trees[0].retirement.as_ref().unwrap().plan["retirements"]
        .as_array()
        .unwrap()
        .iter()
        .all(|r| r["originalAbsent"] == false));
    println!(
        "r6-native-retirement-existing-plan={}",
        trees[0].retirement.as_ref().unwrap().plan
    );
    for tree in &mut trees {
        for index in 0..tree.retirement.as_ref().unwrap().items.len() {
            tree.retire_current_file(index, &mut claim, end).unwrap();
        }
        let originals = tree
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(i, n)| (n.kind == Kind::File).then_some(i))
            .collect::<Vec<_>>();
        for index in originals {
            tree.restore_original_file(index, &mut claim, end).unwrap();
        }
        tree.verify_retirement_restored(&mut claim, end).unwrap();
    }
    assert_eq!(trees.each_ref().map(|t| t.plan()), original);
    assert_eq!(
        std::fs::read(root.join("opencodex-catalog.json")).unwrap(),
        b"{\"models\":[]}"
    );
    drop(claim);
    lease.take().unwrap().release(end).unwrap();
    for tree in &mut trees {
        tree.readback_retirement_after_release(end).unwrap();
    }
    println!("r6 native component existing artifact originals restored by exact original IDs/all8/bytes, modified and atomically replaced current generations retired separately; retained initial work observer/original/current3 identities never conflated, stable released-lock readback passed, no content-policy/Source/Stop/Closed grant");
    drop(journal);
    drop(trees);
    f.retain();
}
#[test]
fn wl2_r6_retirement_current_files_and_original_absent_artifacts_retire_same_handles_full_tree_restore(
) {
    let (mut f, _data, paths, held, mut trees, mut journal, end) = setup();
    let mut lease = Some(held);
    current(&mut trees, &mut journal, &mut lease, end);
    let originals = trees.each_ref().map(|t| t.plan());
    let root = trees[0].root.clone();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    let plans = [
        trees[0].capture_retirement(&mut claim, end).unwrap(),
        trees[1].capture_retirement(&mut claim, end).unwrap(),
    ];
    assert_eq!(plans[0]["retirements"].as_array().unwrap().len(), 5);
    assert_eq!(trees[0].retirement.as_ref().unwrap().plan, plans[0]);
    assert_eq!(plans[1]["retirements"].as_array().unwrap().len(), 1);
    assert_eq!(
        plans[0]["retirements"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["originalAbsent"] == true)
            .count(),
        3
    );
    println!("r6-native-retirement-plans={}", serde_json::json!(plans));
    bun_claim(&paths[0], end, true);
    for tree in &mut trees {
        for index in 0..tree.retirement.as_ref().unwrap().items.len() {
            tree.retire_current_file(index, &mut claim, end).unwrap();
            compete(&paths[0], true);
        }
    }
    for tree in &mut trees {
        let originals = tree
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(i, n)| (n.kind == Kind::File).then_some(i))
            .collect::<Vec<_>>();
        for index in originals {
            tree.restore_original_file(index, &mut claim, end).unwrap();
            compete(&paths[0], true);
        }
        tree.verify_retirement_restored(&mut claim, end).unwrap();
    }
    assert_eq!(trees.each_ref().map(|t| t.plan()), originals);
    for name in [
        "opencodex-catalog.json",
        "models_cache.json",
        "opencodex-catalog-audit.jsonl",
    ] {
        assert!(!root.join(name).exists());
    }
    assert_eq!(
        std::fs::read(root.join("config.toml")).unwrap(),
        b"synthetic original config"
    );
    assert_eq!(
        std::fs::read(trees[1].root.join("settings.json")).unwrap(),
        b"synthetic Grok original"
    );
    bun_claim(&paths[0], end, true);
    drop(claim);
    lease.take().unwrap().release(end).unwrap();
    for tree in &mut trees {
        tree.readback_retirement_after_release(end).unwrap();
    }
    bun_claim(&paths[0], end, false);
    println!("r6-native-retirement component6 current work files incl3 originally-absent artifact leaves retired same-volume via original current handles;3 original files restored by exact original handles; all root/nested/empty dir+two stable IDs/all8/children unchanged. Current cache/audit bytes deliberately not policy-admitted. No canonical per-retirement journal, actual Source/Stop/restore authority/Closed minted; component mutation methods private.");
    drop(journal);
    drop(trees);
    f.retain();
}
#[test]
fn wl2_r6_retirement_early_restore_and_foreign_original_destination_never_overwrite() {
    for early in [false, true] {
        let (mut f, _data, _paths, held, mut trees, mut journal, end) = setup();
        let mut lease = Some(held);
        current(&mut trees, &mut journal, &mut lease, end);
        let root = trees[0].root.clone();
        let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
        trees[0].capture_retirement(&mut claim, end).unwrap();
        let index = trees[0]
            .nodes
            .iter()
            .position(|n| n.original.relative == "config.toml")
            .unwrap();
        let original_location = trees[0].original_location(index);
        let target = root.join("config.toml");
        if !early {
            for item in 0..trees[0].retirement.as_ref().unwrap().items.len() {
                trees[0].retire_current_file(item, &mut claim, end).unwrap();
            }
            drop(
                PrivateRecord::create(&target, b"synthetic foreign restore destination", 1024)
                    .unwrap(),
            );
        }
        assert!(trees[0]
            .restore_original_file(index, &mut claim, end)
            .is_err());
        assert!(trees[0].poisoned);
        assert_eq!(trees[0].nodes[index].state, State::Working);
        assert!(original_location.exists());
        assert_eq!(
            std::fs::read(&original_location).unwrap(),
            b"synthetic original config"
        );
        if !early {
            assert_eq!(
                std::fs::read(&target).unwrap(),
                b"synthetic foreign restore destination"
            );
        }
        println!("r6-retirement-original-restore-refusal early={early}, original native custody and foreign/current destination preserved, no overwrite/target deletion or continuation");
        drop(claim);
        lease.take().unwrap().release(end).unwrap();
        drop(journal);
        drop(trees);
        f.retain();
    }
}
#[test]
fn wl2_r6_retirement_reentry_poison_without_new_native_step() {
    let (mut f, _data, _paths, held, mut trees, mut journal, end) = setup();
    let mut lease = Some(held);
    current(&mut trees, &mut journal, &mut lease, end);
    let root = trees[0].root.clone();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    trees[0].capture_retirement(&mut claim, end).unwrap();
    let shorter = end - std::time::Duration::from_millis(1);
    trees[0]
        .verify_retirement_prefix(&mut claim, shorter)
        .unwrap();
    trees[0].retire_current_file(0, &mut claim, end).unwrap();
    let target = trees[0].retirement.as_ref().unwrap().items[0]
        .target
        .clone();
    assert!(target.exists());
    assert!(trees[0].retire_current_file(0, &mut claim, end).is_err());
    assert!(trees[0].poisoned);
    assert!(target.exists());
    assert!(trees[0].verify_retirement_prefix(&mut claim, end).is_err());
    println!("r6-retirement repeated raw native step cannot replay or recapture current generation, remains poisoned with first retired target intact; original active claim unchanged and no deadline grant");
    drop(claim);
    lease.take().unwrap().release(end).unwrap();
    drop(journal);
    drop(trees);
    f.retain();
}
#[test]
fn wl2_r6_retirement_every_retire_and_restore_prefix_keeps_exact_native_objects() {
    for prefix in 0..=9 {
        let (mut f, _data, _paths, held, mut trees, mut journal, end) = setup();
        let mut lease = Some(held);
        current(&mut trees, &mut journal, &mut lease, end);
        let root = trees[0].root.clone();
        let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
        for tree in &mut trees {
            tree.capture_retirement(&mut claim, end).unwrap();
        }
        let mut completed = 0;
        for tree in &mut trees {
            for index in 0..tree.retirement.as_ref().unwrap().items.len() {
                if completed < prefix {
                    tree.retire_current_file(index, &mut claim, end).unwrap();
                }
                completed += 1;
            }
        }
        for tree in &mut trees {
            let files = tree
                .nodes
                .iter()
                .enumerate()
                .filter_map(|(i, n)| (n.kind == Kind::File).then_some(i))
                .collect::<Vec<_>>();
            for index in files {
                if completed < prefix {
                    tree.restore_original_file(index, &mut claim, end).unwrap();
                }
                completed += 1;
            }
            tree.verify_retirement_prefix(&mut claim, end).unwrap();
        }
        let retired = trees
            .iter()
            .map(|t| {
                t.retirement
                    .as_ref()
                    .unwrap()
                    .items
                    .iter()
                    .filter(|i| i.retired)
                    .count()
            })
            .sum::<usize>();
        let restored = trees
            .iter()
            .flat_map(|t| &t.nodes)
            .filter(|n| n.state == State::Restored)
            .count();
        assert_eq!(retired, prefix.min(6));
        assert_eq!(restored, prefix.saturating_sub(6));
        println!("r6-retirement-native-prefix={prefix} retiredCurrent={retired} restoredOriginal={restored}; all exact paths/IDs/bytes/all8/children/custody verified, no deletion/rollforward/recovery authority");
        drop(claim);
        lease.take().unwrap().release(end).unwrap();
        drop(journal);
        drop(trees);
        f.retain();
    }
}
#[test]
fn wl2_r6_retirement_foreign_target_or_empty_directory_poison_without_overwrite() {
    for target_conflict in [false, true] {
        let (mut f, _data, _paths, held, mut trees, mut journal, end) = setup();
        let mut lease = Some(held);
        current(&mut trees, &mut journal, &mut lease, end);
        let root = trees[0].root.clone();
        let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
        trees[0].capture_retirement(&mut claim, end).unwrap();
        let target = if target_conflict {
            trees[0].retirement.as_ref().unwrap().items[0]
                .target
                .clone()
        } else {
            root.join("foreign-empty")
        };
        if target_conflict {
            drop(PrivateRecord::create(&target, b"synthetic unknown target", 1024).unwrap());
        } else {
            drop(create_private_directory(&target).unwrap());
        }
        assert!(trees[0].retire_current_file(0, &mut claim, end).is_err());
        assert!(trees[0].poisoned);
        assert!(!trees[0].retirement.as_ref().unwrap().items[0].retired);
        assert!(target.exists());
        assert!(trees[0].retire_current_file(0, &mut claim, end).is_err());
        println!("r6-retirement-native-conflict targetConflict={target_conflict} exact foreign preserved, no native move before admission, poisoned tree retains all current/original objects");
        drop(claim);
        lease.take().unwrap().release(end).unwrap();
        drop(journal);
        drop(trees);
        f.retain();
    }
}
