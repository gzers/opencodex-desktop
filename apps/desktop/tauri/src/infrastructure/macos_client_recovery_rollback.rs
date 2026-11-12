//! Native rollback executor. It does not mean original client restoration;
//! only exact Stopped artifact postimages have been restored. The index remains
//! recovery-required and refuses a new startup. UI/replan follows separately.
use super::finalization::{Outcome, Request};
use super::*;
use crate::{
    errors::{AppError, AppResult},
    infrastructure::official_client_coordination::{native::with_native_coordination, Context},
};
use std::time::Instant;
fn fail<T>(_: T) -> AppError {
    AppError::RuntimeManaged {
        code: "client_recovery_rollback_refused".into(),
        detail: "撤销恢复未确认完成；原件与当前文件保留".into(),
    }
}
/// Read-only review for the native executor. No product command is registered
/// until rollback/replan and confirmation UI have been joined and validated.
pub fn review(context: &ProcessContext, data: &Path) -> AppResult<Option<Request>> {
    observation::inspect_map(context, data, |head| {
        let Some(head) = head.filter(|h| h.state == State::RestorePrepared) else {
            return Ok(None);
        };
        let refs = head
            .references
            .as_ref()
            .ok_or_else(|| denied("rollback references absent"))?;
        let client = std::fs::canonicalize(context.working_directory.join(".codex"))?;
        let session = session::StartupSession::open(context, data, &client, refs)
            .map_err(|_| denied("rollback session unconfirmed"))?;
        Ok(session
            .recovery_fingerprint(head, restore::Decision::Rollback)
            .map_err(|_| denied("rollback review unknown"))?
            .map(|confirmation_digest| Request {
                attempt_id: refs.attempt.clone(),
                revision: head.revision,
                confirmation_digest,
            }))
    })
    .map_err(fail)
}
pub fn rollback(context: &ProcessContext, data: &Path, request: Request) -> AppResult<Outcome> {
    let started = Instant::now();
    let deadline = started + crate::modules::process::stop_timeout();
    let phase = std::cell::Cell::new("open-existing-index");
    let result: AppResult<Outcome> = (|| {
        if request.revision == 0
            || !valid_hash(&request.confirmation_digest)
            || uuid::Uuid::parse_str(&request.attempt_id)
                .map(|v| v.to_string() != request.attempt_id)
                .unwrap_or(true)
        {
            return Err(fail(()));
        }
        let client =
            std::fs::canonicalize(context.working_directory.join(".codex")).map_err(fail)?;
        let index = Index::open_existing(context, data, &client).map_err(fail)?;
        let head = index.head().map_err(fail)?.ok_or_else(|| fail(()))?;
        let refs = head.references.as_ref().ok_or_else(|| fail(()))?;
        if head.state != State::RestorePrepared
            || head.revision != request.revision
            || refs.attempt != request.attempt_id
        {
            return Err(fail(()));
        }
        let session = session::StartupSession::open(context, data, &client, refs)?;
        if session
            .recovery_fingerprint(&head, restore::Decision::Rollback)?
            .as_deref()
            != Some(&request.confirmation_digest)
        {
            return Err(fail(()));
        }
        let coordination = Context {
            runtime: context.runtime.clone(),
            user_profile: context.working_directory.clone(),
            opencodex_home: context.opencodex_home.clone(),
            codex_home: client,
            control_root: index.control_root().map_err(fail)?,
        };
        phase.set("admit-and-rollback-original-plan");
        with_native_coordination(&coordination, deadline, |lease| {
            session.execute_interrupted_plan(
                context,
                &head,
                &request.confirmation_digest,
                lease,
                deadline,
                restore::Decision::Rollback,
            )
        })?;
        phase.set("full-tree-and-lock-readback");
        let proof =
            session.explicit_readback(context, &head, deadline, restore::Decision::Rollback)?;
        phase.set("close-rollback-index");
        let next = index.close_rolled_back(&head, &proof).map_err(fail)?;
        Ok(Outcome {
            attempt_id: request.attempt_id,
            revision: next.revision,
            status: "rolled_back",
        })
    })();
    if result.is_err() {
        let _ = crate::infrastructure::runtime_log::RuntimeLog::new(data).append_event(&format!(
            "client-recovery-rollback: refused phase={} elapsed_ms={}",
            phase.get(),
            started.elapsed().as_millis()
        ));
    }
    result.map_err(fail)
}
