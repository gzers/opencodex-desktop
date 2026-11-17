//! Original Owner-owned process half of fresh beforeResume recovery. No
//! caller-supplied DATA/serialized record constructs this state. Client prefix
//! checks, claim/NLK, transport and full restoration remain separate work.
use super::*;
use crate::modules::runtime::windows_lifecycle024_contract::BeforeResumeRecoveryWindow;
#[derive(PartialEq, Eq)]
enum ProcessPhase {
    Pending,
    Prepared,
    Failed,
}
pub(in crate::infrastructure::windows_official_lifecycle024) struct ProcessAbortAttempt {
    id: uuid::Uuid,
    owner: uuid::Uuid,
    failed_start: uuid::Uuid,
    original_outer: Instant,
    original_stop: Instant,
    original_end: Instant,
    end: Instant,
    scope: Option<String>,
    token: Option<native::TokenIdentity>,
    launcher: Option<native::PreparedLauncherAbort>,
    phase: ProcessPhase,
}
impl ProcessAbortAttempt {
    fn pending(
        owner: uuid::Uuid,
        start: &ActionOrigin,
        window: &BeforeResumeRecoveryWindow,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            owner,
            failed_start: start.id,
            original_outer: start.end,
            original_stop: start.stop_end,
            original_end: start.relevant_end(),
            end: window.deadline(),
            scope: None,
            token: None,
            launcher: None,
            phase: ProcessPhase::Pending,
        }
    }
    fn check_origin(&self, owner: uuid::Uuid, start: &ActionOrigin, end: Instant) -> AppResult<()> {
        budget(end)?;
        if owner != self.owner || end != self.end || self.id == self.failed_start {
            return Err(recovery_required());
        }
        // Read-only comparison of the actual original origin. Do NOT call its
        // mutation check with this fresh end or overwrite its old deadlines.
        start.check_retained_metadata(
            self.failed_start,
            OriginalActionKind::Start,
            self.original_outer,
            self.original_stop,
            self.original_end,
        )
    }
}
impl Owner {
    pub(in crate::infrastructure::windows_official_lifecycle024) fn require_normal_dispatch(
        &self,
    ) -> AppResult<()> {
        require_unoccupied(&self.before_resume_process)
    }
    /// Only the PROCESS portion, retained on this original Owner. No returned
    /// PreparedAbort, prefix permit, forced call or restored/Closed claim exists
    /// here; the complete adapter must validate its exact native store as well.
    pub(crate) fn prepare_before_resume_process(
        &mut self,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<()> {
        self.require_normal_dispatch()?;
        self.before_resume_process = Some(ProcessAbortAttempt::pending(
            self.session,
            &self.start_action,
            window,
        ));
        let result = (|| {
            let end = window.deadline();
            if self.stage != "created-suspended"
                || self.service.is_some()
                || self.typed_ready.is_some()
                || self.stopped
                || self.stop_action.is_some()
                || self.stop_started.is_some()
                || self.native.creator_has_resumed()
            {
                return Err(recovery_required());
            }
            let prepared = self
                .native
                .prepare_before_first_resume_abort(Instant::now(), end)
                .map_err(unknown)?;
            // Enroll the actual native preparation before every later fallible
            // check. Its parent is the exact original shared recovery end, so
            // the primitive cannot add time to it or the failed Start.
            self.before_resume_process.as_mut().unwrap().launcher = Some(prepared);
            if self
                .before_resume_process
                .as_ref()
                .unwrap()
                .launcher
                .as_ref()
                .unwrap()
                .deadline()
                != end
            {
                return Err(recovery_required());
            }
            self.source(end)?;
            let scope = self.scope()?;
            let token = native::current_identity().map_err(unknown)?;
            if token.elevated
                || token.elevation_type == 2
                || token != self.native.binding().process.token
            {
                return Err(recovery_required());
            }
            let original = self.before_resume_process.as_mut().unwrap();
            original.scope = Some(scope);
            original.token = Some(token);
            self.check_before_resume_process_inner(window, false)?;
            self.before_resume_process.as_mut().unwrap().phase = ProcessPhase::Prepared;
            Ok(())
        })();
        if result.is_err() {
            self.before_resume_process.as_mut().unwrap().phase = ProcessPhase::Failed;
        }
        result
    }
    pub(crate) fn check_before_resume_process(
        &mut self,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<()> {
        let result = self.check_before_resume_process_inner(window, true);
        if result.is_err() {
            if let Some(original) = &mut self.before_resume_process {
                original.phase = ProcessPhase::Failed;
            }
        }
        result
    }
    fn check_before_resume_process_inner(
        &mut self,
        window: &BeforeResumeRecoveryWindow,
        prepared: bool,
    ) -> AppResult<()> {
        let end = window.deadline();
        let original = self
            .before_resume_process
            .as_ref()
            .ok_or_else(recovery_required)?;
        if original.phase == ProcessPhase::Failed
            || prepared && original.phase != ProcessPhase::Prepared
        {
            return Err(recovery_required());
        }
        original.check_origin(self.session, &self.start_action, end)?;
        if original.scope.as_ref() != Some(&self.scope()?)
            || original.token.as_ref() != Some(&native::current_identity().map_err(unknown)?)
            || original
                .launcher
                .as_ref()
                .ok_or_else(recovery_required)?
                .deadline()
                != end
        {
            return Err(recovery_required());
        }
        self.source(end)?;
        self.native
            .verify_prepared_before_first_resume(
                self.before_resume_process
                    .as_ref()
                    .unwrap()
                    .launcher
                    .as_ref()
                    .unwrap(),
            )
            .map_err(unknown)?;
        budget(end)
    }
}
fn require_unoccupied(original: &Option<ProcessAbortAttempt>) -> AppResult<()> {
    if original.is_some() {
        return Err(recovery_required());
    }
    Ok(())
}
#[cfg(test)]
mod tests;
