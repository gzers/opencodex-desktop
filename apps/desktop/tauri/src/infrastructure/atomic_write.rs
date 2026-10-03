//! 临时文件写入、fsync 与同目录原子替换。
//!
//! 写入失败会尽力清理临时文件；替换失败不删除原始目标，避免产生半写入文件。

use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::errors::AppError;

/// 将数据写入目标同目录的临时文件，fsync 后原子替换目标。
pub fn atomic_write(target: &Path, data: &[u8], permissions: u32) -> Result<(), AppError> {
    let parent = target
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .ok_or_else(|| AppError::AtomicWrite {
            reason: "target has no parent directory".to_string(),
        })?;

    std::fs::create_dir_all(parent).map_err(|error| AppError::AtomicWrite {
        reason: format!("create parent directory: {}", error),
    })?;

    let temp_path = create_temp_sibling(target, permissions)?;
    let result = write_and_replace(&temp_path, target, data, permissions);
    if result.is_err() && temp_path.exists() {
        let _ = std::fs::remove_file(&temp_path);
    }
    result
}

fn create_temp_sibling(target: &Path, permissions: u32) -> Result<PathBuf, AppError> {
    let filename = target
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| AppError::AtomicWrite {
            reason: "target filename is not valid UTF-8".to_string(),
        })?;
    for attempt in 0..16 {
        let candidate = target.with_file_name(format!(
            ".{filename}.tmp-{pid}-{attempt}-{nanos}",
            pid = std::process::id(),
            nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| AppError::AtomicWrite {
                    reason: format!("system clock unavailable: {error}"),
                })?
                .as_nanos()
        ));
        match OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(permissions)
            .open(&candidate)
        {
            Ok(_) => return Ok(candidate),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(AppError::AtomicWrite {
                    reason: format!("create temporary file: {error}"),
                })
            }
        }
    }
    Err(AppError::AtomicWrite {
        reason: "temporary file name collision".to_string(),
    })
}

fn write_and_replace(
    temp_path: &Path,
    target: &Path,
    data: &[u8],
    permissions: u32,
) -> Result<(), AppError> {
    {
        let mut file = OpenOptions::new()
            .write(true)
            .mode(permissions)
            .open(temp_path)
            .map_err(|error| AppError::AtomicWrite {
                reason: format!("open temporary file: {error}"),
            })?;
        file.write_all(data)
            .map_err(|error| AppError::AtomicWrite {
                reason: format!("write temporary file: {error}"),
            })?;
        file.flush().map_err(|error| AppError::AtomicWrite {
            reason: format!("flush temporary file: {error}"),
        })?;
        file.sync_all().map_err(|error| AppError::AtomicWrite {
            reason: format!("fsync temporary file: {error}"),
        })?;
    }
    set_permissions(temp_path, permissions)?;
    std::fs::rename(temp_path, target).map_err(|error| AppError::AtomicWrite {
        reason: format!("atomic replace: {error}"),
    })
}

fn set_permissions(path: &Path, mode: u32) -> Result<(), AppError> {
    let metadata = std::fs::metadata(path).map_err(|error| AppError::AtomicWrite {
        reason: format!("read temporary permissions: {error}"),
    })?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(mode);
    std::fs::set_permissions(path, permissions).map_err(|error| AppError::AtomicWrite {
        reason: format!("set temporary permissions: {error}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::hash::sha256_hex;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn atomic_write_replaces_target_and_preserves_permissions() {
        let temp = tempfile::tempdir().expect("create temporary directory");
        let target = temp.path().join("config.json");
        std::fs::write(&target, b"before").expect("write before");

        atomic_write(&target, b"after", 0o600).expect("write after");

        assert_eq!(std::fs::read(&target).expect("read target"), b"after");
        let mode = std::fs::metadata(&target)
            .map(|metadata| metadata.permissions().mode())
            .expect("read metadata");
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn failed_write_leaves_original_target_unchanged() {
        let temp = tempfile::tempdir().expect("create temporary directory");
        let target = temp.path().join("config.json");
        std::fs::write(&target, b"original").expect("write original");
        let before = sha256_hex(b"original");

        let result = atomic_write(&target, b"changed", 0o040);
        assert!(result.is_err());

        let after = crate::infrastructure::hash::sha256_file(&target).expect("hash target");
        assert_eq!(after, before);
    }
}
