//! Timely completed native history, never a mutation/lease/typed Closed permit.
//! Only successful actual close driver construction; no serde/Clone or setter.
use super::*;
use std::time::Duration;
pub(super) struct CompletedOriginals {
    completed_at: Instant,
    original_end: Instant,
    current_id: FileIdentity,
    state_digest: String,
}
fn state_digest(journal: &Journal) -> io::Result<String> {
    Ok(hash::sha256_hex(&encoded(&(
        &journal.progress.session,
        &journal.progress.manifest,
        &journal.progress.binding,
        &journal.progress.plan,
        &journal.progress.deadline,
        &journal.progress.epoch,
        &journal.progress.claim,
        journal.progress.sequence,
        &journal.progress.tip,
        &journal.current_raw,
        journal.progress.recovery_summary()?,
        journal.progress.history_summary()?,
        &journal.progress.handle_progress,
    ))?))
}
impl CompletedOriginals {
    pub(in crate::infrastructure::windows_client_originals024::v3::journal) fn capture(
        journal: &Journal,
        end: Instant,
    ) -> io::Result<Self> {
        before(end)?;
        if journal.poisoned
            || end != journal.deadline
            || journal.close_completion.is_some()
            || journal.pending_source_action.is_some()
            || !journal.progress.history_linked()
            || !journal
                .progress
                .handle_progress
                .as_ref()
                .is_some_and(|h| h.released)
            || journal.progress.lease_state != LeaseState::Released
            || !journal
                .progress
                .complete_files(&journal.manifest.storage_plan)
            || !journal
                .original_handoff
                .as_ref()
                .is_some_and(|h| h.handles.completed())
            || journal.released_lease.is_some()
            || !journal.previous_releases.is_empty()
            || journal.retired_lease_histories.is_empty()
        {
            return Err(invalid());
        }
        let digest = state_digest(journal)?;
        let completed_at = Instant::now();
        if completed_at >= end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        Ok(Self {
            completed_at,
            original_end: end,
            current_id: journal.current_id,
            state_digest: digest,
        })
    }
    fn matches(&self, journal: &Journal) -> io::Result<()> {
        if self.completed_at >= self.original_end
            || self.original_end != journal.deadline
            || self.current_id != journal.current_id
            || self.state_digest != state_digest(journal)?
            || journal.poisoned
            || journal.pending_source_action.is_some()
            || journal.released_lease.is_some()
            || !journal.previous_releases.is_empty()
            || !journal
                .progress
                .handle_progress
                .as_ref()
                .is_some_and(|h| h.released)
            || !journal
                .original_handoff
                .as_ref()
                .is_some_and(|h| h.handles.completed())
        {
            return Err(invalid());
        }
        Ok(())
    }
    fn data(&self) -> serde_json::Value {
        serde_json::json!({"completedInOriginalWindow":self.completed_at<self.original_end,"currentVolume":self.current_id.volume,
            "currentFileId":self.current_id.index.to_string(),"stateDigest":self.state_digest,
            "originalDeadlineExpiredNow":Instant::now()>=self.original_end,"nativeAuthority":false,"typedClosed":false})
    }
}
impl Journal {
    fn completed_history_at(&mut self, when: Instant, end: Instant) -> io::Result<()> {
        let result = (|| {
            // Caller owns the single original Status sample window. No now+5
            // deadline construction, old-action deadline assignment or retry.
            if when > Instant::now()
                || when >= end
                || end.duration_since(when) > Duration::from_secs(5)
            {
                return Err(invalid());
            }
            before(end)?;
            self.close_completion
                .as_ref()
                .ok_or_else(invalid)?
                .matches(self)?;
            self.check_records(end)?;
            self.original_handoff
                .as_mut()
                .ok_or_else(invalid)?
                .check_history(end)?;
            for history in &self.retired_lease_histories {
                history.check(end)?;
            }
            self.check_records(end)?;
            self.close_completion
                .as_ref()
                .ok_or_else(invalid)?
                .matches(self)?;
            before(end)
        })();
        if result.is_err() {
            self.poisoned = true;
            if let Some(history) = &mut self.original_handoff {
                history.poison();
            }
        }
        result
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_completed_history_at_for_test(
        &mut self,
        when: Instant,
        end: Instant,
    ) -> io::Result<()> {
        if self.source_action.is_some() {
            return Err(invalid());
        }
        self.completed_history_at(when, end)
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_completion_for_test(
        &self,
    ) -> serde_json::Value {
        self.close_completion
            .as_ref()
            .map(CompletedOriginals::data)
            .unwrap_or(serde_json::Value::Null)
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_expired_append_for_test(
        &mut self,
        end: Instant,
    ) -> io::Result<()> {
        let event = self.progress.handle_event(
            super::super::handles::HandleOperation::OriginalHandlesReleasedVerified,
            None,
        )?;
        self.append_entry_inner(ChainEntry::Handles(event), end)
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_drift_completion_context_for_test(
        &mut self,
    ) {
        self.progress.binding = "e".repeat(64);
    }
}
