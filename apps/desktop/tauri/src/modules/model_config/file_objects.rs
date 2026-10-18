//! Portable object payloads keep secrets behind the shared protected envelope.
//! Incoming metadata/IPC never contains key values. Adoption creates fresh local
//! keyring references and only stages policy through the common transaction.
use super::{
    discovery,
    schema::*,
    store::{sha256, ModelStore},
    transfer::{self, IncomingPlan, ModelPackage},
};
use crate::modules::sync::portable_manager::{self, Scope as ManagerScope, Settings};
use crate::{
    infrastructure::atomic_write::atomic_write,
    modules::{backup::safety, sync::portable_file},
};
use serde::{Deserialize, Serialize};
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
    #[serde(default)]
    manager: Settings,
    #[serde(default)]
    webdav_passwords: SecretValues,
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
fn sections(objects: &Objects) -> Vec<String> {
    let mut names = vec!["models".into()];
    if !objects.channel_keys.0.is_empty() {
        names.push("channelKeys".into());
    }
    names.extend(portable_manager::sections(&objects.manager));
    if !objects.webdav_passwords.0.is_empty() {
        names.push("webdavPasswords".into());
    }
    names
}
fn validate(objects: &Objects) -> ModelResult<()> {
    if objects.format != "OCXD_CONFIG_OBJECTS" || objects.version != 1 {
        return Err(invalid("portable_objects", "不支持的配置对象文档版本"));
    }
    transfer::validate_package(&objects.models)?;
    portable_manager::validate(&objects.manager)?;
    for (channel, key) in &objects.channel_keys.0 {
        require_uuid(channel)?;
        if key.is_empty()
            || key.len() > 8192
            || key.contains(['\r', '\n'])
            || objects
                .models
                .payload
                .policy
                .as_ref()
                .is_none_or(|p| !p.channels.iter().any(|c| &c.id == channel))
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
            manager: Settings::default(),
            webdav_passwords: Default::default(),
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
    let (bytes, count, _) = seal_scope(
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
) -> ModelResult<(Vec<u8>, usize, Settings)> {
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
) -> ModelResult<(Vec<u8>, usize, Settings)> {
    if include_keys && passphrase.is_none_or(|password| password.is_empty()) {
        return Err(invalid(
            "passphrase_required",
            "携带渠道Key时须设置独立文件保护口令",
        ));
    }
    let mut keys = SecretValues::default();
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
            let source = snapshot
                .policy
                .channels
                .iter()
                .find(|c| c.id == channel.id)
                .ok_or_else(|| invalid("stale_baseline", "冻结导出后渠道已改变"))?;
            if serde_json::to_value(transfer::portable(source))? != serde_json::to_value(channel)? {
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
        version: 1,
        models: package.clone(),
        channel_keys: keys,
        manager: portable_manager::export(root, scope)?,
        webdav_passwords: Default::default(),
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
    validate(&objects)?;
    let count = objects.channel_keys.0.len() + objects.webdav_passwords.0.len();
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
    Ok((bytes, count, objects.manager))
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
        || opened.manifest.secret_objects != objects.channel_keys.0.len()
    {
        return Err(invalid("portable_manifest", "远端清单与对象不同"));
    }
    Ok((objects, opened.protected))
}
pub fn remote_summary(bytes: &[u8], passphrase: Option<&str>) -> ModelResult<RemoteSummary> {
    let (objects, protected) = remote_objects(bytes, passphrase)?;
    Ok(RemoteSummary {
        package: objects.models,
        manager: objects.manager,
        credential_count: objects.channel_keys.0.len(),
        protected,
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
    for (channel, key) in &local.channel_keys.0 {
        remote.channel_keys.0.insert(channel.clone(), key.clone());
    }
    validate(&remote)?;
    let document = Zeroizing::new(serde_json::to_vec(&remote)?);
    let password =
        if remote_protected || local_protected || !remote.channel_keys.0.is_empty() {
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
            secret_objects: remote.channel_keys.0.len(),
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
    if !manager.preferences {
        objects.manager.preferences = None;
    }
    if !manager.extensions {
        objects.manager.extensions = None;
    }
    let id = uuid::Uuid::new_v4().to_string();
    let document = Zeroizing::new(serde_json::to_vec(&objects)?);
    let selected_file = portable_file::seal(
        portable_file::Manifest {
            id,
            created_at: chrono::Utc::now().to_rfc3339(),
            sections: sections(&objects),
            secret_objects: objects.channel_keys.0.len(),
            document_digest: String::new(),
        },
        &document,
        passphrase,
    )?;
    let incoming = receive(root, &selected_file, passphrase)?;
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
    transfer::select_received(
        root,
        &incoming.id,
        transfer::ReceiveScope {
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
        },
    )
}
pub fn receive(root: &Path, bytes: &[u8], passphrase: Option<&str>) -> ModelResult<IncomingPlan> {
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
            manager: Settings {
                preferences,
                extensions,
                webdav,
            },
            webdav_passwords: Default::default(),
        };
        validate(&objects)?;
        objects
    } else if format["format"] == "OCXDCONF" {
        let opened = portable_file::open(bytes, passphrase)?;
        let objects = decode(&opened.document)?;
        if opened.manifest.sections != sections(&objects)
            || opened.manifest.secret_objects
                != objects.channel_keys.0.len() + objects.webdav_passwords.0.len()
            || ((!objects.channel_keys.0.is_empty() || !objects.webdav_passwords.0.is_empty())
                && !opened.protected)
        {
            return Err(invalid(
                "portable_manifest",
                "文件清单与实际对象或凭据保护不一致",
            ));
        }
        objects
    } else {
        let objects = decode(bytes)?;
        if !objects.channel_keys.0.is_empty() || !objects.webdav_passwords.0.is_empty() {
            return Err(invalid(
                "portable_protection",
                "携带Key的对象文档必须来自受保护配置文件",
            ));
        }
        objects
    };
    let mut plan = transfer::receive(root, objects.models)?;
    if legacy {
        plan.notices.push("已按旧容器实际区段转换；旧文件不含模型库、真实登录态或Key。扩展名清单不表示资源/凭据已恢复。".into());
    }
    plan.manager = Some(objects.manager.clone());
    plan.available_webdav_passwords = objects.webdav_passwords.0.keys().cloned().collect();
    portable_manager::prepare(root, &plan.id, &objects.manager)?;
    if !objects.channel_keys.0.is_empty() || !objects.webdav_passwords.0.is_empty() {
        plan.available_keys = objects.channel_keys.0.keys().cloned().collect();
    }
    atomic_write(&cache_path(root, &plan.id, "ocxdconf")?, bytes, 0o600)?;
    plan.file_digest = Some(sha256(bytes));
    transfer::write_incoming(root, &plan)?;
    let index = root.join("manager-state/portable-latest-incoming.json");
    safety::check_path(root, &index, true)?;
    atomic_write(&index, &serde_json::to_vec(&plan.id)?, 0o600)?;
    Ok(plan)
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
    receive(root, &bytes, passphrase)
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
    selected.available_keys = original
        .available_keys
        .iter()
        .filter(|id| {
            selected
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
    let index = root.join("manager-state/portable-latest-incoming.json");
    safety::check_path(root, &index, true)?;
    atomic_write(&index, &serde_json::to_vec(&selected.id)?, 0o600)?;
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
}
fn accept_all(
    root: &Path,
    id: &str,
    adopt_keys: bool,
    adopt_webdav: bool,
    passphrase: Option<&str>,
    credentials: &impl Credentials,
) -> ModelResult<Snapshot> {
    let plan = transfer::read_incoming(root, id)?;
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
        if record.id != id || record.keys != adopt_keys || record.webdav != adopt_webdav {
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
    let result = accept_with(root, id, adopt_keys, passphrase, credentials)?;
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
    if !opened.protected
        || opened.manifest.secret_objects
            != objects.channel_keys.0.len() + objects.webdav_passwords.0.len()
    {
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
        if let Some(value) = objects.channel_keys.0.get(&channel.id) {
            if !objects.models.payload.policy.as_ref().is_some_and(|p| {
                p.channels.iter().any(|original| {
                    original.id == channel.id
                        && serde_json::to_value(original).ok() == serde_json::to_value(channel).ok()
                })
            }) {
                return Err(invalid("credential_scope", "凭据源渠道与所选配置不一致"));
            }
            wanted.insert(channel.id.clone(), value);
        }
    }
    if wanted.is_empty() {
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
    use super::*;
    use std::{collections::BTreeSet, sync::Mutex};
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
                include_policy: false,
                channel_ids: vec![],
                template_refs: vec![],
                include_global: false,
            },
        )
        .unwrap();
        let (bytes, _, _) = seal_scope(
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
                include_policy: false,
                channel_ids: vec![],
                template_refs: vec![],
                include_global: false,
            },
        )
        .unwrap();
        let (bytes, count, _) = seal_scope(
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
            webdav_passwords: Default::default(),
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
            id: uuid::Uuid::new_v4().to_string(),
            name: "main".into(),
            url: "https://fixture.example.test/v1".into(),
        };
        let channel = ChannelPolicy {
            id: id.into(),
            label: "fixture channel".into(),
            connection: ChannelConnection {
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
            manager: Settings::default(),
            webdav_passwords: Default::default(),
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
