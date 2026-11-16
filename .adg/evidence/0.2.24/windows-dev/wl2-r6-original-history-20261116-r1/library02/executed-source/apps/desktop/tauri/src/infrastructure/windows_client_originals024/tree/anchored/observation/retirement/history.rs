//! Original objects and immutable original-byte copies remain distinct.
//! Only actual retained original handles can create this native view.
use super::*;
use crate::infrastructure::windows_client_originals024::v3::journal::HistoryContext;
impl AnchoredTree {
    pub(in crate::infrastructure::windows_client_originals024::tree::anchored) fn original_history_frame(
        &mut self,
        end: Instant,
    ) -> io::Result<serde_json::Value> {
        self.readback_retirement_after_release(end)?;
        if self.original_history.is_none() {
            let mut frame = WorkFrame::empty(self, end)?;
            for n in &self.nodes {
                // Duplicate the exact already-held object, never reopen/adopt
                // a path or reconstruct a native handle from its data ID.
                frame.nodes.push(FrameNode {
                    file: n.file.try_clone()?,
                    entry: Entry {
                        relative: n.original.relative.clone(),
                        directory: n.original.directory,
                        id: n.original.id,
                        security: serde_json::from_value(
                            serde_json::to_value(&n.original.security).map_err(|_| invalid())?,
                        )
                        .map_err(|_| invalid())?,
                        bytes: n.original.bytes.clone(),
                        children: n.original.children.clone(),
                    },
                });
            }
            // Install before fallible all8/path/byte readback. Copies of DATA
            // above grant no authority: the actual native handle is retained.
            self.original_history = Some(frame);
        }
        let frame = self.original_history.as_mut().ok_or_else(invalid)?;
        frame.check(end)?;
        Ok(frame.metadata())
    }
    pub(in crate::infrastructure::windows_client_originals024::tree::anchored) fn write_original_history(
        &mut self,
        context: &HistoryContext,
        fault: u8,
        end: Instant,
    ) -> io::Result<serde_json::Value> {
        self.original_history_frame(end)?;
        let result = self
            .original_history
            .as_mut()
            .ok_or_else(invalid)?
            .archive_original_history(self.parent.path(), context, fault, end);
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
}
impl RecoveryWitness<'_> {
    pub(in crate::infrastructure::windows_client_originals024) fn original_frames(
        &mut self,
        end: Instant,
    ) -> io::Result<[serde_json::Value; 2]> {
        self.post_release(end)?;
        Ok([
            self.trees[0].original_history_frame(end)?,
            self.trees[1].original_history_frame(end)?,
        ])
    }
    pub(in crate::infrastructure::windows_client_originals024) fn archive_originals(
        &mut self,
        context: &HistoryContext,
        fault: u8,
        end: Instant,
    ) -> io::Result<[serde_json::Value; 2]> {
        let result = (|| {
            self.post_release(end)?;
            let first = self.trees[0].write_original_history(context, fault, end)?;
            if fault == 72 {
                return Err(io::Error::other(
                    "injected after first complete original-history archive before second root",
                ));
            }
            let second = self.trees[1].write_original_history(context, 0, end)?;
            self.post_release(end)?;
            Ok([first, second])
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn history_references(
        &mut self,
        end: Instant,
    ) -> io::Result<[serde_json::Value; 2]> {
        let result = (|| {
            self.post_release(end)?;
            let mut refs = Vec::new();
            for t in &mut *self.trees {
                let frame = t.original_history.as_mut().ok_or_else(invalid)?;
                frame.check(end)?;
                refs.push(frame.archive.as_mut().ok_or_else(invalid)?.reference(end)?);
            }
            self.post_release(end)?;
            refs.try_into().map_err(|_| invalid())
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
}
