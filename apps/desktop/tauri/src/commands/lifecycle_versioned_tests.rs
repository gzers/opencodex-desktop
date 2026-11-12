//! Exercise the production action boundary with a current-format selection,
//! never a second flat-layout fixture or an alternate lifecycle implementation.
use super::*;
use crate::modules::runtime::{program_selection::tests::Fixture, RuntimeHandle};
use std::sync::atomic::{AtomicUsize, Ordering};

struct AdmissionProbe(AtomicUsize);
#[test]
fn managed_restart_has_original_entry_stop_deadline_inside_original_total_budget() {
    let started = std::time::Instant::now();
    let (outer, stop) = managed_lifecycle_deadlines(LifecycleAction::Restart, started);
    assert_eq!(outer - started, std::time::Duration::from_secs(30));
    assert_eq!(stop - started, std::time::Duration::from_secs(10));
    let delayed = started + std::time::Duration::from_secs(7);
    assert_eq!(stop - delayed, std::time::Duration::from_secs(3));
    assert!(
        stop < delayed + crate::modules::process::stop_timeout(),
        "pre-stop admission cannot reset Stop's clock"
    );
    let (outer, stop) = managed_lifecycle_deadlines(LifecycleAction::Stop, started);
    assert_eq!(outer, stop);
}

#[test]
fn an_error_code_from_a_runner_cannot_forge_a_stopped_native_observation() {
    struct CodeOnly;
    impl ProcessRunner for CodeOnly {
        fn execute(
            &self,
            _: &crate::modules::process::ProcessCommand,
        ) -> AppResult<crate::modules::process::LifecycleResult> {
            Err(AppError::RuntimeManaged {
                code: "client_original_restore_pending".into(),
                detail: "synthetic code only, no native stop".into(),
            })
        }
        fn cancel_pending(&self) -> AppResult<()> {
            Ok(())
        }
    }
    let temp = tempfile::tempdir().unwrap();
    let context = ProcessContext {
        runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(
            temp.path().join("not-executed"),
        ),
        working_directory: temp.path().to_path_buf(),
        opencodex_home: temp.path().join("not-opened-home"),
    };
    let log = crate::infrastructure::runtime_log::RuntimeLog::new(temp.path());
    for action in [LifecycleAction::Stop, LifecycleAction::Restart] {
        let result = process_action_with_runner(
            ProcessActionRequest {
                action,
                confirm: true,
            },
            &Mutex::new(CodeOnly),
            &context,
            &log,
            None,
        )
        .unwrap();
        assert_eq!(result.lifecycle_state, ProcessLifecycleState::Running);
        assert!(!result.client_recovery_required);
        assert!(result.result.is_none() && result.failure.is_some());
    }
    assert!(!context.opencodex_home.exists());
}
impl ProcessRunner for AdmissionProbe {
    fn execute(
        &self,
        _: &crate::modules::process::ProcessCommand,
    ) -> AppResult<crate::modules::process::LifecycleResult> {
        panic!("a managed action must use its admitted runtime context")
    }
    fn execute_in_context(
        &self,
        command: &crate::modules::process::ProcessCommand,
        runtime: &crate::infrastructure::runtime_executable::SharedRuntimeExecutable,
        deadline: std::time::Instant,
    ) -> AppResult<crate::modules::process::LifecycleResult> {
        assert_eq!(command.action, LifecycleAction::Start);
        assert_eq!(Some(command.executable.clone()), runtime.executable());
        assert!(std::time::Instant::now() < deadline);
        self.0.fetch_add(1, Ordering::SeqCst);
        // An admission test grants no service-success authority.
        Err(AppError::RuntimeManaged {
            code: "fixture_admission_reached".into(),
            detail: "deliberate non-executing runner boundary".into(),
        })
    }
    fn cancel_pending(&self) -> AppResult<()> {
        Ok(())
    }
}

