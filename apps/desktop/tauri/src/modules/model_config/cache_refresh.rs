//! 保存之后由用户主动触发的缓存刷新或桌面重启。
//!
//! 仅刷新缓存使用 `ocx sync-cache`；重启使用面板的独立 POST 路由，缓存
//! 没变也必须请求重启。官方路由内部的 syncCatalog 由官方负责，本模块不
//! 另行调用会重写 provider 的 `ocx sync`，也不改写 provider 配置。
//! 重启结果只依据官方的实际桌面退出和重新启动结果，不推断模型是否生效。
use super::{management::Management, schema::*};
use crate::{infrastructure::platform, state::ProcessContext};
use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::path::Path;
use std::time::Duration;
#[path = "client_npm_runtime.rs"]
pub mod client_npm_runtime;
#[path = "client_runtime_readiness.rs"]
mod client_runtime_readiness;
#[path = "client_selection_rename.rs"]
pub mod client_selection_rename;
#[path = "saved_runtime_reconcile.rs"]
pub mod saved_runtime_reconcile;

#[derive(Clone)]
struct RenamePreparation {
    id: String,
    file_digest: String,
    mapping: std::collections::BTreeMap<String, String>,
}
fn read_rename_preparation(
    root: &std::path::Path,
    context: &ProcessContext,
) -> ModelResult<Option<RenamePreparation>> {
    use super::config_file::{self, rename_effects};
    let Some(pending) = rename_effects::load(root, &context.opencodex_home)? else {
        return Ok(None);
    };
    if !context
        .opencodex_home
        .join("config.json")
        .try_exists()
        .map_err(|_| {
            invalid(
                "restart_saved_config_unavailable",
                "无法读取已保存配置 config.json；请核对当前数据目录后重试",
            )
        })?
    {
        return Err(invalid("restart_saved_config_missing", "已保存配置 config.json 已不存在；请重新读取并核对配置，当前待处理改名记录已保留，尚未重启"));
    }
    let snapshot = config_file::snapshot(root, &context.opencodex_home)?;
    Ok(Some(RenamePreparation {
        id: pending.id.clone(),
        file_digest: snapshot.policy_digest,
        mapping: rename_effects::current_map(&pending, &snapshot.policy)?,
    }))
}
fn verify_rename_preparation(
    root: &std::path::Path,
    context: &ProcessContext,
    preparation: &RenamePreparation,
) -> ModelResult<()> {
    let now = read_rename_preparation(root, context)?;
    if now.as_ref().is_none_or(|now| {
        now.id != preparation.id
            || now.file_digest != preparation.file_digest
            || now.mapping != preparation.mapping
    }) {
        return Err(invalid(
            "rename_activation_changed",
            "重启准备期间保存配置又有变化，未重启；请重新核对",
        ));
    }
    Ok(())
}
async fn verify_rename_preparation_worker(
    root: &std::path::Path,
    context: &ProcessContext,
    preparation: &RenamePreparation,
) -> ModelResult<()> {
    let root = root.to_path_buf();
    let context = context.clone();
    let preparation = preparation.clone();
    tauri::async_runtime::spawn_blocking(move || {
        verify_rename_preparation(&root, &context, &preparation)
    })
    .await
    .map_err(|_| invalid("rename_activation_worker", "改名准备核对中断，未确认完成"))?
}
struct ClientBinding {
    executable: std::path::PathBuf,
    home: std::path::PathBuf,
    codex_home: std::path::PathBuf,
}
// The upstream settings DTO deliberately displays C:\Users\[USER] (and
// /Users/[USER]) instead of its machine path. Restore only the username
// component proved by this already-bound client's unredacted paths. Never
// expand a masked arbitrary segment, another drive, or an ambiguous identity.
fn bound_runtime_display_path(raw: &str, anchors: &[&str]) -> Option<String> {
    if raw.contains("[REDACTED]") || raw.chars().any(char::is_control) {
        return None;
    }
    let Some(at) = raw.find("[USER]") else {
        return Some(raw.to_owned());
    };
    if raw.matches("[USER]").count() != 1 {
        return None;
    }
    let prefix = &raw[..at];
    let suffix = &raw[at + "[USER]".len()..];
    let windows = prefix.len() == 9
        && prefix.as_bytes()[0].is_ascii_alphabetic()
        && prefix[1..].eq_ignore_ascii_case(r":\Users\");
    if !(windows || matches!(prefix, "/Users/" | "/home/"))
        || !suffix.starts_with(if windows { '\\' } else { '/' })
    {
        return None;
    }
    let mut user = None;
    for anchor in anchors {
        let head = anchor.get(..prefix.len());
        let same = head.is_some_and(|head| {
            if windows {
                head.eq_ignore_ascii_case(prefix)
            } else {
                head == prefix
            }
        });
        if !same {
            continue;
        }
        let value = anchor[prefix.len()..]
            .split(if windows { '\\' } else { '/' })
            .next()?;
        if value.is_empty() || value.contains(['[', ']', '/', '\\']) {
            return None;
        }
        if user.is_some_and(|before: &str| {
            if windows {
                !before.eq_ignore_ascii_case(value)
            } else {
                before != value
            }
        }) {
            return None;
        }
        user = Some(value);
    }
    Some(format!("{prefix}{}{suffix}", user?))
}
fn verify_client_route(binding: &ClientBinding, management_base: &str) -> ModelResult<String> {
    #[derive(Deserialize, Default)]
    struct Provider {
        base_url: Option<String>,
    }
    #[derive(Deserialize, Default)]
    struct Routing {
        openai_base_url: Option<String>,
        model_provider: Option<String>,
        #[serde(default)]
        model_providers: std::collections::BTreeMap<String, Provider>,
    }
    let fail = || {
        invalid(
            "rename_client_route",
            "当前Codex配置未绑定此运行实例，未改写当前选择；请核对客户端来源",
        )
    };
    let path = binding.codex_home.join("config.toml");
    let unavailable = || {
        invalid(
            "rename_client_config_unavailable",
            "Codex 客户端配置 config.toml 缺失或不可安全读取；请核对客户端绑定后重试，尚未重启",
        )
    };
    crate::modules::backup::safety::check_path(&binding.codex_home, &path, false)
        .map_err(|_| unavailable())?;
    let bytes = zeroize::Zeroizing::new(
        crate::modules::backup::safety::read_file(&path, 2 * 1024 * 1024)
            .map_err(|_| unavailable())?,
    );
    let routing: Routing =
        toml::from_str(std::str::from_utf8(&bytes).map_err(|_| fail())?).map_err(|_| fail())?;
    let selected = routing.model_provider.as_deref().unwrap_or("openai");
    let raw = if let Some(provider) = routing.model_providers.get(selected) {
        provider.base_url.as_deref()
    } else if selected == "openai" {
        routing.openai_base_url.as_deref()
    } else {
        None
    }
    .ok_or_else(fail)?;
    let actual = reqwest::Url::parse(raw).map_err(|_| fail())?;
    let expected = reqwest::Url::parse(management_base).map_err(|_| fail())?;
    if actual.scheme() != "http"
        || actual.host_str() != expected.host_str()
        || actual.port_or_known_default() != expected.port_or_known_default()
        || actual.path().trim_end_matches('/') != "/v1"
        || !actual.username().is_empty()
        || actual.password().is_some()
        || actual.query().is_some()
        || actual.fragment().is_some()
    {
        return Err(fail());
    }
    Ok(super::store::sha256(&bytes))
}
fn client_binding(
    settings: &serde_json::Value,
    integration: &serde_json::Value,
) -> ModelResult<Option<ClientBinding>> {
    use std::path::{Component, Path, PathBuf};
    let invalid_binding = || {
        invalid(
            "rename_client_binding",
            "官方客户端路径或运行身份不明确，未更新当前选择，请核对Codex安装",
        )
    };
    if integration["clientId"] != "codex" {
        return Err(invalid_binding());
    }
    let config_path = PathBuf::from(
        integration["configPath"]
            .as_str()
            .ok_or_else(invalid_binding)?,
    );
    if !config_path.is_absolute()
        || config_path.file_name() != Some(std::ffi::OsStr::new("config.toml"))
        || config_path
            .components()
            .any(|c| matches!(c, Component::ParentDir))
    {
        return Err(invalid_binding());
    }
    if !config_path.try_exists().map_err(|_| invalid_binding())? {
        if matches!(integration["state"].as_str(), Some("current" | "stale"))
            && integration["installed"] == true
        {
            return Err(invalid("rename_client_config_missing", "已绑定的 Codex 客户端配置 config.toml 已不存在；请核对客户端安装与配置后重试，尚未重启"));
        }
        return Ok(None);
    }
    if !matches!(integration["state"].as_str(), Some("current" | "stale"))
        || integration["installed"] != true
    {
        return Err(invalid(
            "rename_client_ownership",
            "当前Codex配置来源未确认或有外部冲突，未改写当前选择",
        ));
    }
    let codex_home = config_path
        .parent()
        .ok_or_else(invalid_binding)?
        .to_path_buf();
    let home = if codex_home.file_name() == Some(std::ffi::OsStr::new(".codex")) {
        codex_home.parent().map(Path::to_path_buf)
    } else {
        platform::home_dir()
    }
    .ok_or_else(invalid_binding)?;
    let runtime = &settings["codexRuntime"];
    if runtime["version"].as_str().is_none_or(|v| v.is_empty()) {
        return Err(invalid_binding());
    }
    let raw = runtime["path"].as_str().ok_or_else(invalid_binding)?;
    let raw = bound_runtime_display_path(
        raw,
        &[
            home.to_str().ok_or_else(invalid_binding)?,
            config_path.to_str().ok_or_else(invalid_binding)?,
        ],
    )
    .ok_or_else(invalid_binding)?;
    let executable =
        if let Some(relative) = raw.strip_prefix("~/").or_else(|| raw.strip_prefix("~\\")) {
            home.join(relative)
        } else {
            PathBuf::from(raw)
        };
    if !executable.is_absolute()
        || executable
            .components()
            .any(|c| matches!(c, Component::ParentDir))
        || !executable.is_file()
    {
        return Err(invalid_binding());
    }
    let npm_windows = cfg!(windows)
        && executable
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("cmd"));
    let npm_macos = cfg!(target_os = "macos")
        && std::fs::symlink_metadata(&executable)
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
        && dunce::canonicalize(&executable)
            .is_ok_and(|path| path.extension().is_some_and(|extension| extension == "js"));
    let executable = if npm_windows || npm_macos {
        client_npm_runtime::resolve_npm_native(
            &executable,
            runtime["version"].as_str().ok_or_else(invalid_binding)?,
        )?
    } else {
        executable
    };
    Ok(Some(ClientBinding {
        executable,
        home,
        codex_home,
    }))
}
/// Preparation for the existing explicit restart action. This never invokes the
/// global desktop restart itself, so isolated acceptance can test real owners.
/// Pending is cleared only by sync_cache_after_save after confirmed restart.
pub async fn prepare_saved_rename_restart(
    root: &std::path::Path,
    context: &ProcessContext,
) -> ModelResult<Option<String>> {
    let owned_root = root.to_path_buf();
    let owned_context = context.clone();
    let preparation = tauri::async_runtime::spawn_blocking(move || {
        read_rename_preparation(&owned_root, &owned_context)
    })
    .await
    .map_err(|_| invalid("rename_activation_worker", "读取改名准备中断，未重启Codex"))??;
    let Some(preparation) = preparation else {
        return Ok(None);
    };
    let management = Management::bind(context).await?;
    let selection = if preparation.mapping.is_empty() {
        None
    } else {
        let settings = client_runtime_readiness::read_settings(&management).await?;
        let integration = management.codex_integration().await?;
        let binding = client_binding(&settings, &integration)?;
        match binding {
            None => None,
            Some(binding) => {
                let plan = client_selection_rename::preflight_current_selection(
                    context,
                    &binding.executable,
                    &binding.home,
                    &binding.codex_home,
                    &binding.home,
                    None,
                    &preparation.mapping,
                )
                .await?;
                if plan.has_changes() {
                    verify_client_route(&binding, management.base_url())?;
                }
                Some((plan, binding))
            }
        }
    };
    verify_rename_preparation_worker(root, context, &preparation).await?;
    saved_runtime_reconcile::reconcile(&management, context, &preparation.mapping).await?;
    verify_rename_preparation_worker(root, context, &preparation).await?;
    if let Some((selection, binding)) = selection {
        if selection.has_changes() {
            let raw_digest = verify_client_route(&binding, management.base_url())?;
            selection.apply_with_file_snapshot(raw_digest).await?;
        } else {
            selection.apply().await?;
        }
    }
    verify_rename_preparation_worker(root, context, &preparation).await?;
    management.verify_binding()?;
    Ok(Some(preparation.id))
}
/// File success and activation/restart are distinct. Failure keeps the exact
/// intent for a later explicit retry; cache-only remains unchanged.
pub async fn sync_cache_after_save(
    root: &std::path::Path,
    context: &ProcessContext,
    restart_codex: bool,
) -> ModelResult<CacheRefresh> {
    if !restart_codex {
        return sync_cache(context, false).await;
    }
    let prepared = prepare_saved_rename_restart(root, context)
        .await
        .map_err(|mut error| {
            error.message = format!(
                "配置已写入，但当前引用准备未完成，尚未重启 Codex：{}",
                error.message
            );
            error
        })?;
    let result = sync_cache(context, true).await.map_err(|mut error| {
        error.message = format!("配置已写入，但重启尚未完成：{}", error.message);
        error
    })?;
    if result.success && result.restarted {
        if let Some(id) = prepared {
            let owned_root = root.to_path_buf();
            let home = context.opencodex_home.clone();
            let acknowledged = tauri::async_runtime::spawn_blocking(move || {
                super::config_file::rename_effects::acknowledge(&owned_root, &home, &id)
            })
            .await
            .map_err(|_| {
                invalid(
                    "rename_activation_ack",
                    "Codex已重启，改名准备结果待核对，请稍后重试",
                )
            })??;
            if !acknowledged {
                return Ok(CacheRefresh {
                    restarted: true,
                    success: false,
                    summary: "Codex已重启，但期间又有改名保存，请核对最新配置后再重启".into(),
                    reason: Some("rename_activation_changed".into()),
                });
            }
        }
    }
    Ok(result)
}

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
    let mut command = tokio::process::Command::new(&executable);
    command
        .arg("sync-cache")
        .current_dir(&context.working_directory)
        .env_clear()
        .env("OPENCODEX_HOME", &context.opencodex_home)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    platform::apply_runtime_environment(command.as_std_mut(), &executable);
    for key in ["LANG", "LC_ALL"] {
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
