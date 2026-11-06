//! Public sync commands delegate to the same backend-owned service and preserve
//! structured failure/recovery facts. Paths and credentials are never authority DTOs.
use crate::modules::model_config::{
    objects_overlay_v2::SelectionV2,
    sync_service_v2::{
        CatalogV2, ConfirmRequestV2, PreviewRequestV2, PreviewV2, ScopeRequestV2, ServiceSecretsV2,
        SyncServiceV2,
    },
    sync_v2::types::{
        ErrorCode, OperationFact, ResumeUploadRequest, Step, SyncFailure, SyncResult,
    },
};
use std::{collections::BTreeMap, future::Future, sync::Arc};
use zeroize::Zeroizing;

#[derive(Clone)]
pub struct SharedConfigSyncV2(Arc<SyncResult<Arc<SyncServiceV2>>>);
impl SharedConfigSyncV2 {
    pub fn production(root: std::path::PathBuf, home: std::path::PathBuf) -> Self {
        Self(Arc::new(
            SyncServiceV2::production(root, home).map(Arc::new),
        ))
    }
    fn service(&self) -> SyncResult<Arc<SyncServiceV2>> {
        self.0.as_ref().clone()
    }
}
fn failure() -> SyncFailure {
    SyncFailure {
        code: ErrorCode::SyncLocalStateIo,
        step: Step::Persist,
        retryability: crate::modules::model_config::sync_v2::types::Retryability::AfterRefresh,
        cause: None,
        http_status: None,
        recovery_actions: vec![],
    }
}
/// The actual blocking worker owns admission until every service sub-worker has
/// settled. Dropping the IPC observer cannot release a live storage writer.
async fn run<T, F>(future: F, writes_local_state: bool) -> SyncResult<T>
where
    T: Send + 'static,
    F: Future<Output = SyncResult<T>> + Send + 'static,
{
    let admission = if writes_local_state {
        Some(
            crate::infrastructure::storage_writers::global()
                .admit()
                .map_err(|_| failure())?,
        )
    } else {
        None
    };
    let runtime = tokio::runtime::Handle::current();
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        runtime.block_on(future)
    })
    .await
    .map_err(|_| failure())?
}
#[tauri::command]
pub async fn config_v2_catalog(
    read_passphrase: Option<String>,
    service: tauri::State<'_, SharedConfigSyncV2>,
) -> SyncResult<CatalogV2> {
    let service = service.service()?;
    let passphrase = read_passphrase.map(Zeroizing::new);
    // First successful catalog persists the initial selection locally.
    run(async move { service.catalog(passphrase).await }, true).await
}
#[tauri::command]
pub async fn config_v2_read_scope(
    service: tauri::State<'_, SharedConfigSyncV2>,
) -> SyncResult<Option<ScopeRequestV2>> {
    let service = service.service()?;
    run(async move { service.read_scope().await }, false).await
}
#[tauri::command]
pub async fn config_v2_save_scope(
    selection: SelectionV2,
    mcp_clients: BTreeMap<String, String>,
    assets_reviewed: bool,
    service: tauri::State<'_, SharedConfigSyncV2>,
) -> SyncResult<()> {
    let service = service.service()?;
    run(
        async move {
            service
                .save_scope(ScopeRequestV2 {
                    selection,
                    mcp_clients,
                    assets_reviewed,
                })
                .await
        },
        true,
    )
    .await
}
#[tauri::command]
pub async fn config_v2_preview(
    request: PreviewRequestV2,
    read_passphrase: Option<String>,
    write_passphrase: Option<String>,
    service: tauri::State<'_, SharedConfigSyncV2>,
) -> SyncResult<PreviewV2> {
    let service = service.service()?;
    let secrets = ServiceSecretsV2 {
        read_passphrase: read_passphrase.map(Zeroizing::new),
        write_passphrase: write_passphrase.map(Zeroizing::new),
    };
    run(async move { service.preview(request, secrets).await }, true).await
}
#[tauri::command]
pub async fn config_v2_confirm(
    request: ConfirmRequestV2,
    service: tauri::State<'_, SharedConfigSyncV2>,
) -> SyncResult<OperationFact> {
    let service = service.service()?;
    let original = service.session_operation_id(&request.session_id)?;
    run(
        async move {
            let result = service.confirm(request).await;
            crate::modules::model_config::feedback::v2::emit_sync(
                service.desktop_root(),
                &result,
                original.as_deref(),
            );
            result
        },
        true,
    )
    .await
}
#[tauri::command]
pub async fn config_v2_result(
    operation_id: String,
    service: tauri::State<'_, SharedConfigSyncV2>,
) -> SyncResult<OperationFact> {
    let service = service.service()?;
    run(async move { service.result(operation_id).await }, false).await
}
#[tauri::command]
pub async fn config_v2_verify(
    operation_id: String,
    read_passphrase: Option<String>,
    service: tauri::State<'_, SharedConfigSyncV2>,
) -> SyncResult<OperationFact> {
    let service = service.service()?;
    let passphrase = read_passphrase.map(Zeroizing::new);
    let original = operation_id.clone();
    run(
        async move {
            let result = service.verify(operation_id.clone(), passphrase).await;
            crate::modules::model_config::feedback::v2::emit_sync(
                service.desktop_root(),
                &result,
                Some(&original),
            );
            result
        },
        true,
    )
    .await
}
#[tauri::command]
pub async fn config_v2_resume(
    request: ResumeUploadRequest,
    read_passphrase: Option<String>,
    service: tauri::State<'_, SharedConfigSyncV2>,
) -> SyncResult<OperationFact> {
    let service = service.service()?;
    let passphrase = read_passphrase.map(Zeroizing::new);
    let original = request.operation_id.clone();
    run(
        async move {
            let result = service.resume_original(request, passphrase).await;
            crate::modules::model_config::feedback::v2::emit_sync(
                service.desktop_root(),
                &result,
                Some(&original),
            );
            result
        },
        true,
    )
    .await
}
#[tauri::command]
pub async fn config_v2_cancel(
    session_id: String,
    service: tauri::State<'_, SharedConfigSyncV2>,
) -> SyncResult<Option<OperationFact>> {
    let service = service.service()?;
    run(async move { service.cancel(session_id).await }, true).await
}

