//! Explicit plan preparation from independently registered Stopped evidence.
//! This is a NEW current native authorization, never a reconstructed exit proof.
use super::finalization::{Outcome, Request};
use super::*;
use crate::{
    errors::{AppError, AppResult},
    infrastructure::official_client_coordination::{native::with_native_coordination, Context},
};
use std::time::Instant;
fn fail<T>(_: T) -> AppError {
    AppError::RuntimeManaged {
        code: "client_recovery_plan_refused".into(),
        detail: "恢复计划尚未确认完成；原件与当前文件保留，请重新检查".into(),
    }
}
pub(super) fn confirmation(
    context: &ProcessContext,
    data: &Path,
    head: &Head,
) -> io::Result<Option<String>> {
    if !matches!(
        head.state,
        State::Stopped | State::StopPrepared | State::Ready
    ) || head.restore_plan.is_some()
    {
        return Ok(None);
    }
    let client = std::fs::canonicalize(context.working_directory.join(".codex"))?;
    let refs = head
        .references
        .as_ref()
        .ok_or_else(|| denied("missing-plan original references absent"))?;
    let session = session::StartupSession::open(context, data, &client, refs)
        .map_err(|_| denied("missing-plan session unconfirmed"))?;
    session
        .missing_plan_fingerprint(context, head, false)
        .map_err(|_| denied("missing-plan content admission refused"))
}
pub fn review(context: &ProcessContext, data: &Path) -> AppResult<Option<Request>> {
    observation::inspect_map(context, data, |head| {
        let Some(head) = head.filter(|h| {
            matches!(h.state, State::Stopped | State::StopPrepared | State::Ready)
                && h.restore_plan.is_none()
        }) else {
            return Ok(None);
        };
        let refs = head
            .references
            .as_ref()
            .ok_or_else(|| denied("missing-plan refs absent"))?;
        Ok(
            confirmation(context, data, head)?.map(|confirmation_digest| Request {
                attempt_id: refs.attempt.clone(),
                revision: head.revision,
                confirmation_digest,
            }),
        )
    })
    .map_err(fail)
}
pub fn prepare(context: &ProcessContext, data: &Path, request: Request) -> AppResult<Outcome> {
    let started = Instant::now();
    let deadline = started + crate::modules::process::stop_timeout();
    let phase = std::cell::Cell::new("confirm-stopped-evidence");
    let result: AppResult<Outcome> = (|| {
        let client =
            std::fs::canonicalize(context.working_directory.join(".codex")).map_err(fail)?;
        let index = Index::open_existing(context, data, &client).map_err(fail)?;
        let head = index.head().map_err(fail)?.ok_or_else(|| fail(()))?;
        let refs = head.references.as_ref().ok_or_else(|| fail(()))?;
        if !matches!(
            head.state,
            State::Stopped | State::StopPrepared | State::Ready
        ) || head.restore_plan.is_some()
            || head.revision != request.revision
            || refs.attempt != request.attempt_id
        {
            return Err(fail(()));
        }
        if confirmation(context, data, &head).map_err(fail)?.as_deref()
            != Some(&request.confirmation_digest)
        {
            return Err(fail(()));
        }
        let session = session::StartupSession::open(context, data, &client, refs)?;
        let coordination = Context {
            runtime: context.runtime.clone(),
            user_profile: context.working_directory.clone(),
            opencodex_home: context.opencodex_home.clone(),
            codex_home: client,
            control_root: index.control_root().map_err(fail)?,
        };
        phase.set("admit-and-stage-originals");
        let (id, sha256) = with_native_coordination(&coordination, deadline, |lease| {
            session.prepare_missing_plan(
                context,
                &head,
                &request.confirmation_digest,
                lease,
                deadline,
            )
        })?;
        phase.set("register-plan");
        if Instant::now() >= deadline {
            return Err(fail(()));
        }
        let next = index
            .record_restore_plan(&head, &id, &sha256)
            .map_err(fail)?;
        // A plan registration is not client restoration/Start permission. If a
        // durable write runs over budget, retain its exact plan for inspection.
        if Instant::now() >= deadline {
            return Err(fail(()));
        }
        Ok(Outcome {
            attempt_id: request.attempt_id,
            revision: next.revision,
            status: if next.state == State::RecoveryPrepared {
                "recovery_prepared"
            } else {
                "restore_prepared"
            },
        })
    })();
    if result.is_err() {
        let _ = crate::infrastructure::runtime_log::RuntimeLog::new(data).append_event(&format!(
            "client-missing-plan: refused phase={} elapsed_ms={}",
            phase.get(),
            started.elapsed().as_millis()
        ));
    }
    result.map_err(fail)
}
