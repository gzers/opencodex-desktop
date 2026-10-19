//! New-channel admission: one recoverable draft addition, never runtime registration.
//! Discovery remains read-only; credentials are created only after explicit Join.
use super::{
    compiler, discovery,
    schema::*,
    store::{sha256, ModelStore},
};
use crate::{
    infrastructure::{atomic_write::atomic_write, locking::TargetFileLock},
    modules::backup::safety,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use zeroize::Zeroizing;
const DIRECTORY: &str = "manager-state/new-channel-operations";
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub id: String,
    pub expected_policy: String,
    pub expected_workspace: String,
    pub channel: ChannelPolicy,
    pub api_key: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    id: String,
    created_at: String,
    expected_policy: String,
    expected_workspace: String,
    request_digest: String,
    channel: ChannelPolicy,
    phase: String,
    error: Option<ModelError>,
    #[serde(default)]
    no_write_proven: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
    pub id: String,
    pub channel: ChannelPolicy,
    pub phase: String,
    pub error: Option<ModelError>,
    pub snapshot: Option<Snapshot>,
}
trait Vault {
    fn put(&self, id: &str, key: &str) -> ModelResult<()>;
    fn forget(&self, id: &str) -> ModelResult<()>;
    fn available(&self, id: &str) -> ModelResult<bool>;
}
struct SystemVault;
impl Vault for SystemVault {
    fn available(&self, id: &str) -> ModelResult<bool> {
        discovery::credential(id).map(|_| true)
    }
    fn put(&self, id: &str, key: &str) -> ModelResult<()> {
        discovery::import_credential(id, key)
    }
    fn forget(&self, id: &str) -> ModelResult<()> {
        discovery::forget(id)
    }
}
fn location(root: &Path, id: &str) -> ModelResult<PathBuf> {
    require_uuid(id)?;
    let path = root.join(DIRECTORY).join(format!("{id}.json"));
    safety::check_path(root, &path, true)?;
    Ok(path)
}
fn lock(root: &Path) -> ModelResult<TargetFileLock> {
    let path = root.join("manager-state/new-channel-admission");
    safety::check_path(root, &path, true)?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&path),
        true,
    )?;
    Ok(TargetFileLock::lock(&path)?)
}
fn write(root: &Path, record: &Record) -> ModelResult<()> {
    if serde_json::to_vec_pretty(record)?.len() > 2 * 1024 * 1024 {
        return Err(invalid(
            "new_channel_limit",
            "新增渠道记录超过2MiB，请收窄清单",
        ));
    }
    atomic_write(
        &location(root, &record.id)?,
        &serde_json::to_vec_pretty(record)?,
        0o600,
    )?;
    Ok(())
}
fn read(root: &Path, id: &str) -> ModelResult<Record> {
    let record: Record =
        serde_json::from_slice(&safety::read_file(&location(root, id)?, 2 * 1024 * 1024)?)?;
    if record.id != id {
        return Err(invalid("new_channel_identity", "新增渠道记录身份不一致"));
    }
    Ok(record)
}
fn view(record: Record, snapshot: Option<Snapshot>) -> Outcome {
    Outcome {
        id: record.id,
        channel: record.channel,
        phase: record.phase,
        error: record.error,
        snapshot,
    }
}
fn latest_unlocked(root: &Path) -> ModelResult<Option<Record>> {
    let directory = root.join(DIRECTORY);
    safety::check_path(root, &directory, true)?;
    if !directory.exists() {
        return Ok(None);
    }
    let mut latest: Option<Record> = None;
    for (count, item) in std::fs::read_dir(directory)?.enumerate() {
        if count >= 10000 {
            return Err(invalid("new_channel_limit", "新增记录数量超过读取上限"));
        }
        let path = item?.path();
        if path.extension().is_none_or(|v| v != "json") {
            continue;
        }
        let id = path
            .file_stem()
            .and_then(|v| v.to_str())
            .ok_or_else(|| invalid("new_channel_identity", "新增记录路径无效"))?;
        let record = read(root, id)?;
        if !["committed", "failed"].contains(&record.phase.as_str())
            && latest
                .as_ref()
                .is_none_or(|v| v.created_at < record.created_at)
        {
            latest = Some(record)
        }
    }
    Ok(latest)
}
pub fn latest(root: &Path) -> ModelResult<Option<Outcome>> {
    let _guard = lock(root)?;
    Ok(latest_unlocked(root)?.map(|r| view(r, None)))
}
fn references(snapshot: &Snapshot, id: &str) -> bool {
    snapshot
        .policy
        .channels
        .iter()
        .chain(snapshot.workspace.iter().flat_map(|w| &w.policy.channels))
        .any(|c| c.connection.credential_ref.as_deref() == Some(id))
}
fn reconcile(root: &Path, mut record: Record, vault: &impl Vault) -> ModelResult<Outcome> {
    let snapshot = ModelStore::new(root).load()?;
    if record.phase == "committed" {
        return Ok(view(record, Some(snapshot)));
    }
    if record.phase == "failed" {
        return Ok(view(record, None));
    }
    let adopted = snapshot
        .policy
        .channels
        .iter()
        .chain(snapshot.workspace.iter().flat_map(|w| &w.policy.channels))
        .any(|c| {
            c.id == record.channel.id
                && c.connection.credential_ref == record.channel.connection.credential_ref
                && c.connection.provider_id == record.channel.connection.provider_id
        });
    if adopted {
        record.phase = "committed".into();
        record.error = None;
        write(root, &record)?;
        return Ok(view(record, Some(snapshot)));
    }
    let unchanged = snapshot.workspace_digest == record.expected_workspace
        && snapshot.policy_digest == record.expected_policy;
    if !record.no_write_proven {
        if !unchanged {
            record.phase = "conflict".into();
            record.error = Some(invalid(
                "new_channel_uncertain",
                "工作区已变化且尚不能确认原加入结果，临时凭据保持，请先核对原渠道草稿",
            ));
            write(root, &record)?;
            return Ok(view(record, None));
        }
        if let Some(id) = &record.channel.connection.credential_ref {
            if !vault.available(id)? {
                return Err(invalid(
                    "credential_store",
                    "原加入所需临时凭据不可读，未继续写入",
                ));
            }
        }
        // Recovered draft-only intent may continue only against the exact
        // frozen source. No runtime API or fresh key creation is performed.
        let mut policy = snapshot
            .workspace
            .as_ref()
            .map(|w| w.policy.clone())
            .unwrap_or_else(|| snapshot.policy.clone());
        policy.channels.push(record.channel.clone());
        match ModelStore::new(root).stage_policy(
            &record.expected_policy,
            &record.expected_workspace,
            policy,
        ) {
            Ok(saved) => {
                record.phase = "committed".into();
                record.error = None;
                write(root, &record)?;
                return Ok(view(record, Some(saved)));
            }
            Err(error) => {
                record.no_write_proven = error.code == "stale_workspace";
                record.error = Some(error);
                write(root, &record)?;
                if !record.no_write_proven {
                    return Ok(view(record, None));
                }
            }
        }
    }
    let cleanup = ModelStore::new(root).with_locked_snapshot(|current| {
        if let Some(id) = &record.channel.connection.credential_ref {
            if references(current, id) {
                return Err(invalid(
                    "new_channel_credential_in_use",
                    "临时凭据已有其他草稿引用，未清理",
                ));
            }
            vault.forget(id)?;
        }
        Ok(())
    });
    if let Err(error) = cleanup {
        record.phase = if error.code == "new_channel_credential_in_use" {
            "conflict"
        } else {
            "cleanup_pending"
        }
        .into();
        record.error = Some(error);
        write(root, &record)?;
        return Ok(view(record, None));
    }
    record.phase = "failed".into();
    if record.error.is_none() {
        record.error = Some(invalid(
            "new_channel_interrupted",
            "原加入未完成，临时凭据已清理；可保留输入重新加入",
        ))
    }
    write(root, &record)?;
    Ok(view(record, None))
}
pub fn result(root: &Path, id: &str) -> ModelResult<Outcome> {
    let _guard = lock(root)?;
    if !location(root, id)?.exists() {
        return Err(invalid(
            "new_channel_not_found",
            "原加入尚未登记，未创建凭据或渠道；输入保留，可重新加入",
        ));
    }
    reconcile(root, read(root, id)?, &SystemVault)
}
pub fn add(root: &Path, request: Request) -> ModelResult<Outcome> {
    add_with(root, request, &SystemVault)
}
fn add_with(root: &Path, mut request: Request, vault: &impl Vault) -> ModelResult<Outcome> {
    require_uuid(&request.id)?;
    let key = request.api_key.take().map(Zeroizing::new);
    compiler::validate_channel(&request.channel)?;
    if request.channel.connection.credential_ref.is_some()
        || request.channel.connection.imported_config.is_some()
        || request.channel.connection.source_read_id.is_some()
        || request.channel.connection.enabled
        || ["oauth", "forward"].contains(
            &request
                .channel
                .connection
                .auth_mode
                .as_deref()
                .unwrap_or("key"),
        )
        || ["openai", "chatgpt", "openai-multi", "combo"].contains(
            &request
                .channel
                .connection
                .provider_id
                .to_ascii_lowercase()
                .as_str(),
        )
    {
        return Err(invalid(
            "new_channel_boundary",
            "新渠道只能以独立停用草稿加入，不接受来源凭据或原生/OAuth绑定",
        ));
    }
    if !matches!(&request.channel.selection,ModelSelection::Whitelist(ids) if !ids.is_empty()&&ids.iter().all(|id|request.channel.discovered.iter().any(|m|&m.id==id)))
    {
        return Err(invalid(
            "new_channel_selection",
            "请明确选择至少一个已列出的精确型号",
        ));
    }
    let needs_key = request.channel.connection.protocol != "ollama-native";
    if key
        .as_ref()
        .is_some_and(|v| v.is_empty() || v.len() > 8192 || v.contains(['\r', '\n']))
        || needs_key && key.is_none()
    {
        return Err(invalid(
            "credential",
            "请输入此渠道的有效API Key；手动型号同样需要认证",
        ));
    }
    let digest = sha256(&serde_json::to_vec(
        &serde_json::json!({"channel":request.channel,"policy":request.expected_policy,"workspace":request.expected_workspace}),
    )?);
    let _guard = lock(root)?;
    if location(root, &request.id)?.exists() {
        let record = read(root, &request.id)?;
        if record.request_digest != digest {
            return Err(invalid(
                "new_channel_replay",
                "同次加入的配置不同，请先核对原结果",
            ));
        }
        return reconcile(root, record, vault);
    }
    if latest_unlocked(root)?.is_some() {
        return Err(invalid(
            "new_channel_unfinished",
            "上次新增渠道尚需核对，请先恢复原加入结果",
        ));
    }
    let store = ModelStore::new(root);
    let snapshot = store.load()?;
    if snapshot.policy_digest != request.expected_policy
        || snapshot.workspace_digest != request.expected_workspace
    {
        return Err(invalid(
            "stale_workspace",
            "渠道工作区已变化，输入保留，请重新读取后再加入",
        ));
    }
    let mut policy = snapshot
        .workspace
        .as_ref()
        .map(|w| w.policy.clone())
        .unwrap_or_else(|| snapshot.policy.clone());
    if policy.channels.iter().any(|c| {
        c.id == request.channel.id
            || c.connection
                .provider_id
                .eq_ignore_ascii_case(&request.channel.connection.provider_id)
            || c.label
                .trim()
                .eq_ignore_ascii_case(request.channel.label.trim())
    }) {
        return Err(invalid("new_channel_duplicate", "渠道名称或运行标识已存在"));
    }
    let reference = key.as_ref().map(|_| uuid::Uuid::new_v4().to_string());
    request.channel.connection.credential_ref = reference.clone();
    request.channel.connection.auth_mode = Some(if needs_key { "key" } else { "local" }.into());
    let mut record = Record {
        id: request.id,
        created_at: chrono::Utc::now().to_rfc3339(),
        expected_policy: request.expected_policy,
        expected_workspace: request.expected_workspace,
        request_digest: digest,
        channel: request.channel.clone(),
        phase: "prepared".into(),
        error: None,
        no_write_proven: true,
    };
    write(root, &record)?;
    if let (Some(id), Some(key)) = (&reference, &key) {
        if let Err(error) = vault.put(id, key) {
            record.error = Some(error);
            write(root, &record)?;
            return reconcile(root, record, vault);
        }
    }
    record.no_write_proven = false;
    record.phase = "staging".into();
    write(root, &record)?;
    policy.channels.push(request.channel);
    match store.stage_policy(&record.expected_policy, &record.expected_workspace, policy) {
        Ok(saved) => {
            record.phase = "committed".into();
            write(root, &record)?;
            Ok(view(record, Some(saved)))
        }
        Err(error) => {
            record.no_write_proven = error.code == "stale_workspace";
            record.error = Some(error);
            write(root, &record)?;
            // Observe the same operation first. A further result query can
            // resume its local draft write only if the original baseline holds.
            if record.no_write_proven {
                reconcile(root, record, vault)
            } else {
                Ok(view(record, None))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeMap, sync::Mutex};
    #[derive(Default)]
    struct FakeVault {
        values: Mutex<BTreeMap<String, String>>,
        fail_cleanup: std::sync::atomic::AtomicBool,
    }
    impl Vault for FakeVault {
        fn available(&self, id: &str) -> ModelResult<bool> {
            Ok(self.values.lock().unwrap().contains_key(id))
        }
        fn put(&self, id: &str, key: &str) -> ModelResult<()> {
            self.values.lock().unwrap().insert(id.into(), key.into());
            Ok(())
        }
        fn forget(&self, id: &str) -> ModelResult<()> {
            if self.fail_cleanup.load(std::sync::atomic::Ordering::SeqCst) {
                return Err(invalid("credential_store", "fixture cleanup unavailable"));
            }
            self.values.lock().unwrap().remove(id);
            Ok(())
        }
    }
    fn request(snapshot: &Snapshot) -> Request {
        let id = uuid::Uuid::new_v4().to_string();
        let node = uuid::Uuid::new_v4().to_string();
        Request {
            id: uuid::Uuid::new_v4().to_string(),
            expected_policy: snapshot.policy_digest.clone(),
            expected_workspace: snapshot.workspace_digest.clone(),
            channel: ChannelPolicy {
                id,
                label: "New channel".into(),
                template_refs: vec![],
                model_refs: Default::default(),
                connection: ChannelConnection {
                    provider_id: "independent".into(),
                    protocol: "openai-chat".into(),
                    nodes: vec![ServiceNode {
                        id: node.clone(),
                        name: "Primary".into(),
                        url: "https://example.test/v1".into(),
                    }],
                    active_node_id: node,
                    ..Default::default()
                },
                selection: ModelSelection::Whitelist(vec!["raw/model".into()]),
                model_order: vec!["raw/model".into()],
                discovered: vec![DiscoveredModel {
                    id: "raw/model".into(),
                    label: "Exact ID".into(),
                    rules: Default::default(),
                }],
                defaults: Default::default(),
                overrides: Default::default(),
                family_overrides: Default::default(),
            },
            api_key: Some("fixture-owned-new-channel-SECRET".into()),
        }
    }
    #[test]
    fn only_explicit_join_adds_disabled_channel_and_same_operation_never_readds_later_edits() {
        let root = tempfile::tempdir().unwrap();
        let store = ModelStore::new(root.path());
        let initial = store.load().unwrap();
        let vault = FakeVault::default();
        let req = request(&initial);
        let operation = req.id.clone();
        let channel_id = req.channel.id.clone();
        let outcome = add_with(root.path(), req, &vault).unwrap();
        assert_eq!(outcome.phase, "committed");
        let saved = outcome.snapshot.unwrap();
        let channel = &saved.workspace.as_ref().unwrap().policy.channels[0];
        assert!(!channel.connection.enabled);
        assert!(saved.policy.channels.is_empty());
        assert_eq!(vault.values.lock().unwrap().len(), 1);
        assert!(
            !std::fs::read_to_string(location(root.path(), &operation).unwrap())
                .unwrap()
                .contains("SECRET")
        );
        let mut policy = saved.workspace.as_ref().unwrap().policy.clone();
        policy.channels[0].label = "Later edit".into();
        let later = store
            .stage_policy(&saved.policy_digest, &saved.workspace_digest, policy)
            .unwrap();
        let _guard = lock(root.path()).unwrap();
        let repeated =
            reconcile(root.path(), read(root.path(), &operation).unwrap(), &vault).unwrap();
        assert_eq!(
            repeated
                .snapshot
                .unwrap()
                .workspace
                .unwrap()
                .policy
                .channels[0]
                .label,
            "Later edit"
        );
        assert_eq!(vault.values.lock().unwrap().len(), 1);
        assert!(latest_unlocked(root.path()).unwrap().is_none());
        assert_eq!(later.workspace.unwrap().policy.channels[0].id, channel_id);
    }
    #[test]
    fn invalid_or_duplicate_inputs_are_rejected_before_credentials_and_draft_write() {
        let root = tempfile::tempdir().unwrap();
        let store = ModelStore::new(root.path());
        let initial = store.load().unwrap();
        let vault = FakeVault::default();
        let mut req = request(&initial);
        req.channel.selection = ModelSelection::None;
        assert!(
            matches!(add_with(root.path(),req,&vault),Err(error) if error.code=="new_channel_selection")
        );
        let mut req = request(&initial);
        req.channel.connection.enabled = true;
        assert!(
            matches!(add_with(root.path(),req,&vault),Err(error) if error.code=="new_channel_boundary")
        );
        assert!(vault.values.lock().unwrap().is_empty());
        assert_eq!(
            store.load().unwrap().workspace_digest,
            initial.workspace_digest
        );
        let first = add_with(root.path(), request(&initial), &vault)
            .unwrap()
            .snapshot
            .unwrap();
        let duplicate = request(&first);
        assert!(
            matches!(add_with(root.path(),duplicate,&vault),Err(error) if error.code=="new_channel_duplicate")
        );
        assert_eq!(vault.values.lock().unwrap().len(), 1);
    }
    #[test]
    fn interrupted_before_stage_cleans_only_owned_secret_and_retries_cleanup_without_adding() {
        let root = tempfile::tempdir().unwrap();
        let initial = ModelStore::new(root.path()).load().unwrap();
        let req = request(&initial);
        let vault = FakeVault::default();
        let key_id = uuid::Uuid::new_v4().to_string();
        let mut channel = req.channel;
        channel.connection.credential_ref = Some(key_id.clone());
        vault.put(&key_id, "fixture-owned-secret").unwrap();
        vault.put("unrelated", "unrelated-secret").unwrap();
        let record = Record {
            id: req.id,
            created_at: chrono::Utc::now().to_rfc3339(),
            expected_policy: req.expected_policy,
            expected_workspace: req.expected_workspace,
            request_digest: "fixture".into(),
            channel,
            phase: "prepared".into(),
            error: None,
            no_write_proven: true,
        };
        write(root.path(), &record).unwrap();
        vault
            .fail_cleanup
            .store(true, std::sync::atomic::Ordering::SeqCst);
        assert_eq!(
            reconcile(root.path(), record.clone(), &vault)
                .unwrap()
                .phase,
            "cleanup_pending"
        );
        vault
            .fail_cleanup
            .store(false, std::sync::atomic::Ordering::SeqCst);
        assert_eq!(
            reconcile(root.path(), record, &vault).unwrap().phase,
            "failed"
        );
        assert!(vault.values.lock().unwrap().contains_key("unrelated"));
        assert!(!vault.values.lock().unwrap().contains_key(&key_id));
        assert!(ModelStore::new(root.path())
            .load()
            .unwrap()
            .workspace
            .is_none());
    }
    #[test]
    fn interrupted_staging_resumes_only_frozen_draft_and_never_creates_another_key() {
        let root = tempfile::tempdir().unwrap();
        let store = ModelStore::new(root.path());
        let initial = store.load().unwrap();
        let req = request(&initial);
        let vault = FakeVault::default();
        let key_id = uuid::Uuid::new_v4().to_string();
        let mut channel = req.channel;
        channel.connection.credential_ref = Some(key_id.clone());
        vault.put(&key_id, "fixture-owned-secret").unwrap();
        let record = Record {
            id: req.id,
            created_at: chrono::Utc::now().to_rfc3339(),
            expected_policy: req.expected_policy,
            expected_workspace: req.expected_workspace,
            request_digest: "fixture".into(),
            channel,
            phase: "staging".into(),
            error: None,
            no_write_proven: false,
        };
        write(root.path(), &record).unwrap();
        let result = reconcile(root.path(), record, &vault).unwrap();
        assert_eq!(result.phase, "committed");
        assert_eq!(
            result
                .snapshot
                .unwrap()
                .workspace
                .unwrap()
                .policy
                .channels
                .len(),
            1
        );
        assert_eq!(vault.values.lock().unwrap().len(), 1);
    }
    #[test]
    fn missing_completion_receipt_recognizes_adopted_key_and_never_cleans_later_workspace() {
        let root = tempfile::tempdir().unwrap();
        let store = ModelStore::new(root.path());
        let initial = store.load().unwrap();
        let vault = FakeVault::default();
        let req = request(&initial);
        let operation = req.id.clone();
        add_with(root.path(), req, &vault).unwrap();
        let mut record = read(root.path(), &operation).unwrap();
        record.phase = "staging".into();
        record.no_write_proven = false;
        write(root.path(), &record).unwrap();
        assert_eq!(
            reconcile(root.path(), record.clone(), &vault)
                .unwrap()
                .phase,
            "committed"
        );
        assert_eq!(vault.values.lock().unwrap().len(), 1);
        let current = store.load().unwrap();
        let mut policy = current.workspace.unwrap().policy;
        policy.channels.clear();
        store
            .stage_policy(&current.policy_digest, &current.workspace_digest, policy)
            .unwrap();
        record.phase = "staging".into();
        record.no_write_proven = false;
        write(root.path(), &record).unwrap();
        assert_eq!(
            reconcile(root.path(), record, &vault).unwrap().phase,
            "conflict"
        );
        assert_eq!(vault.values.lock().unwrap().len(), 1);
    }
}
