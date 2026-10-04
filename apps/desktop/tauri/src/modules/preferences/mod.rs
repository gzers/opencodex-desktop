//! MOD-01 桌面壳偏好真实持久化。
//!
//! 读写显式数据根中的 `manager-state/preferences.json`；读取缺失时返回冻结
//! 默认值，不扫描用户目录。损坏文件显式失败，不猜配置；保存使用共享原子
//! 写原语替换目标并保持 `0600`。不访问真实用户配置、Keychain 或 WebDAV。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::errors::AppError;

pub const PREFERENCES_RELATIVE_PATH: &str = "manager-state/preferences.json";
pub const INTERFACE_SCALE_MIN: i32 = 50;
pub const INTERFACE_SCALE_MAX: i32 = 200;

pub const PANEL_MODE_EMBEDDED: &str = "embedded";
pub const PANEL_MODE_BROWSER: &str = "browser";
pub const BACKUP_RETENTION_FIVE: &str = "5";
pub const BACKUP_RETENTION_TEN: &str = "10";
pub const BACKUP_RETENTION_TWENTY: &str = "20";
pub const BACKUP_INTEGRITY_SHA256: &str = "sha-256";
pub const BACKUP_INTEGRITY_SHA512: &str = "sha-512";
pub const BACKUP_INTEGRITY_BLAKE3: &str = "blake3";
pub const MCP_POLICY_ASK: &str = "ask";
pub const MCP_POLICY_KEEP_TARGET: &str = "keep-target";
pub const MCP_POLICY_KEEP_BOTH: &str = "keep-both";
pub const LOG_RETENTION_7D_5000: &str = "7d-5000";
pub const LOG_RETENTION_30D_10000: &str = "30d-10000";
pub const LOG_RETENTION_90D_30000: &str = "90d-30000";
pub const NOTIFICATION_RETENTION_7: &str = "7";
pub const NOTIFICATION_RETENTION_30: &str = "30";
pub const NOTIFICATION_RETENTION_90: &str = "90";
pub const SYNC_POLICY_ASK: &str = "ask";
pub const SYNC_POLICY_KEEP_LOCAL: &str = "keep-local";
pub const SYNC_POLICY_KEEP_REMOTE: &str = "keep-remote";
pub const SYNC_POLICY_KEEP_BOTH: &str = "keep-both";
pub const APP_UPDATE_STABLE_24H: &str = "stable-24h";
pub const APP_UPDATE_BETA_6H: &str = "beta-6h";
pub const APP_UPDATE_MANUAL: &str = "manual";
pub const APP_UPDATE_CHANNEL_STABLE: &str = "stable";
pub const APP_UPDATE_CHANNEL_BETA: &str = "beta";
pub const THEME_LIGHT: &str = "light";
pub const THEME_DARK: &str = "dark";
pub const THEME_SYSTEM: &str = "system";
// 网络代理（U-05）：模式 none/system/manual；协议 http/socks5h；无凭据。
pub const PROXY_MODE_NONE: &str = "none";
pub const PROXY_MODE_SYSTEM: &str = "system";
pub const PROXY_MODE_MANUAL: &str = "manual";
pub const PROXY_SCHEME_HTTP: &str = "http";
pub const PROXY_SCHEME_SOCKS5H: &str = "socks5h";
// 界面特效档位（UI规范 §26.2 / 契约字段 §9）：用户选择；高=默认。
pub const VISUAL_EFFECTS_HIGH: &str = "high";
pub const VISUAL_EFFECTS_MID: &str = "mid";
pub const VISUAL_EFFECTS_LOW: &str = "low";
// 背景光渲染方式（画质卡片）：WEBGL 网格渐变＋颗粒着色器（默认）/ 纯 CSS 极光（兜底）。
pub const GLOW_RENDER_MESH: &str = "mesh";
pub const GLOW_RENDER_CSS: &str = "css";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct Preferences {
    /// 配置 schema 版本（文档级，非应用版本）；缺失按 legacy v0 识别后由迁移引擎补齐。
    pub schema_version: u32,
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
    // 更新通道/自动检查/间隔解耦（U-07）：旧 `app_update_channel` 复合枚举在加载时迁移。
    pub app_update_channel: String,
    pub app_update_auto_check: bool,
    pub app_update_check_interval_seconds: i64,
    // 主题事实源（H-15）：后端为事实源，前端 localStorage 仅作首屏缓存。
    pub theme: String,
    // 网络代理（U-05）：模式 + 手动模式的协议/地址/例外；不含凭据（不用钥匙串）。
    pub network_proxy_mode: String,
    pub network_proxy_scheme: String,
    pub network_proxy_host: String,
    pub network_no_proxy: String,
    // `#[serde(default)]`：旧偏好缺该字段时按默认高档读取，不判损坏。
    pub visual_effects: String,
    // 同理：旧偏好缺该字段时按默认 WEBGL 读取。
    pub glow_render: String,
}

