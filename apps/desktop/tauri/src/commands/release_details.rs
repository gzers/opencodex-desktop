//! Notes are a separate read-only transaction, never a version check or install.
use crate::{
    errors::{AppError, AppResult},
    modules::update::release_notes::{self, Channel, Details, Target},
};
use std::sync::OnceLock;
use tauri::Manager;

#[tauri::command]
pub async fn update_release_details(
    target: Target,
    channel: Channel,
    candidate_version: String,
    force: Option<bool>,
    app: tauri::AppHandle,
) -> AppResult<Details> {
    if !release_notes::valid_request(target, channel, &candidate_version) {
        return Err(AppError::NotConfigured);
    }
    // The detached admitted owner completes its bounded cache transaction even
    // if the UI observer closes. A root binding save cannot interrupt its writes.
    let storage = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn(async move {
        let _storage = storage;
        static LOCKS: OnceLock<[tokio::sync::Mutex<()>; 3]> = OnceLock::new();
        let index = match channel {
            Channel::Stable => 0,
            Channel::Beta => 1,
            Channel::Official => 2,
        };
        let _serial = LOCKS.get_or_init(|| std::array::from_fn(|_| tokio::sync::Mutex::new(())))
            [index]
            .lock()
            .await;
        let prepare_app = app.clone();
        let version = candidate_version.clone();
        let (root, updater_notes, generation, policy) =
            super::run_readonly("prepare release details", move || {
                let root = prepare_app
                    .state::<crate::state::SharedDataRoot>()
                    .0
                    .clone();
                let (notes, generation) = if target == Target::Manager {
                    let store = prepare_app.state::<crate::commands::update::SharedUpdateStatus>();
                    let status = store.lock().map_err(|_| AppError::NotConfigured)?;
                    let expected = match status.channel {
                        crate::modules::update::UpdateChannel::Stable => Channel::Stable,
                        crate::modules::update::UpdateChannel::Beta => Channel::Beta,
                    };
                    if expected != channel
                        || status.available_version.as_deref() != Some(version.as_str())
                        || status.error.is_some()
                    {
                        return Err(AppError::NotConfigured);
                    }
                    (status.notes.clone(), Some(status.generation))
                } else {
                    let cached = crate::modules::update::schedule::load_cache::<
                        crate::modules::about::remote::OfficialRemoteLatest,
                    >(
                        &root, crate::modules::update::schedule::Target::Panel
                    )
                    .ok_or(AppError::NotConfigured)?;
                    if cached.version != version {
                        return Err(AppError::NotConfigured);
                    }
                    (None, None)
                };
                Ok((
                    root,
                    notes,
                    generation,
                    crate::modules::preferences::proxy_policy_for_app(&prepare_app),
                ))
            })
            .await?;
        let limits = crate::modules::runtime_defaults::release_notes();
        let source =
            release_notes::HttpSource::new(policy, limits).map_err(|_| AppError::NotConfigured)?;
        let details = release_notes::query(
            &source,
            &root,
            release_notes::Request {
                target,
                channel,
                version: &candidate_version,
                updater_notes: updater_notes.as_deref(),
                force: force.unwrap_or(false),
                now: chrono::Utc::now().timestamp(),
            },
            limits,
        )
        .await
        .map_err(|_| AppError::NotConfigured)?;
        if let Some(generation) = generation {
            let store = app.state::<crate::commands::update::SharedUpdateStatus>();
            let status = store.lock().map_err(|_| AppError::NotConfigured)?;
            if status.generation != generation
                || status.available_version.as_deref() != Some(candidate_version.as_str())
            {
                return Err(AppError::NotConfigured);
            }
        }
        Ok(details)
    })
    .await
    .map_err(|_| AppError::NotConfigured)?
}
