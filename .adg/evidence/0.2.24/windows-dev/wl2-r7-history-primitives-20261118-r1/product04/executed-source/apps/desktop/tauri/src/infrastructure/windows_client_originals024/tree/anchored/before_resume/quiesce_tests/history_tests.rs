use super::*;
fn historical_scene() -> Scene {
    let scene = Scene::new();
    let base = scene.fixture.as_ref().unwrap().path();
    drop(create_private_directory(&base.join("H")).unwrap());
    assert!(base.is_dir() && scene.root.parent().unwrap().is_dir() && base.join("H").is_dir());
    scene
}
#[test]
fn wl2_r7_history_primitives_native_original_quiescence_after_expiry_never_renews_mutation() {
    let scene = historical_scene();
    let (mut original, outputs) = launcher(&scene);
    let start = Instant::now();
    let original_end = start + Duration::from_secs(1);
    let prepared = original
        .0
        .prepare_before_first_resume_abort(start, original_end)
        .unwrap();
    let proof = original.0.quiesce_before_first_resume(prepared).unwrap();
    assert_eq!(proof.deadline(), original_end);
    original.0.verify_before_resume_quiescent(&proof).unwrap();
    while Instant::now() < original_end {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        original
            .0
            .verify_before_resume_quiescent(&proof)
            .unwrap_err()
            .kind(),
        io::ErrorKind::TimedOut
    );
    let read_end = Instant::now() + Duration::from_secs(5);
    original
        .0
        .read_before_resume_quiescent_history(&proof, read_end)
        .unwrap();
    original
        .0
        .read_before_resume_quiescent_history(&proof, read_end)
        .unwrap();
    assert_eq!(
        original
            .0
            .read_before_resume_quiescent_history(&proof, Instant::now())
            .unwrap_err()
            .kind(),
        io::ErrorKind::TimedOut
    );
    assert_eq!(proof.deadline(), original_end);
    assert_eq!(proof.forced_raw_exit(), 242);
    assert!(original.0.resume().is_err());
    assert!(original
        .0
        .prepare_before_first_resume_abort(Instant::now(), read_end)
        .is_err());
    assert_eq!(
        original.0.root_state().unwrap(),
        ProcessState::Exited { code: 242 }
    );
    assert!(!scene
        .fixture
        .as_ref()
        .unwrap()
        .path()
        .join("must-not-execute")
        .exists());
    println!("r7 historical original nativeJob proof: originalquiescent242 timely; olddeadline expired/strictread TimedOut; freshboundedquery onlysameoriginalroot/Job total1active0; no Resume/reprepare/Stop/Closed");
    drop(proof);
    drop(original);
    drop(outputs);
    drop(scene);
}
#[test]
fn wl2_r7_history_primitives_native_foreign_actual_job_never_passes_history_read() {
    let scene = historical_scene();
    let (mut a, outputs_a) = launcher(&scene);
    // Second original uses the same fixture image/log paths while its separate
    // original creation/Job proof remains distinct. Existing pin admits reads.
    let base = scene.fixture.as_ref().unwrap().path();
    let outputs_b = [
        File::create(base.join("second-out.log")).unwrap(),
        File::create(base.join("second-err.log")).unwrap(),
    ];
    let exe = base.join("never-resumed-fixture.exe");
    let args = [base.join("must-not-execute-second").into_os_string()];
    let environment = BTreeMap::new();
    let mut b = OriginalSynthetic(
        OwnedRuntime::create_suspended(LaunchSpec {
            executable: &exe,
            arguments: &args,
            working_directory: base,
            user_profile: scene.root.parent().unwrap(),
            opencodex_home: &base.join("H"),
            environment: &environment,
            stdout: &outputs_b[0],
            stderr: &outputs_b[1],
        })
        .unwrap(),
    );
    let start = Instant::now();
    let end = start + Duration::from_secs(10);
    let prepared = a.0.prepare_before_first_resume_abort(start, end).unwrap();
    let proof = a.0.quiesce_before_first_resume(prepared).unwrap();
    assert!(b
        .0
        .read_before_resume_quiescent_history(&proof, end)
        .is_err());
    assert!(!b.0.forced_cleanup_requested());
    let prepared_b =
        b.0.prepare_before_first_resume_abort(Instant::now(), end)
            .unwrap();
    let proof_b = b.0.quiesce_before_first_resume(prepared_b).unwrap();
    assert!(b
        .0
        .read_before_resume_quiescent_history(&proof, end)
        .is_err());
    assert!(a
        .0
        .read_before_resume_quiescent_history(&proof_b, end)
        .is_err());
    a.0.read_before_resume_quiescent_history(&proof, end)
        .unwrap();
    b.0.read_before_resume_quiescent_history(&proof_b, end)
        .unwrap();
    assert!(!base.join("must-not-execute").exists());
    assert!(!base.join("must-not-execute-second").exists());
    println!("r7 historical actual2originalJob crossproof denied before/after nativeforced242; ownproofonly, no DATA factory/currentr2");
    drop(proof);
    drop(proof_b);
    drop(a);
    drop(b);
    drop(outputs_a);
    drop(outputs_b);
    drop(scene);
}
