//! Retained complete original trees+journal installed BEFORE client mutation.
//! Source/current/route/original Owner authority belongs to the outer store.
use super::super::super::{
    coordination::NativeLease,
    v3::{FileAction, Journal},
};
use super::*;
pub(in crate::infrastructure::windows_client_originals024) struct CapturedTrees {
    trees: [AnchoredTree; 2],
    journal: Journal,
    prepared: bool,
    opened: bool,
    ready: bool,
    failed: bool,
}
impl CapturedTrees {
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
            failed: false,
        }
    }
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
