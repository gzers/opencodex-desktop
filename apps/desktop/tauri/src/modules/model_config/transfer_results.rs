//! Durable, read-only transfer receipts. Reading a result never retries or
//! contacts WebDAV; only an explicit retry invokes the original frozen plan.
use super::{config_remote, file_exports, schema::*, transfer};
use crate::{
    infrastructure::atomic_write::atomic_write,
    modules::{
        backup::safety,
        sync::{portable_assets, portable_manager},
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Export,
    Import,
    Upload,
    Download,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RetryIntent {
    pub adopt_keys: bool,
    pub adopt_webdav_passwords: bool,
    pub adopt_extension_credentials: bool,
    pub backup_before_update: bool,
    pub credential_ids: Option<Vec<String>>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Item {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub status: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Receipt {
    pub id: String,
    pub kind: Kind,
    pub phase: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub items: Vec<Item>,
    pub error_code: Option<String>,
    pub export_path: Option<String>,
    pub source_digest: String,
    #[serde(default)]
    pub retry: Option<RetryIntent>,
}
fn path(root: &Path, id: &str) -> ModelResult<PathBuf> {
    require_uuid(id)?;
    let p = root
        .join("manager-state/transfer-results")
        .join(format!("{id}.json"));
    safety::check_path(root, &p, true)?;
    Ok(p)
}
fn index(root: &Path) -> ModelResult<PathBuf> {
    let p = root.join("manager-state/transfer-results/latest.json");
    safety::check_path(root, &p, true)?;
    Ok(p)
}
fn mode_index(root: &Path, remote: bool) -> ModelResult<PathBuf> {
    let p = root.join("manager-state/transfer-results").join(if remote {
        "latest-webdav.json"
    } else {
        "latest-file.json"
    });
    safety::check_path(root, &p, true)?;
    Ok(p)
}
pub fn read_mode(root: &Path, mode: &str) -> ModelResult<Option<Receipt>> {
    let remote = match mode {
        "file" => false,
        "webdav" => true,
        _ => return Err(invalid("transfer_result", "结果类型无效")),
    };
    let p = mode_index(root, remote)?;
    if !p.exists() {
        return Ok(None);
    }
    let id: String = serde_json::from_slice(&safety::read_file(&p, 4096)?)?;
    read(root, Some(&id))
}
pub fn read(root: &Path, id: Option<&str>) -> ModelResult<Option<Receipt>> {
    let owned;
    if let Some(id) = id {
        owned = id.to_owned()
    } else {
        let p = index(root)?;
        if !p.exists() {
            return Ok(None);
        }
        owned = serde_json::from_slice(&safety::read_file(&p, 4096)?)?;
    }
    let p = path(root, &owned)?;
    if !p.exists() {
        return Ok(None);
    }
    let r: Receipt = serde_json::from_slice(&safety::read_file(&p, 8 * 1024 * 1024)?)?;
    if r.id != owned || r.items.len() > 30000 {
        return Err(invalid("transfer_result", "结果身份或范围无效"));
    }
    Ok(Some(r))
}
fn persist(root: &Path, r: &Receipt) -> ModelResult<()> {
    let bytes = serde_json::to_vec(r)?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err(invalid("transfer_result", "本次结果范围超过上限"));
    }
    atomic_write(&path(root, &r.id)?, &bytes, 0o600)?;
    atomic_write(&index(root)?, &serde_json::to_vec(&r.id)?, 0o600)?;
    atomic_write(
        &mode_index(root, matches!(r.kind, Kind::Upload | Kind::Download))?,
        &serde_json::to_vec(&r.id)?,
        0o600,
    )?;
    Ok(())
}
fn inventory(
    package: &transfer::ModelPackage,
    manager: &portable_manager::Settings,
    assets: &portable_assets::Summary,
) -> Vec<Item> {
    let mut rows = vec![];
    let mut push = |kind: &str, id: &str, label: &str| {
        rows.push(Item {
            id: format!("{kind}:{id}"),
            label: label.into(),
            kind: kind.into(),
            status: "pending".into(),
        })
    };
    for t in &package.payload.templates {
        push("templates", &t.id, &t.label)
    }
    for c in package.payload.policy.iter().flat_map(|p| &p.channels) {
        push("channels", &c.id, &c.label)
    }
    for a in &manager.accounts {
        push("accounts", &a.id, &a.label)
    }
    for s in &assets.skills {
        push("skills", &s.name, &format!("Skill · {}", s.name))
    }
    for m in &assets.mcps {
        push("mcp", &m.name, &format!("MCP · {}", m.name))
    }
    if manager.preferences.is_some() || manager.extensions.is_some() {
        push("preferences", "desktop", "通用偏好")
    }
    for e in &manager.webdav {
        push("webdav", &e.id, "WebDAV连接配置")
    }
    for c in &assets.conflicts {
        rows.push(Item {
            id: format!("mcp:{}", c.name),
            label: format!("MCP · {}", c.name),
            kind: "mcp".into(),
            status: "skipped".into(),
        })
    }
    rows
}
fn begin(
    root: &Path,
    id: &str,
    kind: Kind,
    digest: String,
    items: Vec<Item>,
    retry: Option<RetryIntent>,
) -> ModelResult<Receipt> {
    let r = if let Some(mut old) = read(root, Some(id))? {
        if old.kind != kind || old.source_digest != digest || old.retry != retry {
            return Err(invalid("transfer_result", "原操作的来源或类型已改变"));
        }
        old.phase = "running".into();
        old.error_code = None;
        old
    } else {
        Receipt {
            id: id.into(),
            kind,
            phase: "running".into(),
            started_at: chrono::Utc::now().to_rfc3339(),
            finished_at: None,
            items,
            error_code: None,
            export_path: None,
            source_digest: digest,
            retry,
        }
    };
    persist(root, &r)?;
    Ok(r)
}
fn finish<T>(root: &Path, mut receipt: Receipt, result: &ModelResult<T>) -> ModelResult<()> {
    let done = receipt.items.iter().any(|i| i.status == "completed");
    receipt.phase = if result.is_ok() {
        if receipt.items.iter().any(|i| i.status == "skipped") {
            "partial"
        } else {
            "complete"
        }
    } else if done {
        "partial"
    } else {
        "failed"
    }
    .into();
    receipt.finished_at = Some(chrono::Utc::now().to_rfc3339());
    receipt.error_code = result.as_ref().err().map(|e| {
        if e.code.len() <= 64
            && e.code
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_')
        {
            e.code.clone()
        } else {
            "operation_failed".into()
        }
    });
    persist(root, &receipt)
}
pub fn export<T>(root: &Path, id: &str, action: impl FnOnce() -> ModelResult<T>) -> ModelResult<T> {
    let plan = transfer::file_summary(root, id)?;
    let mut receipt = begin(
        root,
        id,
        Kind::Export,
        plan.file_digest.clone(),
        inventory(
            &plan.package,
            &plan.manager.unwrap_or_default(),
            &plan.assets.unwrap_or_default(),
        ),
        None,
    )?;
    let result = action();
    if result.is_ok() {
        for i in &mut receipt.items {
            if i.status != "skipped" {
                i.status = "completed".into()
            }
        }
        receipt.export_path = file_exports::receipt(root, id)?.map(|r| r.path)
    }
    finish(root, receipt, &result)?;
    result
}
pub fn upload<T>(root: &Path, id: &str, action: impl FnOnce() -> ModelResult<T>) -> ModelResult<T> {
    let plan = config_remote::upload_result(root, id)?;
    let selected = if let Some(local) = config_remote::upload_selection(root, id)? {
        inventory(
            &local.package,
            &local.manager.unwrap_or_default(),
            &local.assets.unwrap_or_default(),
        )
    } else {
        vec![Item {
            id: format!("operation:{id}"),
            label: "原上传范围（旧版记录）".into(),
            kind: "operation".into(),
            status: "pending".into(),
        }]
    };
    let mut receipt = begin(
        root,
        id,
        Kind::Upload,
        plan.package.digest.clone(),
        selected,
        None,
    )?;
    let result = action();
    let after = config_remote::upload_result(root, id)?;
    if after.phase == "published" {
        for i in &mut receipt.items {
            if i.status != "skipped" {
                i.status = "completed".into()
            }
        }
    }
    finish(root, receipt, &result)?;
    result
}
pub fn import(
    root: &Path,
    id: &str,
    mut intent: RetryIntent,
    action: impl FnOnce() -> ModelResult<Snapshot>,
) -> ModelResult<Snapshot> {
    if let Some(ids) = &mut intent.credential_ids {
        ids.sort();
        ids.dedup();
    }
    let plan = transfer::read_incoming(root, id)?;
    let kind = if plan.sync.is_some() {
        Kind::Download
    } else {
        Kind::Import
    };
    let before = transfer::local_for_plan(root, &plan)?;
    let mut items = inventory(
        &plan.package,
        &plan.manager.clone().unwrap_or_default(),
        &plan.assets.clone().unwrap_or_default(),
    );
    for i in &mut items {
        if i.kind == "templates" {
            let id = i.id.strip_prefix("templates:").unwrap();
            if plan
                .package
                .payload
                .templates
                .iter()
                .find(|t| t.id == id)
                .is_some_and(|t| {
                    t.revisions.iter().all(|r| {
                        before.library.templates.iter().any(|local| {
                            local.id == t.id
                                && local
                                    .revisions
                                    .iter()
                                    .any(|v| v.id == r.id && v.digest == r.digest)
                        })
                    })
                })
            {
                i.status = "unchanged".into()
            }
        }
    }
    let mut receipt = begin(
        root,
        id,
        kind,
        plan.file_digest
            .clone()
            .unwrap_or_else(|| plan.package.digest.clone()),
        items,
        Some(intent),
    )?;
    let result = action();
    let current = transfer::local_for_plan(root, &plan)?;
    let models_done = transfer::accepted(root, &plan)?;
    let (assets_done, skills, mcp_done) = portable_assets::apply_progress(root, id)?;
    let manager_done = portable_manager::applied_paths(root, id)?;
    let templates: BTreeSet<_> = plan
        .package
        .payload
        .templates
        .iter()
        .filter(|t| {
            t.revisions.iter().all(|r| {
                current.library.templates.iter().any(|local| {
                    local.id == t.id
                        && local
                            .revisions
                            .iter()
                            .any(|v| v.id == r.id && v.digest == r.digest)
                })
            })
        })
        .map(|t| t.id.as_str())
        .collect();
    for i in &mut receipt.items {
        let id = i.id.split_once(':').map(|p| p.1).unwrap_or("");
        let done = match i.kind.as_str() {
            "channels" => models_done,
            "templates" => templates.contains(id),
            "skills" => {
                assets_done
                    || skills.contains(
                        plan.asset_mappings
                            .skills
                            .get(id)
                            .map(String::as_str)
                            .unwrap_or(id),
                    )
            }
            "mcp" => mcp_done,
            "preferences" => {
                (plan
                    .manager
                    .as_ref()
                    .is_none_or(|m| m.preferences.is_none())
                    || manager_done.contains("manager-state/preferences.json"))
                    && (plan.manager.as_ref().is_none_or(|m| m.extensions.is_none())
                        || manager_done.contains("manager-state/extension-config.json"))
            }
            "extensions" => manager_done.contains("manager-state/extension-config.json"),
            "accounts" => manager_done.contains("manager-state/imported-account-info.json"),
            "webdav" => manager_done.contains("manager-state/sync-imported-endpoints.json"),
            _ => false,
        };
        if done && i.status != "unchanged" {
            i.status = "completed".into()
        }
        if result.is_ok()
            && plan
                .sync
                .as_ref()
                .and_then(|s| s.changes.as_ref())
                .is_some_and(|changes| !changes.iter().any(|c| c.kind == i.kind && c.id == id))
        {
            i.status = "unchanged".into()
        }
    }
    finish(root, receipt, &result)?;
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    fn export_plan(root: &Path) -> transfer::FileExportPlan {
        transfer::prepare_file_scope(
            root,
            transfer::ExportScope {
                include_policy: false,
                channel_ids: vec![],
                template_refs: vec![],
                include_global: false,
                channel_models: Default::default(),
                channel_families: Default::default(),
            },
            None,
            false,
            &portable_manager::Scope {
                preferences: true,
                ..Default::default()
            },
        )
        .unwrap()
    }
    #[test]
    fn reading_a_failed_export_is_read_only_and_retry_reuses_its_identity() {
        let r = tempfile::tempdir().unwrap();
        let plan = export_plan(r.path());
        let failure: ModelResult<()> = export(r.path(), &plan.id, || {
            Err(invalid("export_fixture_write", "fixture denied"))
        });
        assert!(failure.is_err());
        let before = std::fs::read(path(r.path(), &plan.id).unwrap()).unwrap();
        let receipt = read(r.path(), None).unwrap().unwrap();
        assert_eq!(receipt.phase, "failed");
        assert!(receipt.export_path.is_none());
        assert!(!r.path().join("exports").exists());
        assert_eq!(
            std::fs::read(path(r.path(), &plan.id).unwrap()).unwrap(),
            before
        );
        let output = export(r.path(), &plan.id, || {
            file_exports::export(r.path(), &plan.id, None)
        })
        .unwrap();
        let done = read(r.path(), Some(&plan.id)).unwrap().unwrap();
        assert_eq!(done.phase, "complete");
        assert_eq!(done.export_path.as_deref(), Some(output.path.as_str()));
        assert!(done.items.iter().all(|i| i.status == "completed"));
    }
    #[test]
    fn import_retry_cannot_change_the_original_backup_or_credential_choice() {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        std::fs::write(home.path().join("config.json"), b"{}").unwrap();
        let plan = export_plan(source.path());
        let bytes = transfer::file_bytes(source.path(), &plan.id).unwrap();
        let incoming = super::super::file_objects::receive_context_file(
            target.path(),
            &bytes,
            None,
            None,
            home.path(),
        )
        .unwrap();
        let intent = RetryIntent {
            adopt_keys: false,
            adopt_webdav_passwords: false,
            adopt_extension_credentials: false,
            backup_before_update: false,
            credential_ids: Some(vec![]),
        };
        import(target.path(), &incoming.id, intent.clone(), || {
            super::super::file_objects::accept(target.path(), &incoming.id, false, None)
        })
        .unwrap();
        let before = std::fs::read(path(target.path(), &incoming.id).unwrap()).unwrap();
        let changed = RetryIntent {
            backup_before_update: true,
            ..intent
        };
        let rejected = import(target.path(), &incoming.id, changed, || {
            panic!("changed intent must never execute")
        });
        assert_eq!(rejected.unwrap_err().code, "transfer_result");
        assert_eq!(
            std::fs::read(path(target.path(), &incoming.id).unwrap()).unwrap(),
            before
        );
    }
}