#[test]
fn current_versioned_action_reaches_context_runner_without_flat_runtime() {
    let f = Fixture::new();
    let data = f.root.join("D/desktop");
    let home = f.root.join("profile/opencodex-home");
    std::fs::create_dir_all(&home).unwrap();
    let runtime = RuntimeHandle::initialize_versioned(&data, &f.root);
    let context = ProcessContext {
        runtime,
        working_directory: f.root.clone(),
        opencodex_home: home.clone(),
    };
    let runner = AdmissionProbe(AtomicUsize::new(0));
    let command = crate::modules::process::ProcessCommand::new(
        LifecycleAction::Start,
        context.executable().unwrap(),
        f.root.clone(),
        home,
    );
    assert!(!f.root.join("runtime/opencodex").exists());
    let result = execute_lifecycle(&runner, &command, &context, &data);
    assert!(
        matches!(result, Err(AppError::RuntimeManaged { ref code, .. }) if code == "fixture_admission_reached"),
        "{result:?}"
    );
    assert_eq!(runner.0.load(Ordering::SeqCst), 1);
    assert!(!f.root.join("runtime/opencodex").exists());
}

#[test]
fn changed_versioned_program_is_refused_before_port_write_or_spawn() {
    let f = Fixture::new();
    let data = f.root.join("D/desktop");
    let home = f.root.join("profile/opencodex-home");
    std::fs::create_dir_all(&home).unwrap();
    let runtime = RuntimeHandle::initialize_versioned(&data, &f.root);
    let context = ProcessContext {
        runtime,
        working_directory: f.root.clone(),
        opencodex_home: home.clone(),
    };
    let command = crate::modules::process::ProcessCommand::new(
        LifecycleAction::Start,
        context.executable().unwrap(),
        f.root.clone(),
        home.clone(),
    );
    std::fs::write(&f.paths.node, b"changed after admission").unwrap();
    let runner = AdmissionProbe(AtomicUsize::new(0));
    assert!(execute_lifecycle(&runner, &command, &context, &data).is_err());
    assert_eq!(runner.0.load(Ordering::SeqCst), 0);
    assert!(!home.join("config.json").exists());
    assert!(!data
        .join("manager-state/runtime-port-settings.json")
        .exists());
}