/// 固化默认的解析投影：不带容器级 serde(default)，避免 Default 读取文件时递归。
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
struct FrozenPreferences {
    schema_version: u32,
    interface_scale: i32,
    launch_main: bool,
    auto_panel: bool,
    panel_mode: String,
    keep_proxy_on_close: bool,
    lifecycle_notifications: bool,
    sync_conflict_alerts: bool,
    launch_with_codex: bool,
    auto_backup_upgrade: bool,
    auto_backup_import: bool,
    auto_backup_sync: bool,
    backup_retention: String,
    backup_integrity: String,
    backup_include_skills: bool,
    export_include_skills: bool,
    mcp_conflict_policy: String,
    mcp_mask: bool,
    backup_include_mcp: bool,
    export_include_mcp: bool,
    log_retention: String,
    notification_retention: String,
    startup_cleanup: bool,
    cleanup_backup_summary: bool,
    cli_enabled: bool,
    sync_conflict_policy: String,
    cold_sync: bool,
    backup_before_overwrite: bool,
    app_update_channel: String,
    app_update_auto_check: bool,
    app_update_check_interval_seconds: i64,
    theme: String,
    network_proxy_mode: String,
    network_proxy_scheme: String,
    network_proxy_host: String,
    network_no_proxy: String,
    visual_effects: String,
    glow_render: String,
}

impl Default for Preferences {
    // 版本固化默认（H-01）：从构建期嵌入的 config/preferences.defaults.json 读取，
    // 消费者不再各写 fallback；文件在编译期由 build.rs 校验存在且为合法 JSON。
    fn default() -> Self {
        let raw = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/config/preferences.defaults.json"
        ));
        let frozen: FrozenPreferences =
            serde_json::from_str(raw).expect("frozen preferences defaults must parse");
        Self {
            schema_version: frozen.schema_version,
            interface_scale: frozen.interface_scale,
            launch_main: frozen.launch_main,
            auto_panel: frozen.auto_panel,
            panel_mode: frozen.panel_mode,
            keep_proxy_on_close: frozen.keep_proxy_on_close,
            lifecycle_notifications: frozen.lifecycle_notifications,
            sync_conflict_alerts: frozen.sync_conflict_alerts,
            launch_with_codex: frozen.launch_with_codex,
            auto_backup_upgrade: frozen.auto_backup_upgrade,
            auto_backup_import: frozen.auto_backup_import,
            auto_backup_sync: frozen.auto_backup_sync,
            backup_retention: frozen.backup_retention,
            backup_integrity: frozen.backup_integrity,
            backup_include_skills: frozen.backup_include_skills,
            export_include_skills: frozen.export_include_skills,
            mcp_conflict_policy: frozen.mcp_conflict_policy,
            mcp_mask: frozen.mcp_mask,
            backup_include_mcp: frozen.backup_include_mcp,
            export_include_mcp: frozen.export_include_mcp,
            log_retention: frozen.log_retention,
            notification_retention: frozen.notification_retention,
            startup_cleanup: frozen.startup_cleanup,
            cleanup_backup_summary: frozen.cleanup_backup_summary,
            cli_enabled: frozen.cli_enabled,
            sync_conflict_policy: frozen.sync_conflict_policy,
            cold_sync: frozen.cold_sync,
            backup_before_overwrite: frozen.backup_before_overwrite,
            app_update_channel: frozen.app_update_channel,
            app_update_auto_check: frozen.app_update_auto_check,
            app_update_check_interval_seconds: frozen.app_update_check_interval_seconds,
            theme: frozen.theme,
            network_proxy_mode: frozen.network_proxy_mode,
            network_proxy_scheme: frozen.network_proxy_scheme,
            network_proxy_host: frozen.network_proxy_host,
            network_no_proxy: frozen.network_no_proxy,
            visual_effects: frozen.visual_effects,
            glow_render: frozen.glow_render,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferencesError {
    NotConfigured,
    Corrupted,
    Io,
}

/// 出站代理策略（U-05）：由偏好决定，不含凭据；System 表示沿用系统代理。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyPolicy {
    None,
    System,
    Manual(String),
}

