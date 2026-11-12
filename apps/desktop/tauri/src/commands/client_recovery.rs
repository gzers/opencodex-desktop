//! Read-only diagnostics. Paths/context come only from current app binding,
//! never IPC parameters. Execution of interrupted recovery remains separate.
#[cfg(target_os = "macos")]
pub use crate::infrastructure::macos_client_originals::session_index::preview::Preview;
use crate::{
    errors::AppResult,
    state::{SharedDataRoot, SharedProcessContext},
};
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
            original_evidence:"unknown",plan_evidence:"unknown",completion_evidence:"unknown",phase_count:0,can_continue:false,can_rollback:false})}
    }).await
}
