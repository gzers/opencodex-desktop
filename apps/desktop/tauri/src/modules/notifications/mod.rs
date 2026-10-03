//! MOD-11：通知领域契约与聚合规则。
//!
//! 后端只维护同一份通知实体；界面中心与诊断中心必须消费同一份状态，
//! 不允许各建副本。通知正文不得携带原始敏感值。

pub mod persistence;

use serde::{Deserialize, Serialize};

/// FZ-43 冻结通知级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationLevel {
    Info,
    Warning,
    Danger,
}

/// 通知分类；原型定义全部 / 运行 / 同步 / 更新 / 系统 五类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationCategory {
    Run,
    Sync,
    Update,
    System,
}

/// 触发来源（《领域模型》§16）：与 `level`、`category` 是互相独立的维度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationSource {
    UserAction,
    Runtime,
    Sync,
    Update,
    Diagnostic,
}

/// 可执行动作的跳转目标；无动作时为 `None`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationAction {
    Restore,
    Upgrade,
    AppUpdate,
    Sync,
    Logs,
    SettingsInstallation,
    SettingsCleanup,
}

/// 领域实体：一条通知。
///
/// `read` / `resolved` / `deleted` 是互相独立的维度：已读不等于已解决，
/// 删除是软删除，保留历史判定依据。删除与过期清理资格互不等价。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notification {
    pub notification_id: String,
    pub level: NotificationLevel,
    pub category: NotificationCategory,
    pub source: NotificationSource,
    pub title: String,
    pub body: String,
    pub created_at: String,
    pub read: bool,
    pub resolved: bool,
    pub resolved_at: Option<String>,
    pub deleted: bool,
    pub expires_at: Option<String>,
    pub operation_id: Option<String>,
    pub dedupe_key: Option<String>,
    pub action_ref: Option<NotificationAction>,
}

impl Notification {
    /// 构造前校验必填字段；正文包含原始敏感赋值时拒绝创建。
    ///
    /// 形参逐一对应《领域模型》§16 的必填字段：身份、三个独立维度
    /// （`level` / `category` / `source`）、标题、正文、创建时间与动作目标。
    /// 时间与身份由调用方提供，领域层不臆测时钟也不生成 id，故不再合并参数。
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        notification_id: impl Into<String>,
        level: NotificationLevel,
        category: NotificationCategory,
        source: NotificationSource,
        title: impl Into<String>,
        body: impl Into<String>,
        created_at: impl Into<String>,
        action_ref: Option<NotificationAction>,
    ) -> Result<Self, &'static str> {
        let notification = Self {
            notification_id: notification_id.into(),
            level,
            category,
            source,
            title: title.into(),
            body: body.into(),
            created_at: created_at.into(),
            read: false,
            resolved: false,
            resolved_at: None,
            deleted: false,
            expires_at: None,
            operation_id: None,
            dedupe_key: None,
            action_ref,
        };
        notification.validate()?;
        Ok(notification)
    }

    /// 关联同一长任务的操作标识（用于通知与日志互跳、终态去重）。
    pub fn with_operation_id(mut self, operation_id: impl Into<String>) -> Self {
        self.operation_id = Some(operation_id.into());
        self
    }

    /// 去重键：同一键在存活期内只保留一条，重复事件只更新该条。
    pub fn with_dedupe_key(mut self, dedupe_key: impl Into<String>) -> Self {
        self.dedupe_key = Some(dedupe_key.into());
        self
    }

    /// 构造期显式声明已读（用于基线数据，不触发业务副作用）。
    pub fn with_read(mut self, read: bool) -> Self {
        self.read = read;
        self
    }

    /// 构造期显式声明已解决并写入解决时间。
    pub fn with_resolved(mut self, resolved_at: impl Into<String>) -> Self {
        self.resolved = true;
        self.resolved_at = Some(resolved_at.into());
        self
    }

    /// 可过期条件的时间（《数据与状态》§5.4）。
    ///
    /// `None` 表示不可自动过期；未解决风险、进行中任务、需要用户决策的事项
    /// 一律保持 `None`，过期资格也不等于删除。
    pub fn with_expires_at(mut self, expires_at: Option<impl Into<String>>) -> Self {
        self.expires_at = expires_at.map(Into::into);
        self
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.notification_id.trim().is_empty() {
            return Err("notification id is required");
        }
        if self.title.trim().is_empty() {
            return Err("notification title is required");
        }
        if self.body.trim().is_empty() {
            return Err("notification body is required");
        }
        if self.created_at.trim().is_empty() {
            return Err("notification created_at is required");
        }
        if contains_raw_secret_assignment(&self.body) {
            return Err("notification body must not contain raw secrets");
        }
        Ok(())
    }
}

