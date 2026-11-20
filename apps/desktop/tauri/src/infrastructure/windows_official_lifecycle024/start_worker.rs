use super::*;
use crate::{
    infrastructure::windows_client_originals024::{
        recovery_worker::{RetainedRequest, WorkerControl},
        PendingAttachment, StartRanges,
    },
    modules::runtime::windows_lifecycle024_contract::BeforeResumeRecoveryWindow,
};
use std::{
    io,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::mpsc,
    thread,
};
use tokio::sync::Notify;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Phase {
    Registered,
    Running,
    FailedRetained,
    ExpiredRetained,
    LostWorkerUnknown,
    Returned,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Stage {
    Dispatch,
    Capture,
    BeforeResume,
    Resume,
    AwaitReady,
    RecordReady,
    Return,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Snapshot {
    pub(super) phase: Phase,
    stage: Stage,
    raw: Option<i32>,
    launcher_exit: Option<u32>,
    creator_resumed: Option<bool>,
}

struct State {
    snapshot: Snapshot,
    thread: Option<thread::ThreadId>,
    result: Option<AppResult<LifecycleResult>>,
    recovery: Option<RetainedRequest>,
    recovery_queued: bool,
    #[cfg(test)]
    range_probe: Option<RangeProbe>,
}

#[cfg(test)]
struct RangeProbe {
    request: uuid::Uuid,
    action: uuid::Uuid,
    end: Instant,
    snapshot: Snapshot,
    reply: mpsc::SyncSender<io::Result<serde_json::Value>>,
}

#[cfg(test)]
fn fixture_probe_eligible(state: &State, thread: thread::ThreadId) -> bool {
    !state.recovery_queued
        && state.thread == Some(thread)
        && state.snapshot.phase == Phase::FailedRetained
        && matches!(state.snapshot.stage, Stage::Capture | Stage::BeforeResume)
        && state.snapshot.raw == Some(31)
        && state.snapshot.creator_resumed == Some(false)
        && state.result.as_ref().is_some_and(Result::is_err)
}

#[cfg(test)]
fn probe_refused() -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        "original fixture Start range proof unavailable",
    )
}

pub(super) struct StartWorker {
    pub(super) session: uuid::Uuid,
    pub(super) scope: String,
    action: uuid::Uuid,
    root: native::ProcessIdentity,
    deadline: Instant,
    state: Mutex<State>,
    wake: Notify,
    thread: Arc<OnceLock<thread::JoinHandle<()>>>,
}

impl StartWorker {
    pub(super) fn dispatch(
        entry: &Arc<Slot>,
        slot: &mut Option<Owner>,
        registered: &mut Option<Arc<Self>>,
        plan: shared::LaunchPlan,
        window: shared::ActionWindow,
        admitted: StartAdmission,
    ) -> AppResult<Arc<Self>> {
        window.remaining()?;
        if registered.is_some() {
            return Err(recovery_required());
        }
        let owner = slot.as_ref().ok_or_else(recovery_required)?;
        let control = Arc::new(Self {
            session: owner.session,
            scope: owner.scope()?,
            action: owner.start_action.id(),
            root: owner.native.binding().process.clone(),
            deadline: window.deadline(),
            state: Mutex::new(State {
                snapshot: Snapshot {
                    phase: Phase::Registered,
                    stage: Stage::Dispatch,
                    raw: None,
                    launcher_exit: None,
                    creator_resumed: Some(owner.native.creator_has_resumed()),
                },
                thread: None,
                result: None,
                recovery: None,
                recovery_queued: false,
                #[cfg(test)]
                range_probe: None,
            }),
            wake: Notify::new(),
            thread: Arc::new(OnceLock::new()),
        });
        let (send, receive) = mpsc::sync_channel::<Owner>(1);
        let worker_entry = entry.clone();
        let worker_control = control.clone();
        let worker = thread::Builder::new()
            .name("windows024-original-start".into())
            .spawn(move || {
                if let Ok(owner) = receive.recv() {
                    worker_control.run(worker_entry, owner, plan, window, admitted);
                }
            })
            .map_err(|error| {
                fail(
                    "windows024_start_dispatch_failed",
                    &format!(
                        "原启动工作线程未创建；保留原 Owner。raw={:?}",
                        error.raw_os_error()
                    ),
                )
            })?;
        let _ = control.thread.set(worker);
        *registered = Some(control.clone());
        if let Err(error) = send.send(slot.take().unwrap()) {
            *slot = Some(error.0);
            *registered = None;
            return Err(unknown(()));
        }
        Ok(control)
    }

