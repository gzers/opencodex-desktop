//! An export retry owns one frozen destination and one package directory.
use super::{schema::*, store::sha256, transfer};
use crate::{
    infrastructure::{atomic_write::atomic_write, locking::TargetFileLock},
    modules::backup::safety,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Exported {
    pub path: String,
    pub digest: String,
    pub templates: usize,
    pub channels: usize,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Attempt {
    id: String,
    file_digest: String,
    base: PathBuf,
    date: String,
    clock: String,
    complete: bool,
    receipt: Exported,
}
fn path(root: &Path, id: &str) -> ModelResult<PathBuf> {
    require_uuid(id)?;
    let p = root
        .join("manager-state/transfer-exports")
        .join(format!("{id}.json"));
    safety::check_path(root, &p, true)?;
    Ok(p)
}
fn output(a: &Attempt) -> ModelResult<PathBuf> {
    require_uuid(&a.id)?;
    if a.date.len() != 10
        || a.date.chars().any(|c| !c.is_ascii_digit() && c != '-')
        || a.clock.len() != 6
        || !a.clock.chars().all(|c| c.is_ascii_digit())
        || !a.base.is_absolute()
    {
        return Err(invalid("export_identity", "原导出目标无效"));
    }
    let stamp = format!("opencodex-backup-{}-{}", a.clock, a.id.replace('-', ""));
    let p = a
        .base
        .join(&a.date)
        .join(&stamp)
        .join(format!("{stamp}.ocxdconf"));
    safety::check_path(&a.base, &p, true)?;
    if a.receipt.path != p.to_string_lossy() {
        return Err(invalid("export_identity", "原导出回执与目标不同"));
    }
    Ok(p)
}
fn read(root: &Path, id: &str) -> ModelResult<Option<Attempt>> {
    let p = path(root, id)?;
    if !p.exists() {
        return Ok(None);
    }
    let a: Attempt = serde_json::from_slice(&safety::read_file(&p, 64 * 1024)?)?;
    if a.id != id {
        return Err(invalid("export_identity", "导出操作身份不同"));
    }
    output(&a)?;
    Ok(Some(a))
}
fn persist(root: &Path, a: &Attempt) -> ModelResult<()> {
    atomic_write(&path(root, &a.id)?, &serde_json::to_vec(a)?, 0o600)?;
    Ok(())
}
pub fn export(root: &Path, id: &str, target: Option<&str>) -> ModelResult<Exported> {
    let state = path(root, id)?;
    let _g = TargetFileLock::lock(&state)?;
    let bytes = zeroize::Zeroizing::new(transfer::file_bytes(root, id)?);
    let package = transfer::file_package(root, id)?;
    let custom = target
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(PathBuf::from);
    if custom
        .as_ref()
        .is_some_and(|p| !p.is_absolute() || !p.is_dir())
    {
        return Err(invalid("export_target", "请选择存在的导出目录"));
    }
    let mut a = if let Some(a) = read(root, id)? {
        if a.file_digest != sha256(&bytes)
            || custom
                .as_ref()
                .map(|p| p.canonicalize())
                .transpose()?
                .as_ref()
                .is_some_and(|p| p != &a.base)
        {
            return Err(invalid(
                "export_identity",
                "原导出内容或目标已冻结，请沿原操作重试",
            ));
        }
        a
    } else {
        let base = if let Some(p) = custom {
            p.canonicalize()?
        } else {
            let p = root.join("exports");
            safety::check_path(root, &p, true)?;
            std::fs::create_dir_all(&p)?;
            p.canonicalize()?
        };
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let clock = chrono::Local::now().format("%H%M%S").to_string();
        let stamp = format!("opencodex-backup-{clock}-{}", id.replace('-', ""));
        let p = base
            .join(&date)
            .join(&stamp)
            .join(format!("{stamp}.ocxdconf"));
        let a = Attempt {
            id: id.into(),
            file_digest: sha256(&bytes),
            base,
            date,
            clock,
            complete: false,
            receipt: Exported {
                path: p.to_string_lossy().into_owned(),
                digest: package.digest,
                templates: package.payload.templates.len(),
                channels: package
                    .payload
                    .policy
                    .as_ref()
                    .map(|p| p.channels.len())
                    .unwrap_or(0),
            },
        };
        persist(root, &a)?;
        a
    };
    if a.base.canonicalize()? != a.base {
        return Err(invalid("export_target", "导出目录绑定已改变，未写入"));
    }
    let p = output(&a)?;
    if p.exists() {
        if safety::read_file(&p, 64 * 1024 * 1024)?.as_slice() != bytes.as_slice() {
            return Err(invalid(
                "export_target_changed",
                "原导出文件已改变，未覆盖；请核对原结果",
            ));
        }
    } else {
        if a.complete {
            return Err(invalid(
                "export_target_changed",
                "原导出文件已移走，未重复生成",
            ));
        }
        let dir = p.parent().unwrap();
        safety::check_path(&a.base, dir, true)?;
        if dir.exists() && std::fs::read_dir(dir)?.next().is_some() {
            return Err(invalid(
                "export_target_changed",
                "原导出包目录包含其它内容，未混写",
            ));
        }
        std::fs::create_dir_all(dir)?;
        atomic_write(&p, &bytes, 0o600)?;
    }
    if safety::read_file(&p, 64 * 1024 * 1024)?.as_slice() != bytes.as_slice() {
        return Err(invalid("export_readback", "导出未通过最终读回，原操作保留"));
    }
    a.complete = true;
    persist(root, &a)?;
    Ok(a.receipt)
}
pub fn receipt(root: &Path, id: &str) -> ModelResult<Option<Exported>> {
    Ok(read(root, id)?.filter(|a| a.complete).map(|a| a.receipt))
}
pub fn attempted(root: &Path, id: &str) -> ModelResult<bool> {
    Ok(read(root, id)?.is_some())
}
/// Protect a plan under the same lock used by export/discard so cancellation
/// cannot remove its frozen source while a protection transform is in flight.
pub fn protect(
    root: &Path,
    id: &str,
    passphrase: Option<&str>,
) -> ModelResult<transfer::FileExportPlan> {
    let state = path(root, id)?;
    let _guard = TargetFileLock::lock(&state)?;
    transfer::protect_file(root, id, passphrase)
}
/// A preview may be discarded only before any export attempt has been recorded.
/// The attempt lock serializes this decision with destination creation and retries.
pub fn discard_unattempted(root: &Path, id: &str) -> ModelResult<bool> {
    let state = path(root, id)?;
    let _guard = TargetFileLock::lock(&state)?;
    if read(root, id)?.is_some() {
        return Ok(false);
    }
    // The public operation receipt is persisted before the destination writer
    // starts. Preserve the frozen bytes on a crash/failure in that interval too.
    if super::transfer_results::read(root, Some(id))?.is_some() {
        return Ok(false);
    }
    transfer::discard_prepared_file(root, id)
}
/// Remove abandoned preview-only snapshots after the owning app instance has
/// exited. Export attempts are retained for retry; malformed and linked entries
/// fail closed. Both directory enumeration and per-startup cleanup are bounded.
pub fn discard_abandoned_previews(root: &Path) -> ModelResult<usize> {
    const MAX_PLANS_PER_STARTUP: usize = 512;
    const MAX_DIRECTORY_ENTRIES: usize = MAX_PLANS_PER_STARTUP * 3;
    let directory = root.join("manager-state/model-file-exports");
    safety::check_path(root, &directory, true)?;
    if !directory.exists() {
        return Ok(0);
    }
    let mut ids = std::collections::BTreeSet::new();
    for entry in std::fs::read_dir(&directory)
        .map_err(|error| invalid("export_discard", error.to_string()))?
        .take(MAX_DIRECTORY_ENTRIES)
    {
        let entry = entry.map_err(|error| invalid("export_discard", error.to_string()))?;
        let kind = entry
            .file_type()
            .map_err(|error| invalid("export_discard", error.to_string()))?;
        if !kind.is_file() || kind.is_symlink() {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        let id = name
            .strip_suffix(".protection-source.json")
            .or_else(|| name.strip_suffix(".ocxdconf"))
            .or_else(|| name.strip_suffix(".json"));
        let Some(id) = id else { continue };
        if uuid::Uuid::parse_str(id).is_ok() {
            ids.insert(id.to_string());
        }
    }
    // Exclude durable attempts/results before spending the cleanup budget. If
    // old attempted UUIDs sort before newer abandoned previews, slicing first
    // would starve the latter on every restart. Mere receipt-file existence is
    // enough to preserve data; a corrupt receipt must fail safe and not delete.
    let mut abandoned = Vec::new();
    for id in ids {
        let attempt_path = root
            .join("manager-state/transfer-exports")
            .join(format!("{id}.json"));
        let result_path = root
            .join("manager-state/transfer-results")
            .join(format!("{id}.json"));
        safety::check_path(root, &attempt_path, true)?;
        safety::check_path(root, &result_path, true)?;
        if attempt_path.exists() || result_path.exists() {
            continue;
        }
        abandoned.push(id);
    }
    let mut removed = 0;
    for id in abandoned.into_iter().take(MAX_PLANS_PER_STARTUP) {
        if discard_unattempted(root, &id)? {
            removed += 1;
        }
    }
    Ok(removed)
}
pub fn directory(root: &Path, id: &str) -> ModelResult<PathBuf> {
    let a = read(root, id)?
        .filter(|a| a.complete)
        .ok_or_else(|| invalid("export_result", "原导出尚未完成"))?;
    let p = output(&a)?;
    let dir = p.parent().unwrap();
    safety::check_path(&a.base, dir, false)?;
    Ok(dir.into())
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
            &crate::modules::sync::portable_manager::Scope {
                preferences: true,
                ..Default::default()
            },
        )
        .unwrap()
    }
    #[test]
    fn retry_reuses_the_custom_package_and_never_overwrites_later_content() {
        let root = tempfile::tempdir().unwrap();
        let custom = tempfile::tempdir().unwrap();
        let plan = transfer::prepare_file_scope(
            root.path(),
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
            &crate::modules::sync::portable_manager::Scope {
                preferences: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(std::fs::read_dir(custom.path()).unwrap().count(), 0);
        let first = export(root.path(), &plan.id, Some(custom.path().to_str().unwrap())).unwrap();
        let second = export(root.path(), &plan.id, None).unwrap();
        assert_eq!(first.path, second.path);
        assert!(directory(root.path(), &plan.id)
            .unwrap()
            .starts_with(custom.path().canonicalize().unwrap()));
        std::fs::write(&first.path, b"later edit").unwrap();
        assert_eq!(
            export(root.path(), &plan.id, None).unwrap_err().code,
            "export_target_changed"
        );
        assert_eq!(std::fs::read(&first.path).unwrap(), b"later edit");
    }
    #[test]
    fn canceled_unattempted_preview_removes_only_the_frozen_plan_not_export_destination() {
        let root = tempfile::tempdir().unwrap();
        let plan = export_plan(root.path());
        let metadata = root
            .path()
            .join("manager-state/model-file-exports")
            .join(format!("{}.json", plan.id));
        let package = metadata.with_extension("ocxdconf");
        assert!(metadata.is_file() && package.is_file());
        assert!(!root.path().join("exports").exists());

        assert!(discard_unattempted(root.path(), &plan.id).unwrap());
        assert!(!metadata.exists());
        assert!(!package.exists());
        assert!(!metadata.with_extension("protection-source.json").exists());
        assert!(!root.path().join("exports").exists());
        assert!(!discard_unattempted(root.path(), &plan.id).unwrap());
    }

    #[test]
    fn canceling_a_protected_preview_removes_plaintext_source_and_protected_child() {
        let root = tempfile::tempdir().unwrap();
        let plain = export_plan(root.path());
        let plain_path = root
            .path()
            .join("manager-state/model-file-exports")
            .join(format!("{}.json", plain.id));
        let protected = protect(root.path(), &plain.id, Some("fixture-passphrase")).unwrap();
        assert!(protected.protected);
        let protected_path = root
            .path()
            .join("manager-state/model-file-exports")
            .join(format!("{}.json", protected.id));
        assert!(plain_path.with_extension("ocxdconf").is_file());
        assert!(protected_path.with_extension("ocxdconf").is_file());

        assert!(discard_unattempted(root.path(), &plain.id).unwrap());
        assert!(discard_unattempted(root.path(), &protected.id).unwrap());
        assert!(!plain_path.exists());
        assert!(!plain_path.with_extension("ocxdconf").exists());
        assert!(!protected_path.exists());
        assert!(!protected_path.with_extension("ocxdconf").exists());
        assert!(!protected_path
            .with_extension("protection-source.json")
            .exists());
    }

    #[test]
    fn export_attempt_keeps_frozen_package_for_original_operation_retry() {
        let root = tempfile::tempdir().unwrap();
        let plan = export_plan(root.path());
        let metadata = root
            .path()
            .join("manager-state/model-file-exports")
            .join(format!("{}.json", plan.id));
        let package = metadata.with_extension("ocxdconf");
        let receipt = export(root.path(), &plan.id, None).unwrap();

        assert!(!discard_unattempted(root.path(), &plan.id).unwrap());
        assert!(metadata.is_file());
        assert!(package.is_file());
        assert!(Path::new(&receipt.path).is_file());
        assert_eq!(
            directory(root.path(), &plan.id).unwrap(),
            Path::new(&receipt.path).parent().unwrap()
        );
    }

    #[test]
    fn durable_failed_export_receipt_preserves_plan_even_before_destination_attempt_record() {
        let root = tempfile::tempdir().unwrap();
        let plan = export_plan(root.path());
        let metadata = root
            .path()
            .join("manager-state/model-file-exports")
            .join(format!("{}.json", plan.id));
        let package = metadata.with_extension("ocxdconf");
        let result: ModelResult<()> =
            super::super::transfer_results::export(root.path(), &plan.id, || {
                Err(invalid("fixture_before_destination", "fixture refusal"))
            });
        assert_eq!(result.unwrap_err().code, "fixture_before_destination");
        assert!(!attempted(root.path(), &plan.id).unwrap());

        assert!(!discard_unattempted(root.path(), &plan.id).unwrap());
        assert!(metadata.is_file());
        assert!(package.is_file());
        assert_eq!(
            super::super::transfer_results::read(root.path(), Some(&plan.id))
                .unwrap()
                .unwrap()
                .phase,
            "failed"
        );
    }

    #[test]
    fn startup_prunes_abandoned_preview_plans_but_retains_attempted_exports() {
        let root = tempfile::tempdir().unwrap();
        let abandoned = export_plan(root.path());
        let attempted_plan = export_plan(root.path());
        let attempted_metadata = root
            .path()
            .join("manager-state/model-file-exports")
            .join(format!("{}.json", attempted_plan.id));
        let attempted_payload = attempted_metadata.with_extension("ocxdconf");
        export(root.path(), &attempted_plan.id, None).unwrap();

        assert_eq!(discard_abandoned_previews(root.path()).unwrap(), 1);
        assert!(!root
            .path()
            .join("manager-state/model-file-exports")
            .join(format!("{}.json", abandoned.id))
            .exists());
        assert!(attempted_metadata.is_file());
        assert!(attempted_payload.is_file());
        assert!(receipt(root.path(), &attempted_plan.id).unwrap().is_some());
    }

    #[test]
    fn startup_attempted_receipts_do_not_consume_the_abandoned_preview_budget() {
        let root = tempfile::tempdir().unwrap();
        let abandoned = export_plan(root.path());
        let attempted_plan = export_plan(root.path());
        let attempt_path = root
            .path()
            .join("manager-state/transfer-exports")
            .join(format!("{}.json", attempted_plan.id));
        std::fs::create_dir_all(attempt_path.parent().unwrap()).unwrap();
        // Malformed is still a durable marker for preservation: startup cleanup
        // must never remove the package merely because retry metadata is corrupt.
        std::fs::write(&attempt_path, b"unreadable attempted operation").unwrap();
        let attempted_package = root
            .path()
            .join("manager-state/model-file-exports")
            .join(format!("{}.ocxdconf", attempted_plan.id));

        assert_eq!(discard_abandoned_previews(root.path()).unwrap(), 1);
        assert!(!root
            .path()
            .join("manager-state/model-file-exports")
            .join(format!("{}.json", abandoned.id))
            .exists());
        assert!(attempted_package.is_file());
    }
}
