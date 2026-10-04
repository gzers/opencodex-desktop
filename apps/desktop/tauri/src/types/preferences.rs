//! 桌面壳偏好前端 DTO；领域持久化与 WebView 契约保持分离。

use serde::{Deserialize, Serialize};

use crate::modules::preferences::Preferences;

fn default_preferences_schema() -> u32 {
    crate::modules::config_migration::PREFERENCES_CURRENT_SCHEMA
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreferencesDto {
    #[serde(default = "default_preferences_schema")]
    pub schema_version: u32,
    /// 一次性主题导入信号（H-15/§7）：不含 theme 字段的旧偏好为 true，前端据此把历史
    /// localStorage 主题提交一次；不是持久化字段，保存后即为 false。
    #[serde(default)]
    pub theme_needs_import: bool,
    pub interface_scale: i32,
    pub launch_main: bool,
    pub auto_panel: bool,
    pub panel_mode: String,
    pub keep_proxy_on_close: bool,
    pub lifecycle_notifications: bool,
    pub sync_conflict_alerts: bool,
    pub launch_with_codex: bool,
    pub auto_backup_upgrade: bool,
    pub auto_backup_import: bool,
    pub auto_backup_sync: bool,
    pub backup_retention: String,
    pub backup_integrity: String,
    pub backup_include_skills: bool,
    pub export_include_skills: bool,
    pub mcp_conflict_policy: String,
    pub mcp_mask: bool,
    pub backup_include_mcp: bool,
    pub export_include_mcp: bool,
    pub log_retention: String,
    pub notification_retention: String,
    pub startup_cleanup: bool,
    pub cleanup_backup_summary: bool,
    pub cli_enabled: bool,
    pub sync_conflict_policy: String,
    pub cold_sync: bool,
    pub backup_before_overwrite: bool,
    pub app_update_channel: String,
    pub app_update_auto_check: bool,
    pub app_update_check_interval_seconds: i64,
    pub theme: String,
    pub visual_effects: String,
    pub glow_render: String,
}

impl From<Preferences> for PreferencesDto {
    fn from(value: Preferences) -> Self {
        Self {
            schema_version: value.schema_version,
            theme_needs_import: false,
            interface_scale: value.interface_scale,
            launch_main: value.launch_main,
            auto_panel: value.auto_panel,
            panel_mode: value.panel_mode,
            keep_proxy_on_close: value.keep_proxy_on_close,
            lifecycle_notifications: value.lifecycle_notifications,
            sync_conflict_alerts: value.sync_conflict_alerts,
            launch_with_codex: value.launch_with_codex,
            auto_backup_upgrade: value.auto_backup_upgrade,
            auto_backup_import: value.auto_backup_import,
            auto_backup_sync: value.auto_backup_sync,
            backup_retention: value.backup_retention,
            backup_integrity: value.backup_integrity,
            backup_include_skills: value.backup_include_skills,
            export_include_skills: value.export_include_skills,
            mcp_conflict_policy: value.mcp_conflict_policy,
            mcp_mask: value.mcp_mask,
            backup_include_mcp: value.backup_include_mcp,
            export_include_mcp: value.export_include_mcp,
            log_retention: value.log_retention,
            notification_retention: value.notification_retention,
            startup_cleanup: value.startup_cleanup,
            cleanup_backup_summary: value.cleanup_backup_summary,
            cli_enabled: value.cli_enabled,
            sync_conflict_policy: value.sync_conflict_policy,
            cold_sync: value.cold_sync,
            backup_before_overwrite: value.backup_before_overwrite,
            app_update_channel: value.app_update_channel,
            app_update_auto_check: value.app_update_auto_check,
            app_update_check_interval_seconds: value.app_update_check_interval_seconds,
            theme: value.theme,
            visual_effects: value.visual_effects,
            glow_render: value.glow_render,
        }
    }
}

// 命令边界（WebView → Rust）收到的永远是 camelCase 的 `PreferencesDto`；领域结构
// `Preferences` 使用 snake_case 落盘。这里提供显式转换，避免把两者混用：此前
// `save_preferences` 直接接收 `Preferences`，WebView 的 camelCase 字段被当作未知字段
// 丢弃，`#[serde(default)]` 补齐默认值，于是每次保存都写成默认值、界面随即回滚。
impl From<PreferencesDto> for Preferences {
    fn from(value: PreferencesDto) -> Self {
        Self {
            schema_version: value.schema_version,
            interface_scale: value.interface_scale,
            launch_main: value.launch_main,
            auto_panel: value.auto_panel,
            panel_mode: value.panel_mode,
            keep_proxy_on_close: value.keep_proxy_on_close,
            lifecycle_notifications: value.lifecycle_notifications,
            sync_conflict_alerts: value.sync_conflict_alerts,
            launch_with_codex: value.launch_with_codex,
            auto_backup_upgrade: value.auto_backup_upgrade,
            auto_backup_import: value.auto_backup_import,
            auto_backup_sync: value.auto_backup_sync,
            backup_retention: value.backup_retention,
            backup_integrity: value.backup_integrity,
            backup_include_skills: value.backup_include_skills,
            export_include_skills: value.export_include_skills,
            mcp_conflict_policy: value.mcp_conflict_policy,
            mcp_mask: value.mcp_mask,
            backup_include_mcp: value.backup_include_mcp,
            export_include_mcp: value.export_include_mcp,
            log_retention: value.log_retention,
            notification_retention: value.notification_retention,
            startup_cleanup: value.startup_cleanup,
            cleanup_backup_summary: value.cleanup_backup_summary,
            cli_enabled: value.cli_enabled,
            sync_conflict_policy: value.sync_conflict_policy,
            cold_sync: value.cold_sync,
            backup_before_overwrite: value.backup_before_overwrite,
            app_update_channel: value.app_update_channel,
            app_update_auto_check: value.app_update_auto_check,
            app_update_check_interval_seconds: value.app_update_check_interval_seconds,
            theme: value.theme,
            visual_effects: value.visual_effects,
            glow_render: value.glow_render,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferences_dto_uses_camel_case() {
        let value = PreferencesDto {
            schema_version: 1,
            theme_needs_import: false,
            interface_scale: 110,
            launch_main: false,
            auto_panel: true,
            panel_mode: "browser".into(),
            keep_proxy_on_close: true,
            lifecycle_notifications: true,
            sync_conflict_alerts: true,
            launch_with_codex: true,
            auto_backup_upgrade: true,
            auto_backup_import: true,
            auto_backup_sync: true,
            backup_retention: "10".into(),
            backup_integrity: "sha-256".into(),
            backup_include_skills: true,
            export_include_skills: true,
            mcp_conflict_policy: "ask".into(),
            mcp_mask: true,
            backup_include_mcp: true,
            export_include_mcp: true,
            log_retention: "30d-10000".into(),
            notification_retention: "30".into(),
            startup_cleanup: true,
            cleanup_backup_summary: true,
            cli_enabled: false,
            sync_conflict_policy: "ask".into(),
            cold_sync: true,
            backup_before_overwrite: true,
            app_update_channel: "stable".into(),
            app_update_auto_check: true,
            app_update_check_interval_seconds: 86400,
            theme: "system".into(),
            visual_effects: "high".into(),
            glow_render: "mesh".into(),
        };
        let payload = serde_json::to_value(&value).expect("serialize DTO");
        assert_eq!(payload["interfaceScale"], 110);
        assert_eq!(payload["schemaVersion"], 1);
        assert_eq!(payload["themeNeedsImport"], false);
        assert_eq!(payload["launchMain"], false);
        assert_eq!(payload["backupRetention"], "10");
        assert_eq!(payload["appUpdateChannel"], "stable");
        assert_eq!(payload["appUpdateAutoCheck"], true);
        assert_eq!(payload["theme"], "system");
        assert_eq!(payload["visualEffects"], "high");
        assert_eq!(payload["glowRender"], "mesh");
    }

    // 回归：命令边界收到的是 camelCase 载荷，转成领域结构后必须保留取值，
    // 不能再被 `#[serde(default)]` 静默改写为默认值。
    #[test]
    fn webview_camel_case_payload_survives_command_boundary() {
        let dto: PreferencesDto = serde_json::from_value(serde_json::json!({
            "interfaceScale": 150,
            "launchMain": false,
            "autoPanel": false,
            "panelMode": "browser",
            "keepProxyOnClose": false,
            "lifecycleNotifications": false,
            "syncConflictAlerts": false,
            "launchWithCodex": false,
            "autoBackupUpgrade": false,
            "autoBackupImport": false,
            "autoBackupSync": false,
            "backupRetention": "20",
            "backupIntegrity": "blake3",
            "backupIncludeSkills": false,
            "exportIncludeSkills": false,
            "mcpConflictPolicy": "keep-both",
            "mcpMask": false,
            "backupIncludeMcp": false,
            "exportIncludeMcp": false,
            "logRetention": "7d-5000",
            "notificationRetention": "7",
            "startupCleanup": false,
            "cleanupBackupSummary": false,
            "cliEnabled": true,
            "syncConflictPolicy": "keep-remote",
            "coldSync": false,
            "backupBeforeOverwrite": false,
            "appUpdateChannel": "beta",
            "appUpdateAutoCheck": false,
            "appUpdateCheckIntervalSeconds": 21600,
            "theme": "dark",
            "visualEffects": "low",
            "glowRender": "css"
        }))
        .expect("deserialize camelCase DTO");
        let domain: Preferences = dto.into();
        assert_eq!(domain.interface_scale, 150);
        assert!(!domain.launch_main);
        assert_eq!(domain.panel_mode, "browser");
        assert_eq!(domain.backup_retention, "20");
        assert_eq!(domain.backup_integrity, "blake3");
        assert!(domain.cli_enabled);
        assert_eq!(domain.app_update_channel, "beta");
        assert!(!domain.app_update_auto_check);
        assert_eq!(domain.app_update_check_interval_seconds, 21600);
        assert_eq!(domain.theme, "dark");
        assert_eq!(domain.visual_effects, "low");
        assert_eq!(domain.glow_render, "css");
    }
}
