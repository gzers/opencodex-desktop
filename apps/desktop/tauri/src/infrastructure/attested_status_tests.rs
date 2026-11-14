use super::*;
use tokio::io::AsyncBufReadExt;
use tokio::{net::TcpListener, task::JoinHandle};
const VERSION: &str = "2.97.0-desktop023.names.1";

#[cfg(windows)]
fn icacls(path: &Path, arguments: &[&str]) {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetSystemDirectoryW(buffer: *mut u16, size: u32) -> u32;
    }
    use std::os::windows::ffi::OsStringExt;
    let mut units = vec![0u16; 32768];
    let n = unsafe { GetSystemDirectoryW(units.as_mut_ptr(), units.len() as u32) } as usize;
    assert!(n > 0 && n < units.len());
    let executable = PathBuf::from(std::ffi::OsString::from_wide(&units[..n])).join("icacls.exe");
    let status = std::process::Command::new(executable)
        .arg(path)
        .args(arguments)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap();
    assert!(status.success());
}

#[cfg(windows)]
#[tokio::test]
async fn official_owner_only_protected_record_is_private_without_system_grant() {
    let (f, task) = server("valid").await;
    let path = f.home.join("runtime-port.json");
    icacls(&path, &["/remove:g", "*S-1-5-18"]);
    let file = crate::modules::backup::safety::open_file(&path, false).unwrap();
    // Creation keeps its stricter two-principal contract. The runtime reader
    // accepts the official protected owner-only form without repairing it.
    assert!(super::super::windows_private_create::verify_private_handle(&file, false).is_err());
    assert_eq!(record(&f.home).unwrap().1, f.bytes);
    assert_eq!(
        observe(&f.home, &f.record, &f.bytes, VERSION)
            .await
            .unwrap()["status"],
        "running"
    );
    task.await.unwrap();
}

