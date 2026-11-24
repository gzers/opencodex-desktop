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

/// Real official 2.99 through the product orchestration: provision the pinned
/// repository bundle into a disposable root, then run Start then Status then
/// Stop through process_action_with_runner. Proves the desktop-owned client
/// HOME is used and the real user client directory is never written.
#[test]
#[ignore = "explicit disposable root; provisions the pinned official 2.99 bundle"]
fn official299_product_lifecycle_uses_desktop_client_home() {
    use crate::modules::process::ControlledProcessRunner;
    use crate::modules::runtime::{self, program_image, program_selection, RuntimeHandle};
    use std::os::unix::fs::MetadataExt;
    use std::path::{Path, PathBuf};

    fn identity(path: &Path) -> Option<(u64, i64, u32)> {
        std::fs::metadata(path)
            .ok()
            .map(|m| (m.ino(), m.mtime(), m.mode()))
    }

    let base = PathBuf::from(std::env::var_os("OCXD024_PRODUCT_ROOT").expect("explicit root"));
    std::fs::create_dir_all(&base).unwrap();
    let base = base.canonicalize().unwrap();
    assert!(base.starts_with("/private/tmp") || base.starts_with("/tmp"));
    let program = base.join("program");
    let data = base.join("desktop");
    let home = base.join("opencodex-home");
    let profile = base.join("profile");
    for dir in [&program, &data, &home, &profile] {
        crate::infrastructure::platform::create_private_dir_all(dir, 0o700).unwrap();
    }
    crate::modules::data_root::initialize(&data).unwrap();

    let hud = crate::infrastructure::platform::home_dir()
        .unwrap()
        .join(".codex");
    let before = identity(&hud);
    let _ = &before;

    let bundle = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/runtime-bundle");
    let descriptor = std::fs::read(bundle.join("runtime-bundle.manifest.json")).unwrap();
    let pin = crate::infrastructure::hash::sha256_hex(&descriptor);
    let archive = bundle.join("runtime-bundle-0.2.24-darwin-arm64.tar.gz");
    runtime::bundled::versioned::provision(&program, &data, &archive, &descriptor, &pin).unwrap();
    let selected = program_selection::read(
        &program,
        crate::modules::data_root::layout::Platform::Macos,
        "darwin-arm64",
    )
    .unwrap()
    .unwrap();
    program_image::verify(&selected).unwrap();
    assert_eq!(selected.selection.version, "2.99.0");

    let handle = RuntimeHandle::initialize_versioned(&data, &program);
    let context = ProcessContext {
        runtime: handle,
        working_directory: profile.clone(),
        opencodex_home: home.clone(),
    };
    let runner = Mutex::new(ControlledProcessRunner::new(profile.clone()));
    let log = crate::infrastructure::runtime_log::RuntimeLog::new(&data);
    let act = |action: LifecycleAction| {
        process_action_with_runner(
            ProcessActionRequest {
                action,
                confirm: true,
            },
            &runner,
            &context,
            &log,
            None,
        )
    };

    let client = crate::modules::process::client_home(&home);
    assert!(!client.exists(), "fresh profile starts without CODEX_HOME");
    let started = act(LifecycleAction::Start).expect("product Start");
    assert_eq!(
        started.result,
        Some(crate::modules::process::LifecycleResult::Started),
        "{started:?}"
    );
    let observed = tauri::async_runtime::block_on(crate::infrastructure::attested_status::fetch(
        context.runtime.clone(),
        context.opencodex_home.clone(),
    ))
    .expect("attested status");
    let value = observed.expect("status payload");
    assert_eq!(value["status"], "running");
    assert_eq!(value["ready"], true);

    // Read the service's own effective environment while it is running: the
    // proof is what the live process was given, not what the launcher intended.
    let pid = value["pid"]
        .as_str()
        .and_then(|v| v.parse::<u32>().ok())
        .or_else(|| value["pid"].as_u64().map(|v| v as u32))
        .expect("live pid");
    let ps = std::process::Command::new("/bin/ps")
        .args(["-Eww", "-p", &pid.to_string()])
        .output()
        .expect("read live environment");
    let text = String::from_utf8_lossy(&ps.stdout);
    let client_home_tokens: Vec<&str> = text
        .split_whitespace()
        .filter(|t| t.starts_with("CODEX_HOME="))
        .collect();
    assert_eq!(client_home_tokens.len(), 1, "one effective CODEX_HOME");
    let effective = client_home_tokens[0].trim_start_matches("CODEX_HOME=");
    let expected = crate::modules::process::client_home(&context.opencodex_home);
    assert_eq!(
        std::path::Path::new(effective),
        expected.as_path(),
        "live process must use the Desktop-owned client HOME"
    );
    assert!(
        !std::path::Path::new(effective).starts_with(&hud),
        "the real ~/.codex must not be the effective client HOME"
    );

    let stopped = act(LifecycleAction::Stop).expect("product Stop");
    assert_eq!(
        stopped.result,
        Some(crate::modules::process::LifecycleResult::Stopped),
        "{stopped:?}"
    );

    let metadata = std::fs::symlink_metadata(&client).unwrap();
    assert!(metadata.is_dir());
    assert_eq!(metadata.mode() & 0o777, 0o700);
    // The real user client directory must never be the effective CODEX_HOME.
    // Its own inode and permissions must also be unchanged; mtime is excluded
    // because this test host's own Codex session writes history there.
    let after = identity(&hud);
    assert_eq!(
        before.map(|v| (v.0, v.2)),
        after.map(|v| (v.0, v.2)),
        "the real ~/.codex identity must stay untouched"
    );
    println!(
        "PRODUCT_LIFECYCLE_OK {}",
        serde_json::json!({"root": base, "pid": value["pid"], "port": value["port"], "clientHome": client})
    );
}
