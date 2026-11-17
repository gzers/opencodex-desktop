use super::*;
use crate::infrastructure::windows_client_originals024::{
    coordination::{ReleaseProbe, ReleasedLease},
    tree::AfterReturnGuard,
};
use windows_sys::Win32::{
    Storage::FileSystem::{
        LockFileEx, UnlockFileEx, LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY,
    },
    System::IO::OVERLAPPED,
};
impl AfterReturnGuard for Guard<'_> {
    fn check(
        &mut self,
        journal: &mut Journal,
        released: &mut ReleasedLease,
        end: Instant,
    ) -> io::Result<()> {
        check(end)?;
        if end != self.end
            || released.facts().epoch != self.epoch
            || released.facts().claim_key != self.key
        {
            return Err(invalid());
        }
        self.native.0.verify_before_resume_quiescent(self.proof)?;
        released.check(end)?;
        if journal.component_failed_canonical_for_test(end)? != self.canonical
            || self.storage.read_data(end)?.last() != Some(&self.raw)
        {
            return Err(invalid());
        }
        Ok(())
    }
    fn record_verified(&mut self, nodes: u32, coordination: u32, end: Instant) -> io::Result<()> {
        if self.fail == "final-record" {
            return Err(io::Error::other("injected final record write refusal"));
        }
        let next = self.last.next_data(
            &self.raw,
            Event::AbortRestoreVerified {
                original_nodes: nodes,
                coordination_objects: coordination,
                normal_stop: false,
                closed: false,
                public_start_open: false,
            },
        )?;
        self.raw = next.encode_data()?;
        self.last = next;
        self.storage.append_data(&self.raw, end)?;
        if self.fail == "after-final-member" {
            std::fs::write(
                self.root.join("foreign-after-final.json"),
                b"do not ignore external member",
            )?;
        }
        Ok(())
    }
}
struct ForeignRange {
    file: File,
    overlap: OVERLAPPED,
}
impl ForeignRange {
    fn owner(path: &Path) -> Self {
        let file = OpenOptions::new()
            .read(true)
            .share_mode(3)
            .open(path)
            .unwrap();
        let mut overlap: OVERLAPPED = unsafe { std::mem::zeroed() };
        overlap.Anonymous.Anonymous.Offset = 0x40000000;
        assert_ne!(
            unsafe {
                LockFileEx(
                    file.as_raw_handle(),
                    LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                    0,
                    512,
                    0,
                    &mut overlap,
                )
            },
            0
        );
        Self { file, overlap }
    }
    fn release(&mut self) {
        assert_ne!(
            unsafe { UnlockFileEx(self.file.as_raw_handle(), 0, 512, 0, &mut self.overlap) },
            0
        );
    }
}
fn competing_claim(path: &Path, blocked: bool) {
    let file = OpenOptions::new()
        .read(true)
        .share_mode(3)
        .open(path)
        .unwrap();
    let mut overlap: OVERLAPPED = unsafe { std::mem::zeroed() };
    overlap.Anonymous.Anonymous.Offset = 0x40000000;
    let raw = unsafe {
        LockFileEx(
            file.as_raw_handle(),
            LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
            0,
            512,
            0,
            &mut overlap,
        )
    };
    if blocked {
        assert_eq!(raw, 0);
        assert_eq!(io::Error::last_os_error().raw_os_error(), Some(33));
    } else {
        assert_ne!(raw, 0);
        assert_ne!(
            unsafe { UnlockFileEx(file.as_raw_handle(), 0, 512, 0, &mut overlap) },
            0
        );
    }
}
#[test]
fn wl2_r7_release_readback_native_original_return_explicit_release_five_ranges_and_final_strict_record(
) {
    for fault in [0, 101, 141, 254, 255] {
        let scene = Scene::new();
        let old = end();
        let mut lease = NativeLease::acquire_synthetic(&scene.paths, old).unwrap();
        let mut claim = lease.anchored_claim(&scene.root, old).unwrap();
        let (mut trees, mut journal) = pair(&scene, &mut claim, old);
        if fault != 0 {
            if fault != 254 && fault != 255 {
                journal.inject_driver_fault(fault);
            }
            let result = journal.prepare_initial_batches(
                super::super::super::super::super::batches::initial_batch_witness(
                    &mut trees, &mut claim, old,
                )
                .unwrap(),
                &mut claim,
                old,
            );
            assert_eq!(result.is_err(), fault != 254 && fault != 255);
            if fault == 254 {
                super::super::super::super::super::observation::open_trees(
                    &mut trees,
                    &mut journal,
                    &mut claim,
                    old,
                )
                .unwrap();
            }
        }
        drop(claim);
        lease.release(old).unwrap();
        journal.poison_segment();
        for t in &mut trees {
            t.poisoned = true;
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
            canonical: canonical.clone(),
            storage,
            last,
            raw,
            fail: "none",
            steps: 0,
        };
        let mut core = CapturedTrees::retain(trees, journal);
        core.component_return_before_resume_for_test(&mut guard, &mut claim, fresh)
            .unwrap();
        competing_claim(&scene.paths[0], true);
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
        let data = receipt.data(fresh).unwrap();
        assert_eq!(data["normalStop"], false);
        assert_eq!(data["closed"], false);
        assert_eq!(data["coordinationObjects"], 3);
        assert!(core
            .component_finish_before_resume_release_for_test(
                &mut guard,
                released.as_mut().unwrap(),
                &mut probe,
                fresh
            )
            .is_err());
        assert_eq!(
            core.component_abort_final_state_for_test(),
            (true, true, false)
        );
        assert_eq!(probe.component_held_ranges_for_test(), vec![0; 5]);
        probe.check(released.as_mut().unwrap(), fresh).unwrap();
        competing_claim(&scene.paths[0], false);
        competing_claim(
            &scene.root.join(".opencodex-native-main.owner.sqlite"),
            false,
        );
        let records = guard.storage.read_data(fresh).unwrap();
        let last: serde_json::Value = serde_json::from_slice(records.last().unwrap()).unwrap();
        assert_eq!(last["event"]["kind"], "AbortRestoreVerified");
        assert_eq!(
            core.journal()
                .component_failed_canonical_for_test(fresh)
                .unwrap(),
            canonical
        );
        assert!(core.native_prefix().1);
        println!("r7 native released prefix{fault}: original source/two-tree/readback after same original range explicit release, five actual ranges probed concurrently+released, strict final DATA recorded, native proof private; no Stop/Closed/successor/productOwner budget claim");
        drop(receipt);
        drop(guard);
        drop(proof);
        drop(native);
        drop(outputs);
        drop(core);
        drop(probe);
        drop(released);
        drop(lease);
        drop(scene);
    }
}
#[test]
fn wl2_r7_release_readback_native_busy_anchor_drift_or_final_record_failure_keep_exact_originals_required(
) {
    for failure in [
        "busy-owner",
        "foreign-member",
        "final-record",
        "after-final-member",
        "expired",
    ] {
        let scene = Scene::new();
        let old = end();
        let mut lease = NativeLease::acquire_synthetic(&scene.paths, old).unwrap();
        let mut claim = lease.anchored_claim(&scene.root, old).unwrap();
        let (mut trees, mut journal) = pair(&scene, &mut claim, old);
        journal
            .prepare_initial_batches(
                super::super::super::super::super::batches::initial_batch_witness(
                    &mut trees, &mut claim, old,
                )
                .unwrap(),
                &mut claim,
                old,
            )
            .unwrap();
        drop(claim);
        lease.release(old).unwrap();
        journal.poison_segment();
        for t in &mut trees {
            t.poisoned = true;
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
            fail: failure,
            steps: 0,
        };
        let mut core = CapturedTrees::retain(trees, journal);
        core.component_return_before_resume_for_test(&mut guard, &mut claim, fresh)
            .unwrap();
        drop(claim);
        let mut released = None;
        lease.release_retained(&mut released, fresh).unwrap();
        let mut foreign = if failure == "busy-owner" {
            Some(ForeignRange::owner(
                &scene.root.join(".opencodex-native-main.owner.sqlite"),
            ))
        } else {
            None
        };
        if failure == "foreign-member" {
            std::fs::write(
                scene.root.join("foreign.json"),
                b"preserve external conflict",
            )
            .unwrap();
        }
        let read_end = if failure == "expired" {
            Instant::now()
        } else {
            fresh
        };
        let mut probe = ReleaseProbe::default();
        assert!(
            core.component_finish_before_resume_release_for_test(
                &mut guard,
                released.as_mut().unwrap(),
                &mut probe,
                read_end
            )
            .is_err(),
            "{failure}"
        );
        assert_eq!(
            core.component_abort_final_state_for_test(),
            (true, false, true)
        );
        assert!(core
            .component_finish_before_resume_release_for_test(
                &mut guard,
                released.as_mut().unwrap(),
                &mut probe,
                fresh
            )
            .is_err());
        if failure == "busy-owner" {
            assert_eq!(probe.component_held_ranges_for_test(), vec![1, 1, 1, 1, 0]);
            competing_claim(&scene.paths[0], true);
            foreign.as_mut().unwrap().release();
        }
        if failure != "after-final-member" {
            let last: serde_json::Value =
                serde_json::from_slice(guard.storage.read_data(fresh).unwrap().last().unwrap())
                    .unwrap();
            assert_ne!(last["event"]["kind"], "AbortRestoreVerified");
        }
        println!("r7 native released failure{failure}: no final native grant, failed original driver/actual remaining probe ranges and immutable canonical retained; no retry or overwrite");
        // Isolated own negative fixture disposition only; production worker
        // retains these ranges on failure and has no automatic cleanup path.
        probe.component_cleanup_for_test().unwrap();
        drop(foreign);
        drop(guard);
        drop(proof);
        drop(native);
        drop(outputs);
        drop(core);
        drop(probe);
        drop(released);
        drop(lease);
        drop(scene);
    }
}
