//! 受控官方状态来源：只在 infrastructure 执行显式路径的 `ocx status --json`。
//!
//! FZ-07/FZ-08 规定 5 秒采集窗口；失败保留上一份状态。本层不搜索 PATH，
//! 环境先清空后仅注入冻结键，不读取真实用户配置或 Keychain。

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;

use tokio::process::Command;

use crate::modules::status::{CollectError, StatusSource};

/// 官方状态采集单次冻结窗口；与 FZ-08 保持同一常量。
pub fn status_collect_timeout() -> Duration {
    crate::modules::status::collect_timeout()
}

/// 显式路径官方状态来源。
#[derive(Debug)]
pub struct OfficialStatusSource {
    runtime: crate::infrastructure::runtime_executable::SharedRuntimeExecutable,
    working_directory: PathBuf,
    environment: crate::modules::process::EnvironmentPolicy,
    cache: Mutex<Option<CachedStatus>>,
}
#[derive(Debug, Clone)]
struct CachedStatus {
    executable: PathBuf,
    at: Instant,
    payload: serde_json::Value,
    uptime: Option<f64>,
}

impl OfficialStatusSource {
    /// 构造来源；不立即执行命令、不验证路径内容。
    pub fn new(
        runtime: crate::infrastructure::runtime_executable::SharedRuntimeExecutable,
        working_directory: impl Into<PathBuf>,
        environment: crate::modules::process::EnvironmentPolicy,
    ) -> Self {
        Self {
            runtime,
            working_directory: working_directory.into(),
            environment,
            cache: Mutex::new(None),
        }
    }

    pub fn set_environment(&mut self, opencodex_home: PathBuf) {
        self.environment.opencodex_home = opencodex_home;
        *self.cache.get_mut().expect("status cache") = None;
    }

    /// 每次采集都重新向共享句柄取当前来源，安装 / 卸载后无需重建来源对象。
    fn command(&self, executable: &std::path::Path) -> Command {
        let mut process = Command::new(executable);
        process
            .arg("status")
            .arg("--json")
            .current_dir(&self.working_directory)
            .env_clear()
            .env("OPENCODEX_HOME", &self.environment.opencodex_home)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        crate::infrastructure::platform::apply_user_environment(
            process.as_std_mut(),
            self.environment.home.as_deref(),
        );
        for (key, value) in [
            ("LANG", &self.environment.lang),
            ("LC_ALL", &self.environment.lc_all),
            ("HTTP_PROXY", &self.environment.http_proxy),
            ("HTTPS_PROXY", &self.environment.https_proxy),
            ("NO_PROXY", &self.environment.no_proxy),
            ("PATH", &self.environment.path),
        ] {
            if let Some(value) = value {
                process.env(key, value);
            }
        }
        process
    }

