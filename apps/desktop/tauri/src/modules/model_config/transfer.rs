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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection: Option<PortableConnection>,
    pub selection: ModelSelection,
    pub model_order: Vec<String>,
    pub discovered: Vec<DiscoveredModel>,
    pub template_refs: Vec<TemplateRef>,
    pub model_refs: BTreeMap<String, TemplateRef>,
    pub defaults: ModelRules,
    pub overrides: BTreeMap<String, ModelRules>,
    pub family_overrides: BTreeMap<String, ModelRules>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortableConnection {
    pub nodes: Vec<ServiceNode>,
}
fn portable_connection(channel: &ChannelPolicy) -> Option<PortableConnection> {
    (!channel.connection.nodes.is_empty()).then(|| PortableConnection {
        nodes: channel.connection.nodes.clone(),
    })
}
fn package_version(payload: &TransferPayload) -> u32 {
    if payload.policy.as_ref().is_some_and(|p| {
        p.default_channel_id.is_some() || p.channels.iter().any(|c| c.connection.is_some())
    }) {
        2
    } else {
        1
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortablePolicy {
    pub template_refs: Option<Vec<TemplateRef>>,
    pub channel_order: Option<Vec<String>>,
    pub channels: Vec<PortableChannel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_channel_id: Option<String>,
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
    #[serde(default)]
    pub account_ids: Vec<String>,
    #[serde(default)]
    pub skill_names: Vec<String>,
    #[serde(default)]
    pub mcp_names: Vec<String>,
    #[serde(default)]
    pub channel_mappings: BTreeMap<String, ChannelMapping>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(
    tag = "mode",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ChannelMapping {
    New { label: String },
    Local { target_id: String },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelBinding {
    pub target_id: String,
    pub keep_local_connection: bool,
    pub label: Option<String>,
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
    #[serde(default)]
    pub assets: Option<crate::modules::sync::portable_assets::Summary>,
    #[serde(default)]
    pub available_extension_credentials: bool,
    #[serde(default)]
    pub protected: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub channel_bindings: BTreeMap<String, ChannelBinding>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportAdoption {
    pub keys: bool,
    pub webdav: bool,
    pub completed: bool,
    #[serde(default)]
    pub extension_credentials: bool,
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
    #[serde(default)]
    pub assets: Option<crate::modules::sync::portable_assets::Summary>,
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
        && (manager_scope.preferences
            || manager_scope.extensions
            || manager_scope.webdav
            || !manager_scope.skill_names.is_empty()
            || !manager_scope.mcp_names.is_empty()
            || !manager_scope.account_ids.is_empty());
    let mut package = export(root, scope)?;
    if no_models {
        package.payload.templates.clear();
        package.digest = sha256(&serde_json::to_vec(&package.payload)?);
    }
    let (bytes, credential_count, manager, assets) = super::file_objects::seal_manager(
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
        assets: Some(assets),
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
        connection: portable_connection(channel),
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
            default_channel_id: if scope.include_global {
                snapshot.policy.default_channel_id.clone()
            } else {
                None
            },
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
        if let Some(id) = &policy.default_channel_id {
            require_uuid(id)?;
            // A global-only package may refer to an existing target channel.
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
        format_version: package_version(&payload),
        schema_version: SCHEMA_VERSION,
        created_at: chrono::Utc::now().to_rfc3339(),
        digest,
        payload,
    })
}
pub(super) fn validate_package(package: &ModelPackage) -> ModelResult<()> {
    if package.format != "OCXD_MODEL_CONFIG"
        || ![1, 2].contains(&package.format_version)
        || (package.format_version == 1 && package_version(&package.payload) != 1)
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
        if let Some(id) = &policy.default_channel_id {
            require_uuid(id)?;
        }
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
            if let Some(connection) = &channel.connection {
                let probe = ChannelPolicy {
                    id: channel.id.clone(),
                    label: channel.label.clone(),
                    template_refs: vec![],
                    model_refs: BTreeMap::new(),
                    connection: ChannelConnection {
                        auth_mode: None,
                        source_read_id: None,
                        provider_id: "portable-validation".into(),
                        protocol: channel.protocol.clone(),
                        nodes: connection.nodes.clone(),
                        active_node_id: connection
                            .nodes
                            .first()
                            .map(|n| n.id.clone())
                            .unwrap_or_default(),
                        ..Default::default()
                    },
                    selection: ModelSelection::None,
                    model_order: vec![],
                    discovered: vec![],
                    defaults: Default::default(),
                    overrides: Default::default(),
                    family_overrides: Default::default(),
                };
                super::compiler::validate_channel(&probe)?;
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
        if let Some(default) = &incoming.default_channel_id {
            policy.default_channel_id = Some(default.clone());
        }
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
            if (working.is_some() || committed.is_some()) && connection.protocol != channel.protocol
            {
                return Err(invalid(
                    "channel_endpoint_conflict",
                    "同一渠道的协议不同，请先映射或跳过；未复用本机Key",
                ));
            }
            if let Some(incoming_connection) = &channel.connection {
                if working.is_some() || committed.is_some() {
                    if connection.protocol != channel.protocol
                        || connection.nodes != incoming_connection.nodes
                    {
                        return Err(invalid("channel_endpoint_conflict", "同一渠道的协议或服务节点不同，请先映射渠道或跳过该对象；未改变本机连接或Key"));
                    }
                } else {
                    connection.provider_id = format!("ocxd_import_{}", channel.id.replace('-', ""));
                    connection.nodes = incoming_connection.nodes.clone();
                    connection.active_node_id = connection.nodes[0].id.clone();
                    // Staged candidates never inherit the source's enabled state,
                    // active runtime provider, Keychain reference or private-network permission.
                    connection.enabled = false;
                    connection.allow_private_network = false;
                    connection.credential_ref = None;
                }
            }
            connection.protocol = channel.protocol.clone();
            if working.is_none() {
                notices.push(format!(
                    "{}：新增渠道草稿，服务节点需确认，保持停用且未绑定运行渠道",
                    channel.label
                ));
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
        if policy
            .default_channel_id
            .as_ref()
            .is_some_and(|id| !policy.channels.iter().any(|c| &c.id == id))
        {
            return Err(invalid(
                "default_provider",
                "所选默认渠道未包含且本机没有对应绑定，请选择该渠道或跳过全局字段",
            ));
        }
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
            if incoming.default_channel_id.is_some() {
                existing.default_channel_id = incoming.default_channel_id;
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
        format_version: package_version(&payload),
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
    receive_bound(root, package, BTreeMap::new())
}
fn receive_bound(
    root: &Path,
    package: ModelPackage,
    bindings: BTreeMap<String, ChannelBinding>,
) -> ModelResult<IncomingPlan> {
    validate_package(&package)?;
    let local = ModelStore::new(root).load()?;
    let effective = mapped_package(&local, &package, &bindings)?;
    let (merged_library, merged_policy, mut notices, preview_error) =
        match merge(&local, &effective) {
            Ok((library, policy, notices)) => (library, policy, notices, None),
            Err(error) => (
                local.library.clone(),
                None,
                vec![error.message],
                Some(error.code),
            ),
        };
    for (source, binding) in &bindings {
        notices.push(if binding.keep_local_connection {
            format!("渠道 {source} → {}：采用模型规则，保留本机协议、节点、当前节点和Key；不采用来源Key", binding.target_id)
        } else {
            format!("渠道 {source} → {}：建立独立停用草稿；模版中原有渠道匹配范围保持，保存前请检查匹配", binding.target_id)
        });
    }
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
        assets: None,
        available_extension_credentials: false,
        protected: false,
        channel_bindings: bindings,
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
fn mapped_package(
    local: &Snapshot,
    source: &ModelPackage,
    bindings: &BTreeMap<String, ChannelBinding>,
) -> ModelResult<ModelPackage> {
    let mut result = source.clone();
    let mut targets = BTreeSet::new();
    let local_policy = local
        .workspace
        .as_ref()
        .map(|w| &w.policy)
        .unwrap_or(&local.policy);
    for (id, binding) in bindings {
        require_uuid(id)?;
        require_uuid(&binding.target_id)?;
        let index = source
            .payload
            .policy
            .as_ref()
            .and_then(|p| p.channels.iter().position(|c| &c.id == id))
            .ok_or_else(|| invalid("channel_mapping", "映射的渠道不在所选范围"))?;
        let channel = &mut result.payload.policy.as_mut().unwrap().channels[index];
        if binding.keep_local_connection {
            let target = local_policy
                .channels
                .iter()
                .find(|c| c.id == binding.target_id)
                .ok_or_else(|| invalid("channel_mapping", "映射的本机渠道已不存在"))?;
            if binding.label.is_some() {
                return Err(invalid("channel_mapping", "保留本机连接不能重命名渠道"));
            }
            channel.label = target.label.clone();
            channel.protocol = target.connection.protocol.clone();
            channel.connection = portable_connection(target);
        } else {
            if local_policy
                .channels
                .iter()
                .any(|c| c.id == binding.target_id)
                || source
                    .payload
                    .policy
                    .iter()
                    .flat_map(|p| &p.channels)
                    .any(|c| c.id == binding.target_id)
                || channel.connection.is_none()
                || binding.label.as_ref().is_none_or(|s| {
                    s.trim().is_empty() || s.len() > 256 || s.chars().any(char::is_control)
                })
            {
                return Err(invalid(
                    "channel_mapping",
                    "新渠道须有独立身份、名称及来源节点",
                ));
            }
            channel.label = binding.label.as_ref().unwrap().clone();
        }
        channel.id = binding.target_id.clone();
    }
    for channel in result.payload.policy.iter().flat_map(|p| &p.channels) {
        if !targets.insert(&channel.id) {
            return Err(invalid("channel_mapping", "多个来源渠道不能映射到同一目标"));
        }
    }
    if let Some(order) = result
        .payload
        .policy
        .as_mut()
        .and_then(|p| p.channel_order.as_mut())
    {
        for id in order {
            if let Some(binding) = bindings.get(id) {
                *id = binding.target_id.clone();
            }
        }
    }
    if let Some(id) = result
        .payload
        .policy
        .as_mut()
        .and_then(|p| p.default_channel_id.as_mut())
    {
        if let Some(binding) = bindings.get(id) {
            *id = binding.target_id.clone();
        }
    }
    result.format_version = package_version(&result.payload);
    result.digest = sha256(&serde_json::to_vec(&result.payload)?);
    validate_package(&result)?;
    Ok(result)
}
pub(super) fn key_target<'a>(plan: &'a IncomingPlan, source: &'a str) -> Option<&'a str> {
    match plan.channel_bindings.get(source) {
        Some(binding) if binding.keep_local_connection => None,
        Some(binding) => Some(&binding.target_id),
        None => Some(source),
    }
}
pub fn select_received(root: &Path, id: &str, scope: ReceiveScope) -> ModelResult<IncomingPlan> {
    select_received_context(root, id, scope, None)
}
pub fn select_received_context(
    root: &Path,
    id: &str,
    scope: ReceiveScope,
    passphrase: Option<&str>,
) -> ModelResult<IncomingPlan> {
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
        && scope.account_ids.is_empty()
        && scope.skill_names.is_empty()
        && scope.mcp_names.is_empty()
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
            policy.default_channel_id = None;
        }
        for reference in revision_refs(policy) {
            wanted.insert((reference.template_id.clone(), reference.revision_id.clone()));
        }
        if policy.channels.is_empty()
            && policy.template_refs.is_none()
            && policy.channel_order.is_none()
            && policy.default_channel_id.is_none()
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
    let mut bindings = BTreeMap::new();
    for (source, action) in scope.channel_mappings {
        if !scope.channel_ids.contains(&source) {
            return Err(invalid("channel_mapping", "映射仅适用于已选择渠道"));
        }
        let binding = match action {
            ChannelMapping::Local { target_id } => ChannelBinding {
                target_id,
                keep_local_connection: true,
                label: None,
            },
            ChannelMapping::New { label } => ChannelBinding {
                target_id: original
                    .channel_bindings
                    .get(&source)
                    .filter(|b| !b.keep_local_connection && b.label.as_ref() == Some(&label))
                    .map(|b| b.target_id.clone())
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                keep_local_connection: false,
                label: Some(label),
            },
        };
        bindings.insert(source, binding);
    }
    let mut selected = receive_bound(root, package, bindings)?;
    if !scope.account_ids.is_empty() && original.manager.is_none() {
        return Err(invalid("import_scope", "选择了包内不存在的账户信息"));
    }
    if let Some(manager) = &original.manager {
        if scope
            .account_ids
            .iter()
            .any(|id| !manager.accounts.iter().any(|a| &a.id == id))
            || scope.preferences && manager.preferences.is_none()
            || scope.extensions && manager.extensions.is_none()
            || scope
                .endpoint_ids
                .iter()
                .any(|id| !manager.webdav.iter().any(|e| &e.id == id))
        {
            return Err(invalid("import_scope", "选择了包内不存在的管理器配置"));
        }
        selected.manager = Some(crate::modules::sync::portable_manager::Settings {
            accounts: manager
                .accounts
                .iter()
                .filter(|a| scope.account_ids.contains(&a.id))
                .cloned()
                .collect(),
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
            &scope.account_ids,
        )?;
    }
    super::file_objects::copy_source(root, &original, &mut selected)?;
    super::file_objects::select_asset_subset(
        root,
        &original,
        &mut selected,
        &scope.skill_names,
        &scope.mcp_names,
        passphrase,
    )?;
    write_incoming(root, &selected)?;
    super::file_objects::remember_incoming(root, &selected.id)?;
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
fn intent_digest(
    plan: &IncomingPlan,
    credentials: &BTreeMap<String, String>,
) -> ModelResult<String> {
    let bytes = if plan.channel_bindings.is_empty() {
        serde_json::to_vec(credentials)?
    } else {
        serde_json::to_vec(&(credentials, &plan.channel_bindings))?
    };
    Ok(sha256(&bytes))
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
        || receipt.intent_digest != intent_digest(plan, credentials)?
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
        if plan.package.payload.policy.as_ref().is_none_or(|p| {
            !p.channels
                .iter()
                .any(|c| key_target(&plan, &c.id) == Some(channel.as_str()))
        }) {
            return Err(invalid("credential_scope", "凭据目标不在已选接收渠道内"));
        }
    }
    let intent_digest = intent_digest(&plan, credentials)?;
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
        let effective = mapped_package(&local, &plan.package, &plan.channel_bindings)?;
        let (library, mut policy, _) = merge(&local, &effective)?;
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
    #[test]
    fn explicit_channel_mapping_preserves_local_connection_or_creates_a_separate_draft() {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let original = channel_fixture(source.path());
        let mut local = channel_fixture(target.path());
        local.id = original.id.clone();
        local.connection.nodes[0].url = "https://local.example.test/v1".into();
        local.connection.protocol = "openai-responses".into();
        commit_fixture(target.path(), local.clone());
        let incoming =
            receive(target.path(), channel_package(source.path(), &original.id)).unwrap();
        assert_eq!(
            incoming.preview_error.as_deref(),
            Some("channel_endpoint_conflict")
        );
        let mapped = select_received(
            target.path(),
            &incoming.id,
            ReceiveScope {
                channel_ids: vec![original.id.clone()],
                channel_mappings: BTreeMap::from([(
                    original.id.clone(),
                    ChannelMapping::Local {
                        target_id: local.id.clone(),
                    },
                )]),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(mapped.preview_error.is_none());
        assert_eq!(key_target(&mapped, &original.id), None);
        let result = accept(target.path(), &mapped.id).unwrap();
        let draft = &result.workspace.as_ref().unwrap().policy.channels[0];
        assert_eq!(draft.connection, local.connection);
        assert_eq!(draft.selection, original.selection);
        // A new comparison uses the current baseline, never silently rewrites an old confirmed plan.
        let incoming =
            receive(target.path(), channel_package(source.path(), &original.id)).unwrap();
        let mapped = select_received(
            target.path(),
            &incoming.id,
            ReceiveScope {
                channel_ids: vec![original.id.clone()],
                channel_mappings: BTreeMap::from([(
                    original.id.clone(),
                    ChannelMapping::New {
                        label: "独立副本".into(),
                    },
                )]),
                ..Default::default()
            },
        )
        .unwrap();
        let target_id = mapped.channel_bindings[&original.id].target_id.clone();
        assert_ne!(target_id, local.id);
        assert_eq!(key_target(&mapped, &original.id), Some(target_id.as_str()));
        let result = accept(target.path(), &mapped.id).unwrap();
        let channels = &result.workspace.as_ref().unwrap().policy.channels;
        assert_eq!(channels.len(), 2);
        assert_eq!(channels[0].connection, local.connection);
        let new = channels.iter().find(|c| c.id == target_id).unwrap();
        assert!(!new.connection.enabled);
        assert_eq!(new.label, "独立副本");
    }
    #[test]
    fn channel_mapping_rejects_unselected_sources_duplicate_targets_and_drift() {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let original = channel_fixture(source.path());
        let local = channel_fixture(target.path());
        let mut package = channel_package(source.path(), &original.id);
        let mut second = package.payload.policy.as_ref().unwrap().channels[0].clone();
        second.id = uuid::Uuid::new_v4().to_string();
        package
            .payload
            .policy
            .as_mut()
            .unwrap()
            .channels
            .push(second.clone());
        package.digest = sha256(&serde_json::to_vec(&package.payload).unwrap());
        let incoming = receive(target.path(), package).unwrap();
        let mappings = BTreeMap::from([
            (
                original.id.clone(),
                ChannelMapping::Local {
                    target_id: local.id.clone(),
                },
            ),
            (
                second.id.clone(),
                ChannelMapping::Local {
                    target_id: local.id.clone(),
                },
            ),
        ]);
        assert_eq!(
            select_received(
                target.path(),
                &incoming.id,
                ReceiveScope {
                    channel_ids: vec![original.id.clone()],
                    channel_mappings: mappings.clone(),
                    ..Default::default()
                }
            )
            .unwrap_err()
            .code,
            "channel_mapping"
        );
        assert_eq!(
            select_received(
                target.path(),
                &incoming.id,
                ReceiveScope {
                    channel_ids: vec![original.id.clone(), second.id],
                    channel_mappings: mappings,
                    ..Default::default()
                }
            )
            .unwrap_err()
            .code,
            "channel_mapping"
        );
        let mapped = select_received(
            target.path(),
            &incoming.id,
            ReceiveScope {
                channel_ids: vec![original.id.clone()],
                channel_mappings: BTreeMap::from([(
                    original.id,
                    ChannelMapping::Local {
                        target_id: local.id.clone(),
                    },
                )]),
                ..Default::default()
            },
        )
        .unwrap();
        let mut edited = local;
        edited.label = "后来修改".into();
        commit_fixture(target.path(), edited.clone());
        assert_eq!(
            accept(target.path(), &mapped.id).unwrap_err().code,
            "stale_baseline"
        );
        assert_eq!(
            ModelStore::new(target.path())
                .load()
                .unwrap()
                .policy
                .channels[0],
            edited
        );
    }
    fn channel_fixture(root: &Path) -> ChannelPolicy {
        let nodes = vec![
            ServiceNode {
                id: uuid::Uuid::new_v4().to_string(),
                name: "主站".into(),
                url: "https://source.example.test/v1".into(),
            },
            ServiceNode {
                id: uuid::Uuid::new_v4().to_string(),
                name: "备用".into(),
                url: "https://backup.example.test/v1".into(),
            },
        ];
        let channel = ChannelPolicy {
            id: uuid::Uuid::new_v4().to_string(),
            label: "便携渠道".into(),
            template_refs: vec![],
            model_refs: BTreeMap::new(),
            connection: ChannelConnection {
                auth_mode: None,
                source_read_id: None,
                provider_id: "source-machine-provider".into(),
                protocol: "openai-chat".into(),
                active_node_id: nodes[1].id.clone(),
                nodes,
                credential_ref: Some(uuid::Uuid::new_v4().to_string()),
                enabled: true,
                allow_private_network: true,
            },
            selection: ModelSelection::Whitelist(vec!["model-a".into()]),
            model_order: vec![],
            discovered: vec![],
            defaults: Default::default(),
            overrides: Default::default(),
            family_overrides: Default::default(),
        };
        commit_fixture(root, channel.clone());
        channel
    }
    fn commit_fixture(root: &Path, channel: ChannelPolicy) {
        let store = ModelStore::new(root);
        let snapshot = store.load().unwrap();
        store
            .commit_runtime_policy(
                &snapshot.library_digest,
                &snapshot.policy_digest,
                &snapshot.workspace_digest,
                Policy {
                    channels: vec![channel],
                    ..Default::default()
                },
            )
            .unwrap();
    }
    fn channel_package(root: &Path, id: &str) -> ModelPackage {
        export(
            root,
            ExportScope {
                include_policy: true,
                channel_ids: vec![id.into()],
                template_refs: vec![],
                include_global: false,
            },
        )
        .unwrap()
    }
    #[test]
    fn portable_nodes_restore_as_disabled_draft_with_new_local_binding_and_no_runtime_or_secret_refs(
    ) {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let original = channel_fixture(source.path());
        let package = channel_package(source.path(), &original.id);
        assert_eq!(package.format_version, 2);
        let encoded = serde_json::to_string(&package).unwrap();
        assert!(!encoded.contains(&original.connection.provider_id));
        assert!(!encoded.contains(original.connection.credential_ref.as_deref().unwrap()));
        assert!(
            !encoded.contains("activeNodeId")
                && !encoded.contains("allowPrivateNetwork")
                && !encoded.contains("credentialRef")
        );
        let incoming = receive(target.path(), package).unwrap();
        assert!(incoming.preview_error.is_none());
        let adopted = accept(target.path(), &incoming.id).unwrap();
        assert!(adopted.policy.channels.is_empty());
        let draft = &adopted.workspace.as_ref().unwrap().policy.channels[0];
        assert_eq!(draft.connection.nodes, original.connection.nodes);
        assert_ne!(
            draft.connection.provider_id,
            original.connection.provider_id
        );
        assert!(!draft.connection.enabled && !draft.connection.allow_private_network);
        assert!(draft.connection.credential_ref.is_none());
        assert_ne!(
            draft.connection.active_node_id,
            original.connection.active_node_id
        );
        super::super::compiler::validate_channel(draft).unwrap();
    }
    #[test]
    fn same_identity_different_endpoint_is_review_conflict_and_same_endpoint_keeps_local_selection()
    {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let original = channel_fixture(source.path());
        let mut local = original.clone();
        local.connection.active_node_id = local.connection.nodes[0].id.clone();
        local.connection.provider_id = "local-provider".into();
        local.connection.credential_ref = Some(uuid::Uuid::new_v4().to_string());
        commit_fixture(target.path(), local.clone());
        let incoming =
            receive(target.path(), channel_package(source.path(), &original.id)).unwrap();
        let adopted = accept(target.path(), &incoming.id).unwrap();
        assert_eq!(
            adopted.workspace.as_ref().unwrap().policy.channels[0].connection,
            local.connection
        );
        let other = tempfile::tempdir().unwrap();
        local.connection.nodes[0].url = "https://other.example.test/v1".into();
        commit_fixture(other.path(), local.clone());
        let incoming = receive(other.path(), channel_package(source.path(), &original.id)).unwrap();
        assert_eq!(
            incoming.preview_error.as_deref(),
            Some("channel_endpoint_conflict")
        );
        assert_eq!(
            accept(other.path(), &incoming.id).unwrap_err().code,
            "channel_endpoint_conflict"
        );
        assert_eq!(
            ModelStore::new(other.path())
                .load()
                .unwrap()
                .policy
                .channels[0],
            local
        );
    }
    #[test]
    fn old_packages_keep_their_original_digest_and_new_nodes_cannot_masquerade_as_format_one() {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let channel = channel_fixture(source.path());
        let mut package = channel_package(source.path(), &channel.id);
        package.format_version = 1;
        assert_eq!(
            validate_package(&package).unwrap_err().code,
            "package_version"
        );
        package.payload.policy.as_mut().unwrap().channels[0].connection = None;
        package.digest = sha256(&serde_json::to_vec(&package.payload).unwrap());
        let bytes = serde_json::to_vec(&package).unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains("connection"));
        let parsed: ModelPackage = serde_json::from_slice(&bytes).unwrap();
        validate_package(&parsed).unwrap();
        let incoming = receive(target.path(), parsed).unwrap();
        assert!(incoming.preview_error.is_none());
        let result = accept(target.path(), &incoming.id).unwrap();
        assert!(result.workspace.unwrap().policy.channels[0]
            .connection
            .nodes
            .is_empty());
    }
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
            connection: None,
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
            default_channel_id: None,
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
