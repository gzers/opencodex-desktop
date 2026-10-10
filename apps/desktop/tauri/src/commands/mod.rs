//! Tauri 命令层。命令只做参数转换与模块编排，不直接访问平台 API。

pub mod about;
pub mod backup;
pub mod cleanup;
pub mod codex_shim;
pub mod data_root;
pub mod discovery;
pub mod doctor;
pub(crate) mod event_delivery;
pub mod extensions;
pub mod logs;
pub mod migration;
pub mod network;
pub mod notifications;
pub mod panel;
pub mod preferences;
pub mod runtime;
pub mod sync;
pub mod tray;
pub mod update;
pub mod update_schedule;
pub mod upgrade;
pub mod window;
pub mod workspace;

use crate::errors::{AppError, AppResult};
use crate::modules::notifications::Notification;
#[cfg(test)]
use crate::modules::notifications::{NotificationAction, NotificationCategory, NotificationLevel};
use crate::modules::process::{LifecycleAction, ProcessLifecycleState, ProcessRunner};
use crate::modules::status::{StatusCollector, StatusSource as ModuleStatusSource};
use std::sync::Mutex;

use crate::state::{
    ProcessContext, SharedProcessContext, SharedProcessRunner, SharedStatusCollector,
};
use crate::types::process_action::{
    validate_action_request, ProcessActionRequest, ProcessActionResult,
};
use crate::types::runtime_status::{snapshot_from_matrix, StatusSource};
use crate::types::AppStatus;

/// 把阻塞型工作移出 Tauri 的 IPC/主线程，避免耗时命令冻结界面。
///
/// Tauri 的**同步**命令在 IPC 回调线程内联执行（macOS 上即主线程），
/// 因此凡涉及 Argon2 派生、子进程、批量文件读写或目录扫描的命令都应改为
/// `async` 并经本函数在阻塞线程池执行。
pub(crate) async fn run_blocking<T, F>(operation: &'static str, task: F) -> AppResult<T>
where
    F: FnOnce() -> AppResult<T> + Send + 'static,
    T: Send + 'static,
{
    run_blocking_with_gate(
        crate::infrastructure::storage_writers::global(),
        operation,
        task,
    )
    .await
}

pub(crate) async fn run_blocking_with_gate<T, F>(
    gate: std::sync::Arc<crate::infrastructure::storage_writers::WriterGate>,
    operation: &'static str,
    task: F,
) -> AppResult<T>
where
    F: FnOnce() -> AppResult<T> + Send + 'static,
    T: Send + 'static,
{
    // Admit before dispatch; the actual worker owns it, not the IPC observer.
    let admission = gate.admit()?;
    run_readonly(operation, move || {
        let _admission = admission;
        task()
    })
    .await
}

/// Read-only work and the binding-save transaction must not count as writers.
pub(crate) async fn run_readonly<T, F>(operation: &'static str, task: F) -> AppResult<T>
where
    F: FnOnce() -> AppResult<T> + Send + 'static,
    T: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|error| AppError::FileSystem {
            operation: operation.to_string(),
            detail: error.to_string(),
        })?
}

#[tauri::command]
pub fn app_status() -> AppResult<AppStatus> {
    Ok(AppStatus {
        mode: crate::types::AppMode::Shell,
        ready: true,
    })
}

#[tauri::command]
pub fn process_action(
    request: ProcessActionRequest,
    runner: tauri::State<'_, SharedProcessRunner>,
    context: tauri::State<'_, SharedProcessContext>,
    data_root: tauri::State<'_, crate::state::SharedDataRoot>,
    notifications: tauri::State<'_, crate::state::SharedNotificationStore>,
    app: tauri::AppHandle,
) -> AppResult<ProcessActionResult> {
    use tauri::Manager;
    let _storage = crate::infrastructure::storage_writers::global().admit()?;
    let mutation = app.state::<crate::state::SharedRuntimeInstall>();
    let _lease = mutation.acquire().ok_or(AppError::NotConfigured)?;
    let runtime_log = crate::infrastructure::runtime_log::RuntimeLog::new(&data_root.0);
    // 偏好只控制新增失败通知；真实成功仍需解除已有同 scope 失败。
    let publisher = Some(notifications::NotificationPublisher {
        store: notifications.inner(),
        data_root: &data_root.0,
    });
    let before = notifications.lock().ok().map(|store| store.clone());
    let result =
        process_action_with_runner(request, runner.inner(), &context, &runtime_log, publisher);
    let changed =
        before.is_some_and(|before| notifications.lock().is_ok_and(|store| *store != before));
    if changed {
        let _ = crate::commands::event_delivery::emit_signal(
            &app,
            notifications::NOTIFICATIONS_CHANGED_EVENT,
            crate::modules::notifications::registry::Job::NotificationMutation,
            crate::modules::notifications::registry::Trigger::Commit,
            crate::modules::notifications::registry::Channel::Local,
            (),
        );
    }
    result
}

