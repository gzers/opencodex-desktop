//! MOD-09 扩展统一配置投影与冲突检测。
//!
//! 路径由显式数据根派生；本模块只负责读取、变更与校验投影文件。
//! 不读取真实用户家目录、不访问 Keychain 或网络。

use crate::infrastructure::hash::sha256_file;
use crate::modules::skills::collect_files;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::infrastructure::locking::TargetFileLock;
use crate::modules::backup::{backup_file, BackupAction};
use crate::modules::extensions::discovery::read_server_definition;
pub use crate::modules::extensions::discovery::RawServerDefinition;
use crate::modules::extensions::{
    ClientId, ClientTarget, ConfigFormat, ContainerShape, ExtensionConfig, ProjectionFingerprint,
    CLIENT_IDS,
};

pub const CONFIG_RELATIVE_PATH: &str = "manager-state/extension-config.json";
pub const CONFIG_MAX_BYTES: u64 = 16 * 1024 * 1024;
pub const ARCHIVE_MAX_BYTES: u64 = 128 * 1024 * 1024;
const LOCAL_HOST_PREFIXES: [&str; 4] = ["localhost", "127.", "10.", "192.168."];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionError {
    NotConfigured,
    TooLarge,
    Corrupted,
    ExternalModified,
    HalfWritten,
    LockFailed,
    TargetInvalid,
    TargetType,
    PathEscape,
    Backup,
    AtomicWrite,
    Conflict,
    UnsupportedTarget,
    InvalidArchive,
    NotFound,
    /// FZ-23：自定义源目录未通过校验（不存在 / 不可读 / 符号链接 / 与受管域重叠）。
    SourceDirInvalid,
    /// FZ-24：分发方式取值非法。
    SyncMethodInvalid,
}