    async fn fetch_inner(&self) -> Result<serde_json::Value, CollectError> {
        let executable = self.runtime.executable();
        let validate = || -> Result<(), CollectError> {
            if !self.working_directory.is_dir() {
                return Err(CollectError::Unreachable);
            }
            if !self.environment.opencodex_home.is_absolute() {
                return Err(CollectError::Unreachable);
            }
            Ok(())
        };
        validate()?;
        // 未解析出运行来源时按「不可达」处理：界面据此展示门禁引导，而不是静默停在加载中。
        let Some(executable) = executable else {
            return Err(CollectError::Unreachable);
        };
        if !executable.is_file() {
            return Err(CollectError::Unreachable);
        }

        let mut command = self.command(&executable);
        #[cfg(unix)]
        command.process_group(0);
        let mut child = command.spawn().map_err(|_| CollectError::Unreachable)?;
        #[cfg(unix)]
        let diagnostic_group = child.id();
        let mut stdout = child.stdout.take().ok_or(CollectError::Unreachable)?;
        let limit = crate::modules::runtime_defaults::status_output_max_bytes();
        let mut bytes = Vec::new();
        let result = tokio::time::timeout(status_collect_timeout(), async {
            (&mut stdout)
                .take(limit + 1)
                .read_to_end(&mut bytes)
                .await
                .map_err(|_| CollectError::Unreachable)?;
            if bytes.len() as u64 > limit {
                return Err(CollectError::Parse);
            }
            let status = child.wait().await.map_err(|_| CollectError::Unreachable)?;
            if !status.success() {
                return Err(CollectError::Unreachable);
            }
            serde_json::from_slice(&bytes).map_err(|_| CollectError::Parse)
        })
        .await
        .unwrap_or(Err(CollectError::Timeout));
        if result.is_err() {
            // Only this diagnostic's isolated process group; never the proxy PID.
            #[cfg(unix)]
            if let Some(pid) = diagnostic_group {
                unsafe {
                    libc::kill(-(pid as i32), libc::SIGKILL);
                }
            }
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
        let payload: serde_json::Value = result?;
        *self.cache.lock().map_err(|_| CollectError::Unreachable)? = Some(CachedStatus {
            executable,
            at: Instant::now(),
            payload: payload.clone(),
            uptime: None,
        });
        Ok(payload)
    }

    async fn light_inner(&self) -> Result<serde_json::Value, CollectError> {
        let cached = self
            .cache
            .lock()
            .map_err(|_| CollectError::Unreachable)?
            .clone();
        if let Some(cached) = cached.filter(|cached| {
            self.runtime.executable().as_ref() == Some(&cached.executable)
                && cached.executable.is_file()
                && cached.at.elapsed()
                    < crate::modules::runtime_defaults::status_full_diagnostic_interval()
        }) {
            // Accept only the official identity at the cached port AND PID. /healthz
            // alone is liveness, not readiness. Unknown/old contracts use full CLI.
            let report =
                crate::modules::status::OfficialStatusReport::from_value(cached.payload.clone());
            let mapped = crate::modules::status::map_official_output(
                &report,
                None,
                crate::types::status::ConnectionState::Unconfigured,
                crate::types::status::OperationState::Idle,
            );
            if let (Some(port), Some(pid)) = (
                mapped.port,
                mapped
                    .pid
                    .as_deref()
                    .and_then(|pid| pid.parse::<u64>().ok()),
            ) {
                if let Ok((health, ready)) = probe_local(port).await {
                    let identity = |value: &serde_json::Value| {
                        value["service"] == "opencodex"
                            && value["pid"].as_u64() == Some(pid)
                            && value["port"].as_u64() == Some(port as u64)
                    };
                    let uptime = health["uptime"].as_f64();
                    let continuous = uptime.is_some_and(|uptime| {
                        uptime.is_finite()
                            && uptime >= 0.0
                            && cached.uptime.is_none_or(|previous| uptime >= previous)
                    });
                    let ready_now = ready["status"] == "ready";
                    if identity(&health)
                        && identity(&ready)
                        && health["status"] == "ok"
                        && continuous
                        && (mapped.runtime == crate::types::status::RuntimeState::Running)
                            == ready_now
                        && health["version"]
                            .as_str()
                            .is_some_and(|version| !version.is_empty())
                        && health["version"] == ready["version"]
                        && matches!(
                            ready["status"].as_str(),
                            Some("ready" | "pending" | "failed")
                        )
                    {
                        if let Ok(mut guard) = self.cache.lock() {
                            if let Some(cache) = guard.as_mut() {
                                cache.uptime = uptime;
                            }
                        }
                        return Ok(cached.payload);
                    }
                }
            }
        }
        self.fetch_inner().await
    }
}

async fn probe_local(port: u16) -> Result<(serde_json::Value, serde_json::Value), CollectError> {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    let client = CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(crate::modules::runtime_defaults::status_health_timeout())
            .build()
            .expect("loopback client")
    });
    async fn read(
        client: &reqwest::Client,
        port: u16,
        path: &str,
    ) -> Result<serde_json::Value, CollectError> {
        let mut response = client
            .get(format!("http://127.0.0.1:{port}/{path}"))
            .send()
            .await
            .map_err(|_| CollectError::Unreachable)?;
        if !matches!(response.status().as_u16(), 200 | 503) {
            return Err(CollectError::Unreachable);
        }
        let limit = crate::modules::runtime_defaults::status_output_max_bytes() as usize;
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| CollectError::Unreachable)?
        {
            if chunk.len() > limit.saturating_sub(bytes.len()) {
                return Err(CollectError::Parse);
            }
            bytes.extend_from_slice(&chunk);
        }
        serde_json::from_slice(&bytes).map_err(|_| CollectError::Parse)
    }
    tokio::try_join!(read(client, port, "healthz"), read(client, port, "readyz"))
}

