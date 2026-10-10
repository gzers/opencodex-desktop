//! 管理器本地清理命令层；只作用于显式数据根内的日志分区。
//!
//! 除用户显式触发的「清理日志 / 清理通知」外，本模块还承载「启动时清理」策略：
//! 按偏好里的日志保留、通知保留与备份保留策略做一次轻量后台清理。

use tauri::State;

use chrono::{DateTime, SecondsFormat, Utc};
use std::path::Path;

use crate::errors::{AppError, AppResult};
use crate::state::{SharedDataRoot, SharedNotificationStore};
use crate::types::cleanup::LocalCleanupResultDto;
use crate::types::notifications::NotificationsDto;

/// 日志保留策略缺省值（与设置项 `30d-10000` 一致）。
pub const DEFAULT_LOG_RETENTION_DAYS: i64 = 30;
pub const DEFAULT_LOG_RETENTION_LINES: usize = 10_000;
/// 通知保留策略缺省值（与设置项 `30` 天一致）。
pub const DEFAULT_NOTIFICATION_RETENTION_DAYS: i64 = 30;
/// 备份保留策略缺省值（与设置项 `10` 份一致）。
pub const DEFAULT_BACKUP_RETENTION: usize = 10;
/// 审计日志不参与自动截断：它是安全审计凭据，只能由用户显式清理。
const AUDIT_LOG_EXCLUDED_FROM_TRIM: &str = crate::modules::logs::AUDIT_LOG_FILE_NAME;

#[tauri::command]
pub async fn cleanup_local_logs(
    app: tauri::AppHandle,
    data_root: State<'_, SharedDataRoot>,
) -> AppResult<LocalCleanupResultDto> {
    let root = data_root.0.clone();
    crate::commands::run_blocking("cleanup local logs", move || {
        let identity = crate::commands::event_delivery::prepare(
            &root,
            "local-cleanup-failed",
            crate::modules::notifications::registry::Channel::Local,
            b"manual-local-logs",
        );
        let result = cleanup_local_logs_with_root(&root);
        crate::commands::event_delivery::publish(
            &app,
            &root,
            if result.is_ok() {
                "local-cleanup-succeeded"
            } else {
                "local-cleanup-failed"
            },
            identity,
            crate::modules::notifications::registry::Trigger::User,
        );
        result
    })
    .await
}

#[tauri::command]
pub fn cleanup_local_notifications(
    store: State<'_, SharedNotificationStore>,
    data_root: State<'_, SharedDataRoot>,
) -> AppResult<NotificationsDto> {
    crate::commands::notifications::clear_notifications_with_store(store.inner(), &data_root.0)
}

pub fn cleanup_local_logs_with_root(
    data_root: &std::path::Path,
) -> AppResult<LocalCleanupResultDto> {
    let logs = data_root.join("logs");
    crate::modules::backup::safety::check_path(data_root, &logs, true)?;
    if !logs.exists() {
        return Ok(LocalCleanupResultDto {
            cleaned_logs: 0,
            summary: "没有本地日志文件；原始目录保持不变。".to_string(),
        });
    }
    let mut cleaned = 0_usize;
    for entry in std::fs::read_dir(&logs).map_err(|error| AppError::FileSystem {
        operation: "read local logs".to_string(),
        detail: error.to_string(),
    })? {
        let entry = entry.map_err(|error| AppError::FileSystem {
            operation: "read local logs".to_string(),
            detail: error.to_string(),
        })?;
        let path = entry.path();
        crate::modules::backup::safety::check_path(data_root, &path, false)?;
        if crate::modules::backup::safety::inspect(&path)?.is_file() {
            let file = crate::modules::backup::safety::open_existing_writable(&path)?;
            let metadata = file.metadata().map_err(|error| AppError::FileSystem {
                operation: "inspect local log".into(),
                detail: error.to_string(),
            })?;
            crate::modules::backup::safety::require_single_link(&file)?;
            if metadata.len() == 0 {
                continue;
            }
            file.set_len(0)
                .and_then(|()| file.sync_all())
                .map_err(|error| AppError::FileSystem {
                    operation: "clear local log".into(),
                    detail: error.to_string(),
                })?;
            cleaned += 1;
        }
    }
    Ok(LocalCleanupResultDto {
        cleaned_logs: cleaned,
        summary: format!("已清理 {cleaned} 个本地日志文件；不删除日志目录。"),
    })
}

