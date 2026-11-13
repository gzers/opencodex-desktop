//! Read observations separately from installation authority. An external package
//! version can be shown without admitting its code, rewriting history or executing it.
use super::{
    program_layout::{self, RuntimeVersionPaths, CURRENT_FILENAME},
    program_selection::{self, CurrentSelection},
    RuntimeSourceRecord, RuntimeStore, OFFICIAL_PACKAGE,
};
use crate::{
    errors::{AppError, AppResult},
    infrastructure::hash,
    modules::backup::safety,
};
use serde::Serialize;
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};
fn fail() -> AppError {
    AppError::RuntimeManaged {
        code: "version_observation_unconfirmed".into(),
        detail: "实际版本读取期间来源发生变化，请重新核对".into(),
    }
}
fn read(root: &Path, path: &Path, limit: u64) -> AppResult<Option<Vec<u8>>> {
    if matches!(fs::symlink_metadata(root),Err(e) if e.kind()==std::io::ErrorKind::NotFound) {
        return Ok(None);
    }
    safety::check_path(root, path, true)?;
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(fail()),
        Ok(_) => {
            let f = safety::open_file(path, false)?;
            safety::require_single_link(&f)?;
            let mut b = Vec::new();
            f.take(limit + 1).read_to_end(&mut b).map_err(|_| fail())?;
            if b.len() as u64 > limit {
                return Err(fail());
            }
            Ok(Some(b))
        }
    }
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> AppResult<T> {
    let v =
        crate::modules::model_config::schema_v2::parse_strict_json_v2(bytes).map_err(|_| fail())?;
    serde_json::from_value(v).map_err(|_| fail())
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Integrity {
    Consistent,
    Changed,
    Missing,
    Pending,
    Unavailable,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskFacts {
    pub disk_version: Option<String>,
    pub selected_version: Option<String>,
    pub recorded_version: Option<String>,
    pub recorded_at: Option<String>,
    pub integrity: Integrity,
    pub checked_at: String,
    pub confirmation_digest: String,
}
pub fn observe(i: &Path, d: &Path) -> AppResult<DiskFacts> {
    observe_with(i, d, || {})
}
fn observe_with(i: &Path, d: &Path, checkpoint: impl FnOnce()) -> AppResult<DiskFacts> {
    super::bundled::roots(i, d)?;
    let current = i.join("runtime").join(CURRENT_FILENAME);
    let record_path = RuntimeStore::new(d).path().to_path_buf();
    let record = read(d, &record_path, 1024 * 1024)?;
    let historical = record
        .as_deref()
        .and_then(|b| decode::<RuntimeSourceRecord>(b).ok())
        .filter(|r| r.schema_version == super::RUNTIME_SCHEMA_VERSION);
    let mut facts = DiskFacts {
        disk_version: None,
        selected_version: None,
        recorded_version: historical.as_ref().and_then(|r| r.resolved_version.clone()),
        recorded_at: historical.and_then(|r| r.resolved_at),
        integrity: Integrity::Unavailable,
        checked_at: chrono::Utc::now().to_rfc3339(),
        confirmation_digest: String::new(),
    };
    let mut pending = Vec::new();
    for name in [
        program_selection::ACTIVATION_PENDING,
        super::program_removal::PENDING,
        super::program_reinstall::PENDING,
        super::program_repair::PENDING,
    ] {
        pending.push(read(i, &i.join("runtime").join(name), 64 * 1024 * 1024)?);
    }
    let before = read(i, &current, 65536)?;
    let mut package: Option<(PathBuf, Option<Vec<u8>>)> = None;
    if let Some(bytes) = &before {
        if let Ok(s) = decode::<CurrentSelection>(bytes) {
            let (target, platform) = program_layout::host_target()?;
            if s.schema_version == 1
                && s.platform == target
                && s.image_sha256.len() == 64
                && s.image_sha256.bytes().all(|b| b.is_ascii_hexdigit())
            {
                if let Ok(paths) =
                    RuntimeVersionPaths::new(&i.join("runtime/versions"), &s.version, platform)
                {
                    facts.selected_version = Some(s.version.clone());
                    let path = paths
                        .package_root
                        .join(super::install::PACKAGE_SUBPATH)
                        .join("package.json");
                    let bytes = read(i, &path, 65536)?;
                    if let Some(b) = &bytes {
                        if let Ok(value) = decode::<serde_json::Value>(b) {
                            if value["name"] == OFFICIAL_PACKAGE {
                                facts.disk_version = value["version"]
                                    .as_str()
                                    .filter(|v| crate::modules::update::handoff::valid_version(v))
                                    .map(str::to_owned);
                            }
                        }
                    }
                    package = Some((path, bytes));
                    facts.integrity = if facts.disk_version.is_none() {
                        Integrity::Unavailable
                    } else {
                        match program_selection::read(i, platform, target) {
                            Ok(Some(selected)) if selected.selection == s => Integrity::Consistent,
                            _ => Integrity::Changed,
                        }
                    };
                }
            }
        }
    } else {
        facts.integrity = Integrity::Missing;
    }
    if pending.iter().any(Option::is_some) {
        facts.integrity = Integrity::Pending;
    }
    checkpoint();
    if read(i, &current, 65536)? != before || read(d, &record_path, 1024 * 1024)? != record {
        return Err(fail());
    }
    if let Some((path, bytes)) = &package {
        if &read(i, path, 65536)? != bytes {
            return Err(fail());
        }
    }
    let mut after_pending = Vec::new();
    for name in [
        program_selection::ACTIVATION_PENDING,
        super::program_removal::PENDING,
        super::program_reinstall::PENDING,
        super::program_repair::PENDING,
    ] {
        after_pending.push(read(i, &i.join("runtime").join(name), 64 * 1024 * 1024)?);
    }
    if after_pending != pending {
        return Err(fail());
    }
    facts.confirmation_digest = hash::sha256_hex(
        &serde_json::to_vec(&(i, d, before, record, package, pending)).map_err(|_| fail())?,
    );
    Ok(facts)
}
pub(crate) fn require_confirmation(i: &Path, d: &Path, digest: &str) -> AppResult<DiskFacts> {
    let facts = observe(i, d)?;
    if facts.integrity != Integrity::Consistent || facts.confirmation_digest != digest {
        return Err(fail());
    }
    let (target, platform) = program_layout::host_target()?;
    let selected = program_selection::read(i, platform, target)?.ok_or_else(fail)?;
    super::program_image::verify(&selected)?;
    if observe(i, d)?.confirmation_digest != digest {
        return Err(fail());
    }
    Ok(facts)
}

#[cfg(all(test, unix))]
#[path = "version_facts_tests.rs"]
mod tests;
