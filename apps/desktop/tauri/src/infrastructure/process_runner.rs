//! 受控子进程运行器。
//!
//! MOD-04 只注入显式可执行路径，本层不搜索 PATH；生命周期窗口遵守
//! FZ-10，取消等待遵守 FZ-09/FZ-10，并禁止 SIGKILL 等强制终止语义。
//! 官方子进程的非零退出码按 FZ-11 原样观测，不改写、不推断成功。

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::process::Command;
use tokio::sync::{mpsc, watch, Notify};

use crate::errors::AppError;
use crate::modules::process::{
    LifecycleAction, LifecycleResult, ProcessCommand, ProcessRunner, RESTART_LIMIT, STOP_TIMEOUT,
};

#[cfg(unix)]
/// 正常等待结束后发出的温和信号。
const TERM_SIGNAL: i32 = libc::SIGTERM;

/// 发出 SIGTERM 之后，仍需在固定窗口内收口的等待上限。
const REAP_GRACE: Duration = Duration::from_secs(5);

/// 统一标记取消请求；只能等待已发出的系统调用结束，不强制终止。
#[derive(Debug)]
pub struct CancellationToken {
    cancelled: watch::Receiver<bool>,
    notify: Arc<Notify>,
}

impl CancellationToken {
    pub fn new() -> (Self, Arc<CancelGuard>) {
        let (sender, receiver) = watch::channel(false);
        let notify = Arc::new(Notify::new());
        (
            Self {
                cancelled: receiver,
                notify: Arc::clone(&notify),
            },
            Arc::new(CancelGuard { sender, notify }),
        )
    }

    pub fn is_cancelled(&self) -> bool {
        *self.cancelled.borrow()
    }
}

/// 取消句柄只切换观测标志；已发出的进程调用仍等待自然结束。
#[derive(Debug)]
pub struct CancelGuard {
    sender: watch::Sender<bool>,
    notify: Arc<Notify>,
}

impl CancelGuard {
    pub fn cancel(&self) {
        let _ = self.sender.send(true);
        self.notify.notify_waiters();
    }
}

/// 官方子进程的一次完整观测；官方退出码不改写。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessObservation {
    pub pid: Option<u32>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
}

/// 运行中的进程观测句柄，供上层取消等待中的动作。
#[derive(Debug)]
pub struct RunningProcess {
    pub observation: mpsc::Receiver<ProcessObservation>,
    pub cancel: Arc<CancelGuard>,
}

/// 基础设施层的 tokio 子进程适配器。
///
/// 只接收 [`ProcessCommand`] 中显式给出的可执行文件；生命周期语义
/// 由调用方状态机解释，runner 自身不把非零退出码重分类。
#[derive(Debug)]
pub struct ControlledProcessRunner {
    home: PathBuf,
    running: std::sync::Mutex<Option<RunningProcess>>,
    failure_epoch: Arc<AtomicU64>,
}

impl ControlledProcessRunner {
    /// `home` 必须来自显式配置或用户家目录；生产代码不注入项目上下文。
    pub fn new(home: impl Into<PathBuf>) -> Self {
        Self {
            home: home.into(),
            running: std::sync::Mutex::new(None),
            failure_epoch: Arc::new(AtomicU64::new(0)),
        }
    }

    fn bump_failure_epoch(&self) {
        self.failure_epoch.fetch_add(1, Ordering::SeqCst);
    }

    #[cfg(test)]
    pub(crate) fn failure_epoch(&self) -> u64 {
        self.failure_epoch.load(Ordering::SeqCst)
    }

    pub(crate) fn timeout_for(action: LifecycleAction) -> Duration {
        match action {
            LifecycleAction::Start | LifecycleAction::Restart => RESTART_LIMIT,
            LifecycleAction::Stop => STOP_TIMEOUT,
        }
    }

    pub(crate) fn validate_spawn_context(command: &ProcessCommand) -> Result<(), AppError> {
        command.validate()?;
        if !command.working_directory.is_absolute() {
            return Err(AppError::FileSystem {
                operation: "validate process working directory".to_string(),
                detail: "working directory must be an absolute path".to_string(),
            });
        }
        if !self_home_is_absolute(command) {
            return Err(AppError::FileSystem {
                operation: "validate process HOME".to_string(),
                detail: "HOME must be an absolute path".to_string(),
            });
        }
        Ok(())
    }

