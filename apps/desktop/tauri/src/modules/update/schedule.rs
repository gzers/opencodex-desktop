//! Persistent update deadlines. Metadata is disposable; scheduling survives cache cleanup.
use super::UpdateChannel;
use crate::errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

pub const STATE_PATH: &str = "manager-state/update-schedule.json";
pub const STARTUP_DELAY_SECONDS: u64 = 45;
const DAY: i64 = 86400;
const MAX_BYTES: u64 = 16 * 1024;
const MAX_CACHE_BYTES: u64 = 512 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    ManagerStable,
    ManagerBeta,
    Panel,
}
impl Target {
    pub fn manager(channel: UpdateChannel) -> Self {
        match channel {
            UpdateChannel::Stable => Self::ManagerStable,
            UpdateChannel::Beta => Self::ManagerBeta,
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Self::ManagerStable => "manager-stable",
            Self::ManagerBeta => "manager-beta",
            Self::Panel => "panel-latest",
        }
    }
    pub fn interval(self) -> i64 {
        if self == Self::ManagerBeta {
            21600
        } else {
            DAY
        }
    }
    fn delay(self, failures: u8) -> i64 {
        match failures {
            1 => 300,
            2 => 1800,
            3 => 7200,
            _ => self.interval(),
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Deadline {
    pub next_due: i64,
    pub failures: u8,
    pub last_success: Option<i64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Schedule {
    pub schema: u32,
    pub entries: BTreeMap<String, Deadline>,
}
impl Default for Schedule {
    fn default() -> Self {
        Self {
            schema: 1,
            entries: BTreeMap::new(),
        }
    }
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub targets: Vec<Target>,
    pub next_delay_ms: Option<u64>,
}
impl Schedule {
    pub fn load(root: &Path) -> AppResult<Self> {
        let path = super::storage::file_path(root, Path::new(STATE_PATH), false)?;
        let bytes = match super::storage::read_file(&path, MAX_BYTES) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(_) => return Err(AppError::NotConfigured),
        };
        let value: Self = serde_json::from_slice(&bytes).map_err(|_| AppError::NotConfigured)?;
        if value.schema != 1
            || value.entries.len() > 3
            || value.entries.iter().any(|(key, d)| {
                !["manager-stable", "manager-beta", "panel-latest"].contains(&key.as_str())
                    || d.next_due < 0
                    || d.failures > 4
                    || d.last_success.is_some_and(|t| t < 0)
            })
        {
            return Err(AppError::NotConfigured);
        }
        Ok(value)
    }
    pub fn save(&self, root: &Path) -> AppResult<()> {
        let bytes = serde_json::to_vec(self).map_err(|_| AppError::NotConfigured)?;
        let path = super::storage::file_path(root, Path::new(STATE_PATH), true)?;
        crate::infrastructure::atomic_write::atomic_write(&path, &bytes, 0o600)
    }
    /// Rebase only impossible future deadlines for the selected objects. Persist this
    /// repair before planning so reloads/wakes cannot keep postponing the deadline.
    /// Retain failure counts and history, and wait the existing interval/backoff
    /// rather than querying immediately after a rollback or corrupt future time.
    pub fn reconcile_clock(&mut self, now: i64, channel: UpdateChannel) -> bool {
        let mut changed = false;
        for target in [Target::manager(channel), Target::Panel] {
            if let Some(d) = self.entries.get_mut(target.key()) {
                let latest = now.saturating_add(target.delay(d.failures)).max(0);
                if d.next_due > latest {
                    d.next_due = latest;
                    changed = true;
                }
            }
        }
        changed
    }
    pub fn plan(&self, now: i64, channel: UpdateChannel, enabled: bool, busy: bool) -> Plan {
        if !enabled {
            return Plan {
                targets: vec![],
                next_delay_ms: None,
            };
        }
        if busy {
            return Plan {
                targets: vec![],
                next_delay_ms: Some(60000),
            };
        }
        let candidates = [Target::manager(channel), Target::Panel];
        let targets = candidates
            .iter()
            .copied()
            .filter(|t| self.entries.get(t.key()).is_none_or(|d| d.next_due <= now))
            .collect::<Vec<_>>();
        let next_delay_ms = if targets.is_empty() {
            candidates
                .iter()
                .filter_map(|t| self.entries.get(t.key()))
                .map(|d| (d.next_due.saturating_sub(now).clamp(1, DAY) as u64) * 1000)
                .min()
        } else {
            Some(1000)
        };
        Plan {
            targets,
            next_delay_ms,
        }
    }
    /// Persist before launching a query: a crash/restart cannot cause a request storm.
    pub fn reserve(&mut self, target: Target, now: i64) {
        let d = self.entries.entry(target.key().into()).or_default();
        d.next_due = now.saturating_add(300);
    }
    pub fn complete(&mut self, target: Target, now: i64, success: bool) {
        let d = self.entries.entry(target.key().into()).or_default();
        let delay = if success {
            d.failures = 0;
            d.last_success = Some(now);
            target.interval()
        } else {
            d.failures = d.failures.saturating_add(1).min(4);
            target.delay(d.failures)
        };
        d.next_due = now.saturating_add(delay);
    }
}
pub fn cache_path(root: &Path, target: Target) -> std::path::PathBuf {
    root.join("cache/updates")
        .join(format!("{}.json", target.key()))
}
pub fn save_cache<T: Serialize>(root: &Path, target: Target, value: &T) -> AppResult<()> {
    let bytes = serde_json::to_vec(value).map_err(|_| AppError::NotConfigured)?;
    if bytes.len() as u64 > MAX_CACHE_BYTES {
        return Err(AppError::NotConfigured);
    }
    let relative = Path::new("cache/updates").join(format!("{}.json", target.key()));
    let path = super::storage::file_path(root, &relative, true)?;
    crate::infrastructure::atomic_write::atomic_write(&path, &bytes, 0o600)
}
pub fn load_cache<T: serde::de::DeserializeOwned>(root: &Path, target: Target) -> Option<T> {
    let relative = Path::new("cache/updates").join(format!("{}.json", target.key()));
    let path = super::storage::file_path(root, &relative, false).ok()?;
    let bytes = super::storage::read_file(&path, MAX_CACHE_BYTES).ok()?;
    serde_json::from_slice(&bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    fn directory_link(target: &Path, link: &Path) {
        std::os::unix::fs::symlink(target, link).unwrap();
    }

    #[cfg(windows)]
    fn directory_link(target: &Path, link: &Path) {
        use std::os::windows::process::CommandExt;
        let powershell = std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap())
            .join("System32/WindowsPowerShell/v1.0/powershell.exe");
        // A fixed OS junction command on two test-owned temporary roots; no
        // developer-mode/symlink privilege or application instance is required.
        let result = std::process::Command::new(powershell)
            .args(["-NoProfile", "-NonInteractive", "-Command",
                "$ErrorActionPreference='Stop'; New-Item -ItemType Junction -Path $env:OPENCODEX_TEST_LINK -Target $env:OPENCODEX_TEST_TARGET | Out-Null"])
            // Pass paths as string values, never interpolate them into script code.
            .env("OPENCODEX_TEST_LINK", link)
            .env("OPENCODEX_TEST_TARGET", target)
            .creation_flags(0x0800_0000)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "junction fixture failed: {result:?}"
        );
        use std::os::windows::fs::MetadataExt;
        assert_ne!(
            std::fs::symlink_metadata(link).unwrap().file_attributes() & 0x400,
            0
        );
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn linked_schedule_partition_never_reads_or_rewrites_another_root() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let mut saved = Schedule::default();
        saved.complete(Target::ManagerStable, 100, true);
        saved.save(outside.path()).unwrap();
        let foreign = outside.path().join(STATE_PATH);
        let before = std::fs::read(&foreign).unwrap();
        directory_link(
            &outside.path().join("manager-state"),
            &root.path().join("manager-state"),
        );
        let read_refused = Schedule::load(root.path()).is_err();
        let mut replacement = Schedule::default();
        replacement.complete(Target::Panel, 200, false);
        let write_refused = replacement.save(root.path()).is_err();
        let unchanged = std::fs::read(&foreign).unwrap() == before;
        assert!(read_refused && write_refused && unchanged,
            "linked schedule escaped root: read_refused={read_refused}, write_refused={write_refused}, external_unchanged={unchanged}");
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn linked_version_cache_never_reads_or_rewrites_another_root() {
        for component in ["cache", "updates", "file"] {
            let root = tempfile::tempdir().unwrap();
            let outside = tempfile::tempdir().unwrap();
            let prior = serde_json::json!({"version":"2.83.0"});
            save_cache(outside.path(), Target::Panel, &prior).unwrap();
            let foreign = cache_path(outside.path(), Target::Panel);
            let before = std::fs::read(&foreign).unwrap();
            match component {
                "cache" => {
                    directory_link(&outside.path().join("cache"), &root.path().join("cache"))
                }
                "updates" => {
                    std::fs::create_dir(root.path().join("cache")).unwrap();
                    directory_link(
                        &outside.path().join("cache/updates"),
                        &root.path().join("cache/updates"),
                    );
                }
                _ => {
                    std::fs::create_dir_all(root.path().join("cache/updates")).unwrap();
                    #[cfg(unix)]
                    std::os::unix::fs::symlink(&foreign, cache_path(root.path(), Target::Panel))
                        .unwrap();
                    #[cfg(windows)]
                    std::fs::hard_link(&foreign, cache_path(root.path(), Target::Panel)).unwrap();
                }
            }
            let read_refused =
                load_cache::<serde_json::Value>(root.path(), Target::Panel).is_none();
            let write_refused = save_cache(
                root.path(),
                Target::Panel,
                &serde_json::json!({"version":"2.84.0"}),
            )
            .is_err();
            let unchanged = std::fs::read(&foreign).unwrap() == before;
            assert!(read_refused && write_refused && unchanged,
                "linked cache {component} escaped root: read_refused={read_refused}, write_refused={write_refused}, external_unchanged={unchanged}");
        }
    }
    #[test]
    fn escaped_bounded_release_notes_fit_cache_and_oversized_cache_is_refused() {
        let root = tempfile::tempdir().unwrap();
        // JSON escapes may be six bytes for each byte of plain release notes.
        let metadata = serde_json::json!({"notes": "\u{0001}".repeat(64 * 1024)});
        save_cache(root.path(), Target::ManagerStable, &metadata).unwrap();
        assert_eq!(
            load_cache::<serde_json::Value>(root.path(), Target::ManagerStable),
            Some(metadata)
        );
        let oversized = serde_json::json!({"notes": "x".repeat(MAX_CACHE_BYTES as usize)});
        assert!(save_cache(root.path(), Target::ManagerStable, &oversized).is_err());
        std::fs::write(
            cache_path(root.path(), Target::ManagerStable),
            vec![b' '; MAX_CACHE_BYTES as usize + 1],
        )
        .unwrap();
        assert!(load_cache::<serde_json::Value>(root.path(), Target::ManagerStable).is_none());
    }

    #[test]
    fn hundred_routes_and_ten_restarts_with_fresh_schedule_make_no_requests() {
        let root = tempfile::tempdir().unwrap();
        let mut state = Schedule::default();
        state.complete(Target::ManagerStable, 100, true);
        state.complete(Target::Panel, 100, true);
        state.save(root.path()).unwrap();
        for _ in 0..10 {
            let state = Schedule::load(root.path()).unwrap();
            for _ in 0..100 {
                assert!(state
                    .plan(200, UpdateChannel::Stable, true, false)
                    .targets
                    .is_empty());
            }
        }
    }
    #[test]
    fn backoff_sleep_channel_isolation_and_cache_cleanup() {
        let root = tempfile::tempdir().unwrap();
        let mut s = Schedule::default();
        s.complete(Target::Panel, 10, true);
        for delay in [300, 1800, 7200, 86400] {
            s.complete(Target::ManagerStable, 20, false);
            assert_eq!(s.entries["manager-stable"].next_due, 20 + delay)
        }
        assert_eq!(
            s.plan(100, UpdateChannel::Beta, true, false).targets,
            vec![Target::ManagerBeta]
        );
        assert!(s
            .plan(100, UpdateChannel::Beta, false, false)
            .next_delay_ms
            .is_none());
        assert!(s
            .plan(100, UpdateChannel::Beta, true, true)
            .targets
            .is_empty());
        // Long sleep returns each overdue object once, no catch-up list.
        assert_eq!(
            s.plan(999999, UpdateChannel::Stable, true, false)
                .targets
                .len(),
            2
        );
        save_cache(
            root.path(),
            Target::Panel,
            &serde_json::json!({"version":"1"}),
        )
        .unwrap();
        s.save(root.path()).unwrap();
        std::fs::remove_dir_all(root.path().join("cache")).unwrap();
        assert_eq!(
            Schedule::load(root.path()).unwrap().entries["panel-latest"].next_due,
            86410
        );
    }
    #[test]
    fn channel_intervals_are_fixed_and_not_user_configurable() {
        assert_eq!(Target::ManagerStable.interval(), DAY);
        assert_eq!(Target::Panel.interval(), DAY);
        assert_eq!(Target::ManagerBeta.interval(), 6 * 60 * 60);
    }
    #[test]
    fn malformed_state_is_not_silently_reset_and_crash_reservation_survives() {
        let root = tempfile::tempdir().unwrap();
        let mut s = Schedule::default();
        s.reserve(Target::Panel, 100);
        s.save(root.path()).unwrap();
        assert!(Schedule::load(root.path())
            .unwrap()
            .plan(110, UpdateChannel::Stable, true, false)
            .targets
            .iter()
            .all(|t| *t != Target::Panel));
        std::fs::write(root.path().join(STATE_PATH), b"bad").unwrap();
        assert!(Schedule::load(root.path()).is_err());
    }
    #[test]
    fn invalid_deadline_history_is_refused_without_reset() {
        for (next_due, last_success, failures) in [(100, -1, 0), (-1, 100, 0), (100, 100, 5)] {
            let root = tempfile::tempdir().unwrap();
            let mut state = Schedule::default();
            state.entries.insert(
                Target::Panel.key().into(),
                Deadline {
                    next_due,
                    last_success: Some(last_success),
                    failures,
                },
            );
            state.save(root.path()).unwrap();
            let bytes = std::fs::read(root.path().join(STATE_PATH)).unwrap();
            assert!(Schedule::load(root.path()).is_err());
            assert_eq!(std::fs::read(root.path().join(STATE_PATH)).unwrap(), bytes);
        }
    }
}
