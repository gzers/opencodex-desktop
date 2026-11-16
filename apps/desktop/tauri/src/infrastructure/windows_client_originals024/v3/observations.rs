//! Canonical work-handle interval data. Changed observations stay frozen and
//! cannot become native restoration authority from a digest or parsed record.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum WorkMode {
    Sealed,
    Opening,
    Open,
    Sealing,
    Observed,
    Resuming,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum WorkOperation {
    WorkOpenIntent,
    WorkOpenedVerified,
    WorkSealIntent,
    WorkSealedVerified,
    WorkUnchangedVerified,
    WorkResumeIntent,
    WorkResumedVerified,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct WorkEvent {
    pub(super) schema: String,
    pub(super) document_kind: String,
    pub(super) storage_strategy: String,
    pub(super) event_id: String,
    pub(super) sequence: u32,
    pub(super) session_id: String,
    pub(super) operation: WorkOperation,
    pub(super) frame_digests: Option<[String; 2]>,
    pub(super) lease_epoch: String,
    pub(super) claim_key: String,
    pub(super) deadline_scope: String,
    pub(super) previous_event_digest: String,
    pub(super) storage_plan_digest: String,
    pub(super) security_policy_digest: String,
    pub(super) manifest_digest: String,
    pub(super) binding_digest: String,
}
impl Progress {
    pub(super) fn apply_work(&mut self, plan: &StoragePlan, e: &WorkEvent) -> io::Result<()> {
        if self.recovery.is_some() {
            return Err(invalid());
        }
        if e.schema
            != crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3
            || e.document_kind != "work-interval-event"
            || e.storage_strategy != ANCHORED_STORAGE_STRATEGY
            || e.sequence != self.sequence.checked_add(1).ok_or_else(invalid)?
            || e.session_id != self.session
            || !uuid(&e.event_id)
            || self.event_ids.contains(&e.event_id)
            || e.previous_event_digest != self.tip
            || e.storage_plan_digest != self.plan
            || plan.digest()? != self.plan
            || e.security_policy_digest != Policy::fixed().digest()?
            || e.manifest_digest != self.manifest
            || e.binding_digest != self.binding
            || e.lease_epoch != self.epoch
            || e.claim_key != self.claim
            || e.deadline_scope != self.deadline
            || self.lease_state != segments::LeaseState::Active
            || self.pending.is_some()
            || self.snapshot_pending.is_some()
            || self.action_pending.is_some()
            || e.frame_digests
                .as_ref()
                .is_some_and(|a| a.iter().any(|s| !digest(s)))
        {
            return Err(invalid());
        }
        if !plan
            .roots
            .iter()
            .flat_map(|r| &r.nodes)
            .all(|n| self.nodes[&n.node_id].0 == if n.kind == Kind::CustodiedFile { 4 } else { 1 })
        {
            return Err(invalid());
        }
        let mode = match (self.work_mode, e.operation) {
            (WorkMode::Sealed, WorkOperation::WorkOpenIntent) if e.frame_digests.is_some() => {
                WorkMode::Opening
            }
            (WorkMode::Opening, WorkOperation::WorkOpenedVerified)
                if e.frame_digests == self.work_frames =>
            {
                WorkMode::Open
            }
            (WorkMode::Open, WorkOperation::WorkSealIntent) if e.frame_digests.is_none() => {
                WorkMode::Sealing
            }
            (WorkMode::Sealing, WorkOperation::WorkSealedVerified) if e.frame_digests.is_some() => {
                WorkMode::Observed
            }
            (WorkMode::Observed, WorkOperation::WorkUnchangedVerified)
                if e.frame_digests == self.work_frames
                    && e.frame_digests == self.observed_frames =>
            {
                WorkMode::Sealed
            }
            (WorkMode::Observed, WorkOperation::WorkResumeIntent)
                if e.frame_digests == self.observed_frames
                    && self.linked_frames.get(&self.observed_sequence)
                        == e.frame_digests.as_ref()
                    && self.lifecycle != snapshots::LifecyclePhase::Stopped =>
            {
                WorkMode::Resuming
            }
            (WorkMode::Resuming, WorkOperation::WorkResumedVerified)
                if e.frame_digests == self.observed_frames =>
            {
                WorkMode::Open
            }
            _ => return Err(invalid()),
        };
        if e.operation == WorkOperation::WorkOpenIntent {
            self.work_frames = e.frame_digests.clone();
            self.observed_frames = None;
        }
        if e.operation == WorkOperation::WorkSealedVerified {
            self.observed_frames = e.frame_digests.clone();
            self.observed_sequence = e.sequence;
        }
        self.work_mode = mode;
        self.tip = hash::sha256_hex(&encoded(e)?);
        self.sequence = e.sequence;
        self.event_ids.insert(e.event_id.clone());
        Ok(())
    }
}
#[cfg(test)]
#[path = "observation_data_tests.rs"]
mod tests;
