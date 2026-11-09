//! Content-save receipts. Only successful semantic saves create a clock value;
//! observation, migration, packaging and transfer never manufacture one.
use super::{schema::*, store::sha256};
use crate::{
    infrastructure::{atomic_write::atomic_write, locking::TargetFileLock},
    modules::backup::safety,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};
#[path = "file_save_versions.rs"]
pub mod file_saves;
#[path = "owner_save_versions.rs"]
pub mod owner_saves;

const FILE: &str = "manager-state/saved-content-versions.json";
/// Frozen target for L0's unified content + version-byte transaction.
pub const SAVED_VERSIONS_PATH: &str = FILE;
const MAX_VERSION_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedVersionsV2 {
    pub before: Option<Vec<u8>>,
    pub after: Vec<u8>,
}

pub type SaveState = BTreeMap<String, String>;
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TimeKind {
    Saved,
    LegacyMtime,
    ClockUnreliable,
    #[default]
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Version {
    #[serde(default)]
    pub scope_key: String,
    #[serde(alias = "digest")]
    pub content_digest: String,
    #[serde(default)]
    pub revision: u64,
    #[serde(alias = "saved_at")]
    pub saved_at: String,
    // Old receipts were inferred from mtime. Missing provenance is untrusted.
    #[serde(default)]
    pub time_kind: TimeKind,
    /// Highest trustworthy clock instant seen for this identity. It survives
    /// rollback, content replacement and tombstones; never inferred from mtime.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trusted_floor: Option<String>,
    /// Idempotency belongs to a durable save event, not to bytes which can
    /// return after an unobserved external edit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_event: Option<String>,
}
impl Version {
    pub fn instant(&self) -> Option<chrono::DateTime<chrono::FixedOffset>> {
        (self.time_kind == TimeKind::Saved)
            .then(|| chrono::DateTime::parse_from_rfc3339(&self.saved_at).ok())
            .flatten()
    }
    fn lower_bound(&self) -> Option<chrono::DateTime<chrono::FixedOffset>> {
        self.instant().max(
            self.trusted_floor
                .as_deref()
                .and_then(|time| chrono::DateTime::parse_from_rfc3339(time).ok()),
        )
    }
    pub fn unknown(key: &str, digest: &str) -> Self {
        Self {
            scope_key: key.into(),
            content_digest: digest.into(),
            revision: 0,
            saved_at: String::new(),
            time_kind: TimeKind::Unknown,
            trusted_floor: None,
            last_event: None,
        }
    }
}
fn read(root: &Path) -> ModelResult<BTreeMap<String, Version>> {
    let p = root.join(FILE);
    safety::check_path(root, &p, true)?;
    if !p.exists() {
        return Ok(BTreeMap::new());
    }
    let mut records: BTreeMap<String, Version> =
        serde_json::from_slice(&safety::read_file(&p, 8 * 1024 * 1024)?)?;
    for (key, version) in &mut records {
        if version.scope_key.is_empty() {
            version.scope_key = key.clone();
        }
    }
    Ok(records)
}
/// Caller retains outer file-save admission before taking this version lock.
/// Pure planners never acquire it again; keep it through commit and readback.
pub fn lock_versions(root: &Path) -> ModelResult<TargetFileLock> {
    let p = root.join(FILE);
    safety::check_path(root, &p, true)?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&p),
        true,
    )?;
    checkpoint("before_version_lock")?;
    let guard = TargetFileLock::lock(&p)?;
    checkpoint("version_locked")?;
    Ok(guard)
}

fn encode_records(records: &BTreeMap<String, Version>) -> ModelResult<Vec<u8>> {
    if records.len() > 30000 {
        return Err(invalid("content_version", "保存版本数量超过上限"));
    }
    let bytes = serde_json::to_vec(&records)?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err(invalid("content_version", "保存版本文件超过上限"));
    }
    Ok(bytes)
}

fn update(
    root: &Path,
    f: impl FnOnce(&mut BTreeMap<String, Version>) -> ModelResult<()>,
) -> ModelResult<()> {
    let p = root.join(FILE);
    let _guard = lock_versions(root)?;
    let mut records = read(root)?;
    f(&mut records)?;
    let bytes = encode_records(&records)?;
    atomic_write(&p, &bytes, 0o600)?;
    if safety::read_file(&p, 8 * 1024 * 1024)? != bytes {
        return Err(invalid(
            "content_version_readback",
            "保存版本写入后读回不同，事件待恢复",
        ));
    }
    Ok(())
}
pub fn observe(root: &Path, state: &SaveState) -> ModelResult<Vec<Version>> {
    let records = read(root)?;
    Ok(state
        .iter()
        .map(|(key, digest)| {
            records
                .get(key)
                .filter(|v| v.scope_key == *key && v.content_digest == *digest)
                .cloned()
                .unwrap_or_else(|| Version::unknown(key, digest))
        })
        .collect())
}
/// Call with snapshots taken before and after a successful, read-back save.
/// Unchanged/previously unknown objects stay untouched, even on first discovery.
pub fn record_save(root: &Path, before: &SaveState, after: &SaveState) -> ModelResult<()> {
    record_at(
        root,
        before,
        after,
        &chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
    )
}
pub(super) fn record_at(
    root: &Path,
    before: &SaveState,
    after: &SaveState,
    time: &str,
) -> ModelResult<()> {
    record_at_validated(
        root,
        before,
        after,
        time,
        &uuid::Uuid::new_v4().to_string(),
        || Ok(()),
    )
}
/// The caller's final content check and this version mutation share the version
/// lock; file saves additionally retain their cross-process admission guard.
pub(super) fn record_at_validated(
    root: &Path,
    before: &SaveState,
    after: &SaveState,
    time: &str,
    event_id: &str,
    validate: impl FnOnce() -> ModelResult<()>,
) -> ModelResult<()> {
    require_uuid(event_id)?;
    let instant = chrono::DateTime::parse_from_rfc3339(time)
        .map_err(|_| invalid("content_version", "保存时间格式无效"))?;
    if before == after {
        return validate();
    }
    update(root, |records| {
        validate()?;
        apply_save_at(records, before, after, time, event_id, instant)
    })
}

