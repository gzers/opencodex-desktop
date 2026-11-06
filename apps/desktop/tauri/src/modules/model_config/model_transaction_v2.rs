//! Exact-image local transactions. Caller owns config -> model-v2 and outer gates.
//! Only the fixed pending marker is recovered; historical protected images remain.
//! Checksums detect corruption, not an owner able to rewrite all private metadata.
//! Independent business writers must honor the caller's target locks.
use super::schema_v2::{self, error, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::{File, Metadata, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};
use zeroize::{Zeroize, Zeroizing};

const MAX_IMAGE: usize = 32 * 1024 * 1024;
const MAX_RECEIPT: usize = 32 * 1024;
const RECEIVE: &str = "model-receive-v2";
const MAX_TOTAL: usize = 128 * 1024 * 1024;
const MAX_FILES: usize = 8;
const MAX_JOURNAL: usize = 64 * 1024;
const MAX_MARKER: usize = 4096;
const MAX_RETAINED_TRANSACTIONS: usize = 256;
// Cumulative journal bytes plus dependency-bearing Policy/Draft image bytes.
const MAX_RETAINED_METADATA: usize = 32 * 1024 * 1024;
const MAX_RETAINED_ITEMS: usize = 65536;
const MAX_REF_ARRAY: usize = 4096;
const PENDING: &str = "model-transaction-v2.pending.json";
const TRANSACTIONS: &str = "model-transactions-v2";
const JOURNAL: &str = "journal.json";
const DRAFT: &str = "manager-state/model-config-draft.json";
const MANAGERS: [&str; 9] = [
    "manager-state/model-policy.json",
    "manager-state/model-materialization-ledger.json",
    "manager-state/model-protocol-mappings.json",
    DRAFT,
    "manager-state/model-family-templates.json",
    "manager-state/model-channel-layout.json",
    "manager-state/model-execution-source-ids.json",
    "manager-state/saved-content-versions.json",
    "manager-state/sync-identity-mappings-v2.json",
];

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "relative",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum TargetV2 {
    OfficialConfig,
    Manager(String),
    /// Canonical UUID4 operation ID, never a path. Only
    /// manager-state/model-receive-v2/<id>/receipt.json is addressable.
    ManagerOperationReceipt(String),
}
/// Raw images may contain credentials: intentionally no Debug/Serialize/Clone.
pub struct WriteV2 {
    pub target: TargetV2,
    pub before: Option<Vec<u8>>,
    pub after: Option<Vec<u8>>,
}
impl Drop for WriteV2 {
    fn drop(&mut self) {
        if let Some(b) = &mut self.before {
            b.zeroize();
        }
        if let Some(b) = &mut self.after {
            b.zeroize();
        }
    }
}
type Bytes = Zeroizing<Vec<u8>>;
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Binding {
    root: String,
    home: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Image {
    bytes: usize,
    digest: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Entry {
    target: TargetV2,
    before: Option<Image>,
    after: Option<Image>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Plan {
    id: String,
    binding: Binding,
    entries: Vec<Entry>,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Phase {
    Applying,
    RollingBack,
    Committed,
    RolledBack,
}
// No image bytes, plaintext paths or Debug in the journal.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Journal {
    version: u32,
    plan: Plan,
    phase: Phase,
    rollback_count: usize,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Marker {
    version: u32,
    id: String,
    binding: Binding,
    plan_digest: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope<T> {
    value: T,
    checksum: String,
}
fn fault(code: &'static str) -> schema_v2::MatchErrorV2 {
    error(code)
}
fn io<T>(r: std::io::Result<T>) -> Result<T> {
    r.map_err(|_| fault("model_transaction_io"))
}
fn encoded<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    schema_v2::canonical_bytes(v).map_err(|_| fault("model_transaction_journal_invalid"))
}
fn checksum<T: Serialize>(v: &T) -> Result<String> {
    Ok(schema_v2::digest(&encoded(v)?))
}
fn envelope<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    encoded(&Envelope {
        value: v,
        checksum: checksum(v)?,
    })
}
fn decode<T: DeserializeOwned + Serialize>(bytes: &[u8]) -> Result<T> {
    let value = schema_v2::parse_strict_json_v2(bytes)
        .map_err(|_| fault("model_transaction_journal_invalid"))?;
    let e: Envelope<T> =
        serde_json::from_value(value).map_err(|_| fault("model_transaction_journal_invalid"))?;
    if !schema_v2::valid_digest(&e.checksum) || checksum(&e.value)? != e.checksum {
        return Err(fault("model_transaction_checksum"));
    }
    Ok(e.value)
}
fn uuid4(id: &str) -> bool {
    uuid::Uuid::parse_str(id).is_ok_and(|u| {
        u.to_string() == id && u.get_version_num() == 4 && u.get_variant() == uuid::Variant::RFC4122
    })
}
fn target_name(t: &TargetV2) -> Result<&str> {
    match t {
        TargetV2::OfficialConfig => Ok("config.json"),
        TargetV2::ManagerOperationReceipt(id) if uuid4(id) => Ok("receipt.json"),
        TargetV2::Manager(p) if MANAGERS.contains(&p.as_str()) => Ok(&p["manager-state/".len()..]),
        _ => Err(fault("model_transaction_target_invalid")),
    }
}
fn target_key(t: &TargetV2) -> Result<String> {
    target_name(t)?;
    Ok(match t {
        TargetV2::OfficialConfig => "official".into(),
        TargetV2::Manager(p) => p.clone(),
        TargetV2::ManagerOperationReceipt(id) => {
            format!("manager-state/{RECEIVE}/{id}/receipt.json")
        }
    })
}
fn deletion_allowed(t: &TargetV2) -> bool {
    matches!(t, TargetV2::Manager(p) if p == DRAFT)
}
fn target_limit(target: &TargetV2) -> usize {
    if matches!(target, TargetV2::ManagerOperationReceipt(_)) {
        MAX_RECEIPT
    } else {
        MAX_IMAGE
    }
}
fn target_budget(target: &TargetV2, total: &mut usize, size: usize) -> Result<()> {
    if size > target_limit(target) {
        return Err(fault("model_transaction_limit"));
    }
    budget(total, size)
}
fn budget(total: &mut usize, size: usize) -> Result<()> {
    *total = total
        .checked_add(size)
        .ok_or_else(|| fault("model_transaction_limit"))?;
    if size > MAX_IMAGE || *total > MAX_TOTAL {
        return Err(fault("model_transaction_limit"));
    }
    Ok(())
}

// Unix child operations are relative to pinned directory descriptors. Windows
// directory handles exclude FILE_SHARE_DELETE and reject every reparse point.
// root/home are explicit anchors; OS ancestors may contain /var -> /private/var.
struct Dir {
    path: PathBuf,
    handle: File,
}
fn plain_name(name: &str) -> Result<()> {
    if name.is_empty() || name.contains(['/', '\\', '\0']) || name == "." || name == ".." {
        return Err(fault("model_transaction_path_invalid"));
    }
    Ok(())
}
fn supported(m: &Metadata) -> Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return Err(fault("model_transaction_path_invalid"));
        }
    }
    if m.file_type().is_symlink() {
        return Err(fault("model_transaction_path_invalid"));
    }
    Ok(())
}
fn private(m: &Metadata, directory: bool) -> Result<()> {
    supported(m)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        // SAFETY: geteuid has no arguments or memory preconditions.
        if m.uid() != unsafe { libc::geteuid() }
            || m.mode() & 0o7777 != if directory { 0o700 } else { 0o600 }
        {
            return Err(fault("model_transaction_permissions"));
        }
    }
    #[cfg(not(unix))]
    let _ = directory;
    Ok(())
}
fn regular(m: &Metadata, secret: bool) -> Result<()> {
    supported(m)?;
    if !m.is_file() {
        return Err(fault("model_transaction_path_invalid"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if m.nlink() != 1 {
            return Err(fault("model_transaction_path_invalid"));
        }
    }
    if secret {
        private(m, false)?;
    }
    Ok(())
}
fn same_file(a: &Metadata, b: &Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        a.dev() == b.dev() && a.ino() == b.ino()
    }
    #[cfg(not(unix))]
    {
        a.is_dir() == b.is_dir() && a.created().ok() == b.created().ok()
    }
}
impl Dir {
    fn anchor(path: &Path) -> Result<Self> {
        Self::anchor_access(path, false)
    }
    fn anchor_access(path: &Path, security_write: bool) -> Result<Self> {
        #[cfg(not(windows))]
        let _ = security_write;
        if !path.is_absolute()
            || path
                .components()
                .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
        {
            return Err(fault("model_transaction_path_invalid"));
        }
        let m = io(std::fs::symlink_metadata(path))?;
        supported(&m)?;
        if !m.is_dir() {
            return Err(fault("model_transaction_path_invalid"));
        }
        let path = io(std::fs::canonicalize(path))?;
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC);
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.custom_flags(0x02200000).share_mode(3);
            if security_write {
                options.access_mode(0x80000000 | 0x00040000);
            }
        }
        let handle = io(options.open(&path))?;
        if !same_file(&m, &io(handle.metadata())?) {
            return Err(fault("model_transaction_binding"));
        }
        Ok(Self { path, handle })
    }
    fn live(&self) -> Result<()> {
        let m = io(std::fs::symlink_metadata(&self.path))?;
        supported(&m)?;
        if !m.is_dir() || !same_file(&m, &io(self.handle.metadata())?) {
            return Err(fault("model_transaction_binding"));
        }
        Ok(())
    }
    fn binding(&self) -> Result<String> {
        let m = io(self.handle.metadata())?;
        #[cfg(unix)]
        {
            use std::os::unix::{ffi::OsStrExt, fs::MetadataExt};
            let mut b = self.path.as_os_str().as_bytes().to_vec();
            b.extend_from_slice(&m.dev().to_le_bytes());
            b.extend_from_slice(&m.ino().to_le_bytes());
            Ok(schema_v2::digest(&b))
        }
        #[cfg(not(unix))]
        {
            Ok(schema_v2::digest(
                format!("{}:{:?}", self.path.display(), m.created().ok()).as_bytes(),
            ))
        }
    }
    fn sync(&self) -> Result<()> {
        #[cfg(unix)]
        io(self.handle.sync_all())?;
        Ok(())
    }
    fn child(&self, name: &str, create: bool, secret: bool) -> Result<Option<Self>> {
        plain_name(name)?;
        self.live()?;
        #[cfg(unix)]
        {
            use std::{
                ffi::CString,
                os::fd::{AsRawFd, FromRawFd},
            };
            let c = CString::new(name).map_err(|_| fault("model_transaction_path_invalid"))?;
            if create {
                // SAFETY: owned descriptor and a validated NUL-terminated name.
                let rc = unsafe { libc::mkdirat(self.handle.as_raw_fd(), c.as_ptr(), 0o700) };
                if rc != 0
                    && std::io::Error::last_os_error().kind() != std::io::ErrorKind::AlreadyExists
                {
                    return Err(fault("model_transaction_io"));
                }
                self.sync()?;
            }
            // SAFETY: openat does not follow the final component.
            let fd = unsafe {
                libc::openat(
                    self.handle.as_raw_fd(),
                    c.as_ptr(),
                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                )
            };
            if fd < 0 {
                if std::io::Error::last_os_error().kind() == std::io::ErrorKind::NotFound {
                    return Ok(None);
                }
                return Err(fault("model_transaction_path_invalid"));
            }
            // SAFETY: successful openat gives a new owned descriptor.
            let handle = unsafe { File::from_raw_fd(fd) };
            if secret {
                private(&io(handle.metadata())?, true)?;
            }
            let d = Self {
                path: self.path.join(name),
                handle,
            };
            d.live()?;
            Ok(Some(d))
        }
        #[cfg(not(unix))]
        {
            let path = self.path.join(name);
            let mut created = false;
            if create {
                match std::fs::create_dir(&path) {
                    Ok(()) => created = true,
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
                    Err(_) => return Err(fault("model_transaction_io")),
                }
            }
            match std::fs::symlink_metadata(&path) {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(_) => Err(fault("model_transaction_io")),
                Ok(m) => {
                    if secret {
                        private(&m, true)?;
                    }
                    let d = Self::anchor_access(&path, created && secret)?;
                    #[cfg(windows)]
                    if secret {
                        if created {
                            win_private::protect(&d.handle, true)?;
                        }
                        win_private::verify(&d.handle, true)?;
                    }
                    Ok(Some(d))
                }
            }
        }
    }

    fn open(&self, name: &str, create: bool) -> Result<Option<File>> {
        plain_name(name)?;
        self.live()?;
        #[cfg(unix)]
        let opened = {
            use std::{
                ffi::CString,
                os::fd::{AsRawFd, FromRawFd},
            };
            let name = CString::new(name).map_err(|_| fault("model_transaction_path_invalid"))?;
            let flags = libc::O_NOFOLLOW
                | libc::O_NONBLOCK
                | libc::O_CLOEXEC
                | if create {
                    libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL
                } else {
                    libc::O_RDONLY
                };
            // SAFETY: validated component relative to owned directory descriptor.
            let fd = unsafe { libc::openat(self.handle.as_raw_fd(), name.as_ptr(), flags, 0o600) };
            if fd < 0 {
                Err(std::io::Error::last_os_error())
            } else {
                // SAFETY: successful openat returns a new owned descriptor.
                Ok(unsafe { File::from_raw_fd(fd) })
            }
        };
        #[cfg(not(unix))]
        let opened = {
            let mut options = OpenOptions::new();
            options.read(!create).write(create).create_new(create);
            #[cfg(windows)]
            {
                use std::os::windows::fs::OpenOptionsExt;
                options.custom_flags(0x00200000);
                if create {
                    options.access_mode(0x40000000 | 0x00040000 | 0x00020000);
                }
            }
            options.open(self.path.join(name))
        };
        match opened {
            Ok(f) => {
                #[cfg(windows)]
                {
                    win_private::single_link(&f)?;
                    if create {
                        win_private::protect(&f, false)?;
                    }
                }
                regular(&io(f.metadata())?, create)?;
                Ok(Some(f))
            }
            Err(e) if !create && e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(_) => Err(fault("model_transaction_io")),
        }
    }
    fn read(&self, name: &str, limit: usize, secret: bool) -> Result<Option<Bytes>> {
        let Some(mut f) = self.open(name, false)? else {
            return Ok(None);
        };
        let m = io(f.metadata())?;
        regular(&m, secret)?;
        #[cfg(windows)]
        if secret {
            win_private::verify(&f, false)?;
        }
        if m.len() > limit as u64 {
            return Err(fault("model_transaction_limit"));
        }
        // One allocation plus a growth sentinel: image buffers are never reallocated.
        let size = m.len() as usize;
        let mut bytes = Zeroizing::new(vec![0; size + 1]);
        let mut count = 0;
        while count < bytes.len() {
            let got = io(f.read(&mut bytes[count..]))?;
            if got == 0 {
                break;
            }
            count += got;
        }
        if count != size || io(f.metadata())?.len() != size as u64 {
            return Err(fault("model_transaction_read_changed"));
        }
        bytes.truncate(size);
        Ok(Some(bytes))
    }
    fn unlink(&self, name: &str) -> Result<()> {
        plain_name(name)?;
        self.live()?;
        #[cfg(unix)]
        {
            use std::{ffi::CString, os::fd::AsRawFd};
            let c = CString::new(name).map_err(|_| fault("model_transaction_path_invalid"))?;
            // SAFETY: validated component, owned directory descriptor.
            if unsafe { libc::unlinkat(self.handle.as_raw_fd(), c.as_ptr(), 0) } != 0 {
                return Err(fault("model_transaction_io"));
            }
        }
        #[cfg(not(unix))]
        io(std::fs::remove_file(self.path.join(name)))?;
        self.sync()
    }
    fn rename(&self, from: &str, to: &str, new: bool) -> Result<()> {
        plain_name(from)?;
        plain_name(to)?;
        self.live()?;
        #[cfg(unix)]
        {
            use std::{ffi::CString, os::fd::AsRawFd};
            let from = CString::new(from).map_err(|_| fault("model_transaction_path_invalid"))?;
            let to = CString::new(to).map_err(|_| fault("model_transaction_path_invalid"))?;
            let fd = self.handle.as_raw_fd();
            // An exclusive rename publishes a new object in one step. A
            // link/unlink pair would leave two links if interrupted, preventing
            // safe recovery under our single-link file policy.
            let rc = if new {
                #[cfg(target_vendor = "apple")]
                // SAFETY: both names and the pinned directory fd are valid.
                {
                    unsafe {
                        libc::renameatx_np(fd, from.as_ptr(), fd, to.as_ptr(), libc::RENAME_EXCL)
                    }
                }
                #[cfg(any(target_os = "linux", target_os = "android"))]
                // SAFETY: both names and the pinned directory fd are valid.
                {
                    unsafe {
                        libc::renameat2(
                            fd,
                            from.as_ptr(),
                            fd,
                            to.as_ptr(),
                            libc::RENAME_NOREPLACE as _,
                        )
                    }
                }
                #[cfg(not(any(
                    target_vendor = "apple",
                    target_os = "linux",
                    target_os = "android"
                )))]
                {
                    return Err(fault("model_transaction_platform_unsupported"));
                }
            } else {
                // SAFETY: atomic replacement within the same owned directory.
                unsafe { libc::renameat(fd, from.as_ptr(), fd, to.as_ptr()) }
            };
            if rc != 0 {
                return Err(fault("model_transaction_io"));
            }
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::Storage::FileSystem::{
                MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
            };
            let from: Vec<u16> = self
                .path
                .join(from)
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect();
            let to: Vec<u16> = self
                .path
                .join(to)
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect();
            // SAFETY: both UTF-16 buffers are terminated and valid for this call.
            if unsafe {
                MoveFileExW(
                    from.as_ptr(),
                    to.as_ptr(),
                    MOVEFILE_WRITE_THROUGH | if new { 0 } else { MOVEFILE_REPLACE_EXISTING },
                )
            } == 0
            {
                return Err(fault("model_transaction_io"));
            }
        }
        #[cfg(not(any(unix, windows)))]
        return Err(fault("model_transaction_platform_unsupported"));
        self.sync()
    }
    fn replace(
        &self,
        name: &str,
        before: Option<&[u8]>,
        after: Option<&[u8]>,
        secret: bool,
    ) -> Result<()> {
        if self
            .read(name, MAX_IMAGE, secret)?
            .as_ref()
            .map(|v| v.as_slice())
            != before
        {
            return Err(fault("model_transaction_cas"));
        }
        if before == after {
            return Ok(());
        }
        if let Some(after) = after {
            let temp = format!(".model-transaction-{}.tmp", uuid::Uuid::new_v4());
            let result = (|| {
                let mut f = self
                    .open(&temp, true)?
                    .ok_or_else(|| fault("model_transaction_io"))?;
                io(f.write_all(after))?;
                io(f.sync_all())?;
                drop(f);
                if self
                    .read(name, MAX_IMAGE, secret)?
                    .as_ref()
                    .map(|v| v.as_slice())
                    != before
                {
                    return Err(fault("model_transaction_cas"));
                }
                #[cfg(test)]
                test_point(TestEvent::BeforePublish)?;
                self.rename(&temp, name, before.is_none())
            })();
            if result.is_err() {
                let _ = self.unlink(&temp);
            }
            result?;
        } else {
            self.unlink(name)?;
        }
        if self
            .read(name, MAX_IMAGE, secret)?
            .as_ref()
            .map(|v| v.as_slice())
            != after
        {
            return Err(fault("model_transaction_readback"));
        }
        Ok(())
    }
    fn create(&self, name: &str, bytes: &[u8]) -> Result<()> {
        self.replace(name, None, Some(bytes), true)
    }
}
struct Context {
    root: Dir,
    home: Dir,
    manager: Option<Dir>,
}
impl Context {
    fn open(root: &Path, home: &Path) -> Result<Self> {
        let root = Dir::anchor(root)?;
        let home = Dir::anchor(home)?;
        let manager = root.child("manager-state", false, false)?;
        Ok(Self {
            root,
            home,
            manager,
        })
    }
    fn live(&self) -> Result<()> {
        self.root.live()?;
        self.home.live()?;
        if let Some(m) = &self.manager {
            m.live()?;
        }
        Ok(())
    }
    fn binding(&self) -> Result<Binding> {
        Ok(Binding {
            root: self.root.binding()?,
            home: self.home.binding()?,
        })
    }
    fn manager(&self) -> Result<&Dir> {
        self.manager
            .as_ref()
            .ok_or_else(|| fault("model_transaction_io"))
    }
    fn receipt_dir(&self, id: &str, create: bool) -> Result<Option<Dir>> {
        if !uuid4(id) {
            return Err(fault("model_transaction_target_invalid"));
        }
        self.live()?;
        let Some(manager) = &self.manager else {
            return Ok(None);
        };
        let Some(receive) = manager.child(RECEIVE, create, true)? else {
            return Ok(None);
        };
        let operation = receive.child(id, create, true)?;
        receive.live()?;
        Ok(operation)
    }
    fn read(&self, target: &TargetV2) -> Result<Option<Bytes>> {
        self.live()?;
        let name = target_name(target)?;
        match target {
            TargetV2::OfficialConfig => self.home.read(name, MAX_IMAGE, false),
            TargetV2::ManagerOperationReceipt(id) => self
                .receipt_dir(id, false)?
                .map(|d| d.read(name, MAX_RECEIPT, true))
                .unwrap_or(Ok(None)),
            TargetV2::Manager(_) => self
                .manager
                .as_ref()
                .map(|d| d.read(name, MAX_IMAGE, false))
                .unwrap_or(Ok(None)),
        }
    }
    fn replace(
        &self,
        target: &TargetV2,
        before: Option<&[u8]>,
        after: Option<&[u8]>,
    ) -> Result<()> {
        self.live()?;
        let receipt;
        let dir = match target {
            TargetV2::ManagerOperationReceipt(id) => {
                receipt = self.receipt_dir(id, after.is_some())?;
                let Some(dir) = &receipt else {
                    return if before.is_none() && after.is_none() {
                        Ok(())
                    } else {
                        Err(fault("model_transaction_cas"))
                    };
                };
                dir
            }
            TargetV2::OfficialConfig => &self.home,
            TargetV2::Manager(_) => self.manager()?,
        };
        dir.replace(
            target_name(target)?,
            before,
            after,
            matches!(target, TargetV2::ManagerOperationReceipt(_)),
        )
    }
}
fn image(bytes: &Option<Vec<u8>>) -> Option<Image> {
    bytes.as_ref().map(|b| Image {
        bytes: b.len(),
        digest: schema_v2::digest(b),
    })
}
fn image_name(index: usize, before: bool) -> String {
    format!("{index}.{}", if before { "before" } else { "after" })
}
struct Loaded {
    target: TargetV2,
    before: Option<Bytes>,
    after: Option<Bytes>,
}
impl Loaded {
    fn before(&self) -> Option<&[u8]> {
        self.before.as_ref().map(|b| b.as_slice())
    }
    fn after(&self) -> Option<&[u8]> {
        self.after.as_ref().map(|b| b.as_slice())
    }
}
fn load_image(dir: &Dir, name: &str, expected: &Option<Image>) -> Result<Option<Bytes>> {
    let bytes = dir.read(name, MAX_IMAGE, true)?;
    match (expected, &bytes) {
        (None, None) => (),
        (Some(i), Some(b)) if b.len() == i.bytes && schema_v2::digest(b) == i.digest => (),
        _ => return Err(fault("model_transaction_backup_invalid")),
    }
    Ok(bytes)
}
fn load_images(dir: &Dir, j: &Journal) -> Result<Vec<Loaded>> {
    j.plan
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            Ok(Loaded {
                target: e.target.clone(),
                before: load_image(dir, &image_name(i, true), &e.before)?,
                after: load_image(dir, &image_name(i, false), &e.after)?,
            })
        })
        .collect()
}
fn validate_journal(j: &Journal, m: &Marker, binding: &Binding) -> Result<()> {
    if m.version != 2 || j.version != 2 {
        return Err(fault("model_transaction_version"));
    }
    if !uuid4(&m.id) || m.id != j.plan.id {
        return Err(fault("model_transaction_journal_invalid"));
    }
    if &m.binding != binding || &j.plan.binding != binding {
        return Err(fault("model_transaction_binding"));
    }
    if checksum(&j.plan)? != m.plan_digest {
        return Err(fault("model_transaction_checksum"));
    }
    if j.plan.entries.is_empty()
        || j.plan.entries.len() > MAX_FILES
        || j.rollback_count > j.plan.entries.len()
    {
        return Err(fault("model_transaction_limit"));
    }
    if matches!(j.phase, Phase::Applying | Phase::Committed) && j.rollback_count != 0 {
        return Err(fault("model_transaction_journal_invalid"));
    }
    let mut seen = BTreeSet::new();
    let mut total = 0;
    for e in &j.plan.entries {
        if !seen.insert(target_key(&e.target)?)
            || (e.after.is_none() && !deletion_allowed(&e.target))
        {
            return Err(fault("model_transaction_target_invalid"));
        }
        for i in [&e.before, &e.after].into_iter().flatten() {
            target_budget(&e.target, &mut total, i.bytes)?;
            if !schema_v2::valid_digest(&i.digest) {
                return Err(fault("model_transaction_journal_invalid"));
            }
        }
    }
    Ok(())
}
fn save_journal(dir: &Dir, j: &Journal) -> Result<()> {
    let before = dir.read(JOURNAL, MAX_JOURNAL, true)?;
    let after = envelope(j)?;
    if after.len() > MAX_JOURNAL {
        return Err(fault("model_transaction_limit"));
    }
    dir.replace(
        JOURNAL,
        before.as_ref().map(|b| b.as_slice()),
        Some(&after),
        true,
    )
}
fn clear_marker(c: &Context, marker: &[u8]) -> Result<()> {
    c.live()?;
    c.manager()?.replace(PENDING, Some(marker), None, true)
}
fn legal_states(c: &Context, entries: &[Loaded]) -> Result<()> {
    for e in entries {
        let b = c.read(&e.target)?;
        let b = b.as_ref().map(|b| b.as_slice());
        if b != e.before() && b != e.after() {
            return Err(fault("model_transaction_external_change"));
        }
    }
    Ok(())
}
fn apply(c: &Context, entries: &[Loaded], attempted: &mut usize, recovering: bool) -> Result<()> {
    for (i, e) in entries.iter().enumerate() {
        *attempted = (*attempted).max(i + 1);
        #[cfg(test)]
        test_point(TestEvent::BeforeApply(i))?;
        let current = c.read(&e.target)?;
        if !recovering || current.as_ref().map(|b| b.as_slice()) != e.after() {
            c.replace(&e.target, e.before(), e.after())?;
        }
        #[cfg(test)]
        test_point(TestEvent::AfterApply(i))?;
    }
    Ok(())
}
// Persist the compensation direction before restoring ANY object. Continue past
// an unknown external edit, preserving it while restoring other known objects.
fn compensate(
    c: &Context,
    dir: &Dir,
    j: &mut Journal,
    entries: &[Loaded],
    marker: &[u8],
    attempted: usize,
) -> Result<()> {
    j.phase = Phase::RollingBack;
    j.rollback_count = attempted;
    save_journal(dir, j).map_err(|_| fault("model_transaction_recovery_required"))?;
    let mut failed = false;
    let mut external = false;
    for i in (0..attempted).rev() {
        let e = &entries[i];
        let result = (|| {
            #[cfg(test)]
            test_point(TestEvent::BeforeCompensate(i))?;
            let current = c.read(&e.target)?;
            let current = current.as_ref().map(|b| b.as_slice());
            if current == e.before() {
                return Ok(());
            }
            if current != e.after() {
                return Err(fault("model_transaction_external_change"));
            }
            c.replace(&e.target, e.after(), e.before())
        })();
        if let Err(err) = result {
            external |= matches!(
                err.code.as_str(),
                "model_transaction_external_change" | "model_transaction_cas"
            );
            failed = true;
        }
    }
    if failed {
        return Err(fault(if external {
            "model_transaction_compensation_conflict"
        } else {
            "model_transaction_compensation_failed"
        }));
    }
    legal_states(c, entries)?;
    j.phase = Phase::RolledBack;
    save_journal(dir, j)?;
    clear_marker(c, marker)
}
fn finish(
    c: &Context,
    dir: &Dir,
    j: &mut Journal,
    entries: &[Loaded],
    marker: &[u8],
    recovering: bool,
) -> Result<()> {
    if matches!(j.phase, Phase::RollingBack | Phase::RolledBack) {
        let count = j.rollback_count;
        return compensate(c, dir, j, entries, marker, count);
    }
    let mut attempted = if recovering { entries.len() } else { 0 };
    if let Err(err) = apply(c, entries, &mut attempted, recovering) {
        #[cfg(test)]
        if err.code == "model_transaction_test_interrupted" {
            return Err(err);
        }
        compensate(c, dir, j, entries, marker, attempted)?;
        return Err(err);
    }
    j.phase = Phase::Committed;
    save_journal(dir, j).map_err(|_| fault("model_transaction_recovery_required"))?;
    clear_marker(c, marker).map_err(|_| fault("model_transaction_recovery_required"))
}

