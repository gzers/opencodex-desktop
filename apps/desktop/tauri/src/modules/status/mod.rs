//! MOD-03：官方状态输出采集、字段映射与三维状态维护。
//!
//! 采集器只接受注入的 [`StatusSource`]；真实官方命令由后续
//! infrastructure 接入。**瞬时**采集失败（超时 / 解析失败）保留上一次状态，
//! 并把失败交给 [`RefreshPolicy`] 决定下一次采样间隔；但**运行来源未解析**
//! （未发现安装）是确定性事实，必须落到 `not_found`，否则卸载后界面会沿用
//! 上一次「未运行」而继续允许启动（FZ-07「未发现安装 → not_found」）。

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
pub mod polling;

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

/// 领域模型 §4 的运行事实。
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
    MissingStatus,
    UnknownStatus,
    InvalidStatus,
    InvalidHealth,
    InvalidReady,
    MissingOfficialFields,
    InvalidStartupFacts,
    MissingDataRoot,
}

/// 采集失败分类；调用方必须保留上一份状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectError {
    Timeout,
    Unreachable,
    Parse,
}

/// 官方采集器边界。真实实现必须在 5 秒内返回或转换为 Timeout。
pub trait StatusSource {
    fn fetch(&self) -> Result<Value, CollectError>;

    /// 当前运行来源是否已解析到可用入口。
    ///
    /// 默认视为已解析（虚拟 / 夹具来源）。真实来源在**未解析出 `ocx`** 或
    /// **解析到的入口已不存在**时返回 `false`，采集器据此把状态落到 `not_found`，
    /// 而不是沿用上一次「未运行」状态。
    fn resolved(&self) -> bool {
        true
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
        match self.string(&["pid", "processId", "process_id"]) {
            Field::Present(value) if !value.trim().is_empty() => Some(value),
            _ => None,
        }
    }

