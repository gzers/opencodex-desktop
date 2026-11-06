//! Scope restoration uses backend-frozen original images and opaque plans.
use crate::{
    modules::model_config::{
        schema::{invalid, ModelResult},
        sync_local_v2::restore::{self, BackupStatusV2, RestorePreviewV2, RestoreReceiptV2},
    },
    state::{SharedDataRoot, SharedProcessContext},
};
use std::path::{Path, PathBuf};
async fn work<T: Send + 'static>(
    d: PathBuf,
    h: PathBuf,
    write: bool,
    action: impl FnOnce(&Path, &Path) -> ModelResult<T> + Send + 'static,
) -> ModelResult<T> {
    let admission = if write {
        Some(crate::infrastructure::storage_writers::global().admit()?)
    } else {
        None
    };
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        action(&d, &h)
    })
    .await
    .map_err(|_| invalid("scope_restore_worker", "范围备份操作中断，原记录已保留"))?
}
#[tauri::command]
pub async fn transfer_v2_backup_list(
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
) -> ModelResult<Vec<BackupStatusV2>> {
    work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        false,
        restore::list,
    )
    .await
}
#[tauri::command]
pub async fn transfer_v2_backup_preview(
    original_operation_id: String,
    groups: Vec<String>,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
) -> ModelResult<RestorePreviewV2> {
    work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        true,
        move |d, h| restore::preview(d, h, &original_operation_id, &groups),
    )
    .await
}
#[tauri::command]
pub async fn transfer_v2_backup_restore(
    restore_id: String,
    plan_digest: String,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
) -> ModelResult<RestoreReceiptV2> {
    work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        true,
        move |d, h| {
            // A known original backend plan, rather than arbitrary IPC text,
            // provides the identity used for its existing progress observer.
            let known = restore::operations(d, h, None)?
                .iter()
                .any(|o| o.restore_id == restore_id && o.plan_digest == plan_digest);
            let result = restore::restore(d, h, &restore_id, &plan_digest);
            if known {
                crate::modules::model_config::feedback::v2::emit_scope_restore(
                    d,
                    &result,
                    &restore_id,
                );
            }
            result
        },
    )
    .await
}

#[tauri::command]
pub async fn transfer_v2_backup_restore_operations(
    original_operation_id: Option<String>,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
) -> ModelResult<Vec<crate::modules::model_config::sync_local_v2::restore::RestoreOperationV2>> {
    work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        false,
        move |d, h| restore::operations(d, h, original_operation_id.as_deref()),
    )
    .await
}
