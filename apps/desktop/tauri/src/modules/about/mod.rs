//! MOD-02 / MOD-06：关于页领域契约。
//!
//! 本模块只投影受控来源返回的官方版本与应用契约；不访问网络，不执行
//! 安装、更新或写入。许可边界文本由 UI 展示，来源与版本解析保持分离。

use serde::{Deserialize, Serialize};

use crate::errors::AppError;

/// 单次官方版本查询超时；低于 Doctor 的长诊断窗口。
pub fn official_version_timeout() -> std::time::Duration {
    crate::modules::runtime_defaults::version_probe_timeout()
}

/// 官方版本第一行输出上限；防止失控输出拖慢 WebView 或膨胀 DTO。
pub const OFFICIAL_VERSION_MAX_CHARS: usize = 240;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfficialVersionError {
    Unreachable,
    Timeout,
    Parse,
}

impl From<OfficialVersionError> for AppError {
    fn from(value: OfficialVersionError) -> Self {
        match value {
            OfficialVersionError::Unreachable => Self::NotConfigured,
            OfficialVersionError::Timeout => Self::Timeout,
            OfficialVersionError::Parse => Self::LogSanitization,
        }
    }
}

/// 领域层注入的官方版本来源；真实子进程由 infrastructure 实现一次。
pub trait OfficialVersionSource {
    fn run(&self) -> Result<OfficialVersionOutput, OfficialVersionError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfficialVersionOutput {
    pub stdout: Vec<u8>,
}

/// 官方版本事实；只保留名称、版本与输出形态，不携带路径细节。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OfficialProjectFacts {
    pub display_name: String,
    pub version: Option<String>,
    pub raw_version: String,
    pub truncated: bool,
}

impl OfficialProjectFacts {
    /// 从受控来源输出解析第一行；逐行控制符清除，失败时返回明确错误。
    pub fn from_output(output: &OfficialVersionOutput) -> Result<Self, AppError> {
        let raw = String::from_utf8_lossy(&output.stdout);
        let first_line = raw
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .ok_or(AppError::LogSanitization)?;
        let sanitized: String = first_line
            .chars()
            .filter(|character| !character.is_control())
            .collect();
        if sanitized.is_empty() {
            return Err(AppError::LogSanitization);
        }
        let (display_name, version) = split_name_version(&sanitized);
        let truncated = sanitized.chars().count() > OFFICIAL_VERSION_MAX_CHARS;
        let bounded: String = sanitized.chars().take(OFFICIAL_VERSION_MAX_CHARS).collect();
        Ok(Self {
            display_name: display_name.unwrap_or_else(|| bounded.clone()),
            version,
            raw_version: bounded,
            truncated,
        })
    }

    pub fn run<S: OfficialVersionSource + ?Sized>(source: &S) -> Result<Self, AppError> {
        match source.run() {
            Ok(output) => Self::from_output(&output),
            Err(error) => Err(error.into()),
        }
    }
}

fn split_name_version(line: &str) -> (Option<String>, Option<String>) {
    let mut parts = line.split_whitespace();
    let name = parts.next().map(str::to_string);
    let version = parts
        .find(|part| part.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .map(str::to_string);
    (name, version)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixtureSource(Result<OfficialVersionOutput, OfficialVersionError>);

    impl Clone for FixtureSource {
        fn clone(&self) -> Self {
            match &self.0 {
                Ok(output) => Self(Ok(output.clone())),
                Err(error) => Self(Err(*error)),
            }
        }
    }

    impl OfficialVersionSource for FixtureSource {
        fn run(&self) -> Result<OfficialVersionOutput, OfficialVersionError> {
            self.0.clone()
        }
    }

    #[test]
    fn parses_name_and_version() {
        let output = OfficialVersionOutput {
            stdout: b"OpenCodex 2.50.0\n".to_vec(),
        };
        let facts = OfficialProjectFacts::from_output(&output).expect("facts");
        assert_eq!(facts.display_name, "OpenCodex");
        assert_eq!(facts.version.as_deref(), Some("2.50.0"));
        assert_eq!(facts.raw_version, "OpenCodex 2.50.0");
        assert!(!facts.truncated);
    }

    #[test]
    fn rejects_empty_and_control_only_output() {
        for stdout in [Vec::new(), b"\r\n".to_vec(), [0x07].to_vec()] {
            let output = OfficialVersionOutput { stdout };
            assert!(OfficialProjectFacts::from_output(&output).is_err());
        }
    }

    #[test]
    fn bounds_unbounded_output() {
        let output = OfficialVersionOutput {
            stdout: format!("{} 2.0.0\n", "x".repeat(OFFICIAL_VERSION_MAX_CHARS + 10)).into_bytes(),
        };
        let facts = OfficialProjectFacts::from_output(&output).expect("facts");
        assert!(facts.truncated);
        assert_eq!(
            facts.raw_version.chars().count(),
            OFFICIAL_VERSION_MAX_CHARS
        );
    }

    #[test]
    fn maps_source_errors() {
        for error in [
            OfficialVersionError::Unreachable,
            OfficialVersionError::Timeout,
            OfficialVersionError::Parse,
        ] {
            let source = FixtureSource(Err(error));
            assert!(OfficialProjectFacts::run(&source).is_err());
        }
    }
}
