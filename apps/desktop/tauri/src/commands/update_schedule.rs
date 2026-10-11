//! One schedule for manual and automatic checks. Commands never install automatically.
use crate::errors::{AppError, AppResult};
use crate::modules::update::schedule::{self, Plan, Schedule, Target};
use std::sync::Mutex;
use tauri::Manager;

#[derive(Default)]
pub struct SharedSchedule(pub Mutex<()>);
#[derive(Default)]
pub struct PanelQuery {
    pub sequence: u64,
    pub value: Option<crate::modules::about::remote::OfficialRemoteLatest>,
}
#[derive(Default)]
pub struct SharedPanelQuery(
    pub tokio::sync::Mutex<PanelQuery>,
    pub std::sync::atomic::AtomicU64,
);
pub fn active_root(app: &tauri::AppHandle) -> AppResult<std::path::PathBuf> {
    let root = app.state::<crate::state::SharedDataRoot>();
    Ok(root.0.clone())
}
pub fn reserve(app: &tauri::AppHandle, target: Target) -> AppResult<std::path::PathBuf> {
    let _admission = crate::infrastructure::storage_writers::global().admit()?;
    let binding = app.state::<SharedSchedule>();
    let _lock = binding.0.lock().map_err(|_| AppError::NotConfigured)?;
    let root = active_root(app)?;
    let mut state = Schedule::load(&root)?;
    state.reserve(target, chrono::Utc::now().timestamp());
    state.save(&root)?;
    Ok(root)
}
pub fn complete<T: serde::Serialize>(
    app: &tauri::AppHandle,
    root: &std::path::Path,
    target: Target,
    value: Option<&T>,
) -> AppResult<()> {
    let _admission = crate::infrastructure::storage_writers::global().admit()?;
    let binding = app.state::<SharedSchedule>();
    let _lock = binding.0.lock().map_err(|_| AppError::NotConfigured)?;
    complete_at_root(root, target, value, chrono::Utc::now().timestamp())
}

