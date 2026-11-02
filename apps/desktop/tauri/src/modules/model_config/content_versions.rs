//! Saved-content versions, separate from operation/envelope creation times.
//! Unknown legacy save time stays unknown; reading/transferring never uses now.
use super::{schema::*, store::sha256};
use crate::{
    infrastructure::{atomic_write::atomic_write, locking::TargetFileLock},
    modules::backup::safety,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};
const FILE: &str = "manager-state/saved-content-versions.json";
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Version {
    digest: String,
    revision: u64,
    saved_at: String,
}
pub fn stamp(
    root: &Path,
    key: &str,
    digest: &str,
    saved_at: &str,
    inherited: bool,
) -> ModelResult<String> {
    let p = root.join(FILE);
    safety::check_path(root, &p, true)?;
    let _g = TargetFileLock::lock(&p)?;
    let mut versions: BTreeMap<String, Version> = if p.exists() {
        serde_json::from_slice(&safety::read_file(&p, 2 * 1024 * 1024)?)?
    } else {
        BTreeMap::new()
    };
    if !saved_at.is_empty() && chrono::DateTime::parse_from_rfc3339(saved_at).is_err() {
        return Err(invalid("content_version", "保存时间格式无效"));
    }
    if let Some(old) = versions.get(key) {
        if old.digest == digest && !inherited {
            return Ok(old.saved_at.clone());
        }
    }
    let revision = versions
        .get(key)
        .map(|old| old.revision.saturating_add(u64::from(old.digest != digest)))
        .unwrap_or(1);
    if !versions.contains_key(key) && versions.len() >= 2048 {
        if let Some(first) = versions.keys().next().cloned() {
            versions.remove(&first);
        }
    }
    versions.insert(
        key.into(),
        Version {
            digest: digest.into(),
            revision,
            saved_at: saved_at.into(),
        },
    );
    atomic_write(&p, &serde_json::to_vec(&versions)?, 0o600)?;
    Ok(saved_at.into())
}
pub fn key(value: &impl Serialize) -> ModelResult<String> {
    Ok(sha256(&serde_json::to_vec(value)?))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repeated_reads_and_transfer_keep_the_original_content_time() {
        let r = tempfile::tempdir().unwrap();
        let original = "2026-10-01T00:00:00Z";
        assert_eq!(
            stamp(r.path(), "scope", "digest", original, false).unwrap(),
            original
        );
        assert_eq!(
            stamp(r.path(), "scope", "digest", "2026-11-02T00:00:00Z", false).unwrap(),
            original
        );
        stamp(r.path(), "scope", "new-digest", original, true).unwrap();
        assert_eq!(
            stamp(
                r.path(),
                "scope",
                "new-digest",
                "2026-11-02T00:00:00Z",
                false
            )
            .unwrap(),
            original
        );
    }
    #[test]
    fn unknown_legacy_time_is_not_replaced_with_the_transfer_clock() {
        let r = tempfile::tempdir().unwrap();
        assert_eq!(stamp(r.path(), "scope", "digest", "", false).unwrap(), "");
        assert_eq!(
            stamp(r.path(), "scope", "digest", "2026-11-02T00:00:00Z", false).unwrap(),
            ""
        );
    }
}
