//! Actual observed generations reopen only after durable BOTH-root links.
//! Original and initial-work baselines never change. Immutable snapshot objects
//! and exact historical work observers remain retained across official writes.
use super::*;
use crate::infrastructure::windows_client_originals024::v3::Journal;
impl AnchoredTree {
    fn archive_for_sequence(
        &mut self,
        sequence: u32,
        end: Instant,
    ) -> io::Result<&mut archive::FrameArchive> {
        let mut found = 0;
        for old in &mut self.snapshot_history {
            old.check(end)?;
            if old.archive.observation_sequence()? == sequence {
                found += 1;
            }
        }
        let current_matches = if let Some(frame) = &mut self.observed {
            frame.check(end)?;
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
    pub(in crate::infrastructure::windows_client_originals024) fn assess_observation_content(
        &mut self,
        context:&crate::infrastructure::windows_client_originals024::v3::journal::ObservationSeriesContext,
        declared_routes: Option<&[String]>,
        retained_pids: &[u32],
        audit_home: &str,
        not_before: chrono::DateTime<chrono::Utc>,
        not_after: chrono::DateTime<chrono::Utc>,
        claim: &mut AnchoredClaim<'_>,
        end: Instant,
    ) -> io::Result<crate::modules::runtime::client_artifact_policy::Assessment> {
        let result = (|| {
            let root = self.parent.path().join(".codex");
            claim.check(&root, end)?;
            self.verify_custody_anchors(end)?;
            if self.root != root || self.work_mode != WorkMode::Observed {
                return Err(invalid());
            }
            let original = self.original_artifacts()?;
            let sequences = context.sequences();
            let mut observations = Vec::new();
            for (i, seq) in sequences.iter().enumerate() {
                let archive = self.archive_for_sequence(*seq, end)?;
                archive.matches_series_context(context.binding(i)?, *seq, context.frame(i, 0)?)?;
                observations.push(archive.artifact_frame(end)?);
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
            for seq in sequences {
                self.archive_for_sequence(seq, end)?.check(end)?;
            }
            self.verify_custody_anchors(end)?;
            claim.check(&root, end)?;
            Ok(answer)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
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
pub(super) fn resume_trees(
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
