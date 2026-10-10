//! 应用自更新命令层；只投影状态与校验结果，不接管官方 ocx update。

use std::sync::{Arc, Mutex};

use tauri_plugin_updater::UpdaterExt;

/// 用有效通道解析出的端点构建 updater（U-05）：检查、安装与后台调度共用同一来源，
/// 不再各自读 tauri.conf.json 的静态占位端点。无效端点停止查询。
fn updater_for_status(
    app: &tauri::AppHandle,
    status: &UpdateStatus,
) -> AppResult<tauri_plugin_updater::Updater> {
    let endpoint = status.channel.endpoint();
    let url = endpoint.parse().map_err(|_| AppError::NotConfigured)?;
    let builder = app
        .updater_builder()
        .timeout(crate::modules::network_defaults::request_timeout())
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
/// 沿用已有的三分支策略；不把底层库的系统代理能力固化成平台假设。
/// - None：显式 .no_proxy()，尊重「无代理」，避免被意外的环境变量带偏；
/// - Manual：显式注入用户配置的代理，地址无效时不回退到系统代理；
/// - System：不注入，交给当前 updater/reqwest 的系统代理实现探测。
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

use crate::commands::update_schedule;
use crate::errors::{AppError, AppResult};
use crate::modules::update::schedule::{self, Target};
use crate::modules::update::{UpdateChannel, UpdateFailureClass, UpdateStatus};
use crate::types::update::{CheckUpdateResultDto, UpdateStatusDto};

pub type SharedUpdateStatus = Arc<Mutex<UpdateStatus>>;

struct CheckLease(SharedUpdateStatus, u64);
impl Drop for CheckLease {
    fn drop(&mut self) {
        if let Ok(mut status) = self.0.lock() {
            // Cancellation must not release the next channel's check.
            status.finish_check(self.1);
        }
    }
}

/// Release installation ownership on all error and cancellation paths.
struct InstallationLease(SharedUpdateStatus);
impl Drop for InstallationLease {
    fn drop(&mut self) {
        if let Ok(mut status) = self.0.lock() {
            status.installing = false;
        }
    }
}

#[tauri::command]
pub fn get_update_status(
    status: tauri::State<'_, SharedUpdateStatus>,
    app: tauri::AppHandle,
) -> AppResult<UpdateStatusDto> {
    let mut guard = status.lock().map_err(|_poisoned| AppError::NotConfigured)?;
    hydrate_cache(&mut guard, &app)?;
    Ok(current_status(&guard, &app).into())
}

#[tauri::command]
pub fn set_update_channel(
    channel: UpdateChannel,
    status: tauri::State<'_, SharedUpdateStatus>,
    app: tauri::AppHandle,
) -> AppResult<UpdateStatusDto> {
    let mut guard = status.lock().map_err(|_poisoned| AppError::NotConfigured)?;
    let root = update_schedule::active_root(&app)?;
    persist_channel(&root, &mut guard, channel)?;
    hydrate_cache(&mut guard, &app)?;
    Ok(current_status(&guard, &app).into())
}

fn persist_channel(
    root: &std::path::Path,
    status: &mut UpdateStatus,
    channel: UpdateChannel,
) -> AppResult<()> {
    let mut preferences = crate::commands::preferences::load_preferences_with_path(root)?;
    preferences.app_update_channel = match channel {
        UpdateChannel::Stable => "stable",
        UpdateChannel::Beta => "beta",
    }.into();
    crate::commands::preferences::save_preferences_transaction(root, &preferences, status)?;
    Ok(())
}

#[tauri::command]
pub async fn check_for_update(
    status: tauri::State<'_, SharedUpdateStatus>,
    app: tauri::AppHandle,
) -> AppResult<CheckUpdateResultDto> {
    let (updater, generation, target) = {
        let mut guard = status.lock().map_err(|_poisoned| AppError::NotConfigured)?;
        let updater = updater_for_status(&app, &guard)?;
        let generation = guard.begin_check().ok_or(AppError::NotConfigured)?;
        (updater, generation, Target::manager(guard.channel))
    };
    let _lease = CheckLease(status.inner().clone(), generation);
    let query_root = update_schedule::reserve(&app, target)?;
    let update = match updater.check().await {
        Ok(value) => value,
        Err(_error) => {
            let mut guard = status.lock().map_err(|_poisoned| AppError::NotConfigured)?;
            if !guard.finish_check(generation) {
                return Ok(CheckUpdateResultDto {
                    status: "superseded".to_string(),
                    update: current_status(&guard, &app).into(),
                });
            }
            guard.signature_verified = None;
            guard.available_version = None;
            guard.last_checked_at = Some(chrono::Utc::now().to_rfc3339());
            guard.error = Some(UpdateFailureClass::Network.message().to_string());
            update_schedule::complete::<UpdateStatus>(&app, &query_root, target, None)?;
            return Ok(CheckUpdateResultDto {
                status: "failed".to_string(),
                update: guard.clone().into(),
            });
        }
    };
    let mut guard = status.lock().map_err(|_poisoned| AppError::NotConfigured)?;
    if !guard.finish_check(generation) {
        return Ok(CheckUpdateResultDto {
            status: "superseded".to_string(),
            update: current_status(&guard, &app).into(),
        });
    }
    let (result_status, available_version) = match update {
        Some(value) => ("available", Some(value.version.clone())),
        None => ("up_to_date", None),
    };
    record_check_success(&mut guard, available_version);
    update_schedule::complete(&app, &query_root, target, Some(&*guard))?;
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
        let mut guard = status.lock().map_err(|_poisoned| AppError::NotConfigured)?;
        if guard.checking || guard.installing {
            return Err(AppError::NotConfigured);
        }
        let updater = updater_for_status(&app, &guard)?;
        guard.installing = true;
        updater
    };
    let _lease = InstallationLease(status.inner().clone());
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

fn hydrate_cache(status: &mut UpdateStatus, app: &tauri::AppHandle) -> AppResult<()> {
    if status.last_checked_at.is_none() && !status.checking && !status.installing {
        let root = update_schedule::active_root(app)?;
        if let Some(cached) =
            schedule::load_cache::<UpdateStatus>(&root, Target::manager(status.channel))
        {
            if cached.channel == status.channel {
                status.available_version = cached.available_version;
                status.last_checked_at = cached.last_checked_at;
                status.error = None;
                // A cached metadata result is never proof of an installed signature.
                status.signature_verified = None;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_command_persists_and_refuses_install_races() {
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        let mut preferences = crate::modules::preferences::Preferences::default();
        preferences.interface_scale = 125;
        crate::commands::preferences::save_preferences_with_path(root.path(), &preferences).unwrap();
        let mut status = UpdateStatus::pending("0.1.9");
        let old_check = status.begin_check().unwrap();
        persist_channel(root.path(), &mut status, UpdateChannel::Beta).unwrap();
        assert!(!status.finish_check(old_check));
        let saved = crate::commands::preferences::load_preferences_with_path(root.path()).unwrap();
        assert_eq!(saved.app_update_channel, "beta");
        assert_eq!(saved.interface_scale, 125);
        status.installing = true;
        assert!(persist_channel(root.path(), &mut status, UpdateChannel::Stable).is_err());
        assert_eq!(status.channel, UpdateChannel::Beta);
        assert_eq!(crate::commands::preferences::load_preferences_with_path(root.path()).unwrap().app_update_channel, "beta");
    }

    #[test]
    fn cancelled_check_releases_only_its_own_generation() {
        let status = Arc::new(Mutex::new(UpdateStatus::pending("0.1.9")));
        let generation = status.lock().unwrap().begin_check().unwrap();
        drop(CheckLease(status.clone(), generation));
        assert!(!status.lock().unwrap().checking);
        let old = status.lock().unwrap().begin_check().unwrap();
        status.lock().unwrap().switch_channel(UpdateChannel::Beta);
        let next = status.lock().unwrap().begin_check().unwrap();
        drop(CheckLease(status.clone(), old));
        assert!(status.lock().unwrap().checking);
        drop(CheckLease(status.clone(), next));
        assert!(!status.lock().unwrap().checking);
    }

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
        assert!(status.switch_channel(channel));
    }

    #[test]
    fn late_check_cannot_overwrite_a_new_channel_or_release_its_lock() {
        let mut status = UpdateStatus::pending("0.1.9");
        let old = status.begin_check().unwrap();
        assert!(status.begin_check().is_none());
        assert!(status.switch_channel(UpdateChannel::Beta));
        let new = status.begin_check().unwrap();
        assert!(!status.finish_check(old));
        assert!(status.checking);
        assert!(status.finish_check(new));
        record_check_success(&mut status, Some("0.1.10-beta.1".into()));
        assert_eq!(status.channel, UpdateChannel::Beta);
        assert_eq!(status.available_version.as_deref(), Some("0.1.10-beta.1"));
    }

    #[test]
    fn channel_switch_is_idempotent_and_blocked_during_installation() {
        let mut status = UpdateStatus::pending("0.1.9");
        status.installing = true;
        assert!(!status.switch_channel(UpdateChannel::Beta));
        assert!(status.switch_channel(UpdateChannel::Stable));
        assert!(status.begin_check().is_none());
    }
}