impl ProjectionError {
    pub fn as_app_error(self) -> crate::errors::AppError {
        use crate::errors::AppError;
        let (operation, detail) = match self {
            Self::NotConfigured => (
                "validate extension projection",
                "data root is not configured",
            ),
            Self::TooLarge => (
                "validate extension projection",
                "projection exceeds frozen limit",
            ),
            Self::Corrupted => (
                "parse extension projection",
                "projection is invalid or corrupted",
            ),
            Self::ExternalModified => (
                "inspect extension projection",
                "external target changed; conflict is pending",
            ),
            Self::HalfWritten => (
                "inspect extension projection",
                "projection marker is missing or half written",
            ),
            Self::LockFailed => ("lock extension projection", "projection lock failed"),
            Self::TargetInvalid => (
                "validate extension target",
                "client target is not registered",
            ),
            Self::TargetType => (
                "validate extension target",
                "target context is not a regular directory",
            ),
            Self::PathEscape => (
                "validate extension target",
                "target context escapes its explicit root",
            ),
            Self::Backup => ("backup extension projection", "backup creation failed"),
            Self::AtomicWrite => ("replace extension projection", "atomic write failed"),
            Self::Conflict => (
                "confirm extension projection",
                "existing extension content conflicts and requires confirmation",
            ),
            Self::UnsupportedTarget => (
                "validate extension target",
                "target file or container is not supported",
            ),
            Self::InvalidArchive => ("validate extension archive", "archive is invalid or unsafe"),
            Self::NotFound => ("find skill backup", "no restorable backup was found"),
            Self::SourceDirInvalid => (
                "validate skills source directory",
                "skills source directory did not pass validation",
            ),
            Self::SyncMethodInvalid => (
                "validate skills sync method",
                "skills sync method is not one of the frozen values",
            ),
        };
        AppError::FileSystem {
            operation: operation.to_string(),
            detail: detail.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionConflict {
    pub kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionWriteResult {
    pub config: ExtensionConfig,
    pub fingerprint: ProjectionFingerprint,
    pub document_sha256: String,
    pub backed_up: bool,
    pub skills_linked: Vec<String>,
    pub mcp_written: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProjectionCommand {
    ToggleClient {
        client: ClientId,
        enabled: bool,
    },
    /// 建立**某一个客户端**的链接（源目录只读，这里只描述分发意图）。
    LinkSkill {
        name: String,
        client: ClientId,
    },
    /// 断开**某一个客户端**的链接：只删该客户端下的链接，源码与其它客户端不动。
    UnlinkSkill {
        name: String,
        client: ClientId,
        confirm: bool,
    },
    UninstallSkill {
        name: String,
        confirm: bool,
    },
    /// 把某个 MCP 定义写进**某一个客户端**的配置（行内图标 = 该客户端的连/断）。
    WriteMcp {
        name: String,
        client: ClientId,
        confirm: bool,
    },
    /// 从客户端配置里移除 MCP 定义：`Some(client)` = 只断开该客户端；
    /// `None` = 删除定义（所有启用客户端）。
    RemoveMcp {
        name: String,
        client: Option<ClientId>,
        confirm: bool,
    },
    ImportSkillArchive {
        archive_name: String,
        archive_payload: Vec<u8>,
    },
    RestoreSkill {
        name: String,
    },
    AddMcp {
        definition: RawServerDefinition,
    },
    EditMcp {
        name: String,
        definition: RawServerDefinition,
        confirm: bool,
    },
    /// FZ-23：设置源目录。`None` = 回到默认源目录；`Some` = 自定义目录（先校验）。
    SetSourceDir {
        path: Option<String>,
    },
    /// FZ-24：设置分发方式（`symlink` / `copy`）。
    SetSyncMethod {
        method: String,
    },
    /// 按当前源目录 + 分发方式把已启用的 Skill 重新落一遍，不改配置、不涨修订号。
    /// 用途：切换源目录或分发方式后，把已落地的目标对齐（例如软链接改为副本）。
    ResyncSkills,
}

pub fn config_path(data_root: &Path) -> PathBuf {
    data_root.join(CONFIG_RELATIVE_PATH)
}

/// 读取统一配置的**宽松**入口：文件缺失或损坏时返回默认配置，不报错。
///
/// 只给「只读展示 / 只读发现」用（例如解析源目录、投影客户端路径）。写入路径必须走
/// `execute`，它会把损坏与冲突显式报出来。
pub fn load_config_lenient(data_root: &Path) -> ExtensionConfig {
    let path = config_path(data_root);
    let Ok(bytes) = std::fs::read(&path) else {
        return ExtensionConfig::default();
    };
    if bytes.len() as u64 > CONFIG_MAX_BYTES {
        return ExtensionConfig::default();
    }
    serde_json::from_slice::<ExtensionProjectionFile>(&bytes)
        .map(|payload| payload.config)
        .unwrap_or_default()
}

pub fn validate_data_root(data_root: &Path) -> Result<(), ProjectionError> {
    if !data_root.is_absolute() || !data_root.is_dir() {
        return Err(ProjectionError::NotConfigured);
    }
    Ok(())
}

pub fn load(
    data_root: &Path,
    targets: &[ClientTarget],
) -> Result<
    (
        ExtensionConfig,
        ProjectionFingerprint,
        Option<ProjectionConflict>,
    ),
    ProjectionError,
> {
    validate_data_root(data_root)?;
    let path = config_path(data_root);
    if !path.exists() {
        let config = ExtensionConfig::default();
        let fingerprint = fingerprint_for(&config, targets, Utc::now());
        return Ok((
            config,
            fingerprint,
            Some(ProjectionConflict {
                kind: "half_written",
            }),
        ));
    }

    let (config, fingerprint, conflict) = read_stored(&path, targets)?;
    Ok((config, fingerprint, conflict))
}

pub fn save_with_config(
    data_root: &Path,
    config: &mut ExtensionConfig,
    targets: &[ClientTarget],
) -> Result<ProjectionWriteResult, ProjectionError> {
    validate_data_root(data_root)?;
    let home = home_from_targets(targets)?;
    validate_targets(targets, home)?;
    let _guard =
        TargetFileLock::lock(&config_path(data_root)).map_err(|_| ProjectionError::LockFailed)?;
    save_locked(data_root, config, targets)
}

/// Import owns this lock from strict capture through its multi-file terminal.
/// This saves only manager extension configuration, not live client assets.
pub(crate) struct ConfigTransaction<'a> {
    data_root: &'a Path,
    targets: &'a [ClientTarget],
    _guard: TargetFileLock,
}
impl<'a> ConfigTransaction<'a> {
    pub(crate) fn acquire(
        data_root: &'a Path,
        targets: &'a [ClientTarget],
    ) -> Result<Self, ProjectionError> {
        validate_data_root(data_root)?;
        validate_targets(targets, home_from_targets(targets)?)?;
        let path = config_path(data_root);
        crate::modules::backup::safety::check_path(data_root, &path, true)
            .map_err(|_| ProjectionError::PathEscape)?;
        crate::modules::backup::safety::check_path(
            data_root,
            &crate::infrastructure::locking::lock_path_for(&path),
            true,
        )
        .map_err(|_| ProjectionError::PathEscape)?;
        Ok(Self {
            data_root,
            targets,
            _guard: TargetFileLock::lock(&path).map_err(|_| ProjectionError::LockFailed)?,
        })
    }

    pub(crate) fn load(&self) -> Result<ExtensionConfig, ProjectionError> {
        let path = config_path(self.data_root);
        crate::modules::backup::safety::check_path(self.data_root, &path, true)
            .map_err(|_| ProjectionError::PathEscape)?;
        match std::fs::symlink_metadata(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(ExtensionConfig::default())
            }
            Err(_) => Err(ProjectionError::Corrupted),
            Ok(_) => {
                let (config, _, conflict) = read_stored(&path, self.targets)?;
                if conflict.is_some() {
                    return Err(ProjectionError::ExternalModified);
                }
                Ok(config)
            }
        }
    }

    pub(crate) fn save(&self, config: &mut ExtensionConfig) -> Result<Vec<u8>, ProjectionError> {
        self.load()?;
        let saved = save_locked(self.data_root, config, self.targets)?;
        serde_json::to_vec_pretty(&ExtensionProjectionFile {
            config: saved.config,
            fingerprint: saved.fingerprint,
        })
        .map_err(|_| ProjectionError::Corrupted)
    }
}

pub fn set_client_enabled(
    data_root: &Path,
    targets: &[ClientTarget],
    client: ClientId,
    enabled: bool,
) -> Result<ProjectionWriteResult, ProjectionError> {
    execute(
        data_root,
        targets,
        ProjectionCommand::ToggleClient { client, enabled },
    )
}

/// 执行扩展写入事务：校验、锁定、权威配置替换与 live 投影在同一作用域。
pub fn execute(
    data_root: &Path,
    targets: &[ClientTarget],
    command: ProjectionCommand,
) -> Result<ProjectionWriteResult, ProjectionError> {
    validate_data_root(data_root)?;
    let home = home_from_targets(targets)?;
    validate_targets(targets, home)?;

    let path = config_path(data_root);
    let _guard = TargetFileLock::lock(&path).map_err(|_| ProjectionError::LockFailed)?;
    let mut config = if path.exists() {
        let (loaded, _, conflict) = read_stored(&path, targets)?;
        if conflict.is_some() {
            return Err(match conflict.map(|value| value.kind) {
                Some("half_written") => ProjectionError::HalfWritten,
                _ => ProjectionError::ExternalModified,
            });
        }
        loaded
    } else {
        // 首装无统一配置时按**默认配置**执行，并在本次写入落盘时把它建出来（用户决策 ②）：
        // 全新安装第一次点图标同步 / 写 MCP 不该先失败；默认源目录是 `<主目录>/.agents/skills`，
        // 用户之后可在设置页自行改成自定义目录。
        ExtensionConfig::default()
    };

    match command {
        ProjectionCommand::ToggleClient { client, enabled } => {
            let target = targets
                .iter()
                .find(|target| target.client_id == client)
                .ok_or(ProjectionError::TargetInvalid)?;
            if !target.installed {
                return Err(ProjectionError::TargetInvalid);
            }
            config.enablement.insert(client, enabled);
        }
        ProjectionCommand::LinkSkill { name, client } => {
            // 链接源按名解析：默认读源（Agent Skills 共享目录）优先，其次是
            // 管理器写入区；两处都没有该 Skill 时才失败。
            let declared =
                crate::modules::extensions::source_dir::configured_source_dir(&config, home);
            let source_dir = if config.source_store.is_some() {
                crate::modules::extensions::source_dir::validate_source_dir(
                    &declared, data_root, home, targets,
                )
                .map_err(|_| ProjectionError::SourceDirInvalid)?
            } else {
                declared
            };
            resolve_skill_source(&source_dir, data_root, &name)?;
            // 首次逐个开关时，以「当前实际已落地」的客户端作为基线，
            // 这样点一个客户端只会点亮它，不会把其余客户端一起点亮。
            config.materialize_skill_targets(&name, landed_clients(targets, &name));
            let mut selected = config.skill_targets_for(&name).cloned().unwrap_or_default();
            selected.push(client);
            // 用户显式要求同步到该客户端时，同时确保该客户端在启用矩阵里是开启的，
            // 否则意图会被启用矩阵拦下、点完图标仍不亮（界面与结果不一致）。
            config.enablement.insert(client, true);
            config.set_skill_targets(&name, selected);
        }
        ProjectionCommand::SetSourceDir { path } => {
            match path.as_deref().map(str::trim) {
                None | Some("") => {
                    // 回到默认源目录；默认目录不要求此刻已存在。
                    config.set_source_store(None);
                }
                Some(value) => {
                    let candidate = PathBuf::from(value);
                    let resolved = crate::modules::extensions::source_dir::validate_source_dir(
                        &candidate, data_root, home, targets,
                    )
                    .map_err(|_| ProjectionError::SourceDirInvalid)?;
                    config.set_source_store(Some(resolved.display().to_string()));
                }
            }
        }
        ProjectionCommand::SetSyncMethod { method } => {
            if !config.set_sync_method(method.trim()) {
                // 取值非法，或与当前一致（幂等，不算失败）。
                if !crate::modules::extensions::SYNC_METHODS.contains(&method.trim()) {
                    return Err(ProjectionError::SyncMethodInvalid);
                }
            }
        }
        ProjectionCommand::ResyncSkills => {
            // 只把既有配置按当前源目录与分发方式重新落一遍：**不改配置、不涨修订号**
            // （FZ-25 的修订号只在统一配置或启用矩阵变化时 +1）。
            let live = std::fs::read(&path).map_err(|_| ProjectionError::Corrupted)?;
            let payload: ExtensionProjectionFile =
                serde_json::from_slice(&live).map_err(|_| ProjectionError::Corrupted)?;
            payload
                .fingerprint
                .validate()
                .map_err(|_| ProjectionError::Corrupted)?;
            let skills_linked = apply_skill_links(data_root, &live, targets)?;
            return Ok(ProjectionWriteResult {
                config: payload.config,
                fingerprint: payload.fingerprint,
                document_sha256: sha256_file(&path).map_err(|_| ProjectionError::AtomicWrite)?,
                backed_up: false,
                skills_linked,
                mcp_written: Vec::new(),
            });
        }
        ProjectionCommand::UnlinkSkill {
            name,
            client,
            confirm,
        } => {
            if !confirm {
                return Err(ProjectionError::Conflict);
            }
            validate_artifact_name(&name)?;
            config.materialize_skill_targets(&name, landed_clients(targets, &name));
            let selected: Vec<ClientId> = config
                .skill_targets_for(&name)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|item| *item != client)
                .collect();
            // 关到最后一个客户端时，该 Skill 会从分发清单里摘掉；投影只遍历清单，
            // 所以这里必须显式断开这一个客户端的链接（源码与其它客户端一律不动）。
            if selected.is_empty() {
                if let Some(target) = targets.iter().find(|target| target.client_id == client) {
                    remove_manager_link(&target.skills_dir.join(&name))?;
                }
            }
            config.set_skill_targets(&name, selected);
        }
        ProjectionCommand::ImportSkillArchive {
            archive_name,
            archive_payload,
        } => {
            import_skill_archive(data_root, &archive_name, &archive_payload)?;
            if let Some(root_name) = skill_archive_root_name(&archive_payload)? {
                if !config.skills.contains(&root_name) {
                    config.skills.push(root_name);
                }
            }
        }
        ProjectionCommand::RestoreSkill { name } => {
            validate_artifact_name(&name)?;
            restore_skill_from_backup(data_root, &name)?;
        }
        ProjectionCommand::AddMcp { definition } => {
            validate_artifact_name(&definition.name)?;
            validate_server_definition(&definition)?;
            if config.servers.contains(&definition.name)
                || source_server_definition(targets, &definition.name)?.is_some()
            {
                return Err(ProjectionError::Conflict);
            }
            let written = write_server_definition(data_root, &definition, None, targets, &config)?;
            config.servers.push(definition.name.clone());
            let mut result = save_locked(data_root, &mut config, targets)?;
            result.mcp_written = written;
            let live = std::fs::read(&path).map_err(|_| ProjectionError::AtomicWrite)?;
            result.skills_linked = apply_skill_links(data_root, &live, targets)?;
            return Ok(result);
        }
        ProjectionCommand::EditMcp {
            name,
            definition,
            confirm,
        } => {
            validate_artifact_name(&name)?;
            validate_artifact_name(&definition.name)?;
            validate_server_definition(&definition)?;
            if !confirm && source_server_definition(targets, &name)?.is_none() {
                return Err(ProjectionError::Conflict);
            }
            let written =
                write_server_definition(data_root, &definition, Some(&name), targets, &config)?;
            config.servers.retain(|item| item != &name);
            if !config.servers.contains(&definition.name) {
                config.servers.push(definition.name.clone());
            }
            let mut result = save_locked(data_root, &mut config, targets)?;
            result.mcp_written = written;
            let live = std::fs::read(&path).map_err(|_| ProjectionError::AtomicWrite)?;
            result.skills_linked = apply_skill_links(data_root, &live, targets)?;
            return Ok(result);
        }
        ProjectionCommand::UninstallSkill { name, confirm } => {
            if !confirm {
                return Err(ProjectionError::Conflict);
            }
            validate_artifact_name(&name)?;
            uninstall_skill(data_root, targets, &config, &name)?;
            config.skills.retain(|item| item != &name);
            config.skill_targets.remove(&name);
        }
        ProjectionCommand::WriteMcp {
            name,
            client,
            confirm,
        } => {
            validate_artifact_name(&name)?;
            let definition =
                source_server_definition(targets, &name)?.ok_or(ProjectionError::TargetInvalid)?;
            let mut written = Vec::new();
            let mut prepared = Vec::new();
            for target in mcp_targets(targets, &config, Some(client))? {
                let output = project_server_to_target(&definition, target, Some(&name))
                    .ok_or(ProjectionError::UnsupportedTarget)?;
                let original = std::fs::read(&target.mcp_config_path).ok();
                if !confirm {
                    let valid = original.as_deref().is_some_and(|bytes| {
                        parse_target_payload(target, bytes).is_ok_and(|payload| {
                            payload
                                .get(&target.mcp_key)
                                .is_some_and(|node| contains_server(node, target, &name))
                        })
                    });
                    if !valid {
                        return Err(ProjectionError::Conflict);
                    }
                }
                prepared.push((target.clone(), original, output));
            }
            let payloads = prepared
                .iter()
                .map(|(target, _, output)| {
                    let node: serde_json::Value =
                        serde_json::from_slice(output).map_err(|_| ProjectionError::Corrupted)?;
                    Ok::<_, ProjectionError>((target.clone(), node))
                })
                .collect::<Result<Vec<_>, ProjectionError>>()?;
            let mut merged = Vec::new();
            for (target, node) in payloads {
                let original = std::fs::read(&target.mcp_config_path).ok();
                let mut payload = match original.as_deref() {
                    Some(bytes) => parse_target_payload(&target, bytes)?,
                    None => serde_json::Map::new().into(),
                };
                let container = payload
                    .as_object_mut()
                    .ok_or(ProjectionError::UnsupportedTarget)?
                    .entry(target.mcp_key.clone())
                    .or_insert_with(|| match target.container_shape {
                        ContainerShape::Map => serde_json::Map::new().into(),
                        ContainerShape::Array => serde_json::Value::Array(Vec::new()),
                    });
                upsert_server(container, &target, node, &name, Some(&name))?;
                let output = serialize_target_payload(&target, &payload)?;
                write_mcp_payload(data_root, &target, original.as_deref(), &output)?;
                merged.push(target);
            }
            for target in merged {
                written.push(client_label(target.client_id));
            }
            if !config.servers.contains(&name) {
                config.servers.push(name);
            }
            let mut result = save_locked(data_root, &mut config, targets)?;
            result.mcp_written = written;
            let live = std::fs::read(&path).map_err(|_| ProjectionError::AtomicWrite)?;
            result.skills_linked = apply_skill_links(data_root, &live, targets)?;
            return Ok(result);
        }
        ProjectionCommand::RemoveMcp {
            name,
            client,
            confirm,
        } => {
            if !confirm {
                return Err(ProjectionError::Conflict);
            }
            validate_artifact_name(&name)?;
            let mut written = Vec::new();
            for target in mcp_targets(targets, &config, client)? {
                let Some(original) = std::fs::read(&target.mcp_config_path).ok() else {
                    continue;
                };
                let mut payload = parse_target_payload(target, &original)?;
                if let Some(node) = payload.get_mut(&target.mcp_key) {
                    remove_server_from_node(node, target, &name)?;
                }
                // 必须按目标格式序列化：Codex 是 TOML，YAML 客户端是 YAML；
                // 这里曾直接用 JSON 落盘，会把 `config.toml` 写成 JSON，
                // 之后该客户端的读/写/断开全部失败（且不可自行恢复）。
                let output = serialize_target_payload(target, &payload)?;
                write_mcp_payload(data_root, target, Some(&original), &output)?;
                written.push(client_label(target.client_id));
            }
            // 只断开某一个客户端时，定义仍在其它客户端配置里，保留在受管清单中；
            // 删除定义（`None`）才从受管清单里摘掉。
            if client.is_none() {
                config.servers.retain(|item| item != &name);
            }
            let mut result = save_locked(data_root, &mut config, targets)?;
            result.mcp_written = written;
            let live = std::fs::read(&path).map_err(|_| ProjectionError::AtomicWrite)?;
            result.skills_linked = apply_skill_links(data_root, &live, targets)?;
            return Ok(result);
        }
    }

    let mut result = save_locked(data_root, &mut config, targets)?;
    let live = std::fs::read(&path).map_err(|_| ProjectionError::AtomicWrite)?;
    result.skills_linked = apply_skill_links(data_root, &live, targets)?;
    result.mcp_written = Vec::new();
    Ok(result)
}

pub fn apply_skill_links(
    data_root: &Path,
    payload: &[u8],
    targets: &[ClientTarget],
) -> Result<Vec<String>, ProjectionError> {
    let projection: ExtensionProjectionFile =
        serde_json::from_slice(payload).map_err(|_| ProjectionError::Corrupted)?;
    projection
        .fingerprint
        .validate()
        .map_err(|_| ProjectionError::Corrupted)?;
    // 旧 schema 的指纹按旧结构计算，升级后必然不相等；这不是外部改动。
    if !projection.fingerprint.needs_migration()
        && projection.config.fingerprint() != projection.fingerprint.fingerprint
    {
        return Err(ProjectionError::ExternalModified);
    }
    let home = home_from_targets(targets)?;
    // FZ-23：源目录是配置值（默认为共享目录，可为自定义目录）。写入是显式动作，
    // 这里必须走严格校验——非法配置直接失败，不静默换一个目录去写。
    let declared =
        crate::modules::extensions::source_dir::configured_source_dir(&projection.config, home);
    let source_dir = if projection.config.source_store.is_some() {
        crate::modules::extensions::source_dir::validate_source_dir(
            &declared, data_root, home, targets,
        )
        .map_err(|_| ProjectionError::SourceDirInvalid)?
    } else {
        declared
    };
    // FZ-24：分发方式由用户选择；`symlink` 创建失败时自动回退 `copy`。
    let prefer_copy = projection.config.uses_copy();

    let mut linked = Vec::new();
    for name in &projection.config.skills {
        // 每个 Skill 独立解析来源：源目录优先，其次管理器写入区。
        let source = resolve_skill_source(&source_dir, data_root, name)?;
        // 逐客户端意图：`None` = 该 Skill 还没逐个开关过，沿用「所有启用客户端」。
        let selected = projection.config.skill_targets_for(name);
        for target in targets.iter().filter(|target| target.can_write()) {
            let destination = target.skills_dir.join(name);
            let wanted = projection.config.enablement.get(&target.client_id) == Some(&true)
                && selected.is_none_or(|clients| clients.contains(&target.client_id));
            if !wanted {
                // 关闭该客户端 = 只断开管理器建立的链接；源码与用户自有目录一律不动。
                remove_manager_link(&destination)?;
                continue;
            }
            if let Some(parent) = destination.parent() {
                std::fs::create_dir_all(parent).map_err(|_| ProjectionError::AtomicWrite)?;
            }
            match std::fs::symlink_metadata(&destination) {
                Ok(existing) if existing.is_symlink() => {
                    let same_link = std::fs::read_link(&destination)
                        .map_err(|_| ProjectionError::AtomicWrite)?
                        == source;
                    // 软链接模式且已指向正确来源：无需重做。
                    if same_link && !prefer_copy {
                        continue;
                    }
                    std::fs::remove_file(&destination).map_err(|_| ProjectionError::AtomicWrite)?;
                }
                // 目标已是同名真实目录（用户自有内容，或上一轮「文件复制」的产物）：
                // 按「不静默覆盖」契约跳过，不因此让其它 Skill / 客户端整体失败。
                Ok(_) => continue,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => return Err(ProjectionError::AtomicWrite),
            }

            if !prefer_copy
                && crate::infrastructure::platform::symlink_file(&source, &destination).is_ok()
            {
                linked.push(format!("{}/{}", client_label(target.client_id), name));
                continue;
            }
            // FZ-24 回退：软链接创建失败（或用户选择复制）时写一份独立副本。
            copy_skill_tree(&source, &destination)?;
            linked.push(format!("{}/{}", client_label(target.client_id), name));
        }
    }
    Ok(linked)
}

/// FZ-24 复制投影：递归复制常规文件与目录，保留文件模式，**不复制符号链接**
/// （避免把链接带进客户端目录），深度上限与 symlink 校验一致。
fn copy_skill_tree(source: &Path, destination: &Path) -> Result<(), ProjectionError> {
    const MAX_DEPTH: u32 = 40;
    std::fs::create_dir_all(destination).map_err(|_| ProjectionError::AtomicWrite)?;
    let mut stack = vec![(source.to_path_buf(), destination.to_path_buf(), 0u32)];
    while let Some((from, to, depth)) = stack.pop() {
        if depth > MAX_DEPTH {
            return Err(ProjectionError::AtomicWrite);
        }
        let entries = std::fs::read_dir(&from).map_err(|_| ProjectionError::AtomicWrite)?;
        for entry in entries {
            let entry = entry.map_err(|_| ProjectionError::AtomicWrite)?;
            let path = entry.path();
            let metadata =
                std::fs::symlink_metadata(&path).map_err(|_| ProjectionError::AtomicWrite)?;
            let name = entry.file_name();
            let next = to.join(&name);
            if metadata.is_symlink() {
                continue;
            }
            if metadata.is_dir() {
                std::fs::create_dir_all(&next).map_err(|_| ProjectionError::AtomicWrite)?;
                stack.push((path, next, depth + 1));
            } else if metadata.is_file() {
                std::fs::copy(&path, &next).map_err(|_| ProjectionError::AtomicWrite)?;
                let _ = crate::infrastructure::platform::copy_mode(&path, &next);
            }
        }
    }
    Ok(())
}

fn validate_artifact_name(name: &str) -> Result<(), ProjectionError> {
    if name.trim().is_empty()
        || name.contains("..")
        || name.starts_with('/')
        || name.contains('\\')
        || name.split('/').any(str::is_empty)
    {
        return Err(ProjectionError::PathEscape);
    }
    Ok(())
}

/// 解析一个 Skill 的链接源（规范化绝对路径）。
///
/// 顺序：默认读源 `<主目录>/.agents/skills`（Agent Skills 共享目录）→ 管理器
/// 写入区 `<数据根>/manager-state/skills-store`（导入 / 恢复产物）。两处都必须
/// 是真实目录（拒绝符号链接，避免链接指向链接）。
fn resolve_skill_source(
    source_dir: &Path,
    data_root: &Path,
    name: &str,
) -> Result<PathBuf, ProjectionError> {
    use crate::modules::extensions::discovery;
    validate_artifact_name(name)?;
    for root in [
        source_dir.to_path_buf(),
        discovery::skills_store_dir(data_root),
    ] {
        let Ok(canonical_root) = root.canonicalize() else {
            continue;
        };
        let candidate = canonical_root.join(name);
        match std::fs::symlink_metadata(&candidate) {
            Ok(metadata) if metadata.is_dir() && !metadata.is_symlink() => return Ok(candidate),
            _ => continue,
        }
    }
    Err(ProjectionError::TargetInvalid)
}

fn source_server_definition(
    targets: &[ClientTarget],
    name: &str,
) -> Result<Option<RawServerDefinition>, ProjectionError> {
    for target in targets {
        match read_server_definition(target, name) {
            Ok(Some(value)) => return Ok(Some(value)),
            Ok(None) => {}
            Err(_) => return Err(ProjectionError::Corrupted),
        }
    }
    Ok(None)
}

fn write_targets<'a>(
    targets: &'a [ClientTarget],
    config: &ExtensionConfig,
) -> Vec<&'a ClientTarget> {
    targets
        .iter()
        .filter(|target| {
            target.can_write() && config.enablement.get(&target.client_id) == Some(&true)
        })
        .collect()
}

