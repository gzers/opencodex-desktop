//! 保存之后由用户主动触发的缓存刷新或桌面重启。
//!
//! 仅刷新缓存使用 `ocx sync-cache`；重启使用面板的独立 POST 路由，缓存
//! 没变也必须请求重启。官方路由内部的 syncCatalog 由官方负责，本模块不
//! 另行调用会重写 provider 的 `ocx sync`，也不改写 provider 配置。
//! 重启结果只依据官方的实际桌面退出和重新启动结果，不推断模型是否生效。
use super::{management::Management, schema::*};
use crate::{infrastructure::platform, state::ProcessContext};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

/// 重启 Codex 会结束正在进行的对话，界面必须先说清这一点。
pub const RESTART_WARNING: &str = "会结束当前正在进行的 Codex 对话";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheRefresh {
    /// 只有官方确认桌面退出并重新启动且整个请求成功时才为 true。
    pub restarted: bool,
    /// 所请求的缓存刷新或完整重启是否成功。
    pub success: bool,
    /// 只回报自有脱敏文案，不回显官方输出、路径或任意错误正文。
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RestartResponse {
    success: bool,
    state_before: String,
    synced: bool,
    requested: Vec<u64>,
    stopped: Vec<u64>,
    surviving: Vec<u64>,
    failed: Vec<u64>,
    code: String,
    desktop_app: Option<DesktopRestart>,
}

#[derive(Deserialize)]
struct DesktopRestart {
    attempted: bool,
    stopped: Vec<u64>,
    surviving: Vec<u64>,
    relaunch: String,
    reason: Option<String>,
}

fn valid_pids(pids: &[u64]) -> bool {
    pids.iter()
        .all(|pid| (1..=9_007_199_254_740_991).contains(pid))
}

fn incomplete(reason: &str, summary: &str) -> CacheRefresh {
    CacheRefresh {
        restarted: false,
        success: false,
        summary: summary.into(),
        reason: Some(reason.into()),
    }
}

fn restart_result(value: serde_json::Value) -> ModelResult<CacheRefresh> {
    let malformed = || {
        invalid(
            "cache_restart_response",
            "官方重启响应无效，重启结果未知；可手动重启 Codex",
        )
    };
    let result: RestartResponse = serde_json::from_value(value).map_err(|_| malformed())?;
    // Mirror the official 2.84 response guard, including cross-field invariants.
    // Top-level success describes app-servers, not the desktop app.
    if !["fresh", "stale", "not_running", "unknown"].contains(&result.state_before.as_str())
        || ![
            "stopped",
            "nothing_running",
            "enumeration_unavailable",
            "partially_stopped",
        ]
        .contains(&result.code.as_str())
        || [
            &result.requested,
            &result.stopped,
            &result.surviving,
            &result.failed,
        ]
        .iter()
        .any(|pids| !valid_pids(pids))
        || result.success != (result.code != "partially_stopped")
        || result.success != (result.surviving.is_empty() && result.failed.is_empty())
        || (matches!(
            result.code.as_str(),
            "nothing_running" | "enumeration_unavailable"
        ) && !result.stopped.is_empty())
    {
        return Err(malformed());
    }
    let Some(desktop) = result.desktop_app else {
        return Ok(incomplete(
            "desktop_result_missing",
            "官方未返回桌面重启结果，无法确认完成；可手动重启 Codex",
        ));
    };
    if !valid_pids(&desktop.stopped)
        || !valid_pids(&desktop.surviving)
        || !["started", "skipped"].contains(&desktop.relaunch.as_str())
        || (!desktop.attempted && (!desktop.stopped.is_empty() || !desktop.surviving.is_empty()))
        || (desktop.relaunch == "started"
            && (!desktop.attempted
                || desktop.stopped.is_empty()
                || !desktop.surviving.is_empty()
                || desktop.reason.is_some()))
    {
        return Err(malformed());
    }
    if desktop.relaunch != "started" {
        // Only this closed vocabulary may cross into the UI. Future/free-text
        // reasons receive a fixed fallback instead of leaking local details.
        let (reason, summary) = match desktop.reason.as_deref() {
            Some("unsupported_platform") => (
                "unsupported_platform",
                "当前平台不支持官方桌面重启；请手动重启 Codex",
            ),
            Some("test_environment") => ("test_environment", "官方在测试环境中跳过桌面重启"),
            Some("package_discovery_failed") => (
                "package_discovery_failed",
                "官方未找到 Codex 桌面应用；请手动启动或重启",
            ),
            Some("process_probe_failed") => (
                "process_probe_failed",
                "官方无法读取 Codex 桌面进程，重启未执行；请手动重启",
            ),
            Some("no_targets") => ("no_targets", "未发现正在运行的 Codex 桌面应用，未执行重启"),
            Some("self_ancestry") => (
                "self_ancestry",
                "官方为保护当前 Codex 进程拒绝重启；请手动重启",
            ),
            Some("restart_in_flight") => (
                "restart_in_flight",
                "Codex 正在处理另一项重启请求；请稍后重试",
            ),
            Some("handoff_started") => ("handoff_started", "Codex 重启已交接，尚未确认完成"),
            Some("targets_survived") => (
                "targets_survived",
                "部分 Codex 桌面进程未退出，重启未完成；请手动重启",
            ),
            Some("relaunch_failed") => (
                "relaunch_failed",
                "Codex 桌面进程已退出，但重新启动失败；请手动启动",
            ),
            _ => (
                "desktop_restart_incomplete",
                "Codex 桌面重启未完成；请手动重启",
            ),
        };
        return Ok(incomplete(reason, summary));
    }
    if !result.success {
        return Ok(incomplete(
            "partially_stopped",
            "部分 Codex app-server 未退出，重启未完整完成；请手动重启",
        ));
    }
    if result.code == "enumeration_unavailable" {
        return Ok(incomplete(
            "enumeration_unavailable",
            "官方无法读取 Codex app-server，未能确认完整重启；请手动重启",
        ));
    }
    Ok(CacheRefresh {
        restarted: true,
        success: true,
        summary: if result.synced {
            "模型目录缓存已刷新，Codex 已重启"
        } else {
            "Codex 已重启"
        }
        .into(),
        reason: None,
    })
}

/// `false` 只刷新缓存；`true` 使用官方独立桌面重启路由。
/// 确认在界面层完成，这里只执行用户已请求的动作并如实回报结果。
pub async fn sync_cache(
    context: &ProcessContext,
    restart_codex: bool,
) -> ModelResult<CacheRefresh> {
    if restart_codex {
        let management = Management::bind(context).await?;
        let result = management.restart_codex().await.map_err(|mut error| {
            if error.code == "management_unknown" {
                error.message = "重启请求未完整返回，结果未知；可查看 Codex 或手动重启".into();
            }
            error
        })?;
        return restart_result(result);
    }
    refresh_cache_only(context).await
}

async fn refresh_cache_only(context: &ProcessContext) -> ModelResult<CacheRefresh> {
    let executable = context.executable()?;
    let mut command = tokio::process::Command::new(executable);
    command
        .arg("sync-cache")
        .current_dir(&context.working_directory)
        .env_clear()
        .env("OPENCODEX_HOME", &context.opencodex_home)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    platform::apply_user_environment(
        command.as_std_mut(),
        platform::home_dir().as_deref().map(Path::as_os_str),
    );
    for key in ["PATH", "LANG", "LC_ALL", "HOME"] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    #[cfg(unix)]
    command.process_group(0);
    let child = command
        .spawn()
        .map_err(|_| invalid("cache_refresh", "无法执行模型缓存重建，Codex 仍可用旧列表"))?;
    let output = tokio::time::timeout(Duration::from_secs(60), child.wait_with_output())
        .await
        .map_err(|_| {
            invalid(
                "cache_refresh_timeout",
                "模型缓存重建超时；配置已写入，可稍后手动重启 Codex",
            )
        })?
        .map_err(|_| invalid("cache_refresh", "模型缓存重建结果未知"))?;
    let success = output.status.success();
    let summary = if success {
        "模型缓存已重建".to_string()
    } else {
        "模型缓存重建未成功；配置已写入，可手动重启 Codex".to_string()
    };
    Ok(CacheRefresh {
        restarted: false,
        success,
        summary,
        reason: None,
    })
}

#[cfg(test)]
#[path = "cache_refresh_tests.rs"]
mod tests;
