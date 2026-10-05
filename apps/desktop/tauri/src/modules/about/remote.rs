//! 官方 OpenCodex 远端最新版本只读查询（U-03）。
//!
//! 执行受控 `npm view @bitkyc08/opencodex@<tag> version` 与 `dist.integrity`，带超时；
//! 不安装、不写盘，失败返回明确错误，不改变任何本地状态。代理由进程环境（HTTP(S)_PROXY/NO_PROXY）
//! 承载，与统一网络代理一致。

use std::path::PathBuf;
use std::process::Stdio;

use serde::Deserialize;

use crate::errors::AppError;
use crate::modules::process::EnvironmentPolicy;

/// 远端最新版本事实（只读投影）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficialRemoteLatest {
    pub tag: String,
    pub version: String,
    pub integrity: Option<String>,
}

/// 当前支持的官方 tag；与官方更新器一致。
pub const OFFICIAL_TAGS: [&str; 2] = ["latest", "preview"];

/// 校验 tag 属于允许集合，避免把任意字符串拼进命令。
pub fn validate_tag(tag: &str) -> Result<&str, AppError> {
    if OFFICIAL_TAGS.contains(&tag) {
        Ok(tag)
    } else {
        Err(AppError::NotConfigured)
    }
}

/// 解析 `npm view <pkg>@<tag> version --json` 的输出：可能是字符串或含 version 的对象。
pub fn parse_version(raw: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(raw.trim()).ok()?;
    match value {
        serde_json::Value::String(text) => Some(text),
        serde_json::Value::Object(map) => map
            .get("version")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        _ => None,
    }
}

/// 解析 `npm view <pkg>@<tag> dist.integrity --json`；失败返回 None（integrity 缺失不阻断）。
pub fn parse_integrity(raw: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(raw.trim()).ok()?;
    match value {
        serde_json::Value::String(text) => Some(text),
        serde_json::Value::Object(map) => map
            .get("integrity")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        _ => None,
    }
}

/// 执行一次受控 npm 查询；npm/node 路径由调用方注入，不搜索 PATH。
pub fn query_remote_latest(
    npm: &std::path::Path,
    working_directory: &std::path::Path,
    environment: &EnvironmentPolicy,
    tag: &str,
) -> Result<OfficialRemoteLatest, AppError> {
    let tag = validate_tag(tag)?;
    if !npm.is_file() {
        return Err(AppError::NotConfigured);
    }
    let package = format!("{}@{tag}", crate::modules::runtime::OFFICIAL_PACKAGE);
    let version = run_npm_view(npm, working_directory, environment, &package, "version")?;
    let Some(version) = version.and_then(|raw| parse_version(&raw)) else {
        return Err(AppError::NotConfigured);
    };
    let integrity = run_npm_view(
        npm,
        working_directory,
        environment,
        &package,
        "dist.integrity",
    )
    .ok()
    .flatten()
    .and_then(|raw| parse_integrity(&raw));
    Ok(OfficialRemoteLatest {
        tag: tag.to_string(),
        version,
        integrity,
    })
}

/// 执行单次 `npm view ... <field> --json`，带超时；输出截断防止异常放大。
///
/// 超时来自固化网络行为策略（U-05b）；超时后终止子进程并返回失败，不改写任何状态。
fn run_npm_view(
    npm: &std::path::Path,
    working_directory: &std::path::Path,
    environment: &EnvironmentPolicy,
    package: &str,
    field: &str,
) -> Result<Option<String>, AppError> {
    let mut command = std::process::Command::new(npm);
    command
        .arg("view")
        .arg(package)
        .arg(field)
        .arg("--json")
        .current_dir(working_directory)
        .env_clear()
        .stdin(Stdio::null())
        .stderr(Stdio::null());
    if let Some(home) = environment.home.as_ref() {
        crate::infrastructure::platform::apply_user_environment(&mut command, Some(home));
    } else {
        crate::infrastructure::platform::apply_user_environment(&mut command, None);
    }
    command.env("OPENCODEX_HOME", &environment.opencodex_home);
    // 关键：npm 是 `#!/usr/bin/env node` 脚本，env_clear 后必须给出能找到 node 的 PATH。
    // 至少包含 npm 自身所在目录（node 与它同目录），并补系统最小 PATH 兜底，
    // 不依赖调用方环境的 PATH（与受控安装 `SystemNpmRunner` 同一策略）。
    command.env("PATH", npm_path(npm, environment.path.as_deref()));
    for (key, value) in [
        ("HTTP_PROXY", &environment.http_proxy),
        ("HTTPS_PROXY", &environment.https_proxy),
        ("NO_PROXY", &environment.no_proxy),
    ] {
        if let Some(value) = value {
            command.env(key, value);
        }
    }
    let mut child = command
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|_| AppError::NotConfigured)?;
    // 带超时收口：到点先杀子进程再判定失败，避免查询请求悬挂。
    let deadline = std::time::Instant::now() + crate::modules::network_defaults::request_timeout();
    let output = loop {
        match child.try_wait().map_err(|_| AppError::NotConfigured)? {
            Some(_) => {
                break child
                    .wait_with_output()
                    .map_err(|_| AppError::NotConfigured)?
            }
            None if std::time::Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(AppError::Timeout);
            }
            None => std::thread::sleep(std::time::Duration::from_millis(25)),
        }
    };
    if !output.status.success() {
        return Err(AppError::NotConfigured);
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        return Ok(None);
    }
    Ok(Some(text.chars().take(512).collect()))
}

