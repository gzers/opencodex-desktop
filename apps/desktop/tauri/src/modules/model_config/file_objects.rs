//! Portable object payloads keep secrets behind the shared protected envelope.
//! Incoming metadata/IPC never contains key values. Adoption creates fresh local
//! keyring references and only stages policy through the common transaction.
use super::{
    discovery,
    schema::*,
    store::{sha256, ModelStore},
    transfer::{self, IncomingPlan, ModelPackage},
};
use crate::modules::sync::portable_assets::{self, Assets};
use crate::modules::sync::portable_manager::{self, Scope as ManagerScope, Settings};
use crate::{
    infrastructure::atomic_write::atomic_write,
    modules::{backup::safety, sync::portable_file},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use zeroize::{Zeroize, Zeroizing};
#[derive(Default, Serialize, Deserialize)]
#[serde(transparent)]
struct SecretValues(BTreeMap<String, String>);
impl Drop for SecretValues {
    fn drop(&mut self) {
        for value in self.0.values_mut() {
            value.zeroize();
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Objects {
    format: String,
    version: u32,
    models: ModelPackage,
    channel_keys: SecretValues,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    channel_authorizations: BTreeMap<String, super::provider_auth::PortableAuthorization>,
    #[serde(default)]
    manager: Settings,
    #[serde(default)]
    webdav_passwords: SecretValues,
    #[serde(default)]
    assets: Assets,
}
pub(super) trait Credentials {
    fn read(&self, reference: &str) -> ModelResult<Zeroizing<String>>;
    fn install(&self, reference: &str, value: &str) -> ModelResult<()>;
    fn remove(&self, reference: &str) -> ModelResult<()>;
    fn read_webdav(&self, reference: &str) -> ModelResult<Zeroizing<String>> {
        self.read(reference)
    }
    fn install_webdav(&self, reference: &str, value: &str) -> ModelResult<()> {
        self.install(reference, value)
    }
}
pub(super) struct SystemCredentials;
impl Credentials for SystemCredentials {
    fn read(&self, reference: &str) -> ModelResult<Zeroizing<String>> {
        discovery::credential(reference)
    }
    fn install(&self, reference: &str, value: &str) -> ModelResult<()> {
        discovery::import_credential(reference, value)
    }
    fn remove(&self, reference: &str) -> ModelResult<()> {
        discovery::forget(reference)
    }
    fn read_webdav(&self, reference: &str) -> ModelResult<Zeroizing<String>> {
        crate::infrastructure::keychain::load_webdav_password(reference)
            .map(Zeroizing::new)
            .map_err(Into::into)
    }
    fn install_webdav(&self, reference: &str, value: &str) -> ModelResult<()> {
        use crate::infrastructure::keychain;
        if let Some(current) = keychain::load_optional_keychain_password(
            keychain::WEBDAV_CREDENTIAL_PURPOSE,
            reference,
        )? {
            if current == value {
                return Ok(());
            }
            return Err(invalid(
                "credential_collision",
                "导入WebDAV密码引用存在其他内容",
            ));
        }
        keychain::store_webdav_password(reference, value)?;
        if keychain::load_webdav_password(reference)? != value {
            return Err(invalid("credential_readback", "WebDAV密码写后读回不同"));
        }
        Ok(())
    }
}
fn secret_objects(objects: &Objects) -> usize {
    objects.channel_keys.0.len()
        + objects.channel_authorizations.len()
        + objects.webdav_passwords.0.len()
        + portable_assets::secret_count(&objects.assets)
}
fn sections(objects: &Objects) -> Vec<String> {
    let mut names = vec!["models".into()];
    if !objects.channel_keys.0.is_empty() {
        names.push("channelKeys".into());
    }
    if !objects.channel_authorizations.is_empty() {
        names.push("channelAuthorizations".into());
    }
    names.extend(portable_manager::sections(&objects.manager));
    if !objects.webdav_passwords.0.is_empty() {
        names.push("webdavPasswords".into());
    }
    if !objects.assets.skills.is_empty() {
        names.push("skillAssets".into());
    }
    if !objects.assets.mcps.is_empty() {
        names.push("mcpDefinitions".into());
    }
    if portable_assets::secret_count(&objects.assets) > 0 {
        names.push("extensionCredentials".into());
    }
    names
}
fn object_version(objects: &Objects) -> u32 {
    if objects.manager.accounts.iter().any(|a| a.origin.is_some()) {
        3
    } else if !objects.channel_authorizations.is_empty() {
        2
    } else {
        1
    }
}
fn validate(objects: &Objects) -> ModelResult<()> {
    if objects.format != "OCXD_CONFIG_OBJECTS"
        || ![1, 2, 3].contains(&objects.version)
        || objects.version < object_version(objects)
    {
        return Err(invalid("portable_objects", "不支持的配置对象文档版本"));
    }
    transfer::validate_package(&objects.models)?;
    portable_manager::validate(&objects.manager)?;
    portable_assets::validate(&objects.assets)?;
    for (channel, auth) in &objects.channel_authorizations {
        if objects.channel_keys.0.contains_key(channel) {
            return Err(invalid("credential_scope", "同一渠道不能包含两套授权载荷"));
        }
        let public = super::portable_channels::provider(&objects.models, channel)
            .ok_or_else(|| invalid("credential_scope", "渠道授权缺少同范围原字段"))?;
        super::provider_auth::validate_portable(public, auth)?;
    }
    for (channel, key) in &objects.channel_keys.0 {
        require_uuid(channel)?;
        if key.is_empty()
            || key.len() > 8192
            || key.contains(['\r', '\n'])
            || objects.models.payload.policy.as_ref().is_none_or(|p| {
                !p.channels
                    .iter()
                    .any(|c| &c.id == channel && c.connection.is_some())
            })
        {
            return Err(invalid(
                "credential_scope",
                "凭据格式无效或不在文件的渠道范围内",
            ));
        }
    }
    for (id, value) in &objects.webdav_passwords.0 {
        if value.is_empty()
            || value.len() > 8192
            || !objects
                .manager
                .webdav
                .iter()
                .any(|endpoint| &endpoint.id == id)
        {
            return Err(invalid("credential_scope", "WebDAV密码不在文件连接范围内"));
        }
    }
    Ok(())
}
fn decode(bytes: &[u8]) -> ModelResult<Objects> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|_| invalid("portable_objects", "配置对象文档无法解析"))?;
    for (name, known) in [
        (
            "preferences",
            serde_json::to_value(crate::modules::preferences::Preferences::default())?,
        ),
        (
            "extensions",
            serde_json::to_value(crate::modules::extensions::ExtensionConfig::default())?,
        ),
    ] {
        if value["manager"][name]
            .as_object()
            .is_some_and(|object| object.keys().any(|key| known.get(key).is_none()))
        {
            return Err(invalid(
                "portable_objects",
                "管理器配置包含当前版本未知字段，未截断导入",
            ));
        }
    }
    let objects = if value["format"] == "OCXD_MODEL_CONFIG" {
        Objects {
            format: "OCXD_CONFIG_OBJECTS".into(),
            version: 1,
            models: serde_json::from_slice(bytes)?,
            channel_keys: Default::default(),
            channel_authorizations: Default::default(),
            manager: Settings::default(),
            webdav_passwords: Default::default(),
            assets: Default::default(),
        }
    } else {
        serde_json::from_slice(bytes)
            .map_err(|_| invalid("portable_objects", "配置对象文档含未知字段或结构无效"))?
    };
    validate(&objects)?;
    Ok(objects)
}
#[cfg(test)]
fn seal_with(
    root: &Path,
    id: &str,
    package: &ModelPackage,
    include_keys: bool,
    passphrase: Option<&str>,
    credentials: &impl Credentials,
) -> ModelResult<(Vec<u8>, usize)> {
    let (bytes, count, _, _) = seal_scope(
        root,
        id,
        package,
        include_keys,
        passphrase,
        &ManagerScope::default(),
        credentials,
    )?;
    Ok((bytes, count))
}
pub(super) fn seal_manager(
    root: &Path,
    id: &str,
    package: &ModelPackage,
    include_keys: bool,
    passphrase: Option<&str>,
    scope: &ManagerScope,
) -> ModelResult<(Vec<u8>, usize, Settings, portable_assets::Summary)> {
    seal_scope(
        root,
        id,
        package,
        include_keys,
        passphrase,
        scope,
        &SystemCredentials,
    )
}
fn seal_scope(
    root: &Path,
    id: &str,
    package: &ModelPackage,
    include_keys: bool,
    passphrase: Option<&str>,
    scope: &ManagerScope,
    credentials: &impl Credentials,
) -> ModelResult<(Vec<u8>, usize, Settings, portable_assets::Summary)> {
    if include_keys && passphrase.is_none_or(|password| password.is_empty()) {
        return Err(invalid(
            "passphrase_required",
            "携带渠道Key时须设置独立文件保护口令",
        ));
    }
    let upstream_source = if package
        .payload
        .policy
        .iter()
        .flat_map(|p| &p.channels)
        .any(|c| c.connection.as_ref().is_some_and(|v| v.provider.is_some()))
    {
        let source = scope
            .upstream_root
            .as_deref()
            .ok_or_else(|| invalid("provider_source", "原渠道导出尚未绑定官方数据根"))?;
        let file = source.join("config.json");
        safety::check_path(source, &file, false)?;
        Some((
            file,
            sha256(&Zeroizing::new(safety::read_file(
                &source.join("config.json"),
                16 * 1024 * 1024,
            )?)),
        ))
    } else {
        None
    };
    if let (Some(snapshot), Some(source)) = (&scope.upstream_snapshot, &scope.upstream_root) {
        let catalog = super::portable_channels::catalog(root, snapshot)?;
        for channel in package.payload.policy.iter().flat_map(|p| &p.channels) {
            if let Some(_public) = channel
                .connection
                .as_ref()
                .and_then(|c| c.provider.as_ref())
            {
                let original = catalog
                    .iter()
                    .find(|c| c.id == channel.id)
                    .ok_or_else(|| invalid("provider_scope", "原渠道已不存在"))?;
                let mut expected = transfer::portable(original);
                expected.connection.as_mut().unwrap().provider =
                    Some(snapshot.providers["providers"][&original.connection.provider_id].clone());
                if !transfer::is_scoped_channel(&expected, channel)? {
                    return Err(invalid("provider_scope", "所选型号字段不属于冻结源渠道"));
                }
                let raw =
                    super::provider_auth::source_row(source, &original.connection.provider_id)?;
                let actual = super::provider_fields::public_from_raw(&raw)?;
                if actual.as_object().unwrap().iter().any(|(key, value)| {
                    snapshot.providers["providers"][&original.connection.provider_id].get(key)
                        != Some(value)
                }) {
                    return Err(invalid(
                        "provider_source_drift",
                        "原渠道字段在比较后改变，未混合导出",
                    ));
                }
            }
        }
    }
    let mut keys = SecretValues::default();
    let mut authorizations = BTreeMap::new();
    if include_keys {
        let snapshot = ModelStore::new(root).load()?;
        let channels = package
            .payload
            .policy
            .as_ref()
            .map(|p| p.channels.as_slice())
            .unwrap_or_default();
        if channels.is_empty() {
            return Err(invalid(
                "credential_scope",
                "选择携带渠道Key前，请先选择已提交渠道",
            ));
        }
        for channel in channels {
            if let (Some(public), Some(upstream), Some(upstream_root)) = (
                channel
                    .connection
                    .as_ref()
                    .and_then(|c| c.provider.as_ref()),
                &scope.upstream_snapshot,
                &scope.upstream_root,
            ) {
                let catalog = super::portable_channels::catalog(root, upstream)?;
                let original = catalog
                    .iter()
                    .find(|c| c.id == channel.id)
                    .ok_or_else(|| invalid("credential_scope", "所选原渠道已不存在"))?;
                if public["authMode"]
                    .as_str()
                    .is_some_and(|m| ["oauth", "forward"].contains(&m))
                {
                    // Login material belongs to the separately selected account
                    // domain; a bulk channel export must not convert it to keys.
                    continue;
                }
                authorizations.insert(
                    channel.id.clone(),
                    super::provider_auth::export_portable_scoped(
                        upstream_root,
                        &original.connection.provider_id,
                        public,
                        &upstream.providers["providers"][&original.connection.provider_id],
                    )?,
                );
                continue;
            }
            let source = snapshot
                .policy
                .channels
                .iter()
                .find(|c| c.id == channel.id)
                .ok_or_else(|| invalid("stale_baseline", "冻结导出后渠道已改变"))?;
            if !transfer::is_scoped_channel(&transfer::portable(source), channel)? {
                return Err(invalid(
                    "stale_baseline",
                    "导出配置与当前已提交渠道不同，未携带较新凭据",
                ));
            }
            if source.connection.protocol == "ollama-native"
                && source.connection.credential_ref.is_none()
            {
                continue;
            }
            let reference = source.connection.credential_ref.as_deref().ok_or_else(|| {
                invalid(
                    "credential_unavailable",
                    format!(
                        "{}尚无管理器可导出的Key；请保留配置导出或先补充Key",
                        source.label
                    ),
                )
            })?;
            keys.0
                .insert(channel.id.clone(), credentials.read(reference)?.to_string());
        }
    }
    let mut objects = Objects {
        format: "OCXD_CONFIG_OBJECTS".into(),
        version: if authorizations.is_empty() { 1 } else { 2 },
        models: package.clone(),
        channel_keys: keys,
        channel_authorizations: authorizations,
        manager: portable_manager::export(root, scope)?,
        webdav_passwords: Default::default(),
        assets: if scope.skill_names.is_empty() && scope.mcp_names.is_empty() {
            Assets::default()
        } else {
            portable_assets::export(
                root,
                scope
                    .local_home
                    .as_deref()
                    .ok_or_else(|| invalid("asset_home", "导出扩展资源的本机路径尚未绑定"))?,
                &scope.skill_names,
                &scope.mcp_names,
                scope.extension_credentials,
                scope.assets_reviewed,
            )?
        },
    };
    if scope.webdav_passwords && scope.webdav {
        let config = crate::modules::sync::config::SyncConfigStore::new(root).load()?;
        for endpoint in &objects.manager.webdav {
            let current = config
                .endpoints
                .iter()
                .find(|current| current.endpoint_id == endpoint.id)
                .ok_or_else(|| invalid("stale_baseline", "WebDAV导出基准改变"))?;
            objects.webdav_passwords.0.insert(
                endpoint.id.clone(),
                credentials
                    .read_webdav(&current.credential_ref.ref_id)?
                    .to_string(),
            );
        }
    }
    if let Some((file, expected)) = upstream_source {
        if sha256(&Zeroizing::new(safety::read_file(&file, 16 * 1024 * 1024)?)) != expected {
            return Err(invalid(
                "provider_source_drift",
                "导出期间原渠道源已改变，请重新比较",
            ));
        }
    }
    objects.version = object_version(&objects);
    validate(&objects)?;
    let count = secret_objects(&objects);
    let plaintext = Zeroizing::new(serde_json::to_vec(&objects)?);
    let bytes = portable_file::seal(
        portable_file::Manifest {
            id: id.into(),
            created_at: package.created_at.clone(),
            sections: sections(&objects),
            secret_objects: count,
            document_digest: String::new(),
        },
        &plaintext,
        passphrase,
    )?;
    Ok((
        bytes,
        count,
        objects.manager,
        portable_assets::summarize(&objects.assets),
    ))
}
fn cache_path(root: &Path, id: &str, extension: &str) -> ModelResult<PathBuf> {
    require_uuid(id)?;
    let path = root
        .join("manager-state/model-file-incoming")
        .join(format!("{id}.{extension}"));
    safety::check_path(root, &path, true)?;
    Ok(path)
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSummary {
    pub package: ModelPackage,
    pub manager: Settings,
    pub credential_count: usize,
    pub protected: bool,
    pub assets: portable_assets::Summary,
}
fn remote_objects(bytes: &[u8], passphrase: Option<&str>) -> ModelResult<(Objects, bool)> {
    let opened = portable_file::open(bytes, passphrase)?;
    let objects = decode(&opened.document)?;
    if !objects.manager.webdav.is_empty() || !objects.webdav_passwords.0.is_empty() {
        return Err(invalid(
            "sync_self_scope",
            "WebDAV连接及密码不能进入配置自同步",
        ));
    }
    if opened.manifest.sections != sections(&objects)
        || opened.manifest.secret_objects != secret_objects(&objects)
        || (secret_objects(&objects) > 0 && !opened.protected)
    {
        return Err(invalid("portable_manifest", "远端清单与对象不同"));
    }
    Ok((objects, opened.protected))
}
pub fn remote_summary(bytes: &[u8], passphrase: Option<&str>) -> ModelResult<RemoteSummary> {
    let (objects, protected) = remote_objects(bytes, passphrase)?;
    let credential_count = secret_objects(&objects);
    let assets = portable_assets::summarize(&objects.assets);
    Ok(RemoteSummary {
        package: objects.models,
        manager: objects.manager,
        credential_count,
        protected,
        assets,
    })
}
pub fn merge_remote(
    remote_bytes: &[u8],
    local_bytes: &[u8],
    passphrase: Option<&str>,
    operation_id: &str,
) -> ModelResult<Vec<u8>> {
    let (mut remote, remote_protected) = remote_objects(remote_bytes, passphrase)?;
    let (local, local_protected) = remote_objects(local_bytes, passphrase)?;
    for incoming in local.models.payload.policy.iter().flat_map(|p| &p.channels) {
        if (remote.channel_keys.0.contains_key(&incoming.id)
            || remote.channel_authorizations.contains_key(&incoming.id))
            && !local.channel_keys.0.contains_key(&incoming.id)
            && !local.channel_authorizations.contains_key(&incoming.id)
        {
            let previous = remote
                .models
                .payload
                .policy
                .iter()
                .flat_map(|p| &p.channels)
                .find(|c| c.id == incoming.id);
            if previous.is_none_or(|c| !transfer::same_destination(c, incoming)) {
                return Err(invalid(
                    "channel_credential_binding",
                    "远端渠道含Key，新的协议或节点未携对应凭据；未把旧Key绑定到新服务",
                ));
            }
        }
    }
    remote.models = transfer::merge_packages(remote.models, local.models)?;
    if local.manager.preferences.is_some() {
        remote.manager.preferences = local.manager.preferences;
    }
    if let Some(incoming) = local.manager.extensions {
        if let Some(existing) = remote.manager.extensions.as_mut() {
            for name in incoming.skills {
                if !existing.skills.contains(&name) {
                    existing.skills.push(name);
                }
            }
            for name in incoming.servers {
                if !existing.servers.contains(&name) {
                    existing.servers.push(name);
                }
            }
            existing.enablement.extend(incoming.enablement);
            existing.skill_targets.extend(incoming.skill_targets);
            existing.sync_method = incoming.sync_method;
        } else {
            remote.manager.extensions = Some(incoming);
        }
    }
    remote.manager.accounts = crate::modules::sync::portable_accounts::merge(
        remote.manager.accounts,
        &local.manager.accounts,
    )?;
    for (channel, key) in &local.channel_keys.0 {
        remote.channel_authorizations.remove(channel);
        remote.channel_keys.0.insert(channel.clone(), key.clone());
    }
    for (channel, auth) in local.channel_authorizations {
        remote.channel_keys.0.remove(&channel);
        remote.channel_authorizations.insert(channel, auth);
    }
    remote.version = object_version(&remote);
    for mut skill in local.assets.skills {
        if let Some(index) = remote
            .assets
            .skills
            .iter()
            .position(|old| old.name == skill.name)
        {
            for old in remote.assets.skills[index]
                .files
                .iter()
                .filter(|file| file.sensitive)
            {
                if let Some(new) = skill.files.iter().find(|file| file.path == old.path) {
                    if !new.sensitive && new.digest != old.digest {
                        return Err(invalid(
                            "skill_remote_conflict",
                            "远端Skill的受保护文件与当前内容不同，未覆盖凭据文件",
                        ));
                    }
                } else {
                    skill.files.push(old.clone());
                }
            }
            skill.files.sort_by(|a, b| a.path.cmp(&b.path));
            skill.contains_secrets = skill.files.iter().any(|file| file.sensitive);
            skill.digest = sha256(&serde_json::to_vec(&skill.files)?);
            remote.assets.skills[index] = skill;
        } else {
            remote.assets.skills.push(skill);
        }
    }
    for mut mcp in local.assets.mcps {
        if let Some(index) = remote
            .assets
            .mcps
            .iter()
            .position(|old| old.name == mcp.name)
        {
            let old = &remote.assets.mcps[index];
            if old.definition != mcp.definition
                && !old.secrets.0.is_empty()
                && mcp.secrets.0.is_empty()
            {
                return Err(invalid(
                    "mcp_remote_conflict",
                    "远端MCP含凭据且定义不同，请先明确映射或携带对应新凭据",
                ));
            }
            if old.definition == mcp.definition {
                for (path, value) in &old.secrets.0 {
                    mcp.secrets
                        .0
                        .entry(path.clone())
                        .or_insert_with(|| value.clone());
                }
            }
            remote.assets.mcps[index] = mcp;
        } else {
            remote.assets.mcps.push(mcp);
        }
    }
    validate(&remote)?;
    let document = Zeroizing::new(serde_json::to_vec(&remote)?);
    let password =
        if remote_protected || local_protected || secret_objects(&remote) > 0 {
            Some(passphrase.ok_or_else(|| {
                invalid("passphrase_required", "远端文件受保护，请输入配置同步口令")
            })?)
        } else {
            passphrase
        };
    portable_file::seal(
        portable_file::Manifest {
            id: operation_id.into(),
            created_at: chrono::Utc::now().to_rfc3339(),
            sections: sections(&remote),
            secret_objects: secret_objects(&remote),
            document_digest: String::new(),
        },
        &document,
        password,
    )
}
pub fn select_remote(
    root: &Path,
    bytes: &[u8],
    passphrase: Option<&str>,
    models: &transfer::ExportScope,
    manager: &ManagerScope,
    include_keys: bool,
) -> ModelResult<IncomingPlan> {
    let (mut objects, _) = remote_objects(bytes, passphrase)?;
    let channels = objects
        .models
        .payload
        .policy
        .as_ref()
        .map(|p| {
            p.channels
                .iter()
                .filter(|channel| models.include_policy && models.channel_ids.contains(&channel.id))
                .map(|channel| channel.id.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    objects
        .channel_keys
        .0
        .retain(|channel, _| include_keys && channels.contains(channel));
    objects
        .channel_authorizations
        .retain(|channel, _| include_keys && channels.contains(channel));
    objects
        .manager
        .accounts
        .retain(|a| manager.account_ids.contains(&a.id));
    objects.version = object_version(&objects);
    if !manager.preferences {
        objects.manager.preferences = None;
    }
    if !manager.extensions {
        objects.manager.extensions = None;
    }
    objects
        .assets
        .skills
        .retain(|skill| manager.skill_names.contains(&skill.name));
    objects
        .assets
        .mcps
        .retain(|mcp| manager.mcp_names.contains(&mcp.name));
    if !manager.extension_credentials {
        for skill in &mut objects.assets.skills {
            if skill
                .files
                .iter()
                .any(|file| file.path == "SKILL.md" && file.sensitive)
            {
                return Err(invalid(
                    "asset_credentials_required",
                    "所选Skill正文含凭据，请选择受保护扩展凭据或跳过该对象",
                ));
            }
            skill.files.retain(|file| !file.sensitive);
            skill.contains_secrets = false;
            skill.digest = sha256(&serde_json::to_vec(&skill.files)?);
        }
        for mcp in &mut objects.assets.mcps {
            mcp.secrets.0.clear();
        }
    }
    let id = uuid::Uuid::new_v4().to_string();
    let document = Zeroizing::new(serde_json::to_vec(&objects)?);
    let selected_file = portable_file::seal(
        portable_file::Manifest {
            id,
            created_at: chrono::Utc::now().to_rfc3339(),
            sections: sections(&objects),
            secret_objects: secret_objects(&objects),
            document_digest: String::new(),
        },
        &document,
        passphrase,
    )?;
    let incoming = receive_context(
        root,
        &selected_file,
        passphrase,
        manager.local_home.as_deref(),
    )?;
    let template_refs = models
        .template_refs
        .iter()
        .filter(|reference| {
            incoming.package.payload.templates.iter().any(|template| {
                template.id == reference.template_id
                    && template
                        .revisions
                        .iter()
                        .any(|revision| revision.id == reference.revision_id)
            })
        })
        .cloned()
        .collect();
    transfer::select_received_context(
        root,
        &incoming.id,
        transfer::ReceiveScope {
            channel_models: models.channel_models.clone(),
            asset_mappings: Default::default(),
            channel_mappings: BTreeMap::new(),
            account_ids: manager
                .account_ids
                .iter()
                .filter(|id| {
                    incoming
                        .manager
                        .as_ref()
                        .is_some_and(|m| m.accounts.iter().any(|a| &a.id == *id))
                })
                .cloned()
                .collect(),
            template_refs,
            channel_ids: channels,
            include_global: models.include_global,
            preferences: manager.preferences
                && incoming
                    .manager
                    .as_ref()
                    .is_some_and(|settings| settings.preferences.is_some()),
            extensions: manager.extensions
                && incoming
                    .manager
                    .as_ref()
                    .is_some_and(|settings| settings.extensions.is_some()),
            endpoint_ids: vec![],
            skill_names: manager.skill_names.clone(),
            mcp_names: manager.mcp_names.clone(),
        },
        passphrase,
    )
}
pub fn receive(root: &Path, bytes: &[u8], passphrase: Option<&str>) -> ModelResult<IncomingPlan> {
    receive_context(root, bytes, passphrase, None)
}
pub fn receive_context(
    root: &Path,
    bytes: &[u8],
    passphrase: Option<&str>,
    home: Option<&Path>,
) -> ModelResult<IncomingPlan> {
    let format: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|_| invalid("import_format", "所选配置文件不是有效JSON"))?;
    let legacy = format["container_magic"] == "OCXDCONF";
    let objects = if legacy {
        let document = crate::modules::migration::read_portable_legacy(bytes, passphrase).map_err(
            |error| match error {
                crate::modules::migration::MigrationError::PassphraseRequired => {
                    invalid("passphrase_required", "旧配置文件需要原保护口令")
                }
                crate::modules::migration::MigrationError::Authentication => {
                    invalid("portable_authentication", "旧文件口令不正确或载荷损坏")
                }
                _ => invalid("import_format", "旧配置文件未通过格式或完整性校验"),
            },
        )?;
        let preferences = document
            .preferences
            .map(crate::modules::preferences::Preferences::try_from)
            .transpose()
            .map_err(|_| invalid("portable_preferences", "旧配置偏好无法转换"))?;
        let extensions = document.extension_config.map(|extension| {
            let mut config = crate::modules::extensions::ExtensionConfig {
                enablement: extension.enablement,
                sync_method: extension.sync_method.unwrap_or_else(|| "symlink".into()),
                ..Default::default()
            };
            config.skills = extension
                .skills
                .into_iter()
                .map(|skill| skill.name)
                .collect();
            config.servers = extension.server_names;
            for server in extension.servers {
                if !config.servers.contains(&server.name) {
                    config.servers.push(server.name);
                }
            }
            config
        });
        let webdav = document
            .sync_endpoints
            .unwrap_or_default()
            .into_iter()
            .filter_map(|endpoint| {
                Some(portable_manager::Endpoint {
                    id: endpoint.name,
                    url: endpoint.url?,
                    remote_path: endpoint.remote_path?,
                    username: endpoint.username?,
                    conflict_policy: endpoint.conflict_policy.unwrap_or_else(|| "ask".into()),
                })
            })
            .collect();
        let payload = transfer::TransferPayload {
            templates: vec![],
            policy: None,
        };
        let objects = Objects {
            format: "OCXD_CONFIG_OBJECTS".into(),
            version: 1,
            models: ModelPackage {
                format: "OCXD_MODEL_CONFIG".into(),
                format_version: 1,
                schema_version: SCHEMA_VERSION,
                created_at: chrono::Utc::now().to_rfc3339(),
                digest: sha256(&serde_json::to_vec(&payload)?),
                payload,
            },
            channel_keys: Default::default(),
            channel_authorizations: Default::default(),
            manager: Settings {
                preferences,
                extensions,
                webdav,
                accounts: vec![],
            },
            webdav_passwords: Default::default(),
            assets: Default::default(),
        };
        validate(&objects)?;
        objects
    } else if format["format"] == "OCXDCONF" {
        let opened = portable_file::open(bytes, passphrase)?;
        let objects = decode(&opened.document)?;
        if opened.manifest.sections != sections(&objects)
            || opened.manifest.secret_objects != secret_objects(&objects)
            || ((secret_objects(&objects) > 0) && !opened.protected)
        {
            return Err(invalid(
                "portable_manifest",
                "文件清单与实际对象或凭据保护不一致",
            ));
        }
        objects
    } else {
        let objects = decode(bytes)?;
        if secret_objects(&objects) > 0 {
            return Err(invalid(
                "portable_protection",
                "携带Key的对象文档必须来自受保护配置文件",
            ));
        }
        objects
    };
    let credential_count = secret_objects(&objects);
    let mut plan = transfer::receive(root, objects.models.clone())?;
    plan.protected = format["format"] == "OCXDCONF" && format["payload"]["kind"] == "protected";
    if legacy {
        plan.notices.push("已按旧容器实际区段转换；旧文件不含模型库、真实登录态或Key。扩展名清单不表示资源/凭据已恢复。".into());
    }
    plan.extension_source = objects.manager.extensions.clone();
    plan.manager = Some(objects.manager.clone());
    plan.available_webdav_passwords = objects.webdav_passwords.0.keys().cloned().collect();
    if let Err(error) = portable_manager::prepare(root, &plan.id, &objects.manager) {
        if error.code != "account_capability" {
            return Err(error);
        }
        plan.notices.push(error.message);
        plan.preview_error = Some(error.code);
        let mut supported = objects.manager.clone();
        supported
            .accounts
            .retain(crate::modules::sync::portable_accounts::information_supported);
        portable_manager::prepare(root, &plan.id, &supported)?;
    }
    if !objects.assets.skills.is_empty() || !objects.assets.mcps.is_empty() {
        let home = home.ok_or_else(|| invalid("asset_home", "接收扩展资源需绑定本机主目录"))?;
        atomic_write(
            &cache_path(root, &plan.id, "asset-home.json")?,
            &serde_json::to_vec(home)?,
            0o600,
        )?;
        if let Err(error) = portable_assets::prepare(root, home, &plan.id, &objects.assets) {
            plan.notices.push(error.message);
            plan.preview_error = Some(error.code);
        }
        plan.assets = Some(portable_assets::summarize(&objects.assets));
        plan.available_extension_credentials = portable_assets::secret_count(&objects.assets) > 0;
    }
    if credential_count > 0 {
        plan.available_keys = objects
            .channel_keys
            .0
            .keys()
            .chain(objects.channel_authorizations.keys())
            .filter(|id| transfer::key_target(&plan, id).is_some())
            .cloned()
            .collect();
    }
    atomic_write(&cache_path(root, &plan.id, "ocxdconf")?, bytes, 0o600)?;
    plan.file_digest = Some(sha256(bytes));
    transfer::write_incoming(root, &plan)?;
    let index = root.join("manager-state/portable-latest-incoming.json");
    safety::check_path(root, &index, true)?;
    atomic_write(&index, &serde_json::to_vec(&plan.id)?, 0o600)?;
    Ok(plan)
}
pub(super) fn remember_incoming(root: &Path, id: &str) -> ModelResult<()> {
    let index = root.join("manager-state/portable-latest-incoming.json");
    safety::check_path(root, &index, true)?;
    atomic_write(&index, &serde_json::to_vec(id)?, 0o600)?;
    Ok(())
}
#[allow(clippy::too_many_arguments)]
pub(super) fn select_asset_subset(
    root: &Path,
    original: &IncomingPlan,
    selected: &mut IncomingPlan,
    skills: &[String],
    mcps: &[String],
    mappings: &portable_assets::NameMappings,
    passphrase: Option<&str>,
) -> ModelResult<()> {
    if skills.is_empty() && mcps.is_empty() {
        if !mappings.is_empty() {
            return Err(invalid("asset_mapping", "未选择扩展时不能设置目标名称"));
        }
        return Ok(());
    }
    let summary = original
        .assets
        .as_ref()
        .ok_or_else(|| invalid("asset_scope", "原文件不含扩展资源"))?;
    if skills
        .iter()
        .any(|name| !summary.skills.iter().any(|skill| &skill.name == name))
        || mcps
            .iter()
            .any(|name| !summary.mcps.iter().any(|mcp| &mcp.name == name))
    {
        return Err(invalid("asset_scope", "所选扩展不在原清单内"));
    }
    let selected_summary = portable_assets::Summary {
        skills: summary
            .skills
            .iter()
            .filter(|skill| skills.contains(&skill.name))
            .cloned()
            .collect(),
        mcps: summary
            .mcps
            .iter()
            .filter(|mcp| mcps.contains(&mcp.name))
            .cloned()
            .collect(),
    };
    let mut objects = open_source(root, original, passphrase)?;
    let assets =
        portable_assets::remap(&selected_assets(&mut objects, &selected_summary)?, mappings)?;
    let home: PathBuf = serde_json::from_slice(&safety::read_file(
        &cache_path(root, &original.id, "asset-home.json")?,
        32768,
    )?)?;
    atomic_write(
        &cache_path(root, &selected.id, "asset-home.json")?,
        &serde_json::to_vec(&home)?,
        0o600,
    )?;
    if let Err(error) = portable_assets::prepare(root, &home, &selected.id, &assets) {
        selected.preview_error = Some(error.code);
        selected.notices.push(error.message);
    }
    selected.assets = Some(selected_summary);
    selected.asset_mappings = mappings.clone();
    if !mappings.is_empty() {
        selected.notices.push("扩展目标名称已映射；资源正文、脚本和命令参数保持原内容。内部写死的名称或路径需在使用前检查。".into());
    }
    selected.available_extension_credentials = portable_assets::secret_count(&assets) > 0;
    Ok(())
}
pub fn latest(root: &Path) -> ModelResult<Option<IncomingPlan>> {
    let index = root.join("manager-state/portable-latest-incoming.json");
    safety::check_path(root, &index, true)?;
    if !index.exists() {
        return Ok(None);
    }
    let id: String = serde_json::from_slice(&safety::read_file(&index, 1024)?)?;
    let mut plan = transfer::read_incoming(root, &id)?;
    attach_adoption(root, &mut plan)?;
    Ok(Some(plan))
}
pub(super) fn has_adoption(root: &Path, id: &str) -> ModelResult<bool> {
    Ok(cache_path(root, id, "adoption.json")?.exists())
}
fn attach_adoption(root: &Path, plan: &mut IncomingPlan) -> ModelResult<()> {
    let path = cache_path(root, &plan.id, "adoption.json")?;
    if path.exists() {
        let value: Adoption = serde_json::from_slice(&safety::read_file(&path, 4096)?)?;
        if value.id != plan.id {
            return Err(invalid("incoming_identity", "导入执行记录身份不同"));
        }
        plan.adoption = Some(transfer::ImportAdoption {
            keys: value.keys,
            webdav: value.webdav,
            completed: value.completed,
            extension_credentials: value.extension_credentials,
        });
    }
    Ok(())
}
pub fn recompare(root: &Path, id: &str, passphrase: Option<&str>) -> ModelResult<IncomingPlan> {
    let plan = transfer::read_incoming(root, id)?;
    let bytes = safety::read_file(
        &cache_path(root, id, "ocxdconf")?,
        portable_file::FILE_LIMIT as u64,
    )?;
    if plan.file_digest.as_deref() != Some(sha256(&bytes).as_str()) {
        return Err(invalid(
            "incoming_file_digest",
            "原接收文件摘要已变化，未重新比较",
        ));
    }
    let home_path = cache_path(root, id, "asset-home.json")?;
    let home: Option<PathBuf> = if home_path.exists() {
        Some(serde_json::from_slice(&safety::read_file(
            &home_path, 32768,
        )?)?)
    } else {
        None
    };
    receive_context(root, &bytes, passphrase, home.as_deref())
}
pub fn open_received(root: &Path, id: &str, passphrase: Option<&str>) -> ModelResult<IncomingPlan> {
    let mut plan = transfer::read_incoming(root, id)?;
    let bytes = safety::read_file(
        &cache_path(root, id, "ocxdconf")?,
        portable_file::FILE_LIMIT as u64,
    )?;
    if plan.file_digest.as_deref() != Some(sha256(&bytes).as_str()) {
        return Err(invalid(
            "incoming_file_digest",
            "原接收文件发生变化，未核对",
        ));
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes)?;
    if value["format"] == "OCXDCONF" {
        let opened = portable_file::open(&bytes, passphrase)?;
        let objects = decode(&opened.document)?;
        if opened.manifest.sections != sections(&objects) {
            return Err(invalid("portable_manifest", "原文件清单与载荷不一致"));
        }
    }
    attach_adoption(root, &mut plan)?;
    Ok(plan)
}
pub(super) fn copy_source(
    root: &Path,
    original: &IncomingPlan,
    selected: &mut IncomingPlan,
) -> ModelResult<()> {
    selected.protected = original.protected;
    selected.available_keys = original
        .available_keys
        .iter()
        .filter(|id| {
            transfer::key_target(selected, id).is_some()
                && selected
                    .package
                    .payload
                    .policy
                    .as_ref()
                    .is_some_and(|p| p.channels.iter().any(|c| &c.id == *id))
        })
        .cloned()
        .collect();
    if original.file_digest.is_none() {
        return Ok(());
    }
    let bytes = safety::read_file(
        &cache_path(root, &original.id, "ocxdconf")?,
        portable_file::FILE_LIMIT as u64,
    )?;
    if original.file_digest.as_deref() != Some(sha256(&bytes).as_str()) {
        return Err(invalid(
            "incoming_file_digest",
            "接收源文件发生变化，未扩大采用范围",
        ));
    }
    atomic_write(&cache_path(root, &selected.id, "ocxdconf")?, &bytes, 0o600)?;
    selected.file_digest = original.file_digest.clone();
    transfer::write_incoming(root, selected)?;
    Ok(())
}
pub fn accept(
    root: &Path,
    id: &str,
    adopt_keys: bool,
    passphrase: Option<&str>,
) -> ModelResult<Snapshot> {
    accept_manager(root, id, adopt_keys, false, passphrase)
}
pub fn accept_manager(
    root: &Path,
    id: &str,
    adopt_keys: bool,
    adopt_webdav: bool,
    passphrase: Option<&str>,
) -> ModelResult<Snapshot> {
    accept_all(
        root,
        id,
        adopt_keys,
        adopt_webdav,
        passphrase,
        &SystemCredentials,
    )
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Adoption {
    id: String,
    keys: bool,
    webdav: bool,
    completed: bool,
    #[serde(default)]
    extension_credentials: bool,
}
fn accept_all(
    root: &Path,
    id: &str,
    adopt_keys: bool,
    adopt_webdav: bool,
    passphrase: Option<&str>,
    credentials: &impl Credentials,
) -> ModelResult<Snapshot> {
    accept_domains(
        root,
        id,
        adopt_keys,
        adopt_webdav,
        false,
        passphrase,
        credentials,
    )
}
pub fn accept_extended(
    root: &Path,
    id: &str,
    adopt_keys: bool,
    adopt_webdav: bool,
    adopt_extensions: bool,
    passphrase: Option<&str>,
) -> ModelResult<Snapshot> {
    accept_domains(
        root,
        id,
        adopt_keys,
        adopt_webdav,
        adopt_extensions,
        passphrase,
        &SystemCredentials,
    )
}
fn open_source(root: &Path, plan: &IncomingPlan, passphrase: Option<&str>) -> ModelResult<Objects> {
    let bytes = safety::read_file(
        &cache_path(root, &plan.id, "ocxdconf")?,
        portable_file::FILE_LIMIT as u64,
    )?;
    if plan.file_digest.as_deref() != Some(sha256(&bytes).as_str()) {
        return Err(invalid(
            "incoming_file_digest",
            "原文件摘要变化，未读取资源",
        ));
    }
    let value: Value = serde_json::from_slice(&bytes)?;
    if value["format"] == "OCXDCONF" {
        let opened = portable_file::open(&bytes, passphrase)?;
        let objects = decode(&opened.document)?;
        if opened.manifest.secret_objects != secret_objects(&objects)
            || opened.manifest.sections != sections(&objects)
        {
            return Err(invalid("portable_manifest", "源文件清单与对象不同"));
        }
        Ok(objects)
    } else {
        let objects = decode(&bytes)?;
        if secret_objects(&objects) > 0 {
            return Err(invalid(
                "portable_protection",
                "携带秘密的资源必须来自受保护文件",
            ));
        }
        Ok(objects)
    }
}
fn selected_assets(
    objects: &mut Objects,
    summary: &portable_assets::Summary,
) -> ModelResult<Assets> {
    let original = portable_assets::summarize(&objects.assets);
    for skill in &summary.skills {
        if !original
            .skills
            .iter()
            .any(|source| source.name == skill.name && source.digest == skill.digest)
        {
            return Err(invalid("asset_identity", "Skill目标不在原文件清单内"));
        }
    }
    for mcp in &summary.mcps {
        if !original
            .mcps
            .iter()
            .any(|source| source.name == mcp.name && source.digest == mcp.digest)
        {
            return Err(invalid("asset_identity", "MCP目标不在原文件清单内"));
        }
    }
    objects
        .assets
        .skills
        .retain(|skill| summary.skills.iter().any(|item| item.name == skill.name));
    objects
        .assets
        .mcps
        .retain(|mcp| summary.mcps.iter().any(|item| item.name == mcp.name));
    Ok(objects.assets.clone())
}
fn stage_provider_materials(
    root: &Path,
    plan: &mut IncomingPlan,
    adopt_keys: bool,
    passphrase: Option<&str>,
) -> ModelResult<()> {
    let channels = plan
        .package
        .payload
        .policy
        .as_ref()
        .map(|p| p.channels.clone())
        .unwrap_or_default();
    if !channels
        .iter()
        .any(|c| c.connection.as_ref().is_some_and(|v| v.provider.is_some()))
    {
        return Ok(());
    }
    let objects = open_source(root, plan, passphrase)?;
    let file = cache_path(root, &plan.id, "provider-bindings.json")?;
    let mut bindings: BTreeMap<String, ImportedProviderRef> = if file.exists() {
        serde_json::from_slice(&safety::read_file(&file, 1024 * 1024)?)?
    } else {
        BTreeMap::new()
    };
    let mut changed = false;
    for source in channels {
        let Some(public) = source.connection.as_ref().and_then(|c| c.provider.as_ref()) else {
            continue;
        };
        let Some(target) = transfer::key_target(plan, &source.id).map(str::to_owned) else {
            continue;
        };
        let Some(channel) = plan
            .merged_policy
            .as_mut()
            .and_then(|p| p.channels.iter_mut().find(|c| c.id == target))
        else {
            return Err(invalid("provider_import_scope", "原渠道接收目标缺失"));
        };
        let original = objects
            .models
            .payload
            .policy
            .iter()
            .flat_map(|p| &p.channels)
            .find(|c| c.id == source.id)
            .ok_or_else(|| invalid("provider_import_scope", "原文件缺少所选源渠道"))?;
        if !transfer::is_scoped_channel(original, &source)? {
            return Err(invalid("provider_import_scope", "原字段不属于所选源渠道"));
        }
        if source.model_subset.is_some()
            && ModelStore::new(root)
                .load()?
                .policy
                .channels
                .iter()
                .any(|c| c.id == target)
        {
            continue;
        }
        let reference = if let Some(reference) = bindings.get(&target) {
            let staged = super::provider_import::load(root, channel, reference)?;
            if staged.public != *public
                || staged.authorization.is_some()
                    != (adopt_keys && objects.channel_authorizations.contains_key(&source.id))
            {
                return Err(invalid(
                    "provider_import_scope",
                    "原渠道接收范围已冻结，请沿原操作核对",
                ));
            }
            reference.clone()
        } else {
            let authorization = if adopt_keys {
                objects.channel_authorizations.get(&source.id)
            } else {
                None
            };
            let reference = super::provider_import::stage(root, channel, public, authorization)?;
            bindings.insert(target, reference.clone());
            atomic_write(&file, &serde_json::to_vec(&bindings)?, 0o600)?;
            reference
        };
        channel.connection.imported_config = Some(reference);
        channel.connection.auth_mode = public["authMode"].as_str().map(str::to_owned);
        channel.connection.enabled = false;
        changed = true;
    }
    if changed {
        transfer::write_incoming(root, plan)?;
    }
    Ok(())
}
#[allow(clippy::too_many_arguments)]
fn accept_domains(
    root: &Path,
    id: &str,
    adopt_keys: bool,
    adopt_webdav: bool,
    adopt_extensions: bool,
    passphrase: Option<&str>,
    credentials: &impl Credentials,
) -> ModelResult<Snapshot> {
    let mut plan = transfer::read_incoming(root, id)?;
    if let Some(error) = &plan.preview_error {
        return Err(invalid(error, "所选配置或资源仍有冲突，请比较后再采用"));
    }
    let record_path = cache_path(root, id, "adoption.json")?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&record_path),
        true,
    )?;
    let _guard = crate::infrastructure::locking::TargetFileLock::lock(&record_path)?;
    let resuming = record_path.exists();
    let mut record = if resuming {
        let record: Adoption = serde_json::from_slice(&safety::read_file(&record_path, 4096)?)?;
        if record.id != id
            || record.keys != adopt_keys
            || record.webdav != adopt_webdav
            || record.extension_credentials != adopt_extensions
        {
            return Err(invalid(
                "import_replay_conflict",
                "原导入的凭据采用范围已冻结，请沿原操作核对",
            ));
        }
        record
    } else {
        Adoption {
            id: id.into(),
            keys: adopt_keys,
            webdav: adopt_webdav,
            completed: false,
            extension_credentials: adopt_extensions,
        }
    };
    if record.completed {
        return ModelStore::new(root).load();
    }
    if !transfer::accepted(root, &plan)? {
        let local = ModelStore::new(root).load()?;
        if (
            local.library_digest,
            local.policy_digest,
            local.workspace_digest,
        ) != (
            plan.expected_library.clone(),
            plan.expected_policy.clone(),
            plan.expected_workspace.clone(),
        ) {
            let bindings_path = cache_path(root, id, "bindings.json")?;
            let bindings: BTreeMap<String, String> = if adopt_keys && bindings_path.exists() {
                serde_json::from_slice(&safety::read_file(&bindings_path, 1024 * 1024)?)?
            } else {
                BTreeMap::new()
            };
            if !resuming || !transfer::intended_applied(root, &plan, &bindings)? {
                return Err(invalid(
                    "stale_baseline",
                    "采用前模型基准改变，未安装导入凭据",
                ));
            }
        }
    }
    if portable_manager::exists(root, id)? {
        portable_manager::verify(root, id)?;
    }
    let assets = if let Some(summary) = &plan.assets {
        if summary.skills.is_empty() && summary.mcps.is_empty() {
            None
        } else {
            let mut objects = open_source(root, &plan, passphrase)?;
            let assets = portable_assets::remap(
                &selected_assets(&mut objects, summary)?,
                &plan.asset_mappings,
            )?;
            portable_assets::verify(root, id, &assets)?;
            Some(assets)
        }
    } else {
        None
    };
    if adopt_extensions && !plan.available_extension_credentials {
        return Err(invalid(
            "asset_credentials",
            "原文件的所选资源未包含扩展凭据",
        ));
    }
    if adopt_webdav {
        if plan.available_webdav_passwords.is_empty() {
            return Err(invalid("credential_scope", "已选连接不含可采用密码"));
        }
        let bytes = safety::read_file(
            &cache_path(root, id, "ocxdconf")?,
            portable_file::FILE_LIMIT as u64,
        )?;
        if plan.file_digest.as_deref() != Some(sha256(&bytes).as_str()) {
            return Err(invalid(
                "incoming_file_digest",
                "源文件已变化，未采用连接密码",
            ));
        }
        let opened = portable_file::open(&bytes, passphrase)?;
        let objects = decode(&opened.document)?;
        if !opened.protected || opened.manifest.sections != sections(&objects) {
            return Err(invalid("portable_protection", "连接密码未通过保护核验"));
        }
        let bindings_path = cache_path(root, id, "webdav-bindings.json")?;
        let mut bindings: BTreeMap<String, String> = if bindings_path.exists() {
            serde_json::from_slice(&safety::read_file(&bindings_path, 1024 * 1024)?)?
        } else {
            BTreeMap::new()
        };
        for endpoint in plan
            .manager
            .as_ref()
            .into_iter()
            .flat_map(|manager| manager.webdav.iter())
        {
            if objects.webdav_passwords.0.contains_key(&endpoint.id) {
                if !objects
                    .manager
                    .webdav
                    .iter()
                    .any(|source| source == endpoint)
                {
                    return Err(invalid("credential_scope", "连接密码源身份与所选连接不同"));
                }
                let reference = bindings
                    .entry(endpoint.id.clone())
                    .or_insert_with(crate::infrastructure::keychain::new_ref_id);
                crate::infrastructure::keychain::account_key(
                    crate::infrastructure::keychain::WEBDAV_CREDENTIAL_PURPOSE,
                    reference,
                )?;
            }
        }
        atomic_write(&bindings_path, &serde_json::to_vec(&bindings)?, 0o600)?;
        for (endpoint, reference) in &bindings {
            let value = objects
                .webdav_passwords
                .0
                .get(endpoint)
                .ok_or_else(|| invalid("credential_scope", "导入连接密码不在原范围"))?;
            credentials.install_webdav(reference, value)?;
        }
        portable_manager::bind_passwords(root, id, &bindings)?;
    }
    atomic_write(&record_path, &serde_json::to_vec(&record)?, 0o600)?;
    stage_provider_materials(root, &mut plan, adopt_keys, passphrase)?;
    let result = accept_with(root, id, adopt_keys, passphrase, credentials)?;
    if let Some(assets) = &assets {
        portable_assets::apply(root, id, assets, adopt_extensions).map_err(|error| {
            invalid(
                "portable_partial",
                format!(
                    "模型对象已接收，扩展资源尚未全部完成：{}。请沿原操作核对",
                    error.message
                ),
            )
        })?;
    }
    portable_manager::apply(root, id).map_err(|error| {
        invalid(
            "portable_partial",
            format!(
                "模型对象已接收，管理器配置未全部完成：{}。请核对原导入，不要新建重复操作",
                error.message
            ),
        )
    })?;
    record.completed = true;
    atomic_write(&record_path, &serde_json::to_vec(&record)?, 0o600)?;
    Ok(result)
}
fn accept_with(
    root: &Path,
    id: &str,
    adopt_keys: bool,
    passphrase: Option<&str>,
    credentials: &impl Credentials,
) -> ModelResult<Snapshot> {
    if !adopt_keys {
        return transfer::accept(root, id);
    }
    let plan = transfer::read_incoming(root, id)?;
    if plan.available_keys.is_empty() {
        return Err(invalid("credential_scope", "已选接收范围中没有渠道Key"));
    }
    if let Some(error) = &plan.preview_error {
        return Err(invalid(error, "所选接收配置仍有冲突，未写入凭据"));
    }
    let bindings_path = cache_path(root, id, "bindings.json")?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&bindings_path),
        true,
    )?;
    let _guard = crate::infrastructure::locking::TargetFileLock::lock(&bindings_path)?;
    let bytes = safety::read_file(
        &cache_path(root, id, "ocxdconf")?,
        portable_file::FILE_LIMIT as u64,
    )?;
    if plan.file_digest.as_deref() != Some(sha256(&bytes).as_str()) {
        return Err(invalid(
            "incoming_file_digest",
            "接收源文件已变化，未采用凭据",
        ));
    }
    let opened = portable_file::open(&bytes, passphrase)?;
    let objects = decode(&opened.document)?;
    if !opened.protected || opened.manifest.secret_objects != secret_objects(&objects) {
        return Err(invalid("portable_protection", "凭据载荷未通过文件保护核验"));
    }
    let mut wanted = BTreeMap::new();
    for channel in plan
        .package
        .payload
        .policy
        .as_ref()
        .into_iter()
        .flat_map(|p| p.channels.iter())
    {
        if !plan.available_keys.contains(&channel.id) {
            continue;
        }
        let Some(target) = transfer::key_target(&plan, &channel.id) else {
            continue;
        };
        if let Some(value) = objects.channel_keys.0.get(&channel.id) {
            if !objects.models.payload.policy.as_ref().is_some_and(|p| {
                p.channels.iter().any(|original| {
                    original.id == channel.id
                        && transfer::is_scoped_channel(original, channel).unwrap_or(false)
                })
            }) {
                return Err(invalid("credential_scope", "凭据源渠道与所选配置不一致"));
            }
            wanted.insert(target.to_string(), value);
        }
    }
    if wanted.is_empty() {
        if plan
            .package
            .payload
            .policy
            .iter()
            .flat_map(|p| &p.channels)
            .any(|c| {
                plan.available_keys.contains(&c.id)
                    && transfer::key_target(&plan, &c.id).is_some()
                    && objects.channel_authorizations.contains_key(&c.id)
            })
        {
            return transfer::accept_with_credentials(root, id, &BTreeMap::new());
        }
        return Err(invalid("credential_scope", "没有与所选渠道匹配的Key"));
    }
    let mut bindings: BTreeMap<String, String> = if bindings_path.exists() {
        serde_json::from_slice(&safety::read_file(&bindings_path, 1024 * 1024)?)?
    } else {
        BTreeMap::new()
    };
    for reference in bindings.values() {
        require_uuid(reference)?;
    }
    bindings.retain(|channel, _| wanted.contains_key(channel));
    for channel in wanted.keys() {
        bindings
            .entry(channel.clone())
            .or_insert_with(|| uuid::Uuid::new_v4().to_string());
    }
    if transfer::verify_acceptance(root, &plan, &bindings)? {
        return transfer::accept_with_credentials(root, id, &bindings);
    }
    let local = ModelStore::new(root).load()?;
    if (
        local.library_digest,
        local.policy_digest,
        local.workspace_digest,
    ) != (
        plan.expected_library.clone(),
        plan.expected_policy.clone(),
        plan.expected_workspace.clone(),
    ) {
        for (channel, value) in &wanted {
            if credentials.read(&bindings[channel])?.as_str() != value.as_str() {
                return Err(invalid(
                    "credential_readback",
                    "原导入凭据未读回确认，保留待核对",
                ));
            }
        }
        return transfer::accept_with_credentials(root, id, &bindings);
    }
    atomic_write(&bindings_path, &serde_json::to_vec(&bindings)?, 0o600)?;
    let result = (|| {
        for (channel, value) in wanted {
            credentials.install(&bindings[&channel], value)?;
        }
        transfer::accept_with_credentials(root, id, &bindings)
    })();
    if result.is_err() {
        if let Ok(current) = ModelStore::new(root).load() {
            for reference in bindings.values() {
                let used = current
                    .policy
                    .channels
                    .iter()
                    .chain(
                        current
                            .workspace
                            .as_ref()
                            .into_iter()
                            .flat_map(|w| w.policy.channels.iter()),
                    )
                    .any(|c| c.connection.credential_ref.as_ref() == Some(reference));
                if !used {
                    let _ = credentials.remove(reference);
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn protected_original_channel_model_subset_uses_full_source_auth_and_only_selected_public_fields(
    ) {
        use super::super::upstream::UpstreamSnapshot;
        let source = tempfile::tempdir().unwrap();
        let upstream = tempfile::tempdir().unwrap();
        let receiver = tempfile::tempdir().unwrap();
        let public = serde_json::json!({"adapter":"openai-chat","authMode":"key","baseUrl":"https://source.example.test/v1","models":["a","b"],"selectedModels":["a","b"],"modelAliases":{"a":"alias-a","b":"alias-b"},"modelDisplayNames":{"a":"A","b":"B"},"note":"shared source"});
        let mut raw = public.clone();
        raw["apiKey"] = serde_json::json!("fixture-portable-subset-SECRET");
        raw["headers"] = serde_json::json!({"X-Auth":"fixture-header-SECRET"});
        std::fs::write(
            upstream.path().join("config.json"),
            serde_json::to_vec(
                &serde_json::json!({"providers":{"p":raw,"unrelated":{"unparsed":"irrelevant"}}}),
            )
            .unwrap(),
        )
        .unwrap();
        let snapshot = UpstreamSnapshot {
            version: "2.84.0".into(),
            providers: serde_json::json!({"defaultProvider":"p","providers":{"p":public}}),
            models: serde_json::json!([{"provider":"p","id":"a","namespaced":"p/a"},{"provider":"p","id":"b","namespaced":"p/b"}]),
            order: serde_json::json!({"pickerOrder":["p/a","p/b"]}),
            catalog: serde_json::json!({"models":[{"slug":"p/a"},{"slug":"p/b"}]}),
        };
        let id = super::super::portable_channels::catalog(source.path(), &snapshot).unwrap()[0]
            .id
            .clone();
        let plan = transfer::prepare_file_scope(
            source.path(),
            transfer::ExportScope {
                channel_models: BTreeMap::from([(id.clone(), vec!["a".into()])]),
                include_policy: true,
                channel_ids: vec![id.clone()],
                template_refs: vec![],
                include_global: false,
            },
            Some("fixture-subset-password"),
            true,
            &ManagerScope {
                upstream_root: Some(upstream.path().into()),
                upstream_snapshot: Some(snapshot),
                ..Default::default()
            },
        )
        .unwrap();
        let bytes = transfer::file_bytes(source.path(), &plan.id).unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains("SECRET"));
        assert_eq!(plan.credential_count, 1);
        let package = &plan.package;
        let channel = &package.payload.policy.as_ref().unwrap().channels[0];
        assert_eq!(channel.model_subset, Some(vec!["a".into()]));
        assert_eq!(channel.discovered.len(), 1);
        let fields = channel
            .connection
            .as_ref()
            .unwrap()
            .provider
            .as_ref()
            .unwrap();
        assert_eq!(fields["models"], serde_json::json!(["a"]));
        assert!(fields["modelAliases"].get("b").is_none());
        let incoming = receive(receiver.path(), &bytes, Some("fixture-subset-password")).unwrap();
        assert_eq!(incoming.available_keys, vec![id.clone()]);
        let selected = transfer::select_received_context(
            receiver.path(),
            &incoming.id,
            transfer::ReceiveScope {
                channel_ids: vec![id.clone()],
                channel_models: BTreeMap::from([(id.clone(), vec![])]),
                ..Default::default()
            },
            Some("fixture-subset-password"),
        )
        .unwrap();
        let result = accept(
            receiver.path(),
            &selected.id,
            false,
            Some("fixture-subset-password"),
        )
        .unwrap();
        let target = &result.workspace.unwrap().policy.channels[0];
        assert!(target.discovered.is_empty());
        assert!(!target.connection.enabled);
        let staged = super::super::provider_import::load(
            receiver.path(),
            target,
            target.connection.imported_config.as_ref().unwrap(),
        )
        .unwrap();
        assert!(staged.authorization.is_none());
        assert_eq!(staged.public["models"], serde_json::json!([]));
    }
    use super::*;
    use std::{collections::BTreeSet, sync::Mutex};
    #[test]
    fn partial_remote_original_config_updates_keep_bound_private_pool_and_refuse_endpoint_retargeting(
    ) {
        use super::super::upstream::UpstreamSnapshot;
        let root = tempfile::tempdir().unwrap();
        let snapshot = UpstreamSnapshot {
            version: "2.84.0".into(),
            providers: serde_json::json!({"defaultProvider":"p","providers":{"p":{"adapter":"openai-chat","baseUrl":"https://source.example.test/v1","models":["a"],"newModelPolicy":"off","note":"before"}}}),
            models: serde_json::json!([{"provider":"p","id":"a","namespaced":"p/a"}]),
            order: serde_json::json!({"pickerOrder":["p/a"]}),
            catalog: serde_json::json!({"models":[{"slug":"p/a"}]}),
        };
        let id = super::super::portable_channels::catalog(root.path(), &snapshot).unwrap()[0]
            .id
            .clone();
        let package = super::super::portable_channels::export(
            root.path(),
            transfer::ExportScope {
                channel_models: Default::default(),
                include_policy: true,
                channel_ids: vec![id.clone()],
                template_refs: vec![],
                include_global: false,
            },
            &snapshot,
        )
        .unwrap();
        let key = "fixture-remote-portable-SECRET";
        let digest = sha256(key.as_bytes());
        let key_id = &digest.trim_start_matches("sha256:")[..8];
        let auth:super::super::provider_auth::PortableAuthorization=serde_json::from_value(serde_json::json!({"activeId":key_id,"fields":{"apiKeyPool":[{"id":key_id,"key":key,"label":"original","addedAt":100}],"headers":{"X-Auth":"fixture-header-SECRET"}}})).unwrap();
        let remote = Objects {
            format: "OCXD_CONFIG_OBJECTS".into(),
            version: 2,
            models: package.clone(),
            channel_keys: Default::default(),
            channel_authorizations: BTreeMap::from([(id.clone(), auth)]),
            manager: Default::default(),
            webdav_passwords: Default::default(),
            assets: Default::default(),
        };
        fn bytes(objects: &Objects) -> Vec<u8> {
            portable_file::seal(
                portable_file::Manifest {
                    id: uuid::Uuid::new_v4().to_string(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                    sections: sections(objects),
                    secret_objects: secret_objects(objects),
                    document_digest: String::new(),
                },
                &Zeroizing::new(serde_json::to_vec(objects).unwrap()),
                Some("fixture-remote-password"),
            )
            .unwrap()
        }
        let remote_bytes = bytes(&remote);
        let mut local = Objects {
            format: "OCXD_CONFIG_OBJECTS".into(),
            version: 1,
            models: package,
            channel_keys: Default::default(),
            channel_authorizations: Default::default(),
            manager: Default::default(),
            webdav_passwords: Default::default(),
            assets: Default::default(),
        };
        local.models.payload.policy.as_mut().unwrap().channels[0]
            .connection
            .as_mut()
            .unwrap()
            .provider
            .as_mut()
            .unwrap()["note"] = serde_json::json!("later nonsecret edit");
        local.models.digest = sha256(&serde_json::to_vec(&local.models.payload).unwrap());
        let merged = merge_remote(
            &remote_bytes,
            &bytes(&local),
            Some("fixture-remote-password"),
            &uuid::Uuid::new_v4().to_string(),
        )
        .unwrap();
        let (opened, protected) = remote_objects(&merged, Some("fixture-remote-password")).unwrap();
        assert!(protected);
        assert_eq!(secret_objects(&opened), 1);
        assert_eq!(
            super::super::portable_channels::provider(&opened.models, &id).unwrap()["note"],
            "later nonsecret edit"
        );
        assert!(serde_json::to_string(&opened.channel_authorizations)
            .unwrap()
            .contains(key));
        assert!(!String::from_utf8_lossy(&merged).contains(key));
        let channel = &mut local.models.payload.policy.as_mut().unwrap().channels[0];
        channel.connection.as_mut().unwrap().nodes[0].url =
            "https://different.example.test/v1".into();
        channel
            .connection
            .as_mut()
            .unwrap()
            .provider
            .as_mut()
            .unwrap()["baseUrl"] = serde_json::json!("https://different.example.test/v1");
        local.models.digest = sha256(&serde_json::to_vec(&local.models.payload).unwrap());
        assert_eq!(
            merge_remote(
                &remote_bytes,
                &bytes(&local),
                Some("fixture-remote-password"),
                &uuid::Uuid::new_v4().to_string()
            )
            .unwrap_err()
            .code,
            "channel_credential_binding"
        );
    }
    #[test]
    fn named_asset_conflicts_can_be_remapped_with_settings_without_overwriting_existing_sources() {
        use crate::modules::{
            extensions::{
                self,
                projection::{ExtensionProjection, CONFIG_RELATIVE_PATH},
                ProjectionFingerprint,
            },
            sync::portable_assets::NameMappings,
        };
        let source = tempfile::tempdir().unwrap();
        let source_home = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let target_home = tempfile::tempdir().unwrap();
        for (home, body, command) in [
            (source_home.path(), "# incoming fixture", "incoming-command"),
            (target_home.path(), "# local must remain", "local-command"),
        ] {
            std::fs::create_dir_all(home.join(".agents/skills/fixture")).unwrap();
            std::fs::write(home.join(".agents/skills/fixture/SKILL.md"), body).unwrap();
            std::fs::create_dir_all(home.join(".codex")).unwrap();
            std::fs::write(
                home.join(".codex/config.toml"),
                format!("[mcp_servers.fixture]\ncommand = '{command}'\n"),
            )
            .unwrap();
        }
        let mut source_config = extensions::ExtensionConfig {
            skills: vec!["fixture".into()],
            servers: vec!["fixture".into()],
            ..Default::default()
        };
        source_config
            .skill_targets
            .insert("fixture".into(), vec![extensions::ClientId::Codex]);
        let write_config = |root: &Path, config: extensions::ExtensionConfig| {
            let value = ExtensionProjection {
                fingerprint: ProjectionFingerprint::create(
                    config.fingerprint(),
                    chrono::Utc::now(),
                ),
                config,
            };
            atomic_write(
                &root.join(CONFIG_RELATIVE_PATH),
                &serde_json::to_vec(&value).unwrap(),
                0o600,
            )
            .unwrap();
        };
        write_config(source.path(), source_config);
        write_config(
            target.path(),
            extensions::ExtensionConfig {
                skills: vec!["fixture".into(), "local-other".into()],
                servers: vec!["fixture".into()],
                ..Default::default()
            },
        );
        let plan = transfer::prepare_file_scope(
            source.path(),
            transfer::ExportScope {
                channel_models: Default::default(),
                include_policy: false,
                channel_ids: vec![],
                template_refs: vec![],
                include_global: false,
            },
            None,
            false,
            &ManagerScope {
                extensions: true,
                skill_names: vec!["fixture".into()],
                mcp_names: vec!["fixture".into()],
                assets_reviewed: true,
                local_home: Some(source_home.path().into()),
                ..Default::default()
            },
        )
        .unwrap();
        let bytes = transfer::file_bytes(source.path(), &plan.id).unwrap();
        let incoming =
            receive_context(target.path(), &bytes, None, Some(target_home.path())).unwrap();
        assert_eq!(incoming.preview_error.as_deref(), Some("asset_conflict"));
        let selected = transfer::select_received(
            target.path(),
            &incoming.id,
            transfer::ReceiveScope {
                extensions: true,
                skill_names: vec!["fixture".into()],
                mcp_names: vec!["fixture".into()],
                asset_mappings: NameMappings {
                    skills: BTreeMap::from([("fixture".into(), "received-skill".into())]),
                    mcps: BTreeMap::from([("fixture".into(), "received-mcp".into())]),
                },
                ..Default::default()
            },
        )
        .unwrap();
        assert!(selected.preview_error.is_none());
        let reselected = transfer::select_received(
            target.path(),
            &selected.id,
            transfer::ReceiveScope {
                extensions: true,
                skill_names: vec!["fixture".into()],
                mcp_names: vec!["fixture".into()],
                asset_mappings: NameMappings {
                    skills: BTreeMap::from([("fixture".into(), "second-target".into())]),
                    mcps: BTreeMap::from([("fixture".into(), "second-mcp".into())]),
                },
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            reselected
                .manager
                .as_ref()
                .unwrap()
                .extensions
                .as_ref()
                .unwrap()
                .skills,
            vec!["second-target"]
        );
        assert!(reselected
            .manager
            .as_ref()
            .unwrap()
            .extensions
            .as_ref()
            .unwrap()
            .skill_targets
            .contains_key("second-target"));
        assert_eq!(selected.file_digest, incoming.file_digest);
        assert_eq!(selected.assets.as_ref().unwrap().skills[0].name, "fixture");
        let manager = selected
            .manager
            .as_ref()
            .unwrap()
            .extensions
            .as_ref()
            .unwrap();
        assert_eq!(manager.skills, vec!["received-skill"]);
        assert!(manager.skill_targets.contains_key("received-skill"));
        accept_extended(target.path(), &selected.id, false, false, false, None).unwrap();
        assert_eq!(
            std::fs::read_to_string(
                target
                    .path()
                    .join("manager-state/skills-store/received-skill/SKILL.md")
            )
            .unwrap(),
            "# incoming fixture"
        );
        assert_eq!(
            std::fs::read_to_string(target_home.path().join(".agents/skills/fixture/SKILL.md"))
                .unwrap(),
            "# local must remain"
        );
        assert!(
            std::fs::read_to_string(target_home.path().join(".codex/config.toml"))
                .unwrap()
                .contains("local-command")
        );
        assert_eq!(
            portable_assets::stored_names(target.path()).unwrap(),
            vec!["received-mcp"]
        );
        let applied: ExtensionProjection = serde_json::from_slice(
            &std::fs::read(target.path().join(CONFIG_RELATIVE_PATH)).unwrap(),
        )
        .unwrap();
        assert!(
            applied.config.skills.contains(&"fixture".into())
                && applied.config.skills.contains(&"local-other".into())
                && applied.config.skills.contains(&"received-skill".into())
        );
        assert!(
            applied.config.servers.contains(&"fixture".into())
                && applied.config.servers.contains(&"received-mcp".into())
        );
        std::fs::write(
            target
                .path()
                .join("manager-state/skills-store/received-skill/SKILL.md"),
            "# later edit",
        )
        .unwrap();
        accept_extended(target.path(), &selected.id, false, false, false, None).unwrap();
        assert_eq!(
            std::fs::read_to_string(
                target
                    .path()
                    .join("manager-state/skills-store/received-skill/SKILL.md")
            )
            .unwrap(),
            "# later edit"
        );
        assert_eq!(
            transfer::select_received(
                target.path(),
                &selected.id,
                transfer::ReceiveScope {
                    skill_names: vec!["fixture".into()],
                    ..Default::default()
                }
            )
            .unwrap_err()
            .code,
            "import_in_progress"
        );
    }
    #[test]
    fn account_info_round_trip_selects_only_requested_identity_without_login_material() {
        use crate::modules::sync::portable_accounts;
        let source = tempfile::tempdir().unwrap();
        let upstream = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        std::fs::write(upstream.path().join("auth.json"), serde_json::to_vec(&serde_json::json!({
            "google-antigravity": {"accounts":[
                {"id":"a","credential":{"accountId":"subject-a","email":"a@example.test","refresh":"NEVER-EXPORT-SECRET"}},
                {"id":"b","credential":{"accountId":"subject-b","email":"b@example.test","access":"NEVER-EXPORT-SECRET"}}
            ]}
        })).unwrap()).unwrap();
        let catalog = portable_accounts::catalog(source.path(), upstream.path()).unwrap();
        let scope = ManagerScope {
            account_ids: catalog.iter().map(|a| a.account.id.clone()).collect(),
            upstream_root: Some(upstream.path().to_path_buf()),
            ..Default::default()
        };
        let plan = transfer::prepare_file_scope(
            source.path(),
            transfer::ExportScope {
                channel_models: Default::default(),
                include_policy: false,
                channel_ids: vec![],
                template_refs: vec![],
                include_global: false,
            },
            None,
            false,
            &scope,
        )
        .unwrap();
        let bytes = transfer::file_bytes(source.path(), &plan.id).unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains("NEVER-EXPORT-SECRET"));
        let opened = portable_file::open(&bytes, None).unwrap();
        let mut objects: Objects = serde_json::from_slice(&opened.document).unwrap();
        assert_eq!(objects.version, 3);
        assert_eq!(
            objects.manager.accounts[0]
                .origin
                .as_ref()
                .unwrap()
                .account_platform,
            "antigravity"
        );
        objects.version = 2;
        assert_eq!(
            decode(&serde_json::to_vec(&objects).unwrap())
                .err()
                .unwrap()
                .code,
            "portable_objects"
        );
        objects.version = 3;
        objects.manager.accounts[0].origin.as_mut().unwrap().owner = "platform-native".into();
        let mut manifest = opened.manifest;
        manifest.id = uuid::Uuid::new_v4().to_string();
        let unsupported_bytes =
            portable_file::seal(manifest, &serde_json::to_vec(&objects).unwrap(), None).unwrap();
        let isolated = tempfile::tempdir().unwrap();
        let unavailable = receive(isolated.path(), &unsupported_bytes, None).unwrap();
        assert_eq!(
            unavailable.preview_error.as_deref(),
            Some("account_capability")
        );
        assert!(portable_accounts::imported(isolated.path())
            .unwrap()
            .is_empty());
        assert_eq!(
            transfer::select_received(
                isolated.path(),
                &unavailable.id,
                transfer::ReceiveScope {
                    account_ids: vec![catalog[0].account.id.clone()],
                    ..Default::default()
                }
            )
            .unwrap_err()
            .code,
            "account_capability"
        );
        let available_only = transfer::select_received(
            isolated.path(),
            &unavailable.id,
            transfer::ReceiveScope {
                account_ids: vec![catalog[1].account.id.clone()],
                ..Default::default()
            },
        )
        .unwrap();
        assert!(available_only.preview_error.is_none());
        accept_extended(
            isolated.path(),
            &available_only.id,
            false,
            false,
            false,
            None,
        )
        .unwrap();
        assert_eq!(
            portable_accounts::imported(isolated.path()).unwrap(),
            vec![catalog[1].account.clone()]
        );
        let incoming = receive(target.path(), &bytes, None).unwrap();
        assert_eq!(incoming.manager.as_ref().unwrap().accounts.len(), 2);
        assert!(portable_accounts::imported(target.path())
            .unwrap()
            .is_empty());
        let selected = transfer::select_received(
            target.path(),
            &incoming.id,
            transfer::ReceiveScope {
                account_ids: vec![catalog[0].account.id.clone()],
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(selected.manager.as_ref().unwrap().accounts.len(), 1);
        accept_extended(target.path(), &selected.id, false, false, false, None).unwrap();
        assert_eq!(
            portable_accounts::imported(target.path()).unwrap(),
            vec![catalog[0].account.clone()]
        );
        // A completed replay must not replace later edits, even for metadata.
        let mut later = portable_accounts::imported(target.path()).unwrap();
        later[0].label = "后来修改".into();
        std::fs::write(
            target.path().join(portable_accounts::IMPORTED),
            serde_json::to_vec(&later).unwrap(),
        )
        .unwrap();
        accept_extended(target.path(), &selected.id, false, false, false, None).unwrap();
        assert_eq!(portable_accounts::imported(target.path()).unwrap(), later);
        assert!(!target.path().join("auth.json").exists());
        assert!(!target.path().join("codex-accounts.json").exists());
    }
    #[test]
    fn unified_file_carries_skill_resources_and_mcp_definitions_with_explicit_subset_adoption() {
        let source = tempfile::tempdir().unwrap();
        let source_home = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let target_home = tempfile::tempdir().unwrap();
        let skill = source_home.path().join(".agents/skills/fixture");
        std::fs::create_dir_all(skill.join("scripts")).unwrap();
        std::fs::write(skill.join("SKILL.md"), "# Fixture\nUse scripts/run.py").unwrap();
        std::fs::write(skill.join("scripts/run.py"), "print('fixture')").unwrap();
        std::fs::create_dir(source_home.path().join(".codex")).unwrap();
        std::fs::write(
            source_home.path().join(".codex/config.toml"),
            "[mcp_servers.fixture]\ncommand = 'fixture-command'\n",
        )
        .unwrap();
        let package = transfer::export(
            source.path(),
            transfer::ExportScope {
                channel_models: Default::default(),
                include_policy: false,
                channel_ids: vec![],
                template_refs: vec![],
                include_global: false,
            },
        )
        .unwrap();
        let (bytes, _, _, summary) = seal_scope(
            source.path(),
            &uuid::Uuid::new_v4().to_string(),
            &package,
            false,
            None,
            &ManagerScope {
                skill_names: vec!["fixture".into()],
                mcp_names: vec!["fixture".into()],
                assets_reviewed: true,
                local_home: Some(source_home.path().into()),
                ..Default::default()
            },
            &Memory::default(),
        )
        .unwrap();
        assert_eq!(summary.skills[0].files, 2);
        assert_eq!(summary.mcps.len(), 1);
        let incoming =
            receive_context(target.path(), &bytes, None, Some(target_home.path())).unwrap();
        assert!(incoming.preview_error.is_none());
        assert!(incoming.assets.as_ref().unwrap().skills[0].files == 2);
        let visible = serde_json::to_string(&incoming).unwrap();
        assert!(!visible.contains("print('fixture')"));
        assert!(!target
            .path()
            .join("manager-state/skills-store/fixture")
            .exists());
        let selected = transfer::select_received_context(
            target.path(),
            &incoming.id,
            transfer::ReceiveScope {
                skill_names: vec!["fixture".into()],
                ..Default::default()
            },
            None,
        )
        .unwrap();
        assert!(selected.assets.as_ref().unwrap().mcps.is_empty());
        accept_extended(target.path(), &selected.id, false, false, false, None).unwrap();
        assert_eq!(
            std::fs::read_to_string(
                target
                    .path()
                    .join("manager-state/skills-store/fixture/scripts/run.py")
            )
            .unwrap(),
            "print('fixture')"
        );
        assert!(portable_assets::stored_names(target.path())
            .unwrap()
            .is_empty());
        assert!(!target_home.path().join(".codex").exists());
        let other = tempfile::tempdir().unwrap();
        let full = receive_context(other.path(), &bytes, None, Some(target_home.path())).unwrap();
        accept_extended(other.path(), &full.id, false, false, false, None).unwrap();
        assert_eq!(
            portable_assets::stored_names(other.path()).unwrap(),
            vec!["fixture"]
        );
        assert!(!target_home.path().join(".codex").exists());
    }
    #[test]
    fn protected_remote_skill_files_survive_an_upload_that_did_not_select_extension_credentials() {
        let root = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        let skill = home.path().join(".agents/skills/fixture");
        std::fs::create_dir_all(&skill).unwrap();
        std::fs::write(skill.join("SKILL.md"), "fixture").unwrap();
        std::fs::write(skill.join(".env"), "FIXTURE_PASSWORD=fixture-secret").unwrap();
        let package = transfer::export(
            root.path(),
            transfer::ExportScope {
                channel_models: Default::default(),
                include_policy: false,
                channel_ids: vec![],
                template_refs: vec![],
                include_global: false,
            },
        )
        .unwrap();
        let mut scope = ManagerScope {
            skill_names: vec!["fixture".into()],
            assets_reviewed: true,
            extension_credentials: true,
            local_home: Some(home.path().into()),
            ..Default::default()
        };
        let (remote, _, _, _) = seal_scope(
            root.path(),
            &uuid::Uuid::new_v4().to_string(),
            &package,
            false,
            Some("fixture-password"),
            &scope,
            &Memory::default(),
        )
        .unwrap();
        scope.extension_credentials = false;
        let (local, _, _, _) = seal_scope(
            root.path(),
            &uuid::Uuid::new_v4().to_string(),
            &package,
            false,
            Some("fixture-password"),
            &scope,
            &Memory::default(),
        )
        .unwrap();
        let merged = merge_remote(
            &remote,
            &local,
            Some("fixture-password"),
            &uuid::Uuid::new_v4().to_string(),
        )
        .unwrap();
        let (objects, _) = remote_objects(&merged, Some("fixture-password")).unwrap();
        assert!(objects.assets.skills[0]
            .files
            .iter()
            .any(|file| file.path == ".env" && file.sensitive));
        assert_eq!(secret_objects(&objects), 1);
    }
    #[test]
    fn a_half_applied_unified_import_resumes_the_same_receipt_and_does_not_duplicate_connections() {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        crate::modules::preferences::PreferencesStore::new(source.path())
            .save(&crate::modules::preferences::Preferences {
                interface_scale: 125,
                ..Default::default()
            })
            .unwrap();
        let package = transfer::export(
            source.path(),
            transfer::ExportScope {
                channel_models: Default::default(),
                include_policy: false,
                channel_ids: vec![],
                template_refs: vec![],
                include_global: false,
            },
        )
        .unwrap();
        let (bytes, _, _, _) = seal_scope(
            source.path(),
            &uuid::Uuid::new_v4().to_string(),
            &package,
            false,
            None,
            &ManagerScope {
                preferences: true,
                ..Default::default()
            },
            &Memory::default(),
        )
        .unwrap();
        let incoming = receive(target.path(), &bytes, None).unwrap();
        transfer::accept(target.path(), &incoming.id).unwrap();
        atomic_write(
            &cache_path(target.path(), &incoming.id, "adoption.json").unwrap(),
            &serde_json::to_vec(&Adoption {
                id: incoming.id.clone(),
                keys: false,
                webdav: false,
                completed: false,
                extension_credentials: false,
            })
            .unwrap(),
            0o600,
        )
        .unwrap();
        let recovered = latest(target.path()).unwrap().unwrap();
        assert_eq!(recovered.id, incoming.id);
        assert!(!recovered.adoption.unwrap().completed);
        accept_all(
            target.path(),
            &incoming.id,
            false,
            false,
            None,
            &Memory::default(),
        )
        .unwrap();
        assert_eq!(
            crate::modules::preferences::PreferencesStore::new(target.path())
                .load()
                .unwrap()
                .interface_scale,
            125
        );
        assert!(
            latest(target.path())
                .unwrap()
                .unwrap()
                .adoption
                .unwrap()
                .completed
        );
        assert_eq!(
            transfer::select_received(
                target.path(),
                &incoming.id,
                transfer::ReceiveScope {
                    preferences: true,
                    ..Default::default()
                }
            )
            .unwrap_err()
            .code,
            "import_in_progress"
        );
        crate::modules::preferences::PreferencesStore::new(target.path())
            .save(&crate::modules::preferences::Preferences {
                interface_scale: 150,
                ..Default::default()
            })
            .unwrap();
        accept_all(
            target.path(),
            &incoming.id,
            false,
            false,
            None,
            &Memory::default(),
        )
        .unwrap();
        assert_eq!(
            crate::modules::preferences::PreferencesStore::new(target.path())
                .load()
                .unwrap()
                .interface_scale,
            150
        );
    }
    #[test]
    fn legacy_plaintext_file_is_read_as_its_actual_categories_without_creating_a_model_library() {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(source.path()).unwrap();
        crate::modules::preferences::PreferencesStore::new(source.path())
            .save(&crate::modules::preferences::Preferences {
                interface_scale: 125,
                ..Default::default()
            })
            .unwrap();
        let exported =
            crate::modules::migration::export_with_paths(source.path(), "0.1.10").unwrap();
        let bytes = std::fs::read(exported.path).unwrap();
        let incoming = receive(target.path(), &bytes, None).unwrap();
        assert!(incoming.package.payload.templates.is_empty());
        assert!(incoming.manager.as_ref().unwrap().preferences.is_some());
        let before = ModelStore::new(target.path()).load().unwrap();
        let after = accept_all(
            target.path(),
            &incoming.id,
            false,
            false,
            None,
            &Memory::default(),
        )
        .unwrap();
        assert_eq!(after.library, before.library);
        assert!(!target.path().join(LIBRARY_PATH).exists());
        assert_eq!(
            crate::modules::preferences::PreferencesStore::new(target.path())
                .load()
                .unwrap()
                .interface_scale,
            125
        );
    }
    #[test]
    fn one_protected_manifest_restores_selected_manager_settings_and_inactive_webdav_password() {
        use crate::modules::sync::config::{
            CredentialRef, SyncConfig, SyncConfigStore, SyncEndpointConfig,
        };
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let keys = Memory::default();
        let reference = crate::infrastructure::keychain::new_ref_id();
        keys.values
            .lock()
            .unwrap()
            .insert(reference.clone(), "fixture-dav-password".into());
        let source_config = SyncConfig {
            schema_version: 1,
            endpoints: vec![SyncEndpointConfig {
                endpoint_id: "default".into(),
                url: "https://fixture.example.test".into(),
                remote_path: "portable".into(),
                username: "fixture".into(),
                conflict_policy: "ask".into(),
                tls_policy: "verify_required".into(),
                legacy_encryption: String::new(),
                credential_ref: CredentialRef {
                    ref_id: reference,
                    backend: "keychain".into(),
                    service_name: crate::infrastructure::keychain::keychain_service_name().into(),
                    account_key: "fixture".into(),
                    purpose: crate::infrastructure::keychain::WEBDAV_CREDENTIAL_PURPOSE.into(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                    updated_at: chrono::Utc::now().to_rfc3339(),
                },
            }],
        };
        atomic_write(
            &source
                .path()
                .join(crate::modules::sync::config::SYNC_ENDPOINTS_RELATIVE_PATH),
            &serde_json::to_vec(&source_config).unwrap(),
            0o600,
        )
        .unwrap();
        crate::modules::preferences::PreferencesStore::new(source.path())
            .save(&crate::modules::preferences::Preferences {
                interface_scale: 125,
                ..Default::default()
            })
            .unwrap();
        let package = transfer::export(
            source.path(),
            transfer::ExportScope {
                channel_models: Default::default(),
                include_policy: false,
                channel_ids: vec![],
                template_refs: vec![],
                include_global: false,
            },
        )
        .unwrap();
        let (bytes, count, _, _) = seal_scope(
            source.path(),
            &uuid::Uuid::new_v4().to_string(),
            &package,
            false,
            Some("fixture-config-password"),
            &ManagerScope {
                preferences: true,
                extensions: true,
                webdav: true,
                webdav_passwords: true,
                ..Default::default()
            },
            &keys,
        )
        .unwrap();
        assert_eq!(count, 1);
        let incoming = receive(target.path(), &bytes, Some("fixture-config-password")).unwrap();
        assert!(incoming.manager.as_ref().unwrap().preferences.is_some());
        assert_eq!(incoming.available_webdav_passwords, vec!["default"]);
        assert!(!serde_json::to_string(&incoming)
            .unwrap()
            .contains("fixture-dav-password"));
        let accepted = accept_all(
            target.path(),
            &incoming.id,
            false,
            true,
            Some("fixture-config-password"),
            &keys,
        )
        .unwrap();
        assert_eq!(
            crate::modules::preferences::PreferencesStore::new(target.path())
                .load()
                .unwrap()
                .interface_scale,
            125
        );
        assert!(SyncConfigStore::new(target.path())
            .load()
            .unwrap()
            .active()
            .is_none());
        let connections = portable_manager::load_candidates(target.path()).unwrap();
        assert_eq!(connections.len(), 1);
        assert_eq!(
            keys.read(connections[0].credential_ref.as_deref().unwrap())
                .unwrap()
                .as_str(),
            "fixture-dav-password"
        );
        let again = accept_all(target.path(), &incoming.id, false, true, None, &keys).unwrap();
        assert_eq!(again, accepted);
        assert_eq!(*keys.writes.lock().unwrap(), 1);
    }
    #[test]
    fn unknown_manager_fields_are_rejected_instead_of_being_truncated() {
        let root = tempfile::tempdir().unwrap();
        let package = transfer::export(
            root.path(),
            transfer::ExportScope {
                channel_models: Default::default(),
                include_policy: false,
                channel_ids: vec![],
                template_refs: vec![],
                include_global: false,
            },
        )
        .unwrap();
        let objects = Objects {
            format: "OCXD_CONFIG_OBJECTS".into(),
            version: 1,
            models: package,
            channel_keys: Default::default(),
            channel_authorizations: Default::default(),
            webdav_passwords: Default::default(),
            assets: Default::default(),
            manager: Settings {
                preferences: Some(crate::modules::preferences::Preferences::default()),
                ..Default::default()
            },
        };
        let mut value = serde_json::to_value(objects).unwrap();
        value["manager"]["preferences"]["future_setting"] = serde_json::json!(true);
        assert!(decode(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    #[derive(Default)]
    struct Memory {
        values: Mutex<BTreeMap<String, String>>,
        writes: Mutex<usize>,
    }
    impl Credentials for Memory {
        fn read(&self, reference: &str) -> ModelResult<Zeroizing<String>> {
            self.values
                .lock()
                .unwrap()
                .get(reference)
                .cloned()
                .map(Zeroizing::new)
                .ok_or_else(|| invalid("credential_store", "fixture credential missing"))
        }
        fn install(&self, reference: &str, value: &str) -> ModelResult<()> {
            let mut values = self.values.lock().unwrap();
            if let Some(existing) = values.get(reference) {
                if existing != value {
                    return Err(invalid("credential_collision", "fixture slot mismatch"));
                }
                return Ok(());
            }
            values.insert(reference.into(), value.into());
            *self.writes.lock().unwrap() += 1;
            Ok(())
        }
        fn remove(&self, reference: &str) -> ModelResult<()> {
            self.values.lock().unwrap().remove(reference);
            Ok(())
        }
    }
    fn fixture(root: &Path, id: &str, reference: &str) -> ChannelPolicy {
        let node = ServiceNode {
            id: id.into(),
            name: "main".into(),
            url: "https://fixture.example.test/v1".into(),
        };
        let channel = ChannelPolicy {
            naming: Default::default(),
            id: id.into(),
            label: "fixture channel".into(),
            connection: ChannelConnection {
                auth_mode: None,
                source_read_id: None,
                imported_config: None,
                provider_id: "fixture".into(),
                protocol: "openai-chat".into(),
                nodes: vec![node.clone()],
                active_node_id: node.id,
                credential_ref: Some(reference.into()),
                enabled: true,
                allow_private_network: false,
            },
            selection: ModelSelection::None,
            model_order: vec![],
            discovered: vec![],
            template_refs: vec![],
            model_refs: BTreeMap::new(),
            defaults: ModelRules::default(),
            overrides: BTreeMap::new(),
            family_overrides: BTreeMap::new(),
        };
        let store = ModelStore::new(root);
        let snapshot = store.load().unwrap();
        store
            .commit_runtime_policy(
                &snapshot.library_digest,
                &snapshot.policy_digest,
                &snapshot.workspace_digest,
                Policy {
                    channels: vec![channel.clone()],
                    ..Default::default()
                },
            )
            .unwrap();
        channel
    }
    #[test]
    fn mapping_keeps_source_key_out_of_local_connection_and_binds_new_keys_only_to_new_channel() {
        use transfer::{ChannelMapping, ReceiveScope};
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let reference = uuid::Uuid::new_v4().to_string();
        let local_ref = uuid::Uuid::new_v4().to_string();
        fixture(source.path(), &id, &reference);
        let mut local = fixture(target.path(), &id, &local_ref);
        local.connection.nodes[0].url = "https://local.example.test/v1".into();
        let store = ModelStore::new(target.path());
        let before = store.load().unwrap();
        store
            .commit_runtime_policy(
                &before.library_digest,
                &before.policy_digest,
                &before.workspace_digest,
                Policy {
                    channels: vec![local.clone()],
                    ..Default::default()
                },
            )
            .unwrap();
        let keys = Memory::default();
        keys.values
            .lock()
            .unwrap()
            .insert(reference, "fixture-source-key".into());
        keys.values
            .lock()
            .unwrap()
            .insert(local_ref.clone(), "fixture-local-key".into());
        let package = transfer::export(
            source.path(),
            transfer::ExportScope {
                channel_models: Default::default(),
                include_policy: true,
                channel_ids: vec![id.clone()],
                template_refs: vec![],
                include_global: false,
            },
        )
        .unwrap();
        let bytes = seal_with(
            source.path(),
            &uuid::Uuid::new_v4().to_string(),
            &package,
            true,
            Some("fixture-password"),
            &keys,
        )
        .unwrap()
        .0;
        let original = receive(target.path(), &bytes, Some("fixture-password")).unwrap();
        let kept = transfer::select_received(
            target.path(),
            &original.id,
            ReceiveScope {
                channel_ids: vec![id.clone()],
                channel_mappings: BTreeMap::from([(
                    id.clone(),
                    ChannelMapping::Local {
                        target_id: id.clone(),
                    },
                )]),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(kept.available_keys.is_empty());
        assert_eq!(
            accept_with(
                target.path(),
                &kept.id,
                true,
                Some("fixture-password"),
                &keys
            )
            .unwrap_err()
            .code,
            "credential_scope"
        );
        let adopted = accept_with(target.path(), &kept.id, false, None, &keys).unwrap();
        assert_eq!(
            adopted.workspace.as_ref().unwrap().policy.channels[0].connection,
            local.connection
        );
        assert_eq!(*keys.writes.lock().unwrap(), 0);
        let original = receive(target.path(), &bytes, Some("fixture-password")).unwrap();
        let new = transfer::select_received(
            target.path(),
            &original.id,
            ReceiveScope {
                channel_ids: vec![id.clone()],
                channel_mappings: BTreeMap::from([(
                    id.clone(),
                    ChannelMapping::New {
                        label: "独立新渠道".into(),
                    },
                )]),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(new.available_keys, vec![id.clone()]);
        let target_id = new.channel_bindings[&id].target_id.clone();
        let adopted = accept_with(
            target.path(),
            &new.id,
            true,
            Some("fixture-password"),
            &keys,
        )
        .unwrap();
        let channels = &adopted.workspace.as_ref().unwrap().policy.channels;
        assert_eq!(
            channels.iter().find(|c| c.id == id).unwrap().connection,
            local.connection
        );
        let new_channel = channels.iter().find(|c| c.id == target_id).unwrap();
        let new_ref = new_channel.connection.credential_ref.as_deref().unwrap();
        assert_ne!(new_ref, local_ref);
        assert_eq!(keys.read(new_ref).unwrap().as_str(), "fixture-source-key");
        assert!(!new_channel.connection.enabled);
        accept_with(
            target.path(),
            &new.id,
            true,
            Some("fixture-password"),
            &keys,
        )
        .unwrap();
        assert_eq!(*keys.writes.lock().unwrap(), 1);
    }
    #[test]
    fn remote_endpoint_change_without_new_key_never_reuses_the_old_remote_credential() {
        let source = tempfile::tempdir().unwrap();
        let changed = tempfile::tempdir().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let reference = uuid::Uuid::new_v4().to_string();
        fixture(source.path(), &id, &reference);
        let mut next = fixture(changed.path(), &id, &reference);
        next.connection.nodes[0].url = "https://new.example.test/v1".into();
        let store = ModelStore::new(changed.path());
        let snapshot = store.load().unwrap();
        store
            .commit_runtime_policy(
                &snapshot.library_digest,
                &snapshot.policy_digest,
                &snapshot.workspace_digest,
                Policy {
                    channels: vec![next],
                    ..Default::default()
                },
            )
            .unwrap();
        let keys = Memory::default();
        keys.values
            .lock()
            .unwrap()
            .insert(reference, "fixture-original-key".into());
        let package = |root: &Path| {
            transfer::export(
                root,
                transfer::ExportScope {
                    channel_models: Default::default(),
                    include_policy: true,
                    channel_ids: vec![id.clone()],
                    template_refs: vec![],
                    include_global: false,
                },
            )
            .unwrap()
        };
        let remote = seal_with(
            source.path(),
            &uuid::Uuid::new_v4().to_string(),
            &package(source.path()),
            true,
            Some("fixture-password"),
            &keys,
        )
        .unwrap()
        .0;
        let local = seal_with(
            changed.path(),
            &uuid::Uuid::new_v4().to_string(),
            &package(changed.path()),
            false,
            None,
            &keys,
        )
        .unwrap()
        .0;
        assert_eq!(
            merge_remote(
                &remote,
                &local,
                Some("fixture-password"),
                &uuid::Uuid::new_v4().to_string()
            )
            .unwrap_err()
            .code,
            "channel_credential_binding"
        );
        let target = tempfile::tempdir().unwrap();
        let incoming = receive(target.path(), &remote, Some("fixture-password")).unwrap();
        let restored = accept_with(
            target.path(),
            &incoming.id,
            true,
            Some("fixture-password"),
            &keys,
        )
        .unwrap();
        let channel = &restored.workspace.as_ref().unwrap().policy.channels[0];
        assert_eq!(
            channel.connection.nodes[0].url,
            "https://fixture.example.test/v1"
        );
        assert!(!channel.connection.enabled);
        assert_eq!(
            keys.read(channel.connection.credential_ref.as_deref().unwrap())
                .unwrap()
                .as_str(),
            "fixture-original-key"
        );
    }
    #[test]
    fn exported_keys_are_protected_and_explicit_adoption_only_creates_a_new_draft_reference() {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let original_ref = uuid::Uuid::new_v4().to_string();
        let local_ref = uuid::Uuid::new_v4().to_string();
        let source_channel = fixture(source.path(), &id, &original_ref);
        let target_channel = fixture(target.path(), &id, &local_ref);
        let keys = Memory::default();
        keys.values
            .lock()
            .unwrap()
            .insert(original_ref.clone(), "fixture-source-key".into());
        keys.values
            .lock()
            .unwrap()
            .insert(local_ref.clone(), "fixture-local-key".into());
        let package = transfer::export(
            source.path(),
            transfer::ExportScope {
                channel_models: Default::default(),
                include_policy: true,
                channel_ids: vec![id.clone()],
                template_refs: vec![],
                include_global: false,
            },
        )
        .unwrap();
        assert!(seal_with(
            source.path(),
            &uuid::Uuid::new_v4().to_string(),
            &package,
            true,
            None,
            &keys
        )
        .is_err());
        let (bytes, count) = seal_with(
            source.path(),
            &uuid::Uuid::new_v4().to_string(),
            &package,
            true,
            Some("fixture-password"),
            &keys,
        )
        .unwrap();
        assert_eq!(count, 1);
        assert!(!String::from_utf8_lossy(&bytes).contains("fixture-source-key"));
        let incoming = receive(target.path(), &bytes, Some("fixture-password")).unwrap();
        assert_eq!(incoming.available_keys, vec![id.clone()]);
        assert!(!serde_json::to_string(&incoming)
            .unwrap()
            .contains("fixture-source-key"));
        let unchanged = accept_with(target.path(), &incoming.id, false, None, &keys).unwrap();
        assert_eq!(
            unchanged.policy.channels[0]
                .connection
                .credential_ref
                .as_ref(),
            Some(&local_ref)
        );
        assert_eq!(
            unchanged.workspace.unwrap().policy.channels[0]
                .connection
                .credential_ref
                .as_ref(),
            Some(&local_ref)
        );
        assert_eq!(*keys.writes.lock().unwrap(), 0);
        let incoming = receive(target.path(), &bytes, Some("fixture-password")).unwrap();
        let selected = transfer::select_received(
            target.path(),
            &incoming.id,
            transfer::ReceiveScope {
                template_refs: vec![],
                channel_ids: vec![id.clone()],
                include_global: false,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(selected.available_keys, vec![id]);
        let adopted = accept_with(
            target.path(),
            &selected.id,
            true,
            Some("fixture-password"),
            &keys,
        )
        .unwrap();
        let reference = adopted.workspace.as_ref().unwrap().policy.channels[0]
            .connection
            .credential_ref
            .as_ref()
            .unwrap();
        assert_ne!(reference, &local_ref);
        assert_ne!(
            reference,
            &source_channel.connection.credential_ref.unwrap()
        );
        assert_eq!(keys.read(reference).unwrap().as_str(), "fixture-source-key");
        assert_eq!(adopted.policy.channels[0], target_channel);
        assert_eq!(keys.read(&local_ref).unwrap().as_str(), "fixture-local-key");
        let repeated = accept_with(
            target.path(),
            &selected.id,
            true,
            Some("fixture-password"),
            &keys,
        )
        .unwrap();
        assert_eq!(repeated, adopted);
        assert_eq!(*keys.writes.lock().unwrap(), 1);
        assert_eq!(
            accept_with(target.path(), &selected.id, false, None, &keys)
                .unwrap_err()
                .code,
            "import_replay_conflict"
        );
        assert_eq!(
            keys.values
                .lock()
                .unwrap()
                .keys()
                .collect::<BTreeSet<_>>()
                .len(),
            3
        );
    }
    #[test]
    fn raw_secret_objects_and_unknown_secret_targets_are_rejected_before_any_cache_write() {
        let root = tempfile::tempdir().unwrap();
        let package = transfer::export(
            root.path(),
            transfer::ExportScope {
                channel_models: Default::default(),
                include_policy: false,
                channel_ids: vec![],
                template_refs: vec![],
                include_global: false,
            },
        )
        .unwrap();
        let mut objects = Objects {
            format: "OCXD_CONFIG_OBJECTS".into(),
            version: 1,
            models: package,
            channel_keys: SecretValues::default(),
            channel_authorizations: Default::default(),
            manager: Settings::default(),
            webdav_passwords: Default::default(),
            assets: Default::default(),
        };
        objects
            .channel_keys
            .0
            .insert(uuid::Uuid::new_v4().to_string(), "fixture-secret".into());
        let bytes = serde_json::to_vec(&objects).unwrap();
        assert!(receive(root.path(), &bytes, None).is_err());
        assert!(!root
            .path()
            .join("manager-state/model-file-incoming")
            .exists());
    }
}