/// 只有开启「启停结果通知」时才构造通知发布器；否则返回 `None`（不写任何持久通知）。
pub fn publisher_if_enabled<'a>(
    enabled: bool,
    store: &'a crate::state::SharedNotificationStore,
    data_root: &'a std::path::Path,
) -> Option<crate::commands::notifications::NotificationPublisher<'a>> {
    if !enabled {
        return None;
    }
    Some(crate::commands::notifications::NotificationPublisher { store, data_root })
}

#[tauri::command]
pub fn drain_tray_requests(
    tray_requests: tauri::State<'_, crate::state::SharedTrayRequests>,
) -> AppResult<Vec<crate::modules::tray::TrayAction>> {
    Ok(tray_requests.drain())
}

pub fn process_action_with_runner<R>(
    request: ProcessActionRequest,
    runner: &Mutex<R>,
    context: &ProcessContext,
    runtime_log: &crate::infrastructure::runtime_log::RuntimeLog,
    notifications: Option<crate::commands::notifications::NotificationPublisher<'_>>,
) -> AppResult<ProcessActionResult>
where
    R: ProcessRunner + ?Sized,
{
    use std::ops::DerefMut;

    use crate::modules::process::{LifecycleStateMachine, ProcessCommand};

    validate_action_request(&request).map_err(|_reason| AppError::NotConfigured)?;
    let mut guard = runner.lock().map_err(|_poisoned| AppError::NotConfigured)?;
    let runner = guard.deref_mut();
    let mut machine = LifecycleStateMachine::new(match request.action {
        LifecycleAction::Start => ProcessLifecycleState::Stopped,
        LifecycleAction::Stop | LifecycleAction::Restart => ProcessLifecycleState::Running,
    });
    let before = machine.current();
    let next = machine.advance(request.action)?;
    let command = ProcessCommand::new(
        request.action,
        context.executable()?,
        context.working_directory.clone(),
        context.opencodex_home.clone(),
    );
    let scope = notifications.and_then(|publisher| {
        crate::modules::notifications::registry::lifecycle_identity(
            publisher.data_root,
            &command.executable,
            &context.working_directory,
            &context.opencodex_home,
            registry_action(request.action),
        )
        .map_err(|error| {
            let _ =
                runtime_log.append_result(action_log_label(request.action), Err(error.to_string()));
        })
        .ok()
    });
    let started_at = std::time::Instant::now();
    let outcome = runner.execute(&command);
    let duration_ms = started_at.elapsed().as_millis();
    let failed;
    let result = match &outcome {
        Ok(result) => {
            let status = match result {
                crate::modules::process::LifecycleResult::Started => match request.action {
                    LifecycleAction::Restart => "restart confirmed",
                    _ => "start confirmed",
                },
                crate::modules::process::LifecycleResult::Stopped => "stop completed",
                crate::modules::process::LifecycleResult::Cancelled => "cancelled",
                crate::modules::process::LifecycleResult::Failed => "failed",
            };
            if matches!(
                result,
                crate::modules::process::LifecycleResult::Cancelled
                    | crate::modules::process::LifecycleResult::Failed
            ) {
                let _ = runtime_log.append_result(
                    action_log_label(request.action),
                    Err(format!("{status}; duration_ms={duration_ms}")),
                );
            } else {
                let _ = runtime_log.append_result(
                    action_log_label(request.action),
                    Ok(match result {
                        crate::modules::process::LifecycleResult::Started => match request.action {
                            LifecycleAction::Restart => "restart confirmed",
                            _ => "start confirmed",
                        },
                        _ => "stop completed",
                    }),
                );
            }
            failed = matches!(result, crate::modules::process::LifecycleResult::Failed);
            Ok(ProcessActionResult::from_state(next).with_result(result.clone()))
        }
        Err(error) => {
            failed = true;
            machine.rollback_to_observation(before);
            let _ = runtime_log.append_result(
                action_log_label(request.action),
                Err(format!("{}; duration_ms={duration_ms}", error)),
            );
            Ok(ProcessActionResult::from_state(machine.current()))
        }
    };
    let verified_success = matches!(
        (&request.action, &outcome),
        (
            LifecycleAction::Start | LifecycleAction::Restart,
            Ok(crate::modules::process::LifecycleResult::Started)
        ) | (
            LifecycleAction::Stop,
            Ok(crate::modules::process::LifecycleResult::Stopped)
        )
    );
    // 候选文件/运行上下文在执行中变化时，不把旧候选标成成功。
    let verified_success = verified_success
        && scope.as_ref().is_some_and(|before| {
            notifications.is_some_and(|publisher| {
                crate::modules::notifications::registry::lifecycle_identity(
                    publisher.data_root,
                    &command.executable,
                    &context.working_directory,
                    &context.opencodex_home,
                    registry_action(request.action),
                )
                .is_ok_and(|after| after == *before)
            })
        });
    if failed || verified_success {
        if let (Some(publisher), Some(identity)) = (notifications, scope) {
            use crate::modules::notifications::registry::{Delivery, Evidence, Trigger};
            let event = match (request.action, failed) {
                (LifecycleAction::Start, true) => "run-start-failed",
                (LifecycleAction::Start, false) => "run-start-succeeded",
                (LifecycleAction::Stop, true) => "run-stop-failed",
                (LifecycleAction::Stop, false) => "run-stop-succeeded",
                (LifecycleAction::Restart, true) => "run-restart-failed",
                (LifecycleAction::Restart, false) => "run-restart-succeeded",
            };
            let evidence = if failed {
                Evidence::Failure
            } else {
                Evidence::Success {
                    candidate: identity.candidate.clone(),
                    verified: true,
                }
            };
            let delivery = Delivery {
                event,
                job: registry_job(request.action),
                trigger: Trigger::User,
                identity,
                evidence,
                occurred_at: chrono::Utc::now(),
            };
            if let Err(error) = publisher.publish_event(&delivery) {
                let _ = runtime_log.append_result(
                    action_log_label(request.action),
                    Err(format!("notification publish failed: {error}")),
                );
            }
        }
    }
    result
}

