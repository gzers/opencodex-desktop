//! Explicit user cancellation of the independently registered empty first root.
//! The preview supplies a confirmation digest only, never movement permission.
use super::finalization::{Outcome, Request};
use super::*;
use crate::{
    errors::{AppError, AppResult},
    infrastructure::official_client_coordination::{
        native::with_native_relocation_release, Context,
    },
};

fn fail(e: impl std::fmt::Display) -> AppError {
    AppError::RuntimeManaged {
        code: "client_first_root_cancel_refused".into(),
        detail: format!("首次目录取消未完成，原现场已保留；{e}"),
    }
}
pub fn applies(data: &Path) -> AppResult<bool> {
    let data = PinnedDirectory::open(data).map_err(fail)?;
    let manager = data.child_private("manager-state", false).map_err(fail)?;
    super::super::root_creation::has_creation_evidence(&manager).map_err(fail)
}
pub(super) fn confirmation(
    context: &ProcessContext,
    data: &Path,
    head: &Head,
    history: &[Head],
    origin: Option<&super::super::root_creation::References>,
) -> io::Result<Option<String>> {
    let Some(origin) = origin else {
        return Ok(None);
    };
    if !matches!(head.state, State::Preparing | State::CaptureReserved)
        || origin.generation_slot.is_some()
    {
        return Ok(None);
    }
    if head.ticket != origin.id
        || head.references.is_some()
        || head.restore_plan.is_some()
        || head.root_retirement.is_some()
        || head.completion_sha256.is_some()
        || head.root_generation.is_some()
        || head.revision > 2
        || history.len() != head.revision as usize
        || history
            .first()
            .is_none_or(|h| h.revision != 1 || h.state != State::Preparing)
        || history.last() != Some(head)
        || history.iter().any(|h| {
            h.ticket != head.ticket
                || h.references.is_some()
                || !matches!(h.state, State::Preparing | State::CaptureReserved)
        })
    {
        return Err(denied(
            "first-root confirmation is not exact uncaptured preparation",
        ));
    }
    let data = PinnedDirectory::open(data)?;
    let manager = data.child_private("manager-state", false)?;
    if observation::entry(&manager, "client-root-cancel-plans")?.is_some() {
        return Ok(None);
    }
    let logical = context.working_directory.join(".codex");
    let client = PinnedDirectory::open(&logical)?;
    if head.state == State::Preparing && !super::super::tree::names(client.file())?.is_empty() {
        return Ok(None);
    }
    super::super::root_creation::verify_indexed(
        &manager,
        &data,
        &context.working_directory,
        &client,
        origin,
    )?;
    let binding = root_binding::encode_binding(&scope(context, &data, &client)?, Some(origin))?;
    let tree = TreeSnapshot::read(&client)?;
    if head.state == State::CaptureReserved && !tree.first_root_execution_baseline() {
        return Ok(None);
    }
    let capture_vault = vault_history::inspect(
        &data,
        &client,
        &context.working_directory,
        &binding,
        head,
        history,
    )?;
    let profile = PinnedDirectory::open(&context.working_directory)?;
    let grok = GrokSnapshot::read(&profile)?;
    let digest = digest(&encode(&(
        "macos-first-root-preparation-review-v2",
        head,
        origin,
        digest(&binding),
        history,
        &tree,
        &grok,
        &capture_vault,
    ))?);
    if TreeSnapshot::read(&client)? != tree
        || GrokSnapshot::read(&profile)? != grok
        || vault_history::inspect(
            &data,
            &client,
            &context.working_directory,
            &binding,
            head,
            history,
        )? != capture_vault
    {
        return Err(denied("first-root cancellation confirmation changed"));
    }
    Ok(Some(digest))
}
pub fn cancel(context: &ProcessContext, data: &Path, request: Request) -> AppResult<Outcome> {
    let deadline = Instant::now() + crate::modules::process::stop_timeout();
    let index = Index::open_existing(context, data, &context.working_directory.join(".codex"))
        .map_err(fail)?;
    let head = index
        .head()
        .map_err(fail)?
        .ok_or_else(|| fail("independent preparation missing"))?;
    let review = index
        .review_first_root_preparation(context, &head, deadline)
        .map_err(fail)?;
    if request.attempt_id != head.ticket
        || request.revision != head.revision
        || request.confirmation_digest != review.fingerprint()
    {
        return Err(fail("confirmation changed"));
    }
    let service = super::root_preparation::preservation::service::CurrentService::begin(&review)
        .map_err(fail)?;
    let native_context = Context {
        runtime: context.runtime.clone(),
        user_profile: context.working_directory.clone(),
        codex_home: context.working_directory.join(".codex"),
        opencodex_home: context.opencodex_home.clone(),
        control_root: index.control_root().map_err(fail)?,
    };
    let (preserved, released) =
        with_native_relocation_release(&native_context, deadline, |native| {
            let mut guard = native.config_write_guard()?;
            let pending = super::root_preparation::preservation::Prepared::under_native_guard(
                review, native, &mut guard,
            )
            .and_then(|p| p.register())
            .map_err(fail)?;
            let authority = pending
                .authorize_current(&service, native, &mut guard)
                .map_err(fail)?;
            pending
                .preserve_guarded(&authority, native, guard)
                .map_err(fail)
        })?;
    let proof = super::root_preparation::preservation::completion::verify_released(
        &index, &service, preserved, released,
    )
    .map_err(fail)?;
    let pending = index
        .head()
        .map_err(fail)?
        .ok_or_else(|| fail("pending index missing"))?;
    let closed = index
        .close_first_root_cancellation(&pending, &proof)
        .map_err(fail)?;
    Ok(Outcome {
        attempt_id: head.ticket,
        revision: closed.revision,
        status: "root_cancelled",
    })
}
