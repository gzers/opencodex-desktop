//! Pure config.json mapping. The caller owns file IO, credentials and UI metadata.
//! Never log/debug the input Value: it can contain credentials and unknown secrets.
//! Contract references: official 2.84 `types/provider.ts`, `providers/new-model-policy.ts`,
//! `reasoning-effort.ts::modelRecordValue`, `types/tools.ts::modelInList`, and
//! `providers/slug-codec.ts`. No registry, discovery or generated catalog is consulted.
use super::{matching, schema::*};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy)]
struct RuleField {
    name: &'static str,
    provider: Option<&'static str>,
    model: Option<&'static str>,
}
const FIELDS: &[RuleField] = &[
    RuleField {
        name: "protocol",
        provider: None,
        model: Some("modelAdapters"),
    },
    RuleField {
        name: "contextWindow",
        provider: Some("contextWindow"),
        model: Some("modelContextWindows"),
    },
    RuleField {
        name: "inputModalities",
        provider: None,
        model: Some("modelInputModalities"),
    },
    RuleField {
        name: "reasoningEfforts",
        provider: Some("reasoningEfforts"),
        model: Some("modelReasoningEfforts"),
    },
    RuleField {
        name: "defaultEffort",
        provider: None,
        model: Some("modelDefaultReasoningEfforts"),
    },
    RuleField {
        name: "fast",
        provider: Some("supportsServiceTier"),
        model: Some("modelSupportsServiceTier"),
    },
    RuleField {
        name: "pinnedEffort",
        provider: Some("pinnedReasoningEffort"),
        model: Some("modelPinnedReasoningEfforts"),
    },
    RuleField {
        name: "effortMap",
        provider: Some("reasoningEffortMap"),
        model: Some("modelReasoningEffortMap"),
    },
    RuleField {
        name: "noReasoning",
        provider: None,
        model: None,
    },
    RuleField {
        name: "reasoningAuthoritative",
        provider: Some("modelReasoningEffortsAuthoritative"),
        model: None,
    },
    RuleField {
        name: "fastWire",
        provider: Some("fastWire"),
        model: None,
    },
];

fn document_error() -> ModelError {
    invalid("config_document", "模型配置文件的渠道或模型字段结构不正确")
}
fn object(value: &Value) -> ModelResult<&Map<String, Value>> {
    value.as_object().ok_or_else(document_error)
}
fn strings(value: Option<&Value>) -> ModelResult<Vec<String>> {
    match value {
        None => Ok(vec![]),
        Some(value) => value
            .as_array()
            .ok_or_else(document_error)?
            .iter()
            .map(|v| v.as_str().map(str::to_owned).ok_or_else(document_error))
            .collect(),
    }
}
fn read_field<T: DeserializeOwned>(value: Option<&Value>) -> ModelResult<Field<T>> {
    value
        .map(|v| {
            serde_json::from_value(v.clone())
                .map(Field::Custom)
                .map_err(|_| document_error())
        })
        .unwrap_or(Ok(Field::Inherit))
}
fn read_fast_wire(value: Option<&Value>) -> ModelResult<Field<Option<FastWire>>> {
    match value {
        Some(Value::Object(raw)) => {
            // Future wire properties remain opaque in the original document.
            let known = Value::Object(
                raw.iter()
                    .filter(|(key, _)| {
                        ["kind", "canonicalToWire", "foreignCallerTiers", "betas"]
                            .contains(&key.as_str())
                    })
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
            );
            read_field(Some(&known))
        }
        other => read_field(other),
    }
}
fn custom<T: Serialize>(field: &Field<T>) -> Option<Value> {
    match field {
        Field::Inherit => None,
        // All ModelRules payloads are JSON-representable, without secrets.
        Field::Custom(value) => Some(serde_json::to_value(value).expect("model rule JSON")),
    }
}
fn rule_value(rules: &ModelRules, field: &str) -> Option<Value> {
    match field {
        "protocol" => custom(&rules.protocol),
        "contextWindow" => custom(&rules.context_window),
        "inputModalities" => custom(&rules.input_modalities),
        "reasoningEfforts" => custom(&rules.reasoning_efforts),
        "defaultEffort" => custom(&rules.default_effort),
        "fast" => custom(&rules.fast),
        "pinnedEffort" => custom(&rules.pinned_effort),
        "effortMap" => custom(&rules.effort_map),
        "noReasoning" => custom(&rules.no_reasoning),
        "reasoningAuthoritative" => custom(&rules.reasoning_authoritative),
        "fastWire" => custom(&rules.fast_wire),
        _ => None,
    }
}
fn assign_rule(rules: &mut ModelRules, field: &str, value: Option<&Value>) -> ModelResult<()> {
    match field {
        "protocol" => rules.protocol = read_field(value)?,
        "contextWindow" => rules.context_window = read_field(value)?,
        "inputModalities" => rules.input_modalities = read_field(value)?,
        "reasoningEfforts" => rules.reasoning_efforts = read_field(value)?,
        "defaultEffort" => rules.default_effort = read_field(value)?,
        "fast" => rules.fast = read_field(value)?,
        "pinnedEffort" => rules.pinned_effort = read_field(value)?,
        "effortMap" => rules.effort_map = read_field(value)?,
        "noReasoning" => rules.no_reasoning = read_field(value)?,
        "reasoningAuthoritative" => rules.reasoning_authoritative = read_field(value)?,
        "fastWire" => rules.fast_wire = read_fast_wire(value)?,
        _ => {}
    }
    Ok(())
}

