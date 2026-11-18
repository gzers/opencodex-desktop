//! Canonical original-slot implementation of the unchanged recovery trait.
//! Real resources/prefixes stay on this owning worker outside unwind. V2 partial
//! preparation is deliberately unqualified here and cannot mint an empty tag.
use super::*;
use crate::{
    errors::{AppError, AppResult},
    infrastructure::windows_client_originals024::{PreparedAbortPrefix, QuiescentOriginals},
    modules::runtime::windows_lifecycle024_contract::BeforeFirstResumeRecovery,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Stage {
    Registered,
    Prepared,
    Quiescing,
    Quiescent,
    Restoring,
    Complete,
    Failed,
}
struct Binding {
    session: String,
    action: String,
    scope: String,
    end: Instant,
    nonce: uuid::Uuid,
    thread: thread::ThreadId,
}
impl Binding {
    fn check_metadata(
        &self,
        session: &str,
        action: &str,
        scope: &str,
        end: Instant,
        thread: thread::ThreadId,
    ) -> io::Result<()> {
        if Instant::now() >= end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        if self.session != session
            || self.action != action
            || self.scope != scope
            || self.end != end
            || self.thread != thread
        {
            return Err(invalid());
        }
        Ok(())
    }
    fn check(&self, owner: &mut Owner, window: &BeforeResumeRecoveryWindow) -> AppResult<()> {
        owner.check_before_resume_process(window)?;
        self.check_metadata(
            &owner.before_resume_owner_session(),
            &owner.before_resume_action_id(window)?,
            &owner.scope()?,
            window.deadline(),
            thread::current().id(),
        )
        .map_err(fail)
    }
}
pub(super) struct Attempt {
    binding: Binding,
    stage: Stage,
    prefix: Option<PreparedAbortPrefix>,
    quiescent: Option<QuiescentOriginals>,
}
/// Locators for the original retained native stage, never serde/Clone or a
/// reconstructed native permission. All constructors remain in this module.
pub(crate) struct PreparedCanonicalAbort {
    nonce: uuid::Uuid,
    end: Instant,
}
pub(crate) struct QuiescentCanonicalLauncher {
    nonce: uuid::Uuid,
    end: Instant,
}
pub(crate) struct RestoredCanonicalOriginals {
    nonce: uuid::Uuid,
    end: Instant,
}
fn fail(error: impl std::fmt::Display) -> AppError {
    AppError::RuntimeManaged {
        code: "windows024_canonical_trait_required".into(),
        detail: format!("原worker canonical恢复未确认：{error}"),
    }
}
impl WorkerLocal {
    fn canonical_binding(
        &mut self,
        owner: &mut Owner,
        window: &BeforeResumeRecoveryWindow,
        stage: Stage,
        nonce: uuid::Uuid,
        end: Instant,
    ) -> AppResult<()> {
        if self.revoked {
            return Err(fail(invalid()));
        }
        let original = self.canonical.as_ref().ok_or_else(|| fail(invalid()))?;
        if original.stage != stage || original.binding.nonce != nonce || end != original.binding.end
        {
            return Err(fail(invalid()));
        }
        original.binding.check(owner, window)
    }
    fn fail_canonical(&mut self, owner: &mut Owner) {
        if let Some(original) = &mut self.canonical {
            original.stage = Stage::Failed;
        }
        self.revoke();
        owner.fail_before_resume_process();
    }
}
impl BeforeFirstResumeRecovery for WorkerLocal {
    type Owner = Owner;
    type PreparedAbort = PreparedCanonicalAbort;
    type QuiescentLauncher = QuiescentCanonicalLauncher;
    type AbortRestored = RestoredCanonicalOriginals;
    fn prepare_abort(
        &mut self,
        owner: &mut Owner,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<Self::PreparedAbort> {
        let result = (|| {
            if self.revoked || self.canonical.is_some() {
                return Err(fail(invalid()));
            }
            owner.check_before_resume_process(window)?;
            let nonce = uuid::Uuid::new_v4();
            self.canonical = Some(Box::new(Attempt {
                binding: Binding {
                    session: owner.before_resume_owner_session(),
                    action: owner.before_resume_action_id(window)?,
                    scope: owner.scope()?,
                    end: window.deadline(),
                    nonce,
                    thread: thread::current().id(),
                },
                stage: Stage::Registered,
                prefix: None,
                quiescent: None,
            }));
            self.project_stage(Stage::Registered);
            self.prepare_canonical_lease(owner, window).map_err(fail)?;
            let root = owner.profile_path().join(".codex");
            let mut claim = Self::bound_claim(
                &mut self.lease,
                self.retained_recovery.as_ref(),
                owner,
                &root,
                window,
            )
            .map_err(fail)?;
            let prefix = owner.with_originals(|originals, owner| {
                originals.prepare_abort_prefix(owner, &mut claim, window)
            })?;
            self.canonical.as_mut().unwrap().prefix = Some(prefix);
            drop(claim);
            // A complete original prefix must qualify before creating an attachment.
            let directory = self.attachment_directory(owner, window).map_err(fail)?;
            owner.retain_before_resume_attachment_directory(directory, window)?;
            let mut claim = Self::bound_claim(
                &mut self.lease,
                self.retained_recovery.as_ref(),
                owner,
                &root,
                window,
            )
            .map_err(fail)?;
            let prefix = self.canonical.as_ref().unwrap().prefix.as_ref().unwrap();
            owner.with_originals(|originals, owner| {
                let directory = owner.take_before_resume_attachment_directory(window)?;
                originals.initialize_abort_attachment(owner, prefix, &mut claim, directory, window)
            })?;
            claim.check(&root, window.deadline()).map_err(fail)?;
            drop(claim);
            self.canonical
                .as_ref()
                .unwrap()
                .binding
                .check(owner, window)?;
            self.canonical.as_mut().unwrap().stage = Stage::Prepared;
            self.project_stage(Stage::Prepared);
            Ok(PreparedCanonicalAbort {
                nonce,
                end: window.deadline(),
            })
        })();
        if result.is_err() {
            self.fail_canonical(owner);
        }
        result
    }
    fn quiesce_launcher(
        &mut self,
        owner: &mut Owner,
        prepared: &Self::PreparedAbort,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<Self::QuiescentLauncher> {
        let result = (|| {
            self.canonical_binding(owner, window, Stage::Prepared, prepared.nonce, prepared.end)?;
            self.canonical.as_mut().unwrap().stage = Stage::Quiescing;
            self.project_stage(Stage::Quiescing);
            let root = owner.profile_path().join(".codex");
            let mut claim = Self::bound_claim(
                &mut self.lease,
                self.retained_recovery.as_ref(),
                owner,
                &root,
                window,
            )
            .map_err(fail)?;
            let attempt = &mut self.canonical;
            let quiescent = owner.with_originals(|originals, owner| {
                let prefix = attempt
                    .as_mut()
                    .unwrap()
                    .prefix
                    .take()
                    .ok_or_else(|| fail(invalid()))?;
                originals.quiesce_abort_prefix(owner, prefix, &mut claim, window)
            })?;
            self.canonical.as_mut().unwrap().quiescent = Some(quiescent);
            claim.check(&root, window.deadline()).map_err(fail)?;
            drop(claim);
            self.canonical
                .as_ref()
                .unwrap()
                .binding
                .check(owner, window)?;
            self.canonical.as_mut().unwrap().stage = Stage::Quiescent;
            self.project_stage(Stage::Quiescent);
            Ok(QuiescentCanonicalLauncher {
                nonce: prepared.nonce,
                end: prepared.end,
            })
        })();
        if result.is_err() {
            self.fail_canonical(owner);
        }
        result
    }
    fn restore_originals(
        &mut self,
        owner: &mut Owner,
        prepared: Self::PreparedAbort,
        quiescent: Self::QuiescentLauncher,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<Self::AbortRestored> {
        let result = (|| {
            self.canonical_binding(
                owner,
                window,
                Stage::Quiescent,
                prepared.nonce,
                prepared.end,
            )?;
            if quiescent.nonce != prepared.nonce || quiescent.end != prepared.end {
                return Err(fail(invalid()));
            }
            self.canonical.as_mut().unwrap().stage = Stage::Restoring;
            self.project_stage(Stage::Restoring);
            let root = owner.profile_path().join(".codex");
            let mut claim = Self::bound_claim(
                &mut self.lease,
                self.retained_recovery.as_ref(),
                owner,
                &root,
                window,
            )
            .map_err(fail)?;
            let proof = self
                .canonical
                .as_ref()
                .unwrap()
                .quiescent
                .as_ref()
                .ok_or_else(|| fail(invalid()))?;
            owner.with_originals(|originals, owner| {
                originals.return_quiescent_originals(owner, proof, &mut claim, window)
            })?;
            let attempt = &mut self.canonical;
            owner.with_originals(|originals, owner| {
                let proof = attempt
                    .as_mut()
                    .unwrap()
                    .quiescent
                    .take()
                    .ok_or_else(|| fail(invalid()))?;
                originals.retain_quiescent_for_release(owner, proof, window)
            })?;
            claim.check(&root, window.deadline()).map_err(fail)?;
            drop(claim);
            self.release(window.deadline()).map_err(fail)?;
            self.finish_released_stage(owner, window).map_err(fail)?;
            owner.before_resume_restored_data(window)?;
            self.canonical.as_mut().unwrap().stage = Stage::Complete;
            self.project_stage(Stage::Complete);
            Ok(RestoredCanonicalOriginals {
                nonce: prepared.nonce,
                end: prepared.end,
            })
        })();
        if result.is_err() {
            self.fail_canonical(owner);
        }
        result
    }
}
impl RestoredCanonicalOriginals {
    pub(crate) fn check_original(
        &self,
        local: &WorkerLocal,
        owner: &mut Owner,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<()> {
        let original = local.canonical.as_ref().ok_or_else(|| fail(invalid()))?;
        if local.revoked
            || original.stage != Stage::Complete
            || original.binding.nonce != self.nonce
            || original.binding.end != self.end
            || self.end != window.deadline()
        {
            return Err(fail(invalid()));
        }
        original.binding.check(owner, window)?;
        owner.before_resume_restored_data(window).map(|_| ())
    }
}
#[cfg(test)]
mod tests;
