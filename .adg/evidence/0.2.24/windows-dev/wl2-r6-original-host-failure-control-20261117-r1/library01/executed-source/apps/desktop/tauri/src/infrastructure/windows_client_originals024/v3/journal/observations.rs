use super::super::super::coordination::AnchoredClaim;
use super::super::observations::{WorkEvent, WorkMode, WorkOperation};
use super::*;
impl Journal {
    pub(in crate::infrastructure::windows_client_originals024) fn resume_linked_work<F>(
        &mut self,
        plans: &[serde_json::Value; 2],
        observed: super::super::super::tree::SnapshotLinkObservation,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
        native: F,
    ) -> io::Result<()>
    where
        F: FnOnce(&mut AnchoredClaim<'_>) -> io::Result<[String; 2]>,
    {
        let result = (|| {
            self.check_work_claim(plans, claim, end)?;
            let refs = observed
                .into_values()
                .into_iter()
                .map(super::super::snapshots::SnapshotReference::from_native_data)
                .collect::<io::Result<Vec<_>>>()?;
            let purpose = refs[0].purpose();
            let sequence = refs[0].observation_sequence();
            if refs[1].purpose() != purpose
                || refs[1].observation_sequence() != sequence
                || sequence != self.progress.observed_sequence
            {
                return Err(invalid());
            }
            for (i, r) in refs.iter().enumerate() {
                r.validate(
                    &self.manifest.storage_plan,
                    i,
                    &self.progress,
                    purpose,
                    sequence,
                    self.progress.observed_frames.as_ref().ok_or_else(invalid)?,
                )?;
            }
            let hashes = [
                hash::sha256_hex(&encoded(&refs[0])?),
                hash::sha256_hex(&encoded(&refs[1])?),
            ];
            if self.progress.snapshot_links.get(&purpose) != Some(&hashes) {
                return Err(invalid());
            }
            let frames = self.progress.observed_frames.clone().ok_or_else(invalid)?;
            let e = self.work_event(WorkOperation::WorkResumeIntent, Some(frames.clone()))?;
            self.append_entry_inner(ChainEntry::Work(e), end)?;
            #[cfg(test)]
            if self.fault == 24 {
                return Err(io::Error::other("injected after durable resume intent/current before exact work observer handoff"));
            }
            self.check_work_claim(plans, claim, end)?;
            let actual = native(claim)?;
            if actual != frames {
                return Err(invalid());
            }
            self.check_work_claim(plans, claim, end)?;
            #[cfg(test)]
            {
                if self.fault == 25 {
                    return Err(io::Error::other(
                        "injected after native work generation reopened before resume result",
                    ));
                }
                if self.fault == 26 {
                    self.fault = 1;
                } else if self.fault == 27 {
                    self.fault = 2;
                }
            }
            let e = self.work_event(WorkOperation::WorkResumedVerified, Some(actual))?;
            self.append_entry_inner(ChainEntry::Work(e), end)?;
            self.check_work_claim(plans, claim, end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    fn check_work_claim(
        &mut self,
        plans: &[serde_json::Value; 2],
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        self.check(end)?;
        claim.check(&self.pins[0].path().join(".codex"), end)?;
        if self.progress.lease_state != LeaseState::Active
            || claim.epoch() != self.progress.epoch
            || claim.key() != self.progress.claim
        {
            return Err(invalid());
        }
        let expected = self
            .manifest
            .storage_plan
            .roots
            .iter()
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| invalid())?;
        if expected.as_slice() != plans {
            return Err(invalid());
        }
        Ok(())
    }
    fn work_event(&self, op: WorkOperation, digests: Option<[String; 2]>) -> io::Result<WorkEvent> {
        Ok(WorkEvent {
            schema: ANCHORED_ORIGINALS_SCHEMA_V3.into(),
            document_kind: "work-interval-event".into(),
            storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
            event_id: uuid::Uuid::new_v4().to_string(),
            sequence: self.progress.sequence.checked_add(1).ok_or_else(invalid)?,
            session_id: self.progress.session.clone(),
            operation: op,
            frame_digests: digests,
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
    pub(in crate::infrastructure::windows_client_originals024) fn work_transition<F>(
        &mut self,
        plans: &[serde_json::Value; 2],
        opening: bool,
        before_frames: Option<[String; 2]>,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
        native: F,
    ) -> io::Result<()>
    where
        F: FnOnce(&mut AnchoredClaim<'_>) -> io::Result<[String; 2]>,
    {
        let result = (|| {
            self.check_work_claim(plans, claim, end)?;
            let (intent, verified) = if opening {
                (
                    WorkOperation::WorkOpenIntent,
                    WorkOperation::WorkOpenedVerified,
                )
            } else {
                (
                    WorkOperation::WorkSealIntent,
                    WorkOperation::WorkSealedVerified,
                )
            };
            let e = self.work_event(intent, before_frames)?;
            self.append_entry_inner(ChainEntry::Work(e), end)?;
            #[cfg(test)]
            if self.fault == 13 {
                return Err(io::Error::other(
                    "injected after work intent/current before native handle transition",
                ));
            }
            self.check_work_claim(plans, claim, end)?;
            let frames = native(claim)?;
            self.check_work_claim(plans, claim, end)?;
            #[cfg(test)]
            {
                if self.fault == 14 {
                    return Err(io::Error::other(
                        "injected after native work transition before result",
                    ));
                }
                if self.fault == 15 {
                    self.fault = 1;
                } else if self.fault == 16 {
                    self.fault = 2;
                }
            }
            let e = self.work_event(verified, Some(frames))?;
            self.append_entry_inner(ChainEntry::Work(e), end)?;
            self.check_work_claim(plans, claim, end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn unchanged_work<F>(
        &mut self,
        plans: &[serde_json::Value; 2],
        frames: [String; 2],
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
        native: F,
    ) -> io::Result<()>
    where
        F: FnOnce(&mut AnchoredClaim<'_>) -> io::Result<()>,
    {
        let result = (|| {
            self.check_work_claim(plans, claim, end)?;
            if self.progress.work_mode != WorkMode::Observed
                || Some(&frames) != self.progress.work_frames.as_ref()
            {
                return Err(invalid());
            }
            native(claim)?;
            self.check_work_claim(plans, claim, end)?;
            let e = self.work_event(WorkOperation::WorkUnchangedVerified, Some(frames))?;
            self.append_entry_inner(ChainEntry::Work(e), end)?;
            self.check_work_claim(plans, claim, end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn work_mode_for_test(
        &self,
    ) -> String {
        format!("{:?}", self.progress.work_mode)
    }
}
