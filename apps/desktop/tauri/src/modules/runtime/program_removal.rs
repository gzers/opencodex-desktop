//! Recoverable exact-image removal core. Callers own UI confirmation, process
//! absence and terminal-registration revocation; unknown files are never erased.
use super::{
    bundled,
    program_image::{self, Entry},
    program_layout,
    program_selection::{self, CurrentSelection, IMAGE_MANIFEST},
    RuntimeSourceKind, RuntimeSourceRecord, RuntimeStore,
};
use crate::{
    errors::{AppError, AppResult},
    infrastructure::{atomic_write::atomic_write, hash, locking::TargetFileLock, platform},
    modules::{backup::safety, model_config::config_file::v2::native_import::backups},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub const PENDING: &str = ".removal-pending.json";
pub const REMOVED: &str = ".runtime-removed.json";
const MAX_JOURNAL: u64 = 64 * 1024 * 1024;
fn fail(message: &'static str) -> AppError {
    AppError::RuntimeManaged {
        code: "runtime_removal_unconfirmed".into(),
        detail: message.into(),
    }
}
fn io(_: std::io::Error) -> AppError {
    fail("移除程序文件失败，原操作及未删原件保留")
}
fn encode(v: &impl Serialize) -> AppResult<Vec<u8>> {
    serde_json::to_vec(v).map_err(|_| fail("卸载记录无法编码"))
}
fn decode<T: serde::de::DeserializeOwned>(b: &[u8]) -> AppResult<T> {
    let value = crate::modules::model_config::schema_v2::parse_strict_json_v2(b)
        .map_err(|_| fail("卸载记录格式无法核验"))?;
    serde_json::from_value(value).map_err(|_| fail("卸载记录结构无法核验"))
}
fn read(root: &Path, path: &Path, limit: u64) -> AppResult<Option<Vec<u8>>> {
    safety::check_path(root, path, true)?;
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(io(e)),
        Ok(_) => {
            let file = safety::open_file(path, false)?;
            safety::require_single_link(&file)?;
            let mut bytes = Vec::new();
            file.take(limit + 1).read_to_end(&mut bytes).map_err(io)?;
            if bytes.len() as u64 > limit {
                return Err(fail("卸载记录超过读取上限"));
            }
            Ok(Some(bytes))
        }
    }
}
fn create(path: &Path, bytes: &[u8]) -> AppResult<()> {
    let mut file = platform::create_new_private_file(path, 0o600).map_err(io)?;
    file.write_all(bytes).map_err(io)?;
    file.sync_all().map_err(io)
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Image {
    selection: CurrentSelection,
    entries: BTreeMap<String, Entry>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Plan {
    schema_version: u32,
    operation_id: String,
    program_root: PathBuf,
    desktop_root: PathBuf,
    home: PathBuf,
    images: Vec<Image>,
    current_before: Vec<u8>,
    entry_before: Vec<u8>,
    record_before: Option<Vec<u8>>,
    record_after: Vec<u8>,
    catalog_revision: Option<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Journal {
    sha256: String,
    plan: Plan,
}
/// Server-owned preparation; no file inventories or source records cross IPC.
#[derive(Clone)]
pub struct Prepared(Plan);
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub operation_id: String,
    pub versions: Vec<String>,
    pub program_root: PathBuf,
    pub preserved_home: PathBuf,
    pub confirmation_digest: String,
    pub bytes: u64,
    pub files: usize,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
    pub operation_id: String,
    pub removed_versions: Vec<String>,
    pub configuration_preserved: bool,
}
/// Read-only recovery preparation. Surviving content must still match the
/// original removal plan; it never replaces missing objects or writes a lock.
pub fn inspect_pending(i: &Path, d: &Path, home: &Path) -> AppResult<Option<Prepared>> {
    bundled::roots(i, d)?;
    bundled::roots(i, home)?;
    let Some(bytes) = read(i, &pending(i), MAX_JOURNAL)? else {
        return Ok(None);
    };
    let journal: Journal = decode(&bytes)?;
    let p = &journal.plan;
    if p.program_root != i
        || p.desktop_root != d
        || p.home != home
        || journal.sha256 != hash::sha256_hex(&encode(p)?)
    {
        return Err(fail("原卸载操作与当前程序或配置目录不一致"));
    }
    validate(p)?;
    check_version_names(p)?;
    for image in &p.images {
        remaining(p, image)?;
    }
    for (path, before) in [
        (current(i), &p.current_before),
        (entry(i)?, &p.entry_before),
    ] {
        if read(i, &path, 64 * 1024)?
            .as_ref()
            .is_some_and(|v| v != before)
        {
            return Err(fail("卸载目标包含外部修改，未继续"));
        }
    }
    let record = read(d, RuntimeStore::new(d).path(), 1024 * 1024)?;
    if record != p.record_before && record.as_deref() != Some(p.record_after.as_slice()) {
        return Err(fail("原卸载来源记录发生变化"));
    }
    if read(i, &pending(i), MAX_JOURNAL)?.as_deref() != Some(bytes.as_slice()) {
        return Err(fail("读取期间原卸载操作变化"));
    }
    Ok(Some(Prepared(journal.plan)))
}
fn entry(i: &Path) -> AppResult<PathBuf> {
    Ok(i.join(program_layout::runtime_entry_relative(
        program_layout::host_target()?.1,
    )))
}
fn current(i: &Path) -> PathBuf {
    i.join("runtime").join(program_layout::CURRENT_FILENAME)
}
fn pending(i: &Path) -> PathBuf {
    i.join("runtime").join(PENDING)
}
fn removed(i: &Path) -> PathBuf {
    i.join("runtime").join(REMOVED)
}
/// Prevent an ordinary startup from silently reinstalling a runtime which the
/// user explicitly removed. An inconsistent marker is uncertainty, not absence.
pub fn intentionally_removed(i: &Path, d: &Path) -> AppResult<bool> {
    bundled::roots(i, d)?;
    if matches!(fs::symlink_metadata(i),Err(e) if e.kind()==std::io::ErrorKind::NotFound) {
        return Ok(false);
    }
    let Some(bytes) = read(i, &removed(i), MAX_JOURNAL)? else {
        return Ok(false);
    };
    let journal: Journal = decode(&bytes)?;
    let p = &journal.plan;
    if p.program_root != i || p.desktop_root != d || journal.sha256 != hash::sha256_hex(&encode(p)?)
    {
        return Err(fail("卸载完成记录不属于当前程序和用户"));
    }
    validate(p)?;
    if read(i, &pending(i), MAX_JOURNAL)?.is_some()
        || read(i, &current(i), 64 * 1024)?.is_some()
        || read(i, &entry(i)?, 64 * 1024)?.is_some()
        || read(d, RuntimeStore::new(d).path(), 1024 * 1024)?.as_deref()
            != Some(p.record_after.as_slice())
        || fs::read_dir(i.join("runtime/versions"))
            .map_err(io)?
            .next()
            .is_some()
    {
        return Err(fail("卸载完成状态与实际程序目录不一致"));
    }
    Ok(true)
}
fn image_root(i: &Path, selection: &CurrentSelection) -> AppResult<PathBuf> {
    Ok(program_layout::RuntimeVersionPaths::new(
        &i.join("runtime/versions"),
        &selection.version,
        program_layout::host_target()?.1,
    )?
    .version_root)
}
fn check_version_names(plan: &Plan) -> AppResult<()> {
    let versions = plan.program_root.join("runtime/versions");
    safety::check_path(&plan.program_root, &versions, false)?;
    let allowed = plan
        .images
        .iter()
        .map(|i| i.selection.version.as_str())
        .collect::<BTreeSet<_>>();
    let mut count = 0;
    for item in fs::read_dir(versions).map_err(io)? {
        count += 1;
        if count > 1000 {
            return Err(fail("版本目录数量超过卸载上限"));
        }
        let item = item.map_err(io)?;
        let name = item.file_name();
        let name = name.to_str().ok_or_else(|| fail("版本目录名称无效"))?;
        if !allowed.contains(name) || !safety::inspect(&item.path())?.is_dir() {
            return Err(fail("存在未登记的版本目录或其它对象，未卸载"));
        }
    }
    Ok(())
}
/// Explicit admitted selections must be supplied by the installation/activation
/// owner. This never scans a manifest to grant itself ownership of an image.
pub fn prepare(
    i: &Path,
    d: &Path,
    home: &Path,
    selections: &[CurrentSelection],
) -> AppResult<Prepared> {
    bundled::roots(i, d)?;
    bundled::roots(i, home)?;
    if selections.is_empty() || selections.len() > 1000 {
        return Err(fail("卸载范围需要明确的已登记版本集合"));
    }
    if read(i, &pending(i), MAX_JOURNAL)?.is_some() || read(i, &removed(i), MAX_JOURNAL)?.is_some()
    {
        return Err(fail("已有卸载操作，请查看原操作"));
    }
    let (target, platform) = program_layout::host_target()?;
    let selected = program_selection::read(i, platform, target)?
        .ok_or_else(|| fail("当前运行程序无法核验"))?;
    let mut images = vec![];
    let mut names = BTreeSet::new();
    for s in selections {
        if !names.insert(s.version.clone()) {
            return Err(fail("卸载范围含重复版本"));
        }
        let image = program_selection::read_image(i, platform, target, s)?;
        program_image::verify(&image)?;
        let entries = program_image::inventory(&image.paths.version_root)?;
        if program_image::inventory_digest(&entries)? != image.image.tree_sha256 {
            return Err(fail("冻结卸载范围时文件树变化"));
        }
        images.push(Image {
            selection: s.clone(),
            entries,
        });
    }
    if !selections.contains(&selected.selection) {
        return Err(fail("卸载范围遗漏当前运行版本"));
    }
    let record_before = read(d, RuntimeStore::new(d).path(), 1024 * 1024)?;
    let mut record: RuntimeSourceRecord = record_before
        .as_deref()
        .map(decode)
        .transpose()?
        .unwrap_or_default();
    if record_before.is_some()
        && (record.schema_version != super::RUNTIME_SCHEMA_VERSION
            || record.source != RuntimeSourceKind::Managed
            || record.path.as_deref() != selected.stable_entry.to_str()
            || record.resolved_version.as_deref() != Some(selected.selection.version.as_str()))
    {
        return Err(fail("当前来源记录不属于卸载范围"));
    }
    record.source = RuntimeSourceKind::Unresolved;
    record.path = None;
    record.resolved_version = None;
    record.resolved_at = Some(super::now_rfc3339());
    record.push_history(super::InstallHistoryEntry {
        action: "uninstall".into(),
        result: "succeeded".into(),
        package: super::OFFICIAL_PACKAGE.into(),
        version: selected.selection.version.clone(),
        target: i.to_string_lossy().into_owned(),
        at: super::now_rfc3339(),
        reason: None,
    });
    let mut doc = record_before
        .as_deref()
        .map(decode::<serde_json::Value>)
        .transpose()?
        .unwrap_or(
            serde_json::to_value(RuntimeSourceRecord::default())
                .map_err(|_| fail("来源记录无法编码"))?,
        );
    let next = serde_json::to_value(&record).map_err(|_| fail("来源记录无法编码"))?;
    for key in [
        "source",
        "path",
        "resolved_version",
        "resolved_at",
        "history",
    ] {
        doc[key] = next[key].clone();
    }
    let plan = Plan {
        schema_version: 1,
        operation_id: uuid::Uuid::new_v4().to_string(),
        program_root: i.into(),
        desktop_root: d.into(),
        home: home.into(),
        images,
        current_before: read(i, &current(i), 64 * 1024)?.ok_or_else(|| fail("当前选择记录缺失"))?,
        entry_before: read(i, &entry(i)?, 64 * 1024)?.ok_or_else(|| fail("当前入口缺失"))?,
        record_before,
        record_after: encode(&doc)?,
        catalog_revision: None,
    };
    check_version_names(&plan)?;
    validate(&plan)?;
    Ok(Prepared(plan))
}
/// Production preparation consumes the source-admission ledger, never image
/// names found by scanning the versions directory.
pub fn prepare_registered(i: &Path, d: &Path, home: &Path) -> AppResult<Prepared> {
    let catalog = super::program_catalog::inspect(i)?;
    let mut prepared = prepare(i, d, home, &catalog.selections)?;
    if super::program_catalog::revision(i)? != catalog.revision {
        return Err(fail("冻结卸载期间版本登记发生变化"));
    }
    prepared.0.catalog_revision = Some(catalog.revision);
    Ok(prepared)
}
fn validate(p: &Plan) -> AppResult<()> {
    bundled::roots(&p.program_root, &p.desktop_root)?;
    bundled::roots(&p.program_root, &p.home)?;
    if let Some(expected) = &p.catalog_revision {
        if &super::program_catalog::revision(&p.program_root)? != expected {
            return Err(fail("原卸载版本登记已变化"));
        }
    }
    if p.schema_version != 1
        || uuid::Uuid::parse_str(&p.operation_id).is_err()
        || p.images.is_empty()
        || p.images.len() > 1000
    {
        return Err(fail("卸载操作身份无效"));
    }
    let before: CurrentSelection = decode(&p.current_before)?;
    if !p.images.iter().any(|i| i.selection == before) {
        return Err(fail("卸载操作不含原当前版本"));
    }
    let (_, platform) = program_layout::host_target()?;
    let mut versions = BTreeSet::new();
    for image in &p.images {
        if image.selection.schema_version != 1
            || image.selection.platform != program_layout::host_target()?.0
            || !versions.insert(&image.selection.version)
            || image.entries.len() > 100_000
        {
            return Err(fail("卸载版本记录无效"));
        }
        let root = image_root(&p.program_root, &image.selection)?;
        for name in image.entries.keys() {
            let path = root.join(name);
            if name == IMAGE_MANIFEST
                || !program_layout::clean_absolute(&path)
                || !path.starts_with(&root)
            {
                return Err(fail("卸载文件清单含非法路径"));
            }
        }
        if before == image.selection {
            // Entry spelling comes from the bound manifest while available;
            // after removal, the original text stays protected by the journal.
            if read(&p.program_root, &root.join(IMAGE_MANIFEST), 64 * 1024)?.is_some() {
                let manifest = manifest(&root, &image.selection)?;
                let paths = program_layout::RuntimeVersionPaths::new(
                    &p.program_root.join("runtime/versions"),
                    &before.version,
                    platform,
                )?;
                if p.entry_before
                    != super::install::entry_script(
                        Some(&paths.node),
                        &paths
                            .package_root
                            .join(super::install::PACKAGE_SUBPATH)
                            .join(&manifest.bin),
                    )
                    .as_bytes()
                {
                    return Err(fail("卸载原入口不属于原版本"));
                }
            }
        }
    }
    let after: RuntimeSourceRecord = decode(&p.record_after)?;
    if after.schema_version != super::RUNTIME_SCHEMA_VERSION
        || after.source != RuntimeSourceKind::Unresolved
        || after.path.is_some()
        || after.resolved_version.is_some()
    {
        return Err(fail("卸载来源后像无效"));
    }
    Ok(())
}
fn manifest(root: &Path, s: &CurrentSelection) -> AppResult<program_selection::ImageManifest> {
    let bytes = read(root, &root.join(IMAGE_MANIFEST), 64 * 1024)?
        .ok_or_else(|| fail("卸载镜像清单缺失"))?;
    if hash::sha256_hex(&bytes) != s.image_sha256 {
        return Err(fail("卸载镜像清单已被修改"));
    }
    let m: program_selection::ImageManifest = decode(&bytes)?;
    if m.version != s.version || m.platform != s.platform || m.package != super::OFFICIAL_PACKAGE {
        return Err(fail("卸载镜像身份不符"));
    }
    Ok(m)
}
fn remaining(p: &Plan, image: &Image) -> AppResult<BTreeMap<String, Entry>> {
    let root = image_root(&p.program_root, &image.selection)?;
    safety::check_path(&p.program_root, &root, true)?;
    match fs::symlink_metadata(&root) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(e) => return Err(io(e)),
        Ok(_) => (),
    }
    let actual = program_image::inventory(&root)?;
    let manifest_bytes = read(&root, &root.join(IMAGE_MANIFEST), 64 * 1024)?;
    if manifest_bytes.is_some() {
        let m = manifest(&root, &image.selection)?;
        if program_image::inventory_digest(&image.entries)? != m.tree_sha256 {
            return Err(fail("卸载文件冻结摘要不符"));
        }
    } else if !actual.is_empty() {
        return Err(fail("卸载目录清单缺失但仍有文件，未继续删除"));
    }
    for (name, entry) in &actual {
        if image.entries.get(name) != Some(entry) {
            return Err(fail("卸载目录有外部新增或修改，未继续删除"));
        }
    }
    Ok(actual)
}
impl Prepared {
    #[cfg(test)]
    pub(crate) fn interrupt_fixture(
        &self,
        phase: &str,
        revoke: impl FnOnce() -> AppResult<()>,
    ) -> AppResult<Outcome> {
        self.execute_inner(
            &self.preview()?.confirmation_digest,
            || Ok(()),
            revoke,
            |step| {
                if step == phase {
                    Err(fail("explicit isolated interruption fixture"))
                } else {
                    Ok(())
                }
            },
        )
    }
    pub fn preview(&self) -> AppResult<Preview> {
        let p = &self.0;
        let mut bytes = 0u64;
        let mut files = 0usize;
        for image in &p.images {
            for entry in image.entries.values() {
                if let Entry::File { bytes: n, .. } = entry {
                    bytes = bytes
                        .checked_add(*n)
                        .ok_or_else(|| fail("卸载范围大小溢出"))?;
                    files += 1;
                }
            }
        }
        Ok(Preview {
            operation_id: p.operation_id.clone(),
            versions: p
                .images
                .iter()
                .map(|i| i.selection.version.clone())
                .collect(),
            program_root: p.program_root.clone(),
            preserved_home: p.home.clone(),
            confirmation_digest: hash::sha256_hex(&encode(p)?),
            bytes,
            files,
        })
    }
    /// Hooks must prove ALL named program images unused, and revoke only the
    /// current user's owned terminal registration before any metadata/file removal.
    pub fn execute(
        &self,
        expected: &str,
        unused: impl Fn() -> AppResult<()>,
        revoke_terminal: impl FnOnce() -> AppResult<()>,
    ) -> AppResult<Outcome> {
        self.execute_inner(expected, unused, revoke_terminal, |_| Ok(()))
    }
    fn execute_inner(
        &self,
        expected: &str,
        unused: impl Fn() -> AppResult<()>,
        revoke_terminal: impl FnOnce() -> AppResult<()>,
        checkpoint: impl Fn(&str) -> AppResult<()>,
    ) -> AppResult<Outcome> {
        let p = &self.0;
        if self.preview()?.confirmation_digest != expected {
            return Err(fail("卸载预览已过期"));
        }
        let _program = super::program_activation::Mutation::acquire(&p.program_root)?;
        let record = RuntimeStore::new(&p.desktop_root).path().to_path_buf();
        let _record = TargetFileLock::lock(&record)?;
        validate(p)?;
        check_version_names(p)?;
        unused()?;
        if read(&p.program_root, &pending(&p.program_root), MAX_JOURNAL)?.is_some()
            || read(&p.program_root, &removed(&p.program_root), MAX_JOURNAL)?.is_some()
        {
            return Err(fail("已有卸载操作，请重新核对"));
        }
        if read(&p.program_root, &current(&p.program_root), 64 * 1024)?.as_deref()
            != Some(p.current_before.as_slice())
            || read(&p.program_root, &entry(&p.program_root)?, 64 * 1024)?.as_deref()
                != Some(p.entry_before.as_slice())
            || read(&p.desktop_root, &record, 1024 * 1024)? != p.record_before
        {
            return Err(fail("卸载前程序来源变化"));
        }
        for image in &p.images {
            if remaining(p, image)? != image.entries {
                return Err(fail("卸载范围已变化"));
            }
        }
        let protected = backups::protect_upgrade(
            &p.desktop_root,
            &backups::UpgradeRequest {
                home: p.home.clone(),
                program_source: None,
            },
        )
        .map_err(|_| fail("卸载前配置保护未完成"))?;
        revoke_terminal()?;
        unused()?;
        protected.verify().map_err(|_| fail("卸载前配置已变化"))?;
        let bytes = encode(&Journal {
            sha256: hash::sha256_hex(&encode(p)?),
            plan: p.clone(),
        })?;
        if bytes.len() as u64 > MAX_JOURNAL {
            return Err(fail("卸载记录超过上限"));
        }
        create(&pending(&p.program_root), &bytes)?;
        safety::sync_directories(&p.program_root, &p.program_root.join("runtime"))?;
        checkpoint("prepared")?;
        finish(p, &bytes, &unused, &checkpoint, &|| {
            protected.verify().map_err(|_| fail("卸载期间配置保护变化"))
        })
    }
}
fn finish(
    p: &Plan,
    journal: &[u8],
    unused: &impl Fn() -> AppResult<()>,
    checkpoint: &impl Fn(&str) -> AppResult<()>,
    config: &impl Fn() -> AppResult<()>,
) -> AppResult<Outcome> {
    validate(p)?;
    check_version_names(p)?;
    unused()?;
    config()?;
    // Audit all images and metadata before doing even one continuation write.
    for image in &p.images {
        remaining(p, image)?;
    }
    let record = RuntimeStore::new(&p.desktop_root).path().to_path_buf();
    let source = read(&p.desktop_root, &record, 1024 * 1024)?;
    if source != p.record_before && source.as_deref() != Some(p.record_after.as_slice()) {
        return Err(fail("卸载来源包含外部修改"));
    }
    for (path, old) in [
        (entry(&p.program_root)?, &p.entry_before),
        (current(&p.program_root), &p.current_before),
    ] {
        let value = read(&p.program_root, &path, 64 * 1024)?;
        if value.as_ref().is_some_and(|v| v != old) {
            return Err(fail("卸载入口或选择包含外部修改"));
        }
    }
    let check = || -> AppResult<()> {
        unused()?;
        config()?;
        if let Some(expected) = &p.catalog_revision {
            if &super::program_catalog::revision(&p.program_root)? != expected {
                return Err(fail("卸载期间版本登记发生变化"));
            }
        }
        if read(&p.program_root, &pending(&p.program_root), MAX_JOURNAL)?.as_deref()
            != Some(journal)
        {
            return Err(fail("卸载事务身份在执行期间变化"));
        }
        Ok(())
    };
    for (path, old) in [
        (entry(&p.program_root)?, &p.entry_before),
        (current(&p.program_root), &p.current_before),
    ] {
        check()?;
        if let Some(value) = read(&p.program_root, &path, 64 * 1024)? {
            if &value != old {
                return Err(fail("卸载入口变化"));
            }
            fs::remove_file(&path).map_err(io)?;
            safety::sync_directories(&p.program_root, path.parent().unwrap())?;
        }
        checkpoint("metadata")?;
    }
    for image in &p.images {
        let root = image_root(&p.program_root, &image.selection)?;
        let actual = remaining(p, image)?;
        let mut deleted = 0usize;
        for (name, kind) in &actual {
            if !matches!(kind, Entry::File { .. }) {
                continue;
            }
            if deleted.is_multiple_of(128) {
                check()?;
            }
            let path = root.join(name);
            safety::check_path(&p.program_root, &path, false)?;
            if &program_image::file_entry(&path)? != kind {
                return Err(fail("待删除文件在核对后变化"));
            }
            fs::remove_file(root.join(name)).map_err(io)?;
            deleted += 1;
            checkpoint("file")?;
        }
        let mut dirs = actual
            .iter()
            .filter(|(_, e)| matches!(e, Entry::Directory))
            .map(|(n, _)| n)
            .collect::<Vec<_>>();
        dirs.sort_by_key(|name| std::cmp::Reverse(name.matches('/').count()));
        check()?;
        for name in dirs {
            safety::check_path(&p.program_root, &root.join(name), false)?;
            fs::remove_dir(root.join(name)).map_err(io)?;
            checkpoint("directory")?;
        }
        if root.exists() {
            check()?;
            if !remaining(p, image)?.is_empty() {
                return Err(fail("卸载后仍有版本文件"));
            }
            let path = root.join(IMAGE_MANIFEST);
            if read(&root, &path, 64 * 1024)?.is_some() {
                manifest(&root, &image.selection)?;
                fs::remove_file(path).map_err(io)?;
            }
            fs::remove_dir(&root).map_err(io)?;
            safety::sync_directories(&p.program_root, root.parent().unwrap())?;
            checkpoint("image")?;
        }
    }
    check()?;
    check_version_names(p)?;
    if fs::read_dir(p.program_root.join("runtime/versions"))
        .map_err(io)?
        .next()
        .is_some()
    {
        return Err(fail("仍有版本目录，未确认卸载完成"));
    }
    if read(&p.program_root, &current(&p.program_root), 64 * 1024)?.is_some()
        || read(&p.program_root, &entry(&p.program_root)?, 64 * 1024)?.is_some()
    {
        return Err(fail("卸载入口或当前选择在删除后重新出现"));
    }
    let record_now = read(&p.desktop_root, &record, 1024 * 1024)?;
    if record_now == p.record_before {
        atomic_write(&record, &p.record_after, 0o600)?;
    } else if record_now.as_deref() != Some(p.record_after.as_slice()) {
        return Err(fail("提交卸载来源前外部记录已变化"));
    }
    safety::sync_directories(&p.desktop_root, record.parent().unwrap())?;
    checkpoint("record")?;
    if read(&p.desktop_root, &record, 1024 * 1024)?.as_deref() != Some(p.record_after.as_slice()) {
        return Err(fail("卸载来源记录读回不符"));
    }
    check()?;
    let completed = removed(&p.program_root);
    if let Some(bytes) = read(&p.program_root, &completed, MAX_JOURNAL)? {
        if bytes != journal {
            return Err(fail("已有卸载完成记录不符"));
        }
    } else {
        create(&completed, journal)?;
    }
    safety::sync_directories(&p.program_root, &p.program_root.join("runtime"))?;
    checkpoint("marker")?;
    fs::remove_file(pending(&p.program_root)).map_err(io)?;
    safety::sync_directories(&p.program_root, &p.program_root.join("runtime"))?;
    Ok(Outcome {
        operation_id: p.operation_id.clone(),
        removed_versions: p
            .images
            .iter()
            .map(|i| i.selection.version.clone())
            .collect(),
        configuration_preserved: true,
    })
}
/// Explicit continuation only; inventory comparison permits only already-absent
/// known objects, never a new baseline for changed surviving files.
pub fn resume(
    i: &Path,
    d: &Path,
    home: &Path,
    expected: &str,
    unused: impl Fn() -> AppResult<()>,
    revoke_terminal: impl FnOnce() -> AppResult<()>,
) -> AppResult<Outcome> {
    bundled::roots(i, d)?;
    let _program = super::program_activation::Mutation::acquire(i)?;
    let bytes = read(i, &pending(i), MAX_JOURNAL)?.ok_or_else(|| fail("没有待继续的卸载操作"))?;
    let journal: Journal = decode(&bytes)?;
    let p = &journal.plan;
    if p.program_root != i
        || p.desktop_root != d
        || p.home != home
        || journal.sha256 != hash::sha256_hex(&encode(p)?)
        || hash::sha256_hex(&encode(p)?) != expected
    {
        return Err(fail("卸载继续确认不属于当前操作"));
    }
    validate(p)?;
    let _record = TargetFileLock::lock(RuntimeStore::new(d).path())?;
    unused()?;
    let protected = backups::protect_upgrade(
        d,
        &backups::UpgradeRequest {
            home: home.into(),
            program_source: None,
        },
    )
    .map_err(|_| fail("卸载继续前配置保护失败"))?;
    revoke_terminal()?;
    finish(p, &bytes, &unused, &|_| Ok(()), &|| {
        protected.verify().map_err(|_| fail("配置保护已变化"))
    })
}

#[cfg(all(test, unix))]
#[path = "program_removal_tests.rs"]
mod tests;
