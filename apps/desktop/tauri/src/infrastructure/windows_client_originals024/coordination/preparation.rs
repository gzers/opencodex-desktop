//! Same-owning-thread, one-shot acquisition prefix. No active NativeLease is
//! sent across threads or reconstructed from a serialized receipt.
use super::*;
use crate::modules::runtime::windows_lifecycle024_contract::BeforeResumeRecoveryWindow;
#[derive(Default)]
pub(in crate::infrastructure::windows_client_originals024) struct LeasePreparation {
    lease: Option<NativeLease>,
    pending: PendingFile,
    attempted: bool,
    ready: bool,
    failed: bool,
    #[cfg(test)]
    fault: u8,
}
#[derive(Default)]
struct PendingFile {
    parent: Option<ParentPin>,
    path: Option<PathBuf>,
    file: Option<File>,
    id: Option<FileIdentity>,
    security: Option<SnapshotData>,
    raw: Option<Vec<u8>>,
    complete: Option<LockFile>,
}
impl LeasePreparation {
    pub(super) fn capture_probe_file(
        &mut self,
        path: &Path,
        claim: bool,
        end: Instant,
    ) -> io::Result<LockFile> {
        self.capture(path, claim, end)?;
        let file = self.pending.complete.take().ok_or_else(refused)?;
        self.pending = PendingFile::default();
        Ok(file)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn revoke_retained(&mut self) {
        self.poison();
    }
    pub(in crate::infrastructure::windows_client_originals024) fn empty_after_transfer(
        &self,
    ) -> bool {
        self.lease.is_none()
            && self.pending.file.is_none()
            && self.pending.complete.is_none()
            && self.pending.parent.is_none()
            && self.pending.path.is_none()
    }
    #[cfg(test)]
    pub(in crate::infrastructure::windows_client_originals024) fn prepare_fixture_paths(
        &mut self,
        paths: &[PathBuf; 4],
        end: Instant,
    ) -> io::Result<()> {
        self.acquire(paths, None, end)
    }
    fn checkpoint(&self, stage: u8, end: Instant) -> io::Result<()> {
        before(end)?;
        #[cfg(test)]
        if stage != 0 && self.fault == stage {
            return Err(io::Error::other(format!(
                "injected retained lease stage{stage}"
            )));
        }
        #[cfg(not(test))]
        let _ = stage;
        Ok(())
    }
    fn capture(&mut self, path: &Path, claim: bool, end: Instant) -> io::Result<()> {
        if self.pending.path.is_some() {
            return Err(refused());
        }
        self.pending.path = Some(path.into());
        before(end)?;
        no_sidecars(path)?;
        let directory = path.parent().ok_or_else(refused)?;
        self.pending.parent = Some(if claim {
            ParentPin::Claim(ClaimParent::capture(directory, end)?)
        } else {
            ParentPin::Strict(PinnedDirectory::open_private(directory)?)
        });
        self.pending.file = Some(open(path)?);
        self.checkpoint(2, end)?;
        let file = self.pending.file.as_mut().unwrap();
        native::verify_private_file(file)?;
        tree::streams(path, false)?;
        self.pending.id = Some(file_id(file)?);
        self.pending.security = Some(native_stamp(file, end).map_err(io_error)?);
        self.pending.raw = Some(snapshot(file, end)?);
        self.checkpoint(3, end)?;
        self.pending.complete = Some(LockFile {
            file: self.pending.file.take().unwrap(),
            parent: self.pending.parent.take().unwrap(),
            path: self.pending.path.as_ref().unwrap().clone(),
            id: self.pending.id.take().unwrap(),
            security: self.pending.security.take().unwrap(),
            raw: self.pending.raw.take().unwrap(),
        });
        self.pending.complete.as_mut().unwrap().check(end)?;
        self.checkpoint(4, end)
    }
    fn acquire(
        &mut self,
        paths: &[PathBuf; 4],
        bound: Option<Bound>,
        end: Instant,
    ) -> io::Result<()> {
        if self.attempted {
            return Err(refused());
        }
        self.attempted = true;
        self.failed = true;
        let result = (|| {
            before(end)?;
            let token = native::current_identity()?;
            if token.elevated
                || token.elevation_type == 2
                || paths
                    .iter()
                    .enumerate()
                    .any(|(i, p)| paths[..i].contains(p))
            {
                return Err(refused());
            }
            self.lease = Some(NativeLease {
                held: Vec::new(),
                token,
                deadline: end,
                active: true,
                start_state: StartLeaseState::Other,
                bound,
                acquisition_thread: std::thread::current().id(),
                _not_send: PhantomData,
                epoch: uuid::Uuid::new_v4().to_string(),
            });
            self.checkpoint(1, end)?;
            for (i, path) in paths.iter().enumerate() {
                self.capture(path, i == 0, end)?;
                // Install the actual pin in its original Held before the first
                // byte range. No failed partial acquisition is silently dropped.
                self.lease.as_mut().unwrap().held.push(Held {
                    pin: self.pending.complete.take().unwrap(),
                    ranges: Vec::new(),
                });
                self.pending = PendingFile::default();
                self.checkpoint(5, end)?;
                let held = self.lease.as_mut().unwrap().held.last_mut().unwrap();
                if i == 0 {
                    held.range(BASE, 512, true)?;
                } else {
                    held.range(BASE, 1, false)?;
                    self.checkpoint(20, end)?;
                    self.lease
                        .as_mut()
                        .unwrap()
                        .held
                        .last_mut()
                        .unwrap()
                        .range(BASE + 2, 510, false)?;
                    self.checkpoint(21, end)?;
                    self.lease
                        .as_mut()
                        .unwrap()
                        .held
                        .last_mut()
                        .unwrap()
                        .unlock(BASE)?;
                    self.checkpoint(22, end)?;
                    self.lease
                        .as_mut()
                        .unwrap()
                        .held
                        .last_mut()
                        .unwrap()
                        .range(BASE + 1, 1, true)?;
                }
                self.checkpoint(6, end)?;
                for original in &mut self.lease.as_mut().unwrap().held {
                    original.pin.check(end)?;
                }
                self.checkpoint(7, end)?;
            }
            self.lease.as_mut().unwrap().check(end)?;
            self.checkpoint(8, end)?;
            self.ready = true;
            self.failed = false;
            Ok(())
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
    fn poison(&mut self) {
        self.ready = false;
        self.failed = true;
        if let Some(lease) = &mut self.lease {
            lease.active = false;
        }
    }
    pub(in crate::infrastructure::windows_client_originals024) fn prepare_start(
        &mut self,
        owner: &mut Owner,
        end: Instant,
    ) -> io::Result<()> {
        use crate::infrastructure::windows_official_lifecycle024::OriginalActionKind;

        if self.attempted {
            return Err(refused());
        }
        let context = (|| {
            let action = owner
                .original_action(OriginalActionKind::Start, end)
                .map_err(io_error)?;
            let (paths, bound) = existing_context(owner, end)?;
            Ok::<_, io::Error>((action, paths, bound))
        })();
        let (action, paths, bound) = match context {
            Ok(context) => context,
            Err(error) => {
                self.attempted = true;
                self.poison();
                return Err(error);
            }
        };
        self.acquire(&paths, Some(bound), end)?;
        let result = (|| {
            self.lease.as_mut().unwrap().check_owner(owner, end)?;
            action.check(owner, end).map_err(io_error)?;
            before(end)?;
            self.lease.as_mut().unwrap().start_state = StartLeaseState::Start(action.owner_and_action());
            Ok(())
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn prepare_existing(
        &mut self,
        owner: &mut Owner,
        window: &BeforeResumeRecoveryWindow,
    ) -> io::Result<()> {
        if self.attempted {
            return Err(refused());
        }
        let end = window.deadline();
        // Failure during the pre-lock observation is also a permanent attempted
        // prefix. The exact original process preparation is independently held.
        let context = (|| {
            owner
                .check_before_resume_process(window)
                .map_err(io_error)?;
            existing_context(owner, end)
        })();
        let (paths, bound) = match context {
            Ok(v) => v,
            Err(e) => {
                self.attempted = true;
                self.poison();
                return Err(e);
            }
        };
        self.acquire(&paths, Some(bound), end)?;
        let result = (|| {
            self.lease.as_mut().unwrap().check_owner(owner, end)?;
            owner
                .check_before_resume_process(window)
                .map_err(io_error)?;
            before(end)
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn take_complete(
        &mut self,
    ) -> io::Result<NativeLease> {
        if !self.ready || self.failed {
            return Err(refused());
        }
        self.lease.take().ok_or_else(refused)
    }
}
#[cfg(test)]
mod tests;
