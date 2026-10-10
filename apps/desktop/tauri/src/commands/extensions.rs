//! Tauri 扩展命令层。只做显式路径编排与 DTO 投影，不访问平台细节。

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::path::{Path, PathBuf};

use crate::errors::{AppError, AppResult};
use crate::modules::extensions::{discovery, projection, ClientId, ClientTarget, CLIENT_IDS};
use crate::state::{SharedDataRoot, SharedHomeDir};
use crate::types::extensions::{
    ExtensionConfigDto, ExtensionWriteResultDto, ExtensionsDto, McpDetailDto, McpLandingDto,
    SkillDetailDto,
};

#[tauri::command]
pub async fn extension_config(
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, SharedHomeDir>,
) -> AppResult<ExtensionConfigDto> {
    let root = data_root.0.clone();
    let home = home.0.clone();
    crate::commands::run_readonly("read extension config", move || {
        extension_config_with_paths(&root, &home)
    })
    .await
}

#[tauri::command]
pub async fn list_extensions(
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, SharedHomeDir>,
) -> AppResult<ExtensionsDto> {
    let root = data_root.0.clone();
    let home = home.0.clone();
    crate::commands::run_readonly("discover extensions", move || {
        list_extensions_with_paths(&root, &home)
    })
    .await
}

pub fn list_extensions_with_paths(data_root: &Path, home: &Path) -> AppResult<ExtensionsDto> {
    let targets = CLIENT_IDS.map(|client| ClientTarget::user_target(client, home, true, true));
    // FZ-23：源目录来自统一配置（默认为共享目录，可为自定义目录）。发现是只读动作，
    // 自定义目录不可用时回落默认目录而不整体失败——原因由 `sourceDirNotice` 带出。
    let config = projection::load_config_lenient(data_root);
    let (source_dir, notice) = crate::modules::extensions::source_dir::validate_or_default(
        &config, data_root, home, &targets,
    );
    discovery::discover(data_root, home, &source_dir, &targets)
        .map(ExtensionsDto::from)
        .map_err(|error| match error {
            discovery::DiscoveryError::NotConfigured => AppError::NotConfigured,
            discovery::DiscoveryError::FileSystem => AppError::FileSystem {
                operation: "read extensions".to_string(),
                detail: "skills source or client MCP config could not be read".to_string(),
            },
        })
        .map(|mut dto| {
            dto.source_dir = source_dir.display().to_string();
            dto.source_dir_notice = notice.map(|error| error.code().to_string());
            dto.source_dir_custom = config.source_store.is_some();
            dto.sync_method = config.sync_method_normalized().to_string();
            // 「MCP 敏感值脱敏」（默认开启）：args 的值与据此拼出的说明都要掩码，
            // 否则令牌、API Key 等会随发现结果直接进入界面。
            if mcp_mask_enabled(data_root) {
                mask_extensions_dto(&mut dto);
            }
            dto
        })
}

/// 掩码占位符；不携带任何原始字符。
pub const MASK_TOKEN: &str = "••••";

const SENSITIVE_MARKERS: [&str; 10] = [
    "token",
    "secret",
    "password",
    "passwd",
    "api-key",
    "api_key",
    "apikey",
    "authorization",
    "credential",
    "private-key",
];

/// 「MCP 敏感值脱敏」偏好；读不到时按默认（开启）处理，绝不因为读失败而泄露。
fn mcp_mask_enabled(data_root: &Path) -> bool {
    crate::modules::preferences::PreferencesStore::new(data_root)
        .load()
        .map(|preferences| preferences.mcp_mask)
        .unwrap_or(true)
}

fn looks_sensitive_key(key: &str) -> bool {
    let lowered = key.trim_start_matches('-').to_ascii_lowercase();
    SENSITIVE_MARKERS
        .iter()
        .any(|marker| lowered.contains(marker))
}

/// 形如 `--token=value` / `token: value` 时掩码分隔符右侧。
fn masked_key_value(arg: &str) -> Option<String> {
    for separator in ['=', ':'] {
        if let Some(index) = arg.find(separator) {
            let (key, _) = arg.split_at(index);
            if looks_sensitive_key(key) {
                return Some(format!("{key}{separator}{MASK_TOKEN}"));
            }
        }
    }
    None
}

