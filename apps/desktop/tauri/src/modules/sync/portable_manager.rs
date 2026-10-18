//! Portable manager settings share the file manifest with model objects.
//! Imported connections are inactive candidates; live connection selection is
//! a separate existing endpoint-owner action.
use crate::{
    infrastructure::{atomic_write::atomic_write, locking::TargetFileLock},
    modules::model_config::{schema::*, store::sha256},
    modules::{
        backup::{self, safety, BackupAction},
        extensions::{
            projection::{self, ExtensionProjection},
            ExtensionConfig, ProjectionFingerprint,
        },
        preferences::{self, Preferences, PreferencesStore},
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
pub const CANDIDATES: &str = "manager-state/sync-imported-endpoints.json";
const LIMIT: u64 = 16 * 1024 * 1024;
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Scope {
    pub preferences: bool,
    pub extensions: bool,
    pub webdav: bool,
    pub webdav_passwords: bool,
    #[serde(default)]
    pub skill_names: Vec<String>,
    #[serde(default)]
    pub mcp_names: Vec<String>,
    #[serde(default)]
    pub extension_credentials: bool,
    #[serde(default)]
    pub assets_reviewed: bool,
    #[serde(skip)]
    pub local_home: Option<PathBuf>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Endpoint {
    pub id: String,
    pub url: String,
    pub remote_path: String,
    pub username: String,
    pub conflict_policy: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
    pub preferences: Option<Preferences>,
    pub extensions: Option<ExtensionConfig>,
    pub webdav: Vec<Endpoint>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Candidate {
    pub id: String,
    pub endpoint: Endpoint,
    pub credential_ref: Option<String>,
    pub imported_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct File {
    path: String,
    before: Option<Vec<u8>>,
    after: Vec<u8>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Plan {
    id: String,
    files: Vec<File>,
    completed: bool,
}
fn state_path(root: &Path, id: &str) -> ModelResult<PathBuf> {
    require_uuid(id)?;
    let path = root
        .join("manager-state/portable-manager-plans")
        .join(format!("{id}.json"));
    safety::check_path(root, &path, true)?;
    Ok(path)
}
fn bytes(root: &Path, relative: &str) -> ModelResult<Option<Vec<u8>>> {
    let path = root.join(relative);
    safety::check_path(root, &path, true)?;
    match std::fs::symlink_metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
        Ok(_) => Ok(Some(safety::read_file(&path, LIMIT)?)),
    }
}
pub fn load_candidates(root: &Path) -> ModelResult<Vec<Candidate>> {
    let items: Vec<Candidate> = bytes(root, CANDIDATES)?
        .map(|bytes| serde_json::from_slice(&bytes))
        .transpose()?
        .unwrap_or_default();
    let mut ids = BTreeSet::new();
    for item in &items {
        require_uuid(&item.id)?;
        if let Some(reference) = &item.credential_ref {
            crate::infrastructure::keychain::account_key(
                crate::infrastructure::keychain::WEBDAV_CREDENTIAL_PURPOSE,
                reference,
            )?;
        }
        if !ids.insert(&item.id) {
            return Err(invalid("endpoint_candidate", "导入连接候选身份重复"));
        }
        validate_endpoint(&item.endpoint)?;
    }
    Ok(items)
}
fn validate_endpoint(endpoint: &Endpoint) -> ModelResult<()> {
    if endpoint.id.is_empty() || endpoint.id.len() > 256 || endpoint.username.trim().is_empty() {
        return Err(invalid("endpoint_object", "导入连接的身份或账号无效"));
    }
    let url = reqwest::Url::parse(&endpoint.url)
        .map_err(|_| invalid("endpoint_object", "连接地址无效"))?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(invalid(
            "endpoint_object",
            "WebDAV连接须为不带认证信息的HTTPS地址",
        ));
    }
    let path = endpoint.remote_path.trim_matches('/');
    if path.is_empty()
        || path
            .split('/')
            .any(|part| part.is_empty() || part == ".." || part == ".")
        || !path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'/'))
    {
        return Err(invalid("endpoint_object", "WebDAV远端目录无效"));
    }
    if !["ask", "keep-local", "keep-remote", "keep-both"]
        .contains(&endpoint.conflict_policy.as_str())
    {
        return Err(invalid("endpoint_object", "WebDAV冲突策略未知"));
    }
    Ok(())
}
pub fn validate(settings: &Settings) -> ModelResult<()> {
    if let Some(prefs) = &settings.preferences {
        preferences::validate(prefs)
            .map_err(|_| invalid("portable_preferences", "导入偏好包含无效值"))?;
    }
    if let Some(config) = &settings.extensions {
        if config.source_store.is_some()
            || !["copy", "symlink"].contains(&config.sync_method.as_str())
            || config.skills.len() > 20000
            || config.servers.len() > 20000
            || config
                .skills
                .iter()
                .chain(&config.servers)
                .any(|s| s.trim().is_empty() || s.len() > 256)
        {
            return Err(invalid("portable_extensions", "扩展设置的路径或范围无效"));
        }
    }
    if settings.webdav.len() > 256 {
        return Err(invalid("endpoint_object", "导入连接超过上限"));
    }
    let mut ids = BTreeSet::new();
    for endpoint in &settings.webdav {
        validate_endpoint(endpoint)?;
        if !ids.insert(&endpoint.id) {
            return Err(invalid("endpoint_object", "导入连接身份重复"));
        }
    }
    Ok(())
}
pub fn export(root: &Path, scope: &Scope) -> ModelResult<Settings> {
    let prefs = if scope.preferences {
        Some(
            PreferencesStore::new(root)
                .load()
                .map_err(|_| invalid("portable_preferences", "本机偏好无法读取，未导出"))?,
        )
    } else {
        None
    };
    let extensions = if scope.extensions {
        let mut config = match bytes(root, projection::CONFIG_RELATIVE_PATH)? {
            Some(bytes) => {
                let value: ExtensionProjection = serde_json::from_slice(&bytes)?;
                if value.fingerprint.fingerprint != value.config.fingerprint() {
                    return Err(invalid("portable_extensions", "扩展设置指纹不一致，未导出"));
                }
                value.config
            }
            None => ExtensionConfig::default(),
        };
        config.source_store = None;
        Some(config)
    } else {
        None
    };
    let webdav = if scope.webdav {
        crate::modules::sync::config::SyncConfigStore::new(root)
            .load()?
            .endpoints
            .into_iter()
            .map(|endpoint| Endpoint {
                id: endpoint.endpoint_id,
                url: endpoint.url,
                remote_path: endpoint.remote_path,
                username: endpoint.username,
                conflict_policy: endpoint.conflict_policy,
            })
            .collect()
    } else {
        vec![]
    };
    let result = Settings {
        preferences: prefs,
        extensions,
        webdav,
    };
    validate(&result)?;
    Ok(result)
}
pub fn sections(settings: &Settings) -> Vec<String> {
    let mut items = vec![];
    if settings.preferences.is_some() {
        items.push("preferences".into());
    }
    if settings.extensions.is_some() {
        items.push("extensions".into());
    }
    if !settings.webdav.is_empty() {
        items.push("webdav".into());
    }
    items
}
pub fn prepare(root: &Path, id: &str, settings: &Settings) -> ModelResult<()> {
    validate(settings)?;
    let mut files = vec![];
    if let Some(prefs) = &settings.preferences {
        let mut value = preferences::document_from_preferences(prefs);
        preferences::preserve_model_agent(
            &root.join(preferences::PREFERENCES_RELATIVE_PATH),
            &mut value,
        )
        .map_err(|_| invalid("portable_preferences", "本机执行偏好无法保留"))?;
        files.push(File {
            path: preferences::PREFERENCES_RELATIVE_PATH.into(),
            before: bytes(root, preferences::PREFERENCES_RELATIVE_PATH)?,
            after: serde_json::to_vec_pretty(&value)?,
        });
    }
    if let Some(incoming) = &settings.extensions {
        let before = bytes(root, projection::CONFIG_RELATIVE_PATH)?;
        let mut current = before
            .as_ref()
            .map(|bytes| serde_json::from_slice::<ExtensionProjection>(bytes))
            .transpose()?
            .map(|value| {
                if value.fingerprint.fingerprint != value.config.fingerprint() {
                    return Err(invalid(
                        "portable_extensions",
                        "本机扩展设置指纹不一致，未覆盖",
                    ));
                }
                Ok(value.config)
            })
            .transpose()?
            .unwrap_or_default();
        for name in &incoming.skills {
            if !current.skills.contains(name) {
                current.skills.push(name.clone());
            }
        }
        for name in &incoming.servers {
            if !current.servers.contains(name) {
                current.servers.push(name.clone());
            }
        }
        current.enablement.extend(incoming.enablement.clone());
        current.skill_targets.extend(incoming.skill_targets.clone());
        current.sync_method = incoming.sync_method.clone();
        current.bump_revision();
        let fingerprint = ProjectionFingerprint::create(current.fingerprint(), chrono::Utc::now());
        files.push(File {
            path: projection::CONFIG_RELATIVE_PATH.into(),
            before,
            after: serde_json::to_vec_pretty(&ExtensionProjection {
                config: current,
                fingerprint,
            })?,
        });
    }
    if !settings.webdav.is_empty() {
        let before = bytes(root, CANDIDATES)?;
        let mut candidates = load_candidates(root)?;
        for endpoint in &settings.webdav {
            let active = crate::modules::sync::config::SyncConfigStore::new(root).load()?;
            let credential_ref = active
                .endpoints
                .iter()
                .find(|current| {
                    current.endpoint_id == endpoint.id
                        && current.url == endpoint.url
                        && current.remote_path == endpoint.remote_path
                        && current.username == endpoint.username
                })
                .map(|current| current.credential_ref.ref_id.clone());
            candidates.push(Candidate {
                id: uuid::Uuid::new_v4().to_string(),
                endpoint: endpoint.clone(),
                credential_ref,
                imported_at: chrono::Utc::now().to_rfc3339(),
            });
        }
        files.push(File {
            path: CANDIDATES.into(),
            before,
            after: serde_json::to_vec(&candidates)?,
        });
    }
    atomic_write(
        &state_path(root, id)?,
        &serde_json::to_vec(&Plan {
            id: id.into(),
            files,
            completed: false,
        })?,
        0o600,
    )?;
    Ok(())
}
pub fn select(
    root: &Path,
    original: &str,
    new_id: &str,
    preferences: bool,
    extensions: bool,
    endpoint_ids: &[String],
) -> ModelResult<()> {
    let plan = load(root, original)?;
    let mut files = vec![];
    for mut file in plan.files {
        match file.path.as_str() {
            preferences::PREFERENCES_RELATIVE_PATH if preferences => files.push(file),
            projection::CONFIG_RELATIVE_PATH if extensions => files.push(file),
            CANDIDATES => {
                let before: Vec<Candidate> = file
                    .before
                    .as_ref()
                    .map(|b| serde_json::from_slice(b))
                    .transpose()?
                    .unwrap_or_default();
                let mut after: Vec<Candidate> = serde_json::from_slice(&file.after)?;
                after.retain(|c| {
                    before.iter().any(|old| old.id == c.id) || endpoint_ids.contains(&c.endpoint.id)
                });
                file.after = serde_json::to_vec(&after)?;
                files.push(file)
            }
            _ => {}
        }
    }
    atomic_write(
        &state_path(root, new_id)?,
        &serde_json::to_vec(&Plan {
            id: new_id.into(),
            files,
            completed: false,
        })?,
        0o600,
    )?;
    Ok(())
}
fn load(root: &Path, id: &str) -> ModelResult<Plan> {
    let plan: Plan = serde_json::from_slice(&safety::read_file(
        &state_path(root, id)?,
        64 * 1024 * 1024,
    )?)?;
    if plan.id != id
        || plan.files.len() > 3
        || plan.files.iter().any(|f| {
            ![
                preferences::PREFERENCES_RELATIVE_PATH,
                projection::CONFIG_RELATIVE_PATH,
                CANDIDATES,
            ]
            .contains(&f.path.as_str())
                || f.after.len() as u64 > LIMIT
        })
        || plan
            .files
            .iter()
            .map(|f| &f.path)
            .collect::<BTreeSet<_>>()
            .len()
            != plan.files.len()
    {
        return Err(invalid("portable_manager_plan", "配置导入计划无效"));
    }
    for file in &plan.files {
        match file.path.as_str() {
            preferences::PREFERENCES_RELATIVE_PATH => {
                let document: serde_json::Value = serde_json::from_slice(&file.after)?;
                preferences::preferences_from_document(&document)
                    .map_err(|_| invalid("portable_manager_plan", "偏好产物未通过校验"))?;
            }
            projection::CONFIG_RELATIVE_PATH => {
                let value: ExtensionProjection = serde_json::from_slice(&file.after)?;
                if value.fingerprint.validate().is_err()
                    || value.config.fingerprint() != value.fingerprint.fingerprint
                {
                    return Err(invalid("portable_manager_plan", "扩展产物指纹不一致"));
                }
            }
            CANDIDATES => {
                let values: Vec<Candidate> = serde_json::from_slice(&file.after)?;
                for value in values {
                    require_uuid(&value.id)?;
                    validate_endpoint(&value.endpoint)?;
                }
            }
            _ => unreachable!(),
        }
    }
    Ok(plan)
}
pub fn exists(root: &Path, id: &str) -> ModelResult<bool> {
    Ok(state_path(root, id)?.exists())
}
pub fn verify(root: &Path, id: &str) -> ModelResult<()> {
    let plan = load(root, id)?;
    if plan.completed {
        return Ok(());
    }
    for file in &plan.files {
        let current = bytes(root, &file.path)?;
        if current != file.before && current.as_deref() != Some(file.after.as_slice()) {
            return Err(invalid(
                "portable_manager_drift",
                "接收后管理器配置发生修改，未覆盖",
            ));
        }
    }
    Ok(())
}
pub fn apply(root: &Path, id: &str) -> ModelResult<()> {
    if !exists(root, id)? {
        return Ok(());
    }
    let path = state_path(root, id)?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&path),
        true,
    )?;
    let _plan_guard = TargetFileLock::lock(&path)?;
    let mut plan = load(root, id)?;
    if plan.completed {
        return Ok(());
    }
    let _preferences = backup::manager::acquire_preferences_transaction(root)?;
    let mut guards = vec![];
    let mut paths = plan
        .files
        .iter()
        .map(|f| f.path.as_str())
        .collect::<Vec<_>>();
    paths.sort();
    for relative in paths {
        let target = root.join(relative);
        safety::check_path(
            root,
            &crate::infrastructure::locking::lock_path_for(&target),
            true,
        )?;
        guards.push(TargetFileLock::lock(&target)?);
    }
    verify(root, id)?;
    for file in &plan.files {
        if bytes(root, &file.path)?.as_deref() == Some(file.after.as_slice()) {
            continue;
        }
        if let Some(before) = &file.before {
            backup::backup_file(
                root,
                BackupAction::Import,
                &root.join(&file.path),
                before,
                chrono::Utc::now(),
                Some("统一配置文件采用前保护".into()),
            )?;
        }
    }
    verify(root, id)?;
    for file in &plan.files {
        if bytes(root, &file.path)?.as_deref() != Some(file.after.as_slice()) {
            atomic_write(&root.join(&file.path), &file.after, 0o600)?;
        }
        if bytes(root, &file.path)?.as_deref() != Some(file.after.as_slice()) {
            return Err(invalid(
                "portable_manager_readback",
                "配置文件写后未通过读回；请核对原导入",
            ));
        }
    }
    plan.completed = true;
    atomic_write(&path, &serde_json::to_vec(&plan)?, 0o600)?;
    Ok(())
}
pub fn digest(root: &Path, id: &str) -> ModelResult<String> {
    Ok(sha256(&safety::read_file(
        &state_path(root, id)?,
        64 * 1024 * 1024,
    )?))
}
pub fn bind_passwords(
    root: &Path,
    id: &str,
    bindings: &BTreeMap<String, String>,
) -> ModelResult<()> {
    let mut plan = load(root, id)?;
    if plan.completed {
        return Ok(());
    }
    if let Some(file) = plan.files.iter_mut().find(|f| f.path == CANDIDATES) {
        let mut candidates: Vec<Candidate> = serde_json::from_slice(&file.after)?;
        let existing: Vec<Candidate> = file
            .before
            .as_ref()
            .map(|b| serde_json::from_slice(b))
            .transpose()?
            .unwrap_or_default();
        for candidate in &mut candidates {
            if let Some(reference) = bindings.get(&candidate.endpoint.id) {
                if existing.iter().any(|old| old.id == candidate.id) {
                    return Err(invalid(
                        "endpoint_candidate",
                        "已有候选连接不允许被新导入静默换密码",
                    ));
                }
                if !reference.starts_with("cred_") {
                    return Err(invalid("endpoint_candidate", "导入密码引用格式无效"));
                }
                candidate.credential_ref = Some(reference.clone());
            }
        }
        file.after = serde_json::to_vec(&candidates)?;
    }
    atomic_write(&state_path(root, id)?, &serde_json::to_vec(&plan)?, 0o600)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn settings() -> Settings {
        let preferences = Preferences {
            interface_scale: 125,
            ..Default::default()
        };
        Settings {
            preferences: Some(preferences),
            extensions: Some(ExtensionConfig {
                skills: vec!["fixture-skill".into()],
                sync_method: "copy".into(),
                ..Default::default()
            }),
            webdav: vec![Endpoint {
                id: "remote-connection".into(),
                url: "https://fixture.example.test".into(),
                remote_path: "configuration".into(),
                username: "fixture".into(),
                conflict_policy: "ask".into(),
            }],
        }
    }
    #[test]
    fn manager_import_preserves_local_agent_and_source_paths_and_never_activates_connections() {
        let root = tempfile::tempdir().unwrap();
        crate::modules::model_config::agent::save_settings(
            root.path(),
            crate::modules::model_config::agent::AgentSettings {
                provider_id: "local-model".into(),
                model_id: "fixture".into(),
                reasoning_effort: None,
                fast: Some(false),
            },
        )
        .unwrap();
        let config = ExtensionConfig {
            source_store: Some("/local/owned/source".into()),
            skills: vec!["local-skill".into()],
            ..Default::default()
        };
        let fingerprint = ProjectionFingerprint::create(config.fingerprint(), chrono::Utc::now());
        atomic_write(
            &root.path().join(projection::CONFIG_RELATIVE_PATH),
            &serde_json::to_vec(&ExtensionProjection {
                config,
                fingerprint,
            })
            .unwrap(),
            0o600,
        )
        .unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        prepare(root.path(), &id, &settings()).unwrap();
        apply(root.path(), &id).unwrap();
        assert_eq!(
            PreferencesStore::new(root.path())
                .load()
                .unwrap()
                .interface_scale,
            125
        );
        assert_eq!(
            crate::modules::model_config::agent::settings(root.path())
                .unwrap()
                .unwrap()
                .provider_id,
            "local-model"
        );
        let config: ExtensionProjection = serde_json::from_slice(
            &std::fs::read(root.path().join(projection::CONFIG_RELATIVE_PATH)).unwrap(),
        )
        .unwrap();
        assert_eq!(
            config.config.source_store.as_deref(),
            Some("/local/owned/source")
        );
        assert!(
            config.config.skills.contains(&"local-skill".into())
                && config.config.skills.contains(&"fixture-skill".into())
        );
        assert_eq!(config.fingerprint.fingerprint, config.config.fingerprint());
        assert!(!root
            .path()
            .join(crate::modules::sync::config::SYNC_ENDPOINTS_RELATIVE_PATH)
            .exists());
        let connections = load_candidates(root.path()).unwrap();
        assert_eq!(connections.len(), 1);
        assert!(connections[0].credential_ref.is_none());
        let current = Preferences {
            interface_scale: 150,
            ..Default::default()
        };
        PreferencesStore::new(root.path()).save(&current).unwrap();
        apply(root.path(), &id).unwrap();
        assert_eq!(
            PreferencesStore::new(root.path())
                .load()
                .unwrap()
                .interface_scale,
            150
        );
    }
    #[test]
    fn external_edits_refuse_before_any_manager_object_is_applied() {
        let root = tempfile::tempdir().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        prepare(root.path(), &id, &settings()).unwrap();
        PreferencesStore::new(root.path())
            .save(&Preferences {
                interface_scale: 150,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(
            apply(root.path(), &id).unwrap_err().code,
            "portable_manager_drift"
        );
        assert!(!root.path().join(projection::CONFIG_RELATIVE_PATH).exists());
        assert!(!root.path().join(CANDIDATES).exists());
        assert_eq!(
            PreferencesStore::new(root.path())
                .load()
                .unwrap()
                .interface_scale,
            150
        );
    }
    #[test]
    fn selected_manager_subset_does_not_apply_unselected_preferences_or_connections() {
        let root = tempfile::tempdir().unwrap();
        let original = uuid::Uuid::new_v4().to_string();
        let selected = uuid::Uuid::new_v4().to_string();
        prepare(root.path(), &original, &settings()).unwrap();
        select(root.path(), &original, &selected, false, true, &[]).unwrap();
        apply(root.path(), &selected).unwrap();
        assert!(!root
            .path()
            .join(preferences::PREFERENCES_RELATIVE_PATH)
            .exists());
        assert!(load_candidates(root.path()).unwrap().is_empty());
        assert!(root.path().join(projection::CONFIG_RELATIVE_PATH).exists());
    }
}
