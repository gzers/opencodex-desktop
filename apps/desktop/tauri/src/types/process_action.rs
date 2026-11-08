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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<ProcessActionFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessActionFailure {
    pub code: String,
    pub message: String,
}

impl ProcessActionResult {
    pub fn from_state(state: ProcessLifecycleState) -> Self {
        Self {
            lifecycle_state: state,
            result: None,
            can_start: state.can_start(),
            can_stop: state.can_stop(),
            can_restart: state.can_restart(),
            failure: None,
        }
    }

    pub fn with_result(mut self, result: LifecycleResult) -> Self {
        self.result = Some(result);
        self
    }
    pub fn with_failure(mut self, error: &crate::errors::AppError) -> Self {
        let (code, message) = match error {
            crate::errors::AppError::RuntimeManaged { code, .. } if code == "lifecycle_timeout" => {
                (
                    code.clone(),
                    if self.lifecycle_state == ProcessLifecycleState::StartingFailed {
                        "启动未在规定时间内就绪，请查看诊断日志后重试。"
                    } else {
                        "操作未在规定时间内确认完成，请核对服务状态并查看诊断日志。"
                    }
                    .into(),
                )
            }
            crate::errors::AppError::RuntimeManaged { code, detail }
                if code.starts_with("lifecycle_")
                    || ["port_conflict", "port_permission"].contains(&code.as_str()) =>
            {
                (code.clone(), detail.clone())
            }
            crate::errors::AppError::Timeout => (
                "lifecycle_timeout".into(),
                "操作未在规定时间内确认完成，请核对服务状态。".into(),
            ),
            _ => (
                "lifecycle_unconfirmed".into(),
                "操作未确认完成，已保留当前配置，请查看诊断日志。".into(),
            ),
        };
        self.failure = Some(ProcessActionFailure { code, message });
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

    #[test]
    fn stop_timeout_keeps_running_state_and_hides_internal_phase_timings() {
        let error = crate::errors::AppError::RuntimeManaged {
            code: "lifecycle_timeout".into(),
            detail: "phase=wait-os-absence; timings=internal".into(),
        };
        let value =
            ProcessActionResult::from_state(ProcessLifecycleState::Running).with_failure(&error);
        assert_eq!(value.lifecycle_state, ProcessLifecycleState::Running);
        let json = serde_json::to_string(&value).unwrap();
        assert!(!json.contains("phase=") && !json.contains("timings=") && !json.contains("启动未"));
        assert!(json.contains("核对服务状态"));
    }

    #[test]
    fn startup_timing_diagnostics_stay_out_of_the_product_failure_message() {
        let error = crate::errors::AppError::RuntimeManaged {
            code: "lifecycle_timeout".into(),
            detail: "observations=14 rejected=2 absent_record=10".into(),
        };
        let failure = ProcessActionResult::from_state(ProcessLifecycleState::StartingFailed)
            .with_failure(&error);
        let json = serde_json::to_string(&failure).unwrap();
        assert!(!json.contains("observations") && !json.contains("absent_record"));
        assert!(json.contains("lifecycle_timeout") && json.contains("诊断日志"));
    }

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
