//! Native file boundaries around the backend-owned v6 LocalFile operation.
use crate::modules::model_config::{
    file_transfer_v2::{
        DraftExportRequestV2, ExportPlanV2, FileTransferOwnerV2, FileTransferReceiptV2,
        ImportBindingsV2, ImportPreviewV2,
    },
    objects_overlay_v2::SelectionV2,
    saved_objects_capture_v2::CaptureRequestV2,
    schema::{invalid, ModelResult},
    source_v2::ReadBaseV2,
};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
use tauri_plugin_dialog::DialogExt;
use zeroize::Zeroizing;

#[derive(Clone)]
pub struct SharedConfigFileTransferV2(pub Arc<Mutex<FileTransferOwnerV2>>);
async fn work<T, F>(owner: SharedConfigFileTransferV2, action: F) -> ModelResult<T>
where
    T: Send + 'static,
    F: FnOnce(&mut FileTransferOwnerV2) -> ModelResult<T> + Send + 'static,
{
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        let mut owner = owner
            .0
            .lock()
            .map_err(|_| invalid("file_owner", "文件操作状态不可用"))?;
        action(&mut owner)
    })
    .await
    .map_err(|_| invalid("file_worker", "文件操作中断，原操作保持"))?
}
fn read_selected(path: &Path) -> ModelResult<Zeroizing<Vec<u8>>> {
    if !path.is_absolute() {
        return Err(invalid("file_path", "请选择本机文件"));
    }
    Ok(Zeroizing::new(crate::modules::backup::safety::read_file(
        path,
        64 * 1024 * 1024,
    )?))
}
fn write_and_verify(path: &Path, bytes: &[u8]) -> ModelResult<String> {
    if !path.is_absolute() {
        return Err(invalid("file_path", "请选择本机文件位置"));
    }
    let parent = path
        .parent()
        .ok_or_else(|| invalid("file_path", "文件位置无效"))?;
    crate::modules::backup::safety::check_path(parent, path, true)?;
    crate::infrastructure::atomic_write::atomic_write(path, bytes, 0o600)?;
    let actual = read_selected(path)?;
    Ok(crate::modules::model_config::schema_v2::digest(&actual))
}
#[tauri::command]
pub async fn prepare_config_file_export_v2(
    request: CaptureRequestV2,
    passphrase: Option<String>,
    owner: tauri::State<'_, SharedConfigFileTransferV2>,
) -> ModelResult<ExportPlanV2> {
    let passphrase = passphrase.map(Zeroizing::new);
    work(owner.inner().clone(), move |owner| {
        owner.prepare_export(request, passphrase)
    })
    .await
}
#[tauri::command]
pub async fn prepare_draft_config_file_export_v2(
    expected: ReadBaseV2,
    request: CaptureRequestV2,
    passphrase: Option<String>,
    owner: tauri::State<'_, SharedConfigFileTransferV2>,
) -> ModelResult<ExportPlanV2> {
    let passphrase = passphrase.map(Zeroizing::new);
    work(owner.inner().clone(), move |owner| {
        owner.prepare_draft_export(
            DraftExportRequestV2 {
                expected,
                capture: request,
            },
            passphrase,
        )
    })
    .await
}
#[tauri::command]
pub async fn save_config_file_export_v2(
    operation_id: String,
    app: tauri::AppHandle,
    owner: tauri::State<'_, SharedConfigFileTransferV2>,
) -> ModelResult<Option<FileTransferReceiptV2>> {
    work(owner.inner().clone(), move |owner| {
        // Cancellation leaves the prepared bytes untouched and creates no file.
        let Some(path) = app
            .dialog()
            .file()
            .set_title("导出配置")
            .set_file_name("OpenCodex.ocxdconf")
            .add_filter("OpenCodex 配置", &["ocxdconf"])
            .blocking_save_file()
        else {
            return Ok(None);
        };
        let path = path
            .into_path()
            .map_err(|_| invalid("file_path", "请选择本机文件位置"))?;
        let digest = write_and_verify(&path, owner.bytes(&operation_id)?)?;
        owner.finish_export(&operation_id, &digest).map(Some)
    })
    .await
}
#[tauri::command]
pub async fn cancel_config_file_export_v2(
    operation_id: String,
    owner: tauri::State<'_, SharedConfigFileTransferV2>,
) -> ModelResult<bool> {
    work(owner.inner().clone(), move |owner| {
        owner.cancel_export(&operation_id)
    })
    .await
}
#[tauri::command]
pub async fn open_config_file_import_v2(
    passphrase: Option<String>,
    scope: Option<SelectionV2>,
    bindings: ImportBindingsV2,
    app: tauri::AppHandle,
    owner: tauri::State<'_, SharedConfigFileTransferV2>,
) -> ModelResult<Option<ImportPreviewV2>> {
    let passphrase = passphrase.map(Zeroizing::new);
    work(owner.inner().clone(), move |owner| {
        let Some(path) = app
            .dialog()
            .file()
            .set_title("导入配置")
            .add_filter("OpenCodex 配置", &["ocxdconf"])
            .blocking_pick_file()
        else {
            return Ok(None);
        };
        let path = path
            .into_path()
            .map_err(|_| invalid("file_path", "请选择本机文件"))?;
        let mut bytes = read_selected(&path)?;
        owner
            .prepare_import(std::mem::take(&mut *bytes), passphrase, scope, bindings)
            .map(Some)
    })
    .await
}
#[tauri::command]
pub async fn reselect_config_file_import_v2(
    operation_id: String,
    expected_plan_digest: String,
    scope: SelectionV2,
    bindings: ImportBindingsV2,
    owner: tauri::State<'_, SharedConfigFileTransferV2>,
) -> ModelResult<ImportPreviewV2> {
    work(owner.inner().clone(), move |owner| {
        owner.reselect(&operation_id, &expected_plan_digest, scope, bindings)
    })
    .await
}
#[tauri::command]
pub async fn confirm_config_file_import_v2(
    operation_id: String,
    expected_plan_digest: String,
    backup_before_update: Option<bool>,
    owner: tauri::State<'_, SharedConfigFileTransferV2>,
) -> ModelResult<FileTransferReceiptV2> {
    work(owner.inner().clone(), move |owner| {
        owner.confirm_import(&operation_id, &expected_plan_digest, backup_before_update)
    })
    .await
}
#[tauri::command]
pub async fn cancel_config_file_import_v2(
    operation_id: String,
    owner: tauri::State<'_, SharedConfigFileTransferV2>,
) -> ModelResult<bool> {
    work(owner.inner().clone(), move |owner| {
        owner.cancel_import(&operation_id)
    })
    .await
}

