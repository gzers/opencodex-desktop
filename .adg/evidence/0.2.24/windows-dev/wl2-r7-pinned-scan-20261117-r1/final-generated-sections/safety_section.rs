// Bounded reads and fail-closed boundaries shared by browsing/restore/cleanup.
// Backup document types are outside this read-only section.
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