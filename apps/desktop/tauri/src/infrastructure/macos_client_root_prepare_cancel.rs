//! Read-only preflight for cancelling a registered first-root preparation.
//! This is NOT a movement, service-absence, restore, Closed or successor permit.
//! No public disk reference can construct the borrowed independent index.
use super::*;
use std::time::Instant;
#[path = "macos_client_root_prepare_preserve.rs"]
pub mod preservation;

pub struct Review<'a> {
    index: &'a Index,
    head: Head,
    context: &'a ProcessContext,
    fingerprint: String,
    deadline: Instant,
    initial_grok: super::super::GrokSnapshot,
    history_sha256: String,
    capture_vault: vault_history::Evidence,
}
impl Review<'_> {
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }
    /// The original deadline is retained, never reset by a later readback.
    pub fn recheck(&self) -> io::Result<()> {
        if snapshot(self.index, self.context, &self.head, self.deadline)? != self.fingerprint {
            return Err(denied("first-root cancellation review changed"));
        }
        Ok(())
    }
}
fn check_time(deadline: Instant) -> io::Result<()> {
    if Instant::now() >= deadline {
        Err(denied("first-root cancellation review expired"))
    } else {
        Ok(())
    }
}
fn snapshot(
    index: &Index,
    context: &ProcessContext,
    head: &Head,
    deadline: Instant,
) -> io::Result<String> {
    check_time(deadline)?;
    index.check()?;
    if index.head()?.as_ref() != Some(head)
        || !matches!(head.state, State::Preparing | State::CaptureReserved)
        || head.references.is_some()
        || head.restore_plan.is_some()
        || head.root_retirement.is_some()
        || head.completion_sha256.is_some()
        || head.root_generation.is_some()
        || context.working_directory != index.profile
        || context.opencodex_home != index.context_home
    {
        return Err(denied(
            "first-root cancellation is not the current preparation",
        ));
    }
    // Successor generations require their own complete historical Original
    // verification. Do not silently extend this initial-root preflight.
    let origin = index
        .root_registration
        .as_ref()
        .filter(|origin| origin.id == head.ticket && origin.generation_slot.is_none())
        .ok_or_else(|| denied("first-root cancellation lacks registered initial absence"))?;
    let history = vault_history::read(index.connection()?, head)?;
    let first = history
        .first()
        .ok_or_else(|| denied("first-root history is absent"))?;
    if first.revision != 1
        || first.state != State::Preparing
        || first.ticket != origin.id
        || history.last() != Some(head)
        || history.len() != head.revision as usize
        || history.iter().any(|h| {
            h.ticket != head.ticket
                || h.references.is_some()
                || !matches!(h.state, State::Preparing | State::CaptureReserved)
        })
        || head.revision > 2
    {
        return Err(denied(
            "first-root capture or another lifecycle phase has already begun",
        ));
    }
    let manager = index.data.child_private("manager-state", false)?;
    super::super::root_creation::verify_indexed(
        &manager,
        &index.data,
        &index.profile,
        &index.client,
        origin,
    )?;
    // The original reservation owns an exact retained slot, not complete
    // Originals. Unknown slots, links, permission/content drift still refuse.
    let capture_vault = vault_history::inspect(
        &index.data,
        &index.client,
        &index.profile,
        &index.binding,
        head,
        &history,
    )?;
    if index.client.path() != context.working_directory.join(".codex")
        || head.state == State::Preparing
            && !super::super::tree::names(index.client.file())?.is_empty()
    {
        return Err(denied(
            "created root contains new data or changed logical path",
        ));
    }
    let profile = PinnedDirectory::open(&index.profile)?;
    let grok = super::super::GrokSnapshot::read(&profile)?;
    let tree = super::super::TreeSnapshot::read(&index.client)?;
    if head.state == State::CaptureReserved && !tree.first_root_execution_baseline() {
        return Err(denied(
            "reserved first-root contains client data; not an initialization baseline",
        ));
    }
    let fingerprint = digest(&encode(&(
        "macos-first-root-preparation-review-v2",
        head,
        origin,
        digest(&index.binding),
        &history,
        tree,
        grok,
        &capture_vault,
    ))?);
    index.check()?;
    super::super::root_creation::verify_indexed(
        &manager,
        &index.data,
        &index.profile,
        &index.client,
        origin,
    )?;
    if (head.state == State::Preparing
        && !super::super::tree::names(index.client.file())?.is_empty())
        || index.head()?.as_ref() != Some(head)
        || vault_history::inspect(
            &index.data,
            &index.client,
            &index.profile,
            &index.binding,
            head,
            &history,
        )? != capture_vault
    {
        return Err(denied("first-root preparation changed during review"));
    }
    check_time(deadline)?;
    Ok(fingerprint)
}
impl Index {
    /// Only retained index/context and actual root objects are read. No new
    /// index, directory, capture, cancellation receipt or client file is made.
    pub fn review_first_root_preparation<'a>(
        &'a self,
        context: &'a ProcessContext,
        head: &Head,
        deadline: Instant,
    ) -> io::Result<Review<'a>> {
        let initial_grok =
            super::super::GrokSnapshot::read(&PinnedDirectory::open(&self.profile)?)?;
        let history_sha256 = digest(&encode(&vault_history::read(self.connection()?, head)?)?);
        let capture_vault = vault_history::inspect(
            &self.data,
            &self.client,
            &self.profile,
            &self.binding,
            head,
            &vault_history::read(self.connection()?, head)?,
        )?;
        let first = snapshot(self, context, head, deadline)?;
        if snapshot(self, context, head, deadline)? != first
            || super::super::GrokSnapshot::read(&PinnedDirectory::open(&self.profile)?)?
                != initial_grok
            || digest(&encode(&vault_history::read(self.connection()?, head)?)?) != history_sha256
            || vault_history::inspect(
                &self.data,
                &self.client,
                &self.profile,
                &self.binding,
                head,
                &vault_history::read(self.connection()?, head)?,
            )? != capture_vault
        {
            return Err(denied("first-root preparation did not remain stable"));
        }
        Ok(Review {
            index: self,
            context,
            head: head.clone(),
            fingerprint: first,
            deadline,
            initial_grok,
            history_sha256,
            capture_vault,
        })
    }
}

#[cfg(test)]
#[path = "macos_client_root_prepare_cancel_tests.rs"]
mod tests;
