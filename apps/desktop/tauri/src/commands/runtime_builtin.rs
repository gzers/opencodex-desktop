//! Explicit, reviewed restoration of the app's own pinned offline resource.
use crate::{
    errors::{AppError, AppResult},
    modules::runtime::{bundled, program_reinstall, program_repair, RuntimeHandle},
    state::{SharedDataRoot, SharedProcessContext, SharedRuntimeInstall},
};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tauri::Manager;
fn fail(text: &'static str) -> AppError {
    AppError::RuntimeManaged {
        code: "runtime_builtin_reinstall_refused".into(),
        detail: text.into(),
    }
}
struct Resource {
    descriptor: Vec<u8>,
    archive: PathBuf,
    version: String,
    source: String,
    pin: &'static str,
}
fn resource(root: &Path) -> AppResult<Resource> {
    let pin = option_env!("OCXD_BUNDLED_RUNTIME_DESCRIPTOR_SHA256")
        .ok_or_else(|| fail("当前应用没有固定内置资源"))?;
    let descriptor_path = root.join("runtime-bundle.manifest.json");
    crate::modules::backup::safety::check_path(root, &descriptor_path, false)?;
    let descriptor = crate::modules::backup::safety::read_file(&descriptor_path, 64 * 1024 * 1024)?;
    if crate::infrastructure::hash::sha256_hex(&descriptor) != pin {
        return Err(fail("内置资源清单与当前应用固定摘要不一致"));
    }
    let v: serde_json::Value =
        crate::modules::model_config::schema_v2::parse_strict_json_v2(&descriptor)
            .map_err(|_| fail("内置资源清单无效"))?;
    let name = v["archiveName"]
        .as_str()
        .filter(|s| !s.is_empty() && *s != "." && *s != ".." && !s.contains(['/', '\\', ':']))
        .ok_or_else(|| fail("内置资源文件名无效"))?;
    let version = v["runtimeVersion"]
        .as_str()
        .ok_or_else(|| fail("内置资源版本缺失"))?
        .to_string();
    let source = v["runtimeArchiveSha256"]
        .as_str()
        .ok_or_else(|| fail("内置官方来源摘要缺失"))?
        .to_string();
    Ok(Resource {
        source,
        archive: root.join(name),
        descriptor,
        version,
        pin,
    })
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub confirmation_digest: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
    version: String,
    configuration_preserved: bool,
}
#[tauri::command]
pub async fn runtime_builtin_preview(
    app: tauri::AppHandle,
    runtime: tauri::State<'_, Arc<RuntimeHandle>>,
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
) -> AppResult<program_reinstall::Review> {
    if !runtime.uses_versioned_programs() {
        return Err(fail("当前程序来源不支持内置重装"));
    }
    let i = runtime.installation_root().to_path_buf();
    let d = data.0.clone();
    let h = context.opencodex_home.clone();
    let resources = app.path().resource_dir()?.join("runtime-bundle");
    super::run_readonly("preview bundled runtime reinstall", move || {
        let r = resource(&resources)?;
        program_reinstall::review(&i, &d, &h, r.pin, &r.version)
    })
    .await
}
#[tauri::command]
pub async fn runtime_builtin_reinstall(
    request: Request,
    app: tauri::AppHandle,
    runtime: tauri::State<'_, Arc<RuntimeHandle>>,
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
    install: tauri::State<'_, SharedRuntimeInstall>,
) -> AppResult<Outcome> {
    restore(request, app, runtime, data, context, install, false).await
}
#[tauri::command]
pub async fn runtime_builtin_repair_preview(
    app: tauri::AppHandle,
    runtime: tauri::State<'_, Arc<RuntimeHandle>>,
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
) -> AppResult<program_repair::Review> {
    if !runtime.uses_versioned_programs() {
        return Err(fail("当前程序来源不支持同版本修复"));
    }
    let i = runtime.installation_root().to_path_buf();
    let d = data.0.clone();
    let h = context.opencodex_home.clone();
    let resources = app.path().resource_dir()?.join("runtime-bundle");
    super::run_readonly("preview same-version runtime repair", move || {
        let r = resource(&resources)?;
        program_repair::review(&i, &d, &h, r.pin, &r.version, &r.source)
    })
    .await
}
#[tauri::command]
pub async fn runtime_builtin_repair(
    request: Request,
    app: tauri::AppHandle,
    runtime: tauri::State<'_, Arc<RuntimeHandle>>,
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
    install: tauri::State<'_, SharedRuntimeInstall>,
) -> AppResult<Outcome> {
    restore(request, app, runtime, data, context, install, true).await
}
async fn restore(
    request: Request,
    app: tauri::AppHandle,
    runtime: tauri::State<'_, Arc<RuntimeHandle>>,
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
    install: tauri::State<'_, SharedRuntimeInstall>,
    repair: bool,
) -> AppResult<Outcome> {
    if !cfg!(target_os = "macos") || !runtime.uses_versioned_programs() {
        return Err(fail("当前平台完整程序占用证明尚未接入，未执行重装"));
    }
    let lease = {
        let manager = app.state::<super::update::SharedUpdateStatus>();
        let status = manager.lock().map_err(|_| fail("无法核对管理器更新状态"))?;
        if status.installing || status.pending_restart.is_some() {
            return Err(fail("请先完成管理器更新"));
        }
        install
            .acquire()
            .ok_or_else(|| fail("已有安装、卸载或恢复正在进行"))?
    };
    let runtime = runtime.inner().clone();
    let d = data.0.clone();
    let context = context.inner().clone();
    let resources = app.path().resource_dir()?.join("runtime-bundle");
    super::run_blocking("reinstall reviewed bundled runtime", move || {
        let _lease = lease;
        crate::modules::runtime::protection::ensure_binding_ready(&d)?;
        let r = resource(&resources)?;
        let owner = super::runtime_recovery_v2_owner::SystemServiceOwnerProofProvider::new(
            runtime.installation_root().into(),
            context.clone(),
        )?;
        let supply = |repair: bool| {
            let request = bundled::versioned::ReinstallRequest {
                home: &context.opencodex_home,
                confirmation_digest: &request.confirmation_digest,
            };
            let unused = || owner.require_program_unused(&context);
            if repair {
                bundled::versioned::repair(
                    runtime.installation_root(),
                    &d,
                    request,
                    &r.archive,
                    &r.descriptor,
                    r.pin,
                    unused,
                )
            } else {
                bundled::versioned::reinstall(
                    runtime.installation_root(),
                    &d,
                    request,
                    &r.archive,
                    &r.descriptor,
                    r.pin,
                    unused,
                )
            }
        };
        let result = supply(repair);
        runtime.observe_current();
        match result? {
            bundled::ProvisionResult::Installed { version } => {
                super::runtime::broadcast_source_changed(&app);
                Ok(Outcome {
                    version,
                    configuration_preserved: true,
                })
            }
            _ => Err(fail("内置重装没有返回实际安装完成回执")),
        }
    })
    .await
}