/// Prepare durable exact images, check every before twice, then per-file CAS,
/// atomic replacement and readback. No target locks are acquired here.
pub fn commit(root: &Path, home: &Path, writes: Vec<WriteV2>) -> Result<()> {
    if writes.len() > MAX_FILES {
        return Err(fault("model_transaction_limit"));
    }
    let mut seen = BTreeSet::new();
    let mut total = 0;
    for w in &writes {
        if !seen.insert(target_key(&w.target)?)
            || (w.after.is_none() && !deletion_allowed(&w.target))
        {
            return Err(fault("model_transaction_target_invalid"));
        }
        for b in [&w.before, &w.after].into_iter().flatten() {
            target_budget(&w.target, &mut total, b.len())?;
        }
    }
    let mut c = Context::open(root, home)?;
    if let Some(m) = &c.manager {
        if m.read(PENDING, MAX_MARKER, true)?.is_some() {
            return Err(fault("model_transaction_pending"));
        }
    }
    if writes.is_empty() {
        return Ok(());
    }
    let confirm_before = |c: &Context| -> Result<()> {
        for w in &writes {
            if c.read(&w.target)?.as_ref().map(|b| b.as_slice()) != w.before.as_deref() {
                return Err(fault("model_transaction_cas"));
            }
        }
        Ok(())
    };
    confirm_before(&c)?;
    if c.manager.is_none() {
        c.manager = c.root.child("manager-state", true, false)?;
    }
    let transactions = c
        .manager()?
        .child(TRANSACTIONS, true, true)?
        .ok_or_else(|| fault("model_transaction_io"))?;
    let id = uuid::Uuid::new_v4().to_string();
    let dir = transactions
        .child(&id, true, true)?
        .ok_or_else(|| fault("model_transaction_io"))?;
    let mut j = Journal {
        version: 2,
        plan: Plan {
            id: id.clone(),
            binding: c.binding()?,
            entries: writes
                .iter()
                .map(|w| Entry {
                    target: w.target.clone(),
                    before: image(&w.before),
                    after: image(&w.after),
                })
                .collect(),
        },
        phase: Phase::Applying,
        rollback_count: 0,
    };
    for (i, w) in writes.iter().enumerate() {
        for (before, bytes) in [(true, &w.before), (false, &w.after)] {
            if let Some(bytes) = bytes {
                dir.create(&image_name(i, before), bytes)?;
            }
        }
    }
    save_journal(&dir, &j)?;
    let m = Marker {
        version: 2,
        id,
        binding: c.binding()?,
        plan_digest: checksum(&j.plan)?,
    };
    validate_journal(&j, &m, &c.binding()?)?;
    let entries = load_images(&dir, &j)?;
    #[cfg(test)]
    test_point(TestEvent::Prepared)?;
    confirm_before(&c)?;
    c.live()?;
    let marker = envelope(&m)?;
    c.manager()?.create(PENDING, &marker)?;
    finish(&c, &dir, &mut j, &entries, &marker, false)
}
/// Only this root/home binding's fixed pending marker is consulted. Validate
/// budgets, metadata, images and ALL target states before executing the plan.
pub fn recover(root: &Path, home: &Path) -> Result<bool> {
    let c = Context::open(root, home)?;
    let Some(manager) = &c.manager else {
        return Ok(false);
    };
    let Some(marker_bytes) = manager.read(PENDING, MAX_MARKER, true)? else {
        return Ok(false);
    };
    let marker: Marker = decode(&marker_bytes)?;
    if marker.version != 2 {
        return Err(fault("model_transaction_version"));
    }
    if !uuid4(&marker.id) {
        return Err(fault("model_transaction_journal_invalid"));
    }
    if marker.binding != c.binding()? {
        return Err(fault("model_transaction_binding"));
    }
    let transactions = manager
        .child(TRANSACTIONS, false, true)?
        .ok_or_else(|| fault("model_transaction_journal_invalid"))?;
    let dir = transactions
        .child(&marker.id, false, true)?
        .ok_or_else(|| fault("model_transaction_journal_invalid"))?;
    let journal_bytes = dir
        .read(JOURNAL, MAX_JOURNAL, true)?
        .ok_or_else(|| fault("model_transaction_journal_invalid"))?;
    let mut j: Journal = decode(&journal_bytes)?;
    validate_journal(&j, &marker, &c.binding()?)?;
    let entries = load_images(&dir, &j)?;
    legal_states(&c, &entries)?;
    finish(&c, &dir, &mut j, &entries, &marker_bytes, true)?;
    Ok(true)
}

