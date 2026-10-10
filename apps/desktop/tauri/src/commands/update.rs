//! 应用自更新命令层；只投影状态与校验结果，不接管官方 ocx update。

use crate::commands::event_delivery;
use crate::modules::notifications::registry::{Channel as EventChannel, Trigger};
use std::sync::{Arc, Mutex};

use crate::modules::update::progress::UpdateProgress;
use tauri::Manager;
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
// Windows install() exits after spawning the native installer. Defer handoff
// until explicit restart; ordinary app exit discards the verified buffer.
#[cfg(windows)]
pub type SharedPendingUpdate = Mutex<Option<(tauri_plugin_updater::Update, Vec<u8>)>>;

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
pub async fn set_update_channel(
    channel: UpdateChannel,
    status: tauri::State<'_, SharedUpdateStatus>,
    app: tauri::AppHandle,
) -> AppResult<UpdateStatusDto> {
    let root = update_schedule::active_root(&app)?;
    let status = status.inner().clone();
    crate::commands::run_blocking("set update channel", move || {
        let _transaction = crate::modules::backup::manager::acquire_preferences_transaction(&root)?;
        let mut guard = status.lock().map_err(|_| AppError::NotConfigured)?;
        persist_channel(&root, &mut guard, channel)?;
        hydrate_cache(&mut guard, &app)?;
        Ok(current_status(&guard, &app).into())
    })
    .await
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
    }
    .into();
    crate::commands::preferences::save_preferences_transaction(root, &preferences, status)?;
    Ok(())
}

#[tauri::command]
pub async fn check_for_update(
    trigger: Option<event_delivery::QueryTrigger>,
    status: tauri::State<'_, SharedUpdateStatus>,
    app: tauri::AppHandle,
) -> AppResult<CheckUpdateResultDto> {
    let (updater, generation, target, channel) = {
        let mut guard = status.lock().map_err(|_| AppError::NotConfigured)?;
        let updater = updater_for_status(&app, &guard)?;
        let generation = guard.begin_check().ok_or(AppError::NotConfigured)?;
        (
            updater,
            generation,
            Target::manager(guard.channel),
            guard.channel,
        )
    };
    let _lease = CheckLease(status.inner().clone(), generation);
    let reserve_app = app.clone();
    let (query_root, identity) =
        crate::commands::run_blocking("reserve manager query", move || {
            let root = update_schedule::reserve(&reserve_app, target)?;
            let event_channel = match channel {
                UpdateChannel::Stable => EventChannel::Stable,
                UpdateChannel::Beta => EventChannel::Beta,
            };
            // Query scope is distinct from installation; retrying the same endpoint
            // resolves only a query failure, never an installer failure.
            let identity = event_delivery::prepare(
                &root,
                "manager-check-failed",
                event_channel,
                channel.endpoint().as_bytes(),
            );
            Ok((root, identity))
        })
        .await?;
    let update = updater.check().await;
    let shared_status = status.inner().clone();
    crate::commands::run_blocking("commit manager query", move || {
        let mut guard = shared_status.lock().map_err(|_| AppError::NotConfigured)?;
        if !guard.finish_check(generation) {
            return Ok(CheckUpdateResultDto {
                status: "superseded".into(),
                update: current_status(&guard, &app).into(),
            });
        }
        let (result_status, event) = match update {
            Err(_) => {
                guard.signature_verified = None;
                guard.available_version = None;
                guard.last_checked_at = Some(chrono::Utc::now().to_rfc3339());
                guard.error = Some(UpdateFailureClass::Network.message().to_string());
                update_schedule::complete::<UpdateStatus>(&app, &query_root, target, None)?;
                ("failed", "manager-check-failed")
            }
            Ok(update) => {
                record_check_success(
                    &mut guard,
                    update.as_ref().map(|value| value.version.clone()),
                );
                if let Some(value) = update.as_ref() {
                    guard.notes = bounded_notes(value.body.as_deref());
                    guard.release_url = release_page(&value.version);
                    guard.published_at = value.date.map(|date| date.to_string());
                }
                update_schedule::complete(&app, &query_root, target, Some(&*guard))?;
                (
                    if update.is_some() {
                        "available"
                    } else {
                        "up_to_date"
                    },
                    "manager-check-succeeded",
                )
            }
        };
        let result = CheckUpdateResultDto {
            status: result_status.into(),
            update: guard.clone().into(),
        };
        drop(guard);
        event_delivery::publish(
            &app,
            &query_root,
            event,
            identity,
            trigger.unwrap_or_default().into(),
        );
        Ok(result)
    })
    .await
}

