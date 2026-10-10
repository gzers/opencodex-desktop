//! Tauri 日志命令层。只校验请求并调用模块，不访问文件系统。

use crate::errors::{AppError, AppResult};
use crate::modules::logs::{self, LogFileKind, AUDIT_LOG_FILE_NAME};
use crate::state::SharedDataRoot;
use crate::types::logs::{LogKind, LogsDto};
use std::path::{Path, PathBuf};

#[tauri::command]
pub async fn read_logs(
    kind: LogKind,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> AppResult<LogsDto> {
    let root = data_root.0.clone();
    crate::commands::run_readonly("read logs", move || read_logs_with_root(kind, &root)).await
}

pub fn read_logs_with_root(kind: LogKind, data_root: &std::path::Path) -> AppResult<LogsDto> {
    let requested = LogFileKind::from(kind);
    let recent = read_kind(data_root, requested)?;
    // 只有 `agent.log` 因尚无写入方而回退到应用日志；分类日志（尤其审计）必须如实报「不存在」，
    // 否则会把 app.log 的内容显示成另一个分类。
    if !recent.file_missing || requested != LogFileKind::Agent {
        return Ok(LogsDto::from_recent(kind, recent, None));
    }

    // 请求的日志还不存在时回退到应用日志：否则会出现「磁盘上有内容、界面却显示 0 行」。
    // 目前 `agent.log` 尚无写入方（托管代理输出未落盘），风险/失败态正走这条回退。
    let fallback = read_kind(data_root, LogFileKind::App)?;
    if fallback.file_missing {
        return Ok(LogsDto::from_recent(kind, recent, None));
    }
    Ok(LogsDto::from_recent(
        kind,
        fallback,
        Some(LogFileKind::App.file_name().to_string()),
    ))
}

/// 审计日志写在**活跃数据根**的根目录（与 `AuditStore` 同路径），不在 `logs/` 分区下。
fn audit_log_path(data_root: &Path) -> PathBuf {
    crate::modules::data_root::load_runtime_config(data_root)
        .map(|config| config.active_data_root)
        .unwrap_or_else(|_| data_root.to_path_buf())
        .join(AUDIT_LOG_FILE_NAME)
}

fn read_kind(data_root: &std::path::Path, kind: LogFileKind) -> AppResult<logs::ReadRecentResult> {
    let path = if kind == LogFileKind::Audit {
        audit_log_path(data_root)
    } else {
        data_root.join("logs").join(kind.file_name())
    };
    logs::read_recent(path, logs::default_recent_lines()).map_err(|_error| AppError::FileSystem {
        operation: "read log".to_string(),
        detail: "log is not readable".to_string(),
    })
}
