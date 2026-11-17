//! Whole-tree postconditions and exact final release in the SAME chain.
//! Private until actual Source/typed Stop/content qualification is joined.
use super::super::super::super::coordination::{LeaseFacts, NativeLease};
use super::super::super::segments::{LeaseOperation, LeaseState};
use super::*;
impl Journal {
    fn seal_native_recovery_tree(
        &mut self,
        witness: &mut RecoveryWitness<'_>,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        self.recovery_native_check(witness, claim, end)?;
        let event =
            self.progress
                .recovery_event(RecoveryOperation::WholeTreeRestoreIntent, None, None)?;
        self.append_entry_inner(ChainEntry::Recovery(event), end)?;
        self.recovery_native_check(witness, claim, end)?;
        #[cfg(test)]
        if self.fault == 50 {
            return Err(io::Error::other(
                "injected after whole-tree restore intent/current before final native proof",
            ));
        }
        witness.restored(claim, end)?;
        self.recovery_native_check(witness, claim, end)?;
        #[cfg(test)]
        {
            if self.fault == 51 {
                return Err(io::Error::other(
                    "injected after whole-tree native proof before result publication",
                ));
            }
            if self.fault == 52 {
                self.fault = 1;
            } else if self.fault == 53 {
                self.fault = 2;
            }
        }
        let event = self.progress.recovery_event(
            RecoveryOperation::WholeTreeRestoredVerified,
            None,
            None,
        )?;
        self.append_entry_inner(ChainEntry::Recovery(event), end)?;
        witness.restored(claim, end)?;
        self.recovery_native_check(witness, claim, end)
    }
    pub(in crate::infrastructure::windows_client_originals024::v3::journal) fn release_native_recovery(
        &mut self,
        mut witness: RecoveryWitness<'_>,
        lease: &mut Option<NativeLease>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            let root = self.pins[0].path().join(".codex");
            let mut claim = lease
                .as_mut()
                .ok_or_else(invalid)?
                .anchored_claim(&root, end)?;
            self.seal_native_recovery_tree(&mut witness, &mut claim, end)?;
            if !self
                .progress
                .recovery_tree_ready_for_release(&self.manifest.storage_plan)
                || self.released_lease.is_some()
            {
                return Err(invalid());
            }
            witness.restored(&mut claim, end)?;
            let facts = claim.facts(&root, end)?;
            let event = self.lease_event(LeaseOperation::LeaseReleasePrepared, &facts, None)?;
            self.append_lease(event, end)?;
            witness.restored(&mut claim, end)?;
            #[cfg(test)]
            if self.fault == 54 {
                return Err(io::Error::other(
                    "injected after durable final lease preparation before actual explicit unlock",
                ));
            }
            drop(claim);
            let receipt = lease
                .take()
                .ok_or_else(invalid)?
                .release_with_receipt(end)?;
            // Retain immediately, before ANY fallible post-unlock readback.
            self.retain_lease_release(receipt)?;
            #[cfg(test)]
            if self.fault == 61 {
                witness.foreign_custody_for_test(end)?;
            }
            #[cfg(test)]
            if self.fault == 55 {
                return Err(io::Error::other("injected after actual final unlock/retained receipt before native postconditions"));
            }
            self.final_release_postconditions(&mut witness, end)?;
            if self.progress.uses_node_batches() {
                self.publish_released_anchor_batch(&mut witness, end)?;
            } else {
                let anchors = self
                    .manifest
                    .storage_plan
                    .roots
                    .iter()
                    .flat_map(|r| &r.nodes)
                    .filter(|n| n.kind != Kind::CustodiedFile)
                    .map(|n| n.node_id.clone())
                    .collect::<Vec<_>>();
                for node in anchors {
                    // Each anchor gets its own immutable ReleaseVerified event,
                    // only after actual unlock and stable range/full-tree proof.
                    self.final_release_postconditions(&mut witness, end)?;
                    #[cfg(test)]
                    {
                        if self.fault == 59 {
                            self.fault = 1;
                        } else if self.fault == 60 {
                            self.fault = 2;
                        }
                    }
                    let event = self.progress.recovery_event(
                        RecoveryOperation::ReleaseVerified,
                        None,
                        Some(&node),
                    )?;
                    self.append_entry_inner(ChainEntry::Recovery(event), end)?;
                    self.final_release_postconditions(&mut witness, end)?;
                }
            }
            #[cfg(test)]
            {
                if self.fault == 56 {
                    return Err(io::Error::other("injected after actual final unlock and native postconditions before release result"));
                }
                if self.fault == 57 {
                    self.fault = 1;
                } else if self.fault == 58 {
                    self.fault = 2;
                }
            }
            let receipt = self.released_lease.as_mut().ok_or_else(invalid)?;
            receipt.check(end)?;
            let facts: LeaseFacts =
                serde_json::from_slice(&encoded(receipt.facts())?).map_err(|_| invalid())?;
            let event = self.lease_event(LeaseOperation::LeaseReleasedVerified, &facts, None)?;
            self.append_lease(event, end)?;
            self.final_release_postconditions(&mut witness, end)
        })();
        if result.is_err() {
            self.poisoned = true;
            witness.poison();
        }
        result
    }
    fn final_release_postconditions(
        &mut self,
        witness: &mut RecoveryWitness<'_>,
        end: Instant,
    ) -> io::Result<()> {
        self.check(end)?;
        if !matches!(
            self.progress.lease_state,
            LeaseState::ReleasePrepared | LeaseState::Released
        ) {
            return Err(invalid());
        }
        self.released_lease
            .as_mut()
            .ok_or_else(invalid)?
            .check(end)?;
        witness.post_release(end)?;
        self.released_lease
            .as_mut()
            .ok_or_else(invalid)?
            .check(end)?;
        self.check(end)
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_release_recovery_for_test(
        &mut self,
        witness: RecoveryWitness<'_>,
        lease: &mut Option<NativeLease>,
        end: Instant,
    ) -> io::Result<()> {
        if self.source_action.is_some() || end != self.deadline {
            return Err(invalid());
        }
        self.release_native_recovery(witness, lease, end)
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_final_release_trace_for_test(
        &self,
    ) -> io::Result<serde_json::Value> {
        let mut base = self.component_recovery_trace_for_test()?;
        base["leaseState"] =
            serde_json::to_value(self.progress.lease_state).map_err(|_| invalid())?;
        base["actualReleaseReceiptRetained"] = self.released_lease.is_some().into();
        base["completeFilesDataOnly"] = self
            .progress
            .complete_files(&self.manifest.storage_plan)
            .into();
        base["immutableLeaseEvents"] = serde_json::json!(self
            .events
            .iter()
            .map(|r| serde_json::from_slice::<ChainEntry>(&r.raw).map_err(|_| invalid()))
            .collect::<io::Result<Vec<_>>>()?
            .into_iter()
            .filter_map(|e| if let ChainEntry::Lease(e) = e {
                Some(serde_json::to_value(e).map_err(|_| invalid()))
            } else {
                None
            })
            .collect::<io::Result<Vec<_>>>()?);
        Ok(base)
    }
}
