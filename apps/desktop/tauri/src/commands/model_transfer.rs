//! File transfer shares the sync domain; ordinary snapshots commit saved files.
use crate::infrastructure::webdav_client::{ReqwestWebDavTransport, WebDavConfig};
use crate::modules::model_config::config_remote;
use crate::{
    modules::model_config::{
        schema::*,
        transfer::{self, ExportScope, FileExportPlan, IncomingPlan},
    },
    state::SharedDataRoot,
};

fn require_file_plan(root: &std::path::Path, home: &std::path::Path, id: &str) -> ModelResult<()> {
    let plan = transfer::read_incoming(root, id)?;
    if plan.file_home.as_deref() != Some(home) {
        return Err(invalid(
            "incoming_source",
            "接收记录属于旧来源或其它配置目录，请重新比较原文件；当前编辑保持",
        ));
    }
    Ok(())
}
fn cached_password(
    root: &std::path::Path,
    id: &str,
) -> ModelResult<Option<zeroize::Zeroizing<String>>> {
    let primary = crate::modules::model_config::transfer_protection::read_password(root)?;
    match crate::modules::model_config::file_objects::open_received(
        root,
        id,
        primary.as_deref().map(|p| p.as_str()),
    ) {
        Ok(_) => Ok(primary),
        Err(error)
            if ["passphrase_required", "portable_authentication"]
                .contains(&error.code.as_str()) =>
        {
            let previous =
                crate::modules::model_config::transfer_protection::previous_password(root)?;
            if previous.is_some() {
                crate::modules::model_config::file_objects::open_received(
                    root,
                    id,
                    previous.as_deref().map(|p| p.as_str()),
                )?;
                Ok(previous)
            } else {
                Err(error)
            }
        }
        Err(e) => Err(e),
    }
}
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
    context: tauri::State<'_, crate::state::SharedProcessContext>,
    home: tauri::State<'_, crate::state::SharedHomeDir>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<config_remote::Scope> {
    let root = data_root.0.clone();
    let upstream = context.opencodex_home.clone();
    let home = home.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        let result = remote_operation(&root, |transport, config, binding, unchanged| {
            let password = crate::modules::model_config::transfer_protection::read_password(&root)?;
            let previous =
                crate::modules::model_config::transfer_protection::previous_password(&root)?;
            let scope = tauri::async_runtime::block_on(config_remote::open_scope(
                &root,
                transport,
                config,
                binding,
                &upstream,
                &home,
                password.as_deref().map(|p| p.as_str()),
                previous.as_deref().map(|p| p.as_str()),
                unchanged,
            ))?;
            unchanged()?;
            Ok(scope)
        });
        match result {
            Err(e) if e.code == "not_configured" => config_remote::load_scope(&root),
            other => other,
        }
    })
    .await
    .map_err(|_| invalid("sync_scope", "设备范围读取中断"))?
}

