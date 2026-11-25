//! MOD-03：状态周期轮询编排。
//!
//! 本模块只负责 FZ-08 的调度、失败退避、快照推送门控与窗口状态
//! 观察。来源细节留在 infrastructure，UI 只消费同一份事件快照。

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::modules::status::{EmissionGate, StatusCollector, StatusDimension, StatusSource};
use crate::types::runtime_status::StatusSnapshotDto;

/// 状态事件名；前端监听同名事件。
pub const STATUS_SNAPSHOT_CHANGED_EVENT: &str = "status-snapshot-changed";

/// 后端推送通道边界，隔离测试时替换；生产由 Tauri app handle 实现。
pub trait StatusSnapshotEmitter {
    fn emit(&self, snapshot: &StatusSnapshotDto);
}

/// 窗口状态提供边界；测试可注入状态变化，生产读取 Tauri webview 窗口。
pub trait BackgroundStateProvider {
    fn backgrounded(&self) -> bool;
}

/// FZ-08 周期调度器。运行与连接维度共用同一状态来源与快照。
pub struct StatusPollingService<S, E, B>
where
    S: StatusSource,
    E: StatusSnapshotEmitter,
    B: BackgroundStateProvider,
{
    collector: Arc<Mutex<StatusCollector<S>>>,
    emitter: E,
    background_state: B,
    gate: Mutex<EmissionGate>,
    runtime_at: Option<Instant>,
    connection_at: Option<Instant>,
}