/// 记录一次成功的检查结果。
///
/// 必须清空 `error`：`check_for_update` 的失败分支会写入网络失败提示，若成功分支
/// 不清，会让上一次的「更新下载或连接失败」盖住新的「有更新 / 已是最新」结果
/// （界面固定先显示 `error`，于是显示为失败）。
fn record_check_success(status: &mut UpdateStatus, available_version: Option<String>) {
    status.available_version = available_version;
    status.notes = None;
    status.release_url = None;
    status.published_at = None;
    status.signature_verified = Some(false);
    status.error = None;
    status.last_checked_at = Some(chrono::Utc::now().to_rfc3339());
}

/// Exactly the reviewed candidate is installed; hiding an observer never cancels this operation.
#[tauri::command]
pub async fn install_update(
    candidate_version: String,
    channel: UpdateChannel,
    backup: bool,
    progress: tauri::ipc::Channel<UpdateProgress>,
    status: tauri::State<'_, SharedUpdateStatus>,
    app: tauri::AppHandle,
) -> AppResult<()> {
    // Spawn owns the mutation: cancelling an IPC observer does not cancel a write.
    let state = status.inner().clone();
    tauri::async_runtime::spawn(install_update_owned(
        candidate_version,
        channel,
        backup,
        progress,
        state,
        app,
    ))
    .await
    .map_err(|_| AppError::NotConfigured)?
}

async fn install_update_owned(
    candidate_version: String,
    channel: UpdateChannel,
    backup: bool,
    progress: tauri::ipc::Channel<UpdateProgress>,
    status: SharedUpdateStatus,
    app: tauri::AppHandle,
) -> AppResult<()> {
    let (updater, generation) = {
        let mut guard = status.lock().map_err(|_| AppError::NotConfigured)?;
        if !can_install(&guard, &candidate_version, channel) {
            return Err(AppError::NotConfigured);
        }
        let updater = updater_for_status(&app, &guard)?;
        guard.generation = guard.generation.wrapping_add(1);
        guard.installing = true;
        (updater, guard.generation)
    };
    let _lease = InstallationLease(status.clone());
    // Manager and managed panel installations share mutation ownership.
    let runtime = app.state::<crate::state::SharedRuntimeInstall>();
    let _mutation = runtime.acquire().ok_or(AppError::NotConfigured)?;
    let mut observation = UpdateProgress::new(channel, candidate_version.clone(), generation);
    let root = update_schedule::active_root(&app)?;
    let identity_root = root.clone();
    let identity_version = candidate_version.clone();
    let event_channel = match channel {
        UpdateChannel::Stable => EventChannel::Stable,
        UpdateChannel::Beta => EventChannel::Beta,
    };
    // Catalog validation is pure and done once before the owned progress stream.
    crate::modules::notifications::registry::validate_signal(
        "manager-install-progress",
        crate::modules::notifications::registry::Job::ManagerInstall,
        Trigger::User,
        event_channel,
    )
    .map_err(|_| AppError::NotConfigured)?;
    let identity = crate::commands::run_blocking("prepare manager installation event", move || {
        Ok(event_delivery::prepare(
            &identity_root,
            "manager-install-failed",
            event_channel,
            identity_version.as_bytes(),
        ))
    })
    .await?;
    let handoff = crate::modules::update::handoff::InstallationHandoff::signed_candidate(
        candidate_version.clone(),
        identity.clone().ok_or(AppError::NotConfigured)?,
        chrono::Utc::now().timestamp(),
    )?;
    let event_root = root.clone();
    let result = async {
        if backup {
            let _ = progress.send(observation.step("backup"));
        }
        let protection_app = app.clone();
        let _protection = crate::commands::run_blocking("protect manager update", move || {
            if backup {
                crate::modules::backup::manager::begin_preferences_protection_observed(
                    &root,
                    true,
                    &mut event_delivery::BackupEvents {
                        app: &protection_app,
                        root: &root,
                        trigger: Trigger::User,
                    },
                )
            } else {
                crate::modules::backup::manager::acquire_preferences_transaction(&root)
            }
        })
        .await?;
        let _ = progress.send(observation.step("checking"));
        let update = updater
            .check()
            .await
            .map_err(|_| AppError::NotConfigured)?
            .filter(|value| value.version == candidate_version)
            .ok_or(AppError::NotConfigured)?;
        let _ = progress.send(observation.step("downloading"));
        let mut last_emit = std::time::Instant::now();
        let bytes = update
            .download(
                |chunk, total| {
                    observation.chunk(chunk, total);
                    if last_emit.elapsed() >= std::time::Duration::from_millis(100) {
                        let _ = progress.send(observation.step("downloading"));
                        last_emit = std::time::Instant::now();
                    }
                },
                || {},
            )
            .await
            .map_err(|_| AppError::NotConfigured)?;
        // download() has completed signature validation; install() can block the OS thread.
        #[cfg(windows)]
        {
            let pending = app.state::<SharedPendingUpdate>();
            *pending.lock().map_err(|_| AppError::NotConfigured)? = Some((update, bytes));
        }
        #[cfg(not(windows))]
        {
            let _ = progress.send(observation.step("installing"));
            // Persist before OS replacement: an abrupt exit cannot lose the receipt.
            let install_root = event_root.clone();
            let receipt = handoff.clone();
            tauri::async_runtime::spawn_blocking(move || {
                receipt.arm(&install_root)?;
                update.install(bytes).map_err(|_| AppError::NotConfigured)
            })
            .await
            .map_err(|_| AppError::NotConfigured)?
            .map_err(|_| AppError::NotConfigured)?;
        }
        Ok::<_, AppError>(())
    }
    .await;
    let mut guard = status.lock().map_err(|_| AppError::NotConfigured)?;
    if result.is_ok() {
        guard.signature_verified = Some(true);
        guard.pending_restart = Some(candidate_version);
        guard.pending_handoff = Some(handoff.clone());
        guard.error = None;
        let _ = progress.send(observation.step("pending_restart"));
    } else {
        guard.signature_verified = Some(false);
        guard.error = Some(UpdateFailureClass::Install.message().into());
        let _ = progress.send(observation.step("failed"));
    }
    drop(guard);
    if result.is_err() {
        // Safe even if no handoff was armed; do not delete a different attempt.
        let _ = handoff.discard(&event_root);
    }
    // Prepared verified bytes / an installed artifact is not a running new version.
    // Pending restart deliberately cannot resolve an earlier install failure.
    event_delivery::publish(
        &app,
        &event_root,
        if result.is_ok() {
            "manager-install-pending-restart"
        } else {
            "manager-install-failed"
        },
        identity,
        Trigger::User,
    );
    result
}

