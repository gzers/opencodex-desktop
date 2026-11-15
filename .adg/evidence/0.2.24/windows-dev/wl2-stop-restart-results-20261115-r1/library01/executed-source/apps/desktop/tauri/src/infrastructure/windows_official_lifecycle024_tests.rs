use super::*;
use crate::modules::process::ProcessRunner;
#[test]
fn wl2_restart_stop_receipt_keeps_original_outer_window_and_input() {
    let temp = tempfile::tempdir().unwrap();
    let executable = temp.path().join("ocx.exe");
    fs::write(&executable, b"nonexecuted synthetic source").unwrap();
    let home = temp.path().join("Profile");
    let official = temp.path().join("H");
    fs::create_dir(&home).unwrap();
    fs::create_dir(&official).unwrap();
    let mut command = ProcessCommand::new(LifecycleAction::Restart, executable, home, official);
    command.requested_port = Some(54109);
    command.environment.http_proxy = Some("http://127.0.0.1:43117".into());
    let now = Instant::now();
    let started = now - Duration::from_secs(12);
    let window = shared::ActionWindow::begin(
        LifecycleAction::Restart,
        started,
        now + Duration::from_secs(45),
    )
    .unwrap();
    assert!(window.stop_deadline() < now);
    let stop_time = started + Duration::from_secs(9);
    let receipt = StopReceipt {
        completion: StopCompletion {
            completed_at: stop_time,
            original_deadline: window.stop_deadline(),
        },
    };
    let input = after_stop_input(&command, &window, receipt)
        .unwrap()
        .unwrap();
    assert_eq!(input.action, LifecycleAction::Start);
    assert_eq!(input.executable, command.executable);
    assert_eq!(input.working_directory, command.working_directory);
    assert_eq!(input.environment, command.environment);
    assert_eq!(input.requested_port, command.requested_port);
    assert_eq!(window.deadline(), started + Duration::from_secs(30));
    assert_eq!(window.stop_deadline(), started + Duration::from_secs(10));
    assert!(window.deadline() < now + Duration::from_secs(19));
    // This is a private time/input fixture, NOT a native StopReceipt factory or
    // official Stop proof. It only verifies the actual dispatch helper never
    // rejects a historically timely Stop merely because its subwindow has now
    // passed, nor replaces the remaining outer budget/port/environment.
    assert_eq!(command.action, LifecycleAction::Restart);
    println!("Restart dispatch preserves exact original30/Stop10 window and all launch inputs; timely synthetic Stop at9 remains data-valid at12, new Start has only original remaining18; no process or native permission issued");
}
#[test]
fn wl2_restart_wrong_late_or_expired_stop_receipt_never_returns_launch_input() {
    let now = Instant::now();
    let window =
        shared::ActionWindow::begin(LifecycleAction::Restart, now, now + Duration::from_secs(30))
            .unwrap();
    let command = ProcessCommand::new(
        LifecycleAction::Restart,
        "C:/fixture/ocx.exe",
        "C:/fixture/Profile",
        "C:/fixture/H",
    );
    for (completed, deadline) in [
        (window.stop_deadline(), window.stop_deadline()),
        (
            window.stop_deadline() + Duration::from_millis(1),
            window.stop_deadline(),
        ),
        (now, window.deadline()),
    ] {
        let receipt = StopReceipt {
            completion: StopCompletion {
                completed_at: completed,
                original_deadline: deadline,
            },
        };
        assert!(after_stop_input(&command, &window, receipt).is_err());
    }
    assert!(StopCompletion::record(now).is_err());
    let mut stop = command.clone();
    stop.action = LifecycleAction::Stop;
    let stop_window =
        shared::ActionWindow::begin(LifecycleAction::Stop, now, now + Duration::from_secs(45))
            .unwrap();
    let receipt = StopReceipt {
        completion: StopCompletion {
            completed_at: now,
            original_deadline: stop_window.stop_deadline(),
        },
    };
    assert!(after_stop_input(&stop, &stop_window, receipt)
        .unwrap()
        .is_none());
    let mut start = command;
    start.action = LifecycleAction::Start;
    let receipt = StopReceipt {
        completion: StopCompletion {
            completed_at: now,
            original_deadline: window.stop_deadline(),
        },
    };
    assert!(after_stop_input(&start, &window, receipt).is_err());
    println!("late/wrong-window Stop receipt, expired recording and action mixing refuse successor input; Stop returns no launch input; private time fixtures never mint native ClosedEligibility");
}
#[test]
fn wl2_closed_status_without_original_owner_or_deadline_stays_unknown() {
    let r = crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved();
    let home = Path::new("C:/unowned-exact-H");
    let now = Instant::now();
    assert_eq!(
        recovery_state_at(&r, home, now),
        crate::modules::status::ClientRecoveryState::Unknown
    );
    assert_eq!(
        recovery_state_at(&r, home, now + Duration::from_secs(5)),
        crate::modules::status::ClientRecoveryState::Unknown
    );
    assert_eq!(
        recovery_state(&r, home),
        crate::modules::status::ClientRecoveryState::Unknown
    );
    assert!(!native_stopped(&r, home));
    println!("closed-status receiver requires original retained Owner and deadline; missing or expired context never clears recovery, no text/boolean adoption");
}
#[test]
fn wl2_sd_accounting_preserves_child_raw_exit_for_mandatory_role_proof() {
    let child = native::ProcessIdentity {
        pid: 4242,
        creation_filetime: 123,
        token: native::TokenIdentity {
            sid: "S-1-5-21-1-2-3-1000".into(),
            session_id: 7,
            elevated: false,
            elevation_type: 1,
        },
        executable: PathBuf::from("C:/synthetic/sc.exe"),
        executable_id: native::FileIdentity {
            volume: 1,
            index: 3,
        },
    };
    for raw in [0, 1, 5, 1060, u32::MAX] {
        let family = native::FamilyState {
            root: ProcessState::Exited { code: 0 },
            total_processes: 2,
            active_processes: 0,
            active_members: Vec::new(),
            retained_members: vec![(child.clone(), ProcessState::Exited { code: raw })],
        };
        require_accounted_exit(&family).unwrap();
        assert_eq!(
            family.retained_members[0].1,
            ProcessState::Exited { code: raw }
        );
        // This precheck grants no role, NormalExit or lifecycle result. Even
        // raw0 must still pass the source/argv/parent/native-absence proof.
    }
    let mut incomplete = native::FamilyState {
        root: ProcessState::Exited { code: 0 },
        total_processes: 2,
        active_processes: 0,
        active_members: Vec::new(),
        retained_members: Vec::new(),
    };
    assert!(require_accounted_exit(&incomplete).is_err());
    incomplete
        .retained_members
        .push((child.clone(), ProcessState::Running));
    assert!(require_accounted_exit(&incomplete).is_err());
    incomplete.retained_members[0].1 = ProcessState::Exited { code: 0 };
    incomplete.root = ProcessState::Exited { code: 5 };
    assert!(require_accounted_exit(&incomplete).is_err());
    incomplete.root = ProcessState::Exited { code: 0 };
    incomplete.active_processes = 1;
    assert!(require_accounted_exit(&incomplete).is_err());
    incomplete.active_processes = 0;
    incomplete.active_members.push(child);
    assert!(require_accounted_exit(&incomplete).is_err());
}
#[test]
fn wl2_actual_ocx_pid_blocks_both_teardown_and_stopped_readback() {
    let root = tempfile::tempdir().unwrap();
    assert!(teardown_records_absent(root.path()).is_ok());
    assert!(!root.path().join("opencodex.pid").exists());
    fs::write(root.path().join("ocx.pid"), b"12345").unwrap();
    assert!(teardown_records_absent(root.path()).is_err());
    fs::remove_file(root.path().join("ocx.pid")).unwrap();
    fs::create_dir(root.path().join("ocx.pid")).unwrap();
    assert!(teardown_records_absent(root.path()).is_err());
    fs::remove_dir(root.path().join("ocx.pid")).unwrap();
    fs::write(root.path().join("runtime-port.json"), b"{}").unwrap();
    assert!(teardown_records_absent(root.path()).is_err());
}
#[test]
#[ignore = "one isolated ordinary account; W-L2 independent prerequisites, no official launch"]
fn isolated_wl2_native_boundaries() {
    use super::super::{windows_client_originals024, windows_owner_host024, windows_supervisor024};
    let token = native::current_identity().unwrap();
    assert!(!token.elevated);
    assert_eq!(token.sid, std::env::var("OCXD024_ACCOUNT_SID").unwrap());
    let profile = platform::home_dir().unwrap();
    assert!(profile
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with("ocxw1d"));
    let evidence = PathBuf::from(std::env::var_os("OCXD024_NATIVE_EVIDENCE").unwrap());
    let root = profile.join("wl2-prerequisites");
    assert!(!root.exists());
    platform::create_private_dir_all(&root, 0o700).unwrap();
    let original = root.join("synthetic-original");
    fs::write(&original, b"synthetic original; no credentials").unwrap();
    let before = client_native_identity(&original);
    let security = match windows_client_originals024::InspectedOriginal::open(&original, false) {
        Ok(o) => match o.recheck() {
            Ok(()) => serde_json::json!({"completeSecurityReadable":true}),
            Err(e) => {
                serde_json::json!({"completeSecurityReadable":false,"stage":"recheck-complete-descriptor","rawWin32":e.raw_os_error(),"error":e.to_string()})
            }
        },
        Err(e) => {
            serde_json::json!({"completeSecurityReadable":false,"stage":"Open-original-READ_CONTROL-ACCESS_SYSTEM_SECURITY","rawWin32":e.raw_os_error(),"error":e.to_string()})
        }
    };
    let after = client_native_identity(&original);
    let deadline = Instant::now() + Duration::from_secs(5);
    let namespaces = match windows_supervisor024::query(deadline) {
        Ok(facts) => serde_json::to_value(facts).unwrap(),
        Err(e) => serde_json::json!({"error":e.to_string(),"rawWin32":e.raw_os_error()}),
    };
    let staged = windows_supervisor024::staged(&root);
    let outer = match windows_owner_host024::outer_job_precondition() {
        Ok(()) => {
            serde_json::json!({"outerJobSafe":true,"evidence":"IsProcessInJob current NULL == FALSE"})
        }
        Err(e) => {
            serde_json::json!({"outerJobSafe":false,"error":e.to_string(),"rawWin32":e.raw_os_error()})
        }
    };
    let qpc = windows_owner_host024::qpc().unwrap();
    let result = serde_json::json!({"sid":token.sid,"elevated":token.elevated,"profile":profile,"security":security,"originalBefore":before,"originalAfter":after,"originalUnchanged":before==after,"nativeNamespaces":namespaces,"exactStaged":staged,"ownerOuterJob":outer,"qpc":{"counter":qpc.0.to_string(),"frequency":qpc.1.to_string()},"productionExecuted":false,"accepted":false,"safeToRemoveTestIdentity":true,"ordinaryMsiAcceptance":false});
    fs::write(
        evidence.join("wl2-independent-prerequisites.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    fs::write(evidence.join("wl1-final-owner.json"), serde_json::to_vec_pretty(&serde_json::json!({"sid":token.sid,"safeToRemoveTestIdentity":true,"jobCreated":false,"forced":false,"allAccepted":false})).unwrap()).unwrap();
    assert_eq!(before, after, "prerequisite refusal mutated original");
    assert_eq!(security["completeSecurityReadable"], true, "W-L2 complete original security prerequisite failed; no capture/official mutation/production allowed");
    assert_eq!(
        outer["outerJobSafe"], true,
        "original owner continuity outer Job prerequisite unknown"
    );
}
fn client_native_identity(path: &Path) -> serde_json::Value {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::{
        Security::GetKernelObjectSecurity,
        Storage::FileSystem::{GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION},
    };
    let f = OpenOptions::new()
        .access_mode(0x80020000)
        .share_mode(7)
        .open(path)
        .unwrap();
    let mut i: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    assert_ne!(
        unsafe { GetFileInformationByHandle(f.as_raw_handle(), &mut i) },
        0
    );
    let mut needed = 0;
    unsafe { GetKernelObjectSecurity(f.as_raw_handle(), 7, std::ptr::null_mut(), 0, &mut needed) };
    assert!((20..=262144).contains(&needed));
    let mut partial = vec![0u8; needed as usize];
    assert_ne!(
        unsafe {
            GetKernelObjectSecurity(
                f.as_raw_handle(),
                7,
                partial.as_mut_ptr().cast(),
                needed,
                &mut needed,
            )
        },
        0
    );
    serde_json::json!({"bytesSha256":hash::sha256_file(path).unwrap(),"volume":i.dwVolumeSerialNumber,"fileId":(((i.nFileIndexHigh as u64)<<32)|i.nFileIndexLow as u64).to_string(),"ownerGroupDaclSha256":hash::sha256_hex(&partial),"completeSecurityProven":false})
}
struct TestFailureGuard {
    runtime: SharedRuntimeExecutable,
    home: PathBuf,
    evidence: PathBuf,
    sid: String,
}
impl Drop for TestFailureGuard {
    fn drop(&mut self) {
        let marker = self.evidence.join("wl1-final-owner.json");
        if marker.exists() {
            return;
        }
        let mut clean = false;
        let mut forced = false;
        if let Ok(entry) = slot(&self.runtime, &self.home, false) {
            if let Ok(mut locked) = entry.owner.try_lock() {
                if let Some(owner) = locked.as_mut() {
                    clean = owner.native.family_state().is_ok_and(|f| f.complete_exit());
                    if !clean {
                        forced = true;
                        clean = owner
                            .native
                            .force_owned_cleanup(Instant::now() + Duration::from_secs(5))
                            .is_ok_and(|f| f.final_state.complete_exit());
                    }
                }
            }
        }
        let _=fs::write(marker,serde_json::to_vec(&serde_json::json!({"sid":self.sid,"safeToRemoveTestIdentity":clean,"panicCleanup":true,"forced":forced,"ordinaryStopSuccess":false,"allAccepted":false})).unwrap());
    }
}
fn client_snapshot(profile: &Path) -> serde_json::Value {
    use std::os::windows::{fs::MetadataExt, io::AsRawHandle};
    use windows_sys::Win32::{
        Security::GetKernelObjectSecurity,
        Storage::FileSystem::{GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION},
    };
    let mut rows = vec![];
    let mut queue = vec![profile.join(".codex"), profile.join(".grok")];
    while let Some(p) = queue.pop() {
        match fs::symlink_metadata(&p) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                rows.push(serde_json::json!({"path":p,"absent":true}));
                continue;
            }
            Err(e) => panic!("declared client snapshot unreadable: {e}"),
            Ok(m) => {
                assert_eq!(m.file_attributes() & 0x400, 0);
                if m.is_dir() {
                    for e in fs::read_dir(&p).unwrap() {
                        queue.push(e.unwrap().path())
                    }
                    continue;
                }
            }
        }
        assert!(rows.len() < 512);
        let f = OpenOptions::new()
            .access_mode(0x80020000)
            .share_mode(7)
            .open(&p)
            .unwrap();
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        assert_ne!(
            unsafe { GetFileInformationByHandle(f.as_raw_handle(), &mut info) },
            0
        );
        assert_eq!(info.nNumberOfLinks, 1);
        let mut needed = 0;
        unsafe {
            GetKernelObjectSecurity(f.as_raw_handle(), 7, std::ptr::null_mut(), 0, &mut needed)
        };
        assert!((1..=65536).contains(&needed));
        let mut sd = vec![0u8; needed as usize];
        assert_ne!(
            unsafe {
                GetKernelObjectSecurity(
                    f.as_raw_handle(),
                    7,
                    sd.as_mut_ptr().cast(),
                    needed,
                    &mut needed,
                )
            },
            0
        );
        rows.push(serde_json::json!({"path":p,"bytes":f.metadata().unwrap().len(),"sha256":hash::sha256_file(&p).unwrap(),"volume":info.dwVolumeSerialNumber,"fileId":((info.nFileIndexHigh as u64)<<32)|info.nFileIndexLow as u64,"securitySha256":hash::sha256_hex(&sd)}));
    }
    rows.sort_by_key(|v| v["path"].as_str().unwrap().to_owned());
    serde_json::json!(rows)
}

