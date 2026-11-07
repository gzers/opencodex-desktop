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
    /// Validated public source material only; never a channel protocol or key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<serde_json::Value>,
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
            provider: None,
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
            if let Some(public) = channel
                .connection
                .as_ref()
                .and_then(|c| c.provider.as_ref())
            {
                validate_public_source_v2(channel, public)?;
                if channel.model_subset.is_some()
                    && scoped_public_source_v2(channel, public)? != *public
                {
                    return Err(error("model_public_source_scope"));
                }
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

fn validate_public_source_v2(
    channel: &PortableChannelV2,
    public: &serde_json::Value,
) -> Result<()> {
    super::provider_fields::validate_public(public).map_err(|err| error(&err.code))?;
    if public.get("adapter").is_some_and(|v| !v.is_string()) {
        return Err(error("model_source_invalid"));
    }
    if let Some(url) = public.get("baseUrl") {
        let url = url.as_str().ok_or_else(|| error("model_source_invalid"))?;
        if !channel
            .connection
            .as_ref()
            .is_some_and(|c| c.nodes.iter().any(|node| node.url == url))
        {
            return Err(error("model_public_source_direction"));
        }
    }
    Ok(())
}
/// Reuse the established transfer field scoper. This DTO bridge carries no
/// credentials, chosen template, or matching authority and calls no resolver.
fn scoped_public_source_v2(
    channel: &PortableChannelV2,
    public: &serde_json::Value,
) -> Result<serde_json::Value> {
    let Some(subset) = &channel.model_subset else {
        return Ok(public.clone());
    };
    for key in [
        "modelAliases",
        "modelDisplayNames",
        "modelAdapters",
        "modelSupportsServiceTier",
        "modelContextWindows",
        "modelContextTiers",
        "modelInputModalities",
        "modelCapabilities",
        "modelAutoCompactTokenLimits",
        "modelMaxOutputTokens",
        "modelCosts",
        "modelOpenRouterRouting",
        "modelVercelGatewayRouting",
        "modelReasoningEfforts",
        "modelSuppressSyntheticMax",
        "modelDefaultReasoningEfforts",
        "modelPinnedReasoningEfforts",
        "modelSupportsReasoningSummaries",
        "modelSupportsVerbosity",
        "modelReasoningSummaryDelivery",
        "modelPreferHostedTools",
        "modelReasoningEffortMap",
    ] {
        if public.get(key).is_some_and(|v| !v.is_object()) {
            return Err(error("model_public_source_scope"));
        }
    }
    let connection = channel
        .connection
        .as_ref()
        .ok_or_else(|| error("model_public_source_scope"))?;
    if public.get("liveModels").is_some_and(|v| !v.is_boolean()) {
        return Err(error("model_public_source_scope"));
    }
    let source_models: [&str; 3] = ["models", "retainModels", "selectedModels"];
    let mut observed_ids = BTreeSet::new();
    for key in source_models {
        if let Some(value) = public.get(key) {
            for id in value
                .as_array()
                .ok_or_else(|| error("model_public_source_scope"))?
            {
                observed_ids.insert(
                    id.as_str()
                        .ok_or_else(|| error("model_public_source_scope"))?
                        .to_string(),
                );
            }
        }
    }
    let preset = public
        .get("modelPreset")
        .cloned()
        .map(|mut marker| {
            let fields = marker
                .as_object_mut()
                .ok_or_else(|| error("model_public_source_scope"))?;
            if !matches!(
                fields.get("mode").and_then(serde_json::Value::as_str),
                Some("preset" | "all" | "custom")
            ) {
                return Err(error("model_public_source_scope"));
            }
            fields.insert("mode".into(), serde_json::json!("custom"));
            Ok(marker)
        })
        .transpose()?;
    // modelPreset is a provider marker, not a raw-model-keyed map. Avoid the
    // old scoper's broad model* object rule and restore it after narrowing.
    let mut bridge_public = public.clone();
    bridge_public.as_object_mut().unwrap().remove("modelPreset");
    let bridge = super::transfer::PortableChannel {
        family_scope: None,
        id: channel.id.clone(),
        label: channel.label.clone(),
        // Scope filtering does not resolve protocols; the literal source value
        // remains material only. There is no fabricated fallback protocol.
        protocol: public
            .get("adapter")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .into(),
        model_subset: None,
        connection: Some(super::transfer::PortableConnection {
            nodes: connection.nodes.clone(),
            provider: Some(bridge_public),
        }),
        selection: channel.selection.clone(),
        model_order: channel.model_order.clone(),
        discovered: channel
            .models
            .iter()
            .map(|m| super::schema::DiscoveredModel {
                id: m.raw_model_id.clone(),
                label: m.label.clone(),
                rules: ModelRules::default(),
            })
            .collect(),
        template_refs: vec![],
        model_refs: BTreeMap::new(),
        defaults: ModelRules::default(),
        overrides: BTreeMap::new(),
        family_overrides: BTreeMap::new(),
        naming: channel.naming.clone(),
    };
    let scoped =
        super::transfer::restrict_models(&bridge, subset).map_err(|err| error(&err.code))?;
    let mut public = scoped
        .connection
        .and_then(|c| c.provider)
        .ok_or_else(|| error("model_public_source_scope"))?;
    let fields = public
        .as_object_mut()
        .ok_or_else(|| error("model_source_invalid"))?;
    let wanted: BTreeSet<_> = subset.iter().collect();
    fields.insert("liveModels".into(), serde_json::json!(false));
    if let Some(preset) = preset {
        fields.insert("modelPreset".into(), preset);
    }
    // Existing restrict_models handles every model* object, exact model IDs,
    // models/selectedModels, newModelPolicy and defaultModel. Extend its same
    // explicit mask to the other known declarations/deny lists and review ref.
    for (key, value) in fields.iter_mut() {
        if key != "liveModels" && (key == "retainModels" || key.ends_with("Models")) {
            let list = value
                .as_array_mut()
                .ok_or_else(|| error("model_public_source_scope"))?;
            if list.iter().any(|v| !v.is_string()) {
                return Err(error("model_public_source_scope"));
            }
            list.retain(|v| {
                v.as_str()
                    .is_some_and(|id| wanted.iter().any(|wanted| wanted.as_str() == id))
            });
        }
    }
    if fields.get("defaultModel").is_some_and(|v| !v.is_string()) {
        return Err(error("model_public_source_scope"));
    }
    if fields
        .get("defaultModel")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|id| !wanted.iter().any(|wanted| wanted.as_str() == id))
    {
        fields.remove("defaultModel");
    }
    // Fixed npm 2.97.0 types/provider.ts: exact upstream model IDs map
    // to independent review catalog selectors. Values are not scope IDs.
    if let Some(value) = fields.get_mut("autoReviewModelOverrides") {
        let map = value
            .as_object_mut()
            .ok_or_else(|| error("model_public_source_scope"))?;
        if map.values().any(|v| !v.is_string() && !v.is_null()) {
            return Err(error("model_public_source_scope"));
        }
        map.retain(|id, _| {
            wanted.contains(id)
                || (!observed_ids.contains(id)
                    && wanted
                        .iter()
                        .any(|raw| raw.contains('/') && raw.replace('/', "-") == *id))
        });
    }
    Ok(public)
}
/// Attach only validated, explicitly captured source objects to selected
/// channels. Scope narrowing is pure and payload digests are always refreshed.
pub fn attach_public_sources_v2(
    package: &PortableModelsV2,
    sources: &BTreeMap<String, serde_json::Value>,
) -> Result<PortableModelsV2> {
    validate_portable_models_v2(package)?;
    let mut next = package.clone();
    for (id, public) in sources {
        let channel = next
            .payload
            .policy
            .as_mut()
            .and_then(|p| p.channels.iter_mut().find(|c| &c.id == id))
            .ok_or_else(|| error("model_public_source_scope"))?;
        if channel.family_scope.is_some() || channel.connection.is_none() {
            return Err(error("model_public_source_scope"));
        }
        validate_public_source_v2(channel, public)?;
        let public = scoped_public_source_v2(channel, public)?;
        channel.connection.as_mut().unwrap().provider = Some(public);
    }
    next.digest = digest(&canonical_bytes(&next.payload)?);
    validate_portable_models_v2(&next)?;
    Ok(next)
}
