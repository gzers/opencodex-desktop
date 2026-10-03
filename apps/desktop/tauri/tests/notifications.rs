use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use opencodex_desktop_lib::commands::notifications::{
    clear_notifications_with_store, clear_read_notifications_with_store,
    delete_notification_with_store, list_notifications_with_store,
    mark_all_notifications_read_with_store, mark_notification_read_with_store,
    startup_notification_store,
};
use opencodex_desktop_lib::errors::AppError;
use opencodex_desktop_lib::modules::notifications::persistence::{
    load_notifications, notifications_path,
};
use opencodex_desktop_lib::modules::notifications::{
    Notification, NotificationAction, NotificationCategory, NotificationLevel, NotificationSource,
    NotificationStore,
};

type SharedNotificationStore = Arc<Mutex<NotificationStore>>;

/// 测试夹具：正式运行不预置任何演示种子（《数据与状态》§5.7、《领域模型》§16），
/// 因此夹具只存在于测试代码里，不随产品编译。
fn fixture_items() -> Vec<Notification> {
    vec![
        Notification::new(
            "external-takeover",
            NotificationLevel::Warning,
            NotificationCategory::System,
            NotificationSource::Diagnostic,
            "外部 provider 已接管 Codex 配置",
            "桌面壳只解释影响，不会覆盖外部配置。",
            "2026-09-15T10:24:00Z",
            Some(NotificationAction::Restore),
        )
        .expect("fixture notification"),
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
            "app-update-available",
            NotificationLevel::Info,
            NotificationCategory::Update,
            NotificationSource::Diagnostic,
            "桌面管理器有可用更新",
            "下载校验后重启应用生效，不影响 OpenCodex 代理。",
            "2026-09-15T10:16:00Z",
            Some(NotificationAction::AppUpdate),
        )
        .expect("fixture notification")
        .with_read(true),
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

fn temp_data_root() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temp data root");
    std::fs::create_dir_all(root.path().join("manager-state")).expect("manager state dir");
    root
}

fn fixture_store(items: Vec<Notification>) -> SharedNotificationStore {
    Arc::new(Mutex::new(NotificationStore::with_items(items)))
}

#[test]
fn fixture_contract_keeps_camel_case_aggregate_and_action_mapping() {
    let store = fixture_store(fixture_items());
    let value = list_notifications_with_store(&store).expect("list notifications");
    assert_eq!(value.items.len(), 4);
    assert_eq!(value.items[0].id, "external-takeover");
    assert_eq!(value.items[0].kind, NotificationLevel::Warning);
    assert_eq!(
        value.items[0].target,
        Some(opencodex_desktop_lib::types::status::RuntimeState::ExternalTakeover)
    );
    assert_eq!(value.aggregate.total, 4);
    assert_eq!(value.aggregate.unread, 2);
    assert_eq!(value.aggregate.unresolved, 3);
    assert_eq!(value.aggregate.danger_unread, 1);

    let payload = serde_json::to_value(&value).expect("serialize notification DTO");
    assert_eq!(payload["items"][0]["id"], "external-takeover");
    assert_eq!(payload["items"][0]["action"], "restore");
    assert_eq!(payload["items"][2]["action"], "app_update");
    assert_eq!(payload["aggregate"]["dangerUnread"], 1);
    assert_eq!(payload["items"][0]["resolved"], false);
}

#[test]
fn command_operations_share_one_backend_model_and_persist() {
    let root = temp_data_root();
    let store = fixture_store(fixture_items());
    let first_id = "external-takeover";

    let marked =
        mark_notification_read_with_store(first_id, &store, root.path()).expect("mark one read");
    assert_eq!(marked.aggregate.total, 4);
    assert_eq!(marked.aggregate.unread, 1);
    assert!(marked.items[0].read);

    let all_read =
        mark_all_notifications_read_with_store(&store, root.path()).expect("mark all read");
    assert_eq!(all_read.aggregate.unread, 0);

    let deleted =
        delete_notification_with_store(first_id, &store, root.path()).expect("delete one");
    assert_eq!(deleted.items.len(), 3);
    assert!(!deleted.items.iter().any(|item| item.id == first_id));

    // 重启恢复：从磁盘重新读取时，未解决通知不丢失，软删除条目仍按原样保留。
    let persisted: PathBuf = notifications_path(root.path());
    let restored = load_notifications(&persisted).expect("reload from disk");
    assert_eq!(restored.all().len(), 4);
    assert_eq!(restored.live().len(), 3);
    assert!(restored.live().iter().all(|item| item.read));
    assert!(
        restored
            .all()
            .iter()
            .find(|item| item.notification_id == first_id)
            .expect("soft-deleted item restored")
            .deleted
    );

    let cleared_read =
        clear_read_notifications_with_store(&store, root.path()).expect("clear read");
    assert!(cleared_read.items.is_empty());
    assert_eq!(cleared_read.aggregate.total, 0);

    let cleared_all = clear_notifications_with_store(&store, root.path()).expect("clear all");
    assert!(cleared_all.items.is_empty());
}

#[test]
fn startup_never_seeds_and_uses_the_persisted_file() {
    let root = temp_data_root();

    // 首次启动：没有文件 → 空集，不预置演示数据。
    assert!(startup_notification_store(root.path()).all().is_empty());

    // 有历史文件 → 按 read / resolved / deleted 原样恢复。
    let store = fixture_store(fixture_items());
    mark_notification_read_with_store("start-failed", &store, root.path()).expect("mark read");
    let restarted = startup_notification_store(root.path());
    let live = restarted.live();
    assert_eq!(live.len(), 4);
    assert!(
        live.iter()
            .find(|item| item.notification_id == "start-failed")
            .expect("start-failed restored")
            .read
    );
    assert!(
        live.iter()
            .find(|item| item.notification_id == "sync-connected")
            .expect("resolved item restored")
            .resolved
    );
}

#[test]
fn unknown_id_returns_explicit_not_found_without_mutating_store() {
    let root = temp_data_root();
    let store = fixture_store(fixture_items());
    assert!(matches!(
        delete_notification_with_store("missing", &store, root.path()),
        Err(AppError::NotFound { entity }) if entity == "notification"
    ));
    let unchanged = list_notifications_with_store(&store).expect("list unchanged store");
    assert_eq!(unchanged.items.len(), 4);
    assert_eq!(unchanged.aggregate.unread, 2);
}

#[test]
fn command_contract_does_not_create_raw_secret_payloads() {
    assert!(Notification::new(
        "unsafe",
        NotificationLevel::Warning,
        NotificationCategory::System,
        NotificationSource::Diagnostic,
        "配置同步",
        "token=raw-secret",
        "2026-09-15T10:24:00Z",
        None,
    )
    .is_err());
    let store = fixture_store(fixture_items());
    let value = list_notifications_with_store(&store).expect("fixture notifications");
    let serialized = serde_json::to_string(&value).expect("serialize DTO");
    assert!(!serialized.to_ascii_lowercase().contains("token="));
}
