use opencodex_desktop_lib::types::status::{
    ConnectionState, OperationState, RuntimeState, StatusMatrix,
};

/// FZ-06：用无通配符的穷举匹配登记状态全集；
/// 后续新增枚举值时，本测试必须在编译期提醒更新矩阵。
#[test]
fn status_enums_cover_the_contracted_matrix() {
    let runtime_states = runtime_states();
    let connection_states = connection_states();
    let operation_states = operation_states();

    assert_eq!(runtime_states.len(), 11);
    assert_eq!(connection_states.len(), 7);
    assert_eq!(operation_states.len(), 8);
}

/// FZ-06：验证同一观测点在关键三维组合下的边界。
#[test]
fn status_matrix_guards_start_and_stop_actions() {
    let stopped = StatusMatrix {
        runtime: RuntimeState::Stopped,
        connection: ConnectionState::Synced,
        operation: OperationState::Idle,
    };
    assert!(stopped.can_start());
    assert!(!stopped.can_stop());

    let running = StatusMatrix {
        runtime: RuntimeState::Running,
        connection: ConnectionState::Synced,
        operation: OperationState::Idle,
    };
    assert!(running.can_stop());
    assert!(!running.can_start());

    let external_takeover = StatusMatrix {
        runtime: RuntimeState::ExternalTakeover,
        connection: ConnectionState::Conflict,
        operation: OperationState::Idle,
    };
    assert!(!external_takeover.can_start());
    assert!(!external_takeover.can_stop());

    let applying = StatusMatrix {
        runtime: RuntimeState::Running,
        connection: ConnectionState::Synced,
        operation: OperationState::Applying,
    };
    assert!(!applying.can_start());
    assert!(!applying.can_stop());
}

fn runtime_states() -> Vec<RuntimeState> {
    vec![
        RuntimeState::Loading,
        RuntimeState::StartingFailed,
        RuntimeState::ExternalTakeover,
        RuntimeState::AtRisk,
        RuntimeState::Unreachable,
        RuntimeState::Pending,
        RuntimeState::Starting,
        RuntimeState::Stopping,
        RuntimeState::Running,
        RuntimeState::Stopped,
        RuntimeState::NotFound,
    ]
}

#[allow(dead_code)]
fn enforce_runtime_exhaustiveness(state: RuntimeState) -> &'static str {
    match state {
        RuntimeState::Loading => "loading",
        RuntimeState::StartingFailed => "starting-failed",
        RuntimeState::ExternalTakeover => "external-takeover",
        RuntimeState::AtRisk => "at-risk",
        RuntimeState::Unreachable => "unreachable",
        RuntimeState::Pending => "pending",
        RuntimeState::Starting => "starting",
        RuntimeState::Stopping => "stopping",
        RuntimeState::Running => "running",
        RuntimeState::Stopped => "stopped",
        RuntimeState::NotFound => "not-found",
    }
}

fn connection_states() -> Vec<ConnectionState> {
    vec![
        ConnectionState::Conflict,
        ConnectionState::Syncing,
        ConnectionState::Connecting,
        ConnectionState::Failed,
        ConnectionState::Disconnected,
        ConnectionState::Unconfigured,
        ConnectionState::Synced,
    ]
}

#[allow(dead_code)]
fn enforce_connection_exhaustiveness(state: ConnectionState) -> &'static str {
    match state {
        ConnectionState::Conflict => "conflict",
        ConnectionState::Syncing => "syncing",
        ConnectionState::Connecting => "connecting",
        ConnectionState::Failed => "failed",
        ConnectionState::Disconnected => "disconnected",
        ConnectionState::Unconfigured => "unconfigured",
        ConnectionState::Synced => "synced",
    }
}

fn operation_states() -> Vec<OperationState> {
    vec![
        OperationState::RollingBack,
        OperationState::Applying,
        OperationState::BackingUp,
        OperationState::Validating,
        OperationState::Failed,
        OperationState::Succeeded,
        OperationState::Cancelled,
        OperationState::Idle,
    ]
}

#[allow(dead_code)]
fn enforce_operation_exhaustiveness(state: OperationState) -> &'static str {
    match state {
        OperationState::RollingBack => "rolling-back",
        OperationState::Applying => "applying",
        OperationState::BackingUp => "backing-up",
        OperationState::Validating => "validating",
        OperationState::Failed => "failed",
        OperationState::Succeeded => "succeeded",
        OperationState::Cancelled => "cancelled",
        OperationState::Idle => "idle",
    }
}
