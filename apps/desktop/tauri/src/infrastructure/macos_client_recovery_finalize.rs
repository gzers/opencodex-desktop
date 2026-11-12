//! Explicit completion of metadata after the client artifacts are ALREADY
//! fully restored by the original plan. No partial-file replay/rollback here.
use super::*;
use crate::errors::{AppError, AppResult};
use std::time::Instant;
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub attempt_id: String,
    pub revision: u64,
    pub confirmation_digest: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
    pub attempt_id: String,
    pub revision: u64,
    pub status: &'static str,
}
fn fail<T>(_: T) -> AppError {
    AppError::RuntimeManaged {
        code: "client_recovery_finalize_refused".into(),
        detail: "恢复现场或确认已变化，请重新核对；原件与当前文件保留".into(),
    }
}
fn require_review(preview: &preview::Preview, request: &Request) -> AppResult<()> {
    if !preview.can_finalize
        || preview.attempt_id.as_deref() != Some(&request.attempt_id)
        || preview.revision != Some(request.revision)
        || preview.confirmation_digest.as_deref() != Some(&request.confirmation_digest)
    {
        return Err(fail(()));
    }
    Ok(())
}
pub fn finalize(context: &ProcessContext, data: &Path, request: Request) -> AppResult<Outcome> {
    let started = Instant::now();
    let phase = std::cell::Cell::new("confirm-preview");
    let result = (|| {
        let deadline = Instant::now() + crate::modules::process::stop_timeout();
        // Refuse a stale/missing index BEFORE the action-time Index::open may
        // initialize any missing storage. Preview is not execution authority.
        require_review(&preview::inspect_preview(context, data), &request)?;
        phase.set("open-existing-index");
        let client =
            std::fs::canonicalize(context.working_directory.join(".codex")).map_err(fail)?;
        let index = Index::open_existing(context, data, &client).map_err(fail)?;
        let head = index.head().map_err(fail)?.ok_or_else(|| fail(()))?;
        phase.set("confirm-locked-preview");
        require_review(
            &preview::project(context, data, Some(&head)).map_err(fail)?,
            &request,
        )?;
        let refs = head.references.as_ref().ok_or_else(|| fail(()))?;
        let session = session::StartupSession::open(context, data, &client, refs)?;
        phase.set("admit-current-image");
        let source = context
            .runtime
            .managed_installation_root()
            .ok_or_else(|| fail(()))?;
        let (target, platform) = crate::modules::runtime::program_layout::host_target()?;
        let program =
            crate::modules::runtime::program_selection::read_complete(&source, platform, target)?
                .ok_or_else(|| fail(()))?;
        phase.set("confirm-admitted-preview");
        require_review(
            &preview::project(context, data, Some(&head)).map_err(fail)?,
            &request,
        )?;
        phase.set("full-tree-and-lock-readback");
        let proof = session.finalize_existing_complete(context, &head, deadline)?;
        phase.set("verify-current-image");
        crate::modules::runtime::program_selection::verify_current(&source, &program)?;
        crate::modules::runtime::program_image::verify(&program)?;
        phase.set("close-index");
        let next = index.close_restored(&head, &proof).map_err(fail)?;
        Ok(Outcome {
            attempt_id: request.attempt_id,
            revision: next.revision,
            status: "restored",
        })
    })();
    // Phase and elapsed time only: never write config bytes, paths or request
    // digests to a runtime diagnostic. Preserve the original failure result.
    if result.is_err() {
        let _ = crate::infrastructure::runtime_log::RuntimeLog::new(data).append_event(&format!(
            "client-recovery-finalize: refused phase={} elapsed_ms={}",
            phase.get(),
            started.elapsed().as_millis()
        ));
    }
    result
}
