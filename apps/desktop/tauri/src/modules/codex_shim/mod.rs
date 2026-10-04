//! MOD-05：「随 Codex 启动 OpenCodex」官方 shim 开关契约。
//!
//! 该开关的配置源与**写入通道都跟随官方**：读走 `ocx codex-shim status`，
//! 写走 `ocx codex-shim install|uninstall`。管理器不直接改写官方配置
//! （AC-11「不做旁路写入」）。本模块只解释受控来源输出，不访问文件、
//! 网络、Keychain 或系统 PATH。

use serde::{Deserialize, Serialize};

use crate::errors::AppError;

/// 单次读写窗口；shim 安装会改写 Codex 启动包装，给足余量。
pub fn codex_shim_timeout() -> std::time::Duration {
    crate::modules::runtime_defaults::shim_timeout()
}

/// 摘要行长度上限：状态原文含包装 / 备份绝对路径，界面只需要一个可读结论。
pub const CODEX_SHIM_SUMMARY_MAX: usize = 240;

/// `ocx codex-shim status` 的三种稳定结论。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodexShimState {
    /// 已安装（含「已安装但状态异常」，此时仍视为已开启，由官方 CLI 负责修复）。
    Installed,
    NotInstalled,
    /// 未解析出运行来源，或官方输出无法识别；界面据此禁用开关。
    Unreachable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodexShimError {
    Unreachable,
    Timeout,
    Failed,
    /// 官方 CLI 退出码为 0 但没有真正生效（例如找不到 `codex` 可执行文件）；
    /// `detail` 是官方输出的首行，供界面给出可执行的中文解释。
    Rejected(String),
}

impl From<CodexShimError> for AppError {
    fn from(value: CodexShimError) -> Self {
        match value {
            CodexShimError::Unreachable | CodexShimError::Failed => Self::NotConfigured,
            CodexShimError::Timeout => Self::Timeout,
            CodexShimError::Rejected(detail) => Self::RuntimeManaged {
                code: "codex_shim_rejected".to_string(),
                detail,
            },
        }
    }
}

/// 领域层注入的官方 shim 来源；真实子进程由 infrastructure 实现一次。
pub trait CodexShimSource {
    /// 只读：`ocx codex-shim status`。
    fn status(&self) -> Result<CodexShimOutput, CodexShimError>;
    /// 写入：经官方 CLI 安装 / 卸载 shim，返回变更后的状态输出。
    fn set_enabled(&self, enabled: bool) -> Result<CodexShimOutput, CodexShimError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexShimOutput {
    pub stdout: Vec<u8>,
    /// 官方失败原因（例如 macOS 的 EPERM）。只读查询用不到，但**写入失败必须能解释**，
    /// 否则界面只能给一句没有信息量的「写入失败」。
    pub stderr: Vec<u8>,
    pub exit_code: i32,
}

impl CodexShimOutput {
    /// 只带 stdout 的构造（纯解析与单测用）。
    pub fn from_stdout(stdout: impl Into<Vec<u8>>) -> Self {
        Self {
            stdout: stdout.into(),
            stderr: Vec::new(),
            exit_code: 0,
        }
    }
}

/// 脱敏后的领域结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodexShimReport {
    pub state: CodexShimState,
    pub summary: String,
}

impl CodexShimReport {
    /// 从受控来源输出解析；输出不可识别时按 `Unreachable` 处理，不猜状态。
    pub fn from_output(output: &CodexShimOutput) -> Self {
        let raw = String::from_utf8_lossy(&output.stdout);
        let state = parse_state(&raw);
        let summary = raw
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(summarize)
            .unwrap_or_default();
        Self { state, summary }
    }
}

/// 官方 `codex-shim status` 的稳定判定；先判「未安装」再判「已安装」。
pub fn parse_state(text: &str) -> CodexShimState {
    let lowered = text.to_lowercase();
    if lowered.contains("not installed") {
        CodexShimState::NotInstalled
    } else if lowered.contains("autostart shim") || lowered.contains("opencodex shim") {
        CodexShimState::Installed
    } else {
        CodexShimState::Unreachable
    }
}

fn summarize(line: &str) -> String {
    let sanitized = crate::modules::logs::sanitize_line(line).unwrap_or_else(|| line.to_string());
    let sanitized = sanitized.replace(['\r', '\u{0}', '\u{7}'], "");
    sanitized.chars().take(CODEX_SHIM_SUMMARY_MAX).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_not_installed_before_installed() {
        assert_eq!(
            parse_state("Codex autostart shim is not installed."),
            CodexShimState::NotInstalled
        );
    }

    #[test]
    fn parses_installed_summary_and_corrupt_state() {
        assert_eq!(
            parse_state(
                "Codex autostart shim: wrapper shim present at /x/codex; original backup present at /x/codex.opencodex-real."
            ),
            CodexShimState::Installed
        );
        assert_eq!(
            parse_state("Codex autostart shim state is invalid or corrupt at /x/codex-shim.json."),
            CodexShimState::Installed
        );
    }

    #[test]
    fn unrecognized_output_is_unreachable_not_guessed() {
        assert_eq!(parse_state(""), CodexShimState::Unreachable);
        assert_eq!(
            parse_state("command not found"),
            CodexShimState::Unreachable
        );
    }

    #[test]
    fn report_keeps_first_line_and_truncates_tail() {
        let report = CodexShimReport::from_output(&CodexShimOutput::from_stdout(
            b"Codex autostart shim is not installed.\nsecond line\n".to_vec(),
        ));
        assert_eq!(report.state, CodexShimState::NotInstalled);
        assert_eq!(report.summary, "Codex autostart shim is not installed.");

        let long = format!("shim {}\n", "a".repeat(400));
        let report = CodexShimReport::from_output(&CodexShimOutput::from_stdout(long.into_bytes()));
        assert_eq!(report.summary.chars().count(), CODEX_SHIM_SUMMARY_MAX);
    }
}
