//! 官方 OpenCodex 远端最新版本只读查询（U-03）。
//!
//! 单次查询 version 与 dist.integrity，限制输出并设总截止时间；
//! 不安装，失败返回明确错误。代理由进程环境（HTTP(S)_PROXY/NO_PROXY）
//! 承载，与统一网络代理一致。

use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::AsyncReadExt;

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
    let raw = run_npm_view(npm, working_directory, environment, &package)?;
    let value: serde_json::Value =
        serde_json::from_str(&raw).map_err(|_| AppError::NotConfigured)?;
    let (version, integrity) = parse_metadata(&value).ok_or(AppError::NotConfigured)?;
    Ok(OfficialRemoteLatest {
        tag: tag.to_string(),
        version,
        integrity,
    })
}

/// 大于此上限即拒绝，不把截断数据当作有效元数据。
const MAX_METADATA_BYTES: u64 = 16 * 1024;

fn parse_metadata(value: &serde_json::Value) -> Option<(String, Option<String>)> {
    let version = value.get("version")?.as_str()?;
    if version.is_empty() || version.len() > 128 {
        return None;
    }
    let integrity = value
        .get("dist.integrity")
        .or_else(|| value.get("dist").and_then(|dist| dist.get("integrity")))
        .and_then(|value| value.as_str())
        .filter(|text| text.len() <= 4096)
        .map(str::to_owned);
    Some((version.to_owned(), integrity))
}

fn run_npm_view(
    npm: &std::path::Path,
    working_directory: &std::path::Path,
    environment: &EnvironmentPolicy,
    package: &str,
) -> Result<String, AppError> {
    run_npm_view_with_timeout(
        npm,
        working_directory,
        environment,
        package,
        crate::modules::network_defaults::request_timeout(),
    )
}

fn run_npm_view_with_timeout(
    npm: &std::path::Path,
    working_directory: &std::path::Path,
    environment: &EnvironmentPolicy,
    package: &str,
    timeout: std::time::Duration,
) -> Result<String, AppError> {
    let mut command = tokio::process::Command::new(npm);
    command
        .arg("view")
        .arg(package)
        .args(["version", "dist.integrity"])
        .arg("--json")
        .current_dir(working_directory)
        .env_clear()
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .kill_on_drop(true);
    if let Some(home) = environment.home.as_ref() {
        crate::infrastructure::platform::apply_user_environment(command.as_std_mut(), Some(home));
    } else {
        crate::infrastructure::platform::apply_user_environment(command.as_std_mut(), None);
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
    #[cfg(unix)]
    command.process_group(0);
    // Called from spawn_blocking; never hold the UI executor while reading pipes.
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| AppError::NotConfigured)?
        .block_on(async {
            let mut child = command.spawn().map_err(|_| AppError::NotConfigured)?;
            let pid = child.id();
            let stdout = child.stdout.take().ok_or(AppError::NotConfigured)?;
            let result = tokio::time::timeout(timeout, async {
                let mut bytes = Vec::new();
                stdout
                    .take(MAX_METADATA_BYTES + 1)
                    .read_to_end(&mut bytes)
                    .await
                    .map_err(|_| AppError::NotConfigured)?;
                if bytes.len() as u64 > MAX_METADATA_BYTES {
                    return Err(AppError::NotConfigured);
                }
                if !child
                    .wait()
                    .await
                    .map_err(|_| AppError::NotConfigured)?
                    .success()
                {
                    return Err(AppError::NotConfigured);
                }
                String::from_utf8(bytes).map_err(|_| AppError::NotConfigured)
            })
            .await
            .unwrap_or(Err(AppError::Timeout));
            if result.is_err() {
                #[cfg(unix)]
                if let Some(pid) = pid {
                    // Only this query group; never signal the managed panel process.
                    unsafe {
                        libc::kill(-(pid as i32), libc::SIGKILL);
                    }
                }
                #[cfg(not(unix))]
                let _ = pid;
                let _ = child.kill().await;
                let _ = child.wait().await;
            }
            result
        })
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
    fn combined_metadata_accepts_flat_and_nested_shapes_and_rejects_bad_versions() {
        for raw in [
            r#"{"version":"2.50.0","dist.integrity":"sha512-abc"}"#,
            r#"{"version":"2.50.0","dist":{"integrity":"sha512-abc"}}"#,
        ] {
            assert_eq!(
                parse_metadata(&serde_json::from_str(raw).unwrap()),
                Some(("2.50.0".into(), Some("sha512-abc".into())))
            );
        }
        assert!(parse_metadata(&serde_json::json!({"version":""})).is_none());
        assert!(parse_metadata(&serde_json::json!({"version":"x".repeat(129)})).is_none());
        assert!(parse_metadata(&serde_json::json!(["2.50.0"])).is_none());
    }

    #[cfg(unix)]
    #[test]
    fn metadata_query_rejects_oversize_and_kills_timed_out_process_group() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().unwrap();
        let npm = root.path().join("npm");
        let environment = EnvironmentPolicy {
            opencodex_home: root.path().to_owned(),
            ..Default::default()
        };
        std::fs::write(&npm, "#!/bin/sh\nhead -c 17000 /dev/zero\n").unwrap();
        std::fs::set_permissions(&npm, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert!(run_npm_view_with_timeout(
            &npm,
            root.path(),
            &environment,
            "test@latest",
            std::time::Duration::from_secs(2)
        )
        .is_err());
        // A descendant holding stdout must also be stopped at the total deadline.
        std::fs::write(&npm, "#!/bin/sh\nsleep 30 &\necho $! > child-pid\nwait\n").unwrap();
        let start = std::time::Instant::now();
        assert!(matches!(
            run_npm_view_with_timeout(
                &npm,
                root.path(),
                &environment,
                "test@latest",
                std::time::Duration::from_millis(200)
            ),
            Err(AppError::Timeout)
        ));
        assert!(start.elapsed() < std::time::Duration::from_secs(2));
        let pid: i32 = std::fs::read_to_string(root.path().join("child-pid"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        // The process may be briefly visible as a zombie; it must no longer be running.
        let status = std::process::Command::new("/bin/ps")
            .args(["-o", "stat=", "-p", &pid.to_string()])
            .output()
            .unwrap();
        let stat = String::from_utf8_lossy(&status.stdout);
        assert!(
            stat.trim().is_empty() || stat.trim().starts_with('Z'),
            "descendant still running: {stat}"
        );
    }

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
