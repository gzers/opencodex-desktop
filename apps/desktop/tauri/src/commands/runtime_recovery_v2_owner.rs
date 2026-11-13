//! OS-backed current-user/H recovery ownership. No CLI execution, process-name
//! signalling, port eviction, service-manager mutation or credential logging.
use super::runtime_recovery_v2::{ServiceOwnerProof, ServiceOwnerProofProvider};
use crate::{
    errors::{AppError, AppResult},
    modules::{backup::safety, model_config::management::Management},
    state::ProcessContext,
};
use serde::Deserialize;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use zeroize::Zeroizing;

fn fail(code: &str, detail: impl Into<String>) -> AppError {
    AppError::RuntimeManaged {
        code: code.into(),
        detail: detail.into(),
    }
}
fn io(operation: &str, e: impl ToString) -> AppError {
    AppError::FileSystem {
        operation: operation.into(),
        detail: e.to_string(),
    }
}
fn safety_io(operation: &str, e: AppError) -> AppError {
    match e {
        AppError::FileSystem { detail, .. } => io(operation, detail),
        other => other,
    }
}
fn path_key(path: &Path) -> PathBuf {
    let path = dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    #[cfg(windows)]
    {
        PathBuf::from(path.to_string_lossy().to_lowercase())
    }
    #[cfg(not(windows))]
    {
        path
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProcessIdentity {
    pid: u32,
    owner: String,
    created: String,
    image: PathBuf,
    argv: Vec<String>,
    home: Option<PathBuf>,
    zombie: bool,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct RuntimeRecord {
    pid: u32,
    port: u16,
    #[serde(default)]
    hostname: Option<String>,
}
#[derive(Clone)]
struct Record {
    runtime: RuntimeRecord,
    digest: String,
}

/// ObserveOnly is an explicit backend policy, not a fake Unknown implementation.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStopGuard {
    AttestedHandover,
    ObserveOnly,
}

trait Host: Send + Sync {
    fn owner(&self) -> AppResult<String>;
    fn process(&self, pid: u32) -> AppResult<Option<ProcessIdentity>>;
    /// Enumerate this user's runtime-shaped executables with OS argv/environment.
    fn processes(&self) -> AppResult<Vec<ProcessIdentity>>;
    fn program_processes(&self, _root: &Path) -> AppResult<Vec<ProcessIdentity>> {
        self.processes()
    }
    /// Whole-OS listening TCP table (IPv4 + IPv6), not a connection refusal.
    fn port_clear(&self, port: u16) -> AppResult<bool>;
    /// Read-only query of the official per-user service namespaces.
    fn manager_absent(&self) -> AppResult<bool>;
}

pub struct SystemServiceOwnerProofProvider {
    installation_root: PathBuf,
    context: ProcessContext,
    owner: String,
    host: Arc<dyn Host>,
    guard: RecoveryStopGuard,
    last_record: Mutex<Option<Record>>,
}

/// Issued only by the complete ordinary Stop path. A preexisting Stopped
/// observation carries NO newly verified process exit and must never authorize
/// compensation of an unrelated startup snapshot.
pub struct CompletedLifecycleStop {
    context: ProcessContext,
    data_root: PathBuf,
    record_digest: Option<String>,
    deadline: Instant,
    #[cfg(target_os = "macos")]
    normal_exit: Option<crate::infrastructure::macos_process_exit::NormalExitProof>,
    #[cfg(target_os = "macos")]
    tcp_listeners: Option<crate::infrastructure::macos_tcp_listeners::TcpListeners>,
}
/// An action-time observation only. It grants no file mutation or recovery
/// completion authority and cannot be reconstructed from an error code.
pub(crate) struct StoppedObservation {
    _private: (),
}
impl CompletedLifecycleStop {
    #[cfg(target_os = "macos")]
    pub(crate) fn bound_deadline(&self, candidate: Instant) -> Instant {
        self.deadline.min(candidate)
    }
    #[cfg(target_os = "macos")]
    pub(crate) fn observation(
        &self,
        context: &ProcessContext,
        data_root: &Path,
    ) -> AppResult<StoppedObservation> {
        self.require_context(context, data_root)?;
        Ok(StoppedObservation { _private: () })
    }
    fn require_context(&self, context: &ProcessContext, data_root: &Path) -> AppResult<()> {
        if Instant::now() >= self.deadline {
            return Err(fail(
                "lifecycle_timeout",
                "原停止预算已过，未将旧停止证明用于后继恢复",
            ));
        }
        if path_key(data_root) != self.data_root
            || !Arc::ptr_eq(&context.runtime, &self.context.runtime)
            || path_key(&context.opencodex_home) != path_key(&self.context.opencodex_home)
            || path_key(&context.working_directory) != path_key(&self.context.working_directory)
            || self.record_digest.as_ref().is_some_and(|digest| {
                digest.len() != 64
                    || !digest
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            })
        {
            return Err(fail(
                "client_stop_binding_unknown",
                "停止证明的上下文或原记录无法核对，未恢复客户端文件",
            ));
        }
        Ok(())
    }
    #[cfg(target_os = "macos")]
    pub(crate) fn require_original(
        &self,
        context: &ProcessContext,
        data_root: &Path,
        generation: crate::infrastructure::macos_process_exit::Generation,
        record_digest: &str,
    ) -> AppResult<()> {
        self.require_context(context, data_root)?;
        if path_key(data_root) != self.data_root
            || self.record_digest.as_deref() != Some(record_digest)
            || self
                .normal_exit
                .as_ref()
                .is_none_or(|proof| proof.generation() != generation)
            || self
                .tcp_listeners
                .as_ref()
                .is_none_or(|listeners| listeners.generation() != generation)
        {
            return Err(fail(
                "client_stop_binding_unknown",
                "正常停止证明不属于该次启动、HOME或原记录，未恢复客户端文件",
            ));
        }
        Ok(())
    }
    /// Recheck the complete captured TCP set during this SAME normal Stop
    /// action, not just the advertised primary port or a past stopped label.
    #[cfg(target_os = "macos")]
    pub(crate) fn require_original_absence(
        &self,
        data_root: &Path,
        service: crate::infrastructure::macos_process_exit::Generation,
        launcher: crate::infrastructure::macos_process_exit::Generation,
        record_digest: &str,
        port: u16,
        owner: &SystemServiceOwnerProofProvider,
    ) -> AppResult<()> {
        self.require_original(&owner.context, data_root, service, record_digest)?;
        owner.require_excluded_recovery_absence(&owner.context, service, launcher, port)?;
        let listeners = self.tcp_listeners.as_ref().ok_or_else(|| {
            fail(
                "client_stop_binding_unknown",
                "原停止监听集合缺失，未恢复客户端目录",
            )
        })?;
        if !owner.captured_tcp_clear(listeners)? {
            return Err(fail(
                "client_stop_listener_reappeared",
                "原服务监听端口重新被占用，目录恢复未获准",
            ));
        }
        self.require_original(&owner.context, data_root, service, record_digest)
    }
}
impl SystemServiceOwnerProofProvider {
    /// Per-publication checks INSIDE an already owned official native writer
    /// lease. The session factory requires full OS Stopped before issuing the
    /// borrow-scoped permit and again after publication. Do not replace that
    /// admission with this narrower record/process/endpoint recheck.
    #[cfg(target_os = "macos")]
    pub(crate) fn require_excluded_recovery_absence(
        &self,
        context: &ProcessContext,
        service: crate::infrastructure::macos_process_exit::Generation,
        launcher: crate::infrastructure::macos_process_exit::Generation,
        port: u16,
    ) -> AppResult<()> {
        self.same_context(context)?;
        if port == 0
            || service.pid == launcher.pid
            || service.uid != launcher.uid
            || service.uid.to_string() != self.owner
            || !self.service_home_valid()?
            || self.record()?.is_some()
            || self.read("ocx.pid", 64)?.is_some()
            || self.host.process(service.pid)?.is_some()
            || self.host.process(launcher.pid)?.is_some()
            || !self.host.port_clear(port)?
            || !self.host.manager_absent()?
            || self.record()?.is_some()
            || self.read("ocx.pid", 64)?.is_some()
        {
            return Err(fail(
                "client_recovery_service_unconfirmed",
                "原会话进程或监听尚未确认缺席，恢复未完成",
            ));
        }
        Ok(())
    }
    #[cfg(target_os = "macos")]
    pub(crate) fn require_finalization_absence(
        &self,
        context: &ProcessContext,
        service: crate::infrastructure::macos_process_exit::Generation,
        launcher: crate::infrastructure::macos_process_exit::Generation,
        port: u16,
    ) -> AppResult<()> {
        self.same_context(context)?;
        if port == 0
            || service.pid == launcher.pid
            || service.uid != launcher.uid
            || service.uid.to_string() != self.owner
            || self.query_inner()? != ServiceOwnerProof::Stopped
            || self.host.process(service.pid)?.is_some()
            || self.host.process(launcher.pid)?.is_some()
            || !self.host.port_clear(port)?
            || self.query_inner()? != ServiceOwnerProof::Stopped
        {
            return Err(fail(
                "client_recovery_service_unconfirmed",
                "原会话服务、启动进程或监听尚未确认缺席，未完成恢复收尾",
            ));
        }
        Ok(())
    }
    #[cfg(target_os = "macos")]
    fn captured_tcp_clear(
        &self,
        listeners: &crate::infrastructure::macos_tcp_listeners::TcpListeners,
    ) -> AppResult<bool> {
        for port in listeners.ports() {
            if !self.host.port_clear(*port)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    /// Removal covers all selected images, including another HOME of this OS
    /// user. Windows shared-install proof must come from its native lifetime
    /// backend; do not turn the current-H observer into an all-SID assertion.
    pub(crate) fn require_program_unused(&self, context: &ProcessContext) -> AppResult<()> {
        self.same_context(context)?;
        if !cfg!(target_os = "macos") {
            return Err(fail(
                "removal_platform_proof_unavailable",
                "当前平台的完整程序占用证明尚未接入，未卸载",
            ));
        }
        if self.query_inner()? != ServiceOwnerProof::Stopped {
            return Err(fail(
                "removal_service_unconfirmed",
                "当前OpenCodex服务尚未确认停止，未卸载",
            ));
        }
        let root = path_key(&self.installation_root.join("runtime"));
        for process in self.host.program_processes(&root)? {
            if process.zombie {
                continue;
            }
            let using = path_key(&process.image).starts_with(&root)
                || process.argv.iter().skip(1).any(|arg| {
                    let path = Path::new(arg);
                    path.is_absolute() && path_key(path).starts_with(&root)
                });
            if using {
                return Err(fail(
                    "removal_program_in_use",
                    "当前用户的其它进程仍使用本安装目录，未卸载",
                ));
            }
        }
        Ok(())
    }
    /// Ordinary lifecycle stop retains the official connected-client teardown.
    /// Unlike an installation handover it never writes a deferral receipt.
    pub(crate) fn stop_lifecycle(
        &self,
        context: &ProcessContext,
        action_deadline: Instant,
        data_root: &Path,
    ) -> AppResult<()> {
        let proof = self.stop_lifecycle_verified(context, action_deadline, data_root)?;
        proof.require_context(context, data_root)?;
        // Exercise the same exact binding check on the existing product path;
        // this does not claim an absent record had a current process exit.
        #[cfg(target_os = "macos")]
        if let (Some(exit), Some(digest)) = (&proof.normal_exit, &proof.record_digest) {
            proof.require_original(context, data_root, exit.generation(), digest)?;
        }
        Ok(())
    }
    pub(crate) fn stop_lifecycle_verified(
        &self,
        context: &ProcessContext,
        action_deadline: Instant,
        data_root: &Path,
    ) -> AppResult<CompletedLifecycleStop> {
        let started = Instant::now();
        let budget = crate::modules::process::stop_timeout()
            .min(action_deadline.saturating_duration_since(started));
        let proof_deadline = started + budget;
        let phase = std::cell::Cell::new("context");
        let timings = std::cell::RefCell::new(Vec::with_capacity(16));
        let mark = |next: &'static str| {
            phase.set(next);
            if timings.borrow().len() < 16 {
                timings
                    .borrow_mut()
                    .push((next, started.elapsed().as_millis()));
            }
        };
        let remaining = || {
            budget
                .checked_sub(started.elapsed())
                .filter(|d| !d.is_zero())
                .ok_or(AppError::Timeout)
        };
        let result = (|| {
            self.same_context(context)?;
            mark("initial-owner");
            let (identity, pid) = match self.query_inner()? {
                ServiceOwnerProof::Owned { identity, pid } => (identity, pid),
                ServiceOwnerProof::Stopped => {
                    remaining()?;
                    return Ok(CompletedLifecycleStop {
                        context: context.clone(),
                        data_root: path_key(data_root),
                        record_digest: None,
                        deadline: proof_deadline,
                        #[cfg(target_os = "macos")]
                        normal_exit: None,
                        #[cfg(target_os = "macos")]
                        tcp_listeners: None,
                    });
                }
                ServiceOwnerProof::Unknown => {
                    return Err(fail(
                        "lifecycle_owner_unknown",
                        "无法证明当前用户服务归属，未发送停止请求",
                    ))
                }
            };
            mark("recheck-owner");
            let record = self.checked_owned(&identity, pid)?;
            mark("service-manager");
            if !self.host.manager_absent()? {
                return Err(fail(
                    "lifecycle_supervised",
                    "服务由官方服务管理器托管，未绕过其停止及还原流程",
                ));
            }
            #[cfg(target_os = "macos")]
            let (mut native_exit, native_listeners) = {
                use crate::infrastructure::macos_process_exit::{ExitWatch, Generation};
                let actual = self.host.process(pid)?.ok_or_else(|| {
                    fail(
                        "lifecycle_exit_identity",
                        "原服务进程已无法订阅，未发送停止请求",
                    )
                })?;
                let (seconds, micros) = actual
                    .created
                    .split_once(':')
                    .ok_or_else(|| fail("lifecycle_exit_identity", "原服务进程代次不可核对"))?;
                let expected = Generation {
                    pid,
                    uid: actual
                        .owner
                        .parse()
                        .map_err(|_| fail("lifecycle_exit_identity", "原服务用户不可核对"))?,
                    start_seconds: seconds
                        .parse()
                        .map_err(|_| fail("lifecycle_exit_identity", "原服务创建时间不可核对"))?,
                    start_micros: micros
                        .parse()
                        .map_err(|_| fail("lifecycle_exit_identity", "原服务创建时间不可核对"))?,
                };
                let watch = ExitWatch::capture(expected)?;
                mark("native-tcp-listeners");
                let listeners = crate::infrastructure::macos_tcp_listeners::TcpListeners::capture(
                    expected,
                    proof_deadline,
                )?;
                if !listeners.ports().contains(&record.runtime.port) {
                    return Err(fail(
                        "lifecycle_listener_binding",
                        "运行记录端口尚未证明由该代次服务进程监听，未发送停止请求",
                    ));
                }
                let _ = crate::infrastructure::runtime_log::RuntimeLog::new(data_root)
                    .append_event(&format!(
                        "stop-proof: native-tcp-listeners captured count={}",
                        listeners.ports().len()
                    ));
                (watch, listeners)
            };
            let wait_budget = remaining()?;
            let verified_stop = tauri::async_runtime::block_on(async {
                tokio::time::timeout(wait_budget, async {
                    mark("final-owner-before-stop");
                    self.checked_owned(&identity, pid)?;
                    mark("authenticated-official-stop");
                    crate::infrastructure::attested_status::stop_official(
                        context.runtime.clone(),
                        context.opencodex_home.clone(),
                        pid,
                        &mark,
                    )
                    .await
                    .map_err(|_| {
                        fail(
                            "lifecycle_stop_unknown",
                            "官方停止或客户端还原结果未确认；未强制终止或宣称已停止",
                        )
                    })
                })
                .await
            })
            .map_err(|_| AppError::Timeout)??;
            #[cfg(windows)]
            let mut witness = verified_stop;
            #[cfg(not(windows))]
            let _ = verified_stop;
            mark("wait-os-absence");
            loop {
                remaining()?;
                #[cfg(windows)]
                {
                    // This signed own-H generation was captured while live.
                    // Retain its original process/Job/lease; never enumerate or
                    // skip inaccessible processes from another user's HOME.
                    let empty = witness.quiescent().map_err(|_| {
                        fail(
                            "lifecycle_stop_unknown",
                            "原生生命周期证明不可读，未宣称已停止",
                        )
                    })?;
                    if empty
                        && self.record()?.is_none()
                        && self.read("ocx.pid", 64)?.is_none()
                        && self.host.process(pid)?.is_none()
                        && self.host.port_clear(record.runtime.port)?
                        && self.host.manager_absent()?
                    {
                        // Witness still owns the exclusive lease byte here.
                        // Close late publication/respawn races before return.
                        if self.record()?.is_none()
                            && self.read("ocx.pid", 64)?.is_none()
                            && witness
                                .quiescent()
                                .map_err(|_| fail("lifecycle_stop_unknown", "最终租约证明不可读"))?
                        {
                            remaining()?;
                            mark("final-os-absence");
                            return Ok(CompletedLifecycleStop {
                                context: context.clone(),
                                data_root: path_key(data_root),
                                record_digest: Some(record.digest.clone()),
                                deadline: proof_deadline,
                            });
                        }
                    }
                }
                #[cfg(not(windows))]
                if self.query_stop_observation()? == ServiceOwnerProof::Stopped {
                    mark("final-os-absence");
                    // Require process generation, configured HOME, socket and
                    // service-manager absence; missing record is insufficient.
                    if self.host.process(pid)?.is_none()
                        && self.host.port_clear(record.runtime.port)?
                    {
                        #[cfg(target_os = "macos")]
                        if !self.captured_tcp_clear(&native_listeners)? {
                            std::thread::sleep(Duration::from_millis(10));
                            continue;
                        }
                        #[cfg(target_os = "macos")]
                        let normal_exit = match native_exit.normal_exit(action_deadline)? {
                            Some(proof) => proof,
                            None => {
                                std::thread::sleep(Duration::from_millis(10));
                                continue;
                            }
                        };
                        #[cfg(target_os = "macos")]
                        let _ = crate::infrastructure::runtime_log::RuntimeLog::new(data_root)
                            .append_event("stop-proof: original-native-exit confirmed code=0");
                        #[cfg(target_os = "macos")]
                        let _ = crate::infrastructure::runtime_log::RuntimeLog::new(data_root)
                            .append_event(&format!(
                                "stop-proof: captured-tcp-listeners absent count={}",
                                native_listeners.ports().len()
                            ));
                        remaining()?;
                        return Ok(CompletedLifecycleStop {
                            context: context.clone(),
                            data_root: path_key(data_root),
                            record_digest: Some(record.digest.clone()),
                            deadline: proof_deadline,
                            #[cfg(target_os = "macos")]
                            normal_exit: Some(normal_exit),
                            #[cfg(target_os = "macos")]
                            tcp_listeners: Some(native_listeners),
                        });
                    }
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        })();
        if result.is_err() {
            // Fixed internal stages and durations only. Do not expose paths,
            // HTTP bodies, tokens or native proof material in the public DTO.
            let _ = crate::infrastructure::runtime_log::RuntimeLog::new(data_root).append_event(
                &format!(
                    "stop-proof: failed phase={} elapsed_ms={} timings={:?}",
                    phase.get(),
                    started.elapsed().as_millis(),
                    timings.borrow(),
                ),
            );
        }
        result.map_err(|error| match error {
            AppError::Timeout => fail("lifecycle_timeout", format!(
                "停止未在原预算内完成全部证明；phase={}, elapsed_ms={}, timings={:?}。未延长预算或宣称已停止。",
                phase.get(), started.elapsed().as_millis(), timings.borrow(),
            )),
            other => other,
        })
    }
    pub fn new(installation_root: PathBuf, process_context: ProcessContext) -> AppResult<Self> {
        Self::with_guard(
            installation_root,
            process_context,
            RecoveryStopGuard::AttestedHandover,
        )
    }
    pub fn with_guard(
        installation_root: PathBuf,
        process_context: ProcessContext,
        guard: RecoveryStopGuard,
    ) -> AppResult<Self> {
        if !installation_root.is_absolute()
            || !process_context.opencodex_home.is_absolute()
            || !process_context.working_directory.is_absolute()
        {
            return Err(fail(
                "recovery_owner_context",
                "服务归属查询需要启动时固定的 I、H 和工作目录",
            ));
        }
        let host: Arc<dyn Host> = Arc::new(PlatformHost);
        let owner = host.owner()?;
        Ok(Self {
            installation_root: path_key(&installation_root),
            context: process_context,
            owner,
            host,
            guard,
            last_record: Mutex::new(None),
        })
    }
    fn same_context(&self, context: &ProcessContext) -> AppResult<()> {
        if !Arc::ptr_eq(&context.runtime, &self.context.runtime)
            || path_key(&context.opencodex_home) != path_key(&self.context.opencodex_home)
            || path_key(&context.working_directory) != path_key(&self.context.working_directory)
            || self.host.owner()? != self.owner
        {
            return Err(fail(
                "recovery_owner_context_changed",
                "服务归属查询的用户或启动上下文已变化",
            ));
        }
        Ok(())
    }
    fn read(&self, name: &str, limit: u64) -> AppResult<Option<Vec<u8>>> {
        let root = &self.context.opencodex_home;
        let path = root.join(name);
        safety::check_path(root, &path, true)
            .map_err(|e| safety_io("inspect recovery owner record", e))?;
        match std::fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(io("inspect recovery owner record", e)),
            Ok(_) => safety::read_file(&path, limit)
                .map(Some)
                .map_err(|e| safety_io("read recovery owner record", e)),
        }
    }
    fn record(&self) -> AppResult<Option<Record>> {
        let Some(bytes) = self.read("runtime-port.json", 4096)? else {
            return Ok(None);
        };
        let bytes = Zeroizing::new(bytes);
        let runtime: RuntimeRecord = serde_json::from_slice(&bytes).map_err(|_| {
            fail(
                "recovery_runtime_record_invalid",
                "H 的服务运行记录无效；未推断为已停止",
            )
        })?;
        if runtime.pid == 0
            || (cfg!(target_os = "macos") && runtime.pid > i32::MAX as u32)
            || runtime.port == 0
            || runtime
                .hostname
                .as_ref()
                .is_some_and(|h| !["127.0.0.1", "localhost", "0.0.0.0"].contains(&h.as_str()))
        {
            return Err(fail(
                "recovery_runtime_record_invalid",
                "运行记录不是受支持的本机 IPv4 管理身份",
            ));
        }
        Ok(Some(Record {
            runtime,
            digest: crate::infrastructure::hash::sha256_hex(&bytes),
        }))
    }
    fn service_home_valid(&self) -> AppResult<bool> {
        let Some(bytes) = self.read("service-state.json", 128 * 1024)? else {
            return Ok(true);
        };
        let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|_| {
            fail(
                "recovery_service_record_invalid",
                "服务安装记录损坏，归属未知",
            )
        })?;
        let home = value["opencodexHome"].as_str().map(PathBuf::from);
        Ok(matches!(value["version"].as_u64(), Some(1 | 2))
            && home.as_deref().is_some_and(|p| {
                p.is_absolute() && path_key(p) == path_key(&self.context.opencodex_home)
            })
            && value.get("ownership").is_none_or(|v| v["owner"] == "cli"))
    }
    fn process_home_matches(&self, process: &ProcessIdentity) -> bool {
        // Official process IPC always supplies explicit OPENCODEX_HOME. Never
        // infer another process's H from this manager's HOME or profile: that
        // process may have been launched with a different environment.
        process
            .home
            .as_ref()
            .is_some_and(|h| path_key(h) == path_key(&self.context.opencodex_home))
    }
    fn managed_process(&self, process: &ProcessIdentity) -> bool {
        if process.zombie || process.owner != self.owner || !self.process_home_matches(process) {
            return false;
        }
        let image = path_key(&process.image);
        let (prefix, node, bun) = if let Some(root) =
            self.context.runtime.managed_installation_root()
        {
            if path_key(&root) != self.installation_root {
                return false;
            }
            let Ok(program) = self.context.runtime.observed_program_identity() else {
                return false;
            };
            if path_key(&program.installation_root) != self.installation_root {
                return false;
            }
            if program.prefix == root.join("runtime/opencodex") {
                (
                    program.prefix,
                    program.node,
                    root.join(if cfg!(windows) {
                        "runtime/bun/bun.exe"
                    } else {
                        "runtime/bun/bun"
                    }),
                )
            } else {
                let Ok((_, platform)) = crate::modules::runtime::program_layout::host_target()
                else {
                    return false;
                };
                let Ok(paths) = crate::modules::runtime::program_layout::RuntimeVersionPaths::new(
                    &root.join("runtime/versions"),
                    &program.package_version,
                    platform,
                ) else {
                    return false;
                };
                if program.prefix != paths.package_root || program.node != paths.node {
                    return false;
                }
                (paths.package_root, paths.node, paths.bun)
            }
        } else {
            // Explicit non-versioned providers used by the existing isolated
            // fixture/recovery contract retain their own fixed installation.
            (
                self.installation_root.join("runtime/opencodex"),
                crate::modules::runtime::install::managed_dependency_paths(&self.installation_root)
                    .0,
                self.installation_root.join(if cfg!(windows) {
                    "runtime/bun/bun.exe"
                } else {
                    "runtime/bun/bun"
                }),
            )
        };
        let package_bun = prefix.join("node_modules/bun/bin/bun.exe");
        let native_bun = image.starts_with(prefix.join("node_modules/@oven"))
            && matches!(
                image.file_name().and_then(|s| s.to_str()),
                Some("bun" | "bun.exe")
            );
        if image != path_key(&node)
            && image != path_key(&bun)
            && image != path_key(&package_bun)
            && !native_bun
        {
            return false;
        }
        process.argv.iter().skip(1).any(|arg| {
            let arg = Path::new(arg);
            arg.is_absolute()
                && [
                    "bin/ocx.mjs",
                    "bin/ocx.js",
                    "src/index.ts",
                    "src/cli/index.ts",
                ]
                .iter()
                .any(|entry| {
                    path_key(arg)
                        == path_key(&prefix.join("node_modules/@bitkyc08/opencodex").join(entry))
                })
        })
    }
    fn identity(&self, process: &ProcessIdentity, record: &Record) -> AppResult<String> {
        let bytes = serde_json::to_vec(&(
            process.pid,
            &process.owner,
            &process.created,
            path_key(&process.image),
            path_key(&self.context.opencodex_home),
            &record.digest,
        ))
        .map_err(|e| io("serialize recovery process identity", e))?;
        Ok(crate::infrastructure::hash::sha256_hex(&bytes))
    }
    #[cfg(not(windows))]
    fn query_stop_observation(&self) -> AppResult<ServiceOwnerProof> {
        match self.query_inner() {
            // A confirmed same-generation zombie is not absence or normal
            // exit. Keep waiting inside the ORIGINAL stop deadline; the final
            // live process scan, exit watch and listener proofs remain below.
            Err(AppError::RuntimeManaged { code, .. }) if code == "recovery_process_exiting" => {
                Ok(ServiceOwnerProof::Unknown)
            }
            other => other,
        }
    }
    fn query_inner(&self) -> AppResult<ServiceOwnerProof> {
        if !self.service_home_valid()? {
            return Ok(ServiceOwnerProof::Unknown);
        }
        let record = self.record()?;
        let pid_bytes = self.read("ocx.pid", 64)?;
        let pid_file = pid_bytes
            .as_ref()
            .map(|b| {
                std::str::from_utf8(b)
                    .ok()
                    .and_then(|s| s.trim().parse::<u32>().ok())
                    .filter(|p| *p != 0)
                    .ok_or_else(|| fail("recovery_pid_record_invalid", "H 的 PID 记录无效"))
            })
            .transpose()?;
        if let Some(record) = &record {
            if pid_file.is_some_and(|pid| pid != record.runtime.pid) {
                return Ok(ServiceOwnerProof::Unknown);
            }
            if let Some(process) = self.host.process(record.runtime.pid)? {
                if !self.managed_process(&process) {
                    return Ok(ServiceOwnerProof::Unknown);
                }
                // A second full process snapshot closes argv/environment/PID reuse races.
                if self.host.process(process.pid)?.as_ref() != Some(&process) {
                    return Ok(ServiceOwnerProof::Unknown);
                }
                let identity = self.identity(&process, record)?;
                *self
                    .last_record
                    .lock()
                    .map_err(|_| fail("recovery_owner_lock", "服务归属会话锁不可用"))? =
                    Some(record.clone());
                return Ok(ServiceOwnerProof::Owned {
                    identity,
                    pid: process.pid,
                });
            }
        } else if pid_file.is_some() {
            // With no endpoint record a residual pid cannot prove which old port
            // to inspect, even if its number is currently absent.
            return Ok(ServiceOwnerProof::Unknown);
        }
        let remembered = self
            .last_record
            .lock()
            .map_err(|_| fail("recovery_owner_lock", "服务归属会话锁不可用"))?
            .clone();
        let captured = record.as_ref().or(remembered.as_ref());
        if let Some(record) = captured {
            if self.host.process(record.runtime.pid)?.is_some()
                || !self.host.port_clear(record.runtime.port)?
            {
                return Ok(ServiceOwnerProof::Unknown);
            }
        }
        // Missing record alone is never Stopped. Require an actual current-user
        // process/environment scan and no official job able to respawn this H.
        for process in self.host.processes()? {
            let runtime_command = process.home.is_some()
                || matches!(
                    process.image.file_name().and_then(|s| s.to_str()),
                    Some("ocx" | "ocx.exe" | "opencodex" | "opencodex.exe")
                )
                || process.argv.iter().any(|s| {
                    s.replace('\\', "/").contains("/opencodex/") || s.contains("@bitkyc08")
                });
            if runtime_command && (process.home.is_none() || self.process_home_matches(&process)) {
                return Ok(ServiceOwnerProof::Unknown);
            }
        }
        if !self.host.manager_absent()? {
            return Ok(ServiceOwnerProof::Unknown);
        }
        // Recheck record/PID after the OS scan, rather than trusting its beginning.
        if self.record()?.as_ref().map(|r| &r.digest) != record.as_ref().map(|r| &r.digest)
            || self.read("ocx.pid", 64)? != pid_bytes
        {
            return Ok(ServiceOwnerProof::Unknown);
        }
        Ok(ServiceOwnerProof::Stopped)
    }
    fn checked_owned(&self, identity: &str, pid: u32) -> AppResult<Record> {
        if self.query_inner()?
            != (ServiceOwnerProof::Owned {
                identity: identity.into(),
                pid,
            })
        {
            return Err(fail(
                "recovery_service_changed",
                "服务用户、PID 创建时间、程序入口或 H 运行记录已变化",
            ));
        }
        self.record()?
            .ok_or_else(|| fail("recovery_service_changed", "停止前运行记录已消失"))
    }
    fn stop_inner(&self, identity: &str, pid: u32) -> AppResult<()> {
        if self.guard == RecoveryStopGuard::ObserveOnly {
            return Err(fail(
                "recovery_stop_guard",
                "当前后端策略仅允许归属观测，不允许停止服务",
            ));
        }
        let record = self.checked_owned(identity, pid)?;
        if !self.host.manager_absent()? {
            return Err(fail("recovery_service_supervised", "已识别官方服务管理器；恢复未卸载或停用注册，请先由该用户的服务管理入口停止后重新预览"));
        }
        tauri::async_runtime::block_on(async {
            let management = Management::bind(&self.context)
                .await
                .map_err(|e| fail(&e.code, e.message))?;
            management
                .verify_binding()
                .map_err(|e| fail(&e.code, e.message))?;
            self.checked_owned(identity, pid)?;
            let client = reqwest::Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .connect_timeout(Duration::from_secs(2))
                .timeout(Duration::from_secs(12))
                .build()
                .map_err(|_| fail("recovery_stop_transport", "本机停止连接不可用"))?;
            // Read reusable credentials only after Management proves this H's key.
            let token = Zeroizing::new(
                self.read("admin-api-token", 512)?
                    .ok_or_else(|| fail("recovery_stop_auth", "缺少服务管理令牌"))?,
            );
            let token = std::str::from_utf8(&token)
                .map_err(|_| fail("recovery_stop_auth", "服务管理令牌编码无效"))?
                .trim();
            if !token.starts_with("ocx_admin_")
                || token.len() != 53
                || !token[10..]
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            {
                return Err(fail("recovery_stop_auth", "服务管理令牌格式无效"));
            }
            let capability = client
                .get(format!("{}/api/stop", management.base_url()))
                .bearer_auth(token)
                .send()
                .await
                .map_err(|_| {
                    fail(
                        "recovery_stop_capability",
                        "未能验证配置保留停止协议，未发送停止请求",
                    )
                })?;
            let capability = bounded_json(capability).await?;
            if capability["schema"] != "ocx-stop-capabilities/1"
                || capability["connectedClientOwnership"] != true
                || capability["pid"].as_u64() != Some(pid.into())
            {
                return Err(fail(
                    "recovery_stop_capability",
                    "当前服务未声明配置保留停止协议，未停止",
                ));
            }
            let health = client
                .get(format!("{}/healthz", management.base_url()))
                .send()
                .await
                .map_err(|_| {
                    fail(
                        "recovery_stop_capability",
                        "停止前无法复核维护 runtime 版本",
                    )
                })?;
            let health = bounded_json(health).await?;
            let supported = health["version"].as_str().is_some_and(|version| {
                option_env!("OCXD_BUNDLED_RUNTIME_SOURCE_VERSION") == Some(version)
                    || ["2.97.0-desktop023.names.2", "2.97.0-post021.ownerquery.2"]
                        .contains(&version)
            });
            if !supported || health["pid"].as_u64() != Some(pid.into()) {
                return Err(fail(
                    "recovery_stop_capability",
                    "运行服务不是已知支持 handover 保留协议的维护版本；未停止",
                ));
            }
            management
                .verify_binding()
                .map_err(|e| fail(&e.code, e.message))?;
            self.checked_owned(identity, pid)?;
            let mut receipt = HandoverReceipt::create(&self.context.opencodex_home, &record)?;
            // Retain the durable handover if a POST may have been accepted. A new
            // runtime's first publication owns its discharge; never invent undo.
            management
                .verify_binding()
                .map_err(|e| fail(&e.code, e.message))?;
            self.checked_owned(identity, pid)?;
            receipt.sent = true;
            let response = client
                .post(format!(
                    "{}/api/stop?deferSharedTeardown=1&teardownNonce={}",
                    management.base_url(),
                    receipt.nonce
                ))
                .bearer_auth(token)
                .header(reqwest::header::CONTENT_LENGTH, "0")
                .send()
                .await
                .map_err(|_| {
                    fail(
                        "recovery_stop_unknown",
                        "停止响应未确认；保留本次配置 handover 回执，未再次停止或撤回",
                    )
                })?;
            if !response.status().is_success() {
                // Only documented admission rejections prove no stop accepted.
                // A server error may follow a side effect and keeps the handover.
                receipt.sent = !matches!(response.status().as_u16(), 401 | 403 | 404 | 405 | 409);
                return Err(fail(
                    "recovery_stop_refused",
                    format!(
                        "服务拒绝停止（HTTP {}）；未发送进程信号",
                        response.status().as_u16()
                    ),
                ));
            }
            let response = bounded_json(response).await?;
            if response["success"] != true
                || !matches!(
                    response["sharedTeardown"].as_str(),
                    Some("deferred" | "not-owned")
                )
            {
                return Err(fail(
                    "recovery_stop_unconfirmed",
                    "服务未确认配置保留停止；保留 handover 回执，未执行强制停止",
                ));
            }
            Ok(())
        })?;
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            match self.query_inner()? {
                ServiceOwnerProof::Stopped => return Ok(()),
                ServiceOwnerProof::Owned {
                    identity: current, ..
                } if current == identity => {}
                _ => {} // Observe only; never retry stop against a new identity.
            }
            if Instant::now() >= deadline {
                return Err(fail(
                    "recovery_stop_timeout",
                    "已请求服务自行停止，但仍未证明本用户运行实例缺席；未替换程序",
                ));
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}
impl ServiceOwnerProofProvider for SystemServiceOwnerProofProvider {
    fn query(&self, context: &ProcessContext) -> AppResult<ServiceOwnerProof> {
        self.same_context(context)?;
        self.query_inner()
    }
    fn stop_owned(&self, context: &ProcessContext, identity: &str, pid: u32) -> AppResult<()> {
        self.same_context(context)?;
        self.stop_inner(identity, pid)
    }
}
async fn bounded_json(mut response: reqwest::Response) -> AppResult<serde_json::Value> {
    if !response.status().is_success() {
        return Err(fail("recovery_stop_capability", "服务未接受停止协议查询"));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| fail("recovery_stop_response", "停止协议响应读取失败"))?
    {
        if bytes.len() + chunk.len() > 16 * 1024 {
            return Err(fail("recovery_stop_response", "停止协议响应超限"));
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes)
        .map_err(|_| fail("recovery_stop_response", "停止协议响应格式无效"))
}
struct HandoverReceipt {
    path: PathBuf,
    nonce: String,
    sent: bool,
}
impl HandoverReceipt {
    fn create(home: &Path, record: &Record) -> AppResult<Self> {
        use crate::infrastructure::platform::OpenOptionsModeExt;
        use std::io::Write;
        let nonce = uuid::Uuid::new_v4().simple().to_string();
        let path = home.join(format!("pending-teardown-{nonce}.json"));
        safety::check_path(home, &path, true)
            .map_err(|e| safety_io("inspect recovery handover receipt", e))?;
        let raw = serde_json::to_vec(&serde_json::json!({"ownerPid":std::process::id(),"nonce":nonce,
            "createdAt":chrono::Utc::now().to_rfc3339(),"endpoint":{"hostname":"127.0.0.1","port":record.runtime.port},
            "endpointSource":"exact","handover":"update-restart"})).map_err(|e| io("serialize recovery handover receipt", e))?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .create_mode(0o600)
            .open(&path)
            .map_err(|e| io("create recovery handover receipt", e))?;
        let result = Self {
            path,
            nonce,
            sent: false,
        };
        file.write_all(&raw)
            .and_then(|_| file.sync_all())
            .map_err(|e| io("write recovery handover receipt", e))?;
        Ok(result)
    }
}
impl Drop for HandoverReceipt {
    fn drop(&mut self) {
        if !self.sent {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

/// Bounded Unix system-only read process. Nothing launches the installed CLI.
#[cfg(unix)]
async fn system_query(path: &Path, args: &[String]) -> AppResult<(i32, Vec<u8>)> {
    use tokio::io::AsyncReadExt;
    let mut command = tokio::process::Command::new(path);
    command
        .args(args)
        .env_clear()
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command
        .spawn()
        .map_err(|e| io("spawn recovery OS query", e))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| fail("recovery_os_query", "系统查询未返回输出管道"))?
        .take(2 * 1024 * 1024 + 1);
    let mut bytes = Vec::new();
    let result = tokio::time::timeout(Duration::from_secs(8), async {
        let (status, _) = tokio::try_join!(child.wait(), stdout.read_to_end(&mut bytes))?;
        Ok::<_, std::io::Error>(status)
    })
    .await;
    let status = result
        .map_err(|_| fail("recovery_os_query_timeout", "系统归属查询超时；归属未判定"))?
        .map_err(|e| io("read recovery OS query", e))?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err(fail("recovery_os_query_limit", "系统归属查询输出超限"));
    }
    Ok((status.code().unwrap_or(-1), bytes))
}

struct PlatformHost;

#[cfg(windows)]
mod native {
    use super::*;
    use std::{
        ffi::c_void,
        os::windows::{
            ffi::OsStringExt,
            io::{AsRawHandle, FromRawHandle, OwnedHandle},
        },
        ptr,
    };
    use windows_sys::Win32::{
        Foundation::{
            GetLastError, LocalFree, ERROR_INVALID_PARAMETER, ERROR_NO_MORE_FILES, FILETIME,
            HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT,
        },
        Security::{GetTokenInformation, TokenUser, TOKEN_QUERY, TOKEN_USER},
        System::{
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
                TH32CS_SNAPPROCESS,
            },
            Threading::{
                GetProcessTimes, OpenProcess, OpenProcessToken, QueryFullProcessImageNameW,
                WaitForSingleObject, PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION,
                PROCESS_SYNCHRONIZE,
            },
        },
    };
    #[link(name = "kernel32")]
    extern "system" {
        fn ReadProcessMemory(
            process: HANDLE,
            address: *const c_void,
            buffer: *mut c_void,
            size: usize,
            read: *mut usize,
        ) -> i32;
        #[cfg(test)]
        fn GetSystemDirectoryW(buffer: *mut u16, size: u32) -> u32;
    }
    #[link(name = "advapi32")]
    extern "system" {
        fn ConvertSidToStringSidW(sid: *const c_void, text: *mut *mut u16) -> i32;
        fn OpenSCManagerW(machine: *const u16, database: *const u16, access: u32) -> *mut c_void;
        fn OpenServiceW(manager: *mut c_void, name: *const u16, access: u32) -> *mut c_void;
        fn CloseServiceHandle(service: *mut c_void) -> i32;
    }
    #[link(name = "iphlpapi")]
    extern "system" {
        fn GetExtendedTcpTable(
            table: *mut c_void,
            size: *mut u32,
            order: i32,
            family: u32,
            class: i32,
            reserved: u32,
        ) -> u32;
    }
    #[repr(C)]
    #[derive(Default)]
    struct BasicInfo {
        reserved: usize,
        peb: usize,
        reserved2: [usize; 2],
        pid: usize,
        reserved3: usize,
    }
    #[link(name = "ntdll")]
    extern "system" {
        fn NtQueryInformationProcess(
            process: HANDLE,
            class: u32,
            info: *mut c_void,
            length: u32,
            returned: *mut u32,
        ) -> i32;
    }

    fn token_sid(process: HANDLE) -> AppResult<String> {
        let mut raw = ptr::null_mut();
        if unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut raw) } == 0 {
            return Err(io(
                "open recovery process token",
                std::io::Error::last_os_error(),
            ));
        }
        let token = unsafe { OwnedHandle::from_raw_handle(raw) };
        let mut needed = 0;
        unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                ptr::null_mut(),
                0,
                &mut needed,
            );
        }
        if !(std::mem::size_of::<TOKEN_USER>() as u32..=65536).contains(&needed) {
            return Err(fail("recovery_process_sid", "进程用户令牌大小无效"));
        }
        let mut bytes = vec![0usize; (needed as usize).div_ceil(std::mem::size_of::<usize>())];
        if unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                bytes.as_mut_ptr().cast(),
                needed,
                &mut needed,
            )
        } == 0
        {
            return Err(io(
                "read recovery process SID",
                std::io::Error::last_os_error(),
            ));
        }
        let mut sid = ptr::null_mut();
        if unsafe {
            ConvertSidToStringSidW((*bytes.as_ptr().cast::<TOKEN_USER>()).User.Sid, &mut sid)
        } == 0
        {
            return Err(io(
                "format recovery process SID",
                std::io::Error::last_os_error(),
            ));
        }
        let result = (|| {
            let mut size = 0;
            while size < 256 && unsafe { *sid.add(size) } != 0 {
                size += 1;
            }
            if size == 256 {
                return Err(fail("recovery_process_sid", "进程 SID 字符串超限"));
            }
            String::from_utf16(unsafe { std::slice::from_raw_parts(sid, size) })
                .map_err(|_| fail("recovery_process_sid", "进程 SID 编码无效"))
        })();
        unsafe {
            LocalFree(sid.cast());
        }
        result
    }
    fn read_memory(process: HANDLE, address: usize, size: usize) -> AppResult<Zeroizing<Vec<u8>>> {
        if address < 65536 || size > 1024 * 1024 {
            return Err(fail(
                "recovery_process_parameters",
                "进程参数地址或大小无效",
            ));
        }
        let mut bytes = Zeroizing::new(vec![0u8; size]);
        let mut read = 0;
        if unsafe {
            ReadProcessMemory(
                process,
                address as *const c_void,
                bytes.as_mut_ptr().cast(),
                size,
                &mut read,
            )
        } == 0
            || read != size
        {
            return Err(io(
                "read recovery process parameters",
                std::io::Error::last_os_error(),
            ));
        }
        Ok(bytes)
    }
    fn usize_at(bytes: &[u8], offset: usize) -> usize {
        usize::from_ne_bytes(bytes[offset..offset + 8].try_into().unwrap())
    }
    fn wide(bytes: &[u8]) -> AppResult<String> {
        if !bytes.len().is_multiple_of(2) {
            return Err(fail(
                "recovery_process_parameters",
                "进程 UTF-16 framing 无效",
            ));
        }
        String::from_utf16(
            &bytes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|c| u16::from_ne_bytes([c[0], c[1]]))
                .collect::<Vec<_>>(),
        )
        .map_err(|_| fail("recovery_process_parameters", "进程参数 UTF-16 编码无效"))
    }
    fn parameters(process: HANDLE) -> AppResult<(Vec<String>, Option<PathBuf>)> {
        // The released Windows runtime is x64. Read the documented PEB process
        // parameters through a retained process handle, never a re-opened PID.
        let mut basic = BasicInfo::default();
        if unsafe {
            NtQueryInformationProcess(
                process,
                0,
                (&mut basic as *mut BasicInfo).cast(),
                std::mem::size_of::<BasicInfo>() as u32,
                ptr::null_mut(),
            )
        } < 0
        {
            return Err(fail("recovery_process_parameters", "无法查询进程 PEB"));
        }
        let peb = read_memory(process, basic.peb, 0x28)?;
        let params = read_memory(process, usize_at(&peb, 0x20), 0x88)?;
        let length = u16::from_ne_bytes(params[0x70..0x72].try_into().unwrap()) as usize;
        let maximum = u16::from_ne_bytes(params[0x72..0x74].try_into().unwrap()) as usize;
        if length == 0 || length > maximum || length > 32766 {
            return Err(fail("recovery_process_parameters", "进程命令行长度无效"));
        }
        let command = Zeroizing::new(wide(&read_memory(
            process,
            usize_at(&params, 0x78),
            length,
        )?)?);
        let argv = split_windows_command(&command)?;
        let environment = usize_at(&params, 0x80);
        let mut buffer = Zeroizing::new(Vec::new());
        let mut offset = 0;
        let mut ended = false;
        while offset < 1024 * 1024 {
            // Never cross a page boundary, so a small environment does not need
            // read access to a following unrelated memory region.
            let size = (4096 - (environment + offset) % 4096).min(1024 * 1024 - offset);
            buffer.extend_from_slice(&read_memory(process, environment + offset, size)?);
            if let Some(end) = buffer
                .windows(4)
                .enumerate()
                .find_map(|(n, w)| (n.is_multiple_of(2) && w == [0, 0, 0, 0]).then_some(n))
            {
                buffer.truncate(end + 4);
                ended = true;
                break;
            }
            offset += size;
        }
        if !ended {
            return Err(fail(
                "recovery_process_environment",
                "进程环境超限或缺少结束边界",
            ));
        }
        let env = Zeroizing::new(wide(&buffer)?);
        let mut home = None;
        for item in env.split('\0').take_while(|s| !s.is_empty()) {
            if item
                .get(..15)
                .is_some_and(|s| s.eq_ignore_ascii_case("OPENCODEX_HOME="))
            {
                let value = &item[15..];
                if home.is_some() || !Path::new(value).is_absolute() {
                    return Err(fail("recovery_process_home", "进程 H 不明确"));
                }
                home = Some(PathBuf::from(value));
            }
        }
        Ok((argv, home))
    }
    pub(super) fn split_windows_command(command: &str) -> AppResult<Vec<String>> {
        // Use the OS parser rather than hand-rolling Windows quote/backslash rules.
        use windows_sys::Win32::UI::Shell::CommandLineToArgvW;
        let text: Vec<u16> = command.encode_utf16().chain(Some(0)).collect();
        let mut count = 0;
        let argv = unsafe { CommandLineToArgvW(text.as_ptr(), &mut count) };
        if argv.is_null() || !(1..=16384).contains(&count) {
            return Err(fail(
                "recovery_process_arguments",
                "Windows 命令行 framing 无效",
            ));
        }
        let result = (|| {
            let mut result = Vec::new();
            for i in 0..count {
                let s = unsafe { *argv.add(i as usize) };
                let mut n = 0;
                while n < 32768 && unsafe { *s.add(n) } != 0 {
                    n += 1;
                }
                if n == 32768 {
                    return Err(fail("recovery_process_arguments", "Windows 参数超限"));
                }
                result.push(
                    String::from_utf16(unsafe { std::slice::from_raw_parts(s, n) })
                        .map_err(|_| fail("recovery_process_arguments", "Windows 参数编码无效"))?,
                );
            }
            Ok(result)
        })();
        unsafe {
            LocalFree(argv.cast());
        }
        result
    }
    fn runtime_image(name: &str) -> bool {
        ["node.exe", "bun.exe", "ocx.exe", "opencodex.exe"].contains(&name.to_lowercase().as_str())
    }
    // The isolated native owner test needs a trusted system directory for its
    // synthetic Node child even though production Task Scheduler no longer
    // launches a shell. Keep this test-only helper and its OS lookup intact.
    #[cfg(test)]
    pub(super) fn system_directory() -> AppResult<PathBuf> {
        let mut buffer = vec![0u16; 32768];
        let n = unsafe { GetSystemDirectoryW(buffer.as_mut_ptr(), buffer.len() as u32) };
        if n == 0 || n as usize >= buffer.len() {
            return Err(io(
                "resolve recovery system directory",
                std::io::Error::last_os_error(),
            ));
        }
        Ok(PathBuf::from(std::ffi::OsString::from_wide(
            &buffer[..n as usize],
        )))
    }
    impl Host for PlatformHost {
        fn owner(&self) -> AppResult<String> {
            crate::infrastructure::windows_startup_root::user_scope::current_user_sid()
                .map_err(|e| io("query recovery current SID", e))
        }
        fn process(&self, pid: u32) -> AppResult<Option<ProcessIdentity>> {
            let raw = unsafe {
                OpenProcess(
                    PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                    0,
                    pid,
                )
            };
            if raw.is_null() {
                if unsafe { GetLastError() } == ERROR_INVALID_PARAMETER {
                    return Ok(None);
                }
                return Err(io(
                    "open recovery process identity",
                    std::io::Error::last_os_error(),
                ));
            }
            let process = unsafe { OwnedHandle::from_raw_handle(raw) };
            match unsafe { WaitForSingleObject(raw, 0) } {
                WAIT_OBJECT_0 => return Ok(None),
                WAIT_TIMEOUT => {}
                _ => {
                    return Err(io(
                        "observe recovery process liveness",
                        std::io::Error::last_os_error(),
                    ))
                }
            }
            let owner = token_sid(raw)?;
            let (mut created, mut exit, mut kernel, mut user) = (
                FILETIME::default(),
                FILETIME::default(),
                FILETIME::default(),
                FILETIME::default(),
            );
            if unsafe { GetProcessTimes(raw, &mut created, &mut exit, &mut kernel, &mut user) } == 0
            {
                return Err(io(
                    "query recovery process creation time",
                    std::io::Error::last_os_error(),
                ));
            }
            let mut buffer = vec![0u16; 32768];
            let mut n = buffer.len() as u32;
            if unsafe { QueryFullProcessImageNameW(raw, 0, buffer.as_mut_ptr(), &mut n) } == 0 {
                return Err(io(
                    "query recovery process image",
                    std::io::Error::last_os_error(),
                ));
            }
            let image = PathBuf::from(std::ffi::OsString::from_wide(&buffer[..n as usize]));
            // Foreign SID or non-runtime images never need their environment read.
            let (argv, home) = if owner == self.owner()?
                && runtime_image(image.file_name().and_then(|s| s.to_str()).unwrap_or(""))
            {
                let memory = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | 0x0010, 0, pid) };
                if memory.is_null() {
                    return Err(io(
                        "open recovery process environment",
                        std::io::Error::last_os_error(),
                    ));
                }
                let memory = unsafe { OwnedHandle::from_raw_handle(memory) };
                parameters(memory.as_raw_handle())?
            } else {
                (vec![], None)
            };
            match unsafe { WaitForSingleObject(process.as_raw_handle(), 0) } {
                WAIT_OBJECT_0 => return Ok(None),
                WAIT_TIMEOUT => {}
                _ => {
                    return Err(io(
                        "recheck recovery process liveness",
                        std::io::Error::last_os_error(),
                    ))
                }
            }
            Ok(Some(ProcessIdentity {
                pid,
                owner,
                created: format!(
                    "{}",
                    ((created.dwHighDateTime as u64) << 32) | created.dwLowDateTime as u64
                ),
                image,
                argv,
                home,
                zombie: false,
            }))
        }
        fn processes(&self) -> AppResult<Vec<ProcessIdentity>> {
            let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
            if raw == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
                return Err(io(
                    "snapshot recovery processes",
                    std::io::Error::last_os_error(),
                ));
            }
            let snapshot = unsafe { OwnedHandle::from_raw_handle(raw) };
            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };
            let mut ok = unsafe { Process32FirstW(snapshot.as_raw_handle(), &mut entry) };
            let mut result = Vec::new();
            while ok != 0 {
                let n = entry
                    .szExeFile
                    .iter()
                    .position(|c| *c == 0)
                    .unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..n]);
                if runtime_image(&name) {
                    if let Some(process) = self.process(entry.th32ProcessID)? {
                        if process.owner == self.owner()? {
                            result.push(process);
                        }
                    }
                }
                ok = unsafe { Process32NextW(snapshot.as_raw_handle(), &mut entry) };
            }
            if unsafe { GetLastError() } != ERROR_NO_MORE_FILES {
                return Err(io(
                    "enumerate recovery processes",
                    std::io::Error::last_os_error(),
                ));
            }
            Ok(result)
        }
        fn port_clear(&self, port: u16) -> AppResult<bool> {
            for (family, stride, offset) in [(2, 24usize, 8usize), (23, 56, 20)] {
                let mut size = 0;
                let first =
                    unsafe { GetExtendedTcpTable(ptr::null_mut(), &mut size, 0, family, 3, 0) };
                if first != 122 || !(4..=8 * 1024 * 1024).contains(&size) {
                    return Err(fail("recovery_listener_query", "无法取得系统 TCP 表大小"));
                }
                let mut bytes = vec![0u32; (size as usize).div_ceil(4)];
                let code = unsafe {
                    GetExtendedTcpTable(bytes.as_mut_ptr().cast(), &mut size, 0, family, 3, 0)
                };
                if code != 0 {
                    return Err(fail(
                        "recovery_listener_query",
                        "系统 TCP 表在读取期间变化或不可读",
                    ));
                }
                let raw = unsafe {
                    std::slice::from_raw_parts(bytes.as_ptr().cast::<u8>(), size as usize)
                };
                let count = u32::from_ne_bytes(raw[..4].try_into().unwrap()) as usize;
                if count
                    .checked_mul(stride)
                    .and_then(|s| s.checked_add(4))
                    .is_none_or(|s| s > raw.len())
                {
                    return Err(fail("recovery_listener_query", "系统 TCP 表被截断"));
                }
                for row in raw[4..].chunks_exact(stride).take(count) {
                    let value = u32::from_ne_bytes(row[offset..offset + 4].try_into().unwrap());
                    if u16::from_be(value as u16) == port {
                        return Ok(false);
                    }
                }
            }
            Ok(true)
        }
        fn manager_absent(&self) -> AppResult<bool> {
            // Query native service by handle: only ERROR_SERVICE_DOES_NOT_EXIST
            // establishes absence. Access denial is never treated as missing.
            let manager = unsafe { OpenSCManagerW(ptr::null(), ptr::null(), 1) };
            if manager.is_null() {
                return Err(io(
                    "query recovery service manager",
                    std::io::Error::last_os_error(),
                ));
            }
            let name: Vec<u16> = "opencodex-proxy-native"
                .encode_utf16()
                .chain(Some(0))
                .collect();
            let service = unsafe { OpenServiceW(manager, name.as_ptr(), 4) };
            let code = unsafe { GetLastError() };
            unsafe {
                if !service.is_null() {
                    CloseServiceHandle(service);
                }
                CloseServiceHandle(manager);
            }
            if !service.is_null() {
                return Ok(false);
            }
            if code != 1060 {
                return Err(fail(
                    "recovery_service_domain_unknown",
                    "无法确认原生服务缺席",
                ));
            }
            crate::infrastructure::windows_task_scheduler::opencodex_task_absent().map_err(
                |error| {
                    fail(
                        if error
                            == crate::infrastructure::windows_task_scheduler::QueryError::Timeout
                        {
                            "recovery_os_query_timeout"
                        } else {
                            "recovery_service_domain_unknown"
                        },
                        "无法确认计划任务服务缺席；原生只读查询未提供有效结论",
                    )
                },
            )
        }
    }
}