#[test]
fn expired_window_and_missing_owner_cannot_start_or_adopt() {
    let windows_spelling = "C:/exact/profile/runtime/bin".replace('/', &char::from(92).to_string());
    assert!(same(
        Path::new("C:/exact/profile/runtime/bin"),
        Path::new(&windows_spelling)
    ));
    assert!(!same(
        Path::new("C:/other/profile/runtime/bin"),
        Path::new(&windows_spelling)
    ));
    let now = Instant::now();
    assert!(shared::ActionWindow::begin(LifecycleAction::Start, now, now).is_err());
    let r = crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved();
    assert!(slot(&r, Path::new("C:/explicit-unowned-H"), false).is_err());
    assert_eq!(
        recovery_state(&r, Path::new("C:/explicit-unowned-H")),
        crate::modules::status::ClientRecoveryState::Unknown
    );
    assert!(!native_stopped(&r, Path::new("C:/explicit-unowned-H")));
}
#[test]
fn restart_inherits_original_stop_subwindow_and_production_runner_opts_in() {
    let now = Instant::now();
    let w =
        shared::ActionWindow::begin(LifecycleAction::Restart, now, now + Duration::from_secs(45))
            .unwrap();
    assert_eq!(w.deadline(), now + Duration::from_secs(30));
    assert_eq!(w.stop_deadline(), now + Duration::from_secs(10));
    assert!(
        crate::infrastructure::process_runner::ControlledProcessRunner::new("C:/profile")
            .uses_windows_official024_owner()
    );
}