#[tauri::command]
pub async fn config_sync_observe(
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<config_remote::RemoteVersion> {
    let root = data_root.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        // A read-only observation does not take the publication lock or create
        // manager-state files. Pointer read-back and the endpoint revision guard
        // reject races without blocking a user's range/sync operation.
        let (_, config, binding) = active_remote(&root)?;
        let password = crate::modules::model_config::transfer_protection::read_password(&root)?;
        let previous = crate::modules::model_config::transfer_protection::previous_password(&root)?;
        let result = tauri::async_runtime::block_on(config_remote::observe_version(
            &ReqwestWebDavTransport::new(),
            &config,
            password.as_deref().map(|p| p.as_str()),
            previous.as_deref().map(|p| p.as_str()),
        ))?;
        if active_remote(&root)?.2 != binding {
            return Err(invalid(
                "endpoint_changed",
                "读取版本期间连接已改变，未采用旧结果",
            ));
        }
        Ok(result)
    })
    .await
    .map_err(|_| invalid("remote_observation", "云端版本读取中断，请重新检测"))?
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
        scope.cloud_binding = active_remote(&root).ok().map(|v| v.2);
        config_remote::require_known_scope(&root)?;
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
        let password = match passphrase {
            Some(value) => Some(zeroize::Zeroizing::new(value)),
            None => crate::modules::model_config::transfer_protection::password(&root)?,
        };
        let read_password =
            crate::modules::model_config::transfer_protection::read_password(&root)?;
        let previous_password =
            crate::modules::model_config::transfer_protection::previous_password(&root)?;
        remote_operation(&root, |transport, config, binding, unchanged| {
            let mut scope = scope;
            scope.cloud_binding = Some(binding.into());
            scope.local_home = Some(local_home);
            scope.upstream_root = Some(upstream_root);
            scope.upstream_snapshot = None;
            let scope = config_remote::save_scope(&root, scope)?;
            tauri::async_runtime::block_on(config_remote::prepare_with_protection(
                &root,
                transport,
                config,
                binding,
                scope,
                password.as_deref().map(|p| p.as_str()),
                read_password.as_deref().map(|p| p.as_str()),
                previous_password.as_deref().map(|p| p.as_str()),
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
    context: tauri::State<'_, crate::state::SharedProcessContext>,
    home: tauri::State<'_, crate::state::SharedHomeDir>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<config_remote::UploadView> {
    let root = data_root.0.clone();
    let config_home = context.opencodex_home.clone();
    let local_home = home.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        crate::modules::model_config::feedback::transfer_registered(
            &root,
            &plan_id,
            "config-snapshot-sync",
            || {
                crate::modules::model_config::transfer_results::upload(&root, &plan_id, || {
                    remote_operation(&root, |transport, config, binding, unchanged| {
                        let mut scope = config_remote::load_scope(&root)?;
                        scope.cloud_binding = Some(binding.into());
                        scope.upstream_root = Some(config_home);
                        scope.local_home = Some(local_home);
                        tauri::async_runtime::block_on(config_remote::publish(
                            &root,
                            transport,
                            config,
                            &plan_id,
                            binding,
                            || {
                                unchanged()?;
                                config_remote::verify_upload_source(&root, &plan_id, &scope)
                            },
                        ))
                    })
                })
            },
        )
    })
    .await
    .map_err(|_| invalid("transfer_worker", "配置上传结果待核对，请保留原计划"))?
}
#[tauri::command]
pub async fn config_sync_download(
    passphrase: Option<String>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, crate::state::SharedHomeDir>,
) -> ModelResult<IncomingPlan> {
    let config_home = context.opencodex_home.clone();
    let root = data_root.0.clone();
    let local_home = home.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        // 优先用调用方给的口令；没给就读本机记下的，读不到才由界面提示输入。
        let explicit = passphrase.is_some();
        let password = match passphrase {
            Some(value) => Some(zeroize::Zeroizing::new(value)),
            None => crate::modules::model_config::transfer_protection::read_password(&root)?,
        };
        crate::modules::model_config::feedback::receive_transfer(&root, || {
            remote_operation(&root, |transport, config, binding, unchanged| {
                let download = |password: Option<&str>| {
                    tauri::async_runtime::block_on(config_remote::download_sync(
                        &root,
                        transport,
                        config,
                        password,
                        Some(&local_home),
                        &config_home,
                        binding,
                        unchanged,
                    ))
                };
                let result = download(password.as_deref().map(|p| p.as_str()));
                match result {
                    Err(error)
                        if !explicit
                            && ["passphrase_required", "portable_authentication"]
                                .contains(&error.code.as_str()) =>
                    {
                        let previous =
                            crate::modules::model_config::transfer_protection::previous_password(
                                &root,
                            )?;
                        if previous.is_some() {
                            download(previous.as_deref().map(|p| p.as_str()))
                        } else {
                            Err(error)
                        }
                    }
                    other => other,
                }
            })
        })
    })
    .await
    .map_err(|_| invalid("transfer_worker", "统一配置下载中断，未采用"))?
}
/// 同步按钮：判断本次以本机还是云端为新版。
#[tauri::command]
pub async fn config_sync_check(
    context: tauri::State<'_, crate::state::SharedProcessContext>,
    home: tauri::State<'_, crate::state::SharedHomeDir>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<crate::modules::model_config::sync_state::Check> {
    let root = data_root.0.clone();
    let config_home = context.opencodex_home.clone();
    let local_home = home.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        remote_operation(&root, |transport, config, binding, unchanged| {
            let read_password =
                crate::modules::model_config::transfer_protection::read_password(&root)?;
            let previous =
                crate::modules::model_config::transfer_protection::previous_password(&root)?;
            let catalog = tauri::async_runtime::block_on(config_remote::refresh_catalog(
                &root,
                transport,
                config,
                binding,
                read_password.as_deref().map(|p| p.as_str()),
                previous.as_deref().map(|p| p.as_str()),
            ))?;
            if let Some(catalog) = catalog.as_ref() {
                config_remote::initialize_remote_scope(
                    &root,
                    binding,
                    catalog,
                    &config_home,
                    &local_home,
                )?;
            }
            let mut scope = config_remote::load_scope(&root)?;
            scope.cloud_binding = Some(binding.into());
            scope.upstream_root = Some(config_home);
            scope.local_home = Some(local_home);
            let result =
                tauri::async_runtime::block_on(config_remote::check_source_with_password(
                    &root,
                    transport,
                    config,
                    binding,
                    &scope,
                    read_password.as_deref().map(|p| p.as_str()),
                    previous.as_deref().map(|p| p.as_str()),
                ))?;
            unchanged()?;
            Ok(result)
        })
    })
    .await
    .map_err(|_| invalid("transfer_worker", "同步方向检查中断"))?
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
pub use crate::modules::model_config::file_exports::Exported as ExportedModels;
/// target_dir 为空时写受管 exports/；指定目录时按用户选择落盘。
/// 非受管目录的写入同样做读回校验，确认落盘内容与计划一致。
#[tauri::command]
pub async fn model_transfer_export(
    plan_id: String,
    target_dir: Option<String>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<ExportedModels> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        crate::modules::model_config::feedback::transfer_registered(
            &root,
            &plan_id,
            "config-snapshot-export",
            || {
                crate::modules::model_config::transfer_results::export(&root, &plan_id, || {
                    crate::modules::model_config::file_exports::export(
                        &root,
                        &plan_id,
                        target_dir.as_deref(),
                    )
                })
            },
        )
    })
    .await
    .map_err(|_| invalid("transfer_worker", "导出任务中断"))?
}
#[tauri::command]
pub async fn model_transfer_prepare_draft(
    scope: ExportScope,
    draft: transfer::DraftCandidate,
    policy: Option<Policy>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
    passphrase: Option<String>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<FileExportPlan> {
    let config_home = context.opencodex_home.clone();
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        let passphrase = passphrase.map(zeroize::Zeroizing::new);
        transfer::prepare_memory_file(
            &root,
            &config_home,
            scope,
            draft,
            policy.ok_or_else(|| invalid("draft_export", "请选择当前内存中的渠道草稿"))?,
            passphrase.as_deref().map(|p| p.as_str()),
        )
    })
    .await
    .map_err(|_| invalid("transfer_worker", "草稿候选导出准备中断"))?
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
        manager_scope.upstream_snapshot = None;
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
    context: tauri::State<'_, crate::state::SharedProcessContext>,
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, crate::state::SharedHomeDir>,
) -> ModelResult<IncomingPlan> {
    let config_home = context.opencodex_home.clone();
    let root = data_root.0.clone();
    let local_home = home.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        crate::modules::model_config::feedback::receive_transfer(&root, || {
            let path = std::path::PathBuf::from(path);
            if !path.is_absolute() {
                return Err(invalid("import_path", "请选择绝对文件路径"));
            }
            let bytes = crate::modules::backup::safety::read_file(&path, 64 * 1024 * 1024)?;
            let passphrase = passphrase.map(zeroize::Zeroizing::new);
            crate::modules::model_config::file_objects::receive_context_file(
                &root,
                &bytes,
                passphrase.as_deref().map(|p| p.as_str()),
                Some(&local_home),
                &config_home,
            )
        })
    })
    .await
    .map_err(|_| invalid("transfer_worker", "读取导入任务中断"))?
}
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn model_transfer_accept(
    incoming_id: String,
    adopt_keys: Option<bool>,
    passphrase: Option<String>,
    adopt_webdav_passwords: Option<bool>,
    adopt_extension_credentials: Option<bool>,
    backup_before_update: Option<bool>,
    credential_ids: Option<Vec<String>>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
    home: tauri::State<'_, crate::state::SharedHomeDir>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Snapshot> {
    let config_home = context.opencodex_home.clone();
    let local_home = home.0.clone();
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        require_file_plan(&root, &config_home, &incoming_id)?;
        let is_sync = transfer::read_incoming(&root, &incoming_id)?.sync.is_some();
        let already_complete =
            crate::modules::model_config::file_objects::completed(&root, &incoming_id)?;
        let event = if is_sync {
            "config-snapshot-sync"
        } else {
            "config-snapshot-import"
        };
        crate::modules::model_config::feedback::transfer_registered(
            &root,
            &incoming_id,
            event,
            || {
                let intent = crate::modules::model_config::transfer_results::RetryIntent {
                    adopt_keys: adopt_keys.unwrap_or(false),
                    adopt_webdav_passwords: adopt_webdav_passwords.unwrap_or(false),
                    adopt_extension_credentials: adopt_extension_credentials.unwrap_or(false),
                    backup_before_update: backup_before_update.unwrap_or(true),
                    credential_ids: credential_ids.clone(),
                };
                crate::modules::model_config::transfer_results::import(
                    &root,
                    &incoming_id,
                    intent,
                    || {
                        let accept = || {
                            let passphrase = match passphrase.filter(|p| !p.is_empty()) {
                                Some(p) => Some(zeroize::Zeroizing::new(p)),
                                None => {
                                    if is_sync && !already_complete {
                                        cached_password(&root, &incoming_id)?
                                    } else {
                                        None
                                    }
                                }
                            };
                            crate::modules::model_config::file_objects::freeze_credential_choice(
                                &root,
                                &incoming_id,
                                credential_ids,
                            )?;
                            crate::modules::model_config::transfer_backup::prepare(
                                &root,
                                &incoming_id,
                                backup_before_update.unwrap_or(true),
                            )?;
                            let result =
                                crate::modules::model_config::file_objects::accept_extended(
                                    &root,
                                    &incoming_id,
                                    adopt_keys.unwrap_or(false),
                                    adopt_webdav_passwords.unwrap_or(false),
                                    adopt_extension_credentials.unwrap_or(false),
                                    passphrase.as_deref().map(|p| p.as_str()),
                                );
                            // Keep the original snapshot even on partial adoption; finalization
                            // is mandatory before acknowledging a successful operation.
                            crate::modules::model_config::transfer_backup::finish(
                                &root,
                                &incoming_id,
                            )?;
                            let snapshot = result?;
                            if is_sync && !already_complete {
                                let mut scope = config_remote::load_scope(&root)?;
                                scope.cloud_binding = Some(active_remote(&root)?.2);
                                scope.upstream_root = Some(config_home.clone());
                                scope.local_home = Some(local_home);
                                let (local, key) = config_remote::source_state(&root, &scope)?;
                                let binding = active_remote(&root)?.2;
                                crate::modules::model_config::sync_state::promote_pending_content(
                                    &root,
                                    &incoming_id,
                                    &binding,
                                    &local,
                                    &key,
                                )?;
                            }
                            Ok(snapshot)
                        };
                        if is_sync && !already_complete {
                            remote_operation(&root, |transport, config, binding, unchanged| {
                                tauri::async_runtime::block_on(
                                    crate::modules::model_config::sync_state::verify_pending(
                                        &root,
                                        &incoming_id,
                                        binding,
                                        transport,
                                        config,
                                    ),
                                )?;
                                unchanged()?;
                                accept()
                            })
                        } else {
                            accept()
                        }
                    },
                )
            },
        )
    })
    .await
    .map_err(|_| invalid("transfer_worker", "接受导入结果待核对"))?
}
#[tauri::command]
pub async fn model_transfer_select(
    incoming_id: String,
    scope: crate::modules::model_config::transfer::ReceiveScope,
    passphrase: Option<String>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<IncomingPlan> {
    let config_home = context.opencodex_home.clone();
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        require_file_plan(&root, &config_home, &incoming_id)?;
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
    context: tauri::State<'_, crate::state::SharedProcessContext>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<IncomingPlan> {
    let config_home = context.opencodex_home.clone();
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        let passphrase = passphrase.map(zeroize::Zeroizing::new);
        crate::modules::model_config::file_objects::recompare_file(
            &root,
            &incoming_id,
            passphrase.as_deref().map(|p| p.as_str()),
            &config_home,
        )
    })
    .await
    .map_err(|_| invalid("transfer_worker", "原文件重新比较中断，未采用"))?
}
#[tauri::command]
pub async fn portable_open_received(
    incoming_id: String,
    passphrase: Option<String>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<IncomingPlan> {
    let config_home = context.opencodex_home.clone();
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        require_file_plan(&root, &config_home, &incoming_id)?;
        let passphrase = match passphrase.filter(|p| !p.is_empty()) {
            Some(p) => Some(zeroize::Zeroizing::new(p)),
            None => {
                if transfer::read_incoming(&root, &incoming_id)?.sync.is_some() {
                    cached_password(&root, &incoming_id)?
                } else {
                    None
                }
            }
        };
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

#[tauri::command]
pub async fn portable_channel_catalog(
    context: tauri::State<'_, crate::state::SharedProcessContext>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Vec<ChannelPolicy>> {
    let root = data_root.0.clone();
    let config_home = context.opencodex_home.clone();
    tauri::async_runtime::spawn_blocking(move || {
        crate::modules::model_config::portable_channels::catalog_file(&root, &config_home)
    })
    .await
    .map_err(|_| invalid("provider_scope", "读取当前渠道文件中断"))?
}

#[tauri::command]
pub async fn transfer_choices(
    remote: bool,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Option<crate::modules::model_config::transfer_choices::Choice>> {
    let root = data_root.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        crate::modules::model_config::transfer_choices::load(&root, remote)
    })
    .await
    .map_err(|_| invalid("transfer_choices", "范围读取中断"))?
}
#[tauri::command]
pub async fn save_transfer_choices(
    remote: bool,
    choice: crate::modules::model_config::transfer_choices::Choice,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<crate::modules::model_config::transfer_choices::Choice> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        crate::modules::model_config::transfer_choices::save(&root, remote, choice)
    })
    .await
    .map_err(|_| invalid("transfer_choices", "范围保存中断"))?
}

