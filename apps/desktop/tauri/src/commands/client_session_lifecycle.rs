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
        let result = with_native_coordination(&coordination, deadline, |lease| {
            self.session.seal_completed_stop(context, proof, lease)?;
            self.head = self
                .index
                .advance(&self.head, State::Stopped, Some(self.session.references()?))
                .map_err(failure)?;
            let pending = self
                .session
                .prepare_restore(context, proof, lease, deadline)?;
            self.head = self
                .index
                .record_restore_plan(&self.head, pending.id(), pending.sha256())
                .map_err(failure)?;
            Ok(())
        });
        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(match error {
                AppError::RuntimeManaged { .. } => pending_restore(),
                other => other,
            }),
        }
    }
}
