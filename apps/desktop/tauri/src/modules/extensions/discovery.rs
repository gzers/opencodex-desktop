//! MOD-09 扩展只读发现。
//!
//! 只读取显式 HOME 下的 Agent Skills 共享目录（默认读源）、显式数据根中的
//! 管理器写入区（导入 / 恢复产物），以及 FZ-22 冻结的六个客户端 MCP 配置文件。
//! 不写入、不跟随任意路径、不访问网络或 Keychain。

use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{ClientId, ClientTarget, ConfigFormat, CLIENT_IDS};

/// 默认读源：Agent Skills 共享目录（相对用户主目录）。
/// Windows 上同式解析为 `%USERPROFILE%\.agents\skills`。
const AGENTS_SKILLS_RELATIVE_PATH: &str = ".agents/skills";
/// 管理器写入区：ZIP 导入 / 恢复 / 备份 / 导出的产物落在数据根内（相对数据根）。
const SKILLS_STORE_RELATIVE_PATH: &str = "manager-state/skills-store";
pub const SKILL_MANIFEST_FILE: &str = "SKILL.md";
const SKILL_MANIFEST_MAX_BYTES: u64 = 256 * 1024;
pub(crate) const MCP_CONFIG_MAX_BYTES: u64 = 16 * 1024 * 1024;

/// 一个 Skill 是从哪个位置发现的。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SkillSourceKind {
    /// 默认读源：`<主目录>/.agents/skills`（Agent Skills 共享目录）。
    Agents,
    /// 管理器写入区：`<数据根>/manager-state/skills-store`（导入 / 恢复产物）。
    Store,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredSkill {
    pub id: String,
    pub name: String,
    pub source: SkillSourceKind,
    pub updated_at: Option<String>,
    pub description: String,
    pub clients: Vec<ClientId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredServer {
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExtensionDiscovery {
    pub skills: Vec<DiscoveredSkill>,
    pub servers: Vec<DiscoveredServer>,
}

/// 默认读源相对用户主目录的路径（`.agents/skills`）。
pub fn agents_skills_relative_path() -> &'static str {
    AGENTS_SKILLS_RELATIVE_PATH
}

/// 默认读源绝对路径：`<主目录>/.agents/skills`。
pub fn agents_skills_dir(home: &Path) -> std::path::PathBuf {
    home.join(AGENTS_SKILLS_RELATIVE_PATH)
}

/// 管理器写入区相对数据根的路径（`manager-state/skills-store`）。
pub fn skills_store_relative_path() -> &'static str {
    SKILLS_STORE_RELATIVE_PATH
}

/// 管理器写入区绝对路径：`<数据根>/manager-state/skills-store`。
pub fn skills_store_dir(data_root: &Path) -> std::path::PathBuf {
    data_root.join(SKILLS_STORE_RELATIVE_PATH)
}

/// 原始 MCP 定义；value 保留目标配置中的业务字段，用于跨客户端机械改写。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RawServerDefinition {
    pub name: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryError {
    NotConfigured,
    FileSystem,
}

/// Read one client MCP configuration through the shared bounded, no-follow
/// reader. A missing configuration is an empty client state; every other
/// filesystem/type/size failure is surfaced to the caller.
pub(crate) fn read_mcp_config(path: &Path) -> Result<Option<Vec<u8>>, DiscoveryError> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(DiscoveryError::FileSystem),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(DiscoveryError::FileSystem);
    }
    if metadata.len() > MCP_CONFIG_MAX_BYTES {
        return Err(DiscoveryError::FileSystem);
    }
    crate::modules::backup::safety::read_file(path, MCP_CONFIG_MAX_BYTES)
        .map(Some)
        .map_err(|_| DiscoveryError::FileSystem)
}

/// 发现扩展。`source_dir` 由调用方解析（默认源目录或用户自定义目录），
/// 本函数不再自己推导——源目录是配置值，不是常量。
pub fn discover(
    data_root: &Path,
    home: &Path,
    source_dir: &Path,
    targets: &[ClientTarget],
) -> Result<ExtensionDiscovery, DiscoveryError> {
    if data_root.as_os_str().is_empty() || home.as_os_str().is_empty() {
        return Err(DiscoveryError::NotConfigured);
    }
    if targets.len() != CLIENT_IDS.len() {
        return Err(DiscoveryError::NotConfigured);
    }

    // 源目录优先，管理器写入区作为补充（导入 / 恢复的 Skill 只存在于数据根内）。
    let store_dir = skills_store_dir(data_root);
    let skills = discover_skills(source_dir, &store_dir, targets)?;
    let servers = discover_servers(targets)?;
    Ok(ExtensionDiscovery { skills, servers })
}