/// MCP 写入目标：`Some(client)` = 只操作该客户端（断开时不受启用矩阵影响，
/// 用户要它消失就必须能消失）；`None` = 全部启用客户端（删除定义）。
fn mcp_targets<'a>(
    targets: &'a [ClientTarget],
    config: &ExtensionConfig,
    client: Option<ClientId>,
) -> Result<Vec<&'a ClientTarget>, ProjectionError> {
    match client {
        None => Ok(write_targets(targets, config)),
        Some(client) => {
            let target = targets
                .iter()
                .find(|target| target.can_write() && target.client_id == client)
                .ok_or(ProjectionError::TargetInvalid)?;
            Ok(vec![target])
        }
    }
}

fn client_label(client: ClientId) -> String {
    serde_json::to_value(client)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_default()
}

fn serialize_server_node(
    target: &ClientTarget,
    value: &serde_json::Value,
    previous_name: Option<&str>,
) -> serde_json::Value {
    if matches!(target.container_shape, ContainerShape::Array) {
        let mut node = value.clone();
        if let Some(object) = node.as_object_mut() {
            let mut object = object
                .into_iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<serde_json::Map<_, _>>();
            let mut output = serde_json::Map::new();
            let name = object
                .get("name")
                .cloned()
                .unwrap_or(serde_json::Value::String(String::new()));
            let same_server = name.as_str() == previous_name;
            if let Some(previous) = previous_name {
                object.retain(|key, item| {
                    let node_name = item.get("name").and_then(serde_json::Value::as_str);
                    if same_server {
                        return true;
                    }
                    node_name != Some(previous) && key != previous
                });
            }
            output.insert("name".to_string(), name);
            output.extend(object.into_iter().filter(|(key, _)| key != "name"));
            return serde_json::Value::Object(output);
        }
        return node;
    }
    value.clone()
}