/// 聚合视图：通知中心与诊断中心共用的派生结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationAggregate {
    pub total: usize,
    pub unread: usize,
    pub unresolved: usize,
    pub danger_unread: usize,
}

/// 通知集合；保存与聚合只针对同一份实体。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NotificationStore {
    notifications: Vec<Notification>,
}

impl NotificationStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_items<I>(notifications: I) -> Self
    where
        I: IntoIterator<Item = Notification>,
    {
        Self {
            notifications: notifications.into_iter().collect(),
        }
    }

    /// 底层全部条目（含软删除），仅供内部与历史判定使用；对外投影用 `live()`。
    pub fn all(&self) -> &[Notification] {
        &self.notifications
    }

    /// 存活（未软删除）条目；通知中心与通知历史都只消费这一份投影。
    pub fn live(&self) -> Vec<&Notification> {
        self.notifications
            .iter()
            .filter(|item| !item.deleted)
            .collect()
    }

    fn live_mut(&mut self) -> impl Iterator<Item = &mut Notification> {
        self.notifications.iter_mut().filter(|item| !item.deleted)
    }

    pub fn by_category(&self, category: NotificationCategory) -> Vec<&Notification> {
        self.notifications
            .iter()
            .filter(|item| !item.deleted && item.category == category)
            .collect()
    }

    /// 写入一条通知；命中存活期内的同一 `dedupe_key` 时只更新该条，不新增副本。
    pub fn push(&mut self, notification: Notification) -> bool {
        if let Some(key) = notification.dedupe_key.as_deref() {
            if let Some(existing) = self
                .notifications
                .iter_mut()
                .find(|item| !item.deleted && item.dedupe_key.as_deref() == Some(key))
            {
                let dedupe_key = existing.dedupe_key.clone();
                *existing = notification;
                existing.dedupe_key = dedupe_key;
                return false;
            }
        }
        self.notifications.push(notification);
        true
    }

    pub fn mark_read(&mut self, notification_id: &str) -> bool {
        self.change(notification_id, |notification| notification.read = true)
    }

    pub fn mark_all_read(&mut self) {
        self.live_mut().for_each(|notification| {
            notification.read = true;
        });
    }

    /// 标记已解决；`resolved_at` 由调用方按 UTC 时钟传入，领域层不臆测时间。
    pub fn mark_resolved(&mut self, notification_id: &str, resolved_at: &str) -> bool {
        self.change(notification_id, |notification| {
            notification.resolved = true;
            notification.resolved_at = Some(resolved_at.to_string());
        })
    }

    /// 软删除：保留条目与历史判定依据，不计入存活投影。
    pub fn delete(&mut self, notification_id: &str) -> bool {
        self.change(notification_id, |notification| notification.deleted = true)
    }

    pub fn clear(&mut self) {
        self.notifications
            .iter_mut()
            .filter(|item| !item.deleted)
            .for_each(|notification| notification.deleted = true);
    }

    pub fn clear_read(&mut self) {
        self.notifications
            .iter_mut()
            .filter(|item| !item.deleted && item.read)
            .for_each(|notification| notification.deleted = true);
    }

    /// 清理已解决通知（「清理通知」默认只清已解决）。
    pub fn clear_resolved(&mut self) {
        self.notifications
            .iter_mut()
            .filter(|item| !item.deleted && item.resolved)
            .for_each(|notification| notification.deleted = true);
    }

    pub fn aggregate(&self) -> NotificationAggregate {
        let live = || self.notifications.iter().filter(|item| !item.deleted);
        NotificationAggregate {
            total: live().count(),
            unread: live().filter(|item| !item.read && !item.resolved).count(),
            unresolved: live().filter(|item| !item.resolved).count(),
            danger_unread: live()
                .filter(|item| {
                    !item.read && !item.resolved && item.level == NotificationLevel::Danger
                })
                .count(),
        }
    }

    fn change(&mut self, notification_id: &str, update: impl FnOnce(&mut Notification)) -> bool {
        match self
            .notifications
            .iter_mut()
            .find(|item| item.notification_id == notification_id && !item.deleted)
        {
            Some(notification) => {
                update(notification);
                true
            }
            None => false,
        }
    }
}