fn discover_skills(
    source_dir: &Path,
    store_dir: &Path,
    targets: &[ClientTarget],
) -> Result<Vec<DiscoveredSkill>, DiscoveryError> {
    let mut skills = BTreeMap::new();
    // 先读默认读源，再补管理器写入区；同名时读源胜出。
    for (source, dir) in [
        (SkillSourceKind::Agents, source_dir),
        (SkillSourceKind::Store, store_dir),
    ] {
        for (name, path) in read_skill_dir(dir)? {
            let entry = skills
                .entry(name.clone())
                .or_insert_with(|| DiscoveredSkill {
                    id: name.clone(),
                    name,
                    source,
                    updated_at: modified_at(&path),
                    description: read_skill_description(&path).unwrap_or_default(),
                    clients: Vec::new(),
                });
            if entry.source == SkillSourceKind::Store && source == SkillSourceKind::Agents {
                entry.source = SkillSourceKind::Agents;
                entry.updated_at = modified_at(&path);
                entry.description = read_skill_description(&path).unwrap_or_default();
            }
        }
    }

    for skill in skills.values_mut() {
        for target in targets {
            if skill_target_exists(target, &skill.name) {
                skill.clients.push(target.client_id);
            }
        }
    }

    Ok(skills.into_values().collect())
}

fn read_skill_dir(dir: &Path) -> Result<Vec<(String, std::path::PathBuf)>, DiscoveryError> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(_) => return Err(DiscoveryError::FileSystem),
    };
    let mut result = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|_| DiscoveryError::FileSystem)?;
        let path = entry.path();
        let metadata = std::fs::symlink_metadata(&path).map_err(|_| DiscoveryError::FileSystem)?;
        if !metadata.is_dir() && !metadata.is_symlink() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(DiscoveryError::FileSystem)?
            .to_string();
        result.push((name, path));
    }
    Ok(result)
}

fn read_skill_description(skill_dir: &Path) -> Option<String> {
    let path = skill_dir.join(SKILL_MANIFEST_FILE);
    let metadata = std::fs::metadata(&path).ok()?;
    if !metadata.is_file() || metadata.len() > SKILL_MANIFEST_MAX_BYTES {
        return None;
    }
    let content = std::fs::read_to_string(&path).ok()?;
    parse_skill_description(&content)
}

fn parse_skill_description(content: &str) -> Option<String> {
    let mut lines = content.lines();
    if lines.next()? != "---" {
        return None;
    }
    let mut description_lines: Vec<String> = Vec::new();
    for line in lines {
        if line == "---" || line == "..." {
            break;
        }
        if description_lines.is_empty() {
            if let Some(value) = line.strip_prefix("description:") {
                let value = value.trim();
                if !value.is_empty() {
                    description_lines.push(value.to_string());
                }
            }
            continue;
        }
        if let Some(value) = line.strip_prefix("- ") {
            description_lines.push(value.trim().to_string());
        } else if line.starts_with(' ') {
            description_lines.push(line.trim().to_string());
        } else if !line.trim().is_empty() {
            break;
        }
    }
    let description = description_lines.join(" ");
    if description.is_empty() {
        None
    } else {
        Some(description)
    }
}

fn skill_target_exists(target: &ClientTarget, skill_name: &str) -> bool {
    std::fs::symlink_metadata(target.skills_dir.join(skill_name)).is_ok()
}

