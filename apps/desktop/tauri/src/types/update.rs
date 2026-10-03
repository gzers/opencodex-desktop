//! 应用自更新前端 DTO；不传输下载地址、公钥或签名原文。

use serde::{Deserialize, Serialize};

use crate::modules::update::{UpdateChannel, UpdateStatus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatusDto {
    pub channel: UpdateChannel,
    pub current_version: String,
    pub available_version: Option<String>,
    pub last_checked_at: Option<String>,
    pub signature_verified: Option<bool>,
    pub error: Option<String>,
}

impl From<UpdateStatus> for UpdateStatusDto {
    fn from(value: UpdateStatus) -> Self {
        Self {
            channel: value.channel,
            current_version: value.current_version,
            available_version: value.available_version,
            last_checked_at: value.last_checked_at,
            signature_verified: value.signature_verified,
            error: value.error,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckUpdateResultDto {
    pub status: String,
    pub update: UpdateStatusDto,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_status_dto_is_camel_case_and_masks_raw_metadata() {
        let value = UpdateStatusDto {
            channel: UpdateChannel::Stable,
            current_version: "0.1.0".to_string(),
            available_version: Some("0.2.0".to_string()),
            last_checked_at: Some("2026-09-16T00:00:00Z".to_string()),
            signature_verified: Some(true),
            error: None,
        };
        let payload = serde_json::to_value(&value).unwrap();
        assert_eq!(payload["currentVersion"], "0.1.0");
        assert_eq!(payload["availableVersion"], "0.2.0");
        assert_eq!(payload["lastCheckedAt"], "2026-09-16T00:00:00Z");
        assert_eq!(payload["signatureVerified"], true);
    }
}
