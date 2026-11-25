//! FZ-06 / FZ-07 / FZ-08 前后端共享状态 DTO。
//!
//! 该类型只做命令层序列化和模块事实投影；不读取文件、网络或
//! Keychain，也不执行真实子进程。

use crate::modules::status::{CollectError, HealthState, RuntimeFacts};
use crate::types::status::StatusMatrix;
use serde::{Deserialize, Serialize};

/// 前端可消费的三维状态快照。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StatusSnapshotDto {
    pub matrix: StatusMatrix,
    pub facts: RuntimeFacts,
    pub port: Option<u16>,
    pub pid: Option<String>,
    pub can_start: bool,
    pub can_stop: bool,
    pub can_restart: bool,
    pub source: StatusSource,
    /// Latest sample failure, or JSON null when no failure was observed.
    pub status_failure: Option<StatusFailure>,
}

/// AC-05 restore 风险摘要；只包含可展示事实，不读取配置内容或凭据。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreGuidanceDto {
    pub runtime_state: crate::types::status::RuntimeState,
    pub health: crate::modules::status::HealthState,
    pub startup_status: Option<String>,
    pub protection: Option<String>,
    pub reboot_safe: Option<bool>,
    pub service_present: Option<bool>,
    pub shim_present: Option<bool>,
    pub version_drift: Option<String>,
    pub data_root: String,
    pub opencodex_home: Option<String>,
}

impl RestoreGuidanceDto {
    pub fn from_snapshot(snapshot: &StatusSnapshotDto) -> Self {
        Self {
            runtime_state: snapshot.matrix.runtime,
            health: snapshot.facts.health,
            startup_status: snapshot.facts.startup_status.clone(),
            protection: snapshot.facts.protection.clone(),
            reboot_safe: snapshot.facts.reboot_safe,
            service_present: snapshot.facts.service_present,
            shim_present: snapshot.facts.shim_present,
            version_drift: snapshot.facts.version_drift.clone(),
            data_root: snapshot.facts.data_root.to_string_lossy().into_owned(),
            opencodex_home: snapshot
                .facts
                .opencodex_home
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusSource {
    Unconfigured,
    Live,
    Fixture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusFailure {
    Timeout,
    Unreachable,
    Parse,
    Unmapped,
}

impl StatusFailure {
    pub(crate) fn from_collect_error(error: CollectError) -> Option<Self> {
        match error {
            CollectError::Timeout => Some(Self::Timeout),
            CollectError::Unreachable => Some(Self::Unreachable),
            CollectError::Parse => Some(Self::Parse),
            CollectError::Unmapped => Some(Self::Unmapped),
            CollectError::ManagedHomeIdle => None,
        }
    }
}

/// 组装静态未配置快照；后续由注入采集器替换，避免前端猜测状态。
pub fn unconfigured_snapshot() -> StatusSnapshotDto {
    let matrix = StatusMatrix {
        runtime: crate::types::status::RuntimeState::Loading,
        connection: crate::types::status::ConnectionState::Unconfigured,
        operation: crate::types::status::OperationState::Idle,
    };
    StatusSnapshotDto {
        matrix,
        facts: RuntimeFacts::default(),
        port: None,
        pid: None,
        can_start: false,
        can_stop: false,
        can_restart: false,
        source: StatusSource::Unconfigured,
        status_failure: None,
    }
}

/// HealthState 直接共享，保证前端不发明第五个状态。
pub fn health_label(health: HealthState) -> &'static str {
    match health {
        HealthState::Healthy => "ready",
        HealthState::Degraded => "degraded",
        HealthState::Unhealthy => "unhealthy",
        HealthState::Unknown => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::status::{StatusCollector, VirtualStatusSource};
    use crate::types::status::RuntimeState;
    use serde_json::json;

    #[test]
    fn snapshot_matches_fz06_state_names() {
        let mut collector = StatusCollector::new(VirtualStatusSource::new(Ok(json!({
            "status": "running", "ready": true, "port": 10100, "pid": 39421
        }))));
        collector.refresh().unwrap();
        let value = collector.snapshot();
        assert_eq!(value.matrix.runtime, RuntimeState::Running);
        assert_eq!(value.facts.health, HealthState::Healthy);
        assert_eq!(value.port, Some(10100));
        assert_eq!(value.pid.as_deref(), Some("39421"));
        assert!(!value.can_start);
        assert!(value.can_stop && value.can_restart);
        let payload = serde_json::to_value(value).unwrap();
        assert_eq!(payload["matrix"]["runtime"], "running");
        assert_eq!(payload["facts"]["health"], "healthy");
    }

    #[test]
    fn health_label_matches_frozen_values() {
        assert_eq!(health_label(HealthState::Healthy), "ready");
        assert_eq!(health_label(HealthState::Degraded), "degraded");
        assert_eq!(health_label(HealthState::Unhealthy), "unhealthy");
        assert_eq!(health_label(HealthState::Unknown), "unknown");
    }
}