/// 生命周期动作真实失败 → 一条 `danger` 操作性通知（FZ-43、`领域模型` §16.1）。
///
/// 「启停结果通知」偏好；读不到时按默认（开启）处理。
///
/// 该偏好此前只是被保存，没有任何消费方——关闭它之后失败通知照样产生。
pub fn lifecycle_notifications_enabled(data_root: &std::path::Path) -> bool {
    crate::modules::preferences::PreferencesStore::new(data_root)
        .load()
        .map(|preferences| preferences.lifecycle_notifications)
        .unwrap_or(true)
}

fn registry_action(action: LifecycleAction) -> crate::modules::notifications::registry::Action {
    use crate::modules::notifications::registry::Action;
    match action {
        LifecycleAction::Start => Action::Start,
        LifecycleAction::Stop => Action::Stop,
        LifecycleAction::Restart => Action::Restart,
    }
}
fn registry_job(action: LifecycleAction) -> crate::modules::notifications::registry::Job {
    use crate::modules::notifications::registry::Job;
    match action {
        LifecycleAction::Start => Job::Start,
        LifecycleAction::Stop => Job::Stop,
        LifecycleAction::Restart => Job::Restart,
    }
}

fn action_log_label(action: LifecycleAction) -> &'static str {
    match action {
        LifecycleAction::Start => "start",
        LifecycleAction::Stop => "stop",
        LifecycleAction::Restart => "restart",
    }
}

/// 该状态讲的是「启停命令的结果」，受「启停结果通知」偏好约束。
///
/// `not_found` / `at_risk` / `external_takeover` 讲的是环境与配置所有权，
/// 不是某次启停的结果，因此不受该偏好影响，始终写通知。
fn is_lifecycle_outcome(runtime: crate::types::status::RuntimeState) -> bool {
    use crate::types::status::RuntimeState;
    matches!(
        runtime,
        RuntimeState::StartingFailed | RuntimeState::Unreachable
    )
}

