//! Private original-slot binding for the explicit launcher-quiescence stage.
//! All native objects remain owned by the original slot/Owner. No serde/Clone,
//! PID lookup, old manifest adoption, NormalExit or Closed constructor exists.
use super::*;
use crate::{
    infrastructure::{hash, windows_client_originals024::coordination::AnchoredClaim},
    modules::runtime::windows_lifecycle024_contract::BeforeResumeRecoveryWindow,
};
pub(in crate::infrastructure::windows_client_originals024) mod attachment;
pub(crate) struct PreparedAbortPrefix {
    session: uuid::Uuid,
    scope: String,
    action: String,
    end: Instant,
    epoch: String,
    claim: String,
    digest: String,
    observed: serde_json::Value,
}
// A borrow of the SAME actual claim spans the native call. The only constructor
// is this original adapter's checked local critical section. Not serde/Clone.
pub(crate) struct ClaimBoundAbort<'a, 'lease> {
    prefix: &'a PreparedAbortPrefix,
    claim: &'a mut AnchoredClaim<'lease>,
}
impl ClaimBoundAbort<'_, '_> {
    pub(crate) fn check_owner(
        &mut self,
        owner: &mut Owner,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<()> {
        self.prefix.check_owner(owner, window)?;
        self.claim
            .check(&owner.profile_path().join(".codex"), window.deadline())
            .map_err(refused)?;
        if self.claim.epoch() != self.prefix.epoch || self.claim.key() != self.prefix.claim {
            return Err(refused("不是原绑定的实际claim"));
        }
        Ok(())
    }
}
/// Original native forced proof stays in Owner; this tag locates only the same
/// original verified prefix. It is deliberately incompatible with Closed.
pub(crate) struct QuiescentOriginals {
    session: uuid::Uuid,
    action: String,
    end: Instant,
    prefix_digest: String,
}
pub(crate) struct AbortedOriginals {
    session: uuid::Uuid,
    action: String,
    end: Instant,
    native: crate::infrastructure::windows_client_originals024::tree::AbortRestoredReadback,
}
impl AbortedOriginals {
    pub(crate) fn read_data(&self, end: Instant) -> AppResult<serde_json::Value> {
        let data = self.native.data(end).map_err(refused)?;
        if end != self.end {
            return Err(refused("原Aborted读回截止不符"));
        }
        Ok(
            serde_json::json!({"sessionId":self.session.to_string(),"recoveryActionId":self.action,"restored":data,"normalStop":false,"closed":false,"publicStartOpen":false}),
        )
    }
}
fn data_digest(data: &serde_json::Value) -> AppResult<String> {
    serde_json::to_vec(data)
        .map(|bytes| hash::sha256_hex(&bytes))
        .map_err(refused)
}
impl QuiescentOriginals {
    fn check_tag(&self, prefix: &PreparedAbortPrefix) -> AppResult<()> {
        if self.session != prefix.session
            || self.action != prefix.action
            || self.end != prefix.end
            || self.prefix_digest != prefix.digest
        {
            return Err(refused("外来quiescent prefix"));
        }
        Ok(())
    }
}
impl PreparedAbortPrefix {
    pub(crate) fn check_owner(
        &self,
        owner: &mut Owner,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<()> {
        owner.check_before_resume_process(window)?;
        if self.scope != owner.scope()?
            || self.action != owner.before_resume_action_id(window)?
            || self.end != window.deadline()
            || self.digest != data_digest(&self.observed)?
        {
            return Err(refused("原前缀凭证/恢复动作/截止与原Owner不符"));
        }
        Ok(())
    }
    fn check_read(
        &self,
        session: Option<uuid::Uuid>,
        epoch: &str,
        claim: &str,
        end: Instant,
        data: &serde_json::Value,
    ) -> AppResult<()> {
        if Instant::now() >= end
            || session != Some(self.session)
            || epoch != self.epoch
            || claim != self.claim
            || end != self.end
            || data != &self.observed
            || self.digest != data_digest(data)?
        {
            return Err(refused("不是同原slot/claim/完整前后像"));
        }
        Ok(())
    }
}
impl WindowsOriginals {
    pub(crate) fn recheck_retained_quiescent(
        &mut self,
        owner: &mut Owner,
        released: &mut super::super::coordination::ReleasedLease,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<serde_json::Value> {
        let result = (|| {
            let prefix = self
                .abort_prefix
                .as_ref()
                .ok_or_else(|| refused("原返回前缀缺席"))?;
            let proof = self
                .quiescent_for_release
                .as_ref()
                .ok_or_else(|| refused("原静止凭证缺席"))?;
            proof.check_tag(prefix)?;
            if self.session != Some(prefix.session)
                || self.owner_scope.as_ref() != Some(&owner.scope()?)
            {
                return Err(refused("不是原返回槽"));
            }
            let attachment = self
                .abort_attachment
                .as_mut()
                .ok_or_else(|| refused("原附件缺席"))?;
            let native = self
                .slot
                .recheck_before_resume_return(owner, prefix, proof, attachment, released, window)?;
            Ok(
                serde_json::json!({"sessionId":prefix.session.to_string(),"recoveryActionId":prefix.action,"restored":native,"normalStop":false,"closed":false,"publicStartOpen":false}),
            )
        })();
        if result.is_err() {
            self.failed = true;
            owner.fail_before_resume_process();
        }
        result
    }
    pub(crate) fn retain_quiescent_for_release(
        &mut self,
        owner: &mut Owner,
        proof: QuiescentOriginals,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<()> {
        if self.quiescent_for_release.is_some() {
            return Err(refused("原释放阶段已留位"));
        }
        self.quiescent_for_release = Some(proof);
        owner.check_before_resume_quiescent(window)
    }
    pub(crate) fn finish_retained_quiescent(
        &mut self,
        owner: &mut Owner,
        released: &mut super::super::coordination::ReleasedLease,
        probe: &mut super::super::coordination::ReleaseProbe,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<AbortedOriginals> {
        let proof = self
            .quiescent_for_release
            .as_ref()
            .ok_or_else(|| refused("原静止释放凭证缺席"))?;
        let prefix = self
            .abort_prefix
            .as_ref()
            .ok_or_else(|| refused("原前缀缺席"))?;
        proof.check_tag(prefix)?;
        let native = self.slot.finish_before_resume_release(
            owner,
            prefix,
            proof,
            self.abort_attachment
                .as_mut()
                .ok_or_else(|| refused("原附件缺席"))?,
            released,
            probe,
            window,
        )?;
        Ok(AbortedOriginals {
            session: prefix.session,
            action: prefix.action.clone(),
            end: window.deadline(),
            native,
        })
    }
    pub(crate) fn finish_released_originals(
        &mut self,
        owner: &mut Owner,
        proof: &QuiescentOriginals,
        released: &mut super::super::coordination::ReleasedLease,
        probe: &mut super::super::coordination::ReleaseProbe,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<AbortedOriginals> {
        let result = (|| {
            owner.check_before_resume_quiescent(window)?;
            let prefix = self
                .abort_prefix
                .as_ref()
                .ok_or_else(|| refused("原prefix缺席"))?;
            proof.check_tag(prefix)?;
            let native = self.slot.finish_before_resume_release(
                owner,
                prefix,
                proof,
                self.abort_attachment
                    .as_mut()
                    .ok_or_else(|| refused("原附件缺席"))?,
                released,
                probe,
                window,
            )?;
            Ok(AbortedOriginals {
                session: prefix.session,
                action: prefix.action.clone(),
                end: window.deadline(),
                native,
            })
        })();
        if result.is_err() {
            self.failed = true;
            owner.fail_before_resume_process();
        }
        result
    }
    pub(crate) fn return_quiescent_originals(
        &mut self,
        owner: &mut Owner,
        proof: &QuiescentOriginals,
        claim: &mut AnchoredClaim<'_>,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<serde_json::Value> {
        let result = (|| {
            self.check_quiescent_prefix(owner, proof, claim, window)?;
            let prefix = self
                .abort_prefix
                .as_ref()
                .ok_or_else(|| refused("原quiescent prefix缺席"))?;
            let attachment = self
                .abort_attachment
                .as_mut()
                .ok_or_else(|| refused("原附件缺席"))?;
            self.slot
                .return_before_resume(owner, prefix, proof, attachment, claim, window)
        })();
        if result.is_err() {
            self.failed = true;
            owner.fail_before_resume_process();
        }
        result
    }
    pub(crate) fn prepare_abort_prefix(
        &mut self,
        owner: &mut Owner,
        claim: &mut AnchoredClaim<'_>,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<PreparedAbortPrefix> {
        if self.abort_prefix.is_some() {
            return Err(refused("原quiesce动作已尝试；不重置"));
        }
        let observed = self.read_before_resume_prefix(owner, claim, window)?;
        let prefix = PreparedAbortPrefix {
            session: self.session.ok_or_else(|| refused("原slot session缺席"))?,
            scope: owner.scope()?,
            action: owner.before_resume_action_id(window)?,
            end: window.deadline(),
            epoch: claim.epoch().into(),
            claim: claim.key(),
            digest: data_digest(&observed)?,
            observed,
        };
        prefix.check_owner(owner, window)?;
        Ok(prefix)
    }
    pub(crate) fn quiesce_abort_prefix(
        &mut self,
        owner: &mut Owner,
        prefix: PreparedAbortPrefix,
        claim: &mut AnchoredClaim<'_>,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<QuiescentOriginals> {
        if self.abort_prefix.is_some() {
            return Err(refused("原quiesce前缀已留位；不重试"));
        }
        // Install before every fallible admission/native step. Failure preserves
        // the original local prefix and all native objects; no replacement.
        self.abort_prefix = Some(prefix);
        let result = (|| {
            let actual = self.read_before_resume_prefix(owner, claim, window)?;
            let prefix = self.abort_prefix.as_ref().unwrap();
            prefix.check_owner(owner, window)?;
            prefix.check_read(
                self.session,
                claim.epoch(),
                &claim.key(),
                window.deadline(),
                &actual,
            )?;
            self.abort_attachment
                .as_mut()
                .ok_or_else(|| refused("原恢复附件未先持久化Intent"))?
                .check(owner, prefix, window)?;
            let mut bound = ClaimBoundAbort {
                prefix,
                claim: &mut *claim,
            };
            owner.quiesce_before_resume_process(&mut bound, window)?;
            self.abort_attachment
                .as_mut()
                .unwrap()
                .record_quiescent(owner, prefix, window)?;
            // The SAME actual claim is never released/reacquired here. Native
            // original quiescence plus unchanged complete original prefix must
            // both hold; forced is still not a normal service exit.
            let after = self.read_before_resume_prefix(owner, claim, window)?;
            let prefix = self.abort_prefix.as_ref().unwrap();
            prefix.check_owner(owner, window)?;
            prefix.check_read(
                self.session,
                claim.epoch(),
                &claim.key(),
                window.deadline(),
                &after,
            )?;
            owner.check_before_resume_quiescent(window)?;
            Ok(QuiescentOriginals {
                session: prefix.session,
                action: prefix.action.clone(),
                end: prefix.end,
                prefix_digest: prefix.digest.clone(),
            })
        })();
        if result.is_err() {
            self.failed = true;
            owner.fail_before_resume_process();
        }
        result
    }
    pub(crate) fn check_quiescent_prefix(
        &mut self,
        owner: &mut Owner,
        proof: &QuiescentOriginals,
        claim: &mut AnchoredClaim<'_>,
        window: &BeforeResumeRecoveryWindow,
    ) -> AppResult<()> {
        let result = (|| {
            owner.check_before_resume_quiescent(window)?;
            let actual = self.read_before_resume_prefix(owner, claim, window)?;
            let prefix = self
                .abort_prefix
                .as_ref()
                .ok_or_else(|| refused("原quiescent前缀缺席"))?;
            proof.check_tag(prefix)?;
            self.abort_attachment
                .as_mut()
                .ok_or_else(|| refused("原恢复附件缺席"))?
                .check(owner, prefix, window)?;
            prefix.check_read(
                self.session,
                claim.epoch(),
                &claim.key(),
                window.deadline(),
                &actual,
            )
        })();
        if result.is_err() {
            self.failed = true;
            owner.fail_before_resume_process();
        }
        result
    }
}
#[cfg(test)]
mod tests;
