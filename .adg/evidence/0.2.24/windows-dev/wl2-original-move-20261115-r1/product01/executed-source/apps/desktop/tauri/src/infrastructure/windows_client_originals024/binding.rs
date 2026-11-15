//! Strict persisted identity data. Decoding never constructs NativeSession,
//! OriginalSession, an original handle, custody or ClosedEligibility.
use super::phase::{digest, invalid};
use crate::infrastructure::windows_official_runtime024::validate_local_path;
use serde::{Deserialize, Serialize};
use std::{io, path::Path};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct DirectoryBinding {
    path: String,
    volume: u32,
    file_id: String,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct OwnerBinding {
    pid: u32,
    creation_filetime: String,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct Binding {
    sid: String,
    profile: DirectoryBinding,
    data: DirectoryBinding,
    home: DirectoryBinding,
    installation: DirectoryBinding,
    source_digest: String,
    current_digest: String,
    owner: OwnerBinding,
}
fn positive_decimal(value: &str) -> Option<u64> {
    if value.is_empty()
        || value.len() > 20
        || value.starts_with('0')
        || !value.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    value.parse::<u64>().ok().filter(|n| *n > 0)
}
fn sid(value: &str) -> bool {
    let Some(tail) = value.strip_prefix("S-1-") else {
        return false;
    };
    let parts: Vec<_> = tail.split('-').collect();
    let canonical = |part: &str, bound: u64| {
        part.parse::<u64>()
            .is_ok_and(|n| n <= bound && n.to_string() == part)
    };
    (2..=16).contains(&parts.len())
        && canonical(parts[0], 0xffff_ffff_ffff)
        && parts[1..]
            .iter()
            .all(|part| canonical(part, u32::MAX as u64))
}
fn key(path: &str) -> String {
    dunce::simplified(Path::new(path))
        .to_string_lossy()
        .replace('/', "\\")
        .to_lowercase()
}
fn overlap(a: &str, b: &str) -> bool {
    a == b
        || a.strip_prefix(b).is_some_and(|s| s.starts_with('\\'))
        || b.strip_prefix(a).is_some_and(|s| s.starts_with('\\'))
}
impl DirectoryBinding {
    fn validate(&self) -> io::Result<()> {
        // Local syntax only. File IDs/volume values in a record are data until
        // checked against original retained native directory/security objects.
        if self.path.encode_utf16().count() > 32760 || positive_decimal(&self.file_id).is_none() {
            return Err(invalid());
        }
        validate_local_path(Path::new(&self.path)).map_err(|_| invalid())?;
        let normalized: std::path::PathBuf = dunce::simplified(Path::new(&self.path))
            .components()
            .collect();
        if key(&self.path) != key(&normalized.to_string_lossy()) {
            return Err(invalid());
        }
        Ok(())
    }
    fn same_object(&self, other: &Self) -> bool {
        self.volume == other.volume && self.file_id == other.file_id
    }
}
impl Binding {
    pub(super) fn validate(&self, expected_sid: Option<&str>) -> io::Result<()> {
        if !sid(&self.sid)
            || expected_sid.is_some_and(|expected| self.sid != expected)
            || !digest(&self.source_digest)
            || !digest(&self.current_digest)
            || self.owner.pid == 0
            || positive_decimal(&self.owner.creation_filetime).is_none()
        {
            return Err(invalid());
        }
        let dirs = [&self.profile, &self.data, &self.home, &self.installation];
        for dir in dirs {
            dir.validate()?;
        }
        for i in 0..dirs.len() {
            for j in i + 1..dirs.len() {
                if dirs[i].same_object(dirs[j]) {
                    return Err(invalid());
                }
            }
        }
        let controls = [
            key(&self.data.path),
            key(&self.home.path),
            key(&self.installation.path),
        ];
        let profile = key(&self.profile.path);
        let clients = [format!("{profile}\\.codex"), format!("{profile}\\.grok")];
        for i in 0..controls.len() {
            if controls[i] == profile
                || controls[i].is_empty()
                || clients.iter().any(|root| overlap(&controls[i], root))
            {
                return Err(invalid());
            }
            for j in i + 1..controls.len() {
                if overlap(&controls[i], &controls[j]) {
                    return Err(invalid());
                }
            }
        }
        Ok(())
    }
}
