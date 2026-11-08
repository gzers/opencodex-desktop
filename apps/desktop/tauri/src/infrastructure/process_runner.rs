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

#[cfg(unix)]
type ControlledChild = std::process::Child;
#[cfg(not(unix))]
type ControlledChild = tokio::process::Child;

use crate::errors::AppError;
use crate::modules::process::{runner_start_timeout, stop_timeout};
use crate::modules::process::{LifecycleAction, LifecycleResult, ProcessCommand, ProcessRunner};

#[cfg(unix)]
/// 正常等待结束后发出的温和信号。
const TERM_SIGNAL: i32 = libc::SIGTERM;

/// 发出 SIGTERM 之后，仍需在固定窗口内收口的等待上限。
fn reap_grace() -> Duration {
    crate::modules::runtime_defaults::process_reap_grace()
}

#[derive(Default)]
struct StartDiagnostics {
    tail: zeroize::Zeroizing<Vec<u8>>,
    reason: Option<&'static str>,
}
impl StartDiagnostics {
    fn push(&mut self, bytes: &[u8]) {
        // Only a bounded rolling window is retained, and no upstream text is logged.
        self.tail.extend_from_slice(bytes);
        if self.tail.len() > 8192 {
            let discard = self.tail.len() - 8192;
            self.tail.drain(..discard);
        }
        let text =
            zeroize::Zeroizing::new(String::from_utf8_lossy(&self.tail).to_ascii_lowercase());
        if text.contains("eaddrinuse")
            || text.contains("proxy already running")
            || text.contains("address already in use")
        {
            self.reason = Some("port_conflict");
        } else if self.reason.is_none()
            && (text.contains("eacces") || text.contains("permission denied"))
        {
            self.reason = Some("port_permission");
        }
    }
}

fn capture_start_output(child: &mut ControlledChild) -> Arc<std::sync::Mutex<StartDiagnostics>> {
    let captured = Arc::new(std::sync::Mutex::new(StartDiagnostics::default()));
    #[cfg(unix)]
    fn drain(
        reader: impl std::io::Read + Send + 'static,
        state: Arc<std::sync::Mutex<StartDiagnostics>>,
    ) {
        std::thread::spawn(move || {
            let mut reader = reader;
            let mut buffer = zeroize::Zeroizing::new([0u8; 2048]);
            while let Ok(n) = reader.read(&mut buffer[..]) {
                if n == 0 {
                    break;
                }
                if let Ok(mut state) = state.lock() {
                    state.push(&buffer[..n]);
                }
            }
        });
    }
    #[cfg(not(unix))]
    fn drain(
        reader: impl tokio::io::AsyncRead + Unpin + Send + 'static,
        state: Arc<std::sync::Mutex<StartDiagnostics>>,
    ) {
        tauri::async_runtime::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut reader = reader;
            let mut buffer = zeroize::Zeroizing::new([0u8; 2048]);
            while let Ok(n) = reader.read(&mut buffer[..]).await {
                if n == 0 {
                    break;
                }
                if let Ok(mut state) = state.lock() {
                    state.push(&buffer[..n]);
                }
            }
        });
    }
    if let Some(stdout) = child.stdout.take() {
        drain(stdout, captured.clone());
    }
    if let Some(stderr) = child.stderr.take() {
        drain(stderr, captured.clone());
    }
    captured
}

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

