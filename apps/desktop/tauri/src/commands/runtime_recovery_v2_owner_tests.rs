use super::*;
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Default)]
struct FakeHost {
    processes: Mutex<BTreeMap<u32, ProcessIdentity>>,
    listener: AtomicBool,
    manager: AtomicBool,
}
impl Host for FakeHost {
    fn owner(&self) -> AppResult<String> {
        Ok("fixture-user".into())
    }
    fn process(&self, pid: u32) -> AppResult<Option<ProcessIdentity>> {
        Ok(self.processes.lock().unwrap().get(&pid).cloned())
    }
    fn processes(&self) -> AppResult<Vec<ProcessIdentity>> {
        Ok(self.processes.lock().unwrap().values().cloned().collect())
    }
    fn port_clear(&self, _: u16) -> AppResult<bool> {
        Ok(!self.listener.load(Ordering::SeqCst))
    }
    fn manager_absent(&self) -> AppResult<bool> {
        Ok(!self.manager.load(Ordering::SeqCst))
    }
}
struct Fixture {
    _root: tempfile::TempDir,
    provider: SystemServiceOwnerProofProvider,
    host: Arc<FakeHost>,
}
impl Fixture {
    fn new() -> Self {
        let root = tempfile::Builder::new()
            .prefix("ocxd023-runtime-recovery-ipc-owner-")
            .tempdir()
            .unwrap();
        let h = root.path().join("H");
        let i = root.path().join("I");
        std::fs::create_dir_all(&h).unwrap();
        std::fs::create_dir_all(&i).unwrap();
        let host = Arc::new(FakeHost::default());
        let context = ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(
            ),
            working_directory: root.path().into(),
            opencodex_home: h,
        };
        let provider = SystemServiceOwnerProofProvider {
            installation_root: path_key(&i),
            context,
            owner: "fixture-user".into(),
            host: host.clone(),
            guard: RecoveryStopGuard::AttestedHandover,
            last_record: Mutex::new(None),
        };
        Self {
            _root: root,
            provider,
            host,
        }
    }
    fn record(&self, pid: u32) {
        std::fs::write(
            self.provider
                .context
                .opencodex_home
                .join("runtime-port.json"),
            serde_json::to_vec(
                &serde_json::json!({"pid":pid,"port":19555,"attestationSecret":"synthetic-only"}),
            )
            .unwrap(),
        )
        .unwrap();
    }
    fn process(&self, pid: u32) -> ProcessIdentity {
        ProcessIdentity {
            pid,
            owner: "fixture-user".into(),
            created: "123:456".into(),
            image: crate::modules::runtime::install::managed_dependency_paths(
                &self.provider.installation_root,
            )
            .0,
            argv: vec![
                "node".into(),
                self.provider
                    .installation_root
                    .join("runtime/opencodex/node_modules/@bitkyc08/opencodex/bin/ocx.mjs")
                    .to_string_lossy()
                    .into_owned(),
                "start".into(),
            ],
            home: Some(self.provider.context.opencodex_home.clone()),
            zombie: false,
        }
    }
    fn query(&self) -> ServiceOwnerProof {
        self.provider.query(&self.provider.context).unwrap()
    }
}

#[test]
fn positive_absence_requires_runtime_scan_endpoint_and_service_evidence() {
    let f = Fixture::new();
    assert_eq!(
        f.query(),
        ServiceOwnerProof::Stopped,
        "no records plus OS process and manager absence is positive evidence"
    );
    f.record(4242);
    assert_eq!(
        f.query(),
        ServiceOwnerProof::Stopped,
        "old pid is gone and its port/process/manager observations are absent"
    );
    f.host.listener.store(true, Ordering::SeqCst);
    assert_eq!(
        f.query(),
        ServiceOwnerProof::Unknown,
        "a foreign listener is not ours to evict"
    );
    f.host.listener.store(false, Ordering::SeqCst);
    f.host.manager.store(true, Ordering::SeqCst);
    assert_eq!(
        f.query(),
        ServiceOwnerProof::Unknown,
        "a manager may respawn this H"
    );
    f.host.manager.store(false, Ordering::SeqCst);
    f.host
        .processes
        .lock()
        .unwrap()
        .insert(5353, f.process(5353));
    assert_eq!(
        f.query(),
        ServiceOwnerProof::Unknown,
        "unrecorded current H runtime still blocks absence"
    );
    f.host.processes.lock().unwrap().clear();
    std::fs::remove_file(f.provider.context.opencodex_home.join("runtime-port.json")).unwrap();
    std::fs::write(f.provider.context.opencodex_home.join("ocx.pid"), b"4242").unwrap();
    assert_eq!(
        f.query(),
        ServiceOwnerProof::Unknown,
        "residual pid without an endpoint is not Stopped"
    );
}