#[tauri::command]
pub async fn config_v2_resume_preflight(
    operation_id: String,
    read_passphrase: Option<String>,
    service: tauri::State<'_, SharedConfigSyncV2>,
) -> SyncResult<crate::modules::model_config::sync_service_v2::ResumePreflightV2> {
    let service = service.service()?;
    let passphrase = read_passphrase.map(Zeroizing::new);
    run(
        async move { service.resume_preflight(operation_id, passphrase).await },
        true,
    )
    .await
}
#[tauri::command]
pub async fn config_v2_recovery_preview(
    request: crate::modules::model_config::sync_service_v2::RecoveryRequestV2,
    read_passphrase: Option<String>,
    service: tauri::State<'_, SharedConfigSyncV2>,
) -> SyncResult<crate::modules::model_config::sync_service_v2::RecoveryPreviewV2> {
    let service = service.service()?;
    let passphrase = read_passphrase.map(Zeroizing::new);
    run(
        async move { service.recovery_preview(request, passphrase).await },
        true,
    )
    .await
}
#[tauri::command]
pub async fn config_v2_recovery_confirm(
    request: ConfirmRequestV2,
    service: tauri::State<'_, SharedConfigSyncV2>,
) -> SyncResult<crate::modules::model_config::sync_v2::types::RecoveryReceipt> {
    let service = service.service()?;
    let original = service.recovery_operation_id(&request.session_id)?;
    run(
        async move {
            let result = service.recovery_confirm(request).await;
            crate::modules::model_config::feedback::v2::emit_recovery(
                service.desktop_root(),
                &result,
                &original,
            );
            result
        },
        true,
    )
    .await
}
#[tauri::command]
pub async fn config_v2_recovery_result(
    recovery_plan_id: String,
    service: tauri::State<'_, SharedConfigSyncV2>,
) -> SyncResult<crate::modules::model_config::sync_v2::types::RecoveryReceipt> {
    let service = service.service()?;
    run(
        async move { service.recovery_result(recovery_plan_id).await },
        false,
    )
    .await
}

/// Original download continuation is local-only. It never borrows upload
/// preflight, a current cloud head, or a newly captured target baseline.
#[tauri::command]
pub async fn config_v2_resume_receive(
    operation_id: String,
    read_passphrase: Option<String>,
    service: tauri::State<'_, SharedConfigSyncV2>,
) -> SyncResult<OperationFact> {
    let service = service.service()?;
    let passphrase = read_passphrase.map(Zeroizing::new);
    let original = operation_id.clone();
    run(
        async move {
            let result = service
                .resume_receive(operation_id.clone(), passphrase)
                .await;
            crate::modules::model_config::feedback::v2::emit_sync(
                service.desktop_root(),
                &result,
                Some(&original),
            );
            result
        },
        true,
    )
    .await
}
