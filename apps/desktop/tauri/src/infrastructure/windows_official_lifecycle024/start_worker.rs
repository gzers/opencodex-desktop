use super::*;
use crate::infrastructure::windows_client_originals024::StartRanges;
use std::{
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
}

pub(super) struct StartWorker {
    pub(super) session: uuid::Uuid,
    pub(super) scope: String,
    action: uuid::Uuid,
    root: native::ProcessIdentity,
    deadline: Instant,
    state: Mutex<State>,
    wake: Notify,
    thread: OnceLock<thread::JoinHandle<()>>,
}

impl StartWorker {
    pub(super) fn dispatch(
        entry: &Arc<Slot>,
        slot: &mut Option<Owner>,
        registered: &mut Option<Arc<Self>>,
        plan: shared::LaunchPlan,
        window: shared::ActionWindow,
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
            }),
            wake: Notify::new(),
            thread: OnceLock::new(),
        });
        let (send, receive) = mpsc::sync_channel::<Owner>(1);
        let worker_entry = entry.clone();
        let worker_control = control.clone();
        let worker = thread::Builder::new()
            .name("windows024-original-start".into())
            .spawn(move || {
                if let Ok(owner) = receive.recv() {
                    worker_control.run(worker_entry, owner, plan, window);
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
        if self
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
    ) {
        let mut owner = Some(owner);
        let mut ranges = StartRanges::retained();
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
            tauri::async_runtime::block_on(self.start(
                &entry,
                original,
                plan,
                window,
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
        let result = Err(error);
        let resumed = owner
            .as_ref()
            .map(|owner| owner.native.creator_has_resumed());
        if let Some(owner) = &mut owner {
            owner.remember_start_failure(&result);
        }
        if let Ok(mut state) = self.state.lock() {
            state.snapshot.phase = phase;
            state.snapshot.raw = raw.or(ranges.raw_error());
            state.snapshot.launcher_exit = launcher_exit;
            state.snapshot.creator_resumed = resumed;
            state.result = Some(result);
        }
        self.wake.notify_one();
        loop {
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
        ranges: &mut StartRanges,
        raw: &mut Option<i32>,
        launcher_exit: &mut Option<u32>,
    ) -> AppResult<()> {
        window.remaining()?;
        check_cancel(entry)?;
        self.stage(Stage::Capture);
        owner.capture_originals(&window, ranges)?;
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
            }),
            wake: Notify::new(),
            thread: OnceLock::new(),
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
