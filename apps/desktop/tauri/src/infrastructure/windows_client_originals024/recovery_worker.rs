//! Original owning-thread resource transport, not an AbortRestored factory.
//! Active locks are created only AFTER unique original state reaches this
//! worker; only actual released objects may ever leave it.
use super::{
    coordination::{
        AnchoredClaim, LeasePreparation, NativeLease, RecoveryPermit, ReleaseProbe, ReleasedLease,
    },
    phase::invalid,
    PendingAttachment, StartRanges,
};
use crate::infrastructure::windows_official_lifecycle024::Owner;
use crate::modules::runtime::windows_lifecycle024_contract::BeforeResumeRecoveryWindow;
use std::{
    io,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{mpsc, Arc, Mutex, OnceLock},
    thread,
    time::Instant,
};
mod canonical_trait;
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
    retained_recovery: Option<RecoveryPermit>,
    released: Option<ReleasedLease>,
    probe: ReleaseProbe,
    revoked: bool,
    directories:
        Vec<crate::infrastructure::windows_official_runtime024::PrivateDirectoryPreparation>,
    directory_pins: Vec<crate::infrastructure::windows_official_runtime024::PinnedDirectory>,
    canonical: Option<Box<canonical_trait::Attempt>>,
    projection: Option<Arc<Mutex<State>>>,
    pending_attachment: Option<PendingAttachment>,
}
impl WorkerLocal {
    pub(crate) fn finish_released_stage(
        &mut self,
        owner: &mut Owner,
        window: &BeforeResumeRecoveryWindow,
    ) -> io::Result<()> {
        let (released, probe) = self.released_and_probe()?;
        let proof = owner
            .with_originals(|originals, owner| {
                originals.finish_retained_quiescent(owner, released, probe, window)
            })
            .map_err(io::Error::other)?;
        owner
            .retain_before_resume_restored(proof, window)
            .map_err(io::Error::other)
    }
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
            retained_recovery: None,
            released: None,
            probe: ReleaseProbe::default(),
            revoked: false,
            directories: Vec::new(),
            directory_pins: Vec::new(),
            canonical: None,
            projection: None,
            pending_attachment: None,
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
    fn enroll_retained_start(&mut self, ranges: &mut StartRanges) -> io::Result<()> {
        self.lease = Some(ranges.take_retained_lease()?);
        Ok(())
    }
    fn prepare_canonical_lease(
        &mut self,
        owner: &mut Owner,
        window: &BeforeResumeRecoveryWindow,
    ) -> io::Result<()> {
        match self.lease.as_mut() {
            Some(lease) => {
                self.retained_recovery = Some(lease.authorize_retained_recovery(owner, window)?);
                Ok(())
            }
            None => self.acquire(owner, window),
        }
    }
    fn project_stage(&self, stage: canonical_trait::Stage) {
        if let Some(projection) = &self.projection {
            if let Ok(mut state) = projection.lock() {
                state.stage = Some(stage);
            }
        }
    }
    fn bound_claim<'a>(
        lease: &'a mut Option<NativeLease>,
        recovery: Option<&'a RecoveryPermit>,
        owner: &mut Owner,
        root: &std::path::Path,
        window: &BeforeResumeRecoveryWindow,
    ) -> io::Result<AnchoredClaim<'a>> {
        let lease = lease.as_mut().ok_or_else(invalid)?;
        let end = window.deadline();
        if let Some(permit) = recovery {
            lease.check_recovery_owner(permit, owner, window)?;
            return lease.recovery_claim(permit, root, end);
        }
        lease.check_owner(owner, end)?;
        lease.anchored_claim(root, end)
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
        let lease = self.lease.as_mut().ok_or_else(invalid)?;
        if let Some(permit) = &self.retained_recovery {
            lease.release_recovery_retained(permit, &mut self.released, end)
        } else {
            lease.release_retained(&mut self.released, end)
        }
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
    ReturnedValidated,
    ReturnValidationFailedRetained,
    HistoryReadFailedRetained,
    ReturnUndeliveredRetained,
}
struct State {
    phase: Phase,
    stage: Option<canonical_trait::Stage>,
    thread: Option<String>,
    validator_thread: Option<String>,
    raw: Option<i32>,
    kind: Option<String>,
    binding: Option<TransferBinding>,
    preparation_pending: Option<PreparationPending>,
}
struct PreparationPending {
    records: [Vec<u8>; 2],
    start_raw: Option<i32>,
    attachment: Option<io::Result<()>>,
}
enum WorkerThread {
    Dedicated(thread::JoinHandle<()>),
    InitialStart(Arc<OnceLock<thread::JoinHandle<()>>>),
}
impl WorkerThread {
    fn is_finished(&self) -> bool {
        match self {
            Self::Dedicated(thread) => thread.is_finished(),
            Self::InitialStart(thread) => thread.get().is_none_or(thread::JoinHandle::is_finished),
        }
    }
    #[cfg(test)]
    fn join(self) -> thread::Result<()> {
        match self {
            Self::Dedicated(worker) => worker.join(),
            Self::InitialStart(_) => panic!("the retained Start test owns its thread"),
        }
    }
}
/// NonClone original control. Status takes a bounded try_lock and never owns
/// the worker's actual task, active lease or a mutation callback.
pub(crate) struct WorkerControl<T> {
    state: Arc<Mutex<State>>,
    returned: Mutex<mpsc::Receiver<Returned<T>>>,
    worker: WorkerThread,
    registration: Option<RetainedRegistration>,
    validation: Mutex<return_validation::Validation<T>>,
}
struct RetainedRegistration {
    session: String,
    scope: String,
    deadline: Instant,
}
pub(crate) struct RetainedRequest {
    window: BeforeResumeRecoveryWindow,
    state: Arc<Mutex<State>>,
    returner: mpsc::Sender<Returned<Owner>>,
}
pub(crate) struct Returned<T> {
    pub(crate) original: Box<T>,
    pub(crate) released: ReleasedLease,
    window: BeforeResumeRecoveryWindow,
}
fn publish(state: &Arc<Mutex<State>>, phase: Phase, error: Option<&io::Error>) {
    if let Ok(mut state) = state.lock() {
        state.phase = phase;
        state.thread = Some(format!("{:?}", thread::current().id()));
        state.raw = error.and_then(|e| e.raw_os_error());
        state.kind = error.map(|e| format!("{:?}", e.kind()));
    }
}
fn park<T>(mut original: T, mut local: WorkerLocal, state: Arc<Mutex<State>>) -> ! {
    // Mutable original resources live OUTSIDE the unwind closure and stay here.
    // No shutdown/cleanup/disconnect command exists; worker never runs callbacks
    // or acquires a new deadline in this state. Process crash remains Unknown.
    loop {
        std::hint::black_box((&mut original, &mut local, &state));
        thread::park();
    }
}
impl State {
    fn registered(binding: Option<TransferBinding>) -> Self {
        Self {
            phase: Phase::Registered,
            stage: None,
            thread: None,
            validator_thread: None,
            raw: None,
            kind: None,
            binding,
            preparation_pending: None,
        }
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
        let state = Arc::new(Mutex::new(State::registered(Some(binding))));
        let (incoming, receive) = mpsc::sync_channel::<Box<T>>(1);
        let (returner, returned) = mpsc::channel();
        let worker_state = state.clone();
        // No original Owner is captured by the fallible spawn closure. If
        // spawn fails, slot still owns it. If send fails, SendError returns it.
        let worker = thread::Builder::new()
            .name("windows024-original-recovery".into())
            .spawn(move || {
                let Ok(original) = receive.recv() else {
                    return;
                };
                run_worker(
                    original,
                    window,
                    worker_state,
                    returner,
                    WorkerLocal::new(),
                    |_, _, _, _| Ok(()),
                    run,
                );
            })?;
        let original = Box::new(slot.take().unwrap());
        if let Err(error) = incoming.send(original) {
            *slot = Some(*error.0);
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "original worker transfer failed; Owner returned to slot",
            ));
        }
        Ok(Self {
            state,
            returned: Mutex::new(returned),
            worker: WorkerThread::Dedicated(worker),
            registration: None,
            validation: Mutex::new(return_validation::Validation::new()),
        })
    }
}
fn run_worker<T: Original>(
    mut original: Box<T>,
    window: BeforeResumeRecoveryWindow,
    state: Arc<Mutex<State>>,
    returner: mpsc::Sender<Returned<T>>,
    mut local: WorkerLocal,
    prepare: impl FnOnce(
        &mut T,
        &mut WorkerLocal,
        &BeforeResumeRecoveryWindow,
        &Mutex<State>,
    ) -> io::Result<()>,
    run: impl FnOnce(&mut T, &mut WorkerLocal, &BeforeResumeRecoveryWindow) -> io::Result<()>,
) {
    local.projection = Some(state.clone());
    let result = catch_unwind(AssertUnwindSafe(|| {
        if Instant::now() >= window.deadline() {
            return Err(io::ErrorKind::TimedOut.into());
        }
        publish(&state, Phase::Running, None);
        prepare(&mut original, &mut local, &window, &state)?;
        let binding = original.binding(&window)?;
        if state.lock().map_err(|_| invalid())?.binding.as_ref() != Some(&binding) {
            return Err(invalid());
        }
        run(&mut original, &mut local, &window)?;
        if original.binding(&window)? != binding {
            return Err(invalid());
        }
        local.released_for_transfer(window.deadline())
    }));
    let error = match result {
        Ok(Ok(())) => {
            let value = Returned {
                original,
                released: local.released.take().unwrap(),
                window,
            };
            match returner.send(value) {
                Ok(()) => {
                    publish(&state, Phase::ReturnedForValidation, None);
                    return;
                }
                Err(error) => {
                    let mut value = error.0;
                    value.original.revoke();
                    local.revoke();
                    publish(&state, Phase::ReturnUndeliveredRetained, None);
                    park(value, local, state);
                }
            }
        }
        Ok(Err(error)) => Some(error),
        Err(_) => None,
    };
    original.revoke();
    local.revoke();
    let phase = if error.is_none() {
        Phase::LostWorkerUnknown
    } else if Instant::now() >= window.deadline() {
        Phase::ExpiredRetained
    } else {
        Phase::FailedRetained
    };
    publish(&state, phase, error.as_ref());
    park(original, local, state);
}
impl WorkerControl<Owner> {
    pub(crate) fn register_retained_start(
        session: String,
        scope: String,
        window: BeforeResumeRecoveryWindow,
        worker: Arc<OnceLock<thread::JoinHandle<()>>>,
    ) -> (Self, RetainedRequest) {
        let state = Arc::new(Mutex::new(State::registered(None)));
        let (returner, returned) = mpsc::channel();
        let control = Self {
            state: state.clone(),
            returned: Mutex::new(returned),
            worker: WorkerThread::InitialStart(worker),
            registration: Some(RetainedRegistration {
                session,
                scope,
                deadline: window.deadline(),
            }),
            validation: Mutex::new(return_validation::Validation::new()),
        };
        (
            control,
            RetainedRequest {
                window,
                state,
                returner,
            },
        )
    }
    pub(crate) fn start_owner(
        slot: &mut Option<Owner>,
        window: BeforeResumeRecoveryWindow,
        run: fn(&mut Owner, &mut WorkerLocal, &BeforeResumeRecoveryWindow) -> io::Result<()>,
    ) -> io::Result<Self> {
        Self::start(slot, window, run)
    }
}
impl RetainedRequest {
    pub(crate) fn execute(
        self,
        owner: Owner,
        ranges: &mut StartRanges,
        start_raw: Option<i32>,
        pending_attachment: Option<PendingAttachment>,
        check: impl FnOnce(&Owner) -> io::Result<()>,
        run: fn(&mut Owner, &mut WorkerLocal, &BeforeResumeRecoveryWindow) -> io::Result<()>,
    ) {
        let mut local = WorkerLocal::new();
        local.pending_attachment = pending_attachment;
        run_worker(
            Box::new(owner),
            self.window,
            self.state,
            self.returner,
            local,
            |owner, local, window, state| {
                check(owner)?;
                let partial = owner
                    .with_originals(|originals, _| Ok(originals.has_failed_preparation()))
                    .map_err(io::Error::other)?;
                if partial {
                    owner
                        .prepare_before_resume_process(window)
                        .map_err(io::Error::other)?;
                    let records = owner
                        .with_originals(|originals, owner| {
                            originals.read_before_resume_preparation_record(owner, window)
                        })
                        .map_err(io::Error::other)?
                        .preparation_pending_data(start_raw)?;
                    {
                        let mut projection = state.lock().map_err(|_| invalid())?;
                        projection.preparation_pending = Some(PreparationPending {
                            records: records.clone(),
                            start_raw,
                            attachment: None,
                        });
                    }
                    let binding = owner.binding(window)?;
                    state.lock().map_err(|_| invalid())?.binding = Some(binding);
                    if let Some(attachment) = &mut local.pending_attachment {
                        let end = window.deadline();
                        let outcome = (|| {
                            let origin = owner.before_resume_lease_binding(window).map_err(io::Error::other)?;
                            attachment.persist(
                                owner.data_path(),
                                [owner.original_host_instance(), origin.owner, origin.failed_start],
                                origin.recovery,
                                &records,
                                end,
                            )?;
                            owner.check_before_resume_process(window).map_err(io::Error::other)
                        })();
                        let mut projection = state.lock().map_err(|_| invalid())?;
                        projection.preparation_pending.as_mut().ok_or_else(invalid)?.attachment =
                            Some(outcome.and_then(|()| {
                                if Instant::now() >= end {
                                    return Err(io::ErrorKind::TimedOut.into());
                                }
                                Ok(())
                            }));
                    }
                    // A partial prefix supplies observation only. StartRanges
                    // stays on this thread, even if it holds a complete lease.
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "original preparation pending; restore authority unavailable",
                    ));
                }
                // If Start acquired only a prefix, its exact original ranges stay
                // where they were enrolled and this action cannot acquire replacements.
                local.enroll_retained_start(ranges)?;
                owner
                    .prepare_before_resume_process(window)
                    .map_err(io::Error::other)?;
                let binding = owner.binding(window)?;
                state.lock().map_err(|_| invalid())?.binding = Some(binding);
                Ok(())
            },
            run,
        );
    }
}
impl<T> WorkerControl<T> {
    pub(crate) fn observe(&self, end: Instant) -> io::Result<serde_json::Value> {
        if Instant::now() >= end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        let state = self.state.try_lock().map_err(|error| match error {
            std::sync::TryLockError::WouldBlock => {
                io::Error::new(io::ErrorKind::WouldBlock, "worker projection busy")
            }
            std::sync::TryLockError::Poisoned(_) => invalid(),
        })?;
        let finished = self.worker.is_finished();
        let phase = if finished
            && !matches!(
                state.phase,
                Phase::ReturnedForValidation
                    | Phase::ReturnedValidated
                    | Phase::ReturnValidationFailedRetained
                    | Phase::HistoryReadFailedRetained
            ) {
            Phase::LostWorkerUnknown
        } else {
            state.phase
        };
        let (session, action, scope, deadline) = match (&state.binding, &self.registration) {
            (Some(binding), _) => (
                &binding.session,
                Some(&binding.action),
                &binding.scope,
                binding.end,
            ),
            (None, Some(registration)) => (
                &registration.session,
                None,
                &registration.scope,
                registration.deadline,
            ),
            (None, None) => return Err(invalid()),
        };
        let pending = state.preparation_pending.as_ref().filter(|_| {
            matches!(phase, Phase::FailedRetained | Phase::ExpiredRetained)
        }).map(|original| {
            let [intent, pending] = &original.records;
            let failure = original.attachment.as_ref().and_then(|result| result.as_ref().err());
            use crate::infrastructure::hash::sha256_hex;
            serde_json::json!({
                "intentDigest": sha256_hex(intent),
                "pendingDigest": sha256_hex(pending),
                "startRawError": original.start_raw,
                "attachmentAttempted": original.attachment.is_some(),
                "attachmentRawError": failure.and_then(io::Error::raw_os_error),
                "attachmentErrorKind": failure.map(|error| format!("{:?}", error.kind())),
                "durable": matches!(&original.attachment, Some(Ok(())))
            })
        });
        let data = serde_json::json!({"schema":"windows-original-recovery-worker-v1","phase":format!("{phase:?}"),"stage":state.stage.map(|stage|format!("{stage:?}")),"sessionId":session,
            "actionId":action,"scopeDigest":scope,"workerThread":state.thread,"returnValidatorThread":state.validator_thread,
            "originalDeadlineExpired":Instant::now()>=deadline,"rawError":state.raw,"errorKind":state.kind,
            "workerFinished":finished,"preparationPending":pending,"recoveryRequired":true,"normalStop":false,"closed":false,"runtimeAuthority":false});
        if Instant::now() >= end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        Ok(data)
    }
    #[cfg(test)]
    pub(crate) fn take_returned(&self) -> io::Result<Option<Returned<T>>> {
        match self.returned.try_lock().map_err(|_| invalid())?.try_recv() {
            Ok(original) => Ok(Some(original)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => Err(io::ErrorKind::BrokenPipe.into()),
        }
    }
}
mod return_validation;
#[cfg(test)]
pub(super) mod tests;
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
    pub(in crate::infrastructure::windows_client_originals024) fn join_success_for_test<T: Task>(
        control: WorkerControl<OriginalTask<T>>,
    ) -> io::Result<()> {
        let phase = control.state.try_lock().map_err(|_| invalid())?.phase;
        if phase != Phase::ReturnedForValidation {
            return Err(invalid());
        }
        control.worker.join().map_err(|_| {
            io::Error::new(
                io::ErrorKind::Other,
                "successful native worker unexpectedly panicked during own fixture closure",
            )
        })
    }
}