/// Single source for old writes and v2 plans: no IO, clock reads or callbacks.
fn apply_save_at(
    records: &mut BTreeMap<String, Version>,
    before: &SaveState,
    after: &SaveState,
    time: &str,
    event_id: &str,
    instant: chrono::DateTime<chrono::FixedOffset>,
) -> ModelResult<()> {
    for (key, digest) in after {
        if before.get(key) == Some(digest) {
            continue;
        }
        let old = records.get(key);
        // A real save can restore old bytes after an external edit. Only
        // this same durable event is a replay, even if bytes/time repeat.
        if let Some(old) = old.filter(|old| old.last_event.as_deref() == Some(event_id)) {
            if old.content_digest != *digest || old.saved_at != time {
                return Err(invalid(
                    "content_version_event_conflict",
                    "同一保存事件的内容或时间不同",
                ));
            }
            continue;
        }
        let floor = old.and_then(Version::lower_bound);
        let unreliable = if old.is_some_and(|old| old.time_kind == TimeKind::ClockUnreliable) {
            floor.is_none_or(|floor| instant <= floor)
        } else {
            floor.is_some_and(|floor| instant < floor)
        };
        let trusted_floor = if unreliable {
            floor.map(|time| time.to_rfc3339())
        } else {
            Some(
                floor
                    .map_or(instant, |floor| floor.max(instant))
                    .to_rfc3339(),
            )
        };
        records.insert(
            key.clone(),
            Version {
                scope_key: key.clone(),
                content_digest: digest.clone(),
                revision: old.map_or(1, |v| v.revision.saturating_add(1)),
                saved_at: time.into(),
                time_kind: if unreliable {
                    TimeKind::ClockUnreliable
                } else {
                    TimeKind::Saved
                },
                trusted_floor,
                last_event: Some(event_id.into()),
            },
        );
    }
    for key in before.keys().filter(|key| !after.contains_key(*key)) {
        if let Some(old) = records.get_mut(key) {
            if old.last_event.as_deref() == Some(event_id) {
                continue;
            }
            // A removed object must not erase the clock high-water mark.
            old.trusted_floor = old.lower_bound().map(|time| time.to_rfc3339());
            old.content_digest.clear();
            old.revision = old.revision.saturating_add(1);
            old.last_event = Some(event_id.into());
        }
    }
    Ok(())
}

