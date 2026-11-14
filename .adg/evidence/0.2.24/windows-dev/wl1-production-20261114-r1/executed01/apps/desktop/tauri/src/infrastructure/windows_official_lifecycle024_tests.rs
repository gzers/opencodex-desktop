use super::*;
use crate::modules::process::ProcessRunner;

#[test]
fn expired_window_and_missing_owner_cannot_start_or_adopt() {
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
    let snapshot = || -> serde_json::Value {
        json!(clients.iter().map(|p|{let m=fs::metadata(p).unwrap();json!({"path":p,"bytes":m.len(),"sha256":hash::sha256_file(p).unwrap(),"readonly":m.permissions().readonly()})}).collect::<Vec<_>>())
    };
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
                if let Ok(state) = owner.native.family_state() {
                    clean = state.complete_exit();
                    event(
                        "final-native-state",
                        json!({"completeExit":clean,"active":state.active_processes,"retained":state.retained_members.iter().map(|(p,s)|json!({"pid":p.pid,"filetime":p.creation_filetime,"state":format!("{s:?}")})).collect::<Vec<_>>()}),
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
