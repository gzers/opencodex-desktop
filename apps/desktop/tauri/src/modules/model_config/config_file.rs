//! Current channel data is a view of the official config.json. No catalog,
//! process, historical save-plan or manager draft is required to read/write it.
use super::{
    config_file_mapping,
    schema::*,
    store::{sha256, ModelStore},
};
use crate::{
    infrastructure::{atomic_write::atomic_write, locking::TargetFileLock},
    modules::backup::safety,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
use zeroize::{Zeroize, Zeroizing};

const MAX_CONFIG: u64 = 16 * 1024 * 1024;
const METADATA: &str = "manager-state/model-channel-layout.json";

fn config_path(home: &Path) -> ModelResult<PathBuf> {
    if !home.is_absolute() {
        return Err(invalid("config_path", "配置目录尚未设置"));
    }
    let path = home.join("config.json");
    safety::check_path(home, &path, true)?;
    Ok(path)
}
fn read(home: &Path) -> ModelResult<(Value, Zeroizing<Vec<u8>>)> {
    let path = config_path(home)?;
    let bytes = if path.exists() {
        safety::read_file(&path, MAX_CONFIG)?
    } else {
        b"{}".to_vec()
    };
    let bytes = Zeroizing::new(bytes);
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|_| invalid("config_document", "模型配置文件格式不正确，原文件已保留"))?;
    if !value.is_object() || value.get("providers").is_some_and(|p| !p.is_object()) {
        return Err(invalid(
            "config_document",
            "模型配置文件的渠道结构不正确，原文件已保留",
        ));
    }
    Ok((value, bytes))
}
fn metadata(root: &Path) -> ModelResult<Policy> {
    super::legacy_migration::load_layout(root)
}
fn current_policy(value: &Value, layout: &Policy) -> ModelResult<Policy> {
    let mut policy = config_file_mapping::read_policy(value, layout)?;
    let ordered: BTreeSet<_> = value
        .get("modelPickerOrder")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter_map(|slug| slug.split_once('/').map(|(provider, _)| provider))
        .collect();
    // An empty channel has no picker row in which to persist its position.
    // Use its saved layout anchor, while file order still owns visible groups.
    for (index, saved) in layout.channels.iter().enumerate() {
        if ordered.contains(saved.connection.provider_id.as_str()) {
            continue;
        }
        let Some(position) = policy
            .channels
            .iter()
            .position(|c| c.connection.provider_id == saved.connection.provider_id)
        else {
            continue;
        };
        let channel = policy.channels.remove(position);
        let insertion = layout.channels[..index]
            .iter()
            .rev()
            .find_map(|previous| {
                policy
                    .channels
                    .iter()
                    .position(|c| c.connection.provider_id == previous.connection.provider_id)
            })
            .map_or(0, |position| position + 1);
        policy.channels.insert(insertion, channel);
    }
    Ok(policy)
}
fn snapshot_value(
    root: &Path,
    value: &Value,
    bytes: &[u8],
    meta: &Policy,
) -> ModelResult<Snapshot> {
    let (library, library_digest) = ModelStore::new(root).library_only()?;
    Ok(Snapshot {
        library,
        library_digest,
        policy: current_policy(value, meta)?,
        policy_digest: sha256(bytes),
        workspace: None,
        workspace_digest: sha256(&[]),
    })
}
pub fn snapshot(root: &Path, home: &Path) -> ModelResult<Snapshot> {
    let (value, bytes) = read(home)?;
    snapshot_value(root, &value, &bytes, &metadata(root)?)
}
/// Library mutations must not return a retired channel-policy snapshot to the
/// shared models page or compare a file digest with a manager-policy digest.
pub fn delete_template(
    root: &Path,
    home: &Path,
    expected_library: &str,
    expected_file: &str,
    template: &str,
    revisions: &[String],
) -> ModelResult<Snapshot> {
    let current = snapshot(root, home)?;
    if current.policy_digest != expected_file || current.library_digest != expected_library {
        return Err(invalid(
            "stale_baseline",
            "配置或模版库已变化，请刷新后再删除",
        ));
    }
    let used = current
        .policy
        .template_refs
        .iter()
        .chain(
            current
                .policy
                .channels
                .iter()
                .flat_map(|c| c.template_refs.iter().chain(c.model_refs.values())),
        )
        .any(|reference| {
            reference.template_id == template && revisions.contains(&reference.revision_id)
        });
    if used {
        return Err(invalid(
            "protected_revision",
            "当前渠道仍采用所选版本，未删除",
        ));
    }
    let store = ModelStore::new(root);
    store.delete_library(expected_library, template, revisions)?;
    snapshot(root, home)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CredentialEdit {
    pub channel_id: String,
    pub action: String,
    pub value: Option<String>,
}
impl Drop for CredentialEdit {
    fn drop(&mut self) {
        if let Some(value) = &mut self.value {
            value.zeroize();
        }
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveRequest {
    pub expected_file: String,
    pub policy: Policy,
    #[serde(default)]
    pub credentials: Vec<CredentialEdit>,
    #[serde(default)]
    pub scope: Option<SaveScope>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveScope {
    pub channel_id: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveResult {
    pub snapshot: Snapshot,
    pub changed: bool,
}

fn protected(channel: &ChannelPolicy) -> bool {
    ["openai", "chatgpt", "openai-multi", "combo"]
        .contains(&channel.connection.provider_id.as_str())
        || matches!(
            channel.connection.auth_mode.as_deref(),
            Some("oauth" | "forward")
        )
}
fn apply_credentials(
    config: &mut Value,
    policy: &Policy,
    edits: &[CredentialEdit],
) -> ModelResult<()> {
    let mut seen = BTreeSet::new();
    for edit in edits {
        if !seen.insert(&edit.channel_id) {
            return Err(invalid("credential", "同一渠道的密钥修改重复"));
        }
        let channel = policy
            .channels
            .iter()
            .find(|c| c.id == edit.channel_id)
            .ok_or_else(|| invalid("credential", "密钥所属渠道已不存在"))?;
        if protected(channel) {
            return Err(invalid("credential", "此渠道的认证由官方管理"));
        }
        let row = config["providers"]
            .get_mut(&channel.connection.provider_id)
            .and_then(Value::as_object_mut)
            .ok_or_else(|| invalid("credential", "密钥所属渠道已不存在"))?;
        match edit.action.as_str() {
            "replace" => {
                let key = edit
                    .value
                    .as_deref()
                    .filter(|v| {
                        !v.trim().is_empty() && v.len() <= 8192 && !v.contains(['\r', '\n'])
                    })
                    .ok_or_else(|| invalid("credential", "请填写有效的 API Key"))?;
                let previous = row.get("apiKey").and_then(Value::as_str).map(str::to_owned);
                if let Some(pool) = row.get_mut("apiKeyPool").and_then(Value::as_array_mut) {
                    if let Some(slot) = pool
                        .iter_mut()
                        .find(|slot| slot["key"].as_str() == previous.as_deref())
                    {
                        slot["key"] = json!(key);
                        slot["id"] = json!(sha256(key.as_bytes()).trim_start_matches("sha256:")
                            [..8]
                            .to_string());
                    }
                }
                row.insert("apiKey".into(), json!(key));
            }
            "clear" => {
                row.remove("apiKey");
                row.remove("apiKeyPool");
            }
            _ => return Err(invalid("credential", "不支持的密钥修改方式")),
        }
    }
    Ok(())
}
fn seed_imports(
    root: &Path,
    config: &mut Value,
    before: &Policy,
    intended: &Policy,
) -> ModelResult<Policy> {
    let mut mapping_before = before.clone();
    for channel in &intended.channels {
        // A receipt is produced only by explicit full-provider adoption. Local
        // mappings and partial imports do not carry source raw-field authority.
        let Some(reference) = &channel.connection.imported_config else {
            continue;
        };
        let existing = before.channels.iter().find(|c| c.id == channel.id);
        if protected(channel)
            || existing
                .is_some_and(|old| old.connection.provider_id != channel.connection.provider_id)
            || existing.is_none()
                && config["providers"]
                    .get(&channel.connection.provider_id)
                    .is_some()
        {
            return Err(invalid(
                "provider_import_scope",
                "导入目标与已有或官方渠道冲突，未写入",
            ));
        }
        let material = super::provider_import::load(root, channel, reference)?;
        let mut raw = config["providers"]
            .get(&channel.connection.provider_id)
            .cloned()
            .unwrap_or_else(|| json!({}));
        let object = raw
            .as_object_mut()
            .ok_or_else(|| invalid("provider_import_scope", "当前渠道原字段无效"))?;
        // Preserve unrecognized local fields and local secrets when credentials
        // were not selected, while adopting the complete known public object.
        object.retain(|key, _| !super::provider_fields::PUBLIC.contains(&key.as_str()));
        object.extend(material.public.as_object().unwrap().clone());
        if let Some(auth) = &material.authorization {
            let mut fields = super::provider_auth::imported_file_fields(
                root,
                &material.id,
                &material.provider_id,
                &material.public,
                auth,
            )?;
            raw.as_object_mut()
                .unwrap()
                .retain(|key, _| !super::provider_fields::PRIVATE.contains(&key.as_str()));
            raw.as_object_mut()
                .unwrap()
                .extend(fields.as_object_mut().unwrap().clone());
            super::provider_auth::erase_value(&mut fields);
        }
        if !config["providers"].is_object() {
            config["providers"] = json!({});
        }
        config["providers"][&channel.connection.provider_id] = raw;
        let mut imported = config_file_mapping::read_policy(config, intended)?
            .channels
            .into_iter()
            .find(|c| c.id == channel.id)
            .ok_or_else(|| invalid("provider_import_scope", "导入渠道无法从拟写文件读取"))?;
        // The seed describes raw fields before this edit. In-memory template
        // choices must remain a difference so their projected fields are written.
        imported.template_refs = existing
            .map(|c| c.template_refs.clone())
            .unwrap_or_default();
        imported.model_refs = existing.map(|c| c.model_refs.clone()).unwrap_or_default();
        imported.family_overrides = existing
            .map(|c| c.family_overrides.clone())
            .unwrap_or_default();
        if let Some(slot) = mapping_before
            .channels
            .iter_mut()
            .find(|c| c.id == channel.id)
        {
            *slot = imported;
        } else {
            mapping_before.channels.push(imported);
        }
    }
    Ok(mapping_before)
}

/// A channel save overlays the locked file snapshot, never a filtered request
/// list. Existing channels keep their positions; only a new channel uses the
/// request's nearest already-saved predecessor as an insertion anchor.
fn scoped_policy(before: &Policy, requested: &Policy, scope: &SaveScope) -> ModelResult<Policy> {
    if requested.recovery_id.is_some() {
        return Err(invalid("save_scope", "历史恢复不能作为单渠道普通保存提交"));
    }
    let mut matches = requested
        .channels
        .iter()
        .enumerate()
        .filter(|(_, channel)| channel.id == scope.channel_id);
    let Some((index, channel)) = matches.next() else {
        return Err(invalid("save_scope", "所选保存渠道不在当前草稿中"));
    };
    if matches.next().is_some() {
        return Err(invalid("save_scope", "所选保存渠道身份重复"));
    }
    if before.channels.iter().any(|old| {
        (old.id == channel.id) != (old.connection.provider_id == channel.connection.provider_id)
    }) {
        return Err(invalid(
            "provider_identity",
            "已有渠道的运行绑定或管理身份不能修改",
        ));
    }
    let mut intended = before.clone();
    if let Some(existing) = intended.channels.iter_mut().find(|c| c.id == channel.id) {
        *existing = channel.clone();
    } else {
        let insertion = requested.channels[..index]
            .iter()
            .rev()
            .find_map(|previous| intended.channels.iter().position(|c| c.id == previous.id))
            .map_or(0, |position| position + 1);
        intended.channels.insert(insertion, channel.clone());
    }
    Ok(intended)
}

/// The global mapper can regroup every provider when one inventory changes.
/// Keep its exact/opaque target slugs, but splice only that provider's rows into
/// the original order so a scoped save cannot sort unrelated file contents.
fn scope_picker(
    config: &mut Value,
    original: Option<Value>,
    before: &Policy,
    intended: &Policy,
    scope: &SaveScope,
) {
    let channel = intended
        .channels
        .iter()
        .find(|c| c.id == scope.channel_id)
        .unwrap();
    let prefix = format!("{}/", channel.connection.provider_id);
    let belongs = |value: &Value| value.as_str().is_some_and(|slug| slug.starts_with(&prefix));
    let old_order = original
        .as_ref()
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let old_rows: Vec<_> = old_order
        .iter()
        .filter(|row| belongs(row))
        .cloned()
        .collect();
    let mut rows: Vec<_> = config
        .get("modelPickerOrder")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|row| belongs(row))
        .cloned()
        .collect();
    let is_new = !before.channels.iter().any(|c| c.id == channel.id);
    if is_new && rows.is_empty() {
        // A full-provider import seeds the mapper's before-image. Its inventory
        // can already equal the intended one, so the mapper need not emit order.
        let selected = match &channel.selection {
            ModelSelection::Whitelist(ids) => ids.as_slice(),
            _ => &[],
        };
        for id in channel
            .model_order
            .iter()
            .chain(channel.discovered.iter().map(|m| &m.id))
            .chain(selected)
        {
            let slug = json!(format!("{prefix}{}", id.replace('/', "-")));
            if !rows.contains(&slug) {
                rows.push(slug);
            }
        }
    }
    let next = if old_rows == rows {
        original
    } else {
        let insertion = old_order.iter().position(&belongs).unwrap_or_else(|| {
            let index = intended
                .channels
                .iter()
                .position(|c| c.id == channel.id)
                .unwrap();
            // The predecessor may have interleaved rows after the next
            // channel's first row. Insert before that successor to preserve
            // channel order without regrouping any existing provider rows.
            let successor = intended.channels[index + 1..].iter().find_map(|next| {
                let prefix = format!("{}/", next.connection.provider_id);
                old_order
                    .iter()
                    .position(|row| row.as_str().is_some_and(|s| s.starts_with(&prefix)))
            });
            successor
                .or_else(|| {
                    intended.channels[..index]
                        .iter()
                        .rev()
                        .find_map(|previous| {
                            let prefix = format!("{}/", previous.connection.provider_id);
                            old_order
                                .iter()
                                .rposition(|row| {
                                    row.as_str().is_some_and(|s| s.starts_with(&prefix))
                                })
                                .map(|position| position + 1)
                        })
                })
                .unwrap_or(0)
        });
        let old_len = old_order.len();
        let mut order = Vec::new();
        for (index, row) in old_order.into_iter().enumerate() {
            if index == insertion {
                order.extend(rows.iter().cloned());
            }
            if !belongs(&row) {
                order.push(row);
            }
        }
        // At an end anchor there was no loop iteration at the insertion offset.
        if insertion == old_len {
            order.extend(rows);
        }
        Some(Value::Array(order))
    };
    match next {
        Some(value) => {
            config["modelPickerOrder"] = value;
        }
        None => {
            config.as_object_mut().unwrap().remove("modelPickerOrder");
        }
    }
}
pub fn save(root: &Path, home: &Path, request: SaveRequest) -> ModelResult<SaveResult> {
    let path = config_path(home)?;
    let _lock = TargetFileLock::lock(&path)?;
    let (mut config, before_bytes) = read(home)?;
    if sha256(&before_bytes) != request.expected_file {
        return Err(invalid(
            "file_changed",
            "配置文件已在其它地方修改，请刷新后再保存；当前编辑已保留",
        ));
    }
    let meta = metadata(root)?;
    let before = current_policy(&config, &meta)?;
    let intended = if let Some(scope) = &request.scope {
        if request
            .credentials
            .iter()
            .any(|edit| edit.channel_id != scope.channel_id)
        {
            return Err(invalid(
                "credential_scope",
                "单渠道保存不能修改其他渠道的密钥",
            ));
        }
        scoped_policy(&before, &request.policy, scope)?
    } else {
        request.policy
    };
    let original_picker = config.get("modelPickerOrder").cloned();
    let (library, _) = ModelStore::new(root).library_only()?;
    if request.scope.is_some() {
        // Compile the full merged candidate: unrelated unsaved channels and
        // global draft adoptions must not affect validation or its budget.
        let projection = super::compiler::project(&library, &intended)?;
        if projection.known_active_models > 100 {
            return Err(invalid(
                "catalog_budget",
                "保存后已知启用型号超过100个，请调整当前渠道后重试；当前编辑已保留",
            ));
        }
    }
    let mapping_before = seed_imports(root, &mut config, &before, &intended)?;
    config_file_mapping::write_policy(&mut config, &mapping_before, &intended, &library)?;
    if let Some(scope) = &request.scope {
        scope_picker(&mut config, original_picker, &before, &intended, scope);
    }
    // Legacy portable single-Key entries are adopted only at explicit Save.
    // A local-mapping import has no source reference and cannot replace its Key.
    for channel in &intended.channels {
        if before.channels.iter().any(|c| c.id == channel.id)
            || request
                .credentials
                .iter()
                .any(|e| e.channel_id == channel.id)
            || protected(channel)
        {
            continue;
        }
        if let Some(reference) = &channel.connection.credential_ref {
            let key = super::discovery::credential(reference)?;
            config["providers"][&channel.connection.provider_id]["apiKey"] = json!(key.as_str());
        }
    }
    apply_credentials(&mut config, &intended, &request.credentials)?;
    let old: Value = serde_json::from_slice(&before_bytes)
        .map_err(|_| invalid("config_document", "模型配置文件格式不正确"))?;
    let changed = old != config;
    let bytes = if changed {
        let bytes = Zeroizing::new(serde_json::to_vec_pretty(&config)?);
        // One exact pre-write backup; old manager save plans are not consulted.
        let dir = root.join("backups/model-config").join(format!(
            "{}-{}",
            chrono::Utc::now().format("%Y%m%dT%H%M%S"),
            uuid::Uuid::new_v4()
        ));
        safety::check_path(root, &dir, true)?;
        std::fs::create_dir_all(&dir)?;
        atomic_write(&dir.join("config.json"), &before_bytes, 0o600)?;
        // Check external writers once more immediately before the atomic swap.
        if read(home)?.1.as_slice() != before_bytes.as_slice() {
            return Err(invalid(
                "file_changed",
                "保存期间配置文件发生变化，未覆盖；当前编辑已保留",
            ));
        }
        atomic_write(&path, &bytes, 0o600)?;
        bytes
    } else {
        before_bytes
    };
    record_layout(root, &intended);
    Ok(SaveResult {
        snapshot: snapshot_value(root, &config, &bytes, &intended)?,
        changed,
    })
}

/// Also used after an explicitly requested historical restoration commits.
/// Metadata is never a second copy of current connections/rules/selection.
/// Failure here cannot turn a completed authoritative file write into failure.
pub(super) fn record_layout(root: &Path, policy: &Policy) {
    let mut layout = policy.clone();
    for c in &mut layout.channels {
        c.connection.credential_ref = None;
        c.connection.source_read_id = None;
        c.connection.imported_config = None;
        c.selection = ModelSelection::None;
        c.discovered.clear();
        c.model_order.clear();
        c.defaults = ModelRules::default();
        c.overrides.clear();
        c.naming.clear();
    }
    let meta_path = root.join(METADATA);
    let lock_path = crate::infrastructure::locking::lock_path_for(&meta_path);
    if safety::check_path(root, &meta_path, true).is_ok()
        && safety::check_path(root, &lock_path, true).is_ok()
    {
        // The authoritative config write has succeeded. A UI layout failure
        // cannot honestly be reported as an unsuccessful config write.
        if let (Ok(_guard), Ok(bytes)) = (
            TargetFileLock::lock(&meta_path),
            serde_json::to_vec(&layout),
        ) {
            let _ = atomic_write(&meta_path, &bytes, 0o600);
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialView {
    pub configured: bool,
    pub value: String,
    pub source: String,
    pub viewable: bool,
}
pub fn credential(home: &Path, provider_id: &str) -> ModelResult<CredentialView> {
    let (config, _) = read(home)?;
    let row = config["providers"]
        .get(provider_id)
        .ok_or_else(|| invalid("provider", "渠道已不存在"))?;
    let raw = row["apiKey"]
        .as_str()
        .or_else(|| {
            row["apiKeyPool"]
                .as_array()
                .and_then(|p| p.first())
                .and_then(|p| p["key"].as_str())
        })
        .unwrap_or("");
    let (value, source) = if let Some(name) = raw.strip_prefix('$') {
        let name = name.trim_start_matches('{').trim_end_matches('}');
        (std::env::var(name).unwrap_or_default(), "环境变量")
    } else if let Some(account) = raw.strip_prefix("keychain:") {
        if account != provider_id && !account.starts_with(&format!("{provider_id}/")) {
            return Err(invalid("credential", "凭据引用不属于当前渠道"));
        }
        (
            keyring::Entry::new("opencodex.provider-api-key.v1", account)
                .ok()
                .and_then(|e| e.get_password().ok())
                .unwrap_or_default(),
            "系统钥匙串",
        )
    } else {
        (raw.to_owned(), "配置文件")
    };
    Ok(CredentialView {
        configured: !raw.is_empty(),
        viewable: !value.is_empty(),
        value,
        source: source.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn custom_revision(root: &Path) -> (String, String) {
        let store = ModelStore::new(root);
        let local = store.load().unwrap();
        let content = local.library.templates[0].revisions[0].content.clone();
        let result = store
            .save(SaveTemplate {
                expected_library: local.library_digest,
                template_id: None,
                base_revision: None,
                label: "File-backed custom".into(),
                content,
            })
            .unwrap();
        (result.template_id, result.revision_id)
    }
    #[test]
    fn template_deletion_uses_the_file_digest_and_returns_file_channels() {
        let (_t, root, home) = fixture();
        let (template, revision) = custom_revision(&root);
        let current = snapshot(&root, &home).unwrap();
        let next = delete_template(
            &root,
            &home,
            &current.library_digest,
            &current.policy_digest,
            &template,
            &[revision],
        )
        .unwrap();
        assert_eq!(next.policy.channels[0].connection.provider_id, "demo");
        assert!(!next.library.templates.iter().any(|t| t.id == template));
    }
    #[test]
    fn template_deletion_respects_current_file_layout_adoption() {
        let (_t, root, home) = fixture();
        let (template, revision) = custom_revision(&root);
        let mut current = snapshot(&root, &home).unwrap();
        current.policy.channels[0].template_refs.push(TemplateRef {
            template_id: template.clone(),
            revision_id: revision.clone(),
        });
        atomic_write(
            &root.join(METADATA),
            &serde_json::to_vec(&current.policy).unwrap(),
            0o600,
        )
        .unwrap();
        let error = delete_template(
            &root,
            &home,
            &current.library_digest,
            &current.policy_digest,
            &template,
            &[revision],
        )
        .unwrap_err();
        assert_eq!(error.code, "protected_revision");
    }
    fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("manager");
        let home = t.path().join("official");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&root).unwrap();
        atomic_write(&home.join("config.json"),br#"{"providers":{"demo":{"adapter":"openai-chat","baseUrl":"https://example.test","apiKey":"test-original","models":["vendor/custom"],"selectedModels":["vendor/custom"],"futureField":{"keep":42}}},"unrelated":true}"#,0o600).unwrap();
        (t, root, home)
    }
    #[test]
    fn read_ignores_old_draft_and_unfinished_plan() {
        let (_t, root, home) = fixture();
        std::fs::create_dir_all(root.join("manager-state/model-save-plans")).unwrap();
        std::fs::write(root.join(DRAFT_PATH), b"broken old draft").unwrap();
        std::fs::write(
            root.join("manager-state/model-save-plans/old.json"),
            b"broken old plan",
        )
        .unwrap();
        let s = snapshot(&root, &home).unwrap();
        assert_eq!(s.policy.channels.len(), 1);
        assert_eq!(s.policy.channels[0].connection.provider_id, "demo");
        assert!(s.workspace.is_none());
    }
    #[test]
    fn restored_layout_retains_adoptions_but_file_remains_authority() {
        let (_temp, root, home) = fixture();
        let original = std::fs::read(home.join("config.json")).unwrap();
        let mut history = snapshot(&root, &home).unwrap().policy;
        let (template_id, revision_id) = custom_revision(&root);
        let channel = &mut history.channels[0];
        channel.label = "历史显示名称".into();
        channel.template_refs = vec![TemplateRef {
            template_id,
            revision_id,
        }];
        channel.connection.credential_ref = Some("retired-reference".into());
        channel.connection.nodes[0].url = "https://stale.example.test".into();
        channel.selection = ModelSelection::None;
        record_layout(&root, &history);
        let reopened = snapshot(&root, &home).unwrap();
        let channel = &reopened.policy.channels[0];
        assert_eq!(channel.label, "历史显示名称");
        assert_eq!(channel.template_refs, history.channels[0].template_refs);
        assert_eq!(channel.connection.nodes[0].url, "https://example.test");
        assert!(channel.connection.credential_ref.is_none());
        assert!(channel.discovered.iter().any(|m| m.id == "vendor/custom"));
        assert!(std::fs::read(home.join("config.json")).unwrap() == original);
    }
    #[test]
    fn key_only_save_survives_offline_and_preserves_unknown_fields() {
        let (_t, root, home) = fixture();
        let s = snapshot(&root, &home).unwrap();
        let id = s.policy.channels[0].id.clone();
        let r = save(
            &root,
            &home,
            SaveRequest {
                scope: None,
                expected_file: s.policy_digest,
                policy: s.policy,
                credentials: vec![CredentialEdit {
                    channel_id: id,
                    action: "replace".into(),
                    value: Some("test-replaced".into()),
                }],
            },
        )
        .unwrap();
        assert!(r.changed);
        let (c, _) = read(&home).unwrap();
        assert_eq!(c["providers"]["demo"]["futureField"]["keep"], 42);
        assert_eq!(credential(&home, "demo").unwrap().value, "test-replaced");
        assert_eq!(c["unrelated"], true);
    }
    #[test]
    fn imported_provider_fields_join_only_at_confirmed_file_save() {
        let (_t, root, home) = fixture();
        let original = std::fs::read(home.join("config.json")).unwrap();
        let mut current = snapshot(&root, &home).unwrap();
        let mut channel = current.policy.channels[0].clone();
        channel.id = uuid::Uuid::new_v4().to_string();
        channel.connection.provider_id = "imported".into();
        channel.connection.enabled = false;
        let public = json!({"adapter":"openai-chat","baseUrl":"https://example.test","authMode":"key","models":["vendor/custom"],"selectedModels":["vendor/custom"],"note":"exact source note","retryOn429":false});
        let reference =
            super::super::provider_import::stage(&root, &channel, &public, None).unwrap();
        channel.connection.imported_config = Some(reference);
        current.policy.channels.push(channel);
        assert!(std::fs::read(home.join("config.json")).unwrap() == original);
        save(
            &root,
            &home,
            SaveRequest {
                scope: None,
                expected_file: current.policy_digest,
                policy: current.policy,
                credentials: vec![],
            },
        )
        .unwrap();
        let saved = read(&home).unwrap().0;
        assert_eq!(saved["providers"]["imported"]["note"], "exact source note");
        assert_eq!(saved["providers"]["imported"]["retryOn429"], false);
        assert_eq!(saved["providers"]["imported"]["disabled"], true);
        assert_eq!(
            saved["providers"]["demo"]["futureField"],
            json!({"keep":42})
        );
        assert!(saved["providers"]["demo"]["apiKey"].as_str() == Some("test-original"));
        let reopened = snapshot(&root, &home).unwrap();
        assert_eq!(reopened.policy.channels.len(), 2);
        assert!(reopened
            .policy
            .channels
            .iter()
            .all(|c| c.connection.imported_config.is_none()));
    }
    #[test]
    fn explicit_same_identity_import_writes_raw_overlay_but_keeps_unselected_local_key() {
        let (_t, root, home) = fixture();
        let mut current = snapshot(&root, &home).unwrap();
        let public = json!({"adapter":"openai-chat","baseUrl":"https://example.test","models":["vendor/custom"],"selectedModels":["vendor/custom"],"note":"adopted raw change","retryOn429":false});
        let channel = &mut current.policy.channels[0];
        channel.connection.imported_config =
            Some(super::super::provider_import::stage(&root, channel, &public, None).unwrap());
        let saved = save(
            &root,
            &home,
            SaveRequest {
                scope: None,
                expected_file: current.policy_digest,
                policy: current.policy,
                credentials: vec![],
            },
        )
        .unwrap();
        assert!(saved.changed);
        let raw = read(&home).unwrap().0;
        assert_eq!(raw["providers"]["demo"]["note"], "adopted raw change");
        assert_eq!(raw["providers"]["demo"]["retryOn429"], false);
        assert_eq!(raw["providers"]["demo"]["futureField"], json!({"keep":42}));
        assert!(raw["providers"]["demo"]["apiKey"].as_str() == Some("test-original"));
    }
    #[test]
    fn imported_raw_seed_does_not_hide_new_template_adoption() {
        for same_identity in [false, true] {
            let (_t, root, home) = fixture();
            let store = ModelStore::new(&root);
            let library = store.load_library().unwrap();
            let mut content = TemplateContent::default();
            content.matching.include = vec![Pattern {
                kind: MatchKind::Exact,
                value: "vendor/custom".into(),
            }];
            content.rules.context_window = Field::Custom(128000);
            let template = store
                .save(SaveTemplate {
                    expected_library: library.library_digest,
                    template_id: None,
                    base_revision: None,
                    label: "Imported template".into(),
                    content,
                })
                .unwrap();
            let mut current = snapshot(&root, &home).unwrap();
            let mut channel = current.policy.channels[0].clone();
            if !same_identity {
                channel.id = uuid::Uuid::new_v4().to_string();
                channel.connection.provider_id = "imported".into();
            }
            let public = json!({"adapter":"openai-chat","baseUrl":"https://example.test","models":["vendor/custom"],"selectedModels":["vendor/custom"]});
            channel.connection.imported_config =
                Some(super::super::provider_import::stage(&root, &channel, &public, None).unwrap());
            channel.template_refs = vec![TemplateRef {
                template_id: template.template_id,
                revision_id: template.revision_id,
            }];
            let provider = channel.connection.provider_id.clone();
            if same_identity {
                current.policy.channels[0] = channel;
            } else {
                current.policy.channels.push(channel);
            }
            save(
                &root,
                &home,
                SaveRequest {
                    scope: None,
                    expected_file: current.policy_digest,
                    policy: current.policy,
                    credentials: vec![],
                },
            )
            .unwrap();
            assert_eq!(
                read(&home).unwrap().0["providers"][provider]["modelContextWindows"]
                    ["vendor/custom"],
                128000
            );
        }
    }
    #[test]
    fn external_file_change_preserves_both_edit_and_external_file() {
        let (_t, root, home) = fixture();
        let mut s = snapshot(&root, &home).unwrap();
        s.policy.channels[0].connection.enabled = false;
        let (c, _) = read(&home).unwrap();
        let mut c = c;
        c["external"] = json!("later");
        atomic_write(
            &home.join("config.json"),
            &serde_json::to_vec(&c).unwrap(),
            0o600,
        )
        .unwrap();
        let e = save(
            &root,
            &home,
            SaveRequest {
                scope: None,
                expected_file: s.policy_digest,
                policy: s.policy,
                credentials: vec![],
            },
        )
        .err()
        .unwrap();
        assert_eq!(e.code, "file_changed");
        assert_eq!(read(&home).unwrap().0["external"], "later");
    }
}

#[cfg(test)]
#[path = "config_file_scoped_tests.rs"]
mod scoped_tests;
