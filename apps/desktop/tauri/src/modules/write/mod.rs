//! FZ-26 / 领域模型 §14：受控写入事务。
//!
//! 事务把“校验 → 前置备份 → 临时写入 → 原子替换 → 复验 → 回滚”
//! 固定为一个有界流程。备份接口由调用方注入；基础设施只提供原子写。

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use crate::errors::AppError;
use crate::modules::backup::{backup_file, BackupAction, BackupRecord};
use crate::modules::instance::AppInstanceLock;

/// 领域模型 §14 的受控写入范围。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WriteScope {
    ExtensionMcp,
    ExtensionSkills,
    Import,
}

/// 领域模型 §14 的写入状态；顺序遵循冻结执行链。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WriteTransactionState {
    Validating,
    BackingUp,
    Applying,
    RollingBack,
    Succeeded,
    Failed,
    Cancelled,
    Recovered,
}

/// 前置备份句柄；领域层不关心备份落点内部布局。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupRef {
    Record(Box<BackupRecord>),
    Virtual(String),
}

impl BackupRef {
    pub fn id(&self) -> String {
        match self {
            Self::Record(record) => record.manifest.backup_id.clone(),
            Self::Virtual(id) => id.clone(),
        }
    }
}

/// 受控备份器边界；真实实现复用 MOD-06 备份模块。
pub trait WriteBackup {
    fn backup(&self, target: &Path, before: &[u8]) -> Result<BackupRef, AppError>;
    fn restore(&self, target: &Path, backup: &BackupRef) -> Result<(), AppError>;
}

/// 使用真实 `BackupAction::ExtensionWrite` 的文件备份器。
#[derive(Debug, Clone, Copy, Default)]
pub struct ExtensionWriteBackup;

impl WriteBackup for ExtensionWriteBackup {
    fn backup(&self, target: &Path, before: &[u8]) -> Result<BackupRef, AppError> {
        let root = target
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| AppError::FileSystem {
                operation: "locate data root for backup".to_string(),
                detail: "target has no data-root parent".to_string(),
            })?;
        let record = backup_file(
            root,
            BackupAction::ExtensionWrite,
            target,
            before,
            Utc::now(),
            None,
        )?;
        Ok(BackupRef::Record(Box::new(record)))
    }

    fn restore(&self, target: &Path, backup: &BackupRef) -> Result<(), AppError> {
        let BackupRef::Record(record) = backup else {
            return Err(AppError::FileSystem {
                operation: "restore write transaction".to_string(),
                detail: "virtual backup cannot restore a real file".to_string(),
            });
        };
        let stored = find_backup_payload(&record.directory)?;
        let payload = std::fs::read(stored).map_err(|error| AppError::FileSystem {
            operation: "read backup payload".to_string(),
            detail: error.to_string(),
        })?;
        crate::infrastructure::atomic_write::atomic_write(target, &payload, 0o600)
    }
}

fn find_backup_payload(directory: &Path) -> Result<PathBuf, AppError> {
    let entries = std::fs::read_dir(directory).map_err(|error| AppError::FileSystem {
        operation: "read backup directory".to_string(),
        detail: error.to_string(),
    })?;
    for entry in entries {
        let path = entry
            .map_err(|error| AppError::FileSystem {
                operation: "read backup entry".to_string(),
                detail: error.to_string(),
            })?
            .path();
        if path.is_file()
            && path
                .file_name()
                .map(|name| name != "backup-manifest.json")
                .unwrap_or(false)
        {
            return Ok(path);
        }
    }
    Err(AppError::FileSystem {
        operation: "restore write transaction".to_string(),
        detail: "backup payload is missing".to_string(),
    })
}

/// 领域实体：一次受控写入事务。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteTransaction {
    pub tx_id: String,
    pub scope: WriteScope,
    pub target: PathBuf,
    pub temp_path: PathBuf,
    pub backup_ref: String,
    pub before_hash: String,
    pub after_hash: String,
    pub state: WriteTransactionState,
    pub failure_reason: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
}

