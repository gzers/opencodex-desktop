//! 受控官方状态来源：只在 infrastructure 执行显式路径的 `ocx status --json`。
//!
//! FZ-07/FZ-08 规定 5 秒采集窗口；失败保留上一份状态。本层不搜索 PATH，
//! 环境先清空后仅注入冻结键，不读取真实用户配置或 Keychain。

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use tokio::process::Command;

use crate::modules::status::{CollectError, StatusSource};

/// 官方状态采集单次冻结窗口；与 FZ-08 保持同一常量。
pub const STATUS_COLLECT_TIMEOUT: Duration = crate::modules::status::COLLECT_TIMEOUT;

/// 显式路径官方状态来源。
#[derive(Debug)]
pub struct OfficialStatusSource {
    runtime: crate::infrastructure::runtime_executable::SharedRuntimeExecutable,
    working_directory: PathBuf,
    environment: crate::modules::process::EnvironmentPolicy,
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
        }
    }

    pub fn set_environment(&mut self, opencodex_home: PathBuf) {
        self.environment.opencodex_home = opencodex_home;
    }

    /// 每次采集都重新向共享句柄取当前来源，安装 / 卸载后无需重建来源对象。
    fn command(&self, executable: &std::path::Path) -> Command {
        let mut process = Command::new(executable);
        process
            .arg("status")
            .arg("--json")
            .current_dir(&self.working_directory)
            .env_clear()
            .env("HOME", self.environment.home.clone().unwrap_or_default())
            .env("OPENCODEX_HOME", &self.environment.opencodex_home)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(false);
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

        let child = self
            .command(&executable)
            .spawn()
            .map_err(|_| CollectError::Unreachable)?;
        let output = tokio::time::timeout(STATUS_COLLECT_TIMEOUT, child.wait_with_output())
            .await
            .map_err(|_| CollectError::Timeout)?
            .map_err(|_| CollectError::Unreachable)?;

        if !output.status.success() {
            return Err(CollectError::Unreachable);
        }
        serde_json::from_slice(&output.stdout).map_err(|_| CollectError::Parse)
    }
}

impl StatusSource for OfficialStatusSource {
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
                STATUS_COLLECT_TIMEOUT.as_secs() + 1
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
}