/// 由偏好解析出代理策略；无凭据、非法地址一律视为不指定代理。
pub fn proxy_policy(value: &Preferences) -> ProxyPolicy {
    match value.network_proxy_mode.as_str() {
        PROXY_MODE_SYSTEM => ProxyPolicy::System,
        PROXY_MODE_MANUAL => {
            let host = value.network_proxy_host.trim();
            if host.is_empty() || host.contains('@') || host.chars().any(char::is_whitespace) {
                ProxyPolicy::None
            } else {
                ProxyPolicy::Manual(format!("{}://{}", value.network_proxy_scheme, host))
            }
        }
        _ => ProxyPolicy::None,
    }
}

/// 统一网络环境策略（U-05）：把用户代理偏好落到子进程的 HTTP(S)_PROXY/NO_PROXY，供官方 CLI 与 npm 查询复用。
pub fn network_environment_for_app<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    home: std::path::PathBuf,
) -> crate::modules::process::EnvironmentPolicy {
    use tauri::Manager;
    let mut environment = crate::modules::process::EnvironmentPolicy {
        home: Some(home.into_os_string()),
        ..Default::default()
    };
    if let Some(root) = app.try_state::<crate::state::SharedDataRoot>() {
        if let Ok(value) = PreferencesStore::new(&root.0).load() {
            if let ProxyPolicy::Manual(url) = proxy_policy(&value) {
                environment.http_proxy = Some(url.clone().into());
                environment.https_proxy = Some(url.into());
            }
            let no_proxy = value.network_no_proxy.trim();
            if !no_proxy.is_empty() {
                environment.no_proxy = Some(no_proxy.into());
            }
        }
    }
    environment
}

/// 便捷读取：从数据根读偏好并解析代理策略；读不到按不指定代理。
pub fn proxy_policy_for_app<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> ProxyPolicy {
    use tauri::Manager;
    let Some(root) = app.try_state::<crate::state::SharedDataRoot>() else {
        return ProxyPolicy::None;
    };
    PreferencesStore::new(&root.0)
        .load()
        .map(|value| proxy_policy(&value))
        .unwrap_or(ProxyPolicy::None)
}

pub struct PreferencesStore {
    data_root: PathBuf,
}

impl PreferencesStore {
    pub fn new(data_root: &Path) -> Self {
        Self {
            data_root: data_root.to_path_buf(),
        }
    }

    pub fn path(&self) -> PathBuf {
        self.data_root.join(PREFERENCES_RELATIVE_PATH)
    }

    pub fn load(&self) -> Result<Preferences, PreferencesError> {
        load_preferences(&self.path())
    }

    pub fn save(&self, value: &Preferences) -> Result<Preferences, PreferencesError> {
        save_preferences(&self.path(), value)
    }
}

pub fn load_preferences(path: &Path) -> Result<Preferences, PreferencesError> {
    if !path.is_absolute() {
        return Err(PreferencesError::NotConfigured);
    }
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Preferences::default())
        }
        Err(_) => return Err(PreferencesError::Io),
    };
    let mut value =
        serde_json::from_slice::<Preferences>(&bytes).map_err(|_| PreferencesError::Corrupted)?;
    // 读取即迁移旧复合枚举并校验（U-08/H-22）：非法值不静默放行，也不覆盖原件。
    migrate_legacy_update_fields(&mut value);
    value.schema_version = crate::modules::config_migration::PREFERENCES_CURRENT_SCHEMA;
    validate(&value)?;
    Ok(value)
}

