use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SCHEMA_VERSION: u32 = 1;
pub const LIBRARY_PATH: &str = "manager-state/model-family-templates.json";
pub const POLICY_PATH: &str = "manager-state/model-policy.json";
pub const DRAFT_PATH: &str = "manager-state/model-config-draft.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, thiserror::Error)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelRules {
    pub protocol: Field<String>,
    pub context_window: Field<u64>,
    pub input_modalities: Field<Vec<String>>,
    pub reasoning_efforts: Field<Vec<String>>,
    pub default_effort: Field<String>,
    pub fast: Field<bool>,
    #[serde(skip_serializing_if = "is_inherit")]
    pub pinned_effort: Field<String>,
    #[serde(skip_serializing_if = "is_inherit")]
    pub effort_map: Field<BTreeMap<String, String>>,
}
fn is_inherit<T>(value: &Field<T>) -> bool {
    matches!(value, Field::Inherit)
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
            pinned_effort: self.pinned_effort.overlay(&parent.pinned_effort),
            effort_map: self.effort_map.overlay(&parent.effort_map),
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
        let effort = |value: &str| {
            [
                "none", "minimal", "low", "medium", "high", "xhigh", "max", "ultra",
            ]
            .contains(&value)
        };
        if let Field::Custom(pin) = &self.pinned_effort {
            if !effort(pin)
                || matches!(&self.reasoning_efforts,Field::Custom(values) if !values.contains(pin))
            {
                return Err(invalid(
                    "pinned_effort",
                    "强制推理档必须属于最终可选列表，不能用默认档代替",
                ));
            }
        }
        if let Field::Custom(map) = &self.effort_map {
            if map.len() > 8
                || map.iter().any(|(key, value)| {
                    !effort(key)
                        || value.is_empty()
                        || value.len() > 64
                        || !value
                            .bytes()
                            .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
                })
            {
                return Err(invalid(
                    "effort_map",
                    "推理映射只接受已声明档位与64字节内的上游标签；__omit__表示不发送参数",
                ));
            }
            if matches!(&self.reasoning_efforts,Field::Custom(values) if map.keys().any(|key|!values.contains(key)))
            {
                return Err(invalid("effort_map", "推理映射包含最终列表以外的档位"));
            }
        }
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
                ![
                    "none", "minimal", "low", "medium", "high", "xhigh", "max", "ultra",
                ]
                .contains(&v.as_str())
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
            if ![
                "none", "minimal", "low", "medium", "high", "xhigh", "max", "ultra",
            ]
            .contains(&value.as_str())
            {
                return Err(invalid("reasoning", "未知默认推理档位"));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MatchKind {
    Exact,
    Prefix,
    Glob,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Pattern {
    pub kind: MatchKind,
    pub value: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChannelMode {
    #[default]
    All,
    Whitelist,
    Blacklist,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Variant {
    pub id: String,
    pub name: String,
    pub suffix: String,
    pub rules: ModelRules,
    #[serde(default)]
    pub locked_fields: BTreeSet<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TemplateContent {
    pub description: String,
    pub matching: MatchRules,
    pub rules: ModelRules,
    #[serde(default)]
    pub variants: Vec<Variant>,
    #[serde(default)]
    pub locked_fields: BTreeSet<String>,
    #[serde(default)]
    pub sources: Vec<SourceEvidence>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceEvidence {
    pub field: String,
    pub url: Option<String>,
    pub note: String,
    pub recorded_at: String,
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
        if self.sources.len() > 128 || self.description.len() > 32768 {
            return Err(invalid("source_limit", "模版说明或来源条目过多"));
        }
        for source in &self.sources {
            if source.note.len() > 8192
                || source.field.len() > 256
                || chrono::DateTime::parse_from_rfc3339(&source.recorded_at).is_err()
            {
                return Err(invalid("source", "来源记录缺少有效字段或时间"));
            }
            if let Some(url) = &source.url {
                let url =
                    reqwest::Url::parse(url).map_err(|_| invalid("source", "来源地址无效"))?;
                if !["http", "https"].contains(&url.scheme())
                    || !url.username().is_empty()
                    || url.password().is_some()
                {
                    return Err(invalid("source", "来源地址不能含凭据或非HTTP协议"));
                }
            }
        }
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
        let rule_locks = [
            "protocol",
            "contextWindow",
            "inputModalities",
            "reasoningEfforts",
            "defaultEffort",
            "fast",
            "pinnedEffort",
            "effortMap",
        ];
        if self.locked_fields.iter().any(|v| {
            !["description", "matching", "rules", "variants"].contains(&v.as_str())
                && !rule_locks.iter().any(|key| v == &format!("rules.{key}"))
        }) {
            return Err(invalid("locked_field", "人工锁包含未知字段路径"));
        }
        let mut ids = BTreeSet::new();
        let mut suffixes = BTreeSet::new();
        for variant in &self.variants {
            if variant
                .locked_fields
                .iter()
                .any(|v| !rule_locks.contains(&v.as_str()))
            {
                return Err(invalid("locked_field", "型号人工锁包含未知字段路径"));
            }
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Revision {
    pub id: String,
    pub created_at: String,
    pub saved_label: String,
    pub content: TemplateContent,
    pub digest: String,
    pub base_revision: Option<String>,
    pub changes: Vec<ChangeFact>,
    #[serde(default)]
    pub origin: SaveOrigin,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(
    tag = "kind",
    content = "taskId",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum SaveOrigin {
    #[default]
    Manual,
    Builtin,
    ExternalTask(String),
    InternalAgent(String),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChangeFact {
    pub field: String,
    pub before: serde_json::Value,
    pub after: serde_json::Value,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TemplateSource {
    Builtin,
    User,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Template {
    pub id: String,
    pub label: String,
    pub source: TemplateSource,
    pub default_revision: Option<String>,
    pub revisions: Vec<Revision>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Library {
    pub schema_version: u32,
    pub templates: Vec<Template>,
    #[serde(default)]
    pub submission_receipts: BTreeMap<String, SubmissionReceipt>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TemplateRef {
    pub template_id: String,
    pub revision_id: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelPolicy {
    pub id: String,
    pub label: String,
    pub template_refs: Vec<TemplateRef>,
    pub model_refs: BTreeMap<String, TemplateRef>,
    #[serde(default)]
    pub connection: ChannelConnection,
    #[serde(default)]
    pub selection: ModelSelection,
    #[serde(default)]
    pub model_order: Vec<String>,
    #[serde(default)]
    pub discovered: Vec<DiscoveredModel>,
    #[serde(default)]
    pub defaults: ModelRules,
    #[serde(default)]
    pub overrides: BTreeMap<String, ModelRules>,
    #[serde(default)]
    pub family_overrides: BTreeMap<String, ModelRules>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelConnection {
    pub provider_id: String,
    pub protocol: String,
    pub nodes: Vec<ServiceNode>,
    pub active_node_id: String,
    pub credential_ref: Option<String>,
    pub enabled: bool,
    pub allow_private_network: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_read_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub imported_config: Option<ImportedProviderRef>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportedProviderRef {
    pub id: String,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServiceNode {
    pub id: String,
    pub name: String,
    pub url: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiscoveredModel {
    pub id: String,
    pub label: String,
    pub rules: ModelRules,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(
    tag = "mode",
    content = "models",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ModelSelection {
    All,
    #[default]
    None,
    Whitelist(Vec<String>),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Policy {
    pub schema_version: u32,
    pub template_refs: Vec<TemplateRef>,
    pub channels: Vec<ChannelPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_channel_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pending_removals: Vec<SourceRemoval>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_id: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceRemoval {
    pub read_id: String,
    pub channel_id: String,
    pub provider_id: String,
    pub credential_ref: Option<String>,
}
impl Policy {
    pub fn published(&self) -> Self {
        let mut policy = self.clone();
        policy.pending_removals.clear();
        policy.recovery_id = None;
        policy
    }
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            template_refs: vec![],
            channels: vec![],
            default_channel_id: None,
            pending_removals: vec![],
            recovery_id: None,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveTemplate {
    pub expected_library: String,
    pub template_id: Option<String>,
    pub base_revision: Option<String>,
    pub label: String,
    pub content: TemplateContent,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub library: Library,
    pub policy: Policy,
    pub library_digest: String,
    pub policy_digest: String,
    pub workspace: Option<PolicyDraft>,
    pub workspace_digest: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PolicyDraft {
    pub base_policy: String,
    pub policy: Policy,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    pub snapshot: Snapshot,
    pub template_id: String,
    pub revision_id: String,
    pub changed: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommitIdentity {
    pub template_id: String,
    pub revision_id: String,
    pub changed: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BatchCommit {
    pub snapshot: Snapshot,
    pub outcomes: Vec<CommitIdentity>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubmissionReceipt {
    pub digest: String,
    pub outcomes: Vec<CommitIdentity>,
    pub saved_at: String,
}