#[cfg(windows)]
#[test]
fn runtime_record_reader_still_rejects_broad_acl_and_multiple_links() {
    let f = Fixture::new(12345);
    let path = f.home.join("runtime-port.json");
    icacls(&path, &["/grant", "*S-1-1-0:(F)"]);
    assert!(record(&f.home).is_err());
    icacls(&path, &["/remove:g", "*S-1-1-0"]);
    assert!(record(&f.home).is_ok());
    std::fs::hard_link(&path, f.home.join("second-link")).unwrap();
    assert!(record(&f.home).is_err());
}
struct Fixture {
    _temp: tempfile::TempDir,
    home: PathBuf,
    record: Record,
    bytes: Vec<u8>,
}
impl Fixture {
    fn new(port: u16) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().canonicalize().unwrap();
        let bytes = serde_json::to_vec(
            &json!({"pid":42,"port":port,"attestationSecret":URL_SAFE_NO_PAD.encode([7u8;32])}),
        )
        .unwrap();
        let file = crate::infrastructure::platform::create_new_private_file(
            &home.join("runtime-port.json"),
            0o600,
        )
        .unwrap();
        use std::io::Write;
        (&file).write_all(&bytes).unwrap();
        drop(file);
        let (record, read) = record(&home).unwrap();
        assert_eq!(read, bytes);
        Self {
            _temp: temp,
            home,
            record,
            bytes,
        }
    }
}
async fn server(mode: &'static str) -> (Fixture, JoinHandle<()>) {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let f = Fixture::new(listener.local_addr().unwrap().port());
    let r = f.record.clone();
    let home = f.home.clone();
    let task = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut io = BufReader::new(stream);
        for index in 0..3 {
            let mut request = String::new();
            loop {
                let mut line = String::new();
                if io.read_line(&mut line).await.unwrap() == 0 {
                    return;
                }
                request.push_str(&line);
                if line == "\r\n" {
                    break;
                }
            }
            assert!(request.starts_with(if index == 1 {
                "GET /readyz HTTP/1.1"
            } else {
                "GET /healthz HTTP/1.1"
            }));
            assert!(request.contains("Host: 127.0.0.1:"));
            assert!(!request.contains("Authorization"));
            if mode == "slow" {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                return;
            }
            let mut body = json!({"status":if index==1{"ready"}else{"ok"},"service":"opencodex","version":VERSION,"pid":42,"port":r.port,"uptime":10.0+index as f64});
            if mode.starts_with("official024") {
                body["version"] = "2.99.0".into();
            }
            let mut status = 200;
            if ["pending", "official024-pending"].contains(&mode) && index == 1 {
                status = 503;
                body["status"] = "pending".into();
            }
            if mode == "failed" && index == 1 {
                status = 503;
                body["status"] = "failed".into();
            }
            if mode == "pid" {
                body["pid"] = 43.into();
            }
            if mode == "port" {
                body["port"] = (r.port as u64 + 1).into();
            }
            if mode == "version" {
                body["version"] = "other".into();
            }
            if mode == "foreign" {
                body["service"] = "other".into();
            }
            if mode == "uptime" && index == 2 {
                body["uptime"] = 1.into();
            }
            let mut proof = String::new();
            if index != 1 {
                let challenge = request
                    .lines()
                    .find_map(|line| line.strip_prefix("x-opencodex-attestation-challenge: "))
                    .unwrap();
                let mut mac = Hmac::<Sha256>::new_from_slice(if mode == "key" {
                    b"wrong-key"
                } else {
                    r.attestation_secret.as_bytes()
                })
                .unwrap();
                mac.update(
                    format!(
                        "opencodex-local-management-v1\n{challenge}\n{}\n{}",
                        r.pid, r.port
                    )
                    .as_bytes(),
                );
                proof = format!(
                    "x-opencodex-attestation-proof: {}\r\n",
                    URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
                );
            }
            let mut bytes = serde_json::to_vec(&body).unwrap();
            if mode == "malformed" {
                bytes = b"not-json".to_vec();
            }
            if mode == "large" {
                bytes = vec![b' '; MAX_BODY + 1];
            }
            if mode == "redirect" {
                status = 302;
            }
            let extra = if mode == "duplicate" {
                "Content-Length: 1\r\n"
            } else if mode == "chunked" {
                "Transfer-Encoding: chunked\r\n"
            } else {
                ""
            };
            let response=format!("HTTP/1.1 {status} OK\r\nContent-Length: {}\r\nConnection: keep-alive\r\n{proof}{extra}\r\n",bytes.len());
            if io.get_mut().write_all(response.as_bytes()).await.is_err() {
                return;
            }
            if io.get_mut().write_all(&bytes).await.is_err() {
                return;
            }
            if io.get_mut().flush().await.is_err() {
                return;
            }
            if mode == "replace" && index == 1 {
                std::fs::write(home.join("runtime-port.json"), b"{}").unwrap();
            }
            if mode == "disconnect" && index == 0 {
                return;
            }
        }
    });
    (f, task)
}
#[tokio::test]
async fn fresh_attested_connection_proves_ready_and_leaves_protection_unknown() {
    let (f, task) = server("valid").await;
    let value = observe(&f.home, &f.record, &f.bytes, VERSION)
        .await
        .unwrap();
    task.await.unwrap();
    assert_eq!(value["status"], "running");
    assert_eq!(value["port"], f.record.port);
    assert_eq!(value["pid"], "42");
    assert!(value["startup"].get("protection").is_none());
    let mapped = crate::modules::status::map_official_output(
        &crate::modules::status::OfficialStatusReport::from_value(value),
        None,
        crate::types::status::ConnectionState::Unconfigured,
        crate::types::status::OperationState::Idle,
    );
    assert_eq!(mapped.runtime, crate::types::status::RuntimeState::Running);
    assert_eq!(mapped.facts.protection, None);
    assert_eq!(mapped.facts.reboot_safe, None);
}
#[tokio::test]
async fn pending_or_failed_official_readiness_never_becomes_healthy() {
    for mode in ["pending", "failed"] {
        let (f, task) = server(mode).await;
        let value = observe(&f.home, &f.record, &f.bytes, VERSION)
            .await
            .unwrap();
        task.await.unwrap();
        assert_eq!(value["status"], "pending");
        assert_eq!(value["ready"], false);
    }
}
#[tokio::test]
async fn impostors_record_changes_redirects_and_unsupported_frames_fail_closed() {
    for mode in [
        "key",
        "pid",
        "port",
        "version",
        "foreign",
        "uptime",
        "malformed",
        "large",
        "redirect",
        "duplicate",
        "chunked",
        "replace",
        "disconnect",
    ] {
        let (f, task) = server(mode).await;
        assert!(
            observe(&f.home, &f.record, &f.bytes, VERSION)
                .await
                .is_err(),
            "accepted {mode}"
        );
        task.abort();
        let _ = task.await;
    }
}
#[tokio::test]
async fn a_delayed_server_cannot_escape_the_observation_deadline() {
    let (f, task) = server("slow").await;
    assert!(tokio::time::timeout(
        std::time::Duration::from_millis(50),
        observe(&f.home, &f.record, &f.bytes, VERSION)
    )
    .await
    .is_err());
    task.abort();
    let _ = task.await;
}
#[cfg(unix)]
#[test]
fn unowned_public_linked_and_oversized_record_bytes_are_not_trusted() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let f = Fixture::new(12345);
    let path = f.home.join("runtime-port.json");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(record(&f.home).is_err());
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    let target = f.home.join("original");
    std::fs::rename(&path, &target).unwrap();
    symlink(&target, &path).unwrap();
    assert!(record(&f.home).is_err());
    std::fs::remove_file(&path).unwrap();
    std::fs::hard_link(&target, &path).unwrap();
    assert!(record(&f.home).is_err());
    std::fs::remove_file(&path).unwrap();
    std::fs::rename(&target, &path).unwrap();
    std::fs::write(path, vec![b' '; 4097]).unwrap();
    assert!(record(&f.home).is_err());
}

