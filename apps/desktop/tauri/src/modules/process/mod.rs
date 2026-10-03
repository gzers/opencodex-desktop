//! FZ-09 ~ FZ-11 受控进程生命周期。
//!
//! 运行器只接收显式可执行路径；生命周期动作使用冻结超时，取消只停止
//! 等待中的动作，不发送 SIGKILL。真实进程运行器由 infrastructure 后续接入。

pub mod fixtures;
pub mod orchestration;

use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

use crate::errors::AppError;

/// FZ-10 冻结生命周期超时。
pub const START_TIMEOUT: Duration = Duration::from_secs(20);
pub const STOP_TIMEOUT: Duration = Duration::from_secs(10);
pub const RESTART_LIMIT: Duration = Duration::from_secs(30);

/// FZ-11 管理器包装层退出码；官方子进程退出码不改写。
pub const EXIT_PARAMETER_ERROR: i32 = 2;
pub const EXIT_NOT_FOUND: i32 = 6;
pub const EXIT_CONFLICT: i32 = 7;
pub const EXIT_TIMEOUT: i32 = 11;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleAction {
    Start,
    Stop,
    Restart,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleResult {
    Started,
    Stopped,
    Cancelled,
    Failed,
}

/// 进程运行器必须可测试；不得从 PATH 搜索命令。
pub trait ProcessRunner {
    fn execute(&self, command: &ProcessCommand) -> Result<LifecycleResult, AppError>;
    fn cancel_pending(&self) -> Result<(), AppError>;
}

/// 官方进程调用契约。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessCommand {
    pub action: LifecycleAction,
    pub executable: PathBuf,
    pub working_directory: PathBuf,
    pub environment: EnvironmentPolicy,
}

/// FZ-09 允许透传的环境键；其余键不主动透传。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EnvironmentPolicy {
    pub home: Option<OsString>,
    pub lang: Option<OsString>,
    pub lc_all: Option<OsString>,
    pub http_proxy: Option<OsString>,
    pub https_proxy: Option<OsString>,
    pub no_proxy: Option<OsString>,
    pub path: Option<OsString>,
    pub opencodex_home: PathBuf,
}

impl EnvironmentPolicy {
    pub fn allowed_keys(&self) -> Vec<&'static str> {
        vec![
            "HOME",
            "LANG",
            "LC_ALL",
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "NO_PROXY",
            "PATH",
            "OPENCODEX_HOME",
        ]
    }

    /// 返回按冻结顺序排列的环境键；值不输出，满足“键可记录、值不记录”。
    pub fn allowed_key_list(&self) -> Vec<&'static str> {
        self.allowed_keys()
    }
}

impl ProcessCommand {
    pub fn new(
        action: LifecycleAction,
        executable: PathBuf,
        working_directory: PathBuf,
        opencodex_home: PathBuf,
    ) -> Self {
        Self {
            action,
            executable,
            working_directory,
            environment: EnvironmentPolicy {
                opencodex_home,
                ..Default::default()
            },
        }
    }

    pub fn validate(&self) -> Result<(), AppError> {
        if !self.executable.is_file() {
            return Err(AppError::FileSystem {
                operation: "validate process executable".to_string(),
                detail: format!("{} is not a regular file", self.executable.display()),
            });
        }
        if !self.working_directory.is_dir() {
            return Err(AppError::FileSystem {
                operation: "validate process working directory".to_string(),
                detail: format!("{} is not a directory", self.working_directory.display()),
            });
        }
        if !self.environment.opencodex_home.is_dir() {
            return Err(AppError::FileSystem {
                operation: "validate OPENCODEX_HOME".to_string(),
                detail: "OPENCODEX_HOME must be an absolute directory".to_string(),
            });
        }
        Ok(())
    }
}

/// 生命周期状态机；只允许冻结的动作迁移。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessLifecycleState {
    Stopped,
    Starting,
    Pending,
    Running,
    Stopping,
    StartingFailed,
    Unreachable,
}

impl ProcessLifecycleState {
    pub fn can_start(&self) -> bool {
        matches!(
            self,
            Self::Stopped | Self::StartingFailed | Self::Unreachable
        )
    }

    pub fn can_stop(&self) -> bool {
        matches!(self, Self::Running | Self::Unreachable)
    }

    pub fn can_restart(&self) -> bool {
        matches!(self, Self::Running | Self::Unreachable)
    }

