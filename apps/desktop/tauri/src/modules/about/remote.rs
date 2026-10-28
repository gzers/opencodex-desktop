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

#[path = "remote_diagnostics.rs"]
mod diagnostics;
use diagnostics::{Capture, QueryLog};

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
    cache: &std::path::Path,
    log_directory: &std::path::Path,
    environment: &EnvironmentPolicy,
    tag: &str,
) -> Result<OfficialRemoteLatest, AppError> {
    let tag = validate_tag(tag)?;
    let package = format!("{}@{tag}", crate::modules::runtime::OFFICIAL_PACKAGE);
    let (version, integrity) = run_npm_view_with_timeout(
        npm,
        working_directory,
        cache,
        log_directory,
        environment,
        &package,
        crate::modules::network_defaults::request_timeout(),
    )?;
    Ok(OfficialRemoteLatest {
        tag: tag.to_string(),
        version,
        integrity,
    })
}

/// 大于此上限即拒绝，不把截断数据当作有效元数据。
const MAX_METADATA_BYTES: usize = 16 * 1024;
const MAX_STDERR_BYTES: usize = 64 * 1024;

fn parse_metadata(value: &serde_json::Value) -> Option<(String, Option<String>)> {
    // npm returns a plain JSON string when version is the only requested field
    // present (for example a registry without dist.integrity).
    let version = value
        .as_str()
        .or_else(|| value.get("version").and_then(serde_json::Value::as_str))?;
    if !crate::modules::update::handoff::valid_version(version) {
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

#[allow(clippy::too_many_arguments)]
fn run_npm_view_with_timeout(
    npm: &std::path::Path,
    working_directory: &std::path::Path,
    cache: &std::path::Path,
    log_directory: &std::path::Path,
    environment: &EnvironmentPolicy,
    package: &str,
    timeout: std::time::Duration,
) -> Result<(String, Option<String>), AppError> {
    let started = std::time::Instant::now();
    if !log_directory.is_absolute() {
        return Err(AppError::RemoteQuery {
            reason: "npm_view_paths".into(),
            detail: "npm view 命令日志目录必须是绝对路径；查询未执行。".into(),
        });
    }
    let mut command = tokio::process::Command::new(npm);
    command
        .arg("view")
        .arg(package)
        .args(["version", "dist.integrity"])
        .arg("--json")
        .current_dir(working_directory)
        .env_clear()
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .kill_on_drop(true);
    if let Some(home) = environment.home.as_ref() {
        crate::infrastructure::platform::apply_user_environment(command.as_std_mut(), Some(home));
    } else {
        crate::infrastructure::platform::apply_user_environment(command.as_std_mut(), None);
    }
    command.env("OPENCODEX_HOME", &environment.opencodex_home);
    command.env("npm_config_cache", dunce::simplified(cache));
    command.env("npm_config_update_notifier", "false");
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
    let mut log = QueryLog::create(command.as_std(), log_directory, environment)?;
    let mut stdout_capture = Capture::default();
    let mut stderr_capture = Capture::default();
    let mut exit_status = None;
    // Keep a stable reason separate from localized OS text, including setup
    // failures that happen before npm has a chance to create its own debug log.
    let failure = |reason, detail: String| QueryFailure { reason, detail };
    let mut run = || -> Result<(String, Option<String>), QueryFailure> {
        if !npm.is_absolute()
            || !working_directory.is_absolute()
            || !cache.is_absolute()
            || !environment.opencodex_home.is_absolute()
        {
            return Err(failure(
                "npm_view_paths",
                "npm view 需要绝对的工具、工作目录、OPENCODEX_HOME 和缓存路径。".into(),
            ));
        }
        #[cfg(unix)]
        command.process_group(0);
        #[cfg(windows)]
        let job = crate::infrastructure::windows_query_job::OwnedQueryJob::new(&mut command)
            .map_err(|error| {
                failure(
                    "npm_view_setup",
                    format!("无法建立 npm view 进程边界：{error}"),
                )
            })?;
        // Called from spawn_blocking; never hold the UI executor while reading pipes.
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| {
                failure(
                    "npm_view_setup",
                    format!("无法建立 npm view 查询执行器：{error}"),
                )
            })?
            .block_on(async {
                let mut child = command.spawn().map_err(|error| {
                    failure("npm_view_spawn", format!("无法启动 npm view：{error}"))
                })?;
                let pid = child.id();
                let result = async {
                    #[cfg(windows)]
                    job.assign_and_resume(&child).map_err(|error| {
                        failure(
                            "npm_view_setup",
                            format!("无法绑定 npm view 进程边界：{error}"),
                        )
                    })?;
                    let stdout = child.stdout.take().ok_or_else(|| {
                        failure("npm_view_read", "无法读取 npm view stdout。".into())
                    })?;
                    let stderr = child.stderr.take().ok_or_else(|| {
                        failure("npm_view_read", "无法读取 npm view stderr。".into())
                    })?;
                    tokio::time::timeout(timeout.saturating_sub(started.elapsed()), async {
                        tokio::try_join!(
                            read_bounded(stdout, &mut stdout_capture, MAX_METADATA_BYTES, "stdout"),
                            read_bounded(stderr, &mut stderr_capture, MAX_STDERR_BYTES, "stderr"),
                            async {
                                let status = child.wait().await.map_err(|error| {
                                    failure(
                                        "npm_view_wait",
                                        format!("等待 npm view 退出失败：{error}"),
                                    )
                                })?;
                                exit_status = Some(status);
                                Ok(())
                            },
                        )?;
                        if !exit_status.is_some_and(|status| status.success()) {
                            return Err(failure(
                                "npm_view_exit",
                                format!(
                                    "npm view 以退出码 {} 结束。",
                                    exit_status
                                        .and_then(|status| status.code())
                                        .map_or("未知".into(), |code| code.to_string())
                                ),
                            ));
                        }
                        let raw = std::str::from_utf8(&stdout_capture.bytes).map_err(|error| {
                            failure(
                                "npm_view_encoding",
                                format!("npm view 元数据不是有效 UTF-8：{error}"),
                            )
                        })?;
                        let value = serde_json::from_str(raw).map_err(|error| {
                            failure(
                                "npm_view_metadata",
                                format!("npm view 返回的元数据不是有效 JSON：{error}"),
                            )
                        })?;
                        parse_metadata(&value).ok_or_else(|| {
                            failure(
                                "npm_view_metadata",
                                "npm view 返回的元数据缺少有效版本。".into(),
                            )
                        })
                    })
                    .await
                    .unwrap_or_else(|_| {
                        Err(failure(
                            "npm_view_timeout",
                            format!("npm view 查询超时（{} 毫秒）。", timeout.as_millis()),
                        ))
                    })
                }
                .await;
                if result.is_err() {
                    #[cfg(windows)]
                    let job_termination_failed = job.terminate().is_err();
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
                    #[cfg(windows)]
                    if job_termination_failed {
                        return Err(failure(
                            "npm_view_cleanup",
                            "npm view 查询失败，进程清理失败。".into(),
                        ));
                    }
                }
                result
            })
    };
    let result = run();
    log.finish(
        &stdout_capture,
        &stderr_capture,
        exit_status,
        result
            .as_ref()
            .err()
            .map(|error| (error.reason, error.detail.as_str())),
    )?;
    result.map_err(|error| {
        let output = log.output_detail(&stdout_capture, &stderr_capture);
        log.error(
            error.reason,
            &format!(
                "{}{}{}",
                error.detail,
                if output.is_empty() { "" } else { "\n" },
                output
            ),
        )
    })
}

struct QueryFailure {
    reason: &'static str,
    detail: String,
}

async fn read_bounded(
    mut reader: impl tokio::io::AsyncRead + Unpin,
    capture: &mut Capture,
    limit: usize,
    stream: &'static str,
) -> Result<(), QueryFailure> {
    let mut buffer = [0_u8; 4096];
    loop {
        let remaining = (limit + 1 - capture.bytes.len()).min(buffer.len());
        let read = reader
            .read(&mut buffer[..remaining])
            .await
            .map_err(|error| QueryFailure {
                reason: "npm_view_read",
                detail: format!("读取 npm view {stream} 失败：{error}"),
            })?;
        if read == 0 {
            capture.complete = true;
            return Ok(());
        }
        capture.bytes.extend_from_slice(&buffer[..read]);
        if capture.bytes.len() > limit {
            return Err(QueryFailure {
                reason: "npm_view_output_limit",
                detail: format!(
                    "npm view {stream} 超过 {limit} 字节上限；已停止查询，日志只保留有界完整记录。"
                ),
            });
        }
    }
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

#[cfg(all(test, windows))]
#[path = "remote_windows_tests.rs"]
mod windows_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combined_metadata_accepts_flat_and_nested_shapes_and_rejects_bad_versions() {
        for raw in [
            r#""2.50.0""#,
            r#"{"version":"2.50.0","dist.integrity":"sha512-abc"}"#,
            r#"{"version":"2.50.0","dist":{"integrity":"sha512-abc"}}"#,
        ] {
            assert_eq!(
                parse_metadata(&serde_json::from_str(raw).unwrap()),
                Some((
                    "2.50.0".into(),
                    if raw.starts_with('"') {
                        None
                    } else {
                        Some("sha512-abc".into())
                    }
                ))
            );
        }
        assert!(parse_metadata(&serde_json::json!({"version":""})).is_none());
        assert!(parse_metadata(&serde_json::json!({"version":"x".repeat(129)})).is_none());
        assert!(parse_metadata(&serde_json::json!(["2.50.0"])).is_none());
    }

    #[cfg(unix)]
    #[test]
    fn metadata_query_uses_managed_cache_and_preserves_user_home() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().unwrap();
        let npm = root.path().join("npm");
        let script = r#"#!/bin/sh
[ "$npm_config_cache" = "$OPENCODEX_HOME/cache/npm" ] || exit 11
[ "$HOME" = "$OPENCODEX_HOME/user-home" ] || exit 12
echo '{"version":"2.50.0","dist.integrity":"sha512-fixture"}'
"#;
        std::fs::write(&npm, script).unwrap();
        std::fs::set_permissions(&npm, std::fs::Permissions::from_mode(0o700)).unwrap();
        let environment = EnvironmentPolicy {
            opencodex_home: root.path().to_owned(),
            home: Some(root.path().join("user-home").into_os_string()),
            ..Default::default()
        };
        let result = query_remote_latest(
            &npm,
            root.path(),
            &root.path().join("cache/npm"),
            &root.path().join("logs/runtime-query"),
            &environment,
            "latest",
        )
        .unwrap();
        assert_eq!(result.version, "2.50.0");
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
            &root.path().join("cache/npm"),
            &root.path().join("logs/runtime-query"),
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
                &root.path().join("cache/npm"),
                &root.path().join("logs/runtime-query"),
                &environment,
                "test@latest",
                std::time::Duration::from_millis(200)
            ),
            Err(AppError::RemoteQuery { reason, .. }) if reason == "npm_view_timeout"
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
        let cache = tempfile::tempdir().expect("isolated npm cache");
        let result = query_remote_latest(
            &npm,
            &working,
            cache.path(),
            &cache.path().join("logs"),
            &environment,
            "latest",
        )
        .expect("live remote query");
        assert_eq!(result.tag, "latest");
        assert!(!result.version.is_empty());
        assert!(result.version.chars().next().unwrap().is_ascii_digit());
    }
}