    /// 用冻结窗口执行一次官方生命周期动作。
    async fn execute_inner(&self, command: &ProcessCommand) -> Result<LifecycleResult, AppError> {
        Self::validate_spawn_context(command)?;
        let timeout = Self::timeout_for(command.action);

        let child = self
            .spawn(command)
            .await
            .inspect_err(|_| {
                self.bump_failure_epoch();
            })
            .map_err(|error| AppError::FileSystem {
                operation: "spawn controlled ocx process".to_string(),
                detail: error.to_string(),
            })?;

        let pid = child.id();
        if matches!(
            command.action,
            LifecycleAction::Start | LifecycleAction::Restart
        ) {
            self.clear_running();
            // 后台独立线程负责回收，避免 start/stop 的子进程变成僵尸。
            // 此前这里依赖 `Child::wait()`，而它在本环境下永不就绪。
            let _exit_observer = spawn_reaper(pid);
            return Ok(LifecycleResult::Started);
        }

        let started_at = chrono::Utc::now().to_rfc3339();
        let (cancellation, cancel_guard) = CancellationToken::new();
        let (observation_sender, observation) = mpsc::channel(1);
        self.store_running(observation, Arc::clone(&cancel_guard));
        let started = Instant::now();
        let mut timed_out = false;
        let mut child_exited = false;
        let mut exit_status = None;
        // 子进程退出由独立线程用 `waitpid` 观测；不再依赖 tokio 的 wait。
        let mut exit_observer = spawn_reaper(pid);

        while !child_exited {
            let elapsed = started.elapsed();
            if elapsed >= timeout {
                timed_out = true;
                break;
            }
            tokio::select! {
                biased;
                _ = cancellation.notify.notified() => {
                    // 取消只终止等待；先发 SIGTERM，仍不使用 SIGKILL。
                    terminate(pid);
                }
                _ = tokio::time::sleep(Duration::from_millis(50)) => {}
                code = exit_observer.recv(), if !child_exited => {
                    exit_status = code.flatten();
                    child_exited = true;
                }
            }
        }

        if timed_out {
            if matches!(
                command.action,
                LifecycleAction::Start | LifecycleAction::Restart
            ) {
                return Ok(LifecycleResult::Started);
            }
            terminate(pid);
        }

        // 超时或取消后等待已发出的进程调用自然结束；本层不发送 SIGKILL。
        // 必须在固定窗口内收口：此前这里无限 `await`，当子进程忽略 SIGTERM
        // 时会把 `ocxd stop` 永久挂死。
        if !child_exited {
            if let Ok(code) = tokio::time::timeout(REAP_GRACE, exit_observer.recv()).await {
                exit_status = code.flatten();
                child_exited = true;
            }
        }
        if !child_exited {
            self.clear_running();
            return Ok(LifecycleResult::Cancelled);
        }
        let exit_code = exit_status;

        let observation = ProcessObservation {
            pid,
            started_at: Some(started_at),
            finished_at: Some(chrono::Utc::now().to_rfc3339()),
            exit_code,
            timed_out,
        };
        let _ = observation_sender.send(observation).await;
        self.clear_running();
        Ok(match (command.action, exit_code) {
            (_, Some(0)) => match command.action {
                LifecycleAction::Start | LifecycleAction::Restart => LifecycleResult::Started,
                LifecycleAction::Stop => LifecycleResult::Stopped,
            },
            (_, Some(_)) => LifecycleResult::Failed,
            (_, None) => LifecycleResult::Cancelled,
        })
    }

    fn spawn_environment(&self, command: &ProcessCommand) -> Command {
        let mut process = Command::new(&command.executable);
        process
            .arg(official_subcommand(command.action))
            .current_dir(&command.working_directory)
            .env_clear()
            .env("HOME", self.home.clone())
            .env("OPENCODEX_HOME", &command.environment.opencodex_home)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .kill_on_drop(false);

        if let Some(home) = command.environment.home.as_ref() {
            process.env("HOME", home);
        }
        process.env(
            "PATH",
            command.environment.path.as_ref().map_or_else(
                || self.home.join(".local/bin").into_os_string(),
                |path| path.clone(),
            ),
        );
        for (key, value) in [
            ("LANG", &command.environment.lang),
            ("LC_ALL", &command.environment.lc_all),
            ("HTTP_PROXY", &command.environment.http_proxy),
            ("HTTPS_PROXY", &command.environment.https_proxy),
            ("NO_PROXY", &command.environment.no_proxy),
            ("PATH", &command.environment.path),
        ] {
            if let Some(value) = value {
                process.env(key, value);
            }
        }
        process
    }

