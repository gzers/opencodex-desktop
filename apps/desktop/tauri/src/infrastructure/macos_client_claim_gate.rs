//! Descriptor-owned exclusion for the exact original claim inode. This is a
//! lock primitive, NOT lifecycle, movement, source or writer authority. Callers
//! must prepare this fd before opening claim SQLite, and normally close that
//! connection before acquisition; failed acquisition never closes this pin.
//! the other native coordinators and official writers remain their concern.
//! No unsupported-kernel fallback, copied database or new pathname is adopted.
use super::*;
use std::{marker::PhantomData, time::Instant};
const CLAIM: &str = ".opencodex-native-main.claim.sqlite";

/// Fields and fd stay private: callers cannot release/duplicate the OFD lock or
/// serialize this object into permission. Expiry refuses work without unlocking
/// a still-running owner. Drop closes the original open file description.
pub struct ClaimGate {
    file: File,
    parent_identity: Identity,
    parent_protection: Protection,
    identity: Identity,
    protection: Protection,
    deadline: Instant,
    held: bool,
    _not_send_or_sync: PhantomData<*mut ()>,
}
fn time(deadline: Instant) -> io::Result<()> {
    if Instant::now() >= deadline {
        return Err(denied("claim descriptor gate deadline expired"));
    }
    Ok(())
}
fn set(file: &File, kind: libc::c_short) -> io::Result<()> {
    let request = libc::flock {
        l_start: 0,
        l_len: 0,
        l_pid: 0,
        l_type: kind,
        l_whence: libc::SEEK_SET as libc::c_short,
    };
    if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_OFD_SETLK, &request) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
impl ClaimGate {
    /// Static original identity only, not current lock or movement authority.
    pub(crate) fn key(&self) -> (String, String) {
        (
            self.identity.device.to_string(),
            self.identity.inode.to_string(),
        )
    }
    /// Existing private file only; no CREATE, truncate, chmod, repair or retry.
    /// An in-process SQLite connection still holding this inode is also a
    /// conflicting owner, so a handoff must close it normally first.
    /// Prepare BEFORE any in-process SQLite locks on this inode, and keep this
    /// owner alive until all such connections close. Even an unheld extra fd
    /// would release process-associated SQLite locks when closed.
    pub fn prepare(parent: &PinnedDirectory, deadline: Instant) -> io::Result<Self> {
        time(deadline)?;
        parent.private()?;
        let filename = CString::new(CLAIM).unwrap();
        let fd = unsafe {
            libc::openat(
                parent.file().as_raw_fd(),
                filename.as_ptr(),
                libc::O_RDWR | libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK,
            )
        };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let file = unsafe { File::from_raw_fd(fd) };
        let s = stat(&file)?;
        let p = protection(&file, &s)?;
        if s.st_mode & libc::S_IFMT != libc::S_IFREG
            || s.st_nlink != 1
            || p.uid != unsafe { libc::geteuid() }
            || p.mode != 0o600
            || p.flags != 0
            || p.acl.is_some()
            || p.xattrs
                .iter()
                .any(|(k, v)| k != b"com.apple.provenance" || v.len() > 64)
        {
            return Err(denied("claim descriptor gate file protection unknown"));
        }
        let root = stat(parent.file())?;
        let result = Self {
            file,
            parent_identity: identity(&root),
            parent_protection: protection(parent.file(), &root)?,
            identity: identity(&s),
            protection: p,
            deadline,
            held: false,
            _not_send_or_sync: PhantomData,
        };
        result.check_identity_at(parent)?;
        Ok(result)
    }
    /// No new fd is opened or closed here, including contention/error paths.
    pub fn acquire(&mut self, parent: &PinnedDirectory) -> io::Result<()> {
        time(self.deadline)?;
        if self.held {
            return Err(denied("claim descriptor gate already held"));
        }
        self.check_identity_at(parent)?;
        set(&self.file, libc::F_WRLCK)?;
        self.held = true;
        self.check_at(parent)
    }
    fn check_identity_at(&self, parent: &PinnedDirectory) -> io::Result<()> {
        parent.private()?;
        let root = stat(parent.file())?;
        if identity(&root) != self.parent_identity
            || protection(parent.file(), &root)? != self.parent_protection
        {
            return Err(denied("claim descriptor gate root generation changed"));
        }
        let filename = CString::new(CLAIM).unwrap();
        let mut current = std::mem::MaybeUninit::uninit();
        if unsafe {
            libc::fstatat(
                parent.file().as_raw_fd(),
                filename.as_ptr(),
                current.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } != 0
        {
            return Err(io::Error::last_os_error());
        }
        let current = unsafe { current.assume_init() };
        let retained = stat(&self.file)?;
        if identity(&current) != self.identity
            || identity(&retained) != self.identity
            || current.st_mode & libc::S_IFMT != libc::S_IFREG
            || current.st_nlink != 1
            || retained.st_nlink != 1
            || protection(&self.file, &retained)? != self.protection
            || current.st_uid != retained.st_uid
            || current.st_gid != retained.st_gid
            || current.st_mode != retained.st_mode
            || current.st_flags != retained.st_flags
        {
            return Err(denied("claim descriptor gate inode or protection changed"));
        }
        Ok(())
    }
    /// The owning movement protocol must authorize the supplied parent. This
    /// merely checks the same original directory/inode at that physical anchor;
    /// it grants no permission to move it or adopt a replacement logical root.
    pub fn check_at(&self, parent: &PinnedDirectory) -> io::Result<()> {
        time(self.deadline)?;
        if !self.held {
            return Err(denied("claim descriptor gate is released"));
        }
        self.check_identity_at(parent)?;
        time(self.deadline)
    }
    pub fn finish_at(&mut self, parent: &PinnedDirectory) -> io::Result<()> {
        let valid = self.check_at(parent);
        let release = set(&self.file, libc::F_UNLCK);
        if release.is_ok() {
            self.held = false;
        }
        valid?;
        release?;
        time(self.deadline)
    }
}
impl Drop for ClaimGate {
    fn drop(&mut self) {
        if self.held {
            let _ = set(&self.file, libc::F_UNLCK);
            self.held = false;
        }
        // File::drop is the final release even if explicit unlock failed.
    }
}

#[cfg(test)]
#[path = "macos_client_claim_gate_tests.rs"]
mod tests;
