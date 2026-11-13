//! Native, read-only client snapshots and create-only private evidence frames.
//! Does not restore client files or treat a journal as lifecycle authority.
//! Source files keep their owner/mode/ACL/xattrs; unreadable is never absent.
use crate::modules::model_config::schema_v2::parse_strict_json_v2;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    ffi::{CString, OsStr},
    fs::File,
    io::{self, Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::ffi::OsStrExt,
    },
    path::{Component, Path, PathBuf},
};

#[path = "macos_client_claim_gate.rs"]
pub mod claim_gate;
#[path = "macos_client_completion.rs"]
pub mod completion;
#[path = "macos_client_control_release.rs"]
mod control_release;
#[path = "macos_client_restore.rs"]
pub mod restore;
#[path = "macos_client_root_creation.rs"]
pub mod root_creation;
#[path = "macos_client_root_retirement.rs"]
pub mod root_retirement;
#[path = "macos_client_session.rs"]
pub mod session;
#[path = "macos_client_session_index.rs"]
pub mod session_index;

#[cfg(test)]
#[path = "macos_client_observation_metrics.rs"]
pub(crate) mod observation_metrics;
const MAX_CONTENT: usize = 2 * 1024 * 1024;
const MAX_SECURITY: usize = 256 * 1024;
const MAX_FRAME: usize = 32 * 1024 * 1024;
const NAMES: [&str; 3] = [
    "opencodex-catalog.json",
    "models_cache.json",
    "opencodex-catalog-audit.jsonl",
];

unsafe extern "C" {
    fn acl_get_fd_np(fd: libc::c_int, kind: libc::c_int) -> *mut libc::c_void;
    fn acl_free(value: *mut libc::c_void) -> libc::c_int;
    fn acl_size(value: *mut libc::c_void) -> libc::ssize_t;
    fn acl_copy_ext(
        buf: *mut libc::c_void,
        value: *mut libc::c_void,
        size: libc::ssize_t,
    ) -> libc::ssize_t;
}
fn denied(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, message)
}
fn name(value: &OsStr) -> io::Result<CString> {
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes == b"." || bytes == b".." || bytes.contains(&b'/') {
        return Err(denied("invalid relative component"));
    }
    CString::new(bytes).map_err(|_| denied("nul component"))
}
fn open_at(parent: &File, child: &OsStr, directory: bool) -> io::Result<File> {
    #[cfg(test)]
    let _metric = observation_metrics::span("open_at");
    let child = name(child)?;
    let flags = libc::O_RDONLY
        | libc::O_CLOEXEC
        | libc::O_NOFOLLOW
        | libc::O_NONBLOCK
        | if directory { libc::O_DIRECTORY } else { 0 };
    let fd = unsafe { libc::openat(parent.as_raw_fd(), child.as_ptr(), flags) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    let file = unsafe { File::from_raw_fd(fd) };
    let info = stat(&file)?;
    if ((info.st_mode & libc::S_IFMT) == libc::S_IFDIR) != directory {
        return Err(denied("object kind changed"));
    }
    if !directory && ((info.st_mode & libc::S_IFMT) != libc::S_IFREG || info.st_nlink != 1) {
        return Err(denied("nonregular or linked source"));
    }
    Ok(file)
}
fn stat(file: &File) -> io::Result<libc::stat> {
    let mut value = std::mem::MaybeUninit::uninit();
    if unsafe { libc::fstat(file.as_raw_fd(), value.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { value.assume_init() })
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Identity {
    device: i32,
    inode: u64,
    birth_seconds: i64,
    birth_nanos: i64,
}
fn identity(s: &libc::stat) -> Identity {
    Identity {
        device: s.st_dev,
        inode: s.st_ino,
        birth_seconds: s.st_birthtime,
        birth_nanos: s.st_birthtime_nsec,
    }
}
impl std::fmt::Debug for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Identity(..)")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Protection {
    uid: u32,
    gid: u32,
    mode: u16,
    flags: u32,
    acl: Option<Vec<u8>>,
    xattrs: Vec<(Vec<u8>, Vec<u8>)>,
}
impl std::fmt::Debug for Protection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Protection(..)")
    }
}
fn acl(file: &File) -> io::Result<Option<Vec<u8>>> {
    let pointer = unsafe { acl_get_fd_np(file.as_raw_fd(), 0x100) };
    if pointer.is_null() {
        let error = io::Error::last_os_error();
        return if error.raw_os_error() == Some(libc::ENOENT) {
            Ok(None)
        } else {
            Err(error)
        };
    }
    struct Free(*mut libc::c_void);
    impl Drop for Free {
        fn drop(&mut self) {
            unsafe {
                acl_free(self.0);
            }
        }
    }
    let _free = Free(pointer);
    let size = unsafe { acl_size(pointer) };
    if size <= 0 || size as usize > MAX_SECURITY {
        return Err(denied("ACL size unavailable"));
    }
    let mut bytes = vec![0; size as usize];
    let count = unsafe { acl_copy_ext(bytes.as_mut_ptr().cast(), pointer, size) };
    if count != size {
        return Err(denied("ACL copy incomplete"));
    }
    Ok(Some(bytes))
}
fn xattrs(file: &File) -> io::Result<BTreeMap<Vec<u8>, Vec<u8>>> {
    let fd = file.as_raw_fd();
    let size = unsafe { libc::flistxattr(fd, std::ptr::null_mut(), 0, 0) };
    if size < 0 {
        return Err(io::Error::last_os_error());
    }
    if size as usize > MAX_SECURITY {
        return Err(denied("xattr names over limit"));
    }
    let mut names = vec![0; size as usize];
    if size > 0
        && unsafe { libc::flistxattr(fd, names.as_mut_ptr().cast(), names.len(), 0) } != size
    {
        return Err(denied("xattr names changed"));
    }
    let mut result = BTreeMap::new();
    let mut total = names.len();
    if !names.is_empty() && names.last() != Some(&0) {
        return Err(denied("xattr names truncated"));
    }
    for bytes in names.split(|b| *b == 0).filter(|b| !b.is_empty()) {
        let key = CString::new(bytes).map_err(|_| denied("xattr name invalid"))?;
        let size = unsafe { libc::fgetxattr(fd, key.as_ptr(), std::ptr::null_mut(), 0, 0, 0) };
        if size < 0 {
            return Err(io::Error::last_os_error());
        }
        total = total
            .checked_add(size as usize)
            .ok_or_else(|| denied("xattr over limit"))?;
        if total > MAX_SECURITY {
            return Err(denied("xattr over limit"));
        }
        let mut value = vec![0; size as usize];
        if size > 0
            && unsafe {
                libc::fgetxattr(
                    fd,
                    key.as_ptr(),
                    value.as_mut_ptr().cast(),
                    value.len(),
                    0,
                    0,
                )
            } != size
        {
            return Err(denied("xattr changed"));
        }
        if result.insert(bytes.to_vec(), value).is_some() {
            return Err(denied("duplicate xattr"));
        }
    }
    Ok(result)
}
fn protection(file: &File, s: &libc::stat) -> io::Result<Protection> {
    #[cfg(test)]
    let _metric = observation_metrics::span("protection");
    Ok(Protection {
        uid: s.st_uid,
        gid: s.st_gid,
        mode: s.st_mode & 0o7777,
        flags: s.st_flags,
        acl: acl(file)?,
        xattrs: xattrs(file)?.into_iter().collect(),
    })
}
fn version(s: &libc::stat) -> (Identity, i64, i64, i64, i64, i64, u16) {
    (
        identity(s),
        s.st_size,
        s.st_mtime,
        s.st_mtime_nsec,
        s.st_ctime,
        s.st_ctime_nsec,
        s.st_nlink,
    )
}
fn require_private(p: &Protection, directory: bool) -> io::Result<()> {
    if p.uid != unsafe { libc::geteuid() }
        || p.mode != if directory { 0o700 } else { 0o400 }
        || p.acl.is_some()
        || p.flags != 0
        // macOS stamps this OS provenance blob on ordinary new private files
        // and directories. It is not an ACL or a grant of data access. Retain
        // its exact value in snapshots; every other attribute stays refused.
        || p.xattrs.iter().any(|(name,value)|name!=b"com.apple.provenance" || value.len()>64)
    {
        return Err(denied("private journal protection differs; not repaired"));
    }
    Ok(())
}

