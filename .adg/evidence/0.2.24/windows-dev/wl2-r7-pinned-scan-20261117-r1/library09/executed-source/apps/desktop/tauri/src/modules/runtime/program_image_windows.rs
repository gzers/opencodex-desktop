//! One complete Windows inventory. Every local native object is retained only
//! for this call; no metadata/hash cache or grant survives an operation boundary.
use super::{fail, Entry, MAX_BYTES, MAX_ENTRIES, MAX_FILE, MAX_WORKERS};
use crate::errors::AppResult;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom},
    os::windows::{ffi::OsStrExt, fs::OpenOptionsExt, io::AsRawHandle},
    path::{Component, Path, PathBuf, Prefix},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Instant,
};
use windows_sys::Win32::{
    Foundation::{LocalFree, INVALID_HANDLE_VALUE},
    Security::{GetSecurityDescriptorLength, IsValidSecurityDescriptor},
    Storage::FileSystem::{
        FindClose, FindFirstStreamW, FindNextStreamW, FindStreamInfoStandard,
        GetFileInformationByHandle, GetFileType, GetFinalPathNameByHandleW,
        BY_HANDLE_FILE_INFORMATION, FILE_TYPE_DISK, WIN32_FIND_STREAM_DATA,
    },
};
#[link(name = "advapi32")]
unsafe extern "system" {
    fn QuerySecurityAccessMask(mask: u32, access: *mut u32);
    fn GetSecurityInfo(
        handle: *mut std::ffi::c_void,
        kind: i32,
        mask: u32,
        owner: *mut *mut std::ffi::c_void,
        group: *mut *mut std::ffi::c_void,
        dacl: *mut *mut windows_sys::Win32::Security::ACL,
        sacl: *mut *mut windows_sys::Win32::Security::ACL,
        descriptor: *mut *mut std::ffi::c_void,
    ) -> u32;
}
fn check(end: Option<Instant>) -> AppResult<()> {
    if end.is_some_and(|end| Instant::now() >= end) {
        Err(fail())
    } else {
        Ok(())
    }
}
fn clean(path: &Path) -> bool {
    super::super::program_layout::clean_absolute(path)
        && matches!(path.components().next(),Some(Component::Prefix(p)) if matches!(p.kind(),Prefix::Disk(_)|Prefix::VerbatimDisk(_)))
        && path.components().all(|c| match c {
            Component::Normal(n) => n
                .to_str()
                .is_some_and(|n| !n.ends_with([' ', '.']) && !n.contains(':')),
            _ => true,
        })
}
#[derive(Clone, PartialEq, Eq)]
struct Stamp {
    volume: u32,
    id: u64,
    attributes: u32,
    links: u32,
    bytes: u64,
    created: u64,
    modified: u64,
}
fn stamp(file: &File, directory: bool) -> AppResult<Stamp> {
    let mut i: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut i) } == 0
        || unsafe { GetFileType(file.as_raw_handle()) } != FILE_TYPE_DISK
        || i.dwFileAttributes & 0x400 != 0
        || (i.dwFileAttributes & 0x10 != 0) != directory
        || !directory && i.nNumberOfLinks != 1
    {
        return Err(fail());
    }
    Ok(Stamp {
        volume: i.dwVolumeSerialNumber,
        id: (u64::from(i.nFileIndexHigh) << 32) | u64::from(i.nFileIndexLow),
        attributes: i.dwFileAttributes,
        links: i.nNumberOfLinks,
        bytes: if directory {
            0
        } else {
            (u64::from(i.nFileSizeHigh) << 32) | u64::from(i.nFileSizeLow)
        },
        created: (u64::from(i.ftCreationTime.dwHighDateTime) << 32)
            | u64::from(i.ftCreationTime.dwLowDateTime),
        modified: (u64::from(i.ftLastWriteTime.dwHighDateTime) << 32)
            | u64::from(i.ftLastWriteTime.dwLowDateTime),
    })
}
fn final_path(file: &File, path: &Path) -> AppResult<()> {
    let mut raw = vec![0u16; 32768];
    let n = unsafe { GetFinalPathNameByHandleW(file.as_raw_handle(), raw.as_mut_ptr(), 32768, 0) };
    if n == 0 || n >= 32768 {
        return Err(fail());
    }
    let text = String::from_utf16(&raw[..n as usize]).map_err(|_| fail())?;
    let text = text.strip_prefix(r"\\?\").ok_or_else(fail)?;
    let expected = path.to_string_lossy().replace('/', r"\");
    let expected = expected.strip_prefix(r"\\?\").unwrap_or(&expected);
    if !text.eq_ignore_ascii_case(expected) {
        return Err(fail());
    }
    Ok(())
}
/// Equality fence over the SAME included eight security components. This raw
/// descriptor is never an originals/restore proof. Audit SACL remains excluded.
fn security(file: &File) -> AppResult<Vec<u8>> {
    let mut access = 0;
    unsafe { QuerySecurityAccessMask(0x1f7, &mut access) };
    if access != 0x20000 {
        return Err(fail());
    }
    let mut raw = std::ptr::null_mut();
    let result = unsafe {
        GetSecurityInfo(
            file.as_raw_handle(),
            1,
            0x1f7,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut raw,
        )
    };
    struct Allocation(*mut std::ffi::c_void);
    impl Drop for Allocation {
        fn drop(&mut self) {
            unsafe {
                LocalFree(self.0);
            }
        }
    }
    if result != 0 || raw.is_null() {
        return Err(fail());
    }
    let _allocation = Allocation(raw);
    if unsafe { IsValidSecurityDescriptor(raw) } == 0 {
        return Err(fail());
    }
    let size = unsafe { GetSecurityDescriptorLength(raw) } as usize;
    if !(20..=262144).contains(&size) {
        return Err(fail());
    }
    let bytes = unsafe { std::slice::from_raw_parts(raw.cast::<u8>(), size) };
    // GetSecurityInfo must return a valid relative revision1 descriptor; do not
    // dereference a foreign absolute structure or interpret unsupported state.
    if bytes[0] != 1 || u16::from_le_bytes([bytes[2], bytes[3]]) & 0x8000 == 0 {
        return Err(fail());
    }
    Ok(bytes.to_vec())
}
fn streams(path: &Path, directory: bool) -> AppResult<()> {
    let raw = path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let mut data = WIN32_FIND_STREAM_DATA::default();
    let handle = unsafe {
        FindFirstStreamW(
            raw.as_ptr(),
            FindStreamInfoStandard,
            (&mut data as *mut WIN32_FIND_STREAM_DATA).cast(),
            0,
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return if directory && std::io::Error::last_os_error().raw_os_error() == Some(38) {
            Ok(())
        } else {
            Err(fail())
        };
    }
    struct Search(windows_sys::Win32::Foundation::HANDLE);
    impl Drop for Search {
        fn drop(&mut self) {
            unsafe {
                FindClose(self.0);
            }
        }
    }
    let _search = Search(handle);
    let n = data
        .cStreamName
        .iter()
        .position(|c| *c == 0)
        .ok_or_else(fail)?;
    if String::from_utf16(&data.cStreamName[..n]).map_err(|_| fail())? != "::$DATA" {
        return Err(fail());
    }
    if unsafe { FindNextStreamW(handle, (&mut data as *mut WIN32_FIND_STREAM_DATA).cast()) } != 0
        || std::io::Error::last_os_error().raw_os_error() != Some(38)
    {
        return Err(fail());
    }
    Ok(())
}
struct Pin {
    file: File,
    path: PathBuf,
    directory: bool,
    before: Stamp,
    security: Vec<u8>,
}
impl Pin {
    fn open(path: &Path, directory: bool, end: Option<Instant>) -> AppResult<Self> {
        check(end)?;
        if !clean(path) {
            return Err(fail());
        }
        let file = OpenOptions::new()
            .access_mode(if directory { 0x20080 } else { 0x80020000 })
            .share_mode(if directory { 3 } else { 1 })
            .custom_flags(0x200000 | if directory { 0x2000000 } else { 0 })
            .open(path)
            .map_err(|_| fail())?;
        let before = stamp(&file, directory)?;
        final_path(&file, path)?;
        streams(path, directory)?;
        let security = security(&file)?;
        // Capture each native property once at admission, retain this exact
        // handle, and query everything again at the final inventory fence.
        // Do not immediately repeat the whole property set at the same open.
        if stamp(&file, directory)? != before {
            return Err(fail());
        }
        check(end)?;
        let pin = Self {
            file,
            path: path.into(),
            directory,
            before,
            security,
        };
        Ok(pin)
    }
    fn check(&self, end: Option<Instant>) -> AppResult<()> {
        check(end)?;
        if stamp(&self.file, self.directory)? != self.before
            || security(&self.file)? != self.security
        {
            return Err(fail());
        }
        final_path(&self.file, &self.path)?;
        streams(&self.path, self.directory)?;
        check(end)
    }
    fn hash(&self, end: Option<Instant>, cancelled: &AtomicBool) -> AppResult<Entry> {
        #[cfg(test)]
        let _cost = super::cost::span("windows-file-full-read-hash");
        if self.directory || self.before.bytes > MAX_FILE {
            return Err(fail());
        }
        let mut source = &self.file;
        source.seek(SeekFrom::Start(0)).map_err(|_| fail())?;
        let mut source = source.take(self.before.bytes.checked_add(1).ok_or_else(fail)?);
        let mut buffer = [0u8; 65536];
        let mut bytes = 0u64;
        let mut digest = Sha256::new();
        loop {
            check(end)?;
            if cancelled.load(Ordering::Acquire) {
                return Err(fail());
            }
            let n = source.read(&mut buffer).map_err(|_| fail())?;
            if n == 0 {
                break;
            }
            bytes += n as u64;
            if bytes > self.before.bytes {
                return Err(fail());
            }
            digest.update(&buffer[..n]);
        }
        if bytes != self.before.bytes || stamp(&self.file, false)? != self.before {
            return Err(fail());
        }
        check(end)?;
        Ok(Entry::File {
            bytes,
            sha256: format!("{:x}", digest.finalize()),
            executable: false,
        })
    }
}
fn names(path: &Path) -> AppResult<Vec<String>> {
    let mut rows = Vec::new();
    for item in fs::read_dir(path).map_err(|_| fail())? {
        if rows.len() >= MAX_ENTRIES {
            return Err(fail());
        }
        let name = item
            .map_err(|_| fail())?
            .file_name()
            .into_string()
            .map_err(|_| fail())?;
        if name.is_empty() || name.contains(['/', '\\']) || name == "." || name == ".." {
            return Err(fail());
        }
        rows.push(name);
    }
    rows.sort();
    if rows.windows(2).any(|x| x[0] == x[1]) {
        return Err(fail());
    }
    Ok(rows)
}
struct Directory {
    pin: Pin,
    children: Vec<String>,
    depth: usize,
}
struct FileJob {
    name: String,
    pin: Pin,
    excluded_manifest: bool,
}
struct Inventory {
    directories: Vec<Directory>,
    files: Vec<FileJob>,
    rows: BTreeMap<String, Entry>,
}
impl Inventory {
    fn collect(root: &Path, end: Option<Instant>) -> AppResult<Self> {
        #[cfg(test)]
        let _cost = super::cost::span("windows-native-enumeration-pin");
        let pin = Pin::open(root, true, end)?;
        let volume = pin.before.volume;
        let mut out = Self {
            directories: vec![Directory {
                children: names(root)?,
                pin,
                depth: 0,
            }],
            files: Vec::new(),
            rows: BTreeMap::new(),
        };
        let mut cursor = 0;
        let mut count = 0usize;
        let mut bytes = 0u64;
        let mut seen = BTreeSet::new();
        while cursor < out.directories.len() {
            check(end)?;
            let directory = &out.directories[cursor];
            let depth = directory.depth;
            if depth > 64 {
                return Err(fail());
            }
            let parent = directory.pin.path.clone();
            let children = directory.children.clone();
            for leaf in children {
                check(end)?;
                count += 1;
                if count > MAX_ENTRIES {
                    return Err(fail());
                }
                let path = parent.join(leaf);
                let meta = fs::symlink_metadata(&path).map_err(|_| fail())?;
                let pin = Pin::open(&path, meta.is_dir(), end)?;
                if pin.before.volume != volume {
                    return Err(fail());
                }
                let relative = path
                    .strip_prefix(root)
                    .map_err(|_| fail())?
                    .to_str()
                    .ok_or_else(fail)?
                    .replace('\\', "/");
                if !seen.insert(relative.clone()) {
                    return Err(fail());
                }
                let manifest =
                    depth == 0 && relative == super::super::program_selection::IMAGE_MANIFEST;
                if pin.directory {
                    if manifest || depth >= 64 {
                        return Err(fail());
                    }
                    let children = names(&path)?;
                    out.rows.insert(relative, Entry::Directory);
                    out.directories.push(Directory {
                        pin,
                        children,
                        depth: depth + 1,
                    });
                } else {
                    if !manifest {
                        if pin.before.bytes > MAX_FILE {
                            return Err(fail());
                        }
                        bytes = bytes
                            .checked_add(pin.before.bytes)
                            .filter(|n| *n <= MAX_BYTES)
                            .ok_or_else(fail)?;
                    }
                    out.files.push(FileJob {
                        name: relative,
                        pin,
                        excluded_manifest: manifest,
                    });
                }
            }
            if stamp(&out.directories[cursor].pin.file, true)? != out.directories[cursor].pin.before
            {
                return Err(fail());
            }
            check(end)?;
            if names(&parent)? != out.directories[cursor].children {
                return Err(fail());
            }
            cursor += 1;
        }
        check(end)?;
        Ok(out)
    }
    fn finish(
        self,
        end: Option<Instant>,
        workers: usize,
        checkpoint: &(impl Fn(&str) -> AppResult<()> + Sync),
    ) -> AppResult<BTreeMap<String, Entry>> {
        check(end)?;
        checkpoint("all-objects-pinned")?;
        let next = AtomicUsize::new(0);
        let cancelled = AtomicBool::new(false);
        let workers = workers.clamp(1, MAX_WORKERS).min(self.files.len().max(1));
        #[cfg(test)]
        let measured = super::cost::active();
        #[cfg(test)]
        let metrics = std::sync::Mutex::new(Vec::new());
        let result = std::thread::scope(|scope| {
            let mut handles = Vec::new();
            let mut failure = None;
            for _ in 0..workers {
                let next = &next;
                let cancelled = &cancelled;
                let files = &self.files;
                #[cfg(test)]
                let metrics = &metrics;
                match std::thread::Builder::new()
                    .name("ocxd-image-native-read".into())
                    .spawn_scoped(scope, move || -> AppResult<Vec<(String, Entry)>> {
                        struct Cancel<'a>(&'a AtomicBool);
                        impl Drop for Cancel<'_> {
                            fn drop(&mut self) {
                                if std::thread::panicking() {
                                    self.0.store(true, Ordering::Release);
                                }
                            }
                        }
                        let _cancel = Cancel(cancelled);
                        #[cfg(test)]
                        let _metrics = super::cost::Worker::new(measured, metrics, false);
                        let mut rows = Vec::new();
                        while !cancelled.load(Ordering::Acquire) {
                            let i = next.fetch_add(1, Ordering::Relaxed);
                            let Some(job) = files.get(i) else {
                                break;
                            };
                            if job.excluded_manifest {
                                continue;
                            }
                            let result = checkpoint("before-file-read")
                                .and_then(|()| job.pin.hash(end, cancelled));
                            match result {
                                Ok(entry) => rows.push((job.name.clone(), entry)),
                                Err(e) => {
                                    cancelled.store(true, Ordering::Release);
                                    return Err(e);
                                }
                            }
                        }
                        Ok(rows)
                    }) {
                    Ok(h) => handles.push(h),
                    Err(_) => {
                        cancelled.store(true, Ordering::Release);
                        failure = Some(fail());
                        break;
                    }
                }
            }
            let mut rows = Vec::new();
            for h in handles {
                match h.join() {
                    Ok(Ok(mut part)) => rows.append(&mut part),
                    Ok(Err(e)) => {
                        if failure.is_none() {
                            failure = Some(e);
                        }
                    }
                    Err(_) => {
                        cancelled.store(true, Ordering::Release);
                        failure = Some(fail());
                    }
                }
            }
            if let Some(e) = failure {
                return Err(e);
            }
            if rows.len() != self.files.iter().filter(|j| !j.excluded_manifest).count() {
                return Err(fail());
            }
            Ok(rows)
        });
        #[cfg(test)]
        if let Ok(metrics) = metrics.into_inner() {
            for m in metrics {
                super::cost::merge(m);
            }
        }
        let mut rows = self.rows;
        for (name, entry) in result? {
            if rows.insert(name, entry).is_some() {
                return Err(fail());
            }
        }
        checkpoint("before-final-fence")?;
        #[cfg(test)]
        let _cost = super::cost::span("windows-all-native-final-fence");
        // All original handles remain alive until EVERY worker joined and all
        // object/security/path/stream/membership fences complete in this call.
        for job in &self.files {
            job.pin.check(end)?;
        }
        for directory in &self.directories {
            directory.pin.check(end)?;
            if names(&directory.pin.path)? != directory.children {
                return Err(fail());
            }
        }
        check(end)?;
        Ok(rows)
    }
}
pub(super) fn inventory(root: &Path, end: Option<Instant>) -> AppResult<BTreeMap<String, Entry>> {
    inventory_checked(root, end, MAX_WORKERS, &|_| Ok(()))
}
pub(super) fn inventory_checked(
    root: &Path,
    end: Option<Instant>,
    workers: usize,
    checkpoint: &(impl Fn(&str) -> AppResult<()> + Sync),
) -> AppResult<BTreeMap<String, Entry>> {
    #[cfg(test)]
    let _cost = super::cost::span("windows-inventory-total");
    Inventory::collect(root, end)?.finish(end, workers, checkpoint)
}
