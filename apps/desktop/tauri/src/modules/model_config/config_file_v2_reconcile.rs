//! An explicit source recheck. It can advance a draft, never publish config.
use super::*;
use crate::modules::model_config::store_v2::ProviderObservationV2;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReconcileRequest {
    pub expected: ReadBaseV2,
    // The UI sends only identities of buffered edits, never their contents.
    pub local_channel_ids: BTreeSet<String>,
    /// Only model field edits in memory; no channel or credential edits.
    #[serde(default)]
    pub model_only_channel_ids: BTreeSet<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub source_digest: String,
    pub review_digest: String,
    pub changed_channels: Vec<String>,
    pub conflicts: Vec<Conflict>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Conflict {
    pub label: String,
    pub reason: &'static str,
}

pub(super) fn provider_digest(row: &Value) -> Result<String> {
    Ok(digest(&Zeroizing::new(canonical_bytes(row)?)))
}

pub(super) fn provider_observations(
    raw: &Value,
) -> Result<BTreeMap<String, ProviderObservationV2>> {
    let Some(providers) = raw.get("providers") else {
        return Ok(BTreeMap::new());
    };
    let providers = providers
        .as_object()
        .ok_or_else(|| error("model_source_invalid"))?;
    providers
        .iter()
        .map(|(id, row)| {
            if !row.is_object() {
                return Err(error("model_source_invalid"));
            }
            Ok((
                id.clone(),
                ProviderObservationV2 {
                    row_digest: provider_digest(row)?,
                    name_digest: digest(&canonical_bytes(&row.get("name"))?),
                    non_connection_digest: {
                        let fields = row
                            .as_object()
                            .unwrap()
                            .iter()
                            .filter(|(key, _)| {
                                !matches!(
                                    key.as_str(),
                                    "baseUrl"
                                        | "apiKey"
                                        | "apiKeyPool"
                                        | "headers"
                                        | "apiKeySelectionRevision"
                                        | "_apiKeyAttempt"
                                )
                            })
                            .collect::<BTreeMap<_, _>>();
                        digest(&Zeroizing::new(canonical_bytes(&fields)?))
                    },
                },
            ))
        })
        .collect()
}

fn channel_meaning(channel: &ChannelV2) -> Result<String> {
    let mut p = store_v2::empty_policy_v2();
    p.channels.push(channel.clone());
    policy_semantic_digest_v2(&p)
}

fn same_channel(left: Option<&ChannelV2>, right: Option<&ChannelV2>) -> Result<bool> {
    match (left, right) {
        (Some(a), Some(b)) => Ok(channel_meaning(a)? == channel_meaning(b)?),
        (None, None) => Ok(true),
        _ => Ok(false),
    }
}

fn plan(
    root: &Path,
    state: &StoreSnapshotV2,
    draft: &DraftV2,
    request: &ReconcileRequest,
    observation: (PolicyV2, BTreeMap<String, ProviderObservationV2>),
    raw: &Value,
    source: (bool, String),
) -> Result<(Preview, Option<DraftV2>)> {
    let (observed, current) = observation;
    let (bytes_present, source) = source;
    let baseline = draft
        .observed_baseline
        .as_ref()
        .ok_or_else(|| error("model_reconcile_baseline_missing"))?;
    let prior = draft
        .observed_providers
        .as_ref()
        .ok_or_else(|| error("model_reconcile_baseline_missing"))?;
    let mut preview = Preview {
        source_digest: source.clone(),
        review_digest: String::new(),
        changed_channels: vec![],
        conflicts: vec![],
    };
    let mut next = observed.clone();
    let mut local_ids = request.local_channel_ids.clone();
    local_ids.extend(draft.credential_edits.keys().cloned());
    if local_ids
        .iter()
        .any(|id| !draft.policy.channels.iter().any(|c| &c.id == id))
    {
        return Err(error("model_reconcile_stale"));
    }
    for old in &baseline.channels {
        let existing = observed.channels.iter().find(|c| c.id == old.id);
        let edited = draft.policy.channels.iter().find(|c| c.id == old.id);
        let server_edits =
            !same_channel(Some(old), edited)? || draft.credential_edits.contains_key(&old.id);
        let local = local_ids.contains(&old.id) || server_edits;
        let provider_ids = std::iter::once(&old.connection.provider_id).chain(
            state
                .mappings
                .mappings
                .iter()
                .filter(|m| m.channel_id == old.id)
                .map(|m| &m.provider_id),
        );
        let mut provider_changed = false;
        let mut name_changed = false;
        let mut connection_only = true;
        for id in provider_ids {
            provider_changed |= prior.get(id) != current.get(id);
            name_changed |= prior
                .get(id)
                .zip(current.get(id))
                .is_some_and(|(a, b)| a.name_digest != b.name_digest);
            connection_only &= prior
                .get(id)
                .zip(current.get(id))
                .is_some_and(|(a, b)| a.non_connection_digest == b.non_connection_digest);
        }
        let changed = provider_changed || !same_channel(Some(old), existing)?;
        let keep_latest_connection = if changed
            && connection_only
            && !server_edits
            && request.model_only_channel_ids.contains(&old.id)
        {
            if let Some(current) = existing {
                let mut normalized = current.clone();
                normalized
                    .connection
                    .nodes
                    .clone_from(&old.connection.nodes);
                normalized
                    .connection
                    .active_node_id
                    .clone_from(&old.connection.active_node_id);
                same_channel(Some(old), Some(&normalized))?
            } else {
                false
            }
        } else {
            false
        };
        if changed {
            preview.changed_channels.push(old.label.clone());
        }
        let reason = if existing.is_none() && local {
            Some("removed")
        } else if name_changed {
            Some("native_name")
        } else if changed && local && !keep_latest_connection {
            Some("overlap")
        } else {
            None
        };
        if let Some(reason) = reason {
            preview.conflicts.push(Conflict {
                label: old.label.clone(),
                reason,
            });
        } else if local && !keep_latest_connection {
            // Prefer the draft only where its observed channel is still the same.
            next.channels.retain(|c| c.id != old.id || edited.is_some());
            if let Some((slot, edited)) = next
                .channels
                .iter_mut()
                .find(|c| c.id == old.id)
                .zip(edited)
            {
                *slot = edited.clone();
            }
        }
    }
    for channel in &draft.policy.channels {
        if channel.connection.imported_config.is_some()
            && verified_copy_source(root, raw, channel, &source).is_err()
        {
            // The copy source is bound to the original bytes. A recheck cannot
            // silently change which exact source will supply its authentication.
            preview.conflicts.push(Conflict {
                label: channel.label.clone(),
                reason: "bound_copy",
            });
        }
        if !baseline.channels.iter().any(|c| c.id == channel.id) {
            if current
                .keys()
                .any(|id| id.to_lowercase() == channel.connection.provider_id.to_lowercase())
            {
                preview.conflicts.push(Conflict {
                    label: channel.label.clone(),
                    reason: "identity",
                });
            } else {
                next.channels.push(channel.clone());
            }
        }
    }
    for channel in &observed.channels {
        if !baseline.channels.iter().any(|c| c.id == channel.id) {
            preview.changed_channels.push(channel.label.clone());
        }
    }
    if draft.policy.default_channel_id != baseline.default_channel_id {
        if observed.default_channel_id != baseline.default_channel_id {
            preview.conflicts.push(Conflict {
                label: "默认渠道".into(),
                reason: "overlap",
            });
        } else {
            next.default_channel_id
                .clone_from(&draft.policy.default_channel_id);
        }
    }
    next.template_refs.clone_from(&draft.policy.template_refs);
    if !draft.policy.pending_removals.is_empty() {
        preview.conflicts.push(Conflict {
            label: "待移除的配置来源".into(),
            reason: "bound_copy",
        });
    }
    if !preview.conflicts.is_empty() {
        return Ok((preview, None));
    }
    validate_policy_v2(&next)?;
    let mut rebased = draft.clone();
    rebased.base.source_digest = source;
    rebased.base.draft_generation = rebased
        .base
        .draft_generation
        .checked_add(1)
        .ok_or_else(|| error("model_limit"))?;
    rebased.observed_baseline = Some(observed);
    rebased.observed_config_digest = config_baseline(raw, bytes_present);
    rebased.observed_providers = Some(current);
    rebased.policy = next;
    Ok((preview, Some(rebased)))
}

fn work(
    root: &Path,
    home: &Path,
    request: ReconcileRequest,
    confirmed: Option<&str>,
) -> Result<(Preview, Option<FileSnapshotV2>)> {
    if !request
        .model_only_channel_ids
        .is_subset(&request.local_channel_ids)
    {
        return Err(error("model_reconcile_stale"));
    }
    let _admission = native_import::pending_admission(root)?;
    let _official = if confirmed.is_some() {
        Some(crate::modules::runtime::port::ConfigMutation::acquire(home).map_err(io)?)
    } else {
        None
    };
    safety::check_path(home, &home.join("config.json.lock"), true).map_err(io)?;
    let _config = TargetFileLock::lock(&home.join("config.json")).map_err(io)?;
    let store = StoreV2::new(root);
    let locked = store.lock()?;
    let state = locked.snapshot_unlocked()?;
    let (raw, bytes) = read_raw(home)?;
    let source = source_digest(&bytes);
    let revision = root_revision(root, home)?;
    let draft = state
        .draft
        .as_ref()
        .ok_or_else(|| error("model_reconcile_stale"))?;
    if draft.base != request.expected
        || draft.base.root_revision != revision
        || draft.base.library_digest != state.digests.library
        || draft.base.policy_digest != policy_semantic_digest_v2(&state.policy)?
    {
        return Err(error("model_reconcile_stale"));
    }
    let registry = root.join("manager-state/model-v2-provider-identities.json");
    safety::check_path(root, &registry, true).map_err(io)?;
    let identities = match std::fs::symlink_metadata(&registry) {
        Ok(_) => digest(&safety::read_file(&registry, 1024 * 1024).map_err(io)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => digest(&[]),
        Err(e) => return Err(io(e)),
    };
    let review_digest = digest(&canonical_bytes(&(
        &source,
        &revision,
        &state.digests,
        &request.local_channel_ids,
        &request.model_only_channel_ids,
        identities,
    ))?);
    if confirmed.is_some_and(|expected| expected != review_digest) {
        return Err(error("model_reconcile_stale"));
    }
    if draft.policy.recovery_id.is_some() {
        return Err(error("model_history_draft_active"));
    }
    if source == draft.base.source_digest {
        let snapshot = if confirmed.is_some() {
            Some(view(state, source.clone(), revision)?)
        } else {
            None
        };
        return Ok((
            Preview {
                source_digest: source,
                review_digest,
                changed_channels: vec![],
                conflicts: vec![],
            },
            snapshot,
        ));
    }
    let missing = draft
        .observed_baseline
        .as_ref()
        .into_iter()
        .flat_map(|p| &p.channels)
        .filter(|channel| {
            std::iter::once(&channel.connection.provider_id)
                .chain(
                    state
                        .mappings
                        .mappings
                        .iter()
                        .filter(|m| m.channel_id == channel.id)
                        .map(|m| &m.provider_id),
                )
                .any(|id| raw["providers"].get(id).is_none())
        })
        .map(|channel| Conflict {
            label: channel.label.clone(),
            reason: "removed",
        })
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        if confirmed.is_some() {
            return Err(error("model_reconcile_overlap"));
        }
        return Ok((
            Preview {
                source_digest: source,
                review_digest,
                changed_channels: missing.iter().map(|c| c.label.clone()).collect(),
                conflicts: missing,
            },
            None,
        ));
    }
    let observed = provider_import::read_policy_v2(
        &raw,
        &source,
        &state.policy,
        state.ledger.as_ref(),
        &state.mappings,
        &observed_bindings(
            root,
            &raw,
            &state.policy,
            Some(draft),
            &state.mappings,
            false,
        )?,
    )?;
    let (mut preview, next) = plan(
        root,
        &state,
        draft,
        &request,
        (observed, provider_observations(&raw)?),
        &raw,
        (bytes.is_some(), source.clone()),
    )?;
    preview.review_digest = review_digest;
    let Some(_) = confirmed else {
        return Ok((preview, None));
    };
    let next = next.ok_or_else(|| error("model_reconcile_overlap"))?;
    if source_digest(&read_raw(home)?.1) != source {
        return Err(error("model_reconcile_stale"));
    }
    locked.stage_draft_unlocked(Some(&request.expected), next)?;
    let snapshot = view(locked.snapshot_unlocked()?, source, revision)?;
    Ok((preview, Some(snapshot)))
}

pub fn preview(root: &Path, home: &Path, request: ReconcileRequest) -> Result<Preview> {
    work(root, home, request, None).map(|(preview, _)| preview)
}

pub fn apply(
    root: &Path,
    home: &Path,
    request: ReconcileRequest,
    review_digest: &str,
) -> Result<FileSnapshotV2> {
    work(root, home, request, Some(review_digest))?
        .1
        .ok_or_else(|| error("model_reconcile_stale"))
}