    #[cfg(test)]
    pub(super) fn probe_ranges(
        &self,
        request: uuid::Uuid,
        action: uuid::Uuid,
        end: Instant,
    ) -> io::Result<serde_json::Value> {
        let remaining = || {
            end.checked_duration_since(Instant::now())
                .filter(|time| !time.is_zero())
                .ok_or_else(|| io::Error::from(io::ErrorKind::TimedOut))
        };
        remaining()?;
        let worker = self
            .thread
            .get()
            .filter(|thread| !thread.is_finished())
            .ok_or_else(probe_refused)?;
        let (reply, receive) = mpsc::sync_channel(1);
        {
            let mut state = self.state.try_lock().map_err(|_| probe_refused())?;
            if !fixture_probe_eligible(&state, worker.thread().id())
                || state.range_probe.is_some()
                || request == action
                || action == self.action
            {
                return Err(probe_refused());
            }
            remaining()?;
            let snapshot = state.snapshot;
            state.range_probe = Some(RangeProbe {
                request,
                action,
                end,
                snapshot,
                reply,
            });
        }
        worker.thread().unpark();
        let proof = receive
            .recv_timeout(remaining()?)
            .map_err(|error| match error {
                mpsc::RecvTimeoutError::Timeout => io::ErrorKind::TimedOut.into(),
                mpsc::RecvTimeoutError::Disconnected => probe_refused(),
            })??;
        remaining()?;
        Ok(proof)
    }

    #[cfg(test)]
    fn read_probe(
        &self,
        probe: &RangeProbe,
        owner: &mut Owner,
        ranges: &mut StartRanges,
    ) -> io::Result<serde_json::Value> {
        let thread = thread::current().id();
        let check = |owner: &Owner| {
            if Instant::now() >= probe.end {
                return Err(io::ErrorKind::TimedOut.into());
            }
            let state = self.state.try_lock().map_err(|_| probe_refused())?;
            if !fixture_probe_eligible(&state, thread)
                || state.snapshot != probe.snapshot
                || self
                    .thread
                    .get()
                    .is_none_or(|registered| registered.thread().id() != thread)
                || owner.session != self.session
                || owner.start_action.id() != self.action
                || owner.native.binding().process != self.root
                || owner.native.creator_has_resumed()
                || owner.scope().map_err(io::Error::other)? != self.scope
            {
                return Err(probe_refused());
            }
            Ok(())
        };
        check(owner)?;
        let held = crate::infrastructure::windows_client_originals024::recovery_worker::inspect_fixture_start_ranges(
            ranges, owner, probe.action, self.deadline, probe.end,
        )?;
        check(owner)?;
        Ok(serde_json::json!({
            "requestId":probe.request, "transportActionId":probe.action,
            "hostInstanceId":owner.original_host_instance(), "sessionId":self.session,
            "failedActionId":self.action, "scopeDigest":self.scope,
            "workerThread":format!("{thread:?}"), "startRawError":probe.snapshot.raw,
            "creatorResumed":false, "originalRanges":held,
        }))
    }

    pub(super) fn queue_recovery(
        &self,
        window: BeforeResumeRecoveryWindow,
    ) -> AppResult<WorkerControl<Owner>> {
        budget(window.deadline())?;
        let worker = self
            .thread
            .get()
            .filter(|thread| !thread.is_finished())
            .ok_or_else(recovery_required)?;
        let mut state = self.state.try_lock().map_err(unknown)?;
        if state.recovery_queued
            || !matches!(
                state.snapshot.phase,
                Phase::FailedRetained | Phase::ExpiredRetained
            )
            || !matches!(state.snapshot.stage, Stage::Capture | Stage::BeforeResume)
            || state.snapshot.creator_resumed != Some(false)
            || state.thread != Some(worker.thread().id())
        {
            return Err(recovery_required());
        }
        budget(window.deadline())?;
        #[cfg(test)]
        super::super::windows_client_originals024::recovery_timing::original_worker(
            worker.thread().id(),
            self.action,
            self.session,
        );
        let (control, request) = WorkerControl::register_retained_start(
            self.session.to_string(),
            self.scope.clone(),
            window,
            self.thread.clone(),
        );
        state.recovery_queued = true;
        state.recovery = Some(request);
        drop(state);
        #[cfg(test)]
        super::super::windows_client_originals024::recovery_timing::mark(
            "original-worker-enqueued",
            "return",
            None,
            None,
        );
        worker.thread().unpark();
        Ok(control)
    }

