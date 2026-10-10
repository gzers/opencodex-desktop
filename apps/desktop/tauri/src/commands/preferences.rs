//! 偏好命令层。只做显式数据根编排和 DTO 投影，不访问平台细节。

use crate::errors::{AppError, AppResult};
use crate::modules::preferences::{Preferences, PreferencesStore};
use crate::state::SharedDataRoot;
use crate::types::preferences::PreferencesDto;
use tauri::Manager;

#[tauri::command]
pub fn get_preferences(data_root: tauri::State<'_, SharedDataRoot>) -> AppResult<PreferencesDto> {
    let mut dto: PreferencesDto = load_preferences_with_path(&data_root.0)?.into();
    // 旧偏好文件缺 theme 字段时给前端一次性导入信号（H-15）：让历史 localStorage
    // 主题被提交一次，之后本机缓存跟随后端值。
    dto.theme_needs_import = !preferences_file_has_theme(&data_root.0);
    if cfg!(windows) {
        dto.cli_enabled = false;
    }
    Ok(dto)
}

/// 判断偏好文件是否已显式包含 theme 字段；缺文件按需要处理（更保守）。
fn preferences_file_has_theme(data_root: &std::path::Path) -> bool {
    let path = data_root.join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH);
    match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice::<serde_json::Value>(&bytes)
            .ok()
            .and_then(|value| value.get("theme").cloned())
            .is_some(),
        Err(_) => false,
    }
}

#[tauri::command]
pub async fn save_preferences(
    preferences: PreferencesDto,
    data_root: tauri::State<'_, SharedDataRoot>,
    app: tauri::AppHandle,
) -> AppResult<PreferencesDto> {
    // 命令边界收到的是 WebView 的 camelCase DTO；显式转换为领域结构后再落盘，
    // 避免 camelCase 字段被丢弃、保存被静默改写为默认值。
    let domain: Preferences = preferences.into();
    if cfg!(windows) && domain.cli_enabled {
        return Err(AppError::RuntimeManaged {
            code: "cli_not_supported".to_string(),
            detail: "Windows 暂不支持 CLI 控制面；本机 IPC 未实现".to_string(),
        });
    }
    let root = data_root.0.clone();
    let state = app
        .state::<crate::commands::update::SharedUpdateStatus>()
        .inner()
        .clone();
    let event_app = app.clone();
    let saved = crate::commands::run_blocking("save preferences", move || {
        // All writers take the filesystem lock before the query-state lock.
        let _transaction = crate::modules::backup::manager::acquire_preferences_transaction(&root)?;
        let mut status = state.lock().map_err(|_| AppError::NotConfigured)?;
        save_preferences_observed(&root, &domain, &mut status, |candidate, succeeded| {
            publish_preferences_result(&event_app, &root, "preferences-save", candidate, succeeded);
        })
    })
    .await?;
    crate::infrastructure::window_appearance::refresh(&app);
    Ok(saved.into())
}

#[tauri::command]
pub async fn restore_default_preferences(
    data_root: tauri::State<'_, SharedDataRoot>,
    app: tauri::AppHandle,
) -> AppResult<PreferencesDto> {
    let root = data_root.0.clone();
    let state = app
        .state::<crate::commands::update::SharedUpdateStatus>()
        .inner()
        .clone();
    let event_app = app.clone();
    let restored = crate::commands::run_blocking("restore default preferences", move || {
        let _transaction = crate::modules::backup::manager::acquire_preferences_transaction(&root)?;
        let mut status = state.lock().map_err(|_| AppError::NotConfigured)?;
        save_preferences_observed(
            &root,
            &Preferences::default(),
            &mut status,
            |candidate, succeeded| {
                publish_preferences_result(
                    &event_app,
                    &root,
                    "preferences-reset",
                    candidate,
                    succeeded,
                );
            },
        )
    })
    .await?;
    crate::infrastructure::window_appearance::refresh(&app);
    Ok(restored.into())
}

pub fn load_preferences_with_path(data_root: &std::path::Path) -> AppResult<Preferences> {
    PreferencesStore::new(data_root)
        .load()
        .map_err(AppError::from)
}

/// Disk and query ownership change together. Refusal or failed save changes neither.
pub(crate) fn save_preferences_transaction(
    root: &std::path::Path,
    value: &Preferences,
    status: &mut crate::modules::update::UpdateStatus,
) -> AppResult<Preferences> {
    let channel = preflight_preferences(value, status)?;
    commit_preferences(root, value, status, channel)
}

fn preflight_preferences(
    value: &Preferences,
    status: &crate::modules::update::UpdateStatus,
) -> AppResult<crate::modules::update::UpdateChannel> {
    crate::modules::preferences::validate(value)?;
    let channel = crate::modules::update::UpdateChannel::parse(&value.app_update_channel)
        .ok_or(AppError::NotConfigured)?;
    if (status.installing || status.pending_restart.is_some()) && status.channel != channel {
        return Err(AppError::NotConfigured);
    }
    Ok(channel)
}