/// 基础设施层的受控子进程适配器。
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
    /// 目前只有 Unix 目标上的单测读取该计数；Windows 侧先保留实现避免死代码告警。
    #[cfg_attr(not(unix), allow(dead_code))]
    pub(crate) fn failure_epoch(&self) -> u64 {
        self.failure_epoch.load(Ordering::SeqCst)
    }

    pub(crate) fn timeout_for(action: LifecycleAction) -> Duration {
        match action {
            LifecycleAction::Start | LifecycleAction::Restart => runner_start_timeout(),
            LifecycleAction::Stop => stop_timeout(),
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
        self.execute_inner_observed(command, None, Self::timeout_for(command.action))
            .await
    }

    async fn execute_inner_observed(
        &self,
        command: &ProcessCommand,
        runtime: Option<crate::infrastructure::runtime_executable::SharedRuntimeExecutable>,
        budget: Duration,
    ) -> Result<LifecycleResult, AppError> {
        let action_started = Instant::now();
        Self::validate_spawn_context(command)?;
        let timeout = Self::timeout_for(command.action).min(budget);

        let mut child = self
            .spawn(command)
            .await
            .inspect_err(|_| {
                self.bump_failure_epoch();
            })
            .map_err(|error| AppError::FileSystem {
                operation: "spawn controlled ocx process".to_string(),
                detail: error.to_string(),
            })?;
        let diagnostics = capture_start_output(&mut child);

        #[cfg(unix)]
        let pid = Some(child.id());
        #[cfg(not(unix))]
        let pid = child.id();
        if matches!(
            command.action,
            LifecycleAction::Start | LifecycleAction::Restart
        ) {
            self.clear_running();
            // 后台独立线程负责回收，避免 start/stop 的子进程变成僵尸。
            // Unix 子进程只交给该线程，不参与 Tokio 的孤儿回收队列。
            let mut exit = observe_child_exit(child);
            // A managed Start is confirmed by fresh authenticated readiness, never spawn.
            // Keep the exit receiver alive until success/failure or the original deadline.
            let Some(runtime) = runtime else {
                return Ok(LifecycleResult::Started);
            };
            let started = action_started;
            let (mut observations, mut pending_ready, mut rejected, mut absent_record) =
                (0u32, 0u32, 0u32, 0u32);
            // Fixed labels/timings only: no server payload, key, path or upstream output.
            let mut sample_timings = Vec::with_capacity(16);
            loop {
                if started.elapsed() >= timeout {
                    #[cfg(test)]
                    if std::env::var_os("OCXD_TEST_LIFECYCLE_DIAGNOSTIC").is_some() {
                        if let Ok(captured) = diagnostics.lock() {
                            for line in
                                String::from_utf8_lossy(&captured.tail).lines().filter(|l| {
                                    ["failed", "Failed", "Error", "error", "home", "HOME"]
                                        .iter()
                                        .any(|word| l.contains(word))
                                })
                            {
                                if let Some(line) = crate::modules::logs::sanitize_line(line) {
                                    eprintln!(
                                        "isolated-start-diagnostic: {}",
                                        line.chars().take(512).collect::<String>()
                                    );
                                }
                            }
                        }
                    }
                    return Err(AppError::RuntimeManaged {code:"lifecycle_timeout".into(),detail:format!("启动未在原预算内证明就绪；observations={observations}, pending={pending_ready}, rejected={rejected}, absent_record={absent_record}, samples={sample_timings:?}。未延长预算或判定运行成功。")});
                }
                match exit.try_recv() {
                    Ok(Some(code)) if code != 0 => {
                        self.bump_failure_epoch();
                        let reason = diagnostics
                            .lock()
                            .ok()
                            .and_then(|d| d.reason)
                            .unwrap_or("lifecycle_start_exit");
                        let message = match reason {
                            "port_conflict" => "端口已被占用，请调整运行端口或重新分配后重试",
                            "port_permission" => "没有权限监听所选端口，请检查权限或调整端口",
                            _ => "官方启动进程退出，服务未确认就绪",
                        };
                        return Err(AppError::RuntimeManaged {
                            code: reason.into(),
                            detail: format!("{message}；exit_code={code}"),
                        });
                    }
                    Ok(None) => {
                        return Err(AppError::RuntimeManaged {
                            code: "lifecycle_start_exit".into(),
                            detail: "官方启动进程未提供正常退出状态".into(),
                        })
                    }
                    _ => {}
                }
                // Before publication there is no endpoint to authenticate. Do
                // not hash the complete managed Node activation every 50ms
                // while its cold child is competing for the same disk/CPU.
                // This is only an early non-success check; every real sample
                // below still performs full activation/HMAC/readback validation.
                if matches!(std::fs::symlink_metadata(command.environment.opencodex_home.join("runtime-port.json")),Err(ref e) if e.kind()==std::io::ErrorKind::NotFound)
                {
                    absent_record += 1;
                    tokio::time::sleep(
                        Duration::from_millis(250).min(timeout.saturating_sub(started.elapsed())),
                    )
                    .await;
                    continue;
                }
                let remaining = timeout.saturating_sub(started.elapsed());
                observations += 1;
                let sample_started = started.elapsed().as_millis();
                let observed = tokio::time::timeout(
                    remaining,
                    super::attested_status::fetch(
                        runtime.clone(),
                        command.environment.opencodex_home.clone(),
                    ),
                )
                .await;
                if sample_timings.len() < 16 {
                    let outcome = match &observed {
                        Ok(Ok(Some(value))) if value["ready"] == true => "ready",
                        Ok(Ok(Some(value))) if value["officialReadiness"] == "pending" => "pending",
                        Ok(Ok(Some(_))) => "not-ready",
                        Ok(Ok(None)) => "no-source",
                        Ok(Err(crate::modules::status::CollectError::Timeout)) | Err(_) => {
                            "timeout"
                        }
                        Ok(Err(crate::modules::status::CollectError::Parse)) => "parse",
                        Ok(Err(_)) => "unreachable",
                    };
                    sample_timings.push((sample_started, started.elapsed().as_millis(), outcome));
                }
                if !matches!(&observed, Ok(Ok(Some(_)))) {
                    rejected += 1;
                }
                if let Ok(Ok(Some(value))) = observed {
                    if value["officialReadiness"] == "pending" {
                        pending_ready += 1;
                    }
                    if value["officialReadiness"] == "failed" {
                        return Err(AppError::RuntimeManaged {code:"lifecycle_readiness_failed".into(),detail:"官方服务已报告启动未就绪，可能存在客户端配置或服务归属问题；请查看诊断，未放宽校验。".into()});
                    }
                    if value["ready"] == true && value["status"] == "running" {
                        if started.elapsed() >= timeout {
                            return Err(AppError::Timeout);
                        }
                        if command
                            .requested_port
                            .is_some_and(|port| value["port"].as_u64() != Some(port.into()))
                        {
                            return Err(AppError::RuntimeManaged {
                                code: "lifecycle_port_changed".into(),
                                detail: "运行端口与已保存端口不一致，未宣称启动成功。".into(),
                            });
                        }
                        // An exit received concurrently takes precedence over a stale observation.
                        if matches!(exit.try_recv(), Ok(Some(code)) if code != 0) {
                            return Err(AppError::RuntimeManaged {
                                code: "lifecycle_start_exit".into(),
                                detail: "启动进程异常退出，未确认持续运行".into(),
                            });
                        }
                        return Ok(LifecycleResult::Started);
                    }
                }
                tokio::time::sleep(
                    Duration::from_millis(250).min(timeout.saturating_sub(started.elapsed())),
                )
                .await;
            }
        }

        let started_at = chrono::Utc::now().to_rfc3339();
        let (cancellation, cancel_guard) = CancellationToken::new();
        let (observation_sender, observation) = mpsc::channel(1);
        self.store_running(observation, Arc::clone(&cancel_guard));
        let started = Instant::now();
        let mut timed_out = false;
        let mut child_exited = false;
        let mut exit_status = None;
        // 子进程由唯一持有 Child 的线程回收，保留真实退出码。
        let mut exit_observer = observe_child_exit(child);

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
            if let Ok(code) = tokio::time::timeout(reap_grace(), exit_observer.recv()).await {
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
            .env("OPENCODEX_HOME", &command.environment.opencodex_home)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .kill_on_drop(false);
        if let Some(port) = command.requested_port {
            process.arg("--port").arg(port.to_string());
        }

        crate::infrastructure::platform::apply_user_environment(
            process.as_std_mut(),
            Some(
                command
                    .environment
                    .home
                    .as_deref()
                    .unwrap_or(self.home.as_os_str()),
            ),
        );
        process.env(
            "PATH",
            command.environment.path.as_ref().map_or_else(
                || {
                    crate::infrastructure::discovery_paths::process_path(
                        &self.home,
                        &crate::infrastructure::discovery_paths::paths_for_home(&self.home),
                    )
                },
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

    #[cfg(not(unix))]
    async fn spawn(&self, command: &ProcessCommand) -> std::io::Result<ControlledChild> {
        self.spawn_environment(command).spawn()
    }

    #[cfg(unix)]
    async fn spawn(&self, command: &ProcessCommand) -> std::io::Result<ControlledChild> {
        // Spawn without registering a Tokio child. Dropping a Tokio Child after
        // extracting its PID would enqueue another reaper competing for waitpid.
        self.spawn_environment(command).as_std_mut().spawn()
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
/// Unix 使用 std Child，专用线程持有并 wait，避免 Tokio 孤儿队列
/// 与裸 waitpid 竞争，导致快速退出的非零状态丢失。超时 / 取消后
/// 线程仍回收其自有子进程，不额外发送强制终止信号。
#[cfg(unix)]
fn spawn_reaper(mut child: std::process::Child) -> mpsc::Receiver<Option<i32>> {
    let (sender, receiver) = mpsc::channel(1);
    std::thread::spawn(move || {
        let code = child.wait().ok().and_then(|status| status.code());
        let _ = sender.blocking_send(code);
    });
    receiver
}

fn observe_child_exit(child: ControlledChild) -> mpsc::Receiver<Option<i32>> {
    #[cfg(unix)]
    {
        spawn_reaper(child)
    }
    #[cfg(not(unix))]
    {
        let (sender, receiver) = mpsc::channel(1);
        tauri::async_runtime::spawn(async move {
            let mut child = child;
            let code = child.wait().await.ok().and_then(|status| status.code());
            let _ = sender.send(code).await;
        });
        receiver
    }
}

#[cfg(not(unix))]
fn terminate(_pid: Option<u32>) {
    // Windows 官方生命周期只取消等待，不强制终止运行中的代理。
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
    fn execute_in_context(
        &self,
        command: &ProcessCommand,
        runtime: &crate::infrastructure::runtime_executable::SharedRuntimeExecutable,
        deadline: Instant,
    ) -> Result<LifecycleResult, AppError> {
        if runtime.managed_installation_root().is_none() {
            return self.execute(command);
        }
        std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    tauri::async_runtime::block_on(self.execute_inner_observed(
                        command,
                        Some(runtime.clone()),
                        deadline.saturating_duration_since(Instant::now()),
                    ))
                })
                .join()
                .unwrap_or_else(|_| Err(AppError::NotConfigured))
        })
    }
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

#[cfg(all(test, unix))]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn rapid_stop_exits_preserve_the_nonzero_status() {
        let mut workers = tokio::task::JoinSet::new();
        for worker in 0..8 {
            workers.spawn(async move {
                let context = context(LifecycleAction::Stop);
                std::fs::write(&context.executable, b"#!/bin/sh\nexit 3\n").expect("write fixture");
                let runner = runner(&context);
                for attempt in 0..64 {
                    assert_eq!(
                        runner
                            .execute_inner(&context.command)
                            .await
                            .expect("stop result"),
                        LifecycleResult::Failed,
                        "lost exit status at worker {worker}, attempt {attempt}"
                    );
                }
            });
        }
        while let Some(result) = workers.join_next().await {
            result.expect("join rapid exits");
        }
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

#[cfg(all(test, unix))]
mod managed_observation_tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    #[derive(Debug)]
    struct Unready {
        root: PathBuf,
        entry: PathBuf,
    }
    impl crate::infrastructure::runtime_executable::RuntimeExecutableProvider for Unready {
        fn executable(&self) -> Option<PathBuf> {
            Some(self.entry.clone())
        }
        fn managed_installation_root(&self) -> Option<PathBuf> {
            Some(self.root.clone())
        }
    }
    async fn run(script: &str) -> (AppError, PathBuf, tempfile::TempDir) {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().canonicalize().unwrap();
        let executable = root.join("only-test-child");
        let home = root.join("H");
        std::fs::create_dir(&home).unwrap();
        std::fs::write(&executable, script).unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
        let runtime = Arc::new(Unready {
            root: root.clone(),
            entry: executable.clone(),
        });
        let runner = ControlledProcessRunner::new(&root);
        let command = ProcessCommand::new(LifecycleAction::Start, executable, root.clone(), home);
        let result = runner
            .execute_inner_observed(&command, Some(runtime), Duration::from_millis(700))
            .await;
        (result.unwrap_err(), root, t)
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn immediate_and_delayed_failure_keep_exit_code_without_secret_output() {
        for script in [
            "#!/bin/sh\necho 'apiKey=synthetic_secret EADDRINUSE' >&2\nexit 17\n",
            "#!/bin/sh\nsleep 0.1\necho 'Proxy already running (port 12345)' >&2\nexit 19\n",
        ] {
            let (e, _root, _t) = run(script).await;
            let text = e.to_string();
            assert!(!text.contains("synthetic_secret"));
            match e {
                AppError::RuntimeManaged { code, detail } => {
                    assert_eq!(code, "port_conflict");
                    assert!(detail.contains("exit_code=17") || detail.contains("exit_code=19"));
                }
                other => panic!("{other:?}"),
            }
        }
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn zero_exit_or_live_process_without_authenticated_ready_is_not_started() {
        for script in ["#!/bin/sh\nexit 0\n", "#!/bin/sh\nsleep 0.3\nexit 0\n"] {
            let (e, _root, _t) = run(script).await;
            assert!(matches!(e, AppError::RuntimeManaged{code,..} if code=="lifecycle_timeout"));
        }
    }
    #[test]
    fn bounded_diagnostics_do_not_keep_or_return_arbitrary_upstream_output() {
        let mut diagnostic = StartDiagnostics::default();
        for _ in 0..500 {
            diagnostic.push(&[b'x'; 2048]);
        }
        assert!(diagnostic.tail.len() <= 8192);
        diagnostic.push(b"permission denied");
        assert_eq!(diagnostic.reason, Some("port_permission"));
        diagnostic.push(b"EADDRINUSE");
        assert_eq!(diagnostic.reason, Some("port_conflict"));
    }
}
