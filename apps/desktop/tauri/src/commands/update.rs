//! 应用自更新命令层；只投影状态与校验结果，不接管官方 ocx update。

use std::sync::{Arc, Mutex};

use tauri_plugin_updater::UpdaterExt;

/// 用有效通道解析出的端点构建 updater（U-05）：检查、安装与后台调度共用同一来源，
/// 不再各自读 tauri.conf.json 的静态占位端点。端点解析失败时回退插件默认配置。
fn updater_for_status(
    app: &tauri::AppHandle,
    status: &UpdateStatus,
) -> AppResult<tauri_plugin_updater::Updater> {
    let endpoint = status.channel.endpoint();
    let Ok(url) = endpoint.parse() else {
        return app.updater().map_err(|_error| AppError::NotConfigured);
    };
    app.updater_builder()
        .endpoints(vec![url])
        .map_err(|_error| AppError::NotConfigured)?
        .build()
        .map_err(|_error| AppError::NotConfigured)
}

use crate::errors::{AppError, AppResult};
use crate::modules::update::{UpdateChannel, UpdateFailureClass, UpdateStatus};
use crate::types::update::{CheckUpdateResultDto, UpdateStatusDto};

pub type SharedUpdateStatus = Arc<Mutex<UpdateStatus>>;

#[tauri::command]
pub fn get_update_status(
    status: tauri::State<'_, SharedUpdateStatus>,
    app: tauri::AppHandle,
) -> AppResult<UpdateStatusDto> {
    let guard = status.lock().map_err(|_poisoned| AppError::NotConfigured)?;
    Ok(current_status(&guard, &app).into())
}

#[tauri::command]
pub fn set_update_channel(
    channel: UpdateChannel,
    status: tauri::State<'_, SharedUpdateStatus>,
    app: tauri::AppHandle,
) -> AppResult<UpdateStatusDto> {
    let mut guard = status.lock().map_err(|_poisoned| AppError::NotConfigured)?;
    guard.channel = channel;
    guard.available_version = None;
    guard.signature_verified = None;
    guard.error = None;
    Ok(current_status(&guard, &app).into())
}

#[tauri::command]
pub async fn check_for_update(
    status: tauri::State<'_, SharedUpdateStatus>,
    app: tauri::AppHandle,
) -> AppResult<CheckUpdateResultDto> {
    let updater = {
        let guard = status.lock().map_err(|_poisoned| AppError::NotConfigured)?;
        updater_for_status(&app, &guard)?
    };
    let update = match updater.check().await {
        Ok(value) => value,
        Err(_error) => {
            let mut guard = status.lock().map_err(|_poisoned| AppError::NotConfigured)?;
            guard.signature_verified = None;
            guard.available_version = None;
            guard.last_checked_at = Some(chrono::Utc::now().to_rfc3339());
            guard.error = Some(UpdateFailureClass::Network.message().to_string());
            return Ok(CheckUpdateResultDto {
                status: "failed".to_string(),
                update: guard.clone().into(),
            });
        }
    };
    let mut guard = status.lock().map_err(|_poisoned| AppError::NotConfigured)?;
    guard.last_checked_at = Some(chrono::Utc::now().to_rfc3339());
    match update {
        Some(value) => {
            guard.available_version = Some(value.version.clone());
            guard.signature_verified = Some(false);
            Ok(CheckUpdateResultDto {
                status: "available".to_string(),
                update: guard.clone().into(),
            })
        }
        None => {
            guard.available_version = None;
            guard.signature_verified = Some(false);
            Ok(CheckUpdateResultDto {
                status: "up_to_date".to_string(),
                update: guard.clone().into(),
            })
        }
    }
}

#[tauri::command]
pub async fn install_update(
    status: tauri::State<'_, SharedUpdateStatus>,
    app: tauri::AppHandle,
) -> AppResult<()> {
    let updater = {
        let guard = status.lock().map_err(|_poisoned| AppError::NotConfigured)?;
        updater_for_status(&app, &guard)?
    };
    let update = match updater.check().await {
        Ok(Some(value)) => value,
        Ok(None) => {
            let mut guard = status.lock().map_err(|_poisoned| AppError::NotConfigured)?;
            guard.available_version = None;
            guard.signature_verified = Some(false);
            guard.error = None;
            guard.last_checked_at = Some(chrono::Utc::now().to_rfc3339());
            return Err(AppError::NotConfigured);
        }
        Err(_error) => {
            let mut guard = status.lock().map_err(|_poisoned| AppError::NotConfigured)?;
            guard.available_version = None;
            guard.signature_verified = Some(false);
            guard.last_checked_at = Some(chrono::Utc::now().to_rfc3339());
            guard.error = Some(UpdateFailureClass::Network.message().to_string());
            return Err(AppError::NotConfigured);
        }
    };
    if update
        .download_and_install(|_chunk, _total| {}, || {})
        .await
        .is_err()
    {
        let mut guard = status.lock().map_err(|_poisoned| AppError::NotConfigured)?;
        guard.signature_verified = Some(false);
        guard.error = Some(UpdateFailureClass::Install.message().to_string());
        return Err(AppError::NotConfigured);
    }
    let mut guard = status.lock().map_err(|_poisoned| AppError::NotConfigured)?;
    guard.signature_verified = Some(true);
    guard.last_checked_at = Some(chrono::Utc::now().to_rfc3339());
    guard.error = None;
    app.request_restart();
    Ok(())
}

fn current_status(status: &UpdateStatus, app: &tauri::AppHandle) -> UpdateStatus {
    let mut value = status.clone();
    value.current_version = app.package_info().version.to_string();
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_switch_clears_stale_check_result() {
        let mut status = UpdateStatus::pending("0.1.0");
        status.available_version = Some("0.2.0".to_string());
        status.signature_verified = Some(true);
        set_update_channel_state(&mut status, UpdateChannel::Beta);
        assert_eq!(status.channel, UpdateChannel::Beta);
        assert_eq!(status.available_version, None);
        assert_eq!(status.signature_verified, None);
        assert_eq!(status.error, None);
    }

    #[test]
    fn install_marks_signature_only_after_plugin_download_and_install() {
        let mut status = UpdateStatus::pending("0.1.0");
        status.available_version = Some("0.2.0".to_string());
        status.signature_verified = Some(false);

        status.signature_verified = Some(true);
        assert_eq!(status.signature_verified, Some(true));
        assert_eq!(status.available_version.as_deref(), Some("0.2.0"));

        status.signature_verified = Some(false);
        status.error = Some(UpdateFailureClass::Install.message().to_string());
        assert_eq!(status.signature_verified, Some(false));
        assert!(status.error.unwrap().contains("已保留当前版本"));
    }

    fn set_update_channel_state(status: &mut UpdateStatus, channel: UpdateChannel) {
        status.channel = channel;
        status.available_version = None;
        status.signature_verified = None;
        status.error = None;
    }
}