/// 需要关注的运行状态 → 一条持久通知（《UI规范》§19）。
///
/// 只覆盖「应用内必须被看见、且不是瞬时进度」的状态；稳定事实（`stopped` / `running`）
/// 由前端播报**一次** Toast，瞬时进度（`loading` / `starting` / `pending` / `stopping`）不写。
/// 同一问题用固定 `dedupe_key`，重复出现只更新那一条（并恢复为未读未解决）。
///
/// 观测仍是无 scope 的旧入口；仅同类观测去重，不能解除或覆盖命令 scoped 失败。
pub fn runtime_state_notification(
    runtime: crate::types::status::RuntimeState,
) -> Option<Notification> {
    use crate::types::status::RuntimeState;
    let id = match runtime {
        RuntimeState::NotFound => "runtime-not-found",
        RuntimeState::StartingFailed => "runtime-starting-failed",
        RuntimeState::Unreachable => "run-unreachable",
        RuntimeState::AtRisk => "run-at-risk",
        RuntimeState::ExternalTakeover => "external-takeover",
        _ => return None,
    };
    crate::modules::notifications::registry::registered_notification(id, chrono::Utc::now()).ok()
}

/// 状态轮询观测到关注态时发布通知；发布失败只记运行日志，不影响轮询主流程。
///
/// 「应用刚启动的首次观测」与「状态未变化」都由调用方过滤，这里只做发布。
pub fn publish_runtime_state_notification(
    publisher: Option<crate::commands::notifications::NotificationPublisher<'_>>,
    runtime: crate::types::status::RuntimeState,
    lifecycle_notifications: bool,
    runtime_log: &crate::infrastructure::runtime_log::RuntimeLog,
) -> bool {
    publish_runtime_state_notification_with_context(
        publisher,
        runtime,
        lifecycle_notifications,
        runtime_log,
        None,
    )
}

/// 状态轮询的通知发布入口。
///
/// 观测状态和启停命令拥有不同的作用域：观测告警只能由同一运行上下文的
/// `runtime-observation-recovered` 解除，不能借 `Running` 状态解除某次 start/stop
/// 失败。无法取得完整 ProcessContext 时保留 legacy 记录，但不伪造候选身份。
pub fn publish_runtime_state_notification_with_context(
    publisher: Option<crate::commands::notifications::NotificationPublisher<'_>>,
    runtime: crate::types::status::RuntimeState,
    lifecycle_notifications: bool,
    runtime_log: &crate::infrastructure::runtime_log::RuntimeLog,
    context: Option<&crate::state::ProcessContext>,
) -> bool {
    if is_lifecycle_outcome(runtime) && !lifecycle_notifications {
        return false;
    }
    let Some(publisher) = publisher else {
        return false;
    };

    let scoped_event = match runtime {
        crate::types::status::RuntimeState::StartingFailed => Some((
            "runtime-starting-failed",
            crate::modules::notifications::registry::Fact::Failure,
        )),
        crate::types::status::RuntimeState::Unreachable => Some((
            "run-unreachable",
            crate::modules::notifications::registry::Fact::Failure,
        )),
        crate::types::status::RuntimeState::AtRisk => Some((
            "run-at-risk",
            crate::modules::notifications::registry::Fact::Risk,
        )),
        crate::types::status::RuntimeState::ExternalTakeover => Some((
            "external-takeover",
            crate::modules::notifications::registry::Fact::Risk,
        )),
        crate::types::status::RuntimeState::Running => Some((
            "runtime-observation-recovered",
            crate::modules::notifications::registry::Fact::Success,
        )),
        _ => None,
    };

    if let (Some((event, fact)), Some(context)) = (scoped_event, context) {
        if let Ok(executable) = context.executable() {
            if let Ok(identity) = crate::modules::notifications::registry::observation_identity(
                publisher.data_root,
                &executable,
                &context.working_directory,
                &context.opencodex_home,
            ) {
                let evidence = match fact {
                    crate::modules::notifications::registry::Fact::Failure => {
                        crate::modules::notifications::registry::Evidence::Failure
                    }
                    crate::modules::notifications::registry::Fact::Risk => {
                        crate::modules::notifications::registry::Evidence::Risk
                    }
                    crate::modules::notifications::registry::Fact::Success => {
                        crate::modules::notifications::registry::Evidence::Success {
                            candidate: identity.candidate.clone(),
                            verified: true,
                        }
                    }
                    _ => return false,
                };
                let delivery = crate::modules::notifications::registry::Delivery {
                    event,
                    job: crate::modules::notifications::registry::Job::Observe,
                    trigger: crate::modules::notifications::registry::Trigger::StatusChange,
                    identity,
                    evidence,
                    occurred_at: chrono::Utc::now(),
                };
                return match publisher.publish_event(&delivery) {
                    Ok(outcome) => outcome.changed,
                    Err(error) => {
                        let _ = runtime_log.append_result(
                            "runtime-state-notification",
                            Err(format!("notification publish failed: {error}")),
                        );
                        false
                    }
                };
            }
        }
        let _ = runtime_log.append_result(
            "runtime-state-notification",
            Err("observation scope unavailable; using legacy notification".to_owned()),
        );
    }

    // NotFound intentionally remains legacy: a missing executable has no reliable
    // runtime candidate to bind to, and Running must not clear it accidentally.
    let Some(notification) = runtime_state_notification(runtime) else {
        return false;
    };
    match publisher.publish_legacy_changed(notification) {
        Ok(changed) => changed,
        Err(error) => {
            let _ = runtime_log.append_result(
                "runtime-state-notification",
                Err(format!("notification publish failed: {error}")),
            );
            false
        }
    }
}

