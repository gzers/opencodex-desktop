//! Explicit review of an already retired root. No file replay or native Stop
//! certificate can be constructed from the old indexed metadata.
use super::super::root_retirement::{ArchiveReferences, Pending, RetiredCompletionSnapshot};
use super::*;
use crate::errors::{AppError, AppResult};
use std::time::Instant;
const DOMAIN: &str = "root-retirement-metadata-finalize/v1";
pub(super) struct Review {
    fingerprint: String,
    receipt_sha256: String,
    locks: [Identity; 2],
}
impl Review {
    pub(super) fn digest(&self) -> &str {
        &self.fingerprint
    }
}
fn fail<T>(_: T) -> AppError {
    AppError::RuntimeManaged {
        code: "client_root_finalize_refused".into(),
        detail: "根恢复收尾证据或当前运行状态未确认，请重新核对；原件与当前文件已保留".into(),
    }
}
fn time(deadline: Instant) -> AppResult<()> {
    if Instant::now() >= deadline {
        Err(AppError::Timeout)
    } else {
        Ok(())
    }
}
pub(super) fn review(
    context: &ProcessContext,
    data: &Path,
    head: &Head,
    history: &[Head],
    origin: &super::super::root_creation::References,
) -> io::Result<Option<Review>> {
    if head.state != State::RootRetirementPrepared {
        return Ok(None);
    }
    let refs = head
        .references
        .as_ref()
        .ok_or_else(|| denied("retired completion refs missing"))?;
    if refs.phases.iter().map(|(p, _)| *p).collect::<Vec<_>>()
        != [Phase::Ready, Phase::PreStop, Phase::Stopped]
        || refs.live_sha256.is_none()
    {
        return Ok(None);
    }
    if history.last() != Some(head) || head.ticket != origin.id {
        return Err(denied("retired completion independent history differs"));
    }
    let profile = PinnedDirectory::open(&context.working_directory)?;
    if observation::entry(&profile, ".codex")?.is_some() {
        return Ok(None);
    }
    let movement = head
        .root_retirement
        .as_ref()
        .ok_or_else(|| denied("retired completion plan unregistered"))?;
    let source = history
        .iter()
        .find(|v| v.revision == movement.source_revision)
        .ok_or_else(|| denied("retired completion independent source absent"))?;
    if !matches!(source.state, State::Stopped | State::RestorePrepared)
        || source.references != head.references
        || source.restore_plan != head.restore_plan
        || source.ticket != head.ticket
        || source.root_retirement.is_some()
    {
        return Err(denied("retired completion source rebound"));
    }
    let reference = ArchiveReferences {
        source: source.clone(),
        original_root: origin.clone(),
        plan_id: movement.plan.id.clone(),
        plan_sha256: movement.plan.sha256.clone(),
    };
    let originals = super::super::root_retirement::open_archived_originals(
        data,
        &context.working_directory,
        &reference,
    )?;
    if observation::entry(
        &originals.directory,
        &format!("root-retirement-{}-Retire-readback.json", movement.plan.id),
    )?
    .is_none()
    {
        return Ok(None);
    }
    let pending = Pending::open(&originals, &movement.plan.id, &movement.plan.sha256)?;
    let snapshot: RetiredCompletionSnapshot = pending.retired_completion_snapshot(source)?;
    let frames = originals.frames_through(Phase::Stopped)?;
    session::verify_index_scope_prefix(&originals, refs, &context.opencodex_home, &frames)
        .map_err(|_| denied("retired completion logical scope lost"))?;
    for event in history
        .iter()
        .filter(|v| v.ticket == head.ticket && v.state == State::RootAbsent)
    {
        if event.completion_sha256.as_deref() != Some(&snapshot.receipt_sha256) {
            return Err(denied("retired completion prior close digest differs"));
        }
    }
    profile.recheck()?;
    if observation::entry(&profile, ".codex")?.is_some() {
        return Err(denied("retired completion logical root reappeared"));
    }
    let fingerprint = digest(&encode(&(
        DOMAIN,
        head,
        digest(&encode(&history)?),
        source,
        origin,
        &snapshot,
        refs.scope_sha256.as_str(),
        refs.live_sha256.as_deref(),
        &context.opencodex_home,
    ))?);
    Ok(Some(Review {
        fingerprint,
        receipt_sha256: snapshot.receipt_sha256,
        locks: snapshot.stable_locks,
    }))
}
impl Index {
    fn retired_review(
        &self,
        context: &ProcessContext,
        expected: &Head,
    ) -> io::Result<(Review, Originals)> {
        if context.working_directory != self.profile || context.opencodex_home != self.context_home
        {
            return Err(denied("retired completion context changed"));
        }
        let (head, originals) = self.checked_head_originals()?;
        if head.as_ref() != Some(expected) {
            return Err(denied("retired completion head changed"));
        }
        let history = vault_history::read(self.connection()?, expected)?;
        let lineage = root_lineage::parse(&self.binding, &history)?;
        let observed = review(
            context,
            self.data.path(),
            expected,
            &history,
            &lineage.current().origin,
        )?
        .ok_or_else(|| denied("retired completion is not ready"))?;
        Ok((
            observed,
            originals.ok_or_else(|| denied("retired completion originals absent"))?,
        ))
    }
}
/// New explicit metadata recovery only. Cannot be converted into a prior
/// native Stop/movement certificate or supplied from an IPC document.
struct RecoveryCompletion<'a> {
    index: &'a Index,
    context: &'a ProcessContext,
    head: Head,
    review: Review,
    originals: Originals,
    program: crate::modules::runtime::program_selection::SelectedProgram,
    deadline: Instant,
}
impl RecoveryCompletion<'_> {
    fn check_closed(&self, expected: &Head) -> AppResult<()> {
        time(self.deadline)?;
        let root = self.index.program_root(self.context)?;
        crate::modules::runtime::program_selection::verify_current(&root, &self.program)?;
        if self.index.head().map_err(fail)?.as_ref() != Some(expected) {
            return Err(fail(()));
        }
        let history =
            vault_history::read(self.index.connection().map_err(fail)?, expected).map_err(fail)?;
        let lineage = root_lineage::parse(&self.index.binding, &history).map_err(fail)?;
        let last = history.len().checked_sub(1).ok_or_else(|| fail(()))?;
        let current = review(
            self.context,
            self.index.data.path(),
            &self.head,
            &history[..last],
            &lineage.current().origin,
        )
        .map_err(fail)?
        .ok_or_else(|| fail(()))?;
        if current.fingerprint != self.review.fingerprint
            || current.receipt_sha256 != self.review.receipt_sha256
        {
            return Err(fail(()));
        }
        self.check_current()?;
        time(self.deadline)
    }
    fn check_current(&self) -> AppResult<()> {
        time(self.deadline)?;
        let root = self
            .context
            .runtime
            .managed_installation_root()
            .ok_or_else(|| fail(()))?;
        crate::modules::runtime::program_selection::verify_current(&root, &self.program)?;
        session::require_retired_completion_absence(
            self.context,
            &self.originals,
            self.head.references.as_ref().ok_or_else(|| fail(()))?,
        )?;
        completion::prove_optional_released(
            &self.originals.client,
            &self.review.locks.clone().map(Some),
            self.deadline,
        )
        .map_err(fail)?;
        time(self.deadline)
    }
    fn require_index(&self, index: &Index, head: &Head) -> io::Result<()> {
        if !std::ptr::eq(self.index, index) || self.head != *head || Instant::now() >= self.deadline
        {
            return Err(denied(
                "retired recovery proof not current index/head/deadline",
            ));
        }
        Ok(())
    }
}
fn next_head(pending: &Head, receipt: &str) -> io::Result<Head> {
    if pending.state != State::RootRetirementPrepared || pending.root_retirement.is_none() {
        return Err(denied("retired recovery current phase invalid"));
    }
    let mut next = pending.clone();
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or_else(|| denied("retired recovery revision overflow"))?;
    next.state = State::RootAbsent;
    next.completion_sha256 = Some(receipt.into());
    Ok(next)
}
impl Index {
    fn close_root_recovered(&self, proof: &RecoveryCompletion<'_>) -> AppResult<Head> {
        proof.require_index(self, &proof.head).map_err(fail)?;
        let next = next_head(&proof.head, &proof.review.receipt_sha256).map_err(fail)?;
        let write = self.write_checked_projected(
            Some(&proof.head),
            &next,
            &|point| {
                if matches!(point, "references_sql_written" | "references_committed") {
                    proof
                        .check_current()
                        .map_err(|_| denied("retired recovery current proof failed at commit"))?;
                }
                Ok(())
            },
            &|_| {
                proof
                    .check_current()
                    .map_err(|_| denied("retired recovery admission unconfirmed"))
            },
            &|_, _, _| Ok(()),
        );
        let result = (|| {
            write.map_err(fail)?;
            proof.check_current()?;
            crate::modules::runtime::program_selection::verify_current(
                &self.program_root(proof.context)?,
                &proof.program,
            )?;
            crate::modules::runtime::program_image::verify(&proof.program)?;
            // Full image reads take time; current facts must be checked again
            // after the scan, not accepted from its beginning.
            proof.check_closed(&next)?;
            Ok(next.clone())
        })();
        if result.is_err()
            && read_head(self.connection().map_err(fail)?)
                .map_err(fail)?
                .as_ref()
                == Some(&next)
        {
            let mut pending = proof.head.clone();
            pending.revision = next.revision.checked_add(1).ok_or_else(|| fail(()))?;
            // Never pretend COMMIT rolled back. Reappend the same pending
            // decision only if its full independent source is still valid.
            self.write(Some(&next), &pending).map_err(fail)?;
        }
        result
    }
    fn program_root(&self, context: &ProcessContext) -> AppResult<PathBuf> {
        context
            .runtime
            .managed_installation_root()
            .ok_or_else(|| fail(()))
    }
}
pub fn finalize(
    context: &ProcessContext,
    data: &Path,
    request: super::finalization::Request,
    deadline: Instant,
) -> AppResult<super::finalization::Outcome> {
    let deadline = deadline.min(Instant::now() + crate::modules::process::stop_timeout());
    time(deadline)?;
    if uuid::Uuid::parse_str(&request.attempt_id)
        .map(|v| v.to_string() != request.attempt_id)
        .unwrap_or(true)
        || request.revision == 0
        || !valid_hash(&request.confirmation_digest)
    {
        return Err(fail(()));
    }
    let index = Index::open_existing(context, data, &context.working_directory.join(".codex"))
        .map_err(fail)?;
    let head = index.head().map_err(fail)?.ok_or_else(|| fail(()))?;
    let history = vault_history::read(index.connection().map_err(fail)?, &head).map_err(fail)?;
    let lineage = root_lineage::parse(&index.binding, &history).map_err(fail)?;
    let pending = if head.state == State::RootAbsent {
        history
            .get(history.len().checked_sub(2).ok_or_else(|| fail(()))?)
            .cloned()
            .ok_or_else(|| fail(()))?
    } else {
        head.clone()
    };
    let reviewed_history = if head.state == State::RootAbsent {
        &history[..history.len() - 1]
    } else {
        &history[..]
    };
    let reviewed = review(
        context,
        data,
        &pending,
        reviewed_history,
        &lineage.current().origin,
    )
    .map_err(fail)?
    .ok_or_else(|| fail(()))?;
    if pending.ticket != request.attempt_id
        || pending.revision != request.revision
        || reviewed.fingerprint != request.confirmation_digest
    {
        return Err(fail(()));
    }
    if head.state == State::RootAbsent
        && next_head(&pending, &reviewed.receipt_sha256).map_err(fail)? != head
    {
        return Err(fail(()));
    }
    let originals = if head.state == State::RootRetirementPrepared {
        let (current, originals) = index.retired_review(context, &head).map_err(fail)?;
        if current.fingerprint != reviewed.fingerprint {
            return Err(fail(()));
        }
        originals
    } else {
        index.registered_root_originals(&head).map_err(fail)?
    };
    let root = index.program_root(context)?;
    let (target, platform) = crate::modules::runtime::program_layout::host_target()?;
    let program =
        crate::modules::runtime::program_selection::read_complete(&root, platform, target)?
            .ok_or_else(|| fail(()))?;
    if program.selection.version != "2.99.0" || context.executable()? != program.stable_entry {
        return Err(fail(()));
    }
    let proof = RecoveryCompletion {
        index: &index,
        context,
        head: pending,
        review: reviewed,
        originals,
        program,
        deadline,
    };
    proof.check_current()?;
    let closed = if head.state == State::RootAbsent {
        // Idempotent retry of the exact predecessor confirmation, after all
        // current root/Grok/program/process/lock checks; no extra revision.
        crate::modules::runtime::program_image::verify(&proof.program)?;
        #[cfg(test)]
        if std::env::var_os("OCXD024_RETIRED_DRIFT_AFTER_IMAGE").as_deref()
            == Some(std::ffi::OsStr::new("1"))
        {
            if !context.working_directory.starts_with("/private/tmp")
                || !context.working_directory.ancestors().any(|p| {
                    p.file_name().is_some_and(|n| {
                        n.to_string_lossy()
                            .starts_with("ocxd024-versioned-lifecycle-")
                    })
                })
            {
                return Err(fail(()));
            }
            let grok = context.working_directory.join(".grok");
            crate::infrastructure::platform::create_private_dir_all(&grok, 0o700).map_err(fail)?;
            let mut file = crate::infrastructure::platform::create_new_private_file(
                &grok.join("config.toml"),
                0o600,
            )
            .map_err(fail)?;
            std::io::Write::write_all(&mut file, b"owned-test-drift-after-image=1\n")
                .map_err(fail)?;
        }
        let current = index.head().map_err(fail)?.ok_or_else(|| fail(()))?;
        if current != head {
            return Err(fail(()));
        }
        proof.check_closed(&head)?;
        head
    } else {
        index.close_root_recovered(&proof)?
    };
    Ok(super::finalization::Outcome {
        attempt_id: request.attempt_id,
        revision: closed.revision,
        status: "restored",
    })
}
#[cfg(test)]
#[path = "macos_client_retired_completion_tests.rs"]
mod tests;