#[test]
fn identity_binds_user_creation_executable_home_record_and_context() {
    let f = Fixture::new();
    f.record(4242);
    let original = f.process(4242);
    f.host
        .processes
        .lock()
        .unwrap()
        .insert(4242, original.clone());
    let ServiceOwnerProof::Owned { identity, pid } = f.query() else {
        panic!("expected owned fixture");
    };
    for changed in [
        ProcessIdentity {
            owner: "foreign-user".into(),
            ..original.clone()
        },
        ProcessIdentity {
            image: PathBuf::from("/untrusted/bun"),
            ..original.clone()
        },
        ProcessIdentity {
            home: Some(PathBuf::from("/other/H")),
            ..original.clone()
        },
        ProcessIdentity {
            zombie: true,
            ..original.clone()
        },
        ProcessIdentity {
            home: None,
            ..original.clone()
        },
        ProcessIdentity {
            argv: vec!["node".into(), "-e".into(), "unrelated script".into()],
            ..original.clone()
        },
    ] {
        f.host.processes.lock().unwrap().insert(4242, changed);
        assert_eq!(f.query(), ServiceOwnerProof::Unknown);
        assert!(f
            .provider
            .stop_owned(&f.provider.context, &identity, pid)
            .is_err());
    }
    f.host.processes.lock().unwrap().insert(
        4242,
        ProcessIdentity {
            created: "new-creation".into(),
            ..original.clone()
        },
    );
    assert!(matches!(f.query(), ServiceOwnerProof::Owned {identity: new, ..} if new != identity));
    assert!(f
        .provider
        .stop_owned(&f.provider.context, &identity, pid)
        .is_err());
    f.host.processes.lock().unwrap().insert(4242, original);
    let mut other_context = f.provider.context.clone();
    other_context.runtime =
        crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved();
    assert!(f.provider.query(&other_context).is_err());
    let mut provider = f.provider;
    provider.guard = RecoveryStopGuard::ObserveOnly;
    assert!(
        matches!(provider.stop_owned(&provider.context, &identity, pid), Err(AppError::RuntimeManaged {code, ..}) if code == "recovery_stop_guard")
    );
    provider.guard = RecoveryStopGuard::AttestedHandover;
    f.host.manager.store(true, Ordering::SeqCst);
    assert!(
        matches!(provider.stop_owned(&provider.context, &identity, pid), Err(AppError::RuntimeManaged {code, ..}) if code == "recovery_service_supervised")
    );
    assert!(!std::fs::read_dir(&provider.context.opencodex_home)
        .unwrap()
        .any(|e| e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("pending-teardown-")));
}

#[test]
fn malformed_or_foreign_records_are_never_absence_and_receipts_only_delete_owned_unsent_file() {
    let f = Fixture::new();
    let h = &f.provider.context.opencodex_home;
    std::fs::write(h.join("runtime-port.json"), b"invalid").unwrap();
    assert!(f.provider.query(&f.provider.context).is_err());
    f.record(4242);
    std::fs::write(
        h.join("service-state.json"),
        br#"{"version":2,"opencodexHome":"/foreign/H"}"#,
    )
    .unwrap();
    assert_eq!(f.query(), ServiceOwnerProof::Unknown);
    std::fs::remove_file(h.join("service-state.json")).unwrap();
    let record = f.provider.record().unwrap().unwrap();
    let other = h.join("pending-teardown-00000000000000000000000000000000.json");
    std::fs::write(&other, b"do not overwrite other owner").unwrap();
    let unsent = HandoverReceipt::create(h, &record).unwrap();
    let path = unsent.path.clone();
    drop(unsent);
    assert!(!path.exists());
    let mut sent = HandoverReceipt::create(h, &record).unwrap();
    let path = sent.path.clone();
    sent.sent = true;
    drop(sent);
    assert!(
        path.exists(),
        "an accepted/ambiguous stop keeps its real handover obligation"
    );
    assert_eq!(
        std::fs::read(other).unwrap(),
        b"do not overwrite other owner"
    );
}