    fn proxy_pid(&self) -> Option<String> {
        let proxy = self.nested_object("proxy");
        proxy
            .and_then(|object| object.get("pid"))
            .and_then(Value::as_u64)
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

    fn startup_status(&self) -> Option<String> {
        let startup = self.nested_object("startup");
        match self.nested_string(startup, &["status"]) {
            Field::Present(value) if !value.trim().is_empty() => Some(value),
            _ => None,
        }
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
}

/// 由「代理是否在运行 + 启动保护状态」折叠出运行态。
///
/// 关键点：`proxy.running == false` 绝不能回退到上一次的运行态。官方输出里
/// `running=false` 表示代理确实不在运行，此时沿用旧的 `Running` 会让界面
/// 在进程已经退出后继续显示「运行中 / 健康」——实测杀掉代理进程后
/// `ocxd status` 仍返回 `running/healthy`，端口却已无人监听。
fn fold_proxy_runtime(
    proxy_running: Option<bool>,
    startup_status: Option<&str>,
    fallback: RuntimeState,
) -> RuntimeState {
    match (proxy_running, startup_status) {
        (Some(true), _) => RuntimeState::Running,
        (Some(false), Some("at-risk")) => RuntimeState::AtRisk,
        (Some(false), Some("external-takeover")) => RuntimeState::ExternalTakeover,
        (Some(false), Some("not-found")) => RuntimeState::NotFound,
        (Some(false), _) => RuntimeState::Stopped,
        _ => fallback,
    }
}

fn health_for_runtime(runtime: RuntimeState) -> HealthState {
    match runtime {
        RuntimeState::Running => HealthState::Healthy,
        RuntimeState::AtRisk | RuntimeState::ExternalTakeover => HealthState::Degraded,
        _ => HealthState::Unknown,
    }
}

/// 把官方输出映射为冻结状态；未知值和缺失字段都保持原状态。
pub fn map_official_output(
    report: &OfficialStatusReport,
    previous_runtime: Option<RuntimeState>,
    previous_connection: ConnectionState,
    previous_operation: OperationState,
) -> MappedOfficialStatus {
    let mut reasons = Vec::new();
    let fallback_runtime = previous_runtime.unwrap_or(RuntimeState::NotFound);
    let mut runtime = fallback_runtime;
    let mut health = HealthState::Unknown;
    let mut operation = previous_operation;
    let mut runtime_is_mapped = false;

    let top_status = report.string(&["status", "state", "runtime"]);
    match top_status {
        Field::Missing => {
            let proxy = report.nested_object("proxy");
            let startup = report.nested_object("startup");
            let proxy_running = match report.nested_boolean(proxy, &["running"]) {
                Field::Present(value) => Some(value),
                _ => None,
            };
            let startup_status = match report.nested_string(startup, &["status"]) {
                Field::Present(value) if !value.trim().is_empty() => Some(normalized(&value)),
                _ => None,
            };
            runtime =
                fold_proxy_runtime(proxy_running, startup_status.as_deref(), fallback_runtime);
            health = health_for_runtime(runtime);
            runtime_is_mapped = true;
            reasons.push(DowngradeReason::MissingStatus);
        }
        Field::Unknown => {
            let proxy = report.nested_object("proxy");
            let startup = report.nested_object("startup");
            let proxy_running = match report.nested_boolean(proxy, &["running"]) {
                Field::Present(value) => Some(value),
                _ => None,
            };
            let startup_status = match report.nested_string(startup, &["status"]) {
                Field::Present(value) if !value.trim().is_empty() => Some(normalized(&value)),
                _ => None,
            };
            runtime =
                fold_proxy_runtime(proxy_running, startup_status.as_deref(), fallback_runtime);
            health = health_for_runtime(runtime);
            runtime_is_mapped = true;
            reasons.push(DowngradeReason::InvalidStatus);
        }
        Field::Present(ref status) => match normalized(status).as_str() {
            "running" => match report.boolean(&["ready", "isReady"]) {
                Field::Present(true) => {
                    runtime = RuntimeState::Running;
                    health = HealthState::Healthy;
                    runtime_is_mapped = true;
                }
                Field::Present(false) => {
                    let port_reachable = matches!(
                        report.boolean(&["portReachable", "port_reachable"]),
                        Field::Present(true)
                    );
                    let process_present = matches!(
                        report.boolean(&["processPresent", "process_present"]),
                        Field::Present(true)
                    );
                    if !port_reachable && process_present {
                        runtime = RuntimeState::Unreachable;
                        health = HealthState::Unhealthy;
                    } else {
                        runtime = RuntimeState::Pending;
                        health = HealthState::Degraded;
                    }
                    runtime_is_mapped = true;
                }
                Field::Missing => reasons.push(DowngradeReason::MissingOfficialFields),
                Field::Unknown => reasons.push(DowngradeReason::InvalidReady),
            },
            "loading" | "starting" => {
                runtime = RuntimeState::Starting;
                health = HealthState::Unknown;
                runtime_is_mapped = true;
            }
            "pending" => {
                runtime = RuntimeState::Pending;
                health = HealthState::Degraded;
                runtime_is_mapped = true;
            }
            "unreachable" => {
                runtime = RuntimeState::Unreachable;
                health = HealthState::Unhealthy;
                runtime_is_mapped = true;
            }
            "stopped" | "stop-confirmed" | "stopped-confirmed" => {
                runtime = RuntimeState::Stopped;
                health = HealthState::Unknown;
                runtime_is_mapped = true;
                if status == "stop-confirmed" || status == "stopped-confirmed" {
                    operation = OperationState::Succeeded;
                }
            }
            "at-risk" | "at_risk" | "atrisk" => {
                runtime = RuntimeState::AtRisk;
                health = HealthState::Degraded;
                runtime_is_mapped = true;
            }
            "external-takeover" | "external_takeover" | "externaltakeover" => {
                runtime = RuntimeState::ExternalTakeover;
                health = HealthState::Degraded;
                runtime_is_mapped = true;
            }
            "not-found" | "not_found" | "notfound" => {
                runtime = RuntimeState::NotFound;
                health = HealthState::Unknown;
                runtime_is_mapped = true;
            }
            "stop-timeout" | "stopping-timeout" => {
                runtime = RuntimeState::Stopping;
                health = HealthState::Unknown;
                operation = OperationState::Failed;
                runtime_is_mapped = true;
            }
            "unknown" | "missing" => {
                reasons.push(DowngradeReason::UnknownStatus);
            }
            _ => reasons.push(DowngradeReason::UnknownStatus),
        },
    }
    if matches!(top_status, Field::Missing | Field::Unknown) {
        let proxy = report.nested_object("proxy");
        let startup = report.nested_object("startup");
        let proxy_running = match report.nested_boolean(proxy, &["running"]) {
            Field::Present(value) => Some(value),
            _ => None,
        };
        let startup_status = match report.nested_string(startup, &["status"]) {
            Field::Present(value) if !value.trim().is_empty() => Some(normalized(&value)),
            _ => None,
        };
        runtime = fold_proxy_runtime(proxy_running, startup_status.as_deref(), fallback_runtime);
        health = health_for_runtime(runtime);
    }

    match report.string(&["health"]) {
        Field::Missing => {}
        Field::Unknown => {
            reasons.push(DowngradeReason::InvalidHealth);
            health = HealthState::Unknown;
            runtime = fallback_runtime;
            runtime_is_mapped = false;
        }
        Field::Present(value) => match normalized(&value).as_str() {
            "healthy" if health == HealthState::Unknown && !runtime_is_mapped => {
                health = HealthState::Healthy;
            }
            "degraded" if health == HealthState::Unknown && !runtime_is_mapped => {
                health = HealthState::Degraded;
            }
            "unhealthy" if health == HealthState::Unknown && !runtime_is_mapped => {
                health = HealthState::Unhealthy;
            }
            "unknown" => health = HealthState::Unknown,
            _ => {}
        },
    }

    // 若 ready 无法解析，即使官方状态可读也必须退回上一状态。
    if matches!(reasons.last(), Some(DowngradeReason::InvalidReady)) {
        runtime = fallback_runtime;
        health = HealthState::Unknown;
        runtime_is_mapped = false;
    }
    let _ = runtime_is_mapped;

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

    if let Field::Present(value) = report.string(&["operation"]) {
        operation = match normalized(&value).as_str() {
            "rolling-back" | "rolling_back" => OperationState::RollingBack,
            "applying" => OperationState::Applying,
            "backing-up" | "backing_up" => OperationState::BackingUp,
            "validating" => OperationState::Validating,
            "failed" => OperationState::Failed,
            "succeeded" => OperationState::Succeeded,
            "cancelled" => OperationState::Cancelled,
            "idle" => OperationState::Idle,
            _ => previous_operation,
        };
    }

    let port = report.port();
    let pid = report.pid().or_else(|| report.proxy_pid());
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
        opencodex_home: report
            .path(&["opencodexHome", "opencodex_home", "home"])
            .or_else(|| report.nested_path("paths", &["config"])),
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
    if facts.data_root.as_os_str().is_empty() {
        reasons.push(DowngradeReason::MissingDataRoot);
    }
    let startup = report.nested_object("startup");
    if startup.is_none() {
        reasons.push(DowngradeReason::InvalidStartupFacts);
    }
    let (protection, reboot_safe, service_present, shim_present) = report.startup_facts();
    facts.protection = protection;
    facts.reboot_safe = reboot_safe;
    facts.service_present = service_present;
    facts.shim_present = shim_present;
    facts.startup_status = report.startup_status();

    MappedOfficialStatus {
        runtime,
        health,
        connection,
        operation,
        facts,
        port,
        pid,
        downgrade_reasons: reasons,
    }
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

/// 三维状态采集器；失败保留上一状态，成功重置退避计数。
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
}

impl<S> StatusCollector<S>
where
    S: StatusSource,
{
    pub fn new(source: S) -> Self {
        Self {
            source,
            matrix: StatusMatrix {
                runtime: RuntimeState::NotFound,
                connection: ConnectionState::Unconfigured,
                operation: OperationState::Idle,
            },
            facts: RuntimeFacts::default(),
            port: None,
            pid: None,
            failure_count: 0,
            last_request: None,
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

    pub fn failure_count(&self) -> u32 {
        self.failure_count
    }

    pub fn next_interval(&self, dimension: StatusDimension, backgrounded: bool) -> Duration {
        RefreshPolicy::next_interval(dimension, self.failure_count, backgrounded)
    }

    pub fn request_allowed(&self, now: Instant) -> bool {
        !RefreshPolicy::is_debounced(self.last_request, now)
    }

    pub fn record_failure(&mut self) {
        self.failure_count = self.failure_count.saturating_add(1);
    }

    pub fn refresh(&mut self) -> Result<MappedOfficialStatus, CollectError> {
        self.refresh_at(Instant::now())
    }

    /// 使用注入时钟刷新；去抖期内返回 Timeout，不调用来源。
    pub fn refresh_at(&mut self, now: Instant) -> Result<MappedOfficialStatus, CollectError> {
        if !self.request_allowed(now) {
            return Err(CollectError::Timeout);
        }
        self.last_request = Some(now);
        let payload = match self.source.fetch() {
            Ok(payload) => payload,
            Err(error) => {
                self.record_failure();
                // 运行来源未解析（或解析到的入口已不存在）= 未发现安装。这是确定性事实，
                // 不能沿用上一份「未运行 / 可启动」状态——否则卸载后概览、托盘等仍显示
                // 可启动（真机反馈）。只有瞬时失败才保留上一份状态（FZ-07 / FZ-08）。
                if !self.source.resolved() {
                    self.matrix = StatusMatrix {
                        runtime: RuntimeState::NotFound,
                        connection: self.matrix.connection,
                        operation: self.matrix.operation,
                    };
                    self.facts = RuntimeFacts::default();
                    self.port = None;
                    self.pid = None;
                }
                return Err(error);
            }
        };
        let report = OfficialStatusReport::from_value(payload);
        let mapped = map_official_output(
            &report,
            Some(self.matrix.runtime),
            self.matrix.connection,
            self.matrix.operation,
        );
        self.matrix = StatusMatrix {
            runtime: mapped.runtime,
            connection: mapped.connection,
            operation: mapped.operation,
        };
        self.facts = mapped.facts.clone();
        self.port = mapped.port;
        self.pid = mapped.pid.clone();
        self.failure_count = 0;
        Ok(mapped)
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
    use std::thread::sleep;

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
                Some(RuntimeState::Loading),
                ConnectionState::Synced,
                OperationState::Idle,
            );
            assert_eq!(mapped.runtime, expected_runtime, "runtime for {status}");
            assert_eq!(mapped.health, expected_health, "health for {status}");
        }

        let stopped = map_official_output(
            &official("stop-confirmed", false),
            Some(RuntimeState::Stopping),
            ConnectionState::Synced,
            OperationState::Idle,
        );
        assert_eq!(stopped.operation, OperationState::Succeeded);

        let timed_out = map_official_output(
            &official("stop-timeout", false),
            Some(RuntimeState::Stopping),
            ConnectionState::Synced,
            OperationState::Idle,
        );
        assert_eq!(timed_out.operation, OperationState::Failed);
    }

    #[test]
    fn fz07_keeps_previous_state_for_unknown_output() {
        let mapped = map_official_output(
            &report(json!({"status": "mystery-output", "dataRoot": "/tmp/fixture"})),
            Some(RuntimeState::AtRisk),
            ConnectionState::Synced,
            OperationState::Idle,
        );
        assert_eq!(mapped.runtime, RuntimeState::AtRisk);
        assert_eq!(mapped.health, HealthState::Unknown);
        assert!(mapped
            .downgrade_reasons
            .contains(&DowngradeReason::UnknownStatus));
    }

    #[test]
    fn fz07_health_parse_failure_preserves_previous_state() {
        let mapped = map_official_output(
            &report(json!({
                "status": "running",
                "ready": true,
                "health": 123,
                "dataRoot": "/tmp/fixture"
            })),
            Some(RuntimeState::Stopped),
            ConnectionState::Synced,
            OperationState::Idle,
        );
        assert_eq!(mapped.runtime, RuntimeState::Stopped);
        assert_eq!(mapped.health, HealthState::Unknown);
        assert!(mapped
            .downgrade_reasons
            .contains(&DowngradeReason::InvalidHealth));
    }

    #[test]
    fn fz07_missing_fields_are_degraded_not_invented() {
        let mapped = map_official_output(
            &report(json!({"status": "running"})),
            Some(RuntimeState::Running),
            ConnectionState::Synced,
            OperationState::Idle,
        );
        assert_eq!(mapped.runtime, RuntimeState::Running);
        assert_eq!(mapped.health, HealthState::Unknown);
        assert!(mapped
            .downgrade_reasons
            .contains(&DowngradeReason::MissingOfficialFields));
        assert!(mapped
            .downgrade_reasons
            .contains(&DowngradeReason::MissingDataRoot));
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
    fn timeout_preserves_previous_state_and_increments_backoff() {
        let mut collector =
            StatusCollector::new(VirtualStatusSource::new(Err(CollectError::Timeout)));
        sleep(Duration::from_millis(1));
        let error = collector.refresh().expect_err("timeout");
        assert_eq!(error, CollectError::Timeout);
        assert_eq!(collector.matrix().runtime, RuntimeState::NotFound);
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
            Some(RuntimeState::Loading),
            ConnectionState::Unconfigured,
            OperationState::Idle,
        );

        assert_eq!(mapped.runtime, RuntimeState::AtRisk);
        assert_eq!(mapped.health, HealthState::Degraded);
        assert_eq!(mapped.port, Some(10100));
        assert_eq!(mapped.facts.runtime_label.as_deref(), Some("bundled"));
        assert_eq!(mapped.facts.version_drift.as_deref(), Some("2.50.0"));
        assert_eq!(
            mapped.facts.opencodex_home,
            Some(PathBuf::from("/tmp/opencodex-home/config.json"))
        );
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
            Some(RuntimeState::Loading),
            ConnectionState::Unconfigured,
            OperationState::Idle,
        );
        assert_eq!(running.runtime, RuntimeState::Running);
        assert_eq!(running.health, HealthState::Healthy);

        let not_found = map_official_output(
            &report(json!({
                "proxy": {"running": false},
                "startup": {"status": "not-found"},
                "dataRoot": "/tmp/fixture"
            })),
            Some(RuntimeState::Running),
            ConnectionState::Unconfigured,
            OperationState::Idle,
        );
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
            Some(RuntimeState::Running),
            ConnectionState::Unconfigured,
            OperationState::Idle,
        );
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
            Some(RuntimeState::AtRisk),
            ConnectionState::Unconfigured,
            OperationState::Idle,
        );
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
            Some(RuntimeState::Loading),
            ConnectionState::Synced,
            OperationState::Idle,
        );
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
    /// 事实，必须落到 `not_found`，不得沿用上一次「未运行 / 可启动」。瞬时失败
    /// （超时 / 解析失败）仍保留上一份状态（FZ-07 / FZ-08）。
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
        assert!(crate::types::status::runtime_can_start(
            collector.matrix().runtime
        ));

        // 卸载后入口消失：来源不再解析 ⇒ 立刻落到 not_found（概览据此只给「刷新状态」，
        // 不再给「启动」）。托盘「启动」仍按 FZ-06 冻结门控对 not_found 放行，属既有契约。
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

    /// 瞬时失败（超时）仍保留上一份状态，不因一次抖动就清空事实。
    #[test]
    fn transient_failure_preserves_last_state() {
        use std::cell::Cell;

        struct FlickerSource {
            fail: Cell<bool>,
        }
        impl StatusSource for FlickerSource {
            fn fetch(&self) -> Result<Value, CollectError> {
                if self.fail.get() {
                    Err(CollectError::Timeout)
                } else {
                    Ok(json!({
                        "status": "stopped",
                        "dataRoot": "/tmp/opencodex-fixture",
                        "startup": {"protection": "none", "rebootSafe": false}
                    }))
                }
            }
            // 入口仍然解析得到：只是这一次没采到。
            fn resolved(&self) -> bool {
                true
            }
        }

        let mut collector = StatusCollector::new(FlickerSource {
            fail: Cell::new(false),
        });
        let t0 = Instant::now();
        collector.refresh_at(t0).expect("first refresh ok");
        assert_eq!(collector.matrix().runtime, RuntimeState::Stopped);

        collector.source.fail.set(true);
        let error = collector
            .refresh_at(t0 + Duration::from_secs(2))
            .expect_err("timeout expected");
        assert_eq!(error, CollectError::Timeout);
        assert_eq!(collector.matrix().runtime, RuntimeState::Stopped);
    }
}
