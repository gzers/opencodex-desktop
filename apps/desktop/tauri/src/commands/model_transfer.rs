//! File transfer stays in the unified sync domain; received policy is a draft.
use crate::infrastructure::webdav_client::{ReqwestWebDavTransport, WebDavConfig};
use crate::modules::model_config::config_remote;
use crate::{
    modules::model_config::{
        schema::*,
        transfer::{self, ExportScope, FileExportPlan, IncomingPlan},
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
pub async fn config_sync_scope(
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<config_remote::Scope> {
    let root = data_root.0.clone();
    tauri::async_runtime::spawn_blocking(move || config_remote::load_scope(&root))
        .await
        .map_err(|_| invalid("sync_scope", "设备范围读取中断"))?
}
#[tauri::command]
pub async fn config_sync_save_scope(
    mut scope: config_remote::Scope,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<config_remote::Scope> {
    scope.upstream_root = Some(context.opencodex_home.clone());
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        config_remote::save_scope(&root, scope)
    })
    .await
    .map_err(|_| invalid("sync_scope", "设备范围保存中断，请核对原范围"))?
}
#[tauri::command]
pub async fn config_sync_prepare(
    scope: config_remote::Scope,
    passphrase: Option<String>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, crate::state::SharedHomeDir>,
) -> ModelResult<config_remote::UploadView> {
    let upstream_root = context.opencodex_home.clone();
    let root = data_root.0.clone();
    let local_home = home.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        let password = passphrase.map(zeroize::Zeroizing::new);
        remote_operation(&root, |transport, config, binding, unchanged| {
            let mut scope = scope;
            scope.local_home = Some(local_home);
            scope.upstream_root = Some(upstream_root);
            let scope = config_remote::save_scope(&root, scope)?;
            tauri::async_runtime::block_on(config_remote::prepare(
                &root,
                transport,
                config,
                binding,
                scope,
                password.as_deref().map(|p| p.as_str()),
                unchanged,
            ))
        })
    })
    .await
    .map_err(|_| invalid("transfer_worker", "配置上传准备中断"))?
}
#[tauri::command]
pub async fn config_sync_publish(
    plan_id: String,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<config_remote::UploadView> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        crate::modules::model_config::feedback::transfer(&root, &plan_id, false, || {
            remote_operation(&root, |transport, config, binding, unchanged| {
                tauri::async_runtime::block_on(config_remote::publish(
                    &root, transport, config, &plan_id, binding, unchanged,
                ))
            })
        })
    })
    .await
    .map_err(|_| invalid("transfer_worker", "配置上传结果待核对，请保留原计划"))?
}
#[tauri::command]
pub async fn config_sync_download(
    passphrase: Option<String>,
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, crate::state::SharedHomeDir>,
) -> ModelResult<IncomingPlan> {
    let root = data_root.0.clone();
    let local_home = home.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        let password = passphrase.map(zeroize::Zeroizing::new);
        crate::modules::model_config::feedback::transfer(
            &root,
            &uuid::Uuid::new_v4().to_string(),
            true,
            || {
                remote_operation(&root, |transport, config, _, unchanged| {
                    tauri::async_runtime::block_on(config_remote::download_context(
                        &root,
                        transport,
                        config,
                        password.as_deref().map(|p| p.as_str()),
                        Some(&local_home),
                        unchanged,
                    ))
                })
            },
        )
    })
    .await
    .map_err(|_| invalid("transfer_worker", "统一配置下载中断，未采用"))?
}
#[tauri::command]
pub async fn config_sync_latest_upload(
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Option<config_remote::UploadView>> {
    let root = data_root.0.clone();
    tauri::async_runtime::spawn_blocking(move || config_remote::latest_upload(&root))
        .await
        .map_err(|_| invalid("transfer_worker", "原配置上传记录读取中断"))?
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
            let path = directory.join(format!(
                "opencodex-config-{}.ocxdconf",
                uuid::Uuid::new_v4()
            ));
            crate::modules::backup::safety::check_path(&root, &path, true)?;
            let bytes = transfer::file_bytes(&root, &plan_id)?;
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
    passphrase: Option<String>,
    include_keys: Option<bool>,
    manager_scope: Option<crate::modules::sync::portable_manager::Scope>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, crate::state::SharedHomeDir>,
) -> ModelResult<FileExportPlan> {
    let upstream_root = context.opencodex_home.clone();
    let root = data_root.0.clone();
    let local_home = home.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        let passphrase = passphrase.map(zeroize::Zeroizing::new);
        let mut manager_scope = manager_scope.unwrap_or_default();
        manager_scope.local_home = Some(local_home);
        manager_scope.upstream_root = Some(upstream_root);
        transfer::prepare_file_scope(
            &root,
            scope,
            passphrase.as_deref().map(|p| p.as_str()),
            include_keys.unwrap_or(false),
            &manager_scope,
        )
    })
    .await
    .map_err(|_| invalid("transfer_worker", "导出准备中断"))?
}
#[tauri::command]
pub async fn model_transfer_receive(
    path: String,
    passphrase: Option<String>,
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, crate::state::SharedHomeDir>,
) -> ModelResult<IncomingPlan> {
    let root = data_root.0.clone();
    let local_home = home.0.clone();
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
                let passphrase = passphrase.map(zeroize::Zeroizing::new);
                crate::modules::model_config::file_objects::receive_context(
                    &root,
                    &bytes,
                    passphrase.as_deref().map(|p| p.as_str()),
                    Some(&local_home),
                )
            },
        )
    })
    .await
    .map_err(|_| invalid("transfer_worker", "读取导入任务中断"))?
}
#[tauri::command]
pub async fn model_transfer_accept(
    incoming_id: String,
    adopt_keys: Option<bool>,
    passphrase: Option<String>,
    adopt_webdav_passwords: Option<bool>,
    adopt_extension_credentials: Option<bool>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Snapshot> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        crate::modules::model_config::feedback::transfer(&root, &incoming_id, false, || {
            let passphrase = passphrase.map(zeroize::Zeroizing::new);
            crate::modules::model_config::file_objects::accept_extended(
                &root,
                &incoming_id,
                adopt_keys.unwrap_or(false),
                adopt_webdav_passwords.unwrap_or(false),
                adopt_extension_credentials.unwrap_or(false),
                passphrase.as_deref().map(|p| p.as_str()),
            )
        })
    })
    .await
    .map_err(|_| invalid("transfer_worker", "接受导入结果待核对"))?
}
#[tauri::command]
pub async fn model_transfer_select(
    incoming_id: String,
    scope: crate::modules::model_config::transfer::ReceiveScope,
    passphrase: Option<String>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<IncomingPlan> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        let passphrase = passphrase.map(zeroize::Zeroizing::new);
        transfer::select_received_context(
            &root,
            &incoming_id,
            scope,
            passphrase.as_deref().map(|p| p.as_str()),
        )
    })
    .await
    .map_err(|_| invalid("transfer_worker", "接收范围比较中断，配置未采用"))?
}
#[tauri::command]
pub async fn portable_latest_incoming(
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Option<IncomingPlan>> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        crate::modules::model_config::file_objects::latest(&root)
    })
    .await
    .map_err(|_| invalid("transfer_worker", "上次接收记录读取中断"))?
}
#[tauri::command]
pub async fn portable_recompare(
    incoming_id: String,
    passphrase: Option<String>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<IncomingPlan> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        let passphrase = passphrase.map(zeroize::Zeroizing::new);
        crate::modules::model_config::file_objects::recompare(
            &root,
            &incoming_id,
            passphrase.as_deref().map(|p| p.as_str()),
        )
    })
    .await
    .map_err(|_| invalid("transfer_worker", "原文件重新比较中断，未采用"))?
}
#[tauri::command]
pub async fn portable_open_received(
    incoming_id: String,
    passphrase: Option<String>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<IncomingPlan> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        let passphrase = passphrase.map(zeroize::Zeroizing::new);
        crate::modules::model_config::file_objects::open_received(
            &root,
            &incoming_id,
            passphrase.as_deref().map(|p| p.as_str()),
        )
    })
    .await
    .map_err(|_| invalid("transfer_worker", "原文件核对中断，未采用"))?
}
#[tauri::command]
pub async fn portable_extension_catalog(
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, crate::state::SharedHomeDir>,
) -> ModelResult<crate::modules::sync::portable_assets::Catalog> {
    let root = data_root.0.clone();
    let home = home.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        crate::modules::sync::portable_assets::catalog(&root, &home)
    })
    .await
    .map_err(|_| invalid("asset_discovery", "扩展资源列表读取中断"))?
}

#[tauri::command]
pub async fn portable_account_catalog(
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> ModelResult<Vec<crate::modules::sync::portable_accounts::CatalogEntry>> {
    let root = data_root.0.clone();
    let upstream = context.opencodex_home.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        crate::modules::sync::portable_accounts::catalog(&root, &upstream)
    })
    .await
    .map_err(|_| invalid("account_source", "账户信息读取中断"))?
}