pub(crate) fn complete_at_root<T: serde::Serialize>(
    root: &std::path::Path,
    target: Target,
    value: Option<&T>,
    now: i64,
) -> AppResult<()> {
    let mut state = Schedule::load(root)?;
    if let Some(value) = value {
        schedule::save_cache(root, target, value)?;
    }
    state.complete(target, now, value.is_some());
    state.save(root)
}
#[tauri::command]
pub fn update_schedule_plan(app: tauri::AppHandle) -> AppResult<Plan> {
    let root = active_root(&app)?;
    let prefs = crate::modules::preferences::PreferencesStore::new(&root)
        .load()
        .map_err(AppError::from)?;
    let status = app.state::<crate::commands::update::SharedUpdateStatus>();
    let status = status.lock().map_err(|_| AppError::NotConfigured)?;
    let busy = crate::infrastructure::storage_writers::global().frozen()
        || status.installing
        || status.checking
        || app
            .state::<crate::state::SharedRuntimeInstall>()
            .is_running();
    let channel = status.channel;
    drop(status);
    let binding = app.state::<SharedSchedule>();
    // A channel transition belongs to the preferences transaction, not this scheduler.
    plan_at_root(
        &root,
        chrono::Utc::now().timestamp(),
        channel,
        prefs.app_update_auto_check,
        busy,
        &crate::infrastructure::storage_writers::global(),
        &binding.0,
    )
}
fn plan_at_root(
    root: &std::path::Path,
    now: i64,
    channel: crate::modules::update::UpdateChannel,
    enabled: bool,
    busy: bool,
    gate: &std::sync::Arc<crate::infrastructure::storage_writers::WriterGate>,
    lock: &Mutex<()>,
) -> AppResult<Plan> {
    // Admission precedes the schedule mutex, matching query reserve/complete.
    // A freeze racing the earlier busy snapshot defers both repair and queries.
    let admission = if enabled && !busy {
        gate.admit().ok()
    } else {
        None
    };
    let busy = busy || (enabled && admission.is_none());
    let _lock = lock.lock().map_err(|_| AppError::NotConfigured)?;
    let mut state = Schedule::load(root)?;
    if enabled && !busy && state.reconcile_clock(now, channel) {
        state.save(root)?;
    }
    Ok(state.plan(now, channel, enabled, busy))
}
#[tauri::command]
pub fn official_remote_cache(
    app: tauri::AppHandle,
) -> AppResult<Option<crate::modules::about::remote::OfficialRemoteLatest>> {
    Ok(schedule::load_cache(&active_root(&app)?, Target::Panel))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::storage_writers::WriterGate;
    use crate::modules::update::UpdateChannel;
    use std::sync::Arc;

    #[test]
    fn rollback_deadlines_are_persisted_once_and_survive_restarts() {
        let root = tempfile::tempdir().unwrap();
        let gate = Arc::new(WriterGate::default());
        let lock = Mutex::new(());
        let original_now = 100 * 86400;
        let now = 90 * 86400;
        let mut state = Schedule::default();
        for target in [Target::ManagerStable, Target::ManagerBeta, Target::Panel] {
            state.complete(target, original_now, true);
        }
        state.save(root.path()).unwrap();
        let plan = plan_at_root(
            root.path(),
            now,
            UpdateChannel::Beta,
            true,
            false,
            &gate,
            &lock,
        )
        .unwrap();
        assert!(plan.targets.is_empty());
        assert_eq!(plan.next_delay_ms, Some(21600 * 1000));
        let repaired = Schedule::load(root.path()).unwrap();
        assert_eq!(repaired.entries["manager-beta"].next_due, now + 21600);
        assert_eq!(repaired.entries["panel-latest"].next_due, now + 86400);
        // Inactive channels and historical timestamps are not overwritten.
        assert_eq!(
            repaired.entries["manager-stable"].next_due,
            original_now + 86400
        );
        assert_eq!(
            repaired.entries["manager-beta"].last_success,
            Some(original_now)
        );
        let bytes = std::fs::read(root.path().join(schedule::STATE_PATH)).unwrap();
        let modified = std::fs::metadata(root.path().join(schedule::STATE_PATH))
            .unwrap()
            .modified()
            .unwrap();
        for step in 0..100 {
            assert!(plan_at_root(
                root.path(),
                now + step,
                UpdateChannel::Beta,
                true,
                false,
                &gate,
                &lock
            )
            .unwrap()
            .targets
            .is_empty());
        }
        assert_eq!(
            std::fs::read(root.path().join(schedule::STATE_PATH)).unwrap(),
            bytes
        );
        assert_eq!(
            std::fs::metadata(root.path().join(schedule::STATE_PATH))
                .unwrap()
                .modified()
                .unwrap(),
            modified
        );
        assert_eq!(
            plan_at_root(
                root.path(),
                now + 21600,
                UpdateChannel::Beta,
                true,
                false,
                &gate,
                &lock
            )
            .unwrap()
            .targets,
            vec![Target::ManagerBeta]
        );
        assert_eq!(
            plan_at_root(
                root.path(),
                now + 86400,
                UpdateChannel::Beta,
                true,
                false,
                &gate,
                &lock
            )
            .unwrap()
            .targets,
            vec![Target::ManagerBeta, Target::Panel]
        );
    }

    #[test]
    fn absurd_future_deadline_retains_failure_backoff_without_immediate_retry() {
        for (failures, delay) in [(1, 300), (2, 1800), (3, 7200), (4, 86400)] {
            let root = tempfile::tempdir().unwrap();
            let gate = Arc::new(WriterGate::default());
            let lock = Mutex::new(());
            let mut state = Schedule::default();
            for _ in 0..failures {
                state.complete(Target::ManagerStable, 100, false);
            }
            state.entries.get_mut("manager-stable").unwrap().next_due = i64::MAX;
            state.complete(Target::Panel, 100, true);
            state.save(root.path()).unwrap();
            let plan = plan_at_root(
                root.path(),
                100,
                UpdateChannel::Stable,
                true,
                false,
                &gate,
                &lock,
            )
            .unwrap();
            assert!(plan.targets.is_empty());
            assert_eq!(plan.next_delay_ms, Some(delay * 1000));
            let repaired = Schedule::load(root.path()).unwrap();
            assert_eq!(repaired.entries["manager-stable"].failures, failures);
            assert_eq!(
                repaired.entries["manager-stable"].next_due,
                100 + delay as i64
            );
            assert!(plan_at_root(
                root.path(),
                99 + delay as i64,
                UpdateChannel::Stable,
                true,
                false,
                &gate,
                &lock
            )
            .unwrap()
            .targets
            .is_empty());
            assert!(plan_at_root(
                root.path(),
                100 + delay as i64,
                UpdateChannel::Stable,
                true,
                false,
                &gate,
                &lock
            )
            .unwrap()
            .targets
            .contains(&Target::ManagerStable));
        }
    }

    #[test]
    fn disabled_busy_or_frozen_storage_never_repairs_or_queries() {
        let root = tempfile::tempdir().unwrap();
        let gate = Arc::new(WriterGate::default());
        let lock = Mutex::new(());
        let mut state = Schedule::default();
        state.complete(Target::Panel, 999999, true);
        state.save(root.path()).unwrap();
        let bytes = std::fs::read(root.path().join(schedule::STATE_PATH)).unwrap();
        let disabled = plan_at_root(
            root.path(),
            100,
            UpdateChannel::Stable,
            false,
            false,
            &gate,
            &lock,
        )
        .unwrap();
        assert!(disabled.targets.is_empty());
        assert_eq!(disabled.next_delay_ms, None);
        let busy = plan_at_root(
            root.path(),
            100,
            UpdateChannel::Stable,
            true,
            true,
            &gate,
            &lock,
        )
        .unwrap();
        assert!(busy.targets.is_empty());
        assert_eq!(busy.next_delay_ms, Some(60000));
        // A binding freeze can race the command's earlier busy snapshot.
        gate.freeze().unwrap().unwrap().commit();
        let frozen = plan_at_root(
            root.path(),
            100,
            UpdateChannel::Stable,
            true,
            false,
            &gate,
            &lock,
        )
        .unwrap();
        assert!(frozen.targets.is_empty());
        assert_eq!(frozen.next_delay_ms, Some(60000));
        assert_eq!(
            std::fs::read(root.path().join(schedule::STATE_PATH)).unwrap(),
            bytes
        );
    }

    #[test]
    fn normal_deadlines_and_forward_jumps_keep_channel_isolation_and_one_reservation() {
        let root = tempfile::tempdir().unwrap();
        let gate = Arc::new(WriterGate::default());
        let lock = Mutex::new(());
        let mut state = Schedule::default();
        for target in [Target::ManagerStable, Target::ManagerBeta, Target::Panel] {
            state.complete(target, 100, true);
        }
        state.save(root.path()).unwrap();
        let bytes = std::fs::read(root.path().join(schedule::STATE_PATH)).unwrap();
        let modified = std::fs::metadata(root.path().join(schedule::STATE_PATH))
            .unwrap()
            .modified()
            .unwrap();
        assert!(plan_at_root(
            root.path(),
            200,
            UpdateChannel::Beta,
            true,
            false,
            &gate,
            &lock
        )
        .unwrap()
        .targets
        .is_empty());
        let later = 100 + 7 * 86400;
        assert_eq!(
            plan_at_root(
                root.path(),
                later,
                UpdateChannel::Beta,
                true,
                false,
                &gate,
                &lock
            )
            .unwrap()
            .targets,
            vec![Target::ManagerBeta, Target::Panel]
        );
        assert_eq!(
            std::fs::read(root.path().join(schedule::STATE_PATH)).unwrap(),
            bytes
        );
        assert_eq!(
            std::fs::metadata(root.path().join(schedule::STATE_PATH))
                .unwrap()
                .modified()
                .unwrap(),
            modified
        );
        let mut state = Schedule::load(root.path()).unwrap();
        state.reserve(Target::ManagerBeta, later);
        state.reserve(Target::Panel, later);
        state.save(root.path()).unwrap();
        for step in 0..100 {
            assert!(plan_at_root(
                root.path(),
                later + step,
                UpdateChannel::Beta,
                true,
                false,
                &gate,
                &lock
            )
            .unwrap()
            .targets
            .is_empty());
        }
        let reserved = Schedule::load(root.path()).unwrap();
        assert_eq!(reserved.entries["manager-stable"].next_due, 100 + 86400);
        assert_eq!(reserved.entries["manager-beta"].next_due, later + 300);
    }
}
