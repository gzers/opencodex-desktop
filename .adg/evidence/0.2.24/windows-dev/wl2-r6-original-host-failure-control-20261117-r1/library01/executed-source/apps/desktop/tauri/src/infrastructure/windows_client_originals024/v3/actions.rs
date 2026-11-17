//! Canonical new-action locators. Actual permission comes from NativeAction
//! and original live Source/Owner, never from this serde data or a new timer.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum ActionKind {
    Start,
    Stop,
    Restart,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum ActionOperation {
    StopActionIntent,
    StopActionBoundVerified,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct ActionEvent {
    pub(super) schema: String,
    pub(super) document_kind: String,
    pub(super) storage_strategy: String,
    pub(super) event_id: String,
    pub(super) sequence: u32,
    pub(super) session_id: String,
    pub(super) operation: ActionOperation,
    pub(super) next_action_scope: String,
    pub(super) action_kind: ActionKind,
    pub(super) ready: (u32, u16, String),
    pub(super) lease_objects_digest: String,
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
    pub(super) fn apply_action(&mut self, plan: &StoragePlan, e: &ActionEvent) -> io::Result<()> {
        if self.batch.is_some() {
            return Err(invalid());
        }
        if self.recovery.is_some() {
            return Err(invalid());
        }
        if e.schema
            != crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3
            || e.document_kind != "original-action-event"
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
            || self.lease_state != segments::LeaseState::Released
            || self.work_mode != observations::WorkMode::Open
            || self.pending.is_some()
            || self.snapshot_pending.is_some()
            || self.lifecycle != snapshots::LifecyclePhase::Ready
            || self.ready_generation.as_ref() != Some(&e.ready)
            || self.lease_objects.as_ref() != Some(&e.lease_objects_digest)
            || !uuid(&e.next_action_scope)
            || self.seen_actions.contains(&e.next_action_scope)
            || !matches!(e.action_kind, ActionKind::Stop | ActionKind::Restart)
            || self.action_kind != ActionKind::Start
        {
            return Err(invalid());
        }
        match e.operation {
            ActionOperation::StopActionIntent => {
                if self.action_pending.is_some() {
                    return Err(invalid());
                }
            }
            ActionOperation::StopActionBoundVerified => {
                if self.action_pending.as_ref()
                    != Some(&(e.next_action_scope.clone(), e.action_kind))
                {
                    return Err(invalid());
                }
            }
        }
        self.tip = hash::sha256_hex(&encoded(e)?);
        self.sequence = e.sequence;
        self.event_ids.insert(e.event_id.clone());
        match e.operation {
            ActionOperation::StopActionIntent => {
                self.action_pending = Some((e.next_action_scope.clone(), e.action_kind))
            }
            ActionOperation::StopActionBoundVerified => {
                self.deadline = e.next_action_scope.clone();
                self.seen_actions.insert(e.next_action_scope.clone());
                self.action_kind = e.action_kind;
                self.action_pending = None;
            }
        }
        Ok(())
    }
}
#[cfg(test)]
#[path = "action_data_tests.rs"]
mod tests;
