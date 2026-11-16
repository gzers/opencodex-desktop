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
    failed: bool,
}
impl CapturedTrees {
    pub(in crate::infrastructure::windows_client_originals024) fn retain(
        trees: [AnchoredTree; 2],
        journal: Journal,
    ) -> Self {
        Self {
            trees,
            journal,
            prepared: false,
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