pub fn save_preferences(path: &Path, value: &Preferences) -> Result<Preferences, PreferencesError> {
    validate(value)?;
    if !path.is_absolute() {
        return Err(PreferencesError::NotConfigured);
    }
    let payload = serde_json::to_vec_pretty(value).map_err(|_| PreferencesError::Io)?;
    crate::infrastructure::atomic_write::atomic_write(path, &payload, 0o600)
        .map_err(|_| PreferencesError::Io)?;
    Ok(value.clone())
}

pub fn validate(value: &Preferences) -> Result<(), PreferencesError> {
    if !(INTERFACE_SCALE_MIN..=INTERFACE_SCALE_MAX).contains(&value.interface_scale)
        || !allowed(
            &value.panel_mode,
            &[PANEL_MODE_EMBEDDED, PANEL_MODE_BROWSER],
        )
        || !allowed(
            &value.backup_retention,
            &[
                BACKUP_RETENTION_FIVE,
                BACKUP_RETENTION_TEN,
                BACKUP_RETENTION_TWENTY,
            ],
        )
        || !allowed(
            &value.backup_integrity,
            &[
                BACKUP_INTEGRITY_SHA256,
                BACKUP_INTEGRITY_SHA512,
                BACKUP_INTEGRITY_BLAKE3,
            ],
        )
        || !allowed(
            &value.mcp_conflict_policy,
            &[MCP_POLICY_ASK, MCP_POLICY_KEEP_TARGET, MCP_POLICY_KEEP_BOTH],
        )
        || !allowed(
            &value.log_retention,
            &[
                LOG_RETENTION_7D_5000,
                LOG_RETENTION_30D_10000,
                LOG_RETENTION_90D_30000,
            ],
        )
        || !allowed(
            &value.notification_retention,
            &[
                NOTIFICATION_RETENTION_7,
                NOTIFICATION_RETENTION_30,
                NOTIFICATION_RETENTION_90,
            ],
        )
        || !allowed(
            &value.sync_conflict_policy,
            &[
                SYNC_POLICY_ASK,
                SYNC_POLICY_KEEP_LOCAL,
                SYNC_POLICY_KEEP_REMOTE,
                SYNC_POLICY_KEEP_BOTH,
            ],
        )
        || !allowed(
            &value.app_update_channel,
            &[APP_UPDATE_CHANNEL_STABLE, APP_UPDATE_CHANNEL_BETA],
        )
        || !(0..=30 * 24 * 60 * 60).contains(&value.app_update_check_interval_seconds)
        || !allowed(&value.theme, &[THEME_LIGHT, THEME_DARK, THEME_SYSTEM])
        || !allowed(
            &value.network_proxy_mode,
            &[PROXY_MODE_NONE, PROXY_MODE_SYSTEM, PROXY_MODE_MANUAL],
        )
        || !allowed(
            &value.network_proxy_scheme,
            &[PROXY_SCHEME_HTTP, PROXY_SCHEME_SOCKS5H],
        )
        || !allowed(
            &value.visual_effects,
            &[VISUAL_EFFECTS_HIGH, VISUAL_EFFECTS_MID, VISUAL_EFFECTS_LOW],
        )
        || !allowed(&value.glow_render, &[GLOW_RENDER_MESH, GLOW_RENDER_CSS])
    {
        return Err(PreferencesError::Corrupted);
    }
    Ok(())
}