impl StatusSource for OfficialStatusSource {
    fn fetch_light(&self) -> Result<serde_json::Value, CollectError> {
        tokio::runtime::Handle::try_current().map_err(|_| CollectError::Unreachable)?;
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.light_inner())
        })
    }

    fn fetch(&self) -> Result<serde_json::Value, CollectError> {
        tokio::runtime::Handle::try_current().map_err(|_| CollectError::Unreachable)?;
        let _guard = tokio::runtime::Handle::current().enter();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.fetch_inner())
        })
    }

    /// 未解析出 `ocx`、或解析到的入口已不存在 ⇒ 视为「未发现安装」。
    ///
    /// 采集器据此把状态落到 `not_found`：卸载（尤其完整卸载移除入口）后，
    /// 概览/托盘等消费同一份快照的地方必须立刻不再允许启动。
    fn resolved(&self) -> bool {
        self.runtime.executable().is_some_and(|path| path.is_file())
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::modules::process::{EnvironmentPolicy, LifecycleAction, ProcessCommand};
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    fn command_fixture(root: &Path) -> ProcessCommand {
        let executable = root.join("ocx");
        std::fs::write(&executable, b"#!/bin/sh\necho '{\"status\":\"stopped\",\"dataRoot\":\"/tmp/fixture\",\"startup\":{\"protection\":\"none\",\"rebootSafe\":false}}'\n")
            .expect("write fixture");
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755))
            .expect("chmod fixture");
        let home = root.join("home");
        std::fs::create_dir_all(&home).expect("create home");
        ProcessCommand::new(
            LifecycleAction::Start,
            executable,
            home,
            root.join("opencodex-home"),
        )
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fetches_and_maps_explicit_fixture_output() {
        let root = tempfile::tempdir().expect("create fixture root");
        let command = command_fixture(root.path());
        let source = OfficialStatusSource::new(
            crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(
                command.executable,
            ),
            command.working_directory,
            command.environment,
        );
        let mut collector = crate::modules::status::StatusCollector::new(source);
        let mapped = collector.refresh().expect("fixture output");
        assert_eq!(mapped.runtime, crate::types::status::RuntimeState::Stopped);
        assert_eq!(mapped.facts.data_root, PathBuf::from("/tmp/fixture"));
        assert_eq!(mapped.facts.protection.as_deref(), Some("none"));
        assert_eq!(mapped.facts.reboot_safe, Some(false));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn timeout_maps_to_frozen_timeout_and_preserves_state() {
        let root = tempfile::tempdir().expect("create fixture root");
        let executable = root.path().join("sleeping-ocx");
        std::fs::write(
            &executable,
            format!(
                "#!/bin/sh\nsleep {}\n",
                status_collect_timeout().as_secs() + 1
            ),
        )
        .expect("write sleep fixture");
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755))
            .expect("chmod fixture");
        let home = root.path().join("home");
        std::fs::create_dir_all(&home).expect("create home");
        let environment = EnvironmentPolicy {
            opencodex_home: root.path().join("opencodex-home"),
            ..Default::default()
        };
        let source = OfficialStatusSource::new(
            crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(executable),
            home,
            environment,
        );
        let mut collector = crate::modules::status::StatusCollector::new(source);
        let error = collector.refresh().expect_err("timeout expected");
        assert_eq!(error, CollectError::Timeout);
        assert_eq!(
            collector.matrix().runtime,
            crate::types::status::RuntimeState::NotFound
        );
        assert_eq!(collector.failure_count(), 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn invalid_json_maps_to_parse_and_preserves_state() {
        let root = tempfile::tempdir().expect("create fixture root");
        let executable = root.path().join("invalid-ocx");
        std::fs::write(&executable, b"#!/bin/sh\necho not-json\n").expect("write invalid fixture");
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755))
            .expect("chmod fixture");
        let home = root.path().join("home");
        std::fs::create_dir_all(&home).expect("create home");
        let environment = EnvironmentPolicy {
            opencodex_home: root.path().join("opencodex-home"),
            ..Default::default()
        };
        let source = OfficialStatusSource::new(
            crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(executable),
            home,
            environment,
        );
        let mut collector = crate::modules::status::StatusCollector::new(source);
        let error = collector.refresh().expect_err("parse error expected");
        assert_eq!(error, CollectError::Parse);
        assert_eq!(
            collector.matrix().runtime,
            crate::types::status::RuntimeState::NotFound
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn non_executable_file_is_unreachable() {
        let root = tempfile::tempdir().expect("create fixture root");
        let executable = root.path().join("non-executable");
        std::fs::write(&executable, b"not executable").expect("write fixture");
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o644))
            .expect("chmod fixture");
        let home = root.path().join("home");
        std::fs::create_dir_all(&home).expect("create home");
        let environment = EnvironmentPolicy {
            opencodex_home: root.path().join("opencodex-home"),
            ..Default::default()
        };
        let source = OfficialStatusSource::new(
            crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(executable),
            home,
            environment,
        );
        let mut collector = crate::modules::status::StatusCollector::new(source);
        let error = collector.refresh().expect_err("unreachable expected");
        assert_eq!(error, CollectError::Unreachable);
    }
    fn script_source(root: &Path, script: &str) -> OfficialStatusSource {
        let executable = root.join("ocx-fixture");
        std::fs::write(&executable, script).unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
        OfficialStatusSource::new(
            crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(executable),
            root,
            EnvironmentPolicy {
                opencodex_home: root.join("data"),
                ..Default::default()
            },
        )
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn steady_probe_avoids_cli_but_identity_restart_expiry_and_environment_refresh_it() {
        use std::io::{Read, Write};
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        let root = tempfile::tempdir().unwrap();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        listener.set_nonblocking(true).unwrap();
        let identity = Arc::new(Mutex::new(
            serde_json::json!({"status":"ok","service":"opencodex","pid":42,"port":port,"version":"2.69.0","uptime":100.0}),
        ));
        let stop = Arc::new(AtomicBool::new(false));
        let server_identity = identity.clone();
        let server_stop = stop.clone();
        // A TCP read is not an HTTP request boundary. Serve both concurrent
        // probes independently and read complete headers before selecting the
        // response; otherwise fragmented /readyz requests can look like healthz.
        struct ServerGuard {
            stop: Arc<AtomicBool>,
            thread: Option<std::thread::JoinHandle<()>>,
        }
        impl Drop for ServerGuard {
            fn drop(&mut self) {
                self.stop.store(true, Ordering::SeqCst);
                if let Some(thread) = self.thread.take() {
                    let _ = thread.join();
                }
            }
        }
        let server = std::thread::spawn(move || {
            let mut handlers = Vec::new();
            while !server_stop.load(Ordering::SeqCst) {
                let Ok((mut stream, _)) = listener.accept() else {
                    std::thread::sleep(Duration::from_millis(2));
                    continue;
                };
                let identity = server_identity.clone();
                handlers.push(std::thread::spawn(move || {
                    // macOS can inherit O_NONBLOCK from the listener; a complete
                    // request may not be available at accept time under load.
                    stream.set_nonblocking(false).unwrap();
                    stream
                        .set_read_timeout(Some(Duration::from_secs(1)))
                        .unwrap();
                    stream
                        .set_write_timeout(Some(Duration::from_secs(1)))
                        .unwrap();
                    let mut request = Vec::new();
                    let mut buffer = [0; 512];
                    while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                        let count = stream.read(&mut buffer).unwrap_or(0);
                        if count == 0 || request.len() + count > 8192 {
                            return;
                        }
                        request.extend_from_slice(&buffer[..count]);
                    }
                    let path = request.split(|byte| *byte == b' ').nth(1);
                    let mut payload = identity.lock().unwrap().clone();
                    match path {
                        Some(b"/readyz") => payload["status"] = serde_json::json!("ready"),
                        Some(b"/healthz") => (),
                        _ => panic!("unexpected probe request"),
                    }
                    let body = payload.to_string();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    stream.write_all(response.as_bytes()).unwrap();
                }));
            }
            for handler in handlers {
                handler.join().unwrap();
            }
        });
        let _server = ServerGuard {
            stop,
            thread: Some(server),
        };
        let script = format!("#!/bin/sh\necho call >> calls\necho '{{\"status\":\"running\",\"ready\":true,\"port\":{},\"pid\":\"42\",\"dataRoot\":\"/tmp/fixture\"}}'\n",port);
        let mut source = script_source(root.path(), &script);
        let calls = || {
            std::fs::read_to_string(root.path().join("calls"))
                .unwrap()
                .lines()
                .count()
        };
        source.fetch().unwrap();
        source.fetch_light().unwrap();
        source.fetch_light().unwrap();
        assert_eq!(
            calls(),
            1,
            "steady health and readiness reuse full diagnostic"
        );
        identity.lock().unwrap()["pid"] = serde_json::json!(43);
        source.fetch_light().unwrap();
        assert_eq!(calls(), 2);
        identity.lock().unwrap()["pid"] = serde_json::json!(42);
        source.fetch_light().unwrap();
        identity.lock().unwrap()["uptime"] = serde_json::json!(1.0);
        source.fetch_light().unwrap();
        assert_eq!(calls(), 3, "restart loses continuity");
        source.cache.lock().unwrap().as_mut().unwrap().at =
            Instant::now() - crate::modules::runtime_defaults::status_full_diagnostic_interval();
        source.fetch_light().unwrap();
        assert_eq!(calls(), 4);
        source.set_environment(root.path().join("new-data"));
        source.fetch_light().unwrap();
        assert_eq!(calls(), 5);
        identity.lock().unwrap()["service"] = serde_json::json!("foreign-service");
        source.fetch_light().unwrap();
        assert_eq!(calls(), 6);
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn oversized_diagnostic_is_bounded_and_does_not_seed_cache() {
        let root = tempfile::tempdir().unwrap();
        let source = script_source(
            root.path(),
            "#!/bin/sh\n/usr/bin/head -c 1048576 /dev/zero\n",
        );
        assert_eq!(source.fetch().unwrap_err(), CollectError::Parse);
        assert!(source.cache.lock().unwrap().is_none());
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn timeout_reaps_only_its_diagnostic_group() {
        let root = tempfile::tempdir().unwrap();
        let source = script_source(
            root.path(),
            "#!/bin/sh\necho $$ > diagnostic.pid\nsleep 60 &\necho $! > child.pid\nwait\n",
        );
        let mut unrelated = std::process::Command::new("/bin/sleep")
            .arg("60")
            .spawn()
            .unwrap();
        assert_eq!(source.fetch().unwrap_err(), CollectError::Timeout);
        let pid: i32 = std::fs::read_to_string(root.path().join("diagnostic.pid"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        assert_ne!(
            unsafe { libc::kill(pid, 0) },
            0,
            "diagnostic process reaped"
        );
        assert!(
            unrelated.try_wait().unwrap().is_none(),
            "independent process survives"
        );
        unrelated.kill().unwrap();
        unrelated.wait().unwrap();
    }
}
