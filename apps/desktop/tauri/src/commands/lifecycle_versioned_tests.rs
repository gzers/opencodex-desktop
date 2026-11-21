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
