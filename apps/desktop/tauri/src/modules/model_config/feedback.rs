//! Model operations publish registered facts; payloads contain no prompt/config.
use crate::modules::notifications::registry::{
    self, Action, Channel, EventIdentity, Job, ObjectKind, OpaqueId, Phase, Trigger,
};
use serde::Serialize;
use std::{path::Path, sync::OnceLock};
static APP: OnceLock<tauri::AppHandle> = OnceLock::new();
pub fn configure(app: tauri::AppHandle) {
    let _ = APP.set(app);
}
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    Generation,
    Research,
    ExternalSubmit,
    PolicySave,
    Transfer,
}
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationPhase {
    Running,
    Review,
    Succeeded,
    Failed,
    Stopped,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Signal<'a> {
    id: &'a str,
    kind: OperationKind,
    phase: OperationPhase,
}
pub fn progress(root: &Path, id: &str, kind: OperationKind, phase: OperationPhase) {
    let Some(app) = APP.get() else {
        return;
    };
    if OpaqueId::parse(id).is_err() {
        return;
    }
    if crate::commands::event_delivery::emit_signal(
        app,
        "model-operation-progress",
        Job::ModelConfig,
        Trigger::Commit,
        Channel::Local,
        Signal { id, kind, phase },
    )
    .is_err()
    {
        diagnostic(root);
    }
}
pub fn external_terminal(root: &Path, task: &str, submission: &str, success: bool) {
    let Some(app) = APP.get() else {
        return;
    };
    let Ok(object_id) = OpaqueId::parse(task) else {
        return;
    };
    let Ok(candidate) = OpaqueId::parse(submission) else {
        return;
    };
    let identity = EventIdentity {
        object: ObjectKind::Configuration,
        object_id,
        action: Action::Save,
        phase: Phase::Execution,
        channel: Channel::Local,
        candidate,
    };
    crate::commands::event_delivery::publish(
        app,
        root,
        if success {
            "model-template-external-succeeded"
        } else {
            "model-template-external-failed"
        },
        Some(identity),
        Trigger::Commit,
    );
}
fn diagnostic(root: &Path) {
    let _ = crate::infrastructure::runtime_log::RuntimeLog::new(root)
        .append_event("registered model operation delivery failed");
}
pub fn registered() -> bool {
    registry::validate_signal(
        "model-operation-progress",
        Job::ModelConfig,
        Trigger::Commit,
        Channel::Local,
    )
    .is_ok()
}
pub fn transfer<T>(
    root: &Path,
    id: &str,
    review: bool,
    action: impl FnOnce() -> super::schema::ModelResult<T>,
) -> super::schema::ModelResult<T> {
    progress(root, id, OperationKind::Transfer, OperationPhase::Running);
    let result = action();
    progress(
        root,
        id,
        OperationKind::Transfer,
        if result.is_err() {
            OperationPhase::Failed
        } else if review {
            OperationPhase::Review
        } else {
            OperationPhase::Succeeded
        },
    );
    result
}
