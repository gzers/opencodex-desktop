use super::*;
use crate::{infrastructure::hash, modules::process::ProcessRunner};
#[test]
fn wl2_actual_ocx_pid_blocks_both_teardown_and_stopped_readback() {
    let root = tempfile::tempdir().unwrap();
    assert!(runtime_records_absent(root.path()).is_ok());
    assert!(!root.path().join("opencodex.pid").exists());
    fs::write(root.path().join("ocx.pid"), b"12345").unwrap();
    assert!(runtime_records_absent(root.path()).is_err());
    fs::remove_file(root.path().join("ocx.pid")).unwrap();
    fs::create_dir(root.path().join("ocx.pid")).unwrap();
    assert!(runtime_records_absent(root.path()).is_err());
    fs::remove_dir(root.path().join("ocx.pid")).unwrap();
    fs::write(root.path().join("runtime-port.json"), b"{}").unwrap();
    assert!(runtime_records_absent(root.path()).is_err());
}
#[tokio::test]
async fn start_refuses_pid_record_without_runtime_port_record() {
    use crate::infrastructure::runtime_executable::RuntimeExecutableProvider;

    #[derive(Debug)]
    struct ManagedFixture(PathBuf);

    impl RuntimeExecutableProvider for ManagedFixture {
        fn executable(&self) -> Option<PathBuf> {
            None
        }

        fn managed_installation_root(&self) -> Option<PathBuf> {
            Some(self.0.clone())
        }
    }

    let root = tempfile::tempdir().unwrap();
    let data = root.path().join("data");
    let home = root.path().join("home");
    fs::create_dir(&home).unwrap();
    let pid_record = home.join("ocx.pid");
    fs::write(&pid_record, b"12345").unwrap();
    let runtime: SharedRuntimeExecutable = Arc::new(ManagedFixture(root.path().join("program")));
    let command = ProcessCommand::new(
        LifecycleAction::Start,
        root.path().join("missing-entry.exe"),
        platform::home_dir().unwrap(),
        home.clone(),
    );
    let now = Instant::now();
    let window =
        shared::ActionWindow::begin(command.action, now, now + Duration::from_secs(30)).unwrap();

    let result = execute_async(&runtime, &command, &data, window).await;
    assert!(
        matches!(&result, Err(AppError::RuntimeManaged { code, .. }) if code == "windows024_owner_unknown"),
        "{result:?}"
    );
    assert_eq!(fs::read(&pid_record).unwrap(), b"12345");
    assert!(!data.exists());
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
fn read_json(path: &Path, label: &str) -> serde_json::Value {
    match fs::read(path) {
        Ok(bytes) => match serde_json::from_slice::<serde_json::Value>(&bytes) {
            Ok(value) => serde_json::json!({"label":label,"path":path,"value":value}),
            Err(_) => serde_json::json!({"label":label,"path":path,"invalid":true}),
        },
        Err(error) => {
            serde_json::json!({"label":label,"path":path,"absent":error.kind()==std::io::ErrorKind::NotFound})
        }
    }
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
}

#[tokio::test]
async fn status_uses_original_deadline_without_inventing_an_owner() {
    let runtime = crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved();
    let home = Path::new("C:/explicit-unowned-H");
    let now = Instant::now();
    let deadline = now + crate::infrastructure::status_source::status_collect_timeout();

    assert_eq!(
        status(&runtime, home, now).await,
        Err(CollectError::Timeout)
    );
    assert_eq!(
        status(&runtime, home, deadline).await,
        Err(CollectError::Unreachable)
    );
}

#[test]
#[ignore = "requires a verified, provisioned Windows 2.99 candidate and a fresh diagnostic directory"]
fn provisioned_program_first_status_without_owner() {
    use crate::infrastructure::status_source::{status_collect_timeout, OfficialStatusSource};
    use crate::modules::{
        process::EnvironmentPolicy, runtime::RuntimeHandle, status::StatusSource,
    };

    let install = PathBuf::from(std::env::var_os("OCXD024_STATUS_PROBE_INSTALL").unwrap());
    let scratch = PathBuf::from(std::env::var_os("OCXD024_STATUS_PROBE_SCRATCH").unwrap());
    assert!(install.is_absolute() && install.is_dir());
    assert!(scratch.is_absolute());
    fs::create_dir(&scratch).expect("reserve a fresh diagnostic directory");
    let home = scratch.join("home");
    fs::create_dir(&home).unwrap();
    let runtime: SharedRuntimeExecutable =
        RuntimeHandle::initialize_versioned(&scratch.join("desktop"), &install);
    let source = OfficialStatusSource::new(
        runtime.clone(),
        &scratch,
        EnvironmentPolicy {
            opencodex_home: home.clone(),
            ..Default::default()
        },
    );
    let executor = tokio::runtime::Runtime::new().unwrap();
    let started = Instant::now();
    let identity = runtime.managed_program_identity();
    eprintln!(
        "identity elapsed_us={} result={:?}",
        started.elapsed().as_micros(),
        identity.as_ref().map(|i| &i.package_version)
    );
    assert_eq!(identity.unwrap().package_version, "2.99.0");
    assert!(runtime.executable().is_some_and(|entry| entry.is_file()));
    assert_eq!(status_collect_timeout(), Duration::from_secs(5));

    executor.block_on(async {
        let started = Instant::now();
        let result = status(&runtime, &home, started + status_collect_timeout()).await;
        eprintln!(
            "native no-owner elapsed_us={} result={result:?}",
            started.elapsed().as_micros()
        );
        assert_eq!(result, Err(CollectError::Unreachable));

        for (label, sample) in [
            ("full", OfficialStatusSource::fetch_at as fn(_, _) -> _),
            ("light", OfficialStatusSource::fetch_light_at),
        ] {
            let started = Instant::now();
            let result = sample(&source, started + status_collect_timeout());
            eprintln!(
                "{label} elapsed_us={} result={result:?}",
                started.elapsed().as_micros()
            );
            assert!(matches!(
                result,
                Err(CollectError::Unreachable | CollectError::Timeout)
            ));
        }
    });
    assert!(fs::read_dir(&home).unwrap().next().is_none());
    let owner_lookup = slot(&runtime, &home, false).err().expect("no owner slot");
    assert!(
        matches!(owner_lookup, AppError::RuntimeManaged { code, .. } if code == "windows024_owner_unknown")
    );
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
#[ignore = "one ordinary Windows account; fresh production layout under %USERPROFILE%; fixed official2.99"]
fn r2_boundary_single_user() {
    use crate::modules::extensions::{
        client_targets,
        discovery::RawServerDefinition,
        projection::{self, ProjectionCommand},
        ClientId,
    };
    use crate::modules::runtime::{bundled, RuntimeHandle};
    use crate::modules::status::StatusSource;
    use serde_json::json;
    use std::io::Write;

    let token = native::current_identity().unwrap();
    assert!(!token.elevated, "ordinary token required");
    let profile = platform::home_dir().unwrap();
    let evidence = PathBuf::from(std::env::var_os("OCXD024_NATIVE_EVIDENCE").unwrap());
    let input = PathBuf::from(std::env::var_os("OCXD024_NATIVE_INPUT").unwrap());
    fs::create_dir_all(&evidence).unwrap();
    let event = |name: &str, value: serde_json::Value| {
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(evidence.join("r2-events.jsonl"))
            .unwrap();
        writeln!(f, "{}", json!({"event":name,"facts":value})).unwrap();
        f.sync_all().unwrap();
    };
    event(
        "start",
        json!({"sid":token.sid,"session":token.session_id,"elevated":false,"profile":profile,
            "layout":"production .codex-desktop under the current profile"}),
    );

    let aggregate = profile.join(".codex-desktop");
    // Reserve a new root using an exclusive create. Existing directories,
    // reparse points, and inaccessible paths must never become cleanup targets.
    let reserved = fs::create_dir(&aggregate);
    event(
        "layout-preflight",
        json!({"aggregate":aggregate,"created":reserved.is_ok(),
            "error":reserved.as_ref().err().map(ToString::to_string)}),
    );
    reserved.expect("a fresh .codex-desktop root in a disposable profile is required");
    let data = aggregate.join("desktop");
    let program = aggregate.join("program");
    let home = aggregate.join("opencodex/home");
    let daily_config = profile.join(".codex/config.toml");
    let daily_before = fs::metadata(&daily_config)
        .ok()
        .map(|m| {
            json!({"exists":true,"mtime":m.modified().unwrap().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                "sha256":hash::sha256_file(&daily_config).unwrap()})
        })
        .unwrap_or_else(|| json!({"exists":false}));
    let daily_dir_before = fs::metadata(profile.join(".codex")).ok().is_some();
    event(
        "daily-before",
        json!({"config":daily_before,"dir":daily_dir_before}),
    );

    for p in [&data, &program] {
        platform::create_private_dir_all(p, 0o700).unwrap();
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
    let provision = bundled::versioned::provision(
        &program,
        &data,
        &bundle,
        &descriptor,
        "42539a22dde96977d7575077d3e27e4537fe2794a46c07c565a73cedb7a11a1a",
    );
    event(
        "provision",
        json!({"ok":provision.is_ok(),"result":provision.as_ref().map(|r|format!("{r:?}")),"error":provision.as_ref().err().map(ToString::to_string)}),
    );
    provision.unwrap();
    let runtime: SharedRuntimeExecutable = RuntimeHandle::initialize_versioned(&data, &program);
    let managed = runtime.managed_installation_root().unwrap();
    let managed_codex = crate::modules::extensions::managed_codex_root(&data).unwrap();
    assert_eq!(managed_codex, crate::modules::process::client_home(&home));
    event(
        "managed-roots",
        json!({"dataRoot":data,"programRoot":program,"opencodexHome":home,"codexRoot":managed_codex,"executable":runtime.executable(),"identity":runtime.managed_program_identity().map(|i|i.package_version).ok()}),
    );

    // One MCP definition, Codex only. Disabling the other five targets first
    // keeps the write scope to the Desktop-managed Codex root.
    let targets = client_targets(&data, &profile);
    let codex = targets
        .iter()
        .find(|t| t.client_id == ClientId::Codex)
        .unwrap()
        .clone();
    assert_eq!(codex.mcp_config_path, home.join("codex-client/config.toml"));
    let mut config = crate::modules::extensions::ExtensionConfig::default();
    config.set_enabled(ClientId::Codex, true);
    for client in targets
        .iter()
        .map(|t| t.client_id)
        .filter(|c| *c != ClientId::Codex)
    {
        config.set_enabled(client, false);
    }
    let saved = crate::modules::extensions::ProjectionFingerprint::create(
        config.fingerprint(),
        chrono::Utc::now(),
    );
    crate::infrastructure::atomic_write::atomic_write(
        &projection::config_path(&data),
        &serde_json::to_vec_pretty(&serde_json::json!({"config":config,"fingerprint":saved}))
            .unwrap(),
        0o600,
    )
    .unwrap();
    let add = projection::execute(
        &data,
        &targets,
        ProjectionCommand::AddMcp {
            definition: RawServerDefinition {
                name: format!("proof_probe_{}", uuid::Uuid::new_v4()),
                value: json!({"command":r"C:\Windows\System32\cmd.exe","args":["/c","exit","0"]}),
            },
        },
    );
    event(
        "mcp-add",
        json!({"ok":add.is_ok(),"written":add.as_ref().ok().map(|r|r.mcp_written.clone()),"error":add.as_ref().err().map(|e|format!("{e:?}")),
            "codexConfig":codex.mcp_config_path,"codexRoot":managed_codex}),
    );
    let add = add.unwrap();
    assert_eq!(add.mcp_written, vec!["codex".to_string()]);
    let text = fs::read_to_string(&codex.mcp_config_path).unwrap();
    assert!(text.contains("proof_probe"), "Codex MCP write missing");
    let daily_after = fs::metadata(&daily_config)
        .ok()
        .map(|m| {
            json!({"exists":true,"mtime":m.modified().unwrap().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                "sha256":hash::sha256_file(&daily_config).unwrap()})
        })
        .unwrap_or_else(|| json!({"exists":false}));
    event(
        "daily-after",
        json!({"config":daily_after,"unchanged":daily_after==daily_before,"dirStillPresent":fs::metadata(profile.join(".codex")).ok().is_some()}),
    );
    assert_eq!(daily_after, daily_before, "real ~/.codex changed");

    let context = crate::state::ProcessContext {
        runtime: runtime.clone(),
        working_directory: profile.clone(),
        opencodex_home: home.clone(),
    };
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
            "action",
            json!({"action":action,"elapsedMs":at.elapsed().as_millis(),"ok":result.as_ref().ok(),
            "error":result.as_ref().err().map(ToString::to_string),
            "resultError":result.as_ref().ok().and_then(|v|v.failure.as_ref()).map(|f|f.code.clone()),
            "stage":slot(&runtime,&home,false).ok().and_then(|s|s.owner.try_lock().ok().and_then(|mut o|o.as_mut().map(|o|o.stage)))}),
        );
        result
    };
    let program_now = profile.join(".codex-desktop/program");
    let installed_before = read_json(
        &program_now.join("runtime/current.json"),
        "current.json before Start",
    );
    let started = run(LifecycleAction::Start);
    let installed_after = read_json(
        &program_now.join("runtime/current.json"),
        "current.json after Start",
    );
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
    .with_data_root(data.clone());
    let statuses = std::thread::scope(|s| {
        s.spawn(|| {
            tauri::async_runtime::block_on(async {
                let mut out = vec![];
                for _ in 0..3 {
                    let at = Instant::now();
                    let result = source.fetch();
                    out.push(
                        json!({"elapsedMs":at.elapsed().as_millis(),"ok":result.as_ref().is_ok(),
                        "status":result.as_ref().ok().and_then(|v|v.get("status").cloned())}),
                    );
                }
                out
            })
        })
        .join()
        .unwrap()
    });
    event("status", json!(statuses));
    let stopped = run(LifecycleAction::Stop);
    if stopped
        .as_ref()
        .is_ok_and(|r| r.result != Some(LifecycleResult::Stopped))
    {
        if let Ok(entry) = slot(&runtime, &home, false) {
            if let Ok(mut owner) = entry.owner.try_lock() {
                if let Some(owner) = owner.as_mut() {
                    let records: Vec<_> = ["runtime-port.json", "ocx.pid"]
                        .into_iter()
                        .map(|name| match fs::symlink_metadata(home.join(name)) {
                            Ok(_) => json!({"name":name,"present":true}),
                            Err(e) => json!({"name":name,"absent":e.kind()==std::io::ErrorKind::NotFound,"kind":format!("{:?}",e.kind())}),
                        })
                        .collect();
                    let listeners = owner.original_listeners.as_ref().map(|s| {
                        native::listener_exit_readback(s)
                            .map(|r| r.len())
                            .map_err(|e| e.to_string())
                    });
                    event(
                        "stop-failure-native",
                        json!({"stage":owner.stage,"service":owner.service,"records":records,"listeners":listeners}),
                    );
                }
            }
        }
    }
    event(
        "summary",
        json!({"started":started.as_ref().is_ok_and(|r|r.result==Some(LifecycleResult::Started)),
            "statusOk":statuses.iter().all(|s|s["status"]=="running"),
            "stopped":stopped.as_ref().is_ok_and(|r|r.result==Some(LifecycleResult::Stopped)),
            "managedRoot":managed,"codexRoot":managed_codex,
            "sid":token.sid,"session":token.session_id}),
    );
    drop(source);
    drop(runner);
    drop(context);
    let force = slot(&runtime, &home, false).ok().and_then(|s| {
        let mut owner = s.owner.try_lock().ok()?;
        let owner = owner.as_mut()?;
        let state = owner.native.family_state().ok();
        let forced = if state.as_ref().is_some_and(|s| !s.complete_exit()) {
            owner.native.force_owned_cleanup(Instant::now() + Duration::from_secs(5)).ok()
        } else {
            None
        };
        Some(json!({"stage":owner.stage,"state":state.map(|s|json!({"total":s.total_processes,"active":s.active_processes,"retained":s.retained_members.len()})),
            "forced":forced.map(|f|f.final_state.complete_exit())}))
    });
    event(
        "explicit-failure-cleanup",
        force.clone().unwrap_or(serde_json::Value::Null),
    );
    let stopped_ok = stopped
        .as_ref()
        .is_ok_and(|r| r.result == Some(LifecycleResult::Stopped));
    let start_ok = started
        .as_ref()
        .is_ok_and(|r| r.result == Some(LifecycleResult::Started));
    let release = if stopped_ok {
        release_stopped_owner_for_test(&runtime, &home)
    } else {
        Err(crate::errors::AppError::NotConfigured)
    };
    drop(runtime);
    let cleanup = if release.is_ok() {
        fs::remove_dir_all(&aggregate)
    } else {
        Err(std::io::Error::other(
            "failed Stop retains the production layout",
        ))
    };
    let residue = fs::metadata(&aggregate).ok().is_some();
    let leftover = std::process::Command::new("powershell.exe")
        .args(["-NoProfile","-Command","Get-CimInstance Win32_Process | Where-Object { $_.ExecutablePath -like '*\\.codex-desktop\\*' } | Select-Object ProcessId,ExecutablePath,SessionId | ConvertTo-Json -Compress"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string());
    event(
        "cleanup",
        json!({"removeOk":cleanup.is_ok(),"aggregatePresent":residue,"leftover":leftover,
            "dailyConfig":daily_after,"accountCreated":false,"stopped":stopped_ok,
            "programBefore":installed_before,"programAfter":installed_after}),
    );
    assert!(cleanup.is_ok(), "one-off production layout cleanup failed");
    assert!(!residue, "one-off production layout residue");
    assert!(start_ok, "production Start did not reach the running state");
    assert!(statuses.iter().all(|s| s["status"] == "running"));
    assert!(
        stopped_ok,
        "production Stop did not reach the stopped state"
    );
}

#[test]
#[ignore = "two already-provisioned ordinary Windows profiles; read-only startup roots"]
fn r2_boundary_two_accounts() {
    use serde_json::json;

    let token = native::current_identity().unwrap();
    assert!(!token.elevated, "ordinary token required");
    let evidence = PathBuf::from(std::env::var_os("OCXD024_NATIVE_EVIDENCE").unwrap());
    fs::create_dir_all(&evidence).unwrap();
    let current = platform::home_dir().unwrap();
    let users = [
        (token.sid.clone(), current.clone()),
        (
            std::env::var("OCXD024_SECOND_SID").unwrap(),
            PathBuf::from(std::env::var_os("OCXD024_SECOND_PROFILE").unwrap()),
        ),
    ];
    let roots: Vec<_> = users
        .iter()
        .map(|(sid, profile)| {
            let aggregate = profile.join(".codex-desktop");
            let data = aggregate.join("desktop");
            let home = aggregate.join("opencodex/home");
            let codex = crate::modules::process::client_home(&home);
            let marker = aggregate.join("manager-state/data-root.json");
            json!({
                "sid":sid,
                "profile":profile,
                "aggregate":aggregate,
                "dataRoot":data,
                "opencodexHome":home,
                "codexRoot":codex,
                "aggregateExists":aggregate.is_dir(),
                "metadataExists":marker.is_file(),
                "metadataSha256":fs::read(&marker).ok().map(|b|hash::sha256_hex(&b))
            })
        })
        .collect();
    let distinct = roots[0]["aggregate"] != roots[1]["aggregate"]
        && roots[0]["dataRoot"] != roots[1]["dataRoot"]
        && roots[0]["codexRoot"] != roots[1]["codexRoot"];
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(evidence.join("r2-two-accounts.jsonl"))
        .unwrap();
    std::io::Write::write_all(
        &mut file,
        format!(
            "{}\n",
            json!({"currentSid":token.sid,"accountCount":users.len(),"roots":roots,"distinct":distinct})
        )
        .as_bytes(),
    )
    .unwrap();
    file.sync_all().unwrap();
    assert!(distinct, "each account must have its own managed root");
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
        crate::modules::process::client_home(&home).join("config.toml"),
        crate::modules::process::client_home(&home).join("auth.json"),
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
    .with_data_root(data.clone());
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
    let normal = stopped.is_ok();
    event("clients-stopped-or-failure", snapshot());
    let restart = run(LifecycleAction::Restart);
    let second = run(LifecycleAction::Start);
    event(
        "native-vs-restoration",
        json!({"started":started_ok,"nativeStopped":normal,"fullStop":stopped.as_ref().is_ok_and(|r|r.result==Some(LifecycleResult::Stopped)),"restartStarted":restart.as_ref().is_ok_and(|r|r.result==Some(LifecycleResult::Started)),"secondStarted":second.as_ref().is_ok_and(|r|r.result==Some(LifecycleResult::Started)),"declaredSyntheticClientsUnchanged":snapshot()==before,"managerRestartContinuityVerified":false}),
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
