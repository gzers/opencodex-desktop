//! Read-only descriptor-relative client inventory. Ordinary file contents are
//! hashed in the inventory and archived separately by its owning frame. A native
//! coordination record binds
//! the exact root claim database's stat identity without opening/closing that
//! SQLite inode (which could release another connection's POSIX record lock).
//! An inventory alone is neither a byte archive nor a mutation permit.
use super::*;
use sha2::{Digest, Sha256};
use std::{ffi::OsString, os::unix::ffi::OsStringExt};
#[path = "macos_client_tree_archive.rs"]
mod archive;

const MAX_ENTRIES: usize = 16_384;
const MAX_BYTES: u64 = 128 * 1024 * 1024;
const MAX_FILE: u64 = 16 * 1024 * 1024;
const MAX_DEPTH: usize = 64;
const MAX_PROTECTION: usize = 4 * 1024 * 1024;
const MAX_PATHS: usize = 1024 * 1024;
const CLAIM: &str = ".opencodex-native-main.claim.sqlite";
fn coordination_path(path: &str) -> bool {
    path == CLAIM
        || ["-journal", "-wal", "-shm"]
            .iter()
            .any(|suffix| path.strip_prefix(CLAIM) == Some(*suffix))
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TreeSnapshot {
    pub(super) root: Identity,
    pub(super) root_protection: Protection,
    entries: Vec<Entry>,
    total_bytes: u64,
    coordination: bool,
}

/// Recorded content/identity projection only, never a native mutation permit.
#[derive(Clone, PartialEq, Eq)]
pub(super) struct RecordedFile {
    identity: Identity,
    pub(super) protection: Protection,
    bytes: u64,
    sha256: String,
}
impl RecordedFile {
    pub(super) fn matches(&self, image: &Image) -> bool {
        matches!(image, Image::File { identity, protection, bytes, .. }
            if *identity == self.identity && *protection == self.protection
                && bytes.len() as u64 == self.bytes && digest(bytes) == self.sha256)
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Entry {
    Directory {
        path: String,
        identity: Identity,
        protection: Protection,
    },
    File {
        path: String,
        identity: Identity,
        protection: Protection,
        bytes: u64,
        sha256: String,
    },
    // Metadata-only by construction. This is NOT an expanded cleanup exception
    // or permission to restore/delete a live database. No digest is invented.
    Coordination {
        path: String,
        identity: Identity,
        uid: u32,
        gid: u32,
        mode: u16,
        flags: u32,
        bytes: u64,
    },
}
impl Entry {
    fn path(&self) -> &str {
        match self {
            Self::Directory { path, .. }
            | Self::File { path, .. }
            | Self::Coordination { path, .. } => path,
        }
    }
}
impl std::fmt::Debug for TreeSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreeSnapshot")
            .field("entries", &self.entries.len())
            .field("total_bytes", &self.total_bytes)
            .finish()
    }
}
fn lookup(parent: &File, child: &OsStr) -> io::Result<libc::stat> {
    let c = name(child)?;
    let mut s = std::mem::MaybeUninit::uninit();
    if unsafe {
        libc::fstatat(
            parent.as_raw_fd(),
            c.as_ptr(),
            s.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    } != 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { s.assume_init() })
}
pub(in crate::infrastructure::macos_client_originals) fn names(
    parent: &File,
) -> io::Result<Vec<OsString>> {
    // openat(".") gives readdir an independent cursor for this exact directory;
    // a dup would share the original directory offset across observations.
    let raw = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            c".".as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if raw < 0 {
        return Err(io::Error::last_os_error());
    }
    let dir = unsafe { libc::fdopendir(raw) };
    if dir.is_null() {
        let e = io::Error::last_os_error();
        unsafe { libc::close(raw) };
        return Err(e);
    }
    struct Close(*mut libc::DIR);
    impl Drop for Close {
        fn drop(&mut self) {
            unsafe { libc::closedir(self.0) };
        }
    }
    let _close = Close(dir);
    let mut result = Vec::new();
    loop {
        unsafe { *libc::__error() = 0 };
        let item = unsafe { libc::readdir(dir) };
        if item.is_null() {
            let e = io::Error::last_os_error();
            if e.raw_os_error() != Some(0) {
                return Err(e);
            }
            break;
        }
        let item = unsafe { &*item };
        let n = item.d_namlen as usize;
        if n == 0 || n >= item.d_name.len() || item.d_name[n] != 0 {
            return Err(denied("tree directory entry framing unknown"));
        }
        let raw: Vec<u8> = item.d_name[..n].iter().map(|c| *c as u8).collect();
        if raw == b"." || raw == b".." {
            continue;
        }
        if result.len() >= MAX_ENTRIES {
            return Err(denied("client tree entry count over limit"));
        }
        let child = OsString::from_vec(raw);
        name(&child)?;
        result.push(child);
    }
    result.sort();
    if result.windows(2).any(|w| w[0] == w[1]) {
        return Err(denied("duplicate tree directory entry"));
    }
    Ok(result)
}
#[derive(Default)]
struct Budget {
    entries: usize,
    bytes: u64,
    security: usize,
    paths: usize,
}
impl Budget {
    fn entry(&mut self, path: &str) -> io::Result<()> {
        if path.len() > 4096 || self.entries >= MAX_ENTRIES {
            return Err(denied("client tree depth or entry budget exceeded"));
        }
        self.paths = self
            .paths
            .checked_add(path.len())
            .filter(|n| *n <= MAX_PATHS)
            .ok_or_else(|| denied("tree path budget exceeded"))?;
        self.entries += 1;
        Ok(())
    }
    fn protection(&mut self, p: &Protection) -> io::Result<()> {
        let size = p.acl.as_ref().map_or(0, Vec::len)
            + p.xattrs
                .iter()
                .map(|(k, v)| k.len() + v.len())
                .sum::<usize>();
        self.security = self
            .security
            .checked_add(size)
            .filter(|n| *n <= MAX_PROTECTION)
            .ok_or_else(|| denied("tree protection budget exceeded"))?;
        Ok(())
    }
}
fn stable(
    parent: &File,
    child: &OsStr,
    file: &File,
    before: &libc::stat,
    p: &Protection,
) -> io::Result<()> {
    let after = stat(file)?;
    let current = lookup(parent, child)?;
    if version(before) != version(&after)
        || version(&after) != version(&current)
        || protection(file, &after)? != *p
    {
        return Err(denied("client tree object changed during observation"));
    }
    Ok(())
}
fn walk(
    parent: &File,
    relative: &str,
    depth: usize,
    entries: &mut Vec<Entry>,
    budget: &mut Budget,
    checkpoint: &impl Fn(&str) -> io::Result<()>,
    coordination: bool,
) -> io::Result<()> {
    if depth > MAX_DEPTH {
        return Err(denied("client tree depth exceeded"));
    }
    let before = stat(parent)?;
    let security = protection(parent, &before)?;
    if before.st_uid != unsafe { libc::geteuid() } || before.st_mode & 0o022 != 0 {
        return Err(denied("tree directory owner or write access unknown"));
    }
    let children = names(parent)?;
    for child in &children {
        let text = child
            .to_str()
            .ok_or_else(|| denied("client tree path is not UTF8"))?;
        let path = if relative.is_empty() {
            text.into()
        } else {
            format!("{relative}/{text}")
        };
        // Literal backslashes on POSIX stay literal; they are never rewritten
        // into '/' and cannot collide with a different descendant path.
        budget.entry(&path)?;
        let observed = lookup(parent, child)?;
        if coordination && relative.is_empty() && coordination_path(text) {
            // First-use BEGIN EXCLUSIVE on an empty official claim creates a
            // real rollback journal. Record exact sidecar identity as native
            // coordination too; never open its inode or mistake it for bytes
            // this engine may restore. An orphan sidecar stays refused.
            if text != CLAIM {
                let claim = lookup(parent, OsStr::new(CLAIM))?;
                if claim.st_mode & libc::S_IFMT != libc::S_IFREG
                    || claim.st_nlink != 1
                    || claim.st_uid != unsafe { libc::geteuid() }
                    || claim.st_mode & 0o777 != 0o600
                    || claim.st_flags != 0
                {
                    return Err(denied("tree coordination parent unknown"));
                }
            }
            if observed.st_mode & libc::S_IFMT != libc::S_IFREG
                || observed.st_nlink != 1
                || observed.st_uid != unsafe { libc::geteuid() }
                || observed.st_mode & 0o777 != 0o600
                || observed.st_flags != 0
                || observed.st_size < 0
            {
                return Err(denied("tree coordination identity unavailable"));
            }
            checkpoint(&path)?;
            if version(&observed) != version(&lookup(parent, child)?) {
                return Err(denied("tree coordination identity changed"));
            }
            entries.push(Entry::Coordination {
                path,
                identity: identity(&observed),
                uid: observed.st_uid,
                gid: observed.st_gid,
                mode: observed.st_mode & 0o777,
                flags: observed.st_flags,
                bytes: observed.st_size as u64,
            });
            continue;
        }
        let directory = match observed.st_mode & libc::S_IFMT {
            libc::S_IFDIR => true,
            libc::S_IFREG => false,
            _ => return Err(denied("tree link or special object refused")),
        };
        // Reject an already linked file BEFORE opening its inode. In
        // particular an alias to the SQLite claim cannot cause this observer
        // to close another fd for an actively locked coordination inode.
        if !directory && observed.st_nlink != 1 {
            return Err(denied("tree linked source refused before open"));
        }
        checkpoint(&path)?;
        let file = open_at(parent, child, directory)?;
        let opened = stat(&file)?;
        if version(&observed) != version(&opened) || opened.st_uid != unsafe { libc::geteuid() } {
            return Err(denied("tree object replaced before open"));
        }
        let p = protection(&file, &opened)?;
        budget.protection(&p)?;
        if directory {
            walk(
                &file,
                &path,
                depth + 1,
                entries,
                budget,
                checkpoint,
                coordination,
            )?;
            stable(parent, child, &file, &opened, &p)?;
            entries.push(Entry::Directory {
                path,
                identity: identity(&opened),
                protection: p,
            });
        } else {
            let bytes =
                u64::try_from(opened.st_size).map_err(|_| denied("tree file size invalid"))?;
            if bytes > MAX_FILE {
                return Err(denied("tree file over limit"));
            }
            budget.bytes = budget
                .bytes
                .checked_add(bytes)
                .filter(|n| *n <= MAX_BYTES)
                .ok_or_else(|| denied("tree bytes over limit"))?;
            let mut hash = Sha256::new();
            let mut buffer = zeroize::Zeroizing::new(vec![0; 64 * 1024]);
            let mut count = 0u64;
            let mut reader = &file;
            loop {
                let n = reader.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                count += n as u64;
                if count > bytes {
                    return Err(denied("tree file grew during read"));
                }
                hash.update(&buffer[..n]);
            }
            if count != bytes {
                return Err(denied("tree file truncated during read"));
            }
            checkpoint(&path)?;
            stable(parent, child, &file, &opened, &p)?;
            entries.push(Entry::File {
                path,
                identity: identity(&opened),
                protection: p,
                bytes,
                sha256: format!("{:x}", hash.finalize()),
            });
        }
    }
    if names(parent)? != children
        || version(&stat(parent)?) != version(&before)
        || protection(parent, &stat(parent)?)? != security
    {
        return Err(denied("client tree directory changed during capture"));
    }
    Ok(())
}
impl TreeSnapshot {
    pub(super) fn recorded_file(&self, path: &str) -> io::Result<Option<RecordedFile>> {
        match self.entries.iter().find(|entry| entry.path() == path) {
            None => Ok(None),
            Some(Entry::File {
                identity,
                protection,
                bytes,
                sha256,
                ..
            }) => Ok(Some(RecordedFile {
                identity: identity.clone(),
                protection: protection.clone(),
                bytes: *bytes,
                sha256: sha256.clone(),
            })),
            _ => Err(denied("recorded target is not a regular file")),
        }
    }
    /// Only the exact three plan targets/stages are separately governed by
    /// Pending's native-object review. Everything else keeps the ordinary full
    /// readback rules. A live claim journal is ignored only inside a checked
    /// same-process NativeLease; all other sidecars and foreign stages refuse.
    pub(super) fn recovery_remainder_matches(
        &self,
        current: &Self,
        plan: &str,
        leased: bool,
    ) -> bool {
        self.recovery_remainder_inner(current, Some(plan), leased, false)
    }
    pub(super) fn preplan_remainder_matches(&self, current: &Self, leased: bool) -> bool {
        self.recovery_remainder_inner(current, None, leased, false)
    }
    /// Only Pending may select this projection after admitting an explicit
    /// fourth artifact. Its native object review owns that exact file.
    pub(super) fn remainder_with_empty_config(&self, current: &Self, leased: bool) -> bool {
        self.recovery_remainder_inner(current, None, leased, true)
    }
    fn recovery_remainder_inner(
        &self,
        current: &Self,
        plan: Option<&str>,
        leased: bool,
        empty_config: bool,
    ) -> bool {
        if plan.is_some_and(|plan| {
            uuid::Uuid::parse_str(plan)
                .map(|p| p.to_string() != plan)
                .unwrap_or(true)
        }) || !self.valid()
            || !current.valid()
        {
            return false;
        }
        let stages: Vec<_> = plan
            .into_iter()
            .flat_map(|plan| (0..3).map(move |i| format!(".ocxd-restore-{plan}-{i}")))
            .collect();
        let filtered = |tree: &Self| {
            let mut tree = tree.clone();
            tree.entries.retain(|entry| {
                let path = entry.path();
                !(NAMES.contains(&path)
                    || empty_config && path == "config.toml"
                    || stages.iter().any(|s| s == path)
                    || leased
                        && matches!(entry, Entry::Coordination { path, .. }
                        if path == ".opencodex-native-main.claim.sqlite-journal"))
            });
            tree.total_bytes = tree
                .entries
                .iter()
                .map(|entry| match entry {
                    Entry::File { bytes, .. } => *bytes,
                    _ => 0,
                })
                .sum();
            tree
        };
        filtered(self).restored_matches(&filtered(current), true)
    }
    /// Compare restored content and ALL security attributes, not rename-driven
    /// inode/timestamp changes. Only the two adopted top-level stable lock names
    /// may differ; their native release/identity proof is owned separately.
    pub(super) fn restored_matches(&self, current: &Self, codex: bool) -> bool {
        if !self.valid()
            || !current.valid()
            || self.root != current.root
            || self.root_protection != current.root_protection
            || self.coordination != codex
            || current.coordination != codex
        {
            return false;
        }
        let stable = |path: &str| {
            codex
                && matches!(
                    path,
                    ".opencodex-native-main.claim.sqlite" | ".opencodex-native-main.owner.sqlite"
                )
        };
        // An original claim journal observed inside the native transaction is
        // metadata-only, never user content. Final readback requires it absent,
        // together with ALL other SQLite sidecars; no sidecar may be retained.
        if current.entries.iter().any(|entry| {
            matches!(entry, Entry::Coordination{path,..} if !stable(path))
                || codex
                    && ["-journal", "-wal", "-shm"].iter().any(|suffix| {
                        entry
                            .path()
                            .strip_prefix(".opencodex-native-main.owner.sqlite")
                            == Some(*suffix)
                    })
        }) {
            return false;
        }
        let include = |entry: &&Entry| {
            !(stable(entry.path()) || codex && matches!(entry, Entry::Coordination { .. }))
        };
        let left: Vec<_> = self.entries.iter().filter(include).collect();
        let right: Vec<_> = current.entries.iter().filter(include).collect();
        left.len() == right.len()
            && left.into_iter().zip(right).all(|(a, b)| match (a, b) {
                (
                    Entry::Directory {
                        path: a,
                        identity: ai,
                        protection: ap,
                    },
                    Entry::Directory {
                        path: b,
                        identity: bi,
                        protection: bp,
                    },
                ) => a == b && ai == bi && ap == bp,
                (
                    Entry::File {
                        path: a,
                        protection: ap,
                        bytes: ab,
                        sha256: ah,
                        ..
                    },
                    Entry::File {
                        path: b,
                        protection: bp,
                        bytes: bb,
                        sha256: bh,
                        ..
                    },
                ) => a == b && ap == bp && ab == bb && ah == bh,
                _ => false,
            })
    }
    pub(super) fn stable_identity(&self, path: &str) -> io::Result<Identity> {
        if !self.coordination || !matches!(path, CLAIM | ".opencodex-native-main.owner.sqlite") {
            return Err(denied("not an adopted Codex stable lock"));
        }
        match self.entries.iter().find(|entry| entry.path() == path) {
            Some(Entry::Coordination { identity, .. }) | Some(Entry::File { identity, .. }) => {
                Ok(identity.clone())
            }
            _ => Err(denied("stable lock not independently captured")),
        }
    }
    pub(super) fn optional_stable_identity(&self, path: &str) -> io::Result<Option<Identity>> {
        if !self.valid()
            || !self.coordination
            || !matches!(path, CLAIM | ".opencodex-native-main.owner.sqlite")
        {
            return Err(denied("invalid optional stable lock inventory"));
        }
        if self.entries.iter().any(|e| e.path() == path) {
            self.stable_identity(path).map(Some)
        } else {
            Ok(None)
        }
    }
    pub(super) fn original_stable_identity_matches(&self, path: &str, expected: &Identity) -> bool {
        match self.entries.iter().find(|entry| entry.path() == path) {
            None => true,
            Some(Entry::Coordination { identity, .. }) | Some(Entry::File { identity, .. }) => {
                identity == expected
            }
            _ => false,
        }
    }
    pub fn read(root: &PinnedDirectory) -> io::Result<Self> {
        Self::read_checked(root, &|_| Ok(()))
    }
    fn read_checked(
        root: &PinnedDirectory,
        checkpoint: &impl Fn(&str) -> io::Result<()>,
    ) -> io::Result<Self> {
        Self::read_mode_checked(root, true, checkpoint)
    }
    pub(crate) fn read_mode(root: &PinnedDirectory, coordination: bool) -> io::Result<Self> {
        Self::read_mode_checked(root, coordination, &|_| Ok(()))
    }
    fn read_mode_checked(
        root: &PinnedDirectory,
        coordination: bool,
        checkpoint: &impl Fn(&str) -> io::Result<()>,
    ) -> io::Result<Self> {
        let one = || -> io::Result<Self> {
            root.recheck()?;
            let mut entries = Vec::new();
            let mut b = Budget::default();
            let s = stat(root.file())?;
            let p = protection(root.file(), &s)?;
            b.protection(&p)?;
            walk(
                root.file(),
                "",
                0,
                &mut entries,
                &mut b,
                checkpoint,
                coordination,
            )?;
            entries.sort_by(|a, b| a.path().cmp(b.path()));
            root.recheck()?;
            Ok(Self {
                root: identity(&s),
                root_protection: p,
                entries,
                total_bytes: b.bytes,
                coordination,
            })
        };
        let before = one()?;
        let after = one()?;
        if before != after {
            return Err(denied("complete tree changed between observations"));
        }
        if !after.valid() {
            return Err(denied("captured tree inventory invalid"));
        }
        Ok(after)
    }
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
    pub(super) fn coordinated(&self) -> bool {
        self.coordination
    }
    pub(super) fn matches_artifacts(&self, capture: &Capture) -> bool {
        for (name, image) in NAMES.iter().zip(capture.images()) {
            let entry = self.entries.iter().find(|entry| entry.path() == *name);
            let same = match (image, entry) {
                (Image::Absent, None) => true,
                (
                    Image::File {
                        identity,
                        protection,
                        bytes,
                        ..
                    },
                    Some(Entry::File {
                        identity: id,
                        protection: p,
                        bytes: length,
                        sha256,
                        ..
                    }),
                ) => {
                    identity == id
                        && protection == p
                        && *length == bytes.len() as u64
                        && sha256 == &digest(bytes)
                }
                _ => false,
            };
            if !same {
                return false;
            }
        }
        true
    }
    pub(super) fn valid(&self) -> bool {
        let mut budget = Budget::default();
        if self.root_protection.uid != unsafe { libc::geteuid() }
            || self.root_protection.mode & 0o022 != 0
        {
            return false;
        }
        if budget.protection(&self.root_protection).is_err() {
            return false;
        }
        let mut known = std::collections::BTreeSet::new();
        let mut directories = std::collections::BTreeSet::new();
        let mut last = None;
        for e in &self.entries {
            let path = e.path();
            if (self.coordination && coordination_path(path))
                != matches!(e, Entry::Coordination { .. })
            {
                return false;
            }
            let parts: Vec<_> = path.split('/').collect();
            if path.is_empty()
                || parts.len() > MAX_DEPTH + 1
                || parts
                    .iter()
                    .any(|x| x.is_empty() || *x == "." || *x == ".." || x.contains('\0'))
                || last.is_some_and(|previous: &str| previous >= path)
                || budget.entry(path).is_err()
            {
                return false;
            }
            if let Some((parent, _)) = path.rsplit_once('/') {
                if !directories.contains(parent) {
                    return false;
                }
            }
            if !known.insert(path) {
                return false;
            }
            last = Some(path);
            match e {
                Entry::Directory { protection, .. } => {
                    if protection.uid != unsafe { libc::geteuid() }
                        || protection.mode & 0o022 != 0
                        || budget.protection(protection).is_err()
                    {
                        return false;
                    }
                    directories.insert(path);
                }
                Entry::File {
                    protection,
                    bytes,
                    sha256,
                    ..
                } => {
                    if protection.uid != unsafe { libc::geteuid() }
                        || *bytes > MAX_FILE
                        || sha256.len() != 64
                        || !sha256
                            .bytes()
                            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                        || budget.protection(protection).is_err()
                    {
                        return false;
                    }
                    let Some(total) = budget.bytes.checked_add(*bytes).filter(|n| *n <= MAX_BYTES)
                    else {
                        return false;
                    };
                    budget.bytes = total;
                }
                Entry::Coordination {
                    path,
                    uid,
                    mode,
                    flags,
                    ..
                } => {
                    if !self.coordination
                        || !coordination_path(path)
                        || path != CLAIM && !known.contains(CLAIM)
                        || *uid != unsafe { libc::geteuid() }
                        || *mode != 0o600
                        || *flags != 0
                    {
                        return false;
                    }
                }
            }
        }
        budget.bytes == self.total_bytes
    }
}

#[cfg(test)]
#[path = "macos_client_tree_tests.rs"]
mod tests;
