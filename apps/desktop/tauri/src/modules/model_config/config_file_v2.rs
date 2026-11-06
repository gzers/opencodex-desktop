//! Authoritative v2 draft/config boundary. Raw providers never cross IPC.
#[path = "model_receive_file_v2.rs"]
pub mod receive;
#[cfg(test)]
#[path = "config_file_v2_tests.rs"]
mod tests;
use crate::modules::model_config::{
    compiler_v2, matching_v2,
    protocol_groups::{self, MappingFileV2, RuntimeMappingV2},
    provider_import::{self, ProviderBindingV2},
    schema::{ModelRules, ModelSelection, ServiceNode},
    schema_v2::*,
    source_v2::*,
    store_v2::{self, DraftCredentialEditV2, DraftV2, StoreSnapshotV2, StoreV2},
};
use crate::{infrastructure::locking::TargetFileLock, modules::backup::safety};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use zeroize::Zeroizing;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSnapshotV2 {
    pub dto_version: u32,
    pub library: LibraryV2,
    pub library_digest: String,
    pub saved_policy: PolicyV2,
    pub draft: Option<DraftV2>,
    pub source_digest: String,
    pub root_revision: String,
    pub effective: Vec<EffectiveModelV2>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReadRequestV2 {
    pub expected_source: String,
    pub expected_library: String,
    pub expected_draft: Option<ReadBaseV2>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadResponseV2 {
    pub snapshot: FileSnapshotV2,
    pub results: Vec<ReadMatchDtoV2>,
}
#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ChannelIntentV2 {
    SetActiveNode {
        channel_id: String,
        node_id: String,
    },
    SetDefaults {
        channel_id: String,
        rules: ModelRules,
    },
    SetFamilyDefaults {
        channel_id: String,
        template_id: String,
        rules: ModelRules,
    },
    SetNaming {
        channel_id: String,
        raw_model_id: String,
        alias: Option<String>,
        display_name: Option<String>,
    },
    SetTemplateRefs {
        channel_id: String,
        references: Vec<RefV2>,
    },
    Delete {
        channel_id: String,
    },
    Rename {
        channel_id: String,
        name: String,
    },
    SetSelection {
        channel_id: String,
        selection: ModelSelection,
    },
    SetEnabled {
        channel_id: String,
        enabled: bool,
    },
    SetOrder {
        channel_id: String,
        raw_model_ids: Vec<String>,
    },
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveRequestV2 {
    pub expected: ReadBaseV2,
    pub edits: Vec<ModelEditV2>,
    pub channel_intents: Vec<ChannelIntentV2>,
    #[serde(default)]
    pub scope_channel_id: Option<String>,
}
fn io<T>(_: T) -> MatchErrorV2 {
    error("model_storage")
}
fn root_revision(root: &Path, home: &Path) -> Result<String> {
    let root = root.canonicalize().map_err(io)?;
    let home = home.canonicalize().map_err(io)?;
    Ok(digest(&canonical_bytes(&(root, home))?))
}
fn read_raw(home: &Path) -> Result<(Value, Option<Zeroizing<Vec<u8>>>)> {
    let path = home.join("config.json");
    safety::check_path(home, &path, true).map_err(io)?;
    let bytes = match std::fs::symlink_metadata(&path) {
        Ok(_) => Some(Zeroizing::new(
            safety::read_file(&path, 32 * 1024 * 1024).map_err(io)?,
        )),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(io(e)),
    };
    let raw = bytes
        .as_ref()
        .map(|b| parse_strict_json_v2(b))
        .transpose()?
        .unwrap_or_else(|| json!({}));
    if !raw.is_object() {
        return Err(error("model_source_invalid"));
    }
    Ok((raw, bytes))
}
fn source_digest(bytes: &Option<Zeroizing<Vec<u8>>>) -> String {
    digest(bytes.as_deref().map(|v| v.as_slice()).unwrap_or_default())
}
fn bindings(
    root: &Path,
    raw: &Value,
    policy: &PolicyV2,
    draft: Option<&DraftV2>,
    mappings: &MappingFileV2,
) -> Result<Vec<ProviderBindingV2>> {
    let providers = raw
        .get("providers")
        .map(|v| v.as_object().ok_or_else(|| error("model_source_invalid")))
        .transpose()?;
    let mut rows = policy
        .channels
        .iter()
        .map(|c| ProviderBindingV2 {
            channel_id: c.id.clone(),
            provider_id: c.connection.provider_id.clone(),
        })
        .collect::<Vec<_>>();
    let registry_path = root.join("manager-state/model-v2-provider-identities.json");
    safety::check_path(root, &registry_path, true).map_err(io)?;
    let before_registry = if registry_path.exists() {
        Some(safety::read_file(&registry_path, 1024 * 1024).map_err(io)?)
    } else {
        None
    };
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Registry {
        schema_version: u32,
        providers: BTreeMap<String, String>,
    }
    let mut registry = before_registry
        .as_ref()
        .map(|b| serde_json::from_value::<Registry>(parse_strict_json_v2(b)?).map_err(json_error))
        .transpose()?
        .unwrap_or(Registry {
            schema_version: 2,
            providers: BTreeMap::new(),
        });
    if registry.schema_version != 2 || registry.providers.len() > 10000 {
        return Err(error("model_source_invalid"));
    }
    registry.providers.retain(|key, _| {
        raw["providers"].get(key).is_some()
            && !mappings
                .mappings
                .iter()
                .any(|m| m.provider_id == *key && !m.anchor)
    });
    let mut identities = BTreeSet::new();
    for (key, id) in &registry.providers {
        check_id(id)?;
        if key.is_empty() || key.len() > 512 || !identities.insert(id.clone()) {
            return Err(error("model_source_invalid"));
        }
    }
    for row in &rows {
        registry
            .providers
            .retain(|key, id| id != &row.channel_id || key == &row.provider_id);
        registry
            .providers
            .insert(row.provider_id.clone(), row.channel_id.clone());
    }
    for (id, value) in providers.into_iter().flat_map(|p| p.iter()) {
        if !value.is_object() {
            return Err(error("model_source_invalid"));
        }
        if rows.iter().any(|b| b.provider_id == *id)
            || mappings.mappings.iter().any(|m| m.provider_id == *id)
        {
            continue;
        }
        let known = draft.and_then(|d| {
            d.policy
                .channels
                .iter()
                .find(|c| c.connection.provider_id == *id)
        });
        let channel_id = known
            .map(|c| c.id.clone())
            .or_else(|| registry.providers.get(id).cloned())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        if rows.iter().any(|b| b.channel_id == channel_id) {
            return Err(error("model_source_invalid"));
        }
        registry.providers.insert(id.clone(), channel_id.clone());
        rows.push(ProviderBindingV2 {
            channel_id,
            provider_id: id.clone(),
        });
    }
    let after = canonical_bytes(&registry)?;
    if before_registry.as_deref() != Some(after.as_slice()) {
        crate::infrastructure::atomic_write::atomic_write(&registry_path, &after, 0o600)
            .map_err(io)?;
    }
    Ok(rows)
}
fn effective(library: &LibraryV2, policy: &PolicyV2) -> Result<Vec<EffectiveModelV2>> {
    policy
        .channels
        .iter()
        .flat_map(|c| c.models.iter().map(move |m| (c, m)))
        .map(|(c, m)| matching_v2::resolve_effective_model_v2(library, c, m))
        .collect()
}
fn view(state: StoreSnapshotV2, source: String, revision: String) -> Result<FileSnapshotV2> {
    let policy = state
        .draft
        .as_ref()
        .map(|d| &d.policy)
        .unwrap_or(&state.policy);
    let effective = effective(&state.library, policy)?;
    Ok(FileSnapshotV2 {
        dto_version: 2,
        library: state.library,
        library_digest: state.digests.library,
        saved_policy: state.policy,
        draft: state.draft,
        source_digest: source,
        root_revision: revision,
        effective,
    })
}
pub fn snapshot(root: &Path, home: &Path) -> Result<FileSnapshotV2> {
    let _file_admission =
        crate::modules::model_config::content_versions::file_saves::admit(root, home)
            .map_err(io)?;
    let _rename = super::channel_rename::admit(root, home).map_err(io)?;
    let _config = TargetFileLock::lock(&home.join("config.json")).map_err(io)?;
    let store = StoreV2::new(root);
    let locked = store.lock()?;
    {
        let _recovery_versions =
            crate::modules::model_config::content_versions::lock_versions(root).map_err(io)?;
        recover_model_transactions(root, home)?;
    }
    let (_, bytes) = read_raw(home)?;
    view(
        locked.snapshot_unlocked()?,
        source_digest(&bytes),
        root_revision(root, home)?,
    )
}
/// Explicit Read is the only operation that automatically chooses template defaults.
pub fn read_existing(root: &Path, home: &Path, request: ReadRequestV2) -> Result<ReadResponseV2> {
    let _file_admission =
        crate::modules::model_config::content_versions::file_saves::admit(root, home)
            .map_err(io)?;
    let _rename = super::channel_rename::admit(root, home).map_err(io)?;
    let _config = TargetFileLock::lock(&home.join("config.json")).map_err(io)?;
    let store = StoreV2::new(root);
    let locked = store.lock()?;
    {
        let _recovery_versions =
            crate::modules::model_config::content_versions::lock_versions(root).map_err(io)?;
        recover_model_transactions(root, home)?;
    }
    let state = locked.snapshot_unlocked()?;
    let (raw, bytes) = read_raw(home)?;
    let source = source_digest(&bytes);
    let revision = root_revision(root, home)?;
    if source != request.expected_source
        || state.digests.library != request.expected_library
        || state.draft.as_ref().map(|d| &d.base) != request.expected_draft.as_ref()
    {
        return Err(error("model_read_stale"));
    }
    if state
        .draft
        .as_ref()
        .is_some_and(|d| d.policy.recovery_id.is_some())
    {
        return Err(error("model_history_draft_active"));
    }
    let bindings = bindings(
        root,
        &raw,
        &state.policy,
        state.draft.as_ref(),
        &state.mappings,
    )?;
    let observed = provider_import::read_policy_v2(
        &raw,
        &source,
        &state.policy,
        state.ledger.as_ref(),
        &state.mappings,
        &bindings,
    )?;
    if observed.channels.iter().all(|c| c.models.is_empty()) {
        return Err(error("model_read_empty"));
    }
    let old = state.draft.clone();
    let base = ReadBaseV2 {
        dto_version: 2,
        draft_id: old
            .as_ref()
            .map(|d| d.base.draft_id.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        draft_generation: old.as_ref().map(|d| d.base.draft_generation).unwrap_or(0),
        source_digest: source.clone(),
        policy_digest: policy_semantic_digest_v2(&state.policy)?,
        library_digest: state.digests.library.clone(),
        root_revision: revision.clone(),
    };
    // Import current actual source while retaining the server's deliberate edits.
    let observed_baseline = observed.clone();
    let mut next = observed;
    if let Some(d) = &old {
        for channel in &mut next.channels {
            if let Some(prior) = d.policy.channels.iter().find(|c| c.id == channel.id) {
                let baseline = d
                    .observed_baseline
                    .as_ref()
                    .and_then(|b| b.channels.iter().find(|c| c.id == prior.id));
                if baseline.is_none_or(|b| b.label != prior.label) {
                    channel.label = prior.label.clone();
                }
                if baseline.is_none_or(|b| b.connection != prior.connection) {
                    channel.connection = prior.connection.clone();
                }
                if baseline.is_none_or(|b| b.selection != prior.selection) {
                    channel.selection = prior.selection.clone();
                }
                if baseline.is_none_or(|b| b.model_order != prior.model_order) {
                    channel.model_order = prior.model_order.clone();
                }
                if baseline.is_none_or(|b| b.namespace != prior.namespace) {
                    channel.namespace = prior.namespace.clone();
                }
                if baseline.is_none_or(|b| b.naming != prior.naming) {
                    channel.naming = prior.naming.clone();
                }
                for model in &prior.models {
                    if !channel
                        .models
                        .iter()
                        .any(|m| m.raw_model_id == model.raw_model_id)
                    {
                        channel.models.push(model.clone());
                    }
                }
                for model in &channel.models {
                    if !channel.model_order.contains(&model.raw_model_id) {
                        channel.model_order.push(model.raw_model_id.clone());
                    }
                }
                channel.defaults = prior.defaults.clone();
                channel.family_overrides = prior.family_overrides.clone();
                channel.template_refs = prior.template_refs.clone();
                for model in &mut channel.models {
                    if let Some(prior) = prior
                        .models
                        .iter()
                        .find(|m| m.raw_model_id == model.raw_model_id)
                    {
                        model.explicit_overrides = prior.explicit_overrides.clone();
                        model.binding = prior.binding.clone();
                    }
                }
            }
        }
    }
    if let Some(d) = &old {
        for channel in &d.policy.channels {
            if !next.channels.iter().any(|c| c.id == channel.id) {
                next.channels.push(channel.clone());
            }
        }
    }
    let mut results = vec![];
    for channel in next.channels.clone() {
        if channel.models.is_empty() {
            continue;
        }
        let observation = ReadChannelObservationV2 {
            read_id: uuid::Uuid::new_v4().to_string(),
            base: base.clone(),
            channel_id: channel.id.clone(),
            models: channel.models.iter().map(|m| m.source.clone()).collect(),
        };
        let plan = match_read_to_draft_v2(&state.library, &next, &observation)?;
        next = plan.next_policy;
        results.push(plan.dto);
    }
    let semantic_same = old.as_ref().is_some_and(|d| {
        policy_semantic_digest_v2(&d.policy).ok() == policy_semantic_digest_v2(&next).ok()
            && d.base.source_digest == source
            && d.base.library_digest == state.digests.library
            && d.base.root_revision == revision
    });
    if !semantic_same {
        let mut next_base = base;
        if old.is_some() {
            next_base.draft_generation = next_base
                .draft_generation
                .checked_add(1)
                .ok_or_else(|| error("model_limit"))?;
        }
        locked.stage_draft_unlocked(
            old.as_ref().map(|d| &d.base),
            DraftV2 {
                schema_version: 2,
                base: next_base,
                credential_edits: old
                    .as_ref()
                    .map(|d| d.credential_edits.clone())
                    .unwrap_or_default(),
                observed_baseline: Some(observed_baseline),
                policy: next,
            },
        )?;
    }
    let snapshot = view(locked.snapshot_unlocked()?, source, revision)?;
    if let Some(d) = &snapshot.draft {
        for result in &mut results {
            result.next_draft_generation = d.base.draft_generation;
        }
    }
    Ok(ReadResponseV2 { snapshot, results })
}
fn protected(id: &str, raw: &Value) -> bool {
    ["openai", "chatgpt", "openai-multi", "combo", "native"]
        .contains(&id.to_ascii_lowercase().as_str())
        || matches!(raw["authMode"].as_str(), Some("oauth" | "forward"))
}
fn allocate(name: &str, id: &str, occupied: &BTreeSet<String>) -> String {
    fn legal(s: &str) -> bool {
        !s.is_empty()
            && s.len() <= 64
            && s.bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_'))
            && s.as_bytes()[0].is_ascii_alphanumeric()
            && s.as_bytes()[s.len() - 1].is_ascii_alphanumeric()
            && ![
                "native",
                "openai",
                "chatgpt",
                "openai-multi",
                "combo",
                "policy",
                "__proto__",
                "constructor",
                "prototype",
            ]
            .contains(&s.to_ascii_lowercase().as_str())
    }
    if legal(name) && !occupied.contains(&name.to_ascii_lowercase()) {
        return name.into();
    }
    let base = format!("channel-{}", id.replace('-', ""));
    let mut candidate = base.clone();
    let mut i = 0;
    while occupied.contains(&candidate.to_ascii_lowercase()) {
        i += 1;
        candidate = format!("{base}-{i}");
    }
    candidate
}
fn rename(
    raw: &mut Value,
    policy: &mut PolicyV2,
    mappings: &mut MappingFileV2,
    channel_id: &str,
    name: &str,
) -> Result<()> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 80 || name.len() > 256 {
        return Err(error("model_channel_name"));
    }
    if policy
        .channels
        .iter()
        .any(|c| c.id != channel_id && c.label.trim().to_lowercase() == name.to_lowercase())
    {
        return Err(error("model_channel_name_duplicate"));
    }
    let channel = policy
        .channels
        .iter_mut()
        .find(|c| c.id == channel_id)
        .ok_or_else(|| error("model_source_invalid"))?;
    let from = channel.connection.provider_id.clone();
    let provider = raw["providers"]
        .get(&from)
        .ok_or_else(|| error("model_source_invalid"))?;
    if protected(&from, provider) {
        return Err(error("model_protected_provider"));
    }
    let occupied = raw["providers"]
        .as_object()
        .ok_or_else(|| error("model_source_invalid"))?
        .keys()
        .filter(|id| **id != from)
        .map(|id| id.to_lowercase())
        .collect::<BTreeSet<_>>();
    let to = allocate(name, channel_id, &occupied);
    channel.label = name.into();
    channel.connection.provider_id = to.clone();
    let mut row = raw["providers"]
        .as_object_mut()
        .unwrap()
        .remove(&from)
        .unwrap();
    row["name"] = json!(name);
    raw["providers"]
        .as_object_mut()
        .unwrap()
        .insert(to.clone(), row);
    for mapping in &mut mappings.mappings {
        if mapping.provider_id == from {
            mapping.provider_id = to.clone();
        }
    }
    let changes = vec![
        crate::modules::model_config::config_file_mapping::ProviderRename {
            channel_id: channel_id.into(),
            from,
            to,
        },
    ];
    crate::modules::model_config::config_file_mapping::rewrite_provider_references(raw, &changes)
        .map_err(io)?;
    // defaultModel is also a routed slug; raw model IDs (including slash) remain intact.
    if let Some(value) = raw["defaultModel"].as_str() {
        if let Some(rest) = value.strip_prefix(&(changes[0].from.clone() + "/")) {
            raw["defaultModel"] = json!(format!("{}/{}", changes[0].to, rest));
        }
    }
    Ok(())
}
fn allocate_mappings(
    raw: &Value,
    policy: &PolicyV2,
    groups: &[ProtocolGroupPlanV2],
    existing: &MappingFileV2,
) -> Result<MappingFileV2> {
    let mut next = existing.clone();
    let mut occupied = raw["providers"]
        .as_object()
        .map(|p| p.keys().map(|s| s.to_lowercase()).collect::<BTreeSet<_>>())
        .unwrap_or_default();
    for channel in &policy.channels {
        let channel_groups = groups
            .iter()
            .filter(|g| g.channel_id == channel.id)
            .collect::<Vec<_>>();
        if channel_groups.is_empty() {
            continue;
        }
        let anchor_group = channel_groups
            .iter()
            .find(|g| {
                raw["providers"][&channel.connection.provider_id]["adapter"].as_str()
                    == Some(g.protocol.as_str())
            })
            .copied()
            .unwrap_or(channel_groups[0]);
        if !next
            .mappings
            .iter()
            .any(|m| m.channel_id == channel.id && m.anchor)
        {
            next.mappings.push(RuntimeMappingV2 {
                channel_id: channel.id.clone(),
                protocol: anchor_group.protocol.clone(),
                wire_profile_digest: anchor_group.wire_profile_digest.clone(),
                provider_id: channel.connection.provider_id.clone(),
                anchor: true,
            });
        }
        for group in channel_groups {
            if next.mappings.iter().any(|m| {
                m.channel_id == group.channel_id
                    && m.protocol == group.protocol
                    && m.wire_profile_digest == group.wire_profile_digest
            }) {
                continue;
            }
            let base = format!(
                "{}-{}-{}",
                channel.connection.provider_id,
                group.protocol.as_str(),
                &group.wire_profile_digest[7..15]
            );
            let mut id = base.clone();
            let mut i = 0;
            while occupied.contains(&id.to_lowercase()) {
                i += 1;
                id = format!("{base}-{i}");
            }
            occupied.insert(id.to_lowercase());
            next.mappings.push(RuntimeMappingV2 {
                channel_id: channel.id.clone(),
                protocol: group.protocol.clone(),
                wire_profile_digest: group.wire_profile_digest.clone(),
                provider_id: id,
                anchor: false,
            });
        }
    }
    protocol_groups::plan_mappings_v2(&next, groups, &[])
}
fn auth_bindings(raw: &Value, policy: &PolicyV2) -> Result<Vec<ProtocolAuthBindingV2>> {
    policy
        .channels
        .iter()
        .map(|c| {
            let provider = &raw["providers"][&c.connection.provider_id];
            let mode = provider["authMode"].as_str();
            let allowed = if protected(&c.connection.provider_id, provider) {
                let mut protocols = BTreeSet::new();
                if let Some(protocol) = provider["adapter"].as_str().and_then(ProtocolV2::parse) {
                    protocols.insert(protocol);
                }
                protocols
            } else if matches!(mode, None | Some("key" | "local" | "none")) {
                // Authentication is allowed only within this exact provider's captured
                // base URL; compiler never chooses an endpoint from a model name.
                [
                    ProtocolV2::OpenaiChat,
                    ProtocolV2::OpenaiResponses,
                    ProtocolV2::Anthropic,
                    ProtocolV2::Google,
                    ProtocolV2::OllamaNative,
                ]
                .into()
            } else {
                return Err(error("model_runtime_auth_binding"));
            };
            Ok(ProtocolAuthBindingV2 {
                channel_id: c.id.clone(),
                allowed_protocols: allowed,
            })
        })
        .collect()
}
// Shared pure draft transformations. The save path keeps its original guards,
// credential handling and transaction order; read-only children reuse these.
fn validate_save_scope_v2(request: &SaveRequestV2) -> Result<()> {
    if let Some(scope) = &request.scope_channel_id {
        check_id(scope)?;
        for edit in &request.edits {
            let id = match edit {
                ModelEditV2::SetOverride { channel_id, .. }
                | ModelEditV2::ClearOverride { channel_id, .. }
                | ModelEditV2::SetManualBinding { channel_id, .. }
                | ModelEditV2::ClearManualBinding { channel_id, .. } => channel_id,
            };
            if id != scope {
                return Err(error("model_save_scope"));
            }
        }
        for intent in &request.channel_intents {
            let id = match intent {
                ChannelIntentV2::Rename { channel_id, .. }
                | ChannelIntentV2::SetActiveNode { channel_id, .. }
                | ChannelIntentV2::SetSelection { channel_id, .. }
                | ChannelIntentV2::SetEnabled { channel_id, .. }
                | ChannelIntentV2::SetOrder { channel_id, .. }
                | ChannelIntentV2::SetDefaults { channel_id, .. }
                | ChannelIntentV2::SetFamilyDefaults { channel_id, .. }
                | ChannelIntentV2::SetNaming { channel_id, .. }
                | ChannelIntentV2::SetTemplateRefs { channel_id, .. }
                | ChannelIntentV2::Delete { channel_id } => channel_id,
            };
            if id != scope {
                return Err(error("model_save_scope"));
            }
        }
    }
    Ok(())
}
fn transform_draft_v2(
    root: &Path,
    source: &str,
    state: &StoreSnapshotV2,
    draft: &DraftV2,
    raw: &mut Value,
    request: &mut SaveRequestV2,
) -> Result<(PolicyV2, MappingFileV2)> {
    let edited = apply_explicit_edits_v2(&draft.policy, &request.edits)?;
    let mut intended = if let Some(scope) = &request.scope_channel_id {
        let mut p = state.policy.clone();
        let channel = edited
            .channels
            .iter()
            .find(|c| &c.id == scope)
            .cloned()
            .ok_or_else(|| error("model_save_scope"))?;
        if let Some(slot) = p.channels.iter_mut().find(|c| &c.id == scope) {
            *slot = channel;
        } else {
            p.channels.push(channel);
        }
        p
    } else {
        edited
    };
    if raw.get("providers").is_none() {
        raw["providers"] = json!({});
    }
    if !raw["providers"].is_object() {
        return Err(error("model_source_invalid"));
    }
    for channel in &intended.channels {
        if raw["providers"]
            .get(&channel.connection.provider_id)
            .is_none()
        {
            if state.policy.channels.iter().any(|c| c.id == channel.id) {
                return Err(error("model_save_stale"));
            }
            let seed = copied_seed(root, raw, channel, source)?.unwrap_or_else(
                || json!({"name":channel.label,"disabled":true,"models":[],"selectedModels":[]}),
            );
            raw["providers"]
                .as_object_mut()
                .unwrap()
                .insert(channel.connection.provider_id.clone(), seed);
        }
    }
    if intended.recovery_id.is_some() {
        // The trusted recovery guard fixes this exact target. Express removed
        // channels through the existing Delete transform, never raw rewrites.
        let removed = state
            .policy
            .channels
            .iter()
            .filter(|c| !intended.channels.iter().any(|v| v.id == c.id))
            .cloned()
            .collect::<Vec<_>>();
        for channel in removed {
            request.channel_intents.push(ChannelIntentV2::Delete {
                channel_id: channel.id.clone(),
            });
            intended.channels.push(channel);
        }
    }
    let mut mappings = state.mappings.clone();
    // Name is the product authority: even an already-equal historical label/key
    // pair is reconciled on explicit Save, rather than waiting for another edit.
    for intent in std::mem::take(&mut request.channel_intents) {
        match intent {
            ChannelIntentV2::SetActiveNode {
                channel_id,
                node_id,
            } => {
                let channel = intended
                    .channels
                    .iter_mut()
                    .find(|c| c.id == channel_id)
                    .ok_or_else(|| error("model_source_invalid"))?;
                if !channel.connection.nodes.iter().any(|n| n.id == node_id) {
                    return Err(error("model_node_missing"));
                }
                channel.connection.active_node_id = node_id;
            }
            ChannelIntentV2::SetDefaults { channel_id, rules } => {
                validate_non_protocol(&rules)?;
                intended
                    .channels
                    .iter_mut()
                    .find(|c| c.id == channel_id)
                    .ok_or_else(|| error("model_source_invalid"))?
                    .defaults = rules;
            }
            ChannelIntentV2::SetFamilyDefaults {
                channel_id,
                template_id,
                rules,
            } => {
                check_id(&template_id)?;
                validate_non_protocol(&rules)?;
                intended
                    .channels
                    .iter_mut()
                    .find(|c| c.id == channel_id)
                    .ok_or_else(|| error("model_source_invalid"))?
                    .family_overrides
                    .insert(template_id, rules);
            }
            ChannelIntentV2::SetNaming {
                channel_id,
                raw_model_id,
                alias,
                display_name,
            } => {
                let channel = intended
                    .channels
                    .iter_mut()
                    .find(|c| c.id == channel_id)
                    .ok_or_else(|| error("model_source_invalid"))?;
                if !channel
                    .models
                    .iter()
                    .any(|m| m.raw_model_id == raw_model_id)
                    || alias
                        .iter()
                        .chain(display_name.iter())
                        .any(|s| s.len() > 2048 || s.chars().any(char::is_control))
                {
                    return Err(error("model_source_invalid"));
                }
                channel.naming.insert(
                    raw_model_id,
                    crate::modules::model_config::schema::ModelNaming {
                        alias,
                        display_name,
                    },
                );
            }
            ChannelIntentV2::SetTemplateRefs {
                channel_id,
                references,
            } => {
                for r in &references {
                    crate::modules::model_config::schema_v2::revision(&state.library, r)?;
                }
                intended
                    .channels
                    .iter_mut()
                    .find(|c| c.id == channel_id)
                    .ok_or_else(|| error("model_source_invalid"))?
                    .template_refs = references;
            }
            ChannelIntentV2::Delete { channel_id } => {
                let channel = intended
                    .channels
                    .iter()
                    .find(|c| c.id == channel_id)
                    .ok_or_else(|| error("model_source_invalid"))?;
                if protected(
                    &channel.connection.provider_id,
                    &raw["providers"][&channel.connection.provider_id],
                ) {
                    return Err(error("model_protected_provider"));
                }
                for mapping in mappings
                    .mappings
                    .iter()
                    .filter(|m| m.channel_id == channel_id && !m.anchor)
                {
                    if raw["providers"][&mapping.provider_id]["note"]
                        != protocol_groups::mapping_marker_v2(mapping)
                    {
                        return Err(error("model_runtime_mapping_conflict"));
                    }
                }
                let removed = std::iter::once(channel.connection.provider_id.clone())
                    .chain(
                        mappings
                            .mappings
                            .iter()
                            .filter(|m| m.channel_id == channel_id)
                            .map(|m| m.provider_id.clone()),
                    )
                    .collect::<BTreeSet<_>>();
                for id in &removed {
                    raw["providers"].as_object_mut().unwrap().remove(id);
                }
                for field in ["modelPickerOrder", "disabledModels"] {
                    if let Some(rows) = raw.get_mut(field).and_then(Value::as_array_mut) {
                        rows.retain(|v| {
                            v.as_str().is_none_or(|s| {
                                !removed.iter().any(|id| s.starts_with(&(id.clone() + "/")))
                            })
                        });
                    }
                }
                if raw["defaultProvider"]
                    .as_str()
                    .is_some_and(|s| removed.contains(s))
                {
                    raw.as_object_mut().unwrap().remove("defaultProvider");
                }
                if raw["defaultModel"]
                    .as_str()
                    .is_some_and(|s| removed.iter().any(|id| s.starts_with(&(id.clone() + "/"))))
                {
                    raw.as_object_mut().unwrap().remove("defaultModel");
                }
                intended.channels.retain(|c| c.id != channel_id);
                mappings.mappings.retain(|m| m.channel_id != channel_id);
                if intended.default_channel_id.as_deref() == Some(&channel_id) {
                    intended.default_channel_id = None;
                }
            }
            ChannelIntentV2::Rename { channel_id, name } => {
                rename(raw, &mut intended, &mut mappings, &channel_id, &name)?
            }
            ChannelIntentV2::SetSelection {
                channel_id,
                selection,
            } => {
                intended
                    .channels
                    .iter_mut()
                    .find(|c| c.id == channel_id)
                    .ok_or_else(|| error("model_source_invalid"))?
                    .selection = selection
            }
            ChannelIntentV2::SetEnabled {
                channel_id,
                enabled,
            } => {
                intended
                    .channels
                    .iter_mut()
                    .find(|c| c.id == channel_id)
                    .ok_or_else(|| error("model_source_invalid"))?
                    .connection
                    .enabled = enabled
            }
            ChannelIntentV2::SetOrder {
                channel_id,
                raw_model_ids,
            } => {
                intended
                    .channels
                    .iter_mut()
                    .find(|c| c.id == channel_id)
                    .ok_or_else(|| error("model_source_invalid"))?
                    .model_order = raw_model_ids
            }
        }
    }
    let reconcile_names = intended
        .channels
        .iter()
        .filter(|c| {
            request
                .scope_channel_id
                .as_ref()
                .is_none_or(|id| &c.id == id)
                && raw["providers"].get(&c.connection.provider_id).is_some()
                && !protected(
                    &c.connection.provider_id,
                    &raw["providers"][&c.connection.provider_id],
                )
        })
        .map(|c| (c.id.clone(), c.label.clone()))
        .collect::<Vec<_>>();
    for (id, name) in reconcile_names {
        rename(raw, &mut intended, &mut mappings, &id, &name)?;
    }
    Ok((intended, mappings))
}
fn project_connection_v2(provider: &mut Value, channel: &ChannelV2) {
    if let Some(node) = channel
        .connection
        .nodes
        .iter()
        .find(|n| n.id == channel.connection.active_node_id)
    {
        provider["baseUrl"] = json!(node.url);
    }
    provider["disabled"] = json!(!channel.connection.enabled);
    if let Some(mode) = &channel.connection.auth_mode {
        provider["authMode"] = json!(mode);
    }
}
pub fn save(root: &Path, home: &Path, request: SaveRequestV2) -> Result<FileSnapshotV2> {
    save_with_credentials(root, home, request, &SystemDraftCredentialsV2)
}
pub fn save_with_credentials(
    root: &Path,
    home: &Path,
    mut request: SaveRequestV2,
    vault: &dyn DraftCredentialStoreV2,
) -> Result<FileSnapshotV2> {
    use crate::modules::model_config::model_transaction_v2::{self, TargetV2, WriteV2};
    let _versions = crate::modules::model_config::content_versions::file_saves::admit(root, home)
        .map_err(io)?;
    let _rename = super::channel_rename::admit(root, home).map_err(io)?;
    let _config = TargetFileLock::lock(&home.join("config.json")).map_err(io)?;
    let store = StoreV2::new(root);
    let locked = store.lock()?;
    {
        let _recovery_versions =
            crate::modules::model_config::content_versions::lock_versions(root).map_err(io)?;
        recover_model_transactions(root, home)?;
    }
    let state = locked.snapshot_unlocked()?;
    let (mut raw, bytes) = read_raw(home)?;
    let original_config = raw.clone();
    let source = source_digest(&bytes);
    let revision = root_revision(root, home)?;
    let draft = state
        .draft
        .as_ref()
        .ok_or_else(|| error("model_save_stale"))?;
    if draft.base != request.expected
        || source != request.expected.source_digest
        || state.digests.library != request.expected.library_digest
        || revision != request.expected.root_revision
        || policy_semantic_digest_v2(&state.policy)? != request.expected.policy_digest
    {
        return Err(error("model_save_stale"));
    }
    let history_library = history::validate_save(root, home, &state, draft, &request)?;
    validate_save_scope_v2(&request)?;
    let before_observed = provider_import::read_policy_v2(
        &raw,
        &source,
        &state.policy,
        state.ledger.as_ref(),
        &state.mappings,
        &bindings(root, &raw, &state.policy, Some(draft), &state.mappings)?,
    )?;
    let before_save_state = saved_state(&state.library, &before_observed, &raw)?;
    let (mut intended, mut mappings) =
        transform_draft_v2(root, &source, &state, draft, &mut raw, &mut request)?;
    // A recovery authorization is consumed only by this guarded Save; it is
    // never persisted as authority in the actual saved Policy.
    intended.recovery_id = None;
    for channel in &intended.channels {
        let provider = raw
            .get_mut("providers")
            .and_then(Value::as_object_mut)
            .and_then(|p| p.get_mut(&channel.connection.provider_id))
            .ok_or_else(|| error("model_source_invalid"))?;
        if protected(&channel.connection.provider_id, provider) {
            continue;
        }
        project_connection_v2(provider, channel);
        if let Some(edit) = draft.credential_edits.get(&channel.id) {
            match edit {
                DraftCredentialEditV2::Replace { credential_ref } => {
                    let key = vault.get(credential_ref)?;
                    super::write_credential_row(
                        provider.as_object_mut().unwrap(),
                        "replace",
                        Some(key.as_str()),
                    )
                    .map_err(io)?;
                }
                DraftCredentialEditV2::Clear => {
                    super::write_credential_row(provider.as_object_mut().unwrap(), "clear", None)
                        .map_err(io)?;
                }
            }
            for field in ["apiKeySelectionRevision", "_apiKeyAttempt"] {
                provider.as_object_mut().unwrap().remove(field);
            }
        }
    }
    validate_policy_v2(&intended)?;
    let plan = plan_materialization_v2(&state.library, &state.policy, &intended)?;
    mappings = allocate_mappings(&raw, &intended, &plan.groups, &mappings)?;
    let mut compiled = compiler_v2::compile_file_v2(
        &state.library,
        &state.policy,
        &intended,
        &raw,
        &mappings,
        &auth_bindings(&raw, &intended)?,
    )?;
    let reference_sources = reference_moves(&original_config, &compiled.next_config)?;
    let reference_plan = if reference_sources.is_empty() {
        None
    } else {
        Some(
            crate::modules::model_config::credential_references_v2::prepare(
                root,
                home,
                &original_config,
                &mut compiled.next_config,
                &reference_sources,
                &crate::modules::model_config::credential_references_v2::SystemVault,
            )
            .map_err(io)?,
        )
    };
    compiled.ledger = finalize_materialization_ledger_v2(
        &compiled.policy,
        &compiled.next_config,
        &compiled.targets,
    )?;
    let after = canonical_bytes(&compiled.next_config)?;
    let mut writes = locked
        .prepare_save_unlocked(
            &state.digests,
            &compiled.policy,
            &compiled.ledger,
            &compiled.mappings,
        )?
        .into_iter()
        .map(|w| WriteV2 {
            target: TargetV2::Manager(w.relative_path),
            before: w.before,
            after: Some(w.after),
        })
        .collect::<Vec<_>>();
    if let Some(library) = &history_library {
        let target = root.join(store_v2::LIBRARY_PATH);
        safety::check_path(root, &target, true).map_err(io)?;
        let before = if target.exists() {
            Some(safety::read_file(&target, 32 * 1024 * 1024).map_err(io)?)
        } else {
            None
        };
        if digest(before.as_deref().unwrap_or_default())
            != state.digests.files[store_v2::LIBRARY_PATH]
        {
            return Err(error("model_history_stale"));
        }
        writes.push(WriteV2 {
            target: TargetV2::Manager(store_v2::LIBRARY_PATH.into()),
            before,
            after: Some(canonical_bytes(library)?),
        });
    }
    writes.insert(
        0,
        WriteV2 {
            target: TargetV2::OfficialConfig,
            before: bytes.as_ref().map(|b| b.to_vec()),
            after: Some(after),
        },
    );
    let draft_bytes =
        safety::read_file(&root.join(store_v2::DRAFT_PATH), 32 * 1024 * 1024).map_err(io)?;
    let retained_draft = if let Some(scope) = &request.scope_channel_id {
        let mut policy = draft.policy.clone();
        let position = policy
            .channels
            .iter()
            .position(|c| &c.id == scope)
            .unwrap_or(policy.channels.len());
        policy.channels.retain(|c| &c.id != scope);
        if let Some(saved) = compiled.policy.channels.iter().find(|c| &c.id == scope) {
            policy
                .channels
                .insert(position.min(policy.channels.len()), saved.clone());
        }
        if policy
            .default_channel_id
            .as_ref()
            .is_some_and(|id| !policy.channels.iter().any(|c| &c.id == id))
        {
            policy.default_channel_id = compiled.policy.default_channel_id.clone();
        }
        if policy_semantic_digest_v2(&policy)? != policy_semantic_digest_v2(&compiled.policy)? {
            let mut base = draft.base.clone();
            base.source_digest = digest(&canonical_bytes(&compiled.next_config)?);
            base.policy_digest = policy_semantic_digest_v2(&compiled.policy)?;
            base.draft_generation = base
                .draft_generation
                .checked_add(1)
                .ok_or_else(|| error("model_limit"))?;
            let observation = provider_import::read_policy_v2(
                &compiled.next_config,
                &base.source_digest,
                &compiled.policy,
                Some(&compiled.ledger),
                &compiled.mappings,
                &bindings(
                    root,
                    &compiled.next_config,
                    &compiled.policy,
                    Some(draft),
                    &compiled.mappings,
                )?,
            )?;
            Some(canonical_bytes(&DraftV2 {
                schema_version: 2,
                base,
                credential_edits: draft
                    .credential_edits
                    .iter()
                    .filter(|(id, _)| *id != scope && policy.channels.iter().any(|c| &c.id == *id))
                    .map(|(id, edit)| (id.clone(), edit.clone()))
                    .collect(),
                observed_baseline: Some(observation),
                policy,
            })?)
        } else {
            None
        }
    } else {
        None
    };
    writes.push(WriteV2 {
        target: TargetV2::Manager(store_v2::DRAFT_PATH.into()),
        before: Some(draft_bytes),
        after: retained_draft,
    });
    let after_save_state = saved_state(
        history_library.as_ref().unwrap_or(&state.library),
        &compiled.policy,
        &compiled.next_config,
    )?;
    let _version_lock =
        crate::modules::model_config::content_versions::lock_versions(root).map_err(io)?;
    if before_save_state != after_save_state {
        let planned = crate::modules::model_config::content_versions::plan_save_at(
            root,
            &before_save_state,
            &after_save_state,
            &chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
            &uuid::Uuid::new_v4().to_string(),
        )
        .map_err(io)?;
        writes.push(WriteV2 {
            target: TargetV2::Manager(
                crate::modules::model_config::content_versions::SAVED_VERSIONS_PATH.into(),
            ),
            before: planned.before,
            after: Some(planned.after),
        });
    }
    let result = model_transaction_v2::commit(root, home, writes);
    let (actual, readback) = read_raw(home)?;
    if let Some(plan) = &reference_plan {
        if result.is_ok() {
            crate::modules::model_config::credential_references_v2::finish(
                root,
                home,
                plan,
                &actual,
                &crate::modules::model_config::credential_references_v2::SystemVault,
            )
            .map_err(io)?;
        } else {
            crate::modules::model_config::credential_references_v2::abort(
                root,
                home,
                plan,
                &actual,
                &crate::modules::model_config::credential_references_v2::SystemVault,
            )
            .map_err(io)?;
        }
    }
    result?;
    let bytes = readback;
    view(locked.snapshot_unlocked()?, source_digest(&bytes), revision)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StageConnectionV2 {
    pub expected: Option<ReadBaseV2>,
    pub expected_source: String,
    pub expected_library: String,
    pub channel_id: Option<String>,
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub nodes: Option<Vec<ServiceNode>>,
    #[serde(default)]
    pub active_node_id: Option<String>,
    pub auth_mode: String,
    pub api_key: Option<String>,
    pub raw_model_ids: Vec<String>,
    pub selected_model_ids: Vec<String>,
}
impl Drop for StageConnectionV2 {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        if let Some(key) = &mut self.api_key {
            key.zeroize();
        }
    }
}
fn connection_url(value: &str) -> Result<()> {
    let url = reqwest::Url::parse(value).map_err(|_| error("model_connection_invalid"))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(error("model_connection_invalid"));
    }
    let path = url.path().trim_end_matches('/');
    if ["/responses", "/chat/completions", "/messages"]
        .iter()
        .any(|tail| path.ends_with(tail))
    {
        return Err(error("model_connection_invalid"));
    }
    Ok(())
}
pub fn stage_connection(
    root: &Path,
    home: &Path,
    mut request: StageConnectionV2,
) -> Result<FileSnapshotV2> {
    let _file_admission =
        crate::modules::model_config::content_versions::file_saves::admit(root, home)
            .map_err(io)?;
    connection_url(&request.url)?;
    if let Some(nodes) = &request.nodes {
        if nodes.is_empty()
            || nodes.len() > 32
            || request
                .active_node_id
                .as_ref()
                .is_none_or(|id| !nodes.iter().any(|n| &n.id == id))
        {
            return Err(error("model_connection_invalid"));
        }
        let mut ids = BTreeSet::new();
        for node in nodes {
            check_id(&node.id)?;
            connection_url(&node.url)?;
            if node.name.trim().is_empty() || node.name.len() > 256 || !ids.insert(&node.id) {
                return Err(error("model_connection_invalid"));
            }
        }
        if nodes
            .iter()
            .find(|n| Some(&n.id) == request.active_node_id.as_ref())
            .is_none_or(|n| n.url != request.url)
        {
            return Err(error("model_connection_invalid"));
        }
    } else if request.active_node_id.is_some() {
        return Err(error("model_connection_invalid"));
    }
    let name = request.name.trim();
    if name.is_empty()
        || name.chars().count() > 80
        || name.len() > 256
        || !matches!(request.auth_mode.as_str(), "key" | "none")
    {
        return Err(error("model_connection_invalid"));
    }
    if request.raw_model_ids.is_empty()
        || request.raw_model_ids.len() > 10000
        || request.raw_model_ids.iter().any(|s| {
            s.is_empty() || s.len() > 2048 || s.trim() != s || s.chars().any(char::is_control)
        })
    {
        return Err(error("model_source_invalid"));
    }
    let ids = request.raw_model_ids.iter().collect::<BTreeSet<_>>();
    if ids.len() != request.raw_model_ids.len()
        || request.selected_model_ids.iter().any(|s| !ids.contains(s))
    {
        return Err(error("model_source_invalid"));
    }
    let _rename = super::channel_rename::admit(root, home).map_err(io)?;
    let _config = TargetFileLock::lock(&home.join("config.json")).map_err(io)?;
    let store = StoreV2::new(root);
    let locked = store.lock()?;
    {
        let _recovery_versions =
            crate::modules::model_config::content_versions::lock_versions(root).map_err(io)?;
        recover_model_transactions(root, home)?;
    }
    let state = locked.snapshot_unlocked()?;
    let (raw, bytes) = read_raw(home)?;
    let source = source_digest(&bytes);
    let revision = root_revision(root, home)?;
    if source != request.expected_source
        || state.digests.library != request.expected_library
        || state.draft.as_ref().map(|d| &d.base) != request.expected.as_ref()
    {
        return Err(error("model_read_stale"));
    }
    if state
        .draft
        .as_ref()
        .is_some_and(|d| d.policy.recovery_id.is_some())
    {
        return Err(error("model_history_draft_active"));
    }
    let mut policy = state
        .draft
        .as_ref()
        .map(|d| d.policy.clone())
        .unwrap_or_else(|| state.policy.clone());
    if policy.channels.iter().any(|c| {
        Some(&c.id) != request.channel_id.as_ref()
            && c.label.trim().to_lowercase() == name.to_lowercase()
    }) {
        return Err(error("model_channel_name_duplicate"));
    }
    let id = request
        .channel_id
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    check_id(&id)?;
    let prior = policy.channels.iter().find(|c| c.id == id).cloned();
    if request.channel_id.is_some() && prior.is_none() {
        return Err(error("model_read_stale"));
    }
    if prior.as_ref().is_some_and(|c| {
        protected(
            &c.connection.provider_id,
            &raw["providers"][&c.connection.provider_id],
        )
    }) {
        return Err(error("model_protected_provider"));
    }
    let key = request
        .api_key
        .take()
        .map(Zeroizing::new)
        .filter(|s| !s.is_empty());
    let changed_endpoint = prior.as_ref().is_none_or(|c| {
        !c.connection
            .nodes
            .iter()
            .any(|n| n.id == c.connection.active_node_id && n.url == request.url)
            || request
                .nodes
                .as_ref()
                .is_some_and(|nodes| c.connection.nodes != *nodes)
    });
    if request.auth_mode == "key" && key.is_none() && changed_endpoint {
        return Err(error("model_credential_required"));
    }
    if key
        .as_ref()
        .is_some_and(|k| k.len() > 8192 || k.contains(['\r', '\n']))
    {
        return Err(error("model_credential_invalid"));
    }
    let mut channel = prior.clone().unwrap_or_else(|| ChannelV2 {
        id: id.clone(),
        label: name.into(),
        connection: ConnectionV2 {
            provider_id: allocate(
                name,
                &id,
                &policy
                    .channels
                    .iter()
                    .map(|c| c.connection.provider_id.to_lowercase())
                    .collect(),
            ),
            nodes: vec![],
            active_node_id: String::new(),
            credential_ref: None,
            enabled: false,
            allow_private_network: false,
            auth_mode: Some(request.auth_mode.clone()),
            source_read_id: None,
            imported_config: None,
        },
        selection: ModelSelection::None,
        model_order: vec![],
        models: vec![],
        template_refs: vec![],
        defaults: ModelRules::default(),
        family_overrides: BTreeMap::new(),
        naming: BTreeMap::new(),
        namespace: None,
    });
    channel.label = name.into();
    let node_id = channel.connection.active_node_id.clone();
    if let Some(node) = channel
        .connection
        .nodes
        .iter_mut()
        .find(|n| n.id == node_id)
    {
        node.url = request.url.clone();
    } else {
        let node_id = uuid::Uuid::new_v4().to_string();
        channel.connection.nodes.push(ServiceNode {
            id: node_id.clone(),
            name: "默认节点".into(),
            url: request.url.clone(),
        });
        channel.connection.active_node_id = node_id;
    }
    if let Some(nodes) = &request.nodes {
        channel.connection.nodes = nodes.clone();
        channel.connection.active_node_id = request
            .active_node_id
            .clone()
            .ok_or_else(|| error("model_connection_invalid"))?;
    }
    channel.connection.auth_mode = Some(request.auth_mode.clone());
    let new_reference = key.as_ref().map(|_| uuid::Uuid::new_v4().to_string());
    if let (Some(reference), Some(key)) = (&new_reference, &key) {
        crate::modules::model_config::discovery::import_credential(reference, key).map_err(io)?;
        channel.connection.credential_ref = Some(reference.clone());
    }
    let result = (|| {
        if request.auth_mode == "none" {
            channel.connection.credential_ref = None;
        }
        channel.selection = ModelSelection::Whitelist(request.selected_model_ids.clone());
        let mut old_models = channel.models.clone();
        if changed_endpoint {
            for model in &mut old_models {
                model.source.fields.clear();
            }
        }
        channel.model_order = request.raw_model_ids.clone();
        channel.models = request
            .raw_model_ids
            .iter()
            .map(|raw| {
                old_models
                    .iter()
                    .find(|m| m.raw_model_id == *raw)
                    .cloned()
                    .unwrap_or_else(|| ModelConfigV2 {
                        raw_model_id: raw.clone(),
                        label: raw.clone(),
                        observed: true,
                        source: SourceModelV2 {
                            raw_model_id: raw.clone(),
                            fields: BTreeMap::new(),
                        },
                        explicit_overrides: ModelRules::default(),
                        binding: None,
                    })
            })
            .collect();
        if let Some(slot) = policy.channels.iter_mut().find(|c| c.id == id) {
            *slot = channel.clone();
        } else {
            policy.channels.push(channel.clone());
        }
        let mut base = ReadBaseV2 {
            dto_version: 2,
            draft_id: state
                .draft
                .as_ref()
                .map(|d| d.base.draft_id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            draft_generation: state
                .draft
                .as_ref()
                .map(|d| d.base.draft_generation + 1)
                .unwrap_or(0),
            source_digest: source.clone(),
            policy_digest: policy_semantic_digest_v2(&state.policy)?,
            library_digest: state.digests.library.clone(),
            root_revision: revision.clone(),
        };
        let observation = ReadChannelObservationV2 {
            read_id: uuid::Uuid::new_v4().to_string(),
            base: base.clone(),
            channel_id: id,
            models: channel.models.iter().map(|m| m.source.clone()).collect(),
        };
        policy = match_read_to_draft_v2(&state.library, &policy, &observation)?.next_policy;
        if let Some(old) = &state.draft {
            base.draft_generation = old
                .base
                .draft_generation
                .checked_add(1)
                .ok_or_else(|| error("model_limit"))?;
        }
        locked.stage_draft_unlocked(
            state.draft.as_ref().map(|d| &d.base),
            DraftV2 {
                schema_version: 2,
                base,
                credential_edits: {
                    let mut edits = state
                        .draft
                        .as_ref()
                        .map(|d| d.credential_edits.clone())
                        .unwrap_or_default();
                    if let Some(reference) = &new_reference {
                        edits.insert(
                            channel.id.clone(),
                            DraftCredentialEditV2::Replace {
                                credential_ref: reference.clone(),
                            },
                        );
                    } else if request.auth_mode == "none" {
                        edits.remove(&channel.id);
                    }
                    edits
                },
                observed_baseline: Some(
                    state
                        .draft
                        .as_ref()
                        .and_then(|d| d.observed_baseline.clone())
                        .unwrap_or(provider_import::read_policy_v2(
                            &raw,
                            &source,
                            &state.policy,
                            state.ledger.as_ref(),
                            &state.mappings,
                            &bindings(
                                root,
                                &raw,
                                &state.policy,
                                state.draft.as_ref(),
                                &state.mappings,
                            )?,
                        )?),
                ),
                policy,
            },
        )?;
        view(locked.snapshot_unlocked()?, source, revision)
    })();
    if result.is_err() {
        if let Some(reference) = new_reference {
            let _ = crate::modules::model_config::discovery::forget(&reference);
        }
    }
    result
}

/// Begin from actual saved content without choosing or changing any template.
/// Used by rename/enable/selection editing after a previous save closed its draft.
pub fn begin_draft(root: &Path, home: &Path, request: ReadRequestV2) -> Result<FileSnapshotV2> {
    let _file_admission =
        crate::modules::model_config::content_versions::file_saves::admit(root, home)
            .map_err(io)?;
    let _rename = super::channel_rename::admit(root, home).map_err(io)?;
    let _config = TargetFileLock::lock(&home.join("config.json")).map_err(io)?;
    let store = StoreV2::new(root);
    let locked = store.lock()?;
    {
        let _recovery_versions =
            crate::modules::model_config::content_versions::lock_versions(root).map_err(io)?;
        recover_model_transactions(root, home)?;
    }
    let state = locked.snapshot_unlocked()?;
    let (raw, bytes) = read_raw(home)?;
    let source = source_digest(&bytes);
    let revision = root_revision(root, home)?;
    if source != request.expected_source
        || state.digests.library != request.expected_library
        || state.draft.as_ref().map(|d| &d.base) != request.expected_draft.as_ref()
    {
        return Err(error("model_read_stale"));
    }
    if state.draft.is_none() {
        let policy = provider_import::read_policy_v2(
            &raw,
            &source,
            &state.policy,
            state.ledger.as_ref(),
            &state.mappings,
            &bindings(root, &raw, &state.policy, None, &state.mappings)?,
        )?;
        let base = ReadBaseV2 {
            dto_version: 2,
            draft_id: uuid::Uuid::new_v4().to_string(),
            draft_generation: 0,
            source_digest: source.clone(),
            policy_digest: policy_semantic_digest_v2(&state.policy)?,
            library_digest: state.digests.library.clone(),
            root_revision: revision.clone(),
        };
        locked.stage_draft_unlocked(
            None,
            DraftV2 {
                schema_version: 2,
                base,
                credential_edits: BTreeMap::new(),
                observed_baseline: Some(policy.clone()),
                policy,
            },
        )?;
    }
    view(locked.snapshot_unlocked()?, source, revision)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CopyChannelV2 {
    pub expected: ReadBaseV2,
    pub channel_id: String,
    pub name: String,
    /// Match the adopted copy dialog: explicit opt-in, never accept a key value.
    #[serde(default)]
    pub reuse_source_key: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CopySourceV2 {
    schema_version: u32,
    channel_id: String,
    source_provider_id: String,
    source_digest: String,
    reuse_source_key: bool,
    /// Pending replace/clear is fixed by the backend draft, not the source row.
    source_credential_edit: Option<DraftCredentialEditV2>,
}
fn copy_path(root: &Path, id: &str) -> Result<std::path::PathBuf> {
    check_id(id)?;
    let path = root
        .join("manager-state/model-v2-copy-sources")
        .join(format!("{id}.json"));
    safety::check_path(root, &path, true).map_err(io)?;
    Ok(path)
}
fn copied_seed(
    root: &Path,
    raw: &Value,
    channel: &ChannelV2,
    source: &str,
) -> Result<Option<Value>> {
    let Some(reference) = &channel.connection.imported_config else {
        return Ok(None);
    };
    let bytes = safety::read_file(&copy_path(root, &reference.id)?, 65536).map_err(io)?;
    if digest(&bytes) != reference.digest {
        return Err(error("model_source_invalid"));
    }
    let record: CopySourceV2 =
        serde_json::from_value(parse_strict_json_v2(&bytes)?).map_err(json_error)?;
    if record.schema_version != 2
        || record.channel_id != channel.id
        || record.source_digest != source
    {
        return Err(error("model_save_stale"));
    }
    let source = raw["providers"]
        .get(&record.source_provider_id)
        .ok_or_else(|| error("model_save_stale"))?;
    if protected(&record.source_provider_id, source) {
        return Err(error("model_protected_provider"));
    }
    let mut value = source.clone();
    if !record.reuse_source_key || record.source_credential_edit.is_some() {
        // Retain public and unknown noncredential source fields, while never
        // retaining the source's pool, headers or external credential refs.
        for field in crate::modules::model_config::provider_fields::PRIVATE
            .iter()
            .copied()
            .chain(["apiKeySelectionRevision", "_apiKeyAttempt"])
        {
            value
                .as_object_mut()
                .ok_or_else(|| error("model_source_invalid"))?
                .remove(field);
        }
    }
    value["name"] = json!(channel.label);
    value["disabled"] = json!(true);
    Ok(Some(value))
}
pub fn copy_channel(root: &Path, home: &Path, request: CopyChannelV2) -> Result<FileSnapshotV2> {
    let _file_admission =
        crate::modules::model_config::content_versions::file_saves::admit(root, home)
            .map_err(io)?;
    let name = request.name.trim();
    if name.is_empty() || name.chars().count() > 80 || name.len() > 256 {
        return Err(error("model_channel_name"));
    }
    let _rename = super::channel_rename::admit(root, home).map_err(io)?;
    let _config = TargetFileLock::lock(&home.join("config.json")).map_err(io)?;
    let store = StoreV2::new(root);
    let locked = store.lock()?;
    {
        let _recovery_versions =
            crate::modules::model_config::content_versions::lock_versions(root).map_err(io)?;
        recover_model_transactions(root, home)?;
    }
    let state = locked.snapshot_unlocked()?;
    let (raw, bytes) = read_raw(home)?;
    let source = source_digest(&bytes);
    let revision = root_revision(root, home)?;
    let draft = state
        .draft
        .as_ref()
        .ok_or_else(|| error("model_read_stale"))?;
    if draft.base != request.expected
        || source != request.expected.source_digest
        || revision != request.expected.root_revision
        || state.digests.library != request.expected.library_digest
        || policy_semantic_digest_v2(&state.policy)? != request.expected.policy_digest
    {
        return Err(error("model_read_stale"));
    }
    if draft.policy.recovery_id.is_some() {
        return Err(error("model_history_draft_active"));
    }
    if draft
        .policy
        .channels
        .iter()
        .any(|c| c.label.trim().to_lowercase() == name.to_lowercase())
    {
        return Err(error("model_channel_name_duplicate"));
    }
    let mut channel = draft
        .policy
        .channels
        .iter()
        .find(|c| c.id == request.channel_id)
        .cloned()
        .ok_or_else(|| error("model_source_invalid"))?;
    let source_id = channel.connection.provider_id.clone();
    if protected(&source_id, &raw["providers"][&source_id]) {
        return Err(error("model_protected_provider"));
    }
    if raw["providers"].get(&source_id).is_none() {
        return Err(error("model_copy_unsaved_source"));
    }
    let source_credential_edit = draft.credential_edits.get(&channel.id).cloned();
    let copied_credential_edit = if request.reuse_source_key {
        source_credential_edit.clone()
    } else {
        Some(DraftCredentialEditV2::Clear)
    };
    if !request.reuse_source_key
        || matches!(copied_credential_edit, Some(DraftCredentialEditV2::Clear))
    {
        channel.connection.credential_ref = None;
    }
    channel.id = uuid::Uuid::new_v4().to_string();
    if let Some(namespace) = &mut channel.namespace {
        namespace.channel_id = channel.id.clone();
    }
    channel.label = name.into();
    channel.connection.enabled = false;
    channel.connection.provider_id = allocate(
        name,
        &channel.id,
        &draft
            .policy
            .channels
            .iter()
            .map(|c| c.connection.provider_id.to_lowercase())
            .collect(),
    );
    let next_generation = draft
        .base
        .draft_generation
        .checked_add(1)
        .ok_or_else(|| error("model_limit"))?;
    let record = CopySourceV2 {
        schema_version: 2,
        channel_id: channel.id.clone(),
        source_provider_id: source_id,
        source_digest: source.clone(),
        reuse_source_key: request.reuse_source_key,
        source_credential_edit: request
            .reuse_source_key
            .then_some(source_credential_edit)
            .flatten(),
    };
    let id = uuid::Uuid::new_v4().to_string();
    let bytes = canonical_bytes(&record)?;
    let path = copy_path(root, &id)?;
    crate::infrastructure::atomic_write::atomic_write(&path, &bytes, 0o600).map_err(io)?;
    channel.connection.imported_config =
        Some(crate::modules::model_config::schema::ImportedProviderRef {
            id,
            digest: digest(&bytes),
        });
    let mut next = draft.clone();
    next.base.draft_generation = next_generation;
    if let Some(edit) = copied_credential_edit {
        next.credential_edits.insert(channel.id.clone(), edit);
    }
    next.policy.channels.push(channel);
    if let Err(error) = locked.stage_draft_unlocked(Some(&draft.base), next) {
        let _ = std::fs::remove_file(path);
        return Err(error);
    }
    view(locked.snapshot_unlocked()?, source, revision)
}

#[cfg(test)]
#[path = "config_file_v2_copy_tests.rs"]
mod copy_tests;

/// Public semantic save facts plus plain file-owned keys only. Opaque external
/// credential stores cannot acquire a fabricated save time from a file rewrite.
pub(crate) fn saved_state(
    library: &LibraryV2,
    policy: &PolicyV2,
    raw: &Value,
) -> Result<crate::modules::model_config::content_versions::SaveState> {
    use crate::modules::model_config::{
        file_objects_v2::{self, CapturedObjects, SavedObjects, SecretValues},
        portable_v2::{self, ModelExportScopeV2},
    };
    let channel_ids = policy
        .channels
        .iter()
        .filter(|c| {
            !protected(
                &c.connection.provider_id,
                &raw["providers"][&c.connection.provider_id],
            )
        })
        .map(|c| c.id.clone())
        .collect::<BTreeSet<_>>();
    let template_refs = library
        .templates
        .iter()
        .flat_map(|t| {
            t.revisions.iter().map(move |r| RefV2 {
                template_id: t.id.clone(),
                revision_id: r.id.clone(),
                digest: r.digest.clone(),
            })
        })
        .collect();
    let mut models = portable_v2::export_portable_models_v2(
        library,
        policy,
        &ModelExportScopeV2 {
            channel_ids: channel_ids.clone(),
            template_refs,
            include_global: true,
            ..Default::default()
        },
    )?;
    let public_sources = policy
        .channels
        .iter()
        .filter(|c| channel_ids.contains(&c.id))
        .filter_map(|c| {
            crate::modules::model_config::provider_fields::public_from_raw(
                &raw["providers"][&c.connection.provider_id],
            )
            .ok()
            .map(|public| (c.id.clone(), public))
        })
        .collect::<BTreeMap<_, _>>();
    if !public_sources.is_empty() {
        models = portable_v2::attach_public_sources_v2(&models, &public_sources)?;
    }
    let mut keys = SecretValues::new();
    for channel in policy
        .channels
        .iter()
        .filter(|c| channel_ids.contains(&c.id))
    {
        if let Some(key) = raw["providers"][&channel.connection.provider_id]["apiKey"]
            .as_str()
            .filter(|k| !k.starts_with("keychain:"))
        {
            keys.insert(channel.id.clone(), Zeroizing::new(key.into()))
                .map_err(io)?;
        }
    }
    let objects = SavedObjects::capture(
        CapturedObjects {
            models,
            manager: Default::default(),
            assets: Default::default(),
            channel_keys: keys,
            channel_authorizations: BTreeMap::new(),
            webdav_passwords: SecretValues::new(),
            content_versions: vec![],
        },
        None,
    )
    .map_err(io)?;
    file_objects_v2::save_state(&objects).map_err(io)
}

pub fn library_snapshot(root: &Path, home: &Path) -> Result<StoreSnapshotV2> {
    let _file_admission =
        crate::modules::model_config::content_versions::file_saves::admit(root, home)
            .map_err(io)?;
    let _rename = super::channel_rename::admit(root, home).map_err(io)?;
    let _config = TargetFileLock::lock(&home.join("config.json")).map_err(io)?;
    let store = StoreV2::new(root);
    let locked = store.lock()?;
    {
        let _recovery_versions =
            crate::modules::model_config::content_versions::lock_versions(root).map_err(io)?;
        recover_model_transactions(root, home)?;
    }
    locked.snapshot_unlocked()
}
pub fn save_revision(
    root: &Path,
    home: &Path,
    request: &SaveRevisionRequestV2,
) -> Result<LibraryV2> {
    use crate::modules::model_config::content_versions;
    let _admission = content_versions::file_saves::admit(root, home).map_err(io)?;
    let _rename = super::channel_rename::admit(root, home).map_err(io)?;
    let _config = TargetFileLock::lock(&home.join("config.json")).map_err(io)?;
    let store = StoreV2::new(root);
    let locked = store.lock()?;
    {
        let _recovery_versions =
            crate::modules::model_config::content_versions::lock_versions(root).map_err(io)?;
        recover_model_transactions(root, home)?;
    }
    let state = locked.snapshot_unlocked()?;
    let next = save_revision_plan_v2(&state.library, request)?;
    commit_library_change(root, home, &locked, &state, next)
}
fn commit_library_change(
    root: &Path,
    home: &Path,
    locked: &store_v2::LockedStoreV2<'_>,
    state: &StoreSnapshotV2,
    next: LibraryV2,
) -> Result<LibraryV2> {
    use crate::modules::model_config::{
        content_versions,
        model_transaction_v2::{self, TargetV2, WriteV2},
    };
    let empty = store_v2::empty_policy_v2();
    let before = saved_state(&state.library, &empty, &json!({}))?;
    let after = saved_state(&next, &empty, &json!({}))?;
    let path = root.join(store_v2::LIBRARY_PATH);
    let original = if path.exists() {
        Some(safety::read_file(&path, 32 * 1024 * 1024).map_err(io)?)
    } else {
        None
    };
    let mut writes = vec![WriteV2 {
        target: TargetV2::Manager(store_v2::LIBRARY_PATH.into()),
        before: original,
        after: Some(canonical_bytes(&next)?),
    }];
    let _versions = content_versions::lock_versions(root).map_err(io)?;
    if before != after {
        let plan = content_versions::plan_save_at(
            root,
            &before,
            &after,
            &chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
            &uuid::Uuid::new_v4().to_string(),
        )
        .map_err(io)?;
        writes.push(WriteV2 {
            target: TargetV2::Manager(content_versions::SAVED_VERSIONS_PATH.into()),
            before: plan.before,
            after: Some(plan.after),
        });
    }
    model_transaction_v2::commit(root, home, writes)?;
    Ok(locked.snapshot_unlocked()?.library)
}
#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum LibraryIntentV2 {
    SetDefault {
        reference: RefV2,
    },
    ClearDefault {
        reference: RefV2,
    },
    DeleteRevisions {
        template_id: String,
        revision_ids: Vec<String>,
    },
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LibraryChangeV2 {
    pub expected_library: String,
    pub intent: LibraryIntentV2,
}
pub fn change_library(root: &Path, home: &Path, request: LibraryChangeV2) -> Result<LibraryV2> {
    let _admission = crate::modules::model_config::content_versions::file_saves::admit(root, home)
        .map_err(io)?;
    let _rename = super::channel_rename::admit(root, home).map_err(io)?;
    let _config = TargetFileLock::lock(&home.join("config.json")).map_err(io)?;
    let store = StoreV2::new(root);
    let locked = store.lock()?;
    {
        let _versions =
            crate::modules::model_config::content_versions::lock_versions(root).map_err(io)?;
        recover_model_transactions(root, home)?;
    }
    let state = locked.snapshot_unlocked()?;
    use crate::modules::model_config::template_ops_v2 as ops;
    let next = match request.intent {
        LibraryIntentV2::SetDefault { reference } => {
            ops::set_default(&state.library, &request.expected_library, &reference)?
        }
        LibraryIntentV2::ClearDefault { reference } => {
            ops::clear_default(&state.library, &request.expected_library, &reference)?
        }
        LibraryIntentV2::DeleteRevisions {
            template_id,
            revision_ids,
        } => {
            let dependencies =
                crate::modules::model_config::model_transaction_v2::retained_template_refs(
                    root, home,
                )?;
            ops::delete_revisions(
                &state.library,
                &request.expected_library,
                &template_id,
                &revision_ids,
                &state.policy,
                state.draft.as_ref().map(|d| &d.policy),
                &dependencies,
            )?
        }
    };
    commit_library_change(root, home, &locked, &state, next)
}
pub fn template_usage(
    root: &Path,
    home: &Path,
    template_id: &str,
    revision_id: &str,
) -> Result<crate::modules::model_config::template_ops_v2::UsageReportV2> {
    let snapshot = library_snapshot(root, home)?;
    let dependencies =
        crate::modules::model_config::model_transaction_v2::retained_template_refs(root, home)?;
    crate::modules::model_config::template_ops_v2::usage(
        &snapshot.library,
        &snapshot.policy,
        snapshot.draft.as_ref().map(|d| &d.policy),
        template_id,
        revision_id,
        &dependencies,
    )
}

/// Internal capture boundary: only actual saved config/metadata, never a draft.
/// The closure cannot turn a caller-supplied path or policy into source authority.
pub(crate) fn with_saved_config<T>(
    root: &Path,
    home: &Path,
    read: impl FnOnce(&LibraryV2, &PolicyV2, &MappingFileV2, &Value, &str, &str) -> Result<T>,
) -> Result<T> {
    // Source preview must never repair a pending business transaction.
    let pending = [
        "manager-state/saved-file-save-pending.json",
        "manager-state/provider-rename-pending.json",
        "manager-state/model-transaction-v2.pending.json",
        "manager-state/credential-references-v2.pending.json",
    ];
    let locks = pending
        .iter()
        .map(|relative| {
            let path = root.join(relative);
            safety::check_path(root, &path, true).map_err(io)?;
            TargetFileLock::lock(&path).map_err(io)
        })
        .collect::<Result<Vec<_>>>()?;
    for relative in pending {
        let path = root.join(relative);
        if std::fs::symlink_metadata(path).is_ok() {
            return Err(error("model_transaction_pending"));
        }
    }
    let _config = TargetFileLock::lock(&home.join("config.json")).map_err(io)?;
    let store = StoreV2::new(root);
    let locked = store.lock()?;
    let state = locked.snapshot_saved_unlocked()?;
    let _locks = locks;
    let (raw, bytes) = read_raw(home)?;
    let source = source_digest(&bytes);
    let revision = root_revision(root, home)?;
    let policy = provider_import::read_policy_v2(
        &raw,
        &source,
        &state.policy,
        state.ledger.as_ref(),
        &state.mappings,
        &bindings(root, &raw, &state.policy, None, &state.mappings)?,
    )?;
    read(
        &state.library,
        &policy,
        &state.mappings,
        &raw,
        &source,
        &revision,
    )
}

pub trait DraftCredentialStoreV2 {
    fn put(&self, reference: &str, value: &str) -> Result<()>;
    fn get(&self, reference: &str) -> Result<Zeroizing<String>>;
    fn remove(&self, reference: &str) -> Result<()>;
}
pub struct SystemDraftCredentialsV2;
impl DraftCredentialStoreV2 for SystemDraftCredentialsV2 {
    fn put(&self, reference: &str, value: &str) -> Result<()> {
        crate::modules::model_config::discovery::import_credential(reference, value).map_err(io)
    }
    fn get(&self, reference: &str) -> Result<Zeroizing<String>> {
        crate::modules::model_config::discovery::credential(reference).map_err(io)
    }
    fn remove(&self, reference: &str) -> Result<()> {
        crate::modules::model_config::discovery::forget(reference).map_err(io)
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StageCredentialV2 {
    pub expected: ReadBaseV2,
    pub channel_id: String,
    pub action: String,
    pub value: Option<String>,
}
impl Drop for StageCredentialV2 {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        if let Some(value) = &mut self.value {
            value.zeroize();
        }
    }
}
pub fn stage_credential(
    root: &Path,
    home: &Path,
    request: StageCredentialV2,
) -> Result<FileSnapshotV2> {
    stage_credential_with_store(root, home, request, &SystemDraftCredentialsV2)
}
pub fn stage_credential_with_store(
    root: &Path,
    home: &Path,
    mut request: StageCredentialV2,
    vault: &dyn DraftCredentialStoreV2,
) -> Result<FileSnapshotV2> {
    let value = request.value.take().map(Zeroizing::new);
    if request.action == "replace" {
        if value
            .as_ref()
            .is_none_or(|s| s.trim().is_empty() || s.len() > 8192 || s.contains(['\r', '\n']))
        {
            return Err(error("model_credential_invalid"));
        }
    } else if request.action != "clear" || value.is_some() {
        return Err(error("model_credential_invalid"));
    }
    let _admission = crate::modules::model_config::content_versions::file_saves::admit(root, home)
        .map_err(io)?;
    let _rename = super::channel_rename::admit(root, home).map_err(io)?;
    let _config = TargetFileLock::lock(&home.join("config.json")).map_err(io)?;
    let store = StoreV2::new(root);
    let locked = store.lock()?;
    {
        let _versions =
            crate::modules::model_config::content_versions::lock_versions(root).map_err(io)?;
        recover_model_transactions(root, home)?;
    }
    let state = locked.snapshot_unlocked()?;
    let (raw, bytes) = read_raw(home)?;
    let source = source_digest(&bytes);
    let revision = root_revision(root, home)?;
    let mut draft = state
        .draft
        .clone()
        .ok_or_else(|| error("model_read_stale"))?;
    if draft.base != request.expected
        || source != request.expected.source_digest
        || revision != request.expected.root_revision
        || state.digests.library != request.expected.library_digest
        || policy_semantic_digest_v2(&state.policy)? != request.expected.policy_digest
    {
        return Err(error("model_read_stale"));
    }
    if draft.policy.recovery_id.is_some() {
        return Err(error("model_history_draft_active"));
    }
    let channel = draft
        .policy
        .channels
        .iter_mut()
        .find(|c| c.id == request.channel_id)
        .ok_or_else(|| error("model_source_invalid"))?;
    if protected(
        &channel.connection.provider_id,
        &raw["providers"][&channel.connection.provider_id],
    ) || matches!(
        channel.connection.auth_mode.as_deref(),
        Some("oauth" | "forward")
    ) {
        return Err(error("model_protected_provider"));
    }
    let reference = if request.action == "replace" {
        let reference = uuid::Uuid::new_v4().to_string();
        let value = value.as_ref().unwrap();
        vault.put(&reference, value)?;
        if vault.get(&reference)?.as_str() != value.as_str() {
            let _ = vault.remove(&reference);
            return Err(error("model_credential_readback"));
        }
        channel.connection.credential_ref = Some(reference.clone());
        draft.credential_edits.insert(
            channel.id.clone(),
            DraftCredentialEditV2::Replace {
                credential_ref: reference.clone(),
            },
        );
        Some(reference)
    } else {
        channel.connection.credential_ref = None;
        draft
            .credential_edits
            .insert(channel.id.clone(), DraftCredentialEditV2::Clear);
        None
    };
    draft.base.draft_generation = draft
        .base
        .draft_generation
        .checked_add(1)
        .ok_or_else(|| error("model_limit"))?;
    if let Err(error) = locked.stage_draft_unlocked(Some(&request.expected), draft) {
        if let Some(reference) = reference {
            let _ = vault.remove(&reference);
        }
        return Err(error);
    }
    view(locked.snapshot_unlocked()?, source, revision)
}
struct ResolvedCredentialV2 {
    configured: bool,
    value: Zeroizing<String>,
    source: &'static str,
}
fn resolve_file_credential(row: &Value, provider: &str) -> Result<ResolvedCredentialV2> {
    let raw = row["apiKey"]
        .as_str()
        .or_else(|| {
            row["apiKeyPool"]
                .as_array()
                .and_then(|p| p.first())
                .and_then(|p| p["key"].as_str())
        })
        .unwrap_or("");
    let (value, source) = if let Some(name) = raw.strip_prefix('$') {
        let name = name.trim_start_matches('{').trim_end_matches('}');
        if name.is_empty()
            || name.len() > 128
            || name
                .bytes()
                .any(|b| !b.is_ascii_alphanumeric() && b != b'_')
        {
            return Err(error("model_credential_invalid"));
        }
        (std::env::var(name).unwrap_or_default(), "环境变量")
    } else if let Some(account) = raw.strip_prefix("keychain:") {
        crate::modules::model_config::provider_auth::owned_account(raw, provider).map_err(io)?;
        (
            keyring::Entry::new("opencodex.provider-api-key.v1", account)
                .map_err(io)?
                .get_password()
                .unwrap_or_default(),
            "系统钥匙串",
        )
    } else {
        (raw.to_string(), "配置文件")
    };
    if value.len() > 8192 || value.contains(['\r', '\n']) {
        return Err(error("model_credential_invalid"));
    }
    Ok(ResolvedCredentialV2 {
        configured: !raw.is_empty(),
        value: Zeroizing::new(value),
        source,
    })
}
pub fn credential_view(
    root: &Path,
    home: &Path,
    channel_id: &str,
) -> Result<super::CredentialView> {
    credential_view_with_store(root, home, channel_id, &SystemDraftCredentialsV2)
}
pub fn credential_view_with_store(
    root: &Path,
    home: &Path,
    channel_id: &str,
    vault: &dyn DraftCredentialStoreV2,
) -> Result<super::CredentialView> {
    check_id(channel_id)?;
    let _admission = crate::modules::model_config::content_versions::file_saves::admit(root, home)
        .map_err(io)?;
    let _rename = super::channel_rename::admit(root, home).map_err(io)?;
    let _config = TargetFileLock::lock(&home.join("config.json")).map_err(io)?;
    let store = StoreV2::new(root);
    let locked = store.lock()?;
    {
        let _versions =
            crate::modules::model_config::content_versions::lock_versions(root).map_err(io)?;
        recover_model_transactions(root, home)?;
    }
    let state = locked.snapshot_unlocked()?;
    let (raw, bytes) = read_raw(home)?;
    let actual = provider_import::read_policy_v2(
        &raw,
        &source_digest(&bytes),
        &state.policy,
        state.ledger.as_ref(),
        &state.mappings,
        &bindings(
            root,
            &raw,
            &state.policy,
            state.draft.as_ref(),
            &state.mappings,
        )?,
    )?;
    let channel = state
        .draft
        .as_ref()
        .and_then(|d| d.policy.channels.iter().find(|c| c.id == channel_id))
        .or_else(|| actual.channels.iter().find(|c| c.id == channel_id))
        .ok_or_else(|| error("model_source_invalid"))?;
    if protected(
        &channel.connection.provider_id,
        &raw["providers"][&channel.connection.provider_id],
    ) {
        return Err(error("model_protected_provider"));
    }
    if let Some(edit) = state
        .draft
        .as_ref()
        .and_then(|d| d.credential_edits.get(channel_id))
    {
        return match edit {
            DraftCredentialEditV2::Clear => Ok(super::CredentialView {
                configured: false,
                value: String::new(),
                source: "草稿中已清空".into(),
                viewable: false,
            }),
            DraftCredentialEditV2::Replace { credential_ref } => {
                let value = vault.get(credential_ref)?;
                Ok(super::CredentialView {
                    configured: true,
                    value: value.to_string(),
                    source: "本机待保存".into(),
                    viewable: !value.is_empty(),
                })
            }
        };
    }
    let provider = raw["providers"]
        .get(&channel.connection.provider_id)
        .ok_or_else(|| error("model_source_invalid"))?;
    let resolved = resolve_file_credential(provider, &channel.connection.provider_id)?;
    Ok(super::CredentialView {
        configured: resolved.configured,
        viewable: !resolved.value.is_empty(),
        value: resolved.value.to_string(),
        source: resolved.source.into(),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MeasurementIntentV2 {
    pub expected: ReadBaseV2,
    pub channel_id: String,
    pub node_ids: Vec<String>,
    pub discovery_kind: String,
    pub run_id: String,
}
pub fn prepare_measurements(
    root: &Path,
    home: &Path,
    request: MeasurementIntentV2,
) -> Result<crate::modules::model_config::measurements::Request> {
    prepare_measurements_with_store(root, home, request, &SystemDraftCredentialsV2)
}
pub fn prepare_measurements_with_store(
    root: &Path,
    home: &Path,
    request: MeasurementIntentV2,
    vault: &dyn DraftCredentialStoreV2,
) -> Result<crate::modules::model_config::measurements::Request> {
    let protocol = match request.discovery_kind.as_str() {
        "openai_list" => "openai-chat",
        "anthropic_list" => "anthropic",
        "google_list" => "google",
        "ollama_list" => "ollama-native",
        _ => return Err(error("model_discovery_kind")),
    };
    check_id(&request.run_id)?;
    check_id(&request.channel_id)?;
    if request.node_ids.is_empty()
        || request.node_ids.len() > 32
        || request.node_ids.iter().collect::<BTreeSet<_>>().len() != request.node_ids.len()
    {
        return Err(error("model_node_scope"));
    }
    let _admission = crate::modules::model_config::content_versions::file_saves::admit(root, home)
        .map_err(io)?;
    let _rename = super::channel_rename::admit(root, home).map_err(io)?;
    let _config = TargetFileLock::lock(&home.join("config.json")).map_err(io)?;
    let store = StoreV2::new(root);
    let locked = store.lock()?;
    {
        let _versions =
            crate::modules::model_config::content_versions::lock_versions(root).map_err(io)?;
        recover_model_transactions(root, home)?;
    }
    let state = locked.snapshot_unlocked()?;
    let (raw, bytes) = read_raw(home)?;
    let revision = root_revision(root, home)?;
    let draft = state
        .draft
        .as_ref()
        .ok_or_else(|| error("model_read_stale"))?;
    if draft.base != request.expected
        || source_digest(&bytes) != request.expected.source_digest
        || revision != request.expected.root_revision
        || state.digests.library != request.expected.library_digest
        || policy_semantic_digest_v2(&state.policy)? != request.expected.policy_digest
    {
        return Err(error("model_read_stale"));
    }
    let channel = draft
        .policy
        .channels
        .iter()
        .find(|c| c.id == request.channel_id)
        .ok_or_else(|| error("model_source_invalid"))?;
    let mut nodes = vec![];
    for id in &request.node_ids {
        check_id(id)?;
        let node = channel
            .connection
            .nodes
            .iter()
            .find(|n| &n.id == id)
            .ok_or_else(|| error("model_node_missing"))?;
        connection_url(&node.url)?;
        nodes.push(node.clone());
    }
    if protected(
        &channel.connection.provider_id,
        &raw["providers"][&channel.connection.provider_id],
    ) || matches!(
        channel.connection.auth_mode.as_deref(),
        Some("oauth" | "forward")
    ) {
        return Err(error("model_node_auth_unsupported"));
    }
    let edit = draft.credential_edits.get(&channel.id);
    let key = if channel.connection.auth_mode.as_deref() == Some("none")
        || matches!(edit, Some(DraftCredentialEditV2::Clear))
    {
        None
    } else if let Some(DraftCredentialEditV2::Replace { credential_ref }) = edit {
        Some(vault.get(credential_ref)?)
    } else {
        let baseline = draft
            .observed_baseline
            .as_ref()
            .and_then(|b| b.channels.iter().find(|c| c.id == channel.id))
            .or_else(|| state.policy.channels.iter().find(|c| c.id == channel.id));
        if nodes.iter().any(|node| {
            baseline.is_none_or(|b| {
                !b.connection
                    .nodes
                    .iter()
                    .any(|n| n.id == node.id && n.url == node.url)
            })
        }) {
            return Err(error("model_node_credential_unconfirmed"));
        }
        let row = raw["providers"]
            .get(&channel.connection.provider_id)
            .ok_or_else(|| error("model_node_credential_unconfirmed"))?;
        let key = resolve_file_credential(row, &channel.connection.provider_id)?;
        if key.value.is_empty() {
            None
        } else {
            Some(key.value)
        }
    };
    if channel.connection.auth_mode.as_deref() != Some("none")
        && protocol != "ollama-native"
        && key.as_ref().is_none_or(|k| k.is_empty())
    {
        return Err(error("model_credential_required"));
    }
    Ok(crate::modules::model_config::measurements::Request {
        run_id: request.run_id,
        protocol: protocol.into(),
        nodes,
        api_key: key.as_ref().map(|k| k.to_string()),
        credential_ref: None,
        allow_private_network: false,
        provider_id: None,
        home: None,
    })
}

fn reference_moves(
    original: &Value,
    next: &Value,
) -> Result<Vec<crate::modules::model_config::credential_references_v2::ReferenceSourceV2>> {
    fn refs(value: &Value, out: &mut BTreeSet<String>) {
        match value {
            Value::String(v) if v.starts_with("keychain:") => {
                out.insert(v[9..].to_owned());
            }
            Value::Object(v) => v.values().for_each(|v| refs(v, out)),
            Value::Array(v) => v.iter().for_each(|v| refs(v, out)),
            _ => {}
        }
    }
    let mut moves = vec![];
    for (target, row) in next["providers"].as_object().into_iter().flatten() {
        let mut accounts = BTreeSet::new();
        for field in crate::modules::model_config::provider_fields::PRIVATE {
            if let Some(value) = row.get(*field) {
                refs(value, &mut accounts);
            }
        }
        let sources = accounts
            .iter()
            .filter(|a| a.as_str() != target && !a.starts_with(&(target.clone() + "/")))
            .map(|a| {
                original["providers"]
                    .as_object()
                    .into_iter()
                    .flatten()
                    .filter(|(id, _)| {
                        a.as_str() == id.as_str() || a.starts_with(&((*id).clone() + "/"))
                    })
                    .max_by_key(|(id, _)| id.len())
                    .map(|(id, _)| id.clone())
                    .ok_or_else(|| error("model_credential_reference"))
            })
            .collect::<Result<BTreeSet<_>>>()?;
        if sources.len() > 1 {
            return Err(error("model_credential_reference"));
        }
        if let Some(source) = sources.first() {
            moves.push(
                crate::modules::model_config::credential_references_v2::ReferenceSourceV2 {
                    source_provider_id: source.clone(),
                    target_provider_id: target.clone(),
                },
            );
        }
    }
    Ok(moves)
}

fn recover_model_transactions(root: &Path, home: &Path) -> Result<()> {
    crate::modules::model_config::model_transaction_v2::recover(root, home)?;
    let (actual, _) = read_raw(home)?;
    crate::modules::model_config::credential_references_v2::recover(
        root,
        home,
        &actual,
        &crate::modules::model_config::credential_references_v2::SystemVault,
    )
    .map_err(io)?;
    Ok(())
}

/// Only the trusted run owner can construct ApprovedSavePlanV2. Recheck all
/// immutable facts under the same file/model locks before publishing its library.
pub fn accept_agent_library(
    root: &Path,
    home: &Path,
    plan: &crate::modules::model_config::agent_runtime_v2::ApprovedSavePlanV2,
) -> Result<LibraryV2> {
    let _admission = crate::modules::model_config::content_versions::file_saves::admit(root, home)
        .map_err(io)?;
    let _rename = super::channel_rename::admit(root, home).map_err(io)?;
    let _config = TargetFileLock::lock(&home.join("config.json")).map_err(io)?;
    let store = StoreV2::new(root);
    let locked = store.lock()?;
    {
        let _versions =
            crate::modules::model_config::content_versions::lock_versions(root).map_err(io)?;
        recover_model_transactions(root, home)?;
    }
    let state = locked.snapshot_unlocked()?;
    let (_, bytes) = read_raw(home)?;
    if state.digests.library != plan.expected_library
        || source_digest(&bytes) != plan.expected_source
        || root_revision(root, home)? != plan.expected_root_revision
    {
        return Err(error("model_save_stale"));
    }
    validate_library_v2(&plan.library)?;
    for template in &state.library.templates {
        let next = plan
            .library
            .templates
            .iter()
            .find(|t| t.id == template.id)
            .ok_or_else(|| error("model_agent_commit_unconfirmed"))?;
        if template
            .revisions
            .iter()
            .any(|r| !next.revisions.contains(r))
        {
            return Err(error("model_agent_commit_unconfirmed"));
        }
    }
    commit_library_change(root, home, &locked, &state, plan.library.clone())
}

#[path = "config_file_v2_export.rs"]
pub mod export;

#[path = "config_file_v2_history.rs"]
pub mod history;

#[path = "config_file_v2_discard.rs"]
mod draft_discard;
pub use draft_discard::discard;
