//! MOD-03：官方状态输出采集、字段映射与三维状态维护。
//!
//! 每次完整或轻量采样都产生当前观测或明确失败。只有当前观测能授权进程动作。
//! 采样失败时保留独立的连接、操作与管理器路径，并按刷新策略退避。

use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::status::{ConnectionState, OperationState, RuntimeState, StatusMatrix};

/// FZ-08 冻结的常态、后台与安全阈值（H-08）：数值来自固化运行策略，消费者不再各写常量。
pub fn run_interval() -> Duration {
    crate::modules::runtime_defaults::status_foreground()
}
pub fn connection_interval() -> Duration {
    crate::modules::runtime_defaults::status_connection_foreground()
}
pub fn background_run_interval() -> Duration {
    crate::modules::runtime_defaults::status_background()
}
pub fn background_connection_interval() -> Duration {
    crate::modules::runtime_defaults::status_connection_background()
}
pub fn refresh_debounce() -> Duration {
    crate::modules::runtime_defaults::status_debounce()
}
pub fn collect_timeout() -> Duration {
    crate::modules::runtime_defaults::status_sample_timeout()
}
pub fn max_backoff() -> Duration {
    crate::modules::runtime_defaults::status_max_backoff()
}
#[cfg(test)]
mod context_paths_tests;
pub mod polling;
#[cfg(test)]
mod sample_deadline_tests;

pub fn max_emissions_per_second() -> u32 {
    crate::modules::runtime_defaults::status_max_events_per_second()
}

/// FZ-07 健康值；无法解析时固定为 unknown。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeFacts {
    pub health: HealthState,
    pub runtime_label: Option<String>,
    pub opencodex_home: Option<PathBuf>,
    pub data_root: PathBuf,
    pub protection: Option<String>,
    pub reboot_safe: Option<bool>,
    pub service_present: Option<bool>,
    pub shim_present: Option<bool>,
    pub version_drift: Option<String>,
    pub startup_status: Option<String>,
}

impl Default for RuntimeFacts {
    fn default() -> Self {
        Self {
            health: HealthState::Unknown,
            runtime_label: None,
            opencodex_home: None,
            data_root: PathBuf::new(),
            protection: None,
            reboot_safe: None,
            service_present: None,
            shim_present: None,
            version_drift: None,
            startup_status: None,
        }
    }
}

/// 字段缺失、类型不符合或命令失败时的降级原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DowngradeReason {
    InvalidStartupFacts,
    MissingDataRoot,
}

/// 状态源读取结果；ManagedHomeIdle 是受管 HOME 的正向闲置观测。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectError {
    Timeout,
    Unreachable,
    Parse,
    Unmapped,
    /// The managed HOME has no endpoint record, so no process can be its
    /// running service.
    ManagedHomeIdle,
}

/// 官方采集器边界。真实实现必须在 5 秒内返回或转换为 Timeout。
pub trait StatusSource {
    /// Native lifecycle/recovery sources need one fresh observation window even
    /// during debounce. Defaults preserve legacy fixtures and Mac proof behavior.
    fn requires_fresh_sample(&self) -> bool {
        false
    }
    fn fetch_at(&self, _deadline: Instant) -> Result<Value, CollectError> {
        self.fetch()
    }
    fn fetch_light_at(&self, _deadline: Instant) -> Result<Value, CollectError> {
        self.fetch_light()
    }
    /// Resolved Desktop launch context, distinct from official config-file
    /// paths. Payload fields and directory-name guesses cannot supply it.
    fn manager_paths(&self) -> Option<(PathBuf, PathBuf)> {
        None
    }
    fn fetch(&self) -> Result<Value, CollectError>;
    fn fetch_light(&self) -> Result<Value, CollectError> {
        self.fetch()
    }

    /// 当前运行来源是否已解析到可用入口。
    ///
    /// 默认视为已解析（虚拟 / 夹具来源）。真实来源在**未解析出 `ocx`** 或
    /// **解析到的入口已不存在**时返回 `false`，采集器据此把状态落到 `not_found`，
    /// 而不是沿用上一次「未运行」状态。
    fn resolved(&self) -> bool {
        true
    }
    /// Invalid/incomplete admission is not evidence that installation is absent.
    fn source_unknown(&self) -> bool {
        false
    }
}

/// 官方输出的宽松读取视图；未知 JSON 形状不会导致整个报告被丢弃。
#[derive(Debug, Clone)]
pub struct OfficialStatusReport {
    payload: Value,
}

/// 单个字段的三态读取结果。
#[derive(Debug, Clone, PartialEq)]
enum Field<T> {
    Missing,
    Unknown,
    Present(T),
}

impl<T> Field<T> {
    fn optional(self) -> Result<Option<T>, CollectError> {
        match self {
            Self::Present(value) => Ok(Some(value)),
            Self::Missing => Ok(None),
            Self::Unknown => Err(CollectError::Parse),
        }
    }
}

impl OfficialStatusReport {
    pub fn from_value(payload: Value) -> Self {
        Self { payload }
    }

    fn value(&self, names: &[&str]) -> Option<&Value> {
        names.iter().find_map(|name| self.payload.get(*name))
    }

    fn string(&self, names: &[&str]) -> Field<String> {
        match self.value(names) {
            None => Field::Missing,
            Some(Value::String(value)) => Field::Present(value.clone()),
            Some(_) => Field::Unknown,
        }
    }

    fn boolean(&self, names: &[&str]) -> Field<bool> {
        match self.value(names) {
            None => Field::Missing,
            Some(Value::Bool(value)) => Field::Present(*value),
            Some(_) => Field::Unknown,
        }
    }

    fn nested_object(&self, name: &str) -> Option<&Value> {
        self.payload.get(name).filter(|value| value.is_object())
    }

