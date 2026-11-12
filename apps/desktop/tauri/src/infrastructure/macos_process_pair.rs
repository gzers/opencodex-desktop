//! Exact ready-bound service and launcher pair. This is NOT a complete process
//! family, socket or persistent-writer certificate and grants no file mutation.
use super::macos_process_exit::{parent_pid, ExitWatch, Generation, NormalExitProof};
use crate::errors::{AppError, AppResult};
use std::time::{Duration, Instant};
fn fail(code: &str) -> AppError {
    AppError::RuntimeManaged {
        code: code.into(),
        detail: "原服务与启动父进程的正常退出尚未完整核对，原会话保留".into(),
    }
}
pub struct PairWatch {
    service: ExitWatch,
    launcher: ExitWatch,
}
pub struct PairExitProof {
    service: NormalExitProof,
    launcher: NormalExitProof,
}
impl PairExitProof {
    pub fn require(
        &self,
        service: Generation,
        launcher: Generation,
        deadline: Instant,
    ) -> AppResult<()> {
        if Instant::now() >= deadline {
            return Err(fail("lifecycle_timeout"));
        }
        if self.service.generation() != service || self.launcher.generation() != launcher {
            return Err(fail("lifecycle_pair_binding"));
        }
        Ok(())
    }
}
impl PairWatch {
    pub fn capture(
        service: Generation,
        launcher: Generation,
        deadline: Instant,
    ) -> AppResult<Self> {
        let relation = || {
            if Instant::now() >= deadline {
                return Err(fail("lifecycle_timeout"));
            }
            if service.pid == launcher.pid
                || Generation::observe(service.pid)? != service
                || Generation::observe(launcher.pid)? != launcher
                || parent_pid(service.pid)? != launcher.pid
            {
                return Err(fail("lifecycle_pair_binding"));
            }
            Ok(())
        };
        relation()?;
        let launcher_watch = ExitWatch::capture(launcher)?;
        let service_watch = ExitWatch::capture(service)?;
        relation()?;
        Ok(Self {
            service: service_watch,
            launcher: launcher_watch,
        })
    }
    pub fn normal_exit(&mut self, deadline: Instant) -> AppResult<Option<PairExitProof>> {
        // Inspect BOTH even if one has not exited. A fork/exec/nonzero event on
        // the other must refuse now, not be hidden behind an unrelated wait.
        let service = self.service.normal_exit(deadline)?;
        let launcher = self.launcher.normal_exit(deadline)?;
        match (service, launcher) {
            (Some(service), Some(launcher)) => Ok(Some(PairExitProof { service, launcher })),
            _ => Ok(None),
        }
    }
    pub fn wait(&mut self, deadline: Instant) -> AppResult<PairExitProof> {
        loop {
            if let Some(proof) = self.normal_exit(deadline)? {
                return Ok(proof);
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }
}

#[cfg(test)]
#[path = "macos_process_pair_tests.rs"]
mod tests;
