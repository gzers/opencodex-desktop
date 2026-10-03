//! 受控官方 shim 开关来源：只在 infrastructure 执行显式路径的
//! `ocx codex-shim status|install|uninstall`。
//!
//! 不搜索 PATH；环境先清空后仅注入冻结键。写入只经由官方 CLI，管理器不旁路
//! 改写官方配置。写入失败时官方会把原因写在 stderr（例如 macOS 的 EPERM），
//! 因此**写入路径必须采集 stderr**，否则界面只能给一句没有信息量的失败提示。

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::process::Command;

use crate::modules::codex_shim::{
    parse_state, CodexShimError, CodexShimOutput, CodexShimSource, CodexShimState,
    CODEX_SHIM_TIMEOUT,
};
use crate::modules::process::EnvironmentPolicy;

/// `codex` 可执行文件的受控候选目录（**不读 PATH**，与运行来源的发现口径一致）。
///
/// 官方 `ocx codex-shim install` 只会在 `PATH` 上找 `codex`：找不到时它打印
/// 「⚠️ Could not find a codex executable on PATH.」但**退出码仍是 0**，等于什么都没做。
/// 管理器给子进程的 `PATH` 原本只有 node 所在目录（`FZ-13` 的受控发现结果），
/// 于是 macOS 上很常见的「codex 来自 ChatGPT.app 内置」这一类安装永远装不上，
/// 界面表现为开关点不动。这里按固定候选补齐，并逐个校验确实存在 `codex`。
pub const CODEX_EXECUTABLE_CANDIDATE_DIRS: &[&str] = &[
    // macOS 桌面版 ChatGPT 内置的 codex CLI。
    "/Applications/ChatGPT.app/Contents/Resources",
    // npm 全局前缀与 Homebrew / 系统前缀（`npm i -g @openai/codex` 的常见落点）。
    "/opt/homebrew/bin",
    "/usr/local/bin",
];

/// 显式路径 `ocx codex-shim ...` 来源。
#[derive(Debug)]
pub struct OfficialCodexShimSource {
    runtime: crate::infrastructure::runtime_executable::SharedRuntimeExecutable,
    working_directory: PathBuf,
    environment: EnvironmentPolicy,
}

impl OfficialCodexShimSource {
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
    fn command(&self, executable: &Path, args: &[&str]) -> Command {
        let mut process = Command::new(executable);
        process
            .args(args)
            .current_dir(&self.working_directory)
            .env_clear()
            .env("HOME", self.environment.home.clone().unwrap_or_default())
            .env("OPENCODEX_HOME", &self.environment.opencodex_home)
            .env("PATH", self.child_path())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(false);
        for (key, value) in [
            ("LANG", &self.environment.lang),
            ("LC_ALL", &self.environment.lc_all),
            ("HTTP_PROXY", &self.environment.http_proxy),
            ("HTTPS_PROXY", &self.environment.https_proxy),
            ("NO_PROXY", &self.environment.no_proxy),
        ] {
            if let Some(value) = value {
                process.env(key, value);
            }
        }
        process
    }

    /// 子进程的 `PATH`：冻结的 node 目录在前，其后是**确实含 `codex`** 的候选目录。
    fn child_path(&self) -> OsString {
        let mut dirs: Vec<PathBuf> = self
            .environment
            .path
            .as_ref()
            .map(|path| std::env::split_paths(path).collect())
            .unwrap_or_default();
        for candidate in self.codex_candidate_dirs() {
            if !candidate.join("codex").is_file() || dirs.contains(&candidate) {
                continue;
            }
            dirs.push(candidate);
        }
        std::env::join_paths(dirs).unwrap_or_default()
    }

    fn codex_candidate_dirs(&self) -> Vec<PathBuf> {
        let mut dirs: Vec<PathBuf> = Vec::new();
        if let Some(home) = self.environment.home.as_ref() {
            dirs.push(PathBuf::from(home).join(".local/bin"));
        }
        dirs.extend(CODEX_EXECUTABLE_CANDIDATE_DIRS.iter().map(PathBuf::from));
        dirs
    }