fn project_server_to_target(
    definition: &RawServerDefinition,
    target: &ClientTarget,
    previous_name: Option<&str>,
) -> Option<Vec<u8>> {
    let mut value = definition.value.clone();
    let is_array = matches!(target.container_shape, ContainerShape::Array);
    if let Some(object) = value.as_object_mut() {
        if is_array && !object.contains_key("name") {
            object.insert(
                "name".to_string(),
                serde_json::Value::String(definition.name.clone()),
            );
        }
    }
    adapt_shape(&mut value, target)?;
    normalize_env_key(&mut value, target);
    let node = serialize_server_node(target, &value, previous_name);
    serde_json::to_vec(&node).ok()
}

fn adapt_shape(value: &mut serde_json::Value, target: &ClientTarget) -> Option<()> {
    match target.format {
        ConfigFormat::Toml => {
            let object = value.as_object_mut()?;
            let command = object
                .get("command")
                .and_then(serde_json::Value::as_str)?
                .to_string();
            let args = object
                .get("args")
                .and_then(serde_json::Value::as_array)
                .cloned()
                .unwrap_or_default();
            if args.iter().any(|item| item.as_str().is_none()) {
                return None;
            }
            object.insert("command".to_string(), serde_json::Value::String(command));
            object.insert("args".to_string(), serde_json::Value::Array(args));
        }
        ConfigFormat::Yaml => {
            let object = value.as_object_mut()?;
            for key in ["command", "args"] {
                if let Some(item) = object.get(key).cloned() {
                    let converted = serde_json::from_value::<serde_yaml::Value>(item).ok()?;
                    let converted = serde_yaml::to_value(converted).ok()?;
                    let converted = serde_json::to_value(converted).ok()?;
                    object.insert(key.to_string(), converted);
                }
            }
        }
        ConfigFormat::Json | ConfigFormat::Jsonc => {}
    }
    let transport = value
        .get("type")
        .or_else(|| value.get("transport"))
        .or_else(|| value.get("transport_type"))
        .and_then(serde_json::Value::as_str)
        .map(|value| match value {
            "local" => "stdio",
            _ => value,
        })
        .unwrap_or(if value.get("url").is_some() {
            "remote"
        } else {
            "stdio"
        })
        .to_string();
    let object = value.as_object_mut()?;
    object.insert("type".to_string(), serde_json::Value::String(transport));
    Some(())
}

