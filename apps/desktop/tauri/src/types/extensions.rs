//! 扩展管理前端 DTO；只传输脱敏后的配置与发现结果。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::modules::extensions::discovery::{
    DiscoveredServer, DiscoveredSkill, ExtensionDiscovery, SkillDetail, SkillSourceKind,
};
use crate::modules::extensions::ClientId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionSkillDto {
    pub id: String,
    pub name: String,
    pub source: SkillSourceKind,
    pub updated_at: Option<String>,
    pub description: String,
    pub clients: Vec<ClientId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionServerDto {
    pub id: String,
    pub name: String,
    pub transport: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env_keys: Vec<String>,
    pub description: String,
    pub clients: Vec<ClientId>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionConfigDto {
    pub skills: Vec<String>,
    pub servers: Vec<String>,
    pub enablement: BTreeMap<ClientId, bool>,
    /// FZ-23 配置的自定义源目录；`null` = 使用默认目录。
    pub source_dir: Option<String>,
    /// 是否使用自定义源目录（便于前端直接渲染「默认 / 自定义」）。
    pub source_dir_custom: bool,
    /// FZ-24 分发方式（`symlink` / `copy`）。
    pub sync_method: String,
    pub revision: u64,
    pub fingerprint: String,
    pub updated_at: String,
    pub conflict: Option<String>,
    pub document_sha256: String,
    pub backed_up: bool,
}

/// FZ-23 / AC-11：单个 Skill 的只读详情（描述与 `SKILL.md` 正文按需读取，掩码规则不变）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDetailDto {
    pub name: String,
    pub source: SkillSourceKind,
    pub source_path: String,
    pub entry_file: String,
    pub body: Option<String>,
    pub body_truncated: bool,
    pub file_count: Option<u64>,
    pub total_bytes: Option<u64>,
    pub stats_note: Option<String>,
}

impl From<SkillDetail> for SkillDetailDto {
    fn from(value: SkillDetail) -> Self {
        Self {
            name: value.name,
            source: value.source,
            source_path: value.source_path.display().to_string(),
            entry_file: value.entry_file,
            body: value.body,
            body_truncated: value.body_truncated,
            file_count: value.file_count,
            total_bytes: value.total_bytes,
            stats_note: value.stats_note,
        }
    }
}

/// 一个客户端上的 MCP 配置落点。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpLandingDto {
    pub client: ClientId,
    pub config_path: String,
    pub key: String,
    pub present: bool,
}

/// FZ-46 / AC-11：MCP 条目的只读详情（命令、参数、配置落点；`env` 只给键名，`args` 掩码）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpDetailDto {
    pub name: String,
    pub transport: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env_keys: Vec<String>,
    pub description: String,
    pub clients: Vec<ClientId>,
    pub updated_at: Option<String>,
    /// 统一 MCP 定义的 JSON（敏感参数已掩码）；取不到时为 `null`。
    pub config_json: Option<String>,
    pub landings: Vec<McpLandingDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionWriteResultDto {
    pub config: ExtensionConfigDto,
    pub skills_linked: Vec<String>,
    pub mcp_written: Vec<String>,
}

