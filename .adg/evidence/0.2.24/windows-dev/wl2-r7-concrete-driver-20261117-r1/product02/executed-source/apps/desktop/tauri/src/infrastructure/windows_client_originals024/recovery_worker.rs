//! Original owning-thread resource transport, not an AbortRestored factory.
//! Active locks are created only AFTER unique original state reaches this
//! worker; only actual released objects may ever leave it.
use super::{
    coordination::{LeasePreparation, NativeLease, ReleaseProbe, ReleasedLease},
    phase::invalid,
};
use crate::infrastructure::windows_official_lifecycle024::Owner;
use crate::modules::runtime::windows_lifecycle024_contract::BeforeResumeRecoveryWindow;
use std::{
    io,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Instant,
};
#[derive(PartialEq, Eq)]
pub(crate) struct TransferBinding {
    session: String,
    action: String,
    scope: String,
    end: Instant,
}
impl TransferBinding {
    pub(crate) fn original_owner(
        owner: &mut Owner,
        window: &BeforeResumeRecoveryWindow,
    ) -> io::Result<Self> {
        owner
            .check_before_resume_process(window)
            .map_err(io::Error::other)?;
        Ok(Self {
            session: owner.before_resume_owner_session(),
            action: owner
                .before_resume_action_id(window)
                .map_err(io::Error::other)?,
            scope: owner.scope().map_err(io::Error::other)?,
            end: window.deadline(),
        })
    }
}
// Implementations verify their actual original resources. The generic core is
// private; only the Owner wrapper and explicit cfg(test) fixtures may spawn it.
trait Original: Send + 'static {
    fn binding(&mut self, window: &BeforeResumeRecoveryWindow) -> io::Result<TransferBinding>;
    fn revoke(&mut self);
}
impl Original for Owner {
    fn binding(&mut self, window: &BeforeResumeRecoveryWindow) -> io::Result<TransferBinding> {
        TransferBinding::original_owner(self, window)
    }
    fn revoke(&mut self) {
        self.fail_before_resume_process();
    }
}
pub(crate) struct WorkerLocal {
    preparation: LeasePreparation,
    lease: Option<NativeLease>,
    released: Option<ReleasedLease>,
    probe: ReleaseProbe,
    revoked: bool,
    directories:
        Vec<crate::infrastructure::windows_official_runtime024::PrivateDirectoryPreparation>,
    directory_pins: Vec<crate::infrastructure::windows_official_runtime024::PinnedDirectory>,
}
impl WorkerLocal {
    pub(crate) fn canonical_claim_stage(
        &mut self,
        owner: &mut Owner,
        window: &BeforeResumeRecoveryWindow,
    ) -> io::Result<()> {
        let lease = self.lease()?;
        lease.check_owner(owner, window.deadline())?;
        let root = owner.profile_path().join(".codex");
        let mut claim = lease.anchored_claim(&root, window.deadline())?;
        let result = (|| {
            let prefix = owner.with_originals(|originals, owner| {
                originals.prepare_abort_prefix(owner, &mut claim, window)
            })?;
            owner.with_originals(|originals, owner| {
                let directory = owner.take_before_resume_attachment_directory(window)?;
                originals.initialize_abort_attachment(owner, &prefix, &mut claim, directory, window)
            })?;
            let quiescent = owner.with_originals(|originals, owner| {
                originals.quiesce_abort_prefix(owner, prefix, &mut claim, window)
            })?;
            owner.with_originals(|originals, owner| {
                originals.return_quiescent_originals(owner, &quiescent, &mut claim, window)
            })?;
            owner.with_originals(|originals, owner| {
                originals.retain_quiescent_for_release(owner, quiescent, window)
            })?;
            Ok::<_, crate::errors::AppError>(())
        })();
        result.map_err(io::Error::other)?;
        claim.check(&root, window.deadline())
    }
    fn new() -> Self {
        Self {
            preparation: LeasePreparation::default(),
            lease: None,
            released: None,
            probe: ReleaseProbe::default(),
            revoked: false,
            directories: Vec::new(),
            directory_pins: Vec::new(),
        }
    }
    pub(crate) fn acquire(
        &mut self,
        owner: &mut Owner,
        window: &BeforeResumeRecoveryWindow,
    ) -> io::Result<()> {
        if self.revoked || self.lease.is_some() || self.released.is_some() {
            return Err(invalid());
        }
        self.preparation.prepare_existing(owner, window)?;
        self.lease = Some(self.preparation.take_complete()?);
        Ok(())
    }
    pub(crate) fn lease(&mut self) -> io::Result<&mut NativeLease> {
        if self.revoked {
            return Err(invalid());
        }
        self.lease.as_mut().ok_or_else(invalid)
    }
    pub(crate) fn attachment_directory(
        &mut self,
        owner: &mut Owner,
        window: &BeforeResumeRecoveryWindow,
    ) -> io::Result<crate::infrastructure::windows_official_runtime024::PinnedDirectory> {
        use crate::infrastructure::windows_official_runtime024::{
            PinnedDirectory, PrivateDirectoryPreparation,
        };
        if self.revoked || !self.directories.is_empty() || !self.directory_pins.is_empty() {
            return Err(invalid());
        }
        owner
            .check_before_resume_process(window)
            .map_err(io::Error::other)?;
        let action = owner
            .before_resume_action_id(window)
            .map_err(io::Error::other)?;
        self.directory_pins
            .push(PinnedDirectory::open_private(owner.data_path())?);
        let parent = owner.data_path().join("before-resume-recovery");
        match std::fs::symlink_metadata(&parent) {
            Ok(_) => self
                .directory_pins
                .push(PinnedDirectory::open_private(&parent)?),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                self.directories
                    .push(PrivateDirectoryPreparation::default());
                let original = self.directories.last_mut().unwrap();
                original.create(&parent, window.deadline())?;
                self.directory_pins.push(original.take_complete()?);
            }
            Err(e) => return Err(e),
        }
        let path = parent.join(action);
        self.directories
            .push(PrivateDirectoryPreparation::default());
        let original = self.directories.last_mut().unwrap();
        original.create(&path, window.deadline())?;
        original.take_complete()
    }
    pub(crate) fn release(&mut self, end: Instant) -> io::Result<()> {
        if self.revoked {
            return Err(invalid());
        }
        self.lease
            .as_mut()
            .ok_or_else(invalid)?
            .release_retained(&mut self.released, end)
    }
    pub(crate) fn released_and_probe(
        &mut self,
    ) -> io::Result<(&mut ReleasedLease, &mut ReleaseProbe)> {
        if self.revoked {
            return Err(invalid());
        }
        Ok((self.released.as_mut().ok_or_else(invalid)?, &mut self.probe))
    }
    fn revoke(&mut self) {
        self.revoked = true;
        self.preparation.revoke_retained();
        if let Some(lease) = &mut self.lease {
            lease.revoke_retained();
        }
    }
    fn released_for_transfer(&mut self, end: Instant) -> io::Result<()> {
        if self.revoked || !self.preparation.empty_after_transfer() {
            return Err(invalid());
        }
        // release_retained already moved all exact held files into this native
        // receipt BEFORE fallible readback. No JSON/boolean can supply it.
        let released = self.released.as_mut().ok_or_else(invalid)?;
        released.check(end)?;
        if self.probe.started() {
            self.probe.check(released, end)?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Registered,
    Running,
    FailedRetained,
    ExpiredRetained,
    LostWorkerUnknown,
    ReturnedForValidation,
    ReturnUndeliveredRetained,
}
struct State {
    phase: Phase,
    thread: Option<String>,
    raw: Option<i32>,
    kind: Option<String>,
    binding: TransferBinding,
}
/// NonClone original control. Status takes a bounded try_lock and never owns
/// the worker's actual task, active lease or a mutation callback.
pub(crate) struct WorkerControl<T> {
    state: Arc<Mutex<State>>,
    returned: Mutex<mpsc::Receiver<Returned<T>>>,
    worker: thread::JoinHandle<()>,
}
pub(crate) struct Returned<T> {
    pub(crate) original: T,
    pub(crate) released: ReleasedLease,
}
fn publish(state: &Arc<Mutex<State>>, phase: Phase, error: Option<&io::Error>) {
    if let Ok(mut state) = state.lock() {
        state.phase = phase;
        state.thread = Some(format!("{:?}", thread::current().id()));
        state.raw = error.and_then(|e| e.raw_os_error());
        state.kind = error.map(|e| format!("{:?}", e.kind()));
    }
}
fn park<T>(
    mut original: T,
    mut local: WorkerLocal,
    state: Arc<Mutex<State>>,
    undelivered: Option<Returned<T>>,
) -> ! {
    // Mutable original resources live OUTSIDE the unwind closure and stay here.
    // No shutdown/cleanup/disconnect command exists; worker never runs callbacks
    // or acquires a new deadline in this state. Process crash remains Unknown.
    loop {
        std::hint::black_box((&mut original, &mut local, &state, &undelivered));
        thread::park();
    }
}
impl<T: Original> WorkerControl<T> {
    fn start<F>(
        slot: &mut Option<T>,
        window: BeforeResumeRecoveryWindow,
        run: F,
    ) -> io::Result<Self>
    where
        F: FnOnce(&mut T, &mut WorkerLocal, &BeforeResumeRecoveryWindow) -> io::Result<()>
            + Send
            + 'static,
    {
        let binding = slot.as_mut().ok_or_else(invalid)?.binding(&window)?;
        let state = Arc::new(Mutex::new(State {
            phase: Phase::Registered,
            thread: None,
            raw: None,
            kind: None,
            binding,
        }));
        let (incoming, receive) = mpsc::sync_channel::<T>(1);
        let (returner, returned) = mpsc::channel();
        let worker_state = state.clone();
        // No original Owner is captured by the fallible spawn closure. If
        // spawn fails, slot still owns it. If send fails, SendError returns it.
        let worker = thread::Builder::new()
            .name("windows024-original-recovery".into())
            .spawn(move || {
                let Ok(mut original) = receive.recv() else {
                    return;
                };
                let mut local = WorkerLocal::new();
                let result = catch_unwind(AssertUnwindSafe(|| {
                    let binding = original.binding(&window)?;
                    if worker_state.lock().map_err(|_| invalid())?.binding != binding {
                        return Err(invalid());
                    }
                    publish(&worker_state, Phase::Running, None);
                    run(&mut original, &mut local, &window)?;
                    let after = original.binding(&window)?;
                    if worker_state.lock().map_err(|_| invalid())?.binding != after {
                        return Err(invalid());
                    }
                    local.released_for_transfer(window.deadline())
                }));
                match result {
                    Ok(Ok(())) => {
                        let receipt = local.released.take().unwrap();
                        let value = Returned {
                            original,
                            released: receipt,
                        };
                        match returner.send(value) {
                            Ok(()) => publish(&worker_state, Phase::ReturnedForValidation, None),
                            Err(error) => {
                                // The observer disappeared. It cannot make the
                                // worker drop returned original objects or autoStart.
                                let value = error.0;
                                publish(&worker_state, Phase::ReturnUndeliveredRetained, None);
                                let Returned {
                                    mut original,
                                    released,
                                } = value;
                                local.released = Some(released);
                                original.revoke();
                                local.revoke();
                                park(original, local, worker_state, None);
                            }
                        }
                    }
                    Ok(Err(error)) => {
                        original.revoke();
                        local.revoke();
                        let phase = if Instant::now() >= window.deadline() {
                            Phase::ExpiredRetained
                        } else {
                            Phase::FailedRetained
                        };
                        publish(&worker_state, phase, Some(&error));
                        park(original, local, worker_state, None);
                    }
                    Err(_) => {
                        original.revoke();
                        local.revoke();
                        publish(&worker_state, Phase::LostWorkerUnknown, None);
                        park(original, local, worker_state, None);
                    }
                }
            })?;
        let original = slot.take().unwrap();
        if let Err(error) = incoming.send(original) {
            *slot = Some(error.0);
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "original worker transfer failed; Owner returned to slot",
            ));
        }
        Ok(Self {
            state,
            returned: Mutex::new(returned),
            worker,
        })
    }
}
impl WorkerControl<Owner> {
    pub(crate) fn start_owner(
        slot: &mut Option<Owner>,
        window: BeforeResumeRecoveryWindow,
        run: fn(&mut Owner, &mut WorkerLocal, &BeforeResumeRecoveryWindow) -> io::Result<()>,
    ) -> io::Result<Self> {
        Self::start(slot, window, run)
    }
}
impl<T> WorkerControl<T> {
    pub(crate) fn observe(&self, end: Instant) -> io::Result<serde_json::Value> {
        if Instant::now() >= end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        let state = self
            .state
            .try_lock()
            .map_err(|_| io::Error::new(io::ErrorKind::WouldBlock, "worker projection busy"))?;
        let phase = if self.worker.is_finished() && state.phase != Phase::ReturnedForValidation {
            Phase::LostWorkerUnknown
        } else {
            state.phase
        };
        let data = serde_json::json!({"schema":"windows-original-recovery-worker-v1","phase":format!("{phase:?}"),"sessionId":state.binding.session,
            "actionId":state.binding.action,"scopeDigest":state.binding.scope,"workerThread":state.thread,
            "originalDeadlineExpired":Instant::now()>=state.binding.end,"rawError":state.raw,"errorKind":state.kind,
            "workerFinished":self.worker.is_finished(),"recoveryRequired":true,"normalStop":false,"closed":false,"runtimeAuthority":false});
        if Instant::now() >= end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        Ok(data)
    }
    pub(crate) fn take_returned(&self) -> io::Result<Option<Returned<T>>> {
        match self.returned.try_lock().map_err(|_| invalid())?.try_recv() {
            Ok(original) => Ok(Some(original)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => Err(io::ErrorKind::BrokenPipe.into()),
        }
    }
}
#[cfg(test)]
mod tests;
#[cfg(test)]
pub(in crate::infrastructure::windows_client_originals024) mod component {
    use super::*;
    pub(in crate::infrastructure::windows_client_originals024) trait Task:
        Send + 'static
    {
        fn actual_binding(
            &mut self,
            window: &BeforeResumeRecoveryWindow,
        ) -> io::Result<[String; 3]>;
        fn run(
            &mut self,
            local: &mut WorkerLocal,
            window: &BeforeResumeRecoveryWindow,
        ) -> io::Result<()>;
        fn revoke(&mut self);
    }
    pub(in crate::infrastructure::windows_client_originals024) struct OriginalTask<T: Task> {
        pub(in crate::infrastructure::windows_client_originals024) task: T,
    }
    impl<T: Task> Original for OriginalTask<T> {
        fn binding(&mut self, w: &BeforeResumeRecoveryWindow) -> io::Result<TransferBinding> {
            let [session, action, scope] = self.task.actual_binding(w)?;
            Ok(TransferBinding {
                session,
                action,
                scope,
                end: w.deadline(),
            })
        }
        fn revoke(&mut self) {
            self.task.revoke();
        }
    }
    pub(in crate::infrastructure::windows_client_originals024) fn start<T: Task>(
        slot: &mut Option<OriginalTask<T>>,
        window: BeforeResumeRecoveryWindow,
    ) -> io::Result<WorkerControl<OriginalTask<T>>> {
        WorkerControl::start(slot, window, |original, local, w| {
            original.task.run(local, w)
        })
    }
    pub(in crate::infrastructure::windows_client_originals024) fn acquire(
        local: &mut WorkerLocal,
        paths: &[std::path::PathBuf; 4],
        end: Instant,
    ) -> io::Result<()> {
        local.preparation.prepare_fixture_paths(paths, end)?;
        local.lease = Some(local.preparation.take_complete()?);
        Ok(())
    }
}
