//! Pure observation, draft CAS plans, explicit edits and runtime materialization.
use super::matching_v2::*;
use super::schema::ModelRules;
use super::schema_v2::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub fn source_rules_v2(source: &SourceModelV2) -> Result<ModelRules> {
    if source.raw_model_id.is_empty() || source.raw_model_id.len() > 2048 {
        return Err(error("model_limit"));
    }
    for field in source.fields.values() {
        if !valid_digest(&field.evidence.source_digest)
            || field.evidence.provider_id.is_empty()
            || !field.evidence.pointer.starts_with("/providers/")
            || field.evidence.pointer.len() > 8192
        {
            return Err(error("model_source_invalid"));
        }
    }
    let values = source
        .fields
        .iter()
        .map(|(k, f)| (k.clone(), f.value.clone()))
        .collect();
    let rules = rules_from_values(&values)?;
    validate_rules(&rules, true)?;
    Ok(rules)
}
fn pointer(s: &str) -> String {
    s.replace('~', "~0").replace('/', "~1")
}
/// Only known public rule values leave this function. The original object stays with L0.
pub fn source_from_provider_v2(
    provider_id: &str,
    raw_model_id: &str,
    provider: &Value,
    source_digest: &str,
    kind: SourceKindV2,
) -> Result<SourceModelV2> {
    if !provider.is_object()
        || !valid_digest(source_digest)
        || raw_model_id.is_empty()
        || raw_model_id.len() > 2048
    {
        return Err(error("model_source_invalid"));
    }
    let mappings = [
        ("protocol", Some("adapter"), Some("modelAdapters")),
        (
            "contextWindow",
            Some("contextWindow"),
            Some("modelContextWindows"),
        ),
        ("inputModalities", None, Some("modelInputModalities")),
        (
            "reasoningEfforts",
            Some("reasoningEfforts"),
            Some("modelReasoningEfforts"),
        ),
        ("defaultEffort", None, Some("modelDefaultReasoningEfforts")),
        (
            "fast",
            Some("supportsServiceTier"),
            Some("modelSupportsServiceTier"),
        ),
        (
            "pinnedEffort",
            Some("pinnedReasoningEffort"),
            Some("modelPinnedReasoningEfforts"),
        ),
        (
            "effortMap",
            Some("reasoningEffortMap"),
            Some("modelReasoningEffortMap"),
        ),
        (
            "reasoningAuthoritative",
            Some("modelReasoningEffortsAuthoritative"),
            None,
        ),
        ("fastWire", Some("fastWire"), None),
    ];
    let root = format!("/providers/{}", pointer(provider_id));
    let mut fields = BTreeMap::new();
    for (key, parent, map_key) in mappings {
        let mut selected =
            parent.and_then(|p| provider.get(p).map(|v| (format!("{root}/{p}"), v.clone())));
        if let Some(map_key) = map_key {
            if let Some(map) = provider.get(map_key) {
                let map = map
                    .as_object()
                    .ok_or_else(|| error("model_source_invalid"))?;
                let found = map
                    .get_key_value(raw_model_id)
                    .or_else(|| {
                        if key == "protocol" {
                            return None;
                        }
                        raw_model_id
                            .split_once(':')
                            .and_then(|(family, _)| map.get_key_value(family))
                    })
                    .or_else(|| {
                        if key == "protocol" {
                            None
                        } else {
                            map.iter()
                                .find(|(k, _)| k.to_lowercase() == raw_model_id.to_lowercase())
                        }
                    });
                if let Some((id, value)) = found {
                    selected = Some((format!("{root}/{map_key}/{}", pointer(id)), value.clone()));
                }
            }
        }
        if key == "fast" && provider.get("fastEnabled") == Some(&Value::Bool(false)) {
            selected = Some((format!("{root}/fastEnabled"), Value::Bool(false)));
        }
        if let Some((path, value)) = selected {
            fields.insert(
                key.into(),
                SourceFieldV2 {
                    value,
                    evidence: SourceEvidenceV2 {
                        kind: kind.clone(),
                        source_digest: source_digest.into(),
                        provider_id: provider_id.into(),
                        pointer: path,
                    },
                },
            );
        }
    }
    if let Some(values) = provider.get("noReasoningModels") {
        let values = values
            .as_array()
            .ok_or_else(|| error("model_source_invalid"))?;
        if values.iter().any(|v| !v.is_string()) {
            return Err(error("model_source_invalid"));
        }
        let index = values.iter().position(|v| v.as_str() == Some(raw_model_id));
        fields.insert(
            "noReasoning".into(),
            SourceFieldV2 {
                value: json!(index.is_some()),
                evidence: SourceEvidenceV2 {
                    kind,
                    source_digest: source_digest.into(),
                    provider_id: provider_id.into(),
                    pointer: format!("{root}/noReasoningModels"),
                },
            },
        );
    }
    let source = SourceModelV2 {
        raw_model_id: raw_model_id.into(),
        fields,
    };
    source_rules_v2(&source)?;
    Ok(source)
}
pub fn validate_policy_v2(policy: &PolicyV2) -> Result<()> {
    if policy.schema_version != SCHEMA_VERSION {
        return Err(error("model_schema_unsupported"));
    }
    let mut ids = BTreeSet::new();
    let mut providers = BTreeSet::new();
    for channel in &policy.channels {
        check_id(&channel.id)?;
        if !ids.insert(&channel.id)
            || channel.connection.provider_id.is_empty()
            || !providers.insert(&channel.connection.provider_id)
        {
            return Err(error("model_source_invalid"));
        }
        validate_non_protocol(&channel.defaults)?;
        for rules in channel.family_overrides.values() {
            validate_non_protocol(rules)?;
        }
        let mut models = BTreeSet::new();
        for model in &channel.models {
            if !models.insert(&model.raw_model_id)
                || model.source.raw_model_id != model.raw_model_id
            {
                return Err(error("model_source_invalid"));
            }
            source_rules_v2(&model.source)?;
            validate_rules(&model.explicit_overrides, false)?;
        }
        if channel.model_order.iter().collect::<BTreeSet<_>>().len() != channel.model_order.len() {
            return Err(error("model_source_invalid"));
        }
        if let Some(ns) = &channel.namespace {
            matching_view_v2(channel, "validation-id")?;
            if ns.channel_id != channel.id {
                return Err(error("model_source_invalid"));
            }
        }
    }
    if policy
        .default_channel_id
        .as_ref()
        .is_some_and(|id| !ids.contains(id))
    {
        return Err(error("model_dependency_missing"));
    }
    for refs in std::iter::once(&policy.template_refs)
        .chain(policy.channels.iter().map(|c| &c.template_refs))
    {
        let mut ids = BTreeSet::new();
        for r in refs {
            check_id(&r.template_id)?;
            check_id(&r.revision_id)?;
            if !valid_digest(&r.digest) || !ids.insert(&r.template_id) {
                return Err(error("model_dependency_missing"));
            }
        }
    }
    Ok(())
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReadBaseV2 {
    pub dto_version: u32,
    pub draft_id: String,
    pub draft_generation: u64,
    pub source_digest: String,
    pub policy_digest: String,
    pub library_digest: String,
    pub root_revision: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReadChannelObservationV2 {
    pub read_id: String,
    pub base: ReadBaseV2,
    pub channel_id: String,
    pub models: Vec<SourceModelV2>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MatchOutcomeV2 {
    Matched,
    ManualPreserved,
    Unmatched,
    MissingDefault,
    MissingDependency,
    Conflict,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReadModelDtoV2 {
    pub raw_model_id: String,
    pub outcome: MatchOutcomeV2,
    pub changed: bool,
    pub binding_before: Option<ModelBindingV2>,
    pub binding_after: Option<ModelBindingV2>,
    pub effective: EffectiveModelV2,
    pub issues: Vec<MatchIssueV2>,
}
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MatchCountsV2 {
    pub total: u32,
    pub matched: u32,
    pub manual_preserved: u32,
    pub unmatched: u32,
    pub missing_default: u32,
    pub missing_dependency: u32,
    pub conflict: u32,
    pub changed: u32,
    pub protocol_pending: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReadMatchDtoV2 {
    pub dto_version: u32,
    pub read_id: String,
    pub base: ReadBaseV2,
    pub next_draft_generation: u64,
    pub changed: bool,
    pub models: Vec<ReadModelDtoV2>,
    pub counts: MatchCountsV2,
}
#[derive(Debug, Clone)]
pub struct ReadMatchPlanV2 {
    pub expected: ReadBaseV2,
    pub next_policy: PolicyV2,
    pub dto: ReadMatchDtoV2,
}
fn model_semantics(model: &ModelConfigV2) -> Result<Vec<u8>> {
    canonical_bytes(&(
        model.raw_model_id.clone(),
        model.label.clone(),
        model.observed,
        &model.binding,
        &model.explicit_overrides,
        model
            .source
            .fields
            .iter()
            .map(|(k, v)| (k, &v.value))
            .collect::<BTreeMap<_, _>>(),
    ))
}
pub fn policy_semantic_digest_v2(policy: &PolicyV2) -> Result<String> {
    let mut value = serde_json::to_value(policy).map_err(json_error)?;
    for channel in value["channels"]
        .as_array_mut()
        .ok_or_else(|| error("model_source_invalid"))?
    {
        for model in channel["models"]
            .as_array_mut()
            .ok_or_else(|| error("model_source_invalid"))?
        {
            for field in model["source"]["fields"]
                .as_object_mut()
                .ok_or_else(|| error("model_source_invalid"))?
                .values_mut()
            {
                field.as_object_mut().unwrap().remove("evidence");
            }
        }
    }
    Ok(digest(&canonical_bytes(&value)?))
}
pub fn match_read_to_draft_v2(
    library: &LibraryV2,
    draft: &PolicyV2,
    observation: &ReadChannelObservationV2,
) -> Result<ReadMatchPlanV2> {
    validate_library_v2(library)?;
    validate_policy_v2(draft)?;
    check_id(&observation.read_id)?;
    check_id(&observation.base.draft_id)?;
    if observation.base.dto_version != DTO_VERSION
        || observation.base.library_digest != digest(&canonical_bytes(library)?)
    {
        return Err(error("model_read_stale"));
    }
    if observation.models.is_empty() {
        return Err(error("model_read_empty"));
    }
    let channel = draft
        .channels
        .iter()
        .find(|c| c.id == observation.channel_id)
        .ok_or_else(|| error("model_source_invalid"))?;
    let mut next = channel.clone();
    let mut counts = MatchCountsV2::default();
    let mut results = vec![];
    let mut seen = BTreeSet::new();
    for source in &observation.models {
        if !seen.insert(&source.raw_model_id) {
            return Err(error("model_source_invalid"));
        }
        source_rules_v2(source)?;
        let original = channel
            .models
            .iter()
            .find(|m| m.raw_model_id == source.raw_model_id)
            .cloned()
            .unwrap_or_else(|| ModelConfigV2 {
                raw_model_id: source.raw_model_id.clone(),
                label: source.raw_model_id.clone(),
                observed: true,
                source: source.clone(),
                explicit_overrides: ModelRules::default(),
                binding: None,
            });
        let mut candidate = original.clone();
        candidate.source = source.clone();
        let manual = matches!(original.binding, Some(ModelBindingV2::Manual { .. }));
        let mut outcome = if manual {
            MatchOutcomeV2::ManualPreserved
        } else {
            MatchOutcomeV2::Matched
        };
        let selected = if manual {
            Ok(original.binding.as_ref().map(|b| b.reference().clone()))
        } else {
            select_reference_v2(library, draft, channel, &source.raw_model_id)
        };
        let resolved = selected.and_then(|reference| {
            if !manual {
                candidate.binding =
                    reference.map(|reference| ModelBindingV2::AutoRead { reference });
            }
            if candidate.binding.is_none() {
                outcome = MatchOutcomeV2::Unmatched;
                let view = matching_view_v2(channel, &source.raw_model_id)?;
                // Diagnosis only; never adopt an arbitrary non-default saved revision.
                if library.templates.iter().any(|t| {
                    t.default_revision.is_none()
                        && t.revisions
                            .iter()
                            .any(|r| family_matches_v2(&r.content, &channel.id, &view))
                }) {
                    outcome = MatchOutcomeV2::MissingDefault;
                }
            }
            resolve_effective_model_v2(library, channel, &candidate)
        });
        let mut issues = vec![];
        let effective = match resolved {
            Ok(effective) if manual || candidate.binding.is_some() => effective,
            Ok(_) => {
                candidate = original.clone();
                if outcome == MatchOutcomeV2::MissingDefault {
                    issues = error("model_template_default_missing").issues;
                }
                resolve_effective_model_v2(library, channel, &original)?
            }
            Err(err) => {
                outcome = if ["model_matching_conflict", "model_condition_overlap"]
                    .contains(&err.code.as_str())
                {
                    MatchOutcomeV2::Conflict
                } else {
                    MatchOutcomeV2::MissingDependency
                };
                candidate = original.clone();
                issues = at(err, &channel.id, &source.raw_model_id).issues;
                match resolve_effective_model_v2(library, channel, &original) {
                    Ok(v) => v,
                    Err(_) => {
                        // Preserve invalid pinned intent; this fallback is display-only and carries the error.
                        let mut unbound = original.clone();
                        unbound.binding = None;
                        let mut v = resolve_effective_model_v2(library, channel, &unbound)?;
                        v.binding = original.binding.clone();
                        v.notices = issues.clone();
                        v
                    }
                }
            }
        };
        let changed = model_semantics(&candidate)? != model_semantics(&original)?;
        match outcome {
            MatchOutcomeV2::Matched => counts.matched += 1,
            MatchOutcomeV2::ManualPreserved => counts.manual_preserved += 1,
            MatchOutcomeV2::Unmatched => counts.unmatched += 1,
            MatchOutcomeV2::MissingDefault => counts.missing_default += 1,
            MatchOutcomeV2::MissingDependency => counts.missing_dependency += 1,
            MatchOutcomeV2::Conflict => counts.conflict += 1,
        }
        counts.total += 1;
        counts.changed += u32::from(changed);
        counts.protocol_pending += u32::from(!matches!(
            effective.protocol,
            ProtocolStateV2::Resolved { .. }
        ));
        if let Some(slot) = next
            .models
            .iter_mut()
            .find(|m| m.raw_model_id == candidate.raw_model_id)
        {
            *slot = candidate.clone();
        } else {
            next.models.push(candidate.clone());
        }
        results.push(ReadModelDtoV2 {
            raw_model_id: source.raw_model_id.clone(),
            outcome,
            changed,
            binding_before: original.binding.clone(),
            binding_after: candidate.binding.clone(),
            effective,
            issues,
        });
    }
    let mut next_policy = draft.clone();
    *next_policy
        .channels
        .iter_mut()
        .find(|c| c.id == channel.id)
        .unwrap() = next;
    let changed = policy_semantic_digest_v2(&next_policy)? != policy_semantic_digest_v2(draft)?;
    let next_generation = observation
        .base
        .draft_generation
        .checked_add(u64::from(changed))
        .ok_or_else(|| error("model_limit"))?;
    Ok(ReadMatchPlanV2 {
        expected: observation.base.clone(),
        next_policy,
        dto: ReadMatchDtoV2 {
            dto_version: DTO_VERSION,
            read_id: observation.read_id.clone(),
            base: observation.base.clone(),
            next_draft_generation: next_generation,
            changed,
            models: results,
            counts,
        },
    })
}
pub fn apply_read_plan_v2(
    plan: ReadMatchPlanV2,
    current: &ReadBaseV2,
    latest_read_id: &str,
) -> Result<PolicyV2> {
    if &plan.expected != current || plan.dto.read_id != latest_read_id {
        return Err(error("model_read_stale"));
    }
    Ok(plan.next_policy)
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ModelEditV2 {
    SetOverride {
        channel_id: String,
        raw_model_id: String,
        field: String,
        value: Value,
    },
    ClearOverride {
        channel_id: String,
        raw_model_id: String,
        field: String,
    },
    SetManualBinding {
        channel_id: String,
        raw_model_id: String,
        reference: RefV2,
    },
    ClearManualBinding {
        channel_id: String,
        raw_model_id: String,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveModelsRequestV2 {
    pub dto_version: u32,
    pub expected: ReadBaseV2,
    pub edits: Vec<ModelEditV2>,
    pub scope_channel_id: Option<String>,
}
pub fn apply_explicit_edits_v2(draft: &PolicyV2, edits: &[ModelEditV2]) -> Result<PolicyV2> {
    let mut next = draft.clone();
    for edit in edits {
        let (channel_id, raw_model_id) = match edit {
            ModelEditV2::SetOverride {
                channel_id,
                raw_model_id,
                ..
            }
            | ModelEditV2::ClearOverride {
                channel_id,
                raw_model_id,
                ..
            }
            | ModelEditV2::SetManualBinding {
                channel_id,
                raw_model_id,
                ..
            }
            | ModelEditV2::ClearManualBinding {
                channel_id,
                raw_model_id,
            } => (channel_id, raw_model_id),
        };
        let model = next
            .channels
            .iter_mut()
            .find(|c| &c.id == channel_id)
            .and_then(|c| {
                c.models
                    .iter_mut()
                    .find(|m| &m.raw_model_id == raw_model_id)
            })
            .ok_or_else(|| error("model_source_invalid"))?;
        match edit {
            ModelEditV2::SetOverride { field, value, .. } => {
                let mut values = rules_values(&model.explicit_overrides)?;
                values.insert(field.clone(), value.clone());
                model.explicit_overrides = rules_from_values(&values)?;
                validate_rules(&model.explicit_overrides, false)?;
            }
            ModelEditV2::ClearOverride { field, .. } => {
                if !RULE_KEYS.contains(&field.as_str()) {
                    return Err(error("model_unknown_field"));
                }
                let mut values = rules_values(&model.explicit_overrides)?;
                values.remove(field);
                model.explicit_overrides = rules_from_values(&values)?;
            }
            ModelEditV2::SetManualBinding { reference, .. } => {
                model.binding = Some(ModelBindingV2::Manual {
                    reference: reference.clone(),
                })
            }
            ModelEditV2::ClearManualBinding { .. } => model.binding = None,
        }
    }
    validate_policy_v2(&next)?;
    Ok(next)
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MaterializedFieldV2 {
    pub channel_id: String,
    pub raw_model_id: String,
    pub field: String,
    pub target_pointer: String,
    pub value_digest: String,
    pub present: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MaterializationLedgerV2 {
    pub source_policy_digest: String,
    pub written_file_digest: String,
    pub fields: Vec<MaterializedFieldV2>,
}
/// Reconcile actual materialized pointers, not resolved provider fallbacks. No mutation.
pub fn reconcile_source_v2(
    channel_id: &str,
    saved: &SourceModelV2,
    imported: &SourceModelV2,
    ledger: Option<&MaterializationLedgerV2>,
    actual_document: &Value,
) -> Result<SourceModelV2> {
    let ledger = ledger.ok_or_else(|| error("model_source_provenance_unknown"))?;
    if saved.raw_model_id != imported.raw_model_id
        || !valid_digest(&ledger.source_policy_digest)
        || !valid_digest(&ledger.written_file_digest)
    {
        return Err(error("model_source_provenance_unknown"));
    }
    let mut fields = imported.fields.clone();
    let mut seen = BTreeSet::new();
    for entry in ledger
        .fields
        .iter()
        .filter(|e| e.channel_id == channel_id && e.raw_model_id == saved.raw_model_id)
    {
        if !RULE_KEYS.contains(&entry.field.as_str())
            || !seen.insert(&entry.field)
            || !entry.target_pointer.starts_with("/providers/")
            || !valid_digest(&entry.value_digest)
        {
            return Err(error("model_source_provenance_unknown"));
        }
        let actual = actual_document.pointer(&entry.target_pointer);
        let unchanged = if entry.present {
            actual
                .is_some_and(|v| canonical_bytes(v).is_ok_and(|b| digest(&b) == entry.value_digest))
        } else {
            actual.is_none()
        };
        if unchanged {
            if let Some(old) = saved.fields.get(&entry.field) {
                fields.insert(entry.field.clone(), old.clone());
            } else {
                fields.remove(&entry.field);
            }
        } else if let Some(field) = fields.get_mut(&entry.field) {
            field.evidence.kind = SourceKindV2::ExternalFileEdit;
        }
    }
    // A managed source field without a materialization record is not safe to reclassify.
    if saved.fields.keys().any(|k| !seen.contains(k)) {
        return Err(error("model_source_provenance_unknown"));
    }
    let source = SourceModelV2 {
        raw_model_id: saved.raw_model_id.clone(),
        fields,
    };
    source_rules_v2(&source)?;
    Ok(source)
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProtocolGroupPlanV2 {
    pub channel_id: String,
    pub protocol: ProtocolV2,
    pub wire_profile_digest: String,
    pub raw_model_ids: Vec<String>,
}
pub fn plan_protocol_groups_v2(rows: &[EffectiveModelV2]) -> Result<Vec<ProtocolGroupPlanV2>> {
    let mut groups: BTreeMap<(String, ProtocolV2, String), Vec<String>> = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for row in rows {
        if !seen.insert((&row.channel_id, &row.raw_model_id)) {
            return Err(error("model_runtime_mapping_conflict"));
        }
        let protocol = match &row.protocol {
            ProtocolStateV2::Resolved { value, .. } => value.clone(),
            _ if !row.selected || !row.enabled => continue,
            ProtocolStateV2::Missing => {
                return Err(at(
                    error("model_protocol_missing"),
                    &row.channel_id,
                    &row.raw_model_id,
                ))
            }
            _ => {
                return Err(at(
                    error("model_protocol_unsupported"),
                    &row.channel_id,
                    &row.raw_model_id,
                ))
            }
        };
        let signature = digest(&canonical_bytes(
            &json!({"reasoningAuthoritative":row.rules.reasoning_authoritative,"fastWire":row.rules.fast_wire}),
        )?);
        groups
            .entry((row.channel_id.clone(), protocol, signature))
            .or_default()
            .push(row.raw_model_id.clone());
    }
    Ok(groups
        .into_iter()
        .map(
            |((channel_id, protocol, wire_profile_digest), mut raw_model_ids)| {
                raw_model_ids.sort();
                ProtocolGroupPlanV2 {
                    channel_id,
                    protocol,
                    wire_profile_digest,
                    raw_model_ids,
                }
            },
        )
        .collect())
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelFieldWriteV2 {
    pub channel_id: String,
    pub raw_model_id: String,
    pub field: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelMaterializationPlanV2 {
    pub expected_intent_digest: String,
    pub effective: Vec<EffectiveModelV2>,
    pub writes: Vec<ModelFieldWriteV2>,
    pub groups: Vec<ProtocolGroupPlanV2>,
    pub dependencies: Vec<RefV2>,
}
pub fn plan_materialization_v2(
    library: &LibraryV2,
    before: &PolicyV2,
    intended: &PolicyV2,
) -> Result<ModelMaterializationPlanV2> {
    validate_library_v2(library)?;
    validate_policy_v2(before)?;
    validate_policy_v2(intended)?;
    let mut effective = vec![];
    let mut writes = vec![];
    let mut dependencies = BTreeSet::new();
    for channel in &intended.channels {
        for model in &channel.models {
            let after = resolve_effective_model_v2(library, channel, model)?;
            let previous = before
                .channels
                .iter()
                .find(|c| c.id == channel.id)
                .and_then(|c| {
                    c.models
                        .iter()
                        .find(|m| m.raw_model_id == model.raw_model_id)
                        .map(|m| (c, m))
                });
            let prior = previous
                .map(|(c, m)| resolve_effective_model_v2(library, c, m))
                .transpose()?;
            let prior = prior
                .as_ref()
                .map(|v| rules_values(&v.rules))
                .transpose()?
                .unwrap_or_default();
            let next = rules_values(&after.rules)?;
            for key in RULE_KEYS {
                if prior.get(key) != next.get(key) {
                    writes.push(ModelFieldWriteV2 {
                        channel_id: channel.id.clone(),
                        raw_model_id: model.raw_model_id.clone(),
                        field: key.into(),
                        before: prior.get(key).cloned(),
                        after: next.get(key).cloned(),
                    });
                }
            }
            if let Some(binding) = &model.binding {
                dependencies.insert(binding.reference().clone());
            }
            effective.push(after);
        }
    }
    let groups = plan_protocol_groups_v2(&effective)?;
    Ok(ModelMaterializationPlanV2 {
        expected_intent_digest: policy_semantic_digest_v2(intended)?,
        effective,
        writes,
        groups,
        dependencies: dependencies.into_iter().collect(),
    })
}
