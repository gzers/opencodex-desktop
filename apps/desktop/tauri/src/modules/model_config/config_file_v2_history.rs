//! Explicit restoration from this version's exact transaction images only.
//! Prepare freezes private metadata; accept stages a guarded backend draft.
use super::*;
use crate::modules::model_config::{
    model_transaction_v2::{history as images, TargetV2},
    schema::ChangeFact,
    store_v2::LockedStoreV2,
};
const DIRECTORY: &str = "manager-state/model-history-v2";
const LIMIT: u64 = 32 * 1024 * 1024;
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SideV2 {
    Before,
    After,
}
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeV2 {
    Affected,
    All,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HistoryPrepareV2 {
    pub source_id: String,
    pub side: SideV2,
    pub scope: ScopeV2,
    pub restore_defaults: bool,
    pub expected: ReadBaseV2,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HistoryPlanV2 {
    pub recovery_id: String,
    pub source_id: String,
    pub side: SideV2,
    pub scope: ScopeV2,
    pub restore_defaults: bool,
    pub changes: Vec<ChangeFact>,
    pub default_changes: Vec<ChangeFact>,
    pub channel_labels: Vec<String>,
    pub notices: Vec<String>,
    pub requires_runtime_review: bool,
    pub target_digest: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntryV2 {
    pub id: String,
    pub created_at: Option<String>,
    pub phase: String,
    pub channel_labels: Vec<String>,
    pub recoverable: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    schema_version: u32,
    request: HistoryPrepareV2,
    view: HistoryPlanV2,
    source_digest: String,
    policy: PolicyV2,
    defaults: Option<LibraryV2>,
    accepted_base: Option<ReadBaseV2>,
}
fn fail(code: &str) -> MatchErrorV2 {
    error(code)
}
fn path(d: &Path, id: &str) -> Result<std::path::PathBuf> {
    check_id(id)?;
    let path = d.join(DIRECTORY).join(format!("{id}.json"));
    safety::check_path(d, &path, true).map_err(io)?;
    Ok(path)
}
fn read_record(d: &Path, id: &str) -> Result<Record> {
    let r: Record = serde_json::from_value(parse_strict_json_v2(
        &safety::read_file(&path(d, id)?, LIMIT).map_err(io)?,
    )?)
    .map_err(json_error)?;
    if r.schema_version != 2
        || deterministic_id(&canonical_bytes(&r.request)?) != id
        || r.view.recovery_id != id
        || r.policy.recovery_id.as_deref() != Some(id)
        || r.view.target_digest != target_digest(&r.policy, r.defaults.as_ref())?
    {
        return Err(fail("model_history_plan_invalid"));
    }
    Ok(r)
}
fn persist(d: &Path, r: &Record) -> Result<()> {
    let bytes = canonical_bytes(r)?;
    if bytes.len() > LIMIT as usize {
        return Err(error("model_limit"));
    }
    let target = path(d, &r.view.recovery_id)?;
    crate::infrastructure::atomic_write::atomic_write(&target, &bytes, 0o600).map_err(io)?;
    if safety::read_file(&target, LIMIT).map_err(io)? != bytes {
        return Err(fail("model_history_plan_invalid"));
    }
    Ok(())
}
fn target_digest(policy: &PolicyV2, library: Option<&LibraryV2>) -> Result<String> {
    Ok(digest(&canonical_bytes(&(policy, library))?))
}
fn with_state<T>(
    d: &Path,
    h: &Path,
    expected: Option<&ReadBaseV2>,
    read: impl FnOnce(&LockedStoreV2<'_>, &StoreSnapshotV2, Value) -> Result<T>,
) -> Result<T> {
    if !d.is_absolute() || !d.is_dir() || !h.is_absolute() || !h.is_dir() {
        return Err(fail("model_history_root_invalid"));
    }
    let _admission = crate::infrastructure::storage_writers::global()
        .admit()
        .map_err(io)?;
    let pending = [
        "manager-state/saved-file-save-pending.json",
        "manager-state/provider-rename-pending.json",
        "manager-state/model-transaction-v2.pending.json",
        "manager-state/credential-references-v2.pending.json",
    ];
    let _guards = pending
        .iter()
        .map(|p| {
            let path = d.join(p);
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
    for p in pending {
        if std::fs::symlink_metadata(d.join(p)).is_ok() {
            return Err(fail("model_transaction_pending"));
        }
    }
    safety::check_path(h, &h.join("config.json"), true).map_err(io)?;
    safety::check_path(
        h,
        &crate::infrastructure::locking::lock_path_for(&h.join("config.json")),
        true,
    )
    .map_err(io)?;
    let _config = TargetFileLock::lock(&h.join("config.json")).map_err(io)?;
    let store = StoreV2::new(d);
    let locked = store.lock()?;
    let state = if expected.is_some() {
        locked.snapshot_unlocked()?
    } else {
        locked.snapshot_saved_unlocked()?
    };
    let (raw, bytes) = read_raw(h)?;
    if let Some(expected) = expected {
        let draft = state
            .draft
            .as_ref()
            .ok_or_else(|| fail("model_history_stale"))?;
        if &draft.base != expected
            || source_digest(&bytes) != expected.source_digest
            || root_revision(d, h)? != expected.root_revision
            || state.digests.library != expected.library_digest
            || policy_semantic_digest_v2(&state.policy)? != expected.policy_digest
        {
            return Err(fail("model_history_stale"));
        }
        let baseline = draft
            .observed_baseline
            .as_ref()
            .ok_or_else(|| fail("model_history_baseline_missing"))?;
        let bindings = baseline
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
            &bindings,
        )?;
        if policy_semantic_digest_v2(&observed)? != policy_semantic_digest_v2(baseline)? {
            return Err(fail("model_history_stale"));
        }
    }
    read(&locked, &state, raw)
}
fn image<'a>(
    source: &'a images::HistoryImagesV2,
    target: &TargetV2,
    side: SideV2,
) -> Result<Option<&'a [u8]>> {
    let row = source
        .images
        .iter()
        .find(|r| &r.target == target)
        .ok_or_else(|| fail("model_history_source_incomplete"))?;
    Ok(if side == SideV2::Before {
        row.before.as_deref()
    } else {
        row.after.as_deref()
    })
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    let value = parse_strict_json_v2(bytes)?;
    if value["schemaVersion"] != 2 {
        return Err(error("model_schema_unsupported"));
    }
    serde_json::from_value(value).map_err(json_error)
}
fn side_policy(
    source: &images::HistoryImagesV2,
    side: SideV2,
    current: &LibraryV2,
) -> Result<(PolicyV2, Option<LibraryV2>)> {
    let raw_bytes = image(source, &TargetV2::OfficialConfig, side)?
        .ok_or_else(|| fail("model_history_source_incomplete"))?;
    let raw = parse_strict_json_v2(raw_bytes)?;
    let policy_target = TargetV2::Manager(store_v2::POLICY_PATH.into());
    let metadata: PolicyV2 = image(source, &policy_target, side)?
        .map(decode)
        .transpose()?
        .unwrap_or_else(store_v2::empty_policy_v2);
    validate_policy_v2(&metadata)?;
    let opposite = if side == SideV2::Before {
        SideV2::After
    } else {
        SideV2::Before
    };
    let identities: PolicyV2 = if metadata.channels.is_empty() {
        image(source, &policy_target, opposite)?
            .map(decode)
            .transpose()?
            .unwrap_or_else(store_v2::empty_policy_v2)
    } else {
        metadata.clone()
    };
    // An unchanged mapping file is omitted by the image transaction. An
    // anchor-only source needs no additional mapping authority: provider IDs
    // are exact metadata bindings. Unknown/nonanchor rows below still fail;
    // never borrow a later mapping file or allocate historical identities.
    let mapping_bytes = source
        .images
        .iter()
        .find(|r| r.target == TargetV2::Manager(store_v2::MAPPING_PATH.into()))
        .and_then(|r| {
            if side == SideV2::Before {
                r.before.as_deref()
            } else {
                r.after.as_deref()
            }
        });
    let mappings: MappingFileV2 = mapping_bytes
        .map(decode)
        .transpose()?
        .unwrap_or(MappingFileV2 {
            schema_version: 2,
            mappings: vec![],
        });
    let ledger: Option<store_v2::LedgerFileV2> = image(
        source,
        &TargetV2::Manager(store_v2::LEDGER_PATH.into()),
        side,
    )?
    .map(decode)
    .transpose()?;
    let library = source
        .images
        .iter()
        .find(|r| r.target == TargetV2::Manager(store_v2::LIBRARY_PATH.into()))
        .and_then(|r| {
            if side == SideV2::Before {
                r.before.as_deref()
            } else {
                r.after.as_deref()
            }
        })
        .map(decode::<LibraryV2>)
        .transpose()?;
    if let Some(library) = &library {
        validate_library_v2(library)?;
    }
    let bindings = identities
        .channels
        .iter()
        .filter(|c| raw["providers"].get(&c.connection.provider_id).is_some())
        .map(|c| ProviderBindingV2 {
            channel_id: c.id.clone(),
            provider_id: c.connection.provider_id.clone(),
        })
        .collect::<Vec<_>>();
    for (provider, row) in raw["providers"].as_object().into_iter().flatten() {
        if !protected(provider, row)
            && !bindings.iter().any(|b| b.provider_id == *provider)
            && !mappings.mappings.iter().any(|m| m.provider_id == *provider)
        {
            return Err(fail("model_history_identity_missing"));
        }
    }
    let mut policy = provider_import::read_policy_v2(
        &raw,
        &digest(raw_bytes),
        &metadata,
        ledger.as_ref().map(|l| &l.ledger),
        &mappings,
        &bindings,
    )?;
    policy.recovery_id = None;
    policy.pending_removals.clear();
    for r in policy_refs(&policy) {
        revision(current, r)?;
        if let Some(library) = &library {
            if revision(library, r)? != revision(current, r)? {
                return Err(fail("model_history_dependency_mismatch"));
            }
        }
    }
    Ok((policy, library))
}
fn policy_refs(policy: &PolicyV2) -> Vec<&RefV2> {
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
pub fn list(d: &Path, h: &Path) -> Result<Vec<HistoryEntryV2>> {
    with_state(d, h, None, |_, state, _| {
        let mut out = vec![];
        for entry in images::list(d, h)? {
            if !entry.has_official_config {
                continue;
            }
            let source = images::read(d, h, &entry.id)?;
            let before = side_policy(&source, SideV2::Before, &state.library);
            let after = side_policy(&source, SideV2::After, &state.library);
            let labels = before
                .as_ref()
                .ok()
                .into_iter()
                .chain(after.as_ref().ok())
                .flat_map(|(p, _)| p.channels.iter().map(|c| c.label.clone()))
                .collect::<BTreeSet<_>>();
            out.push(HistoryEntryV2 {
                id: entry.id,
                created_at: entry.created_at,
                phase: entry.phase,
                channel_labels: labels.into_iter().collect(),
                recoverable: before.is_ok() && after.is_ok(),
            });
        }
        Ok(out)
    })
}
fn redact(v: &mut Value) {
    match v {
        Value::Object(o) => {
            for key in ["credentialRef", "importedConfig", "sourceReadId"] {
                o.remove(key);
            }
            for v in o.values_mut() {
                redact(v);
            }
        }
        Value::Array(a) => {
            for v in a {
                redact(v)
            }
        }
        _ => {}
    }
}
fn same_channel(a: Option<&ChannelV2>, b: Option<&ChannelV2>) -> Result<bool> {
    match (a, b) {
        (Some(a), Some(b)) => {
            let mut left = store_v2::empty_policy_v2();
            left.channels.push(a.clone());
            let mut right = store_v2::empty_policy_v2();
            right.channels.push(b.clone());
            Ok(policy_semantic_digest_v2(&left)? == policy_semantic_digest_v2(&right)?)
        }
        (None, None) => Ok(true),
        _ => Ok(false),
    }
}
fn changes(before: &PolicyV2, after: &PolicyV2) -> Result<Vec<ChangeFact>> {
    let mut out = vec![];
    let ids = before
        .channels
        .iter()
        .chain(&after.channels)
        .map(|c| &c.id)
        .collect::<BTreeSet<_>>();
    for id in ids {
        let b = before.channels.iter().find(|c| &c.id == id);
        let a = after.channels.iter().find(|c| &c.id == id);
        if !same_channel(b, a)? {
            let mut bv = serde_json::to_value(b).map_err(json_error)?;
            let mut av = serde_json::to_value(a).map_err(json_error)?;
            redact(&mut bv);
            redact(&mut av);
            out.push(ChangeFact {
                field: format!("channel/{id}"),
                before: bv,
                after: av,
            });
        }
    }
    if before.template_refs != after.template_refs {
        out.push(ChangeFact {
            field: "globalTemplateRefs".into(),
            before: json!(before.template_refs),
            after: json!(after.template_refs),
        });
    }
    if before.default_channel_id != after.default_channel_id {
        out.push(ChangeFact {
            field: "defaultChannelId".into(),
            before: json!(before.default_channel_id),
            after: json!(after.default_channel_id),
        });
    }
    let before_order = before.channels.iter().map(|c| &c.id).collect::<Vec<_>>();
    let after_order = after.channels.iter().map(|c| &c.id).collect::<Vec<_>>();
    if before_order != after_order {
        out.push(ChangeFact {
            field: "channelOrder".into(),
            before: json!(before_order),
            after: json!(after_order),
        });
    }
    Ok(out)
}
fn deterministic_id(bytes: &[u8]) -> String {
    let value = digest(bytes);
    let mut raw = [0u8; 16];
    for (i, v) in raw.iter_mut().enumerate() {
        *v = u8::from_str_radix(&value[7 + i * 2..9 + i * 2], 16).unwrap();
    }
    raw[6] = (raw[6] & 15) | 64;
    raw[8] = (raw[8] & 63) | 128;
    uuid::Uuid::from_bytes(raw).to_string()
}
pub fn prepare(d: &Path, h: &Path, request: HistoryPrepareV2) -> Result<HistoryPlanV2> {
    let expected = request.expected.clone();
    with_state(d, h, Some(&expected), |_, state, raw| {
        let draft = state.draft.as_ref().unwrap();
        if draft.policy.recovery_id.is_some()
            || !draft.credential_edits.is_empty()
            || !draft.policy.pending_removals.is_empty()
        {
            return Err(fail("model_history_draft_conflict"));
        }
        let source = images::read(d, h, &request.source_id)?;
        let (before, _) = side_policy(&source, SideV2::Before, &state.library)?;
        let (after, _) = side_policy(&source, SideV2::After, &state.library)?;
        let (target, source_library) = side_policy(&source, request.side, &state.library)?;
        let mut affected = BTreeSet::new();
        for channel in before.channels.iter().chain(&after.channels) {
            if !same_channel(
                before.channels.iter().find(|c| c.id == channel.id),
                after.channels.iter().find(|c| c.id == channel.id),
            )? {
                affected.insert(channel.id.clone());
            }
        }
        let selected = if request.scope == ScopeV2::All {
            draft
                .policy
                .channels
                .iter()
                .chain(&target.channels)
                .map(|c| c.id.clone())
                .collect()
        } else {
            affected
        };
        let baseline = draft.observed_baseline.as_ref().unwrap();
        let mut policy = draft.policy.clone();
        for id in &selected {
            let old = baseline.channels.iter().find(|c| &c.id == id);
            let now = draft.policy.channels.iter().find(|c| &c.id == id);
            let next = target.channels.iter().find(|c| &c.id == id);
            if !same_channel(now, old)? && !same_channel(next, now)? {
                return Err(fail("model_history_draft_conflict"));
            }
        }
        policy.channels.retain(|c| !selected.contains(&c.id));
        for c in target.channels.iter().filter(|c| selected.contains(&c.id)) {
            let mut c = c.clone();
            let current = baseline.channels.iter().find(|v| v.id == c.id);
            c.connection.imported_config = None;
            c.connection.source_read_id = None;
            if let Some(current) = current {
                c.connection.provider_id = current.connection.provider_id.clone();
            }
            // Historical secret refs never grant current authority. Current local
            // credentials survive only the exact current endpoint direction.
            let same = current.is_some_and(|v| {
                v.connection.nodes == c.connection.nodes
                    && v.connection.auth_mode == c.connection.auth_mode
            });
            c.connection.credential_ref = if same {
                current.and_then(|v| v.connection.credential_ref.clone())
            } else {
                None
            };
            if !same && current.is_some() {
                return Err(fail("model_history_credential_direction_review"));
            }
            if current.is_none() {
                c.connection.enabled = false;
            }
            policy.channels.push(c);
        }
        let order = if request.scope == ScopeV2::All {
            &target.channels
        } else {
            &draft.policy.channels
        };
        policy.channels.sort_by_key(|c| {
            order
                .iter()
                .position(|v| v.id == c.id)
                .unwrap_or(usize::MAX)
        });
        if before.template_refs != after.template_refs || request.scope == ScopeV2::All {
            policy.template_refs = target.template_refs.clone();
        }
        if request.restore_defaults {
            policy.default_channel_id = target.default_channel_id.clone();
        }
        if policy
            .default_channel_id
            .as_ref()
            .is_some_and(|id| !policy.channels.iter().any(|c| &c.id == id))
        {
            policy.default_channel_id = None;
        }
        let mut defaults = None;
        let mut default_changes = vec![];
        if request.restore_defaults {
            let historical =
                source_library.ok_or_else(|| fail("model_history_defaults_missing"))?;
            let mut library = state.library.clone();
            for t in &historical.templates {
                let current = library
                    .templates
                    .iter_mut()
                    .find(|v| v.id == t.id)
                    .ok_or_else(|| fail("model_dependency_missing"))?;
                if let Some(id) = &t.default_revision {
                    let r = t
                        .revisions
                        .iter()
                        .find(|r| &r.id == id)
                        .ok_or_else(|| fail("model_dependency_missing"))?;
                    if !current.revisions.iter().any(|c| c == r) {
                        return Err(fail("model_history_dependency_mismatch"));
                    }
                }
                if current.default_revision != t.default_revision {
                    default_changes.push(ChangeFact {
                        field: format!("template/{}/defaultRevision", t.id),
                        before: json!(current.default_revision),
                        after: json!(t.default_revision),
                    });
                    current.default_revision = t.default_revision.clone();
                }
            }
            validate_library_v2(&library)?;
            defaults = Some(library);
        }
        let recovery_id = deterministic_id(&canonical_bytes(&request)?);
        policy.recovery_id = Some(recovery_id.clone());
        let mut candidate = draft.clone();
        candidate.policy = policy;
        let projected = super::export::project_trusted_draft(d, state, &candidate, raw)?;
        let policy = projected.intended;
        let target_digest = target_digest(&policy, defaults.as_ref())?;
        let view = HistoryPlanV2 {
            recovery_id: recovery_id.clone(),
            source_id: request.source_id.clone(),
            side: request.side,
            scope: request.scope,
            restore_defaults: request.restore_defaults,
            changes: changes(&draft.policy, &policy)?,
            default_changes,
            channel_labels: policy
                .channels
                .iter()
                .filter(|c| selected.contains(&c.id))
                .map(|c| c.label.clone())
                .collect(),
            notices: vec![
                "仅加入恢复草稿；需核对后显式保存，实际配置与模版库尚未写入。".into(),
                "仅支持本版事务精确镜像；不恢复历史凭据、客户端状态或虚构保存时间。".into(),
            ],
            requires_runtime_review: true,
            target_digest,
        };
        if path(d, &recovery_id)?.exists() {
            let record = read_record(d, &recovery_id)?;
            if record.request != request
                || record.source_digest != source.source_digest
                || record.view.target_digest != view.target_digest
            {
                return Err(fail("model_history_plan_conflict"));
            }
            return Ok(record.view);
        }
        persist(
            d,
            &Record {
                schema_version: 2,
                request,
                view: view.clone(),
                source_digest: source.source_digest,
                policy,
                defaults,
                accepted_base: None,
            },
        )?;
        Ok(view)
    })
}
pub fn accept(d: &Path, h: &Path, recovery_id: &str) -> Result<FileSnapshotV2> {
    let record = read_record(d, recovery_id)?;
    let observed = StoreV2::new(d).snapshot()?;
    let replay = observed
        .draft
        .as_ref()
        .filter(|draft| {
            draft.policy == record.policy
                && draft.base.draft_id == record.request.expected.draft_id
                && draft.base.draft_generation
                    == record.request.expected.draft_generation.saturating_add(1)
        })
        .map(|draft| draft.base.clone());
    let expected = record
        .accepted_base
        .clone()
        .or(replay)
        .unwrap_or_else(|| record.request.expected.clone());
    with_state(d, h, Some(&expected), |locked, state, _| {
        let mut record = read_record(d, recovery_id)?;
        let source = images::read(d, h, &record.request.source_id)?;
        if source.source_digest != record.source_digest {
            return Err(fail("model_history_source_stale"));
        }
        let current = state.draft.as_ref().unwrap();
        if let Some(base) = &record.accepted_base {
            if &current.base != base || current.policy != record.policy {
                return Err(fail("model_history_stale"));
            }
            let (_, bytes) = read_raw(h)?;
            return view(
                locked.snapshot_unlocked()?,
                source_digest(&bytes),
                root_revision(d, h)?,
            );
        }
        if current.policy == record.policy
            && current.base.draft_id == record.request.expected.draft_id
            && current.base.draft_generation
                == record.request.expected.draft_generation.saturating_add(1)
        {
            record.accepted_base = Some(current.base.clone());
            persist(d, &record)?;
            let (_, bytes) = read_raw(h)?;
            return view(
                locked.snapshot_unlocked()?,
                source_digest(&bytes),
                root_revision(d, h)?,
            );
        }
        if current.base != record.request.expected || !current.credential_edits.is_empty() {
            return Err(fail("model_history_stale"));
        }
        let mut draft = current.clone();
        draft.policy = record.policy.clone();
        draft.base.draft_generation = draft
            .base
            .draft_generation
            .checked_add(1)
            .ok_or_else(|| error("model_limit"))?;
        let accepted = locked.stage_draft_unlocked(Some(&current.base), draft)?;
        record.accepted_base = Some(accepted.base);
        persist(d, &record)?;
        let (_, bytes) = read_raw(h)?;
        view(
            locked.snapshot_unlocked()?,
            source_digest(&bytes),
            root_revision(d, h)?,
        )
    })
}
/// Called under the actual save's config/model locks before any transform or
/// credential action. Recovery authority cannot be combined with new edits.
pub(super) fn validate_save(
    d: &Path,
    h: &Path,
    state: &StoreSnapshotV2,
    draft: &DraftV2,
    request: &SaveRequestV2,
) -> Result<Option<LibraryV2>> {
    let Some(id) = &draft.policy.recovery_id else {
        return Ok(None);
    };
    let record = read_record(d, id)?;
    if record.accepted_base.as_ref() != Some(&draft.base)
        || record.policy != draft.policy
        || !request.edits.is_empty()
        || !request.channel_intents.is_empty()
        || request.scope_channel_id.is_some()
        || !draft.credential_edits.is_empty()
    {
        return Err(fail("model_history_save_mixed_intent"));
    }
    if images::read(d, h, &record.request.source_id)?.source_digest != record.source_digest
        || record.request.expected.library_digest != state.digests.library
    {
        return Err(fail("model_history_source_stale"));
    }
    for r in policy_refs(&record.policy) {
        revision(&state.library, r)?;
    }
    Ok(record.defaults)
}
#[cfg(test)]
#[path = "config_file_v2_history_tests.rs"]
mod tests;