    pub(super) async fn wait(&self) -> AppResult<LifecycleResult> {
        loop {
            let result = self.state.lock().map_err(unknown)?.result.clone();
            if let Some(result) = result {
                if result.is_ok() {
                    budget(self.deadline)?;
                }
                return result;
            }
            let notified = self.wake.notified();
            tokio::time::timeout_at(self.deadline.into(), notified)
                .await
                .map_err(|_| AppError::Timeout)?;
        }
    }

    pub(super) fn observe(&self, end: Instant) -> AppResult<(Snapshot, serde_json::Value)> {
        budget(end)?;
        let state = self.state.try_lock().map_err(unknown)?;
        let mut sample = state.snapshot;
        if !state.recovery_queued
            && self
                .thread
                .get()
                .is_none_or(thread::JoinHandle::is_finished)
            && sample.phase != Phase::Returned
        {
            sample.phase = Phase::LostWorkerUnknown;
            sample.creator_resumed = None;
        }
        let payload = serde_json::json!({
            "sessionId":self.session, "actionId":self.action, "scopeDigest":self.scope,
            "phase":format!("{:?}",sample.phase), "stage":format!("{:?}",sample.stage),
            "thread":state.thread.map(|id| format!("{id:?}")),
            "originalDeadlineExpired":Instant::now()>=self.deadline,
            "rawError":sample.raw, "launcherExitCode":sample.launcher_exit,
            "creatorResumed":sample.creator_resumed,
            "binding":{"pid":self.root.pid,"filetime":self.root.creation_filetime.to_string(),"image":self.root.executable}
        });
        budget(end)?;
        Ok((sample, payload))
    }

    pub(super) fn stage(&self, stage: Stage) {
        if let Ok(mut state) = self.state.lock() {
            state.snapshot.stage = stage;
        }
    }

