//! Original preparation sink. Win32 CreateDirectoryW returns no handle: a
//! successful create with an unopened terminal remains incomplete/unknown.
//! No retry, path-based recovery proof, cleanup or existing-name adoption.
use super::*;
use std::time::Instant;
#[derive(Default)]
pub(crate) struct PrivateDirectoryPreparation {
    path: Option<PathBuf>,
    parent: Option<PinnedDirectory>,
    sid: Option<String>,
    created: bool,
    pending: Option<File>,
    complete: Option<PinnedDirectory>,
    attempted: bool,
    ready: bool,
    failed: bool,
    #[cfg(test)]
    fault: u8,
}
impl PrivateDirectoryPreparation {
    /// Bounded DATA only, no new serialization dependency in the native backend.
    pub(crate) fn retained_stage_data(&self) -> (Option<PathBuf>, [bool; 7]) {
        (
            self.path.clone(),
            [
                self.attempted,
                self.parent.is_some(),
                self.created,
                self.pending.is_some(),
                self.complete.is_some(),
                self.ready,
                self.failed,
            ],
        )
    }
    fn checkpoint(&self, stage: u8, end: Instant) -> io::Result<()> {
        if Instant::now() >= end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        #[cfg(test)]
        if stage != 0 && self.fault == stage {
            return Err(io::Error::other(format!(
                "injected original directory preparation stage{stage}"
            )));
        }
        #[cfg(not(test))]
        let _ = stage;
        Ok(())
    }
    pub(crate) fn create(&mut self, path: &Path, end: Instant) -> io::Result<()> {
        if self.attempted {
            return Err(denied("original directory preparation already attempted"));
        }
        self.attempted = true;
        self.failed = true;
        self.path = Some(path.into());
        let result = self.create_inner(end);
        if result.is_ok() {
            self.ready = true;
            self.failed = false;
        }
        result
    }
    fn create_inner(&mut self, end: Instant) -> io::Result<()> {
        self.checkpoint(1, end)?;
        let path = self.path.as_ref().unwrap();
        validate(path)?;
        let parent_path = path
            .parent()
            .ok_or_else(|| denied("non-root directory required"))?;
        self.parent = Some(PinnedDirectory::open(parent_path)?);
        let token = current_identity()?;
        if token.elevated || token.elevation_type == 2 {
            return Err(denied("ordinary original directory creator required"));
        }
        self.sid = Some(token.sid);
        let sd = PrivateDescriptor::new(self.sid.as_ref().unwrap(), true)?;
        let sa = sd.attributes();
        let w = wide(path)?;
        self.parent.as_ref().unwrap().recheck()?;
        self.checkpoint(1, end)?;
        check(unsafe { CreateDirectoryW(w.as_ptr(), &sa) })?;
        // The first infallible assignment follows the actual native mutation.
        // No terminal ID is claimed until its own retained handle is observed.
        self.created = true;
        self.checkpoint(2, end)?;
        self.pending = Some(
            OpenOptions::new()
                .access_mode(0x80020000)
                .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .open(self.path.as_ref().unwrap())?,
        );
        self.checkpoint(3, end)?;
        let file = self.pending.as_ref().unwrap();
        let id = identity(file, true)?;
        verify_private(file.as_raw_handle(), self.sid.as_ref().unwrap(), true)?;
        self.parent.as_ref().unwrap().recheck()?;
        self.checkpoint(4, end)?;
        // Transfer the same actual handles without reopening the new directory
        // or introducing a fallible operation while either set is local.
        let mut files = self.parent.take().unwrap().into_owned_files();
        files.push(self.pending.take().unwrap());
        self.complete = Some(PinnedDirectory {
            path: self.path.as_ref().unwrap().clone(),
            files,
            id,
            sid: self.sid.take(),
        });
        self.checkpoint(5, end)?;
        self.complete.as_ref().unwrap().recheck()?;
        self.checkpoint(0, end)
    }
    pub(crate) fn take_complete(&mut self) -> io::Result<PinnedDirectory> {
        if !self.ready || self.failed {
            return Err(denied("original directory preparation incomplete"));
        }
        self.complete
            .take()
            .ok_or_else(|| denied("original directory already transferred"))
    }
}
#[cfg(test)]
mod tests;
