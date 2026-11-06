//! Pure selected-object overlay over zero or one explicit complete baseline.
//! A chosen conflict head is resolved by L0 BEFORE this call; no latest lookup,
//! network, file access, clock, or automatic multi-head merge exists here.
use super::{
    file_objects_v2::{AuthorizationValidator, CapturedObjects, SavedObjects, SecretValues},
    portable_v2::{self, ModelPayloadV2, PortableChannelV2, PortableModelsV2, PortablePolicyV2},
    schema::{invalid, ModelError, ModelResult, ModelSelection},
    schema_v2::{self, LibraryV2, RefV2, TemplateV2},
};
use crate::modules::sync::{
    portable_assets::{self, Assets, Mcp, Skill},
    portable_manager,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use zeroize::Zeroizing;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageV2 {
    #[default]
    RemoteSync,
    LocalFile,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct SelectionV2 {
    pub channels: Vec<ChannelSelectionV2>,
    pub template_refs: Vec<RefV2>,
    pub adopt_template_defaults: Vec<RefV2>,
    pub include_global_template_refs: bool,
    pub manager: ManagerSelectionV2,
    pub skill_names: Vec<String>,
    pub mcp_sources: Vec<McpSourceSelectionV2>,
    pub extension_credentials: ExtensionCredentialsV2,
    pub usage: UsageV2,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelSelectionV2 {
    pub portable_channel_id: String,
    #[serde(default)]
    pub model_ids: Option<Vec<String>>,
    #[serde(default)]
    pub family_ids: Option<Vec<String>>,
    #[serde(default)]
    pub include_connection: bool,
    #[serde(default)]
    pub include_key: bool,
    #[serde(default)]
    pub include_authorization: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ManagerSelectionV2 {
    pub preferences: bool,
    pub extensions: bool,
    pub webdav_ids: Vec<String>,
    pub webdav_password_ids: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpSourceSelectionV2 {
    pub name: String,
    pub definition_digest: String,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtensionCredentialsV2 {
    pub skill_names: Vec<String>,
    pub mcp_names: Vec<String>,
}
fn err(code: &str) -> ModelError {
    invalid(code, "选区无法在保留未选内容及原凭据归属的条件下应用")
}
fn model_err(e: schema_v2::MatchErrorV2) -> ModelError {
    err(&e.code)
}
fn set(values: &[String], max: usize) -> ModelResult<BTreeSet<String>> {
    let out: BTreeSet<_> = values.iter().cloned().collect();
    if values.len() > max
        || values.iter().any(|v| v.is_empty() || v.len() > 8192)
        || out.len() != values.len()
    {
        return Err(err("overlay_scope_invalid"));
    }
    Ok(out)
}
fn validate_selection(s: &SelectionV2) -> ModelResult<()> {
    if s.channels.len() > 4096
        || s.template_refs.len() > 4096
        || s.adopt_template_defaults.len() > 4096
        || s.mcp_sources.len() > 1000
    {
        return Err(err("overlay_scope_invalid"));
    }
    let mut channels = BTreeSet::new();
    for c in &s.channels {
        schema_v2::check_id(&c.portable_channel_id).map_err(model_err)?;
        if !channels.insert(&c.portable_channel_id)
            || c.model_ids.is_some() && c.family_ids.is_some()
            || c.include_key && c.include_authorization
        {
            return Err(err("overlay_scope_invalid"));
        }
        if let Some(ids) = &c.model_ids {
            if set(ids, 20000)?.is_empty() {
                return Err(err("overlay_scope_invalid"));
            }
        }
        if let Some(ids) = &c.family_ids {
            if set(ids, 4096)?.is_empty() {
                return Err(err("overlay_scope_invalid"));
            }
            for id in ids {
                schema_v2::check_id(id).map_err(model_err)?;
            }
        }
    }
    let mut refs = BTreeSet::new();
    for r in &s.template_refs {
        if !refs.insert((&r.template_id, &r.revision_id)) {
            return Err(err("overlay_scope_invalid"));
        }
    }
    let mut defaults = BTreeSet::new();
    for r in &s.adopt_template_defaults {
        if !defaults.insert(&r.template_id) {
            return Err(err("overlay_scope_invalid"));
        }
    }
    let skills = set(&s.skill_names, 500)?;
    let mcps = set(
        &s.mcp_sources
            .iter()
            .map(|m| m.name.clone())
            .collect::<Vec<_>>(),
        1000,
    )?;
    for m in &s.mcp_sources {
        if !schema_v2::valid_digest(&m.definition_digest) {
            return Err(err("overlay_scope_invalid"));
        }
    }
    if !set(&s.extension_credentials.skill_names, 500)?.is_subset(&skills)
        || !set(&s.extension_credentials.mcp_names, 1000)?.is_subset(&mcps)
    {
        return Err(err("overlay_scope_invalid"));
    }
    set(&s.manager.webdav_ids, 256)?;
    set(&s.manager.webdav_password_ids, 256)?;
    if s.usage == UsageV2::RemoteSync
        && (!s.manager.webdav_ids.is_empty() || !s.manager.webdav_password_ids.is_empty())
    {
        return Err(err("sync_self_scope"));
    }
    Ok(())
}
fn empty(local: &PortableModelsV2) -> ModelResult<CapturedObjects> {
    let payload = ModelPayloadV2 {
        templates: vec![],
        policy: None,
    };
    Ok(CapturedObjects {
        models: PortableModelsV2 {
            format: portable_v2::PORTABLE_FORMAT.into(),
            format_version: 6,
            schema_version: 2,
            created_at: local.created_at.clone(),
            digest: schema_v2::digest(&schema_v2::canonical_bytes(&payload).map_err(model_err)?),
            payload,
        },
        manager: Default::default(),
        assets: Default::default(),
        channel_keys: SecretValues::new(),
        channel_authorizations: BTreeMap::new(),
        webdav_passwords: SecretValues::new(),
        content_versions: vec![],
    })
}
fn library(models: &PortableModelsV2) -> LibraryV2 {
    LibraryV2 {
        schema_version: 2,
        templates: models.payload.templates.clone(),
        submission_receipts: BTreeMap::new(),
    }
}
fn channel<'a>(models: &'a PortableModelsV2, id: &str) -> ModelResult<&'a PortableChannelV2> {
    models
        .payload
        .policy
        .as_ref()
        .and_then(|p| p.channels.iter().find(|c| c.id == id))
        .ok_or_else(|| err("overlay_selection_missing"))
}
fn selected_models(c: &PortableChannelV2, s: &ChannelSelectionV2) -> ModelResult<BTreeSet<String>> {
    let ids: BTreeSet<_> = c.models.iter().map(|m| m.raw_model_id.clone()).collect();
    match (&s.model_ids, &s.family_ids) {
        (Some(expected), None) => {
            if c.family_scope.is_some()
                || c.model_subset
                    .as_ref()
                    .map(|ids| ids.iter().cloned().collect::<BTreeSet<_>>())
                    != Some(expected.iter().cloned().collect())
                || ids != expected.iter().cloned().collect()
            {
                return Err(err("overlay_channel_scope_conflict"));
            }
        }
        (None, Some(expected)) => {
            if c.model_subset.is_some()
                || c.family_scope
                    .as_ref()
                    .map(|ids| ids.iter().cloned().collect::<BTreeSet<_>>())
                    != Some(expected.iter().cloned().collect())
            {
                return Err(err("overlay_channel_scope_conflict"));
            }
        }
        (None, None) => {
            if c.model_subset.is_some() || c.family_scope.is_some() {
                return Err(err("overlay_channel_scope_conflict"));
            }
        }
        _ => return Err(err("overlay_scope_invalid")),
    }
    Ok(ids)
}
/// Bind the exact captured public MCP definition and credential-path identities.
/// The client-source choice is separately protected by L0's SourceGuard. This
/// never invents a client preference, and does not put secret VALUES into JSON.
pub fn mcp_source_digest(mcp: &Mcp) -> ModelResult<String> {
    portable_assets::validate(&Assets {
        skills: vec![],
        mcps: vec![mcp.clone()],
    })
    .map_err(|e| err(&e.code))?;
    let fields: BTreeSet<_> = mcp.secret_fields.iter().collect();
    Ok(schema_v2::digest(&schema_v2::canonical_bytes(&serde_json::json!({"format":"OCXD_MCP_SOURCE_V2","name":mcp.name,"definition":mcp.definition,"secretFields":fields})).map_err(model_err)?))
}
/// L0 must pass a complete, explicitly chosen head payload as baseline. Other
/// conflict parents are graph identities only; this function cannot accept a
/// collection of heads and cannot resolve their outside-scope changes silently.
pub fn overlay_selected(
    baseline: Option<SavedObjects>,
    local: SavedObjects,
    selection: SelectionV2,
    auth: Option<&dyn AuthorizationValidator>,
) -> ModelResult<SavedObjects> {
    validate_selection(&selection)?;
    let local_atoms = super::file_objects_v2::save_state(&local)?;
    let had_baseline = baseline.is_some();
    let mut local = local.into_captured();
    let mut out = match baseline {
        Some(b) => b.into_captured(),
        None => empty(&local.models)?,
    };
    // Both inputs were wrapper-validated, but use the ACTUAL L1 validator again
    // at the semantic combination boundary. No legacy ModelPackage is involved.
    portable_v2::validate_portable_models_v2(&out.models).map_err(model_err)?;
    portable_v2::validate_portable_models_v2(&local.models).map_err(model_err)?;
    let mut required = selection.template_refs.clone();
    required.extend(selection.adopt_template_defaults.clone());
    let local_policy = local.models.payload.policy.clone();
    let mut policy = out
        .models
        .payload
        .policy
        .clone()
        .unwrap_or(PortablePolicyV2 {
            template_refs: vec![],
            channels: vec![],
            channel_order: vec![],
            default_channel_id: None,
        });
    let mut changed_channel_ids = BTreeSet::new();
    for scope in &selection.channels {
        let incoming = channel(&local.models, &scope.portable_channel_id)?.clone();
        let model_ids = selected_models(&incoming, scope)?;
        let old = policy
            .channels
            .iter()
            .position(|c| c.id == scope.portable_channel_id);
        let previous = old.map(|index| policy.channels[index].clone());
        if scope.include_connection && incoming.connection.is_none() {
            return Err(err("overlay_selection_missing"));
        }
        if previous.is_none() && !scope.include_connection && scope.family_ids.is_none() {
            return Err(err("overlay_connection_required"));
        }
        let merged = merge_channel(previous.as_ref(), &incoming, scope, &model_ids)?;
        if (scope.include_key || scope.include_authorization)
            && incoming.connection != merged.connection
        {
            return Err(err("overlay_unselected_credential_conflict"));
        }
        if previous
            .as_ref()
            .is_some_and(|p| p.connection != merged.connection)
        {
            let has_key = out.channel_keys.get(&scope.portable_channel_id).is_some();
            let has_auth = out
                .channel_authorizations
                .contains_key(&scope.portable_channel_id);
            if has_key && !scope.include_key || has_auth && !scope.include_authorization {
                return Err(err("overlay_unselected_credential_conflict"));
            }
        }
        if scope.include_key
            && out
                .channel_authorizations
                .contains_key(&scope.portable_channel_id)
            || scope.include_authorization
                && out.channel_keys.get(&scope.portable_channel_id).is_some()
        {
            return Err(err("overlay_credential_conflict"));
        }
        if scope.include_key && local.channel_keys.get(&scope.portable_channel_id).is_none() {
            return Err(err("overlay_selection_missing"));
        }
        if scope.include_authorization {
            let value = local
                .channel_authorizations
                .remove(&scope.portable_channel_id)
                .ok_or_else(|| err("overlay_selection_missing"))?;
            out.channel_authorizations
                .insert(scope.portable_channel_id.clone(), value);
        }
        // Dependencies are the references actually written into the selected
        // result, plus explicit library/global/default choices. An unselected
        // local template never becomes an implicit import.
        let local_refs = references_of(&incoming);
        if scope.model_ids.is_none() {
            required.extend(local_refs);
        } else {
            required.extend(
                incoming
                    .models
                    .iter()
                    .filter(|m| model_ids.contains(&m.raw_model_id))
                    .filter_map(|m| m.binding.as_ref().map(|b| b.reference().clone())),
            );
        }
        if let Some(index) = old {
            policy.channels[index] = merged;
        } else {
            policy.channels.push(merged);
        }
        changed_channel_ids.insert(scope.portable_channel_id.clone());
    }
    let key_ids: BTreeSet<_> = selection
        .channels
        .iter()
        .filter(|c| c.include_key)
        .map(|c| c.portable_channel_id.clone())
        .collect();
    out.channel_keys = overlay_secret_values(&out.channel_keys, &local.channel_keys, &key_ids)?;
    if selection.include_global_template_refs {
        let source = local_policy
            .as_ref()
            .ok_or_else(|| err("overlay_selection_missing"))?;
        merge_refs(&mut policy.template_refs, &source.template_refs)?;
        required.extend(source.template_refs.clone());
    }
    if let Some(source) = &local_policy {
        policy.channel_order = reorder_selected(
            &policy.channel_order,
            &source.channel_order,
            &changed_channel_ids,
            policy.channels.iter().map(|c| c.id.clone()),
        );
        if !had_baseline {
            policy.default_channel_id = source
                .default_channel_id
                .as_ref()
                .filter(|id| changed_channel_ids.contains(*id))
                .cloned();
        }
    }
    check_channel_names(&policy)?;
    if !selection.channels.is_empty()
        || selection.include_global_template_refs
        || out.models.payload.policy.is_some()
    {
        out.models.payload.policy = Some(policy);
    }
    merge_templates(
        &mut out.models,
        &local.models,
        &required,
        &selection.adopt_template_defaults,
    )?;
    overlay_manager(&mut out, &local, &selection.manager)?;
    overlay_assets(&mut out.assets, &mut local.assets, &selection)?;
    // Opaque save-event scope keys cannot be rebound here without the content
    // fact owner's identity contract. Preserve baseline evidence, do not mint or
    // silently inherit local timestamps for a newly composed document.
    out.models.digest =
        schema_v2::digest(&schema_v2::canonical_bytes(&out.models.payload).map_err(model_err)?);
    portable_v2::validate_portable_models_v2(&out.models).map_err(model_err)?;
    let result = SavedObjects::capture(out, auth)?;
    let result = inherit_receipts(
        result,
        &local.content_versions,
        &local_atoms,
        &selection,
        &required,
        auth,
    )?;
    if selection.usage == UsageV2::RemoteSync {
        result.validate_remote_sync_scope()?;
    }
    Ok(result)
}
fn references_of(channel: &PortableChannelV2) -> Vec<RefV2> {
    channel
        .template_refs
        .iter()
        .cloned()
        .chain(
            channel
                .models
                .iter()
                .filter_map(|m| m.binding.as_ref().map(|b| b.reference().clone())),
        )
        .collect()
}
fn merge_refs(base: &mut Vec<RefV2>, incoming: &[RefV2]) -> ModelResult<()> {
    let mut seen = BTreeSet::new();
    for r in incoming {
        if !seen.insert(&r.template_id) {
            return Err(err("overlay_scope_invalid"));
        }
        if let Some(old) = base.iter_mut().find(|old| old.template_id == r.template_id) {
            *old = r.clone();
        } else {
            base.push(r.clone());
        }
    }
    Ok(())
}
fn reorder_selected(
    base: &[String],
    source: &[String],
    selected: &BTreeSet<String>,
    available: impl Iterator<Item = String>,
) -> Vec<String> {
    let available: Vec<_> = available.collect();
    let avail: BTreeSet<_> = available.iter().cloned().collect();
    let existing: BTreeSet<_> = base.iter().cloned().collect();
    let mut preferred: Vec<_> = source
        .iter()
        .filter(|id| selected.contains(*id) && existing.contains(*id) && avail.contains(*id))
        .cloned()
        .collect();
    for id in base
        .iter()
        .filter(|id| selected.contains(*id) && avail.contains(*id))
    {
        if !preferred.contains(id) {
            preferred.push(id.clone());
        }
    }
    let mut replacements = preferred.into_iter();
    let mut out = Vec::new();
    for id in base {
        if !avail.contains(id) {
            continue;
        }
        if selected.contains(id) {
            if let Some(next) = replacements.next() {
                out.push(next);
            }
        } else {
            out.push(id.clone());
        }
    }
    for id in source.iter().chain(&available) {
        if avail.contains(id) && !out.contains(id) {
            out.push(id.clone());
        }
    }
    out
}
fn merge_selection(
    base: &ModelSelection,
    incoming: &ModelSelection,
    ids: &BTreeSet<String>,
    all: &BTreeSet<String>,
    whole: bool,
) -> ModelResult<ModelSelection> {
    fn chosen(s: &ModelSelection, id: &str) -> bool {
        match s {
            ModelSelection::All => true,
            ModelSelection::None => false,
            ModelSelection::Whitelist(ids) => ids.iter().any(|i| i == id),
        }
    }
    if matches!(base, ModelSelection::All) {
        if ids.iter().any(|id| !chosen(incoming, id)) {
            return Err(err("overlay_model_selection_conflict"));
        }
        return Ok(ModelSelection::All);
    }
    if whole && ids == all && matches!(incoming, ModelSelection::All) {
        return Ok(ModelSelection::All);
    }
    let mut selected = match base {
        ModelSelection::Whitelist(v) => v.clone(),
        _ => vec![],
    };
    for id in ids {
        if chosen(incoming, id) {
            if !selected.contains(id) {
                selected.push(id.clone());
            }
        } else {
            selected.retain(|old| old != id);
        }
    }
    if selected.is_empty() {
        Ok(ModelSelection::None)
    } else {
        Ok(ModelSelection::Whitelist(selected))
    }
}
fn merge_channel(
    base: Option<&PortableChannelV2>,
    source: &PortableChannelV2,
    scope: &ChannelSelectionV2,
    ids: &BTreeSet<String>,
) -> ModelResult<PortableChannelV2> {
    let whole = scope.model_ids.is_none() && scope.family_ids.is_none();
    let Some(base) = base else {
        let mut out = source.clone();
        if !scope.include_connection {
            out.connection = None;
        }
        if !whole && matches!(out.selection, ModelSelection::All) {
            out.selection = ModelSelection::Whitelist(ids.iter().cloned().collect());
        }
        return Ok(out);
    };
    let mut out = base.clone();
    if scope.model_ids.is_some() {
        if base.family_scope.is_some() || source.defaults != base.defaults {
            return Err(err("overlay_shared_rules_conflict"));
        }
        for model in &source.models {
            if let Some(binding) = &model.binding {
                let family = &binding.reference().template_id;
                if source.family_overrides.get(family) != base.family_overrides.get(family) {
                    return Err(err("overlay_shared_rules_conflict"));
                }
            }
        }
    }
    if let Some(families) = &scope.family_ids {
        // L1 family exports deliberately have empty channel defaults. They
        // have no authority to reset the existing channel-wide layer.
        let selected: BTreeSet<_> = families.iter().collect();
        for (id, rules) in &source.family_overrides {
            if selected.contains(id) {
                out.family_overrides.insert(id.clone(), rules.clone());
            }
        }
        let refs: Vec<_> = source
            .template_refs
            .iter()
            .filter(|r| selected.contains(&r.template_id))
            .cloned()
            .collect();
        merge_refs(&mut out.template_refs, &refs)?;
        if let Some(old_families) = &mut out.family_scope {
            for id in families {
                if !old_families.contains(id) {
                    old_families.push(id.clone());
                }
            }
        }
    }
    if whole {
        out.label = source.label.clone();
        out.namespace = source.namespace.clone();
        out.defaults = source.defaults.clone();
        for (id, rules) in &source.family_overrides {
            out.family_overrides.insert(id.clone(), rules.clone());
        }
        merge_refs(&mut out.template_refs, &source.template_refs)?;
        if base.family_scope.is_some() && !scope.include_connection {
            return Err(err("overlay_connection_required"));
        }
        if scope.include_connection {
            out.family_scope = None;
            out.model_subset = None;
        }
    }
    if scope.include_connection {
        out.connection = source.connection.clone();
    }
    for model in &source.models {
        if !ids.contains(&model.raw_model_id) {
            return Err(err("overlay_channel_scope_conflict"));
        }
        if let Some(old) = out
            .models
            .iter_mut()
            .find(|old| old.raw_model_id == model.raw_model_id)
        {
            *old = model.clone();
        } else {
            out.models.push(model.clone());
        }
        if let Some(name) = source.naming.get(&model.raw_model_id) {
            out.naming.insert(model.raw_model_id.clone(), name.clone());
        }
    }
    let all: BTreeSet<_> = out.models.iter().map(|m| m.raw_model_id.clone()).collect();
    if let Some(subset) = &mut out.model_subset {
        *subset = all.iter().cloned().collect();
    }
    out.selection = merge_selection(&base.selection, &source.selection, ids, &all, whole)?;
    out.model_order = reorder_selected(
        &base.model_order,
        &source.model_order,
        ids,
        out.models.iter().map(|m| m.raw_model_id.clone()),
    );
    Ok(out)
}
fn merge_templates(
    out: &mut PortableModelsV2,
    local: &PortableModelsV2,
    required: &[RefV2],
    adopt: &[RefV2],
) -> ModelResult<()> {
    let local_library = library(local);
    let base = library(out);
    let mut wanted: BTreeMap<String, BTreeMap<String, RefV2>> = BTreeMap::new();
    for r in required {
        schema_v2::revision(&local_library, r).map_err(model_err)?;
        if let Some(old) = wanted
            .entry(r.template_id.clone())
            .or_default()
            .insert(r.revision_id.clone(), r.clone())
        {
            if old.digest != r.digest {
                return Err(err("model_revision_digest_mismatch"));
            }
        }
    }
    let incoming: Vec<_> = wanted
        .iter()
        .map(|(id, refs)| {
            let source = local_library
                .templates
                .iter()
                .find(|t| t.id == *id)
                .expect("revision validated");
            TemplateV2 {
                id: source.id.clone(),
                label: source.label.clone(),
                source: source.source.clone(),
                default_revision: None,
                revisions: source
                    .revisions
                    .iter()
                    .filter(|r| refs.contains_key(&r.id))
                    .cloned()
                    .collect(),
            }
        })
        .collect();
    let mut merged = portable_v2::merge_library_v2(
        &base,
        &LibraryV2 {
            schema_version: 2,
            templates: incoming,
            submission_receipts: BTreeMap::new(),
        },
    )
    .map_err(model_err)?;
    for r in adopt {
        let source = local_library
            .templates
            .iter()
            .find(|t| t.id == r.template_id)
            .ok_or_else(|| err("overlay_selection_missing"))?;
        if source.default_revision.as_ref() != Some(&r.revision_id) {
            return Err(err("overlay_default_source_changed"));
        }
        merged
            .templates
            .iter_mut()
            .find(|t| t.id == r.template_id)
            .ok_or_else(|| err("overlay_selection_missing"))?
            .default_revision = Some(r.revision_id.clone());
    }
    // Family-level defaults may require a template identity without an executable
    // revision reference. Import its identity only; never invent a latest/default.
    let needed_ids: BTreeSet<_> = out
        .payload
        .policy
        .iter()
        .flat_map(|p| &p.channels)
        .flat_map(|c| {
            c.family_overrides
                .keys()
                .chain(c.family_scope.iter().flatten())
        })
        .cloned()
        .collect();
    for id in needed_ids {
        if !merged.templates.iter().any(|t| t.id == id) {
            let source = local_library
                .templates
                .iter()
                .find(|t| t.id == id)
                .ok_or_else(|| err("model_dependency_missing"))?;
            merged.templates.push(TemplateV2 {
                id: source.id.clone(),
                label: source.label.clone(),
                source: source.source.clone(),
                default_revision: None,
                revisions: vec![],
            });
        }
    }
    schema_v2::validate_library_v2(&merged).map_err(model_err)?;
    out.payload.templates = merged.templates;
    Ok(())
}
fn overlay_secret_values(
    base: &SecretValues,
    local: &SecretValues,
    selected: &BTreeSet<String>,
) -> ModelResult<SecretValues> {
    let mut out = SecretValues::new();
    for id in base.ids().filter(|id| !selected.contains(*id)) {
        out.insert(
            id.into(),
            Zeroizing::new(base.get(id).expect("known id").to_owned()),
        )?;
    }
    for id in selected {
        let value = local
            .get(id)
            .ok_or_else(|| err("overlay_selection_missing"))?;
        out.insert(id.clone(), Zeroizing::new(value.to_owned()))?;
    }
    Ok(out)
}
fn check_channel_names(policy: &PortablePolicyV2) -> ModelResult<()> {
    let mut names = BTreeSet::new();
    for c in &policy.channels {
        if !names.insert(c.label.trim().to_lowercase()) {
            return Err(err("overlay_channel_name_conflict"));
        }
    }
    Ok(())
}
fn endpoint_binding_eq(a: &portable_manager::Endpoint, b: &portable_manager::Endpoint) -> bool {
    a.url == b.url && a.remote_path == b.remote_path && a.username == b.username
}
fn overlay_manager(
    out: &mut CapturedObjects,
    local: &CapturedObjects,
    scope: &ManagerSelectionV2,
) -> ModelResult<()> {
    if scope.preferences {
        out.manager.preferences = Some(
            local
                .manager
                .preferences
                .clone()
                .ok_or_else(|| err("overlay_selection_missing"))?,
        );
    }
    if scope.extensions {
        out.manager.extensions = Some(
            local
                .manager
                .extensions
                .clone()
                .ok_or_else(|| err("overlay_selection_missing"))?,
        );
    }
    for id in &scope.webdav_ids {
        let incoming = local
            .manager
            .webdav
            .iter()
            .find(|e| e.id == *id)
            .ok_or_else(|| err("overlay_selection_missing"))?;
        if let Some(old) = out.manager.webdav.iter_mut().find(|e| e.id == *id) {
            if !endpoint_binding_eq(old, incoming)
                && out.webdav_passwords.get(id).is_some()
                && !scope.webdav_password_ids.contains(id)
            {
                return Err(err("overlay_unselected_credential_conflict"));
            }
            *old = incoming.clone();
        } else {
            out.manager.webdav.push(incoming.clone());
        }
    }
    for id in &scope.webdav_password_ids {
        let source = local
            .manager
            .webdav
            .iter()
            .find(|e| e.id == *id)
            .ok_or_else(|| err("overlay_selection_missing"))?;
        let target = out
            .manager
            .webdav
            .iter()
            .find(|e| e.id == *id)
            .ok_or_else(|| err("overlay_selection_missing"))?;
        if !endpoint_binding_eq(source, target) {
            return Err(err("overlay_unselected_credential_conflict"));
        }
    }
    out.webdav_passwords = overlay_secret_values(
        &out.webdav_passwords,
        &local.webdav_passwords,
        &scope.webdav_password_ids.iter().cloned().collect(),
    )?;
    Ok(())
}
fn overlay_assets(out: &mut Assets, local: &mut Assets, scope: &SelectionV2) -> ModelResult<()> {
    for name in &scope.skill_names {
        let source = local
            .skills
            .iter()
            .position(|s| s.name == *name)
            .ok_or_else(|| err("overlay_selection_missing"))?;
        let mut incoming = local.skills.remove(source);
        if out
            .skills
            .iter()
            .any(|s| s.name != *name && s.name.to_lowercase() == name.to_lowercase())
        {
            return Err(err("overlay_asset_identity_conflict"));
        }
        let pos = out.skills.iter().position(|s| s.name == *name);
        let base = pos.map(|i| out.skills.remove(i));
        merge_skill(
            base.as_ref(),
            &mut incoming,
            scope.extension_credentials.skill_names.contains(name),
        )?;
        if let Some(pos) = pos {
            out.skills.insert(pos, incoming);
        } else {
            out.skills.push(incoming);
        }
    }
    for selected in &scope.mcp_sources {
        let index = local
            .mcps
            .iter()
            .position(|m| m.name == selected.name)
            .ok_or_else(|| err("overlay_selection_missing"))?;
        let mut incoming = local.mcps.remove(index);
        if mcp_source_digest(&incoming)? != selected.definition_digest {
            return Err(err("overlay_mcp_source_changed"));
        }
        if out.mcps.iter().any(|m| {
            m.name != selected.name && m.name.to_lowercase() == selected.name.to_lowercase()
        }) {
            return Err(err("overlay_asset_identity_conflict"));
        }
        let old = out.mcps.iter().position(|m| m.name == selected.name);
        let mut base = old.map(|i| out.mcps.remove(i));
        let include_credentials = scope
            .extension_credentials
            .mcp_names
            .contains(&selected.name);
        if include_credentials && incoming.secrets.0.is_empty() {
            return Err(err("overlay_selection_missing"));
        }
        if let Some(base) = &mut base {
            let changed = mcp_source_digest(base)? != selected.definition_digest;
            if base.secrets.0.keys().any(|path| {
                !incoming.secret_fields.contains(path)
                    || (changed && (!include_credentials || !incoming.secrets.0.contains_key(path)))
            }) {
                return Err(err("overlay_unselected_credential_conflict"));
            }
            if include_credentials {
                for (path, value) in &base.secrets.0 {
                    if !incoming.secrets.0.contains_key(path) {
                        incoming.secrets.0.insert(path.clone(), value.clone());
                    }
                }
            } else {
                incoming.secrets = std::mem::take(&mut base.secrets);
            }
        } else if !include_credentials {
            incoming.secrets = Default::default();
        }
        if let Some(pos) = old {
            out.mcps.insert(pos, incoming);
        } else {
            out.mcps.push(incoming);
        }
    }
    portable_assets::validate(out).map_err(|e| {
        err(if e.code == "asset_identity" {
            "overlay_asset_identity_conflict"
        } else {
            &e.code
        })
    })?;
    Ok(())
}
fn merge_skill(
    base: Option<&Skill>,
    incoming: &mut Skill,
    include_credentials: bool,
) -> ModelResult<()> {
    if include_credentials && !incoming.contains_secrets {
        return Err(err("overlay_selection_missing"));
    }
    let private: BTreeMap<_, _> = base
        .into_iter()
        .flat_map(|s| &s.files)
        .filter(|f| f.sensitive)
        .map(|f| (f.path.to_lowercase(), f))
        .collect();
    let files = std::mem::take(&mut incoming.files);
    let mut out = Vec::new();
    for file in files {
        if !include_credentials {
            if file.sensitive {
                continue;
            }
            if private.contains_key(&file.path.to_lowercase()) {
                return Err(err("overlay_unselected_credential_conflict"));
            }
        }
        out.push(file);
    }
    for (path, file) in private {
        if !out.iter().any(|f| f.path.to_lowercase() == path) {
            out.push(file.clone());
        }
    }
    if !out.iter().any(|f| f.path == "SKILL.md") {
        return Err(err("overlay_asset_credential_required"));
    }
    incoming.contains_secrets = out.iter().any(|f| f.sensitive);
    incoming.digest = schema_v2::digest(&Zeroizing::new(
        serde_json::to_vec(&out).map_err(|_| err("overlay_asset_identity_conflict"))?,
    ));
    incoming.files = out;
    Ok(())
}
fn inherit_receipts(
    result: SavedObjects,
    local: &[super::content_versions::Version],
    local_atoms: &super::content_versions::SaveState,
    s: &SelectionV2,
    required: &[RefV2],
    auth: Option<&dyn AuthorizationValidator>,
) -> ModelResult<SavedObjects> {
    let atoms = super::file_objects_v2::save_state(&result)?;
    let mut eligible = BTreeSet::new();
    let mut allow = |identity: serde_json::Value| -> ModelResult<()> {
        eligible.insert(super::content_versions::key(&identity).map_err(|e| err(&e.code))?);
        Ok(())
    };
    for scope in &s.channels {
        let c = channel(result.models(), &scope.portable_channel_id)?;
        allow(serde_json::json!([
            "v2-channel",
            c.id,
            c.model_subset,
            c.family_scope
        ]))?;
        if scope.include_key {
            allow(serde_json::json!([
                "v2-channelKeys",
                c.id,
                c.model_subset,
                c.family_scope
            ]))?;
        }
        if scope.include_authorization {
            allow(serde_json::json!([
                "v2-channelAuthorizations",
                c.id,
                c.model_subset,
                c.family_scope
            ]))?;
        }
    }
    for r in required {
        allow(serde_json::json!([
            "v2-template",
            r.template_id,
            r.revision_id
        ]))?;
    }
    if !s.channels.is_empty() || s.include_global_template_refs {
        allow(serde_json::json!(["v2-global"]))?;
    }
    if s.manager.preferences {
        allow(serde_json::json!(["v2-preferences"]))?;
    }
    if s.manager.extensions {
        allow(serde_json::json!(["v2-extensions"]))?;
    }
    for id in &s.manager.webdav_ids {
        allow(serde_json::json!(["v2-webdav", id]))?;
    }
    for id in &s.manager.webdav_password_ids {
        allow(serde_json::json!(["v2-webdavPasswords", id]))?;
    }
    for name in &s.skill_names {
        allow(serde_json::json!(["v2-skills", name]))?;
    }
    for mcp in &s.mcp_sources {
        allow(serde_json::json!(["v2-mcps", mcp.name]))?;
    }
    for name in &s.extension_credentials.mcp_names {
        allow(serde_json::json!(["v2-mcpSecrets", name]))?;
    }
    let matched: Vec<_> = local
        .iter()
        .filter(|v| {
            eligible.contains(&v.scope_key)
                && atoms.get(&v.scope_key) == Some(&v.content_digest)
                && local_atoms.get(&v.scope_key) == Some(&v.content_digest)
        })
        .collect();
    if matched.is_empty() {
        return Ok(result);
    }
    let mut out = result.into_captured();
    for v in matched {
        if let Some(old) = out
            .content_versions
            .iter_mut()
            .find(|old| old.scope_key == v.scope_key)
        {
            // Re-receiving already known content must not replace its existing
            // save event with a later transport/device clock value.
            if old.content_digest == v.content_digest
                && old.time_kind == super::content_versions::TimeKind::Saved
            {
                continue;
            }
            *old = v.clone();
        } else {
            out.content_versions.push(v.clone());
        }
    }
    SavedObjects::capture(out, auth)
}
#[cfg(test)]
#[path = "objects_overlay_v2_tests.rs"]
mod tests;
