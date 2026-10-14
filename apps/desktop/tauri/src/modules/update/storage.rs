//! Fixed update files remain below the captured active data root. These helpers
//! do not select a root or change its binding, and never initialize another one.
use crate::errors::{AppError, AppResult};
use std::{
    fs::{File, Metadata, OpenOptions},
    io::Read,
    path::{Component, Path, PathBuf},
};

fn inspect(path: &Path) -> AppResult<Metadata> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| AppError::NotConfigured)?;
    require_owned_type(&metadata)?;
    Ok(metadata)
}

fn require_owned_type(metadata: &Metadata) -> AppResult<()> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(AppError::NotConfigured); // FILE_ATTRIBUTE_REPARSE_POINT
        }
    }
    if metadata.file_type().is_symlink() || !(metadata.is_file() || metadata.is_dir()) {
        return Err(AppError::NotConfigured);
    }
    Ok(())
}

/// The approved root is the anchor; OS ancestors such as /var -> /private/var
/// may be aliases. Check every directory and the final existing file below it.
pub(super) fn file_path(root: &Path, relative: &Path, create: bool) -> AppResult<PathBuf> {
    if !root.is_absolute() || !inspect(root)?.is_dir() {
        return Err(AppError::NotConfigured);
    }
    let parts = relative.components().collect::<Vec<_>>();
    if parts.is_empty()
        || parts
            .iter()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(AppError::NotConfigured);
    }
    let mut path = root.to_path_buf();
    for part in &parts[..parts.len() - 1] {
        path.push(part.as_os_str());
        match std::fs::symlink_metadata(&path) {
            Ok(metadata) => {
                require_owned_type(&metadata)?;
                if !metadata.is_dir() {
                    return Err(AppError::NotConfigured);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if create {
                    if let Err(error) = std::fs::create_dir(&path) {
                        if error.kind() != std::io::ErrorKind::AlreadyExists {
                            return Err(AppError::NotConfigured);
                        }
                    }
                    if !inspect(&path)?.is_dir() {
                        return Err(AppError::NotConfigured);
                    }
                }
            }
            _ => return Err(AppError::NotConfigured),
        }
    }
    path.push(parts.last().unwrap().as_os_str());
    match std::fs::symlink_metadata(&path) {
        Ok(metadata) => {
            require_owned_type(&metadata)?;
            if !metadata.is_file() {
                return Err(AppError::NotConfigured);
            }
            if create {
                open_regular(&path).map_err(|_| AppError::NotConfigured)?;
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        _ => return Err(AppError::NotConfigured),
    }
    Ok(path)
}

fn open_regular(path: &Path) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x00200000); // FILE_FLAG_OPEN_REPARSE_POINT
    }
    let file = options.open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || require_owned_type(&metadata).is_err() {
        return Err(std::io::Error::other(
            "update file is not a regular owned file",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 {
            return Err(std::io::Error::other("update file is linked"));
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Storage::FileSystem::{
            GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
        };
        let mut information = std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::uninit();
        // SAFETY: File owns this live handle; the API writes its output on success.
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), information.as_mut_ptr()) }
            == 0
        {
            return Err(std::io::Error::last_os_error());
        }
        // SAFETY: the API returned success above.
        if unsafe { information.assume_init() }.nNumberOfLinks != 1 {
            return Err(std::io::Error::other("update file is linked"));
        }
    }
    Ok(file)
}

pub(super) fn read_file(path: &Path, limit: u64) -> std::io::Result<Vec<u8>> {
    let file = open_regular(path)?;
    if file.metadata()?.len() > limit {
        return Err(std::io::Error::other("update file exceeds read limit"));
    }
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(std::io::Error::other("update file exceeds read limit"));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_paths_and_missing_read_parent_do_not_initialize_storage() {
        let root = tempfile::tempdir().unwrap();
        for invalid in ["../escape.json", "cache/../escape.json", ""] {
            assert!(file_path(root.path(), Path::new(invalid), true).is_err());
        }
        assert!(file_path(root.path(), root.path(), true).is_err());
        assert!(file_path(Path::new("relative"), Path::new("cache/value.json"), true).is_err());
        let absent = file_path(root.path(), Path::new("cache/updates/value.json"), false).unwrap();
        assert_eq!(
            read_file(&absent, 10).unwrap_err().kind(),
            std::io::ErrorKind::NotFound
        );
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    }

    #[test]
    fn bounds_and_linked_files_are_refused_without_truncating_the_payload() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let payload = root.path().join("payload.json");
        std::fs::write(&payload, b"12345678901").unwrap();
        assert!(read_file(&payload, 10).is_err());
        assert_eq!(read_file(&payload, 11).unwrap(), b"12345678901");
        std::fs::hard_link(&payload, outside.path().join("linked.json")).unwrap();
        assert!(read_file(&payload, 11).is_err());
        assert!(file_path(root.path(), Path::new("payload.json"), true).is_err());
        assert_eq!(std::fs::read(&payload).unwrap(), b"12345678901");
        assert_eq!(
            std::fs::read(outside.path().join("linked.json")).unwrap(),
            b"12345678901"
        );
    }

    #[cfg(unix)]
    #[test]
    fn special_file_is_rejected_without_waiting_for_a_writer() {
        use std::os::unix::ffi::OsStrExt;
        let root = tempfile::tempdir().unwrap();
        let fifo = root.path().join("fifo.json");
        let name = std::ffi::CString::new(fifo.as_os_str().as_bytes()).unwrap();
        // SAFETY: the CString owns a NUL-terminated path in this test's root.
        assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
        assert!(file_path(root.path(), Path::new("fifo.json"), false).is_err());
        assert!(read_file(&fifo, 10).is_err());
    }
}
