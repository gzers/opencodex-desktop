//! Actual BOTH-tree witnesses surround canonical intent/native/result writes.
//! Private driver until joined to original typed Stop/content qualification.
//! Only explicit synthetic component wrappers exist below under cfg(test).
use super::super::super::{coordination::AnchoredClaim, tree::RecoveryWitness};
use super::super::recovery::{RecoveryOperation, RecoveryPlan};
use super::*;
impl Journal {
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_reacquire_same_window_for_test(
        &mut self,
        plans: &[serde_json::Value; 2],
        end: Instant,
    ) -> io::Result<super::super::super::coordination::NativeLease> {
        if self.source_action.is_some() || end != self.deadline {
            return Err(invalid());
        }
        self.prepare_lease_acquire(plans, end)?;
        let mut lease = self
            .released_lease
            .as_mut()
            .ok_or_else(invalid)?
            .reacquire(None, end)?;
        let root = self.pins[0].path().join(".codex");
        let mut claim = lease.anchored_claim(&root, end)?;
        self.finish_lease_acquire(plans, &mut claim, end)?;
        drop(claim);
        Ok(lease)
    }
    fn recovery_native_check(
        &mut self,
        witness: &mut RecoveryWitness<'_>,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        self.check(end)?;
        claim.check(&self.pins[0].path().join(".codex"), end)?;
        if self.progress.lease_state != LeaseState::Active
            || self.progress.epoch != claim.epoch()
            || self.progress.claim != claim.key()
        {
            return Err(invalid());
        }
        let data = RecoveryPlan::from_native_data(witness.plans(claim, end)?)?;
        data.validate(&self.manifest.storage_plan, &self.progress)?;
        let digest = data.digest()?;
        if self
            .progress
            .recovery_summary()?
            .is_some_and(|s| s.plan_digest != digest)
        {
            return Err(invalid());
        }
        witness.check(claim, end)?;
        self.check(end)
    }
    fn prepare_native_recovery(
        &mut self,
        mut witness: RecoveryWitness<'_>,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.recovery_native_check(&mut witness, claim, end)?;
            let plan = RecoveryPlan::from_native_data(witness.plans(claim, end)?)?;
            let e = self.progress.recovery_event(
                RecoveryOperation::RecoveryPlanIntent,
                Some(&plan),
                None,
            )?;
            self.append_entry_inner(ChainEntry::Recovery(e), end)?;
            self.recovery_native_check(&mut witness, claim, end)?;
            #[cfg(test)]
            if self.fault == 44 {
                return Err(io::Error::other("injected after durable recovery plan intent/current before verified native plan result"));
            }
            let e = self.progress.recovery_event(
                RecoveryOperation::RecoveryPlanVerified,
                None,
                None,
            )?;
            self.append_entry_inner(ChainEntry::Recovery(e), end)?;
            self.recovery_native_check(&mut witness, claim, end)
        })();
        if result.is_err() {
            self.poisoned = true;
            witness.poison();
        }
        result
    }
    fn execute_native_recovery(
        &mut self,
        mut witness: RecoveryWitness<'_>,
        node: &str,
        retire: bool,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.recovery_native_check(&mut witness, claim, end)?;
            let (intent, completion) = if retire {
                (
                    RecoveryOperation::WorkRetireIntent,
                    RecoveryOperation::WorkRetiredVerified,
                )
            } else {
                (
                    RecoveryOperation::OriginalRestoreIntent,
                    RecoveryOperation::OriginalRestoredVerified,
                )
            };
            let e = self.progress.recovery_event(intent, None, Some(node))?;
            self.append_entry_inner(ChainEntry::Recovery(e), end)?;
            self.recovery_native_check(&mut witness, claim, end)?;
            #[cfg(test)]
            if self.fault == 40 {
                return Err(io::Error::other("injected after durable recovery node intent/current before native same-handle move"));
            }
            if retire {
                witness.retire(node, claim, end)?;
            } else {
                witness.restore(node, claim, end)?;
            }
            self.recovery_native_check(&mut witness, claim, end)?;
            #[cfg(test)]
            {
                if self.fault == 41 {
                    return Err(io::Error::other("injected after actual same-handle recovery move/readback before result publication"));
                }
                if self.fault == 42 {
                    self.fault = 1;
                } else if self.fault == 43 {
                    self.fault = 2;
                }
            }
            let e = self.progress.recovery_event(completion, None, Some(node))?;
            self.append_entry_inner(ChainEntry::Recovery(e), end)?;
            self.recovery_native_check(&mut witness, claim, end)
        })();
        if result.is_err() {
            self.poisoned = true;
            witness.poison();
        }
        result
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_prepare_recovery_for_test(
        &mut self,
        witness: RecoveryWitness<'_>,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        if self.source_action.is_some() {
            return Err(invalid());
        }
        self.prepare_native_recovery(witness, claim, end)
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_execute_recovery_for_test(
        &mut self,
        witness: RecoveryWitness<'_>,
        node: &str,
        retire: bool,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        if self.source_action.is_some() {
            return Err(invalid());
        }
        self.execute_native_recovery(witness, node, retire, claim, end)
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_recovery_state_for_test(
        &self,
    ) -> io::Result<serde_json::Value> {
        serde_json::to_value(self.progress.recovery_summary()?).map_err(|_| invalid())
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_recovery_trace_for_test(
        &self,
    ) -> io::Result<serde_json::Value> {
        let raw = std::fs::read(self.directory.path().join("current.json"))?;
        let mut entries = Vec::new();
        for record in &self.events {
            if let ChainEntry::Recovery(event) =
                serde_json::from_slice::<ChainEntry>(&record.raw).map_err(|_| invalid())?
            {
                entries.push(serde_json::to_value(event).map_err(|_| invalid())?);
            }
        }
        Ok(
            serde_json::json!({"directory":self.directory.path(),"sequence":self.progress.sequence,"poisoned":self.poisoned,
            "summary":self.component_recovery_state_for_test()?,"immutableRecoveryEvents":entries,
            "currentActualBytes":raw.len(),"currentActualSha256":hash::sha256_hex(&raw),
            "currentActual":serde_json::from_slice::<serde_json::Value>(&raw).ok(),"nativeAuthority":false}),
        )
    }
}
