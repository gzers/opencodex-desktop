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
pub fn save_preferences(
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
    let state = app.state::<crate::commands::update::SharedUpdateStatus>();
    let mut status = state.lock().map_err(|_| AppError::NotConfigured)?;
    let saved = save_preferences_transaction(&data_root.0, &domain, &mut status)?;
    drop(status);
    crate::infrastructure::window_appearance::refresh(&app);
    Ok(saved.into())
}

#[tauri::command]
pub fn restore_default_preferences(
    data_root: tauri::State<'_, SharedDataRoot>,
    app: tauri::AppHandle,
) -> AppResult<PreferencesDto> {
    let state = app.state::<crate::commands::update::SharedUpdateStatus>();
    let mut status = state.lock().map_err(|_| AppError::NotConfigured)?;
    let restored = save_preferences_transaction(&data_root.0, &Preferences::default(), &mut status)?;
    drop(status);
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
    let channel = crate::modules::update::UpdateChannel::parse(&value.app_update_channel)
        .ok_or(AppError::NotConfigured)?;
    if status.installing && status.channel != channel {
        return Err(AppError::NotConfigured);
    }
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
    use crate::types::preferences::PreferencesDto;

    #[test]
    fn channel_save_invalidates_old_checks_only_after_success() {
        use crate::modules::update::{UpdateChannel, UpdateStatus};
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        let mut status = UpdateStatus::pending("0.1.9");
        let old = status.begin_check().unwrap();
        let value = Preferences { app_update_channel: "beta".into(), ..Preferences::default() };
        save_preferences_transaction(root.path(), &value, &mut status).unwrap();
        assert_eq!(status.channel, UpdateChannel::Beta);
        assert!(!status.finish_check(old));
        status.installing = true;
        assert!(save_preferences_transaction(root.path(), &Preferences::default(), &mut status).is_err());
        assert_eq!(load_preferences_with_path(root.path()).unwrap().app_update_channel, "beta");
        status.installing = false;
        let bad = Preferences { interface_scale: -1, ..Preferences::default() };
        assert!(save_preferences_transaction(root.path(), &bad, &mut status).is_err());
        assert_eq!(status.channel, UpdateChannel::Beta);
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