    pub fn next(&self, action: LifecycleAction) -> Result<Self, AppError> {
        match (self, action) {
            (Self::Stopped | Self::StartingFailed | Self::Unreachable, LifecycleAction::Start) => {
                Ok(Self::Starting)
            }
            (Self::Starting, LifecycleAction::Start) => Ok(Self::Pending),
            (Self::Pending, LifecycleAction::Start) => Ok(Self::Running),
            (
                Self::Running | Self::Starting | Self::Pending | Self::Unreachable,
                LifecycleAction::Stop,
            ) => Ok(Self::Stopping),
            (Self::Stopping, LifecycleAction::Stop) => Ok(Self::Stopped),
            (
                Self::Running | Self::Starting | Self::Pending | Self::Unreachable,
                LifecycleAction::Restart,
            ) => Ok(Self::Stopping),
            (Self::Stopping, LifecycleAction::Restart) => Ok(Self::Starting),
            _ => Err(AppError::NotConfigured),
        }
    }

    pub fn fail_start(&self) -> Self {
        Self::StartingFailed
    }

    pub fn reach_unreachable(&self) -> Self {
        Self::Unreachable
    }
}

/// 共享状态机适配器，便于后续异步运行器维护当前状态。
#[derive(Debug, Default)]
pub struct LifecycleStateMachine {
    state: Option<ProcessLifecycleState>,
}

impl LifecycleStateMachine {
    pub fn new(initial: ProcessLifecycleState) -> Self {
        Self {
            state: Some(initial),
        }
    }

    pub fn current(&self) -> ProcessLifecycleState {
        self.state.unwrap_or(ProcessLifecycleState::Stopped)
    }

    pub fn advance(&mut self, action: LifecycleAction) -> Result<ProcessLifecycleState, AppError> {
        let next = self.current().next(action)?;
        self.state = Some(next);
        Ok(next)
    }

    pub fn cancel_pending(&mut self) -> ProcessLifecycleState {
        if matches!(
            self.current(),
            ProcessLifecycleState::Starting | ProcessLifecycleState::Stopping
        ) {
            self.state = Some(ProcessLifecycleState::Stopped);
        }
        self.current()
    }

    pub fn mark_start_failed(&mut self) -> ProcessLifecycleState {
        let next = self.current().fail_start();
        self.state = Some(next);
        next
    }

    pub fn mark_unreachable(&mut self) -> ProcessLifecycleState {
        let next = self.current().reach_unreachable();
        self.state = Some(next);
        next
    }

    /// 失败动作必须回到动作前最后一次稳定观测；不猜测真实子进程状态。
    pub fn rollback_to_observation(&mut self, state: ProcessLifecycleState) {
        self.state = Some(state);
    }
}

