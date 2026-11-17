//! Read-only native prefix DATA under the actual fresh claim. It neither
//! clears poison nor permits a move, lease transition, restore or Closed.
use super::*;
impl AnchoredTree {
    pub(in crate::infrastructure::windows_client_originals024) fn read_before_resume_prefix(
        &mut self,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<serde_json::Value> {
        let result = (|| {
            check(end)?;
            let root = self.parent.path().join(".codex");
            claim.check(&root, end)?;
            // Incomplete unverified opens/writes stay retained and Required.
            // Never reopen them as original objects or infer a successful node.
            if self.pending_capture.is_some()
                || self.nodes.is_empty()
                || self.retirement.is_some()
                || self.original_history.is_some()
                || self.nodes.iter().any(|n| {
                    n.pending_work.is_some()
                        || match n.kind {
                            Kind::Directory | Kind::Coordinator => {
                                n.state != State::Original || n.work.is_some() || n.work_open
                            }
                            Kind::File => match n.state {
                                State::Original | State::Moved => n.work.is_some() || n.work_open,
                                State::Working => n.work.is_none(),
                                State::Retired | State::Restored => true,
                            },
                        }
                })
            {
                return Err(invalid());
            }
            self.read_retained_image(end)?;
            let rows = self.nodes.iter().enumerate().map(|(i, n)| {
                serde_json::json!({
                    "nodeId":self.key(i),
                    "state":match n.state {State::Original=>"Original",State::Moved=>"Moved",State::Working=>"Working",State::Retired=>"Retired",State::Restored=>"Restored"},
                    "originalLocation":self.original_location(i),
                    "originalVolume":n.original.id.volume,
                    "originalFileId":n.original.id.index.to_string(),
                    "originalSecurity":n.original.security,
                    "originalBytes":n.original.bytes.len(),
                    "originalSha256":hash::sha256_hex(&n.original.bytes),
                    "work":n.work.as_ref().map(|(_,e)|serde_json::json!({
                        "location":self.work_location(i),"volume":e.id.volume,
                        "fileId":e.id.index.to_string(),"security":e.security,
                        "bytes":e.bytes.len(),"sha256":hash::sha256_hex(&e.bytes),
                        "open":n.work_open
                    }))
                })
            }).collect::<Vec<_>>();
            let data = serde_json::json!({
                "purpose":"before-resume-retained-tree-read-data",
                "plan":self.plan(),"nodes":rows,
                "parentVolume":self.parent.identity().volume,
                "parentId":self.parent.identity().index.to_string(),
                "parentSecurity":self.parent_security,
                "custodySecurity":self.custody_security,
                "claimEpoch":claim.epoch(),"claimKey":claim.key(),
                "poisoned":self.poisoned,"runtimeAuthority":false
            });
            // Full native both-location fence after constructing the DATA;
            // original file handles, state and poison remain the same objects.
            self.read_retained_image(end)?;
            claim.check(&root, end)?;
            Ok(data)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
}
#[cfg(test)]
mod tests;