    fn nested_string(&self, object: Option<&Value>, names: &[&str]) -> Field<String> {
        let Some(object) = object else {
            return Field::Missing;
        };
        match names.iter().find_map(|name| object.get(*name)) {
            None => Field::Missing,
            Some(Value::String(value)) => Field::Present(value.clone()),
            Some(_) => Field::Unknown,
        }
    }

    fn nested_boolean(&self, object: Option<&Value>, names: &[&str]) -> Field<bool> {
        let Some(object) = object else {
            return Field::Missing;
        };
        match names.iter().find_map(|name| object.get(*name)) {
            None => Field::Missing,
            Some(Value::Bool(value)) => Field::Present(*value),
            Some(_) => Field::Unknown,
        }
    }

    fn path(&self, names: &[&str]) -> Option<PathBuf> {
        if let Field::Present(value) = self.string(names) {
            if !value.trim().is_empty() {
                return Some(PathBuf::from(value));
            }
        }
        for name in names {
            let nested = self.nested_object(name);
            if nested.is_some() {
                if let Field::Present(value) = self.nested_string(nested, &["path", "config"]) {
                    if !value.trim().is_empty() {
                        return Some(PathBuf::from(value));
                    }
                }
            }
        }
        None
    }

    fn port(&self) -> Option<u16> {
        match self.value(&["port", "listen"]) {
            Some(Value::Number(value)) => value.as_u64().and_then(|raw| u16::try_from(raw).ok()),
            Some(Value::String(value)) => value.trim().parse().ok(),
            Some(Value::Object(_)) => {
                let nested = self.value(&["listen"]).filter(|value| value.is_object());
                match self.nested_number(nested, &["port"]) {
                    Field::Present(value) => Some(value),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn nested_number(&self, object: Option<&Value>, names: &[&str]) -> Field<u16> {
        let Some(object) = object else {
            return Field::Missing;
        };
        match names.iter().find_map(|name| object.get(*name)) {
            None => Field::Missing,
            Some(Value::Number(value)) => {
                let raw = value.as_u64().and_then(|value| u16::try_from(value).ok());
                match raw {
                    Some(value) => Field::Present(value),
                    None => Field::Unknown,
                }
            }
            Some(_) => Field::Unknown,
        }
    }

    fn pid(&self) -> Option<String> {
        match self.value(&["pid", "processId", "process_id"])? {
            Value::String(value) if !value.trim().is_empty() => Some(value.clone()),
            value => value
                .as_u64()
                .filter(|pid| *pid > 0)
                .map(|pid| pid.to_string()),
        }
    }

    fn proxy_pid(&self) -> Option<String> {
        let proxy = self.nested_object("proxy");
        proxy
            .and_then(|object| object.get("pid"))
            .and_then(Value::as_u64)
            .filter(|pid| *pid > 0)
            .map(|value| value.to_string())
    }

    fn startup_facts(&self) -> (Option<String>, Option<bool>, Option<bool>, Option<bool>) {
        let startup = self.nested_object("startup");
        let protection = match self.nested_string(startup, &["protection"]) {
            Field::Present(value) if !value.trim().is_empty() => Some(value),
            _ => None,
        };
        let reboot_safe = match self.nested_boolean(startup, &["rebootSafe", "reboot_safe"]) {
            Field::Present(value) => Some(value),
            _ => None,
        };
        let service_present =
            self.nested_presence(startup, &["servicePresent", "service_present"], "service");
        let shim_present = self.nested_presence(startup, &["shimPresent", "shim_present"], "shim");
        (protection, reboot_safe, service_present, shim_present)
    }

    /// 支持官方输出常见嵌套：`service.present` / `shim.present`。
    fn nested_path(&self, name: &str, names: &[&str]) -> Option<PathBuf> {
        let object = self.nested_object(name);
        match self.nested_string(object, names) {
            Field::Present(value) if !value.trim().is_empty() => Some(PathBuf::from(value)),
            _ => None,
        }
    }

    fn nested_presence(
        &self,
        startup: Option<&Value>,
        names: &[&str],
        nested_name: &str,
    ) -> Option<bool> {
        if let Field::Present(value) = self.nested_boolean(startup, names) {
            return Some(value);
        }
        let nested = startup.and_then(|object| object.get(nested_name));
        let nested = nested.filter(|value| value.is_object());
        match self.nested_boolean(nested, &["present", "installed"]) {
            Field::Present(value) => Some(value),
            _ => None,
        }
    }
}

/// FZ-07 映射结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappedOfficialStatus {
    pub runtime: RuntimeState,
    pub health: HealthState,
    pub connection: ConnectionState,
    pub operation: OperationState,
    pub facts: RuntimeFacts,
    pub port: Option<u16>,
    pub pid: Option<String>,
    pub downgrade_reasons: Vec<DowngradeReason>,
    idle_observed: bool,
}

/// 官方扁平状态与嵌套 proxy.running 共用同一运行观测。runtime 对象只描述运行来源。
pub fn map_official_output(
    report: &OfficialStatusReport,
    previous_connection: ConnectionState,
    previous_operation: OperationState,
) -> Result<MappedOfficialStatus, CollectError> {
    if !report.payload.is_object() {
        return Err(CollectError::Parse);
    }
    let status = match report.string(&["status", "state"]) {
        Field::Missing if report.nested_object("runtime").is_some() => None,
        Field::Missing => report.string(&["runtime"]).optional()?,
        field => field.optional()?,
    };
    let proxy = report.nested_object("proxy");
    if report.payload.get("proxy").is_some() && proxy.is_none() {
        return Err(CollectError::Parse);
    }
    let proxy_running = report.nested_boolean(proxy, &["running"]).optional()?;
    let ready = report.boolean(&["ready", "isReady"]).optional()?;
    let process_present = report
        .boolean(&["processPresent", "process_present"])
        .optional()?;
    let startup = report.nested_object("startup");
    if report.payload.get("startup").is_some() && startup.is_none() {
        return Err(CollectError::Parse);
    }
    let startup_status = report.nested_string(startup, &["status"]).optional()?;
    let mut operation = previous_operation;
    let runtime = match status.as_deref().map(normalized).as_deref() {
        None => match proxy_running {
            Some(true) => RuntimeState::Running,
            Some(false) => match startup_status.as_deref().map(normalized).as_deref() {
                Some("at-risk") => RuntimeState::AtRisk,
                Some("external-takeover") => RuntimeState::ExternalTakeover,
                Some("not-found") => RuntimeState::NotFound,
                _ => RuntimeState::Stopped,
            },
            None => return Err(CollectError::Unmapped),
        },
        Some("running") => match ready {
            Some(true) => RuntimeState::Running,
            Some(false) => {
                let reachable = report
                    .boolean(&["portReachable", "port_reachable"])
                    .optional()?;
                if reachable == Some(false) && process_present == Some(true) {
                    RuntimeState::Unreachable
                } else {
                    RuntimeState::Pending
                }
            }
            None => return Err(CollectError::Unmapped),
        },
        Some("loading" | "starting") => RuntimeState::Starting,
        Some("pending") => RuntimeState::Pending,
        Some("unreachable") => RuntimeState::Unreachable,
        Some("stopped") => RuntimeState::Stopped,
        Some("stop-confirmed" | "stopped-confirmed") => {
            operation = OperationState::Succeeded;
            RuntimeState::Stopped
        }
        Some("at-risk" | "atrisk") => RuntimeState::AtRisk,
        Some("external-takeover" | "externaltakeover") => RuntimeState::ExternalTakeover,
        Some("not-found" | "notfound") => RuntimeState::NotFound,
        Some("stop-timeout" | "stopping-timeout") => {
            operation = OperationState::Failed;
            RuntimeState::Stopping
        }
        Some(_) => return Err(CollectError::Unmapped),
    };
    let idle_observed = runtime == RuntimeState::Stopped
        || proxy_running == Some(false)
        || (runtime == RuntimeState::AtRisk && process_present == Some(false));
    if (idle_observed && [ready, process_present, proxy_running].contains(&Some(true)))
        || (matches!(
            runtime,
            RuntimeState::Running | RuntimeState::Pending | RuntimeState::Unreachable
        ) && proxy_running == Some(false))
        || (runtime == RuntimeState::Running && process_present == Some(false))
    {
        return Err(CollectError::Parse);
    }
    let health = match report.string(&["health"]).optional()? {
        Some(value) if normalized(&value) == "unknown" => HealthState::Unknown,
        _ => match runtime {
            RuntimeState::Running => HealthState::Healthy,
            RuntimeState::AtRisk | RuntimeState::ExternalTakeover | RuntimeState::Pending => {
                HealthState::Degraded
            }
            RuntimeState::Unreachable => HealthState::Unhealthy,
            _ => HealthState::Unknown,
        },
    };

    let connection = match report.string(&["connection"]) {
        Field::Present(value) => match normalized(&value).as_str() {
            "conflict" => ConnectionState::Conflict,
            "syncing" => ConnectionState::Syncing,
            "connecting" => ConnectionState::Connecting,
            "failed" => ConnectionState::Failed,
            "disconnected" => ConnectionState::Disconnected,
            "unconfigured" => ConnectionState::Unconfigured,
            "synced" => ConnectionState::Synced,
            _ => previous_connection,
        },
        _ => previous_connection,
    };

    if let Some(value) = report.string(&["operation"]).optional()? {
        operation = match normalized(&value).as_str() {
            "rolling-back" | "rolling_back" => OperationState::RollingBack,
            "applying" => OperationState::Applying,
            "backing-up" | "backing_up" => OperationState::BackingUp,
            "validating" => OperationState::Validating,
            "failed" => OperationState::Failed,
            "succeeded" => OperationState::Succeeded,
            "cancelled" => OperationState::Cancelled,
            "idle" => OperationState::Idle,
            _ => return Err(CollectError::Unmapped),
        };
    }

    let inactive = idle_observed || runtime == RuntimeState::NotFound;
    let port = if inactive { None } else { report.port() };
    let pid = if inactive {
        None
    } else {
        report.pid().or_else(|| report.proxy_pid())
    };
    let runtime_version = {
        let skew = report.nested_object("versionSkew");
        match report.nested_string(skew, &["cliVersion", "cli_version"]) {
            Field::Present(value) if !value.trim().is_empty() => Some(value),
            _ => None,
        }
    };
    let runtime_label = optional_string(report, &["runtimeLabel", "runtime_label"])
        .or_else(|| {
            let runtime = report.nested_object("runtime");
            match report.nested_string(runtime, &["source"]) {
                Field::Present(value) if !value.trim().is_empty() => Some(value),
                _ => None,
            }
        })
        .or_else(|| runtime_version.clone());
    let version_drift =
        optional_string(report, &["versionDrift", "version_drift"]).or(runtime_version);
    let mut facts = RuntimeFacts {
        health,
        runtime_label,
        opencodex_home: report.path(&["opencodexHome", "opencodex_home", "home"]),
        data_root: report
            .path(&["dataRoot", "data_root"])
            .or_else(|| report.nested_path("paths", &["home", "dataRoot", "data_root"]))
            .unwrap_or_default(),
        protection: None,
        reboot_safe: None,
        service_present: None,
        shim_present: None,
        version_drift,
        startup_status: None,
    };
    let mut reasons = Vec::new();
    if facts.data_root.as_os_str().is_empty() {
        reasons.push(DowngradeReason::MissingDataRoot);
    }
    if startup.is_none() {
        reasons.push(DowngradeReason::InvalidStartupFacts);
    }
    let (protection, reboot_safe, service_present, shim_present) = report.startup_facts();
    facts.protection = protection;
    facts.reboot_safe = reboot_safe;
    facts.service_present = service_present;
    facts.shim_present = shim_present;
    facts.startup_status = startup_status.filter(|value| !value.trim().is_empty());

    Ok(MappedOfficialStatus {
        runtime,
        health,
        connection,
        operation,
        facts,
        port,
        pid,
        downgrade_reasons: reasons,
        idle_observed,
    })
}

fn optional_string(report: &OfficialStatusReport, names: &[&str]) -> Option<String> {
    match report.string(names) {
        Field::Present(value) if !value.trim().is_empty() => Some(value),
        _ => None,
    }
}

fn normalized(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('_', "-")
}

/// FZ-08 刷新策略；时间比较由调用方注入，便于测试和后台调度。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusDimension {
    Runtime,
    Connection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefreshPolicy;

impl RefreshPolicy {
    /// 连续失败指数退避；成功后由调用方把 failure_count 归零。
    pub fn next_interval(
        dimension: StatusDimension,
        failure_count: u32,
        backgrounded: bool,
    ) -> Duration {
        let base = match (dimension, backgrounded) {
            (StatusDimension::Runtime, false) => run_interval(),
            (StatusDimension::Runtime, true) => background_run_interval(),
            (StatusDimension::Connection, false) => connection_interval(),
            (StatusDimension::Connection, true) => background_connection_interval(),
        };
        let mut interval = base;
        let max_backoff = max_backoff();
        for _ in 0..failure_count.min(8) {
            interval = interval.saturating_mul(2);
            if interval >= max_backoff {
                return max_backoff;
            }
        }
        interval.min(max_backoff)
    }

    /// 同一维度 1 秒内只触发一次。
    pub fn is_debounced(last_request: Option<Instant>, now: Instant) -> bool {
        last_request.is_some_and(|at| now.duration_since(at) < refresh_debounce())
    }

    /// 单次采集契约超时；不真实启动进程，只提供冻结阈值。
    pub fn collect_timeout() -> Duration {
        crate::modules::runtime_defaults::status_sample_timeout()
    }
}

/// IPC 同一事件的发送门；窗口内最多两次，不依赖调用方节流。
#[derive(Debug)]
pub struct EmissionGate {
    window_start: Instant,
    emissions: u32,
}

impl Default for EmissionGate {
    fn default() -> Self {
        Self::new()
    }
}

impl EmissionGate {
    pub fn new() -> Self {
        Self {
            window_start: Instant::now(),
            emissions: 0,
        }
    }

    pub fn should_emit(&mut self, now: Instant) -> bool {
        if now.duration_since(self.window_start) >= Duration::from_secs(1) {
            self.window_start = now;
            self.emissions = 0;
        }
        if self.emissions >= max_emissions_per_second() {
            return false;
        }
        self.emissions += 1;
        true
    }
}

/// 三维状态采集器；失败撤回运行事实和动作权限，成功重置退避计数。
#[derive(Debug)]
pub struct StatusCollector<S>
where
    S: StatusSource,
{
    pub source: S,
    matrix: StatusMatrix,
    facts: RuntimeFacts,
    port: Option<u16>,
    pid: Option<String>,
    failure_count: u32,
    last_request: Option<Instant>,
    last_result: Option<Result<MappedOfficialStatus, CollectError>>,
    last_was_light: bool,
}

impl<S> StatusCollector<S>
where
    S: StatusSource,
{
    pub fn new(source: S) -> Self {
        Self {
            source,
            // 首次采集成功前运行态未知：用 `Loading`（与 `unconfigured_snapshot` 一致）。
            // 冷启动时官方 `ocx status` 可能超过采集上限（Windows 实测首轮 6.8 s），
            // 若初值是 `NotFound`，这段瞬时超时会把正在运行的代理报成「未发现」，
            // 面板与启动进度据此误判（S18-02）。来源未解析仍在失败分支落到 `NotFound`。
            matrix: StatusMatrix {
                runtime: RuntimeState::Loading,
                connection: ConnectionState::Unconfigured,
                operation: OperationState::Idle,
            },
            facts: RuntimeFacts::default(),
            port: None,
            pid: None,
            failure_count: 0,
            last_request: None,
            last_result: None,
            last_was_light: false,
        }
    }

    pub fn matrix(&self) -> StatusMatrix {
        self.matrix
    }

    pub fn facts(&self) -> &RuntimeFacts {
        &self.facts
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }

    pub fn pid(&self) -> Option<String> {
        self.pid.clone()
    }

    /// 使用最近一次采样的证据投影动作，三维状态标签本身不能授权。
    pub fn snapshot(&self) -> crate::types::runtime_status::StatusSnapshotDto {
        use crate::types::runtime_status::{StatusFailure, StatusSnapshotDto, StatusSource};
        let observed = self
            .last_result
            .as_ref()
            .and_then(|result| result.as_ref().ok());
        let can_start = observed.is_some_and(|sample| {
            self.matrix
                .can_start_with_idle_evidence(sample.idle_observed)
        });
        let can_stop = observed.is_some() && self.matrix.can_stop();
        StatusSnapshotDto {
            matrix: self.matrix,
            facts: self.facts.clone(),
            port: self.port,
            pid: self.pid.clone(),
            can_start,
            can_stop,
            can_restart: can_stop,
            source: StatusSource::Live,
            status_failure: self
                .last_result
                .as_ref()
                .and_then(|r| r.as_ref().err())
                .and_then(|error| StatusFailure::from_collect_error(*error)),
        }
    }

    pub fn failure_count(&self) -> u32 {
        self.failure_count
    }

    pub fn next_interval(&self, dimension: StatusDimension, backgrounded: bool) -> Duration {
        RefreshPolicy::next_interval(dimension, self.failure_count, backgrounded)
    }

    pub fn request_allowed(&self, now: Instant) -> bool {
        !RefreshPolicy::is_debounced(self.last_request, now)
    }

    pub fn invalidate(&mut self) {
        self.last_request = None;
        self.last_result = None;
    }

    fn refresh_manager_facts(&mut self) {
        if let Some((data_root, home)) = self.source.manager_paths() {
            self.facts.data_root = data_root;
            self.facts.opencodex_home = Some(home);
        }
    }

    pub fn refresh(&mut self) -> Result<MappedOfficialStatus, CollectError> {
        self.refresh_at(Instant::now())
    }

    /// 注入时钟只用于去抖；声明原生实时采样的来源使用真实时钟及全新有界读回。
    pub fn refresh_at(&mut self, now: Instant) -> Result<MappedOfficialStatus, CollectError> {
        // Native admission clock is never the injected debounce timestamp.
        self.refresh_with(now, false)
    }

    pub fn refresh_light_at(&mut self, now: Instant) -> Result<MappedOfficialStatus, CollectError> {
        self.refresh_with(now, true)
    }

    fn refresh_with(
        &mut self,
        now: Instant,
        light: bool,
    ) -> Result<MappedOfficialStatus, CollectError> {
        let deadline = Instant::now() + collect_timeout();
        if !self.source.requires_fresh_sample()
            && !self.request_allowed(now)
            && (light || !self.last_was_light)
            && self.source.resolved()
            && !self.source.source_unknown()
        {
            self.refresh_manager_facts();
            if let Some(result) = &self.last_result {
                return result.clone().map(|mut sample| {
                    sample.facts = self.facts.clone();
                    sample
                });
            }
        }
        self.refresh_with_deadline(now, light, deadline)
    }

    fn refresh_with_deadline(
        &mut self,
        now: Instant,
        light: bool,
        deadline: Instant,
    ) -> Result<MappedOfficialStatus, CollectError> {
        self.last_request = Some(now);
        self.last_was_light = light;
        let fetched = if Instant::now() >= deadline {
            Err(CollectError::Timeout)
        } else if light {
            self.source.fetch_light_at(deadline)
        } else {
            self.source.fetch_at(deadline)
        };
        let result = match fetched {
            Ok(payload) => map_official_output(
                &OfficialStatusReport::from_value(payload),
                self.matrix.connection,
                self.matrix.operation,
            ),
            Err(CollectError::ManagedHomeIdle) => Ok(MappedOfficialStatus {
                runtime: RuntimeState::Stopped,
                health: HealthState::Unknown,
                connection: self.matrix.connection,
                operation: self.matrix.operation,
                facts: RuntimeFacts::default(),
                port: None,
                pid: None,
                downgrade_reasons: vec![],
                idle_observed: true,
            }),
            Err(error) => Err(error),
        };
        let paths = self.source.manager_paths();
        let result = if Instant::now() >= deadline {
            Err(CollectError::Timeout)
        } else {
            result
        };
        match result {
            Ok(mut sample) => {
                if let Some((data_root, home)) = paths {
                    sample.facts.data_root = data_root;
                    sample.facts.opencodex_home = Some(home);
                    sample
                        .downgrade_reasons
                        .retain(|reason| *reason != DowngradeReason::MissingDataRoot);
                }
                self.matrix = StatusMatrix {
                    runtime: sample.runtime,
                    connection: sample.connection,
                    operation: sample.operation,
                };
                self.facts = sample.facts.clone();
                self.port = sample.port;
                self.pid = sample.pid.clone();
                self.failure_count = 0;
                self.last_result = Some(Ok(sample.clone()));
                Ok(sample)
            }
            Err(error) => {
                self.matrix.runtime = if !self.source.source_unknown() && !self.source.resolved() {
                    RuntimeState::NotFound
                } else {
                    RuntimeState::Unreachable
                };
                self.facts = RuntimeFacts::default();
                if let Some((data_root, home)) = paths {
                    self.facts.data_root = data_root;
                    self.facts.opencodex_home = Some(home);
                }
                self.port = None;
                self.pid = None;
                self.failure_count = self.failure_count.saturating_add(1);
                self.last_result = Some(Err(error));
                Err(error)
            }
        }
    }
}

/// 测试专用虚拟采集器；不执行官方命令。
pub struct VirtualStatusSource {
    result: Result<Value, CollectError>,
}

impl VirtualStatusSource {
    pub fn new(result: Result<Value, CollectError>) -> Self {
        Self { result }
    }
}

impl StatusSource for VirtualStatusSource {
    fn fetch(&self) -> Result<Value, CollectError> {
        self.result.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn report(payload: Value) -> OfficialStatusReport {
        OfficialStatusReport::from_value(payload)
    }

    fn official(status: &str, ready: bool) -> OfficialStatusReport {
        report(json!({
            "status": status,
            "ready": ready,
            "dataRoot": "/tmp/opencodex-fixture",
            "startup": {"protection": "none", "rebootSafe": false}
        }))
    }

    #[test]
    fn fz07_maps_the_frozen_eleven_rows() {
        let cases = [
            ("running", true, RuntimeState::Running, HealthState::Healthy),
            (
                "loading",
                false,
                RuntimeState::Starting,
                HealthState::Unknown,
            ),
            (
                "pending",
                false,
                RuntimeState::Pending,
                HealthState::Degraded,
            ),
            (
                "unreachable",
                false,
                RuntimeState::Unreachable,
                HealthState::Unhealthy,
            ),
            (
                "stopped",
                false,
                RuntimeState::Stopped,
                HealthState::Unknown,
            ),
            (
                "at-risk",
                false,
                RuntimeState::AtRisk,
                HealthState::Degraded,
            ),
            (
                "external-takeover",
                false,
                RuntimeState::ExternalTakeover,
                HealthState::Degraded,
            ),
            (
                "not-found",
                false,
                RuntimeState::NotFound,
                HealthState::Unknown,
            ),
            (
                "stop-confirmed",
                false,
                RuntimeState::Stopped,
                HealthState::Unknown,
            ),
            (
                "stop-timeout",
                false,
                RuntimeState::Stopping,
                HealthState::Unknown,
            ),
        ];

        for (status, ready, expected_runtime, expected_health) in cases {
            let mapped = map_official_output(
                &official(status, ready),
                ConnectionState::Synced,
                OperationState::Idle,
            )
            .unwrap();
            assert_eq!(mapped.runtime, expected_runtime, "runtime for {status}");
            assert_eq!(mapped.health, expected_health, "health for {status}");
        }

        let stopped = map_official_output(
            &official("stop-confirmed", false),
            ConnectionState::Synced,
            OperationState::Idle,
        )
        .unwrap();
        assert_eq!(stopped.operation, OperationState::Succeeded);

        let timed_out = map_official_output(
            &official("stop-timeout", false),
            ConnectionState::Synced,
            OperationState::Idle,
        )
        .unwrap();
        assert_eq!(timed_out.operation, OperationState::Failed);
    }

    #[test]
    fn malformed_observations_are_distinct_from_unmapped_official_objects() {
        let cases = [
            (json!(null), CollectError::Parse),
            (json!([]), CollectError::Parse),
            (
                json!({"status": 1, "proxy": {"running": false}}),
                CollectError::Parse,
            ),
            (json!({"runtime": false}), CollectError::Parse),
            (
                json!({"status": "running", "ready": "yes"}),
                CollectError::Parse,
            ),
            (
                json!({"status": "running", "ready": true, "health": 123}),
                CollectError::Parse,
            ),
            (json!({"proxy": []}), CollectError::Parse),
            (json!({"proxy": {"running": "false"}}), CollectError::Parse),
            (
                json!({"proxy": {"running": false}, "startup": {"status": true}}),
                CollectError::Parse,
            ),
            (
                json!({"status": "stopped", "processPresent": true}),
                CollectError::Parse,
            ),
            (
                json!({"status": "stopped", "operation": false}),
                CollectError::Parse,
            ),
            (
                json!({"status": "stopped", "startup": 3}),
                CollectError::Parse,
            ),
            (
                json!({"status": "at-risk", "processPresent": false, "proxy": {"running": true}}),
                CollectError::Parse,
            ),
            (
                json!({"status": "running", "ready": true, "proxy": {"running": false}}),
                CollectError::Parse,
            ),
            (json!({}), CollectError::Unmapped),
            (
                json!({"status": "mystery-output", "proxy": {"running": false}}),
                CollectError::Unmapped,
            ),
            (
                json!({"runtime": {"source": "bundled"}}),
                CollectError::Unmapped,
            ),
            (
                json!({"startup": {"status": "at-risk"}}),
                CollectError::Unmapped,
            ),
            (
                json!({"status": "stopped", "operation": "future-operation"}),
                CollectError::Unmapped,
            ),
            (json!({"status": "running"}), CollectError::Unmapped),
        ];
        for (payload, expected) in cases {
            let result = map_official_output(
                &report(payload.clone()),
                ConnectionState::Synced,
                OperationState::Idle,
            );
            assert_eq!(result.unwrap_err(), expected, "{payload}");
        }
    }

    #[test]
    fn fz08_backoff_doubles_and_caps_at_sixty_seconds() {
        assert_eq!(
            RefreshPolicy::next_interval(StatusDimension::Runtime, 0, false),
            run_interval()
        );
        assert_eq!(
            RefreshPolicy::next_interval(StatusDimension::Runtime, 1, false),
            Duration::from_secs(6)
        );
        assert_eq!(
            RefreshPolicy::next_interval(StatusDimension::Runtime, 2, false),
            Duration::from_secs(12)
        );
        assert_eq!(
            RefreshPolicy::next_interval(StatusDimension::Runtime, 5, false),
            max_backoff()
        );
        assert_eq!(
            RefreshPolicy::next_interval(StatusDimension::Connection, 0, false),
            connection_interval()
        );
        assert_eq!(
            RefreshPolicy::next_interval(StatusDimension::Connection, 1, false),
            Duration::from_secs(30)
        );
    }

    #[test]
    fn fz08_background_intervals_are_frozen() {
        assert_eq!(
            RefreshPolicy::next_interval(StatusDimension::Runtime, 0, true),
            background_run_interval()
        );
        assert_eq!(
            RefreshPolicy::next_interval(StatusDimension::Connection, 0, true),
            background_connection_interval()
        );
    }

    #[test]
    fn fz08_debounces_the_same_dimension_for_one_second() {
        let _collector = StatusCollector::new(VirtualStatusSource::new(Ok(json!({}))));
        let now = Instant::now();
        assert!(!RefreshPolicy::is_debounced(None, now));
        assert!(RefreshPolicy::is_debounced(Some(now), now));
    }

    #[test]
    fn first_timeout_records_uncertainty_and_increments_backoff() {
        let mut collector =
            StatusCollector::new(VirtualStatusSource::new(Err(CollectError::Timeout)));
        let error = collector.refresh().expect_err("timeout");
        assert_eq!(error, CollectError::Timeout);
        assert_eq!(collector.matrix().runtime, RuntimeState::Unreachable);
        assert_eq!(collector.failure_count(), 1);
        assert_eq!(
            collector.next_interval(StatusDimension::Runtime, false),
            Duration::from_secs(6)
        );
    }

    #[test]
    fn official_schema_maps_startup_status_and_facts() {
        let mapped = map_official_output(
            &report(json!({
                "proxy": {"running": false},
                "listen": {"port": 10100},
                "paths": {
                    "config": "/tmp/opencodex-home/config.json",
                    "runtime": "/tmp/bun"
                },
                "runtime": {"source": "bundled"},
                "startup": {"status": "at-risk", "protection": "none", "rebootSafe": false},
                "versionSkew": {"cliVersion": "2.50.0"}
            })),
            ConnectionState::Unconfigured,
            OperationState::Idle,
        )
        .unwrap();

        assert_eq!(mapped.runtime, RuntimeState::AtRisk);
        assert_eq!(mapped.health, HealthState::Degraded);
        assert_eq!(mapped.port, None);
        assert_eq!(mapped.facts.runtime_label.as_deref(), Some("bundled"));
        assert_eq!(mapped.facts.version_drift.as_deref(), Some("2.50.0"));
        // Official paths.config is a file, not an admissible HOME. The real
        // Desktop collector supplies its independently resolved context.
        assert_eq!(mapped.facts.opencodex_home, None);
    }

    #[test]
    fn official_schema_derives_running_and_not_found() {
        let running = map_official_output(
            &report(json!({
                "proxy": {"running": true, "health": {"ok": true}},
                "listen": {"port": 10100},
                "startup": {"status": "not-found"},
                "dataRoot": "/tmp/fixture"
            })),
            ConnectionState::Unconfigured,
            OperationState::Idle,
        )
        .unwrap();
        assert_eq!(running.runtime, RuntimeState::Running);
        assert_eq!(running.health, HealthState::Healthy);

        let not_found = map_official_output(
            &report(json!({
                "proxy": {"running": false},
                "startup": {"status": "not-found"},
                "dataRoot": "/tmp/fixture"
            })),
            ConnectionState::Unconfigured,
            OperationState::Idle,
        )
        .unwrap();
        assert_eq!(not_found.runtime, RuntimeState::NotFound);
        assert_eq!(not_found.health, HealthState::Unknown);
    }

    /// 回归：代理已退出时不得沿用上一次的「运行中/健康」。
    ///
    /// 官方输出为 `proxy.running=false` 且启动状态不是 at-risk/external-takeover/
    /// not-found（实测为 `native`）时，此前会回退到上一次的运行态，界面于是在
    /// 代理被杀掉后仍显示 running/healthy。
    #[test]
    fn stopped_proxy_does_not_keep_stale_running_state() {
        let mapped = map_official_output(
            &report(json!({
                "schemaVersion": 1,
                "proxy": {"running": false, "pid": null, "staleProcessState": true,
                           "health": {"ok": false, "message": "unreachable"}},
                "listen": {"port": 10100},
                "runtime": {"source": "bundled"},
                "startup": {"status": "native", "protection": "none", "rebootSafe": true}
            })),
            ConnectionState::Unconfigured,
            OperationState::Idle,
        )
        .unwrap();
        assert_eq!(
            mapped.runtime,
            RuntimeState::Stopped,
            "proxy.running=false 时必须如实报告未运行"
        );
        assert_ne!(mapped.health, HealthState::Healthy);
    }

    #[test]
    fn official_2_50_running_report_overrides_startup_risk() {
        let mapped = map_official_output(
            &report(json!({
                "schemaVersion": 1,
                "proxy": {"running": true, "pid": 33409, "health": {"ok": true}},
                "listen": {"port": 10100},
                "paths": {"config": "/tmp/config.json"},
                "runtime": {"source": "bundled"},
                "startup": {"status": "at-risk", "protection": "none", "rebootSafe": false},
                "versionSkew": {"cliVersion": "2.50.0"}
            })),
            ConnectionState::Unconfigured,
            OperationState::Idle,
        )
        .unwrap();
        assert_eq!(mapped.runtime, RuntimeState::Running);
        assert_eq!(mapped.health, HealthState::Healthy);
        assert_eq!(mapped.port, Some(10100));
        assert_eq!(mapped.pid.as_deref(), Some("33409"));
        assert_eq!(mapped.facts.startup_status.as_deref(), Some("at-risk"));
    }

    #[test]
    fn at_risk_facts_are_filled_from_startup() {
        let mapped = map_official_output(
            &report(json!({
                "status": "at-risk",
                "dataRoot": "/tmp/fixture",
                "startup": {
                    "protection": "none",
                    "rebootSafe": false,
                    "service": {"present": false},
                    "shim": {"present": false}
                }
            })),
            ConnectionState::Synced,
            OperationState::Idle,
        )
        .unwrap();
        assert_eq!(mapped.runtime, RuntimeState::AtRisk);
        assert_eq!(mapped.facts.protection.as_deref(), Some("none"));
        assert_eq!(mapped.facts.reboot_safe, Some(false));
        assert_eq!(mapped.facts.service_present, Some(false));
        assert_eq!(mapped.facts.shim_present, Some(false));
    }

    #[test]
    fn ipc_gate_allows_at_most_two_emissions_per_second() {
        let mut gate = EmissionGate::new();
        let now = Instant::now();
        assert!(gate.should_emit(now));
        assert!(gate.should_emit(now));
        assert!(!gate.should_emit(now));
        assert!(gate.should_emit(now + Duration::from_secs(1)));
    }

    #[test]
    fn source_timeout_is_a_frozen_contract_value() {
        assert_eq!(RefreshPolicy::collect_timeout(), Duration::from_secs(5));
    }

    /// 回归（TASK-160，真机反馈「卸载后仍允许启动」）：运行来源未解析 ⇒ 未发现安装。
    ///
    /// 卸载（尤其完整卸载移除入口）后来源不再解析，采集必然失败；这属于**确定性**
    /// 安装缺失与普通采样失败分别投影，二者均撤销进程权限。
    #[test]
    fn unresolved_source_falls_back_to_not_found() {
        use std::cell::Cell;

        struct PhasedSource {
            resolved: Cell<bool>,
        }
        impl StatusSource for PhasedSource {
            fn fetch(&self) -> Result<Value, CollectError> {
                if self.resolved.get() {
                    Ok(json!({
                        "status": "stopped",
                        "dataRoot": "/tmp/opencodex-fixture",
                        "startup": {"protection": "none", "rebootSafe": false}
                    }))
                } else {
                    Err(CollectError::Unreachable)
                }
            }
            fn resolved(&self) -> bool {
                self.resolved.get()
            }
        }

        let mut collector = StatusCollector::new(PhasedSource {
            resolved: Cell::new(true),
        });
        let t0 = Instant::now();
        collector
            .refresh_at(t0)
            .expect("resolved source reports stopped");
        assert_eq!(collector.matrix().runtime, RuntimeState::Stopped);
        assert!(collector.snapshot().can_start);

        // 卸载后入口消失，概览和托盘使用相同的未发现快照。
        collector.source.resolved.set(false);
        let error = collector
            .refresh_at(t0 + Duration::from_secs(2))
            .expect_err("unresolved source must fail collection");
        assert_eq!(error, CollectError::Unreachable);
        assert_eq!(collector.matrix().runtime, RuntimeState::NotFound);
        assert_eq!(collector.facts().health, HealthState::Unknown);
        assert_eq!(collector.port(), None);
        assert_eq!(collector.pid(), None);
    }

    #[test]
    fn newly_resolved_source_invalidates_pre_supply_absence_without_inventing_service_success() {
        use std::cell::Cell;
        struct Source {
            resolved: Cell<bool>,
            error: CollectError,
            successful: Cell<bool>,
        }
        impl StatusSource for Source {
            fn fetch(&self) -> Result<Value, CollectError> {
                if self.successful.get() {
                    Ok(
                        json!({"status":"stopped","dataRoot":"/fixture","startup":{"protection":"none","rebootSafe":false}}),
                    )
                } else {
                    Err(self.error)
                }
            }
            fn resolved(&self) -> bool {
                self.resolved.get()
            }
        }
        for error in [
            CollectError::Timeout,
            CollectError::Unreachable,
            CollectError::Parse,
            CollectError::Unmapped,
        ] {
            for light in [false, true] {
                let mut collector = StatusCollector::new(Source {
                    resolved: Cell::new(false),
                    error,
                    successful: Cell::new(false),
                });
                let now = Instant::now();
                assert_eq!(collector.refresh_at(now).unwrap_err(), error);
                assert_eq!(collector.matrix().runtime, RuntimeState::NotFound);
                collector.source.resolved.set(true);
                let result = if light {
                    collector.refresh_light_at(now + Duration::from_secs(2))
                } else {
                    collector.refresh_at(now + Duration::from_secs(2))
                };
                assert_eq!(result.unwrap_err(), error);
                assert_eq!(collector.matrix().runtime, RuntimeState::Unreachable);
                assert_eq!(collector.facts().health, HealthState::Unknown);
                assert_eq!(collector.port(), None);
                assert_eq!(collector.pid(), None);
                collector.source.successful.set(true);
                collector.refresh_at(now + Duration::from_secs(4)).unwrap();
                assert_eq!(collector.matrix().runtime, RuntimeState::Stopped);
                collector.source.successful.set(false);
                collector.source.resolved.set(false);
                assert_eq!(
                    collector
                        .refresh_at(now + Duration::from_secs(6))
                        .unwrap_err(),
                    error
                );
                assert_eq!(collector.matrix().runtime, RuntimeState::NotFound);
            }
        }
    }

    #[test]
    fn source_admission_failure_is_unknown_and_bypasses_cached_success() {
        use std::cell::Cell;
        struct Source {
            unknown: Cell<bool>,
            calls: Cell<u32>,
        }
        impl StatusSource for Source {
            fn fetch(&self) -> Result<Value, CollectError> {
                self.calls.set(self.calls.get() + 1);
                if self.unknown.get() {
                    return Err(CollectError::Unreachable);
                }
                Ok(
                    json!({"status":"running","ready":true,"health":"healthy","pid":"42","port":12345,"dataRoot":"/fixture","startup":{"protection":"none","rebootSafe":false}}),
                )
            }
            fn resolved(&self) -> bool {
                !self.unknown.get()
            }
            fn source_unknown(&self) -> bool {
                self.unknown.get()
            }
        }
        for light in [false, true] {
            let mut collector = StatusCollector::new(Source {
                unknown: Cell::new(false),
                calls: Cell::new(0),
            });
            let now = Instant::now();
            collector.refresh_at(now).unwrap();
            assert_eq!(collector.matrix.runtime, RuntimeState::Running);
            collector.source.unknown.set(true);
            let result = if light {
                collector.refresh_light_at(now + Duration::from_millis(1))
            } else {
                collector.refresh_at(now + Duration::from_millis(1))
            };
            assert_eq!(result.unwrap_err(), CollectError::Unreachable);
            assert_eq!(collector.source.calls.get(), 2);
            assert_eq!(collector.matrix.runtime, RuntimeState::Unreachable);
            assert_eq!(collector.facts.health, HealthState::Unknown);
            assert_eq!(collector.port, None);
            assert_eq!(collector.pid, None);
        }
    }
}
