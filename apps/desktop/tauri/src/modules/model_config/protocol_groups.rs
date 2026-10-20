//! Protocol partitions are persistent runtime identities, independent of the
//! active URL or selection. Product channels still own one credential and node.
use super::{compiler, schema::*, upstream::UpstreamSnapshot};
use crate::{infrastructure::atomic_write::atomic_write, modules::backup::safety};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
const PATH: &str = "manager-state/model-protocol-mappings.json";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Mapping {
    pub id: String,
    pub channel_id: String,
    pub protocol: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    pub provider_id: String,
}
pub fn marker(channel: &str, protocol: &str) -> String {
    format!("OpenCodeX channel={channel} protocol={protocol}")
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Partition {
    protocol: String,
    variant: Option<String>,
}
pub fn mapping_marker(mapping: &Mapping) -> String {
    let mut value = marker(&mapping.channel_id, &mapping.protocol);
    if let Some(variant) = &mapping.variant {
        value.push_str(" rules=");
        value.push_str(variant);
    }
    value
}
fn bucket(channel: &ChannelPolicy, row: &compiler::ModelProjection) -> ModelResult<String> {
    let protocol = match &row.rules.protocol {
        Field::Custom(v) => v,
        _ => &channel.connection.protocol,
    };
    if protocol == &channel.connection.protocol
        || ["openai-chat", "openai-responses"].contains(&protocol.as_str())
    {
        return Ok(channel.connection.protocol.clone());
    }
    if !["anthropic", "google", "ollama-native"].contains(&protocol.as_str())
        || ![
            "openai-chat",
            "openai-responses",
            "anthropic",
            "google",
            "ollama-native",
        ]
        .contains(&channel.connection.protocol.as_str())
        || protocol == "ollama-native"
        || channel.connection.protocol == "ollama-native"
    {
        return Err(invalid(
            "protocol_credentials",
            "专用认证或本机无认证协议不能按普通Key复制到协议分组",
        ));
    }
    Ok(protocol.clone())
}
fn partitions<'a>(
    channel: &ChannelPolicy,
    projection: &'a compiler::DraftProjection,
) -> ModelResult<BTreeMap<Partition, Vec<&'a compiler::ModelProjection>>> {
    let mut buckets: BTreeMap<String, BTreeMap<Vec<u8>, Vec<&compiler::ModelProjection>>> =
        BTreeMap::new();
    for row in projection
        .rows
        .iter()
        .filter(|r| r.channel_id == channel.id)
    {
        let rules = explicit_rules(row);
        let signature = serde_json::to_vec(&(rules.reasoning_authoritative, rules.fast_wire))?;
        buckets
            .entry(bucket(channel, row)?)
            .or_default()
            .entry(signature)
            .or_default()
            .push(row);
    }
    let inherit =
        serde_json::to_vec(&(Field::<bool>::Inherit, Field::<Option<FastWire>>::Inherit))?;
    let defaults = serde_json::to_vec(&(
        &channel.defaults.reasoning_authoritative,
        &channel.defaults.fast_wire,
    ))?;
    let mut result = BTreeMap::new();
    for (protocol, groups) in buckets {
        // Visibility, ordering and current node never select the anchor identity.
        let primary = if groups.contains_key(&defaults) {
            defaults.clone()
        } else if groups.contains_key(&inherit) {
            inherit.clone()
        } else {
            groups.keys().next().cloned().unwrap()
        };
        for (signature, rows) in groups {
            let variant = (signature != primary).then(|| super::store::sha256(&signature));
            result.insert(
                Partition {
                    protocol: protocol.clone(),
                    variant,
                },
                rows,
            );
        }
    }
    Ok(result)
}
fn desired(
    library: &Library,
    policy: &Policy,
) -> ModelResult<BTreeSet<(String, String, Option<String>)>> {
    let projection = compiler::project(library, policy)?;
    let mut result = BTreeSet::new();
    for channel in &policy.channels {
        for key in partitions(channel, &projection)?.into_keys() {
            if key.protocol != channel.connection.protocol || key.variant.is_some() {
                result.insert((channel.id.clone(), key.protocol, key.variant));
            }
        }
    }
    Ok(result)
}
pub fn mappings(root: &Path, library: &Library, policy: &Policy) -> ModelResult<Vec<Mapping>> {
    let path = root.join(PATH);
    safety::check_path(root, &path, true)?;
    let lock = crate::infrastructure::locking::lock_path_for(&path);
    safety::check_path(root, &lock, true)?;
    let _guard = crate::infrastructure::locking::TargetFileLock::lock(&path)?;
    let mut values: Vec<Mapping> = if path.exists() {
        serde_json::from_slice(&safety::read_file(&path, 2 * 1024 * 1024)?)?
    } else {
        vec![]
    };
    let mut keys = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut providers = BTreeSet::new();
    for value in &values {
        require_uuid(&value.id)?;
        require_uuid(&value.channel_id)?;
        if !keys.insert((&value.channel_id, &value.protocol, &value.variant))
            || value.variant.as_ref().is_some_and(|v| {
                !v.starts_with("sha256:")
                    || v.len() != 71
                    || !v[7..].bytes().all(|b| b.is_ascii_hexdigit())
            })
            || !ids.insert(&value.id)
            || !providers.insert(&value.provider_id)
            || value.provider_id != format!("ocxd-{}", value.id.replace('-', ""))
        {
            return Err(invalid("protocol_mapping", "协议分组身份重复或被更改"));
        }
    }
    let before = serde_json::to_vec(&values)?;
    for (channel_id, protocol, variant) in desired(library, policy)? {
        if !values.iter().any(|value| {
            value.channel_id == channel_id && value.protocol == protocol && value.variant == variant
        }) {
            let id = uuid::Uuid::new_v4().to_string();
            values.push(Mapping {
                provider_id: format!("ocxd-{}", id.replace('-', "")),
                id,
                channel_id,
                protocol,
                variant,
            });
        }
    }
    let after = serde_json::to_vec(&values)?;
    if before != after {
        atomic_write(&path, &after, 0o600)?;
        if safety::read_file(&path, 2 * 1024 * 1024)? != after {
            return Err(invalid("protocol_mapping", "协议分组保存后待核对"));
        }
    }
    Ok(values)
}
fn explicit_rules(row: &compiler::ModelProjection) -> ModelRules {
    let mut rules = row.rules.clone();
    if row
        .source
        .get("contextWindow")
        .is_none_or(|s| s == "上游声明")
    {
        rules.context_window = Field::Inherit;
    }
    if row
        .source
        .get("inputModalities")
        .is_none_or(|s| s == "上游声明")
    {
        rules.input_modalities = Field::Inherit;
    }
    if row
        .source
        .get("reasoningEfforts")
        .is_none_or(|s| s == "上游声明")
    {
        rules.reasoning_efforts = Field::Inherit;
    }
    if row
        .source
        .get("defaultEffort")
        .is_none_or(|s| s == "上游声明")
    {
        rules.default_effort = Field::Inherit;
    }
    if row.source.get("fast").is_none_or(|s| s == "上游声明") {
        rules.fast = Field::Inherit;
    }
    if row
        .source
        .get("pinnedEffort")
        .is_none_or(|s| s == "上游声明")
    {
        rules.pinned_effort = Field::Inherit;
    }
    if row.source.get("effortMap").is_none_or(|s| s == "上游声明") {
        rules.effort_map = Field::Inherit;
    }
    if row
        .source
        .get("noReasoning")
        .is_none_or(|s| s == "上游声明")
    {
        rules.no_reasoning = Field::Inherit;
    }
    if row
        .source
        .get("reasoningAuthoritative")
        .is_none_or(|s| s == "上游声明")
    {
        rules.reasoning_authoritative = Field::Inherit;
    }
    if row.source.get("fastWire").is_none_or(|s| s == "上游声明") {
        rules.fast_wire = Field::Inherit;
    }
    rules
}
pub fn expand(
    snapshot: &UpstreamSnapshot,
    library: &Library,
    policy: &Policy,
    mappings: &[Mapping],
) -> ModelResult<Policy> {
    let projection = compiler::project(library, policy)?;
    let desired = desired(library, policy)?;
    let mut result = Policy {
        default_channel_id: policy.default_channel_id.clone(),
        ..Default::default()
    };
    for channel in &policy.channels {
        let own = mappings
            .iter()
            .filter(|m| m.channel_id == channel.id)
            .collect::<Vec<_>>();
        if !own.is_empty()
            && snapshot.providers["providers"][&channel.connection.provider_id]["authMode"]
                .as_str()
                .is_some_and(|mode| ["oauth", "forward"].contains(&mode))
        {
            return Err(invalid(
                "protocol_credentials",
                "OAuth或转发认证不能作为普通Key复制到协议分组",
            ));
        }
        if own.is_empty() {
            let mut next = channel.clone();
            next.template_refs.clear();
            next.model_refs.clear();
            next.family_overrides.clear();
            next.defaults = ModelRules::default();
            next.overrides = projection
                .rows
                .iter()
                .filter(|row| row.channel_id == channel.id)
                .map(|row| (row.model_id.clone(), explicit_rules(row)))
                .collect();
            result.channels.push(next);
            continue;
        }
        let groups = partitions(channel, &projection)?;
        let mut destinations = vec![(
            channel.id.clone(),
            channel.connection.provider_id.clone(),
            Partition {
                protocol: channel.connection.protocol.clone(),
                variant: None,
            },
            false,
        )];
        for mapping in own {
            if let Some(prior) = snapshot.providers["providers"].get(&mapping.provider_id) {
                if prior["note"] != mapping_marker(mapping) {
                    return Err(invalid(
                        "protocol_mapping_collision",
                        "协议或配置分组标识已被其他配置占用，未覆盖",
                    ));
                }
            }
            if desired.contains(&(
                channel.id.clone(),
                mapping.protocol.clone(),
                mapping.variant.clone(),
            )) || snapshot.providers["providers"]
                .get(&mapping.provider_id)
                .is_some()
            {
                destinations.push((
                    mapping.id.clone(),
                    mapping.provider_id.clone(),
                    Partition {
                        protocol: mapping.protocol.clone(),
                        variant: mapping.variant.clone(),
                    },
                    true,
                ));
            }
        }
        for (id, provider, key, is_group) in destinations {
            let rows =
                if is_group && key.protocol == channel.connection.protocol && key.variant.is_none()
                {
                    vec![]
                } else {
                    groups.get(&key).cloned().unwrap_or_default()
                };
            let protocol = key.protocol;
            let exists = snapshot.providers["providers"].get(&provider).is_some();
            if rows.is_empty() && !exists {
                continue;
            }
            if is_group && !rows.is_empty() && channel.connection.credential_ref.is_none() {
                return Err(invalid(
                    "protocol_credential",
                    "此渠道首次使用协议分组，请补充并验证共用Key",
                ));
            }
            let mut next = channel.clone();
            next.id = id;
            next.connection.provider_id = provider;
            next.connection.protocol = protocol;
            next.template_refs.clear();
            next.model_refs.clear();
            next.family_overrides.clear();
            next.defaults = ModelRules::default();
            next.model_order = rows.iter().map(|row| row.model_id.clone()).collect();
            next.naming
                .retain(|id, _| rows.iter().any(|row| row.model_id == *id));
            next.discovered = channel
                .discovered
                .iter()
                .filter(|m| rows.iter().any(|row| row.model_id == m.id))
                .cloned()
                .collect();
            next.overrides = rows
                .iter()
                .map(|row| (row.model_id.clone(), explicit_rules(row)))
                .collect();
            let selected = rows
                .iter()
                .filter(|row| row.selected)
                .map(|row| row.model_id.clone())
                .collect::<Vec<_>>();
            next.selection = if selected.is_empty() {
                ModelSelection::None
            } else {
                ModelSelection::Whitelist(selected)
            };
            result.channels.push(next);
        }
    }
    // All matching was resolved in product channel scope above, including exact
    // model references and channel filters. Reapplying it to synthetic IDs is wrong.
    Ok(result)
}
pub fn seeds(
    snapshot: &UpstreamSnapshot,
    policy: &Policy,
    execution: &Policy,
    mappings: &[Mapping],
    materials: &BTreeMap<String, Value>,
) -> ModelResult<BTreeMap<String, Value>> {
    let mut result = BTreeMap::new();
    for mapping in mappings.iter().filter(|m| m.variant.is_some()) {
        if snapshot.providers["providers"]
            .get(&mapping.provider_id)
            .is_some()
        {
            continue;
        }
        let Some(parent) = policy
            .channels
            .iter()
            .find(|c| c.id == mapping.channel_id && c.connection.protocol == mapping.protocol)
        else {
            continue;
        };
        let Some(target) = execution.channels.iter().find(|c| c.id == mapping.id) else {
            continue;
        };
        let raw = materials
            .get(&parent.connection.provider_id)
            .or_else(|| snapshot.providers["providers"].get(&parent.connection.provider_id));
        let Some(raw) = raw else { continue };
        let public = super::provider_fields::public_from_raw(raw)?;
        let mut portable = super::transfer::portable(parent);
        portable
            .connection
            .as_mut()
            .ok_or_else(|| invalid("protocol_mapping", "配置分组缺少节点"))?
            .provider = Some(public);
        let ids = target
            .discovered
            .iter()
            .map(|m| m.id.clone())
            .chain(target.model_order.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if ids.is_empty() {
            continue;
        }
        let restricted = super::transfer::restrict_models(&portable, &ids)?;
        let mut public = restricted.connection.unwrap().provider.unwrap();
        public.as_object_mut().unwrap().retain(|key, value| {
            !(key.starts_with("model") && value.as_object().is_some_and(|map| map.is_empty()))
        });
        for key in ["noReasoningModels", "noVisionModels"] {
            if let Some(values) = public.get_mut(key).and_then(Value::as_array_mut) {
                values.retain(|value| {
                    value.as_str().is_some_and(|family| {
                        ids.iter()
                            .any(|id| id == family || id.starts_with(&format!("{family}:")))
                    })
                });
                if values.is_empty() {
                    public.as_object_mut().unwrap().remove(key);
                }
            }
        }
        // Provider-wide shortcut aliases/defaults cannot be cloned into a second route.
        for key in ["alias", "defaultModel"] {
            public.as_object_mut().unwrap().remove(key);
        }
        public["adapter"] = serde_json::json!(mapping.protocol);
        public["newModelPolicy"] = serde_json::json!("off");
        result.insert(mapping.provider_id.clone(), public);
    }
    Ok(result)
}
pub fn decorate(next: &mut Value, policy: &Policy, mappings: &[Mapping]) {
    for mapping in mappings {
        if !policy.channels.iter().any(|c| c.id == mapping.channel_id) {
            continue;
        }
        if let Some(provider) = next["providers"].get_mut(&mapping.provider_id) {
            provider["note"] = serde_json::json!(mapping_marker(mapping));
        }
    }
    for channel in &policy.channels {
        if matches!(channel.selection, ModelSelection::All) {
            if let Some(provider) = next["providers"].get_mut(&channel.connection.provider_id) {
                provider["newModelPolicy"] = serde_json::json!("on");
            }
        }
    }
}
pub fn order(
    snapshot: &UpstreamSnapshot,
    library: &Library,
    policy: &Policy,
    execution: &Policy,
    mappings: &[Mapping],
    compiled: Vec<String>,
    expected: &BTreeSet<String>,
) -> ModelResult<Vec<String>> {
    let projection = compiler::project(library, policy)?;
    let managed = execution
        .channels
        .iter()
        .map(|c| c.connection.provider_id.as_str())
        .collect::<BTreeSet<_>>();
    let source = snapshot
        .models
        .as_array()
        .ok_or_else(|| invalid("runtime_models", "官方模型清单无效"))?;
    let mut routes = BTreeMap::new();
    for channel in &policy.channels {
        for (key, rows) in partitions(channel, &projection)? {
            let provider = if key.protocol == channel.connection.protocol && key.variant.is_none() {
                channel.connection.provider_id.as_str()
            } else {
                mappings
                    .iter()
                    .find(|m| {
                        m.channel_id == channel.id
                            && m.protocol == key.protocol
                            && m.variant == key.variant
                    })
                    .map(|m| m.provider_id.as_str())
                    .ok_or_else(|| invalid("protocol_mapping", "模型协议或配置没有稳定分组"))?
            };
            for row in rows {
                routes.insert((row.channel_id.as_str(), row.model_id.as_str()), provider);
            }
        }
    }
    let mut desired = vec![];
    for row in projection
        .rows
        .iter()
        .filter(|row| row.enabled && row.selected)
    {
        let provider = *routes
            .get(&(row.channel_id.as_str(), row.model_id.as_str()))
            .ok_or_else(|| invalid("protocol_mapping", "型号缺少配置分组"))?;
        let slug = source
            .iter()
            .find(|model| {
                model["provider"].as_str() == Some(provider)
                    && model["id"].as_str() == Some(&row.model_id)
            })
            .and_then(|model| model["namespaced"].as_str())
            .map(str::to_owned)
            .unwrap_or_else(|| format!("{provider}/{}", row.model_id.replace('/', "-")));
        if !expected.contains(&slug) {
            return Err(invalid(
                "protocol_order",
                "协议分组模型缺少经过投影核验的目录身份",
            ));
        }
        desired.push(slug);
    }
    let mut result = vec![];
    let mut inserted = false;
    for slug in compiled {
        if slug
            .split_once('/')
            .is_some_and(|(provider, _)| managed.contains(provider))
        {
            if !inserted {
                result.extend(desired.clone());
                inserted = true;
            }
        } else {
            result.push(slug);
        }
    }
    if !inserted {
        result.extend(desired);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn same_protocol_rules_partition_without_extending_to_inheriting_models_and_keep_identity() {
        let root = tempfile::tempdir().unwrap();
        let (library, mut policy, mut snapshot) = fixture();
        policy.channels[0].overrides.clear();
        policy.channels[0].overrides.insert(
            "base-model".into(),
            ModelRules {
                reasoning_efforts: Field::Custom(vec!["high".into()]),
                reasoning_authoritative: Field::Custom(true),
                fast_wire: Field::Custom(Some(FastWire {
                    kind: "service-tier".into(),
                    canonical_to_wire: BTreeMap::from([("priority".into(), "priority".into())]),
                    foreign_caller_tiers: "drop".into(),
                    betas: vec![],
                })),
                ..Default::default()
            },
        );
        snapshot.providers["providers"]["mixed"]["statelessResponses"] = serde_json::json!(true);
        snapshot.providers["providers"]["mixed"]["modelDisplayNames"] =
            serde_json::json!({"base-model":"Original","anthropic-model":"Other"});
        snapshot.providers["providers"]["mixed"]["modelPinnedReasoningEfforts"] =
            serde_json::json!({"anthropic-model":"high"});
        snapshot.providers["providers"]["mixed"]["noReasoningModels"] =
            serde_json::json!(["anthropic-model"]);
        let maps = mappings(root.path(), &library, &policy).unwrap();
        assert_eq!(maps.len(), 1);
        assert_eq!(maps[0].protocol, "openai-responses");
        assert!(maps[0].variant.is_some());
        let expanded = expand(&snapshot, &library, &policy, &maps).unwrap();
        assert_eq!(
            expanded.channels[0].selection,
            ModelSelection::Whitelist(vec!["anthropic-model".into()])
        );
        assert_eq!(
            expanded.channels[1].selection,
            ModelSelection::Whitelist(vec!["base-model".into()])
        );
        let seed = seeds(&snapshot, &policy, &expanded, &maps, &BTreeMap::new()).unwrap();
        assert!(seed[&maps[0].provider_id]
            .get("modelPinnedReasoningEfforts")
            .is_none());
        assert!(seed[&maps[0].provider_id]
            .get("noReasoningModels")
            .is_none());
        assert_eq!(seed[&maps[0].provider_id]["statelessResponses"], true);
        assert!(seed[&maps[0].provider_id]["modelDisplayNames"]
            .get("anthropic-model")
            .is_none());
        let (mut next, compiled, expected) = super::super::runtime::compile_materials(
            &snapshot,
            &library,
            &expanded,
            &[],
            &BTreeMap::new(),
            &seed,
        )
        .unwrap();
        decorate(&mut next, &policy, &maps);
        let group = &next["providers"][&maps[0].provider_id];
        assert_eq!(group["modelReasoningEffortsAuthoritative"], true);
        assert_eq!(group["fastWire"]["kind"], "service-tier");
        assert!(next["providers"]["mixed"].get("fastWire").is_none());
        assert!(next["providers"]["mixed"]
            .get("modelReasoningEffortsAuthoritative")
            .is_none());
        assert_eq!(group["note"], mapping_marker(&maps[0]));
        let ordered = order(
            &snapshot, &library, &policy, &expanded, &maps, compiled, &expected,
        )
        .unwrap();
        assert_eq!(
            ordered,
            vec![
                "mixed/anthropic-model".to_string(),
                format!("{}/base-model", maps[0].provider_id)
            ]
        );
        policy.channels[0].selection = ModelSelection::None;
        policy.channels[0].connection.nodes[0].url = "https://other.example.test/v1".into();
        let hidden = mappings(root.path(), &library, &policy).unwrap();
        assert_eq!(hidden[0].provider_id, maps[0].provider_id);
        snapshot.providers = next;
        let expanded = expand(&snapshot, &library, &policy, &hidden).unwrap();
        assert!(expanded
            .channels
            .iter()
            .all(|c| c.selection == ModelSelection::None));
        snapshot.providers["providers"][&maps[0].provider_id]["note"] =
            serde_json::json!(marker(&maps[0].channel_id, &maps[0].protocol));
        assert_eq!(
            expand(&snapshot, &library, &policy, &hidden)
                .unwrap_err()
                .code,
            "protocol_mapping_collision"
        );
    }
    fn fixture() -> (Library, Policy, UpstreamSnapshot) {
        let library = super::super::store::seed_library();
        let node = ServiceNode {
            id: uuid::Uuid::new_v4().to_string(),
            name: "main".into(),
            url: "https://fixture.example.test/v1".into(),
        };
        let channel = ChannelPolicy {
            naming: Default::default(),
            id: uuid::Uuid::new_v4().to_string(),
            label: "mixed".into(),
            connection: ChannelConnection {
                auth_mode: None,
                source_read_id: None,
                imported_config: None,
                provider_id: "mixed".into(),
                protocol: "openai-responses".into(),
                nodes: vec![node.clone()],
                active_node_id: node.id,
                credential_ref: Some(uuid::Uuid::new_v4().to_string()),
                enabled: true,
                ..Default::default()
            },
            selection: ModelSelection::All,
            discovered: ["base-model", "anthropic-model"]
                .iter()
                .map(|id| DiscoveredModel {
                    id: (*id).into(),
                    label: (*id).into(),
                    rules: ModelRules::default(),
                })
                .collect(),
            model_order: vec!["anthropic-model".into(), "base-model".into()],
            overrides: BTreeMap::from([(
                "anthropic-model".into(),
                ModelRules {
                    protocol: Field::Custom("anthropic".into()),
                    ..Default::default()
                },
            )]),
            template_refs: vec![],
            model_refs: BTreeMap::new(),
            defaults: ModelRules::default(),
            family_overrides: BTreeMap::new(),
        };
        let snapshot = UpstreamSnapshot {
            version: "2.84.0".into(),
            providers: serde_json::json!({"defaultProvider":"mixed","providers":{"mixed":{"adapter":"openai-responses","baseUrl":"https://fixture.example.test/v1","models":["base-model","anthropic-model"],"liveModels":true}}}),
            models: serde_json::json!([{"provider":"mixed","id":"base-model","namespaced":"mixed/base-model","disabled":false},{"provider":"mixed","id":"anthropic-model","namespaced":"mixed/anthropic-model","disabled":false}]),
            order: serde_json::json!({"pickerOrder":["mixed/anthropic-model","mixed/base-model"]}),
            catalog: serde_json::json!({"models":[{"slug":"mixed/anthropic-model"},{"slug":"mixed/base-model"}]}),
        };
        (
            library,
            Policy {
                channels: vec![channel],
                ..Default::default()
            },
            snapshot,
        )
    }
    #[test]
    fn disabled_wire_uses_exact_partition_fast_overrides_without_disabling_sibling_capability() {
        let root = tempfile::tempdir().unwrap();
        let (library, mut policy, mut snapshot) = fixture();
        policy.channels[0].overrides.clear();
        policy.channels[0].overrides.insert(
            "base-model".into(),
            ModelRules {
                fast: Field::Custom(false),
                fast_wire: Field::Custom(None),
                ..Default::default()
            },
        );
        snapshot.providers["providers"]["mixed"]["supportsServiceTier"] = serde_json::json!(true);
        let maps = mappings(root.path(), &library, &policy).unwrap();
        let expanded = expand(&snapshot, &library, &policy, &maps).unwrap();
        let seed = seeds(&snapshot, &policy, &expanded, &maps, &BTreeMap::new()).unwrap();
        let (next, _, _) = super::super::runtime::compile_materials(
            &snapshot,
            &library,
            &expanded,
            &[],
            &BTreeMap::new(),
            &seed,
        )
        .unwrap();
        assert_eq!(
            next["providers"][&maps[0].provider_id]["fastWire"],
            Value::Null
        );
        assert_eq!(
            next["providers"][&maps[0].provider_id]["modelSupportsServiceTier"]["base-model"],
            false
        );
        assert_eq!(next["providers"]["mixed"]["supportsServiceTier"], true);
    }
    #[test]
    fn mapping_identity_survives_node_switch_selection_changes_and_process_reload() {
        let root = tempfile::tempdir().unwrap();
        let (library, mut policy, snapshot) = fixture();
        let original = mappings(root.path(), &library, &policy).unwrap();
        assert_eq!(original.len(), 1);
        let expanded = expand(&snapshot, &library, &policy, &original).unwrap();
        assert_eq!(expanded.channels.len(), 2);
        let group = &expanded.channels[1];
        assert_eq!(group.connection.provider_id, original[0].provider_id);
        assert_eq!(
            group.connection.credential_ref,
            policy.channels[0].connection.credential_ref
        );
        assert_eq!(group.connection.nodes, policy.channels[0].connection.nodes);
        assert_eq!(
            group.selection,
            ModelSelection::Whitelist(vec!["anthropic-model".into()])
        );
        assert_eq!(
            expanded.channels[0].selection,
            ModelSelection::Whitelist(vec!["base-model".into()])
        );
        let (next, compiled, expected) =
            super::super::runtime::compile(&snapshot, &library, &expanded).unwrap();
        assert_eq!(
            next["providers"][&original[0].provider_id]["adapter"],
            "anthropic"
        );
        assert_eq!(
            order(&snapshot, &library, &policy, &expanded, &original, compiled, &expected).unwrap(),
            vec![
                format!("{}/anthropic-model", original[0].provider_id),
                "mixed/base-model".into()
            ]
        );
        policy.channels[0].connection.nodes[0].url = "https://other.example.test/v1".into();
        policy.channels[0].selection = ModelSelection::Whitelist(vec!["base-model".into()]);
        let reloaded = mappings(root.path(), &library, &policy).unwrap();
        assert_eq!(reloaded[0].id, original[0].id);
        assert_eq!(reloaded[0].provider_id, original[0].provider_id);
        let next = expand(&snapshot, &library, &policy, &reloaded).unwrap();
        assert_eq!(next.channels[1].selection, ModelSelection::None);
        assert_eq!(
            next.channels[1].connection.nodes[0].url,
            "https://other.example.test/v1"
        );
    }
    #[test]
    fn adopted_channel_filters_are_resolved_once_before_partitioning() {
        let root = tempfile::tempdir().unwrap();
        let (mut library, mut policy, snapshot) = fixture();
        let channel = &mut policy.channels[0];
        let template = &mut library.templates[0];
        let content = &mut template.revisions[0].content;
        content.matching.include = vec![Pattern {
            kind: MatchKind::Exact,
            value: "anthropic-model".into(),
        }];
        content.matching.channel_mode = ChannelMode::Whitelist;
        content.matching.channels = BTreeSet::from([channel.id.clone()]);
        content.rules.context_window = Field::Custom(9000);
        channel.template_refs = vec![TemplateRef {
            template_id: template.id.clone(),
            revision_id: template.revisions[0].id.clone(),
        }];
        let mappings = mappings(root.path(), &library, &policy).unwrap();
        let expanded = expand(&snapshot, &library, &policy, &mappings).unwrap();
        let projected = compiler::project(&library, &expanded).unwrap();
        assert_eq!(
            projected
                .rows
                .iter()
                .find(|row| row.model_id == "anthropic-model")
                .unwrap()
                .rules
                .context_window,
            Field::Custom(9000)
        );
    }
    #[test]
    fn occupied_provider_identity_or_auth_boundary_is_refused_before_runtime_changes() {
        let root = tempfile::tempdir().unwrap();
        let (library, mut policy, mut snapshot) = fixture();
        let mappings = mappings(root.path(), &library, &policy).unwrap();
        snapshot.providers["providers"][&mappings[0].provider_id] =
            serde_json::json!({"adapter":"anthropic","note":"unrelated provider"});
        assert!(
            matches!(expand(&snapshot,&library,&policy,&mappings),Err(error) if error.code=="protocol_mapping_collision")
        );
        policy.channels[0].connection.protocol = "ollama-native".into();
        assert!(desired(&library, &policy).is_err());
    }
}
