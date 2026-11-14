//! Process-local production authority for unchanged official2.99. No adoption,
//! durable recovery journal or manager-restart handle transfer is invented here.
use super::windows_official_runtime024::{
    self as native, LaunchSpec, OwnedListeners, OwnedRuntime, ProcessState,
};
use crate::{
    errors::{AppError, AppResult},
    infrastructure::{hash, platform, runtime_executable::SharedRuntimeExecutable},
    modules::{
        process::{LifecycleAction, LifecycleResult, ProcessCommand},
        runtime::{official_lifecycle024 as shared, program_image, program_selection},
        status::CollectError,
    },
};
use std::{
    fs::{self, File, OpenOptions},
    os::windows::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, OnceLock,
    },
    time::{Duration, Instant},
};

fn fail(code: &str, detail: &str) -> AppError {
    AppError::RuntimeManaged {
        code: code.into(),
        detail: detail.into(),
    }
}
fn unknown<T>(_: T) -> AppError {
    fail(
        "windows024_owner_unknown",
        "原生句柄、Job、来源或原记录未确认；保留原owner，未推断成功",
    )
}
fn budget(deadline: Instant) -> AppResult<()> {
    if Instant::now() >= deadline {
        Err(AppError::Timeout)
    } else {
        Ok(())
    }
}
fn same(a: &Path, b: &Path) -> bool {
    dunce::simplified(a)
        .to_string_lossy()
        .eq_ignore_ascii_case(&dunce::simplified(b).to_string_lossy())
}
fn system_console() -> AppResult<PathBuf> {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetSystemDirectoryW(buffer: *mut u16, size: u32) -> u32;
    }
    let mut buffer = vec![0u16; 32768];
    let n = unsafe { GetSystemDirectoryW(buffer.as_mut_ptr(), buffer.len() as u32) };
    if n == 0 || n as usize >= buffer.len() {
        return Err(unknown(()));
    }
    Ok(
        PathBuf::from(String::from_utf16(&buffer[..n as usize]).map_err(unknown)?)
            .join("conhost.exe"),
    )
}

