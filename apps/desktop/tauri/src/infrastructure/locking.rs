//! 受控目标文件的跨进程写锁。
//!
//! 该实现只作用于调用方提供的工程内目标路径；测试使用系统临时目录，
//! 不访问真实用户配置或数据根。锁语义为 `flock(LOCK_EX)`，超时 3 秒。

use std::fs::OpenOptions;
use std::io;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::errors::AppError;

/// FZ-26 冻结的目标文件锁超时。
pub const TARGET_LOCK_TIMEOUT: Duration = Duration::from_secs(3);

/// RAII 目标文件锁；Drop 时自动释放，进程崩溃时由操作系统释放。
#[derive(Debug)]
pub struct TargetFileLock {
    _file: std::fs::File,
}

impl TargetFileLock {
    pub fn lock(target: &Path) -> Result<Self, AppError> {
        Self::try_lock_with_timeout(target, TARGET_LOCK_TIMEOUT)
    }

    pub fn try_lock_with_timeout(target: &Path, timeout: Duration) -> Result<Self, AppError> {
        let lock_path = lock_path_for(target);
        if let Some(parent) = lock_path.parent() {
            std::fs::create_dir_all(parent).map_err(fs_error)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .read(true)
            .open(&lock_path)
            .map_err(fs_error)?;

        let started = Instant::now();
        loop {
            let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
            if result == 0 {
                let _ = lock_path;
                return Ok(Self { _file: file });
            }
            let error = io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::EWOULDBLOCK) {
                return Err(fs_error(error));
            }
            if started.elapsed() >= timeout {
                return Err(AppError::TargetLockTimeout {
                    timeout_ms: timeout.as_millis() as u32,
                });
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Drop for TargetFileLock {
    fn drop(&mut self) {
        unsafe {
            libc::flock(self._file.as_raw_fd(), libc::LOCK_UN);
        }
    }
}

pub fn lock_path_for(target: &Path) -> PathBuf {
    let mut filename = target
        .file_name()
        .map(std::ffi::OsStr::to_os_string)
        .unwrap_or_else(|| "target".into());
    filename.push(".lock");
    target.with_file_name(filename)
}

fn fs_error(error: io::Error) -> AppError {
    AppError::FileSystem {
        operation: "lock".to_string(),
        detail: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn target_lock_serializes_writers() {
        let temp = tempfile::tempdir().expect("create temporary directory");
        let target = temp.path().join("config.json");
        std::fs::write(&target, "before").expect("write target");

        let guard = TargetFileLock::lock(&target).expect("acquire first lock");
        let blocked = TargetFileLock::try_lock_with_timeout(&target, Duration::from_millis(20));
        assert!(matches!(
            blocked,
            Err(AppError::TargetLockTimeout { timeout_ms: 20 })
        ));

        drop(guard);
        let mut second = TargetFileLock::lock(&target).expect("acquire lock after release");
        second
            ._file
            .write_all(b"holder")
            .expect("write lock metadata");
    }

    #[test]
    fn lock_path_uses_frozen_suffix() {
        assert_eq!(
            lock_path_for(Path::new("/tmp/project/config.json")),
            Path::new("/tmp/project/config.json.lock")
        );
    }
}
