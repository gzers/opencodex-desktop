//! Windows original rollback-SQLite kernel locks. No production SQLite open,
//! database creation/recovery, schema write or helper-held ownership receipt.
//! This primitive alone does not authorize official client mutations or moves.
use super::{
    bounded::{native_stamp, SnapshotData},
    tree,
};
use crate::{
    infrastructure::{
        hash,
        windows_official_lifecycle024::Owner,
        windows_official_runtime024::{
            self as native, FileIdentity, PinnedDirectory, TokenIdentity,
        },
    },
    modules::runtime::windows_lifecycle024_contract::NativeSession,
};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom},
    marker::PhantomData,
    os::windows::{
        fs::OpenOptionsExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::{Path, PathBuf},
    time::Instant,
};
use windows_sys::Win32::{
    Security::TOKEN_QUERY,
    Storage::FileSystem::{
        GetFileInformationByHandle, GetFileType, LockFileEx, UnlockFileEx,
        BY_HANDLE_FILE_INFORMATION, FILE_TYPE_DISK, LOCKFILE_EXCLUSIVE_LOCK,
        LOCKFILE_FAIL_IMMEDIATELY,
    },
    System::{
        Threading::{GetCurrentProcess, OpenProcessToken},
        IO::OVERLAPPED,
    },
};
mod preparation;
mod segments;
pub(super) use preparation::LeasePreparation;
pub(super) use segments::RetiredLeaseHistory;
pub(super) use segments::{LeaseFacts, ReleasedLease};
const BASE: u32 = 0x40000000;
const MAX: u64 = 2 * 1024 * 1024;
fn refused() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "original Windows coordination context/file/rollback lock unconfirmed",
    )
}
fn before(end: Instant) -> io::Result<()> {
    if Instant::now() >= end {
        Err(io::ErrorKind::TimedOut.into())
    } else {
        Ok(())
    }
}
fn io_error(e: impl std::fmt::Display) -> io::Error {
    io::Error::other(e.to_string())
}
fn file_id(file: &File) -> io::Result<FileIdentity> {
    let mut data: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileType(file.as_raw_handle()) } != FILE_TYPE_DISK
        || unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut data) } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if data.dwFileAttributes & (0x400 | 0x10) != 0 || data.nNumberOfLinks != 1 {
        return Err(refused());
    }
    Ok(FileIdentity {
        volume: data.dwVolumeSerialNumber,
        index: ((data.nFileIndexHigh as u64) << 32) | data.nFileIndexLow as u64,
    })
}
fn open(path: &Path) -> io::Result<File> {
    native::validate_local_path(path)?;
    // Sharing WRITE permits official SQLite readers/writers to open, while
    // actual kernel byte locks control exclusion. Never share DELETE/reparse.
    OpenOptions::new()
        .access_mode(0x80020000)
        .share_mode(1 | 2)
        .custom_flags(0x200000)
        .open(path)
}
fn snapshot(file: &mut File, end: Instant) -> io::Result<Vec<u8>> {
    before(end)?;
    let len = file.metadata()?.len();
    if len > MAX {
        return Err(refused());
    }
    file.seek(SeekFrom::Start(0))?;
    let mut raw = Vec::new();
    file.take(MAX + 1).read_to_end(&mut raw)?;
    if raw.len() as u64 != len {
        return Err(refused());
    }
    validate_stable_bytes(&raw)?;
    before(end)?;
    Ok(raw)
}
pub(super) fn validate_stable_bytes(raw: &[u8]) -> io::Result<()> {
    if raw.len() > MAX as usize {
        return Err(refused());
    }
    if !raw.is_empty() {
        if raw.len() < 100 || &raw[..16] != b"SQLite format 3\0" || raw[18] != 1 || raw[19] != 1 {
            return Err(refused());
        }
        let page = u16::from_be_bytes([raw[16], raw[17]]) as u32;
        if page != 1 && (!(512..=32768).contains(&page) || !page.is_power_of_two()) {
            return Err(refused());
        }
    }
    Ok(())
}
fn no_sidecars(path: &Path) -> io::Result<()> {
    for suffix in ["-journal", "-wal", "-shm"] {
        let path = PathBuf::from(format!("{}{suffix}", path.to_str().ok_or_else(refused)?));
        match std::fs::symlink_metadata(path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            _ => return Err(refused()),
        }
    }
    Ok(())
}
struct LockFile {
    file: File,
    parent: ParentPin,
    path: PathBuf,
    id: FileIdentity,
    security: SnapshotData,
    raw: Vec<u8>,
}
enum ParentPin {
    Strict(PinnedDirectory),
    Claim(ClaimParent),
}
impl ParentPin {
    fn check(&self, end: Instant) -> io::Result<()> {
        match self {
            Self::Strict(pin) => pin.recheck(),
            Self::Claim(pin) => pin.check(end),
        }
    }
}
/// Metadata-only original root observation; RootTree owns DELETE and movement.
/// An external parent pin still prevents ancestor substitution. This handle
/// cannot enumerate/mutate/rename and contains no persisted path authority.
struct ClaimParent {
    path: PathBuf,
    file: File,
    ancestor: PinnedDirectory,
    id: FileIdentity,
    security: SnapshotData,
}
impl ClaimParent {
    fn open(path: &Path) -> io::Result<File> {
        native::validate_local_path(path)?;
        OpenOptions::new()
            .access_mode(0x20080)
            .share_mode(1 | 2 | 4)
            .custom_flags(0x2200000)
            .open(path)
    }
    fn identity(file: &File) -> io::Result<FileIdentity> {
        let mut data: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut data) } == 0 {
            return Err(io::Error::last_os_error());
        }
        if data.dwFileAttributes & 0x400 != 0 || data.dwFileAttributes & 0x10 == 0 {
            return Err(refused());
        }
        Ok(FileIdentity {
            volume: data.dwVolumeSerialNumber,
            index: ((data.nFileIndexHigh as u64) << 32) | data.nFileIndexLow as u64,
        })
    }
    fn capture(path: &Path, end: Instant) -> io::Result<Self> {
        before(end)?;
        let ancestor = PinnedDirectory::open(path.parent().ok_or_else(refused)?)?;
        let file = Self::open(path)?;
        native::verify_private_directory(&file)?;
        let id = Self::identity(&file)?;
        let security = native_stamp(&file, end).map_err(io_error)?;
        let out = Self {
            path: path.into(),
            file,
            ancestor,
            id,
            security,
        };
        out.check(end)?;
        Ok(out)
    }
    fn check(&self, end: Instant) -> io::Result<()> {
        before(end)?;
        self.ancestor.recheck()?;
        let current = Self::open(&self.path)?;
        native::verify_private_directory(&self.file)?;
        native::verify_private_directory(&current)?;
        if Self::identity(&self.file)? != self.id
            || Self::identity(&current)? != self.id
            || native_stamp(&self.file, end).map_err(io_error)? != self.security
            || native_stamp(&current, end).map_err(io_error)? != self.security
        {
            return Err(refused());
        }
        self.ancestor.recheck()?;
        before(end)
    }
}
impl LockFile {
    fn capture(path: &Path, end: Instant, claim: bool) -> io::Result<Self> {
        before(end)?;
        no_sidecars(path)?;
        let directory = path.parent().ok_or_else(refused)?;
        let parent = if claim {
            ParentPin::Claim(ClaimParent::capture(directory, end)?)
        } else {
            ParentPin::Strict(PinnedDirectory::open_private(directory)?)
        };
        // The share-delete experiment did not permit actual directory rename.
        // Preserve the original strict file sharing; claim handoff remains an
        // explicit unresolved plan, never an automatic close/reopen fallback.
        let mut file = open(path)?;
        native::verify_private_file(&file)?;
        tree::streams(path, false)?;
        let id = file_id(&file)?;
        let security = native_stamp(&file, end).map_err(io_error)?;
        let raw = snapshot(&mut file, end)?;
        let mut out = Self {
            file,
            parent,
            path: path.into(),
            id,
            security,
            raw,
        };
        out.check(end)?;
        Ok(out)
    }
    fn check(&mut self, end: Instant) -> io::Result<()> {
        before(end)?;
        self.parent.check(end)?;
        no_sidecars(&self.path)?;
        native::verify_private_file(&self.file)?;
        let current = open(&self.path)?;
        tree::streams(&self.path, false)?;
        if file_id(&self.file)? != self.id
            || file_id(&current)? != self.id
            || native_stamp(&self.file, end).map_err(io_error)? != self.security
            || native_stamp(&current, end).map_err(io_error)? != self.security
            || snapshot(&mut self.file, end)? != self.raw
        {
            return Err(refused());
        }
        before(end)
    }
}
struct Range {
    offset: u32,
    length: u32,
    overlap: OVERLAPPED,
}
struct Held {
    pin: LockFile,
    ranges: Vec<Range>,
}
impl Held {
    fn range(&mut self, offset: u32, length: u32, exclusive: bool) -> io::Result<()> {
        let mut range = Range {
            offset,
            length,
            overlap: unsafe { std::mem::zeroed() },
        };
        range.overlap.Anonymous.Anonymous.Offset = offset;
        let flags = LOCKFILE_FAIL_IMMEDIATELY
            | if exclusive {
                LOCKFILE_EXCLUSIVE_LOCK
            } else {
                0
            };
        if unsafe {
            LockFileEx(
                self.pin.file.as_raw_handle(),
                flags,
                0,
                length,
                0,
                &mut range.overlap,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        self.ranges.push(range);
        Ok(())
    }
    fn unlock(&mut self, offset: u32) -> io::Result<()> {
        let index = self
            .ranges
            .iter()
            .position(|r| r.offset == offset)
            .ok_or_else(refused)?;
        let range = &mut self.ranges[index];
        if unsafe {
            UnlockFileEx(
                self.pin.file.as_raw_handle(),
                0,
                range.length,
                0,
                &mut range.overlap,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        self.ranges.remove(index);
        Ok(())
    }
    fn release(&mut self) -> io::Result<()> {
        let mut error = None;
        while let Some(range) = self.ranges.last() {
            let offset = range.offset;
            if let Err(e) = self.unlock(offset) {
                error = Some(e);
                break;
            }
        }
        if let Some(e) = error {
            Err(e)
        } else {
            Ok(())
        }
    }
    fn acquire(pin: LockFile, claim: bool, end: Instant) -> io::Result<Self> {
        before(end)?;
        let mut held = Self {
            pin,
            ranges: Vec::new(),
        };
        let result = (|| {
            if claim {
                held.range(BASE, 512, true)?;
            } else {
                // Original SQLite Windows VFS SHARED admission and RESERVED writer
                // exclusion; no BEGIN/PRAGMA or writes through a SQLite connection.
                held.range(BASE, 1, false)?;
                held.range(BASE + 2, 510, false)?;
                held.unlock(BASE)?;
                held.range(BASE + 1, 1, true)?;
            }
            held.pin.check(end)
        })();
        if let Err(error) = result {
            // Before enrollment in NativeLease, explicitly attempt each owned
            // partial range. Closing this own file on a failed unlock remains
            // failure cleanup, never a successful ordinary release receipt.
            let _ = held.release();
            return Err(error);
        }
        Ok(held)
    }
}
// NativeLease owns release for every enrolled Held. A Held is never exposed
// independently; failed acquisition above releases its own incomplete ranges.
struct Bound {
    scope: String,
    profile: PinnedDirectory,
    home: PinnedDirectory,
    client: PinnedDirectory,
}
/// Actual retained file handles and OS ranges, stack-scoped and not Send/Sync.
/// No successful constructor from JSON, PID, bool or file bytes is exposed.
pub(super) struct NativeLease {
    held: Vec<Held>,
    token: TokenIdentity,
    deadline: Instant,
    active: bool,
    bound: Option<Bound>,
    _not_send: PhantomData<*mut ()>,
    epoch: String,
}
/// Borrowed exclusion for one anchored critical section. Cannot outlive or
/// release the actual lease. Persisted epoch/key are locators, never permits.
pub(super) struct AnchoredClaim<'a> {
    lease: &'a mut NativeLease,
    path: PathBuf,
    id: FileIdentity,
    end: Instant,
}
impl AnchoredClaim<'_> {
    pub(super) fn check(&mut self, root: &Path, end: Instant) -> io::Result<()> {
        if end > self.end || self.path.parent() != Some(root) {
            self.lease.active = false;
            return Err(refused());
        }
        self.lease.check(end)?;
        let held = &self.lease.held[0];
        if held.pin.path != self.path
            || held.pin.id != self.id
            || held.ranges.len() != 1
            || held.ranges[0].offset != BASE
            || held.ranges[0].length != 512
        {
            self.lease.active = false;
            return Err(refused());
        }
        Ok(())
    }
    pub(super) fn epoch(&self) -> &str {
        &self.lease.epoch
    }
    pub(super) fn key(&self) -> String {
        hash::sha256_hex(
            format!(
                "{}:{}:{}",
                self.path.display(),
                self.id.volume,
                self.id.index
            )
            .as_bytes(),
        )
    }
}
impl NativeLease {
    pub(super) fn revoke_retained(&mut self) {
        self.active = false;
    }
    #[cfg(test)]
    pub(super) fn acquire_synthetic(paths: &[PathBuf; 4], end: Instant) -> io::Result<Self> {
        Self::acquire_paths(paths, end)
    }
    pub(super) fn anchored_claim(
        &mut self,
        root: &Path,
        end: Instant,
    ) -> io::Result<AnchoredClaim<'_>> {
        self.check(end)?;
        let pin = &self.held[0].pin;
        if pin.path != root.join(".opencodex-native-main.claim.sqlite") {
            self.active = false;
            return Err(refused());
        }
        let path = pin.path.clone();
        let id = pin.id;
        let mut claim = AnchoredClaim {
            lease: self,
            path,
            id,
            end,
        };
        claim.check(root, end)?;
        Ok(claim)
    }
    fn acquire_paths(paths: &[PathBuf; 4], end: Instant) -> io::Result<Self> {
        before(end)?;
        let token = native::current_identity()?;
        if token.elevated || token.elevation_type == 2 {
            return Err(refused());
        }
        if paths
            .iter()
            .enumerate()
            .any(|(i, p)| paths[..i].iter().any(|other| other == p))
        {
            return Err(refused());
        }
        let mut lease = Self {
            held: Vec::new(),
            token,
            deadline: end,
            active: true,
            bound: None,
            _not_send: PhantomData,
            epoch: uuid::Uuid::new_v4().to_string(),
        };
        for (i, path) in paths.iter().enumerate() {
            let pin = LockFile::capture(path, end, i == 0)?;
            lease.held.push(Held::acquire(pin, i == 0, end)?);
            for held in &mut lease.held {
                held.pin.check(end)?;
            }
        }
        lease.check(end)?;
        Ok(lease)
    }
    /// Existing original files only. Initial official admission/owner receipts,
    /// relocation handoff and post-write compensation integration remain gated.
    pub(super) fn acquire_existing(owner: &mut Owner, end: Instant) -> io::Result<Self> {
        let (paths, bound) = existing_context(owner, end)?;
        let mut lease = Self::acquire_paths(&paths, end)?;
        lease.bound = Some(bound);
        lease.check_owner(owner, end)?;
        Ok(lease)
    }
}
// Observes the original live Owner/Profile/H/claim location. It holds no ranges
// and grants no mutation; both acquisition paths still perform real native IO.
fn existing_context(owner: &mut Owner, end: Instant) -> io::Result<([PathBuf; 4], Bound)> {
    before(end)?;
    owner.check_current(end).map_err(io_error)?;
    let profile = PinnedDirectory::open(owner.profile_path())?;
    let home = PinnedDirectory::open(owner.home_path())?;
    let client = PinnedDirectory::open_private(&profile.path().join(".codex"))?;
    let scope = owner.scope().map_err(io_error)?;
    let token = native::current_identity()?;
    let root = official_runtime_root(&token)?;
    let canonical = std::fs::canonicalize(client.path())?;
    let canonical = dunce::simplified(&canonical);
    if !canonical
        .to_string_lossy()
        .eq_ignore_ascii_case(&client.path().to_string_lossy())
    {
        return Err(refused());
    }
    let digest = hash::sha256_hex(canonical.to_str().ok_or_else(refused)?.as_bytes());
    let paths = [
        client.path().join(".opencodex-native-main.claim.sqlite"),
        root.join("native-write-locks")
            .join(format!("{digest}.sqlite")),
        root.join("client-desktop-lifecycle.sqlite"),
        root.join("catalog-write-locks")
            .join(format!("{digest}.sqlite")),
    ];
    Ok((
        paths,
        Bound {
            scope,
            profile,
            home,
            client,
        },
    ))
}
impl NativeLease {
    pub(super) fn check(&mut self, end: Instant) -> io::Result<()> {
        let result = self.check_inner(end);
        if result.is_err() {
            self.active = false;
        }
        result
    }
    fn check_inner(&mut self, end: Instant) -> io::Result<()> {
        if !self.active || self.held.len() != 4 || end > self.deadline {
            return Err(refused());
        }
        before(end)?;
        before(self.deadline)?;
        if native::current_identity()? != self.token {
            return Err(refused());
        }
        for held in &mut self.held {
            held.pin.check(end)?;
        }
        if let Some(bound) = &self.bound {
            bound.profile.recheck()?;
            bound.home.recheck()?;
            bound.client.recheck()?;
        }
        before(end)
    }
    pub(super) fn check_owner(&mut self, owner: &mut Owner, end: Instant) -> io::Result<()> {
        let result = self.check_owner_inner(owner, end);
        if result.is_err() {
            self.active = false;
        }
        result
    }
    fn check_owner_inner(&mut self, owner: &mut Owner, end: Instant) -> io::Result<()> {
        self.check(end)?;
        let bound = self.bound.as_ref().ok_or_else(refused)?;
        if owner.scope().map_err(io_error)? != bound.scope
            || owner.profile_path() != bound.profile.path()
            || owner.home_path() != bound.home.path()
        {
            return Err(refused());
        }
        owner.check_current(end).map_err(io_error)?;
        self.check(end)
    }
    pub(super) fn release(mut self, end: Instant) -> io::Result<()> {
        let valid = self.check(end);
        self.active = false;
        let mut released = Ok(());
        for held in self.held.iter_mut().rev() {
            if let Err(e) = held.release() {
                released = Err(e);
            }
        }
        // Expired/invalid work never becomes success by releasing its own locks.
        valid.and(released)
    }
}
impl Drop for NativeLease {
    fn drop(&mut self) {
        self.active = false;
        for held in self.held.iter_mut().rev() {
            let _ = held.release();
        }
    }
}
pub(super) fn official_runtime_root(token: &TokenIdentity) -> io::Result<PathBuf> {
    // Official user-identity.ts uses FOLDERID_LocalAppData DEFAULT_PATH(0x400),
    // effective TokenUser SID, then OpenCodex/Runtime/v1/<uppercaseSID>.
    let folder = windows_sys::core::GUID {
        data1: 0xf1b32785,
        data2: 0x6fba,
        data3: 0x4fcf,
        data4: [0x9d, 0x55, 0x7b, 0x8e, 0x7f, 0x15, 0x70, 0x91],
    };
    #[link(name = "shell32")]
    unsafe extern "system" {
        fn SHGetKnownFolderPath(
            id: *const windows_sys::core::GUID,
            flags: u32,
            token: *mut std::ffi::c_void,
            path: *mut *mut u16,
        ) -> i32;
    }
    #[link(name = "ole32")]
    unsafe extern "system" {
        fn CoTaskMemFree(value: *const std::ffi::c_void);
    }
    let mut raw_token = std::ptr::null_mut();
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw_token) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let original = unsafe { OwnedHandle::from_raw_handle(raw_token) };
    let mut raw = std::ptr::null_mut();
    let result =
        unsafe { SHGetKnownFolderPath(&folder, 0x400, original.as_raw_handle(), &mut raw) };
    if result < 0 || raw.is_null() {
        return Err(io::Error::from_raw_os_error(result));
    }
    struct Local(*mut u16);
    impl Drop for Local {
        fn drop(&mut self) {
            unsafe { CoTaskMemFree(self.0.cast()) }
        }
    }
    let guard = Local(raw);
    let mut len = 0;
    while len < 32768 && unsafe { *guard.0.add(len) } != 0 {
        len += 1;
    }
    if len == 0 || len == 32768 {
        return Err(refused());
    }
    use std::os::windows::ffi::OsStringExt;
    let path = PathBuf::from(std::ffi::OsString::from_wide(unsafe {
        std::slice::from_raw_parts(guard.0, len)
    }));
    native::validate_local_path(&path)?;
    if native::current_identity()? != *token {
        return Err(refused());
    }
    Ok(path
        .join("OpenCodex/Runtime/v1")
        .join(token.sid.to_uppercase()))
}
#[cfg(test)]
#[path = "coordination_tests.rs"]
mod tests;
