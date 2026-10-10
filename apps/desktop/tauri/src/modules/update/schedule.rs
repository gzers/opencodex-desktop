//! Persistent update deadlines. Metadata is disposable; scheduling survives cache cleanup.
use super::UpdateChannel;
use crate::errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io::Read, path::Path};

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
        let path = root.join(STATE_PATH);
        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(_) => return Err(AppError::NotConfigured),
        };
        let mut bytes = Vec::new();
        file.take(MAX_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| AppError::NotConfigured)?;
        if bytes.len() as u64 > MAX_BYTES {
            return Err(AppError::NotConfigured);
        }
        let value: Self = serde_json::from_slice(&bytes).map_err(|_| AppError::NotConfigured)?;
        if value.schema != 1
            || value.entries.len() > 3
            || value.entries.iter().any(|(key, d)| {
                !["manager-stable", "manager-beta", "panel-latest"].contains(&key.as_str())
                    || d.next_due < 0
                    || d.failures > 4
            })
        {
            return Err(AppError::NotConfigured);
        }
        Ok(value)
    }
    pub fn save(&self, root: &Path) -> AppResult<()> {
        let bytes = serde_json::to_vec(self).map_err(|_| AppError::NotConfigured)?;
        crate::infrastructure::atomic_write::atomic_write(&root.join(STATE_PATH), &bytes, 0o600)
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
            match d.failures {
                1 => 300,
                2 => 1800,
                3 => 7200,
                _ => target.interval(),
            }
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
    crate::infrastructure::atomic_write::atomic_write(&cache_path(root, target), &bytes, 0o600)
}
pub fn load_cache<T: serde::de::DeserializeOwned>(root: &Path, target: Target) -> Option<T> {
    let file = std::fs::File::open(cache_path(root, target)).ok()?;
    let mut bytes = Vec::new();
    file.take(MAX_CACHE_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() as u64 > MAX_CACHE_BYTES {
        return None;
    }
    serde_json::from_slice(&bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
