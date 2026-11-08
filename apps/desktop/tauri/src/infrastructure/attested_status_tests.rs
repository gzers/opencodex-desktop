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
            let mut status = 200;
            if mode == "pending" && index == 1 {
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