/// 解析 `30d-10000` 形式的日志保留策略；无法解析时回落到默认值，不臆测。
pub fn parse_log_retention(value: &str) -> (i64, usize) {
    let Some((days, lines)) = value.split_once('-') else {
        return (DEFAULT_LOG_RETENTION_DAYS, DEFAULT_LOG_RETENTION_LINES);
    };
    let days = days
        .trim_end_matches('d')
        .parse::<i64>()
        .ok()
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_LOG_RETENTION_DAYS);
    let lines = lines
        .parse::<usize>()
        .ok()
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_LOG_RETENTION_LINES);
    (days, lines)
}

/// 解析通知保留天数；无法解析时回落到默认值。
pub fn parse_notification_retention(value: &str) -> i64 {
    value
        .trim()
        .parse::<i64>()
        .ok()
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_NOTIFICATION_RETENTION_DAYS)
}

/// 解析备份保留份数；无法解析时回落到默认值。
pub fn parse_backup_retention(value: &str) -> usize {
    value
        .trim()
        .parse::<usize>()
        .ok()
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_BACKUP_RETENTION)
}

/// 一次「启动时清理」的真实结果；只带计数与落点，不携带日志或通知正文。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RetentionCleanupOutcome {
    pub ran: bool,
    pub trimmed_log_files: usize,
    pub trimmed_log_lines: usize,
    pub removed_backups: usize,
    pub pruned_notifications: usize,
    pub summary_path: Option<String>,
}

/// 按偏好执行一次保留期清理。
///
/// 关闭「启动时清理」时**不做任何事**，如实返回 `ran=false`；
/// 读取偏好失败按错误上报，不静默跳过。
#[cfg(test)]
pub fn run_startup_cleanup(
    data_root: &Path,
    now: DateTime<Utc>,
) -> Result<RetentionCleanupOutcome, AppError> {
    let preferences = crate::modules::preferences::PreferencesStore::new(data_root)
        .load()
        .map_err(|_| AppError::NotConfigured)?;
    if !preferences.startup_cleanup {
        return Ok(RetentionCleanupOutcome::default());
    }
    retention_cleanup(data_root, &preferences, now, false)
}

/// Prepare one owned startup attempt before cleanup; disabled means no identity
/// and no event. Delivery must wait until the post-cleanup history is managed.
pub fn run_startup_cleanup_registered(
    data_root: &Path,
    now: DateTime<Utc>,
) -> (
    AppResult<RetentionCleanupOutcome>,
    Option<crate::modules::notifications::registry::EventIdentity>,
) {
    let preferences = crate::modules::preferences::PreferencesStore::new(data_root).load();
    if preferences.as_ref().is_ok_and(|p| !p.startup_cleanup) {
        return (Ok(RetentionCleanupOutcome::default()), None);
    }
    let identity = crate::commands::event_delivery::prepare(
        data_root,
        "local-cleanup-failed",
        crate::modules::notifications::registry::Channel::Local,
        b"startup-local-retention",
    );
    let result = preferences
        .map_err(|_| AppError::NotConfigured)
        .and_then(|p| retention_cleanup(data_root, &p, now, false));
    (result, identity)
}

/// 按给定偏好执行保留期清理（导出给测试与后台任务复用）。
pub fn run_retention_cleanup(
    data_root: &Path,
    preferences: &crate::modules::preferences::Preferences,
    now: DateTime<Utc>,
) -> Result<RetentionCleanupOutcome, AppError> {
    retention_cleanup(data_root, preferences, now, true)
}

fn retention_cleanup(
    data_root: &Path,
    preferences: &crate::modules::preferences::Preferences,
    now: DateTime<Utc>,
    include_backups: bool,
) -> Result<RetentionCleanupOutcome, AppError> {
    let (log_days, log_lines) = parse_log_retention(&preferences.log_retention);
    let (trimmed_files, trimmed_lines) =
        trim_logs_by_retention(data_root, now, log_days, log_lines)?;
    let backup_retention = parse_backup_retention(&preferences.backup_retention);
    let removed_backups = if include_backups {
        crate::modules::backup::cleanup_retention(data_root, now, backup_retention)?
    } else {
        Vec::new()
    };
    let notification_days = parse_notification_retention(&preferences.notification_retention);
    let pruned_notifications = prune_notifications_by_retention(data_root, now, notification_days)?;

    let outcome_counts = RetentionCleanupOutcome {
        ran: true,
        trimmed_log_files: trimmed_files,
        trimmed_log_lines: trimmed_lines,
        removed_backups: removed_backups.len(),
        pruned_notifications,
        summary_path: None,
    };
    if !preferences.cleanup_backup_summary
        || (trimmed_lines == 0 && removed_backups.is_empty() && pruned_notifications == 0)
    {
        return Ok(outcome_counts);
    }
    let summary_path = write_cleanup_summary(data_root, now, &outcome_counts)?;
    Ok(RetentionCleanupOutcome {
        summary_path: Some(summary_path),
        ..outcome_counts
    })
}

