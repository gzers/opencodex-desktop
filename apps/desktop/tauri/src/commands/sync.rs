//! WebDAV 同步命令层。
//!
//! 只投影端点配置、连接测试与同步执行结果；真实密钥不回传前端。

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::errors::{AppError, AppResult};
use crate::infrastructure::keychain;
use crate::infrastructure::webdav_client::{WebDavClient, WebDavConfig, WebDavOperation};
use crate::modules::sync::config::{
    SyncConfig, SyncConfigStore, SyncEndpointConfig, SyncEndpointInput,
};
use crate::modules::sync::runner::SyncRunFailure;
use crate::state::{SharedDataRoot, SharedHomeDir, SharedNotificationStore, SharedSyncStatus};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncEndpointDto {
    pub endpoint_id: String,
    pub url: String,
    pub remote_path: String,
    pub username: String,
    pub credential_ref_id: String,
    pub tls_policy: String,
    /// 同步载荷保护方式：当前固定为明文 + 完整性校验，不含额外内容加密口令。
    pub payload_protection: String,
    pub conflict_policy: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncConfigDto {
    pub endpoint: Option<SyncEndpointDto>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveSyncEndpointRequest {
    pub base_url: String,
    pub remote_path: String,
    pub username: String,
    pub password: String,
    pub conflict_policy: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncOperationResultDto {
    pub connection_state: String,
    pub operation_state: String,
    pub message: String,
    pub snapshot_id: Option<String>,
    pub backup_id: Option<String>,
    pub etag: Option<String>,
}

fn project_endpoint(endpoint: &SyncEndpointConfig) -> SyncEndpointDto {
    SyncEndpointDto {
        endpoint_id: endpoint.endpoint_id.clone(),
        url: endpoint.url.clone(),
        remote_path: endpoint.remote_path.clone(),
        username: endpoint.username.clone(),
        credential_ref_id: endpoint.credential_ref.ref_id.clone(),
        tls_policy: endpoint.tls_policy.clone(),
        payload_protection: "integrity_only".to_string(),
        conflict_policy: endpoint.conflict_policy.clone(),
    }
}

pub fn project_config(config: &SyncConfig) -> SyncConfigDto {
    SyncConfigDto {
        endpoint: config.active().map(project_endpoint),
    }
}

fn config_store(data_root: &SharedDataRoot) -> SyncConfigStore {
    SyncConfigStore::new(&data_root.0)
}

#[tauri::command]
pub fn get_sync_config(data_root: State<'_, SharedDataRoot>) -> AppResult<SyncConfigDto> {
    let config = config_store(&data_root).load()?;
    Ok(project_config(&config))
}

#[tauri::command]
pub async fn save_sync_endpoint(
    request: SaveSyncEndpointRequest,
    data_root: State<'_, SharedDataRoot>,
) -> AppResult<SyncConfigDto> {
    let input = SyncEndpointInput {
        base_url: request.base_url,
        remote_path: request.remote_path,
        username: request.username,
        password: request.password,
        conflict_policy: request.conflict_policy,
    };
    let root = data_root.inner().0.clone();
    let endpoint = tauri::async_runtime::spawn_blocking(move || {
        let store = SyncConfigStore::new(&root);
        store.save_endpoint(&input)
    })
    .await
    .map_err(|_| AppError::NotConfigured)??;
    Ok(SyncConfigDto {
        endpoint: Some(project_endpoint(&endpoint)),
    })
}

#[tauri::command]
pub async fn delete_sync_endpoint(
    delete_credentials: bool,
    data_root: State<'_, SharedDataRoot>,
) -> AppResult<SyncConfigDto> {
    {
        let root = data_root.inner().0.clone();
        tauri::async_runtime::spawn_blocking(move || {
            SyncConfigStore::new(&root).delete_endpoint(delete_credentials)
        })
        .await
        .map_err(|_| AppError::NotConfigured)??;
    }
    Ok(project_config(&SyncConfig::default()))
}

fn webdav_config(config: &SyncEndpointConfig) -> AppResult<WebDavConfig> {
    let password = keychain::load_webdav_password(&config.credential_ref.ref_id)?;
    Ok(WebDavConfig {
        base_url: config.url.clone(),
        remote_path: config.remote_path.clone(),
        username: config.username.clone(),
        password,
    })
}

fn status_guard<'a>(
    status: &'a State<'_, SharedSyncStatus>,
) -> std::sync::MutexGuard<'a, crate::modules::sync::SyncRun> {
    status.inner().lock().expect("sync status mutex poisoned")
}

#[tauri::command]
pub async fn test_sync_connection(
    data_root: State<'_, SharedDataRoot>,
    status: State<'_, SharedSyncStatus>,
) -> AppResult<SyncOperationResultDto> {
    let config = config_store(&data_root).load()?;
    let Some(endpoint) = config.active() else {
        return Err(AppError::NotConfigured);
    };
    let webdav = webdav_config(endpoint)?;
    {
        let mut run = status_guard(&status);
        run.connection_state = crate::types::status::ConnectionState::Connecting;
        run.operation_state = crate::types::status::OperationState::Validating;
        run.finished_at = None;
        run.failure_reason = None;
    }
    let result = WebDavClient::production().test_connection(&webdav).await;
    let mut run = status_guard(&status);
    match result {
        Ok(()) => {
            run.connection_state = crate::types::status::ConnectionState::Synced;
            run.operation_state = crate::types::status::OperationState::Succeeded;
            run.finished_at = Some(chrono::Utc::now());
            Ok(SyncOperationResultDto {
                connection_state: "synced".to_string(),
                operation_state: "succeeded".to_string(),
                message: "WebDAV 连接成功。".to_string(),
                snapshot_id: run.snapshot_id.clone(),
                backup_id: None,
                etag: None,
            })
        }
        Err(error) => {
            run.connection_state = crate::types::status::ConnectionState::Failed;
            run.operation_state = crate::types::status::OperationState::Failed;
            run.finished_at = Some(chrono::Utc::now());
            run.failure_reason = Some(
                error
                    .user_message(crate::infrastructure::webdav_client::WebDavOperation::Probe)
                    .to_string(),
            );
            Ok(SyncOperationResultDto {
                connection_state: "failed".to_string(),
                operation_state: "failed".to_string(),
                message: run.failure_reason.clone().unwrap_or_default(),
                snapshot_id: None,
                backup_id: None,
                etag: None,
            })
        }
    }
}

#[tauri::command]
pub async fn run_sync_now(
    data_root: State<'_, SharedDataRoot>,
    home: State<'_, SharedHomeDir>,
    status: State<'_, SharedSyncStatus>,
    notifications: State<'_, SharedNotificationStore>,
) -> AppResult<SyncOperationResultDto> {
    let config = config_store(&data_root).load()?;
    let Some(endpoint) = config.active().cloned() else {
        return Err(AppError::NotConfigured);
    };
    let webdav = webdav_config(&endpoint)?;
    let now = chrono::Utc::now();
    {
        let mut run = status_guard(&status);
        run.connection_state = crate::types::status::ConnectionState::Connecting;
        run.operation_state = crate::types::status::OperationState::Validating;
        run.started_at = now;
        run.finished_at = None;
        run.failure_reason = None;
        run.items_total = crate::modules::sync::runner::SYNCED_ARTIFACTS.len();
        run.items_done = 0;
    }
    let data_root_path = data_root.inner().0.clone();
    let home_path = home.inner().0.clone();
    let outcome =
        crate::modules::sync::runner::run_sync(&data_root_path, &home_path, &endpoint, &webdav)
            .await;
    let mut run = status_guard(&status);
    match outcome {
        Ok(outcome) => {
            if outcome.conflicted {
                // 「同步冲突提醒」：检测到冲突时写入一条持久通知；关闭开关则不打扰用户。
                // 无论开关如何，覆盖都被暂停——提醒只影响通知，不影响安全语义。
                publish_sync_conflict(
                    sync_conflict_alerts_enabled(&data_root_path),
                    notifications.inner(),
                    &data_root_path,
                );
            }
            run.connection_state = crate::types::status::ConnectionState::Synced;
            run.operation_state = crate::types::status::OperationState::Succeeded;
            run.finished_at = Some(chrono::Utc::now());
            run.items_done = outcome.uploaded.len();
            run.snapshot_id = Some(outcome.snapshot_id.clone());
            Ok(SyncOperationResultDto {
                connection_state: "synced".to_string(),
                operation_state: "succeeded".to_string(),
                message: outcome.message(),
                snapshot_id: Some(outcome.snapshot_id),
                backup_id: None,
                etag: outcome.etag,
            })
        }
        // 上传阶段失败：按既有契约返回「失败但命令成功」的结果投影。
        Err(failure @ SyncRunFailure::WebDav(WebDavOperation::Upload, _)) => {
            let message = failure.user_message();
            run.connection_state = crate::types::status::ConnectionState::Failed;
            run.operation_state = crate::types::status::OperationState::Failed;
            run.finished_at = Some(chrono::Utc::now());
            run.failure_reason = Some(message.clone());
            Ok(SyncOperationResultDto {
                connection_state: "failed".to_string(),
                operation_state: "failed".to_string(),
                message,
                snapshot_id: None,
                backup_id: None,
                etag: None,
            })
        }
        Err(failure) => {
            run.connection_state = crate::types::status::ConnectionState::Failed;
            run.operation_state = crate::types::status::OperationState::Failed;
            run.finished_at = Some(chrono::Utc::now());
            run.failure_reason = Some(failure.user_message());
            Err(app_error_for(failure))
        }
    }
}

fn app_error_for(failure: SyncRunFailure) -> AppError {
    match failure {
        SyncRunFailure::ColdSync => AppError::TargetLockTimeout { timeout_ms: 0 },
        SyncRunFailure::TargetLocked => AppError::TargetLockTimeout { timeout_ms: 3000 },
        _ => AppError::NotConfigured,
    }
}

/// 「同步冲突提醒」偏好；读不到时按默认（开启）处理。
fn sync_conflict_alerts_enabled(data_root: &std::path::Path) -> bool {
    crate::modules::preferences::PreferencesStore::new(data_root)
        .load()
        .map(|preferences| preferences.sync_conflict_alerts)
        .unwrap_or(true)
}

/// 冲突提醒：同一冲突在存活期内只保留一条（`dedupe_key`）。
///
/// 发布失败不影响主流程：同步结果仍如实返回，仅记入运行日志。
fn publish_sync_conflict(
    enabled: bool,
    store: &SharedNotificationStore,
    data_root: &std::path::Path,
) {
    if !enabled {
        return;
    }
    let notification = match crate::modules::notifications::Notification::new(
        "sync-conflict-detected",
        crate::modules::notifications::NotificationLevel::Warning,
        crate::modules::notifications::NotificationCategory::Sync,
        crate::modules::notifications::NotificationSource::Sync,
        "检测到同步冲突",
        // 回放冲突在写盘前即返回，**不会覆盖本地文件**，因此不存在「被覆盖侧的可恢复备份」；
        // 通知文案必须与真实副作用一致（此前谎称已保留覆盖备份，属 §26.1 排除的假承诺）。
        "远端快照与本地历史冲突，已暂停覆盖并保留双方历史；请确认后重试。",
        chrono::Utc::now().to_rfc3339(),
        Some(crate::modules::notifications::NotificationAction::Sync),
    ) {
        Ok(value) => value.with_dedupe_key("sync:conflict-detected"),
        Err(_) => return,
    };
    let publisher = crate::commands::notifications::NotificationPublisher { store, data_root };
    if publisher.publish(notification).is_err() {
        let _ = crate::infrastructure::runtime_log::RuntimeLog::new(data_root)
            .append_event("publish sync conflict notification failed");
    }
}

#[tauri::command]
pub fn get_sync_status(status: State<'_, SharedSyncStatus>) -> AppResult<SyncOperationResultDto> {
    let run = status_guard(&status);
    Ok(SyncOperationResultDto {
        connection_state: format!("{:?}", run.connection_state).to_snake_case(),
        operation_state: format!("{:?}", run.operation_state).to_snake_case(),
        message: run.failure_reason.clone().unwrap_or_default(),
        snapshot_id: run.snapshot_id.clone(),
        backup_id: None,
        etag: None,
    })
}

trait SnakeCase {
    fn to_snake_case(self) -> String;
}

impl SnakeCase for String {
    fn to_snake_case(self) -> String {
        let mut output = String::new();
        for (index, character) in self.char_indices() {
            if character.is_ascii_uppercase() {
                if index > 0 {
                    output.push('_');
                }
                output.extend(character.to_lowercase());
            } else {
                output.push(character);
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::sync::config::{CredentialRef, SyncEndpointConfig};

    fn endpoint() -> SyncEndpointConfig {
        SyncEndpointConfig {
            endpoint_id: "default".to_string(),
            url: "https://dav.example.test".to_string(),
            remote_path: "desktop-sync".to_string(),
            username: "user".to_string(),
            credential_ref: CredentialRef {
                ref_id: "cred_12345678-1234-1234-1234-123456789abc".to_string(),
                backend: "keychain".to_string(),
                service_name: "OpenCodex Desktop".to_string(),
                account_key: "ocx.dav.cred_12345678-1234-1234-1234-123456789abc".to_string(),
                purpose: "webdav_credential".to_string(),
                created_at: "2026-09-17T00:00:00Z".to_string(),
                updated_at: "2026-09-17T00:00:00Z".to_string(),
            },
            tls_policy: "verify_required".to_string(),
            legacy_encryption: "forced".to_string(),
            conflict_policy: "ask".to_string(),
        }
    }

    #[test]
    fn config_dto_is_camel_case_and_masks_password() {
        let value = serde_json::to_value(project_config(&SyncConfig {
            schema_version: 1,
            endpoints: vec![endpoint()],
        }))
        .unwrap();
        assert_eq!(value["endpoint"]["endpointId"], "default");
        assert_eq!(
            value["endpoint"]["credentialRefId"],
            "cred_12345678-1234-1234-1234-123456789abc"
        );
        assert!(value["endpoint"].get("password").is_none());
        assert!(serde_json::to_string(&value)
            .unwrap()
            .contains("remotePath"));
    }

    #[test]
    fn state_names_are_frozen_snake_case() {
        assert_eq!(
            format!("{:?}", crate::types::status::ConnectionState::Connecting).to_snake_case(),
            "connecting"
        );
        assert_eq!(
            format!("{:?}", crate::types::status::OperationState::Validating).to_snake_case(),
            "validating"
        );
    }

    fn seeded_data_root(
        preferences: crate::modules::preferences::Preferences,
    ) -> tempfile::TempDir {
        let root = tempfile::tempdir().expect("temp root");
        crate::modules::data_root::initialize(root.path()).expect("initialize");
        crate::modules::preferences::PreferencesStore::new(root.path())
            .save(&preferences)
            .expect("save preferences");
        root
    }

    // 回归：「同步冲突提醒」此前没有任何消费方，冲突时不会产生任何通知。
    #[test]
    fn sync_conflict_alerts_preference_is_consumed() {
        let root = seeded_data_root(crate::modules::preferences::Preferences::default());
        assert!(sync_conflict_alerts_enabled(root.path()));
        crate::modules::preferences::PreferencesStore::new(root.path())
            .save(&crate::modules::preferences::Preferences {
                sync_conflict_alerts: false,
                ..Default::default()
            })
            .expect("save");
        assert!(!sync_conflict_alerts_enabled(root.path()));
    }

    #[test]
    fn conflict_alert_writes_one_deduplicated_notification() {
        use std::sync::{Arc, Mutex};
        let root = seeded_data_root(crate::modules::preferences::Preferences::default());
        let store: SharedNotificationStore = Arc::new(Mutex::new(
            crate::modules::notifications::NotificationStore::new(),
        ));
        publish_sync_conflict(true, &store, root.path());
        publish_sync_conflict(true, &store, root.path());
        let guard = store.lock().expect("lock");
        assert_eq!(guard.live().len(), 1, "同一冲突只保留一条");
        assert_eq!(
            guard.live()[0].category,
            crate::modules::notifications::NotificationCategory::Sync
        );
        // 回放冲突不覆盖本地文件 ⇒ 不存在「覆盖前备份」；文案不得谎称已保留覆盖备份。
        assert!(
            guard.live()[0].body.contains("保留双方历史"),
            "冲突通知应如实说明保留双方历史：{}",
            guard.live()[0].body
        );
        assert!(
            !guard.live()[0].body.contains("可恢复备份"),
            "回放冲突未覆盖本地，通知不得声称保留覆盖备份：{}",
            guard.live()[0].body
        );
        drop(guard);

        let quiet = seeded_data_root(crate::modules::preferences::Preferences::default());
        let empty: SharedNotificationStore = Arc::new(Mutex::new(
            crate::modules::notifications::NotificationStore::new(),
        ));
        publish_sync_conflict(false, &empty, quiet.path());
        assert!(empty.lock().expect("lock").live().is_empty());
    }
}