#[tauri::command]
pub fn runtime_preparation_status(
    runtime: tauri::State<'_, Arc<RuntimeHandle>>,
) -> AppResult<crate::modules::runtime::preparation::Snapshot> {
    runtime.preparation.snapshot()
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreparationRetry {
    attempt_id: String,
}
#[tauri::command]
pub async fn runtime_preparation_retry(
    request: PreparationRetry,
    app: tauri::AppHandle,
    runtime: tauri::State<'_, Arc<RuntimeHandle>>,
    data: tauri::State<'_, SharedDataRoot>,
    install: tauri::State<'_, SharedRuntimeInstall>,
) -> AppResult<crate::modules::runtime::preparation::Snapshot> {
    let runtime = runtime.inner().clone();
    let install = install.inner().clone();
    let d = data.0.clone();
    let root = app.path().resource_dir()?.join("runtime-bundle");
    super::run_blocking("retry first runtime preparation", move || {
        {
            let update = app.state::<super::update::SharedUpdateStatus>();
            let state = update.lock().map_err(|_| fail("无法核对更新状态"))?;
            if state.installing || state.pending_restart.is_some() {
                return Err(fail("请先完成管理器更新"));
            }
        }
        let pin = option_env!("OCXD_BUNDLED_RUNTIME_DESCRIPTOR_SHA256")
            .ok_or_else(|| fail("当前应用没有固定运行资源"))?;
        let source_app = app.clone();
        crate::modules::runtime::startup_bundle::retry(
            d,
            root,
            pin,
            runtime.clone(),
            install,
            request.attempt_id,
            move || super::runtime::broadcast_source_changed(&source_app),
        )?;
        runtime.preparation.snapshot()
    })
    .await
}

#[cfg(test)]
mod preparation_input_tests {
    use super::*;
    #[test]
    fn retry_accepts_only_the_observed_attempt_not_paths_or_runtime_versions() {
        assert!(serde_json::from_value::<PreparationRetry>(
            serde_json::json!({"attemptId":"observed","programRoot":"arbitrary"})
        )
        .is_err());
        assert!(serde_json::from_value::<PreparationRetry>(
            serde_json::json!({"attemptId":"observed","version":"other"})
        )
        .is_err());
        assert!(serde_json::from_value::<PreparationRetry>(
            serde_json::json!({"attemptId":"observed"})
        )
        .is_ok());
    }
}