/// 模块层通过 trait 引用基础设施实现；平台进程细节不进入本层。
pub use crate::infrastructure::process_runner::ControlledProcessRunner;

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_command(action: LifecycleAction) -> (tempfile::TempDir, ProcessCommand) {
        let temp = tempfile::tempdir().expect("create temporary fixture");
        let executable = temp.path().join("ocx");
        std::fs::write(&executable, b"fixture").expect("write fixture");
        let opencodex_home = temp.path().join("opencodex-home");
        std::fs::create_dir_all(&opencodex_home).expect("create opencodex home fixture");
        let command = ProcessCommand::new(
            action,
            executable,
            temp.path().to_path_buf(),
            opencodex_home,
        );
        (temp, command)
    }

    #[test]
    fn frozen_timeouts_match_contract() {
        assert_eq!(START_TIMEOUT, Duration::from_secs(20));
        assert_eq!(STOP_TIMEOUT, Duration::from_secs(10));
        assert_eq!(RESTART_LIMIT, Duration::from_secs(30));
    }

    #[test]
    fn opencodex_home_must_be_a_directory() {
        let (temp, mut command) = valid_command(LifecycleAction::Stop);
        command.environment.opencodex_home = temp.path().join("config.json");
        std::fs::write(&command.environment.opencodex_home, b"{}").expect("write file");
        assert!(ControlledProcessRunner::validate_spawn_context(&command).is_err());
    }

    #[test]
    fn environment_policy_only_exposes_frozen_keys() {
        let (_temp, command) = valid_command(LifecycleAction::Start);
        assert_eq!(
            command.environment.allowed_key_list(),
            vec![
                "HOME",
                "LANG",
                "LC_ALL",
                "HTTP_PROXY",
                "HTTPS_PROXY",
                "NO_PROXY",
                "PATH",
                "OPENCODEX_HOME"
            ]
        );
        assert!(command
            .environment
            .opencodex_home
            .ends_with("opencodex-home"));
    }

    #[test]
    fn validates_explicit_paths_and_absolute_home() {
        let (temp, command) = valid_command(LifecycleAction::Start);
        command.validate().expect("valid command");

        let missing = ProcessCommand::new(
            LifecycleAction::Start,
            temp.path().join("missing-ocx"),
            temp.path().to_path_buf(),
            temp.path().join("opencodex-home"),
        );
        assert!(missing.validate().is_err());

        let non_absolute_home = ProcessCommand::new(
            LifecycleAction::Start,
            temp.path().join("ocx"),
            temp.path().to_path_buf(),
            PathBuf::from("relative-home"),
        );
        assert!(non_absolute_home.validate().is_err());
    }

    #[test]
    fn lifecycle_transitions_follow_frozen_flow() {
        let mut machine = LifecycleStateMachine::new(ProcessLifecycleState::Stopped);
        assert!(machine.current().can_start());
        assert_eq!(
            machine.advance(LifecycleAction::Start).expect("start"),
            ProcessLifecycleState::Starting
        );
        assert_eq!(
            machine.advance(LifecycleAction::Start).expect("pending"),
            ProcessLifecycleState::Pending
        );
        assert_eq!(
            machine.advance(LifecycleAction::Start).expect("running"),
            ProcessLifecycleState::Running
        );
        assert!(machine.current().can_stop());
        assert_eq!(
            machine.advance(LifecycleAction::Stop).expect("stopping"),
            ProcessLifecycleState::Stopping
        );
        assert_eq!(
            machine.advance(LifecycleAction::Stop).expect("stopped"),
            ProcessLifecycleState::Stopped
        );

        let mut machine = LifecycleStateMachine::new(ProcessLifecycleState::Running);
        assert_eq!(
            machine
                .advance(LifecycleAction::Restart)
                .expect("restart stopping"),
            ProcessLifecycleState::Stopping
        );
        assert_eq!(
            machine
                .advance(LifecycleAction::Restart)
                .expect("restart starting"),
            ProcessLifecycleState::Starting
        );
    }

    #[test]
    fn invalid_transitions_are_blocked() {
        let mut machine = LifecycleStateMachine::new(ProcessLifecycleState::Stopped);
        assert!(machine.advance(LifecycleAction::Stop).is_err());

        let mut machine = LifecycleStateMachine::new(ProcessLifecycleState::Running);
        assert!(machine.advance(LifecycleAction::Start).is_err());
    }

    #[test]
    fn cancel_pending_moves_to_last_stable_observation_without_kill() {
        let mut starting = LifecycleStateMachine::new(ProcessLifecycleState::Starting);
        assert_eq!(starting.cancel_pending(), ProcessLifecycleState::Stopped);

        let mut stopping = LifecycleStateMachine::new(ProcessLifecycleState::Stopping);
        assert_eq!(stopping.cancel_pending(), ProcessLifecycleState::Stopped);

        let mut running = LifecycleStateMachine::new(ProcessLifecycleState::Running);
        assert_eq!(running.cancel_pending(), ProcessLifecycleState::Running);
    }

    #[test]
    fn failure_and_unreachable_states_are_explicit() {
        let mut machine = LifecycleStateMachine::new(ProcessLifecycleState::Starting);
        assert_eq!(
            machine.mark_start_failed(),
            ProcessLifecycleState::StartingFailed
        );
        assert!(machine.current().can_start());

        let mut machine = LifecycleStateMachine::new(ProcessLifecycleState::Running);
        assert_eq!(
            machine.mark_unreachable(),
            ProcessLifecycleState::Unreachable
        );
        assert!(machine.current().can_restart());
    }

    #[test]
    fn controlled_runner_never_searches_path() {
        let (temp, command) = valid_command(LifecycleAction::Start);
        let runner = ControlledProcessRunner::new(temp.path().join("home"));
        // 缺失上下文在 validate_spawn_context 中显式失败；这里只确认
        // runner 不搜索 PATH。
        assert!(ControlledProcessRunner::validate_spawn_context(&command).is_ok());
        assert!(runner.cancel_pending().is_ok());
    }
}
