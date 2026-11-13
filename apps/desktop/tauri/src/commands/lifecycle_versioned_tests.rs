//! Exercise the production action boundary with a current-format selection,
//! never a second flat-layout fixture or an alternate lifecycle implementation.
use super::*;
use crate::modules::runtime::{program_selection::tests::Fixture, RuntimeHandle};
use std::sync::atomic::{AtomicUsize, Ordering};
#[path = "lifecycle_loopback_models_fixture.rs"]
mod loopback_models;
#[path = "lifecycle_root_cycles_fixture.rs"]
mod root_cycles;

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
    let fresh_clients = std::env::var("OCXD024_FRESH_CLIENTS").unwrap_or_default();
    assert!(
        !h.join("config.json").exists() && !client.join("config.toml").exists(),
        "native fixture must not overwrite preexisting configuration"
    );
    for path in [&data, &h] {
        crate::infrastructure::platform::create_private_dir_all(path, 0o700).unwrap();
    }
    if fresh_clients != "codex-absent" {
        crate::infrastructure::platform::create_private_dir_all(&client, 0o700).unwrap();
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
    let mut local_discovery = (std::env::var_os("OCXD024_DYNAMIC_SELECTED").as_deref()
        == Some(std::ffi::OsStr::new("1")))
    .then(loopback_models::Server::start);
    let mut config = serde_json::json!({"hostname":"127.0.0.1","port":0,"openaiProviderTierVersion":2,"defaultProvider":"native-fixture","defaultModelAliases":false,"providers":{"native-fixture":{"adapter":"openai-chat","baseUrl":"http://127.0.0.1:1/v1","allowPrivateNetwork":true,"apiKey":"synthetic-unused","models":["native-fixture-model"],"selectedModels":["native-fixture-model"],"liveModels":false,"defaultAliases":false}}});
    if std::env::var_os("OCXD024_ALIAS_GRAPH").as_deref() == Some(std::ffi::OsStr::new("1")) {
        config["defaultModelAliases"] = true.into();
        config["fastRows"] = true.into();
        let p = &mut config["providers"]["native-fixture"];
        p["models"] = serde_json::json!(["claude-opus-5-5", "gpt-6-sol", "vendor/gemini-3.5-pro"]);
        p["selectedModels"] = p["models"].clone();
        p.as_object_mut().unwrap().remove("defaultAliases");
        p["modelAliases"] = serde_json::json!({"gpt-6-sol":"custom-sol"});
    }
    if let Some(server) = &local_discovery {
        let p = &mut config["providers"]["native-fixture"];
        p["baseUrl"] = server.base_url().into();
        p["liveModels"] = true.into();
        p["models"] = serde_json::json!(["stale-known-model"]);
        p["selectedModels"] =
            serde_json::json!(["claude-opus-5-5", "gpt-6-sol", "vendor/gemini-3.5-pro"]);
    }
    if std::env::var_os("OCXD024_STATIC_ALL_GRAPH").as_deref() == Some(std::ffi::OsStr::new("1")) {
        let p = &mut config["providers"]["native-fixture"];
        p["liveModels"] = false.into();
        p["models"] = serde_json::json!(["claude-opus-5-5", "gpt-6-sol"]);
        p["retainModels"] = serde_json::json!(["gpt-6-sol", "vendor/gemini-3.5-pro"]);
        p["selectedModels"] = serde_json::json!([]);
        p["defaultModel"] = "gpt-6-sol".into();
        config["disabledModels"] = serde_json::json!(["native-fixture/gpt-6-sol"]);
    }
    if std::env::var_os("OCXD024_CUSTOM_GRAPH").as_deref() == Some(std::ffi::OsStr::new("1")) {
        let p = &mut config["providers"]["native-fixture"];
        p["liveModels"] = false.into();
        p["models"] = serde_json::json!(["gpt-6-sol"]);
        p["selectedModels"] = serde_json::json!([]);
        config["customModels"] = serde_json::json!([
            {"provider":"native-fixture","modelId":"gpt-6-sol","displayName":"Custom base","contextWindow":196608,"inputModalities":["text"],"reasoningEfforts":["low","medium"]},
            {"provider":"native-fixture","modelId":"vendor/custom-native","displayName":"Custom added","contextWindow":262144,"inputModalities":["text"]},
            {"provider":"native-fixture","modelId":"custom-disabled","displayName":"Excluded custom"}
        ]);
        config["disabledModels"] = serde_json::json!(["native-fixture/custom-disabled"]);
    }
    if std::env::var_os("OCXD024_COMBO_GRAPH").as_deref() == Some(std::ffi::OsStr::new("1")) {
        let p = &mut config["providers"]["native-fixture"];
        p["liveModels"] = false.into();
        p["models"] = serde_json::json!(["alpha-fixture", "beta-fixture"]);
        p["selectedModels"] = serde_json::json!([]);
        config["customModels"] = serde_json::json!([
            {"provider":"native-fixture","modelId":"alpha-fixture","contextWindow":196608,"inputModalities":["text","image"],"reasoningEfforts":["low","medium"]},
            {"provider":"native-fixture","modelId":"beta-fixture","contextWindow":131072,"inputModalities":["text"],"reasoningEfforts":["medium","high"]}
        ]);
        config["combos"] = serde_json::json!({
            "mixed":{"strategy":"failover","displayName":"Mixed fixture","targets":[{"provider":"native-fixture","model":"alpha-fixture"},{"provider":"native-fixture","model":"beta-fixture"}]},
            "named":{"alias":"Public/fixture","displayName":"Public fixture","targets":[{"provider":"native-fixture","model":"alpha-fixture"}]},
            "hidden":{"alias":"Public/hidden","targets":[{"provider":"native-fixture","model":"beta-fixture"}]}
        });
        config["disabledModels"] = serde_json::json!(["Public/hidden"]);
    }
    atomic_write(
        &h.join("config.json"),
        &serde_json::to_vec(&config).unwrap(),
        0o600,
    )
    .unwrap();
    assert!(matches!(
        fresh_clients.as_str(),
        "" | "grok-absent"
            | "codex-empty"
            | "grok-first-backup"
            | "catalog-absent"
            | "codex-absent"
    ));
    let grok = profile.join(".grok");
    if !matches!(fresh_clients.as_str(), "codex-empty" | "codex-absent") {
        atomic_write(
            &client.join("config.toml"),
            br#"model = "gpt-5.4"
model_provider = "openai"
"#,
            0o600,
        )
        .unwrap();
        if fresh_clients != "catalog-absent" {
            let catalog = serde_json::json!({"models":[{"slug":"gpt-5.4","display_name":"Native fixture model","description":"synthetic native record","visibility":"list","supported_in_api":true,"priority":0}]});
            atomic_write(
                &client.join("opencodex-catalog.json"),
                &serde_json::to_vec(&catalog).unwrap(),
                0o600,
            )
            .unwrap();
        }
    }
    if fresh_clients.is_empty() || fresh_clients == "grok-first-backup" {
        // This round exercises existing Grok input, not merely backup absence.
        // It is an owned synthetic profile; no installed Grok client is invoked.
        crate::infrastructure::platform::create_private_dir_all(&grok.join("agents"), 0o700)
            .unwrap();
        atomic_write(
            &grok.join("config.toml"),
            b"[model.native_fixture]\nmodel = 'native-grok-fixture'\nmodel_provider = 'native'\n",
            0o600,
        )
        .unwrap();
        if fresh_clients != "grok-first-backup" {
            atomic_write(
                &grok.join("config.toml.bak-opencodex"),
                b"prior synthetic Grok backup\n",
                0o600,
            )
            .unwrap();
        }
        atomic_write(
            &grok.join("agents/AGENTS.md"),
            b"synthetic Grok agent guidance\n",
            0o600,
        )
        .unwrap();
    }
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
    let unwind_cleanup = root_cycles::StopOnUnwind::new(&context, &runner, &log, &evidence);
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
    report["grokConfigChangedDuringStart"] =
        (std::fs::read(grok.join("config.toml")).ok() != before["grok/config.toml"]).into();
    report["initialClientState"] = fresh_clients.clone().into();
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
    if std::env::var_os("OCXD024_CAPTURE_INTERRUPT_POINT").is_some() {
        assert!(!started_ok && record.is_none());
        assert_eq!(
            after_start.state,
            crate::infrastructure::macos_client_originals::session_index::State::CaptureReserved
        );
        assert!(after_start.references.is_none());
        assert!(
            start.as_ref().unwrap().client_recovery_required && !start.as_ref().unwrap().can_start
        );
        report["reservedCaptureInterruptionExpected"] = true.into();
        report["reservedAttempt"] = after_start.ticket.clone().into();
        report["indexRevision"] = after_start.revision.into();
        report["indexState"] = serde_json::json!(after_start.state);
        report["result"] = "failed".into();
        report["statusAndStopSkipped"] = "directed capture-only failure; no service started".into();
        write(&report);
        panic!(
            "expected owned reserved capture interruption at {}",
            root.display()
        );
    }
    if std::env::var_os("OCXD024_INTERRUPT_BEFORE_CAPTURE").as_deref()
        == Some(std::ffi::OsStr::new("1"))
    {
        assert!(!started_ok && record.is_none());
        assert_eq!(
            after_start.state,
            crate::infrastructure::macos_client_originals::session_index::State::Preparing
        );
        assert!(after_start.references.is_none());
        assert!(!data.join("manager-state/client-originals").exists());
        assert!(
            start.as_ref().unwrap().client_recovery_required && !start.as_ref().unwrap().can_start
        );
        report["preparationInterruptionExpected"] = true.into();
        report["indexedState"] = serde_json::json!(after_start.state);
        report["indexedRevision"] = after_start.revision.into();
        report["result"] = "failed".into();
        report["statusAndStopSkipped"] = "directed pre-capture fixture, no service startup".into();
        write(&report);
        panic!(
            "expected owned Preparing interruption retained at {}",
            root.display()
        );
    }
    if std::env::var_os("OCXD024_INTERRUPT_AFTER_CAPTURE").as_deref()
        == Some(std::ffi::OsStr::new("1"))
    {
        assert!(!started_ok);
        assert_eq!(
            after_start.state,
            crate::infrastructure::macos_client_originals::session_index::State::Captured
        );
        assert!(record.is_none());
        report["captureInterruptionExpected"] = true.into();
        report["indexedState"] = serde_json::json!(after_start.state);
        report["indexedRevision"] = after_start.revision.into();
        report["result"] = "failed".into();
        report["statusAndStopSkipped"] =
            "directed capture-only fixture; no service generation started".into();
        write(&report);
        panic!(
            "expected owned capture interruption retained at {}",
            root.display()
        );
    }
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
            serde_json::json!({"status":value["status"],"ready":value["ready"],"pid":value["pid"],"port":value["port"],"statusObservation":value["statusObservation"],"officialReadiness":value["officialReadiness"],"opencodexHome":value["opencodexHome"]})
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
            && head.revision == 12;
        report["singleRestart"] = serde_json::json!({"result":result,"elapsedMs":elapsed,"oldPid":old,"newPid":new,"newPort":current.as_ref().map(|v|v["port"].clone()),"indexState":head.state,"indexRevision":head.revision,"verified":restart_verified});
        write(&report);
    }
    let stopped_at = Instant::now();
    unwind_cleanup.stop_attempted();
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
    // Preserve the exact failed receipt before any gate assertion can unwind.
    report["stop"] = match &stop {
        Ok(value) => serde_json::to_value(value).unwrap(),
        Err(error) => serde_json::json!({"error":error.to_string()}),
    };
    write(&report);
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
    if std::env::var_os("OCXD024_RETIRED_METADATA_FINALIZE").as_deref()
        == Some(std::ffi::OsStr::new("1"))
    {
        use crate::infrastructure::macos_client_originals::session_index::{self, State};
        assert_eq!(fresh_clients, "codex-absent");
        assert!(
            matches!(&stop,Ok(v) if v.result.is_none()&&v.client_recovery_required&&v.failure.is_some()&&v.lifecycle_state==ProcessLifecycleState::Stopped)
        );
        assert_eq!(after_stop.state, State::RootRetirementPrepared);
        assert!(!client.exists());
        report["retiredPendingBeforeFinalize"] = serde_json::to_value(&after_stop).unwrap();
        let previous_events: Vec<serde_json::Value> = {
            let c = indexed_after_stop.fixture_existing_connection().unwrap();
            let mut statement = c
                .prepare("SELECT document FROM events ORDER BY revision")
                .unwrap();
            statement
                .query_map([], |row| row.get::<_, Vec<u8>>(0))
                .unwrap()
                .map(|r| serde_json::from_slice(&r.unwrap()).unwrap())
                .collect()
        };
        drop(indexed_after_stop);
        let preview = session_index::preview::inspect_preview(&context, &data);
        assert!(preview.can_finalize && !preview.can_continue && !preview.can_rollback);
        let request = session_index::finalization::Request {
            attempt_id: preview.attempt_id.clone().unwrap(),
            revision: preview.revision.unwrap(),
            confirmation_digest: preview.confirmation_digest.clone().unwrap(),
        };
        report["retiredFinalizePreview"] = serde_json::to_value(&preview).unwrap();
        write(&report);
        let began = Instant::now();
        let completed = session_index::finalization::finalize(&context, &data, request.clone());
        report["retiredFinalizeElapsedMs"] = serde_json::json!(began.elapsed().as_millis());
        report["retiredFinalizeOutcome"] = match &completed {
            Ok(v) => serde_json::to_value(v).unwrap(),
            Err(e) => serde_json::json!({"error":e.to_string()}),
        };
        write(&report);
        let completed = completed.unwrap();
        assert_eq!(completed.status, "restored");
        assert_eq!(completed.revision, after_stop.revision + 1);
        assert!(report["retiredFinalizeElapsedMs"].as_u64().unwrap() < 10000);
        let index = session_index::Index::open_existing(&context, &data, &client).unwrap();
        let closed = index.head().unwrap().unwrap();
        assert_eq!(closed.state, State::RootAbsent);
        assert_eq!(closed.revision, completed.revision);
        let current_events: Vec<serde_json::Value> = {
            let c = index.fixture_existing_connection().unwrap();
            let mut statement = c
                .prepare("SELECT document FROM events ORDER BY revision")
                .unwrap();
            statement
                .query_map([], |row| row.get::<_, Vec<u8>>(0))
                .unwrap()
                .map(|r| serde_json::from_slice(&r.unwrap()).unwrap())
                .collect()
        };
        assert!(current_events.starts_with(&previous_events));
        report["retiredClosedAfterFinalize"] = serde_json::to_value(&closed).unwrap();
        report["retiredFinalizeEvents"] = serde_json::json!(current_events);
        capture("after-retired-finalize");
        let final_artifacts: BTreeMap<_, _> = paths
            .iter()
            .map(|(name, path)| (*name, std::fs::read(path).ok() == before[*name]))
            .collect();
        assert!(final_artifacts.values().all(|v| *v));
        report["retiredFinalizeOriginalArtifactsRestored"] = serde_json::json!(final_artifacts);
        drop(index);
        write(&report);
        let began = Instant::now();
        let duplicate =
            session_index::finalization::finalize(&context, &data, request.clone()).unwrap();
        assert_eq!(duplicate.revision, completed.revision);
        assert_eq!(duplicate.status, "restored");
        assert!(began.elapsed().as_millis() < 10000);
        report["retiredDuplicateElapsedMs"] = serde_json::json!(began.elapsed().as_millis());
        report["retiredDuplicateOutcome"] = serde_json::to_value(&duplicate).unwrap();
        let index = session_index::Index::open_existing(&context, &data, &client).unwrap();
        assert_eq!(index.head().unwrap(), Some(closed));
        drop(index);
        capture("after-retired-duplicate");
        let duplicate_artifacts: BTreeMap<_, _> = paths
            .iter()
            .map(|(name, path)| (*name, std::fs::read(path).ok() == before[*name]))
            .collect();
        assert!(duplicate_artifacts.values().all(|v| *v));
        report["retiredDuplicateOriginalArtifactsRestored"] =
            serde_json::json!(duplicate_artifacts);
        std::env::set_var("OCXD024_RETIRED_DRIFT_AFTER_IMAGE", "1");
        let drift = session_index::finalization::finalize(&context, &data, request);
        std::env::remove_var("OCXD024_RETIRED_DRIFT_AFTER_IMAGE");
        report["retiredPostImageDriftRefused"] = drift.is_err().into();
        assert!(drift.is_err());
        assert_eq!(
            std::fs::read(context.working_directory.join(".grok/config.toml")).unwrap(),
            b"owned-test-drift-after-image=1\n"
        );
        assert!(!client.exists() && !h.join("runtime-port.json").exists());
        report["result"] = "passed".into();
        report["scope"]="directed explicit retired metadata finalization and exact duplicate retry; original Stop remains failed by deliberate test interruption; not normal two-cycle acceptance".into();
        write(&report);
        println!("OFFICIAL299_RETIRED_METADATA_FINALIZE {report}");
        return;
    }
    if fresh_clients == "codex-absent" {
        let root_absent = after_stop.state
            == crate::infrastructure::macos_client_originals::session_index::State::RootAbsent;
        report["scope"]="diagnostic first-root Start/Stop only; next Start and interrupted root recovery remain unaccepted".into();
        report["accepted"] = false.into();
        report["rootAbsentTerminal"] =
            (root_absent && !client.exists() && after_stop.completion_sha256.is_some()).into();
        report["indexedAfterStop"] = serde_json::to_value(&after_stop).unwrap();
        write(&report);
        assert!(root_absent&&!client.exists()&&after_stop.completion_sha256.is_some(),"first-root Stop must commit exact original absence; preserve report and app log on refusal");
        let _archive = indexed_after_stop
            .archived_root_originals(&after_stop)
            .unwrap();
        let archive_path = data
            .join("manager-state/client-originals")
            .join(&after_stop.ticket)
            .join(format!(
                "root-retired-{}",
                after_stop.root_retirement.as_ref().unwrap().plan.id
            ));
        assert!(!archive_path.join("config.toml.ocx-write.lock").exists());
        let raw = std::fs::read(
            data.join("manager-state/client-originals")
                .join(&after_stop.ticket)
                .join("original.json"),
        )
        .unwrap();
        let original: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(original["schemaVersion"], 5);
        assert!(original["originalRootAbsence"].is_object());
        assert_eq!(
            after_stop.references.as_ref().unwrap().original_sha256,
            crate::infrastructure::hash::sha256_hex(&raw)
        );
        assert!(restored.values().all(|v| *v));
        report["originalAbsentProvenanceAndReadback"] = true.into();
        report["result"] = "failed".into();
        write(&report);
        assert!(stop_ok,"RootAbsent alone cannot replace normal Stop success including coordination release and final source checks");
        assert!(
            matches!(&stop,Ok(v) if v.lifecycle_state==ProcessLifecycleState::Stopped && v.failure.is_none() && !v.client_recovery_required && v.can_start && !v.can_stop && !v.can_restart)
        );
        assert!(matches!(&start,Ok(v) if v.result==Some(LifecycleResult::Started)));
        assert!(report["startElapsedMs"].as_u64().unwrap() < 30000);
        assert!(report["stopElapsedMs"].as_u64().unwrap() < 10000);
        assert!(!h.join("runtime-port.json").exists() && !h.join("ocx.pid").exists());
        report["result"] = if std::env::var_os("OCXD024_ROOT_CYCLES").as_deref()
            == Some(std::ffi::OsStr::new("2"))
        {
            "failed"
        } else {
            "passed"
        }
        .into();
        write(&report);
        if std::env::var_os("OCXD024_ROOT_CYCLES").as_deref() == Some(std::ffi::OsStr::new("2")) {
            let first_status_ok = matches!(&status,Ok(v) if v["status"]=="running" && v["ready"]==true
                && record.as_ref().is_some_and(|r|r["pid"].as_u64().is_some_and(|n|n>0)&&r["port"].as_u64().is_some_and(|n|n>0))
                && v["statusObservation"]=="attested-same-connection" && v["officialReadiness"]=="ready"
                && v["pid"].as_str().and_then(|v|v.parse::<u64>().ok())==record.as_ref().and_then(|r|r["pid"].as_u64())
                && v["port"].as_u64()==record.as_ref().and_then(|r|r["port"].as_u64())
                && v["opencodexHome"].as_str()==h.to_str());
            report["firstAttestedStatusGate"] = first_status_ok.into();
            write(&report);
            assert!(
                first_status_ok
                    && report["indexedReady"] == true
                    && report["statusElapsedMs"].as_u64().unwrap() < 5000
            );
            report["result"] = "failed".into();
            report["scope"] = "directed two-generation root lifecycle; fixed application candidate and whole version unaccepted".into();
            write(&report);
            let cycles = root_cycles::run(root_cycles::Request {
                context: &context,
                runner: &runner,
                log: &log,
                evidence: &evidence,
                first_index: indexed_after_stop,
                first_head: after_stop,
                first_report: &report,
            });
            let passed = cycles["passed"] == true;
            report["rootCycles"] = cycles;
            report["result"] = if passed { "passed" } else { "failed" }.into();
            write(&report);
            assert!(passed,"two-generation original lifecycle did not satisfy every gate; retain root-cycles.json and app log");
            println!("OFFICIAL299_TWO_ROOT_CYCLES_DIAGNOSTIC {report}");
            return;
        }
        println!("OFFICIAL299_FIRST_ROOT_DIAGNOSTIC {report}");
        return;
    }
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
    // Persist lifecycle facts before supplemental archive assertions: a fixture
    // assumption must not hide the actual product Stop result.
    write(&report);
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
        let original_path = data
            .join("manager-state/client-originals")
            .join(&refs.attempt)
            .join("original.json");
        let raw = std::fs::read(original_path).unwrap();
        assert_eq!(
            crate::infrastructure::hash::sha256_hex(&raw),
            refs.original_sha256
        );
        let frame: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        let grok_state = frame["grok"]["state"].as_str().unwrap();
        assert!(matches!(grok_state, "absent" | "directory"));
        for relative in [
            "config.toml",
            "config.toml.bak-opencodex",
            "agents/AGENTS.md",
        ] {
            let expected = &before[format!("grok/{relative}").as_str()];
            let entry = if grok_state == "directory" {
                frame["grok"]["tree"]["entries"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|e| e["path"] == relative)
            } else {
                None
            };
            let archived = originals.archived_grok_file(
                crate::infrastructure::macos_client_originals::Phase::Original,
                relative,
            );
            if let Some(bytes) = expected {
                let entry =
                    entry.expect("present original must have an independently recorded file");
                assert_eq!(entry["kind"], "file");
                assert_eq!(
                    entry["sha256"],
                    crate::infrastructure::hash::sha256_hex(bytes)
                );
                assert_eq!(archived.unwrap().as_slice(), bytes);
            } else {
                assert!(
                    entry.is_none(),
                    "original absence must be recorded, not an archive read error"
                );
                assert!(
                    archived.is_err(),
                    "no absent original may gain archived file bytes"
                );
            }
        }
        report["originalGrokPresentAndAbsentPathsVerified"] = true.into();
        report["originalGrokConfigBackupAndAgentArchived"] = [
            "grok/config.toml",
            "grok/config.toml.bak-opencodex",
            "grok/agents/AGENTS.md",
        ]
        .iter()
        .all(|p| before[*p].is_some())
        .into();
        report["originalGrokRootState"] = grok_state.into();
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
            && head.revision == 16
            && head.completion_sha256.is_some();
        report["secondSession"] = serde_json::json!({"start":second_start,"stop":second_stop,"startElapsedMs":second_start_ms,"stopElapsedMs":second_stop_ms,"allOriginalBytesRestored":all_restored,"indexRevision":head.revision,"newTicket":head.ticket!=after_stop.ticket,"result":if repeat_ok{"passed"}else{"failed"}});
        passed &= repeat_ok && head.ticket != after_stop.ticket;
    }
    if let Some(mut server) = local_discovery.take() {
        let discovery = server.finish();
        passed &= discovery["getModelsRequests"]
            .as_u64()
            .is_some_and(|n| n > 0)
            && discovery["unexpectedOrBusinessRequests"] == 0;
        report["localModelDiscovery"] = discovery;
    }
    report["result"] = if passed { "passed" } else { "failed" }.into();
    write(&report);
    println!("OFFICIAL299_NATIVE_LIFECYCLE {report}");
    assert!(
        passed,
        "production native lifecycle failed; owned fixture retained at {}",
        root.display()
    );
    if std::env::var_os("OCXD024_HISTORY_PREPARING").as_deref() == Some(std::ffi::OsStr::new("1")) {
        let start = Instant::now();
        let result = process_action_with_runner(
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
        let elapsed = start.elapsed().as_millis();
        let index =
            crate::infrastructure::macos_client_originals::session_index::Index::open_existing(
                &context, &data, &client,
            )
            .unwrap();
        let head = index.head().unwrap().unwrap();
        drop(index);
        assert_eq!(
            head.state,
            crate::infrastructure::macos_client_originals::session_index::State::Preparing
        );
        assert_eq!(head.revision, 9);
        assert!(head.references.is_none());
        assert!(result.client_recovery_required && !result.can_start);
        assert!(paths
            .iter()
            .all(|(name, path)| std::fs::read(path).ok() == before[*name]));
        let report = serde_json::json!({"accepted":false,"precedingNativeLifecycle":"summary.json","expectedPreparingInterruption":true,"elapsedMs":elapsed,"result":result,"indexState":head.state,"indexRevision":head.revision,"historicalVaultRetained":data.join("manager-state/client-originals").exists(),"currentClientsUnchanged":true});
        atomic_write(
            &evidence.join("historical-preparing.json"),
            &serde_json::to_vec_pretty(&report).unwrap(),
            0o600,
        )
        .unwrap();
        panic!(
            "expected historical Preparing interruption retained at {}",
            root.display()
        );
    }
}
