//! W3 集中事件目录与投递边界。目录不是调度器，不存储事件原始 payload。
//! triggers / jobs / events / notification policies 独立；planned 永远不可投递。
use super::{
    Notification, NotificationAction, NotificationCategory, NotificationLevel, NotificationSource,
    NotificationStore,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    Active,
    Planned,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trigger {
    User,
    Startup,
    StatusChange,
    Deadline,
    Foreground,
    Online,
    NativeCallback,
    Commit,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Job {
    Start,
    Stop,
    Restart,
    Observe,
    LoadHistory,
    Install,
    Source,
    NotificationMutation,
    Tray,
    Activity,
    Appearance,
    Panel,
    ManagerCheck,
    ManagerInstall,
    PanelCheck,
    Sync,
    SyncConflict,
    Backup,
    Migration,
    PreferencesBackup,
    PreferencesRestore,
    BackupCleanup,
    BackupList,
    BackupPin,
    Cleanup,
    UiFeedback,
    ProcessSignals,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobDefinition {
    pub id: Job,
    pub availability: Availability,
    pub triggers: Vec<Trigger>,
}

macro_rules! dimension { ($name:ident { $($value:ident),+ }) => {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum $name { $($value),+ }
}; }
dimension!(ObjectKind {
    Runtime,
    NotificationHistory,
    Manager,
    Panel,
    Window,
    Tray,
    Sync,
    Backup,
    Configuration
});
dimension!(Action {
    Start,
    Stop,
    Restart,
    Observe,
    Read,
    Install,
    ChangeSource,
    Mutate,
    Dispatch,
    Activate,
    Render,
    Bridge,
    Check,
    Synchronize,
    Backup,
    Migrate,
    Restore,
    Cleanup,
    Pin
});
dimension!(Phase {
    Execution,
    Observation,
    Load,
    Progress,
    Commit,
    Query
});
dimension!(Channel {
    Local,
    Stable,
    Beta,
    Official
});

/// 只允许非 nil UUID；禁止把路径、版本原文或错误充当对象/候选标识。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct OpaqueId(String);
impl TryFrom<String> for OpaqueId {
    type Error = RegistryError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let id = uuid::Uuid::parse_str(&value).map_err(|_| RegistryError::InvalidIdentity)?;
        if id.is_nil() {
            return Err(RegistryError::InvalidIdentity);
        }
        Ok(Self(id.to_string()))
    }
}
impl From<OpaqueId> for String {
    fn from(value: OpaqueId) -> Self {
        value.0
    }
}
impl OpaqueId {
    pub fn parse(value: &str) -> Result<Self, RegistryError> {
        value.to_owned().try_into()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventIdentity {
    pub object: ObjectKind,
    /// 数据根/运行对象的稳定 UUID，由拥有对象的模块提供，不接收路径。
    pub object_id: OpaqueId,
    pub action: Action,
    pub phase: Phase,
    pub channel: Channel,
    /// 候选 UUID；重试是否沿用候选由对象拥有者根据真实事实决定。
    pub candidate: OpaqueId,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fact {
    Failure,
    Risk,
    Success,
    Signal,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyId {
    StartFailure,
    StopFailure,
    RestartFailure,
    MissingRuntime,
    Unreachable,
    AtRisk,
    Takeover,
    HistoryUnreadable,
    ManagerInstallFailure,
    PreferencesBackupFailure,
    PreferencesRestoreFailure,
    BackupCleanupFailure,
    LocalCleanupFailure,
    SyncConflict,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotificationPolicy {
    pub id: PolicyId,
    pub level: NotificationLevel,
    pub category: NotificationCategory,
    pub source: NotificationSource,
    pub title: String,
    pub body: String,
    pub action: NotificationAction,
    pub dedupe: String,
    pub lifecycle_preference: bool,
    /// Same unresolved candidate may remind again only after this persisted window.
    pub cooldown_seconds: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventDefinition {
    pub id: String,
    pub job: Job,
    pub object: ObjectKind,
    pub action: Action,
    pub phase: Phase,
    pub channels: Vec<Channel>,
    pub fact: Fact,
    pub availability: Availability,
    pub policy: Option<PolicyId>,
    /// 成功只消除显式列出的失败，且必须匹配完整 EventIdentity。
    pub resolves: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RegistryError {
    #[error("event is not registered")]
    UnknownEvent,
    #[error("planned event delivery is disabled")]
    Planned,
    #[error("event identity is invalid")]
    InvalidIdentity,
    #[error("event does not belong to this job or trigger")]
    InvalidJob,
    #[error("event fact does not match its registration")]
    InvalidFact,
    #[error("candidate success has not been verified")]
    UnverifiedSuccess,
    #[error("event has no persistent notification policy")]
    NoNotificationPolicy,
    #[error("event timestamp is invalid")]
    InvalidTime,
    #[error("event registry configuration is invalid")]
    InvalidConfig,
    #[error("persistent event scope is unavailable")]
    ScopeUnavailable,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Evidence {
    Failure,
    Risk,
    Signal,
    /// 拥有任务的模块确认候选真实成功后提供；收到请求、读缓存、发现版本不等于成功。
    Success {
        candidate: OpaqueId,
        verified: bool,
    },
}
pub struct Delivery<'a> {
    pub event: &'a str,
    pub job: Job,
    pub trigger: Trigger,
    pub identity: EventIdentity,
    pub evidence: Evidence,
    pub occurred_at: DateTime<Utc>,
}
/// 终态适配器：job 来自配置，身份来自操作开始时捕获的 scope。
/// 成功证据必须由任务拥有者核验；此函数不把检查/请求成功推断为操作成功。
pub fn terminal_delivery<'a>(
    event: &'a str,
    trigger: Trigger,
    identity: EventIdentity,
    evidence: Evidence,
    occurred_at: DateTime<Utc>,
) -> Result<Delivery<'a>, RegistryError> {
    let definition = lookup(event)?;
    if definition.phase != Phase::Execution
        || !matches!(definition.fact, Fact::Failure | Fact::Success)
    {
        return Err(RegistryError::InvalidFact);
    }
    let delivery = Delivery {
        event,
        job: definition.job,
        trigger,
        identity,
        evidence,
        occurred_at,
    };
    validate(&delivery)?;
    Ok(delivery)
}
#[derive(Debug, Clone, Copy)]
pub struct NotificationPreferences {
    pub lifecycle: bool,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DeliveryOutcome {
    /// 新增、去重刷新或解决是否改变同一份 store；广播应依据 changed，而非 added。
    pub changed: bool,
    pub added: bool,
    pub resolved: usize,
}
pub fn lookup(id: &str) -> Result<&'static EventDefinition, RegistryError> {
    let event = registry()?
        .events
        .iter()
        .find(|event| event.id == id)
        .ok_or(RegistryError::UnknownEvent)?;
    if event.availability == Availability::Planned {
        return Err(RegistryError::Planned);
    }
    Ok(event)
}
/// High-frequency signals validate only the embedded catalog. They never create
/// candidate UUIDs, acquire a persistence lock, or mutate notification history.
pub fn validate_signal(
    event: &str,
    job: Job,
    trigger: Trigger,
    channel: Channel,
) -> Result<(), RegistryError> {
    let definition = lookup(event)?;
    let owner = registry()?
        .jobs
        .iter()
        .find(|owner| owner.id == job)
        .ok_or(RegistryError::InvalidJob)?;
    if owner.availability != Availability::Active {
        return Err(RegistryError::Planned);
    }
    if definition.job != job || !owner.triggers.contains(&trigger) {
        return Err(RegistryError::InvalidJob);
    }
    if !definition.channels.contains(&channel) {
        return Err(RegistryError::InvalidIdentity);
    }
    if definition.fact != Fact::Signal
        || definition.policy.is_some()
        || !definition.resolves.is_empty()
    {
        return Err(RegistryError::InvalidFact);
    }
    Ok(())
}
fn policy(id: PolicyId) -> &'static NotificationPolicy {
    registry()
        .expect("validated embedded registry")
        .notification_policies
        .iter()
        .find(|policy| policy.id == id)
        .expect("registered policy")
}
fn validate(delivery: &Delivery<'_>) -> Result<&'static EventDefinition, RegistryError> {
    let event = lookup(delivery.event)?;
    let job = registry()?
        .jobs
        .iter()
        .find(|job| job.id == delivery.job)
        .ok_or(RegistryError::InvalidJob)?;
    if job.availability == Availability::Planned {
        return Err(RegistryError::Planned);
    }
    if event.job != job.id || !job.triggers.contains(&delivery.trigger) {
        return Err(RegistryError::InvalidJob);
    }
    let identity = &delivery.identity;
    if event.object != identity.object
        || event.action != identity.action
        || event.phase != identity.phase
        || !event.channels.contains(&identity.channel)
    {
        return Err(RegistryError::InvalidIdentity);
    }
    let fact = match &delivery.evidence {
        Evidence::Failure => Fact::Failure,
        Evidence::Risk => Fact::Risk,
        Evidence::Signal => Fact::Signal,
        Evidence::Success {
            candidate,
            verified,
        } => {
            if !verified || *candidate != identity.candidate {
                return Err(RegistryError::UnverifiedSuccess);
            }
            Fact::Success
        }
    };
    if fact != event.fact {
        return Err(RegistryError::InvalidFact);
    }
    Ok(event)
}
fn notification(event: &EventDefinition, at: DateTime<Utc>) -> Result<Notification, RegistryError> {
    let policy = policy(event.policy.ok_or(RegistryError::NoNotificationPolicy)?);
    let id = if event.id == "runtime-starting-failed" {
        "run-start-failed"
    } else {
        event.id.as_str()
    };
    Notification::new(
        id,
        policy.level,
        policy.category,
        policy.source,
        &policy.title,
        &policy.body,
        at.to_rfc3339(),
        Some(policy.action),
    )
    .map(|item| item.with_dedupe_key(&policy.dedupe))
    .map_err(|_| RegistryError::InvalidFact)
}
pub fn registered_notification(id: &str, at: DateTime<Utc>) -> Result<Notification, RegistryError> {
    notification(lookup(id)?, at)
}
pub fn lifecycle_gated(id: &str) -> Result<bool, RegistryError> {
    Ok(policy(
        lookup(id)?
            .policy
            .ok_or(RegistryError::NoNotificationPolicy)?,
    )
    .lifecycle_preference)
}
/// 旧入口只采纳注册 ID 与合法时间；丢弃文案、operation_id、dedupe、状态和伪造恢复证据。
/// 缺少作用域的旧通知可记录事实，但不能触发自动恢复。
pub fn canonical_legacy(input: &Notification) -> Result<Notification, RegistryError> {
    let event = lookup(&input.notification_id)?;
    let at = DateTime::parse_from_rfc3339(&input.created_at)
        .map_err(|_| RegistryError::InvalidTime)?
        .with_timezone(&Utc);
    notification(event, at)
}
/// 先完整验证，再修改 store；publisher 在副本上调用，落盘成功才提交内存。
/// Signal / 无策略事件只验证元数据，不保存原始 payload、不生成通知。
pub fn deliver(
    store: &mut NotificationStore,
    delivery: &Delivery<'_>,
    preferences: NotificationPreferences,
) -> Result<DeliveryOutcome, RegistryError> {
    let event = validate(delivery)?;
    let mut outcome = DeliveryOutcome::default();
    if event.fact == Fact::Success {
        let resolved_at = delivery.occurred_at.to_rfc3339();
        for item in store
            .notifications
            .iter_mut()
            .filter(|item| !item.deleted && !item.resolved)
        {
            let Some(failure) = item.event_identity.as_ref() else {
                continue;
            };
            let failure_id = item.notification_id.split(':').next().unwrap_or("");
            if event.resolves.iter().any(|id| id == failure_id)
                && *failure == delivery.identity
                && DateTime::parse_from_rfc3339(&item.created_at)
                    .is_ok_and(|at| delivery.occurred_at >= at.with_timezone(&Utc))
            {
                item.resolved = true;
                item.resolved_at = Some(resolved_at.clone());
                outcome.resolved += 1;
                outcome.changed = true;
            }
        }
    }
    if let Some(policy_id) = event.policy {
        let policy = policy(policy_id);
        if !policy.lifecycle_preference || preferences.lifecycle {
            let mut item = notification(event, delivery.occurred_at)?;
            let scope = format!(
                "{}:{:?}:{:?}:{:?}:{}",
                delivery.identity.object_id.0,
                delivery.identity.action,
                delivery.identity.phase,
                delivery.identity.channel,
                delivery.identity.candidate.0
            );
            item.notification_id = format!("{}:{scope}", event.id);
            item.dedupe_key = Some(format!("{}:{scope}", &policy.dedupe));
            item.event_identity = Some(delivery.identity.clone());
            if let Some(existing) = store
                .notifications
                .iter_mut()
                .filter(|existing| {
                    !existing.deleted
                        && !existing.resolved
                        && existing.dedupe_key == item.dedupe_key
                        && existing.event_identity == item.event_identity
                })
                .max_by_key(|existing| DateTime::parse_from_rfc3339(&existing.created_at).ok())
            {
                let last = existing
                    .last_observed_at
                    .as_ref()
                    .unwrap_or(&existing.created_at);
                if DateTime::parse_from_rfc3339(last)
                    .is_ok_and(|at| delivery.occurred_at <= at.with_timezone(&Utc))
                {
                    return Ok(outcome);
                }
                let count = existing.occurrence_count.saturating_add(1);
                if DateTime::parse_from_rfc3339(&existing.created_at).is_ok_and(|at| {
                    delivery
                        .occurred_at
                        .signed_duration_since(at.with_timezone(&Utc))
                        < chrono::Duration::seconds(i64::from(policy.cooldown_seconds))
                }) {
                    existing.occurrence_count = count;
                    existing.last_observed_at = Some(delivery.occurred_at.to_rfc3339());
                    outcome.changed = true;
                    return Ok(outcome);
                }
                item.occurrence_count = count;
            }
            item.last_observed_at = Some(delivery.occurred_at.to_rfc3339());
            outcome.changed |= !store
                .notifications
                .iter()
                .find(|existing| !existing.deleted && existing.dedupe_key == item.dedupe_key)
                .is_some_and(|existing| *existing == item);
            outcome.added = store.push(item);
        }
    }
    Ok(outcome)
}

/// 有界盘点：正式后端 emit / DOM bridge / 持久通知及调度入口。
/// 排除测试夹具、开发审计 dispatchEvent、Vue 局部组件 emit 和普通日志 append。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Wiring {
    LegacyNotificationGate,
    StartupGate,
    AwaitingAdapter,
    PlannedClosed,
    ScopedLifecycle,
    ScopedTerminal,
    ValidatedSignal,
    InternalOnly,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmissionSite {
    pub event: String,
    pub source: String,
    pub wiring: Wiring,
}

#[derive(Debug, Deserialize)]
enum SupportedEvent {
    #[serde(rename = "manager-install-progress")]
    ManagerInstallProgress,
    #[serde(rename = "run-start-failed")]
    RunStartFailed,
    #[serde(rename = "run-stop-failed")]
    RunStopFailed,
    #[serde(rename = "run-restart-failed")]
    RunRestartFailed,
    #[serde(rename = "run-start-succeeded")]
    RunStartSucceeded,
    #[serde(rename = "run-stop-succeeded")]
    RunStopSucceeded,
    #[serde(rename = "run-restart-succeeded")]
    RunRestartSucceeded,
    #[serde(rename = "runtime-starting-failed")]
    RuntimeStartingFailed,
    #[serde(rename = "runtime-not-found")]
    RuntimeNotFound,
    #[serde(rename = "run-unreachable")]
    RunUnreachable,
    #[serde(rename = "run-at-risk")]
    RunAtRisk,
    #[serde(rename = "external-takeover")]
    ExternalTakeover,
    #[serde(rename = "notifications-store-unreadable")]
    NotificationsStoreUnreadable,
    #[serde(rename = "status-snapshot-changed")]
    StatusSnapshotChanged,
    #[serde(rename = "notifications-changed")]
    NotificationsChanged,
    #[serde(rename = "runtime-install-progress")]
    RuntimeInstallProgress,
    #[serde(rename = "runtime-source-changed")]
    RuntimeSourceChanged,
    #[serde(rename = "tray-requests-available")]
    TrayRequestsAvailable,
    #[serde(rename = "app-foreground-changed")]
    AppForegroundChanged,
    #[serde(rename = "native-window-appearance")]
    NativeWindowAppearance,
    #[serde(rename = "ocxd-panel")]
    OcxdPanel,
    #[serde(rename = "ocxd-panel-state")]
    OcxdPanelState,
    #[serde(rename = "manager-check-succeeded")]
    ManagerCheckSucceeded,
    #[serde(rename = "manager-check-failed")]
    ManagerCheckFailed,
    #[serde(rename = "panel-check-succeeded")]
    PanelCheckSucceeded,
    #[serde(rename = "panel-check-failed")]
    PanelCheckFailed,
    #[serde(rename = "sync-conflict-detected")]
    SyncConflictDetected,
    #[serde(rename = "backup-completed")]
    BackupCompleted,
    #[serde(rename = "migration-completed")]
    MigrationCompleted,
    #[serde(rename = "manager-install-failed")]
    ManagerInstallFailed,
    #[serde(rename = "manager-install-succeeded")]
    ManagerInstallSucceeded,
    #[serde(rename = "manager-install-pending-restart")]
    ManagerInstallPendingRestart,
    #[serde(rename = "preferences-backup-succeeded")]
    PreferencesBackupSucceeded,
    #[serde(rename = "preferences-backup-failed")]
    PreferencesBackupFailed,
    #[serde(rename = "preferences-restore-succeeded")]
    PreferencesRestoreSucceeded,
    #[serde(rename = "preferences-restore-failed")]
    PreferencesRestoreFailed,
    #[serde(rename = "preferences-backup-cleanup-succeeded")]
    PreferencesBackupCleanupSucceeded,
    #[serde(rename = "preferences-backup-cleanup-failed")]
    PreferencesBackupCleanupFailed,
    #[serde(rename = "local-cleanup-succeeded")]
    LocalCleanupSucceeded,
    #[serde(rename = "local-cleanup-failed")]
    LocalCleanupFailed,
    #[serde(rename = "preferences-backup-list")]
    PreferencesBackupList,
    #[serde(rename = "preferences-backup-pin")]
    PreferencesBackupPin,
    #[serde(rename = "preferences-backup-preview")]
    PreferencesBackupPreview,
    #[serde(rename = "preferences-backup-open")]
    PreferencesBackupOpen,
    #[serde(rename = "process-cancel-signal")]
    ProcessCancelSignal,
    #[serde(rename = "process-observation-signal")]
    ProcessObservationSignal,
    #[serde(rename = "process-exit-signal")]
    ProcessExitSignal,
    #[serde(rename = "runtime-stdout-signal")]
    RuntimeStdoutSignal,
    #[serde(rename = "ui-toast")]
    UiToast,
    #[serde(rename = "ui-recent-event")]
    UiRecentEvent,
    #[serde(rename = "ui-fault")]
    UiFault,
    #[serde(rename = "runtime-install-failed")]
    RuntimeInstallFailed,
    #[serde(rename = "runtime-install-succeeded")]
    RuntimeInstallSucceeded,
    #[serde(rename = "preferences-restored")]
    PreferencesRestored,
}

dimension!(PathId {
    UpdateHandoff,
    RuntimeProtection,
    Notifications,
    EventScope,
    EventScopeLock,
    Preferences,
    Backups,
    BackupLock,
    CleanupSummaries,
    RuntimeHistory,
    Logs,
    BackupRecords
});
dimension!(CleanupId {
    Notifications,
    PreferencesBackups,
    Logs,
    EventScope
});
dimension!(CleanupMode {
    ReadAgeSoftDelete,
    ManualOrAfterVerifiedBackup,
    AgeAndLineLimit,
    NeverAutomatic
});
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathDefinition {
    pub id: PathId,
    /// 数据根相对路径；花括号只用于盘点模板，不可直接拼接调用方输入。
    pub relative_path: String,
    pub owner: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CleanupPolicy {
    pub id: CleanupId,
    pub owner: String,
    pub mode: CleanupMode,
    pub preference: Option<String>,
    pub default_days: Option<u32>,
    pub keep_recent: Option<usize>,
    pub preview_ttl_seconds: Option<u32>,
    pub protected: Vec<String>,
}
/// owner 字段标明现有执行者。通知/scope 路径由此消费，其余路径/清理待所属模块接入。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryConfig {
    pub schema_version: u32,
    pub triggers: Vec<Trigger>,
    pub jobs: Vec<JobDefinition>,
    pub events: Vec<EventDefinition>,
    pub notification_policies: Vec<NotificationPolicy>,
    pub emission_sites: Vec<EmissionSite>,
    pub scheduling_sites: Vec<String>,
    pub ui_feedback_sites: Vec<String>,
    pub paths: Vec<PathDefinition>,
    pub cleanup_policies: Vec<CleanupPolicy>,
}
pub const EMBEDDED_CONFIG: &str = include_str!("../../../config/event-registry.json");
static REGISTRY: std::sync::OnceLock<Result<RegistryConfig, RegistryError>> =
    std::sync::OnceLock::new();
pub fn registry() -> Result<&'static RegistryConfig, RegistryError> {
    REGISTRY
        .get_or_init(|| parse_config(EMBEDDED_CONFIG))
        .as_ref()
        .map_err(|error| *error)
}
fn unique<T: Eq + std::hash::Hash>(values: impl IntoIterator<Item = T>) -> bool {
    let mut seen = std::collections::HashSet::new();
    values.into_iter().all(|value| seen.insert(value))
}
fn config_assert(condition: bool) -> Result<(), RegistryError> {
    condition.then_some(()).ok_or(RegistryError::InvalidConfig)
}
/// 无原始解析错误回显。未知 enum、未知字段、未知事件与重复注册均拒绝。
pub fn parse_config(raw: &str) -> Result<RegistryConfig, RegistryError> {
    let config: RegistryConfig =
        serde_json::from_str(raw).map_err(|_| RegistryError::InvalidConfig)?;
    config.validate()?;
    Ok(config)
}
impl RegistryConfig {
    pub fn path(&self, id: PathId) -> &str {
        &self
            .paths
            .iter()
            .find(|path| path.id == id)
            .expect("validated path id")
            .relative_path
    }
    fn validate(&self) -> Result<(), RegistryError> {
        config_assert(
            self.schema_version == 1
                && !self.triggers.is_empty()
                && !self.jobs.is_empty()
                && !self.events.is_empty(),
        )?;
        config_assert(
            unique(self.triggers.iter())
                && unique(self.jobs.iter().map(|j| j.id))
                && unique(self.events.iter().map(|e| &e.id))
                && unique(self.notification_policies.iter().map(|p| p.id))
                && unique(self.paths.iter().map(|p| p.id))
                && unique(self.paths.iter().map(|p| &p.relative_path))
                && unique(self.cleanup_policies.iter().map(|p| p.id)),
        )?;
        for job in &self.jobs {
            config_assert(
                !job.triggers.is_empty()
                    && unique(job.triggers.iter())
                    && job.triggers.iter().all(|t| self.triggers.contains(t)),
            )?;
            if matches!(job.id, Job::Sync | Job::Backup | Job::Migration) {
                config_assert(job.availability == Availability::Planned)?;
            }
            if job.id == Job::SyncConflict {
                config_assert(
                    job.availability == Availability::Active && job.triggers == [Trigger::User],
                )?;
            }
        }
        for event in &self.events {
            serde_json::from_value::<SupportedEvent>(serde_json::Value::String(event.id.clone()))
                .map_err(|_| RegistryError::InvalidConfig)?;
            let job = self
                .jobs
                .iter()
                .find(|j| j.id == event.job)
                .ok_or(RegistryError::InvalidConfig)?;
            config_assert(
                job.availability == event.availability
                    && !event.channels.is_empty()
                    && unique(event.channels.iter())
                    && unique(event.resolves.iter()),
            )?;
            if matches!(
                event.id.as_str(),
                "backup-completed" | "migration-completed"
            ) {
                config_assert(event.availability == Availability::Planned)?;
            }
            if event.availability == Availability::Planned {
                config_assert(event.policy.is_none() && event.resolves.is_empty())?;
            }
            if let Some(id) = event.policy {
                config_assert(
                    matches!(event.fact, Fact::Failure | Fact::Risk)
                        && self.notification_policies.iter().any(|p| p.id == id),
                )?;
            }
            for target in &event.resolves {
                // JSON owns the mapping; only matching execution terminal names may recover.
                // Query/progress/commit successes cannot acquire a recovery capability.
                config_assert(
                    event.phase == Phase::Execution
                        && event
                            .id
                            .strip_suffix("-succeeded")
                            .is_some_and(|stem| target.strip_suffix("-failed") == Some(stem)),
                )?;
                let failure = self
                    .events
                    .iter()
                    .find(|e| e.id == *target)
                    .ok_or(RegistryError::InvalidConfig)?;
                config_assert(
                    event.fact == Fact::Success
                        && failure.fact == Fact::Failure
                        && failure.availability == Availability::Active
                        && failure.policy.is_some()
                        && (event.job, event.object, event.action, event.phase)
                            == (failure.job, failure.object, failure.action, failure.phase)
                        && event.channels == failure.channels,
                )?;
            }
        }
        for p in &self.notification_policies {
            config_assert(
                !p.dedupe.is_empty()
                    && (1..=604_800).contains(&p.cooldown_seconds)
                    && p.dedupe.len() <= 128
                    && p.dedupe
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || "-_:".contains(c)),
            )?;
            Notification::new(
                "validate",
                p.level,
                p.category,
                p.source,
                &p.title,
                &p.body,
                Utc::now().to_rfc3339(),
                Some(p.action),
            )
            .map_err(|_| RegistryError::InvalidConfig)?;
        }
        config_assert(unique(
            self.emission_sites.iter().map(|s| (&s.event, &s.source)),
        ))?;
        for site in &self.emission_sites {
            let event = self
                .events
                .iter()
                .find(|e| e.id == site.event)
                .ok_or(RegistryError::InvalidConfig)?;
            if site.wiring == Wiring::ValidatedSignal {
                config_assert(
                    event.availability == Availability::Active
                        && event.fact == Fact::Signal
                        && event.policy.is_none()
                        && event.resolves.is_empty(),
                )?;
            }
            config_assert(
                !site.source.trim().is_empty()
                    && (site.wiring == Wiring::PlannedClosed)
                        == (event.availability == Availability::Planned),
            )?;
        }
        for path in &self.paths {
            config_assert(
                !path.owner.is_empty()
                    && !path.relative_path.is_empty()
                    && !path.relative_path.contains(['\\', ':'])
                    && path
                        .relative_path
                        .split('/')
                        .all(|part| !part.is_empty() && part != "." && part != ".."),
            )?;
            config_assert(path.relative_path.split('/').all(|part| {
                !part.contains(['{', '}'])
                    || matches!(part, "{year}" | "{month}" | "{action}" | "{backup_id}")
            }))?;
        }
        for id in [
            PathId::UpdateHandoff,
            PathId::RuntimeProtection,
            PathId::Notifications,
            PathId::EventScope,
            PathId::EventScopeLock,
            PathId::Preferences,
            PathId::Backups,
            PathId::BackupLock,
            PathId::CleanupSummaries,
            PathId::RuntimeHistory,
            PathId::Logs,
            PathId::BackupRecords,
        ] {
            config_assert(self.paths.iter().any(|p| p.id == id))?;
        }
        // Scope stays in manager-state; callers cannot relocate the durable UUID or lock.
        for id in [
            PathId::UpdateHandoff,
            PathId::RuntimeProtection,
            PathId::Notifications,
            PathId::EventScope,
            PathId::EventScopeLock,
        ] {
            config_assert(
                self.path(id).starts_with("manager-state/")
                    && self.path(id).split('/').count() == 2
                    && !self.path(id).contains('{'),
            )?;
        }
        config_assert(self.path(PathId::EventScope) != self.path(PathId::EventScopeLock))?;
        for p in &self.cleanup_policies {
            config_assert(
                !p.owner.is_empty()
                    && !p.protected.is_empty()
                    && unique(p.protected.iter())
                    && p.default_days != Some(0)
                    && p.keep_recent != Some(0)
                    && p.preview_ttl_seconds != Some(0),
            )?;
            let (mode, required): (_, &[&str]) = match p.id {
                CleanupId::Notifications => (
                    CleanupMode::ReadAgeSoftDelete,
                    &["unread", "invalid_time", "deleted_history"],
                ),
                CleanupId::PreferencesBackups => (
                    CleanupMode::ManualOrAfterVerifiedBackup,
                    &[
                        "pinned",
                        "transaction",
                        "referenced",
                        "restore_guard",
                        "unknown_metadata",
                        "failed_integrity",
                    ],
                ),
                CleanupId::Logs => (CleanupMode::AgeAndLineLimit, &["outside_logs"]),
                CleanupId::EventScope => (
                    CleanupMode::NeverAutomatic,
                    &["stable_object_id", "current_candidate"],
                ),
            };
            config_assert(
                p.mode == mode
                    && p.protected.len() == required.len()
                    && required
                        .iter()
                        .all(|name| p.protected.iter().any(|value| value == name)),
            )?;
            if p.id == CleanupId::EventScope {
                config_assert(
                    p.preference.is_none()
                        && p.default_days.is_none()
                        && p.keep_recent.is_none()
                        && p.preview_ttl_seconds.is_none(),
                )?;
            }
        }
        for id in [
            CleanupId::Notifications,
            CleanupId::PreferencesBackups,
            CleanupId::Logs,
            CleanupId::EventScope,
        ] {
            config_assert(self.cleanup_policies.iter().any(|p| p.id == id))?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CandidateScope {
    object: ObjectKind,
    action: Action,
    phase: Phase,
    channel: Channel,
    fingerprint: String,
    id: OpaqueId,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScopeFile {
    schema_version: u32,
    object_id: OpaqueId,
    candidates: Vec<CandidateScope>,
    // Retain every old slot: its action/phase cannot be reconstructed from schema 1.
    legacy_candidates: Vec<LegacyCandidateScope>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyCandidateScope {
    object: ObjectKind,
    channel: Channel,
    fingerprint: String,
    id: OpaqueId,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyScopeFile {
    schema_version: u32,
    object_id: OpaqueId,
    candidates: Vec<LegacyCandidateScope>,
}
pub const MAX_CANDIDATE_SCOPES: usize = 128;
const MAX_LEGACY_SCOPES: usize = 36;
fn valid_fingerprint(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}
impl ScopeFile {
    fn valid(&self) -> bool {
        self.schema_version == 2
            && self.candidates.len() <= MAX_CANDIDATE_SCOPES
            && self.legacy_candidates.len() <= MAX_LEGACY_SCOPES
            && unique(
                self.candidates
                    .iter()
                    .map(|c| (c.object, c.action, c.phase, c.channel)),
            )
            && unique(self.legacy_candidates.iter().map(|c| (c.object, c.channel)))
            && self
                .candidates
                .iter()
                .all(|c| valid_fingerprint(&c.fingerprint))
            && self
                .legacy_candidates
                .iter()
                .all(|c| valid_fingerprint(&c.fingerprint))
    }
}
fn new_opaque() -> OpaqueId {
    OpaqueId(uuid::Uuid::new_v4().to_string())
}
fn scope_io<T>(result: std::io::Result<T>) -> Result<T, RegistryError> {
    result.map_err(|_| RegistryError::ScopeUnavailable)
}
fn check_scope_file(path: &std::path::Path) -> Result<(), RegistryError> {
    match std::fs::symlink_metadata(path) {
        Ok(meta) => {
            if !meta.is_file() || meta.file_type().is_symlink() {
                return Err(RegistryError::ScopeUnavailable);
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if meta.nlink() != 1 {
                    return Err(RegistryError::ScopeUnavailable);
                }
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(RegistryError::ScopeUnavailable),
    }
}
/// 一次持久化 root UUID；损坏/链接文件拒绝，不生成替代身份掩盖旧作用域。
fn with_scope<T>(
    root: &std::path::Path,
    task: impl FnOnce(&mut ScopeFile) -> Result<T, RegistryError>,
) -> Result<T, RegistryError> {
    let _storage = crate::infrastructure::storage_writers::global()
        .admit()
        .map_err(|_| RegistryError::ScopeUnavailable)?;
    use std::io::Read;
    if !root.is_absolute() {
        return Err(RegistryError::ScopeUnavailable);
    }
    let config = registry()?;
    let root = scope_io(root.canonicalize())?;
    let parent = root.join("manager-state");
    match std::fs::symlink_metadata(&parent) {
        Ok(meta) if meta.is_dir() && !meta.file_type().is_symlink() => (),
        Ok(_) => return Err(RegistryError::ScopeUnavailable),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            match std::fs::create_dir(&parent) {
                Ok(()) => (),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => (),
                Err(_) => return Err(RegistryError::ScopeUnavailable),
            }
            let meta = scope_io(std::fs::symlink_metadata(&parent))?;
            if !meta.is_dir() || meta.file_type().is_symlink() {
                return Err(RegistryError::ScopeUnavailable);
            }
        }
        Err(_) => return Err(RegistryError::ScopeUnavailable),
    }
    let path = root.join(config.path(PathId::EventScope));
    let lock_path = root.join(config.path(PathId::EventScopeLock));
    check_scope_file(&lock_path)?;
    let mut options = std::fs::OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    let lock = scope_io(options.open(&lock_path))?;
    scope_io(lock.lock())?;
    check_scope_file(&path)?;
    let mut read = std::fs::OpenOptions::new();
    read.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        read.custom_flags(libc::O_NOFOLLOW);
    }
    let mut migrated = false;
    let mut state = match read.open(&path) {
        Ok(file) => {
            let mut bytes = Vec::new();
            scope_io(file.take(65537).read_to_end(&mut bytes))?;
            if bytes.len() > 65536 {
                return Err(RegistryError::ScopeUnavailable);
            }
            let value = if let Ok(legacy) = serde_json::from_slice::<LegacyScopeFile>(&bytes) {
                if legacy.schema_version != 1 {
                    return Err(RegistryError::ScopeUnavailable);
                }
                migrated = true;
                ScopeFile {
                    schema_version: 2,
                    object_id: legacy.object_id,
                    candidates: vec![],
                    legacy_candidates: legacy.candidates,
                }
            } else {
                serde_json::from_slice::<ScopeFile>(&bytes)
                    .map_err(|_| RegistryError::ScopeUnavailable)?
            };
            if !value.valid() {
                return Err(RegistryError::ScopeUnavailable);
            }
            value
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => ScopeFile {
            schema_version: 2,
            object_id: new_opaque(),
            candidates: vec![],
            legacy_candidates: vec![],
        },
        Err(_) => return Err(RegistryError::ScopeUnavailable),
    };
    let previous = serde_json::to_vec(&state).map_err(|_| RegistryError::ScopeUnavailable)?;
    let result = task(&mut state)?;
    if !state.valid() {
        return Err(RegistryError::ScopeUnavailable);
    }
    let bytes = serde_json::to_vec(&state).map_err(|_| RegistryError::ScopeUnavailable)?;
    if bytes.len() > 65536 {
        return Err(RegistryError::ScopeUnavailable);
    }
    if migrated || bytes != previous || !path.exists() {
        crate::infrastructure::atomic_write::atomic_write(&path, &bytes, 0o600)
            .map_err(|_| RegistryError::ScopeUnavailable)?;
    }
    Ok(result)
}
pub fn data_root_object_id(root: &std::path::Path) -> Result<OpaqueId, RegistryError> {
    with_scope(root, |state| Ok(state.object_id.clone()))
}
/// 跨边界 API：拥有者提供候选事实摘要（SHA-256），存储仅 UUID 和摘要。
/// 对象/动作/阶段/通道独立；同分区候选改变会旋转，返回旧摘要不能恢复旧 UUID。
pub fn candidate_identity(
    root: &std::path::Path,
    object: ObjectKind,
    action: Action,
    phase: Phase,
    channel: Channel,
    candidate_fingerprint: [u8; 32],
) -> Result<EventIdentity, RegistryError> {
    let fingerprint = candidate_fingerprint
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    with_scope(root, |state| {
        let candidate = if let Some(entry) = state.candidates.iter_mut().find(|c| {
            c.object == object && c.action == action && c.phase == phase && c.channel == channel
        }) {
            if entry.fingerprint != fingerprint {
                entry.fingerprint = fingerprint;
                entry.id = new_opaque();
            }
            entry.id.clone()
        } else {
            if state.candidates.len() == MAX_CANDIDATE_SCOPES {
                return Err(RegistryError::ScopeUnavailable);
            }
            let id = state
                .legacy_candidates
                .iter()
                .find(|c| {
                    c.object == object && c.channel == channel && c.fingerprint == fingerprint
                })
                .map(|c| c.id.clone())
                .unwrap_or_else(new_opaque);
            state.candidates.push(CandidateScope {
                object,
                action,
                phase,
                channel,
                fingerprint,
                id: id.clone(),
            });
            id
        };
        Ok(EventIdentity {
            object,
            object_id: state.object_id.clone(),
            action,
            phase,
            channel,
            candidate,
        })
    })
}
/// Read an existing candidate slot without replacing it. A stale handoff must
/// never displace a newer installation attempt during startup reconciliation.
pub fn matches_candidate(
    root: &std::path::Path,
    identity: &EventIdentity,
    candidate_fingerprint: [u8; 32],
) -> Result<bool, RegistryError> {
    let config = registry()?;
    if !root.join(config.path(PathId::EventScope)).exists() {
        return Ok(false);
    }
    let fingerprint = candidate_fingerprint
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    with_scope(root, |state| {
        Ok(state.object_id == identity.object_id
            && state.candidates.iter().any(|c| {
                c.object == identity.object
                    && c.action == identity.action
                    && c.phase == identity.phase
                    && c.channel == identity.channel
                    && c.id == identity.candidate
                    && c.fingerprint == fingerprint
            }))
    })
}
/// 包内容 + 执行位置/工作目录/OPENCODEX_HOME 的摘要；路径从不进入持久文件。
pub fn lifecycle_identity(
    root: &std::path::Path,
    executable: &std::path::Path,
    working_directory: &std::path::Path,
    opencodex_home: &std::path::Path,
    action: Action,
) -> Result<EventIdentity, RegistryError> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut hash = Sha256::new();
    hash.update(b"ocxd-lifecycle-candidate-v1");
    for path in [executable, working_directory, opencodex_home] {
        let path = match path.canonicalize() {
            Ok(path) => path,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && path.is_absolute() => {
                path.to_path_buf()
            }
            Err(_) => return Err(RegistryError::ScopeUnavailable),
        };
        let bytes = path.as_os_str().as_encoded_bytes();
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    match std::fs::File::open(executable) {
        Ok(mut file) => {
            if !scope_io(file.metadata())?.is_file() {
                return Err(RegistryError::ScopeUnavailable);
            }
            hash.update(b"present");
            let mut buffer = [0u8; 65536];
            loop {
                let n = scope_io(file.read(&mut buffer))?;
                if n == 0 {
                    break;
                }
                hash.update(&buffer[..n]);
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => hash.update(b"missing"),
        Err(_) => return Err(RegistryError::ScopeUnavailable),
    }
    candidate_identity(
        root,
        ObjectKind::Runtime,
        action,
        Phase::Execution,
        Channel::Local,
        hash.finalize().into(),
    )
}
