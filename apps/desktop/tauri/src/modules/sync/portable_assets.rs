//! Explicit extension assets. Source trees are read-only; received assets live
//! in the existing manager skill store or a manager-owned MCP definition source.
//! A definition does not execute or project into a client merely by importing it.
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
pub struct Summary {
    pub skills: Vec<SkillSummary>,
    pub mcps: Vec<McpSummary>,
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
    if !home.is_absolute() || !home.is_dir() {
        return Err(invalid("asset_home", "扩展读源未绑定有效主目录"));
    }
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
        let mut definitions = vec![];
        for target in &targets {
            if let Some(definition) = discovery::read_server_definition(target, server)
                .map_err(|_| invalid("mcp_read", "MCP源配置无法安全读取"))?
            {
                if !definitions
                    .iter()
                    .any(|old: &discovery::RawServerDefinition| old.value == definition.value)
                {
                    definitions.push(definition);
                }
            }
        }
        if definitions.len() != 1 {
            return Err(invalid(
                "mcp_source",
                if definitions.is_empty() {
                    "所选MCP定义不存在"
                } else {
                    "多个客户端的同名MCP定义不同，请先明确采用来源"
                },
            ));
        }
        result
            .mcps
            .push(split_mcp(definitions.remove(0), include_secrets, reviewed)?);
    }
    validate(&result)?;
    Ok(result)
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
        let before = skill_digest(&store.join(&skill.name), skill)?;
        let original = skill_digest(&source.join(&skill.name), skill)?;
        if before
            .as_ref()
            .is_some_and(|digest| digest != &skill.digest)
            || original
                .as_ref()
                .is_some_and(|digest| digest != &skill.digest)
        {
            return Err(invalid(
                "asset_conflict",
                format!(
                    "Skill「{}」与本机同名内容不同，请选择跳过或改名",
                    skill.name
                ),
            ));
        }
        skills_before.insert(skill.name.clone(), before);
        source_before.insert(skill.name.clone(), original);
    }
    let stored = mcp_store(root)?;
    let targets = CLIENT_IDS.map(|client| ClientTarget::user_target(client, home, true, true));
    for mcp in &assets.mcps {
        if stored
            .iter()
            .any(|existing| existing.name == mcp.name && existing.definition != mcp.definition)
        {
            return Err(invalid(
                "mcp_conflict",
                format!("MCP「{}」与本机定义不同，请先明确映射", mcp.name),
            ));
        }
        for target in &targets {
            if let Some(existing) = discovery::read_server_definition(target, &mcp.name)
                .map_err(|_| invalid("mcp_read", "本机MCP定义无法安全核对"))?
            {
                if split_mcp(existing, false, true)?.definition != mcp.definition {
                    return Err(invalid(
                        "mcp_conflict",
                        format!("客户端中的MCP「{}」存在不同定义，未覆盖", mcp.name),
                    ));
                }
            }
        }
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
fn verify_skill_target(
    root: &Path,
    skill: &Skill,
    expected: Option<&String>,
    resuming: bool,
) -> ModelResult<()> {
    let path = discovery::skills_store_dir(root).join(&skill.name);
    safety::check_path(root, &path, true)?;
    if !path.exists() {
        if expected.is_some() {
            return Err(invalid(
                "asset_drift",
                "本机Skill在接收后被移除，未自动重建",
            ));
        }
        return Ok(());
    }
    if skill_digest(&path, skill).is_ok_and(|digest| digest.as_deref() == Some(&skill.digest)) {
        return Ok(());
    }
    if expected.is_some() || !resuming {
        return Err(invalid("asset_drift", "接收后Skill发生修改，未覆盖"));
    }
    let canonical = path.canonicalize()?;
    let mut current = vec![];
    walk(&canonical, &canonical, &mut current, &mut 0)?;
    if current.iter().any(|file| {
        !skill.files.iter().any(|desired| {
            desired.path == file.path
                && desired.digest == file.digest
                && desired.executable == file.executable
        })
    }) {
        return Err(invalid(
            "asset_drift",
            "部分导入目录出现后来内容，未继续覆盖",
        ));
    }
    Ok(())
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
    let source = source_dir(root, &plan.home)?;
    for skill in &assets.skills {
        if skill_digest(&source.join(&skill.name), skill)?
            != *plan
                .source_before
                .get(&skill.name)
                .ok_or_else(|| invalid("asset_plan", "Skill不在原接收范围"))?
        {
            return Err(invalid("asset_drift", "扩展读源发生变化，请重新比较"));
        }
        verify_skill_target(
            root,
            skill,
            plan.skills_before.get(&skill.name).and_then(Option::as_ref),
            plan.adopt_credentials.is_some(),
        )?;
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
    for skill in &assets.skills {
        let directory = discovery::skills_store_dir(root).join(&skill.name);
        safety::check_path(root, &directory, true)?;
        for file in skill
            .files
            .iter()
            .filter(|file| adopt_credentials || !file.sensitive)
        {
            let destination = directory.join(relative(&file.path)?);
            safety::check_path(root, &destination, true)?;
            let bytes = STANDARD
                .decode(&file.data_b64)
                .map_err(|_| invalid("asset_payload", "资源编码无效"))?;
            if destination.exists() {
                if safety::read_file(&destination, FILE_LIMIT)? != bytes {
                    return Err(invalid("asset_drift", "资源文件已有后来内容，未覆盖"));
                }
            } else {
                atomic_write(
                    &destination,
                    &bytes,
                    if file.executable { 0o700 } else { 0o600 },
                )?;
            }
            if safety::read_file(&destination, FILE_LIMIT)? != bytes {
                return Err(invalid("asset_readback", "资源文件未通过读回确认"));
            }
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
        assert_eq!(
            prepare(
                root.path(),
                home.path(),
                &uuid::Uuid::new_v4().to_string(),
                &assets
            )
            .unwrap_err()
            .code,
            "asset_conflict"
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