/// 按保留期与行数上限截断日志；审计日志不参与，未知时间戳的行按新行保留。
fn trim_logs_by_retention(
    data_root: &Path,
    now: DateTime<Utc>,
    max_days: i64,
    max_lines: usize,
) -> Result<(usize, usize), AppError> {
    let logs = data_root.join("logs");
    crate::modules::backup::safety::check_path(data_root, &logs, true)?;
    if !logs.exists() {
        return Ok((0, 0));
    }
    let mut trimmed_files = 0_usize;
    let mut trimmed_lines = 0_usize;
    for entry in std::fs::read_dir(&logs).map_err(|error| AppError::FileSystem {
        operation: "read local logs".to_string(),
        detail: error.to_string(),
    })? {
        let entry = entry.map_err(|error| AppError::FileSystem {
            operation: "read local logs".to_string(),
            detail: error.to_string(),
        })?;
        let path = entry.path();
        crate::modules::backup::safety::check_path(data_root, &path, false)?;
        if !crate::modules::backup::safety::inspect(&path)?.is_file()
            || path.file_name().and_then(|name| name.to_str()) == Some(AUDIT_LOG_EXCLUDED_FROM_TRIM)
        {
            continue;
        }
        let bytes = crate::modules::backup::safety::read_file(
            &path,
            crate::modules::backup::safety::MAX_PAYLOAD_BYTES,
        )?;
        let raw = String::from_utf8(bytes).map_err(|error| AppError::FileSystem {
            operation: "read local log".into(),
            detail: error.to_string(),
        })?;
        let lines: Vec<&str> = raw.lines().collect();
        if lines.is_empty() {
            continue;
        }
        let fresh: Vec<&str> = lines
            .iter()
            .copied()
            .filter(|line| line_is_within_retention(line, now, max_days))
            .collect();
        let keep_from = fresh.len().saturating_sub(max_lines);
        let kept = &fresh[keep_from..];
        if kept.len() == lines.len() {
            continue;
        }
        let mut payload = kept.join("\n");
        if !payload.is_empty() {
            payload.push('\n');
        }
        crate::infrastructure::atomic_write::atomic_write(&path, payload.as_bytes(), 0o600)
            .map_err(|error| AppError::FileSystem {
                operation: "trim local log".to_string(),
                detail: error.to_string(),
            })?;
        trimmed_files += 1;
        trimmed_lines += lines.len() - kept.len();
    }
    Ok((trimmed_files, trimmed_lines))
}

/// 时间戳不可解析时按保留处理（宁可留，不误删用户日志）。
fn line_is_within_retention(line: &str, now: DateTime<Utc>, max_days: i64) -> bool {
    let Some(token) = line.split_whitespace().next() else {
        return true;
    };
    let Ok(created) = DateTime::parse_from_rfc3339(token) else {
        return true;
    };
    now.signed_duration_since(created.with_timezone(&Utc))
        .num_days()
        <= max_days
}

/// 清理超过保留期的已读通知（软删除，保留历史判定依据），返回被清理条数。
fn prune_notifications_by_retention(
    data_root: &Path,
    now: DateTime<Utc>,
    max_days: i64,
) -> Result<usize, AppError> {
    let path = crate::modules::notifications::persistence::notifications_path(data_root);
    crate::modules::backup::safety::check_path(data_root, &path, true)?;
    let mut store = crate::modules::notifications::persistence::load_notifications(&path)?;
    let expired: Vec<String> = store
        .live()
        .into_iter()
        .filter(|item| item.read)
        .filter(|item| {
            DateTime::parse_from_rfc3339(&item.created_at)
                .map(|created| {
                    now.signed_duration_since(created.with_timezone(&Utc))
                        .num_days()
                        > max_days
                })
                .unwrap_or(false)
        })
        .map(|item| item.notification_id.clone())
        .collect();
    if expired.is_empty() {
        return Ok(0);
    }
    for notification_id in &expired {
        store.delete(notification_id);
    }
    crate::modules::notifications::persistence::save_notifications(&path, &store)
        .map_err(|_| AppError::NotConfigured)?;
    Ok(expired.len())
}