fn modified_at(path: &Path) -> Option<String> {
    let modified = std::fs::symlink_metadata(path).ok()?.modified().ok()?;
    let time = chrono::DateTime::<chrono::Utc>::from(modified);
    Some(time.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
}

/// 单个 Skill 的只读详情：正文按需读取，目录统计带上限（列表不预读）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillDetail {
    pub name: String,
    pub source: SkillSourceKind,
    pub source_path: std::path::PathBuf,
    pub entry_file: String,
    pub body: Option<String>,
    pub body_truncated: bool,
    pub file_count: Option<u64>,
    pub total_bytes: Option<u64>,
    /// 统计不可用时给稳定原因码（`stats_unavailable` / `stats_limit_exceeded`），不伪造数字。
    pub stats_note: Option<String>,
}

/// 目录统计上限：条目数超过即如实报「未知」，避免在超大目录上长时间遍历。
const SKILL_STATS_MAX_FILES: u64 = 2_000;

/// 去掉正文开头的 YAML frontmatter。
///
/// frontmatter 里的 `name` / `description` 已由列表与详情信息区展示；若原样进入 Markdown 渲染，
/// 详情里会出现一段与描述重复的元数据文本（`---` / `name:` / `description:`）。
/// 只在开头确实是完整 frontmatter 块时才剥离，否则按原样返回，避免误删正文。
fn strip_frontmatter(content: &str) -> &str {
    let Some(after) = content.strip_prefix("---") else {
        return content;
    };
    if !(after.starts_with('\n') || after.starts_with("\r\n")) {
        return content;
    }
    let body_start = content.len() - after.len();
    let mut cursor = body_start;
    for line in content[body_start..].split_inclusive('\n') {
        cursor += line.len();
        let marker = line.trim_end_matches(['\r', '\n']).trim();
        if marker == "---" || marker == "..." {
            return content[cursor..].trim_start_matches(['\r', '\n']);
        }
    }
    content
}

/// Skill 名必须是单一路径段：详情入口来自界面，但仍当不可信输入处理，避免路径穿越。
pub fn is_safe_skill_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains('/')
        && !name.contains('\\')
        && Path::new(name).components().count() == 1
}

/// 读取一条 Skill 的详情；不存在或名字不合法时返回 `None`。
/// 源目录优先，管理器写入区其次——与发现顺序一致，保证详情和列表指向同一条。
pub fn read_skill_detail(
    source_dir: &Path,
    store_dir: &Path,
    name: &str,
) -> Result<Option<SkillDetail>, DiscoveryError> {
    if !is_safe_skill_name(name) {
        return Ok(None);
    }
    let located = read_skill_dir(source_dir)?
        .into_iter()
        .find(|(candidate, _)| candidate == name)
        .map(|(_, path)| (SkillSourceKind::Agents, path))
        .or_else(|| {
            read_skill_dir(store_dir)
                .ok()
                .and_then(|entries| entries.into_iter().find(|(candidate, _)| candidate == name))
                .map(|(_, path)| (SkillSourceKind::Store, path))
        });
    let Some((source, path)) = located else {
        return Ok(None);
    };

    // 正文只读 `SKILL.md`，超过上限则截断并标记，绝不整文件读入无界内容。
    let mut body = None;
    let mut body_truncated = false;
    if let Ok(metadata) = std::fs::metadata(path.join(SKILL_MANIFEST_FILE)) {
        if metadata.is_file() {
            let payload = std::fs::read(path.join(SKILL_MANIFEST_FILE))
                .map_err(|_| DiscoveryError::FileSystem)?;
            let limit = SKILL_MANIFEST_MAX_BYTES as usize;
            let truncated = payload.len() > limit;
            let slice = if truncated {
                &payload[..limit]
            } else {
                &payload[..]
            };
            let decoded = String::from_utf8_lossy(slice);
            body = Some(strip_frontmatter(&decoded).to_string());
            body_truncated = truncated;
        }
    }

    let (file_count, total_bytes, stats_note) = skill_directory_stats(&path);
    Ok(Some(SkillDetail {
        name: name.to_string(),
        source,
        source_path: path,
        entry_file: SKILL_MANIFEST_FILE.to_string(),
        body,
        body_truncated,
        file_count,
        total_bytes,
        stats_note,
    }))
}

