//! 管理器运行日志的最小写入器。
//!
//! 只在数据根日志分区追加脱敏后的单行事件；失败不阻断业务命令，
//! 文件超过 5 MB 时滚动到 `.1`，旧 `.1` 删除。

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::errors::AppError;
use crate::infrastructure::platform::OpenOptionsModeExt;
use crate::modules::logs::{sanitize_line, APP_LOG_FILE_NAME};

pub const RUNTIME_LOG_MAX_BYTES: u64 = 5 * 1024 * 1024;
pub const RUNTIME_LOG_ROTATIONS: usize = 1;

#[derive(Debug, Clone)]
pub struct RuntimeLog {
    path: PathBuf,
}

impl RuntimeLog {
    pub fn new(data_root: &Path) -> Self {
        Self {
            path: data_root.join("logs").join(APP_LOG_FILE_NAME),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append_event(&self, message: &str) -> Result<(), AppError> {
        let _admission = crate::infrastructure::storage_writers::global().admit()?;
        self.rotate_if_needed()?;
        let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        self.append_line(&format!("{timestamp} [manager] {message}"))
    }

    pub fn append_result(
        &self,
        action: &'static str,
        result: Result<&'static str, String>,
    ) -> Result<(), AppError> {
        let message = match result {
            Ok(status) => format!("{action}: {status}"),
            Err(error) => format!("{action}: failed ({error})"),
        };
        self.append_event(&message)
    }

    fn append_line(&self, line: &str) -> Result<(), AppError> {
        let line = sanitize_line(line).ok_or(AppError::LogSanitization)?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .create_mode(0o600)
            .open(&self.path)
            .map_err(|error| AppError::FileSystem {
                operation: "open runtime log".to_string(),
                detail: error.to_string(),
            })?;
        file.write_all(line.as_bytes())
            .map_err(|error| AppError::FileSystem {
                operation: "write runtime log".to_string(),
                detail: error.to_string(),
            })?;
        file.write_all(b"\n")
            .map_err(|error| AppError::FileSystem {
                operation: "write runtime log separator".to_string(),
                detail: error.to_string(),
            })?;
        file.sync_all().map_err(|error| AppError::FileSystem {
            operation: "sync runtime log".to_string(),
            detail: error.to_string(),
        })
    }

    fn rotate_if_needed(&self) -> Result<bool, AppError> {
        let metadata = match std::fs::metadata(&self.path) {
            Ok(value) => value,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(error) => {
                return Err(AppError::FileSystem {
                    operation: "inspect runtime log".to_string(),
                    detail: error.to_string(),
                })
            }
        };
        if metadata.len() < RUNTIME_LOG_MAX_BYTES {
            return Ok(false);
        }
        let rotated = self.rotated_path()?;
        let _ = std::fs::remove_file(&rotated);
        std::fs::rename(&self.path, &rotated).map_err(|error| AppError::FileSystem {
            operation: "rotate runtime log".to_string(),
            detail: error.to_string(),
        })?;
        Ok(true)
    }

    fn rotated_path(&self) -> Result<PathBuf, AppError> {
        self.path
            .file_name()
            .map(|name| {
                self.path
                    .with_file_name(format!("{}.1", name.to_string_lossy()))
            })
            .ok_or_else(|| AppError::FileSystem {
                operation: "resolve runtime log rotation".to_string(),
                detail: "runtime log filename is not valid UTF-8".to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_sanitized_private_events_and_rotates() {
        let root = tempfile::tempdir().expect("create temporary root");
        std::fs::create_dir_all(root.path().join("logs")).expect("create logs partition");
        let logger = RuntimeLog::new(root.path());
        logger
            .append_event("https://user:token@example.com/v1?api_key=secret")
            .expect("append sanitized event");
        logger.append_event("normal event").expect("append event");
        let payload = std::fs::read_to_string(logger.path()).expect("read log");
        assert!(payload.contains("https://example.com/v1"));
        assert!(!payload.contains("api_key=secret"));
        assert_eq!(payload.lines().count(), 2);
        #[cfg(unix)]
        assert_eq!(
            crate::infrastructure::platform::mode_of(logger.path()).unwrap(),
            0o600
        );
    }
}
