use super::super::super::{coordination::AnchoredClaim, source::Source, tree::SnapshotWitness};
use super::super::snapshots::{
    LifecyclePhase, SnapshotEvent, SnapshotOperation, SnapshotPurpose, SnapshotReference,
};
use super::*;
use crate::infrastructure::windows_official_lifecycle024::{OriginalActionKind, Owner};
use crate::modules::runtime::official_lifecycle024::ServiceObservation;
impl Journal {
    fn snapshot_claim(&mut self, claim: &mut AnchoredClaim<'_>, end: Instant) -> io::Result<()> {
        self.check(end)?;
        claim.check(&self.pins[0].path().join(".codex"), end)?;
        if self.progress.lease_state != LeaseState::Active
            || self.progress.epoch != claim.epoch()
            || self.progress.claim != claim.key()
        {
            return Err(invalid());
        }
        Ok(())
    }
    fn snapshot_event(
        &self,
        operation: SnapshotOperation,
        purpose: SnapshotPurpose,
        sequence: u32,
        references: Option<[SnapshotReference; 2]>,
        ready: Option<(u32, u16, String)>,
    ) -> io::Result<SnapshotEvent> {
        Ok(SnapshotEvent {
            schema: ANCHORED_ORIGINALS_SCHEMA_V3.into(),
            document_kind: "snapshot-phase-event".into(),
            storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
            event_id: uuid::Uuid::new_v4().to_string(),
            sequence: self.progress.sequence.checked_add(1).ok_or_else(invalid)?,
            session_id: self.progress.session.clone(),
            operation,
            purpose,
            observation_sequence: sequence,
            frame_digests: self.progress.observed_frames.clone().ok_or_else(invalid)?,
            references,
            ready,
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
    fn begin_snapshot(
        &mut self,
        purpose: SnapshotPurpose,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<ObservationContext> {
        self.snapshot_claim(claim, end)?;
        let sequence = self.progress.sequence;
        let e = self.snapshot_event(
            SnapshotOperation::SnapshotLinkIntent,
            purpose,
            sequence,
            None,
            None,
        )?;
        self.append_entry_inner(ChainEntry::Snapshot(e), end)?;
        #[cfg(test)]
        if self.fault == 17 {
            return Err(io::Error::other(
                "injected after durable snapshot intent/current, before snapshot creation",
            ));
        }
        self.snapshot_claim(claim, end)?;
        Ok(ObservationContext {
            session: self.progress.session.clone(),
            plan: self.progress.plan.clone(),
            binding: self.progress.binding.clone(),
            manifest: self.progress.manifest.clone(),
            deadline_scope: self.progress.deadline.clone(),
            frames: self.progress.observed_frames.clone().ok_or_else(invalid)?,
            purpose,
            observation_sequence: sequence,
        })
    }
    pub(in crate::infrastructure::windows_client_originals024) fn begin_observation_snapshot_link(
        &mut self,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<ObservationContext> {
        let result = self.begin_snapshot(SnapshotPurpose::Observation, claim, end);
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn finish_snapshot_link(
        &mut self,
        mut witness: SnapshotWitness<'_>,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.snapshot_claim(claim, end)?;
            let (purpose, sequence) = self.progress.snapshot_pending.ok_or_else(invalid)?;
            let references = witness
                .references(end)?
                .into_iter()
                .map(SnapshotReference::from_native_data)
                .collect::<io::Result<Vec<_>>>()?
                .try_into()
                .map_err(|_| invalid())?;
            #[cfg(test)]
            {
                if self.fault == 18 {
                    return Err(io::Error::other(
                        "injected after BOTH native snapshot archives before link result",
                    ));
                }
                if self.fault == 19 {
                    self.fault = 1;
                } else if self.fault == 20 {
                    self.fault = 2;
                }
            }
            let e = self.snapshot_event(
                SnapshotOperation::SnapshotLinkedVerified,
                purpose,
                sequence,
                Some(references),
                None,
            )?;
            self.append_entry_inner(ChainEntry::Snapshot(e), end)?;
            witness.references(end)?;
            self.snapshot_claim(claim, end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    fn ready_source(
        &mut self,
        source: &mut Source,
        owner: &mut Owner,
        ready: &ServiceObservation,
        end: Instant,
    ) -> io::Result<()> {
        self.check_source_owner(owner, end)?;
        source.check(owner, end).map_err(io::Error::other)?;
        if source.binding_raw != encoded(&self.manifest.binding)?
            || self.source_action.as_ref().ok_or_else(invalid)?.kind() != OriginalActionKind::Start
        {
            return Err(invalid());
        }
        owner
            .originals_check_ready(ready, end)
            .map_err(io::Error::other)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn begin_ready_snapshot_link(
        &mut self,
        source: &mut Source,
        owner: &mut Owner,
        ready: &ServiceObservation,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<ObservationContext> {
        let result = (|| {
            self.ready_source(source, owner, ready, end)?;
            let context = self.begin_snapshot(SnapshotPurpose::Ready, claim, end)?;
            self.ready_source(source, owner, ready, end)?;
            Ok(context)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn record_ready_phase(
        &mut self,
        source: &mut Source,
        owner: &mut Owner,
        ready: &ServiceObservation,
        mut witness: SnapshotWitness<'_>,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.ready_source(source, owner, ready, end)?;
            self.snapshot_claim(claim, end)?;
            if self.progress.lifecycle != LifecyclePhase::Preparing {
                return Err(invalid());
            }
            let refs = witness
                .references(end)?
                .into_iter()
                .map(SnapshotReference::from_native_data)
                .collect::<io::Result<Vec<_>>>()?;
            let sequence = refs.first().ok_or_else(invalid)?.observation_sequence();
            let e = self.snapshot_event(
                SnapshotOperation::ReadyVerified,
                SnapshotPurpose::Ready,
                sequence,
                Some(refs.try_into().map_err(|_| invalid())?),
                Some((ready.pid(), ready.port(), ready.record_sha256().into())),
            )?;
            self.append_entry_inner(ChainEntry::Snapshot(e), end)?;
            witness.references(end)?;
            self.ready_source(source, owner, ready, end)?;
            self.snapshot_claim(claim, end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn snapshot_state_for_test(
        &self,
    ) -> (bool, usize, String) {
        (
            self.progress.snapshot_pending.is_some(),
            self.progress.snapshot_links.len(),
            format!("{:?}", self.progress.lifecycle),
        )
    }
}
