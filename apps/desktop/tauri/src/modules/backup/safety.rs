//! Bounded reads and fail-closed boundaries shared by browsing/restore/cleanup.
use super::{BackupManifest, BackupRecord, MANIFEST_NAME, SCHEMA_VERSION};
use crate::errors::{AppError, AppResult};
use std::{
    fs::{File, Metadata, OpenOptions},
    io::Read,
    path::{Component, Path, PathBuf},
};

pub const MAX_ENTRIES: usize = 5000;
pub const MAX_MANIFEST_BYTES: u64 = 64 * 1024;
pub const MAX_PAYLOAD_BYTES: u64 = 16 * 1024 * 1024;
pub fn fail(detail: impl Into<String>) -> AppError {
    AppError::FileSystem {
        operation: "manage backups".into(),
        detail: detail.into(),
    }
}
pub fn inspect(path: &Path) -> AppResult<Metadata> {
    let m = std::fs::symlink_metadata(path).map_err(|e| fail(e.to_string()))?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            // FILE_ATTRIBUTE_REPARSE_POINT
            return Err(fail("reparse point is not owned"));
        }
    }
    if m.file_type().is_symlink() || !(m.is_dir() || m.is_file()) {
        return Err(fail("symlink or unsupported file type"));
    }
    Ok(m)
}
/// Root is the explicit trust anchor (OS ancestors may include /var -> /private/var).
/// Every component below it is checked, including dangling links and absent parents.
pub fn check_path(root: &Path, path: &Path, allow_missing: bool) -> AppResult<()> {
    if !root.is_absolute() || !inspect(root)?.is_dir() {
        return Err(fail("invalid active root"));
    }
    let relative = path
        .strip_prefix(root)
        .map_err(|_| fail("outside active root"))?;
    let mut current = root.to_path_buf();
    for part in relative.components() {
        if !matches!(part, Component::Normal(_)) {
            return Err(fail("invalid path component"));
        }
        current.push(part);
        match std::fs::symlink_metadata(&current) {
            Ok(_) => {
                inspect(&current)?;
            }
            Err(e) if allow_missing && e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(fail(e.to_string())),
        }
    }
    Ok(())
}
pub fn open_file(path: &Path, create: bool) -> AppResult<File> {
    open_regular_file(path, create, create)
}
pub fn open_existing_writable(path: &Path) -> AppResult<File> {
    open_regular_file(path, true, false)
}
/// Check the opened handle, so an in-place truncate cannot affect another link.
pub fn require_single_link(file: &File) -> AppResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if file.metadata().map_err(|e| fail(e.to_string()))?.nlink() != 1 {
            return Err(fail("linked file is not owned"));
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Storage::FileSystem::{
            GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
        };
        let mut information = std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::uninit();
        // SAFETY: the live File owns the handle; Windows initializes information on success.
        let success =
            unsafe { GetFileInformationByHandle(file.as_raw_handle(), information.as_mut_ptr()) };
        if success == 0 {
            return Err(fail(std::io::Error::last_os_error().to_string()));
        }
        // SAFETY: the API returned success above.
        if unsafe { information.assume_init() }.nNumberOfLinks != 1 {
            return Err(fail("linked file is not owned"));
        }
    }
    Ok(())
}
fn open_regular_file(path: &Path, writable: bool, create: bool) -> AppResult<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(writable).create(create);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .mode(0o600);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x00200000); // FILE_FLAG_OPEN_REPARSE_POINT
    }
    let file = options.open(path).map_err(|e| fail(e.to_string()))?;
    let metadata = file.metadata().map_err(|e| fail(e.to_string()))?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(fail("opened reparse point"));
        }
    }
    if !metadata.is_file() {
        return Err(fail("not a regular file"));
    }
    Ok(file)
}
pub fn read_file(path: &Path, limit: u64) -> AppResult<Vec<u8>> {
    if !inspect(path)?.is_file() {
        return Err(fail("not a regular file"));
    }
    let file = open_file(path, false)?;
    if file.metadata().map_err(|e| fail(e.to_string()))?.len() > limit {
        return Err(fail("file exceeds read limit"));
    }
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| fail(e.to_string()))?;
    if bytes.len() as u64 > limit {
        return Err(fail("file exceeds read limit"));
    }
    Ok(bytes)
}
pub fn entries(path: &Path, remaining: &mut usize) -> AppResult<Vec<PathBuf>> {
    if !inspect(path)?.is_dir() {
        return Err(fail("not a directory"));
    }
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(path).map_err(|e| fail(e.to_string()))? {
        if *remaining == 0 {
            return Err(fail("backup tree exceeds entry limit"));
        }
        *remaining -= 1;
        let path = entry.map_err(|e| fail(e.to_string()))?.path();
        inspect(&path)?;
        paths.push(path);
    }
    paths.sort();
    Ok(paths)
}
pub fn payload_path(directory: &Path) -> AppResult<PathBuf> {
    let mut budget = MAX_ENTRIES;
    let paths = entries(directory, &mut budget)?;
    let mut payload = None;
    for path in paths {
        if path.file_name().is_some_and(|s| s == MANIFEST_NAME) {
            continue;
        }
        if !inspect(&path)?.is_file() || payload.is_some() {
            return Err(fail("expected exactly one payload"));
        }
        payload = Some(path);
    }
    payload.ok_or_else(|| fail("missing payload"))
}
pub fn validate_record(record: &BackupRecord) -> AppResult<()> {
    let m = &record.manifest;
    if m.schema_version != SCHEMA_VERSION
        || m.file_count != 1
        || m.sha256.len() != 64
        || !m.sha256.bytes().all(|b| b.is_ascii_hexdigit())
        || record.directory.file_name().and_then(|s| s.to_str()) != Some(&m.backup_id)
        || record
            .directory
            .parent()
            .and_then(Path::file_name)
            .and_then(|s| s.to_str())
            != Some(m.action.as_str())
    {
        return Err(fail("unsupported manifest schema/type/ownership"));
    }
    chrono::DateTime::parse_from_rfc3339(&m.created_at).map_err(|_| fail("invalid timestamp"))?;
    let payload = payload_path(&record.directory)?;
    if inspect(&payload)?.len() != m.bytes {
        return Err(fail("payload size mismatch"));
    }
    Ok(())
}
pub fn load_record(directory: &Path) -> AppResult<BackupRecord> {
    let bytes = read_file(&directory.join(MANIFEST_NAME), MAX_MANIFEST_BYTES)?;
    let manifest: BackupManifest =
        serde_json::from_slice(&bytes).map_err(|_| fail("invalid manifest"))?;
    let record = BackupRecord {
        manifest,
        directory: directory.into(),
    };
    validate_record(&record)?;
    Ok(record)
}
pub fn scan(backups: &Path) -> AppResult<Vec<BackupRecord>> {
    let root = backups
        .parent()
        .ok_or_else(|| fail("invalid backup root"))?;
    check_path(root, backups, true)?;
    if !backups.exists() {
        return Ok(Vec::new());
    }
    let mut budget = MAX_ENTRIES;
    let mut records = Vec::new();
    for year in entries(backups, &mut budget)? {
        if !inspect(&year)?.is_dir() {
            continue;
        }
        for month in entries(&year, &mut budget)? {
            if !inspect(&month)?.is_dir() {
                continue;
            }
            for action in entries(&month, &mut budget)? {
                if !inspect(&action)?.is_dir() {
                    continue;
                }
                // Unknown transaction layouts are never interpreted or deleted.
                if !super::ACTIONS
                    .iter()
                    .any(|s| action.file_name().is_some_and(|n| n == *s))
                {
                    continue;
                }
                for directory in entries(&action, &mut budget)? {
                    if !inspect(&directory)?.is_dir() {
                        continue;
                    }
                    // Count payload entries against the global traversal budget too.
                    entries(&directory, &mut budget)?;
                    if directory.join(MANIFEST_NAME).exists() {
                        records.push(load_record(&directory)?);
                    }
                }
            }
        }
    }
    Ok(records)
}

/// Persist backup directory entries before handing a protection handle to a caller.
/// Windows uses the existing file-sync/atomic-replace infrastructure contract.
pub fn sync_directories(root: &Path, directory: &Path) -> AppResult<()> {
    check_path(root, directory, false)?;
    #[cfg(unix)]
    {
        let mut current = directory;
        loop {
            if !inspect(current)?.is_dir() {
                return Err(fail("sync target is not a directory"));
            }
            File::open(current)
                .and_then(|file| file.sync_all())
                .map_err(|e| fail(e.to_string()))?;
            if current == root {
                break;
            }
            current = current
                .parent()
                .ok_or_else(|| fail("directory sync escaped root"))?;
        }
    }
    Ok(())
}
