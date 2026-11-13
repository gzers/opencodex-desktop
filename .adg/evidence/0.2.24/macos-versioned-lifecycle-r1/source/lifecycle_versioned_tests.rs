//! Exercise the production action boundary with a current-format selection,
//! never a second flat-layout fixture or an alternate lifecycle implementation.
use super::*;
use crate::modules::runtime::{program_selection::tests::Fixture, RuntimeHandle};
use std::sync::atomic::{AtomicUsize, Ordering};

struct AdmissionProbe(AtomicUsize);
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
    let program = root.join("program 中文 space");
    let data = profile.join(".codex-desktop/desktop");
    let h = profile.join(".codex-desktop/opencodex/home");
    let client = profile.join(".codex");
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
    runtime::bundled::versioned::provision(
        &program,
        &data,
        &root.join("resources/runtime-bundle-0.2.24-darwin-arm64.tar.gz"),
        &descriptor,
        &pin,
    )
    .unwrap();
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
    let paths = [
        "config.toml",
        "opencodex-catalog.json",
        "models_cache.json",
        "opencodex-catalog-audit.jsonl",
        "opencodex-grok-backup.json",
    ];
    let before: BTreeMap<_, _> = paths
        .iter()
        .map(|name| (*name, std::fs::read(client.join(name)).ok()))
        .collect();
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
        (std::fs::read(client.join("config.toml")).ok() != before["config.toml"]).into();
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
    let status = source.fetch();
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
    report["stop"] = match &stop {
        Ok(value) => serde_json::to_value(value).unwrap(),
        Err(error) => serde_json::json!({"error":error.to_string()}),
    };
    let stop_ok = matches!(&stop, Ok(value) if value.result == Some(LifecycleResult::Stopped));
    let restored: BTreeMap<_, _> = paths
        .iter()
        .map(|name| {
            (
                *name,
                std::fs::read(client.join(name)).ok() == before[*name],
            )
        })
        .collect();
    report["nativeClientRestored"] = serde_json::to_value(&restored).unwrap();
    report["recordAbsent"] = (!h.join("runtime-port.json").exists()).into();
    report["pidRecordAbsent"] = (!h.join("ocx.pid").exists()).into();
    if let Ok(bytes) = std::fs::read(log.path()) {
        atomic_write(&evidence.join("runtime.log"), &bytes, 0o600).unwrap();
    }
    let passed = started_ok
        && stop_ok
        && restored.values().all(|v| *v)
        && matches!(&status, Ok(value) if value["ready"]==true && value["status"]=="running")
        && report["recordAbsent"] == true
        && report["pidRecordAbsent"] == true
        && report["startElapsedMs"].as_u64().unwrap() < 30000
        && report["statusElapsedMs"].as_u64().unwrap() < 5000
        && report["stopElapsedMs"].as_u64().unwrap() < 10000;
    report["result"] = if passed { "passed" } else { "failed" }.into();
    write(&report);
    println!("OFFICIAL299_NATIVE_LIFECYCLE {report}");
    assert!(
        passed,
        "production native lifecycle failed; owned fixture retained at {}",
        root.display()
    );
}
