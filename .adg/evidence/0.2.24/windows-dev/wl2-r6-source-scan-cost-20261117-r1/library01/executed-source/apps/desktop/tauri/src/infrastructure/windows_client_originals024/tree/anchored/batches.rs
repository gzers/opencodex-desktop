//! Borrowed native initial-tree batch operations; no JSON/path factory.
use super::*;
use crate::infrastructure::windows_client_originals024::v3::batches::BatchKind;
pub(in crate::infrastructure::windows_client_originals024) struct VerifiedBatchValue {
    root: String,
    node: String,
    id: String,
    security: SnapshotData,
}
impl VerifiedBatchValue {
    pub(in crate::infrastructure::windows_client_originals024::tree::anchored) fn observed(
        root: String,
        node: String,
        id: String,
        security: SnapshotData,
    ) -> Self {
        Self {
            root,
            node,
            id,
            security,
        }
    }
    pub(in crate::infrastructure::windows_client_originals024) fn node(&self) -> &str {
        &self.node
    }
    pub(in crate::infrastructure::windows_client_originals024) fn root(&self) -> &str {
        &self.root
    }
    pub(in crate::infrastructure::windows_client_originals024) fn parts(
        &self,
    ) -> io::Result<(String, SnapshotData)> {
        Ok((
            self.id.clone(),
            serde_json::from_value(serde_json::to_value(&self.security).map_err(|_| invalid())?)
                .map_err(|_| invalid())?,
        ))
    }
    pub(in crate::infrastructure::windows_client_originals024) fn data(&self) -> serde_json::Value {
        serde_json::json!({"rootId":self.root,"nodeId":self.node,"nativeId":self.id,"security":self.security,"runtimeAuthority":false})
    }
}
pub(in crate::infrastructure::windows_client_originals024) struct InitialBatchWitness<'a> {
    trees: &'a mut [AnchoredTree; 2],
}
pub(in crate::infrastructure::windows_client_originals024::tree::anchored) fn initial_batch_witness<
    'a,
>(
    trees: &'a mut [AnchoredTree; 2],
    claim: &mut AnchoredClaim<'_>,
    end: Instant,
) -> io::Result<InitialBatchWitness<'a>> {
    let mut witness = InitialBatchWitness { trees };
    witness.check(claim, end)?;
    Ok(witness)
}
impl InitialBatchWitness<'_> {
    pub(in crate::infrastructure::windows_client_originals024) fn check(
        &mut self,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            if self.trees[0].parent.identity() != self.trees[1].parent.identity() {
                return Err(invalid());
            }
            let codex = self.trees[0].parent.path().join(".codex");
            for (i, t) in self.trees.iter_mut().enumerate() {
                if t.root
                    != t.parent
                        .path()
                        .join(if i == 0 { ".codex" } else { ".grok" })
                    || t.work_mode != WorkMode::Sealed
                {
                    return Err(invalid());
                }
                t.verify(claim, &codex, end)?;
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
        Ok(self.trees.each_ref().map(|t| t.plan()))
    }
    pub(in crate::infrastructure::windows_client_originals024) fn anchors(
        &mut self,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<Vec<VerifiedBatchValue>> {
        let result = (|| {
            self.check(claim, end)?;
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
            self.check(claim, end)?;
            Ok(rows)
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn execute(
        &mut self,
        node: &str,
        kind: BatchKind,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<VerifiedBatchValue> {
        let result = (|| {
            self.check(claim, end)?;
            let mut found = None;
            for (r, t) in self.trees.iter().enumerate() {
                for (i, n) in t.nodes.iter().enumerate() {
                    if n.kind == Kind::File && t.key(i) == node {
                        if found.replace((r, i)).is_some() {
                            return Err(invalid());
                        }
                    }
                }
            }
            let (r, i) = found.ok_or_else(invalid)?;
            let state = match kind {
                BatchKind::OriginalMove => State::Moved,
                BatchKind::WorkCreate => State::Working,
                _ => return Err(invalid()),
            };
            let codex = self.trees[0].parent.path().join(".codex");
            self.trees[r].step(i, state, claim, &codex, end)?;
            self.check(claim, end)?;
            let t = &mut self.trees[r];
            let (id, security) = t.node_security(i, kind == BatchKind::WorkCreate, end)?;
            Ok(VerifiedBatchValue::observed(
                t.root
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(invalid)?
                    .into(),
                node.into(),
                id,
                security,
            ))
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
}