#[tauri::command]
pub async fn transfer_protection(
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<crate::modules::model_config::transfer_protection::Status> {
    let root = data_root.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        crate::modules::model_config::transfer_protection::status(&root)
    })
    .await
    .map_err(|_| invalid("sync_protection", "保护设置读取中断"))?
}
#[tauri::command]
pub async fn save_transfer_protection(
    enabled: bool,
    passphrase: Option<String>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<crate::modules::model_config::transfer_protection::Status> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        let passphrase = passphrase.map(zeroize::Zeroizing::new);
        crate::modules::model_config::transfer_protection::save(
            &root,
            enabled,
            passphrase.as_deref().map(|v| v.as_str()),
        )
    })
    .await
    .map_err(|_| invalid("sync_protection", "保护设置保存中断"))?
}

#[tauri::command]
pub async fn transfer_backup_status(
    incoming_id: String,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Option<crate::modules::model_config::transfer_backup::Status>> {
    let root = data_root.0.clone();
    let home = context.opencodex_home.clone();
    tauri::async_runtime::spawn_blocking(move || {
        require_file_plan(&root, &home, &incoming_id)?;
        crate::modules::model_config::transfer_backup::status(&root, &incoming_id)
    })
    .await
    .map_err(|_| invalid("scope_backup", "备份状态读取中断"))?
}
#[tauri::command]
pub async fn restore_transfer_backup(
    incoming_id: String,
    groups: Vec<String>,
    baseline: String,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Snapshot> {
    let root = data_root.0.clone();
    let home = context.opencodex_home.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        require_file_plan(&root, &home, &incoming_id)?;
        crate::modules::model_config::transfer_backup::restore_checked(
            &root,
            &incoming_id,
            &groups,
            &baseline,
        )?;
        crate::modules::model_config::config_file::snapshot(&root, &home)
    })
    .await
    .map_err(|_| invalid("scope_backup", "恢复结果待核对"))?
}
#[tauri::command]
pub async fn import_credential_choices(
    preferences: Option<crate::modules::model_config::transfer_choices::ImportCredentials>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<crate::modules::model_config::transfer_choices::ImportCredentials> {
    let root = data_root.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        crate::modules::model_config::transfer_choices::import_credentials(&root, preferences)
    })
    .await
    .map_err(|_| invalid("transfer_choices", "导入凭据偏好读写中断"))?
}

