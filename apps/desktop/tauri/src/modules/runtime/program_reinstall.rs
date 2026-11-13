//! Explicit resupply after a completed, registered removal. Old evidence is
//! archived byte-for-byte; ordinary consumers refuse the original pending work.
use super::{
    bundled,
    program_activation::Mutation,
    program_catalog, program_image, program_layout,
    program_removal::{self, ReinstallOrigin},
    program_selection::{self, CurrentSelection},
    RuntimeStore,
};
use crate::{
    errors::{AppError, AppResult},
    infrastructure::{hash, platform},
    modules::{backup::safety, model_config::config_file::v2::native_import::backups},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub(crate) const PENDING: &str = ".reinstall-pending.json";
const LIMIT: u64 = 64 * 1024 * 1024;
fn fail(text: &'static str) -> AppError {
    AppError::RuntimeManaged {
        code: "runtime_reinstall_unconfirmed".into(),
        detail: text.into(),
    }
}
fn io(_: std::io::Error) -> AppError {
    fail("内置程序重装文件操作失败，原记录及未改原件保留")
}
fn encode(v: &impl Serialize) -> AppResult<Vec<u8>> {
    serde_json::to_vec(v).map_err(|_| fail("重装记录无法编码"))
}
fn decode<T: serde::de::DeserializeOwned>(b: &[u8]) -> AppResult<T> {
    let v = crate::modules::model_config::schema_v2::parse_strict_json_v2(b)
        .map_err(|_| fail("重装记录不是合法唯一JSON"))?;
    serde_json::from_value(v).map_err(|_| fail("重装记录结构无法核验"))
}
fn read(root: &Path, path: &Path) -> AppResult<Option<Vec<u8>>> {
    safety::check_path(root, path, true)?;
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(io(e)),
        Ok(_) => {
            let f = safety::open_file(path, false)?;
            safety::require_single_link(&f)?;
            let mut bytes = Vec::new();
            f.take(LIMIT + 1).read_to_end(&mut bytes).map_err(io)?;
            if bytes.len() as u64 > LIMIT {
                return Err(fail("重装证据超过读取上限"));
            }
            Ok(Some(bytes))
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
        return Err(fail("程序根不是明确的绝对路径"));
    }
    match fs::symlink_metadata(i) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(io(e)),
        Ok(_) => {}
    }
    if present(i, &i.join("runtime").join(PENDING))? {
        return Err(fail(
            "原内置程序重装尚未完成，请显式继续；未启动或采用其它来源",
        ));
    }
    Ok(())
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Journal {
    schema_version: u32,
    operation_id: String,
    program_root: PathBuf,
    desktop_root: PathBuf,
    home: PathBuf,
    descriptor_pin: String,
    source_sha256: String,
    selection: CurrentSelection,
    origin: ReinstallOrigin,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Envelope {
    sha256: String,
    journal: Journal,
}
impl Journal {
    fn history(&self) -> PathBuf {
        self.program_root
            .join("runtime/reinstall-history")
            .join(&self.operation_id)
    }
    fn pending(&self) -> PathBuf {
        self.program_root.join("runtime").join(PENDING)
    }
    fn marker(&self) -> PathBuf {
        self.program_root
            .join("runtime")
            .join(program_removal::REMOVED)
    }
    fn check(&self, i: &Path, d: &Path, h: &Path) -> AppResult<()> {
        bundled::roots(i, d)?;
        bundled::roots(i, h)?;
        if self.schema_version != 1
            || self.program_root != i
            || self.desktop_root != d
            || self.home != h
            || uuid::Uuid::parse_str(&self.operation_id).is_err()
            || self.selection.platform != program_layout::host_target()?.0
        {
            return Err(fail("原重装操作不属于当前程序或用户配置目录"));
        }
        program_layout::RuntimeVersionPaths::new(
            &i.join("runtime/versions"),
            &self.selection.version,
            program_layout::host_target()?.1,
        )?;
        for value in [
            &self.descriptor_pin,
            &self.source_sha256,
            &self.origin.marker_sha256,
            &self.origin.catalog_revision,
            &self.origin.record_sha256,
        ] {
            if value.len() != 64
                || !value
                    .bytes()
                    .all(|c| c.is_ascii_digit() || matches!(c, b'a'..=b'f'))
            {
                return Err(fail("原重装来源摘要无效"));
            }
        }
        Ok(())
    }
}
fn load(i: &Path, d: &Path, h: &Path) -> AppResult<Option<(Journal, Vec<u8>)>> {
    let Some(bytes) = read(i, &i.join("runtime").join(PENDING))? else {
        return Ok(None);
    };
    let e: Envelope = decode(&bytes)?;
    e.journal.check(i, d, h)?;
    if e.sha256 != hash::sha256_hex(&encode(&e.journal)?) {
        return Err(fail("原重装操作摘要不符"));
    }
    Ok(Some((e.journal, bytes)))
}
fn versions(i: &Path, allowed: Option<&CurrentSelection>) -> AppResult<()> {
    let dir = i.join("runtime/versions");
    safety::check_path(i, &dir, false)?;
    let mut count = 0;
    for row in fs::read_dir(dir).map_err(io)? {
        count += 1;
        let path = row.map_err(io)?.path();
        if count > 1
            || !safety::inspect(&path)?.is_dir()
            || allowed.is_none_or(|a| {
                path.file_name().and_then(|s| s.to_str()) != Some(a.version.as_str())
            })
        {
            return Err(fail("存在原重装范围外的版本或对象，未覆盖"));
        }
    }
    Ok(())
}
fn original(j: &Journal) -> AppResult<()> {
    let i = &j.program_root;
    let archive = j.history().join("removed.json");
    let live = read(i, &j.marker())?;
    let old = read(i, &archive)?;
    let bytes = match (&live, &old) {
        (Some(b), None) | (None, Some(b)) => b,
        _ => return Err(fail("原卸载证据缺失或出现重复，未继续重装")),
    };
    if program_removal::reinstall_origin(i, &j.desktop_root, &j.home, bytes)? != j.origin {
        return Err(fail("原卸载证据在重装期间变化"));
    }
    let archived = j.history().join("image-admissions");
    let revision = if present(i, &archived)? {
        program_catalog::archived_revision(i, &archived)?
    } else {
        program_catalog::revision(i)?
    };
    if revision != j.origin.catalog_revision {
        return Err(fail("原版本登记在重装期间变化"));
    }
    Ok(())
}
/// Read-only before staging the large pinned archive. It is not permission to
/// resume at startup and never adopts a partially removed installation.
pub(super) fn preflight(i: &Path, d: &Path, h: &Path) -> AppResult<()> {
    bundled::roots(i, d)?;
    bundled::roots(i, h)?;
    if let Some((j, _)) = load(i, d, h)? {
        original(&j)?;
        versions(i, Some(&j.selection))?;
        return Ok(());
    }
    if !program_removal::intentionally_removed(i, d)? {
        return Err(fail("仅可在已明确卸载程序后重装内置资源，请先核对程序状态"));
    }
    let b = read(i, &i.join("runtime").join(program_removal::REMOVED))?
        .ok_or_else(|| fail("原卸载证据缺失"))?;
    program_removal::reinstall_origin(i, d, h, &b)?;
    Ok(())
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
pub(crate) fn review(i: &Path, d: &Path, h: &Path, pin: &str, version: &str) -> AppResult<Review> {
    preflight(i, d, h)?;
    let original = if let Some((j, b)) = load(i, d, h)? {
        if j.descriptor_pin != pin || j.selection.version != version {
            return Err(fail("当前内置资源不是原重装候选"));
        }
        (true, b)
    } else {
        (
            false,
            read(i, &i.join("runtime").join(program_removal::REMOVED))?
                .ok_or_else(|| fail("原卸载证据缺失"))?,
        )
    };
    let digest = hash::sha256_hex(&encode(&(
        i,
        d,
        h,
        pin,
        version,
        hash::sha256_hex(&original.1),
    ))?);
    Ok(Review {
        version: version.into(),
        resume: original.0,
        program_root: i.into(),
        config_path: h.join("config.json"),
        confirmation_digest: digest,
    })
}
pub(super) struct Candidate<'a> {
    pub program: &'a Path,
    pub desktop: &'a Path,
    pub home: &'a Path,
    pub staged_image: &'a Path,
    pub selection: &'a CurrentSelection,
    pub descriptor_pin: &'a str,
    pub source_sha256: &'a str,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
    pub operation_id: String,
    pub version: String,
    pub configuration_preserved: bool,
}
/// Caller holds I's Mutation and supplies only its verified, pinned staged image.
/// No removal inventory, arbitrary path or bootstrap bytes are accepted via IPC.
pub(super) fn install(
    c: Candidate<'_>,
    mutation: &Mutation,
    unused: impl Fn() -> AppResult<()>,
    checkpoint: impl Fn(&str) -> AppResult<()>,
) -> AppResult<Outcome> {
    let i = c.program;
    let d = c.desktop;
    let h = c.home;
    mutation.require_root(i)?;
    preflight(i, d, h)?;
    unused()?;
    safety::check_path(i, c.staged_image, false)?;
    if c.staged_image.starts_with(i.join("runtime")) {
        return Err(fail("候选必须来自隔离供应区"));
    }
    let (j, bytes) = if let Some((j, b)) = load(i, d, h)? {
        if j.descriptor_pin != c.descriptor_pin
            || j.source_sha256 != c.source_sha256
            || j.selection != *c.selection
        {
            return Err(fail("内置资源与原重装候选不一致，未创建替代操作"));
        }
        (j, b)
    } else {
        let marker = read(i, &i.join("runtime").join(program_removal::REMOVED))?
            .ok_or_else(|| fail("原卸载证据缺失"))?;
        let origin = program_removal::reinstall_origin(i, d, h, &marker)?;
        let j = Journal {
            schema_version: 1,
            operation_id: uuid::Uuid::new_v4().to_string(),
            program_root: i.into(),
            desktop_root: d.into(),
            home: h.into(),
            descriptor_pin: c.descriptor_pin.into(),
            source_sha256: c.source_sha256.into(),
            selection: c.selection.clone(),
            origin,
        };
        j.check(i, d, h)?;
        let b = encode(&Envelope {
            sha256: hash::sha256_hex(&encode(&j)?),
            journal: j.clone(),
        })?;
        (j, b)
    };
    original(&j)?;
    let protected = backups::protect_upgrade(
        d,
        &backups::UpgradeRequest {
            home: h.into(),
            program_source: None,
        },
    )
    .map_err(|_| fail("当前完整配置无法备份或核验，未执行重装"))?;
    let admit = || {
        unused()?;
        protected
            .verify()
            .map_err(|_| fail("配置或保护备份发生变化，重装结果待核对"))
    };
    admit()?;
    if let Some(b) = read(i, &j.pending())? {
        if b != bytes {
            return Err(fail("取得写者期间原重装操作变化"));
        }
    } else {
        create(&j.pending(), &bytes)?;
        safety::sync_directories(i, &i.join("runtime"))?;
    }
    checkpoint("prepared")?;
    let check = || {
        admit()?;
        if read(i, &j.pending())?.as_deref() != Some(bytes.as_slice()) {
            return Err(fail("原重装记录变化"));
        }
        original(&j)
    };
    let history = j.history();
    safety::check_path(i, &history, true)?;
    platform::create_private_dir_all(&history, 0o700).map_err(io)?;
    check()?;
    let old_marker = history.join("removed.json");
    if read(i, &old_marker)?.is_none() {
        bundled::publish(&j.marker(), &old_marker)?;
        safety::sync_directories(i, &i.join("runtime"))?;
        safety::sync_directories(i, &history)?;
    }
    checkpoint("marker_archived")?;
    check()?;
    let old_catalog = history.join("image-admissions");
    let catalog = i.join("runtime/image-admissions");
    if !present(i, &old_catalog)? {
        bundled::publish(&catalog, &old_catalog)?;
        safety::sync_directories(i, &i.join("runtime"))?;
        safety::sync_directories(i, &history)?;
    }
    checkpoint("catalog_archived")?;
    check()?;
    versions(i, Some(c.selection))?;
    let paths = program_layout::RuntimeVersionPaths::new(
        &i.join("runtime/versions"),
        &c.selection.version,
        program_layout::host_target()?.1,
    )?;
    if !present(i, &paths.version_root)? {
        bundled::publish(c.staged_image, &paths.version_root)?;
        safety::sync_directories(i, &i.join("runtime/versions"))?;
    }
    let candidate = program_selection::read_image(
        i,
        program_layout::host_target()?.1,
        &c.selection.platform,
        c.selection,
    )?;
    if candidate.image.official_archive_sha256 != c.source_sha256 {
        return Err(fail("实际镜像来源与原内置候选不符"));
    }
    program_image::verify(&candidate)?;
    checkpoint("image")?;
    check()?;
    if present(i, &catalog)? {
        program_catalog::require_only(i, c.selection, c.source_sha256)?;
    }
    program_catalog::register(i, c.selection, c.source_sha256)?;
    program_catalog::require_only(i, c.selection, c.source_sha256)?;
    checkpoint("admission")?;
    check()?;
    let activation_path = history.join("activation.json");
    let activation = if let Some(b) = read(i, &activation_path)? {
        b
    } else {
        // Do not turn a changed source into a new unresolved baseline.
        if read(d, RuntimeStore::new(d).path())?
            .as_deref()
            .map(hash::sha256_hex)
            .as_deref()
            != Some(j.origin.record_sha256.as_str())
        {
            return Err(fail("原卸载来源后像变化"));
        }
        let b = mutation.prepare_bootstrap(d, c.selection, &j.origin.record_sha256)?;
        create(&activation_path, &b)?;
        safety::sync_directories(i, &history)?;
        b
    };
    checkpoint("activation_prepared")?;
    let selected = mutation.complete_bootstrap(
        d,
        &activation,
        c.selection,
        &j.origin.record_sha256,
        check,
        |phase| checkpoint(phase),
    )?;
    if selected.selection != *c.selection {
        return Err(fail("重装激活结果不符"));
    }
    check()?;
    checkpoint("activated")?;
    let receipts = d.join("manager-state/runtime-reinstalls");
    safety::check_path(d, &receipts, true)?;
    platform::create_private_dir_all(&receipts, 0o700).map_err(io)?;
    let receipt = receipts.join(format!("{}.json", j.operation_id));
    let receipt_bytes = encode(
        &serde_json::json!({"schemaVersion":1,"journalSha256":hash::sha256_hex(&bytes),"activationSha256":hash::sha256_hex(&activation),"journal":j}),
    )?;
    if let Some(b) = read(d, &receipt)? {
        if b != receipt_bytes {
            return Err(fail("已有重装回执与原操作不符"));
        }
    } else {
        create(&receipt, &receipt_bytes)?;
    }
    safety::sync_directories(d, &receipts)?;
    checkpoint("receipt")?;
    check()?;
    fs::remove_file(j.pending()).map_err(io)?;
    safety::sync_directories(i, &i.join("runtime"))?;
    if program_selection::read(i, program_layout::host_target()?.1, &c.selection.platform)?
        .as_ref()
        .map(|s| &s.selection)
        != Some(c.selection)
    {
        return Err(fail("内置程序重装最终读回不符"));
    }
    Ok(Outcome {
        operation_id: j.operation_id,
        version: c.selection.version.clone(),
        configuration_preserved: true,
    })
}
#[cfg(all(test, unix))]
#[path = "program_reinstall_tests.rs"]
mod tests;