/// Exact adoption dependencies retained by this root/home's local transaction
/// history. All valid phases retain BOTH images: even an uncommitted/rolled-back
/// plan is kept for diagnosis and explicit historical restoration. Library
/// images/parent chains are storage history, not evidence of template adoption.
///
/// Read-only and lock-free under the caller's admission/config/model locks. No
/// pending recovery, directory creation, historical-root discovery or truncation.
/// Metadata means journals plus dependency-bearing Policy/Draft images; both
/// count against the cumulative 32 MiB scan budget before image allocation.
pub fn retained_template_refs(root: &Path, home: &Path) -> Result<Vec<schema_v2::RefV2>> {
    let c = Context::open(root, home)?;
    let Some(manager) = &c.manager else {
        return Ok(vec![]);
    };
    let Some(transactions) = manager.child(TRANSACTIONS, false, true)? else {
        return Ok(vec![]);
    };
    let binding = c.binding()?;
    // Enumerate only this fixed directory, never payload trees or old backups.
    // Materialize a bounded name list first: do not return a successful prefix.
    let mut names = Vec::new();
    transactions.live()?;
    for entry in io(std::fs::read_dir(&transactions.path))? {
        let entry = io(entry)?;
        if names.len() == MAX_RETAINED_TRANSACTIONS {
            return Err(error("model_limit"));
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| fault("model_transaction_path_invalid"))?;
        if !uuid4(&name) {
            return Err(fault("model_transaction_path_invalid"));
        }
        let metadata = io(std::fs::symlink_metadata(entry.path()))?;
        supported(&metadata)?;
        if !metadata.is_dir() {
            return Err(fault("model_transaction_path_invalid"));
        }
        names.push(name);
    }
    names.sort_unstable();
    transactions.live()?;
    let mut scan = RetainedScan {
        metadata: 0,
        items: 0,
        refs: BTreeSet::new(),
    };
    for id in names {
        c.live()?;
        transactions.live()?;
        let dir = transactions
            .child(&id, false, true)?
            .ok_or_else(|| fault("model_transaction_journal_invalid"))?;
        let remaining = MAX_RETAINED_METADATA - scan.metadata;
        let bytes = dir
            .read(JOURNAL, MAX_JOURNAL.min(remaining), true)
            .map_err(retained_limit)?
            .ok_or_else(|| fault("model_transaction_journal_invalid"))?;
        scan.charge(bytes.len())?;
        let journal: Journal = decode(&bytes)?;
        // Historical directories have no pending marker. Bind validation to the
        // directory's canonical UUID, current anchors and checksummed plan.
        let marker = Marker {
            version: 2,
            id,
            binding: Binding {
                root: binding.root.clone(),
                home: binding.home.clone(),
            },
            plan_digest: checksum(&journal.plan)?,
        };
        validate_journal(&journal, &marker, &binding).map_err(retained_limit)?;
        for (index, entry) in journal.plan.entries.iter().enumerate() {
            let TargetV2::Manager(name) = &entry.target else {
                continue;
            };
            if name != "manager-state/model-policy.json" && name != DRAFT {
                continue;
            }
            for (before, expected) in [(true, &entry.before), (false, &entry.after)] {
                if let Some(image) = expected {
                    scan.charge(image.bytes)?;
                }
                // Use the declared, already-charged size as a hard read limit;
                // a corrupt larger file must not bypass the metadata budget.
                let image = scan.image(&dir, &image_name(index, before), expected)?;
                if let Some(image) = image {
                    let value = schema_v2::parse_strict_json_v2(&image)
                        .map_err(|_| fault("model_transaction_payload_invalid"))?;
                    if name == DRAFT {
                        scan.draft(&value)?;
                    } else {
                        scan.policy(&value)?;
                    }
                }
            }
        }
        dir.live()?;
    }
    c.live()?;
    transactions.live()?;
    Ok(scan.refs.into_iter().collect())
}