fn stable_id(kind: &str, provider: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(b"opencodex-desktop/config-file/v1/");
    hash.update(kind.as_bytes());
    hash.update([0]);
    hash.update(provider.as_bytes());
    let digest = hash.finalize();
    let mut bytes = [0; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    uuid::Uuid::from_bytes(bytes).to_string()
}
fn keep_id(value: &str) -> bool {
    require_uuid(value).is_ok()
}
fn append_unique(values: &mut Vec<String>, value: &str) {
    if !value.is_empty() && !values.iter().any(|v| v == value) {
        values.push(value.into());
    }
}
fn routed(provider: &str, id: &str) -> String {
    format!("{provider}/{}", id.replace('/', "-"))
}
fn slug_matches(slug: &str, provider: &str, id: &str) -> bool {
    slug == format!("{provider}/{id}") || slug == routed(provider, id)
}
fn model_for_slug<'a>(slug: &str, provider: &str, ids: &'a [String]) -> Option<&'a str> {
    // Native exact wins; never reverse '-' into '/' without a unique dictionary match.
    if let Some(id) = ids.iter().find(|id| slug == format!("{provider}/{id}")) {
        return Some(id);
    }
    let mut matches = ids.iter().filter(|id| routed(provider, id) == slug);
    let first = matches.next()?;
    if matches.next().is_some() {
        None
    } else {
        Some(first)
    }
}
fn model_ids(channel: &ChannelPolicy) -> Vec<String> {
    let mut ids = vec![];
    for id in channel
        .model_order
        .iter()
        .chain(channel.discovered.iter().map(|m| &m.id))
    {
        append_unique(&mut ids, id);
    }
    if let ModelSelection::Whitelist(selected) = &channel.selection {
        for id in selected {
            append_unique(&mut ids, id);
        }
    }
    ids
}
fn exact_rule(channel: Option<&ChannelPolicy>, id: &str, field: &str) -> Option<Value> {
    channel
        .and_then(|c| c.overrides.get(id))
        .and_then(|r| rule_value(r, field))
}
fn declared_rule(channel: Option<&ChannelPolicy>, id: &str, field: &str) -> Option<Value> {
    channel
        .and_then(|c| c.discovered.iter().find(|m| m.id == id))
        .and_then(|m| rule_value(&m.rules, field))
}
fn map_rule(channel: &ChannelPolicy, id: &str, field: &str) -> Option<Value> {
    if let Some(value) = exact_rule(Some(channel), id, field) {
        return Some(value);
    }
    // modelAdapters is an exact native-id lookup. Other mapped rules use modelRecordValue.
    if field != "protocol" {
        if let Some((family, _)) = id.split_once(':') {
            if let Some(value) = exact_rule(Some(channel), family, field) {
                return Some(value);
            }
        }
        if field != "noReasoning" {
            return channel.overrides.iter().find_map(|(key, rules)| {
                (key.to_lowercase() == id.to_lowercase())
                    .then(|| rule_value(rules, field))
                    .flatten()
            });
        }
    }
    None
}
fn defaults_from_file(raw: &Map<String, Value>) -> ModelResult<ModelRules> {
    let mut rules = ModelRules::default();
    for field in FIELDS {
        if let Some(key) = field.provider {
            assign_rule(&mut rules, field.name, raw.get(key))?;
        }
    }
    if raw.get("fastEnabled") == Some(&Value::Bool(false)) {
        rules.fast = Field::Custom(false);
    }
    Ok(rules)
}
fn overrides_from_file(raw: &Map<String, Value>) -> ModelResult<BTreeMap<String, ModelRules>> {
    let mut result = BTreeMap::<String, ModelRules>::new();
    for field in FIELDS {
        if let Some(map) = field.model.and_then(|key| raw.get(key)) {
            for (id, value) in object(map)? {
                // The operator switch denies the whole lane. Keep the original
                // positive capabilities opaque until that switch is lifted.
                if field.name == "fast" && raw.get("fastEnabled") == Some(&Value::Bool(false)) {
                    result.entry(id.clone()).or_default();
                    continue;
                }
                assign_rule(
                    result.entry(id.clone()).or_default(),
                    field.name,
                    Some(value),
                )?;
            }
        }
    }
    for id in strings(raw.get("noReasoningModels"))? {
        result.entry(id).or_default().no_reasoning = Field::Custom(true);
    }
    if let Some(capabilities) = raw.get("modelCapabilities") {
        for (id, capability) in object(capabilities)? {
            if let Some(modalities) = object(capability)?.get("inputModalities") {
                result.entry(id.clone()).or_default().input_modalities =
                    read_field(Some(modalities))?;
            }
        }
    }
    Ok(result)
}
fn is_disabled(disabled: &[String], provider: &str, id: &str) -> bool {
    disabled.iter().any(|slug| {
        slug_matches(slug, provider, id)
            || (["openai", "chatgpt", "openai-multi", "combo"].contains(&provider) && slug == id)
    })
}
fn selection_from_file(
    config: &Value,
    provider: &str,
    raw: &Map<String, Value>,
    ids: &[String],
    disabled: &[String],
) -> ModelResult<ModelSelection> {
    if raw.get("initialModelSelection").is_some_and(|s| {
        s["version"] == 1
            && s["status"] == "pending"
            && s["registrationId"].as_str().is_some_and(keep_id)
    }) {
        return Ok(ModelSelection::None);
    }
    if let Some(selected) = raw.get("selectedModels") {
        // Deliberately retain explicit [] as the manager's all-hidden intent.
        // Writing it also updates disabledModels; official 2.84 alone treats [] as ALL.
        let mut selected = strings(Some(selected))?;
        for id in &mut selected {
            if !ids.contains(id) {
                if let Some(native) = model_for_slug(id, provider, ids) {
                    *id = native.into();
                }
            }
        }
        selected.retain(|id| !is_disabled(disabled, provider, id));
        let mut seen = BTreeSet::new();
        selected.retain(|id| seen.insert(id.clone()));
        return Ok(if selected.is_empty() {
            ModelSelection::None
        } else {
            ModelSelection::Whitelist(selected)
        });
    }
    // effectiveNewModelPolicy: provider on/off, otherwise install policy, default on.
    let local = raw.get("newModelPolicy").and_then(Value::as_str);
    let policy = match local {
        Some("on" | "off") => local.unwrap(),
        _ => config["modelDiscovery"]["newModelPolicy"]
            .as_str()
            .unwrap_or("on"),
    };
    let visible: Vec<_> = ids
        .iter()
        .filter(|id| !is_disabled(disabled, provider, id))
        .cloned()
        .collect();
    Ok(if policy == "on" && visible.len() == ids.len() {
        ModelSelection::All
    } else if visible.is_empty() {
        ModelSelection::None
    } else {
        ModelSelection::Whitelist(visible)
    })
}

/// File data wins over every factual field in legacy metadata. Only layout and
/// exact template adoptions survive; missing providers are never resurrected.
pub(super) fn read_policy(config: &Value, metadata: &Policy) -> ModelResult<Policy> {
    let root = object(config)?;
    let empty = Map::new();
    let providers = root
        .get("providers")
        .map(object)
        .transpose()?
        .unwrap_or(&empty);
    let picker = strings(root.get("modelPickerOrder"))?;
    let disabled = strings(root.get("disabledModels"))?;
    let mut channels = vec![];
    let mut identities = BTreeSet::new();
    for (provider, value) in providers {
        let raw = object(value)?;
        let old = metadata
            .channels
            .iter()
            .find(|c| c.connection.provider_id == *provider);
        let mut id = old
            .filter(|c| keep_id(&c.id))
            .map(|c| c.id.clone())
            .unwrap_or_else(|| stable_id("channel", provider));
        if !identities.insert(id.clone()) {
            id = stable_id("channel", provider);
            identities.insert(id.clone());
        }
        let url = raw
            .get("baseUrl")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let mut nodes = old.map(|c| c.connection.nodes.clone()).unwrap_or_default();
        let mut node_ids = BTreeSet::new();
        nodes.retain(|n| keep_id(&n.id) && node_ids.insert(n.id.clone()));
        let active_node_id = if let Some(node) = nodes.iter().find(|n| n.url == url) {
            node.id.clone()
        } else if let Some(node) = nodes
            .iter_mut()
            .find(|n| old.is_some_and(|c| c.connection.active_node_id == n.id))
        {
            // The active slot's name/id is layout; its old URL is never file authority.
            node.url = url.clone();
            node.id.clone()
        } else {
            let node_id = stable_id("active-node", provider);
            nodes.retain(|n| n.id != node_id);
            nodes.push(ServiceNode {
                id: node_id.clone(),
                name: "当前节点".into(),
                url,
            });
            node_id
        };
        let defaults = defaults_from_file(raw)?;
        let overrides = overrides_from_file(raw)?;
        let mut ids = vec![];
        for key in ["models", "retainModels"] {
            for model in strings(raw.get(key))? {
                append_unique(&mut ids, &model);
            }
        }
        // Config-only/manual ids are real inventory; no discovery validation.
        for model in strings(raw.get("selectedModels"))? {
            let model = model_for_slug(&model, provider, &ids)
                .unwrap_or(&model)
                .to_owned();
            append_unique(&mut ids, &model);
        }
        for key in overrides.keys() {
            if !key.contains(['*', '?']) {
                append_unique(&mut ids, key);
            }
        }
        let mut naming = BTreeMap::<String, ModelNaming>::new();
        for (key, alias) in [("modelAliases", true), ("modelDisplayNames", false)] {
            if let Some(map) = raw.get(key) {
                for (model, value) in object(map)? {
                    let value = value.as_str().ok_or_else(document_error)?.to_owned();
                    let name = naming.entry(model.clone()).or_default();
                    if alias {
                        name.alias = Some(value);
                    } else {
                        name.display_name = Some(value);
                    }
                    append_unique(&mut ids, model);
                }
            }
        }
        let selection = selection_from_file(config, provider, raw, &ids, &disabled)?;
        let mut order = vec![];
        for slug in &picker {
            if let Some(model) = model_for_slug(slug, provider, &ids) {
                append_unique(&mut order, model);
            }
        }
        for model in &ids {
            append_unique(&mut order, model);
        }
        let mut channel = ChannelPolicy {
            id,
            label: old
                .filter(|c| !c.label.trim().is_empty())
                .map(|c| c.label.clone())
                .or_else(|| {
                    raw.get("name")
                        .and_then(Value::as_str)
                        .filter(|v| !v.trim().is_empty())
                        .map(str::to_owned)
                })
                .unwrap_or_else(|| provider.clone()),
            template_refs: old.map(|c| c.template_refs.clone()).unwrap_or_default(),
            model_refs: old.map(|c| c.model_refs.clone()).unwrap_or_default(),
            connection: ChannelConnection {
                provider_id: provider.clone(),
                protocol: raw
                    .get("adapter")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .into(),
                nodes,
                active_node_id,
                enabled: raw.get("disabled") != Some(&Value::Bool(true)),
                allow_private_network: raw.get("allowPrivateNetwork") == Some(&Value::Bool(true)),
                auth_mode: raw
                    .get("authMode")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                ..Default::default()
            },
            selection,
            model_order: order,
            discovered: vec![],
            defaults,
            overrides,
            // Old rules are not adoption metadata and must not supersede the file.
            family_overrides: BTreeMap::new(),
            naming,
        };
        for model in &ids {
            let mut rules = channel.defaults.clone();
            rules.protocol = Field::Custom(channel.connection.protocol.clone());
            for field in FIELDS {
                if let Some(value) = map_rule(&channel, model, field.name) {
                    assign_rule(&mut rules, field.name, Some(&value))?;
                }
            }
            if raw.contains_key("noReasoningModels") && matches!(rules.no_reasoning, Field::Inherit)
            {
                rules.no_reasoning = Field::Custom(false);
            }
            // fastEnabled=false is an operator denial, never invent a positive capability.
            if raw.get("fastEnabled") == Some(&Value::Bool(false)) {
                rules.fast = Field::Custom(false);
            }
            channel.discovered.push(DiscoveredModel {
                id: model.clone(),
                label: channel
                    .naming
                    .get(model)
                    .and_then(|n| n.display_name.clone())
                    .filter(|v| !v.is_empty())
                    .unwrap_or_else(|| model.clone()),
                rules,
            });
        }
        channels.push(channel);
    }
    channels.sort_by_key(|c| {
        picker
            .iter()
            .position(|slug| slug.starts_with(&format!("{}/", c.connection.provider_id)))
            .unwrap_or(usize::MAX)
    });
    let default_channel_id = root
        .get("defaultProvider")
        .and_then(Value::as_str)
        .and_then(|provider| {
            channels
                .iter()
                .find(|c| c.connection.provider_id == provider)
        })
        .map(|c| c.id.clone());
    Ok(Policy {
        channels,
        template_refs: metadata.template_refs.clone(),
        default_channel_id,
        ..Policy::default()
    })
}

