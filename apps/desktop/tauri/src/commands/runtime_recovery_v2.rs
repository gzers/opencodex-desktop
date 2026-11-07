//! Opaque recovery sessions. Startup registers these commands and injects the
//! trusted resource/OS-owner hooks; neither hook is an IPC authority DTO.
use crate::{
    errors::{AppError, AppResult},
    modules::{
        backup::{manager, safety},
        model_config::management::Management,
        runtime::{
            archive,
            install::{
                InstallError, InstallSourceKind, NullProgressSink, RealVersionProbe,
                RecoveryServiceBoundary, RecoveryServiceState, RuntimeInstaller,
                RuntimeRecoveryOperation, RuntimeRecoveryPhase, RuntimeRecoverySource,
                RuntimeRecoveryStatus, SystemNpmRunner,
            },
            offline_drop::SharedOfflineDrop,
            protection::{self, PanelProtection},
            RuntimeHandle,
        },
    },
    state::{ProcessContext, SharedRuntimeInstall},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

fn error(code: &str, detail: impl Into<String>) -> AppError {
    AppError::RuntimeManaged {
        code: code.into(),
        detail: detail.into(),
    }
}
fn file_error(operation: &str, detail: impl ToString) -> AppError {
    AppError::FileSystem {
        operation: operation.into(),
        detail: detail.to_string(),
    }
}
fn runtime_io(operation: &str, value: AppError) -> AppError {
    match value {
        AppError::FileSystem { detail, .. } => file_error(operation, detail),
        other => other,
    }
}
fn install_error(value: InstallError) -> AppError {
    match value {
        InstallError::FileSystem { operation, detail } => {
            AppError::FileSystem { operation, detail }
        }
        other => other.into(),
    }
}
fn lock<T>(mutex: &Mutex<T>) -> AppResult<std::sync::MutexGuard<'_, T>> {
    mutex
        .lock()
        .map_err(|_| error("recovery_session_poisoned", "恢复私有会话锁不可用"))
}
fn validate_id(id: &str) -> AppResult<()> {
    if uuid::Uuid::parse_str(id).is_err() || id.len() != 36 {
        return Err(error("bad_operation_id", "恢复操作 ID 必须是 UUID"));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecoverySourceKind {
    Bundled,
    Offline,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeRecoveryPrepareRequest {
    pub source: RecoverySourceKind,
    pub staged_archive_id: Option<String>,
}
impl RuntimeRecoveryPrepareRequest {
    pub(super) fn validate(&self) -> AppResult<()> {
        match (self.source, self.staged_archive_id.as_deref()) {
            (RecoverySourceKind::Bundled, None) => Ok(()),
            (RecoverySourceKind::Offline, Some(id)) if validate_id(id).is_ok() => Ok(()),
            _ => Err(error(
                "bad_recovery_source",
                "bundled 不接受路径或 ID；offline 需要受控 stagedArchiveId UUID",
            )),
        }
    }
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeRecoveryApplyRequest {
    pub operation_id: String,
    pub confirmation_digest: String,
}
/// Core Status JSON shape plus a digest authenticating the entire frozen preview.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeRecoveryV2Status {
    #[serde(flatten)]
    pub status: RuntimeRecoveryStatus,
    pub confirmation_digest: String,
}
fn response(status: RuntimeRecoveryStatus) -> AppResult<RuntimeRecoveryV2Status> {
    let mut bytes = b"opencodex-runtime-recovery-confirm-v2\n".to_vec();
    bytes.extend(
        serde_json::to_vec(&status.preview)
            .map_err(|e| file_error("serialize recovery confirmation", e))?,
    );
    Ok(RuntimeRecoveryV2Status {
        confirmation_digest: crate::infrastructure::hash::sha256_hex(&bytes),
        status,
    })
}

#[derive(Clone)]
pub struct RecoveryContext {
    handle: Arc<RuntimeHandle>,
    data_root: PathBuf,
    home: PathBuf,
    process: ProcessContext,
}
impl RecoveryContext {
    pub fn new(
        handle: Arc<RuntimeHandle>,
        data_root: PathBuf,
        home: PathBuf,
        process: ProcessContext,
    ) -> AppResult<Self> {
        let runtime: crate::infrastructure::runtime_executable::SharedRuntimeExecutable =
            handle.clone();
        if !handle.resolver_managed_only()
            || !Arc::ptr_eq(&runtime, &process.runtime)
            || handle.store().path()
                != crate::modules::runtime::RuntimeStore::new(&data_root).path()
            || [
                &data_root,
                &home,
                &process.opencodex_home,
                &process.working_directory,
            ]
            .iter()
            .any(|p| !p.is_absolute())
        {
            return Err(error(
                "recovery_context_mismatch",
                "恢复必须绑定启动时同一受管句柄、D、H 与用户上下文",
            ));
        }
        Ok(Self {
            handle,
            data_root,
            home,
            process,
        })
    }
    pub fn installation_root(&self) -> &Path {
        self.handle.installation_root()
    }
    pub fn data_root(&self) -> &Path {
        &self.data_root
    }
    pub fn process(&self) -> &ProcessContext {
        &self.process
    }
}

/// ONLY compile-pinned resource metadata or an already verified staged preview
/// may supply this pin. Inspecting a tgz does not create distribution authority.
/// Deliberately not deserializable and never accepted by a Tauri command.
#[derive(Clone)]
pub struct RecoverySourcePin {
    version: String,
    sha256: String,
}
impl RecoverySourcePin {
    pub fn from_trusted_metadata(version: String, sha256: String) -> AppResult<Self> {
        if !crate::modules::update::handoff::valid_version(&version)
            || sha256.len() != 64
            || !sha256.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(error("untrusted_recovery_source", "可信恢复源元数据无效"));
        }
        Ok(Self {
            version,
            sha256: sha256.to_ascii_lowercase(),
        })
    }
}
/// Owns only its private copy. Drop cannot delete resources or the user's tgz.
pub struct ResolvedRecoverySource {
    source: RuntimeRecoverySource,
    _owned: tempfile::TempDir,
}
impl ResolvedRecoverySource {
    pub fn copy_pinned(archive_path: &Path, pin: &RecoverySourcePin) -> AppResult<Self> {
        let info =
            safety::inspect(archive_path).map_err(|e| runtime_io("inspect recovery source", e))?;
        if !archive_path.is_absolute() || !info.is_file() || info.len() > archive::MAX_TOTAL_BYTES {
            return Err(error(
                "untrusted_recovery_source",
                "恢复源必须是受控普通归档文件",
            ));
        }
        let owned = tempfile::Builder::new()
            .prefix("ocxd023-runtime-recovery-ipc-source-")
            .tempdir()
            .map_err(|e| file_error("create recovery source temporary directory", e))?;
        let copy = owned.path().join(format!("opencodex-{}.tgz", pin.version));
        use std::io::Read;
        let source = safety::open_file(archive_path, false)
            .map_err(|e| runtime_io("open recovery source", e))?;
        let mut target = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&copy)
            .map_err(|e| file_error("create private recovery archive", e))?;
        let copied = std::io::copy(&mut source.take(archive::MAX_TOTAL_BYTES + 1), &mut target)
            .map_err(|e| file_error("copy pinned recovery archive", e))?;
        if copied > archive::MAX_TOTAL_BYTES {
            return Err(error(
                "recovery_source_too_large",
                "恢复归档读取期间超过限制",
            ));
        }
        let inspection = archive::inspect(&copy, None).map_err(|e| install_error(e.into()))?;
        if inspection.sha256 != pin.sha256 || inspection.version != pin.version {
            return Err(error(
                "recovery_source_pin_mismatch",
                "恢复归档与可信分发摘要或版本不一致",
            ));
        }
        Ok(Self {
            source: RuntimeRecoverySource {
                archive: copy,
                expected_sha256: pin.sha256.clone(),
                expected_version: pin.version.clone(),
            },
            _owned: owned,
        })
    }
    /// Receiver stays locked until the verified private copy is complete. A new
    /// drop cannot invalidate the session, and only this staged ID is cleared.
    pub fn from_staged_offline(
        receiver: &SharedOfflineDrop,
        id: &str,
        pin: &RecoverySourcePin,
    ) -> AppResult<Self> {
        validate_id(id)?;
        let mut receiver = lock(&receiver.0)?;
        let path = match receiver.finish(id) {
            Ok(path) => path,
            Err(e) => {
                receiver.clear(id);
                return Err(runtime_io("finish staged recovery archive", e));
            }
        };
        let result = Self::copy_pinned(&path, pin);
        receiver.clear(id);
        result
    }
}
/// Startup hook: bundled resolves fixed resource names + a compile-time pin;
/// offline resolves the existing staged UUID + verified preview/pin. No free
/// path/digest/version is supplied by the frontend. No default-disabled bundled.
pub trait SourceResolver: Send + Sync {
    fn resolve(
        &self,
        request: &RuntimeRecoveryPrepareRequest,
        context: &RecoveryContext,
    ) -> AppResult<ResolvedRecoverySource>;
}

/// Trusted L0 OS evidence. Stopped requires positive same-user/H/service absence
/// evidence. Missing records, stale/reused PID, foreign SID/uid and unreachable
/// ports are Unknown, including when RuntimeHandle cannot resolve a broken CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceOwnerProof {
    Stopped,
    Owned { identity: String, pid: u32 },
    Unknown,
}
pub trait ServiceOwnerProofProvider: Send + Sync {
    /// Read actual H/service/OS state; never launch unresolved/damaged ocx.
    fn query(&self, context: &ProcessContext) -> AppResult<ServiceOwnerProof>;
    /// Recheck SID/uid + PID creation identity at the targeted stop. Never stop
    /// by name/port or a bare PID; the supplied identity came from core preview.
    fn stop_owned(&self, context: &ProcessContext, identity: &str, pid: u32) -> AppResult<()>;
}
struct ServiceBoundary<'a> {
    context: &'a ProcessContext,
    proof: &'a dyn ServiceOwnerProofProvider,
}
impl ServiceBoundary<'_> {
    fn attest(&self, pid: u32) -> AppResult<()> {
        // Called inside spawn_blocking; use the current Tauri runtime for HTTP.
        tauri::async_runtime::block_on(async {
            let management = Management::bind(self.context)
                .await
                .map_err(|e| error(&e.code, e.message))?;
            management
                .verify_binding()
                .map_err(|e| error(&e.code, e.message))?;
            let path = self.context.opencodex_home.join("runtime-port.json");
            safety::check_path(&self.context.opencodex_home, &path, false)
                .map_err(|e| runtime_io("inspect recovery service record", e))?;
            let record: serde_json::Value = serde_json::from_slice(
                &safety::read_file(&path, 4096)
                    .map_err(|e| runtime_io("read recovery service record", e))?,
            )
            .map_err(|_| error("recovery_service_unknown", "H 的运行身份不可读"))?;
            if record["pid"].as_u64() != Some(u64::from(pid)) {
                return Err(error(
                    "recovery_service_changed",
                    "OS 归属与证明服务 PID 不一致",
                ));
            }
            management
                .verify_binding()
                .map_err(|e| error(&e.code, e.message))
        })
    }
    fn snapshot_inner(&self) -> AppResult<RecoveryServiceState> {
        match self.proof.query(self.context)? {
            ServiceOwnerProof::Stopped => Ok(RecoveryServiceState::Stopped),
            ServiceOwnerProof::Unknown => Ok(RecoveryServiceState::Unknown),
            ServiceOwnerProof::Owned { identity, pid } if !identity.is_empty() && pid != 0 => {
                self.attest(pid)?;
                if self.proof.query(self.context)?
                    != (ServiceOwnerProof::Owned {
                        identity: identity.clone(),
                        pid,
                    })
                {
                    return Err(error(
                        "recovery_service_changed",
                        "服务用户或进程创建身份已改变",
                    ));
                }
                Ok(RecoveryServiceState::Owned { identity })
            }
            _ => Ok(RecoveryServiceState::Unknown),
        }
    }
}
fn service_error(value: AppError) -> InstallError {
    InstallError::Recovery {
        code: "recovery_service_proof_failed",
        detail: value.to_string(),
    }
}
impl RecoveryServiceBoundary for ServiceBoundary<'_> {
    fn snapshot(&self) -> Result<RecoveryServiceState, InstallError> {
        self.snapshot_inner().map_err(service_error)
    }
    fn stop_owned(&self, identity: &str) -> Result<(), InstallError> {
        let ServiceOwnerProof::Owned {
            identity: current,
            pid,
        } = self.proof.query(self.context).map_err(service_error)?
        else {
            return Err(InstallError::Recovery {
                code: "recovery_service_changed",
                detail: "停止前服务归属不再有效".into(),
            });
        };
        if current != identity
            || self.snapshot()?
                != (RecoveryServiceState::Owned {
                    identity: identity.into(),
                })
        {
            return Err(InstallError::Recovery {
                code: "recovery_service_changed",
                detail: "停止前服务身份已变化".into(),
            });
        }
        self.proof
            .stop_owned(self.context, identity, pid)
            .map_err(service_error)
    }
}

