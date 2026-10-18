//! File transfer stays in the unified sync domain; received policy is a draft.
use crate::infrastructure::webdav_client::{ReqwestWebDavTransport, WebDavConfig};
use crate::modules::model_config::remote::{self, UploadPlan};
use crate::{
    modules::model_config::{
        schema::*,
        transfer::{self, ExportScope, FileExportPlan, IncomingPlan, ModelPackage},
    },
    state::SharedDataRoot,
};
use serde::Serialize;

fn active_remote(root: &std::path::Path) -> ModelResult<(String, WebDavConfig, String)> {
    let config = crate::modules::sync::config::SyncConfigStore::new(root).load()?;
    let endpoint = config
        .active()
        .ok_or_else(|| invalid("not_configured", "请先在同步设置中配置活动WebDAV端点"))?;
    let webdav = crate::commands::sync::webdav_config(endpoint)?;
    webdav
        .validate()
        .map_err(|_| invalid("endpoint", "WebDAV端点配置无效"))?;
    let binding = crate::modules::model_config::store::sha256(&serde_json::to_vec(&(
        endpoint,
        &webdav.password,
    ))?);
    Ok((endpoint.endpoint_id.clone(), webdav, binding))
}
fn remote_operation<T>(
    root: &std::path::Path,
    action: impl FnOnce(
        &ReqwestWebDavTransport,
        &WebDavConfig,
        &str,
        &dyn Fn() -> ModelResult<()>,
    ) -> ModelResult<T>,
) -> ModelResult<T> {
    let (endpoint, config, binding) = active_remote(root)?;
    let _guard = crate::modules::sync::engine::SyncEndpointLocks::global()
        .acquire(&endpoint)
        .map_err(|_| invalid("sync_busy", "该端点正在同步，请等待原操作完成"))?;
    let lock = root.join("manager-state").join(format!(
        "model-remote-{}",
        crate::modules::model_config::store::sha256(endpoint.as_bytes())
            .trim_start_matches("sha256:")
    ));
    crate::modules::backup::safety::check_path(root, &lock, true)?;
    let _file_lock = crate::infrastructure::locking::TargetFileLock::lock(&lock)?;
    let unchanged = || {
        if active_remote(root)?.2 != binding {
            return Err(invalid(
                "endpoint_changed",
                "同步期间活动端点或凭据改变，已停止",
            ));
        }
        Ok(())
    };
    let result = action(
        &ReqwestWebDavTransport::new(),
        &config,
        &binding,
        &unchanged,
    )?;
    unchanged()?;
    Ok(result)
}
#[tauri::command]
pub async fn model_remote_prepare(
    scope: ExportScope,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<UploadPlan> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        remote_operation(&root, |transport, config, binding, _| {
            tauri::async_runtime::block_on(remote::prepare(
                &root, transport, config, binding, scope,
            ))
        })
    })
    .await
    .map_err(|_| invalid("transfer_worker", "模型上传准备中断"))?
}
#[tauri::command]
pub async fn model_remote_publish(
    plan_id: String,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<UploadPlan> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        crate::modules::model_config::feedback::transfer(&root, &plan_id, false, || {
            remote_operation(&root, |transport, config, binding, unchanged| {
                tauri::async_runtime::block_on(remote::publish(
                    &root, transport, config, &plan_id, binding, unchanged,
                ))
            })
        })
    })
    .await
    .map_err(|_| {
        invalid(
            "transfer_worker",
            "模型上传结果待核对，请使用原操作重新核对",
        )
    })?
}
#[tauri::command]
pub async fn model_remote_download(
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<IncomingPlan> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        crate::modules::model_config::feedback::transfer(
            &root,
            &uuid::Uuid::new_v4().to_string(),
            true,
            || {
                remote_operation(&root, |transport, config, _, unchanged| {
                    tauri::async_runtime::block_on(remote::download(
                        &root, transport, config, unchanged,
                    ))
                })
            },
        )
    })
    .await
    .map_err(|_| invalid("transfer_worker", "模型下载中断，本机配置未采用"))?
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedModels {
    pub path: String,
    pub digest: String,
    pub templates: usize,
    pub channels: usize,
}
#[tauri::command]
pub async fn model_transfer_export(
    plan_id: String,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<ExportedModels> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        crate::modules::model_config::feedback::transfer(&root, &plan_id, false, || {
            let package = transfer::file_package(&root, &plan_id)?;
            let directory = root.join("exports");
            crate::modules::backup::safety::check_path(&root, &directory, true)?;
            let path = directory.join(format!("model-config-{}.ocxd.json", uuid::Uuid::new_v4()));
            crate::modules::backup::safety::check_path(&root, &path, true)?;
            let bytes = serde_json::to_vec_pretty(&package)?;
            crate::infrastructure::atomic_write::atomic_write(&path, &bytes, 0o600)?;
            if crate::modules::backup::safety::read_file(&path, 64 * 1024 * 1024)? != bytes {
                return Err(invalid("export_readback", "导出文件读回失败"));
            }
            Ok(ExportedModels {
                path: path.to_string_lossy().into_owned(),
                digest: package.digest,
                templates: package.payload.templates.len(),
                channels: package
                    .payload
                    .policy
                    .as_ref()
                    .map(|p| p.channels.len())
                    .unwrap_or_default(),
            })
        })
    })
    .await
    .map_err(|_| invalid("transfer_worker", "导出任务中断"))?
}
#[tauri::command]
pub async fn model_transfer_prepare(
    scope: ExportScope,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<FileExportPlan> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        transfer::prepare_file(&root, scope)
    })
    .await
    .map_err(|_| invalid("transfer_worker", "导出准备中断"))?
}
#[tauri::command]
pub async fn model_transfer_receive(
    path: String,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<IncomingPlan> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        crate::modules::model_config::feedback::transfer(
            &root,
            &uuid::Uuid::new_v4().to_string(),
            true,
            || {
                let path = std::path::PathBuf::from(path);
                if !path.is_absolute() {
                    return Err(invalid("import_path", "请选择绝对文件路径"));
                }
                let bytes = crate::modules::backup::safety::read_file(&path, 64 * 1024 * 1024)?;
                let package: ModelPackage = serde_json::from_slice(&bytes).map_err(|_| {
                    invalid(
                        "import_format",
                        "所选文件不是受支持的模型配置包；原文件和本机配置保持",
                    )
                })?;
                transfer::receive(&root, package)
            },
        )
    })
    .await
    .map_err(|_| invalid("transfer_worker", "读取导入任务中断"))?
}
#[tauri::command]
pub async fn model_transfer_accept(
    incoming_id: String,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Snapshot> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        crate::modules::model_config::feedback::transfer(&root, &incoming_id, false, || {
            transfer::accept(&root, &incoming_id)
        })
    })
    .await
    .map_err(|_| invalid("transfer_worker", "接受导入结果待核对"))?
}