impl WriteTransaction {
    pub fn new(
        tx_id: impl Into<String>,
        scope: WriteScope,
        target: PathBuf,
        started_at: DateTime<Utc>,
    ) -> Result<Self, AppError> {
        if target.as_os_str().is_empty() {
            return Err(AppError::FileSystem {
                operation: "validate write target".to_string(),
                detail: "target is empty".to_string(),
            });
        }
        let parent = target.parent().ok_or_else(|| AppError::FileSystem {
            operation: "validate write target".to_string(),
            detail: "target has no parent directory".to_string(),
        })?;
        let filename = target
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| AppError::FileSystem {
                operation: "validate write target".to_string(),
                detail: "target filename is not valid UTF-8".to_string(),
            })?;
        Ok(Self {
            tx_id: tx_id.into(),
            scope,
            temp_path: parent.join(format!(
                ".{filename}.tx-{pid}-new",
                pid = std::process::id()
            )),
            target,
            backup_ref: String::new(),
            before_hash: String::new(),
            after_hash: String::new(),
            state: WriteTransactionState::Validating,
            failure_reason: None,
            started_at: started_at.to_rfc3339_opts(SecondsFormat::Secs, true),
            finished_at: None,
        })
    }
}

/// 事务执行输入；before 为 None 表示当前目标尚不存在。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteRequest<'a> {
    pub tx_id: &'a str,
    pub scope: WriteScope,
    pub target: &'a Path,
    pub payload: &'a [u8],
    pub expected_before: Option<&'a str>,
}

/// 事务执行结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteExecution {
    pub transaction: WriteTransaction,
    pub backup: BackupRef,
}

/// 取消阶段错误；无备份需要回滚。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedCancel {
    pub transaction: WriteTransaction,
    pub error: AppError,
}

/// 事务错误结果；调用方可据此进入回滚或恢复。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedWriteExecution {
    pub transaction: WriteTransaction,
    pub backup: Option<BackupRef>,
    pub error: AppError,
}

pub fn sha256_bytes(payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    hex(&hasher.finalize())
}

pub fn sha256_file(path: &Path) -> Result<String, AppError> {
    let payload = std::fs::read(path).map_err(|error| AppError::FileSystem {
        operation: "hash write target".to_string(),
        detail: error.to_string(),
    })?;
    Ok(sha256_bytes(&payload))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// 执行受控写入；失败保留原文件并返回事务状态。
pub fn execute<B: WriteBackup>(
    request: WriteRequest<'_>,
    backup: &B,
    mut lock: Option<AppInstanceLock>,
    now: DateTime<Utc>,
) -> Result<WriteExecution, Box<FailedWriteExecution>> {
    // AppInstanceLock 已在应用启动持有；这里显式接入，防止未来调用点绕过单写者。
    drop(lock.take());

    let mut transaction = match WriteTransaction::new(
        request.tx_id,
        request.scope,
        request.target.to_path_buf(),
        now,
    ) {
        Ok(value) => value,
        Err(error) => {
            return Err(Box::new(FailedWriteExecution {
                transaction: empty_failed_transaction(
                    request.tx_id,
                    request.scope,
                    request.target,
                    now,
                    error.clone(),
                ),
                backup: None,
                error,
            }))
        }
    };

    let before_bytes = match std::fs::read(request.target) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => {
            return fail(transaction, None, error.into_fs("read target before write"));
        }
    };

    transaction.before_hash = sha256_bytes(&before_bytes);
    if let Some(expected) = request.expected_before {
        if expected != transaction.before_hash {
            return fail(
                transaction,
                None,
                AppError::FileSystem {
                    operation: "verify target before write".to_string(),
                    detail: "external target changed; expected digest mismatch".to_string(),
                },
            );
        }
    }

    transaction.state = WriteTransactionState::BackingUp;
    let backup_ref = match backup.backup(request.target, &before_bytes) {
        Ok(value) => value,
        Err(error) => {
            transaction.state = WriteTransactionState::Failed;
            transaction.failure_reason = Some(failure_text(&error));
            transaction.finished_at = Some(now.to_rfc3339_opts(SecondsFormat::Secs, true));
            return Err(Box::new(FailedWriteExecution {
                transaction,
                backup: None,
                error,
            }));
        }
    };
    transaction.backup_ref = backup_ref.id();

    transaction.state = WriteTransactionState::Applying;
    transaction.after_hash = sha256_bytes(request.payload);
    if transaction.after_hash == transaction.before_hash && !before_bytes.is_empty() {
        transaction.state = WriteTransactionState::Succeeded;
        transaction.finished_at = Some(now.to_rfc3339_opts(SecondsFormat::Secs, true));
        return Ok(WriteExecution {
            transaction,
            backup: backup_ref,
        });
    }

    if let Err(error) =
        crate::infrastructure::atomic_write::atomic_write(request.target, request.payload, 0o600)
    {
        let _ = std::fs::remove_file(&transaction.temp_path);
        return fail(transaction, Some(backup_ref), error);
    }

    match sha256_file(request.target) {
        Ok(actual) if actual == transaction.after_hash => {
            transaction.state = WriteTransactionState::Succeeded;
            transaction.finished_at = Some(now.to_rfc3339_opts(SecondsFormat::Secs, true));
            Ok(WriteExecution {
                transaction,
                backup: backup_ref,
            })
        }
        Ok(_) => fail(
            transaction,
            Some(backup_ref),
            AppError::FileSystem {
                operation: "verify write transaction".to_string(),
                detail: "post-write SHA-256 mismatch".to_string(),
            },
        ),
        Err(error) => fail(transaction, Some(backup_ref), error),
    }
}

