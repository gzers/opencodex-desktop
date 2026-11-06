//! Incoming provider materials are immutable local staging objects. Accepting
//! them writes a disabled draft; the normal runtime save owns actual adoption.
use super::{provider_auth, schema::*, store::sha256};
use crate::{
    infrastructure::atomic_write::atomic_write, modules::backup::safety, state::ProcessContext,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Staged {
    pub id: String,
    pub channel_id: String,
    pub provider_id: String,
    pub public: Value,
    pub authorization: Option<provider_auth::Backup>,
}
fn path(root: &Path, id: &str) -> ModelResult<PathBuf> {
    require_uuid(id)?;
    let path = root
        .join("manager-state/imported-provider-configs")
        .join(format!("{id}.json"));
    safety::check_path(root, &path, true)?;
    Ok(path)
}
pub fn stage(
    root: &Path,
    channel: &ChannelPolicy,
    public: &Value,
    auth: Option<&provider_auth::PortableAuthorization>,
) -> ModelResult<ImportedProviderRef> {
    super::provider_fields::validate_public(public)?;
    let id = uuid::Uuid::new_v4().to_string();
    let authorization = auth
        .map(|value| {
            provider_auth::stage_portable(root, &id, &channel.connection.provider_id, public, value)
        })
        .transpose()?;
    let value = Staged {
        id: id.clone(),
        channel_id: channel.id.clone(),
        provider_id: channel.connection.provider_id.clone(),
        public: public.clone(),
        authorization,
    };
    let bytes = serde_json::to_vec(&value)?;
    atomic_write(&path(root, &id)?, &bytes, 0o600)?;
    let reference = ImportedProviderRef {
        id,
        digest: sha256(&bytes),
    };
    load(root, channel, &reference)?;
    Ok(reference)
}
pub fn load(
    root: &Path,
    channel: &ChannelPolicy,
    reference: &ImportedProviderRef,
) -> ModelResult<Staged> {
    let bytes = safety::read_file(&path(root, &reference.id)?, 4 * 1024 * 1024)?;
    if sha256(&bytes) != reference.digest {
        return Err(invalid(
            "provider_import_digest",
            "原渠道接收材料已改变，未采用",
        ));
    }
    let value: Staged = serde_json::from_slice(&bytes)?;
    if value.id != reference.id
        || value.channel_id != channel.id
        || value.provider_id != channel.connection.provider_id
    {
        return Err(invalid(
            "provider_import_identity",
            "原渠道材料属于其他接收目标，未复用授权",
        ));
    }
    super::provider_fields::validate_public(&value.public)?;
    if value.public["adapter"] != channel.connection.protocol
        || !channel
            .connection
            .nodes
            .iter()
            .any(|node| value.public["baseUrl"] == node.url)
    {
        return Err(invalid(
            "provider_import_direction",
            "原渠道材料与当前协议或节点不同，请重新比较",
        ));
    }
    Ok(value)
}
pub fn authorization(
    root: &Path,
    context: &ProcessContext,
    value: &Staged,
) -> ModelResult<Option<provider_auth::Backup>> {
    value
        .authorization
        .as_ref()
        .map(|backup| {
            provider_auth::bind_portable(
                root,
                context,
                &value.id,
                &value.provider_id,
                &value.public,
                backup,
            )
        })
        .transpose()
}

/// L0 provides stable logical identities. Reading config never invents a UUID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderBindingV2 {
    pub channel_id: String,
    pub provider_id: String,
}
fn string_list_v2(value: Option<&Value>) -> super::schema_v2::Result<Vec<String>> {
    use super::schema_v2::error;
    value
        .map(|v| {
            v.as_array()
                .ok_or_else(|| error("model_source_invalid"))
                .and_then(|a| {
                    a.iter()
                        .map(|v| {
                            v.as_str()
                                .map(String::from)
                                .ok_or_else(|| error("model_source_invalid"))
                        })
                        .collect()
                })
        })
        .transpose()
        .map(Option::unwrap_or_default)
}
/// The caller holds config→model-v2 locks. Raw JSON and secrets never enter the
/// returned DTO; only known source rule fields and existing credential refs do.
pub fn read_policy_v2(
    config: &Value,
    source_digest: &str,
    metadata: &super::schema_v2::PolicyV2,
    ledger: Option<&super::source_v2::MaterializationLedgerV2>,
    mappings: &super::protocol_groups::MappingFileV2,
    bindings: &[ProviderBindingV2],
) -> super::schema_v2::Result<super::schema_v2::PolicyV2> {
    use super::{schema_v2 as v2, source_v2, store_v2::empty_policy_v2};
    use std::collections::{BTreeMap, BTreeSet};
    source_v2::validate_policy_v2(metadata)?;
    super::protocol_groups::validate_mappings_v2(mappings)?;
    if !config.is_object() || !v2::valid_digest(source_digest) {
        return Err(v2::error("model_source_invalid"));
    }
    let empty = serde_json::Map::new();
    let providers = config
        .get("providers")
        .map(|v| {
            v.as_object()
                .ok_or_else(|| v2::error("model_source_invalid"))
        })
        .transpose()?
        .unwrap_or(&empty);
    let picker = string_list_v2(config.get("modelPickerOrder"))?;
    let disabled = string_list_v2(config.get("disabledModels"))?;
    let expected_policy = source_v2::policy_semantic_digest_v2(metadata)?;
    let mut policy = empty_policy_v2();
    policy.template_refs = metadata.template_refs.clone();
    let mut identities = BTreeSet::new();
    let mut ownership = BTreeMap::new();
    for binding in bindings {
        v2::check_id(&binding.channel_id)?;
        if !identities.insert(&binding.channel_id) {
            return Err(v2::error("model_runtime_mapping_conflict"));
        }
        let aliases = std::iter::once(binding.provider_id.clone())
            .chain(
                mappings
                    .mappings
                    .iter()
                    .filter(|m| m.channel_id == binding.channel_id)
                    .map(|m| m.provider_id.clone()),
            )
            .collect::<BTreeSet<_>>();
        for alias in &aliases {
            if ownership
                .insert(alias.clone(), binding.channel_id.clone())
                .is_some_and(|old| old != binding.channel_id)
            {
                return Err(v2::error("model_runtime_mapping_conflict"));
            }
        }
        let old = metadata
            .channels
            .iter()
            .find(|c| c.id == binding.channel_id);
        let anchor = providers.get(&binding.provider_id);
        if anchor.is_none() {
            if old.is_some() {
                return Err(v2::error("model_source_provenance_unknown"));
            } else {
                continue;
            }
        }
        let anchor = anchor.unwrap();
        if !anchor.is_object() {
            return Err(v2::error("model_source_invalid"));
        }
        let mut nodes = old.map(|c| c.connection.nodes.clone()).unwrap_or_default();
        let url = anchor.get("baseUrl").and_then(Value::as_str).unwrap_or("");
        let active = if let Some(node) = nodes.iter().find(|n| n.url == url) {
            node.id.clone()
        } else if let Some(node) = nodes
            .iter_mut()
            .find(|n| old.is_some_and(|c| c.connection.active_node_id == n.id))
        {
            node.url = url.into();
            node.id.clone()
        } else {
            // A fresh node uses the explicit binding identity; it is not a discovered path.
            if !url.is_empty() {
                nodes.push(ServiceNode {
                    id: binding.channel_id.clone(),
                    name: "当前节点".into(),
                    url: url.into(),
                });
            }
            if url.is_empty() {
                String::new()
            } else {
                binding.channel_id.clone()
            }
        };
        let mut channel = v2::ChannelV2 {
            id: binding.channel_id.clone(),
            label: old
                .map(|c| c.label.clone())
                .or_else(|| anchor.get("name").and_then(Value::as_str).map(String::from))
                .unwrap_or_else(|| binding.provider_id.clone()),
            connection: v2::ConnectionV2 {
                provider_id: binding.provider_id.clone(),
                nodes,
                active_node_id: active,
                credential_ref: old.and_then(|c| c.connection.credential_ref.clone()),
                enabled: anchor.get("disabled") != Some(&Value::Bool(true)),
                allow_private_network: anchor.get("allowPrivateNetwork")
                    == Some(&Value::Bool(true)),
                auth_mode: anchor
                    .get("authMode")
                    .and_then(Value::as_str)
                    .map(String::from),
                source_read_id: None,
                imported_config: None,
            },
            selection: ModelSelection::None,
            model_order: vec![],
            models: vec![],
            template_refs: old.map(|c| c.template_refs.clone()).unwrap_or_default(),
            defaults: old.map(|c| c.defaults.clone()).unwrap_or_default(),
            family_overrides: old.map(|c| c.family_overrides.clone()).unwrap_or_default(),
            naming: BTreeMap::new(),
            namespace: old.and_then(|c| c.namespace.clone()),
        };
        let mut selected = BTreeSet::new();
        let mut model_providers = BTreeMap::new();
        let mut all_selected = true;
        let mut aliases = aliases.into_iter().collect::<Vec<_>>();
        aliases.sort_by_key(|p| if p == &binding.provider_id { 0 } else { 1 });
        for alias in aliases {
            let Some(raw) = providers.get(&alias) else {
                return Err(v2::error("model_source_provenance_unknown"));
            };
            if !raw.is_object() {
                return Err(v2::error("model_source_invalid"));
            }
            if let Some(mapping) = mappings
                .mappings
                .iter()
                .find(|m| m.provider_id == alias && !m.anchor)
            {
                if raw["note"] != super::protocol_groups::mapping_marker_v2(mapping) {
                    return Err(v2::error("model_runtime_mapping_conflict"));
                }
            }
            let mut ids = Vec::new();
            let mut seen = BTreeSet::new();
            for key in ["models", "retainModels"] {
                for id in string_list_v2(raw.get(key))? {
                    if seen.insert(id.clone()) {
                        ids.push(id);
                    }
                }
            }
            for key in [
                "modelAdapters",
                "modelContextWindows",
                "modelInputModalities",
                "modelReasoningEfforts",
                "modelDefaultReasoningEfforts",
                "modelSupportsServiceTier",
                "modelPinnedReasoningEfforts",
                "modelReasoningEffortMap",
                "modelCapabilities",
                "modelAliases",
                "modelDisplayNames",
            ] {
                if let Some(map) = raw.get(key) {
                    for id in map
                        .as_object()
                        .ok_or_else(|| v2::error("model_source_invalid"))?
                        .keys()
                    {
                        if !id.contains(['*', '?']) && seen.insert(id.clone()) {
                            ids.push(id.clone());
                        }
                    }
                }
            }
            let explicit = raw
                .get("selectedModels")
                .map(|v| string_list_v2(Some(v)))
                .transpose()?
                .map(|selected| {
                    selected
                        .into_iter()
                        .map(|raw| {
                            if ids.contains(&raw) {
                                raw
                            } else {
                                native_id_for_slug_v2(&raw, &alias, &ids).unwrap_or(raw)
                            }
                        })
                        .collect::<Vec<_>>()
                });
            for id in explicit.iter().flatten() {
                if seen.insert(id.clone()) {
                    ids.push(id.clone());
                }
            }
            let pending = raw["initialModelSelection"]["version"] == 1
                && raw["initialModelSelection"]["status"] == "pending";
            let auto = raw
                .get("newModelPolicy")
                .and_then(Value::as_str)
                .or_else(|| config["modelDiscovery"]["newModelPolicy"].as_str())
                .unwrap_or("on")
                == "on";
            all_selected &= explicit.is_none() && auto && !pending;
            for id in ids {
                if model_providers.insert(id.clone(), alias.clone()).is_some() {
                    return Err(v2::error("model_runtime_mapping_conflict"));
                }
                let imported = source_v2::source_from_provider_v2(
                    &alias,
                    &id,
                    raw,
                    source_digest,
                    v2::SourceKindV2::ImportedFile,
                )?;
                let saved = old.and_then(|c| c.models.iter().find(|m| m.raw_model_id == id));
                let source = if let Some(saved) = saved {
                    source_v2::reconcile_source_v2(
                        &binding.channel_id,
                        &saved.source,
                        &imported,
                        ledger,
                        &expected_policy,
                        config,
                    )?
                } else {
                    imported
                };
                let visible = !disabled.iter().any(|slug| {
                    slug == &format!("{alias}/{id}")
                        || slug == &format!("{alias}/{}", id.replace('/', "-"))
                });
                let picked = !pending
                    && visible
                    && explicit.as_ref().map_or(auto, |ids| {
                        ids.contains(&id) || ids.contains(&format!("{alias}/{id}"))
                    });
                if picked {
                    selected.insert(id.clone());
                } else {
                    all_selected = false;
                }
                channel.model_order.push(id.clone());
                let naming = ModelNaming {
                    alias: raw["modelAliases"][&id].as_str().map(String::from),
                    display_name: raw["modelDisplayNames"][&id].as_str().map(String::from),
                };
                if naming.alias.is_some() || naming.display_name.is_some() {
                    channel.naming.insert(id.clone(), naming.clone());
                }
                channel.models.push(v2::ModelConfigV2 {
                    raw_model_id: id.clone(),
                    label: naming.display_name.unwrap_or(id),
                    observed: true,
                    source,
                    explicit_overrides: saved
                        .map(|m| m.explicit_overrides.clone())
                        .unwrap_or_default(),
                    binding: saved.and_then(|m| m.binding.clone()),
                });
            }
        }
        let mut order = vec![];
        let mut seen = BTreeSet::new();
        for slug in &picker {
            for id in &channel.model_order {
                if model_providers
                    .get(id)
                    .is_some_and(|provider| slug == &format!("{provider}/{id}"))
                    && seen.insert(id.clone())
                {
                    order.push(id.clone());
                }
            }
        }
        for id in &channel.model_order {
            if seen.insert(id.clone()) {
                order.push(id.clone());
            }
        }
        channel.model_order = order;
        channel.selection = if all_selected && selected.len() == channel.models.len() {
            ModelSelection::All
        } else if selected.is_empty() {
            ModelSelection::None
        } else {
            ModelSelection::Whitelist(
                channel
                    .model_order
                    .iter()
                    .filter(|id| selected.contains(*id))
                    .cloned()
                    .collect(),
            )
        };
        policy.channels.push(channel);
    }
    policy.default_channel_id = config
        .get("defaultProvider")
        .and_then(Value::as_str)
        .and_then(|id| ownership.get(id))
        .filter(|id| policy.channels.iter().any(|c| &c.id == *id))
        .cloned();
    source_v2::validate_policy_v2(&policy)?;
    Ok(policy)
}

fn native_id_for_slug_v2(slug: &str, provider: &str, ids: &[String]) -> Option<String> {
    if let Some(id) = ids.iter().find(|id| slug == format!("{provider}/{id}")) {
        return Some(id.clone());
    }
    let matches = ids
        .iter()
        .filter(|id| slug == format!("{provider}/{}", id.replace('/', "-")))
        .collect::<Vec<_>>();
    if matches.len() == 1 {
        Some(matches[0].clone())
    } else {
        None
    }
}
