//! Original actual typed Stop scope. No parsed/synthetic constructor, no
//! original-process adoption or permission from a content-policy bool.
use super::super::super::{
    content::StoppedContentScope,
    coordination::{AnchoredClaim, NativeLease},
    source::Source,
    tree::{NativeContentAssessment, RecoveryWitness},
    v2::routes::RoutePin,
};
use super::*;
use crate::{
    infrastructure::{
        windows_diagnostic_roles024::NormalExit, windows_official_lifecycle024::Owner,
        windows_supervisor024::Absent,
    },
    modules::runtime::official_lifecycle024::StopAccepted,
};
pub(in crate::infrastructure::windows_client_originals024) struct ProductionStop<'a> {
    source: &'a mut Source,
    owner: &'a mut Owner,
    normal: &'a NormalExit,
    absent: &'a Absent,
    official: &'a StopAccepted,
    routes: &'a mut RoutePin,
    content: &'a StoppedContentScope,
    observations: ObservationSeriesContext,
}
impl<'a> ProductionStop<'a> {
    pub(in crate::infrastructure::windows_client_originals024) fn bind(
        journal: &mut Journal,
        source: &'a mut Source,
        owner: &'a mut Owner,
        normal: &'a NormalExit,
        absent: &'a Absent,
        official: &'a StopAccepted,
        routes: &'a mut RoutePin,
        content: &'a StoppedContentScope,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<Self> {
        content.check(source, owner, normal, routes, end)?;
        let observations = journal
            .stopped_content_observations(source, owner, normal, absent, official, claim, end)?;
        let mut guard = Self {
            source,
            owner,
            normal,
            absent,
            official,
            routes,
            content,
            observations,
        };
        guard.check(journal, end)?;
        Ok(guard)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn observations(
        &self,
    ) -> &ObservationSeriesContext {
        &self.observations
    }
    pub(in crate::infrastructure::windows_client_originals024) fn check(
        &mut self,
        journal: &mut Journal,
        end: Instant,
    ) -> io::Result<()> {
        self.content
            .check(self.source, self.owner, self.normal, self.routes, end)?;
        journal.stopped_source(
            self.source,
            self.owner,
            self.normal,
            self.absent,
            self.official,
            end,
        )?;
        if self.content.deadline() != end {
            return Err(invalid());
        }
        before(end)
    }
}
impl Journal {
    pub(in crate::infrastructure::windows_client_originals024) fn close_qualified_stopped_originals(
        &mut self,
        guard: &mut ProductionStop<'_>,
        assessment: &NativeContentAssessment,
        trees: &mut Option<[super::super::super::tree::AnchoredTree; 2]>,
        end: Instant,
    ) -> io::Result<()> {
        // Install exact ownership before guard checks, native readback or any
        // possible deadline failure. A failed/expired close retains its trees,
        // held/closed/unknown inventory and canonical prefix in this journal.
        let result = (|| {
            self.retain_original_handle_ownership(trees, end)?;
            self.check_stopped_content(guard, assessment, end)?;
            self.close_original_handles(end)?;
            self.check_stopped_content(guard, assessment, end)?;
            if self.close_completion.is_none() {
                return Err(invalid());
            }
            before(end)
        })();
        if result.is_err() {
            self.poison_segment();
        }
        result
    }
    /// The intent/result both require the same ACTUAL typed guard and private
    /// native assessment. The caller re-runs the complete BOTH-tree barrier
    /// between these records, before transferring any frozen current handle.
    pub(in crate::infrastructure::windows_client_originals024) fn record_stopped_content(
        &mut self,
        guard: &mut ProductionStop<'_>,
        assessment: &NativeContentAssessment,
        verified: bool,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        use super::super::qualification::{ContentData, ContentEvent, ContentOperation};
        let result = (|| {
            guard.check(self, end)?;
            assessment.matches(guard.content, &guard.observations, end)?;
            self.snapshot_claim(claim, end)?;
            let content: ContentData =
                serde_json::from_value(assessment.qualification_data(&guard.observations)?)
                    .map_err(|_| invalid())?;
            let e = ContentEvent {
                schema: ANCHORED_ORIGINALS_SCHEMA_V3.into(),
                document_kind: "content-qualification-event".into(),
                storage_strategy: ANCHORED_STORAGE_STRATEGY.into(),
                event_id: uuid::Uuid::new_v4().to_string(),
                sequence: self.progress.sequence.checked_add(1).ok_or_else(invalid)?,
                session_id: self.progress.session.clone(),
                operation: if verified {
                    ContentOperation::ContentQualifiedVerified
                } else {
                    ContentOperation::ContentQualificationIntent
                },
                content,
                lease_epoch: self.progress.epoch.clone(),
                claim_key: self.progress.claim.clone(),
                deadline_scope: self.progress.deadline.clone(),
                previous_event_digest: self.progress.tip.clone(),
                storage_plan_digest: self.progress.plan.clone(),
                security_policy_digest: Policy::fixed().digest()?,
                manifest_digest: self.progress.manifest.clone(),
                binding_digest: self.progress.binding.clone(),
            };
            self.append_entry_inner(ChainEntry::Content(e), end)?;
            guard.check(self, end)?;
            assessment.matches(guard.content, &guard.observations, end)?;
            self.snapshot_claim(claim, end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    fn check_stopped_content(
        &mut self,
        guard: &mut ProductionStop<'_>,
        assessment: &NativeContentAssessment,
        end: Instant,
    ) -> io::Result<()> {
        guard.check(self, end)?;
        assessment.matches(guard.content, &guard.observations, end)?;
        let value = assessment.qualification_data(&guard.observations)?;
        let content: super::super::qualification::ContentData =
            serde_json::from_value(value).map_err(|_| invalid())?;
        if self.progress.content.as_ref()
            != Some(&super::super::qualification::ContentSummary {
                digest: content.digest()?,
                verified: true,
            })
        {
            return Err(invalid());
        }
        self.check(end)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn restore_qualified_stopped(
        &mut self,
        guard: &mut ProductionStop<'_>,
        assessment: &NativeContentAssessment,
        mut witness: RecoveryWitness<'_>,
        lease: &mut Option<NativeLease>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.check_stopped_content(guard, assessment, end)?;
            let root = self.pins[0].path().join(".codex");
            let mut claim = lease
                .as_mut()
                .ok_or_else(invalid)?
                .anchored_claim(&root, end)?;
            self.prepare_native_recovery(witness.reborrow(), &mut claim, end)?;
            guard.check(self, end)?;
            for retire in [true, false] {
                self.check_stopped_content(guard, assessment, end)?;
                self.recover_native_batch(witness.reborrow(), retire, &mut claim, end)?;
                guard.check(self, end)?;
            }
            witness.restored(&mut claim, end)?;
            drop(claim);
            guard.check(self, end)?;
            self.release_native_recovery(witness.reborrow(), lease, end)?;
            guard.check(self, end)?;
            self.seal_original_history(witness.reborrow(), end)?;
            guard.check(self, end)?;
            self.check_stopped_content(guard, assessment, end)?;
            self.check(end)
        })();
        if result.is_err() {
            self.poisoned = true;
            witness.poison();
        }
        result
    }
}
