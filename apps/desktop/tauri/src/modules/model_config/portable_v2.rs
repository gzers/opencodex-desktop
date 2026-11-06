//! Portable model v6 codec. Transport/encryption, credentials and application are L2/L0-owned.
use super::schema::{ModelNaming, ModelRules, ModelSelection, ServiceNode};
use super::schema_v2::*;
use super::source_v2::validate_policy_v2;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const PORTABLE_FORMAT: &str = "OCXD_MODEL_PACKAGE";
pub const PORTABLE_VERSION: u32 = 6;
pub const MAX_DOCUMENT_BYTES: usize = 32 * 1024 * 1024;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortableConnectionV2 {
    pub nodes: Vec<ServiceNode>,
    pub active_node_id: String,
    pub enabled: bool,
    pub allow_private_network: bool,
    pub auth_mode: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortableChannelV2 {
    pub id: String,
    pub label: String,
    pub connection: Option<PortableConnectionV2>,
    pub model_subset: Option<Vec<String>>,
    pub family_scope: Option<Vec<String>>,
    pub models: Vec<ModelConfigV2>,
    pub template_refs: Vec<RefV2>,
    pub selection: ModelSelection,
    pub model_order: Vec<String>,
    pub defaults: ModelRules,
    pub family_overrides: BTreeMap<String, ModelRules>,
    pub naming: BTreeMap<String, ModelNaming>,
    pub namespace: Option<NamespaceV2>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortablePolicyV2 {
    pub template_refs: Vec<RefV2>,
    pub channels: Vec<PortableChannelV2>,
    pub channel_order: Vec<String>,
    pub default_channel_id: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelPayloadV2 {
    pub templates: Vec<TemplateV2>,
    pub policy: Option<PortablePolicyV2>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortableModelsV2 {
    pub format: String,
    pub format_version: u32,
    pub schema_version: u32,
    pub created_at: String,
    pub digest: String,
    pub payload: ModelPayloadV2,
}
#[derive(Debug, Clone, Default)]
pub struct ModelExportScopeV2 {
    pub channel_ids: BTreeSet<String>,
    pub channel_models: BTreeMap<String, BTreeSet<String>>,
    pub channel_families: BTreeMap<String, BTreeSet<String>>,
    pub template_refs: Vec<RefV2>,
    pub include_global: bool,
}
fn refs_of(policy: &PortablePolicyV2) -> Vec<&RefV2> {
    policy
        .template_refs
        .iter()
        .chain(policy.channels.iter().flat_map(|c| {
            c.template_refs.iter().chain(
                c.models
                    .iter()
                    .filter_map(|m| m.binding.as_ref().map(ModelBindingV2::reference)),
            )
        }))
        .collect()
}
pub fn export_portable_models_v2(
    library: &LibraryV2,
    saved_policy: &PolicyV2,
    scope: &ModelExportScopeV2,
) -> Result<PortableModelsV2> {
    validate_library_v2(library)?;
    validate_policy_v2(saved_policy)?;
    let all: BTreeSet<_> = scope
        .channel_ids
        .iter()
        .chain(scope.channel_models.keys())
        .chain(scope.channel_families.keys())
        .collect();
    if all
        .iter()
        .any(|id| !saved_policy.channels.iter().any(|c| &c.id == *id))
        || scope
            .channel_models
            .keys()
            .any(|k| scope.channel_families.contains_key(k))
    {
        return Err(error("model_source_invalid"));
    }
    let mut channels = vec![];
    for channel in saved_policy.channels.iter().filter(|c| all.contains(&c.id)) {
        if matches!(
            channel.connection.provider_id.to_ascii_lowercase().as_str(),
            "openai" | "chatgpt" | "openai-multi" | "combo"
        ) {
            return Err(error("model_native_channel_not_portable"));
        }
        let subset = scope.channel_models.get(&channel.id);
        let family = scope.channel_families.get(&channel.id);
        if subset.is_some_and(|ids| {
            ids.iter()
                .any(|id| !channel.models.iter().any(|m| &m.raw_model_id == id))
        }) {
            return Err(error("model_source_invalid"));
        }
        let models: Vec<_> = channel
            .models
            .iter()
            .filter(|m| {
                subset.is_none_or(|ids| ids.contains(&m.raw_model_id))
                    && family.is_none_or(|ids| {
                        m.binding
                            .as_ref()
                            .is_some_and(|b| ids.contains(&b.reference().template_id))
                    })
            })
            .cloned()
            .collect();
        let model_ids: BTreeSet<_> = models.iter().map(|m| m.raw_model_id.clone()).collect();
        let connection = family.is_none().then(|| PortableConnectionV2 {
            nodes: channel.connection.nodes.clone(),
            active_node_id: channel.connection.active_node_id.clone(),
            enabled: channel.connection.enabled,
            allow_private_network: channel.connection.allow_private_network,
            auth_mode: channel.connection.auth_mode.clone(),
        });
        let selection = match &channel.selection {
            ModelSelection::Whitelist(ids) => ModelSelection::Whitelist(
                ids.iter()
                    .filter(|id| model_ids.contains(*id))
                    .cloned()
                    .collect(),
            ),
            other => other.clone(),
        };
        channels.push(PortableChannelV2 {
            id: channel.id.clone(),
            label: channel.label.clone(),
            connection,
            model_subset: subset.map(|ids| ids.iter().cloned().collect()),
            family_scope: family.map(|ids| ids.iter().cloned().collect()),
            models,
            template_refs: channel
                .template_refs
                .iter()
                .filter(|r| family.is_none_or(|ids| ids.contains(&r.template_id)))
                .cloned()
                .collect(),
            selection,
            model_order: channel
                .model_order
                .iter()
                .filter(|id| model_ids.contains(*id))
                .cloned()
                .collect(),
            defaults: if family.is_some() {
                ModelRules::default()
            } else {
                channel.defaults.clone()
            },
            family_overrides: channel
                .family_overrides
                .iter()
                .filter(|(id, _)| family.is_none_or(|ids| ids.contains(*id)))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            naming: channel
                .naming
                .iter()
                .filter(|(id, _)| model_ids.contains(*id))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            namespace: channel.namespace.clone(),
        });
    }
    let policy = PortablePolicyV2 {
        template_refs: if scope.include_global {
            saved_policy.template_refs.clone()
        } else {
            vec![]
        },
        channel_order: channels.iter().map(|c| c.id.clone()).collect(),
        default_channel_id: saved_policy
            .default_channel_id
            .as_ref()
            .filter(|id| all.contains(*id))
            .cloned(),
        channels,
    };
    let mut templates = BTreeSet::new();
    for reference in refs_of(&policy).into_iter().chain(&scope.template_refs) {
        revision(library, reference)?;
        templates.insert(reference.template_id.clone());
    }
    for channel in &policy.channels {
        for id in channel.family_overrides.keys() {
            if !library.templates.iter().any(|t| &t.id == id) {
                return Err(error("model_dependency_missing"));
            }
            templates.insert(id.clone());
        }
    }
    for ids in scope.channel_families.values() {
        for id in ids {
            if !library.templates.iter().any(|t| &t.id == id) {
                return Err(error("model_dependency_missing"));
            }
            templates.insert(id.clone());
        }
    }
    let payload = ModelPayloadV2 {
        templates: library
            .templates
            .iter()
            .filter(|t| templates.contains(&t.id))
            .cloned()
            .collect(),
        policy: (!policy.channels.is_empty() || scope.include_global).then_some(policy),
    };
    let package = PortableModelsV2 {
        format: PORTABLE_FORMAT.into(),
        format_version: PORTABLE_VERSION,
        schema_version: SCHEMA_VERSION,
        created_at: chrono::Utc::now().to_rfc3339(),
        digest: digest(&canonical_bytes(&payload)?),
        payload,
    };
    validate_portable_models_v2(&package)?;
    Ok(package)
}
pub fn validate_portable_models_v2(package: &PortableModelsV2) -> Result<()> {
    if package.format != PORTABLE_FORMAT
        || package.format_version != PORTABLE_VERSION
        || package.schema_version != SCHEMA_VERSION
    {
        return Err(error("model_schema_unsupported"));
    }
    chrono::DateTime::parse_from_rfc3339(&package.created_at)
        .map_err(|_| error("model_source_invalid"))?;
    if digest(&canonical_bytes(&package.payload)?) != package.digest {
        return Err(error("model_revision_digest_mismatch"));
    }
    let library = LibraryV2 {
        schema_version: SCHEMA_VERSION,
        templates: package.payload.templates.clone(),
        submission_receipts: BTreeMap::new(),
    };
    validate_library_v2(&library)?;
    if let Some(policy) = &package.payload.policy {
        for reference in refs_of(policy) {
            revision(&library, reference)?;
        }
        let mut ids = BTreeSet::new();
        for channel in &policy.channels {
            if !ids.insert(&channel.id)
                || channel.model_subset.is_some() && channel.family_scope.is_some()
            {
                return Err(error("model_source_invalid"));
            }
            if channel.family_scope.is_some() && channel.connection.is_some() {
                return Err(error("model_source_invalid"));
            }
            if let Some(subset) = &channel.model_subset {
                let selected: BTreeSet<_> = subset.iter().collect();
                if selected.len() != subset.len()
                    || channel
                        .models
                        .iter()
                        .any(|m| !selected.contains(&m.raw_model_id))
                    || selected.len() != channel.models.len()
                {
                    return Err(error("model_source_invalid"));
                }
            }
            if let Some(families) = &channel.family_scope {
                let set: BTreeSet<_> = families.iter().collect();
                if set.len() != families.len()
                    || families
                        .iter()
                        .any(|id| !library.templates.iter().any(|t| &t.id == id))
                    || channel
                        .template_refs
                        .iter()
                        .any(|r| !set.contains(&r.template_id))
                    || channel.family_overrides.keys().any(|id| !set.contains(id))
                    || channel.defaults != ModelRules::default()
                    || channel.models.iter().any(|m| {
                        m.binding
                            .as_ref()
                            .is_none_or(|b| !set.contains(&b.reference().template_id))
                    })
                {
                    return Err(error("model_source_invalid"));
                }
            }
            let c = ChannelV2 {
                id: channel.id.clone(),
                label: channel.label.clone(),
                connection: ConnectionV2 {
                    provider_id: format!("portable-{}", channel.id),
                    nodes: channel
                        .connection
                        .as_ref()
                        .map(|c| c.nodes.clone())
                        .unwrap_or_default(),
                    active_node_id: channel
                        .connection
                        .as_ref()
                        .map(|c| c.active_node_id.clone())
                        .unwrap_or_default(),
                    credential_ref: None,
                    enabled: channel.connection.as_ref().is_some_and(|c| c.enabled),
                    allow_private_network: channel
                        .connection
                        .as_ref()
                        .is_some_and(|c| c.allow_private_network),
                    auth_mode: channel
                        .connection
                        .as_ref()
                        .and_then(|c| c.auth_mode.clone()),
                    source_read_id: None,
                    imported_config: None,
                },
                selection: channel.selection.clone(),
                model_order: channel.model_order.clone(),
                models: channel.models.clone(),
                template_refs: channel.template_refs.clone(),
                defaults: channel.defaults.clone(),
                family_overrides: channel.family_overrides.clone(),
                naming: channel.naming.clone(),
                namespace: channel.namespace.clone(),
            };
            validate_policy_v2(&PolicyV2 {
                schema_version: SCHEMA_VERSION,
                channels: vec![c.clone()],
                template_refs: policy.template_refs.clone(),
                default_channel_id: None,
                pending_removals: vec![],
                recovery_id: None,
            })?;
            for model in &channel.models {
                super::matching_v2::resolve_effective_model_v2(&library, &c, model)?;
            }
        }
        if policy.channel_order.iter().collect::<BTreeSet<_>>() != ids
            || policy.channel_order.len() != ids.len()
            || policy
                .default_channel_id
                .as_ref()
                .is_some_and(|id| !ids.contains(id))
        {
            return Err(error("model_dependency_missing"));
        }
    }
    Ok(())
}
pub fn encode_portable_models_v2(value: &PortableModelsV2) -> Result<Vec<u8>> {
    validate_portable_models_v2(value)?;
    let bytes = canonical_bytes(value)?;
    if bytes.len() > MAX_DOCUMENT_BYTES {
        return Err(error("model_limit"));
    }
    Ok(bytes)
}
pub fn decode_portable_models_v2(bytes: &[u8]) -> Result<PortableModelsV2> {
    if bytes.len() > MAX_DOCUMENT_BYTES {
        return Err(error("model_limit"));
    }
    let header = parse_strict_json_v2(bytes)?;
    if header["format"] != PORTABLE_FORMAT
        || header["formatVersion"] != PORTABLE_VERSION
        || header["schemaVersion"] != SCHEMA_VERSION
    {
        return Err(error("model_schema_unsupported"));
    }
    let package = serde_json::from_value(header).map_err(json_error)?;
    validate_portable_models_v2(&package)?;
    Ok(package)
}
/// Pure merge never replaces immutable revisions or silently changes an existing default.
pub fn merge_library_v2(local: &LibraryV2, incoming: &LibraryV2) -> Result<LibraryV2> {
    validate_library_v2(local)?;
    validate_library_v2(incoming)?;
    let mut merged = local.clone();
    for template in &incoming.templates {
        if let Some(existing) = merged.templates.iter_mut().find(|t| t.id == template.id) {
            for revision in &template.revisions {
                if let Some(old) = existing.revisions.iter().find(|r| r.id == revision.id) {
                    if canonical_bytes(old)? != canonical_bytes(revision)? {
                        return Err(error("model_revision_digest_mismatch"));
                    }
                } else {
                    existing.revisions.push(revision.clone());
                }
            }
        } else {
            merged.templates.push(template.clone());
        }
    }
    validate_library_v2(&merged)?;
    Ok(merged)
}
