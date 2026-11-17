//! One explicit original return attempt. The original poisoned trees, journal
//! and exact files stay retained. No normal Stop/Closed or lock-release proof.
use super::*;
use crate::infrastructure::windows_client_originals024::before_resume_record::Event;
mod released;
pub(in crate::infrastructure::windows_client_originals024) use released::{
    AbortRestoredReadback, AfterReturnGuard,
};
pub(super) struct ReturnProgress {
    frozen: Vec<(usize, usize, File)>,
    pending: Option<ReturnStep>,
    completed: Vec<ReturnStep>,
    failed: bool,
    finished: bool,
    final_attempted: bool,
    final_verified: bool,
    final_image: Option<serde_json::Value>,
    final_end: Option<Instant>,
    final_completed_at: Option<Instant>,
}
pub(in crate::infrastructure::windows_client_originals024) struct ReturnStep {
    node: String,
    id: FileIdentity,
    parent: u64,
    retire: bool,
}
impl ReturnStep {
    pub(in crate::infrastructure::windows_client_originals024) fn event(
        &self,
        verified: bool,
    ) -> Event {
        match (self.retire, verified) {
            (true, false) => Event::WorkRetireIntent {
                node_id: self.node.clone(),
                work_file_id: self.id.index.to_string(),
                parent_id: self.parent.to_string(),
            },
            (true, true) => Event::WorkRetiredVerified {
                node_id: self.node.clone(),
                work_file_id: self.id.index.to_string(),
                parent_id: self.parent.to_string(),
            },
            (false, false) => Event::OriginalReturnIntent {
                node_id: self.node.clone(),
                original_file_id: self.id.index.to_string(),
                parent_id: self.parent.to_string(),
            },
            (false, true) => Event::OriginalReturnedVerified {
                node_id: self.node.clone(),
                original_file_id: self.id.index.to_string(),
                parent_id: self.parent.to_string(),
            },
        }
    }
}
// Only a private original adapter scope implements this in production. Generic
// execution below is private; native fixtures use the explicit cfg(test) entry.
pub(in crate::infrastructure::windows_client_originals024) trait ReturnGuard {
    fn check(
        &mut self,
        journal: &mut Journal,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<()>;
    fn record(&mut self, step: &ReturnStep, verified: bool, end: Instant) -> io::Result<()>;
}
fn image(
    trees: &mut [AnchoredTree; 2],
    claim: &mut AnchoredClaim<'_>,
    end: Instant,
) -> io::Result<()> {
    let root = trees[0].parent.path().join(".codex");
    claim.check(&root, end)?;
    if trees[0].parent.identity() != trees[1].parent.identity() {
        return Err(invalid());
    }
    for (i, tree) in trees.iter_mut().enumerate() {
        if tree.root
            != tree
                .parent
                .path()
                .join(if i == 0 { ".codex" } else { ".grok" })
        {
            return Err(invalid());
        }
        tree.read_retained_image(end)?;
    }
    claim.check(&root, end)
}
impl CapturedTrees {
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_abort_return_state(
        &self,
    ) -> (bool, bool, usize, usize, bool) {
        self.abort_return
            .as_ref()
            .map(|p| {
                (
                    p.failed,
                    p.finished,
                    p.frozen.len(),
                    p.completed.len(),
                    p.pending.is_some(),
                )
            })
            .unwrap_or((false, false, 0, 0, false))
    }
    pub(in crate::infrastructure::windows_client_originals024) fn return_before_resume(
        &mut self,
        scope: &mut crate::infrastructure::windows_client_originals024::OriginalReturnScope<'_>,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<serde_json::Value> {
        self.execute_abort_return(scope, claim, end)
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn component_return_before_resume_for_test<
        G: ReturnGuard,
    >(
        &mut self,
        scope: &mut G,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<serde_json::Value> {
        self.execute_abort_return(scope, claim, end)
    }
    fn execute_abort_return<G: ReturnGuard>(
        &mut self,
        scope: &mut G,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<serde_json::Value> {
        if self.abort_return.is_some() {
            return Err(invalid());
        }
        self.abort_return = Some(ReturnProgress {
            frozen: Vec::new(),
            pending: None,
            completed: Vec::new(),
            failed: true,
            finished: false,
            final_attempted: false,
            final_verified: false,
            final_image: None,
            final_end: None,
            final_completed_at: None,
        });
        // Freeze the old mutation path before every fallible read/write. This
        // operation has its own native guard; ordinary verify stays poisoned.
        self.poison();
        let result = (|| {
            check(end)?;
            if self.ready
                || self.prepared_stop.is_some()
                || self.stopped_observed
                || self.content_assessment.is_some()
                || self.restored_history
            {
                return Err(invalid());
            }
            let trees = self.trees.as_mut().ok_or_else(invalid)?;
            for tree in trees.iter_mut() {
                tree.read_before_resume_prefix(claim, end)?;
                if tree.nodes.iter().any(|n| n.pending_work.is_some()) || tree.retirement.is_some()
                {
                    return Err(invalid());
                }
            }
            image(trees, claim, end)?;
            scope.check(&mut self.journal, claim, end)?;
            // Same-object work handles are frozen before any file is moved.
            // ReOpenFile acts on the original retained object, never a path.
            for (r, tree) in trees.iter_mut().enumerate() {
                for (i, n) in tree.nodes.iter_mut().enumerate() {
                    if n.kind != Kind::File || n.state != State::Working || !n.work_open {
                        continue;
                    }
                    let (file, entry) = n.work.as_ref().ok_or_else(invalid)?;
                    let freeze = observation::reopen(file, 0x80030000, 1)?;
                    let progress = self.abort_return.as_mut().unwrap();
                    progress.frozen.push((r, i, freeze));
                    let actual = &progress.frozen.last().unwrap().2;
                    if id(actual, false)? != entry.id
                        || native_stamp(actual, end).map_err(failure)? != entry.security
                    {
                        return Err(invalid());
                    }
                }
            }
            image(trees, claim, end)?;
            scope.check(&mut self.journal, claim, end)?;
            let progress = self.abort_return.as_mut().unwrap();
            for retire in [true, false] {
                for r in 0..2 {
                    for i in 0..trees[r].nodes.len() {
                        let tree = &trees[r];
                        let n = &tree.nodes[i];
                        if n.kind != Kind::File
                            || if retire {
                                n.state != State::Working
                            } else {
                                !matches!(n.state, State::Moved | State::Retired)
                            }
                        {
                            continue;
                        }
                        let parent = Path::new(&n.original.relative)
                            .parent()
                            .and_then(|p| {
                                tree.nodes.iter().find(|a| {
                                    a.kind == Kind::Directory
                                        && Path::new(&a.original.relative) == p
                                })
                            })
                            .ok_or_else(invalid)?
                            .original
                            .id
                            .index;
                        let native = if retire {
                            n.work.as_ref().ok_or_else(invalid)?.1.id
                        } else {
                            n.original.id
                        };
                        let target = if retire {
                            tree.custody.path().join(format!("retired-{}", tree.key(i)))
                        } else {
                            tree.root.join(&n.original.relative)
                        };
                        if native.volume != tree.custody.identity().volume
                            || native.volume != tree.parent.identity().volume
                        {
                            return Err(invalid());
                        }
                        progress.pending = Some(ReturnStep {
                            node: tree.key(i),
                            id: native,
                            parent,
                            retire,
                        });
                        image(trees, claim, end)?;
                        scope.check(&mut self.journal, claim, end)?;
                        require_absent(&target)?;
                        scope.record(progress.pending.as_ref().unwrap(), false, end)?;
                        image(trees, claim, end)?;
                        scope.check(&mut self.journal, claim, end)?;
                        require_absent(&target)?;
                        check(end)?;
                        if retire {
                            let handle = if trees[r].nodes[i].work_open {
                                &progress
                                    .frozen
                                    .iter()
                                    .find(|(a, b, _)| *a == r && *b == i)
                                    .ok_or_else(invalid)?
                                    .2
                            } else {
                                &trees[r].nodes[i].work.as_ref().ok_or_else(invalid)?.0
                            };
                            rename_original_handle(handle, &target)?;
                            trees[r].nodes[i].state = State::Retired;
                        } else {
                            rename_original_handle(&trees[r].nodes[i].file, &target)?;
                            trees[r].nodes[i].state = State::Restored;
                        }
                        // Infallible native-state assignment precedes all IO.
                        image(trees, claim, end)?;
                        scope.check(&mut self.journal, claim, end)?;
                        scope.record(progress.pending.as_ref().unwrap(), true, end)?;
                        image(trees, claim, end)?;
                        scope.check(&mut self.journal, claim, end)?;
                        progress.completed.push(progress.pending.take().unwrap());
                    }
                }
            }
            image(trees, claim, end)?;
            scope.check(&mut self.journal, claim, end)?;
            if trees.iter().flat_map(|t| &t.nodes).any(|n| {
                n.kind == Kind::File && !matches!(n.state, State::Original | State::Restored)
            }) {
                return Err(invalid());
            }
            progress.finished = true;
            progress.failed = false;
            Ok(
                serde_json::json!({"purpose":"before-resume-native-return-readback-only","originalNodes":trees.iter().map(|t|t.nodes.len()).sum::<usize>(),
                "completedSteps":progress.completed.len(),"allOriginalFilesAtClient":true,"claimStillHeld":true,
                "locksReleased":false,"normalStop":false,"closed":false,"publicStartOpen":false,"runtimeAuthority":false}),
            )
        })();
        if result.is_err() {
            self.abort_return.as_mut().unwrap().failed = true;
            self.poison();
        }
        result
    }
}
