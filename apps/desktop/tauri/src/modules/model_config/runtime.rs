//! Prepared runtime plans bind public snapshots to local policy/library digests.
//! Public OpenCodex verbs own persistence; no direct config.json replacement.
use super::{
    compiler,
    schema::*,
    store::ModelStore,
    upstream::{self, UpstreamSnapshot, WriteOperation},
};
use crate::{
    infrastructure::{atomic_write::atomic_write, locking::TargetFileLock},
    modules::backup::{backup_file, safety, BackupAction},
    state::ProcessContext,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
const PLAN_DIR: &str = "manager-state/model-save-plans";
const OWNERSHIP: &str = "manager-state/model-runtime-ownership.json";
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HistoryRestoration {
    pub source_id: String,
    pub projection: super::runtime_history::Projection,
    pub authorizations: BTreeMap<String, Option<super::provider_auth::Backup>>,
    pub registration_models: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub directions: BTreeMap<String, super::provider_auth::Direction>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CredentialChange {
    provider: String,
    reference: String,
    before: super::management::KeyState,
    intended_id: String,
    intended_digest: String,
    observed: Option<super::management::KeyState>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemovedKey {
    provider: String,
    before: super::management::KeyState,
    reference: Option<String>,
    #[serde(default)]
    backup: Option<super::provider_auth::Backup>,
}
impl CredentialChange {
    fn intended(&self) -> super::management::KeyState {
        let mut next = self.before.clone();
        next.ids.insert(self.intended_id.clone());
        next.active_id = Some(self.intended_id.clone());
        next
    }
    fn compensation_owned(&self, current: &super::management::KeyState) -> bool {
        if current == &self.before {
            return true;
        }
        let Some(observed) = &self.observed else {
            return false;
        };
        let mut restoring = observed.clone();
        restoring.active_id = self.before.active_id.clone();
        current == observed || current == &restoring
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OwnedField {
    provider: String,
    key: String,
    model: String,
    original: Option<Value>,
    written: Value,
    #[serde(default)]
    removed: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimePlan {
    pub id: String,
    pub phase: String,
    pub expected_library: String,
    pub expected_policy: String,
    pub expected_workspace: String,
    pub policy: Policy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deferred_policy: Option<Policy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<SaveScope>,
    #[serde(default)]
    pub deferred_channels: Vec<String>,
    pub before: UpstreamSnapshot,
    pub next_providers: Value,
    pub next_order: Vec<String>,
    pub expected_slugs: BTreeSet<String>,
    pub runtime_executable: String,
    pub error: Option<String>,
    pub created_at: String,
    pub ownership: Vec<OwnedField>,
    pub previous_ownership: Vec<OwnedField>,
    pub disabled_before: Vec<String>,
    pub observed_providers: Value,
    pub observed_order: Vec<String>,
    pub observed_disabled: Vec<String>,
    #[serde(default)]
    pub credential_changes: Vec<CredentialChange>,
    #[serde(default)]
    pub execution_policy: Option<Policy>,
    #[serde(default)]
    pub route_changes: Vec<String>,
    #[serde(default)]
    pub recovery: Option<super::recovery::Baseline>,
    #[serde(default)]
    pub runtime_binding: Option<String>,
    #[serde(default)]
    pub removed_providers: Vec<String>,
    #[serde(default)]
    pub removed_keys: Vec<RemovedKey>,
    #[serde(default)]
    pub committed_snapshot: Option<UpstreamSnapshot>,
    #[serde(default)]
    pub authorization_before: BTreeMap<String, Option<super::provider_auth::Backup>>,
    #[serde(default)]
    pub authorization_after: BTreeMap<String, Option<super::provider_auth::Backup>>,
    #[serde(default)]
    pub history_restoration: Option<HistoryRestoration>,
    #[serde(default)]
    pub imported_authorizations: BTreeMap<String, super::provider_auth::Backup>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePlanView {
    pub id: String,
    pub phase: String,
    pub catalog_count: usize,
    pub current_catalog_count: usize,
    pub changed_channels: Vec<String>,
    pub deferred_channels: Vec<String>,
    pub error: Option<String>,
    pub protocol_groups: Vec<String>,
    pub route_changes: Vec<String>,
    pub removed_providers: Vec<String>,
    pub history_source: Option<String>,
    pub runtime_changes: Vec<ChangeFact>,
}
impl RuntimePlan {
    fn view(&self) -> SavePlanView {
        SavePlanView {
            id: self.id.clone(),
            phase: self.phase.clone(),
            catalog_count: self.expected_slugs.len(),
            current_catalog_count: self.before.catalog["models"]
                .as_array()
                .map(Vec::len)
                .unwrap_or_default(),
            changed_channels: self
                .policy
                .channels
                .iter()
                .filter(|c| self.scope.as_ref().is_none_or(|scope|scope.include_shared||scope.channel_ids.contains(&c.id)))
                .map(|c| c.label.clone())
                .chain(self.recovery.iter().flat_map(|baseline|baseline.policy.channels.iter()).filter(|old|!self.policy.channels.iter().any(|c|c.id==old.id)&&self.scope.as_ref().is_none_or(|scope|scope.channel_ids.contains(&old.id))).map(|old|old.label.clone()))
                .collect(),
            deferred_channels: self.deferred_channels.clone(),
            error: self.error.clone(),
            protocol_groups: self
                .execution_policy
                .as_ref()
                .map(|policy| {
                    policy
                        .channels
                        .iter()
                        .filter(|c| {
                            !self.policy.channels.iter().any(|original| {
                                original.connection.provider_id == c.connection.provider_id
                            })
                        })
                        .map(|c| {
                            format!(
                                "{} · {} → {}",
                                c.label, c.connection.protocol, c.connection.provider_id
                            )
                        })
                        .collect()
                })
                .unwrap_or_default(),
            route_changes: self.route_changes.clone(),
            removed_providers: self.removed_providers.clone(),
            history_source: self
                .history_restoration
                .as_ref()
                .map(|h| h.source_id.clone()),
            runtime_changes: self
                .history_restoration
                .as_ref()
                .map(|history| {
                    let mut values = vec![];
                    for provider in &history.projection.affected {
                        let before = &self.before.providers["providers"][provider];
                        let after = &self.next_providers["providers"][provider];
                        if before != after {
                            if before.is_object() && after.is_object(){
                                let fields=before.as_object().unwrap().keys().chain(after.as_object().unwrap().keys()).collect::<BTreeSet<_>>();
                                for field in fields {
                                    if before.get(field)!=after.get(field){values.push(ChangeFact{field:format!("provider/{provider}/{field}"),before:before.get(field).cloned().unwrap_or(Value::Null),after:after.get(field).cloned().unwrap_or(Value::Null)});}
                                }
                            }else{
                                values.push(ChangeFact{field:format!("provider/{provider}"),before:before.clone(),after:after.clone()});
                            }
                        }
                    }
                    for (provider,target) in &history.authorizations {
                        let current=self.authorization_before.get(provider).and_then(Option::as_ref);
                        if current.map(|b|b.digest.as_str())!=target.as_ref().map(|b|b.digest.as_str()) {
                            let summary=|backup:Option<&super::provider_auth::Backup>|json!({"Key数量":backup.map(|b|b.state.ids.len()).unwrap_or(0),"活动项":backup.and_then(|b|b.state.active_id.clone()),"范围":"Key池原顺序、标签与自定义认证头"});
                            values.push(ChangeFact{field:format!("授权选择/{provider}"),before:summary(current),after:summary(target.as_ref())});
                        }
                    }
                    if self.before.providers["defaultProvider"]
                        != self.next_providers["defaultProvider"]
                    {
                        values.push(ChangeFact {
                            field: "defaultProvider".into(),
                            before: self.before.providers["defaultProvider"].clone(),
                            after: self.next_providers["defaultProvider"].clone(),
                        });
                    }
                    if self.before.order["pickerOrder"] != json!(self.next_order) {
                        values.push(ChangeFact {
                            field: "pickerOrder".into(),
                            before: self.before.order["pickerOrder"].clone(),
                            after: json!(self.next_order),
                        });
                    }
                    if self.disabled_before != history.projection.disabled {
                        values.push(ChangeFact {
                            field: "disabledModels".into(),
                            before: json!(self.disabled_before),
                            after: json!(history.projection.disabled),
                        });
                    }
                    values
                })
                .unwrap_or_default(),
        }
    }
}
impl RuntimePlan {
    fn execution_policy(&self) -> &Policy {
        self.execution_policy.as_ref().unwrap_or(&self.policy)
    }
}
pub(super) fn runtime_binding(context: &ProcessContext) -> ModelResult<String> {
    Ok(super::store::sha256(&serde_json::to_vec(&(
        context.executable()?,
        &context.opencodex_home,
    ))?))
}
pub(super) fn credential_runtime_changed(plan: &RuntimePlan) -> bool {
    plan.credential_changes
        .iter()
        .any(|c| c.before != c.intended())
        || !plan.imported_authorizations.is_empty()
        || plan.history_restoration.as_ref().is_some_and(|history| {
            history.authorizations.iter().any(|(provider, target)| {
                plan.authorization_before
                    .get(provider)
                    .and_then(Option::as_ref)
                    .map(|b| &b.digest)
                    != target.as_ref().map(|b| &b.digest)
            })
        })
}
pub(super) async fn verify_committed_credentials(
    root: &Path,
    plan: &RuntimePlan,
    context: &ProcessContext,
) -> ModelResult<bool> {
    for backup in plan.imported_authorizations.values() {
        super::provider_auth::verify(root, context, backup).await?;
    }
    if let Some(history) = &plan.history_restoration {
        for (provider, backup) in &history.authorizations {
            if plan.next_providers["providers"].get(provider).is_none() {
                continue;
            }
            match backup {
                Some(backup) => super::provider_auth::verify(root, context, backup).await?,
                None => super::provider_auth::verify_absent(context, provider)?,
            }
        }
    }
    if plan.credential_changes.is_empty() {
        return Ok(true);
    }
    let management = super::management::Management::bind(context).await?;
    for change in &plan.credential_changes {
        if management.key_state(&change.provider).await? != change.intended() {
            return Ok(false);
        }
    }
    Ok(true)
}
async fn removed_key_states(
    root: &Path,
    plan: &RuntimePlan,
    context: &ProcessContext,
    providers: &Value,
) -> ModelResult<bool> {
    if plan.removed_keys.is_empty() {
        return Ok(true);
    }
    let management = super::management::Management::bind(context).await?;
    let mut matches = true;
    for removed in &plan.removed_keys {
        if let Some(backup) = &removed.backup {
            super::provider_auth::check_backup(root, backup)?;
            if providers["providers"].get(&removed.provider).is_none() {
                matches = false;
                continue;
            }
            if !super::provider_auth::inspect_target(root, context, backup)? {
                matches = false;
            }
            continue;
        }
        if let (Some(reference), Some(active)) = (&removed.reference, &removed.before.active_id) {
            let secret = super::discovery::credential(reference)?;
            let hash = super::store::sha256(secret.trim().as_bytes());
            if &hash.trim_start_matches("sha256:")[..8] != active {
                return Err(invalid("removal_key_backup", "恢复Key不可核对，未继续"));
            }
        }
        if providers["providers"].get(&removed.provider).is_none() {
            matches = false;
            continue;
        }
        let current = management.key_state(&removed.provider).await?;
        if removed.before.ids.is_empty() && removed.reference.is_none() {
            super::provider_auth::verify_absent(context, &removed.provider)?;
        }
        if current == removed.before {
            continue;
        }
        if !current.ids.is_empty() {
            return Err(invalid(
                "credential_drift",
                "删除目标的Key池出现后来修改，未覆盖",
            ));
        }
        matches = false;
    }
    Ok(matches)
}

fn path(root: &Path, id: &str) -> ModelResult<PathBuf> {
    require_uuid(id)?;
    let p = root.join(PLAN_DIR).join(format!("{id}.json"));
    safety::check_path(root, &p, true)?;
    Ok(p)
}
fn save(root: &Path, plan: &RuntimePlan) -> ModelResult<()> {
    let path = path(root, &plan.id)?;
    let bytes = serde_json::to_vec(plan)?;
    atomic_write(&path, &bytes, 0o600)?;
    if safety::read_file(&path, 256 * 1024 * 1024)? != bytes {
        return Err(invalid("plan_readback", "保存计划写后读回不一致，未继续"));
    }
    Ok(())
}
pub(super) fn load(root: &Path, id: &str) -> ModelResult<RuntimePlan> {
    let plan: RuntimePlan =
        serde_json::from_slice(&safety::read_file(&path(root, id)?, 256 * 1024 * 1024)?)?;
    if plan.id != id {
        return Err(invalid("plan_identity", "保存计划的完整身份不一致"));
    }
    if let Some(baseline) = &plan.recovery {
        baseline.validate()?;
    }
    if let Some(snapshot) = &plan.committed_snapshot {
        if snapshot.providers != plan.next_providers
            || snapshot.order["pickerOrder"] != serde_json::to_value(&plan.next_order)?
            || catalog_slugs(snapshot)? != plan.expected_slugs
        {
            return Err(invalid(
                "plan_readback",
                "成功保存快照与冻结目标不同，未采用历史",
            ));
        }
    }
    Ok(plan)
}
pub(super) fn execution_lock(root: &Path) -> ModelResult<TargetFileLock> {
    let target = root.join("manager-state/model-runtime-transaction");
    safety::check_path(root, &target, true)?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&target),
        true,
    )?;
    Ok(TargetFileLock::lock(&target)?)
}
pub(super) fn plan_ids(root: &Path) -> ModelResult<Vec<String>> {
    let directory = root.join(PLAN_DIR);
    safety::check_path(root, &directory, true)?;
    if !directory.exists() {
        return Ok(vec![]);
    }
    let mut ids = vec![];
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let Some(id) = name.strip_suffix(".json") else {
            continue;
        };
        if uuid::Uuid::parse_str(id).is_err() {
            continue;
        }
        require_uuid(id)?;
        if !entry.file_type()?.is_file() {
            return Err(invalid("plan_path", "保存计划路径不是普通文件"));
        }
        ids.push(id.into());
        if ids.len() > 10000 {
            return Err(invalid("plan_limit", "保存计划过多，需核对后再继续"));
        }
    }
    Ok(ids)
}
pub fn latest(root: &Path) -> ModelResult<Option<SavePlanView>> {
    let mut latest = None;
    for id in plan_ids(root)? {
        let plan = load(root, &id)?;
        if ["prepared", "compensated"].contains(&plan.phase.as_str()) {
            continue;
        }
        let created = chrono::DateTime::parse_from_rfc3339(&plan.created_at)
            .map_err(|_| invalid("plan_time", "保存计划时间无效"))?;
        if latest.as_ref().is_none_or(|(time, _)| &created > time) {
            latest = Some((created, plan.view()));
        }
    }
    Ok(latest.map(|(_, view)| view))
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub created_at: String,
    pub phase: String,
    pub channel_labels: Vec<String>,
    pub recoverable: bool,
}
pub fn history(root: &Path) -> ModelResult<Vec<HistoryEntry>> {
    let mut values = vec![];
    for id in plan_ids(root)? {
        let plan = load(root, &id)?;
        if plan.phase == "prepared" {
            continue;
        }
        if let Some(baseline) = &plan.recovery {
            baseline.validate()?;
        }
        values.push(HistoryEntry {
            id,
            created_at: plan.created_at,
            phase: plan.phase.clone(),
            channel_labels: plan
                .policy
                .channels
                .iter()
                .map(|c| c.label.clone())
                .collect(),
            recoverable: plan.phase == "committed" && plan.recovery.is_some(),
        });
    }
    values.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(a.id.cmp(&b.id)));
    Ok(values)
}
pub fn prepare_recovery(
    root: &Path,
    request: super::recovery::Request,
) -> ModelResult<super::recovery::Preview> {
    let _guard = execution_lock(root)?;
    reject_incomplete(root)?;
    let source = load(root, &request.source_id)?;
    if source.phase != "committed" {
        return Err(invalid(
            "recovery_source",
            "仅成功保存记录可生成恢复草稿；未完成记录请核对原计划",
        ));
    }
    let baseline = source.recovery.as_ref().ok_or_else(|| {
        invalid(
            "recovery_source",
            "该旧记录缺少组合恢复基线，保留为核对记录",
        )
    })?;
    let mut after = source.policy.published();
    if after.default_channel_id.is_none() {
        after.default_channel_id = after
            .channels
            .iter()
            .find(|c| {
                Some(c.connection.provider_id.as_str())
                    == source.next_providers["defaultProvider"].as_str()
            })
            .map(|c| c.id.clone());
    }
    super::recovery::prepare(root, baseline, &after, request)
}
pub fn accept_recovery(root: &Path, id: &str) -> ModelResult<Snapshot> {
    let _guard = execution_lock(root)?;
    reject_incomplete(root)?;
    super::recovery::accept(root, id)
}
pub(super) fn reject_incomplete(root: &Path) -> ModelResult<()> {
    for id in plan_ids(root)? {
        let plan = load(root, &id)?;
        if !["prepared", "committed", "compensated"].contains(&plan.phase.as_str()) {
            return Err(invalid(
                "plan_incomplete",
                "已有保存计划待核对，请先核对原计划或撤回其未完成写入",
            ));
        }
    }
    Ok(())
}
pub(super) fn recovery_uses(root: &Path, template: &str, revision: &str) -> ModelResult<bool> {
    for id in plan_ids(root)? {
        let plan = load(root, &id)?;
        if ["prepared", "compensated"].contains(&plan.phase.as_str()) {
            continue;
        }
        if std::iter::once(&plan.policy)
            .chain(plan.recovery.as_ref().map(|b| &b.policy))
            .flat_map(|policy| {
                policy.template_refs.iter().chain(
                    policy
                        .channels
                        .iter()
                        .flat_map(|c| c.template_refs.iter().chain(c.model_refs.values())),
                )
            })
            .any(|r| r.template_id == template && r.revision_id == revision)
        {
            return Ok(true);
        }
    }
    Ok(false)
}
fn rows(snapshot: &UpstreamSnapshot) -> ModelResult<&Vec<Value>> {
    snapshot
        .models
        .as_array()
        .ok_or_else(|| invalid("runtime_models", "官方模型清单无效"))
}
pub(super) fn catalog_slugs(snapshot: &UpstreamSnapshot) -> ModelResult<BTreeSet<String>> {
    let models = snapshot.catalog["models"]
        .as_array()
        .ok_or_else(|| invalid("runtime_catalog", "官方持久目录不可核对"))?;
    let values: ModelResult<BTreeSet<String>> = models
        .iter()
        .map(|m| {
            m["slug"]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| invalid("runtime_catalog", "目录行缺少稳定身份"))
        })
        .collect();
    let values = values?;
    if values.len() != models.len() {
        return Err(invalid("runtime_catalog", "目录包含重复身份，不能准确投影"));
    }
    Ok(values)
}
fn provider_map(value: &Value) -> ModelResult<&serde_json::Map<String, Value>> {
    value["providers"]
        .as_object()
        .ok_or_else(|| invalid("runtime_providers", "官方渠道快照无效"))
}
fn selected(provider: &Value, model: &Value) -> bool {
    !model["disabled"].as_bool().unwrap_or(false)
        && !model["initialSelectionPending"].as_bool().unwrap_or(false)
        && !provider["disabled"].as_bool().unwrap_or(false)
        && provider["selectedModels"]
            .as_array()
            .filter(|v| !v.is_empty())
            .is_none_or(|v| v.iter().any(|id| id == &model["id"]))
}
fn set_map(provider: &mut Value, key: &str, model: &str, value: Value) -> ModelResult<()> {
    let object = provider
        .as_object_mut()
        .ok_or_else(|| invalid("runtime_providers", "渠道字段无效"))?;
    let map = object
        .entry(key)
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| invalid("runtime_field", "官方覆盖字段不是对象，未覆盖"))?;
    map.insert(model.into(), value);
    Ok(())
}
fn apply_naming(
    provider: &mut Value,
    channel: &ChannelPolicy,
    snapshot: &UpstreamSnapshot,
) -> ModelResult<()> {
    for (id, naming) in &channel.naming {
        naming.validate()?;
        if let Some(alias) = &naming.alias {
            if alias.is_empty() {
                if provider
                    .get("modelAliases")
                    .is_some_and(|value| !value.is_object())
                {
                    return Err(invalid(
                        "model_alias",
                        "原路由别名字段不是映射，未当作已清除",
                    ));
                }
                if let Some(map) = provider
                    .get_mut("modelAliases")
                    .and_then(Value::as_object_mut)
                {
                    map.remove(id);
                }
            } else {
                set_map(provider, "modelAliases", id, json!(alias))?;
            }
        }
    }
    let known = channel
        .discovered
        .iter()
        .map(|m| m.id.clone())
        .chain(channel.model_order.iter().cloned())
        .chain(
            rows(snapshot)?
                .iter()
                .filter(|row| row["provider"] == channel.connection.provider_id)
                .filter_map(|row| row["id"].as_str().map(str::to_owned)),
        )
        .collect::<BTreeSet<_>>();
    let mut claims = BTreeSet::new();
    for (id, value) in provider
        .get("modelAliases")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        let Some(alias) = value.as_str() else {
            return Err(invalid("model_alias", "原路由别名不是字符串，未混合覆盖"));
        };
        let lower = alias.to_ascii_lowercase();
        if !claims.insert(lower.clone()) {
            return Err(invalid(
                "model_alias_collision",
                "别名与同运行渠道其他型号冲突",
            ));
        }
        if channel
            .naming
            .get(id)
            .is_some_and(|n| n.alias.as_ref().is_some_and(|v| !v.is_empty()))
            && (known.iter().any(|native| {
                native.to_ascii_lowercase() == lower
                    || native.replace('/', "-").to_ascii_lowercase() == lower
            }) || rows(snapshot)?.iter().any(|row| {
                (row["native"] == true || row["provider"] == "combo")
                    && row["id"]
                        .as_str()
                        .is_some_and(|id| id.to_ascii_lowercase() == lower)
            }) || catalog_slugs(snapshot)?
                .iter()
                .any(|slug| !slug.contains('/') && slug.to_ascii_lowercase() == lower))
        {
            return Err(invalid(
                "model_alias_collision",
                "路由别名与实际原始、原生或组合目录身份冲突",
            ));
        }
    }
    Ok(())
}
fn compile_shared_rules(
    provider: &mut Value,
    channel: &ChannelPolicy,
    projection: &compiler::DraftProjection,
) -> ModelResult<()> {
    let rows = projection
        .rows
        .iter()
        .filter(|row| row.channel_id == channel.id)
        .collect::<Vec<_>>();
    for (field, key) in [
        (
            "reasoningAuthoritative",
            "modelReasoningEffortsAuthoritative",
        ),
        ("fastWire", "fastWire"),
    ] {
        let declared = |row: &compiler::ModelProjection| -> ModelResult<Option<Value>> {
            Ok(match field {
                "reasoningAuthoritative" => match &row.rules.reasoning_authoritative {
                    Field::Custom(value) => Some(json!(value)),
                    _ => None,
                },
                _ => match &row.rules.fast_wire {
                    Field::Custom(value) => Some(serde_json::to_value(value)?),
                    _ => None,
                },
            })
        };
        let mut intended = None;
        for row in &rows {
            if row
                .source
                .get(field)
                .is_some_and(|source| source != "上游声明")
            {
                if let Some(value) = declared(row)? {
                    if intended.as_ref().is_some_and(|old| old != &value) {
                        return Err(invalid("shared_rule_scope","权威声明和Fast wire由同一运行协议分组共用，组内值不同不能静默扩展作用范围"));
                    }
                    intended = Some(value)
                }
            }
        }
        let Some(value) = intended else { continue };
        for row in &rows {
            if declared(row)?.as_ref() != Some(&value) && provider.get(key) != Some(&value) {
                return Err(invalid("shared_rule_scope","此字段由运行协议分组共用，请在渠道默认设置或使组内全部型号保持一致；不对其他型号静默套用"));
            }
            let protocol = match &row.rules.protocol {
                Field::Custom(value) => value.as_str(),
                _ => channel.connection.protocol.as_str(),
            };
            if field == "reasoningAuthoritative"
                && value == true
                && !matches!(&row.rules.reasoning_efforts, Field::Custom(_))
            {
                return Err(invalid(
                    "reasoning_authoritative",
                    "权威声明需要每个受影响型号都有明确推理列表",
                ));
            }
            if field == "fastWire" {
                if value.is_null() {
                    if row.rules.fast == Field::Custom(true) {
                        return Err(invalid("fast_wire", "禁用Fast wire与型号Fast能力冲突"));
                    }
                } else {
                    let supported = match value["kind"].as_str() {
                        Some("service-tier") => {
                            ["openai-chat", "openai-responses", "azure", "azure-openai"]
                                .contains(&protocol)
                        }
                        Some("anthropic-speed") => protocol == "anthropic",
                        Some("cursor-variant") => protocol == "cursor",
                        _ => false,
                    };
                    if !supported {
                        return Err(invalid(
                            "fast_wire_protocol",
                            "所选Fast wire不适用于该型号最终协议",
                        ));
                    }
                }
            }
        }
        if field == "fastWire"
            && value.is_null()
            && rows.iter().any(|row| {
                provider["modelSupportsServiceTier"][&row.model_id]
                    .as_bool()
                    .unwrap_or(provider["supportsServiceTier"] == true)
            })
        {
            return Err(invalid(
                "fast_wire",
                "上游仍声明Fast能力，禁用wire前须明确调整能力，不能仅隐藏入口",
            ));
        }
        provider
            .as_object_mut()
            .ok_or_else(|| invalid("provider", "渠道结构无效"))?
            .insert(key.into(), value);
    }
    Ok(())
}
pub fn compile(
    snapshot: &UpstreamSnapshot,
    library: &Library,
    policy: &Policy,
) -> ModelResult<(Value, Vec<String>, BTreeSet<String>)> {
    compile_owned(snapshot, library, policy, &[])
}
fn compile_owned(
    snapshot: &UpstreamSnapshot,
    library: &Library,
    policy: &Policy,
    ownership: &[OwnedField],
) -> ModelResult<(Value, Vec<String>, BTreeSet<String>)> {
    compile_materials(
        snapshot,
        library,
        policy,
        ownership,
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
}
pub(super) fn compile_materials(
    snapshot: &UpstreamSnapshot,
    library: &Library,
    policy: &Policy,
    ownership: &[OwnedField],
    materials: &BTreeMap<String, Value>,
    group_seeds: &BTreeMap<String, Value>,
) -> ModelResult<(Value, Vec<String>, BTreeSet<String>)> {
    let projection = compiler::project(library, policy)?;
    let source_rows = rows(snapshot)?;
    let before = provider_map(&snapshot.providers)?;
    let actual = catalog_slugs(snapshot)?;
    // Prove baseline reconstruction against ALL persisted slugs, retaining native/control/unknown occupancy.
    let visible: BTreeSet<_> = source_rows
        .iter()
        .filter(|r| {
            r["native"] != true
                && before
                    .get(r["provider"].as_str().unwrap_or(""))
                    .is_some_and(|p| selected(p, r))
        })
        .filter_map(|r| r["namespaced"].as_str().map(str::to_owned))
        .collect();
    if !visible.is_subset(&actual) {
        return Err(invalid(
            "catalog_projection",
            "管理目录与持久目录不同步，写前计数暂不可用",
        ));
    }
    let mut next = snapshot.providers.clone();
    for field in ownership {
        if policy
            .channels
            .iter()
            .any(|c| c.connection.provider_id == field.provider)
        {
            if (if field.model.is_empty() {
                next["providers"][&field.provider].get(&field.key)
            } else {
                next["providers"][&field.provider][&field.key].get(&field.model)
            }) != if field.removed {
                None
            } else {
                Some(&field.written)
            } {
                return Err(invalid(
                    "field_drift",
                    "受管模型字段已被外部修改，请显式比较后再保存",
                ));
            }
            let provider = next["providers"]
                .get_mut(&field.provider)
                .ok_or_else(|| invalid("provider", "受管渠道缺失"))?;
            if field.model.is_empty() {
                let object = provider
                    .as_object_mut()
                    .ok_or_else(|| invalid("provider", "渠道结构无效"))?;
                match &field.original {
                    Some(value) => {
                        object.insert(field.key.clone(), value.clone());
                    }
                    None => {
                        object.remove(&field.key);
                    }
                }
            } else if let Some(map) = provider.get_mut(&field.key).and_then(Value::as_object_mut) {
                match &field.original {
                    Some(value) => {
                        map.insert(field.model.clone(), value.clone());
                    }
                    None => {
                        map.remove(&field.model);
                    }
                }
                if map.is_empty() {
                    provider
                        .as_object_mut()
                        .ok_or_else(|| invalid("provider", "渠道结构无效"))?
                        .remove(&field.key);
                }
            }
        }
    }
    let mut expected = actual.clone();
    let managed: BTreeSet<_> = policy
        .channels
        .iter()
        .map(|c| c.connection.provider_id.as_str())
        .collect();
    let mut desired_order = vec![];
    for channel in &policy.channels {
        let provider_id = &channel.connection.provider_id;
        let prior = before.get(provider_id);
        let imported = materials.get(provider_id);
        if prior.is_none() {
            if imported.is_some() && channel.connection.enabled {
                return Err(invalid(
                    "provider_import_activation",
                    "首次保存原渠道配置须保持停用；读回完整目录与别名后再确认启用",
                ));
            }
            if ["openai", "chatgpt", "openai-multi", "combo"].contains(&provider_id.as_str()) {
                return Err(invalid(
                    "reserved_provider",
                    "此运行渠道身份保留，请使用新的自建标识",
                ));
            }
            if imported.is_none()
                && !matches!(&channel.selection,ModelSelection::Whitelist(values) if !values.is_empty())
            {
                return Err(invalid(
                    "new_channel_selection",
                    "新增渠道须明确勾选至少一个模型，不能以空白名单注册",
                ));
            }
            if imported.is_none()
                && channel.connection.protocol != "ollama-native"
                && channel.connection.credential_ref.is_none()
            {
                return Err(invalid("credential", "新增渠道尚未配置经过发现确认的密钥"));
            }
            next["providers"].as_object_mut().ok_or_else(||invalid("provider","渠道结构无效"))?.insert(provider_id.clone(),json!({"adapter":channel.connection.protocol,"baseUrl":"","authMode":if channel.connection.protocol=="ollama-native"{"local"}else{"key"},"models":channel.discovered.iter().map(|m|m.id.clone()).collect::<Vec<_>>(),"liveModels":true}));
            if let Some(seed) = group_seeds.get(provider_id) {
                super::provider_fields::validate_public(seed)?;
                next["providers"][provider_id] = seed.clone();
            }
        }
        if let Some(original) = imported {
            if channel.connection.enabled {
                return Err(invalid(
                    "provider_import_activation",
                    "采用原渠道字段时须保持停用，读回实际目录与别名后再启用",
                ));
            }
            super::provider_fields::validate_public(original)?;
            next["providers"][provider_id] = original.clone();
        }
        let node = channel
            .connection
            .nodes
            .iter()
            .find(|n| n.id == channel.connection.active_node_id)
            .ok_or_else(|| invalid("node", "当前节点不存在"))?;
        let provider = next["providers"]
            .get_mut(provider_id)
            .ok_or_else(|| invalid("provider", "渠道绑定不存在"))?;
        if prior.is_some_and(|p| {
            p["authMode"]
                .as_str()
                .is_some_and(|mode| ["oauth", "forward"].contains(&mode))
        }) && (provider["baseUrl"] != json!(node.url)
            || provider["adapter"] != json!(channel.connection.protocol))
        {
            return Err(invalid(
                "provider_auth_boundary",
                "OAuth或转发认证的服务/协议不能作为普通Key渠道迁移，请保持官方绑定",
            ));
        }
        provider["baseUrl"] = json!(node.url);
        provider["adapter"] = json!(channel.connection.protocol);
        apply_naming(provider, channel, snapshot)?;
        // UI 规范 §29.1：本轮不使用友好名称。写入时清掉这些型号的旧展示名，
        // 让正式配置、目录读回与客户端选择器都回到 渠道名/原始模型ID；
        // 路由别名（modelAliases）仍按各自规则维护，不与显示名混用。
        for model in &channel.discovered {
            if let Some(names) = provider
                .get_mut("modelDisplayNames")
                .and_then(Value::as_object_mut)
            {
                names.remove(&model.id);
                if names.is_empty() {
                    provider
                        .as_object_mut()
                        .unwrap()
                        .remove("modelDisplayNames");
                }
            }
        }
        let mut declared = provider["models"].as_array().cloned().unwrap_or_default();
        for model in &channel.discovered {
            if !declared.iter().any(|id| id.as_str() == Some(&model.id)) {
                declared.push(json!(model.id));
            }
        }
        if !declared.is_empty() {
            provider["models"] = json!(declared);
        }
        provider["disabled"] = json!(!channel.connection.enabled);
        if channel.connection.allow_private_network {
            provider["allowPrivateNetwork"] = json!(true);
        } else if provider.get("allowPrivateNetwork").is_some() {
            provider["allowPrivateNetwork"] = json!(false);
        }
        provider["newModelPolicy"] = json!(if matches!(channel.selection, ModelSelection::All) {
            "on"
        } else {
            "off"
        });
        match &channel.selection {
            ModelSelection::Whitelist(values) => provider["selectedModels"] = json!(values),
            _ => {
                provider
                    .as_object_mut()
                    .ok_or_else(|| invalid("provider", "渠道字段无效"))?
                    .remove("selectedModels");
            }
        }
        let known: Vec<_> = source_rows
            .iter()
            .filter(|r| r["provider"].as_str() == Some(provider_id) && r["native"] != true)
            .collect();
        for model in &known {
            if let Some(slug) = model["namespaced"].as_str() {
                expected.remove(slug);
            }
        }
        // An unmapped row under a changed provider may be alias/featured/derived occupancy. Never guess it away.
        if expected
            .iter()
            .any(|slug| slug.starts_with(&format!("{provider_id}/")))
        {
            return Err(invalid(
                "catalog_projection",
                "该渠道有尚未映射的别名或派生目录行，未通过写前完整计数",
            ));
        }
        for row in projection
            .rows
            .iter()
            .filter(|r| r.channel_id == channel.id)
        {
            let source = known
                .iter()
                .find(|r| r["id"].as_str() == Some(&row.model_id));
            if row.enabled && row.selected {
                let slugs = if source.is_some() {
                    known
                        .iter()
                        .filter(|r| r["id"] == row.model_id)
                        .map(|r| {
                            r["namespaced"]
                                .as_str()
                                .map(str::to_owned)
                                .ok_or_else(|| invalid("model_identity", "官方模型缺少路由身份"))
                        })
                        .collect::<ModelResult<BTreeSet<_>>>()?
                } else if channel.discovered.iter().any(|m| m.id == row.model_id) {
                    BTreeSet::from([format!("{provider_id}/{}", row.model_id.replace('/', "-"))])
                } else {
                    return Err(invalid(
                        "model_unverified",
                        "所选模型未在当前发现清单中核对",
                    ));
                };
                for slug in slugs {
                    if !expected.insert(slug.clone()) {
                        return Err(invalid(
                            "model_identity_collision",
                            "模型原始身份编码发生碰撞，请调整选择",
                        ));
                    }
                    desired_order.push(slug);
                }
            }
            let fields = &row.rules;
            // 2.84.0 reads these request controls on the OpenAI Chat path. Its
            // Responses/Anthropic adapters can persist the same provider keys
            // while leaving caller reasoning untouched. Do not turn metadata
            // readback into a claim that explicit runtime constraints work.
            let protocol = match &fields.protocol {
                Field::Custom(protocol) => protocol.as_str(),
                Field::Inherit => channel.connection.protocol.as_str(),
            };
            let explicit = |field: &str| {
                row.source
                    .get(field)
                    .is_some_and(|source| source != "上游声明")
            };
            if protocol != "openai-chat"
                && (fields.no_reasoning == Field::Custom(true) && explicit("noReasoning")
                    || matches!(&fields.pinned_effort, Field::Custom(_))
                        && explicit("pinnedEffort")
                    || matches!(&fields.effort_map,Field::Custom(map) if !map.is_empty())
                        && explicit("effortMap")
                    || fields.reasoning_authoritative == Field::Custom(true)
                        && explicit("reasoningAuthoritative"))
            {
                return Err(invalid("reasoning_wire_unsupported",format!("{} / {}：当前2.84.0仅核实OpenAI Chat支持不传推理参数、强制档、标签映射与请求权威约束；{}尚未核实，保留草稿，请恢复这些字段的继承或选用已支持的协议",channel.label,row.model_id,protocol)));
            }
            if let Field::Custom(value) = fields.no_reasoning {
                if row
                    .source
                    .get("noReasoning")
                    .is_some_and(|s| s != "上游声明")
                {
                    let mut ids = match provider.get("noReasoningModels") {
                        Some(value) => string_array(value)?,
                        None => vec![],
                    };
                    if value
                        && !row.model_id.contains(':')
                        && source_rows.iter().any(|other| {
                            other["provider"] == provider_id.as_str()
                                && other["id"]
                                    .as_str()
                                    .is_some_and(|id| id.starts_with(&format!("{}:", row.model_id)))
                        })
                    {
                        return Err(invalid("no_reasoning_scope","上游分类会同时影响带规格后缀的型号，请逐项设置完整ID，不能把基础ID当单型号覆盖"));
                    }
                    if !value
                        && row
                            .model_id
                            .split_once(':')
                            .is_some_and(|(family, _)| ids.iter().any(|id| id == family))
                    {
                        return Err(invalid("no_reasoning_scope","上游族级不传参数规则覆盖此型号，不能以单型号false伪称已允许；请先明确调整原族规则"));
                    }
                    if value && !ids.contains(&row.model_id) {
                        ids.push(row.model_id.clone());
                        provider["noReasoningModels"] = json!(ids);
                    } else if !value && ids.contains(&row.model_id) {
                        ids.retain(|id| id != &row.model_id);
                        if ids.is_empty() {
                            provider
                                .as_object_mut()
                                .unwrap()
                                .remove("noReasoningModels");
                        } else {
                            provider["noReasoningModels"] = json!(ids);
                        }
                    }
                }
            }
            if let Field::Custom(pin) = &fields.pinned_effort {
                if fields.no_reasoning != Field::Custom(true)
                    && !matches!(&fields.reasoning_efforts,Field::Custom(values) if values.contains(pin))
                {
                    return Err(invalid(
                        "pinned_effort",
                        "目标型号尚无包含强制档的可核验推理列表",
                    ));
                }
                if row
                    .source
                    .get("pinnedEffort")
                    .is_some_and(|s| s != "上游声明")
                {
                    set_map(
                        provider,
                        "modelPinnedReasoningEfforts",
                        &row.model_id,
                        json!(pin),
                    )?;
                }
            }
            if let Field::Custom(map) = &fields.effort_map {
                if fields.no_reasoning != Field::Custom(true)
                    && !map.is_empty()
                    && !matches!(&fields.reasoning_efforts,Field::Custom(values) if map.keys().all(|key|values.contains(key)))
                {
                    return Err(invalid(
                        "effort_map",
                        "目标型号的推理列表未知，不能确认标签映射",
                    ));
                }
                if row.source.get("effortMap").is_some_and(|s| s != "上游声明") {
                    set_map(
                        provider,
                        "modelReasoningEffortMap",
                        &row.model_id,
                        json!(map),
                    )?;
                }
            }
            if let Field::Custom(value) = &fields.protocol {
                if value != &channel.connection.protocol {
                    if !["openai-chat", "openai-responses"].contains(&value.as_str()) {
                        return Err(invalid(
                            "model_protocol",
                            "该模型协议需要稳定受管渠道映射，当前组合不可直接写入",
                        ));
                    }
                    set_map(provider, "modelAdapters", &row.model_id, json!(value))?;
                }
            }
            if let Field::Custom(value) = fields.context_window {
                if row
                    .source
                    .get("contextWindow")
                    .is_some_and(|s| s != "上游声明")
                {
                    set_map(provider, "modelContextWindows", &row.model_id, json!(value))?;
                }
            }
            if let Field::Custom(value) = &fields.input_modalities {
                if row
                    .source
                    .get("inputModalities")
                    .is_some_and(|s| s != "上游声明")
                {
                    set_map(
                        provider,
                        "modelInputModalities",
                        &row.model_id,
                        json!(value),
                    )?;
                }
            }
            if let Field::Custom(value) = &fields.reasoning_efforts {
                if row
                    .source
                    .get("reasoningEfforts")
                    .is_some_and(|s| s != "上游声明")
                {
                    set_map(
                        provider,
                        "modelReasoningEfforts",
                        &row.model_id,
                        json!(value),
                    )?;
                }
            }
            if let Field::Custom(value) = &fields.default_effort {
                if row
                    .source
                    .get("defaultEffort")
                    .is_some_and(|s| s != "上游声明")
                {
                    set_map(
                        provider,
                        "modelDefaultReasoningEfforts",
                        &row.model_id,
                        json!(value),
                    )?;
                }
            }
            if let Field::Custom(value) = fields.fast {
                if row.source.get("fast").is_some_and(|s| s != "上游声明") {
                    set_map(
                        provider,
                        "modelSupportsServiceTier",
                        &row.model_id,
                        json!(value),
                    )?;
                }
            }
        }
        compile_shared_rules(provider, channel, &projection)?;
    }
    super::channel_actions::default_provider(policy, &mut next)?;
    // 官方 defaultProvider 若恰好指向被停用的渠道，官方自身会回落；
    // 本产品不要求用户先挑一个“默认渠道”，因此这里不再中断保存。
    if expected.len() > 100 {
        return Err(invalid(
            "catalog_budget",
            format!(
                "本次完整目录投影{}条，超过当前兼容预算100；草稿保留",
                expected.len()
            ),
        ));
    }
    // Native/non-managed placements retain their positions; managed groups replace one block.
    let old_order = snapshot.order["pickerOrder"]
        .as_array()
        .ok_or_else(|| invalid("picker_order", "官方选择器顺序未知"))?;
    let mut order: Vec<String> = vec![];
    let mut inserted = false;
    for slug in old_order.iter().filter_map(Value::as_str) {
        let is_managed = slug
            .split_once('/')
            .is_some_and(|(p, _)| managed.contains(p));
        if is_managed {
            if !inserted {
                order.extend(desired_order.clone());
                inserted = true;
            }
        } else if expected.contains(slug) {
            order.push(slug.into());
        }
    }
    if !inserted {
        order.extend(desired_order);
    }
    let mut dedupe = BTreeSet::new();
    order.retain(|slug| dedupe.insert(slug.clone()));
    Ok((next, order, expected))
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveScope {
    pub channel_ids: Vec<String>,
    pub include_shared: bool,
}
fn select_save_scope(
    saved: &Policy,
    draft: &Policy,
    scope: &SaveScope,
) -> ModelResult<(Policy, Option<Policy>)> {
    let ids: BTreeSet<_> = scope.channel_ids.iter().cloned().collect();
    if ids.len() != scope.channel_ids.len()
        || ids.len() > 256
        || ids.iter().any(|id| {
            !saved
                .channels
                .iter()
                .chain(&draft.channels)
                .any(|c| &c.id == id)
                && !draft.pending_removals.iter().any(|r| &r.channel_id == id)
        })
    {
        return Err(invalid(
            "save_scope",
            "本次渠道范围重复或已失效，请重新选择",
        ));
    }
    if draft.recovery_id.is_some() {
        return Err(invalid(
            "save_scope",
            "成功历史恢复须按原恢复范围完整核对，不能拆开提交",
        ));
    }
    let mut result = saved.clone();
    result.pending_removals = draft
        .pending_removals
        .iter()
        .filter(|r| ids.contains(&r.channel_id))
        .cloned()
        .collect();
    result.channels = saved
        .channels
        .iter()
        .filter_map(|old| {
            if ids.contains(&old.id) {
                draft.channels.iter().find(|c| c.id == old.id).cloned()
            } else {
                Some(old.clone())
            }
        })
        .collect();
    for channel in draft
        .channels
        .iter()
        .filter(|c| ids.contains(&c.id) && !saved.channels.iter().any(|old| old.id == c.id))
    {
        result.channels.push(channel.clone());
    }
    if scope.include_shared {
        result.template_refs = draft.template_refs.clone();
        // “默认渠道”已退役：不再把草稿里的残留值写回策略，
        // 官方 defaultProvider 保持用户自己的设置。
        let positions: BTreeMap<_, _> = draft
            .channels
            .iter()
            .enumerate()
            .map(|(i, c)| (&c.id, i))
            .collect();
        result
            .channels
            .sort_by_key(|c| positions.get(&c.id).copied().unwrap_or(usize::MAX));
    }
    if result.published() == saved.published() && result.pending_removals.is_empty() {
        return Err(invalid("save_scope", "所选范围没有可提交修改"));
    }
    let mut remaining = draft.clone();
    remaining
        .pending_removals
        .retain(|r| !ids.contains(&r.channel_id));
    let remaining = (remaining != result.published()).then_some(remaining);
    Ok((result, remaining))
}
fn deferred_names(saved: &Policy, draft: &Policy, scope: &SaveScope) -> Vec<String> {
    let ids: BTreeSet<_> = saved
        .channels
        .iter()
        .chain(&draft.channels)
        .map(|c| &c.id)
        .collect();
    let mut names = vec![];
    for id in ids {
        let old = saved.channels.iter().find(|c| &c.id == id);
        let next = draft.channels.iter().find(|c| &c.id == id);
        if old != next && !scope.channel_ids.contains(id) {
            names.push(next.or(old).unwrap().label.clone());
        }
    }
    for removal in draft
        .pending_removals
        .iter()
        .filter(|r| !scope.channel_ids.contains(&r.channel_id))
    {
        names.push(removal.provider_id.clone());
    }
    if !scope.include_shared
        && (saved.template_refs != draft.template_refs
            || saved.channels.iter().map(|c| &c.id).collect::<Vec<_>>()
                != draft.channels.iter().map(|c| &c.id).collect::<Vec<_>>())
    {
        names.push("全局模版采用或渠道顺序".into());
    }
    names
}
pub async fn prepare(
    root: &Path,
    context: &ProcessContext,
    policy: Policy,
    expected_library: String,
    expected_policy: String,
    expected_workspace: String,
) -> ModelResult<SavePlanView> {
    prepare_scoped(
        root,
        context,
        policy,
        expected_library,
        expected_policy,
        expected_workspace,
        None,
    )
    .await
}
pub async fn prepare_scoped(
    root: &Path,
    context: &ProcessContext,
    policy: Policy,
    expected_library: String,
    expected_policy: String,
    expected_workspace: String,
    scope: Option<SaveScope>,
) -> ModelResult<SavePlanView> {
    let _execution = execution_lock(root)?;
    reject_incomplete(root)?;
    let local = ModelStore::new(root).load()?;
    if local.library_digest != expected_library
        || local.policy_digest != expected_policy
        || local.workspace_digest != expected_workspace
    {
        return Err(invalid("stale_baseline", "本地保存基准已变化，请重新比较"));
    }
    if local
        .workspace
        .as_ref()
        .is_none_or(|w| w.policy != policy || w.base_policy != expected_policy)
    {
        return Err(invalid(
            "stale_workspace",
            "当前草稿尚未暂存完成，请稍后重试",
        ));
    }
    let deferred_channels = scope
        .as_ref()
        .map(|scope| deferred_names(&local.policy, &policy, scope))
        .unwrap_or_default();
    let (policy, deferred_policy) = if let Some(scope) = &scope {
        select_save_scope(&local.policy, &policy, scope)?
    } else {
        (policy, None)
    };
    let before = upstream::snapshot(context).await?;
    let initial_removals =
        super::channel_actions::initial_removals(root, context, &before, &local.policy, &policy)?;
    let mut prior_policy = local.policy.clone();
    prior_policy.channels.extend(initial_removals);
    let ownership_path = root.join(OWNERSHIP);
    safety::check_path(root, &ownership_path, true)?;
    let previous: Vec<OwnedField> = if ownership_path.exists() {
        serde_json::from_slice(&safety::read_file(&ownership_path, 16 * 1024 * 1024)?)?
    } else {
        vec![]
    };
    if let Some(id) = &policy.recovery_id {
        let request = super::recovery::selected_request(root, id, &policy)?;
        return prepare_history(root, context, local, before, policy, previous, request).await;
    }
    let mappings = super::protocol_groups::mappings(root, &local.library, &policy)?;
    let execution_policy =
        super::protocol_groups::expand(&before, &local.library, &policy, &mappings)?;
    let removed = super::channel_actions::removals(&before, &prior_policy, &policy, &mappings)?;
    let mut removed_keys = vec![];
    if !removed.is_empty() {
        let management = super::management::Management::bind(context).await?;
        for provider in &removed {
            let keys = management.key_state(provider).await?;
            let backup = super::provider_auth::capture(root, context, provider, &keys).await?;
            removed_keys.push(RemovedKey {
                provider: provider.clone(),
                before: keys,
                reference: None,
                backup,
            });
        }
    }
    let projection_baseline = super::channel_actions::projection(&before, &removed)?;
    let mut imported_materials = BTreeMap::new();
    let mut imported_authorizations = BTreeMap::new();
    for channel in &policy.channels {
        if local.policy.channels.iter().any(|c| {
            c.id == channel.id && c.connection.imported_config == channel.connection.imported_config
        }) && before.providers["providers"]
            .get(&channel.connection.provider_id)
            .is_some()
        {
            continue;
        }
        if let Some(reference) = &channel.connection.imported_config {
            if super::channel_actions::protected(&before, &channel.connection.provider_id) {
                return Err(invalid(
                    "provider_auth_boundary",
                    "原生或OAuth来源不能作为普通渠道接收目标，请明确账户映射",
                ));
            }
            let material = super::provider_import::load(root, channel, reference)?;
            let node = channel
                .connection
                .nodes
                .iter()
                .find(|n| n.id == channel.connection.active_node_id)
                .ok_or_else(|| invalid("node", "当前节点不存在"))?;
            if material.public["baseUrl"] != node.url {
                return Err(invalid(
                    "provider_import_direction",
                    "导入授权所属服务与当前节点不同，未发送",
                ));
            }
            if material.public["authMode"]
                .as_str()
                .is_some_and(|m| ["oauth", "forward"].contains(&m))
            {
                return Err(invalid(
                    "provider_auth_boundary",
                    "原生或OAuth渠道须先完成账户连接映射，未注册为普通Key渠道",
                ));
            }
            if let Some(backup) = super::provider_import::authorization(root, context, &material)? {
                imported_authorizations.insert(channel.connection.provider_id.clone(), backup);
            }
            imported_materials.insert(channel.connection.provider_id.clone(), material.public);
        }
    }
    let group_seeds = super::protocol_groups::seeds(
        &before,
        &policy,
        &execution_policy,
        &mappings,
        &imported_materials,
    )?;
    let (mut next_providers, next_order, expected_slugs) = compile_materials(
        &projection_baseline,
        &local.library,
        &execution_policy,
        &previous,
        &imported_materials,
        &group_seeds,
    )?;
    let next_order = super::protocol_groups::order(
        &before,
        &local.library,
        &policy,
        &execution_policy,
        &mappings,
        next_order,
        &expected_slugs,
    )?;
    let before_catalog = catalog_slugs(&before)?;
    let mut route_changes = vec![];
    for channel in &execution_policy.channels {
        let parent = policy
            .channels
            .iter()
            .find(|c| c.id == channel.id)
            .or_else(|| {
                mappings
                    .iter()
                    .find(|m| m.id == channel.id)
                    .and_then(|m| policy.channels.iter().find(|c| c.id == m.channel_id))
            });
        let Some(parent) = parent else {
            continue;
        };
        let providers = std::iter::once(parent.connection.provider_id.as_str())
            .chain(
                mappings
                    .iter()
                    .filter(|m| m.channel_id == parent.id)
                    .map(|m| m.provider_id.as_str()),
            )
            .collect::<BTreeSet<_>>();
        for model in &channel.discovered {
            let next = format!(
                "{}/{}",
                channel.connection.provider_id,
                model.id.replace('/', "-")
            );
            if let Some(previous) = rows(&before)?
                .iter()
                .find(|row| {
                    row["id"].as_str() == Some(&model.id)
                        && row["provider"]
                            .as_str()
                            .is_some_and(|provider| providers.contains(provider))
                        && row["namespaced"]
                            .as_str()
                            .is_some_and(|slug| before_catalog.contains(slug))
                })
                .and_then(|row| row["namespaced"].as_str())
            {
                if previous != next {
                    route_changes.push(format!("{}：{} → {}", model.id, previous, next));
                }
            }
            if channel
                .naming
                .get(&model.id)
                .is_some_and(|n| n.alias.is_some())
            {
                let before = before.providers["providers"][&channel.connection.provider_id]
                    ["modelAliases"][&model.id]
                    .as_str();
                let after = next_providers["providers"][&channel.connection.provider_id]
                    ["modelAliases"][&model.id]
                    .as_str();
                if before != after {
                    route_changes.push(format!(
                        "{}：自定义快捷入口 {} → {}（原始ID保持；使用旧别名的会话需重新选择）",
                        model.id,
                        before.unwrap_or("无"),
                        after.unwrap_or("已移除")
                    ));
                }
            }
        }
    }
    super::protocol_groups::decorate(&mut next_providers, &policy, &mappings);
    let ownership = next_ownership(
        &before.providers,
        &next_providers,
        &previous,
        &execution_policy,
    )?;
    let disabled_before =
        string_array(&upstream::read(context, upstream::ReadOperation::DisabledModels).await?)?;
    let observed_providers = before.providers.clone();
    let observed_order = string_array(&before.order["pickerOrder"])?;
    let mut credential_changes = vec![];
    let mut management = None;
    for channel in &execution_policy.channels {
        if imported_authorizations.contains_key(&channel.connection.provider_id) {
            continue;
        }
        let Some(reference) = &channel.connection.credential_ref else {
            continue;
        };
        if local
            .policy
            .channels
            .iter()
            .any(|c| c.id == channel.id && c.connection.credential_ref.as_ref() == Some(reference))
        {
            continue;
        }
        let key = super::discovery::credential(reference)?;
        let intended_digest = super::store::sha256(key.trim().as_bytes());
        let intended_id = intended_digest.trim_start_matches("sha256:")[..8].to_owned();
        let provider = &channel.connection.provider_id;
        let keys = if before.providers["providers"].get(provider).is_some() {
            if management.is_none() {
                management = Some(super::management::Management::bind(context).await?);
            }
            management.as_ref().unwrap().key_state(provider).await?
        } else {
            super::management::KeyState {
                active_id: None,
                ids: BTreeSet::new(),
            }
        };
        credential_changes.push(CredentialChange {
            provider: provider.clone(),
            reference: reference.clone(),
            before: keys,
            intended_id,
            intended_digest,
            observed: None,
        });
    }
    let mut recovery_local = local.clone();
    recovery_local.policy = prior_policy;
    let recovery = super::recovery::Baseline::capture_runtime(
        &recovery_local,
        before.providers["defaultProvider"].as_str(),
    )?;
    let managed = execution_policy
        .channels
        .iter()
        .map(|c| c.connection.provider_id.clone())
        .chain(removed.iter().cloned())
        .collect::<BTreeSet<_>>();
    let mut authorization_before = BTreeMap::new();
    for provider in managed.into_iter().filter(|provider| {
        before.providers["providers"].get(provider) != next_providers["providers"].get(provider)
            || credential_changes.iter().any(|c| &c.provider == provider)
            || imported_authorizations.contains_key(provider)
    }) {
        if let Some(removed) = removed_keys.iter().find(|r| r.provider == provider) {
            authorization_before.insert(provider, removed.backup.clone());
        } else if before.providers["providers"].get(&provider).is_none() {
            authorization_before.insert(provider, None);
        } else if !super::channel_actions::protected(&before, &provider) {
            if management.is_none() {
                management = Some(super::management::Management::bind(context).await?);
            }
            let keys = management.as_ref().unwrap().key_state(&provider).await?;
            authorization_before.insert(
                provider.clone(),
                super::provider_auth::capture(root, context, &provider, &keys).await?,
            );
        }
    }
    for (provider, target) in &imported_authorizations {
        if before.providers["providers"].get(provider).is_some() {
            super::provider_auth::review_change(
                root,
                context,
                provider,
                authorization_before.get(provider).and_then(Option::as_ref),
                Some(target),
            )?;
        }
    }
    let plan = RuntimePlan {
        id: uuid::Uuid::new_v4().to_string(),
        phase: "prepared".into(),
        expected_library,
        expected_policy,
        expected_workspace,
        policy,
        deferred_policy,
        scope,
        deferred_channels,
        before,
        next_providers,
        next_order,
        expected_slugs,
        runtime_executable: context.executable()?.to_string_lossy().into_owned(),
        error: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        ownership,
        previous_ownership: previous,
        observed_providers,
        observed_order,
        observed_disabled: disabled_before.clone(),
        disabled_before,
        credential_changes,
        execution_policy: Some(execution_policy),
        route_changes,
        recovery: Some(recovery),
        runtime_binding: Some(runtime_binding(context)?),
        removed_providers: removed.into_iter().collect(),
        removed_keys,
        committed_snapshot: None,
        authorization_before,
        authorization_after: BTreeMap::new(),
        history_restoration: None,
        imported_authorizations,
    };
    save(root, &plan)?;
    Ok(plan.view())
}
pub fn query(root: &Path, id: &str) -> ModelResult<SavePlanView> {
    Ok(load(root, id)?.view())
}
async fn capture_after(
    root: &Path,
    context: &ProcessContext,
    plan: &mut RuntimePlan,
) -> ModelResult<()> {
    let snapshot = plan
        .committed_snapshot
        .as_ref()
        .ok_or_else(|| invalid("history_snapshot", "成功运行读回缺失"))?;
    let mut values = BTreeMap::new();
    let mut management = None;
    for provider in plan.authorization_before.keys() {
        if snapshot.providers["providers"].get(provider).is_none() {
            values.insert(provider.clone(), None);
            continue;
        }
        if super::channel_actions::protected(snapshot, provider) {
            continue;
        }
        if management.is_none() {
            management = Some(super::management::Management::bind(context).await?);
        }
        let keys = management.as_ref().unwrap().key_state(provider).await?;
        values.insert(
            provider.clone(),
            super::provider_auth::capture(root, context, provider, &keys).await?,
        );
    }
    plan.authorization_after = values;
    save(root, plan)
}
fn direction_public(
    provider: &str,
    current: &Value,
    target: &Value,
    models: &Value,
) -> ModelResult<Value> {
    let mut staged = target.clone();
    for key in ["alias", "modelAliases", "defaultAliases"] {
        if let Some(value) = current["providers"][provider].get(key) {
            staged[key] = value.clone();
        } else {
            staged
                .as_object_mut()
                .ok_or_else(|| invalid("history_direction", "历史渠道字段无效"))?
                .remove(key);
        }
    }
    let exact = target["selectedModels"]
        .as_array()
        .or_else(|| target["models"].as_array())
        .map(|v| {
            v.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| {
            models
                .as_array()
                .into_iter()
                .flatten()
                .filter(|m| m["provider"] == provider)
                .filter_map(|m| m["id"].as_str().map(str::to_owned))
                .collect()
        });
    if exact.is_empty() {
        return Err(invalid(
            "history_direction",
            "历史服务方向缺少可核对的精确型号",
        ));
    }
    staged["selectedModels"] = json!(exact);
    staged["newModelPolicy"] = json!("off");
    // The single-provider mutation may temporarily disable the default row.
    // Keeping every direction staged off prevents intermediate alias/model
    // publication from exceeding the already-reviewed final catalogue budget.
    // The final provider batch restores the chosen default and enabled state.
    staged["disabled"] = json!(true);
    Ok(staged)
}
fn inspect_history_authorization(
    root: &Path,
    context: &ProcessContext,
    plan: &RuntimePlan,
    provider: &str,
    target: Option<&super::provider_auth::Backup>,
) -> ModelResult<()> {
    let before = plan
        .authorization_before
        .get(provider)
        .and_then(Option::as_ref);
    if let Some(direction) = plan
        .history_restoration
        .as_ref()
        .and_then(|h| h.directions.get(provider))
    {
        super::provider_auth::inspect_direction(
            root, context, &plan.id, provider, before, target, direction,
        )
    } else {
        super::provider_auth::inspect_change(root, context, &plan.id, provider, before, target)
    }
}
async fn apply_history_effects(
    root: &Path,
    context: &ProcessContext,
    plan: &mut RuntimePlan,
    before_path: &Path,
) -> ModelResult<()> {
    let history = plan.history_restoration.as_ref().unwrap().clone();
    let mut baseline = before_path.to_path_buf();
    if !history.registration_models.is_empty() {
        let mut staged = plan.before.providers.clone();
        for (provider, models) in &history.registration_models {
            let mut row = plan.next_providers["providers"][provider].clone();
            row["disabled"] = json!(true);
            row["newModelPolicy"] = json!("off");
            row["selectedModels"] = json!(models);
            staged["providers"][provider] = row;
        }
        plan.phase = "history_registering".into();
        save(root, plan)?;
        if !upstream::write(
            context,
            WriteOperation::Providers {
                baseline: &baseline,
                next: &staged,
            },
        )
        .await?
        .success
        {
            return Err(invalid(
                "history_registration",
                "历史停用渠道注册结果待核对",
            ));
        }
        observe(root, context, plan).await?;
        if plan.observed_providers != staged {
            return Err(invalid(
                "history_readback",
                "历史停用注册读回不同，未发送授权",
            ));
        }
        baseline = root
            .join(PLAN_DIR)
            .join(format!("{}-history-registered.json", plan.id));
        safety::check_path(root, &baseline, true)?;
        atomic_write(&baseline, &serde_json::to_vec(&staged)?, 0o600)?;
    }
    plan.phase = "history_authorizations".into();
    save(root, plan)?;
    for (provider, backup) in &history.authorizations {
        if plan.next_providers["providers"].get(provider).is_none() {
            continue;
        }
        if plan.before.providers["providers"].get(provider).is_some() {
            if let Some(direction) = history.directions.get(provider) {
                super::provider_auth::change_direction(
                    root,
                    context,
                    &plan.id,
                    provider,
                    plan.authorization_before
                        .get(provider)
                        .and_then(Option::as_ref),
                    backup.as_ref(),
                    direction,
                )
                .await?;
            } else {
                super::provider_auth::change(
                    root,
                    context,
                    &plan.id,
                    provider,
                    plan.authorization_before
                        .get(provider)
                        .and_then(Option::as_ref),
                    backup.as_ref(),
                )
                .await?;
            }
        } else if let Some(backup) = backup {
            super::provider_auth::restore(root, context, backup).await?;
        } else {
            super::provider_auth::verify_absent(context, provider)?;
        }
        observe(root, context, plan).await?;
    }
    if !history.directions.is_empty() {
        let mut expected = plan.before.providers.clone();
        for (provider, models) in &history.registration_models {
            let mut row = plan.next_providers["providers"][provider].clone();
            row["disabled"] = json!(true);
            row["newModelPolicy"] = json!("off");
            row["selectedModels"] = json!(models);
            expected["providers"][provider] = row;
        }
        for (provider, direction) in &history.directions {
            expected["providers"][provider] = direction.to_public.clone();
        }
        if plan.observed_providers != expected {
            return Err(invalid(
                "history_direction_drift",
                "服务方向恢复后存在其他公开字段变化，未继续",
            ));
        }
        baseline = root
            .join(PLAN_DIR)
            .join(format!("{}-history-direction.json", plan.id));
        safety::check_path(root, &baseline, true)?;
        atomic_write(&baseline, &serde_json::to_vec(&expected)?, 0o600)?;
    }
    plan.phase = "history_providers".into();
    save(root, plan)?;
    if !upstream::write(
        context,
        WriteOperation::Providers {
            baseline: &baseline,
            next: &plan.next_providers,
        },
    )
    .await?
    .success
    {
        return Err(invalid(
            "history_provider_write",
            "历史公开字段写入结果待核对",
        ));
    }
    observe(root, context, plan).await?;
    if plan.observed_providers != plan.next_providers {
        return Err(invalid("history_readback", "历史公开字段未准确读回"));
    }
    if !upstream::write(
        context,
        WriteOperation::DisabledModels(&history.projection.disabled),
    )
    .await?
    .success
    {
        return Err(invalid("history_visibility", "历史可见性结果待核对"));
    }
    observe(root, context, plan).await?;
    if plan.observed_disabled != history.projection.disabled {
        return Err(invalid("history_visibility", "历史可见性读回不同"));
    }
    if !upstream::write(context, WriteOperation::Order(&plan.next_order))
        .await?
        .success
    {
        return Err(invalid("history_order", "历史顺序结果待核对"));
    }
    observe(root, context, plan).await?;
    plan.phase = "order_written".into();
    save(root, plan)
}
async fn prepare_history(
    root: &Path,
    context: &ProcessContext,
    local: Snapshot,
    before: UpstreamSnapshot,
    policy: Policy,
    previous: Vec<OwnedField>,
    request: super::recovery::Request,
) -> ModelResult<SavePlanView> {
    let source = load(root, &request.source_id)?;
    if source.phase != "committed"
        || source.runtime_binding.as_ref() != Some(&runtime_binding(context)?)
    {
        return Err(invalid(
            "history_binding",
            "历史记录不属于当前成功运行来源，未生成保存计划",
        ));
    }
    let after = source.committed_snapshot.as_ref().ok_or_else(|| {
        invalid(
            "history_snapshot",
            "旧记录缺少成功后的完整官方快照，不能推算完整运行恢复",
        )
    })?;
    let managed = source
        .policy
        .channels
        .iter()
        .map(|c| c.connection.provider_id.clone())
        .chain(source.recovery.iter().flat_map(|b| {
            b.policy
                .channels
                .iter()
                .map(|c| c.connection.provider_id.clone())
        }))
        .chain(
            source
                .execution_policy
                .iter()
                .flat_map(|p| p.channels.iter().map(|c| c.connection.provider_id.clone())),
        )
        .chain(source.removed_providers.iter().cloned())
        .collect::<BTreeSet<_>>();
    let disabled =
        string_array(&upstream::read(context, upstream::ReadOperation::DisabledModels).await?)?;
    let authorization_changes = source
        .authorization_before
        .keys()
        .chain(source.authorization_after.keys())
        .filter(|provider| {
            source
                .authorization_before
                .get(*provider)
                .and_then(Option::as_ref)
                .map(|b| &b.digest)
                != source
                    .authorization_after
                    .get(*provider)
                    .and_then(Option::as_ref)
                    .map(|b| &b.digest)
        })
        .cloned()
        .collect::<BTreeSet<_>>();
    let projection = super::runtime_history::project(
        &before,
        &source.before,
        after,
        [
            &disabled,
            &source.disabled_before,
            &source.observed_disabled,
        ],
        request.side,
        super::runtime_history::RecoveryScope {
            managed: &managed,
            authorization_changes: &authorization_changes,
            all: request.scope == super::recovery::Scope::All,
        },
    )?;
    let target = if request.side == super::recovery::Side::Before {
        &source.before
    } else {
        after
    };
    let authorizations = if request.side == super::recovery::Side::Before {
        &source.authorization_before
    } else {
        &source.authorization_after
    };
    let mut target_auth = BTreeMap::new();
    let mut before_auth = BTreeMap::new();
    let mut removed_keys = vec![];
    let mut registration_models = BTreeMap::new();
    let mut directions = BTreeMap::new();
    let management = super::management::Management::bind(context).await?;
    for provider in &projection.affected {
        if super::channel_actions::protected(target, provider)
            || super::channel_actions::protected(&before, provider)
        {
            return Err(invalid(
                "history_protected",
                "此历史范围涉及原生或专用授权来源，不能作为普通自建渠道恢复",
            ));
        }
        let current = before.providers["providers"].get(provider);
        let desired = target.providers["providers"].get(provider);
        if let Some(backup) = authorizations.get(provider) {
            target_auth.insert(provider.clone(), backup.clone());
        } else if desired.is_some() && current != desired {
            return Err(invalid(
                "history_authorization",
                "历史目标缺少完整授权保护事实，请选择具备完整保护的成功记录",
            ));
        }
        if current == desired && !target_auth.contains_key(provider) {
            continue;
        }
        if current.is_some() {
            let keys = management.key_state(provider).await?;
            let backup = super::provider_auth::capture(root, context, provider, &keys).await?;
            if desired.is_none() {
                removed_keys.push(RemovedKey {
                    provider: provider.clone(),
                    before: keys,
                    reference: None,
                    backup: backup.clone(),
                });
            }
            before_auth.insert(provider.clone(), backup);
        } else {
            before_auth.insert(provider.clone(), None);
        }
        if let Some(Some(backup)) = target_auth.get(provider) {
            super::provider_auth::check_backup(root, backup)?;
        }
        if let (Some(current), Some(desired), true) =
            (current, desired, target_auth.contains_key(provider))
        {
            if super::provider_auth::different_direction(current, desired)? {
                let staged =
                    direction_public(provider, &before.providers, desired, &target.models)?;
                let direction = super::provider_auth::Direction {
                    from_public: current.clone(),
                    to_public: staged,
                };
                super::provider_auth::review_direction(
                    root,
                    context,
                    provider,
                    before_auth.get(provider).and_then(Option::as_ref),
                    target_auth.get(provider).and_then(Option::as_ref),
                    &direction,
                )?;
                directions.insert(provider.clone(), direction);
            } else {
                super::provider_auth::review_change(
                    root,
                    context,
                    provider,
                    before_auth.get(provider).and_then(Option::as_ref),
                    target_auth.get(provider).and_then(Option::as_ref),
                )?;
            }
        }
        if let Some(row) = desired.filter(|_| current.is_none()) {
            let values = row["selectedModels"]
                .as_array()
                .or_else(|| row["models"].as_array())
                .map(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let values = if values.is_empty() {
                target
                    .models
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|m| m["provider"] == provider.as_str())
                    .filter_map(|m| m["id"].as_str().map(str::to_owned))
                    .collect()
            } else {
                values
            };
            if values.is_empty() {
                return Err(invalid(
                    "history_registration",
                    "历史渠道没有可核对的精确型号，无法执行停用注册",
                ));
            }
            registration_models.insert(provider.clone(), values);
        }
    }
    let recovery = super::recovery::Baseline::capture_runtime(
        &local,
        before.providers["defaultProvider"].as_str(),
    )?;
    let plan = RuntimePlan {
        id: uuid::Uuid::new_v4().to_string(),
        phase: "prepared".into(),
        expected_library: local.library_digest,
        expected_policy: local.policy_digest,
        expected_workspace: local.workspace_digest,
        policy,
        deferred_policy: None,
        scope: None,
        deferred_channels: vec![],
        before: before.clone(),
        next_providers: projection.providers.clone(),
        next_order: projection.order.clone(),
        expected_slugs: projection.catalog.clone(),
        runtime_executable: context.executable()?.to_string_lossy().into_owned(),
        error: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        ownership: previous
            .iter()
            .filter(|f| !projection.affected.contains(&f.provider))
            .cloned()
            .collect(),
        previous_ownership: previous,
        disabled_before: disabled.clone(),
        observed_providers: before.providers.clone(),
        observed_order: string_array(&before.order["pickerOrder"])?,
        observed_disabled: disabled,
        credential_changes: vec![],
        execution_policy: None,
        route_changes: vec![],
        recovery: Some(recovery),
        runtime_binding: Some(runtime_binding(context)?),
        removed_providers: projection
            .affected
            .iter()
            .filter(|p| {
                projection.providers["providers"].get(*p).is_none()
                    && before.providers["providers"].get(*p).is_some()
            })
            .cloned()
            .collect(),
        removed_keys,
        committed_snapshot: None,
        authorization_before: before_auth,
        authorization_after: BTreeMap::new(),
        imported_authorizations: BTreeMap::new(),
        history_restoration: Some(HistoryRestoration {
            source_id: source.id,
            projection,
            authorizations: target_auth,
            registration_models,
            directions,
        }),
    };
    save(root, &plan)?;
    Ok(plan.view())
}
async fn history_auth_matches(
    root: &Path,
    context: &ProcessContext,
    plan: &RuntimePlan,
    providers: &Value,
    before: bool,
) -> ModelResult<bool> {
    let Some(history) = &plan.history_restoration else {
        return Ok(true);
    };
    for (provider, target) in &history.authorizations {
        let expected_rows = if before {
            &plan.before.providers
        } else {
            &plan.next_providers
        };
        let exists = providers["providers"].get(provider).is_some();
        if !exists {
            if expected_rows["providers"].get(provider).is_some() {
                return Ok(false);
            }
            continue;
        }
        if expected_rows["providers"].get(provider).is_none() {
            return Ok(false);
        }
        let backup = if before {
            plan.authorization_before
                .get(provider)
                .and_then(Option::as_ref)
        } else {
            target.as_ref()
        };
        if !super::provider_auth::matches(root, context, provider, backup).await? {
            return Ok(false);
        }
    }
    Ok(true)
}
pub async fn reconcile(
    root: &Path,
    context: &ProcessContext,
    id: &str,
) -> ModelResult<SavePlanView> {
    let _execution = execution_lock(root)?;
    let file = path(root, id)?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&file),
        true,
    )?;
    let _guard = TargetFileLock::lock(&file)?;
    let mut plan = load(root, id)?;
    if ["committed", "compensated", "prepared"].contains(&plan.phase.as_str()) {
        return Ok(plan.view());
    }
    if context.executable()?.to_string_lossy() != plan.runtime_executable
        || plan
            .runtime_binding
            .as_ref()
            .is_some_and(|binding| runtime_binding(context).as_ref() != Ok(binding))
    {
        return Err(invalid("runtime_binding", "运行来源改变，原计划仍保留"));
    }
    let actual = upstream::snapshot(context).await?;
    if !plan.credential_changes.is_empty() {
        let management = super::management::Management::bind(context).await?;
        for change in &mut plan.credential_changes {
            if actual.providers["providers"]
                .get(&change.provider)
                .is_none()
            {
                change.observed = Some(change.before.clone());
                continue;
            }
            let current = management.key_state(&change.provider).await?;
            if current != change.intended() && current != change.before {
                return Err(invalid(
                    "credential_drift",
                    "密钥集合有无法归于本次保存的变化，未更新本地策略",
                ));
            }
            change.observed = Some(current);
        }
    }
    let disabled =
        string_array(&upstream::read(context, upstream::ReadOperation::DisabledModels).await?)?;
    let removed_match = removed_key_states(root, &plan, context, &actual.providers).await?;
    if let Some(history) = &plan.history_restoration {
        for (provider, target) in &history.authorizations {
            if plan.before.providers["providers"].get(provider).is_some()
                && plan.next_providers["providers"].get(provider).is_some()
            {
                inspect_history_authorization(root, context, &plan, provider, target.as_ref())?;
            }
        }
    }
    let history_target_matches =
        history_auth_matches(root, context, &plan, &actual.providers, false).await?;
    let history_before_matches =
        history_auth_matches(root, context, &plan, &actual.providers, true).await?;
    let mut imported_target_matches = true;
    let mut imported_before_matches = true;
    for backup in plan.imported_authorizations.values() {
        if actual.providers["providers"]
            .get(&backup.provider)
            .is_some()
        {
            super::provider_auth::inspect_change(
                root,
                context,
                &plan.id,
                &backup.provider,
                plan.authorization_before
                    .get(&backup.provider)
                    .and_then(Option::as_ref),
                Some(backup),
            )?;
            imported_target_matches &=
                super::provider_auth::matches(root, context, &backup.provider, Some(backup))
                    .await?;
            imported_before_matches &= plan.before.providers["providers"]
                .get(&backup.provider)
                .is_some()
                && super::provider_auth::matches(
                    root,
                    context,
                    &backup.provider,
                    plan.authorization_before
                        .get(&backup.provider)
                        .and_then(Option::as_ref),
                )
                .await?;
        } else {
            imported_target_matches = false;
            imported_before_matches &= plan.before.providers["providers"]
                .get(&backup.provider)
                .is_none();
        }
    }

    let mut staged = plan.before.providers.clone();
    for channel in &plan.execution_policy().channels {
        if plan.before.providers["providers"]
            .get(&channel.connection.provider_id)
            .is_none()
        {
            let mut value =
                plan.next_providers["providers"][&channel.connection.provider_id].clone();
            value["disabled"] = json!(true);
            value["newModelPolicy"] = json!("off");
            if channel.connection.imported_config.is_some() {
                value["selectedModels"] = json!(channel
                    .discovered
                    .iter()
                    .map(|m| m.id.clone())
                    .collect::<Vec<_>>());
            }
            staged["providers"]
                .as_object_mut()
                .ok_or_else(|| invalid("provider", "渠道结构无效"))?
                .insert(channel.connection.provider_id.clone(), value);
        }
    }
    if let Some(history) = &plan.history_restoration {
        for (provider, models) in &history.registration_models {
            let mut row = plan.next_providers["providers"][provider].clone();
            row["disabled"] = json!(true);
            row["newModelPolicy"] = json!("off");
            row["selectedModels"] = json!(models);
            staged["providers"][provider] = row;
        }
    }
    let directional = if let Some(history) = &plan.history_restoration {
        if history.directions.is_empty() {
            false
        } else {
            let all = provider_map(&plan.before.providers)?
                .keys()
                .chain(provider_map(&plan.next_providers)?.keys())
                .cloned()
                .collect::<BTreeSet<_>>();
            provider_map(&actual.providers)?
                .keys()
                .all(|p| all.contains(p))
                && all.iter().all(|provider| {
                    let row = actual.providers["providers"].get(provider);
                    row == plan.before.providers["providers"].get(provider)
                        || row == plan.next_providers["providers"].get(provider)
                        || row == staged["providers"].get(provider)
                        || history
                            .directions
                            .get(provider)
                            .is_some_and(|d| row == Some(&d.to_public))
                })
                && [
                    plan.before.providers.get("defaultProvider"),
                    plan.next_providers.get("defaultProvider"),
                ]
                .contains(&actual.providers.get("defaultProvider"))
        }
    } else {
        false
    };
    let attributable = actual.providers == plan.before.providers
        || actual.providers == plan.next_providers
        || actual.providers == staged
        || directional;
    let original_order = string_array(&plan.before.order["pickerOrder"])?;
    let actual_order = string_array(&actual.order["pickerOrder"])?;
    let changed: BTreeSet<_> = disabled
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .symmetric_difference(&plan.disabled_before.iter().cloned().collect())
        .cloned()
        .collect();
    let mut model_slugs: BTreeSet<_> = rows(&plan.before)?
        .iter()
        .filter(|m| {
            plan.execution_policy()
                .channels
                .iter()
                .any(|c| m["provider"].as_str() == Some(&c.connection.provider_id))
        })
        .filter_map(|m| m["namespaced"].as_str().map(str::to_owned))
        .collect();
    if let Some(history) = &plan.history_restoration {
        model_slugs.extend(history.projection.disabled.iter().cloned());
        model_slugs.extend(
            plan.disabled_before
                .iter()
                .filter(|slug| {
                    slug.split_once('/').is_some_and(|(p, _)| {
                        history
                            .projection
                            .affected
                            .iter()
                            .any(|provider| provider == p)
                    })
                })
                .cloned(),
        );
    }
    if !attributable
        || ![&original_order, &plan.next_order].contains(&&actual_order)
        || !changed.is_subset(&model_slugs)
    {
        return Err(invalid(
            "reconcile_drift",
            "发现无法归于本次计划的修改，未改写运行态或基准",
        ));
    }
    plan.observed_providers = actual.providers.clone();
    plan.observed_order = actual_order.clone();
    plan.observed_disabled = disabled;
    plan.phase = "verified_partial".into();
    if history_target_matches
        && imported_target_matches
        && actual.providers == plan.next_providers
        && actual_order == plan.next_order
        && catalog_slugs(&actual)? == plan.expected_slugs
        && plan
            .history_restoration
            .as_ref()
            .is_none_or(|h| h.projection.disabled == plan.observed_disabled)
        && plan
            .credential_changes
            .iter()
            .all(|change| change.observed.as_ref() == Some(&change.intended()))
    {
        plan.committed_snapshot = Some(actual.clone());
        verify_committed_credentials(root, &plan, context).await?;
        capture_after(root, context, &mut plan).await?;
        ModelStore::new(root).commit_runtime_scope(
            &plan.expected_library,
            &plan.expected_policy,
            &plan.expected_workspace,
            plan.policy.published(),
            plan.deferred_policy.as_ref(),
        )?;
        let ownership = root.join(OWNERSHIP);
        safety::check_path(root, &ownership, true)?;
        atomic_write(&ownership, &serde_json::to_vec(&plan.ownership)?, 0o600)?;
        plan.phase = "committed".into();
        plan.error = None;
    } else if removed_match
        && history_before_matches
        && imported_before_matches
        && actual.providers == plan.before.providers
        && actual_order == original_order
        && plan.observed_disabled == plan.disabled_before
        && plan
            .credential_changes
            .iter()
            .all(|change| change.observed.as_ref() == Some(&change.before))
    {
        plan.phase = "compensated".into();
        plan.error = None;
    }
    save(root, &plan)?;
    Ok(plan.view())
}
pub async fn apply(root: &Path, context: &ProcessContext, id: &str) -> ModelResult<SavePlanView> {
    let _execution = execution_lock(root)?;
    reject_incomplete(root)?;
    let bound_executable = context.executable()?;
    let frozen = ProcessContext {
        runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(
            bound_executable.clone(),
        ),
        working_directory: context.working_directory.clone(),
        opencodex_home: context.opencodex_home.clone(),
    };
    let file = path(root, id)?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&file),
        true,
    )?;
    let _guard = TargetFileLock::lock(&file)?;
    let mut plan = load(root, id)?;
    if plan.phase == "committed" {
        return Ok(plan.view());
    }
    if plan.phase != "prepared" {
        return Err(invalid(
            "plan_incomplete",
            "此计划已有执行记录，请先核对原结果，不重复写入",
        ));
    }
    if context.executable()?.to_string_lossy() != plan.runtime_executable
        || plan
            .runtime_binding
            .as_ref()
            .is_some_and(|binding| runtime_binding(context).as_ref() != Ok(binding))
    {
        return Err(invalid("runtime_binding", "运行来源已变化，确认过期"));
    }
    let local = ModelStore::new(root).load()?;
    if local.library_digest != plan.expected_library
        || local.policy_digest != plan.expected_policy
        || local.workspace_digest != plan.expected_workspace
    {
        return Err(invalid(
            "stale_baseline",
            "确认后本地配置已变化，未执行保存",
        ));
    }
    let current = upstream::snapshot(&frozen).await?;
    for backup in plan.imported_authorizations.values() {
        super::provider_auth::check_backup(root, backup)?;
    }
    for (provider, backup) in &plan.authorization_before {
        if plan.before.providers["providers"].get(provider).is_none() {
            continue;
        }
        match backup {
            Some(backup) => super::provider_auth::verify(root, &frozen, backup).await?,
            None => super::provider_auth::verify_absent(&frozen, provider)?,
        }
    }
    if !plan.removed_keys.is_empty() {
        let management = super::management::Management::bind(&frozen).await?;
        for removed in &plan.removed_keys {
            if let Some(backup) = &removed.backup {
                super::provider_auth::verify(root, &frozen, backup).await?;
            } else if management.key_state(&removed.provider).await? != removed.before {
                return Err(invalid(
                    "removal_key_drift",
                    "删除确认后Key池或活动选择改变，未删除",
                ));
            } else if removed.before.ids.is_empty() && removed.reference.is_none() {
                super::provider_auth::verify_absent(&frozen, &removed.provider)?;
            }
        }
    }
    if current.providers != plan.before.providers
        || current.order != plan.before.order
        || catalog_slugs(&current)? != catalog_slugs(&plan.before)?
        || current.models != plan.before.models
    {
        return Err(invalid(
            "stale_upstream",
            "官方配置或发现目录已变化，确认失效",
        ));
    }
    let directory = root.join(PLAN_DIR);
    let before_path = directory.join(format!("{id}-providers-before.json"));
    safety::check_path(root, &before_path, true)?;
    atomic_write(
        &before_path,
        &serde_json::to_vec_pretty(&plan.before.providers)?,
        0o600,
    )?;
    backup_file(
        root,
        BackupAction::ModelConfig,
        &before_path,
        &serde_json::to_vec_pretty(&plan.before.providers)?,
        chrono::Utc::now(),
        Some("渠道保存前官方非敏感字段与原顺序保存在同一计划".into()),
    )?;
    plan.phase = "applying".into();
    save(root, &plan)?;
    let result: ModelResult<()> = async {
        if plan.history_restoration.is_some() {
            apply_history_effects(root, &frozen, &mut plan, &before_path).await?;
        } else {
            // Clearing a whitelist must not briefly publish every model. Hide the
            // frozen known rows first; new discoveries remain off under the provider
            // policy, and every effect is journaled for exact compensation.
            let none_channels = plan
                .execution_policy()
                .channels
                .iter()
                .filter(|c| matches!(c.selection, ModelSelection::None))
                .map(|c| c.connection.provider_id.clone())
                .collect::<BTreeSet<_>>();
            let hidden = rows(&plan.before)?
                .iter()
                .filter(|row| {
                    row["native"] != true
                        && row["disabled"] != true
                        && row["provider"]
                            .as_str()
                            .is_some_and(|provider| none_channels.contains(provider))
                })
                .filter_map(|row| row["namespaced"].as_str().map(str::to_owned))
                .collect::<Vec<_>>();
            if !hidden.is_empty() {
                plan.phase = "hiding_none_selection".into();
                save(root, &plan)?;
                for slug in hidden {
                    let receipt =
                        upstream::write(&frozen, WriteOperation::DisableModel(&slug)).await?;
                    if !receipt.success {
                        return Err(invalid(
                            "visibility_write",
                            "隐藏当前模型结果待核对，未清空白名单",
                        ));
                    }
                    observe(root, &frozen, &mut plan).await?;
                }
            }
            let new_channels: Vec<_> = plan
                .execution_policy()
                .channels
                .iter()
                .filter(|c| {
                    plan.before.providers["providers"]
                        .get(&c.connection.provider_id)
                        .is_none()
                })
                .collect();
            let mut final_baseline = before_path.clone();
            if !new_channels.is_empty() {
                let mut staged = plan.before.providers.clone();
                for channel in &new_channels {
                    let mut value =
                        plan.next_providers["providers"][&channel.connection.provider_id].clone();
                    value["disabled"] = json!(true);
                    value["newModelPolicy"] = json!("off");
                    if channel.connection.imported_config.is_some() {
                        let exact = channel
                            .discovered
                            .iter()
                            .map(|m| m.id.clone())
                            .collect::<Vec<_>>();
                        if exact.is_empty() {
                            return Err(invalid(
                                "provider_import_models",
                                "导入原渠道没有可核对的精确型号，请先补充型号",
                            ));
                        }
                        value["selectedModels"] = json!(exact);
                    }
                    staged["providers"]
                        .as_object_mut()
                        .ok_or_else(|| invalid("provider", "渠道结构无效"))?
                        .insert(channel.connection.provider_id.clone(), value);
                }
                plan.phase = "registering_new".into();
                save(root, &plan)?;
                let result = upstream::write(
                    &frozen,
                    WriteOperation::Providers {
                        baseline: &before_path,
                        next: &staged,
                    },
                )
                .await?;
                if !result.success {
                    return Err(invalid(
                        "registration_unknown",
                        "新渠道注册结果待核对，未按成功继续",
                    ));
                }
                let readback =
                    upstream::read(&frozen, upstream::ReadOperation::ProviderSnapshot).await?;
                if readback != staged {
                    return Err(invalid(
                        "registration_readback",
                        "新渠道停用注册的读回不同，未发送密钥",
                    ));
                }
                plan.phase = "new_registered".into();
                save(root, &plan)?;
                let current =
                    upstream::read(&frozen, upstream::ReadOperation::ProviderSnapshot).await?;
                if current != staged {
                    return Err(invalid(
                        "registration_drift",
                        "密钥接入期间官方渠道字段改变，未启用新渠道",
                    ));
                }
                final_baseline = root.join(PLAN_DIR).join(format!("{id}-registered.json"));
                safety::check_path(root, &final_baseline, true)?;
                atomic_write(
                    &final_baseline,
                    &serde_json::to_vec_pretty(&current)?,
                    0o600,
                )?;
                plan.phase = "credentials_registered".into();
                save(root, &plan)?;
            }
            if !plan.imported_authorizations.is_empty() {
                plan.phase = "writing_imported_authorizations".into();
                save(root, &plan)?;
                for backup in plan.imported_authorizations.values() {
                    super::provider_auth::change(
                        root,
                        &frozen,
                        &plan.id,
                        &backup.provider,
                        plan.authorization_before
                            .get(&backup.provider)
                            .and_then(Option::as_ref),
                        Some(backup),
                    )
                    .await?;
                }
                observe(root, &frozen, &mut plan).await?;
            }
            if !plan.credential_changes.is_empty() {
                let management = super::management::Management::bind(&frozen).await?;
                for index in 0..plan.credential_changes.len() {
                    let change = plan.credential_changes[index].clone();
                    if management.key_state(&change.provider).await? != change.before {
                        return Err(invalid(
                            "credential_drift",
                            "确认后活动密钥或集合改变，未覆盖",
                        ));
                    }
                    let key = super::discovery::credential(&change.reference)?;
                    if super::store::sha256(key.trim().as_bytes()) != change.intended_digest {
                        return Err(invalid("credential_drift", "本机密钥引用内容变化，未发送"));
                    }
                    plan.phase = "writing_credentials".into();
                    save(root, &plan)?;
                    management.add_key(&change.provider, &key).await?;
                    let observed = management.key_state(&change.provider).await?;
                    if observed != change.intended() {
                        return Err(invalid(
                            "credential_readback",
                            "密钥写后状态不同，原计划待核对",
                        ));
                    }
                    plan.credential_changes[index].observed = Some(observed);
                    save(root, &plan)?;
                }
                plan.phase = "credentials_verified".into();
                save(root, &plan)?;
            }
            let providers = upstream::write(
                &frozen,
                WriteOperation::Providers {
                    baseline: &final_baseline,
                    next: &plan.next_providers,
                },
            )
            .await?;
            if !providers.success {
                return Err(invalid(
                    "provider_write",
                    "官方渠道写入返回未完成；请按原计划核对，不盲目重试",
                ));
            }
            plan.phase = "providers_written".into();
            observe(root, &frozen, &mut plan).await?;
            save(root, &plan)?;
            let visibility_channels = plan.execution_policy().channels.clone();
            let visibility_rows = rows(&plan.before)?.clone();
            for channel in &visibility_channels {
                if !channel.connection.enabled {
                    continue;
                }
                for row in visibility_rows.iter().filter(|r| {
                    r["provider"].as_str() == Some(&channel.connection.provider_id)
                        && r["native"] != true
                }) {
                    let id = row["id"]
                        .as_str()
                        .ok_or_else(|| invalid("model_identity", "模型原始身份缺失"))?;
                    let enable = match &channel.selection {
                        ModelSelection::All => true,
                        ModelSelection::None => false,
                        ModelSelection::Whitelist(values) => values.iter().any(|v| v == id),
                    };
                    // A whitelist already excludes unselected IDs; only none needs explicit disables.
                    if !enable && !matches!(channel.selection, ModelSelection::None) {
                        continue;
                    }
                    if enable && row["disabled"] == false {
                        continue;
                    }
                    if !enable && row["disabled"] == true {
                        continue;
                    }
                    let slug = row["namespaced"]
                        .as_str()
                        .ok_or_else(|| invalid("model_identity", "模型缺少官方路由身份"))?;
                    let result = upstream::write(
                        &frozen,
                        if enable {
                            WriteOperation::EnableModel(slug)
                        } else {
                            WriteOperation::DisableModel(slug)
                        },
                    )
                    .await?;
                    if !result.success {
                        return Err(invalid(
                            "visibility_write",
                            "模型可见性更新未完成，结果待核对",
                        ));
                    }
                    observe(root, &frozen, &mut plan).await?;
                }
            }
            plan.phase = "visibility_written".into();
            save(root, &plan)?;
            let observed_order =
                upstream::read(&frozen, upstream::ReadOperation::ModelOrder).await?;
            if observed_order["pickerOrder"] != plan.before.order["pickerOrder"]
                || observed_order["pickerOrderMode"] != plan.before.order["pickerOrderMode"]
            {
                return Err(invalid("order_drift", "保存期间检测到外部排序修改，未覆盖"));
            }
            if !upstream::write(&frozen, WriteOperation::Order(&plan.next_order))
                .await?
                .success
            {
                return Err(invalid("order_write", "选择器顺序更新未完成，结果待核对"));
            }
            plan.phase = "order_written".into();
            observe(root, &frozen, &mut plan).await?;
            save(root, &plan)?;
        }
        if !upstream::write(&frozen, WriteOperation::Sync)
            .await?
            .success
        {
            return Err(invalid("catalog_write", "目录刷新未完成，结果待核对"));
        }
        let actual = upstream::snapshot(&frozen).await?;
        if actual.providers != plan.next_providers {
            return Err(invalid(
                "provider_readback",
                "写后官方渠道字段与确认内容不一致，结果待核对",
            ));
        }
        if context.executable()? != bound_executable {
            return Err(invalid(
                "runtime_binding",
                "保存期间运行来源改变，结果待核对",
            ));
        }
        if catalog_slugs(&actual)? != plan.expected_slugs {
            return Err(invalid(
                "catalog_readback",
                "写后完整目录与确认投影不同，未晋升成功；保留原计划供核对/补偿",
            ));
        }
        if actual.order["pickerOrder"] != json!(plan.next_order) {
            return Err(invalid(
                "order_readback",
                "写后顺序与确认内容不同，结果待核对",
            ));
        }
        verify_committed_credentials(root, &plan, &frozen).await?;
        plan.phase = "runtime_verified".into();
        plan.committed_snapshot = Some(actual.clone());
        save(root, &plan)?;
        capture_after(root, &frozen, &mut plan).await?;
        let ownership_path = root.join(OWNERSHIP);
        safety::check_path(root, &ownership_path, true)?;
        atomic_write(
            &ownership_path,
            &serde_json::to_vec(&plan.ownership)?,
            0o600,
        )?;
        ModelStore::new(root).commit_runtime_scope(
            &plan.expected_library,
            &plan.expected_policy,
            &plan.expected_workspace,
            plan.policy.published(),
            plan.deferred_policy.as_ref(),
        )?;
        plan.phase = "committed".into();
        save(root, &plan)?;
        Ok(())
    }
    .await;
    if let Err(error) = result {
        plan.error = Some(error.code);
        save(root, &plan)?;
        return Ok(plan.view());
    }
    Ok(plan.view())
}
fn next_ownership(
    before: &Value,
    after: &Value,
    previous: &[OwnedField],
    policy: &Policy,
) -> ModelResult<Vec<OwnedField>> {
    let mut result: Vec<_> = previous
        .iter()
        .filter(|f| {
            !policy
                .channels
                .iter()
                .any(|c| c.connection.provider_id == f.provider)
        })
        .filter(|f| after["providers"].get(&f.provider).is_some())
        .cloned()
        .collect();
    for channel in &policy.channels {
        let provider = &channel.connection.provider_id;
        // Empty model is a reserved provider-field slot: real raw IDs are nonempty.
        // Whole-list ownership keeps order and absence exact and refuses later edits.
        for key in [
            "noReasoningModels",
            "modelReasoningEffortsAuthoritative",
            "fastWire",
        ] {
            let old = before["providers"][provider].get(key);
            let new = after["providers"][provider].get(key);
            let prior = previous
                .iter()
                .find(|f| f.provider == *provider && f.key == key && f.model.is_empty());
            let original = prior
                .map(|f| f.original.clone())
                .unwrap_or_else(|| old.cloned());
            if new != original.as_ref() {
                result.push(OwnedField {
                    provider: provider.clone(),
                    key: key.into(),
                    model: String::new(),
                    original,
                    written: new.cloned().unwrap_or(Value::Null),
                    removed: new.is_none(),
                });
            }
        }
        for key in [
            "modelDisplayNames",
            "modelAliases",
            "modelAdapters",
            "modelContextWindows",
            "modelInputModalities",
            "modelReasoningEfforts",
            "modelDefaultReasoningEfforts",
            "modelSupportsServiceTier",
            "modelPinnedReasoningEfforts",
            "modelReasoningEffortMap",
        ] {
            let old = before["providers"][provider][key].as_object();
            let new = after["providers"][provider][key].as_object();
            let keys: BTreeSet<_> = old
                .into_iter()
                .flat_map(|m| m.keys())
                .chain(new.into_iter().flat_map(|m| m.keys()))
                .collect();
            for model in keys {
                let a = old.and_then(|m| m.get(model));
                let b = new.and_then(|m| m.get(model));
                let prior = previous
                    .iter()
                    .find(|f| f.provider == *provider && f.key == key && f.model == *model);
                let original = prior
                    .map(|f| f.original.clone())
                    .unwrap_or_else(|| a.cloned());
                if b != original.as_ref() {
                    result.push(OwnedField {
                        provider: provider.clone(),
                        key: key.into(),
                        model: model.clone(),
                        original,
                        written: b.cloned().unwrap_or(Value::Null),
                        removed: b.is_none(),
                    });
                }
            }
        }
    }
    Ok(result)
}
fn string_array(value: &Value) -> ModelResult<Vec<String>> {
    value
        .as_array()
        .ok_or_else(|| invalid("runtime_shape", "运行字段不是数组"))?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or_else(|| invalid("runtime_shape", "运行数组包含非字符串"))
        })
        .collect()
}
async fn observe(root: &Path, context: &ProcessContext, plan: &mut RuntimePlan) -> ModelResult<()> {
    plan.observed_providers =
        upstream::read(context, upstream::ReadOperation::ProviderSnapshot).await?;
    plan.observed_order = string_array(
        &upstream::read(context, upstream::ReadOperation::ModelOrder).await?["pickerOrder"],
    )?;
    plan.observed_disabled =
        string_array(&upstream::read(context, upstream::ReadOperation::DisabledModels).await?)?;
    save(root, plan)
}
pub async fn compensate(
    root: &Path,
    context: &ProcessContext,
    id: &str,
) -> ModelResult<SavePlanView> {
    let _execution = execution_lock(root)?;
    let file = path(root, id)?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&file),
        true,
    )?;
    let _guard = TargetFileLock::lock(&file)?;
    let mut plan = load(root, id)?;
    if plan.phase == "compensated" {
        return Ok(plan.view());
    }
    if ["prepared", "committed"].contains(&plan.phase.as_str()) {
        return Err(invalid(
            "compensation_scope",
            "此计划无需失败补偿；已提交配置请经历史转草稿恢复",
        ));
    }
    if context.executable()?.to_string_lossy() != plan.runtime_executable
        || plan
            .runtime_binding
            .as_ref()
            .is_some_and(|binding| runtime_binding(context).as_ref() != Ok(binding))
    {
        return Err(invalid("runtime_binding", "运行来源改变，补偿已拒绝"));
    }
    let mut current = upstream::read(context, upstream::ReadOperation::ProviderSnapshot).await?;
    removed_key_states(root, &plan, context, &current).await?;
    for backup in plan.imported_authorizations.values() {
        if current["providers"].get(&backup.provider).is_some() {
            super::provider_auth::inspect_change(
                root,
                context,
                &plan.id,
                &backup.provider,
                plan.authorization_before
                    .get(&backup.provider)
                    .and_then(Option::as_ref),
                Some(backup),
            )?;
        }
    }
    if let Some(history) = &plan.history_restoration {
        for (provider, target) in &history.authorizations {
            if plan.before.providers["providers"].get(provider).is_some()
                && plan.next_providers["providers"].get(provider).is_some()
            {
                inspect_history_authorization(root, context, &plan, provider, target.as_ref())?;
            }
        }
    }

    let order = string_array(
        &upstream::read(context, upstream::ReadOperation::ModelOrder).await?["pickerOrder"],
    )?;
    let disabled =
        string_array(&upstream::read(context, upstream::ReadOperation::DisabledModels).await?)?;
    if current != plan.observed_providers
        || order != plan.observed_order
        || disabled != plan.observed_disabled
    {
        return Err(invalid(
            "compensation_drift",
            "当前运行态不同于本次最后核验的副作用；为保留后来修改，拒绝自动补偿",
        ));
    }
    if !plan.credential_changes.is_empty() {
        let management = super::management::Management::bind(context).await?;
        for change in &plan.credential_changes {
            if current["providers"].get(&change.provider).is_none() {
                continue;
            }
            let state = management.key_state(&change.provider).await?;
            if !change.compensation_owned(&state) {
                return Err(invalid(
                    "credential_drift",
                    "密钥存在后来修改或未核验副作用，拒绝自动补偿",
                ));
            }
        }
        for change in &plan.credential_changes {
            if current["providers"].get(&change.provider).is_none() {
                continue;
            }
            if management.key_state(&change.provider).await? == change.before {
                continue;
            }
            if let Some(active) = &change.before.active_id {
                management.activate_key(&change.provider, active).await?;
            }
            if !change.before.ids.contains(&change.intended_id) {
                management
                    .remove_key(&change.provider, &change.intended_id)
                    .await?;
            }
            if management.key_state(&change.provider).await? != change.before {
                return Err(invalid(
                    "credential_compensation",
                    "密钥补偿读回不一致，保留原计划",
                ));
            }
        }
    }
    if let Some(history) = plan.history_restoration.clone() {
        for (provider, direction) in &history.directions {
            let restore = direction_public(
                provider,
                &current,
                &plan.before.providers["providers"][provider],
                &plan.before.models,
            )?;
            plan.phase = "compensating_direction".into();
            save(root, &plan)?;
            let result = super::provider_auth::rollback_direction(
                root,
                context,
                &plan.id,
                provider,
                plan.authorization_before
                    .get(provider)
                    .and_then(Option::as_ref),
                history
                    .authorizations
                    .get(provider)
                    .and_then(Option::as_ref),
                direction,
                &restore,
            )
            .await;
            observe(root, context, &mut plan).await?;
            if let Err(error) = result {
                plan.error = Some(error.code);
                save(root, &plan)?;
                return Ok(plan.view());
            }
            current = plan.observed_providers.clone();
        }
    }
    let baseline = root
        .join(PLAN_DIR)
        .join(format!("{id}-compensation-baseline.json"));
    safety::check_path(root, &baseline, true)?;
    atomic_write(&baseline, &serde_json::to_vec_pretty(&current)?, 0o600)?;
    plan.phase = "compensating".into();
    save(root, &plan)?;
    let result: ModelResult<()> = async {
        if current != plan.before.providers
            && !upstream::write(
                context,
                WriteOperation::Providers {
                    baseline: &baseline,
                    next: &plan.before.providers,
                },
            )
            .await?
            .success
        {
            return Err(invalid("compensation_failed", "渠道补偿未完成，保留原计划"));
        }
        observe(root, context, &mut plan).await?;
        for backup in plan.imported_authorizations.values() {
            if plan.before.providers["providers"]
                .get(&backup.provider)
                .is_some()
            {
                super::provider_auth::rollback_change(
                    root,
                    context,
                    &plan.id,
                    &backup.provider,
                    plan.authorization_before
                        .get(&backup.provider)
                        .and_then(Option::as_ref),
                    Some(backup),
                )
                .await?;
            }
        }
        if let Some(history) = &plan.history_restoration {
            for (provider, target) in &history.authorizations {
                if !history.directions.contains_key(provider)
                    && plan.before.providers["providers"].get(provider).is_some()
                    && plan.next_providers["providers"].get(provider).is_some()
                {
                    super::provider_auth::rollback_change(
                        root,
                        context,
                        &plan.id,
                        provider,
                        plan.authorization_before
                            .get(provider)
                            .and_then(Option::as_ref),
                        target.as_ref(),
                    )
                    .await?;
                }
            }
        }

        if !plan.removed_keys.is_empty() {
            let management = super::management::Management::bind(context).await?;
            for removed in &plan.removed_keys {
                if let Some(backup) = &removed.backup {
                    super::provider_auth::restore(root, context, backup).await?;
                    continue;
                }

                let current = management.key_state(&removed.provider).await?;
                if current == removed.before {
                    continue;
                }
                if !current.ids.is_empty() {
                    return Err(invalid("credential_drift", "恢复时发现新的Key，未覆盖"));
                }
                if let Some(reference) = &removed.reference {
                    let secret = super::discovery::credential(reference)?;
                    management
                        .add_key(&removed.provider, secret.as_str())
                        .await?;
                }
                if management.key_state(&removed.provider).await? != removed.before {
                    return Err(invalid(
                        "credential_compensation",
                        "恢复Key未读回原选择，保留原计划",
                    ));
                }
            }
        }
        if plan.observed_disabled != plan.disabled_before
            && !upstream::write(
                context,
                WriteOperation::DisabledModels(&plan.disabled_before),
            )
            .await?
            .success
        {
            return Err(invalid("compensation_failed", "可见性补偿未完成"));
        }
        observe(root, context, &mut plan).await?;
        let original_order = string_array(&plan.before.order["pickerOrder"])?;
        if plan.observed_order != original_order
            && !upstream::write(context, WriteOperation::Order(&original_order))
                .await?
                .success
        {
            return Err(invalid("compensation_failed", "顺序补偿未完成"));
        }
        if !upstream::write(context, WriteOperation::Sync)
            .await?
            .success
        {
            return Err(invalid("compensation_failed", "补偿后目录核对未完成"));
        }
        let actual = upstream::snapshot(context).await?;
        if actual.providers != plan.before.providers
            || catalog_slugs(&actual)? != catalog_slugs(&plan.before)?
            || actual.order["pickerOrder"] != plan.before.order["pickerOrder"]
        {
            return Err(invalid(
                "compensation_readback",
                "补偿结果仍不同，保持待核对",
            ));
        }
        let ownership = root.join(OWNERSHIP);
        safety::check_path(root, &ownership, true)?;
        atomic_write(
            &ownership,
            &serde_json::to_vec(&plan.previous_ownership)?,
            0o600,
        )?;
        plan.phase = "compensated".into();
        plan.error = None;
        observe(root, context, &mut plan).await?;
        Ok(())
    }
    .await;
    if let Err(error) = result {
        plan.error = Some(error.code);
        save(root, &plan)?;
    }
    Ok(plan.view())
}

