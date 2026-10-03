//! MOD-12 FZ-40 脱敏审计写入、保留清理与轮转。
//!
//! 每行一条 JSON 记录，只写入调用键与结果；不记录参数值、路径、
//! 凭据或响应数据。日志写入数据根下的 `audit.log`。

use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use crate::errors::AppError;
use crate::modules::ipc::{
    AuditRecord, AUDIT_LOG_MAX_BYTES, AUDIT_LOG_ROTATIONS, AUDIT_MAX_RECORDS, AUDIT_RETENTION_DAYS,
};

pub const AUDIT_LOG_FILE_NAME: &str = "audit.log";
const AUDIT_RECORD_LINE_ESTIMATE: usize = 512;
const AUDIT_COUNT_SLACK: usize = 16;

fn write_retained(path: &Path, records: &[AuditRecord]) -> Result<(), AppError> {
    let payload = records
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AppError::FileSystem {
            operation: "serialize retained audit records".to_string(),
            detail: error.to_string(),
        })?
        .join("\n");
    let payload = if payload.is_empty() {
        Vec::new()
    } else {
        let mut bytes = payload.into_bytes();
        bytes.push(b'\n');
        bytes
    };
    crate::infrastructure::atomic_write::atomic_write(path, &payload, 0o600)
}

pub struct AuditStore {
    path: PathBuf,
}