/// 受控 npm 查询的最小 PATH：npm 自身目录 → 调用方注入的 PATH（若有）→ 系统最小兜底。
fn npm_path(npm: &std::path::Path, injected: Option<&std::ffi::OsStr>) -> std::ffi::OsString {
    crate::infrastructure::platform::controlled_path(
        npm.parent().map(std::path::Path::to_path_buf),
        injected,
    )
}

/// 便捷：解析受控发现路径下的 npm。
pub fn discovered_npm() -> Option<PathBuf> {
    let paths = crate::types::discovery_paths::default_paths()?;
    crate::modules::runtime::paths::validate_executable(&paths.npm).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_validation_rejects_arbitrary_input() {
        assert!(validate_tag("latest").is_ok());
        assert!(validate_tag("preview").is_ok());
        assert!(validate_tag("../../etc/passwd").is_err());
        assert!(validate_tag("--force").is_err());
    }

    #[test]
    fn parses_version_and_integrity_shapes() {
        assert_eq!(parse_version("\"2.77.0\"").as_deref(), Some("2.77.0"));
        assert_eq!(
            parse_version("{\"version\":\"2.77.0\"}").as_deref(),
            Some("2.77.0")
        );
        assert_eq!(parse_version("not json"), None);
        assert_eq!(
            parse_integrity("\"sha512-abc\"").as_deref(),
            Some("sha512-abc")
        );
    }

    #[cfg(unix)]
    #[test]
    fn npm_path_includes_npm_dir_and_fallback() {
        let p = npm_path(std::path::Path::new("/Users/x/.local/bin/npm"), None);
        let p = p.to_string_lossy();
        assert!(
            p.starts_with("/Users/x/.local/bin:"),
            "npm dir must lead PATH: {p}"
        );
        assert!(p.contains("/usr/bin"), "fallback must be present: {p}");
    }

    #[cfg(unix)]
    #[test]
    fn npm_path_appends_injected_before_fallback() {
        let injected = std::ffi::OsString::from("/opt/tools");
        let p = npm_path(std::path::Path::new("/a/b/npm"), Some(&injected));
        let p = p.to_string_lossy();
        assert!(p.starts_with("/a/b:/opt/tools:"), "order wrong: {p}");
    }

    /// 真实远端查询（U-03）：默认忽略，按需 `cargo test -- --ignored` 运行，
    /// 走生产函数解析 `@bitkyc08/opencodex@latest` 的版本与 integrity。
    #[test]
    #[ignore = "hits the npm registry; run explicitly"]
    fn live_remote_query_returns_version_and_integrity() {
        let npm = discovered_npm().expect("npm must be discoverable");
        let working = std::env::temp_dir();
        // 受控子进程会 `env_clear()`；npm 是脚本，必须显式注入 PATH 才能找到 node。
        let environment = EnvironmentPolicy {
            home: crate::infrastructure::platform::home_dir().map(|home| home.into_os_string()),
            path: std::env::var_os("PATH"),
            opencodex_home: std::env::temp_dir(),
            ..Default::default()
        };
        let result =
            query_remote_latest(&npm, &working, &environment, "latest").expect("live remote query");
        assert_eq!(result.tag, "latest");
        assert!(!result.version.is_empty());
        assert!(result.version.chars().next().unwrap().is_ascii_digit());
    }
}