#[tauri::command]
pub async fn open_transfer_export_directory(
    plan_id: String,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<()> {
    let root = data_root.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let dir = crate::modules::model_config::file_exports::directory(&root, &plan_id)?;
        crate::commands::workspace::open_path(&dir)
            .map_err(|_| invalid("export_directory", "无法打开本次导出目录"))?;
        crate::modules::model_config::feedback::export_directory_opened(&root, &plan_id);
        Ok(())
    })
    .await
    .map_err(|_| invalid("export_directory", "打开导出目录任务中断"))?
}

#[tauri::command]
pub async fn transfer_result(
    operation_id: Option<String>,
    mode: Option<String>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Option<crate::modules::model_config::transfer_results::Receipt>> {
    let root = data_root.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        if operation_id.is_none() {
            if let Some(mode) = mode {
                return crate::modules::model_config::transfer_results::read_mode(&root, &mode);
            }
        }
        crate::modules::model_config::transfer_results::read(&root, operation_id.as_deref())
    })
    .await
    .map_err(|_| invalid("transfer_result", "结果读取任务中断"))?
}

#[tauri::command]
pub async fn model_transfer_protect(
    plan_id: String,
    passphrase: Option<String>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<FileExportPlan> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        let passphrase = passphrase.map(zeroize::Zeroizing::new);
        crate::modules::model_config::file_exports::protect(
            &root,
            &plan_id,
            passphrase.as_deref().map(|p| p.as_str()),
        )
    })
    .await
    .map_err(|_| invalid("transfer_worker", "文件保护任务中断，未重新读取业务内容"))?
}