/// 清理后写一份**脱敏摘要**（只有计数与时间，没有日志或通知正文）。
fn write_cleanup_summary(
    data_root: &Path,
    now: DateTime<Utc>,
    outcome: &RetentionCleanupOutcome,
) -> Result<String, AppError> {
    let backup_id = crate::modules::backup::generate_backup_id(now);
    let directory = data_root
        .join("backups")
        .join(now.format("%Y").to_string())
        .join(now.format("%m").to_string())
        .join("cleanup")
        .join(&backup_id);
    std::fs::create_dir_all(&directory).map_err(|error| AppError::FileSystem {
        operation: "create cleanup summary directory".to_string(),
        detail: error.to_string(),
    })?;
    let summary = serde_json::json!({
        "schema_version": 1,
        "cleanup_id": backup_id,
        "created_at": now.to_rfc3339_opts(SecondsFormat::Secs, true),
        "trimmed_log_files": outcome.trimmed_log_files,
        "trimmed_log_lines": outcome.trimmed_log_lines,
        "removed_backups": outcome.removed_backups,
        "pruned_notifications": outcome.pruned_notifications,
    });
    let path = directory.join("cleanup-summary.json");
    let payload = serde_json::to_vec_pretty(&summary).map_err(|_| AppError::NotConfigured)?;
    crate::infrastructure::atomic_write::atomic_write(&path, &payload, 0o600).map_err(|error| {
        AppError::FileSystem {
            operation: "write cleanup summary".to_string(),
            detail: error.to_string(),
        }
    })?;
    Ok(path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup_clears_only_log_partition_files() {
        let root = tempfile::tempdir().expect("temp root");
        crate::modules::data_root::initialize(root.path()).expect("initialize");
        let logs = root.path().join("logs");
        std::fs::create_dir_all(&logs).expect("create logs");
        std::fs::write(logs.join("app.log"), b"line\n").expect("write log");
        std::fs::write(root.path().join("manager-state/preferences.json"), b"{}")
            .expect("write managed state");
        let result = cleanup_local_logs_with_root(root.path()).expect("cleanup");
        assert_eq!(result.cleaned_logs, 1);
        assert_eq!(
            std::fs::read(logs.join("app.log")).expect("read log"),
            Vec::<u8>::new()
        );
        assert_eq!(
            std::fs::read(root.path().join("manager-state/preferences.json")).expect("read state"),
            b"{}"
        );
    }

    #[test]
    fn cleanup_handles_missing_logs() {
        let root = tempfile::tempdir().expect("temp root");
        crate::modules::data_root::initialize(root.path()).expect("initialize");
        let result = cleanup_local_logs_with_root(root.path()).expect("cleanup");
        assert_eq!(result.cleaned_logs, 0);
    }

    #[cfg(unix)]
    #[test]
    fn log_cleanup_rejects_linked_directory_and_preserves_external_log() {
        let root = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let log = external.path().join("app.log");
        std::fs::write(&log, b"external log").unwrap();
        std::os::unix::fs::symlink(external.path(), root.path().join("logs")).unwrap();
        assert!(cleanup_local_logs_with_root(root.path()).is_err());
        assert!(trim_logs_by_retention(root.path(), Utc::now(), 1, 1).is_err());
        assert_eq!(std::fs::read(&log).unwrap(), b"external log");
    }

    #[cfg(unix)]
    #[test]
    fn log_cleanup_rejects_linked_file_and_preserves_external_log() {
        let root = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let log = external.path().join("app.log");
        std::fs::write(&log, b"external log").unwrap();
        std::fs::create_dir(root.path().join("logs")).unwrap();
        std::os::unix::fs::symlink(&log, root.path().join("logs/app.log")).unwrap();
        assert!(cleanup_local_logs_with_root(root.path()).is_err());
        assert!(trim_logs_by_retention(root.path(), Utc::now(), 1, 1).is_err());
        assert_eq!(std::fs::read(&log).unwrap(), b"external log");
    }

    #[test]
    fn manual_log_cleanup_rejects_hard_link_and_preserves_external_log() {
        let root = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let log = external.path().join("app.log");
        std::fs::write(&log, b"external log").unwrap();
        std::fs::create_dir(root.path().join("logs")).unwrap();
        std::fs::hard_link(&log, root.path().join("logs/app.log")).unwrap();
        assert!(cleanup_local_logs_with_root(root.path()).is_err());
        assert_eq!(std::fs::read(&log).unwrap(), b"external log");
    }

    #[test]
    fn retention_rejects_non_utf8_without_rewriting_log() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("logs")).unwrap();
        let log = root.path().join("logs/app.log");
        std::fs::write(&log, [0xff, 0xfe]).unwrap();
        assert!(trim_logs_by_retention(root.path(), Utc::now(), 1, 1).is_err());
        assert_eq!(std::fs::read(&log).unwrap(), [0xff, 0xfe]);
    }

    fn seeded_root(preferences: crate::modules::preferences::Preferences) -> tempfile::TempDir {
        let root = tempfile::tempdir().expect("temp root");
        crate::modules::data_root::initialize(root.path()).expect("initialize");
        crate::modules::preferences::PreferencesStore::new(root.path())
            .save(&preferences)
            .expect("save preferences");
        root
    }

    #[test]
    fn parses_retention_policies_and_falls_back() {
        assert_eq!(parse_log_retention("30d-10000"), (30, 10_000));
        assert_eq!(parse_log_retention("7d-5000"), (7, 5_000));
        assert_eq!(
            parse_log_retention("nonsense"),
            (DEFAULT_LOG_RETENTION_DAYS, DEFAULT_LOG_RETENTION_LINES)
        );
        assert_eq!(parse_notification_retention("7"), 7);
        assert_eq!(
            parse_notification_retention("soon"),
            DEFAULT_NOTIFICATION_RETENTION_DAYS
        );
        assert_eq!(parse_backup_retention("5"), 5);
        assert_eq!(parse_backup_retention("many"), DEFAULT_BACKUP_RETENTION);
    }

    // 回归：「启动时清理」是真实消费的开关，关闭时不产生任何副作用。
    #[test]
    fn startup_cleanup_is_skipped_when_disabled() {
        let preferences = crate::modules::preferences::Preferences {
            startup_cleanup: false,
            ..Default::default()
        };
        let root = seeded_root(preferences);
        let logs = root.path().join("logs");
        std::fs::create_dir_all(&logs).expect("logs");
        std::fs::write(logs.join("app.log"), "old line\n".repeat(50)).expect("write log");
        let outcome = run_startup_cleanup(root.path(), Utc::now()).expect("cleanup");
        assert!(!outcome.ran);
        assert_eq!(outcome.trimmed_log_files, 0);
        assert_eq!(
            std::fs::read_to_string(logs.join("app.log"))
                .expect("log")
                .lines()
                .count(),
            50
        );
    }

    #[test]
    fn registered_startup_disabled_is_silent_and_corrupt_history_is_failure() {
        use crate::modules::notifications::registry::{self, Channel, Trigger};
        let disabled = seeded_root(crate::modules::preferences::Preferences {
            startup_cleanup: false,
            ..Default::default()
        });
        let (outcome, identity) = run_startup_cleanup_registered(disabled.path(), Utc::now());
        assert!(!outcome.unwrap().ran);
        assert!(identity.is_none());
        assert!(!disabled
            .path()
            .join("manager-state/event-scope.json")
            .exists());

        let enabled = seeded_root(crate::modules::preferences::Preferences {
            startup_cleanup: true,
            ..Default::default()
        });
        let history = enabled.path().join("manager-state/notifications.json");
        std::fs::write(&history, b"{broken").unwrap();
        let (outcome, identity) = run_startup_cleanup_registered(enabled.path(), Utc::now());
        assert!(outcome.is_err());
        assert_eq!(std::fs::read(&history).unwrap(), b"{broken");
        let original = identity.unwrap();
        assert_eq!(original.channel, Channel::Local);
        let delivery =
            |event: &'static str, identity: registry::EventIdentity, evidence| registry::Delivery {
                event,
                job: registry::Job::Cleanup,
                trigger: Trigger::Startup,
                identity,
                evidence,
                occurred_at: Utc::now(),
            };
        let mut store = crate::modules::notifications::NotificationStore::new();
        registry::deliver(
            &mut store,
            &delivery(
                "local-cleanup-failed",
                original.clone(),
                registry::Evidence::Failure,
            ),
            registry::NotificationPreferences { lifecycle: true },
        )
        .unwrap();
        // A later successful cleanup uses the exact original attempt, not a new scope.
        std::fs::remove_file(history).unwrap();
        let (outcome, success) = run_startup_cleanup_registered(enabled.path(), Utc::now());
        assert!(outcome.unwrap().ran);
        assert_eq!(success.as_ref(), Some(&original));
        registry::deliver(
            &mut store,
            &delivery(
                "local-cleanup-succeeded",
                success.unwrap(),
                registry::Evidence::Success {
                    candidate: original.candidate.clone(),
                    verified: true,
                },
            ),
            registry::NotificationPreferences { lifecycle: true },
        )
        .unwrap();
        assert!(
            store
                .all()
                .iter()
                .find(|n| n.notification_id.starts_with("local-cleanup-failed:"))
                .unwrap()
                .resolved
        );
    }

    #[test]
    fn registered_startup_policy_error_is_not_silent() {
        let root = seeded_root(Default::default());
        let path = root.path().join("manager-state/preferences.json");
        std::fs::write(&path, b"{broken").unwrap();
        let (outcome, identity) = run_startup_cleanup_registered(root.path(), Utc::now());
        assert!(outcome.is_err());
        assert!(identity.is_some());
        assert_eq!(std::fs::read(path).unwrap(), b"{broken");
    }

    #[test]
    fn startup_cleanup_trims_logs_by_line_budget_and_keeps_audit_log() {
        let preferences = crate::modules::preferences::Preferences {
            log_retention: "7d-5000".to_string(),
            notification_retention: "30".to_string(),
            startup_cleanup: true,
            cleanup_backup_summary: false,
            ..Default::default()
        };
        let root = seeded_root(preferences);
        let logs = root.path().join("logs");
        std::fs::create_dir_all(&logs).expect("logs");
        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        let line = format!("{timestamp} [manager] event\n");
        std::fs::write(logs.join("app.log"), line.repeat(5003)).expect("write app log");
        std::fs::write(logs.join("audit.log"), line.repeat(5003)).expect("write audit log");

        let outcome = run_startup_cleanup(root.path(), Utc::now()).expect("cleanup");
        assert!(outcome.ran);
        assert_eq!(outcome.trimmed_log_files, 1);
        assert_eq!(outcome.trimmed_log_lines, 3);
        assert_eq!(
            std::fs::read_to_string(logs.join("app.log"))
                .expect("app log")
                .lines()
                .count(),
            5000
        );
        // 审计日志是安全凭据，不参与自动截断。
        assert_eq!(
            std::fs::read_to_string(logs.join("audit.log"))
                .expect("audit log")
                .lines()
                .count(),
            5003
        );
    }

    #[test]
    fn startup_cleanup_drops_lines_older_than_retention_window() {
        let preferences = crate::modules::preferences::Preferences {
            log_retention: "7d-5000".to_string(),
            startup_cleanup: true,
            cleanup_backup_summary: false,
            ..Default::default()
        };
        let root = seeded_root(preferences);
        let logs = root.path().join("logs");
        std::fs::create_dir_all(&logs).expect("logs");
        let now = Utc::now();
        let fresh = format!(
            "{} [manager] fresh\n",
            now.to_rfc3339_opts(SecondsFormat::Secs, true)
        );
        let stale = format!(
            "{} [manager] stale\n",
            (now - chrono::Duration::days(30)).to_rfc3339_opts(SecondsFormat::Secs, true)
        );
        std::fs::write(logs.join("app.log"), format!("{stale}{fresh}")).expect("write log");
        let outcome = run_startup_cleanup(root.path(), now).expect("cleanup");
        assert_eq!(outcome.trimmed_log_lines, 1);
        let remaining = std::fs::read_to_string(logs.join("app.log")).expect("log");
        assert!(remaining.contains("fresh"));
        assert!(!remaining.contains("stale"));
    }

    #[test]
    fn startup_cleanup_prunes_read_notifications_past_retention() {
        use crate::modules::notifications::persistence::{notifications_path, save_notifications};
        use crate::modules::notifications::{
            Notification, NotificationAction, NotificationCategory, NotificationLevel,
            NotificationSource, NotificationStore,
        };
        let preferences = crate::modules::preferences::Preferences {
            notification_retention: "7".to_string(),
            log_retention: "90d-30000".to_string(),
            startup_cleanup: true,
            cleanup_backup_summary: false,
            ..Default::default()
        };
        let root = seeded_root(preferences);
        let now = Utc::now();
        let old = (now - chrono::Duration::days(30)).to_rfc3339_opts(SecondsFormat::Secs, true);
        let recent = now.to_rfc3339_opts(SecondsFormat::Secs, true);
        let make = |id: &str, created: &str, read: bool| {
            let mut item = Notification::new(
                id,
                NotificationLevel::Info,
                NotificationCategory::System,
                NotificationSource::Diagnostic,
                "标题",
                "正文",
                created,
                Some(NotificationAction::Logs),
            )
            .expect("notification");
            item.read = read;
            item
        };
        let store = NotificationStore::with_items([
            make("old-read", &old, true),
            make("old-unread", &old, false),
            make("recent-read", &recent, true),
        ]);
        save_notifications(&notifications_path(root.path()), &store).expect("save notifications");

        let outcome = run_startup_cleanup(root.path(), now).expect("cleanup");
        assert_eq!(outcome.pruned_notifications, 1);
        let reloaded = crate::modules::notifications::persistence::load_notifications(
            &notifications_path(root.path()),
        )
        .expect("reload");
        let live: Vec<&str> = reloaded
            .live()
            .iter()
            .map(|item| item.notification_id.as_str())
            .collect();
        assert_eq!(live, vec!["old-unread", "recent-read"]);
    }

    #[test]
    fn startup_keeps_backups_and_explicit_cleanup_honors_retention() {
        use crate::modules::backup::{self, BackupAction};
        use crate::types::upgrade::BackupCleanupPolicy;
        let preferences = crate::modules::preferences::Preferences {
            backup_retention: "5".to_string(),
            log_retention: "90d-30000".to_string(),
            startup_cleanup: true,
            cleanup_backup_summary: true,
            ..Default::default()
        };
        let root = seeded_root(preferences);
        let now = Utc::now();
        for index in 0..8 {
            let created = now - chrono::Duration::days(60 + index);
            // Legacy transaction records must remain protected. Exercise explicit
            // retention with identifiable standalone manager preferences backups.
            backup::manager::create(root.path(), created, BackupCleanupPolicy::Manual)
                .expect("backup");
        }
        let outcome = run_startup_cleanup(root.path(), now).expect("cleanup");
        assert_eq!(outcome.removed_backups, 0);
        assert!(outcome.summary_path.is_none());
        let records = backup::list_action_records(
            &root.path().join("backups"),
            BackupAction::ManualPreferences.as_str(),
        )
        .unwrap();
        assert_eq!(records.len(), 8);
        let prefs = crate::modules::preferences::PreferencesStore::new(root.path())
            .load()
            .unwrap();
        let outcome = run_retention_cleanup(root.path(), &prefs, now).expect("explicit cleanup");
        assert_eq!(outcome.removed_backups, 3);
        let summary = outcome.summary_path.expect("summary path");
        let raw = std::fs::read_to_string(&summary).expect("read summary");
        assert!(raw.contains("removed_backups"));
        assert!(!raw.contains("payload"));
        let records = backup::list_action_records(
            &root.path().join("backups"),
            BackupAction::ManualPreferences.as_str(),
        )
        .expect("list");
        assert_eq!(records.len(), 5);
    }

    #[test]
    fn startup_does_not_read_malformed_backup_manifests() {
        let prefs = crate::modules::preferences::Preferences {
            startup_cleanup: true,
            ..Default::default()
        };
        let root = seeded_root(prefs);
        let backup = root.path().join("backups/2026/01/upgrade/bk_broken");
        std::fs::create_dir_all(&backup).unwrap();
        let manifest = backup.join(crate::modules::backup::MANIFEST_NAME);
        std::fs::write(&manifest, b"{broken").unwrap();
        assert!(run_startup_cleanup(root.path(), Utc::now()).is_ok());
        assert_eq!(std::fs::read(manifest).unwrap(), b"{broken");
    }
}
