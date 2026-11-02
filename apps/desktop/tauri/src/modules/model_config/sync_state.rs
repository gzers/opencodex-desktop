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

/// 判断本次同步方向：远端未变化或为空时以本机为新版，否则以云端为新版。
pub async fn check<T: WebDavTransport>(
    root: &Path,
    transport: &T,
    config: &WebDavConfig,
    binding: &str,
) -> ModelResult<Check> {
    let head = remote::pointer_for(remote::CONFIG, transport, config).await?;
    let last = read_state::<Last>(root, LAST).filter(|last| last.binding == binding);
    let local = match (&head, &last) {
        (None, _) => true,
        (Some(head), Some(last)) => head.pointer == last.pointer,
        _ => false,
    };
    Ok(Check {
        source: if local { "local" } else { "remote" }.into(),
        has_remote: head.is_some(),
        last_synced_at: last.map(|last| last.synced_at),
    })
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
                digest: digest(channel, &strip),
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
            "label": template.label,
            "defaultRevision": template.default_revision,
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
        let key = match (&account.email, &account.subject) {
            (Some(email), _) => format!("{}:{}", account.provider, email),
            (None, Some(subject)) => format!("{}:{}", account.provider, subject),
            _ => account.id.clone(),
        };
        out.insert(
            (2, key),
            Fact {
                label: account.label.clone(),
                digest: digest(account, &["id", "origin"]),
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
    if let Some(preferences) = &manager.preferences {
        out.insert(
            (5, "desktop".into()),
            Fact {
                label: "桌面偏好".into(),
                digest: digest(
                    preferences,
                    &["revision", "source_store", "sourceStore", "schema_version"],
                ),
                ..Fact::default()
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
    if let Some(extensions) = &manager.extensions {
        out.insert(
            (6, "extensions".into()),
            Fact {
                label: "扩展设置".into(),
                digest: digest(extensions, &["revision"]),
                ..Fact::default()
            },
        );
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
            id: id.clone(),
            label: fact.label.clone(),
            ..Change::default()
        };
        match target.get(&(*rank, id.clone())) {
            None => changes.push(Change {
                mark: "new".into(),
                ..base
            }),
            Some(old) if old.digest != fact.digest => changes.push(Change {
                mark: "update".into(),
                version: fact.version,
                before: old.selected,
                after: fact.selected,
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

/// 下载后记录同步状态：没有变化时直接视为已同步，否则等待确认导入。
pub(super) fn record_download(
    root: &Path,
    binding: &str,
    pointer: &Pointer,
    plan_id: &str,
    changes: Option<&[Change]>,
) -> ModelResult<()> {
    match changes {
        Some([]) => record_synced(root, binding, pointer),
        _ => record_pending(root, binding, pointer, plan_id),
    }
}

/// 用于测试与预览：本机记录的上次同步时间。
#[cfg(test)]
pub(super) fn last_synced(root: &Path, binding: &str) -> Option<String> {
    read_state::<Last>(root, LAST)
        .filter(|last| last.binding == binding)
        .map(|last| last.synced_at)
}
