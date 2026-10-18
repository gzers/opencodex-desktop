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
    collections::BTreeSet,
    path::{Path, PathBuf},
};
const PLAN_DIR: &str = "manager-state/model-save-plans";
const OWNERSHIP: &str = "manager-state/model-runtime-ownership.json";
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OwnedField {
    provider: String,
    key: String,
    model: String,
    original: Option<Value>,
    written: Value,
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
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePlanView {
    pub id: String,
    pub phase: String,
    pub catalog_count: usize,
    pub current_catalog_count: usize,
    pub changed_channels: Vec<String>,
    pub error: Option<String>,
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
                .map(|c| c.label.clone())
                .collect(),
            error: self.error.clone(),
        }
    }
}
fn path(root: &Path, id: &str) -> ModelResult<PathBuf> {
    require_uuid(id)?;
    let p = root.join(PLAN_DIR).join(format!("{id}.json"));
    safety::check_path(root, &p, true)?;
    Ok(p)
}
fn save(root: &Path, plan: &RuntimePlan) -> ModelResult<()> {
    atomic_write(&path(root, &plan.id)?, &serde_json::to_vec(plan)?, 0o600)?;
    Ok(())
}
fn load(root: &Path, id: &str) -> ModelResult<RuntimePlan> {
    Ok(serde_json::from_slice(&safety::read_file(
        &path(root, id)?,
        256 * 1024 * 1024,
    )?)?)
}
fn rows(snapshot: &UpstreamSnapshot) -> ModelResult<&Vec<Value>> {
    snapshot
        .models
        .as_array()
        .ok_or_else(|| invalid("runtime_models", "官方模型清单无效"))
}
fn catalog_slugs(snapshot: &UpstreamSnapshot) -> ModelResult<BTreeSet<String>> {
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
            if next["providers"][&field.provider][&field.key].get(&field.model)
                != Some(&field.written)
            {
                return Err(invalid(
                    "field_drift",
                    "受管模型字段已被外部修改，请显式比较后再保存",
                ));
            }
            let provider = next["providers"]
                .get_mut(&field.provider)
                .ok_or_else(|| invalid("provider", "受管渠道缺失"))?;
            if let Some(map) = provider.get_mut(&field.key).and_then(Value::as_object_mut) {
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
        if prior.is_none() {
            if ["openai", "chatgpt", "openai-multi", "combo"].contains(&provider_id.as_str()) {
                return Err(invalid(
                    "reserved_provider",
                    "此运行渠道身份保留，请使用新的自建标识",
                ));
            }
            if !matches!(&channel.selection,ModelSelection::Whitelist(values) if !values.is_empty())
            {
                return Err(invalid(
                    "new_channel_selection",
                    "新增渠道须明确勾选至少一个模型，不能以空白名单注册",
                ));
            }
            if channel.connection.protocol != "ollama-native"
                && channel.connection.credential_ref.is_none()
            {
                return Err(invalid("credential", "新增渠道尚未配置经过发现确认的密钥"));
            }
            next["providers"].as_object_mut().ok_or_else(||invalid("provider","渠道结构无效"))?.insert(provider_id.clone(),json!({"adapter":channel.connection.protocol,"baseUrl":"","authMode":if channel.connection.protocol=="ollama-native"{"local"}else{"key"},"models":channel.discovered.iter().map(|m|m.id.clone()).collect::<Vec<_>>(),"liveModels":true}));
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
        provider["baseUrl"] = json!(node.url);
        provider["adapter"] = json!(channel.connection.protocol);
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
                let slug = if let Some(model) = source {
                    model["namespaced"]
                        .as_str()
                        .ok_or_else(|| invalid("model_identity", "官方模型缺少路由身份"))?
                        .to_string()
                } else if prior.is_none() && channel.discovered.iter().any(|m| m.id == row.model_id)
                {
                    format!("{provider_id}/{}", row.model_id.replace('/', "-"))
                } else {
                    return Err(invalid(
                        "model_unverified",
                        "所选模型未在当前发现清单中核对",
                    ));
                };
                if !expected.insert(slug.clone()) {
                    return Err(invalid(
                        "model_identity_collision",
                        "模型原始身份编码发生碰撞，请调整选择",
                    ));
                }
                desired_order.push(slug);
            }
            let fields = &row.rules;
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
        if prior.is_some_and(|p| p["liveModels"] == false)
            && channel
                .discovered
                .iter()
                .any(|m| !known.iter().any(|v| v["id"].as_str() == Some(&m.id)))
        {
            return Err(invalid("discovery", "发现基准已变化"));
        }
    }
    if next["providers"]
        .get(next["defaultProvider"].as_str().unwrap_or(""))
        .is_some_and(|p| p["disabled"] == true)
    {
        return Err(invalid(
            "default_provider",
            "不能在未选择替代默认渠道时停用当前默认渠道",
        ));
    }
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
pub async fn prepare(
    root: &Path,
    context: &ProcessContext,
    policy: Policy,
    expected_library: String,
    expected_policy: String,
    expected_workspace: String,
) -> ModelResult<SavePlanView> {
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
    let before = upstream::snapshot(context).await?;
    let ownership_path = root.join(OWNERSHIP);
    safety::check_path(root, &ownership_path, true)?;
    let previous: Vec<OwnedField> = if ownership_path.exists() {
        serde_json::from_slice(&safety::read_file(&ownership_path, 16 * 1024 * 1024)?)?
    } else {
        vec![]
    };
    let (next_providers, next_order, expected_slugs) =
        compile_owned(&before, &local.library, &policy, &previous)?;
    let ownership = next_ownership(&before.providers, &next_providers, &previous, &policy)?;
    let disabled_before =
        string_array(&upstream::read(context, upstream::ReadOperation::DisabledModels).await?)?;
    let observed_providers = before.providers.clone();
    let observed_order = string_array(&before.order["pickerOrder"])?;
    let plan = RuntimePlan {
        id: uuid::Uuid::new_v4().to_string(),
        phase: "prepared".into(),
        expected_library,
        expected_policy,
        expected_workspace,
        policy,
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
    };
    save(root, &plan)?;
    Ok(plan.view())
}
pub fn query(root: &Path, id: &str) -> ModelResult<SavePlanView> {
    Ok(load(root, id)?.view())
}
pub async fn reconcile(
    root: &Path,
    context: &ProcessContext,
    id: &str,
) -> ModelResult<SavePlanView> {
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
    if context.executable()?.to_string_lossy() != plan.runtime_executable {
        return Err(invalid("runtime_binding", "运行来源改变，原计划仍保留"));
    }
    let actual = upstream::snapshot(context).await?;
    let disabled =
        string_array(&upstream::read(context, upstream::ReadOperation::DisabledModels).await?)?;
    let mut staged = plan.before.providers.clone();
    for channel in &plan.policy.channels {
        if plan.before.providers["providers"]
            .get(&channel.connection.provider_id)
            .is_none()
        {
            let mut value =
                plan.next_providers["providers"][&channel.connection.provider_id].clone();
            value["disabled"] = json!(true);
            value["newModelPolicy"] = json!("off");
            staged["providers"]
                .as_object_mut()
                .ok_or_else(|| invalid("provider", "渠道结构无效"))?
                .insert(channel.connection.provider_id.clone(), value);
        }
    }
    let attributable = actual.providers == plan.before.providers
        || actual.providers == plan.next_providers
        || actual.providers == staged;
    let original_order = string_array(&plan.before.order["pickerOrder"])?;
    let actual_order = string_array(&actual.order["pickerOrder"])?;
    let changed: BTreeSet<_> = disabled
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .symmetric_difference(&plan.disabled_before.iter().cloned().collect())
        .cloned()
        .collect();
    let model_slugs: BTreeSet<_> = rows(&plan.before)?
        .iter()
        .filter(|m| {
            plan.policy
                .channels
                .iter()
                .any(|c| m["provider"].as_str() == Some(&c.connection.provider_id))
        })
        .filter_map(|m| m["namespaced"].as_str().map(str::to_owned))
        .collect();
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
    if actual.providers == plan.next_providers
        && actual_order == plan.next_order
        && catalog_slugs(&actual)? == plan.expected_slugs
    {
        ModelStore::new(root).commit_runtime_policy(
            &plan.expected_library,
            &plan.expected_policy,
            &plan.expected_workspace,
            plan.policy.clone(),
        )?;
        let ownership = root.join(OWNERSHIP);
        safety::check_path(root, &ownership, true)?;
        atomic_write(&ownership, &serde_json::to_vec(&plan.ownership)?, 0o600)?;
        plan.phase = "committed".into();
        plan.error = None;
    } else if actual.providers == plan.before.providers
        && actual_order == original_order
        && plan.observed_disabled == plan.disabled_before
    {
        plan.phase = "compensated".into();
        plan.error = None;
    }
    save(root, &plan)?;
    Ok(plan.view())
}
pub async fn apply(root: &Path, context: &ProcessContext, id: &str) -> ModelResult<SavePlanView> {
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
    if context.executable()?.to_string_lossy() != plan.runtime_executable {
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
        let new_channels: Vec<_> = plan
            .policy
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
            if new_channels
                .iter()
                .any(|c| c.connection.credential_ref.is_some())
            {
                let management = super::management::Management::bind(&frozen).await?;
                for channel in &new_channels {
                    if let Some(reference) = &channel.connection.credential_ref {
                        let key = super::discovery::credential(reference)?;
                        management
                            .add_key(&channel.connection.provider_id, &key)
                            .await?;
                    }
                }
            }
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
        let visibility_channels = plan.policy.channels.clone();
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
        let observed_order = upstream::read(&frozen, upstream::ReadOperation::ModelOrder).await?;
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
        plan.phase = "runtime_verified".into();
        save(root, &plan)?;
        let ownership_path = root.join(OWNERSHIP);
        safety::check_path(root, &ownership_path, true)?;
        atomic_write(
            &ownership_path,
            &serde_json::to_vec(&plan.ownership)?,
            0o600,
        )?;
        ModelStore::new(root).commit_runtime_policy(
            &plan.expected_library,
            &plan.expected_policy,
            &plan.expected_workspace,
            plan.policy.clone(),
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
        .cloned()
        .collect();
    for channel in &policy.channels {
        let provider = &channel.connection.provider_id;
        for key in [
            "modelAdapters",
            "modelContextWindows",
            "modelInputModalities",
            "modelReasoningEfforts",
            "modelDefaultReasoningEfforts",
            "modelSupportsServiceTier",
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
                    if let Some(written) = b {
                        result.push(OwnedField {
                            provider: provider.clone(),
                            key: key.into(),
                            model: model.clone(),
                            original,
                            written: written.clone(),
                        });
                    }
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
    if context.executable()?.to_string_lossy() != plan.runtime_executable {
        return Err(invalid("runtime_binding", "运行来源改变，补偿已拒绝"));
    }
    let current = upstream::read(context, upstream::ReadOperation::ProviderSnapshot).await?;
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
    use super::*;
    use std::collections::BTreeMap;
    fn fixture() -> (UpstreamSnapshot, Policy) {
        let node = ServiceNode {
            id: uuid::Uuid::new_v4().to_string(),
            name: "主节点".into(),
            url: "https://example.test/v1".into(),
        };
        let channel = ChannelPolicy {
            id: uuid::Uuid::new_v4().to_string(),
            label: "测试渠道".into(),
            template_refs: vec![],
            model_refs: BTreeMap::new(),
            connection: ChannelConnection {
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
    fn managed_inheritance_restores_original_values_but_external_edits_fail_closed() {
        let (mut snapshot, policy) = fixture();
        snapshot.providers["providers"]["test"]["modelContextWindows"] = json!({"b":8000});
        let ownership = vec![OwnedField {
            provider: "test".into(),
            key: "modelContextWindows".into(),
            model: "b".into(),
            original: Some(json!(4000)),
            written: json!(8000),
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
def save(name,value):(root/(name+'.json')).write_text(json.dumps(value))
if args==['--version']:print('2.84.0')
elif args[:2]==['provider','snapshot']:print(json.dumps(read('providers')))
elif args[:2]==['models','live']:print(json.dumps(read('models')))
elif args[:3]==['models','order','status']:print(json.dumps(read('order')))
elif args[:2]==['inspect','catalog']:print(json.dumps(read('catalog')))
elif args[:3]==['config','get','disabledModels']:print(json.dumps(read('disabled')))
elif args[:3]==['config','set','disabledModels']:save('disabled',json.loads(args[3]));print('{}')
elif args[:2]==['provider','apply']:
 baseline=json.loads(Path(args[args.index('--baseline')+1]).read_text())
 if baseline!=read('providers'):sys.exit(5)
 next_value=json.load(sys.stdin);old=read('providers');models=read('models')
 for name,provider in next_value['providers'].items():
  if name not in old['providers']:
   assert provider.get('disabled') is True and provider.get('selectedModels'), 'new provider exposed all models'
   (root/'safe-registration').write_text('disabled with nonempty allowlist')
   for model in provider.get('models',[]):models.append({'provider':name,'id':model,'namespaced':name+'/'+model.replace('/','-'),'disabled':False})
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
        assert_eq!(store.load().unwrap().policy, policy);
        assert!(root.path().join("backups").is_dir());
        assert_eq!(
            apply(root.path(), &context, &plan.id).await.unwrap().phase,
            "committed"
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
        assert_eq!(partial.error.as_deref(), Some("order_write"));
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
