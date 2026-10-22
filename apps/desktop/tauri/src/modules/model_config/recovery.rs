//! Successful local application snapshots restore into a new draft. Runtime
//! compensation and client-loaded evidence remain separate operations.
use super::{
    schema::*,
    store::{sha256, validate_library, ModelStore},
};
use crate::{infrastructure::atomic_write::atomic_write, modules::backup::safety};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Baseline {
    pub library: Library,
    pub policy: Policy,
    pub digest: String,
}
impl Baseline {
    pub fn capture_runtime(
        snapshot: &Snapshot,
        default_provider: Option<&str>,
    ) -> ModelResult<Self> {
        let mut baseline = Self::capture(snapshot)?;
        if baseline.policy.default_channel_id.is_none() {
            baseline.policy.default_channel_id = baseline
                .policy
                .channels
                .iter()
                .find(|c| Some(c.connection.provider_id.as_str()) == default_provider)
                .map(|c| c.id.clone());
        }
        baseline.digest = sha256(&serde_json::to_vec(&(&baseline.library, &baseline.policy))?);
        Ok(baseline)
    }
    pub fn capture(snapshot: &Snapshot) -> ModelResult<Self> {
        Ok(Self {
            library: snapshot.library.clone(),
            policy: snapshot.policy.clone(),
            digest: sha256(&serde_json::to_vec(&(&snapshot.library, &snapshot.policy))?),
        })
    }
    pub fn validate(&self) -> ModelResult<()> {
        validate_library(&self.library)?;
        if self.digest != sha256(&serde_json::to_vec(&(&self.library, &self.policy))?) {
            return Err(invalid("recovery_digest", "恢复基线摘要不同，未采用"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Before,
    After,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Affected,
    All,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub source_id: String,
    pub side: Side,
    pub scope: Scope,
    pub restore_defaults: bool,
    pub expected_library: String,
    pub expected_policy: String,
    pub expected_workspace: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Preview {
    pub id: String,
    pub source_id: String,
    pub side: Side,
    pub scope: Scope,
    pub restore_defaults: bool,
    pub changes: Vec<ChangeFact>,
    pub default_changes: Vec<ChangeFact>,
    pub channel_labels: Vec<String>,
    pub notices: Vec<String>,
    #[serde(default)]
    pub requires_runtime_review: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Plan {
    request: Request,
    view: Preview,
    library: Library,
    policy: Policy,
    digest: String,
    completed: bool,
}
fn path(root: &Path, id: &str) -> ModelResult<PathBuf> {
    require_uuid(id)?;
    let path = root
        .join("manager-state/model-recovery-plans")
        .join(format!("{id}.json"));
    safety::check_path(root, &path, true)?;
    Ok(path)
}
fn check_expected(local: &Snapshot, request: &Request) -> ModelResult<()> {
    if local.library_digest != request.expected_library
        || local.policy_digest != request.expected_policy
        || local.workspace_digest != request.expected_workspace
    {
        return Err(invalid(
            "stale_baseline",
            "恢复预览后本机配置或草稿改变，请重新比较",
        ));
    }
    Ok(())
}
fn refs(policy: &Policy) -> Vec<&TemplateRef> {
    policy
        .template_refs
        .iter()
        .chain(
            policy
                .channels
                .iter()
                .flat_map(|c| c.template_refs.iter().chain(c.model_refs.values())),
        )
        .collect()
}
fn merge_dependencies(
    current: &Library,
    historical: &Library,
    policy: &Policy,
    defaults: bool,
) -> ModelResult<(Library, Vec<ChangeFact>)> {
    let mut merged = current.clone();
    let mut default_changes = vec![];
    let wanted = refs(policy)
        .into_iter()
        .map(|r| (r.template_id.clone(), r.revision_id.clone()))
        .collect::<BTreeSet<_>>();
    for (template_id, revision_id) in &wanted {
        let old = historical
            .templates
            .iter()
            .find(|t| &t.id == template_id)
            .or_else(|| current.templates.iter().find(|t| &t.id == template_id))
            .ok_or_else(|| invalid("recovery_dependency", "恢复快照缺少模版"))?;
        let revision = old
            .revisions
            .iter()
            .find(|r| &r.id == revision_id)
            .or_else(|| {
                current
                    .templates
                    .iter()
                    .find(|t| &t.id == template_id)
                    .and_then(|t| t.revisions.iter().find(|r| &r.id == revision_id))
            })
            .ok_or_else(|| invalid("recovery_dependency", "恢复快照缺少精确版本"))?;
        if let Some(existing) = merged.templates.iter_mut().find(|t| &t.id == template_id) {
            if let Some(current) = existing.revisions.iter().find(|r| &r.id == revision_id) {
                if current != revision {
                    return Err(invalid(
                        "revision_conflict",
                        "恢复版本身份与本机内容不同，未覆盖",
                    ));
                }
            } else {
                existing.revisions.push(revision.clone());
            }
        } else {
            let mut template = old.clone();
            template.default_revision = None;
            template
                .revisions
                .retain(|r| wanted.contains(&(template.id.clone(), r.id.clone())));
            merged.templates.push(template);
        }
    }
    if defaults {
        for old in &historical.templates {
            if !merged.templates.iter().any(|t| t.id == old.id) && old.default_revision.is_some() {
                let mut template = old.clone();
                template
                    .revisions
                    .retain(|r| Some(&r.id) == old.default_revision.as_ref());
                template.default_revision = None;
                merged.templates.push(template);
            }
            let Some(local) = merged.templates.iter_mut().find(|t| t.id == old.id) else {
                continue;
            };
            if local.default_revision == old.default_revision {
                continue;
            }
            if let Some(id) = &old.default_revision {
                let revision = old
                    .revisions
                    .iter()
                    .find(|r| &r.id == id)
                    .ok_or_else(|| invalid("recovery_dependency", "历史默认版本不存在"))?;
                if let Some(current) = local.revisions.iter().find(|r| &r.id == id) {
                    if current != revision {
                        return Err(invalid("revision_conflict", "历史默认版本与本机冲突"));
                    }
                } else {
                    local.revisions.push(revision.clone());
                }
            }
            default_changes.push(ChangeFact {
                field: format!("default/{}", old.id),
                before: serde_json::to_value(&local.default_revision)?,
                after: serde_json::to_value(&old.default_revision)?,
            });
            local.default_revision = old.default_revision.clone();
        }
    }
    validate_library(&merged)?;
    for reference in refs(policy) {
        super::matching::revision(&merged, reference)?;
    }
    Ok((merged, default_changes))
}
fn channel_diff(current: &Policy, target: &Policy) -> ModelResult<Vec<ChangeFact>> {
    let mut changes = vec![];
    let ids = current
        .channels
        .iter()
        .chain(&target.channels)
        .map(|c| &c.id)
        .collect::<BTreeSet<_>>();
    for id in ids {
        let before = current.channels.iter().find(|c| &c.id == id);
        let after = target.channels.iter().find(|c| &c.id == id);
        if before == after {
            continue;
        }
        if let (Some(before), Some(after)) = (before, after) {
            let b = serde_json::to_value(before)?;
            let a = serde_json::to_value(after)?;
            for key in b.as_object().unwrap().keys().filter(|k| k.as_str() != "id") {
                if b[key] != a[key] {
                    changes.push(ChangeFact {
                        field: format!("channel/{id}/{key}"),
                        before: b[key].clone(),
                        after: a[key].clone(),
                    });
                }
            }
        } else {
            changes.push(ChangeFact {
                field: format!("channel/{id}"),
                before: serde_json::to_value(before)?,
                after: serde_json::to_value(after)?,
            });
        }
    }
    if current.default_channel_id != target.default_channel_id {
        changes.push(ChangeFact {
            field: "defaultChannelId".into(),
            before: serde_json::to_value(&current.default_channel_id)?,
            after: serde_json::to_value(&target.default_channel_id)?,
        });
    }
    if current.template_refs != target.template_refs {
        changes.push(ChangeFact {
            field: "globalTemplateRefs".into(),
            before: serde_json::to_value(&current.template_refs)?,
            after: serde_json::to_value(&target.template_refs)?,
        });
    }
    let order = |policy: &Policy| {
        policy
            .channels
            .iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>()
    };
    if order(current) != order(target) {
        changes.push(ChangeFact {
            field: "channelOrder".into(),
            before: serde_json::to_value(order(current))?,
            after: serde_json::to_value(order(target))?,
        });
    }
    // Local credential slots are not values, but neither belong in preview IPC.
    for change in &mut changes {
        for value in [&mut change.before, &mut change.after] {
            redacted(value);
        }
    }
    Ok(changes)
}
fn redacted(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(object) => {
            if let Some(slot) = object.get_mut("credentialRef") {
                *slot = serde_json::json!(if slot.is_null() {
                    "未绑定"
                } else {
                    "本机安全引用"
                });
            }
            for child in object.values_mut() {
                redacted(child);
            }
        }
        serde_json::Value::Array(array) => {
            for child in array {
                redacted(child)
            }
        }
        _ => {}
    }
}
pub(super) fn prepare(
    root: &Path,
    baseline: &Baseline,
    after: &Policy,
    request: Request,
) -> ModelResult<Preview> {
    baseline.validate()?;
    let local = ModelStore::new(root).load()?;
    check_expected(&local, &request)?;
    let target = if request.side == Side::Before {
        &baseline.policy
    } else {
        after
    };
    let affected = baseline
        .policy
        .channels
        .iter()
        .chain(&after.channels)
        .filter(|c| {
            baseline.policy.channels.iter().find(|b| b.id == c.id)
                != after.channels.iter().find(|a| a.id == c.id)
        })
        .map(|c| c.id.clone())
        .collect::<BTreeSet<_>>();
    let mut intended = if request.scope == Scope::All {
        target.clone()
    } else {
        let mut policy = local.policy.clone();
        if baseline.policy.default_channel_id != after.default_channel_id {
            policy.default_channel_id = target.default_channel_id.clone();
        }
        policy.channels.retain(|c| !affected.contains(&c.id));
        policy.channels.extend(
            target
                .channels
                .iter()
                .filter(|c| affected.contains(&c.id))
                .cloned(),
        );
        if baseline.policy.template_refs != after.template_refs {
            policy.template_refs = target.template_refs.clone();
        }
        let order = target
            .channels
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>();
        if baseline
            .policy
            .channels
            .iter()
            .map(|c| &c.id)
            .collect::<Vec<_>>()
            != after.channels.iter().map(|c| &c.id).collect::<Vec<_>>()
        {
            let ranks = order
                .iter()
                .enumerate()
                .map(|(i, id)| (*id, i))
                .collect::<std::collections::BTreeMap<_, _>>();
            policy
                .channels
                .sort_by_key(|c| ranks.get(c.id.as_str()).copied().unwrap_or(usize::MAX));
        } else {
            let ranks = local
                .policy
                .channels
                .iter()
                .enumerate()
                .map(|(i, c)| (c.id.as_str(), i))
                .collect::<std::collections::BTreeMap<_, _>>();
            policy
                .channels
                .sort_by_key(|c| ranks.get(c.id.as_str()).copied().unwrap_or(usize::MAX));
        }
        policy
    };
    // Preserve independent uncommitted edits; overlapping edits require the user
    // to resolve the draft rather than accepting a stale destructive preview.
    if let Some(workspace) = &local.workspace {
        if workspace.base_policy != local.policy_digest {
            return Err(invalid("draft_conflict", "当前草稿基准已过期，请先处理"));
        }
        if !workspace.policy.pending_removals.is_empty() {
            return Err(invalid("draft_conflict","当前有首次读取渠道的待删除操作，请先保存或还原该操作后再生成历史恢复草稿；当前输入保持"));
        }
        for draft in &workspace.policy.channels {
            let committed = local.policy.channels.iter().find(|c| c.id == draft.id);
            if Some(draft) == committed {
                continue;
            }
            let recovered = intended.channels.iter().find(|c| c.id == draft.id);
            if recovered != committed && recovered != Some(draft) {
                return Err(invalid(
                    "draft_conflict",
                    "恢复范围与未保存渠道修改重叠，请先保留或还原该范围",
                ));
            }
            if let Some(index) = intended.channels.iter().position(|c| c.id == draft.id) {
                intended.channels[index] = draft.clone();
            } else {
                intended.channels.push(draft.clone());
            }
        }
        for committed in &local.policy.channels {
            if workspace
                .policy
                .channels
                .iter()
                .any(|c| c.id == committed.id)
            {
                continue;
            }
            if intended
                .channels
                .iter()
                .find(|c| c.id == committed.id)
                .is_some_and(|c| c != committed)
            {
                return Err(invalid("draft_conflict", "恢复范围与草稿删除重叠"));
            }
            intended.channels.retain(|c| c.id != committed.id);
        }
        // defaultChannelId 已退役：旧草稿的残留值不参与恢复冲突判定。
        if workspace.policy.template_refs != local.policy.template_refs {
            if intended.template_refs != local.policy.template_refs
                && intended.template_refs != workspace.policy.template_refs
            {
                return Err(invalid("draft_conflict", "全局采用与恢复重叠"));
            }
            intended.template_refs = workspace.policy.template_refs.clone();
        }
        let ids = |policy: &Policy| {
            policy
                .channels
                .iter()
                .map(|c| c.id.clone())
                .collect::<Vec<_>>()
        };
        if ids(&workspace.policy) != ids(&local.policy) {
            if ids(&intended) != ids(&local.policy) && ids(&intended) != ids(&workspace.policy) {
                return Err(invalid(
                    "draft_conflict",
                    "渠道顺序与恢复重叠，请先处理草稿",
                ));
            }
            let rank = ids(&workspace.policy);
            intended
                .channels
                .sort_by_key(|c| rank.iter().position(|id| id == &c.id).unwrap_or(usize::MAX));
        }
    }
    let (library, default_changes) = merge_dependencies(
        &local.library,
        &baseline.library,
        &intended,
        request.restore_defaults,
    )?;
    let changes = channel_diff(
        local
            .workspace
            .as_ref()
            .map(|w| &w.policy)
            .unwrap_or(&local.policy),
        &intended,
    )?;
    let view = Preview {
        id: uuid::Uuid::new_v4().to_string(),
        source_id: request.source_id.clone(),
        side: request.side,
        scope: request.scope,
        restore_defaults: request.restore_defaults,
        requires_runtime_review: true,
        changes,
        default_changes,
        channel_labels: intended
            .channels
            .iter()
            .filter(|c| request.scope == Scope::All || affected.contains(&c.id))
            .map(|c| c.label.clone())
            .collect(),
        notices: vec![
            "先放入渠道草稿；保存时重新读取官方运行态、检查漂移和目录预算。".into(),
            "本机连接与凭据引用需在保存前核对，历史客户端状态不迁移。".into(),
        ],
    };
    intended.recovery_id = Some(view.id.clone());
    let digest = sha256(&serde_json::to_vec(&(&library, &intended))?);
    atomic_write(
        &path(root, &view.id)?,
        &serde_json::to_vec(&Plan {
            request,
            view: view.clone(),
            library,
            policy: intended,
            digest,
            completed: false,
        })?,
        0o600,
    )?;
    Ok(view)
}
pub(super) fn selected_request(root: &Path, id: &str, policy: &Policy) -> ModelResult<Request> {
    let plan: Plan =
        serde_json::from_slice(&safety::read_file(&path(root, id)?, 128 * 1024 * 1024)?)?;
    if plan.view.id != id
        || !plan.completed
        || plan.digest != sha256(&serde_json::to_vec(&(&plan.library, &plan.policy))?)
        || policy.published() != plan.policy.published()
    {
        return Err(invalid(
            "recovery_candidate_changed",
            "恢复草稿已修改或尚未采用，请重新比较历史目标后再保存",
        ));
    }
    Ok(plan.request)
}
pub(super) fn accept(root: &Path, id: &str) -> ModelResult<Snapshot> {
    let file = path(root, id)?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&file),
        true,
    )?;
    let _guard = crate::infrastructure::locking::TargetFileLock::lock(&file)?;
    let mut plan: Plan = serde_json::from_slice(&safety::read_file(&file, 128 * 1024 * 1024)?)?;
    if plan.view.id != id
        || plan.digest != sha256(&serde_json::to_vec(&(&plan.library, &plan.policy))?)
    {
        return Err(invalid("recovery_digest", "恢复计划内容不符"));
    }
    let current = ModelStore::new(root).load()?;
    if plan.completed {
        return Ok(current);
    }
    let applied = current.library == plan.library
        && current.workspace.as_ref().is_some_and(|w| {
            w.base_policy == plan.request.expected_policy && w.policy == plan.policy
        });
    let result = if applied {
        current
    } else {
        check_expected(&current, &plan.request)?;
        ModelStore::new(root).accept_import(
            &plan.request.expected_library,
            &plan.request.expected_policy,
            &plan.request.expected_workspace,
            plan.library.clone(),
            Some(plan.policy.clone()),
        )?
    };
    plan.completed = true;
    atomic_write(&file, &serde_json::to_vec(&plan)?, 0o600)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    #[test]
    fn unrelated_later_revision_of_a_historical_template_remains_available() {
        let root = tempfile::tempdir().unwrap();
        let store = ModelStore::new(root.path());
        let local = store.load().unwrap();
        let mut content = TemplateContent::default();
        content.matching.include.push(Pattern {
            kind: MatchKind::Prefix,
            value: "sample".into(),
        });
        let first = store
            .save(SaveTemplate {
                expected_library: local.library_digest,
                template_id: None,
                base_revision: None,
                label: "同一模版".into(),
                content: content.clone(),
            })
            .unwrap();
        content.rules.context_window = Field::Custom(64000);
        let next = store
            .save(SaveTemplate {
                expected_library: first.snapshot.library_digest.clone(),
                template_id: Some(first.template_id.clone()),
                base_revision: Some(first.revision_id.clone()),
                label: "同一模版".into(),
                content,
            })
            .unwrap();
        let mut a = channel("a");
        a.template_refs.push(TemplateRef {
            template_id: first.template_id.clone(),
            revision_id: first.revision_id.clone(),
        });
        let mut b = channel("b");
        b.template_refs.push(TemplateRef {
            template_id: next.template_id.clone(),
            revision_id: next.revision_id.clone(),
        });
        let policy = Policy {
            channels: vec![a, b],
            ..Default::default()
        };
        let (merged, _) = merge_dependencies(
            &next.snapshot.library,
            &first.snapshot.library,
            &policy,
            false,
        )
        .unwrap();
        assert_eq!(merged, next.snapshot.library);
    }
    fn channel(name: &str) -> ChannelPolicy {
        let node = ServiceNode {
            id: uuid::Uuid::new_v4().to_string(),
            name: "主站".into(),
            url: "https://fixture.example.test/v1".into(),
        };
        ChannelPolicy {
            naming: Default::default(),
            id: uuid::Uuid::new_v4().to_string(),
            label: name.into(),
            template_refs: vec![],
            model_refs: BTreeMap::new(),
            connection: ChannelConnection {
                auth_mode: None,
                source_read_id: None,
                imported_config: None,
                provider_id: format!("fixture_{name}"),
                protocol: "openai-chat".into(),
                active_node_id: node.id.clone(),
                nodes: vec![node],
                credential_ref: Some(uuid::Uuid::new_v4().to_string()),
                enabled: true,
                allow_private_network: false,
            },
            selection: ModelSelection::All,
            model_order: vec![],
            discovered: vec![],
            defaults: Default::default(),
            overrides: Default::default(),
            family_overrides: Default::default(),
        }
    }
    fn commit(root: &Path, policy: Policy) -> Snapshot {
        let store = ModelStore::new(root);
        let local = store.load().unwrap();
        store
            .commit_runtime_policy(
                &local.library_digest,
                &local.policy_digest,
                &local.workspace_digest,
                policy,
            )
            .unwrap()
    }
    fn request(local: &Snapshot) -> Request {
        Request {
            source_id: uuid::Uuid::new_v4().to_string(),
            side: Side::Before,
            scope: Scope::Affected,
            restore_defaults: false,
            expected_library: local.library_digest.clone(),
            expected_policy: local.policy_digest.clone(),
            expected_workspace: local.workspace_digest.clone(),
        }
    }
    #[test]
    fn restores_only_affected_channels_to_draft_preserves_independent_edits_and_replay_does_not_replace_later_input(
    ) {
        let root = tempfile::tempdir().unwrap();
        let mut a = channel("a");
        let mut b = channel("b");
        let baseline = Baseline::capture(&commit(
            root.path(),
            Policy {
                channels: vec![a.clone(), b.clone()],
                ..Default::default()
            },
        ))
        .unwrap();
        a.defaults.context_window = Field::Custom(64000);
        let after = Policy {
            channels: vec![a.clone(), b.clone()],
            ..Default::default()
        };
        let local = commit(root.path(), after.clone());
        b.label = "独立人工修改".into();
        let mut draft = after.clone();
        draft.channels[1] = b.clone();
        let local = ModelStore::new(root.path())
            .stage_policy(&local.policy_digest, &local.workspace_digest, draft)
            .unwrap();
        let preview = prepare(root.path(), &baseline, &after, request(&local)).unwrap();
        assert!(preview.default_changes.is_empty());
        assert!(preview
            .changes
            .iter()
            .any(|c| c.field.ends_with("defaults")));
        assert!(!serde_json::to_string(&preview)
            .unwrap()
            .contains(a.connection.credential_ref.as_deref().unwrap()));
        let restored = accept(root.path(), &preview.id).unwrap();
        assert_eq!(restored.policy, after);
        let draft = &restored.workspace.as_ref().unwrap().policy;
        assert_eq!(draft.channels[0], baseline.policy.channels[0]);
        assert_eq!(draft.channels[1], b);
        let mut later = draft.clone();
        later.channels[0].label = "后来编辑".into();
        let later = ModelStore::new(root.path())
            .stage_policy(&restored.policy_digest, &restored.workspace_digest, later)
            .unwrap();
        assert_eq!(accept(root.path(), &preview.id).unwrap(), later);
    }
    #[test]
    fn overlapping_draft_and_changed_confirmation_are_refused_without_runtime_or_library_writes() {
        let root = tempfile::tempdir().unwrap();
        let mut a = channel("a");
        let baseline = Baseline::capture(&commit(
            root.path(),
            Policy {
                channels: vec![a.clone()],
                ..Default::default()
            },
        ))
        .unwrap();
        a.label = "成功保存".into();
        let after = Policy {
            channels: vec![a.clone()],
            ..Default::default()
        };
        let local = commit(root.path(), after.clone());
        let preview = prepare(root.path(), &baseline, &after, request(&local)).unwrap();
        a.label = "当前草稿".into();
        let draft = ModelStore::new(root.path())
            .stage_policy(
                &local.policy_digest,
                &local.workspace_digest,
                Policy {
                    channels: vec![a],
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(
            accept(root.path(), &preview.id).unwrap_err().code,
            "stale_baseline"
        );
        assert_eq!(
            prepare(root.path(), &baseline, &after, request(&draft))
                .unwrap_err()
                .code,
            "draft_conflict"
        );
        assert_eq!(ModelStore::new(root.path()).load().unwrap(), draft);
    }
    #[test]
    fn exact_snapshot_dependencies_can_return_without_discarding_new_templates_or_default_pointers()
    {
        let root = tempfile::tempdir().unwrap();
        let store = ModelStore::new(root.path());
        let new_template = |store: &ModelStore, label: &str| {
            let local = store.load().unwrap();
            let mut content = TemplateContent::default();
            content.matching.include.push(Pattern {
                kind: MatchKind::Prefix,
                value: label.into(),
            });
            store
                .save(SaveTemplate {
                    expected_library: local.library_digest,
                    template_id: None,
                    base_revision: None,
                    label: label.into(),
                    content,
                })
                .unwrap()
        };
        let first = new_template(&store, "old");
        store
            .set_default(
                &first.snapshot.library_digest,
                &first.template_id,
                &first.revision_id,
            )
            .unwrap();
        let mut a = channel("a");
        a.template_refs.push(TemplateRef {
            template_id: first.template_id.clone(),
            revision_id: first.revision_id.clone(),
        });
        let baseline = Baseline::capture(&commit(
            root.path(),
            Policy {
                channels: vec![a.clone()],
                ..Default::default()
            },
        ))
        .unwrap();
        a.template_refs.clear();
        let after = Policy {
            channels: vec![a],
            ..Default::default()
        };
        commit(root.path(), after.clone());
        let local = store.load().unwrap();
        store
            .set_default(
                &local.library_digest,
                &first.template_id,
                &first.revision_id,
            )
            .unwrap();
        // Simulate a missing former dependency after external/manual removal, while the snapshot retains exact content.
        let local = store.load().unwrap();
        let mut library = local.library.clone();
        library.templates.retain(|t| t.id != first.template_id);
        store
            .accept_import(
                &local.library_digest,
                &local.policy_digest,
                &local.workspace_digest,
                library,
                None,
            )
            .unwrap();
        let unrelated = new_template(&store, "unrelated");
        let local = store.load().unwrap();
        let preview = prepare(root.path(), &baseline, &after, request(&local)).unwrap();
        let result = accept(root.path(), &preview.id).unwrap();
        assert!(result
            .library
            .templates
            .iter()
            .any(|t| t.id == unrelated.template_id));
        let restored = result
            .library
            .templates
            .iter()
            .find(|t| t.id == first.template_id)
            .unwrap();
        assert_eq!(restored.revisions[0].id, first.revision_id);
        assert!(restored.default_revision.is_none());
        let mut req = request(&result);
        req.restore_defaults = true;
        req.side = Side::After;
        let preview = prepare(root.path(), &baseline, &after, req).unwrap();
        assert_eq!(preview.default_changes.len(), 1);
        let result = accept(root.path(), &preview.id).unwrap();
        assert_eq!(
            result
                .library
                .templates
                .iter()
                .find(|t| t.id == first.template_id)
                .unwrap()
                .default_revision
                .as_deref(),
            Some(first.revision_id.as_str())
        );
    }
    #[test]
    fn corrupted_baseline_or_plan_digest_is_rejected() {
        let root = tempfile::tempdir().unwrap();
        let local = ModelStore::new(root.path()).load().unwrap();
        let mut baseline = Baseline::capture(&local).unwrap();
        baseline.policy.channels.push(channel("changed"));
        assert_eq!(
            prepare(root.path(), &baseline, &local.policy, request(&local))
                .unwrap_err()
                .code,
            "recovery_digest"
        );
        let baseline = Baseline::capture(&local).unwrap();
        let preview = prepare(root.path(), &baseline, &local.policy, request(&local)).unwrap();
        let file = path(root.path(), &preview.id).unwrap();
        let mut plan: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
        plan["digest"] = serde_json::json!("bad");
        std::fs::write(file, serde_json::to_vec(&plan).unwrap()).unwrap();
        assert_eq!(
            accept(root.path(), &preview.id).unwrap_err().code,
            "recovery_digest"
        );
    }
}
