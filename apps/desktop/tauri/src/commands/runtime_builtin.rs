//! Explicit, reviewed restoration of the app's own pinned offline resource.
use crate::{
    errors::{AppError, AppResult},
    modules::runtime::{bundled, program_reinstall, RuntimeHandle},
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
    Ok(Resource {
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
        let result = bundled::versioned::reinstall(
            runtime.installation_root(),
            &d,
            bundled::versioned::ReinstallRequest {
                home: &context.opencodex_home,
                confirmation_digest: &request.confirmation_digest,
            },
            &r.archive,
            &r.descriptor,
            r.pin,
            || owner.require_program_unused(&context),
        );
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
