//! Native whole-tree records and same-volume move intentions in the one v2
//! journal. This component cannot launch official processes or issue a Closed
//! eligibility. Production source admission and native exit proofs remain gates.
use super::super::tree::WorkingArchive;
use super::super::{bounded::native_stamp, tree::RootTree};
use super::*;
use crate::infrastructure::windows_official_runtime024::{create_private_directory, RetainedPeer};
use std::{path::PathBuf, time::Instant};
mod history;

fn before(end: Instant) -> io::Result<()> {
    if Instant::now() >= end {
        Err(io::ErrorKind::TimedOut.into())
    } else {
        Ok(())
    }
}
fn stamp(path: &Path, end: Instant) -> io::Result<SnapshotData> {
    use std::os::windows::fs::OpenOptionsExt;
    let file = std::fs::OpenOptions::new()
        .access_mode(0x20000)
        .share_mode(1)
        .custom_flags(0x2200000)
        .open(path)?;
    native_stamp(&file, end).map_err(io::Error::other)
}
/// Borrowed original immutable records and the exclusive actual current file.
/// Only TreeCaptureV2 below can construct this; JSON cannot create an intent.
pub(crate) struct MoveIntent<'a> {
    pending: &'a PendingV2,
    current: &'a mut BoundedCurrentV2,
    custody: &'a PinnedDirectory,
    manifest: &'a Manifest,
}
impl MoveIntent<'_> {
    pub(crate) fn authorize(
        &mut self,
        original: &Path,
        nodes: &[serde_json::Value],
        end: Instant,
    ) -> io::Result<PathBuf> {
        before(end)?;
        self.current.verify_pending(self.pending, end)?;
        self.custody.recheck()?;
        if self.custody.path() != Path::new(&self.manifest.custody.path)
            || self.custody.identity().volume != self.manifest.custody.identity.volume
            || self.custody.identity().index.to_string() != self.manifest.custody.identity.file_id
            || stamp(self.custody.path(), end)?
                != self.manifest.custody.security_projection_snapshot
        {
            return Err(invalid());
        }
        let root = original
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(invalid)?;
        if !matches!(self.pending.tip.tree_record,TreeRecord::Intent{root:record_root,action:TreeAction::MoveToCustody} if record_root.leaf()==root)
        {
            return Err(invalid());
        }
        if ![".codex", ".grok"].contains(&root)
            || original.parent() != Some(self.manifest.binding.paths()[0])
        {
            return Err(invalid());
        }
        let declared = self
            .manifest
            .nodes
            .iter()
            .filter(|node| node.relative == root || node.relative.starts_with(&format!("{root}/")))
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| invalid())?;
        if declared != nodes {
            return Err(invalid());
        }
        before(end)?;
        Ok(self.custody.path().into())
    }
}
/// Retained native original trees plus their actual immutable private journal.
/// Binding digests are data until production source admission binds them. This
/// type intentionally does not implement ClientOriginals or NativeSession yet.
pub(super) struct TreeCaptureV2 {
    trees: [RootTree; 2],
    pins: [PinnedDirectory; 4],
    peer: RetainedPeer,
    custody: PinnedDirectory,
    manifest: Manifest,
    pending: PendingV2,
    current: BoundedCurrentV2,
    working_archives: Vec<(Phase, WorkingArchive)>,
    original_history: Option<WorkingArchive>,
    original_handles_released: bool,
    #[cfg(test)]
    fail_working_after_first_part: bool,
    #[cfg(test)]
    fail_history_after_first_part: bool,
    failed: bool,
}
fn private_child(parent: &PinnedDirectory, name: &str) -> io::Result<PinnedDirectory> {
    parent.recheck()?;
    let path = parent.path().join(name);
    match std::fs::symlink_metadata(&path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => create_private_directory(&path),
        Err(e) => Err(e),
        Ok(_) => PinnedDirectory::open_private(&path),
    }
}
impl TreeCaptureV2 {
    /// Content-only assessment on actual original/archived/current snapshots.
    /// Does not replace the unchanged-working veto, advance phase or restore.
    pub(super) fn assess_artifacts(
        &mut self,
        routes: &[String],
        pids: &[u32],
        audit_home: &str,
        not_before: chrono::DateTime<chrono::Utc>,
        not_after: chrono::DateTime<chrono::Utc>,
        end: Instant,
    ) -> io::Result<crate::modules::runtime::client_artifact_policy::Assessment> {
        let result =
            self.assess_artifacts_inner(routes, pids, audit_home, not_before, not_after, end);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn assess_artifacts_inner(
        &mut self,
        routes: &[String],
        pids: &[u32],
        audit_home: &str,
        not_before: chrono::DateTime<chrono::Utc>,
        not_after: chrono::DateTime<chrono::Utc>,
        end: Instant,
    ) -> io::Result<crate::modules::runtime::client_artifact_policy::Assessment> {
        use crate::modules::runtime::client_artifact_policy::{self as policy, Observations};
        self.recheck(end)?;
        let root = self.manifest.binding.paths()[0].join(".codex");
        self.trees[0].verify_non_artifact_working(end)?;
        self.trees[1].verify_working(end)?;
        let original = self.trees[0].original_artifact_frame(end)?;
        let ready = self
            .working_archives
            .iter_mut()
            .find(|(p, _)| *p == Phase::Ready)
            .ok_or_else(invalid)?
            .1
            .artifact_frame(end)?;
        let prestop = self
            .working_archives
            .iter_mut()
            .find(|(p, _)| *p == Phase::PreparedRestore)
            .ok_or_else(invalid)?
            .1
            .artifact_frame(end)?;
        let current = self.trees[0].current_artifact_frame(end)?;
        for frame in [&original, &ready, &prestop, &current] {
            frame.require_root(&root)?;
        }
        let result = policy::assess(&Observations {
            before: original.stage(),
            ready: ready.stage(),
            pre_stop: prestop.stage(),
            stopped: current.stage(),
            declared_routes: Some(routes),
            retained_pids: pids,
            audit_home,
            audit_not_before: not_before,
            audit_not_after: not_after,
        })
        .map_err(|_| invalid())?;
        self.recheck(end)?;
        before(end)?;
        Ok(result)
    }
    pub(super) fn capture(binding: Binding, end: Instant) -> io::Result<Self> {
        Self::capture_with_history(binding, &mut [], end)
    }
    pub(super) fn capture_with_history(
        binding: Binding,
        histories: &mut [&mut Self],
        end: Instant,
    ) -> io::Result<Self> {
        before(end)?;
        binding.validate(Some(&current_identity()?.sid))?;
        let paths = binding.paths();
        let pins = [
            PinnedDirectory::open(paths[0])?,
            PinnedDirectory::open_private(paths[1])?,
            PinnedDirectory::open(paths[2])?,
            PinnedDirectory::open(paths[3])?,
        ];
        let peer = RetainedPeer::current()?;
        binding.check_native(&peer, &pins)?;
        let official = private_child(&pins[1], "official024")?;
        let windows = private_child(&official, "windows")?;
        let predecessors = Self::native_history_set(&binding, histories, &windows, None, end)?;
        // Inspect BOTH roots in the original state before creating custody or
        // any manifest. Failure cannot have moved either original root.
        let mut trees = [
            RootTree::capture(&paths[0].join(".codex"), end)?,
            RootTree::capture(&paths[0].join(".grok"), end)?,
        ];
        let mut nodes = Vec::new();
        for tree in &mut trees {
            for node in tree.manifest_nodes(end)? {
                nodes.push(serde_json::from_value(node).map_err(|_| invalid())?);
            }
        }
        let session_id = uuid::Uuid::new_v4().to_string();
        let custody =
            create_private_directory(&paths[0].join(format!(".ocxd024-originals-{session_id}")))?;
        let custody_data = Custody {
            path: custody.path().to_str().ok_or_else(invalid)?.into(),
            identity: Identity {
                volume: custody.identity().volume,
                file_id: custody.identity().index.to_string(),
            },
            security_projection_snapshot: stamp(custody.path(), end)?,
        };
        let policy = Policy::fixed();
        let manifest = Manifest {
            schema: ORIGINALS_RECORD_SCHEMA_V2.into(),
            document_kind: "manifest".into(),
            session_id: session_id.clone(),
            binding,
            security_policy_digest: policy.digest()?,
            security_policy: policy,
            custody: custody_data,
            nodes,
            predecessors,
        };
        check_manifest(&manifest, &current_identity()?.sid)?;
        let raw = serde_json::to_vec(&manifest).map_err(|_| invalid())?;
        if raw.len() > 1024 * 1024 {
            return Err(invalid());
        }
        let directory = create_private_directory(&windows.path().join(&session_id))?;
        if Self::native_history_set(
            &manifest.binding,
            histories,
            &windows,
            Some(&directory),
            end,
        )? != manifest.predecessors
        {
            return Err(invalid());
        }
        before(end)?;
        let mut record =
            PrivateRecord::create(&directory.path().join("manifest.json"), &raw, 1024 * 1024)?;
        if record.read()? != raw {
            return Err(invalid());
        }
        let stage = Stage {
            schema: ORIGINALS_RECORD_SCHEMA_V2.into(),
            document_kind: "phase".into(),
            session_id,
            sequence: 1,
            previous_digest: String::new(),
            previous_record_id: None,
            binding: serde_json::from_slice(
                &serde_json::to_vec(&manifest.binding).map_err(|_| invalid())?,
            )
            .map_err(|_| invalid())?,
            phase: Phase::Captured,
            tree_record: TreeRecord::None,
            working_record: WorkingRecord::None,
            history_record: WorkingRecord::None,
            originals_manifest_digest: hash::sha256_hex(&raw),
            security_policy_digest: manifest.security_policy_digest.clone(),
        };
        let stage_raw = serde_json::to_vec(&stage).map_err(|_| invalid())?;
        let phase = PrivateRecord::create(
            &directory.path().join("phase-00000001.json"),
            &stage_raw,
            65536,
        )?;
        let pending = read_pending(directory.path())?;
        let current = BoundedCurrentV2::create(&pending, end)?;
        drop(phase);
        drop(record);
        drop(directory);
        let mut out = Self {
            trees,
            pins,
            peer,
            custody,
            manifest,
            pending,
            current,
            working_archives: Vec::new(),
            original_history: None,
            original_handles_released: false,
            #[cfg(test)]
            fail_working_after_first_part: false,
            #[cfg(test)]
            fail_history_after_first_part: false,
            failed: false,
        };
        out.recheck(end)?;
        if Self::native_history_set(
            &out.manifest.binding,
            histories,
            &windows,
            Some(&out.pending.directory),
            end,
        )? != out.manifest.predecessors
        {
            return Err(invalid());
        }
        Ok(out)
    }
    pub(super) fn recheck(&mut self, end: Instant) -> io::Result<()> {
        let result = self.recheck_inner(end);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn recheck_inner(&mut self, end: Instant) -> io::Result<()> {
        before(end)?;
        if self.failed {
            return Err(invalid());
        }
        self.manifest.binding.check_native(&self.peer, &self.pins)?;
        self.current.verify_pending(&self.pending, end)?;
        self.custody.recheck()?;
        if stamp(self.custody.path(), end)? != self.manifest.custody.security_projection_snapshot {
            return Err(invalid());
        }
        if self.original_handles_released {
            if self.pending.tip.phase != Phase::Closed {
                return Err(invalid());
            }
            self.original_history
                .as_mut()
                .ok_or_else(invalid)?
                .check_original_history(&mut self.trees, end)?;
        } else {
            for tree in &mut self.trees {
                tree.manifest_nodes(end)?;
            }
        }
        for (phase, archive) in &mut self.working_archives {
            let reference = archive.reference(end)?;
            let observed = self
                .pending
                .phase_raw
                .iter()
                .map(|raw| json::<Stage>(raw, 65536))
                .collect::<io::Result<Vec<_>>>()?
                .into_iter()
                .find(|stage| stage.phase == *phase && stage.tree_record == TreeRecord::None)
                .ok_or_else(invalid)?;
            match observed.working_record {
                WorkingRecord::Snapshot(expected) if expected == reference => {}
                _ => return Err(invalid()),
            }
        }
        if let Some(archive) = &mut self.original_history {
            let reference = archive.reference(end)?;
            let stage: Stage = json(self.pending.phase_raw.last().ok_or_else(invalid)?, 65536)?;
            match stage.history_record {
                WorkingRecord::Snapshot(expected)
                    if stage.phase == Phase::Closed && expected == reference => {}
                _ => return Err(invalid()),
            }
        }
        let mut expected = Vec::new();
        for tree in &self.trees {
            expected.extend(tree.retained_custody_names(self.custody.path())?);
        }
        for (phase, _) in &self.working_archives {
            expected.push(
                match phase {
                    Phase::Ready => "ready",
                    Phase::PreparedRestore => "pre-stop",
                    _ => return Err(invalid()),
                }
                .into(),
            );
        }
        if self.original_history.is_some() {
            expected.push("original-history".into());
        }
        expected.sort();
        if expected.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(invalid());
        }
        let mut observed = std::fs::read_dir(self.custody.path())?
            .take(17)
            .map(|row| row.and_then(|row| row.file_name().into_string().map_err(|_| invalid())))
            .collect::<io::Result<Vec<_>>>()?;
        observed.sort();
        if observed != expected {
            return Err(invalid());
        }
        before(end)
    }
    fn move_root(&mut self, index: usize, end: Instant) -> io::Result<()> {
        self.recheck(end)?;
        self.begin_tree(index, TreeAction::MoveToCustody, end)?;
        let tree = self.trees.get_mut(index).ok_or_else(invalid)?;
        let mut intent = MoveIntent {
            pending: &self.pending,
            current: &mut self.current,
            custody: &self.custody,
            manifest: &self.manifest,
        };
        tree.move_from_manifest(&mut intent, end)?;
        self.complete_tree(index, TreeAction::MoveToCustody, end)
    }
    fn begin_tree(&mut self, index: usize, action: TreeAction, end: Instant) -> io::Result<()> {
        self.recheck(end)?;
        let root = TreeRoot::from_index(index)?;
        self.failed = true;
        self.pending = self.current.append_tree_record(
            &self.pending,
            TreeRecord::Intent { root, action },
            end,
        )?;
        Ok(())
    }
    fn complete_tree(&mut self, index: usize, action: TreeAction, end: Instant) -> io::Result<()> {
        before(end)?;
        self.manifest.binding.check_native(&self.peer, &self.pins)?;
        let root = TreeRoot::from_index(index)?;
        // Original tree ID/bytes/projections must remain true before the record
        // may describe a completed operation. Any write/postcheck failure keeps
        // this capture poisoned and its last durable intention in place.
        let tree = self.trees.get_mut(index).ok_or_else(invalid)?;
        match action {
            TreeAction::MoveToCustody => tree.verify_moved(end)?,
            TreeAction::CreateWorkingCopy => tree.verify_working(end)?,
            TreeAction::RestoreOriginal => tree.verify_restored(end)?,
        }
        self.pending = self.current.append_tree_record(
            &self.pending,
            TreeRecord::Completed { root, action },
            end,
        )?;
        self.failed = false;
        if let Err(error) = self.recheck(end) {
            self.failed = true;
            return Err(error);
        }
        Ok(())
    }
    pub(super) fn create_working_trees(&mut self, end: Instant) -> io::Result<()> {
        for index in 0..self.trees.len() {
            self.move_root(index, end)?;
            self.begin_tree(index, TreeAction::CreateWorkingCopy, end)?;
            self.trees[index].create_working_copy(end)?;
            self.complete_tree(index, TreeAction::CreateWorkingCopy, end)?;
        }
        self.recheck(end)
    }
    /// Component restoration only: no official process can be created through
    /// this component. Aborted remains recovery-required; no Restored/Closed
    /// record or eligibility is emitted for this nonproduction mechanism.
    pub(super) fn restore_exact(&mut self, end: Instant) -> io::Result<()> {
        self.recheck(end)?;
        if self.pending.tip.phase != Phase::Captured {
            return Err(invalid());
        }
        self.failed = true;
        self.pending = self
            .current
            .append_pending(&self.pending, Phase::Aborted, end)?;
        self.failed = false;
        for index in 0..self.trees.len() {
            self.restore_root(index, end)?;
        }
        self.recheck(end)
    }
    fn restore_root(&mut self, index: usize, end: Instant) -> io::Result<()> {
        self.begin_tree(index, TreeAction::RestoreOriginal, end)?;
        self.trees[index].restore_original(end)?;
        self.complete_tree(index, TreeAction::RestoreOriginal, end)
    }
    pub(super) fn session_id(&self) -> &str {
        &self.manifest.session_id
    }
    pub(super) fn binding_bytes(&self) -> io::Result<Vec<u8>> {
        serde_json::to_vec(&self.manifest.binding).map_err(|_| invalid())
    }
    pub(super) fn advance_phase(&mut self, phase: Phase, end: Instant) -> io::Result<()> {
        self.recheck(end)?;
        self.failed = true;
        self.pending = self.current.append_pending(&self.pending, phase, end)?;
        self.failed = false;
        if let Err(error) = self.recheck(end) {
            self.failed = true;
            return Err(error);
        }
        Ok(())
    }
    /// Native storage part of the typed close driver; no process proof is
    /// generated here. Partial data, phase or current failure preserves state.
    pub(super) fn seal_closed_originals(&mut self, end: Instant) -> io::Result<()> {
        self.recheck(end)?;
        if self.pending.tip.phase != Phase::Restored || self.original_history.is_some() {
            return Err(invalid());
        }
        self.failed = true;
        let binding = hash::sha256_hex(&self.binding_bytes()?);
        self.original_history = Some(WorkingArchive::create(
            &self.custody,
            SnapshotPhase::OriginalHistory,
            &self.manifest.session_id,
            &binding,
            &self.manifest.security_policy_digest,
            end,
        )?);
        let archive = self.original_history.as_mut().unwrap();
        #[cfg(test)]
        if self.fail_history_after_first_part {
            archive.inject_first_part_failure();
        }
        archive.capture(&mut self.trees, end)?;
        archive.matches_live(&mut self.trees, end)?;
        let reference = archive.reference(end)?;
        self.pending = self
            .current
            .append_closed_history(&self.pending, reference, end)?;
        self.failed = false;
        if let Err(error) = self.recheck(end) {
            self.failed = true;
            return Err(error);
        }
        Ok(())
    }
    pub(super) fn release_closed_original_handles(&mut self, end: Instant) -> io::Result<()> {
        self.recheck(end)?;
        if self.pending.tip.phase != Phase::Closed || self.original_handles_released {
            return Err(invalid());
        }
        self.original_history
            .as_mut()
            .ok_or_else(invalid)?
            .matches_live(&mut self.trees, end)?;
        self.failed = true;
        for tree in &mut self.trees {
            tree.release_closed_original_handle(end)?;
        }
        self.original_handles_released = true;
        self.failed = false;
        if let Err(error) = self.recheck(end) {
            self.failed = true;
            return Err(error);
        }
        Ok(())
    }
    /// Observe and durably copy the ACTUAL working generation. This does not
    /// absorb it as an allowed official mutation. Unknown content keeps the
    /// archive and original canonical prefix, but never returns a phase tag.
    pub(super) fn record_working_phase(&mut self, phase: Phase, end: Instant) -> io::Result<()> {
        self.recheck(end)?;
        let expected = match (self.pending.tip.phase, phase) {
            (Phase::Captured, Phase::Ready) => SnapshotPhase::Ready,
            (Phase::Ready, Phase::PreparedRestore) => SnapshotPhase::PreStop,
            _ => return Err(invalid()),
        };
        if self
            .working_archives
            .iter()
            .any(|(existing, _)| *existing == phase)
        {
            return Err(invalid());
        }
        self.failed = true;
        let binding = hash::sha256_hex(&self.binding_bytes()?);
        let archive = WorkingArchive::create(
            &self.custody,
            expected,
            &self.manifest.session_id,
            &binding,
            &self.manifest.security_policy_digest,
            end,
        )?;
        self.working_archives.push((phase, archive));
        let archive = &mut self.working_archives.last_mut().unwrap().1;
        #[cfg(test)]
        if self.fail_working_after_first_part {
            archive.inject_first_part_failure();
        }
        archive.capture(&mut self.trees, end)?;
        // Full declared-side-effect assessment remains a separate requirement.
        // Do not turn the newly archived bytes into an adopted working baseline.
        for tree in &mut self.trees {
            tree.verify_working(end)?;
        }
        archive.matches_live(&mut self.trees, end)?;
        let reference = archive.reference(end)?;
        self.pending = self
            .current
            .append_working_phase(&self.pending, phase, reference, end)?;
        self.failed = false;
        if let Err(error) = self.recheck(end) {
            self.failed = true;
            return Err(error);
        }
        Ok(())
    }
    /// Conservative admission until declared official changes are implemented:
    /// never absorb a new generation/content merely because the service is Ready.
    pub(super) fn verify_working_unchanged(&mut self, end: Instant) -> io::Result<()> {
        self.recheck(end)?;
        for tree in &mut self.trees {
            tree.verify_working(end)?;
        }
        before(end)
    }
    pub(super) fn restore_after_teardown(&mut self, end: Instant) -> io::Result<()> {
        self.recheck(end)?;
        if self.pending.tip.phase != Phase::OfficialTeardown {
            return Err(invalid());
        }
        for index in 0..self.trees.len() {
            self.restore_root(index, end)?;
        }
        self.advance_phase(Phase::Restored, end)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::tree::tests::Fixture;
    use super::*;
    fn end() -> Instant {
        Instant::now() + std::time::Duration::from_secs(10)
    }
    fn close_storage(captured: &mut TreeCaptureV2, deadline: Instant) {
        captured.create_working_trees(deadline).unwrap();
        captured
            .record_working_phase(Phase::Ready, deadline)
            .unwrap();
        captured
            .record_working_phase(Phase::PreparedRestore, deadline)
            .unwrap();
        captured
            .advance_phase(Phase::OfficialTeardown, deadline)
            .unwrap();
        captured.restore_after_teardown(deadline).unwrap();
        captured.seal_closed_originals(deadline).unwrap();
        captured.release_closed_original_handles(deadline).unwrap();
    }
    fn close_storage_with_history(
        captured: &mut TreeCaptureV2,
        previous: &mut [&mut TreeCaptureV2],
        deadline: Instant,
    ) {
        captured
            .create_working_trees_with_history(previous, deadline)
            .unwrap();
        captured
            .record_working_phase(Phase::Ready, deadline)
            .unwrap();
        captured.recheck_with_history(previous, deadline).unwrap();
        captured
            .record_working_phase(Phase::PreparedRestore, deadline)
            .unwrap();
        captured
            .advance_phase(Phase::OfficialTeardown, deadline)
            .unwrap();
        captured
            .restore_after_teardown_with_history(previous, deadline)
            .unwrap();
        captured.seal_closed_originals(deadline).unwrap();
        captured.release_closed_original_handles(deadline).unwrap();
        captured.recheck_with_history(previous, deadline).unwrap();
    }
    #[test]
    fn wl2_r4_successor_three_native_generations_keep_exact_predecessors() {
        let mut temp = Fixture::new();
        let binding = setup(&temp);
        let binding_raw = serde_json::to_vec(&binding).unwrap();
        let profile = binding.paths()[0].to_path_buf();
        let data = binding.paths()[1].to_path_buf();
        let mut first = TreeCaptureV2::capture(binding, end()).unwrap();
        close_storage(&mut first, end());
        let mut histories = vec![first];
        let mut manifests = vec![histories[0].pending.manifest_raw.clone()];
        for generation in 2..=3 {
            std::fs::write(
                profile.join(".codex/config.toml"),
                format!("synthetic user original generation{generation}"),
            )
            .unwrap();
            let binding: Binding = serde_json::from_slice(&binding_raw).unwrap();
            let mut previous = histories.iter_mut().collect::<Vec<_>>();
            let mut current =
                TreeCaptureV2::capture_with_history(binding, &mut previous, end()).unwrap();
            assert_eq!(current.manifest.predecessors.len(), generation - 1);
            current.recheck_with_history(&mut previous, end()).unwrap();
            close_storage_with_history(&mut current, &mut previous, end());
            current.recheck_with_history(&mut previous, end()).unwrap();
            assert_eq!(
                std::fs::read(profile.join(".codex/config.toml")).unwrap(),
                format!("synthetic user original generation{generation}").as_bytes()
            );
            drop(previous);
            for (old, expected) in histories.iter().zip(&manifests) {
                assert_eq!(&old.pending.manifest_raw, expected);
            }
            manifests.push(current.pending.manifest_raw.clone());
            histories.push(current);
        }
        assert_eq!(
            std::fs::read_dir(data.join("official024/windows"))
                .unwrap()
                .count(),
            3
        );
        let binding: Binding = serde_json::from_slice(&binding_raw).unwrap();
        let windows = PinnedDirectory::open_private(&data.join("official024/windows")).unwrap();
        let mut reversed = histories.iter_mut().rev().collect::<Vec<_>>();
        assert!(
            TreeCaptureV2::native_history_set(&binding, &mut reversed, &windows, None, end())
                .is_err()
        );
        drop(reversed);
        let mut sparse = histories
            .iter_mut()
            .enumerate()
            .filter_map(|(i, value)| (i != 1).then_some(value))
            .collect::<Vec<_>>();
        assert!(
            TreeCaptureV2::native_history_set(&binding, &mut sparse, &windows, None, end())
                .is_err()
        );
        drop(sparse);
        let refs = TreeCaptureV2::native_history_set(
            &binding,
            &mut histories.iter_mut().collect::<Vec<_>>(),
            &windows,
            None,
            end(),
        )
        .unwrap();
        assert_eq!(refs.len(), 3);
        assert_eq!(
            std::fs::read(
                histories[0]
                    .custody
                    .path()
                    .join("original-history/.codex/data-0001-0000.bin")
            )
            .unwrap(),
            b"synthetic nonsecret config"
        );
        drop(histories);
        drop(windows);
        println!("three actual native-storage generations create independent original manifests/current/custody and ordered predecessor refs, keep original histories unchanged after new user bytes, preserve exact live original each cycle; storage fixture only, no official ClosedEligibility/Start budget or Owner adoption");
        temp.retain();
    }
    #[test]
    fn wl2_r4_successor_old_history_drift_before_first_move_preserves_new_original() {
        let mut temp = Fixture::new();
        let binding = setup(&temp);
        let raw = serde_json::to_vec(&binding).unwrap();
        let profile = binding.paths()[0].to_path_buf();
        let mut old = TreeCaptureV2::capture(binding, end()).unwrap();
        close_storage(&mut old, end());
        std::fs::write(
            profile.join(".codex/config.toml"),
            b"fresh synthetic new original",
        )
        .unwrap();
        let mut current = TreeCaptureV2::capture_with_history(
            serde_json::from_slice(&raw).unwrap(),
            &mut [&mut old],
            end(),
        )
        .unwrap();
        let foreign = old.custody.path().join("foreign.new");
        std::fs::write(&foreign, b"old native history changed after new capture").unwrap();
        assert!(current
            .create_working_trees_with_history(&mut [&mut old], end())
            .is_err());
        assert!(current.failed && old.failed);
        assert!(!current.custody.path().join(".codex").exists());
        assert_eq!(
            std::fs::read(profile.join(".codex/config.toml")).unwrap(),
            b"fresh synthetic new original"
        );
        let directory = current.pending.directory.path().to_path_buf();
        assert!(!directory.join("phase-00000002.json").exists());
        drop(current);
        drop(old);
        let pending = read_pending(&directory).unwrap();
        assert_eq!(pending.tip.sequence, 1);
        assert!(pending.current_matches && !pending.terminal_record_only);
        drop(pending);
        println!("old native history drift after successor capture blocks first root move, poisons both native stores, leaves new originals/phase1/current and foreign old scene intact; no permission or catch-up");
        temp.retain();
    }
    #[test]
    fn wl2_r4_successor_missing_history_or_foreign_namespace_never_adopts() {
        let mut temp = Fixture::new();
        let binding = setup(&temp);
        let raw = serde_json::to_vec(&binding).unwrap();
        let profile = binding.paths()[0].to_path_buf();
        let data = binding.paths()[1].to_path_buf();
        let mut old = TreeCaptureV2::capture(binding, end()).unwrap();
        close_storage(&mut old, end());
        assert!(TreeCaptureV2::capture(serde_json::from_slice(&raw).unwrap(), end()).is_err());
        let unknown = data.join("official024/windows/foreign.json");
        std::fs::write(&unknown, b"{\"phase\":\"Closed\"}").unwrap();
        assert!(TreeCaptureV2::capture_with_history(
            serde_json::from_slice(&raw).unwrap(),
            &mut [&mut old],
            end()
        )
        .is_err());
        assert_eq!(
            std::fs::read(profile.join(".codex/config.toml")).unwrap(),
            b"synthetic nonsecret config"
        );
        assert_eq!(
            std::fs::read_dir(data.join("official024/windows"))
                .unwrap()
                .count(),
            2
        );
        assert!(unknown.exists());
        drop(old);
        println!("missing actual native history or unknown exact namespace child refuses new capture before original moves; text Closed is not adopted, foreign record and old original history retained");
        temp.retain();
    }
    #[test]
    fn wl2_r4_successor_source_change_or_old_archive_drift_blocks_new_capture() {
        let mut temp = Fixture::new();
        let binding = setup(&temp);
        let raw = serde_json::to_vec(&binding).unwrap();
        let profile = binding.paths()[0].to_path_buf();
        let data = binding.paths()[1].to_path_buf();
        let mut old = TreeCaptureV2::capture(binding, end()).unwrap();
        close_storage(&mut old, end());
        let mut changed: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        changed["sourceDigest"] = serde_json::json!("f".repeat(64));
        assert!(TreeCaptureV2::capture_with_history(
            serde_json::from_value(changed).unwrap(),
            &mut [&mut old],
            end()
        )
        .is_err());
        let foreign = old.custody.path().join("foreign.new");
        std::fs::write(&foreign, b"unknown historical custody drift").unwrap();
        assert!(TreeCaptureV2::capture_with_history(
            serde_json::from_slice(&raw).unwrap(),
            &mut [&mut old],
            end()
        )
        .is_err());
        assert!(old.failed);
        assert_eq!(
            std::fs::read_dir(data.join("official024/windows"))
                .unwrap()
                .count(),
            1
        );
        assert_eq!(
            std::fs::read(profile.join(".codex/config.toml")).unwrap(),
            b"synthetic nonsecret config"
        );
        assert!(foreign.exists());
        drop(old);
        println!("different source binding and old archive native drift refuse successor before any new generation/root mutation; old poisoned native history preserved, no catch-up");
        temp.retain();
    }
    fn setup(temp: &Fixture) -> Binding {
        let pins = ["Profile", "D", "H", "I"]
            .map(|n| create_private_directory(&temp.path().join(n)).unwrap());
        std::fs::create_dir(pins[0].path().join(".codex")).unwrap();
        std::fs::write(
            pins[0].path().join(".codex/config.toml"),
            b"synthetic nonsecret config",
        )
        .unwrap();
        let peer = RetainedPeer::current().unwrap();
        Binding::from_native(
            &peer,
            &pins,
            hash::sha256_hex(b"synthetic source; not official admission"),
            hash::sha256_hex(b"synthetic current"),
        )
        .unwrap()
    }
    #[test]
    fn wl2_r4_closed_history_unknown_custody_child_keeps_required() {
        let mut temp = Fixture::new();
        let mut captured = TreeCaptureV2::capture(setup(&temp), end()).unwrap();
        captured.create_working_trees(end()).unwrap();
        captured.record_working_phase(Phase::Ready, end()).unwrap();
        captured
            .record_working_phase(Phase::PreparedRestore, end())
            .unwrap();
        captured
            .advance_phase(Phase::OfficialTeardown, end())
            .unwrap();
        captured.restore_after_teardown(end()).unwrap();
        captured.seal_closed_originals(end()).unwrap();
        captured.release_closed_original_handles(end()).unwrap();
        let foreign = captured.custody.path().join("foreign.new");
        std::fs::write(&foreign, b"unknown direct custody child").unwrap();
        assert!(captured.recheck(end()).is_err());
        assert!(captured.failed);
        assert!(captured.recheck(end()).is_err());
        assert!(foreign.exists());
        let directory = captured.pending.directory.path().to_path_buf();
        drop(captured);
        let pending = read_pending(&directory).unwrap();
        assert!(pending.current_matches && pending.terminal_record_only);
        drop(pending);
        println!("exact native custody membership rejects an unknown top-level child after root release, poisons the native capture and preserves text Closed only as pending data; no deletion or successor permission");
        temp.retain();
    }
    #[test]
    fn wl2_r4_closed_history_is_independent_of_later_live_clients() {
        let mut temp = Fixture::new();
        let mut captured = TreeCaptureV2::capture(setup(&temp), end()).unwrap();
        let profile = captured.pins[0].path().to_path_buf();
        captured.create_working_trees(end()).unwrap();
        captured.record_working_phase(Phase::Ready, end()).unwrap();
        captured
            .record_working_phase(Phase::PreparedRestore, end())
            .unwrap();
        captured
            .advance_phase(Phase::OfficialTeardown, end())
            .unwrap();
        captured.restore_after_teardown(end()).unwrap();
        assert!(captured.release_closed_original_handles(end()).is_err());
        assert!(!captured.original_handles_released);
        captured.seal_closed_originals(end()).unwrap();
        // This is a native STORAGE fixture, not an adapter ClosedEligibility;
        // it has never supplied actual official typed exit/Owner/source proofs.
        assert!(RootTree::capture(&profile.join(".codex"), end()).is_err());
        captured.release_closed_original_handles(end()).unwrap();
        assert!(captured.original_handles_released);
        let current_file = profile.join(".codex/config.toml");
        std::fs::write(&current_file, b"new user generation after stored close").unwrap();
        std::fs::create_dir(profile.join(".grok")).unwrap();
        std::fs::write(profile.join(".grok/settings.json"), b"new user grok").unwrap();
        let mut fresh = RootTree::capture(&profile.join(".codex"), end()).unwrap();
        fresh.manifest_nodes(end()).unwrap();
        drop(fresh);
        captured.recheck(end()).unwrap();
        let archive = captured
            .custody
            .path()
            .join("original-history/.codex/data-0001-0000.bin");
        assert_eq!(
            std::fs::read(&archive).unwrap(),
            b"synthetic nonsecret config"
        );
        let absent: serde_json::Value = serde_json::from_slice(
            &std::fs::read(
                captured
                    .custody
                    .path()
                    .join("original-history/.grok/snapshot.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(absent["sourceAbsent"], true);
        assert_eq!(absent["nodes"].as_array().unwrap().len(), 0);
        assert!(std::fs::write(&archive, b"tamper").is_err());
        assert!(captured.release_closed_original_handles(end()).is_err());
        let directory = captured.pending.directory.path().to_path_buf();
        assert_eq!(captured.pending.tip.sequence, 18);
        drop(captured);
        let pending = read_pending(&directory).unwrap();
        assert!(pending.current_matches && pending.terminal_record_only);
        drop(pending);
        assert_eq!(
            std::fs::read(&current_file).unwrap(),
            b"new user generation after stored close"
        );
        println!("native storage Closed history preserves exact old original bytes/absence; actual old DELETE roots released only after archive/current, fresh native capture works, later live clients do not become historical baseline; no official ClosedEligibility/successor permit");
        temp.retain();
    }
    #[test]
    fn wl2_r4_closed_history_drift_refuses_after_root_release() {
        let mut temp = Fixture::new();
        let mut captured = TreeCaptureV2::capture(setup(&temp), end()).unwrap();
        captured.create_working_trees(end()).unwrap();
        captured.record_working_phase(Phase::Ready, end()).unwrap();
        captured
            .record_working_phase(Phase::PreparedRestore, end())
            .unwrap();
        captured
            .advance_phase(Phase::OfficialTeardown, end())
            .unwrap();
        captured.restore_after_teardown(end()).unwrap();
        captured.seal_closed_originals(end()).unwrap();
        captured.release_closed_original_handles(end()).unwrap();
        let foreign = captured
            .custody
            .path()
            .join("original-history/.grok/foreign.new");
        std::fs::write(&foreign, b"unknown stored-history drift").unwrap();
        assert!(captured.recheck(end()).is_err());
        assert!(captured.recheck(end()).is_err());
        assert!(foreign.exists());
        drop(captured);
        temp.retain();
        println!("released-root historical archive unknown file causes native refusal, not automatic history adoption/deletion; exact scene retained, no successor proof");
    }
    #[test]
    fn wl2_r4_closed_history_first_part_failure_retains_native_roots() {
        let mut temp = Fixture::new();
        let mut captured = TreeCaptureV2::capture(setup(&temp), end()).unwrap();
        captured.create_working_trees(end()).unwrap();
        captured.record_working_phase(Phase::Ready, end()).unwrap();
        captured
            .record_working_phase(Phase::PreparedRestore, end())
            .unwrap();
        captured
            .advance_phase(Phase::OfficialTeardown, end())
            .unwrap();
        captured.restore_after_teardown(end()).unwrap();
        captured.fail_history_after_first_part = true;
        let error = captured.seal_closed_originals(end()).unwrap_err();
        assert!(error
            .to_string()
            .contains("injected after actual first sealed"));
        assert!(captured.failed && !captured.original_handles_released);
        let history = captured.custody.path().join("original-history/.codex");
        assert_eq!(
            std::fs::read(history.join("data-0001-0000.bin")).unwrap(),
            b"synthetic nonsecret config"
        );
        assert!(!history.join("snapshot.json").exists());
        assert!(RootTree::capture(&captured.pins[0].path().join(".codex"), end()).is_err());
        assert!(captured.release_closed_original_handles(end()).is_err());
        let directory = captured.pending.directory.path().to_path_buf();
        assert!(!directory.join("phase-00000018.json").exists());
        drop(captured);
        let pending = read_pending(&directory).unwrap();
        assert_eq!(pending.tip.sequence, 17);
        assert!(pending.current_matches && !pending.terminal_record_only);
        drop(pending);
        println!("native original-history first sealed part fault retains partial private bytes and exact restored root handles, canonical Restored17/current unchanged, no Closed or release/catch-up");
        temp.retain();
    }
    #[test]
    fn wl2_r4_closed_history_partial_current_never_releases_originals() {
        let mut temp = Fixture::new();
        let mut captured = TreeCaptureV2::capture(setup(&temp), end()).unwrap();
        captured.create_working_trees(end()).unwrap();
        captured.record_working_phase(Phase::Ready, end()).unwrap();
        captured
            .record_working_phase(Phase::PreparedRestore, end())
            .unwrap();
        captured
            .advance_phase(Phase::OfficialTeardown, end())
            .unwrap();
        captured.restore_after_teardown(end()).unwrap();
        captured.current.inject_partial_write_failure();
        let error = captured.seal_closed_originals(end()).unwrap_err();
        assert!(error
            .to_string()
            .contains("injected after real partial current write and sync"));
        assert!(captured.failed);
        assert!(!captured.original_handles_released);
        let profile = captured.pins[0].path().to_path_buf();
        assert!(RootTree::capture(&profile.join(".codex"), end()).is_err());
        let archive = captured
            .custody
            .path()
            .join("original-history/.codex/data-0001-0000.bin");
        assert_eq!(
            std::fs::read(archive).unwrap(),
            b"synthetic nonsecret config"
        );
        assert!(captured.release_closed_original_handles(end()).is_err());
        let directory = captured.pending.directory.path().to_path_buf();
        assert!(directory.join("phase-00000018.json").exists());
        assert!(!directory.join("phase-00000019.json").exists());
        drop(captured);
        assert_eq!(std::fs::read(directory.join("current.json")).unwrap(), b"{");
        assert!(read_pending(&directory).is_err());
        let pending = read_pending_impl(&directory, false).unwrap();
        assert!(pending.terminal_record_only && !pending.current_matches);
        drop(pending);
        println!("original-history+Closed phase18 then actual partial current fault preserves old native root handles/archive, no release or catch-up; text Closed is locator-only without matching current/typed native closure");
        temp.retain();
    }
    #[test]
    fn wl2_r4_working_first_part_failure_keeps_original_and_canonical_prefix() {
        let mut temp = Fixture::new();
        let binding = setup(&temp);
        let profile = binding.paths()[0].to_path_buf();
        let payload = vec![0x42; 2 * 1024 * 1024];
        std::fs::write(profile.join(".codex/blob.bin"), &payload).unwrap();
        let deadline = end();
        let mut captured = TreeCaptureV2::capture(binding, deadline).unwrap();
        captured.create_working_trees(deadline).unwrap();
        captured.fail_working_after_first_part = true;
        let error = captured
            .record_working_phase(Phase::Ready, deadline)
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("injected after actual first sealed"));
        assert!(captured.failed);
        let partial = captured
            .custody
            .path()
            .join("ready/.codex/data-0001-0000.bin");
        assert_eq!(std::fs::read(&partial).unwrap(), payload[..1024 * 1024]);
        assert!(!partial
            .parent()
            .unwrap()
            .join("data-0001-0001.bin")
            .exists());
        assert!(!partial.parent().unwrap().join("snapshot.json").exists());
        assert_eq!(
            std::fs::read(captured.custody.path().join(".codex/blob.bin")).unwrap(),
            payload
        );
        let directory = captured.pending.directory.path().to_path_buf();
        assert!(!directory.join("phase-00000010.json").exists());
        assert!(captured
            .record_working_phase(Phase::Ready, deadline)
            .is_err());
        drop(captured);
        let pending = read_pending(&directory).unwrap();
        assert_eq!(pending.tip.sequence, 9);
        assert!(pending.current_matches && !pending.terminal_record_only);
        drop(pending);
        println!("actual first1MiB sealed/synced working part then injected failure: partial private data/original custody retained, canonical Captured/current unchanged, poisoned driver refuses retry, no Ready/metadata completion/catch-up");
        temp.retain();
    }
    #[test]
    fn wl2_r4_working_partial_current_keeps_archive_without_ready_authority() {
        let mut temp = Fixture::new();
        let mut captured = TreeCaptureV2::capture(setup(&temp), end()).unwrap();
        captured.create_working_trees(end()).unwrap();
        captured.current.inject_partial_write_failure();
        assert!(captured.record_working_phase(Phase::Ready, end()).is_err());
        assert!(captured.failed);
        let directory = captured.pending.directory.path().to_path_buf();
        let phase = std::fs::read(directory.join("phase-00000010.json")).unwrap();
        let stage: Stage = json(&phase, 65536).unwrap();
        assert!(matches!(stage.working_record, WorkingRecord::Snapshot(_)));
        assert_eq!(
            std::fs::read(
                captured
                    .custody
                    .path()
                    .join("ready/.codex/data-0001-0000.bin")
            )
            .unwrap(),
            b"synthetic nonsecret config"
        );
        assert!(captured.record_working_phase(Phase::Ready, end()).is_err());
        assert!(!directory.join("phase-00000011.json").exists());
        drop(captured);
        assert_eq!(std::fs::read(directory.join("current.json")).unwrap(), b"{");
        assert!(read_pending(&directory).is_err());
        let located = read_pending_impl(&directory, false).unwrap();
        assert_eq!(located.tip.sequence, 10);
        assert!(!located.current_matches && !located.terminal_record_only);
        drop(located);
        println!("actual Ready snapshot followed by current one-byte write+sync injected failure: phase10/archive/original custody retained, current corrupt and pending-only locator available; no Ready permit/retry/catch-up");
        temp.retain();
    }
    #[test]
    fn wl2_r4_working_ready_and_pre_stop_archive_real_bytes_and_original_limit() {
        let mut temp = Fixture::new();
        let binding = setup(&temp);
        let profile = binding.paths()[0].to_path_buf();
        let payload = vec![0x5a; 2 * 1024 * 1024];
        std::fs::write(profile.join(".codex/blob.bin"), &payload).unwrap();
        let deadline = end();
        let mut captured = TreeCaptureV2::capture(binding, deadline).unwrap();
        captured.create_working_trees(deadline).unwrap();
        captured
            .record_working_phase(Phase::Ready, deadline)
            .unwrap();
        captured
            .record_working_phase(Phase::PreparedRestore, deadline)
            .unwrap();
        captured.recheck(deadline).unwrap();
        for stage in ["ready", "pre-stop"] {
            let root = captured.custody.path().join(stage).join(".codex");
            for part in 0..2 {
                let path = root.join(format!("data-0001-{part:04}.bin"));
                assert_eq!(
                    std::fs::read(&path).unwrap(),
                    payload[part * 1024 * 1024..(part + 1) * 1024 * 1024]
                );
                assert!(std::fs::write(&path, b"tamper").is_err());
            }
            assert!(root.join("snapshot.json").is_file());
        }
        let directory = captured.pending.directory.path().to_path_buf();
        assert_eq!(captured.pending.tip.sequence, 11);
        drop(captured);
        let pending = read_pending(&directory).unwrap();
        assert!(pending.current_matches && !pending.terminal_record_only);
        assert!(pending.tip.phase == Phase::PreparedRestore);
        drop(pending);
        println!("actual Ready and PreStop immutable private snapshots bound to canonical current; exact2MiB file preserved in two1MiB parts, seal denies rewrite; synthetic source only, no official Owner/ReadySession/Closed proof");
        temp.retain();
    }
    #[test]
    fn wl2_r4_working_unknown_edit_archives_actual_bytes_without_adoption() {
        let mut temp = Fixture::new();
        let mut captured = TreeCaptureV2::capture(setup(&temp), end()).unwrap();
        captured.create_working_trees(end()).unwrap();
        let work = captured.pins[0].path().join(".codex/config.toml");
        std::fs::write(&work, b"synthetic unknown external edit").unwrap();
        assert!(captured.record_working_phase(Phase::Ready, end()).is_err());
        assert!(captured.failed);
        let actual = captured
            .custody
            .path()
            .join("ready/.codex/data-0001-0000.bin");
        assert_eq!(
            std::fs::read(actual).unwrap(),
            b"synthetic unknown external edit"
        );
        assert_eq!(
            std::fs::read(captured.custody.path().join(".codex/config.toml")).unwrap(),
            b"synthetic nonsecret config"
        );
        assert_eq!(captured.pending.tip.sequence, 9);
        assert!(captured.pending.tip.phase == Phase::Captured);
        let directory = captured.pending.directory.path().to_path_buf();
        assert!(!directory.join("phase-00000010.json").exists());
        assert!(captured.record_working_phase(Phase::Ready, end()).is_err());
        drop(captured);
        let pending = read_pending(&directory).unwrap();
        assert!(pending.current_matches && !pending.terminal_record_only);
        drop(pending);
        println!("actual unknown bytes copied privately but never adopted as allowed official changes; canonical Captured/current unchanged, original custody and failure preserved, no catch-up or permission");
        temp.retain();
    }
    #[test]
    fn wl2_r4_working_archive_extra_file_refuses_native_phase_readback() {
        let mut temp = Fixture::new();
        let mut captured = TreeCaptureV2::capture(setup(&temp), end()).unwrap();
        captured.create_working_trees(end()).unwrap();
        captured.record_working_phase(Phase::Ready, end()).unwrap();
        let foreign = captured.custody.path().join("ready/.grok/foreign.new");
        std::fs::write(&foreign, b"synthetic archive drift").unwrap();
        assert!(captured.recheck(end()).is_err());
        assert!(captured.working_archives[0].1.reference(end()).is_err());
        assert!(captured
            .record_working_phase(Phase::PreparedRestore, end())
            .is_err());
        let directory = captured.pending.directory.path().to_path_buf();
        assert!(!directory.join("phase-00000011.json").exists());
        assert!(foreign.exists());
        drop(captured);
        let pending = read_pending(&directory).unwrap();
        assert!(pending.current_matches && !pending.terminal_record_only);
        drop(pending);
        println!("native archive extra-file drift poisons retained archive and refuses further PreStop; locator can still identify pending Ready data but cannot issue native authority");
        temp.retain();
    }
    #[test]
    fn wl2_r4_retired_working_drift_keeps_canonical_aborted_reference() {
        let mut temp = Fixture::new();
        let mut captured = TreeCaptureV2::capture(setup(&temp), end()).unwrap();
        let directory = captured.pending.directory.path().to_path_buf();
        captured.create_working_trees(end()).unwrap();
        captured.restore_exact(end()).unwrap();
        let foreign = captured
            .custody
            .path()
            .join("retired-working-.codex/foreign.new");
        std::fs::write(&foreign, b"synthetic retired-tree external drift").unwrap();
        assert!(
            captured.recheck(end()).is_err(),
            "canonical readback must not forget the retired working tree"
        );
        assert_eq!(captured.pending.tip.sequence, 14);
        assert!(captured.pending.tip.phase == Phase::Aborted);
        assert!(foreign.exists());
        drop(captured);
        let pending = read_pending(&directory).unwrap();
        assert!(pending.current_matches && !pending.terminal_record_only);
        assert_eq!(pending.tip.sequence, 14);
        drop(pending);
        println!("retired working drift refuses native canonical recheck; original restored but exact Aborted/current prefix unchanged, conflict retained, no successor or Closed proof");
        temp.retain();
    }
    #[test]
    fn wl2_r4_unperformed_native_root_moves_refuse_ready_before_phase_write() {
        let temp = Fixture::new();
        let mut captured = TreeCaptureV2::capture(setup(&temp), end()).unwrap();
        let directory = captured.pending.directory.path().to_path_buf();
        let current_before = std::fs::read(directory.join("phase-00000001.json")).unwrap();
        let original = captured.pins[0].path().join(".codex/config.toml");
        assert!(
            captured.advance_phase(Phase::Ready, end()).is_err(),
            "native append must not publish Ready without recorded working roots"
        );
        assert!(captured.failed);
        assert!(original.exists());
        assert!(!captured.custody.path().join(".codex").exists());
        assert!(!directory.join("phase-00000002.json").exists());
        assert_eq!(
            std::fs::read(directory.join("phase-00000001.json")).unwrap(),
            current_before
        );
        drop(captured);
        let pending = read_pending(&directory).unwrap();
        assert_eq!(pending.tip.sequence, 1);
        assert!(pending.current_matches && !pending.terminal_record_only);
        println!("native premature Ready refused before extra phase/current mutation, original root unchanged, canonical Captured/current remains pending; no retry or production proof");
    }
    #[test]
    fn wl2_r4_native_manifest_and_exclusive_v2_intent_precede_both_root_moves() {
        let temp = Fixture::new();
        let mut captured = TreeCaptureV2::capture(setup(&temp), end()).unwrap();
        assert_eq!(captured.manifest.nodes.len(), 3);
        let before_digest = captured.pending.tip.manifest_digest.clone();
        let path = captured.pending.directory.path().to_path_buf();
        captured.create_working_trees(end()).unwrap();
        assert!(read_pending(&path).is_err()); // exclusive actual current remains held
        assert!(captured.custody.path().join(".codex/config.toml").exists());
        captured.restore_exact(end()).unwrap();
        assert!(!captured.pins[0].path().join(".grok").exists());
        assert_eq!(captured.pending.tip.manifest_digest, before_digest);
        drop(captured);
        let pending = read_pending(&path).unwrap();
        assert!(pending.current_matches && !pending.terminal_record_only);
        assert_eq!(pending.tip.sequence, 14);
        assert!(pending.tip.phase == Phase::Aborted);
        println!("native v2 actual tree manifest/current and each Intent/Completed before/after move/workcopy/abort-restore; both roots restored IDs/content/8projections; single canonical journal only; pending Aborted not Closed; production source admission unproven");
    }
    #[test]
    fn wl2_r4_native_interrupted_tree_capture_is_pending_and_never_text_closed() {
        let mut temp = Fixture::new();
        let mut captured = TreeCaptureV2::capture(setup(&temp), end()).unwrap();
        let path = captured.pending.directory.path().to_path_buf();
        let custody = captured.custody.path().to_path_buf();
        captured.move_root(0, end()).unwrap();
        assert!(custody.join(".codex/config.toml").exists());
        assert!(!captured.pins[0].path().join(".codex").exists());
        drop(captured); // deliberately interrupted; Drop never restores/deletes
        let pending = read_pending(&path).unwrap();
        assert!(pending.current_matches && !pending.terminal_record_only);
        assert_eq!(pending.tip.sequence, 3);
        drop(pending);
        assert!(custody.join(".codex/config.toml").exists());
        println!("native actual first-root interruption preserves original custody and canonical v2 pending, no reopen restoration authority or Closed");
        temp.retain();
    }
    #[test]
    fn wl2_r4_native_intent_without_native_move_cannot_publish_completed() {
        let mut temp = Fixture::new();
        let mut captured = TreeCaptureV2::capture(setup(&temp), end()).unwrap();
        let path = captured.pending.directory.path().to_path_buf();
        let original = captured.pins[0].path().join(".codex/config.toml");
        captured
            .begin_tree(0, TreeAction::MoveToCustody, end())
            .unwrap();
        assert!(captured
            .complete_tree(0, TreeAction::MoveToCustody, end())
            .is_err());
        assert!(captured.create_working_trees(end()).is_err());
        assert!(original.exists());
        drop(captured);
        let pending = read_pending(&path).unwrap();
        assert_eq!(pending.tip.sequence, 2);
        assert!(pending.current_matches && !pending.terminal_record_only);
        assert_eq!(
            pending.tip.tree_record,
            TreeRecord::Intent {
                root: TreeRoot::Codex,
                action: TreeAction::MoveToCustody
            }
        );
        drop(pending);
        println!("native first move not executed: Completed refused, original unchanged, canonical Intent/current retained, no retry or Closed");
        temp.retain();
    }
    #[test]
    fn wl2_r4_native_second_restore_conflict_keeps_exact_mixed_state_and_intent() {
        let mut temp = Fixture::new();
        let binding = setup(&temp);
        let profile = binding.paths()[0];
        std::fs::create_dir(profile.join(".grok")).unwrap();
        std::fs::write(
            profile.join(".grok/settings.json"),
            b"synthetic original Grok",
        )
        .unwrap();
        let mut captured = TreeCaptureV2::capture(binding, end()).unwrap();
        let path = captured.pending.directory.path().to_path_buf();
        captured.create_working_trees(end()).unwrap();
        captured.pending = captured
            .current
            .append_pending(&captured.pending, Phase::Aborted, end())
            .unwrap();
        captured.restore_root(0, end()).unwrap();
        let foreign = captured.pins[0].path().join(".grok/foreign.new");
        std::fs::write(&foreign, b"external synthetic conflict").unwrap();
        assert!(captured.restore_root(1, end()).is_err());
        assert!(captured.failed);
        assert_eq!(
            std::fs::read(&foreign).unwrap(),
            b"external synthetic conflict"
        );
        assert!(captured.pins[0].path().join(".codex/config.toml").exists());
        assert!(captured.custody.path().join(".grok/settings.json").exists());
        assert!(captured.restore_exact(end()).is_err());
        drop(captured);
        let pending = read_pending(&path).unwrap();
        assert_eq!(pending.tip.sequence, 13);
        assert_eq!(
            pending.tip.tree_record,
            TreeRecord::Intent {
                root: TreeRoot::Grok,
                action: TreeAction::RestoreOriginal
            }
        );
        assert!(pending.current_matches && !pending.terminal_record_only);
        drop(pending);
        println!("native first root restored, second foreign edit retained, original Grok in custody, exact restore Intent pending; no rollback/catch-up/Closed");
        temp.retain();
    }
}
