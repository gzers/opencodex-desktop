use super::*;
use std::cell::Cell;
struct Changing {
    state: Cell<ClientRecoveryState>,
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
            serde_json::json!({"status":"running","ready":true,"dataRoot":"/fixtures/D",
            "startup":{"protection":"none","rebootSafe":false},
            "client_recovery":"not_required","facts":{"client_recovery":"not_required"}}),
        )
    }
}
#[test]
fn fresh_native_recovery_is_not_cached_with_the_official_process_payload_or_cleared_by_failure() {
    let mut collector = StatusCollector::new(Changing {
        state: Cell::new(ClientRecoveryState::Required),
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
