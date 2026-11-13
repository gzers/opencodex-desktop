use super::*;
use std::cell::Cell;
struct Changing {
    state: Cell<ClientRecoveryState>,
    runtime: Cell<&'static str>,
    calls: Cell<u32>,
    fail: Cell<bool>,
}
impl StatusSource for Changing {
    fn client_recovery(&self) -> ClientRecoveryState {
        self.state.get()
    }
    fn fetch(&self) -> Result<Value, CollectError> {
        self.calls.set(self.calls.get() + 1);
        if self.fail.get() {
            return Err(CollectError::Timeout);
        }
        Ok(
            serde_json::json!({"status":self.runtime.get(),"ready":self.runtime.get()=="running","dataRoot":"/fixtures/D",
            "startup":{"protection":"none","rebootSafe":false},
            "client_recovery":"not_required","facts":{"client_recovery":"not_required"}}),
        )
    }
}
#[test]
fn fresh_native_recovery_is_not_cached_with_the_official_process_payload_or_cleared_by_failure() {
    let mut collector = StatusCollector::new(Changing {
        state: Cell::new(ClientRecoveryState::Required),
        runtime: Cell::new("running"),
        calls: Cell::new(0),
        fail: Cell::new(false),
    });
    let now = Instant::now();
    let first = collector.refresh_at(now).unwrap();
    assert_eq!(first.facts.client_recovery, ClientRecoveryState::Required);
    assert_eq!(first.runtime, RuntimeState::Running);
    collector.source.state.set(ClientRecoveryState::Unknown);
    let debounced = collector
        .refresh_at(now + Duration::from_millis(1))
        .unwrap();
    assert_eq!(
        debounced.facts.client_recovery,
        ClientRecoveryState::Unknown
    );
    assert_eq!(
        collector.facts().client_recovery,
        ClientRecoveryState::Unknown
    );
    assert_eq!(collector.source.calls.get(), 1);
    collector.source.state.set(ClientRecoveryState::Active);
    assert_eq!(
        collector
            .refresh_light_at(now + Duration::from_millis(2))
            .unwrap()
            .facts
            .client_recovery,
        ClientRecoveryState::Active
    );
    assert_eq!(collector.source.calls.get(), 1);
    collector.source.state.set(ClientRecoveryState::Required);
    collector.source.fail.set(true);
    assert_eq!(
        collector.refresh_at(now + Duration::from_secs(2)),
        Err(CollectError::Timeout)
    );
    assert_eq!(
        collector.facts().client_recovery,
        ClientRecoveryState::Required
    );
    assert_eq!(
        collector.matrix().runtime,
        RuntimeState::Running,
        "recovery state must not invent a stopped process"
    );
}

#[test]
fn historical_ready_never_claims_active_after_observed_stop_or_failed_observation() {
    let mut collector = StatusCollector::new(Changing {
        state: Cell::new(ClientRecoveryState::Active),
        runtime: Cell::new("stopped"),
        calls: Cell::new(0),
        fail: Cell::new(false),
    });
    let now = Instant::now();
    let stopped = collector.refresh_at(now).unwrap();
    assert_eq!(stopped.runtime, RuntimeState::Stopped);
    assert_eq!(stopped.facts.client_recovery, ClientRecoveryState::Required);
    assert_eq!(
        collector
            .refresh_light_at(now + Duration::from_millis(1))
            .unwrap()
            .facts
            .client_recovery,
        ClientRecoveryState::Required
    );
    assert_eq!(collector.source.calls.get(), 1);
    collector.source.fail.set(true);
    assert_eq!(
        collector.refresh_at(now + Duration::from_secs(2)),
        Err(CollectError::Timeout)
    );
    assert_eq!(
        collector.facts().client_recovery,
        ClientRecoveryState::Unknown
    );
    assert_eq!(collector.matrix().runtime, RuntimeState::Stopped);
    collector.source.fail.set(false);
    collector.source.runtime.set("running");
    assert_eq!(
        collector
            .refresh_at(now + Duration::from_secs(4))
            .unwrap()
            .facts
            .client_recovery,
        ClientRecoveryState::Active
    );
    // Persisted recovery work never disappears merely because a process runs.
    collector.source.state.set(ClientRecoveryState::Required);
    assert_eq!(
        collector
            .refresh_light_at(now + Duration::from_millis(4001))
            .unwrap()
            .facts
            .client_recovery,
        ClientRecoveryState::Required
    );
}
