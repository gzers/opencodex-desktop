//! 受控官方 CLI 来源：只在 infrastructure 执行显式路径的只读命令。
//!
//! 不搜索 PATH；环境先清空后仅注入冻结键。stderr 不采集，避免把
//! 官方进度或本地路径直接暴露到界面。

use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::{io::AsyncReadExt, process::Command, time::Instant};

use crate::modules::data_root::{
    check_partitions,
    startup_v2::{check_owner, checked_path},
};
use crate::modules::doctor::{DoctorError, DoctorOutput, DoctorSource};
use crate::modules::process::EnvironmentPolicy;

const MAX_OUTPUT_BYTES: u64 = 256 * 1024;

/// 显式路径 `ocx doctor` 只读来源。
#[derive(Debug)]
pub struct OfficialDoctorSource {
    runtime: crate::infrastructure::runtime_executable::SharedRuntimeExecutable,
    desktop_root: PathBuf,
    environment: EnvironmentPolicy,
}

impl OfficialDoctorSource {
    /// 构造来源；不立即执行命令、不验证路径内容。
    pub fn new(
        runtime: crate::infrastructure::runtime_executable::SharedRuntimeExecutable,
        desktop_root: impl Into<PathBuf>,
        environment: EnvironmentPolicy,
    ) -> Self {
        Self {
            runtime,
            desktop_root: desktop_root.into(),
            environment,
        }
    }

    /// 每次运行都重新取当前来源，安装 / 卸载后无需重建来源对象。
    fn command(&self, executable: &Path, cwd: &Path) -> Command {
        let mut process = Command::new(executable);
        process
            .arg("doctor")
            .current_dir(cwd)
            .env_clear()
            .env("OPENCODEX_HOME", &self.environment.opencodex_home)
            .env(
                "CODEX_HOME",
                crate::modules::process::client_home(&self.environment.opencodex_home),
            )
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        #[cfg(unix)]
        process.process_group(0);
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
}

impl DoctorSource for OfficialDoctorSource {
    fn run(&self) -> Result<DoctorOutput, DoctorError> {
        let runtime = if self.executable().is_some() {
            "已发现"
        } else {
            "未发现"
        };
        let cwd = if self.checked_cwd().is_some() {
            "可用"
        } else {
            "不可用"
        };
        Ok(report(&format!(
            "OpenCodex 程序：{runtime}
诊断工作目录：{cwd}
本地检查完成。可手动执行联网诊断。"
        )))
    }

    fn run_network(&self) -> Result<DoctorOutput, DoctorError> {
        let Some(cwd) = self.checked_cwd() else {
            return Ok(report("联网诊断未启动：诊断工作目录不可用。"));
        };
        let Some(executable) = self.executable() else {
            return Ok(report("联网诊断未启动：未发现可用的 OpenCodex 程序。"));
        };
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| DoctorError::Unreachable)?;
        Ok(runtime.block_on(run_command(self.command(&executable, &cwd))))
    }
}

impl OfficialDoctorSource {
    fn checked_cwd(&self) -> Option<PathBuf> {
        let root = checked_path(&self.desktop_root).ok()?;
        check_owner(&root).ok()?;
        check_partitions(&root).ok()?;
        let cwd = root.join("cache");
        cwd.is_dir().then_some(cwd)
    }

    fn executable(&self) -> Option<PathBuf> {
        self.runtime
            .executable()
            .filter(|path| path.is_absolute() && path.is_file())
    }
}

fn report(text: &str) -> DoctorOutput {
    DoctorOutput {
        stdout: text.as_bytes().to_vec(),
    }
}

async fn run_command(mut command: Command) -> DoctorOutput {
    let deadline = Instant::now() + crate::modules::doctor::doctor_timeout();
    #[cfg(windows)]
    let Ok(job) = crate::infrastructure::windows_query_job::OwnedQueryJob::new(&mut command) else {
        return report("联网诊断未启动：无法创建受控进程组。");
    };
    let Ok(mut child) = command.spawn() else {
        return report("联网诊断未启动：无法创建官方诊断进程。");
    };
    #[cfg(windows)]
    if job.assign_and_resume(&child).is_err() {
        let _ = child.start_kill();
        let exited = matches!(
            tokio::time::timeout_at(deadline, child.wait()).await,
            Ok(Ok(_))
        );
        let job_exited = job.terminate_and_wait().await.is_ok();
        return failed("无法验证并启动受控诊断进程", exited && job_exited);
    }
    #[cfg(unix)]
    let group = child.id().and_then(|pid| libc::pid_t::try_from(pid).ok());
    let mut stdout = Vec::new();
    let result = match child.stdout.take() {
        Some(pipe) => Some(tokio::join!(
            tokio::time::timeout_at(deadline, child.wait()),
            tokio::time::timeout_at(deadline, async {
                pipe.take(MAX_OUTPUT_BYTES + 1)
                    .read_to_end(&mut stdout)
                    .await
            }),
        )),
        None => None,
    };
    #[cfg(windows)]
    let cleaned = job.terminate_and_wait().await.is_ok();
    #[cfg(unix)]
    let cleaned = terminate_group(group, &mut child).await;
    #[cfg(not(any(unix, windows)))]
    let cleaned = false;
    let Some((root, output)) = result else {
        return failed("无法打开报告输出", cleaned);
    };
    let error = match (root, output) {
        (Err(_), _) => "等待诊断进程退出超时（5 秒）",
        (_, Err(_)) => "等待读完诊断输出超时（5 秒）",
        (Ok(Ok(exit)), Ok(Ok(_))) if !exit.success() => {
            return failed(&format!("官方诊断进程退出异常（{exit}）"), cleaned);
        }
        (Ok(Ok(_)), Ok(Ok(_))) if stdout.is_empty() => "官方诊断未返回报告",
        (Ok(Ok(_)), Ok(Ok(size))) if size as u64 > MAX_OUTPUT_BYTES => "官方诊断输出超过上限",
        (Ok(Ok(_)), Ok(Ok(_))) if !cleaned => "诊断进程退出后清理未确认",
        (Ok(Ok(_)), Ok(Ok(_))) => return DoctorOutput { stdout },
        _ => "无法读取进程退出状态或诊断输出",
    };
    failed(error, cleaned)
}

fn failed(reason: &str, cleaned: bool) -> DoctorOutput {
    let cleanup = if cleaned {
        "受控进程已退出。"
    } else {
        "受控进程退出未确认。"
    };
    report(&format!(
        "联网诊断未完成：{reason}。
{cleanup}"
    ))
}

#[cfg(unix)]
async fn terminate_group(group: Option<libc::pid_t>, child: &mut tokio::process::Child) -> bool {
    let Some(group) = group else {
        return false;
    };
    if unsafe { libc::killpg(group, libc::SIGKILL) } != 0
        && std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
    {
        return false;
    }
    matches!(
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            child.wait().await?;
            loop {
                if unsafe { libc::killpg(group, 0) } != 0 {
                    return Ok::<_, std::io::Error>(
                        std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH),
                    );
                }
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            }
        })
        .await,
        Ok(Ok(true))
    )
}