struct Session {
    operation: Option<RuntimeRecoveryOperation>,
    material: Option<ResolvedRecoverySource>,
    status: RuntimeRecoveryStatus,
    busy: bool,
}
struct RecoverySessions {
    context: RecoveryContext,
    install: SharedRuntimeInstall,
    sources: Arc<dyn SourceResolver>,
    services: Arc<dyn ServiceOwnerProofProvider>,
    sessions: Mutex<BTreeMap<String, Arc<Mutex<Session>>>>,
    gate: Arc<crate::infrastructure::storage_writers::WriterGate>,
}
#[derive(Clone)]
pub struct SharedRuntimeRecoveryV2(Arc<RecoverySessions>);
impl SharedRuntimeRecoveryV2 {
    pub fn new(
        context: RecoveryContext,
        install: SharedRuntimeInstall,
        sources: Arc<dyn SourceResolver>,
        services: Arc<dyn ServiceOwnerProofProvider>,
    ) -> Self {
        Self(Arc::new(RecoverySessions {
            context,
            install,
            sources,
            services,
            sessions: Mutex::new(BTreeMap::new()),
            gate: crate::infrastructure::storage_writers::global(),
        }))
    }
    fn session(&self, id: &str) -> AppResult<Arc<Mutex<Session>>> {
        validate_id(id)?;
        lock(&self.0.sessions)?.get(id).cloned().ok_or_else(|| {
            error(
                "recovery_session_missing",
                "此操作不在当前进程私有会话中；历史状态只读，请重新预览",
            )
        })
    }
    fn service(&self) -> ServiceBoundary<'_> {
        ServiceBoundary {
            context: &self.0.context.process,
            proof: self.0.services.as_ref(),
        }
    }
    fn require_owner_proof(&self) -> AppResult<()> {
        if self.service().snapshot_inner()? == RecoveryServiceState::Unknown {
            return Err(error(
                "recovery_service_unknown",
                "无法确认当前用户、H 与实际服务归属；未查询损坏程序，未创建备份或停止服务",
            ));
        }
        Ok(())
    }
    fn prepare_sync(
        &self,
        request: RuntimeRecoveryPrepareRequest,
    ) -> AppResult<RuntimeRecoveryV2Status> {
        request.validate()?;
        let _storage = self.0.gate.admit()?;
        let _lease = self
            .0
            .install
            .acquire()
            .ok_or_else(|| error("install_in_progress", "已有运行程序操作正在进行"))?;
        let material = self.0.sources.resolve(&request, &self.0.context)?;
        self.require_owner_proof()?;
        let context = &self.0.context;
        let (node, npm) =
            crate::modules::runtime::install::managed_dependency_paths(context.installation_root());
        let runner = SystemNpmRunner::new(
            Some(context.home.clone()),
            context.data_root.join("cache/npm"),
        );
        let probe = RealVersionProbe::default();
        let mut installer = RuntimeInstaller::new(
            &context.data_root,
            &runner,
            Some(context.home.clone()),
            &NullProgressSink,
            &probe,
            Some(node),
        );
        installer.installation_root = Some(context.installation_root().into());
        let operation = super::runtime::prepare_runtime_recovery(
            &installer,
            material.source.clone(),
            npm,
            &self.service(),
        )?;
        let status = operation.status();
        let result = response(status.clone())?;
        let mut sessions = lock(&self.0.sessions)?;
        // One retained unconfirmed source. Active workers are never evicted.
        for session in sessions.values() {
            let mut old = lock(session)?;
            if !old.busy && old.status.phase == RuntimeRecoveryPhase::Prepared {
                if let Some(mut op) = old.operation.take() {
                    old.status = op.cancel().map_err(install_error)?;
                }
                old.material.take();
            }
        }
        if sessions.len() >= 128 {
            sessions.retain(|_, s| {
                s.lock().map_or(true, |s| {
                    s.busy || s.status.phase == RuntimeRecoveryPhase::Prepared
                })
            });
        }
        sessions.insert(
            status.preview.operation_id.clone(),
            Arc::new(Mutex::new(Session {
                operation: Some(operation),
                material: Some(material),
                status,
                busy: false,
            })),
        );
        Ok(result)
    }
    fn status_sync(&self, id: &str) -> AppResult<RuntimeRecoveryV2Status> {
        validate_id(id)?;
        let session = lock(&self.0.sessions)?.get(id).cloned();
        if let Some(session) = session {
            let (busy, cached) = {
                let session = lock(&session)?;
                (session.busy, session.status.clone())
            };
            if !busy {
                return response(cached);
            }
            // No invented Applying before the core's first journal publication.
            let journal = self
                .0
                .context
                .data_root
                .join("manager-state/runtime-recovery")
                .join(format!("{id}.json"));
            safety::check_path(&self.0.context.data_root, &journal, true)
                .map_err(|e| runtime_io("inspect recovery status journal", e))?;
            if !journal
                .try_exists()
                .map_err(|e| file_error("inspect recovery status journal", e))?
            {
                return response(cached);
            }
        }
        response(
            RuntimeRecoveryOperation::read_status(&self.0.context.data_root, id)
                .map_err(install_error)
                .map_err(|e| runtime_io("read recovery status journal", e))?,
        )
    }
    fn cancel_sync(&self, id: &str) -> AppResult<RuntimeRecoveryV2Status> {
        let session = self.session(id)?;
        let mut session = lock(&session)?;
        if session.busy {
            return Err(error(
                "recovery_not_cancellable",
                "恢复线程已接管操作，不能宣称撤回",
            ));
        }
        let operation = session
            .operation
            .as_mut()
            .ok_or_else(|| error("recovery_not_cancellable", "操作已终结或仅有历史状态"))?;
        session.status = operation.cancel().map_err(install_error)?;
        session.operation.take();
        session.material.take();
        response(session.status.clone())
    }
    fn apply_sync(
        &self,
        request: RuntimeRecoveryApplyRequest,
        manager_state: &super::update::SharedUpdateStatus,
    ) -> AppResult<RuntimeRecoveryV2Status> {
        let session = self.session(&request.operation_id)?;
        let (mut operation, material) = {
            let mut session = lock(&session)?;
            if session.busy
                || session.status.phase != RuntimeRecoveryPhase::Prepared
                || session.operation.is_none()
            {
                return Err(error(
                    "recovery_not_prepared",
                    "恢复操作已开始或终结，请重新预览",
                ));
            }
            if response(session.status.clone())?.confirmation_digest != request.confirmation_digest
            {
                if let Some(mut op) = session.operation.take() {
                    session.status = op.cancel().map_err(install_error)?;
                }
                session.material.take();
                return Err(error(
                    "recovery_confirmation_required",
                    "confirmationDigest 与本次服务端冻结预览不一致",
                ));
            }
            let operation = session
                .operation
                .take()
                .ok_or_else(|| error("recovery_session_missing", "缺少后端恢复对象"))?;
            session.busy = true;
            (operation, session.material.take())
        };
        let mut work = SessionWork {
            session,
            operation: &mut operation,
            material,
            _mutation: None,
            _storage: None,
        };
        // Normal installer ordering. These guards live in the blocking worker.
        let admission = (|| {
            let storage = self.0.gate.admit()?;
            let mutation = {
                let update = lock(manager_state)?;
                if update.installing || update.pending_restart.is_some() {
                    return Err(error(
                        "desktop_update_in_progress",
                        "Desktop 更新占用运行程序变更边界",
                    ));
                }
                self.0
                    .install
                    .acquire()
                    .ok_or_else(|| error("install_in_progress", "已有运行程序操作正在进行"))?
            };
            Ok((storage, mutation))
        })();
        let (_storage, mutation) = match admission {
            Ok(leases) => leases,
            Err(e) => {
                work.record_error(&e)?;
                return Err(e);
            }
        };
        let cancel = mutation.cancel.clone();
        work._storage = Some(_storage);
        work._mutation = Some(mutation);
        let context = &self.0.context;
        let result: AppResult<()> = (|| {
            self.require_owner_proof()?;
            // Re-preview under the shared mutation lease before any backup or
            // receipt writes. Core repeats its snapshot check at actual apply.
            let (node, npm) = crate::modules::runtime::install::managed_dependency_paths(
                context.installation_root(),
            );
            let runner = SystemNpmRunner::new(
                Some(context.home.clone()),
                context.data_root.join("cache/npm"),
            );
            let probe = RealVersionProbe::default();
            let mut installer = RuntimeInstaller::new(
                &context.data_root,
                &runner,
                Some(context.home.clone()),
                &NullProgressSink,
                &probe,
                Some(node),
            );
            installer.installation_root = Some(context.installation_root().into());
            let material = work
                .material
                .as_ref()
                .ok_or_else(|| error("recovery_source_missing", "私有恢复源已释放，请重新预览"))?;
            let mut current = super::runtime::prepare_runtime_recovery(
                &installer,
                material.source.clone(),
                npm,
                &self.service(),
            )?
            .status()
            .preview;
            let frozen = work.operation.status().preview;
            current.operation_id.clone_from(&frozen.operation_id);
            current.retained_prefix.clone_from(&frozen.retained_prefix);
            current.metadata_backup.clone_from(&frozen.metadata_backup);
            if current != frozen {
                return Err(error(
                    "recovery_snapshot_changed",
                    "程序、来源或服务基准已改变，请重新预览",
                ));
            }
            {
                let guard = manager::acquire_preferences_transaction(&context.data_root)?;
                protection::reconcile_pending(&context.data_root, &guard)?;
            }
            let guard = manager::begin_preferences_protection(&context.data_root, true)?;
            let preview = work.operation.status().preview;
            let mut receipt = PanelProtection::arm_for_source(
                &context.data_root,
                &guard,
                &preview.prefix,
                &preview.selected_version,
                Some(work.operation.dependency_paths().0),
                InstallSourceKind::Offline,
            )?;
            let runner = SystemNpmRunner::new(
                Some(context.home.clone()),
                context.data_root.join("cache/npm"),
            );
            let probe = RealVersionProbe::default();
            let mut installer = RuntimeInstaller::new(
                &context.data_root,
                &runner,
                Some(context.home.clone()),
                &NullProgressSink,
                &probe,
                Some(work.operation.dependency_paths().0.into()),
            );
            installer.installation_root = Some(context.installation_root().into());
            installer.cancel = cancel;
            installer.attempt_id = receipt.as_ref().map(|r| r.attempt_id());
            // Translate UI digest to core's ID confirmation only in the backend.
            let applied = installer
                .apply_recovery(work.operation, &request.operation_id, &self.service())
                .map_err(install_error);
            match applied {
                Ok(_) => {
                    context.handle.refresh();
                    if let Some(receipt) = receipt.as_mut() {
                        receipt.reconcile(&context.data_root, &guard)?;
                    }
                    Ok(())
                }
                Err(e) => {
                    if let Some(receipt) = receipt {
                        receipt.discard(&context.data_root)?;
                    }
                    Err(e)
                }
            }
        })();
        // Core owns durable phases. Pre-core rejection cancels only the prepared
        // capability and deletes its owned temporary material.
        if let Err(e) = &result {
            work.record_error(e)?;
        }
        let status = work.operation.status();
        drop(work);
        result?;
        response(status)
    }
}
struct SessionWork<'a> {
    session: Arc<Mutex<Session>>,
    operation: &'a mut RuntimeRecoveryOperation,
    material: Option<ResolvedRecoverySource>,
    // Field order releases temporary material before mutation/storage leases.
    _mutation: Option<crate::state::RuntimeMutationLease>,
    _storage: Option<crate::infrastructure::storage_writers::Admission>,
}
impl SessionWork<'_> {
    fn record_error(&mut self, error: &AppError) -> AppResult<()> {
        let mut session = lock(&self.session)?;
        if self.operation.status().phase == RuntimeRecoveryPhase::Prepared {
            self.operation.cancel().map_err(install_error)?;
        }
        session.status = self.operation.status();
        session.status.detail = Some(error.to_string());
        Ok(())
    }
}
impl Drop for SessionWork<'_> {
    fn drop(&mut self) {
        if let Ok(mut session) = self.session.lock() {
            let detail = session.status.detail.take();
            session.status = self.operation.status();
            session.status.detail = session.status.detail.take().or(detail);
            session.busy = false;
        }
    }
}

