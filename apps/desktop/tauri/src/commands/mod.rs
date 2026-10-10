//! Tauri 命令层。命令只做参数转换与模块编排，不直接访问平台 API。

pub mod about;
pub mod cleanup;
pub mod codex_shim;
pub mod data_root;
pub mod discovery;
pub mod doctor;
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
use crate::modules::notifications::{
    Notification, NotificationAction, NotificationCategory, NotificationLevel, NotificationSource,
};
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
) -> AppResult<ProcessActionResult> {
    let runtime_log = crate::infrastructure::runtime_log::RuntimeLog::new(&data_root.0);
    // 「启停结果通知」关闭时不再产生持久通知；成功/取消本来就不写通知。
    let publisher = publisher_if_enabled(
        lifecycle_notifications_enabled(&data_root.0),
        notifications.inner(),
        &data_root.0,
    );
    process_action_with_runner(request, runner.inner(), &context, &runtime_log, publisher)
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
    if failed {
        publish_run_failure(notifications, request.action, runtime_log);
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

/// 成功与用户取消**不写入**通知：低风险成功走 Toast，取消是用户主动行为，
/// 都不应占用持久未读。失败按 `run:<action>-failed` 去重，重复失败只更新同一条，
/// 不堆叠多条同类提醒。发布失败单独记入运行日志，不改变主流程的返回值。
fn publish_run_failure(
    notifications: Option<crate::commands::notifications::NotificationPublisher<'_>>,
    action: LifecycleAction,
    runtime_log: &crate::infrastructure::runtime_log::RuntimeLog,
) {
    let Some(publisher) = notifications else {
        return;
    };
    let Some(notification) = run_failure_notification(action) else {
        return;
    };
    if let Err(error) = publisher.publish(notification) {
        let _ = runtime_log.append_result(
            action_log_label(action),
            Err(format!("notification publish failed: {error}")),
        );
    }
}

/// 失败通知只描述**已发生的事实**，不写固定延迟、示例端口或版本号。
fn run_failure_notification(action: LifecycleAction) -> Option<Notification> {
    let (label, title) = match action {
        LifecycleAction::Start => ("start", "OpenCodex 启动失败"),
        LifecycleAction::Stop => ("stop", "OpenCodex 停止失败"),
        LifecycleAction::Restart => ("restart", "OpenCodex 重启失败"),
    };
    let notification = Notification::new(
        format!("run-{label}-failed"),
        NotificationLevel::Danger,
        NotificationCategory::Run,
        NotificationSource::Runtime,
        title,
        "官方命令未成功完成；具体原因以诊断中心日志中的同动作记录为准。",
        chrono::Utc::now().to_rfc3339(),
        Some(NotificationAction::Logs),
    )
    .ok()?;
    Some(notification.with_dedupe_key(format!("run:{label}-failed")))
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
/// `run-start-failed` 与启停失败通知共用去重键 `run:start-failed`：同一次启动失败
/// 只保留一条，不出现「命令失败」与「状态失败」两条并排。
pub fn runtime_state_notification(
    runtime: crate::types::status::RuntimeState,
) -> Option<Notification> {
    use crate::types::status::RuntimeState;
    let (id, dedupe, level, category, title, body, action) = match runtime {
        RuntimeState::NotFound => (
            "runtime-not-found",
            "runtime-not-found",
            NotificationLevel::Warning,
            NotificationCategory::System,
            "未发现可运行的 OpenCodex",
            "可以在「安装配置」里把官方包托管安装到数据根内，或导入官方离线包。",
            NotificationAction::SettingsInstallation,
        ),
        RuntimeState::StartingFailed => (
            "run-start-failed",
            "run:start-failed",
            NotificationLevel::Danger,
            NotificationCategory::Run,
            "OpenCodex 启动失败",
            "具体原因以诊断中心日志中的同动作记录为准；不会自动重试。",
            NotificationAction::Logs,
        ),
        RuntimeState::Unreachable => (
            "run-unreachable",
            "run-unreachable",
            NotificationLevel::Danger,
            NotificationCategory::Run,
            "进程或端口不可达",
            "建议刷新状态或查看日志确认原因。",
            NotificationAction::Logs,
        ),
        RuntimeState::AtRisk => (
            "run-at-risk",
            "run-at-risk",
            NotificationLevel::Warning,
            NotificationCategory::Run,
            "官方状态报告启动存在风险",
            "桌面壳不会自动修复；请先查看建议，再决定是否走官方恢复（restore）。",
            NotificationAction::Restore,
        ),
        RuntimeState::ExternalTakeover => (
            "external-takeover",
            "external-takeover",
            NotificationLevel::Warning,
            NotificationCategory::Run,
            "外部 provider 已接管 Codex 配置",
            "该所有权边界属于 OpenCodex CLI 本身；桌面壳只解释影响，不会覆盖外部配置。",
            NotificationAction::Restore,
        ),
        _ => return None,
    };
    let notification = Notification::new(
        id,
        level,
        category,
        NotificationSource::Runtime,
        title,
        body,
        chrono::Utc::now().to_rfc3339(),
        Some(action),
    )
    .ok()?;
    Some(notification.with_dedupe_key(dedupe))
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
    if is_lifecycle_outcome(runtime) && !lifecycle_notifications {
        return false;
    }
    let Some(publisher) = publisher else {
        return false;
    };
    let Some(notification) = runtime_state_notification(runtime) else {
        return false;
    };
    if let Err(error) = publisher.publish(notification) {
        let _ = runtime_log.append_result(
            "runtime-state-notification",
            Err(format!("notification publish failed: {error}")),
        );
        return false;
    }
    true
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

    /// 同一次启动失败只保留一条：状态观测与命令失败共用同一条目与去重键。
    #[test]
    fn start_failure_merges_with_command_failure() {
        let observed = runtime_state_notification(RuntimeState::StartingFailed).expect("通知");
        let command = run_failure_notification(LifecycleAction::Start).expect("通知");
        assert_eq!(observed.notification_id, command.notification_id);
        assert_eq!(observed.dedupe_key, command.dedupe_key);
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