/// Called under the admitted worker and filesystem/query locks. Validation and
/// ownership refusals do not write event scopes or persistent history. The
/// observer owns only delivery, never the result of the preference mutation.
fn save_preferences_observed(
    root: &std::path::Path,
    value: &Preferences,
    status: &mut crate::modules::update::UpdateStatus,
    observer: impl FnOnce(&[u8], bool),
) -> AppResult<Preferences> {
    let channel = preflight_preferences(value, status)?;
    // Struct field order is deterministic; candidates include all settings,
    // including the update channel. Only the hash leaves this owned callback.
    let candidate = serde_json::to_vec(value).map_err(|_| AppError::NotConfigured)?;
    let result = commit_preferences(root, value, status, channel);
    observer(&candidate, result.is_ok());
    result
}

fn publish_preferences_result(
    app: &tauri::AppHandle,
    root: &std::path::Path,
    stem: &str,
    candidate: &[u8],
    succeeded: bool,
) {
    use crate::modules::notifications::registry::{Channel, Trigger};
    let failed = format!("{stem}-failed");
    let identity =
        crate::commands::event_delivery::prepare(root, &failed, Channel::Local, candidate);
    let event = format!("{stem}-{}", if succeeded { "succeeded" } else { "failed" });
    crate::commands::event_delivery::publish(app, root, &event, identity, Trigger::User);
}

fn commit_preferences(
    root: &std::path::Path,
    value: &Preferences,
    status: &mut crate::modules::update::UpdateStatus,
    channel: crate::modules::update::UpdateChannel,
) -> AppResult<Preferences> {
    let saved = save_preferences_with_path(root, value)?;
    status.switch_channel(channel);
    Ok(saved)
}

pub fn save_preferences_with_path(
    data_root: &std::path::Path,
    value: &Preferences,
) -> AppResult<Preferences> {
    PreferencesStore::new(data_root)
        .save(value)
        .map_err(AppError::from)
}

pub fn restore_preferences_with_path(data_root: &std::path::Path) -> AppResult<Preferences> {
    save_preferences_with_path(data_root, &Preferences::default())
}

impl From<crate::modules::preferences::PreferencesError> for AppError {
    fn from(value: crate::modules::preferences::PreferencesError) -> Self {
        value.as_app_error()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::notifications::{
        persistence::{load_notifications, notifications_path},
        registry::{self, Channel, Delivery, Evidence, Trigger},
        NotificationStore,
    };
    use crate::types::preferences::PreferencesDto;
    use std::sync::{Arc, Mutex};

    fn observed_save(
        root: &std::path::Path,
        value: &Preferences,
        status: &mut crate::modules::update::UpdateStatus,
        store: &Arc<Mutex<NotificationStore>>,
        stem: &str,
    ) -> AppResult<Preferences> {
        let _transaction = crate::modules::backup::manager::acquire_preferences_transaction(root)?;
        save_preferences_observed(root, value, status, |bytes, succeeded| {
            let failed = format!("{stem}-failed");
            let identity =
                crate::commands::event_delivery::prepare(root, &failed, Channel::Local, bytes)
                    .unwrap();
            let event = format!("{stem}-{}", if succeeded { "succeeded" } else { "failed" });
            let evidence = if succeeded {
                Evidence::Success {
                    candidate: identity.candidate.clone(),
                    verified: true,
                }
            } else {
                Evidence::Failure
            };
            let delivery = Delivery {
                event: &event,
                job: registry::lookup(&event).unwrap().job,
                trigger: Trigger::User,
                identity,
                evidence,
                occurred_at: chrono::Utc::now(),
            };
            // Delivery errors deliberately do not change the save result.
            let _ = crate::commands::notifications::NotificationPublisher {
                store,
                data_root: root,
            }
            .publish_event(&delivery);
        })
    }

    #[test]
    fn real_save_failures_recover_exact_contents_and_keep_reset_independent() {
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        let path = root
            .path()
            .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH);
        std::fs::create_dir_all(&path).unwrap();
        let value = Preferences {
            theme: "dark".into(),
            ..Preferences::default()
        };
        let mut status = crate::modules::update::UpdateStatus::pending("0.1.9");
        let store = Arc::new(Mutex::new(NotificationStore::new()));
        assert!(
            observed_save(root.path(), &value, &mut status, &store, "preferences-save").is_err()
        );
        assert!(observed_save(
            root.path(),
            &Preferences::default(),
            &mut status,
            &store,
            "preferences-reset"
        )
        .is_err());
        let restarted = Arc::new(Mutex::new(
            load_notifications(&notifications_path(root.path())).unwrap(),
        ));
        assert_eq!(restarted.lock().unwrap().all().len(), 2);
        std::fs::remove_dir(&path).unwrap();
        observed_save(
            root.path(),
            &value,
            &mut status,
            &restarted,
            "preferences-save",
        )
        .unwrap();
        let loaded = load_notifications(&notifications_path(root.path())).unwrap();
        assert!(loaded.all()[0].resolved && !loaded.all()[0].read);
        assert!(!loaded.all()[1].resolved);
        observed_save(
            root.path(),
            &Preferences::default(),
            &mut status,
            &restarted,
            "preferences-reset",
        )
        .unwrap();
        let loaded = load_notifications(&notifications_path(root.path())).unwrap();
        assert_eq!(loaded.all().len(), 2);
        assert!(loaded.all().iter().all(|n| n.resolved && !n.read));
        let history = std::fs::read_to_string(notifications_path(root.path())).unwrap();
        assert!(!history.contains(root.path().to_str().unwrap()));
        assert!(!history.contains("dark"));
        assert_eq!(
            load_preferences_with_path(root.path()).unwrap(),
            Preferences::default()
        );
    }