/// Prepare exact receipt bytes without taking locks or writing any file.
/// The caller must hold file-save admission, then lock_versions(root), through
/// this read/plan and the unified transaction commit/readback. Replay keeps the
/// frozen event/time; neither mtime nor the current wall clock is consulted.
pub fn plan_save_at(
    root: &Path,
    before: &SaveState,
    after: &SaveState,
    time: &str,
    event_id: &str,
) -> ModelResult<PlannedVersionsV2> {
    require_uuid(event_id)?;
    let instant = chrono::DateTime::parse_from_rfc3339(time)
        .map_err(|_| invalid("content_version", "保存时间格式无效"))?;
    let path = root.join(FILE);
    safety::check_path(root, &path, true)?;
    let original = match std::fs::symlink_metadata(&path) {
        Ok(_) => Some(safety::read_file(&path, MAX_VERSION_BYTES as u64)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    let mut records: BTreeMap<String, Version> = match &original {
        Some(bytes) => {
            let value = super::schema_v2::parse_strict_json_v2(bytes).map_err(|error| {
                invalid(&error.code, "保存版本文件不是严格有效JSON，原件已保留")
            })?;
            serde_json::from_value(value)?
        }
        None => BTreeMap::new(),
    };
    for (key, version) in &mut records {
        if version.scope_key.is_empty() {
            version.scope_key = key.clone();
        }
    }
    // Still strictly validate no-op documents; do not normalize their aliases,
    // whitespace, saved times, revisions or clock floors merely by observing.
    if before == after {
        encode_records(&records)?;
        return Ok(PlannedVersionsV2 {
            before: original.clone(),
            after: original.unwrap_or_else(|| b"{}".to_vec()),
        });
    }
    apply_save_at(&mut records, before, after, time, event_id, instant)?;
    Ok(PlannedVersionsV2 {
        before: original,
        after: encode_records(&records)?,
    })
}

/// Import only exact applied content. Re-receiving identical bytes cannot change
/// an existing content time, including an unknown legacy time.
pub fn inherit(root: &Path, versions: &[Version]) -> ModelResult<()> {
    if versions.is_empty() {
        return Ok(());
    }
    update(root, |records| {
        for version in versions {
            if records
                .get(&version.scope_key)
                .is_some_and(|old| old.content_digest == version.content_digest)
            {
                continue;
            }
            let mut inherited = version.clone();
            inherited.trusted_floor = inherited
                .lower_bound()
                .max(
                    records
                        .get(&version.scope_key)
                        .and_then(Version::lower_bound),
                )
                .map(|time| time.to_rfc3339());
            records.insert(version.scope_key.clone(), inherited);
        }
        Ok(())
    })
}
pub fn key(value: &impl Serialize) -> ModelResult<String> {
    Ok(sha256(&serde_json::to_vec(&serde_json::to_value(value)?)?))
}

#[cfg(test)]
type TestVersionHook = Box<dyn FnMut(&str) -> ModelResult<()>>;
#[cfg(test)]
thread_local! { pub(super) static TEST_VERSION_HOOK: std::cell::RefCell<Option<TestVersionHook>> = const { std::cell::RefCell::new(None) }; }
fn checkpoint(stage: &str) -> ModelResult<()> {
    #[cfg(test)]
    {
        TEST_VERSION_HOOK.with(|hook| match &mut *hook.borrow_mut() {
            Some(hook) => hook(stage),
            None => Ok(()),
        })
    }
    #[cfg(not(test))]
    {
        let _ = stage;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_save_noop_external_copy_and_inheritance_have_separate_lifecycles() {
        let r = tempfile::tempdir().unwrap();
        let a = BTreeMap::from([("scope".into(), "digest".into())]);
        assert_eq!(
            observe(r.path(), &a).unwrap()[0].time_kind,
            TimeKind::Unknown
        );
        record_at(r.path(), &a, &a, "2026-11-04T00:00:00Z").unwrap();
        assert!(observe(r.path(), &a).unwrap()[0].instant().is_none());
        record_at(r.path(), &SaveState::new(), &a, "2026-10-30T00:00:00Z").unwrap();
        let original = observe(r.path(), &a).unwrap();
        let mut later = original.clone();
        later[0].saved_at = "2026-11-04T00:00:00Z".into();
        inherit(r.path(), &later).unwrap();
        assert_eq!(observe(r.path(), &a).unwrap(), original);
        let b = BTreeMap::from([("scope".into(), "external".into())]);
        assert!(observe(r.path(), &b).unwrap()[0].instant().is_none());
        record_at(r.path(), &a, &b, "2026-10-01T00:00:00Z").unwrap();
        assert_eq!(
            observe(r.path(), &b).unwrap()[0].time_kind,
            TimeKind::ClockUnreliable
        );
    }
    #[test]
    fn legacy_receipts_are_readable_but_mtime_is_not_trusted() {
        let r = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(r.path().join("manager-state")).unwrap();
        std::fs::write(
            r.path().join(FILE),
            r#"{"scope":{"digest":"d","revision":1,"saved_at":"2026-11-04T00:00:00Z"}}"#,
        )
        .unwrap();
        let state = BTreeMap::from([("scope".into(), "d".into())]);
        assert!(observe(r.path(), &state).unwrap()[0].instant().is_none());
    }
    #[test]
    fn post04_review_rollback_stays_unreliable_until_the_trusted_floor_is_crossed() {
        let root = tempfile::tempdir().unwrap();
        let state = |value: &str| BTreeMap::from([("same-scope".into(), value.into())]);
        let a = state("a");
        let b = state("b");
        let c = state("c");
        let d = state("d");
        record_at(root.path(), &SaveState::new(), &a, "2026-11-04T12:00:00Z").unwrap();
        record_at(root.path(), &a, &b, "2026-11-04T10:00:00Z").unwrap();
        assert_eq!(
            observe(root.path(), &b).unwrap()[0].time_kind,
            TimeKind::ClockUnreliable
        );
        record_at(root.path(), &b, &c, "2026-11-04T11:00:00Z").unwrap();
        assert_eq!(
            observe(root.path(), &c).unwrap()[0].time_kind,
            TimeKind::ClockUnreliable
        );
        record_at(root.path(), &c, &d, "2026-11-04T12:00:00Z").unwrap();
        assert_eq!(
            observe(root.path(), &d).unwrap()[0].time_kind,
            TimeKind::ClockUnreliable
        );
        let e = state("e");
        record_at(root.path(), &d, &e, "2026-11-04T12:00:01Z").unwrap();
        assert_eq!(
            observe(root.path(), &e).unwrap()[0].time_kind,
            TimeKind::Saved
        );
    }
}

#[cfg(test)]
#[path = "content_versions_v2_tests.rs"]
mod v2_tests;
