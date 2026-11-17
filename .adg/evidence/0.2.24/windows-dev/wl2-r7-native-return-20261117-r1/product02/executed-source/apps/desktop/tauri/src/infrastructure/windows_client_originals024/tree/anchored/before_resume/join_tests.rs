use super::*;
use crate::infrastructure::windows_client_originals024::{
    binding::Binding,
    v3::{Journal, StoragePlan},
};
use native::RetainedPeer;
#[path = "quiesce_tests.rs"]
mod quiesce_tests;
#[path = "return_tests.rs"]
mod return_tests;
fn pair(
    scene: &Scene,
    claim: &mut AnchoredClaim<'_>,
    end: Instant,
) -> ([AnchoredTree; 2], Journal) {
    let grok = scene.root.with_file_name(".grok");
    drop(create_private_directory(&grok).unwrap());
    drop(
        PrivateRecord::create(&grok.join("config.json"), b"synthetic original grok", 1024).unwrap(),
    );
    let custody = scene.root.with_file_name("grok-custody");
    drop(create_private_directory(&custody).unwrap());
    let trees = [
        scene.tree(claim, end),
        AnchoredTree::capture(&grok, &custody, claim, &scene.root, end).unwrap(),
    ];
    let base = scene.fixture.as_ref().unwrap().path();
    let pins = [
        PinnedDirectory::open_private(scene.root.parent().unwrap()).unwrap(),
        create_private_directory(&base.join("D")).unwrap(),
        create_private_directory(&base.join("H")).unwrap(),
        create_private_directory(&base.join("I")).unwrap(),
    ];
    let plan = StoragePlan::from_native_data(
        trees.iter().map(AnchoredTree::plan).collect(),
        pins[0].path(),
        pins[0].identity(),
    )
    .unwrap();
    let directory = create_private_directory(&pins[1].path().join("session")).unwrap();
    let peer = RetainedPeer::current().unwrap();
    let binding = Binding::from_native(
        &peer,
        &pins,
        hash::sha256_hex(b"synthetic source DATA, no original Owner"),
        hash::sha256_hex(b"synthetic current DATA"),
    )
    .unwrap();
    let journal = Journal::create(
        directory,
        pins,
        peer,
        binding,
        plan,
        claim.epoch(),
        &claim.key(),
        &uuid::Uuid::new_v4().to_string(),
        end,
    )
    .unwrap();
    (trees, journal)
}
fn view(
    trees: &mut [AnchoredTree; 2],
    claim: &mut AnchoredClaim<'_>,
    end: Instant,
) -> [serde_json::Value; 2] {
    [
        trees[0].read_before_resume_prefix(claim, end).unwrap(),
        trees[1].read_before_resume_prefix(claim, end).unwrap(),
    ]
}
#[test]
fn wl2_r7_prefix_join_native_actual_initial_batch_interruptions_match_retained_two_tree_afterimages(
) {
    for fault in [0, 100, 101, 112, 140, 141, 152, 255] {
        let scene = Scene::new();
        let first = end();
        let mut lease = NativeLease::acquire_synthetic(&scene.paths, first).unwrap();
        let mut claim = lease.anchored_claim(&scene.root, first).unwrap();
        let (mut trees, mut journal) = pair(&scene, &mut claim, first);
        if fault != 0 {
            if fault != 255 {
                journal.inject_driver_fault(fault);
            }
            let result = journal.prepare_initial_batches(
                super::super::super::batches::initial_batch_witness(&mut trees, &mut claim, first)
                    .unwrap(),
                &mut claim,
                first,
            );
            assert_eq!(result.is_err(), fault != 255, "actual interruption{fault}");
        }
        drop(claim);
        lease.release(first).unwrap();
        journal.poison_segment();
        for t in &mut trees {
            t.poisoned = true;
        }
        let fresh = end();
        let mut lease = NativeLease::acquire_synthetic(&scene.paths, fresh).unwrap();
        let mut claim = lease.anchored_claim(&scene.root, fresh).unwrap();
        let before = view(&mut trees, &mut claim, fresh);
        let old = journal.native_progress_for_test();
        let data = journal
            .component_read_before_resume_trees_for_test(
                &before,
                claim.epoch(),
                &claim.key(),
                fresh,
            )
            .unwrap();
        assert_eq!(data["runtimeAuthority"], false);
        assert_eq!(data["poisoned"], true);
        assert_eq!(old, journal.native_progress_for_test());
        assert_eq!(before, view(&mut trees, &mut claim, fresh));
        assert!(trees.iter().all(|t| t.poisoned));
        assert!(journal.check(fresh).is_err());
        println!("r7 actual two-tree/canonical join interruption{fault}: retained native batch/actual original+work IDs and all8/complete members match, no intent completion written or old poison cleared");
        drop(journal);
        drop(trees);
        drop(claim);
        lease.release(fresh).unwrap();
        drop(scene);
    }
}
#[test]
fn wl2_r7_prefix_join_native_changed_plan_root_claim_order_state_and_work_identity_refuse() {
    let scene = Scene::new();
    let end = end();
    let mut lease = NativeLease::acquire_synthetic(&scene.paths, end).unwrap();
    let mut claim = lease.anchored_claim(&scene.root, end).unwrap();
    let (mut trees, mut journal) = pair(&scene, &mut claim, end);
    journal
        .prepare_initial_batches(
            super::super::super::batches::initial_batch_witness(&mut trees, &mut claim, end)
                .unwrap(),
            &mut claim,
            end,
        )
        .unwrap();
    journal.poison_segment();
    for t in &mut trees {
        t.poisoned = true;
    }
    let actual = view(&mut trees, &mut claim, end);
    for change in [
        "plan",
        "root-order",
        "claim",
        "node-order",
        "state",
        "work-id",
        "work-security",
        "missing-member",
        "original-location",
    ] {
        let mut bad = actual.clone();
        let file = bad[0]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .position(|n| !n["work"].is_null())
            .unwrap();
        match change {
            "plan" => bad[0]["plan"]["custodyId"] = "999".into(),
            "root-order" => bad.swap(0, 1),
            "claim" => bad[0]["claimKey"] = "f".repeat(64).into(),
            "node-order" => bad[0]["nodes"].as_array_mut().unwrap().swap(0, 1),
            "state" => bad[0]["nodes"][file]["state"] = "Moved".into(),
            "work-id" => bad[0]["nodes"][file]["work"]["fileId"] = "999".into(),
            "work-security" => bad[0]["nodes"][file]["work"]["security"] = serde_json::Value::Null,
            "missing-member" => {
                bad[0]["nodes"].as_array_mut().unwrap().pop();
            }
            _ => bad[0]["nodes"][file]["originalLocation"] = "C:/invalid/not-original".into(),
        }
        assert!(
            journal
                .component_read_before_resume_trees_for_test(&bad, claim.epoch(), &claim.key(), end)
                .is_err(),
            "{change}"
        );
    }
    journal
        .component_read_before_resume_trees_for_test(&actual, claim.epoch(), &claim.key(), end)
        .unwrap();
    assert!(journal.check(end).is_err());
    drop(journal);
    drop(trees);
    drop(claim);
    lease.release(end).unwrap();
    drop(scene);
}