#[tauri::command]
pub async fn reopen_config_file_export_v2(
    operation_id: String,
    owner: tauri::State<'_, SharedConfigFileTransferV2>,
) -> ModelResult<ExportPlanV2> {
    work(owner.inner().clone(), move |owner| {
        owner.reopen_export(&operation_id)
    })
    .await
}
#[tauri::command]
pub async fn reopen_config_file_import_v2(
    operation_id: String,
    owner: tauri::State<'_, SharedConfigFileTransferV2>,
) -> ModelResult<ImportPreviewV2> {
    work(owner.inner().clone(), move |owner| {
        owner.reopen_import(&operation_id)
    })
    .await
}
#[tauri::command]
pub async fn recoverable_config_file_operations_v2(
    owner: tauri::State<'_, SharedConfigFileTransferV2>,
) -> ModelResult<Vec<crate::modules::model_config::file_transfer_v2::RecoverableFileOperationV2>> {
    work(owner.inner().clone(), move |owner| {
        owner.recoverable_operations()
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn export_receipt_digest_is_actual_file_bytes_and_preserves_conflicting_objects() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("synthetic.ocxdconf");
        let digest = write_and_verify(&file, b"synthetic exact file bytes").unwrap();
        assert_eq!(
            digest,
            crate::modules::model_config::schema_v2::digest(&std::fs::read(&file).unwrap())
        );
        let directory = root.path().join("directory.ocxdconf");
        std::fs::create_dir(&directory).unwrap();
        assert!(write_and_verify(&directory, b"no").is_err());
        assert!(directory.is_dir());
    }
    #[cfg(unix)]
    #[test]
    fn native_selected_link_cannot_redirect_export_to_another_file() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("outside");
        std::fs::write(&target, b"preserve").unwrap();
        let link = root.path().join("link.ocxdconf");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(write_and_verify(&link, b"no").is_err());
        assert_eq!(std::fs::read(&target).unwrap(), b"preserve");
    }
}

/// Local inventory has no WebDAV dependency. Stable source identity bookkeeping
/// is local metadata, so it holds the same fixed-root admission as preparation.
#[tauri::command]
pub async fn catalog_config_file_v2(
    data_root: tauri::State<'_, crate::state::SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> ModelResult<crate::modules::model_config::sync_service_v2::LocalFileCatalogV2> {
    let root = data_root.0.clone();
    let home = context.opencodex_home.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        crate::modules::model_config::sync_service_v2::ProductionServiceBackendV2::new(
            root,
            home,
            Arc::new(
                crate::modules::model_config::saved_objects_capture_v2::SystemCaptureCredentialsV2,
            ),
        )?
        .local_file_catalog()
    })
    .await
    .map_err(|_| invalid("file_inventory", "本机配置目录读取中断，当前选择保持"))?
}
