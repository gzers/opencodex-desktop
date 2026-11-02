//! Persistent per-device participation, common object document and the same
//! immutable-payload/conditional-head transport used by the model-only domain.
use super::{
    file_objects,
    remote::{self, Baseline},
    schema::*,
    store::{sha256, ModelStore},
    transfer::{self, ExportScope, IncomingPlan, ModelPackage},
};
use crate::{
    infrastructure::{
        atomic_write::atomic_write,
        webdav_client::{WebDavConfig, WebDavTransport},
    },
    modules::{
        backup::safety,
        sync::{
            portable_file,
            portable_manager::{Scope as ManagerScope, Settings},
        },
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
const SCOPE: &str = "manager-state/config-sync-scope.json";
const CATALOG: &str = "manager-state/config-sync-catalog.json";
const INITIALIZED: &str = "manager-state/config-sync-initialized.json";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CatalogObject {
    pub id: String,
    pub category: String,
    pub category_label: String,
    pub label: String,
    pub detail: String,
    pub mark: String,
    #[serde(default)]
    pub files: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CloudCatalog {
    pub binding: String,
    pub objects: Vec<CatalogObject>,
    pub credential_ids: Vec<String>,
    pub created_at: String,
    #[serde(default)]
    pub accounts: Vec<crate::modules::sync::portable_accounts::AccountInfo>,
}
pub fn cached_catalog(root: &Path, binding: &str) -> ModelResult<Option<CloudCatalog>> {
    let p = root.join(CATALOG);
    safety::check_path(root, &p, true)?;
    if !p.exists() {
        return Ok(None);
    }
    let c: CloudCatalog = serde_json::from_slice(&safety::read_file(&p, 16 * 1024 * 1024)?)?;
    if c.binding != binding {
        return Ok(None);
    }
    if c.objects.len() > 30000 {
        return Err(invalid("sync_catalog", "云端对象目录超过上限"));
    }
    Ok(Some(c))
}
fn has_cloud(catalog: &Option<CloudCatalog>, kind: &str, id: &str) -> bool {
    catalog.as_ref().is_some_and(|c| {
        c.objects.iter().any(|o| {
            o.category == kind && o.id == serde_json::to_string(&[kind, id]).unwrap_or_default()
        })
    })
}
pub async fn refresh_catalog<T: WebDavTransport>(
    root: &Path,
    transport: &T,
    config: &WebDavConfig,
    binding: &str,
    password: Option<&str>,
    previous: Option<&str>,
) -> ModelResult<Option<CloudCatalog>> {
    let path = root.join(CATALOG);
    safety::check_path(root, &path, true)?;
    let Some(head) = remote::pointer_for(remote::CONFIG, transport, config).await? else {
        if path.exists() {
            std::fs::remove_file(path)?
        }
        return Ok(None);
    };
    let bytes = zeroize::Zeroizing::new(
        remote::fetch_for(remote::CONFIG, transport, config, &head.pointer).await?,
    );
    let mut effective = password;
    let summary = match file_objects::remote_summary(&bytes, password) {
        Ok(s) => s,
        Err(e)
            if previous.is_some()
                && ["passphrase_required", "portable_authentication"]
                    .contains(&e.code.as_str()) =>
        {
            effective = previous;
            file_objects::remote_summary(&bytes, previous)?
        }
        Err(e) => return Err(e),
    };
    if remote::pointer_for(remote::CONFIG, transport, config)
        .await?
        .as_ref()
        .map(|p| &p.pointer)
        != Some(&head.pointer)
    {
        return Err(invalid(
            "remote_changed",
            "读取目录期间云端快照改变，请重新读取",
        ));
    }
    let mut objects = vec![];
    let mut push = |kind: &str, group: &str, id: &str, label: &str| {
        objects.push(CatalogObject {
            id: serde_json::to_string(&[kind, id]).unwrap(),
            category: kind.into(),
            category_label: group.into(),
            label: label.into(),
            detail: "云端已保存内容 · 本机尚无".into(),
            mark: "云端配置".into(),
            files: vec![],
        })
    };
    for t in &summary.package.payload.templates {
        push("templates", "模型模版", &t.id, &t.label)
    }
    for c in summary
        .package
        .payload
        .policy
        .iter()
        .flat_map(|p| &p.channels)
    {
        push("channels", "渠道与模型", &c.id, &c.label)
    }
    for a in &summary.manager.accounts {
        push("accounts", "账户", &a.id, &a.label)
    }
    for s in &summary.assets.skills {
        push("skills", "Skills", &s.name, &format!("Skill · {}", s.name));
    }
    for m in &summary.assets.mcps {
        push("mcps", "MCP", &m.name, &format!("MCP · {}", m.name))
    }
    for object in &mut objects {
        if object.category == "skills" {
            let id: Vec<String> = serde_json::from_str(&object.id)?;
            if let Some(skill) = summary.assets.skills.iter().find(|s| s.name == id[1]) {
                object.files = skill.file_entries.iter().map(|f| f.path.clone()).collect();
            }
        }
    }
    let c = CloudCatalog {
        binding: binding.into(),
        objects,
        credential_ids: file_objects::credential_ids(&bytes, effective)?,
        created_at: summary.package.created_at,
        accounts: summary.manager.accounts,
    };
    atomic_write(&path, &serde_json::to_vec(&c)?, 0o600)?;
    Ok(Some(c))
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Scope {
    #[serde(skip)]
    pub cloud_binding: Option<String>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub channel_families: std::collections::BTreeMap<String, transfer::FamilyScope>,
    pub template_ids: Vec<String>,
    pub channel_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub channel_models: std::collections::BTreeMap<String, Vec<String>>,
    pub include_global: bool,
    pub preferences: bool,
    pub extensions: bool,
    pub include_keys: bool,
    #[serde(default)]
    pub skill_names: Vec<String>,
    #[serde(default)]
    pub mcp_names: Vec<String>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub mcp_sources: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub extension_credentials: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_ids: Option<Vec<String>>,
    #[serde(default)]
    pub account_ids: Vec<String>,
    #[serde(default)]
    /// Recomputed from the bound local source by save_scope, never trusted from IPC.
    pub account_identities: Vec<crate::modules::sync::portable_accounts::AccountInfo>,
    #[serde(skip)]
    pub upstream_root: Option<PathBuf>,
    #[serde(skip)]
    pub upstream_snapshot: Option<super::upstream::UpstreamSnapshot>,
    #[serde(default, skip_serializing)]
    pub assets_reviewed: bool,
    #[serde(default)]
    pub all_templates: bool,
    #[serde(default)]
    pub all_channels: bool,
    #[serde(default)]
    pub all_skills: bool,
    #[serde(default)]
    pub all_mcps: bool,
    #[serde(skip)]
    pub local_home: Option<PathBuf>,
}
impl Default for Scope {
    fn default() -> Self {
        Self {
            cloud_binding: None,
            channel_families: Default::default(),
            template_ids: vec![],
            channel_ids: vec![],
            channel_models: Default::default(),
            include_global: false,
            preferences: true,
            extensions: true,
            include_keys: false,
            skill_names: vec![],
            mcp_names: vec![],
            mcp_sources: Default::default(),
            extension_credentials: false,
            credential_ids: None,
            account_ids: vec![],
            account_identities: vec![],
            upstream_root: None,
            upstream_snapshot: None,
            assets_reviewed: false,
            all_templates: false,
            all_channels: false,
            all_skills: false,
            all_mcps: false,
            local_home: None,
        }
    }
}
impl Scope {
    fn validate(&self) -> ModelResult<()> {
        if self.template_ids.len() > 5000
            || self.channel_ids.len() > 256
            || self.template_ids.iter().collect::<BTreeSet<_>>().len() != self.template_ids.len()
            || self.channel_ids.iter().collect::<BTreeSet<_>>().len() != self.channel_ids.len()
        {
            return Err(invalid("sync_scope", "本设备同步范围重复或超过上限"));
        }
        if self.channel_families.len() > 256
            || self.channel_families.iter().any(|(id, scope)| {
                (!self.all_channels && !self.channel_ids.contains(id))
                    || self.channel_models.contains_key(id)
                    || scope.template_ids.is_empty()
                    || scope.template_ids.len() > 5000
                    || scope.template_ids.iter().collect::<BTreeSet<_>>().len()
                        != scope.template_ids.len()
            })
        {
            return Err(invalid("sync_scope", "族片段范围无效或与型号子集重复"));
        }
        if self.channel_models.len() > 256
            || self.channel_models.iter().any(|(id, models)| {
                (!self.all_channels && !self.channel_ids.contains(id))
                    || models.len() > 10000
                    || models.iter().collect::<BTreeSet<_>>().len() != models.len()
                    || models.iter().any(|m| m.is_empty())
            })
        {
            return Err(invalid(
                "sync_scope",
                "型号子集不属于本设备渠道范围或格式无效",
            ));
        }
        for id in self.template_ids.iter().chain(&self.channel_ids) {
            require_uuid(id)?;
        }
        if self.account_ids.len() > 1000
            || self.account_ids.iter().collect::<BTreeSet<_>>().len() != self.account_ids.len()
        {
            return Err(invalid("sync_scope", "账户同步范围重复或超过上限"));
        }
        for id in &self.account_ids {
            require_uuid(id)?;
        }
        crate::modules::sync::portable_accounts::validate(&self.account_identities)?;
        if self.include_keys && self.channel_ids.is_empty() && !self.all_channels {
            return Err(invalid("sync_scope", "携带Key前请选择渠道"));
        }
        if self.template_ids.is_empty()
            && self.channel_ids.is_empty()
            && !self.include_global
            && !self.preferences
            && !self.extensions
            && self.account_ids.is_empty()
            && self.skill_names.is_empty()
            && self.mcp_names.is_empty()
            && !self.all_templates
            && !self.all_channels
            && !self.all_skills
            && !self.all_mcps
        {
            return Err(invalid("sync_scope", "请选择至少一个参与同步的类别"));
        }
        if self.skill_names.len() > 500
            || self.mcp_names.len() > 1000
            || self
                .skill_names
                .iter()
                .chain(&self.mcp_names)
                .any(|name| !crate::modules::extensions::discovery::is_safe_skill_name(name))
        {
            return Err(invalid("sync_scope", "扩展范围名称或数量无效"));
        }
        if self.mcp_sources.len() > 1000
            || self.mcp_sources.iter().any(|(name, client)| {
                !crate::modules::extensions::discovery::is_safe_skill_name(name)
                    || !crate::modules::extensions::CLIENT_IDS
                        .iter()
                        .any(|id| crate::modules::sync::portable_assets::client_key(*id) == *client)
            })
        {
            return Err(invalid("sync_scope", "MCP采用来源无效"));
        }
        Ok(())
    }
    fn manager(&self) -> ManagerScope {
        ManagerScope {
            preferences: self.preferences,
            extensions: self.extensions,
            webdav: false,
            webdav_passwords: false,
            skill_names: self.skill_names.clone(),
            mcp_names: self.mcp_names.clone(),
            mcp_sources: self.mcp_sources.clone(),
            extension_credentials: self.extension_credentials,
            credential_ids: self.credential_ids.clone(),
            assets_reviewed: self.assets_reviewed,
            account_ids: self.account_ids.clone(),
            upstream_root: self.upstream_root.clone(),
            upstream_snapshot: self.upstream_snapshot.clone(),
            local_home: self.local_home.clone(),
        }
    }
    fn export(&self, root: &Path) -> ModelResult<ExportScope> {
        let catalog = self
            .cloud_binding
            .as_deref()
            .map(|b| cached_catalog(root, b))
            .transpose()?
            .flatten();
        let snapshot = match self.upstream_root.as_deref() {
            Some(home) => super::config_file::snapshot(root, home)?,
            None => ModelStore::new(root).load()?,
        };
        let mut refs = vec![];
        let template_ids = if self.all_templates {
            snapshot
                .library
                .templates
                .iter()
                .map(|template| template.id.clone())
                .collect::<Vec<_>>()
        } else {
            self.template_ids.clone()
        };
        for id in &template_ids {
            let family = snapshot.library.templates.iter().find(|t| &t.id == id);
            let Some(family) = family else {
                if has_cloud(&catalog, "templates", id) {
                    continue;
                } else {
                    return Err(invalid("sync_scope", "范围中的模版已不存在，请重新选择"));
                }
            };
            refs.extend(family.revisions.iter().map(|r| TemplateRef {
                template_id: family.id.clone(),
                revision_id: r.id.clone(),
            }));
        }
        let channel_inventory = if self.upstream_root.is_some() {
            snapshot.policy.channels.clone()
        } else if let Some(upstream) = &self.upstream_snapshot {
            super::portable_channels::catalog(root, upstream)?
        } else {
            snapshot.policy.channels.clone()
        };
        let local_ids = channel_inventory
            .iter()
            .map(|c| c.id.clone())
            .collect::<BTreeSet<_>>();
        if self
            .channel_ids
            .iter()
            .any(|id| !local_ids.contains(id) && !has_cloud(&catalog, "channels", id))
        {
            return Err(invalid(
                "sync_scope",
                "范围中的渠道不在本机或当前云端目录中",
            ));
        }
        Ok(ExportScope {
            channel_families: self
                .channel_families
                .iter()
                .filter(|(id, _)| local_ids.contains(*id))
                .map(|(a, b)| (a.clone(), b.clone()))
                .collect(),
            channel_models: self
                .channel_models
                .iter()
                .filter(|(id, _)| local_ids.contains(*id))
                .map(|(a, b)| (a.clone(), b.clone()))
                .collect(),
            include_policy: self.include_global
                || !self.channel_ids.is_empty()
                || self.all_channels,
            channel_ids: if self.all_channels {
                channel_inventory
                    .iter()
                    .map(|channel| channel.id.clone())
                    .collect()
            } else {
                self.channel_ids
                    .iter()
                    .filter(|id| local_ids.contains(*id))
                    .cloned()
                    .collect()
            },
            template_refs: refs,
            include_global: self.include_global,
        })
    }
}
fn remote_account_ids(
    scope: &Scope,
    remote: &[crate::modules::sync::portable_accounts::AccountInfo],
) -> Vec<String> {
    remote
        .iter()
        .filter(|account| {
            crate::modules::sync::portable_accounts::information_supported(account)
                && scope.account_identities.iter().any(|local| {
                    crate::modules::sync::portable_accounts::same_identity(local, account)
                })
        })
        .map(|account| account.id.clone())
        .collect()
}

pub fn load_scope(root: &Path) -> ModelResult<Scope> {
    let path = root.join(SCOPE);
    safety::check_path(root, &path, true)?;
    let scope = if path.exists() {
        serde_json::from_slice(&safety::read_file(&path, 1024 * 1024)?)?
    } else {
        Scope::default()
    };
    Scope::validate(&scope)?;
    if scope.account_ids.iter().collect::<BTreeSet<_>>()
        != scope
            .account_identities
            .iter()
            .map(|a| &a.id)
            .collect::<BTreeSet<_>>()
    {
        return Err(invalid(
            "sync_scope",
            "账户同步范围缺少身份绑定，请重新保存范围",
        ));
    }
    Ok(scope)
}
pub fn save_scope(root: &Path, mut scope: Scope) -> ModelResult<Scope> {
    scope.validate()?;
    let catalog = scope
        .cloud_binding
        .as_deref()
        .map(|b| cached_catalog(root, b))
        .transpose()?
        .flatten();
    let cloud_accounts = catalog
        .as_ref()
        .map(|c| c.accounts.as_slice())
        .unwrap_or(&[]);
    let local_ids = scope
        .account_ids
        .iter()
        .filter(|id| !cloud_accounts.iter().any(|a| a.id == **id))
        .cloned()
        .collect::<Vec<_>>();
    scope.account_identities = crate::modules::sync::portable_accounts::export(
        root,
        scope.upstream_root.as_deref(),
        &local_ids,
    )?;
    scope.account_identities.extend(
        cloud_accounts
            .iter()
            .filter(|a| scope.account_ids.contains(&a.id))
            .cloned(),
    );
    let path = root.join(SCOPE);
    safety::check_path(root, &path, true)?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&path),
        true,
    )?;
    let _guard = crate::infrastructure::locking::TargetFileLock::lock(&path)?;
    let bytes = serde_json::to_vec(&scope)?;
    atomic_write(&path, &bytes, 0o600)?;
    if safety::read_file(&path, 1024 * 1024)? != bytes {
        return Err(invalid("sync_scope_readback", "范围保存后未通过读回"));
    }
    Ok(scope)
}
/// Initialize the device range exactly once, after a complete cloud catalog has
/// been fetched and validated. Later catalogs never widen the saved range.
pub fn initialize_remote_scope(
    root: &Path,
    binding: &str,
    catalog: &CloudCatalog,
    upstream_root: &Path,
    local_home: &Path,
) -> ModelResult<bool> {
    let marker = root.join(INITIALIZED);
    safety::check_path(root, &marker, true)?;
    let _guard = crate::infrastructure::locking::TargetFileLock::lock(&marker)?;
    if marker.exists() {
        return Ok(false);
    }
    let scope_path = root.join(SCOPE);
    safety::check_path(root, &scope_path, true)?;
    if scope_path.exists() {
        // A previously persisted selection is an explicit user choice. Mark
        // the device initialized without widening it during an upgrade.
        atomic_write(&marker, b"{\"initialized\":true}", 0o600)?;
        return Ok(false);
    }
    let mut scope = load_scope(root)?;
    scope.cloud_binding = Some(binding.into());
    scope.upstream_root = Some(upstream_root.into());
    scope.local_home = Some(local_home.into());
    for object in &catalog.objects {
        let parts: Vec<String> = serde_json::from_str(&object.id)?;
        if parts.len() != 2 {
            return Err(invalid("sync_catalog", "云端对象身份无效"));
        }
        match parts[0].as_str() {
            "templates" => scope.template_ids.push(parts[1].clone()),
            "channels" => scope.channel_ids.push(parts[1].clone()),
            "accounts" => scope.account_ids.push(parts[1].clone()),
            "skills" => scope.skill_names.push(parts[1].clone()),
            "mcps" => scope.mcp_names.push(parts[1].clone()),
            _ => return Err(invalid("sync_catalog", "云端对象类别无效")),
        }
    }
    scope.template_ids.sort();
    scope.template_ids.dedup();
    scope.channel_ids.sort();
    scope.channel_ids.dedup();
    scope.account_ids.sort();
    scope.account_ids.dedup();
    scope.skill_names.sort();
    scope.skill_names.dedup();
    scope.mcp_names.sort();
    scope.mcp_names.dedup();
    save_scope(root, scope)?;
    atomic_write(&marker, b"{\"initialized\":true}", 0o600)?;
    Ok(true)
}
fn scope_digest(scope: &Scope) -> ModelResult<String> {
    Ok(sha256(&serde_json::to_vec(scope)?))
}
fn local_manager(root: &Path, scope: &Scope) -> ModelResult<ManagerScope> {
    let mut m = scope.manager();
    if let Some(home) = scope.local_home.as_ref().filter(|_| {
        !scope.skill_names.is_empty()
            || !scope.mcp_names.is_empty()
            || scope.all_skills
            || scope.all_mcps
    }) {
        let catalog = crate::modules::sync::portable_assets::catalog(root, home)?;
        m.skill_names.retain(|n| catalog.skills.contains(n));
        m.mcp_names.retain(|n| catalog.mcps.contains(n));
    }
    if let Some(home) = &scope.upstream_root {
        let accounts = crate::modules::sync::portable_accounts::catalog(root, home)?;
        m.account_ids
            .retain(|id| accounts.iter().any(|a| a.account.id == *id));
    }
    Ok(m)
}
/// Fingerprint the currently saved selection. This never stages a business
/// import or writes an export file; temporary secret bytes are zeroized.
pub fn source_state(root: &Path, scope: &Scope) -> ModelResult<(String, String)> {
    source_state_with_time(root, scope, None)
}
pub fn source_state_with_time(
    root: &Path,
    scope: &Scope,
    inherited: Option<&str>,
) -> ModelResult<(String, String)> {
    scope.validate()?;
    let export = scope.export(root)?;
    let package = if let Some(home) = &scope.upstream_root {
        super::portable_channels::export_file(root, home, export)?
    } else {
        transfer::export(root, export)?
    };
    let mut manager = local_manager(root, scope)?;
    if scope.all_skills || scope.all_mcps {
        let c = crate::modules::sync::portable_assets::catalog(
            root,
            scope
                .local_home
                .as_deref()
                .ok_or_else(|| invalid("asset_home", "同步资源未绑定本机主目录"))?,
        )?;
        if scope.all_skills {
            manager.skill_names = c.skills
        }
        if scope.all_mcps {
            manager.mcp_names = c
                .mcps
                .into_iter()
                .filter(|n| {
                    crate::modules::sync::portable_assets::default_included_mcp(n)
                        || scope.mcp_names.contains(n)
                })
                .collect()
        }
    }
    let (bytes, _, _, _) = file_objects::seal_manager(
        root,
        &uuid::Uuid::new_v4().to_string(),
        &package,
        scope.include_keys
            && package
                .payload
                .policy
                .as_ref()
                .is_some_and(|p| !p.channels.is_empty()),
        None,
        &manager,
    )?;
    let bytes = zeroize::Zeroizing::new(bytes);
    if let Some(time) = inherited {
        file_objects::inherit_content_time(root, &bytes, None, time)?;
    }
    Ok((
        file_objects::content_digest(&bytes, None)?,
        scope_digest(scope)?,
    ))
}
pub async fn check_source<T: WebDavTransport>(
    root: &Path,
    transport: &T,
    config: &WebDavConfig,
    binding: &str,
    scope: &Scope,
) -> ModelResult<super::sync_state::Check> {
    let (local, key) = source_state(root, scope)?;
    super::sync_state::check_content(root, transport, config, binding, &local, &key).await
}
pub fn verify_upload_source(root: &Path, id: &str, scope: &Scope) -> ModelResult<()> {
    let p: Plan = serde_json::from_slice(&safety::read_file(&path(root, id)?, 64 * 1024 * 1024)?)?;
    if p.view.phase == "published" {
        return Ok(());
    }
    let (local, key) = source_state(root, scope)?;
    if p.local_content.as_deref() != Some(local.as_str()) || p.scope_digest != key {
        return Err(invalid(
            "sync_source_changed",
            "预览后已保存内容改变，未上传旧快照；请重新读取",
        ));
    }
    Ok(())
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UploadView {
    pub id: String,
    pub phase: String,
    pub package: ModelPackage,
    pub manager: Settings,
    pub credential_count: usize,
    #[serde(default)]
    pub credential_counts: Option<file_objects::CredentialCounts>,
    pub protected: bool,
    #[serde(default)]
    pub assets: crate::modules::sync::portable_assets::Summary,
    /// 设备间同步预览（S18-01）：新版来源、变化列表与保持不变的数量。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sync: Option<super::sync_state::SyncPreview>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Plan {
    view: UploadView,
    binding: String,
    scope_digest: String,
    baseline: Option<Baseline>,
    file_digest: String,
    #[serde(default)]
    local_content: Option<String>,
    #[serde(default)]
    local_plan: Option<transfer::FileExportPlan>,
}
fn path(root: &Path, id: &str) -> ModelResult<PathBuf> {
    require_uuid(id)?;
    let path = root
        .join("manager-state/config-uploads")
        .join(format!("{id}.json"));
    safety::check_path(root, &path, true)?;
    Ok(path)
}
fn persist(root: &Path, plan: &Plan) -> ModelResult<()> {
    atomic_write(
        &path(root, &plan.view.id)?,
        &serde_json::to_vec(plan)?,
        0o600,
    )?;
    Ok(())
}
pub fn latest_upload(root: &Path) -> ModelResult<Option<UploadView>> {
    let index = root.join("manager-state/config-latest-upload.json");
    safety::check_path(root, &index, true)?;
    if !index.exists() {
        return Ok(None);
    }
    let id: String = serde_json::from_slice(&safety::read_file(&index, 1024)?)?;
    let plan: Plan =
        serde_json::from_slice(&safety::read_file(&path(root, &id)?, 64 * 1024 * 1024)?)?;
    if plan.view.id != id {
        return Err(invalid("upload_identity", "原上传记录身份不同"));
    }
    Ok(Some(plan.view))
}
pub fn upload_result(root: &Path, id: &str) -> ModelResult<UploadView> {
    let p: Plan = serde_json::from_slice(&safety::read_file(&path(root, id)?, 64 * 1024 * 1024)?)?;
    if p.view.id != id {
        return Err(invalid("upload_identity", "上传记录身份不同"));
    }
    Ok(p.view)
}
pub fn upload_selection(root: &Path, id: &str) -> ModelResult<Option<transfer::FileExportPlan>> {
    let p: Plan = serde_json::from_slice(&safety::read_file(&path(root, id)?, 64 * 1024 * 1024)?)?;
    if p.view.id != id {
        return Err(invalid("upload_identity", "上传记录身份不同"));
    }
    Ok(p.local_plan)
}
pub async fn prepare<T: WebDavTransport, F: Fn() -> ModelResult<()>>(
    root: &Path,
    transport: &T,
    config: &WebDavConfig,
    binding: &str,
    scope: Scope,
    passphrase: Option<&str>,
    unchanged: F,
) -> ModelResult<UploadView> {
    prepare_with_protection(
        root, transport, config, binding, scope, passphrase, passphrase, None, unchanged,
    )
    .await
}
#[allow(clippy::too_many_arguments)]
pub async fn prepare_with_protection<T: WebDavTransport, F: Fn() -> ModelResult<()>>(
    root: &Path,
    transport: &T,
    config: &WebDavConfig,
    binding: &str,
    scope: Scope,
    write_password: Option<&str>,
    read_password: Option<&str>,
    previous_password: Option<&str>,
    unchanged: F,
) -> ModelResult<UploadView> {
    scope.validate()?;
    if scope_digest(&load_scope(root)?)? != scope_digest(&scope)? {
        return Err(invalid("sync_scope", "先明确保存本设备范围，再准备上传"));
    }
    let export = scope.export(root)?;
    let mut manager = local_manager(root, &scope)?;
    if scope.all_skills || scope.all_mcps {
        let catalog = crate::modules::sync::portable_assets::catalog(
            root,
            scope
                .local_home
                .as_deref()
                .ok_or_else(|| invalid("asset_home", "同步扩展前需绑定本机主目录"))?,
        )?;
        if scope.all_skills {
            manager.skill_names = catalog.skills;
        }
        if scope.all_mcps {
            // 「全部 MCP」只跳过默认不同步项（如 node_repl）；用户手动勾选的仍保留。
            manager.mcp_names = catalog
                .mcps
                .into_iter()
                .filter(|name| {
                    crate::modules::sync::portable_assets::default_included_mcp(name)
                        || scope.mcp_names.contains(name)
                })
                .collect();
        }
    }
    let include_keys = scope.include_keys && !export.channel_ids.is_empty();
    let local = transfer::prepare_file_scope(root, export, write_password, include_keys, &manager)?;
    let local_bytes = transfer::file_bytes(root, &local.id)?;
    let baseline = remote::pointer_for(remote::CONFIG, transport, config).await?;
    unchanged()?;
    let id = uuid::Uuid::new_v4().to_string();
    let remote_bytes = match &baseline {
        Some(base) => {
            Some(remote::fetch_for(remote::CONFIG, transport, config, &base.pointer).await?)
        }
        None => None,
    };
    let effective_read = if let Some(bytes) = &remote_bytes {
        match file_objects::remote_summary(bytes, read_password) {
            Ok(_) => read_password,
            Err(error)
                if previous_password.is_some()
                    && ["passphrase_required", "portable_authentication"]
                        .contains(&error.code.as_str()) =>
            {
                file_objects::remote_summary(bytes, previous_password)?;
                previous_password
            }
            Err(error) => return Err(error),
        }
    } else {
        read_password
    };
    let payload = match &remote_bytes {
        Some(rb) => file_objects::merge_remote_protection(
            rb,
            &local_bytes,
            effective_read,
            write_password,
            &id,
        )?,
        None => local_bytes.clone(),
    };
    let summary = file_objects::remote_summary(&payload, write_password)?;
    let mut assets = summary.assets.clone();
    assets.conflicts = local
        .assets
        .as_ref()
        .map(|local| local.conflicts.clone())
        .unwrap_or_default();
    let mut preview = upload_preview(
        &local,
        &summary,
        &assets.conflicts,
        remote_bytes.as_deref(),
        &payload,
        write_password,
        effective_read,
    );
    if let Some(bytes) = &remote_bytes {
        let remote = file_objects::remote_summary(bytes, effective_read)?;
        if remote.protected != summary.protected
            || summary.protected && effective_read != write_password
        {
            if let Some(changes) = &mut preview.changes {
                changes.push(super::sync_state::Change {
                    kind: "protection".into(),
                    id: "snapshot-protection".into(),
                    label: "同步内容保护".into(),
                    mark: "update".into(),
                    ..Default::default()
                });
            }
        }
    }
    let view = UploadView {
        id: id.clone(),
        phase: "prepared".into(),
        package: summary.package,
        manager: summary.manager,
        credential_count: summary.credential_count,
        credential_counts: Some(summary.credential_counts),
        protected: summary.protected,
        assets,
        sync: Some(preview),
    };
    let plan = Plan {
        view: view.clone(),
        binding: binding.into(),
        scope_digest: scope_digest(&scope)?,
        baseline,
        file_digest: sha256(&payload),
        local_content: Some(file_objects::content_digest(&local_bytes, write_password)?),
        local_plan: Some(local),
    };
    if scope_digest(&load_scope(root)?)? != plan.scope_digest {
        return Err(invalid(
            "sync_scope_changed",
            "准备期间本设备范围改变，未发布",
        ));
    }
    unchanged()?;
    let payload_path = path(root, &id)?.with_extension("ocxdconf");
    safety::check_path(root, &payload_path, true)?;
    atomic_write(&payload_path, &payload, 0o600)?;
    persist(root, &plan)?;
    let index = root.join("manager-state/config-latest-upload.json");
    safety::check_path(root, &index, true)?;
    atomic_write(&index, &serde_json::to_vec(&id)?, 0o600)?;
    Ok(view)
}
pub async fn publish<T: WebDavTransport, F: Fn() -> ModelResult<()>>(
    root: &Path,
    transport: &T,
    config: &WebDavConfig,
    id: &str,
    binding: &str,
    unchanged: F,
) -> ModelResult<UploadView> {
    let path = path(root, id)?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&path),
        true,
    )?;
    let _guard = crate::infrastructure::locking::TargetFileLock::lock(&path)?;
    let mut plan: Plan = serde_json::from_slice(&safety::read_file(&path, 64 * 1024 * 1024)?)?;
    if plan.view.id != id || plan.binding != binding {
        return Err(invalid("endpoint_changed", "原上传计划绑定已变化，未重写"));
    }
    if plan.view.sync.as_ref().is_some_and(|p| p.changes.is_none()) {
        return Err(invalid("sync_compare", "无法比较变化，未发布；请重新读取"));
    }
    if plan.view.phase == "prepared"
        && plan
            .view
            .sync
            .as_ref()
            .is_some_and(|p| p.changes.as_ref().is_some_and(Vec::is_empty))
    {
        return Err(invalid("sync_unchanged", "所选内容已一致，无需发布"));
    }
    let expected_scope = plan.scope_digest.clone();
    let current = || {
        unchanged()?;
        if scope_digest(&load_scope(root)?)? != expected_scope {
            return Err(invalid(
                "sync_scope_changed",
                "设备范围已变化，原上传不能继续",
            ));
        }
        Ok(())
    };
    current()?;
    let payload_path = path.with_extension("ocxdconf");
    safety::check_path(root, &payload_path, false)?;
    let bytes = safety::read_file(&payload_path, portable_file::FILE_LIMIT as u64)?;
    if sha256(&bytes) != plan.file_digest {
        return Err(invalid("upload_digest", "原冻结上传文件改变，未上传"));
    }
    let baseline = plan.baseline.clone();
    let published = plan.view.phase == "published";
    let pointer = remote::pointer_of(remote::CONFIG, id, &bytes);
    remote::publish_blob(
        remote::CONFIG,
        transport,
        config,
        id,
        baseline.as_ref(),
        bytes,
        published,
        |phase| {
            plan.view.phase = phase.into();
            persist(root, &plan)
        },
        current,
    )
    .await?;
    // 同步记录失败只影响下次方向判断（会按云端为新版重新比较），不回退已完成的发布。
    if let Some(local) = &plan.local_content {
        super::sync_state::record_synced_content(
            root,
            binding,
            &pointer,
            local,
            &plan.scope_digest,
        )?;
    } else {
        super::sync_state::record_synced(root, binding, &pointer)?;
    }
    Ok(plan.view)
}
pub async fn download<T: WebDavTransport, F: Fn() -> ModelResult<()>>(
    root: &Path,
    transport: &T,
    config: &WebDavConfig,
    passphrase: Option<&str>,
    unchanged: F,
) -> ModelResult<IncomingPlan> {
    download_context(root, transport, config, passphrase, None, unchanged).await
}
pub async fn download_context<T: WebDavTransport, F: Fn() -> ModelResult<()>>(
    root: &Path,
    transport: &T,
    config: &WebDavConfig,
    passphrase: Option<&str>,
    home: Option<&Path>,
    unchanged: F,
) -> ModelResult<IncomingPlan> {
    download_bound_context(root, transport, config, passphrase, home, None, unchanged).await
}
#[allow(clippy::too_many_arguments)]
pub async fn download_context_file<T: WebDavTransport, F: Fn() -> ModelResult<()>>(
    root: &Path,
    transport: &T,
    config: &WebDavConfig,
    passphrase: Option<&str>,
    local_home: Option<&Path>,
    config_home: &Path,
    unchanged: F,
) -> ModelResult<IncomingPlan> {
    download_bound_context(
        root,
        transport,
        config,
        passphrase,
        local_home,
        Some(config_home),
        unchanged,
    )
    .await
}
#[allow(clippy::too_many_arguments)]
async fn download_bound_context<T: WebDavTransport, F: Fn() -> ModelResult<()>>(
    root: &Path,
    transport: &T,
    config: &WebDavConfig,
    passphrase: Option<&str>,
    home: Option<&Path>,
    config_home: Option<&Path>,
    unchanged: F,
) -> ModelResult<IncomingPlan> {
    download_pointed(
        root,
        transport,
        config,
        passphrase,
        home,
        config_home,
        unchanged,
    )
    .await
    .map(|(plan, _)| plan)
}
/// 设备间同步下载（S18-01）：接收云端新版，并与本机当前内容比较生成同步预览。
#[allow(clippy::too_many_arguments)]
pub async fn download_sync<T: WebDavTransport, F: Fn() -> ModelResult<()>>(
    root: &Path,
    transport: &T,
    config: &WebDavConfig,
    passphrase: Option<&str>,
    local_home: Option<&Path>,
    config_home: &Path,
    binding: &str,
    unchanged: F,
) -> ModelResult<IncomingPlan> {
    let (mut plan, pointer) = download_pointed(
        root,
        transport,
        config,
        passphrase,
        local_home,
        Some(config_home),
        unchanged,
    )
    .await?;
    let changes = plan
        .comparison
        .as_ref()
        .and_then(|p| p.changes.clone().map(|changes| (changes, p.unchanged)));
    super::sync_state::record_download(
        root,
        binding,
        &pointer,
        &plan.id,
        changes.as_ref().map(|(changes, _)| changes.as_slice()),
    )?;
    if changes
        .as_ref()
        .is_some_and(|(changes, _)| changes.is_empty())
    {
        let mut scope = load_scope(root)?;
        scope.upstream_root = Some(config_home.into());
        scope.local_home = local_home.map(Path::to_path_buf);
        let (local, key) = source_state_with_time(root, &scope, Some(&plan.package.created_at))?;
        super::sync_state::record_synced_content(root, binding, &pointer, &local, &key)?;
    }
    plan.sync = Some(super::sync_state::SyncPreview {
        source: "remote".into(),
        unchanged: changes.as_ref().map(|(_, n)| *n).unwrap_or_default(),
        changes: changes.map(|(changes, _)| changes),
        created_at: plan.package.created_at.clone(),
    });
    if plan.sync.as_ref().is_some_and(|p| p.changes.is_none()) {
        plan.preview_error = Some("sync_compare".into());
    }
    transfer::write_incoming(root, &plan)?;
    Ok(plan)
}
/// 上传预览：只比较本机所选条目与当前云端内容；冲突 MCP 单独列为「选择来源」。
fn upload_preview(
    local: &transfer::FileExportPlan,
    merged: &file_objects::RemoteSummary,
    conflicts: &[crate::modules::sync::portable_assets::McpConflict],
    remote_bytes: Option<&[u8]>,
    payload: &[u8],
    passphrase: Option<&str>,
    remote_password: Option<&str>,
) -> super::sync_state::SyncPreview {
    let none = Default::default();
    let keep = BTreeSet::new();
    let compare = || -> ModelResult<(Vec<super::sync_state::Change>, usize)> {
        let local_facts = super::sync_state::facts(
            &local.package,
            &local.manager.clone().unwrap_or_default(),
            &local.assets.clone().unwrap_or_default(),
            &none,
            &keep,
        );
        let merged_facts = super::sync_state::facts(
            &merged.package,
            &merged.manager,
            &merged.assets,
            &none,
            &keep,
        );
        let remote_facts = match remote_bytes {
            Some(bytes) => {
                let remote = file_objects::remote_summary(bytes, remote_password)?;
                Some(super::sync_state::facts(
                    &remote.package,
                    &remote.manager,
                    &remote.assets,
                    &none,
                    &keep,
                ))
            }
            None => None,
        };
        let (mut changes, mut unchanged) =
            super::sync_state::upload_changes(&local_facts, merged_facts, remote_facts, conflicts);
        let secrets =
            file_objects::credential_changes(payload, remote_bytes, passphrase, remote_password)?;
        super::sync_state::include_credential_changes(&mut changes, &mut unchanged, secrets);
        if changes.is_empty() {
            if let Some(bytes) = remote_bytes {
                if file_objects::content_digest(bytes, remote_password)?
                    != file_objects::content_digest(payload, passphrase)?
                {
                    return Err(invalid(
                        "sync_compare",
                        "配置内容存在尚未解释的差异，未生成可执行预览",
                    ));
                }
            }
        }
        Ok((changes, unchanged))
    };
    let result = compare().ok();
    super::sync_state::SyncPreview {
        source: "local".into(),
        unchanged: result.as_ref().map(|(_, n)| *n).unwrap_or_default(),
        changes: result.map(|(changes, _)| changes),
        created_at: merged.package.created_at.clone(),
    }
}
/// 同步下载的比较目标：按来源条目在本机已有的部分生成同一形态的配置包。
/// 本机不存在的渠道、模版版本、Skill与MCP不进入目标，比较结果即显示为「新增」。
fn local_target_scoped(
    root: &Path,
    plan: &IncomingPlan,
    renames: &std::collections::BTreeMap<String, String>,
    config_home: &Path,
    local_home: Option<&Path>,
    scope: &Scope,
) -> ModelResult<(
    ModelPackage,
    Settings,
    crate::modules::sync::portable_assets::Summary,
)> {
    let snapshot = super::config_file::snapshot(root, config_home)?;
    let channel_ids: Vec<String> = plan
        .package
        .payload
        .policy
        .iter()
        .flat_map(|policy| &policy.channels)
        .map(|channel| {
            renames
                .get(&channel.id)
                .cloned()
                .unwrap_or_else(|| channel.id.clone())
        })
        .filter(|id| snapshot.policy.channels.iter().any(|c| &c.id == id))
        .collect();
    let template_refs: Vec<TemplateRef> =
        plan.package
            .payload
            .templates
            .iter()
            .flat_map(|template| {
                template.revisions.iter().map(|revision| TemplateRef {
                    template_id: template.id.clone(),
                    revision_id: revision.id.clone(),
                })
            })
            .filter(|r| {
                snapshot.library.templates.iter().any(|t| {
                    t.id == r.template_id && t.revisions.iter().any(|v| v.id == r.revision_id)
                })
            })
            .collect();
    let package = super::portable_channels::export_file(
        root,
        config_home,
        ExportScope {
            channel_families: scope
                .channel_families
                .iter()
                .filter(|(id, _)| channel_ids.contains(id))
                .map(|(id, value)| (id.clone(), value.clone()))
                .collect(),
            channel_models: scope
                .channel_models
                .iter()
                .filter(|(id, _)| channel_ids.contains(id))
                .map(|(id, models)| (id.clone(), models.clone()))
                .collect(),
            include_policy: scope.include_global || !channel_ids.is_empty(),
            channel_ids,
            template_refs,
            include_global: scope.include_global,
        },
    )?;
    let plan_manager = plan.manager.clone().unwrap_or_default();
    let plan_assets = plan.assets.clone().unwrap_or_default();
    let (skill_names, mcp_names) = match local_home {
        Some(home) => {
            let catalog = crate::modules::sync::portable_assets::catalog(root, home)?;
            (
                plan_assets
                    .skills
                    .iter()
                    .map(|skill| skill.name.clone())
                    .filter(|name| catalog.skills.contains(name))
                    .collect(),
                plan_assets
                    .mcps
                    .iter()
                    .map(|mcp| mcp.name.clone())
                    .filter(|name| catalog.mcps.contains(name))
                    .collect(),
            )
        }
        None => (vec![], vec![]),
    };
    let manager = ManagerScope {
        preferences: plan_manager.preferences.is_some(),
        extensions: plan_manager.extensions.is_some(),
        webdav: false,
        webdav_passwords: false,
        skill_names,
        mcp_names,
        mcp_sources: scope.mcp_sources.clone(),
        extension_credentials: false,
        credential_ids: None,
        assets_reviewed: true,
        account_ids: scope
            .account_identities
            .iter()
            .filter(|local| {
                plan_manager.accounts.iter().any(|remote| {
                    crate::modules::sync::portable_accounts::same_identity(local, remote)
                })
            })
            .map(|local| local.id.clone())
            .collect(),
        upstream_root: Some(config_home.to_path_buf()),
        upstream_snapshot: None,
        local_home: local_home.map(Path::to_path_buf),
    };
    let id = uuid::Uuid::new_v4().to_string();
    let (bytes, _, mut settings, assets) =
        file_objects::seal_manager(root, &id, &package, false, None, &manager)?;
    let package = file_objects::content_package(&bytes, None)?;
    if let (Some(target), Some(source)) = (&mut settings.extensions, &plan_manager.extensions) {
        target.skills.retain(|n| source.skills.contains(n));
        target.servers.retain(|n| source.servers.contains(n));
        target
            .skill_targets
            .retain(|n, _| source.skill_targets.contains_key(n));
        target
            .enablement
            .retain(|id, _| source.enablement.contains_key(id));
    }
    Ok((package, settings, assets))
}
pub(super) fn compare_incoming(
    root: &Path,
    plan: &IncomingPlan,
    config_home: &Path,
    local_home: Option<&Path>,
) -> ModelResult<super::sync_state::SyncPreview> {
    let renames = plan
        .channel_bindings
        .iter()
        .map(|(s, b)| (s.clone(), b.target_id.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let keep = plan
        .channel_bindings
        .values()
        .filter(|b| b.keep_local_connection)
        .map(|b| b.target_id.clone())
        .collect::<BTreeSet<_>>();
    let manager = plan.manager.clone().unwrap_or_default();
    let catalog = crate::modules::sync::portable_accounts::catalog(root, config_home)?;
    let scope = Scope {
        account_identities: catalog
            .into_iter()
            .filter(|a| {
                manager.accounts.iter().any(|source| {
                    crate::modules::sync::portable_accounts::same_identity(&a.account, source)
                })
            })
            .map(|a| a.account)
            .collect(),
        channel_models: plan
            .package
            .payload
            .policy
            .iter()
            .flat_map(|p| &p.channels)
            .filter_map(|c| {
                c.model_subset.as_ref().map(|m| {
                    (
                        renames.get(&c.id).cloned().unwrap_or_else(|| c.id.clone()),
                        m.clone(),
                    )
                })
            })
            .collect(),
        channel_families: plan
            .package
            .payload
            .policy
            .iter()
            .flat_map(|p| &p.channels)
            .filter_map(|c| {
                c.family_scope.as_ref().map(|m| {
                    (
                        renames.get(&c.id).cloned().unwrap_or_else(|| c.id.clone()),
                        m.clone(),
                    )
                })
            })
            .collect(),
        include_global: false,
        ..Default::default()
    };
    let (package, settings, assets) =
        local_target_scoped(root, plan, &renames, config_home, local_home, &scope)?;
    let source = super::sync_state::facts(
        &plan.package,
        &manager,
        &plan.assets.clone().unwrap_or_default(),
        &renames,
        &keep,
    );
    let target = super::sync_state::facts(&package, &settings, &assets, &Default::default(), &keep);
    let (changes, unchanged) = super::sync_state::diff(&source, &target);
    Ok(super::sync_state::SyncPreview {
        source: "file".into(),
        changes: Some(changes),
        unchanged,
        created_at: plan.package.created_at.clone(),
    })
}
#[allow(clippy::too_many_arguments)]
async fn download_pointed<T: WebDavTransport, F: Fn() -> ModelResult<()>>(
    root: &Path,
    transport: &T,
    config: &WebDavConfig,
    passphrase: Option<&str>,
    home: Option<&Path>,
    config_home: Option<&Path>,
    unchanged: F,
) -> ModelResult<(IncomingPlan, remote::Pointer)> {
    let scope = load_scope(root)?;
    let base = remote::pointer_for(remote::CONFIG, transport, config)
        .await?
        .ok_or_else(|| invalid("remote_empty", "该端点还没有统一配置快照；旧同步域保持独立"))?;
    let bytes = remote::fetch_for(remote::CONFIG, transport, config, &base.pointer).await?;
    if remote::pointer_for(remote::CONFIG, transport, config)
        .await?
        .as_ref()
        .map(|head| &head.pointer)
        != Some(&base.pointer)
    {
        return Err(invalid(
            "remote_changed",
            "下载期间远端清单改变，请重新检查",
        ));
    }
    unchanged()?;
    if load_scope(root)? != scope {
        return Err(invalid(
            "sync_scope_changed",
            "下载期间设备范围改变，未接收",
        ));
    }
    let summary = file_objects::remote_summary(&bytes, passphrase)?;
    let template_refs = summary
        .package
        .payload
        .templates
        .iter()
        .filter(|t| scope.all_templates || scope.template_ids.contains(&t.id))
        .flat_map(|t| {
            t.revisions.iter().map(|r| TemplateRef {
                template_id: t.id.clone(),
                revision_id: r.id.clone(),
            })
        })
        .collect();
    let mut manager = scope.manager();
    manager.local_home = home.map(Path::to_path_buf);
    manager.upstream_root = config_home.map(Path::to_path_buf);
    manager.account_ids = remote_account_ids(&scope, &summary.manager.accounts);
    if scope.all_skills {
        manager.skill_names = summary
            .assets
            .skills
            .iter()
            .map(|skill| skill.name.clone())
            .collect();
    }
    if scope.all_mcps {
        manager.mcp_names = summary
            .assets
            .mcps
            .iter()
            .filter(|mcp| {
                crate::modules::sync::portable_assets::default_included_mcp(&mcp.name)
                    || scope.mcp_names.contains(&mcp.name)
            })
            .map(|mcp| mcp.name.clone())
            .collect();
    }
    let channel_ids = if scope.all_channels {
        summary
            .package
            .payload
            .policy
            .as_ref()
            .map(|policy| {
                policy
                    .channels
                    .iter()
                    .map(|channel| channel.id.clone())
                    .collect()
            })
            .unwrap_or_default()
    } else {
        scope.channel_ids.clone()
    };
    file_objects::select_remote(
        root,
        &bytes,
        passphrase,
        &ExportScope {
            channel_families: scope
                .channel_families
                .iter()
                .filter(|(id, _)| channel_ids.contains(id))
                .map(|(id, value)| (id.clone(), value.clone()))
                .collect(),
            channel_models: scope
                .channel_models
                .iter()
                .filter(|(id, _)| channel_ids.contains(id))
                .map(|(id, models)| (id.clone(), models.clone()))
                .collect(),
            include_policy: scope.include_global || !channel_ids.is_empty(),
            channel_ids,
            template_refs,
            include_global: scope.include_global,
        },
        &manager,
        scope.include_keys,
    )
    .map(|plan| (plan, base.pointer.clone()))
}

#[cfg(test)]
mod tests {
    use super::super::remote::tests::Memory;
    use super::*;
    #[test]
    fn existing_scope_is_not_expanded_during_first_catalog_migration() {
        let root = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        let saved = save_scope(
            root.path(),
            Scope {
                channel_ids: vec!["00000000-0000-4000-8000-000000000001".into()],
                ..Default::default()
            },
        )
        .unwrap();
        let catalog = CloudCatalog {
            binding: "server".into(),
            objects: vec![CatalogObject {
                id: serde_json::to_string(&["channels", "cloud-channel"]).unwrap(),
                category: "channels".into(),
                category_label: String::new(),
                label: String::new(),
                detail: String::new(),
                mark: String::new(),
                files: vec![],
            }],
            credential_ids: vec![],
            created_at: String::new(),
            accounts: vec![],
        };
        assert!(!initialize_remote_scope(
            root.path(),
            "server",
            &catalog,
            home.path(),
            home.path(),
        )
        .unwrap());
        assert_eq!(
            load_scope(root.path()).unwrap().channel_ids,
            saved.channel_ids
        );
        assert!(root.path().join(INITIALIZED).exists());
    }
    #[tokio::test]
    async fn new_device_lists_remote_objects_without_expanding_scope_and_can_explicitly_select_them(
    ) {
        let source = tempfile::tempdir().unwrap();
        let source_home = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let target_home = tempfile::tempdir().unwrap();
        let local_home = tempfile::tempdir().unwrap();
        let remote = Memory::default();
        std::fs::write(source_home.path().join("config.json"),br#"{"providers":{"fixture":{"adapter":"openai-chat","baseUrl":"https://fixture.example/v1","models":["fixture-model"],"selectedModels":["fixture-model"]}}}"#).unwrap();
        std::fs::write(target_home.path().join("config.json"), b"{}").unwrap();
        let source_scope = save_scope(
            source.path(),
            Scope {
                all_channels: true,
                preferences: false,
                extensions: false,
                upstream_root: Some(source_home.path().into()),
                ..Default::default()
            },
        )
        .unwrap();
        let up = prepare(
            source.path(),
            &remote,
            &config(),
            "catalog-bound",
            source_scope,
            None,
            || Ok(()),
        )
        .await
        .unwrap();
        publish(
            source.path(),
            &remote,
            &config(),
            &up.id,
            "catalog-bound",
            || Ok(()),
        )
        .await
        .unwrap();
        let _initial = save_scope(
            target.path(),
            Scope {
                preferences: true,
                extensions: false,
                upstream_root: Some(target_home.path().into()),
                local_home: Some(local_home.path().into()),
                ..Default::default()
            },
        )
        .unwrap();
        let before = std::fs::read(target_home.path().join("config.json")).unwrap();
        let c = refresh_catalog(
            target.path(),
            &remote,
            &config(),
            "catalog-bound",
            None,
            None,
        )
        .await
        .unwrap()
        .unwrap();
        std::fs::remove_file(target.path().join(SCOPE)).unwrap();
        initialize_remote_scope(
            target.path(),
            "catalog-bound",
            &c,
            target_home.path(),
            local_home.path(),
        )
        .unwrap();
        assert_eq!(
            std::fs::read(target_home.path().join("config.json")).unwrap(),
            before
        );
        let cloud_id = serde_json::from_str::<Vec<String>>(
            &c.objects
                .iter()
                .find(|o| o.category == "channels")
                .unwrap()
                .id,
        )
        .unwrap()[1]
            .clone();
        assert_eq!(
            load_scope(target.path()).unwrap().channel_ids,
            vec![cloud_id.clone()],
            "the first successful cloud catalog initializes all available objects as selected"
        );
        let saved_scope = load_scope(target.path()).unwrap();
        let mut later_catalog = c.clone();
        later_catalog.objects.push(CatalogObject {
            id: serde_json::to_string(&["channels", "later-channel"]).unwrap(),
            category: "channels".into(),
            category_label: "渠道与模型".into(),
            label: "后续新增".into(),
            detail: String::new(),
            mark: String::new(),
            files: vec![],
        });
        assert!(!initialize_remote_scope(
            target.path(),
            "catalog-bound",
            &later_catalog,
            target_home.path(),
            local_home.path(),
        )
        .unwrap());
        assert_eq!(load_scope(target.path()).unwrap(), saved_scope);
        let failed_root = tempfile::tempdir().unwrap();
        let invalid_catalog = CloudCatalog {
            binding: "catalog-bound".into(),
            objects: vec![CatalogObject {
                id: "not-json".into(),
                category: "channels".into(),
                category_label: String::new(),
                label: String::new(),
                detail: String::new(),
                mark: String::new(),
                files: vec![],
            }],
            credential_ids: vec![],
            created_at: String::new(),
            accounts: vec![],
        };
        assert!(initialize_remote_scope(
            failed_root.path(),
            "catalog-bound",
            &invalid_catalog,
            target_home.path(),
            local_home.path(),
        )
        .is_err());
        assert!(!failed_root.path().join(INITIALIZED).exists());
        assert!(cached_catalog(target.path(), "other-endpoint")
            .unwrap()
            .is_none());
        let selected = save_scope(
            target.path(),
            Scope {
                channel_ids: vec![cloud_id],
                preferences: false,
                cloud_binding: Some("catalog-bound".into()),
                ..saved_scope
            },
        )
        .unwrap();
        assert_eq!(
            check_source(
                target.path(),
                &remote,
                &config(),
                "catalog-bound",
                &selected
            )
            .await
            .unwrap()
            .source,
            "remote"
        );
        let incoming = download_sync(
            target.path(),
            &remote,
            &config(),
            None,
            Some(local_home.path()),
            target_home.path(),
            "catalog-bound",
            || Ok(()),
        )
        .await
        .unwrap();
        assert_eq!(
            incoming
                .package
                .payload
                .policy
                .as_ref()
                .unwrap()
                .channels
                .len(),
            1
        );
        let saved = file_objects::accept(target.path(), &incoming.id, false, None).unwrap();
        assert_eq!(saved.policy.channels.len(), 1);
        assert!(saved.workspace.is_none());
        assert!(saved.policy.channels[0].connection.enabled);
    }
    #[tokio::test]
    async fn protection_can_rotate_or_disable_without_changing_saved_content_time() {
        let root = tempfile::tempdir().unwrap();
        let remote = Memory::default();
        let scope = save_scope(
            root.path(),
            Scope {
                preferences: true,
                extensions: false,
                ..Default::default()
            },
        )
        .unwrap();
        let a = prepare(
            root.path(),
            &remote,
            &config(),
            "protection-fixture",
            scope.clone(),
            Some("fixture-password-A"),
            || Ok(()),
        )
        .await
        .unwrap();
        let saved_time = a.package.created_at.clone();
        publish(
            root.path(),
            &remote,
            &config(),
            &a.id,
            "protection-fixture",
            || Ok(()),
        )
        .await
        .unwrap();
        let b = prepare_with_protection(
            root.path(),
            &remote,
            &config(),
            "protection-fixture",
            scope.clone(),
            Some("fixture-password-B"),
            Some("fixture-password-B"),
            Some("fixture-password-A"),
            || Ok(()),
        )
        .await
        .unwrap();
        assert!(b.protected);
        assert_eq!(b.package.created_at, saved_time);
        assert!(b
            .sync
            .as_ref()
            .unwrap()
            .changes
            .as_ref()
            .unwrap()
            .iter()
            .any(|c| c.kind == "protection"));
        publish(
            root.path(),
            &remote,
            &config(),
            &b.id,
            "protection-fixture",
            || Ok(()),
        )
        .await
        .unwrap();
        let plain = prepare_with_protection(
            root.path(),
            &remote,
            &config(),
            "protection-fixture",
            scope,
            None,
            Some("fixture-password-B"),
            None,
            || Ok(()),
        )
        .await
        .unwrap();
        assert!(!plain.protected);
        assert_eq!(plain.package.created_at, saved_time);
        assert!(plain
            .sync
            .as_ref()
            .unwrap()
            .changes
            .as_ref()
            .unwrap()
            .iter()
            .any(|c| c.kind == "protection"));
        publish(
            root.path(),
            &remote,
            &config(),
            &plain.id,
            "protection-fixture",
            || Ok(()),
        )
        .await
        .unwrap();
        let head = remote::pointer_for(remote::CONFIG, &remote, &config())
            .await
            .unwrap()
            .unwrap();
        let bytes = remote::fetch_for(remote::CONFIG, &remote, &config(), &head.pointer)
            .await
            .unwrap();
        assert!(
            !file_objects::remote_summary(&bytes, None)
                .unwrap()
                .protected
        );
    }
    #[tokio::test]
    async fn file_channels_use_the_same_export_and_webdav_source_without_retired_policy_or_cli() {
        use super::super::{config_file, portable_channels};
        use serde_json::json;
        let remote = Memory::default();
        let source = tempfile::tempdir().unwrap();
        let source_home = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let target_home = tempfile::tempdir().unwrap();
        let mut raw = json!({"providers":{
            "source":{"adapter":"openai-chat","baseUrl":"https://source.example.test/v1","models":["native/custom/a","native/custom-b"],"selectedModels":["native/custom-b"],"modelAliases":{"native/custom/a":"keep-alias"},"note":"current-file"},
            "other":{"adapter":"ollama-native","baseUrl":"http://127.0.0.1:11434","models":["local:tag"],"newModelPolicy":"off"}
        }});
        std::fs::write(
            source_home.path().join("config.json"),
            serde_json::to_vec(&raw).unwrap(),
        )
        .unwrap();
        std::fs::write(
            target_home.path().join("config.json"),
            br#"{"providers":{},"localOnly":true}"#,
        )
        .unwrap();
        for root in [source.path(), target.path()] {
            std::fs::create_dir_all(root.join("manager-state")).unwrap();
            std::fs::write(root.join(POLICY_PATH), b"retired broken policy").unwrap();
            std::fs::write(root.join(DRAFT_PATH), b"retired broken draft").unwrap();
        }
        let source_scope = save_scope(
            source.path(),
            Scope {
                all_channels: true,
                include_global: true,
                preferences: false,
                extensions: false,
                upstream_root: Some(source_home.path().into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            check_source(
                source.path(),
                &remote,
                &config(),
                "file-bound",
                &source_scope
            )
            .await
            .unwrap()
            .reason,
            "no_remote"
        );
        let file = transfer::prepare_file_scope(
            source.path(),
            source_scope.export(source.path()).unwrap(),
            None,
            false,
            &source_scope.manager(),
        )
        .unwrap();
        let prepared = prepare(
            source.path(),
            &remote,
            &config(),
            "file-bound",
            source_scope.clone(),
            None,
            || Ok(()),
        )
        .await
        .unwrap();
        assert_eq!(
            serde_json::to_value(&prepared.package.payload).unwrap(),
            serde_json::to_value(&file.package.payload).unwrap()
        );
        assert_eq!(
            prepared
                .package
                .payload
                .policy
                .as_ref()
                .unwrap()
                .channels
                .len(),
            2
        );
        publish(
            source.path(),
            &remote,
            &config(),
            &prepared.id,
            "file-bound",
            || Ok(()),
        )
        .await
        .unwrap();
        save_scope(
            target.path(),
            Scope {
                all_channels: true,
                include_global: true,
                preferences: false,
                extensions: false,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            check_source(
                source.path(),
                &remote,
                &config(),
                "file-bound",
                &source_scope
            )
            .await
            .unwrap()
            .reason,
            "unchanged"
        );
        let mut target_scope = load_scope(target.path()).unwrap();
        target_scope.upstream_root = Some(target_home.path().into());
        let saved = config_file::snapshot(target.path(), target_home.path()).unwrap();
        let incoming = download_sync(
            target.path(),
            &remote,
            &config(),
            None,
            None,
            target_home.path(),
            "file-bound",
            || Ok(()),
        )
        .await
        .unwrap();
        assert_eq!(incoming.file_home.as_deref(), Some(target_home.path()));
        assert_eq!(incoming.expected_policy, saved.policy_digest);
        let adopted = file_objects::accept(target.path(), &incoming.id, false, None).unwrap();
        assert!(adopted.workspace.is_none());
        assert_ne!(adopted.policy_digest, saved.policy_digest);
        assert_eq!(adopted.policy.channels.len(), 2);
        assert_eq!(
            config_file::snapshot(target.path(), target_home.path()).unwrap(),
            adopted
        );
        let (local, key) = source_state(target.path(), &target_scope).unwrap();
        assert!(super::super::sync_state::promote_pending_content(
            target.path(),
            &incoming.id,
            "file-bound",
            &local,
            &key
        )
        .unwrap());
        assert_eq!(
            check_source(
                target.path(),
                &remote,
                &config(),
                "file-bound",
                &target_scope
            )
            .await
            .unwrap()
            .reason,
            "unchanged"
        );
        let same = download_sync(
            target.path(),
            &remote,
            &config(),
            None,
            None,
            target_home.path(),
            "file-bound",
            || Ok(()),
        )
        .await
        .unwrap();
        assert_eq!(
            same.sync.unwrap().changes,
            Some(vec![]),
            "same saved content must converge without another write"
        );
        assert_eq!(
            std::fs::read(target.path().join(DRAFT_PATH)).unwrap(),
            b"retired broken draft"
        );
        raw["providers"]["source"]["note"] = json!("file-only-later");
        std::fs::write(
            source_home.path().join("config.json"),
            serde_json::to_vec(&raw).unwrap(),
        )
        .unwrap();
        let next = prepare(
            source.path(),
            &remote,
            &config(),
            "file-bound",
            source_scope.clone(),
            None,
            || Ok(()),
        )
        .await
        .unwrap();
        assert_eq!(
            check_source(
                source.path(),
                &remote,
                &config(),
                "file-bound",
                &source_scope
            )
            .await
            .unwrap()
            .reason,
            "local_changed"
        );
        verify_upload_source(source.path(), &next.id, &source_scope).unwrap();
        publish(
            source.path(),
            &remote,
            &config(),
            &next.id,
            "file-bound",
            || Ok(()),
        )
        .await
        .unwrap();
        assert_eq!(
            check_source(
                target.path(),
                &remote,
                &config(),
                "file-bound",
                &target_scope
            )
            .await
            .unwrap()
            .reason,
            "remote_changed"
        );
        let mut target_raw: serde_json::Value =
            serde_json::from_slice(&std::fs::read(target_home.path().join("config.json")).unwrap())
                .unwrap();
        let provider = adopted.policy.channels[0].connection.provider_id.clone();
        target_raw["providers"][&provider]["note"] = json!("independent-local-edit");
        std::fs::write(
            target_home.path().join("config.json"),
            serde_json::to_vec(&target_raw).unwrap(),
        )
        .unwrap();
        assert_eq!(
            check_source(
                target.path(),
                &remote,
                &config(),
                "file-bound",
                &target_scope
            )
            .await
            .unwrap()
            .reason,
            "both_changed_remote_commit"
        );
        let source_id = portable_channels::catalog_file(source.path(), source_home.path())
            .unwrap()
            .into_iter()
            .find(|c| c.connection.provider_id == "source")
            .unwrap()
            .id;
        assert_eq!(
            portable_channels::provider(&next.package, &source_id).unwrap()["note"],
            "file-only-later"
        );
        assert_eq!(
            std::fs::read(source.path().join(POLICY_PATH)).unwrap(),
            b"retired broken policy"
        );
    }
    #[tokio::test]
    async fn account_only_sync_is_reviewed_and_preserves_an_existing_local_login_store() {
        use crate::modules::sync::portable_accounts;
        let remote = Memory::default();
        let first = tempfile::tempdir().unwrap();
        let first_auth = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        let second_auth = tempfile::tempdir().unwrap();
        let document=serde_json::to_vec(&serde_json::json!({"google-antigravity":{"accounts":[{"id":"slot","credential":{"accountId":"same-subject","email":"fixture@example.test","refresh":"DO-NOT-TRANSFER"}}]}})).unwrap();
        for source in [&first_auth, &second_auth] {
            std::fs::write(source.path().join("auth.json"), &document).unwrap();
        }
        let a = portable_accounts::catalog(first.path(), first_auth.path()).unwrap()[0]
            .account
            .id
            .clone();
        let b = portable_accounts::catalog(second.path(), second_auth.path()).unwrap()[0]
            .account
            .id
            .clone();
        assert_ne!(a, b);
        let scope = save_scope(
            first.path(),
            Scope {
                account_ids: vec![a.clone()],
                upstream_root: Some(first_auth.path().into()),
                preferences: false,
                extensions: false,
                ..Default::default()
            },
        )
        .unwrap();
        let upload = prepare(
            first.path(),
            &remote,
            &config(),
            "bound",
            scope,
            None,
            || Ok(()),
        )
        .await
        .unwrap();
        assert_eq!(upload.credential_count, 0);
        publish(
            first.path(),
            &remote,
            &config(),
            &upload.id,
            "bound",
            || Ok(()),
        )
        .await
        .unwrap();
        save_scope(
            second.path(),
            Scope {
                account_ids: vec![b.clone()],
                upstream_root: Some(second_auth.path().into()),
                preferences: false,
                extensions: false,
                ..Default::default()
            },
        )
        .unwrap();
        let incoming = download(second.path(), &remote, &config(), None, || Ok(()))
            .await
            .unwrap();
        assert_eq!(incoming.manager.as_ref().unwrap().accounts[0].id, a);
        assert!(portable_accounts::imported(second.path())
            .unwrap()
            .is_empty());
        file_objects::accept_extended(second.path(), &incoming.id, false, false, false, None)
            .unwrap();
        assert_eq!(
            std::fs::read(second_auth.path().join("auth.json")).unwrap(),
            document
        );
        let catalog = portable_accounts::catalog(second.path(), second_auth.path()).unwrap();
        assert_eq!(catalog.len(), 1);
        assert_eq!(catalog[0].account.id, b);
        assert!(catalog[0].local_source);
    }
    #[test]
    fn account_participation_matches_subject_across_devices_and_never_matches_email() {
        use crate::modules::sync::portable_accounts::AccountInfo;
        let local = AccountInfo {
            id: uuid::Uuid::new_v4().to_string(),
            provider: "google-antigravity".into(),
            subject: Some("subject".into()),
            label: "设备一".into(),
            email: Some("fixture@example.test".into()),
            origin: Some(crate::modules::sync::portable_accounts::AccountOrigin {
                version: 1,
                account_platform: "antigravity".into(),
                owner: "opencodex".into(),
                auth_kind: "oauth".into(),
                source_namespace: "oauth-account-pool".into(),
                source_account_id: "fixture-slot".into(),
                tenant: None,
            }),
        };
        let mut same = local.clone();
        same.id = uuid::Uuid::new_v4().to_string();
        same.label = "设备二".into();
        let mut other = same.clone();
        other.id = uuid::Uuid::new_v4().to_string();
        other.subject = Some("other-subject".into());
        let scope = Scope {
            account_ids: vec![local.id.clone()],
            account_identities: vec![local.clone()],
            ..Default::default()
        };
        assert_eq!(
            remote_account_ids(&scope, &[same.clone(), other.clone()]),
            vec![same.id.clone()]
        );
        let mut reused_id = local.clone();
        reused_id.origin.as_mut().unwrap().tenant = Some("another-tenant".into());
        assert!(remote_account_ids(&scope, &[reused_id.clone()]).is_empty());
        reused_id.origin.as_mut().unwrap().owner = "platform-native".into();
        assert!(remote_account_ids(&scope, &[reused_id]).is_empty());
        let mut unidentified = local;
        unidentified.subject = None;
        let scope = Scope {
            account_ids: vec![unidentified.id.clone()],
            account_identities: vec![unidentified],
            ..Default::default()
        };
        same.subject = None;
        other.subject = None;
        assert!(remote_account_ids(&scope, &[same, other]).is_empty());
    }
    #[tokio::test]
    async fn actual_skill_resources_follow_saved_device_scope_and_remain_pending_until_asset_adoption(
    ) {
        let transport = Memory::default();
        let source = tempfile::tempdir().unwrap();
        let source_home = tempfile::tempdir().unwrap();
        let skill = source_home.path().join(".agents/skills/fixture");
        std::fs::create_dir_all(&skill).unwrap();
        std::fs::write(skill.join("SKILL.md"), "# fixture portable resource").unwrap();
        std::fs::write(skill.join("data.txt"), "fixture-resource").unwrap();
        let scope = save_scope(
            source.path(),
            Scope {
                preferences: false,
                extensions: false,
                skill_names: vec!["fixture".into()],
                assets_reviewed: true,
                local_home: Some(source_home.path().into()),
                ..Default::default()
            },
        )
        .unwrap();
        let plan = prepare(
            source.path(),
            &transport,
            &config(),
            "bound",
            scope,
            None,
            || Ok(()),
        )
        .await
        .unwrap();
        assert_eq!(plan.assets.skills[0].files, 2);
        publish(
            source.path(),
            &transport,
            &config(),
            &plan.id,
            "bound",
            || Ok(()),
        )
        .await
        .unwrap();
        let target = tempfile::tempdir().unwrap();
        let target_home = tempfile::tempdir().unwrap();
        save_scope(
            target.path(),
            Scope {
                preferences: false,
                extensions: false,
                skill_names: vec!["fixture".into()],
                ..Default::default()
            },
        )
        .unwrap();
        let incoming = download_context(
            target.path(),
            &transport,
            &config(),
            None,
            Some(target_home.path()),
            || Ok(()),
        )
        .await
        .unwrap();
        let destination = target
            .path()
            .join("manager-state/skills-store/fixture/data.txt");
        assert!(!destination.exists());
        assert!(incoming.preview_error.is_none());
        assert_eq!(incoming.assets.as_ref().unwrap().skills[0].name, "fixture");
        file_objects::accept_extended(target.path(), &incoming.id, false, false, false, None)
            .unwrap();
        assert_eq!(
            std::fs::read_to_string(destination).unwrap(),
            "fixture-resource"
        );
        assert!(!target_home.path().join(".codex").exists());
    }
    fn config() -> WebDavConfig {
        WebDavConfig {
            base_url: "https://fixture.invalid".into(),
            remote_path: "sync".into(),
            username: "fixture".into(),
            password: "fixture-transport-password".into(),
        }
    }
    fn create(root: &Path, label: &str) -> String {
        let store = ModelStore::new(root);
        let snapshot = store.load().unwrap();
        let mut content = TemplateContent::default();
        content.matching.include.push(Pattern {
            kind: MatchKind::Prefix,
            value: label.into(),
        });
        store
            .save(SaveTemplate {
                expected_library: snapshot.library_digest,
                template_id: None,
                base_revision: None,
                label: label.into(),
                content,
            })
            .unwrap()
            .template_id
    }
    #[tokio::test]
    async fn sync_direction_follows_last_synced_pointer_and_preview_lists_only_changes() {
        use super::super::sync_state;
        let remote = Memory::default();
        let first = tempfile::tempdir().unwrap();
        let first_home = tempfile::tempdir().unwrap();
        let a = create(first.path(), "sync-a");
        let scope = Scope {
            template_ids: vec![a.clone()],
            ..Default::default()
        };
        let saved = save_scope(first.path(), scope.clone()).unwrap();
        assert_eq!(
            sync_state::check(first.path(), &remote, &config(), "bound")
                .await
                .unwrap()
                .source,
            "local"
        );
        let plan = prepare(
            first.path(),
            &remote,
            &config(),
            "bound",
            saved,
            None,
            || Ok(()),
        )
        .await
        .unwrap();
        let changes = plan.sync.as_ref().unwrap().changes.clone().unwrap();
        assert!(changes
            .iter()
            .any(|change| change.kind == "templates" && change.id == a && change.mark == "new"));
        publish(first.path(), &remote, &config(), &plan.id, "bound", || {
            Ok(())
        })
        .await
        .unwrap();
        let first_check = sync_state::check(first.path(), &remote, &config(), "bound")
            .await
            .unwrap();
        assert_eq!(first_check.source, "local");
        assert!(first_check.has_remote && first_check.last_synced_at.is_some());
        assert_eq!(
            sync_state::check(first.path(), &remote, &config(), "other-binding")
                .await
                .unwrap()
                .source,
            "remote"
        );
        let again = prepare(
            first.path(),
            &remote,
            &config(),
            "bound",
            load_scope(first.path()).unwrap(),
            None,
            || Ok(()),
        )
        .await
        .unwrap();
        assert_eq!(again.sync.unwrap().changes, Some(vec![]));

        let second = tempfile::tempdir().unwrap();
        let second_home = tempfile::tempdir().unwrap();
        save_scope(second.path(), scope).unwrap();
        assert_eq!(
            sync_state::check(second.path(), &remote, &config(), "bound")
                .await
                .unwrap()
                .source,
            "remote"
        );
        let incoming = download_sync(
            second.path(),
            &remote,
            &config(),
            None,
            None,
            second_home.path(),
            "bound",
            || Ok(()),
        )
        .await
        .unwrap();
        let preview = incoming.sync.clone().unwrap();
        assert_eq!(preview.source, "remote");
        assert!(preview
            .changes
            .unwrap()
            .iter()
            .any(|change| change.kind == "templates" && change.id == a));
        assert_eq!(
            sync_state::check(second.path(), &remote, &config(), "bound")
                .await
                .unwrap()
                .source,
            "remote"
        );
        file_objects::accept(second.path(), &incoming.id, false, None).unwrap();
        assert!(sync_state::promote_pending(second.path(), &incoming.id).unwrap());
        assert!(sync_state::last_synced(second.path(), "bound").is_some());
        assert_eq!(
            sync_state::check(second.path(), &remote, &config(), "bound")
                .await
                .unwrap()
                .source,
            "local"
        );
        let same = download_sync(
            first.path(),
            &remote,
            &config(),
            None,
            None,
            first_home.path(),
            "bound",
            || Ok(()),
        )
        .await
        .unwrap();
        assert_eq!(same.sync.unwrap().changes, Some(vec![]));
        assert_eq!(
            sync_state::check(first.path(), &remote, &config(), "bound")
                .await
                .unwrap()
                .source,
            "local"
        );
    }
    #[tokio::test]
    async fn partial_remote_upload_preserves_other_objects_and_saved_device_scope_bounds_reception()
    {
        let remote = Memory::default();
        let first = tempfile::tempdir().unwrap();
        let a = create(first.path(), "remote-a");
        crate::modules::preferences::PreferencesStore::new(first.path())
            .save(&crate::modules::preferences::Preferences {
                interface_scale: 125,
                ..Default::default()
            })
            .unwrap();
        let scope = save_scope(
            first.path(),
            Scope {
                template_ids: vec![a.clone()],
                preferences: true,
                extensions: false,
                ..Default::default()
            },
        )
        .unwrap();
        let plan = prepare(
            first.path(),
            &remote,
            &config(),
            "bound",
            scope,
            None,
            || Ok(()),
        )
        .await
        .unwrap();
        publish(first.path(), &remote, &config(), &plan.id, "bound", || {
            Ok(())
        })
        .await
        .unwrap();
        let second = tempfile::tempdir().unwrap();
        let b = create(second.path(), "local-b");
        let scope = save_scope(
            second.path(),
            Scope {
                template_ids: vec![b.clone()],
                preferences: false,
                extensions: true,
                ..Default::default()
            },
        )
        .unwrap();
        let plan = prepare(
            second.path(),
            &remote,
            &config(),
            "bound",
            scope,
            None,
            || Ok(()),
        )
        .await
        .unwrap();
        assert!(
            plan.package.payload.templates.iter().any(|t| t.id == a)
                && plan.package.payload.templates.iter().any(|t| t.id == b)
        );
        assert_eq!(
            plan.manager.preferences.as_ref().unwrap().interface_scale,
            125
        );
        publish(second.path(), &remote, &config(), &plan.id, "bound", || {
            Ok(())
        })
        .await
        .unwrap();
        let third = tempfile::tempdir().unwrap();
        let own = create(third.path(), "own");
        let receiver = save_scope(
            third.path(),
            Scope {
                template_ids: vec![a.clone()],
                preferences: false,
                extensions: true,
                ..Default::default()
            },
        )
        .unwrap();
        let before = ModelStore::new(third.path()).load().unwrap();
        let incoming = download(third.path(), &remote, &config(), None, || Ok(()))
            .await
            .unwrap();
        assert_eq!(ModelStore::new(third.path()).load().unwrap(), before);
        assert!(incoming.manager.as_ref().unwrap().preferences.is_none());
        let after = file_objects::accept(third.path(), &incoming.id, false, None).unwrap();
        assert!(
            after.library.templates.iter().any(|t| t.id == a)
                && after.library.templates.iter().any(|t| t.id == own)
        );
        assert!(!after.library.templates.iter().any(|t| t.id == b));
        assert!(!third
            .path()
            .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH)
            .exists());
        assert_eq!(load_scope(third.path()).unwrap(), receiver);
        assert!(remote
            .writes()
            .iter()
            .all(|path| path.starts_with("config-v1")));
    }
    #[tokio::test]
    async fn lost_pointer_reply_reuses_original_configuration_plan_and_does_not_republish() {
        let remote = Memory::default();
        let root = tempfile::tempdir().unwrap();
        let scope = save_scope(root.path(), Scope::default()).unwrap();
        let plan = prepare(
            root.path(),
            &remote,
            &config(),
            "bound",
            scope,
            None,
            || Ok(()),
        )
        .await
        .unwrap();
        remote.lose_reply();
        assert!(publish(
            root.path(),
            &remote,
            &config(),
            &plan.id,
            "bound",
            || Ok(())
        )
        .await
        .is_err());
        assert_eq!(latest_upload(root.path()).unwrap().unwrap().id, plan.id);
        assert_eq!(
            latest_upload(root.path()).unwrap().unwrap().phase,
            "publishing"
        );
        assert_eq!(
            publish(
                root.path(),
                &remote,
                &config(),
                &plan.id,
                "bound",
                || Ok(())
            )
            .await
            .unwrap()
            .phase,
            "published"
        );
        assert_eq!(
            remote
                .writes()
                .iter()
                .filter(|path| path.ends_with("-latest.json"))
                .count(),
            1
        );
        assert_eq!(
            latest_upload(root.path()).unwrap().unwrap().phase,
            "published"
        );
    }
    #[tokio::test]
    async fn protected_remote_requires_content_password_and_scope_change_cannot_publish_an_old_plan(
    ) {
        let remote = Memory::default();
        let root = tempfile::tempdir().unwrap();
        let scope = save_scope(
            root.path(),
            Scope {
                preferences: true,
                extensions: false,
                ..Default::default()
            },
        )
        .unwrap();
        let plan = prepare(
            root.path(),
            &remote,
            &config(),
            "bound",
            scope.clone(),
            Some("fixture-content-password"),
            || Ok(()),
        )
        .await
        .unwrap();
        assert!(plan.protected);
        save_scope(
            root.path(),
            Scope {
                extensions: true,
                ..scope.clone()
            },
        )
        .unwrap();
        assert!(
            matches!(publish(root.path(),&remote,&config(),&plan.id,"bound",||Ok(())).await,Err(error) if error.code=="sync_scope_changed")
        );
        assert!(remote.writes().is_empty());
        save_scope(root.path(), scope).unwrap();
        publish(
            root.path(),
            &remote,
            &config(),
            &plan.id,
            "bound",
            || Ok(()),
        )
        .await
        .unwrap();
        let receiver = tempfile::tempdir().unwrap();
        assert!(
            matches!(download(receiver.path(),&remote,&config(),None,||Ok(())).await,Err(error) if error.code=="passphrase_required")
        );
        assert!(!receiver
            .path()
            .join("manager-state/model-config-incoming")
            .exists());
        let incoming = download(
            receiver.path(),
            &remote,
            &config(),
            Some("fixture-content-password"),
            || Ok(()),
        )
        .await
        .unwrap();
        assert!(incoming.package.payload.templates.is_empty());
        assert!(incoming.manager.as_ref().unwrap().preferences.is_some());
    }
}
