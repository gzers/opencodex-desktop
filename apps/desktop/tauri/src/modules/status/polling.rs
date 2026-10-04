//! MOD-03：状态周期轮询编排。
//!
//! 本模块只负责 FZ-08 的调度、失败退避、快照推送门控与窗口状态
//! 观察。来源细节留在 infrastructure，UI 只消费同一份事件快照。

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::modules::status::{EmissionGate, StatusCollector, StatusDimension, StatusSource};
use crate::types::runtime_status::{
    snapshot_from_matrix, StatusSnapshotDto, StatusSource as DtoStatusSource,
};

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

    fn snapshot(&self, collector: &StatusCollector<S>) -> StatusSnapshotDto {
        snapshot_from_matrix(
            collector.matrix(),
            collector.facts(),
            collector.port(),
            collector.pid(),
            DtoStatusSource::Live,
        )
    }

    fn current_snapshot(&self) -> StatusSnapshotDto {
        let guard = self.collector.lock().expect("status collector lock");
        self.snapshot(&guard)
    }

    fn emit_current(&mut self) {
        let snapshot = self.current_snapshot();
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
        let (runtime_interval, connection_interval) = {
            let guard = self.collector.lock().expect("status collector lock");
            (
                guard.next_interval(StatusDimension::Runtime, backgrounded),
                guard.next_interval(StatusDimension::Connection, backgrounded),
            )
        };
        let runtime_due = self
            .runtime_at
            .is_none_or(|at| now.duration_since(at) >= runtime_interval);
        let connection_due = self
            .connection_at
            .is_none_or(|at| now.duration_since(at) >= connection_interval);

        if runtime_due || connection_due {
            let refresh_result = self
                .collector
                .lock()
                .map(|mut collector| collector.refresh_at(now))
                .unwrap_or(Err(crate::modules::status::CollectError::Unreachable));
            if refresh_result.is_err() {
                if let Ok(mut collector) = self.collector.lock() {
                    collector.record_failure();
                }
            }
            self.emit_current();
            if runtime_due {
                self.runtime_at = Some(now);
            }
            if connection_due {
                self.connection_at = Some(now);
            }
        }
    }

    /// 执行 FZ-08 周期循环；调用方负责运行时与停机信号。
    pub async fn run_until(&mut self, mut should_stop: impl FnMut() -> bool) {
        loop {
            self.poll_once().await;
            let now = Instant::now();
            let backgrounded = self.background_state.backgrounded();
            let runtime_interval = {
                let guard = self.collector.lock().expect("status collector lock");
                guard.next_interval(StatusDimension::Runtime, backgrounded)
            };
            let connection_interval = {
                let guard = self.collector.lock().expect("status collector lock");
                guard.next_interval(StatusDimension::Connection, backgrounded)
            };
            let runtime_wait = self
                .runtime_at
                .map(|at| {
                    runtime_interval
                        .checked_sub(now.duration_since(at))
                        .unwrap_or(Duration::ZERO)
                })
                .unwrap_or(Duration::ZERO);
            let connection_wait = self
                .connection_at
                .map(|at| {
                    connection_interval
                        .checked_sub(now.duration_since(at))
                        .unwrap_or(Duration::ZERO)
                })
                .unwrap_or(Duration::ZERO);
            let wait = runtime_wait
                .min(connection_wait)
                .max(Duration::from_millis(20));
            tokio::time::sleep(wait).await;
            if should_stop() {
                break;
            }
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
}