#[tauri::command]
pub async fn model_transfer_discard_unattempted(
    plan_id: String,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<bool> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        crate::modules::model_config::file_exports::discard_unattempted(&root, &plan_id)
    })
    .await
    .map_err(|_| invalid("transfer_worker", "取消导出预览的冻结计划清理中断"))?
}

#[tauri::command]
pub async fn list_transfer_backups(
    context: tauri::State<'_, crate::state::SharedProcessContext>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Vec<crate::modules::model_config::transfer_backup::Status>> {
    let root = data_root.0.clone();
    let home = context.opencodex_home.clone();
    tauri::async_runtime::spawn_blocking(move || {
        crate::modules::model_config::transfer_backup::list(&root, &home)
    })
    .await
    .map_err(|_| invalid("scope_backup", "读取更新前备份任务中断"))?
}
#[tauri::command]
pub async fn preview_transfer_restore(
    incoming_id: String,
    groups: Vec<String>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<String> {
    let root = data_root.0.clone();
    let home = context.opencodex_home.clone();
    tauri::async_runtime::spawn_blocking(move || {
        require_file_plan(&root, &home, &incoming_id)?;
        crate::modules::model_config::transfer_backup::restore_baseline(
            &root,
            &incoming_id,
            &groups,
        )
    })
    .await
    .map_err(|_| invalid("scope_restore", "恢复预览读取中断"))?
}

#[tauri::command]
pub async fn config_sync_catalog(
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Option<config_remote::CloudCatalog>> {
    let root = data_root.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let binding = match active_remote(&root) {
            Ok(v) => v.2,
            Err(error) if error.code == "not_configured" => return Ok(None),
            Err(error) => return Err(error),
        };
        config_remote::cached_catalog(&root, &binding)
    })
    .await
    .map_err(|_| invalid("sync_catalog", "云端对象目录读取中断"))?
}

#[tauri::command]
pub async fn portable_asset_detail(
    kind: String,
    name: String,
    source: Option<String>,
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, crate::state::SharedHomeDir>,
) -> ModelResult<crate::modules::sync::portable_assets::Detail> {
    let root = data_root.0.clone();
    let home = home.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        crate::modules::sync::portable_assets::detail(&root, &home, &kind, &name, source.as_deref())
    })
    .await
    .map_err(|_| invalid("asset_detail", "读取资源详情任务中断"))?
}
