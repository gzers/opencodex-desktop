#![cfg(windows)]
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::{self, File},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
use windows024_native_proof::backend::*;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
    home: Option<PinnedDirectory>,
    out: Option<File>,
    err: Option<File>,
    executable: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "ocxd024-process-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let executable = root.join("fixture.exe");
        fs::copy(env!("CARGO_BIN_EXE_native024-fixture"), &executable).unwrap();
        let home = create_private_directory(&root.join("home")).unwrap();
        let out = File::create(root.join("stdout.log")).unwrap();
        let err = File::create(root.join("stderr.log")).unwrap();
        Self {
            root,
            home: Some(home),
            out: Some(out),
            err: Some(err),
            executable,
        }
    }
    fn launch(&self, args: &[OsString]) -> Guarded {
        let mut env = BTreeMap::new();
        env.insert("SystemRoot".into(), std::env::var_os("SystemRoot").unwrap());
        env.insert("PATH".into(), std::env::var_os("PATH").unwrap());
        Guarded(
            OwnedRuntime::create_suspended(LaunchSpec {
                executable: &self.executable,
                arguments: args,
                working_directory: &self.root,
                home: self.home.as_ref().unwrap().path(),
                environment: &env,
                stdout: self.out.as_ref().unwrap(),
                stderr: self.err.as_ref().unwrap(),
            })
            .unwrap(),
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.home.take();
        self.out.take();
        self.err.take();
        if let Err(e) = fs::remove_dir_all(&self.root) {
            if std::thread::panicking() {
                eprintln!("separate fixture cleanup failed: {e}");
            } else {
                panic!("exact fixture cleanup failed: {e}");
            }
        }
    }
}
struct Guarded(OwnedRuntime);
impl Drop for Guarded {
    fn drop(&mut self) {
        if !self
            .0
            .family_state()
            .map(|s| s.complete_exit())
            .unwrap_or(false)
        {
            eprintln!("test-only separate forced cleanup after incomplete case");
            self.0
                .force_owned_cleanup(Instant::now() + Duration::from_secs(3))
                .expect("owned test cleanup");
        }
    }
}
fn wait_file(path: &Path) {
    let end = Instant::now() + Duration::from_secs(5);
    while !path.exists() {
        assert!(Instant::now() < end, "test marker missing");
        std::thread::sleep(Duration::from_millis(5));
    }
}
#[test]
fn suspended_binding_and_original_nonzero_exit() {
    let f = Fixture::new();
    let mut r = f.launch(&["exit".into(), "7".into()]);
    assert_eq!(r.0.root_state().unwrap(), ProcessState::Running);
    let good = r.0.binding().clone();
    println!("native root binding before resume: {good:?}");
    assert_eq!(good.process.token, current_identity().unwrap());
    assert_eq!(good.home_id, f.home.as_ref().unwrap().identity());
    for kind in 0..4 {
        let mut bad = good.clone();
        match kind {
            0 => bad.process.pid += 1,
            1 => bad.process.creation_filetime += 1,
            2 => bad.process.token.sid.push_str("-wrong"),
            _ => bad.home = f.root.join("other"),
        };
        assert!(r.0.verify_binding(&bad).is_err());
    }
    r.0.resume().unwrap();
    assert!(r.0.resume().is_err());
    let exit =
        r.0.wait_normal_exit(Instant::now() + Duration::from_secs(5))
            .unwrap();
    assert_eq!(exit.root, ProcessState::Exited { code: 7 });
    assert!(exit.complete_exit());
    println!("original exit through retained handles: {exit:?}");
    assert!(
        r.0.wait_normal_exit(Instant::now() - Duration::from_millis(1))
            .is_err(),
        "late observation cannot extend caller deadline"
    );
    drop(r);
    drop(f);
}
#[test]
fn home_environment_arguments_and_pre_resume_window() {
    let f = Fixture::new();
    let root = f.root.clone();
    let output = root.join("env.txt");
    let text = OsString::from("spaces \"quoted\" Unicode 雪 trailing\\");
    let mut r = f.launch(&["env".into(), output.as_os_str().into(), text.clone()]);
    std::thread::sleep(Duration::from_millis(30));
    assert!(
        !output.exists(),
        "no user code before Job assignment/resume"
    );
    r.0.resume().unwrap();
    r.0.wait_normal_exit(Instant::now() + Duration::from_secs(5))
        .unwrap();
    let value = fs::read_to_string(&output).unwrap();
    assert_eq!(
        value,
        format!(
            "{}\n{}\n{}",
            f.home.as_ref().unwrap().path().display(),
            f.home.as_ref().unwrap().path().display(),
            text.to_string_lossy()
        )
    );
    drop(r);
    drop(f);
}
#[test]
fn root_exit_is_not_complete_while_owned_child_remains() {
    let f = Fixture::new();
    let root = f.root.clone();
    let child_ready = root.join("child-pid");
    let child_release = root.join("child-release");
    let root_release = root.join("root-release");
    let mut r = f.launch(&[
        "family".into(),
        child_ready.as_os_str().into(),
        child_release.as_os_str().into(),
        root_release.as_os_str().into(),
    ]);
    r.0.resume().unwrap();
    wait_file(&child_ready);
    let pid = fs::read_to_string(&child_ready)
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let before = r.0.family_state().unwrap();
    println!("controlled original family: {before:?}");
    assert!(before.active_members.iter().any(|p| p.pid == pid));
    fs::write(&root_release, b"root exits normally").unwrap();
    assert!(r
        .0
        .wait_normal_exit(Instant::now() + Duration::from_millis(80))
        .is_err());
    assert_eq!(r.0.root_state().unwrap(), ProcessState::Exited { code: 0 });
    assert!(r
        .0
        .family_state()
        .unwrap()
        .active_members
        .iter()
        .any(|p| p.pid == pid));
    fs::write(&child_release, b"child normal exit").unwrap();
    assert!(r
        .0
        .wait_normal_exit(Instant::now() + Duration::from_secs(5))
        .unwrap()
        .complete_exit());
    drop(r);
    drop(f);
}
#[test]
fn real_ipv4_ipv6_listeners_bind_to_original_job_generation() {
    let f = Fixture::new();
    let root = f.root.clone();
    let ready = root.join("ports");
    let release = root.join("release");
    let mut r = f.launch(&[
        "listen".into(),
        ready.as_os_str().into(),
        release.as_os_str().into(),
    ]);
    r.0.resume().unwrap();
    wait_file(&ready);
    let ports: Vec<u16> = fs::read_to_string(&ready)
        .unwrap()
        .lines()
        .map(|s| s.parse().unwrap())
        .collect();
    let observed = r.0.listeners().unwrap();
    println!("native IPv4/IPv6 observations: {observed:?}");
    assert_eq!(observed.listeners.len(), 2);
    for (l, p) in &observed.listeners {
        assert!(ports.contains(&l.port));
        assert_eq!(p, &r.0.binding().process);
    }
    assert!(observed
        .listeners
        .iter()
        .any(|(l, _)| l.family == ListenerFamily::Ipv4));
    assert!(observed
        .listeners
        .iter()
        .any(|(l, _)| l.family == ListenerFamily::Ipv6));
    fs::write(&release, b"normal exit").unwrap();
    r.0.wait_normal_exit(Instant::now() + Duration::from_secs(5))
        .unwrap();
    assert!(listener_exit_readback(&observed).unwrap().is_empty());
    drop(r);
    drop(f);
}