    async fn spawn(&self, command: &ProcessCommand) -> std::io::Result<tokio::process::Child> {
        self.spawn_environment(command).spawn()
    }

    fn store_running(
        &self,
        observation: mpsc::Receiver<ProcessObservation>,
        cancel: Arc<CancelGuard>,
    ) {
        *self
            .running
            .lock()
            .expect("process runner state must not be poisoned") = Some(RunningProcess {
            observation,
            cancel,
        });
    }

    fn clear_running(&self) {
        self.running
            .lock()
            .expect("process runner state must not be poisoned")
            .take();
    }
}

#[cfg(unix)]
fn terminate(pid: Option<u32>) {
    if let Some(pid) = pid {
        unsafe {
            _ = libc::kill(pid as libc::pid_t, TERM_SIGNAL);
        }
    }
}

/// 用独立线程回收子进程，并把退出码送回调用方。
///
/// 不使用 `tokio::process::Child::wait()`：实测在本环境（macOS + Tauri 事件循环）
/// 该 future 可能永不就绪，导致 `ocxd stop` 永久挂起，且 `ocx start/stop`
/// 的子进程全部残留为僵尸。`waitpid` 由专用线程阻塞调用，语义简单且必然收敛。
#[cfg(unix)]
fn spawn_reaper(pid: Option<u32>) -> mpsc::Receiver<Option<i32>> {
    let (sender, receiver) = mpsc::channel(1);
    let Some(pid) = pid else {
        // 没有 pid 说明已经被回收；直接给一个空结果，调用方不会永久等待。
        let _ = sender.try_send(None);
        return receiver;
    };
    std::thread::spawn(move || {
        let mut status: libc::c_int = 0;
        let reaped = unsafe { libc::waitpid(pid as libc::pid_t, &mut status, 0) };
        let code = if reaped == pid as libc::pid_t && libc::WIFEXITED(status) {
            Some(libc::WEXITSTATUS(status))
        } else {
            None
        };
        let _ = sender.blocking_send(code);
    });
    receiver
}

#[cfg(not(unix))]
fn spawn_reaper(_pid: Option<u32>) -> mpsc::Receiver<Option<i32>> {
    let (sender, receiver) = mpsc::channel(1);
    let _ = sender.try_send(None);
    receiver
}

#[cfg(not(unix))]
fn terminate(_pid: Option<u32>) {
    // 当前正式目标只有 aarch64-apple-darwin；跨平台强制终止仍另行设计。
}

fn self_home_is_absolute(command: &ProcessCommand) -> bool {
    command
        .environment
        .home
        .as_ref()
        .map(|home| PathBuf::from(home).is_absolute())
        .unwrap_or(true)
}

fn official_subcommand(action: LifecycleAction) -> &'static str {
    match action {
        LifecycleAction::Start => "start",
        LifecycleAction::Stop => "stop",
        LifecycleAction::Restart => "restart",
    }
}

impl ControlledProcessRunner {
    /// 消费当前等待中的观测；已发出的系统调用仍等待自然结束。
    pub fn take_running(&self) -> Option<RunningProcess> {
        self.running
            .lock()
            .expect("process runner state must not be poisoned")
            .take()
    }
}

impl ProcessRunner for ControlledProcessRunner {
    fn execute(&self, command: &ProcessCommand) -> Result<LifecycleResult, AppError> {
        // Tauri 同步 command 运行在主线程，需要切换到共享异步 runtime 的
        // 阻塞池执行；不依赖当前线程是否进入 runtime。
        let command = command.clone();
        let runner_ref: &ControlledProcessRunner = self;
        let result = std::thread::scope(|scope| {
            scope
                .spawn(move || {
                    tokio::runtime::Handle::block_on(
                        tauri::async_runtime::handle().inner(),
                        runner_ref.execute_inner(&command),
                    )
                })
                .join()
                .unwrap_or_else(|panic| {
                    Err(AppError::FileSystem {
                        operation: "execute controlled ocx process".to_string(),
                        detail: format!("runner panicked: {panic:?}"),
                    })
                })
        });
        result
    }