/// 只拦截常见 key=value / key: value 形态；正文普通文案不误伤。
fn contains_raw_secret_assignment(body: &str) -> bool {
    let lowered = body.to_ascii_lowercase();
    [
        "api_key=",
        "apikey=",
        "authorization:",
        "bearer ",
        "password=",
        "secret=",
        "token=",
        "cookie=",
    ]
    .iter()
    .any(|marker| lowered.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(level: NotificationLevel, read: bool) -> Notification {
        Notification {
            notification_id: format!("{level:?}"),
            level,
            category: NotificationCategory::System,
            source: NotificationSource::Diagnostic,
            title: "通知".to_string(),
            body: "正文".to_string(),
            created_at: "2026-09-15T00:00:00Z".to_string(),
            read,
            resolved: false,
            resolved_at: None,
            deleted: false,
            expires_at: None,
            operation_id: None,
            dedupe_key: None,
            action_ref: None,
        }
    }

    #[test]
    fn notification_requires_core_fields() {
        assert!(Notification::new(
            "id",
            NotificationLevel::Info,
            NotificationCategory::System,
            NotificationSource::Diagnostic,
            "标题",
            "正文",
            "2026-09-15T00:00:00Z",
            None
        )
        .is_ok());
        assert_eq!(
            Notification::new(
                "",
                NotificationLevel::Info,
                NotificationCategory::System,
                NotificationSource::Diagnostic,
                "标题",
                "正文",
                "time",
                None
            )
            .err(),
            Some("notification id is required")
        );
        assert_eq!(
            Notification::new(
                "id",
                NotificationLevel::Info,
                NotificationCategory::System,
                NotificationSource::Diagnostic,
                " ",
                "正文",
                "time",
                None
            )
            .err(),
            Some("notification title is required")
        );
    }

    #[test]
    fn notification_body_rejects_raw_secret_assignment() {
        let notification = Notification {
            notification_id: "bad".to_string(),
            level: NotificationLevel::Warning,
            category: NotificationCategory::System,
            source: NotificationSource::Diagnostic,
            title: "配置".to_string(),
            body: "apiKey=abcd1234".to_string(),
            created_at: "2026-09-15T00:00:00Z".to_string(),
            read: false,
            resolved: false,
            resolved_at: None,
            deleted: false,
            expires_at: None,
            operation_id: None,
            dedupe_key: None,
            action_ref: None,
        };
        assert_eq!(
            notification.validate(),
            Err("notification body must not contain raw secrets")
        );
    }

    #[test]
    fn source_and_expiry_are_independent_dimensions() {
        let notification = Notification::new(
            "expiring",
            NotificationLevel::Info,
            NotificationCategory::System,
            NotificationSource::Runtime,
            "临时提示",
            "低优先级信息，可过期。",
            "2026-09-15T00:00:00Z",
            None,
        )
        .expect("valid notification");
        // 未显式声明时不可自动过期（《数据与状态》§5.4）。
        assert_eq!(notification.expires_at, None);
        assert_eq!(notification.source, NotificationSource::Runtime);

        let expiring = notification.with_expires_at(Some("2026-09-16T00:00:00Z"));
        assert_eq!(expiring.expires_at.as_deref(), Some("2026-09-16T00:00:00Z"));
        // 过期资格不等于删除。
        assert!(!expiring.deleted);

        assert_eq!(
            serde_json::to_string(&NotificationSource::UserAction).unwrap(),
            "\"user_action\""
        );
    }

    #[test]
    fn levels_match_fz43() {
        let levels = [
            NotificationLevel::Info,
            NotificationLevel::Warning,
            NotificationLevel::Danger,
        ];
        assert_eq!(levels.len(), 3);
        let serialized = serde_json::to_string(&NotificationLevel::Danger).unwrap();
        assert_eq!(serialized, "\"danger\"");
    }

    #[test]
    fn aggregation_counts_unread_and_danger() {
        let mut store = NotificationStore::new();
        store.push(sample(NotificationLevel::Info, false));
        store.push(sample(NotificationLevel::Warning, false));
        store.push(sample(NotificationLevel::Danger, true));
        assert_eq!(
            store.aggregate(),
            NotificationAggregate {
                total: 3,
                unread: 2,
                unresolved: 3,
                danger_unread: 0
            }
        );

        store.push(sample(NotificationLevel::Danger, false));
        assert_eq!(store.aggregate().danger_unread, 1);
    }

    #[test]
    fn single_delete_updates_same_store() {
        let mut store = NotificationStore::with_items([
            sample(NotificationLevel::Info, false),
            sample(NotificationLevel::Warning, false),
        ]);
        let first_id = store.all()[0].notification_id.clone();
        assert!(store.mark_read(&first_id));
        assert!(store.delete(&first_id));
        assert_eq!(store.live().len(), 1);
        assert_eq!(store.aggregate().total, 1);
    }

    #[test]
    fn batch_operations_only_change_requested_subset() {
        let mut store = NotificationStore::with_items([
            sample(NotificationLevel::Info, false),
            sample(NotificationLevel::Warning, true),
            sample(NotificationLevel::Danger, true),
        ]);
        store.mark_all_read();
        assert_eq!(store.aggregate().unread, 0);
        store.clear_read();
        assert_eq!(store.live().len(), 0);
    }

    #[test]
    fn notification_category_by_category_filters_correctly() {
        let mut store = NotificationStore::new();
        store.push(sample(NotificationLevel::Info, false));
        store.push(Notification {
            category: NotificationCategory::Sync,
            ..sample(NotificationLevel::Warning, false)
        });
        store.push(Notification {
            category: NotificationCategory::Run,
            ..sample(NotificationLevel::Danger, false)
        });
        assert_eq!(store.by_category(NotificationCategory::System).len(), 1);
        assert_eq!(store.by_category(NotificationCategory::Sync).len(), 1);
        assert_eq!(store.by_category(NotificationCategory::Run).len(), 1);
    }

    #[test]
    fn action_refs_use_frozen_targets() {
        let action = serde_json::to_string(&NotificationAction::AppUpdate).unwrap();
        assert_eq!(action, "\"app_update\"");
    }

    #[test]
    fn read_and_resolved_are_independent_dimensions() {
        let mut store = NotificationStore::new();
        store.push(sample(NotificationLevel::Warning, false));
        let id = store.live()[0].notification_id.clone();

        assert!(store.mark_read(&id));
        // 已读不等于已解决：未读归零，但未解决数与存活数不变。
        assert_eq!(store.aggregate().unread, 0);
        assert_eq!(store.aggregate().unresolved, 1);
        assert!(!store.live()[0].resolved);

        assert!(store.mark_resolved(&id, "2026-09-19T00:00:00Z"));
        let item = store.live()[0];
        assert!(item.resolved);
        assert_eq!(item.resolved_at.as_deref(), Some("2026-09-19T00:00:00Z"));
        // 已解决才使未解决数归零。
        assert_eq!(store.aggregate().unresolved, 0);
    }

    #[test]
    fn delete_is_soft_and_preserves_history() {
        let mut store = NotificationStore::new();
        store.push(sample(NotificationLevel::Info, false));
        let id = store.live()[0].notification_id.clone();
        assert!(store.delete(&id));
        // 存活投影剔除，但底层仍保留，删除不可重复作用于同一存活条目。
        assert_eq!(store.live().len(), 0);
        assert_eq!(store.all().len(), 1);
        assert!(!store.delete(&id));
    }

    #[test]
    fn duplicate_dedupe_key_updates_instead_of_appending() {
        let mut store = NotificationStore::new();
        let first = sample(NotificationLevel::Info, false).with_dedupe_key("sync:ok");
        assert!(store.push(first));
        let second = Notification {
            body: "更新后的正文".to_string(),
            ..sample(NotificationLevel::Info, false)
        }
        .with_dedupe_key("sync:ok");
        assert!(!store.push(second));
        assert_eq!(store.live().len(), 1);
        assert_eq!(store.live()[0].body, "更新后的正文");
    }

    #[test]
    fn clear_resolved_only_touches_resolved_items() {
        let mut store = NotificationStore::new();
        store.push(sample(NotificationLevel::Info, false));
        store.push(sample(NotificationLevel::Warning, false));
        let keep = store.live()[0].notification_id.clone();
        let resolve = store.live()[1].notification_id.clone();
        assert!(store.mark_resolved(&resolve, "2026-09-19T00:00:00Z"));
        store.clear_resolved();
        assert_eq!(store.live().len(), 1);
        assert_eq!(store.live()[0].notification_id, keep);
    }
}
