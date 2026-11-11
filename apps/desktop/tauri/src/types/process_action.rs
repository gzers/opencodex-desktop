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
    pub client_recovery_required: bool,
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
            client_recovery_required: false,
            failure: None,
        }
    }

    pub fn with_result(mut self, result: LifecycleResult) -> Self {
        // The runner has finished its ownership/readiness or stop proof. Do
        // not return the pre-execution Starting/Stopping gates after success.
        // Failed/cancelled work gains no terminal-state authority here.
        match result {
            LifecycleResult::Started => self = Self::from_state(ProcessLifecycleState::Running),
            LifecycleResult::Stopped => self = Self::from_state(ProcessLifecycleState::Stopped),
            LifecycleResult::Cancelled | LifecycleResult::Failed => {}
        }
        self.result = Some(result);
        self
    }
    fn stopped_client_recovery(mut self) -> Self {
        self.lifecycle_state = ProcessLifecycleState::Stopped;
        self.result = None;
        self.can_start = false;
        self.can_stop = false;
        self.can_restart = false;
        self.client_recovery_required = true;
        self
    }
    pub(crate) fn with_stopped_client_recovery(
        self,
        _observed: &crate::commands::runtime_recovery_v2_owner::StoppedObservation,
    ) -> Self {
        self.stopped_client_recovery()
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
            crate::errors::AppError::RuntimeManaged { code, .. }
                if code == "lifecycle_start_exit" =>
            {
                (
                    code.clone(),
                    "服务启动进程已退出，请查看诊断日志后重试。".into(),
                )
            }
            crate::errors::AppError::RuntimeManaged { code, detail }
                if code.starts_with("lifecycle_")
                    || [
                        "client_original_restore_pending",
                        "client_original_session_unconfirmed",
                    ]
                    .contains(&code.as_str())
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
    fn stopped_service_with_pending_original_restore_keeps_distinct_actionable_failure() {
        let error = crate::errors::AppError::RuntimeManaged {
            code: "client_original_restore_pending".into(),
            detail: "服务已正常停止，客户端配置原件恢复尚未完成；原会话和现场已保留".into(),
        };
        let value =
            ProcessActionResult::from_state(ProcessLifecycleState::Running).with_failure(&error);
        assert_eq!(
            value.failure.as_ref().unwrap().code,
            "client_original_restore_pending"
        );
        assert!(value
            .failure
            .as_ref()
            .unwrap()
            .message
            .contains("服务已正常停止"));
        assert!(value.result.is_none());
        // A matching string supplied by an arbitrary runner is not a native
        // stopped observation. Production requires the separate private token.
        assert_eq!(value.lifecycle_state, ProcessLifecycleState::Running);
        assert!(!value.client_recovery_required);
    }

    #[test]
    fn observed_stop_with_recovery_failure_is_not_running_or_complete() {
        let error = crate::errors::AppError::RuntimeManaged {
            code: "client_original_restore_pending".into(),
            detail: "服务已停止，原件恢复待完成".into(),
        };
        // DTO-only synthetic projection. The real native action test separately
        // exercises issuance from CompletedLifecycleStop, not this private seam.
        let value = ProcessActionResult::from_state(ProcessLifecycleState::Running)
            .stopped_client_recovery()
            .with_failure(&error);
        assert_eq!(value.lifecycle_state, ProcessLifecycleState::Stopped);
        assert!(value.client_recovery_required);
        assert!(!value.can_start && !value.can_stop && !value.can_restart);
        assert!(value.result.is_none());
        let json = serde_json::to_value(&value).unwrap();
        assert_eq!(json["clientRecoveryRequired"], true);
        assert_eq!(json["failure"]["code"], "client_original_restore_pending");
    }

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

    #[test]
    fn native_admission_stage_is_retained_in_logs_not_product_message() {
        let error = crate::errors::AppError::RuntimeManaged {
            code: "lifecycle_start_exit".into(),
            detail: "exit_code=1 lifetime_admission=job-create kind=error win32=5".into(),
        };
        let value = ProcessActionResult::from_state(ProcessLifecycleState::StartingFailed)
            .with_failure(&error);
        let raw = serde_json::to_string(&value).unwrap();
        assert!(raw.contains("lifecycle_start_exit"));
        assert!(!raw.contains("job-create") && !raw.contains("win32="));
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
    fn confirmed_actions_return_terminal_gates_without_a_later_status_poll() {
        for before in [
            ProcessLifecycleState::Starting,
            ProcessLifecycleState::Stopping,
        ] {
            let started =
                ProcessActionResult::from_state(before).with_result(LifecycleResult::Started);
            assert_eq!(started.lifecycle_state, ProcessLifecycleState::Running);
            assert!(!started.can_start && started.can_stop && started.can_restart);
            assert_eq!(started.result, Some(LifecycleResult::Started));
        }
        let stopped = ProcessActionResult::from_state(ProcessLifecycleState::Stopping)
            .with_result(LifecycleResult::Stopped);
        assert_eq!(stopped.lifecycle_state, ProcessLifecycleState::Stopped);
        assert!(stopped.can_start && !stopped.can_stop && !stopped.can_restart);
        assert_eq!(stopped.result, Some(LifecycleResult::Stopped));
    }

    #[test]
    fn unsuccessful_results_cannot_claim_running_or_stopped() {
        for before in [
            ProcessLifecycleState::Starting,
            ProcessLifecycleState::Stopping,
        ] {
            for outcome in [LifecycleResult::Failed, LifecycleResult::Cancelled] {
                let value = ProcessActionResult::from_state(before).with_result(outcome.clone());
                assert_eq!(value.lifecycle_state, before);
                assert_eq!(value.can_start, before.can_start());
                assert_eq!(value.can_stop, before.can_stop());
                assert_eq!(value.can_restart, before.can_restart());
                assert_eq!(value.result, Some(outcome));
            }
        }
    }

    #[test]
    fn serializes_camel_case_frontend_contract() {
        let value = ProcessActionResult::from_state(ProcessLifecycleState::StartingFailed);
        let payload = serde_json::to_value(&value).unwrap();
        assert_eq!(payload["lifecycleState"], "starting_failed");
        assert_eq!(payload["canStart"], true);
    }
}
