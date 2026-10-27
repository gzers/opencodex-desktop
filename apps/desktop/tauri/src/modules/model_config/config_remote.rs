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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Scope {
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
    #[serde(default)]
    pub extension_credentials: bool,
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
            extension_credentials: false,
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
            extension_credentials: self.extension_credentials,
            assets_reviewed: self.assets_reviewed,
            account_ids: self.account_ids.clone(),
            upstream_root: self.upstream_root.clone(),
            upstream_snapshot: self.upstream_snapshot.clone(),
            local_home: self.local_home.clone(),
        }
    }
    fn export(&self, root: &Path) -> ModelResult<ExportScope> {
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
            let family = snapshot
                .library
                .templates
                .iter()
                .find(|t| &t.id == id)
                .ok_or_else(|| invalid("sync_scope", "范围中的模版已不存在，请重新选择"))?;
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
        if !self.all_channels
            && self
                .channel_ids
                .iter()
                .any(|id| !channel_inventory.iter().any(|c| &c.id == id))
        {
            return Err(invalid("sync_scope", "范围中的渠道尚未提交或已不存在"));
        }
        Ok(ExportScope {
            channel_families: self.channel_families.clone(),
            channel_models: self.channel_models.clone(),
            include_policy: self.include_global
                || !self.channel_ids.is_empty()
                || self.all_channels,
            channel_ids: if self.all_channels {
                channel_inventory
                    .iter()
                    .map(|channel| channel.id.clone())
                    .collect()
            } else {
                self.channel_ids.clone()
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
    scope.account_identities = crate::modules::sync::portable_accounts::export(
        root,
        scope.upstream_root.as_deref(),
        &scope.account_ids,
    )?;
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
fn scope_digest(scope: &Scope) -> ModelResult<String> {
    Ok(sha256(&serde_json::to_vec(scope)?))
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UploadView {
    pub id: String,
    pub phase: String,
    pub package: ModelPackage,
    pub manager: Settings,
    pub credential_count: usize,
    pub protected: bool,
    #[serde(default)]
    pub assets: crate::modules::sync::portable_assets::Summary,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Plan {
    view: UploadView,
    binding: String,
    scope_digest: String,
    baseline: Option<Baseline>,
    file_digest: String,
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
pub async fn prepare<T: WebDavTransport, F: Fn() -> ModelResult<()>>(
    root: &Path,
    transport: &T,
    config: &WebDavConfig,
    binding: &str,
    scope: Scope,
    passphrase: Option<&str>,
    unchanged: F,
) -> ModelResult<UploadView> {
    scope.validate()?;
    if scope_digest(&load_scope(root)?)? != scope_digest(&scope)? {
        return Err(invalid("sync_scope", "先明确保存本设备范围，再准备上传"));
    }
    let export = scope.export(root)?;
    let mut manager = scope.manager();
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
            manager.mcp_names = catalog.mcps;
        }
    }
    let local =
        transfer::prepare_file_scope(root, export, passphrase, scope.include_keys, &manager)?;
    let local_bytes = transfer::file_bytes(root, &local.id)?;
    let baseline = remote::pointer_for(remote::CONFIG, transport, config).await?;
    unchanged()?;
    let id = uuid::Uuid::new_v4().to_string();
    let payload = match &baseline {
        Some(base) => file_objects::merge_remote(
            &remote::fetch_for(remote::CONFIG, transport, config, &base.pointer).await?,
            &local_bytes,
            passphrase,
            &id,
        )?,
        None => local_bytes,
    };
    let summary = file_objects::remote_summary(&payload, passphrase)?;
    let view = UploadView {
        id: id.clone(),
        phase: "prepared".into(),
        package: summary.package,
        manager: summary.manager,
        credential_count: summary.credential_count,
        protected: summary.protected,
        assets: summary.assets,
    };
    let plan = Plan {
        view: view.clone(),
        binding: binding.into(),
        scope_digest: scope_digest(&scope)?,
        baseline,
        file_digest: sha256(&payload),
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
}

#[cfg(test)]
mod tests {
    use super::super::remote::tests::Memory;
    use super::*;
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
        let saved = config_file::snapshot(target.path(), target_home.path()).unwrap();
        let incoming = download_context_file(
            target.path(),
            &remote,
            &config(),
            None,
            None,
            target_home.path(),
            || Ok(()),
        )
        .await
        .unwrap();
        assert_eq!(incoming.file_home.as_deref(), Some(target_home.path()));
        assert_eq!(incoming.expected_policy, saved.policy_digest);
        let adopted = file_objects::accept(target.path(), &incoming.id, false, None).unwrap();
        assert_eq!(adopted.policy, saved.policy);
        assert_eq!(adopted.policy_digest, saved.policy_digest);
        assert_eq!(
            adopted.workspace.as_ref().unwrap().base_policy,
            saved.policy_digest
        );
        assert_eq!(adopted.workspace.unwrap().policy.channels.len(), 2);
        assert_eq!(
            config_file::snapshot(target.path(), target_home.path())
                .unwrap()
                .policy,
            saved.policy
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
            source_scope,
            None,
            || Ok(()),
        )
        .await
        .unwrap();
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
