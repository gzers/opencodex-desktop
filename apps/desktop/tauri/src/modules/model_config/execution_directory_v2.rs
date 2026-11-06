//! Actual saved configuration + official live catalog, joined by stable v2
//! logical/group identity. Never reads draft as a source or invents selectors.
use super::{
    agent::AgentSettings,
    matching_v2::resolve_effective_model_v2,
    protocol_groups::{validate_mappings_v2, MappingFileV2},
    schema::{Field, ModelRules},
    schema_v2::*,
    source_v2::{plan_protocol_groups_v2, validate_policy_v2},
    upstream::UpstreamSnapshot,
};
use crate::state::ProcessContext;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DirectionFactsV2 {
    adapter: String,
    base_url: String,
    auth_mode: String,
    api_key_transport: Option<String>,
    responses_path: Option<String>,
    chat_completions_path: Option<String>,
}
fn direction(value: &Value) -> Result<DirectionFactsV2> {
    let text = |key: &str| -> Result<Option<String>> {
        value
            .get(key)
            .map(|v| {
                v.as_str()
                    .map(String::from)
                    .ok_or_else(|| error("model_execution_direction_invalid"))
            })
            .transpose()
    };
    Ok(DirectionFactsV2 {
        adapter: text("adapter")?.unwrap_or_default(),
        base_url: text("baseUrl")?.unwrap_or_default(),
        auth_mode: text("authMode")?.unwrap_or_else(|| "key".into()),
        api_key_transport: text("apiKeyTransport")?,
        responses_path: text("responsesPath")?,
        chat_completions_path: text("chatCompletionsPath")?,
    })
}
fn same_direction(saved: &DirectionFactsV2, live: &DirectionFactsV2) -> bool {
    saved.adapter == live.adapter
        && saved.base_url == live.base_url
        && saved.auth_mode == live.auth_mode
        && saved
            .api_key_transport
            .as_ref()
            .is_none_or(|v| live.api_key_transport.as_ref() == Some(v))
        && saved
            .responses_path
            .as_ref()
            .is_none_or(|v| live.responses_path.as_ref() == Some(v))
        && saved
            .chat_completions_path
            .as_ref()
            .is_none_or(|v| live.chat_completions_path.as_ref() == Some(v))
}
/// Internal owned capture, not deserializable or sent wholesale to the UI/model.
#[derive(Debug, Clone)]
pub struct SavedExecutionV2 {
    pub(crate) library: LibraryV2,
    pub(crate) policy: PolicyV2,
    pub(crate) mappings: MappingFileV2,
    pub source_digest: String,
    pub root_revision: String,
    pub runtime_binding: String,
    directions: BTreeMap<String, DirectionFactsV2>,
}
impl SavedExecutionV2 {
    pub fn library(&self) -> &LibraryV2 {
        &self.library
    }
    pub fn library_digest(&self) -> Result<String> {
        Ok(digest(&canonical_bytes(&self.library)?))
    }
}
/// Only this adapter obtains production facts; its source closure owns the
/// config→model locks and returns before any CLI/HTTP await is possible.
pub fn capture_saved(root: &Path, context: &ProcessContext) -> Result<SavedExecutionV2> {
    let runtime_binding = super::runtime::runtime_binding(context).map_err(|e| error(&e.code))?;
    let saved = super::config_file::v2::with_saved_config(
        root,
        &context.opencodex_home,
        |library, policy, mappings, raw, source, revision| {
            from_saved_facts(
                library,
                policy,
                mappings,
                raw,
                source,
                revision,
                &runtime_binding,
            )
        },
    )?;
    if super::runtime::runtime_binding(context).map_err(|e| error(&e.code))? != runtime_binding {
        return Err(error("model_execution_binding_stale"));
    }
    Ok(saved)
}
fn from_saved_facts(
    library: &LibraryV2,
    policy: &PolicyV2,
    mappings: &MappingFileV2,
    raw: &Value,
    source: &str,
    root_revision: &str,
    runtime_binding: &str,
) -> Result<SavedExecutionV2> {
    validate_library_v2(library)?;
    validate_policy_v2(policy)?;
    validate_mappings_v2(mappings)?;
    if !valid_digest(source) || !valid_digest(root_revision) || !valid_digest(runtime_binding) {
        return Err(error("model_execution_binding_invalid"));
    }
    let mut directions = BTreeMap::new();
    if let Some(providers) = raw.get("providers") {
        for (id, row) in providers
            .as_object()
            .ok_or_else(|| error("model_execution_direction_invalid"))?
        {
            directions.insert(id.clone(), direction(row)?);
        }
    }
    Ok(SavedExecutionV2 {
        library: library.clone(),
        policy: policy.clone(),
        mappings: mappings.clone(),
        source_digest: source.into(),
        root_revision: root_revision.into(),
        runtime_binding: runtime_binding.into(),
        directions,
    })
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionChannelV2 {
    pub id: String,
    pub label: String,
    pub provider_id: String,
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionRowV2 {
    pub channel_id: String,
    pub provider_id: String,
    pub model_id: String,
    pub label: String,
    pub rules: ModelRules,
    pub protocol: ProtocolStateV2,
    pub selected: bool,
    pub enabled: bool,
    pub native_model_id: Option<String>,
    pub unavailable_reason: Option<String>,
    pub aliases: Vec<String>,
    pub connection_digest: Option<String>,
    pub picker_visible: bool,
    pub actual_selector: Option<String>,
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionDirectoryV2 {
    pub dto_version: u32,
    pub source_digest: String,
    pub root_revision: String,
    pub library_digest: String,
    pub channels: Vec<ExecutionChannelV2>,
    pub rows: Vec<ExecutionRowV2>,
}
/// Live-only capability projection, intersected with explicit saved denials.
fn actual_rules(row: &Value, effective: Option<&EffectiveModelV2>) -> Result<ModelRules> {
    let mut rules = ModelRules::default();
    if let Some(context) = row
        .get("contextWindow")
        .and_then(Value::as_u64)
        .filter(|c| *c > 0)
    {
        rules.context_window = Field::Custom(context);
    }
    for (key, target) in [
        ("inputModalities", &mut rules.input_modalities),
        ("reasoningEfforts", &mut rules.reasoning_efforts),
    ] {
        if let Some(values) = row.get(key) {
            let values = values
                .as_array()
                .ok_or_else(|| error("model_execution_capabilities_invalid"))?
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(String::from)
                        .ok_or_else(|| error("model_execution_capabilities_invalid"))
                })
                .collect::<Result<Vec<_>>>()?;
            *target = Field::Custom(values);
        }
    }
    if let Some(value) = row.get("defaultReasoningEffort").and_then(Value::as_str) {
        rules.default_effort = Field::Custom(value.into());
    }
    if let Some(value) = row.get("fastRowAvailable").and_then(Value::as_bool) {
        rules.fast = Field::Custom(value);
    }
    if let Some(effective) = effective {
        if let (Field::Custom(actual), Field::Custom(allowed)) = (
            &mut rules.reasoning_efforts,
            &effective.rules.reasoning_efforts,
        ) {
            actual.retain(|v| allowed.contains(v));
        }
        if effective.rules.fast == Field::Custom(false) {
            rules.fast = Field::Custom(false);
        }
        if matches!((&rules.default_effort,&rules.reasoning_efforts),(Field::Custom(default),Field::Custom(values)) if !values.contains(default))
        {
            rules.default_effort = Field::Inherit;
        }
        rules.protocol = effective.rules.protocol.clone();
    }
    validate_rules(&rules, true)?;
    Ok(rules)
}
pub fn build(saved: &SavedExecutionV2, live: &UpstreamSnapshot) -> Result<ExecutionDirectoryV2> {
    let live_providers = live
        .providers
        .get("providers")
        .and_then(Value::as_object)
        .ok_or_else(|| error("model_execution_directory_invalid"))?;
    let live_rows = live
        .models
        .as_array()
        .ok_or_else(|| error("model_execution_directory_invalid"))?;
    let picker = live
        .order
        .get("pickerOrder")
        .map(|v| {
            v.as_array()
                .ok_or_else(|| error("model_execution_directory_invalid"))
        })
        .transpose()?
        .map(|a| {
            a.iter()
                .map(|v| {
                    v.as_str()
                        .map(String::from)
                        .ok_or_else(|| error("model_execution_directory_invalid"))
                })
                .collect::<Result<BTreeSet<_>>>()
        })
        .transpose()?;
    let mut channels = saved
        .policy
        .channels
        .iter()
        .map(|c| ExecutionChannelV2 {
            id: c.id.clone(),
            label: c.label.clone(),
            provider_id: c.connection.provider_id.clone(),
        })
        .collect::<Vec<_>>();
    let mut rows = vec![];
    let mut seen = BTreeSet::new();
    let mut native_seen = false;
    for row in live_rows {
        if rows.len() >= 20_000 {
            return Err(error("model_limit"));
        }
        let id = row
            .get("id")
            .and_then(Value::as_str)
            .filter(|v| !v.is_empty() && v.len() <= 2048)
            .ok_or_else(|| error("model_execution_directory_invalid"))?;
        let provider = row
            .get("provider")
            .and_then(Value::as_str)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| error("model_execution_directory_invalid"))?;
        let native = row.get("native") == Some(&Value::Bool(true));
        let mapping = saved
            .mappings
            .mappings
            .iter()
            .find(|m| m.provider_id == provider);
        let channel = if let Some(mapping) = mapping {
            saved
                .policy
                .channels
                .iter()
                .find(|c| c.id == mapping.channel_id)
        } else {
            saved
                .policy
                .channels
                .iter()
                .find(|c| c.connection.provider_id == provider)
        };
        if channel.is_none() && !native {
            continue;
        }
        let channel_id = channel.map(|c| c.id.as_str()).unwrap_or("native");
        if !seen.insert((channel_id.to_string(), id.to_string())) {
            return Err(error("model_execution_identity_collision"));
        }
        let model = channel.and_then(|c| c.models.iter().find(|m| m.raw_model_id == id));
        let resolved = channel
            .zip(model)
            .map(|(c, m)| resolve_effective_model_v2(&saved.library, c, m));
        let effective = resolved.as_ref().and_then(|r| r.as_ref().ok());
        let protocol = effective
            .map(|e| e.protocol.clone())
            .unwrap_or(ProtocolStateV2::Missing);
        let selector = row
            .get("namespaced")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(String::from);
        let mut reason = if native {
            native_seen = true;
            Some("model_execution_native_unbound".into())
        } else if provider.to_ascii_lowercase().contains("a6api") {
            Some("model_execution_provider_unavailable".into())
        } else if model.is_none() {
            Some("model_execution_not_saved".into())
        } else if resolved.as_ref().is_some_and(|r| r.is_err()) {
            Some("model_execution_effective_invalid".into())
        } else if !channel.unwrap().connection.enabled
            || live_providers
                .get(provider)
                .is_some_and(|p| p["disabled"] == true)
        {
            Some("model_execution_channel_disabled".into())
        } else if effective.is_some_and(|e| !e.selected)
            || row["disabled"] == true
            || row["initialSelectionPending"] == true
        {
            Some("model_execution_model_hidden".into())
        } else if selector.is_none() {
            Some("model_execution_selector_missing".into())
        } else if picker
            .as_ref()
            .is_some_and(|visible| !visible.contains(selector.as_deref().unwrap_or(id)))
        {
            Some("model_execution_model_hidden".into())
        } else {
            None
        };
        let live_direction = live_providers.get(provider).map(direction).transpose()?;
        if reason.is_none() {
            match (saved.directions.get(provider), &live_direction) {
                (Some(source), Some(actual)) if same_direction(source, actual) => {}
                _ => reason = Some("model_execution_direction_changed".into()),
            }
        }
        if reason.is_none() {
            let e = effective.unwrap();
            match &e.protocol {
                ProtocolStateV2::Resolved { value, .. } => {
                    let group = plan_protocol_groups_v2(std::slice::from_ref(e))?
                        .into_iter()
                        .next()
                        .ok_or_else(|| error("model_execution_binding_invalid"))?;
                    if live_direction
                        .as_ref()
                        .is_none_or(|d| d.adapter != value.as_str())
                        || mapping.is_some_and(|m| {
                            m.protocol != *value
                                || m.wire_profile_digest != group.wire_profile_digest
                        })
                    {
                        reason = Some("model_execution_protocol_changed".into());
                    }
                }
                _ => reason = Some("model_execution_protocol_unresolved".into()),
            }
        }
        let rules = actual_rules(row, effective)?;
        let authority = if reason.is_none() {
            Some(digest(&canonical_bytes(&(
                2,
                &saved.root_revision,
                &saved.runtime_binding,
                channel_id,
                provider,
                id,
                &selector,
                &protocol,
                &rules,
                saved.directions.get(provider),
                &live_direction,
            ))?))
        } else {
            None
        };
        let aliases = live_providers
            .get(provider)
            .and_then(|p| p["modelAliases"].get(id))
            .and_then(Value::as_str)
            .map(|a| vec![a.to_string()])
            .unwrap_or_default();
        rows.push(ExecutionRowV2 {
            channel_id: channel_id.into(),
            provider_id: provider.into(),
            model_id: id.into(),
            label: if native {
                row.get("displayName")
                    .and_then(Value::as_str)
                    .unwrap_or(id)
                    .into()
            } else {
                format!("{}/{id}", channel.unwrap().label)
            },
            rules,
            protocol,
            selected: effective.is_some_and(|e| e.selected),
            enabled: reason.is_none(),
            native_model_id: native.then(|| id.to_string()),
            unavailable_reason: reason.clone(),
            aliases,
            connection_digest: authority,
            picker_visible: reason.is_none(),
            actual_selector: selector,
        });
    }
    for channel in &saved.policy.channels {
        for model in &channel.models {
            if seen.contains(&(channel.id.clone(), model.raw_model_id.clone())) {
                continue;
            }
            if rows.len() >= 20_000 {
                return Err(error("model_limit"));
            }
            let effective = resolve_effective_model_v2(&saved.library, channel, model).ok();
            let provider = effective
                .as_ref()
                .and_then(|e| plan_protocol_groups_v2(std::slice::from_ref(e)).ok())
                .and_then(|groups| groups.into_iter().next())
                .and_then(|group| {
                    saved.mappings.mappings.iter().find(|m| {
                        m.channel_id == channel.id
                            && m.protocol == group.protocol
                            && m.wire_profile_digest == group.wire_profile_digest
                    })
                })
                .map(|m| m.provider_id.clone())
                .unwrap_or_else(|| channel.connection.provider_id.clone());
            rows.push(ExecutionRowV2 {
                channel_id: channel.id.clone(),
                provider_id: provider,
                model_id: model.raw_model_id.clone(),
                label: format!("{}/{}", channel.label, model.raw_model_id),
                rules: effective
                    .as_ref()
                    .map(|e| e.rules.clone())
                    .unwrap_or_default(),
                protocol: effective
                    .as_ref()
                    .map(|e| e.protocol.clone())
                    .unwrap_or(ProtocolStateV2::Missing),
                selected: effective.as_ref().is_some_and(|e| e.selected),
                enabled: false,
                native_model_id: None,
                unavailable_reason: Some("model_execution_not_live".into()),
                aliases: vec![],
                connection_digest: None,
                picker_visible: false,
                actual_selector: None,
            });
        }
    }
    if native_seen {
        channels.push(ExecutionChannelV2 {
            id: "native".into(),
            label: "官方 / 原生".into(),
            provider_id: "native".into(),
        });
    }
    Ok(ExecutionDirectoryV2 {
        dto_version: 2,
        source_digest: saved.source_digest.clone(),
        root_revision: saved.root_revision.clone(),
        library_digest: saved.library_digest()?,
        channels,
        rows,
    })
}
#[derive(Debug, Clone)]
pub struct BoundExecutionV2 {
    settings: AgentSettings,
    pub source_digest: String,
    pub root_revision: String,
    pub runtime_binding: String,
    selector: String,
}
impl BoundExecutionV2 {
    pub fn settings(&self) -> &AgentSettings {
        &self.settings
    }
    pub fn selector(&self) -> &str {
        &self.selector
    }
}
pub fn bind(
    saved: &SavedExecutionV2,
    live: &UpstreamSnapshot,
    mut settings: AgentSettings,
) -> Result<BoundExecutionV2> {
    let directory = build(saved, live)?;
    let row = directory
        .rows
        .iter()
        .find(|row| {
            row.provider_id == settings.provider_id
                && row.model_id == settings.model_id
                && row.native_model_id.is_none()
        })
        .ok_or_else(|| error("model_execution_model_missing"))?;
    if let Some(reason) = &row.unavailable_reason {
        return Err(error(reason));
    }
    if settings
        .channel_id
        .as_ref()
        .is_some_and(|id| id != &row.channel_id)
        || settings
            .connection_digest
            .as_ref()
            .is_some_and(|digest| row.connection_digest.as_ref() != Some(digest))
    {
        return Err(error("model_execution_binding_stale"));
    }
    if settings.reasoning_effort.as_ref().is_some_and(|effort|!matches!(&row.rules.reasoning_efforts,Field::Custom(values) if values.contains(effort))) {return Err(error("model_execution_reasoning_unavailable"));}
    if settings.fast == Some(true) && row.rules.fast != Field::Custom(true) {
        return Err(error("model_execution_fast_unavailable"));
    }
    settings.channel_id = Some(row.channel_id.clone());
    settings.connection_digest = row.connection_digest.clone();
    Ok(BoundExecutionV2 {
        settings,
        source_digest: saved.source_digest.clone(),
        root_revision: saved.root_revision.clone(),
        runtime_binding: saved.runtime_binding.clone(),
        selector: row
            .actual_selector
            .clone()
            .ok_or_else(|| error("model_execution_selector_missing"))?,
    })
}
pub fn validate_binding(
    context: &ProcessContext,
    current: &SavedExecutionV2,
    live: &UpstreamSnapshot,
    bound: &BoundExecutionV2,
) -> Result<()> {
    if super::runtime::runtime_binding(context).map_err(|e| error(&e.code))?
        != bound.runtime_binding
        || current.root_revision != bound.root_revision
        || current.source_digest != bound.source_digest
    {
        return Err(error("model_execution_binding_stale"));
    }
    let actual = bind(current, live, bound.settings.clone())?;
    if actual.selector != bound.selector || actual.runtime_binding != bound.runtime_binding {
        return Err(error("model_execution_binding_stale"));
    }
    Ok(())
}
#[cfg(test)]
#[path = "execution_directory_v2_tests.rs"]
mod tests;
