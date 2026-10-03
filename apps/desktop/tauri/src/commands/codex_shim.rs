//! 「随 Codex 启动 OpenCodex」命令层：只锁共享来源并投影 DTO。
//!
//! 读写都经官方 CLI（`ocx codex-shim`），命令层不访问文件系统或进程细节，
//! 也不直接改写官方配置（AC-11）。

use std::sync::Arc;

use crate::errors::{AppError, AppResult};
use crate::modules::codex_shim::{CodexShimError, CodexShimReport};
use crate::state::SharedCodexShimSource;
use crate::types::codex_shim::CodexShimDto;

/// 只读：读取官方 shim 当前状态；来源不可达时返回 `unreachable` 事实而非报错。
#[tauri::command]
pub async fn codex_shim_status(
    source: tauri::State<'_, SharedCodexShimSource>,
) -> AppResult<CodexShimDto> {
    let source = Arc::clone(&source);
    crate::commands::run_blocking("read codex shim status", move || {
        Ok(status_with_source(&source))
    })
    .await
}

/// 写入：经官方 CLI 安装 / 卸载 shim，并回读官方状态。
#[tauri::command]
pub async fn set_codex_shim(
    enabled: bool,
    source: tauri::State<'_, SharedCodexShimSource>,
) -> AppResult<CodexShimDto> {
    let source = Arc::clone(&source);
    crate::commands::run_blocking("set codex shim", move || set_with_source(&source, enabled)).await
}

pub fn status_with_source(source: &SharedCodexShimSource) -> CodexShimDto {
    let guard = match source.lock() {
        Ok(guard) => guard,
        Err(_poisoned) => return CodexShimDto::unreachable("locked"),
    };
    match guard.status() {
        Ok(output) => CodexShimReport::from_output(&output).into(),
        Err(error) => CodexShimDto::unreachable(reason_code(&error)),
    }
}

/// 稳定原因码；界面据此给中文说明（不把官方英文原文丢给用户）。
fn reason_code(error: &CodexShimError) -> &'static str {
    match error {
        CodexShimError::Unreachable => "unresolved",
        CodexShimError::Timeout => "timeout",
        CodexShimError::Failed => "failed",
        CodexShimError::Rejected(_) => "rejected",
    }
}

pub fn set_with_source(source: &SharedCodexShimSource, enabled: bool) -> AppResult<CodexShimDto> {
    let guard = source.lock().map_err(|_poisoned| AppError::NotConfigured)?;
    let output = guard.set_enabled(enabled).map_err(AppError::from)?;
    Ok(CodexShimReport::from_output(&output).into())
}
