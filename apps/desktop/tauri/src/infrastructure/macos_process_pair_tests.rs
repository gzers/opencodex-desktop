use super::*;
use std::{
    fs,
    path::PathBuf,
    process::{Child, Command, Stdio},
    thread,
};
const ROLE: &str = "OCXD024_EXIT_PAIR_ROLE";
const ROOT: &str = "OCXD024_EXIT_PAIR_ROOT";
const HELPER: &str = "infrastructure::macos_process_pair::tests::pair_helper";
struct HelperChild {
    child: Child,
    root: PathBuf,
}
impl Drop for HelperChild {
    fn drop(&mut self) {
        let _ = fs::write(
            self.root.join("release-service"),
            b"release owned helper child",
        );
        let _ = self.child.wait();
    }
}
fn until(mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !ready() {
        assert!(
            Instant::now() < deadline,
            "owned pair fixture gate timed out"
        );
        thread::sleep(Duration::from_millis(2));
    }
}
#[test]
fn pair_helper() {
    let Some(role) = std::env::var_os(ROLE) else {
        return;
    };
    let root = PathBuf::from(std::env::var_os(ROOT).unwrap());
    assert!(root.is_absolute() && root.starts_with("/private/tmp"));
    let deadline = Instant::now() + Duration::from_secs(8);
    if role == "service" {
        fs::write(root.join("service-ready"), b"ready").unwrap();
        while !root.join("release-service").is_file() {
            if Instant::now() >= deadline {
                std::process::exit(91);
            }
            thread::sleep(Duration::from_millis(2));
        }
        std::process::exit(
            std::env::var("OCXD024_PAIR_SERVICE_CODE")
                .unwrap()
                .parse()
                .unwrap(),
        );
    }
    let child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", HELPER, "--nocapture"])
        .env(ROLE, "service")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    // Ordinary error unwinds release/reap the child. The explicit exec
    // negative intentionally replaces its launcher while the leaf stays
    // alive; the outer fixture retains the leaf generation and releases it.
    let mut child = HelperChild {
        child,
        root: root.clone(),
    };
    until(|| root.join("service-ready").is_file());
    fs::write(root.join("service-pid"), child.child.id().to_string()).unwrap();
    fs::write(root.join("pair-ready"), b"ready").unwrap();
    let mut changed = false;
    loop {
        if !changed && root.join("change-launcher").is_file() {
            changed = true;
            match std::env::var("OCXD024_PAIR_CHANGE").unwrap().as_str() {
                "fork" => {
                    assert!(Command::new("/usr/bin/true").status().unwrap().success());
                }
                "exec" => {
                    use std::os::unix::process::CommandExt;
                    let error = Command::new("/usr/bin/true").exec();
                    panic!("owned fixture exec failed: {error}");
                }
                _ => panic!("unknown owned fixture change"),
            }
            fs::write(root.join("launcher-changed"), b"changed").unwrap();
        }
        if root.join("release-launcher").is_file() {
            child.child.wait().unwrap();
            std::process::exit(
                std::env::var("OCXD024_PAIR_LAUNCHER_CODE")
                    .unwrap()
                    .parse()
                    .unwrap(),
            );
        }
        if Instant::now() >= deadline {
            fs::write(root.join("release-service"), b"release owned child").unwrap();
            child.child.wait().unwrap();
            std::process::exit(91);
        }
        thread::sleep(Duration::from_millis(2));
    }
}
struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    child: Child,
    service: Generation,
    launcher: Generation,
}
impl Fixture {
    fn new(service_code: u8, launcher_code: u8, change: &str) -> Self {
        let temp = tempfile::tempdir_in("/private/tmp").unwrap();
        let root = temp.path().canonicalize().unwrap();
        let child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", HELPER, "--nocapture"])
            .env(ROLE, "launcher")
            .env(ROOT, &root)
            .env("OCXD024_PAIR_SERVICE_CODE", service_code.to_string())
            .env("OCXD024_PAIR_LAUNCHER_CODE", launcher_code.to_string())
            .env("OCXD024_PAIR_CHANGE", change)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        until(|| root.join("pair-ready").is_file());
        let pid = fs::read_to_string(root.join("service-pid"))
            .unwrap()
            .parse()
            .unwrap();
        let service = Generation::observe(pid).unwrap();
        let launcher = Generation::observe(child.id()).unwrap();
        Self {
            _temp: temp,
            root,
            child,
            service,
            launcher,
        }
    }
    fn watch(&self) -> PairWatch {
        PairWatch::capture(
            self.service,
            self.launcher,
            Instant::now() + Duration::from_secs(3),
        )
        .unwrap()
    }
    fn release(&self) {
        fs::write(self.root.join("release-service"), b"release").unwrap();
        fs::write(self.root.join("release-launcher"), b"release").unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.release();
        let deadline = Instant::now() + Duration::from_secs(2);
        while self.child.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(2));
        }
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
        while Generation::observe(self.service.pid).is_ok() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(2));
        }
    }
}
#[test]
fn service_exit_alone_never_completes_the_bound_pair() {
    let f = Fixture::new(0, 0, "none");
    let mut watch = f.watch();
    fs::write(f.root.join("release-service"), b"release").unwrap();
    until(|| Generation::observe(f.service.pid).is_err());
    assert!(watch
        .normal_exit(Instant::now() + Duration::from_secs(1))
        .unwrap()
        .is_none());
    assert_eq!(Generation::observe(f.launcher.pid).unwrap(), f.launcher);
    f.release();
    let deadline = Instant::now() + Duration::from_secs(2);
    let proof = watch.wait(deadline).unwrap();
    proof.require(f.service, f.launcher, deadline).unwrap();
    let wrong = Generation {
        start_micros: (f.launcher.start_micros + 1) % 1_000_000,
        ..f.launcher
    };
    assert!(proof.require(f.service, wrong, deadline).is_err());
    assert!(proof.require(f.launcher, f.service, deadline).is_err());
    assert!(proof
        .require(f.service, f.launcher, Instant::now())
        .is_err());
}
#[test]
fn unrelated_duplicate_and_wrong_generation_are_refused_before_subscription() {
    let a = Fixture::new(0, 0, "none");
    let b = Fixture::new(0, 0, "none");
    let deadline = Instant::now() + Duration::from_secs(2);
    assert!(PairWatch::capture(a.service, b.launcher, deadline).is_err());
    assert!(PairWatch::capture(a.service, a.service, deadline).is_err());
    let wrong = Generation {
        start_seconds: a.launcher.start_seconds + 1,
        ..a.launcher
    };
    assert!(PairWatch::capture(a.service, wrong, deadline).is_err());
    assert!(PairWatch::capture(a.service, a.launcher, Instant::now()).is_err());
}
#[test]
fn either_nonzero_exit_refuses_the_pair_without_partial_success() {
    for (service, launcher) in [(7, 0), (0, 7)] {
        let f = Fixture::new(service, launcher, "none");
        let mut watch = f.watch();
        f.release();
        assert!(
            matches!(watch.wait(Instant::now()+Duration::from_secs(2)),Err(AppError::RuntimeManaged{code,..}) if code=="lifecycle_exit_nonzero")
        );
    }
}
#[test]
fn launcher_fork_or_exec_is_refused_even_while_service_has_not_exited() {
    for (change, expected) in [
        ("fork", "lifecycle_exit_membership_changed"),
        ("exec", "lifecycle_exit_image_changed"),
    ] {
        let f = Fixture::new(0, 0, change);
        let mut watch = f.watch();
        fs::write(f.root.join("change-launcher"), b"change").unwrap();
        if change == "fork" {
            until(|| f.root.join("launcher-changed").is_file());
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        assert!(
            matches!(watch.wait(deadline),Err(AppError::RuntimeManaged{code,..}) if code==expected)
        );
        assert_eq!(Generation::observe(f.service.pid).unwrap(), f.service);
        f.release();
        assert!(
            matches!(watch.wait(deadline),Err(AppError::RuntimeManaged{code,..}) if code==expected)
        );
    }
}
#[test]
fn queued_and_cached_pair_cannot_outlive_the_original_deadline() {
    let mut f = Fixture::new(0, 0, "none");
    let mut watch = f.watch();
    f.release();
    assert!(f.child.wait().unwrap().success());
    assert!(watch.normal_exit(Instant::now()).is_err());
    watch.wait(Instant::now() + Duration::from_secs(2)).unwrap();
    assert!(watch.normal_exit(Instant::now()).is_err());
    assert!(PairWatch::capture(
        f.service,
        f.launcher,
        Instant::now() + Duration::from_secs(1)
    )
    .is_err());
}
