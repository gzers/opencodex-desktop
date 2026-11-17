//! Process-local production authority for unchanged official2.99. No adoption,
//! durable recovery journal or manager-restart handle transfer is invented here.
use super::windows_client_originals024::{LivePhase, WindowsOriginals};
use super::windows_official_runtime024::{
    self as native, LaunchSpec, OwnedListeners, OwnedRuntime, ProcessState,
};
use super::{
    windows_diagnostic_roles024::WindowsDiagnostics, windows_supervisor024::WindowsSupervisor,
};
use crate::modules::runtime::windows_lifecycle024_contract::{
    ClientOriginals, DiagnosticRoles, SupervisorAbsence,
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
mod actions;
pub(crate) use actions::{NativeAction, OriginalActionKind};

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
        .replace('/', &char::from(92).to_string())
        .eq_ignore_ascii_case(
            &dunce::simplified(b)
                .to_string_lossy()
                .replace('/', &char::from(92).to_string()),
        )
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
    root: PathBuf,
    home: PathBuf,
    profile: PathBuf,
    owner: Mutex<Option<Owner>>,
    cancel: AtomicBool,
}
/// Data about completion time only, not native close authority. Only Owner's
/// native typed close/readback path records it; a later observation must still
/// borrow the original Closed tag and revalidate every retained native object.
struct StopCompletion {
    completed_at: Instant,
    original_deadline: Instant,
}
impl StopCompletion {
    fn record(deadline: Instant) -> AppResult<Self> {
        budget(deadline)?;
        let completed_at = Instant::now();
        if completed_at >= deadline {
            return Err(AppError::Timeout);
        }
        Ok(Self {
            completed_at,
            original_deadline: deadline,
        })
    }
    fn original_window_valid(&self) -> bool {
        self.completed_at < self.original_deadline
    }
}
/// One native Stop readback in a particular action's stop subwindow. Private,
/// nonserde/nonClone; never reconstructed from the public LifecycleResult.
struct StopReceipt {
    completion: StopCompletion,
}
fn after_stop_input(
    command: &ProcessCommand,
    window: &shared::ActionWindow,
    receipt: StopReceipt,
) -> AppResult<Option<ProcessCommand>> {
    window.remaining()?;
    if !receipt.completion.original_window_valid()
        || receipt.completion.original_deadline != window.stop_deadline()
    {
        return Err(recovery_required());
    }
    match command.action {
        LifecycleAction::Stop => Ok(None),
        LifecycleAction::Restart => {
            let mut start = command.clone();
            start.action = LifecycleAction::Start;
            Ok(Some(start))
        }
        LifecycleAction::Start => Err(recovery_required()),
    }
}
pub(crate) struct Owner {
    start_action: actions::ActionOrigin,
    stop_action: Option<actions::ActionOrigin>,
    runtime: SharedRuntimeExecutable,
    selected: program_selection::SelectedProgram,
    root: PathBuf,
    profile: PathBuf,
    home: PathBuf,
    native: OwnedRuntime,
    service: Option<(u32, u16, String)>,
    // Exact original typed observation, never rebuilt from the service tuple.
    typed_ready: Option<shared::ServiceObservation>,
    original_listeners: Option<OwnedListeners>,
    stopped: bool,
    stop_completion: Option<StopCompletion>,
    stage: &'static str,
    session: uuid::Uuid,
    data: PathBuf,
    scope_pins: Vec<native::PinnedDirectory>,
    created_at: Instant,
    stop_started: Option<Instant>,
    originals: Arc<Mutex<WindowsOriginals>>,
    original_phase: Option<LivePhase>,
    // Every previous ORIGINAL creator/runtime/source remains retained; this is
    // a flat vector, never reconstructed from a PID or serialized Closed text.
    previous: Vec<Owner>,
    _outputs: [File; 2],
    _pins: Vec<File>,
}
fn slot(runtime: &SharedRuntimeExecutable, home: &Path, create: bool) -> AppResult<Arc<Slot>> {
    let root = runtime
        .managed_installation_root()
        .ok_or_else(|| unknown(()))?;
    let mut registry = OWNERS
        .get_or_init(Default::default)
        .try_lock()
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
    // A refreshed provider/H must not strand a previous process-local owner.
    // Stop/client-recovery continuity is unresolved even after native exit.
    if registry.iter().any(|s| same(&s.root, &root)) {
        return Err(unknown(()));
    }
    let slot = Arc::new(Slot {
        runtime: runtime.clone(),
        root,
        home: home.into(),
        profile: platform::home_dir().ok_or_else(|| unknown(()))?,
        owner: Mutex::new(None),
        cancel: AtomicBool::new(false),
    });
    registry.push(slot.clone());
    Ok(slot)
}
impl Owner {
    pub(crate) fn originals_previous_owners(&mut self) -> &mut [Owner] {
        &mut self.previous
    }
    pub(crate) fn originals_is_stopped(&self) -> bool {
        self.stopped
            && self.stop_started.is_some()
            && self
                .stop_completion
                .as_ref()
                .is_some_and(StopCompletion::original_window_valid)
    }
    pub(crate) fn originals_creator_has_resumed(&self) -> bool {
        self.native.creator_has_resumed()
    }
    fn authorize_originals_successor(&mut self, window: &shared::ActionWindow) -> AppResult<()> {
        self.check_completed_stop_for_action(window)?;
        let Some(LivePhase::Closed(closed)) = self.original_phase.take() else {
            return Err(recovery_required());
        };
        self.with_originals(|originals, owner| originals.authorize_successor(closed, owner, window))
    }
    fn check_closed_native(&mut self, deadline: Instant) -> AppResult<()> {
        let phase = self.original_phase.take().ok_or_else(recovery_required)?;
        let result = match &phase {
            LivePhase::Closed(closed) => self
                .with_originals(|originals, owner| originals.check_closed(closed, owner, deadline)),
            _ => Err(recovery_required()),
        };
        self.original_phase = Some(phase);
        result
    }
    fn check_completed_stop(&mut self, deadline: Instant) -> AppResult<()> {
        budget(deadline)?;
        if !self.originals_is_stopped() {
            return Err(recovery_required());
        }
        self.check_closed_native(deadline)?;
        budget(deadline)
    }
    fn check_closed_native_original(&mut self, deadline: Instant) -> AppResult<()> {
        let phase = self.original_phase.take().ok_or_else(recovery_required)?;
        let result = match &phase {
            LivePhase::Closed(closed) => self.with_originals(|originals, owner| {
                originals.check_closed_original(closed, owner, deadline)
            }),
            _ => Err(recovery_required()),
        };
        self.original_phase = Some(phase);
        result
    }
    fn check_completed_stop_for_action(&mut self, window: &shared::ActionWindow) -> AppResult<()> {
        window.remaining()?;
        if !self.originals_is_stopped() {
            return Err(recovery_required());
        }
        let phase = self.original_phase.take().ok_or_else(recovery_required)?;
        let result = match &phase {
            LivePhase::Closed(closed) => self.with_originals(|originals, owner| {
                originals.check_closed_for_action(closed, owner, window)
            }),
            _ => Err(recovery_required()),
        };
        self.original_phase = Some(phase);
        result?;
        window.remaining()?;
        Ok(())
    }
    fn check_action_scope(
        &self,
        runtime: &SharedRuntimeExecutable,
        command: &ProcessCommand,
        data: &Path,
    ) -> AppResult<()> {
        if !Arc::ptr_eq(&self.runtime, runtime)
            || !same(&self.profile, &command.working_directory)
            || !same(&self.home, &command.environment.opencodex_home)
            || !same(&self.data, data)
            || command.environment.home.as_deref() != Some(self.profile.as_os_str())
            || command.executable != self.selected.stable_entry
        {
            return Err(unknown(()));
        }
        Ok(())
    }
    fn with_originals<T>(
        &mut self,
        operation: impl FnOnce(&mut WindowsOriginals, &mut Owner) -> AppResult<T>,
    ) -> AppResult<T> {
        let retained = self.originals.clone();
        let mut originals = retained.try_lock().map_err(unknown)?;
        operation(&mut originals, self)
    }
    fn capture_originals(&mut self, window: &shared::ActionWindow) -> AppResult<()> {
        if self.original_phase.is_some() {
            return Err(recovery_required());
        }
        let original = self.with_originals(|originals, owner| originals.capture(owner, window))?;
        self.original_phase = Some(LivePhase::Original(original));
        Ok(())
    }
    fn originals_before_resume(&mut self, deadline: Instant) -> AppResult<()> {
        if !matches!(self.original_phase, Some(LivePhase::Original(_))) {
            return Err(recovery_required());
        }
        self.with_originals(|originals, owner| originals.before_resume(owner, deadline))
    }
    pub(crate) fn scope(&self) -> AppResult<String> {
        use serde_json::json;
        let b = self.native.binding();
        let ids: Vec<_> = self.scope_pins.iter().map(|p| json!({"path":p.path(),"volume":p.identity().volume,"id":p.identity().index.to_string()})).collect();
        let bytes = serde_json::to_vec(&json!({"session":self.session.to_string(),"sid":b.process.token.sid,"profile":self.profile,"D":self.data,"H":self.home,"I":self.root,"directories":ids,"source":self.selected.selection.image_sha256,"rootPid":b.process.pid,"rootFiletime":b.process.creation_filetime.to_string()})).map_err(unknown)?;
        Ok(hash::sha256_hex(&bytes))
    }
    pub(crate) fn profile_path(&self) -> &Path {
        &self.profile
    }
    pub(crate) fn installation_path(&self) -> &Path {
        &self.root
    }
    pub(crate) fn home_path(&self) -> &Path {
        &self.home
    }
    pub(crate) fn data_path(&self) -> &Path {
        &self.data
    }
    pub(crate) fn originals_require_unresumed(&mut self, deadline: Instant) -> AppResult<()> {
        self.source(deadline)?;
        if self.stage != "created-suspended"
            || self.service.is_some()
            || self.stopped
            || self.stop_started.is_some()
        {
            return Err(unknown(()));
        }
        self.native
            .require_creator_not_resumed(deadline)
            .map_err(|error| {
                fail(
                    "windows024_originals_creator_unconfirmed",
                    &format!("original creator raw={:?}; {error}", error.raw_os_error()),
                )
            })
    }
    pub(crate) fn originals_check_ready(
        &mut self,
        ready: &shared::ServiceObservation,
        deadline: Instant,
    ) -> AppResult<()> {
        budget(deadline)?;
        if !ready.ready() || ready.version() != "2.99.0" {
            return Err(unknown(()));
        }
        ready
            .check_current(&self.runtime, &self.home)
            .map_err(unknown)?;
        self.live(ready.pid(), ready.port(), ready.record_sha256(), deadline)?;
        ready
            .check_current(&self.runtime, &self.home)
            .map_err(unknown)?;
        budget(deadline)
    }
    pub(crate) fn originals_check_live_record(
        &mut self,
        pid: u32,
        port: u16,
        digest: &str,
        deadline: Instant,
    ) -> AppResult<()> {
        use std::io::Read;
        if self.service.as_ref() != Some(&(pid, port, digest.to_string())) {
            return Err(unknown(()));
        }
        self.live(pid, port, digest, deadline)?;
        let mut record = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .custom_flags(0x200000)
            .open(self.home.join("runtime-port.json"))
            .map_err(unknown)?;
        let mut bytes = Vec::new();
        (&mut record)
            .take(65537)
            .read_to_end(&mut bytes)
            .map_err(unknown)?;
        if bytes.is_empty() || bytes.len() > 65536 || hash::sha256_hex(&bytes) != digest {
            return Err(unknown(()));
        }
        self.source(deadline)
    }
    pub(crate) fn originals_check_stopped_records(&self, deadline: Instant) -> AppResult<()> {
        self.source(deadline)?;
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
        teardown_records_absent(&self.home)?;
        budget(deadline)
    }
    pub(crate) fn selected_program(&self) -> &program_selection::SelectedProgram {
        &self.selected
    }
    pub(crate) fn official_package_root(&self) -> PathBuf {
        self.selected
            .paths
            .package_root
            .join(crate::modules::runtime::install::PACKAGE_SUBPATH)
    }
    pub(crate) fn original_native(&mut self) -> &mut OwnedRuntime {
        &mut self.native
    }
    /// A generation observed in the original pre-stop listener set after the
    /// typed Ready/current checks. A PID supplied by the caller cannot create it.
    pub(crate) fn confirmed_service_generation(
        &self,
    ) -> AppResult<Option<native::ProcessIdentity>> {
        let Some((pid, port, _)) = self.service.as_ref() else {
            return Ok(None);
        };
        let listeners = self
            .original_listeners
            .as_ref()
            .ok_or_else(|| unknown(()))?;
        if !listeners
            .listeners
            .iter()
            .any(|(l, _)| l.pid == *pid && l.port == *port && l.address.is_loopback())
        {
            return Err(unknown(()));
        }
        listeners
            .family
            .active_members
            .iter()
            .find(|p| p.pid == *pid)
            .cloned()
            .map(Some)
            .ok_or_else(|| unknown(()))
    }
    pub(crate) fn command_phase(&self, when: Instant) -> AppResult<&'static str> {
        if when < self.created_at || when > Instant::now() {
            return Err(unknown(()));
        }
        Ok(if self.stop_started.is_some_and(|s| when >= s) {
            "stop"
        } else {
            "start"
        })
    }
    fn source(&self, deadline: Instant) -> AppResult<()> {
        budget(deadline)?;
        if native::current_identity().map_err(unknown)? != self.native.binding().process.token {
            return Err(unknown(()));
        }
        for pin in &self.scope_pins {
            pin.recheck().map_err(unknown)?;
        }
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
        self.observe_in(deadline, None).await
    }
    async fn observe_for_action(
        &mut self,
        window: &shared::ActionWindow,
        stopping: bool,
    ) -> AppResult<serde_json::Value> {
        window.remaining()?;
        let end = if stopping {
            window.stop_deadline()
        } else {
            window.deadline()
        };
        budget(end)?;
        self.observe_in(end, Some((window, stopping))).await
    }
    async fn observe_in(
        &mut self,
        deadline: Instant,
        action: Option<(&shared::ActionWindow, bool)>,
    ) -> AppResult<serde_json::Value> {
        if !matches!(
            self.original_phase,
            Some(LivePhase::Original(_) | LivePhase::Ready(_))
        ) {
            return Err(recovery_required());
        }
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
        match self.original_phase.take() {
            Some(LivePhase::Original(original)) => {
                let ready = self.with_originals(|originals, owner| {
                    originals.record_ready(original, owner, &observed, deadline)
                })?;
                self.original_phase = Some(LivePhase::Ready(ready));
            }
            Some(LivePhase::Ready(ready)) => {
                self.original_phase = Some(LivePhase::Ready(ready));
                self.with_originals(|originals, owner| match action {
                    Some((window, stopping)) => {
                        originals.check_ready_for_action(owner, &observed, window, stopping)
                    }
                    None => originals.check_ready(owner, &observed, deadline),
                })?;
            }
            phase => {
                self.original_phase = phase;
                return Err(recovery_required());
            }
        }
        budget(deadline)?;
        self.stage = "ready-current";
        let payload = serde_json::json!({"status":"running","ready":true,"pid":observed.pid(),"port":observed.port(),"version":observed.version(),"protection":"retained-job","rebootSafe":false});
        self.typed_ready = Some(observed);
        Ok(payload)
    }
    async fn stop(&mut self, window: &shared::ActionWindow) -> AppResult<StopReceipt> {
        window.remaining()?;
        let deadline = window.stop_deadline();
        self.source(deadline)?;
        self.stop_started.get_or_insert_with(Instant::now);
        if self.stopped {
            self.check_completed_stop_for_action(window)?;
            return Ok(StopReceipt {
                completion: StopCompletion::record(deadline)?,
            });
        }
        self.observe_for_action(window, true).await?;
        program_image::verify_until(&self.selected, deadline)?;
        self.source(deadline)?;
        self.original_listeners = Some(self.native.listeners().map_err(unknown)?);
        let Some(LivePhase::Ready(ready)) = self.original_phase.take() else {
            return Err(recovery_required());
        };
        let prepared =
            self.with_originals(|originals, owner| originals.pre_stop(ready, owner, deadline))?;
        self.original_phase = Some(LivePhase::Prepared(prepared));
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
        require_accounted_exit(&final_state)?;
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
        teardown_records_absent(&self.home)?;
        self.source(deadline)?;
        let mut supervisor = WindowsSupervisor;
        let absent = supervisor.observe_absent(self, deadline)?;
        let mut diagnostics = WindowsDiagnostics;
        let normal = diagnostics.complete_family(self, &absent, deadline)?;
        normal.check(self, deadline)?;
        supervisor.recheck_absent(self, &absent, deadline)?;
        require_accounted_exit(&self.native.family_state().map_err(unknown)?)?;
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
        teardown_records_absent(&self.home)?;
        self.source(deadline)?;
        let Some(LivePhase::Prepared(prepared)) = self.original_phase.take() else {
            return Err(recovery_required());
        };
        let closed = self.with_originals(|originals, owner| {
            originals.complete_stop(prepared, owner, normal, absent, &accepted, deadline)
        })?;
        self.original_phase = Some(LivePhase::Closed(closed));
        self.check_closed_native_original(deadline)?;
        let completion = StopCompletion::record(deadline)?;
        let receipt = StopReceipt {
            completion: StopCompletion::record(deadline)?,
        };
        self.stop_completion = Some(completion);
        self.stopped = true;
        self.stage = "closed-current";
        Ok(receipt)
    }
}
/// Admission to role verification only. Child exits retain their raw values;
/// only WindowsDiagnostics with independent native supervisor proof can accept
/// exact registration queries. This check never creates NormalExit or Stopped.
fn require_accounted_exit(family: &native::FamilyState) -> AppResult<()> {
    if !family.complete_exit()
        || family.total_processes as usize != family.retained_members.len() + 1
        || !matches!(family.root, ProcessState::Exited { code: 0 })
    {
        return Err(unknown(()));
    }
    Ok(())
}
impl crate::modules::runtime::windows_lifecycle024_contract::NativeSession for Owner {
    fn check_current(&mut self, deadline: Instant) -> AppResult<()> {
        self.source(deadline)
    }
}
fn teardown_records_absent(home: &Path) -> AppResult<()> {
    // Fixed official desktop-supervision.mjs uses H/ocx.pid. A missing legacy
    // spelling cannot prove the actual runtime PID record has been retired.
    for name in ["runtime-port.json", "ocx.pid"] {
        match fs::symlink_metadata(home.join(name)) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            _ => return Err(unknown(())),
        }
    }
    Ok(())
}
fn recovery_required() -> AppError {
    fail(
        "windows024_client_restore_required",
        "原件、owner、阶段或正常退出证明尚未全部确认；保留原生现场，未宣告完整停止或后继启动",
    )
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
    execute_admitted(runtime, command, data, window, |_, _, _, _| {
        super::windows_client_originals024::require_durable_capture()
    })
    .await
}
// Private factoring only. Public execute always supplies the unchanged refusal.
// Debug qualification below supplies native task admission, never a bool/env
// switch; the original source/lease/custody/typed gates below still all execute.
async fn execute_admitted(
    runtime: &SharedRuntimeExecutable,
    command: &ProcessCommand,
    data: &Path,
    window: shared::ActionWindow,
    admission: impl Fn(
        &SharedRuntimeExecutable,
        &ProcessCommand,
        &Path,
        &shared::ActionWindow,
    ) -> AppResult<()>,
) -> AppResult<LifecycleResult> {
    window.remaining()?;
    let entry = slot(
        runtime,
        &command.environment.opencodex_home,
        command.action == LifecycleAction::Start,
    )?;
    let mut guard = entry.owner.try_lock().map_err(unknown)?;
    entry.cancel.store(false, Ordering::SeqCst);
    let mut restart_input = None;
    if let Some(owner) = guard.as_mut() {
        owner.check_action_scope(runtime, command, data)?;
        match command.action {
            LifecycleAction::Start => {
                if owner.stopped {
                    owner.check_completed_stop_for_action(&window)?;
                } else {
                    return owner
                        .observe_for_action(&window, false)
                        .await
                        .map(|_| LifecycleResult::Started);
                }
            }
            LifecycleAction::Stop | LifecycleAction::Restart => {
                if !owner.stopped {
                    owner.bind_stop_action(command.action, &window)?;
                }
                let receipt = owner.stop(&window).await?;
                restart_input = after_stop_input(command, &window, receipt)?;
                if restart_input.is_none() {
                    return Ok(LifecycleResult::Stopped);
                }
            }
        }
    }
    let command = restart_input.as_ref().unwrap_or(command);
    if command.action != LifecycleAction::Start {
        return Err(unknown(()));
    }
    if entry.cancel.load(Ordering::SeqCst) {
        return Err(fail(
            "windows024_cancelled_retained",
            "动作已取消；保留原Owner与闭合历史，未派生新启动",
        ));
    }
    window.remaining()?;
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
    // Original capture must precede the first official mutation. This read-only
    // gate uses the fixed r4 scope for NEW sessions, never a Full fallback. The
    // still-unfinished durable custody adapter cannot be bypassed on success.
    super::windows_client_originals024::preflight_bounded(plan.user_profile(), window.deadline()).map_err(
        |e| {
            fail(
                "windows024_originals_security_unconfirmed",
                &format!(
                    "原件声明访问安全捕获前拒绝；scope=windows-bounded-access-v1; audit=excluded-unknown; fullSecurityProven=false; {e}"
                ),
            )
        },
    )?;
    admission(runtime, command, data, &window)?;
    // This remains BELOW the unchanged public production gate. Qualification
    // consumes the one original Closed tag only after actual new source/plan
    // admission. Failure retains the old Owner and all old native stores.
    let inherited_originals = if let Some(old) = guard.as_mut() {
        old.authorize_originals_successor(&window)?;
        Some(old.originals.clone())
    } else {
        None
    };
    let scope_pins = [
        data,
        plan.user_profile(),
        plan.opencodex_home(),
        plan.program_root(),
    ]
    .into_iter()
    .map(|p| native::PinnedDirectory::open(p).map_err(unknown))
    .collect::<AppResult<Vec<_>>>()?;
    let start_action = actions::ActionOrigin::admit(OriginalActionKind::Start, &window)?;
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
    let mut previous = Vec::new();
    if let Some(mut old) = guard.take() {
        previous.append(&mut old.previous);
        previous.push(old);
    }
    *guard = Some(Owner {
        start_action,
        stop_action: None,
        runtime: runtime.clone(),
        selected,
        root,
        profile: plan.user_profile().into(),
        home: plan.opencodex_home().into(),
        native,
        service: None,
        typed_ready: None,
        original_listeners: None,
        stopped: false,
        stop_completion: None,
        stage: "created-suspended",
        session: uuid::Uuid::new_v4(),
        data: data.into(),
        scope_pins,
        created_at: Instant::now(),
        stop_started: None,
        originals: inherited_originals
            .unwrap_or_else(|| Arc::new(Mutex::new(WindowsOriginals::default()))),
        original_phase: None,
        previous,
        _outputs: outputs,
        _pins: pins,
    });
    let owner = guard.as_mut().unwrap();
    owner.capture_originals(&window)?;
    plan.check_current(runtime)?;
    if entry.cancel.load(Ordering::SeqCst) {
        return Err(fail(
            "windows024_cancelled_retained",
            "取消后保留原创建句柄与Job，未推断退出",
        ));
    }
    owner.originals_before_resume(window.deadline())?;
    plan.check_current(runtime)?;
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
        if owner.observe_for_action(&window, false).await.is_ok() {
            return Ok(LifecycleResult::Started);
        }
        if owner.original_phase.is_none() {
            return Err(recovery_required());
        }
        tokio::time::sleep(
            Duration::from_millis(20)
                .min(window.deadline().saturating_duration_since(Instant::now())),
        )
        .await;
    }
}
#[cfg(debug_assertions)]
pub(crate) fn qualify_existing(
    qualification: &super::windows_client_originals024::existing_qualification::ExistingQualification,
    runtime: &SharedRuntimeExecutable,
    command: &ProcessCommand,
    data: &Path,
    window: shared::ActionWindow,
) -> AppResult<LifecycleResult> {
    qualification.admit(runtime, command, data, &window)?;
    tauri::async_runtime::block_on(execute_admitted(
        runtime,
        command,
        data,
        window,
        |r, c, d, w| qualification.admit(r, c, d, w),
    ))
}
#[cfg(debug_assertions)]
pub(crate) struct QualificationFailureCleanup {
    pub(crate) report: serde_json::Value,
}
/// Only called by the original debug host after native peer authentication
/// and a locally retained failed Result. Neither a PID nor a JSON receipt can
/// reconstruct this registry entry or its original Job. Custody is never moved.
#[cfg(debug_assertions)]
pub(crate) fn qualification_failure_cleanup(
    runtime: &SharedRuntimeExecutable,
    home: &Path,
    deadline: Instant,
) -> std::io::Result<QualificationFailureCleanup> {
    budget(deadline).map_err(std::io::Error::other)?;
    let entry = slot(runtime, home, false).map_err(std::io::Error::other)?;
    let mut guard = entry
        .owner
        .try_lock()
        .map_err(|_| std::io::Error::other("original Owner busy"))?;
    let owner = guard
        .as_mut()
        .ok_or_else(|| std::io::Error::other("original Owner missing"))?;
    entry.cancel.store(true, Ordering::SeqCst);
    budget(deadline).map_err(std::io::Error::other)?;
    let forced = owner.native.force_owned_cleanup(deadline)?;
    // Forced cleanup never closes an original phase, restores clients, or
    // proves previous owners absent. Keep those originals and their host alive.
    owner.stopped = false;
    owner.stop_completion = None;
    owner.stage = "forced-failure-cleanup";
    Ok(QualificationFailureCleanup {
        report: serde_json::json!({
            "forced":forced.forced, "root":format!("{:?}",forced.final_state.root),
            "active":forced.final_state.active_processes,
            "total":forced.final_state.total_processes,
            "originalPhaseRetained":owner.original_phase.is_some(),
            "previousOwners":owner.previous.len(), "hostMayExit":false,
            "forcedExitCode":242,
            "normalStop":false, "closed":false, "clientsRestored":false,
            "allAccepted":false, "publicStartOpen":false
        }),
    })
}
#[cfg(debug_assertions)]
pub(crate) fn qualification_trace(
    runtime: &SharedRuntimeExecutable,
    home: &Path,
) -> serde_json::Value {
    let Ok(entry) = slot(runtime, home, false) else {
        return serde_json::json!({"retainedSlot":false});
    };
    let Ok(mut guard) = entry.owner.try_lock() else {
        return serde_json::json!({"retainedSlot":true,"ownerLock":"unknown"});
    };
    match guard.as_mut() {
        None => serde_json::json!({"retainedSlot":true,"nativeOwner":false}),
        Some(o) => {
            let family = o.native.family_state();
            let detail = match family {
                Ok(f) => {
                    serde_json::json!({"root":format!("{:?}",f.root),"active":f.active_processes,"total":f.total_processes,"members":f.retained_members.iter().map(|(i,s)|serde_json::json!({"pid":i.pid,"filetime":i.creation_filetime.to_string(),"image":i.executable,"state":format!("{s:?}")})).collect::<Vec<_>>()})
                }
                Err(e) => serde_json::json!({"raw":e.raw_os_error(),"error":e.to_string()}),
            };
            serde_json::json!({"retainedSlot":true,"nativeOwner":true,"stage":o.stage,"originalPhaseRetained":o.original_phase.is_some(),"stopped":o.stopped,"creatorResumed":o.native.creator_has_resumed(),"ownerSession":o.session.to_string(),"ownerScope":o.scope().ok(),"previousOwners":o.previous.len(),"binding":{"pid":o.native.binding().process.pid,"filetime":o.native.binding().process.creation_filetime.to_string(),"image":o.native.binding().process.executable},"family":detail})
        }
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
pub(crate) fn retains_owner(runtime: &SharedRuntimeExecutable, home: &Path) -> bool {
    OWNERS.get().is_some_and(|r| {
        r.lock().map_or(true, |owners| {
            owners
                .iter()
                .any(|s| Arc::ptr_eq(&s.runtime, runtime) && same(&s.home, home))
        })
    })
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
                Some(o) if o.stopped || o.original_phase.is_none() => R::Required,
                Some(_) => R::Unknown,
                None => R::Unknown,
            },
            Err(_) => R::Unknown,
        },
        Err(_) => R::Unknown,
    }
}
/// Explicit original observation deadline for a future/shared status caller.
/// The legacy no-deadline trait remains conservative; do not invent a new5s
/// window here or infer NotRequired from a cached stopped boolean.
pub(crate) fn recovery_state_at(
    runtime: &SharedRuntimeExecutable,
    home: &Path,
    deadline: Instant,
) -> crate::modules::status::ClientRecoveryState {
    use crate::modules::status::ClientRecoveryState as R;
    if budget(deadline).is_err() {
        return R::Unknown;
    }
    match slot(runtime, home, false) {
        Ok(entry) => match entry.owner.try_lock() {
            Ok(mut guard) => match guard.as_mut() {
                Some(owner) if owner.originals_is_stopped() => {
                    if owner.check_completed_stop(deadline).is_ok() {
                        R::NotRequired
                    } else {
                        R::Required
                    }
                }
                Some(owner) if owner.original_phase.is_none() => R::Required,
                _ => R::Unknown,
            },
            Err(_) => R::Unknown,
        },
        Err(_) => R::Unknown,
    }
}
pub(crate) fn unknown_status() -> serde_json::Value {
    serde_json::json!({"status":"unknown","ready":false,"protection":"retained-owner-unconfirmed","rebootSafe":false})
}
/// Binds two observations to the SAME retained native Owner. This is not a
/// Closed proof and cannot be decoded/cloned or used in another sample.
pub(crate) struct StatusReceipt {
    entry: Arc<Slot>,
    session: uuid::Uuid,
    scope: String,
    stopped: bool,
    deadline: Instant,
}
impl std::fmt::Debug for StatusReceipt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StatusReceipt")
            .field("session", &self.session)
            .finish_non_exhaustive()
    }
}
pub(crate) fn recovery_for_sample(
    runtime: &SharedRuntimeExecutable,
    home: &Path,
    deadline: Instant,
    receipt: StatusReceipt,
) -> crate::modules::status::ClientRecoveryState {
    use crate::modules::status::ClientRecoveryState as R;
    if budget(deadline).is_err() || deadline != receipt.deadline {
        return R::Unknown;
    }
    let Ok(entry) = slot(runtime, home, false) else {
        return R::Unknown;
    };
    if !Arc::ptr_eq(&entry, &receipt.entry) {
        return R::Unknown;
    }
    let Ok(mut guard) = entry.owner.try_lock() else {
        return R::Unknown;
    };
    let Some(owner) = guard.as_mut() else {
        return R::Unknown;
    };
    if owner.session != receipt.session
        || owner.stopped != receipt.stopped
        || owner.scope().ok().as_deref() != Some(receipt.scope.as_str())
    {
        return R::Unknown;
    }
    // Same original Closed/complete native check as recovery_state_at, while
    // retaining the SAME lock used to compare this sample's original Owner.
    let result = if owner.originals_is_stopped() {
        if owner.check_completed_stop(deadline).is_ok() {
            R::NotRequired
        } else {
            R::Required
        }
    } else if owner.original_phase.is_none() {
        R::Required
    } else {
        R::Unknown
    };
    if budget(deadline).is_err() {
        R::Unknown
    } else {
        result
    }
}
pub(crate) async fn status(
    runtime: &SharedRuntimeExecutable,
    home: &Path,
    deadline: Instant,
) -> Result<serde_json::Value, CollectError> {
    status_sample(runtime, home, deadline)
        .await
        .map(|(payload, _)| payload)
}
pub(crate) async fn status_sample(
    runtime: &SharedRuntimeExecutable,
    home: &Path,
    deadline: Instant,
) -> Result<(serde_json::Value, StatusReceipt), CollectError> {
    budget(deadline).map_err(|_| CollectError::Timeout)?;
    let entry = slot(runtime, home, false).map_err(|_| CollectError::Unreachable)?;
    let mut guard = entry
        .owner
        .try_lock()
        .map_err(|_| CollectError::Unreachable)?;
    let owner = guard.as_mut().ok_or(CollectError::Unreachable)?;
    let payload = if owner.stopped {
        owner
            .check_completed_stop(deadline)
            .map_err(|_| CollectError::Unreachable)?;
        serde_json::json!({"status":"stopped","ready":false,"protection":"retained-job-client-closed","rebootSafe":false})
    } else {
        owner
            .observe(deadline)
            .await
            .map_err(|_| CollectError::Unreachable)?
    };
    let scope = owner.scope().map_err(|_| CollectError::Unreachable)?;
    budget(deadline).map_err(|_| CollectError::Timeout)?;
    let receipt = StatusReceipt {
        entry: entry.clone(),
        session: owner.session,
        scope,
        stopped: owner.stopped,
        deadline,
    };
    Ok((payload, receipt))
}
#[cfg(test)]
#[path = "windows_official_lifecycle024_tests.rs"]
mod tests;