    #[test]
    fn changed_channel_candidate_cannot_clear_save_failure() {
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        let path = root
            .path()
            .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH);
        std::fs::create_dir_all(&path).unwrap();
        let store = Arc::new(Mutex::new(NotificationStore::new()));
        let mut status = crate::modules::update::UpdateStatus::pending("0.1.9");
        assert!(observed_save(
            root.path(),
            &Preferences::default(),
            &mut status,
            &store,
            "preferences-save"
        )
        .is_err());
        std::fs::remove_dir(&path).unwrap();
        let beta = Preferences {
            app_update_channel: "beta".into(),
            ..Preferences::default()
        };
        observed_save(root.path(), &beta, &mut status, &store, "preferences-save").unwrap();
        assert!(
            !load_notifications(&notifications_path(root.path()))
                .unwrap()
                .all()[0]
                .resolved
        );
        assert_eq!(status.channel.as_str(), "beta");
    }

    #[test]
    fn preflight_refusals_do_not_deliver_or_create_event_files() {
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        let snapshot = || {
            std::fs::read_dir(root.path().join("manager-state"))
                .unwrap()
                .map(|entry| {
                    let entry = entry.unwrap();
                    (entry.file_name(), std::fs::read(entry.path()).unwrap())
                })
                .collect::<std::collections::BTreeMap<_, _>>()
        };
        let before = snapshot();
        let mut status = crate::modules::update::UpdateStatus::pending("0.1.9");
        let invalid = Preferences {
            interface_scale: -1,
            ..Preferences::default()
        };
        let beta = Preferences {
            app_update_channel: "beta".into(),
            ..Preferences::default()
        };
        assert!(
            save_preferences_observed(root.path(), &invalid, &mut status, |_, _| panic!(
                "invalid value delivered"
            ))
            .is_err()
        );
        status.installing = true;
        assert!(
            save_preferences_observed(root.path(), &beta, &mut status, |_, _| panic!(
                "busy change delivered"
            ))
            .is_err()
        );
        status.installing = false;
        status.pending_restart = Some("0.1.10".into());
        assert!(
            save_preferences_observed(root.path(), &beta, &mut status, |_, _| panic!(
                "pending restart change delivered"
            ))
            .is_err()
        );
        assert!(!notifications_path(root.path()).exists());
        assert_eq!(snapshot(), before);
    }

    #[test]
    fn failed_notification_persistence_does_not_fail_committed_preferences() {
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        let path = root
            .path()
            .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH);
        std::fs::create_dir_all(&path).unwrap();
        let store = Arc::new(Mutex::new(NotificationStore::new()));
        let mut status = crate::modules::update::UpdateStatus::pending("0.1.9");
        assert!(observed_save(
            root.path(),
            &Preferences::default(),
            &mut status,
            &store,
            "preferences-save"
        )
        .is_err());
        std::fs::remove_dir(&path).unwrap();
        let notifications = notifications_path(root.path());
        std::fs::remove_file(&notifications).unwrap();
        std::fs::create_dir(&notifications).unwrap();
        observed_save(
            root.path(),
            &Preferences::default(),
            &mut status,
            &store,
            "preferences-save",
        )
        .unwrap();
        assert_eq!(
            load_preferences_with_path(root.path()).unwrap(),
            Preferences::default()
        );
        assert!(!store.lock().unwrap().all()[0].resolved);
    }

    #[test]
    fn channel_save_invalidates_old_checks_only_after_success() {
        use crate::modules::update::{UpdateChannel, UpdateStatus};
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        let mut status = UpdateStatus::pending("0.1.9");
        let old = status.begin_check().unwrap();
        let value = Preferences {
            app_update_channel: "beta".into(),
            ..Preferences::default()
        };
        save_preferences_transaction(root.path(), &value, &mut status).unwrap();
        assert_eq!(status.channel, UpdateChannel::Beta);
        assert!(!status.finish_check(old));
        status.installing = true;
        assert!(
            save_preferences_transaction(root.path(), &Preferences::default(), &mut status)
                .is_err()
        );
        assert_eq!(
            load_preferences_with_path(root.path())
                .unwrap()
                .app_update_channel,
            "beta"
        );
        status.installing = false;
        let bad = Preferences {
            interface_scale: -1,
            ..Preferences::default()
        };
        assert!(save_preferences_transaction(root.path(), &bad, &mut status).is_err());
        assert_eq!(status.channel, UpdateChannel::Beta);
    }

    #[test]
    fn prepared_update_refuses_channel_change_before_writing_disk() {
        use crate::modules::update::UpdateStatus;
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        save_preferences_with_path(root.path(), &Preferences::default()).unwrap();
        let path = root
            .path()
            .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH);
        let original = std::fs::read(&path).unwrap();
        let mut status = UpdateStatus::pending("0.1.9");
        status.pending_restart = Some("0.1.10".into());
        let beta = Preferences {
            app_update_channel: "beta".into(),
            ..Preferences::default()
        };
        assert!(save_preferences_transaction(root.path(), &beta, &mut status).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), original);
        assert_eq!(status.channel.as_str(), "stable");
        let dark = Preferences {
            theme: "dark".into(),
            ..Preferences::default()
        };
        save_preferences_transaction(root.path(), &dark, &mut status).unwrap();
        assert_eq!(status.pending_restart.as_deref(), Some("0.1.10"));
    }

    // 回归：WebView 传入 camelCase 的完整偏好，保存后重新读取，取值必须保持一致，
    // 不能再被静默改写为默认值（此前所有开关/选择器/缩放都会回滚）。
    #[test]
    fn camel_case_preferences_round_trip_through_disk() {
        let root = tempfile::tempdir().expect("temp root");
        crate::modules::data_root::initialize(root.path()).expect("initialize");

        let dto = PreferencesDto {
            schema_version: crate::modules::config_migration::PREFERENCES_CURRENT_SCHEMA,
            theme_needs_import: false,
            interface_scale: 175,
            launch_main: false,
            auto_panel: false,
            panel_mode: "browser".into(),
            keep_proxy_on_close: false,
            lifecycle_notifications: false,
            sync_conflict_alerts: false,
            launch_with_codex: false,
            auto_backup_upgrade: false,
            auto_backup_import: false,
            auto_backup_sync: false,
            backup_retention: "20".into(),
            backup_integrity: "blake3".into(),
            backup_include_skills: false,
            export_include_skills: false,
            mcp_conflict_policy: "keep-both".into(),
            mcp_mask: false,
            backup_include_mcp: false,
            export_include_mcp: false,
            log_retention: "7d-5000".into(),
            notification_retention: "7".into(),
            startup_cleanup: false,
            cleanup_backup_summary: false,
            cli_enabled: true,
            sync_conflict_policy: "keep-remote".into(),
            cold_sync: false,
            backup_before_overwrite: false,
            app_update_channel: "beta".into(),
            app_update_auto_check: false,
            app_update_check_interval_seconds: 21600,
            theme: "dark".into(),
            network_proxy_mode: "manual".into(),
            network_proxy_scheme: "socks5h".into(),
            network_proxy_host: "127.0.0.1:1080".into(),
            network_no_proxy: "localhost,127.0.0.1".into(),
            visual_effects: "mid".into(),
            glow_render: "css".into(),
        };

        let domain: Preferences = dto.into();
        let saved: PreferencesDto = save_preferences_with_path(root.path(), &domain)
            .expect("save")
            .into();
        let reloaded: PreferencesDto = load_preferences_with_path(root.path())
            .expect("load")
            .into();

        assert_eq!(saved, reloaded);
        assert_eq!(reloaded.interface_scale, 175);
        assert!(!reloaded.launch_main);
        assert_eq!(reloaded.panel_mode, "browser");
        assert_eq!(reloaded.backup_retention, "20");
        assert!(reloaded.cli_enabled);
        assert_eq!(reloaded.app_update_channel, "beta");
        assert!(!reloaded.app_update_auto_check);
        assert_eq!(reloaded.app_update_check_interval_seconds, 21600);
        assert_eq!(reloaded.theme, "dark");
        assert_eq!(reloaded.visual_effects, "mid");
        assert_eq!(reloaded.glow_render, "css");
    }
}
