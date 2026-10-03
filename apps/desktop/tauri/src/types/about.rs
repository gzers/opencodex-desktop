//! 关于页前端 DTO；领域事实与 WebView 契约保持分离。

use serde::{Deserialize, Serialize};

use crate::modules::about::OfficialProjectFacts;
#[cfg(test)]
use crate::modules::about::OfficialVersionOutput;

/// 应用自身契约；来源是构建配置与运行时元数据，不是硬编码 UI 文案。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AboutAppDto {
    pub name: String,
    pub version: String,
    pub identifier: String,
    pub platform: String,
    pub framework: String,
    pub license: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficialProjectDto {
    pub display_name: String,
    pub version: Option<String>,
    pub raw_version: String,
    pub truncated: bool,
}

impl From<OfficialProjectFacts> for OfficialProjectDto {
    fn from(value: OfficialProjectFacts) -> Self {
        Self {
            display_name: value.display_name,
            version: value.version,
            raw_version: value.raw_version,
            truncated: value.truncated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_dto_is_camel_case() {
        let value = AboutAppDto {
            name: "OpenCodeX-Desktop".into(),
            version: "0.1.0".into(),
            identifier: "com.gzers.opencodex.desktop".into(),
            platform: "macOS".into(),
            framework: "Tauri v2".into(),
            license: "MIT License".into(),
        };
        let payload = serde_json::to_value(&value).expect("dto");
        assert_eq!(payload["name"], "OpenCodeX-Desktop");
        assert_eq!(payload["version"], "0.1.0");
        assert_eq!(payload["identifier"], "com.gzers.opencodex.desktop");
    }

    #[test]
    fn official_dto_is_camel_case() {
        let facts = OfficialProjectFacts::from_output(&OfficialVersionOutput {
            stdout: b"OpenCodex 2.50.0\n".to_vec(),
        })
        .expect("facts");
        let payload = serde_json::to_value(OfficialProjectDto::from(facts)).expect("dto");
        assert_eq!(payload["displayName"], "OpenCodex");
        assert_eq!(payload["version"], "2.50.0");
        assert_eq!(payload["truncated"], false);
    }
}