impl AuditStore {
    pub fn new(data_root: &Path) -> Self {
        Self {
            path: data_root.join(AUDIT_LOG_FILE_NAME),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 追加一条审计记录；每次写入前执行保留与轮转。
    pub fn record(&self, value: &AuditRecord) -> Result<(), AppError> {
        self.rotate_if_needed()?;
        self.enforce_count_limit()?;
        let payload = serde_json::to_vec(value).map_err(|error| AppError::FileSystem {
            operation: "serialize audit record".to_string(),
            detail: error.to_string(),
        })?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .mode(0o600)
            .open(&self.path)
            .map_err(|error| AppError::FileSystem {
                operation: "open audit log".to_string(),
                detail: error.to_string(),
            })?;
        file.write_all(&payload)
            .map_err(|error| AppError::FileSystem {
                operation: "write audit record".to_string(),
                detail: error.to_string(),
            })?;
        file.write_all(b"\n")
            .map_err(|error| AppError::FileSystem {
                operation: "write audit record separator".to_string(),
                detail: error.to_string(),
            })?;
        file.sync_all().map_err(|error| AppError::FileSystem {
            operation: "sync audit log".to_string(),
            detail: error.to_string(),
        })
    }

    /// 超过保留期或最大记录数的旧行会被整行移除。
    pub fn retire(&self, now: chrono::DateTime<chrono::Utc>) -> Result<(), AppError> {
        let mut records = self.read_records()?;
        if records.is_empty() {
            return Ok(());
        }
        let before = records.len();
        records.retain(|record| {
            matches!(
                chrono::DateTime::parse_from_rfc3339(&record.finished_at).map(|finished| now
                    .signed_duration_since(finished.with_timezone(&chrono::Utc))
                    .num_days()
                    < AUDIT_RETENTION_DAYS),
                Ok(true)
            )
        });
        if records.len() > AUDIT_MAX_RECORDS {
            let drop_count = records.len() - AUDIT_MAX_RECORDS;
            records.drain(..drop_count);
        }
        if records.len() == before {
            return Ok(());
        }
        write_retained(&self.path, &records)
    }

    /// 追加前的轻量上限检查；准确日期清理由维护调用触发。
    fn enforce_count_limit(&self) -> Result<(), AppError> {
        let count = match std::fs::metadata(&self.path) {
            Ok(metadata) => metadata.len() as usize / (AUDIT_RECORD_LINE_ESTIMATE + 1),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
            Err(error) => {
                return Err(AppError::FileSystem {
                    operation: "inspect audit count".to_string(),
                    detail: error.to_string(),
                })
            }
        };
        if count < AUDIT_MAX_RECORDS + AUDIT_COUNT_SLACK {
            return Ok(());
        }
        let mut records = self.read_records()?;
        if records.len() <= AUDIT_MAX_RECORDS {
            return Ok(());
        }
        let drop_count = records.len() - AUDIT_MAX_RECORDS;
        records.drain(..drop_count);
        write_retained(&self.path, &records)
    }

    /// `audit.log` 超过 5 MB 时滚动为 `.1`，最旧 `.5` 删除。
    pub fn rotate_if_needed(&self) -> Result<bool, AppError> {
        let metadata = match std::fs::metadata(&self.path) {
            Ok(value) => value,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(error) => {
                return Err(AppError::FileSystem {
                    operation: "inspect audit log".to_string(),
                    detail: error.to_string(),
                })
            }
        };
        if metadata.len() < AUDIT_LOG_MAX_BYTES {
            return Ok(false);
        }
        let oldest = self.rotated_path(AUDIT_LOG_ROTATIONS)?;
        let _ = std::fs::remove_file(&oldest);
        for index in (1..AUDIT_LOG_ROTATIONS).rev() {
            let source = self.rotated_path(index)?;
            let target = self.rotated_path(index + 1)?;
            if !source.exists() {
                continue;
            }
            std::fs::rename(&source, &target).map_err(|error| AppError::FileSystem {
                operation: "rotate audit log".to_string(),
                detail: error.to_string(),
            })?;
        }
        std::fs::rename(&self.path, self.rotated_path(1)?).map_err(|error| {
            AppError::FileSystem {
                operation: "rotate audit log".to_string(),
                detail: error.to_string(),
            }
        })?;
        Ok(true)
    }

    fn read_records(&self) -> Result<Vec<AuditRecord>, AppError> {
        let bytes = match std::fs::read(&self.path) {
            Ok(value) => value,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => {
                return Err(AppError::FileSystem {
                    operation: "read audit log".to_string(),
                    detail: error.to_string(),
                })
            }
        };
        let mut records = Vec::new();
        for line in String::from_utf8_lossy(&bytes).lines() {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<AuditRecord>(line) {
                Ok(record) => records.push(record),
                Err(_) => {
                    // 损坏行不代表任何业务事实，直接丢弃；不尝试恢复参数值。
                }
            }
        }
        Ok(records)
    }

    fn rotated_path(&self, index: usize) -> Result<PathBuf, AppError> {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| self.path.with_file_name(format!("{name}.{index}")))
            .ok_or_else(|| AppError::FileSystem {
                operation: "resolve audit rotation path".to_string(),
                detail: "audit log filename is not valid UTF-8".to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::ipc::{AuditRecord, AuditResult, AuditSource, IpcCommand, IpcRequest};
    use std::collections::BTreeMap;
    use std::os::unix::fs::PermissionsExt;

    fn request() -> IpcRequest {
        IpcRequest {
            request_id: "req_00000000-0000-0000-0000-000000000001".to_string(),
            command: IpcCommand::DataRootSwitch,
            args: BTreeMap::from([("target".to_string(), "/private/fixture".to_string())]),
            confirm: true,
            contract_version: 1,
            secret: None,
        }
    }

    fn record(started: chrono::DateTime<chrono::Utc>) -> AuditRecord {
        AuditRecord::from_request(
            &request(),
            AuditSource::Cli,
            started,
            started + chrono::Duration::seconds(1),
            AuditResult::Succeeded,
            None,
        )
    }

    fn store_with(root: &Path) -> AuditStore {
        std::fs::create_dir_all(root).expect("create data root");
        AuditStore::new(root)
    }

    #[test]
    fn writes_sanitized_record_as_jsonl() {
        let root = tempfile::tempdir().expect("create root");
        let store = store_with(root.path());
        let started = chrono::Utc::now() - chrono::Duration::days(1);
        store.record(&record(started)).expect("write record");
        let payload = std::fs::read_to_string(store.path()).expect("read audit");
        assert!(payload.contains("\"arg_keys\":[\"target\"]"));
        assert!(!payload.contains("/private/fixture"));
        let mode = std::fs::metadata(store.path())
            .expect("audit metadata")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn retention_keeps_ninety_days_and_max_records() {
        let root = tempfile::tempdir().expect("create root");
        let store = store_with(root.path());
        let now = chrono::Utc::now();
        for index in 0..AUDIT_MAX_RECORDS + 4 {
            let started = if index % 2 == 0 {
                now - chrono::Duration::days(91)
            } else {
                now - chrono::Duration::days(1)
            };
            store.record(&record(started)).expect("write record");
        }
        store.retire(now).expect("retire");
        let records = store.read_records().expect("read retained records");
        assert_eq!(records.len(), (AUDIT_MAX_RECORDS + 4).div_ceil(2));

        // 将有效记录推过全局上限，验证维护清理也会裁剪最旧条目。
        let recent = now - chrono::Duration::days(1);
        let overflow_count = AUDIT_MAX_RECORDS - records.len() + 1;
        let mut overflow = String::new();
        for _ in 0..overflow_count {
            overflow.push_str(
                &serde_json::to_string(&record(recent)).expect("serialize overflow record"),
            );
            overflow.push('\n');
        }
        let mut audit = std::fs::OpenOptions::new()
            .append(true)
            .open(store.path())
            .expect("open audit log");
        std::io::Write::write_all(&mut audit, overflow.as_bytes()).expect("append overflow");
        store.retire(now).expect("retire overflow");
        let records = store.read_records().expect("read capped records");
        assert_eq!(records.len(), AUDIT_MAX_RECORDS);
        assert!(records
            .iter()
            .all(|record| record.started_at > (now - chrono::Duration::days(90)).to_rfc3339()));
    }

    #[test]
    fn rotates_five_megabyte_logs_five_times() {
        let root = tempfile::tempdir().expect("create root");
        let store = store_with(root.path());
        let payload = vec![b'a'; AUDIT_LOG_MAX_BYTES as usize];
        std::fs::write(store.path(), &payload).expect("create oversized audit log");
        assert!(store.rotate_if_needed().expect("rotate"));
        assert!(!store.path().exists());
        assert_eq!(
            std::fs::metadata(store.rotated_path(1).unwrap())
                .expect("rotated log")
                .len(),
            AUDIT_LOG_MAX_BYTES
        );

        // Stage four prior rotations and rotate again; the oldest (.5) is removed.
        for index in (1..AUDIT_LOG_ROTATIONS).rev() {
            let source = store.rotated_path(index).unwrap();
            let target = store.rotated_path(index + 1).unwrap();
            if source.exists() {
                std::fs::rename(source, target).expect("stage rotations");
            }
        }
        for index in 1..=AUDIT_LOG_ROTATIONS {
            let path = store.rotated_path(index).unwrap();
            if !path.exists() {
                std::fs::write(path, b"staged").expect("create staged rotation");
            }
        }
        std::fs::write(store.path(), &payload).expect("rewrite");
        assert!(store.rotate_if_needed().expect("rotate again"));
        assert!(!store
            .rotated_path(AUDIT_LOG_ROTATIONS + 1)
            .unwrap()
            .exists());
        assert!(store.rotated_path(AUDIT_LOG_ROTATIONS).unwrap().exists());
    }
}
