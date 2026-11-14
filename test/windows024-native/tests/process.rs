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
#[test]
fn wl2_r3_native_private_pipe_keeps_actual_peer_and_one_local_instance() {
    let instance = "0".repeat(64);
    let server = PrivatePipe::create(&instance).unwrap();
    assert!(PrivatePipe::create(&instance).is_err());
    let name = server.name().to_owned();
    let deadline = Instant::now() + Duration::from_secs(3);
    let client = std::thread::spawn(move || {
        let client = PrivatePipe::connect_local(&name).unwrap();
        let actual = client.peer().unwrap();
        actual.verify().unwrap();
        assert_eq!(actual.identity().pid, std::process::id());
        client
            .write_frame(b"synthetic-single-request", deadline)
            .unwrap();
        let answer = client.read_frame(deadline).unwrap();
        assert_eq!(&answer[4..], b"fresh-native-peer-response");
    });
    let peer = server.accept(deadline).unwrap();
    assert_eq!(peer.identity().pid, std::process::id());
    assert_eq!(peer.identity().token, current_identity().unwrap());
    peer.verify().unwrap();
    peer.verify_image(&std::env::current_exe().unwrap())
        .unwrap();
    let frame = server.read_frame(deadline).unwrap();
    assert_eq!(&frame[4..], b"synthetic-single-request");
    server
        .write_frame(b"fresh-native-peer-response", deadline)
        .unwrap();
    client.join().unwrap();
    println!("same-SID private native pipe peer pid={} FILETIME={} session={}; original Job not transferred; no manager/production proof",peer.identity().pid,peer.identity().creation_filetime,peer.identity().token.session_id);
}
#[test]
fn wl2_r3_private_pipe_bounds_and_original_deadline_are_enforced() {
    assert!(PrivatePipe::create("bad-instance").is_err());
    assert!(PrivatePipe::connect_local("remote-or-untrusted").is_err());
    let server = PrivatePipe::create(&"1".repeat(64)).unwrap();
    assert!(server
        .write_frame(&vec![0u8; 65537], Instant::now() + Duration::from_secs(1))
        .is_err());
    assert!(server.accept(Instant::now()).is_err());
    let begin = Instant::now();
    assert!(server.accept(begin + Duration::from_millis(30)).is_err());
    assert!(
        begin.elapsed() < Duration::from_secs(1),
        "native cancel wait exceeded bounded test guard"
    );
}
#[test]
fn wl2_r3_private_pipe_queued_trailer_cannot_be_another_request() {
    let server = PrivatePipe::create(&"2".repeat(64)).unwrap();
    let name = server.name().to_owned();
    let deadline = Instant::now() + Duration::from_secs(3);
    let (sent, received) = std::sync::mpsc::sync_channel(1);
    let (release, wait) = std::sync::mpsc::sync_channel(1);
    let child = std::thread::spawn(move || {
        let client = PrivatePipe::connect_local(&name).unwrap();
        client.write_frame(b"first-frame", deadline).unwrap();
        client.write_frame(b"second-frame", deadline).unwrap();
        sent.send(()).unwrap();
        wait.recv_timeout(Duration::from_secs(3)).unwrap();
    });
    server.accept(deadline).unwrap();
    received.recv_timeout(Duration::from_secs(3)).unwrap();
    assert!(server.read_frame(deadline).is_err());
    release.send(()).unwrap();
    child.join().unwrap();
}
#[test]
fn wl2_r3_connected_but_silent_peer_cannot_extend_original_read_deadline() {
    let server = PrivatePipe::create(&"3".repeat(64)).unwrap();
    let name = server.name().to_owned();
    let (release, wait) = std::sync::mpsc::sync_channel(1);
    let child = std::thread::spawn(move || {
        let _client = PrivatePipe::connect_local(&name).unwrap();
        wait.recv_timeout(Duration::from_secs(3)).unwrap();
    });
    server
        .accept(Instant::now() + Duration::from_secs(3))
        .unwrap();
    let start = Instant::now();
    assert!(server
        .read_frame(start + Duration::from_millis(30))
        .is_err());
    assert!(start.elapsed() < Duration::from_secs(1));
    release.send(()).unwrap();
    child.join().unwrap();
}
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
        fs::create_dir(root.join("profile 雪")).unwrap();
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
        env.insert("HOME".into(), "must-be-overridden".into());
        env.insert("userprofile".into(), "must-be-overridden".into());
        env.insert("opencodex_home".into(), "must-be-overridden".into());
        Guarded(
            OwnedRuntime::create_suspended(LaunchSpec {
                executable: &self.executable,
                arguments: args,
                working_directory: &self.root,
                user_profile: &self.root.join("profile 雪"),
                opencodex_home: self.home.as_ref().unwrap().path(),
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
fn wl2_retained_command_parent_and_raw_query_exits() {
    let f = Fixture::new();
    let done = f.root.join("query-result");
    let release = f.root.join("query-release");
    let mut runtime = f.launch(&[
        "wl2-queries".into(),
        done.as_os_str().into(),
        release.as_os_str().into(),
    ]);
    let root_pid = runtime.0.binding().process.pid;
    runtime.0.resume().unwrap();
    wait_file(&done);
    let outcomes = fs::read_to_string(&done).unwrap();
    // This independent fixture is not the official source/call-site. Even a
    // perfectly captured query here must not receive a production role proof.
    let mut captured = 0;
    for row in outcomes.lines() {
        let pid: u32 = row.split_whitespace().next().unwrap().parse().unwrap();
        match runtime.0.captured_command(pid) {
            Ok(c) => {
                assert_eq!(c.parent.pid, root_pid);
                assert!(!c.raw_command_line.is_empty());
                assert!(c.argv.len() >= 3);
                println!("original query native argv parent={} FILETIME={} argv={:?} rawOutcome={row}; role=Unknown(no official source)", c.parent.pid, c.parent.creation_filetime, c.argv);
                captured += 1;
            }
            Err(e) => println!("original query role=Unknown rawOutcome={row} captureFailure={e}"),
        }
    }
    assert!(runtime.0.captured_command(std::process::id()).is_err());
    fs::write(&release, b"normal fixture release").unwrap();
    let final_state = runtime
        .0
        .wait_normal_exit(Instant::now() + Duration::from_secs(3))
        .unwrap();
    assert!(final_state.complete_exit());
    for row in outcomes.lines() {
        let fields: Vec<_> = row.split_whitespace().collect();
        let pid: u32 = fields[0].parse().unwrap();
        let expected: u32 = fields[2].parse().unwrap();
        let actual = final_state
            .retained_members
            .iter()
            .find(|(p, _)| p.pid == pid)
            .unwrap()
            .1;
        assert_eq!(actual, ProcessState::Exited { code: expected });
    }
    println!("captured={captured}/2; no diagnostic normal-exit exception issued");
    assert_eq!(
        captured, 2,
        "short-lived argv capture must be proven before production roles"
    );
}
#[test]
fn short_lived_job_history_is_retained_or_explicitly_unknown_never_invented() {
    let f = Fixture::new();
    let completed = f.root.join("burst-complete");
    let release = f.root.join("release");
    let mut runtime = f.launch(&[
        "burst".into(),
        completed.as_os_str().into(),
        release.as_os_str().into(),
    ]);
    runtime.0.resume().unwrap();
    wait_file(&completed);
    let observed = runtime.0.family_state();
    match &observed {
        Ok(s) => {
            println!(
                "burst history retained: total={} retained={} active={}",
                s.total_processes,
                s.retained_members.len(),
                s.active_processes
            );
            assert_eq!(s.total_processes as usize, s.retained_members.len() + 1);
            assert!(!s.complete_exit());
        }
        Err(e) => println!(
            "burst history remains explicitly unknown: kind={:?} error={}",
            e.kind(),
            e
        ),
    }
    fs::write(&release, b"explicit normal fixture release").unwrap();
    if observed.is_ok() {
        match runtime
            .0
            .wait_normal_exit(Instant::now() + Duration::from_secs(3))
        {
            Ok(s) => {
                assert_eq!(s.total_processes as usize, s.retained_members.len() + 1);
                assert!(s.complete_exit());
                println!("all retained burst members normally exited");
            }
            Err(e) => println!("burst final history unknown, no normal proof: {}", e),
        }
    }
}
#[test]
fn system_helper_job_history_requires_observed_original_image_and_exit() {
    let f = Fixture::new();
    let done = f.root.join("done");
    let release = f.root.join("release");
    let mut runtime = f.launch(&[
        "system-child".into(),
        done.as_os_str().into(),
        release.as_os_str().into(),
    ]);
    runtime.0.resume().unwrap();
    wait_file(&done);
    let observed = runtime.0.family_state();
    match &observed {
        Ok(s) => println!(
            "system helper retained: total={} retained={}",
            s.total_processes,
            s.retained_members.len()
        ),
        Err(e) => println!("system helper proof refused: {}", e),
    }
    fs::write(release, b"normal root release").unwrap();
    let final_state = runtime
        .0
        .wait_normal_exit(Instant::now() + Duration::from_secs(3))
        .unwrap();
    assert!(final_state.complete_exit());
    assert_eq!(
        final_state.total_processes as usize,
        final_state.retained_members.len() + 1
    );
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
            "{}\n{}\n{}\n{}",
            f.root.join("profile 雪").display(),
            f.root.join("profile 雪").display(),
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
        user_profile: &f.root.join("profile 雪"),
        opencodex_home: f.home.as_ref().unwrap().path(),
        environment: &env,
        stdout: f.out.as_ref().unwrap(),
        stderr: f.err.as_ref().unwrap(),
    });
    assert!(result.is_err());
    assert!(!marker.exists());
}
