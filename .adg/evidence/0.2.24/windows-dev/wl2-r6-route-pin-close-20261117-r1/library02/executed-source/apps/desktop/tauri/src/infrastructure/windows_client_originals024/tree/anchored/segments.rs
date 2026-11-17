//! Whole-tree native barriers surrounding canonical lease segment events.
//! Original directories/files remain retained; no root or claim relocation.
use super::super::super::{coordination::NativeLease, v3::Journal};
use super::*;
pub(super) fn release_trees(
    trees: &mut [AnchoredTree; 2],
    journal: &mut Journal,
    lease: &mut Option<NativeLease>,
    end: Instant,
) -> io::Result<()> {
    let result = (|| {
        let root = trees[0].parent.path().join(".codex");
        let mut claim = lease
            .as_mut()
            .ok_or_else(invalid)?
            .anchored_claim(&root, end)?;
        for tree in trees.iter_mut() {
            tree.verify_release_boundary(Some(&mut claim), end)?;
        }
        let plans = trees.each_ref().map(|t| t.plan());
        journal.prepare_lease_release(&plans, &mut claim, end)?;
        drop(claim);
        // The lease is consumed only after the prepared record+current are
        // durable. Explicit receipt is never inferred from Drop or old JSON.
        let receipt = lease
            .take()
            .ok_or_else(invalid)?
            .release_with_receipt(end)?;
        journal.retain_lease_release(receipt)?;
        for tree in trees.iter_mut() {
            tree.verify_release_boundary(None, end)?;
        }
        journal.finish_lease_release(&plans, end)?;
        for tree in trees.iter_mut() {
            tree.finish_release_boundary();
        }
        Ok(())
    })();
    if result.is_err() {
        journal.poison_segment();
        for tree in trees {
            tree.poisoned = true;
        }
    }
    result
}
pub(super) fn reacquire_trees(
    trees: &mut [AnchoredTree; 2],
    journal: &mut Journal,
    lease: &mut Option<NativeLease>,
    owner: Option<&mut crate::infrastructure::windows_official_lifecycle024::Owner>,
    end: Instant,
) -> io::Result<()> {
    let result = (|| {
        if lease.is_some() {
            return Err(invalid());
        }
        for tree in trees.iter_mut() {
            tree.verify_image(end)?;
        }
        let plans = trees.each_ref().map(|t| t.plan());
        journal.prepare_lease_acquire(&plans, end)?;
        // Install before fallible whole-tree readback/result publication so a
        // failed new segment retains its actual native held ranges for cleanup.
        *lease = Some(journal.acquire_prepared_lease(owner, end)?);
        let root = trees[0].parent.path().join(".codex");
        let mut claim = lease
            .as_mut()
            .ok_or_else(invalid)?
            .anchored_claim(&root, end)?;
        for tree in trees.iter_mut() {
            tree.verify(&mut claim, &root, end)?;
        }
        journal.finish_lease_acquire(&plans, &mut claim, end)
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
#[path = "segment_tests.rs"]
mod tests;
