//! Same-image repair from original installer admission. Damaged bytes are never
//! re-admitted or deleted: the original directory is moved into a bound receipt.
use super::{
    bundled, install,
    program_activation::Mutation,
    program_catalog,
    program_image::{self, Entry},
    program_layout,
    program_selection::{self, CurrentSelection, IMAGE_MANIFEST},
    RuntimeSourceKind, RuntimeSourceRecord, RuntimeStore,
};
use crate::{
    errors::{AppError, AppResult},
    infrastructure::{atomic_write::atomic_write, hash, platform},
    modules::{backup::safety, model_config::config_file::v2::native_import::backups},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub(crate) const PENDING: &str = ".repair-pending.json";
const LIMIT: u64 = 64 * 1024 * 1024;
fn fail(text: &'static str) -> AppError {
    AppError::RuntimeManaged {
        code: "runtime_repair_unconfirmed".into(),
        detail: text.into(),
    }
}
fn io(_: std::io::Error) -> AppError {
    fail("同版本修复文件操作失败，原目录与恢复记录保留")
}
fn encode(v: &impl Serialize) -> AppResult<Vec<u8>> {
    serde_json::to_vec(v).map_err(|_| fail("修复记录无法编码"))
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> AppResult<T> {
    let v = crate::modules::model_config::schema_v2::parse_strict_json_v2(bytes)
        .map_err(|_| fail("修复记录不是唯一合法JSON"))?;
    serde_json::from_value(v).map_err(|_| fail("修复记录结构无法核验"))
}
fn read(root: &Path, path: &Path, limit: u64) -> AppResult<Option<Vec<u8>>> {
    safety::check_path(root, path, true)?;
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(io(e)),
        Ok(_) => {
            let f = safety::open_file(path, false)?;
            safety::require_single_link(&f)?;
            let mut b = Vec::new();
            f.take(limit + 1).read_to_end(&mut b).map_err(io)?;
            if b.len() as u64 > limit {
                return Err(fail("修复文件超过读取上限"));
            }
            Ok(Some(b))
        }
    }
}
fn create(path: &Path, bytes: &[u8]) -> AppResult<()> {
    let mut f = platform::create_new_private_file(path, 0o600).map_err(io)?;
    f.write_all(bytes).map_err(io)?;
    f.sync_all().map_err(io)
}
fn present(root: &Path, path: &Path) -> AppResult<bool> {
    safety::check_path(root, path, true)?;
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(io(e)),
    }
}
pub(crate) fn require_idle(i: &Path) -> AppResult<()> {
    if !program_layout::clean_absolute(i) {
        return Err(fail("修复程序根必须是明确绝对路径"));
    }
    match fs::symlink_metadata(i) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(io(e)),
        Ok(_) => {}
    }
    if present(i, &i.join("runtime").join(PENDING))? {
        return Err(fail(
            "原同版本修复尚未完成，请核对并显式继续；未启动其它来源",
        ));
    }
    Ok(())
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Snapshot {
    present: bool,
    entries: BTreeMap<String, Entry>,
}
fn snapshot(i: &Path, path: &Path) -> AppResult<Snapshot> {
    if !present(i, path)? {
        return Ok(Snapshot {
            present: false,
            entries: BTreeMap::new(),
        });
    }
    let mut entries = program_image::inventory(path)?;
    let manifest = path.join(IMAGE_MANIFEST);
    if present(i, &manifest)? {
        entries.insert(IMAGE_MANIFEST.into(), program_image::file_entry(&manifest)?);
    }
    Ok(Snapshot {
        present: true,
        entries,
    })
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Plan {
    program_root: PathBuf,
    desktop_root: PathBuf,
    home: PathBuf,
    selection: CurrentSelection,
    source_sha256: String,
    descriptor_pin: String,
    catalog_revision: String,
    current: Vec<u8>,
    record: Vec<u8>,
    entry_before: Option<Vec<u8>>,
    image_before: Snapshot,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Journal {
    schema_version: u32,
    operation_id: String,
    plan: Plan,
    entry_after: Vec<u8>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Envelope {
    sha256: String,
    journal: Journal,
}
fn entry(i: &Path) -> AppResult<PathBuf> {
    Ok(i.join(program_layout::runtime_entry_relative(
        program_layout::host_target()?.1,
    )))
}
impl Plan {
    fn image(&self) -> AppResult<PathBuf> {
        Ok(program_layout::RuntimeVersionPaths::new(
            &self.program_root.join("runtime/versions"),
            &self.selection.version,
            program_layout::host_target()?.1,
        )?
        .version_root)
    }
    fn digest(&self) -> AppResult<String> {
        Ok(hash::sha256_hex(&encode(self)?))
    }
    fn validate(&self, i: &Path, d: &Path, h: &Path) -> AppResult<()> {
        bundled::roots(i, d)?;
        bundled::roots(i, h)?;
        if self.program_root != i || self.desktop_root != d || self.home != h {
            return Err(fail("原修复操作不属于当前程序与用户配置目录"));
        }
        let (selected, source, revision, current) = program_catalog::repair_admission(i)?;
        if selected != self.selection
            || source != self.source_sha256
            || revision != self.catalog_revision
            || current != self.current
            || read(d, RuntimeStore::new(d).path(), 1024 * 1024)?.as_deref()
                != Some(self.record.as_slice())
        {
            return Err(fail("原修复选择、来源或版本登记发生变化"));
        }
        for name in [
            super::program_removal::PENDING,
            super::program_removal::REMOVED,
            super::program_reinstall::PENDING,
            program_selection::ACTIVATION_PENDING,
        ] {
            if present(i, &i.join("runtime").join(name))? {
                return Err(fail("存在其它未完成程序操作，未执行修复"));
            }
        }
        Ok(())
    }
}
impl Journal {
    fn history(&self) -> PathBuf {
        self.plan
            .program_root
            .join("runtime/repair-history")
            .join(&self.operation_id)
    }
    fn pending(&self) -> PathBuf {
        self.plan.program_root.join("runtime").join(PENDING)
    }
    fn validate(&self, i: &Path, d: &Path, h: &Path) -> AppResult<()> {
        if self.schema_version != 1
            || uuid::Uuid::parse_str(&self.operation_id).is_err()
            || self.entry_after.len() > 65536
        {
            return Err(fail("原修复操作身份无效"));
        }
        self.plan.validate(i, d, h)
    }
}
fn load(i: &Path, d: &Path, h: &Path) -> AppResult<Option<(Journal, Vec<u8>)>> {
    let Some(bytes) = read(i, &i.join("runtime").join(PENDING), LIMIT)? else {
        return Ok(None);
    };
    let e: Envelope = decode(&bytes)?;
    e.journal.validate(i, d, h)?;
    if e.sha256 != hash::sha256_hex(&encode(&e.journal)?) {
        return Err(fail("原修复记录摘要不符"));
    }
    Ok(Some((e.journal, bytes)))
}
fn repaired(j: &Journal) -> AppResult<()> {
    let p = &j.plan;
    let candidate = program_selection::read_image(
        &p.program_root,
        program_layout::host_target()?.1,
        &p.selection.platform,
        &p.selection,
    )?;
    if candidate.image.official_archive_sha256 != p.source_sha256
        || install::entry_script(Some(&candidate.paths.node), &candidate.script).as_bytes()
            != j.entry_after
    {
        return Err(fail("修复后的镜像或入口不是原候选"));
    }
    program_image::verify(&candidate)
}
fn states(j: &Journal) -> AppResult<()> {
    let p = &j.plan;
    i_check(j)?;
    let observed = read(&p.program_root, &entry(&p.program_root)?, 65536)?;
    if observed != p.entry_before && observed.as_deref() != Some(j.entry_after.as_slice()) {
        return Err(fail("修复期间入口包含新的外部修改"));
    }
    let live = p.image()?;
    let archived = j.history().join("image");
    if present(&p.program_root, &archived)? {
        if !p.image_before.present || snapshot(&p.program_root, &archived)? != p.image_before {
            return Err(fail("原损坏镜像归档发生变化"));
        }
        if present(&p.program_root, &live)? {
            repaired(j)?;
        }
    } else if p.image_before.present {
        if snapshot(&p.program_root, &live)? != p.image_before {
            return Err(fail("原损坏镜像在修复确认后变化"));
        }
    } else if present(&p.program_root, &live)? {
        repaired(j)?;
    }
    Ok(())
}
fn i_check(j: &Journal) -> AppResult<()> {
    j.validate(&j.plan.program_root, &j.plan.desktop_root, &j.plan.home)
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    pub version: String,
    pub resume: bool,
    pub program_root: PathBuf,
    pub config_path: PathBuf,
    pub confirmation_digest: String,
}
fn prepare(
    i: &Path,
    d: &Path,
    h: &Path,
    pin: &str,
    version: &str,
    source: &str,
) -> AppResult<(Plan, Option<Journal>)> {
    if let Some((j, _)) = load(i, d, h)? {
        if j.plan.descriptor_pin != pin
            || j.plan.selection.version != version
            || j.plan.source_sha256 != source
        {
            return Err(fail("固定资源不对应原同版本修复候选"));
        }
        states(&j)?;
        return Ok((j.plan.clone(), Some(j)));
    }
    let (selection, origin, revision, current) = program_catalog::repair_admission(i)?;
    if selection.version != version || origin != source {
        return Err(fail("内置资源与原安装版本或官方来源不同，未修复"));
    }
    let record = read(d, RuntimeStore::new(d).path(), 1024 * 1024)?
        .ok_or_else(|| fail("原安装来源记录缺失"))?;
    let r: RuntimeSourceRecord = decode(&record)?;
    if r.schema_version != super::RUNTIME_SCHEMA_VERSION
        || r.source != RuntimeSourceKind::Managed
        || r.path.as_deref() != entry(i)?.to_str()
        || r.resolved_version.as_deref() != Some(version)
    {
        return Err(fail("原安装来源不能证明同版本归属"));
    }
    let image = program_layout::RuntimeVersionPaths::new(
        &i.join("runtime/versions"),
        version,
        program_layout::host_target()?.1,
    )?
    .version_root;
    let p = Plan {
        program_root: i.into(),
        desktop_root: d.into(),
        home: h.into(),
        selection,
        source_sha256: source.into(),
        descriptor_pin: pin.into(),
        catalog_revision: revision,
        current,
        record,
        entry_before: read(i, &entry(i)?, 65536)?,
        image_before: snapshot(i, &image)?,
    };
    p.validate(i, d, h)?;
    if let Ok(Some(selected)) =
        program_selection::read(i, program_layout::host_target()?.1, &p.selection.platform)
    {
        if program_image::verify(&selected).is_ok() {
            return Err(fail("当前运行程序完整，无需同版本修复"));
        }
    }
    Ok((p, None))
}
pub(crate) fn review(
    i: &Path,
    d: &Path,
    h: &Path,
    pin: &str,
    version: &str,
    source: &str,
) -> AppResult<Review> {
    let (p, j) = prepare(i, d, h, pin, version, source)?;
    Ok(Review {
        version: version.into(),
        resume: j.is_some(),
        program_root: i.into(),
        config_path: h.join("config.json"),
        confirmation_digest: p.digest()?,
    })
}
pub(super) struct Candidate<'a> {
    pub program: &'a Path,
    pub desktop: &'a Path,
    pub home: &'a Path,
    pub image: &'a Path,
    pub selection: &'a CurrentSelection,
    pub source: &'a str,
    pub pin: &'a str,
    pub confirmation: &'a str,
    pub entry_after: &'a [u8],
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
    pub operation_id: String,
    pub version: String,
    pub configuration_preserved: bool,
}
pub(super) fn execute(
    c: Candidate<'_>,
    mutation: &Mutation,
    unused: impl Fn() -> AppResult<()>,
    checkpoint: impl Fn(&str) -> AppResult<()>,
) -> AppResult<Outcome> {
    let i = c.program;
    let d = c.desktop;
    let h = c.home;
    mutation.require_root(i)?;
    let (p, pending) = prepare(i, d, h, c.pin, &c.selection.version, c.source)?;
    if p.selection != *c.selection || p.digest()?.as_str() != c.confirmation {
        return Err(fail("同版本修复预览或候选镜像身份变化"));
    }
    unused()?;
    let protected = backups::protect_upgrade(
        d,
        &backups::UpgradeRequest {
            home: h.into(),
            program_source: None,
        },
    )
    .map_err(|_| fail("完整配置备份无法核验，未修复程序"))?;
    let admit = || {
        unused()?;
        protected
            .verify()
            .map_err(|_| fail("当前配置或保护备份变化，修复结果待核对"))
    };
    let j = if let Some(j) = pending {
        if j.entry_after != c.entry_after {
            return Err(fail("原修复入口后像变化"));
        }
        j
    } else {
        Journal {
            schema_version: 1,
            operation_id: uuid::Uuid::new_v4().to_string(),
            plan: p,
            entry_after: c.entry_after.into(),
        }
    };
    states(&j)?;
    admit()?;
    let bytes = encode(&Envelope {
        sha256: hash::sha256_hex(&encode(&j)?),
        journal: j.clone(),
    })?;
    if bytes.len() as u64 > LIMIT {
        return Err(fail("修复恢复记录超过上限"));
    }
    if let Some(original) = read(i, &j.pending(), LIMIT)? {
        if original != bytes {
            return Err(fail("修复记录在取得写者期间变化"));
        }
    } else {
        create(&j.pending(), &bytes)?;
        safety::sync_directories(i, &i.join("runtime"))?;
    }
    checkpoint("prepared")?;
    let check = || {
        admit()?;
        states(&j)?;
        if read(i, &j.pending(), LIMIT)?.as_deref() != Some(bytes.as_slice()) {
            return Err(fail("原修复操作记录变化"));
        }
        Ok(())
    };
    let history = j.history();
    safety::check_path(i, &history, true)?;
    platform::create_private_dir_all(&history, 0o700).map_err(io)?;
    check()?;
    if let Some(original) = &j.plan.entry_before {
        let path = history.join("entry-before");
        if let Some(b) = read(i, &path, 65536)? {
            if &b != original {
                return Err(fail("原入口备份变化"));
            }
        } else {
            create(&path, original)?;
            safety::sync_directories(i, &history)?;
        }
    }
    checkpoint("entry_archived")?;
    check()?;
    let image = j.plan.image()?;
    let archived = history.join("image");
    if j.plan.image_before.present && !present(i, &archived)? {
        bundled::publish(&image, &archived)?;
        safety::sync_directories(i, &i.join("runtime/versions"))?;
        safety::sync_directories(i, &history)?;
    }
    checkpoint("image_archived")?;
    check()?;
    if !present(i, &image)? {
        safety::check_path(i, c.image, false)?;
        if c.image.starts_with(i.join("runtime")) {
            return Err(fail("修复候选必须来自隔离供应区"));
        }
        bundled::publish(c.image, &image)?;
        safety::sync_directories(i, &i.join("runtime/versions"))?;
    }
    repaired(&j)?;
    checkpoint("image_published")?;
    check()?;
    let target = entry(i)?;
    let observed = read(i, &target, 65536)?;
    if observed.as_deref() != Some(j.entry_after.as_slice()) {
        if observed != j.plan.entry_before {
            return Err(fail("提交修复入口前出现外部修改"));
        }
        atomic_write(&target, &j.entry_after, 0o755)?;
        safety::sync_directories(i, target.parent().unwrap())?;
    }
    checkpoint("entry")?;
    check()?;
    if read(i, &target, 65536)?.as_deref() != Some(j.entry_after.as_slice()) {
        return Err(fail("修复入口读回不符"));
    }
    let receipt_dir = d.join("manager-state/runtime-repairs");
    safety::check_path(d, &receipt_dir, true)?;
    platform::create_private_dir_all(&receipt_dir, 0o700).map_err(io)?;
    let receipt = receipt_dir.join(format!("{}.json", j.operation_id));
    let receipt_bytes = encode(
        &serde_json::json!({"schemaVersion":1,"journalSha256":hash::sha256_hex(&bytes),"journal":j}),
    )?;
    if let Some(b) = read(d, &receipt, LIMIT)? {
        if b != receipt_bytes {
            return Err(fail("已有修复回执不属于原操作"));
        }
    } else {
        create(&receipt, &receipt_bytes)?;
    }
    safety::sync_directories(d, &receipt_dir)?;
    checkpoint("receipt")?;
    check()?;
    fs::remove_file(j.pending()).map_err(io)?;
    safety::sync_directories(i, &i.join("runtime"))?;
    let selected =
        program_selection::read(i, program_layout::host_target()?.1, &c.selection.platform)?
            .ok_or_else(|| fail("修复后当前选择缺失"))?;
    program_image::verify(&selected)?;
    if selected.selection != *c.selection {
        return Err(fail("修复后当前选择不符"));
    }
    Ok(Outcome {
        operation_id: j.operation_id,
        version: c.selection.version.clone(),
        configuration_preserved: true,
    })
}
#[cfg(all(test, unix))]
#[path = "program_repair_tests.rs"]
mod tests;
