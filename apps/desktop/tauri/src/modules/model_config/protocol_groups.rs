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
    pub provider_id: String,
}
pub fn marker(channel: &str, protocol: &str) -> String {
    format!("OpenCodeX channel={channel} protocol={protocol}")
}
fn desired(library: &Library, policy: &Policy) -> ModelResult<BTreeSet<(String, String)>> {
    let projection = compiler::project(library, policy)?;
    let mut values = BTreeSet::new();
    for channel in &policy.channels {
        for row in projection
            .rows
            .iter()
            .filter(|row| row.channel_id == channel.id)
        {
            let protocol = match &row.rules.protocol {
                Field::Custom(value) => value,
                Field::Inherit => &channel.connection.protocol,
            };
            if protocol != &channel.connection.protocol
                && !["openai-chat", "openai-responses"].contains(&protocol.as_str())
            {
                if !["anthropic", "google", "ollama-native"].contains(&protocol.as_str())
                    || ![
                        "openai-chat",
                        "openai-responses",
                        "anthropic",
                        "google",
                        "ollama-native",
                    ]
                    .contains(&channel.connection.protocol.as_str())
                {
                    return Err(invalid(
                        "protocol_credentials",
                        "专用认证协议不能按普通Key复制到协议分组",
                    ));
                }
                if protocol == "ollama-native" || channel.connection.protocol == "ollama-native" {
                    return Err(invalid(
                        "protocol_credentials",
                        "本机无认证与密钥协议不能混用同一渠道认证",
                    ));
                }
                values.insert((channel.id.clone(), protocol.clone()));
            }
        }
    }
    Ok(values)
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
        if !keys.insert((&value.channel_id, &value.protocol))
            || !ids.insert(&value.id)
            || !providers.insert(&value.provider_id)
            || value.provider_id != format!("ocxd-{}", value.id.replace('-', ""))
        {
            return Err(invalid("protocol_mapping", "协议分组身份重复或被更改"));
        }
    }
    let before = serde_json::to_vec(&values)?;
    for (channel_id, protocol) in desired(library, policy)? {
        if !values
            .iter()
            .any(|value| value.channel_id == channel_id && value.protocol == protocol)
        {
            let id = uuid::Uuid::new_v4().to_string();
            values.push(Mapping {
                provider_id: format!("ocxd-{}", id.replace('-', "")),
                id,
                channel_id,
                protocol,
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
        let mut groups: BTreeMap<String, Vec<&compiler::ModelProjection>> = BTreeMap::new();
        for row in projection
            .rows
            .iter()
            .filter(|row| row.channel_id == channel.id)
        {
            let protocol = match &row.rules.protocol {
                Field::Custom(value) => value,
                Field::Inherit => &channel.connection.protocol,
            };
            let bucket = if protocol == &channel.connection.protocol
                || ["openai-chat", "openai-responses"].contains(&protocol.as_str())
            {
                channel.connection.protocol.clone()
            } else {
                protocol.clone()
            };
            groups.entry(bucket).or_default().push(row);
        }
        let mut destinations = vec![(
            channel.id.clone(),
            channel.connection.provider_id.clone(),
            channel.connection.protocol.clone(),
            false,
        )];
        for mapping in own {
            if let Some(prior) = snapshot.providers["providers"].get(&mapping.provider_id) {
                if prior["note"] != marker(&channel.id, &mapping.protocol) {
                    return Err(invalid(
                        "protocol_mapping_collision",
                        "协议分组标识已被其他配置占用，未覆盖",
                    ));
                }
            }
            if desired.contains(&(channel.id.clone(), mapping.protocol.clone()))
                || snapshot.providers["providers"]
                    .get(&mapping.provider_id)
                    .is_some()
            {
                destinations.push((
                    mapping.id.clone(),
                    mapping.provider_id.clone(),
                    mapping.protocol.clone(),
                    true,
                ));
            }
        }
        for (id, provider, protocol, is_group) in destinations {
            let rows = if is_group && protocol == channel.connection.protocol {
                vec![]
            } else {
                groups.get(&protocol).cloned().unwrap_or_default()
            };
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
pub fn decorate(next: &mut Value, policy: &Policy, mappings: &[Mapping]) {
    for mapping in mappings {
        if !policy.channels.iter().any(|c| c.id == mapping.channel_id) {
            continue;
        }
        if let Some(provider) = next["providers"].get_mut(&mapping.provider_id) {
            provider["note"] = serde_json::json!(marker(&mapping.channel_id, &mapping.protocol));
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
    let mut desired = vec![];
    for row in projection
        .rows
        .iter()
        .filter(|row| row.enabled && row.selected)
    {
        let channel = policy
            .channels
            .iter()
            .find(|c| c.id == row.channel_id)
            .ok_or_else(|| invalid("channel", "渠道身份缺失"))?;
        let protocol = match &row.rules.protocol {
            Field::Custom(value) => value,
            Field::Inherit => &channel.connection.protocol,
        };
        let provider = if protocol == &channel.connection.protocol
            || ["openai-chat", "openai-responses"].contains(&protocol.as_str())
        {
            channel.connection.provider_id.as_str()
        } else {
            mappings
                .iter()
                .find(|m| m.channel_id == channel.id && m.protocol == *protocol)
                .map(|m| m.provider_id.as_str())
                .ok_or_else(|| invalid("protocol_mapping", "模型协议没有稳定分组"))?
        };
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
