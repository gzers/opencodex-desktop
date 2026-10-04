//! 受控官方版本来源：只在 infrastructure 执行显式路径的 `ocx --version`。
//!
//! 不搜索 PATH；环境先清空后仅注入冻结键。stderr 不采集，超时与
//! 非零退出都映射为明确失败，不向 UI 泄漏子进程细节。

use std::path::PathBuf;
use std::process::Stdio;

use tokio::process::Command;

use crate::modules::about::{OfficialVersionError, OfficialVersionOutput, OfficialVersionSource};
use crate::modules::process::EnvironmentPolicy;

#[derive(Debug)]
pub struct OfficialCliVersionSource {
    runtime: crate::infrastructure::runtime_executable::SharedRuntimeExecutable,
    working_directory: PathBuf,
    environment: EnvironmentPolicy,
}

impl OfficialCliVersionSource {
    pub fn new(
        runtime: crate::infrastructure::runtime_executable::SharedRuntimeExecutable,
        working_directory: impl Into<PathBuf>,
        environment: EnvironmentPolicy,
    ) -> Self {
        Self {
            runtime,
            working_directory: working_directory.into(),
            environment,
        }
    }

    /// 每次运行都重新取当前来源，安装 / 卸载后无需重建来源对象。
    fn command(&self, executable: &std::path::Path) -> Command {
        let mut process = Command::new(executable);
        process
            .arg("--version")
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
}

impl OfficialVersionSource for OfficialCliVersionSource {
    fn run(&self) -> Result<OfficialVersionOutput, OfficialVersionError> {
        let executable = self
            .runtime
            .executable()
            .ok_or(OfficialVersionError::Unreachable)?;
        if !executable.is_file() || !self.working_directory.is_dir() {
            return Err(OfficialVersionError::Unreachable);
        }
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| OfficialVersionError::Unreachable)?;
        runtime.block_on(async {
            let child = self
                .command(&executable)
                .spawn()
                .map_err(|_| OfficialVersionError::Unreachable)?;
            let output = tokio::time::timeout(
                crate::modules::about::official_version_timeout(),
                child.wait_with_output(),
            )
            .await
            .map_err(|_| OfficialVersionError::Timeout)?
            .map_err(|_| OfficialVersionError::Unreachable)?;
            if !output.status.success() {
                return Err(OfficialVersionError::Unreachable);
            }
            Ok(OfficialVersionOutput {
                stdout: output.stdout,
            })
        })
    }
}
