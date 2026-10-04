//! 公共错误模型，先稳定结构，后续再扩展错误码与脱敏策略。

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("application is not configured yet")]
    NotConfigured,
    #[error("file system operation failed: {operation}; {detail}")]
    FileSystem { operation: String, detail: String },
    #[error("another application instance is already running")]
    InstanceLockConflict,
    #[error("target file lock timed out after {timeout_ms}ms")]
    TargetLockTimeout { timeout_ms: u32 },
    #[error("request timed out")]
    Timeout,
    #[error("atomic write failed: {reason}")]
    AtomicWrite { reason: String },
    #[error("log sanitization failed")]
    LogSanitization,
    /// 旧版加密容器需要原口令；前端据此提示输入，而非常规导入失败。
    #[error("legacy container requires the original passphrase")]
    PassphraseRequired,
    /// 托管安装 / 卸载失败；`code` 是稳定标识，`detail` 已脱敏（IMP `FZ-47` ~ `FZ-51`）。
    #[error("managed runtime operation failed: {code}; {detail}")]
    RuntimeManaged { code: String, detail: String },
    #[error("{entity} not found")]
    NotFound { entity: String },
    /// 配置文档迁移失败（识别、转换、备份或提交）；`detail` 已脱敏。
    #[error("config migration failed: {detail}")]
    ConfigMigration { detail: String },
    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let payload: AppErrorPayload = self.into();
        payload.serialize(serializer)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppErrorPayload {
    pub code: u16,
    pub message: String,
}

impl From<&AppError> for AppErrorPayload {
    fn from(value: &AppError) -> Self {
        let code = match value {
            AppError::Tauri(_) => 1000,
            AppError::NotConfigured => 1,
            AppError::FileSystem { .. } => 1,
            AppError::InstanceLockConflict => 7,
            AppError::TargetLockTimeout { .. } | AppError::Timeout => 7,
            AppError::AtomicWrite { .. } => 12,
            AppError::LogSanitization => 12,
            AppError::NotFound { .. } => 6,
            AppError::PassphraseRequired => 13,
            AppError::RuntimeManaged { .. } => 14,
            AppError::ConfigMigration { .. } => 15,
        };
        Self {
            code,
            message: value.to_string(),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl Clone for AppError {
    fn clone(&self) -> Self {
        match self {
            Self::NotConfigured => Self::NotConfigured,
            Self::FileSystem { operation, detail } => Self::FileSystem {
                operation: operation.clone(),
                detail: detail.clone(),
            },
            Self::InstanceLockConflict => Self::InstanceLockConflict,
            Self::TargetLockTimeout { timeout_ms } => Self::TargetLockTimeout {
                timeout_ms: *timeout_ms,
            },
            Self::AtomicWrite { reason } => Self::AtomicWrite {
                reason: reason.clone(),
            },
            Self::LogSanitization => Self::LogSanitization,
            Self::PassphraseRequired => Self::PassphraseRequired,
            Self::RuntimeManaged { code, detail } => Self::RuntimeManaged {
                code: code.clone(),
                detail: detail.clone(),
            },
            Self::Timeout => Self::Timeout,
            Self::NotFound { entity } => Self::NotFound {
                entity: entity.clone(),
            },
            Self::ConfigMigration { detail } => Self::ConfigMigration {
                detail: detail.clone(),
            },
            Self::Tauri(error) => {
                Self::Tauri(tauri::Error::Io(std::io::Error::other(error.to_string())))
            }
        }
    }
}

/// 只做领域分类比较；Tauri 内部错误细节不参与相等性。
impl PartialEq for AppError {
    fn eq(&self, other: &Self) -> bool {
        self.to_error_kind() == other.to_error_kind()
    }
}

impl Eq for AppError {}

impl AppError {
    fn to_error_kind(&self) -> &'static str {
        match self {
            Self::NotConfigured => "not-configured",
            Self::FileSystem { .. } => "file-system",
            Self::InstanceLockConflict => "instance-lock-conflict",
            Self::TargetLockTimeout { .. } | Self::Timeout => "target-lock-timeout",
            Self::AtomicWrite { .. } => "atomic-write",
            Self::LogSanitization => "log-sanitization",
            Self::NotFound { .. } => "not-found",
            Self::PassphraseRequired => "passphrase-required",
            Self::RuntimeManaged { .. } => "runtime-managed",
            Self::ConfigMigration { .. } => "config-migration",
            Self::Tauri(_) => "tauri",
        }
    }
}
