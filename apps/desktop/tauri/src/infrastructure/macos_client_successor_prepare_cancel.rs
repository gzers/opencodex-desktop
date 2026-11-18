//! Read-only successor preparation review. It never constructs movement,
//! service-absence, restoration, cancellation completion or Start authority.
use super::*;
use crate::infrastructure::macos_client_originals::{GrokSnapshot, TreeSnapshot};

pub struct Review<'a> {
    index: &'a Index,
    context: &'a ProcessContext,
    head: Head,
    deadline: Instant,
    fingerprint: String,
}
impl Review<'_> {
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }
    pub fn recheck(&self) -> io::Result<()> {
        if snapshot(self.index, self.context, &self.head, self.deadline)? != self.fingerprint {
            return Err(denied("successor preparation review changed"));
        }
        Ok(())
    }
}
fn time(deadline: Instant) -> io::Result<()> {
    if Instant::now() >= deadline {
        return Err(denied("successor preparation review expired"));
    }
    Ok(())
}
fn snapshot(
    index: &Index,
    context: &ProcessContext,
    head: &Head,
    deadline: Instant,
) -> io::Result<String> {
    time(deadline)?;
    index.check()?;
    if index.head()?.as_ref() != Some(head)
        || head.state != State::Preparing
        || head.references.is_some()
        || head.restore_plan.is_some()
        || head.root_retirement.is_some()
        || head.completion_sha256.is_some()
        || context.working_directory != index.profile
        || context.opencodex_home != index.context_home
        || index.client.path() != context.working_directory.join(".codex")
    {
        return Err(denied(
            "successor cancellation is not the current preparation",
        ));
    }
    let origin = index
        .root_registration
        .as_ref()
        .filter(|r| r.id == head.ticket && r.generation_slot.as_deref() == Some(r.id.as_str()))
        .ok_or_else(|| denied("successor preparation lacks registered generation origin"))?;
    let generation = head
        .root_generation
        .as_ref()
        .ok_or_else(|| denied("successor preparation lacks its atomic initial event"))?;
    let history = vault_history::read(index.connection()?, head)?;
    let lineage = root_lineage::parse(&index.binding, &history)?;
    let current = lineage.current();
    let predecessor = history
        .get(history.len().saturating_sub(2))
        .ok_or_else(|| denied("successor predecessor is missing"))?;
    if lineage.generations().len() < 2
        || current.origin != *origin
        || current.heads != std::slice::from_ref(head)
        || generation.preparing_revision != head.revision
        || generation.predecessor_revision != predecessor.revision
        || predecessor.revision.checked_add(1) != Some(head.revision)
        || generation.predecessor_sha256 != digest(&encode(predecessor)?)
        || !matches!(
            predecessor.state,
            State::RootAbsent | State::RootCancellationAbsent
        )
        || predecessor.completion_sha256.is_none()
    {
        return Err(denied(
            "successor preparation differs from its complete indexed predecessor",
        ));
    }
    root_binding::verify_initial_event(index.connection()?, origin)?;
    let manager = index.data.child_private("manager-state", false)?;
    super::super::super::root_creation::verify_indexed(
        &manager,
        &index.data,
        &index.profile,
        &index.client,
        origin,
    )?;
    if !super::super::super::tree::names(index.client.file())?.is_empty() {
        return Err(denied("successor preparation contains new client data"));
    }
    // Ordinary cancellation's vault inspection accepts Restored/Unchanged
    // attempts. Created-root generations instead require the full independent
    // RootAbsent/cancelled-root lineage, archives and exact slot namespace.
    let closures = vault_history::inspect_root_history(
        &index.data,
        &index.profile,
        &index.binding,
        &history,
        None,
    )?;
    let tree = TreeSnapshot::read(&index.client)?;
    let profile = PinnedDirectory::open(&index.profile)?;
    let grok = GrokSnapshot::read(&profile)?;
    let fingerprint = digest(&encode(&(
        "macos-successor-preparation-review-v1",
        head,
        origin,
        digest(&index.binding),
        &history,
        &closures,
        tree,
        &grok,
    ))?);
    index.check()?;
    if index.head()?.as_ref() != Some(head)
        || vault_history::read(index.connection()?, head)? != history
        || GrokSnapshot::read(&profile)? != grok
        || !super::super::super::tree::names(index.client.file())?.is_empty()
    {
        return Err(denied("successor preparation changed during full review"));
    }
    super::super::super::root_creation::verify_indexed(
        &manager,
        &index.data,
        &index.profile,
        &index.client,
        origin,
    )?;
    time(deadline)?;
    Ok(fingerprint)
}
impl Index {
    /// This fact review borrows the original index. It creates no file, plan,
    /// receipt, native lease or mutation capability and never renews a deadline.
    pub fn review_successor_root_preparation<'a>(
        &'a self,
        context: &'a ProcessContext,
        head: &Head,
        deadline: Instant,
    ) -> io::Result<Review<'a>> {
        let first = snapshot(self, context, head, deadline)?;
        if snapshot(self, context, head, deadline)? != first {
            return Err(denied(
                "successor preparation was not stable across both observations",
            ));
        }
        Ok(Review {
            index: self,
            context,
            head: head.clone(),
            deadline,
            fingerprint: first,
        })
    }
}

#[cfg(test)]
#[path = "macos_client_successor_prepare_cancel_tests.rs"]
mod tests;
