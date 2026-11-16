//! Snapshot links are canonical DATA, not native Ready/Closed qualifications.
//! Native object retention and typed official/source predicates remain distinct.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum SnapshotPurpose {
    Observation,
    Ready,
    PreStop,
    Stopped,
}
impl SnapshotPurpose {
    pub(super) fn leaf(self) -> &'static str {
        match self {
            Self::Observation => "observation",
            Self::Ready => "ready",
            Self::PreStop => "pre-stop",
            Self::Stopped => "stopped",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum LifecyclePhase {
    Preparing,
    Ready,
    PreStop,
    TeardownPrepared,
    Stopped,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct SnapshotContext {
    session_id: String,
    storage_plan_digest: String,
    binding_digest: String,
    manifest_digest: String,
    deadline_scope: String,
    frame_digests: [String; 2],
    purpose: SnapshotPurpose,
    observation_sequence: u32,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct SnapshotReference {
    schema: String,
    directory: String,
    volume: u32,
    file_id: String,
    security: SnapshotData,
    metadata_id: String,
    metadata_security: SnapshotData,
    metadata_sha256: String,
    frame_digest: String,
    context: SnapshotContext,
    runtime_authority: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum SnapshotOperation {
    SnapshotLinkIntent,
    SnapshotLinkedVerified,
    ReadyVerified,
    PreStopVerified,
    TeardownPrepared,
    StoppedVerified,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct SnapshotEvent {
    pub(super) schema: String,
    pub(super) document_kind: String,
    pub(super) storage_strategy: String,
    pub(super) event_id: String,
    pub(super) sequence: u32,
    pub(super) session_id: String,
    pub(super) operation: SnapshotOperation,
    pub(super) purpose: SnapshotPurpose,
    pub(super) observation_sequence: u32,
    pub(super) frame_digests: [String; 2],
    pub(super) references: Option<[SnapshotReference; 2]>,
    pub(super) ready: Option<(u32, u16, String)>,
    pub(super) lease_epoch: String,
    pub(super) claim_key: String,
    pub(super) deadline_scope: String,
    pub(super) previous_event_digest: String,
    pub(super) storage_plan_digest: String,
    pub(super) security_policy_digest: String,
    pub(super) manifest_digest: String,
    pub(super) binding_digest: String,
}
impl SnapshotReference {
    pub(super) fn object_ids(&self) -> [(u32, String); 2] {
        [
            (self.volume, self.file_id.clone()),
            (self.volume, self.metadata_id.clone()),
        ]
    }
    pub(super) fn context_metadata(&self) -> io::Result<serde_json::Value> {
        serde_json::to_value(&self.context).map_err(|_| invalid())
    }
    pub(super) fn purpose(&self) -> SnapshotPurpose {
        self.context.purpose
    }
    pub(super) fn observation_sequence(&self) -> u32 {
        self.context.observation_sequence
    }
    pub(super) fn from_native_data(value: serde_json::Value) -> io::Result<Self> {
        serde_json::from_value(value).map_err(|_| invalid())
    }
    pub(super) fn validate(
        &self,
        plan: &StoragePlan,
        index: usize,
        g: &Progress,
        purpose: SnapshotPurpose,
        sequence: u32,
        frames: &[String; 2],
    ) -> io::Result<()> {
        let root = plan.roots.get(index).ok_or_else(invalid)?;
        let profile = Path::new(&root.root).parent().ok_or_else(invalid)?;
        let leaf = if index == 0 { "codex" } else { "grok" };
        let purpose_leaf = if purpose == SnapshotPurpose::Observation {
            format!("{}-{sequence:06}", purpose.leaf())
        } else {
            purpose.leaf().into()
        };
        let expected = profile.join(format!(
            ".ocxd024-observed-{}-{purpose_leaf}-{leaf}",
            g.session
        ));
        if self.schema != "windows-anchored-work-observation-reference-v3"
            || self.runtime_authority
            || Path::new(&self.directory) != expected
            || self.volume != root.custody_volume
            || !decimal(&self.file_id)
            || !decimal(&self.metadata_id)
            || self.file_id == self.metadata_id
            || !digest(&self.metadata_sha256)
            || self.frame_digest != frames[index]
            || !self.security.matches_id(self.volume, &self.file_id)
            || !self
                .metadata_security
                .matches_id(self.volume, &self.metadata_id)
            || self.context.session_id != g.session
            || self.context.storage_plan_digest != g.plan
            || self.context.binding_digest != g.binding
            || self.context.manifest_digest != g.manifest
            || self.context.deadline_scope != g.deadline
            || self.context.frame_digests != *frames
            || self.context.purpose != purpose
            || self.context.observation_sequence != sequence
        {
            return Err(invalid());
        }
        self.security.validate()?;
        self.metadata_security.validate()?;
        Ok(())
    }
}
impl Progress {
    pub(super) fn apply_snapshot(
        &mut self,
        plan: &StoragePlan,
        e: &SnapshotEvent,
    ) -> io::Result<()> {
        if self.recovery.is_some() {
            return Err(invalid());
        }
        if e.schema
            != crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_ORIGINALS_SCHEMA_V3
            || e.document_kind != "snapshot-phase-event"
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
            || self.work_mode != observations::WorkMode::Observed
            || self.pending.is_some()
            || self.action_pending.is_some()
            || self.observed_frames.as_ref() != Some(&e.frame_digests)
            || e.frame_digests.iter().any(|d| !digest(d))
        {
            return Err(invalid());
        }
        let mut links = None;
        let mut next_pending = self.snapshot_pending;
        let mut next_phase = self.lifecycle;
        match e.operation {
            SnapshotOperation::SnapshotLinkIntent => {
                if self.snapshot_pending.is_some()
                    || if e.purpose == SnapshotPurpose::Observation {
                        self.linked_frames.contains_key(&self.observed_sequence)
                    } else {
                        self.snapshot_links.contains_key(&e.purpose)
                    }
                    || e.observation_sequence != self.observed_sequence
                    || e.references.is_some()
                    || e.ready.is_some()
                {
                    return Err(invalid());
                }
                if match e.purpose {
                    SnapshotPurpose::Observation => false,
                    SnapshotPurpose::Ready => {
                        self.snapshot_links.contains_key(&SnapshotPurpose::PreStop)
                    }
                    SnapshotPurpose::PreStop => {
                        !self.snapshot_links.contains_key(&SnapshotPurpose::Ready)
                            || self.lifecycle != LifecyclePhase::Ready
                            || !matches!(
                                self.action_kind,
                                actions::ActionKind::Stop | actions::ActionKind::Restart
                            )
                    }
                    SnapshotPurpose::Stopped => {
                        !self.snapshot_links.contains_key(&SnapshotPurpose::PreStop)
                            || self.lifecycle != LifecyclePhase::TeardownPrepared
                    }
                } {
                    return Err(invalid());
                }
                next_pending = Some((e.purpose, e.observation_sequence));
            }
            SnapshotOperation::SnapshotLinkedVerified => {
                if self.snapshot_pending != Some((e.purpose, e.observation_sequence))
                    || e.ready.is_some()
                {
                    return Err(invalid());
                }
                let refs = e.references.as_ref().ok_or_else(invalid)?;
                for (i, r) in refs.iter().enumerate() {
                    r.validate(
                        plan,
                        i,
                        self,
                        e.purpose,
                        e.observation_sequence,
                        &e.frame_digests,
                    )?;
                }
                let mut identities = plan
                    .roots
                    .iter()
                    .flat_map(|root| {
                        root.nodes
                            .iter()
                            .map(|n| (n.volume, n.file_id.clone()))
                            .chain(std::iter::once((
                                root.custody_volume,
                                root.custody_id.clone(),
                            )))
                    })
                    .collect::<BTreeSet<_>>();
                for r in refs {
                    if !identities.insert((r.volume, r.file_id.clone()))
                        || !identities.insert((r.volume, r.metadata_id.clone()))
                    {
                        return Err(invalid());
                    }
                }
                links = Some([
                    hash::sha256_hex(&encoded(&refs[0])?),
                    hash::sha256_hex(&encoded(&refs[1])?),
                ]);
                next_pending = None;
            }
            SnapshotOperation::ReadyVerified
            | SnapshotOperation::PreStopVerified
            | SnapshotOperation::StoppedVerified => {
                let next = match e.operation {
                    SnapshotOperation::ReadyVerified
                        if self.lifecycle == LifecyclePhase::Preparing
                            && e.purpose == SnapshotPurpose::Ready
                            && self.action_kind == actions::ActionKind::Start =>
                    {
                        LifecyclePhase::Ready
                    }
                    SnapshotOperation::PreStopVerified
                        if self.lifecycle == LifecyclePhase::Ready
                            && e.purpose == SnapshotPurpose::PreStop
                            && matches!(
                                self.action_kind,
                                actions::ActionKind::Stop | actions::ActionKind::Restart
                            )
                            && self.ready_generation == e.ready =>
                    {
                        LifecyclePhase::PreStop
                    }
                    SnapshotOperation::StoppedVerified
                        if self.lifecycle == LifecyclePhase::TeardownPrepared
                            && e.purpose == SnapshotPurpose::Stopped
                            && matches!(
                                self.action_kind,
                                actions::ActionKind::Stop | actions::ActionKind::Restart
                            )
                            && self.ready_generation == e.ready =>
                    {
                        LifecyclePhase::Stopped
                    }
                    _ => return Err(invalid()),
                };
                if self.snapshot_pending.is_some() {
                    return Err(invalid());
                }
                let refs = e.references.as_ref().ok_or_else(invalid)?;
                for (i, r) in refs.iter().enumerate() {
                    r.validate(
                        plan,
                        i,
                        self,
                        e.purpose,
                        e.observation_sequence,
                        &e.frame_digests,
                    )?;
                }
                let hashes = [
                    hash::sha256_hex(&encoded(&refs[0])?),
                    hash::sha256_hex(&encoded(&refs[1])?),
                ];
                if self.snapshot_links.get(&e.purpose) != Some(&hashes)
                    || !e
                        .ready
                        .as_ref()
                        .is_some_and(|(pid, port, d)| *pid > 0 && *port > 0 && digest(d))
                {
                    return Err(invalid());
                }
                next_phase = next;
            }
            SnapshotOperation::TeardownPrepared => {
                if self.lifecycle != LifecyclePhase::PreStop
                    || e.purpose != SnapshotPurpose::PreStop
                    || e.references.is_some()
                    || e.ready != self.ready_generation
                    || self.snapshot_pending.is_some()
                {
                    return Err(invalid());
                }
                next_phase = LifecyclePhase::TeardownPrepared;
            }
        }
        self.tip = hash::sha256_hex(&encoded(e)?);
        self.sequence = e.sequence;
        self.event_ids.insert(e.event_id.clone());
        self.snapshot_pending = next_pending;
        self.lifecycle = next_phase;
        if e.operation == SnapshotOperation::ReadyVerified {
            self.ready_generation = e.ready.clone();
        }
        if let Some(hashes) = links {
            self.snapshot_links.insert(e.purpose, hashes);
            self.linked_frames
                .insert(e.observation_sequence, e.frame_digests.clone());
        }
        Ok(())
    }
}
#[cfg(test)]
#[path = "snapshot_data_tests.rs"]
pub(super) mod tests;
