//! Borrowed exact native objects, not Stop/content qualification. Only the
//! anchored module can construct it; no path/JSON/Clone/serde factory exists.
use super::*;
#[path = "history.rs"]
mod history;
pub(in crate::infrastructure::windows_client_originals024) struct RecoveryWitness<'a> {
    trees: &'a mut [AnchoredTree; 2],
}
pub(in crate::infrastructure::windows_client_originals024::tree::anchored) fn recovery_witness<
    'a,
>(
    trees: &'a mut [AnchoredTree; 2],
    claim: &mut AnchoredClaim<'_>,
    end: Instant,
) -> io::Result<RecoveryWitness<'a>> {
    let mut out = RecoveryWitness { trees };
    out.check(claim, end)?;
    Ok(out)
}
pub(in crate::infrastructure::windows_client_originals024::tree::anchored) fn released_history_witness(
    trees: &mut [AnchoredTree; 2],
    end: Instant,
) -> io::Result<RecoveryWitness<'_>> {
    let mut witness = RecoveryWitness { trees };
    witness.post_release(end)?;
    Ok(witness)
}
/// Ownership borrow only, not qualification. Every native history operation
/// still supplies its complete before/after object barrier.
pub(in crate::infrastructure::windows_client_originals024::tree::anchored) fn borrow_history(
    trees: &mut [AnchoredTree; 2],
) -> RecoveryWitness<'_> {
    RecoveryWitness { trees }
}
impl RecoveryWitness<'_> {
    pub(in crate::infrastructure::windows_client_originals024) fn reborrow(
        &mut self,
    ) -> RecoveryWitness<'_> {
        RecoveryWitness {
            trees: &mut *self.trees,
        }
    }
    pub(in crate::infrastructure::windows_client_originals024) fn execute_batch_node(
        &mut self,
        node: &str,
        retire: bool,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<crate::infrastructure::windows_client_originals024::tree::VerifiedBatchValue>
    {
        use crate::infrastructure::windows_client_originals024::tree::VerifiedBatchValue;
        if retire {
            self.retire(node, claim, end)?;
        } else {
            self.restore(node, claim, end)?;
        }
        // Existing native methods already checked complete BOTH-tree before
        // and after the actual same-handle move. Read the exact retained file
        // again for the sealed result, never produce it from parsed JSON.
        for t in &mut *self.trees {
            let root = t
                .root
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(invalid)?
                .to_string();
            if retire {
                let frozen = t.retirement.as_mut().ok_or_else(invalid)?;
                for item in &frozen.items {
                    let n = &mut frozen.frame.nodes[item.frame_index];
                    if hash::sha256_hex(format!("{root}/{}", n.entry.relative).as_bytes()) == node {
                        if !item.retired {
                            return Err(invalid());
                        }
                        let observed = stamp(
                            &item.target,
                            &mut n.file,
                            &n.entry.relative,
                            Kind::File,
                            end,
                        )?;
                        if observed != n.entry {
                            return Err(invalid());
                        }
                        return Ok(VerifiedBatchValue::observed(
                            root,
                            node.into(),
                            observed.id.index.to_string(),
                            observed.security,
                        ));
                    }
                }
            } else {
                for i in 0..t.nodes.len() {
                    if t.nodes[i].kind == Kind::File && t.key(i) == node {
                        if t.nodes[i].state != State::Restored {
                            return Err(invalid());
                        }
                        let (id, security) = t.node_security(i, false, end)?;
                        return Ok(VerifiedBatchValue::observed(
                            root,
                            node.into(),
                            id,
                            security,
                        ));
                    }
                }
            }
        }
        Err(invalid())
    }
    pub(in crate::infrastructure::windows_client_originals024) fn released_anchors(
        &mut self,
        end: Instant,
    ) -> io::Result<Vec<crate::infrastructure::windows_client_originals024::tree::VerifiedBatchValue>>
    {
        use crate::infrastructure::windows_client_originals024::tree::VerifiedBatchValue;
        self.post_release(end)?;
        let mut rows = Vec::new();
        for t in &mut *self.trees {
            let root = t
                .root
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(invalid)?
                .to_string();
            for i in 0..t.nodes.len() {
                if t.nodes[i].kind != Kind::File {
                    let (id, security) = t.node_security(i, false, end)?;
                    rows.push(VerifiedBatchValue::observed(
                        root.clone(),
                        t.key(i),
                        id,
                        security,
                    ));
                }
            }
        }
        self.post_release(end)?;
        Ok(rows)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn check(
        &mut self,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        self.check_trees(claim, end, AnchoredTree::verify_retirement_prefix)
    }
    fn check_trees(
        &mut self,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
        verify: fn(&mut AnchoredTree, &mut AnchoredClaim<'_>, Instant) -> io::Result<()>,
    ) -> io::Result<()> {
        let result = (|| {
            if self.trees[0].parent.identity() != self.trees[1].parent.identity() {
                return Err(invalid());
            }
            for (i, t) in self.trees.iter_mut().enumerate() {
                if t.root
                    != t.parent
                        .path()
                        .join(if i == 0 { ".codex" } else { ".grok" })
                {
                    return Err(invalid());
                }
                verify(t, claim, end)?;
            }
            Ok(())
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn plans(
        &mut self,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<[serde_json::Value; 2]> {
        self.check(claim, end)?;
        self.retirement_plans()
    }
    pub(in crate::infrastructure::windows_client_originals024) fn restored_plans(
        &mut self,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<[serde_json::Value; 2]> {
        self.restored(claim, end)?;
        self.retirement_plans()
    }
    fn retirement_plans(&self) -> io::Result<[serde_json::Value; 2]> {
        Ok([
            self.trees[0]
                .retirement
                .as_ref()
                .ok_or_else(invalid)?
                .plan
                .clone(),
            self.trees[1]
                .retirement
                .as_ref()
                .ok_or_else(invalid)?
                .plan
                .clone(),
        ])
    }
    pub(in crate::infrastructure::windows_client_originals024) fn retire(
        &mut self,
        node: &str,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.check(claim, end)?;
            let mut found = None;
            for (r, t) in self.trees.iter().enumerate() {
                let frozen = t.retirement.as_ref().ok_or_else(invalid)?;
                for (i, item) in frozen.items.iter().enumerate() {
                    let n = &frozen.frame.nodes[item.frame_index].entry;
                    let key = hash::sha256_hex(
                        format!(
                            "{}/{}",
                            t.root.file_name().unwrap().to_str().unwrap(),
                            n.relative
                        )
                        .as_bytes(),
                    );
                    if key == node {
                        if found.replace((r, i)).is_some() {
                            return Err(invalid());
                        }
                    }
                }
            }
            let (root, index) = found.ok_or_else(invalid)?;
            self.trees[root].retire_current_file_checked(index, claim, end)?;
            self.check(claim, end)
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn restore(
        &mut self,
        node: &str,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.check(claim, end)?;
            let mut found = None;
            for (r, t) in self.trees.iter().enumerate() {
                if t.retirement
                    .as_ref()
                    .ok_or_else(invalid)?
                    .items
                    .iter()
                    .any(|i| !i.retired)
                {
                    return Err(invalid());
                }
                for (i, n) in t
                    .nodes
                    .iter()
                    .enumerate()
                    .filter(|(_, n)| n.kind == Kind::File)
                {
                    let key = hash::sha256_hex(
                        format!(
                            "{}/{}",
                            t.root.file_name().unwrap().to_str().unwrap(),
                            n.original.relative
                        )
                        .as_bytes(),
                    );
                    if key == node {
                        if found.replace((r, i)).is_some() {
                            return Err(invalid());
                        }
                    }
                }
            }
            let (root, index) = found.ok_or_else(invalid)?;
            self.trees[root].restore_original_file_checked(index, claim, end)?;
            self.check(claim, end)
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn restored(
        &mut self,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        self.check_trees(claim, end, AnchoredTree::verify_retirement_restored)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn poison(&mut self) {
        for t in &mut *self.trees {
            t.poisoned = true;
        }
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn foreign_custody_for_test(
        &mut self,
        end: Instant,
    ) -> io::Result<()> {
        check(end)?;
        let path = self.trees[0]
            .custody
            .path()
            .join("synthetic-final-release-foreign");
        drop(create_private_directory(&path)?);
        println!(
            "r6 final-release synthetic foreign custody directory preserved={}",
            path.display()
        );
        check(end)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn post_release(
        &mut self,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            if self.trees[0].parent.identity() != self.trees[1].parent.identity() {
                return Err(invalid());
            }
            for (i, t) in self.trees.iter_mut().enumerate() {
                if t.root
                    != t.parent
                        .path()
                        .join(if i == 0 { ".codex" } else { ".grok" })
                {
                    return Err(invalid());
                }
                // Includes every exact original/retired object/archive/child
                // and acquisition+explicit release of the two stable ranges.
                t.readback_retirement_after_release(end)?;
                if let Some(history) = &mut t.original_history {
                    history.check(end)?;
                }
            }
            Ok(())
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
}