impl ExtensionWriteResultDto {
    pub fn from_projection(
        value: crate::modules::extensions::projection::ProjectionWriteResult,
        conflict: Option<&str>,
    ) -> Self {
        let crate::modules::extensions::projection::ProjectionWriteResult {
            config,
            fingerprint,
            document_sha256,
            backed_up,
            skills_linked,
            mcp_written,
        } = value;
        // 先取派生值，再逐字段移动——`sync_method_normalized` 需要借用 config。
        let source_dir = config.source_store.clone();
        let sync_method = config.sync_method_normalized().to_string();
        Self {
            skills_linked,
            mcp_written,
            config: ExtensionConfigDto {
                skills: config.skills,
                servers: config.servers,
                enablement: config.enablement,
                source_dir_custom: source_dir.is_some(),
                source_dir,
                sync_method,
                revision: config.revision,
                fingerprint: fingerprint.fingerprint,
                updated_at: fingerprint.updated_at,
                conflict: conflict.map(str::to_string),
                document_sha256,
                backed_up,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionsDto {
    pub skills: Vec<ExtensionSkillDto>,
    pub servers: Vec<ExtensionServerDto>,
    /// FZ-23 本次发现实际使用的源目录（默认目录或自定义目录的解析结果）。
    pub source_dir: String,
    /// 自定义源目录被忽略时给前端的稳定原因码；正常为 `None`。
    pub source_dir_notice: Option<String>,
    /// 源目录是否为用户自定义（false = 使用默认目录）。
    pub source_dir_custom: bool,
    /// FZ-24 当前分发方式。
    pub sync_method: String,
}

impl ExtensionConfigDto {
    pub fn from_projection(
        value: crate::modules::extensions::projection::ProjectionWriteResult,
        conflict: Option<&str>,
    ) -> Self {
        let source_dir = value.config.source_store.clone();
        let sync_method = value.config.sync_method_normalized().to_string();
        Self {
            skills: value.config.skills,
            servers: value.config.servers,
            enablement: value.config.enablement,
            source_dir_custom: source_dir.is_some(),
            source_dir,
            sync_method,
            revision: value.config.revision,
            fingerprint: value.fingerprint.fingerprint,
            updated_at: value.fingerprint.updated_at,
            conflict: conflict.map(str::to_string),
            document_sha256: value.document_sha256,
            backed_up: value.backed_up,
        }
    }
}

impl From<ExtensionDiscovery> for ExtensionsDto {
    fn from(value: ExtensionDiscovery) -> Self {
        Self {
            skills: value.skills.into_iter().map(Into::into).collect(),
            servers: value.servers.into_iter().map(Into::into).collect(),
            // 由命令层填充：发现本身不关心源目录与分发方式的展示字段。
            source_dir: String::new(),
            source_dir_notice: None,
            source_dir_custom: false,
            sync_method: crate::modules::extensions::SYNC_METHOD_SYMLINK.to_string(),
        }
    }
}

impl From<DiscoveredSkill> for ExtensionSkillDto {
    fn from(value: DiscoveredSkill) -> Self {
        Self {
            id: value.id,
            name: value.name,
            source: value.source,
            updated_at: value.updated_at,
            description: value.description,
            clients: value.clients,
        }
    }
}

impl From<DiscoveredServer> for ExtensionServerDto {
    fn from(value: DiscoveredServer) -> Self {
        Self {
            id: value.id,
            name: value.name,
            transport: value.transport,
            command: value.command,
            args: value.args,
            env_keys: value.env_keys,
            description: value.description,
            clients: value.clients,
            updated_at: value.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::extensions::discovery::{DiscoveredServer, DiscoveredSkill};
    use crate::modules::extensions::ClientId;
    use serde_json::json;

    #[test]
    fn extension_config_dto_uses_camel_case_and_no_paths() {
        let config = ExtensionConfigDto {
            skills: vec!["sample".to_string()],
            servers: vec!["node_repl".to_string()],
            enablement: [(ClientId::Codex, true)].into_iter().collect(),
            source_dir: None,
            source_dir_custom: false,
            sync_method: "symlink".to_string(),
            revision: 2,
            fingerprint: "a".repeat(64),
            updated_at: "2026-09-16T00:00:00Z".to_string(),
            conflict: None,
            document_sha256: "b".repeat(64),
            backed_up: false,
        };
        let payload = serde_json::to_value(&config).expect("serialize config");
        assert_eq!(payload["updatedAt"], "2026-09-16T00:00:00Z");
        assert_eq!(payload["documentSha256"], "b".repeat(64));
        assert_eq!(payload["enablement"]["codex"], true);
        assert_eq!(payload["sourceDir"], serde_json::Value::Null);
        assert_eq!(payload["sourceDirCustom"], false);
        assert_eq!(payload["syncMethod"], "symlink");
        assert!(!serde_json::to_string(&config)
            .expect("serialize")
            .contains("64"));
    }

    #[test]
    fn extensions_dto_uses_camel_case() {
        let dto = ExtensionsDto {
            skills: vec![ExtensionSkillDto {
                id: "sample".to_string(),
                name: "sample".to_string(),
                source: SkillSourceKind::Agents,
                updated_at: Some("2026-09-15T00:00:00Z".to_string()),
                description: "desc".to_string(),
                clients: vec![ClientId::Codex],
            }],
            servers: vec![ExtensionServerDto {
                id: "node_repl".to_string(),
                name: "node_repl".to_string(),
                transport: "stdio".to_string(),
                command: Some("node".to_string()),
                args: vec!["--foo".to_string()],
                env_keys: vec!["TOKEN".to_string()],
                description: "node --foo".to_string(),
                clients: vec![ClientId::Codex],
                updated_at: None,
            }],
            source_dir: "/Users/example/.agents/skills".to_string(),
            source_dir_notice: None,
            source_dir_custom: false,
            sync_method: "symlink".to_string(),
        };
        let payload = serde_json::to_value(&dto).expect("serialize");
        assert_eq!(payload["skills"][0]["updatedAt"], "2026-09-15T00:00:00Z");
        assert_eq!(payload["skills"][0]["clients"][0], "codex");
        assert_eq!(payload["servers"][0]["envKeys"][0], "TOKEN");
        assert_eq!(payload["sourceDir"], "/Users/example/.agents/skills");
        assert_eq!(payload["sourceDirCustom"], false);
        assert_eq!(payload["syncMethod"], "symlink");
    }

    #[test]
    fn discovers_from_module_shapes() {
        let discovery = ExtensionDiscovery {
            skills: vec![DiscoveredSkill {
                id: "sample".to_string(),
                name: "sample".to_string(),
                source: SkillSourceKind::Store,
                updated_at: None,
                description: String::new(),
                clients: vec![ClientId::Claude],
            }],
            servers: vec![DiscoveredServer {
                id: "node_repl".to_string(),
                name: "node_repl".to_string(),
                transport: "stdio".to_string(),
                command: None,
                args: Vec::new(),
                env_keys: Vec::new(),
                description: "stdio".to_string(),
                clients: vec![ClientId::Codex],
                updated_at: None,
            }],
        };
        let dto: ExtensionsDto = discovery.into();
        assert_eq!(dto.skills[0].source, SkillSourceKind::Store);
        assert_eq!(dto.servers[0].clients, vec![ClientId::Codex]);
        // 源目录与分发方式的展示字段由命令层填充，`From` 只保证结构完整。
        assert_eq!(dto.source_dir, "");
        assert_eq!(dto.sync_method, "symlink");
        assert!(!dto.source_dir_custom);
        assert_eq!(
            serde_json::to_value(&dto).unwrap(),
            json!({
                "skills": [{
                    "id": "sample",
                    "name": "sample",
                    "source": "store",
                    "updatedAt": null,
                    "description": "",
                    "clients": ["claude"]
                }],
                "servers": [{
                    "id": "node_repl",
                    "name": "node_repl",
                    "transport": "stdio",
                    "command": null,
                    "args": [],
                    "envKeys": [],
                    "description": "stdio",
                    "clients": ["codex"],
                    "updatedAt": null
                }],
                "sourceDir": "",
                "sourceDirNotice": null,
                "sourceDirCustom": false,
                "syncMethod": "symlink"
            })
        );
    }
}