/// 掩码参数列表；返回掩码结果与「原值 → 掩码值」替换对（用于同步掩码说明文本）。
fn mask_sensitive_args(args: &[String]) -> (Vec<String>, Vec<(String, String)>) {
    let mut masked_args = Vec::with_capacity(args.len());
    let mut replacements: Vec<(String, String)> = Vec::new();
    let mut mask_next = false;
    for arg in args {
        if mask_next {
            masked_args.push(MASK_TOKEN.to_string());
            replacements.push((arg.clone(), MASK_TOKEN.to_string()));
            mask_next = false;
            continue;
        }
        if let Some(masked) = masked_key_value(arg) {
            replacements.push((arg.clone(), masked.clone()));
            masked_args.push(masked);
            continue;
        }
        if looks_sensitive_key(arg) {
            // 形如 `--token` 的旗帜：掩码紧随其后的值，仍保留旗帜本身。
            mask_next = true;
        }
        masked_args.push(arg.clone());
    }
    (masked_args, replacements)
}

pub fn mask_extensions_dto(dto: &mut ExtensionsDto) {
    for server in &mut dto.servers {
        let (masked_args, replacements) = mask_sensitive_args(&server.args);
        if replacements.is_empty() {
            continue;
        }
        let mut description = server.description.clone();
        for (original, masked) in &replacements {
            if !original.is_empty() {
                description = description.replace(original.as_str(), masked.as_str());
            }
        }
        server.description = description;
        server.args = masked_args;
    }
}

/// 键名是否指向参数数组（需要按参数掩码规则处理）。
fn is_args_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "args" | "arguments" | "commandargs" | "command_args"
    )
}

/// 键名是否指向环境变量集合（值一律掩码，只保留键名）。
fn is_env_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "env" | "environment" | "envs"
    )
}

enum MaskMode {
    /// 参数数组：按 `--token=xxx` / `--token xxx` 规则掩码。
    Args,
    /// 环境变量值：一律掩码。
    EnvValues,
    /// 其它：只对敏感键名的字符串值掩码。
    Default,
}

/// 对 MCP 定义 JSON 做展示用掩码；与发现结果使用同一套掩码口径。
fn mask_json_value(value: &mut serde_json::Value, mode: MaskMode) {
    use serde_json::Value;
    match value {
        Value::Array(items) => {
            let strings: Vec<String> = items
                .iter()
                .filter_map(|item| item.as_str().map(|text| text.to_string()))
                .collect();
            if strings.len() == items.len() && !strings.is_empty() {
                match mode {
                    MaskMode::Args => {
                        let (masked, _) = mask_sensitive_args(&strings);
                        for (slot, text) in items.iter_mut().zip(masked) {
                            *slot = Value::String(text);
                        }
                    }
                    MaskMode::EnvValues => {
                        for slot in items.iter_mut() {
                            *slot = Value::String(MASK_TOKEN.to_string());
                        }
                    }
                    MaskMode::Default => {
                        for item in items.iter_mut() {
                            mask_json_value(item, MaskMode::Default);
                        }
                    }
                }
                return;
            }
            for item in items.iter_mut() {
                mask_json_value(item, MaskMode::Default);
            }
        }
        Value::Object(map) => {
            for (key, child) in map.iter_mut() {
                if is_args_key(key) {
                    mask_json_value(child, MaskMode::Args);
                } else if is_env_key(key) {
                    mask_json_value(child, MaskMode::EnvValues);
                } else if looks_sensitive_key(key) && child.is_string() {
                    let empty = child.as_str().map(|text| text.is_empty()).unwrap_or(false);
                    if !empty {
                        *child = Value::String(MASK_TOKEN.to_string());
                    }
                } else {
                    mask_json_value(child, MaskMode::Default);
                }
            }
        }
        Value::String(text) if matches!(mode, MaskMode::EnvValues) && !text.is_empty() => {
            *text = MASK_TOKEN.to_string();
        }
        _ => {}
    }
}