    fn run(
        &self,
        entry: Arc<Slot>,
        owner: Owner,
        plan: shared::LaunchPlan,
        window: shared::ActionWindow,
        admitted: StartAdmission,
    ) {
        let mut owner = Some(owner);
        let mut ranges = StartRanges::retained();
        let mut pending_attachment = None;
        let capture_session = uuid::Uuid::new_v4();
        let mut raw = None;
        let mut launcher_exit = None;
        let run = catch_unwind(AssertUnwindSafe(|| {
            let original = owner.as_mut().ok_or_else(recovery_required)?;
            let action = original.original_action(OriginalActionKind::Start, self.deadline)?;
            if original.session != self.session
                || original.scope()? != self.scope
                || original.start_action.id() != self.action
                || original.native.binding().process != self.root
                || window.deadline() != self.deadline
            {
                return Err(recovery_required());
            }
            action.check(original, self.deadline)?;
            {
                let mut state = self.state.lock().map_err(unknown)?;
                state.snapshot.phase = Phase::Running;
                state.thread = Some(thread::current().id());
            }
            if let Some(parent) = admitted.pending_parent {
                let original_data = original.scope_pins.first().ok_or_else(recovery_required)?;
                if parent.path() != original.data_path()
                    || parent.identity() != original_data.identity()
                {
                    return Err(recovery_required());
                }
                pending_attachment = Some(
                    PendingAttachment::bind(
                        parent,
                        [original.original_host_instance(), self.session, self.action],
                        capture_session,
                        self.deadline,
                    )
                    .map_err(unknown)?,
                );
            }
            #[cfg(test)]
            require_isolated_injection(
                original.fixture_isolation().is_some(),
                admitted.capture_failure.is_some(),
            )?;
            #[cfg(test)]
            let _scans = admitted.capture_failure.as_ref().map(|_| {
                super::super::windows_client_originals024::recovery_timing::StartScans::begin(
                    "start-worker",
                    self.deadline,
                )
            });
            #[cfg(test)]
            if let Some(arm) = admitted.capture_failure {
                ranges.arm_capture_failure(arm, original, capture_session, self.deadline)?;
            }
            tauri::async_runtime::block_on(self.start(
                &entry,
                original,
                plan,
                window,
                capture_session,
                &mut ranges,
                &mut raw,
                &mut launcher_exit,
            ))?;
            self.stage(Stage::Return);
            action.check(original, self.deadline)?;
            if !ranges.empty() || !matches!(original.original_phase, Some(LivePhase::Ready(_))) {
                return Err(recovery_required());
            }
            let mut gate = entry.start_worker.lock().map_err(unknown)?;
            let mut slot = entry.owner.lock().map_err(unknown)?;
            let mut state = self.state.lock().map_err(unknown)?;
            if !gate
                .as_ref()
                .is_some_and(|registered| std::ptr::eq(Arc::as_ptr(registered), self))
                || slot.is_some()
            {
                return Err(recovery_required());
            }
            budget(self.deadline)?;
            *slot = owner.take();
            state.snapshot.phase = Phase::Returned;
            state.result = Some(Ok(LifecycleResult::Started));
            *gate = None;
            Ok(())
        }));
        if matches!(run, Ok(Ok(()))) {
            self.wake.notify_one();
            return;
        }
        let (phase, error) = match run {
            Ok(Err(error)) if Instant::now() >= self.deadline => (Phase::ExpiredRetained, error),
            Ok(Err(error)) => (Phase::FailedRetained, error),
            Err(_) => (Phase::LostWorkerUnknown, unknown(())),
            Ok(Ok(())) => unreachable!(),
        };
        ranges.revoke();
        let start_raw = raw.or(ranges.raw_error());
        let result = Err(error);
        let resumed = owner
            .as_ref()
            .map(|owner| owner.native.creator_has_resumed());
        if let Some(owner) = &mut owner {
            owner.remember_start_failure(&result);
        }
        if let Ok(mut state) = self.state.lock() {
            state.snapshot.phase = phase;
            state.snapshot.raw = start_raw;
            state.snapshot.launcher_exit = launcher_exit;
            state.snapshot.creator_resumed = resumed;
            state.result = Some(result);
        }
        self.wake.notify_one();
        loop {
            #[cfg(test)]
            {
                let probe = self
                    .state
                    .lock()
                    .ok()
                    .and_then(|mut state| state.range_probe.take());
                if let Some(probe) = probe {
                    let result = owner
                        .as_mut()
                        .ok_or_else(probe_refused)
                        .and_then(|owner| self.read_probe(&probe, owner, &mut ranges));
                    let _ = probe.reply.send(result);
                }
            }
            let request = self
                .state
                .lock()
                .ok()
                .and_then(|mut state| state.recovery.take());
            if let Some(request) = request {
                if let Some(original) = owner.take() {
                    request.execute(
                        original,
                        &mut ranges,
                        start_raw,
                        pending_attachment.take(),
                        |owner| {
                            let thread = self
                                .thread
                                .get()
                                .ok_or_else(|| io::Error::other(recovery_required()))?;
                            if thread.thread().id() != thread::current().id()
                                || owner.session != self.session
                                || owner.scope().map_err(io::Error::other)? != self.scope
                                || owner.start_action.id() != self.action
                                || owner.native.binding().process != self.root
                                || owner.native.creator_has_resumed()
                            {
                                return Err(io::Error::other(recovery_required()));
                            }
                            Ok(())
                        },
                        actions::run_before_resume_worker,
                    );
                    return;
                }
            }
            std::hint::black_box((&mut owner, &mut ranges, &entry, self));
            thread::park();
        }
    }

    async fn start(
        &self,
        entry: &Slot,
        owner: &mut Owner,
        plan: shared::LaunchPlan,
        window: shared::ActionWindow,
        capture_session: uuid::Uuid,
        ranges: &mut StartRanges,
        raw: &mut Option<i32>,
        launcher_exit: &mut Option<u32>,
    ) -> AppResult<()> {
        window.remaining()?;
        check_cancel(entry)?;
        self.stage(Stage::Capture);
        owner.capture_originals(capture_session, &window, ranges)?;
        #[cfg(test)]
        require_resume_allowed(owner.fixture_isolation().is_some())?;
        plan.check_current(&entry.runtime)?;
        check_cancel(entry)?;
        self.stage(Stage::BeforeResume);
        owner.originals_before_resume(window.deadline(), ranges)?;
        plan.check_current(&entry.runtime)?;
        check_cancel(entry)?;
        budget(window.deadline())?;
        {
            let mut state = self.state.lock().map_err(unknown)?;
            state.snapshot.stage = Stage::Resume;
            state.snapshot.creator_resumed = None;
        }
        #[cfg(test)]
        require_resume_allowed(owner.fixture_isolation().is_some())?;
        owner.native.resume().map_err(|error| {
            *raw = error.raw_os_error();
            unknown(error)
        })?;
        {
            let mut state = self.state.lock().map_err(unknown)?;
            state.snapshot.stage = Stage::AwaitReady;
            state.snapshot.creator_resumed = Some(owner.native.creator_has_resumed());
        }
        loop {
            window.remaining()?;
            check_cancel(entry)?;
            if let ProcessState::Exited { code } = owner.native.root_state().map_err(|error| {
                *raw = error.raw_os_error();
                unknown(error)
            })? {
                if code != 0 {
                    *launcher_exit = Some(code);
                    return Err(fail(
                        "windows024_start_exit",
                        &format!("官方launcher非零退出 exit_code={code}；原owner保留"),
                    ));
                }
            }
            match owner
                .observe_in(
                    window.deadline(),
                    Some((&window, false)),
                    Some((ranges, self)),
                )
                .await
            {
                Ok(_) => return Ok(()),
                Err(error) if owner.original_phase.is_none() => return Err(error),
                Err(_) => {}
            }
            tokio::time::sleep(Duration::from_millis(20).min(window.remaining()?)).await;
        }
    }
}

