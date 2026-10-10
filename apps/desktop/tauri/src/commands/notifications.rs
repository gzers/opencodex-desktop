//! Tauri 通知命令层。只加锁、投影和转发领域操作，不访问平台细节。
//!
//! 变更一律「先落盘、再提交内存」：持久化失败时命令显式报错且内存不变，
//! 避免「界面说成功、重启后丢失」的假成功（《数据与状态》§5.7）。

use std::path::Path;

use crate::errors::{AppError, AppResult};
use crate::modules::notifications::persistence::{
    load_notifications, notifications_path, save_notifications,
};
use crate::modules::notifications::registry::{
    self, Delivery, DeliveryOutcome, NotificationPreferences, RegistryError,
};
use crate::modules::notifications::{Notification, NotificationStore};
use crate::state::{SharedDataRoot, SharedNotificationStore};
use crate::types::notifications::NotificationsDto;

/// 后端侧写入通知后广播的事件名。
///
/// 前端在启动时拉取一次列表；后端异步写入（状态观测、同步冲突等）时必须广播，
/// 否则界面会一直停留在启动时那份旧列表——「后端写了但通知中心不显示」。
pub const NOTIFICATIONS_CHANGED_EVENT: &str = "notifications-changed";

impl From<RegistryError> for AppError {
    fn from(value: RegistryError) -> Self {
        AppError::FileSystem {
            operation: "validate registered event delivery".to_owned(),
            // RegistryError 文案是固定分类，绝不回显未知 ID 或调用方 payload。
            detail: value.to_string(),
        }
    }
}

impl From<crate::modules::notifications::persistence::NotificationsError> for AppError {
    fn from(value: crate::modules::notifications::persistence::NotificationsError) -> Self {
        value.as_app_error()
    }
}

/// 启动时的通知集合。
///
/// 文件缺失 → 空集：正式运行不预置任何演示种子（《领域模型》§16、
/// 《数据与状态》§5.7）。文件损坏或不可读 → **不静默丢弃**：以空集启动，
/// 并写入一条**真实**失败通知说明历史无法读取。这不是演示种子，而是
/// 「本次读取失败」这一真实事件的记录，因此允许出现。
pub fn startup_notification_store(data_root: &Path) -> NotificationStore {
    registry::registry().expect("embedded event registry must validate before startup");
    let path = notifications_path(data_root);
    let loaded = crate::modules::backup::safety::check_path(data_root, &path, true)
        .and_then(|()| load_notifications(&path).map_err(AppError::from));
    match loaded {
        Ok(store) => store,
        Err(_) => {
            let mut store = NotificationStore::new();
            if let Ok(notification) = registry::registered_notification(
                "notifications-store-unreadable",
                chrono::Utc::now(),
            ) {
                store.push(notification);
            }
            store
        }
    }
}

fn project(store: &SharedNotificationStore) -> AppResult<NotificationsDto> {
    let guard = store.lock().map_err(|_poisoned| AppError::NotConfigured)?;
    let live = guard.live();
    Ok(NotificationsDto::from_domain(
        &live.into_iter().cloned().collect::<Vec<_>>(),
        guard.aggregate(),
    ))
}

/// 变更并持久化：先把变更应用到副本、落盘成功后，才提交到内存。
///
/// 任一步失败都返回显式错误且**不改内存**，因此不会出现内存与磁盘不一致。
fn mutate_and_persist<T>(
    store: &SharedNotificationStore,
    data_root: &Path,
    operation: impl FnOnce(&mut NotificationStore) -> Result<T, AppError>,
) -> AppResult<T> {
    let mut guard = store.lock().map_err(|_poisoned| AppError::NotConfigured)?;
    let mut next = guard.clone();
    let outcome = operation(&mut next)?;
    if next == *guard {
        return Ok(outcome);
    }
    let path = notifications_path(data_root);
    crate::modules::backup::safety::check_path(data_root, &path, true)?;
    save_notifications(&path, &next)?;
    *guard = next;
    Ok(outcome)
}

/// 运行期通知发布器：把**真实事件**写入同一份 store 并落盘。
///
/// 只负责「写入 + 持久化」；发布失败只影响这条通知，不改变调用方主流程的返回值，
/// 因此调用方必须把发布失败单独记入运行日志，不得把它当成主流程成功或失败。
#[derive(Clone, Copy)]
pub struct NotificationPublisher<'a> {
    pub store: &'a SharedNotificationStore,
    pub data_root: &'a Path,
}

