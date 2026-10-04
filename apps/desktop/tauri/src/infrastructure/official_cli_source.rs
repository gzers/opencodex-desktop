//! 受控官方 CLI 来源：只在 infrastructure 执行显式路径的只读命令。
//!
//! 不搜索 PATH；环境先清空后仅注入冻结键。stderr 不采集，避免把
//! 官方进度或本地路径直接暴露到界面。

use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::process::Command;

use crate::modules::doctor::{DoctorError, DoctorOutput, DoctorSource};
use crate::modules::process::EnvironmentPolicy;

/// 显式路径 `ocx doctor` 只读来源。
#[derive(Debug)]
pub struct OfficialDoctorSource {
    runtime: crate::infrastructure::runtime_executable::SharedRuntimeExecutable,
    working_directory: PathBuf,
    environment: EnvironmentPolicy,
}

impl OfficialDoctorSource {
    /// 构造来源；不立即执行命令、不验证路径内容。
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
    fn command(&self, executable: &Path) -> Command {
        let mut process = Command::new(executable);
        process
            .arg("doctor")
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

impl DoctorSource for OfficialDoctorSource {
    fn run(&self) -> Result<DoctorOutput, DoctorError> {
        // 未解析出运行来源时按「不可达」处理，界面据此展示门禁引导。
        let executable = self.runtime.executable().ok_or(DoctorError::Unreachable)?;
        validate_paths(&executable, &self.working_directory)?;

        // Doctor 是同步 trait 契约；用独立 current_thread runtime 隔离执行，
        // 避免嵌套阻塞现有 Tauri runtime。
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| DoctorError::Unreachable)?;
        runtime.block_on(async {
            let child = self
                .command(&executable)
                .spawn()
                .map_err(|_| DoctorError::Unreachable)?;
            let output = tokio::time::timeout(
                crate::modules::doctor::doctor_timeout(),
                child.wait_with_output(),
            )
            .await
            .map_err(|_| DoctorError::Timeout)?
            .map_err(|_| DoctorError::Unreachable)?;

            if !output.status.success() {
                return Err(DoctorError::Unreachable);
            }
            Ok(DoctorOutput {
                stdout: output.stdout,
            })
        })
    }
}

fn validate_paths(executable: &Path, working_directory: &Path) -> Result<(), DoctorError> {
    if !executable.is_file() || !working_directory.is_dir() {
        return Err(DoctorError::Unreachable);
    }
    Ok(())
}
