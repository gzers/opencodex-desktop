//! Fresh STOP authority from original Owner/Source; no setter for deadlines.
//! Old record reads are permitted under the new native receipt, while old lease
//! mutation permission stays expired. Only bound event/current activates scope.
use super::super::super::source::Source;
use super::super::actions::{ActionEvent, ActionKind, ActionOperation};
use super::super::snapshots::LifecyclePhase;
use super::*;
use crate::{
    infrastructure::windows_official_lifecycle024::{OriginalActionKind, Owner},
    modules::runtime::official_lifecycle024::ServiceObservation,
};
impl Journal {
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_stop_scope_for_test(
        &mut self,
        end: Instant,
    ) -> io::Result<()> {
        if self.source_action.is_some() {
            return Err(invalid());
        }
        let scope = uuid::Uuid::new_v4().to_string();
        for operation in [
            ActionOperation::StopActionIntent,
            ActionOperation::StopActionBoundVerified,
        ] {
            let e = self.action_event(operation, ActionKind::Stop, scope.clone())?;
            self.append_entry_inner(ChainEntry::Action(e), end)?;
        }
        // Synthetic DATA only, absolutely no new deadline or native receipt.
        // self.deadline remains the exact original fixture cutoff.
        self.check(end)
    }
    fn action_event(
        &self,
        operation: ActionOperation,
        kind: ActionKind,
        next: String,
    ) -> io::Result<ActionEvent> {
        Ok(ActionEvent {
            schema: ANCHORED_ORIGINALS_SCHEMA_V3.into(),
            document_kind: "original-action-event".into(),
            storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
            event_id: uuid::Uuid::new_v4().to_string(),
            sequence: self.progress.sequence.checked_add(1).ok_or_else(invalid)?,
            session_id: self.progress.session.clone(),
            operation,
            next_action_scope: next,
            action_kind: kind,
            ready: self.progress.ready_generation.clone().ok_or_else(invalid)?,
            lease_objects_digest: self.progress.lease_objects.clone().ok_or_else(invalid)?,
            lease_epoch: self.progress.epoch.clone(),
            claim_key: self.progress.claim.clone(),
            deadline_scope: self.progress.deadline.clone(),
            previous_event_digest: self.progress.tip.clone(),
            storage_plan_digest: self.progress.plan.clone(),
            security_policy_digest: Policy::fixed().digest()?,
            manifest_digest: self.progress.manifest.clone(),
            binding_digest: self.progress.binding.clone(),
        })
    }
    pub(in crate::infrastructure::windows_client_originals024) fn bind_original_stop_action(
        &mut self,
        source: &mut Source,
        owner: &mut Owner,
        ready: &ServiceObservation,
        kind: OriginalActionKind,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            if self.poisoned
                || self.pending_source_action.is_some()
                || self.progress.lifecycle != LifecyclePhase::Ready
                || self.progress.action_pending.is_some()
                || self.progress.lease_state != LeaseState::Released
                || self.progress.snapshot_pending.is_some()
                || self.progress.work_mode != super::super::observations::WorkMode::Open
            {
                return Err(invalid());
            }
            let action_kind = match kind {
                OriginalActionKind::Stop => ActionKind::Stop,
                OriginalActionKind::Restart => ActionKind::Restart,
                _ => return Err(invalid()),
            };
            source.check(owner, end).map_err(io::Error::other)?;
            owner
                .originals_check_ready(ready, end)
                .map_err(io::Error::other)?;
            if self.progress.ready_generation
                != Some((ready.pid(), ready.port(), ready.record_sha256().into()))
                || source.binding_raw != encoded(&self.manifest.binding)?
            {
                return Err(invalid());
            }
            let context = source
                .journal_context(owner, kind, end)
                .map_err(io::Error::other)?;
            let (pins, peer, binding, action) = context.into_parts();
            binding.check_native(&peer, &pins)?;
            if encoded(&binding)? != encoded(&self.manifest.binding)?
                || action.deadline() != end
                || !action.same_original_owner(self.source_action.as_ref().ok_or_else(invalid)?)
            {
                return Err(invalid());
            }
            action.check(owner, end).map_err(io::Error::other)?;
            self.released_lease
                .as_mut()
                .ok_or_else(invalid)?
                .readback_history(end)?;
            let next = action.scope_id();
            // No client mutation/new lease is possible in this private pending
            // interval. Keep actual new authority before fallible original-data
            // readback/event IO; poison retains both old and new receipts.
            self.pending_source_action = Some(action);
            self.check(end)?;
            let e =
                self.action_event(ActionOperation::StopActionIntent, action_kind, next.clone())?;
            self.append_entry_inner(ChainEntry::Action(e), end)?;
            #[cfg(test)]
            if self.fault == 21 {
                return Err(io::Error::other(
                    "injected after original stop-action intent/current before binding result",
                ));
            }
            self.pending_source_action
                .as_ref()
                .ok_or_else(invalid)?
                .check(owner, end)
                .map_err(io::Error::other)?;
            source.check(owner, end).map_err(io::Error::other)?;
            owner
                .originals_check_ready(ready, end)
                .map_err(io::Error::other)?;
            self.released_lease
                .as_mut()
                .ok_or_else(invalid)?
                .readback_history(end)?;
            #[cfg(test)]
            {
                if self.fault == 22 {
                    self.fault = 1;
                } else if self.fault == 23 {
                    self.fault = 2;
                }
            }
            let e =
                self.action_event(ActionOperation::StopActionBoundVerified, action_kind, next)?;
            self.append_entry_inner(ChainEntry::Action(e), end)?;
            let old = self.source_action.take().ok_or_else(invalid)?;
            self.old_source_actions.push(old);
            self.source_action = self.pending_source_action.take();
            self.deadline = end;
            self.check_source_owner(owner, end)?;
            source.check(owner, end).map_err(io::Error::other)?;
            owner
                .originals_check_ready(ready, end)
                .map_err(io::Error::other)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
}
