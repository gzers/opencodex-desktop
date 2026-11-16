//! Explicit own-file close after sealed native original history. The retained
//! ownership batch, not persisted numbers, is the only native close source.
use super::super::super::{
    handle_release::HandleRow,
    tree::{AnchoredTree, OriginalHandleHandoff},
};
use super::super::handles::HandleOperation;
use super::*;
impl Journal {
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_check_handoff_history_for_test(
        &mut self,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.check(end)?;
            if !self
                .progress
                .handle_progress
                .as_ref()
                .is_some_and(|h| h.released)
            {
                return Err(invalid());
            }
            for receipt in &self.retired_lease_histories {
                receipt.check(end)?;
            }
            self.original_handoff
                .as_mut()
                .ok_or_else(invalid)?
                .check_history(end)?;
            self.check(end)
        })();
        if result.is_err() {
            self.poisoned = true;
            if let Some(h) = &mut self.original_handoff {
                h.poison();
            }
        }
        result
    }
    fn close_original_handles(&mut self, end: Instant) -> io::Result<()> {
        let result = (|| {
            before(end)?;
            if self.poisoned || end > self.deadline {
                return Err(invalid());
            }
            if !self.progress.history_linked()
                || self.progress.handle_progress.is_some()
                || self.original_handoff.is_none()
            {
                return Err(invalid());
            }
            self.released_lease
                .as_mut()
                .ok_or_else(invalid)?
                .check(end)?;
            for prior in &mut self.previous_releases {
                prior.readback_history(end)?;
            }
            let refs = self
                .original_handoff
                .as_mut()
                .ok_or_else(invalid)?
                .pending_history_references(end)?;
            self.progress
                .verify_original_history_references(&self.manifest.storage_plan, refs)?;
            self.original_handoff
                .as_mut()
                .ok_or_else(invalid)?
                .detach(end)?;
            let batch = &mut self.original_handoff.as_mut().ok_or_else(invalid)?.handles;
            for (i, lease) in self.previous_releases.drain(..).enumerate() {
                self.retired_lease_histories
                    .push(lease.transfer_original_files(batch, &format!("lease/old/{i:04}")));
            }
            let lease = self.released_lease.take().ok_or_else(invalid)?;
            self.retired_lease_histories
                .push(lease.transfer_original_files(batch, "lease/final"));
            batch.check_held(end)?;
            let inventory = batch.inventory()?;
            let rows: Vec<HandleRow> =
                serde_json::from_value(inventory["rows"].clone()).map_err(|_| invalid())?;
            let event = self
                .progress
                .handle_event(HandleOperation::OriginalHandlesReleaseIntent, Some(rows))?;
            self.append_entry_inner(ChainEntry::Handles(event), end)?;
            self.original_handoff
                .as_mut()
                .ok_or_else(invalid)?
                .check_originals(end)?;
            self.original_handoff
                .as_ref()
                .ok_or_else(invalid)?
                .handles
                .check_held(end)?;
            #[cfg(test)]
            if self.fault == 80 {
                return Err(io::Error::other(
                    "injected after durable exact handle-close intent/current before native closes",
                ));
            }
            #[cfg(test)]
            if self.fault == 85 {
                self.original_handoff
                    .as_ref()
                    .ok_or_else(invalid)?
                    .handles
                    .protect_first_for_test()?;
            }
            #[cfg(test)]
            let fault = self.fault;
            #[cfg(not(test))]
            let fault = 0;
            self.original_handoff
                .as_mut()
                .ok_or_else(invalid)?
                .handles
                .close_all(fault, end)?;
            self.original_handoff
                .as_mut()
                .ok_or_else(invalid)?
                .check_originals(end)?;
            if !self
                .original_handoff
                .as_ref()
                .ok_or_else(invalid)?
                .handles
                .completed()
            {
                return Err(invalid());
            }
            #[cfg(test)]
            {
                if self.fault == 82 {
                    return Err(io::Error::other("injected after all actual original handles closed and native readback before result"));
                }
                if self.fault == 83 {
                    self.fault = 1;
                } else if self.fault == 84 {
                    self.fault = 2;
                }
            }
            let event = self
                .progress
                .handle_event(HandleOperation::OriginalHandlesReleasedVerified, None)?;
            self.append_entry_inner(ChainEntry::Handles(event), end)?;
            self.original_handoff
                .as_mut()
                .ok_or_else(invalid)?
                .check_originals(end)?;
            self.check(end)?;
            #[cfg(test)]
            if self.fault == 87 {
                while Instant::now() < end {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            }
            self.close_completion = Some(super::completed::CompletedOriginals::capture(self, end)?);
            Ok(())
        })();
        if result.is_err() {
            self.poisoned = true;
            if let Some(state) = &mut self.original_handoff {
                state.poison();
            }
        }
        result
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_close_original_handles_for_test(
        &mut self,
        trees: [AnchoredTree; 2],
        end: Instant,
    ) -> io::Result<()> {
        // Called exactly once from each isolated native component. Install
        // owned originals before any fallible check or native operation.
        if self.original_handoff.is_some() {
            return Err(invalid());
        }
        self.original_handoff = Some(OriginalHandleHandoff::retain(trees, end));
        if self.source_action.is_some() || end != self.deadline {
            self.poisoned = true;
            return Err(invalid());
        }
        self.close_original_handles(end)
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_handle_trace_for_test(
        &self,
    ) -> io::Result<serde_json::Value> {
        let mut trace = self.component_history_trace_for_test()?;
        trace["originalHandleSummary"] =
            serde_json::to_value(&self.progress.handle_progress).map_err(|_| invalid())?;
        trace["originalNativeHandoff"] = self
            .original_handoff
            .as_ref()
            .map(|h| h.trace())
            .unwrap_or(serde_json::Value::Null);
        trace["nativeLeaseCompletionHistories"] = serde_json::json!(self
            .retired_lease_histories
            .iter()
            .map(|h| h.data())
            .collect::<Vec<_>>());
        let mut events = Vec::new();
        for r in &self.events {
            if let ChainEntry::Handles(e) =
                serde_json::from_slice::<ChainEntry>(&r.raw).map_err(|_| invalid())?
            {
                events.push(serde_json::to_value(e).map_err(|_| invalid())?);
            }
        }
        trace["immutableHandleEvents"] = serde_json::json!(events);
        Ok(trace)
    }
}