fn installation(home: &Path) -> PathBuf {
    use crate::modules::runtime::{
        install, MANAGED_ENTRY_RELATIVE, MANAGED_PREFIX_RELATIVE, MANIFEST_FILENAME,
        OFFICIAL_PACKAGE,
    };
    let root = home.join("installed");
    let prefix = root.join(MANAGED_PREFIX_RELATIVE);
    let package = prefix.join(install::PACKAGE_SUBPATH);
    let (node, _) = install::managed_dependency_paths(&root);
    let entry = root.join(MANAGED_ENTRY_RELATIVE);
    let bin = "bin/ocx.mjs";
    for path in [
        node.parent().unwrap(),
        entry.parent().unwrap(),
        &package.join("bin"),
    ] {
        std::fs::create_dir_all(path).unwrap();
    }
    std::fs::write(&node, b"node identity fixture never executed").unwrap();
    std::fs::write(package.join(bin), b"throw Error('never execute fixture')").unwrap();
    std::fs::write(
        package.join("package.json"),
        serde_json::to_vec(&json!({"name":OFFICIAL_PACKAGE,"version":VERSION,"bin":{"ocx":bin}}))
            .unwrap(),
    )
    .unwrap();
    std::fs::write(
        &entry,
        install::entry_script(Some(&node), &package.join(bin)),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for path in [&node, &entry, &package.join(bin)] {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
    let hash = |p: &Path| crate::infrastructure::hash::sha256_file(p).unwrap();
    let proof = json!({"attempt_id":uuid::Uuid::new_v4().to_string(),"bin":bin,"node":node,"node_sha256":hash(&node),"entry_sha256":hash(&entry),"package_sha256":hash(&package.join("package.json")),"bin_sha256":hash(&package.join(bin))});
    let manifest = json!({"activation":proof,"package":OFFICIAL_PACKAGE,"version":VERSION,"tarball_sha256":"a".repeat(64),"installed_at":"2026-11-08T00:00:00Z","npm_path":"fixture never executed","scripts_enabled":false,"source":"offline"});
    std::fs::write(
        prefix.join(MANIFEST_FILENAME),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    root
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_status_consumer_uses_fresh_proofs_and_rejects_disappearing_service_or_changed_activation(
) {
    use crate::{
        infrastructure::status_source::OfficialStatusSource, modules::process::EnvironmentPolicy,
        modules::runtime::RuntimeHandle, modules::status::StatusSource,
    };
    let (f, task) = server("valid").await;
    let root = installation(&f.home);
    let data = f.home.join("desktop");
    std::fs::create_dir(&data).unwrap();
    let runtime = RuntimeHandle::initialize_managed(&data, &root);
    assert_eq!(
        runtime.kind(),
        crate::modules::runtime::RuntimeSourceKind::Managed
    );
    let source = std::sync::Arc::new(OfficialStatusSource::new(
        runtime,
        f.home.clone(),
        EnvironmentPolicy {
            opencodex_home: f.home.clone(),
            ..Default::default()
        },
    ));
    let value = {
        let source = source.clone();
        tokio::task::spawn_blocking(move || source.fetch())
            .await
            .unwrap()
            .unwrap()
    };
    task.await.unwrap();
    assert_eq!(value["status"], "running");
    // A previous ready observation cannot survive loss of the authenticated peer.
    {
        let source = source.clone();
        assert!(tokio::task::spawn_blocking(move || source.fetch_light())
            .await
            .unwrap()
            .is_err());
    }
    std::fs::write(
        root.join(crate::modules::runtime::MANAGED_ENTRY_RELATIVE),
        b"tampered",
    )
    .unwrap();
    {
        let source = source.clone();
        assert!(tokio::task::spawn_blocking(move || source.fetch())
            .await
            .unwrap()
            .is_err());
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stop_uses_one_attested_connection_and_requires_official_client_teardown() {
    for mode in [
        "valid",
        "forged",
        "deferred",
        "refused",
        "record-changed",
        "capability-pid",
        "slow-stop",
        "witness-refused",
        "duplicate-lifetime-proof",
        "retained-first-refused",
        "retained-before-post-refused",
    ] {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let f = Fixture::new(listener.local_addr().unwrap().port());
        let root = installation(&f.home);
        let data = f.home.join("desktop");
        std::fs::create_dir(&data).unwrap();
        let runtime = crate::modules::runtime::RuntimeHandle::initialize_managed(&data, &root);
        let token = format!("ocx_admin_{}", "x".repeat(43));
        crate::infrastructure::atomic_write::atomic_write(
            &f.home.join("admin-api-token"),
            token.as_bytes(),
            0o600,
        )
        .unwrap();
        let r = f.record.clone();
        let home = f.home.clone();
        let record_bytes = f.bytes.clone();
        let requests = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let log = requests.clone();
        let task = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut io = BufReader::new(stream);
            for index in 0..4 {
                let mut request = String::new();
                loop {
                    let mut line = String::new();
                    if io.read_line(&mut line).await.unwrap_or(0) == 0 {
                        return;
                    }
                    request.push_str(&line);
                    if line == "\r\n" {
                        break;
                    }
                }
                log.lock().unwrap().push((
                    request.lines().next().unwrap().to_string(),
                    request.contains("Authorization: Bearer"),
                ));
                let health = index == 0 || index == 2;
                let mut proof = String::new();
                let mut body = if health {
                    let challenge = request
                        .lines()
                        .find_map(|line| line.strip_prefix("x-opencodex-attestation-challenge: "))
                        .unwrap();
                    let mut mac = Hmac::<Sha256>::new_from_slice(if mode == "forged" {
                        b"foreign"
                    } else {
                        r.attestation_secret.as_bytes()
                    })
                    .unwrap();
                    mac.update(
                        format!(
                            "opencodex-local-management-v1\n{challenge}\n{}\n{}",
                            r.pid, r.port
                        )
                        .as_bytes(),
                    );
                    proof = format!(
                        "x-opencodex-attestation-proof: {}\r\n",
                        URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
                    );
                    if mode == "duplicate-lifetime-proof" {
                        proof.push_str("x-opencodex-runtime-lifetime-proof: A\r\nx-opencodex-runtime-lifetime-proof: B\r\n");
                    }
                    json!({"service":"opencodex","status":"ok","pid":r.pid,"port":r.port,"version":VERSION})
                } else if index == 1 {
                    json!({"schema":"ocx-stop-capabilities/1","connectedClientOwnership":true,"pid":if mode=="capability-pid"{43}else{r.pid}})
                } else {
                    json!({"success":mode!="refused","sharedTeardown":if mode=="deferred"{"deferred"}else{"performed"}})
                };
                if mode == "record-changed" && index == 2 {
                    std::fs::write(home.join("runtime-port.json"), b"{}").unwrap();
                }
                if mode == "slow-stop" && index == 3 {
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                }
                let bytes = serde_json::to_vec(&body.take()).unwrap();
                let response=format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: keep-alive\r\n{proof}\r\n",bytes.len());
                if io.get_mut().write_all(response.as_bytes()).await.is_err() {
                    return;
                }
                if io.get_mut().write_all(&bytes).await.is_err() {
                    return;
                }
                let _ = io.get_mut().flush().await;
            }
            // No second connection is used by stop_official, including its POST.
            assert!(
                tokio::time::timeout(std::time::Duration::from_millis(30), listener.accept())
                    .await
                    .is_err()
            );
            let _ = record_bytes;
        });
        let progress = std::cell::RefCell::new(Vec::new());
        let retained_calls = std::cell::Cell::new(0);
        let observed = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            stop_official_with_checks(
                runtime,
                f.home.clone(),
                42,
                |phase| progress.borrow_mut().push(phase),
                |_, _, _, _, _| {
                    if mode == "witness-refused" {
                        Err(CollectError::Unreachable)
                    } else {
                        Ok(())
                    }
                },
                |record, bytes| {
                    assert_eq!(record.pid, 42);
                    assert_eq!(bytes, f.bytes);
                    let call = retained_calls.get() + 1;
                    retained_calls.set(call);
                    if mode == "retained-first-refused"
                        || mode == "retained-before-post-refused" && call == 2
                    {
                        Err(CollectError::Unreachable)
                    } else {
                        Ok(())
                    }
                },
            ),
        )
        .await;
        if mode == "slow-stop" {
            assert!(
                observed.is_err(),
                "delayed restoration response must remain timed out"
            );
            assert_eq!(progress.borrow().last(), Some(&"stop-post-client-restore"));
            assert!(!progress.borrow().contains(&"stop-response-verified"));
            assert_eq!(
                requests.lock().unwrap().last().unwrap().0,
                "POST /api/stop HTTP/1.1"
            );
            task.abort();
            let _ = task.await;
            continue;
        }
        let outcome = observed.unwrap();
        task.await.unwrap();
        let requests = requests.lock().unwrap();
        assert_eq!(outcome.is_ok(), mode == "valid", "{mode}");
        if [
            "forged",
            "witness-refused",
            "duplicate-lifetime-proof",
            "retained-first-refused",
        ]
        .contains(&mode)
        {
            assert_eq!(requests.len(), 1);
            assert!(!requests[0].1, "no token before identity proof");
        }
        if [
            "record-changed",
            "capability-pid",
            "retained-before-post-refused",
        ]
        .contains(&mode)
        {
            assert!(!requests.iter().any(|r| r.0.starts_with("POST")));
        }
        if [
            "forged",
            "record-changed",
            "capability-pid",
            "witness-refused",
            "duplicate-lifetime-proof",
            "retained-first-refused",
            "retained-before-post-refused",
        ]
        .contains(&mode)
        {
            assert!(!progress.borrow().contains(&"stop-post-client-restore"));
        }
        assert!(progress
            .borrow()
            .iter()
            .all(|phase| !phase.contains("ocx_admin_")
                && !phase.contains(&f.record.attestation_secret)));

        if mode == "valid" {
            assert_eq!(retained_calls.get(), 2);
            assert!(progress.borrow().contains(&"stop-post-response"));
            assert_eq!(requests.len(), 4);
            assert_eq!(requests[3].0, "POST /api/stop HTTP/1.1");
            assert!(requests[3].1);
        }
    }
}

#[cfg(windows)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn production_stop_without_lifetime_refuses_before_reusable_token_or_post() {
    let (f, task) = server("valid").await;
    let root = installation(&f.home);
    let data = f.home.join("desktop");
    std::fs::create_dir(&data).unwrap();
    let runtime = crate::modules::runtime::RuntimeHandle::initialize_managed(&data, &root);
    let phases = std::cell::RefCell::new(Vec::new());
    let outcome = stop_official(runtime, f.home.clone(), 42, |p| phases.borrow_mut().push(p)).await;
    assert!(matches!(outcome, Err(CollectError::Unreachable)));
    assert_eq!(phases.borrow().last(), Some(&"stop-lifetime-witness"));
    assert!(!phases.borrow().contains(&"stop-private-token"));
    assert!(!phases.borrow().contains(&"stop-post-client-restore"));
    assert!(!f.home.join("admin-api-token").exists());
    task.await.unwrap();
}
#[tokio::test]
async fn independent_version_observation_accepts_old_attested_process_without_admitting_disk_or_executing_cli(
) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let f = Fixture::new(listener.local_addr().unwrap().port());
    let expected = f.record.clone();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut io = BufReader::new(stream);
        for index in 0..2 {
            let mut request = String::new();
            loop {
                let mut line = String::new();
                assert!(io.read_line(&mut line).await.unwrap() > 0);
                request.push_str(&line);
                if line == "\r\n" {
                    break;
                }
            }
            assert!(request.starts_with("GET /healthz HTTP/1.1"));
            assert!(!request.contains("Authorization"));
            let challenge = request
                .lines()
                .find_map(|l| l.strip_prefix("x-opencodex-attestation-challenge: "))
                .unwrap();
            let mut mac =
                Hmac::<Sha256>::new_from_slice(expected.attestation_secret.as_bytes()).unwrap();
            mac.update(
                format!(
                    "opencodex-local-management-v1\n{challenge}\n{}\n{}",
                    expected.pid, expected.port
                )
                .as_bytes(),
            );
            let proof = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
            let body=json!({"service":"opencodex","status":"ok","pid":42,"port":expected.port,"version":"2.98.0","uptime":10+index}).to_string();
            let response=format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: keep-alive\r\nx-opencodex-attestation-proof: {proof}\r\n\r\n{body}",body.len());
            io.get_mut().write_all(response.as_bytes()).await.unwrap();
        }
    });
    let result = running_version(f.home.clone()).await.unwrap().unwrap();
    assert_eq!(result.version, "2.98.0");
    assert_eq!(result.pid, 42);
    assert_eq!(result.port, f.record.port);
    server.await.unwrap();
    assert_eq!(record(&f.home).unwrap().1, f.bytes);
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn public_official024_stop_keeps_native_checks_and_does_not_require_maintenance_fields() {
    use super::official024::{request_stop_official024, RetainedService};
    struct Owner {
        calls: usize,
        refuse: usize,
        digest: String,
        record_path: PathBuf,
    }
    impl RetainedService for Owner {
        fn verify_live(&mut self, pid: u32, port: u16, digest: &str) -> Result<(), CollectError> {
            self.calls += 1;
            assert_eq!(pid, 42);
            assert_ne!(port, 0);
            assert_eq!(digest, self.digest);
            if self.refuse == 3 && self.calls == 2 {
                std::fs::write(&self.record_path, b"outside record during native check").unwrap();
            }
            if self.calls == self.refuse {
                Err(CollectError::Unreachable)
            } else {
                Ok(())
            }
        }
    }
    for refuse in [0, 1, 2, 3] {
        let program = crate::modules::runtime::terminal_cli::tests::Fixture::new();
        let runtime: SharedRuntimeExecutable =
            crate::modules::runtime::RuntimeHandle::initialize_versioned(
                &program.c.desktop_root,
                &program.c.program_root,
            );
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let f = Fixture::new(listener.local_addr().unwrap().port());
        crate::infrastructure::atomic_write::atomic_write(
            &f.home.join("admin-api-token"),
            format!("ocx_admin_{}", "x".repeat(43)).as_bytes(),
            0o600,
        )
        .unwrap();
        let record = f.record.clone();
        let requests = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
        let captured = requests.clone();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut io = BufReader::new(stream);
            for i in 0..4 {
                let mut request = String::new();
                loop {
                    let mut line = String::new();
                    if io.read_line(&mut line).await.unwrap_or(0) == 0 {
                        return;
                    }
                    request.push_str(&line);
                    if line == "\r\n" {
                        break;
                    }
                }
                captured.lock().unwrap().push((
                    request.lines().next().unwrap().to_string(),
                    request.contains("Authorization: Bearer"),
                ));
                let mut proof = String::new();
                let value = if i == 0 || i == 2 {
                    let challenge = request
                        .lines()
                        .find_map(|l| l.strip_prefix("x-opencodex-attestation-challenge: "))
                        .unwrap();
                    let mut mac =
                        Hmac::<Sha256>::new_from_slice(record.attestation_secret.as_bytes())
                            .unwrap();
                    mac.update(
                        format!(
                            "opencodex-local-management-v1\n{challenge}\n{}\n{}",
                            record.pid, record.port
                        )
                        .as_bytes(),
                    );
                    proof = format!(
                        "x-opencodex-attestation-proof: {}\r\n",
                        URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
                    );
                    json!({"service":"opencodex","status":"ok","pid":42,"port":record.port,"version":"2.99.0"})
                } else if i == 1 {
                    json!({"schema":"ocx-stop-capabilities/1","connectedClientOwnership":true,"pid":42})
                } else {
                    json!({"success":true,"sharedTeardown":"performed"})
                };
                let raw = serde_json::to_vec(&value).unwrap();
                let header=format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: keep-alive\r\n{proof}\r\n",raw.len());
                if io.get_mut().write_all(header.as_bytes()).await.is_err() {
                    return;
                }
                if io.get_mut().write_all(&raw).await.is_err() {
                    return;
                }
                io.get_mut().flush().await.unwrap();
            }
        });
        let mut owner = Owner {
            calls: 0,
            refuse,
            digest: crate::infrastructure::hash::sha256_hex(&f.bytes),
            record_path: f.home.join("runtime-port.json"),
        };
        let result = request_stop_official024(
            runtime,
            f.home.clone(),
            42,
            std::time::Instant::now() + std::time::Duration::from_secs(5),
            &mut owner,
            |_| {},
        )
        .await;
        if refuse == 0 {
            let reply = result.unwrap();
            assert_eq!(reply.pid(), 42);
            assert_eq!(reply.port(), f.record.port);
            assert_eq!(reply.record_sha256(), owner.digest);
            assert_eq!(owner.calls, 2);
        } else {
            assert!(result.is_err());
            assert_eq!(owner.calls, refuse.min(2));
            if refuse == 3 {
                assert_eq!(
                    std::fs::read(&owner.record_path).unwrap(),
                    b"outside record during native check"
                );
            }
        }
        server.await.unwrap();
        let sent = requests.lock().unwrap();
        assert_eq!(
            sent.len(),
            match refuse {
                0 => 4,
                1 => 1,
                _ => 3,
            }
        );
        assert_eq!(sent.iter().any(|r| r.0.starts_with("POST")), refuse == 0);
        if refuse == 1 {
            assert!(!sent[0].1);
        }
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn typed_official024_observation_is_fresh_and_does_not_turn_pending_into_ready() {
    use super::official024::observe_official024;
    use std::time::{Duration, Instant};
    for mode in ["official024", "official024-pending"] {
        let program = crate::modules::runtime::terminal_cli::tests::Fixture::new();
        let runtime: SharedRuntimeExecutable =
            crate::modules::runtime::RuntimeHandle::initialize_versioned(
                &program.c.desktop_root,
                &program.c.program_root,
            );
        let (f, task) = server(mode).await;
        let observation = observe_official024(
            runtime.clone(),
            f.home.clone(),
            Instant::now() + Duration::from_secs(5),
        )
        .await
        .unwrap();
        assert_eq!(observation.pid(), 42);
        assert_eq!(observation.port(), f.record.port);
        assert_eq!(observation.ready(), mode == "official024");
        assert_eq!(observation.version(), "2.99.0");
        assert_eq!(
            observation.record_sha256(),
            crate::infrastructure::hash::sha256_hex(&f.bytes)
        );
        observation.check_current(&runtime, &f.home).unwrap();
        let other: SharedRuntimeExecutable =
            crate::modules::runtime::RuntimeHandle::initialize_versioned(
                &program.c.desktop_root,
                &program.c.program_root,
            );
        assert!(observation.check_current(&other, &f.home).is_err());
        assert!(observation
            .check_current(&runtime, &program.c.opencodex_home)
            .is_err());
        std::fs::write(f.home.join("runtime-port.json"), b"outside").unwrap();
        assert!(observation.check_current(&runtime, &f.home).is_err());
        task.await.unwrap();
    }
}
#[tokio::test]
async fn official024_expired_parent_deadline_never_connects_or_calls_native_owner() {
    struct Refuse;
    impl super::official024::RetainedService for Refuse {
        fn verify_live(&mut self, _: u32, _: u16, _: &str) -> Result<(), CollectError> {
            panic!("expired action cannot reach native owner")
        }
    }
    let runtime = crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved();
    let deadline = std::time::Instant::now();
    assert!(matches!(
        super::official024::request_stop_official024(
            runtime.clone(),
            PathBuf::from("/unread"),
            42,
            deadline,
            &mut Refuse,
            |_| {}
        )
        .await,
        Err(CollectError::Timeout)
    ));
    assert!(matches!(
        super::official024::observe_official024(runtime, PathBuf::from("/unread"), deadline).await,
        Err(CollectError::Timeout)
    ));
}
#[tokio::test]
async fn independent_version_read_refuses_wrong_proof_and_missing_record_is_only_no_record() {
    let root = tempfile::tempdir().unwrap();
    assert!(running_version(root.path().canonicalize().unwrap())
        .await
        .unwrap()
        .is_none());
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    let (f, server) = server("key").await;
    assert!(running_version(f.home.clone()).await.is_err());
    server.await.unwrap();
    assert_eq!(record(&f.home).unwrap().1, f.bytes);
}
