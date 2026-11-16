//! Strict canonical lease transition data in the SAME node phase chain.
//! Parsing/sequence validation never substitutes a live native release receipt.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum LeaseState {
    Active,
    ReleasePrepared,
    Released,
    AcquirePrepared,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum LeaseOperation {
    LeaseReleasePrepared,
    LeaseReleasedVerified,
    LeaseAcquirePrepared,
    LeaseAcquiredVerified,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct LeaseEvent {
    pub(super) schema: String,
    pub(super) document_kind: String,
    pub(super) storage_strategy: String,
    pub(super) event_id: String,
    pub(super) sequence: u32,
    pub(super) session_id: String,
    pub(super) operation: LeaseOperation,
    pub(super) lease_epoch: String,
    pub(super) next_epoch: Option<String>,
    pub(super) claim_key: String,
    pub(super) objects_digest: String,
    pub(super) deadline_scope: String,
    pub(super) previous_event_digest: String,
    pub(super) storage_plan_digest: String,
    pub(super) security_policy_digest: String,
    pub(super) manifest_digest: String,
    pub(super) binding_digest: String,
}
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum ChainEntry {
    Node(Event),
    Lease(LeaseEvent),
    Work(super::observations::WorkEvent),
    Snapshot(super::snapshots::SnapshotEvent),
    Action(super::actions::ActionEvent),
    Recovery(super::recovery::RecoveryEvent),
    History(super::history::HistoryEvent),
    Handles(super::handles::HandleEvent),
}
impl Progress {
    pub(super) fn apply_entry(&mut self, plan: &StoragePlan, e: &ChainEntry) -> io::Result<()> {
        if self.history.is_some() && !matches!(e, ChainEntry::History(_) | ChainEntry::Handles(_)) {
            return Err(invalid());
        }
        match e {
            ChainEntry::Node(n) => self.apply(plan, n),
            ChainEntry::Lease(e) => self.apply_lease(plan, e),
            ChainEntry::Work(e) => self.apply_work(plan, e),
            ChainEntry::Snapshot(e) => self.apply_snapshot(plan, e),
            ChainEntry::Action(e) => self.apply_action(plan, e),
            ChainEntry::Recovery(e) => self.apply_recovery(plan, e),
            ChainEntry::History(e) => self.apply_history(plan, e),
            ChainEntry::Handles(e) => self.apply_handles(plan, e),
        }
    }
    pub(super) fn apply_lease(&mut self, plan: &StoragePlan, e: &LeaseEvent) -> io::Result<()> {
        // Final recovery admits ONLY its original prepared/result release pair;
        // never a new acquisition or an incomplete per-node aggregate.
        let recovery_release = self.recovery.is_some();
        if recovery_release
            && (!self.recovery_tree_ready_for_release(plan)
                || !matches!(
                    e.operation,
                    LeaseOperation::LeaseReleasePrepared | LeaseOperation::LeaseReleasedVerified
                ))
        {
            return Err(invalid());
        }
        if recovery_release
            && !plan
                .roots
                .iter()
                .flat_map(|r| &r.nodes)
                .filter(|n| n.kind != Kind::CustodiedFile)
                .all(|n| {
                    self.nodes[&n.node_id].0
                        == if e.operation == LeaseOperation::LeaseReleasePrepared {
                            1
                        } else {
                            2
                        }
                })
        {
            return Err(invalid());
        }
        use crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3;
        if e.schema != ANCHORED_ORIGINALS_SCHEMA_V3
            || e.document_kind != "lease-event"
            || e.storage_strategy != ANCHORED_STORAGE_STRATEGY
            || e.sequence != self.sequence.checked_add(1).ok_or_else(invalid)?
            || e.session_id != self.session
            || e.previous_event_digest != self.tip
            || !uuid(&e.event_id)
            || self.event_ids.contains(&e.event_id)
            || e.storage_plan_digest != self.plan
            || plan.digest()? != self.plan
            || e.security_policy_digest != Policy::fixed().digest()?
            || e.manifest_digest != self.manifest
            || e.binding_digest != self.binding
            || e.lease_epoch != self.epoch
            || e.claim_key != self.claim
            || e.deadline_scope != self.deadline
            || !digest(&e.objects_digest)
            || self
                .lease_objects
                .as_ref()
                .is_some_and(|s| *s != e.objects_digest)
            || self.pending.is_some()
            || self.snapshot_pending.is_some()
            || self.action_pending.is_some()
        {
            return Err(invalid());
        }
        // An original file may not be missing/config half-created when the
        // official callback could run. Whole tree must be Working or Restored.
        let all_files = |state| {
            plan.roots
                .iter()
                .flat_map(|r| &r.nodes)
                .filter(|n| n.kind == Kind::CustodiedFile)
                .all(|n| self.nodes[&n.node_id].0 == state)
        };
        let anchors = plan
            .roots
            .iter()
            .flat_map(|r| &r.nodes)
            .filter(|n| n.kind != Kind::CustodiedFile)
            .all(|n| self.nodes[&n.node_id].0 >= 1);
        let next = match (self.lease_state, e.operation) {
            (LeaseState::Active, LeaseOperation::LeaseReleasePrepared)
                if anchors
                    && (all_files(4) || all_files(8))
                    && (recovery_release
                        || matches!(
                            self.work_mode,
                            super::observations::WorkMode::Sealed
                                | super::observations::WorkMode::Open
                        )) =>
            {
                LeaseState::ReleasePrepared
            }
            (LeaseState::ReleasePrepared, LeaseOperation::LeaseReleasedVerified) => {
                LeaseState::Released
            }
            (LeaseState::Released, LeaseOperation::LeaseAcquirePrepared) => {
                LeaseState::AcquirePrepared
            }
            (LeaseState::AcquirePrepared, LeaseOperation::LeaseAcquiredVerified) => {
                LeaseState::Active
            }
            _ => return Err(invalid()),
        };
        let new_epoch = if e.operation == LeaseOperation::LeaseAcquiredVerified {
            let new = e.next_epoch.as_ref().ok_or_else(invalid)?;
            if !uuid(new) || self.seen_epochs.contains(new) {
                return Err(invalid());
            }
            Some(new.clone())
        } else {
            if e.next_epoch.is_some() {
                return Err(invalid());
            }
            None
        };
        self.tip = hash::sha256_hex(&encoded(e)?);
        self.sequence = e.sequence;
        self.event_ids.insert(e.event_id.clone());
        self.lease_objects = Some(e.objects_digest.clone());
        self.lease_state = next;
        if let Some(epoch) = new_epoch {
            self.seen_epochs.insert(epoch.clone());
            self.epoch = epoch;
        }
        Ok(())
    }
}
#[cfg(test)]
#[path = "segment_data_tests.rs"]
mod tests;
