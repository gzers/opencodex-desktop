//! Borrowed exact native objects, not Stop/content qualification. Only the
//! anchored module can construct it; no path/JSON/Clone/serde factory exists.
use super::*;
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
impl RecoveryWitness<'_> {
    pub(in crate::infrastructure::windows_client_originals024) fn check(
        &mut self,
        claim: &mut AnchoredClaim<'_>,
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
                t.verify_retirement_prefix(claim, end)?;
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
            self.trees[root].retire_current_file(index, claim, end)?;
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
            self.trees[root].restore_original_file(index, claim, end)?;
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
        self.check(claim, end)?;
        let result = (|| {
            for t in &mut *self.trees {
                t.verify_retirement_restored(claim, end)?;
            }
            Ok(())
        })();
        if result.is_err() {
            self.poison();
        }
        result
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
            }
            Ok(())
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
}
