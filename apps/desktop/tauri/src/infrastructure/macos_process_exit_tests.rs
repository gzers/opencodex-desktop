use super::*;
use std::{
    io::Write,
    path::PathBuf,
    process::{Child, Command, Stdio},
    thread,
    time::Duration,
};

struct OwnedChild(Child);
impl OwnedChild {
    fn gated(code: u8) -> Self {
        Self(
            Command::new("/bin/sh")
                .args([
                    "-c",
                    "read owned_fixture; exit \"$1\"",
                    "owned-fixture",
                    &code.to_string(),
                ])
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        )
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
    if let Some(root) = std::env::var_os(CHILD) {
        let root = PathBuf::from(root);
        assert!(root.is_absolute() && root.starts_with("/private/tmp"));
        let release = root.join("release");
        let child=Command::new("/bin/sh").args(["-c","i=0; while [ ! -f \"$1\" ]; do /bin/sleep 0.02; i=$((i+1)); [ $i -lt 250 ] || exit 91; done; exit 0","owned-fixture"]).arg(release).env_clear().env("PATH","/usr/bin:/bin").current_dir(&root).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap();
        std::fs::write(root.join("pid"), child.id().to_string()).unwrap();
        std::process::exit(0);
    }
    let root = tempfile::tempdir().unwrap();
    let root = root.path().canonicalize().unwrap();
    let result=Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_process_exit::tests::same_uid_nonchild_process_is_observed_without_waitpid_or_reaping_its_launcher","--nocapture"]).env(CHILD,&root).output().unwrap();
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
