//! Actual Mac application/CLI/tray action orchestration for current originals.
//! A completed indexed Stop currently remains pending client compensation.
use crate::{
    commands::runtime_recovery_v2_owner::CompletedLifecycleStop,
    errors::{AppError, AppResult},
    infrastructure::{
        macos_client_originals::{
            session::StartupSession,
            session_index::{Head, Index, State},
        },
        official_client_coordination::{native::with_native_coordination, Context},
    },
    state::ProcessContext,
};
use std::{
    path::{Path, PathBuf},
    time::Instant,
};
fn failure<T>(_: T) -> AppError {
    AppError::RuntimeManaged {
        code: "client_original_session_unconfirmed".into(),
        detail: "客户端原件会话未核对，原件及当前输入已保留；请检查诊断后处理原会话".into(),
    }
}
pub(super) fn pending_restore() -> AppError {
    AppError::RuntimeManaged {
        code: "client_original_restore_pending".into(),
        detail: "服务已正常停止，客户端配置原件恢复尚未完成；原会话和现场已保留".into(),
    }
}
fn client(context: &ProcessContext) -> AppResult<PathBuf> {
    std::fs::canonicalize(context.working_directory.join(".codex")).map_err(failure)
}
pub(super) struct Active {
    index: Index,
    head: Head,
    session: StartupSession,
    client: PathBuf,
    stop_pair: Option<crate::infrastructure::macos_process_pair::PairWatch>,
}
impl Active {
    fn coordination(&self, context: &ProcessContext) -> AppResult<Context> {
        Ok(Context {
            runtime: context.runtime.clone(),
            user_profile: context.working_directory.clone(),
            opencodex_home: context.opencodex_home.clone(),
            codex_home: self.client.clone(),
            control_root: self.index.control_root().map_err(failure)?,
        })
    }
    pub fn capture(context: &ProcessContext, data: &Path, deadline: Instant) -> AppResult<Self> {
        let client = client(context)?;
        let index = Index::open(context, data, &client).map_err(failure)?;
        let intent = index.reserve_start().map_err(failure)?;
        let coordination = Context {
            runtime: context.runtime.clone(),
            user_profile: context.working_directory.clone(),
            opencodex_home: context.opencodex_home.clone(),
            codex_home: client.clone(),
            control_root: index.control_root().map_err(failure)?,
        };
        let session = with_native_coordination(&coordination, deadline, |lease| {
            StartupSession::begin(context, data, &client, lease, deadline)
        })?;
        let head = index
            .advance(&intent, State::Captured, Some(session.references()?))
            .map_err(failure)?;
        Ok(Self {
            index,
            head,
            session,
            client,
            stop_pair: None,
        })
    }
    pub fn load(context: &ProcessContext, data: &Path) -> AppResult<Self> {
        let client = client(context)?;
        let index = Index::open(context, data, &client).map_err(failure)?;
        let head = index.head().map_err(failure)?.ok_or_else(|| failure(()))?;
        if head.state != State::Ready {
            return Err(failure(()));
        }
        let refs = head.references.as_ref().ok_or_else(|| failure(()))?;
        let session = StartupSession::open(context, data, &client, refs)?;
        Ok(Self {
            index,
            head,
            session,
            client,
            stop_pair: None,
        })
    }
    pub fn verify_ready(&self, context: &ProcessContext, deadline: Instant) -> AppResult<()> {
        self.session.verify_current_ready(context, deadline)
    }
    pub fn ready(&mut self, context: &ProcessContext, deadline: Instant) -> AppResult<()> {
        let coordination = self.coordination(context)?;
        with_native_coordination(&coordination, deadline, |lease| {
            self.session.bind_ready(context, lease, deadline)
        })?;
        self.head = self
            .index
            .advance(&self.head, State::Ready, Some(self.session.references()?))
            .map_err(failure)?;
        Ok(())
    }
    pub fn pre_stop(&mut self, context: &ProcessContext, deadline: Instant) -> AppResult<()> {
        self.stop_pair = Some(self.session.watch_stop_pair(context, deadline)?);
        self.session.seal_pre_stop(context, deadline)?;
        self.head = self
            .index
            .advance(
                &self.head,
                State::StopPrepared,
                Some(self.session.references()?),
            )
            .map_err(failure)?;
        Ok(())
    }
    pub fn stopped(
        &mut self,
        context: &ProcessContext,
        proof: &CompletedLifecycleStop,
        deadline: Instant,
    ) -> AppResult<()> {
        let coordination = self.coordination(context)?;
        let phase = std::cell::Cell::new("native-coordination");
        let started = Instant::now();
        let timings = std::cell::RefCell::new(Vec::with_capacity(8));
        let mark = |next: &'static str| {
            phase.set(next);
            timings
                .borrow_mut()
                .push((next, started.elapsed().as_millis()));
        };
        let result = (|| {
            mark("original-pair-exit");
            let pair = self
                .stop_pair
                .as_mut()
                .ok_or_else(|| failure(()))?
                .wait(deadline)?;
            self.session
                .require_stopped_pair(context, &pair, deadline)?;
            let _ = crate::infrastructure::runtime_log::RuntimeLog::new(self.index.data_root())
                .append_event("stop-proof: original-native-launcher-and-service confirmed code=0");
            mark("native-coordination");
            with_native_coordination(&coordination, deadline, |lease| {
                mark("seal-stopped");
                self.session
                    .seal_completed_stop(context, proof, &pair, lease, deadline)?;
                mark("index-stopped");
                self.head = self
                    .index
                    .advance(&self.head, State::Stopped, Some(self.session.references()?))
                    .map_err(failure)?;
                mark("prepare-plan");
                let pending = self
                    .session
                    .prepare_restore(context, proof, &pair, lease, deadline)?;
                mark("index-plan");
                self.head = self
                    .index
                    .record_restore_plan(&self.head, pending.id(), pending.sha256())
                    .map_err(failure)?;
                mark("restore-three-artifacts");
                let plan = self.head.restore_plan.as_ref().ok_or_else(|| failure(()))?;
                self.session
                    .restore_completed_stop(context, proof, &pair, lease, deadline, plan)?;
                let _ = crate::infrastructure::runtime_log::RuntimeLog::new(self.index.data_root())
                    .append_event("client-stop-recovery: three-artifact-originals restored and readback verified");
                mark("release-coordination");
                Ok(())
            })?;
            mark("full-tree-and-lock-release");
            let plan = self.head.restore_plan.as_ref().ok_or_else(|| failure(()))?;
            let readback = self
                .session
                .completed_readback(context, proof, &pair, deadline, plan)?;
            if self
                .head
                .references
                .as_ref()
                .is_none_or(|refs| refs.attempt != readback.attempt())
            {
                return Err(failure(()));
            }
            let _ = crate::infrastructure::runtime_log::RuntimeLog::new(self.index.data_root())
                .append_event(
                "client-stop-recovery: full-tree protection and two stable-lock releases verified",
            );
            mark("close-session-index");
            #[cfg(test)]
            if std::env::var_os("OCXD024_INTERRUPT_BEFORE_CLOSE").as_deref()
                == Some(std::ffi::OsStr::new("1"))
            {
                if !self.index.data_root().starts_with("/private/tmp")
                    || !self.index.data_root().ancestors().any(|p| {
                        p.file_name().is_some_and(|n| {
                            n.to_string_lossy()
                                .starts_with("ocxd024-versioned-lifecycle-")
                        })
                    })
                {
                    return Err(failure(()));
                }
                return Err(crate::errors::AppError::RuntimeManaged {
                    code: "owned_fixture_close_interrupted".into(),
                    detail: "owned fixture interruption after actual complete readback".into(),
                });
            }
            self.head = self
                .index
                .close_restored(&self.head, &readback)
                .map_err(failure)?;
            let _ = crate::infrastructure::runtime_log::RuntimeLog::new(self.index.data_root())
                .append_event(
                    "client-stop-recovery: restored session index committed and readback verified",
                );
            Ok(())
        })();
        // Fixed phase names and monotonic durations only; no paths, source
        // bytes, process arguments, token or native ownership material.
        let _ = crate::infrastructure::runtime_log::RuntimeLog::new(self.index.data_root())
            .append_event(&format!(
                "client-stop-recovery: measured elapsed_ms={} timings={:?}",
                started.elapsed().as_millis(),
                timings.borrow(),
            ));
        match result {
            Ok(()) => Ok(()),
            Err(error) => {
                let classification = match &error {
                    AppError::RuntimeManaged { code, .. }
                        if code.len() <= 96
                            && code.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') =>
                    {
                        code.as_str()
                    }
                    AppError::Timeout => "timeout",
                    AppError::FileSystem { .. } => "filesystem",
                    _ => "unconfirmed",
                };
                let _ = crate::infrastructure::runtime_log::RuntimeLog::new(self.index.data_root())
                    .append_event(&format!(
                        "client-stop-recovery: failed phase={} code={}",
                        phase.get(),
                        classification
                    ));
                Err(match error {
                    AppError::RuntimeManaged { .. } => pending_restore(),
                    other => other,
                })
            }
        }
    }
}
