//! FZ-06 三维状态契约。
//!
//! 状态枚举是冻结契约；模块、命令层和 UI 都必须复用这些类型，
//! 不允许在手写状态字符串时扩大枚举范围。

use serde::{Deserialize, Serialize};

/// 运行态，按 FZ-06 从高风险到常规状态排列。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeState {
    Loading,
    StartingFailed,
    ExternalTakeover,
    AtRisk,
    Unreachable,
    Pending,
    Starting,
    Stopping,
    Running,
    Stopped,
    NotFound,
}

/// 连接态，描述数据根、同步与会话的当前一致性。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    Conflict,
    Syncing,
    Connecting,
    Failed,
    Disconnected,
    Unconfigured,
    Synced,
}

/// 操作态，描述最近一次用户请求的进行中或终态结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationState {
    RollingBack,
    Applying,
    BackingUp,
    Validating,
    Failed,
    Succeeded,
    Cancelled,
    Idle,
}

/// 一个三维状态观测点。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusMatrix {
    pub runtime: RuntimeState,
    pub connection: ConnectionState,
    pub operation: OperationState,
}

impl StatusMatrix {
    /// 没有进行中的操作，且进程不在启动、等待就绪、停止或被外部接管时才可启动。
    pub fn can_start(&self) -> bool {
        matches!(self.operation, OperationState::Idle) && runtime_can_start(self.runtime)
    }

    /// 只有本管理器仍能触达且未在执行操作时才允许停止。
    pub fn can_stop(&self) -> bool {
        matches!(self.operation, OperationState::Idle) && runtime_can_stop(self.runtime)
    }
}

/// 运行态维度上的「可启动」判定；FZ-06 契约的唯一定义处。
///
/// 托盘菜单、应用菜单、概览按钮都必须复用本函数，不允许各写一份白名单——
/// 此前托盘手写白名单漏掉 `AtRisk`，导致「存在风险」时代理明明没在跑却无法启动。
///
/// `AtRisk` 允许启动：该状态由「代理未运行 + 官方报告 startup at-risk」折叠而来，
/// 启动代理正是恢复手段；风险只作提醒，不阻断动作。
pub fn runtime_can_start(runtime: RuntimeState) -> bool {
    !matches!(
        runtime,
        RuntimeState::Loading
            | RuntimeState::ExternalTakeover
            | RuntimeState::Pending
            | RuntimeState::Starting
            | RuntimeState::Stopping
            | RuntimeState::Running
    )
}

/// 运行态维度上的「可停止 / 可重启」判定；FZ-06 契约的唯一定义处。
pub fn runtime_can_stop(runtime: RuntimeState) -> bool {
    matches!(runtime, RuntimeState::Running | RuntimeState::Unreachable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_gates_cover_every_frozen_state() {
        let cases = [
            (RuntimeState::Loading, false, false),
            (RuntimeState::StartingFailed, true, false),
            (RuntimeState::ExternalTakeover, false, false),
            (RuntimeState::AtRisk, true, false),
            (RuntimeState::Unreachable, true, true),
            (RuntimeState::Pending, false, false),
            (RuntimeState::Starting, false, false),
            (RuntimeState::Stopping, false, false),
            (RuntimeState::Running, false, true),
            (RuntimeState::Stopped, true, false),
            (RuntimeState::NotFound, true, false),
        ];
        for (runtime, can_start, can_stop) in cases {
            assert_eq!(
                runtime_can_start(runtime),
                can_start,
                "{runtime:?} can_start"
            );
            assert_eq!(runtime_can_stop(runtime), can_stop, "{runtime:?} can_stop");
        }
    }

    // 回归：存在风险时代理没在跑，必须允许启动——这是托盘置灰的根因。
    #[test]
    fn at_risk_can_start_and_is_not_blocked_by_risk() {
        let matrix = StatusMatrix {
            runtime: RuntimeState::AtRisk,
            connection: ConnectionState::Unconfigured,
            operation: OperationState::Idle,
        };
        assert!(matrix.can_start());
        assert!(!matrix.can_stop());
    }
}