impl NotificationPublisher<'_> {
    pub fn publish(&self, notification: Notification) -> AppResult<()> {
        self.publish_legacy(notification, false).map(|_| ())
    }

    /// Compatibility publishers can broadcast only after a durable change.
    pub fn publish_changed(&self, notification: Notification) -> AppResult<bool> {
        self.publish_legacy(notification, true)
    }

    fn publish_legacy(
        &self,
        notification: Notification,
        suppress_reobservation: bool,
    ) -> AppResult<bool> {
        let notification = registry::canonical_legacy(&notification)?;
        if registry::lifecycle_gated(&notification.notification_id)?
            && !self.preferences().lifecycle
        {
            return Ok(false);
        }
        mutate_and_persist(self.store, self.data_root, |notifications| {
            // Reobserving the same live condition is not a new occurrence. Keep
            // its timestamp and read state; a resolved/deleted condition may recur.
            if suppress_reobservation
                && notifications.all().iter().any(|existing| {
                    !existing.deleted
                        && !existing.resolved
                        && existing.notification_id == notification.notification_id
                        && existing.dedupe_key == notification.dedupe_key
                })
            {
                return Ok(false);
            }
            let before = notifications.clone();
            notifications.push(notification);
            Ok(*notifications != before)
        })
    }

    fn preferences(&self) -> NotificationPreferences {
        NotificationPreferences {
            lifecycle: crate::commands::lifecycle_notifications_enabled(self.data_root),
        }
    }

    /// 跨边界接入点：调用方提供完整作用域和候选真实成功证据；无原始 payload 参数。
    /// 调用方在持久化成功且 outcome.changed 时广播 notifications-changed。
    pub fn publish_event(&self, delivery: &Delivery<'_>) -> AppResult<DeliveryOutcome> {
        let preferences = self.preferences();
        mutate_and_persist(self.store, self.data_root, |store| {
            registry::deliver(store, delivery, preferences).map_err(AppError::from)
        })
    }
}

#[tauri::command]
pub fn list_notifications(
    store: tauri::State<'_, SharedNotificationStore>,
) -> AppResult<NotificationsDto> {
    list_notifications_with_store(store.inner())
}

pub fn list_notifications_with_store(
    store: &SharedNotificationStore,
) -> AppResult<NotificationsDto> {
    project(store)
}

#[tauri::command]
pub fn mark_notification_read(
    id: String,
    store: tauri::State<'_, SharedNotificationStore>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> AppResult<NotificationsDto> {
    mark_notification_read_with_store(&id, store.inner(), &data_root.0)
}

pub fn mark_notification_read_with_store(
    id: &str,
    store: &SharedNotificationStore,
    data_root: &Path,
) -> AppResult<NotificationsDto> {
    mutate_and_persist(store, data_root, |notifications| {
        notifications
            .mark_read(id)
            .then_some(())
            .ok_or(AppError::NotFound {
                entity: "notification".to_string(),
            })
    })?;
    project(store)
}

#[tauri::command]
pub fn mark_all_notifications_read(
    store: tauri::State<'_, SharedNotificationStore>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> AppResult<NotificationsDto> {
    mark_all_notifications_read_with_store(store.inner(), &data_root.0)
}

pub fn mark_all_notifications_read_with_store(
    store: &SharedNotificationStore,
    data_root: &Path,
) -> AppResult<NotificationsDto> {
    mutate_and_persist(store, data_root, |notifications| {
        notifications.mark_all_read();
        Ok(())
    })?;
    project(store)
}

#[tauri::command]
pub fn delete_notification(
    id: String,
    store: tauri::State<'_, SharedNotificationStore>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> AppResult<NotificationsDto> {
    delete_notification_with_store(&id, store.inner(), &data_root.0)
}

pub fn delete_notification_with_store(
    id: &str,
    store: &SharedNotificationStore,
    data_root: &Path,
) -> AppResult<NotificationsDto> {
    mutate_and_persist(store, data_root, |notifications| {
        notifications
            .delete(id)
            .then_some(())
            .ok_or(AppError::NotFound {
                entity: "notification".to_string(),
            })
    })?;
    project(store)
}

#[tauri::command]
pub fn mark_notification_resolved(
    id: String,
    store: tauri::State<'_, SharedNotificationStore>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> AppResult<NotificationsDto> {
    mark_notification_resolved_with_store(&id, store.inner(), &data_root.0)
}

pub fn mark_notification_resolved_with_store(
    id: &str,
    store: &SharedNotificationStore,
    data_root: &Path,
) -> AppResult<NotificationsDto> {
    // 解决时间由命令层按 UTC 时钟生成，领域层不臆测时钟。
    let resolved_at = chrono::Utc::now().to_rfc3339();
    mutate_and_persist(store, data_root, |notifications| {
        notifications
            .mark_resolved(id, &resolved_at)
            .then_some(())
            .ok_or(AppError::NotFound {
                entity: "notification".to_string(),
            })
    })?;
    project(store)
}