impl<S, E, B> StatusPollingService<S, E, B>
where
    S: StatusSource,
    E: StatusSnapshotEmitter,
    B: BackgroundStateProvider,
{
    pub fn new(collector: Arc<Mutex<StatusCollector<S>>>, emitter: E, background_state: B) -> Self {
        Self {
            collector,
            emitter,
            background_state,
            gate: Mutex::new(EmissionGate::new()),
            runtime_at: None,
            connection_at: None,
        }
    }

    fn emit_snapshot(&mut self, snapshot: StatusSnapshotDto) {
        let allowed = self
            .gate
            .lock()
            .map(|mut gate| gate.should_emit(Instant::now()))
            .unwrap_or(false);
        if allowed {
            self.emitter.emit(&snapshot);
        }
    }

    /// 到达任一维度刷新点时刷新一次并推送当前快照。
    pub async fn poll_once(&mut self) {
        let now = Instant::now();
        let backgrounded = self.background_state.backgrounded();
        // A manual sampler can be awaiting timers/IO on this runtime while
        // owning the collector. Blocking here can starve that very runtime.
        // A busy tick is neither a completed sample nor a successful event.
        let Ok(mut collector) = self.collector.try_lock() else {
            return;
        };
        let runtime_interval = collector.next_interval(StatusDimension::Runtime, backgrounded);
        let connection_interval =
            collector.next_interval(StatusDimension::Connection, backgrounded);
        let runtime_due = self
            .runtime_at
            .is_none_or(|at| now.duration_since(at) >= runtime_interval);
        let connection_due = self
            .connection_at
            .is_none_or(|at| now.duration_since(at) >= connection_interval);

        if runtime_due || connection_due {
            let _ = collector.refresh_light_at(now);
            let snapshot = collector.snapshot();
            // Emit only this sample's projection. Never acquire
            // the collector a second time on the asynchronous executor.
            drop(collector);
            self.emit_snapshot(snapshot);
            if runtime_due {
                self.runtime_at = Some(now);
            }
            if connection_due {
                self.connection_at = Some(now);
            }
        }
    }

    pub fn invalidate_schedule(&mut self) {
        self.runtime_at = None;
        self.connection_at = None;
    }

    pub fn next_wait(&self) -> Duration {
        let now = Instant::now();
        let backgrounded = self.background_state.backgrounded();
        let Ok(collector) = self.collector.try_lock() else {
            // Preserve original due times; this is a scheduling retry, not a
            // new status window or permission inferred from the old snapshot.
            return crate::modules::status::refresh_debounce();
        };
        let wait = |at: Option<Instant>, dimension| {
            at.map(|at| {
                collector
                    .next_interval(dimension, backgrounded)
                    .saturating_sub(now.duration_since(at))
            })
            .unwrap_or(Duration::ZERO)
        };
        wait(self.runtime_at, StatusDimension::Runtime)
            .min(wait(self.connection_at, StatusDimension::Connection))
    }

    /// Wait for the actual deadline instead of waking on a fixed foreground ticker.
    pub async fn run_until(&mut self, mut should_stop: impl FnMut() -> bool) {
        loop {
            self.poll_once().await;
            if should_stop() {
                break;
            }
            tokio::time::sleep(self.next_wait()).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::status::RefreshPolicy;
    use serde_json::json;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[derive(Debug)]
    struct SequenceSource {
        calls: AtomicU32,
    }

    impl SequenceSource {
        fn new() -> Self {
            Self {
                calls: AtomicU32::new(0),
            }
        }
    }

    impl StatusSource for SequenceSource {
        fn fetch(&self) -> Result<serde_json::Value, crate::modules::status::CollectError> {
            let call = self.calls.fetch_add(1, Ordering::SeqCst);
            if call == 0 {
                Ok(json!({
                    "status": "running",
                    "ready": true,
                    "connection": "synced",
                    "dataRoot": "/tmp/fixture",
                    "startup": {"protection": "none", "rebootSafe": false}
                }))
            } else {
                Err(crate::modules::status::CollectError::Unreachable)
            }
        }
    }

    #[derive(Debug, Default)]
    struct RecordingEmitter {
        snapshots: Mutex<Vec<StatusSnapshotDto>>,
    }

    impl StatusSnapshotEmitter for Arc<RecordingEmitter> {
        fn emit(&self, snapshot: &StatusSnapshotDto) {
            self.snapshots
                .lock()
                .expect("emit lock")
                .push(snapshot.clone());
        }
    }

    struct ForegroundFixture;

    impl BackgroundStateProvider for ForegroundFixture {
        fn backgrounded(&self) -> bool {
            false
        }
    }

    type TestService =
        StatusPollingService<SequenceSource, Arc<RecordingEmitter>, ForegroundFixture>;

    fn service() -> (
        TestService,
        Arc<Mutex<StatusCollector<SequenceSource>>>,
        Arc<RecordingEmitter>,
    ) {
        let collector = Arc::new(Mutex::new(StatusCollector::new(SequenceSource::new())));
        let emitter = Arc::new(RecordingEmitter::default());
        let service =
            StatusPollingService::new(collector.clone(), emitter.clone(), ForegroundFixture);
        (service, collector, emitter)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn busy_tick_preserves_schedule_source_and_emissions_then_retries_original_due_sample() {
        let (mut service, collector, emitter) = service();
        let other = collector.clone();
        let (entered, entry) = std::sync::mpsc::channel();
        let (release, released) = std::sync::mpsc::channel();
        let holder = std::thread::spawn(move || {
            let held = other.lock().unwrap();
            entered.send(()).unwrap();
            released.recv().unwrap();
            assert_eq!(held.source.calls.load(Ordering::SeqCst), 0);
            assert_eq!(held.failure_count(), 0);
        });
        entry.recv().unwrap();
        let before = Instant::now();
        service.poll_once().await;
        assert!(before.elapsed() < Duration::from_millis(100));
        assert!(service.runtime_at.is_none() && service.connection_at.is_none());
        assert!(emitter.snapshots.lock().unwrap().is_empty());
        assert_eq!(
            service.next_wait(),
            crate::modules::status::refresh_debounce()
        );
        release.send(()).unwrap();
        holder.join().unwrap();
        service.poll_once().await;
        assert_eq!(
            collector
                .lock()
                .unwrap()
                .source
                .calls
                .load(Ordering::SeqCst),
            1
        );
        assert!(service.runtime_at.is_some() && service.connection_at.is_some());
        assert_eq!(emitter.snapshots.lock().unwrap().len(), 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn emitted_sample_has_released_collector_and_keeps_the_same_projection() {
        struct ReentrantEmitter {
            collector: Arc<Mutex<StatusCollector<SequenceSource>>>,
            called: Arc<AtomicU32>,
        }
        impl StatusSnapshotEmitter for ReentrantEmitter {
            fn emit(&self, snapshot: &StatusSnapshotDto) {
                let collector = self
                    .collector
                    .try_lock()
                    .expect("event emission must never own the sampler lock");
                assert_eq!(collector.matrix().runtime, snapshot.matrix.runtime);
                self.called.fetch_add(1, Ordering::SeqCst);
            }
        }
        let collector = Arc::new(Mutex::new(StatusCollector::new(SequenceSource::new())));
        let called = Arc::new(AtomicU32::new(0));
        let mut service = StatusPollingService::new(
            collector.clone(),
            ReentrantEmitter {
                collector,
                called: called.clone(),
            },
            ForegroundFixture,
        );
        service.poll_once().await;
        assert_eq!(called.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn refreshes_once_within_debounce_and_gates_emissions() {
        let (mut service, collector, emitter) = service();
        service.poll_once().await;
        assert_eq!(
            emitter.snapshots.lock().expect("snapshots").len(),
            1,
            "first refresh emits one snapshot"
        );
        assert_eq!(
            emitter.snapshots.lock().expect("snapshots")[0]
                .matrix
                .runtime,
            crate::types::status::RuntimeState::Running
        );
        service.poll_once().await;
        assert_eq!(
            emitter.snapshots.lock().expect("snapshots").len(),
            1,
            "emission gate stays within two snapshots per second"
        );
        assert_eq!(
            collector
                .lock()
                .expect("collector")
                .source
                .calls
                .load(Ordering::SeqCst),
            1,
            "same-dimension refresh is debounced for one second"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn background_interval_uses_frozen_thirty_seconds() {
        assert_eq!(
            RefreshPolicy::next_interval(StatusDimension::Connection, 0, true),
            crate::modules::status::background_connection_interval()
        );
    }
    struct BackgroundFixture;
    impl BackgroundStateProvider for BackgroundFixture {
        fn backgrounded(&self) -> bool {
            true
        }
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn deadline_wait_and_failure_backoff_are_applied_once() {
        let collector = Arc::new(Mutex::new(StatusCollector::new(SequenceSource::new())));
        let emitter = Arc::new(RecordingEmitter::default());
        let mut service =
            StatusPollingService::new(collector.clone(), emitter.clone(), BackgroundFixture);
        service.poll_once().await;
        assert!(
            service.next_wait() > Duration::from_secs(9),
            "no foreground ticker while backgrounded"
        );
        service.runtime_at = Some(Instant::now() - Duration::from_secs(100));
        collector.lock().unwrap().last_request = None;
        service.poll_once().await;
        assert_eq!(collector.lock().unwrap().failure_count(), 1);
        {
            let snapshots = emitter.snapshots.lock().unwrap();
            assert_eq!(snapshots.len(), 2);
            assert!(snapshots[0].can_stop);
            let failure = serde_json::to_value(&snapshots[1]).unwrap();
            assert_eq!(failure["matrix"]["runtime"], "unreachable");
            assert_eq!(failure["facts"]["health"], "unknown");
            assert_eq!(failure["status_failure"], "unreachable");
            assert!(!snapshots[1].can_start && !snapshots[1].can_stop && !snapshots[1].can_restart);
        }
        assert!(
            service.next_wait() > Duration::from_secs(19),
            "single failure doubles interval once"
        );
        service.invalidate_schedule();
        assert_eq!(service.next_wait(), Duration::ZERO);
    }
}
