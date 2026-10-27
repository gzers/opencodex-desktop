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
fn metadata(root: &Path) -> Policy {
    // One-time compatibility input for ids, extra node labels and template
    // adoptions; factual connection/selection data always comes from config.
    for relative in [METADATA, POLICY_PATH] {
        let path = root.join(relative);
        if safety::check_path(root, &path, true).is_ok() {
            if let Ok(bytes) = safety::read_file(&path, MAX_CONFIG) {
                if let Ok(value) = serde_json::from_slice::<Policy>(&bytes) {
                    return value;
                }
            }
        }
    }
    Policy::default()
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
        policy: config_file_mapping::read_policy(value, meta)?,
        policy_digest: sha256(bytes),
        workspace: None,
        workspace_digest: sha256(&[]),
    })
}
pub fn snapshot(root: &Path, home: &Path) -> ModelResult<Snapshot> {
    let (value, bytes) = read(home)?;
    snapshot_value(root, &value, &bytes, &metadata(root))
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
    let local = store.load()?;
    store.delete(
        expected_library,
        &local.policy_digest,
        &local.workspace_digest,
        template,
        revisions,
    )?;
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
    let meta = metadata(root);
    let before = config_file_mapping::read_policy(&config, &meta)?;
    let (library, _) = ModelStore::new(root).library_only()?;
    config_file_mapping::write_policy(&mut config, &before, &request.policy, &library)?;
    apply_credentials(&mut config, &request.policy, &request.credentials)?;
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
    record_layout(root, &request.policy);
    Ok(SaveResult {
        snapshot: snapshot_value(root, &config, &bytes, &request.policy)?,
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
    if safety::check_path(root, &meta_path, true).is_ok() {
        // The authoritative config write has succeeded. A UI layout failure
        // cannot honestly be reported as an unsuccessful config write.
        if let Ok(bytes) = serde_json::to_vec(&layout) {
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