fn check_cancel(entry: &Slot) -> AppResult<()> {
    if entry.cancel.load(Ordering::SeqCst) {
        return Err(fail(
            "windows024_cancelled_retained",
            "取消后保留原创建句柄与Job，未推断退出",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn control(deadline: Instant) -> Arc<StartWorker> {
        Arc::new(StartWorker {
            session: uuid::Uuid::new_v4(),
            scope: "status-fixture".into(),
            action: uuid::Uuid::new_v4(),
            root: native::RetainedPeer::current().unwrap().identity().clone(),
            deadline,
            state: Mutex::new(State {
                snapshot: Snapshot {
                    phase: Phase::Running,
                    stage: Stage::BeforeResume,
                    raw: None,
                    launcher_exit: None,
                    creator_resumed: Some(false),
                },
                thread: None,
                result: None,
                recovery: None,
                recovery_queued: false,
                range_probe: None,
            }),
            wake: Notify::new(),
            thread: Arc::new(OnceLock::new()),
        })
    }

    #[test]
    fn wl2_r7_start_worker_status_stays_bounded_and_reports_lost_worker_unknown() {
        let started = Instant::now();
        let end = started + Duration::from_secs(5);
        let worker = control(end);
        let state = worker.state.lock().unwrap();
        assert!(worker.observe(end).is_err());
        drop(state);
        let (snapshot, facts) = worker.observe(end).unwrap();
        assert_eq!(snapshot.phase, Phase::LostWorkerUnknown);
        assert_eq!(facts["phase"], "LostWorkerUnknown");
        assert_eq!(facts["stage"], "BeforeResume");
        assert!(facts["creatorResumed"].is_null());
        assert_eq!(facts["binding"]["pid"], std::process::id());
        assert!(worker.observe(started).is_err());
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn wl2_r7_fixture_range_probe_dispatches_once_to_registered_thread_and_keeps_native_error() {
        let worker = control(Instant::now());
        let assigned = worker.clone();
        let (ready, started) = mpsc::sync_channel(1);
        let end = Instant::now() + Duration::from_secs(5);
        let request = uuid::Uuid::new_v4();
        let action = uuid::Uuid::new_v4();
        let thread = thread::spawn(move || {
            let id = thread::current().id();
            assigned.state.lock().unwrap().thread = Some(id);
            ready.send(id).unwrap();
            loop {
                assert!(Instant::now() < end);
                let probe = assigned.state.lock().unwrap().range_probe.take();
                if let Some(probe) = probe {
                    assert_eq!(
                        (probe.request, probe.action, probe.end),
                        (request, action, end)
                    );
                    assert_eq!(probe.snapshot.raw, Some(31));
                    assert!(fixture_probe_eligible(&assigned.state.lock().unwrap(), id));
                    drop(assigned);
                    probe
                        .reply
                        .send(Err(io::Error::from_raw_os_error(32)))
                        .unwrap();
                    break;
                }
                thread::park_timeout(end.saturating_duration_since(Instant::now()));
            }
        });
        worker.thread.set(thread).unwrap();
        assert_eq!(
            started.recv_timeout(Duration::from_secs(5)).unwrap(),
            worker.thread.get().unwrap().thread().id()
        );
        assert!(worker.probe_ranges(request, action, end).is_err());
        {
            let mut state = worker.state.lock().unwrap();
            state.snapshot.phase = Phase::FailedRetained;
            state.snapshot.raw = Some(31);
            state.result = Some(Err(recovery_required()));
        }
        assert!(worker.probe_ranges(request, request, end).is_err());
        assert!(worker.probe_ranges(request, worker.action, end).is_err());
        assert_eq!(
            worker
                .probe_ranges(request, action, Instant::now())
                .unwrap_err()
                .kind(),
            io::ErrorKind::TimedOut
        );
        assert_eq!(
            worker
                .probe_ranges(request, action, end)
                .unwrap_err()
                .raw_os_error(),
            Some(32)
        );
        let thread = Arc::try_unwrap(worker).ok().unwrap().thread;
        Arc::try_unwrap(thread)
            .unwrap()
            .into_inner()
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn wl2_r7_same_start_queue_admits_only_one_known_unresumed_failure() {
        let worker = control(Instant::now());
        let same_worker = worker.clone();
        let (ready, started) = mpsc::sync_channel(1);
        let (received, request) = mpsc::sync_channel(1);
        let end = Instant::now() + Duration::from_secs(5);
        let thread = thread::spawn(move || {
            let id = thread::current().id();
            same_worker.state.lock().unwrap().thread = Some(id);
            ready.send(id).unwrap();
            while Instant::now() < end {
                let item = same_worker.state.lock().unwrap().recovery.take();
                if let Some(item) = item {
                    drop(same_worker);
                    received.send((id, item)).unwrap();
                    return;
                }
                thread::park_timeout(end.saturating_duration_since(Instant::now()));
            }
        });
        worker.thread.set(thread).unwrap();
        let id = started.recv_timeout(Duration::from_secs(5)).unwrap();
        let now = Instant::now();
        let window = || BeforeResumeRecoveryWindow::begin(now, end).unwrap();
        assert!(worker.queue_recovery(window()).is_err());
        {
            let mut s = worker.state.lock().unwrap();
            s.snapshot.phase = Phase::LostWorkerUnknown;
        }
        assert!(worker.queue_recovery(window()).is_err());
        {
            let mut s = worker.state.lock().unwrap();
            s.snapshot.phase = Phase::ExpiredRetained;
            s.snapshot.stage = Stage::Resume;
            s.snapshot.creator_resumed = None;
        }
        assert!(worker.queue_recovery(window()).is_err());
        {
            let mut s = worker.state.lock().unwrap();
            s.snapshot.stage = Stage::BeforeResume;
            s.snapshot.creator_resumed = Some(false);
            s.snapshot.raw = Some(33);
        }
        let recovery = worker.queue_recovery(window()).unwrap();
        let (consuming_id, queued) = request.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(consuming_id, id);
        assert!(worker.queue_recovery(window()).is_err());
        let (_, original) = worker.observe(end).unwrap();
        assert_eq!(original["phase"], "ExpiredRetained");
        assert_eq!(original["rawError"], 33);
        drop(recovery);
        drop(queued);
        let thread = Arc::try_unwrap(worker).ok().unwrap().thread;
        Arc::try_unwrap(thread)
            .ok()
            .unwrap()
            .into_inner()
            .unwrap()
            .join()
            .unwrap();
        println!("one recovery dispatched to original Start thread; Running/Unknown/uncertain Resume and replay refused; first raw33 preserved");
    }

    #[tokio::test]
    async fn wl2_r7_start_worker_waiter_drop_preserves_first_result_and_rejects_late_success() {
        let worker = control(Instant::now() + Duration::from_secs(5));
        {
            let mut waiting = std::pin::pin!(worker.wait());
            assert!(
                std::future::poll_fn(|cx| {
                    std::task::Poll::Ready(
                        std::future::Future::poll(waiting.as_mut(), cx).is_pending(),
                    )
                })
                .await
            );
        }
        let mut state = worker.state.lock().unwrap();
        state.result = Some(Err(fail("fixture-first-error", "original raw33")));
        worker.wake.notify_one();
        drop(state);
        assert!(matches!(
            worker.wait().await,
            Err(AppError::RuntimeManaged { code, .. }) if code == "fixture-first-error"
        ));

        let late = control(Instant::now());
        late.state.lock().unwrap().result = Some(Ok(LifecycleResult::Started));
        assert!(matches!(late.wait().await, Err(AppError::Timeout)));
    }
}