#[cfg(target_os = "macos")]
#[test]
fn kernel_argument_parser_handles_padding_spaces_and_rejects_ambiguous_home() {
    let mut bytes = 3i32.to_ne_bytes().to_vec();
    bytes.extend_from_slice(b"/node\0\0\0node\0/with spaces/ocx.mjs\0start\0OPENCODEX_HOME=/tmp/fixture H\0SECRET=never return this\0\0");
    let (argv, home) = native::parse_procargs(&bytes).unwrap();
    assert_eq!(argv, ["node", "/with spaces/ocx.mjs", "start"]);
    assert_eq!(home, Some(PathBuf::from("/tmp/fixture H")));
    bytes.extend_from_slice(b"OPENCODEX_HOME=/other\0");
    assert!(native::parse_procargs(&bytes).is_err());
    for bytes in [
        vec![],
        100i32.to_ne_bytes().to_vec(),
        [1i32.to_ne_bytes().as_slice(), b"/node\0\0"].concat(),
    ] {
        assert!(native::parse_procargs(&bytes).is_err());
    }
}

#[cfg(any(target_os = "macos", windows))]
#[test]
fn actual_os_reports_current_identity_and_listener_ownership_without_service_mutation() {
    let host = PlatformHost;
    let process = host.process(std::process::id()).unwrap().unwrap();
    assert_eq!(process.owner, host.owner().unwrap());
    assert!(!process.created.is_empty());
    assert_eq!(
        path_key(&process.image),
        path_key(&std::env::current_exe().unwrap())
    );
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    assert!(!host.port_clear(port).unwrap());
    drop(listener);
    assert!(host.port_clear(port).unwrap());
}

