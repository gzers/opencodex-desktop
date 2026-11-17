use super::*;
#[test]
fn wl2_r7_return_validation_native_full_trees_canonical_original_release_and_cutoff() {
    for mode in [
        "valid",
        "tree-member",
        "journal-member",
        "wrong-cutoff",
        "expired",
        "foreign-released",
    ] {
        let scene = Scene::new();
        let old = end();
        let mut old_lease = NativeLease::acquire_synthetic(&scene.paths, old).unwrap();
        let mut claim = old_lease.anchored_claim(&scene.root, old).unwrap();
        let (mut trees, mut journal) = pair(&scene, &mut claim, old);
        journal
            .prepare_initial_batches(
                super::super::super::super::super::super::batches::initial_batch_witness(
                    &mut trees, &mut claim, old,
                )
                .unwrap(),
                &mut claim,
                old,
            )
            .unwrap();
        drop(claim);
        old_lease.release(old).unwrap();
        journal.poison_segment();
        for tree in &mut trees {
            tree.poisoned = true;
        }
        let (mut native, outputs) = launcher(&scene);
        let fresh = end();
        let mut lease = NativeLease::acquire_synthetic(&scene.paths, fresh).unwrap();
        let mut claim = lease.anchored_claim(&scene.root, fresh).unwrap();
        let (storage, last, raw, canonical, proof) =
            setup_data(&mut journal, &mut claim, &scene, fresh, &mut native);
        let mut guard = Guard {
            native: &mut native,
            proof: &proof,
            root: scene.root.clone(),
            end: fresh,
            epoch: claim.epoch().into(),
            key: claim.key(),
            canonical,
            storage,
            last,
            raw,
            fail: "none",
            steps: 0,
        };
        let mut core = CapturedTrees::retain(trees, journal);
        core.component_return_before_resume_for_test(&mut guard, &mut claim, fresh)
            .unwrap();
        drop(claim);
        let mut released = None;
        lease.release_retained(&mut released, fresh).unwrap();
        let mut probe = ReleaseProbe::default();
        let receipt = core
            .component_finish_before_resume_release_for_test(
                &mut guard,
                released.as_mut().unwrap(),
                &mut probe,
                fresh,
            )
            .unwrap();
        let expected = receipt.data(fresh).unwrap();
        let old_records = guard.storage.read_data(fresh).unwrap();
        let mut foreign_lease = None;
        let mut foreign_released = None;
        match mode {
            "tree-member" => std::fs::write(
                scene.root.join("foreign-returned.json"),
                b"preserve external member",
            )
            .unwrap(),
            "journal-member" => std::fs::write(
                scene
                    .fixture
                    .as_ref()
                    .unwrap()
                    .path()
                    .join("D/session/foreign-returned.json"),
                b"preserve external member",
            )
            .unwrap(),
            "foreign-released" => {
                let mut actual = NativeLease::acquire_synthetic(&scene.paths, fresh).unwrap();
                actual
                    .release_retained(&mut foreign_released, fresh)
                    .unwrap();
                foreign_lease = Some(actual);
            }
            _ => {}
        }
        let deadline = match mode {
            "wrong-cutoff" => fresh + Duration::from_nanos(1),
            "expired" => Instant::now(),
            _ => fresh,
        };
        let actual = if mode == "foreign-released" {
            foreign_released.as_mut().unwrap()
        } else {
            released.as_mut().unwrap()
        };
        let result = core.component_recheck_abort_return_for_test(&mut guard, actual, deadline);
        if mode == "valid" {
            assert_eq!(result.unwrap(), expected);
            assert_eq!(
                core.component_abort_final_state_for_test(),
                (true, true, false)
            );
        } else {
            assert!(result.is_err(), "{mode}");
            assert_eq!(
                core.component_abort_final_state_for_test(),
                (true, true, true)
            );
            assert!(core
                .component_recheck_abort_return_for_test(
                    &mut guard,
                    released.as_mut().unwrap(),
                    fresh
                )
                .is_err());
        }
        assert_eq!(
            guard.storage.read_data(fresh).unwrap(),
            old_records,
            "no extra completion record written"
        );
        assert_eq!(probe.component_held_ranges_for_test(), vec![0; 5]);
        assert!(!scene
            .fixture
            .as_ref()
            .unwrap()
            .path()
            .join("must-not-execute")
            .exists());
        println!("r7-return-validation native {mode}: complete original trees/8/bytes/membership and original released objects, same deadline; no new ranges/record/native permit or currentr2 action");
        drop(guard);
        drop(proof);
        drop(native);
        drop(outputs);
        drop(core);
        drop(probe);
        drop(released);
        drop(foreign_released);
        drop(foreign_lease);
        drop(lease);
        drop(scene);
    }
}
