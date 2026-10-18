//! Portable model objects and exact dependency closure. Transfer never applies runtime state.
use super::{
    schema::*,
    store::{seed_library, sha256, validate_library, ModelStore},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortableChannel {
    pub id: String,
    pub label: String,
    pub protocol: String,
    pub selection: ModelSelection,
    pub model_order: Vec<String>,
    pub discovered: Vec<DiscoveredModel>,
    pub template_refs: Vec<TemplateRef>,
    pub model_refs: BTreeMap<String, TemplateRef>,
    pub defaults: ModelRules,
    pub overrides: BTreeMap<String, ModelRules>,
    pub family_overrides: BTreeMap<String, ModelRules>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortablePolicy {
    pub template_refs: Option<Vec<TemplateRef>>,
    pub channel_order: Option<Vec<String>>,
    pub channels: Vec<PortableChannel>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TransferPayload {
    pub templates: Vec<Template>,
    pub policy: Option<PortablePolicy>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelPackage {
    pub format: String,
    pub format_version: u32,
    pub schema_version: u32,
    pub created_at: String,
    pub digest: String,
    pub payload: TransferPayload,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportScope {
    pub include_policy: bool,
    pub channel_ids: Vec<String>,
    pub template_refs: Vec<TemplateRef>,
    pub include_global: bool,
}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReceiveScope {
    pub template_refs: Vec<TemplateRef>,
    pub channel_ids: Vec<String>,
    pub include_global: bool,
    #[serde(default)]
    pub preferences: bool,
    #[serde(default)]
    pub extensions: bool,
    #[serde(default)]
    pub endpoint_ids: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IncomingPlan {
    pub id: String,
    pub expected_library: String,
    pub expected_policy: String,
    pub expected_workspace: String,
    pub package: ModelPackage,
    pub merged_library: Library,
    pub merged_policy: Option<Policy>,
    pub notices: Vec<String>,
    #[serde(default)]
    pub preview_error: Option<String>,
    #[serde(default)]
    pub available_keys: Vec<String>,
    #[serde(default)]
    pub file_digest: Option<String>,
    #[serde(default)]
    pub manager: Option<crate::modules::sync::portable_manager::Settings>,
    #[serde(default)]
    pub available_webdav_passwords: Vec<String>,
    #[serde(default)]
    pub adoption: Option<ImportAdoption>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportAdoption {
    pub keys: bool,
    pub webdav: bool,
    pub completed: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileExportPlan {
    pub id: String,
    pub package: ModelPackage,
    pub protected: bool,
    pub file_digest: String,
    #[serde(default)]
    pub credential_count: usize,
    #[serde(default)]
    pub manager: Option<crate::modules::sync::portable_manager::Settings>,
}
fn export_plan_path(root: &Path, id: &str) -> ModelResult<std::path::PathBuf> {
    require_uuid(id)?;
    let path = root
        .join("manager-state/model-file-exports")
        .join(format!("{id}.json"));
    crate::modules::backup::safety::check_path(root, &path, true)?;
    Ok(path)
}
pub fn prepare_file(
    root: &Path,
    scope: ExportScope,
    passphrase: Option<&str>,
) -> ModelResult<FileExportPlan> {
    prepare_file_with_keys(root, scope, passphrase, false)
}
pub fn prepare_file_with_keys(
    root: &Path,
    scope: ExportScope,
    passphrase: Option<&str>,
    include_keys: bool,
) -> ModelResult<FileExportPlan> {
    prepare_file_scope(
        root,
        scope,
        passphrase,
        include_keys,
        &crate::modules::sync::portable_manager::Scope::default(),
    )
}
pub fn prepare_file_scope(
    root: &Path,
    scope: ExportScope,
    passphrase: Option<&str>,
    include_keys: bool,
    manager_scope: &crate::modules::sync::portable_manager::Scope,
) -> ModelResult<FileExportPlan> {
    let id = uuid::Uuid::new_v4().to_string();
    let no_models = !scope.include_policy
        && scope.template_refs.is_empty()
        && (manager_scope.preferences || manager_scope.extensions || manager_scope.webdav);
    let mut package = export(root, scope)?;
    if no_models {
        package.payload.templates.clear();
        package.digest = sha256(&serde_json::to_vec(&package.payload)?);
    }
    let (bytes, credential_count, manager) = super::file_objects::seal_manager(
        root,
        &id,
        &package,
        include_keys,
        passphrase,
        manager_scope,
    )?;
    let plan = FileExportPlan {
        id,
        package,
        protected: passphrase.is_some(),
        file_digest: sha256(&bytes),
        credential_count,
        manager: Some(manager),
    };
    let payload = export_plan_path(root, &plan.id)?.with_extension("ocxdconf");
    crate::modules::backup::safety::check_path(root, &payload, true)?;
    crate::infrastructure::atomic_write::atomic_write(&payload, &bytes, 0o600)?;
    crate::infrastructure::atomic_write::atomic_write(
        &export_plan_path(root, &plan.id)?,
        &serde_json::to_vec(&plan)?,
        0o600,
    )?;
    Ok(plan)
}
pub fn file_bytes(root: &Path, id: &str) -> ModelResult<Vec<u8>> {
    let plan: FileExportPlan = serde_json::from_slice(&crate::modules::backup::safety::read_file(
        &export_plan_path(root, id)?,
        64 * 1024 * 1024,
    )?)?;
    if plan.id != id {
        return Err(invalid("export_identity", "导出计划身份不一致"));
    }
    let path = export_plan_path(root, id)?.with_extension("ocxdconf");
    crate::modules::backup::safety::check_path(root, &path, false)?;
    let bytes = crate::modules::backup::safety::read_file(
        &path,
        crate::modules::sync::portable_file::FILE_LIMIT as u64,
    )?;
    if sha256(&bytes) != plan.file_digest {
        return Err(invalid("export_digest", "已冻结导出文件发生变化，未导出"));
    }
    Ok(bytes)
}
pub fn file_package(root: &Path, id: &str) -> ModelResult<ModelPackage> {
    let plan: FileExportPlan = serde_json::from_slice(&crate::modules::backup::safety::read_file(
        &export_plan_path(root, id)?,
        64 * 1024 * 1024,
    )?)?;
    if plan.id != id {
        return Err(invalid("export_identity", "导出计划身份不一致"));
    }
    validate_package(&plan.package)?;
    Ok(plan.package)
}
fn revision_refs(policy: &PortablePolicy) -> Vec<&TemplateRef> {
    policy
        .template_refs
        .iter()
        .flatten()
        .chain(
            policy
                .channels
                .iter()
                .flat_map(|c| c.template_refs.iter().chain(c.model_refs.values())),
        )
        .collect()
}
pub(super) fn portable(channel: &ChannelPolicy) -> PortableChannel {
    PortableChannel {
        id: channel.id.clone(),
        label: channel.label.clone(),
        protocol: channel.connection.protocol.clone(),
        selection: channel.selection.clone(),
        model_order: channel.model_order.clone(),
        discovered: channel.discovered.clone(),
        template_refs: channel.template_refs.clone(),
        model_refs: channel.model_refs.clone(),
        defaults: channel.defaults.clone(),
        overrides: channel.overrides.clone(),
        family_overrides: channel.family_overrides.clone(),
    }
}
pub fn export(root: &Path, scope: ExportScope) -> ModelResult<ModelPackage> {
    let snapshot = ModelStore::new(root).load()?;
    let mut wanted: BTreeSet<(String, String)> = scope
        .template_refs
        .iter()
        .map(|r| (r.template_id.clone(), r.revision_id.clone()))
        .collect();
    let policy = if scope.include_policy {
        let channels = snapshot
            .policy
            .channels
            .iter()
            .filter(|c| scope.channel_ids.contains(&c.id))
            .map(portable)
            .collect();
        Some(PortablePolicy {
            template_refs: if scope.include_global {
                Some(snapshot.policy.template_refs.clone())
            } else {
                None
            },
            channel_order: if scope.include_global {
                Some(
                    snapshot
                        .policy
                        .channels
                        .iter()
                        .map(|c| c.id.clone())
                        .collect(),
                )
            } else {
                None
            },
            channels,
        })
    } else {
        None
    };
    if let Some(policy) = &policy {
        for reference in revision_refs(policy) {
            wanted.insert((reference.template_id.clone(), reference.revision_id.clone()));
        }
    }
    let full_library = !scope.include_policy && scope.template_refs.is_empty();
    let mut templates = vec![];
    for template in &snapshot.library.templates {
        let mut selected = template.clone();
        selected
            .revisions
            .retain(|r| full_library || wanted.contains(&(template.id.clone(), r.id.clone())));
        if selected.revisions.is_empty() {
            continue;
        }
        if selected
            .default_revision
            .as_ref()
            .is_some_and(|id| !selected.revisions.iter().any(|r| &r.id == id))
        {
            selected.default_revision = None;
        }
        templates.push(selected);
    }
    for (reference, id) in &wanted {
        if !templates
            .iter()
            .any(|t| &t.id == reference && t.revisions.iter().any(|r| &r.id == id))
        {
            return Err(invalid("missing_dependency", "导出范围缺少精确保存版本"));
        }
    }
    let payload = TransferPayload { templates, policy };
    let digest = sha256(&serde_json::to_vec(&payload)?);
    Ok(ModelPackage {
        format: "OCXD_MODEL_CONFIG".into(),
        format_version: 1,
        schema_version: SCHEMA_VERSION,
        created_at: chrono::Utc::now().to_rfc3339(),
        digest,
        payload,
    })
}
pub(super) fn validate_package(package: &ModelPackage) -> ModelResult<()> {
    if package.format != "OCXD_MODEL_CONFIG"
        || package.format_version != 1
        || package.schema_version != SCHEMA_VERSION
    {
        return Err(invalid("package_version", "模型配置文件格式/版本不受支持"));
    }
    if sha256(&serde_json::to_vec(&package.payload)?) != package.digest {
        return Err(invalid("package_digest", "模型配置文件摘要不匹配，未接受"));
    }
    if package.payload.templates.len() > 5000
        || package
            .payload
            .policy
            .as_ref()
            .is_some_and(|p| p.channels.len() > 256)
    {
        return Err(invalid("package_limit", "配置包对象超过上限"));
    }
    let mut templates = BTreeSet::new();
    let mut revisions = BTreeSet::new();
    for template in &package.payload.templates {
        require_uuid(&template.id)?;
        if !templates.insert(&template.id)
            || template.label.trim().is_empty()
            || template.label.len() > 256
            || template.revisions.is_empty()
            || template
                .default_revision
                .as_ref()
                .is_some_and(|id| !template.revisions.iter().any(|r| &r.id == id))
        {
            return Err(invalid("package_identity", "模版重复/名称无效/版本为空"));
        }
        for revision in &template.revisions {
            require_uuid(&revision.id)?;
            revision.content.validate()?;
            if !revisions.insert(&revision.id)
                || chrono::DateTime::parse_from_rfc3339(&revision.created_at).is_err()
                || revision.saved_label.trim().is_empty()
                || revision.saved_label.len() > 256
                || super::store::content_digest(&revision.saved_label, &revision.content)?
                    != revision.digest
            {
                return Err(invalid("revision_conflict", "保存版本重复或摘要不符"));
            }
        }
    }
    if let Some(policy) = &package.payload.policy {
        let mut channels = BTreeSet::new();
        for channel in &policy.channels {
            require_uuid(&channel.id)?;
            if !channels.insert(&channel.id)
                || channel.label.trim().is_empty()
                || channel.label.len() > 256
                || channel.protocol.len() > 128
            {
                return Err(invalid("channel", "包内渠道身份重复"));
            }
            let unique = |ids: &[String]| {
                ids.iter().collect::<BTreeSet<_>>().len() == ids.len()
                    && ids.iter().all(|id| !id.is_empty())
            };
            if !unique(&channel.model_order)
                || channel
                    .discovered
                    .iter()
                    .map(|m| &m.id)
                    .collect::<BTreeSet<_>>()
                    .len()
                    != channel.discovered.len()
            {
                return Err(invalid("channel_models", "模型身份或顺序重复"));
            }
            if let ModelSelection::Whitelist(ids) = &channel.selection {
                if !unique(ids) {
                    return Err(invalid("selection", "模型选择包含重复或空身份"));
                }
            }
            channel.defaults.validate()?;
            for rules in channel
                .overrides
                .values()
                .chain(channel.family_overrides.values())
                .chain(channel.discovered.iter().map(|m| &m.rules))
            {
                rules.validate()?;
            }
            let ids = channel
                .template_refs
                .iter()
                .map(|r| &r.template_id)
                .collect::<BTreeSet<_>>();
            if ids.len() != channel.template_refs.len() {
                return Err(invalid("reference", "重复渠道模版引用"));
            }
        }
        if policy.template_refs.as_ref().is_some_and(|refs| {
            refs.iter()
                .map(|r| &r.template_id)
                .collect::<BTreeSet<_>>()
                .len()
                != refs.len()
        }) || policy
            .channel_order
            .as_ref()
            .is_some_and(|ids| ids.iter().collect::<BTreeSet<_>>().len() != ids.len())
        {
            return Err(invalid("reference", "全局引用或渠道顺序重复"));
        }
        for reference in revision_refs(policy) {
            if !package.payload.templates.iter().any(|t| {
                t.id == reference.template_id
                    && t.revisions.iter().any(|r| r.id == reference.revision_id)
            }) {
                return Err(invalid(
                    "missing_dependency",
                    "包内缺少策略所需的精确模版版本",
                ));
            }
        }
    }
    Ok(())
}
fn merge(
    local: &Snapshot,
    package: &ModelPackage,
) -> ModelResult<(Library, Option<Policy>, Vec<String>)> {
    validate_package(package)?;
    let mut library = local.library.clone();
    let seeds = seed_library();
    let mut notices = vec![];
    for incoming in &package.payload.templates {
        if let Some(existing) = library.templates.iter_mut().find(|t| t.id == incoming.id) {
            for revision in &incoming.revisions {
                if let Some(current) = existing.revisions.iter().find(|r| r.id == revision.id) {
                    if current.content != revision.content
                        || current.digest != revision.digest
                        || current.saved_label != revision.saved_label
                        || current.created_at != revision.created_at
                        || current.changes != revision.changes
                        || current.base_revision != revision.base_revision
                        || current.origin != revision.origin
                    {
                        return Err(invalid(
                            "revision_conflict",
                            "同一完整版本身份有不同内容或历史事实，未覆盖",
                        ));
                    }
                } else {
                    existing.revisions.push(revision.clone());
                }
            }
            if existing.default_revision != incoming.default_revision {
                notices.push(format!(
                    "{}：远端默认指针仅作候选，保持本机默认",
                    existing.label
                ));
            }
        } else {
            let mut candidate = incoming.clone();
            candidate.source = TemplateSource::User;
            candidate.default_revision = None;
            if seeds.templates.iter().any(|t| t.id == candidate.id) {
                return Err(invalid("source", "内置基线身份冲突"));
            }
            notices.push(format!(
                "{}：新增自建来源候选，不授予内置保护或自动采用",
                candidate.label
            ));
            library.templates.push(candidate);
        }
    }
    validate_library(&library)?;
    let policy = if let Some(incoming) = &package.payload.policy {
        let mut policy = local
            .workspace
            .as_ref()
            .map(|w| w.policy.clone())
            .unwrap_or_else(|| local.policy.clone());
        if let Some(global) = &incoming.template_refs {
            if local
                .workspace
                .as_ref()
                .is_some_and(|w| w.policy.template_refs != local.policy.template_refs)
                && policy.template_refs != *global
            {
                return Err(invalid(
                    "draft_conflict",
                    "全局采用已有未保存修改，请先明确保留或处理本机草稿",
                ));
            }
            policy.template_refs = global.clone();
        }
        for channel in &incoming.channels {
            let committed = local.policy.channels.iter().find(|c| c.id == channel.id);
            let working = policy.channels.iter().find(|c| c.id == channel.id);
            if working != committed
                && working.is_some_and(|c| {
                    serde_json::to_value(portable(c)).ok() != serde_json::to_value(channel).ok()
                })
            {
                return Err(invalid(
                    "draft_conflict",
                    "渠道已有未保存修改，未覆盖；请先比较本机草稿",
                ));
            }
            let mut connection = working
                .or(committed)
                .map(|c| c.connection.clone())
                .unwrap_or_default();
            connection.protocol = channel.protocol.clone();
            if working.is_none() {
                notices.push(format!("{}：跨机渠道待配置本机连接，未启用", channel.label));
            }
            let next = ChannelPolicy {
                id: channel.id.clone(),
                label: channel.label.clone(),
                connection,
                template_refs: channel.template_refs.clone(),
                model_refs: channel.model_refs.clone(),
                selection: channel.selection.clone(),
                model_order: channel.model_order.clone(),
                discovered: channel.discovered.clone(),
                defaults: channel.defaults.clone(),
                overrides: channel.overrides.clone(),
                family_overrides: channel.family_overrides.clone(),
            };
            if let Some(index) = policy.channels.iter().position(|c| c.id == channel.id) {
                policy.channels[index] = next;
            } else {
                policy.channels.push(next);
            }
        }
        if let Some(order) = &incoming.channel_order {
            let local_order: Vec<_> = local
                .workspace
                .as_ref()
                .map(|w| &w.policy)
                .unwrap_or(&local.policy)
                .channels
                .iter()
                .map(|c| c.id.clone())
                .collect();
            let committed_order: Vec<_> =
                local.policy.channels.iter().map(|c| c.id.clone()).collect();
            if local.workspace.is_some() && local_order != committed_order && local_order != *order
            {
                return Err(invalid(
                    "draft_conflict",
                    "渠道顺序已有未保存修改，未自动覆盖",
                ));
            }
            let rank: BTreeMap<_, _> = order.iter().enumerate().map(|(i, id)| (id, i)).collect();
            policy
                .channels
                .sort_by_key(|c| rank.get(&c.id).copied().unwrap_or(usize::MAX));
        }
        notices.push("渠道策略仅进入本机草稿；运行保存、预算和重启核对仍须明确执行".into());
        Some(policy)
    } else {
        None
    };
    Ok((library, policy, notices))
}
/// Partial uploads preserve every unrelated remote object. The caller still
/// freezes the remote ETag and publishes with a conditional pointer update.
pub(super) fn merge_packages(
    remote: ModelPackage,
    local: ModelPackage,
) -> ModelResult<ModelPackage> {
    validate_package(&remote)?;
    validate_package(&local)?;
    let mut payload = remote.payload;
    for incoming in local.payload.templates {
        if let Some(existing) = payload.templates.iter_mut().find(|t| t.id == incoming.id) {
            for revision in incoming.revisions {
                if let Some(old) = existing.revisions.iter().find(|r| r.id == revision.id) {
                    if old != &revision {
                        return Err(invalid(
                            "revision_conflict",
                            "远端同一保存版本有不同内容，未发布",
                        ));
                    }
                } else {
                    existing.revisions.push(revision);
                }
            }
        } else {
            payload.templates.push(incoming);
        }
    }
    if let Some(incoming) = local.payload.policy {
        if let Some(existing) = payload.policy.as_mut() {
            if incoming.template_refs.is_some() {
                existing.template_refs = incoming.template_refs;
            }
            if incoming.channel_order.is_some() {
                existing.channel_order = incoming.channel_order;
            }
            for channel in incoming.channels {
                if let Some(index) = existing.channels.iter().position(|c| c.id == channel.id) {
                    existing.channels[index] = channel;
                } else {
                    existing.channels.push(channel);
                }
            }
        } else {
            payload.policy = Some(incoming);
        }
    }
    let result = ModelPackage {
        format: "OCXD_MODEL_CONFIG".into(),
        format_version: 1,
        schema_version: SCHEMA_VERSION,
        created_at: chrono::Utc::now().to_rfc3339(),
        digest: sha256(&serde_json::to_vec(&payload)?),
        payload,
    };
    validate_package(&result)?;
    Ok(result)
}
fn incoming_path(root: &Path, id: &str) -> ModelResult<std::path::PathBuf> {
    require_uuid(id)?;
    let path = root
        .join("manager-state/model-config-incoming")
        .join(format!("{id}.json"));
    crate::modules::backup::safety::check_path(root, &path, true)?;
    Ok(path)
}
pub fn receive(root: &Path, package: ModelPackage) -> ModelResult<IncomingPlan> {
    validate_package(&package)?;
    let local = ModelStore::new(root).load()?;
    let (merged_library, merged_policy, notices, preview_error) = match merge(&local, &package) {
        Ok((library, policy, notices)) => (library, policy, notices, None),
        Err(error) => (
            local.library.clone(),
            None,
            vec![error.message],
            Some(error.code),
        ),
    };
    let plan = IncomingPlan {
        id: uuid::Uuid::new_v4().to_string(),
        expected_library: local.library_digest,
        expected_policy: local.policy_digest,
        expected_workspace: local.workspace_digest,
        package,
        merged_library,
        merged_policy,
        notices,
        preview_error,
        available_keys: vec![],
        file_digest: None,
        manager: None,
        available_webdav_passwords: vec![],
        adoption: None,
    };
    let bytes = serde_json::to_vec_pretty(&plan)?;
    if bytes.len() > 64 * 1024 * 1024 {
        return Err(invalid("package_limit", "模型包超过64MiB"));
    }
    crate::infrastructure::atomic_write::atomic_write(
        &incoming_path(root, &plan.id)?,
        &bytes,
        0o600,
    )?;
    Ok(plan)
}
pub fn select_received(root: &Path, id: &str, scope: ReceiveScope) -> ModelResult<IncomingPlan> {
    if super::file_objects::has_adoption(root, id)? {
        return Err(invalid(
            "import_in_progress",
            "原导入已开始采用，请先核对其结果，不改变原范围",
        ));
    }
    let original: IncomingPlan = serde_json::from_slice(
        &crate::modules::backup::safety::read_file(&incoming_path(root, id)?, 64 * 1024 * 1024)?,
    )?;
    if original.id != id {
        return Err(invalid("incoming_identity", "接收记录身份不一致"));
    }
    let current = ModelStore::new(root).load()?;
    if (
        current.library_digest,
        current.policy_digest,
        current.workspace_digest,
    ) != (
        original.expected_library.clone(),
        original.expected_policy.clone(),
        original.expected_workspace.clone(),
    ) {
        return Err(invalid(
            "stale_baseline",
            "接收后本机基准已变化，请重新比较原文件",
        ));
    }
    if scope.template_refs.is_empty()
        && scope.channel_ids.is_empty()
        && !scope.include_global
        && !scope.preferences
        && !scope.extensions
        && scope.endpoint_ids.is_empty()
    {
        return Err(invalid("import_scope", "请选择至少一个接收对象"));
    }
    let mut package = original.package.clone();
    validate_package(&package)?;
    if scope.channel_ids.iter().any(|id| {
        package
            .payload
            .policy
            .as_ref()
            .is_none_or(|p| !p.channels.iter().any(|c| &c.id == id))
    }) {
        return Err(invalid("import_scope", "选择了包内不存在的渠道"));
    }
    let mut wanted = scope
        .template_refs
        .iter()
        .map(|r| (r.template_id.clone(), r.revision_id.clone()))
        .collect::<BTreeSet<_>>();
    for reference in &scope.template_refs {
        if !package.payload.templates.iter().any(|t| {
            t.id == reference.template_id
                && t.revisions.iter().any(|r| r.id == reference.revision_id)
        }) {
            return Err(invalid("import_scope", "选择了包内不存在的保存版本"));
        }
    }
    if let Some(policy) = &mut package.payload.policy {
        policy
            .channels
            .retain(|channel| scope.channel_ids.contains(&channel.id));
        if !scope.include_global {
            policy.template_refs = None;
            policy.channel_order = None;
        }
        for reference in revision_refs(policy) {
            wanted.insert((reference.template_id.clone(), reference.revision_id.clone()));
        }
        if policy.channels.is_empty()
            && policy.template_refs.is_none()
            && policy.channel_order.is_none()
        {
            package.payload.policy = None;
        }
    } else if scope.include_global {
        return Err(invalid("import_scope", "包内没有全局字段"));
    }
    for template in &mut package.payload.templates {
        template
            .revisions
            .retain(|revision| wanted.contains(&(template.id.clone(), revision.id.clone())));
        if template
            .default_revision
            .as_ref()
            .is_some_and(|id| !template.revisions.iter().any(|r| &r.id == id))
        {
            template.default_revision = None;
        }
    }
    package
        .payload
        .templates
        .retain(|template| !template.revisions.is_empty());
    package.digest = sha256(&serde_json::to_vec(&package.payload)?);
    let mut selected = receive(root, package)?;
    if let Some(manager) = &original.manager {
        if scope.preferences && manager.preferences.is_none()
            || scope.extensions && manager.extensions.is_none()
            || scope
                .endpoint_ids
                .iter()
                .any(|id| !manager.webdav.iter().any(|e| &e.id == id))
        {
            return Err(invalid("import_scope", "选择了包内不存在的管理器配置"));
        }
        selected.manager = Some(crate::modules::sync::portable_manager::Settings {
            preferences: if scope.preferences {
                manager.preferences.clone()
            } else {
                None
            },
            extensions: if scope.extensions {
                manager.extensions.clone()
            } else {
                None
            },
            webdav: manager
                .webdav
                .iter()
                .filter(|e| scope.endpoint_ids.contains(&e.id))
                .cloned()
                .collect(),
        });
        selected.available_webdav_passwords = original
            .available_webdav_passwords
            .iter()
            .filter(|id| scope.endpoint_ids.contains(id))
            .cloned()
            .collect();
        crate::modules::sync::portable_manager::select(
            root,
            id,
            &selected.id,
            scope.preferences,
            scope.extensions,
            &scope.endpoint_ids,
        )?;
    }
    super::file_objects::copy_source(root, &original, &mut selected)?;
    write_incoming(root, &selected)?;
    Ok(selected)
}
pub fn accept(root: &Path, id: &str) -> ModelResult<Snapshot> {
    accept_with_credentials(root, id, &BTreeMap::new())
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AcceptanceReceipt {
    id: String,
    package_digest: String,
    intent_digest: String,
}
pub(super) fn verify_acceptance(
    root: &Path,
    plan: &IncomingPlan,
    credentials: &BTreeMap<String, String>,
) -> ModelResult<bool> {
    let path = incoming_path(root, &plan.id)?.with_extension("accepted.json");
    crate::modules::backup::safety::check_path(root, &path, true)?;
    if !path.exists() {
        return Ok(false);
    }
    let receipt: AcceptanceReceipt =
        serde_json::from_slice(&crate::modules::backup::safety::read_file(&path, 4096)?)?;
    if receipt.id != plan.id
        || receipt.package_digest != plan.package.digest
        || receipt.intent_digest != sha256(&serde_json::to_vec(credentials)?)
    {
        return Err(invalid(
            "import_replay_conflict",
            "该接收操作已确认其他范围或凭据选择，请重新比较",
        ));
    }
    Ok(true)
}
pub(super) fn accepted(root: &Path, plan: &IncomingPlan) -> ModelResult<bool> {
    let path = incoming_path(root, &plan.id)?.with_extension("accepted.json");
    crate::modules::backup::safety::check_path(root, &path, true)?;
    if !path.exists() {
        return Ok(false);
    }
    let receipt: AcceptanceReceipt =
        serde_json::from_slice(&crate::modules::backup::safety::read_file(&path, 4096)?)?;
    if receipt.id != plan.id || receipt.package_digest != plan.package.digest {
        return Err(invalid("incoming_identity", "原接收回执与清单不同"));
    }
    Ok(true)
}
pub(super) fn intended_applied(
    root: &Path,
    plan: &IncomingPlan,
    credentials: &BTreeMap<String, String>,
) -> ModelResult<bool> {
    let local = ModelStore::new(root).load()?;
    let mut policy = plan.merged_policy.clone();
    bind_credentials(&mut policy, credentials)?;
    Ok(plan.preview_error.is_none()
        && local.library == plan.merged_library
        && policy.as_ref().is_none_or(|policy| {
            local
                .workspace
                .as_ref()
                .is_some_and(|w| &w.policy == policy)
        }))
}
pub(super) fn read_incoming(root: &Path, id: &str) -> ModelResult<IncomingPlan> {
    let plan: IncomingPlan = serde_json::from_slice(&crate::modules::backup::safety::read_file(
        &incoming_path(root, id)?,
        64 * 1024 * 1024,
    )?)?;
    if plan.id != id {
        return Err(invalid("incoming_identity", "接收记录身份不一致"));
    }
    validate_package(&plan.package)?;
    Ok(plan)
}
pub(super) fn write_incoming(root: &Path, plan: &IncomingPlan) -> ModelResult<()> {
    crate::infrastructure::atomic_write::atomic_write(
        &incoming_path(root, &plan.id)?,
        &serde_json::to_vec(plan)?,
        0o600,
    )?;
    Ok(())
}
pub(super) fn accept_with_credentials(
    root: &Path,
    id: &str,
    credentials: &BTreeMap<String, String>,
) -> ModelResult<Snapshot> {
    let path = incoming_path(root, id)?;
    crate::modules::backup::safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&path),
        true,
    )?;
    let _guard = crate::infrastructure::locking::TargetFileLock::lock(&path)?;
    let plan = read_incoming(root, id)?;
    let local = ModelStore::new(root).load()?;
    for (channel, reference) in credentials {
        require_uuid(reference)?;
        if plan
            .package
            .payload
            .policy
            .as_ref()
            .is_none_or(|p| !p.channels.iter().any(|c| &c.id == channel))
        {
            return Err(invalid("credential_scope", "凭据目标不在已选接收渠道内"));
        }
    }
    let intent_digest = sha256(&serde_json::to_vec(credentials)?);
    let receipt_path = path.with_extension("accepted.json");
    crate::modules::backup::safety::check_path(root, &receipt_path, true)?;
    if verify_acceptance(root, &plan, credentials)? {
        return Ok(local);
    }
    let mut intended_policy = plan.merged_policy.clone();
    bind_credentials(&mut intended_policy, credentials)?;
    let already_applied = plan.preview_error.is_none()
        && local.library == plan.merged_library
        && intended_policy.as_ref().is_none_or(|policy| {
            local
                .workspace
                .as_ref()
                .is_some_and(|w| &w.policy == policy)
        });
    if (
        local.library_digest.as_str(),
        local.policy_digest.as_str(),
        local.workspace_digest.as_str(),
    ) != (
        plan.expected_library.as_str(),
        plan.expected_policy.as_str(),
        plan.expected_workspace.as_str(),
    ) && !already_applied
    {
        return Err(invalid(
            "stale_baseline",
            "接收后本地内容已变化，请重新比较导入",
        ));
    }
    let snapshot = if already_applied {
        local
    } else {
        let (library, mut policy, _) = merge(&local, &plan.package)?;
        bind_credentials(&mut policy, credentials)?;
        ModelStore::new(root).accept_import(
            &plan.expected_library,
            &plan.expected_policy,
            &plan.expected_workspace,
            library,
            policy,
        )?
    };
    crate::infrastructure::atomic_write::atomic_write(
        &receipt_path,
        &serde_json::to_vec(&AcceptanceReceipt {
            id: id.into(),
            package_digest: plan.package.digest,
            intent_digest,
        })?,
        0o600,
    )?;
    Ok(snapshot)
}
fn bind_credentials(
    policy: &mut Option<Policy>,
    credentials: &BTreeMap<String, String>,
) -> ModelResult<()> {
    for (id, reference) in credentials {
        let channel = policy
            .as_mut()
            .and_then(|p| p.channels.iter_mut().find(|c| &c.id == id))
            .ok_or_else(|| invalid("credential_scope", "接收草稿缺少凭据目标"))?;
        channel.connection.credential_ref = Some(reference.clone());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn create(root: &Path) -> Commit {
        let store = ModelStore::new(root);
        let snapshot = store.load().unwrap();
        let mut content = TemplateContent::default();
        content.matching.include.push(Pattern {
            kind: MatchKind::Prefix,
            value: "test-".into(),
        });
        store
            .save(SaveTemplate {
                expected_library: snapshot.library_digest,
                template_id: None,
                base_revision: None,
                label: "用户模版".into(),
                content,
            })
            .unwrap()
    }
    fn all() -> ExportScope {
        ExportScope {
            include_policy: false,
            channel_ids: vec![],
            template_refs: vec![],
            include_global: false,
        }
    }
    #[test]
    fn protected_file_export_freezes_bytes_and_never_records_the_passphrase() {
        let root = tempfile::tempdir().unwrap();
        let saved = create(root.path());
        let plan = prepare_file(root.path(), all(), Some("fixture-export-passphrase")).unwrap();
        assert!(plan.protected);
        let metadata = crate::modules::backup::safety::read_file(
            &export_plan_path(root.path(), &plan.id).unwrap(),
            64 * 1024 * 1024,
        )
        .unwrap();
        assert!(!String::from_utf8_lossy(&metadata).contains("fixture-export-passphrase"));
        let bytes = file_bytes(root.path(), &plan.id).unwrap();
        let opened =
            crate::modules::sync::portable_file::open(&bytes, Some("fixture-export-passphrase"))
                .unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&opened.document).unwrap();
        let package: ModelPackage = serde_json::from_value(payload["models"].clone()).unwrap();
        assert_eq!(package.digest, plan.package.digest);
        assert!(package
            .payload
            .templates
            .iter()
            .any(|t| t.id == saved.template_id));
        let store = ModelStore::new(root.path());
        let current = store.load().unwrap();
        let family = current
            .library
            .templates
            .iter()
            .find(|t| t.id == saved.template_id)
            .unwrap();
        let mut content = family.revisions[0].content.clone();
        content.description = "冻结后的人工保存".into();
        store
            .save(SaveTemplate {
                expected_library: current.library_digest,
                template_id: Some(family.id.clone()),
                base_revision: Some(family.revisions[0].id.clone()),
                label: family.label.clone(),
                content,
            })
            .unwrap();
        assert_eq!(file_bytes(root.path(), &plan.id).unwrap(), bytes);
        crate::infrastructure::atomic_write::atomic_write(
            &export_plan_path(root.path(), &plan.id)
                .unwrap()
                .with_extension("ocxdconf"),
            b"changed",
            0o600,
        )
        .unwrap();
        assert_eq!(
            file_bytes(root.path(), &plan.id).unwrap_err().code,
            "export_digest"
        );
    }
    #[test]
    fn importing_versions_is_reviewed_and_preserves_local_defaults_and_runtime() {
        let source = tempfile::tempdir().unwrap();
        let saved = create(source.path());
        let package = export(source.path(), all()).unwrap();
        let target = tempfile::tempdir().unwrap();
        let incoming = receive(target.path(), package).unwrap();
        assert!(!target.path().join(LIBRARY_PATH).exists());
        let accepted = accept(target.path(), &incoming.id).unwrap();
        let template = accepted
            .library
            .templates
            .iter()
            .find(|t| t.id == saved.template_id)
            .unwrap();
        assert!(template.default_revision.is_none());
        assert_eq!(template.revisions[0].id, saved.revision_id);
        assert!(!target.path().join(POLICY_PATH).exists());
        let duplicate = receive(target.path(), export(source.path(), all()).unwrap()).unwrap();
        let after = accept(target.path(), &duplicate.id).unwrap();
        assert_eq!(after.library, accepted.library);
    }
    #[test]
    fn mismatched_identity_or_digest_is_rejected_without_modifying_the_library() {
        let root = tempfile::tempdir().unwrap();
        create(root.path());
        let before = ModelStore::new(root.path()).load().unwrap();
        let mut package = export(root.path(), all()).unwrap();
        package.payload.templates.last_mut().unwrap().revisions[0]
            .content
            .description = "changed".into();
        assert_eq!(
            receive(root.path(), package.clone()).unwrap_err().code,
            "package_digest"
        );
        let revision = &mut package.payload.templates.last_mut().unwrap().revisions[0];
        revision.digest =
            super::super::store::content_digest(&revision.saved_label, &revision.content).unwrap();
        package.digest = sha256(&serde_json::to_vec(&package.payload).unwrap());
        let incoming = receive(root.path(), package).unwrap();
        assert_eq!(incoming.preview_error.as_deref(), Some("revision_conflict"));
        assert_eq!(
            accept(root.path(), &incoming.id).unwrap_err().code,
            "revision_conflict"
        );
        assert_eq!(ModelStore::new(root.path()).load().unwrap(), before);
    }
    #[test]
    fn policy_package_carries_exact_dependencies_but_never_local_node_or_keychain_references() {
        let source = tempfile::tempdir().unwrap();
        let saved = create(source.path());
        let store = ModelStore::new(source.path());
        let snapshot = store.load().unwrap();
        let policy = Policy {
            template_refs: vec![TemplateRef {
                template_id: saved.template_id.clone(),
                revision_id: saved.revision_id.clone(),
            }],
            channels: vec![],
            ..Default::default()
        };
        let staged = store
            .stage_policy(
                &snapshot.policy_digest,
                &snapshot.workspace_digest,
                policy.clone(),
            )
            .unwrap();
        store
            .commit_runtime_policy(
                &staged.library_digest,
                &staged.policy_digest,
                &staged.workspace_digest,
                policy,
            )
            .unwrap();
        let package = export(
            source.path(),
            ExportScope {
                include_policy: true,
                channel_ids: vec![],
                template_refs: vec![],
                include_global: true,
            },
        )
        .unwrap();
        assert_eq!(package.payload.templates.len(), 1);
        assert_eq!(
            package.payload.templates[0].revisions[0].id,
            saved.revision_id
        );
        let bytes = serde_json::to_string(&package).unwrap();
        assert!(!bytes.contains("credentialRef"));
        assert!(!bytes.contains("activeNodeId"));
        let target = tempfile::tempdir().unwrap();
        let incoming = receive(target.path(), package).unwrap();
        let accepted = accept(target.path(), &incoming.id).unwrap();
        assert_eq!(
            accepted.workspace.unwrap().policy.template_refs[0].revision_id,
            saved.revision_id
        );
        assert!(!target.path().join(POLICY_PATH).exists());
    }
    #[test]
    fn a_changed_local_baseline_invalidates_old_import_confirmation() {
        let source = tempfile::tempdir().unwrap();
        create(source.path());
        let package = export(source.path(), all()).unwrap();
        let target = tempfile::tempdir().unwrap();
        let incoming = receive(target.path(), package).unwrap();
        create(target.path());
        let current = ModelStore::new(target.path()).load().unwrap();
        assert_eq!(
            accept(target.path(), &incoming.id).unwrap_err().code,
            "stale_baseline"
        );
        assert_eq!(ModelStore::new(target.path()).load().unwrap(), current);
    }
    #[test]
    fn receive_subset_keeps_other_objects_and_adds_exact_channel_dependencies() {
        let source = tempfile::tempdir().unwrap();
        let saved = create(source.path());
        let mut package = export(source.path(), all()).unwrap();
        let channel = PortableChannel {
            id: uuid::Uuid::new_v4().to_string(),
            label: "依赖渠道".into(),
            protocol: "openai-chat".into(),
            selection: ModelSelection::None,
            model_order: vec![],
            discovered: vec![],
            template_refs: vec![TemplateRef {
                template_id: saved.template_id.clone(),
                revision_id: saved.revision_id.clone(),
            }],
            model_refs: BTreeMap::new(),
            defaults: ModelRules::default(),
            overrides: BTreeMap::new(),
            family_overrides: BTreeMap::new(),
        };
        package.payload.policy = Some(PortablePolicy {
            template_refs: None,
            channel_order: None,
            channels: vec![channel.clone()],
        });
        package.digest = sha256(&serde_json::to_vec(&package.payload).unwrap());
        let target = tempfile::tempdir().unwrap();
        let before = ModelStore::new(target.path()).load().unwrap();
        let incoming = receive(target.path(), package).unwrap();
        let selected = select_received(
            target.path(),
            &incoming.id,
            ReceiveScope {
                template_refs: vec![],
                channel_ids: vec![channel.id],
                include_global: false,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(selected.package.payload.templates.len(), 1);
        assert_eq!(selected.package.payload.templates[0].id, saved.template_id);
        let after = accept(target.path(), &selected.id).unwrap();
        assert_eq!(
            after.library.templates.len(),
            before.library.templates.len() + 1
        );
        assert_eq!(after.policy, before.policy);
        let draft = after.workspace.unwrap();
        assert_eq!(draft.policy.channels.len(), 1);
        assert!(!draft.policy.channels[0].connection.enabled);
        assert!(draft.policy.channels[0].connection.nodes.is_empty());
    }
    #[test]
    fn unrelated_conflicting_revision_can_be_skipped_but_cannot_be_accepted_or_silently_rebased() {
        let source = tempfile::tempdir().unwrap();
        let saved = create(source.path());
        let mut package = export(source.path(), all()).unwrap();
        let target = tempfile::tempdir().unwrap();
        let local = ModelStore::new(target.path()).load().unwrap();
        let conflict = &mut package.payload.templates[0].revisions[0];
        conflict.content.description = "冲突同身份".into();
        conflict.digest =
            super::super::store::content_digest(&conflict.saved_label, &conflict.content).unwrap();
        package.digest = sha256(&serde_json::to_vec(&package.payload).unwrap());
        let incoming = receive(target.path(), package).unwrap();
        assert!(incoming.preview_error.is_some());
        assert!(accept(target.path(), &incoming.id).is_err());
        assert_eq!(ModelStore::new(target.path()).load().unwrap(), local);
        let selected = select_received(
            target.path(),
            &incoming.id,
            ReceiveScope {
                template_refs: vec![TemplateRef {
                    template_id: saved.template_id,
                    revision_id: saved.revision_id,
                }],
                channel_ids: vec![],
                include_global: false,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(selected.preview_error.is_none());
        let after = accept(target.path(), &selected.id).unwrap();
        assert_eq!(
            after.library.templates.len(),
            local.library.templates.len() + 1
        );
        assert_eq!(
            select_received(
                target.path(),
                &incoming.id,
                ReceiveScope {
                    template_refs: vec![],
                    channel_ids: vec![],
                    include_global: true,
                    ..Default::default()
                }
            )
            .unwrap_err()
            .code,
            "stale_baseline"
        );
    }
}
