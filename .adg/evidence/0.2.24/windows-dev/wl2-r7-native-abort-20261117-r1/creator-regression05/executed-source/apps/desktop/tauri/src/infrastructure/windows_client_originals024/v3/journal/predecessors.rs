use super::super::predecessors::Predecessor;
use super::*;
impl Journal {
    pub(in crate::infrastructure::windows_client_originals024::v3) fn check_predecessor_references(
        &mut self,
        refs: &[Predecessor],
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.check_records(end)?;
            super::super::predecessors::validate_set(refs, &self.manifest.binding)?;
            if encoded(&refs)? != encoded(&self.manifest.predecessors)? {
                return Err(invalid());
            }
            self.check_capture_namespace(end)?;
            self.check_records(end)
        })();
        if result.is_err() {
            self.poison_segment();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn check_capture_namespace(
        &mut self,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            let windows =
                PinnedDirectory::open_private(&self.pins[1].path().join("official024/windows"))?;
            super::super::predecessors::check_namespace(
                &windows,
                &self.manifest.predecessors,
                &self.manifest.binding,
                Some(&self.directory),
                end,
            )
        })();
        if result.is_err() {
            self.poison_segment();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024::v3) fn completed_predecessor(
        &mut self,
        read: &super::super::read_window::HistoryReadWindow,
    ) -> io::Result<Predecessor> {
        let result = (|| {
            self.completed_stop_history_read(read)?;
            let completion = self.stop_completion.as_ref().ok_or_else(invalid)?;
            let native_completion_digest = completion.original_state_digest(self)?;
            let closed = self
                .progress
                .closed
                .as_ref()
                .filter(|c| c.verified)
                .ok_or_else(invalid)?;
            let route = self
                .progress
                .route_close
                .as_ref()
                .filter(|c| c.verified)
                .ok_or_else(invalid)?;
            let content = self
                .progress
                .content
                .as_ref()
                .filter(|c| c.verified)
                .ok_or_else(invalid)?;
            let out = Predecessor {
                schema: "windows-anchored-closed-predecessor-v3".into(),
                session_id: self.progress.session.clone(),
                binding_digest: self.progress.binding.clone(),
                directory: self.directory.path().into(),
                volume: self.directory.identity().volume,
                directory_id: self.directory.identity().index.to_string(),
                directory_security_digest: hash::sha256_hex(&encoded(&self.directory_security)?),
                current_id: self.current_id.index.to_string(),
                current_security_digest: hash::sha256_hex(&encoded(&self.current_security)?),
                current_sha256: hash::sha256_hex(&self.current_raw),
                manifest_digest: self.progress.manifest.clone(),
                sequence: self.progress.sequence,
                event_digest: self.progress.tip.clone(),
                content_digest: content.digest.clone(),
                route_close_digest: route.digest.clone(),
                closed_digest: closed.digest.clone(),
                native_completion_digest,
                prior_set_digest: super::super::predecessors::set_digest(
                    &self.manifest.predecessors,
                )?,
            };
            self.completed_stop_history_read(read)?;
            Ok(out)
        })();
        if result.is_err() {
            self.poison_segment();
        }
        result
    }
}
