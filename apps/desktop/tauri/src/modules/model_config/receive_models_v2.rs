//! Pure v6 receive planning. L0 allocates identities and owns credentials,
//! activation, source reads, runtime mappings and the configuration transaction.
use super::{
    matching_v2::resolve_effective_model_v2,
    objects_overlay_v2::{ChannelSelectionV2, SelectionV2},
    portable_v2::{self, PortableChannelV2, PortableModelsV2},
    schema::{ModelRules, ModelSelection},
    schema_v2::*,
    source_v2::validate_policy_v2,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiveBindingModeV2 {
    New,
    Existing,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiveBindingV2 {
    pub portable_channel_id: String,
    pub local_channel_id: String,
    pub provider_id: String,
    pub mode: ReceiveBindingModeV2,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiveCredentialKindV2 {
    Key,
    Authorization,
}
/// Selection intent only. L0 validates the outer material, binds a fresh local
/// credential and authorizes its actual protocols before applying.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiveCredentialActionV2 {
    pub portable_channel_id: String,
    pub local_channel_id: String,
    pub provider_id: String,
    pub kind: ReceiveCredentialKindV2,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiveModelsPlanV2 {
    pub library: LibraryV2,
    pub policy: PolicyV2,
    /// Selected connection objects or constrained model attachments, keyed by
    /// local channel. L0 overlays only selected entries, never replaces an
    /// entire existing provider with a partial attachment. Shared execution
    /// flags are source evidence, not auth/activation or final protocol intent.
    pub public_sources: BTreeMap<String, Value>,
    pub source_to_target: BTreeMap<String, String>,
    pub credential_actions: Vec<ReceiveCredentialActionV2>,
    pub changed_channel_ids: Vec<String>,
    pub changed_template_refs: Vec<RefV2>,
}
fn scope_error() -> MatchErrorV2 {
    error("model_receive_scope_invalid")
}
fn unique(values: &[String]) -> Result<BTreeSet<String>> {
    let ids: BTreeSet<_> = values.iter().cloned().collect();
    if ids.is_empty() || ids.len() != values.len() || values.iter().any(|s| s.is_empty()) {
        return Err(scope_error());
    }
    Ok(ids)
}
fn selected_ids(c: &PortableChannelV2, s: &ChannelSelectionV2) -> Result<BTreeSet<String>> {
    let all: BTreeSet<_> = c.models.iter().map(|m| m.raw_model_id.clone()).collect();
    match (&s.model_ids, &s.family_ids) {
        (Some(ids), None) => {
            let ids = unique(ids)?;
            if c.family_scope.is_some() || !ids.is_subset(&all) {
                return Err(scope_error());
            }
            Ok(ids)
        }
        (None, Some(families)) => {
            let families = unique(families)?;
            if c.model_subset.is_some() || s.include_connection {
                return Err(scope_error());
            }
            let available: BTreeSet<_> = c
                .family_scope
                .clone()
                .unwrap_or_else(|| {
                    c.family_overrides
                        .keys()
                        .cloned()
                        .chain(c.template_refs.iter().map(|r| r.template_id.clone()))
                        .chain(c.models.iter().filter_map(|m| {
                            m.binding
                                .as_ref()
                                .map(|b| b.reference().template_id.clone())
                        }))
                        .collect()
                })
                .into_iter()
                .collect();
            for id in &families {
                check_id(id)?;
            }
            if !families.is_subset(&available) {
                return Err(scope_error());
            }
            Ok(c.models
                .iter()
                .filter(|m| {
                    m.binding
                        .as_ref()
                        .is_some_and(|b| families.contains(&b.reference().template_id))
                })
                .map(|m| m.raw_model_id.clone())
                .collect())
        }
        (None, None) if c.model_subset.is_none() && c.family_scope.is_none() => Ok(all),
        _ => Err(scope_error()),
    }
}
fn validate_dependencies(library: &LibraryV2, policy: &PolicyV2) -> Result<()> {
    validate_policy_v2(policy)?;
    for r in policy
        .template_refs
        .iter()
        .chain(policy.channels.iter().flat_map(|c| &c.template_refs))
    {
        revision(library, r)?;
    }
    for c in &policy.channels {
        for id in c.family_overrides.keys() {
            if !library.templates.iter().any(|t| &t.id == id) {
                return Err(error("model_dependency_missing"));
            }
        }
        for m in &c.models {
            resolve_effective_model_v2(library, c, m)?;
        }
    }
    Ok(())
}
/// Import selected immutable revisions and family identities only. All package
/// revisions were already validated; a saved default is never a latest lookup.
fn selected_library(package: &PortableModelsV2, selected: &SelectionV2) -> Result<LibraryV2> {
    let full = LibraryV2 {
        schema_version: SCHEMA_VERSION,
        templates: package.payload.templates.clone(),
        submission_receipts: BTreeMap::new(),
    };
    // L1's portable validator checks executable refs. Check rule-only family
    // dependencies too, including those outside the receive selection.
    for c in package.payload.policy.iter().flat_map(|p| &p.channels) {
        for id in c.family_overrides.keys() {
            check_id(id)?;
            if !full.templates.iter().any(|t| &t.id == id) {
                return Err(error("model_dependency_missing"));
            }
        }
    }
    let mut required: BTreeSet<RefV2> = selected
        .template_refs
        .iter()
        .chain(&selected.adopt_template_defaults)
        .cloned()
        .collect();
    let mut families = BTreeSet::new();
    if selected.include_global_template_refs {
        let p = package.payload.policy.as_ref().ok_or_else(scope_error)?;
        required.extend(p.template_refs.iter().cloned());
    }
    for scope in &selected.channels {
        let c = package
            .payload
            .policy
            .as_ref()
            .and_then(|p| {
                p.channels
                    .iter()
                    .find(|c| c.id == scope.portable_channel_id)
            })
            .ok_or_else(scope_error)?;
        let ids = selected_ids(c, scope)?;
        required.extend(
            c.models
                .iter()
                .filter(|m| ids.contains(&m.raw_model_id))
                .filter_map(|m| m.binding.as_ref().map(|b| b.reference().clone())),
        );
        if let Some(ids) = &scope.family_ids {
            families.extend(ids.iter().cloned());
            required.extend(
                c.template_refs
                    .iter()
                    .filter(|r| ids.contains(&r.template_id))
                    .cloned(),
            );
        } else if scope.model_ids.is_none() {
            families.extend(c.family_overrides.keys().cloned());
            required.extend(c.template_refs.iter().cloned());
        } else {
            let bound: BTreeSet<_> = c
                .models
                .iter()
                .filter(|m| ids.contains(&m.raw_model_id))
                .filter_map(|m| m.binding.as_ref().map(|b| &b.reference().template_id))
                .collect();
            required.extend(
                c.template_refs
                    .iter()
                    .filter(|r| bound.contains(&r.template_id))
                    .cloned(),
            );
        }
    }
    for r in &required {
        revision(&full, r)?;
        families.insert(r.template_id.clone());
    }
    Ok(LibraryV2 {
        schema_version: SCHEMA_VERSION,
        templates: full
            .templates
            .iter()
            .filter(|t| families.contains(&t.id))
            .map(|t| TemplateV2 {
                id: t.id.clone(),
                label: t.label.clone(),
                source: t.source.clone(),
                default_revision: None,
                revisions: t
                    .revisions
                    .iter()
                    .filter(|r| {
                        required
                            .iter()
                            .any(|wanted| wanted.template_id == t.id && wanted.revision_id == r.id)
                    })
                    .cloned()
                    .collect(),
            })
            .collect(),
        submission_receipts: BTreeMap::new(),
    })
}
fn merge_refs(base: &mut Vec<RefV2>, incoming: impl Iterator<Item = RefV2>) {
    for r in incoming {
        if let Some(old) = base.iter_mut().find(|old| old.template_id == r.template_id) {
            *old = r;
        } else {
            base.push(r);
        }
    }
}
// Same selected-slot reorder as objects_overlay_v2: unselected slots survive.
fn reorder(
    base: &[String],
    source: &[String],
    ids: &BTreeSet<String>,
    available: &[String],
) -> Vec<String> {
    let mut preferred: Vec<_> = source
        .iter()
        .filter(|id| ids.contains(*id) && base.contains(*id))
        .cloned()
        .collect();
    for id in base.iter().filter(|id| ids.contains(*id)) {
        if !preferred.contains(id) {
            preferred.push(id.clone());
        }
    }
    let mut replacements = preferred.into_iter();
    let mut out = vec![];
    for id in base {
        if ids.contains(id) {
            if let Some(id) = replacements.next() {
                out.push(id);
            }
        } else {
            out.push(id.clone());
        }
    }
    for id in source.iter().chain(available) {
        if ids.contains(id) && !out.contains(id) {
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
) -> Result<ModelSelection> {
    let chosen = |id: &String| match incoming {
        ModelSelection::All => true,
        ModelSelection::None => false,
        ModelSelection::Whitelist(v) => v.contains(id),
    };
    if matches!(base, ModelSelection::All) {
        if ids.iter().any(|id| !chosen(id)) {
            return Err(error("overlay_model_selection_conflict"));
        }
        return Ok(ModelSelection::All);
    }
    if whole && ids == all && matches!(incoming, ModelSelection::All) {
        return Ok(ModelSelection::All);
    }
    let mut out = match base {
        ModelSelection::Whitelist(v) => v.clone(),
        _ => vec![],
    };
    for id in ids {
        if chosen(id) {
            if !out.contains(id) {
                out.push(id.clone());
            }
        } else {
            out.retain(|old| old != id);
        }
    }
    Ok(if out.is_empty() {
        ModelSelection::None
    } else {
        ModelSelection::Whitelist(out)
    })
}
fn empty_channel(binding: &ReceiveBindingV2, label: &str) -> ChannelV2 {
    ChannelV2 {
        id: binding.local_channel_id.clone(),
        label: label.into(),
        connection: ConnectionV2 {
            provider_id: binding.provider_id.clone(),
            nodes: vec![],
            active_node_id: String::new(),
            credential_ref: None,
            enabled: false,
            allow_private_network: false,
            auth_mode: None,
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
    }
}
/// No IO, clock, UUID allocation, credential access or config write bytes.
pub fn plan(
    local_library: &LibraryV2,
    local_policy: &PolicyV2,
    package: &PortableModelsV2,
    selected: &SelectionV2,
    bindings: &[ReceiveBindingV2],
) -> Result<ReceiveModelsPlanV2> {
    plan_with_namespace_adoption(
        local_library,
        local_policy,
        package,
        selected,
        bindings,
        &[],
    )
}
/// L0 supplies only the explicitly adopted portable IDs. Prefixes and namespace
/// contents remain the validated package's evidence, not new caller policy.
pub fn plan_with_namespace_adoption(
    local_library: &LibraryV2,
    local_policy: &PolicyV2,
    package: &PortableModelsV2,
    selected: &SelectionV2,
    bindings: &[ReceiveBindingV2],
    adopt_ids: &[String],
) -> Result<ReceiveModelsPlanV2> {
    portable_v2::validate_portable_models_v2(package)?;
    validate_library_v2(local_library)?;
    validate_dependencies(local_library, local_policy)?;
    let adopted: BTreeSet<_> = adopt_ids.iter().collect();
    if adopted.len() != adopt_ids.len() {
        return Err(scope_error());
    }
    let namespaced: BTreeSet<_> = package
        .payload
        .policy
        .iter()
        .flat_map(|p| &p.channels)
        .filter(|c| c.namespace.is_some())
        .map(|c| &c.id)
        .collect();
    if !namespaced.is_subset(&adopted) {
        return Err(error("model_receive_namespace_adoption_required"));
    }
    if namespaced != adopted {
        return Err(scope_error());
    }
    for id in &adopted {
        check_id(id)?;
        let source = package
            .payload
            .policy
            .iter()
            .flat_map(|p| &p.channels)
            .find(|c| &c.id == *id)
            .unwrap();
        let scope = selected
            .channels
            .iter()
            .find(|s| &s.portable_channel_id == *id)
            .ok_or_else(scope_error)?;
        if scope.model_ids.is_some()
            || scope.family_ids.is_some()
            || source.model_subset.is_some()
            || source.family_scope.is_some()
        {
            return Err(scope_error());
        }
    }
    if selected.channels.len() > 4096
        || selected.template_refs.len() > 4096
        || selected.adopt_template_defaults.len() > 4096
    {
        return Err(error("model_limit"));
    }
    let incoming = LibraryV2 {
        schema_version: SCHEMA_VERSION,
        templates: package.payload.templates.clone(),
        submission_receipts: BTreeMap::new(),
    };
    let scoped_library = selected_library(package, selected)?;
    let mut library = portable_v2::merge_library_v2(local_library, &scoped_library)?;
    // Imported defaults remain evidence until adopted, including new templates.
    for t in &mut library.templates {
        if !local_library.templates.iter().any(|old| old.id == t.id) {
            t.default_revision = None;
        }
    }
    let mut refs = BTreeSet::new();
    for r in &selected.template_refs {
        revision(&incoming, r)?;
        if !refs.insert((&r.template_id, &r.revision_id)) {
            return Err(scope_error());
        }
    }
    let mut defaults = BTreeSet::new();
    for r in &selected.adopt_template_defaults {
        revision(&incoming, r)?;
        let t = incoming
            .templates
            .iter()
            .find(|t| t.id == r.template_id)
            .unwrap();
        if !defaults.insert(&r.template_id) || t.default_revision.as_ref() != Some(&r.revision_id) {
            return Err(error("overlay_default_source_changed"));
        }
        library
            .templates
            .iter_mut()
            .find(|t| t.id == r.template_id)
            .unwrap()
            .default_revision = Some(r.revision_id.clone());
    }
    let mut mapping = BTreeMap::new();
    let mut targets = BTreeSet::new();
    let mut providers = BTreeSet::new();
    for b in bindings {
        check_id(&b.portable_channel_id)?;
        check_id(&b.local_channel_id)?;
        if b.provider_id.trim() != b.provider_id
            || b.provider_id.is_empty()
            || b.provider_id.len() > 2048
            || b.provider_id.chars().any(char::is_control)
            || !targets.insert(&b.local_channel_id)
            || !providers.insert(&b.provider_id)
            || mapping
                .insert(b.portable_channel_id.clone(), b.local_channel_id.clone())
                .is_some()
        {
            return Err(scope_error());
        }
        let old = local_policy
            .channels
            .iter()
            .find(|c| c.id == b.local_channel_id);
        match b.mode {
            ReceiveBindingModeV2::Existing
                if old.is_some_and(|c| c.connection.provider_id == b.provider_id) => {}
            ReceiveBindingModeV2::New
                if old.is_none()
                    && !local_policy
                        .channels
                        .iter()
                        .any(|c| c.connection.provider_id == b.provider_id) => {}
            _ => return Err(error("model_receive_identity_conflict")),
        }
    }
    let selected_channels: BTreeSet<_> = selected
        .channels
        .iter()
        .map(|s| &s.portable_channel_id)
        .collect();
    if selected_channels.len() != selected.channels.len()
        || mapping.len() != selected_channels.len()
        || mapping.keys().any(|id| !selected_channels.contains(id))
    {
        return Err(scope_error());
    }
    let mut out = ReceiveModelsPlanV2 {
        library,
        policy: local_policy.clone(),
        public_sources: BTreeMap::new(),
        source_to_target: mapping,
        credential_actions: vec![],
        changed_channel_ids: vec![],
        changed_template_refs: vec![],
    };
    for scope in &selected.channels {
        let source = package
            .payload
            .policy
            .as_ref()
            .and_then(|p| {
                p.channels
                    .iter()
                    .find(|c| c.id == scope.portable_channel_id)
            })
            .ok_or_else(scope_error)?;
        let b = bindings
            .iter()
            .find(|b| b.portable_channel_id == source.id)
            .unwrap();
        let ids = selected_ids(source, scope)?;
        if scope.include_key && scope.include_authorization
            || (scope.include_key || scope.include_authorization) && !scope.include_connection
        {
            return Err(scope_error());
        }
        let old = local_policy
            .channels
            .iter()
            .find(|c| c.id == b.local_channel_id);
        if old.is_none() && !scope.include_connection && scope.family_ids.is_none() {
            return Err(error("overlay_connection_required"));
        }
        if b.mode == ReceiveBindingModeV2::New
            && out.policy.channels.iter().any(|c| c.label == source.label)
        {
            return Err(error("model_receive_identity_conflict"));
        }
        let whole = scope.model_ids.is_none() && scope.family_ids.is_none();
        let mut next = old
            .cloned()
            .unwrap_or_else(|| empty_channel(b, &source.label));
        if scope.model_ids.is_some()
            && old.is_some()
            && (next.defaults != source.defaults
                || source
                    .models
                    .iter()
                    .filter(|m| ids.contains(&m.raw_model_id))
                    .filter_map(|m| m.binding.as_ref())
                    .any(|binding| {
                        next.family_overrides.get(&binding.reference().template_id)
                            != source
                                .family_overrides
                                .get(&binding.reference().template_id)
                    }))
        {
            return Err(error("overlay_shared_rules_conflict"));
        }
        if scope.include_connection {
            let connection = source
                .connection
                .as_ref()
                .ok_or_else(|| error("overlay_connection_required"))?;
            let direction_changed = next.connection.nodes != connection.nodes
                || next.connection.active_node_id != connection.active_node_id
                || next.connection.auth_mode.as_deref().unwrap_or("key")
                    != connection.auth_mode.as_deref().unwrap_or("key");
            if old.is_some()
                && direction_changed
                && !scope.include_key
                && !scope.include_authorization
            {
                return Err(error("model_receive_credentials_direction"));
            }
            next.connection.nodes = connection.nodes.clone();
            next.connection.active_node_id = connection.active_node_id.clone();
            // A newly created no-auth/local source has no existing credential
            // authority to preserve. Keep its explicit public direction instead
            // of silently converting it to the absent-field default "key".
            if old.is_none()
                && !scope.include_key
                && !scope.include_authorization
                && matches!(connection.auth_mode.as_deref(), Some("none" | "local"))
            {
                next.connection.auth_mode = connection.auth_mode.clone();
            }
            // Portable enabled/private-network flags confer no local authority.
            // L0 adopts auth/activation together with the actual outer material.
            if scope.include_key || scope.include_authorization {
                next.connection.credential_ref = None;
                next.connection.auth_mode = None;
                out.credential_actions.push(ReceiveCredentialActionV2 {
                    portable_channel_id: source.id.clone(),
                    local_channel_id: b.local_channel_id.clone(),
                    provider_id: b.provider_id.clone(),
                    kind: if scope.include_key {
                        ReceiveCredentialKindV2::Key
                    } else {
                        ReceiveCredentialKindV2::Authorization
                    },
                });
            }
            next.connection.source_read_id = None;
            next.connection.imported_config = None;
        }
        if whole {
            if adopted.contains(&source.id) {
                let mut namespace = source.namespace.clone().ok_or_else(scope_error)?;
                namespace.channel_id = b.local_channel_id.clone();
                next.namespace = Some(namespace);
            }
            next.label = source.label.clone();
            next.defaults = source.defaults.clone();
            next.family_overrides
                .extend(source.family_overrides.clone());
            merge_refs(
                &mut next.template_refs,
                source.template_refs.iter().cloned(),
            );
        } else if let Some(families) = &scope.family_ids {
            next.family_overrides.extend(
                source
                    .family_overrides
                    .iter()
                    .filter(|(id, _)| families.contains(id))
                    .map(|(id, rules)| (id.clone(), rules.clone())),
            );
            merge_refs(
                &mut next.template_refs,
                source
                    .template_refs
                    .iter()
                    .filter(|r| families.contains(&r.template_id))
                    .cloned(),
            );
        } else if old.is_none() {
            next.defaults = source.defaults.clone();
            let families: BTreeSet<_> = source
                .models
                .iter()
                .filter(|m| ids.contains(&m.raw_model_id))
                .filter_map(|m| m.binding.as_ref().map(|b| &b.reference().template_id))
                .collect();
            next.family_overrides.extend(
                source
                    .family_overrides
                    .iter()
                    .filter(|(id, _)| families.contains(id))
                    .map(|(id, rules)| (id.clone(), rules.clone())),
            );
            next.template_refs = source
                .template_refs
                .iter()
                .filter(|r| families.contains(&r.template_id))
                .cloned()
                .collect();
        }
        for model in source
            .models
            .iter()
            .filter(|m| ids.contains(&m.raw_model_id))
        {
            let mut model = model.clone();
            for field in model.source.fields.values_mut() {
                field.evidence.kind = SourceKindV2::PortableImport;
            }
            if let Some(slot) = next
                .models
                .iter_mut()
                .find(|m| m.raw_model_id == model.raw_model_id)
            {
                *slot = model;
            } else {
                next.models.push(model);
            }
        }
        for id in &ids {
            match source.naming.get(id) {
                Some(n) => {
                    next.naming.insert(id.clone(), n.clone());
                }
                None => {
                    next.naming.remove(id);
                }
            }
        }
        let available: Vec<_> = next.models.iter().map(|m| m.raw_model_id.clone()).collect();
        next.selection = merge_selection(
            &next.selection,
            &source.selection,
            &ids,
            &available.iter().cloned().collect(),
            whole,
        )?;
        next.model_order = reorder(&next.model_order, &source.model_order, &ids, &available);
        if old.is_none() && whole {
            next.selection = source.selection.clone();
        }
        if let Some(public) = selected_public(package, source, scope, &ids)? {
            out.public_sources
                .insert(b.local_channel_id.clone(), public);
        }
        if old != Some(&next)
            || out.public_sources.contains_key(&b.local_channel_id)
            || scope.include_key
            || scope.include_authorization
        {
            out.changed_channel_ids.push(b.local_channel_id.clone());
        }
        if let Some(slot) = out.policy.channels.iter_mut().find(|c| c.id == next.id) {
            *slot = next;
        } else {
            out.policy.channels.push(next);
        }
    }
    if selected.include_global_template_refs {
        let p = package.payload.policy.as_ref().ok_or_else(scope_error)?;
        merge_refs(
            &mut out.policy.template_refs,
            p.template_refs.iter().cloned(),
        );
        let order: Vec<_> = p
            .channel_order
            .iter()
            .filter_map(|id| out.source_to_target.get(id).cloned())
            .collect();
        let current: Vec<_> = out.policy.channels.iter().map(|c| c.id.clone()).collect();
        let ids = order.iter().cloned().collect();
        let order = reorder(&current, &order, &ids, &current);
        let channels = out.policy.channels.clone();
        out.policy.channels = order
            .iter()
            .map(|id| channels.iter().find(|c| &c.id == id).unwrap().clone())
            .collect();
        if let Some(id) = &p.default_channel_id {
            out.policy.default_channel_id = Some(
                out.source_to_target
                    .get(id)
                    .ok_or_else(scope_error)?
                    .clone(),
            );
        }
    }
    validate_library_v2(&out.library)?;
    validate_dependencies(&out.library, &out.policy)?;
    let mut changed = BTreeSet::new();
    for t in &out.library.templates {
        let before = local_library.templates.iter().find(|old| old.id == t.id);
        for r in &t.revisions {
            if before.is_none_or(|t| !t.revisions.iter().any(|old| old.id == r.id))
                || t.default_revision.as_ref() == Some(&r.id)
                    && before.and_then(|t| t.default_revision.as_ref()) != Some(&r.id)
            {
                changed.insert(RefV2 {
                    template_id: t.id.clone(),
                    revision_id: r.id.clone(),
                    digest: r.digest.clone(),
                });
            }
        }
    }
    for r in out
        .policy
        .template_refs
        .iter()
        .filter(|r| !local_policy.template_refs.contains(r))
        .chain(out.policy.channels.iter().flat_map(|c| {
            c.template_refs.iter().filter(|r| {
                local_policy
                    .channels
                    .iter()
                    .find(|old| old.id == c.id)
                    .is_none_or(|old| !old.template_refs.contains(r))
            })
        }))
    {
        changed.insert(r.clone());
    }
    out.changed_template_refs = changed.into_iter().collect();
    out.changed_channel_ids.sort();
    out.credential_actions
        .sort_by(|a, b| a.local_channel_id.cmp(&b.local_channel_id));
    Ok(out)
}
fn selected_public(
    package: &PortableModelsV2,
    source: &PortableChannelV2,
    scope: &ChannelSelectionV2,
    ids: &BTreeSet<String>,
) -> Result<Option<Value>> {
    let Some(public) = source.connection.as_ref().and_then(|c| c.provider.as_ref()) else {
        return Ok(None);
    };
    if scope.family_ids.is_some() {
        return Ok(None);
    }
    if scope.include_connection && scope.model_ids.is_none() {
        return Ok(Some(public.clone()));
    }
    // Use the actual L1 attachment/subset path on a constrained v6 copy.
    // No v1 receive/resolver or provider.adapter protocol inference is involved.
    let mut narrowed = package.clone();
    let c = narrowed
        .payload
        .policy
        .as_mut()
        .unwrap()
        .channels
        .iter_mut()
        .find(|c| c.id == source.id)
        .unwrap();
    c.models.retain(|m| ids.contains(&m.raw_model_id));
    c.model_order.retain(|id| ids.contains(id));
    c.naming.retain(|id, _| ids.contains(id));
    if let ModelSelection::Whitelist(v) = &mut c.selection {
        v.retain(|id| ids.contains(id));
    }
    c.model_subset = Some(ids.iter().cloned().collect());
    c.connection.as_mut().unwrap().provider = None;
    narrowed.digest = digest(&canonical_bytes(&narrowed.payload)?);
    let attached = portable_v2::attach_public_sources_v2(
        &narrowed,
        &BTreeMap::from([(source.id.clone(), public.clone())]),
    )?;
    let mut public = attached
        .payload
        .policy
        .unwrap()
        .channels
        .into_iter()
        .find(|c| c.id == source.id)
        .unwrap()
        .connection
        .unwrap()
        .provider
        .unwrap();
    if !scope.include_connection {
        // Only model-keyed/list attachments are authorized. Provider-wide
        // fields remain unselected, including modelPreset and execution flags.
        public.as_object_mut().unwrap().retain(|key, value| {
            key != "modelPreset" && key.starts_with("model") && value.is_object()
                || key == "autoReviewModelOverrides"
                || matches!(key.as_str(), "models" | "retainModels" | "selectedModels")
                || key.ends_with("Models") && value.is_array()
        });
    }
    Ok(Some(public))
}
#[cfg(test)]
#[path = "receive_models_v2_tests.rs"]
mod tests;
