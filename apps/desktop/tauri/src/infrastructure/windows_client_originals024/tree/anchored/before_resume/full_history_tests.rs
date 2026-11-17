use super::*;
struct HistoryGuard<'a, 'b> {
    original: &'a mut Guard<'b>,
    read_end: Instant,
    checked: u32,
}
impl AfterReturnGuard for HistoryGuard<'_, '_> {
    fn check(
        &mut self,
        journal: &mut Journal,
        released: &mut ReleasedLease,
        end: Instant,
    ) -> io::Result<()> {
        if end != self.read_end {
            return Err(invalid());
        }
        check(end)?;
        self.original
            .native
            .0
            .read_before_resume_quiescent_history(self.original.proof, end)?;
        released.readback_history(end)?;
        if released.facts().epoch != self.original.epoch
            || released.facts().claim_key != self.original.key
            || journal.component_failed_canonical_for_test(end)? != self.original.canonical
            || self.original.storage.read_history_data(end)?.last() != Some(&self.original.raw)
        {
            return Err(invalid());
        }
        self.checked += 1;
        Ok(())
    }
    fn record_verified(&mut self, _: u32, _: u32, _: Instant) -> io::Result<()> {
        panic!("historical native read must never write another completion record")
    }
}
#[test]
fn wl2_r7_full_history_native_original_trees_job_canonical_attachment_and_old_cutoff() {
    for mode in [
        "valid-after-expiry",
        "tree-member",
        "wrong-original",
        "expired-reader",
    ] {
        let scene = Scene::new();
        let old = end();
        let mut initial = NativeLease::acquire_synthetic(&scene.paths, old).unwrap();
        let mut claim = initial.anchored_claim(&scene.root, old).unwrap();
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
        initial.release(old).unwrap();
        journal.poison_segment();
        for tree in &mut trees {
            tree.poisoned = true;
        }
        let (mut native, outputs) = launcher(&scene);
        let original_end = end();
        let mut lease = NativeLease::acquire_synthetic(&scene.paths, original_end).unwrap();
        let mut claim = lease.anchored_claim(&scene.root, original_end).unwrap();
        let (storage, last, raw, canonical, proof) =
            setup_data(&mut journal, &mut claim, &scene, original_end, &mut native);
        let mut guard = Guard {
            native: &mut native,
            proof: &proof,
            root: scene.root.clone(),
            end: original_end,
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
        core.component_return_before_resume_for_test(&mut guard, &mut claim, original_end)
            .unwrap();
        drop(claim);
        let mut released = None;
        lease.release_retained(&mut released, original_end).unwrap();
        let mut probe = ReleaseProbe::default();
        let receipt = core
            .component_finish_before_resume_release_for_test(
                &mut guard,
                released.as_mut().unwrap(),
                &mut probe,
                original_end,
            )
            .unwrap();
        let expected = receipt.data(original_end).unwrap();
        let records = guard.storage.read_data(original_end).unwrap();
        if mode == "valid-after-expiry" {
            while Instant::now() < original_end {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        if mode == "tree-member" {
            std::fs::write(
                scene.root.join("foreign-history.json"),
                b"preserve externalmember",
            )
            .unwrap();
        }
        let read_end = if mode == "expired-reader" {
            Instant::now()
        } else {
            Instant::now() + Duration::from_secs(5)
        };
        let original = if mode == "wrong-original" {
            original_end + Duration::from_nanos(1)
        } else {
            original_end
        };
        let mut scope = HistoryGuard {
            original: &mut guard,
            read_end,
            checked: 0,
        };
        let result = core.component_read_abort_history_for_test(
            &mut scope,
            released.as_mut().unwrap(),
            original,
            read_end,
        );
        if mode == "valid-after-expiry" {
            assert_eq!(result.unwrap(), expected);
            assert!(scope.checked >= 2);
            assert_eq!(receipt.history_data(read_end).unwrap(), expected);
            assert!(receipt.data(original_end).is_err());
            assert!(scope
                .original
                .native
                .0
                .verify_before_resume_quiescent(scope.original.proof)
                .is_err());
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
                .component_read_abort_history_for_test(
                    &mut scope,
                    released.as_mut().unwrap(),
                    original_end,
                    read_end
                )
                .is_err());
        }
        drop(scope);
        let fresh_read = Instant::now() + Duration::from_secs(5);
        assert_eq!(
            guard.storage.read_history_data(fresh_read).unwrap(),
            records
        );
        assert_eq!(probe.component_held_ranges_for_test(), vec![0; 5]);
        competing_claim(&scene.paths[0], false);
        assert!(!scene
            .fixture
            .as_ref()
            .unwrap()
            .path()
            .join("must-not-execute")
            .exists());
        println!("r7 fullhistory actualnative {mode}: same originalJob242/twoTrees/all8/bytes/members/canonical/attachment/releasedobjects, callerStatus5 only, no newrecord/range or oldmutationauthority; productOwner/officialSource acceptance false");
        drop(guard);
        drop(proof);
        drop(receipt);
        drop(native);
        drop(outputs);
        drop(core);
        drop(probe);
        drop(released);
        drop(lease);
        drop(scene);
    }
}