    /// 执行一次官方命令，**不按退出码拒绝**：把 stdout / stderr / 退出码都带回来，
    /// 由调用方决定「退出码 0 但没生效」这类官方行为算成功还是失败。
    fn run_capturing(&self, args: &[&str]) -> Result<CodexShimOutput, CodexShimError> {
        // 未解析出运行来源时按「不可达」处理，界面据此禁用开关。
        let executable = self
            .runtime
            .executable()
            .ok_or(CodexShimError::Unreachable)?;
        validate_paths(&executable, &self.working_directory)?;

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| CodexShimError::Unreachable)?;
        runtime.block_on(async {
            let child = self
                .command(&executable, args)
                .spawn()
                .map_err(|_| CodexShimError::Unreachable)?;
            let output = tokio::time::timeout(CODEX_SHIM_TIMEOUT, child.wait_with_output())
                .await
                .map_err(|_| CodexShimError::Timeout)?
                .map_err(|_| CodexShimError::Unreachable)?;
            Ok(CodexShimOutput {
                stdout: output.stdout,
                stderr: output.stderr,
                exit_code: output.status.code().unwrap_or(-1),
            })
        })
    }

    /// 只读查询：非零退出即视为失败（界面退回只读事实，不给假开关）。
    fn run_strict(&self, args: &[&str]) -> Result<CodexShimOutput, CodexShimError> {
        let output = self.run_capturing(args)?;
        if output.exit_code != 0 {
            return Err(CodexShimError::Failed);
        }
        Ok(output)
    }
}

impl CodexShimSource for OfficialCodexShimSource {
    fn status(&self) -> Result<CodexShimOutput, CodexShimError> {
        self.run_strict(&["codex-shim", "status"])
    }

    fn set_enabled(&self, enabled: bool) -> Result<CodexShimOutput, CodexShimError> {
        let subcommand = if enabled { "install" } else { "uninstall" };
        let action = self.run_capturing(&["codex-shim", subcommand])?;
        // 以官方 `status` 复读为准，避免把官方提示语当成状态事实。
        let status = self.run_capturing(&["codex-shim", "status"])?;
        // 官方 CLI 在「找不到 codex」这类情况下仍然退出 0：只看退出码会把
        // 「什么都没做」当成功。因此按**目标状态**校验，不一致就是失败。
        let expected = if enabled {
            CodexShimState::Installed
        } else {
            CodexShimState::NotInstalled
        };
        let actual = parse_state(&String::from_utf8_lossy(&status.stdout));
        if actual != expected || action.exit_code != 0 {
            let detail = {
                let from_action = diagnostic(&action);
                if from_action.is_empty() {
                    diagnostic(&status)
                } else {
                    from_action
                }
            };
            return Err(CodexShimError::Rejected(detail));
        }
        Ok(status)
    }
}

/// 官方失败原因。
///
/// 成功 / 已知前置缺失时官方把结论写在 **stdout 首行**；真正的异常（例如 macOS 拒绝
/// 改动别的应用包）则是一段 **stderr 堆栈**，首行往往只是源码上下文而不是原因本身
/// （真机实测：`EPERM: operation not permitted, rename …` 出现在第 6 行左右）。
/// 因此 stdout 为空时保留**整段脱敏后的 stderr**（截断），让调用方还能找到原因。
fn diagnostic(output: &CodexShimOutput) -> String {
    if let Some(line) = first_line(&output.stdout) {
        return sanitize(&line, CODEX_SHIM_DIAGNOSTIC_LIMIT);
    }
    sanitize(
        &String::from_utf8_lossy(&output.stderr),
        CODEX_SHIM_DIAGNOSTIC_LIMIT,
    )
}

/// 输出的首个非空行；官方在成功与失败两侧都先给一行结论。
fn first_line(bytes: &[u8]) -> Option<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
}

/// 逐行脱敏并截断；官方 stderr 里含绝对路径与源码片段，脱敏后才允许离开适配器。
fn sanitize(text: &str, limit: usize) -> String {
    let mut out = String::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cleaned = crate::modules::logs::sanitize_line(line).unwrap_or_else(|| line.to_string());
        out.push_str(&cleaned);
        out.push('\n');
        if out.chars().count() >= limit {
            break;
        }
    }
    out.chars().take(limit).collect()
}

const CODEX_SHIM_DIAGNOSTIC_LIMIT: usize = 800;

fn validate_paths(executable: &Path, working_directory: &Path) -> Result<(), CodexShimError> {
    if !executable.is_file() || !working_directory.is_dir() {
        return Err(CodexShimError::Unreachable);
    }
    Ok(())
}