fn normalize_env_key(value: &mut serde_json::Value, target: &ClientTarget) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    let source_key = if target.env_key_name == "environment" {
        "env"
    } else {
        "environment"
    };
    if let Some(env) = object.remove(source_key) {
        object.insert(target.env_key_name.clone(), env);
    }
}

fn parse_target_payload(
    target: &ClientTarget,
    payload: &[u8],
) -> Result<serde_json::Value, ProjectionError> {
    let text = String::from_utf8(payload.to_vec()).map_err(|_| ProjectionError::Corrupted)?;
    let value = match target.format {
        ConfigFormat::Json | ConfigFormat::Jsonc => {
            let cleaned = crate::modules::extensions::discovery::strip_json_comments(&text);
            serde_json::from_str(&cleaned).map_err(|_| ProjectionError::Corrupted)?
        }
        ConfigFormat::Toml => {
            let value: toml::Value =
                toml::from_str(&text).map_err(|_| ProjectionError::Corrupted)?;
            serde_json::to_value(value).map_err(|_| ProjectionError::Corrupted)?
        }
        ConfigFormat::Yaml => {
            let value: serde_yaml::Value =
                serde_yaml::from_str(&text).map_err(|_| ProjectionError::Corrupted)?;
            serde_json::to_value(value).map_err(|_| ProjectionError::Corrupted)?
        }
    };
    if !value.is_object() {
        return Err(ProjectionError::UnsupportedTarget);
    }
    Ok(value)
}

fn contains_server(node: &serde_json::Value, target: &ClientTarget, name: &str) -> bool {
    match target.container_shape {
        ContainerShape::Map => node.get(name).is_some(),
        ContainerShape::Array => node.as_array().is_some_and(|items| {
            items.iter().any(|item| {
                item.get("name")
                    .or_else(|| item.get("id"))
                    .or_else(|| item.get("server_name"))
                    .and_then(serde_json::Value::as_str)
                    == Some(name)
            })
        }),
    }
}

fn remove_server_from_node(
    node: &mut serde_json::Value,
    target: &ClientTarget,
    name: &str,
) -> Result<(), ProjectionError> {
    match target.container_shape {
        ContainerShape::Map => {
            let map = node
                .as_object_mut()
                .ok_or(ProjectionError::UnsupportedTarget)?;
            map.remove(name);
        }
        ContainerShape::Array => {
            let items = node
                .as_array_mut()
                .ok_or(ProjectionError::UnsupportedTarget)?;
            items.retain(|item| {
                item.get("name")
                    .or_else(|| item.get("id"))
                    .or_else(|| item.get("server_name"))
                    .and_then(serde_json::Value::as_str)
                    != Some(name)
            });
        }
    }
    Ok(())
}

fn upsert_server(
    container: &mut serde_json::Value,
    target: &ClientTarget,
    node: serde_json::Value,
    name: &str,
    previous_name: Option<&str>,
) -> Result<(), ProjectionError> {
    match target.container_shape {
        ContainerShape::Map => {
            let map = container
                .as_object_mut()
                .ok_or(ProjectionError::UnsupportedTarget)?;
            let server_name = definition_name(&node).unwrap_or_else(|| name.to_string());
            if let Some(previous) = previous_name {
                map.remove(previous);
            }
            map.insert(server_name, node);
        }
        ContainerShape::Array => {
            let items = container
                .as_array_mut()
                .ok_or(ProjectionError::UnsupportedTarget)?;
            let server_name = definition_name(&node).unwrap_or_else(|| name.to_string());
            items.retain(|item| {
                let item_name = item
                    .get("name")
                    .or_else(|| item.get("id"))
                    .or_else(|| item.get("server_name"))
                    .and_then(serde_json::Value::as_str);
                item_name != Some(server_name.as_str())
                    && previous_name.is_none_or(|previous| item_name != Some(previous))
            });
            items.push(node);
        }
    }
    Ok(())
}

