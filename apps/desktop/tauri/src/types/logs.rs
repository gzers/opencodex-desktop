//! FZ-14 日志前端 DTO；只传输脱敏后的只读结果。

use crate::modules::logs::{LogFileKind, ReadRecentResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LogKind {
    App,
    Agent,
    Audit,
    Sync,
}

impl From<LogKind> for LogFileKind {
    fn from(value: LogKind) -> Self {
        match value {
            LogKind::App => Self::App,
            LogKind::Agent => Self::Agent,
            LogKind::Audit => Self::Audit,
            LogKind::Sync => Self::Sync,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogsDto {
    pub kind: LogKind,
    pub lines: Vec<String>,
    pub redacted_line_count: usize,
    pub truncated: bool,
    pub file_missing: bool,
    /// 请求的日志文件不存在、实际改读了别的文件时，这里是那个文件名（例如 `app.log`）。
    pub fallback_file: Option<String>,
}

impl LogsDto {
    pub fn from_recent(
        kind: LogKind,
        value: ReadRecentResult,
        fallback_file: Option<String>,
    ) -> Self {
        Self {
            kind,
            lines: value.lines,
            redacted_line_count: value.redacted_line_count,
            truncated: value.truncated,
            file_missing: value.file_missing,
            fallback_file,
        }
    }
}
