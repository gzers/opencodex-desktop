//! 设备间同步状态：记录上次同步的远端指针，并计算同步预览中的变化项。

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};

use super::{
    remote::{self, Pointer},
    schema::*,
    store::sha256,
    transfer::ModelPackage,
};
use crate::{
    infrastructure::{
        atomic_write::atomic_write,
        webdav_client::{WebDavConfig, WebDavTransport},
    },
    modules::{
        backup::safety,
        sync::{
            portable_assets::{McpConflict, Summary},
            portable_manager::Settings,
        },
    },
};

const LAST: &str = "manager-state/config-sync-last.json";
const PENDING: &str = "manager-state/config-sync-pending.json";
const STATE_LIMIT: u64 = 64 * 1024;
const KINDS: [&str; 7] = [
    "channels",
    "templates",
    "accounts",
    "skills",
    "mcp",
    "preferences",
    "other",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Last {
    binding: String,
    pointer: Pointer,
    synced_at: String,
    #[serde(default)]
    local_content: Option<String>,
    #[serde(default)]
    scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Pending {
    binding: String,
    pointer: Pointer,
    plan_id: String,
}

/// 同步按钮点击后的方向判断结果。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    pub source: String,
    pub has_remote: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_synced_at: Option<String>,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub local_saved_at: Option<String>,
    #[serde(default)]
    pub remote_saved_at: Option<String>,
    #[serde(default)]
    pub remote_revision: Option<String>,
}

/// 同步预览中的一项变化。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    pub kind: String,
    pub id: String,
    pub label: String,
    pub mark: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clients: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files_added: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files_updated: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files_removed: Vec<String>,
}

/// 同步预览：新版来源、变化列表与保持不变的数量。changes 为 None 表示无法比较。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncPreview {
    pub source: String,
    #[serde(default)]
    pub changes: Option<Vec<Change>>,
    #[serde(default)]
    pub unchanged: usize,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct Fact {
    label: String,
    digest: String,
    version: Option<usize>,
    selected: Option<usize>,
    files: BTreeMap<String, crate::modules::sync::portable_assets::SkillFileFact>,
    output_id: Option<String>,
}

pub(super) type Facts = BTreeMap<(u8, String), Fact>;

