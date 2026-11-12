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
    blocked_ports: Mutex<Vec<u16>>,
    queried_ports: Mutex<Vec<u16>>,
    port_error: AtomicBool,
    owner_override: Mutex<Option<String>>,
}
impl Host for FakeHost {
    fn owner(&self) -> AppResult<String> {
        Ok(self
            .owner_override
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| "fixture-user".into()))
    }
    fn process(&self, pid: u32) -> AppResult<Option<ProcessIdentity>> {
        Ok(self.processes.lock().unwrap().get(&pid).cloned())
    }
    fn processes(&self) -> AppResult<Vec<ProcessIdentity>> {
        Ok(self.processes.lock().unwrap().values().cloned().collect())
    }
    fn port_clear(&self, port: u16) -> AppResult<bool> {
        self.queried_ports.lock().unwrap().push(port);
        if self.port_error.load(Ordering::SeqCst) {
            return Err(AppError::NotConfigured);
        }
        Ok(!self.listener.load(Ordering::SeqCst)
            && !self.blocked_ports.lock().unwrap().contains(&port))
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

#[cfg(target_os = "macos")]
#[test]
fn excluded_recovery_step_refuses_any_record_process_endpoint_manager_or_context_change() {
    struct CurrentUserHost(Arc<FakeHost>);
    impl Host for CurrentUserHost {
        fn owner(&self) -> AppResult<String> {
            Ok(unsafe { libc::geteuid() }.to_string())
        }
        fn process(&self, pid: u32) -> AppResult<Option<ProcessIdentity>> {
            self.0.process(pid)
        }
        fn processes(&self) -> AppResult<Vec<ProcessIdentity>> {
            panic!("per-step check must not claim full batch enumeration")
        }
        fn port_clear(&self, port: u16) -> AppResult<bool> {
            self.0.port_clear(port)
        }
        fn manager_absent(&self) -> AppResult<bool> {
            self.0.manager_absent()
        }
    }
    for mode in [
        "clear",
        "record",
        "pid",
        "process",
        "launcher",
        "port",
        "manager",
        "unknown-port",
        "context",
        "identity",
    ] {
        let mut f = Fixture::new();
        f.provider.owner = unsafe { libc::geteuid() }.to_string();
        f.provider.host = Arc::new(CurrentUserHost(f.host.clone()));
        let service = crate::infrastructure::macos_process_exit::Generation {
            pid: 4242,
            uid: unsafe { libc::geteuid() },
            start_seconds: 1,
            start_micros: 1,
        };
        let launcher = crate::infrastructure::macos_process_exit::Generation {
            pid: 4241,
            ..service
        };
        let mut context = f.provider.context.clone();
        let mut port = 19555;
        match mode {
            "record" => f.record(4242),
            "pid" => std::fs::write(context.opencodex_home.join("ocx.pid"), b"4242").unwrap(),
            "process" | "launcher" => {
                let pid = if mode == "process" { 4242 } else { 4241 };
                f.host.processes.lock().unwrap().insert(pid, f.process(pid));
            }
            "port" => f.host.listener.store(true, Ordering::SeqCst),
            "manager" => f.host.manager.store(true, Ordering::SeqCst),
            "unknown-port" => f.host.port_error.store(true, Ordering::SeqCst),
            "context" => context.opencodex_home = context.opencodex_home.join("other"),
            "identity" => port = 0,
            _ => {}
        }
        assert_eq!(
            f.provider
                .require_excluded_recovery_absence(&context, service, launcher, port)
                .is_ok(),
            mode == "clear",
            "{mode}"
        );
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

#[cfg(target_os = "macos")]
mod completed_stop_binding {
    use super::*;
    use crate::infrastructure::macos_process_exit::{ExitWatch, Generation, NormalExitProof};
    use std::{
        io::{Read, Write},
        process::{Command, Stdio},
        time::Duration,
    };
    #[test]
    fn captured_listener_exit_child() {
        let Some(marker) = std::env::var_os("OCXD_ROOT_STOP_LISTENER_MARKER") else {
            return;
        };
        let one = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let two = std::net::TcpListener::bind("[::1]:0").unwrap();
        std::fs::write(
            marker,
            serde_json::to_vec(&[
                one.local_addr().unwrap().port(),
                two.local_addr().unwrap().port(),
            ])
            .unwrap(),
        )
        .unwrap();
        let mut byte = [0u8; 1];
        std::io::stdin().read_exact(&mut byte).unwrap();
        assert_eq!(byte, [1]);
        drop((one, two));
    }
    #[test]
    fn root_stop_current_absence_rechecks_secondary_listener_and_refuses_unknown_or_new_owner() {
        let mut f = Fixture::new();
        let owner = unsafe { libc::geteuid() }.to_string();
        *f.host.owner_override.lock().unwrap() = Some(owner.clone());
        f.provider.owner = owner;
        let marker = f._root.path().join("owned-listener-ports.json");
        let mut child=Command::new(std::env::current_exe().unwrap()).args(["--exact","commands::runtime_recovery_v2_owner::tests::completed_stop_binding::captured_listener_exit_child","--nocapture","--test-threads=1"]).env("OCXD_ROOT_STOP_LISTENER_MARKER",&marker).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::piped()).spawn().unwrap();
        let until = Instant::now() + Duration::from_secs(10);
        while !marker.exists() {
            assert!(child.try_wait().unwrap().is_none());
            assert!(Instant::now() < until);
            std::thread::sleep(Duration::from_millis(2));
        }
        let ports: Vec<u16> = serde_json::from_slice(&std::fs::read(marker).unwrap()).unwrap();
        let service = Generation::observe(child.id()).unwrap();
        let mut watch = ExitWatch::capture(service).unwrap();
        let listeners =
            crate::infrastructure::macos_tcp_listeners::TcpListeners::capture(service, until)
                .unwrap();
        assert!(ports.iter().all(|p| listeners.ports().contains(p)));
        child.stdin.take().unwrap().write_all(&[1]).unwrap();
        let normal = loop {
            if let Some(proof) = watch.normal_exit(until).unwrap() {
                break proof;
            }
            std::thread::sleep(Duration::from_millis(2));
        };
        assert!(child.wait_with_output().unwrap().status.success());
        let (launcher, _, _) = normal_exit();
        // This wrapper is an explicit reference-predicate fixture; it does NOT
        // enter the production root permission factory or certify an official
        // runtime. Exit and listeners above are actual kernel evidence.
        let proof = CompletedLifecycleStop {
            context: f.provider.context.clone(),
            data_root: path_key(f._root.path()),
            record_digest: Some("a".repeat(64)),
            deadline: until,
            normal_exit: Some(normal),
            tcp_listeners: Some(listeners),
        };
        let require = || {
            proof.require_original_absence(
                f._root.path(),
                service,
                launcher,
                &"a".repeat(64),
                ports[0],
                &f.provider,
            )
        };
        require().unwrap();
        f.host.blocked_ports.lock().unwrap().push(ports[1]);
        assert!(
            matches!(require(),Err(AppError::RuntimeManaged{code,..}) if code=="client_stop_listener_reappeared")
        );
        assert!(f.host.queried_ports.lock().unwrap().contains(&ports[1]));
        f.host.blocked_ports.lock().unwrap().clear();
        f.host.port_error.store(true, Ordering::SeqCst);
        assert!(require().is_err());
        f.host.port_error.store(false, Ordering::SeqCst);
        f.host.manager.store(true, Ordering::SeqCst);
        assert!(require().is_err());
        f.host.manager.store(false, Ordering::SeqCst);
        std::fs::write(f.provider.context.opencodex_home.join("ocx.pid"), b"99999").unwrap();
        assert!(require().is_err());
        std::fs::remove_file(f.provider.context.opencodex_home.join("ocx.pid")).unwrap();
        require().unwrap();
        *f.host.owner_override.lock().unwrap() = Some("outside-owner".into());
        assert!(require().is_err());
        println!(
            "ROOT_STOP_CURRENT_ABSENCE_EVIDENCE {}",
            serde_json::json!({"accepted":false,"kernelServiceNormalExit":true,"kernelCapturedTcpPorts":ports.len(),"secondaryPortRefused":true,"unknownPortRefused":true,"managerAndRecordReappearanceRefused":true,"ownerChangeRefused":true,"syntheticReferenceWrapperNotOfficialStop":true})
        );
    }

    fn normal_exit() -> (
        Generation,
        NormalExitProof,
        crate::infrastructure::macos_tcp_listeners::TcpListeners,
    ) {
        let mut child = Command::new("/bin/sh")
            .args(["-c", "printf R; read owned_fixture; exit 0"])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut ready = [0];
        child
            .stdout
            .as_mut()
            .unwrap()
            .read_exact(&mut ready)
            .unwrap();
        assert_eq!(&ready, b"R");
        let generation = Generation::observe(child.id()).unwrap();
        let mut watch = ExitWatch::capture(generation).unwrap();
        let listeners = crate::infrastructure::macos_tcp_listeners::TcpListeners::capture(
            generation,
            Instant::now() + Duration::from_secs(2),
        )
        .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(b"continue\n")
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        let proof = loop {
            if let Some(proof) = watch.normal_exit(deadline).unwrap() {
                break proof;
            }
            std::thread::sleep(Duration::from_millis(2));
        };
        assert!(child.wait().unwrap().success());
        (generation, proof, listeners)
    }
    #[test]
    fn preexisting_stopped_observation_has_no_fresh_exit_or_compensation_authority() {
        let f = Fixture::new();
        let result = f
            .provider
            .stop_lifecycle_verified(
                &f.provider.context,
                Instant::now() + Duration::from_secs(1),
                f._root.path(),
            )
            .unwrap();
        result
            .require_context(&f.provider.context, f._root.path())
            .unwrap();
        assert!(result.record_digest.is_none() && result.normal_exit.is_none());
        assert!(result
            .require_original(
                &f.provider.context,
                f._root.path(),
                Generation {
                    pid: 42,
                    uid: unsafe { libc::geteuid() },
                    start_seconds: 1,
                    start_micros: 1
                },
                &"a".repeat(64)
            )
            .is_err());
    }
    #[test]
    fn fixed_internal_reference_check_binds_runtime_handle_profile_home_digest_and_generation() {
        // Test only the reference comparison with a real kernel child-exit
        // proof. The synthetic outer struct here is NOT a service-owned Stop
        // certificate and never goes through a restoration permit factory.
        let f = Fixture::new();
        let context = &f.provider.context;
        let (generation, normal, listeners) = normal_exit();
        let proof = CompletedLifecycleStop {
            deadline: Instant::now() + Duration::from_secs(1),
            context: context.clone(),
            data_root: path_key(f._root.path()),
            record_digest: Some("a".repeat(64)),
            normal_exit: Some(normal),
            tcp_listeners: Some(listeners),
        };
        proof
            .require_original(context, f._root.path(), generation, &"a".repeat(64))
            .unwrap();
        let no_listeners = CompletedLifecycleStop {
            context: context.clone(),
            data_root: path_key(f._root.path()),
            record_digest: Some("a".repeat(64)),
            normal_exit: proof.normal_exit.clone(),
            tcp_listeners: None,
            deadline: proof.deadline,
        };
        assert!(
            no_listeners
                .require_original(context, f._root.path(), generation, &"a".repeat(64))
                .is_err(),
            "exit alone must not replace missing native listener evidence"
        );
        assert!(proof
            .require_original(
                context,
                &f._root.path().join("another-desktop-data-root"),
                generation,
                &"a".repeat(64)
            )
            .is_err());
        assert!(proof
            .require_original(context, f._root.path(), generation, &"b".repeat(64))
            .is_err());
        assert!(proof
            .require_original(
                context,
                f._root.path(),
                Generation {
                    start_seconds: generation.start_seconds + 1,
                    ..generation
                },
                &"a".repeat(64)
            )
            .is_err());
        let mut other = context.clone();
        other.working_directory = f._root.path().join("another-profile");
        assert!(proof
            .require_original(&other, f._root.path(), generation, &"a".repeat(64))
            .is_err());
        let mut other = context.clone();
        other.opencodex_home = f._root.path().join("another-home");
        assert!(proof
            .require_original(&other, f._root.path(), generation, &"a".repeat(64))
            .is_err());
        let mut other = context.clone();
        other.runtime =
            crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved();
        assert!(proof
            .require_original(&other, f._root.path(), generation, &"a".repeat(64))
            .is_err());
    }
    #[test]
    fn malformed_record_digest_and_a_record_without_native_exit_cannot_authorize_restore() {
        let f = Fixture::new();
        let context = &f.provider.context;
        let (generation, normal, listeners) = normal_exit();
        let proof = CompletedLifecycleStop {
            deadline: Instant::now() + Duration::from_secs(1),
            context: context.clone(),
            data_root: path_key(f._root.path()),
            record_digest: Some("not-a-runtime-record-sha256".into()),
            normal_exit: Some(normal),
            tcp_listeners: Some(listeners),
        };
        assert!(proof.require_context(context, f._root.path()).is_err());
        let no_exit = CompletedLifecycleStop {
            deadline: Instant::now() + Duration::from_secs(1),
            context: context.clone(),
            data_root: path_key(f._root.path()),
            record_digest: Some("a".repeat(64)),
            normal_exit: None,
            tcp_listeners: None,
        };
        assert!(no_exit
            .require_original(context, f._root.path(), generation, &"a".repeat(64))
            .is_err());
    }
    #[test]
    fn completed_stop_reference_never_gets_a_fresh_budget_for_subsequent_work() {
        let f = Fixture::new();
        let (generation, normal, listeners) = normal_exit();
        let proof = CompletedLifecycleStop {
            deadline: Instant::now(),
            context: f.provider.context.clone(),
            data_root: path_key(f._root.path()),
            record_digest: Some("a".repeat(64)),
            normal_exit: Some(normal),
            tcp_listeners: Some(listeners),
        };
        assert_eq!(
            proof.bound_deadline(Instant::now() + Duration::from_secs(35)),
            proof.deadline
        );
        let earlier = proof.deadline - Duration::from_millis(1);
        assert_eq!(proof.bound_deadline(earlier), earlier);
        assert!(
            matches!(proof.require_original(&f.provider.context,f._root.path(),generation,&"a".repeat(64)),Err(AppError::RuntimeManaged{code,..}) if code=="lifecycle_timeout")
        );
    }
    #[test]
    fn secondary_captured_listener_or_unknown_socket_table_blocks_complete_stop_predicate() {
        let one = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let two = std::net::TcpListener::bind("[::1]:0").unwrap();
        let expected = [
            one.local_addr().unwrap().port(),
            two.local_addr().unwrap().port(),
        ];
        let proof = crate::infrastructure::macos_tcp_listeners::TcpListeners::capture(
            Generation::observe(std::process::id()).unwrap(),
            Instant::now() + Duration::from_secs(2),
        )
        .unwrap();
        assert!(expected.iter().all(|port| proof.ports().contains(port)));
        let secondary = *proof.ports().last().unwrap();
        let primary = proof.ports()[0];
        assert_ne!(secondary, primary);
        let f = Fixture::new();
        f.host.blocked_ports.lock().unwrap().push(secondary);
        assert!(!f.provider.captured_tcp_clear(&proof).unwrap());
        let queries = f.host.queried_ports.lock().unwrap().clone();
        assert!(queries.contains(&primary) && queries.contains(&secondary));
        f.host.blocked_ports.lock().unwrap().clear();
        assert!(f.provider.captured_tcp_clear(&proof).unwrap());
        f.host.port_error.store(true, Ordering::SeqCst);
        assert!(f.provider.captured_tcp_clear(&proof).is_err());
    }
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
fn actual_bun_cli_child_is_owned_only_at_exact_managed_entry_home_and_owner() {
    let f = Fixture::new();
    f.record(4242);
    let mut process = f.process(4242);
    process.image = f
        .provider
        .installation_root
        .join("runtime/opencodex/node_modules/bun/bin/bun.exe");
    process.argv = vec![
        process.image.to_string_lossy().into_owned(),
        f.provider
            .installation_root
            .join("runtime/opencodex/node_modules/@bitkyc08/opencodex/src/cli/index.ts")
            .to_string_lossy()
            .into_owned(),
        "start".into(),
    ];
    f.host
        .processes
        .lock()
        .unwrap()
        .insert(4242, process.clone());
    assert!(matches!(
        f.query(),
        ServiceOwnerProof::Owned { pid: 4242, .. }
    ));
    for arg in [
        "src/cli/index.ts".into(),
        f.provider
            .installation_root
            .join("other/src/cli/index.ts")
            .to_string_lossy()
            .into_owned(),
    ] {
        let mut changed = process.clone();
        changed.argv[1] = arg;
        f.host.processes.lock().unwrap().insert(4242, changed);
        assert_eq!(f.query(), ServiceOwnerProof::Unknown);
    }
    let mut changed = process;
    changed.home = Some(f.provider.context.working_directory.join("foreign-H"));
    f.host.processes.lock().unwrap().insert(4242, changed);
    assert_eq!(f.query(), ServiceOwnerProof::Unknown);
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
    res.end(JSON.stringify({service:'opencodex',pid:process.pid,version:fs.existsSync(path.join(h,'unsupported-version'))?'0.0.0-unsupported':'2.97.0-desktop023.names.2'}));return;
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

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn versioned_service_owner_uses_selected_image_and_rejects_flat_paths_or_selection_drift() {
    use crate::modules::runtime::{
        program_selection::tests::Fixture as ProgramFixture, RuntimeHandle,
    };
    let program = ProgramFixture::new();
    let mut f = Fixture::new();
    let data = f._root.path().join("desktop");
    std::fs::create_dir(&data).unwrap();
    let runtime = RuntimeHandle::initialize_versioned(&data, &program.root);
    let selected = runtime.selected_program().unwrap().unwrap();
    f.provider.installation_root = program.root.clone();
    f.provider.context.runtime = runtime;
    f.record(4242);
    let mut process = f.process(4242);
    process.image = selected.paths.node.clone();
    process.argv[1] = selected.script.to_string_lossy().into_owned();
    f.host
        .processes
        .lock()
        .unwrap()
        .insert(4242, process.clone());
    assert!(matches!(
        f.query(),
        ServiceOwnerProof::Owned { pid: 4242, .. }
    ));
    let mut flat = process.clone();
    flat.image = program.root.join("runtime/node/bin/node");
    f.host.processes.lock().unwrap().insert(4242, flat);
    assert_eq!(f.query(), ServiceOwnerProof::Unknown);
    process.image = selected
        .paths
        .package_root
        .join("node_modules/bun/bin/bun.exe");
    process.argv[1] = selected
        .paths
        .package_root
        .join("node_modules/@bitkyc08/opencodex/src/cli/index.ts")
        .to_string_lossy()
        .into_owned();
    f.host.processes.lock().unwrap().insert(4242, process);
    assert!(matches!(
        f.query(),
        ServiceOwnerProof::Owned { pid: 4242, .. }
    ));
    std::fs::write(
        program.root.join("runtime/current.json"),
        b"outside selected image",
    )
    .unwrap();
    assert_eq!(f.query(), ServiceOwnerProof::Unknown);
}

#[cfg(target_os = "macos")]
#[test]
fn removal_proof_rejects_another_home_using_this_program_and_preserves_unrelated_processes() {
    let f = Fixture::new();
    assert!(f
        .provider
        .require_program_unused(&f.provider.context)
        .is_ok());
    let mut process = f.process(4242);
    process.home = Some(f.provider.context.working_directory.join("other-home"));
    f.host
        .processes
        .lock()
        .unwrap()
        .insert(4242, process.clone());
    assert_eq!(
        f.query(),
        ServiceOwnerProof::Stopped,
        "current HOME absence alone was insufficient"
    );
    assert!(f
        .provider
        .require_program_unused(&f.provider.context)
        .is_err());
    process.image = PathBuf::from("/fixture/unrelated/node");
    process.argv = vec!["node".into(), "/fixture/unrelated/tool.js".into()];
    f.host.processes.lock().unwrap().insert(4242, process);
    f.provider
        .require_program_unused(&f.provider.context)
        .unwrap();
}

#[cfg(target_os = "macos")]
#[test]
fn actual_current_user_image_scan_detects_a_renamed_owned_program_without_service_mutation() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().canonicalize().unwrap();
    let i = root.join("I");
    let h = root.join("H");
    std::fs::create_dir_all(i.join("runtime/versions/test")).unwrap();
    std::fs::create_dir(&h).unwrap();
    let image = i.join("runtime/versions/test/renamed-helper");
    std::fs::copy("/bin/sleep", &image).unwrap();
    // The copied system binary's original signature is not valid at this test
    // inode. Ad-hoc signing affects this isolated fixture only, no release keys.
    let signed = std::process::Command::new("/usr/bin/codesign")
        .args(["--force", "--sign", "-"])
        .arg(&image)
        .output()
        .unwrap();
    assert!(
        signed.status.success(),
        "{}",
        String::from_utf8_lossy(&signed.stderr)
    );

    struct Child(std::process::Child);
    impl Drop for Child {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let mut child = Child(
        std::process::Command::new(&image)
            .arg("30")
            .spawn()
            .unwrap(),
    );
    let host = PlatformHost;
    let processes = host
        .program_processes(&path_key(&i.join("runtime")))
        .unwrap();
    assert!(
        processes
            .iter()
            .any(|process| process.pid == child.0.id()
                && path_key(&process.image) == path_key(&image))
    );
    std::thread::sleep(Duration::from_millis(100));
    let before_unlink = child.0.try_wait().unwrap();
    assert!(
        before_unlink.is_none(),
        "owned process must be running before unlink: {before_unlink:?}"
    );
    // A live, exclusively test-owned executable may lose its disk path. That
    // must preserve a positive in-use observation or refuse proof, never absence.
    std::fs::remove_file(&image).unwrap();
    let observed = host.program_processes(&path_key(&i.join("runtime")));
    assert!(
        child.0.try_wait().unwrap().is_none(),
        "unlinked signed fixture must remain live during absence check"
    );
    assert!(
        !matches!(observed,Ok(processes) if !processes.iter().any(|process|process.pid==child.0.id()))
    );
    drop(child);
    let processes = host
        .program_processes(&path_key(&i.join("runtime")))
        .unwrap();
    assert!(!processes
        .iter()
        .any(|process| path_key(&process.image) == path_key(&image)));
}

#[cfg(target_os = "macos")]
#[test]
fn removal_kernel_path_decoder_uses_exec_header_not_caller_argv_or_environment() {
    let mut bytes = 2i32.to_ne_bytes().to_vec();
    bytes.extend_from_slice(b"/fixture/runtime/versions/2.99.0/dependencies/node/bin/node\0\0unrelated-spoofed-argv0\0/other/tool.js\0OPENCODEX_HOME=/other/home\0");
    assert_eq!(
        native::exec_path_from_procargs(&bytes).unwrap(),
        PathBuf::from("/fixture/runtime/versions/2.99.0/dependencies/node/bin/node")
    );
    for payload in [
        b"".as_slice(),
        b"node\0caller-argv",
        b"/absolute-with-no-terminator",
        b"/absolute/../escape\0",
        b"/with\ncontrol\0",
    ] {
        let mut frame = 1i32.to_ne_bytes().to_vec();
        frame.extend_from_slice(payload);
        assert!(native::exec_path_from_procargs(&frame).is_err());
    }
}