#[test]
#[ignore = "one fresh ordinary ocxw1d Profile; fixed official2.99, no MSI/CLI or upstream request"]
fn isolated_production_lifecycle_wl1() {
    use crate::modules::runtime::{bundled, RuntimeHandle};
    use serde_json::json;
    use std::io::Write;
    let token = native::current_identity().unwrap();
    assert!(!token.elevated);
    assert_eq!(token.sid, std::env::var("OCXD024_ACCOUNT_SID").unwrap());
    let profile = platform::home_dir().unwrap();
    assert!(profile
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with("ocxw1d"));
    assert!(!crate::modules::test_sandbox::enabled());
    let input = PathBuf::from(std::env::var_os("OCXD024_NATIVE_INPUT").unwrap());
    let evidence = PathBuf::from(std::env::var_os("OCXD024_NATIVE_EVIDENCE").unwrap());
    let event = |name: &str, value: serde_json::Value| {
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(evidence.join("wl1-events.jsonl"))
            .unwrap();
        writeln!(f, "{}", json!({"event":name,"facts":value})).unwrap();
        f.sync_all().unwrap();
    };
    event(
        "hypothesis",
        json!({"sid":token.sid,"session":token.session_id,"elevated":false,"profile":profile,"scope":"actual production command and status; normal native exit distinct from client recovery"}),
    );
    let base = profile.join("wl1 native");
    let data = base.join("data/desktop");
    let root = base.join("I 空格 & !");
    let home = base.join("custom H 雪");
    assert!(!base.exists());
    for p in [&data, &root, &home] {
        platform::create_private_dir_all(p, 0o700).unwrap()
    }
    crate::modules::data_root::initialize(&data).unwrap();
    let descriptor = fs::read(input.join("runtime-bundle.manifest.json")).unwrap();
    assert_eq!(
        hash::sha256_hex(&descriptor),
        "42539a22dde96977d7575077d3e27e4537fe2794a46c07c565a73cedb7a11a1a"
    );
    let bundle = input.join("runtime-bundle-0.2.24-win32-x64.tar.gz");
    assert_eq!(fs::metadata(&bundle).unwrap().len(), 274033721);
    assert_eq!(
        hash::sha256_file(&bundle).unwrap(),
        "14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7"
    );
    bundled::versioned::provision(
        &root,
        &data,
        &bundle,
        &descriptor,
        "42539a22dde96977d7575077d3e27e4537fe2794a46c07c565a73cedb7a11a1a",
    )
    .unwrap();
    let runtime: SharedRuntimeExecutable = RuntimeHandle::initialize_versioned(&data, &root);
    let _failure_guard = TestFailureGuard {
        runtime: runtime.clone(),
        home: home.clone(),
        evidence: evidence.clone(),
        sid: token.sid.clone(),
    };
    let context = crate::state::ProcessContext {
        runtime: runtime.clone(),
        working_directory: profile.clone(),
        opencodex_home: home.clone(),
    };
    let clients = [
        profile.join(".codex/config.toml"),
        profile.join(".codex/auth.json"),
        profile.join(".grok/settings.json"),
    ];
    for (n, p) in clients.iter().enumerate() {
        platform::create_private_dir_all(p.parent().unwrap(), 0o700).unwrap();
        let mut f = platform::create_new_private_file(p, 0o600).unwrap();
        f.write_all([b"# synthetic wl1 client config\n".as_slice(), b"{}", b"{}"][n])
            .unwrap();
    }
    let snapshot = || client_snapshot(&profile);
    let before = snapshot();
    event("clients-before", before.clone());
    let runner =
        Mutex::new(crate::infrastructure::process_runner::ControlledProcessRunner::new(&profile));
    let log = crate::infrastructure::runtime_log::RuntimeLog::new(&data);
    let run = |action| {
        let at = Instant::now();
        let result = crate::commands::process_action_with_runner(
            crate::types::process_action::ProcessActionRequest {
                action,
                confirm: true,
            },
            &runner,
            &context,
            &log,
            None,
        );
        event(
            "production-action",
            json!({"action":action,"elapsedMs":at.elapsed().as_millis(),"result":result.as_ref().ok(),"error":result.as_ref().err().map(ToString::to_string)}),
        );
        result
    };
    let started = run(LifecycleAction::Start);
    if let Ok(entry) = slot(&runtime, &home, false) {
        if let Ok(mut locked) = entry.owner.try_lock() {
            if let Some(owner) = locked.as_mut() {
                let state = owner.native.family_state();
                event(
                    "post-start-native-diagnostic",
                    json!({"stage":owner.stage,"snapshot":state.as_ref().ok().map(|s|json!({"total":s.total_processes,"active":s.active_processes,"retained":s.retained_members.len()})),"error":state.err().map(|e|e.to_string())}),
                );
            }
        }
    }
    event("clients-ready-or-failure", snapshot());
    let started_ok = started
        .as_ref()
        .is_ok_and(|r| r.result == Some(LifecycleResult::Started));
    let environment = crate::modules::process::EnvironmentPolicy {
        home: Some(profile.clone().into_os_string()),
        opencodex_home: home.clone(),
        ..Default::default()
    };
    let source = crate::infrastructure::status_source::OfficialStatusSource::new(
        runtime.clone(),
        &profile,
        environment,
    )
    .with_client_recovery_root(data.clone());
    let statuses = std::thread::scope(|s| {
        s.spawn(|| {
            tauri::async_runtime::block_on(async {
                let mut results = vec![];
                for _ in 0..3 {
                    use crate::modules::status::StatusSource;
                    let at = Instant::now();
                    let result = source.fetch();
                    results
                        .push(json!({"elapsedMs":at.elapsed().as_millis(),"result":result.ok()}));
                }
                results
            })
        })
        .join()
        .unwrap()
    });
    event("status-samples", json!(statuses));
    if started_ok {
        let entry = slot(&runtime, &home, false).unwrap();
        let mut locked = entry.owner.lock().unwrap();
        let owner = locked.as_mut().unwrap();
        let (pid, port, digest) = owner.service.clone().unwrap();
        let mut refused = vec![];
        for (label, p, n, d) in [
            ("wrong-pid", pid.wrapping_add(1), port, digest.clone()),
            ("wrong-port", pid, port.wrapping_add(1), digest.clone()),
            ("wrong-record", pid, port, "0".repeat(64)),
        ] {
            refused.push(json!({"case":label,"refused":owner.live(p,n,&d,Instant::now()+Duration::from_secs(5)).is_err()}));
        }
        event("native-binding-negatives", json!(refused));
        assert!(refused.iter().all(|v| v["refused"] == true));
    }
    event("clients-pre-stop", snapshot());
    let stopped = run(LifecycleAction::Stop);
    let normal = native_stopped(&runtime, &home);
    event("clients-stopped-or-failure", snapshot());
    let restart = run(LifecycleAction::Restart);
    let second = run(LifecycleAction::Start);
    event(
        "native-vs-restoration",
        json!({"started":started_ok,"nativeStopped":normal,"fullStop":stopped.as_ref().is_ok_and(|r|r.result==Some(LifecycleResult::Stopped)),"restartStarted":restart.as_ref().is_ok_and(|r|r.result==Some(LifecycleResult::Started)),"secondStarted":second.as_ref().is_ok_and(|r|r.result==Some(LifecycleResult::Started)),"declaredSyntheticClientsUnchanged":snapshot()==before,"durableClientRecoveryVerified":false,"managerRestartContinuityVerified":false}),
    );
    let mut clean = false;
    if let Ok(slot) = slot(&runtime, &home, false) {
        if let Ok(mut owner) = slot.owner.try_lock() {
            if let Some(owner) = owner.as_mut() {
                event("owner-diagnostic-stage", json!({"stage":owner.stage}));
                if let Ok(state) = owner.native.family_state() {
                    clean = state.complete_exit();
                    event(
                        "final-native-state",
                        json!({"completeExit":clean,"active":state.active_processes,"total":state.total_processes,"retained":state.retained_members.iter().map(|(p,s)|json!({"pid":p.pid,"filetime":p.creation_filetime,"image":p.executable,"state":format!("{s:?}")})).collect::<Vec<_>>()}),
                    );
                }
                if !clean {
                    let cleanup = owner
                        .native
                        .force_owned_cleanup(Instant::now() + Duration::from_secs(5));
                    clean = cleanup
                        .as_ref()
                        .is_ok_and(|r| r.final_state.complete_exit());
                    event(
                        "explicit-test-only-failure-cleanup",
                        json!({"forced":true,"completeExit":clean,"ordinaryStopSuccess":false}),
                    );
                }
            }
        }
    }
    fs::write(evidence.join("wl1-final-owner.json"),serde_json::to_vec(&json!({"sid":token.sid,"safeToRemoveTestIdentity":clean,"nativeStopped":normal,"allAccepted":false})).unwrap()).unwrap();
    assert!(
        started_ok,
        "production Start failed; exact owner retained until explicit failure cleanup"
    );
    assert!(normal, "native Stop proof incomplete");
    assert!(statuses.iter().all(|s| s["result"]["status"] == "running"));
}