fn set_value(raw: &mut Map<String, Value>, key: &str, value: Option<Value>) {
    match value {
        Some(value) => {
            raw.insert(key.into(), value);
        }
        None => {
            raw.remove(key);
        }
    }
}
fn patch_rule_value(
    raw: &mut Map<String, Value>,
    key: &str,
    before: Option<Value>,
    after: Option<Value>,
) {
    // In a known structured field, edit only changed known members. Unknown
    // members (which can themselves be secret) never pass through ModelRules.
    if let (Some(Value::Object(old)), Some(Value::Object(new)), Some(Value::Object(target))) =
        (&before, &after, raw.get_mut(key))
    {
        for name in old.keys().chain(new.keys()).collect::<BTreeSet<_>>() {
            if old.get(name) != new.get(name) {
                set_value(target, name, new.get(name).cloned());
            }
        }
    } else {
        set_value(raw, key, after);
    }
}
fn set_model_value(
    raw: &mut Map<String, Value>,
    key: &str,
    id: &str,
    value: Option<Value>,
) -> ModelResult<()> {
    if value.is_none() && !raw.contains_key(key) {
        return Ok(());
    }
    let map = raw
        .entry(key.to_owned())
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(document_error)?;
    set_value(map, id, value);
    if map.is_empty() {
        raw.remove(key);
    }
    Ok(())
}
fn active_url(channel: &ChannelPolicy) -> ModelResult<&str> {
    channel
        .connection
        .nodes
        .iter()
        .find(|n| n.id == channel.connection.active_node_id)
        .map(|n| n.url.as_str())
        .ok_or_else(|| invalid("active_node", "当前服务节点不存在"))
}
fn protected(provider: &str, raw: &Map<String, Value>) -> bool {
    ["openai", "chatgpt", "openai-multi", "combo"].contains(&provider.to_ascii_lowercase().as_str())
        || matches!(
            raw.get("authMode").and_then(Value::as_str),
            Some("oauth" | "forward")
        )
}
fn template_rule(
    library: &Library,
    policy: &Policy,
    channel: &ChannelPolicy,
    model: &str,
) -> ModelResult<ModelRules> {
    let mut rules = channel.defaults.clone();
    if let Some((reference, adopted)) = matching::resolve(library, policy, &channel.id, model)? {
        rules = channel.defaults.overlay(&adopted);
        if let Some(family) = channel.family_overrides.get(&reference.template_id) {
            rules = family.overlay(&rules);
        }
    }
    Ok(rules)
}
fn discovered_delta(
    before: Option<&ChannelPolicy>,
    after: &ChannelPolicy,
    id: &str,
) -> ModelResult<ModelRules> {
    let mut rules = ModelRules::default();
    for field in FIELDS {
        let old = declared_rule(before, id, field.name);
        let new = declared_rule(Some(after), id, field.name);
        if old != new {
            assign_rule(&mut rules, field.name, new.as_ref())?;
        }
    }
    Ok(rules)
}

/// The mutation is transactional in memory. Only changed Policy fields authorize
/// set/remove; opaque JSON, credentials, unchanged maps and wire values survive.
pub(super) fn write_policy(
    config: &mut Value,
    before: &Policy,
    after: &Policy,
    library: &Library,
) -> ModelResult<()> {
    let mut next = config.clone();
    write_inner(&mut next, before, after, library)?;
    *config = next;
    Ok(())
}
fn write_inner(
    config: &mut Value,
    before: &Policy,
    after: &Policy,
    library: &Library,
) -> ModelResult<()> {
    object(config)?;
    let mut identities = BTreeSet::new();
    let mut providers = BTreeSet::new();
    for channel in &after.channels {
        if !identities.insert(&channel.id) || !providers.insert(&channel.connection.provider_id) {
            return Err(invalid("channel_identity", "渠道身份或运行绑定重复"));
        }
        if before
            .channels
            .iter()
            .find(|c| c.id == channel.id)
            .is_some_and(|c| c.connection.provider_id != channel.connection.provider_id)
        {
            return Err(invalid(
                "provider_identity",
                "已有渠道的 providerId 不能修改",
            ));
        }
        if before.channels.iter().any(|c| {
            c.connection.provider_id == channel.connection.provider_id && c.id != channel.id
        }) {
            return Err(invalid("provider_identity", "已有运行渠道不能更换管理身份"));
        }
    }
    if before.channels.is_empty()
        && after.channels.is_empty()
        && before.default_channel_id == after.default_channel_id
    {
        return Ok(());
    }
    let mut disabled = strings(config.get("disabledModels"))?;
    let original_disabled = disabled.clone();
    let root = config.as_object_mut().ok_or_else(document_error)?;
    if !root.contains_key("providers") && !after.channels.is_empty() {
        root.insert("providers".into(), json!({}));
    }
    let Some(value) = root.get_mut("providers") else {
        return Ok(());
    };
    let provider_map = value.as_object_mut().ok_or_else(document_error)?;
    for old in &before.channels {
        if !after.channels.iter().any(|c| c.id == old.id) {
            if let Some(raw) = provider_map.get(&old.connection.provider_id) {
                if protected(&old.connection.provider_id, object(raw)?) {
                    return Err(invalid(
                        "protected_provider",
                        "官方认证渠道不能作为普通渠道删除",
                    ));
                }
            }
            provider_map.remove(&old.connection.provider_id);
            let ids = model_ids(old);
            disabled.retain(|slug| {
                !ids.iter()
                    .any(|id| slug_matches(slug, &old.connection.provider_id, id))
            });
        }
    }
    for channel in &after.channels {
        let old = before.channels.iter().find(|c| c.id == channel.id);
        let provider = &channel.connection.provider_id;
        if old.is_none() {
            if provider_map.contains_key(provider) {
                return Err(invalid(
                    "provider_identity",
                    "运行渠道已存在，不能替换原对象",
                ));
            }
            let mut seed = Map::new();
            if let Some(mode) = &channel.connection.auth_mode {
                seed.insert("authMode".into(), json!(mode));
            }
            if protected(provider, &seed) {
                return Err(invalid(
                    "protected_provider",
                    "官方认证渠道不能作为普通渠道创建",
                ));
            }
            provider_map.insert(provider.clone(), Value::Object(seed));
        }
        let raw = provider_map
            .get_mut(provider)
            .and_then(Value::as_object_mut)
            .ok_or_else(document_error)?;
        let url_changed = old.map(active_url).transpose()? != Some(active_url(channel)?);
        let protocol_changed =
            old.is_none_or(|c| c.connection.protocol != channel.connection.protocol);
        let auth_changed =
            old.is_none_or(|c| c.connection.auth_mode != channel.connection.auth_mode);
        let private_changed = old.is_none_or(|c| {
            c.connection.allow_private_network != channel.connection.allow_private_network
        });
        if (protected(provider, raw)
            || matches!(
                channel.connection.auth_mode.as_deref(),
                Some("oauth" | "forward")
            ))
            && (url_changed
                || protocol_changed
                || auth_changed
                || private_changed
                || old.is_some_and(|c| {
                    c.connection.credential_ref != channel.connection.credential_ref
                }))
        {
            return Err(invalid(
                "protected_provider",
                "官方认证渠道的凭据和连接由官方管理",
            ));
        }
        if url_changed {
            raw.insert("baseUrl".into(), json!(active_url(channel)?));
        }
        if protocol_changed {
            raw.insert("adapter".into(), json!(channel.connection.protocol));
        }
        if auth_changed {
            set_value(
                raw,
                "authMode",
                channel.connection.auth_mode.as_ref().map(|v| json!(v)),
            );
        }
        if private_changed && (channel.connection.allow_private_network || old.is_some()) {
            raw.insert(
                "allowPrivateNetwork".into(),
                json!(channel.connection.allow_private_network),
            );
        }
        if old.is_none_or(|c| c.connection.enabled != channel.connection.enabled) {
            raw.insert("disabled".into(), json!(!channel.connection.enabled));
        }

        patch_models(raw, old, channel)?;
        patch_naming(raw, old, channel)?;
        patch_rules(raw, before, after, old, channel, library)?;
        let selection_changed = old.is_none_or(|c| c.selection != channel.selection);
        if selection_changed {
            match &channel.selection {
                ModelSelection::All => {
                    raw.remove("selectedModels");
                    raw.insert("newModelPolicy".into(), json!("on"));
                }
                ModelSelection::None => {
                    raw.insert("selectedModels".into(), json!([]));
                    raw.insert("newModelPolicy".into(), json!("off"));
                }
                ModelSelection::Whitelist(models) => {
                    raw.insert("selectedModels".into(), json!(models));
                    raw.insert("newModelPolicy".into(), json!("off"));
                }
            }
        }
        let ids = model_ids(channel);
        let added = old.map(model_ids).unwrap_or_default();
        if selection_changed || ids.iter().any(|id| !added.contains(id)) {
            for id in &ids {
                let selected = match &channel.selection {
                    ModelSelection::All => true,
                    ModelSelection::None => false,
                    ModelSelection::Whitelist(models) => models.contains(id),
                };
                if selected {
                    disabled.retain(|slug| !slug_matches(slug, provider, id));
                } else if matches!(channel.selection, ModelSelection::None)
                    && !is_disabled(&disabled, provider, id)
                {
                    disabled.push(routed(provider, id));
                }
            }
        }
    }
    if disabled != original_disabled {
        config["disabledModels"] = json!(disabled);
    }
    if before.default_channel_id != after.default_channel_id {
        let provider = after
            .default_channel_id
            .as_ref()
            .map(|id| {
                after
                    .channels
                    .iter()
                    .find(|c| c.id == *id)
                    .map(|c| json!(c.connection.provider_id))
                    .ok_or_else(|| invalid("default_channel", "默认渠道不存在"))
            })
            .transpose()?;
        set_value(
            config.as_object_mut().ok_or_else(document_error)?,
            "defaultProvider",
            provider,
        );
    }
    patch_picker(config, before, after)?;
    Ok(())
}

