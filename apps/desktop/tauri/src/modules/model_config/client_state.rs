//! A successful runtime readback is not proof that a Codex client loaded it.
//! The currently supported fallback records a separate explicit user confirmation.
use super::{
    runtime::{self, RuntimePlan},
    schema::*,
    store::sha256,
    upstream,
};
use crate::{
    infrastructure::atomic_write::atomic_write, modules::backup::safety, state::ProcessContext,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const STATE: &str = "manager-state/model-client-state.json";
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    NotRequired,
    RestartRequired,
    PendingVerification,
    UserConfirmed,
    RuntimeChanged,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct State {
    plan_id: String,
    target_digest: String,
    checked_at: String,
    check_id: String,
    runtime_matches: bool,
    confirmed_at: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub plan_id: Option<String>,
    pub status: Status,
    pub checked_at: Option<String>,
    pub check_id: Option<String>,
    pub confirmed_at: Option<String>,
    pub runtime_matches: bool,
    pub automatic_client_verification: bool,
}
fn path(root: &Path) -> ModelResult<PathBuf> {
    let path = root.join(STATE);
    safety::check_path(root, &path, true)?;
    Ok(path)
}
fn load(root: &Path) -> ModelResult<Option<State>> {
    let path = path(root)?;
    if !path.exists() {
        return Ok(None);
    }
    let state: State = serde_json::from_slice(&safety::read_file(&path, 16384)?)?;
    require_uuid(&state.plan_id)?;
    require_uuid(&state.check_id)?;
    if chrono::DateTime::parse_from_rfc3339(&state.checked_at).is_err()
        || state
            .confirmed_at
            .as_ref()
            .is_some_and(|s| chrono::DateTime::parse_from_rfc3339(s).is_err())
    {
        return Err(invalid("client_state", "客户端核对记录无效"));
    }
    Ok(Some(state))
}
fn persist(root: &Path, state: &State) -> ModelResult<()> {
    let path = path(root)?;
    let bytes = serde_json::to_vec(state)?;
    atomic_write(&path, &bytes, 0o600)?;
    if safety::read_file(&path, 16384)? != bytes {
        return Err(invalid("client_state", "客户端核对记录未读回"));
    }
    Ok(())
}
fn changed(plan: &RuntimePlan) -> ModelResult<bool> {
    Ok(plan.before.providers != plan.next_providers
        || plan.before.order["pickerOrder"] != serde_json::to_value(&plan.next_order)?
        || runtime::catalog_slugs(&plan.before)? != plan.expected_slugs
        || plan.disabled_before != plan.observed_disabled
        || runtime::credential_runtime_changed(plan))
}
fn target(root: &Path) -> ModelResult<Option<RuntimePlan>> {
    let mut latest: Option<RuntimePlan> = None;
    for id in runtime::plan_ids(root)? {
        let plan = runtime::load(root, &id)?;
        if plan.phase != "committed" || !changed(&plan)? {
            continue;
        }
        if latest
            .as_ref()
            .is_none_or(|old| (&plan.created_at, &plan.id) > (&old.created_at, &old.id))
        {
            latest = Some(plan);
        }
    }
    Ok(latest)
}
fn digest(plan: &RuntimePlan) -> ModelResult<String> {
    Ok(sha256(&serde_json::to_vec(&(
        &plan.next_providers,
        &plan.next_order,
        &plan.expected_slugs,
        &plan.observed_disabled,
        &plan.runtime_executable,
        &plan.runtime_binding,
    ))?))
}
fn project(plan: Option<&RuntimePlan>, state: Option<&State>) -> ModelResult<View> {
    let Some(plan) = plan else {
        return Ok(View {
            plan_id: None,
            status: Status::NotRequired,
            checked_at: None,
            check_id: None,
            confirmed_at: None,
            runtime_matches: false,
            automatic_client_verification: false,
        });
    };
    let state =
        state.filter(|s| s.plan_id == plan.id && digest(plan).is_ok_and(|d| d == s.target_digest));
    Ok(View {
        plan_id: Some(plan.id.clone()),
        status: match state {
            None => Status::RestartRequired,
            Some(s) if !s.runtime_matches => Status::RuntimeChanged,
            Some(s) if s.confirmed_at.is_some() => Status::UserConfirmed,
            _ => Status::PendingVerification,
        },
        checked_at: state.map(|s| s.checked_at.clone()),
        check_id: state
            .filter(|s| s.runtime_matches)
            .map(|s| s.check_id.clone()),
        confirmed_at: state.and_then(|s| s.confirmed_at.clone()),
        runtime_matches: state.is_some_and(|s| s.runtime_matches),
        automatic_client_verification: false,
    })
}
pub fn status(root: &Path) -> ModelResult<View> {
    project(target(root)?.as_ref(), load(root)?.as_ref())
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub plan_id: String,
    pub confirmation_check: Option<String>,
}
pub async fn check(root: &Path, context: &ProcessContext, request: Request) -> ModelResult<View> {
    let _guard = runtime::execution_lock(root)?;
    runtime::reject_incomplete(root)?;
    let plan =
        target(root)?.ok_or_else(|| invalid("client_target", "当前没有待客户端核对的运行保存"))?;
    if plan.id != request.plan_id {
        return Err(invalid(
            "client_target_changed",
            "待核对目标已更新，请查看最新保存",
        ));
    }
    let previous = load(root)?;
    if let Some(check_id) = &request.confirmation_check {
        require_uuid(check_id)?;
        if previous.as_ref().is_none_or(|s| {
            s.plan_id != plan.id
                || s.target_digest != digest(&plan).unwrap_or_default()
                || &s.check_id != check_id
                || !s.runtime_matches
        }) {
            return Err(invalid(
                "client_confirmation",
                "先重新检查当前保存目标，再明确确认客户端所见结果",
            ));
        }
    }
    let executable = context.executable()?;
    if executable.to_string_lossy() != plan.runtime_executable
        || plan.runtime_binding.as_ref() != Some(&runtime::runtime_binding(context)?)
    {
        return Err(invalid(
            "runtime_binding",
            "运行来源或数据根改变，不能沿用历史客户端确认",
        ));
    }
    let frozen = ProcessContext {
        runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(
            executable.clone(),
        ),
        working_directory: context.working_directory.clone(),
        opencodex_home: context.opencodex_home.clone(),
    };
    let current = upstream::snapshot(&frozen).await?;
    let disabled = upstream::read(&frozen, upstream::ReadOperation::DisabledModels).await?;
    let matches = current.providers == plan.next_providers
        && current.order["pickerOrder"] == serde_json::to_value(&plan.next_order)?
        && runtime::catalog_slugs(&current)? == plan.expected_slugs
        && disabled == serde_json::to_value(&plan.observed_disabled)?
        && runtime::verify_committed_credentials(root, &plan, &frozen).await?;
    if context.executable()? != executable {
        return Err(invalid("runtime_binding", "核对期间运行来源改变"));
    }
    let now = chrono::Utc::now().to_rfc3339();
    let confirmed_at = if matches {
        if request.confirmation_check.is_some() {
            Some(now.clone())
        } else {
            previous
                .filter(|s| {
                    s.plan_id == plan.id
                        && s.target_digest == digest(&plan).unwrap_or_default()
                        && s.runtime_matches
                })
                .and_then(|s| s.confirmed_at)
        }
    } else {
        None
    };
    let state = State {
        plan_id: plan.id.clone(),
        target_digest: digest(&plan)?,
        checked_at: now,
        check_id: uuid::Uuid::new_v4().to_string(),
        runtime_matches: matches,
        confirmed_at,
    };
    persist(root, &state)?;
    project(Some(&plan), Some(&state))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_or_unbound_confirmations_never_imply_client_verification() {
        let root = tempfile::tempdir().unwrap();
        let view = status(root.path()).unwrap();
        assert!(matches!(view.status, Status::NotRequired));
        assert!(!view.automatic_client_verification);
        let state = State {
            plan_id: uuid::Uuid::new_v4().to_string(),
            target_digest: "fixture".into(),
            checked_at: chrono::Utc::now().to_rfc3339(),
            check_id: uuid::Uuid::new_v4().to_string(),
            runtime_matches: true,
            confirmed_at: Some(chrono::Utc::now().to_rfc3339()),
        };
        persist(root.path(), &state).unwrap();
        let view = status(root.path()).unwrap();
        assert!(view.plan_id.is_none());
        assert!(view.confirmed_at.is_none());
    }
}
