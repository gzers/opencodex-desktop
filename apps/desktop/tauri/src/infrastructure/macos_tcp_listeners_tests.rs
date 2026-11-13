use super::*;
use std::{
    fs,
    net::{TcpListener, UdpSocket},
    path::PathBuf,
    process::{Child, Command, Stdio},
    thread,
    time::Duration,
};
const TEST:&str="infrastructure::macos_tcp_listeners::tests::actual_child_ipv4_ipv6_listeners_are_complete_and_udp_is_not_a_tcp_listener";
const ROOT: &str = "OCXD024_TCP_LISTENER_ROOT";
fn until(path: &std::path::Path) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !path.exists() {
        assert!(Instant::now() < deadline, "owned fixture response missing");
        thread::sleep(Duration::from_millis(5));
    }
}
struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    child: Child,
    generation: Generation,
    ports: Vec<u16>,
    udp: u16,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::Builder::new()
            .prefix("ocxd024-listener-fixture-")
            .tempdir()
            .unwrap();
        let root = temp.path().canonicalize().unwrap();
        let child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", TEST, "--nocapture"])
            .env_clear()
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .env("HOME", &root)
            .env("TMPDIR", &root)
            .env(ROOT, &root)
            .current_dir(&root)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        until(&root.join("ready.json"));
        let v: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("ready.json")).unwrap()).unwrap();
        assert_eq!(v["pid"].as_u64(), Some(child.id().into()));
        let mut ports: Vec<u16> = v["tcp"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_u64().unwrap().try_into().unwrap())
            .collect();
        ports.sort_unstable();
        ports.dedup();
        assert_eq!(ports.len(), 2);
        let generation = Generation::observe(child.id()).unwrap();
        let udp = v["udp"].as_u64().unwrap().try_into().unwrap();
        Self {
            _temp: temp,
            root,
            child,
            generation,
            ports,
            udp,
        }
    }
    fn stop(&mut self) {
        fs::write(self.root.join("release"), b"release").unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                assert_eq!(status.code(), Some(0));
                break;
            }
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(5));
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}
#[test]
fn actual_child_ipv4_ipv6_listeners_are_complete_and_udp_is_not_a_tcp_listener() {
    if let Some(root) = std::env::var_os(ROOT) {
        let root = PathBuf::from(root);
        assert!(
            root.starts_with("/private/tmp")
                && root
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("ocxd024-listener-fixture-")
        );
        let one = TcpListener::bind("127.0.0.1:0").unwrap();
        let two = TcpListener::bind("[::1]:0").unwrap();
        let udp = UdpSocket::bind("127.0.0.1:0").unwrap();
        crate::infrastructure::atomic_write::atomic_write(&root.join("ready.json"),&serde_json::to_vec(&serde_json::json!({"pid":std::process::id(),"tcp":[one.local_addr().unwrap().port(),two.local_addr().unwrap().port()],"udp":udp.local_addr().unwrap().port()})).unwrap(),0o600).unwrap();
        let mut third = None;
        let deadline = Instant::now() + Duration::from_secs(8);
        while !root.join("release").exists() {
            if third.is_none() && root.join("change").exists() {
                let socket = TcpListener::bind("127.0.0.1:0").unwrap();
                crate::infrastructure::atomic_write::atomic_write(
                    &root.join("third-port"),
                    socket.local_addr().unwrap().port().to_string().as_bytes(),
                    0o600,
                )
                .unwrap();
                third = Some(socket);
            }
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(5));
        }
        drop((one, two, udp, third));
        return;
    }
    let mut f = Fixture::new();
    let observed =
        TcpListeners::capture(f.generation, Instant::now() + Duration::from_secs(2)).unwrap();
    assert_eq!(observed.generation(), f.generation);
    assert_eq!(observed.ports(), f.ports);
    assert!(!observed.ports().contains(&f.udp));
    assert_eq!(
        observed
            ._listeners
            .iter()
            .map(|s| s.family)
            .collect::<std::collections::BTreeSet<_>>(),
        [libc::AF_INET as u32, libc::AF_INET6 as u32].into()
    );
    assert!(!format!("{observed:?}").contains("object"));
    println!(
        "NATIVE_TCP_LISTENERS ipv4=true ipv6=true ports={} udpExcluded=true",
        observed.ports().len()
    );
    f.stop();
}
#[test]
fn changed_fd_or_listener_set_between_two_observations_refuses_without_a_fake_partial_result() {
    let mut f = Fixture::new();
    let result = TcpListeners::capture_checked(
        f.generation,
        Instant::now() + Duration::from_secs(3),
        &|| {
            fs::write(f.root.join("change"), b"change").unwrap();
            until(&f.root.join("third-port"));
            Ok(())
        },
    );
    assert!(
        matches!(result,Err(AppError::RuntimeManaged{code,..}) if code=="lifecycle_listener_changed")
    );
    assert!(f.child.try_wait().unwrap().is_none());
    let third: u16 = fs::read_to_string(f.root.join("third-port"))
        .unwrap()
        .parse()
        .unwrap();
    let current =
        TcpListeners::capture(f.generation, Instant::now() + Duration::from_secs(2)).unwrap();
    assert_eq!(current.ports().len(), 3);
    assert!(current.ports().contains(&third));
    f.stop();
}
#[test]
fn wrong_generation_expired_budget_and_already_exited_process_never_become_a_listener_proof() {
    let mut f = Fixture::new();
    let wrong = Generation {
        start_seconds: f.generation.start_seconds + 1,
        ..f.generation
    };
    assert!(TcpListeners::capture(wrong, Instant::now() + Duration::from_secs(1)).is_err());
    assert!(
        matches!(TcpListeners::capture(f.generation,Instant::now()),Err(AppError::RuntimeManaged{code,..}) if code=="lifecycle_timeout")
    );
    assert!(f.child.try_wait().unwrap().is_none());
    f.stop();
    assert!(TcpListeners::capture(f.generation, Instant::now() + Duration::from_secs(1)).is_err());
}
