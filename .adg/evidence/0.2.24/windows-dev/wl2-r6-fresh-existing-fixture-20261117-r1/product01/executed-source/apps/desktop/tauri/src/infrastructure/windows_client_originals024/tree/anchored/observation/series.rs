//! Actual observed generations reopen only after durable BOTH-root links.
//! Original and initial-work baselines never change. Immutable snapshot objects
//! and exact historical work observers remain retained across official writes.
use super::*;
use crate::infrastructure::windows_client_originals024::content::StoppedContentScope;
use crate::infrastructure::windows_client_originals024::v3::Journal;
pub(in crate::infrastructure::windows_client_originals024) struct NativeContentAssessment {
    scope_digest: String,
    series_digest: String,
    assessment: crate::modules::runtime::client_artifact_policy::Assessment,
    deadline: Instant,
}
fn series_digest(
    context:&crate::infrastructure::windows_client_originals024::v3::journal::ObservationSeriesContext,
) -> io::Result<String> {
    let values = serde_json::json!({"sequences":context.sequences(),"bindings":[context.binding(0)?,context.binding(1)?,context.binding(2)?],
        "frames":[[context.frame(0,0)?,context.frame(0,1)?],[context.frame(1,0)?,context.frame(1,1)?],[context.frame(2,0)?,context.frame(2,1)?]]});
    Ok(hash::sha256_hex(
        &serde_json::to_vec(&values).map_err(|_| invalid())?,
    ))
}
impl NativeContentAssessment {
    pub(in crate::infrastructure::windows_client_originals024) fn qualification_data(
        &self,
        context: &crate::infrastructure::windows_client_originals024::v3::journal::ObservationSeriesContext,
    ) -> io::Result<serde_json::Value> {
        use crate::modules::runtime::client_artifact_policy::{Action, Artifact};
        if self.series_digest != series_digest(context)? {
            return Err(invalid());
        }
        let mut actions = Vec::new();
        for ((artifact, action), expected) in self.assessment.0.iter().zip([
            Artifact::Catalog,
            Artifact::ModelsCache,
            Artifact::CatalogAudit,
        ]) {
            if *artifact != expected {
                return Err(invalid());
            }
            actions.push(match action {
                Action::AlreadyOriginal => "AlreadyOriginal",
                Action::RestoreOriginal => "RestoreOriginal",
            });
        }
        Ok(
            serde_json::json!({"scopeDigest":self.scope_digest,"seriesDigest":self.series_digest,
            "sequences":context.sequences(),"bindings":[context.binding(0)?,context.binding(1)?,context.binding(2)?],
            "frames":[[context.frame(0,0)?,context.frame(0,1)?],[context.frame(1,0)?,context.frame(1,1)?],[context.frame(2,0)?,context.frame(2,1)?]],"actions":actions}),
        )
    }
    pub(in crate::infrastructure::windows_client_originals024) fn matches(
        &self,
        scope: &StoppedContentScope,
        context:&crate::infrastructure::windows_client_originals024::v3::journal::ObservationSeriesContext,
        end: Instant,
    ) -> io::Result<()> {
        check(end)?;
        if self.deadline != end
            || scope.deadline() != end
            || self.scope_digest != scope.digest()?
            || self.series_digest != series_digest(context)?
        {
            return Err(invalid());
        }
        use crate::modules::runtime::client_artifact_policy::{Action, Artifact};
        for ((artifact, action), expected) in self.assessment.0.iter().zip([
            Artifact::Catalog,
            Artifact::ModelsCache,
            Artifact::CatalogAudit,
        ]) {
            if *artifact != expected
                || !matches!(action, Action::AlreadyOriginal | Action::RestoreOriginal)
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
    pub(in crate::infrastructure::windows_client_originals024) fn data(&self) -> serde_json::Value {
        serde_json::json!({"scopeDigest":self.scope_digest,"seriesDigest":self.series_digest,
            "assessment":format!("{:?}",self.assessment),"nativeAuthority":false,"typedClosed":false})
    }
}
pub(in crate::infrastructure::windows_client_originals024) fn assess_native_stopped_series(
    trees: &mut [AnchoredTree; 2],
    context:&crate::infrastructure::windows_client_originals024::v3::journal::ObservationSeriesContext,
    scope: &StoppedContentScope,
    claim: &mut AnchoredClaim<'_>,
    end: Instant,
) -> io::Result<NativeContentAssessment> {
    if scope.deadline() != end {
        return Err(invalid());
    }
    let pids = scope.pids();
    let (from, to) = scope.interval();
    let assessment = assess_tree_series(
        trees,
        context,
        Some(scope.routes()),
        &pids,
        scope.audit_home(),
        from,
        to,
        claim,
        end,
    )?;
    let out = NativeContentAssessment {
        scope_digest: scope.digest()?,
        series_digest: series_digest(context)?,
        assessment,
        deadline: end,
    };
    out.matches(scope, context, end)?;
    Ok(out)
}
impl AnchoredTree {
    pub(in crate::infrastructure::windows_client_originals024::tree::anchored) fn verify_release_boundary(
        &mut self,
        claim: Option<&mut AnchoredClaim<'_>>,
        end: Instant,
    ) -> io::Result<()> {
        let root = self.parent.path().join(".codex");
        if let Some(claim) = claim {
            claim.check(&root, end)?;
        }
        if self.work_mode != WorkMode::Open || self.snapshot_history.is_empty() {
            return self.verify_image(end);
        }
        self.verify_custody_anchors(end)?;
        if self.release_view.is_none() {
            self.release_view = Some(WorkFrame::empty(self, end)?);
            self.release_view
                .as_mut()
                .ok_or_else(invalid)?
                .capture(end)?;
        }
        let current = self.release_view.as_mut().ok_or_else(invalid)?;
        current.check(end)?;
        let metadata = current.metadata();
        self.snapshot_history
            .last_mut()
            .ok_or_else(invalid)?
            .archive
            .matches_metadata(&metadata, end)?;
        self.verify_custody_anchors(end)
    }
    pub(in crate::infrastructure::windows_client_originals024::tree::anchored) fn finish_release_boundary(
        &mut self,
    ) {
        // Only after canonical explicit-release result. Dropping the temporary
        // deny-write view restores the already-recorded writable interval; all
        // exact historical work observers and original custody stay retained.
        self.release_view = None;
    }
    fn archive_for_sequence(
        &mut self,
        sequence: u32,
        _end: Instant,
    ) -> io::Result<&mut archive::FrameArchive> {
        let mut found = 0;
        for old in &mut self.snapshot_history {
            if old.archive.observation_sequence()? == sequence {
                found += 1;
            }
        }
        let current_matches = if let Some(frame) = &mut self.observed {
            frame
                .archive
                .as_ref()
                .is_some_and(|a| a.observation_sequence().ok() == Some(sequence))
        } else {
            false
        };
        found += usize::from(current_matches);
        if found != 1 {
            return Err(invalid());
        }
        if current_matches {
            return self
                .observed
                .as_mut()
                .and_then(|f| f.archive.as_mut())
                .ok_or_else(invalid);
        }
        self.snapshot_history
            .iter_mut()
            .find(|s| s.archive.observation_sequence().ok() == Some(sequence))
            .map(|s| &mut s.archive)
            .ok_or_else(invalid)
    }
    fn original_artifacts(&self) -> io::Result<super::super::super::ArtifactFrame> {
        let entries = self
            .nodes
            .iter()
            .map(|n| {
                Ok(Entry {
                    relative: n.original.relative.clone(),
                    directory: n.original.directory,
                    id: n.original.id,
                    security: serde_json::from_value(
                        serde_json::to_value(&n.original.security).map_err(|_| invalid())?,
                    )
                    .map_err(|_| invalid())?,
                    bytes: n.original.bytes.clone(),
                    children: n.original.children.clone(),
                })
            })
            .collect::<io::Result<Vec<_>>>()?;
        super::super::super::ArtifactFrame::from_entries(&self.root, &entries)
    }
    fn classify_observation_data(
        &mut self,
        context:&crate::infrastructure::windows_client_originals024::v3::journal::ObservationSeriesContext,
        declared_routes: Option<&[String]>,
        retained_pids: &[u32],
        audit_home: &str,
        not_before: chrono::DateTime<chrono::Utc>,
        not_after: chrono::DateTime<chrono::Utc>,
        end: Instant,
    ) -> io::Result<crate::modules::runtime::client_artifact_policy::Assessment> {
        let result = (|| {
            let root = self.parent.path().join(".codex");
            if self.root != root || self.work_mode != WorkMode::Observed {
                return Err(invalid());
            }
            let original = self.original_artifacts()?;
            let sequences = context.sequences();
            let mut observations = Vec::new();
            for (i, seq) in sequences.iter().enumerate() {
                let archive = self.archive_for_sequence(*seq, end)?;
                archive.matches_series_context(context.binding(i)?, *seq, context.frame(i, 0)?)?;
                observations.push(archive.artifact_data()?);
            }
            let answer = crate::modules::runtime::client_artifact_policy::assess(
                &crate::modules::runtime::client_artifact_policy::Observations {
                    before: original.stage(),
                    ready: observations[0].stage(),
                    pre_stop: observations[1].stage(),
                    stopped: observations[2].stage(),
                    declared_routes,
                    retained_pids,
                    audit_home,
                    audit_not_before: not_before,
                    audit_not_after: not_after,
                },
            )
            .map_err(|r| io::Error::other(format!("original shared artifact policy: {r:?}")))?;
            check(end)?;
            Ok(answer)
        })();
        result
    }
    fn resume_linked_generation(
        &mut self,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<String> {
        let root = self.parent.path().join(".codex");
        claim.check(&root, end)?;
        self.verify_custody_anchors(end)?;
        if self.work_mode != WorkMode::Observed {
            return Err(invalid());
        }
        let frame = self.observed.as_mut().ok_or_else(invalid)?;
        frame.check(end)?;
        let digest = frame_digest(&frame.metadata())?;
        if frame.archive.is_none() {
            return Err(invalid());
        }
        self.work_mode = WorkMode::Opening;
        for n in &mut frame.nodes {
            if frame_kind(&frame.root, &n.entry.relative, n.entry.directory) != Kind::File {
                continue;
            }
            claim.check(&root, end)?;
            let observer = reopen(&n.file, 0x20080, 1 | 2 | 4)?;
            if id(&observer, false)? != n.entry.id
                || native_stamp(&observer, end).map_err(failure)? != n.entry.security
            {
                return Err(invalid());
            }
            let old = std::mem::replace(&mut n.file, observer);
            drop(old);
        }
        let mut frame = self.observed.take().ok_or_else(invalid)?;
        let archive = frame.archive.take().ok_or_else(invalid)?;
        let frame_root = frame.root.clone();
        let work_objects = frame
            .nodes
            .into_iter()
            .filter(|n| frame_kind(&frame_root, &n.entry.relative, n.entry.directory) == Kind::File)
            .map(|n| (n.file, n.entry.id))
            .collect();
        self.snapshot_history.push(RetainedSnapshot {
            archive,
            work_objects,
        });
        self.work_mode = WorkMode::Open;
        self.verify_custody_anchors(end)?;
        claim.check(&root, end)?;
        Ok(digest)
    }
}
/// A single complete compound observation barrier on BOTH trees surrounds the
/// pure shared-policy call. All original custody, retained generations, every
/// historical archive and current native frame are checked before AND after;
/// no per-row path scan/cache stands in for this barrier or a later mutation.
pub(in crate::infrastructure::windows_client_originals024) fn assess_tree_series(
    trees: &mut [AnchoredTree; 2],
    context:&crate::infrastructure::windows_client_originals024::v3::journal::ObservationSeriesContext,
    declared_routes: Option<&[String]>,
    pids: &[u32],
    audit_home: &str,
    from: chrono::DateTime<chrono::Utc>,
    to: chrono::DateTime<chrono::Utc>,
    claim: &mut AnchoredClaim<'_>,
    end: Instant,
) -> io::Result<crate::modules::runtime::client_artifact_policy::Assessment> {
    let result = (|| {
        if trees[0].parent.identity() != trees[1].parent.identity() {
            return Err(invalid());
        }
        let root = trees[0].parent.path().join(".codex");
        let boundary =
            |trees: &mut [AnchoredTree; 2], claim: &mut AnchoredClaim<'_>| -> io::Result<()> {
                claim.check(&root, end)?;
                for (i, t) in trees.iter_mut().enumerate() {
                    if t.work_mode != WorkMode::Observed
                        || t.root
                            != t.parent
                                .path()
                                .join(if i == 0 { ".codex" } else { ".grok" })
                    {
                        return Err(invalid());
                    }
                    t.verify_custody_anchors(end)?;
                    t.observed.as_mut().ok_or_else(invalid)?.check(end)?;
                    for (stage, seq) in context.sequences().iter().enumerate() {
                        t.archive_for_sequence(*seq, end)?.matches_series_context(
                            context.binding(stage)?,
                            *seq,
                            context.frame(stage, i)?,
                        )?;
                    }
                }
                claim.check(&root, end)
            };
        boundary(trees, claim)?;
        let judgment = trees[0].classify_observation_data(
            context,
            declared_routes,
            pids,
            audit_home,
            from,
            to,
            end,
        );
        boundary(trees, claim)?;
        judgment
    })();
    if result.is_err() {
        for t in trees {
            t.poisoned = true;
        }
    }
    result
}
pub(in crate::infrastructure::windows_client_originals024::tree::anchored) fn resume_trees(
    trees: &mut [AnchoredTree; 2],
    journal: &mut Journal,
    claim: &mut AnchoredClaim<'_>,
    end: Instant,
) -> io::Result<()> {
    let result = (|| {
        let plans = trees.each_ref().map(|t| t.plan());
        let observed = super::snapshots::witness(trees, end)?.link_observation(end)?;
        journal.resume_linked_work(&plans, observed, claim, end, |claim| {
            Ok([
                trees[0].resume_linked_generation(claim, end)?,
                trees[1].resume_linked_generation(claim, end)?,
            ])
        })
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
#[path = "series_tests.rs"]
mod tests;
