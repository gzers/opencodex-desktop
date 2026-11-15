use super::security::{check, current_identity, denied, verify_private, PrivateDescriptor};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    os::windows::{ffi::OsStrExt, fs::OpenOptionsExt, io::AsRawHandle},
    path::{Component, Path, PathBuf, Prefix},
    ptr,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateDirectoryW, GetFileInformationByHandle, GetFileType, BY_HANDLE_FILE_INFORMATION,
    FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS,
    FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_TYPE_DISK,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileIdentity {
    pub volume: u32,
    pub index: u64,
}
pub(super) fn identity(file: &File, directory: bool) -> io::Result<FileIdentity> {
    let mut i: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    check(unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut i) })?;
    if unsafe { GetFileType(file.as_raw_handle()) } != FILE_TYPE_DISK
        || i.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || (i.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0) != directory
        || !directory && i.nNumberOfLinks != 1
    {
        return Err(denied("reparse, multiple-link or wrong-kind object"));
    }
    Ok(FileIdentity {
        volume: i.dwVolumeSerialNumber,
        index: ((i.nFileIndexHigh as u64) << 32) | i.nFileIndexLow as u64,
    })
}
pub(super) fn validate(path: &Path) -> io::Result<()> {
    if !path.is_absolute()
        || !matches!(path.components().next(),Some(Component::Prefix(p)) if matches!(p.kind(),Prefix::Disk(_)|Prefix::VerbatimDisk(_)))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "explicit local drive path required",
        ));
    }
    let units: Vec<u16> = path.as_os_str().encode_wide().collect();
    if units.contains(&0)
        || units
            .split(|c| *c == 47 || *c == 92)
            .any(|p| p == [46] || p == [46, 46])
    {
        return Err(denied("nul/dot path rejected"));
    }
    for c in path.components() {
        if let Component::Normal(n) = c {
            let s = n
                .to_str()
                .ok_or_else(|| denied("non-Unicode path refused"))?;
            let stem = s.split('.').next().unwrap_or("").to_uppercase();
            if s.chars().any(|c| c < ' ' || ":<>\"|?*".contains(c))
                || s.ends_with([' ', '.'])
                || ["CON", "PRN", "AUX", "NUL"].contains(&stem.as_str())
                || ["COM", "LPT"].iter().any(|p| {
                    stem.strip_prefix(p).is_some_and(|x| {
                        ["1", "2", "3", "4", "5", "6", "7", "8", "9", "¹", "²", "³"].contains(&x)
                    })
                })
            {
                return Err(denied("ambiguous/device/stream path rejected"));
            }
        }
    }
    Ok(())
}
pub(super) fn wide(path: &Path) -> io::Result<Vec<u16>> {
    validate(path)?;
    let mut units: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .map(|c| if c == 47 { 92 } else { c })
        .collect();
    if !units.starts_with(&[92, 92, 63, 92]) {
        let mut p: Vec<u16> = "\\\\?\\".encode_utf16().collect();
        p.extend(units);
        units = p;
    }
    units.push(0);
    Ok(units)
}
fn directory(path: &Path) -> io::Result<File> {
    validate(path)?;
    let f = OpenOptions::new()
        .access_mode(0x80020000)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    identity(&f, true)?;
    Ok(f)
}

