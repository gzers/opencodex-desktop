//! Retained complete original trees+journal installed BEFORE client mutation.
//! Source/current/route/original Owner authority belongs to the outer store.
use super::super::super::{
    coordination::NativeLease,
    v3::{FileAction, Journal},
};
use super::*;
/// Retained stop-boundary locator. Data fields alone do not authorize any
/// mutation; actual Source/Owner/typed exit checks surround every native step.
struct PreparedStop {
    pid: u32,
    port: u16,
    record: String,
    deadline: Instant,
}
impl PreparedStop {
    fn admit(
        pid: u32,
        port: u16,
        record: &str,
        kind: crate::infrastructure::windows_official_lifecycle024::OriginalActionKind,
        end: Instant,
    ) -> io::Result<Self> {
        use crate::infrastructure::windows_official_lifecycle024::OriginalActionKind;
        check(end)?;
        if !matches!(kind, OriginalActionKind::Stop | OriginalActionKind::Restart)
            || pid == 0
            || port == 0
            || !super::super::super::phase::digest(record)
        {
            return Err(invalid());
        }
        Ok(Self {
            pid,
            port,
            record: record.into(),
            deadline: end,
        })
    }
    fn check(&self, pid: u32, port: u16, record: &str, end: Instant) -> io::Result<()> {
        if self.deadline != end || self.pid != pid || self.port != port || self.record != record {
            return Err(invalid());
        }
        check(end)
    }
}
pub(in crate::infrastructure::windows_client_originals024) struct CapturedTrees {
    trees: [AnchoredTree; 2],
    journal: Journal,
    prepared: bool,
    opened: bool,
    ready: bool,
    prepared_stop: Option<PreparedStop>,
    stopped_observed: bool,
    failed: bool,
}
impl CapturedTrees {
    /// Native planning DATA only. Current and original handles remain stored;
    /// no new retirement step is called here or authorized from the returned
    /// plan. Canonical per-row progress + exact typed recovery scope still next.
    pub(in crate::infrastructure::windows_client_originals024) fn inspect_original_retirement(
        &mut self,
        source: &mut crate::infrastructure::windows_client_originals024::source::Source,
        owner: &mut crate::infrastructure::windows_official_lifecycle024::Owner,
        normal: &crate::infrastructure::windows_diagnostic_roles024::NormalExit,
        absent: &crate::infrastructure::windows_supervisor024::Absent,
        official: &crate::modules::runtime::official_lifecycle024::StopAccepted,
        lease: &mut NativeLease,
        end: Instant,
    ) -> io::Result<serde_json::Value> {
        use crate::modules::runtime::windows_lifecycle024_contract::SupervisorAbsence;
        let result = (|| {
            if self.failed || !self.stopped_observed {
                return Err(invalid());
            }
            self.prepared_stop.as_ref().ok_or_else(invalid)?.check(
                official.pid(),
                official.port(),
                official.record_sha256(),
                end,
            )?;
            source.check(owner, end).map_err(io::Error::other)?;
            self.journal.check_source_owner(owner, end)?;
            normal.check(owner, end).map_err(io::Error::other)?;
            crate::infrastructure::windows_supervisor024::WindowsSupervisor
                .recheck_absent(owner, absent, end)
                .map_err(io::Error::other)?;
            owner
                .originals_check_stopped_records(end)
                .map_err(io::Error::other)?;
            let root = self.trees[0].parent.path().join(".codex");
            let mut claim = lease.anchored_claim(&root, end)?;
            let observations = self.journal.stopped_content_observations(
                source, owner, normal, absent, official, &mut claim, end,
            )?;
            let plans = [
                self.trees[0].capture_retirement(&mut claim, end)?,
                self.trees[1].capture_retirement(&mut claim, end)?,
            ];
            source.check(owner, end).map_err(io::Error::other)?;
            normal.check(owner, end).map_err(io::Error::other)?;
            crate::infrastructure::windows_supervisor024::WindowsSupervisor
                .recheck_absent(owner, absent, end)
                .map_err(io::Error::other)?;
            owner
                .originals_check_stopped_records(end)
                .map_err(io::Error::other)?;
            self.journal.check_source_owner(owner, end)?;
            claim.check(&root, end)?;
            Ok(
                serde_json::json!({"schema":"windows-anchored-stopped-retirement-preview-v3","roots":plans,"observationSequences":observations.sequences(),"stageContexts":[observations.binding(0)?,observations.binding(1)?,observations.binding(2)?],"contentPolicyAssessed":false,"mutationAuthority":false,"ClosedEligibility":false}),
            )
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn prepare_original_stop(
        &mut self,
        source: &mut crate::infrastructure::windows_client_originals024::source::Source,
        owner: &mut crate::infrastructure::windows_official_lifecycle024::Owner,
        ready: &crate::modules::runtime::official_lifecycle024::ServiceObservation,
        kind: crate::infrastructure::windows_official_lifecycle024::OriginalActionKind,
        lease: &mut Option<NativeLease>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            if self.failed
                || !self.prepared
                || !self.opened
                || !self.ready
                || self.prepared_stop.is_some()
                || self.stopped_observed
                || lease.is_some()
            {
                return Err(invalid());
            }
            let intent =
                PreparedStop::admit(ready.pid(), ready.port(), ready.record_sha256(), kind, end)?;
            source.check(owner, end).map_err(io::Error::other)?;
            owner
                .originals_check_ready(ready, end)
                .map_err(io::Error::other)?;
            self.journal
                .bind_original_stop_action(source, owner, ready, kind, end)?;
            self.journal.check_source_owner(owner, end)?;
            super::observation::reacquire_for_observation(
                &mut self.trees,
                &mut self.journal,
                lease,
                Some(owner),
                end,
            )?;
            let root = self.trees[0].parent.path().join(".codex");
            let mut claim = lease
                .as_mut()
                .ok_or_else(invalid)?
                .anchored_claim(&root, end)?;
            super::observation::observe_trees(&mut self.trees, &mut self.journal, &mut claim, end)?;
            let context = self
                .journal
                .begin_pre_stop_snapshot_link(source, owner, ready, &mut claim, end)?;
            super::observation::snapshots::create_snapshots(
                &mut self.trees,
                &mut self.journal,
                &context,
                &mut claim,
                false,
                end,
            )?;
            let witness = super::observation::snapshots::witness(&mut self.trees, end)?;
            self.journal
                .record_pre_stop_phase(source, owner, ready, witness, &mut claim, end)?;
            self.journal
                .record_teardown_prepared(source, owner, ready, &mut claim, end)?;
            super::observation::series::resume_trees(
                &mut self.trees,
                &mut self.journal,
                &mut claim,
                end,
            )?;
            drop(claim);
            // Release completed capture critical section before official code
            // may itself acquire claim. Never await teardown with our lock held.
            super::segments::release_trees(&mut self.trees, &mut self.journal, lease, end)?;
            source.check(owner, end).map_err(io::Error::other)?;
            owner
                .originals_check_ready(ready, end)
                .map_err(io::Error::other)?;
            self.journal.check_source_owner(owner, end)?;
            intent.check(ready.pid(), ready.port(), ready.record_sha256(), end)?;
            self.prepared_stop = Some(intent);
            Ok(())
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
    /// Continues the ORIGINAL stop window, leaves actual NativeLease installed
    /// on caller stack for exact restoration in the SAME critical section.
    /// A returned observation context is DATA; no successful Stop or Closed.
    pub(in crate::infrastructure::windows_client_originals024) fn observe_original_stopped(
        &mut self,
        source: &mut crate::infrastructure::windows_client_originals024::source::Source,
        owner: &mut crate::infrastructure::windows_official_lifecycle024::Owner,
        normal: &crate::infrastructure::windows_diagnostic_roles024::NormalExit,
        absent: &crate::infrastructure::windows_supervisor024::Absent,
        official: &crate::modules::runtime::official_lifecycle024::StopAccepted,
        lease: &mut Option<NativeLease>,
        end: Instant,
    ) -> io::Result<
        crate::infrastructure::windows_client_originals024::v3::journal::ObservationSeriesContext,
    > {
        use crate::modules::runtime::windows_lifecycle024_contract::SupervisorAbsence;
        let result = (|| {
            if self.failed || !self.ready || self.stopped_observed || lease.is_some() {
                return Err(invalid());
            }
            self.prepared_stop.as_ref().ok_or_else(invalid)?.check(
                official.pid(),
                official.port(),
                official.record_sha256(),
                end,
            )?;
            source.check(owner, end).map_err(io::Error::other)?;
            self.journal.check_source_owner(owner, end)?;
            normal.check(owner, end).map_err(io::Error::other)?;
            crate::infrastructure::windows_supervisor024::WindowsSupervisor
                .recheck_absent(owner, absent, end)
                .map_err(io::Error::other)?;
            owner
                .originals_check_stopped_records(end)
                .map_err(io::Error::other)?;
            super::observation::reacquire_for_observation(
                &mut self.trees,
                &mut self.journal,
                lease,
                Some(owner),
                end,
            )?;
            let root = self.trees[0].parent.path().join(".codex");
            let mut claim = lease
                .as_mut()
                .ok_or_else(invalid)?
                .anchored_claim(&root, end)?;
            super::observation::observe_trees(&mut self.trees, &mut self.journal, &mut claim, end)?;
            let context = self.journal.begin_stopped_snapshot_link(
                source, owner, normal, absent, official, &mut claim, end,
            )?;
            super::observation::snapshots::create_snapshots(
                &mut self.trees,
                &mut self.journal,
                &context,
                &mut claim,
                false,
                end,
            )?;
            let witness = super::observation::snapshots::witness(&mut self.trees, end)?;
            self.journal.record_stopped_phase(
                source, owner, normal, absent, official, witness, &mut claim, end,
            )?;
            let observations = self.journal.stopped_content_observations(
                source, owner, normal, absent, official, &mut claim, end,
            )?;
            self.prepared_stop.as_ref().ok_or_else(invalid)?.check(
                official.pid(),
                official.port(),
                official.record_sha256(),
                end,
            )?;
            self.stopped_observed = true;
            Ok(observations)
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn open_prepared_working(
        &mut self,
        lease: &mut NativeLease,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            if self.failed || !self.prepared || self.opened || self.ready {
                return Err(invalid());
            }
            self.check_working(lease, end)?;
            let root = self.trees[0].parent.path().join(".codex");
            let mut claim = lease.anchored_claim(&root, end)?;
            super::observation::open_trees(&mut self.trees, &mut self.journal, &mut claim, end)?;
            drop(claim);
            self.opened = true;
            self.check_working(lease, end)
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn verify_original_pre_resume(
        &mut self,
        source: &mut crate::infrastructure::windows_client_originals024::source::Source,
        owner: &mut crate::infrastructure::windows_official_lifecycle024::Owner,
        lease: &mut Option<NativeLease>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            if self.failed || !self.prepared || !self.opened || self.ready || lease.is_some() {
                return Err(invalid());
            }
            owner
                .originals_require_unresumed(end)
                .map_err(io::Error::other)?;
            source.check(owner, end).map_err(io::Error::other)?;
            self.journal.check_source_owner(owner, end)?;
            super::segments::reacquire_trees(
                &mut self.trees,
                &mut self.journal,
                lease,
                Some(owner),
                end,
            )?;
            self.check_working(lease.as_mut().ok_or_else(invalid)?, end)?;
            owner
                .originals_require_unresumed(end)
                .map_err(io::Error::other)?;
            source.check(owner, end).map_err(io::Error::other)?;
            self.release_prepared(lease, end)?;
            owner
                .originals_require_unresumed(end)
                .map_err(io::Error::other)?;
            self.journal.check_source_owner(owner, end)
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn record_original_ready(
        &mut self,
        source: &mut crate::infrastructure::windows_client_originals024::source::Source,
        owner: &mut crate::infrastructure::windows_official_lifecycle024::Owner,
        ready: &crate::modules::runtime::official_lifecycle024::ServiceObservation,
        lease: &mut Option<NativeLease>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            if self.failed || !self.prepared || !self.opened || self.ready || lease.is_some() {
                return Err(invalid());
            }
            source.check(owner, end).map_err(io::Error::other)?;
            owner
                .originals_check_ready(ready, end)
                .map_err(io::Error::other)?;
            self.journal.check_source_owner(owner, end)?;
            super::observation::reacquire_for_observation(
                &mut self.trees,
                &mut self.journal,
                lease,
                Some(owner),
                end,
            )?;
            let root = self.trees[0].parent.path().join(".codex");
            let mut claim = lease
                .as_mut()
                .ok_or_else(invalid)?
                .anchored_claim(&root, end)?;
            super::observation::observe_trees(&mut self.trees, &mut self.journal, &mut claim, end)?;
            let context = self
                .journal
                .begin_ready_snapshot_link(source, owner, ready, &mut claim, end)?;
            super::observation::snapshots::create_snapshots(
                &mut self.trees,
                &mut self.journal,
                &context,
                &mut claim,
                false,
                end,
            )?;
            let witness = super::observation::snapshots::witness(&mut self.trees, end)?;
            self.journal
                .record_ready_phase(source, owner, ready, witness, &mut claim, end)?;
            super::observation::series::resume_trees(
                &mut self.trees,
                &mut self.journal,
                &mut claim,
                end,
            )?;
            drop(claim);
            super::segments::release_trees(&mut self.trees, &mut self.journal, lease, end)?;
            source.check(owner, end).map_err(io::Error::other)?;
            owner
                .originals_check_ready(ready, end)
                .map_err(io::Error::other)?;
            self.journal.check_source_owner(owner, end)?;
            self.ready = true;
            Ok(())
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn retain(
        trees: [AnchoredTree; 2],
        journal: Journal,
    ) -> Self {
        Self {
            trees,
            journal,
            prepared: false,
            opened: false,
            ready: false,
            prepared_stop: None,
            stopped_observed: false,
            failed: false,
        }
    }
    pub(in crate::infrastructure::windows_client_originals024) fn prepare_working_batched(
        &mut self,
        lease: &mut NativeLease,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            if self.failed || self.prepared {
                return Err(invalid());
            }
            self.journal.check(end)?;
            let root = self.trees[0].parent.path().join(".codex");
            let mut claim = lease.anchored_claim(&root, end)?;
            self.journal.prepare_initial_batches(
                super::batches::initial_batch_witness(&mut self.trees, &mut claim, end)?,
                &mut claim,
                end,
            )?;
            self.journal.check(end)?;
            self.prepared = true;
            Ok(())
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn prepare_working(
        &mut self,
        lease: &mut NativeLease,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            if self.failed || self.prepared {
                return Err(invalid());
            }
            self.journal.check(end)?;
            let root = self.trees[0].parent.path().join(".codex");
            let mut claim = lease.anchored_claim(&root, end)?;
            for tree in &mut self.trees {
                tree.verify(&mut claim, &root, end)?;
                tree.journal_snapshot_anchors(&mut self.journal, &mut claim, end)?;
            }
            let files = self
                .trees
                .iter()
                .enumerate()
                .flat_map(|(r, t)| {
                    t.nodes
                        .iter()
                        .enumerate()
                        .filter_map(move |(i, n)| (n.kind == Kind::File).then_some((r, i)))
                })
                .collect::<Vec<_>>();
            for action in [FileAction::MoveOriginal, FileAction::CreateWork] {
                for (r, i) in &files {
                    self.trees[*r].journal_step(*i, action, &mut self.journal, &mut claim, end)?;
                }
            }
            for tree in &mut self.trees {
                tree.verify(&mut claim, &root, end)?;
            }
            self.journal.check(end)?;
            self.prepared = true;
            Ok(())
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn check_working(
        &mut self,
        lease: &mut NativeLease,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            if self.failed || !self.prepared {
                return Err(invalid());
            }
            let root = self.trees[0].parent.path().join(".codex");
            let mut claim = lease.anchored_claim(&root, end)?;
            for tree in &mut self.trees {
                if tree
                    .nodes
                    .iter()
                    .any(|n| n.kind == Kind::File && n.state != State::Working)
                {
                    return Err(invalid());
                }
                tree.verify(&mut claim, &root, end)?;
            }
            self.journal.check(end)
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn release_prepared(
        &mut self,
        lease: &mut Option<NativeLease>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            if self.failed || !self.prepared {
                return Err(invalid());
            }
            self.check_working(lease.as_mut().ok_or_else(invalid)?, end)?;
            super::segments::release_trees(&mut self.trees, &mut self.journal, lease, end)
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn poison(&mut self) {
        self.failed = true;
        self.journal.poison_segment();
        for tree in &mut self.trees {
            tree.poisoned = true;
        }
    }
    pub(in crate::infrastructure::windows_client_originals024) fn journal(
        &mut self,
    ) -> &mut Journal {
        &mut self.journal
    }
    #[cfg(test)]
    pub(super) fn native_prefix(&self) -> (bool, bool, Vec<String>) {
        (
            self.prepared,
            self.failed,
            self.trees
                .iter()
                .flat_map(|t| {
                    t.nodes
                        .iter()
                        .filter(|n| n.kind == Kind::File)
                        .map(|n| format!("{:?}", n.state))
                })
                .collect(),
        )
    }
}
#[cfg(test)]
#[path = "capture_tests.rs"]
mod tests;
