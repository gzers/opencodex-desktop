//! FZ-43 通知前端 DTO；领域实体、聚合与传输契约分离。

use crate::modules::notifications::{Notification, NotificationAggregate, NotificationCategory};
use crate::types::status::RuntimeState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationAggregateDto {
    pub total: usize,
    pub unread: usize,
    pub unresolved: usize,
    pub danger_unread: usize,
}

impl From<NotificationAggregate> for NotificationAggregateDto {
    fn from(value: NotificationAggregate) -> Self {
        Self {
            total: value.total,
            unread: value.unread,
            unresolved: value.unresolved,
            danger_unread: value.danger_unread,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationDto {
    pub id: String,
    pub kind: crate::modules::notifications::NotificationLevel,
    pub category: NotificationCategory,
    pub source: crate::modules::notifications::NotificationSource,
    pub title: String,
    pub detail: String,
    pub time: String,
    pub read: bool,
    pub resolved: bool,
    pub resolved_at: Option<String>,
    pub expires_at: Option<String>,
    pub operation_id: Option<String>,
    pub dedupe_key: Option<String>,
    pub occurrence_count: u32,
    pub last_observed_at: Option<String>,
    pub action: Option<crate::modules::notifications::NotificationAction>,
    pub target: Option<RuntimeState>,
}

impl NotificationDto {
    pub fn from_domain(value: &Notification) -> Self {
        Self {
            id: value.notification_id.clone(),
            kind: value.level,
            category: value.category,
            source: value.source,
            title: value.title.clone(),
            detail: value.body.clone(),
            time: value.created_at.clone(),
            read: value.read,
            resolved: value.resolved,
            resolved_at: value.resolved_at.clone(),
            expires_at: value.expires_at.clone(),
            operation_id: value.operation_id.clone(),
            dedupe_key: value.dedupe_key.clone(),
            occurrence_count: value.occurrence_count,
            last_observed_at: value.last_observed_at.clone(),
            action: value.action_ref,
            // restore 引导的文案分两种（外部接管 / startup at-risk），而领域实体没有
            // target 字段；按发布方使用的稳定 id 反推，避免把所有 restore 都当成外部接管。
            target: match value.action_ref {
                Some(crate::modules::notifications::NotificationAction::Restore) => Some(
                    match value.notification_id.split(':').next().unwrap_or("") {
                        "external-takeover" => RuntimeState::ExternalTakeover,
                        _ => RuntimeState::AtRisk,
                    },
                ),
                _ => None,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationsDto {
    pub items: Vec<NotificationDto>,
    pub aggregate: NotificationAggregateDto,
}

impl NotificationsDto {
    pub fn from_domain(items: &[Notification], aggregate: NotificationAggregate) -> Self {
        Self {
            items: items.iter().map(NotificationDto::from_domain).collect(),
            aggregate: aggregate.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::notifications::{NotificationAction, NotificationLevel};

    #[test]
    fn notification_dto_uses_camel_case_and_frozen_action_refs() {
        let notification = Notification::new(
            "app-update-available",
            NotificationLevel::Info,
            crate::modules::notifications::NotificationCategory::Update,
            crate::modules::notifications::NotificationSource::Update,
            "桌面管理器有可用更新",
            "当前 v0.1.0，最新 v0.1.1。",
            "2026-09-15T10:16:00Z",
            Some(NotificationAction::AppUpdate),
        )
        .expect("valid notification");
        let payload = serde_json::to_value(NotificationDto::from_domain(&notification)).unwrap();
        assert_eq!(payload["id"], "app-update-available");
        assert_eq!(payload["kind"], "info");
        // 触发来源是《领域模型》§16 的独立维度，必须跨边界下发。
        assert_eq!(payload["source"], "update");
        assert_eq!(payload["expiresAt"], serde_json::Value::Null);
        assert_eq!(payload["time"], "2026-09-15T10:16:00Z");
        assert_eq!(payload["action"], "app_update");
        assert_eq!(payload["dangerUnread"], serde_json::Value::Null);
    }

    #[test]
    fn aggregate_dto_uses_camel_case_counts() {
        let payload = serde_json::to_value(NotificationAggregateDto {
            total: 4,
            unread: 3,
            unresolved: 4,
            danger_unread: 1,
        })
        .unwrap();
        assert_eq!(payload["dangerUnread"], 1);
        assert_eq!(payload["unresolved"], 4);
    }
}
