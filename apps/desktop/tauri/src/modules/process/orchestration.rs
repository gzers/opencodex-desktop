//! FZ-10 ~ FZ-12 的进程编排结果模型。
//!
//! 领域服务只解释受控运行器返回的动作结果；真实子进程与异步等待
//! 仍由后续 infrastructure 适配器接入，避免把平台细节混入业务状态机。

use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{LifecycleAction, LifecycleResult, LifecycleStateMachine, ProcessLifecycleState};
use crate::errors::AppError;

/// FZ-10 冻结动作窗口（H-04）；重启是停止与启动的组合上限。数值来自固化运行策略。
pub fn action_timeouts() -> [(LifecycleAction, Duration); 3] {
    [
        (LifecycleAction::Start, super::start_timeout()),
        (LifecycleAction::Stop, super::stop_timeout()),
        (LifecycleAction::Restart, super::restart_limit()),
    ]
}

/// FZ-11 官方退出码不改写，包装层只登记冻结的错误分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WrappedExitCode {
    InstanceNotRunning = 3,
    ConfirmationRequired = 4,
    AccessDenied = 5,
    TargetNotFound = 6,
    TargetStateConflict = 7,
    ValidationFailed = 8,
    ExecutionFailed = 9,
    Cancelled = 10,
    Timeout = 11,
    InternalError = 12,
}

/// FZ-11 管理器包装层退出码分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessExitCode;

impl ProcessExitCode {
    pub const INSTANCE_NOT_RUNNING: i32 = 3;
    pub const CONFIRMATION_REQUIRED: i32 = 4;
    pub const ACCESS_DENIED: i32 = 5;
    pub const TARGET_NOT_FOUND: i32 = 6;
    pub const TARGET_STATE_CONFLICT: i32 = 7;
    pub const VALIDATION_FAILED: i32 = 8;
    pub const EXECUTION_FAILED: i32 = 9;
    pub const CANCELLED: i32 = 10;
    pub const TIMEOUT: i32 = 11;
    pub const INTERNAL_ERROR: i32 = 12;

    pub fn for_error(error: &AppError) -> i32 {
        match error {
            AppError::NotConfigured => Self::INTERNAL_ERROR,
            AppError::AtomicWrite { .. } => Self::EXECUTION_FAILED,
            AppError::InstanceLockConflict => Self::TARGET_STATE_CONFLICT,
            AppError::TargetLockTimeout { .. } | AppError::Timeout => Self::TIMEOUT,
            AppError::NotFound { .. } => Self::TARGET_NOT_FOUND,
            AppError::PassphraseRequired => Self::VALIDATION_FAILED,
            AppError::RuntimeManaged { .. } => Self::EXECUTION_FAILED,
            AppError::ConfigMigration { .. } => Self::INTERNAL_ERROR,
            AppError::FileSystem { .. } | AppError::LogSanitization | AppError::Tauri(_) => {
                Self::EXECUTION_FAILED
            }
        }
    }
}

/// 领域实体：被托管的官方 `ocx` 进程观测。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentProcess {
    pub pid: Option<i32>,
    pub port: Option<u16>,
    pub started_at: Option<String>,
    pub exit_code: Option<i32>,
    pub last_error: Option<String>,
    pub command: LifecycleAction,
    pub env_injected_keys: Vec<String>,
    pub workdir: std::path::PathBuf,
}

impl AgentProcess {
    pub fn not_attached(command: LifecycleAction, workdir: std::path::PathBuf) -> Self {
        Self {
            pid: None,
            port: None,
            started_at: None,
            exit_code: None,
            last_error: None,
            command,
            env_injected_keys: super::EnvironmentPolicy::default()
                .allowed_key_list()
                .iter()
                .map(|value| value.to_string())
                .collect(),
            workdir,
        }
    }

    pub fn mark_started(&mut self, pid: i32, port: u16, started_at: String) {
        self.pid = Some(pid);
        self.port = Some(port);
        self.started_at = Some(started_at);
        self.exit_code = None;
        self.last_error = None;
    }

    pub fn mark_exit(&mut self, exit_code: i32) {
        self.pid = None;
        self.exit_code = Some(exit_code);
        self.started_at = None;
    }
}

/// 取消或失败后的编排结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessOutcome {
    pub action: LifecycleAction,
    pub state: ProcessLifecycleState,
    pub result: LifecycleResult,
    pub exit_code: Option<i32>,
    pub error_summary: Option<String>,
    pub log_id: String,
}

/// FZ-12 错误摘要；只保留白名单来源、短语和 16 位日志 ID。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorSummary;