fn definition_name(node: &serde_json::Value) -> Option<String> {
    node.get("name")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

fn serialize_target_payload(
    target: &ClientTarget,
    payload: &serde_json::Value,
) -> Result<Vec<u8>, ProjectionError> {
    let mut text = match target.format {
        ConfigFormat::Json | ConfigFormat::Jsonc => {
            serde_json::to_vec_pretty(payload).map_err(|_| ProjectionError::Corrupted)?
        }
        ConfigFormat::Toml => {
            let value: toml::Value =
                serde_json::from_value(payload.clone()).map_err(|_| ProjectionError::Corrupted)?;
            toml::to_string_pretty(&value)
                .map_err(|_| ProjectionError::Corrupted)?
                .into_bytes()
        }
        ConfigFormat::Yaml => {
            let value: serde_yaml::Value =
                serde_json::from_value(payload.clone()).map_err(|_| ProjectionError::Corrupted)?;
            serde_yaml::to_string(&value)
                .map_err(|_| ProjectionError::Corrupted)?
                .into_bytes()
        }
    };
    if !text.ends_with(b"\n") {
        text.push(b'\n');
    }
    Ok(text)
}

fn write_mcp_payload(
    data_root: &Path,
    target: &ClientTarget,
    original: Option<&[u8]>,
    output: &[u8],
) -> Result<(), ProjectionError> {
    if target.mcp_config_path.is_symlink() {
        return Err(ProjectionError::TargetType);
    }
    let parent = target
        .mcp_config_path
        .parent()
        .ok_or(ProjectionError::TargetInvalid)?;
    // 边界校验：客户端配置文件必须落在解析后的主目录之内。
    // 曾用 `skills_dir.ancestors().nth(4)` 充当主目录，但各客户端 skills_dir 深度不同
    // （`.config/opencode/skills` 比 `.claude/skills` 深一层），该推导在真实 `$HOME`
    // 下会退化成 `/`，隔离根下则落到 `$HOME/..`；备份也因此写到了数据根之外。
    if !parent.starts_with(&target.home) {
        return Err(ProjectionError::PathEscape);
    }
    if let Some(payload) = original {
        backup_file(
            data_root,
            BackupAction::ExtensionWrite,
            &target.mcp_config_path,
            payload,
            Utc::now(),
            Some("before extension MCP projection write".to_string()),
        )
        .map_err(|_| ProjectionError::Backup)?;
    }
    std::fs::create_dir_all(parent).map_err(|_| ProjectionError::AtomicWrite)?;
    crate::infrastructure::atomic_write::atomic_write(&target.mcp_config_path, output, 0o600)
        .map_err(|_| ProjectionError::AtomicWrite)
}

fn validate_zip_member_name(name: &str) -> Result<PathBuf, ProjectionError> {
    if name.is_empty() || name.starts_with('/') || name.contains('\\') || name.contains('\0') {
        return Err(ProjectionError::InvalidArchive);
    }
    let path = PathBuf::from(name);
    let mut depth = 0usize;
    for component in path.components() {
        match component {
            std::path::Component::Normal(part) => {
                if part.is_empty() || part == "." || part == ".." {
                    return Err(ProjectionError::InvalidArchive);
                }
                depth += 1;
                if depth > 64 {
                    return Err(ProjectionError::InvalidArchive);
                }
            }
            _ => return Err(ProjectionError::InvalidArchive),
        }
    }
    Ok(path)
}

fn read_skill_archive(path: &Path) -> Result<Vec<(PathBuf, Vec<u8>)>, ProjectionError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| ProjectionError::InvalidArchive)?;
    if !metadata.is_file() {
        return Err(ProjectionError::InvalidArchive);
    }
    if metadata.len() > ARCHIVE_MAX_BYTES {
        return Err(ProjectionError::TooLarge);
    }
    let file = std::fs::File::open(path).map_err(|_| ProjectionError::InvalidArchive)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|_| ProjectionError::InvalidArchive)?;
    if archive.len() > crate::modules::skills::ARCHIVE_MAX_ENTRIES {
        return Err(ProjectionError::TooLarge);
    }
    let mut entries = Vec::with_capacity(archive.len());
    let mut total = 0_u64;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|_| ProjectionError::InvalidArchive)?;
        let relative = entry
            .enclosed_name()
            .map(|path| path.to_path_buf())
            .or_else(|| validate_zip_member_name(entry.name()).ok())
            .ok_or(ProjectionError::InvalidArchive)?;
        if entry.is_dir() {
            continue;
        }
        let size = entry.size();
        total = total.saturating_add(size);
        if size > crate::modules::skills::ARCHIVE_SINGLE_FILE_MAX_BYTES
            || total > crate::modules::skills::ARCHIVE_MAX_TOTAL_BYTES
        {
            return Err(ProjectionError::TooLarge);
        }
        let mut payload = Vec::with_capacity(size.min(1024 * 1024) as usize);
        entry
            .read_to_end(&mut payload)
            .map_err(|_| ProjectionError::InvalidArchive)?;
        if payload.len() as u64 != size {
            return Err(ProjectionError::InvalidArchive);
        }
        entries.push((relative, payload));
    }
    Ok(entries)
}

fn skill_store(data_root: &Path) -> Result<PathBuf, ProjectionError> {
    let store = crate::modules::extensions::discovery::skills_store_dir(data_root);
    std::fs::create_dir_all(&store).map_err(|_| ProjectionError::AtomicWrite)?;
    store
        .canonicalize()
        .map_err(|_| ProjectionError::TargetInvalid)
}

fn safe_store_path(store: &Path, name: &str) -> Result<PathBuf, ProjectionError> {
    validate_artifact_name(name)?;
    let target = store.join(name);
    if !target.starts_with(store) {
        return Err(ProjectionError::PathEscape);
    }
    Ok(target)
}

fn write_skill_payload(
    data_root: &Path,
    store: &Path,
    entries: Vec<(PathBuf, Vec<u8>)>,
) -> Result<(), ProjectionError> {
    let root_name = entries
        .first()
        .and_then(|(path, _)| path.components().next())
        .and_then(|component| component.as_os_str().to_str())
        .ok_or(ProjectionError::InvalidArchive)?
        .to_string();
    validate_artifact_name(&root_name)?;
    let target = safe_store_path(store, &root_name)?;
    if target.exists() {
        backup_skill_directory(data_root, &target, "before skill import overwrite")?;
    }
    std::fs::create_dir_all(&target).map_err(|_| ProjectionError::AtomicWrite)?;
    for (relative, payload) in entries {
        let destination = target.join(
            relative
                .strip_prefix(root_name.as_str())
                .unwrap_or(&relative),
        );
        if !destination.starts_with(&target) {
            return Err(ProjectionError::PathEscape);
        }
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).map_err(|_| ProjectionError::AtomicWrite)?;
        }
        crate::infrastructure::atomic_write::atomic_write(&destination, &payload, 0o600)
            .map_err(|_| ProjectionError::AtomicWrite)?;
    }
    if std::fs::read_dir(&target)
        .map_err(|_| ProjectionError::InvalidArchive)?
        .next()
        .is_none()
    {
        return Err(ProjectionError::InvalidArchive);
    }
    Ok(())
}

fn validate_archive_name(filename: &str) -> Result<(), ProjectionError> {
    let path = Path::new(filename);
    if filename.is_empty()
        || path.as_os_str().is_empty()
        || path.components().count() != 1
        || !matches!(
            path.components().next(),
            Some(std::path::Component::Normal(_))
        )
    {
        return Err(ProjectionError::InvalidArchive);
    }
    Ok(())
}

fn import_skill_archive(
    data_root: &Path,
    archive_name: &str,
    archive_payload: &[u8],
) -> Result<(), ProjectionError> {
    if archive_payload.len() as u64 > ARCHIVE_MAX_BYTES {
        return Err(ProjectionError::TooLarge);
    }
    validate_archive_name(archive_name)?;
    let export_dir = data_root.join("exports");
    std::fs::create_dir_all(&export_dir).map_err(|_| ProjectionError::AtomicWrite)?;
    let export_path = export_dir.join(archive_name);
    if !export_path.starts_with(&export_dir) || export_path.symlink_metadata().is_ok() {
        return Err(ProjectionError::PathEscape);
    }
    crate::infrastructure::atomic_write::atomic_write(&export_path, archive_payload, 0o600)
        .map_err(|_| ProjectionError::AtomicWrite)?;
    let entries = read_skill_archive(&export_path)?;
    let store = skill_store(data_root)?;
    write_skill_payload(data_root, &store, entries)
}

fn skill_archive_root_name(payload: &[u8]) -> Result<Option<String>, ProjectionError> {
    if payload.is_empty() {
        return Err(ProjectionError::InvalidArchive);
    }
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(payload))
        .map_err(|_| ProjectionError::InvalidArchive)?;
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|_| ProjectionError::InvalidArchive)?;
        if entry.is_dir() {
            continue;
        }
        let relative = entry
            .enclosed_name()
            .map(|path| path.to_path_buf())
            .or_else(|| validate_zip_member_name(entry.name()).ok())
            .ok_or(ProjectionError::InvalidArchive)?;
        return Ok(relative
            .components()
            .next()
            .map(|component| component.as_os_str().to_string_lossy().into_owned()));
    }
    Ok(None)
}

