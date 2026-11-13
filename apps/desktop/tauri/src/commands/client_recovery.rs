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
    can_rearm: bool,
    can_prepare_plan: bool,
    plan_digest: Option<String>,
    can_finish_unchanged: bool,
    can_cancel_preparation: bool,
    preparation_digest: Option<String>,
    unchanged_digest: Option<String>,
    rollback_digest: Option<String>,
    rearm_digest: Option<String>,
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
            original_evidence:"unknown",plan_evidence:"unknown",completion_evidence:"unknown",phase_count:0,can_continue:false,can_rollback:false,can_finalize:false,can_rearm:false,can_prepare_plan:false,plan_digest:None,can_finish_unchanged:false,can_cancel_preparation:false,preparation_digest:None,unchanged_digest:None,rollback_digest:None,rearm_digest:None,confirmation_digest:None})}
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
#[tauri::command]
pub async fn client_recovery_rollback(
    request: Request,
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
    install: tauri::State<'_, SharedRuntimeInstall>,
    app: tauri::AppHandle,
) -> AppResult<Outcome> {
    use tauri::Manager;
    let fail = || AppError::RuntimeManaged {
        code: "client_recovery_rollback_refused".into(),
        detail: "当前有其它运行或更新操作，撤销恢复未执行".into(),
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
    super::run_blocking("rollback client recovery", move || {
        let _lease = lease;
        crate::modules::runtime::protection::ensure_binding_ready(&data)?;
        #[cfg(target_os = "macos")]
        {
            crate::infrastructure::macos_client_originals::session_index::rollback::rollback(
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
                detail: "当前平台的客户端撤销恢复尚未接入".into(),
            })
        }
    })
    .await
}
#[tauri::command]
pub async fn client_recovery_rearm(
    request: Request,
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
    install: tauri::State<'_, SharedRuntimeInstall>,
    app: tauri::AppHandle,
) -> AppResult<Outcome> {
    use tauri::Manager;
    let fail = || AppError::RuntimeManaged {
        code: "client_recovery_rearm_refused".into(),
        detail: "当前有其它运行或更新操作，重新准备恢复未执行".into(),
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
    super::run_blocking("rearm client recovery", move || {
        let _lease = lease;
        crate::modules::runtime::protection::ensure_binding_ready(&data)?;
        #[cfg(target_os = "macos")]
        {
            crate::infrastructure::macos_client_originals::session_index::rearm::prepare(
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
                detail: "当前平台的客户端重新准备恢复尚未接入".into(),
            })
        }
    })
    .await
}
#[tauri::command]
pub async fn client_recovery_finish_unchanged(
    request: Request,
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
    install: tauri::State<'_, SharedRuntimeInstall>,
    app: tauri::AppHandle,
) -> AppResult<Outcome> {
    use tauri::Manager;
    let fail = || AppError::RuntimeManaged {
        code: "client_recovery_finish_unchanged_refused".into(),
        detail: "当前有其它运行或更新操作，结束未完成会话未执行".into(),
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
    super::run_blocking("finish unchanged client session", move || {
        let _lease = lease;
        crate::modules::runtime::protection::ensure_binding_ready(&data)?;
        #[cfg(target_os = "macos")]
        {
            crate::infrastructure::macos_client_originals::session_index::unchanged::finish(
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
                detail: "当前平台的客户端结束未完成会话尚未接入".into(),
            })
        }
    })
    .await
}
#[tauri::command]
pub async fn client_recovery_cancel_preparation(
    request: Request,
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
    install: tauri::State<'_, SharedRuntimeInstall>,
    app: tauri::AppHandle,
) -> AppResult<Outcome> {
    use tauri::Manager;
    let fail = || AppError::RuntimeManaged {
        code: "client_recovery_cancel_preparation_refused".into(),
        detail: "当前有其它运行或更新操作，取消启动准备未执行".into(),
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
    super::run_blocking("cancel unstarted preparation", move || {
        let _lease = lease;
        crate::modules::runtime::protection::ensure_binding_ready(&data)?;
        #[cfg(target_os = "macos")]
        {
            crate::infrastructure::macos_client_originals::session_index::preparation::cancel(
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
                detail: "当前平台的客户端取消启动准备尚未接入".into(),
            })
        }
    })
    .await
}
#[tauri::command]
pub async fn client_recovery_prepare_plan(
    request: Request,
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
    install: tauri::State<'_, SharedRuntimeInstall>,
    app: tauri::AppHandle,
) -> AppResult<Outcome> {
    use tauri::Manager;
    let fail = || AppError::RuntimeManaged {
        code: "client_recovery_prepare_plan_refused".into(),
        detail: "当前有其它运行或更新操作，准备恢复计划未执行".into(),
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
    super::run_blocking("prepare missing client restore plan", move || {
        let _lease = lease;
        crate::modules::runtime::protection::ensure_binding_ready(&data)?;
        #[cfg(target_os = "macos")]
        {
            crate::infrastructure::macos_client_originals::session_index::missing_plan::prepare(
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
                detail: "当前平台的客户端准备恢复计划尚未接入".into(),
            })
        }
    })
    .await
}
#[cfg(all(test, target_os = "macos", target_arch = "aarch64"))]
#[path = "client_recovery_tests.rs"]
mod tests;
