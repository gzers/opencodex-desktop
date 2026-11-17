//! Timely completed native history, never a mutation/lease/typed Closed permit.
//! Only successful actual close driver construction; no serde/Clone or setter.
use super::*;
use std::time::Duration;
pub(super) struct CompletedOriginals {
    completed_at: Instant,
    original_end: Instant,
    current_id: FileIdentity,
    state_digest: String,
    sequence: u32,
    event_digest: String,
    current_digest: String,
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
            sequence: journal.progress.sequence,
            event_digest: journal.progress.tip.clone(),
            current_digest: hash::sha256_hex(&journal.current_raw),
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
    pub(super) fn original_close_data(
        &mut self,
        route_close_digest: String,
        end: Instant,
    ) -> io::Result<super::super::closed::ClosedData> {
        self.check(end)?;
        let proof = self.close_completion.as_ref().ok_or_else(invalid)?;
        proof.matches(self)?;
        let handles = self.progress.handle_progress.as_ref().ok_or_else(invalid)?;
        let content = self
            .progress
            .content
            .as_ref()
            .filter(|c| c.verified)
            .ok_or_else(invalid)?;
        Ok(super::super::closed::ClosedData {
            route_close_digest,
            close_sequence: proof.sequence,
            close_event_digest: proof.event_digest.clone(),
            close_current_digest: proof.current_digest.clone(),
            native_close_digest: proof.state_digest.clone(),
            content_digest: content.digest.clone(),
            original_history_digest: hash::sha256_hex(&encoded(
                &self.progress.history_summary()?.ok_or_else(invalid)?,
            )?),
            inventory_digest: handles.inventory_digest.clone(),
            handle_count: handles.count,
            ready: self.progress.ready_generation.clone().ok_or_else(invalid)?,
        })
    }
    pub(super) fn check_close_lineage(&self) -> io::Result<()> {
        let original = self.close_completion.as_ref().ok_or_else(invalid)?;
        let closed = self.progress.closed.as_ref().ok_or_else(invalid)?;
        if original.completed_at >= original.original_end
            || original.original_end != self.deadline
            || original.current_id != self.current_id
            || closed.close_sequence != original.sequence
            || closed.close_event_digest != original.event_digest
            || closed.native_close_digest != original.state_digest
            || self.progress.handle_close.as_ref()
                != Some(&(original.sequence, original.event_digest.clone()))
        {
            return Err(invalid());
        }
        let record = self
            .events
            .get(original.sequence.checked_sub(1).ok_or_else(invalid)? as usize)
            .ok_or_else(invalid)?;
        if hash::sha256_hex(&record.raw) != original.event_digest
            || !matches!(record.entry.as_ref(),Some(ChainEntry::Handles(e)) if e.operation==super::super::handles::HandleOperation::OriginalHandlesReleasedVerified)
        {
            return Err(invalid());
        }
        let data: super::super::closed::ClosedData = match self
            .events
            .get(original.sequence as usize)
            .and_then(|r| r.entry.as_ref())
        {
            Some(ChainEntry::Closed(e))
                if e.operation == super::super::closed::ClosedOperation::StopClosureIntent =>
            {
                serde_json::from_slice(&encoded(&e.closed)?).map_err(|_| invalid())?
            }
            _ => return Err(invalid()),
        };
        if data.close_current_digest != original.current_digest
            || data.native_close_digest != original.state_digest
            || data.digest()? != closed.digest
        {
            return Err(invalid());
        }
        if closed.verified
            && self.progress.route_close.as_ref()
                != Some(&super::super::route_close::RouteCloseSummary {
                    digest: closed.route_close_digest.clone(),
                    verified: true,
                })
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub(in crate::infrastructure::windows_client_originals024) fn check_route_closed_history(
        &self,
        route: &super::super::super::v2::routes::RetainedRouteClose,
        end: Instant,
    ) -> io::Result<()> {
        route.check_history(end)?;
        let data: super::super::route_close::RouteCloseData =
            serde_json::from_value(route.canonical_data()?).map_err(|_| invalid())?;
        if self.progress.route_close.as_ref()
            != Some(&super::super::route_close::RouteCloseSummary {
                digest: data.digest()?,
                verified: true,
            })
        {
            return Err(invalid());
        }
        before(end)
    }
    pub(super) fn completed_stop_boundary(&mut self, end: Instant) -> io::Result<()> {
        self.check(end)?;
        self.check_close_lineage()?;
        self.original_handoff
            .as_mut()
            .ok_or_else(invalid)?
            .check_originals(end)?;
        for history in &self.retired_lease_histories {
            history.check(end)?;
        }
        self.check(end)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn completed_stop_history_at(
        &mut self,
        when: Instant,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            let read = super::super::read_window::HistoryReadWindow::status(when, end)?;
            self.completed_stop_history_read(&read)
        })();
        if result.is_err() {
            self.poison_segment();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn completed_stop_history_for_action(
        &mut self,
        window: &crate::modules::runtime::official_lifecycle024::ActionWindow,
    ) -> io::Result<()> {
        let result = (|| {
            let read = super::super::read_window::HistoryReadWindow::action(window)?;
            self.completed_stop_history_read(&read)
        })();
        if result.is_err() {
            self.poison_segment();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024::v3) fn completed_stop_history_read(
        &mut self,
        read: &super::super::read_window::HistoryReadWindow,
    ) -> io::Result<()> {
        let result = (|| {
            let end = read.end()?;
            if read.requires_original_end() && end != self.deadline {
                return Err(invalid());
            }
            before(end)?;
            self.stop_completion
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
            self.stop_completion
                .as_ref()
                .ok_or_else(invalid)?
                .matches(self)?;
            read.check()
        })();
        if result.is_err() {
            self.poison_segment();
        }
        result
    }
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
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_corrupt_completed_current_for_test(
        &mut self,
    ) -> io::Result<()> {
        self.current.seek(SeekFrom::Start(0))?;
        self.current.write_all(b"{")?;
        self.current.set_len(1)?;
        self.current.sync_all()
    }
}
/// Final native current seal with the original close receipt retained as a
/// parent. Constructed only after actual typed exit/supervisor history guards
/// and both canonical closure events. It grants no mutation or public Start.
pub(super) struct CompletedStop {
    completed_at: Instant,
    original_end: Instant,
    current_id: FileIdentity,
    state_digest: String,
}
impl CompletedStop {
    pub(super) fn capture(
        journal: &mut Journal,
        guard: &mut super::closed_stop::CompletedStopGuard<'_>,
        end: Instant,
    ) -> io::Result<Self> {
        guard.check(journal, end)?;
        before(end)?;
        if journal.poisoned
            || end != journal.deadline
            || journal.stop_completion.is_some()
            || !journal.progress.closed.as_ref().is_some_and(|c| c.verified)
            || journal.released_lease.is_some()
            || !journal.previous_releases.is_empty()
            || !journal
                .original_handoff
                .as_ref()
                .is_some_and(|h| h.handles.completed())
        {
            return Err(invalid());
        }
        journal.check_close_lineage()?;
        let state_digest = state_digest(journal)?;
        guard.check(journal, end)?;
        let completed_at = Instant::now();
        before(end)?;
        Ok(Self {
            completed_at,
            original_end: end,
            current_id: journal.current_id,
            state_digest,
        })
    }
    fn matches(&self, journal: &Journal) -> io::Result<()> {
        if self.completed_at >= self.original_end
            || self.original_end != journal.deadline
            || self.current_id != journal.current_id
            || self.state_digest != state_digest(journal)?
            || journal.poisoned
            || !journal.progress.closed.as_ref().is_some_and(|c| c.verified)
            || !journal
                .original_handoff
                .as_ref()
                .is_some_and(|h| h.handles.completed())
        {
            return Err(invalid());
        }
        journal.check_close_lineage()
    }
}