fn restore_skill_from_backup(data_root: &Path, name: &str) -> Result<(), ProjectionError> {
    let store = crate::modules::extensions::discovery::skills_store_dir(data_root)
        .canonicalize()
        .map_err(|_| ProjectionError::TargetInvalid)?;
    validate_artifact_name(name)?;
    let backups = data_root.join("backups");
    let mut candidates = Vec::new();
    for year in std::fs::read_dir(&backups).map_err(|_| ProjectionError::Backup)? {
        let year = year.map_err(|_| ProjectionError::Backup)?.path();
        for month in std::fs::read_dir(&year).map_err(|_| ProjectionError::Backup)? {
            let action_dir = month
                .map_err(|_| ProjectionError::Backup)?
                .path()
                .join("extension-write");
            if !action_dir.is_dir() {
                continue;
            }
            for directory in std::fs::read_dir(&action_dir).map_err(|_| ProjectionError::Backup)? {
                let directory = directory.map_err(|_| ProjectionError::Backup)?.path();
                let manifest_path = directory.join(crate::modules::backup::MANIFEST_NAME);
                if !manifest_path.is_file() {
                    continue;
                }
                let bytes = std::fs::read(&manifest_path).map_err(|_| ProjectionError::Backup)?;
                let manifest: crate::modules::backup::BackupManifest =
                    serde_json::from_slice(&bytes).map_err(|_| ProjectionError::Backup)?;
                if std::path::Path::new(&manifest.target_path)
                    .file_name()
                    .and_then(|value| value.to_str())
                    != Some(name)
                {
                    continue;
                }
                candidates.push((manifest, directory.clone()));
            }
        }
    }
    candidates.sort_by(|left, right| right.0.stored_at.cmp(&left.0.stored_at));
    let (manifest, directory) = candidates.first().ok_or(ProjectionError::NotFound)?;
    let stored_path = directory.join(
        manifest
            .target_path
            .rsplit('/')
            .next()
            .ok_or(ProjectionError::Backup)?,
    );
    let payload = std::fs::read(&stored_path).map_err(|_| ProjectionError::Backup)?;
    let actual = sha256_file(&stored_path).map_err(|_| ProjectionError::Backup)?;
    if actual != manifest.sha256 {
        return Err(ProjectionError::Backup);
    }
    let current = std::fs::symlink_metadata(store.join(name)).ok();
    if let Some(metadata) = current {
        if metadata.is_dir() && !metadata.is_symlink() {
            std::fs::remove_dir_all(store.join(name)).map_err(|_| ProjectionError::AtomicWrite)?;
        } else {
            return Err(ProjectionError::TargetType);
        }
    }
    std::fs::create_dir_all(store.join(name)).map_err(|_| ProjectionError::AtomicWrite)?;
    let mut archive =
        zip::ZipArchive::new(std::io::Cursor::new(payload)).map_err(|_| ProjectionError::Backup)?;
    let mut entries = Vec::new();
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|_| ProjectionError::Backup)?;
        let relative = entry
            .enclosed_name()
            .map(|path| path.to_path_buf())
            .ok_or(ProjectionError::Backup)?;
        if entry.is_dir() {
            continue;
        }
        let mut payload = Vec::new();
        entry
            .read_to_end(&mut payload)
            .map_err(|_| ProjectionError::Backup)?;
        entries.push((relative, payload));
    }
    for (relative, payload) in entries {
        let destination = store.join(name).join(relative);
        if !destination.starts_with(store.join(name)) {
            return Err(ProjectionError::PathEscape);
        }
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).map_err(|_| ProjectionError::AtomicWrite)?;
        }
        crate::infrastructure::atomic_write::atomic_write(&destination, &payload, 0o600)
            .map_err(|_| ProjectionError::AtomicWrite)?;
    }
    Ok(())
}

fn validate_server_definition(definition: &RawServerDefinition) -> Result<(), ProjectionError> {
    let object = definition
        .value
        .as_object()
        .ok_or(ProjectionError::UnsupportedTarget)?;
    let command = object
        .get("command")
        .and_then(serde_json::Value::as_str)
        .ok_or(ProjectionError::UnsupportedTarget)?;
    if command.trim().is_empty() {
        return Err(ProjectionError::UnsupportedTarget);
    }
    if let Some(args) = object.get("args") {
        if !args.is_array()
            || args
                .as_array()
                .unwrap()
                .iter()
                .any(|value| !value.is_string())
        {
            return Err(ProjectionError::UnsupportedTarget);
        }
    }
    if let Some(env) = object.get("env") {
        if !env.is_object()
            || env
                .as_object()
                .unwrap()
                .iter()
                .any(|(_, value)| !value.is_string())
        {
            return Err(ProjectionError::UnsupportedTarget);
        }
    }
    let remote = object.get("url").is_some();
    if remote {
        let url = object
            .get("url")
            .and_then(serde_json::Value::as_str)
            .ok_or(ProjectionError::UnsupportedTarget)?;
        if !(url.starts_with("http://") || url.starts_with("https://"))
            || !LOCAL_HOST_PREFIXES
                .iter()
                .any(|prefix| url.contains(prefix))
        {
            return Err(ProjectionError::UnsupportedTarget);
        }
    }
    Ok(())
}

fn write_server_definition(
    data_root: &Path,
    definition: &RawServerDefinition,
    previous_name: Option<&str>,
    targets: &[ClientTarget],
    config: &ExtensionConfig,
) -> Result<Vec<String>, ProjectionError> {
    let mut written = Vec::new();
    for target in write_targets(targets, config) {
        let output = project_server_to_target(definition, target, previous_name)
            .ok_or(ProjectionError::UnsupportedTarget)?;
        let original = std::fs::read(&target.mcp_config_path).ok();
        let mut payload = match original.as_deref() {
            Some(bytes) => parse_target_payload(target, bytes)?,
            None => serde_json::Map::new().into(),
        };
        let container = payload
            .as_object_mut()
            .ok_or(ProjectionError::UnsupportedTarget)?
            .entry(target.mcp_key.clone())
            .or_insert_with(|| match target.container_shape {
                ContainerShape::Map => serde_json::Map::new().into(),
                ContainerShape::Array => serde_json::Value::Array(Vec::new()),
            });
        upsert_server(
            container,
            target,
            serde_json::from_slice(&output).map_err(|_| ProjectionError::Corrupted)?,
            &definition.name,
            previous_name,
        )?;
        let output = serialize_target_payload(target, &payload)?;
        write_mcp_payload(data_root, target, original.as_deref(), &output)?;
        written.push(client_label(target.client_id));
    }
    if written.is_empty() {
        return Err(ProjectionError::TargetInvalid);
    }
    Ok(written)
}

fn backup_skill_directory(
    data_root: &Path,
    target: &Path,
    note: &'static str,
) -> Result<(), ProjectionError> {
    let mut files = Vec::new();
    collect_files(target, &mut files).map_err(|_| ProjectionError::Backup)?;
    if files.is_empty() {
        return Err(ProjectionError::NotFound);
    }
    let mut zip_buffer = std::io::Cursor::new(Vec::new());
    {
        let mut archive = zip::ZipWriter::new(&mut zip_buffer);
        let options: zip::write::SimpleFileOptions = zip::write::FileOptions::default();
        for file in files {
            let relative = file
                .strip_prefix(target)
                .map_err(|_| ProjectionError::Backup)?
                .components()
                .map(|component| component.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            let payload = std::fs::read(&file).map_err(|_| ProjectionError::Backup)?;
            archive
                .start_file(relative, options)
                .map_err(|_| ProjectionError::Backup)?;
            std::io::Write::write_all(&mut archive, &payload)
                .map_err(|_| ProjectionError::Backup)?;
        }
        archive.finish().map_err(|_| ProjectionError::Backup)?;
    }
    let payload = zip_buffer.into_inner();
    backup_file(
        data_root,
        BackupAction::ExtensionWrite,
        target,
        &payload,
        Utc::now(),
        Some(note.to_string()),
    )
    .map_err(|_| ProjectionError::Backup)?;
    Ok(())
}

fn uninstall_skill(
    data_root: &Path,
    targets: &[ClientTarget],
    _config: &ExtensionConfig,
    name: &str,
) -> Result<(), ProjectionError> {
    let store = skill_store(data_root)?;
    let target = safe_store_path(&store, name)?;
    match std::fs::symlink_metadata(&target) {
        Ok(metadata) if metadata.is_dir() && !metadata.is_symlink() => {
            backup_skill_directory(data_root, &target, "before skill uninstall")?;
            remove_skill_links(targets, name)?;
            std::fs::remove_dir_all(&target).map_err(|_| ProjectionError::AtomicWrite)?;
            Ok(())
        }
        Ok(_) => Err(ProjectionError::TargetType),
        // 只存在于默认读源（Agent Skills 共享目录）时不删除来源——管理器不是
        // 该目录的写入者——只断开各客户端的链接。
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            remove_skill_links(targets, name)?;
            Ok(())
        }
        Err(_) => Err(ProjectionError::AtomicWrite),
    }
}

