//! Read-only failed-Start canonical DATA. No old deadline changes, poison
//! clearing, event/current writes, deserialized native proofs or Closed grant.
use super::*;
mod join;
use crate::{
    infrastructure::{
        windows_client_originals024::{coordination::AnchoredClaim, source::Source},
        windows_official_lifecycle024::{OriginalActionKind, Owner},
    },
    modules::runtime::windows_lifecycle024_contract::BeforeResumeRecoveryWindow,
};
impl Journal {
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_read_before_resume_trees_for_test(
        &mut self,
        trees: &[serde_json::Value; 2],
        epoch: &str,
        key: &str,
        end: Instant,
    ) -> io::Result<serde_json::Value> {
        if self.source_action.is_some() {
            return Err(invalid());
        }
        let data = self.read_failed_start_records(end)?;
        self.match_before_resume_trees(trees, epoch, key)?;
        Ok(data)
    }
    fn read_failed_start_records(&mut self, end: Instant) -> io::Result<serde_json::Value> {
        let result = (|| {
            before(end)?;
            if self.events.len() != self.progress.sequence as usize
                || self.pending_source_action.is_some()
                || self.progress.lifecycle != super::super::snapshots::LifecyclePhase::Preparing
                || self.progress.action_kind != super::super::actions::ActionKind::Start
                || self.progress.action_pending.is_some()
                || self.progress.ready_generation.is_some()
                || self.progress.closed.is_some()
                || self.progress.recovery.is_some()
                || self.progress.history.is_some()
                || self.progress.handle_progress.is_some()
                || self.progress.content.is_some()
                || self.progress.route_close.is_some()
                || self.stop_completion.is_some()
                || self.close_completion.is_some()
            {
                return Err(invalid());
            }
            if let Some(released) = &mut self.released_lease {
                released.readback_history(end)?;
            }
            self.read_retained_records(end)?;
            let mut replay = Progress::new(
                &self.manifest.storage_plan,
                &self.manifest.session_id,
                &hash::sha256_hex(&self.manifest_record.raw),
                &hash::sha256_hex(&encoded(&self.manifest.binding)?),
                &self.manifest.initial_lease_epoch,
                &self.manifest.claim_key,
                &self.manifest.deadline_scope,
            )?;
            for record in &self.events {
                before(end)?;
                let event: ChainEntry =
                    serde_json::from_slice(&record.raw).map_err(|_| invalid())?;
                replay.apply_entry(&self.manifest.storage_plan, &event)?;
            }
            if encoded(&replay)? != encoded(&self.progress)? {
                return Err(invalid());
            }
            let data = serde_json::json!({
                "purpose":"before-resume-retained-canonical-read-data",
                "sessionId":self.manifest.session_id,
                "manifestDigest":hash::sha256_hex(&self.manifest_record.raw),
                "bindingDigest":self.progress.binding,
                "storagePlanDigest":self.progress.plan,
                "securityPolicyDigest":Policy::fixed().digest()?,
                "currentDigest":hash::sha256_hex(&self.current_raw),
                "sequence":self.progress.sequence,
                "eventDigest":self.progress.tip,
                "progress":self.progress,
                "plan":self.manifest.storage_plan,
                "poisoned":self.poisoned,
                "runtimeAuthority":false
            });
            self.read_retained_records(end)?;
            if let Some(released) = &mut self.released_lease {
                released.readback_history(end)?;
            }
            Ok(data)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn read_before_resume_prefix(
        &mut self,
        source: &mut Source,
        owner: &mut Owner,
        claim: &mut AnchoredClaim<'_>,
        window: &BeforeResumeRecoveryWindow,
    ) -> io::Result<serde_json::Value> {
        let result = (|| {
            let end = window.deadline();
            let root = self.pins[0].path().join(".codex");
            owner
                .check_before_resume_process(window)
                .map_err(io::Error::other)?;
            claim.check(&root, end)?;
            source.check(owner, end).map_err(io::Error::other)?;
            if source.binding_raw != encoded(&self.manifest.binding)? {
                return Err(invalid());
            }
            let original = self.source_action.as_ref().ok_or_else(invalid)?;
            if original.kind() != OriginalActionKind::Start
                || original.scope_id() != self.manifest.deadline_scope
            {
                return Err(invalid());
            }
            original
                .check_retained_owner_for_read(owner, end)
                .map_err(io::Error::other)?;
            let data = self.read_failed_start_records(end)?;
            source.check(owner, end).map_err(io::Error::other)?;
            self.source_action
                .as_ref()
                .unwrap()
                .check_retained_owner_for_read(owner, end)
                .map_err(io::Error::other)?;
            claim.check(&root, end)?;
            owner
                .check_before_resume_process(window)
                .map_err(io::Error::other)?;
            Ok(data)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn read_before_resume_trees(
        &mut self,
        source: &mut Source,
        owner: &mut Owner,
        claim: &mut AnchoredClaim<'_>,
        window: &BeforeResumeRecoveryWindow,
        trees: &[serde_json::Value; 2],
    ) -> io::Result<serde_json::Value> {
        let result = (|| {
            let canonical = self.read_before_resume_prefix(source, owner, claim, window)?;
            self.match_before_resume_trees(trees, claim.epoch(), &claim.key())?;
            before(window.deadline())?;
            Ok(canonical)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
}
#[cfg(test)]
mod tests;
