//! WebDAV 同步命令层。
//!
//! 只投影端点配置、连接测试与同步执行结果；真实密钥不回传前端。

use serde::{Deserialize, Serialize};
use tauri::{Manager, State};

use crate::errors::{AppError, AppResult};
use crate::infrastructure::keychain;
use crate::infrastructure::webdav_client::{
    WebDavClient, WebDavConfig, WebDavError, WebDavOperation, WebDavTransport,
};
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

/// A single admitted sync operation owns endpoint and status mutations. Busy
/// commands fail admission; they do not queue or publish network failures.
fn sync_operation_gate() -> std::sync::Arc<tokio::sync::Mutex<()>> {
    static GATE: std::sync::OnceLock<std::sync::Arc<tokio::sync::Mutex<()>>> =
        std::sync::OnceLock::new();
    GATE.get_or_init(|| std::sync::Arc::new(tokio::sync::Mutex::new(())))
        .clone()
}

/// The worker, rather than its IPC observer, owns both admissions until all
/// state projection and notifications complete. This also moves local IO and
/// credentials off the IPC/main thread.
async fn run_owned_sync<T, F>(operation: &'static str, task: F) -> AppResult<T>
where
    F: FnOnce() -> AppResult<T> + Send + 'static,
    T: Send + 'static,
{
    run_owned_sync_with_gates(
        crate::infrastructure::storage_writers::global(),
        sync_operation_gate(),
        operation,
        task,
    )
    .await
}

async fn run_owned_sync_with_gates<T, F>(
    writers: std::sync::Arc<crate::infrastructure::storage_writers::WriterGate>,
    operations: std::sync::Arc<tokio::sync::Mutex<()>>,
    operation: &'static str,
    task: F,
) -> AppResult<T>
where
    F: FnOnce() -> AppResult<T> + Send + 'static,
    T: Send + 'static,
{
    crate::commands::run_blocking_with_gate(writers, operation, move || {
        let _sync = admit_sync_with_gate(operations)?;
        task()
    })
    .await
}

fn admit_sync_with_gate(
    gate: std::sync::Arc<tokio::sync::Mutex<()>>,
) -> AppResult<tokio::sync::OwnedMutexGuard<()>> {
    gate.try_lock_owned()
        .map_err(|_| AppError::TargetLockTimeout { timeout_ms: 0 })
}

fn probe_candidate(endpoint: &SyncEndpointConfig, webdav: &WebDavConfig) -> AppResult<Vec<u8>> {
    serde_json::to_vec(&(
        endpoint,
        &webdav.base_url,
        &webdav.remote_path,
        &webdav.username,
        &webdav.password,
    ))
    .map_err(|_| AppError::NotConfigured)
}

/// Only an actual, bounded request followed by unchanged configuration is a
/// terminal fact. The owner holds the sync gate through this callback. Secrets
/// exist only in local candidate bytes; event delivery persists their digest.
async fn probe_connection_observed<T: WebDavTransport>(
    client: &WebDavClient<T>,
    config: &WebDavConfig,
    timeout: std::time::Duration,
    current: impl FnOnce() -> AppResult<bool>,
    observer: impl FnOnce(bool),
) -> AppResult<Result<(), WebDavError>> {
    config.validate().map_err(|_| AppError::NotConfigured)?;
    let result = tokio::time::timeout(timeout, client.test_connection(config))
        .await
        .unwrap_or(Err(WebDavError::Network));
    if !current()? {
        return Err(AppError::NotConfigured);
    }
    observer(result.is_ok());
    Ok(result)
}

#[tauri::command]
pub async fn save_sync_endpoint(
    request: SaveSyncEndpointRequest,
    data_root: State<'_, SharedDataRoot>,
    app: tauri::AppHandle,
) -> AppResult<SyncConfigDto> {
    let input = SyncEndpointInput {
        base_url: request.base_url,
        remote_path: request.remote_path,
        username: request.username,
        password: request.password,
        conflict_policy: request.conflict_policy,
    };
    let root = data_root.inner().0.clone();
    let endpoint = run_owned_sync("save sync endpoint", move || {
        let store = SyncConfigStore::new(&root);
        store.save_endpoint_observed(&input, |candidate, succeeded| {
            publish_endpoint_result(&app, &root, "sync-endpoint-save", candidate, succeeded);
        })
    })
    .await?;
    Ok(SyncConfigDto {
        endpoint: Some(project_endpoint(&endpoint)),
    })
}

#[tauri::command]
pub async fn delete_sync_endpoint(
    delete_credentials: bool,
    data_root: State<'_, SharedDataRoot>,
    app: tauri::AppHandle,
) -> AppResult<SyncConfigDto> {
    {
        let root = data_root.inner().0.clone();
        run_owned_sync("delete sync endpoint", move || {
            SyncConfigStore::new(&root).delete_endpoint_observed(
                delete_credentials,
                |candidate, succeeded| {
                    publish_endpoint_result(
                        &app,
                        &root,
                        "sync-endpoint-delete",
                        candidate,
                        succeeded,
                    );
                },
            )
        })
        .await?;
    }
    Ok(project_config(&SyncConfig::default()))
}

fn publish_endpoint_result(
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
    app: tauri::AppHandle,
    data_root: State<'_, SharedDataRoot>,
) -> AppResult<SyncOperationResultDto> {
    let root = data_root.inner().0.clone();
    run_owned_sync("test sync connection", move || {
        let store = SyncConfigStore::new(&root);
        let config = store.load()?;
        let endpoint = config.active().cloned().ok_or(AppError::NotConfigured)?;
        let webdav = webdav_config(&endpoint)?;
        webdav.validate().map_err(|_| AppError::NotConfigured)?;
        let candidate = probe_candidate(&endpoint, &webdav)?;
        let status = app.state::<SharedSyncStatus>();
        {
            let mut run = status_guard(&status);
            run.run_id = uuid::Uuid::new_v4().to_string();
            run.started_at = chrono::Utc::now();
            run.connection_state = crate::types::status::ConnectionState::Connecting;
            run.operation_state = crate::types::status::OperationState::Validating;
            run.finished_at = None;
            run.failure_reason = None;
        }
        let client = WebDavClient::production();
        let result = tauri::async_runtime::block_on(probe_connection_observed(
            &client,
            &webdav,
            client.total_timeout(),
            || {
                let current = store.load()?;
                let Some(active) = current.active() else {
                    return Ok(false);
                };
                Ok(active == &endpoint && webdav_config(active)? == webdav)
            },
            |succeeded| {
                use crate::modules::notifications::registry::{Channel, Trigger};
                let identity = crate::commands::event_delivery::prepare(
                    &root,
                    "sync-connection-failed",
                    Channel::Local,
                    &candidate,
                );
                crate::commands::event_delivery::publish(
                    &app,
                    &root,
                    if succeeded {
                        "sync-connection-succeeded"
                    } else {
                        "sync-connection-failed"
                    },
                    identity,
                    Trigger::User,
                );
            },
        ));
        let mut run = status_guard(&status);
        run.finished_at = Some(chrono::Utc::now());
        let result = match result {
            Ok(result) => result,
            Err(error) => {
                // External changes/readback failures are not a verified result.
                run.operation_state = crate::types::status::OperationState::Cancelled;
                run.connection_state = crate::types::status::ConnectionState::Unconfigured;
                run.failure_reason = None;
                return Err(error);
            }
        };
        match result {
            Ok(()) => {
                run.connection_state = crate::types::status::ConnectionState::Synced;
                run.operation_state = crate::types::status::OperationState::Succeeded;
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
                let message = error.user_message(WebDavOperation::Probe).to_string();
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
        }
    })
    .await
}

#[tauri::command]
pub async fn run_sync_now(
    app: tauri::AppHandle,
    data_root: State<'_, SharedDataRoot>,
    home: State<'_, SharedHomeDir>,
) -> AppResult<SyncOperationResultDto> {
    let data_root_path = data_root.inner().0.clone();
    let home_path = home.inner().0.clone();
    run_owned_sync("run sync now", move || {
        let config = SyncConfigStore::new(&data_root_path).load()?;
        let endpoint = config.active().cloned().ok_or(AppError::NotConfigured)?;
        let webdav = webdav_config(&endpoint)?;
        webdav.validate().map_err(|_| AppError::NotConfigured)?;
        let status = app.state::<SharedSyncStatus>();
        let notifications = app.state::<SharedNotificationStore>();
        let now = chrono::Utc::now();
        {
            let mut run = status_guard(&status);
            run.run_id = uuid::Uuid::new_v4().to_string();
            run.connection_state = crate::types::status::ConnectionState::Connecting;
            run.operation_state = crate::types::status::OperationState::Validating;
            run.started_at = now;
            run.finished_at = None;
            run.failure_reason = None;
            run.items_total = crate::modules::sync::runner::SYNCED_ARTIFACTS.len();
            run.items_done = 0;
        }
        let outcome = tauri::async_runtime::block_on(crate::modules::sync::runner::run_sync(
            &data_root_path,
            &home_path,
            &endpoint,
            &webdav,
        ));
        // Persist and broadcast outside the sync-state lock. A repeated conflict or
        // failed persistence must not emit a notification-list change.
        if outcome.as_ref().is_ok_and(|outcome| outcome.conflicted)
            && publish_sync_conflict(
                sync_conflict_alerts_enabled(&data_root_path),
                notifications.inner(),
                &data_root_path,
            )
        {
            let _ = crate::commands::event_delivery::emit_signal(
                &app,
                crate::commands::notifications::NOTIFICATIONS_CHANGED_EVENT,
                crate::modules::notifications::registry::Job::NotificationMutation,
                crate::modules::notifications::registry::Trigger::Commit,
                crate::modules::notifications::registry::Channel::Local,
                (),
            );
        }
        let mut run = status_guard(&status);
        match outcome {
            Ok(outcome) => {
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
    })
    .await
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
) -> bool {
    if !enabled {
        return false;
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
        Err(_) => return false,
    };
    let publisher = crate::commands::notifications::NotificationPublisher { store, data_root };
    match publisher.publish_changed(notification) {
        Ok(changed) => changed,
        Err(_) => {
            let _ = crate::infrastructure::runtime_log::RuntimeLog::new(data_root)
                .append_event("publish sync conflict notification failed");
            false
        }
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

    pub(super) fn endpoint() -> SyncEndpointConfig {
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
        assert!(publish_sync_conflict(true, &store, root.path()));
        assert!(!publish_sync_conflict(true, &store, root.path()));
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
        assert!(!publish_sync_conflict(false, &empty, quiet.path()));
        assert!(empty.lock().expect("lock").live().is_empty());
    }
}

#[cfg(test)]
mod probe_tests {
    use super::*;
    use crate::commands::notifications::NotificationPublisher;
    use crate::infrastructure::webdav_client::{WebDavMethod, WebDavResponse};
    use crate::modules::notifications::{
        persistence::{load_notifications, notifications_path},
        registry::{self, Channel, Evidence, Trigger},
        NotificationStore,
    };
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    };
    use std::time::Duration;

    struct Transport {
        result: Result<(), WebDavError>,
        calls: Arc<AtomicUsize>,
        entered: Option<Arc<tokio::sync::Notify>>,
        release: Option<Arc<tokio::sync::Notify>>,
    }
    #[async_trait::async_trait]
    impl WebDavTransport for Transport {
        async fn request(
            &self,
            _: &WebDavConfig,
            method: WebDavMethod,
            path: &str,
            body: Option<Vec<u8>>,
        ) -> Result<WebDavResponse, WebDavError> {
            assert_eq!(method, WebDavMethod::Propfind);
            assert!(path.is_empty() && body.is_none());
            self.calls.fetch_add(1, Ordering::SeqCst);
            if let Some(entered) = &self.entered {
                entered.notify_one();
            }
            if let Some(release) = &self.release {
                release.notified().await;
            }
            self.result?;
            Ok(WebDavResponse {
                status: 207,
                body: vec![],
                etag: None,
            })
        }
    }
    fn config() -> WebDavConfig {
        WebDavConfig {
            base_url: "https://dav.example.test".into(),
            remote_path: "desktop-sync".into(),
            username: "user".into(),
            password: "secret-not-for-history".into(),
        }
    }
    fn client(result: Result<(), WebDavError>) -> (WebDavClient<Transport>, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        (
            WebDavClient::new(Transport {
                result,
                calls: calls.clone(),
                entered: None,
                release: None,
            }),
            calls,
        )
    }
    fn publish(
        root: &std::path::Path,
        store: &Arc<Mutex<NotificationStore>>,
        bytes: &[u8],
        success: bool,
    ) -> AppResult<registry::DeliveryOutcome> {
        let identity = crate::commands::event_delivery::prepare(
            root,
            "sync-connection-failed",
            Channel::Local,
            bytes,
        )
        .ok_or(AppError::NotConfigured)?;
        let evidence = if success {
            Evidence::Success {
                candidate: identity.candidate.clone(),
                verified: true,
            }
        } else {
            Evidence::Failure
        };
        let delivery = registry::terminal_delivery(
            if success {
                "sync-connection-succeeded"
            } else {
                "sync-connection-failed"
            },
            Trigger::User,
            identity,
            evidence,
            chrono::Utc::now(),
        )?;
        NotificationPublisher {
            store,
            data_root: root,
        }
        .publish_event(&delivery)
    }

    #[tokio::test]
    async fn invalid_probe_has_no_request_or_terminal() {
        let (client, calls) = client(Ok(()));
        let mut config = config();
        config.base_url = "http://invalid.test".into();
        let result = probe_connection_observed(
            &client,
            &config,
            Duration::from_secs(1),
            || panic!("not admitted"),
            |_| panic!("no terminal"),
        )
        .await;
        assert!(result.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
    #[tokio::test]
    async fn changed_endpoint_or_credential_and_failed_readback_do_not_publish() {
        let (client, calls) = client(Ok(()));
        let original = config();
        for field in 0..3 {
            let mut current = original.clone();
            match field {
                0 => current.remote_path = "different".into(),
                1 => current.password = "rotated".into(),
                _ => current.base_url = "https://other.test".into(),
            }
            assert!(probe_connection_observed(
                &client,
                &original,
                Duration::from_secs(1),
                || Ok(current == original),
                |_| panic!("stale result")
            )
            .await
            .is_err());
        }
        assert!(probe_connection_observed(
            &client,
            &original,
            Duration::from_secs(1),
            || Err(AppError::NotConfigured),
            |_| panic!("readback failed")
        )
        .await
        .is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 4);
    }
    #[tokio::test]
    async fn bounded_probe_timeout_is_an_actual_failure() {
        let calls = Arc::new(AtomicUsize::new(0));
        let client = WebDavClient::new(Transport {
            result: Ok(()),
            calls: calls.clone(),
            entered: None,
            release: Some(Arc::new(tokio::sync::Notify::new())),
        });
        let mut observed = None;
        let result = probe_connection_observed(
            &client,
            &config(),
            Duration::from_millis(1),
            || Ok(true),
            |success| observed = Some(success),
        )
        .await
        .unwrap();
        assert_eq!(result, Err(WebDavError::Network));
        assert_eq!(observed, Some(false));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
    #[tokio::test]
    async fn actual_failure_persists_and_exact_verified_retry_resolves_after_reload() {
        let root = tempfile::tempdir().unwrap();
        let store = Arc::new(Mutex::new(NotificationStore::new()));
        let config = config();
        let bytes = probe_candidate(&super::tests::endpoint(), &config).unwrap();
        let (bad, _) = client(Err(WebDavError::Unauthorized));
        let result = probe_connection_observed(
            &bad,
            &config,
            Duration::from_secs(1),
            || Ok(true),
            |success| {
                assert!(publish(root.path(), &store, &bytes, success).unwrap().added);
            },
        )
        .await
        .unwrap();
        assert_eq!(result, Err(WebDavError::Unauthorized)); // IPC can still return Ok failed DTO.
        let store = Arc::new(Mutex::new(
            load_notifications(&notifications_path(root.path())).unwrap(),
        ));
        let (good, _) = client(Ok(()));
        assert!(probe_connection_observed(
            &good,
            &config,
            Duration::from_secs(1),
            || Ok(true),
            |success| {
                assert_eq!(
                    publish(root.path(), &store, &bytes, success)
                        .unwrap()
                        .resolved,
                    1
                );
            }
        )
        .await
        .unwrap()
        .is_ok());
        let saved = load_notifications(&notifications_path(root.path())).unwrap();
        assert!(saved.all()[0].resolved && !saved.all()[0].read);
        let text = std::fs::read_to_string(notifications_path(root.path())).unwrap();
        assert!(!text.contains(&config.password) && !text.contains(&config.base_url));
        let scope = std::fs::read_to_string(
            root.path().join(
                registry::registry()
                    .unwrap()
                    .path(registry::PathId::EventScope),
            ),
        )
        .unwrap();
        assert!(!scope.contains(&config.password) && !scope.contains(&config.base_url));
    }
    #[test]
    fn new_endpoint_credentials_or_root_cannot_clear_old_failure() {
        let root = tempfile::tempdir().unwrap();
        let other_root = tempfile::tempdir().unwrap();
        let store = Arc::new(Mutex::new(NotificationStore::new()));
        let endpoint = super::tests::endpoint();
        let original = config();
        let bytes = probe_candidate(&endpoint, &original).unwrap();
        assert!(publish(root.path(), &store, &bytes, false).unwrap().added);
        assert_eq!(
            publish(other_root.path(), &store, &bytes, true)
                .unwrap()
                .resolved,
            0
        );
        for field in 0..3 {
            let mut changed = original.clone();
            let mut endpoint = endpoint.clone();
            match field {
                0 => endpoint.endpoint_id = "different-endpoint".into(),
                1 => changed.password = "rotated".into(),
                _ => changed.remote_path = "another-folder".into(),
            }
            let changed_bytes = probe_candidate(&endpoint, &changed).unwrap();
            assert_eq!(
                publish(root.path(), &store, &changed_bytes, true)
                    .unwrap()
                    .resolved,
                0
            );
        }
        // Returning to old bytes creates a new candidate after scope rotation.
        assert_eq!(
            publish(root.path(), &store, &bytes, true).unwrap().resolved,
            0
        );
        assert!(!store.lock().unwrap().all()[0].resolved);
    }
    #[tokio::test]
    async fn notification_write_failure_does_not_change_network_success() {
        let root = tempfile::tempdir().unwrap();
        let store = Arc::new(Mutex::new(NotificationStore::new()));
        let bytes = probe_candidate(&super::tests::endpoint(), &config()).unwrap();
        assert!(publish(root.path(), &store, &bytes, false).unwrap().added);
        std::fs::remove_file(notifications_path(root.path())).unwrap();
        std::fs::create_dir(notifications_path(root.path())).unwrap();
        let (good, _) = client(Ok(()));
        assert!(probe_connection_observed(
            &good,
            &config(),
            Duration::from_secs(1),
            || Ok(true),
            |success| {
                assert!(publish(root.path(), &store, &bytes, success).is_err());
            }
        )
        .await
        .unwrap()
        .is_ok());
        assert!(!store.lock().unwrap().all()[0].resolved);
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cancelled_ipc_observer_does_not_release_probe_or_storage_ownership() {
        let entered = Arc::new(tokio::sync::Notify::new());
        let release = Arc::new(tokio::sync::Notify::new());
        let calls = Arc::new(AtomicUsize::new(0));
        let client = WebDavClient::new(Transport {
            result: Ok(()),
            calls: calls.clone(),
            entered: Some(entered.clone()),
            release: Some(release.clone()),
        });
        let writer_gate = Arc::new(crate::infrastructure::storage_writers::WriterGate::default());
        let operation_gate = Arc::new(tokio::sync::Mutex::new(()));
        let completed = Arc::new(AtomicUsize::new(0));
        let worker_completed = completed.clone();
        let worker_gate = operation_gate.clone();
        let writers = writer_gate.clone();
        let observer = tokio::spawn(async move {
            run_owned_sync_with_gates(writers, worker_gate, "injected sync probe", move || {
                let result = tauri::async_runtime::block_on(probe_connection_observed(
                    &client,
                    &config(),
                    Duration::from_secs(1),
                    || Ok(true),
                    |success| {
                        assert!(success);
                        worker_completed.fetch_add(1, Ordering::SeqCst);
                    },
                ))?;
                assert!(result.is_ok());
                Ok(())
            })
            .await
        });
        tokio::time::timeout(Duration::from_secs(2), entered.notified())
            .await
            .unwrap();
        observer.abort();
        assert!(observer.await.unwrap_err().is_cancelled());
        assert!(admit_sync_with_gate(operation_gate.clone()).is_err());
        assert!(writer_gate.freeze().unwrap().is_none());
        assert_eq!(completed.load(Ordering::SeqCst), 0);
        release.notify_one();
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                if completed.load(Ordering::SeqCst) == 1 && writer_gate.freeze().unwrap().is_some()
                {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert!(admit_sync_with_gate(operation_gate).is_ok());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}

#[cfg(test)]
mod owned_sync_tests {
    use super::*;
    use crate::infrastructure::storage_writers::WriterGate;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[tokio::test]
    async fn busy_and_frozen_operations_do_not_run_or_queue() {
        let writers = Arc::new(WriterGate::default());
        let operations = Arc::new(tokio::sync::Mutex::new(()));
        let busy = admit_sync_with_gate(operations.clone()).unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let result = run_owned_sync_with_gates(
            writers.clone(),
            operations.clone(),
            "busy sync",
            move || {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
        )
        .await;
        assert!(matches!(
            result,
            Err(AppError::TargetLockTimeout { timeout_ms: 0 })
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        drop(busy);
        let freeze = writers.freeze().unwrap().unwrap();
        let counter = calls.clone();
        assert!(
            run_owned_sync_with_gates(writers, operations.clone(), "frozen sync", move || {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .await
            .is_err()
        );
        drop(freeze);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(admit_sync_with_gate(operations).is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cancelled_observer_keeps_owner_through_committed_write_and_terminal_projection() {
        let root = tempfile::tempdir().unwrap();
        let committed = root.path().join("commit");
        let terminal = root.path().join("terminal");
        let entered = Arc::new(tokio::sync::Notify::new());
        let release = Arc::new(tokio::sync::Notify::new());
        let writers = Arc::new(WriterGate::default());
        let operations = Arc::new(tokio::sync::Mutex::new(()));
        let worker_writers = writers.clone();
        let worker_operations = operations.clone();
        let worker_committed = committed.clone();
        let worker_terminal = terminal.clone();
        let worker_entered = entered.clone();
        let worker_release = release.clone();
        let observer = tokio::spawn(async move {
            run_owned_sync_with_gates(
                worker_writers,
                worker_operations,
                "injected manual sync",
                move || {
                    std::fs::write(worker_committed, b"committed").unwrap();
                    worker_entered.notify_one();
                    tauri::async_runtime::block_on(worker_release.notified());
                    std::fs::write(worker_terminal, b"succeeded").unwrap();
                    Ok(())
                },
            )
            .await
        });
        tokio::time::timeout(std::time::Duration::from_secs(2), entered.notified())
            .await
            .unwrap();
        observer.abort();
        assert!(observer.await.unwrap_err().is_cancelled());
        assert_eq!(std::fs::read(&committed).unwrap(), b"committed");
        assert!(!terminal.exists());
        assert!(writers.freeze().unwrap().is_none());
        assert!(admit_sync_with_gate(operations.clone()).is_err());
        release.notify_one();
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                if terminal.exists() && writers.freeze().unwrap().is_some() {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(std::fs::read(terminal).unwrap(), b"succeeded");
        assert!(admit_sync_with_gate(operations).is_ok());
    }

    #[tokio::test]
    async fn worker_error_and_panic_release_both_admissions() {
        let writers = Arc::new(WriterGate::default());
        let operations = Arc::new(tokio::sync::Mutex::new(()));
        assert!(run_owned_sync_with_gates(
            writers.clone(),
            operations.clone(),
            "failed sync",
            || { Err::<(), _>(AppError::NotConfigured) }
        )
        .await
        .is_err());
        assert!(writers.freeze().unwrap().is_some());
        assert!(admit_sync_with_gate(operations.clone()).is_ok());
        assert!(run_owned_sync_with_gates(
            writers.clone(),
            operations.clone(),
            "panicked sync",
            || {
                panic!("injected worker panic");
                #[allow(unreachable_code)]
                Ok(())
            }
        )
        .await
        .is_err());
        assert!(writers.freeze().unwrap().is_some());
        assert!(admit_sync_with_gate(operations).is_ok());
    }
}