/// AC-11：单个 Skill 的只读详情。按需读取一条 `SKILL.md` 与目录统计，不预读列表正文。
pub fn read_skill_detail_with_paths(
    name: &str,
    data_root: &Path,
    home: &Path,
) -> AppResult<SkillDetailDto> {
    let targets = CLIENT_IDS.map(|client| ClientTarget::user_target(client, home, true, true));
    let config = projection::load_config_lenient(data_root);
    let (source_dir, _notice) = crate::modules::extensions::source_dir::validate_or_default(
        &config, data_root, home, &targets,
    );
    let store_dir = discovery::skills_store_dir(data_root);
    discovery::read_skill_detail(&source_dir, &store_dir, name)
        .map_err(|_| AppError::FileSystem {
            operation: "read skill detail".to_string(),
            detail: "skills source could not be read".to_string(),
        })?
        .map(SkillDetailDto::from)
        .ok_or_else(|| AppError::NotFound {
            entity: format!("skill {name}"),
        })
}

#[tauri::command]
pub async fn read_skill_detail(
    name: String,
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, SharedHomeDir>,
) -> AppResult<SkillDetailDto> {
    let root = data_root.0.clone();
    let home = home.0.clone();
    crate::commands::run_readonly("read skill detail", move || {
        read_skill_detail_with_paths(&name, &root, &home)
    })
    .await
}

/// AC-11：MCP 条目的只读详情。命令 / 参数 / 落点回读；`env` 只给键名，`args` 掩码。
pub fn read_mcp_detail_with_paths(
    name: &str,
    data_root: &Path,
    home: &Path,
) -> AppResult<McpDetailDto> {
    let dto = list_extensions_with_paths(data_root, home)?;
    let server = dto
        .servers
        .into_iter()
        .find(|server| server.name == name)
        .ok_or_else(|| AppError::NotFound {
            entity: format!("mcp server {name}"),
        })?;

    let targets = CLIENT_IDS.map(|client| ClientTarget::user_target(client, home, true, true));
    let mut landings = Vec::new();
    let mut config_json = None;
    for target in &targets {
        let definition =
            discovery::read_server_definition(target, name).map_err(|_| AppError::FileSystem {
                operation: "read mcp detail".to_string(),
                detail: "client mcp config could not be read".to_string(),
            })?;
        if let Some(definition) = definition {
            if config_json.is_none() {
                let mut value = definition.value;
                mask_json_value(&mut value, MaskMode::Default);
                config_json = serde_json::to_string_pretty(&value).ok();
            }
            landings.push(McpLandingDto {
                client: target.client_id,
                config_path: target.mcp_config_path.display().to_string(),
                key: target.mcp_key.clone(),
                present: true,
            });
        } else {
            landings.push(McpLandingDto {
                client: target.client_id,
                config_path: target.mcp_config_path.display().to_string(),
                key: target.mcp_key.clone(),
                present: false,
            });
        }
    }

    Ok(McpDetailDto {
        name: server.name,
        transport: server.transport,
        command: server.command,
        args: server.args,
        env_keys: server.env_keys,
        description: server.description,
        clients: server.clients,
        updated_at: server.updated_at,
        config_json,
        landings,
    })
}

#[tauri::command]
pub async fn read_mcp_detail(
    name: String,
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, SharedHomeDir>,
) -> AppResult<McpDetailDto> {
    let root = data_root.0.clone();
    let home = home.0.clone();
    crate::commands::run_readonly("read mcp detail", move || {
        read_mcp_detail_with_paths(&name, &root, &home)
    })
    .await
}

