use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::{self, File},
    time::{Duration, Instant},
};
use windows024_r7_query_proof::backend::*;
struct Scene {
    root: Option<tempfile::TempDir>,
    home: Option<PinnedDirectory>,
    out: Option<File>,
    err: Option<File>,
}
impl Scene {
    fn new() -> Self {
        let root = tempfile::Builder::new()
            .prefix("ocx024-r7-native-abort-")
            .tempdir()
            .unwrap();
        fs::copy(
            env!("CARGO_BIN_EXE_r7-abort-fixture"),
            root.path().join("fixture.exe"),
        )
        .unwrap();
        fs::create_dir(root.path().join("profile")).unwrap();
        let home = create_private_directory(&root.path().join("home")).unwrap();
        let out = File::create(root.path().join("stdout.log")).unwrap();
        let err = File::create(root.path().join("stderr.log")).unwrap();
        Self {
            root: Some(root),
            home: Some(home),
            out: Some(out),
            err: Some(err),
        }
    }
    fn path(&self) -> &std::path::Path {
        self.root.as_ref().unwrap().path()
    }
    fn launch(&self) -> Owner {
        let image = self.path().join("fixture.exe");
        let marker = self.path().join("must-not-execute");
        let args = [marker.into_os_string()];
        let env = BTreeMap::<OsString, OsString>::new();
        let native = OwnedRuntime::create_suspended(LaunchSpec {
            executable: &image,
            arguments: &args,
            working_directory: self.path(),
            user_profile: &self.path().join("profile"),
            opencodex_home: self.home.as_ref().unwrap().path(),
            environment: &env,
            stdout: self.out.as_ref().unwrap(),
            stderr: self.err.as_ref().unwrap(),
        })
        .unwrap();
        Owner(native)
    }
}
impl Drop for Scene {
    fn drop(&mut self) {
        self.home.take();
        self.out.take();
        self.err.take();
        if std::thread::panicking() {
            if let Some(root) = self.root.take() {
                eprintln!(
                    "failed native abort fixture retained: {}",
                    root.keep().display()
                );
            }
        }
    }
}
struct Owner(OwnedRuntime);
impl Drop for Owner {
    fn drop(&mut self) {
        if !self
            .0
            .root_state()
            .is_ok_and(|s| matches!(s, ProcessState::Exited { .. }))
        {
            eprintln!("separate owned synthetic cleanup after failure; not recovery success");
            self.0
                .force_owned_cleanup(Instant::now() + Duration::from_secs(3))
                .unwrap();
        }
    }
}
#[test]
fn r7_abort_original_primary_job_is_quiesced_forced242_and_resume_permanently_revoked() {
    let scene = Scene::new();
    let mut owner = scene.launch();
    let started = Instant::now();
    let prepared = owner
        .0
        .prepare_before_first_resume_abort(started, started + Duration::from_secs(30))
        .unwrap();
    assert!(prepared.deadline() <= started + Duration::from_secs(10));
    assert!(owner.0.resume_revoked_by_abort());
    assert!(owner.0.resume().is_err());
    assert!(owner
        .0
        .require_creator_not_resumed(prepared.deadline())
        .is_err());
    let quiescent = owner.0.quiesce_before_first_resume(prepared).unwrap();
    assert_eq!(quiescent.forced_raw_exit(), 242);
    owner.0.verify_before_resume_quiescent(&quiescent).unwrap();
    assert!(owner.0.resume().is_err());
    assert!(!owner.0.creator_has_resumed());
    assert!(owner
        .0
        .prepare_before_first_resume_abort(Instant::now(), Instant::now() + Duration::from_secs(10))
        .is_err());
    assert!(!scene.path().join("must-not-execute").exists());
    println!("original retained primary/original unchanged creation-cycle baseline + exact original Job total1/active1 -> forced242 total1/active0; Resume revoked; no client restore/NormalStop/Closed/successor proof");
}
#[test]
fn r7_abort_future_expired_and_replayed_prepare_keep_original_owner_revoked() {
    for future in [false, true] {
        let scene = Scene::new();
        let mut owner = scene.launch();
        let now = Instant::now();
        let started = if future {
            now + Duration::from_secs(1)
        } else {
            now - Duration::from_secs(11)
        };
        assert!(owner
            .0
            .prepare_before_first_resume_abort(started, now + Duration::from_secs(20))
            .is_err());
        assert!(owner.0.resume_revoked_by_abort());
        assert!(owner.0.resume().is_err());
        assert!(owner
            .0
            .prepare_before_first_resume_abort(now, now + Duration::from_secs(10))
            .is_err());
        assert!(!scene.path().join("must-not-execute").exists());
    }
}
#[test]
fn r7_abort_foreign_original_native_proof_refuses_before_any_job_force() {
    let scene = Scene::new();
    let mut a = scene.launch();
    let mut b = scene.launch();
    let started = Instant::now();
    let prepared =
        a.0.prepare_before_first_resume_abort(started, started + Duration::from_secs(10))
            .unwrap();
    assert!(b.0.quiesce_before_first_resume(prepared).is_err());
    assert!(!a.0.forced_cleanup_requested());
    assert!(!b.0.forced_cleanup_requested());
    assert!(a.0.resume().is_err());
    assert!(b.0.resume().is_err());
    assert!(!scene.path().join("must-not-execute").exists());
}
#[test]
fn r7_abort_resumed_original_never_enters_before_resume_branch() {
    let scene = Scene::new();
    let mut owner = scene.launch();
    owner.0.resume().unwrap();
    let now = Instant::now();
    assert!(owner
        .0
        .prepare_before_first_resume_abort(now, now + Duration::from_secs(10))
        .is_err());
    assert!(owner.0.creator_has_resumed());
    assert!(owner.0.resume_revoked_by_abort());
    assert!(!owner.0.forced_cleanup_requested());
}

#[test]
fn r7_abort_short_parent_window_cannot_be_renewed_or_force_after_expiry() {
    let scene = Scene::new();
    let mut owner = scene.launch();
    let now = Instant::now();
    let parent = now + Duration::from_millis(100);
    let prepared = owner
        .0
        .prepare_before_first_resume_abort(now, parent)
        .unwrap();
    assert_eq!(prepared.deadline(), parent);
    std::thread::sleep(Duration::from_millis(110));
    assert!(owner.0.quiesce_before_first_resume(prepared).is_err());
    assert!(!owner.0.forced_cleanup_requested());
    assert!(owner.0.resume().is_err());
    let now = Instant::now();
    assert!(owner
        .0
        .prepare_before_first_resume_abort(now, now + Duration::from_secs(10))
        .is_err());
    assert!(!scene.path().join("must-not-execute").exists());
}

#[test]
fn r7_abort_quiescent_proof_refuses_other_original_job() {
    let scene = Scene::new();
    let mut a = scene.launch();
    let mut b = scene.launch();
    let now = Instant::now();
    let prepared =
        a.0.prepare_before_first_resume_abort(now, now + Duration::from_secs(10))
            .unwrap();
    let proof = a.0.quiesce_before_first_resume(prepared).unwrap();
    assert!(b.0.verify_before_resume_quiescent(&proof).is_err());
    assert!(!b.0.forced_cleanup_requested());
    a.0.verify_before_resume_quiescent(&proof).unwrap();
    assert!(!scene.path().join("must-not-execute").exists());
}
