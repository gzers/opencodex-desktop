//! Metadata-only preparation of a NEW logical restore decision after rollback.
//! Native objects and immutable history remain the SAME; no stage is copied or
//! discovered. Actual restoration is a separate explicit continuation action.
use super::finalization::{Outcome, Request};
use super::*;
use crate::{
    errors::{AppError, AppResult},
    infrastructure::official_client_coordination::{native::with_native_coordination, Context},
};
use std::time::Instant;
fn fail<T>(_: T) -> AppError {
    AppError::RuntimeManaged {
        code: "client_recovery_rearm_refused".into(),
        detail: "原件恢复准备未确认完成；原件与当前文件保留".into(),
    }
}
pub fn review(context: &ProcessContext, data: &Path) -> AppResult<Option<Request>> {
    observation::inspect_map(context, data, |head| {
        let Some(head) = head.filter(|h| h.state == State::RestoreRolledBack) else {
            return Ok(None);
        };
        let refs = head
            .references
            .as_ref()
            .ok_or_else(|| denied("rearm references absent"))?;
        let client = std::fs::canonicalize(context.working_directory.join(".codex"))?;
        let session = session::StartupSession::open(context, data, &client, refs)
            .map_err(|_| denied("rearm session unconfirmed"))?;
        Ok(session
            .rearm_fingerprint(head, false)
            .map_err(|_| denied("rearm review unknown"))?
            .map(|confirmation_digest| Request {
                attempt_id: refs.attempt.clone(),
                revision: head.revision,
                confirmation_digest,
            }))
    })
    .map_err(fail)
}
pub fn prepare(context: &ProcessContext, data: &Path, request: Request) -> AppResult<Outcome> {
    let started = Instant::now();
    let deadline = started + crate::modules::process::stop_timeout();
    let result: AppResult<Outcome> = (|| {
        let client =
            std::fs::canonicalize(context.working_directory.join(".codex")).map_err(fail)?;
        let index = Index::open_existing(context, data, &client).map_err(fail)?;
        let head = index.head().map_err(fail)?.ok_or_else(|| fail(()))?;
        let refs = head.references.as_ref().ok_or_else(|| fail(()))?;
        if head.state != State::RestoreRolledBack
            || head.revision != request.revision
            || refs.attempt != request.attempt_id
        {
            return Err(fail(()));
        }
        let session = session::StartupSession::open(context, data, &client, refs)?;
        if session.rearm_fingerprint(&head, false)?.as_deref() != Some(&request.confirmation_digest)
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
        let (id, sha256) = with_native_coordination(&coordination, deadline, |lease| {
            session.rearm_rolled_back(
                context,
                &head,
                &request.confirmation_digest,
                lease,
                deadline,
            )
        })?;
        if Instant::now() >= deadline {
            return Err(fail(()));
        }
        let next = index.rearm_rolled_back(&head, &id, &sha256).map_err(fail)?;
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
            "client-recovery-rearm: refused elapsed_ms={}",
            started.elapsed().as_millis()
        ));
    }
    result.map_err(fail)
}
