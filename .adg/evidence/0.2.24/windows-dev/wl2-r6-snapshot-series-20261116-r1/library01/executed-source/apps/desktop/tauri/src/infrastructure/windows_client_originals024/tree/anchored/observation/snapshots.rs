//! Borrowed actual BOTH native frames+archives. No constructor from reference
//! data, no boolean callback, no Clone/serde or native Ready grant.
use super::*;
use crate::infrastructure::windows_client_originals024::v3::{
    journal::ObservationContext, Journal,
};
pub(in crate::infrastructure::windows_client_originals024) struct SnapshotWitness<'a> {
    frames: [&'a mut WorkFrame; 2],
}
/// Immediately observed actual BOTH native frames, no JSON constructor.
pub(in crate::infrastructure::windows_client_originals024) struct SnapshotLinkObservation {
    references: [serde_json::Value; 2],
}
impl SnapshotLinkObservation {
    pub(in crate::infrastructure::windows_client_originals024) fn into_values(
        self,
    ) -> [serde_json::Value; 2] {
        self.references
    }
}
impl SnapshotWitness<'_> {
    pub(in crate::infrastructure::windows_client_originals024) fn link_observation(
        &mut self,
        end: Instant,
    ) -> io::Result<SnapshotLinkObservation> {
        Ok(SnapshotLinkObservation {
            references: self.references(end)?,
        })
    }
    pub(in crate::infrastructure::windows_client_originals024) fn reborrow(
        &mut self,
    ) -> SnapshotWitness<'_> {
        let (first, second) = self.frames.split_at_mut(1);
        SnapshotWitness {
            frames: [&mut *first[0], &mut *second[0]],
        }
    }
    pub(in crate::infrastructure::windows_client_originals024) fn references(
        &mut self,
        end: Instant,
    ) -> io::Result<[serde_json::Value; 2]> {
        let mut values = Vec::new();
        for (frame, root) in self.frames.iter_mut().zip([".codex", ".grok"]) {
            frame.check(end)?;
            if frame.root.file_name().and_then(|s| s.to_str()) != Some(root) {
                return Err(invalid());
            }
            values.push(frame.archive.as_mut().ok_or_else(invalid)?.reference(end)?);
        }
        values.try_into().map_err(|_| invalid())
    }
}
pub(super) fn witness(
    trees: &mut [AnchoredTree; 2],
    end: Instant,
) -> io::Result<SnapshotWitness<'_>> {
    for tree in trees.iter_mut() {
        if tree.poisoned || tree.work_mode != WorkMode::Observed {
            return Err(invalid());
        }
        tree.verify_custody_anchors(end)?;
        tree.observed.as_mut().ok_or_else(invalid)?.check(end)?;
    }
    let (first, second) = trees.split_at_mut(1);
    Ok(SnapshotWitness {
        frames: [
            first[0].observed.as_mut().ok_or_else(invalid)?,
            second[0].observed.as_mut().ok_or_else(invalid)?,
        ],
    })
}
pub(super) fn create_snapshots(
    trees: &mut [AnchoredTree; 2],
    journal: &mut Journal,
    context: &ObservationContext,
    claim: &mut AnchoredClaim<'_>,
    fail_first: bool,
    end: Instant,
) -> io::Result<()> {
    let result = (|| {
        let root = trees[0].parent.path().join(".codex");
        claim.check(&root, end)?;
        for (i, tree) in trees.iter_mut().enumerate() {
            tree.verify_custody_anchors(end)?;
            let frame = tree.observed.as_mut().ok_or_else(invalid)?;
            frame.archive(tree.parent.path(), context, fail_first && i == 0, end)?;
            claim.check(&root, end)?;
        }
        journal.finish_snapshot_link(witness(trees, end)?, claim, end)
    })();
    if result.is_err() {
        journal.poison_segment();
        for tree in trees {
            tree.poisoned = true;
        }
    }
    result
}
pub(super) fn link_observation(
    trees: &mut [AnchoredTree; 2],
    journal: &mut Journal,
    claim: &mut AnchoredClaim<'_>,
    fail_first: bool,
    end: Instant,
) -> io::Result<()> {
    let result = (|| {
        let context = journal.begin_observation_snapshot_link(claim, end)?;
        create_snapshots(trees, journal, &context, claim, fail_first, end)
    })();
    if result.is_err() {
        journal.poison_segment();
        for tree in trees {
            tree.poisoned = true;
        }
    }
    result
}
#[cfg(test)]
#[path = "snapshot_tests.rs"]
mod tests;