fn patch_models(
    raw: &mut Map<String, Value>,
    before: Option<&ChannelPolicy>,
    after: &ChannelPolicy,
) -> ModelResult<()> {
    let old: Vec<_> = before
        .map(|c| c.discovered.iter().map(|m| m.id.clone()).collect())
        .unwrap_or_default();
    let new: Vec<_> = after.discovered.iter().map(|m| m.id.clone()).collect();
    if before.is_some()
        && old.iter().collect::<BTreeSet<_>>() == new.iter().collect::<BTreeSet<_>>()
    {
        return Ok(());
    }
    let mut declared = strings(raw.get("models"))?;
    declared.retain(|id| !old.contains(id) || new.contains(id));
    for id in &new {
        if !old.contains(id) {
            append_unique(&mut declared, id);
        }
    }
    if before.is_none() || raw.contains_key("models") || !declared.is_empty() {
        raw.insert("models".into(), json!(declared));
    }
    if let Some(retain) = raw.get("retainModels") {
        let values = strings(Some(retain))?;
        let kept: Vec<_> = values
            .iter()
            .filter(|id| !old.contains(id) || new.contains(id))
            .cloned()
            .collect();
        if kept != values {
            raw.insert("retainModels".into(), json!(kept));
        }
    }
    Ok(())
}
fn patch_naming(
    raw: &mut Map<String, Value>,
    before: Option<&ChannelPolicy>,
    after: &ChannelPolicy,
) -> ModelResult<()> {
    let mut ids: BTreeSet<String> = after.naming.keys().cloned().collect();
    if let Some(old) = before {
        ids.extend(old.naming.keys().cloned());
    }
    ids.extend(after.discovered.iter().map(|m| m.id.clone()));
    for id in ids {
        let old = before.and_then(|c| c.naming.get(&id));
        let new = after.naming.get(&id);
        for (key, alias) in [("modelAliases", true), ("modelDisplayNames", false)] {
            let value = |n: Option<&ModelNaming>| {
                if alias {
                    n.and_then(|v| v.alias.clone())
                } else {
                    n.and_then(|v| v.display_name.clone())
                }
            };
            if value(old) != value(new) {
                set_model_value(
                    raw,
                    key,
                    &id,
                    value(new).filter(|v| !v.is_empty()).map(Value::String),
                )?;
            }
        }
        // A direct model label edit uses the same display-only map.
        if old.and_then(|n| n.display_name.as_ref()) == new.and_then(|n| n.display_name.as_ref()) {
            let old_label = before
                .and_then(|c| c.discovered.iter().find(|m| m.id == id))
                .map(|m| m.label.as_str());
            if let Some(model) = after.discovered.iter().find(|m| m.id == id) {
                if old_label != Some(&model.label) && (old_label.is_some() || model.label != id) {
                    set_model_value(
                        raw,
                        "modelDisplayNames",
                        &id,
                        (!model.label.is_empty() && model.label != id).then(|| json!(model.label)),
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn patch_rules(
    raw: &mut Map<String, Value>,
    before_policy: &Policy,
    after_policy: &Policy,
    before: Option<&ChannelPolicy>,
    after: &ChannelPolicy,
    library: &Library,
) -> ModelResult<()> {
    let adoption_changed = before_policy.template_refs != after_policy.template_refs
        || before.is_none_or(|c| {
            c.template_refs != after.template_refs
                || c.model_refs != after.model_refs
                || c.family_overrides != after.family_overrides
        });
    if let Some(old) = before {
        let declarations = |c: &ChannelPolicy| {
            c.discovered
                .iter()
                .map(|m| (m.id.clone(), m.rules.clone()))
                .collect::<BTreeMap<_, _>>()
        };
        if !adoption_changed
            && old.defaults == after.defaults
            && old.overrides == after.overrides
            && declarations(old) == declarations(after)
        {
            return Ok(());
        }
    }
    let empty = ModelRules::default();
    let old_defaults = before.map(|c| &c.defaults).unwrap_or(&empty);
    for field in FIELDS {
        if let Some(key) = field.provider {
            if rule_value(old_defaults, field.name) != rule_value(&after.defaults, field.name) {
                patch_rule_value(
                    raw,
                    key,
                    rule_value(old_defaults, field.name),
                    rule_value(&after.defaults, field.name),
                );
                if field.name == "fast" && raw.get("fastEnabled") == Some(&Value::Bool(false)) {
                    match &after.defaults.fast {
                        Field::Custom(true) => {
                            raw.insert("fastEnabled".into(), json!(true));
                        }
                        Field::Inherit => {
                            raw.remove("fastEnabled");
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    let mut ids = model_ids(after);
    if let Some(old) = before {
        for id in model_ids(old) {
            append_unique(&mut ids, &id);
        }
    }
    for id in after
        .overrides
        .keys()
        .chain(before.into_iter().flat_map(|c| c.overrides.keys()))
    {
        append_unique(&mut ids, id);
    }
    let mut shared: BTreeMap<&str, Vec<(String, Option<Value>)>> = BTreeMap::new();
    let mut no_reasoning = strings(raw.get("noReasoningModels"))?;
    let original_no_reasoning = no_reasoning.clone();
    for id in &ids {
        let needs_template = adoption_changed
            || before.is_none()
            || before.is_some_and(|c| {
                !c.discovered.iter().any(|m| m.id == *id)
                    && after.discovered.iter().any(|m| m.id == *id)
            })
            || FIELDS.iter().any(|field| {
                exact_rule(before, id, field.name).is_some()
                    && exact_rule(Some(after), id, field.name).is_none()
                    || rule_value(old_defaults, field.name).is_some()
                        && rule_value(&after.defaults, field.name).is_none()
            });
        let old_template = if needs_template {
            before
                .map(|c| template_rule(library, before_policy, c, id))
                .transpose()?
                .unwrap_or_default()
        } else {
            old_defaults.clone()
        };
        let new_template = if needs_template {
            template_rule(library, after_policy, after, id)?
        } else {
            after.defaults.clone()
        };
        let changed_declaration = discovered_delta(before, after, id)?;
        for field in FIELDS {
            let old_exact = exact_rule(before, id, field.name);
            let new_exact = exact_rule(Some(after), id, field.name);
            let exact_changed = old_exact != new_exact;
            let default_changed =
                rule_value(old_defaults, field.name) != rule_value(&after.defaults, field.name);
            let declaration_changed =
                declared_rule(before, id, field.name) != declared_rule(Some(after), id, field.name);
            let template_changed = (adoption_changed
                || default_changed && rule_value(&after.defaults, field.name).is_none())
                && rule_value(&old_template, field.name) != rule_value(&new_template, field.name);
            let changed = exact_changed
                || declaration_changed
                || template_changed
                || default_changed && field.provider.is_none();
            if !changed {
                continue;
            }
            // Unchanged extracted declarations are display data, not another layer
            // of overrides that could resurrect a just-deleted map entry.
            let inherited = rule_value(&new_template, field.name)
                .or_else(|| rule_value(&changed_declaration, field.name));
            let desired = new_exact.clone().or(inherited);
            if !exact_changed && new_exact.is_some() {
                continue;
            }
            if let Some(key) = field.model {
                let mut value = if new_exact.is_some() {
                    new_exact
                } else if field.provider.is_some()
                    && rule_value(&after.defaults, field.name).is_some()
                    && desired == rule_value(&after.defaults, field.name)
                {
                    None
                } else {
                    desired
                };
                if field.name == "protocol"
                    && !exact_changed
                    && value.as_ref() == Some(&json!(after.connection.protocol))
                {
                    value = None;
                }
                if field.name == "fast"
                    && value == Some(json!(true))
                    && raw.get("fastEnabled") == Some(&Value::Bool(false))
                {
                    return Err(invalid(
                        "fast_scope",
                        "渠道 fastEnabled=false 关闭了整个 Fast 通道，请先修改渠道 Fast 设置",
                    ));
                }
                if field.name == "protocol"
                    && value
                        .as_ref()
                        .is_some_and(|v| v != "openai-chat" && v != "openai-responses")
                {
                    return Err(invalid(
                        "model_protocol",
                        "modelAdapters 仅支持 OpenAI Chat 和 Responses 协议",
                    ));
                }
                if field.name == "inputModalities" {
                    let capability = raw
                        .get_mut("modelCapabilities")
                        .and_then(Value::as_object_mut)
                        .and_then(|m| m.get_mut(id))
                        .and_then(Value::as_object_mut);
                    if let Some(capability) =
                        capability.filter(|m| m.contains_key("inputModalities"))
                    {
                        set_value(capability, "inputModalities", value.clone());
                        if value.is_some() {
                            continue;
                        }
                    }
                }
                set_model_value(raw, key, id, value)?;
            } else if field.name == "noReasoning" {
                let enabled = desired.as_ref().and_then(Value::as_bool).unwrap_or(false);
                if !enabled {
                    no_reasoning.retain(|value| value != id);
                } else {
                    append_unique(&mut no_reasoning, id);
                }
            } else {
                shared
                    .entry(field.name)
                    .or_default()
                    .push((id.clone(), desired));
            }
        }
    }
    // Lists have no exact false override. Refuse to pretend false/inherit defeated
    // a surviving ':size' family classification; do not broaden removal to siblings.
    for id in &ids {
        if let Some((family, _)) = id.split_once(':') {
            if exact_rule(before, id, "noReasoning") != exact_rule(Some(after), id, "noReasoning")
                && exact_rule(Some(after), id, "noReasoning") != Some(json!(true))
                && no_reasoning.iter().any(|v| v == family)
            {
                return Err(invalid(
                    "no_reasoning_scope",
                    "此型号仍受族级 noReasoningModels 规则约束，请修改对应族规则",
                ));
            }
        }
    }
    if no_reasoning != original_no_reasoning {
        raw.insert("noReasoningModels".into(), json!(no_reasoning));
    }
    for field in FIELDS.iter().filter(|f| shared.contains_key(f.name)) {
        let edits = &shared[field.name];
        let intended = &edits[0].1;
        if edits.iter().any(|(_, value)| value != intended) {
            return Err(invalid(
                "shared_rule_scope",
                "Fast wire 和推理权威声明是渠道级字段，不能保存不同的型号值",
            ));
        }
        // Verify all other known models would keep the same value under the
        // proposed provider-wide assignment. No silent scope expansion.
        for id in model_ids(after) {
            let template = template_rule(library, after_policy, after, &id)?;
            let value = exact_rule(Some(after), &id, field.name)
                .or_else(|| rule_value(&after.defaults, field.name))
                .or_else(|| rule_value(&template, field.name));
            if value != *intended {
                return Err(invalid(
                    "shared_rule_scope",
                    "此字段由同渠道共用，请在渠道默认设置或给全部型号设置一致值",
                ));
            }
        }
        patch_rule_value(
            raw,
            field.provider.expect("shared rule key"),
            rule_value(old_defaults, field.name),
            intended.clone(),
        );
    }
    Ok(())
}

fn patch_picker(config: &mut Value, before: &Policy, after: &Policy) -> ModelResult<()> {
    let signature = |p: &Policy| {
        p.channels
            .iter()
            .map(|c| (c.id.clone(), c.connection.provider_id.clone(), model_ids(c)))
            .collect::<Vec<_>>()
    };
    if signature(before) == signature(after) {
        return Ok(());
    }
    let old_order = strings(config.get("modelPickerOrder"))?;
    let provider_map = object(&config["providers"])?;
    let mut managed = BTreeSet::<String>::new();
    for channel in before.channels.iter().chain(&after.channels) {
        let provider = &channel.connection.provider_id;
        let empty = Map::new();
        let raw = provider_map
            .get(provider)
            .map(object)
            .transpose()?
            .unwrap_or(&empty);
        if !protected(provider, raw) {
            managed.insert(provider.clone());
        }
    }
    let mut desired = vec![];
    for channel in &after.channels {
        let provider = &channel.connection.provider_id;
        if !managed.contains(provider) {
            continue;
        }
        let ids = model_ids(channel);
        for id in &ids {
            let slug = old_order
                .iter()
                .find(|slug| model_for_slug(slug, provider, &ids) == Some(id.as_str()))
                .cloned()
                .unwrap_or_else(|| routed(provider, id));
            append_unique(&mut desired, &slug);
        }
        let previous = before
            .channels
            .iter()
            .find(|c| c.id == channel.id)
            .map(model_ids)
            .unwrap_or_default();
        // Keep alias/variant and other opaque routed rows within their provider group.
        for slug in &old_order {
            if slug.starts_with(&format!("{provider}/"))
                && model_for_slug(slug, provider, &ids).is_none()
                && !previous
                    .iter()
                    .any(|id| !ids.contains(id) && slug_matches(slug, provider, id))
            {
                append_unique(&mut desired, slug);
            }
        }
    }
    let mut order = vec![];
    let mut inserted = false;
    for slug in old_order {
        if slug
            .split_once('/')
            .is_some_and(|(provider, _)| managed.contains(provider))
        {
            if !inserted {
                order.extend(desired.clone());
                inserted = true;
            }
        } else {
            order.push(slug);
        }
    }
    if !inserted {
        order.extend(desired);
    }
    config["modelPickerOrder"] = json!(order);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHANNEL: &str = "10000000-0000-4000-8000-000000000001";
    const NODE: &str = "10000000-0000-4000-8000-000000000002";
    const OTHER_NODE: &str = "10000000-0000-4000-8000-000000000003";
    const TEMPLATE: &str = "10000000-0000-4000-8000-000000000004";
    const REVISION: &str = "10000000-0000-4000-8000-000000000005";

    fn library() -> Library {
        Library {
            schema_version: SCHEMA_VERSION,
            templates: vec![],
            submission_receipts: BTreeMap::new(),
        }
    }
    // Config-shaped fixture derived from the installed official 2.84 provider
    // contract. These are synthetic secrets; assertions on whole raw documents
    // deliberately avoid Debug output, as the production input can hold real keys.
    fn fixture() -> Value {
        serde_json::from_str(r#"{
          "defaultProvider": "demo",
          "modelDiscovery": {"newModelPolicy": "on"},
          "disabledModels": ["native-unknown", "other/keep"],
          "modelPickerOrder": ["native-unknown", "demo/b", "demo/vendor-a", "native-tail"],
          "futureRoot": {"private": "fixture-root-private"},
          "providers": {
            "demo": {
              "name": "File name", "adapter": "openai-chat",
              "authMode": "key", "baseUrl": "https://file.example.test/v1",
              "disabled": false, "allowPrivateNetwork": true,
              "apiKey": "fixture-key", "apiKeyPool": [{"key": "fixture-pool", "future": 7}],
              "headers": {"X-Secret": "fixture-header"},
              "futureSecret": {"nested": ["fixture-opaque"]},
              "models": ["vendor/a", "b", "tag:8b"], "liveModels": false,
              "selectedModels": ["vendor/a"], "newModelPolicy": "off",
              "contextWindow": 50000, "reasoningEfforts": ["low", "high"],
              "supportsServiceTier": true, "pinnedReasoningEffort": "low",
              "reasoningEffortMap": {"low": "__omit__"},
              "modelContextWindows": {"vendor/a": 128000, "b": 64000, "declared-only": 32768},
              "modelInputModalities": {"vendor/a": ["text", "image"], "b": ["text"]},
              "modelReasoningEfforts": {"vendor/a": ["low", "high"], "b": []},
              "modelDefaultReasoningEfforts": {"vendor/a": "high", "b": "low"},
              "modelPinnedReasoningEfforts": {"vendor/a": "high", "b": "none"},
              "modelReasoningEffortMap": {"vendor/a": {"high": "max"}, "b": {}},
              "modelSupportsServiceTier": {"vendor/a": false, "b": true},
              "modelAdapters": {"vendor/a": "openai-responses", "b": "openai-chat"},
              "noReasoningModels": ["b"], "modelReasoningEffortsAuthoritative": false,
              "fastWire": {"kind": "service-tier", "canonicalToWire": {"priority": "fast"}, "foreignCallerTiers": "drop", "betas": ["test-beta"]},
              "modelAliases": {"vendor/a": "short-a", "b": "short-b"},
              "modelDisplayNames": {"vendor/a": "Friendly A", "b": "Friendly B"}
            }
          }
        }"#).unwrap()
    }
    fn read(config: &Value) -> Policy {
        read_policy(config, &Policy::default()).unwrap()
    }
    fn model<'a>(policy: &'a Policy, id: &str) -> &'a DiscoveredModel {
        policy.channels[0]
            .discovered
            .iter()
            .find(|m| m.id == id)
            .unwrap()
    }
    fn save(config: &mut Value, before: &Policy, after: &Policy) {
        write_policy(config, before, after, &library()).unwrap();
    }

    #[test]
    fn file_snapshot_replaces_stale_facts_but_keeps_ids_labels_nodes_and_adoptions() {
        let config = fixture();
        let mut metadata = read(&config);
        let channel = &mut metadata.channels[0];
        channel.id = CHANNEL.into();
        channel.label = "User label".into();
        channel.selection = ModelSelection::None;
        channel.connection.enabled = false;
        channel.connection.protocol = "anthropic".into();
        channel.connection.auth_mode = Some("oauth".into());
        channel.connection.allow_private_network = false;
        channel.connection.credential_ref = Some("stale-key-reference".into());
        channel.connection.nodes = vec![
            ServiceNode {
                id: NODE.into(),
                name: "Primary".into(),
                url: "https://stale.example.test".into(),
            },
            ServiceNode {
                id: OTHER_NODE.into(),
                name: "Backup".into(),
                url: "https://backup.example.test".into(),
            },
        ];
        channel.connection.active_node_id = NODE.into();
        let reference = TemplateRef {
            template_id: TEMPLATE.into(),
            revision_id: REVISION.into(),
        };
        channel.template_refs = vec![reference.clone()];
        channel.family_overrides.insert(
            TEMPLATE.into(),
            ModelRules {
                context_window: Field::Custom(1),
                ..Default::default()
            },
        );
        channel.defaults.context_window = Field::Custom(1);
        channel.overrides.clear();
        channel.model_order.reverse();
        let mut removed = channel.clone();
        removed.id = stable_id("removed", "gone");
        removed.connection.provider_id = "gone".into();
        metadata.channels.push(removed);
        metadata.default_channel_id = None;
        let actual = read_policy(&config, &metadata).unwrap();
        let c = &actual.channels[0];
        assert_eq!(actual.channels.len(), 1);
        assert_eq!(c.id, CHANNEL);
        assert_eq!(c.label, "User label");
        assert_eq!(c.connection.provider_id, "demo");
        assert_eq!(c.connection.protocol, "openai-chat");
        assert_eq!(active_url(c).unwrap(), "https://file.example.test/v1");
        assert_eq!(c.connection.active_node_id, NODE);
        assert_eq!(c.connection.nodes[1].id, OTHER_NODE);
        assert!(c.connection.enabled && c.connection.allow_private_network);
        assert!(c.connection.credential_ref.is_none());
        assert_eq!(c.connection.auth_mode.as_deref(), Some("key"));
        assert_eq!(
            c.selection,
            ModelSelection::Whitelist(vec!["vendor/a".into()])
        );
        assert_eq!(c.defaults.context_window, Field::Custom(50000));
        assert_eq!(c.template_refs, vec![reference]);
        assert!(c.family_overrides.is_empty());
        assert_eq!(c.model_order[..2], ["b", "vendor/a"]);
        assert_eq!(actual.default_channel_id.as_deref(), Some(CHANNEL));
    }

    #[test]
    fn declared_only_models_get_every_file_rule_without_discovery() {
        let policy = read(&fixture());
        let a = &model(&policy, "vendor/a").rules;
        assert_eq!(a.protocol, Field::Custom("openai-responses".into()));
        assert_eq!(a.context_window, Field::Custom(128000));
        assert_eq!(
            a.input_modalities,
            Field::Custom(vec!["text".into(), "image".into()])
        );
        assert_eq!(
            a.reasoning_efforts,
            Field::Custom(vec!["low".into(), "high".into()])
        );
        assert_eq!(a.default_effort, Field::Custom("high".into()));
        assert_eq!(a.pinned_effort, Field::Custom("high".into()));
        assert_eq!(
            a.effort_map,
            Field::Custom(BTreeMap::from([("high".into(), "max".into())]))
        );
        assert_eq!(a.fast, Field::Custom(false));
        assert_eq!(a.no_reasoning, Field::Custom(false));
        assert_eq!(a.reasoning_authoritative, Field::Custom(false));
        assert!(matches!(a.fast_wire, Field::Custom(Some(_))));
        assert_eq!(model(&policy, "b").rules.no_reasoning, Field::Custom(true));
        assert_eq!(
            model(&policy, "b").rules.reasoning_efforts,
            Field::Custom(vec![])
        );
        assert_eq!(
            model(&policy, "declared-only").rules.context_window,
            Field::Custom(32768)
        );
        assert_eq!(
            policy.channels[0].naming["vendor/a"].alias.as_deref(),
            Some("short-a")
        );
        assert_eq!(model(&policy, "vendor/a").label, "Friendly A");
    }

    #[test]
    fn deterministic_ids_are_lowercase_uuid_v4_and_do_not_depend_on_url() {
        let config = fixture();
        let first = read(&config);
        let second = read(&config);
        assert_eq!(first, second);
        require_uuid(&first.channels[0].id).unwrap();
        require_uuid(&first.channels[0].connection.active_node_id).unwrap();
        let mut changed = config;
        changed["providers"]["demo"]["baseUrl"] = json!("https://new.example.test");
        let third = read(&changed);
        assert_eq!(first.channels[0].id, third.channels[0].id);
        assert_eq!(
            first.channels[0].connection.active_node_id,
            third.channels[0].connection.active_node_id
        );
    }

    #[test]
    fn noop_or_metadata_label_save_preserves_every_json_field_even_with_stale_adoption() {
        let mut config = fixture();
        let original = config.clone();
        let mut before = read(&config);
        before.channels[0].template_refs.push(TemplateRef {
            template_id: TEMPLATE.into(),
            revision_id: REVISION.into(),
        });
        save(&mut config, &before, &before);
        assert!(config == original);
        let mut after = before.clone();
        after.channels[0].label = "Renamed label".into();
        save(&mut config, &before, &after);
        assert!(config == original);
    }

    #[test]
    fn one_model_field_changes_only_one_map_entry_and_default_change_only_scalar() {
        let mut config = fixture();
        let before = read(&config);
        let mut after = before.clone();
        after.channels[0]
            .overrides
            .get_mut("vendor/a")
            .unwrap()
            .context_window = Field::Custom(262144);
        let mut expected = config.clone();
        expected["providers"]["demo"]["modelContextWindows"]["vendor/a"] = json!(262144);
        save(&mut config, &before, &after);
        assert!(config == expected);
        let before = read(&config);
        let mut after = before.clone();
        after.channels[0].defaults.context_window = Field::Custom(99999);
        expected["providers"]["demo"]["contextWindow"] = json!(99999);
        save(&mut config, &before, &after);
        assert!(config == expected);
    }

    #[test]
    fn each_model_control_is_an_independent_single_field_patch() {
        for (field, key, value) in [
            ("protocol", "modelAdapters", json!("openai-chat")),
            ("contextWindow", "modelContextWindows", json!(262144)),
            (
                "inputModalities",
                "modelInputModalities",
                json!(["text", "audio"]),
            ),
            (
                "reasoningEfforts",
                "modelReasoningEfforts",
                json!(["low", "medium", "high"]),
            ),
            (
                "defaultEffort",
                "modelDefaultReasoningEfforts",
                json!("low"),
            ),
            ("pinnedEffort", "modelPinnedReasoningEfforts", json!("low")),
            (
                "effortMap",
                "modelReasoningEffortMap",
                json!({"high":"__omit__"}),
            ),
            ("fast", "modelSupportsServiceTier", json!(true)),
        ] {
            let mut config = fixture();
            let before = read(&config);
            let mut after = before.clone();
            assign_rule(
                after.channels[0].overrides.get_mut("vendor/a").unwrap(),
                field,
                Some(&value),
            )
            .unwrap();
            let mut expected = config.clone();
            expected["providers"]["demo"][key]["vendor/a"] = value;
            save(&mut config, &before, &after);
            assert!(config == expected, "unexpected JSON mutation for {field}");
        }
        let mut config = fixture();
        let before = read(&config);
        let mut after = before.clone();
        after.channels[0]
            .overrides
            .get_mut("vendor/a")
            .unwrap()
            .no_reasoning = Field::Custom(true);
        let mut expected = config.clone();
        expected["providers"]["demo"]["noReasoningModels"] = json!(["b", "vendor/a"]);
        save(&mut config, &before, &after);
        assert!(config == expected);
    }

    #[test]
    fn connection_enable_and_node_edits_do_not_rewrite_model_json_or_keys() {
        for field in ["url", "protocol", "enabled", "private", "auth"] {
            let mut config = fixture();
            let before = read(&config);
            let mut after = before.clone();
            let mut expected = config.clone();
            match field {
                "url" => {
                    after.channels[0].connection.nodes[0].url =
                        "https://new.example.test/v1".into();
                    expected["providers"]["demo"]["baseUrl"] = json!("https://new.example.test/v1");
                }
                "protocol" => {
                    after.channels[0].connection.protocol = "openai-responses".into();
                    expected["providers"]["demo"]["adapter"] = json!("openai-responses");
                }
                "enabled" => {
                    after.channels[0].connection.enabled = false;
                    expected["providers"]["demo"]["disabled"] = json!(true);
                }
                "private" => {
                    after.channels[0].connection.allow_private_network = false;
                    expected["providers"]["demo"]["allowPrivateNetwork"] = json!(false);
                }
                _ => {
                    after.channels[0].connection.auth_mode = Some("local".into());
                    expected["providers"]["demo"]["authMode"] = json!("local");
                }
            }
            save(&mut config, &before, &after);
            assert!(config == expected, "unexpected JSON mutation for {field}");
        }
    }

    #[test]
    fn future_fast_wire_members_survive_read_noop_and_known_member_edits() {
        let mut config = fixture();
        config["providers"]["demo"]["fastWire"]["futureSecret"] =
            json!({"token":"fixture-wire-private"});
        let original = config.clone();
        let before = read(&config);
        save(&mut config, &before, &before);
        assert!(config == original);
        let mut after = before.clone();
        if let Field::Custom(Some(wire)) = &mut after.channels[0].defaults.fast_wire {
            wire.foreign_caller_tiers = "verbatim".into();
        } else {
            panic!("fixture wire missing");
        }
        let mut expected = config.clone();
        expected["providers"]["demo"]["fastWire"]["foreignCallerTiers"] = json!("verbatim");
        save(&mut config, &before, &after);
        assert!(config == expected);
    }

    #[test]
    fn fast_operator_off_is_effective_without_erasing_original_capabilities() {
        let mut config = fixture();
        config["providers"]["demo"]["fastEnabled"] = json!(false);
        let original = config.clone();
        let before = read(&config);
        assert_eq!(before.channels[0].defaults.fast, Field::Custom(false));
        assert!(before.channels[0]
            .discovered
            .iter()
            .all(|m| m.rules.fast == Field::Custom(false)));
        save(&mut config, &before, &before);
        assert!(config == original);
        let mut after = before.clone();
        after.channels[0]
            .overrides
            .get_mut("vendor/a")
            .unwrap()
            .fast = Field::Custom(true);
        assert_eq!(
            write_policy(&mut config, &before, &after, &library())
                .unwrap_err()
                .code,
            "fast_scope"
        );
        assert!(config == original);
        let mut after = before.clone();
        after.channels[0].defaults.fast = Field::Custom(true);
        let mut expected = config.clone();
        expected["providers"]["demo"]["fastEnabled"] = json!(true);
        save(&mut config, &before, &after);
        assert!(config == expected);
        let actual = read(&config);
        assert_eq!(model(&actual, "vendor/a").rules.fast, Field::Custom(false));
        assert_eq!(model(&actual, "b").rules.fast, Field::Custom(true));
    }

    #[test]
    fn inherit_deletes_all_exact_rule_keys_without_recompiling_other_models() {
        let mut config = fixture();
        let before = read(&config);
        let mut after = before.clone();
        after.channels[0].overrides.remove("vendor/a");
        let mut expected = config.clone();
        for field in FIELDS {
            if let Some(key) = field.model {
                expected["providers"]["demo"][key]
                    .as_object_mut()
                    .unwrap()
                    .remove("vendor/a");
            }
        }
        save(&mut config, &before, &after);
        assert!(config == expected);
        let result = read(&config);
        assert_eq!(
            model(&result, "vendor/a").rules.context_window,
            Field::Custom(50000)
        );
        assert_eq!(
            model(&result, "vendor/a").rules.protocol,
            Field::Custom("openai-chat".into())
        );
        assert_eq!(model(&result, "vendor/a").rules.fast, Field::Custom(true));
    }

    #[test]
    fn selection_absence_uses_official_install_default_but_explicit_empty_stays_none() {
        let mut config = json!({"providers":{"p":{"adapter":"openai-chat","baseUrl":"https://example.test","models":["a","b"]}}});
        assert_eq!(read(&config).channels[0].selection, ModelSelection::All);
        config["modelDiscovery"] = json!({"newModelPolicy":"off"});
        assert_eq!(
            read(&config).channels[0].selection,
            ModelSelection::Whitelist(vec!["a".into(), "b".into()])
        );
        config["providers"]["p"]["newModelPolicy"] = json!("inherit");
        config["disabledModels"] = json!(["p/b"]);
        assert_eq!(
            read(&config).channels[0].selection,
            ModelSelection::Whitelist(vec!["a".into()])
        );
        config["providers"]["p"]["newModelPolicy"] = json!("on");
        config["disabledModels"] = json!([]);
        assert_eq!(read(&config).channels[0].selection, ModelSelection::All);
        config["providers"]["p"]["selectedModels"] = json!([]);
        assert_eq!(read(&config).channels[0].selection, ModelSelection::None);
        config["providers"]["p"]["selectedModels"] = json!(["a"]);
        config["providers"]["p"]["initialModelSelection"] =
            json!({"version":1,"registrationId":NODE,"status":"pending"});
        assert_eq!(read(&config).channels[0].selection, ModelSelection::None);
    }

    #[test]
    fn none_persists_empty_allowlist_off_policy_and_real_official_disables_then_all_restores() {
        let mut config = fixture();
        let before = read(&config);
        let mut after = before.clone();
        after.channels[0].selection = ModelSelection::None;
        save(&mut config, &before, &after);
        let raw = &config["providers"]["demo"];
        assert!(raw["selectedModels"] == json!([]));
        assert!(raw["newModelPolicy"] == "off");
        let disabled = strings(config.get("disabledModels")).unwrap();
        for id in model_ids(&after.channels[0]) {
            assert!(is_disabled(&disabled, "demo", &id));
        }
        assert!(disabled.contains(&"native-unknown".into()));
        let before = read(&config);
        assert_eq!(before.channels[0].selection, ModelSelection::None);
        let mut after = before.clone();
        after.channels[0].selection = ModelSelection::All;
        save(&mut config, &before, &after);
        assert!(config["providers"]["demo"].get("selectedModels").is_none());
        assert!(config["providers"]["demo"]["newModelPolicy"] == "on");
        assert!(config["disabledModels"] == json!(["native-unknown", "other/keep"]));
    }

    #[test]
    fn naming_patch_keeps_display_names_until_explicitly_cleared() {
        let mut config = fixture();
        let before = read(&config);
        let mut after = before.clone();
        after.channels[0].naming.get_mut("vendor/a").unwrap().alias = Some("new-route".into());
        let mut expected = config.clone();
        expected["providers"]["demo"]["modelAliases"]["vendor/a"] = json!("new-route");
        save(&mut config, &before, &after);
        assert!(config == expected);
        let before = read(&config);
        let mut after = before.clone();
        after.channels[0]
            .naming
            .get_mut("vendor/a")
            .unwrap()
            .display_name = None;
        expected["providers"]["demo"]["modelDisplayNames"]
            .as_object_mut()
            .unwrap()
            .remove("vendor/a");
        save(&mut config, &before, &after);
        assert!(config == expected);
    }

    #[test]
    fn picker_restores_channel_and_model_order_and_keeps_opaque_native_rows() {
        let mut config = json!({
            "providers": {
                "a": {"adapter":"openai-chat","baseUrl":"https://a.example.test","models":["vendor/a","b"]},
                "b": {"adapter":"openai-chat","baseUrl":"https://b.example.test","models":["y","z"]},
                "openai": {"adapter":"openai-responses","baseUrl":"https://openai.example.test","models":["gpt"]}
            },
            "modelPickerOrder":["native-unknown","b/y","openai/gpt","a/b","a/vendor-a","a/future-variant","native-tail"]
        });
        let before = read(&config);
        assert_eq!(
            before
                .channels
                .iter()
                .map(|c| c.connection.provider_id.as_str())
                .collect::<Vec<_>>(),
            ["b", "openai", "a"]
        );
        assert_eq!(before.channels[2].model_order, ["b", "vendor/a"]);
        let mut after = before.clone();
        after
            .channels
            .sort_by_key(|c| match c.connection.provider_id.as_str() {
                "a" => 0,
                "b" => 1,
                _ => 2,
            });
        after.channels[0].model_order = vec!["vendor/a".into(), "b".into()];
        let untouched = config["providers"].clone();
        save(&mut config, &before, &after);
        assert!(config["providers"] == untouched);
        assert!(
            config["modelPickerOrder"]
                == json!([
                    "native-unknown",
                    "a/vendor-a",
                    "a/b",
                    "a/future-variant",
                    "b/y",
                    "b/z",
                    "openai/gpt",
                    "native-tail"
                ])
        );
        let actual = read_policy(&config, &after).unwrap();
        assert_eq!(actual.channels[0].connection.provider_id, "a");
        assert_eq!(
            actual.channels[0].model_order,
            after.channels[0].model_order
        );
    }

    #[test]
    fn manual_model_id_is_saved_offline_without_catalog_or_key_gate() {
        let mut config = json!({"future":true});
        let before = read(&config);
        let mut after = read(
            &json!({"providers":{"custom":{"adapter":"anthropic","baseUrl":"https://custom.example.test","models":["site/hand-written-id"],"selectedModels":["site/hand-written-id"]}}}),
        );
        after.channels[0].connection.enabled = true;
        save(&mut config, &before, &after);
        assert!(config["providers"]["custom"]["models"] == json!(["site/hand-written-id"]));
        assert!(config["providers"]["custom"]["selectedModels"] == json!(["site/hand-written-id"]));
        assert!(config["providers"]["custom"].get("apiKey").is_none());
        assert!(config["future"] == true);
    }

    #[test]
    fn exact_native_id_wins_over_lossy_slug_and_family_rules_are_read_from_file() {
        let config = json!({"providers":{"p":{"adapter":"openai-chat","baseUrl":"https://example.test","models":["a/b","a-b","family:8b","MODEL"],"selectedModels":["a/b"],"modelContextWindows":{"family":12345,"model":45678},"noReasoningModels":["family"]}},"modelPickerOrder":["p/a-b","p/a/b"]});
        let policy = read(&config);
        assert_eq!(policy.channels[0].model_order[..2], ["a-b", "a/b"]);
        assert_eq!(
            model(&policy, "family:8b").rules.context_window,
            Field::Custom(12345)
        );
        assert_eq!(
            model(&policy, "family:8b").rules.no_reasoning,
            Field::Custom(true)
        );
        assert_eq!(
            model(&policy, "MODEL").rules.context_window,
            Field::Custom(45678)
        );
        assert_eq!(
            policy.channels[0].selection,
            ModelSelection::Whitelist(vec!["a/b".into()])
        );
    }

    #[test]
    fn capability_modalities_patch_preserves_nested_unknowns_and_inherit_clears_both_sources() {
        let mut config = fixture();
        config["providers"]["demo"]["modelCapabilities"] = json!({"vendor/a":{"inputModalities":["text","video"],"contextTier":"long_context","future":{"keep":true}}});
        let before = read(&config);
        assert_eq!(
            model(&before, "vendor/a").rules.input_modalities,
            Field::Custom(vec!["text".into(), "video".into()])
        );
        let mut after = before.clone();
        after.channels[0]
            .overrides
            .get_mut("vendor/a")
            .unwrap()
            .input_modalities = Field::Custom(vec!["text".into(), "audio".into()]);
        let mut expected = config.clone();
        expected["providers"]["demo"]["modelCapabilities"]["vendor/a"]["inputModalities"] =
            json!(["text", "audio"]);
        save(&mut config, &before, &after);
        assert!(config == expected);
        let before = read(&config);
        let mut after = before.clone();
        after.channels[0]
            .overrides
            .get_mut("vendor/a")
            .unwrap()
            .input_modalities = Field::Inherit;
        expected["providers"]["demo"]["modelCapabilities"]["vendor/a"]
            .as_object_mut()
            .unwrap()
            .remove("inputModalities");
        expected["providers"]["demo"]["modelInputModalities"]
            .as_object_mut()
            .unwrap()
            .remove("vendor/a");
        save(&mut config, &before, &after);
        assert!(config == expected);
    }

    #[test]
    fn shared_rules_are_persisted_at_provider_scope_and_no_reasoning_inherit_removes_membership() {
        let mut config = fixture();
        let before = read(&config);
        let mut after = before.clone();
        after.channels[0].defaults.reasoning_authoritative = Field::Custom(true);
        after.channels[0].defaults.fast_wire = Field::Custom(None);
        after.channels[0]
            .overrides
            .get_mut("b")
            .unwrap()
            .no_reasoning = Field::Inherit;
        let mut expected = config.clone();
        expected["providers"]["demo"]["modelReasoningEffortsAuthoritative"] = json!(true);
        expected["providers"]["demo"]["fastWire"] = Value::Null;
        expected["providers"]["demo"]["noReasoningModels"] = json!([]);
        save(&mut config, &before, &after);
        assert!(config == expected);
    }

    #[test]
    fn template_adoption_projects_only_changed_fields_and_preserves_unrelated_json() {
        let mut config = json!({"providers":{"p":{"adapter":"openai-chat","baseUrl":"https://example.test","models":["manual"],"unknown":{"key":"fixture-secret"}}}});
        let before = read(&config);
        let mut after = before.clone();
        let reference = TemplateRef {
            template_id: TEMPLATE.into(),
            revision_id: REVISION.into(),
        };
        after.channels[0].template_refs = vec![reference];
        let mut content = TemplateContent::default();
        content.matching.include = vec![Pattern {
            kind: MatchKind::Prefix,
            value: "man".into(),
        }];
        content.rules.context_window = Field::Custom(200000);
        let mut library = library();
        library.templates.push(Template {
            id: TEMPLATE.into(),
            label: "Fixture".into(),
            source: TemplateSource::User,
            default_revision: Some(REVISION.into()),
            revisions: vec![Revision {
                id: REVISION.into(),
                created_at: "2026-10-27T00:00:00Z".into(),
                saved_label: "Fixture".into(),
                content,
                digest: "fixture".into(),
                base_revision: None,
                changes: vec![],
                origin: SaveOrigin::Manual,
            }],
        });
        let mut expected = config.clone();
        expected["providers"]["p"]["modelContextWindows"] = json!({"manual":200000});
        write_policy(&mut config, &before, &after, &library).unwrap();
        assert!(config == expected);
        let before = after.clone();
        after.channels[0].template_refs.clear();
        expected["providers"]["p"]
            .as_object_mut()
            .unwrap()
            .remove("modelContextWindows");
        write_policy(&mut config, &before, &after, &library).unwrap();
        assert!(config == expected);
    }

    #[test]
    fn protected_connections_identity_and_unrepresentable_shared_edits_fail_atomically() {
        for provider in ["openai", "combo", "ordinary-oauth", "ordinary-forward"] {
            let mode = if provider.ends_with("oauth") {
                "oauth"
            } else if provider.ends_with("forward") {
                "forward"
            } else {
                "key"
            };
            let mut config = json!({"providers":{provider:{"adapter":"openai-chat","baseUrl":"https://example.test","authMode":mode,"apiKey":"fixture-secret","models":["a"]}}});
            let original = config.clone();
            let before = read(&config);
            let mut after = before.clone();
            after.channels[0].connection.nodes[0].url = "https://changed.example.test".into();
            assert_eq!(
                write_policy(&mut config, &before, &after, &library())
                    .unwrap_err()
                    .code,
                "protected_provider"
            );
            assert!(config == original);
        }
        let mut config = fixture();
        let original = config.clone();
        let before = read(&config);
        let mut after = before.clone();
        after.channels[0].connection.provider_id = "changed".into();
        assert_eq!(
            write_policy(&mut config, &before, &after, &library())
                .unwrap_err()
                .code,
            "provider_identity"
        );
        assert!(config == original);
        let mut after = before.clone();
        after.channels[0]
            .overrides
            .get_mut("vendor/a")
            .unwrap()
            .reasoning_authoritative = Field::Custom(true);
        assert_eq!(
            write_policy(&mut config, &before, &after, &library())
                .unwrap_err()
                .code,
            "shared_rule_scope"
        );
        assert!(config == original);
    }
}
