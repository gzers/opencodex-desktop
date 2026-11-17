//! Strict original-history locator DATA; never native Closed or handle release.
use super::*;
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct HistoryContextData {
    pub(super) session_id: String,
    pub(super) storage_plan_digest: String,
    pub(super) security_policy_digest: String,
    pub(super) manifest_digest: String,
    pub(super) binding_digest: String,
    pub(super) deadline_scope: String,
    pub(super) lease_epoch: String,
    pub(super) claim_key: String,
    pub(super) release_objects_digest: String,
    pub(super) released_event_digest: String,
    pub(super) recovery_plan_digest: String,
    pub(super) original_frames: [String; 2],
    pub(super) history_intent_sequence: u32,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct HistoryReference {
    schema: String,
    root_id: String,
    directory: String,
    volume: u32,
    file_id: String,
    security: SnapshotData,
    metadata_id: String,
    metadata_security: SnapshotData,
    metadata_sha256: String,
    frame_digest: String,
    context: HistoryContextData,
    runtime_authority: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum HistoryOperation {
    OriginalHistoryIntent,
    OriginalHistoryLinkedVerified,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct HistoryEvent {
    pub(super) schema: String,
    pub(super) document_kind: String,
    pub(super) storage_strategy: String,
    pub(super) event_id: String,
    pub(super) sequence: u32,
    pub(super) session_id: String,
    pub(super) operation: HistoryOperation,
    pub(super) context: HistoryContextData,
    pub(super) references: Option<[HistoryReference; 2]>,
    pub(super) previous_event_digest: String,
}
#[derive(Serialize, Deserialize)]
pub(super) struct HistoryProgress {
    context: HistoryContextData,
    references: Option<[String; 2]>,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct HistorySummary {
    context_digest: String,
    intent_sequence: u32,
    linked_references: Option<[String; 2]>,
}
impl HistoryReference {
    pub(super) fn from_native_data(value: serde_json::Value) -> io::Result<Self> {
        serde_json::from_value(value).map_err(|_| invalid())
    }
    fn validate(
        &self,
        plan: &StoragePlan,
        index: usize,
        context: &HistoryContextData,
    ) -> io::Result<()> {
        let root = &plan.roots[index];
        let leaf = if index == 0 { ".codex" } else { ".grok" };
        let profile = Path::new(&root.root).parent().ok_or_else(invalid)?;
        let expected = profile.join(format!(
            ".ocxd024-history-{}-{}",
            context.session_id,
            leaf.trim_start_matches('.')
        ));
        if self.schema != "windows-anchored-original-history-reference-v3"
            || self.root_id != leaf
            || Path::new(&self.directory) != expected
            || self.volume != root.custody_volume
            || !decimal(&self.file_id)
            || !decimal(&self.metadata_id)
            || self.file_id == self.metadata_id
            || !self.security.matches_id(self.volume, &self.file_id)
            || !self
                .metadata_security
                .matches_id(self.volume, &self.metadata_id)
            || !digest(&self.metadata_sha256)
            || self.frame_digest != context.original_frames[index]
            || self.context != *context
            || self.runtime_authority
        {
            return Err(invalid());
        }
        self.security.validate()?;
        self.metadata_security.validate()?;
        validate_local_path(Path::new(&self.directory))?;
        Ok(())
    }
}
impl Progress {
    pub(super) fn verify_original_history_references(
        &self,
        plan: &StoragePlan,
        values: [serde_json::Value; 2],
    ) -> io::Result<()> {
        let history = self.history.as_ref().ok_or_else(invalid)?;
        let mut hashes = Vec::new();
        for (i, value) in values.into_iter().enumerate() {
            let reference = HistoryReference::from_native_data(value)?;
            reference.validate(plan, i, &history.context)?;
            hashes.push(hash::sha256_hex(&encoded(&reference)?));
        }
        if history.references.as_ref().map(|r| r.as_slice()) != Some(hashes.as_slice()) {
            return Err(invalid());
        }
        Ok(())
    }
    pub(super) fn history_linked(&self) -> bool {
        self.history
            .as_ref()
            .is_some_and(|h| h.references.is_some())
    }
    pub(super) fn history_context(&self, plan: &StoragePlan) -> io::Result<HistoryContextData> {
        if !self.complete_files(plan)
            || self.lease_state != segments::LeaseState::Released
            || self.lifecycle != snapshots::LifecyclePhase::Stopped
            || self.history.is_some()
            || !matches!(
                self.action_kind,
                actions::ActionKind::Stop | actions::ActionKind::Restart
            )
        {
            return Err(invalid());
        }
        let frames = self.original_history_frames(plan)?;
        Ok(HistoryContextData {
            session_id: self.session.clone(),
            storage_plan_digest: self.plan.clone(),
            security_policy_digest: Policy::fixed().digest()?,
            manifest_digest: self.manifest.clone(),
            binding_digest: self.binding.clone(),
            deadline_scope: self.deadline.clone(),
            lease_epoch: self.epoch.clone(),
            claim_key: self.claim.clone(),
            release_objects_digest: self.lease_objects.clone().ok_or_else(invalid)?,
            released_event_digest: self.tip.clone(),
            recovery_plan_digest: self.retirement_history_digest()?,
            original_frames: [
                hash::sha256_hex(&encoded(&frames[0])?),
                hash::sha256_hex(&encoded(&frames[1])?),
            ],
            history_intent_sequence: self.sequence.checked_add(1).ok_or_else(invalid)?,
        })
    }
    pub(super) fn history_summary(&self) -> io::Result<Option<HistorySummary>> {
        self.history
            .as_ref()
            .map(|h| {
                Ok(HistorySummary {
                    context_digest: hash::sha256_hex(&encoded(&h.context)?),
                    intent_sequence: h.context.history_intent_sequence,
                    linked_references: h.references.clone(),
                })
            })
            .transpose()
    }
    pub(super) fn history_event(
        &self,
        operation: HistoryOperation,
        context: &HistoryContextData,
        references: Option<[HistoryReference; 2]>,
    ) -> io::Result<HistoryEvent> {
        Ok(HistoryEvent {
            schema:
                crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3
                    .into(),
            document_kind: "original-history-event".into(),
            storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
            event_id: uuid::Uuid::new_v4().to_string(),
            sequence: self.sequence.checked_add(1).ok_or_else(invalid)?,
            session_id: self.session.clone(),
            operation,
            context: serde_json::from_slice(&encoded(context)?).map_err(|_| invalid())?,
            references,
            previous_event_digest: self.tip.clone(),
        })
    }
    pub(super) fn apply_history(&mut self, plan: &StoragePlan, e: &HistoryEvent) -> io::Result<()> {
        if self.batch.is_some() {
            return Err(invalid());
        }
        if e.schema
            != crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3
            || e.document_kind != "original-history-event"
            || e.storage_strategy != ANCHORED_STORAGE_STRATEGY
            || e.session_id != self.session
            || e.sequence != self.sequence.checked_add(1).ok_or_else(invalid)?
            || !uuid(&e.event_id)
            || self.event_ids.contains(&e.event_id)
            || e.previous_event_digest != self.tip
            || !self.complete_files(plan)
            || self.lease_state != segments::LeaseState::Released
            || self.pending.is_some()
            || self.action_pending.is_some()
            || self.snapshot_pending.is_some()
            || self.lifecycle != snapshots::LifecyclePhase::Stopped
        {
            return Err(invalid());
        }
        let bytes = encoded(e)?;
        match e.operation {
            HistoryOperation::OriginalHistoryIntent => {
                if e.references.is_some() || e.context != self.history_context(plan)? {
                    return Err(invalid());
                }
                self.history = Some(HistoryProgress {
                    context: serde_json::from_slice(&encoded(&e.context)?)
                        .map_err(|_| invalid())?,
                    references: None,
                });
            }
            HistoryOperation::OriginalHistoryLinkedVerified => {
                let prior = self.history.as_ref().ok_or_else(invalid)?;
                if prior.references.is_some() || prior.context != e.context {
                    return Err(invalid());
                }
                let refs = e.references.as_ref().ok_or_else(invalid)?;
                let mut ids = self.recovery_reserved_identities()?;
                for (index, r) in refs.iter().enumerate() {
                    r.validate(plan, index, &e.context)?;
                    if !ids.insert((r.volume, r.file_id.clone()))
                        || !ids.insert((r.volume, r.metadata_id.clone()))
                    {
                        return Err(invalid());
                    }
                }
                self.history.as_mut().ok_or_else(invalid)?.references = Some([
                    hash::sha256_hex(&encoded(&refs[0])?),
                    hash::sha256_hex(&encoded(&refs[1])?),
                ]);
            }
        }
        self.sequence = e.sequence;
        self.tip = hash::sha256_hex(&bytes);
        self.event_ids.insert(e.event_id.clone());
        Ok(())
    }
}
#[cfg(test)]
#[path = "history_data_tests.rs"]
pub(super) mod tests;
