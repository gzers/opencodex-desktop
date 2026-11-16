//! Canonical completed Stop locators. DATA never creates actual completion,
//! process/supervisor history, Closed eligibility or a new mutation deadline.
use super::*;
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum ClosedOperation {
    StopClosureIntent,
    StopClosedVerified,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct ClosedData {
    pub(super) close_sequence: u32,
    pub(super) close_event_digest: String,
    pub(super) close_current_digest: String,
    pub(super) native_close_digest: String,
    pub(super) content_digest: String,
    pub(super) original_history_digest: String,
    pub(super) inventory_digest: String,
    pub(super) handle_count: usize,
    pub(super) ready: (u32, u16, String),
}
impl ClosedData {
    pub(super) fn digest(&self) -> io::Result<String> {
        Ok(hash::sha256_hex(&encoded(self)?))
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct ClosedSummary {
    pub(super) digest: String,
    pub(super) verified: bool,
    pub(super) close_sequence: u32,
    pub(super) close_event_digest: String,
    pub(super) native_close_digest: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct ClosedEvent {
    pub(super) schema: String,
    pub(super) document_kind: String,
    pub(super) storage_strategy: String,
    pub(super) event_id: String,
    pub(super) sequence: u32,
    pub(super) session_id: String,
    pub(super) operation: ClosedOperation,
    pub(super) closed: ClosedData,
    pub(super) manifest_digest: String,
    pub(super) binding_digest: String,
    pub(super) storage_plan_digest: String,
    pub(super) security_policy_digest: String,
    pub(super) deadline_scope: String,
    pub(super) lease_epoch: String,
    pub(super) claim_key: String,
    pub(super) previous_event_digest: String,
}
impl Progress {
    pub(super) fn apply_closed(&mut self, plan: &StoragePlan, e: &ClosedEvent) -> io::Result<()> {
        let handles = self.handle_progress.as_ref().ok_or_else(invalid)?;
        let content = self.content.as_ref().ok_or_else(invalid)?;
        if self.batch.is_some() || self.pending.is_some() || self.snapshot_pending.is_some() || self.action_pending.is_some()
            || !self.complete_files(plan) || !self.history_linked() || !handles.released || !content.verified
            || self.lifecycle!=snapshots::LifecyclePhase::Stopped || self.lease_state!=segments::LeaseState::Released
            || !matches!(self.action_kind,actions::ActionKind::Stop|actions::ActionKind::Restart)
            || e.schema!=crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3
            || e.document_kind!="completed-stop-event" || e.storage_strategy!=ANCHORED_STORAGE_STRATEGY
            || e.sequence!=self.sequence.checked_add(1).ok_or_else(invalid)? || e.session_id!=self.session
            || !uuid(&e.event_id) || self.event_ids.contains(&e.event_id) || e.previous_event_digest!=self.tip
            || e.manifest_digest!=self.manifest || e.binding_digest!=self.binding || e.storage_plan_digest!=self.plan || plan.digest()?!=self.plan
            || e.security_policy_digest!=Policy::fixed().digest()? || e.deadline_scope!=self.deadline || e.lease_epoch!=self.epoch || e.claim_key!=self.claim
            || e.closed.content_digest!=content.digest || e.closed.inventory_digest!=handles.inventory_digest || e.closed.handle_count!=handles.count
            || self.ready_generation.as_ref()!=Some(&e.closed.ready)
            || e.closed.original_history_digest!=hash::sha256_hex(&encoded(&self.history_summary()?.ok_or_else(invalid)?)?)
            || !digest(&e.closed.close_event_digest) || !digest(&e.closed.close_current_digest) || !digest(&e.closed.native_close_digest)
            || self.handle_close.as_ref()!=Some(&(e.closed.close_sequence,e.closed.close_event_digest.clone()))
        {return Err(invalid());}
        let value = e.closed.digest()?;
        match e.operation {
            ClosedOperation::StopClosureIntent
                if self.closed.is_none()
                    && e.closed.close_sequence == self.sequence
                    && e.closed.close_event_digest == self.tip => {}
            ClosedOperation::StopClosedVerified
                if self.closed.as_ref().is_some_and(|c| {
                    !c.verified
                        && c.digest == value
                        && c.close_sequence == e.closed.close_sequence
                        && c.close_event_digest == e.closed.close_event_digest
                        && c.native_close_digest == e.closed.native_close_digest
                }) => {}
            _ => return Err(invalid()),
        }
        let tip = hash::sha256_hex(&encoded(e)?);
        self.closed = Some(ClosedSummary {
            digest: value,
            verified: e.operation == ClosedOperation::StopClosedVerified,
            close_sequence: e.closed.close_sequence,
            close_event_digest: e.closed.close_event_digest.clone(),
            native_close_digest: e.closed.native_close_digest.clone(),
        });
        self.tip = tip;
        self.sequence = e.sequence;
        self.event_ids.insert(e.event_id.clone());
        Ok(())
    }
    pub(super) fn closed_event(
        &self,
        operation: ClosedOperation,
        closed: ClosedData,
    ) -> io::Result<ClosedEvent> {
        Ok(ClosedEvent {
            schema:
                crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3
                    .into(),
            document_kind: "completed-stop-event".into(),
            storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
            event_id: uuid::Uuid::new_v4().to_string(),
            sequence: self.sequence.checked_add(1).ok_or_else(invalid)?,
            session_id: self.session.clone(),
            operation,
            closed,
            manifest_digest: self.manifest.clone(),
            binding_digest: self.binding.clone(),
            storage_plan_digest: self.plan.clone(),
            security_policy_digest: Policy::fixed().digest()?,
            deadline_scope: self.deadline.clone(),
            lease_epoch: self.epoch.clone(),
            claim_key: self.claim.clone(),
            previous_event_digest: self.tip.clone(),
        })
    }
}
#[cfg(test)]
#[path = "closed_data_tests.rs"]
mod tests;