fn remove_skill_links(targets: &[ClientTarget], name: &str) -> Result<(), ProjectionError> {
    for target in targets {
        remove_manager_link(&target.skills_dir.join(name))?;
    }
    Ok(())
}

/// 只断开**管理器建立的软链接**：同名真实目录属于用户自有内容（或「文件复制」产物），
/// 一律保留不动，绝不删除客户端目录下的真实内容。
pub(crate) fn remove_manager_link(destination: &Path) -> Result<(), ProjectionError> {
    match std::fs::symlink_metadata(destination) {
        Ok(metadata) if metadata.is_symlink() => {
            std::fs::remove_file(destination).map_err(|_| ProjectionError::AtomicWrite)
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(ProjectionError::AtomicWrite),
    }
}

/// 某 Skill 当前实际落在哪些客户端（软链接或已存在的同名目录）。
///
/// 用于把「还没逐个开关过」的 Skill 显式化成精确清单时的基线：
/// 以现状为基线，用户点一个客户端只改变那一个客户端。
fn landed_clients(targets: &[ClientTarget], name: &str) -> Vec<ClientId> {
    targets
        .iter()
        .filter(|target| std::fs::symlink_metadata(target.skills_dir.join(name)).is_ok())
        .map(|target| target.client_id)
        .collect()
}

fn read_config_bounded(path: &Path) -> Result<Vec<u8>, ProjectionError> {
    let metadata =
        crate::modules::backup::safety::inspect(path).map_err(|_| ProjectionError::Corrupted)?;
    if metadata.len() > CONFIG_MAX_BYTES {
        return Err(ProjectionError::TooLarge);
    }
    crate::modules::backup::safety::read_file(path, CONFIG_MAX_BYTES)
        .map_err(|_| ProjectionError::Corrupted)
}

fn read_stored(
    path: &Path,
    targets: &[ClientTarget],
) -> Result<
    (
        ExtensionConfig,
        ProjectionFingerprint,
        Option<ProjectionConflict>,
    ),
    ProjectionError,
> {
    let bytes = read_config_bounded(path)?;
    let payload: ExtensionProjectionFile =
        serde_json::from_slice(&bytes).map_err(|_| ProjectionError::Corrupted)?;
    payload
        .fingerprint
        .validate()
        .map_err(|_| ProjectionError::Corrupted)?;

    let config = payload.config;
    let live_hash = config.fingerprint();
    let conflict = if !target_sets_match(&config, targets) {
        Some(ProjectionConflict {
            kind: "client_inconsistent",
        })
    } else if payload.fingerprint.needs_migration() {
        // 旧 schema：指纹按旧结构算，与当前结构必然不同，不能据此判外部改动。
        // 词法上仍是同一份统一配置，本次按当前结构读取，下次写入落新 schema。
        None
    } else if live_hash != payload.fingerprint.fingerprint {
        Some(ProjectionConflict {
            kind: "external_modified",
        })
    } else {
        None
    };
    Ok((config, payload.fingerprint, conflict))
}

fn save_locked(
    data_root: &Path,
    config: &mut ExtensionConfig,
    targets: &[ClientTarget],
) -> Result<ProjectionWriteResult, ProjectionError> {
    let path = config_path(data_root);
    let previous = if path.exists() {
        let bytes = read_config_bounded(&path)?;
        let payload: ExtensionProjectionFile =
            serde_json::from_slice(&bytes).map_err(|_| ProjectionError::Corrupted)?;
        payload
            .fingerprint
            .validate()
            .map_err(|_| ProjectionError::Corrupted)?;
        // 旧 schema 的指纹按旧结构计算，升级后必然不相等；按当前结构读取并迁移，
        // 不视为外部改动（否则升级后既有配置永久无法写入）。
        if !payload.fingerprint.needs_migration()
            && payload.config.fingerprint() != payload.fingerprint.fingerprint
        {
            return Err(ProjectionError::ExternalModified);
        }
        Some(bytes)
    } else {
        None
    };
    let backed_up = if let Some(previous) = previous.as_deref() {
        backup_file(
            data_root,
            BackupAction::ExtensionWrite,
            &path,
            previous,
            Utc::now(),
            Some("before extension config projection write".to_string()),
        )
        .is_ok()
    } else {
        false
    };
    if previous.is_some() && !backed_up {
        return Err(ProjectionError::Backup);
    }

    config.bump_revision();
    let fingerprint = fingerprint_for(config, targets, Utc::now());
    let payload = ExtensionProjectionFile {
        config: config.clone(),
        fingerprint: fingerprint.clone(),
    };
    let bytes = serde_json::to_vec_pretty(&payload).map_err(|_| ProjectionError::Corrupted)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| ProjectionError::AtomicWrite)?;
    }
    crate::infrastructure::atomic_write::atomic_write(&path, &bytes, 0o600)
        .map_err(|_| ProjectionError::AtomicWrite)?;
    let actual = crate::modules::backup::safety::read_file(&path, CONFIG_MAX_BYTES)
        .map_err(|_| ProjectionError::AtomicWrite)?;
    if actual != bytes {
        return Err(ProjectionError::AtomicWrite);
    }
    let hash = crate::infrastructure::hash::sha256_hex(&bytes);
    Ok(ProjectionWriteResult {
        config: config.clone(),
        fingerprint,
        document_sha256: hash,
        backed_up,
        skills_linked: Vec::new(),
        mcp_written: Vec::new(),
    })
}

fn validate_targets(targets: &[ClientTarget], home: &Path) -> Result<(), ProjectionError> {
    if targets.is_empty() || !home.is_absolute() || !home.is_dir() {
        return Err(ProjectionError::TargetInvalid);
    }
    let mut seen = BTreeMap::new();
    for target in targets {
        if !target.installed || !target.mcp_config_path.is_absolute() {
            return Err(ProjectionError::TargetInvalid);
        }
        let context = target
            .mcp_config_path
            .parent()
            .ok_or(ProjectionError::TargetInvalid)?;
        // FZ-25/27：home 是显式输入。这里校验 target 未越出该输入；
        // 不要求每个客户端目录已存在，未安装由调用方冻结目标矩阵决定。
        let within = |path: &Path| {
            path.strip_prefix(home).is_ok()
                && path
                    .components()
                    .all(|component| !matches!(component, std::path::Component::ParentDir))
        };
        if !within(&target.skills_dir) || !within(context) {
            eprintln!(
                "escape: home={home:?} skills={:?} context={context:?}",
                target.skills_dir
            );
            return Err(ProjectionError::PathEscape);
        }
        if seen.insert(target.client_id, ()).is_some() {
            return Err(ProjectionError::TargetInvalid);
        }
    }
    Ok(())
}

fn home_from_targets(targets: &[ClientTarget]) -> Result<&Path, ProjectionError> {
    let codex = targets
        .iter()
        .find(|target| target.client_id == ClientId::Codex)
        .ok_or(ProjectionError::TargetInvalid)?;
    codex
        .skills_dir
        .parent()
        .and_then(Path::parent)
        .ok_or(ProjectionError::TargetInvalid)
}

fn target_sets_match(config: &ExtensionConfig, _targets: &[ClientTarget]) -> bool {
    let expected = CLIENT_IDS
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let actual = config
        .enablement
        .keys()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    expected == actual
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct ExtensionProjection {
    pub config: crate::modules::extensions::ExtensionConfig,
    pub fingerprint: ProjectionFingerprint,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
struct ExtensionProjectionFile {
    config: ExtensionConfig,
    fingerprint: ProjectionFingerprint,
}

fn fingerprint_for(
    config: &ExtensionConfig,
    _targets: &[ClientTarget],
    now: chrono::DateTime<Utc>,
) -> ProjectionFingerprint {
    // 冻结矩阵参与指纹；targets 只用于保证落点上下文完整，不写入路径。
    ProjectionFingerprint::create(config.fingerprint(), now)
}
