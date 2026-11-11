//! Trusted read-only projection and explicit LocalFile draft capture.
//! Backend draft only; no recovery, registry writes, rematching or save events.
use super::*;
use crate::modules::model_config::{
    content_versions::{self, TimeKind},
    file_objects_v2::{self, CapturedObjects, SavedObjects, SecretValues},
    file_transfer_v2::{DraftFileCaptureV2, DraftFileIntentsV2},
    objects_overlay_v2::UsageV2,
    portable_v2::{self, ModelExportScopeV2},
    saved_objects_capture_v2::{
        self as capture, CaptureCredentialsV2, CaptureRequestV2, CapturedSnapshotV2,
    },
    schema::{invalid, ModelError, ModelResult},
    sync_v2::types::SourceGuard,
};
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftProjectionV2 {
    pub expected: ReadBaseV2,
    pub effective: Vec<EffectiveModelV2>,
    pub intended: PolicyV2,
    pub digest: String,
}
pub struct ConfigDraftFileCaptureV2;
fn mapped(e: MatchErrorV2) -> ModelError {
    invalid(&e.code, "草稿只读投影或导出未通过核对，业务文件未写入")
}
fn with_draft<T>(
    d: &Path,
    h: &Path,
    expected: &ReadBaseV2,
    allow_equivalent_for_projection: bool,
    read: impl FnOnce(&StoreSnapshotV2, &DraftV2, Value) -> Result<T>,
) -> Result<T> {
    if !d.is_absolute() || !d.is_dir() || !h.is_absolute() || !h.is_dir() {
        return Err(error("capture_root_invalid"));
    }
    let _admission = crate::infrastructure::storage_writers::global()
        .admit()
        .map_err(io)?;
    // Use the actual admission lock targets, without their recovery callbacks.
    let pending = [
        "manager-state/saved-file-save-pending.json",
        "manager-state/provider-rename-pending.json",
        "manager-state/model-transaction-v2.pending.json",
        "manager-state/credential-references-v2.pending.json",
    ];
    let _pending_locks = pending
        .iter()
        .map(|relative| {
            let path = d.join(relative);
            safety::check_path(d, &path, true).map_err(io)?;
            safety::check_path(
                d,
                &crate::infrastructure::locking::lock_path_for(&path),
                true,
            )
            .map_err(io)?;
            TargetFileLock::lock(&path).map_err(io)
        })
        .collect::<Result<Vec<_>>>()?;
    for relative in pending {
        if std::fs::symlink_metadata(d.join(relative)).is_ok() {
            return Err(error("model_transaction_pending"));
        }
    }
    let config = h.join("config.json");
    safety::check_path(h, &config, true).map_err(io)?;
    safety::check_path(
        h,
        &crate::infrastructure::locking::lock_path_for(&config),
        true,
    )
    .map_err(io)?;
    let _config = TargetFileLock::lock(&config).map_err(io)?;
    let store = StoreV2::new(d);
    let locked = store.lock()?;
    let state = locked.snapshot_unlocked()?;
    let (raw, bytes) = read_raw(h)?;
    let observed_source = source_digest(&bytes);
    let draft = state
        .draft
        .as_ref()
        .ok_or_else(|| error("model_read_stale"))?;
    if &draft.base != expected
        || (expected.source_digest != observed_source
            && !(allow_equivalent_for_projection
                && source_matches_draft(draft, &observed_source, &raw, bytes.is_some())))
        || expected.root_revision != root_revision(d, h)?
        || expected.library_digest != state.digests.library
        || expected.policy_digest != policy_semantic_digest_v2(&state.policy)?
    {
        return Err(error("model_read_stale"));
    }
    let baseline = draft
        .observed_baseline
        .as_ref()
        .ok_or_else(|| error("capture_draft_baseline_required"))?;
    // The stored observation fixes existing provider identities. No call to
    // bindings(), which may allocate UUIDs or write the identity registry.
    let actual_bindings = baseline
        .channels
        .iter()
        .map(|c| ProviderBindingV2 {
            channel_id: c.id.clone(),
            provider_id: c.connection.provider_id.clone(),
        })
        .collect::<Vec<_>>();
    let observed = provider_import::read_policy_v2(
        &raw,
        &expected.source_digest,
        &state.policy,
        state.ledger.as_ref(),
        &state.mappings,
        &actual_bindings,
    )?;
    if policy_semantic_digest_v2(&observed)? != policy_semantic_digest_v2(baseline)? {
        return Err(error("capture_draft_baseline_stale"));
    }
    let value = read(&state, draft, raw)?;
    let (_, after) = read_raw(h)?;
    if source_digest(&after) != observed_source
        || root_revision(d, h)? != expected.root_revision
        || locked.snapshot_unlocked()?.digests != state.digests
    {
        return Err(error("capture_stale"));
    }
    Ok(value)
}
struct Projection {
    dto: DraftProjectionV2,
    compiled: compiler_v2::CompiledFileV2,
}
fn render(
    d: &Path,
    state: &StoreSnapshotV2,
    draft: &DraftV2,
    mut raw: Value,
    mut request: SaveRequestV2,
) -> Result<Projection> {
    validate_save_scope_v2(&request)?;
    let (intended, mappings) = transform_draft_v2(
        d,
        &draft.base.source_digest,
        state,
        draft,
        &mut raw,
        &mut request,
    )?;
    for channel in &intended.channels {
        let provider = raw["providers"]
            .get_mut(&channel.connection.provider_id)
            .ok_or_else(|| error("model_source_invalid"))?;
        if !protected(&channel.connection.provider_id, provider) {
            project_connection_v2(provider, channel);
        }
    }
    validate_policy_v2(&intended)?;
    let plan = plan_materialization_v2(&state.library, &state.policy, &intended)?;
    let mappings = allocate_mappings(&raw, &intended, &plan.groups, &mappings)?;
    let compiled = compiler_v2::compile_file_v2(
        &state.library,
        &state.policy,
        &intended,
        &raw,
        &mappings,
        &auth_bindings(&raw, &intended)?,
    )?;
    let alias_policy = if let Some(scope) = &request.scope_channel_id {
        let mut selected = intended.clone();
        selected.channels.retain(|c| &c.id == scope);
        selected
    } else {
        intended.clone()
    };
    validate_aliases(&alias_policy, &compiled, &raw)?;
    // Save scopes isolate persisted changes. A nonwrite projection must still
    // retain every other server draft edit in the DTO returned to its editor.
    let intended = if let Some(scope) = &request.scope_channel_id {
        let mut full = draft.policy.clone();
        if let Some(channel) = intended.channels.iter().find(|c| &c.id == scope) {
            if let Some(slot) = full.channels.iter_mut().find(|c| &c.id == scope) {
                *slot = channel.clone();
            } else {
                full.channels.push(channel.clone());
            }
        } else {
            full.channels.retain(|c| &c.id != scope);
            if full.default_channel_id.as_ref() == Some(scope) {
                full.default_channel_id = None;
            }
        }
        full
    } else {
        intended
    };
    let effective = super::effective(&state.library, &intended)?;
    let digest = digest(&canonical_bytes(&(
        &request.expected,
        &effective,
        &intended,
    ))?);
    Ok(Projection {
        dto: DraftProjectionV2 {
            expected: request.expected,
            effective,
            intended,
            digest,
        },
        compiled,
    })
}
fn validate_aliases(
    policy: &PolicyV2,
    compiled: &compiler_v2::CompiledFileV2,
    raw: &Value,
) -> Result<()> {
    let native = raw["providers"]
        .as_object()
        .into_iter()
        .flatten()
        .filter(|(id, row)| protected(id, row))
        .flat_map(|(_, row)| {
            row["models"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
        })
        .map(str::to_ascii_lowercase)
        .collect::<BTreeSet<_>>();
    for channel in &policy.channels {
        for naming in channel.naming.values() {
            naming.validate().map_err(|e| error(&e.code))?;
        }
        let known = channel
            .models
            .iter()
            .flat_map(|m| {
                [
                    m.raw_model_id.to_ascii_lowercase(),
                    m.raw_model_id.replace('/', "-").to_ascii_lowercase(),
                ]
            })
            .collect::<BTreeSet<_>>();
        let mut claims = BTreeMap::new();
        let providers = std::iter::once(channel.connection.provider_id.as_str())
            .chain(
                compiled
                    .mappings
                    .mappings
                    .iter()
                    .filter(|m| m.channel_id == channel.id)
                    .map(|m| m.provider_id.as_str()),
            )
            .collect::<BTreeSet<_>>();
        for provider in providers {
            let Some(value) = compiled.next_config["providers"][provider].get("modelAliases")
            else {
                continue;
            };
            let aliases = value.as_object().ok_or_else(|| error("model_alias"))?;
            for (id, value) in aliases {
                let alias = value.as_str().ok_or_else(|| error("model_alias"))?;
                if alias.is_empty() {
                    continue;
                }
                let lower = alias.to_ascii_lowercase();
                if known.contains(&lower)
                    || native.contains(&lower)
                    || claims.insert(lower, id).is_some_and(|prior| prior != id)
                {
                    return Err(error("model_alias_collision"));
                }
            }
        }
    }
    Ok(())
}
/// Backend-only historical policy projection; callers first validate exact
/// image/source identity. No UI deserializer can supply this authority.
pub(super) fn project_trusted_draft(
    d: &Path,
    state: &StoreSnapshotV2,
    draft: &DraftV2,
    raw: Value,
) -> Result<DraftProjectionV2> {
    render(
        d,
        state,
        draft,
        raw,
        SaveRequestV2 {
            expected: draft.base.clone(),
            edits: vec![],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )
    .map(|p| p.dto)
}
/// Public nonwrite intent projection. The original draft remains unchanged.
pub fn project(d: &Path, h: &Path, request: SaveRequestV2) -> Result<DraftProjectionV2> {
    let expected = request.expected.clone();
    with_draft(d, h, &expected, true, |state, draft, raw| {
        if draft.policy.recovery_id.is_some() {
            super::history::validate_save(d, h, state, draft, &request)?;
        }
        render(d, state, draft, raw, request).map(|p| p.dto)
    })
}
fn validate_capture_request(request: &CaptureRequestV2) -> ModelResult<()> {
    capture::validate_request(request)?;
    let s = &request.selection;
    if s.usage != UsageV2::LocalFile
        || s.channels.is_empty()
        || s.include_global_template_refs
        || !s.adopt_template_defaults.is_empty()
        || s.channels
            .iter()
            .any(|c| c.include_key || c.include_authorization)
        || s.manager.preferences
        || s.manager.extensions
        || !s.manager.account_ids.is_empty()
        || !s.manager.webdav_ids.is_empty()
        || !s.manager.webdav_password_ids.is_empty()
        || !s.skill_names.is_empty()
        || !s.mcp_sources.is_empty()
        || !s.extension_credentials.skill_names.is_empty()
        || !s.extension_credentials.mcp_names.is_empty()
        || !request.mcp_clients.is_empty()
    {
        return Err(invalid(
            "capture_draft_scope",
            "显式草稿导出仅支持无凭据的LocalFile渠道与精确模版依赖",
        ));
    }
    Ok(())
}
impl DraftFileCaptureV2 for ConfigDraftFileCaptureV2 {
    fn capture(
        &self,
        d: &Path,
        h: &Path,
        expected: &ReadBaseV2,
        request: &CaptureRequestV2,
        _credentials: &dyn CaptureCredentialsV2,
    ) -> ModelResult<CapturedSnapshotV2> {
        capture_typed_intents(
            d,
            h,
            request,
            SaveRequestV2 {
                expected: expected.clone(),
                edits: vec![],
                channel_intents: vec![],
                scope_channel_id: None,
            },
        )
    }
    fn capture_with_intents(
        &self,
        d: &Path,
        h: &Path,
        expected: &ReadBaseV2,
        request: &CaptureRequestV2,
        _credentials: &dyn CaptureCredentialsV2,
        intents: DraftFileIntentsV2,
    ) -> ModelResult<CapturedSnapshotV2> {
        capture_typed_intents(
            d,
            h,
            request,
            SaveRequestV2 {
                expected: expected.clone(),
                edits: intents.edits,
                channel_intents: intents.channel_intents,
                scope_channel_id: None,
            },
        )
    }
}
fn export_scope(policy: &PolicyV2, request: &CaptureRequestV2) -> Result<ModelExportScopeV2> {
    let s = &request.selection;
    let mut scope = ModelExportScopeV2 {
        template_refs: s.template_refs.clone(),
        ..Default::default()
    };
    for c in &s.channels {
        let channel = policy
            .channels
            .iter()
            .find(|v| v.id == c.portable_channel_id)
            .ok_or_else(|| error("capture_selection_missing"))?;
        if c.family_ids.as_ref().is_some_and(|ids| {
            ids.iter().any(|id| {
                !channel.family_overrides.contains_key(id)
                    && !channel.template_refs.iter().any(|r| &r.template_id == id)
                    && !channel.models.iter().any(|m| {
                        m.binding
                            .as_ref()
                            .is_some_and(|b| &b.reference().template_id == id)
                    })
            })
        }) {
            return Err(error("capture_selection_missing"));
        }
        if let Some(ids) = &c.model_ids {
            scope
                .channel_models
                .insert(c.portable_channel_id.clone(), ids.iter().cloned().collect());
        } else if let Some(ids) = &c.family_ids {
            scope
                .channel_families
                .insert(c.portable_channel_id.clone(), ids.iter().cloned().collect());
        } else {
            scope.channel_ids.insert(c.portable_channel_id.clone());
        }
    }
    Ok(scope)
}
fn capture_typed_intents(
    d: &Path,
    h: &Path,
    request: &CaptureRequestV2,
    intent: SaveRequestV2,
) -> ModelResult<CapturedSnapshotV2> {
    validate_capture_request(request)?;
    let expected = intent.expected.clone();
    with_draft(d, h, &expected, false, |state, draft, raw| {
        if intent.edits.is_empty() && intent.channel_intents.is_empty() {
            return capture_locked(d, state, draft, raw, request);
        }
        if draft.policy.recovery_id.is_some() {
            super::history::validate_save(d, h, state, draft, &intent)?;
        }
        let projection = render(d, state, draft, raw, intent)?;
        let scope = export_scope(&projection.dto.intended, request)?;
        capture_projection(d, state, draft, request, &scope, projection)
    })
    .map_err(mapped)
}
fn capture_locked(
    d: &Path,
    state: &StoreSnapshotV2,
    draft: &DraftV2,
    raw: Value,
    request: &CaptureRequestV2,
) -> Result<CapturedSnapshotV2> {
    let s = &request.selection;
    let scope = export_scope(&draft.policy, request)?;
    // Reuse the v6 scoper, retaining server-owned connection identities. This
    // prevents an unrelated draft/model with missing protocol from blocking a
    // selected export or leaking into its actual compiled public projection.
    let scoped = portable_v2::export_portable_models_v2(&state.library, &draft.policy, &scope)?;
    let mut selected = draft.clone();
    let ids = s
        .channels
        .iter()
        .map(|c| c.portable_channel_id.as_str())
        .collect::<BTreeSet<_>>();
    selected
        .policy
        .channels
        .retain(|c| ids.contains(c.id.as_str()));
    selected.policy.template_refs.clear();
    selected.policy.pending_removals.clear();
    if selected
        .policy
        .default_channel_id
        .as_deref()
        .is_some_and(|id| !ids.contains(id))
    {
        selected.policy.default_channel_id = None;
    }
    for channel in &mut selected.policy.channels {
        let c = scoped
            .payload
            .policy
            .as_ref()
            .and_then(|p| p.channels.iter().find(|c| c.id == channel.id))
            .ok_or_else(|| error("capture_selection_missing"))?;
        channel.models = c.models.clone();
        channel.template_refs = c.template_refs.clone();
        channel.selection = c.selection.clone();
        channel.model_order = c.model_order.clone();
        channel.defaults = c.defaults.clone();
        channel.family_overrides = c.family_overrides.clone();
        channel.naming = c.naming.clone();
    }
    let projection = render(
        d,
        state,
        &selected,
        raw,
        SaveRequestV2 {
            expected: draft.base.clone(),
            edits: vec![],
            channel_intents: vec![],
            scope_channel_id: None,
        },
    )?;
    capture_projection(d, state, draft, request, &scope, projection)
}
fn capture_projection(
    d: &Path,
    state: &StoreSnapshotV2,
    draft: &DraftV2,
    request: &CaptureRequestV2,
    scope: &ModelExportScopeV2,
    projection: Projection,
) -> Result<CapturedSnapshotV2> {
    let s = &request.selection;
    let mut package =
        portable_v2::export_portable_models_v2(&state.library, &projection.dto.intended, scope)?;
    capture::exact_revisions(&mut package, &state.library, s).map_err(|e| error(&e.code))?;
    let mut sources = BTreeMap::new();
    for choice in &s.channels {
        if choice.family_ids.is_none() {
            let channel = projection
                .dto
                .intended
                .channels
                .iter()
                .find(|c| c.id == choice.portable_channel_id)
                .ok_or_else(|| error("capture_selection_missing"))?;
            sources.insert(
                channel.id.clone(),
                capture::public_source(
                    &projection.compiled.next_config,
                    channel,
                    &projection.compiled.mappings,
                )
                .map_err(|e| error(&e.code))?,
            );
        }
    }
    package = portable_v2::attach_public_sources_v2(&package, &sources)?;
    for choice in &s.channels {
        if !choice.include_connection {
            package
                .payload
                .policy
                .as_mut()
                .unwrap()
                .channels
                .iter_mut()
                .find(|c| c.id == choice.portable_channel_id)
                .unwrap()
                .connection = None;
        }
    }
    package.digest = digest(&canonical_bytes(&package.payload)?);
    let objects = SavedObjects::capture(
        CapturedObjects {
            models: package,
            manager: Default::default(),
            assets: Default::default(),
            channel_keys: SecretValues::new(),
            channel_authorizations: BTreeMap::new(),
            webdav_passwords: SecretValues::new(),
            content_versions: vec![],
        },
        None,
    )
    .map_err(|e| error(&e.code))?;
    let save_state = file_objects_v2::save_state(&objects).map_err(|e| error(&e.code))?;
    let semantic_digest = file_objects_v2::semantic_digest(&objects).map_err(|e| error(&e.code))?;
    let _versions = content_versions::lock_versions(d).map_err(|e| error(&e.code))?;
    let mut captured = objects.into_captured();
    captured.content_versions = content_versions::observe(d, &save_state)
        .map_err(|e| error(&e.code))?
        .into_iter()
        .filter(|v| {
            v.time_kind != TimeKind::Unknown
                && save_state.get(&v.scope_key) == Some(&v.content_digest)
        })
        .collect();
    let objects = SavedObjects::capture(captured, None).map_err(|e| error(&e.code))?;
    let source_guard = SourceGuard {
        saved_source_digest: digest(&canonical_bytes(&json!([
            "OCXD_DRAFT_CAPTURE_V2",
            draft.base,
            draft.observed_baseline,
            state.digests,
            semantic_digest,
            save_state
        ]))?),
        scope_digest: capture::scope_digest(request).map_err(|e| error(&e.code))?,
        mcp_sources_digest: digest(&canonical_bytes(&json!([]))?),
    };
    let summary = objects.summary();
    Ok(CapturedSnapshotV2 {
        objects,
        source_guard,
        root_revision: draft.base.root_revision.clone(),
        semantic_digest,
        save_state,
        summary,
    })
}
#[cfg(test)]
#[path = "config_file_v2_export_tests.rs"]
mod tests;