impl ErrorSummary {
    pub const MAX_LENGTH: usize = 280;
    pub const LOG_ID_LENGTH: usize = 16;

    pub fn build(source: &str, phrase: &str, hint: Option<&str>, log_id: &str) -> String {
        let log_id = if log_id.len() >= Self::LOG_ID_LENGTH {
            &log_id[..Self::LOG_ID_LENGTH]
        } else {
            log_id
        };
        let base = match hint {
            Some(hint) if !hint.trim().is_empty() => format!("[{source}] {phrase}; {hint}"),
            _ => format!("[{source}] {phrase}"),
        };
        let mut output = base.chars().take(Self::MAX_LENGTH).collect::<String>();
        if !log_id.is_empty() {
            output.push_str(&format!(" [log:{log_id}]"));
        }
        output
    }

    pub fn from_error(error: &AppError, log_id: &str) -> String {
        let (source, phrase) = match error {
            AppError::NotConfigured => ("ocx", "动作未配置"),
            AppError::FileSystem { .. } => ("fs", "文件访问失败"),
            AppError::InstanceLockConflict => ("ipc", "已有运行中实例"),
            AppError::TargetLockTimeout { .. } => ("fs", "目标锁等待超时"),
            AppError::AtomicWrite { .. } => ("fs", "原子写入失败"),
            AppError::LogSanitization => ("fs", "日志脱敏失败"),
            AppError::NotFound { .. } => ("ipc", "目标不存在"),
            AppError::PassphraseRequired => ("ipc", "需要旧容器口令"),
            AppError::RuntimeManaged { .. } => ("ocx", "托管安装失败"),
            AppError::ConfigMigration { .. } => ("fs", "配置迁移失败"),
            AppError::Timeout => ("ocx", "命令等待超时"),
            AppError::Tauri(_) => ("ipc", "桌面壳通信失败"),
        };
        Self::build(source, phrase, None, log_id)
    }
}

/// 用确定性 SHA-256 生成 16 位日志 ID；真实日志由日志模块后续关联。
pub fn log_id_for(seed: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(seed.as_bytes());
    digest
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// 把生命周期动作转换为官方命令文本；不执行命令。
pub fn official_command(action: LifecycleAction) -> &'static str {
    match action {
        LifecycleAction::Start => "ensure",
        LifecycleAction::Stop => "stop",
        LifecycleAction::Restart => "restart",
    }
}

/// 冻结超时查询；未知动作不可能出现在当前生命周期契约里。
pub fn action_timeout(action: LifecycleAction) -> Duration {
    action_timeouts()
        .iter()
        .find(|(candidate, _)| *candidate == action)
        .map(|(_, timeout)| *timeout)
        .unwrap_or_else(super::restart_limit)
}

/// 编排一次已受控的生命周期动作；运行器负责校验显式路径。
pub struct ProcessOrchestrator<R>
where
    R: super::ProcessRunner,
{
    runner: R,
}

