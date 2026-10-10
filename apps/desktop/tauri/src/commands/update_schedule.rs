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
    let binding = app.state::<SharedSchedule>();
    let _lock = binding.0.lock().map_err(|_| AppError::NotConfigured)?;
    let mut state = Schedule::load(root)?;
    if let Some(value) = value {
        schedule::save_cache(root, target, value)?;
    }
    state.complete(target, chrono::Utc::now().timestamp(), value.is_some());
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
    let busy = status.installing
        || status.checking
        || app
            .state::<crate::state::SharedRuntimeInstall>()
            .is_running();
    let channel = status.channel;
    drop(status);
    let binding = app.state::<SharedSchedule>();
    let _lock = binding.0.lock().map_err(|_| AppError::NotConfigured)?;
    // A channel transition belongs to the preferences transaction, not this scheduler.
    Ok(Schedule::load(&root)?.plan(
        chrono::Utc::now().timestamp(),
        channel,
        prefs.app_update_auto_check,
        busy,
    ))
}
#[tauri::command]
pub fn official_remote_cache(
    app: tauri::AppHandle,
) -> AppResult<Option<crate::modules::about::remote::OfficialRemoteLatest>> {
    Ok(schedule::load_cache(&active_root(&app)?, Target::Panel))
}
