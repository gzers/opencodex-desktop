//! 单实例约束。
//!
//! 使用 FZ-39 冻结的 `app.lock` 机制：独占文件锁（Unix `flock` /
//! Windows `LockFileEx`），非阻塞获取，进程退出或崩溃时由操作系统释放。

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::errors::AppError;

/// RAII 单实例锁；Drop 时释放文件锁。
#[derive(Debug)]
pub struct AppInstanceLock {
    _file: std::fs::File,
    lock_path: PathBuf,
}

impl AppInstanceLock {
    /// 在指定数据根下获取单实例锁。
    ///
    /// `data_root` 是应用数据根的绝对路径；实际锁文件为
    /// `<data_root>/manager-state/app.lock`。
    pub fn acquire(data_root: &Path) -> Result<Self, AppError> {
        let state_dir = data_root.join("manager-state");
        std::fs::create_dir_all(&state_dir).map_err(|error| AppError::FileSystem {
            operation: "create manager-state directory".to_string(),
            detail: error.to_string(),
        })?;

        let lock_path = state_dir.join("app.lock");
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&lock_path)
            .map_err(|error| AppError::FileSystem {
                operation: "open instance lock".to_string(),
                detail: error.to_string(),
            })?;

        match file.try_lock() {
            Ok(()) => {}
            Err(std::fs::TryLockError::WouldBlock) => {
                return Err(AppError::InstanceLockConflict);
            }
            Err(std::fs::TryLockError::Error(error)) => {
                return Err(AppError::FileSystem {
                    operation: "acquire instance lock".to_string(),
                    detail: error.to_string(),
                });
            }
        }

        let pid = std::process::id();
        let started_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or_default();
        writeln!(file, "{{\"pid\":{pid},\"started_at\":{started_at}}}").map_err(|error| {
            AppError::FileSystem {
                operation: "write instance lock metadata".to_string(),
                detail: error.to_string(),
            }
        })?;

        Ok(Self {
            _file: file,
            lock_path,
        })
    }

    pub fn lock_path(&self) -> &Path {
        &self.lock_path
    }
}

impl Drop for AppInstanceLock {
    fn drop(&mut self) {
        let _ = self._file.unlock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_instance_acquires_lock() {
        let temp = tempfile::tempdir().expect("create temporary directory");
        let data_root = temp.path().join("data-root");

        let lock = AppInstanceLock::acquire(&data_root).expect("acquire lock");
        assert_eq!(lock.lock_path(), data_root.join("manager-state/app.lock"));
    }

    #[test]
    fn second_instance_is_blocked_with_conflict_error() {
        let temp = tempfile::tempdir().expect("create temporary directory");
        let data_root = temp.path().join("data-root");

        let _first = AppInstanceLock::acquire(&data_root).expect("acquire first lock");
        let second = AppInstanceLock::acquire(&data_root);
        assert!(matches!(second, Err(AppError::InstanceLockConflict)));
    }

    #[test]
    fn lock_is_released_after_drop() {
        let temp = tempfile::tempdir().expect("create temporary directory");
        let data_root = temp.path().join("data-root");

        {
            let _first = AppInstanceLock::acquire(&data_root).expect("acquire first lock");
        }

        let second = AppInstanceLock::acquire(&data_root);
        assert!(second.is_ok(), "lock should be released after drop");
    }

    #[test]
    fn lock_metadata_contains_pid() {
        let temp = tempfile::tempdir().expect("create temporary directory");
        let data_root = temp.path().join("data-root");

        let _lock = AppInstanceLock::acquire(&data_root).expect("acquire lock");
        let content =
            std::fs::read_to_string(data_root.join("manager-state/app.lock")).expect("read lock");
        assert!(content.contains(&format!("\"pid\":{}", std::process::id())));
    }
}
