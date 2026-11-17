use super::*;
use native::{LaunchSpec, OwnedRuntime, ProcessState};
use std::collections::BTreeMap;
pub(super) struct OriginalSynthetic(pub(super) OwnedRuntime);
impl Drop for OriginalSynthetic {
    fn drop(&mut self) {
        if !self
            .0
            .root_state()
            .is_ok_and(|s| matches!(s, ProcessState::Exited { .. }))
        {
            eprintln!("separate own synthetic launcher failure cleanup; not normal Stop or recovery success");
            self.0
                .force_owned_cleanup(Instant::now() + Duration::from_secs(3))
                .unwrap();
        }
    }
}
pub(super) fn launcher(scene: &Scene) -> (OriginalSynthetic, [File; 2]) {
    let base = scene.fixture.as_ref().unwrap().path();
    let source=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../test/windows024-native/target/r7-query-proof/x86_64-pc-windows-msvc/debug/r7-abort-fixture.exe");
    let source_pin = OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(&source)
        .unwrap();
    assert_eq!(
        hash::sha256_file(&source).unwrap(),
        "4699667b79d3aebd6e1e90c8b063f80a95b3f32c5c4a69ec49bd4c70217e4a38",
        "fixed self-owned abort fixture only, never an official executable"
    );
    let exe = base.join("never-resumed-fixture.exe");
    std::fs::copy(&source, &exe).unwrap();
    drop(source_pin);
    assert_eq!(
        hash::sha256_file(&exe).unwrap(),
        "4699667b79d3aebd6e1e90c8b063f80a95b3f32c5c4a69ec49bd4c70217e4a38"
    );
    let outputs = [
        File::create(base.join("launcher-out.log")).unwrap(),
        File::create(base.join("launcher-err.log")).unwrap(),
    ];
    let args = [base.join("must-not-execute").into_os_string()];
    let env = BTreeMap::new();
    let native = OwnedRuntime::create_suspended(LaunchSpec {
        executable: &exe,
        arguments: &args,
        working_directory: base,
        user_profile: scene.root.parent().unwrap(),
        opencodex_home: &base.join("H"),
        environment: &env,
        stdout: &outputs[0],
        stderr: &outputs[1],
    })
    .unwrap();
    (OriginalSynthetic(native), outputs)
}
#[test]
fn wl2_r7_quiesce_join_native_same_original_claim_and_two_tree_canonical_before_after_forced242() {
    for fault in [101, 141, 255] {
        let scene = Scene::new();
        let old = end();
        let mut lease = NativeLease::acquire_synthetic(&scene.paths, old).unwrap();
        let mut claim = lease.anchored_claim(&scene.root, old).unwrap();
        let (mut trees, mut journal) = pair(&scene, &mut claim, old);
        if fault != 255 {
            journal.inject_driver_fault(fault);
        }
        let initialized = journal.prepare_initial_batches(
            super::super::super::super::batches::initial_batch_witness(&mut trees, &mut claim, old)
                .unwrap(),
            &mut claim,
            old,
        );
        assert_eq!(initialized.is_err(), fault != 255);
        drop(claim);
        lease.release(old).unwrap();
        journal.poison_segment();
        for tree in &mut trees {
            tree.poisoned = true;
        }
        let (mut native, outputs) = launcher(&scene);
        let start = Instant::now();
        let fresh = start + Duration::from_secs(10);
        let prepared = native
            .0
            .prepare_before_first_resume_abort(start, fresh)
            .unwrap();
        assert_eq!(prepared.deadline(), fresh);
        let mut lease = NativeLease::acquire_synthetic(&scene.paths, fresh).unwrap();
        let mut claim = lease.anchored_claim(&scene.root, fresh).unwrap();
        let epoch = claim.epoch().to_string();
        let key = claim.key();
        let before = view(&mut trees, &mut claim, fresh);
        let canonical = journal
            .component_read_before_resume_trees_for_test(&before, &epoch, &key, fresh)
            .unwrap();
        claim.check(&scene.root, fresh).unwrap();
        let quiescent = native.0.quiesce_before_first_resume(prepared).unwrap();
        assert_eq!(quiescent.forced_raw_exit(), 242);
        native.0.verify_before_resume_quiescent(&quiescent).unwrap();
        assert_eq!(
            native.0.root_state().unwrap(),
            ProcessState::Exited { code: 242 }
        );
        assert!(!native.0.creator_has_resumed());
        assert!(native.0.resume().is_err());
        assert!(!scene
            .fixture
            .as_ref()
            .unwrap()
            .path()
            .join("must-not-execute")
            .exists());
        claim.check(&scene.root, fresh).unwrap();
        assert_eq!(claim.epoch(), epoch);
        assert_eq!(claim.key(), key);
        let after = view(&mut trees, &mut claim, fresh);
        assert_eq!(after, before);
        assert_eq!(
            journal
                .component_read_before_resume_trees_for_test(&after, &epoch, &key, fresh)
                .unwrap(),
            canonical
        );
        native.0.verify_before_resume_quiescent(&quiescent).unwrap();
        assert!(journal.check(fresh).is_err());
        assert!(trees.iter().all(|t| t.poisoned));
        println!("r7 actual original synthetic launcher quiescence with same actual claim and two-tree/canonical prefix fault{fault}: raw242, never Resume, full before/after exact, no original-return/Stop/Closed or actual product Owner claim");
        drop(quiescent);
        drop(native);
        drop(outputs);
        drop(journal);
        drop(trees);
        drop(claim);
        lease.release(fresh).unwrap();
        drop(scene);
    }
}
