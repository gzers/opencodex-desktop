//! Explicit extension assets. A received Skill replaces the directory that
//! discovery uses for that name (the read source when present, otherwise the
//! manager skill store) after a backup (S18-04); MCPs live in a manager-owned
//! definition source. A definition does not execute or project into a client
//! merely by importing it.
use crate::{
    infrastructure::atomic_write::atomic_write,
    modules::{
        backup::safety,
        extensions::{
            self, discovery,
            projection::{self, ExtensionProjection},
            ClientTarget, CLIENT_IDS,
        },
        model_config::{schema::*, store::sha256},
    },
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
use zeroize::Zeroize;
const MCP_STORE: &str = "manager-state/mcp-definitions.json";
const FILE_LIMIT: u64 = 8 * 1024 * 1024;
const TOTAL_LIMIT: usize = 24 * 1024 * 1024;
/// MCPs maintained by a client application itself (local paths, app versions).
/// "All" scopes skip them; an explicit selection still includes them (S18-03).
pub const DEFAULT_EXCLUDED_MCPS: [&str; 1] = ["node_repl"];
pub fn default_included_mcp(name: &str) -> bool {
    !DEFAULT_EXCLUDED_MCPS.contains(&name)
}
/// Same-named MCP whose client definitions differ and no source was chosen.
/// Only client ids and differing top-level field names, never values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpConflict {
    pub name: String,
    pub clients: Vec<String>,
    pub fields: Vec<String>,
}
pub fn client_key(client: extensions::ClientId) -> String {
    serde_json::to_value(client)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_default()
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillFile {
    pub path: String,
    pub digest: String,
    pub data_b64: String,
    pub executable: bool,
    #[serde(default)]
    pub sensitive: bool,
}
impl Drop for SkillFile {
    fn drop(&mut self) {
        self.data_b64.zeroize();
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Skill {
    pub name: String,
    pub digest: String,
    pub files: Vec<SkillFile>,
    pub contains_secrets: bool,
    pub warnings: Vec<String>,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Secrets(pub BTreeMap<String, String>);
impl Drop for Secrets {
    fn drop(&mut self) {
        for value in self.0.values_mut() {
            value.zeroize();
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Mcp {
    pub name: String,
    pub definition: Value,
    pub secret_fields: Vec<String>,
    pub secrets: Secrets,
    pub warnings: Vec<String>,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Assets {
    pub skills: Vec<Skill>,
    pub mcps: Vec<Mcp>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NameMappings {
    #[serde(default)]
    pub skills: BTreeMap<String, String>,
    #[serde(default)]
    pub mcps: BTreeMap<String, String>,
}
impl NameMappings {
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty() && self.mcps.is_empty()
    }
}
pub fn remap(assets: &Assets, mappings: &NameMappings) -> ModelResult<Assets> {
    for (map, available) in [
        (
            &mappings.skills,
            assets
                .skills
                .iter()
                .map(|s| s.name.as_str())
                .collect::<BTreeSet<_>>(),
        ),
        (
            &mappings.mcps,
            assets.mcps.iter().map(|m| m.name.as_str()).collect(),
        ),
    ] {
        if map.len() > 1000 {
            return Err(invalid("asset_mapping", "扩展映射超过数量上限"));
        }
        for (source, target) in map {
            if !available.contains(source.as_str()) {
                return Err(invalid("asset_mapping", "仅允许映射已选扩展"));
            }
            name(source)?;
            relative(target)?;
            if target.contains('/') {
                return Err(invalid("asset_mapping", "扩展目标名称不能是路径"));
            }
        }
    }
    let mut result = assets.clone();
    for skill in &mut result.skills {
        if let Some(target) = mappings.skills.get(&skill.name) {
            skill.name = target.clone();
        }
    }
    for mcp in &mut result.mcps {
        if let Some(target) = mappings.mcps.get(&mcp.name) {
            mcp.name = target.clone();
        }
    }
    validate(&result)?;
    Ok(result)
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Summary {
    pub skills: Vec<SkillSummary>,
    pub mcps: Vec<McpSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conflicts: Vec<McpConflict>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillSummary {
    pub name: String,
    pub digest: String,
    pub files: usize,
    pub bytes: usize,
    pub warnings: Vec<String>,
    pub protected: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpSummary {
    pub name: String,
    pub digest: String,
    pub secret_fields: Vec<String>,
    pub warnings: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredMcp {
    name: String,
    definition: Value,
    secret_fields: Vec<String>,
    references: BTreeMap<String, String>,
}
fn name(value: &str) -> ModelResult<()> {
    if !discovery::is_safe_skill_name(value)
        || value.len() > 256
        || value
            .chars()
            .any(|c| c.is_control() || "<>:\"|?*".contains(c))
        || value.ends_with([' ', '.'])
    {
        return Err(invalid("asset_name", "扩展名称不能安全迁移到目标平台"));
    }
    Ok(())
}
fn relative(value: &str) -> ModelResult<PathBuf> {
    if value.is_empty()
        || value.starts_with('/')
        || value.contains('\\')
        || value.contains('\0')
        || value.split('/').count() > 64
    {
        return Err(invalid("asset_path", "资源相对路径无效"));
    }
    for component in value.split('/') {
        name(component)?;
        let stem = component
            .split('.')
            .next()
            .unwrap_or("")
            .to_ascii_uppercase();
        if ["CON", "PRN", "AUX", "NUL"].contains(&stem.as_str())
            || ["COM", "LPT"].iter().any(|prefix| {
                stem.strip_prefix(prefix)
                    .is_some_and(|s| s.len() == 1 && s.as_bytes()[0].is_ascii_digit())
            })
        {
            return Err(invalid("asset_path", "资源路径包含目标平台保留名称"));
        }
    }
    Ok(PathBuf::from(value))
}
fn walk(
    root: &Path,
    current: &Path,
    files: &mut Vec<SkillFile>,
    total: &mut usize,
) -> ModelResult<()> {
    safety::check_path(root, current, false)?;
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = std::fs::symlink_metadata(&path)?;
        if metadata.is_symlink() {
            return Err(invalid(
                "asset_link",
                "资源包含符号链接；请先明确整理为可迁移普通文件",
            ));
        }
        if metadata.is_dir() {
            walk(root, &path, files, total)?;
        } else if metadata.is_file() {
            if files.len() >= 20000 {
                return Err(invalid("asset_limit", "Skill资源文件超过上限"));
            }
            let relative = path
                .strip_prefix(root)
                .map_err(|_| invalid("asset_path", "资源超出读源"))?
                .to_str()
                .ok_or_else(|| invalid("asset_path", "资源路径编码不受支持"))?
                .replace(std::path::MAIN_SEPARATOR, "/");
            relative_path_check(&relative)?;
            let bytes = safety::read_file(&path, FILE_LIMIT)?;
            *total = total.saturating_add(bytes.len());
            if *total > TOTAL_LIMIT {
                return Err(invalid("asset_limit", "扩展资源总量超过24MiB"));
            }
            #[cfg(unix)]
            let executable = {
                use std::os::unix::fs::PermissionsExt;
                metadata.permissions().mode() & 0o111 != 0
            };
            #[cfg(not(unix))]
            let executable = false;
            files.push(SkillFile {
                path: relative,
                digest: sha256(&bytes),
                data_b64: STANDARD.encode(bytes),
                executable,
                sensitive: false,
            });
        } else {
            return Err(invalid("asset_type", "资源包含非常规文件，未导出"));
        }
    }
    Ok(())
}
fn relative_path_check(value: &str) -> ModelResult<()> {
    relative(value).map(|_| ())
}
fn secret_text(value: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(value) else {
        return false;
    };
    let lower = text.to_ascii_lowercase();
    ["-----begin private key-----","-----begin rsa private key-----","bearer sk-","sk-proj-","ghp_","github_pat_","akia"].iter().any(|needle|lower.contains(needle))||regex::Regex::new(r#"(?im)(?:api[_-]?key|access[_-]?token|refresh[_-]?token|password|authorization)\s*[:=]\s*["']?(?:sk-|ghp_|eyJ|[a-zA-Z0-9_+/=-]{20,})"#).expect("static credential pattern").is_match(text)
}
fn sensitive_file(path: &str, bytes: &[u8]) -> bool {
    let basename = Path::new(path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_ascii_lowercase();
    basename == ".env"
        || basename.starts_with(".env.")
        || basename.ends_with(".pem")
        || ["id_rsa", "id_ed25519", "credentials.json", "auth.json"].contains(&basename.as_str())
        || secret_text(bytes)
}
pub fn snapshot_skill(
    path: &Path,
    label: &str,
    include_secrets: bool,
    reviewed: bool,
) -> ModelResult<Skill> {
    name(label)?;
    if !reviewed {
        return Err(invalid(
            "asset_review",
            "请先检查所选Skill的自定义文本与资源，再确认参与迁移",
        ));
    }
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.is_dir() || metadata.is_symlink() {
        return Err(invalid("asset_link", "Skill读源须为普通目录"));
    }
    let root = path.canonicalize()?;
    let mut files = vec![];
    let mut total = 0;
    walk(&root, &root, &mut files, &mut total)?;
    files.sort_by(|a, b| a.path.cmp(&b.path));
    if !files.iter().any(|file| file.path == "SKILL.md") {
        return Err(invalid("asset_manifest", "所选Skill缺少SKILL.md"));
    }
    let mut secrets = false;
    let mut warnings = vec!["自定义文本与二进制资源仅作受限检查，需由选择者核对内容。".into()];
    let mut kept = vec![];
    for mut file in files {
        let bytes = STANDARD
            .decode(&file.data_b64)
            .map_err(|_| invalid("asset_payload", "资源编码无效"))?;
        let protected = sensitive_file(&file.path, &bytes);
        if protected {
            if !include_secrets {
                if file.path == "SKILL.md" {
                    return Err(invalid(
                        "asset_secret",
                        "SKILL.md检测到认证材料；请处理内容或明确选择受保护扩展凭据",
                    ));
                }
                warnings.push(format!("未携带可能的凭据文件：{}", file.path));
                continue;
            }
            secrets = true;
            file.sensitive = true;
        }
        kept.push(file);
    }
    let digest = sha256(&serde_json::to_vec(&kept)?);
    Ok(Skill {
        name: label.into(),
        digest,
        files: kept,
        contains_secrets: secrets,
        warnings,
    })
}
pub fn split_mcp(
    definition: discovery::RawServerDefinition,
    include_secrets: bool,
    reviewed: bool,
) -> ModelResult<Mcp> {
    name(&definition.name)?;
    let mut value = definition.value;
    let object = value
        .as_object_mut()
        .ok_or_else(|| invalid("mcp_definition", "MCP定义不是对象"))?;
    let mut secret_fields = vec![];
    let mut secrets = Secrets::default();
    let mut warnings = vec![];
    for key in ["env", "environment", "headers"] {
        if let Some(fields) = object.remove(key) {
            let fields = fields
                .as_object()
                .ok_or_else(|| invalid("mcp_definition", "MCP环境或请求头须为对象"))?;
            for (field, value) in fields {
                let value = value
                    .as_str()
                    .ok_or_else(|| invalid("mcp_definition", "MCP环境或请求头值须为文本"))?;
                let path = format!("/{}/{}", key, field.replace('~', "~0").replace('/', "~1"));
                secret_fields.push(path.clone());
                if include_secrets {
                    secrets.0.insert(path, value.into());
                }
            }
        }
    }
    if let Some(Value::String(address)) = object.get_mut("url") {
        let mut url =
            reqwest::Url::parse(address).map_err(|_| invalid("mcp_url", "MCP服务地址无效"))?;
        if !["http", "https"].contains(&url.scheme()) {
            return Err(invalid("mcp_url", "MCP服务须为HTTP或HTTPS地址"));
        }
        let sensitive = |key: &str| {
            [
                "key",
                "api_key",
                "apikey",
                "token",
                "access_token",
                "auth",
                "authorization",
                "password",
            ]
            .contains(&key.to_ascii_lowercase().as_str())
        };
        if !url.username().is_empty()
            || url.password().is_some()
            || url.query_pairs().any(|(key, _)| sensitive(&key))
        {
            secret_fields.push("/url".into());
            if include_secrets {
                secrets.0.insert("/url".into(), address.clone());
            }
            let remaining = url
                .query_pairs()
                .filter(|(key, _)| !sensitive(key))
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect::<Vec<_>>();
            let _ = url.set_username("");
            let _ = url.set_password(None);
            url.set_query(None);
            if !remaining.is_empty() {
                url.query_pairs_mut().extend_pairs(remaining);
            }
            *address = url.to_string();
        }
    }
    let known = [
        "command",
        "args",
        "url",
        "type",
        "transport",
        "enabled",
        "disabled",
        "timeout",
        "timeout_ms",
        "startup_timeout_sec",
        "tool_timeout_sec",
        "description",
        "cwd",
        "name",
        "id",
        "server_name",
    ];
    let unknown = object
        .keys()
        .filter(|key| !known.contains(&key.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        if !reviewed {
            return Err(invalid(
                "asset_review",
                "MCP含自定义字段，请检查后确认参与迁移",
            ));
        }
        warnings.push(format!("已人工确认的自定义字段：{}", unknown.join("、")));
    }
    if value.get("command").is_none() && value.get("url").is_none() {
        return Err(invalid("mcp_definition", "MCP定义缺少command或url"));
    }
    if secret_text(&serde_json::to_vec(&value)?) {
        return Err(invalid(
            "mcp_secret",
            "命令或自定义字段含可能的认证材料，请先改为明确的环境变量或授权头再导出",
        ));
    }
    secret_fields.sort();
    Ok(Mcp {
        name: definition.name,
        definition: value,
        secret_fields,
        secrets,
        warnings,
    })
}
pub fn summarize(assets: &Assets) -> Summary {
    Summary {
        skills: assets
            .skills
            .iter()
            .map(|skill| SkillSummary {
                name: skill.name.clone(),
                digest: skill.digest.clone(),
                files: skill.files.len(),
                bytes: skill
                    .files
                    .iter()
                    .filter_map(|f| STANDARD.decode(&f.data_b64).ok())
                    .map(|b| b.len())
                    .sum(),
                warnings: skill.warnings.clone(),
                protected: skill.contains_secrets,
            })
            .collect(),
        mcps: assets
            .mcps
            .iter()
            .map(|mcp| McpSummary {
                name: mcp.name.clone(),
                digest: sha256(&serde_json::to_vec(&mcp.definition).unwrap_or_default()),
                secret_fields: mcp.secret_fields.clone(),
                warnings: mcp.warnings.clone(),
            })
            .collect(),
        conflicts: vec![],
    }
}
pub fn secret_count(assets: &Assets) -> usize {
    assets.skills.iter().filter(|s| s.contains_secrets).count()
        + assets.mcps.iter().map(|m| m.secrets.0.len()).sum::<usize>()
}
pub fn validate(assets: &Assets) -> ModelResult<()> {
    if assets.skills.len() > 500 || assets.mcps.len() > 1000 {
        return Err(invalid("asset_limit", "扩展对象过多"));
    }
    let mut names = BTreeSet::new();
    let mut total = 0;
    for skill in &assets.skills {
        name(&skill.name)?;
        if !names.insert(("skill", skill.name.to_lowercase()))
            || skill.files.is_empty()
            || skill.files.len() > 20000
        {
            return Err(invalid("asset_identity", "Skill身份重复或文件数量无效"));
        }
        let mut paths = BTreeSet::new();
        for file in &skill.files {
            relative_path_check(&file.path)?;
            if !paths.insert(file.path.to_lowercase()) {
                return Err(invalid("asset_identity", "资源路径重复或大小写冲突"));
            }
            let bytes = STANDARD
                .decode(&file.data_b64)
                .map_err(|_| invalid("asset_payload", "资源编码无效"))?;
            total += bytes.len();
            if bytes.len() as u64 > FILE_LIMIT
                || total > TOTAL_LIMIT
                || sha256(&bytes) != file.digest
                || !file.sensitive && sensitive_file(&file.path, &bytes)
            {
                return Err(invalid("asset_digest", "资源超过上限或摘要不符"));
            }
        }
        if !skill.files.iter().any(|f| f.path == "SKILL.md")
            || skill.contains_secrets != skill.files.iter().any(|file| file.sensitive)
            || sha256(&serde_json::to_vec(&skill.files)?) != skill.digest
        {
            return Err(invalid("asset_digest", "Skill清单或摘要无效"));
        }
    }
    for mcp in &assets.mcps {
        name(&mcp.name)?;
        if !names.insert(("mcp", mcp.name.to_lowercase()))
            || !mcp.definition.is_object()
            || ["env", "environment", "headers"]
                .iter()
                .any(|field| mcp.definition.get(field).is_some())
            || secret_text(&serde_json::to_vec(&mcp.definition)?)
            || mcp.secret_fields.len() > 512
            || mcp
                .secrets
                .0
                .iter()
                .any(|(path, value)| !mcp.secret_fields.contains(path) || value.len() > 32768)
        {
            return Err(invalid("mcp_definition", "MCP定义或凭据范围无效"));
        }
        let checked = split_mcp(
            discovery::RawServerDefinition {
                name: mcp.name.clone(),
                value: mcp.definition.clone(),
            },
            false,
            true,
        )?;
        if checked.definition != mcp.definition || !checked.secret_fields.is_empty() {
            return Err(invalid("mcp_definition", "MCP公开部分仍含认证材料，未采用"));
        }
        for path in &mcp.secret_fields {
            if path != "/url"
                && !path.starts_with("/env/")
                && !path.starts_with("/environment/")
                && !path.starts_with("/headers/")
            {
                return Err(invalid("mcp_definition", "MCP秘密路径无效"));
            }
        }
    }
    Ok(())
}
fn config(root: &Path) -> ModelResult<extensions::ExtensionConfig> {
    let path = root.join(projection::CONFIG_RELATIVE_PATH);
    safety::check_path(root, &path, true)?;
    if !path.exists() {
        return Ok(Default::default());
    }
    let projection: ExtensionProjection =
        serde_json::from_slice(&safety::read_file(&path, 16 * 1024 * 1024)?)?;
    if projection.fingerprint.validate().is_err()
        || projection.config.fingerprint() != projection.fingerprint.fingerprint
    {
        return Err(invalid("asset_config", "本机扩展配置无效，未读取资源"));
    }
    Ok(projection.config)
}
pub fn export(
    root: &Path,
    home: &Path,
    skills: &[String],
    mcps: &[String],
    include_secrets: bool,
    reviewed: bool,
) -> ModelResult<Assets> {
    let (assets, conflicts) = export_with_sources(
        root,
        home,
        skills,
        mcps,
        &BTreeMap::new(),
        include_secrets,
        reviewed,
    )?;
    if let Some(conflict) = conflicts.first() {
        return Err(invalid(
            "mcp_source",
            format!(
                "MCP「{}」在多个客户端的定义不同，请先选择采用来源",
                conflict.name
            ),
        ));
    }
    Ok(assets)
}
fn differing_fields(definitions: &[&Value]) -> Vec<String> {
    let mut keys = BTreeSet::new();
    for definition in definitions {
        if let Some(object) = definition.as_object() {
            keys.extend(object.keys().cloned());
        }
    }
    keys.into_iter()
        .filter(|key| {
            let first = definitions[0].get(key);
            definitions.iter().any(|other| other.get(key) != first)
        })
        .collect()
}
/// Exports the selected assets. A same-named MCP with differing client
/// definitions uses the chosen source client; without a choice it is skipped
/// and reported, so one conflict does not block the rest (S18-03).
pub fn export_with_sources(
    root: &Path,
    home: &Path,
    skills: &[String],
    mcps: &[String],
    sources: &BTreeMap<String, String>,
    include_secrets: bool,
    reviewed: bool,
) -> ModelResult<(Assets, Vec<McpConflict>)> {
    if !home.is_absolute() || !home.is_dir() {
        return Err(invalid("asset_home", "扩展读源未绑定有效主目录"));
    }
    let mut conflicts = vec![];
    let config = config(root)?;
    let targets = CLIENT_IDS.map(|client| ClientTarget::user_target(client, home, true, true));
    let source = extensions::source_dir::configured_source_dir(&config, home);
    let source = if config.source_store.is_some() {
        extensions::source_dir::validate_source_dir(&source, root, home, &targets)
            .map_err(|_| invalid("asset_source", "自定义扩展读源未通过校验"))?
    } else {
        source
    };
    let mut result = Assets::default();
    for skill in skills {
        name(skill)?;
        let path = if source.join(skill).exists() {
            source.join(skill)
        } else {
            discovery::skills_store_dir(root).join(skill)
        };
        result
            .skills
            .push(snapshot_skill(&path, skill, include_secrets, reviewed)?);
    }
    for server in mcps {
        name(server)?;
        if let Some(stored) = mcp_store(root)?
            .into_iter()
            .find(|stored| stored.name == *server)
        {
            if include_secrets {
                let resolved = resolve_mcp(root, server, &targets)?
                    .ok_or_else(|| invalid("mcp_source", "受管MCP定义无法解析"))?;
                result.mcps.push(split_mcp(resolved, true, reviewed)?);
            } else {
                result.mcps.push(Mcp {
                    name: stored.name,
                    definition: stored.definition,
                    secret_fields: stored.secret_fields,
                    secrets: Default::default(),
                    warnings: vec![],
                });
            }
            continue;
        }
        let mut found = vec![];
        for target in &targets {
            if let Some(definition) = discovery::read_server_definition(target, server)
                .map_err(|_| invalid("mcp_read", "MCP源配置无法安全读取"))?
            {
                found.push((client_key(target.client_id), definition));
            }
        }
        let chosen = if let Some(source) = sources.get(server) {
            Some(
                found
                    .iter()
                    .find(|(client, _)| client == source)
                    .map(|(_, definition)| definition.clone())
                    .ok_or_else(|| {
                        invalid(
                            "mcp_source",
                            format!("所选来源客户端中已没有MCP「{server}」，请重新选择"),
                        )
                    })?,
            )
        } else {
            let mut distinct: Vec<&discovery::RawServerDefinition> = vec![];
            for (_, definition) in &found {
                if !distinct.iter().any(|old| old.value == definition.value) {
                    distinct.push(definition);
                }
            }
            match distinct.len() {
                0 => return Err(invalid("mcp_source", "所选MCP定义不存在")),
                1 => Some(distinct[0].clone()),
                _ => {
                    conflicts.push(McpConflict {
                        name: server.clone(),
                        clients: found.iter().map(|(client, _)| client.clone()).collect(),
                        fields: differing_fields(
                            &distinct.iter().map(|d| &d.value).collect::<Vec<_>>(),
                        ),
                    });
                    None
                }
            }
        };
        if let Some(definition) = chosen {
            result
                .mcps
                .push(split_mcp(definition, include_secrets, reviewed)?);
        }
    }
    validate(&result)?;
    Ok((result, conflicts))
}
fn mcp_store(root: &Path) -> ModelResult<Vec<StoredMcp>> {
    let path = root.join(MCP_STORE);
    safety::check_path(root, &path, true)?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let items: Vec<StoredMcp> =
        serde_json::from_slice(&safety::read_file(&path, 16 * 1024 * 1024)?)?;
    let mut names = BTreeSet::new();
    for item in &items {
        name(&item.name)?;
        if !names.insert(&item.name)
            || item
                .references
                .keys()
                .any(|path| !item.secret_fields.contains(path))
        {
            return Err(invalid("mcp_store", "MCP定义源身份或凭据范围无效"));
        }
        for reference in item.references.values() {
            require_uuid(reference)?;
        }
    }
    Ok(items)
}
fn credential(reference: &str) -> ModelResult<keyring::Entry> {
    require_uuid(reference)?;
    keyring::Entry::new(
        crate::infrastructure::keychain::keychain_service_name(),
        &format!("extension_secret:{reference}"),
    )
    .map_err(|_| invalid("extension_credential", "扩展系统凭据不可用"))
}
pub fn resolve_mcp(
    root: &Path,
    label: &str,
    targets: &[ClientTarget],
) -> ModelResult<Option<discovery::RawServerDefinition>> {
    resolve_mcp_with(root, label, targets, &SystemCredentials)
}
fn resolve_mcp_with(
    root: &Path,
    label: &str,
    targets: &[ClientTarget],
    credentials: &impl Credentials,
) -> ModelResult<Option<discovery::RawServerDefinition>> {
    let Some(stored) = mcp_store(root)?.into_iter().find(|item| item.name == label) else {
        return Ok(None);
    };
    let mut result = stored.definition.clone();
    let mut secrets = BTreeMap::new();
    for target in targets {
        if let Some(source) = discovery::read_server_definition(target, label)
            .map_err(|_| invalid("mcp_read", "现有MCP配置不可读"))?
        {
            let source = split_mcp(source, true, true)?;
            if source.definition == stored.definition {
                for (path, value) in &source.secrets.0 {
                    secrets.insert(path.clone(), value.clone());
                }
                break;
            }
        }
    }
    for (path, reference) in &stored.references {
        secrets.insert(path.clone(), credentials.read(reference)?);
    }
    for path in &stored.secret_fields {
        let value = secrets
            .get(path)
            .ok_or_else(|| invalid("extension_credential", "导入定义缺少凭据，请编辑后再分发"))?;
        set_pointer(&mut result, path, Value::String(value.clone()))?;
    }
    Ok(Some(discovery::RawServerDefinition {
        name: label.into(),
        value: result,
    }))
}
fn set_pointer(value: &mut Value, path: &str, replacement: Value) -> ModelResult<()> {
    let fields = path
        .trim_start_matches('/')
        .split('/')
        .map(|part| part.replace("~1", "/").replace("~0", "~"))
        .collect::<Vec<_>>();
    if fields.is_empty() {
        return Err(invalid("mcp_path", "MCP字段路径无效"));
    }
    let mut current = value;
    for field in &fields[..fields.len() - 1] {
        let object = current
            .as_object_mut()
            .ok_or_else(|| invalid("mcp_path", "MCP字段目标不是对象"))?;
        current = object.entry(field).or_insert_with(|| serde_json::json!({}));
    }
    current
        .as_object_mut()
        .ok_or_else(|| invalid("mcp_path", "MCP字段目标不是对象"))?
        .insert(fields.last().unwrap().clone(), replacement);
    Ok(())
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ApplyPlan {
    id: String,
    home: PathBuf,
    digest: String,
    store_before: String,
    skills_before: BTreeMap<String, Option<String>>,
    source_before: BTreeMap<String, Option<String>>,
    references: BTreeMap<String, BTreeMap<String, String>>,
    adopt_credentials: Option<bool>,
    completed: bool,
}
fn apply_path(root: &Path, id: &str) -> ModelResult<PathBuf> {
    require_uuid(id)?;
    let path = root
        .join("manager-state/extension-import-plans")
        .join(format!("{id}.json"));
    safety::check_path(root, &path, true)?;
    Ok(path)
}
fn store_bytes(root: &Path) -> ModelResult<Option<Vec<u8>>> {
    let path = root.join(MCP_STORE);
    safety::check_path(root, &path, true)?;
    match std::fs::symlink_metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
        Ok(_) => Ok(Some(safety::read_file(&path, 16 * 1024 * 1024)?)),
    }
}
fn source_dir(root: &Path, home: &Path) -> ModelResult<PathBuf> {
    let config = config(root)?;
    let targets = CLIENT_IDS.map(|id| ClientTarget::user_target(id, home, true, true));
    let source = extensions::source_dir::configured_source_dir(&config, home);
    if config.source_store.is_some() {
        extensions::source_dir::validate_source_dir(&source, root, home, &targets)
            .map_err(|_| invalid("asset_source", "本机源目录已失效，未采用扩展"))
    } else {
        Ok(source)
    }
}
fn skill_digest(path: &Path, skill: &Skill) -> ModelResult<Option<String>> {
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
        Ok(_) => Ok(Some(
            snapshot_skill(path, &skill.name, skill.contains_secrets, true)?.digest,
        )),
    }
}
pub fn prepare(root: &Path, home: &Path, id: &str, assets: &Assets) -> ModelResult<()> {
    validate(assets)?;
    if !home.is_absolute() || !home.is_dir() {
        return Err(invalid("asset_home", "接收扩展前需绑定本机主目录"));
    }
    let store = discovery::skills_store_dir(root);
    safety::check_path(root, &store, true)?;
    let source = source_dir(root, home)?;
    let mut skills_before = BTreeMap::new();
    let mut source_before = BTreeMap::new();
    for skill in &assets.skills {
        // Same-named local content is replaced on confirm (S18-04); the
        // digests only detect edits made between preview and confirm.
        skills_before.insert(
            skill.name.clone(),
            skill_digest(&store.join(&skill.name), skill)?,
        );
        source_before.insert(
            skill.name.clone(),
            skill_digest(&source.join(&skill.name), skill)?,
        );
    }
    let digest = sha256(&serde_json::to_vec(assets)?);
    let plan = ApplyPlan {
        id: id.into(),
        home: home.into(),
        digest,
        store_before: sha256(store_bytes(root)?.as_deref().unwrap_or_default()),
        skills_before,
        source_before,
        references: BTreeMap::new(),
        adopt_credentials: None,
        completed: false,
    };
    atomic_write(&apply_path(root, id)?, &serde_json::to_vec(&plan)?, 0o600)?;
    Ok(())
}
pub trait Credentials {
    fn read(&self, reference: &str) -> ModelResult<String>;
    fn install(&self, reference: &str, value: &str) -> ModelResult<()>;
}
pub struct SystemCredentials;
impl Credentials for SystemCredentials {
    fn read(&self, reference: &str) -> ModelResult<String> {
        credential(reference)?
            .get_password()
            .map_err(|_| invalid("extension_credential", "扩展秘密引用读取失败"))
    }
    fn install(&self, reference: &str, value: &str) -> ModelResult<()> {
        let entry = credential(reference)?;
        match entry.get_password() {
            Ok(current) if current == value => return Ok(()),
            Ok(_) => {
                return Err(invalid(
                    "extension_credential",
                    "扩展秘密引用内容碰撞，未替换",
                ))
            }
            Err(keyring::Error::NoEntry) => {}
            Err(_) => return Err(invalid("extension_credential", "系统秘密存储不可用")),
        }
        entry
            .set_password(value)
            .map_err(|_| invalid("extension_credential", "扩展秘密写入失败"))?;
        if entry
            .get_password()
            .map_err(|_| invalid("extension_credential", "扩展秘密尚未读回确认"))?
            != value
        {
            return Err(invalid("extension_credential", "扩展秘密读回不一致"));
        }
        Ok(())
    }
}
fn read_apply(root: &Path, id: &str) -> ModelResult<ApplyPlan> {
    let plan: ApplyPlan =
        serde_json::from_slice(&safety::read_file(&apply_path(root, id)?, 4 * 1024 * 1024)?)?;
    if plan.id != id
        || !plan.home.is_absolute()
        || plan.references.len() > 1000
        || plan.skills_before.len() > 500
    {
        return Err(invalid("asset_plan", "扩展采用计划无效"));
    }
    for name in plan
        .skills_before
        .keys()
        .chain(plan.source_before.keys())
        .chain(plan.references.keys())
    {
        self::name(name)?;
    }
    for refs in plan.references.values() {
        for reference in refs.values() {
            require_uuid(reference)?;
        }
    }
    Ok(plan)
}
pub fn verify(root: &Path, id: &str, assets: &Assets) -> ModelResult<()> {
    validate(assets)?;
    let plan = read_apply(root, id)?;
    if plan.completed {
        return Ok(());
    }
    if plan.digest != sha256(&serde_json::to_vec(assets)?) {
        return Err(invalid("asset_plan", "所选扩展与原采用计划不同"));
    }
    if !assets.mcps.is_empty()
        && plan.adopt_credentials.is_none()
        && sha256(store_bytes(root)?.as_deref().unwrap_or_default()) != plan.store_before
    {
        return Err(invalid(
            "mcp_drift",
            "接收后本机MCP定义源改变，请重新比较原文件",
        ));
    }
    if plan.adopt_credentials.is_some() {
        // Resuming an interrupted confirm: replaced Skills are rewritten
        // from the frozen package, so earlier partial work is not drift.
        return Ok(());
    }
    let source = source_dir(root, &plan.home)?;
    let store = discovery::skills_store_dir(root);
    for skill in &assets.skills {
        for (path, before) in [
            (source.join(&skill.name), &plan.source_before),
            (store.join(&skill.name), &plan.skills_before),
        ] {
            let expected = before
                .get(&skill.name)
                .ok_or_else(|| invalid("asset_plan", "Skill不在原接收范围"))?;
            let current = skill_digest(&path, skill)?;
            if current != *expected && current.as_deref() != Some(skill.digest.as_str()) {
                return Err(invalid(
                    "asset_drift",
                    format!("Skill「{}」在预览后被修改，请重新预览", skill.name),
                ));
            }
        }
    }
    Ok(())
}
fn same_files(current: &[SkillFile], desired: &[SkillFile]) -> bool {
    current.len() == desired.len()
        && desired.iter().all(|want| {
            current.iter().any(|have| {
                have.path == want.path
                    && have.digest == want.digest
                    && (cfg!(not(unix)) || have.executable == want.executable)
            })
        })
}
/// Replaces one Skill directory as a whole (S18-04). The target is the
/// directory discovery reads for this name. Existing content is backed up
/// first, files absent from the package are removed, and local credential
/// files that the package withheld are kept.
fn replace_skill(
    root: &Path,
    source: &Path,
    skill: &Skill,
    adopt_credentials: bool,
) -> ModelResult<()> {
    let (base, target) = if source.join(&skill.name).exists() {
        (source.to_path_buf(), source.join(&skill.name))
    } else {
        let store = discovery::skills_store_dir(root);
        (root.to_path_buf(), store.join(&skill.name))
    };
    safety::check_path(&base, &target, true)?;
    let mut desired: Vec<SkillFile> = skill
        .files
        .iter()
        .filter(|file| adopt_credentials || !file.sensitive)
        .cloned()
        .collect();
    let exists = std::fs::symlink_metadata(&target).is_ok();
    if exists {
        let metadata = std::fs::symlink_metadata(&target)?;
        if !metadata.is_dir() || metadata.is_symlink() {
            return Err(invalid("asset_link", "本机同名Skill不是普通目录，未覆盖"));
        }
        let canonical = target.canonicalize()?;
        let mut current = vec![];
        walk(&canonical, &canonical, &mut current, &mut 0)?;
        if !adopt_credentials {
            for file in current {
                let bytes = STANDARD
                    .decode(&file.data_b64)
                    .map_err(|_| invalid("asset_payload", "资源编码无效"))?;
                if sensitive_file(&file.path, &bytes)
                    && !desired.iter().any(|want| want.path == file.path)
                {
                    desired.push(file);
                }
            }
            let mut again = vec![];
            walk(&canonical, &canonical, &mut again, &mut 0)?;
            current = again;
        }
        if same_files(&current, &desired) {
            return Ok(());
        }
        match projection::backup_skill_directory(root, &target, "同步覆盖Skill前保护") {
            Ok(()) | Err(projection::ProjectionError::NotFound) => {}
            Err(_) => {
                return Err(invalid(
                    "asset_backup",
                    format!("Skill「{}」备份失败，未覆盖原内容", skill.name),
                ))
            }
        }
    }
    let parent = target
        .parent()
        .ok_or_else(|| invalid("asset_path", "Skill目标目录无效"))?;
    std::fs::create_dir_all(parent)?;
    for entry in std::fs::read_dir(parent)? {
        let entry = entry?;
        if entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.starts_with(&format!(".ocx-tmp-{}-", skill.name)))
        {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
    let token = uuid::Uuid::new_v4().simple().to_string();
    let staging = parent.join(format!(".ocx-tmp-{}-{token}", skill.name));
    safety::check_path(&base, &staging, true)?;
    let staged = (|| -> ModelResult<()> {
        std::fs::create_dir(&staging)?;
        for file in &desired {
            let destination = staging.join(relative(&file.path)?);
            let bytes = STANDARD
                .decode(&file.data_b64)
                .map_err(|_| invalid("asset_payload", "资源编码无效"))?;
            atomic_write(
                &destination,
                &bytes,
                if file.executable { 0o700 } else { 0o600 },
            )?;
        }
        Ok(())
    })();
    if let Err(error) = staged {
        let _ = std::fs::remove_dir_all(&staging);
        return Err(error);
    }
    let retired = parent.join(format!(".ocx-old-{}-{token}", skill.name));
    if exists {
        if let Err(error) = std::fs::rename(&target, &retired) {
            let _ = std::fs::remove_dir_all(&staging);
            return Err(error.into());
        }
    }
    if let Err(error) = std::fs::rename(&staging, &target) {
        if exists {
            let _ = std::fs::rename(&retired, &target);
        }
        let _ = std::fs::remove_dir_all(&staging);
        return Err(error.into());
    }
    if exists {
        let _ = std::fs::remove_dir_all(&retired);
    }
    for file in &desired {
        let bytes = STANDARD
            .decode(&file.data_b64)
            .map_err(|_| invalid("asset_payload", "资源编码无效"))?;
        if safety::read_file(&target.join(relative(&file.path)?), FILE_LIMIT)? != bytes {
            return Err(invalid("asset_readback", "资源文件未通过读回确认"));
        }
    }
    Ok(())
}
pub fn apply(
    root: &Path,
    id: &str,
    assets: &Assets,
    adopt_credentials: bool,
) -> ModelResult<Summary> {
    apply_with(root, id, assets, adopt_credentials, &SystemCredentials)
}
fn apply_with(
    root: &Path,
    id: &str,
    assets: &Assets,
    adopt_credentials: bool,
    credentials: &impl Credentials,
) -> ModelResult<Summary> {
    let path = apply_path(root, id)?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&path),
        true,
    )?;
    let _guard = crate::infrastructure::locking::TargetFileLock::lock(&path)?;
    let transaction = root.join("manager-state/extension-assets-transaction");
    safety::check_path(root, &transaction, true)?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&transaction),
        true,
    )?;
    let _transaction = crate::infrastructure::locking::TargetFileLock::lock(&transaction)?;
    let store_path = root.join(MCP_STORE);
    safety::check_path(root, &store_path, true)?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&store_path),
        true,
    )?;
    let _store_guard = crate::infrastructure::locking::TargetFileLock::lock(&store_path)?;
    let mut plan = read_apply(root, id)?;
    if plan
        .adopt_credentials
        .is_some_and(|adopt| adopt != adopt_credentials)
    {
        return Err(invalid("asset_intent", "原扩展采用的秘密范围已冻结"));
    }
    if plan.completed {
        return Ok(summarize(assets));
    }
    verify(root, id, assets)?;
    if !adopt_credentials
        && assets.skills.iter().any(|skill| {
            skill
                .files
                .iter()
                .any(|file| file.path == "SKILL.md" && file.sensitive)
        })
    {
        return Err(invalid(
            "asset_secret",
            "所选Skill的正文含秘密，须采用受保护凭据或跳过该Skill",
        ));
    }
    for mcp in &assets.mcps {
        let refs = plan.references.entry(mcp.name.clone()).or_default();
        if adopt_credentials {
            for field in mcp.secrets.0.keys() {
                refs.entry(field.clone())
                    .or_insert_with(|| uuid::Uuid::new_v4().to_string());
            }
        }
    }
    plan.adopt_credentials = Some(adopt_credentials);
    atomic_write(&path, &serde_json::to_vec(&plan)?, 0o600)?;
    let mut stored = mcp_store(root)?;
    for mcp in &assets.mcps {
        let mut refs = stored
            .iter()
            .find(|item| item.name == mcp.name)
            .map(|item| item.references.clone())
            .unwrap_or_default();
        if adopt_credentials {
            for field in mcp.secrets.0.keys() {
                let reference = &plan.references[&mcp.name][field];
                refs.insert(field.clone(), reference.clone());
            }
        }
        let next = StoredMcp {
            name: mcp.name.clone(),
            definition: mcp.definition.clone(),
            secret_fields: mcp.secret_fields.clone(),
            references: refs,
        };
        if let Some(index) = stored.iter().position(|item| item.name == mcp.name) {
            stored[index] = next;
        } else {
            stored.push(next);
        }
    }
    let next = serde_json::to_vec(&stored)?;
    let current = store_bytes(root)?;
    if sha256(current.as_deref().unwrap_or_default()) != plan.store_before
        && current.as_deref() != Some(next.as_slice())
    {
        return Err(invalid(
            "mcp_drift",
            "本机MCP定义源在接收后发生变化，未覆盖",
        ));
    }
    if adopt_credentials {
        for mcp in &assets.mcps {
            for (field, value) in &mcp.secrets.0 {
                let reference = &plan.references[&mcp.name][field];
                credentials.install(reference, value)?;
                if credentials.read(reference)? != *value {
                    return Err(invalid("extension_credential", "扩展秘密未通过最终读回"));
                }
            }
        }
    }
    if !assets.skills.is_empty() {
        let source = source_dir(root, &plan.home)?;
        for skill in &assets.skills {
            replace_skill(root, &source, skill, adopt_credentials)?;
        }
    }
    if !assets.mcps.is_empty() && current.as_deref() != Some(next.as_slice()) {
        let store_path = root.join(MCP_STORE);
        if let Some(previous) = current {
            crate::modules::backup::backup_file(
                root,
                crate::modules::backup::BackupAction::Import,
                &store_path,
                &previous,
                chrono::Utc::now(),
                Some("导入MCP源定义前保护".into()),
            )?;
        }
        if sha256(store_bytes(root)?.as_deref().unwrap_or_default()) != plan.store_before {
            return Err(invalid("mcp_drift", "保护备份期间MCP定义源变化，未覆盖"));
        }
        atomic_write(&store_path, &next, 0o600)?;
        if store_bytes(root)?.as_deref() != Some(next.as_slice()) {
            return Err(invalid("mcp_readback", "MCP定义源未通过读回"));
        }
    }
    plan.completed = true;
    atomic_write(&path, &serde_json::to_vec(&plan)?, 0o600)?;
    Ok(summarize(assets))
}
pub fn stored_names(root: &Path) -> ModelResult<Vec<String>> {
    Ok(mcp_store(root)?.into_iter().map(|item| item.name).collect())
}
pub fn stored_public(root: &Path) -> ModelResult<Vec<discovery::RawServerDefinition>> {
    Ok(mcp_store(root)?
        .into_iter()
        .map(|item| discovery::RawServerDefinition {
            name: item.name,
            value: item.definition,
        })
        .collect())
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub skills: Vec<String>,
    pub mcps: Vec<String>,
}
pub fn catalog(root: &Path, home: &Path) -> ModelResult<Catalog> {
    let source = source_dir(root, home)?;
    let targets = CLIENT_IDS.map(|id| ClientTarget::user_target(id, home, true, true));
    let discovered = discovery::discover(root, home, &source, &targets)
        .map_err(|_| invalid("asset_discovery", "扩展资源列表读取失败"))?;
    Ok(Catalog {
        skills: discovered
            .skills
            .into_iter()
            .map(|skill| skill.name)
            .collect(),
        mcps: discovered
            .servers
            .into_iter()
            .map(|server| server.name)
            .collect(),
    })
}
pub fn remove_stored(root: &Path, label: &str) -> ModelResult<()> {
    let mut items = mcp_store(root)?;
    if !items.iter().any(|item| item.name == label) {
        return Ok(());
    }
    let path = root.join(MCP_STORE);
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&path),
        true,
    )?;
    let _guard = crate::infrastructure::locking::TargetFileLock::lock(&path)?;
    items = mcp_store(root)?;
    items.retain(|item| item.name != label);
    atomic_write(&path, &serde_json::to_vec(&items)?, 0o600)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn renamed_mcp_secrets_bind_only_to_new_definition_and_later_destination_edits_refuse() {
        let root = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(home.path().join(".codex")).unwrap();
        std::fs::write(home.path().join(".codex/config.toml"),"[mcp_servers.fixture]\ncommand = 'local-command'\n[mcp_servers.fixture.env]\nTOKEN = 'local-SECRET'\n").unwrap();
        let source=Assets{skills:vec![],mcps:vec![split_mcp(discovery::RawServerDefinition{name:"fixture".into(),value:serde_json::json!({"command":"incoming-command","env":{"TOKEN":"incoming-SECRET"}})},true,true).unwrap()]};
        let mapped = remap(
            &source,
            &NameMappings {
                mcps: BTreeMap::from([("fixture".into(), "new-definition".into())]),
                ..Default::default()
            },
        )
        .unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        prepare(root.path(), home.path(), &id, &mapped).unwrap();
        let credentials = Memory::default();
        apply_with(root.path(), &id, &mapped, true, &credentials).unwrap();
        let targets = CLIENT_IDS.map(|id| ClientTarget::user_target(id, home.path(), true, true));
        let restored = resolve_mcp_with(root.path(), "new-definition", &targets, &credentials)
            .unwrap()
            .unwrap();
        assert_eq!(restored.value["env"]["TOKEN"], "incoming-SECRET");
        assert_eq!(
            discovery::read_server_definition(
                targets
                    .iter()
                    .find(|t| t.client_id == extensions::ClientId::Codex)
                    .unwrap(),
                "fixture"
            )
            .unwrap()
            .unwrap()
            .value["env"]["TOKEN"],
            "local-SECRET"
        );
        let id = uuid::Uuid::new_v4().to_string();
        prepare(root.path(), home.path(), &id, &mapped).unwrap();
        let mut stored = mcp_store(root.path()).unwrap();
        stored[0].definition["command"] = serde_json::json!("later-command");
        atomic_write(
            &root.path().join(MCP_STORE),
            &serde_json::to_vec(&stored).unwrap(),
            0o600,
        )
        .unwrap();
        assert!(verify(root.path(), &id, &mapped).is_err());
    }
    #[test]
    fn mapping_rejects_paths_platform_names_duplicate_targets_and_unselected_sources() {
        let mcp = |name: &str| {
            split_mcp(
                discovery::RawServerDefinition {
                    name: name.into(),
                    value: serde_json::json!({"command":"fixture-command"}),
                },
                false,
                true,
            )
            .unwrap()
        };
        let assets = Assets {
            skills: vec![],
            mcps: vec![mcp("first"), mcp("second")],
        };
        for target in ["../escape", "nested/name", "CON", "name."] {
            assert!(remap(
                &assets,
                &NameMappings {
                    mcps: BTreeMap::from([("first".into(), target.into())]),
                    ..Default::default()
                }
            )
            .is_err());
        }
        assert!(remap(
            &assets,
            &NameMappings {
                mcps: BTreeMap::from([
                    ("first".into(), "same".into()),
                    ("second".into(), "SAME".into())
                ]),
                ..Default::default()
            }
        )
        .is_err());
        assert_eq!(
            remap(
                &assets,
                &NameMappings {
                    mcps: BTreeMap::from([("unselected".into(), "target".into())]),
                    ..Default::default()
                }
            )
            .err()
            .unwrap()
            .code,
            "asset_mapping"
        );
        let moved = remap(
            &assets,
            &NameMappings {
                mcps: BTreeMap::from([("first".into(), "received".into())]),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(moved.mcps[0].name, "received");
        assert_eq!(moved.mcps[0].definition, assets.mcps[0].definition);
    }
    #[derive(Default)]
    struct Memory(std::sync::Mutex<BTreeMap<String, String>>);
    impl Credentials for Memory {
        fn read(&self, reference: &str) -> ModelResult<String> {
            self.0
                .lock()
                .unwrap()
                .get(reference)
                .cloned()
                .ok_or_else(|| invalid("extension_credential", "fixture missing"))
        }
        fn install(&self, reference: &str, value: &str) -> ModelResult<()> {
            let mut values = self.0.lock().unwrap();
            if let Some(current) = values.get(reference) {
                if current != value {
                    return Err(invalid("extension_credential", "fixture collision"));
                }
            } else {
                values.insert(reference.into(), value.into());
            }
            Ok(())
        }
    }
    #[test]
    fn imported_mcp_is_discovered_and_explicit_distribution_uses_the_existing_client_writer() {
        let root = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        let mcp = split_mcp(
            discovery::RawServerDefinition {
                name: "fixture".into(),
                value: serde_json::json!({"command":"fixture-command","args":["--fixture"]}),
            },
            false,
            true,
        )
        .unwrap();
        let assets = Assets {
            skills: vec![],
            mcps: vec![mcp],
        };
        let id = uuid::Uuid::new_v4().to_string();
        prepare(root.path(), home.path(), &id, &assets).unwrap();
        apply(root.path(), &id, &assets, false).unwrap();
        assert_eq!(
            catalog(root.path(), home.path()).unwrap().mcps,
            vec!["fixture"]
        );
        let targets =
            CLIENT_IDS.map(|client| ClientTarget::user_target(client, home.path(), true, true));
        assert!(targets
            .iter()
            .all(|target| !target.mcp_config_path.exists()));
        let written = projection::execute(
            root.path(),
            &targets,
            projection::ProjectionCommand::WriteMcp {
                name: "fixture".into(),
                client: extensions::ClientId::Codex,
                confirm: true,
            },
        )
        .unwrap();
        assert!(!written.mcp_written.is_empty());
        let target = targets
            .iter()
            .find(|target| target.client_id == extensions::ClientId::Codex)
            .unwrap();
        assert_eq!(
            discovery::read_server_definition(target, "fixture")
                .unwrap()
                .unwrap()
                .value["command"],
            "fixture-command"
        );
        assert!(targets
            .iter()
            .filter(|target| target.client_id != extensions::ClientId::Codex)
            .all(|target| !target.mcp_config_path.exists()));
    }
    #[test]
    fn skill_resource_import_resumes_a_partial_owned_directory_and_never_touches_client_or_source_files(
    ) {
        let source = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        std::fs::write(
            source.path().join("SKILL.md"),
            "# fixture\nUse scripts/check.py",
        )
        .unwrap();
        std::fs::create_dir(source.path().join("scripts")).unwrap();
        std::fs::write(source.path().join("scripts/check.py"), "print('fixture')").unwrap();
        let skill = snapshot_skill(source.path(), "fixture", false, true).unwrap();
        let assets = Assets {
            skills: vec![skill],
            mcps: vec![],
        };
        let id = uuid::Uuid::new_v4().to_string();
        prepare(root.path(), home.path(), &id, &assets).unwrap();
        let mut plan = read_apply(root.path(), &id).unwrap();
        plan.adopt_credentials = Some(false);
        atomic_write(
            &apply_path(root.path(), &id).unwrap(),
            &serde_json::to_vec(&plan).unwrap(),
            0o600,
        )
        .unwrap();
        let destination = discovery::skills_store_dir(root.path()).join("fixture");
        atomic_write(
            &destination.join("SKILL.md"),
            b"# fixture\nUse scripts/check.py",
            0o600,
        )
        .unwrap();
        apply_with(root.path(), &id, &assets, false, &Memory::default()).unwrap();
        assert_eq!(
            std::fs::read_to_string(destination.join("scripts/check.py")).unwrap(),
            "print('fixture')"
        );
        assert_eq!(
            std::fs::read_to_string(source.path().join("scripts/check.py")).unwrap(),
            "print('fixture')"
        );
        assert!(!home.path().join(".codex").exists());
        std::fs::write(destination.join("scripts/check.py"), "later human edit").unwrap();
        apply_with(root.path(), &id, &assets, false, &Memory::default()).unwrap();
        assert_eq!(
            std::fs::read_to_string(destination.join("scripts/check.py")).unwrap(),
            "later human edit"
        );
        // A new sync replaces the same-named Skill as a whole (S18-04):
        // edited files are restored, stale files removed, old content backed up.
        std::fs::write(destination.join("stale.txt"), "stale").unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        prepare(root.path(), home.path(), &id, &assets).unwrap();
        apply_with(root.path(), &id, &assets, false, &Memory::default()).unwrap();
        assert_eq!(
            std::fs::read_to_string(destination.join("scripts/check.py")).unwrap(),
            "print('fixture')"
        );
        assert!(!destination.join("stale.txt").exists());
        assert!(backup_count(root.path()) > 0);
    }
    fn backup_count(root: &Path) -> usize {
        fn count(path: &Path) -> usize {
            std::fs::read_dir(path)
                .map(|entries| {
                    entries
                        .flatten()
                        .map(|entry| {
                            if entry.path().is_dir() {
                                count(&entry.path())
                            } else {
                                1
                            }
                        })
                        .sum()
                })
                .unwrap_or(0)
        }
        count(&root.join("backups"))
    }
    #[test]
    fn differing_same_named_mcp_is_skipped_and_reported_until_a_source_client_is_chosen() {
        let root = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(home.path().join(".codex")).unwrap();
        std::fs::create_dir_all(home.path().join(".hermes")).unwrap();
        std::fs::write(
            home.path().join(".codex/config.toml"),
            "[mcp_servers.fixture]\ncommand = 'codex-command'\n[mcp_servers.same]\ncommand = 'same-command'\n",
        )
        .unwrap();
        std::fs::write(
            home.path().join(".hermes/config.yaml"),
            "mcp_servers:\n  fixture:\n    command: hermes-command\n  same:\n    command: same-command\n",
        )
        .unwrap();
        let names = vec!["fixture".to_string(), "same".to_string()];
        let (assets, conflicts) = export_with_sources(
            root.path(),
            home.path(),
            &[],
            &names,
            &BTreeMap::new(),
            false,
            true,
        )
        .unwrap();
        assert_eq!(
            assets
                .mcps
                .iter()
                .map(|m| m.name.as_str())
                .collect::<Vec<_>>(),
            vec!["same"]
        );
        assert_eq!(
            conflicts,
            vec![McpConflict {
                name: "fixture".into(),
                clients: vec!["codex".into(), "hermes".into()],
                fields: vec!["command".into()],
            }]
        );
        let summary = serde_json::to_string(&conflicts).unwrap();
        assert!(!summary.contains("codex-command") && !summary.contains("hermes-command"));
        let (assets, conflicts) = export_with_sources(
            root.path(),
            home.path(),
            &[],
            &names,
            &BTreeMap::from([("fixture".to_string(), "hermes".to_string())]),
            false,
            true,
        )
        .unwrap();
        assert!(conflicts.is_empty());
        let chosen = assets.mcps.iter().find(|m| m.name == "fixture").unwrap();
        assert_eq!(chosen.definition["command"], "hermes-command");
        assert_eq!(
            export_with_sources(
                root.path(),
                home.path(),
                &[],
                &names,
                &BTreeMap::from([("fixture".to_string(), "gemini".to_string())]),
                false,
                true,
            )
            .err()
            .unwrap()
            .code,
            "mcp_source"
        );
        assert_eq!(
            export(root.path(), home.path(), &[], &names, false, true)
                .err()
                .unwrap()
                .code,
            "mcp_source"
        );
        assert!(!default_included_mcp("node_repl"));
        assert!(default_included_mcp("fixture"));
    }
    #[test]
    fn same_named_source_skill_is_replaced_with_backup_and_keeps_withheld_local_credentials() {
        let package = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        std::fs::write(package.path().join("SKILL.md"), "# fixture\nincoming").unwrap();
        let skill = snapshot_skill(package.path(), "fixture", false, true).unwrap();
        let assets = Assets {
            skills: vec![skill],
            mcps: vec![],
        };
        let local = home.path().join(".agents/skills/fixture");
        std::fs::create_dir_all(&local).unwrap();
        std::fs::write(local.join("SKILL.md"), "# fixture\nlocal").unwrap();
        std::fs::write(local.join("notes.md"), "local only").unwrap();
        std::fs::write(local.join(".env"), "TOKEN=local-SECRET").unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        prepare(root.path(), home.path(), &id, &assets).unwrap();
        apply_with(root.path(), &id, &assets, false, &Memory::default()).unwrap();
        assert_eq!(
            std::fs::read_to_string(local.join("SKILL.md")).unwrap(),
            "# fixture\nincoming"
        );
        assert!(!local.join("notes.md").exists());
        assert_eq!(
            std::fs::read_to_string(local.join(".env")).unwrap(),
            "TOKEN=local-SECRET"
        );
        assert!(!discovery::skills_store_dir(root.path())
            .join("fixture")
            .exists());
        assert!(backup_count(root.path()) > 0);
        let leftovers: Vec<_> = std::fs::read_dir(home.path().join(".agents/skills"))
            .unwrap()
            .flatten()
            .filter(|entry| entry.file_name().to_string_lossy().starts_with(".ocx-"))
            .collect();
        assert!(leftovers.is_empty());
    }
    #[test]
    fn edits_between_preview_and_confirm_refuse_instead_of_being_overwritten() {
        let package = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        std::fs::write(package.path().join("SKILL.md"), "# fixture\nincoming").unwrap();
        let skill = snapshot_skill(package.path(), "fixture", false, true).unwrap();
        let assets = Assets {
            skills: vec![skill],
            mcps: vec![],
        };
        let local = home.path().join(".agents/skills/fixture");
        std::fs::create_dir_all(&local).unwrap();
        std::fs::write(local.join("SKILL.md"), "# fixture\nlocal").unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        prepare(root.path(), home.path(), &id, &assets).unwrap();
        std::fs::write(local.join("SKILL.md"), "# fixture\nedited after preview").unwrap();
        assert_eq!(
            apply_with(root.path(), &id, &assets, false, &Memory::default())
                .unwrap_err()
                .code,
            "asset_drift"
        );
        assert_eq!(
            std::fs::read_to_string(local.join("SKILL.md")).unwrap(),
            "# fixture\nedited after preview"
        );
    }
    #[test]
    fn mcp_definition_restores_secrets_through_fresh_references_without_running_or_projecting_any_client(
    ) {
        let root = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        let raw = serde_json::json!({"command":"fixture-only-command","args":["--fixture"],"env":{"API_KEY":"fixture-secret"},"headers":{"Authorization":"Bearer fixture-token"}});
        let mcp = split_mcp(
            discovery::RawServerDefinition {
                name: "fixture".into(),
                value: raw.clone(),
            },
            true,
            true,
        )
        .unwrap();
        let assets = Assets {
            skills: vec![],
            mcps: vec![mcp],
        };
        let id = uuid::Uuid::new_v4().to_string();
        let keys = Memory::default();
        prepare(root.path(), home.path(), &id, &assets).unwrap();
        apply_with(root.path(), &id, &assets, true, &keys).unwrap();
        let metadata = String::from_utf8(store_bytes(root.path()).unwrap().unwrap()).unwrap();
        assert!(!metadata.contains("fixture-secret") && !metadata.contains("fixture-token"));
        assert_eq!(
            resolve_mcp_with(root.path(), "fixture", &[], &keys)
                .unwrap()
                .unwrap()
                .value,
            raw
        );
        assert!(!home.path().join(".codex").exists());
        let before = keys.0.lock().unwrap().len();
        apply_with(root.path(), &id, &assets, true, &keys).unwrap();
        assert_eq!(keys.0.lock().unwrap().len(), before);
        assert_eq!(
            apply_with(root.path(), &id, &assets, false, &keys)
                .unwrap_err()
                .code,
            "asset_intent"
        );
        let other = tempfile::tempdir().unwrap();
        let no_key = uuid::Uuid::new_v4().to_string();
        prepare(other.path(), home.path(), &no_key, &assets).unwrap();
        apply_with(other.path(), &no_key, &assets, false, &keys).unwrap();
        assert_eq!(
            resolve_mcp_with(other.path(), "fixture", &[], &keys)
                .unwrap_err()
                .code,
            "extension_credential"
        );
    }
    #[test]
    fn skill_snapshot_carries_real_resources_and_excludes_recognized_credentials_until_selected() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("SKILL.md"),
            "# Fixture\nUse scripts/check.py",
        )
        .unwrap();
        std::fs::create_dir(root.path().join("scripts")).unwrap();
        std::fs::write(root.path().join("scripts/check.py"), "print('fixture')").unwrap();
        std::fs::write(root.path().join(".env"), "TOKEN=fixture-only-value").unwrap();
        assert_eq!(
            snapshot_skill(root.path(), "fixture", false, false)
                .unwrap_err()
                .code,
            "asset_review"
        );
        let ordinary = snapshot_skill(root.path(), "fixture", false, true).unwrap();
        assert_eq!(ordinary.files.len(), 2);
        assert!(ordinary
            .warnings
            .iter()
            .any(|warning| warning.contains(".env")));
        assert!(!ordinary.contains_secrets);
        let protected = snapshot_skill(root.path(), "fixture", true, true).unwrap();
        assert_eq!(protected.files.len(), 3);
        assert!(protected.contains_secrets);
        assert!(
            protected
                .files
                .iter()
                .find(|file| file.path == ".env")
                .unwrap()
                .sensitive
        );
        validate(&Assets {
            skills: vec![protected],
            mcps: vec![],
        })
        .unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn links_path_escape_and_case_collisions_refuse_instead_of_being_skipped() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("outside"), "outside-owned-data").unwrap();
        std::fs::write(root.path().join("SKILL.md"), "fixture").unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("linked")).unwrap();
        assert_eq!(
            snapshot_skill(root.path(), "fixture", false, true)
                .unwrap_err()
                .code,
            "asset_link"
        );
        assert!(relative("../../outside").is_err());
        assert!(relative("a/C:\\escape").is_err());
        assert!(relative("CON.txt").is_err());
        std::fs::remove_file(root.path().join("linked")).unwrap();
        let mut skill = snapshot_skill(root.path(), "fixture", false, true).unwrap();
        let mut duplicate = skill.files[0].clone();
        duplicate.path = "skill.md".into();
        skill.files.push(duplicate);
        skill.digest = sha256(&serde_json::to_vec(&skill.files).unwrap());
        assert_eq!(
            validate(&Assets {
                skills: vec![skill],
                mcps: vec![]
            })
            .unwrap_err()
            .code,
            "asset_identity"
        );
        assert_eq!(
            std::fs::read_to_string(outside.path().join("outside")).unwrap(),
            "outside-owned-data"
        );
    }
    #[test]
    fn mcp_separates_environment_headers_and_credential_urls_without_discarding_normal_fields() {
        let value = serde_json::json!({"url":"https://user:fixturepass@example.test/mcp?token=fixture-token&project=demo","headers":{"Authorization":"Bearer fixture-only-key"},"env":{"API_TOKEN":"fixture-secret"},"timeout":9000,"customField":{"keep":"yes"}});
        assert!(
            matches!(split_mcp(discovery::RawServerDefinition {name:"fixture".into(),value:value.clone()},false,false),Err(error) if error.code=="asset_review")
        );
        let public = split_mcp(
            discovery::RawServerDefinition {
                name: "fixture".into(),
                value: value.clone(),
            },
            false,
            true,
        )
        .unwrap();
        assert_eq!(public.definition["customField"]["keep"], "yes");
        assert_eq!(public.definition["timeout"], 9000);
        assert_eq!(
            public.definition["url"],
            "https://example.test/mcp?project=demo"
        );
        assert!(public.secrets.0.is_empty());
        let text = serde_json::to_string(&public.definition).unwrap();
        assert!(
            !text.contains("fixture-secret")
                && !text.contains("fixturepass")
                && !text.contains("fixture-token")
        );
        let full = split_mcp(
            discovery::RawServerDefinition {
                name: "fixture".into(),
                value: value.clone(),
            },
            true,
            true,
        )
        .unwrap();
        let mut restored = full.definition.clone();
        for (path, value) in &full.secrets.0 {
            set_pointer(&mut restored, path, Value::String(value.clone())).unwrap();
        }
        assert_eq!(restored, value);
        let summary = summarize(&Assets {
            skills: vec![],
            mcps: vec![full],
        });
        assert!(!serde_json::to_string(&summary)
            .unwrap()
            .contains("fixture-secret"));
    }
}