impl<R> ProcessOrchestrator<R>
where
    R: super::ProcessRunner,
{
    pub fn new(runner: R) -> Self {
        Self { runner }
    }

    pub fn run(
        &self,
        mut machine: LifecycleStateMachine,
        action: LifecycleAction,
    ) -> Result<ProcessOutcome, AppError> {
        let timeout = action_timeout(action);
        let seed = format!("{}:{}", official_command(action), timeout.as_secs());
        let log_id = log_id_for(&seed);
        let before = machine.current();
        let next_state = machine.advance(action)?;
        match self.runner.execute(&super::ProcessCommand::new(
            action,
            std::path::PathBuf::from("/fixtures/ocx"),
            std::path::PathBuf::from("/fixtures/workdir"),
            std::path::PathBuf::from("/fixtures/opencodex-home"),
        )) {
            Ok(result) => Ok(ProcessOutcome {
                action,
                state: next_state,
                result,
                exit_code: None,
                error_summary: None,
                log_id,
            }),
            Err(error) => {
                machine.rollback_to_observation(before);
                Ok(ProcessOutcome {
                    action,
                    state: machine.current(),
                    result: if matches!(error, AppError::NotConfigured) {
                        LifecycleResult::Cancelled
                    } else {
                        LifecycleResult::Failed
                    },
                    exit_code: Some(ProcessExitCode::for_error(&error)),
                    error_summary: Some(ErrorSummary::from_error(&error, &log_id)),
                    log_id,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::process::fixtures::VirtualProcessRunner;

    fn outcome(action: LifecycleAction) -> ProcessOutcome {
        let runner = VirtualProcessRunner::new();
        let orchestrator = ProcessOrchestrator::new(runner);
        let machine = LifecycleStateMachine::new(match action {
            LifecycleAction::Start | LifecycleAction::Restart => ProcessLifecycleState::Stopped,
            LifecycleAction::Stop => ProcessLifecycleState::Running,
        });
        orchestrator.run(machine, action).expect("outcome")
    }

    #[test]
    fn frozen_action_timeouts_match_fz10() {
        assert_eq!(
            action_timeout(LifecycleAction::Start),
            Duration::from_secs(20)
        );
        assert_eq!(
            action_timeout(LifecycleAction::Stop),
            Duration::from_secs(10)
        );
        assert_eq!(
            action_timeout(LifecycleAction::Restart),
            Duration::from_secs(30)
        );
    }

    #[test]
    fn wrapper_exit_codes_match_fz11() {
        assert_eq!(ProcessExitCode::INSTANCE_NOT_RUNNING, 3);
        assert_eq!(ProcessExitCode::CONFIRMATION_REQUIRED, 4);
        assert_eq!(ProcessExitCode::ACCESS_DENIED, 5);
        assert_eq!(ProcessExitCode::TARGET_NOT_FOUND, 6);
        assert_eq!(ProcessExitCode::TARGET_STATE_CONFLICT, 7);
        assert_eq!(ProcessExitCode::VALIDATION_FAILED, 8);
        assert_eq!(ProcessExitCode::EXECUTION_FAILED, 9);
        assert_eq!(ProcessExitCode::CANCELLED, 10);
        assert_eq!(ProcessExitCode::TIMEOUT, 11);
        assert_eq!(ProcessExitCode::INTERNAL_ERROR, 12);
    }

    #[test]
    fn error_summary_is_fz12_shape_and_capped() {
        let summary =
            ErrorSummary::build("ocx", "启动失败", Some("检查端口"), "abcdef0123456789ffff");
        assert_eq!(summary, "[ocx] 启动失败; 检查端口 [log:abcdef0123456789]");
        assert!(summary.chars().count() <= ErrorSummary::MAX_LENGTH);

        let long_phrase = "x".repeat(500);
        let long = ErrorSummary::build("fs", &long_phrase, Some("hint"), "0123456789abcdef");
        assert_eq!(long.chars().count(), ErrorSummary::MAX_LENGTH + 23);
    }

    #[test]
    fn unconfigured_runner_preserves_last_observation() {
        let outcome = outcome(LifecycleAction::Start);
        assert_eq!(outcome.state, ProcessLifecycleState::Stopped);
        assert_eq!(outcome.result, LifecycleResult::Failed);
        assert_eq!(outcome.exit_code, Some(ProcessExitCode::EXECUTION_FAILED));
        assert_eq!(
            outcome.error_summary,
            Some("[fs] 文件访问失败 [log:310a84585af183a0]".to_string())
        );
    }

    #[test]
    fn failed_stop_preserves_running_observation() {
        let outcome = outcome(LifecycleAction::Stop);
        assert_eq!(outcome.state, ProcessLifecycleState::Running);
        assert_eq!(outcome.result, LifecycleResult::Failed);
        assert_eq!(outcome.exit_code, Some(ProcessExitCode::EXECUTION_FAILED));
    }

    #[test]
    fn agent_process_tracks_not_attached_and_exit() {
        let mut process = AgentProcess::not_attached(
            LifecycleAction::Start,
            std::path::PathBuf::from("/fixture"),
        );
        assert_eq!(process.pid, None);
        let mut expected = vec![
            "HOME",
            "LANG",
            "LC_ALL",
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "NO_PROXY",
            "PATH",
            "OPENCODEX_HOME",
        ];
        if cfg!(windows) {
            expected.extend([
                "USERPROFILE",
                "APPDATA",
                "LOCALAPPDATA",
                "SystemRoot",
                "WINDIR",
                "COMSPEC",
                "TEMP",
                "TMP",
            ]);
        }
        assert_eq!(process.env_injected_keys, expected);
        process.mark_started(42, 10100, "2026-09-15T00:00:00Z".to_string());
        assert_eq!(process.pid, Some(42));
        assert_eq!(process.port, Some(10100));
        process.mark_exit(0);
        assert_eq!(process.pid, None);
        assert_eq!(process.exit_code, Some(0));
    }

    #[test]
    fn official_commands_are_frozen() {
        assert_eq!(official_command(LifecycleAction::Start), "ensure");
        assert_eq!(official_command(LifecycleAction::Stop), "stop");
        assert_eq!(official_command(LifecycleAction::Restart), "restart");
    }

    #[test]
    fn log_id_is_sixteen_hex_characters() {
        let id = log_id_for("start:20");
        assert_eq!(id.len(), 16);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