#[cfg(test)]
mod tests {
    #[test]
    fn scoped_save_keeps_unselected_changes_and_requires_explicit_shared_scope() {
        let (_, mut saved) = fixture();
        let first = saved.channels[0].id.clone();
        let mut other = saved.channels[0].clone();
        other.id = uuid::Uuid::new_v4().to_string();
        other.connection.provider_id = "other".into();
        other.label = "以后再改".into();
        saved.channels.push(other.clone());
        let mut draft = saved.clone();
        draft.channels[0].label = "本次修改".into();
        draft.channels[1].label = "暂缓修改".into();
        draft.channels.reverse();
        draft.default_channel_id = Some(other.id.clone());
        let (selected, remaining) = select_save_scope(
            &saved,
            &draft,
            &SaveScope {
                channel_ids: vec![first.clone()],
                include_shared: false,
            },
        )
        .unwrap();
        assert_eq!(selected.channels[0].label, "本次修改");
        assert_eq!(selected.channels[1], other);
        // 默认渠道已退役：草稿里的残留值既不进入本次范围，也不被写回。
        assert_eq!(selected.default_channel_id, saved.default_channel_id);
        assert_eq!(remaining, Some(draft.clone()));
        let (shared, _) = select_save_scope(
            &saved,
            &draft,
            &SaveScope {
                channel_ids: vec![first.clone()],
                include_shared: true,
            },
        )
        .unwrap();
        // 共享范围只带全局模版采用与渠道顺序，不再复制默认渠道。
        assert_eq!(shared.default_channel_id, saved.default_channel_id);
        assert_eq!(shared.channels[0].id, other.id);
        assert_eq!(
            select_save_scope(
                &saved,
                &draft,
                &SaveScope {
                    channel_ids: vec![first.clone(), first],
                    include_shared: false
                }
            )
            .unwrap_err()
            .code,
            "save_scope"
        );
        draft.recovery_id = Some(uuid::Uuid::new_v4().to_string());
        assert_eq!(
            select_save_scope(
                &saved,
                &draft,
                &SaveScope {
                    channel_ids: vec![],
                    include_shared: true
                }
            )
            .unwrap_err()
            .code,
            "save_scope"
        );
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn scoped_save_defers_unbound_channel_and_atomically_rebases_remaining_workspace() {
        let root = tempfile::tempdir().unwrap();
        let (snapshot, saved) = fixture();
        let context = cli_fixture(root.path(), &snapshot);
        let store = ModelStore::new(root.path());
        let initial = store.load().unwrap();
        store
            .commit_runtime_policy(
                &initial.library_digest,
                &initial.policy_digest,
                &initial.workspace_digest,
                saved.clone(),
            )
            .unwrap();
        let mut draft = saved.clone();
        draft.channels[0].naming.insert(
            "b".into(),
            ModelNaming {
                alias: None,
                display_name: Some("本次名称".into()),
            },
        );
        let mut blocked = saved.channels[0].clone();
        blocked.id = uuid::Uuid::new_v4().to_string();
        blocked.label = "待补连接".into();
        blocked.connection.provider_id = "deferred".into();
        blocked.connection.credential_ref = None;
        blocked.connection.enabled = false;
        draft.channels.push(blocked.clone());
        let baseline = store.load().unwrap();
        let staged = store
            .stage_policy(
                &baseline.policy_digest,
                &baseline.workspace_digest,
                draft.clone(),
            )
            .unwrap();
        let full = prepare(
            root.path(),
            &context,
            draft.clone(),
            staged.library_digest.clone(),
            staged.policy_digest.clone(),
            staged.workspace_digest.clone(),
        )
        .await
        .unwrap_err();
        assert_eq!(full.code, "credential");
        let plan = prepare_scoped(
            root.path(),
            &context,
            draft.clone(),
            staged.library_digest.clone(),
            staged.policy_digest.clone(),
            staged.workspace_digest.clone(),
            Some(SaveScope {
                channel_ids: vec![saved.channels[0].id.clone()],
                include_shared: false,
            }),
        )
        .await
        .unwrap();
        assert!(plan.deferred_channels.contains(&blocked.label));
        assert_eq!(plan.changed_channels, vec![saved.channels[0].label.clone()]);
        let applied = apply(root.path(), &context, &plan.id).await.unwrap();
        assert_eq!(applied.phase, "committed", "{:?}", applied.error);
        let now = store.load().unwrap();
        assert_eq!(now.policy.channels.len(), 1);
        assert_eq!(
            now.policy.channels[0].naming["b"].display_name.as_deref(),
            Some("本次名称")
        );
        let remaining = now.workspace.as_ref().unwrap();
        assert_eq!(remaining.base_policy, now.policy_digest);
        assert_eq!(remaining.policy, draft);
        let upstream = upstream::snapshot(&context).await.unwrap();
        assert!(upstream.providers["providers"].get("deferred").is_none());
        // §29.1：本轮不写友好名称，正式配置里该型号不再带 modelDisplayNames。
        assert!(upstream.providers["providers"]["test"]["modelDisplayNames"]
            .get("b")
            .is_none());
        let mut later = remaining.policy.clone();
        later.channels[1].label = "后来编辑".into();
        let later = store
            .stage_policy(&now.policy_digest, &now.workspace_digest, later)
            .unwrap();
        assert_eq!(
            apply(root.path(), &context, &plan.id).await.unwrap().phase,
            "committed"
        );
        assert_eq!(store.load().unwrap().workspace, later.workspace);
    }
    fn service_wire() -> FastWire {
        FastWire {
            kind: "service-tier".into(),
            canonical_to_wire: BTreeMap::from([("priority".into(), "priority".into())]),
            foreign_caller_tiers: "drop".into(),
            betas: vec![],
        }
    }
    #[test]
    fn naming_proposals_preserve_raw_identity_and_restore_previous_alias_on_inheritance() {
        let (mut snapshot, mut policy) = fixture();
        let library = super::super::store::seed_library();
        snapshot.providers["providers"]["test"]["modelAliases"] = json!({"b":"previous-short"});
        policy.channels[0].naming.insert(
            "b".into(),
            ModelNaming {
                alias: Some("new-short".into()),
                display_name: Some("新的名称".into()),
            },
        );
        let (next, order, slugs) = compile(&snapshot, &library, &policy).unwrap();
        assert_eq!(next["providers"]["test"]["modelAliases"]["b"], "new-short");
        // §29.1：路由别名照常写入，但显示名不再写到 modelDisplayNames。
        assert!(next["providers"]["test"]["modelDisplayNames"]
            .get("b")
            .is_none());
        assert_eq!(next["providers"]["test"]["selectedModels"], json!(["b"]));
        assert!(slugs.contains("test/b"));
        assert!(order.contains(&"test/b".into()));
        let projected = compiler::project(&library, &policy).unwrap();
        let row = projected.rows.iter().find(|r| r.model_id == "b").unwrap();
        assert_eq!(row.model_id, "b");
        // §29.1：投影统一 渠道名/原始模型ID，渠道名取编辑中的 label。
        assert_eq!(row.label, "测试渠道/b");
        assert_eq!(row.aliases, vec!["new-short"]);
        let ownership = next_ownership(&snapshot.providers, &next, &[], &policy).unwrap();
        snapshot.providers = next;
        policy.channels[0].naming.clear();
        let (restored, _, _) = compile_owned(&snapshot, &library, &policy, &ownership).unwrap();
        assert_eq!(
            restored["providers"]["test"]["modelAliases"]["b"],
            "previous-short"
        );
        policy.channels[0].naming.insert(
            "b".into(),
            ModelNaming {
                alias: Some("".into()),
                display_name: Some("".into()),
            },
        );
        let (removed, _, _) = compile_owned(&snapshot, &library, &policy, &ownership).unwrap();
        assert!(removed["providers"]["test"]["modelAliases"]
            .get("b")
            .is_none());
        assert!(removed["providers"]["test"]["modelDisplayNames"]
            .get("b")
            .is_none());
        snapshot.providers["providers"]["test"]["modelAliases"]["b"] = json!("operator-later");
        assert_eq!(
            compile_owned(&snapshot, &library, &policy, &ownership)
                .unwrap_err()
                .code,
            "field_drift"
        );
    }
    #[test]
    fn naming_rejects_reserved_duplicate_native_combo_and_original_model_id_collisions() {
        let (mut snapshot, mut policy) = fixture();
        let library = super::super::store::seed_library();
        for alias in ["GPT-9", "codex-any", "a", "a/b", "native"] {
            policy.channels[0].naming.insert(
                "b".into(),
                ModelNaming {
                    alias: Some(alias.into()),
                    display_name: None,
                },
            );
            assert!(compile(&snapshot, &library, &policy).is_err(), "{alias}");
        }
        policy.channels[0].naming.insert(
            "a".into(),
            ModelNaming {
                alias: Some("Short".into()),
                display_name: None,
            },
        );
        policy.channels[0].naming.insert(
            "b".into(),
            ModelNaming {
                alias: Some("short".into()),
                display_name: None,
            },
        );
        assert_eq!(
            compile(&snapshot, &library, &policy).unwrap_err().code,
            "model_alias_collision"
        );
        policy.channels[0].naming.remove("a");
        snapshot.models.as_array_mut().unwrap().push(
            json!({"provider":"combo","id":"virtual-short","namespaced":"combo/virtual-short"}),
        );
        policy.channels[0].naming.get_mut("b").unwrap().alias = Some("virtual-short".into());
        assert_eq!(
            compile(&snapshot, &library, &policy).unwrap_err().code,
            "model_alias_collision"
        );
    }
    #[test]
    fn shared_reasoning_and_fast_wire_validate_scope_and_restore_original_provider_fields() {
        let (mut snapshot, mut policy) = fixture();
        let library = super::super::store::seed_library();
        snapshot.providers["providers"]["test"]["modelReasoningEffortsAuthoritative"] =
            json!(false);
        policy.channels[0].overrides.insert(
            "b".into(),
            ModelRules {
                reasoning_authoritative: Field::Custom(true),
                reasoning_efforts: Field::Custom(vec!["low".into()]),
                ..Default::default()
            },
        );
        assert_eq!(
            compile(&snapshot, &library, &policy).unwrap_err().code,
            "shared_rule_scope"
        );
        policy.channels[0].overrides.clear();
        policy.channels[0].defaults = ModelRules {
            reasoning_efforts: Field::Custom(vec!["low".into(), "high".into()]),
            reasoning_authoritative: Field::Custom(true),
            fast_wire: Field::Custom(Some(service_wire())),
            ..Default::default()
        };
        let (next, _, _) = compile(&snapshot, &library, &policy).unwrap();
        assert_eq!(
            next["providers"]["test"]["modelReasoningEffortsAuthoritative"],
            true
        );
        assert_eq!(
            next["providers"]["test"]["fastWire"]["kind"],
            "service-tier"
        );
        let ownership = next_ownership(&snapshot.providers, &next, &[], &policy).unwrap();
        snapshot.providers = next;
        policy.channels[0].defaults = ModelRules::default();
        let (restored, _, _) = compile_owned(&snapshot, &library, &policy, &ownership).unwrap();
        assert_eq!(
            restored["providers"]["test"]["modelReasoningEffortsAuthoritative"],
            false
        );
        assert!(restored["providers"]["test"].get("fastWire").is_none());
        snapshot.providers["providers"]["test"]["fastWire"]["foreignCallerTiers"] =
            json!("verbatim");
        assert_eq!(
            compile_owned(&snapshot, &library, &policy, &ownership)
                .unwrap_err()
                .code,
            "field_drift"
        );
        policy.channels[0].defaults.fast_wire = Field::Custom(Some(FastWire {
            kind: "anthropic-speed".into(),
            ..service_wire()
        }));
        assert_eq!(
            compile(&snapshot, &library, &policy).unwrap_err().code,
            "fast_wire_protocol"
        );
    }
    #[test]
    fn no_reasoning_false_cannot_silently_override_an_upstream_family_classification() {
        let (mut snapshot, mut policy) = fixture();
        policy.channels[0].discovered[0].id = "family:tag".into();
        policy.channels[0].model_order = vec!["family:tag".into()];
        policy.channels[0].selection = ModelSelection::Whitelist(vec!["family:tag".into()]);
        snapshot.models[0]["id"] = json!("family:tag");
        snapshot.models[0]["namespaced"] = json!("test/family-tag");
        snapshot.catalog["models"] = json!([{"slug":"native"},{"slug":"control"},{"slug":"test/family-tag"},{"slug":"test/b"}]);
        snapshot.providers["providers"]["test"]["noReasoningModels"] = json!(["family"]);
        policy.channels[0].overrides.insert(
            "family:tag".into(),
            ModelRules {
                no_reasoning: Field::Custom(false),
                ..Default::default()
            },
        );
        assert_eq!(
            compile(&snapshot, &super::super::store::seed_library(), &policy)
                .unwrap_err()
                .code,
            "no_reasoning_scope"
        );
    }
    #[test]
    fn unsupported_request_constraints_fail_before_writes_instead_of_claiming_metadata_is_effective(
    ) {
        let library = super::super::store::seed_library();
        // Retain legacy upstream metadata during an unrelated product edit;
        // it is not an explicit promise of supported request constraints.
        let (mut original, mut unrelated) = fixture();
        original.providers["providers"]["test"]["noReasoningModels"] = json!(["b"]);
        unrelated.channels[0].connection.protocol = "openai-responses".into();
        original.providers["providers"]["test"]["adapter"] = json!("openai-responses");
        unrelated.channels[0].label = "unrelated display edit".into();
        assert_eq!(
            compile(&original, &library, &unrelated).unwrap().0["providers"]["test"]
                ["noReasoningModels"],
            json!(["b"])
        );
        for protocol in ["openai-responses", "anthropic", "google"] {
            for kind in [
                "noReasoning",
                "pinnedEffort",
                "effortMap",
                "reasoningAuthoritative",
            ] {
                let (snapshot, mut policy) = fixture();
                let mut rules = ModelRules {
                    protocol: Field::Custom(protocol.into()),
                    reasoning_efforts: Field::Custom(vec!["low".into(), "high".into()]),
                    ..Default::default()
                };
                match kind {
                    "noReasoning" => rules.no_reasoning = Field::Custom(true),
                    "pinnedEffort" => rules.pinned_effort = Field::Custom("high".into()),
                    "effortMap" => {
                        rules.effort_map =
                            Field::Custom(BTreeMap::from([("high".into(), "low".into())]))
                    }
                    _ => rules.reasoning_authoritative = Field::Custom(true),
                }
                policy.channels[0].overrides.insert("b".into(), rules);
                assert_eq!(
                    compile(&snapshot, &library, &policy).unwrap_err().code,
                    "reasoning_wire_unsupported",
                    "{protocol}/{kind}"
                );
            }
        }
    }
    #[test]
    fn no_reasoning_rule_preserves_other_models_and_tracks_exact_list_for_inheritance_and_drift() {
        let (mut snapshot, mut policy) = fixture();
        snapshot.providers["providers"]["test"]["noReasoningModels"] = json!(["other"]);
        policy.channels[0].overrides.insert(
            "b".into(),
            ModelRules {
                no_reasoning: Field::Custom(true),
                ..Default::default()
            },
        );
        let library = super::super::store::seed_library();
        let (next, _, _) = compile(&snapshot, &library, &policy).unwrap();
        assert_eq!(
            next["providers"]["test"]["noReasoningModels"],
            json!(["other", "b"])
        );
        let ownership = next_ownership(&snapshot.providers, &next, &[], &policy).unwrap();
        assert!(ownership
            .iter()
            .any(|v| v.key == "noReasoningModels" && v.model.is_empty()));
        snapshot.providers = next;
        policy.channels[0].overrides.clear();
        let (restored, _, _) = compile_owned(&snapshot, &library, &policy, &ownership).unwrap();
        assert_eq!(
            restored["providers"]["test"]["noReasoningModels"],
            json!(["other"])
        );
        policy.channels[0].overrides.insert(
            "b".into(),
            ModelRules {
                no_reasoning: Field::Custom(false),
                ..Default::default()
            },
        );
        let (allowed, _, _) = compile(&snapshot, &library, &policy).unwrap();
        assert_eq!(
            allowed["providers"]["test"]["noReasoningModels"],
            json!(["other"])
        );
        snapshot.providers["providers"]["test"]["noReasoningModels"]
            .as_array_mut()
            .unwrap()
            .push(json!("later"));
        assert_eq!(
            compile_owned(&snapshot, &library, &policy, &ownership)
                .unwrap_err()
                .code,
            "field_drift"
        );
    }
    #[test]
    fn explicit_reasoning_pin_and_label_map_compile_and_restore_original_without_resetting_default()
    {
        let (mut snapshot, mut policy) = fixture();
        snapshot.providers["providers"]["test"]["modelPinnedReasoningEfforts"] = json!({"b":"low"});
        snapshot.providers["providers"]["test"]["modelReasoningEffortMap"] =
            json!({"b":{"high":"legacy-high"},"other":{"high":"keep"}});
        let channel = &mut policy.channels[0];
        channel.overrides.insert(
            "b".into(),
            ModelRules {
                reasoning_efforts: Field::Custom(vec!["low".into(), "high".into()]),
                default_effort: Field::Custom("low".into()),
                pinned_effort: Field::Custom("high".into()),
                effort_map: Field::Custom(BTreeMap::from([("high".into(), "__omit__".into())])),
                ..Default::default()
            },
        );
        let library = super::super::store::seed_library();
        let (next, _, _) = compile(&snapshot, &library, &policy).unwrap();
        assert_eq!(
            next["providers"]["test"]["modelPinnedReasoningEfforts"]["b"],
            "high"
        );
        assert_eq!(
            next["providers"]["test"]["modelDefaultReasoningEfforts"]["b"],
            "low"
        );
        assert_eq!(
            next["providers"]["test"]["modelReasoningEffortMap"]["b"]["high"],
            "__omit__"
        );
        assert_eq!(
            next["providers"]["test"]["modelReasoningEffortMap"]["other"]["high"],
            "keep"
        );
        let ownership = next_ownership(&snapshot.providers, &next, &[], &policy).unwrap();
        snapshot.providers = next;
        policy.channels[0].overrides.clear();
        let (restored, _, _) = compile_owned(&snapshot, &library, &policy, &ownership).unwrap();
        assert_eq!(
            restored["providers"]["test"]["modelPinnedReasoningEfforts"]["b"],
            "low"
        );
        assert_eq!(
            restored["providers"]["test"]["modelReasoningEffortMap"]["b"]["high"],
            "legacy-high"
        );
        snapshot.providers["providers"]["test"]["modelReasoningEffortMap"]["b"]["high"] =
            json!("later-edit");
        assert_eq!(
            compile_owned(&snapshot, &library, &policy, &ownership)
                .unwrap_err()
                .code,
            "field_drift"
        );
    }
    #[test]
    fn display_names_use_official_display_field_preserve_routing_aliases_and_track_explicit_removal(
    ) {
        let (mut snapshot, mut policy) = fixture();
        snapshot.providers["providers"]["test"]["modelAliases"] = json!({"b":"route-b"});
        snapshot.providers["providers"]["test"]["modelDisplayNames"] = json!({"b":"原显示名"});
        let original = snapshot.providers.clone();
        policy.channels[0]
            .discovered
            .iter_mut()
            .find(|m| m.id == "b")
            .unwrap()
            .label = "".into();
        let (next, _, _) =
            compile(&snapshot, &super::super::store::seed_library(), &policy).unwrap();
        assert_eq!(
            next["providers"]["test"]["modelAliases"],
            original["providers"]["test"]["modelAliases"]
        );
        assert!(next["providers"]["test"]["modelDisplayNames"]
            .get("b")
            .is_none());
        let ownership = next_ownership(&snapshot.providers, &next, &[], &policy).unwrap();
        assert!(ownership
            .iter()
            .any(|f| f.key == "modelDisplayNames" && f.model == "b" && f.removed));
        snapshot.providers = next.clone();
        compile_owned(
            &snapshot,
            &super::super::store::seed_library(),
            &policy,
            &ownership,
        )
        .unwrap();
        snapshot.providers["providers"]["test"]["modelDisplayNames"]["b"] = json!("外部后来修改");
        assert_eq!(
            compile_owned(
                &snapshot,
                &super::super::store::seed_library(),
                &policy,
                &ownership
            )
            .unwrap_err()
            .code,
            "field_drift"
        );
        policy.channels[0].discovered[0].label = "invalid/label".into();
        assert_eq!(
            compile(&snapshot, &super::super::store::seed_library(), &policy)
                .unwrap_err()
                .code,
            "model_label"
        );
    }
    #[test]
    fn oauth_or_forward_binding_cannot_be_retargeted_to_a_new_service() {
        let (mut snapshot, mut policy) = fixture();
        snapshot.providers["providers"]["test"]["authMode"] = json!("oauth");
        policy.channels[0].connection.nodes[0].url = "https://different.example.test/v1".into();
        assert_eq!(
            compile(&snapshot, &super::super::store::seed_library(), &policy)
                .unwrap_err()
                .code,
            "provider_auth_boundary"
        );
        assert_eq!(
            snapshot.providers["providers"]["test"]["baseUrl"],
            "https://example.test/v1"
        );
    }
    use super::*;
    #[test]
    fn historical_direction_stage_pairs_target_service_with_exact_models_and_current_alias_ownership(
    ) {
        let mut current = json!({"defaultProvider":"p","providers":{"p":{"adapter":"openai-chat","baseUrl":"https://new.example.test","alias":"current-alias","modelAliases":{"a":"current-a"},"defaultAliases":false}}});
        let target = json!({"adapter":"openai-responses","authMode":"key","baseUrl":"https://old.example.test","alias":"historical-alias","modelAliases":{"a":"historical-a"},"models":["a","b"],"newModelPolicy":"on","disabled":false});
        let staged = direction_public("p", &current, &target, &json!([])).unwrap();
        assert_eq!(staged["baseUrl"], "https://old.example.test");
        assert_eq!(staged["adapter"], "openai-responses");
        assert_eq!(staged["selectedModels"], json!(["a", "b"]));
        assert_eq!(staged["newModelPolicy"], "off");
        assert_eq!(staged["alias"], "current-alias");
        assert_eq!(
            staged["modelAliases"],
            current["providers"]["p"]["modelAliases"]
        );
        assert_eq!(staged["disabled"], true);
        assert_eq!(target["alias"], "historical-alias");
        current["defaultProvider"] = json!("keep");
        assert_eq!(
            direction_public("p", &current, &target, &json!([])).unwrap()["disabled"],
            true
        );
        assert!(direction_public(
            "p",
            &current,
            &json!({"adapter":"openai-chat","baseUrl":"https://old.example.test"}),
            &json!([])
        )
        .is_err());
    }
    #[test]
    fn credential_compensation_accepts_its_known_intermediate_but_refuses_new_keys_or_selection() {
        let before = super::super::management::KeyState {
            active_id: Some("old".into()),
            ids: BTreeSet::from(["old".into()]),
        };
        let mut change = CredentialChange {
            provider: "fixture".into(),
            reference: uuid::Uuid::new_v4().to_string(),
            before: before.clone(),
            intended_id: "new".into(),
            intended_digest: "fixture-digest".into(),
            observed: None,
        };
        let intended = change.intended();
        assert!(!change.compensation_owned(&intended));
        change.observed = Some(intended.clone());
        assert!(change.compensation_owned(&intended));
        assert!(change.compensation_owned(&before));
        let mut intermediate = intended.clone();
        intermediate.active_id = before.active_id.clone();
        assert!(change.compensation_owned(&intermediate));
        let mut later = intended;
        later.ids.insert("operator-key".into());
        assert!(!change.compensation_owned(&later));
        later.active_id = Some("operator-key".into());
        assert!(!change.compensation_owned(&later));
    }
    use std::collections::BTreeMap;
    fn fixture() -> (UpstreamSnapshot, Policy) {
        let node = ServiceNode {
            id: uuid::Uuid::new_v4().to_string(),
            name: "主节点".into(),
            url: "https://example.test/v1".into(),
        };
        let channel = ChannelPolicy {
            naming: Default::default(),
            id: uuid::Uuid::new_v4().to_string(),
            label: "测试渠道".into(),
            template_refs: vec![],
            model_refs: BTreeMap::new(),
            connection: ChannelConnection {
                auth_mode: None,
                source_read_id: None,
                imported_config: None,
                provider_id: "test".into(),
                protocol: "openai-chat".into(),
                nodes: vec![node.clone()],
                active_node_id: node.id,
                credential_ref: None,
                enabled: true,
                allow_private_network: false,
            },
            selection: ModelSelection::Whitelist(vec!["b".into()]),
            model_order: vec!["b".into(), "a".into()],
            discovered: vec![
                DiscoveredModel {
                    id: "a".into(),
                    label: "A".into(),
                    rules: Default::default(),
                },
                DiscoveredModel {
                    id: "b".into(),
                    label: "B".into(),
                    rules: Default::default(),
                },
            ],
            defaults: Default::default(),
            overrides: Default::default(),
            family_overrides: Default::default(),
        };
        (
            UpstreamSnapshot {
                version: "2.84.0".into(),
                providers: json!({"defaultProvider":"test","providers":{"test":{"adapter":"openai-chat","baseUrl":"https://example.test/v1","models":["a","b"]},"unmanaged":{"adapter":"openai-chat","baseUrl":"https://other.test"}}}),
                models: json!([{"provider":"test","id":"a","namespaced":"test/a","disabled":false},{"provider":"test","id":"b","namespaced":"test/b","disabled":false}]),
                order: json!({"pickerOrder":["native","test/a","test/b"],"pickerOrderMode":null}),
                catalog: json!({"models":[{"slug":"native"},{"slug":"control","visibility":"hide"},{"slug":"test/a"},{"slug":"test/b"}]}),
            },
            Policy {
                channels: vec![channel],
                ..Default::default()
            },
        )
    }
    #[test]
    fn exact_catalog_projection_retains_all_native_hidden_and_unmanaged_occupancy() {
        let (snapshot, policy) = fixture();
        let (next, order, slugs) =
            compile(&snapshot, &super::super::store::seed_library(), &policy).unwrap();
        assert_eq!(
            slugs,
            BTreeSet::from(["native".into(), "control".into(), "test/b".into()])
        );
        assert_eq!(order, vec!["native", "test/b"]);
        assert_eq!(
            next["providers"]["unmanaged"],
            snapshot.providers["providers"]["unmanaged"]
        );
        assert_eq!(next["providers"]["test"]["selectedModels"], json!(["b"]));
        assert!(next["providers"]["test"]
            .get("modelContextWindows")
            .is_none());
    }
    #[test]
    fn projection_rejects_missing_real_model_mapping_catalog_mismatch_and_budget_overflow() {
        let (mut snapshot, mut policy) = fixture();
        policy.channels[0].selection = ModelSelection::Whitelist(vec!["missing".into()]);
        assert_eq!(
            compile(&snapshot, &super::super::store::seed_library(), &policy)
                .unwrap_err()
                .code,
            "model_unverified"
        );
        policy.channels[0].selection = ModelSelection::All;
        snapshot.catalog["models"] = json!([{"slug":"native"}]);
        assert_eq!(
            compile(&snapshot, &super::super::store::seed_library(), &policy)
                .unwrap_err()
                .code,
            "catalog_projection"
        );
        let (mut snapshot, policy) = fixture();
        for index in 0..100 {
            snapshot.catalog["models"]
                .as_array_mut()
                .unwrap()
                .push(json!({"slug":format!("native-{index}")}));
        }
        assert_eq!(
            compile(&snapshot, &super::super::store::seed_library(), &policy)
                .unwrap_err()
                .code,
            "catalog_budget"
        );
    }
    #[test]
    fn full_catalog_compatibility_budget_is_exact_at_99_100_101_and_108_without_truncation() {
        for total in [99_usize, 100, 101, 108] {
            let (mut snapshot, policy) = fixture();
            // Two non-managed rows (one hidden) plus the one selected managed model.
            for index in 0..total - 3 {
                snapshot.catalog["models"].as_array_mut().unwrap().push(
                    json!({"slug":format!("native-boundary-{index}"),"visibility":if index%2==0{"hide"}else{"list"}}),
                );
            }
            let before = serde_json::to_value((&snapshot, &policy)).unwrap();
            let result = compile(&snapshot, &super::super::store::seed_library(), &policy);
            if total <= 100 {
                let (target, order, ids) = result.unwrap();
                assert_eq!(ids.len(), total);
                assert!(ids.contains("control"), "hidden occupancy must count");
                assert_eq!(order, vec!["native", "test/b"]);
                assert_eq!(target["providers"]["test"]["selectedModels"], json!(["b"]));
            } else {
                let error = result.unwrap_err();
                assert_eq!(error.code, "catalog_budget");
                assert!(error.message.contains(&total.to_string()));
            }
            assert_eq!(serde_json::to_value((&snapshot, &policy)).unwrap(), before);
        }
    }
    #[test]
    fn managed_inheritance_restores_original_values_but_external_edits_fail_closed() {
        let (mut snapshot, policy) = fixture();
        snapshot.providers["providers"]["test"]["modelContextWindows"] = json!({"b":8000});
        let ownership = vec![OwnedField {
            provider: "test".into(),
            key: "modelContextWindows".into(),
            model: "b".into(),
            original: Some(json!(4000)),
            written: json!(8000),
            removed: false,
        }];
        let (next, _, _) = compile_owned(
            &snapshot,
            &super::super::store::seed_library(),
            &policy,
            &ownership,
        )
        .unwrap();
        assert_eq!(next["providers"]["test"]["modelContextWindows"]["b"], 4000);
        snapshot.providers["providers"]["test"]["modelContextWindows"]["b"] = json!(6000);
        assert_eq!(
            compile_owned(
                &snapshot,
                &super::super::store::seed_library(),
                &policy,
                &ownership
            )
            .unwrap_err()
            .code,
            "field_drift"
        );
    }
    #[cfg(unix)]
    fn cli_fixture(root: &Path, snapshot: &UpstreamSnapshot) -> ProcessContext {
        use base64::Engine;
        use hmac::Mac;
        use std::os::unix::fs::PermissionsExt;
        for (name, value) in [
            ("providers", &snapshot.providers),
            ("models", &snapshot.models),
            ("order", &snapshot.order),
            ("catalog", &snapshot.catalog),
        ] {
            std::fs::write(
                root.join(format!("{name}.json")),
                serde_json::to_vec(value).unwrap(),
            )
            .unwrap();
        }
        std::fs::write(root.join("disabled.json"), b"[]").unwrap();
        let executable = root.join("ocx");
        std::fs::write(&executable,br#"#!/usr/bin/python3
import json,os,sys
from pathlib import Path
root=Path.cwd();args=sys.argv[1:]
def read(name):return json.loads((root/(name+'.json')).read_text())
def save(name,value):
 (root/(name+'.json')).write_text(json.dumps(value))
 if name=='providers':(Path(os.environ['OPENCODEX_HOME'])/'config.json').write_text(json.dumps(value))
if args==['--version']:print('2.84.0')
elif args[:2]==['provider','snapshot']:print(json.dumps(read('providers')))
elif args[:2]==['models','live']:print(json.dumps(read('models')))
elif args[:3]==['models','order','status']:print(json.dumps(read('order')))
elif args[:2]==['inspect','catalog']:print(json.dumps(read('catalog')))
elif args[:2]==['config','show']:print(json.dumps({'disabledModels':read('disabled')}))
elif args[:3]==['config','set','disabledModels']:save('disabled',json.loads(args[3]));print('{}')
elif args[:2]==['provider','apply']:
 baseline=json.loads(Path(args[args.index('--baseline')+1]).read_text())
 if baseline!=read('providers'):sys.exit(5)
 next_value=json.load(sys.stdin);old=read('providers');models=read('models')
 for name,provider in next_value['providers'].items():
  if name not in old['providers']:
   if 'compensation-baseline' not in args[args.index('--baseline')+1]:assert provider.get('disabled') is True and provider.get('selectedModels'), 'new provider exposed all models'
   (root/'safe-registration').write_text('disabled with nonempty allowlist')
   for model in provider.get('models',[]):models.append({'provider':name,'id':model,'namespaced':name+'/'+model.replace('/','-'),'disabled':False})
 models=[row for row in models if row['provider'] in next_value['providers']]
 save('models',models);save('providers',next_value);print('{}')
elif args[:3]==['config','set','modelPickerOrder']:
 if (root/'fail-order').exists():sys.exit(6)
 order=read('order');order['pickerOrder']=json.loads(args[3]);save('order',order);print('{}')
elif args[:2] in [['models','enable'],['models','disable']]:
 rows=read('models')
 for row in rows:
  if row['namespaced']==args[2]:row['disabled']=args[1]=='disable'
 save('models',rows);print('{}')
elif args==['sync']:
 prior=read('catalog');providers=read('providers')['providers'];rows=read('models');keep=[r for r in prior['models'] if '/' not in r['slug']]
 for row in rows:
  provider=providers[row['provider']];selected=provider.get('selectedModels',[])
  if not provider.get('disabled',False) and not row.get('disabled',False) and (not selected or row['id'] in selected):keep.append({'slug':row['namespaced']})
 save('catalog',{'models':keep});print('{}')
else:sys.exit(7)
"#).unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
        let home = root.join("home");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(
            home.join("config.json"),
            serde_json::to_vec(&snapshot.providers).unwrap(),
        )
        .unwrap();
        let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let port = listener.local_addr().unwrap().port();
        let pid = std::process::id();
        let secret = "s".repeat(43);
        atomic_write(
            &home.join("runtime-port.json"),
            &serde_json::to_vec(&json!({"pid":pid,"port":port,"attestationSecret":secret}))
                .unwrap(),
            0o600,
        )
        .unwrap();
        atomic_write(
            &home.join("admin-api-token"),
            format!("ocx_admin_{}", "t".repeat(43)).as_bytes(),
            0o600,
        )
        .unwrap();
        let order_root = root.to_path_buf();
        let disable_root = root.to_path_buf();
        let app = axum::Router::new()
            .route(
                "/api/providers/keys",
                axum::routing::get(|| async { axum::Json(json!({"activeId":null,"keys":[]})) }),
            )
            .route(
                "/healthz",
                axum::routing::get(move |headers: axum::http::HeaderMap| {
                    let secret = secret.clone();
                    async move {
                        let challenge = headers
                            .get("x-opencodex-attestation-challenge")
                            .unwrap()
                            .to_str()
                            .unwrap();
                        let mut mac =
                            hmac::Hmac::<sha2::Sha256>::new_from_slice(secret.as_bytes()).unwrap();
                        mac.update(
                            format!("opencodex-local-management-v1\n{challenge}\n{pid}\n{port}")
                                .as_bytes(),
                        );
                        let mut response = axum::http::HeaderMap::new();
                        response.insert(
                            "x-opencodex-attestation-proof",
                            base64::engine::general_purpose::URL_SAFE_NO_PAD
                                .encode(mac.finalize().into_bytes())
                                .parse()
                                .unwrap(),
                        );
                        (
                            response,
                            axum::Json(json!({"service":"opencodex","pid":pid})),
                        )
                    }
                }),
            )
            .route(
                "/api/subagent-models",
                axum::routing::put(
                    move |headers: axum::http::HeaderMap, axum::Json(body): axum::Json<Value>| {
                        let root = order_root.clone();
                        async move {
                            assert_eq!(
                                headers.get("authorization").unwrap(),
                                format!("Bearer ocx_admin_{}", "t".repeat(43)).as_str()
                            );
                            if root.join("fail-order").exists() {
                                return (
                                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                    axum::Json(json!({"error":"fixture-order"})),
                                );
                            }
                            let mut order: Value = serde_json::from_slice(
                                &std::fs::read(root.join("order.json")).unwrap(),
                            )
                            .unwrap();
                            order["pickerOrder"] = body["pickerOrder"].clone();
                            std::fs::write(
                                root.join("order.json"),
                                serde_json::to_vec(&order).unwrap(),
                            )
                            .unwrap();
                            (axum::http::StatusCode::OK, axum::Json(order))
                        }
                    },
                ),
            )
            .route(
                "/api/disabled-models",
                axum::routing::put(
                    move |headers: axum::http::HeaderMap, axum::Json(body): axum::Json<Value>| {
                        let root = disable_root.clone();
                        async move {
                            assert_eq!(
                                headers.get("authorization").unwrap(),
                                format!("Bearer ocx_admin_{}", "t".repeat(43)).as_str()
                            );
                            std::fs::write(
                                root.join("disabled.json"),
                                serde_json::to_vec(&body["models"]).unwrap(),
                            )
                            .unwrap();
                            axum::Json(json!({"ok":true}))
                        }
                    },
                ),
            );
        let listener = tokio::net::TcpListener::from_std(listener).unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(
                executable,
            ),
            working_directory: root.into(),
            opencodex_home: root.join("home"),
        }
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn removing_committed_custom_channel_switches_default_and_reads_back_actual_absence() {
        let root = tempfile::tempdir().unwrap();
        let (mut snapshot, mut original) = fixture();
        let mut keep = original.channels[0].clone();
        keep.id = uuid::Uuid::new_v4().to_string();
        keep.label = "保留渠道".into();
        keep.connection.provider_id = "unmanaged".into();
        keep.connection.nodes[0].url = "https://other.test".into();
        keep.selection = ModelSelection::Whitelist(vec!["a".into()]);
        keep.discovered.retain(|m| m.id == "a");
        keep.model_order = vec!["a".into()];
        snapshot.providers["providers"]["unmanaged"]["models"] = json!(["a"]);
        snapshot.models.as_array_mut().unwrap().push(
            json!({"provider":"unmanaged","id":"a","namespaced":"unmanaged/a","disabled":false}),
        );
        snapshot.catalog["models"]
            .as_array_mut()
            .unwrap()
            .push(json!({"slug":"unmanaged/a"}));
        snapshot.order["pickerOrder"]
            .as_array_mut()
            .unwrap()
            .push(json!("unmanaged/a"));
        original.channels.push(keep.clone());
        let context = cli_fixture(root.path(), &snapshot);
        let store = ModelStore::new(root.path());
        let initial = store.load().unwrap();
        store
            .commit_runtime_policy(
                &initial.library_digest,
                &initial.policy_digest,
                &initial.workspace_digest,
                original,
            )
            .unwrap();
        let current = store.load().unwrap();
        let intended = Policy {
            channels: vec![keep.clone()],
            default_channel_id: Some(keep.id.clone()),
            ..Default::default()
        };
        let staged = store
            .stage_policy(
                &current.policy_digest,
                &current.workspace_digest,
                intended.clone(),
            )
            .unwrap();
        let plan = prepare(
            root.path(),
            &context,
            intended.clone(),
            staged.library_digest,
            staged.policy_digest,
            staged.workspace_digest,
        )
        .await
        .unwrap();
        assert_eq!(plan.removed_providers, vec!["test"]);
        assert!(snapshot.providers["providers"].get("test").is_some());
        std::fs::write(root.path().join("fail-order"), b"fixture").unwrap();
        let partial = apply(root.path(), &context, &plan.id).await.unwrap();
        assert_eq!(partial.phase, "visibility_written");
        assert!(
            upstream::snapshot(&context).await.unwrap().providers["providers"]
                .get("test")
                .is_none()
        );
        std::fs::remove_file(root.path().join("fail-order")).unwrap();
        assert_eq!(
            compensate(root.path(), &context, &plan.id)
                .await
                .unwrap()
                .phase,
            "compensated"
        );
        assert_eq!(
            upstream::snapshot(&context).await.unwrap().providers,
            snapshot.providers
        );
        let local = store.load().unwrap();
        let plan = prepare(
            root.path(),
            &context,
            intended.clone(),
            local.library_digest,
            local.policy_digest,
            local.workspace_digest,
        )
        .await
        .unwrap();
        let result = apply(root.path(), &context, &plan.id).await.unwrap();
        assert_eq!(result.phase, "committed", "{:?}", result.error);
        let actual = upstream::snapshot(&context).await.unwrap();
        assert!(actual.providers["providers"].get("test").is_none());
        assert_eq!(actual.providers["defaultProvider"], "unmanaged");
        assert_eq!(
            actual.order["pickerOrder"],
            json!(["native", "unmanaged/a"])
        );
        assert_eq!(store.load().unwrap().policy, intended);
        let local = store.load().unwrap();
        let recovery = prepare_recovery(
            root.path(),
            super::super::recovery::Request {
                source_id: plan.id.clone(),
                side: super::super::recovery::Side::Before,
                scope: super::super::recovery::Scope::Affected,
                restore_defaults: false,
                expected_library: local.library_digest,
                expected_policy: local.policy_digest,
                expected_workspace: local.workspace_digest,
            },
        )
        .unwrap();
        let draft = accept_recovery(root.path(), &recovery.id).unwrap();
        assert_eq!(
            draft
                .workspace
                .as_ref()
                .unwrap()
                .policy
                .recovery_id
                .as_deref(),
            Some(recovery.id.as_str())
        );
        let history_plan = prepare(
            root.path(),
            &context,
            draft.workspace.unwrap().policy,
            draft.library_digest,
            draft.policy_digest,
            draft.workspace_digest,
        )
        .await
        .unwrap();
        assert_eq!(
            history_plan.history_source.as_deref(),
            Some(plan.id.as_str())
        );
        let history_saved = apply(root.path(), &context, &history_plan.id)
            .await
            .unwrap();
        assert_eq!(
            history_saved.phase, "committed",
            "{:?}",
            history_saved.error
        );
        let actual = upstream::snapshot(&context).await.unwrap();
        assert_eq!(actual.providers, snapshot.providers);
        assert_eq!(
            catalog_slugs(&actual).unwrap(),
            catalog_slugs(&snapshot).unwrap()
        );
        assert!(store.load().unwrap().policy.recovery_id.is_none());

        // First-read rows need not be saved once merely to authorize their deletion.
        let initial_root = tempfile::tempdir().unwrap();
        let initial_context = cli_fixture(initial_root.path(), &snapshot);
        let read = super::super::channel_actions::remember(
            initial_root.path(),
            &initial_context,
            snapshot.clone(),
        )
        .unwrap();
        let initial_store = ModelStore::new(initial_root.path());
        let local = initial_store.load().unwrap();
        let mut pending = intended.clone();
        pending.pending_removals.push(SourceRemoval {
            read_id: read.read_id,
            channel_id: uuid::Uuid::new_v4().to_string(),
            provider_id: "test".into(),
            credential_ref: None,
        });
        let staged = initial_store
            .stage_policy(
                &local.policy_digest,
                &local.workspace_digest,
                pending.clone(),
            )
            .unwrap();
        let plan = prepare(
            initial_root.path(),
            &initial_context,
            pending,
            staged.library_digest,
            staged.policy_digest,
            staged.workspace_digest,
        )
        .await
        .unwrap();
        assert_eq!(plan.removed_providers, vec!["test"]);
        assert_eq!(
            apply(initial_root.path(), &initial_context, &plan.id)
                .await
                .unwrap()
                .phase,
            "committed"
        );
        let committed = initial_store.load().unwrap();
        assert!(committed.policy.pending_removals.is_empty());
        assert!(upstream::snapshot(&initial_context)
            .await
            .unwrap()
            .providers["providers"]
            .get("test")
            .is_none());
        let historical = load(initial_root.path(), &plan.id).unwrap();
        assert!(historical
            .recovery
            .unwrap()
            .policy
            .channels
            .iter()
            .any(|c| c.connection.provider_id == "test"));
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn official_save_plan_freezes_baselines_backs_up_reads_back_and_commits_once() {
        let root = tempfile::tempdir().unwrap();
        let (snapshot, policy) = fixture();
        let context = cli_fixture(root.path(), &snapshot);
        let store = ModelStore::new(root.path());
        let local = store.load().unwrap();
        let local = store
            .stage_policy(
                &local.policy_digest,
                &local.workspace_digest,
                policy.clone(),
            )
            .unwrap();
        let plan = prepare(
            root.path(),
            &context,
            policy.clone(),
            local.library_digest.clone(),
            local.policy_digest.clone(),
            local.workspace_digest.clone(),
        )
        .await
        .unwrap();
        assert_eq!(plan.catalog_count, 3);
        assert!(!root.path().join(POLICY_PATH).exists());
        let committed = apply(root.path(), &context, &plan.id).await.unwrap();
        assert_eq!(committed.phase, "committed");
        assert!(load(root.path(), &plan.id)
            .unwrap()
            .committed_snapshot
            .is_some());
        assert_eq!(store.load().unwrap().policy, policy);
        assert!(root.path().join("backups").is_dir());
        assert_eq!(
            apply(root.path(), &context, &plan.id).await.unwrap().phase,
            "committed"
        );
        let entries = history(root.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].recoverable);
        use super::super::client_state::{self, Request as ClientRequest, Status as ClientStatus};
        assert!(matches!(
            client_state::status(root.path()).unwrap().status,
            ClientStatus::RestartRequired
        ));
        assert_eq!(
            client_state::check(
                root.path(),
                &context,
                ClientRequest {
                    plan_id: plan.id.clone(),
                    confirmation_check: Some(uuid::Uuid::new_v4().to_string())
                }
            )
            .await
            .unwrap_err()
            .code,
            "client_confirmation"
        );
        let checked = client_state::check(
            root.path(),
            &context,
            ClientRequest {
                plan_id: plan.id.clone(),
                confirmation_check: None,
            },
        )
        .await
        .unwrap();
        assert!(matches!(checked.status, ClientStatus::PendingVerification));
        assert!(!checked.automatic_client_verification);
        let confirmed = client_state::check(
            root.path(),
            &context,
            ClientRequest {
                plan_id: plan.id.clone(),
                confirmation_check: checked.check_id,
            },
        )
        .await
        .unwrap();
        assert!(matches!(confirmed.status, ClientStatus::UserConfirmed));
        let current = store.load().unwrap();
        let label_only = policy.clone();
        let staged = store
            .stage_policy(
                &current.policy_digest,
                &current.workspace_digest,
                label_only.clone(),
            )
            .unwrap();
        let metadata_plan = prepare(
            root.path(),
            &context,
            label_only,
            staged.library_digest.clone(),
            staged.policy_digest.clone(),
            staged.workspace_digest.clone(),
        )
        .await
        .unwrap();
        assert_eq!(
            apply(root.path(), &context, &metadata_plan.id)
                .await
                .unwrap()
                .phase,
            "committed"
        );
        let retained = client_state::status(root.path()).unwrap();
        assert_eq!(retained.plan_id.as_deref(), Some(plan.id.as_str()));
        assert!(matches!(retained.status, ClientStatus::UserConfirmed));
        let mut drift = serde_json::from_slice::<Value>(
            &std::fs::read(root.path().join("providers.json")).unwrap(),
        )
        .unwrap();
        drift["providers"]["test"]["note"] = json!("later external edit");
        std::fs::write(
            root.path().join("providers.json"),
            serde_json::to_vec(&drift).unwrap(),
        )
        .unwrap();
        let changed = client_state::check(
            root.path(),
            &context,
            ClientRequest {
                plan_id: plan.id.clone(),
                confirmation_check: None,
            },
        )
        .await
        .unwrap();
        assert!(matches!(changed.status, ClientStatus::RuntimeChanged));
        assert!(changed.confirmed_at.is_none());
        let current = store.load().unwrap();
        let preview = prepare_recovery(
            root.path(),
            super::super::recovery::Request {
                source_id: plan.id.clone(),
                side: super::super::recovery::Side::After,
                scope: super::super::recovery::Scope::Affected,
                restore_defaults: false,
                expected_library: current.library_digest,
                expected_policy: current.policy_digest,
                expected_workspace: current.workspace_digest,
            },
        )
        .unwrap();
        let restored = accept_recovery(root.path(), &preview.id).unwrap();
        assert_eq!(restored.policy, policy);
        let mut recovered_policy = policy.clone();
        recovered_policy.default_channel_id = Some(policy.channels[0].id.clone());
        assert_eq!(
            restored.workspace.unwrap().policy.published(),
            recovered_policy
        );
        let mut source = load(root.path(), &plan.id).unwrap();
        source.phase = "compensated".into();
        save(root.path(), &source).unwrap();
        assert!(
            !history(root.path())
                .unwrap()
                .iter()
                .find(|entry| entry.id == plan.id)
                .unwrap()
                .recoverable
        );
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn stale_plan_refuses_before_writes_and_partial_official_failure_is_not_success() {
        let root = tempfile::tempdir().unwrap();
        let (snapshot, policy) = fixture();
        let context = cli_fixture(root.path(), &snapshot);
        let store = ModelStore::new(root.path());
        let local = store.load().unwrap();
        let local = store
            .stage_policy(
                &local.policy_digest,
                &local.workspace_digest,
                policy.clone(),
            )
            .unwrap();
        let plan = prepare(
            root.path(),
            &context,
            policy.clone(),
            local.library_digest.clone(),
            local.policy_digest.clone(),
            local.workspace_digest.clone(),
        )
        .await
        .unwrap();
        let mut changed = snapshot.providers.clone();
        changed["providers"]["test"]["note"] = json!("external");
        std::fs::write(
            root.path().join("providers.json"),
            serde_json::to_vec(&changed).unwrap(),
        )
        .unwrap();
        assert_eq!(
            apply(root.path(), &context, &plan.id)
                .await
                .unwrap_err()
                .code,
            "stale_upstream"
        );
        assert_eq!(query(root.path(), &plan.id).unwrap().phase, "prepared");
        std::fs::write(
            root.path().join("providers.json"),
            serde_json::to_vec(&snapshot.providers).unwrap(),
        )
        .unwrap();
        std::fs::write(root.path().join("fail-order"), b"1").unwrap();
        let partial = apply(root.path(), &context, &plan.id).await.unwrap();
        assert_eq!(partial.phase, "visibility_written");
        assert_eq!(partial.error.as_deref(), Some("management_rejected"));
        assert_eq!(latest(root.path()).unwrap().unwrap().id, plan.id);
        let pending = ModelStore::new(root.path()).load().unwrap();
        assert_eq!(
            prepare(
                root.path(),
                &context,
                pending.workspace.as_ref().unwrap().policy.clone(),
                pending.library_digest,
                pending.policy_digest,
                pending.workspace_digest
            )
            .await
            .unwrap_err()
            .code,
            "plan_incomplete"
        );
        assert!(!root.path().join(POLICY_PATH).exists());
        assert_eq!(
            apply(root.path(), &context, &plan.id)
                .await
                .unwrap_err()
                .code,
            "plan_incomplete"
        );
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn partial_plan_can_only_compensate_its_own_verified_effects() {
        let root = tempfile::tempdir().unwrap();
        let (snapshot, policy) = fixture();
        let context = cli_fixture(root.path(), &snapshot);
        let store = ModelStore::new(root.path());
        let local = store.load().unwrap();
        let local = store
            .stage_policy(
                &local.policy_digest,
                &local.workspace_digest,
                policy.clone(),
            )
            .unwrap();
        let plan = prepare(
            root.path(),
            &context,
            policy,
            local.library_digest,
            local.policy_digest,
            local.workspace_digest,
        )
        .await
        .unwrap();
        std::fs::write(root.path().join("fail-order"), b"1").unwrap();
        let partial = apply(root.path(), &context, &plan.id).await.unwrap();
        assert_eq!(partial.phase, "visibility_written");
        let restored = compensate(root.path(), &context, &plan.id).await.unwrap();
        assert_eq!(restored.phase, "compensated");
        assert!(latest(root.path()).unwrap().is_none());
        assert!(!root.path().join(POLICY_PATH).exists());
        assert_eq!(
            upstream::read(&context, upstream::ReadOperation::ProviderSnapshot)
                .await
                .unwrap(),
            snapshot.providers
        );
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn compensation_never_overwrites_later_operator_changes() {
        let root = tempfile::tempdir().unwrap();
        let (snapshot, policy) = fixture();
        let context = cli_fixture(root.path(), &snapshot);
        let store = ModelStore::new(root.path());
        let local = store.load().unwrap();
        let local = store
            .stage_policy(
                &local.policy_digest,
                &local.workspace_digest,
                policy.clone(),
            )
            .unwrap();
        let plan = prepare(
            root.path(),
            &context,
            policy,
            local.library_digest,
            local.policy_digest,
            local.workspace_digest,
        )
        .await
        .unwrap();
        std::fs::write(root.path().join("fail-order"), b"1").unwrap();
        apply(root.path(), &context, &plan.id).await.unwrap();
        let mut changed = upstream::read(&context, upstream::ReadOperation::ProviderSnapshot)
            .await
            .unwrap();
        changed["providers"]["unmanaged"]["note"] = json!("later operator edit");
        let bytes = serde_json::to_vec(&changed).unwrap();
        std::fs::write(root.path().join("providers.json"), &bytes).unwrap();
        assert_eq!(
            compensate(root.path(), &context, &plan.id)
                .await
                .unwrap_err()
                .code,
            "compensation_drift"
        );
        assert_eq!(
            std::fs::read(root.path().join("providers.json")).unwrap(),
            bytes
        );
        assert_eq!(
            reconcile(root.path(), &context, &plan.id)
                .await
                .unwrap_err()
                .code,
            "reconcile_drift"
        );
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn new_channel_is_first_registered_disabled_with_an_explicit_nonempty_allowlist() {
        let root = tempfile::tempdir().unwrap();
        let (snapshot, mut policy) = fixture();
        let context = cli_fixture(root.path(), &snapshot);
        let mut new = policy.channels[0].clone();
        new.id = uuid::Uuid::new_v4().to_string();
        new.label = "本机模型".into();
        new.connection.provider_id = "local-new".into();
        new.connection.protocol = "ollama-native".into();
        new.connection.nodes[0].url = "https://local.example.test".into();
        new.discovered = vec![DiscoveredModel {
            id: "vendor/model-one".into(),
            label: "one".into(),
            rules: Default::default(),
        }];
        new.model_order = vec!["vendor/model-one".into()];
        new.selection = ModelSelection::Whitelist(vec!["vendor/model-one".into()]);
        policy.channels.push(new);
        let store = ModelStore::new(root.path());
        let local = store.load().unwrap();
        let local = store
            .stage_policy(
                &local.policy_digest,
                &local.workspace_digest,
                policy.clone(),
            )
            .unwrap();
        let plan = prepare(
            root.path(),
            &context,
            policy,
            local.library_digest,
            local.policy_digest,
            local.workspace_digest,
        )
        .await
        .unwrap();
        let result = apply(root.path(), &context, &plan.id).await.unwrap();
        assert_eq!(result.phase, "committed");
        assert!(root.path().join("safe-registration").exists());
    }
}
