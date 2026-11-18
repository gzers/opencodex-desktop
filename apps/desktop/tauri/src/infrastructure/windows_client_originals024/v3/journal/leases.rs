//! One-action-window lease segments in the canonical v3 chain. Scope/phase
//! authorization for a DIFFERENT lifecycle action is intentionally not minted.
use super::super::super::coordination::{AnchoredClaim, LeaseFacts, NativeLease};
use super::super::segments::{LeaseEvent, LeaseOperation};
use super::*;
impl Journal {
    pub(in crate::infrastructure::windows_client_originals024) fn poison_segment(&mut self) {
        self.poisoned = true;
        if let Some(handoff) = &mut self.original_handoff {
            handoff.poison();
        }
    }
    pub(super) fn lease_event(
        &self,
        operation: LeaseOperation,
        facts: &LeaseFacts,
        next: Option<String>,
    ) -> io::Result<LeaseEvent> {
        if facts.claim_key != self.progress.claim || !digest(&facts.objects_digest) {
            return Err(invalid());
        }
        if operation != LeaseOperation::LeaseAcquiredVerified && facts.epoch != self.progress.epoch
        {
            return Err(invalid());
        }
        Ok(LeaseEvent {
            schema: ANCHORED_ORIGINALS_SCHEMA_V3.into(),
            document_kind: "lease-event".into(),
            storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
            event_id: uuid::Uuid::new_v4().to_string(),
            sequence: self.progress.sequence.checked_add(1).ok_or_else(invalid)?,
            session_id: self.progress.session.clone(),
            operation,
            lease_epoch: self.progress.epoch.clone(),
            next_epoch: next,
            claim_key: self.progress.claim.clone(),
            objects_digest: facts.objects_digest.clone(),
            deadline_scope: self.progress.deadline.clone(),
            previous_event_digest: self.progress.tip.clone(),
            storage_plan_digest: self.progress.plan.clone(),
            security_policy_digest: Policy::fixed().digest()?,
            manifest_digest: self.progress.manifest.clone(),
            binding_digest: self.progress.binding.clone(),
        })
    }
    pub(super) fn append_lease(&mut self, e: LeaseEvent, end: Instant) -> io::Result<()> {
        let result = self.append_entry_inner(ChainEntry::Lease(e), end);
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    fn check_plans(&mut self, plans: &[serde_json::Value; 2], end: Instant) -> io::Result<()> {
        self.check(end)?;
        let actual = self
            .manifest
            .storage_plan
            .roots
            .iter()
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()
            .map_err(io_error)?;
        if actual.as_slice() != plans {
            return Err(invalid());
        }
        Ok(())
    }
    pub(in crate::infrastructure::windows_client_originals024) fn prepare_lease_release(
        &mut self,
        plans: &[serde_json::Value; 2],
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            if self.progress.recovery.is_some() {
                return Err(invalid());
            }
            self.check_plans(plans, end)?;
            if self.progress.lease_state != LeaseState::Active || self.released_lease.is_some() {
                return Err(invalid());
            }
            let root = self.pins[0].path().join(".codex");
            let facts = claim.facts(&root, end)?;
            let event = self.lease_event(LeaseOperation::LeaseReleasePrepared, &facts, None)?;
            self.append_lease(event, end)?;
            claim.check(&root, end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    /// The caller keeps any remaining ranges, and the journal keeps any
    /// release receipt, even when post-unlock verification fails.
    pub(in crate::infrastructure::windows_client_originals024) fn release_prepared_native(
        &mut self,
        lease: &mut Option<NativeLease>,
        end: Instant,
    ) -> io::Result<()> {
        if self.poisoned || self.progress.lease_state != LeaseState::ReleasePrepared {
            return Err(invalid());
        }
        lease
            .as_mut()
            .ok_or_else(invalid)?
            .release_retained(&mut self.released_lease, end)?;
        drop(lease.take());
        Ok(())
    }
    pub(in crate::infrastructure::windows_client_originals024) fn finish_lease_release(
        &mut self,
        plans: &[serde_json::Value; 2],
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            if self.progress.recovery.is_some() {
                return Err(invalid());
            }
            self.check_plans(plans, end)?;
            if self.progress.lease_state != LeaseState::ReleasePrepared {
                return Err(invalid());
            }
            #[cfg(test)]
            if self.fault == 7 {
                return Err(io::Error::other(
                    "injected after explicit native release, before release-result event",
                ));
            }
            let receipt = self.released_lease.as_mut().ok_or_else(invalid)?;
            receipt.check(end)?;
            let facts: LeaseFacts =
                serde_json::from_slice(&encoded(receipt.facts())?).map_err(io_error)?;
            #[cfg(test)]
            {
                if self.fault == 8 {
                    self.fault = 1;
                } else if self.fault == 9 {
                    self.fault = 2;
                }
            }
            let event = self.lease_event(LeaseOperation::LeaseReleasedVerified, &facts, None)?;
            self.append_lease(event, end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn prepare_lease_acquire(
        &mut self,
        plans: &[serde_json::Value; 2],
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.check_plans(plans, end)?;
            if self.progress.lease_state != LeaseState::Released {
                return Err(invalid());
            }
            let receipt = self.released_lease.as_mut().ok_or_else(invalid)?;
            if self.progress.action_kind == super::super::actions::ActionKind::Start {
                receipt.check(end)?;
            } else {
                receipt.readback_history(end)?;
            }
            let facts: LeaseFacts =
                serde_json::from_slice(&encoded(receipt.facts())?).map_err(io_error)?;
            let e = self.lease_event(LeaseOperation::LeaseAcquirePrepared, &facts, None)?;
            self.append_lease(e, end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn acquire_prepared_lease(
        &mut self,
        owner: Option<&mut crate::infrastructure::windows_official_lifecycle024::Owner>,
        end: Instant,
    ) -> io::Result<NativeLease> {
        let result = (|| {
            self.check(end)?;
            if self.progress.lease_state != LeaseState::AcquirePrepared {
                return Err(invalid());
            }
            if self.progress.action_kind == super::super::actions::ActionKind::Start {
                self.released_lease
                    .as_mut()
                    .ok_or_else(invalid)?
                    .reacquire(owner, end)
            } else {
                let actual = owner.ok_or_else(invalid)?;
                let action = self.source_action.as_ref().ok_or_else(invalid)?;
                action.check(actual, end).map_err(io_error)?;
                self.released_lease
                    .as_mut()
                    .ok_or_else(invalid)?
                    .reacquire_for_action(action, actual, end)
            }
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn finish_lease_acquire(
        &mut self,
        plans: &[serde_json::Value; 2],
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.check_plans(plans, end)?;
            if self.progress.lease_state != LeaseState::AcquirePrepared {
                return Err(invalid());
            }
            let root = self.pins[0].path().join(".codex");
            let facts = claim.facts(&root, end)?;
            self.released_lease
                .as_mut()
                .ok_or_else(invalid)?
                .readback_history(end)?;
            #[cfg(test)]
            {
                if self.fault == 10 {
                    return Err(io::Error::other(
                        "injected after new native lease, before acquire-result event",
                    ));
                }
                if self.fault == 11 {
                    self.fault = 1;
                } else if self.fault == 12 {
                    self.fault = 2;
                }
            }
            let event = self.lease_event(
                LeaseOperation::LeaseAcquiredVerified,
                &facts,
                Some(facts.epoch.clone()),
            )?;
            self.append_lease(event, end)?;
            claim.check(&root, end)?;
            // Retain released original handles for the entire session history;
            // they cannot themselves be reused to acquire again.
            let receipt = self.released_lease.take().ok_or_else(invalid)?;
            self.previous_releases.push(receipt);
            Ok(())
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn lease_state_for_test(
        &self,
    ) -> String {
        format!("{:?}", self.progress.lease_state)
    }
}
fn io_error(e: impl std::fmt::Display) -> io::Error {
    io::Error::other(e.to_string())
}