/// Retained read-data handles participate in Windows rename/delete sharing.
/// Ancestors are pinned from the drive root down; links are never followed.
pub struct PinnedDirectory {
    path: PathBuf,
    files: Vec<File>,
    id: FileIdentity,
    sid: Option<String>,
}
impl PinnedDirectory {
    pub fn open(path: &Path) -> io::Result<Self> {
        validate(path)?;
        let mut files = Vec::new();
        for p in path.ancestors().collect::<Vec<_>>().into_iter().rev() {
            files.push(directory(p)?);
        }
        let id = identity(
            files.last().ok_or_else(|| denied("missing directory"))?,
            true,
        )?;
        Ok(Self {
            path: path.into(),
            files,
            id,
            sid: None,
        })
    }
    pub fn open_private(path: &Path) -> io::Result<Self> {
        let mut p = Self::open(path)?;
        let sid = current_identity()?.sid;
        verify_private(p.handle(), &sid, true)?;
        p.sid = Some(sid);
        Ok(p)
    }
    pub fn identity(&self) -> FileIdentity {
        self.id
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub(super) fn handle(&self) -> windows_sys::Win32::Foundation::HANDLE {
        self.files.last().unwrap().as_raw_handle()
    }
    pub fn recheck(&self) -> io::Result<()> {
        if identity(self.files.last().unwrap(), true)? != self.id {
            return Err(denied("directory identity changed"));
        }
        if let Some(sid) = &self.sid {
            verify_private(self.handle(), sid, true)?;
        }
        Ok(())
    }
}
pub fn create_private_directory(path: &Path) -> io::Result<PinnedDirectory> {
    let parent = path
        .parent()
        .ok_or_else(|| denied("non-root directory required"))?;
    let _parents = PinnedDirectory::open(parent)?;
    let sid = current_identity()?.sid;
    let sd = PrivateDescriptor::new(&sid, true)?;
    let sa = sd.attributes();
    let w = wide(path)?;
    check(unsafe { CreateDirectoryW(w.as_ptr(), &sa) })?;
    PinnedDirectory::open_private(path)
}

/// An immutable, current-user-owned record. Format/meaning belongs to the caller.
/// The handle denies write/delete sharing for the entire object's lifetime.
pub struct PrivateRecord {
    file: File,
    _parent: PinnedDirectory,
    id: FileIdentity,
    sid: String,
    max: usize,
}
impl PrivateRecord {
    pub fn create(path: &Path, bytes: &[u8], max: usize) -> io::Result<Self> {
        if max == 0 || max > 1024 * 1024 || bytes.len() > max {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "record size bound",
            ));
        }
        let parent = PinnedDirectory::open_private(
            path.parent()
                .ok_or_else(|| denied("record parent required"))?,
        )?;
        validate(path)?;
        let sid = current_identity()?.sid;
        let sd = PrivateDescriptor::new(&sid, false)?;
        use std::os::windows::io::FromRawHandle;
        let w = wide(path)?;
        let sa = sd.attributes();
        let raw = unsafe {
            windows_sys::Win32::Storage::FileSystem::CreateFileW(
                w.as_ptr(),
                0xc0020000,
                FILE_SHARE_READ,
                &sa,
                1,
                FILE_FLAG_OPEN_REPARSE_POINT,
                ptr::null_mut(),
            )
        };
        if raw == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        let mut file = unsafe { File::from_raw_handle(raw) };
        let id = identity(&file, false)?;
        verify_private(raw, &sid, true)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        Ok(Self {
            file,
            _parent: parent,
            id,
            sid,
            max,
        })
    }
    pub fn open(path: &Path, max: usize) -> io::Result<Self> {
        if max == 0 || max > 1024 * 1024 {
            return Err(denied("invalid record bound"));
        }
        let parent = PinnedDirectory::open_private(
            path.parent()
                .ok_or_else(|| denied("record parent required"))?,
        )?;
        validate(path)?;
        let sid = current_identity()?.sid;
        let file = OpenOptions::new()
            .access_mode(0x80020000)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)?;
        let id = identity(&file, false)?;
        verify_private(file.as_raw_handle(), &sid, true)?;
        if file.metadata()?.len() > max as u64 {
            return Err(denied("record too large"));
        }
        Ok(Self {
            file,
            _parent: parent,
            id,
            sid,
            max,
        })
    }
    pub fn identity(&self) -> FileIdentity {
        self.id
    }
    pub fn read(&mut self) -> io::Result<Vec<u8>> {
        self._parent.recheck()?;
        verify_private(self.file.as_raw_handle(), &self.sid, true)?;
        if identity(&self.file, false)? != self.id {
            return Err(denied("record identity changed"));
        }
        if self.file.metadata()?.len() > self.max as u64 {
            return Err(denied("record too large"));
        }
        self.file.seek(SeekFrom::Start(0))?;
        let mut b = Vec::new();
        (&mut self.file)
            .take(self.max as u64 + 1)
            .read_to_end(&mut b)?;
        if b.len() > self.max {
            return Err(denied("record exceeded bound"));
        }
        Ok(b)
    }
}
