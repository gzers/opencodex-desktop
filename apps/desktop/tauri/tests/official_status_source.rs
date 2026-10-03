use opencodex_desktop_lib::modules::status::{CollectError, StatusCollector};
use opencodex_desktop_lib::types::status::{ConnectionState, OperationState, RuntimeState};

#[test]
fn collector_refresh_success_maps_official_output_and_resets_backoff() {
    let mut collector = StatusCollector::new(
        opencodex_desktop_lib::modules::status::VirtualStatusSource::new(Ok(serde_json::json!({
            "status": "running",
            "ready": true,
            "port": 10100,
            "pid": "39421",
            "dataRoot": "/tmp/fixture",
            "startup": {"protection": "none", "rebootSafe": false}
        }))),
    );
    let mapped = collector.refresh().expect("successful refresh");
    assert_eq!(mapped.runtime, RuntimeState::Running);
    assert_eq!(mapped.connection, ConnectionState::Unconfigured);
    assert_eq!(mapped.operation, OperationState::Idle);
    assert_eq!(collector.failure_count(), 0);
}

#[test]
fn collector_refresh_failure_preserves_state_and_increments_backoff() {
    let mut collector = StatusCollector::new(
        opencodex_desktop_lib::modules::status::VirtualStatusSource::new(Err(
            CollectError::Timeout,
        )),
    );
    let error = collector.refresh().expect_err("timeout expected");
    assert_eq!(error, CollectError::Timeout);
    assert_eq!(collector.matrix().runtime, RuntimeState::NotFound);
    assert_eq!(collector.failure_count(), 1);
    assert_eq!(
        collector.next_interval(
            opencodex_desktop_lib::modules::status::StatusDimension::Runtime,
            false
        ),
        std::time::Duration::from_secs(6)
    );
}

#[test]
fn collector_request_debounces_one_second() {
    let mut collector = StatusCollector::new(
        opencodex_desktop_lib::modules::status::VirtualStatusSource::new(Ok(serde_json::json!({
            "status": "running",
            "ready": true,
            "dataRoot": "/tmp/fixture",
            "startup": {"protection": "none", "rebootSafe": false}
        }))),
    );
    collector.refresh().expect("first refresh");
    assert_eq!(
        collector.refresh().unwrap_err(),
        CollectError::Timeout,
        "same-second requests must be debounced"
    );
}