fn retained_limit(err: schema_v2::MatchErrorV2) -> schema_v2::MatchErrorV2 {
    if err.code == "model_transaction_limit" {
        error("model_limit")
    } else {
        err
    }
}

struct RetainedScan {
    metadata: usize,
    items: usize,
    refs: BTreeSet<schema_v2::RefV2>,
}
impl RetainedScan {
    fn image(&self, dir: &Dir, name: &str, expected: &Option<Image>) -> Result<Option<Bytes>> {
        let limit = expected.as_ref().map_or(0, |image| image.bytes);
        let bytes = dir.read(name, limit, true).map_err(|err| {
            if err.code == "model_transaction_limit" {
                fault("model_transaction_backup_invalid")
            } else {
                err
            }
        })?;
        match (expected, &bytes) {
            (None, None) => (),
            (Some(image), Some(bytes))
                if bytes.len() == image.bytes && schema_v2::digest(bytes) == image.digest => {}
            _ => return Err(fault("model_transaction_backup_invalid")),
        }
        Ok(bytes)
    }
    fn charge(&mut self, bytes: usize) -> Result<()> {
        self.metadata = self
            .metadata
            .checked_add(bytes)
            .ok_or_else(|| error("model_limit"))?;
        if self.metadata > MAX_RETAINED_METADATA {
            return Err(error("model_limit"));
        }
        Ok(())
    }
    fn item(&mut self) -> Result<()> {
        self.items += 1;
        if self.items > MAX_RETAINED_ITEMS {
            return Err(error("model_limit"));
        }
        Ok(())
    }
    fn version(value: &serde_json::Value) -> Result<()> {
        if !value.is_object() {
            return Err(fault("model_transaction_payload_invalid"));
        }
        if value
            .get("schemaVersion")
            .and_then(serde_json::Value::as_u64)
            != Some(2)
        {
            return Err(fault("model_transaction_version"));
        }
        Ok(())
    }
    fn array<'a>(value: &'a serde_json::Value, field: &str) -> Result<&'a Vec<serde_json::Value>> {
        let values = value
            .get(field)
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| fault("model_transaction_payload_invalid"))?;
        if values.len() > MAX_REF_ARRAY {
            return Err(error("model_limit"));
        }
        Ok(values)
    }
    fn reference(&mut self, value: &serde_json::Value) -> Result<()> {
        self.item()?;
        // Bound strings before cloning into a typed exact RefV2. Unknown fields
        // in the reference itself are refused by RefV2's strict deserializer.
        for (key, len) in [("templateId", 36), ("revisionId", 36), ("digest", 71)] {
            if value
                .get(key)
                .and_then(serde_json::Value::as_str)
                .is_none_or(|s| s.len() != len)
            {
                return Err(fault("model_transaction_reference_invalid"));
            }
        }
        let r: schema_v2::RefV2 = serde_json::from_value(value.clone())
            .map_err(|_| fault("model_transaction_reference_invalid"))?;
        if !schema_v2::valid_id(&r.template_id)
            || !schema_v2::valid_id(&r.revision_id)
            || !schema_v2::valid_digest(&r.digest)
        {
            return Err(fault("model_transaction_reference_invalid"));
        }
        self.refs.insert(r);
        Ok(())
    }
    fn references(&mut self, value: &serde_json::Value) -> Result<()> {
        for r in Self::array(value, "templateRefs")? {
            self.reference(r)?;
        }
        Ok(())
    }
    fn policy(&mut self, value: &serde_json::Value) -> Result<()> {
        Self::version(value)?;
        self.references(value)?;
        for channel in Self::array(value, "channels")? {
            self.item()?;
            self.references(channel)?;
            for model in Self::array(channel, "models")? {
                self.item()?;
                if !model.is_object() {
                    return Err(fault("model_transaction_payload_invalid"));
                }
                if let Some(binding) = model.get("binding").filter(|v| !v.is_null()) {
                    let kind = binding.get("kind").and_then(serde_json::Value::as_str);
                    if !matches!(kind, Some("manual" | "auto_read"))
                        || binding.as_object().is_none_or(|o| o.len() != 2)
                    {
                        return Err(fault("model_transaction_reference_invalid"));
                    }
                    self.reference(
                        binding
                            .get("reference")
                            .ok_or_else(|| fault("model_transaction_reference_invalid"))?,
                    )?;
                }
            }
        }
        Ok(())
    }
    fn draft(&mut self, value: &serde_json::Value) -> Result<()> {
        Self::version(value)?;
        if value
            .pointer("/base/dtoVersion")
            .and_then(serde_json::Value::as_u64)
            != Some(2)
        {
            return Err(fault("model_transaction_version"));
        }
        self.policy(
            value
                .get("policy")
                .ok_or_else(|| fault("model_transaction_payload_invalid"))?,
        )?;
        if let Some(baseline) = value.get("observedBaseline").filter(|v| !v.is_null()) {
            self.policy(baseline)?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum TestEvent {
    Prepared,
    BeforePublish,
    BeforeApply(usize),
    AfterApply(usize),
    BeforeCompensate(usize),
}
#[cfg(test)]
enum TestAction {
    Continue,
    Fail,
    Interrupt,
}
#[cfg(test)]
type TestHook = Box<dyn FnMut(TestEvent) -> TestAction>;
#[cfg(test)]
thread_local! { static TEST_HOOK: std::cell::RefCell<Option<TestHook>> = const { std::cell::RefCell::new(None) }; }
#[cfg(test)]
fn test_point(event: TestEvent) -> Result<()> {
    TEST_HOOK.with(|h| {
        match h
            .borrow_mut()
            .as_mut()
            .map(|h| h(event))
            .unwrap_or(TestAction::Continue)
        {
            TestAction::Continue => Ok(()),
            TestAction::Fail => Err(fault("model_transaction_write")),
            TestAction::Interrupt => Err(fault("model_transaction_test_interrupted")),
        }
    })
}
#[cfg(test)]
#[path = "model_transaction_v2_tests.rs"]
mod tests;

// POSIX 0600/0700 equivalent on Windows: a protected owner-only DACL. Apply it
// before writing any bytes, and verify existing private objects without changing
// their permissions. The three authorization declarations avoid adding a Cargo
// feature solely for this adapter; all types are from existing windows-sys.
#[cfg(windows)]
mod win_private {
    use super::*;
    use std::{
        ffi::c_void,
        os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
        ptr,
    };
    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::{
            EqualSid, GetSecurityDescriptorControl, GetSecurityDescriptorDacl, GetTokenInformation,
            TokenUser, ACL, TOKEN_QUERY, TOKEN_USER,
        },
        Storage::FileSystem::{GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION},
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    };
    #[link(name = "advapi32")]
    extern "system" {
        fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
            text: *const u16,
            revision: u32,
            descriptor: *mut *mut c_void,
            length: *mut u32,
        ) -> i32;
        fn GetSecurityInfo(
            handle: *mut c_void,
            kind: i32,
            information: u32,
            owner: *mut *mut c_void,
            group: *mut *mut c_void,
            dacl: *mut *mut ACL,
            sacl: *mut *mut ACL,
            descriptor: *mut *mut c_void,
        ) -> u32;
        fn SetSecurityInfo(
            handle: *mut c_void,
            kind: i32,
            information: u32,
            owner: *mut c_void,
            group: *mut c_void,
            dacl: *const ACL,
            sacl: *const ACL,
        ) -> u32;
    }
    struct Descriptor(*mut c_void);
    impl Drop for Descriptor {
        fn drop(&mut self) {
            // SAFETY: Windows returned this LocalAlloc-owned descriptor.
            unsafe {
                LocalFree(self.0);
            }
        }
    }
    fn descriptor(directory: bool) -> Result<Descriptor> {
        let text = if directory {
            "D:P(A;OICI;FA;;;OW)"
        } else {
            "D:P(A;;FA;;;OW)"
        };
        let text: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
        let mut sd = ptr::null_mut();
        // SAFETY: terminated input and a live output pointer; revision is SDDL 1.
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                text.as_ptr(),
                1,
                &mut sd,
                ptr::null_mut(),
            )
        } == 0
        {
            return Err(fault("model_transaction_permissions"));
        }
        Ok(Descriptor(sd))
    }
    fn dacl(sd: &Descriptor) -> Result<*mut ACL> {
        let mut present = 0;
        let mut defaulted = 0;
        let mut acl = ptr::null_mut();
        // SAFETY: descriptor is owned and valid; outputs are initialized by Win32.
        if unsafe { GetSecurityDescriptorDacl(sd.0, &mut present, &mut acl, &mut defaulted) } == 0
            || present == 0
            || acl.is_null()
        {
            return Err(fault("model_transaction_permissions"));
        }
        Ok(acl)
    }
    pub(super) fn protect(file: &File, directory: bool) -> Result<()> {
        let expected = descriptor(directory)?;
        let acl = dacl(&expected)?;
        // SAFETY: live file handle and ACL; protect the DACL against inheritance.
        if unsafe {
            SetSecurityInfo(
                file.as_raw_handle(),
                1,
                0x80000004,
                ptr::null_mut(),
                ptr::null_mut(),
                acl,
                ptr::null(),
            )
        } != 0
        {
            return Err(fault("model_transaction_permissions"));
        }
        verify(file, directory)
    }
    fn current_owner(owner: *mut c_void) -> Result<()> {
        let mut token = ptr::null_mut();
        // SAFETY: pseudo process handle and a live output pointer.
        if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
            return Err(fault("model_transaction_permissions"));
        }
        // SAFETY: successful OpenProcessToken transfers a new owned handle.
        let token = unsafe { OwnedHandle::from_raw_handle(token) };
        let mut size = 0;
        // SAFETY: the first call only queries the required buffer length.
        unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                ptr::null_mut(),
                0,
                &mut size,
            );
        }
        if size < std::mem::size_of::<TOKEN_USER>() as u32 || size > 65536 {
            return Err(fault("model_transaction_permissions"));
        }
        let mut buffer = vec![0usize; (size as usize).div_ceil(std::mem::size_of::<usize>())];
        // SAFETY: buffer is aligned for TOKEN_USER and has at least size bytes.
        if unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                buffer.as_mut_ptr().cast(),
                size,
                &mut size,
            )
        } == 0
        {
            return Err(fault("model_transaction_permissions"));
        }
        // SAFETY: successful TokenUser query initialized this aligned structure.
        let sid = unsafe { (*(buffer.as_ptr().cast::<TOKEN_USER>())).User.Sid };
        // SAFETY: both SIDs originate from successful Windows security queries.
        if owner.is_null() || unsafe { EqualSid(owner, sid) } == 0 {
            return Err(fault("model_transaction_permissions"));
        }
        Ok(())
    }
    pub(super) fn verify(file: &File, directory: bool) -> Result<()> {
        let expected = descriptor(directory)?;
        let expected_acl = dacl(&expected)?;
        let mut owner = ptr::null_mut();
        let mut acl = ptr::null_mut();
        let mut sd = ptr::null_mut();
        // SAFETY: live handle and output pointers; request owner and DACL only.
        if unsafe {
            GetSecurityInfo(
                file.as_raw_handle(),
                1,
                5,
                &mut owner,
                ptr::null_mut(),
                &mut acl,
                ptr::null_mut(),
                &mut sd,
            )
        } != 0
        {
            return Err(fault("model_transaction_permissions"));
        }
        let sd = Descriptor(sd);
        current_owner(owner)?;
        let mut control = 0;
        let mut revision = 0;
        // SAFETY: live security descriptor and valid output pointers.
        if unsafe { GetSecurityDescriptorControl(sd.0, &mut control, &mut revision) } == 0
            || control & 0x1000 == 0
            || acl.is_null()
        {
            return Err(fault("model_transaction_permissions"));
        }
        // SAFETY: successful security queries guarantee valid ACL headers and
        // AclSize-byte buffers, both kept alive by their descriptor guards.
        let equal = unsafe {
            let a = std::slice::from_raw_parts(acl.cast::<u8>(), (*acl).AclSize as usize);
            let b = std::slice::from_raw_parts(
                expected_acl.cast::<u8>(),
                (*expected_acl).AclSize as usize,
            );
            a == b
        };
        if !equal {
            return Err(fault("model_transaction_permissions"));
        }
        Ok(())
    }
    pub(super) fn single_link(file: &File) -> Result<()> {
        let mut info = std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::uninit();
        // SAFETY: live file handle and writable output storage.
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), info.as_mut_ptr()) } == 0 {
            return Err(fault("model_transaction_io"));
        }
        // SAFETY: successful query initialized the output.
        if unsafe { info.assume_init() }.nNumberOfLinks != 1 {
            return Err(fault("model_transaction_path_invalid"));
        }
        Ok(())
    }
}

#[path = "model_transaction_v2_history.rs"]
pub mod history;