/// 旧复合枚举 → 通道/自动检查/间隔（U-07）。
///
/// `stable-24h` → stable/true/86400；`beta-6h` → beta/true/21600；
/// `manual` → stable/false/86400。已迁移的新值原样返回。
pub fn migrate_legacy_update_fields(value: &mut Preferences) {
    match value.app_update_channel.as_str() {
        APP_UPDATE_STABLE_24H => {
            value.app_update_channel = APP_UPDATE_CHANNEL_STABLE.to_string();
            value.app_update_auto_check = true;
            value.app_update_check_interval_seconds = 86400;
        }
        APP_UPDATE_BETA_6H => {
            value.app_update_channel = APP_UPDATE_CHANNEL_BETA.to_string();
            value.app_update_auto_check = true;
            value.app_update_check_interval_seconds = 21600;
        }
        APP_UPDATE_MANUAL => {
            value.app_update_channel = APP_UPDATE_CHANNEL_STABLE.to_string();
            value.app_update_auto_check = false;
            value.app_update_check_interval_seconds = 86400;
        }
        _ => {}
    }
}

fn allowed(field: &str, values: &[&str]) -> bool {
    values.contains(&field)
}

pub fn restore_defaults(path: &Path) -> Result<Preferences, PreferencesError> {
    save_preferences(path, &Preferences::default())
}

impl TryFrom<std::collections::BTreeMap<String, crate::modules::container::PreferenceValue>>
    for Preferences
{
    type Error = PreferencesError;

    fn try_from(
        value: std::collections::BTreeMap<String, crate::modules::container::PreferenceValue>,
    ) -> Result<Self, Self::Error> {
        let mut serialized = serde_json::Map::new();
        for (key, value) in value {
            let converted = match value {
                crate::modules::container::PreferenceValue::Boolean(value) => {
                    serde_json::Value::Bool(value)
                }
                crate::modules::container::PreferenceValue::Integer(value) => {
                    serde_json::Value::Number(value.into())
                }
                crate::modules::container::PreferenceValue::Text(value) => {
                    serde_json::Value::String(value)
                }
            };
            serialized.insert(key, converted);
        }
        let mut value =
            serde_json::from_value::<Preferences>(serde_json::Value::Object(serialized))
                .map_err(|_| PreferencesError::Corrupted)?;
        migrate_legacy_update_fields(&mut value);
        validate(&value)?;
        Ok(value)
    }
}

impl PreferencesError {
    pub fn as_app_error(self) -> AppError {
        match self {
            Self::NotConfigured => AppError::NotConfigured,
            Self::Corrupted => AppError::FileSystem {
                operation: "read or validate preferences".to_string(),
                detail: "preferences file is corrupted or contains an unsupported value"
                    .to_string(),
            },
            Self::Io => AppError::FileSystem {
                operation: "read or write preferences".to_string(),
                detail: "preferences file could not be read or written".to_string(),
            },
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn mode(path: &Path) -> u32 {
        std::fs::metadata(path)
            .expect("metadata")
            .permissions()
            .mode()
    }

    #[test]
    fn missing_file_returns_frozen_defaults() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = PreferencesStore::new(temp.path());
        let value = store.load().expect("load missing preferences");
        assert_eq!(value, Preferences::default());
        assert_eq!(value.interface_scale, 100);
        assert_eq!(value.panel_mode, PANEL_MODE_EMBEDDED);
        assert!(!value.cli_enabled);
    }

    #[test]
    fn save_writes_private_file_and_reload_round_trip() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = PreferencesStore::new(temp.path());
        let value = Preferences {
            interface_scale: 150,
            launch_main: false,
            backup_retention: BACKUP_RETENTION_TWENTY.to_string(),
            cli_enabled: true,
            ..Default::default()
        };

        store.save(&value).expect("save preferences");

        assert_eq!(mode(&store.path()) & 0o777, 0o600);
        assert_eq!(store.load().expect("reload"), value);
    }

    #[test]
    fn corrupted_file_fails_without_overwrite() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = PreferencesStore::new(temp.path());
        std::fs::create_dir_all(store.path().parent().expect("parent")).expect("mkdir");
        std::fs::write(store.path(), b"{broken").expect("write fixture");

