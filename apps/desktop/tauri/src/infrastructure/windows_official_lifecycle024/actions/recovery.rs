//! Original Owner-owned process half of fresh beforeResume recovery. No
//! caller-supplied DATA/serialized record constructs this state. Client prefix
//! checks, claim/NLK, transport and full restoration remain separate work.
use super::*;
use crate::modules::runtime::windows_lifecycle024_contract::BeforeResumeRecoveryWindow;
mod clock;
pub(in crate::infrastructure::windows_official_lifecycle024) mod driver;
use clock::RecoveryClock;
#[derive(PartialEq, Eq)]
enum ProcessPhase {
    Pending,
    Prepared,
    Quiescing,
    Quiescent,
    Failed,
}
// Minted only from this original Start's locally returned Result, never an IPC
// failed flag, error string or deserialized attachment. The Result is returned
// unchanged to the caller; this retains only its original failure provenance.
pub(in crate::infrastructure::windows_official_lifecycle024) struct StartFailure {
    owner: uuid::Uuid,
    start: uuid::Uuid,
    observed_at: Instant,
    creator_resumed: bool,
}
impl StartFailure {
    fn record(
        owner: uuid::Uuid,
        start: uuid::Uuid,
        creator_resumed: bool,
        result: &AppResult<LifecycleResult>,
    ) -> Option<Self> {
        result.as_ref().err().map(|_| Self {
            owner,
            start,
            observed_at: Instant::now(),
            creator_resumed,
        })
    }
}
pub(in crate::infrastructure::windows_official_lifecycle024) struct ProcessAbortAttempt {
    id: uuid::Uuid,
    owner: uuid::Uuid,
    failed_start: uuid::Uuid,
    original_outer: Instant,
    original_stop: Instant,
    original_end: Instant,
    failure_at: Option<Instant>,
    end: Instant,
    scope: Option<String>,
    token: Option<native::TokenIdentity>,
    launcher: Option<native::PreparedLauncherAbort>,
    quiescent: Option<native::QuiescentLauncher>,
    clock: Option<RecoveryClock>,
    restored: Option<crate::infrastructure::windows_client_originals024::AbortedOriginals>,
    attachment_directory: Option<native::PinnedDirectory>,
    phase: ProcessPhase,
}
impl ProcessAbortAttempt {
    fn pending(
        owner: uuid::Uuid,
        start: &ActionOrigin,
        failure: Option<&StartFailure>,
        window: &BeforeResumeRecoveryWindow,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            owner,
            failed_start: start.id,
            original_outer: start.end,
            original_stop: start.stop_end,
            original_end: start.relevant_end(),
            failure_at: failure.map(|f| f.observed_at),
            end: window.deadline(),
            scope: None,
            token: None,
            launcher: None,
            quiescent: None,
            clock: None,
            restored: None,
            attachment_directory: None,
            phase: ProcessPhase::Pending,
        }
    }
    fn check_origin(
        &self,
        owner: uuid::Uuid,
        start: &ActionOrigin,
        failure: Option<&StartFailure>,
        end: Instant,
    ) -> AppResult<()> {
        budget(end)?;
        self.check_origin_metadata(owner, start, failure, end)
    }
    fn check_origin_metadata(
        &self,
        owner: uuid::Uuid,
        start: &ActionOrigin,
        failure: Option<&StartFailure>,
        original_end: Instant,
    ) -> AppResult<()> {
        let failure = failure.ok_or_else(recovery_required)?;
        if owner != self.owner
            || original_end != self.end
            || self.id == self.failed_start
            || failure.owner != owner
            || failure.start != self.failed_start
            || Some(failure.observed_at) != self.failure_at
            || failure.observed_at < start.admitted_at
            || failure.observed_at > Instant::now()
            || failure.creator_resumed
        {
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
    pub(crate) fn check_before_resume_history(
        &mut self,
        window: &BeforeResumeRecoveryWindow,
        read_end: Instant,
    ) -> AppResult<()> {
        let result = (|| {
            budget(read_end)?;
            let original = self
                .before_resume_process
                .as_ref()
                .ok_or_else(recovery_required)?;
            original.check_origin_metadata(
                self.session,
                &self.start_action,
                self.start_failure.as_ref(),
                window.deadline(),
            )?;
            if original.phase != ProcessPhase::Quiescent
                || original.restored.is_none()
                || original.launcher.is_some()
                || original.scope.as_ref() != Some(&self.scope()?)
                || original.token.as_ref()
                    != Some(&native::current_identity().map_err(native_failure)?)
            {
                return Err(recovery_required());
            }
            original
                .clock
                .as_ref()
                .ok_or_else(recovery_required)?
                .check_history(window.deadline(), read_end)
                .map_err(native_failure)?;
            original
                .restored
                .as_ref()
                .unwrap()
                .read_history_data(read_end)?;
            self.source(read_end)?;
            let original = self.before_resume_process.as_ref().unwrap();
            let proof = original.quiescent.as_ref().ok_or_else(recovery_required)?;
            if proof.deadline() != window.deadline() {
                return Err(recovery_required());
            }
            self.native
                .read_before_resume_quiescent_history(proof, read_end)
                .map_err(native_failure)?;
            budget(read_end)
        })();
        if result.is_err() {
            self.fail_before_resume_process();
        }
        result
    }
    pub(crate) fn before_resume_history_origin(
        &mut self,
        window: &BeforeResumeRecoveryWindow,
        read_end: Instant,
    ) -> AppResult<(String, [String; 3])> {
        self.check_before_resume_history(window, read_end)?;
        let original = self.before_resume_process.as_ref().unwrap();
        Ok((
            original.failed_start.to_string(),
            original.clock.as_ref().unwrap().data(),
        ))
    }
    pub(crate) fn before_resume_history_binding(
        &mut self,
        window: &BeforeResumeRecoveryWindow,
        read_end: Instant,
    ) -> AppResult<[String; 3]> {
        self.check_before_resume_history(window, read_end)?;
        Ok([
            self.session.to_string(),
            self.before_resume_process.as_ref().unwrap().id.to_string(),
            self.scope()?,
        ])
    }
    pub(crate) fn before_resume_restored_history_data(
        &mut self,
        window: &BeforeResumeRecoveryWindow,
        read_end: Instant,
    ) -> AppResult<serde_json::Value> {
        self.check_before_resume_history(window, read_end)?;
        self.before_resume_process
            .as_ref()
            .unwrap()
            .restored
            .as_ref()
            .unwrap()
            .read_history_data(read_end)
    }
    pub(crate) fn before_resume_restored_data(
        &mut self,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<serde_json::Value> {
        self.check_before_resume_quiescent(window)?;
        self.before_resume_process
            .as_ref()
            .unwrap()
            .restored
            .as_ref()
            .ok_or_else(recovery_required)?
            .read_data(window.deadline())
    }
    pub(crate) fn retain_before_resume_restored(
        &mut self,
        proof: crate::infrastructure::windows_client_originals024::AbortedOriginals,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<()> {
        if self
            .before_resume_process
            .as_ref()
            .ok_or_else(recovery_required)?
            .restored
            .is_some()
        {
            return Err(recovery_required());
        }
        self.before_resume_process.as_mut().unwrap().restored = Some(proof);
        self.check_before_resume_quiescent(window)?;
        self.before_resume_process
            .as_ref()
            .unwrap()
            .restored
            .as_ref()
            .unwrap()
            .read_data(window.deadline())?;
        Ok(())
    }
    pub(crate) fn take_before_resume_attachment_directory(
        &mut self,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<native::PinnedDirectory> {
        self.check_before_resume_process(window)?;
        self.before_resume_process
            .as_mut()
            .unwrap()
            .attachment_directory
            .take()
            .ok_or_else(recovery_required)
    }
    pub(crate) fn before_resume_owner_session(&self) -> String {
        self.session.to_string()
    }
    pub(crate) fn before_resume_quiescent_record(
        &mut self,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<(u32, u64, u32)> {
        self.check_before_resume_quiescent(window)?;
        let binding = &self.native.binding().process;
        let proof = self
            .before_resume_process
            .as_ref()
            .unwrap()
            .quiescent
            .as_ref()
            .unwrap();
        Ok((
            binding.pid,
            binding.creation_filetime,
            proof.forced_raw_exit(),
        ))
    }
    /// DATA projection only from this original retained recovery action. It
    /// never creates a native deadline or action from serialized QPC strings.
    pub(crate) fn before_resume_record_origin(
        &mut self,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<(String, [String; 3])> {
        self.check_before_resume_process(window)?;
        let original = self.before_resume_process.as_ref().unwrap();
        let clock = original.clock.as_ref().ok_or_else(recovery_required)?;
        clock.check(window.deadline()).map_err(native_failure)?;
        Ok((original.failed_start.to_string(), clock.data()))
    }
    pub(crate) fn fail_before_resume_process(&mut self) {
        if let Some(original) = &mut self.before_resume_process {
            original.phase = ProcessPhase::Failed;
        }
    }
    pub(crate) fn before_resume_action_id(
        &mut self,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<String> {
        self.check_before_resume_process(window)?;
        Ok(self.before_resume_process.as_ref().unwrap().id.to_string())
    }
    pub(crate) fn quiesce_before_resume_process(
        &mut self,
        prefix: &mut crate::infrastructure::windows_client_originals024::ClaimBoundAbort<'_, '_>,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<()> {
        let result = (|| {
            prefix.check_owner(self, window)?;
            let original = self
                .before_resume_process
                .as_mut()
                .ok_or_else(recovery_required)?;
            if original.phase != ProcessPhase::Prepared || original.quiescent.is_some() {
                return Err(recovery_required());
            }
            original.phase = ProcessPhase::Quiescing;
            let prepared = original.launcher.take().ok_or_else(recovery_required)?;
            let quiescent = self
                .native
                .quiesce_before_first_resume(prepared)
                .map_err(native_failure)?;
            // Retain the ACTUAL native result before any post-quiescence check.
            // Failed postchecks never discard it or turn raw242 into NormalExit.
            let original = self.before_resume_process.as_mut().unwrap();
            original.quiescent = Some(quiescent);
            original.phase = ProcessPhase::Quiescent;
            self.check_before_resume_quiescent(window)?;
            prefix.check_owner(self, window)
        })();
        if result.is_err() {
            self.fail_before_resume_process();
        }
        result
    }
    pub(crate) fn check_before_resume_quiescent(
        &mut self,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<()> {
        self.check_before_resume_process(window)?;
        let original = self.before_resume_process.as_ref().unwrap();
        if original.phase != ProcessPhase::Quiescent || original.quiescent.is_none() {
            return Err(recovery_required());
        }
        Ok(())
    }
    pub(in crate::infrastructure::windows_official_lifecycle024) fn remember_start_failure(
        &mut self,
        result: &AppResult<LifecycleResult>,
    ) {
        if self.start_failure.is_none() {
            self.start_failure = StartFailure::record(
                self.session,
                self.start_action.id,
                self.native.creator_has_resumed(),
                result,
            );
        }
    }
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
            self.start_failure.as_ref(),
            window,
        ));
        let result = (|| {
            let end = window.deadline();
            self.before_resume_process.as_mut().unwrap().clock =
                Some(RecoveryClock::capture(end).map_err(native_failure)?);
            self.before_resume_process.as_ref().unwrap().check_origin(
                self.session,
                &self.start_action,
                self.start_failure.as_ref(),
                end,
            )?;
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
                .map_err(native_failure)?;
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
            let token = native::current_identity().map_err(native_failure)?;
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
            || prepared
                && !matches!(
                    original.phase,
                    ProcessPhase::Prepared | ProcessPhase::Quiescent
                )
        {
            return Err(recovery_required());
        }
        original.check_origin(
            self.session,
            &self.start_action,
            self.start_failure.as_ref(),
            end,
        )?;
        original
            .clock
            .as_ref()
            .ok_or_else(recovery_required)?
            .check(end)
            .map_err(native_failure)?;
        if original.scope.as_ref() != Some(&self.scope()?)
            || original.token.as_ref() != Some(&native::current_identity().map_err(native_failure)?)
        {
            return Err(recovery_required());
        }
        self.source(end)?;
        let original = self.before_resume_process.as_ref().unwrap();
        if original.phase == ProcessPhase::Quiescent {
            let proof = original.quiescent.as_ref().ok_or_else(recovery_required)?;
            if proof.deadline() != end || original.launcher.is_some() {
                return Err(recovery_required());
            }
            self.native
                .verify_before_resume_quiescent(proof)
                .map_err(native_failure)?;
        } else {
            let proof = original.launcher.as_ref().ok_or_else(recovery_required)?;
            if proof.deadline() != end || original.quiescent.is_some() {
                return Err(recovery_required());
            }
            self.native
                .verify_prepared_before_first_resume(proof)
                .map_err(native_failure)?;
        }
        budget(end)
    }
}
fn native_failure(error: std::io::Error) -> AppError {
    fail(
        "windows024_before_resume_native_unconfirmed",
        &format!(
            "原beforeResume native raw={:?}; kind={:?}; {error}",
            error.raw_os_error(),
            error.kind()
        ),
    )
}
fn require_unoccupied(original: &Option<ProcessAbortAttempt>) -> AppResult<()> {
    if original.is_some() {
        return Err(recovery_required());
    }
    Ok(())
}
#[cfg(test)]
mod tests;
