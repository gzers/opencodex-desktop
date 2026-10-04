//! MOD-04：官方 Doctor 只读领域契约。
//!
//! 本模块只解释受控来源返回的官方输出；不执行写入型修复，不访问文件、
//! 网络、Keychain 或系统 PATH。平台差异由 infrastructure 适配器承担。

use serde::{Deserialize, Serialize};

use crate::errors::AppError;

/// FZ-08 冻结的单次采集窗口；Doctor 与状态采集保持同一只读语义。
pub fn doctor_timeout() -> std::time::Duration {
    crate::modules::runtime_defaults::doctor_timeout()
}

/// 输出行数上限；防止失控输出拖慢 WebView 或膨胀 DTO。
pub const DOCTOR_MAX_LINES: usize = 400;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoctorMode {
    ReadOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoctorError {
    Unreachable,
    Timeout,
    Parse,
}

impl From<DoctorError> for AppError {
    fn from(value: DoctorError) -> Self {
        match value {
            DoctorError::Unreachable => Self::NotConfigured,
            DoctorError::Timeout => Self::Timeout,
            DoctorError::Parse => Self::NotConfigured,
        }
    }
}

/// 领域层注入的官方命令来源；真实子进程由 infrastructure 实现一次。
pub trait DoctorSource {
    fn run(&self) -> Result<DoctorOutput, DoctorError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorOutput {
    pub stdout: Vec<u8>,
}

/// 脱敏后的领域结果；`truncated` 表示尾部输出被安全丢弃。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorReport {
    pub mode: DoctorMode,
    pub lines: Vec<String>,
    pub truncated: bool,
}

impl Default for DoctorReport {
    fn default() -> Self {
        Self {
            mode: DoctorMode::ReadOnly,
            lines: Vec::new(),
            truncated: false,
        }
    }
}

impl DoctorReport {
    /// 从受控来源输出解析并逐行脱敏；无法安全脱敏时明确失败，不输出半脱敏内容。
    pub fn from_output(output: &DoctorOutput) -> Result<Self, AppError> {
        let raw = String::from_utf8_lossy(&output.stdout);
        let mut lines = Vec::new();
        let mut truncated = false;

        for line in raw.lines() {
            if lines.len() >= DOCTOR_MAX_LINES {
                truncated = true;
                break;
            }
            let line = line.replace(['\r', '\u{0}', '\u{7}'], "");
            if line.contains('\n') {
                return Err(AppError::LogSanitization);
            }
            let sanitized =
                crate::modules::logs::sanitize_line(&line).ok_or(AppError::LogSanitization)?;
            lines.push(sanitize_secret_assignments(&sanitized));
        }

        // UTF-8 边界可能截断尾部；完整有效行已输出，仍按尾部截断处理。
        if raw.lines().count() > lines.len() {
            truncated = true;
        }

        Ok(Self {
            mode: DoctorMode::ReadOnly,
            lines,
            truncated,
        })
    }

    /// 编排只读 Doctor；不传入任何写入型修复参数。
    pub fn run<S: DoctorSource + ?Sized>(source: &S) -> Result<Self, AppError> {
        match source.run() {
            Ok(output) => Self::from_output(&output),
            Err(error) => Err(error.into()),
        }
    }
}

/// FZ-12 常见 key=value / key: value 敏感赋值形态；仅固定键名触发。
#[cfg_attr(not(test), allow(dead_code))]
fn sanitize_secret_assignments(line: &str) -> String {
    let mut output = line.to_string();
    for key in [
        "api_key",
        "apikey",
        "authorization",
        "password",
        "secret",
        "token",
        "cookie",
    ] {
        for separator in ["=", ": "] {
            let needle = format!("{key}{separator}");
            if let Some(start) = output.to_ascii_lowercase().find(&needle) {
                let value_start = start + needle.len();
                let value_end = output[value_start..]
                    .find(|character: char| character.is_whitespace() || character == '"')
                    .map(|position| value_start + position)
                    .unwrap_or(output.len());
                output.replace_range(start..value_end, &format!("{needle}[REDACTED]"));
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_only_is_serialized() {
        assert_eq!(
            serde_json::to_string(&DoctorMode::ReadOnly).unwrap(),
            "\"read_only\""
        );
    }

    #[test]
    fn parses_and_limits_output() {
        let output = DoctorOutput {
            stdout: b"opencodex doctor\n\nline three\n".to_vec(),
        };
        let report = DoctorReport::from_output(&output).expect("valid output");
        assert_eq!(report.mode, DoctorMode::ReadOnly);
        assert_eq!(report.lines, vec!["opencodex doctor", "", "line three"]);
        assert!(!report.truncated);
    }

    #[test]
    fn truncates_unbounded_output_after_frozen_limit() {
        let output = DoctorOutput {
            stdout: (0..DOCTOR_MAX_LINES + 10)
                .map(|index| format!("line-{index}\n"))
                .collect::<Vec<_>>()
                .join("")
                .into_bytes(),
        };
        let report = DoctorReport::from_output(&output).expect("valid output");
        assert_eq!(report.lines.len(), DOCTOR_MAX_LINES);
        assert!(report.truncated);
    }
}