    fn cancel_pending(&self) -> Result<(), AppError> {
        if let Some(running) = self.take_running() {
            running.cancel.cancel();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::time::Duration;

    use super::*;
    use crate::modules::process::{EnvironmentPolicy, LifecycleAction, ProcessCommand};

    struct TestContext {
        _root: tempfile::TempDir,
        command: ProcessCommand,
        executable: PathBuf,
    }

    fn context(action: LifecycleAction) -> TestContext {
        let root = tempfile::tempdir().expect("create temporary fixture");
        let executable = root.path().join("ocx");
        std::fs::write(&executable, b"#!/bin/sh\nexit 0\n").expect("write fixture");
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755))
            .expect("chmod fixture");
        let home = root.path().join("home");
        std::fs::create_dir_all(&home).expect("create home fixture");
        let opencodex_home = root.path().join("opencodex-home");
        std::fs::create_dir_all(&opencodex_home).expect("create opencodex home fixture");
        let command = ProcessCommand::new(action, executable.clone(), home.clone(), opencodex_home);
        TestContext {
            _root: root,
            command,
            executable,
        }
    }

    fn runner(context: &TestContext) -> ControlledProcessRunner {
        ControlledProcessRunner::new(context.command.working_directory.clone())
    }

    #[tokio::test]
    async fn frozen_timeouts_match_fz10() {
        assert_eq!(
            ControlledProcessRunner::timeout_for(LifecycleAction::Start),
            Duration::from_secs(30)
        );
        assert_eq!(
            ControlledProcessRunner::timeout_for(LifecycleAction::Restart),
            Duration::from_secs(30)
        );
        assert_eq!(
            ControlledProcessRunner::timeout_for(LifecycleAction::Stop),
            Duration::from_secs(10)
        );
    }

    #[tokio::test]
    async fn validates_explicit_context_before_spawning() {
        let context = context(LifecycleAction::Start);
        assert!(ControlledProcessRunner::validate_spawn_context(&context.command).is_ok());

        let missing = ProcessCommand::new(
            LifecycleAction::Start,
            context.executable.with_file_name("missing"),
            context.command.working_directory.clone(),
            context.command.environment.opencodex_home.clone(),
        );
        assert!(ControlledProcessRunner::validate_spawn_context(&missing).is_err());

        let non_absolute_workdir = ProcessCommand::new(
            LifecycleAction::Start,
            context.executable.clone(),
            PathBuf::from("relative-workdir"),
            context.command.environment.opencodex_home.clone(),
        );
        assert!(ControlledProcessRunner::validate_spawn_context(&non_absolute_workdir).is_err());

        let relative_home = ProcessCommand {
            environment: EnvironmentPolicy {
                home: Some(PathBuf::from("relative-home").into_os_string()),
                ..context.command.environment.clone()
            },
            ..context.command.clone()
        };
        assert!(ControlledProcessRunner::validate_spawn_context(&relative_home).is_err());
    }

    #[tokio::test]
    async fn failed_spawn_bumps_failure_epoch_without_running_process() {
        let context = context(LifecycleAction::Start);
        let runner = std::sync::Arc::new(runner(&context));
        // 验证器要求常规文件；这里用非可执行位触发真实的 spawn 失败，
        // 确保 failure epoch 记录的是“已进入子进程调用后的失败”。
        let non_executable = context.executable.with_file_name("non-executable");
        std::fs::write(&non_executable, b"fixture").expect("write fixture");
        std::fs::set_permissions(&non_executable, std::fs::Permissions::from_mode(0o644))
            .expect("chmod fixture");
        let invalid = ProcessCommand::new(
            LifecycleAction::Start,
            non_executable,
            context.command.working_directory.clone(),
            context.command.environment.opencodex_home.clone(),
        );

        let task = {
            let runner = std::sync::Arc::clone(&runner);
            tokio::spawn(async move { runner.execute_inner(&invalid).await })
        };
        let error = task
            .await
            .expect("join runner task")
            .expect_err("non-executable file must fail at spawn");
        assert!(matches!(error, AppError::FileSystem { .. }));
        assert_eq!(runner.failure_epoch(), 1);
        assert!(runner.take_running().is_none());
    }

    #[tokio::test]
    async fn cancel_pending_is_repeatable_and_does_not_panic() {
        let context = context(LifecycleAction::Start);
        let runner = runner(&context);
        runner.cancel_pending().expect("first cancel");
        runner.cancel_pending().expect("second cancel");
        assert!(runner.take_running().is_none());
    }

    #[test]
    fn environment_keys_are_frozen() {
        let context = context(LifecycleAction::Start);
        assert_eq!(
            context.command.environment.allowed_key_list(),
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
    }
}