// Shared by GUI, IPC runner and status. A short registry lock never spans I/O.
// Owners deliberately remain retained even after a failed action or runner Drop.
static OWNERS: OnceLock<Mutex<Vec<Arc<Slot>>>> = OnceLock::new();
struct Slot {
    runtime: SharedRuntimeExecutable,
    home: PathBuf,
    profile: PathBuf,
    owner: Mutex<Option<Owner>>,
    cancel: AtomicBool,
}
struct Owner {
    runtime: SharedRuntimeExecutable,
    selected: program_selection::SelectedProgram,
    root: PathBuf,
    profile: PathBuf,
    home: PathBuf,
    native: OwnedRuntime,
    service: Option<(u32, u16, String)>,
    original_listeners: Option<OwnedListeners>,
    stopped: bool,
    stage: &'static str,
    _outputs: [File; 2],
    _pins: Vec<File>,
}
fn slot(runtime: &SharedRuntimeExecutable, home: &Path, create: bool) -> AppResult<Arc<Slot>> {
    let mut registry = OWNERS
        .get_or_init(Default::default)
        .lock()
        .map_err(unknown)?;
    if let Some(slot) = registry
        .iter()
        .find(|s| Arc::ptr_eq(&s.runtime, runtime) && same(&s.home, home))
    {
        return Ok(slot.clone());
    }
    if !create {
        return Err(unknown(()));
    }
    let slot = Arc::new(Slot {
        runtime: runtime.clone(),
        home: home.into(),
        profile: platform::home_dir().ok_or_else(|| unknown(()))?,
        owner: Mutex::new(None),
        cancel: AtomicBool::new(false),
    });
    registry.push(slot.clone());
    Ok(slot)
}
impl Owner {
    fn source(&self, deadline: Instant) -> AppResult<()> {
        budget(deadline)?;
        if self.runtime.managed_installation_root().as_ref() != Some(&self.root)
            || self.runtime.executable().as_ref() != Some(&self.selected.stable_entry)
        {
            return Err(unknown(()));
        }
        program_selection::verify_current(&self.root, &self.selected)?;
        self.native
            .verify_binding(self.native.binding())
            .map_err(unknown)?;
        budget(deadline)
    }
    fn live(&mut self, pid: u32, port: u16, digest: &str, deadline: Instant) -> AppResult<()> {
        self.stage = "native-source";
        self.source(deadline)?;
        if self.stopped || port == 0 || digest.len() != 64 {
            return Err(unknown(()));
        }
        if self
            .service
            .as_ref()
            .is_some_and(|s| s != &(pid, port, digest.into()))
        {
            return Err(unknown(()));
        }
        self.stage = "service-image";
        let bundled_bun = self
            .selected
            .paths
            .package_root
            .join("node_modules/bun/bin/bun.exe");
        if self.native.verify_member_image(pid, &bundled_bun).is_err() {
            self.native
                .verify_member_image(pid, &self.selected.paths.bun)
                .map_err(unknown)?;
        }
        self.stage = "native-listeners";
        let listeners = self.native.listeners().map_err(unknown)?;
        if !listeners
            .listeners
            .iter()
            .any(|(l, _)| l.pid == pid && l.port == port && l.address.is_loopback())
            || listeners
                .listeners
                .iter()
                .any(|(l, _)| !l.address.is_loopback())
        {
            return Err(unknown(()));
        }
        for member in &listeners.family.active_members {
            self.stage = "job-member-image";
            let console = system_console()?;
            let image = if same(&member.executable, &self.selected.paths.node) {
                &self.selected.paths.node
            } else if same(&member.executable, &self.selected.paths.bun) {
                &self.selected.paths.bun
            } else if same(&member.executable, &bundled_bun) {
                &bundled_bun
            } else if same(&member.executable, &console) {
                &console
            } else {
                return Err(unknown(()));
            };
            self.native
                .verify_member_image(member.pid, image)
                .map_err(unknown)?;
        }
        self.service = Some((pid, port, digest.into()));
        self.source(deadline)
    }
    async fn observe(&mut self, deadline: Instant) -> AppResult<serde_json::Value> {
        self.stage = "observation-source";
        self.source(deadline)?;
        self.stage = "typed-health-ready";
        let observed =
            shared::observe_official024(self.runtime.clone(), self.home.clone(), deadline)
                .await
                .map_err(unknown)?;
        if !observed.ready() {
            return Err(unknown(()));
        }
        self.live(
            observed.pid(),
            observed.port(),
            observed.record_sha256(),
            deadline,
        )?;
        observed
            .check_current(&self.runtime, &self.home)
            .map_err(unknown)?;
        budget(deadline)?;
        self.stage = "ready-current";
        Ok(
            serde_json::json!({"status":"running","ready":true,"pid":observed.pid(),"port":observed.port(),"version":observed.version(),"protection":"retained-job","rebootSafe":false}),
        )
    }
    async fn stop(&mut self, deadline: Instant) -> AppResult<()> {
        self.source(deadline)?;
        if self.stopped {
            return Err(recovery_required());
        }
        self.observe(deadline).await?;
        program_image::verify(&self.selected)?;
        self.source(deadline)?;
        self.original_listeners = Some(self.native.listeners().map_err(unknown)?);
        let runtime = self.runtime.clone();
        let home = self.home.clone();
        let pid = self.service.as_ref().ok_or_else(|| unknown(()))?.0;
        let accepted = shared::request_stop_official024(
            runtime,
            home,
            pid,
            deadline,
            &mut LiveBoundary {
                owner: self,
                deadline,
            },
            |_| {},
        )
        .await
        .map_err(unknown)?;
        if self.service.as_ref()
            != Some(&(
                accepted.pid(),
                accepted.port(),
                accepted.record_sha256().into(),
            ))
        {
            return Err(unknown(()));
        }
        self.stage = "native-normal-exit";
        let final_state = self.native.wait_normal_exit(deadline).map_err(unknown)?;
        if !final_state.complete_exit()
            || final_state.total_processes as usize != final_state.retained_members.len() + 1
            || !matches!(final_state.root, ProcessState::Exited { code: 0 })
            || final_state
                .retained_members
                .iter()
                .any(|(_, s)| !matches!(s, ProcessState::Exited { code: 0 }))
        {
            return Err(unknown(()));
        }
        if !native::listener_exit_readback(
            self.original_listeners
                .as_ref()
                .ok_or_else(|| unknown(()))?,
        )
        .map_err(unknown)?
        .is_empty()
        {
            return Err(unknown(()));
        }
        for name in ["runtime-port.json", "opencodex.pid"] {
            match fs::symlink_metadata(self.home.join(name)) {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                _ => return Err(unknown(())),
            }
        }
        self.source(deadline)?;
        if !self.native.family_state().map_err(unknown)?.complete_exit() {
            return Err(unknown(()));
        }
        self.stopped = true;
        // No Windows durable original capture/restore or supervisor observation
        // authority is supplied by r1. Native success is separately retained;
        // it cannot mint LifecycleResult::Stopped or permit successor Start.
        Err(recovery_required())
    }
}
fn recovery_required() -> AppError {
    fail("windows024_client_restore_required","本次官方服务及保留Job已正常退出；Windows客户端原件持久恢复及supervisor缺席接口尚未确认，未宣告完整停止或继续启动")
}
struct LiveBoundary<'a> {
    owner: &'a mut Owner,
    deadline: Instant,
}
impl shared::RetainedService for LiveBoundary<'_> {
    fn verify_live(&mut self, pid: u32, port: u16, digest: &str) -> Result<(), CollectError> {
        self.owner
            .live(pid, port, digest, self.deadline)
            .map_err(|_| CollectError::Unreachable)
    }
}