pub fn status_snapshot_from_collector<S>(
    collector: &mut StatusCollector<S>,
) -> AppResult<crate::types::runtime_status::StatusSnapshotDto>
where
    S: ModuleStatusSource,
{
    // 采集失败时 collector 保留上一份状态；本层只投影，不发明状态。
    let _ = collector.refresh();
    Ok(snapshot_from_matrix(
        collector.matrix(),
        collector.facts(),
        collector.port(),
        collector.pid(),
        StatusSource::Live,
    ))
}

#[tauri::command]
pub async fn get_status_snapshot(
    collector: tauri::State<'_, SharedStatusCollector>,
) -> AppResult<crate::types::runtime_status::StatusSnapshotDto> {
    let collector = collector.inner().clone();
    tauri::async_runtime::spawn_blocking(move || get_status_snapshot_with_collector(&collector))
        .await
        .map_err(|_| AppError::NotConfigured)?
}

pub fn get_status_snapshot_with_collector(
    collector: &SharedStatusCollector,
) -> AppResult<crate::types::runtime_status::StatusSnapshotDto> {
    let mut collector = collector.lock().map_err(|_| AppError::NotConfigured)?;
    status_snapshot_from_collector(&mut collector)
}

#[tauri::command]
pub fn restore_guidance(
    collector: tauri::State<'_, SharedStatusCollector>,
) -> AppResult<crate::types::runtime_status::RestoreGuidanceDto> {
    let collector = collector
        .lock()
        .map_err(|_poisoned| AppError::NotConfigured)?;
    Ok(
        crate::types::runtime_status::RestoreGuidanceDto::from_snapshot(&snapshot_from_matrix(
            collector.matrix(),
            collector.facts(),
            collector.port(),
            collector.pid(),
            StatusSource::Live,
        )),
    )
}
pub mod skills;

#[cfg(test)]
mod runtime_state_notification_tests {
    use super::*;
    use crate::types::status::RuntimeState;

    /// 稳定事实与瞬时进度都不写通知：前者走前端一次 Toast，后者由状态点与按钮表达。
    #[test]
    fn stable_and_transitional_states_do_not_notify() {
        for runtime in [
            RuntimeState::Loading,
            RuntimeState::Starting,
            RuntimeState::Pending,
            RuntimeState::Stopping,
            RuntimeState::Stopped,
            RuntimeState::Running,
        ] {
            assert!(
                runtime_state_notification(runtime).is_none(),
                "{runtime:?} 不应写持久通知"
            );
        }
    }