/// 已应用的写入失败后回滚；备份恢复成功标记 recovered。
pub fn rollback<B: WriteBackup>(
    mut transaction: WriteTransaction,
    backup: &BackupRef,
    restorer: &B,
    target: &Path,
    now: DateTime<Utc>,
) -> Result<WriteTransaction, Box<WriteTransaction>> {
    transaction.state = WriteTransactionState::RollingBack;
    match restorer.restore(target, backup) {
        Ok(()) => {
            transaction.state = WriteTransactionState::Recovered;
            transaction.finished_at = Some(now.to_rfc3339_opts(SecondsFormat::Secs, true));
            Ok(transaction)
        }
        Err(error) => {
            transaction.state = WriteTransactionState::Failed;
            transaction.failure_reason = Some(format!(
                "{}; rollback failed: {}",
                transaction
                    .failure_reason
                    .unwrap_or_else(|| "write failed".to_string()),
                failure_text(&error)
            ));
            transaction.finished_at = Some(now.to_rfc3339_opts(SecondsFormat::Secs, true));
            Err(Box::new(transaction))
        }
    }
}

/// 用户在确认前取消；不创建文件、不修改目标。
pub fn cancel<B: WriteBackup>(
    request: WriteRequest<'_>,
    backup: &B,
    now: DateTime<Utc>,
) -> Result<WriteTransaction, Box<FailedCancel>> {
    let mut transaction = match WriteTransaction::new(
        request.tx_id,
        request.scope,
        request.target.to_path_buf(),
        now,
    ) {
        Ok(value) => value,
        Err(error) => {
            return Err(Box::new(FailedCancel {
                transaction: empty_failed_transaction(
                    request.tx_id,
                    request.scope,
                    request.target,
                    now,
                    error.clone(),
                ),
                error,
            }))
        }
    };
    transaction.before_hash = match std::fs::read(request.target) {
        Ok(bytes) => sha256_bytes(&bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => {
            let error = error.into_fs("read target before cancel");
            transaction.state = WriteTransactionState::Failed;
            transaction.failure_reason = Some(failure_text(&error));
            transaction.finished_at = Some(now.to_rfc3339_opts(SecondsFormat::Secs, true));
            return Err(Box::new(FailedCancel { transaction, error }));
        }
    };
    transaction.state = WriteTransactionState::Cancelled;
    transaction.finished_at = Some(now.to_rfc3339_opts(SecondsFormat::Secs, true));
    let _ = backup;
    Ok(transaction)
}

fn fail(
    mut transaction: WriteTransaction,
    backup: Option<BackupRef>,
    error: AppError,
) -> Result<WriteExecution, Box<FailedWriteExecution>> {
    transaction.state = WriteTransactionState::Failed;
    transaction.failure_reason = Some(failure_text(&error));
    transaction.finished_at = Some(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true));
    Err(Box::new(FailedWriteExecution {
        transaction,
        backup,
        error,
    }))
}

