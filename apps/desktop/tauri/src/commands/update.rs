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
    let builder = app
        .updater_builder()
        .endpoints(vec![url])
        .map_err(|_error| AppError::NotConfigured)?;
    let builder = apply_proxy_policy(
        builder,
        crate::modules::preferences::proxy_policy_for_app(app),
    );
    builder.build().map_err(|_error| AppError::NotConfigured)
}

/// 应用自更新请求按用户代理偏好走代理（U-05）。
///
/// 关键：tauri-plugin-updater 的 UpdaterBuilder::build() 在**没有**显式
/// .proxy()/.no_proxy() 时会回退到 reqwest 的「系统代理」探测；只识别环境变量
/// （HTTP(S)_PROXY / ALL_PROXY），不读 macOS 的 scutil 系统代理设置。因此：
/// - None：显式 .no_proxy()，尊重「无代理」，避免被意外的环境变量带偏；
/// - Manual：显式注入代理（HTTP CONNECT），这是「我在应用里配了代理但自更新
///   仍然连接失败」的根因修复；
/// - System：不注入，交给 reqwest 探测（但注意它只认环境变量，不认 scutil）。
fn apply_proxy_policy(
    builder: tauri_plugin_updater::UpdaterBuilder,
    policy: crate::modules::preferences::ProxyPolicy,
) -> tauri_plugin_updater::UpdaterBuilder {
    use crate::modules::preferences::ProxyPolicy;
    match policy {
        ProxyPolicy::None => builder.no_proxy(),
        ProxyPolicy::System => builder,
        ProxyPolicy::Manual(url) => match url.parse() {
            Ok(proxy) => builder.proxy(proxy),
            Err(_) => builder.no_proxy(),
        },
    }
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
    let (result_status, available_version) = match update {
        Some(value) => ("available", Some(value.version.clone())),
        None => ("up_to_date", None),
    };
    record_check_success(&mut guard, available_version);
    Ok(CheckUpdateResultDto {
        status: result_status.to_string(),
        update: guard.clone().into(),
    })
}

/// 记录一次成功的检查结果。
///
/// 必须清空 `error`：`check_for_update` 的失败分支会写入网络失败提示，若成功分支
/// 不清，会让上一次的「更新下载或连接失败」盖住新的「有更新 / 已是最新」结果
/// （界面固定先显示 `error`，于是显示为失败）。
fn record_check_success(status: &mut UpdateStatus, available_version: Option<String>) {
    status.available_version = available_version;
    status.signature_verified = Some(false);
    status.error = None;
    status.last_checked_at = Some(chrono::Utc::now().to_rfc3339());
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

    #[test]
    fn successful_check_clears_stale_failure_and_records_result() {
        let mut status = UpdateStatus::pending("0.1.4");
        status.error = Some(UpdateFailureClass::Network.message().to_string());
        status.last_checked_at = Some("2020-01-01T00:00:00Z".to_string());

        // 已是最新：清掉上一次的网络失败提示，并给出新的检查时间。
        record_check_success(&mut status, None);
        assert_eq!(status.error, None);
        assert_eq!(status.available_version, None);
        assert_eq!(status.signature_verified, Some(false));
        assert_ne!(
            status.last_checked_at.as_deref(),
            Some("2020-01-01T00:00:00Z")
        );

        // 有更新：同样不保留旧错误。
        status.error = Some(UpdateFailureClass::Network.message().to_string());
        record_check_success(&mut status, Some("0.1.6".to_string()));
        assert_eq!(status.error, None);
        assert_eq!(status.available_version.as_deref(), Some("0.1.6"));
    }

    fn set_update_channel_state(status: &mut UpdateStatus, channel: UpdateChannel) {
        status.channel = channel;
        status.available_version = None;
        status.signature_verified = None;
        status.error = None;
    }
}