    /// 需要关注的状态各写一条，级别 / 分类 / 动作与《UI规范》§19 一致，且默认未读未解决。
    #[test]
    fn attention_states_carry_level_category_and_action() {
        let cases = [
            (
                RuntimeState::NotFound,
                NotificationLevel::Warning,
                NotificationCategory::System,
                NotificationAction::SettingsInstallation,
            ),
            (
                RuntimeState::StartingFailed,
                NotificationLevel::Danger,
                NotificationCategory::Run,
                NotificationAction::Logs,
            ),
            (
                RuntimeState::Unreachable,
                NotificationLevel::Danger,
                NotificationCategory::Run,
                NotificationAction::Logs,
            ),
            (
                RuntimeState::AtRisk,
                NotificationLevel::Warning,
                NotificationCategory::Run,
                NotificationAction::Restore,
            ),
            (
                RuntimeState::ExternalTakeover,
                NotificationLevel::Warning,
                NotificationCategory::Run,
                NotificationAction::Restore,
            ),
        ];
        for (runtime, level, category, action) in cases {
            let notification = runtime_state_notification(runtime).expect("关注态必须产生通知");
            assert_eq!(notification.level, level, "{runtime:?} 级别");
            assert_eq!(notification.category, category, "{runtime:?} 分类");
            assert_eq!(notification.action_ref, Some(action), "{runtime:?} 动作");
            assert!(
                notification.dedupe_key.is_some(),
                "{runtime:?} 必须带去重键"
            );
            assert!(
                !notification.read && !notification.resolved,
                "{runtime:?} 必须是未读且未解决"
            );
        }
    }

    /// 「启停结果通知」只约束启停结果类；环境与配置所有权类不受影响。
    #[test]
    fn lifecycle_preference_only_gates_lifecycle_outcomes() {
        assert!(is_lifecycle_outcome(RuntimeState::StartingFailed));
        assert!(is_lifecycle_outcome(RuntimeState::Unreachable));
        assert!(!is_lifecycle_outcome(RuntimeState::AtRisk));
        assert!(!is_lifecycle_outcome(RuntimeState::ExternalTakeover));
        assert!(!is_lifecycle_outcome(RuntimeState::NotFound));
    }

    #[test]
    fn observed_start_failure_remains_legacy_without_recovery_scope() {
        let observed = runtime_state_notification(RuntimeState::StartingFailed).expect("通知");
        assert_eq!(observed.notification_id, "run-start-failed");
        assert_eq!(observed.dedupe_key.as_deref(), Some("run:start-failed"));
        assert!(observed.event_identity.is_none());
    }

    /// 关掉「启停结果通知」后启停结果类不再入通知中心，但风险类照旧写入。
    #[test]
    fn disabled_preference_skips_lifecycle_outcomes_only() {
        let root = tempfile::tempdir().expect("temp data root");
        std::fs::create_dir_all(root.path().join("manager-state")).expect("manager state dir");
        let store: crate::state::SharedNotificationStore = std::sync::Arc::new(
            std::sync::Mutex::new(crate::modules::notifications::NotificationStore::new()),
        );
        let log = crate::infrastructure::runtime_log::RuntimeLog::new(root.path());
        let publisher = crate::commands::notifications::NotificationPublisher {
            store: &store,
            data_root: root.path(),
        };
        publish_runtime_state_notification(
            Some(publisher),
            RuntimeState::StartingFailed,
            false,
            &log,
        );
        publish_runtime_state_notification(Some(publisher), RuntimeState::AtRisk, false, &log);
        let guard = store.lock().expect("notification store lock");
        let ids: Vec<&str> = guard
            .live()
            .iter()
            .map(|item| item.notification_id.as_str())
            .collect();
        assert!(
            !ids.contains(&"run-start-failed"),
            "关闭偏好后启停结果通知不应写入：{ids:?}"
        );
        assert!(
            ids.contains(&"run-at-risk"),
            "风险类通知不受偏好影响：{ids:?}"
        );
    }

    /// 同一问题重复出现只更新那一条，并恢复为未读。
    #[test]
    fn repeated_observation_updates_one_entry_and_resets_read() {
        let root = tempfile::tempdir().expect("temp data root");
        std::fs::create_dir_all(root.path().join("manager-state")).expect("manager state dir");
        let store: crate::state::SharedNotificationStore = std::sync::Arc::new(
            std::sync::Mutex::new(crate::modules::notifications::NotificationStore::new()),
        );
        let log = crate::infrastructure::runtime_log::RuntimeLog::new(root.path());
        let publisher = crate::commands::notifications::NotificationPublisher {
            store: &store,
            data_root: root.path(),
        };
        publish_runtime_state_notification(Some(publisher), RuntimeState::Unreachable, true, &log);
        store.lock().expect("lock").mark_all_read();
        publish_runtime_state_notification(Some(publisher), RuntimeState::Unreachable, true, &log);
        let guard = store.lock().expect("notification store lock");
        let live = guard.live();
        assert_eq!(live.len(), 1, "同一问题只保留一条");
        assert!(!live[0].read, "重复出现应恢复未读");
    }

