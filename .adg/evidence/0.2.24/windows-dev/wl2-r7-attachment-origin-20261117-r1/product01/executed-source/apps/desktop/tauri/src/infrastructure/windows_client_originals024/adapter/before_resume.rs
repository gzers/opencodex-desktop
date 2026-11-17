//! Private original-slot binding for the explicit launcher-quiescence stage.
//! All native objects remain owned by the original slot/Owner. No serde/Clone,
//! PID lookup, old manifest adoption, NormalExit or Closed constructor exists.
use super::*;
use crate::{
    infrastructure::{hash, windows_client_originals024::coordination::AnchoredClaim},
    modules::runtime::windows_lifecycle024_contract::BeforeResumeRecoveryWindow,
};
pub(super) mod attachment;
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