/// 统计源目录下的文件数与总体量。不跟随符号链接，避免目录环与越出源目录。
fn skill_directory_stats(root: &Path) -> (Option<u64>, Option<u64>, Option<String>) {
    let mut files = 0u64;
    let mut bytes = 0u64;
    let mut stack = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let entries = match std::fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(_) => return (None, None, Some("stats_unavailable".to_string())),
        };
        for entry in entries {
            let Ok(entry) = entry else {
                return (None, None, Some("stats_unavailable".to_string()));
            };
            let path = entry.path();
            let Ok(metadata) = std::fs::symlink_metadata(&path) else {
                return (None, None, Some("stats_unavailable".to_string()));
            };
            if metadata.is_dir() {
                stack.push(path);
                continue;
            }
            if !metadata.is_file() {
                continue;
            }
            files += 1;
            bytes += metadata.len();
            if files > SKILL_STATS_MAX_FILES {
                return (None, None, Some("stats_limit_exceeded".to_string()));
            }
        }
    }
    (Some(files), Some(bytes), None)
}

fn discover_servers(targets: &[ClientTarget]) -> Result<Vec<DiscoveredServer>, DiscoveryError> {
    let mut servers = BTreeMap::new();
    for target in targets {
        let Some(payload) = read_mcp_config(&target.mcp_config_path)? else {
            continue;
        };
        let updated_at = modified_at(&target.mcp_config_path);
        let text = String::from_utf8(payload).map_err(|_| DiscoveryError::FileSystem)?;
        let root = parse_config(&text, target.format)?;
        for entry in extract_server_entries(&root, &target.mcp_key) {
            let server = servers
                .entry(entry.name.clone())
                .or_insert_with(|| DiscoveredServer {
                    id: entry.name.clone(),
                    name: entry.name.clone(),
                    transport: entry.transport,
                    command: entry.command,
                    args: entry.args,
                    env_keys: entry.env_keys,
                    description: entry.description,
                    clients: Vec::new(),
                    updated_at: updated_at.clone(),
                });
            if !server.clients.contains(&target.client_id) {
                server.clients.push(target.client_id);
            }
        }
    }
    Ok(servers.into_values().collect())
}

pub fn read_server_definition(
    target: &ClientTarget,
    name: &str,
) -> Result<Option<RawServerDefinition>, DiscoveryError> {
    let Some(payload) = read_mcp_config(&target.mcp_config_path)? else {
        return Ok(None);
    };
    let text = String::from_utf8(payload).map_err(|_| DiscoveryError::FileSystem)?;
    let root = parse_config(&text, target.format)?;
    let container = root.get(&target.mcp_key);
    let value = match container {
        Some(serde_json::Value::Object(map)) => map.get(name).cloned(),
        Some(serde_json::Value::Array(items)) => items
            .iter()
            .find(|item| {
                item.get("name")
                    .or_else(|| item.get("id"))
                    .or_else(|| item.get("server_name"))
                    .and_then(serde_json::Value::as_str)
                    == Some(name)
            })
            .cloned(),
        _ => None,
    };
    Ok(value.map(|value| RawServerDefinition {
        name: name.to_string(),
        value,
    }))
}

fn parse_config(text: &str, format: ConfigFormat) -> Result<serde_json::Value, DiscoveryError> {
    let value = match format {
        ConfigFormat::Toml => {
            let parsed: toml::Value =
                toml::from_str(text).map_err(|_| DiscoveryError::FileSystem)?;
            serde_json::to_value(parsed).map_err(|_| DiscoveryError::FileSystem)?
        }
        ConfigFormat::Yaml => {
            let parsed: serde_yaml::Value =
                serde_yaml::from_str(text).map_err(|_| DiscoveryError::FileSystem)?;
            serde_json::to_value(parsed).map_err(|_| DiscoveryError::FileSystem)?
        }
        ConfigFormat::Json | ConfigFormat::Jsonc => {
            let cleaned = strip_json_comments(text);
            serde_json::from_str(&cleaned).map_err(|_| DiscoveryError::FileSystem)?
        }
    };
    Ok(value)
}