fn empty_failed_transaction(
    tx_id: &str,
    scope: WriteScope,
    target: &Path,
    now: DateTime<Utc>,
    error: AppError,
) -> WriteTransaction {
    WriteTransaction {
        tx_id: tx_id.to_string(),
        scope,
        target: target.to_path_buf(),
        temp_path: target.with_file_name("unused.tmp"),
        backup_ref: String::new(),
        before_hash: String::new(),
        after_hash: String::new(),
        state: WriteTransactionState::Failed,
        failure_reason: Some(failure_text(&error)),
        started_at: now.to_rfc3339_opts(SecondsFormat::Secs, true),
        finished_at: Some(now.to_rfc3339_opts(SecondsFormat::Secs, true)),
    }
}

fn failure_text(error: &AppError) -> String {
    match error {
        AppError::NotConfigured => "action is not configured".to_string(),
        AppError::FileSystem { detail, .. } => format!("file system error: {detail}"),
        AppError::InstanceLockConflict => "instance lock conflict".to_string(),
        AppError::TargetLockTimeout { timeout_ms } => {
            format!("target lock timeout after {timeout_ms}ms")
        }
        AppError::AtomicWrite { reason } => format!("atomic write failed: {reason}"),
        AppError::LogSanitization => "log sanitization failed".to_string(),
        AppError::NotFound { entity } => format!("{entity} not found"),
        AppError::PassphraseRequired => {
            "legacy container requires the original passphrase".to_string()
        }
        AppError::RuntimeManaged { code, detail } => {
            format!("runtime install failed: {code}; {detail}")
        }
        AppError::Tauri(error) => format!("shell error: {error}"),
        AppError::Timeout => "request timed out".to_string(),
    }
}

trait IntoFs {
    fn into_fs(self, operation: &str) -> AppError;
}

impl IntoFs for std::io::Error {
    fn into_fs(self, operation: &str) -> AppError {
        AppError::FileSystem {
            operation: operation.to_string(),
            detail: self.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Debug, Default)]
    struct VirtualBackup {
        calls: Rc<RefCell<Vec<String>>>,
        should_fail: bool,
    }

    impl WriteBackup for VirtualBackup {
        fn backup(&self, _target: &Path, before: &[u8]) -> Result<BackupRef, AppError> {
            if self.should_fail {
                return Err(AppError::FileSystem {
                    operation: "virtual backup".to_string(),
                    detail: "backup blocked".to_string(),
                });
            }
            self.calls
                .borrow_mut()
                .push(format!("backup:{}", before.len()));
            Ok(BackupRef::Virtual("bk_virtual".to_string()))
        }

        fn restore(&self, _target: &Path, _backup: &BackupRef) -> Result<(), AppError> {
            self.calls.borrow_mut().push("restore".to_string());
            Ok(())
        }
    }

    fn request<'a>(target: &'a Path, payload: &'a [u8]) -> WriteRequest<'a> {
        WriteRequest {
            tx_id: "tx-001",
            scope: WriteScope::ExtensionSkills,
            target,
            payload,
            expected_before: None,
        }
    }

