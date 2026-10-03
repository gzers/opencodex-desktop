//! 官方升级引导前端 DTO；领域结果与 WebView 契约保持分离。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeBackupResult {
    pub backup_id: String,
    pub directory: String,
    pub target_path: String,
}

/// AC-05 restore 前置风险摘要；只展示配置存在性与冻结运行态事实。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreRiskSummaryDto {
    pub guidance: crate::types::runtime_status::RestoreGuidanceDto,
    pub preferences_configured: bool,
    pub extension_configured: bool,
    pub sync_endpoint_configured: bool,
    pub opencodex_home: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrade_backup_dto_uses_camel_case() {
        let value = UpgradeBackupResult {
            backup_id: "bk_01".to_string(),
            directory: "/tmp/opencodex-data/backups".to_string(),
            target_path: "/tmp/opencodex-data/manager-state/preferences.json".to_string(),
        };
        let payload = serde_json::to_value(&value).expect("serialize upgrade DTO");
        assert_eq!(payload["backupId"], "bk_01");
        assert_eq!(payload["directory"], value.directory);
        assert_eq!(payload["targetPath"], value.target_path);
    }

    #[test]
    fn restore_risk_summary_dto_uses_camel_case() {
        let guidance = crate::types::runtime_status::RestoreGuidanceDto {
            runtime_state: crate::types::status::RuntimeState::AtRisk,
            health: crate::modules::status::HealthState::Degraded,
            startup_status: Some("at-risk".to_string()),
            protection: Some("none".to_string()),
            reboot_safe: Some(false),
            service_present: Some(false),
            shim_present: Some(false),
            version_drift: Some("2.50.0".to_string()),
            data_root: "/tmp/opencodex-data".to_string(),
            opencodex_home: Some("/tmp/opencodex-home".to_string()),
        };
        let value = RestoreRiskSummaryDto {
            guidance,
            preferences_configured: true,
            extension_configured: true,
            sync_endpoint_configured: false,
            opencodex_home: Some("/tmp/opencodex-home".to_string()),
        };
        let payload = serde_json::to_value(&value).expect("serialize restore risk DTO");
        assert_eq!(payload["guidance"]["runtimeState"], "at_risk");
        assert_eq!(payload["guidance"]["health"], "degraded");
        assert_eq!(payload["guidance"]["startupStatus"], "at-risk");
        assert_eq!(payload["guidance"]["rebootSafe"], false);
        assert_eq!(payload["guidance"]["dataRoot"], value.guidance.data_root);
        assert_eq!(payload["preferencesConfigured"], true);
        assert_eq!(payload["extensionConfigured"], true);
        assert_eq!(payload["syncEndpointConfigured"], false);
        assert_eq!(payload["opencodexHome"], "/tmp/opencodex-home");
    }
}
