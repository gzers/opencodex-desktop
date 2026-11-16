//! Immutable original-byte history joined to the SAME canonical chain.
//! Context locators are data; only actual retained tree/receipt witnesses may
//! complete native history. No Closed or original-handle-release factory here.
use super::super::super::tree::RecoveryWitness;
use super::super::history::{HistoryContextData, HistoryOperation, HistoryReference};
use super::*;
pub(in crate::infrastructure::windows_client_originals024) struct HistoryContext {
    data: HistoryContextData,
    profile: PathBuf,
}
impl HistoryContext {
    pub(in crate::infrastructure::windows_client_originals024) fn session(&self) -> &str {
        &self.data.session_id
    }
    pub(in crate::infrastructure::windows_client_originals024) fn metadata(
        &self,
    ) -> serde_json::Value {
        // Infallible struct of primitive data; callers may not replace context.
        serde_json::json!(self.data)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn matches(
        &self,
        root: &Path,
        frame: &str,
    ) -> io::Result<()> {
        let index = if root == self.profile.join(".codex") {
            0
        } else if root == self.profile.join(".grok") {
            1
        } else {
            return Err(invalid());
        };
        if self.data.original_frames[index] != frame {
            return Err(invalid());
        }
        Ok(())
    }
}
impl Journal {
    fn original_history_boundary(&mut self, end: Instant) -> io::Result<()> {
        before(end)?;
        if self.poisoned || end > self.deadline {
            return Err(invalid());
        }
        if self.progress.lease_state != LeaseState::Released
            || !self.progress.complete_files(&self.manifest.storage_plan)
        {
            return Err(invalid());
        }
        self.released_lease
            .as_mut()
            .ok_or_else(invalid)?
            .check(end)?;
        // Receipt checks read only retained objects. One full journal-native
        // observation after it covers this boundary; native operations have
        // their own full BOTH-tree before/after and append checks BEFORE/AFTER.
        self.check(end)
    }
    fn seal_original_history(
        &mut self,
        mut witness: RecoveryWitness<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.original_history_boundary(end)?;
            let data = self.progress.history_context(&self.manifest.storage_plan)?;
            let frames = witness.original_frames(end)?;
            if frames
                != self
                    .progress
                    .original_history_frames(&self.manifest.storage_plan)?
            {
                return Err(invalid());
            }
            let context = HistoryContext {
                data,
                profile: self.pins[0].path().into(),
            };
            let event = self.progress.history_event(
                HistoryOperation::OriginalHistoryIntent,
                &context.data,
                None,
            )?;
            self.append_entry_inner(ChainEntry::History(event), end)?;
            self.original_history_boundary(end)?;
            #[cfg(test)]
            if self.fault == 70 {
                return Err(io::Error::other(
                    "injected after durable original-history intent/current before archive copies",
                ));
            }
            #[cfg(test)]
            let fault = self.fault;
            #[cfg(not(test))]
            let fault = 0;
            let refs = witness.archive_originals(&context, fault, end)?;
            self.original_history_boundary(end)?;
            #[cfg(test)]
            {
                if self.fault == 73 {
                    return Err(io::Error::other("injected after BOTH complete original-history archives before linked result"));
                }
                if self.fault == 74 {
                    self.fault = 1;
                } else if self.fault == 75 {
                    self.fault = 2;
                }
            }
            let mut parsed = Vec::new();
            for value in refs {
                parsed.push(HistoryReference::from_native_data(value)?);
            }
            let refs = parsed.try_into().map_err(|_| invalid())?;
            let event = self.progress.history_event(
                HistoryOperation::OriginalHistoryLinkedVerified,
                &context.data,
                Some(refs),
            )?;
            self.append_entry_inner(ChainEntry::History(event), end)?;
            witness.history_references(end)?;
            self.original_history_boundary(end)
        })();
        if result.is_err() {
            self.poisoned = true;
            witness.poison();
        }
        result
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_original_history_for_test(
        &mut self,
        witness: RecoveryWitness<'_>,
        end: Instant,
    ) -> io::Result<()> {
        if self.source_action.is_some() || end != self.deadline {
            return Err(invalid());
        }
        self.seal_original_history(witness, end)
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_history_trace_for_test(
        &self,
    ) -> io::Result<serde_json::Value> {
        let mut base = self.component_final_release_trace_for_test()?;
        base["historySummary"] =
            serde_json::to_value(self.progress.history_summary()?).map_err(|_| invalid())?;
        let mut events = Vec::new();
        for record in &self.events {
            if let ChainEntry::History(e) =
                serde_json::from_slice::<ChainEntry>(&record.raw).map_err(|_| invalid())?
            {
                events.push(serde_json::to_value(e).map_err(|_| invalid())?);
            }
        }
        base["immutableHistoryEvents"] = serde_json::json!(events);
        Ok(base)
    }
}