pub fn strip_json_comments(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_string = false;
    let mut escaped = false;
    let mut chars = input.chars().peekable();
    while let Some(current) = chars.next() {
        if in_string {
            output.push(current);
            if escaped {
                escaped = false;
            } else if current == '\\' {
                escaped = true;
            } else if current == '"' {
                in_string = false;
            }
            continue;
        }
        if current == '"' {
            in_string = true;
            output.push(current);
            continue;
        }
        if current == '/' {
            match chars.peek() {
                Some('/') => {
                    for skipped in chars.by_ref() {
                        if skipped == '\n' {
                            output.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
                    chars.next();
                    let mut previous = ' ';
                    for skipped in chars.by_ref() {
                        if previous == '*' && skipped == '/' {
                            break;
                        }
                        previous = skipped;
                    }
                    continue;
                }
                _ => {}
            }
        }
        output.push(current);
    }
    output
}

struct RawServerEntry {
    name: String,
    transport: String,
    command: Option<String>,
    args: Vec<String>,
    env_keys: Vec<String>,
    description: String,
}

fn extract_server_entries(root: &serde_json::Value, mcp_key: &str) -> Vec<RawServerEntry> {
    let Some(container) = root.get(mcp_key) else {
        return Vec::new();
    };
    match container {
        serde_json::Value::Object(map) => map
            .iter()
            .filter_map(|(name, value)| server_entry(name, value))
            .collect(),
        serde_json::Value::Array(items) => items
            .iter()
            .filter_map(|value| {
                let name = value
                    .get("name")
                    .or_else(|| value.get("id"))
                    .or_else(|| value.get("server_name"))
                    .and_then(serde_json::Value::as_str)?;
                server_entry(name, value)
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn server_entry(name: &str, value: &serde_json::Value) -> Option<RawServerEntry> {
    if name.trim().is_empty() {
        return None;
    }
    let transport = value
        .get("type")
        .or_else(|| value.get("transport"))
        .or_else(|| value.get("transport_type"))
        .and_then(serde_json::Value::as_str)
        .map(|value| match value.trim().to_ascii_lowercase().as_str() {
            "local" => "stdio".to_string(),
            other => other.to_string(),
        })
        .or_else(|| {
            if value.get("url").is_some() {
                Some("remote".to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "stdio".to_string());
    let command = value
        .get("command")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let args: Vec<String> = value
        .get("args")
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    let env_keys = ["env", "environment"]
        .iter()
        .find_map(|key| value.get(*key).and_then(serde_json::Value::as_object))
        .map(|map| map.keys().cloned().collect())
        .unwrap_or_default();
    let description = value
        .get("description")
        .or_else(|| value.get("note"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| {
            let mut parts = Vec::new();
            if let Some(command) = &command {
                parts.push(command.clone());
            }
            if !args.is_empty() {
                parts.push(args.join(" "));
            }
            if parts.is_empty() {
                transport.clone()
            } else {
                parts.join(" ")
            }
        });
    Some(RawServerEntry {
        name: name.to_string(),
        transport,
        command,
        args,
        env_keys,
        description: sanitize_description(&description),
    })
}

fn sanitize_description(input: &str) -> String {
    let mut output = input.trim().to_string();
    if let Some(scheme_index) = output.find("://") {
        let authority_start = scheme_index + 3;
        let host_start = output[authority_start..]
            .find('@')
            .map(|at_index| authority_start + at_index + 1)
            .unwrap_or(authority_start);
        if let Some(path_index) = output[host_start..].find('/') {
            let path_index = host_start + path_index;
            if let Some(query_index) = output[path_index..].find('?') {
                output.truncate(path_index + query_index);
            }
        }
        output.replace_range(authority_start..host_start, "");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_jsonc_comments_and_extracts_server_entries() {
        let payload = r#"{
            // comment
            "mcp_servers": {
                "node_repl": {
                    "type": "stdio",
                    "command": "node",
                    "args": ["--foo"],
                    "env": { "TOKEN": "x" }
                }
            }
        }"#;
        let value = parse_config(payload, ConfigFormat::Jsonc).expect("parse jsonc");
        let entries = extract_server_entries(&value, "mcp_servers");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "node_repl");
        assert_eq!(entries[0].command.as_deref(), Some("node"));
        assert_eq!(entries[0].args, vec!["--foo"]);
        assert_eq!(entries[0].env_keys, vec!["TOKEN"]);
    }

    #[test]
    fn sanitize_description_removes_query_and_userinfo() {
        assert_eq!(
            sanitize_description("https://user:token@example.com/v1?api_key=secret"),
            "https://example.com/v1"
        );
    }
}
