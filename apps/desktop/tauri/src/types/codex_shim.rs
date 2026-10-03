//! 「随 Codex 启动 OpenCodex」官方 shim 开关的前端 DTO。

use serde::{Deserialize, Serialize};

use crate::modules::codex_shim::{CodexShimReport, CodexShimState};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexShimDto {
    pub state: CodexShimState,
    /// 便于界面直接绑定开关；等价于 `state == installed`。
    pub installed: bool,
    /// 官方 `status` 首行的脱敏摘要；不可达时为空。
    pub summary: String,
    /// 不可达的稳定原因码（`unresolved` / `timeout` / `failed` / `locked`）；
    /// 可读时为 `""`。界面据此给中文说明，而不是把官方英文原文丢给用户。
    pub reason: String,
}

impl CodexShimDto {
    /// 读不到状态时的事实投影；界面据此禁用开关并说明原因，而不是显示假状态。
    pub fn unreachable(reason: impl Into<String>) -> Self {
        Self {
            state: CodexShimState::Unreachable,
            installed: false,
            summary: String::new(),
            reason: reason.into(),
        }
    }
}

impl From<CodexShimReport> for CodexShimDto {
    fn from(value: CodexShimReport) -> Self {
        Self {
            installed: matches!(value.state, CodexShimState::Installed),
            state: value.state,
            summary: value.summary,
            reason: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::codex_shim::CodexShimOutput;

    #[test]
    fn dto_uses_camel_case_and_maps_installed_flag() {
        let report = CodexShimReport::from_output(&CodexShimOutput::from_stdout(
            b"Codex autostart shim: wrapper shim present at /x/codex.\n".to_vec(),
        ));
        let payload = serde_json::to_value(CodexShimDto::from(report)).unwrap();
        assert_eq!(payload["state"], "installed");
        assert_eq!(payload["installed"], true);
        assert!(payload["summary"]
            .as_str()
            .unwrap()
            .contains("autostart shim"));
    }

    #[test]
    fn unreachable_dto_is_honest_and_empty() {
        let dto = CodexShimDto::unreachable("unresolved");
        assert_eq!(dto.state, CodexShimState::Unreachable);
        assert!(!dto.installed);
        assert!(dto.summary.is_empty());
        assert_eq!(dto.reason, "unresolved");
    }

    #[test]
    fn readable_dto_has_no_reason_code() {
        let report = CodexShimReport::from_output(&CodexShimOutput::from_stdout(
            b"Codex autostart shim is not installed.\n".to_vec(),
        ));
        let dto = CodexShimDto::from(report);
        assert_eq!(dto.state, CodexShimState::NotInstalled);
        assert!(dto.reason.is_empty());
    }
}