#[cfg(target_os = "macos")]
mod native {
    use super::*;
    fn bsd(pid: u32) -> AppResult<Option<libc::proc_bsdinfo>> {
        let mut info: libc::proc_bsdinfo = unsafe { std::mem::zeroed() };
        let size = std::mem::size_of_val(&info) as i32;
        let n = unsafe {
            libc::proc_pidinfo(
                pid as i32,
                libc::PROC_PIDTBSDINFO,
                0,
                (&mut info as *mut libc::proc_bsdinfo).cast(),
                size,
            )
        };
        if n == size && info.pbi_pid == pid {
            return Ok(Some(info));
        }
        let e = std::io::Error::last_os_error();
        if n == 0 && e.raw_os_error() == Some(libc::ESRCH) {
            Ok(None)
        } else {
            Err(io("query recovery process identity", e))
        }
    }
    fn image(pid: u32) -> AppResult<PathBuf> {
        use std::os::unix::ffi::OsStringExt;
        let mut buffer = vec![0u8; libc::PROC_PIDPATHINFO_MAXSIZE as usize];
        let n = unsafe {
            libc::proc_pidpath(pid as i32, buffer.as_mut_ptr().cast(), buffer.len() as u32)
        };
        if n <= 0 {
            return Err(io(
                "read recovery process image",
                std::io::Error::last_os_error(),
            ));
        }
        buffer.truncate(buffer.iter().position(|b| *b == 0).unwrap_or(n as usize));
        Ok(PathBuf::from(std::ffi::OsString::from_vec(buffer)))
    }
    fn raw_procargs(pid: u32) -> AppResult<Zeroizing<Vec<u8>>> {
        let mut mib = [libc::CTL_KERN, libc::KERN_PROCARGS2, pid as i32];
        let mut bytes = Zeroizing::new(vec![0u8; 1024 * 1024]);
        let mut size = bytes.len();
        if unsafe {
            libc::sysctl(
                mib.as_mut_ptr(),
                3,
                bytes.as_mut_ptr().cast(),
                &mut size,
                std::ptr::null_mut(),
                0,
            )
        } != 0
        {
            return Err(io(
                "read recovery process arguments",
                std::io::Error::last_os_error(),
            ));
        }
        if size > bytes.len() {
            return Err(fail("removal_process_framing", "内核进程路径记录超过上限"));
        }
        bytes.truncate(size);
        Ok(bytes)
    }
    fn arguments(pid: u32) -> AppResult<(Vec<String>, Option<PathBuf>)> {
        parse_procargs(&raw_procargs(pid)?)
    }
    fn same_generation(before: &libc::proc_bsdinfo, after: &libc::proc_bsdinfo) -> bool {
        (
            before.pbi_pid,
            before.pbi_uid,
            before.pbi_start_tvsec,
            before.pbi_start_tvusec,
        ) == (
            after.pbi_pid,
            after.pbi_uid,
            after.pbi_start_tvsec,
            after.pbi_start_tvusec,
        )
    }
    fn exiting() -> AppError {
        fail(
            "recovery_process_exiting",
            "原进程代次正在退出，尚未取得最终缺席与正常退出证明",
        )
    }
    fn process_snapshot(
        pid: u32,
        observe: impl Fn() -> AppResult<Option<libc::proc_bsdinfo>>,
        read: impl FnOnce() -> AppResult<(PathBuf, Vec<String>, Option<PathBuf>)>,
    ) -> AppResult<Option<ProcessIdentity>> {
        let Some(before) = observe()? else {
            return Ok(None);
        };
        if before.pbi_pid != pid {
            return Err(fail("recovery_process_reused", "进程PID身份不符"));
        }
        if before.pbi_status == 5 {
            return Err(exiting());
        }
        let fields = read();
        // Reobserve even when image/argv fails. EINVAL/ESRCH alone is never
        // absence; only the native observer's explicit ESRCH may return None.
        let Some(after) = observe()? else {
            return Ok(None);
        };
        if !same_generation(&before, &after) {
            return Err(fail("recovery_process_reused", "进程 PID 已复用"));
        }
        if after.pbi_status == 5 {
            return Err(exiting());
        }
        let (image, argv, home) = fields?;
        Ok(Some(ProcessIdentity {
            pid,
            owner: before.pbi_uid.to_string(),
            created: format!("{}:{}", before.pbi_start_tvsec, before.pbi_start_tvusec),
            image,
            argv,
            home,
            zombie: false,
        }))
    }
    // KERN_PROCARGS2 includes the original executable path before argv/environment.
    // Read only that header for classification when the vnode path is gone; the
    // bounded backing buffer is zeroized and never logged or treated as a key.
    pub(super) fn kernel_exec_path(pid: u32) -> AppResult<PathBuf> {
        exec_path_from_procargs(&raw_procargs(pid)?)
    }
    pub(super) fn exec_path_from_procargs(bytes: &[u8]) -> AppResult<PathBuf> {
        use std::os::unix::ffi::OsStringExt;
        if bytes.len() < 6 {
            return Err(fail("removal_process_framing", "内核执行路径记录无效"));
        }
        let argc = i32::from_ne_bytes(bytes[..4].try_into().unwrap());
        if !(1..=16384).contains(&argc) {
            return Err(fail("removal_process_framing", "内核执行路径参数个数无效"));
        }
        let end = bytes[4..]
            .iter()
            .position(|b| *b == 0)
            .filter(|n| *n > 0 && *n <= 4096)
            .ok_or_else(|| fail("removal_process_framing", "内核执行路径边界无效"))?;
        let path = PathBuf::from(std::ffi::OsString::from_vec(bytes[4..4 + end].to_vec()));
        if !crate::modules::runtime::program_layout::clean_absolute(&path) {
            return Err(fail(
                "removal_process_framing",
                "内核执行路径不是明确的绝对路径",
            ));
        }
        Ok(path)
    }
    pub(super) fn parse_procargs(bytes: &[u8]) -> AppResult<(Vec<String>, Option<PathBuf>)> {
        if bytes.len() < 5 {
            return Err(fail("recovery_process_arguments", "进程参数 framing 无效"));
        }
        let argc = i32::from_ne_bytes(bytes[..4].try_into().unwrap());
        if !(1..=16384).contains(&argc) {
            return Err(fail("recovery_process_arguments", "进程参数数目无效"));
        }
        let mut pos = 4;
        while pos < bytes.len() && bytes[pos] != 0 {
            pos += 1;
        }
        while pos < bytes.len() && bytes[pos] == 0 {
            pos += 1;
        }
        let mut argv = Vec::new();
        for _ in 0..argc {
            if pos >= bytes.len() {
                return Err(fail("recovery_process_arguments", "进程参数数目与长度不符"));
            }
            let end = bytes[pos..]
                .iter()
                .position(|b| *b == 0)
                .map(|n| pos + n)
                .ok_or_else(|| fail("recovery_process_arguments", "进程参数被截断"))?;
            argv.push(
                std::str::from_utf8(&bytes[pos..end])
                    .map_err(|_| fail("recovery_process_arguments", "进程参数编码不明确"))?
                    .to_string(),
            );
            pos = end + 1;
        }
        let mut home = None;
        for entry in bytes[pos..].split(|b| *b == 0).filter(|s| !s.is_empty()) {
            if let Some(value) = entry.strip_prefix(b"OPENCODEX_HOME=") {
                let value = std::str::from_utf8(value)
                    .map_err(|_| fail("recovery_process_home", "进程 H 编码无效"))?;
                if home.is_some() || !Path::new(value).is_absolute() {
                    return Err(fail("recovery_process_home", "进程 H 不明确"));
                }
                home = Some(PathBuf::from(value));
            }
        }
        Ok((argv, home))
    }
    impl Host for PlatformHost {
        fn owner(&self) -> AppResult<String> {
            Ok(unsafe { libc::geteuid() }.to_string())
        }
        fn process(&self, pid: u32) -> AppResult<Option<ProcessIdentity>> {
            process_snapshot(
                pid,
                || bsd(pid),
                || {
                    let image = image(pid)?;
                    let (argv, home) = arguments(pid)?;
                    Ok((image, argv, home))
                },
            )
        }
        fn processes(&self) -> AppResult<Vec<ProcessIdentity>> {
            // PROC_UID_ONLY is the libproc ABI value 4. The kernel filters uid,
            // so inaccessible unrelated system processes cannot create false absence.
            let mut pids = vec![0i32; 65536];
            let n = unsafe {
                libc::proc_listpids(
                    4,
                    libc::geteuid(),
                    pids.as_mut_ptr().cast(),
                    (pids.len() * 4) as i32,
                )
            };
            if n <= 0 || n as usize >= pids.len() * 4 {
                return Err(fail(
                    "recovery_process_list",
                    "当前用户进程列表不可完整读取",
                ));
            }
            let mut result = Vec::new();
            for pid in &pids[..n as usize / 4] {
                if *pid <= 0 {
                    continue;
                }
                let Some(info) = bsd(*pid as u32)? else {
                    continue;
                };
                let name = info
                    .pbi_name
                    .iter()
                    .map(|c| *c as u8)
                    .take_while(|c| *c != 0)
                    .collect::<Vec<_>>();
                let name = String::from_utf8_lossy(&name).to_lowercase();
                if !["node", "bun", "bun.exe", "ocx", "opencodex"].contains(&name.as_str()) {
                    continue;
                }
                if let Some(process) = self.process(*pid as u32)? {
                    result.push(process);
                }
            }
            Ok(result)
        }
        fn program_processes(&self, root: &Path) -> AppResult<Vec<ProcessIdentity>> {
            let mut pids = vec![0i32; 65536];
            // The kernel restricts this enumeration to the current UID. Image
            // inspection comes before name filtering, so a renamed process in
            // this runtime directory cannot disappear from the removal proof.
            let count = unsafe {
                libc::proc_listpids(
                    4,
                    libc::geteuid(),
                    pids.as_mut_ptr().cast(),
                    (pids.len() * 4) as i32,
                )
            };
            if count <= 0 || count as usize >= pids.len() * 4 {
                return Err(fail(
                    "removal_process_list",
                    "当前用户程序占用列表无法完整读取",
                ));
            }
            let mut result = vec![];
            for pid in &pids[..count as usize / 4] {
                if *pid <= 0 {
                    continue;
                }
                let Some(info) = bsd(*pid as u32)? else {
                    continue;
                };
                if info.pbi_status == 5 {
                    continue;
                }
                let mut observed = None;
                for attempt in 0..3 {
                    match image(*pid as u32) {
                        Ok(path) => {
                            observed = Some(path);
                            break;
                        }
                        Err(error) => {
                            let Some(after) = bsd(*pid as u32)? else {
                                break;
                            };
                            if after.pbi_status == 5 {
                                break;
                            }
                            if (after.pbi_uid, after.pbi_start_tvsec, after.pbi_start_tvusec)
                                != (info.pbi_uid, info.pbi_start_tvsec, info.pbi_start_tvusec)
                            {
                                return Err(fail(
                                    "removal_process_reused",
                                    "核对程序占用期间PID代次发生变化",
                                ));
                            }
                            if let Ok(path) = kernel_exec_path(*pid as u32) {
                                observed = Some(path);
                                break;
                            }
                            if attempt == 2 {
                                return Err(error);
                            }
                            std::thread::sleep(Duration::from_millis(5));
                        }
                    }
                }
                let Some(executable) = observed else {
                    continue;
                };
                let Some(after) = bsd(*pid as u32)? else {
                    continue;
                };
                if after.pbi_status == 5 {
                    continue;
                }
                if (after.pbi_uid, after.pbi_start_tvsec, after.pbi_start_tvusec)
                    != (info.pbi_uid, info.pbi_start_tvsec, info.pbi_start_tvusec)
                {
                    return Err(fail(
                        "removal_process_reused",
                        "读取程序路径期间PID代次发生变化",
                    ));
                }
                let relevant = path_key(&executable).starts_with(root)
                    || matches!(
                        executable.file_name().and_then(|n| n.to_str()),
                        Some("node" | "bun" | "bun.exe" | "ocx" | "opencodex")
                    );
                if relevant {
                    match self.process(*pid as u32) {
                        Ok(Some(process)) => {
                            if process.owner != info.pbi_uid.to_string()
                                || process.created
                                    != format!("{}:{}", info.pbi_start_tvsec, info.pbi_start_tvusec)
                            {
                                return Err(fail(
                                    "removal_process_reused",
                                    "读取程序参数期间PID代次发生变化",
                                ));
                            }
                            result.push(process)
                        }
                        Ok(None) => (),
                        Err(error) => {
                            if bsd(*pid as u32)?.is_some_and(|info| info.pbi_status != 5) {
                                return Err(error);
                            }
                        }
                    }
                }
            }
            Ok(result)
        }
        fn port_clear(&self, port: u16) -> AppResult<bool> {
            // Darwin's exported pcblist64 ABI uses #pragma pack(4). Verified
            // against SDK xtcpcb64/xinpgen: row=472, local port=4+18, state=292.
            // Unknown framing or a changed snapshot is not absence. Unlike
            // netstat, this also works in non-terminal app subprocess contexts.
            for _ in 0..3 {
                let mut size = 0usize;
                let name = c"net.inet.tcp.pcblist64";
                if unsafe {
                    libc::sysctlbyname(
                        name.as_ptr(),
                        std::ptr::null_mut(),
                        &mut size,
                        std::ptr::null_mut(),
                        0,
                    )
                } != 0
                {
                    return Err(io(
                        "size recovery TCP listener table",
                        std::io::Error::last_os_error(),
                    ));
                }
                if !(48..=16 * 1024 * 1024).contains(&size) {
                    return Err(fail("recovery_listener_query", "系统 TCP 表大小无效"));
                }
                let mut bytes = vec![0u8; size];
                if unsafe {
                    libc::sysctlbyname(
                        name.as_ptr(),
                        bytes.as_mut_ptr().cast(),
                        &mut size,
                        std::ptr::null_mut(),
                        0,
                    )
                } != 0
                {
                    if std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOMEM) {
                        continue;
                    }
                    return Err(io(
                        "read recovery TCP listener table",
                        std::io::Error::last_os_error(),
                    ));
                }
                bytes.truncate(size);
                if size < 48
                    || bytes[..4] != 24u32.to_ne_bytes()
                    || bytes[size - 24..size - 20] != 24u32.to_ne_bytes()
                {
                    return Err(fail(
                        "recovery_listener_query",
                        "系统 TCP 表头尾 framing 无效",
                    ));
                }
                if bytes[..24] != bytes[size - 24..] {
                    continue;
                }
                let rows = &bytes[24..size - 24];
                if !rows.len().is_multiple_of(472) {
                    return Err(fail("recovery_listener_query", "系统 TCP 表 ABI 不受支持"));
                }
                for row in rows.as_chunks::<472>().0 {
                    if row[..4] != 472u32.to_ne_bytes() {
                        return Err(fail(
                            "recovery_listener_query",
                            "系统 TCP 表条目 framing 无效",
                        ));
                    }
                    if u32::from_ne_bytes(row[292..296].try_into().unwrap()) == 1
                        && u16::from_be_bytes(row[22..24].try_into().unwrap()) == port
                    {
                        return Ok(false);
                    }
                }
                return Ok(true);
            }
            Err(fail(
                "recovery_listener_query_changed",
                "系统 TCP 表读取期间变化，未确认监听缺席",
            ))
        }
        fn manager_absent(&self) -> AppResult<bool> {
            let uid = unsafe { libc::geteuid() };
            for domain in [format!("gui/{uid}"), format!("user/{uid}")] {
                let (code, _) = tauri::async_runtime::block_on(system_query(
                    Path::new("/bin/launchctl"),
                    &["print".into(), format!("{domain}/com.opencodex.proxy")],
                ))?;
                match code {
                    0 => return Ok(false),
                    113 => {}
                    112 => {
                        // A missing gui domain is only benign when the user's own
                        // user domain can still positively answer the namespace.
                        if domain.starts_with("user/") {
                            return Err(fail(
                                "recovery_service_domain_unknown",
                                "用户服务域不可查询",
                            ));
                        }
                    }
                    _ => {
                        return Err(fail(
                            "recovery_service_domain_unknown",
                            "用户服务管理器未给出明确缺席证据",
                        ))
                    }
                }
            }
            Ok(true)
        }
    }
    #[cfg(test)]
    mod snapshot_tests {
        include!("runtime_recovery_macos_snapshot_tests.rs");
    }
}

#[cfg(not(any(target_os = "macos", windows)))]
impl Host for PlatformHost {
    fn owner(&self) -> AppResult<String> {
        Err(fail(
            "recovery_owner_platform",
            "当前平台不支持恢复进程身份查询",
        ))
    }
    fn process(&self, _: u32) -> AppResult<Option<ProcessIdentity>> {
        self.owner().map(|_| None)
    }
    fn processes(&self) -> AppResult<Vec<ProcessIdentity>> {
        self.owner().map(|_| vec![])
    }
    fn port_clear(&self, _: u16) -> AppResult<bool> {
        self.owner().map(|_| false)
    }
    fn manager_absent(&self) -> AppResult<bool> {
        self.owner().map(|_| false)
    }
}

#[cfg(test)]
#[path = "runtime_recovery_v2_owner_tests.rs"]
mod tests;
