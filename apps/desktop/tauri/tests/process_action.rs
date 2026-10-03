use std::os::unix::fs::PermissionsExt;

use opencodex_desktop_lib::commands::process_action_with_runner;
use opencodex_desktop_lib::infrastructure::process_runner::ControlledProcessRunner;
use opencodex_desktop_lib::infrastructure::runtime_executable::FixedRuntimeExecutable;
use opencodex_desktop_lib::modules::process::LifecycleResult;
use opencodex_desktop_lib::modules::process::ProcessRunner;
use opencodex_desktop_lib::modules::process::{LifecycleAction, ProcessLifecycleState};
use opencodex_desktop_lib::state::ProcessContext;
use opencodex_desktop_lib::types::process_action::ProcessActionRequest;

struct ProcessFixture {
    _root: tempfile::TempDir,
    context: ProcessContext,
    runtime_log: opencodex_desktop_lib::infrastructure::runtime_log::RuntimeLog,
}

fn fixture() -> ProcessFixture {
    let root = tempfile::tempdir().expect("create temporary fixture");
    let executable = root.path().join("ocx");
    std::fs::write(&executable, b"#!/bin/sh\nexit 0\n").expect("write fixture");
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755))
        .expect("chmod fixture");
    let working_directory = root.path().join("home");
    std::fs::create_dir_all(&working_directory).expect("create workdir fixture");
    let opencodex_home = root.path().join("opencodex-home");
    std::fs::create_dir_all(&opencodex_home).expect("create opencodex home fixture");
    let context = ProcessContext {
        runtime: FixedRuntimeExecutable::resolved(executable),
        working_directory,
        opencodex_home,
    };
    let runtime_log =
        opencodex_desktop_lib::infrastructure::runtime_log::RuntimeLog::new(root.path());
    ProcessFixture {
        _root: root,
        context,
        runtime_log,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn process_action_requires_explicit_confirm() {
    let fixture = fixture();
    let runner = std::sync::Mutex::new(ControlledProcessRunner::new(
        fixture.context.working_directory.clone(),
    ));
    let request = ProcessActionRequest {
        action: LifecycleAction::Start,
        confirm: false,
    };
    let result = process_action_with_runner(
        request,
        &runner,
        &fixture.context,
        &fixture.runtime_log,
        None,
    );
    assert!(
        result.is_err(),
        "unconfirmed actions must not advance lifecycle"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn process_action_preserves_observation_when_context_is_unavailable() {
    let fixture = fixture();
    let runner = std::sync::Mutex::new(ControlledProcessRunner::new(
        fixture.context.working_directory.clone(),
    ));
    let missing = ProcessContext {
        runtime: FixedRuntimeExecutable::resolved(fixture._root.path().join("missing-ocx")),
        working_directory: fixture.context.working_directory.clone(),
        opencodex_home: fixture.context.opencodex_home.clone(),
    };
    let start = process_action_with_runner(
        ProcessActionRequest {
            action: LifecycleAction::Start,
            confirm: true,
        },
        &runner,
        &missing,
        &fixture.runtime_log,
        None,
    )
    .expect("unconfigured runtime should preserve state");
    assert_eq!(start.lifecycle_state, ProcessLifecycleState::Stopped);
    assert_eq!(start.result, None);
    assert!(start.can_start);

    let stop = process_action_with_runner(
        ProcessActionRequest {
            action: LifecycleAction::Stop,
            confirm: true,
        },
        &runner,
        &missing,
        &fixture.runtime_log,
        None,
    )
    .expect("unconfigured runtime should preserve running observation");
    assert_eq!(stop.lifecycle_state, ProcessLifecycleState::Running);
    assert_eq!(stop.result, None);
    assert!(!stop.can_start);
    assert!(stop.can_restart);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn process_action_uses_controlled_context() {
    let fixture = fixture();
    let runner = std::sync::Mutex::new(ControlledProcessRunner::new(
        fixture.context.working_directory.clone(),
    ));
    let result = process_action_with_runner(
        ProcessActionRequest {
            action: LifecycleAction::Start,
            confirm: true,
        },
        &runner,
        &fixture.context,
        &fixture.runtime_log,
        None,
    )
    .expect("fixture context should execute");
    assert_eq!(result.lifecycle_state, ProcessLifecycleState::Starting);
    assert_eq!(result.result, Some(LifecycleResult::Started));
    assert!(!result.can_start);
    assert!(!result.can_stop);
    assert!(!result.can_restart);
}

// ---- 真实通知生产者（FZ-43 / 领域模型 §16.1）----

use opencodex_desktop_lib::commands::notifications::{
    list_notifications_with_store, NotificationPublisher,
};
use opencodex_desktop_lib::infrastructure::runtime_log::RuntimeLog;
use opencodex_desktop_lib::modules::notifications::persistence::{
    load_notifications, notifications_path,
};
use opencodex_desktop_lib::modules::notifications::{
    NotificationAction, NotificationCategory, NotificationLevel, NotificationSource,
    NotificationStore,
};
use opencodex_desktop_lib::state::SharedNotificationStore;
use std::sync::{Arc, Mutex};

struct ProducerFixture {
    root: tempfile::TempDir,
    context: ProcessContext,
    runtime_log: RuntimeLog,
}

fn producer_fixture(exit_code: u8) -> ProducerFixture {
    let root = tempfile::tempdir().expect("create temporary fixture");
    std::fs::create_dir_all(root.path().join("manager-state")).expect("create manager-state");
    let executable = root.path().join("ocx");
    std::fs::write(&executable, format!("#!/bin/sh\nexit {exit_code}\n")).expect("write fixture");
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755))
        .expect("chmod fixture");
    let working_directory = root.path().join("home");
    std::fs::create_dir_all(&working_directory).expect("create workdir fixture");
    let opencodex_home = root.path().join("opencodex-home");
    std::fs::create_dir_all(&opencodex_home).expect("create opencodex home fixture");
    let context = ProcessContext {
        runtime: FixedRuntimeExecutable::resolved(executable),
        working_directory,
        opencodex_home,
    };
    let runtime_log = RuntimeLog::new(root.path());
    ProducerFixture {
        root,
        context,
        runtime_log,
    }
}

fn empty_store() -> SharedNotificationStore {
    Arc::new(Mutex::new(NotificationStore::new()))
}

async fn run_action(
    fixture: &ProducerFixture,
    context: &ProcessContext,
    store: &SharedNotificationStore,
    action: LifecycleAction,
) -> Option<LifecycleResult> {
    let runner = Mutex::new(ControlledProcessRunner::new(
        context.working_directory.clone(),
    ));
    let result = process_action_with_runner(
        ProcessActionRequest {
            action,
            confirm: true,
        },
        &runner,
        context,
        &fixture.runtime_log,
        Some(NotificationPublisher {
            store,
            data_root: fixture.root.path(),
        }),
    )
    .expect("process action returns an explicit result");
    result.result
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn run_failure_publishes_one_deduped_danger_notification() {
    // `Start` / `Restart` 按设计在拉起进程后立即返回 `Started`（守护式启动），
    // 因此终态失败来自 `Stop` 的非零退出，或 `Err` 的启动失败（见下一个用例）。
    let fixture = producer_fixture(3);
    let store = empty_store();
    let context = fixture.context.clone();

    // 同一动作重复失败只保留一条（FZ-43.2 去重），不堆叠同类提醒。
    assert_eq!(
        run_action(&fixture, &context, &store, LifecycleAction::Stop).await,
        Some(LifecycleResult::Failed)
    );
    assert_eq!(
        run_action(&fixture, &context, &store, LifecycleAction::Stop).await,
        Some(LifecycleResult::Failed)
    );

    let value = list_notifications_with_store(&store).expect("list notifications");
    assert_eq!(value.items.len(), 1);
    assert_eq!(value.items[0].kind, NotificationLevel::Danger);
    assert_eq!(value.items[0].category, NotificationCategory::Run);
    assert_eq!(value.items[0].source, NotificationSource::Runtime);
    assert_eq!(value.items[0].action, Some(NotificationAction::Logs));
    assert_eq!(value.aggregate.unread, 1);
    assert_eq!(value.aggregate.danger_unread, 1);

    // 失败通知已经落盘：模拟重启后仍按原样恢复。
    let restored = load_notifications(&notifications_path(fixture.root.path()))
        .expect("reload persisted notifications");
    assert_eq!(restored.live().len(), 1);
    assert_eq!(restored.live()[0].level, NotificationLevel::Danger);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn spawn_failure_also_publishes_a_danger_notification() {
    let fixture = producer_fixture(0);
    let store = empty_store();
    // 上下文不可用 → `Err` 分支：状态保持观察，但真实失败仍要产生一条通知。
    let missing = ProcessContext {
        runtime: FixedRuntimeExecutable::resolved(fixture.root.path().join("missing-ocx")),
        working_directory: fixture.context.working_directory.clone(),
        opencodex_home: fixture.context.opencodex_home.clone(),
    };

    assert_eq!(
        run_action(&fixture, &missing, &store, LifecycleAction::Start).await,
        None
    );

    let value = list_notifications_with_store(&store).expect("list notifications");
    assert_eq!(value.items.len(), 1);
    assert_eq!(value.items[0].kind, NotificationLevel::Danger);
    assert_eq!(value.items[0].source, NotificationSource::Runtime);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn run_success_publishes_no_notification() {
    let fixture = producer_fixture(0);
    let store = empty_store();
    let context = fixture.context.clone();

    assert_eq!(
        run_action(&fixture, &context, &store, LifecycleAction::Start).await,
        Some(LifecycleResult::Started)
    );

    // 低风险成功走 Toast，不占持久未读（FZ-43.2）。
    let value = list_notifications_with_store(&store).expect("list notifications");
    assert!(value.items.is_empty());
}

/// 回归：`stop` 必须真正等到子进程结束后收口，不能永久挂起。
///
/// 此前 Stop 依赖 `tokio::process::Child::wait()`，该 future 在本环境永不就绪：
/// 超时窗口过后还有一次无上限的 `await`，实测把 `ocxd stop` 挂死超过 3 分钟。
#[test]
fn stop_reaps_the_child_and_returns_within_the_window() {
    let fixture = fixture();
    let runner = ControlledProcessRunner::new(fixture.context.working_directory.clone());
    let command = opencodex_desktop_lib::modules::process::ProcessCommand::new(
        LifecycleAction::Stop,
        fixture.context.executable().expect("resolved runtime"),
        fixture.context.working_directory.clone(),
        fixture.context.opencodex_home.clone(),
    );
    let started = std::time::Instant::now();
    let result = runner.execute(&command).expect("stop should return");
    assert_eq!(result, LifecycleResult::Stopped);
    assert!(
        started.elapsed() < std::time::Duration::from_secs(9),
        "正常退出的子进程不该等到超时窗口：{:?}",
        started.elapsed()
    );
}

/// 回归：子进程忽略 SIGTERM 时，`stop` 必须在固定窗口内如实返回，
/// 而不是无限等待（策略禁止 SIGKILL，所以只会报告未收口）。
#[test]
fn stop_returns_within_a_bounded_window_when_sigterm_is_ignored() {
    let root = tempfile::tempdir().expect("fixture root");
    let executable = root.path().join("ocx");
    std::fs::write(
        &executable,
        b"#!/bin/sh\ntrap '' TERM\nwhile true; do sleep 1; done # ocx-hang-marker\n",
    )
    .expect("write fixture");
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755))
        .expect("chmod fixture");
    let working_directory = root.path().join("home");
    std::fs::create_dir_all(&working_directory).expect("workdir");
    let opencodex_home = root.path().join("opencodex-home");
    std::fs::create_dir_all(&opencodex_home).expect("home");

    let runner = ControlledProcessRunner::new(working_directory.clone());
    let command = opencodex_desktop_lib::modules::process::ProcessCommand::new(
        LifecycleAction::Stop,
        executable,
        working_directory,
        opencodex_home,
    );
    let started = std::time::Instant::now();
    let result = runner.execute(&command).expect("stop should return");
    let elapsed = started.elapsed();
    assert_eq!(result, LifecycleResult::Cancelled);
    assert!(
        elapsed < std::time::Duration::from_secs(30),
        "超时后必须在固定窗口内收口，实际 {:?}",
        elapsed
    );
    // 策略禁止 SIGKILL，但测试进程必须自己收拾干净。
    let _ = std::process::Command::new("pkill")
        .args(["-9", "-f", "ocx-hang-marker"])
        .status();
}
