//! Environment discovery is bound to the current Desktop installation. A null
//! request never scans legacy homes, system package managers or the caller PATH.
use crate::{
    errors::{AppError, AppResult},
    modules::{
        backup::safety,
        discovery::{CheckResult, EnvironmentGate, EnvironmentPaths},
        runtime::{
            self,
            install::{self, ManagedRuntimeIdentity},
            RuntimeHandle, RuntimeSourceKind,
        },
    },
    types::discovery::{DiscoveryRequest, EnvironmentReportDto},
};
use std::{
    ffi::OsString,
    path::{Component, Path, PathBuf},
    sync::Arc,
    time::Duration,
};

#[path = "discovery_flight.rs"]
mod discovery_flight;

#[tauri::command]
pub async fn discover_environment(
    request: Option<DiscoveryRequest>,
    handle: tauri::State<'_, Arc<RuntimeHandle>>,
) -> AppResult<EnvironmentReportDto> {
    let handle = handle.inner().clone();
    // Explicit path reports never join a managed observation. Equal default
    // requests share only the currently running checks on this exact handle.
    if explicit(request.clone())?.is_some() {
        return discover(request, handle).await;
    }
    static FLIGHTS: std::sync::OnceLock<Arc<discovery_flight::Flights>> =
        std::sync::OnceLock::new();
    let key = Arc::as_ptr(&handle) as usize;
    FLIGHTS
        .get_or_init(|| Arc::new(discovery_flight::Flights::default()))
        .clone()
        .run(key, async move { discover(request, handle).await })
        .await
}
fn error(code: &str, detail: &str) -> AppError {
    AppError::RuntimeManaged {
        code: code.into(),
        detail: detail.into(),
    }
}
fn explicit(request: Option<DiscoveryRequest>) -> AppResult<Option<EnvironmentPaths>> {
    let Some(request) = request else {
        return Ok(None);
    };
    match (&request.node_path, &request.npm_path, &request.ocx_path) {
        (None, None, None) => Ok(None),
        (Some(node), Some(npm), Some(ocx)) => {
            let paths = EnvironmentPaths::new(node.into(), npm.into(), ocx.into());
            for path in [&paths.node, &paths.npm, &paths.ocx] {
                if !path.is_absolute()
                    || path.to_string_lossy().chars().any(char::is_control)
                    || path
                        .components()
                        .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
                {
                    return Err(error(
                        "discovery_request_invalid",
                        "显式检查需要三项完整、绝对且不含跳转段的工具路径",
                    ));
                }
            }
            Ok(Some(paths))
        }
        _ => Err(error(
            "discovery_request_invalid",
            "Node、npm 和 ocx 路径必须同时明确提供；不完整输入不会回退旧路径",
        )),
    }
}
#[derive(PartialEq, Eq)]
struct Managed {
    identity: ManagedRuntimeIdentity,
    paths: EnvironmentPaths,
    script: PathBuf,
    npm_digest: String,
}
fn managed(handle: &RuntimeHandle) -> AppResult<Managed> {
    if !handle.resolver_managed_only() {
        return Err(error(
            "discovery_managed_source_required",
            "当前环境检查仅接受 Desktop 集成程序来源",
        ));
    }
    let root = handle.installation_root();
    if !root.is_absolute() || !root.is_dir() {
        return Err(error(
            "discovery_runtime_unresolved",
            "当前 Desktop 集成安装根尚未就绪，未检查旧环境",
        ));
    }
    for path in [
        root.join(".runtime-bundle-pending.json"),
        root.join(".runtime-bundle-failure.json"),
    ] {
        safety::check_path(root, &path, true)?;
        if path
            .try_exists()
            .map_err(|_| error("discovery_runtime_invalid", "集成运行包状态不可读"))?
        {
            return Err(error(
                "discovery_runtime_pending",
                "集成运行包尚待供应或核对，未检查旧环境",
            ));
        }
    }
    let current = handle.current();
    if current.kind != RuntimeSourceKind::Managed
        || current.path.as_ref() != Some(&handle.managed_entry())
    {
        return Err(error(
            "discovery_runtime_unresolved",
            "当前 Desktop 集成运行来源尚未就绪，未检查旧环境",
        ));
    }
    let identity = install::resolve_managed_installation(root).map_err(|_| {
        error(
            "discovery_runtime_invalid",
            "集成运行包或激活证明不一致，请核对当前安装来源",
        )
    })?;
    // resolve_managed_installation above already fully verifies Node, launcher,
    // package and bin. Repeating managed_executables would hash the same Node
    // a second time here; retain one full check both BEFORE and AFTER probes.
    let (node, npm) = install::managed_dependency_paths(root);
    for path in [&node, &npm] {
        safety::check_path(root, path, false)?;
    }
    runtime::paths::validate_executable(&node)
        .map_err(|_| error("discovery_runtime_invalid", "固定 Node 依赖未就绪"))?;
    if identity.node != node || identity.entry != handle.managed_entry() {
        return Err(error(
            "discovery_runtime_invalid",
            "固定运行入口与当前激活来源不一致",
        ));
    }
    // The exact package bin is proved by resolve_managed_installation. Invoke
    // through that same Node on every platform, including Windows .cmd reports.
    let package = identity.prefix.join(install::PACKAGE_SUBPATH);
    let value: serde_json::Value =
        serde_json::from_slice(&safety::read_file(&package.join("package.json"), 65536)?)
            .map_err(|_| error("discovery_runtime_invalid", "集成包声明不可读"))?;
    let bin = runtime::archive::resolve_bin(&value)
        .ok_or_else(|| error("discovery_runtime_invalid", "集成包没有受管入口"))?;
    let bin = bin.strip_prefix("./").unwrap_or(&bin);
    let relative = Path::new(bin);
    if relative.is_absolute()
        || relative
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(error("discovery_runtime_invalid", "集成包入口越出受管目录"));
    }
    let script = package.join(relative);
    safety::check_path(root, &script, false)?;
    let npm_digest =
        crate::infrastructure::hash::sha256_hex(&safety::read_file(&npm, 1024 * 1024)?);
    let paths = EnvironmentPaths::new(node, npm, identity.entry.clone());
    Ok(Managed {
        identity,
        paths,
        script,
        npm_digest,
    })
}
async fn checked_managed(handle: Arc<RuntimeHandle>) -> AppResult<Managed> {
    tauri::async_runtime::spawn_blocking(move || managed(&handle))
        .await
        .map_err(|_| error("discovery_worker", "集成来源检查中断"))?
}
async fn discover(
    request: Option<DiscoveryRequest>,
    handle: Arc<RuntimeHandle>,
) -> AppResult<EnvironmentReportDto> {
    let paths = explicit(request)?;
    let managed = if paths.is_none() {
        Some(checked_managed(handle.clone()).await?)
    } else {
        None
    };
    let paths = paths
        .or_else(|| managed.as_ref().map(|m| m.paths.clone()))
        .ok_or(AppError::NotConfigured)?;
    let temp = tempfile::tempdir().map_err(|_| {
        error(
            "discovery_probe_environment",
            "无法建立只读检查的隔离工作区",
        )
    })?;
    // Diagnostic builds retain stage timings only: never output arguments,
    // paths, credentials or version stdout, and never substitute them for gates.
    #[cfg(debug_assertions)]
    let trace = handle
        .store()
        .path()
        .parent()
        .and_then(Path::parent)
        .map(crate::infrastructure::runtime_log::RuntimeLog::new);
    #[cfg(not(debug_assertions))]
    let trace = None;
    let report = probe_paths_traced(
        &paths,
        managed.as_ref().map(|m| m.script.as_path()),
        temp.path(),
        Duration::from_secs(10),
        trace.as_ref(),
    )
    .await;
    if let Some(expected) = managed {
        if !report.node.found || !report.npm.found || !report.ocx.found {
            let code = if !report.node.found {
                "discovery_node_probe_failed"
            } else if !report.npm.found {
                "discovery_npm_probe_failed"
            } else {
                "discovery_ocx_probe_failed"
            };
            return Err(error(
                code,
                "集成依赖或运行入口版本探测未成功，请核对当前安装来源；不会回退到系统安装指引",
            ));
        }
        let pins: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../.github/runtime-platforms-023.json"
        )))
        .map_err(|_| error("discovery_runtime_policy", "集成依赖版本合同不可用"))?;
        for (check, version) in [
            (&report.node, pins["node"]["version"].as_str()),
            (&report.npm, pins["node"]["npmVersion"].as_str()),
            (
                &report.ocx,
                Some(expected.identity.package_version.as_str()),
            ),
        ] {
            if check.found && check.version.as_deref() != version {
                return Err(error(
                    "discovery_runtime_version_mismatch",
                    "实际执行的依赖或运行包版本不符合当前受管来源",
                ));
            }
        }
        if expected != checked_managed(handle).await? {
            return Err(error(
                "discovery_runtime_changed",
                "检查期间集成来源或依赖发生变化，请重新核对",
            ));
        }
    }
    Ok(report)
}
fn not_checked() -> CheckResult {
    CheckResult {
        found: false,
        path: None,
        version: None,
    }
}
async fn check(
    program: &Path,
    args: &[OsString],
    reported: &Path,
    home: &Path,
    timeout: Duration,
) -> CheckResult {
    let version = probe(program, args, home, timeout).await;
    CheckResult {
        found: version.is_some(),
        path: Some(reported.to_string_lossy().into_owned()),
        version,
    }
}
async fn probe_paths_traced(
    paths: &EnvironmentPaths,
    managed_script: Option<&Path>,
    home: &Path,
    timeout: Duration,
    trace: Option<&crate::infrastructure::runtime_log::RuntimeLog>,
) -> EnvironmentReportDto {
    let started = std::time::Instant::now();
    let node = check(
        &paths.node,
        &["--version".into()],
        &paths.node,
        home,
        timeout,
    )
    .await;
    trace_probe(trace, "node", started, node.found);
    let mut report = EnvironmentReportDto {
        node,
        npm: not_checked(),
        ocx: not_checked(),
        gate: EnvironmentGate::MissingNode,
        short_circuited: true,
        brew_found: false,
    };
    if !report.node.found {
        return report;
    }
    let npm_is_js = paths
        .npm
        .extension()
        .is_some_and(|v| v == "js" || v == "mjs" || v == "cjs");
    let started = std::time::Instant::now();
    report.npm = if npm_is_js {
        check(
            &paths.node,
            &[paths.npm.as_os_str().into(), "--version".into()],
            &paths.npm,
            home,
            timeout,
        )
        .await
    } else {
        check(&paths.npm, &["--version".into()], &paths.npm, home, timeout).await
    };
    trace_probe(trace, "npm", started, report.npm.found);
    report.gate = EnvironmentGate::MissingNpm;
    if !report.npm.found {
        return report;
    }
    let started = std::time::Instant::now();
    report.ocx = if let Some(script) = managed_script {
        check(
            &paths.node,
            &[script.as_os_str().into(), "--version".into()],
            &paths.ocx,
            home,
            timeout,
        )
        .await
    } else {
        check(&paths.ocx, &["--version".into()], &paths.ocx, home, timeout).await
    };
    trace_probe(trace, "ocx", started, report.ocx.found);
    report.gate = if report.ocx.found {
        EnvironmentGate::Ready
    } else {
        EnvironmentGate::MissingOcx
    };
    report.short_circuited = false;
    report
}
fn trace_probe(
    trace: Option<&crate::infrastructure::runtime_log::RuntimeLog>,
    stage: &str,
    started: std::time::Instant,
    found: bool,
) {
    if let Some(log) = trace {
        let _ = log.append_event(&format!(
            "environment probe stage={stage} elapsed_ms={} found={found}",
            started.elapsed().as_millis(),
        ));
    }
}
/// Execute only --version with an empty inherited environment and bounded pipe
/// reads/deadline. No installation, config/profile reads, server or PATH lookup.
async fn probe(
    program: &Path,
    args: &[OsString],
    home: &Path,
    timeout: Duration,
) -> Option<String> {
    use tokio::io::AsyncReadExt;
    runtime::paths::validate_executable(program).ok()?;
    let mut command = tokio::process::Command::new(program);
    command
        .args(args)
        .env_clear()
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("OPENCODEX_HOME", home)
        .env("CODEX_HOME", home)
        .env("CLAUDE_CONFIG_DIR", home)
        .env("XDG_CONFIG_HOME", home)
        .env("XDG_CACHE_HOME", home)
        .env("TMPDIR", home)
        .env("TEMP", home)
        .env("PATH", "")
        .env("CI", "1")
        .env("OCX_NO_DESKTOP_HANDOFF", "1")
        .env("npm_config_userconfig", home.join("no-user.npmrc"))
        .env("npm_config_globalconfig", home.join("no-global.npmrc"))
        .env("npm_config_update_notifier", "false")
        .current_dir(home)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
    #[cfg(windows)]
    if let Some(root) = std::env::var_os("SystemRoot") {
        command.env("SystemRoot", root);
    }
    #[cfg(unix)]
    command.process_group(0);
    #[cfg(windows)]
    let job = crate::infrastructure::windows_query_job::OwnedQueryJob::new(&mut command).ok()?;
    let mut child = command.spawn().ok()?;
    #[cfg(windows)]
    if job.assign_and_resume(&child).is_err() {
        let _ = child.kill().await;
        let _ = child.wait().await;
        return None;
    }
    #[cfg(unix)]
    let pid = child.id();
    let mut stdout = child.stdout.take()?.take(4097);
    let mut output = vec![];
    let result = tokio::time::timeout(timeout, async {
        let (status, _) = tokio::try_join!(child.wait(), stdout.read_to_end(&mut output))?;
        Ok::<_, std::io::Error>(status)
    })
    .await;
    #[cfg(windows)]
    let family_done = job.terminate_and_wait().await.is_ok();
    #[cfg(unix)]
    if let Some(pid) = pid {
        unsafe {
            libc::kill(-(pid as i32), libc::SIGKILL);
        }
    }
    let _ = child.kill().await;
    let _ = child.wait().await;
    #[cfg(windows)]
    if !family_done {
        return None;
    }
    if !result.ok()?.ok()?.success() || output.len() > 4096 {
        return None;
    }
    install::extract_semver(std::str::from_utf8(&output).ok()?)
}
#[cfg(test)]
#[path = "discovery_v2_tests.rs"]
mod tests;
