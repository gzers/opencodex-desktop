//! Versioned model semantics. No file, credential, network or runtime access.
use super::schema::{
    ChannelMode, ModelNaming, ModelRules, ModelSelection, ServiceNode, SourceEvidence,
};
pub use super::schema::{FastWire, Field};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest as _, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const SCHEMA_VERSION: u32 = 2;
pub const CONTENT_VERSION: u32 = 2;
pub const DTO_VERSION: u32 = 2;
pub const RULE_KEYS: [&str; 11] = [
    "protocol",
    "contextWindow",
    "inputModalities",
    "reasoningEfforts",
    "defaultEffort",
    "fast",
    "pinnedEffort",
    "effortMap",
    "noReasoning",
    "reasoningAuthoritative",
    "fastWire",
];
pub type Result<T> = std::result::Result<T, MatchErrorV2>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, thiserror::Error)]
#[error("{code}")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MatchErrorV2 {
    pub code: String,
    pub issues: Vec<MatchIssueV2>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MatchIssueV2 {
    pub code: String,
    pub channel_id: Option<String>,
    pub raw_model_id: Option<String>,
    pub field_path: Option<String>,
    pub condition_ids: Vec<String>,
    pub reference: Option<RefV2>,
    pub message: String,
    pub recovery_actions: Vec<String>,
}
pub fn error(code: &str) -> MatchErrorV2 {
    let actions: &[&str] = match code {
        "model_schema_unsupported" | "model_unknown_field" => &["upgrade_reader", "inspect_source"],
        "model_source_provenance_unknown" => &["inspect_source", "reestablish_source"],
        "model_matching_conflict" | "model_condition_overlap" => {
            &["edit_matching", "choose_manual_template"]
        }
        "model_selector_invalid" | "model_selector_duplicate" => &["edit_conditions"],
        "model_template_default_missing" => &["set_saved_default", "choose_manual_template"],
        "model_dependency_missing" | "model_revision_digest_mismatch" => {
            &["restore_exact_revision", "choose_manual_template"]
        }
        "model_protocol_missing" | "model_protocol_unsupported" => {
            &["configure_model_protocol", "edit_template"]
        }
        "model_channel_protocol_forbidden" => &["remove_channel_protocol"],
        "model_read_stale" | "model_save_stale" => &["refresh_and_retry"],
        "model_limit" => &["reduce_scope"],
        "model_source_invalid" | "model_read_empty" | "model_read_unavailable" => {
            &["inspect_source", "retry_read"]
        }
        "model_locked_field" => &["review_locks"],
        _ => &["inspect_source"],
    };
    MatchErrorV2 {
        code: code.into(),
        issues: vec![MatchIssueV2 {
            code: code.into(),
            channel_id: None,
            raw_model_id: None,
            field_path: None,
            condition_ids: vec![],
            reference: None,
            message: "模型配置未通过核对".into(),
            recovery_actions: actions.iter().map(|s| (*s).into()).collect(),
        }],
    }
}
pub fn at(mut err: MatchErrorV2, channel: &str, model: &str) -> MatchErrorV2 {
    for issue in &mut err.issues {
        issue.channel_id = Some(channel.into());
        issue.raw_model_id = Some(model.into());
    }
    err
}
pub fn json_error(err: serde_json::Error) -> MatchErrorV2 {
    error(if err.to_string().contains("duplicate JSON object key") {
        "model_json_duplicate_key"
    } else if err.to_string().contains("unknown field") {
        "model_unknown_field"
    } else {
        "model_source_invalid"
    })
}
pub fn digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
pub fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    fn ordered(v: Value) -> Value {
        match v {
            Value::Object(map) => Value::Object(
                map.into_iter()
                    .map(|(k, v)| (k, ordered(v)))
                    .collect::<BTreeMap<_, _>>()
                    .into_iter()
                    .collect(),
            ),
            Value::Array(a) => Value::Array(a.into_iter().map(ordered).collect()),
            v => v,
        }
    }
    serde_json::to_vec(&ordered(serde_json::to_value(value).map_err(json_error)?))
        .map_err(json_error)
}
pub fn valid_digest(s: &str) -> bool {
    s.len() == 71
        && s.starts_with("sha256:")
        && s[7..]
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub fn valid_id(s: &str) -> bool {
    uuid::Uuid::parse_str(s).is_ok_and(|u| u.to_string() == s)
}
pub fn check_id(s: &str) -> Result<()> {
    if valid_id(s) {
        Ok(())
    } else {
        Err(error("model_source_invalid"))
    }
}
pub fn rules_values(rules: &ModelRules) -> Result<BTreeMap<String, Value>> {
    let Value::Object(map) = serde_json::to_value(rules).map_err(json_error)? else {
        unreachable!()
    };
    Ok(map
        .into_iter()
        .filter_map(|(k, v)| (v["mode"] == "custom").then(|| (k, v["value"].clone())))
        .collect())
}
pub fn rules_from_values(values: &BTreeMap<String, Value>) -> Result<ModelRules> {
    let mut map = serde_json::Map::new();
    for (k, v) in values {
        if !RULE_KEYS.contains(&k.as_str()) {
            return Err(error("model_unknown_field"));
        }
        map.insert(k.clone(), serde_json::json!({"mode":"custom","value":v}));
    }
    serde_json::from_value(Value::Object(map)).map_err(json_error)
}
pub fn validate_rules(rules: &ModelRules, allow_unknown_protocol: bool) -> Result<()> {
    let mut check = rules.clone();
    if let Field::Custom(p) = &check.protocol {
        if ProtocolV2::parse(p).is_none() && !allow_unknown_protocol {
            return Err(error("model_protocol_unsupported"));
        }
    }
    check.protocol = Field::Inherit;
    check.validate().map_err(|_| error("model_source_invalid"))
}
pub fn validate_non_protocol(rules: &ModelRules) -> Result<()> {
    if rules.protocol != Field::Inherit {
        return Err(error("model_channel_protocol_forbidden"));
    }
    validate_rules(rules, false)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RefV2 {
    pub template_id: String,
    pub revision_id: String,
    pub digest: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ModelBindingV2 {
    Manual { reference: RefV2 },
    AutoRead { reference: RefV2 },
}
impl ModelBindingV2 {
    pub fn reference(&self) -> &RefV2 {
        match self {
            Self::Manual { reference } | Self::AutoRead { reference } => reference,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub enum ProtocolV2 {
    #[serde(rename = "openai-chat")]
    OpenaiChat,
    #[serde(rename = "openai-responses")]
    OpenaiResponses,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "google")]
    Google,
    #[serde(rename = "ollama-native")]
    OllamaNative,
}
impl ProtocolV2 {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "openai-chat" => Self::OpenaiChat,
            "openai-responses" => Self::OpenaiResponses,
            "anthropic" => Self::Anthropic,
            "google" => Self::Google,
            "ollama-native" => Self::OllamaNative,
            _ => return None,
        })
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenaiChat => "openai-chat",
            Self::OpenaiResponses => "openai-responses",
            Self::Anthropic => "anthropic",
            Self::Google => "google",
            Self::OllamaNative => "ollama-native",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SourceKindV2 {
    ImportedFile,
    UpstreamRead,
    PortableImport,
    ExternalFileEdit,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceEvidenceV2 {
    pub kind: SourceKindV2,
    pub source_digest: String,
    pub provider_id: String,
    pub pointer: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceFieldV2 {
    pub value: Value,
    pub evidence: SourceEvidenceV2,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceModelV2 {
    pub raw_model_id: String,
    pub fields: BTreeMap<String, SourceFieldV2>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelConfigV2 {
    pub raw_model_id: String,
    pub label: String,
    pub observed: bool,
    pub source: SourceModelV2,
    pub explicit_overrides: ModelRules,
    pub binding: Option<ModelBindingV2>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum FieldOriginV2 {
    Source {
        evidence: SourceEvidenceV2,
    },
    TemplateCommon {
        reference: RefV2,
    },
    TemplateCondition {
        reference: RefV2,
        condition_id: String,
    },
    ChannelDefault,
    ChannelFamily {
        template_id: String,
    },
    ExplicitModel,
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProtocolStateV2 {
    Resolved {
        value: ProtocolV2,
        origin: FieldOriginV2,
    },
    Missing,
    Unsupported {
        value: String,
        origin: FieldOriginV2,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EffectiveModelV2 {
    pub channel_id: String,
    pub raw_model_id: String,
    pub selected: bool,
    pub enabled: bool,
    pub observed: bool,
    pub binding: Option<ModelBindingV2>,
    pub rules: ModelRules,
    pub protocol: ProtocolStateV2,
    pub field_origins: BTreeMap<String, FieldOriginV2>,
    pub matched_condition_id: Option<String>,
    pub notices: Vec<MatchIssueV2>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PatternKindV2 {
    Exact,
    Prefix,
    Glob,
    AsciiNumericPrefix,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PatternV2 {
    pub kind: PatternKindV2,
    pub value: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MatchRulesV2 {
    pub channel_mode: ChannelMode,
    pub channels: BTreeSet<String>,
    pub case_sensitive: bool,
    pub priority: i32,
    pub include: Vec<PatternV2>,
    pub exclude: Vec<PatternV2>,
    pub reject_repeated_literal_prefix: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NameLayoutV2 {
    pub id: String,
    pub literal_prefix: String,
    pub version_capture: String,
    pub numeric_joiners: Vec<String>,
    pub tail_capture: String,
    pub trailing_date_policy: String,
    pub ambiguous_numeric_tail: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LiteralPositionV2 {
    Any,
    BeforeVersion,
    AfterVersion,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SelectorAtomV2 {
    ExactId {
        value: String,
    },
    NumericVersion {
        components: Vec<String>,
    },
    LiteralSegment {
        value: String,
        position: LiteralPositionV2,
        boundary: String,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SelectorV2 {
    pub all: Vec<SelectorAtomV2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub literal_tail: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConditionOverrideV2 {
    pub id: String,
    pub name: String,
    pub selector: SelectorV2,
    pub rules: ModelRules,
    pub locked_fields: BTreeSet<String>,
    pub source_exact_id: String,
    pub scope: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TemplateContentV2 {
    pub description: String,
    pub matching: MatchRulesV2,
    pub name_layouts: Vec<NameLayoutV2>,
    pub common_rules: ModelRules,
    pub condition_overrides: Vec<ConditionOverrideV2>,
    pub locked_fields: BTreeSet<String>,
    pub sources: Vec<SourceEvidence>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RevisionV2 {
    pub id: String,
    pub created_at: String,
    pub saved_label: String,
    pub content_version: u32,
    pub content: TemplateContentV2,
    pub digest: String,
    pub base_revision: Option<String>,
    pub changes: Vec<super::schema::ChangeFact>,
    pub origin: super::schema::SaveOrigin,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TemplateV2 {
    pub id: String,
    pub label: String,
    pub source: super::schema::TemplateSource,
    pub default_revision: Option<String>,
    pub revisions: Vec<RevisionV2>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LibraryV2 {
    pub schema_version: u32,
    pub templates: Vec<TemplateV2>,
    #[serde(default)]
    pub submission_receipts: BTreeMap<String, super::schema::SubmissionReceipt>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConnectionV2 {
    pub provider_id: String,
    pub nodes: Vec<ServiceNode>,
    pub active_node_id: String,
    pub credential_ref: Option<String>,
    pub enabled: bool,
    pub allow_private_network: bool,
    pub auth_mode: Option<String>,
    pub source_read_id: Option<String>,
    pub imported_config: Option<super::schema::ImportedProviderRef>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NamespaceV2 {
    pub channel_id: String,
    pub remove_leading_prefixes: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelV2 {
    pub id: String,
    pub label: String,
    pub connection: ConnectionV2,
    pub selection: ModelSelection,
    pub model_order: Vec<String>,
    pub models: Vec<ModelConfigV2>,
    pub template_refs: Vec<RefV2>,
    pub defaults: ModelRules,
    pub family_overrides: BTreeMap<String, ModelRules>,
    pub naming: BTreeMap<String, ModelNaming>,
    pub namespace: Option<NamespaceV2>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PolicyV2 {
    pub schema_version: u32,
    pub channels: Vec<ChannelV2>,
    pub template_refs: Vec<RefV2>,
    pub default_channel_id: Option<String>,
    pub pending_removals: Vec<super::schema::SourceRemoval>,
    pub recovery_id: Option<String>,
}

pub fn content_digest(label: &str, content: &TemplateContentV2) -> Result<String> {
    Ok(digest(&canonical_bytes(
        &serde_json::json!({"label":label,"contentVersion":CONTENT_VERSION,"content":content}),
    )?))
}
pub fn revision<'a>(library: &'a LibraryV2, reference: &RefV2) -> Result<&'a RevisionV2> {
    if library.schema_version != SCHEMA_VERSION {
        return Err(error("model_schema_unsupported"));
    }
    let template = library
        .templates
        .iter()
        .find(|t| t.id == reference.template_id)
        .ok_or_else(|| error("model_dependency_missing"))?;
    let revision = template
        .revisions
        .iter()
        .find(|r| r.id == reference.revision_id)
        .ok_or_else(|| error("model_dependency_missing"))?;
    if revision.content_version != CONTENT_VERSION {
        return Err(error("model_schema_unsupported"));
    }
    if reference.digest != revision.digest
        || content_digest(&revision.saved_label, &revision.content)? != revision.digest
    {
        return Err(error("model_revision_digest_mismatch"));
    }
    super::matching_v2::validate_content_v2(&revision.content)?;
    Ok(revision)
}
pub fn validate_library_v2(library: &LibraryV2) -> Result<()> {
    if library.schema_version != SCHEMA_VERSION {
        return Err(error("model_schema_unsupported"));
    }
    if library.templates.len() > 4096 {
        return Err(error("model_limit"));
    }
    let mut ids = BTreeSet::new();
    let mut revision_ids = BTreeSet::new();
    for template in &library.templates {
        check_id(&template.id)?;
        if !ids.insert(&template.id) || template.revisions.len() > 4096 {
            return Err(error("model_source_invalid"));
        }
        for r in &template.revisions {
            check_id(&r.id)?;
            if !revision_ids.insert(&r.id) || r.content_version != CONTENT_VERSION {
                return Err(error("model_schema_unsupported"));
            }
            chrono::DateTime::parse_from_rfc3339(&r.created_at)
                .map_err(|_| error("model_source_invalid"))?;
            super::matching_v2::validate_content_v2(&r.content)?;
            if content_digest(&r.saved_label, &r.content)? != r.digest {
                return Err(error("model_revision_digest_mismatch"));
            }
        }
        if template
            .default_revision
            .as_ref()
            .is_some_and(|id| !template.revisions.iter().any(|r| &r.id == id))
        {
            return Err(error("model_dependency_missing"));
        }
    }
    Ok(())
}
pub fn seed_library_v2() -> Result<LibraryV2> {
    let library = serde_json::from_value(parse_strict_json_v2(include_bytes!("seed_v2.json"))?)
        .map_err(json_error)?;
    validate_library_v2(&library)?;
    Ok(library)
}

/// Immutable revision append plan; IDs and time are supplied by the transaction owner.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveRevisionRequestV2 {
    pub expected_library: String,
    pub template_id: String,
    pub base_revision: Option<String>,
    pub revision_id: String,
    pub created_at: String,
    pub label: String,
    pub content: TemplateContentV2,
    pub make_default: bool,
    pub origin: super::schema::SaveOrigin,
}
pub fn save_revision_plan_v2(
    library: &LibraryV2,
    request: &SaveRevisionRequestV2,
) -> Result<LibraryV2> {
    validate_library_v2(library)?;
    if request.expected_library != digest(&canonical_bytes(library)?) {
        return Err(error("model_save_stale"));
    }
    check_id(&request.template_id)?;
    check_id(&request.revision_id)?;
    if request.label.trim().is_empty() || request.label.len() > 1024 {
        return Err(error("model_source_invalid"));
    }
    chrono::DateTime::parse_from_rfc3339(&request.created_at)
        .map_err(|_| error("model_source_invalid"))?;
    super::matching_v2::validate_content_v2(&request.content)?;
    if library
        .templates
        .iter()
        .flat_map(|t| &t.revisions)
        .any(|r| r.id == request.revision_id)
    {
        return Err(error("model_revision_digest_mismatch"));
    }
    let existing = library
        .templates
        .iter()
        .find(|t| t.id == request.template_id);
    match (existing, &request.base_revision) {
        (Some(t), Some(id)) if t.revisions.iter().any(|r| &r.id == id) => {}
        (None, None) => {}
        _ => return Err(error("model_dependency_missing")),
    }
    if let Some(t) = existing {
        if matches!(
            request.origin,
            super::schema::SaveOrigin::ExternalTask(_)
                | super::schema::SaveOrigin::InternalAgent(_)
        ) {
            let old = &t
                .revisions
                .iter()
                .find(|r| Some(&r.id) == request.base_revision.as_ref())
                .unwrap()
                .content;
            let old_json = serde_json::to_value(old).map_err(json_error)?;
            let new_json = serde_json::to_value(&request.content).map_err(json_error)?;
            for locked in &old.locked_fields {
                let path = if locked == "rules" {
                    "/commonRules".into()
                } else if locked == "variants" {
                    "/conditionOverrides".into()
                } else if let Some(key) = locked.strip_prefix("rules.") {
                    format!("/commonRules/{key}")
                } else {
                    format!("/{}", locked.replace('.', "/"))
                };
                if old_json.pointer(&path) != new_json.pointer(&path) {
                    return Err(error("model_locked_field"));
                }
            }
            for condition in &old.condition_overrides {
                if condition.locked_fields.is_empty() {
                    continue;
                }
                let next = request
                    .content
                    .condition_overrides
                    .iter()
                    .find(|c| c.id == condition.id)
                    .ok_or_else(|| error("model_locked_field"))?;
                let old = rules_values(&condition.rules)?;
                let new = rules_values(&next.rules)?;
                if condition
                    .locked_fields
                    .iter()
                    .any(|key| old.get(key) != new.get(key))
                    || next.locked_fields != condition.locked_fields
                {
                    return Err(error("model_locked_field"));
                }
            }
            if request.content.locked_fields != old.locked_fields {
                return Err(error("model_locked_field"));
            }
        }
    }
    let prior = existing.and_then(|t| {
        t.revisions
            .iter()
            .find(|r| Some(&r.id) == request.base_revision.as_ref())
    });
    let mut changes = Vec::new();
    let old = prior
        .map(|r| serde_json::json!({"label":r.saved_label,"content":r.content}))
        .unwrap_or(Value::Null);
    let new = serde_json::json!({"label":request.label,"content":request.content});
    if old != new {
        changes.push(super::schema::ChangeFact {
            field: "content".into(),
            before: old,
            after: new,
        });
    }
    let revision = RevisionV2 {
        id: request.revision_id.clone(),
        created_at: request.created_at.clone(),
        saved_label: request.label.clone(),
        content_version: CONTENT_VERSION,
        content: request.content.clone(),
        digest: content_digest(&request.label, &request.content)?,
        base_revision: request.base_revision.clone(),
        changes,
        origin: request.origin.clone(),
    };
    let mut next = library.clone();
    if let Some(t) = next
        .templates
        .iter_mut()
        .find(|t| t.id == request.template_id)
    {
        t.revisions.push(revision);
        if request.make_default {
            t.default_revision = Some(request.revision_id.clone());
        }
    } else {
        next.templates.push(TemplateV2 {
            id: request.template_id.clone(),
            label: request.label.clone(),
            source: super::schema::TemplateSource::User,
            default_revision: request.make_default.then(|| request.revision_id.clone()),
            revisions: vec![revision],
        });
    }
    validate_library_v2(&next)?;
    Ok(next)
}

/// Strict recursive JSON decoder shared by every v2 file/package boundary.
/// serde_json::Value alone would silently keep the last duplicate object key.
pub fn parse_strict_json_v2(bytes: &[u8]) -> Result<Value> {
    if bytes.len() > 32 * 1024 * 1024 {
        return Err(error("model_limit"));
    }
    struct Unique(Value);
    impl<'de> Deserialize<'de> for Unique {
        fn deserialize<D: serde::Deserializer<'de>>(
            decoder: D,
        ) -> std::result::Result<Self, D::Error> {
            struct Visitor;
            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = Unique;
                fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str("JSON with unique object keys")
                }
                fn visit_bool<E: serde::de::Error>(
                    self,
                    value: bool,
                ) -> std::result::Result<Unique, E> {
                    Ok(Unique(Value::Bool(value)))
                }
                fn visit_i64<E: serde::de::Error>(
                    self,
                    value: i64,
                ) -> std::result::Result<Unique, E> {
                    Ok(Unique(Value::Number(value.into())))
                }
                fn visit_u64<E: serde::de::Error>(
                    self,
                    value: u64,
                ) -> std::result::Result<Unique, E> {
                    Ok(Unique(Value::Number(value.into())))
                }
                fn visit_f64<E: serde::de::Error>(
                    self,
                    value: f64,
                ) -> std::result::Result<Unique, E> {
                    serde_json::Number::from_f64(value)
                        .map(|n| Unique(Value::Number(n)))
                        .ok_or_else(|| E::custom("invalid JSON number"))
                }
                fn visit_str<E: serde::de::Error>(
                    self,
                    value: &str,
                ) -> std::result::Result<Unique, E> {
                    Ok(Unique(Value::String(value.into())))
                }
                fn visit_string<E: serde::de::Error>(
                    self,
                    value: String,
                ) -> std::result::Result<Unique, E> {
                    Ok(Unique(Value::String(value)))
                }
                fn visit_unit<E: serde::de::Error>(self) -> std::result::Result<Unique, E> {
                    Ok(Unique(Value::Null))
                }
                fn visit_none<E: serde::de::Error>(self) -> std::result::Result<Unique, E> {
                    Ok(Unique(Value::Null))
                }
                fn visit_seq<A: serde::de::SeqAccess<'de>>(
                    self,
                    mut sequence: A,
                ) -> std::result::Result<Unique, A::Error> {
                    let mut values = Vec::new();
                    while let Some(Unique(value)) = sequence.next_element::<Unique>()? {
                        values.push(value);
                    }
                    Ok(Unique(Value::Array(values)))
                }
                fn visit_map<A: serde::de::MapAccess<'de>>(
                    self,
                    mut object: A,
                ) -> std::result::Result<Unique, A::Error> {
                    let mut values = serde_json::Map::new();
                    while let Some(key) = object.next_key::<String>()? {
                        if values.contains_key(&key) {
                            return Err(serde::de::Error::custom("duplicate JSON object key"));
                        }
                        let Unique(value) = object.next_value::<Unique>()?;
                        values.insert(key, value);
                    }
                    Ok(Unique(Value::Object(values)))
                }
            }
            decoder.deserialize_any(Visitor)
        }
    }
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    let Unique(value) = Unique::deserialize(&mut decoder).map_err(json_error)?;
    decoder.end().map_err(json_error)?;
    Ok(value)
}
