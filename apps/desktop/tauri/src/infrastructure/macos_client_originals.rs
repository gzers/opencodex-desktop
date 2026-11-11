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
        let current = Self::open(&self.path)?;
        if current.chain.len() != self.chain.len()
            || current.chain.iter().zip(&self.chain).any(
                |((file, id, observed), (original, expected, original_protection))| {
                    id != expected
                        || observed != original_protection
                        || stat(file)
                            .map(|s| {
                                identity(&s) != *expected
                                    || protection(file, &s)
                                        .map(|p| p != *original_protection)
                                        .unwrap_or(true)
                            })
                            .unwrap_or(true)
                        || stat(original)
                            .map(|s| {
                                identity(&s) != *expected
                                    || protection(original, &s)
                                        .map(|p| p != *original_protection)
                                        .unwrap_or(true)
                            })
                            .unwrap_or(true)
                },
            )
        {
            return Err(denied("directory generation changed"));
        }
        Ok(())
    }
    pub(crate) fn private(&self) -> io::Result<()> {
        self.recheck()?;
        require_private(&protection(self.file(), &stat(self.file())?)?, true)
    }
    fn child_private(&self, child: &str, create: bool) -> io::Result<Self> {
        self.private()?;
        let c = name(OsStr::new(child))?;
        if create && unsafe { libc::mkdirat(self.file().as_raw_fd(), c.as_ptr(), 0o700) } != 0 {
            let error = io::Error::last_os_error();
            if error.kind() != io::ErrorKind::AlreadyExists {
                return Err(error);
            }
        }
        let result = Self::open(&self.path.join(child))?;
        result.private()?;
        self.recheck()?;
        Ok(result)
    }
    fn sync(&self) -> io::Result<()> {
        self.recheck()?;
        self.file().sync_all()
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
    serde_json::from_value(
        parse_strict_json_v2(bytes).map_err(|_| denied("invalid or duplicate JSON"))?,
    )
    .map_err(|_| denied("invalid frame fields"))
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
pub struct Originals {
    data: PinnedDirectory,
    client: PinnedDirectory,
    directory: PinnedDirectory,
    attempt: String,
    original_sha256: String,
    known: std::sync::Mutex<[Option<String>; 4]>,
}
impl Originals {
    pub fn begin(data_root: &Path, client_root: &Path) -> io::Result<Self> {
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
        let capture = Capture::read(&client)?;
        let manager = data.child_private("manager-state", true)?;
        let vault = manager.child_private("client-originals", true)?;
        let attempt = uuid::Uuid::new_v4().to_string();
        let c = name(OsStr::new(&attempt))?;
        if unsafe { libc::mkdirat(vault.file().as_raw_fd(), c.as_ptr(), 0o700) } != 0 {
            return Err(io::Error::last_os_error());
        }
        let directory = PinnedDirectory::open(&vault.path.join(&attempt))?;
        directory.private()?;
        vault.sync()?;
        let frame = Frame {
            schema_version: 1,
            attempt: attempt.clone(),
            uid: unsafe { libc::geteuid() },
            data_root: data.identity().clone(),
            client_root: client.identity().clone(),
            phase: Phase::Original,
            previous_sha256: None,
            capture,
        };
        if Capture::read(&client)? != frame.capture {
            return Err(denied("source changed before original publication"));
        }
        let bytes = encode(&frame)?;
        create(&directory, Phase::Original.filename(), &bytes)?;
        data.recheck()?;
        client.recheck()?;
        if Capture::read(&client)? != frame.capture {
            return Err(denied(
                "source changed during original publication; original retained",
            ));
        }
        let hash = digest(&bytes);
        Ok(Self {
            data,
            client,
            directory,
            attempt,
            original_sha256: hash.clone(),
            known: std::sync::Mutex::new([Some(hash), None, None, None]),
        })
    }
    pub fn open(
        data_root: &Path,
        client_root: &Path,
        attempt: &str,
        original_sha256: &str,
        expected_phases: &[(Phase, String)],
    ) -> io::Result<Self> {
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
        let directory = data
            .child_private("manager-state", false)?
            .child_private("client-originals", false)?
            .child_private(attempt, false)?;
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
            data,
            client,
            directory,
            attempt: attempt.into(),
            original_sha256: original_sha256.into(),
            known: std::sync::Mutex::new(known),
        };
        result.frame(Phase::Original)?;
        for (phase, _) in expected_phases {
            result.frame(*phase)?;
        }
        Ok(result)
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
    fn frame(&self, phase: Phase) -> io::Result<(Frame, Vec<u8>)> {
        self.data.private()?;
        self.client.recheck()?;
        self.directory.private()?;
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
        if frame.schema_version != 1
            || frame.attempt != self.attempt
            || frame.uid != unsafe { libc::geteuid() }
            || frame.phase != phase
            || frame.data_root != *self.data.identity()
            || frame.client_root != *self.client.identity()
            || frame.capture.root != frame.client_root
            || frame.capture.root_protection
                != protection(self.client.file(), &stat(self.client.file())?)?
            || !frame.capture.valid()
        {
            return Err(denied("journal context or snapshot changed"));
        }
        if let Some(prior) = phase.predecessor() {
            let (_, previous) = self.frame(prior)?;
            if frame.previous_sha256.as_deref() != Some(digest(&previous).as_str()) {
                return Err(denied("journal predecessor changed"));
            }
        } else if frame.previous_sha256.is_some() || digest(&bytes) != self.original_sha256 {
            return Err(denied("original snapshot digest changed"));
        }
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
        checkpoint()?;
        if Capture::read(&self.client)? != capture || self.frame(prior)?.1 != previous {
            return Err(denied("snapshot changed before durable publication"));
        }
        let frame = Frame {
            schema_version: 1,
            attempt: self.attempt.clone(),
            uid: unsafe { libc::geteuid() },
            data_root: self.data.identity().clone(),
            client_root: self.client.identity().clone(),
            phase,
            previous_sha256: Some(digest(&previous)),
            capture,
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
