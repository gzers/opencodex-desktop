//! Native post-release original tree readback. No old Start renewal, normal
//! Stop/Closed or successor capability. Actual objects remain original-owned.
use super::*;
use crate::infrastructure::windows_client_originals024::coordination::{
    ReleaseProbe, ReleasedLease,
};
pub(in crate::infrastructure::windows_client_originals024) struct AbortRestoredReadback {
    nodes: u32,
    coordination: u32,
    end: Instant,
    digest: String,
}
impl AbortRestoredReadback {
    pub(in crate::infrastructure::windows_client_originals024) fn data(
        &self,
        end: Instant,
    ) -> io::Result<serde_json::Value> {
        check(end)?;
        if end != self.end {
            return Err(invalid());
        }
        Ok(
            serde_json::json!({"purpose":"native-aborted-before-resume-restored-readback","originalNodes":self.nodes,"coordinationObjects":self.coordination,"digest":self.digest,"normalStop":false,"closed":false,"publicStartOpen":false,"runtimeAuthority":false}),
        )
    }
}
pub(in crate::infrastructure::windows_client_originals024) trait AfterReturnGuard {
    fn check(
        &mut self,
        journal: &mut Journal,
        released: &mut ReleasedLease,
        end: Instant,
    ) -> io::Result<()>;
    fn record_verified(&mut self, nodes: u32, coordination: u32, end: Instant) -> io::Result<()>;
}
fn restored(trees: &mut [AnchoredTree; 2], end: Instant) -> io::Result<serde_json::Value> {
    check(end)?;
    if trees[0].parent.identity() != trees[1].parent.identity() {
        return Err(invalid());
    }
    let mut data = Vec::new();
    for (r, t) in trees.iter_mut().enumerate() {
        if t.root
            != t.parent
                .path()
                .join(if r == 0 { ".codex" } else { ".grok" })
            || t.pending_capture.is_some()
            || t.nodes.iter().any(|n| {
                n.pending_work.is_some()
                    || n.kind == Kind::File && !matches!(n.state, State::Original | State::Restored)
            })
        {
            return Err(invalid());
        }
        t.read_retained_image(end)?;
        for n in &mut t.nodes {
            if stamp(
                &t.root.join(&n.original.relative),
                &mut n.file,
                &n.original.relative,
                n.kind,
                end,
            )? != n.original
            {
                return Err(invalid());
            }
        }
        t.read_retained_image(end)?;
        data.push(t.plan());
    }
    check(end)?;
    Ok(serde_json::json!(data))
}
impl CapturedTrees {
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_abort_final_state_for_test(
        &self,
    ) -> (bool, bool, bool) {
        self.abort_return
            .as_ref()
            .map(|p| (p.final_attempted, p.final_verified, p.failed))
            .unwrap_or((false, false, false))
    }
    pub(in crate::infrastructure::windows_client_originals024) fn finish_before_resume_release(
        &mut self,
        scope: &mut crate::infrastructure::windows_client_originals024::OriginalReleasedScope<'_>,
        released: &mut ReleasedLease,
        probe: &mut ReleaseProbe,
        end: Instant,
    ) -> io::Result<AbortRestoredReadback> {
        self.finish_abort_release(scope, released, probe, end)
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_finish_before_resume_release_for_test<
        G: AfterReturnGuard,
    >(
        &mut self,
        scope: &mut G,
        released: &mut ReleasedLease,
        probe: &mut ReleaseProbe,
        end: Instant,
    ) -> io::Result<AbortRestoredReadback> {
        self.finish_abort_release(scope, released, probe, end)
    }
    fn finish_abort_release<G: AfterReturnGuard>(
        &mut self,
        scope: &mut G,
        released: &mut ReleasedLease,
        probe: &mut ReleaseProbe,
        end: Instant,
    ) -> io::Result<AbortRestoredReadback> {
        let progress = self.abort_return.as_mut().ok_or_else(invalid)?;
        if progress.final_attempted {
            return Err(invalid());
        }
        progress.final_attempted = true;
        let result = (|| {
            check(end)?;
            let progress = self.abort_return.as_ref().unwrap();
            if progress.failed
                || !progress.finished
                || progress.pending.is_some()
                || progress.final_verified
            {
                return Err(invalid());
            }
            scope.check(&mut self.journal, released, end)?;
            let trees = self.trees.as_mut().ok_or_else(invalid)?;
            let before = restored(trees, end)?;
            let owner = trees[0]
                .nodes
                .iter()
                .position(|n| {
                    n.kind == Kind::Coordinator
                        && n.original.relative == ".opencodex-native-main.owner.sqlite"
                })
                .ok_or_else(invalid)?;
            let owner_path = trees[0].root.join(".opencodex-native-main.owner.sqlite");
            released.observe_released_ranges(
                probe,
                &owner_path,
                &mut trees[0].nodes[owner].file,
                end,
            )?;
            probe.check(released, end)?;
            scope.check(&mut self.journal, released, end)?;
            let after = restored(trees, end)?;
            if before != after {
                return Err(invalid());
            }
            probe.check(released, end)?;
            let nodes = u32::try_from(trees.iter().map(|t| t.nodes.len()).sum::<usize>())
                .map_err(|_| invalid())?;
            // The three external N/L/K are distinct from both stable anchors
            // already counted in the full original tree above.
            let coordination = 3;
            scope.record_verified(nodes, coordination, end)?;
            scope.check(&mut self.journal, released, end)?;
            probe.check(released, end)?;
            let final_image = restored(trees, end)?;
            if after != final_image {
                return Err(invalid());
            }
            let data = serde_json::json!({"originalTrees":final_image,"releasedRanges":probe.data()?,"originalNodes":nodes,"coordinationObjects":coordination});
            let digest = hash::sha256_hex(&serde_json::to_vec(&data).map_err(|_| invalid())?);
            check(end)?;
            self.abort_return.as_mut().unwrap().final_verified = true;
            Ok(AbortRestoredReadback {
                nodes,
                coordination,
                end,
                digest,
            })
        })();
        if result.is_err() {
            self.abort_return.as_mut().unwrap().failed = true;
            self.poison();
        }
        result
    }
}