#[allow(dead_code)]
fn ensure_explicit_paths(data_root: &Path, home: &Path) -> Result<(PathBuf, PathBuf), AppError> {
    if data_root.as_os_str().is_empty() || home.as_os_str().is_empty() {
        return Err(AppError::NotConfigured);
    }
    Ok((data_root.to_path_buf(), home.to_path_buf()))
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionClientToggleRequest {
    pub client: ClientId,
    pub enabled: bool,
}

#[tauri::command]
pub fn set_extension_client_enabled(
    request: ExtensionClientToggleRequest,
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, SharedHomeDir>,
) -> AppResult<ExtensionConfigDto> {
    let _admission = crate::infrastructure::storage_writers::global().admit()?;
    set_extension_client_enabled_with_paths(&data_root.0, &home.0, request.client, request.enabled)
}

// `kind` 用小写蛇形（前端命令名）；变体字段用 camelCase（前端 DTO 口径）。
// `rename_all` 只重命名变体名，不重命名变体字段，必须另加 `rename_all_fields`，
// 否则多词字段（`archive_name` / `archive_payload`）会与前端 `archiveName` 对不上，
// 反序列化阶段就失败（真机 ZIP 导入曾因此显示「未归类的失败」）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum ExtensionWriteCommand {
    ToggleClient {
        client: ClientId,
        enabled: bool,
    },
    LinkSkill {
        name: String,
        client: ClientId,
    },
    UnlinkSkill {
        name: String,
        client: ClientId,
        confirm: bool,
    },
    UninstallSkill {
        name: String,
        confirm: bool,
    },
    WriteMcp {
        name: String,
        client: ClientId,
        confirm: bool,
    },
    RemoveMcp {
        name: String,
        client: Option<ClientId>,
        confirm: bool,
    },
    ImportSkillArchive {
        archive_name: String,
        archive_payload: String,
    },
    RestoreSkill {
        name: String,
    },
    AddMcp {
        definition: projection::RawServerDefinition,
    },
    EditMcp {
        name: String,
        definition: projection::RawServerDefinition,
        confirm: bool,
    },
    /// FZ-23：设置源目录。`path` 为空或 `null` = 回到默认源目录。
    SetSourceDir {
        path: Option<String>,
    },
    /// FZ-24：设置分发方式（`symlink` / `copy`）。
    SetSyncMethod {
        method: String,
    },
    /// 按当前源目录 + 分发方式对齐已落地的客户端目标（不改配置）。
    ResyncSkills,
}

#[tauri::command]
pub async fn execute_extension_write(
    command: ExtensionWriteCommand,
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, SharedHomeDir>,
) -> AppResult<ExtensionWriteResultDto> {
    let root = data_root.0.clone();
    let home = home.0.clone();
    crate::commands::run_blocking("execute extension write", move || {
        execute_extension_write_with_paths(&root, &home, command)
    })
    .await
}

pub fn execute_extension_write_with_paths(
    data_root: &Path,
    home: &Path,
    command: ExtensionWriteCommand,
) -> AppResult<ExtensionWriteResultDto> {
    let targets = client_targets(home)?;
    let command = match command {
        ExtensionWriteCommand::ToggleClient { client, enabled } => {
            projection::ProjectionCommand::ToggleClient { client, enabled }
        }
        ExtensionWriteCommand::LinkSkill { name, client } => {
            projection::ProjectionCommand::LinkSkill { name, client }
        }
        ExtensionWriteCommand::UnlinkSkill {
            name,
            client,
            confirm,
        } => projection::ProjectionCommand::UnlinkSkill {
            name,
            client,
            confirm,
        },
        ExtensionWriteCommand::UninstallSkill { name, confirm } => {
            projection::ProjectionCommand::UninstallSkill { name, confirm }
        }
        ExtensionWriteCommand::WriteMcp {
            name,
            client,
            confirm,
        } => projection::ProjectionCommand::WriteMcp {
            name,
            client,
            confirm,
        },
        ExtensionWriteCommand::RemoveMcp {
            name,
            client,
            confirm,
        } => projection::ProjectionCommand::RemoveMcp {
            name,
            client,
            confirm,
        },
        ExtensionWriteCommand::ImportSkillArchive {
            archive_name,
            archive_payload,
        } => {
            let archive_payload =
                BASE64
                    .decode(archive_payload.trim().as_bytes())
                    .map_err(|_| AppError::FileSystem {
                        operation: "decode skill archive".to_string(),
                        detail: "archive payload is not valid base64".to_string(),
                    })?;
            projection::ProjectionCommand::ImportSkillArchive {
                archive_name,
                archive_payload,
            }
        }
        ExtensionWriteCommand::RestoreSkill { name } => {
            projection::ProjectionCommand::RestoreSkill { name }
        }
        ExtensionWriteCommand::AddMcp { definition } => {
            projection::ProjectionCommand::AddMcp { definition }
        }
        ExtensionWriteCommand::EditMcp {
            name,
            definition,
            confirm,
        } => projection::ProjectionCommand::EditMcp {
            name,
            definition,
            confirm,
        },
        ExtensionWriteCommand::SetSourceDir { path } => {
            projection::ProjectionCommand::SetSourceDir { path }
        }
        ExtensionWriteCommand::SetSyncMethod { method } => {
            projection::ProjectionCommand::SetSyncMethod { method }
        }
        ExtensionWriteCommand::ResyncSkills => projection::ProjectionCommand::ResyncSkills,
    };
    let result = projection::execute(data_root, &targets, command).map_err(projection_error)?;
    Ok(ExtensionWriteResultDto::from_projection(result, None))
}

