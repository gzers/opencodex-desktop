//! MOD-11 通知持久化：把同一份通知实体读写到显式数据根的
//! `manager-state/notifications.json`（《数据与状态》§5.7）。
//!
//! 读取缺失时返回**空集**——正式运行不预置任何演示种子；损坏或版本不受支持时
//! **显式失败**，不静默丢弃用户历史。保存使用共享原子写原语替换目标并保持 `0600`。
//! 只读写管理器自己的数据根分区，不访问官方配置、Keychain、WebDAV 或扫描用户目录。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::errors::AppError;
use crate::modules::notifications::{Notification, NotificationStore};

/// 通知实体在数据根中的落点（`manager state` 分区）。
pub const NOTIFICATIONS_RELATIVE_PATH: &str = "manager-state/notifications.json";
/// 存储格式版本；字段语义变更时必须显式迁移，不猜测旧内容。
pub const NOTIFICATIONS_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct NotificationsFile {
    version: u32,
    items: Vec<Notification>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationsError {
    NotConfigured,
    Corrupted,
    Io,
}

impl NotificationsError {
    pub fn as_app_error(self) -> AppError {
        match self {
            Self::NotConfigured => AppError::NotConfigured,
            Self::Corrupted => AppError::FileSystem {
                operation: "read or validate notifications store".to_string(),
                detail: "notifications file is corrupted or has an unsupported schema version"
                    .to_string(),
            },
            Self::Io => AppError::FileSystem {
                operation: "read or write notifications store".to_string(),
                detail: "notifications file could not be read or written".to_string(),
            },
        }
    }
}

pub fn notifications_path(data_root: &Path) -> PathBuf {
    data_root.join(NOTIFICATIONS_RELATIVE_PATH)
}

/// 读取通知集合。
///
/// 文件缺失返回空集；损坏或版本不受支持显式失败。`read` / `resolved` / `deleted`
/// 原样保留（软删除条目也要恢复，它们仍是历史判定依据），因此这里用 `all()`
/// 而不是对外投影 `live()`。
pub fn load_notifications(path: &Path) -> Result<NotificationStore, NotificationsError> {
    if !path.is_absolute() {
        return Err(NotificationsError::NotConfigured);
    }
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(NotificationStore::new())
        }
        Err(_) => return Err(NotificationsError::Io),
    };
    let file: NotificationsFile =
        serde_json::from_slice(&bytes).map_err(|_| NotificationsError::Corrupted)?;
    if file.version != NOTIFICATIONS_SCHEMA_VERSION {
        return Err(NotificationsError::Corrupted);
    }
    Ok(NotificationStore::with_items(file.items))
}

/// 原子保存整个通知集合（含软删除条目），保持 `0600`。
pub fn save_notifications(
    path: &Path,
    store: &NotificationStore,
) -> Result<(), NotificationsError> {
    if !path.is_absolute() {
        return Err(NotificationsError::NotConfigured);
    }
    let file = NotificationsFile {
        version: NOTIFICATIONS_SCHEMA_VERSION,
        items: store.all().to_vec(),
    };
    let payload = serde_json::to_vec_pretty(&file).map_err(|_| NotificationsError::Io)?;
    crate::infrastructure::atomic_write::atomic_write(path, &payload, 0o600)
        .map_err(|_| NotificationsError::Io)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::notifications::{
        NotificationAction, NotificationCategory, NotificationLevel, NotificationSource,
    };
    use std::os::unix::fs::PermissionsExt;

    fn mode(path: &Path) -> u32 {
        std::fs::metadata(path)
            .expect("metadata")
            .permissions()
            .mode()
            & 0o777
    }

    fn fixture() -> NotificationStore {
        NotificationStore::with_items([
            Notification::new(
                "keep-unread",
                NotificationLevel::Danger,
                NotificationCategory::Run,
                NotificationSource::Diagnostic,
                "启动失败",
                "进程启动后立即退出。",
                "2026-09-19T01:00:00Z",
                Some(NotificationAction::Logs),
            )
            .expect("fixture notification"),
            Notification::new(
                "keep-resolved",
                NotificationLevel::Info,
                NotificationCategory::Sync,
                NotificationSource::Diagnostic,
                "同步完成",
                "没有发现冲突。",
                "2026-09-19T01:01:00Z",
                Some(NotificationAction::Sync),
            )
            .expect("fixture notification")
            .with_read(true)
            .with_resolved("2026-09-19T01:02:00Z"),
            Notification::new(
                "keep-deleted",
                NotificationLevel::Warning,
                NotificationCategory::System,
                NotificationSource::Diagnostic,
                "外部接管",
                "外部应用托管了官方配置。",
                "2026-09-19T01:03:00Z",
                None,
            )
            .expect("fixture notification"),
        ])
    }

    #[test]
    fn missing_file_loads_as_empty_instead_of_a_seed() {
        let temp = tempfile::tempdir().expect("temp dir");
        let store = load_notifications(&notifications_path(temp.path()))
            .expect("missing file is not an error");
        assert!(store.all().is_empty());
    }

    #[test]
    fn round_trip_preserves_read_resolved_and_deleted() {
        let temp = tempfile::tempdir().expect("temp dir");
        std::fs::create_dir_all(temp.path().join("manager-state")).expect("manager state dir");
        let path = notifications_path(temp.path());

        let mut original = fixture();
        assert!(original.delete("keep-deleted"));
        save_notifications(&path, &original).expect("save");
        assert_eq!(mode(&path), 0o600);

        let restored = load_notifications(&path).expect("reload");
        assert_eq!(restored, original);
        assert_eq!(restored.all().len(), 3);
        assert_eq!(restored.live().len(), 2);
        assert_eq!(restored.aggregate().unread, 1);
        assert_eq!(restored.aggregate().unresolved, 1);
    }

    #[test]
    fn corrupted_or_unknown_version_fails_explicitly() {
        let temp = tempfile::tempdir().expect("temp dir");
        std::fs::create_dir_all(temp.path().join("manager-state")).expect("manager state dir");
        let path = notifications_path(temp.path());

        std::fs::write(&path, b"{ not json").expect("write corrupt");
        assert_eq!(
            load_notifications(&path),
            Err(NotificationsError::Corrupted)
        );

        std::fs::write(&path, br#"{"version":99,"items":[]}"#).expect("write future version");
        assert_eq!(
            load_notifications(&path),
            Err(NotificationsError::Corrupted)
        );
    }

    #[test]
    fn relative_paths_are_refused() {
        let relative = Path::new("manager-state/notifications.json");
        assert_eq!(
            load_notifications(relative),
            Err(NotificationsError::NotConfigured)
        );
        assert_eq!(
            save_notifications(relative, &NotificationStore::new()),
            Err(NotificationsError::NotConfigured)
        );
    }

    #[test]
    fn error_mapping_never_claims_success() {
        assert!(matches!(
            NotificationsError::NotConfigured.as_app_error(),
            AppError::NotConfigured
        ));
        for error in [NotificationsError::Corrupted, NotificationsError::Io] {
            assert!(matches!(error.as_app_error(), AppError::FileSystem { .. }));
        }
    }
}