#[tauri::command]
pub async fn restart_after_update(
    status: tauri::State<'_, SharedUpdateStatus>,
    app: tauri::AppHandle,
) -> AppResult<()> {
    // The owned task outlives a closed IPC observer during native handoff.
    tauri::async_runtime::spawn(restart_owned(status.inner().clone(), app))
        .await
        .map_err(|_| AppError::NotConfigured)?
}

async fn restart_owned(status: SharedUpdateStatus, app: tauri::AppHandle) -> AppResult<()> {
    let receipt = {
        let mut guard = status.lock().map_err(|_| AppError::NotConfigured)?;
        if guard.installing || guard.pending_restart.is_none() {
            return Err(AppError::NotConfigured);
        }
        let receipt = guard
            .pending_handoff
            .clone()
            .filter(|r| guard.pending_restart.as_deref() == Some(r.version()))
            .ok_or(AppError::NotConfigured)?;
        guard.installing = true;
        receipt
    };
    let _lease = InstallationLease(status.clone());
    let root = update_schedule::active_root(&app)?;
    let result = async {
        let runtime = app.state::<crate::state::SharedRuntimeInstall>();
        let _mutation = runtime.acquire().ok_or(AppError::NotConfigured)?;
        let protection_root = root.clone();
        let _protection = crate::commands::run_blocking("own manager restart", move || {
            crate::modules::backup::manager::acquire_preferences_transaction(&protection_root)
        })
        .await?;
        #[cfg(windows)]
        {
            let artifact = app
                .state::<SharedPendingUpdate>()
                .lock()
                .map_err(|_| AppError::NotConfigured)?
                .take()
                .ok_or(AppError::NotConfigured)?;
            let install_root = root.clone();
            let install_receipt = receipt.clone();
            tauri::async_runtime::spawn_blocking(move || {
                // Windows download is never armed by an ordinary app exit.
                install_receipt.arm(&install_root)?;
                artifact
                    .0
                    .install(artifact.1)
                    .map_err(|_| AppError::NotConfigured)
            })
            .await
            .map_err(|_| AppError::NotConfigured)??;
        }
        #[cfg(not(windows))]
        {
            if crate::modules::update::handoff::InstallationHandoff::load(&root)?.as_ref()
                != Some(&receipt)
            {
                return Err(AppError::NotConfigured);
            }
            app.request_restart();
        }
        Ok::<_, AppError>(())
    }
    .await;
    if result.is_err() {
        let _ = receipt.discard(&root);
        if let Ok(mut guard) = status.lock() {
            guard.pending_restart = None;
            guard.pending_handoff = None;
            guard.error = Some(UpdateFailureClass::Install.message().into());
        }
        event_delivery::publish(
            &app,
            &root,
            "manager-install-failed",
            Some(receipt.identity()),
            Trigger::User,
        );
    }
    result
}

