use opencodex_desktop_lib::commands::status_snapshot_from_collector;
use opencodex_desktop_lib::modules::status::{CollectError, StatusCollector, VirtualStatusSource};
use opencodex_desktop_lib::types::runtime_status::{unconfigured_snapshot, StatusSource};
use opencodex_desktop_lib::types::status::{ConnectionState, OperationState, RuntimeState};

#[test]
fn concurrent_status_observer_refuses_instead_of_queuing_another_full_sample() {
    use opencodex_desktop_lib::{
        commands::get_status_snapshot_with_collector,
        infrastructure::{
            runtime_executable::FixedRuntimeExecutable, status_source::OfficialStatusSource,
        },
        modules::process::EnvironmentPolicy,
    };
    use std::{
        sync::{mpsc, Arc, Mutex},
        time::Duration,
    };
    let root = tempfile::tempdir().unwrap();
    let source = OfficialStatusSource::new(
        FixedRuntimeExecutable::unresolved(),
        root.path(),
        EnvironmentPolicy {
            opencodex_home: root.path().join("home"),
            ..Default::default()
        },
    );
    let collector = Arc::new(Mutex::new(StatusCollector::new(source)));
    let held = collector.lock().unwrap();
    let other = collector.clone();
    let (send, receive) = mpsc::channel();
    let observer = std::thread::spawn(move || {
        let result = get_status_snapshot_with_collector(&other);
        send.send(result.map(|_| None).unwrap_or_else(|e| Some(e.to_string())))
            .unwrap();
    });
    let immediate = receive.recv_timeout(Duration::from_millis(500));
    drop(held);
    observer.join().unwrap();
    let error = immediate
        .expect("busy sample must not queue behind the original lock")
        .expect("busy is uncertainty, never an old successful snapshot");
    assert!(error.contains("status_sample_in_progress"), "{error}");
    assert_eq!(
        collector.lock().unwrap().failure_count(),
        0,
        "no second sampling or failure mutation occurred"
    );
}

#[test]
fn status_snapshot_reports_unconfigured_without_guessing_live_state() {
    let snapshot = unconfigured_snapshot();
    assert_eq!(snapshot.matrix.runtime, RuntimeState::Loading);
    assert_eq!(snapshot.matrix.connection, ConnectionState::Unconfigured);
    assert_eq!(snapshot.matrix.operation, OperationState::Idle);
    assert_eq!(snapshot.source, StatusSource::Unconfigured);
    let wire = serde_json::to_value(snapshot).unwrap();
    assert_eq!(wire.get("status_failure"), Some(&serde_json::Value::Null));
}

#[test]
fn injected_collector_projects_frozen_running_snapshot() {
    let mut collector = StatusCollector::new(VirtualStatusSource::new(Ok(serde_json::json!({
        "status": "running",
        "ready": true,
        "port": 10100,
        "pid": "39421",
        "connection": "synced",
        "dataRoot": "/fixtures/data-root",
        "startup": {"protection": "none", "rebootSafe": false}
    }))));
    let snapshot = status_snapshot_from_collector(&mut collector)
        .expect("controlled status snapshot should be available");

    assert_eq!(snapshot.matrix.runtime, RuntimeState::Running);
    assert_eq!(snapshot.matrix.connection, ConnectionState::Synced);
    assert_eq!(snapshot.matrix.operation, OperationState::Idle);
    assert_eq!(snapshot.port, Some(10100));
    assert_eq!(snapshot.pid.as_deref(), Some("39421"));
    assert!(!snapshot.can_start);
    assert!(snapshot.can_stop);
    assert!(snapshot.can_restart);
    assert_eq!(snapshot.source, StatusSource::Live);
}

#[test]
fn failed_command_samples_revoke_actions_in_ui_and_tray_until_idle_is_observed() {
    use opencodex_desktop_lib::commands::tray::tray_state_from_parts;
    use serde_json::{json, Value};
    use std::time::{Duration, Instant};

    for (result, expected) in [
        (Err(CollectError::Timeout), "timeout"),
        (Err(CollectError::Unreachable), "unreachable"),
        (Ok(json!({"proxy": {"running": "no"}})), "parse"),
        (Ok(json!({"status": "future-state"})), "unmapped"),
    ] {
        for previous in [
            json!({"status":"stopped"}),
            json!({"status":"running", "ready":true, "pid":42, "port":10100}),
        ] {
            let mut collector = StatusCollector::new(VirtualStatusSource::new(Ok(previous)));
            collector
                .refresh_at(Instant::now() - Duration::from_secs(2))
                .unwrap();
            assert!(collector.snapshot().can_start || collector.snapshot().can_stop);

            collector.source = VirtualStatusSource::new(result.clone());
            let snapshot = status_snapshot_from_collector(&mut collector).unwrap();
            let wire = serde_json::to_value(&snapshot).unwrap();
            assert_eq!(wire["status_failure"], expected);
            assert_eq!(wire["matrix"]["runtime"], "unreachable");
            assert_eq!(wire["facts"]["health"], "unknown");
            for key in ["can_start", "can_stop", "can_restart"] {
                assert_eq!(wire[key], false);
            }
            assert_eq!((snapshot.pid, snapshot.port), (None, None));
            let tray =
                tray_state_from_parts(&collector, std::path::Path::new("/fixtures/data-root"))
                    .unwrap();
            assert!(!tray.can_start && !tray.can_stop && !tray.can_restart);

            collector.source = VirtualStatusSource::new(Err(CollectError::ManagedHomeIdle));
            collector.invalidate();
            let recovered =
                serde_json::to_value(status_snapshot_from_collector(&mut collector).unwrap())
                    .unwrap();
            assert_eq!(recovered.get("status_failure"), Some(&Value::Null));
            assert_eq!(recovered["matrix"]["runtime"], "stopped");
            assert_eq!(recovered["can_start"], true);
            assert_eq!(collector.failure_count(), 0);
        }
    }
}

#[cfg(feature = "integration-test")]
mod ipc {
    use super::*;
    use opencodex_desktop_lib::commands::get_status_snapshot_with_collector;
    use opencodex_desktop_lib::modules::status::{StatusCollector, VirtualStatusSource};
    use tauri::Manager;

    #[test]
    #[ignore = "Tauri tray setup requires the native main thread"]
    fn managed_collector_is_injected_into_status_command() {
        let app = tauri::test::mock_builder()
            .manage(std::sync::Mutex::new(StatusCollector::new(
                VirtualStatusSource::new(Ok(serde_json::json!({
                    "status": "running",
                    "ready": true,
                    "port": 10100,
                    "pid": "39421",
                    "connection": "synced",
                    "dataRoot": "/fixtures/data-root",
                    "startup": {"protection": "none", "rebootSafe": false}
                }))),
            )))
            .build(tauri::generate_context!())
            .expect("mock Tauri app should build");
        let handle = app.handle().clone();
        handle
            .clone()
            .run_on_main_thread(move || {
                let state = handle.state::<opencodex_desktop_lib::state::SharedStatusCollector>();
                let result = get_status_snapshot_with_collector(state.inner());
                let snapshot = result.expect("managed status command should succeed");
                assert_eq!(snapshot.matrix.runtime, RuntimeState::Running);
                assert_eq!(snapshot.source, StatusSource::Live);
            })
            .expect("run on main thread should succeed");
    }
}