/// Open every component from / with O_NOFOLLOW, retain the native root, and
/// compare pathname/handle generations on every subsequent use.
pub struct PinnedDirectory {
    path: PathBuf,
    chain: Vec<(File, Identity, Protection)>,
}
impl PinnedDirectory {
    pub fn open(path: &Path) -> io::Result<Self> {
        #[cfg(test)]
        let _metric = observation_metrics::span("pinned_open");
        if !path.is_absolute()
            || path
                .as_os_str()
                .as_bytes()
                .split(|b| *b == b'/')
                .any(|p| p == b"." || p == b"..")
        {
            return Err(denied("absolute clean source root required"));
        }
        let slash = CString::new("/").unwrap();
        let fd = unsafe {
            libc::open(
                slash.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let root = unsafe { File::from_raw_fd(fd) };
        let metadata = stat(&root)?;
        let original_protection = protection(&root, &metadata)?;
        let mut chain = vec![(root, identity(&metadata), original_protection)];
        for c in path.components() {
            if let Component::Normal(child) = c {
                let file = open_at(&chain.last().unwrap().0, child, true)?;
                let metadata = stat(&file)?;
                let id = identity(&metadata);
                let protection = protection(&file, &metadata)?;
                chain.push((file, id, protection));
            }
        }
        Ok(Self {
            path: path.into(),
            chain,
        })
    }
    fn file(&self) -> &File {
        &self.chain.last().unwrap().0
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn identity(&self) -> &Identity {
        &self.chain.last().unwrap().1
    }
    pub fn recheck(&self) -> io::Result<()> {
        #[cfg(test)]
        let _metric = observation_metrics::span("pinned_recheck");
        self.recheck_checked(&|_| Ok(()))
    }
    fn recheck_checked(&self, checkpoint: &impl Fn(usize) -> io::Result<()>) -> io::Result<()> {
        let components: Vec<_> = self
            .path
            .components()
            .filter_map(|c| {
                if let Component::Normal(n) = c {
                    Some(n)
                } else {
                    None
                }
            })
            .collect();
        if self.chain.len() != components.len() + 1 {
            return Err(denied("directory pin chain differs"));
        }
        let path_stat = |i: usize| -> io::Result<libc::stat> {
            let mut value = std::mem::MaybeUninit::uninit();
            let child = if i == 0 {
                CString::new("/").unwrap()
            } else {
                name(components[i - 1])?
            };
            let parent = if i == 0 {
                libc::AT_FDCWD
            } else {
                self.chain[i - 1].0.as_raw_fd()
            };
            if unsafe {
                libc::fstatat(
                    parent,
                    child.as_ptr(),
                    value.as_mut_ptr(),
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            } != 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(unsafe { value.assume_init() })
        };
        let matches = |observed: &libc::stat, expected: &Identity, p: &Protection| {
            identity(observed) == *expected
                && observed.st_mode & libc::S_IFMT == libc::S_IFDIR
                && observed.st_uid == p.uid
                && observed.st_gid == p.gid
                && observed.st_mode & 0o7777 == p.mode
                && observed.st_flags == p.flags
        };
        // Each pathname entry must be the SAME inode as its retained fd. ACLs
        // and xattrs belong to that inode, so read their full current values
        // through the retained fd instead of opening the same directory and
        // redundantly reading its protection three times. A second full
        // attribute pass below still fences changes during this scan.
        for (i, (file, expected, p)) in self.chain.iter().enumerate() {
            let current = path_stat(i)?;
            let held = stat(file)?;
            if !matches(&current, expected, p)
                || !matches(&held, expected, p)
                || protection(file, &held)? != *p
            {
                return Err(denied("directory generation changed"));
            }
            checkpoint(i)?;
        }
        // Recheck the complete pathname chain after the attribute reads: a
        // renamed ancestor or nofollow leaf cannot hide behind retained fds.
        for (i, (file, expected, p)) in self.chain.iter().enumerate() {
            let held = stat(file)?;
            if !matches(&path_stat(i)?, expected, p)
                || !matches(&held, expected, p)
                || protection(file, &held)? != *p
            {
                return Err(denied("directory pathname generation changed"));
            }
        }
        Ok(())
    }
    pub(crate) fn private(&self) -> io::Result<()> {
        self.recheck()?;
        require_private(&protection(self.file(), &stat(self.file())?)?, true)
    }
    fn child_private(&self, child: &str, create: bool) -> io::Result<Self> {
        self.child_private_checked(child, create, &|_| Ok(()))
    }
    fn child_private_checked(
        &self,
        child: &str,
        create: bool,
        checkpoint: &impl Fn(&str) -> io::Result<()>,
    ) -> io::Result<Self> {
        #[cfg(test)]
        let _metric = observation_metrics::span("child_private");
        self.private()?;
        let c = name(OsStr::new(child))?;
        if create && unsafe { libc::mkdirat(self.file().as_raw_fd(), c.as_ptr(), 0o700) } != 0 {
            let error = io::Error::last_os_error();
            if error.kind() != io::ErrorKind::AlreadyExists {
                return Err(error);
            }
        }
        checkpoint("parent-admitted")?;
        #[cfg(not(test))]
        let reference = false;
        #[cfg(test)]
        let reference = observation_metrics::child_reference_enabled();
        let result = if reference {
            let result = Self::open(&self.path.join(child))?;
            result.private()?;
            self.recheck()?;
            result
        } else {
            // Clone DIRECTORY handles only. Never acquire/close a second fd
            // for a SQLite inode while its process-associated lock is held.
            let mut chain = self
                .chain
                .iter()
                .map(|(file, id, p)| Ok((file.try_clone()?, id.clone(), p.clone())))
                .collect::<io::Result<Vec<_>>>()?;
            checkpoint("parent-handles-cloned")?;
            let file = open_at(self.file(), OsStr::new(child), true)?;
            let info = stat(&file)?;
            let security = protection(&file, &info)?;
            chain.push((file, identity(&info), security));
            checkpoint("child-opened")?;
            let result = Self {
                path: self.path.join(child),
                chain,
            };
            // Original parent and new child expectations must agree for all
            // shared ancestors. Two complete fresh paths/security passes
            // fence the actual open; private assertion is then pure data.
            Self::recheck_set(&[self, &result])?;
            require_private(&result.chain.last().unwrap().2, true)?;
            checkpoint("child-chain-checked")?;
            // Keep the original final parent boundary AFTER the child security
            // read. Cloning avoids reopening ancestors; it does not waive the
            // owning parent's return-time generation/security observation.
            self.recheck()?;
            result
        };
        Ok(result)
    }
    fn sync(&self) -> io::Result<()> {
        self.recheck()?;
        self.file().sync_all()
    }
    /// Acquire existing private directories only. No content is read and no
    /// intermediate pin escapes before complete final path/security admission.
    fn descendant_private(&self, children: &[&str]) -> io::Result<Self> {
        #[cfg(test)]
        if observation_metrics::descendant_reference_enabled() {
            let mut result = None;
            for child in children {
                result = Some(
                    result
                        .as_ref()
                        .unwrap_or(self)
                        .child_private(child, false)?,
                );
            }
            return result.ok_or_else(|| denied("empty reference descendant"));
        }
        self.descendant_private_checked(children, &|_, _| Ok(()))
    }
    fn descendant_private_checked(
        &self,
        children: &[&str],
        checkpoint: &impl Fn(&str, usize) -> io::Result<()>,
    ) -> io::Result<Self> {
        #[cfg(test)]
        let _metric = observation_metrics::span("descendant_private");
        if children.is_empty() || children.len() > 8 {
            return Err(denied("bounded nonempty private descendant required"));
        }
        for child in children {
            name(OsStr::new(child))?;
        }
        self.private()?;
        checkpoint("base-admitted", 0)?;
        let clone_chain = |chain: &[(File, Identity, Protection)]| {
            chain
                .iter()
                .map(|(file, id, p)| Ok((file.try_clone()?, id.clone(), p.clone())))
                .collect::<io::Result<Vec<_>>>()
        };
        let mut result = Self {
            path: self.path.clone(),
            chain: clone_chain(&self.chain)?,
        };
        let mut final_parent = None;
        for (i, child) in children.iter().enumerate() {
            if i + 1 == children.len() {
                // Preserve the last child_private's AFTER-leaf parent boundary,
                // including every intermediate ancestor, not just the base.
                final_parent = Some(Self {
                    path: result.path.clone(),
                    chain: clone_chain(&result.chain)?,
                });
            }
            let file = open_at(result.file(), OsStr::new(child), true)?;
            let info = stat(&file)?;
            let security = protection(&file, &info)?;
            require_private(&security, true)?;
            result.path.push(child);
            result.chain.push((file, identity(&info), security));
            checkpoint("component-pinned", i)?;
        }
        Self::recheck_set(&[self, &result])?;
        require_private(&result.chain.last().unwrap().2, true)?;
        checkpoint("chain-checked", children.len())?;
        final_parent.unwrap().recheck()?;
        Ok(result)
    }
    /// Fresh union observation, not a cache. Shared ancestors are the same
    /// path/inode/protection expectation; inspect each unique object twice
    /// instead of repeating it for every overlapping root. Conflicting pins
    /// are rejected before any evidence can be returned.
    fn recheck_set(roots: &[&Self]) -> io::Result<()> {
        Self::recheck_set_checked(roots, &|_| Ok(()))
    }
    fn recheck_set_checked(
        roots: &[&Self],
        checkpoint: &impl Fn(usize) -> io::Result<()>,
    ) -> io::Result<()> {
        #[cfg(test)]
        let _metric = observation_metrics::span("pinned_set");
        struct Entry<'a> {
            file: &'a File,
            parent: libc::c_int,
            name: CString,
            id: &'a Identity,
            security: &'a Protection,
        }
        let mut entries = BTreeMap::<PathBuf, Entry<'_>>::new();
        for root in roots {
            let components = root
                .path
                .components()
                .filter_map(|c| {
                    if let Component::Normal(n) = c {
                        Some(n)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            if root.chain.len() != components.len() + 1 {
                return Err(denied("directory union pin chain differs"));
            }
            let mut path = PathBuf::from("/");
            for (i, (file, id, security)) in root.chain.iter().enumerate() {
                if i > 0 {
                    path.push(components[i - 1]);
                }
                if let Some(old) = entries.get(&path) {
                    if old.id != id || old.security != security {
                        return Err(denied("directory union has conflicting source pins"));
                    }
                } else {
                    entries.insert(
                        path.clone(),
                        Entry {
                            file,
                            parent: if i == 0 {
                                libc::AT_FDCWD
                            } else {
                                root.chain[i - 1].0.as_raw_fd()
                            },
                            name: if i == 0 {
                                CString::new("/").unwrap()
                            } else {
                                name(components[i - 1])?
                            },
                            id,
                            security,
                        },
                    );
                }
            }
        }
        for pass in 0..2 {
            for (i, entry) in entries.values().enumerate() {
                let mut named = std::mem::MaybeUninit::uninit();
                if unsafe {
                    libc::fstatat(
                        entry.parent,
                        entry.name.as_ptr(),
                        named.as_mut_ptr(),
                        libc::AT_SYMLINK_NOFOLLOW,
                    )
                } != 0
                {
                    return Err(io::Error::last_os_error());
                }
                let named = unsafe { named.assume_init() };
                let held = stat(entry.file)?;
                let matches = |s: &libc::stat| {
                    identity(s) == *entry.id
                        && s.st_mode & libc::S_IFMT == libc::S_IFDIR
                        && s.st_uid == entry.security.uid
                        && s.st_gid == entry.security.gid
                        && s.st_mode & 0o7777 == entry.security.mode
                        && s.st_flags == entry.security.flags
                };
                if !matches(&named)
                    || !matches(&held)
                    || protection(entry.file, &held)? != *entry.security
                {
                    return Err(denied("directory union identity or protection changed"));
                }
                if pass == 0 {
                    checkpoint(i)?;
                }
            }
        }
        Ok(())
    }
}

/// Kept open BEFORE SQLite acquires process-associated POSIX locks. Never open
/// and close another descriptor for this inode while its SQLite transaction is
/// held: closing any such descriptor could release all of this process's locks.
pub(crate) struct CoordinationFilePin {
    parent: PinnedDirectory,
    file: File,
    name: CString,
    identity: Identity,
    protection: Protection,
}
impl CoordinationFilePin {
    pub(crate) fn open(path: &Path) -> io::Result<Self> {
        let parent =
            PinnedDirectory::open(path.parent().ok_or_else(|| denied("lock parent missing"))?)?;
        let root = stat(parent.file())?;
        let root_protection = protection(parent.file(), &root)?;
        if root.st_uid != unsafe { libc::geteuid() }
            || root.st_mode & 0o022 != 0
            || root_protection.acl.is_some()
        {
            return Err(denied("coordination parent access unknown"));
        }
        let filename = path
            .file_name()
            .ok_or_else(|| denied("lock name missing"))?;
        let file = open_at(parent.file(), filename, false)?;
        let s = stat(&file)?;
        let p = protection(&file, &s)?;
        if p.uid != unsafe { libc::geteuid() }
            || p.mode != 0o600
            || p.acl.is_some()
            || p.flags != 0
            || p.xattrs
                .iter()
                .any(|(k, v)| k != b"com.apple.provenance" || v.len() > 64)
        {
            return Err(denied("unsafe existing native coordination file"));
        }
        let result = Self {
            parent,
            file,
            name: name(filename)?,
            identity: identity(&s),
            protection: p,
        };
        result.recheck()?;
        Ok(result)
    }
    pub(crate) fn key(&self) -> (String, String) {
        (
            self.identity.device.to_string(),
            self.identity.inode.to_string(),
        )
    }
    pub(crate) fn recheck(&self) -> io::Result<()> {
        self.parent.recheck()?;
        let s = stat(&self.file)?;
        let mut path = std::mem::MaybeUninit::uninit();
        if unsafe {
            libc::fstatat(
                self.parent.file().as_raw_fd(),
                self.name.as_ptr(),
                path.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } != 0
        {
            return Err(io::Error::last_os_error());
        }
        let path = unsafe { path.assume_init() };
        if identity(&s) != self.identity
            || identity(&path) != self.identity
            || path.st_mode & libc::S_IFMT != libc::S_IFREG
            || path.st_nlink != 1
            || s.st_nlink != 1
            || protection(&self.file, &s)? != self.protection
            || path.st_uid != s.st_uid
            || path.st_gid != s.st_gid
            || path.st_mode != s.st_mode
            || path.st_flags != s.st_flags
        {
            return Err(denied("coordination inode or protection changed"));
        }
        Ok(())
    }
}

/// Matches the opaque receipt produced by official admission while K is held.
/// It grants no authority by itself. A native coordination owner compares it
/// before/after acquisition; paths are fixed here, never taken from a record.
pub(crate) fn ownership_receipt(root: &PinnedDirectory) -> io::Result<String> {
    match capture_file(root, "opencodex-journal.json", 1024 * 1024)? {
        Image::Absent => Ok("absent".into()),
        Image::File {
            identity,
            protection,
            bytes,
            ..
        } => {
            if protection.mode & 0o022 != 0 || protection.acl.is_some() || protection.flags != 0 {
                return Err(denied(
                    "ownership receipt has unknown write access; not repaired",
                ));
            }
            Ok(digest(&encode(&[
                identity.device.to_string(),
                identity.inode.to_string(),
                protection.uid.to_string(),
                protection.gid.to_string(),
                protection.mode.to_string(),
                digest(&bytes),
            ])?))
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum Image {
    Absent,
    File {
        identity: Identity,
        protection: Protection,
        change: [i64; 4],
        bytes: Vec<u8>,
    },
}
impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Absent => f.write_str("Absent"),
            Self::File { bytes, .. } => f
                .debug_struct("File")
                .field("bytes", &bytes.len())
                .finish_non_exhaustive(),
        }
    }
}
impl Image {
    pub fn bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Absent => None,
            Self::File { bytes, .. } => Some(bytes),
        }
    }
    pub fn same_native_generation(&self, other: &Self) -> bool {
        self == other
    }
}
fn capture_file(root: &PinnedDirectory, child: &str, limit: usize) -> io::Result<Image> {
    #[cfg(test)]
    let _metric = observation_metrics::span("capture_file");
    root.recheck()?;
    let file = match open_at(root.file(), OsStr::new(child), false) {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            // An absent result still needs the same original parent and a second
            // no-follow lookup; only ENOENT, never access denial, means absence.
            root.recheck()?;
            return match open_at(root.file(), OsStr::new(child), false) {
                Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Image::Absent),
                _ => Err(denied("absence changed")),
            };
        }
        Err(e) => return Err(e),
    };
    let before = stat(&file)?;
    if before.st_uid != unsafe { libc::geteuid() }
        || before.st_size < 0
        || before.st_size as usize > limit
    {
        return Err(denied("source owner or size not admitted"));
    }
    let protections = protection(&file, &before)?;
    let mut bytes = Vec::new();
    (&file).take(limit as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(denied("source exceeded read limit"));
    }
    let after = stat(&file)?;
    let current = open_at(root.file(), OsStr::new(child), false)?;
    if version(&before) != version(&after)
        || version(&after) != version(&stat(&current)?)
        || protection(&file, &after)? != protections
        || protection(&current, &stat(&current)?)? != protections
        || bytes.len() != after.st_size as usize
    {
        return Err(denied("source changed during capture"));
    }
    root.recheck()?;
    Ok(Image::File {
        identity: identity(&after),
        protection: protections,
        change: [
            after.st_mtime,
            after.st_mtime_nsec,
            after.st_ctime,
            after.st_ctime_nsec,
        ],
        bytes,
    })
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Capture {
    root: Identity,
    root_protection: Protection,
    images: [Image; 3],
}

#[path = "macos_client_tree.rs"]
pub mod tree;
use tree::TreeSnapshot;
#[path = "macos_client_grok.rs"]
pub mod grok;
use grok::GrokSnapshot;

impl std::fmt::Debug for Capture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Capture")
            .field("images", &self.images)
            .finish_non_exhaustive()
    }
}
impl Capture {
    pub fn read(root: &PinnedDirectory) -> io::Result<Self> {
        let one = || -> io::Result<Self> {
            Ok(Self {
                root: root.identity().clone(),
                root_protection: protection(root.file(), &stat(root.file())?)?,
                images: [
                    capture_file(root, NAMES[0], MAX_CONTENT)?,
                    capture_file(root, NAMES[1], MAX_CONTENT)?,
                    capture_file(root, NAMES[2], MAX_CONTENT)?,
                ],
            })
        };
        let before = one()?;
        let after = one()?;
        if before != after {
            return Err(denied("artifact set changed during capture"));
        }
        Ok(after)
    }
    pub fn images(&self) -> &[Image; 3] {
        &self.images
    }
    fn valid(&self) -> bool {
        self.images.iter().all(|image| match image {
            Image::Absent => true,
            Image::File {
                protection, bytes, ..
            } => {
                protection.uid == unsafe { libc::geteuid() }
                    && bytes.len() <= MAX_CONTENT
                    && protection
                        .acl
                        .as_ref()
                        .is_none_or(|a| a.len() <= MAX_SECURITY)
                    && protection
                        .xattrs
                        .iter()
                        .map(|(k, v)| k.len() + v.len())
                        .sum::<usize>()
                        <= MAX_SECURITY
            }
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Original,
    Ready,
    PreStop,
    Stopped,
}
impl Phase {
    fn index(self) -> usize {
        match self {
            Self::Original => 0,
            Self::Ready => 1,
            Self::PreStop => 2,
            Self::Stopped => 3,
        }
    }
    fn filename(self) -> &'static str {
        match self {
            Self::Original => "original.json",
            Self::Ready => "ready.json",
            Self::PreStop => "pre-stop.json",
            Self::Stopped => "stopped.json",
        }
    }
    fn predecessor(self) -> Option<Self> {
        match self {
            Self::Original => None,
            Self::Ready => Some(Self::Original),
            Self::PreStop => Some(Self::Ready),
            Self::Stopped => Some(Self::PreStop),
        }
    }
    fn next(self) -> Option<Self> {
        match self {
            Self::Original => Some(Self::Ready),
            Self::Ready => Some(Self::PreStop),
            Self::PreStop => Some(Self::Stopped),
            Self::Stopped => None,
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Frame {
    schema_version: u32,
    attempt: String,
    uid: u32,
    data_root: Identity,
    client_root: Identity,
    phase: Phase,
    previous_sha256: Option<String>,
    capture: Capture,
    tree: TreeSnapshot,
    profile_root: Identity,
    grok: GrokSnapshot,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    original_root_absence: Option<root_creation::References>,
}
fn encode(value: &impl Serialize) -> io::Result<Vec<u8>> {
    serde_json::to_vec(value).map_err(|_| denied("frame encoding failed"))
}
fn digest(bytes: &[u8]) -> String {
    crate::infrastructure::hash::sha256_hex(bytes)
}
fn decode(bytes: &[u8]) -> io::Result<Frame> {
    if bytes.len() > MAX_FRAME {
        return Err(denied("journal frame over limit"));
    }
    let frame: Frame = serde_json::from_value(
        parse_strict_json_v2(bytes).map_err(|_| denied("invalid or duplicate JSON"))?,
    )
    .map_err(|_| denied("invalid frame fields"))?;
    if !matches!(
        (frame.schema_version, frame.original_root_absence.is_some()),
        (4, false) | (5, true)
    ) {
        return Err(denied("unsupported client tree frame schema"));
    }
    Ok(frame)
}
fn create(directory: &PinnedDirectory, filename: &str, bytes: &[u8]) -> io::Result<()> {
    directory.private()?;
    if bytes.len() > MAX_FRAME {
        return Err(denied("journal frame too large"));
    }
    let name = name(OsStr::new(filename))?;
    let fd = unsafe {
        libc::openat(
            directory.file().as_raw_fd(),
            name.as_ptr(),
            libc::O_RDWR | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            0o400,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    let mut file = unsafe { File::from_raw_fd(fd) };
    // Refuse inherited ACLs/attributes, do not repair or take ownership.
    require_private(&protection(&file, &stat(&file)?)?, false)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    directory.sync()?;
    let observed = capture_file(directory, filename, MAX_FRAME)?;
    if observed.bytes() != Some(bytes) {
        return Err(denied("journal readback differs"));
    }
    Ok(())
}

/// A protected set of immutable snapshots, not a restore permit. original_sha256
/// must remain bound by the owning lifecycle transaction; never discover a new
/// baseline by reading a changed journal. No startup auto-recovery is provided.
type FramePrefix = Vec<(Frame, Vec<u8>)>;
type OpenedPrefix = (Originals, FramePrefix);
pub struct Originals {
    profile: PinnedDirectory,
    data: PinnedDirectory,
    client: PinnedDirectory,
    directory: PinnedDirectory,
    attempt: String,
    original_sha256: String,
    original_root_absence: Option<root_creation::References>,
    retired_root_slot: Option<String>,
    known: std::sync::Mutex<[Option<String>; 4]>,
}
pub(super) struct OriginalKeys<'a> {
    pub(super) attempt: &'a str,
    pub(super) original_sha256: &'a str,
    pub(super) expected_phases: &'a [(Phase, String)],
}
impl Originals {
    #[cfg(test)]
    pub fn begin(data_root: &Path, client_root: &Path) -> io::Result<Self> {
        // Existing single-client fixtures explicitly use their known parent
        // as synthetic profile; production must supply ProcessContext profile.
        Self::begin_for_profile(
            data_root,
            client_root,
            client_root
                .parent()
                .ok_or_else(|| denied("test profile missing"))?,
        )
    }
    pub fn begin_for_profile(
        data_root: &Path,
        client_root: &Path,
        profile_root: &Path,
    ) -> io::Result<Self> {
        Self::begin_profile_attempt(
            data_root,
            client_root,
            profile_root,
            &uuid::Uuid::new_v4().to_string(),
            None,
            &|_| Ok(()),
        )
    }
    pub(super) fn begin_reserved_profile(
        data_root: &Path,
        client_root: &Path,
        profile_root: &Path,
        reservation: &session_index::CaptureReservation<'_>,
    ) -> io::Result<Self> {
        reservation.require(data_root, client_root, profile_root)?;
        Self::begin_profile_attempt(
            data_root,
            client_root,
            profile_root,
            reservation.attempt(),
            reservation.root_absence()?,
            &|point| {
                reservation.require(data_root, client_root, profile_root)?;
                reservation.root_absence()?;
                #[cfg(test)]
                if std::env::var("OCXD024_CAPTURE_INTERRUPT_POINT")
                    .ok()
                    .as_deref()
                    == Some(point)
                {
                    if !data_root.starts_with("/private/tmp")
                        || !data_root.ancestors().any(|p| {
                            p.file_name().is_some_and(|n| {
                                n.to_string_lossy()
                                    .starts_with("ocxd024-versioned-lifecycle-")
                            })
                        })
                    {
                        return Err(denied("capture interruption fixture refused"));
                    }
                    return Err(denied(
                        "owned capture interruption after reserved publication",
                    ));
                }
                let _ = point;
                Ok(())
            },
        )
    }
    fn begin_profile_attempt(
        data_root: &Path,
        client_root: &Path,
        profile_root: &Path,
        attempt: &str,
        original_root_absence: Option<root_creation::References>,
        checkpoint: &impl Fn(&str) -> io::Result<()>,
    ) -> io::Result<Self> {
        if uuid::Uuid::parse_str(attempt)
            .map(|v| v.to_string() != attempt)
            .unwrap_or(true)
        {
            return Err(denied("capture attempt is not canonical"));
        }
        let grok_path = profile_root.join(".grok");
        if data_root.starts_with(&grok_path)
            || grok_path.starts_with(data_root)
            || client_root.starts_with(&grok_path)
            || grok_path.starts_with(client_root)
        {
            return Err(denied("Grok client and journal roots overlap"));
        }
        let profile = PinnedDirectory::open(profile_root)?;
        let ps = stat(profile.file())?;
        if ps.st_uid != unsafe { libc::geteuid() } || ps.st_mode & 0o022 != 0 {
            return Err(denied("profile owner or write access unknown"));
        }

        if data_root.starts_with(client_root) || client_root.starts_with(data_root) {
            return Err(denied("client and journal roots must be disjoint"));
        }
        let data = PinnedDirectory::open(data_root)?;
        data.private()?;
        let client = PinnedDirectory::open(client_root)?;
        let source = stat(client.file())?;
        if source.st_uid != unsafe { libc::geteuid() } || source.st_mode & 0o022 != 0 {
            return Err(denied("client root owner or write access unknown"));
        }
        let existing_manager =
            if session_index::observation::entry(&data, "manager-state")?.is_some() {
                Some(data.child_private("manager-state", false)?)
            } else {
                None
            };
        if let Some(origin) = &original_root_absence {
            let manager = existing_manager
                .as_ref()
                .ok_or_else(|| denied("registered root manager is absent"))?;
            if origin.id != attempt {
                return Err(denied(
                    "first-root original attempt differs from root origin",
                ));
            }
            root_creation::verify_indexed(manager, &data, profile_root, &client, origin)?;
        } else if let Some(manager) = &existing_manager {
            root_creation::require_no_pending_root(manager)?;
        }
        let capture = Capture::read(&client)?;
        let tree = TreeSnapshot::read(&client)?;
        let grok = GrokSnapshot::read(&profile)?;
        if original_root_absence.is_some() && !tree.first_root_execution_baseline() {
            return Err(denied(
                "new client data cannot become the original absent-root baseline",
            ));
        }
        let manager = data.child_private("manager-state", true)?;
        let vault = manager.child_private("client-originals", true)?;
        let attempt = attempt.to_owned();
        let c = name(OsStr::new(&attempt))?;
        if unsafe { libc::mkdirat(vault.file().as_raw_fd(), c.as_ptr(), 0o700) } != 0 {
            return Err(io::Error::last_os_error());
        }
        checkpoint("directory_created")?;
        let directory = PinnedDirectory::open(&vault.path.join(&attempt))?;
        directory.private()?;
        vault.sync()?;
        tree.archive(&client, &directory)?;
        checkpoint("codex_archived")?;
        grok.archive(&profile, &directory)?;
        checkpoint("grok_archived")?;
        let frame = Frame {
            schema_version: if original_root_absence.is_some() {
                5
            } else {
                4
            },
            attempt: attempt.clone(),
            uid: unsafe { libc::geteuid() },
            data_root: data.identity().clone(),
            client_root: client.identity().clone(),
            phase: Phase::Original,
            previous_sha256: None,
            capture,
            tree,
            profile_root: profile.identity().clone(),
            grok,
            original_root_absence: original_root_absence.clone(),
        };
        if Capture::read(&client)? != frame.capture
            || TreeSnapshot::read(&client)? != frame.tree
            || GrokSnapshot::read(&profile)? != frame.grok
        {
            return Err(denied("source changed before original publication"));
        }
        let bytes = encode(&frame)?;
        create(&directory, Phase::Original.filename(), &bytes)?;
        checkpoint("original_published")?;
        data.recheck()?;
        client.recheck()?;
        if Capture::read(&client)? != frame.capture
            || TreeSnapshot::read(&client)? != frame.tree
            || GrokSnapshot::read(&profile)? != frame.grok
        {
            return Err(denied(
                "source changed during original publication; original retained",
            ));
        }
        let hash = digest(&bytes);
        Ok(Self {
            profile,
            data,
            client,
            directory,
            attempt,
            original_sha256: hash.clone(),
            original_root_absence,
            retired_root_slot: None,
            known: std::sync::Mutex::new([Some(hash), None, None, None]),
        })
    }
    #[cfg(test)]
    pub fn open(
        data_root: &Path,
        client_root: &Path,
        attempt: &str,
        original_sha256: &str,
        expected_phases: &[(Phase, String)],
    ) -> io::Result<Self> {
        Self::open_for_profile(
            data_root,
            client_root,
            client_root
                .parent()
                .ok_or_else(|| denied("test profile missing"))?,
            attempt,
            original_sha256,
            expected_phases,
        )
    }
    pub fn open_for_profile(
        data_root: &Path,
        client_root: &Path,
        profile_root: &Path,
        attempt: &str,
        original_sha256: &str,
        expected_phases: &[(Phase, String)],
    ) -> io::Result<Self> {
        Self::open_profile_origin(
            data_root,
            client_root,
            profile_root,
            OriginalKeys {
                attempt,
                original_sha256,
                expected_phases,
            },
            None,
        )
    }
    pub(super) fn open_profile_origin(
        data_root: &Path,
        client_root: &Path,
        profile_root: &Path,
        keys: OriginalKeys<'_>,
        original_root_absence: Option<root_creation::References>,
    ) -> io::Result<Self> {
        Self::open_profile_origin_prefix(
            data_root,
            client_root,
            profile_root,
            keys,
            original_root_absence,
        )
        .map(|v| v.0)
    }
    /// Facts consumed within one fresh observation. This does not cache frames
    /// on Originals, adopt hashes or grant any native mutation permission.
    fn open_profile_origin_prefix(
        data_root: &Path,
        client_root: &Path,
        profile_root: &Path,
        keys: OriginalKeys<'_>,
        original_root_absence: Option<root_creation::References>,
    ) -> io::Result<OpenedPrefix> {
        let OriginalKeys {
            attempt,
            original_sha256,
            expected_phases,
        } = keys;
        let profile = PinnedDirectory::open(profile_root)?;
        if data_root.starts_with(client_root) || client_root.starts_with(data_root) {
            return Err(denied("client and journal roots must be disjoint"));
        }
        if uuid::Uuid::parse_str(attempt)
            .map(|v| v.to_string() != attempt)
            .unwrap_or(true)
            || original_sha256.len() != 64
            || !original_sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(denied("explicit attempt and original digest required"));
        }
        let data = PinnedDirectory::open(data_root)?;
        data.private()?;
        let client = PinnedDirectory::open(client_root)?;
        let manager = data.child_private("manager-state", false)?;
        if let Some(origin) = &original_root_absence {
            if origin.id != attempt {
                return Err(denied("root-aware original source differs"));
            }
            root_creation::verify_indexed(&manager, &data, profile_root, &client, origin)?;
        } else {
            root_creation::require_no_pending_root(&manager)?;
        }
        let directory = manager.descendant_private(&["client-originals", attempt])?;
        let mut known = [Some(original_sha256.to_owned()), None, None, None];
        for (phase, hash) in expected_phases {
            if *phase == Phase::Original
                || known[phase.index()].is_some()
                || hash.len() != 64
                || !hash
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err(denied("phase digest must be independent and unique"));
            }
            known[phase.index()] = Some(hash.clone());
        }
        let result = Self {
            profile,
            data,
            client,
            directory,
            attempt: attempt.into(),
            original_sha256: original_sha256.into(),
            original_root_absence,
            retired_root_slot: None,
            known: std::sync::Mutex::new(known),
        };
        // One fresh complete prefix validates every independently named frame
        // exactly once. No result survives this open/validation boundary.
        let last = expected_phases
            .iter()
            .max_by_key(|(phase, _)| phase.index())
            .map(|(phase, _)| *phase)
            .unwrap_or(Phase::Original);
        let frames = result.frames_through(last)?;
        Ok((result, frames))
    }
    pub(super) fn logical_client(&self) -> PathBuf {
        if self.original_root_absence.is_some() {
            self.profile.path().join(".codex")
        } else {
            self.client.path().into()
        }
    }
    pub fn attempt(&self) -> &str {
        &self.attempt
    }
    pub fn original_sha256(&self) -> &str {
        &self.original_sha256
    }
    pub fn read(&self, phase: Phase) -> io::Result<Capture> {
        Ok(self.frame(phase)?.0.capture)
    }
    /// Read only a file named by the independently bound phase tree. Never
    /// falls back to current source bytes or learns a new content reference.
    pub fn archived_tree_file(
        &self,
        phase: Phase,
        relative: &str,
    ) -> io::Result<zeroize::Zeroizing<Vec<u8>>> {
        let bytes = self
            .frame(phase)?
            .0
            .tree
            .archived_file(&self.directory, relative)?;
        self.verify_root_namespace()?;
        Ok(bytes)
    }
    pub fn archived_grok_file(
        &self,
        phase: Phase,
        relative: &str,
    ) -> io::Result<zeroize::Zeroizing<Vec<u8>>> {
        let bytes = self
            .frame(phase)?
            .0
            .grok
            .archived_file(&self.directory, relative)?;
        self.verify_root_namespace()?;
        Ok(bytes)
    }
    fn verify_root_namespace(&self) -> io::Result<()> {
        if let Some(origin) = &self.original_root_absence {
            origin.verify_namespace(&self.data.child_private("manager-state", false)?)?;
        }
        Ok(())
    }
    fn frame(&self, phase: Phase) -> io::Result<(Frame, Vec<u8>)> {
        self.frames_through(phase)?
            .pop()
            .ok_or_else(|| denied("empty validated phase prefix"))
    }
    fn frames_through(&self, phase: Phase) -> io::Result<Vec<(Frame, Vec<u8>)>> {
        self.frames_through_checked(phase, &|_| Ok(()))
    }
    /// A single validation boundary reads the entire authenticated prefix.
    /// It keeps no cache across mutations, archive writes or later callers.
    fn frames_through_checked(
        &self,
        last: Phase,
        observed: &impl Fn(Phase) -> io::Result<()>,
    ) -> io::Result<Vec<(Frame, Vec<u8>)>> {
        #[cfg(test)]
        let _metric = observation_metrics::span("frames_prefix");
        let mut result = Vec::new();
        let mut predecessor = None;
        for phase in [
            Phase::Original,
            Phase::Ready,
            Phase::PreStop,
            Phase::Stopped,
        ]
        .into_iter()
        .take(last.index() + 1)
        {
            let value = self.frame_with_predecessor(phase, predecessor.as_deref())?;
            observed(phase)?;
            predecessor = Some(digest(&value.1));
            result.push(value);
        }
        self.verify_root_namespace()?;
        Ok(result)
    }
    fn frame_with_predecessor(
        &self,
        phase: Phase,
        predecessor: Option<&str>,
    ) -> io::Result<(Frame, Vec<u8>)> {
        #[cfg(test)]
        let _metric = observation_metrics::span("frame");
        #[cfg(not(test))]
        let reference = false;
        #[cfg(test)]
        let reference = observation_metrics::reference_enabled();
        if reference {
            self.data.private()?;
            self.profile.recheck()?;
            self.client.recheck()?;
            self.directory.private()?;
        } else {
            PinnedDirectory::recheck_set(&[
                &self.data,
                &self.profile,
                &self.client,
                &self.directory,
            ])?;
            require_private(&self.data.chain.last().unwrap().2, true)?;
            require_private(&self.directory.chain.last().unwrap().2, true)?;
        }
        let image = capture_file(&self.directory, phase.filename(), MAX_FRAME)?;
        let Image::File {
            protection: saved_protection,
            bytes,
            ..
        } = image
        else {
            return Err(denied("journal phase missing"));
        };
        require_private(&saved_protection, false)?;
        let expected = self
            .known
            .lock()
            .map_err(|_| denied("journal digest ownership unavailable"))?[phase.index()]
        .clone()
        .ok_or_else(|| denied("no independently retained phase digest"))?;
        if digest(&bytes) != expected {
            return Err(denied("sealed phase bytes changed"));
        }
        let frame = decode(&bytes)?;
        if !matches!(
            (frame.schema_version, frame.original_root_absence.is_some()),
            (4, false) | (5, true)
        ) || frame.original_root_absence != self.original_root_absence
            || frame.attempt != self.attempt
            || frame.uid != unsafe { libc::geteuid() }
            || frame.phase != phase
            || frame.data_root != *self.data.identity()
            || frame.client_root != *self.client.identity()
            || frame.profile_root != *self.profile.identity()
            || !frame.tree.coordinated()
            || frame.capture.root != frame.client_root
            || frame.tree.root != frame.client_root
            || frame.tree.root_protection != frame.capture.root_protection
            || !frame.tree.valid()
            || !frame.tree.matches_artifacts(&frame.capture)
            || frame.capture.root_protection
                != protection(self.client.file(), &stat(self.client.file())?)?
            || !frame.capture.valid()
        {
            return Err(denied("journal context or snapshot changed"));
        }
        if phase.predecessor().is_some() {
            if predecessor.is_none() || frame.previous_sha256.as_deref() != predecessor {
                return Err(denied("journal predecessor changed"));
            }
        } else if predecessor.is_some()
            || frame.previous_sha256.is_some()
            || digest(&bytes) != self.original_sha256
        {
            return Err(denied("original snapshot digest changed"));
        }
        if let Some(origin) = &self.original_root_absence {
            let anchor = if let Some(slot) = &self.retired_root_slot {
                self.directory.path().join(slot)
            } else {
                self.profile.path().join(".codex")
            };
            root_creation::verify_indexed_anchor(
                &self.data.child_private("manager-state", false)?,
                &self.data,
                self.profile.path(),
                &self.client,
                origin,
                &anchor,
            )?;
            if phase == Phase::Original && !frame.tree.first_root_execution_baseline() {
                return Err(denied("first root Original includes unowned client data"));
            }
        }
        frame.tree.verify_archive(&self.directory)?;
        frame.grok.verify_archive(&self.directory)?;
        self.verify_root_namespace()?;
        Ok((frame, bytes))
    }
    /// Seal new actual snapshots in order; never replace a recorded phase or
    /// learn a new startup original from current disk after a crash.
    pub fn seal(&self, phase: Phase) -> io::Result<String> {
        self.seal_checked(phase, || Ok(()))
    }
    fn seal_checked(
        &self,
        phase: Phase,
        checkpoint: impl FnOnce() -> io::Result<()>,
    ) -> io::Result<String> {
        if self.retired_root_slot.is_some() {
            return Err(denied("archived root view cannot publish new history"));
        }
        let prior = phase
            .predecessor()
            .ok_or_else(|| denied("original cannot be resealed"))?;
        let (_, previous) = self.frame(prior)?;
        if let Some(next) = phase.next() {
            match open_at(self.directory.file(), OsStr::new(next.filename()), false) {
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                _ => return Err(denied("later phase already exists or is unknown")),
            }
        }
        let capture = Capture::read(&self.client)?;
        let tree = TreeSnapshot::read(&self.client)?;
        let grok = GrokSnapshot::read(&self.profile)?;
        checkpoint()?;
        if Capture::read(&self.client)? != capture
            || TreeSnapshot::read(&self.client)? != tree
            || GrokSnapshot::read(&self.profile)? != grok
            || self.frame(prior)?.1 != previous
        {
            return Err(denied("snapshot changed before durable publication"));
        }
        tree.archive(&self.client, &self.directory)?;
        grok.archive(&self.profile, &self.directory)?;
        let frame = Frame {
            schema_version: if self.original_root_absence.is_some() {
                5
            } else {
                4
            },
            attempt: self.attempt.clone(),
            uid: unsafe { libc::geteuid() },
            data_root: self.data.identity().clone(),
            client_root: self.client.identity().clone(),
            phase,
            previous_sha256: Some(digest(&previous)),
            capture,
            tree,
            profile_root: self.profile.identity().clone(),
            grok,
            original_root_absence: self.original_root_absence.clone(),
        };
        let bytes = encode(&frame)?;
        create(&self.directory, phase.filename(), &bytes)?;
        self.known
            .lock()
            .map_err(|_| denied("phase digest retention unavailable"))?[phase.index()] =
            Some(digest(&bytes));
        if self.frame(phase)?.1 != bytes {
            return Err(denied("published phase changed"));
        }
        Ok(digest(&bytes))
    }
}

#[cfg(test)]
#[path = "macos_client_originals_tests.rs"]
mod tests;

// Narrow read-only projection for the exact official config writer lock. It
// shares the original descriptor/ACL/xattr checks without exposing arbitrary
// artifact capture or granting mutation authority to the lock adapter.
impl PinnedDirectory {
    pub(crate) fn config_write_lock_image(&self) -> io::Result<Image> {
        let image = capture_file(self, "config.toml.ocx-write.lock", 4096)?;
        if let Image::File { protection, .. } = &image {
            if protection.uid != unsafe { libc::geteuid() }
                || protection.mode != 0o600
                || protection.flags != 0
                || protection.acl.is_some()
                || protection
                    .xattrs
                    .iter()
                    .any(|(k, v)| k != b"com.apple.provenance" || v.len() > 64)
            {
                return Err(denied("unsafe official config write lock image"));
            }
        }
        Ok(image)
    }
}

#[cfg(test)]
#[path = "macos_client_root_originals_tests.rs"]
mod root_originals_tests;

#[cfg(test)]
#[path = "macos_client_descendant_pin_tests.rs"]
mod descendant_pin_tests;
