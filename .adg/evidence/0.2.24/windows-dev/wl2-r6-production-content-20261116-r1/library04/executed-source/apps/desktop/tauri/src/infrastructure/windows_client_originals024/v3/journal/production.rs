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
    pub(in crate::infrastructure::windows_client_originals024) fn restore_qualified_stopped(
        &mut self,
        guard: &mut ProductionStop<'_>,
        assessment: &NativeContentAssessment,
        mut witness: RecoveryWitness<'_>,
        lease: &mut Option<NativeLease>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            guard.check(self, end)?;
            assessment.matches(guard.content, &guard.observations, end)?;
            let root = self.pins[0].path().join(".codex");
            let mut claim = lease
                .as_mut()
                .ok_or_else(invalid)?
                .anchored_claim(&root, end)?;
            self.prepare_native_recovery(witness.reborrow(), &mut claim, end)?;
            guard.check(self, end)?;
            for retire in [true, false] {
                guard.check(self, end)?;
                assessment.matches(guard.content, &guard.observations, end)?;
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
            assessment.matches(guard.content, &guard.observations, end)?;
            self.check(end)
        })();
        if result.is_err() {
            self.poisoned = true;
            witness.poison();
        }
        result
    }
}