#[test]
fn timeout_is_observation_only_and_force_is_separately_typed() {
    let f = Fixture::new();
    let ready = f.root.join("live");
    let release = f.root.join("release");
    let mut r = f.launch(&[
        "child".into(),
        ready.as_os_str().into(),
        release.as_os_str().into(),
    ]);
    r.0.resume().unwrap();
    wait_file(&ready);
    assert!(r
        .0
        .wait_normal_exit(Instant::now() + Duration::from_millis(30))
        .is_err());
    assert_eq!(r.0.root_state().unwrap(), ProcessState::Running);
    let cleanup =
        r.0.force_owned_cleanup(Instant::now() + Duration::from_secs(3))
            .unwrap();
    assert!(cleanup.forced);
    assert!(cleanup.final_state.complete_exit());
    assert_eq!(cleanup.final_state.root, ProcessState::Exited { code: 242 });
    println!("separate forced-cleanup fact, not normal stop: {cleanup:?}");
}

#[test]
fn replaced_listener_on_original_endpoint_prevents_exit_readback() {
    let f = Fixture::new();
    let root = f.root.clone();
    let ready = root.join("ports");
    let release = root.join("release");
    let mut r = f.launch(&[
        "listen".into(),
        ready.as_os_str().into(),
        release.as_os_str().into(),
    ]);
    r.0.resume().unwrap();
    wait_file(&ready);
    let seen = r.0.listeners().unwrap();
    fs::write(&release, b"normal").unwrap();
    r.0.wait_normal_exit(Instant::now() + Duration::from_secs(5))
        .unwrap();
    let original = &seen
        .listeners
        .iter()
        .find(|(l, _)| l.family == ListenerFamily::Ipv4)
        .unwrap()
        .0;
    let replacement = std::net::TcpListener::bind((original.address, original.port)).unwrap();
    let remaining = listener_exit_readback(&seen).unwrap();
    assert!(remaining
        .iter()
        .any(|l| l.pid == std::process::id() && l.port == original.port));
    drop(replacement);
    assert!(listener_exit_readback(&seen).unwrap().is_empty());
}

#[test]
fn case_duplicate_environment_is_rejected_without_running_child() {
    let f = Fixture::new();
    let marker = f.root.join("should-not-run");
    let args = ["env".into(), marker.as_os_str().into(), "x".into()];
    let mut env = BTreeMap::new();
    env.insert("PATH".into(), "a".into());
    env.insert("Path".into(), "b".into());
    let result = OwnedRuntime::create_suspended(LaunchSpec {
        executable: &f.executable,
        arguments: &args,
        working_directory: &f.root,
        home: f.home.as_ref().unwrap().path(),
        environment: &env,
        stdout: f.out.as_ref().unwrap(),
        stderr: f.err.as_ref().unwrap(),
    });
    assert!(result.is_err());
    assert!(!marker.exists());
}