#[tauri::command]
pub async fn runtime_recovery_v2_prepare(
    request: RuntimeRecoveryPrepareRequest,
    recovery: tauri::State<'_, SharedRuntimeRecoveryV2>,
) -> AppResult<RuntimeRecoveryV2Status> {
    let recovery = recovery.inner().clone();
    super::run_readonly("prepare runtime recovery", move || {
        recovery.prepare_sync(request)
    })
    .await
}
#[tauri::command]
pub async fn runtime_recovery_v2_apply(
    request: RuntimeRecoveryApplyRequest,
    recovery: tauri::State<'_, SharedRuntimeRecoveryV2>,
    updater: tauri::State<'_, super::update::SharedUpdateStatus>,
) -> AppResult<RuntimeRecoveryV2Status> {
    let recovery = recovery.inner().clone();
    let updater = updater.inner().clone();
    super::run_readonly("apply runtime recovery", move || {
        recovery.apply_sync(request, &updater)
    })
    .await
}
#[tauri::command]
pub async fn runtime_recovery_v2_status(
    operation_id: String,
    recovery: tauri::State<'_, SharedRuntimeRecoveryV2>,
) -> AppResult<RuntimeRecoveryV2Status> {
    let recovery = recovery.inner().clone();
    super::run_readonly("read runtime recovery status", move || {
        recovery.status_sync(&operation_id)
    })
    .await
}
#[tauri::command]
pub async fn runtime_recovery_v2_cancel(
    operation_id: String,
    recovery: tauri::State<'_, SharedRuntimeRecoveryV2>,
) -> AppResult<RuntimeRecoveryV2Status> {
    let recovery = recovery.inner().clone();
    super::run_readonly("cancel prepared runtime recovery", move || {
        recovery.cancel_sync(&operation_id)
    })
    .await
}

#[cfg(test)]
#[path = "runtime_recovery_v2_tests.rs"]
mod tests;
