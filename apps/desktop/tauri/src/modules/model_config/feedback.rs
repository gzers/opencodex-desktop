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
    NodeMeasurement,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_id: Option<&'a str>,
}
pub fn progress(root: &Path, id: &str, kind: OperationKind, phase: OperationPhase) {
    progress_linked(root, id, kind, phase, None);
}
fn progress_linked(
    root: &Path,
    id: &str,
    kind: OperationKind,
    phase: OperationPhase,
    previous_id: Option<&str>,
) {
    let Some(app) = APP.get() else {
        return;
    };
    if OpaqueId::parse(id).is_err()
        || previous_id.is_some_and(|previous| OpaqueId::parse(previous).is_err())
    {
        return;
    }
    if crate::commands::event_delivery::emit_signal(
        app,
        "model-operation-progress",
        Job::ModelConfig,
        Trigger::Commit,
        Channel::Local,
        Signal {
            id,
            kind,
            phase,
            previous_id,
        },
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
pub fn policy_terminal(root: &Path, id: &str, success: bool) {
    let Some(app) = APP.get() else {
        return;
    };
    let Ok(object_id) = OpaqueId::parse(id) else {
        return;
    };
    let identity = EventIdentity {
        object: ObjectKind::Configuration,
        object_id: object_id.clone(),
        action: Action::Save,
        phase: Phase::Execution,
        channel: Channel::Local,
        candidate: object_id,
    };
    crate::commands::event_delivery::publish(
        app,
        root,
        if success {
            "model-policy-save-succeeded"
        } else {
            "model-policy-save-failed"
        },
        Some(identity),
        Trigger::Commit,
    );
}

pub use crate::modules::notifications::registry::BlockedReason as SaveBlockedReason;
/// 每次确认都产生一次明确事实；同一原因按固定 dedupe 更新同一条通知。
pub fn save_blocked(app: &tauri::AppHandle, root: &Path, reason: SaveBlockedReason) {
    let _ = crate::infrastructure::runtime_log::RuntimeLog::new(root)
        .append_event(&format!("model policy save blocked: {}", reason.text()));
    let identity = crate::commands::event_delivery::prepare(
        root,
        "model-policy-save-blocked",
        Channel::Local,
        &[reason as u8],
    );
    crate::commands::event_delivery::publish_detail(
        app,
        root,
        "model-policy-save-blocked",
        identity,
        Trigger::User,
        Some(crate::modules::notifications::registry::Detail::ModelPolicyBlocked(reason)),
    );
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

/// Receiving creates its durable identity only after the file/remote document
/// validates. Rebind the transient observation to that exact receipt, so later
/// adoption/retries update it instead of leaving a permanent review card.
pub fn receive_transfer(
    root: &Path,
    action: impl FnOnce() -> super::schema::ModelResult<super::transfer::IncomingPlan>,
) -> super::schema::ModelResult<super::transfer::IncomingPlan> {
    let pending = uuid::Uuid::new_v4().to_string();
    progress(
        root,
        &pending,
        OperationKind::Transfer,
        OperationPhase::Running,
    );
    let result = action();
    match &result {
        Ok(receipt) => progress_linked(
            root,
            &receipt.id,
            OperationKind::Transfer,
            OperationPhase::Review,
            Some(&pending),
        ),
        Err(_) => progress(
            root,
            &pending,
            OperationKind::Transfer,
            OperationPhase::Failed,
        ),
    }
    result
}
