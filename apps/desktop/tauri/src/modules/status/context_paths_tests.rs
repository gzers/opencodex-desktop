use super::*;
use std::cell::{Cell, RefCell};

// Official2.99 status.ts returns the file in paths.config and may report a
// selected/configured listen port even when proxy.running=false. Neither is a
// Desktop data root or a live listener. Synthetic values never query a service.
fn stopped_payload() -> Value {
    serde_json::json!({"schemaVersion":1,"proxy":{"running":false,"pid":7123,"health":{"ok":false}},
        "listen":{"port":54109,"source":"config"},
        "paths":{"config":"/payload-only/config.json","pid":"/payload-only/ocx.pid"},
        "startup":{"status":"native","protection":"none","rebootSafe":true}})
}
#[test]
fn official_config_file_and_selected_port_are_not_a_home_or_live_listener_after_stop() {
    let report = OfficialStatusReport::from_value(stopped_payload());
    let mapped = map_official_output(
        &report,
        Some(RuntimeState::Running),
        ConnectionState::Unconfigured,
        OperationState::Idle,
    );
    assert_eq!(mapped.runtime, RuntimeState::Stopped);
    assert_eq!((mapped.port, mapped.pid), (None, None));
    assert_eq!(
        mapped.facts.opencodex_home, None,
        "do not guess HOME from a config filename"
    );
    assert!(mapped.facts.data_root.as_os_str().is_empty());
}

struct Contextual {
    paths: RefCell<(PathBuf, PathBuf)>,
    calls: Cell<u32>,
    fail: Cell<bool>,
}
impl StatusSource for Contextual {
    fn manager_paths(&self) -> Option<(PathBuf, PathBuf)> {
        Some(self.paths.borrow().clone())
    }
    fn client_recovery(&self) -> ClientRecoveryState {
        ClientRecoveryState::Required
    }
    fn source_unknown(&self) -> bool {
        self.fail.get()
    }
    fn fetch(&self) -> Result<Value, CollectError> {
        self.calls.set(self.calls.get() + 1);
        if self.fail.get() {
            return Err(CollectError::Unreachable);
        }
        let mut payload = stopped_payload();
        payload["dataRoot"] = "/payload-only/not-desktop".into();
        payload["opencodexHome"] = "/payload-only/not-home".into();
        Ok(payload)
    }
}
#[test]
fn desktop_context_owns_paths_on_success_debounce_and_source_failure_without_changing_runtime_admission(
) {
    let mut collector = StatusCollector::new(Contextual {
        paths: RefCell::new(("/actual/D".into(), "/actual/H".into())),
        calls: Cell::new(0),
        fail: Cell::new(false),
    });
    let now = Instant::now();
    let first = collector.refresh_at(now).unwrap();
    assert_eq!(first.facts.data_root, PathBuf::from("/actual/D"));
    assert_eq!(first.facts.opencodex_home, Some("/actual/H".into()));
    assert_eq!(first.facts.client_recovery, ClientRecoveryState::Required);
    assert!(!first
        .downgrade_reasons
        .contains(&DowngradeReason::MissingDataRoot));
    *collector.source.paths.borrow_mut() = ("/next/D".into(), "/next/H".into());
    let second = collector
        .refresh_at(now + Duration::from_millis(1))
        .unwrap();
    assert_eq!(second.facts.data_root, PathBuf::from("/next/D"));
    assert_eq!(second.facts.opencodex_home, Some("/next/H".into()));
    assert_eq!(
        collector.source.calls.get(),
        1,
        "paths must not need a second status CLI"
    );
    collector.source.fail.set(true);
    assert_eq!(
        collector.refresh_at(now + Duration::from_secs(2)),
        Err(CollectError::Unreachable)
    );
    assert_eq!(collector.facts().data_root, PathBuf::from("/next/D"));
    assert_eq!(collector.facts().opencodex_home, Some("/next/H".into()));
    assert_eq!(
        collector.matrix().runtime,
        RuntimeState::Unreachable,
        "context paths grant no process-success authority"
    );
    assert_eq!((collector.port(), collector.pid()), (None, None));
}

#[test]
fn unknown_state_fallback_does_not_invent_new_endpoint_absence() {
    let mapped = map_official_output(
        &OfficialStatusReport::from_value(
            serde_json::json!({"status":"unknown","port":1234,"pid":"42"}),
        ),
        Some(RuntimeState::Stopped),
        ConnectionState::Unconfigured,
        OperationState::Idle,
    );
    assert_eq!(mapped.runtime, RuntimeState::Stopped);
    assert_eq!((mapped.port, mapped.pid), (Some(1234), Some("42".into())));
    assert!(mapped
        .downgrade_reasons
        .contains(&DowngradeReason::UnknownStatus));
}
