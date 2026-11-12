//! Read-only diagnostics. Paths/context come only from current app binding,
//! never IPC parameters. Execution of interrupted recovery remains separate.
#[cfg(target_os = "macos")]
pub use crate::infrastructure::macos_client_originals::session_index::finalization::{
    Outcome, Request,
};
#[cfg(target_os = "macos")]
pub use crate::infrastructure::macos_client_originals::session_index::preview::Preview;
use crate::{
    errors::{AppError, AppResult},
    state::{SharedDataRoot, SharedProcessContext, SharedRuntimeInstall},
};
#[cfg(not(target_os = "macos"))]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    attempt_id: String,
    revision: u64,
    confirmation_digest: String,
}
#[cfg(not(target_os = "macos"))]
#[derive(serde::Serialize)]
pub struct Outcome {
    status: &'static str,
}
#[cfg(not(target_os = "macos"))]
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    status: &'static str,
    phase: Option<String>,
    revision: Option<u64>,
    attempt_id: Option<String>,
    original_evidence: &'static str,
    plan_evidence: &'static str,
    completion_evidence: &'static str,
    phase_count: usize,
    can_continue: bool,
    can_rollback: bool,
    can_finalize: bool,
    confirmation_digest: Option<String>,
}
#[tauri::command]
pub async fn client_recovery_preview(
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
) -> AppResult<Preview> {
    let data = data.0.clone();
    let context = context.inner().clone();
    super::run_readonly("inspect current client recovery",move||{
        #[cfg(target_os="macos")]
        {Ok(crate::infrastructure::macos_client_originals::session_index::preview::inspect_preview(&context,&data))}
        #[cfg(not(target_os="macos"))]
        {let _=(data,context);Ok(Preview{status:"unsupported",phase:None,revision:None,attempt_id:None,
            original_evidence:"unknown",plan_evidence:"unknown",completion_evidence:"unknown",phase_count:0,can_continue:false,can_rollback:false,can_finalize:false,confirmation_digest:None})}
    }).await
}
#[tauri::command]
pub async fn client_recovery_finalize(
    request: Request,
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
    install: tauri::State<'_, SharedRuntimeInstall>,
    app: tauri::AppHandle,
) -> AppResult<Outcome> {
    use tauri::Manager;
    let fail = || AppError::RuntimeManaged {
        code: "client_recovery_finalize_refused".into(),
        detail: "当前有其它运行或更新操作，恢复收尾未执行".into(),
    };
    let lease = {
        let updates = app.state::<crate::commands::update::SharedUpdateStatus>();
        let state = updates.lock().map_err(|_| fail())?;
        if state.installing || state.pending_restart.is_some() {
            return Err(fail());
        }
        install.acquire().ok_or_else(fail)?
    };
    let data = data.0.clone();
    let context = context.inner().clone();
    super::run_blocking("finalize completed client recovery", move || {
        let _lease = lease;
        crate::modules::runtime::protection::ensure_binding_ready(&data)?;
        #[cfg(target_os = "macos")]
        {
            crate::infrastructure::macos_client_originals::session_index::finalization::finalize(
                &context, &data, request,
            )
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (
                context,
                request.attempt_id,
                request.revision,
                request.confirmation_digest,
            );
            Err(AppError::RuntimeManaged {
                code: "client_recovery_unsupported".into(),
                detail: "当前平台的客户端恢复收尾尚未接入".into(),
            })
        }
    })
    .await
}
#[tauri::command]
pub async fn client_recovery_continue(
    request: Request,
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
    install: tauri::State<'_, SharedRuntimeInstall>,
    app: tauri::AppHandle,
) -> AppResult<Outcome> {
    use tauri::Manager;
    let fail = || AppError::RuntimeManaged {
        code: "client_recovery_continue_refused".into(),
        detail: "当前有其它运行或更新操作，恢复继续未执行".into(),
    };
    let lease = {
        let updates = app.state::<crate::commands::update::SharedUpdateStatus>();
        let state = updates.lock().map_err(|_| fail())?;
        if state.installing || state.pending_restart.is_some() {
            return Err(fail());
        }
        install.acquire().ok_or_else(fail)?
    };
    let data = data.0.clone();
    let context = context.inner().clone();
    super::run_blocking("continue interrupted client recovery", move || {
        let _lease = lease;
        crate::modules::runtime::protection::ensure_binding_ready(&data)?;
        #[cfg(target_os = "macos")]
        {
            crate::infrastructure::macos_client_originals::session_index::continuation::resume(
                &context, &data, request,
            )
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (
                context,
                request.attempt_id,
                request.revision,
                request.confirmation_digest,
            );
            Err(AppError::RuntimeManaged {
                code: "client_recovery_unsupported".into(),
                detail: "当前平台的客户端恢复继续尚未接入".into(),
            })
        }
    })
    .await
}
#[cfg(all(test, target_os = "macos", target_arch = "aarch64"))]
#[path = "client_recovery_tests.rs"]
mod tests;