pub(crate) fn execute(
    runtime: &SharedRuntimeExecutable,
    command: &ProcessCommand,
    data: &Path,
    window: shared::ActionWindow,
) -> AppResult<LifecycleResult> {
    std::thread::scope(|s| {
        s.spawn(|| tauri::async_runtime::block_on(execute_async(runtime, command, data, window)))
            .join()
            .map_err(unknown)?
    })
}
async fn execute_async(
    runtime: &SharedRuntimeExecutable,
    command: &ProcessCommand,
    data: &Path,
    window: shared::ActionWindow,
) -> AppResult<LifecycleResult> {
    window.remaining()?;
    let entry = slot(
        runtime,
        &command.environment.opencodex_home,
        command.action == LifecycleAction::Start,
    )?;
    let mut guard = entry.owner.try_lock().map_err(unknown)?;
    entry.cancel.store(false, Ordering::SeqCst);
    if let Some(owner) = guard.as_mut() {
        if !same(&owner.profile, &command.working_directory) {
            return Err(unknown(()));
        }
        return match command.action {
            LifecycleAction::Start => {
                if owner.stopped {
                    Err(recovery_required())
                } else {
                    owner
                        .observe(window.deadline())
                        .await
                        .map(|_| LifecycleResult::Started)
                }
            }
            LifecycleAction::Stop | LifecycleAction::Restart => {
                owner.stop(window.stop_deadline()).await?;
                Err(recovery_required())
            }
        };
    }
    if command.action != LifecycleAction::Start {
        return Err(unknown(()));
    }
    // Existing record is uncertainty, never permission to create/adopt by PID.
    if !matches!(fs::symlink_metadata(command.environment.opencodex_home.join("runtime-port.json")),Err(e) if e.kind()==std::io::ErrorKind::NotFound)
    {
        return Err(unknown(()));
    }
    let plan = shared::LaunchPlan::prepare(runtime, command, window.deadline())?;
    if !same(
        plan.user_profile(),
        &platform::home_dir().ok_or_else(|| unknown(()))?,
    ) {
        return Err(unknown(()));
    }
    plan.verify_full(runtime)?;
    let root = plan.program_root().to_path_buf();
    let selected = program_selection::read_complete(
        &root,
        crate::modules::data_root::layout::Platform::Windows,
        "win32-x64",
    )?
    .ok_or_else(|| unknown(()))?;
    let dir = data
        .join("logs")
        .join(format!("windows024-{}", uuid::Uuid::new_v4()));
    platform::create_private_dir_all(&dir, 0o700).map_err(unknown)?;
    let outputs = [
        platform::create_new_private_file(&dir.join("stdout.private.log"), 0o600)
            .map_err(unknown)?,
        platform::create_new_private_file(&dir.join("stderr.private.log"), 0o600)
            .map_err(unknown)?,
    ];
    let pins = [
        &selected.paths.node,
        &selected.paths.bun,
        &selected.script,
        &selected
            .paths
            .package_root
            .join("node_modules/bun/bin/bun.exe"),
    ]
    .iter()
    .map(|p| OpenOptions::new().read(true).share_mode(1).open(p))
    .collect::<std::io::Result<Vec<_>>>()
    .map_err(unknown)?;
    let native = OwnedRuntime::create_suspended(LaunchSpec {
        executable: plan.executable(),
        arguments: plan.arguments(),
        working_directory: plan.user_profile(),
        user_profile: plan.user_profile(),
        opencodex_home: plan.opencodex_home(),
        environment: plan.environment(),
        stdout: &outputs[0],
        stderr: &outputs[1],
    })
    .map_err(unknown)?;
    *guard = Some(Owner {
        runtime: runtime.clone(),
        selected,
        root,
        profile: plan.user_profile().into(),
        home: plan.opencodex_home().into(),
        native,
        service: None,
        original_listeners: None,
        stopped: false,
        stage: "created-suspended",
        _outputs: outputs,
        _pins: pins,
    });
    let owner = guard.as_mut().unwrap();
    plan.check_current(runtime)?;
    if entry.cancel.load(Ordering::SeqCst) {
        return Err(fail(
            "windows024_cancelled_retained",
            "取消后保留原创建句柄与Job，未推断退出",
        ));
    }
    owner.native.resume().map_err(unknown)?;
    loop {
        if Instant::now() >= window.deadline() {
            return Err(fail(
                "lifecycle_timeout",
                &format!("原30秒内未确认就绪；stage={}；owner保留", owner.stage),
            ));
        }
        if entry.cancel.load(Ordering::SeqCst) {
            return Err(fail(
                "windows024_cancelled_retained",
                "取消后保留原owner，未强杀服务",
            ));
        }
        if let ProcessState::Exited { code } = owner.native.root_state().map_err(unknown)? {
            if code != 0 {
                return Err(fail(
                    "windows024_start_exit",
                    &format!("官方launcher非零退出 exit_code={code}；原owner保留"),
                ));
            }
        }
        if owner.observe(window.deadline()).await.is_ok() {
            return Ok(LifecycleResult::Started);
        }
        tokio::time::sleep(
            Duration::from_millis(20)
                .min(window.deadline().saturating_duration_since(Instant::now())),
        )
        .await;
    }
}
pub(crate) fn cancel(runtime: &SharedRuntimeExecutable, home: &Path) {
    if let Ok(entry) = slot(runtime, home, false) {
        entry.cancel.store(true, Ordering::SeqCst)
    }
}
pub(crate) fn cancel_profile(profile: &Path) {
    if let Ok(registry) = OWNERS.get_or_init(Default::default).lock() {
        for entry in registry.iter().filter(|s| same(&s.profile, profile)) {
            entry.cancel.store(true, Ordering::SeqCst);
        }
    }
}
pub(crate) fn native_stopped(runtime: &SharedRuntimeExecutable, home: &Path) -> bool {
    slot(runtime, home, false).ok().is_some_and(|s| {
        s.owner
            .try_lock()
            .ok()
            .is_some_and(|o| o.as_ref().is_some_and(|o| o.stopped))
    })
}
pub(crate) fn recovery_state(
    runtime: &SharedRuntimeExecutable,
    home: &Path,
) -> crate::modules::status::ClientRecoveryState {
    use crate::modules::status::ClientRecoveryState as R;
    match slot(runtime, home, false) {
        Ok(s) => match s.owner.try_lock() {
            Ok(o) => match o.as_ref() {
                Some(o) if o.stopped => R::Required,
                Some(_) => R::Active,
                None => R::Unknown,
            },
            Err(_) => R::Unknown,
        },
        Err(_) => R::Unknown,
    }
}
pub(crate) fn unknown_status() -> serde_json::Value {
    serde_json::json!({"status":"unknown","ready":false,"protection":"retained-owner-unconfirmed","rebootSafe":false})
}
pub(crate) async fn status(
    runtime: &SharedRuntimeExecutable,
    home: &Path,
    deadline: Instant,
) -> Result<serde_json::Value, CollectError> {
    let entry = slot(runtime, home, false).map_err(|_| CollectError::Unreachable)?;
    let mut guard = entry
        .owner
        .try_lock()
        .map_err(|_| CollectError::Unreachable)?;
    let owner = guard.as_mut().ok_or(CollectError::Unreachable)?;
    if owner.stopped {
        owner
            .source(deadline)
            .map_err(|_| CollectError::Unreachable)?;
        if !owner
            .native
            .family_state()
            .map_err(|_| CollectError::Unreachable)?
            .complete_exit()
            || owner
                .original_listeners
                .as_ref()
                .is_none_or(|s| native::listener_exit_readback(s).map_or(true, |r| !r.is_empty()))
            || !matches!(fs::symlink_metadata(home.join("runtime-port.json")),Err(e) if e.kind()==std::io::ErrorKind::NotFound)
        {
            return Err(CollectError::Unreachable);
        }
        return Ok(
            serde_json::json!({"status":"stopped","ready":false,"protection":"retained-job-client-recovery-required","rebootSafe":false}),
        );
    }
    owner
        .observe(deadline)
        .await
        .map_err(|_| CollectError::Unreachable)
}
#[cfg(test)]
#[path = "windows_official_lifecycle024_tests.rs"]
mod tests;
