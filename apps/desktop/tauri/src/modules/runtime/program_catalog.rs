//! Explicit image-admission records produced by installers. The catalog never
//! infers a removable installation by enumerating arbitrary image manifests.
use super::{
    program_image, program_layout,
    program_selection::{self, CurrentSelection},
};
use crate::{
    errors::{AppError, AppResult},
    infrastructure::{hash, platform},
    modules::backup::safety,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
const DIRECTORY: &str = "runtime/image-admissions";
const MAX_RECORD: u64 = 64 * 1024;
fn fail() -> AppError {
    AppError::RuntimeManaged {
        code: "runtime_catalog_unconfirmed".into(),
        detail: "运行版本登记或实际目录无法核对，未接管未登记的程序".into(),
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Admission {
    schema_version: u32,
    program_root: PathBuf,
    selection: CurrentSelection,
    official_archive_sha256: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub selections: Vec<CurrentSelection>,
    pub revision: String,
}
fn filename(s: &CurrentSelection) -> String {
    format!("{}-{}.json", s.version, s.image_sha256)
}
fn read(root: &Path, path: &Path) -> AppResult<Vec<u8>> {
    safety::check_path(root, path, false)?;
    let file = safety::open_file(path, false)?;
    safety::require_single_link(&file)?;
    let mut bytes = Vec::new();
    file.take(MAX_RECORD + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| fail())?;
    if bytes.len() as u64 > MAX_RECORD {
        return Err(fail());
    }
    Ok(bytes)
}
fn decode(bytes: &[u8]) -> AppResult<Admission> {
    let value =
        crate::modules::model_config::schema_v2::parse_strict_json_v2(bytes).map_err(|_| fail())?;
    serde_json::from_value(value).map_err(|_| fail())
}
fn admitted(root: &Path, a: &Admission) -> AppResult<()> {
    let (target, platform) = program_layout::host_target()?;
    if a.schema_version != 1 || a.program_root != root || a.selection.platform != target {
        return Err(fail());
    }
    let image = program_selection::read_image(root, platform, target, &a.selection)?;
    if image.image.official_archive_sha256 != a.official_archive_sha256 {
        return Err(fail());
    }
    program_image::verify(&image)
}
/// Installer-only: after the pinned resource or selected official archive has
/// been checked and published, before any post-publication executable probe.
/// This durable record covers even a candidate whose later probe/activation fails.
pub(super) fn register(
    root: &Path,
    selection: &CurrentSelection,
    source_sha256: &str,
) -> AppResult<()> {
    let record = Admission {
        schema_version: 1,
        program_root: root.into(),
        selection: selection.clone(),
        official_archive_sha256: source_sha256.into(),
    };
    admitted(root, &record)?;
    let directory = root.join(DIRECTORY);
    safety::check_path(root, &directory, true)?;
    platform::create_private_dir_all(&directory, 0o700).map_err(|_| fail())?;
    let path = directory.join(filename(selection));
    match fs::symlink_metadata(&path) {
        Ok(_) => {
            if decode(&read(root, &path)?)? != record {
                return Err(fail());
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let bytes = serde_json::to_vec(&record).map_err(|_| fail())?;
            let mut file = platform::create_new_private_file(&path, 0o600).map_err(|_| fail())?;
            file.write_all(&bytes).map_err(|_| fail())?;
            file.sync_all().map_err(|_| fail())?;
            if read(root, &path)? != bytes {
                return Err(fail());
            }
        }
        Err(_) => return Err(fail()),
    }
    safety::sync_directories(root, &directory)?;
    Ok(())
}
/// Read the explicit ledger first, then compare actual version names. Unknown,
/// missing or edited rows/images are refusals, not implicit adoption or pruning.
fn records(root: &Path) -> AppResult<(BTreeMap<String, Admission>, String)> {
    records_at(root, &root.join(DIRECTORY))
}
fn records_at(root: &Path, directory: &Path) -> AppResult<(BTreeMap<String, Admission>, String)> {
    safety::check_path(root, directory, false)?;
    let mut records = BTreeMap::new();
    let mut selections = BTreeMap::new();
    let mut count = 0usize;
    for row in fs::read_dir(directory).map_err(|_| fail())? {
        count += 1;
        if count > 1000 {
            return Err(fail());
        }
        let path = row.map_err(|_| fail())?.path();
        let bytes = read(root, &path)?;
        let a = decode(&bytes)?;
        if path.file_name().and_then(|n| n.to_str()) != Some(filename(&a.selection).as_str())
            || selections
                .insert(a.selection.version.clone(), a.clone())
                .is_some()
        {
            return Err(fail());
        }
        if a.schema_version != 1
            || a.program_root != root
            || a.selection.schema_version != 1
            || a.selection.platform != program_layout::host_target()?.0
            || ![&a.selection.image_sha256, &a.official_archive_sha256]
                .into_iter()
                .all(|s| {
                    s.len() == 64
                        && s.bytes()
                            .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
                })
        {
            return Err(fail());
        }
        super::program_layout::RuntimeVersionPaths::new(
            &root.join("runtime/versions"),
            &a.selection.version,
            program_layout::host_target()?.1,
        )?;
        records.insert(
            path.file_name().unwrap().to_string_lossy().into_owned(),
            hash::sha256_hex(&bytes),
        );
    }
    if selections.is_empty() {
        return Err(fail());
    }
    let revision = hash::sha256_hex(&serde_json::to_vec(&records).map_err(|_| fail())?);
    Ok((selections, revision))
}
/// Ledger identity only, usable after an acknowledged image has been partially
/// removed. It never grants admission to changed or unregistered image content.
pub fn revision(root: &Path) -> AppResult<String> {
    records(root).map(|(_, revision)| revision)
}
/// Same original ledger bytes after explicit archival; never admits live images.
pub(super) fn archived_revision(root: &Path, directory: &Path) -> AppResult<String> {
    records_at(root, directory).map(|(_, revision)| revision)
}
pub(super) fn require_only(
    root: &Path,
    selection: &CurrentSelection,
    source: &str,
) -> AppResult<()> {
    let (records, _) = records(root)?;
    if records.len() != 1
        || records
            .values()
            .next()
            .is_none_or(|a| a.selection != *selection || a.official_archive_sha256 != source)
    {
        return Err(fail());
    }
    Ok(())
}

pub fn inspect(root: &Path) -> AppResult<Catalog> {
    let (admissions, revision) = records(root)?;
    for admission in admissions.values() {
        admitted(root, admission)?;
    }
    let selections = admissions
        .into_iter()
        .map(|(version, admission)| (version, admission.selection))
        .collect::<BTreeMap<_, _>>();
    let versions = root.join("runtime/versions");
    safety::check_path(root, &versions, false)?;
    let mut actual = std::collections::BTreeSet::new();
    for row in fs::read_dir(&versions).map_err(|_| fail())? {
        let path = row.map_err(|_| fail())?.path();
        if !safety::inspect(&path)?.is_dir() {
            return Err(fail());
        }
        let name = path.file_name().and_then(|n| n.to_str()).ok_or_else(fail)?;
        if !selections.contains_key(name) || !actual.insert(name.to_string()) {
            return Err(fail());
        }
    }
    if actual.len() != selections.len() {
        return Err(fail());
    }
    let (target, platform) = program_layout::host_target()?;
    let current = program_selection::read(root, platform, target)?.ok_or_else(fail)?;
    if selections.get(&current.selection.version) != Some(&current.selection) {
        return Err(fail());
    }
    Ok(Catalog {
        selections: selections.into_values().collect(),
        revision,
    })
}

#[cfg(all(test, unix))]
#[path = "program_catalog_tests.rs"]
mod tests;