pub fn set_extension_client_enabled_with_paths(
    data_root: &Path,
    home: &Path,
    client: ClientId,
    enabled: bool,
) -> AppResult<ExtensionConfigDto> {
    let targets = client_targets(home)?;
    let result = projection::set_client_enabled(data_root, &targets, client, enabled)
        .map_err(projection_error)?;
    Ok(ExtensionConfigDto::from_projection(result, None))
}

pub fn extension_config_with_paths(data_root: &Path, home: &Path) -> AppResult<ExtensionConfigDto> {
    let targets = client_targets(home)?;
    let (config, fingerprint, conflict) =
        projection::load(data_root, &targets).map_err(projection_error)?;
    let projection_result = projection::ProjectionWriteResult {
        config,
        fingerprint,
        document_sha256: read_projection_hash(data_root)?,
        backed_up: false,
        skills_linked: Vec::new(),
        mcp_written: Vec::new(),
    };
    Ok(ExtensionConfigDto::from_projection(
        projection_result,
        conflict.as_ref().map(|value| value.kind),
    ))
}

fn client_targets(home: &Path) -> Result<Vec<ClientTarget>, AppError> {
    let home = home.canonicalize().map_err(|error| AppError::FileSystem {
        operation: "resolve extension home".to_string(),
        detail: error.to_string(),
    })?;
    Ok(CLIENT_IDS
        .map(|client| ClientTarget::user_target(client, &home, true, true))
        .to_vec())
}

fn read_projection_hash(data_root: &Path) -> Result<String, AppError> {
    crate::infrastructure::hash::sha256_file(&projection::config_path(data_root)).map_err(|error| {
        AppError::FileSystem {
            operation: "read extension projection".to_string(),
            detail: error.to_string(),
        }
    })
}