fn read_state<T: DeserializeOwned>(root: &Path, name: &str) -> Option<T> {
    let path = root.join(name);
    safety::check_path(root, &path, true).ok()?;
    let bytes = safety::read_file(&path, STATE_LIMIT).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn write_state<T: Serialize>(root: &Path, name: &str, value: &T) -> ModelResult<()> {
    let path = root.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    atomic_write(&path, &serde_json::to_vec(value)?, 0o600)?;
    Ok(())
}

/// 记录本机与远端在该绑定下已一致。
pub(super) fn record_synced(root: &Path, binding: &str, pointer: &Pointer) -> ModelResult<()> {
    write_state(
        root,
        LAST,
        &Last {
            binding: binding.into(),
            pointer: pointer.clone(),
            synced_at: chrono::Utc::now().to_rfc3339(),
            local_content: None,
            scope: None,
        },
    )
}

/// 记录待确认的下载计划；确认导入后才视为已同步。
pub(super) fn record_pending(
    root: &Path,
    binding: &str,
    pointer: &Pointer,
    plan_id: &str,
) -> ModelResult<()> {
    write_state(
        root,
        PENDING,
        &Pending {
            binding: binding.into(),
            pointer: pointer.clone(),
            plan_id: plan_id.into(),
        },
    )
}

/// 下载计划确认导入后，把待确认记录提升为上次同步记录。
pub fn promote_pending(root: &Path, plan_id: &str) -> ModelResult<bool> {
    let Some(pending) = read_state::<Pending>(root, PENDING) else {
        return Ok(false);
    };
    if pending.plan_id != plan_id {
        return Ok(false);
    }
    record_synced(root, &pending.binding, &pending.pointer)?;
    let _ = std::fs::remove_file(root.join(PENDING));
    Ok(true)
}

/// Compatibility entry point uses the same content comparison as the UI.
#[cfg(test)]
pub async fn check<T: WebDavTransport>(
    root: &Path,
    transport: &T,
    config: &WebDavConfig,
    binding: &str,
) -> ModelResult<Check> {
    let mut scope = super::config_remote::load_scope(root)?;
    scope.cloud_binding = Some(binding.into());
    super::config_remote::check_source(root, transport, config, binding, &scope).await
}
pub(super) fn record_synced_content(
    root: &Path,
    binding: &str,
    pointer: &Pointer,
    local_content: &str,
    scope: &str,
) -> ModelResult<()> {
    write_state(
        root,
        LAST,
        &Last {
            binding: binding.into(),
            pointer: pointer.clone(),
            synced_at: chrono::Utc::now().to_rfc3339(),
            local_content: Some(local_content.into()),
            scope: Some(scope.into()),
        },
    )
}
/// Direction is derived from selected content and reliable save provenance,
/// never from a device revision, last-sync record or pointer ordering.
pub(super) fn compare_content(
    local: &super::file_objects::ContentFact,
    remote: Option<&super::file_objects::ContentFact>,
) -> Check {
    let (source, reason) = match remote {
        None if local.empty => ("unchanged", "both_empty"),
        None => ("local", "no_remote"),
        Some(r) if local.content_digest == r.content_digest => ("unchanged", "same_content"),
        Some(r) if local.empty && r.empty => ("unchanged", "both_empty"),
        Some(r) if r.empty => ("local", "remote_selection_empty"),
        Some(_) if local.empty => ("remote", "local_selection_empty"),
        Some(r) => {
            let instant = |f: &super::file_objects::ContentFact| {
                if f.time_kind != super::content_versions::TimeKind::Saved {
                    return None;
                }
                f.saved_at
                    .as_deref()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            };
            match (instant(local), instant(r)) {
                (Some(l), Some(r)) if l > r => ("local", "local_saved_later"),
                (Some(l), Some(r)) if l < r => ("remote", "remote_saved_later"),
                (Some(_), Some(_)) => ("undetermined", "same_instant_different_content"),
                _ => ("undetermined", "unreliable_save_time"),
            }
        }
    };
    Check {
        source: source.into(),
        reason: reason.into(),
        has_remote: remote.is_some(),
        last_synced_at: None,
        local_saved_at: local.saved_at.clone(),
        remote_saved_at: remote.and_then(|r| r.saved_at.clone()),
        remote_revision: None,
    }
}
pub(super) fn last_synced_at(root: &Path, binding: &str) -> Option<String> {
    read_state::<Last>(root, LAST)
        .filter(|l| l.binding == binding)
        .map(|l| l.synced_at)
}
pub fn promote_pending_content(
    root: &Path,
    plan_id: &str,
    binding: &str,
    local: &str,
    scope: &str,
) -> ModelResult<bool> {
    let Some(p) = read_state::<Pending>(root, PENDING) else {
        return Ok(false);
    };
    if p.plan_id != plan_id {
        return Ok(false);
    }
    if p.binding != binding {
        return Err(invalid(
            "endpoint_changed",
            "配置已更新，但活动连接改变；未标记新连接为已同步",
        ));
    }
    record_synced_content(root, &p.binding, &p.pointer, local, scope)?;
    std::fs::remove_file(root.join(PENDING))?;
    Ok(true)
}
pub(super) fn rebind_pending(root: &Path, original: &str, selected: &str) -> ModelResult<()> {
    let mut p = read_state::<Pending>(root, PENDING)
        .ok_or_else(|| invalid("sync_plan", "原下载指针缺失，请重新同步"))?;
    if p.plan_id != original {
        return Err(invalid("sync_plan", "原下载已被其它操作替换，请重新同步"));
    }
    super::config_remote::rebind_download_guard(root, original, selected)?;
    p.plan_id = selected.into();
    write_state(root, PENDING, &p)
}
pub async fn verify_pending<T: WebDavTransport>(
    root: &Path,
    id: &str,
    binding: &str,
    transport: &T,
    config: &WebDavConfig,
) -> ModelResult<()> {
    let p = read_state::<Pending>(root, PENDING)
        .ok_or_else(|| invalid("sync_plan", "原下载指针缺失，请重新同步"))?;
    if p.plan_id != id || p.binding != binding {
        return Err(invalid("sync_plan", "下载来源或范围已变化，请重新同步"));
    }
    super::config_remote::verify_download_source(root, id, binding)?;
    let head = remote::pointer_for(remote::CONFIG, transport, config).await?;
    if head.as_ref().map(|h| &h.pointer) != Some(&p.pointer) {
        return Err(invalid(
            "remote_changed",
            "预览后云端已有后续版本，请重新同步；未覆盖本机配置",
        ));
    }
    Ok(())
}

fn digest<T: Serialize>(value: &T, strip: &[&str]) -> String {
    let mut value = serde_json::to_value(value).unwrap_or(Value::Null);
    if let Some(object) = value.as_object_mut() {
        for key in strip {
            object.remove(*key);
        }
    }
    sha256(&serde_json::to_vec(&value).unwrap_or_default())
}

/// 把一份配置整理为可比较的条目：键为（类别, 标识），值为显示名与内容摘要。
pub(super) fn facts(
    package: &ModelPackage,
    manager: &Settings,
    assets: &Summary,
    renames: &BTreeMap<String, String>,
    keep_local: &BTreeSet<String>,
) -> Facts {
    let mut out = Facts::new();
    let policy = package.payload.policy.as_ref();
    for channel in policy
        .map(|policy| policy.channels.as_slice())
        .unwrap_or_default()
    {
        let id = renames
            .get(&channel.id)
            .cloned()
            .unwrap_or_else(|| channel.id.clone());
        let mut strip = vec!["id"];
        if keep_local.contains(&id) {
            strip.extend(["connection", "label", "protocol"]);
        }
        let selected = match &channel.selection {
            ModelSelection::All => channel.model_order.len(),
            ModelSelection::Whitelist(models) => models.len(),
            _ => 0,
        };
        out.insert(
            (0, id),
            Fact {
                label: channel.label.clone(),
                digest: digest(&json!({"configuration":digest(channel,&strip),"position":policy.and_then(|p|p.channel_order.as_ref()).and_then(|order|order.iter().position(|c|c==&channel.id))}), &[]),
                selected: Some(selected),
                ..Fact::default()
            },
        );
    }
    for template in &package.payload.templates {
        let mut revisions: Vec<_> = template
            .revisions
            .iter()
            .map(|revision| (revision.id.clone(), revision.digest.clone()))
            .collect();
        revisions.sort();
        let content = json!({
            "revisions": revisions,
        });
        out.insert(
            (1, template.id.clone()),
            Fact {
                label: template.label.clone(),
                digest: digest(&content, &[]),
                version: Some(template.revisions.len()),
                ..Fact::default()
            },
        );
    }
    for account in &manager.accounts {
        let key = crate::modules::sync::portable_accounts::comparison_identity(account)
            .unwrap_or_else(|_| account.id.clone());
        out.insert(
            (2, key),
            Fact {
                label: account.label.clone(),
                digest: digest(
                    &crate::modules::sync::portable_accounts::comparison_value(account)
                        .unwrap_or(Value::Null),
                    &[],
                ),
                output_id: Some(account.id.clone()),
                ..Fact::default()
            },
        );
    }
    for skill in &assets.skills {
        out.insert(
            (3, skill.name.clone()),
            Fact {
                label: skill.name.clone(),
                digest: skill.digest.clone(),
                files: skill
                    .file_entries
                    .iter()
                    .map(|f| (f.path.clone(), f.clone()))
                    .collect(),
                ..Fact::default()
            },
        );
    }
    for mcp in &assets.mcps {
        out.insert(
            (4, mcp.name.clone()),
            Fact {
                label: mcp.name.clone(),
                digest: mcp.digest.clone(),
                ..Fact::default()
            },
        );
    }
    if manager.preferences.is_some() || manager.extensions.is_some() {
        let mut portable = serde_json::to_value(manager).unwrap_or(Value::Null);
        let preferences = portable.get("preferences").cloned().unwrap_or(Value::Null);
        let mut extensions = portable
            .get_mut("extensions")
            .cloned()
            .unwrap_or(Value::Null);
        if let Some(fields) = extensions.as_object_mut() {
            fields.remove("revision");
            fields.remove("source_store");
        }
        out.insert(
            (5, "desktop".into()),
            Fact {
                label: "通用偏好".into(),
                digest: digest(
                    &json!({"preferences":preferences,"extensions":extensions}),
                    &[],
                ),
                ..Default::default()
            },
        );
    }
    if let Some(policy) = policy {
        if policy.template_refs.is_some() || policy.default_channel_id.is_some() {
            let mut refs: Vec<(String, String)> = policy
                .template_refs
                .iter()
                .flatten()
                .map(|item| (item.template_id.clone(), item.revision_id.clone()))
                .collect();
            refs.sort();
            refs.dedup();
            let default = policy
                .default_channel_id
                .as_ref()
                .map(|id| renames.get(id).unwrap_or(id));
            out.insert(
                (6, "global".into()),
                Fact {
                    label: "全局模型设置".into(),
                    digest: digest(
                        &json!({"templateRefs": refs, "defaultChannelId": default}),
                        &[],
                    ),
                    ..Fact::default()
                },
            );
        }
    }
    out
}

/// 以新版来源为准逐项比较；只报告新增与更新，从不删除目标中多出的内容。
pub(super) fn diff(source: &Facts, target: &Facts) -> (Vec<Change>, usize) {
    let mut changes = Vec::new();
    let mut unchanged = 0;
    for ((rank, id), fact) in source {
        let kind = KINDS[usize::from(*rank).min(KINDS.len() - 1)];
        let base = Change {
            kind: kind.into(),
            id: fact.output_id.clone().unwrap_or_else(|| id.clone()),
            label: fact.label.clone(),
            ..Change::default()
        };
        match target.get(&(*rank, id.clone())) {
            None => changes.push(Change {
                mark: "new".into(),
                files_added: fact.files.keys().cloned().collect(),
                ..base
            }),
            Some(old) if old.digest != fact.digest => changes.push(Change {
                mark: "update".into(),
                version: fact.version,
                before: old.selected,
                after: fact.selected,
                files_added: fact
                    .files
                    .keys()
                    .filter(|p| !old.files.contains_key(*p))
                    .cloned()
                    .collect(),
                files_updated: fact
                    .files
                    .iter()
                    .filter(|(p, f)| old.files.get(*p).is_some_and(|old| old != *f))
                    .map(|(p, _)| p.clone())
                    .collect(),
                files_removed: old
                    .files
                    .iter()
                    .filter(|(p, f)| !f.sensitive && !fact.files.contains_key(*p))
                    .map(|(p, _)| p.clone())
                    .collect(),
                ..base
            }),
            Some(_) => unchanged += 1,
        }
    }
    (changes, unchanged)
}

/// 同名 MCP 在多个客户端定义不同且尚未选择来源时的预览项。
pub(super) fn conflict_change(conflict: &McpConflict) -> Change {
    Change {
        kind: "mcp".into(),
        id: conflict.name.clone(),
        label: conflict.name.clone(),
        mark: "conflict".into(),
        clients: conflict.clients.clone(),
        fields: conflict.fields.clone(),
        ..Change::default()
    }
}

/// 上传预览：来源为合并后的上传内容（只取本机所选条目），目标为当前远端内容。
pub(super) fn upload_changes(
    local: &Facts,
    merged: Facts,
    remote: Option<Facts>,
    conflicts: &[McpConflict],
) -> (Vec<Change>, usize) {
    let excluded: BTreeSet<(u8, String)> = conflicts
        .iter()
        .map(|conflict| (4, conflict.name.clone()))
        .collect();
    let source: Facts = merged
        .into_iter()
        .filter(|(key, _)| local.contains_key(key) && !excluded.contains(key))
        .collect();
    let mut target = remote.unwrap_or_default();
    target.retain(|key, _| !excluded.contains(key));
    let (changes, unchanged) = diff(&source, &target);
    let mut all: Vec<Change> = conflicts.iter().map(conflict_change).collect();
    all.extend(changes);
    (all, unchanged)
}
pub(super) fn include_credential_changes(
    changes: &mut Vec<Change>,
    unchanged: &mut usize,
    secrets: Vec<Change>,
) {
    for secret in secrets {
        if let Some(item) = changes
            .iter_mut()
            .find(|c| c.kind == secret.kind && c.id == secret.id)
        {
            if !item.fields.contains(&"credentials".into()) {
                item.fields.push("credentials".into());
            }
        } else {
            changes.push(secret);
            *unchanged = unchanged.saturating_sub(1);
        }
    }
}

/// 下载后记录同步状态：没有变化时直接视为已同步，否则等待确认导入。
pub(super) fn record_download(
    root: &Path,
    binding: &str,
    pointer: &Pointer,
    plan_id: &str,
    changes: Option<&[Change]>,
) -> ModelResult<()> {
    record_pending(root, binding, pointer, plan_id)?;
    if matches!(changes, Some([])) {
        record_synced(root, binding, pointer)?;
    }
    Ok(())
}

/// 用于测试与预览：本机记录的上次同步时间。
#[cfg(test)]
pub(super) fn last_synced(root: &Path, binding: &str) -> Option<String> {
    read_state::<Last>(root, LAST)
        .filter(|last| last.binding == binding)
        .map(|last| last.synced_at)
}

#[cfg(test)]
mod content_direction_tests {
    use super::*;
    #[test]
    fn same_email_accounts_with_distinct_subjects_do_not_collapse_in_preview() {
        use crate::modules::sync::portable_accounts::AccountInfo;
        let account = |id: &str, subject: &str, slot: &str| {
            serde_json::from_value::<AccountInfo>(serde_json::json!({"id":id,"provider":"chatgpt","subject":subject,"label":subject,"email":"same@example.test","origin":{"version":1,"accountPlatform":"chatgpt","owner":"opencodex","authKind":"oauth","sourceNamespace":"codex-account-pool","sourceAccountId":slot,"tenant":"shared"}})).unwrap()
        };
        let root = tempfile::tempdir().unwrap();
        let package = super::super::transfer::export(
            root.path(),
            super::super::transfer::ExportScope {
                include_policy: false,
                channel_ids: vec![],
                template_refs: vec![],
                include_global: false,
                channel_models: Default::default(),
                channel_families: Default::default(),
            },
        )
        .unwrap();
        let a = uuid::Uuid::new_v4().to_string();
        let b = uuid::Uuid::new_v4().to_string();
        let source = Settings {
            accounts: vec![
                account(&a, "personal", "source-a"),
                account(&b, "work", "source-b"),
            ],
            ..Default::default()
        };
        let target = Settings {
            accounts: vec![
                account(&uuid::Uuid::new_v4().to_string(), "personal", "target-a"),
                account(&uuid::Uuid::new_v4().to_string(), "work", "target-b"),
            ],
            ..Default::default()
        };
        let mut source = facts(
            &package,
            &source,
            &Summary::default(),
            &Default::default(),
            &Default::default(),
        );
        let mut target = facts(
            &package,
            &target,
            &Summary::default(),
            &Default::default(),
            &Default::default(),
        );
        source.retain(|(rank, _), _| *rank == 2);
        target.retain(|(rank, _), _| *rank == 2);
        assert_eq!(source.len(), 2);
        assert_eq!(diff(&source, &target), (vec![], 2));
        let (changes, _) = diff(&source, &Facts::new());
        assert_eq!(
            changes
                .iter()
                .map(|c| c.id.clone())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([a, b])
        );
    }
    #[test]
    fn saved_time_decisions_are_symmetric_and_unknown_never_selects_remote() {
        use super::super::{content_versions::TimeKind, file_objects::ContentFact};
        let fact = |digest: &str, time: Option<&str>| ContentFact {
            scope_key: "scope".into(),
            content_digest: digest.into(),
            saved_at: time.map(str::to_owned),
            time_kind: TimeKind::Saved,
            empty: false,
            version_digest: digest.into(),
        };
        let local = fact("local", Some("2026-11-04T00:00:00Z"));
        let remote = fact("remote", Some("2026-10-30T00:00:00Z"));
        assert_eq!(compare_content(&local, Some(&remote)).source, "local");
        assert_eq!(compare_content(&remote, Some(&local)).source, "remote");
        assert_eq!(compare_content(&local, Some(&local)).source, "unchanged");
        for unknown in [None, Some("broken"), Some("2026-11-04T00:00:00")] {
            assert_eq!(
                compare_content(&fact("other", unknown), Some(&local)).source,
                "undetermined"
            );
        }
        assert_eq!(
            compare_content(
                &fact("other", Some("2026-11-04T08:00:00+08:00")),
                Some(&local)
            )
            .source,
            "undetermined"
        );
        let mut mtime = remote.clone();
        mtime.time_kind = TimeKind::LegacyMtime;
        assert_eq!(compare_content(&local, Some(&mtime)).source, "undetermined");
    }
}
