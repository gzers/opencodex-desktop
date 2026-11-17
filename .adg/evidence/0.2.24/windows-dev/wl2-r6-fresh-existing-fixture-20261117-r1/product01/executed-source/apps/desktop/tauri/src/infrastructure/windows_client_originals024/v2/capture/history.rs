//! Original native histories only. No directory scan is an adoption protocol.
use super::*;
use crate::infrastructure::windows_client_originals024::v2::history::MAX_HISTORY;
impl TreeCaptureV2 {
    pub(in crate::infrastructure::windows_client_originals024::v2) fn create_working_trees_with_history(
        &mut self,
        histories: &mut [&mut Self],
        end: Instant,
    ) -> io::Result<()> {
        for index in 0..self.trees.len() {
            self.recheck_with_history(histories, end)?;
            self.move_root(index, end)?;
            self.recheck_with_history(histories, end)?;
            self.begin_tree(index, TreeAction::CreateWorkingCopy, end)?;
            self.trees[index].create_working_copy(end)?;
            self.complete_tree(index, TreeAction::CreateWorkingCopy, end)?;
            self.recheck_with_history(histories, end)?;
        }
        Ok(())
    }
    pub(in crate::infrastructure::windows_client_originals024::v2) fn restore_after_teardown_with_history(
        &mut self,
        histories: &mut [&mut Self],
        end: Instant,
    ) -> io::Result<()> {
        self.recheck_with_history(histories, end)?;
        if self.pending.tip.phase != Phase::OfficialTeardown {
            return Err(invalid());
        }
        for index in 0..self.trees.len() {
            self.recheck_with_history(histories, end)?;
            self.restore_root(index, end)?;
            self.recheck_with_history(histories, end)?;
        }
        self.advance_phase(Phase::Restored, end)?;
        self.recheck_with_history(histories, end)
    }
    fn closed_reference(&mut self, end: Instant) -> io::Result<ClosedReference> {
        self.recheck(end)?;
        if self.pending.tip.phase != Phase::Closed
            || !self.original_handles_released
            || self.original_history.is_none()
        {
            return Err(invalid());
        }
        let (id, current_digest) = self.current.native_identity_digest(&self.pending, end)?;
        let directory = self.pending.directory.identity();
        Ok(ClosedReference {
            session_id: self.manifest.session_id.clone(),
            directory: Identity {
                volume: directory.volume,
                file_id: directory.index.to_string(),
            },
            current: Identity {
                volume: id.volume,
                file_id: id.index.to_string(),
            },
            closed_sequence: self.pending.tip.sequence,
            phase_digest: self.pending.tip.phase_digest.clone(),
            manifest_digest: self.pending.tip.manifest_digest.clone(),
            current_digest,
            binding_digest: self.pending.tip.binding_digest.clone(),
            security_policy_digest: self.pending.tip.policy_digest.clone(),
        })
    }
    /// Callers supply actual retained native captures and their exact known
    /// order. Enumerating textClosed directories cannot manufacture this slice.
    pub(in crate::infrastructure::windows_client_originals024::v2) fn native_history_set(
        binding: &Binding,
        histories: &mut [&mut Self],
        windows: &PinnedDirectory,
        active: Option<&PinnedDirectory>,
        end: Instant,
    ) -> io::Result<Vec<ClosedReference>> {
        before(end)?;
        windows.recheck()?;
        if histories.len() > MAX_HISTORY
            || windows.path() != binding.paths()[1].join("official024/windows")
        {
            return Err(invalid());
        }
        let mut refs = Vec::new();
        let mut expected = Vec::new();
        for old in histories.iter_mut() {
            if old.manifest.binding != *binding
                || old.manifest.predecessors != refs
                || old.pending.directory.path().parent() != Some(windows.path())
            {
                return Err(invalid());
            }
            let reference = old.closed_reference(end)?;
            expected.push(reference.session_id.clone());
            refs.push(reference);
        }
        let mut active_name = String::new();
        if let Some(active) = active {
            active.recheck()?;
            if active.path().parent() != Some(windows.path()) {
                return Err(invalid());
            }
            active_name = active
                .path()
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(invalid)?
                .into();
            if !session(&active_name) || expected.contains(&active_name) {
                return Err(invalid());
            }
            expected.push(active_name.clone());
        }
        check_predecessors(&refs, &active_name, binding, &Policy::fixed().digest()?)?;
        expected.sort();
        let mut names = std::fs::read_dir(windows.path())?
            .take(MAX_HISTORY + 2)
            .map(|row| row.and_then(|row| row.file_name().into_string().map_err(|_| invalid())))
            .collect::<io::Result<Vec<_>>>()?;
        names.sort();
        if names != expected {
            return Err(invalid());
        }
        windows.recheck()?;
        before(end)?;
        Ok(refs)
    }
    /// Revalidate an owned current capture together with ALL of its retained
    /// known predecessors. A failure poisons the current capture; no catch-up.
    pub(in crate::infrastructure::windows_client_originals024::v2) fn recheck_with_history(
        &mut self,
        histories: &mut [&mut Self],
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.recheck(end)?;
            let windows = PinnedDirectory::open_private(
                self.pending.directory.path().parent().ok_or_else(invalid)?,
            )?;
            let refs = Self::native_history_set(
                &self.manifest.binding,
                histories,
                &windows,
                Some(&self.pending.directory),
                end,
            )?;
            if refs != self.manifest.predecessors {
                return Err(invalid());
            }
            self.recheck(end)
        })();
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024::v2) fn binding_data(
        &self,
    ) -> &Binding {
        &self.manifest.binding
    }
}