/// Production orchestration against the immutable official distribution. The
/// launcher must give this one test a private OS profile and kernel boundary;
/// test failure keeps the fixture for owner-scoped cleanup and original evidence.
#[test]
#[ignore = "explicit isolated macOS profile, official299 bundle and evidence root required"]
fn official299_versioned_start_status_stop_preserves_native_client() {
    use crate::{
        infrastructure::{atomic_write::atomic_write, process_runner::ControlledProcessRunner},
        modules::{process::LifecycleResult, runtime, status::StatusSource},
    };
    use std::{collections::BTreeMap, path::PathBuf, time::Instant};
    let root = PathBuf::from(std::env::var_os("OCXD024_NATIVE_ROOT").expect("private root"));
    assert!(root.is_absolute() && root.canonicalize().unwrap() == root);
    assert!(
        root.starts_with("/private/tmp")
            && root
                .parent()
                .and_then(|p| p.file_name())
                .is_some_and(|name| name
                    .to_string_lossy()
                    .starts_with("ocxd024-versioned-lifecycle-")),
        "a disposable task-owned root is required"
    );
    let profile = root.join("profile");
    assert_eq!(
        std::env::var_os("HOME"),
        Some(profile.clone().into_os_string())
    );
    assert_eq!(
        std::env::var_os("CODEX_HOME"),
        Some(profile.join(".codex").into_os_string())
    );
    let evidence = root.join("evidence/native");
    std::fs::create_dir_all(root.join("evidence")).unwrap();
    std::fs::create_dir(&evidence).expect("fresh evidence directory");
    let reuse = std::env::var_os("OCXD024_RUNTIME_REUSE").map(PathBuf::from);
    let program = reuse
        .clone()
        .unwrap_or_else(|| root.join("program 中文 space"));
    if reuse.is_some() {
        assert!(
            program.is_absolute()
                && program.canonicalize().unwrap() == program
                && program.starts_with("/private/tmp")
                && program.ancestors().any(|p| p.file_name().is_some_and(|n| n
                    .to_string_lossy()
                    .starts_with("ocxd024-versioned-lifecycle-")))
        );
    }
    let data = profile.join(".codex-desktop/desktop");
    let h = profile.join(".codex-desktop/opencodex/home");
    let client = profile.join(".codex");
    assert!(
        !h.join("config.json").exists() && !client.join("config.toml").exists(),
        "native fixture must not overwrite preexisting configuration"
    );
    for path in [&data, &h, &client] {
        crate::infrastructure::platform::create_private_dir_all(path, 0o700).unwrap();
    }
    std::fs::create_dir_all(data.join("logs")).unwrap();
    let descriptor = std::fs::read(root.join("resources/runtime-bundle.manifest.json")).unwrap();
    let pin = crate::infrastructure::hash::sha256_hex(&descriptor);
    assert_eq!(
        pin,
        "e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3"
    );
    if reuse.is_none() {
        runtime::bundled::versioned::provision(
            &program,
            &data,
            &root.join("resources/runtime-bundle-0.2.24-darwin-arm64.tar.gz"),
            &descriptor,
            &pin,
        )
        .unwrap();
    }
    let selected = runtime::program_selection::read(
        &program,
        crate::modules::data_root::layout::Platform::Macos,
        "darwin-arm64",
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        selected.image.official_archive_sha256,
        "b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c"
    );
    runtime::program_image::verify(&selected).unwrap();
    let config = serde_json::json!({"hostname":"127.0.0.1","port":0,"openaiProviderTierVersion":2,"defaultProvider":"native-fixture","defaultModelAliases":false,"providers":{"native-fixture":{"adapter":"openai-chat","baseUrl":"http://127.0.0.1:1/v1","allowPrivateNetwork":true,"apiKey":"synthetic-unused","models":["native-fixture-model"],"selectedModels":["native-fixture-model"],"liveModels":false,"defaultAliases":false}}});
    atomic_write(
        &h.join("config.json"),
        &serde_json::to_vec(&config).unwrap(),
        0o600,
    )
    .unwrap();
    atomic_write(
        &client.join("config.toml"),
        br#"model = "gpt-5.4"
model_provider = "openai"
"#,
        0o600,
    )
    .unwrap();
    let catalog = serde_json::json!({"models":[{"slug":"gpt-5.4","display_name":"Native fixture model","description":"synthetic native record","visibility":"list","supported_in_api":true,"priority":0}]});
    atomic_write(
        &client.join("opencodex-catalog.json"),
        &serde_json::to_vec(&catalog).unwrap(),
        0o600,
    )
    .unwrap();
    // This round exercises existing Grok input, not merely backup absence.
    // It is an owned synthetic profile; no installed Grok client is invoked.
    let grok = profile.join(".grok");
    crate::infrastructure::platform::create_private_dir_all(&grok.join("agents"), 0o700).unwrap();
    atomic_write(
        &grok.join("config.toml"),
        b"[model.native_fixture]\nmodel = 'native-grok-fixture'\nmodel_provider = 'native'\n",
        0o600,
    )
    .unwrap();
    atomic_write(
        &grok.join("config.toml.bak-opencodex"),
        b"prior synthetic Grok backup\n",
        0o600,
    )
    .unwrap();
    atomic_write(
        &grok.join("agents/AGENTS.md"),
        b"synthetic Grok agent guidance\n",
        0o600,
    )
    .unwrap();
    let paths = [
        ("codex/config.toml", client.join("config.toml")),
        (
            "codex/opencodex-catalog.json",
            client.join("opencodex-catalog.json"),
        ),
        ("codex/models_cache.json", client.join("models_cache.json")),
        (
            "codex/opencodex-catalog-audit.jsonl",
            client.join("opencodex-catalog-audit.jsonl"),
        ),
        (
            "grok/config.toml.bak-opencodex",
            profile.join(".grok/config.toml.bak-opencodex"),
        ),
        ("grok/config.toml", grok.join("config.toml")),
        ("grok/agents/AGENTS.md", grok.join("agents/AGENTS.md")),
    ];
    let before: BTreeMap<_, _> = paths
        .iter()
        .map(|(name, path)| (*name, std::fs::read(path).ok()))
        .collect();
    let capture = |label: &str| {
        use base64::Engine;
        let mut rows = Vec::new();
        for (name, path) in &paths {
            let bytes = match std::fs::read(path) {
                Ok(bytes) => Some(bytes),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => panic!("synthetic artifact capture is unreadable: {error}"),
            };
            assert!(bytes.as_ref().is_none_or(|b| b.len() <= 2 * 1024 * 1024));
            rows.push(serde_json::json!({"path":name,"present":bytes.is_some(),
                "sha256":bytes.as_ref().map(|b| crate::infrastructure::hash::sha256_hex(b)),
                "bytesBase64":bytes.as_ref().map(|b| base64::engine::general_purpose::STANDARD.encode(b))}));
        }
        atomic_write(
            &evidence.join(format!("{label}.json")),
            &serde_json::to_vec_pretty(&rows).unwrap(),
            0o600,
        )
        .unwrap();
    };
    capture("before");
    let runtime = RuntimeHandle::initialize_versioned(&data, &program);
    let context = ProcessContext {
        runtime,
        working_directory: profile.clone(),
        opencodex_home: h.clone(),
    };
    let runner = Mutex::new(ControlledProcessRunner::new(&profile));
    let log = crate::infrastructure::runtime_log::RuntimeLog::new(&data);
    let mut report = serde_json::json!({"accepted":false,"officialVersion":"2.99.0","serviceStarted":false,"startBudgetMs":30000,"statusBudgetMs":5000,"stopBudgetMs":10000,"businessRequests":false});
    let write = |report: &serde_json::Value| {
        atomic_write(
            &evidence.join("summary.json"),
            &serde_json::to_vec_pretty(report).unwrap(),
            0o600,
        )
        .unwrap()
    };
    write(&report);
    let started = Instant::now();
    let start = process_action_with_runner(
        ProcessActionRequest {
            action: LifecycleAction::Start,
            confirm: true,
        },
        &runner,
        &context,
        &log,
        None,
    );
    report["startElapsedMs"] = serde_json::json!(started.elapsed().as_millis());
    report["start"] = match &start {
        Ok(value) => serde_json::to_value(value).unwrap(),
        Err(error) => serde_json::json!({"error":error.to_string()}),
    };
    let started_ok = matches!(&start, Ok(value) if value.result == Some(LifecycleResult::Started));
    report["serviceStarted"] = started_ok.into();
    let record: Option<serde_json::Value> = std::fs::read(h.join("runtime-port.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok());
    // Never save the official attestation secret or admin token.
    report["record"] = record
        .as_ref()
        .map(|r| serde_json::json!({"pid":r["pid"],"port":r["port"]}))
        .unwrap_or(serde_json::Value::Null);
    report["clientChangedDuringStart"] =
        (std::fs::read(client.join("config.toml")).ok() != before["codex/config.toml"]).into();
    write(&report);
    capture("after-start");
    report["grokConfigChangedDuringStart"] = (std::fs::read(grok.join("config.toml")).unwrap()
        != b"[model.native_fixture]\nmodel = 'native-grok-fixture'\nmodel_provider = 'native'\n")
        .into();
    let indexed_after_start =
        crate::infrastructure::macos_client_originals::session_index::Index::open(
            &context, &data, &client,
        )
        .unwrap();
    let after_start = indexed_after_start.head().unwrap().unwrap();
    report["indexedReady"] = (after_start.state
        == crate::infrastructure::macos_client_originals::session_index::State::Ready)
        .into();
    report["independentOriginalsReferenced"] = after_start.references.is_some().into();
    drop(indexed_after_start);
    write(&report);
    let status_started = Instant::now();
    let source = crate::infrastructure::status_source::OfficialStatusSource::new(
        context.runtime.clone(),
        &profile,
        crate::modules::process::EnvironmentPolicy {
            home: Some(profile.clone().into_os_string()),
            opencodex_home: h.clone(),
            ..Default::default()
        },
    );
    // Production status executes on a worker entered in the application runtime.
    // A bare synchronous test thread has no Tokio handle and must not be used
    // to claim an endpoint observation failed.
    let status = tauri::async_runtime::block_on(async {
        tokio::task::spawn_blocking(move || source.fetch())
            .await
            .unwrap()
    });
    report["statusElapsedMs"] = serde_json::json!(status_started.elapsed().as_millis());
    report["status"] = match &status {
        Ok(value) => {
            serde_json::json!({"status":value["status"],"ready":value["ready"],"pid":value["pid"],"port":value["port"]})
        }
        Err(error) => serde_json::json!({"error":format!("{error:?}")}),
    };
    write(&report);
    // Even failed confirmation may have started our own service. Try the same
    // production Stop once; no direct signal or alternate client teardown.
    let mut restart_verified = true;
    if started_ok
        && std::env::var_os("OCXD024_SINGLE_RESTART").as_deref() == Some(std::ffi::OsStr::new("1"))
    {
        let t = Instant::now();
        let result = process_action_with_runner(
            ProcessActionRequest {
                action: LifecycleAction::Restart,
                confirm: true,
            },
            &runner,
            &context,
            &log,
            None,
        )
        .unwrap();
        let elapsed = t.elapsed().as_millis();
        let current: Option<serde_json::Value> = std::fs::read(h.join("runtime-port.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok());
        let index = crate::infrastructure::macos_client_originals::session_index::Index::open(
            &context, &data, &client,
        )
        .unwrap();
        let head = index.head().unwrap().unwrap();
        drop(index);
        let old = record.as_ref().and_then(|r| r["pid"].as_u64());
        let new = current.as_ref().and_then(|r| r["pid"].as_u64());
        restart_verified = result.result == Some(LifecycleResult::Started)
            && elapsed < 30000
            && old.is_some()
            && new.is_some()
            && old != new
            && head.state
                == crate::infrastructure::macos_client_originals::session_index::State::Ready
            && head.revision == 10;
        report["singleRestart"] = serde_json::json!({"result":result,"elapsedMs":elapsed,"oldPid":old,"newPid":new,"newPort":current.as_ref().map(|v|v["port"].clone()),"indexState":head.state,"indexRevision":head.revision,"verified":restart_verified});
        write(&report);
    }
    let stopped_at = Instant::now();
    let stop = process_action_with_runner(
        ProcessActionRequest {
            action: LifecycleAction::Stop,
            confirm: true,
        },
        &runner,
        &context,
        &log,
        None,
    );
    report["stopElapsedMs"] = serde_json::json!(stopped_at.elapsed().as_millis());
    if let Ok(value) = &stop {
        if value.client_recovery_required
            || value
                .failure
                .as_ref()
                .is_some_and(|e| e.code == "client_original_restore_pending")
        {
            let observed = value.lifecycle_state == ProcessLifecycleState::Stopped
                && value.client_recovery_required
                && value.result.is_none()
                && !value.can_start
                && !value.can_stop
                && !value.can_restart
                && value.failure.is_some();
            report["stoppedRecoveryObservation"] = observed.into();
            assert!(
                observed,
                "verified Stop with incomplete recovery must not remain running or claim success"
            );
        }
    }
    report["stop"] = match &stop {
        Ok(value) => serde_json::to_value(value).unwrap(),
        Err(error) => serde_json::json!({"error":error.to_string()}),
    };
    let stop_ok = matches!(&stop, Ok(value) if value.result == Some(LifecycleResult::Stopped));
    let restored: BTreeMap<_, _> = paths
        .iter()
        .map(|(name, path)| (*name, std::fs::read(path).ok() == before[*name]))
        .collect();
    report["nativeClientRestored"] = serde_json::to_value(&restored).unwrap();
    report["recordAbsent"] = (!h.join("runtime-port.json").exists()).into();
    report["pidRecordAbsent"] = (!h.join("ocx.pid").exists()).into();
    let indexed_after_stop =
        crate::infrastructure::macos_client_originals::session_index::Index::open(
            &context, &data, &client,
        )
        .unwrap();
    let after_stop = indexed_after_stop.head().unwrap().unwrap();
    report["indexedStopped"] = (after_stop.state
        == crate::infrastructure::macos_client_originals::session_index::State::Stopped)
        .into();
    report["indexedRestored"] = (after_stop.state
        == crate::infrastructure::macos_client_originals::session_index::State::Restored)
        .into();
    if stop_ok {
        assert_eq!(
            after_stop.state,
            crate::infrastructure::macos_client_originals::session_index::State::Restored
        );
        assert!(
            after_stop.completion_sha256.is_some(),
            "successful Stop requires independent completion digest"
        );
    }
    report["indexedRevision"] = serde_json::json!(after_stop.revision);
    report["indexedRestorePrepared"] = (after_stop.state
        == crate::infrastructure::macos_client_originals::session_index::State::RestorePrepared)
        .into();
    if let Some(plan) = &after_stop.restore_plan {
        let references = after_stop.references.as_ref().unwrap();
        let session = crate::infrastructure::macos_client_originals::session::StartupSession::open(
            &context, &data, &client, references,
        )
        .unwrap();
        session
            .validate_restore_plan(&context, &plan.id, &plan.sha256)
            .unwrap();
        report["restorePlanMatchesIndependentIndex"] = true.into();
    }
    if let Some(refs) = &after_stop.references {
        let originals = crate::infrastructure::macos_client_originals::Originals::open_for_profile(
            &data,
            &client,
            &profile,
            &refs.attempt,
            &refs.original_sha256,
            &refs.phases,
        )
        .unwrap();
        for (path, bytes) in [
            ("config.toml", b"[model.native_fixture]\nmodel = 'native-grok-fixture'\nmodel_provider = 'native'\n".as_slice()),
            (
                "config.toml.bak-opencodex",
                b"prior synthetic Grok backup\n".as_slice(),
            ),
            (
                "agents/AGENTS.md",
                b"synthetic Grok agent guidance\n".as_slice(),
            ),
        ] {
            assert_eq!(
                originals
                    .archived_grok_file(
                        crate::infrastructure::macos_client_originals::Phase::Original,
                        path
                    )
                    .unwrap()
                    .as_slice(),
                bytes
            );
        }
        report["originalGrokConfigBackupAndAgentArchived"] = true.into();
    }
    drop(indexed_after_stop);
    let reopened_at = Instant::now();
    let source = crate::infrastructure::status_source::OfficialStatusSource::new(
        context.runtime.clone(),
        &profile,
        crate::modules::process::EnvironmentPolicy {
            home: Some(profile.clone().into_os_string()),
            opencodex_home: h.clone(),
            ..Default::default()
        },
    )
    .with_client_recovery_root(data.clone());
    let reopened = tauri::async_runtime::block_on(async {
        tokio::task::spawn_blocking(move || {
            let mut collector = crate::modules::status::StatusCollector::new(source);
            status_snapshot_from_collector(&mut collector)
        })
        .await
        .unwrap()
    });
    report["reopenedStatusElapsedMs"] = serde_json::json!(reopened_at.elapsed().as_millis());
    let expects_pending = stop
        .as_ref()
        .is_ok_and(|value| value.client_recovery_required);
    let reopened_gate = matches!(&reopened,Ok(snapshot) if
        snapshot.matrix.runtime==crate::types::status::RuntimeState::Stopped
        && snapshot.port.is_none() && snapshot.pid.is_none()
        && snapshot.facts.data_root == data
        && snapshot.facts.opencodex_home.as_ref() == Some(&h)
        && !snapshot.can_stop && !snapshot.can_restart
        && if expects_pending {snapshot.facts.client_recovery==crate::modules::status::ClientRecoveryState::Required && !snapshot.can_start}
           else {snapshot.facts.client_recovery==crate::modules::status::ClientRecoveryState::NotRequired}
    ) && reopened_at.elapsed() < std::time::Duration::from_secs(5);
    report["reopenedRecoveryGate"] = reopened_gate.into();
    report["reopenedSnapshot"] = match &reopened {
        Ok(snapshot) => serde_json::to_value(snapshot).unwrap(),
        Err(error) => serde_json::json!({"error":error.to_string()}),
    };
    capture("after-stop");
    if let Ok(bytes) = std::fs::read(log.path()) {
        atomic_write(&evidence.join("runtime.log"), &bytes, 0o600).unwrap();
    }
    let passed = started_ok
        && restart_verified
        && reopened_gate
        && stop_ok
        && restored.values().all(|v| *v)
        && matches!(&status, Ok(value) if value["ready"]==true && value["status"]=="running")
        && report["recordAbsent"] == true
        && report["pidRecordAbsent"] == true
        && report["startElapsedMs"].as_u64().unwrap() < 30000
        && report["statusElapsedMs"].as_u64().unwrap() < 5000
        && report["stopElapsedMs"].as_u64().unwrap() < 10000;
    let mut passed = passed;
    if passed
        && std::env::var_os("OCXD024_REPEAT_SESSION").as_deref() == Some(std::ffi::OsStr::new("1"))
    {
        let again = Instant::now();
        let second_start = process_action_with_runner(
            ProcessActionRequest {
                action: LifecycleAction::Start,
                confirm: true,
            },
            &runner,
            &context,
            &log,
            None,
        )
        .unwrap();
        let second_start_ms = again.elapsed().as_millis();
        let stopping = Instant::now();
        let second_stop = process_action_with_runner(
            ProcessActionRequest {
                action: LifecycleAction::Stop,
                confirm: true,
            },
            &runner,
            &context,
            &log,
            None,
        )
        .unwrap();
        let second_stop_ms = stopping.elapsed().as_millis();
        let index = crate::infrastructure::macos_client_originals::session_index::Index::open(
            &context, &data, &client,
        )
        .unwrap();
        let head = index.head().unwrap().unwrap();
        let all_restored = paths
            .iter()
            .all(|(name, path)| std::fs::read(path).ok() == before[*name]);
        let repeat_ok = second_start.result == Some(LifecycleResult::Started)
            && second_stop.result == Some(LifecycleResult::Stopped)
            && second_start_ms < 30000
            && second_stop_ms < 10000
            && all_restored
            && head.state
                == crate::infrastructure::macos_client_originals::session_index::State::Restored
            && head.revision == 14
            && head.completion_sha256.is_some();
        report["secondSession"] = serde_json::json!({"start":second_start,"stop":second_stop,"startElapsedMs":second_start_ms,"stopElapsedMs":second_stop_ms,"allOriginalBytesRestored":all_restored,"indexRevision":head.revision,"newTicket":head.ticket!=after_stop.ticket,"result":if repeat_ok{"passed"}else{"failed"}});
        passed &= repeat_ok && head.ticket != after_stop.ticket;
    }
    report["result"] = if passed { "passed" } else { "failed" }.into();
    write(&report);
    println!("OFFICIAL299_NATIVE_LIFECYCLE {report}");
    assert!(
        passed,
        "production native lifecycle failed; owned fixture retained at {}",
        root.display()
    );
}