#[tauri::command]
pub fn clear_resolved_notifications(
    store: tauri::State<'_, SharedNotificationStore>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> AppResult<NotificationsDto> {
    clear_resolved_notifications_with_store(store.inner(), &data_root.0)
}

pub fn clear_resolved_notifications_with_store(
    store: &SharedNotificationStore,
    data_root: &Path,
) -> AppResult<NotificationsDto> {
    mutate_and_persist(store, data_root, |notifications| {
        notifications.clear_resolved();
        Ok(())
    })?;
    project(store)
}

#[tauri::command]
pub fn clear_notifications(
    store: tauri::State<'_, SharedNotificationStore>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> AppResult<NotificationsDto> {
    clear_notifications_with_store(store.inner(), &data_root.0)
}

pub fn clear_notifications_with_store(
    store: &SharedNotificationStore,
    data_root: &Path,
) -> AppResult<NotificationsDto> {
    mutate_and_persist(store, data_root, |notifications| {
        notifications.clear();
        Ok(())
    })?;
    project(store)
}

#[tauri::command]
pub fn clear_read_notifications(
    store: tauri::State<'_, SharedNotificationStore>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> AppResult<NotificationsDto> {
    clear_read_notifications_with_store(store.inner(), &data_root.0)
}

pub fn clear_read_notifications_with_store(
    store: &SharedNotificationStore,
    data_root: &Path,
) -> AppResult<NotificationsDto> {
    mutate_and_persist(store, data_root, |notifications| {
        notifications.clear_read();
        Ok(())
    })?;
    project(store)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::notifications::{
        NotificationAction, NotificationCategory, NotificationLevel, NotificationSource,
    };
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// 测试夹具：只在测试代码里构造通知。
    ///
    /// 正式运行不预置任何演示种子（《数据与状态》§5.7、《领域模型》§16），
    /// 因此夹具不随产品编译，也不会出现在真实通知中心。
    fn fixture_items() -> Vec<Notification> {
        vec![
            Notification::new(
                "sync-connected",
                NotificationLevel::Info,
                NotificationCategory::Sync,
                NotificationSource::Diagnostic,
                "WebDAV 同步连接正常",
                "最近同步没有发现冲突。",
                "2026-09-15T10:24:00Z",
                Some(NotificationAction::Sync),
            )
            .expect("fixture notification")
            .with_read(true)
            .with_resolved("2026-09-15T10:24:00Z"),
            Notification::new(
                "sync-conflict",
                NotificationLevel::Warning,
                NotificationCategory::Sync,
                NotificationSource::Diagnostic,
                "WebDAV 同步冲突",
                "本地与远程配置存在差异，未处理前不会自动覆盖。",
                "2026-09-15T10:30:00Z",
                Some(NotificationAction::Sync),
            )
            .expect("fixture notification"),
            Notification::new(
                "start-failed",
                NotificationLevel::Danger,
                NotificationCategory::Run,
                NotificationSource::Diagnostic,
                "OpenCodex 启动失败",
                "进程启动后立即退出。",
                "2026-09-15T10:32:00Z",
                Some(NotificationAction::Restore),
            )
            .expect("fixture notification"),
        ]
    }

    fn temp_root() -> TempDir {
        let root = tempfile::tempdir().expect("temp data root");
        std::fs::create_dir_all(root.path().join("manager-state")).expect("manager state dir");
        root
    }

    fn fixture_store(items: Vec<Notification>) -> SharedNotificationStore {
        Arc::new(Mutex::new(NotificationStore::with_items(items)))
    }

    #[test]
    fn production_store_starts_without_demo_seed() {
        let root = temp_root();
        // 缺失文件 → 空集，不预置任何演示数据。
        let store = startup_notification_store(root.path());
        assert!(store.all().is_empty());

        let shared: SharedNotificationStore = Arc::new(Mutex::new(store));
        let value = list_notifications_with_store(&shared).expect("list empty store");
        assert!(value.items.is_empty());
        assert_eq!(value.aggregate.total, 0);
        assert_eq!(value.aggregate.unread, 0);
        assert_eq!(value.aggregate.unresolved, 0);
        assert_eq!(value.aggregate.danger_unread, 0);
    }

    #[test]
    fn unreadable_store_starts_empty_and_reports_a_real_failure() {
        let root = temp_root();
        std::fs::write(
            root.path().join("manager-state/notifications.json"),
            b"{ broken",
        )
        .expect("write corrupt store");

        let store = startup_notification_store(root.path());
        let live = store.live();
        assert_eq!(live.len(), 1);
        assert_eq!(live[0].notification_id, "notifications-store-unreadable");
        // 这是「本次读取失败」的真实事件，不是演示种子。
        assert_eq!(
            live[0].dedupe_key.as_deref(),
            Some("store:notifications-unreadable")
        );
    }

    #[test]
    fn unreadable_history_cannot_be_overwritten_by_a_notification_or_clear() {
        for bytes in [
            b"{broken".as_slice(),
            br#"{"version":99,"items":[]}"#.as_slice(),
        ] {
            let root = temp_root();
            let path = notifications_path(root.path());
            std::fs::write(&path, bytes).unwrap();
            let initial = startup_notification_store(root.path());
            let shared = Arc::new(Mutex::new(initial.clone()));
            let notification =
                registry::registered_notification("run-start-failed", chrono::Utc::now()).unwrap();
            let publisher = NotificationPublisher {
                store: &shared,
                data_root: root.path(),
            };
            assert!(publisher.publish(notification).is_err());
            assert!(mark_all_notifications_read_with_store(&shared, root.path()).is_err());
            assert!(clear_notifications_with_store(&shared, root.path()).is_err());
            assert_eq!(*shared.lock().unwrap(), initial);
            assert_eq!(std::fs::read(path).unwrap(), bytes);
        }
    }

    #[test]
    fn mutations_persist_so_state_survives_restart() {
        let root = temp_root();
        let store = fixture_store(fixture_items());

        mark_notification_read_with_store("sync-conflict", &store, root.path()).expect("mark read");
        mark_notification_resolved_with_store("sync-conflict", &store, root.path())
            .expect("resolve");
        delete_notification_with_store("start-failed", &store, root.path()).expect("delete");

        // 模拟重启：从磁盘重新读取。
        let restored =
            load_notifications(&notifications_path(root.path())).expect("reload from disk");
        let conflict = restored
            .all()
            .iter()
            .find(|item| item.notification_id == "sync-conflict")
            .expect("conflict persisted");
        assert!(conflict.read);
        assert!(conflict.resolved);
        assert!(conflict.resolved_at.is_some());
        let deleted = restored
            .all()
            .iter()
            .find(|item| item.notification_id == "start-failed")
            .expect("soft-deleted item is still restored");
        assert!(deleted.deleted);
        assert_eq!(restored.live().len(), 2);
    }

    #[test]
    fn failed_persist_does_not_report_success_or_change_memory() {
        let root = temp_root();
        let store = fixture_store(fixture_items());
        // 让落点变成一个目录，原子写必然失败。
        std::fs::create_dir_all(root.path().join("manager-state/notifications.json"))
            .expect("blocking directory");

        let result = mark_notification_read_with_store("sync-conflict", &store, root.path());
        assert!(result.is_err(), "写入失败必须显式报错");

        let after = list_notifications_with_store(&store).expect("list");
        let conflict = after
            .items
            .iter()
            .find(|item| item.id == "sync-conflict")
            .expect("conflict still live");
        assert!(!conflict.read, "落盘失败时内存不得被改动");
    }

    #[test]
    fn resolving_and_clearing_only_affect_requested_items() {
        let root = temp_root();
        let store = fixture_store(fixture_items());
        let before = list_notifications_with_store(&store).expect("list fixture");
        assert_eq!(before.aggregate.total, 3);
        assert_eq!(before.aggregate.unresolved, 2);

        let after = mark_notification_resolved_with_store("sync-conflict", &store, root.path())
            .expect("resolve sync conflict");
        let conflict = after
            .items
            .iter()
            .find(|item| item.id == "sync-conflict")
            .expect("conflict still live");
        assert!(conflict.resolved);
        assert!(conflict.resolved_at.is_some());
        assert_eq!(after.aggregate.unresolved, 1);

        let cleared =
            clear_resolved_notifications_with_store(&store, root.path()).expect("clear resolved");
        assert!(cleared.items.iter().all(|item| !item.resolved));
        assert_eq!(cleared.aggregate.unresolved, cleared.aggregate.total);
        assert_eq!(cleared.aggregate.total, 1);
    }

    #[test]
    fn unknown_notification_is_explicit_not_silent() {
        let root = temp_root();
        let store = fixture_store(fixture_items());
        assert!(matches!(
            mark_notification_read_with_store("missing", &store, root.path()),
            Err(AppError::NotFound { entity }) if entity == "notification"
        ));
    }
}