    #[test]
    fn observation_recovery_requires_the_same_process_context() {
        use crate::infrastructure::runtime_executable::FixedRuntimeExecutable;
        use crate::modules::notifications::registry::{Action, Phase};

        let root = tempfile::tempdir().expect("temp data root");
        let executable = root.path().join("ocx");
        let other_executable = root.path().join("other-ocx");
        std::fs::write(&executable, b"candidate-a").expect("runtime fixture");
        std::fs::write(&other_executable, b"candidate-b").expect("runtime fixture");
        let working_directory = root.path().join("work");
        let other_working_directory = root.path().join("other-work");
        let opencodex_home = root.path().join("home");
        let other_opencodex_home = root.path().join("other-home");
        for path in [
            &working_directory,
            &other_working_directory,
            &opencodex_home,
            &other_opencodex_home,
        ] {
            std::fs::create_dir_all(path).expect("process context fixture");
        }
        let context = crate::state::ProcessContext {
            runtime: FixedRuntimeExecutable::resolved(executable.clone()),
            working_directory: working_directory.clone(),
            opencodex_home: opencodex_home.clone(),
        };
        let different_executable = crate::state::ProcessContext {
            runtime: FixedRuntimeExecutable::resolved(other_executable),
            working_directory: working_directory.clone(),
            opencodex_home: opencodex_home.clone(),
        };
        let different_working_directory = crate::state::ProcessContext {
            runtime: FixedRuntimeExecutable::resolved(executable.clone()),
            working_directory: other_working_directory,
            opencodex_home: opencodex_home.clone(),
        };
        let different_opencodex_home = crate::state::ProcessContext {
            runtime: FixedRuntimeExecutable::resolved(executable.clone()),
            working_directory,
            opencodex_home: other_opencodex_home,
        };
        let store: crate::state::SharedNotificationStore = std::sync::Arc::new(
            std::sync::Mutex::new(crate::modules::notifications::NotificationStore::new()),
        );
        let log = crate::infrastructure::runtime_log::RuntimeLog::new(root.path());
        let publisher = crate::commands::notifications::NotificationPublisher {
            store: &store,
            data_root: root.path(),
        };

        assert!(publish_runtime_state_notification_with_context(
            Some(publisher),
            RuntimeState::StartingFailed,
            true,
            &log,
            Some(&context),
        ));
        let failure = store.lock().expect("notification store lock").all()[0].clone();
        let identity = failure.event_identity.expect("观测失败必须有作用域");
        assert_eq!(identity.action, Action::Observe);
        assert_eq!(identity.phase, Phase::Observation);
        let same_identity = crate::modules::notifications::registry::observation_identity(
            root.path(),
            &executable,
            &context.working_directory,
            &context.opencodex_home,
        )
        .expect("same observation identity");
        assert_eq!(identity, same_identity);
        assert_eq!(
            failure.notification_id.split(':').next(),
            Some("runtime-starting-failed")
        );
        assert!(publish_runtime_state_notification_with_context(
            Some(publisher),
            RuntimeState::Running,
            true,
            &log,
            Some(&context),
        ));
        assert!(store.lock().expect("notification store lock").all()[0].resolved);

        for different in [
            &different_executable,
            &different_working_directory,
            &different_opencodex_home,
        ] {
            assert!(publish_runtime_state_notification_with_context(
                Some(publisher),
                RuntimeState::StartingFailed,
                true,
                &log,
                Some(&context),
            ));
            let failure_index = store.lock().expect("notification store lock").all().len() - 1;
            assert!(!publish_runtime_state_notification_with_context(
                Some(publisher),
                RuntimeState::Running,
                true,
                &log,
                Some(different),
            ));
            assert!(
                !store.lock().expect("notification store lock").all()[failure_index].resolved,
                "不同运行上下文不应解除观测异常"
            );
        }
        assert_eq!(identity.action, Action::Observe);
        assert_eq!(identity.phase, Phase::Observation);
    }