#[cfg(any(target_os = "macos", windows))]
mod local_process {
    use super::*;
    use std::{io::Write, sync::mpsc};
    // A dedicated synthetic service implements the actual attestation/stop
    // protocol. Its port, H and installed executable are exclusively test-owned.
    const SERVICE: &str = r#"
import fs from 'node:fs';
import http from 'node:http';
import crypto from 'node:crypto';
import path from 'node:path';
const h=process.env.OPENCODEX_HOME;
const secret='s'.repeat(43), token='ocx_admin_'+'t'.repeat(43);
let port=0;
const server=http.createServer((req,res)=>{
  const url=new URL(req.url,'http://127.0.0.1');
  res.setHeader('content-type','application/json');
  if(url.pathname==='/healthz'){
    const challenge=req.headers['x-opencodex-attestation-challenge'];
    if(challenge)res.setHeader('x-opencodex-attestation-proof',crypto.createHmac('sha256',fs.existsSync(path.join(h,'bad-attestation'))?'foreign-proof':secret).update('opencodex-local-management-v1\n'+challenge+'\n'+process.pid+'\n'+port).digest('base64url'));
    res.end(JSON.stringify({service:'opencodex',pid:process.pid,version:fs.existsSync(path.join(h,'unsupported-version'))?'0.0.0-unsupported':'2.97.0-desktop023.names.1'}));return;
  }
  if(req.headers.authorization!=='Bearer '+token){res.statusCode=401;res.end('{}');return;}
  if(url.pathname==='/api/stop'&&req.method==='GET'){
    res.end(JSON.stringify({schema:'ocx-stop-capabilities/1',pid:process.pid,connectedClientOwnership:true}));return;
  }
  if(url.pathname==='/api/stop'&&req.method==='POST'){
    fs.appendFileSync(path.join(h,'post-count'),'1');
    if(fs.existsSync(path.join(h,'refuse'))){res.statusCode=409;res.end('{}');return;}
    const nonce=url.searchParams.get('teardownNonce');
    if(!/^[a-f0-9]{32}$/.test(nonce)||url.searchParams.get('deferSharedTeardown')!=='1')throw Error('no owned handover');
    const receipt=JSON.parse(fs.readFileSync(path.join(h,'pending-teardown-'+nonce+'.json')));
    if(receipt.nonce!==nonce||receipt.handover!=='update-restart'||receipt.endpoint.port!==port||receipt.endpointSource!=='exact')throw Error('bad handover receipt');
    res.end(JSON.stringify({success:true,sharedTeardown:'deferred'}));
    setTimeout(()=>{
      fs.unlinkSync(path.join(h,'runtime-port.json'));fs.unlinkSync(path.join(h,'ocx.pid'));
      server.close(()=>process.exit(0));
    },100);return;
  }
  res.statusCode=404;res.end('{}');
});
server.listen(0,'127.0.0.1',()=>{
  port=server.address().port;
  fs.writeFileSync(path.join(h,'admin-api-token'),token);
  fs.writeFileSync(path.join(h,'ocx.pid'),String(process.pid));
  fs.writeFileSync(path.join(h,'runtime-port.json'),JSON.stringify({pid:process.pid,port,hostname:'127.0.0.1',attestationSecret:secret}));
});
setTimeout(()=>process.exit(77),45000).unref();
"#;
    struct ManagerAbsent;
    impl Host for ManagerAbsent {
        fn owner(&self) -> AppResult<String> {
            PlatformHost.owner()
        }
        fn process(&self, pid: u32) -> AppResult<Option<ProcessIdentity>> {
            PlatformHost.process(pid)
        }
        fn processes(&self) -> AppResult<Vec<ProcessIdentity>> {
            PlatformHost.processes()
        }
        fn port_clear(&self, port: u16) -> AppResult<bool> {
            PlatformHost.port_clear(port)
        }
        // Do not couple a test's stopping authority to the user's real service
        // registration. Only the test namespace has no service-manager job.
        fn manager_absent(&self) -> AppResult<bool> {
            Ok(true)
        }
    }
    struct OwnedChild {
        cancel: mpsc::Sender<()>,
        join: Option<std::thread::JoinHandle<std::process::ExitStatus>>,
    }
    impl OwnedChild {
        fn new(mut child: std::process::Child) -> Self {
            let (cancel, end) = mpsc::channel();
            let join = std::thread::spawn(move || loop {
                if let Some(exit) = child.try_wait().unwrap() {
                    return exit;
                }
                if end.try_recv().is_ok() {
                    let _ = child.kill();
                    return child.wait().unwrap();
                }
                std::thread::sleep(Duration::from_millis(10));
            });
            Self {
                cancel,
                join: Some(join),
            }
        }
    }
    impl Drop for OwnedChild {
        fn drop(&mut self) {
            let _ = self.cancel.send(());
            if let Some(join) = self.join.take() {
                let _ = join.join();
            }
        }
    }
    #[test]
    #[ignore = "dedicated OS process + attested HTTP fixture: set absolute OPENCODEX_TEST_NODE; run --ignored"]
    fn actual_process_uid_creation_h_attestation_and_preserving_stop() {
        let root = tempfile::Builder::new()
            .prefix("ocxd023-runtime-recovery-ipc-owner-native-")
            .tempdir()
            .unwrap();
        let i = root.path().join("I");
        let h = root.path().join("H");
        let os_home = root.path().join("os-home");
        std::fs::create_dir_all(&h).unwrap();
        std::fs::create_dir_all(&os_home).unwrap();
        let config = br#"{"fixtureOnly":true,"apiKey":"synthetic-never-daily"}"#;
        std::fs::write(h.join("config.json"), config).unwrap();
        std::fs::write(h.join("keychain.json"), b"synthetic kept key bytes").unwrap();
        let (node, _) = crate::modules::runtime::install::managed_dependency_paths(&i);
        std::fs::create_dir_all(node.parent().unwrap()).unwrap();
        let input = PathBuf::from(
            std::env::var_os("OPENCODEX_TEST_NODE").expect("explicit test Node required"),
        );
        assert!(input.is_absolute());
        std::fs::copy(std::fs::canonicalize(input).unwrap(), &node).unwrap();
        let script = i.join("runtime/opencodex/node_modules/@bitkyc08/opencodex/bin/ocx.mjs");
        std::fs::create_dir_all(script.parent().unwrap()).unwrap();
        std::fs::write(&script, SERVICE).unwrap();
        let context = ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(
            ),
            working_directory: root.path().into(),
            opencodex_home: h.clone(),
        };
        let mut provider = SystemServiceOwnerProofProvider::new(i, context.clone()).unwrap();
        provider.host = Arc::new(ManagerAbsent);
        assert_eq!(provider.query(&context).unwrap(), ServiceOwnerProof::Stopped, "missing records only pass after actual OS enumeration in an absent test manager namespace");
        let mut command = std::process::Command::new(&node);
        command
            .arg(&script)
            .env_clear()
            .env("HOME", &os_home)
            .env("OPENCODEX_HOME", &h)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        #[cfg(windows)]
        {
            let system = native::system_directory().unwrap();
            command
                .env("SystemRoot", system.parent().unwrap())
                .env("PATH", &system);
        }
        let child = command.spawn().unwrap();
        let pid = child.id();
        let mut child = OwnedChild::new(child);
        let deadline = Instant::now() + Duration::from_secs(10);
        while !h.join("runtime-port.json").exists() {
            assert!(
                Instant::now() < deadline,
                "dedicated test service did not start"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
        let ServiceOwnerProof::Owned {
            identity,
            pid: actual,
        } = provider.query(&context).unwrap()
        else {
            panic!("OS owner query did not prove the synthetic service");
        };
        assert_eq!(actual, pid);
        let process = PlatformHost.process(pid).unwrap().unwrap();
        assert_eq!(path_key(process.home.as_deref().unwrap()), path_key(&h));
        assert_eq!(process.owner, PlatformHost.owner().unwrap());
        assert!(!process.created.is_empty());
        assert!(provider.stop_owned(&context, &"0".repeat(64), pid).is_err());
        assert!(
            !h.join("post-count").exists(),
            "wrong creation identity must never send stop"
        );
        std::fs::write(h.join("bad-attestation"), b"foreign proof fixture").unwrap();
        assert!(provider.stop_owned(&context, &identity, pid).is_err());
        assert!(!h.join("post-count").exists());
        std::fs::remove_file(h.join("bad-attestation")).unwrap();
        std::fs::write(
            h.join("unsupported-version"),
            b"unsupported stop preservation fixture",
        )
        .unwrap();
        assert!(
            matches!(provider.stop_owned(&context, &identity, pid), Err(AppError::RuntimeManaged {code,..}) if code == "recovery_stop_capability")
        );
        assert!(!h.join("post-count").exists());
        std::fs::remove_file(h.join("unsupported-version")).unwrap();
        std::fs::write(h.join("refuse"), b"test refusal").unwrap();
        assert!(
            matches!(provider.stop_owned(&context, &identity, pid), Err(AppError::RuntimeManaged {code,..}) if code == "recovery_stop_refused")
        );
        assert!(matches!(
            provider.query(&context).unwrap(),
            ServiceOwnerProof::Owned { .. }
        ));
        assert!(
            !std::fs::read_dir(&h).unwrap().any(|e| e
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("pending-teardown-")),
            "explicit rejection cleans only the unsent/unaccepted claim"
        );
        std::fs::remove_file(h.join("refuse")).unwrap();
        provider.stop_owned(&context, &identity, pid).unwrap();
        assert!(child.join.take().unwrap().join().unwrap().success());
        assert_eq!(
            provider.query(&context).unwrap(),
            ServiceOwnerProof::Stopped
        );
        assert_eq!(std::fs::read(h.join("post-count")).unwrap(), b"11");
        assert_eq!(std::fs::read(h.join("config.json")).unwrap(), config);
        assert_eq!(
            std::fs::read(h.join("keychain.json")).unwrap(),
            b"synthetic kept key bytes"
        );
        assert_eq!(
            std::fs::read_dir(&h)
                .unwrap()
                .filter(|e| e
                    .as_ref()
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with("pending-teardown-"))
                .count(),
            1
        );
        let log = PathBuf::from(format!(
            "/tmp/ocxd023-runtime-recovery-ipc-owner-native-{}.log",
            uuid::Uuid::new_v4()
        ));
        let mut file = std::fs::File::create(&log).unwrap();
        writeln!(file, "dedicated synthetic Node/H only; actual OS user/creation/image/argv/environment + Management HMAC attestation; stale identity sent no stop; explicit 409 kept process and cleaned own unaccepted receipt; accepted handover self-stopped and reaped; old TCP port absent; OS runtime scan proved Stopped; config/keys exact; one durable replacement-owned handover retained").unwrap();
        eprintln!("recovery OS owner evidence: {}", log.display());
    }
}