        let error = store.load().expect_err("corrupted should fail");
        assert_eq!(error, PreferencesError::Corrupted);
    }

    #[test]
    fn rejects_out_of_range_and_unknown_enum_values() {
        let value = Preferences {
            interface_scale: 49,
            ..Default::default()
        };
        assert_eq!(validate(&value), Err(PreferencesError::Corrupted));

        let value = Preferences {
            interface_scale: 201,
            ..Default::default()
        };
        assert_eq!(validate(&value), Err(PreferencesError::Corrupted));

        let value = Preferences {
            panel_mode: "portal".to_string(),
            ..Default::default()
        };
        assert_eq!(validate(&value), Err(PreferencesError::Corrupted));
    }

    #[test]
    fn visual_effects_defaults_to_high_and_rejects_unknown() {
        assert_eq!(Preferences::default().visual_effects, VISUAL_EFFECTS_HIGH);

        for value in [VISUAL_EFFECTS_HIGH, VISUAL_EFFECTS_MID, VISUAL_EFFECTS_LOW] {
            let candidate = Preferences {
                visual_effects: value.to_string(),
                ..Default::default()
            };
            assert_eq!(validate(&candidate), Ok(()));
        }

        let candidate = Preferences {
            visual_effects: "ultra".to_string(),
            ..Default::default()
        };
        assert_eq!(validate(&candidate), Err(PreferencesError::Corrupted));
    }

    // 兼容：旧偏好文件缺 `visual_effects` 字段时按默认高档读取，不判损坏、不阻塞启动。
    #[test]
    fn legacy_preferences_without_visual_effects_load_as_high() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = PreferencesStore::new(temp.path());
        std::fs::create_dir_all(store.path().parent().expect("parent")).expect("mkdir");
        std::fs::write(
            store.path(),
            br#"{"interface_scale":120,"launch_main":true,"auto_panel":true,"panel_mode":"embedded","keep_proxy_on_close":true,"lifecycle_notifications":true,"sync_conflict_alerts":true,"launch_with_codex":true,"auto_backup_upgrade":true,"auto_backup_import":true,"auto_backup_sync":true,"backup_retention":"10","backup_integrity":"sha-256","backup_include_skills":true,"export_include_skills":true,"mcp_conflict_policy":"ask","mcp_mask":true,"backup_include_mcp":true,"export_include_mcp":true,"log_retention":"30d-10000","notification_retention":"30","startup_cleanup":true,"cleanup_backup_summary":true,"cli_enabled":false,"sync_conflict_policy":"ask","cold_sync":true,"backup_before_overwrite":true,"app_update_channel":"stable-24h"}"#,
        )
        .expect("write legacy fixture");

        let loaded = store.load().expect("legacy load");
        assert_eq!(loaded.visual_effects, VISUAL_EFFECTS_HIGH);
        assert_eq!(loaded.glow_render, GLOW_RENDER_MESH);
        assert_eq!(loaded.interface_scale, 120);
    }

    #[test]
    fn glow_render_defaults_to_mesh_and_rejects_unknown() {
        assert_eq!(Preferences::default().glow_render, GLOW_RENDER_MESH);

        for value in [GLOW_RENDER_MESH, GLOW_RENDER_CSS] {
            let candidate = Preferences {
                glow_render: value.to_string(),
                ..Default::default()
            };
            assert_eq!(validate(&candidate), Ok(()));
        }

        let candidate = Preferences {
            glow_render: "canvas2d".to_string(),
            ..Default::default()
        };
        assert_eq!(validate(&candidate), Err(PreferencesError::Corrupted));
    }

    #[test]
    fn restore_defaults_writes_frozen_contract() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = PreferencesStore::new(temp.path());
        let value = Preferences {
            interface_scale: 200,
            cli_enabled: true,
            ..Default::default()
        };
        store.save(&value).expect("save changed preferences");

        let restored = store
            .save(&Preferences::default())
            .expect("restore defaults");

        assert_eq!(restored, Preferences::default());
        assert_eq!(
            store.load().expect("reload restored"),
            Preferences::default()
        );
    }

    #[test]
    fn unconfigured_path_is_rejected() {
        let error = load_preferences(Path::new("")).expect_err("unconfigured load");
        assert_eq!(error, PreferencesError::NotConfigured);
        assert!(save_preferences(Path::new(""), &Preferences::default()).is_err());
    }
}
