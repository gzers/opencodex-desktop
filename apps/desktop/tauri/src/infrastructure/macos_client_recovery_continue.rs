//! Explicit continuation of the SAME independently indexed interrupted plan.
//! No historical exit authority is reconstructed from stored observations.
use super::finalization::{Outcome, Request};
use super::*;
use crate::{
    errors::{AppError, AppResult},
    infrastructure::official_client_coordination::{native::with_native_coordination, Context},
};
use std::time::Instant;
fn fail<T>(_: T) -> AppError {
    AppError::RuntimeManaged {
        code: "client_recovery_continue_refused".into(),
        detail: "恢复现场或确认已变化，未确认完成；原件与当前文件保留".into(),
    }
}
fn require(preview: &preview::Preview, request: &Request) -> AppResult<()> {
    if !preview.can_continue
        || preview.can_finalize
        || preview.attempt_id.as_deref() != Some(&request.attempt_id)
        || preview.revision != Some(request.revision)
        || preview.confirmation_digest.as_deref() != Some(&request.confirmation_digest)
    {
        return Err(fail(()));
    }
    Ok(())
}
pub fn resume(context: &ProcessContext, data: &Path, request: Request) -> AppResult<Outcome> {
    let started = Instant::now();
    let deadline = started + crate::modules::process::stop_timeout();
    let phase = std::cell::Cell::new("confirm-preview");
    let result: AppResult<Outcome> = (|| {
        // open_existing never initializes storage. The first authoritative
        // confirmation belongs INSIDE its exclusive action lock; an unlocked
        // preview followed by the same locked preview only duplicates work.
        if uuid::Uuid::parse_str(&request.attempt_id)
            .map(|v| v.to_string() != request.attempt_id)
            .unwrap_or(true)
            || !valid_hash(&request.confirmation_digest)
            || request.revision == 0
        {
            return Err(fail(()));
        }
        phase.set("open-existing-index");
        let client =
            std::fs::canonicalize(context.working_directory.join(".codex")).map_err(fail)?;
        let index = Index::open_existing(context, data, &client).map_err(fail)?;
        let head = index.head().map_err(fail)?.ok_or_else(|| fail(()))?;
        require(
            &preview::project(context, data, Some(&head)).map_err(fail)?,
            &request,
        )?;
        let refs = head.references.as_ref().ok_or_else(|| fail(()))?;
        let session = session::StartupSession::open(context, data, &client, refs)?;
        let coordination = Context {
            runtime: context.runtime.clone(),
            user_profile: context.working_directory.clone(),
            opencodex_home: context.opencodex_home.clone(),
            codex_home: client,
            control_root: index.control_root().map_err(fail)?,
        };
        phase.set("admit-and-exclude-writers");
        with_native_coordination(&coordination, deadline, |lease| {
            phase.set("continue-original-plan");
            session.continue_interrupted_plan(
                context,
                &head,
                &request.confirmation_digest,
                lease,
                deadline,
            )
        })?;
        phase.set("full-tree-and-lock-readback");
        let proof = session.continued_readback(context, &head, deadline)?;
        phase.set("close-index");
        let next = index.close_restored(&head, &proof).map_err(fail)?;
        Ok(Outcome {
            attempt_id: request.attempt_id,
            revision: next.revision,
            status: "restored",
        })
    })();
    if result.is_err() {
        let _ = crate::infrastructure::runtime_log::RuntimeLog::new(data).append_event(&format!(
            "client-recovery-continue: refused phase={} elapsed_ms={}",
            phase.get(),
            started.elapsed().as_millis()
        ));
    }
    // A failure may occur after one or more native publications. Do not leak
    // ordinary-session wording that claims no files were restored. The exact
    // pending plan and observed objects remain for a NEW explicit inspection.
    result.map_err(fail)
}
