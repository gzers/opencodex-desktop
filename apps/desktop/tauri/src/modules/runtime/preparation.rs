//! Runtime-handle-scoped startup observations. Never persisted as installation
//! authority; stale workers cannot overwrite a newer explicit attempt.
use crate::errors::{AppError, AppResult};
use serde::Serialize;
use std::sync::Mutex;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Idle,
    Checking,
    Preparing,
    Verifying,
    Extracting,
    Assembling,
    Activating,
    Supplied,
    Failed,
    Removed,
    Review,
}
impl Phase {
    pub fn active(self) -> bool {
        matches!(
            self,
            Self::Checking
                | Self::Preparing
                | Self::Verifying
                | Self::Extracting
                | Self::Assembling
                | Self::Activating
        )
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Failure {
    Integrity,
    Busy,
    RecoveryRequired,
    Unavailable,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub attempt_id: Option<String>,
    pub revision: u64,
    pub phase: Phase,
    pub failed_phase: Option<Phase>,
    pub failure: Option<Failure>,
    pub retryable: bool,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            attempt_id: None,
            revision: 0,
            phase: Phase::Idle,
            failed_phase: None,
            failure: None,
            retryable: false,
        }
    }
}
#[derive(Debug, Default)]
pub struct Preparation(Mutex<Snapshot>);
fn fail() -> AppError {
    AppError::RuntimeManaged {
        code: "preparation_changed".into(),
        detail: "内置环境准备状态已变化，请重新核对".into(),
    }
}
impl Preparation {
    pub fn snapshot(&self) -> AppResult<Snapshot> {
        self.0.lock().map(|s| s.clone()).map_err(|_| fail())
    }
    pub fn begin(&self, expected: Option<&str>) -> AppResult<String> {
        let mut state = self.0.lock().map_err(|_| fail())?;
        if state.phase.active()
            || expected
                .is_some_and(|id| state.attempt_id.as_deref() != Some(id) || !state.retryable)
        {
            return Err(fail());
        }
        let id = uuid::Uuid::new_v4().to_string();
        *state = Snapshot {
            attempt_id: Some(id.clone()),
            revision: state.revision.saturating_add(1),
            phase: Phase::Checking,
            ..Default::default()
        };
        Ok(id)
    }
    pub fn stage(&self, id: &str, phase: Phase) -> AppResult<()> {
        let mut state = self.0.lock().map_err(|_| fail())?;
        if state.attempt_id.as_deref() != Some(id) || !state.phase.active() || !phase.active() {
            return Err(fail());
        }
        state.phase = phase;
        state.revision = state.revision.saturating_add(1);
        Ok(())
    }
    pub fn finish(
        &self,
        id: &str,
        phase: Phase,
        failure: Option<Failure>,
        retryable: bool,
    ) -> AppResult<()> {
        let mut state = self.0.lock().map_err(|_| fail())?;
        if state.attempt_id.as_deref() != Some(id) || !state.phase.active() || phase.active() {
            return Err(fail());
        }
        let failed_phase = failure.map(|_| state.phase);
        state.phase = phase;
        state.failed_phase = failed_phase;
        state.failure = failure;
        state.retryable = retryable;
        state.revision = state.revision.saturating_add(1);
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_workers_or_double_retry_cannot_replace_the_current_attempt() {
        let state = Preparation::default();
        let a = state.begin(None).unwrap();
        assert!(state.begin(None).is_err());
        state.stage(&a, Phase::Extracting).unwrap();
        state
            .finish(&a, Phase::Failed, Some(Failure::Integrity), true)
            .unwrap();
        assert_eq!(
            state.snapshot().unwrap().failed_phase,
            Some(Phase::Extracting)
        );
        assert!(state.begin(Some("foreign")).is_err());
        let b = state.begin(Some(&a)).unwrap();
        assert!(state.stage(&a, Phase::Activating).is_err());
        assert!(state.finish(&a, Phase::Supplied, None, false).is_err());
        state.finish(&b, Phase::Supplied, None, false).unwrap();
        assert!(state.begin(Some(&b)).is_err());
    }
}
