//! 让 Codex 真正读到新写入的模型目录。
//!
//! 为什么单独一个模块：这是保存之后的**下一步动作**，不是保存的一部分。
//! 用户口径是「写文件 + 重启就肯定生效」，所以这里只做两件事：
//! 1. `ocx sync-cache` 按当前 catalog 重建 `models_cache.json`；
//! 2. 可选 `--restart-codex` 让 Codex 桌面端退出并重启，选择器才会重读。
//!
//! 边界（不可放宽）：
//! - **只用 sync-cache，不用 sync**：后者会重写 provider 配置，等于又写一次配置。
//! - 不判断「是否生效」：重启后由 Codex 自己读，管理器不做比对、不留状态。
//! - 缓存没变时官方自己不会重启进程，这是幂等行为，不是失败。
use super::schema::*;
use crate::{infrastructure::platform, state::ProcessContext};
use serde::Serialize;
use std::path::Path;
use std::time::Duration;

/// 重启 Codex 会结束正在进行的对话，界面必须先说清这一点。
pub const RESTART_WARNING: &str = "会结束当前正在进行的 Codex 对话";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheRefresh {
    /// 是否同时要求重启 Codex 桌面端。
    pub restarted: bool,
    /// 官方命令的退出码是否成功。
    pub success: bool,
    /// 官方标准输出的**脱敏摘要**：只保留我们自己生成的行，不回显原文。
    pub summary: String,
}

fn args(restart_codex: bool) -> Vec<String> {
    let mut args = vec!["sync-cache".to_string()];
    if restart_codex {
        args.push("--restart-codex".into());
    }
    args
}

/// 重建模型缓存；`restart_codex` 为 true 时连桌面端一起重启。
///
/// 这是**用户主动触发**的动作（点「立即重启」），所以不做二次确认——
/// 确认在界面层完成，这里只负责执行与如实回报结果。
pub async fn sync_cache(
    context: &ProcessContext,
    restart_codex: bool,
) -> ModelResult<CacheRefresh> {
    let executable = context.executable()?;
    let mut command = tokio::process::Command::new(executable);
    command
        .args(args(restart_codex))
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
    // 重启桌面端可能等它退出，给足时间但仍然有上界：卡住要能报错，不能永远转圈。
    let budget = if restart_codex { 120 } else { 60 };
    let output = tokio::time::timeout(Duration::from_secs(budget), child.wait_with_output())
        .await
        .map_err(|_| {
            invalid(
                "cache_refresh_timeout",
                "模型缓存重建超时；配置已写入，可稍后手动重启 Codex",
            )
        })?
        .map_err(|_| invalid("cache_refresh", "模型缓存重建结果未知"))?;
    let success = output.status.success();
    // 只回我们自己的结论，不把官方 stdout 原样带进界面（可能含本机路径）。
    let summary = if success {
        if restart_codex {
            "模型缓存已重建，Codex 已重启".to_string()
        } else {
            "模型缓存已重建".to_string()
        }
    } else if restart_codex {
        "模型缓存已重建，但 Codex 重启未成功；可手动重启".to_string()
    } else {
        "模型缓存重建未成功；配置已写入，可手动重启 Codex".to_string()
    };
    Ok(CacheRefresh {
        restarted: restart_codex && success,
        success,
        summary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn always_uses_sync_cache_and_never_sync() {
        // sync 会重写 provider 配置；这条断言锁死「保存之后不许再写一次配置」。
        assert_eq!(args(false), vec!["sync-cache".to_string()]);
        assert_eq!(
            args(true),
            vec!["sync-cache".to_string(), "--restart-codex".to_string()]
        );
        assert!(!args(true).iter().any(|a| a == "sync"));
    }

    #[test]
    fn warning_states_that_live_conversations_end() {
        assert!(RESTART_WARNING.contains("对话"));
    }
}
