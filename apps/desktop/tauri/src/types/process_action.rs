//! FZ-09 ~ FZ-11 前后端进程动作 DTO 与状态门槛。
//!
//! 只转换请求与结果；真实子进程执行仍留在 infrastructure。

use crate::modules::process::{LifecycleAction, LifecycleResult, ProcessLifecycleState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessActionRequest {
    pub action: LifecycleAction,
    pub confirm: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessActionResult {
    pub lifecycle_state: ProcessLifecycleState,
    pub result: Option<LifecycleResult>,
    pub can_start: bool,
    pub can_stop: bool,
    pub can_restart: bool,
}

impl ProcessActionResult {
    pub fn from_state(state: ProcessLifecycleState) -> Self {
        Self {
            lifecycle_state: state,
            result: None,
            can_start: state.can_start(),
            can_stop: state.can_stop(),
            can_restart: state.can_restart(),
        }
    }

    pub fn with_result(mut self, result: LifecycleResult) -> Self {
        self.result = Some(result);
        self
    }
}

/// 命令层参数校验：变更动作必须显式确认。
pub fn validate_action_request(request: &ProcessActionRequest) -> Result<(), &'static str> {
    if !request.confirm {
        return Err("this action requires confirm");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(action: LifecycleAction, confirm: bool) -> ProcessActionRequest {
        ProcessActionRequest { action, confirm }
    }

    #[test]
    fn requires_confirm_for_every_action() {
        for action in [
            LifecycleAction::Start,
            LifecycleAction::Stop,
            LifecycleAction::Restart,
        ] {
            assert_eq!(
                validate_action_request(&request(action, false)),
                Err("this action requires confirm")
            );
            assert!(validate_action_request(&request(action, true)).is_ok());
        }
    }

    #[test]
    fn result_exposes_frozen_action_gates() {
        let value = ProcessActionResult::from_state(ProcessLifecycleState::Stopped);
        assert!(value.can_start);
        assert!(!value.can_stop);
        assert!(!value.can_restart);
        let value = ProcessActionResult::from_state(ProcessLifecycleState::Running)
            .with_result(LifecycleResult::Started);
        assert!(!value.can_start);
        assert!(value.can_stop);
        assert!(value.can_restart);
        assert_eq!(value.result, Some(LifecycleResult::Started));
    }

    #[test]
    fn serializes_camel_case_frontend_contract() {
        let value = ProcessActionResult::from_state(ProcessLifecycleState::StartingFailed);
        let payload = serde_json::to_value(&value).unwrap();
        assert_eq!(payload["lifecycleState"], "starting_failed");
        assert_eq!(payload["canStart"], true);
    }
}