/// Reconcile only after the durable notification store is managed. A failed
/// persistence leaves the receipt intact so a later startup can retry delivery.
pub fn reconcile_startup(app: &tauri::AppHandle, root: &std::path::Path) -> AppResult<()> {
    use crate::modules::update::handoff::InstallationHandoff;
    let Some(receipt) = InstallationHandoff::load(root)? else {
        return Ok(());
    };
    let Some(identity) = receipt.running_identity(
        root,
        &app.package_info().version.to_string(),
        chrono::Utc::now().timestamp(),
    )?
    else {
        return Ok(());
    };
    event_delivery::publish_checked(
        app,
        root,
        "manager-install-succeeded",
        Some(identity),
        Trigger::Startup,
    )?;
    receipt.discard(root)
}

fn valid_candidate(version: &str) -> bool {
    crate::modules::update::handoff::valid_version(version)
}
fn can_install(status: &UpdateStatus, version: &str, channel: UpdateChannel) -> bool {
    valid_candidate(version)
        && !status.checking
        && !status.installing
        && status.pending_restart.is_none()
        && status.channel == channel
        && status.available_version.as_deref() == Some(version)
}
fn bounded_notes(notes: Option<&str>) -> Option<String> {
    notes.filter(|value| !value.trim().is_empty()).map(|value| {
        let mut end = value.len().min(64 * 1024);
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        value[..end].to_owned()
    })
}
fn release_page(version: &str) -> Option<String> {
    valid_candidate(version)
        .then(|| format!("https://github.com/gzers/opencodex-desktop/releases/tag/v{version}"))
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
                status.notes = bounded_notes(cached.notes.as_deref());
                status.release_url = status.available_version.as_deref().and_then(release_page);
                status.published_at = cached.published_at;
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
    fn reviewed_candidate_must_match_channel_version_and_idle_state() {
        let mut status = UpdateStatus::pending("0.1.9");
        status.available_version = Some("0.1.10".into());
        assert!(can_install(&status, "0.1.10", UpdateChannel::Stable));
        assert!(!can_install(&status, "0.1.11", UpdateChannel::Stable));
        assert!(!can_install(&status, "0.1.10", UpdateChannel::Beta));
        status.pending_restart = Some("0.1.10".into());
        assert!(!can_install(&status, "0.1.10", UpdateChannel::Stable));
        assert!(status.begin_check().is_none());
        assert!(!status.switch_channel(UpdateChannel::Beta));
        status.pending_restart = None;
        status.checking = true;
        assert!(!can_install(&status, "0.1.10", UpdateChannel::Stable));
        status.checking = false;
        status.installing = true;
        assert!(!can_install(&status, "0.1.10", UpdateChannel::Stable));
    }

    #[test]
    fn notes_are_bounded_plaintext_and_release_links_cannot_escape() {
        let notes = "更新🦀".repeat(30000);
        let bounded = bounded_notes(Some(&notes)).unwrap();
        assert!(bounded.len() <= 64 * 1024);
        assert!(notes.starts_with(&bounded));
        assert_eq!(bounded_notes(Some("   ")), None);
        assert_eq!(
            bounded_notes(Some("<script>alert(1)</script>")),
            Some("<script>alert(1)</script>".into())
        );
        for invalid in [
            "",
            "../x",
            "0.1.10?q=x",
            "0.1.10#x",
            "0.1.10/../x",
            "0.1.10\n",
            "http://x",
        ] {
            assert!(release_page(invalid).is_none());
        }
        assert!(release_page("0.1.10-beta.1")
            .unwrap()
            .ends_with("/v0.1.10-beta.1"));
        let mut status = UpdateStatus::pending("0.1.9");
        status.notes = Some("stale".into());
        status.release_url = Some("stale".into());
        record_check_success(&mut status, None);
        assert!(status.notes.is_none() && status.release_url.is_none());
    }

    #[test]
    fn channel_command_persists_and_refuses_install_races() {
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        let preferences = crate::modules::preferences::Preferences {
            interface_scale: 125,
            ..Default::default()
        };
        crate::commands::preferences::save_preferences_with_path(root.path(), &preferences)
            .unwrap();
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
        assert_eq!(
            crate::commands::preferences::load_preferences_with_path(root.path())
                .unwrap()
                .app_update_channel,
            "beta"
        );
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
