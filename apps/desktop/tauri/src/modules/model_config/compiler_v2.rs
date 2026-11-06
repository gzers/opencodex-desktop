//! Materialize v2 model semantics into an already-read config Value. Pure: L0
//! supplies allocated runtime identities and owns rename, locking and commit.
use super::{
    protocol_groups::{mapping_for_group_v2, validate_mappings_v2, MappingFileV2},
    schema_v2::*,
    source_v2::*,
};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub struct CompiledFileV2 {
    pub next_config: Value,
    pub policy: PolicyV2,
    pub ledger: MaterializationLedgerV2,
    pub mappings: MappingFileV2,
    pub targets: Vec<MaterializedTargetV2>,
    pub plan: ModelMaterializationPlanV2,
}
fn escaped(s: &str) -> String {
    s.replace('~', "~0").replace('/', "~1")
}
fn strings(value: Option<&Value>) -> Result<Vec<String>> {
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
fn set_exact(
    row: &mut Map<String, Value>,
    map: &str,
    id: &str,
    value: Option<&Value>,
) -> Result<()> {
    if value.is_none() && !row.contains_key(map) {
        return Ok(());
    }
    let values = row
        .entry(map.to_string())
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| error("model_source_invalid"))?;
    match value {
        Some(value) => {
            values.insert(id.into(), value.clone());
        }
        None => {
            values.remove(id);
        }
    }
    Ok(())
}
fn targets_for(row: &EffectiveModelV2, provider: &str) -> Vec<MaterializedTargetV2> {
    let root = format!("/providers/{}/", escaped(provider));
    let per_model = [
        ("protocol", "modelAdapters"),
        ("contextWindow", "modelContextWindows"),
        ("inputModalities", "modelInputModalities"),
        ("reasoningEfforts", "modelReasoningEfforts"),
        ("defaultEffort", "modelDefaultReasoningEfforts"),
        ("fast", "modelSupportsServiceTier"),
        ("pinnedEffort", "modelPinnedReasoningEfforts"),
        ("effortMap", "modelReasoningEffortMap"),
    ];
    let mut targets = per_model
        .into_iter()
        .map(|(field, map)| MaterializedTargetV2 {
            channel_id: row.channel_id.clone(),
            raw_model_id: row.raw_model_id.clone(),
            field: field.into(),
            provider_id: provider.into(),
            pointer: format!("{root}{map}/{}", escaped(&row.raw_model_id)),
        })
        .collect::<Vec<_>>();
    for (field, key) in [
        ("noReasoning", "noReasoningModels"),
        (
            "reasoningAuthoritative",
            "modelReasoningEffortsAuthoritative",
        ),
        ("fastWire", "fastWire"),
    ] {
        targets.push(MaterializedTargetV2 {
            channel_id: row.channel_id.clone(),
            raw_model_id: row.raw_model_id.clone(),
            field: field.into(),
            provider_id: provider.into(),
            pointer: format!("{root}{key}"),
        });
    }
    targets
}
pub fn compile_file_v2(
    library: &LibraryV2,
    before: &PolicyV2,
    intended: &PolicyV2,
    raw: &Value,
    mappings: &MappingFileV2,
    auth: &[ProtocolAuthBindingV2],
) -> Result<CompiledFileV2> {
    validate_mappings_v2(mappings)?;
    if !raw.is_object() || raw.get("providers").is_some_and(|v| !v.is_object()) {
        return Err(error("model_source_invalid"));
    }
    let plan = plan_materialization_v2(library, before, intended)?;
    validate_protocol_group_auth_v2(&plan.groups, auth)?;
    let verified_mappings = super::protocol_groups::plan_mappings_v2(mappings, &plan.groups, &[])?;
    let mut next = raw.clone();
    if next.get("providers").is_none() {
        next["providers"] = json!({});
    }
    let original = raw
        .get("providers")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut destinations = BTreeMap::new();
    let mut active_by_channel: BTreeMap<String, Vec<(String, Vec<String>)>> = BTreeMap::new();
    for group in &plan.groups {
        let mapping = mapping_for_group_v2(mappings, group)?;
        let channel = intended
            .channels
            .iter()
            .find(|c| c.id == group.channel_id)
            .ok_or_else(|| error("model_runtime_mapping_conflict"))?;
        if mapping.anchor && mapping.provider_id != channel.connection.provider_id {
            return Err(error("model_runtime_mapping_conflict"));
        }
        // A mapped destination already present in raw must be known to this
        // logical channel in before; no collision can overwrite another source.
        if original.get(&mapping.provider_id).is_some_and(|row| {
            !mapping.anchor && row["note"] != super::protocol_groups::mapping_marker_v2(mapping)
        }) {
            return Err(error("model_runtime_mapping_conflict"));
        }
        let template = original
            .get(&mapping.provider_id)
            .or_else(|| original.get(&channel.connection.provider_id))
            .cloned()
            .unwrap_or_else(|| json!({}));
        let mut row = template
            .as_object()
            .cloned()
            .ok_or_else(|| error("model_source_invalid"))?;
        if !original.contains_key(&mapping.provider_id)
            && !original.contains_key(&channel.connection.provider_id)
        {
            return Err(error("model_runtime_mapping_conflict"));
        }
        row.insert("adapter".into(), json!(group.protocol.as_str()));
        if !mapping.anchor {
            row.insert(
                "note".into(),
                json!(super::protocol_groups::mapping_marker_v2(mapping)),
            );
        }
        if let Some(node) = channel
            .connection
            .nodes
            .iter()
            .find(|n| n.id == channel.connection.active_node_id)
        {
            row.insert("baseUrl".into(), json!(node.url));
        }
        row.insert("disabled".into(), json!(!channel.connection.enabled));
        let ids = channel
            .model_order
            .iter()
            .filter(|id| group.raw_model_ids.contains(id))
            .cloned()
            .chain(
                group
                    .raw_model_ids
                    .iter()
                    .filter(|id| !channel.model_order.contains(id))
                    .cloned(),
            )
            .collect::<Vec<_>>();
        // Group projection must not retain another group's declarations.
        let managed_ids: BTreeSet<_> = channel
            .models
            .iter()
            .map(|m| m.raw_model_id.as_str())
            .collect();
        for key in [
            "modelAdapters",
            "modelContextWindows",
            "modelInputModalities",
            "modelReasoningEfforts",
            "modelDefaultReasoningEfforts",
            "modelSupportsServiceTier",
            "modelPinnedReasoningEfforts",
            "modelReasoningEffortMap",
            "modelAliases",
            "modelDisplayNames",
            "modelCapabilities",
        ] {
            if let Some(value) = row.get_mut(key) {
                let map = value
                    .as_object_mut()
                    .ok_or_else(|| error("model_source_invalid"))?;
                map.retain(|id, _| !managed_ids.contains(id.as_str()) || ids.contains(id));
            }
        }
        row.insert("models".into(), json!(ids));
        row.insert("retainModels".into(), json!(ids));
        let selected = ids
            .iter()
            .filter(|id| {
                plan.effective
                    .iter()
                    .any(|m| m.channel_id == channel.id && m.raw_model_id == **id && m.selected)
            })
            .cloned()
            .collect::<Vec<_>>();
        row.insert("selectedModels".into(), json!(selected));
        let mut no_reasoning = strings(row.get("noReasoningModels"))?;
        no_reasoning.retain(|id| !managed_ids.contains(id.as_str()));
        for id in &ids {
            // When a model moves back from a split provider, transfer its
            // public per-model objects before overlaying known rule fields.
            // This preserves opaque subfields such as capability extensions.
            let owners = original
                .iter()
                .filter(|(provider, raw)| {
                    (*provider == &channel.connection.provider_id
                        || mappings
                            .mappings
                            .iter()
                            .any(|m| m.channel_id == channel.id && &m.provider_id == *provider))
                        && ["models", "retainModels"].iter().any(|key| {
                            raw.get(*key)
                                .and_then(Value::as_array)
                                .is_some_and(|models| models.iter().any(|m| m.as_str() == Some(id)))
                        })
                })
                .collect::<Vec<_>>();
            if owners.len() > 1 {
                return Err(error("model_runtime_mapping_conflict"));
            }
            if let Some((_, previous)) = owners.first() {
                for key in super::provider_fields::PUBLIC
                    .iter()
                    .filter(|key| key.starts_with("model"))
                {
                    if let Some(value) = previous
                        .get(*key)
                        .and_then(Value::as_object)
                        .and_then(|map| map.get(id))
                    {
                        set_exact(&mut row, key, id, Some(value))?;
                    }
                }
            }
            let model = plan
                .effective
                .iter()
                .find(|m| m.channel_id == channel.id && m.raw_model_id == *id)
                .ok_or_else(|| error("model_source_invalid"))?;
            let values = rules_values(&model.rules)?;
            for (field, map) in [
                ("protocol", "modelAdapters"),
                ("contextWindow", "modelContextWindows"),
                ("inputModalities", "modelInputModalities"),
                ("reasoningEfforts", "modelReasoningEfforts"),
                ("defaultEffort", "modelDefaultReasoningEfforts"),
                ("fast", "modelSupportsServiceTier"),
                ("pinnedEffort", "modelPinnedReasoningEfforts"),
                ("effortMap", "modelReasoningEffortMap"),
            ] {
                set_exact(&mut row, map, id, values.get(field))?;
            }
            // modelCapabilities takes precedence in the original reader.
            if let Some(cap) = row
                .get_mut("modelCapabilities")
                .and_then(Value::as_object_mut)
                .and_then(|m| m.get_mut(id))
            {
                let cap = cap
                    .as_object_mut()
                    .ok_or_else(|| error("model_source_invalid"))?;
                match values.get("inputModalities") {
                    Some(value) => {
                        cap.insert("inputModalities".into(), value.clone());
                    }
                    None => {
                        cap.remove("inputModalities");
                    }
                }
            }
            if values.get("noReasoning") == Some(&Value::Bool(true)) {
                no_reasoning.push(id.clone());
            }
            for (field, key) in [
                (
                    "reasoningAuthoritative",
                    "modelReasoningEffortsAuthoritative",
                ),
                ("fastWire", "fastWire"),
            ] {
                match values.get(field) {
                    Some(value) => {
                        row.insert(key.into(), value.clone());
                    }
                    None => {
                        row.remove(key);
                    }
                }
            }
            if values.get("fast") == Some(&Value::Bool(true))
                && row.get("fastEnabled") == Some(&Value::Bool(false))
            {
                row.insert("fastEnabled".into(), json!(true));
            }
            if let Some(naming) = channel.naming.get(id) {
                for (map, value) in [
                    ("modelAliases", naming.alias.as_ref()),
                    ("modelDisplayNames", naming.display_name.as_ref()),
                ] {
                    set_exact(&mut row, map, id, value.map(|v| json!(v)).as_ref())?;
                }
            }
            destinations.insert(
                (channel.id.clone(), id.clone()),
                mapping.provider_id.clone(),
            );
        }
        row.insert("noReasoningModels".into(), json!(no_reasoning));
        next["providers"][&mapping.provider_id] = Value::Object(row);
        active_by_channel
            .entry(channel.id.clone())
            .or_default()
            .push((mapping.provider_id.clone(), ids));
    }
    for mapping in &mappings.mappings {
        if plan.groups.iter().any(|g| {
            g.channel_id == mapping.channel_id
                && g.protocol == mapping.protocol
                && g.wire_profile_digest == mapping.wire_profile_digest
        }) || mapping.anchor
        {
            continue;
        }
        if let Some(row) = next["providers"].get_mut(&mapping.provider_id) {
            if row["note"] != super::protocol_groups::mapping_marker_v2(mapping) {
                return Err(error("model_runtime_mapping_conflict"));
            }
            let row = row
                .as_object_mut()
                .ok_or_else(|| error("model_source_invalid"))?;
            row.insert("disabled".into(), json!(true));
            for key in [
                "models",
                "retainModels",
                "selectedModels",
                "noReasoningModels",
            ] {
                row.insert(key.into(), json!([]));
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
                "modelAliases",
                "modelDisplayNames",
                "modelCapabilities",
            ] {
                if row.contains_key(key) {
                    row.insert(key.into(), json!({}));
                }
            }
        }
    }
    // Route existing picker/disabled/default references by explicit group mapping;
    // raw model IDs, including slashes, are never normalized or guessed.
    let mut routes = BTreeMap::new();
    for channel in &intended.channels {
        for model in &channel.models {
            if let Some(target) =
                destinations.get(&(channel.id.clone(), model.raw_model_id.clone()))
            {
                for source in std::iter::once(channel.connection.provider_id.as_str()).chain(
                    mappings
                        .mappings
                        .iter()
                        .filter(|m| m.channel_id == channel.id)
                        .map(|m| m.provider_id.as_str()),
                ) {
                    routes.insert(
                        format!("{source}/{}", model.raw_model_id),
                        format!("{target}/{}", model.raw_model_id),
                    );
                }
            }
        }
    }
    for key in ["modelPickerOrder", "disabledModels"] {
        if let Some(value) = next.get(key) {
            let values = strings(Some(value))?;
            let mut seen = BTreeSet::new();
            let values = values
                .into_iter()
                .map(|s| routes.get(&s).cloned().unwrap_or(s))
                .filter(|s| seen.insert(s.clone()))
                .collect::<Vec<_>>();
            next[key] = json!(values);
        }
    }
    if let Some(value) = next.get("defaultModel").and_then(Value::as_str) {
        if let Some(target) = routes.get(value) {
            next["defaultModel"] = json!(target);
        }
    }
    if let Some(id) = &intended.default_channel_id {
        if let Some(groups) = active_by_channel.get(id) {
            let anchor = mappings
                .mappings
                .iter()
                .find(|m| {
                    m.channel_id == *id
                        && m.anchor
                        && groups.iter().any(|(p, _)| p == &m.provider_id)
                })
                .map(|m| &m.provider_id)
                .ok_or_else(|| error("model_runtime_mapping_conflict"))?;
            next["defaultProvider"] = json!(anchor);
        }
    }
    let mut targets = vec![];
    for model in &plan.effective {
        let provider = destinations
            .get(&(model.channel_id.clone(), model.raw_model_id.clone()))
            .cloned()
            .or_else(|| {
                intended
                    .channels
                    .iter()
                    .find(|c| c.id == model.channel_id)
                    .map(|c| c.connection.provider_id.clone())
            })
            .ok_or_else(|| error("model_runtime_mapping_conflict"))?;
        let mut model_targets = targets_for(model, &provider);
        if next["providers"][&provider]["modelCapabilities"]
            .get(&model.raw_model_id)
            .and_then(|v| v.get("inputModalities"))
            .is_some()
        {
            if let Some(target) = model_targets
                .iter_mut()
                .find(|t| t.field == "inputModalities")
            {
                target.pointer = format!(
                    "/providers/{}/modelCapabilities/{}/inputModalities",
                    escaped(&provider),
                    escaped(&model.raw_model_id)
                );
            }
        }
        targets.extend(model_targets);
    }
    let ledger = finalize_materialization_ledger_v2(intended, &next, &targets)?;
    Ok(CompiledFileV2 {
        next_config: next,
        policy: intended.clone(),
        ledger,
        mappings: verified_mappings,
        targets,
        plan,
    })
}
