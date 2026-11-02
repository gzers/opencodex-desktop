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
}
