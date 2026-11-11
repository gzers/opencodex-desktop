use super::*;
use std::{
    io::{Read, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    thread,
    time::Duration,
};

struct OwnedChild(Child);
impl OwnedChild {
    fn gated(code: u8) -> Self {
        Self::script("read owned_fixture; exit \"$1\"", code)
    }
    fn script(script: &str, code: u8) -> Self {
        let script = format!("printf R; {script}");
        let mut child = Self(
            Command::new("/bin/sh")
                .args(["-c", &script, "owned-fixture", &code.to_string()])
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
        // posix_spawn can return before the initial exec event is delivered.
        // Subscribe only after the actual image reaches its explicit gate.
        let mut ready = [0];
        child
            .0
            .stdout
            .as_mut()
            .unwrap()
            .read_exact(&mut ready)
            .unwrap();
        assert_eq!(&ready, b"R");
        child
    }
    fn release(&mut self) {
        self.0
            .stdin
            .take()
            .unwrap()
            .write_all(b"continue\n")
            .unwrap();
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
        }
        let _ = self.0.wait();
    }
}
fn wait(watch: &mut ExitWatch) -> AppResult<NormalExitProof> {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(proof) = watch.normal_exit(deadline)? {
            return Ok(proof);
        }
        thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn live_exact_generation_requires_a_real_zero_exit_event_not_spawn_or_absence() {
    let mut child = OwnedChild::gated(0);
    let generation = Generation::observe(child.0.id()).unwrap();
    let mut watch = ExitWatch::capture(generation).unwrap();
    assert!(watch
        .normal_exit(Instant::now() + Duration::from_secs(1))
        .unwrap()
        .is_none());
    child.release();
    let proof = wait(&mut watch).unwrap();
    assert_eq!(proof.generation(), generation);
    assert_eq!(child.0.wait().unwrap().code(), Some(0));
    assert_eq!(
        watch
            .normal_exit(Instant::now() + Duration::from_secs(1))
            .unwrap()
            .unwrap()
            .generation(),
        generation
    );
}

#[test]
fn actual_nonzero_exit_and_signal_never_become_normal_stop() {
    let mut child = OwnedChild::gated(7);
    let mut watch = ExitWatch::capture(Generation::observe(child.0.id()).unwrap()).unwrap();
    child.release();
    assert!(
        matches!(wait(&mut watch),Err(AppError::RuntimeManaged{code,..}) if code=="lifecycle_exit_nonzero")
    );
    assert_eq!(child.0.wait().unwrap().code(), Some(7));
    let mut child = OwnedChild::gated(0);
    let mut watch = ExitWatch::capture(Generation::observe(child.0.id()).unwrap()).unwrap();
    assert_eq!(unsafe { libc::kill(child.0.id() as i32, libc::SIGTERM) }, 0);
    assert!(
        matches!(wait(&mut watch),Err(AppError::RuntimeManaged{code,..}) if code=="lifecycle_exit_signalled")
    );
    assert!(!child.0.wait().unwrap().success());
}

#[test]
fn wrong_uid_birth_generation_and_self_are_refused_without_touching_live_process() {
    let child = OwnedChild::gated(0);
    let expected = Generation::observe(child.0.id()).unwrap();
    for changed in [
        Generation {
            uid: expected.uid + 1,
            ..expected
        },
        Generation {
            start_seconds: expected.start_seconds + 1,
            ..expected
        },
        Generation {
            start_micros: (expected.start_micros + 1) % 1_000_000,
            ..expected
        },
    ] {
        assert!(ExitWatch::capture(changed).is_err());
    }
    assert!(ExitWatch::capture(Generation::observe(std::process::id()).unwrap()).is_err());
    assert_eq!(Generation::observe(child.0.id()).unwrap(), expected);
}

#[test]
fn already_exited_process_is_not_retroactive_normal_exit_proof() {
    let mut child = OwnedChild::gated(0);
    let expected = Generation::observe(child.0.id()).unwrap();
    child.release();
    assert!(child.0.wait().unwrap().success());
    assert!(ExitWatch::capture(expected).is_err());
    assert!(Generation::observe(expected.pid).is_err());
}

#[test]
fn elapsed_budget_never_accepts_a_queued_or_cached_exit_event() {
    let mut child = OwnedChild::gated(0);
    let mut watch = ExitWatch::capture(Generation::observe(child.0.id()).unwrap()).unwrap();
    child.release();
    assert!(child.0.wait().unwrap().success());
    assert!(
        matches!(watch.normal_exit(Instant::now()),Err(AppError::RuntimeManaged{code,..}) if code=="lifecycle_timeout")
    );
    assert!(wait(&mut watch).is_ok());
    assert!(watch.normal_exit(Instant::now()).is_err());
}

#[test]
fn fork_after_subscription_invalidates_even_a_coalesced_zero_exit() {
    let mut child = OwnedChild::script("read owned_fixture; /bin/sleep 0.02 & wait; exit 0", 0);
    let mut watch = ExitWatch::capture(Generation::observe(child.0.id()).unwrap()).unwrap();
    child.release();
    assert_eq!(child.0.wait().unwrap().code(), Some(0));
    for _ in 0..2 {
        assert!(
            matches!(wait(&mut watch), Err(AppError::RuntimeManaged { code, .. }) if code == "lifecycle_exit_membership_changed"),
            "a real fork must not become exit authority after its event is consumed"
        );
    }
}

#[test]
fn exec_after_subscription_invalidates_even_when_birth_and_exit_code_match() {
    let mut child = OwnedChild::script("read owned_fixture; exec /usr/bin/true", 0);
    let generation = Generation::observe(child.0.id()).unwrap();
    let mut watch = ExitWatch::capture(generation).unwrap();
    child.release();
    assert_eq!(child.0.wait().unwrap().code(), Some(0));
    for _ in 0..2 {
        assert!(
            matches!(wait(&mut watch), Err(AppError::RuntimeManaged { code, .. }) if code == "lifecycle_exit_image_changed"),
            "an exec must not reuse the former image's exit authority"
        );
    }
}

#[test]
fn fork_or_exec_is_refused_while_original_pid_is_still_alive_and_stays_refused() {
    for (script, code) in [
        (
            "read owned_fixture; /bin/sleep 0.02 & wait; read again; exit 0",
            "lifecycle_exit_membership_changed",
        ),
        (
            "read owned_fixture; exec /bin/cat >/dev/null",
            "lifecycle_exit_image_changed",
        ),
    ] {
        let mut child = OwnedChild::script(script, 0);
        let mut watch = ExitWatch::capture(Generation::observe(child.0.id()).unwrap()).unwrap();
        child
            .0
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"continue\n")
            .unwrap();
        assert!(
            matches!(wait(&mut watch), Err(AppError::RuntimeManaged { code: actual, .. }) if actual == code)
        );
        assert!(child.0.try_wait().unwrap().is_none());
        child.release();
        assert_eq!(child.0.wait().unwrap().code(), Some(0));
        assert!(
            matches!(wait(&mut watch), Err(AppError::RuntimeManaged { code: actual, .. }) if actual == code)
        );
    }
}

#[test]
fn malformed_wait_status_is_not_a_successful_event() {
    assert_eq!(decode_wait(0).unwrap(), Exit::Normal(0));
    assert_eq!(decode_wait(7 << 8).unwrap(), Exit::Normal(7));
    assert_eq!(decode_wait(15).unwrap(), Exit::Signal(15));
    for status in [-1, 65536, 0x7f, 0xffff, i64::MAX] {
        assert!(decode_wait(status).is_err());
    }
}

#[test]
fn same_uid_nonchild_process_is_observed_without_waitpid_or_reaping_its_launcher() {
    const CHILD: &str = "OCXD024_EXIT_NONCHILD";
    const LEAF: &str = "OCXD024_EXIT_NONCHILD_LEAF";
    const TEST: &str = "infrastructure::macos_process_exit::tests::same_uid_nonchild_process_is_observed_without_waitpid_or_reaping_its_launcher";
    if let Some(root) = std::env::var_os(LEAF) {
        let root = PathBuf::from(root);
        assert!(root.is_absolute() && root.starts_with("/private/tmp"));
        std::fs::write(root.join("leaf-ready"), b"ready").unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !root.join("release").is_file() {
            if Instant::now() >= deadline {
                std::process::exit(91);
            }
            thread::sleep(Duration::from_millis(10));
        }
        std::process::exit(0);
    }
    if let Some(root) = std::env::var_os(CHILD) {
        let root = PathBuf::from(root);
        assert!(root.is_absolute() && root.starts_with("/private/tmp"));
        // Pure Rust wait: a shell invoking sleep would itself fork while the
        // supposedly stable single-process positive fixture is subscribed.
        let child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", TEST, "--nocapture"])
            .env_remove(CHILD)
            .env(LEAF, &root)
            .current_dir(&root)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        std::fs::write(root.join("pid"), child.id().to_string()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !root.join("leaf-ready").is_file() {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(10));
        }
        std::process::exit(0);
    }
    let root = tempfile::tempdir().unwrap();
    let root = root.path().canonicalize().unwrap();
    let result = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--nocapture"])
        .env(CHILD, &root)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let pid = std::fs::read_to_string(root.join("pid"))
        .unwrap()
        .parse()
        .unwrap();
    let generation = Generation::observe(pid).unwrap();
    let mut watch = ExitWatch::capture(generation).unwrap();
    std::fs::write(root.join("release"), b"release owned fixture").unwrap();
    let proof = wait(&mut watch).unwrap();
    assert_eq!(proof.generation(), generation);
    println!(
        "KERNEL_NONCHILD_NORMAL_EXIT pid={} uid={} generation={}:{} exit_code=0",
        generation.pid, generation.uid, generation.start_seconds, generation.start_micros
    );
}