    #[test]
    fn observation_recovery_cannot_resolve_lifecycle_execution_failure() {
        use crate::infrastructure::runtime_executable::FixedRuntimeExecutable;
        use crate::modules::notifications::registry::{
            lifecycle_identity, terminal_delivery, Evidence, Trigger,
        };

        let root = tempfile::tempdir().expect("temp data root");
        let executable = root.path().join("ocx");
        let working_directory = root.path().join("work");
        let opencodex_home = root.path().join("home");
        std::fs::write(&executable, b"candidate").expect("runtime fixture");
        std::fs::create_dir_all(&working_directory).expect("working directory");
        std::fs::create_dir_all(&opencodex_home).expect("OpenCodex home");
        let context = crate::state::ProcessContext {
            runtime: FixedRuntimeExecutable::resolved(executable.clone()),
            working_directory: working_directory.clone(),
            opencodex_home: opencodex_home.clone(),
        };
        let store: crate::state::SharedNotificationStore = std::sync::Arc::new(
            std::sync::Mutex::new(crate::modules::notifications::NotificationStore::new()),
        );
        let publisher = crate::commands::notifications::NotificationPublisher {
            store: &store,
            data_root: root.path(),
        };
        let identity = lifecycle_identity(
            root.path(),
            &executable,
            &working_directory,
            &opencodex_home,
            crate::modules::notifications::registry::Action::Start,
        )
        .expect("lifecycle identity");
        let failure = terminal_delivery(
            "run-start-failed",
            Trigger::User,
            identity.clone(),
            Evidence::Failure,
            chrono::Utc::now(),
        )
        .expect("lifecycle failure delivery");
        assert!(
            publisher
                .publish_event(&failure)
                .expect("persist failure")
                .added
        );

        let log = crate::infrastructure::runtime_log::RuntimeLog::new(root.path());
        assert!(!publish_runtime_state_notification_with_context(
            Some(publisher),
            RuntimeState::Running,
            true,
            &log,
            Some(&context),
        ));
        let store_guard = store.lock().expect("notification store lock");
        let saved = store_guard.all();
        assert_eq!(saved.len(), 1);
        assert!(!saved[0].resolved);
        assert_eq!(saved[0].event_identity.as_ref(), Some(&identity));
    }

    #[test]
    fn observation_recovery_registration_is_explicit_and_scoped() {
        use crate::modules::notifications::registry::{Action, Fact, Job, Phase};

        let event =
            crate::modules::notifications::registry::lookup("runtime-observation-recovered")
                .expect("观测恢复事件必须登记");
        assert_eq!(event.job, Job::Observe);
        assert_eq!(
            event.object,
            crate::modules::notifications::registry::ObjectKind::Runtime
        );
        assert_eq!(event.action, Action::Observe);
        assert_eq!(event.phase, Phase::Observation);
        assert_eq!(event.fact, Fact::Success);
        assert_eq!(
            event.resolves,
            vec![
                "runtime-starting-failed".to_owned(),
                "run-unreachable".to_owned(),
                "run-at-risk".to_owned(),
                "external-takeover".to_owned(),
            ]
        );
    }

    /// restore 引导按目标分文案：外部接管与 startup at-risk 不能混成同一 target。
    #[test]
    fn restore_notifications_map_to_their_own_target() {
        let at_risk = runtime_state_notification(RuntimeState::AtRisk).expect("通知");
        let takeover = runtime_state_notification(RuntimeState::ExternalTakeover).expect("通知");
        assert_eq!(
            crate::types::notifications::NotificationDto::from_domain(&at_risk).target,
            Some(RuntimeState::AtRisk)
        );
        assert_eq!(
            crate::types::notifications::NotificationDto::from_domain(&takeover).target,
            Some(RuntimeState::ExternalTakeover)
        );
    }
}

#[cfg(test)]
mod writer_dispatch_tests {
    use super::*;

    #[tokio::test]
    async fn cancelled_observer_does_not_release_blocking_worker_admission() {
        let gate =
            std::sync::Arc::new(crate::infrastructure::storage_writers::WriterGate::default());
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let worker_gate = gate.clone();
        let observer = tokio::spawn(async move {
            run_blocking_with_gate(worker_gate, "fixture worker", move || {
                let _ = started_tx.send(());
                release_rx.recv().unwrap();
                Ok(())
            })
            .await
        });
        started_rx.await.unwrap();
        observer.abort();
        assert!(observer.await.unwrap_err().is_cancelled());
        assert!(
            gate.freeze().unwrap().is_none(),
            "worker remains admitted after observer cancellation"
        );
        release_tx.send(()).unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(3), async {
            loop {
                if let Some(binding) = gate.freeze().unwrap() {
                    drop(binding);
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("actual worker releases admission on completion");
    }
}
