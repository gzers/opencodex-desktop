//! Content qualification DATA in the original canonical chain. Decoding never
//! reconstructs an actual Stop scope, claim, assessment or restoration permit.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum ContentOperation {
    ContentQualificationIntent,
    ContentQualifiedVerified,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum ContentAction {
    AlreadyOriginal,
    RestoreOriginal,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct ContentData {
    pub(super) scope_digest: String,
    pub(super) series_digest: String,
    pub(super) sequences: [u32; 3],
    pub(super) bindings: [snapshots::SnapshotContext; 3],
    pub(super) frames: [[String; 2]; 3],
    // Fixed order of the unchanged shared policy: Catalog, ModelsCache, Audit.
    pub(super) actions: [ContentAction; 3],
}
impl ContentData {
    pub(super) fn digest(&self) -> io::Result<String> {
        Ok(hash::sha256_hex(&encoded(self)?))
    }
    fn validate(&self, g: &Progress) -> io::Result<()> {
        if !digest(&self.scope_digest)
            || self.series_digest
                != hash::sha256_hex(&encoded(&serde_json::json!({
                    "sequences": self.sequences, "bindings": self.bindings, "frames": self.frames
                }))?)
            || self.sequences[0] == 0
            || self.sequences[0] >= self.sequences[1]
            || self.sequences[1] >= self.sequences[2]
            || self.sequences[2] != g.observed_sequence
        {
            return Err(invalid());
        }
        for (i, purpose) in [
            SnapshotPurpose::Ready,
            SnapshotPurpose::PreStop,
            SnapshotPurpose::Stopped,
        ]
        .into_iter()
        .enumerate()
        {
            let context = &self.bindings[i];
            let binding = serde_json::to_value(context).map_err(|_| invalid())?;
            let exact = serde_json::json!({"sessionId":g.session,"storagePlanDigest":g.plan,
                "bindingDigest":g.binding,"manifestDigest":g.manifest,
                "deadlineScope":context.deadline_scope(),"frameDigests":self.frames[i],
                "purpose":purpose,"observationSequence":self.sequences[i]});
            if binding != exact
                || !uuid(context.deadline_scope())
                || !g.seen_actions.contains(context.deadline_scope())
                || (i > 0 && context.deadline_scope() != g.deadline)
                || self.frames[i].iter().any(|s| !digest(s))
                || g.linked_frames.get(&self.sequences[i]) != Some(&self.frames[i])
                || !g.snapshot_links.contains_key(&purpose)
                || g.snapshot_stages.get(&purpose)
                    != Some(&(self.sequences[i], hash::sha256_hex(&encoded(&binding)?)))
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct ContentSummary {
    pub(super) digest: String,
    pub(super) verified: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct ContentEvent {
    pub(super) schema: String,
    pub(super) document_kind: String,
    pub(super) storage_strategy: String,
    pub(super) event_id: String,
    pub(super) sequence: u32,
    pub(super) session_id: String,
    pub(super) operation: ContentOperation,
    pub(super) content: ContentData,
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
    pub(super) fn apply_content(&mut self, plan: &StoragePlan, e: &ContentEvent) -> io::Result<()> {
        if e.schema
            != crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3
            || e.document_kind != "content-qualification-event"
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
            || !matches!(
                self.action_kind,
                actions::ActionKind::Stop | actions::ActionKind::Restart
            )
            || self.lifecycle != snapshots::LifecyclePhase::Stopped
            || self.lease_state != segments::LeaseState::Active
            || self.work_mode != observations::WorkMode::Observed
            || self.pending.is_some()
            || self.snapshot_pending.is_some()
            || self.action_pending.is_some()
            || self.recovery.is_some()
            || self.history.is_some()
            || self.handle_progress.is_some()
            || self.batch.is_some()
            || !plan.roots.iter().flat_map(|r| &r.nodes).all(|n| {
                self.nodes
                    .get(&n.node_id)
                    .is_some_and(|s| s.0 == if n.kind == Kind::CustodiedFile { 4 } else { 1 })
            })
        {
            return Err(invalid());
        }
        e.content.validate(self)?;
        let content_digest = e.content.digest()?;
        match e.operation {
            ContentOperation::ContentQualificationIntent if self.content.is_none() => {}
            ContentOperation::ContentQualifiedVerified
                if self.content.as_ref()
                    == Some(&ContentSummary {
                        digest: content_digest.clone(),
                        verified: false,
                    }) => {}
            _ => return Err(invalid()),
        }
        let tip = hash::sha256_hex(&encoded(e)?);
        self.content = Some(ContentSummary {
            digest: content_digest,
            verified: e.operation == ContentOperation::ContentQualifiedVerified,
        });
        self.tip = tip;
        self.sequence = e.sequence;
        self.event_ids.insert(e.event_id.clone());
        Ok(())
    }
}
#[cfg(test)]
#[path = "qualification_data_tests.rs"]
mod tests;
