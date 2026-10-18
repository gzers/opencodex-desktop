use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SCHEMA_VERSION: u32 = 1;
pub const LIBRARY_PATH: &str = "manager-state/model-family-templates.json";
pub const POLICY_PATH: &str = "manager-state/model-policy.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[error("{message}")]
#[serde(rename_all = "camelCase")]
pub struct ModelError {
    pub code: String,
    pub message: String,
}
pub type ModelResult<T> = Result<T, ModelError>;
pub fn invalid(code: &str, message: impl Into<String>) -> ModelError {
    ModelError {
        code: code.into(),
        message: message.into(),
    }
}
impl From<crate::errors::AppError> for ModelError {
    fn from(error: crate::errors::AppError) -> Self {
        invalid("storage", error.to_string())
    }
}
impl From<std::io::Error> for ModelError {
    fn from(error: std::io::Error) -> Self {
        invalid("storage", error.to_string())
    }
}
impl From<serde_json::Error> for ModelError {
    fn from(error: serde_json::Error) -> Self {
        invalid("invalid_document", error.to_string())
    }
}

/// Explicit false/empty custom values are distinct from inheritance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(
    tag = "mode",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Field<T> {
    #[default]
    Inherit,
    Custom(T),
}
impl<T: Clone> Field<T> {
    pub fn overlay(&self, parent: &Self) -> Self {
        match self {
            Self::Inherit => parent.clone(),
            Self::Custom(_) => self.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelRules {
    pub protocol: Field<String>,
    pub context_window: Field<u64>,
    pub input_modalities: Field<Vec<String>>,
    pub reasoning_efforts: Field<Vec<String>>,
    pub default_effort: Field<String>,
    pub fast: Field<bool>,
}
impl ModelRules {
    pub fn overlay(&self, parent: &Self) -> Self {
        Self {
            protocol: self.protocol.overlay(&parent.protocol),
            context_window: self.context_window.overlay(&parent.context_window),
            input_modalities: self.input_modalities.overlay(&parent.input_modalities),
            reasoning_efforts: self.reasoning_efforts.overlay(&parent.reasoning_efforts),
            default_effort: self.default_effort.overlay(&parent.default_effort),
            fast: self.fast.overlay(&parent.fast),
        }
    }
    pub fn normalize(&mut self) {
        for field in [&mut self.input_modalities, &mut self.reasoning_efforts] {
            if let Field::Custom(values) = field {
                values.sort();
                values.dedup();
            }
        }
    }
    pub fn validate(&self) -> ModelResult<()> {
        if matches!(self.context_window, Field::Custom(0)) {
            return Err(invalid("context_window", "上下文窗口必须大于0"));
        }
        if let Field::Custom(value) = &self.protocol {
            if ![
                "openai-responses",
                "openai-chat",
                "anthropic",
                "google",
                "ollama-native",
                "codebuddy",
                "command-code",
                "kiro",
                "azure",
                "azure-openai",
                "cursor",
                "devin",
                "zed",
                "mimo-free",
                "qoder",
                "claude-cli",
            ]
            .contains(&value.as_str())
            {
                return Err(invalid(
                    "protocol",
                    "请选择受支持的协议；实际渠道能力仍需核验",
                ));
            }
        }
        if let Field::Custom(values) = &self.input_modalities {
            if values.is_empty()
                || values
                    .iter()
                    .any(|v| !["text", "image", "audio", "video"].contains(&v.as_str()))
            {
                return Err(invalid("modalities", "输入模态为空或包含未知值"));
            }
        }
        if let Field::Custom(values) = &self.reasoning_efforts {
            if values.iter().any(|v| {
                !["none", "minimal", "low", "medium", "high", "xhigh"].contains(&v.as_str())
            }) {
                return Err(invalid("reasoning", "包含未知推理档位"));
            }
            if let Field::Custom(default) = &self.default_effort {
                if !values.contains(default) {
                    return Err(invalid("reasoning", "默认推理档位不在可选集合内"));
                }
            }
        }
        if let Field::Custom(value) = &self.default_effort {
            if !["none", "minimal", "low", "medium", "high", "xhigh"].contains(&value.as_str()) {
                return Err(invalid("reasoning", "未知默认推理档位"));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchKind {
    Exact,
    Prefix,
    Glob,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pattern {
    pub kind: MatchKind,
    pub value: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChannelMode {
    #[default]
    All,
    Whitelist,
    Blacklist,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct MatchRules {
    pub include: Vec<Pattern>,
    pub exclude: Vec<Pattern>,
    pub case_sensitive: bool,
    pub channel_mode: ChannelMode,
    pub channels: BTreeSet<String>,
    pub priority: i32,
}
impl Default for MatchRules {
    fn default() -> Self {
        Self {
            include: vec![],
            exclude: vec![],
            case_sensitive: false,
            channel_mode: ChannelMode::All,
            channels: BTreeSet::new(),
            priority: 0,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Variant {
    pub id: String,
    pub name: String,
    pub suffix: String,
    pub rules: ModelRules,
    #[serde(default)]
    pub locked_fields: BTreeSet<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TemplateContent {
    pub description: String,
    pub matching: MatchRules,
    pub rules: ModelRules,
    #[serde(default)]
    pub variants: Vec<Variant>,
    #[serde(default)]
    pub locked_fields: BTreeSet<String>,
}
impl TemplateContent {
    pub fn normalize(&mut self) {
        self.matching.include.sort();
        self.matching.include.dedup();
        self.matching.exclude.sort();
        self.matching.exclude.dedup();
        if self.matching.channel_mode == ChannelMode::All {
            self.matching.channels.clear();
        }
        self.rules.normalize();
        for variant in &mut self.variants {
            variant.rules.normalize();
        }
    }
    pub fn validate(&self) -> ModelResult<()> {
        let patterns = self.matching.include.iter().chain(&self.matching.exclude);
        if self.matching.include.is_empty()
            || patterns
                .clone()
                .any(|p| p.value.is_empty() || p.value.len() > 1024)
        {
            return Err(invalid(
                "matching",
                "包含规则不能为空；匹配文本须为1至1024字节",
            ));
        }
        if self.matching.include.len() + self.matching.exclude.len() > 256
            || self.variants.len() > 256
        {
            return Err(invalid("limit", "规则或后缀项超过单模版上限"));
        }
        self.rules.validate()?;
        let mut ids = BTreeSet::new();
        let mut suffixes = BTreeSet::new();
        for variant in &self.variants {
            require_uuid(&variant.id)?;
            let suffix = if self.matching.case_sensitive {
                variant.suffix.clone()
            } else {
                variant.suffix.to_lowercase()
            };
            if suffix.is_empty() || !ids.insert(&variant.id) || !suffixes.insert(suffix) {
                return Err(invalid("variant", "后缀或子项身份为空/重复"));
            }
            variant.rules.overlay(&self.rules).validate()?;
        }
        Ok(())
    }
}
pub fn require_uuid(value: &str) -> ModelResult<()> {
    let id = uuid::Uuid::parse_str(value).map_err(|_| invalid("identity", "完整身份格式无效"))?;
    if id.to_string() != value || id.get_version_num() != 4 {
        return Err(invalid("identity", "须使用完整小写UUIDv4身份"));
    }
    Ok(())
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Revision {
    pub id: String,
    pub created_at: String,
    pub saved_label: String,
    pub content: TemplateContent,
    pub digest: String,
    pub base_revision: Option<String>,
    pub changes: Vec<ChangeFact>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChangeFact {
    pub field: String,
    pub before: serde_json::Value,
    pub after: serde_json::Value,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateSource {
    Builtin,
    User,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Template {
    pub id: String,
    pub label: String,
    pub source: TemplateSource,
    pub default_revision: Option<String>,
    pub revisions: Vec<Revision>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Library {
    pub schema_version: u32,
    pub templates: Vec<Template>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TemplateRef {
    pub template_id: String,
    pub revision_id: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelPolicy {
    pub id: String,
    pub label: String,
    pub template_refs: Vec<TemplateRef>,
    pub model_refs: BTreeMap<String, TemplateRef>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Policy {
    pub schema_version: u32,
    pub template_refs: Vec<TemplateRef>,
    pub channels: Vec<ChannelPolicy>,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            template_refs: vec![],
            channels: vec![],
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveTemplate {
    pub expected_library: String,
    pub template_id: Option<String>,
    pub base_revision: Option<String>,
    pub label: String,
    pub content: TemplateContent,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub library: Library,
    pub library_digest: String,
    pub policy_digest: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    pub snapshot: Snapshot,
    pub template_id: String,
    pub revision_id: String,
    pub changed: bool,
}