pub fn projection_error(error: projection::ProjectionError) -> AppError {
    match error {
        projection::ProjectionError::NotConfigured => AppError::NotConfigured,
        projection::ProjectionError::TooLarge => AppError::FileSystem {
            operation: "validate extension projection".to_string(),
            detail: "projection exceeds frozen limit".to_string(),
        },
        projection::ProjectionError::Corrupted => AppError::FileSystem {
            operation: "parse extension projection".to_string(),
            detail: "projection is invalid or corrupted".to_string(),
        },
        projection::ProjectionError::ExternalModified => AppError::FileSystem {
            operation: "inspect extension projection".to_string(),
            detail: "external target changed; conflict is pending".to_string(),
        },
        projection::ProjectionError::HalfWritten => AppError::FileSystem {
            operation: "inspect extension projection".to_string(),
            detail: "projection marker is missing or half written".to_string(),
        },
        projection::ProjectionError::LockFailed => AppError::TargetLockTimeout { timeout_ms: 3000 },
        projection::ProjectionError::TargetInvalid => AppError::FileSystem {
            operation: "validate extension target".to_string(),
            detail: "client target is not registered or not installed".to_string(),
        },
        projection::ProjectionError::TargetType => AppError::FileSystem {
            operation: "validate extension target".to_string(),
            detail: "target context is not a regular directory".to_string(),
        },
        projection::ProjectionError::PathEscape => AppError::FileSystem {
            operation: "validate extension target".to_string(),
            detail: "target context escapes its explicit root".to_string(),
        },
        projection::ProjectionError::Backup => AppError::FileSystem {
            operation: "backup extension projection".to_string(),
            detail: "backup creation failed".to_string(),
        },
        projection::ProjectionError::AtomicWrite => AppError::FileSystem {
            operation: "replace extension projection".to_string(),
            detail: "atomic write failed".to_string(),
        },
        projection::ProjectionError::Conflict => AppError::FileSystem {
            operation: "confirm extension projection".to_string(),
            detail: "extension conflict requires confirmation".to_string(),
        },
        projection::ProjectionError::UnsupportedTarget => AppError::FileSystem {
            operation: "validate extension target".to_string(),
            detail: "target file or container is not supported".to_string(),
        },
        projection::ProjectionError::InvalidArchive => AppError::FileSystem {
            operation: "validate extension archive".to_string(),
            detail: "archive is invalid or unsafe".to_string(),
        },
        projection::ProjectionError::NotFound => AppError::NotFound {
            entity: "skill backup".to_string(),
        },
        // FZ-23 / FZ-24：源目录与分发方式由用户决定，取值非法时显式失败并保留原值。
        projection::ProjectionError::SourceDirInvalid => AppError::FileSystem {
            operation: "validate skills source directory".to_string(),
            detail: "skills source directory did not pass validation".to_string(),
        },
        projection::ProjectionError::SyncMethodInvalid => AppError::FileSystem {
            operation: "validate skills sync method".to_string(),
            detail: "skills sync method is not one of the frozen values".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::extensions::ExtensionServerDto;

    fn server(args: &[&str], description: &str) -> ExtensionServerDto {
        ExtensionServerDto {
            id: "srv".to_string(),
            name: "srv".to_string(),
            transport: "stdio".to_string(),
            command: Some("node".to_string()),
            args: args.iter().map(|value| value.to_string()).collect(),
            env_keys: vec!["TOKEN".to_string()],
            description: description.to_string(),
            clients: Vec::new(),
            updated_at: None,
        }
    }

    fn dto(server: ExtensionServerDto) -> ExtensionsDto {
        ExtensionsDto {
            skills: Vec::new(),
            servers: vec![server],
            source_dir: "/tmp/skills".to_string(),
            source_dir_notice: None,
            source_dir_custom: false,
            sync_method: "symlink".to_string(),
        }
    }

    // 回归：「MCP 敏感值脱敏」此前没有任何消费方，令牌会随 args 与说明直接进界面。
    #[test]
    fn masks_key_value_args_and_description() {
        let mut value = dto(server(
            &["--token=abc123", "--stdio"],
            "node --token=abc123 --stdio",
        ));
        mask_extensions_dto(&mut value);
        assert_eq!(value.servers[0].args, vec!["--token=••••", "--stdio"]);
        assert!(!value.servers[0].description.contains("abc123"));
        assert!(value.servers[0].description.contains("--token=••••"));
    }

    #[test]
    fn masks_value_following_sensitive_flag() {
        let mut value = dto(server(
            &["--api-key", "topsecret", "--stdio"],
            "node --api-key topsecret --stdio",
        ));
        mask_extensions_dto(&mut value);
        assert_eq!(value.servers[0].args, vec!["--api-key", "••••", "--stdio"]);
        assert!(!value.servers[0].description.contains("topsecret"));
    }

    #[test]
    fn leaves_ordinary_args_untouched() {
        let mut value = dto(server(&["--foo", "bar"], "node --foo bar"));
        mask_extensions_dto(&mut value);
        assert_eq!(value.servers[0].args, vec!["--foo", "bar"]);
        assert_eq!(value.servers[0].description, "node --foo bar");
    }
}
