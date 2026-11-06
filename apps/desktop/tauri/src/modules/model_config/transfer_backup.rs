//! User-restorable snapshots, bound to the original operation and scope.
//! Retry never replaces original bytes. Restore derives paths from the plan.
use super::{schema::*, transfer};
use crate::{
    infrastructure::{atomic_write::atomic_write, locking::TargetFileLock},
    modules::{backup::safety, sync::portable_assets},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
const LIMIT: u64 = 128 * 1024 * 1024;
type Target = (String, PathBuf, PathBuf);
#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Image {
    pub(crate) exists: bool,
    pub(crate) directory: bool,
    #[serde(with = "file_bytes")]
    pub(crate) files: BTreeMap<String, Vec<u8>>,
    #[serde(default)]
    pub(crate) executable: BTreeSet<String>,
}
mod file_bytes {
    use super::*;
    use base64::{engine::general_purpose::STANDARD, Engine};
    pub fn serialize<S: serde::Serializer>(
        files: &BTreeMap<String, Vec<u8>>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        files
            .iter()
            .map(|(p, b)| (p, STANDARD.encode(b)))
            .collect::<BTreeMap<_, _>>()
            .serialize(s)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        d: D,
    ) -> Result<BTreeMap<String, Vec<u8>>, D::Error> {
        let values = BTreeMap::<String, String>::deserialize(d)?;
        values
            .into_iter()
            .map(|(p, b)| {
                STANDARD
                    .decode(b)
                    .map(|bytes| (p, bytes))
                    .map_err(|_| serde::de::Error::custom("invalid backup file encoding"))
            })
            .collect()
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    id: String,
    digest: String,
    enabled: bool,
    groups: Vec<String>,
    before: Vec<Image>,
    after: Option<Vec<Image>>,
    integrity: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub id: String,
    pub enabled: bool,
    pub ready: bool,
    pub groups: Vec<String>,
}
fn path(root: &Path, id: &str) -> ModelResult<PathBuf> {
    require_uuid(id)?;
    let p = root
        .join("backups/transfer-scope")
        .join(format!("{id}.json"));
    safety::check_path(root, &p, true)?;
    Ok(p)
}
fn targets(root: &Path, id: &str) -> ModelResult<Vec<Target>> {
    let p = transfer::read_incoming(root, id)?;
    let mut out = vec![];
    if !p.package.payload.templates.is_empty() {
        out.push(("templates".into(), root.into(), root.join(LIBRARY_PATH)))
    }
    if p.package.payload.policy.is_some() {
        if let Some(home) = &p.file_home {
            out.push(("channels".into(), home.clone(), home.join("config.json")));
            out.push((
                "channels".into(),
                root.into(),
                root.join("manager-state/model-channel-layout.json"),
            ))
        } else {
            out.push(("channels".into(), root.into(), root.join(POLICY_PATH)));
            out.push(("channels".into(), root.into(), root.join(DRAFT_PATH)))
        }
    }
    if let Some(m) = p.manager {
        for (selected, group, relative) in [
            (
                m.preferences.is_some(),
                "preferences",
                "manager-state/preferences.json",
            ),
            (
                m.extensions.is_some(),
                "extensions",
                "manager-state/extension-config.json",
            ),
            (
                !m.webdav.is_empty(),
                "webdav",
                "manager-state/sync-imported-endpoints.json",
            ),
            (
                !m.accounts.is_empty(),
                "accounts",
                "manager-state/imported-account-info.json",
            ),
        ] {
            if selected {
                out.push((group.into(), root.into(), root.join(relative)))
            }
        }
    }
    if p.assets
        .as_ref()
        .is_some_and(|a| !a.skills.is_empty() || !a.mcps.is_empty())
    {
        let mcps = p.assets.as_ref().is_some_and(|a| !a.mcps.is_empty());
        out.extend(
            portable_assets::backup_targets(root, id)?
                .into_iter()
                .filter(|t| t.0 != "mcp" || mcps),
        )
    }
    let mut seen = BTreeSet::new();
    out.retain(|(_, _, p)| seen.insert(p.clone()));
    Ok(out)
}
pub(crate) fn image(base: &Path, target: &Path) -> ModelResult<Image> {
    safety::check_path(base, target, true)?;
    let meta = match std::fs::symlink_metadata(target) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Image::default()),
        Err(e) => return Err(e.into()),
    };
    if meta.file_type().is_symlink() {
        return Err(invalid("scope_backup", "备份范围不允许软链接"));
    }
    let mut out = Image {
        exists: true,
        directory: meta.is_dir(),
        files: BTreeMap::new(),
        executable: BTreeSet::new(),
    };
    let mut total = 0;
    fn walk(
        base: &Path,
        path: &Path,
        prefix: &str,
        out: &mut BTreeMap<String, Vec<u8>>,
        total: &mut u64,
        depth: usize,
    ) -> ModelResult<()> {
        if depth > 32 || out.len() > 20000 {
            return Err(invalid("scope_backup", "备份范围超过上限"));
        }
        safety::check_path(base, path, false)?;
        let m = std::fs::symlink_metadata(path)?;
        if m.file_type().is_symlink() {
            return Err(invalid("scope_backup", "备份范围不允许软链接"));
        }
        if m.is_dir() {
            for item in std::fs::read_dir(path)? {
                let item = item?;
                let n = item
                    .file_name()
                    .into_string()
                    .map_err(|_| invalid("scope_backup", "资源文件名无法备份"))?;
                let next = if prefix.is_empty() {
                    n
                } else {
                    format!("{prefix}/{n}")
                };
                walk(base, &item.path(), &next, out, total, depth + 1)?
            }
        } else if m.is_file() {
            let bytes = safety::read_file(path, LIMIT)?;
            *total += bytes.len() as u64;
            if *total > LIMIT {
                return Err(invalid("scope_backup", "备份内容超过上限"));
            }
            out.insert(prefix.into(), bytes);
        } else {
            return Err(invalid("scope_backup", "备份包含非普通文件"));
        }
        Ok(())
    }
    walk(base, target, "", &mut out.files, &mut total, 0)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for name in out.files.keys() {
            let p = if name.is_empty() {
                target.to_path_buf()
            } else {
                target.join(name)
            };
            if std::fs::metadata(p)?.permissions().mode() & 0o111 != 0 {
                out.executable.insert(name.clone());
            }
        }
    }
    Ok(out)
}
fn integrity(r: &Record) -> ModelResult<String> {
    Ok(super::store::sha256(&serde_json::to_vec(&(
        &r.id, &r.digest, r.enabled, &r.groups, &r.before, &r.after,
    ))?))
}
fn persist(root: &Path, r: &mut Record) -> ModelResult<()> {
    r.integrity = integrity(r)?;
    atomic_write(&path(root, &r.id)?, &serde_json::to_vec(r)?, 0o600)?;
    Ok(())
}
fn read(root: &Path, id: &str) -> ModelResult<Record> {
    let r: Record = serde_json::from_slice(&safety::read_file(&path(root, id)?, LIMIT * 6)?)?;
    if r.integrity != integrity(&r)? {
        return Err(invalid(
            "scope_backup_integrity",
            "更新前备份未通过完整性校验，未恢复",
        ));
    }
    let p = transfer::read_incoming(root, id)?;
    if r.id != id || r.digest != p.package.digest {
        return Err(invalid("scope_backup", "备份与原操作身份不同"));
    }
    let t = targets(root, id)?;
    for image in r.before.iter().chain(r.after.iter().flatten()) {
        if image.files.len() > 20000
            || image.files.iter().any(|(name, b)| {
                b.len() as u64 > LIMIT
                    || if image.directory {
                        name.is_empty()
                            || name.contains(['\\', ':'])
                            || Path::new(name)
                                .components()
                                .any(|p| !matches!(p, std::path::Component::Normal(_)))
                    } else {
                        !name.is_empty()
                    }
            })
            || image
                .executable
                .iter()
                .any(|p| !image.files.contains_key(p))
        {
            return Err(invalid("scope_backup", "备份文件范围或属性无效"));
        }
    }
    if r.after.as_ref().is_some_and(|a| a.len() != r.before.len()) {
        return Err(invalid("scope_backup", "备份结果范围无效"));
    }
    if r.enabled
        && (r.before.len() != t.len()
            || r.groups != t.iter().map(|t| t.0.clone()).collect::<Vec<_>>())
    {
        return Err(invalid("scope_backup", "备份范围与原操作不同"));
    }
    Ok(r)
}
pub fn prepare(root: &Path, id: &str, enabled: bool) -> ModelResult<()> {
    let file = path(root, id)?;
    let _g = TargetFileLock::lock(&file)?;
    if file.exists() {
        let r = read(root, id)?;
        if r.enabled != enabled {
            return Err(invalid("scope_backup", "原操作的更新前备份选择已冻结"));
        }
        return Ok(());
    }
    let p = transfer::read_incoming(root, id)?;
    if p.preview_error.is_some() || p.comparison.as_ref().is_some_and(|p| p.changes.is_none()) {
        return Err(invalid("scope_backup", "配置仍有冲突，未开始备份"));
    }
    let local = transfer::local_for_plan(root, &p)?;
    if local.policy_digest != p.expected_policy
        || local.library_digest != p.expected_library
        || local.workspace_digest != p.expected_workspace
        || !transfer::file_layout_matches(root, &p)?
    {
        return Err(invalid("stale_baseline", "配置已改变，未生成更新前备份"));
    }
    let t = if enabled { targets(root, id)? } else { vec![] };
    let before = t
        .iter()
        .map(|(_, base, p)| image(base, p))
        .collect::<ModelResult<Vec<_>>>()?;
    if before
        .iter()
        .flat_map(|i| i.files.values())
        .map(|b| b.len() as u64)
        .sum::<u64>()
        > LIMIT
    {
        return Err(invalid("scope_backup", "本次备份总内容超过上限"));
    }
    let mut r = Record {
        id: id.into(),
        digest: p.package.digest,
        enabled,
        groups: t.iter().map(|t| t.0.clone()).collect(),
        before,
        after: None,
        integrity: String::new(),
    };
    persist(root, &mut r)?;
    let check = read(root, id)?;
    if check.before != r.before {
        return Err(invalid("scope_backup", "更新前备份未通过读回"));
    }
    Ok(())
}
pub fn finish(root: &Path, id: &str) -> ModelResult<()> {
    let file = path(root, id)?;
    let _g = TargetFileLock::lock(&file)?;
    let mut r = read(root, id)?;
    if r.enabled {
        let plan = transfer::read_incoming(root, id)?;
        let model_done = transfer::accepted(root, &plan)?;
        let manager = crate::modules::sync::portable_manager::applied_files(root, id)?;
        let file_material = transfer::saved_file_material(root, &plan)?;
        let (asset_done, skills, mcp_done) = portable_assets::apply_progress(root, id)?;
        let snapshot = transfer::local_for_plan(root, &plan)?;
        let t = targets(root, id)?;
        let mut after = r.after.clone().unwrap_or_else(|| r.before.clone());
        for (i, (group, base, path)) in t.iter().enumerate() {
            if after[i] != r.before[i] {
                continue;
            }
            let done = match group.as_str() {
                "channels" => model_done,
                "templates" => snapshot.library == plan.merged_library,
                "mcp" => mcp_done,
                g if g.starts_with("skill:") => asset_done || skills.contains(&g[6..]),
                _ => manager.keys().any(|p| root.join(p) == *path),
            };
            if done {
                let bytes = if group == "channels" {
                    file_material.as_ref().map(|(raw, metadata)| {
                        if plan
                            .file_home
                            .as_ref()
                            .is_some_and(|h| h.join("config.json") == *path)
                        {
                            raw.clone()
                        } else {
                            metadata.clone()
                        }
                    })
                } else if group == "mcp" {
                    portable_assets::completed_mcp_bytes(root, id)?
                } else {
                    manager
                        .iter()
                        .find(|(p, _)| root.join(p) == *path)
                        .map(|(_, b)| b.clone())
                };
                if let Some(bytes) = bytes {
                    after[i] = Image {
                        exists: true,
                        directory: false,
                        files: BTreeMap::from([(String::new(), bytes)]),
                        executable: BTreeSet::new(),
                    };
                } else if let Some(name) = group.strip_prefix("skill:") {
                    if let Some((mode_path, bytes)) =
                        portable_assets::completed_modes(root, id, name)?.filter(|(p, _)| p == path)
                    {
                        let _ = mode_path;
                        after[i] = Image {
                            exists: true,
                            directory: false,
                            files: BTreeMap::from([(String::new(), bytes)]),
                            executable: BTreeSet::new(),
                        };
                    } else if portable_assets::completed_skill_target(root, id, name)?.as_ref()
                        == Some(path)
                    {
                        after[i] = image(base, path)?;
                    }
                } else {
                    after[i] = image(base, path)?;
                }
            }
        }
        r.after = Some(after);
        persist(root, &mut r)?;
    }
    Ok(())
}
pub fn list(root: &Path, home: &Path) -> ModelResult<Vec<Status>> {
    let dir = root.join("backups/transfer-scope");
    safety::check_path(root, &dir, true)?;
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut out = vec![];
    for e in std::fs::read_dir(&dir)? {
        let e = e?;
        let p = e.path();
        safety::check_path(root, &p, false)?;
        if p.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let id = p
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| invalid("scope_backup", "备份记录文件名无效"))?;
        let plan = transfer::read_incoming(root, id)?;
        if plan.file_home.as_deref() != Some(home) {
            continue;
        }
        if let Some(s) = status(root, id)? {
            if s.enabled {
                out.push(s)
            }
        }
        if out.len() > 20000 {
            return Err(invalid("scope_backup", "备份记录超过上限"));
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}
pub fn restore_baseline(root: &Path, id: &str, groups: &[String]) -> ModelResult<String> {
    let r = read(root, id)?;
    if !r.enabled
        || r.after.is_none()
        || groups.is_empty()
        || groups.iter().any(|g| !r.groups.contains(g))
    {
        return Err(invalid("scope_restore", "请选择可恢复的原备份范围"));
    }
    let targets = targets(root, id)?;
    let current = targets
        .iter()
        .filter(|t| groups.contains(&t.0))
        .map(|(_, base, path)| image(base, path))
        .collect::<ModelResult<Vec<_>>>()?;
    Ok(super::store::sha256(&serde_json::to_vec(&(
        id, groups, current,
    ))?))
}
pub fn restore_checked(
    root: &Path,
    id: &str,
    groups: &[String],
    baseline: &str,
) -> ModelResult<()> {
    if restore_baseline(root, id, groups)? != baseline {
        return Err(invalid(
            "scope_restore_conflict",
            "恢复预览后内容改变，请重新核对，未覆盖",
        ));
    }
    restore(root, id, groups)
}
pub fn status(root: &Path, id: &str) -> ModelResult<Option<Status>> {
    if !path(root, id)?.exists() {
        return Ok(None);
    }
    let r = read(root, id)?;
    Ok(Some(Status {
        id: r.id,
        enabled: r.enabled,
        ready: r.after.is_some(),
        groups: r
            .groups
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
    }))
}
// Reverse only the fields changed by this operation. Independent later edits
// remain; overlapping edits stop the entire restore before the first write.
fn undo(
    before: Option<&Value>,
    after: Option<&Value>,
    current: Option<&Value>,
) -> ModelResult<Option<Value>> {
    if before == after {
        return Ok(current.cloned());
    }
    if current == after || current == before {
        return Ok(before.cloned());
    }
    if let (Some(Value::Object(a)), Some(Value::Object(c))) = (after, current) {
        if before.is_none() || before.is_some_and(Value::is_object) {
            let empty = serde_json::Map::new();
            let b = before.and_then(Value::as_object).unwrap_or(&empty);
            let keys = b
                .keys()
                .chain(a.keys())
                .chain(c.keys())
                .cloned()
                .collect::<BTreeSet<_>>();
            let mut out = serde_json::Map::new();
            for k in keys {
                if let Some(v) = undo(b.get(&k), a.get(&k), c.get(&k))? {
                    out.insert(k, v);
                }
            }
            return Ok(if before.is_none() && out.is_empty() {
                None
            } else {
                Some(Value::Object(out))
            });
        }
    }
    Err(invalid(
        "scope_restore_conflict",
        "本次更新涉及的内容后来又被修改，未覆盖；请先核对冲突",
    ))
}
fn merge_bytes(
    before: Option<&Vec<u8>>,
    after: Option<&Vec<u8>>,
    current: Option<&Vec<u8>>,
) -> ModelResult<Option<Vec<u8>>> {
    if before == after {
        return Ok(current.cloned());
    }
    if current == after || current == before {
        return Ok(before.cloned());
    }
    let parse = |v: Option<&Vec<u8>>| -> ModelResult<Option<Value>> {
        v.map(|v| {
            serde_json::from_slice(v)
                .map_err(|_| invalid("scope_restore_conflict", "资源在更新后又被修改，未覆盖"))
        })
        .transpose()
    };
    let b = parse(before)?;
    let a = parse(after)?;
    let c = parse(current)?;
    undo(b.as_ref(), a.as_ref(), c.as_ref())?
        .map(|v| serde_json::to_vec_pretty(&v).map_err(Into::into))
        .transpose()
}
pub(crate) fn write_image(
    base: &Path,
    target: &Path,
    current: &Image,
    next: &Image,
) -> ModelResult<()> {
    if image(base, target)? != *current {
        return Err(invalid(
            "scope_restore_conflict",
            "恢复期间文件变化，未覆盖",
        ));
    }
    for name in current
        .files
        .keys()
        .chain(next.files.keys())
        .collect::<BTreeSet<_>>()
    {
        let p = if name.is_empty() {
            target.to_path_buf()
        } else {
            target.join(name)
        };
        if !name.is_empty()
            && Path::new(name)
                .components()
                .any(|p| !matches!(p, std::path::Component::Normal(_)))
        {
            return Err(invalid("scope_restore", "资源路径超出所选对象，未恢复"));
        }
        safety::check_path(base, &p, true)?;
        match next.files.get(name) {
            Some(bytes) => {
                atomic_write(
                    &p,
                    bytes,
                    if next.executable.contains(name) {
                        0o700
                    } else {
                        0o600
                    },
                )?;
            }
            None => {
                if p.exists() {
                    std::fs::remove_file(p)?;
                }
            }
        }
    }
    if next.directory && next.exists {
        std::fs::create_dir_all(target)?;
    } else if current.directory && !next.exists {
        fn prune(p: &Path) -> std::io::Result<()> {
            for e in std::fs::read_dir(p)? {
                let e = e?;
                if e.file_type()?.is_dir() {
                    prune(&e.path())?
                }
            }
            std::fs::remove_dir(p)
        }
        if target.exists() {
            prune(target)?;
        }
    }
    if image(base, target)? != *next {
        return Err(invalid("scope_restore_readback", "恢复未通过读回"));
    }
    Ok(())
}
fn owned_intermediate(live: &Image, before: &Image, after: &Image) -> bool {
    live.files
        .keys()
        .chain(before.files.keys())
        .chain(after.files.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .all(|key| {
            live.files.get(key) == before.files.get(key)
                || live.files.get(key) == after.files.get(key)
        })
}
pub fn restore(root: &Path, id: &str, groups: &[String]) -> ModelResult<()> {
    let file = path(root, id)?;
    let _guard = TargetFileLock::lock(&file)?;
    let r = read(root, id)?;
    let after = r
        .after
        .ok_or_else(|| invalid("scope_restore", "原操作结果尚未核对，不能恢复"))?;
    let t = targets(root, id)?;
    if groups.is_empty() || groups.iter().any(|g| !r.groups.contains(g)) {
        return Err(invalid("scope_restore", "请选择原备份范围"));
    }
    if after.len() != t.len() {
        return Err(invalid("scope_restore", "备份结果范围无效"));
    }
    let mut writes = vec![];
    for (i, (group, base, target)) in t.iter().enumerate() {
        if !groups.contains(group) {
            continue;
        }
        let current = image(base, target)?;
        let b = &r.before[i];
        let a = &after[i];
        let mut next = current.clone();
        let keys = b
            .files
            .keys()
            .chain(a.files.keys())
            .chain(current.files.keys())
            .cloned()
            .collect::<BTreeSet<_>>();
        next.files.clear();
        for key in keys {
            let bm = b.executable.contains(&key);
            let am = a.executable.contains(&key);
            let cm = current.executable.contains(&key);
            if bm != am {
                if cm != am && cm != bm {
                    return Err(invalid(
                        "scope_restore_conflict",
                        "文件权限在更新后改变，未覆盖",
                    ));
                }
                if bm {
                    next.executable.insert(key.clone());
                } else {
                    next.executable.remove(&key);
                }
            }
            if let Some(bytes) = merge_bytes(
                b.files.get(&key),
                a.files.get(&key),
                current.files.get(&key),
            )? {
                next.files.insert(key, bytes);
            }
        }
        next.executable.retain(|key| next.files.contains_key(key));
        if current == *a {
            next.exists = b.exists;
            next.directory = b.directory
        } else {
            next.exists = !next.files.is_empty() || b.exists;
            next.directory = current.directory
        }
        writes.push((base, target, current, next));
    }
    for (index, (base, target, current, next)) in writes.iter().enumerate() {
        if let Err(e) = write_image(base, target, current, next) {
            let mut failed = false;
            for (b, p, c, n) in writes[..=index].iter().rev() {
                match image(b, p) {
                    Ok(live) if owned_intermediate(&live, c, n) => {
                        if write_image(b, p, &live, c).is_err() {
                            failed = true
                        }
                    }
                    _ => failed = true,
                }
            }
            return Err(if failed {
                invalid(
                    "scope_restore_partial",
                    "恢复未完成且部分补偿失败，原备份仍保留，请核对",
                )
            } else {
                e
            });
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inverse_preserves_outside_edits_and_rejects_overlap() {
        let b = serde_json::json!({"providers":{"a":{"models":["old"],"key":"local"}},"outside":1});
        let a = serde_json::json!({"providers":{"a":{"models":["new"],"key":"local"}},"outside":1});
        let mut c = a.clone();
        c["outside"] = Value::from(2);
        let restored = undo(Some(&b), Some(&a), Some(&c)).unwrap().unwrap();
        assert_eq!(restored["outside"], 2);
        assert_eq!(
            restored["providers"]["a"]["models"],
            serde_json::json!(["old"])
        );
        c["providers"]["a"]["models"] = serde_json::json!(["later"]);
        assert!(undo(Some(&b), Some(&a), Some(&c)).is_err());
    }
    #[test]
    fn snapshots_reject_symlinks_and_restore_exact_resource_bytes() {
        let t = tempfile::tempdir().unwrap();
        let p = t.path().join("skill");
        std::fs::create_dir_all(&p).unwrap();
        std::fs::write(p.join("SKILL.md"), b"original\n").unwrap();
        let b = image(t.path(), &p).unwrap();
        std::fs::write(p.join("SKILL.md"), b"incoming\n").unwrap();
        let a = image(t.path(), &p).unwrap();
        write_image(t.path(), &p, &a, &b).unwrap();
        assert_eq!(std::fs::read(p.join("SKILL.md")).unwrap(), b"original\n");
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("/tmp", p.join("escape")).unwrap();
            assert!(image(t.path(), &p).is_err());
        }
    }
}