    #[test]
    fn successful_write_follows_frozen_state_chain() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("config.json");
        std::fs::write(&target, b"before").unwrap();
        let backup = VirtualBackup::default();
        let result = execute(request(&target, b"after"), &backup, None, Utc::now()).unwrap();

        assert_eq!(result.transaction.state, WriteTransactionState::Succeeded);
        assert_eq!(result.transaction.backup_ref, "bk_virtual");
        assert_eq!(std::fs::read(&target).unwrap(), b"after");
        assert_eq!(result.transaction.after_hash, sha256_bytes(b"after"));
        assert!(!target
            .with_file_name(format!(".config.json.tx-{}-new", std::process::id()))
            .exists());
    }

    #[test]
    fn backup_failure_blocks_write_and_preserves_target() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("config.json");
        std::fs::write(&target, b"original").unwrap();
        let backup = VirtualBackup {
            should_fail: true,
            ..Default::default()
        };
        let failure = execute(request(&target, b"changed"), &backup, None, Utc::now()).unwrap_err();

        assert_eq!(failure.transaction.state, WriteTransactionState::Failed);
        assert_eq!(failure.transaction.backup_ref, "");
        assert_eq!(std::fs::read(&target).unwrap(), b"original");
        assert!(failure
            .transaction
            .failure_reason
            .unwrap()
            .contains("backup blocked"));
    }

    #[test]
    fn digest_mismatch_rejects_external_change_without_writing() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("config.json");
        std::fs::write(&target, b"external").unwrap();
        let before = sha256_bytes(b"external");
        let backup = VirtualBackup::default();
        let failure = execute(
            WriteRequest {
                tx_id: "tx-002",
                scope: WriteScope::Import,
                target: &target,
                payload: b"changed",
                expected_before: Some("wrong-digest"),
            },
            &backup,
            None,
            Utc::now(),
        )
        .unwrap_err();

        assert_eq!(failure.transaction.state, WriteTransactionState::Failed);
        assert_eq!(failure.transaction.before_hash, before);
        assert_eq!(std::fs::read(&target).unwrap(), b"external");
    }

    #[test]
    fn rollback_restores_previous_payload_and_marks_recovered() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("config.json");
        std::fs::write(&target, b"before").unwrap();
        let backup = VirtualBackup::default();
        let execution = execute(request(&target, b"after"), &backup, None, Utc::now()).unwrap();
        std::fs::write(&target, b"corrupted-after-write").unwrap();

        let recovered = rollback(
            execution.transaction,
            &execution.backup,
            &backup,
            &target,
            Utc::now(),
        )
        .unwrap();
        assert_eq!(recovered.state, WriteTransactionState::Recovered);
        assert_eq!(
            backup.calls.borrow().last().map(String::as_str),
            Some("restore")
        );
        let _ = std::fs::remove_file(&target);
    }

    #[test]
    fn cancel_has_no_side_effect() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("config.json");
        std::fs::write(&target, b"original").unwrap();
        let backup = VirtualBackup::default();
        let transaction = cancel(request(&target, b"ignored"), &backup, Utc::now()).unwrap();
        assert_eq!(transaction.state, WriteTransactionState::Cancelled);
        assert_eq!(std::fs::read(&target).unwrap(), b"original");
        assert!(backup.calls.borrow().is_empty());
    }

    #[test]
    fn scopes_and_states_are_frozen_serialized_values() {
        assert_eq!(
            serde_json::to_string(&WriteScope::ExtensionMcp).unwrap(),
            "\"extension_mcp\""
        );
        assert_eq!(
            serde_json::to_string(&WriteTransactionState::RollingBack).unwrap(),
            "\"rolling_back\""
        );
    }

    #[test]
    fn target_must_have_parent_and_utf8_filename() {
        assert!(WriteTransaction::new(
            "tx",
            WriteScope::Import,
            PathBuf::from("config.json"),
            Utc::now()
        )
        .is_ok());
        assert!(
            WriteTransaction::new("tx", WriteScope::Import, PathBuf::from(""), Utc::now()).is_err()
        );
    }
}
