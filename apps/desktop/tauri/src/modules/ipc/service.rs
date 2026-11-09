//! MOD-12 IPC 委托执行与命令验证。
//!
//! CLI 是运行中 GUI 的客户端：只读取共享快照，生命周期动作复用共享
//! runner；同步由持有存储与同步准入的后台执行者完成核验、反馈及审计。

use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::modules::ipc::{
    validate_request, IpcCommand, IpcError, IpcErrorCode, IpcRequest, IpcResponse,
};
use crate::modules::model_config::{
    sync_service_v2::{
        CatalogV2, ConfirmRequestV2, PreviewRequestV2, ServicePlanV2, ServiceSecretsV2,
        SyncServiceV2,
    },
    sync_v2::types as sync_types,
};
use crate::modules::process::{LifecycleAction, LifecycleResult, ProcessRunner};
use crate::types::runtime_status::StatusSnapshotDto;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct IpcStatusData {
    pub matrix: crate::types::status::StatusMatrix,
    pub facts: crate::modules::status::RuntimeFacts,
    pub port: Option<u16>,
    pub pid: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct IpcDataRootData {
    pub active_data_root: String,
    pub opencodex_home: String,
    pub home_mode: String,
    pub migration_status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct IpcListData<T> {
    pub items: Vec<T>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct IpcMessageData {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct IpcExportData {
    pub path: String,
    pub backup_id: Option<String>,
    pub document_sha256: String,
    pub format_version: u32,
    pub sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct IpcUpdateData {
    pub status: String,
    pub available_version: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct IpcImportData {
    pub backup_id: String,
    pub document_sha256: String,
    pub format_version: u32,
    pub applied_sections: Vec<String>,
    pub skipped_sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct IpcSyncData {
    pub snapshot_id: String,
    pub artifacts: usize,
    pub direction: &'static str,
}

#[derive(Debug, Clone)]
pub struct IpcDependencies {
    pub process_context: crate::state::ProcessContext,
    pub active_data_root: std::path::PathBuf,
    /// Fixed current-user D captured from SharedDataRootAnchor, never active D.
    pub app_data_root: std::path::PathBuf,
    /// 用户主目录：同步要把 Skills 软链接投射到各客户端的真实目录。
    pub home: std::path::PathBuf,
    pub opencodex_home: std::path::PathBuf,
    pub external_home: Option<std::path::PathBuf>,
    pub current_version: &'static str,
    pub runtime_log: crate::infrastructure::runtime_log::RuntimeLog,
}

struct BindingAccess<'a> {
    anchor: &'a std::path::Path,
    installation: &'a std::path::Path,
    boundary: Option<&'a std::path::Path>,
    locks: &'a [crate::modules::instance::AppInstanceLock],
    update_pending: bool,
}

#[cfg(test)]
struct TestBindingContext {
    startup: crate::modules::data_root::bootstrap::StartupResolution,
    installation: std::path::PathBuf,
    update_pending: bool,
}

#[derive(Clone)]
pub struct IpcService {
    collector: Arc<
        Mutex<
            crate::modules::status::StatusCollector<
                crate::infrastructure::status_source::OfficialStatusSource,
            >,
        >,
    >,
    runner: Arc<Mutex<crate::modules::process::ControlledProcessRunner>>,
    dependencies: IpcDependencies,
    writers: Arc<crate::infrastructure::storage_writers::WriterGate>,
    sync_operations: Arc<tokio::sync::Mutex<()>>,
    /// One instance per fixed GUI process D/H; clones retain prepared sessions.
    /// Construct under the owned storage/sync admission, never at an endpoint.
    sync_v2: Arc<Mutex<Option<Arc<SyncServiceV2>>>>,
    runtime_mutations: crate::state::SharedRuntimeInstall,
    app: Option<tauri::AppHandle>,
    #[cfg(test)]
    binding_context: Option<Arc<Mutex<TestBindingContext>>>,
}

impl IpcService {
    pub fn new(
        collector: Arc<
            Mutex<
                crate::modules::status::StatusCollector<
                    crate::infrastructure::status_source::OfficialStatusSource,
                >,
            >,
        >,
        runner: Arc<Mutex<crate::modules::process::ControlledProcessRunner>>,
        dependencies: IpcDependencies,
    ) -> Self {
        let writers = crate::infrastructure::storage_writers::global();
        Self {
            collector,
            runner,
            dependencies,
            writers,
            sync_operations: crate::commands::sync::sync_operation_gate(),
            sync_v2: Arc::new(Mutex::new(None)),
            runtime_mutations: crate::state::SharedRuntimeInstall::default(),
            app: None,
            #[cfg(test)]
            binding_context: None,
        }
    }

    /// Bind the running GUI's shared status and notification store. The CLI must
    /// never instantiate its own GUI state or a second notification publisher.
    pub(crate) fn with_app_handle(mut self, app: tauri::AppHandle) -> Self {
        use tauri::Manager;
        self.runtime_mutations = app
            .state::<crate::state::SharedRuntimeInstall>()
            .inner()
            .clone();
        self.dependencies.app_data_root =
            app.state::<crate::state::SharedDataRootAnchor>().0.clone();
        self.dependencies.active_data_root = app.state::<crate::state::SharedDataRoot>().0.clone();
        let context = app.state::<crate::state::SharedProcessContext>();
        self.dependencies.opencodex_home = context.opencodex_home.clone();
        self.dependencies.process_context = context.inner().clone();
        self.app = Some(app);
        self
    }

    #[cfg(test)]
    pub fn for_tests() -> Self {
        Self::for_tests_with(
            std::path::PathBuf::from("/private/tmp/fixture-root"),
            std::path::PathBuf::from("/private/tmp/fixture-root"),
        )
    }

    #[cfg(test)]
    pub fn for_tests_with(
        active_data_root: std::path::PathBuf,
        app_data_root: std::path::PathBuf,
    ) -> Self {
        let collector = Arc::new(Mutex::new(crate::modules::status::StatusCollector::new(
            crate::infrastructure::status_source::OfficialStatusSource::new(
                crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(
                    "/private/tmp/missing-ocx",
                ),
                std::path::PathBuf::from("/private/tmp"),
                crate::modules::process::EnvironmentPolicy {
                    opencodex_home: std::path::PathBuf::from("/private/tmp/opencodex-home"),
                    ..Default::default()
                },
            ),
        )));
        let mut service = Self::new(
            collector,
            Arc::new(Mutex::new(
                crate::modules::process::ControlledProcessRunner::new(std::path::PathBuf::from(
                    "/private/tmp",
                )),
            )),
            IpcDependencies {
                process_context: crate::state::ProcessContext {
                    runtime:
                        crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(
                            "/private/tmp/missing-ocx",
                        ),
                    working_directory: std::path::PathBuf::from("/private/tmp"),
                    opencodex_home: std::path::PathBuf::from("/private/tmp/opencodex-home"),
                },
                active_data_root: active_data_root.clone(),
                app_data_root: app_data_root.clone(),
                home: std::path::PathBuf::from("/private/tmp"),
                runtime_log: crate::infrastructure::runtime_log::RuntimeLog::new(&app_data_root),
                opencodex_home: std::path::PathBuf::from("/private/tmp/opencodex-home"),
                external_home: None,
                current_version: "0.1.0",
            },
        );
        service.writers = Arc::new(crate::infrastructure::storage_writers::WriterGate::default());
        service.sync_operations = Arc::new(tokio::sync::Mutex::new(()));
        service
    }

    pub async fn execute(&mut self, request: IpcRequest) -> IpcResponse<serde_json::Value> {
        let started = chrono::Utc::now();
        let request = match validate_request(request) {
            Ok(value) => value,
            Err(response) => return response,
        };
        // Unsupported public operations return before admissions, status probes,
        // filesystem audits, environment secrets or legacy container reads.
        if matches!(request.command, IpcCommand::Export | IpcCommand::Import)
            || (request.command == IpcCommand::DataRootSwitch
                && request
                    .args
                    .get("migrate")
                    .is_some_and(|mode| mode == "true"))
        {
            let mut response =
                owned_response(request.request_id, Err(IpcErrorCode::ValidationFailed));
            response.error.as_mut().expect("error response").message =
                "unsupported: v2 permits initialized aggregate-root selection only; migration and copying are disabled.".into();
            return response;
        }
        if request.command == IpcCommand::DataRootSwitch
            && request
                .args
                .get("migrate")
                .is_some_and(|mode| mode != "false")
        {
            return owned_response(request.request_id, Err(IpcErrorCode::ValidationFailed));
        }
        // A disconnected/cancelled observer must not cancel an admitted mutation or
        // release its old-root admission before terminal feedback and audit.
        if request.command == IpcCommand::SyncRun {
            // CLI sync has no authority-bearing scope/head/password arguments.
            // Only the GUI's persisted scope and backend catalog may supply them.
            if !request.args.is_empty() || request.secret.is_some() {
                return owned_response(request.request_id, Err(IpcErrorCode::ValidationFailed));
            }
            let request_id = request.request_id.clone();
            let mut response = owned_response(request_id, self.sync_run(request, started).await);
            if let Some(error) = response.error.as_mut() {
                if error.code == IpcErrorCode::PassphraseRequired {
                    error.message = sync_v2_ipc_error_message(error.code).into();
                }
            }
            return response;
        }
        if request.command == IpcCommand::BackupCreate {
            let request_id = request.request_id.clone();
            return owned_response(request_id, self.create_backup(request, started).await);
        }
        if matches!(
            request.command,
            IpcCommand::Start | IpcCommand::Stop | IpcCommand::Restart
        ) {
            let request_id = request.request_id.clone();
            return owned_response(request_id, self.run_lifecycle(request, started).await);
        }
        if request.command == IpcCommand::UpdateCheck {
            let request_id = request.request_id.clone();
            return owned_response(request_id, self.update_check(request, started).await);
        }
        let request_id = request.request_id.clone();
        owned_response(request_id, self.query_or_binding(request, started).await)
    }

    async fn query_or_binding(
        &self,
        request: IpcRequest,
        started: chrono::DateTime<chrono::Utc>,
    ) -> Result<serde_json::Value, IpcErrorCode> {
        let service = self.clone();
        let worker_request = request.clone();
        if request.command == IpcCommand::DataRootSwitch {
            run_ipc_binding_owned(
                self.writers.clone(),
                self.runtime_mutations.clone(),
                self.dependencies.active_data_root.clone(),
                request,
                started,
                move |saved_pending| service.switch_data_root(&worker_request.args, saved_pending),
            )
            .await
        } else {
            run_ipc_query_owned(
                self.writers.clone(),
                self.dependencies.active_data_root.clone(),
                request,
                started,
                move || match worker_request.command {
                    IpcCommand::Status => service.read_status(),
                    IpcCommand::DataRootShow => service.read_data_root(),
                    IpcCommand::BackupList => service.list_backups(),
                    _ => Err(IpcErrorCode::ValidationFailed),
                },
            )
            .await
        }
    }

    fn read_status(&self) -> Result<serde_json::Value, IpcErrorCode> {
        let mut guard = self
            .collector
            .lock()
            .map_err(|_| IpcErrorCode::InternalError)?;
        let _ = guard.refresh();
        let snapshot = crate::types::runtime_status::snapshot_from_matrix(
            guard.matrix(),
            guard.facts(),
            guard.port(),
            guard.pid().map(|pid| pid.to_string()),
            crate::types::runtime_status::StatusSource::Live,
        );
        serde_json::to_value(IpcStatusData::from_snapshot(&snapshot))
            .map_err(|_| IpcErrorCode::InternalError)
    }

    fn read_data_root(&self) -> Result<serde_json::Value, IpcErrorCode> {
        serde_json::to_value(IpcDataRootData {
            active_data_root: self.dependencies.active_data_root.display().to_string(),
            opencodex_home: self.dependencies.opencodex_home.display().to_string(),
            home_mode: if self.dependencies.external_home.is_some() {
                "external"
            } else {
                "inside"
            }
            .to_string(),
            migration_status: "idle".to_string(),
        })
        .map_err(|_| IpcErrorCode::InternalError)
    }

    // Called only inside the owned query worker: both cooperative lock writes
    // and the bounded manifest scan remain admitted through the final audit.
    fn list_backups(&self) -> Result<serde_json::Value, IpcErrorCode> {
        let records =
            crate::modules::backup::manager::list_records(&self.dependencies.active_data_root)
                .map_err(map_app_error)?;
        let items: Vec<serde_json::Value> = records
            .iter()
            .map(|record| {
                serde_json::json!({
                    "backup_id": record.manifest.backup_id,
                    "created_at": record.manifest.created_at,
                    "target_path": record.manifest.target_path,
                    "bytes": record.manifest.bytes,
                    "restorable": record.manifest.restorable,
                })
            })
            .collect();
        serde_json::to_value(IpcListData { items }).map_err(|_| IpcErrorCode::InternalError)
    }

    async fn update_check(
        &self,
        request: IpcRequest,
        started: chrono::DateTime<chrono::Utc>,
    ) -> Result<serde_json::Value, IpcErrorCode> {
        let app = self.app.clone();
        run_ipc_update_owned(
            self.writers.clone(),
            self.dependencies.active_data_root.clone(),
            request,
            started,
            move || async move {
                use tauri::Manager;
                let app = app.ok_or(IpcErrorCode::TargetNotFound)?;
                let status = app
                    .state::<crate::commands::update::SharedUpdateStatus>()
                    .inner()
                    .clone();
                let result = crate::commands::update::check_for_update_shared(
                    crate::commands::event_delivery::QueryTrigger::User,
                    status,
                    app,
                )
                .await
                .map_err(map_backup_worker_error)?;
                project_ipc_update(result)
            },
        )
        .await
    }

    fn switch_data_root(
        &self,
        args: &BTreeMap<String, String>,
        saved_pending: &mut bool,
    ) -> Result<serde_json::Value, IpcErrorCode> {
        use tauri::Manager;
        {
            let mut collector = self
                .collector
                .lock()
                .map_err(|_| IpcErrorCode::TargetStateConflict)?;
            collector
                .refresh()
                .map_err(|_| IpcErrorCode::TargetStateConflict)?;
            if !matches!(
                collector.matrix().runtime,
                crate::types::status::RuntimeState::Stopped
                    | crate::types::status::RuntimeState::NotFound
            ) {
                return Err(IpcErrorCode::TargetStateConflict);
            }
        }
        let root = &self.dependencies.active_data_root;
        let _transaction = crate::modules::backup::manager::acquire_preferences_transaction(root)
            .map_err(map_app_error)?;
        crate::modules::runtime::protection::ensure_binding_ready(root).map_err(map_app_error)?;
        if let Some(app) = &self.app {
            let anchor = app.state::<crate::state::SharedDataRootAnchor>();
            let sandbox = app.state::<crate::state::SharedSandboxBoundary>();
            let runtime = app.state::<Arc<crate::modules::runtime::RuntimeHandle>>();
            let status = app.state::<crate::commands::update::SharedUpdateStatus>();
            let status = status
                .lock()
                .map_err(|_| IpcErrorCode::TargetStateConflict)?;
            let instances = app.state::<crate::state::InstanceState>();
            let locks = instances
                .locks
                .lock()
                .map_err(|_| IpcErrorCode::TargetStateConflict)?;
            return self.save_reference(
                args,
                saved_pending,
                BindingAccess {
                    anchor: &anchor.0,
                    installation: runtime.installation_root(),
                    boundary: sandbox.0.as_deref(),
                    locks: &locks,
                    update_pending: status.installing || status.pending_restart.is_some(),
                },
            );
        }
        #[cfg(test)]
        if let Some(context) = &self.binding_context {
            let context = context
                .lock()
                .map_err(|_| IpcErrorCode::TargetStateConflict)?;
            return self.save_reference(
                args,
                saved_pending,
                BindingAccess {
                    anchor: &context.startup.anchor,
                    installation: &context.installation,
                    boundary: context.startup.sandbox_boundary.as_deref(),
                    locks: &context.startup.locks,
                    update_pending: context.update_pending,
                },
            );
        }
        // A detached CLI cannot invent service ownership or startup locks.
        Err(IpcErrorCode::TargetStateConflict)
    }

    fn save_reference(
        &self,
        args: &BTreeMap<String, String>,
        saved_pending: &mut bool,
        access: BindingAccess<'_>,
    ) -> Result<serde_json::Value, IpcErrorCode> {
        use crate::commands::data_root::{
            require_current_locks, switch_data_root_with_paths, validate_reference_target,
            DataRootSwitchStatus,
        };
        if access.update_pending {
            return Err(IpcErrorCode::TargetStateConflict);
        }
        require_current_locks(
            access.locks,
            access.anchor,
            &self.dependencies.active_data_root,
        )
        .map_err(|_| IpcErrorCode::TargetStateConflict)?;
        let target =
            std::path::Path::new(args.get("target").ok_or(IpcErrorCode::ValidationFailed)?);
        validate_reference_target(target, access.boundary, access.installation)
            .map_err(|_| IpcErrorCode::ValidationFailed)?;
        let result = switch_data_root_with_paths(
            access.anchor,
            target,
            crate::modules::data_root::DataRootSwitchMode::ReferenceOnly,
        )
        .map_err(map_app_error)?;
        *saved_pending = result.reconciliation_required
            || result.config.as_ref().is_some_and(|config| {
                !crate::modules::data_root::same_runtime_directory(
                    std::path::Path::new(&config.active_data_root),
                    &self.dependencies.active_data_root,
                ) || !crate::modules::data_root::same_runtime_directory(
                    std::path::Path::new(&config.opencodex_home),
                    &self.dependencies.opencodex_home,
                )
            });
        if result.reconciliation_required {
            return Err(IpcErrorCode::TargetStateConflict);
        }
        if result.status == DataRootSwitchStatus::Blocked {
            return Err(IpcErrorCode::ValidationFailed);
        }
        serde_json::to_value(IpcMessageData {
            status: if *saved_pending {
                "restart_required"
            } else {
                "applied"
            }
            .into(),
            message: if *saved_pending {
                "Aggregate root reference saved; restart is required."
            } else {
                "Aggregate root already matches the running services."
            }
            .into(),
        })
        .map_err(|_| IpcErrorCode::InternalError)
    }

    async fn create_backup(
        &self,
        request: IpcRequest,
        started: chrono::DateTime<chrono::Utc>,
    ) -> Result<serde_json::Value, IpcErrorCode> {
        let root = self.dependencies.active_data_root.clone();
        let worker_root = root.clone();
        let app = self.app.clone();
        run_ipc_backup_owned(self.writers.clone(), root, request, started, move || {
            let outcome = crate::commands::backup::execute_preferences_backup(
                app.as_ref(),
                &worker_root,
                chrono::Utc::now(),
            )
            .map_err(map_app_error)?;
            // Preserve the existing CLI output; cleanup failure is diagnostic,
            // not a rewrite of the verified backup's success.
            serde_json::to_value(IpcMessageData {
                status: "backed_up".to_string(),
                message: outcome.backup.backup_id,
            })
            .map_err(|_| IpcErrorCode::InternalError)
        })
        .await
    }

    async fn sync_run(
        &self,
        request: IpcRequest,
        started: chrono::DateTime<chrono::Utc>,
    ) -> Result<serde_json::Value, IpcErrorCode> {
        let root = self.dependencies.active_data_root.clone();
        let home = self.dependencies.process_context.opencodex_home.clone();
        let app = self.app.clone();
        let service = self.sync_v2.clone();
        let runtime = tokio::runtime::Handle::current();
        let worker_request = request.clone();
        let worker_root = root.clone();
        run_ipc_sync_owned(
            self.writers.clone(),
            self.sync_operations.clone(),
            root,
            request,
            started,
            move || {
                // Validation already ran at execute(), but this boundary must
                // never turn a missing CLI confirmation into a GUI confirm.
                if !worker_request.confirm {
                    return Err(IpcErrorCode::RequireConfirm);
                }
                begin_ipc_sync_v2(app.as_ref(), &worker_request.request_id, started);
                let notification_candidate = ipc_sync_notification_candidate(&worker_root);
                let mut terminal = IpcSyncTerminalV2::default();
                let result = (|| {
                    let service = ipc_sync_service_v2(&service, worker_root.clone(), home)?;
                    runtime.block_on(execute_ipc_sync_v2(service.as_ref(), &mut terminal))
                })();
                finish_ipc_sync_v2(
                    app.as_ref(),
                    &worker_root,
                    &worker_request.request_id,
                    started,
                    &result,
                    terminal.conflict_observed,
                    notification_candidate.as_deref(),
                );
                result
            },
        )
        .await
    }

    async fn run_lifecycle(
        &self,
        request: IpcRequest,
        started: chrono::DateTime<chrono::Utc>,
    ) -> Result<serde_json::Value, IpcErrorCode> {
        let runner = self.runner.clone();
        let dependencies = self.dependencies.clone();
        let app = self.app.clone();
        let worker_request = request.clone();
        run_ipc_lifecycle_owned(
            self.writers.clone(),
            self.runtime_mutations.clone(),
            dependencies.active_data_root.clone(),
            request,
            started,
            move || {
                use tauri::Manager;
                let notifications = app.as_ref().map(|app| {
                    app.state::<crate::state::SharedNotificationStore>()
                        .inner()
                        .clone()
                });
                let before = notifications
                    .as_ref()
                    .and_then(|store| store.lock().ok().map(|store| store.clone()));
                let publisher = notifications.as_ref().map(|store| {
                    crate::commands::notifications::NotificationPublisher {
                        store,
                        data_root: &dependencies.active_data_root,
                    }
                });
                let result = execute_ipc_lifecycle(
                    runner.as_ref(),
                    &dependencies,
                    &worker_request,
                    publisher,
                );
                if let (Some(app), Some(store), Some(before)) =
                    (app.as_ref(), notifications.as_ref(), before)
                {
                    if store.lock().is_ok_and(|after| *after != before) {
                        let _ = crate::commands::event_delivery::emit_signal(
                            app,
                            crate::commands::notifications::NOTIFICATIONS_CHANGED_EVENT,
                            crate::modules::notifications::registry::Job::NotificationMutation,
                            crate::modules::notifications::registry::Trigger::Commit,
                            crate::modules::notifications::registry::Channel::Local,
                            (),
                        );
                    }
                }
                result
            },
        )
        .await
    }
}

impl IpcStatusData {
    pub fn from_snapshot(value: &StatusSnapshotDto) -> Self {
        Self {
            matrix: value.matrix,
            facts: value.facts.clone(),
            port: value.port,
            pid: value.pid.clone(),
        }
    }
}

fn map_app_error(error: crate::errors::AppError) -> IpcErrorCode {
    use crate::errors::AppError;
    match error {
        // 旧版容器缺口令可被调用方修复（提示输入原口令），必须区别于「执行失败」。
        AppError::PassphraseRequired => IpcErrorCode::PassphraseRequired,
        AppError::NotFound { .. } => IpcErrorCode::TargetNotFound,
        _ => IpcErrorCode::ExecutionFailed,
    }
}

fn map_backup_worker_error(error: crate::errors::AppError) -> IpcErrorCode {
    match error {
        crate::errors::AppError::FileSystem { operation, .. } if operation == "storage binding" => {
            IpcErrorCode::TargetStateConflict
        }
        other => map_app_error(other),
    }
}

fn ipc_sync_service_v2(
    slot: &Mutex<Option<Arc<SyncServiceV2>>>,
    root: std::path::PathBuf,
    home: std::path::PathBuf,
) -> Result<Arc<SyncServiceV2>, IpcErrorCode> {
    let mut slot = slot.lock().map_err(|_| IpcErrorCode::InternalError)?;
    if let Some(service) = slot.as_ref() {
        return Ok(service.clone());
    }
    let service = Arc::new(SyncServiceV2::production(root, home).map_err(map_sync_v2_failure)?);
    *slot = Some(service.clone());
    Ok(service)
}

/// No plaintext fallback, implicit head choice, legacy snapshot or engine call.
/// Catalog is required even with a saved scope: it proves current availability
/// and source options without replacing a deliberately empty saved selection.
#[derive(Default)]
struct IpcSyncTerminalV2 {
    conflict_observed: bool,
}
impl IpcSyncTerminalV2 {
    fn failure(&mut self, failure: sync_types::SyncFailure) -> IpcErrorCode {
        self.conflict_observed |= matches!(
            failure.code,
            sync_types::ErrorCode::SyncRemoteConflict
                | sync_types::ErrorCode::SyncGenerationAmbiguous
        );
        map_sync_v2_failure(failure)
    }
}
async fn execute_ipc_sync_v2(
    service: &SyncServiceV2,
    terminal: &mut IpcSyncTerminalV2,
) -> Result<serde_json::Value, IpcErrorCode> {
    let saved = service.read_scope().await.map_err(map_sync_v2_failure)?;
    let catalog = service
        .catalog(None)
        .await
        .map_err(|f| terminal.failure(f))?;
    terminal.conflict_observed = matches!(
        catalog.cloud_state,
        sync_types::RemoteState::Conflict | sync_types::RemoteState::InitializationAmbiguous
    ) || catalog.mcp_sources.iter().any(|sources| {
        !sources.differing_fields.is_empty()
            && !catalog
                .selected
                .selection
                .mcp_sources
                .iter()
                .any(|s| s.name == sources.name)
    });
    validate_ipc_sync_catalog(&catalog)?;
    if saved.as_ref().is_some_and(|scope| {
        serde_json::to_value(scope).ok() != serde_json::to_value(&catalog.selected).ok()
    }) {
        return Err(IpcErrorCode::TargetStateConflict);
    }
    let scope = catalog.selected;
    let preview = service
        .preview(
            PreviewRequestV2 {
                selection: scope.selection,
                mcp_clients: scope.mcp_clients,
                assets_reviewed: scope.assets_reviewed,
                selected_head: None,
                forced_direction: None,
                backup_before_update: true,
            },
            ServiceSecretsV2::default(),
        )
        .await
        .map_err(|f| terminal.failure(f))?;
    let (operation_id, kind) = match &preview.plan {
        ServicePlanV2::NoChange if preview.no_change => {
            // No remote/local application fact exists for a semantic no-op.
            service
                .cancel(preview.session_id)
                .await
                .map_err(map_sync_v2_failure)?;
            return Ok(
                serde_json::json!({"snapshot_id":"", "artifacts":0, "direction":"no_change", "status":"no_change"}),
            );
        }
        ServicePlanV2::Upload(plan) if !preview.no_change => {
            (plan.operation_id.clone(), sync_types::OperationKind::Upload)
        }
        ServicePlanV2::Download(plan) if !preview.no_change => (
            plan.operation_id.clone(),
            sync_types::OperationKind::Download,
        ),
        _ => return Err(IpcErrorCode::ExecutionFailed),
    };
    let fact = service
        .confirm(ConfirmRequestV2 {
            session_id: preview.session_id,
            plan_digest: preview.plan_digest,
        })
        .await
        .map_err(|f| terminal.failure(f))?;
    terminal.conflict_observed |= matches!(
        fact.relation.remote,
        sync_types::RemoteRelation::Conflict | sync_types::RemoteRelation::InitializationAmbiguous
    );
    project_ipc_sync_v2(&operation_id, kind, &fact)
}

fn validate_ipc_sync_catalog(c: &CatalogV2) -> Result<(), IpcErrorCode> {
    use sync_types::RemoteState;
    if !c.initialized
        || matches!(
            c.cloud_state,
            RemoteState::Conflict | RemoteState::InitializationAmbiguous
        )
    {
        return Err(IpcErrorCode::TargetStateConflict);
    }
    if !matches!(
        c.cloud_state,
        RemoteState::Empty | RemoteState::GenerationEmpty | RemoteState::Ready
    ) {
        return Err(IpcErrorCode::ExecutionFailed);
    }
    for sources in &c.mcp_sources {
        let selected = c
            .selected
            .selection
            .mcp_sources
            .iter()
            .find(|s| s.name == sources.name);
        // Initial all-selected must not silently omit conflicting definitions.
        if !sources.differing_fields.is_empty() && selected.is_none() {
            return Err(IpcErrorCode::TargetStateConflict);
        }
        if let Some(selected) = selected {
            let source = c
                .selected
                .mcp_clients
                .get(&selected.name)
                .ok_or(IpcErrorCode::TargetStateConflict)?;
            if !sources
                .sources
                .iter()
                .any(|s| &s.source == source && s.definition_digest == selected.definition_digest)
            {
                return Err(IpcErrorCode::TargetStateConflict);
            }
        }
    }
    for selected in &c.selected.selection.mcp_sources {
        if !c.entries.iter().any(|e| {
            e.kind == crate::modules::model_config::sync_service_v2::ObjectKindV2::Mcp
                && e.id == selected.name
                && (e.available_local || e.available_cloud)
        }) {
            return Err(IpcErrorCode::TargetStateConflict);
        }
    }
    Ok(())
}

fn project_ipc_sync_v2(
    id: &str,
    kind: sync_types::OperationKind,
    fact: &sync_types::OperationFact,
) -> Result<serde_json::Value, IpcErrorCode> {
    use sync_types::{LocalApplyStatus, OperationKind, Phase, RemoteRelation, StorageStatus};
    if fact.operation_id != id
        || fact.kind != kind
        || !uuid::Uuid::parse_str(id).is_ok_and(|u| u.to_string() == id && u.get_version_num() == 4)
    {
        return Err(IpcErrorCode::ExecutionFailed);
    }
    if let Some(failure) = &fact.failure {
        return Err(map_sync_v2_failure(failure.clone()));
    }
    if matches!(
        fact.relation.remote,
        RemoteRelation::Conflict
            | RemoteRelation::InitializationAmbiguous
            | RemoteRelation::GenerationChanged
    ) {
        return Err(IpcErrorCode::TargetStateConflict);
    }
    let (status, direction) = match kind {
        OperationKind::Upload
            if fact.phase == Phase::Settled
                && fact.storage_status == StorageStatus::Stored
                && fact
                    .commit
                    .as_ref()
                    .is_some_and(|commit| commit.operation_id == id)
                && fact.stored_at.is_some() =>
        {
            ("stored", "upload")
        }
        OperationKind::Download
            if fact.phase == Phase::Settled
                && fact.local_apply_status == LocalApplyStatus::Applied
                && fact.storage_status == StorageStatus::Stored
                && fact.commit.is_some() =>
        {
            ("download_applied", "download")
        }
        _ => return Err(IpcErrorCode::ExecutionFailed),
    };
    let mut data = serde_json::to_value(IpcSyncData {
        snapshot_id: fact.operation_id.clone(),
        // One verified v2 wrapper; never reinterpret unrelated model/resource
        // counts as old per-artifact transport or local application evidence.
        artifacts: 1,
        direction,
    })
    .map_err(|_| IpcErrorCode::InternalError)?;
    data["status"] = serde_json::json!(status);
    // Preserve independent storage/application/local/remote facts verbatim.
    data["operation"] = serde_json::to_value(fact).map_err(|_| IpcErrorCode::InternalError)?;
    Ok(data)
}

fn map_sync_v2_failure(failure: sync_types::SyncFailure) -> IpcErrorCode {
    use sync_types::ErrorCode as E;
    match failure.code {
        E::SyncPassphraseRequired | E::SyncPassphraseInvalid => IpcErrorCode::PassphraseRequired,
        E::SyncAuthRequired => IpcErrorCode::AccessDenied,
        E::SyncCollectionMissing => IpcErrorCode::TargetNotFound,
        E::SyncCancelled => IpcErrorCode::Cancelled,
        E::SyncGenerationAmbiguous
        | E::SyncGenerationChanged
        | E::SyncRemoteConflict
        | E::SyncSourceChanged
        | E::SyncScopeChanged
        | E::SyncEndpointChanged
        | E::SyncPlanChanged
        | E::SyncPlanExpired
        | E::SyncLocalChanged
        | E::SyncOperationCollision => IpcErrorCode::TargetStateConflict,
        E::SyncNetwork if failure.cause == Some(sync_types::Cause::Timeout) => {
            IpcErrorCode::Timeout
        }
        _ => IpcErrorCode::ExecutionFailed,
    }
}

fn sync_v2_ipc_error_message(code: IpcErrorCode) -> &'static str {
    if code == IpcErrorCode::PassphraseRequired {
        "Protected v2 configuration requires a passphrase; continue in the GUI."
    } else {
        crate::modules::ipc::error_message(code)
    }
}

/// Called inside the same admitted worker before execute_and_audit returns.
/// Notification delivery failure cannot rewrite a committed operation result.
fn finish_ipc_sync_v2(
    app: Option<&tauri::AppHandle>,
    root: &std::path::Path,
    request_id: &str,
    started: chrono::DateTime<chrono::Utc>,
    result: &Result<serde_json::Value, IpcErrorCode>,
    conflict_observed: bool,
    notification_candidate: Option<&[u8]>,
) {
    use crate::modules::notifications::registry::{Channel, Trigger};
    use crate::types::status::{ConnectionState, OperationState};
    use tauri::Manager;
    let Some(app) = app else {
        return;
    };
    if let Some(status) = app.try_state::<crate::state::SharedSyncStatus>() {
        if let Ok(mut run) = status.lock() {
            run.run_id = request_id.into();
            run.started_at = started;
            run.finished_at = Some(chrono::Utc::now());
            run.snapshot_id = result
                .as_ref()
                .ok()
                .and_then(|d| d["snapshot_id"].as_str())
                .filter(|id| !id.is_empty())
                .map(str::to_owned);
            run.items_done = result
                .as_ref()
                .ok()
                .and_then(|d| d["artifacts"].as_u64())
                .unwrap_or(0) as usize;
            run.items_total = run.items_done;
            run.failure_reason = result
                .as_ref()
                .err()
                .map(|e| sync_v2_ipc_error_message(*e).into());
            if result
                .as_ref()
                .ok()
                .is_some_and(|d| d["direction"] == "download")
            {
                run.direction = crate::modules::sync::SyncDirection::Download;
            } else {
                run.direction = crate::modules::sync::SyncDirection::Upload;
            }
            run.connection_state = match result {
                Ok(data)
                    if data["status"] == "no_change"
                        || (data["operation"]["relation"]["local"] == "observed_equal"
                            && data["operation"]["relation"]["remote"] == "observed_equal") =>
                {
                    ConnectionState::Synced
                }
                Ok(_) => ConnectionState::Disconnected, // Stored is not proof of current equality.
                Err(_) if conflict_observed => ConnectionState::Conflict,
                Err(_) => ConnectionState::Failed,
            };
            run.operation_state = if result.is_ok() {
                OperationState::Succeeded
            } else {
                OperationState::Failed
            };
        }
    }
    // A no-change observation is not a new upload/apply success event.
    if result
        .as_ref()
        .ok()
        .is_some_and(|d| d["status"] == "no_change")
    {
        return;
    }
    // Stable endpoint candidate allows a later verified success to resolve its
    // previous failure. Public configuration only; no credential retrieval.
    if ipc_sync_notification_candidate(root).as_deref() != notification_candidate {
        let _ = crate::infrastructure::runtime_log::RuntimeLog::new(root)
            .append_event("CLI v2 sync terminal endpoint changed; notification not projected");
        return;
    }
    let candidate = notification_candidate.unwrap_or(request_id.as_bytes());
    let event = if result.is_ok() {
        "sync-run-succeeded"
    } else {
        "sync-run-failed"
    };
    let identity = crate::commands::event_delivery::prepare(
        root,
        "sync-run-failed",
        Channel::Local,
        candidate,
    );
    crate::commands::event_delivery::publish(app, root, event, identity, Trigger::User);
    if conflict_observed
        && crate::modules::preferences::PreferencesStore::new(root)
            .load()
            .map(|p| p.sync_conflict_alerts)
            .unwrap_or(true)
    {
        let identity = crate::commands::event_delivery::prepare(
            root,
            "sync-conflict-detected",
            Channel::Local,
            candidate,
        );
        crate::commands::event_delivery::publish(
            app,
            root,
            "sync-conflict-detected",
            identity,
            Trigger::User,
        );
    }
}

fn ipc_sync_notification_candidate(root: &std::path::Path) -> Option<Vec<u8>> {
    crate::modules::sync::config::SyncConfigStore::new(root)
        .load()
        .ok()
        .and_then(|config| {
            config
                .active()
                .and_then(|endpoint| serde_json::to_vec(endpoint).ok())
        })
}

fn begin_ipc_sync_v2(
    app: Option<&tauri::AppHandle>,
    request_id: &str,
    started: chrono::DateTime<chrono::Utc>,
) {
    use tauri::Manager;
    if let Some(status) = app.and_then(|app| app.try_state::<crate::state::SharedSyncStatus>()) {
        if let Ok(mut run) = status.lock() {
            run.run_id = request_id.into();
            run.started_at = started;
            run.finished_at = None;
            run.connection_state = crate::types::status::ConnectionState::Connecting;
            run.operation_state = crate::types::status::OperationState::Validating;
            run.snapshot_id = None;
            run.items_total = 0;
            run.items_done = 0;
            run.failure_reason = None;
        }
    }
}

#[cfg(test)]
#[path = "service_sync_v2_tests.rs"]
mod sync_v2_tests;

/// 同步失败按「原因」映射退出码，而不是一律 `ExecutionFailed`。
#[cfg(test)]
fn map_sync_run_failure(failure: crate::modules::sync::runner::SyncRunFailure) -> IpcErrorCode {
    use crate::infrastructure::webdav_client::WebDavError;
    use crate::modules::sync::runner::SyncRunFailure;
    match failure {
        SyncRunFailure::NotConfigured => IpcErrorCode::ExecutionFailed,
        SyncRunFailure::ColdSync => IpcErrorCode::TargetStateConflict,
        SyncRunFailure::TargetLocked => IpcErrorCode::Timeout,
        SyncRunFailure::WebDav(_, WebDavError::Unauthorized) => IpcErrorCode::AccessDenied,
        SyncRunFailure::WebDav(_, WebDavError::NotFound) => IpcErrorCode::TargetNotFound,
        SyncRunFailure::WebDav(..) => IpcErrorCode::ExecutionFailed,
    }
}

/// The same owner holds the storage and sync admissions across execution,
/// terminal status/notification projection and the final redacted audit write.
async fn run_ipc_sync_owned(
    writers: Arc<crate::infrastructure::storage_writers::WriterGate>,
    operations: Arc<tokio::sync::Mutex<()>>,
    root: std::path::PathBuf,
    request: IpcRequest,
    started: chrono::DateTime<chrono::Utc>,
    task: impl FnOnce() -> Result<serde_json::Value, IpcErrorCode> + Send + 'static,
) -> Result<serde_json::Value, IpcErrorCode> {
    let audit = super::audit::AuditStore::with_writers(&root, writers.clone());
    crate::commands::sync::run_owned_sync_with_gates(
        writers,
        operations,
        "CLI sync run",
        move || Ok(execute_and_audit(&audit, &root, &request, started, task)),
    )
    .await
    .map_err(|error| match error {
        crate::errors::AppError::TargetLockTimeout { .. } => IpcErrorCode::Timeout,
        crate::errors::AppError::FileSystem { operation, .. } if operation == "storage binding" => {
            IpcErrorCode::TargetStateConflict
        }
        _ => IpcErrorCode::ExecutionFailed,
    })?
}

/// Querying updates writes schedule/cache and terminal events. The async owner
/// retains storage admission until an uncancellable blocking audit finishes.
/// The inner task captures panic without dropping the supervisor's admission.
async fn run_ipc_update_owned<F, Fut>(
    writers: Arc<crate::infrastructure::storage_writers::WriterGate>,
    root: std::path::PathBuf,
    request: IpcRequest,
    started: chrono::DateTime<chrono::Utc>,
    task: F,
) -> Result<serde_json::Value, IpcErrorCode>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<serde_json::Value, IpcErrorCode>> + Send + 'static,
{
    let admission = writers
        .admit()
        .map_err(|_| IpcErrorCode::TargetStateConflict)?;
    let audit = super::audit::AuditStore::with_writers(&root, writers);
    tokio::spawn(async move {
        let _admission = admission;
        let result = tokio::spawn(async move { task().await })
            .await
            .unwrap_or(Err(IpcErrorCode::InternalError));
        crate::commands::run_readonly("CLI update query audit", move || {
            Ok(execute_and_audit(&audit, &root, &request, started, || {
                result
            }))
        })
        .await
        .map_err(map_backup_worker_error)?
    })
    .await
    .map_err(|_| IpcErrorCode::InternalError)?
}

fn project_ipc_update(
    result: crate::types::update::CheckUpdateResultDto,
) -> Result<serde_json::Value, IpcErrorCode> {
    match result.status.as_str() {
        "available" | "up_to_date" => serde_json::to_value(IpcUpdateData {
            status: result.status,
            available_version: result.update.available_version,
            error: result.update.error,
        })
        .map_err(|_| IpcErrorCode::InternalError),
        "superseded" => Err(IpcErrorCode::TargetStateConflict),
        "failed" => Err(IpcErrorCode::ExecutionFailed),
        _ => Err(IpcErrorCode::InternalError),
    }
}

/// Metadata queries keep their optional old-root audit admission in the worker.
/// Backup listing creates a cooperative lock file, so it must refuse frozen roots.
/// The other read-only queries may still run after a pending binding save, without
/// writing audit to the old root. No synchronous refresh runs on the executor.
async fn run_ipc_query_owned(
    writers: Arc<crate::infrastructure::storage_writers::WriterGate>,
    root: std::path::PathBuf,
    request: IpcRequest,
    started: chrono::DateTime<chrono::Utc>,
    task: impl FnOnce() -> Result<serde_json::Value, IpcErrorCode> + Send + 'static,
) -> Result<serde_json::Value, IpcErrorCode> {
    let admission = writers.admit().ok();
    if request.command == IpcCommand::BackupList && admission.is_none() {
        return Err(IpcErrorCode::TargetStateConflict);
    }
    let audit = super::audit::AuditStore::with_writers(&root, writers);
    crate::commands::run_readonly("CLI query", move || {
        let admitted = admission.is_some();
        let _admission = admission;
        Ok(if admitted {
            execute_and_audit(&audit, &root, &request, started, task)
        } else {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(task))
                .unwrap_or(Err(IpcErrorCode::InternalError))
        })
    })
    .await
    .map_err(map_backup_worker_error)?
}

/// The exclusive old-root binding and runtime mutation leases live in the
/// blocking worker through persistence and final audit. A disconnected observer
/// cannot reopen writers after a saved reference. Even a later panic must retain
/// a known pending binding until restart; validation failure reopens admission.
async fn run_ipc_binding_owned(
    writers: Arc<crate::infrastructure::storage_writers::WriterGate>,
    mutations: crate::state::SharedRuntimeInstall,
    root: std::path::PathBuf,
    request: IpcRequest,
    started: chrono::DateTime<chrono::Utc>,
    task: impl FnOnce(&mut bool) -> Result<serde_json::Value, IpcErrorCode> + Send + 'static,
) -> Result<serde_json::Value, IpcErrorCode> {
    let lease = mutations
        .acquire()
        .ok_or(IpcErrorCode::TargetStateConflict)?;
    let binding = writers
        .freeze()
        .map_err(|_| IpcErrorCode::TargetStateConflict)?
        .ok_or(IpcErrorCode::TargetStateConflict)?;
    let audit = super::audit::AuditStore::with_writers(&root, writers);
    crate::commands::run_readonly("CLI data root switch", move || {
        let _lease = lease;
        // Settle storage before releasing the runtime lease, including unwind.
        // A closure capture otherwise outlives the worker's local lease.
        let binding = binding;
        let mut saved_pending = false;
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| task(&mut saved_pending)))
                .unwrap_or(Err(IpcErrorCode::InternalError));
        let error = result.as_ref().err().copied();
        let record = super::AuditRecord::from_request(
            &request,
            super::AuditSource::Cli,
            started,
            chrono::Utc::now(),
            if error.is_some() {
                super::AuditResult::Failed
            } else {
                super::AuditResult::Succeeded
            },
            error,
        );
        // The last old-root write uses the exclusive transaction, not admission.
        // Audit failure cannot erase a persisted reference or rewrite its result.
        let _ = audit.record_during_binding(&record, &binding);
        if saved_pending {
            binding.commit();
        } else {
            drop(binding);
        }
        Ok(result)
    })
    .await
    .map_err(map_backup_worker_error)?
}

/// Backup creation has no independent sync admission. The W2 cooperative lock
/// serializes filesystem transactions; storage admission survives observer drop.
async fn run_ipc_backup_owned(
    writers: Arc<crate::infrastructure::storage_writers::WriterGate>,
    root: std::path::PathBuf,
    request: IpcRequest,
    started: chrono::DateTime<chrono::Utc>,
    task: impl FnOnce() -> Result<serde_json::Value, IpcErrorCode> + Send + 'static,
) -> Result<serde_json::Value, IpcErrorCode> {
    run_ipc_storage_owned(writers, root, request, started, "CLI backup create", task).await
}

/// The shared GUI runtime lease and storage admission belong to the worker
/// through terminal projection and audit, not to the connected CLI observer.
async fn run_ipc_lifecycle_owned(
    writers: Arc<crate::infrastructure::storage_writers::WriterGate>,
    mutations: crate::state::SharedRuntimeInstall,
    root: std::path::PathBuf,
    request: IpcRequest,
    started: chrono::DateTime<chrono::Utc>,
    task: impl FnOnce() -> Result<serde_json::Value, IpcErrorCode> + Send + 'static,
) -> Result<serde_json::Value, IpcErrorCode> {
    let lease = mutations
        .acquire()
        .ok_or(IpcErrorCode::TargetStateConflict)?;
    let audit = super::audit::AuditStore::with_writers(&root, writers.clone());
    crate::commands::run_blocking_with_gate(writers, "CLI lifecycle", move || {
        let _lease = lease;
        Ok(execute_and_audit(&audit, &root, &request, started, task))
    })
    .await
    .map_err(map_backup_worker_error)?
}

/// Reuse the GUI lifecycle verifier and registered notifications. CLI success
/// must match the requested action; a stopped result cannot satisfy Start.
fn execute_ipc_lifecycle<R: ProcessRunner + ?Sized>(
    runner: &Mutex<R>,
    dependencies: &IpcDependencies,
    request: &IpcRequest,
    publisher: Option<crate::commands::notifications::NotificationPublisher<'_>>,
) -> Result<serde_json::Value, IpcErrorCode> {
    if request.args.contains_key("force") || request.args.contains_key("skip_backup") {
        return Err(IpcErrorCode::ValidationFailed);
    }
    let action = match request.command {
        IpcCommand::Start => LifecycleAction::Start,
        IpcCommand::Stop => LifecycleAction::Stop,
        IpcCommand::Restart => LifecycleAction::Restart,
        _ => return Err(IpcErrorCode::ValidationFailed),
    };
    let result = crate::commands::process_action_with_runner(
        crate::types::process_action::ProcessActionRequest {
            action,
            confirm: request.confirm,
        },
        runner,
        &dependencies.process_context,
        &dependencies.runtime_log,
        publisher,
    )
    .map_err(|_| IpcErrorCode::InternalError)?;
    if let Some(failure) = &result.failure {
        return Err(match failure.code.as_str() {
            "port_conflict" => IpcErrorCode::TargetStateConflict,
            "port_permission" => IpcErrorCode::AccessDenied,
            "lifecycle_timeout" => IpcErrorCode::Timeout,
            _ => IpcErrorCode::ExecutionFailed,
        });
    }
    if !matches!(
        (action, result.result),
        (
            LifecycleAction::Start | LifecycleAction::Restart,
            Some(LifecycleResult::Started)
        ) | (LifecycleAction::Stop, Some(LifecycleResult::Stopped))
    ) {
        return Err(IpcErrorCode::ExecutionFailed);
    }
    serde_json::to_value(IpcMessageData {
        status: action_to_status(action).to_string(),
        message: "Lifecycle command delegated to the running manager.".to_string(),
    })
    .map_err(|_| IpcErrorCode::InternalError)
}

/// Filesystem work and its final audit share worker-owned admission. Dropping
/// the IPC observer cannot allow a data-root switch during an admitted write.
async fn run_ipc_storage_owned(
    writers: Arc<crate::infrastructure::storage_writers::WriterGate>,
    root: std::path::PathBuf,
    request: IpcRequest,
    started: chrono::DateTime<chrono::Utc>,
    operation: &'static str,
    task: impl FnOnce() -> Result<serde_json::Value, IpcErrorCode> + Send + 'static,
) -> Result<serde_json::Value, IpcErrorCode> {
    let audit = super::audit::AuditStore::with_writers(&root, writers.clone());
    crate::commands::run_blocking_with_gate(writers, operation, move || {
        Ok(execute_and_audit(&audit, &root, &request, started, task))
    })
    .await
    .map_err(map_backup_worker_error)?
}

/// Called inside an admitted worker after its domain executor has projected the
/// terminal fact. Audit errors never invalidate the committed domain result.
fn execute_and_audit(
    audit: &super::audit::AuditStore,
    root: &std::path::Path,
    request: &IpcRequest,
    started: chrono::DateTime<chrono::Utc>,
    task: impl FnOnce() -> Result<serde_json::Value, IpcErrorCode>,
) -> Result<serde_json::Value, IpcErrorCode> {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(task))
        .unwrap_or(Err(IpcErrorCode::InternalError));
    let error = result.as_ref().err().copied();
    let record = super::AuditRecord::from_request(
        request,
        super::AuditSource::Cli,
        started,
        chrono::Utc::now(),
        if error.is_some() {
            super::AuditResult::Failed
        } else {
            super::AuditResult::Succeeded
        },
        error,
    );
    if audit.record(&record).is_err() {
        let _ = crate::infrastructure::runtime_log::RuntimeLog::new(root)
            .append_event("CLI mutation audit persistence failed");
    }
    result
}

#[cfg(test)]
fn project_ipc_sync(
    result: Result<
        crate::commands::sync::VerifiedSyncOutcome,
        crate::commands::sync::SyncExecutionFailure,
    >,
) -> Result<serde_json::Value, IpcErrorCode> {
    let verified = result.map_err(|failure| match failure {
        crate::commands::sync::SyncExecutionFailure::Setup(error) => map_app_error(error),
        crate::commands::sync::SyncExecutionFailure::Run(failure) => map_sync_run_failure(failure),
    })?;
    let outcome = verified.outcome;
    if outcome.conflicted {
        return Err(IpcErrorCode::TargetStateConflict);
    }
    if !verified.verified {
        return Err(IpcErrorCode::ExecutionFailed);
    }
    serde_json::to_value(IpcSyncData {
        snapshot_id: outcome.snapshot_id,
        artifacts: outcome.uploaded.len(),
        direction: if outcome.applied.is_empty() {
            "upload"
        } else {
            "sync"
        },
    })
    .map_err(|_| IpcErrorCode::InternalError)
}

fn owned_response(
    request_id: String,
    result: Result<serde_json::Value, IpcErrorCode>,
) -> IpcResponse<serde_json::Value> {
    let request_id = if request_id.starts_with("req_") {
        request_id
    } else {
        format!("req_{request_id}")
    };
    match result {
        Ok(data) => IpcResponse {
            request_id,
            ok: true,
            data: Some(data),
            error: None,
        },
        Err(code) => IpcResponse {
            request_id,
            ok: false,
            data: None,
            error: Some(IpcError {
                code,
                message: crate::modules::ipc::error_message(code).to_string(),
            }),
        },
    }
}

fn action_to_status(action: LifecycleAction) -> &'static str {
    match action {
        LifecycleAction::Start => "started",
        LifecycleAction::Stop => "stopped",
        LifecycleAction::Restart => "restarted",
    }
}

/// 端点只依赖执行抽象，避免在传输层引入具体 service 类型。
#[async_trait::async_trait]
pub trait IpcExecutor<E>: Send {
    async fn execute_ipc(&mut self, request: IpcRequest) -> IpcResponse<E>;
}

#[async_trait::async_trait]
impl IpcExecutor<serde_json::Value> for IpcService {
    async fn execute_ipc(&mut self, request: IpcRequest) -> IpcResponse<serde_json::Value> {
        self.execute(request).await
    }
}

#[async_trait::async_trait]
impl IpcExecutor<serde_json::Value> for &mut IpcService {
    async fn execute_ipc(&mut self, request: IpcRequest) -> IpcResponse<serde_json::Value> {
        self.execute(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 回归：CLI `sync run` 必须真的执行同步。
    ///
    /// 此前 `IpcService::sync_run` 只读同步状态文件就返回 `ok: true`，伪造一个 snapshot_id
    /// 与 `artifacts: 0`，从不接触网络——这是「假成功」。未配置端点时必须如实失败。
    #[tokio::test]
    async fn sync_run_without_endpoint_reports_failure_instead_of_fake_success() {
        let root = tempfile::tempdir().unwrap();
        let mut service = IpcService::for_tests_with(root.path().into(), root.path().into());
        // The v2 production boundary requires the actual fixed HOME to exist.
        service.dependencies.process_context.opencodex_home = root.path().into();
        let request = IpcRequest {
            request_id: "req_00000000-0000-0000-0000-0000000000ff".to_string(),
            command: IpcCommand::SyncRun,
            args: std::collections::BTreeMap::new(),
            confirm: true,
            contract_version: 1,
            secret: None,
        };
        let response = service.execute(request).await;
        assert!(!response.ok, "没有端点配置时 CLI 同步不得返回成功");
        assert_eq!(
            response.error.map(|error| error.code),
            Some(IpcErrorCode::TargetStateConflict)
        );
    }

    fn sync_request() -> IpcRequest {
        IpcRequest {
            request_id: "req_00000000-0000-0000-0000-0000000000aa".to_string(),
            command: IpcCommand::SyncRun,
            args: BTreeMap::new(),
            confirm: true,
            contract_version: 1,
            secret: None,
        }
    }

    fn verified_outcome(
        conflicted: bool,
        verified: bool,
    ) -> crate::commands::sync::VerifiedSyncOutcome {
        crate::commands::sync::VerifiedSyncOutcome {
            outcome: crate::modules::sync::runner::SyncOutcome {
                snapshot_id: "snap_test".to_string(),
                uploaded: vec!["state/preferences.json".to_string()],
                applied: vec![],
                conflicted,
                etag: None,
            },
            verified,
        }
    }

    #[test]
    fn sync_projection_preserves_conflicts_readback_failure_and_typed_network_errors() {
        use crate::infrastructure::webdav_client::{WebDavError, WebDavOperation};
        use crate::modules::sync::runner::SyncRunFailure as F;
        let data = project_ipc_sync(Ok(verified_outcome(false, true))).unwrap();
        assert_eq!(data["artifacts"], 1);
        assert_eq!(data["direction"], "upload");
        assert_eq!(
            project_ipc_sync(Ok(verified_outcome(true, true))),
            Err(IpcErrorCode::TargetStateConflict)
        );
        assert_eq!(
            project_ipc_sync(Ok(verified_outcome(true, false))),
            Err(IpcErrorCode::TargetStateConflict)
        );
        assert_eq!(
            project_ipc_sync(Ok(verified_outcome(false, false))),
            Err(IpcErrorCode::ExecutionFailed)
        );
        for (failure, code) in [
            (F::ColdSync, IpcErrorCode::TargetStateConflict),
            (F::TargetLocked, IpcErrorCode::Timeout),
            (
                F::WebDav(WebDavOperation::Upload, WebDavError::Unauthorized),
                IpcErrorCode::AccessDenied,
            ),
            (
                F::WebDav(WebDavOperation::Download, WebDavError::NotFound),
                IpcErrorCode::TargetNotFound,
            ),
            (
                F::WebDav(WebDavOperation::Upload, WebDavError::Network),
                IpcErrorCode::ExecutionFailed,
            ),
        ] {
            let response =
                owned_response("caller-id".into(), project_ipc_sync(Err(failure.into())));
            assert!(!response.ok);
            assert!(response.data.is_none());
            assert_eq!(response.request_id, "req_caller-id");
            assert_eq!(response.error.unwrap().code, code);
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cancelled_sync_observer_keeps_admissions_until_terminal_and_audit_for_each_result() {
        use crate::infrastructure::storage_writers::WriterGate;
        use std::time::Duration;
        // Real filesystem/admission/audit ownership, injected terminal executor.
        // This is not a live WebDAV or native notification transport test.
        for case in 0..5 {
            let root = tempfile::tempdir().unwrap();
            let committed = root.path().join("committed");
            let terminal = root.path().join("terminal");
            let writers = Arc::new(WriterGate::default());
            let operations = Arc::new(tokio::sync::Mutex::new(()));
            let entered = Arc::new(tokio::sync::Notify::new());
            let release = Arc::new(tokio::sync::Notify::new());
            let worker_writers = writers.clone();
            let worker_operations = operations.clone();
            let worker_root = root.path().to_path_buf();
            let worker_committed = committed.clone();
            let worker_terminal = terminal.clone();
            let worker_entered = entered.clone();
            let worker_release = release.clone();
            let observer = tokio::spawn(async move {
                run_ipc_sync_owned(
                    worker_writers,
                    worker_operations,
                    worker_root,
                    sync_request(),
                    chrono::Utc::now(),
                    move || {
                        std::fs::write(worker_committed, b"committed").unwrap();
                        worker_entered.notify_one();
                        tauri::async_runtime::block_on(worker_release.notified());
                        std::fs::write(worker_terminal, b"projected").unwrap();
                        match case {
                            0 => project_ipc_sync(Ok(verified_outcome(false, true))),
                            1 => project_ipc_sync(Ok(verified_outcome(true, true))),
                            2 => project_ipc_sync(Ok(verified_outcome(false, false))),
                            3 => Err(IpcErrorCode::AccessDenied),
                            _ => panic!("injected terminal panic"),
                        }
                    },
                )
                .await
            });
            tokio::time::timeout(Duration::from_secs(2), entered.notified())
                .await
                .unwrap();
            observer.abort();
            assert!(observer.await.unwrap_err().is_cancelled());
            assert!(committed.exists());
            assert!(!terminal.exists());
            assert!(!root.path().join("audit.log").exists());
            assert!(writers.freeze().unwrap().is_none());
            assert!(operations.clone().try_lock_owned().is_err());
            release.notify_one();
            tokio::time::timeout(Duration::from_secs(2), async {
                writers.wait_until_idle_for_test().await.unwrap();
                assert!(writers.freeze().unwrap().is_some());
            })
            .await
            .unwrap();
            assert!(operations.clone().try_lock_owned().is_ok());
            assert!(terminal.exists());
            let log = std::fs::read_to_string(root.path().join("audit.log")).unwrap();
            let records: Vec<super::super::AuditRecord> = log
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect();
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].command, IpcCommand::SyncRun);
            assert_eq!(
                records[0].error_code,
                match case {
                    0 => None,
                    1 => Some(IpcErrorCode::TargetStateConflict),
                    2 => Some(IpcErrorCode::ExecutionFailed),
                    3 => Some(IpcErrorCode::AccessDenied),
                    _ => Some(IpcErrorCode::InternalError),
                }
            );
            assert_eq!(
                records[0].result,
                if case == 0 {
                    super::super::AuditResult::Succeeded
                } else {
                    super::super::AuditResult::Failed
                }
            );
            assert!(!log.contains("committed"));
            assert!(!log.contains("projected"));
        }
    }

    #[tokio::test]
    async fn busy_frozen_and_unconfirmed_cli_sync_do_not_execute_or_write_audit() {
        use crate::infrastructure::storage_writers::WriterGate;
        use std::sync::atomic::{AtomicUsize, Ordering};
        let root = tempfile::tempdir().unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        let writers = Arc::new(WriterGate::default());
        let operations = Arc::new(tokio::sync::Mutex::new(()));
        let held = operations.clone().try_lock_owned().unwrap();
        let counter = calls.clone();
        assert_eq!(
            run_ipc_sync_owned(
                writers.clone(),
                operations.clone(),
                root.path().to_path_buf(),
                sync_request(),
                chrono::Utc::now(),
                move || {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok(serde_json::Value::Null)
                }
            )
            .await,
            Err(IpcErrorCode::Timeout)
        );
        drop(held);
        let binding = writers.freeze().unwrap().unwrap();
        let counter = calls.clone();
        assert_eq!(
            run_ipc_sync_owned(
                writers.clone(),
                operations,
                root.path().to_path_buf(),
                sync_request(),
                chrono::Utc::now(),
                move || {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok(serde_json::Value::Null)
                }
            )
            .await,
            Err(IpcErrorCode::TargetStateConflict)
        );
        drop(binding);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let mut service = IpcService::for_tests_with(root.path().into(), root.path().into());
        let mut request = sync_request();
        request.confirm = false;
        let response = service.execute(request).await;
        assert!(!response.ok);
        assert_eq!(response.error.unwrap().code, IpcErrorCode::RequireConfirm);
        assert!(!root.path().join("audit.log").exists());
    }

    fn backup_request(command: IpcCommand) -> IpcRequest {
        let mut request = sync_request();
        request.command = command;
        request
    }

    fn preferences_fixture(root: &std::path::Path) {
        crate::modules::preferences::PreferencesStore::new(root)
            .save(&crate::modules::preferences::Preferences::default())
            .unwrap();
    }

    fn audit_records(root: &std::path::Path) -> Vec<super::super::AuditRecord> {
        std::fs::read_to_string(root.join("audit.log"))
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    struct LifecycleFixtureRunner<F>(F);

    impl<F> ProcessRunner for LifecycleFixtureRunner<F>
    where
        F: Fn(
            &crate::modules::process::ProcessCommand,
        ) -> Result<LifecycleResult, crate::errors::AppError>,
    {
        fn execute(
            &self,
            command: &crate::modules::process::ProcessCommand,
        ) -> Result<LifecycleResult, crate::errors::AppError> {
            (self.0)(command)
        }
        fn cancel_pending(&self) -> Result<(), crate::errors::AppError> {
            Ok(())
        }
    }

    fn lifecycle_dependencies(root: &std::path::Path) -> IpcDependencies {
        let mut dependencies = IpcService::for_tests_with(root.into(), root.into()).dependencies;
        let executable = root.join("fixture-runtime");
        std::fs::write(&executable, b"isolated runtime candidate").unwrap();
        dependencies.process_context.runtime =
            crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(executable);
        dependencies.process_context.working_directory = root.into();
        dependencies.process_context.opencodex_home = root.join("runtime-home");
        dependencies
    }

    #[test]
    fn cli_lifecycle_projects_only_matching_terminal_and_preserves_context_and_wire_contract() {
        for command in [IpcCommand::Start, IpcCommand::Stop, IpcCommand::Restart] {
            let root = tempfile::tempdir().unwrap();
            let dependencies = lifecycle_dependencies(root.path());
            let request = backup_request(command);
            let expected_action = match command {
                IpcCommand::Start => LifecycleAction::Start,
                IpcCommand::Stop => LifecycleAction::Stop,
                _ => LifecycleAction::Restart,
            };
            for terminal in [
                Some(LifecycleResult::Started),
                Some(LifecycleResult::Stopped),
                Some(LifecycleResult::Cancelled),
                Some(LifecycleResult::Failed),
                None,
            ] {
                let runner = Mutex::new(LifecycleFixtureRunner(
                    |process: &crate::modules::process::ProcessCommand| {
                        assert_eq!(process.action, expected_action);
                        assert_eq!(
                            process.executable,
                            dependencies.process_context.executable().unwrap()
                        );
                        assert_eq!(
                            process.working_directory,
                            dependencies.process_context.working_directory
                        );
                        assert_eq!(
                            process.environment.opencodex_home,
                            dependencies.process_context.opencodex_home
                        );
                        terminal
                            .clone()
                            .ok_or(crate::errors::AppError::NotConfigured)
                    },
                ));
                let result = execute_ipc_lifecycle(&runner, &dependencies, &request, None);
                let expected = matches!(
                    (expected_action, terminal),
                    (
                        LifecycleAction::Start | LifecycleAction::Restart,
                        Some(LifecycleResult::Started)
                    ) | (LifecycleAction::Stop, Some(LifecycleResult::Stopped))
                );
                let response = owned_response("lifecycle-id".into(), result);
                assert_eq!(response.ok, expected);
                assert_eq!(response.request_id, "req_lifecycle-id");
                if expected {
                    let data = response.data.unwrap();
                    assert_eq!(data["status"], action_to_status(expected_action));
                    assert_eq!(
                        data["message"],
                        "Lifecycle command delegated to the running manager."
                    );
                    assert_eq!(data.as_object().unwrap().len(), 2);
                } else {
                    assert!(response.data.is_none());
                    assert_eq!(response.error.unwrap().code, IpcErrorCode::ExecutionFailed);
                }
            }
        }
    }

    #[test]
    fn cli_lifecycle_reuses_registered_events_and_recovers_only_exact_action_and_candidate() {
        use crate::commands::notifications::NotificationPublisher;
        use crate::modules::notifications::NotificationStore;
        use crate::modules::preferences::{Preferences, PreferencesStore};
        let root = tempfile::tempdir().unwrap();
        let dependencies = lifecycle_dependencies(root.path());
        let notifications = Arc::new(Mutex::new(NotificationStore::new()));
        let publisher = Some(NotificationPublisher {
            store: &notifications,
            data_root: root.path(),
        });
        let failed = Mutex::new(LifecycleFixtureRunner(
            |_: &crate::modules::process::ProcessCommand| Ok(LifecycleResult::Failed),
        ));
        let started = Mutex::new(LifecycleFixtureRunner(
            |_: &crate::modules::process::ProcessCommand| Ok(LifecycleResult::Started),
        ));
        let stopped = Mutex::new(LifecycleFixtureRunner(
            |_: &crate::modules::process::ProcessCommand| Ok(LifecycleResult::Stopped),
        ));
        let start = backup_request(IpcCommand::Start);
        let stop = backup_request(IpcCommand::Stop);
        assert_eq!(
            execute_ipc_lifecycle(&failed, &dependencies, &start, publisher),
            Err(IpcErrorCode::ExecutionFailed)
        );
        assert_eq!(
            execute_ipc_lifecycle(&failed, &dependencies, &start, publisher),
            Err(IpcErrorCode::ExecutionFailed)
        );
        assert_eq!(notifications.lock().unwrap().live().len(), 1);
        assert_eq!(notifications.lock().unwrap().aggregate().unresolved, 1);
        // A different action and a mismatched terminal must not recover Start.
        execute_ipc_lifecycle(&stopped, &dependencies, &stop, publisher).unwrap();
        assert_eq!(
            execute_ipc_lifecycle(&stopped, &dependencies, &start, publisher),
            Err(IpcErrorCode::ExecutionFailed)
        );
        assert_eq!(notifications.lock().unwrap().aggregate().unresolved, 1);
        PreferencesStore::new(root.path())
            .save(&Preferences {
                lifecycle_notifications: false,
                ..Default::default()
            })
            .unwrap();
        // Preference suppresses new failures, not recovery of an existing one.
        execute_ipc_lifecycle(&started, &dependencies, &start, publisher).unwrap();
        assert_eq!(notifications.lock().unwrap().aggregate().unresolved, 0);
        assert_eq!(notifications.lock().unwrap().live().len(), 1);
        assert!(notifications.lock().unwrap().live()[0].resolved);
        let persisted = crate::modules::notifications::persistence::load_notifications(
            &crate::modules::notifications::persistence::notifications_path(root.path()),
        )
        .unwrap();
        assert_eq!(persisted, *notifications.lock().unwrap());
        assert_eq!(
            execute_ipc_lifecycle(&failed, &dependencies, &stop, publisher),
            Err(IpcErrorCode::ExecutionFailed)
        );
        assert_eq!(notifications.lock().unwrap().live().len(), 1);
        PreferencesStore::new(root.path())
            .save(&Preferences::default())
            .unwrap();
        assert_eq!(
            execute_ipc_lifecycle(&failed, &dependencies, &start, publisher),
            Err(IpcErrorCode::ExecutionFailed)
        );
        let executable = dependencies.process_context.executable().unwrap();
        std::fs::write(&executable, b"different candidate").unwrap();
        execute_ipc_lifecycle(&started, &dependencies, &start, publisher).unwrap();
        assert_eq!(notifications.lock().unwrap().aggregate().unresolved, 1);
        // Returning to old bytes is an ABA candidate generation, not a retry
        // of the previous execution. Neither success may clear that old failure.
        std::fs::write(&executable, b"isolated runtime candidate").unwrap();
        execute_ipc_lifecycle(&started, &dependencies, &start, publisher).unwrap();
        assert_eq!(notifications.lock().unwrap().aggregate().unresolved, 1);
        assert_eq!(
            execute_ipc_lifecycle(&failed, &dependencies, &start, publisher),
            Err(IpcErrorCode::ExecutionFailed)
        );
        assert_eq!(notifications.lock().unwrap().aggregate().unresolved, 2);
        execute_ipc_lifecycle(&started, &dependencies, &start, publisher).unwrap();
        assert_eq!(notifications.lock().unwrap().aggregate().unresolved, 1);
        let persisted = crate::modules::notifications::persistence::load_notifications(
            &crate::modules::notifications::persistence::notifications_path(root.path()),
        )
        .unwrap();
        assert_eq!(persisted, *notifications.lock().unwrap());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cli_lifecycle_worker_survives_disconnect_and_holds_both_gates_through_audit() {
        use crate::infrastructure::storage_writers::WriterGate;
        use std::time::Duration;
        // Injected runner; real gates, terminal notification files and audit.
        // This is not native runtime execution or a performance budget result.
        for command in [IpcCommand::Start, IpcCommand::Stop, IpcCommand::Restart] {
            for case in 0..3 {
                let root = tempfile::tempdir().unwrap();
                let dependencies = lifecycle_dependencies(root.path());
                let writers = Arc::new(WriterGate::default());
                let mutations = crate::state::SharedRuntimeInstall::default();
                let entered = Arc::new(tokio::sync::Notify::new());
                let release = Arc::new(tokio::sync::Notify::new());
                let notifications = Arc::new(Mutex::new(
                    crate::modules::notifications::NotificationStore::new(),
                ));
                let task_entered = entered.clone();
                let task_release = release.clone();
                let task_notifications = notifications.clone();
                let task_writers = writers.clone();
                let task_mutations = mutations.clone();
                let worker_root = root.path().to_path_buf();
                let request = backup_request(command);
                let task_request = request.clone();
                let observer = tokio::spawn(async move {
                    run_ipc_lifecycle_owned(
                        task_writers,
                        task_mutations,
                        worker_root,
                        request,
                        chrono::Utc::now(),
                        move || {
                            let runner = Mutex::new(LifecycleFixtureRunner(
                                |_: &crate::modules::process::ProcessCommand| {
                                    task_entered.notify_one();
                                    tauri::async_runtime::block_on(task_release.notified());
                                    match case {
                                        0 => Ok(if command == IpcCommand::Stop {
                                            LifecycleResult::Stopped
                                        } else {
                                            LifecycleResult::Started
                                        }),
                                        1 => Ok(LifecycleResult::Failed),
                                        _ => panic!("isolated lifecycle worker panic"),
                                    }
                                },
                            ));
                            execute_ipc_lifecycle(
                                &runner,
                                &dependencies,
                                &task_request,
                                Some(crate::commands::notifications::NotificationPublisher {
                                    store: &task_notifications,
                                    data_root: &dependencies.active_data_root,
                                }),
                            )
                        },
                    )
                    .await
                });
                tokio::time::timeout(Duration::from_secs(2), entered.notified())
                    .await
                    .unwrap();
                // This timer must run while the synchronous runner is still blocked.
                tokio::time::timeout(
                    Duration::from_secs(2),
                    tokio::time::sleep(Duration::from_millis(20)),
                )
                .await
                .unwrap();
                assert!(!observer.is_finished());
                observer.abort();
                assert!(observer.await.unwrap_err().is_cancelled());
                assert!(writers.freeze().unwrap().is_none());
                assert!(mutations.acquire().is_none());
                assert!(!root.path().join("audit.log").exists());
                release.notify_one();
                tokio::time::timeout(Duration::from_secs(2), async {
                    loop {
                        if writers.freeze().unwrap().is_some() {
                            break;
                        }
                        tokio::task::yield_now().await;
                    }
                })
                .await
                .unwrap();
                assert!(mutations.acquire().is_some());
                let audit = audit_records(root.path());
                assert_eq!(audit.len(), 1);
                assert_eq!(audit[0].command, command);
                assert_eq!(
                    audit[0].error_code,
                    match case {
                        0 => None,
                        1 => Some(IpcErrorCode::ExecutionFailed),
                        _ => Some(IpcErrorCode::InternalError),
                    }
                );
                assert_eq!(
                    notifications.lock().unwrap().aggregate().unresolved,
                    usize::from(case == 1)
                );
            }
        }
    }

    #[tokio::test]
    async fn cli_lifecycle_rejects_busy_frozen_unconfirmed_and_unsafe_arguments() {
        for command in [IpcCommand::Start, IpcCommand::Stop, IpcCommand::Restart] {
            let root = tempfile::tempdir().unwrap();
            let mut service = IpcService::for_tests_with(root.path().into(), root.path().into());
            let lease = service.runtime_mutations.acquire().unwrap();
            let response = service.execute(backup_request(command)).await;
            assert_eq!(
                response.error.unwrap().code,
                IpcErrorCode::TargetStateConflict
            );
            assert!(!root.path().join("audit.log").exists());
            drop(lease);
            let binding = service.writers.freeze().unwrap().unwrap();
            let response = service.execute(backup_request(command)).await;
            assert_eq!(
                response.error.unwrap().code,
                IpcErrorCode::TargetStateConflict
            );
            assert!(!service.runtime_mutations.is_running());
            assert!(!root.path().join("audit.log").exists());
            drop(binding);
            let mut request = backup_request(command);
            request.confirm = false;
            assert_eq!(
                service.execute(request).await.error.unwrap().code,
                IpcErrorCode::RequireConfirm
            );
            assert!(!root.path().join("audit.log").exists());
            for key in ["force", "skip_backup"] {
                let mut request = backup_request(command);
                request.args.insert(key.into(), "true".into());
                assert_eq!(
                    service.execute(request).await.error.unwrap().code,
                    IpcErrorCode::ValidationFailed
                );
            }
            assert!(!service.runtime_mutations.is_running());
            let audit = audit_records(root.path());
            assert_eq!(audit.len(), 2);
            assert!(audit
                .iter()
                .all(|record| record.result == super::super::AuditResult::Failed));
            service.dependencies.process_context.runtime =
                crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved();
            assert_eq!(
                service
                    .execute(backup_request(command))
                    .await
                    .error
                    .unwrap()
                    .code,
                IpcErrorCode::InternalError
            );
            assert_eq!(audit_records(root.path()).len(), 3);
        }
    }

    #[cfg(unix)]
    fn v2_binding_service(user: &std::path::Path, sandbox: Option<&std::path::Path>) -> IpcService {
        use crate::modules::data_root;
        use std::os::unix::fs::PermissionsExt;
        let startup = data_root::startup_v2::resolve(user, sandbox).unwrap();
        let root = startup.runtime_config.active_data_root.clone();
        let home = data_root::resolve_opencodex_home(&startup.runtime_config);
        let installation = sandbox
            .unwrap_or_else(|| user.parent().unwrap())
            .join("program");
        std::fs::create_dir_all(&installation).unwrap();
        let executable = installation.join("stopped-ocx");
        std::fs::write(
            &executable,
            b"#!/bin/sh\necho '{\"status\":\"stopped\",\"dataRoot\":\"/tmp/fixture\"}'\n",
        )
        .unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
        let mut service = IpcService::for_tests_with(root, startup.anchor.clone());
        service.dependencies.opencodex_home = home.clone();
        service.dependencies.process_context.opencodex_home = home.clone();
        service.dependencies.home = user.into();
        service.collector = Arc::new(Mutex::new(crate::modules::status::StatusCollector::new(
            crate::infrastructure::status_source::OfficialStatusSource::new(
                crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(
                    executable,
                ),
                &home,
                crate::modules::process::EnvironmentPolicy {
                    opencodex_home: home.clone(),
                    ..Default::default()
                },
            ),
        )));
        service.binding_context = Some(Arc::new(Mutex::new(TestBindingContext {
            startup,
            installation,
            update_pending: false,
        })));
        service
    }

    fn reference_request(target: &std::path::Path) -> IpcRequest {
        let mut request = backup_request(IpcCommand::DataRootSwitch);
        request
            .args
            .insert("target".into(), target.display().to_string());
        request
    }

    fn migration_request(command: IpcCommand, path: &std::path::Path) -> IpcRequest {
        let mut request = backup_request(command);
        let key = match command {
            IpcCommand::Export => "output",
            IpcCommand::Import => "input",
            _ => unreachable!(),
        };
        request.args.insert(key.into(), path.display().to_string());
        request
    }

    #[tokio::test]
    async fn cli_migration_is_explicitly_unsupported_without_reading_or_changing_files() {
        for command in [IpcCommand::Export, IpcCommand::Import] {
            let t = tempfile::tempdir().unwrap();
            let root = t.path().join("desktop");
            crate::modules::data_root::initialize(&root).unwrap();
            let metadata =
                std::fs::read(root.join(crate::modules::data_root::METADATA_RELATIVE_PATH))
                    .unwrap();
            let container = t.path().join("old-container");
            let home = t.path().join("native-home");
            std::fs::create_dir(&home).unwrap();
            std::fs::write(home.join("config.json"), b"native config").unwrap();
            std::fs::write(&container, b"legacy secret payload").unwrap();
            let mut service = IpcService::for_tests_with(root.clone(), root.clone());
            service.dependencies.home = home.clone();
            let mut request = migration_request(command, &container);
            request.secret = Some("must never echo".into());
            let response = service.execute(request).await;
            assert!(!response.ok && response.data.is_none());
            let wire = serde_json::to_value(&response).unwrap();
            assert_eq!(wire["error"]["code"], "validation_failed");
            assert!(wire["error"]["message"]
                .as_str()
                .unwrap()
                .contains("unsupported"));
            assert!(!wire.to_string().contains("must never echo"));
            assert_eq!(std::fs::read(&container).unwrap(), b"legacy secret payload");
            assert_eq!(
                std::fs::read(home.join("config.json")).unwrap(),
                b"native config"
            );
            assert_eq!(
                std::fs::read(root.join(crate::modules::data_root::METADATA_RELATIVE_PATH))
                    .unwrap(),
                metadata
            );
            assert!(!root.join("audit.log").exists());
            assert!(!root.join(".backup-w2.lock").exists());
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cli_migration_refusal_does_not_wait_for_legacy_file_locks_or_create_targets() {
        use crate::infrastructure::locking::TargetFileLock;
        use std::time::Duration;
        for command in [IpcCommand::Export, IpcCommand::Import] {
            let t = tempfile::tempdir().unwrap();
            let root = t.path().join("missing-desktop");
            let container = t.path().join("locked-legacy-container");
            std::fs::write(&container, b"retained").unwrap();
            let _lock = TargetFileLock::lock(&container).unwrap();
            let mut service = IpcService::for_tests_with(root.clone(), root.clone());
            let response = tokio::time::timeout(
                Duration::from_secs(1),
                service.execute(migration_request(command, &container)),
            )
            .await
            .unwrap();
            assert_eq!(
                response.error.as_ref().unwrap().code,
                IpcErrorCode::ValidationFailed
            );
            assert!(response.error.unwrap().message.contains("unsupported"));
            assert!(!root.exists());
            assert_eq!(std::fs::read(container).unwrap(), b"retained");
            assert!(service.writers.admit().is_ok());
            assert!(service.runtime_mutations.acquire().is_some());
        }
    }

    #[tokio::test]
    async fn cli_migration_rejects_unconfirmed_frozen_and_invalid_requests_without_file_mutation() {
        for command in [IpcCommand::Export, IpcCommand::Import] {
            let root = tempfile::tempdir().unwrap();
            let container = root.path().join("config.ocx");
            let mut service = IpcService::for_tests_with(root.path().into(), root.path().into());
            let mut unconfirmed = migration_request(command, &container);
            unconfirmed.confirm = false;
            assert_eq!(
                service.execute(unconfirmed).await.error.unwrap().code,
                IpcErrorCode::RequireConfirm
            );
            let binding = service.writers.freeze().unwrap().unwrap();
            assert_eq!(
                service
                    .execute(migration_request(command, &container))
                    .await
                    .error
                    .unwrap()
                    .code,
                IpcErrorCode::ValidationFailed
            );
            assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
            drop(binding);
            let mut invalid = migration_request(command, &container);
            invalid.args.clear();
            invalid.secret = Some("test-only-secret-not-for-audit".into());
            assert_eq!(
                service.execute(invalid).await.error.unwrap().code,
                IpcErrorCode::ValidationFailed
            );
            assert!(!container.exists());
            assert!(!root.path().join("manager-state").exists());
            assert!(!root.path().join("audit.log").exists());
            assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
        }
    }

    #[tokio::test]
    async fn cli_backup_uses_manual_scope_verified_source_and_complete_metadata_listing() {
        use crate::modules::backup::{self, manager, BackupAction};
        let root = tempfile::tempdir().unwrap();
        preferences_fixture(root.path());
        let payload = std::fs::read(
            root.path()
                .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH),
        )
        .unwrap();
        let legacy = backup::backup_file(
            root.path(),
            BackupAction::Upgrade,
            &root
                .path()
                .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH),
            &payload,
            chrono::Utc::now(),
            None,
        )
        .unwrap();
        let mut service = IpcService::for_tests_with(root.path().into(), root.path().into());
        let response = service
            .execute(backup_request(IpcCommand::BackupCreate))
            .await;
        assert!(response.ok, "{:?}", response.error);
        let data = response.data.unwrap();
        assert_eq!(data["status"], "backed_up");
        assert_eq!(
            data.as_object().unwrap().len(),
            2,
            "preserve existing CLI DTO"
        );
        let id = data["message"].as_str().unwrap();
        let records = manager::list_records(root.path()).unwrap();
        let record = records.iter().find(|r| r.manifest.backup_id == id).unwrap();
        assert_eq!(record.manifest.action, BackupAction::ManualPreferences);
        assert_eq!(
            record.manifest.management.as_ref().unwrap().kind,
            "standalone"
        );
        assert_eq!(
            std::fs::read(record.directory.join("preferences.json")).unwrap(),
            payload
        );
        assert_eq!(
            record.manifest.sha256,
            crate::infrastructure::hash::sha256_hex(&payload)
        );
        let listed = service
            .execute(backup_request(IpcCommand::BackupList))
            .await;
        assert!(listed.ok);
        let items = listed.data.unwrap()["items"].as_array().unwrap().clone();
        assert_eq!(items.len(), 2);
        for id in [id, legacy.manifest.backup_id.as_str()] {
            assert!(items.iter().any(|r| r["backup_id"] == id));
        }
        let log = audit_records(root.path());
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].command, IpcCommand::BackupCreate);
        assert_eq!(log[0].result, super::super::AuditResult::Succeeded);
        assert!(!std::fs::read_to_string(root.path().join("audit.log"))
            .unwrap()
            .contains(&root.path().display().to_string()));
    }

    #[tokio::test]
    async fn cli_backup_honours_saved_cleanup_mode_and_retains_pins_and_legacy_transactions() {
        use crate::modules::backup::{
            self, manager,
            policy::{CleanupMode, CleanupPolicy},
            BackupAction,
        };
        for mode in [CleanupMode::Manual, CleanupMode::Automatic] {
            let root = tempfile::tempdir().unwrap();
            preferences_fixture(root.path());
            let now = chrono::Utc::now();
            for days in 60..72 {
                crate::commands::backup::execute_preferences_backup(
                    None,
                    root.path(),
                    now - chrono::Duration::days(days),
                )
                .unwrap();
            }
            let pinned = crate::commands::backup::execute_preferences_backup(
                None,
                root.path(),
                now - chrono::Duration::days(200),
            )
            .unwrap();
            manager::set_pinned(root.path(), &pinned.backup.backup_id, true).unwrap();
            let payload = std::fs::read(
                root.path()
                    .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH),
            )
            .unwrap();
            let legacy = backup::backup_file(
                root.path(),
                BackupAction::Upgrade,
                &root
                    .path()
                    .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH),
                &payload,
                now - chrono::Duration::days(365),
                None,
            )
            .unwrap();
            manager::save_policy(
                root.path(),
                &CleanupPolicy {
                    mode,
                    ..Default::default()
                },
            )
            .unwrap();
            let mut service = IpcService::for_tests_with(root.path().into(), root.path().into());
            let response = service
                .execute(backup_request(IpcCommand::BackupCreate))
                .await;
            assert!(response.ok, "{:?}", response.error);
            let records = manager::list(root.path()).unwrap();
            assert_eq!(
                records.len(),
                if mode == CleanupMode::Manual { 15 } else { 12 }
            );
            for id in [&pinned.backup.backup_id, &legacy.manifest.backup_id] {
                assert!(records.iter().any(|r| &r.id == id && r.protected));
            }
            assert_eq!(
                audit_records(root.path())[0].result,
                super::super::AuditResult::Succeeded
            );
        }
    }

    #[tokio::test]
    async fn cli_backup_rejects_missing_corrupt_source_or_policy_without_fake_success_or_cleanup() {
        use crate::modules::backup::{
            manager,
            policy::{CleanupMode, CleanupPolicy, POLICY_RELATIVE_PATH},
        };
        for case in 0..3 {
            let root = tempfile::tempdir().unwrap();
            preferences_fixture(root.path());
            let now = chrono::Utc::now();
            for days in 60..72 {
                crate::commands::backup::execute_preferences_backup(
                    None,
                    root.path(),
                    now - chrono::Duration::days(days),
                )
                .unwrap();
            }
            manager::save_policy(
                root.path(),
                &CleanupPolicy {
                    mode: CleanupMode::Automatic,
                    ..Default::default()
                },
            )
            .unwrap();
            let target = root
                .path()
                .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH);
            match case {
                0 => std::fs::remove_file(&target).unwrap(),
                1 => std::fs::write(&target, b"{invalid source}").unwrap(),
                _ => std::fs::write(root.path().join(POLICY_RELATIVE_PATH), b"{invalid policy}")
                    .unwrap(),
            }
            let before = manager::list(root.path()).unwrap();
            let mut service = IpcService::for_tests_with(root.path().into(), root.path().into());
            let response = service
                .execute(backup_request(IpcCommand::BackupCreate))
                .await;
            assert!(!response.ok);
            assert!(response.data.is_none());
            assert_eq!(response.error.unwrap().code, IpcErrorCode::ExecutionFailed);
            assert_eq!(manager::list(root.path()).unwrap(), before);
            assert_eq!(
                audit_records(root.path())[0].result,
                super::super::AuditResult::Failed
            );
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cancelled_backup_observer_keeps_admission_through_real_terminal_and_audit() {
        use crate::infrastructure::storage_writers::WriterGate;
        use std::time::Duration;
        // Real source/manager callbacks/gate/audit, no native GUI transport.
        for case in 0..3 {
            let root = tempfile::tempdir().unwrap();
            preferences_fixture(root.path());
            if case == 1 {
                std::fs::write(
                    root.path()
                        .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH),
                    b"invalid",
                )
                .unwrap();
            }
            let writers = Arc::new(WriterGate::default());
            let entered = Arc::new(tokio::sync::Notify::new());
            let release = Arc::new(tokio::sync::Notify::new());
            let worker_writers = writers.clone();
            let worker_root = root.path().to_path_buf();
            let worker_entered = entered.clone();
            let worker_release = release.clone();
            let observer = tokio::spawn(async move {
                run_ipc_backup_owned(
                    worker_writers,
                    worker_root.clone(),
                    backup_request(IpcCommand::BackupCreate),
                    chrono::Utc::now(),
                    move || {
                        crate::modules::backup::manager::create_with_saved_policy_observed(
                            &worker_root,
                            chrono::Utc::now(),
                            |_| {
                                worker_entered.notify_one();
                                tauri::async_runtime::block_on(worker_release.notified());
                                if case == 2 {
                                    panic!("injected backup panic");
                                }
                            },
                            |(), result| {
                                std::fs::write(
                                    worker_root.join("terminal"),
                                    if result.is_ok() {
                                        b"ok".as_slice()
                                    } else {
                                        b"failed".as_slice()
                                    },
                                )
                                .unwrap()
                            },
                        )
                        .map(|v| serde_json::json!({"backup_id": v.backup.backup_id}))
                        .map_err(map_app_error)
                    },
                )
                .await
            });
            tokio::time::timeout(Duration::from_secs(2), entered.notified())
                .await
                .unwrap();
            observer.abort();
            assert!(observer.await.unwrap_err().is_cancelled());
            assert!(writers.freeze().unwrap().is_none());
            assert!(!root.path().join("audit.log").exists());
            release.notify_one();
            tokio::time::timeout(Duration::from_secs(2), async {
                writers.wait_until_idle_for_test().await.unwrap();
                assert!(writers.freeze().unwrap().is_some());
            })
            .await
            .unwrap();
            let records = crate::modules::backup::manager::list(root.path()).unwrap();
            assert_eq!(records.len(), usize::from(case == 0));
            let audit = audit_records(root.path());
            assert_eq!(audit.len(), 1);
            assert_eq!(audit[0].command, IpcCommand::BackupCreate);
            assert_eq!(
                audit[0].error_code,
                match case {
                    0 => None,
                    1 => Some(IpcErrorCode::ExecutionFailed),
                    _ => Some(IpcErrorCode::InternalError),
                }
            );
            assert_eq!(root.path().join("terminal").exists(), case != 2);
        }
    }

    #[tokio::test]
    async fn frozen_and_unconfirmed_cli_backup_do_not_execute_or_write_audit() {
        let root = tempfile::tempdir().unwrap();
        preferences_fixture(root.path());
        let mut service = IpcService::for_tests_with(root.path().into(), root.path().into());
        let mut request = backup_request(IpcCommand::BackupCreate);
        request.confirm = false;
        let response = service.execute(request).await;
        assert_eq!(response.error.unwrap().code, IpcErrorCode::RequireConfirm);
        let binding = service.writers.freeze().unwrap().unwrap();
        for command in [IpcCommand::BackupCreate, IpcCommand::BackupList] {
            let response = service.execute(backup_request(command)).await;
            assert_eq!(
                response.error.unwrap().code,
                IpcErrorCode::TargetStateConflict
            );
        }
        assert!(!root.path().join("audit.log").exists());
        assert!(!root.path().join(".backup-w2.lock").exists());
        assert!(!root.path().join("backups").exists());
        drop(binding);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn backup_list_lock_wait_keeps_executor_responsive_and_survives_observer_drop() {
        use std::time::{Duration, Instant};
        let root = tempfile::tempdir().unwrap();
        let lock = crate::infrastructure::platform::create_new_private_file(
            &root.path().join(".backup-w2.lock"),
            0o600,
        )
        .unwrap();
        lock.lock().unwrap();
        let (release, released) = std::sync::mpsc::channel();
        // A watchdog releases the real lock even if a regression blocks this
        // single-thread async executor; this is not a native performance budget.
        let holder = std::thread::spawn(move || {
            let _ = released.recv_timeout(Duration::from_secs(2));
            drop(lock);
        });
        let mut service = IpcService::for_tests_with(root.path().into(), root.path().into());
        let writers = service.writers.clone();
        let started = Instant::now();
        let observer = tokio::spawn(async move {
            service
                .execute(backup_request(IpcCommand::BackupList))
                .await
        });
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(started.elapsed() < Duration::from_secs(1));
        assert!(!observer.is_finished());
        observer.abort();
        assert!(observer.await.unwrap_err().is_cancelled());
        assert!(writers.freeze().unwrap().is_none());
        release.send(()).unwrap();
        holder.join().unwrap();
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                if writers.freeze().unwrap().is_some() {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let audit = audit_records(root.path());
        assert_eq!(audit.len(), 1);
        assert_eq!(audit[0].command, IpcCommand::BackupList);
        assert_eq!(audit[0].result, super::super::AuditResult::Succeeded);
        assert!(!root.path().join("backups").exists());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn public_status_lock_wait_keeps_executor_responsive_and_survives_disconnect() {
        use std::time::{Duration, Instant};
        let root = tempfile::tempdir().unwrap();
        let mut service = IpcService::for_tests_with(root.path().into(), root.path().into());
        let collector = service.collector.clone();
        let writers = service.writers.clone();
        let (entered, entry) = tokio::sync::oneshot::channel();
        let (release, released) = std::sync::mpsc::channel();
        let holder = std::thread::spawn(move || {
            let _collector = collector.lock().unwrap();
            entered.send(()).unwrap();
            let _ = released.recv_timeout(Duration::from_secs(2));
        });
        tokio::time::timeout(Duration::from_secs(2), entry)
            .await
            .unwrap()
            .unwrap();
        let started = Instant::now();
        let observer =
            tokio::spawn(async move { service.execute(backup_request(IpcCommand::Status)).await });
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(started.elapsed() < Duration::from_secs(1));
        assert!(!observer.is_finished());
        assert!(writers.freeze().unwrap().is_none());
        observer.abort();
        assert!(observer.await.unwrap_err().is_cancelled());
        assert!(writers.freeze().unwrap().is_none());
        assert!(!root.path().join("audit.log").exists());
        release.send(()).unwrap();
        holder.join().unwrap();
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                if writers.freeze().unwrap().is_some() {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let records = audit_records(root.path());
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].command, IpcCommand::Status);
        assert_eq!(records[0].result, super::super::AuditResult::Succeeded);
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn public_binding_lock_wait_persists_v2_fixed_pointer_after_disconnect() {
        use crate::modules::data_root;
        use std::time::Duration;
        let t = tempfile::tempdir().unwrap();
        let mut service = v2_binding_service(&t.path().join("user"), None);
        let root = service.dependencies.active_data_root.clone();
        let anchor = service.dependencies.app_data_root.clone();
        let target = t.path().join("selected");
        data_root::initialize(&target.join("desktop")).unwrap();
        let before = std::fs::read(anchor.join("manager-location.json")).unwrap();
        let writers = service.writers.clone();
        let mutations = service.runtime_mutations.clone();
        // Retain real service locks after the disconnected service clone exits.
        let _context = service.binding_context.clone();
        let lock = crate::infrastructure::platform::create_new_private_file(
            &root.join(".backup-w2.lock"),
            0o600,
        )
        .unwrap();
        lock.lock().unwrap();
        let (release, released) = std::sync::mpsc::channel();
        let holder = std::thread::spawn(move || {
            let _ = released.recv_timeout(Duration::from_secs(3));
            drop(lock);
        });
        let request = reference_request(&target);
        let observer = tokio::spawn(async move { service.execute(request).await });
        tokio::time::sleep(Duration::from_millis(30)).await;
        assert!(!observer.is_finished());
        assert!(writers.frozen());
        assert!(mutations.acquire().is_none());
        observer.abort();
        assert!(observer.await.unwrap_err().is_cancelled());
        assert_eq!(
            std::fs::read(anchor.join("manager-location.json")).unwrap(),
            before
        );
        release.send(()).unwrap();
        holder.join().unwrap();
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                if mutations.acquire().is_some() {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert!(writers.frozen());
        assert!(writers.admit().is_err());
        assert_eq!(
            data_root::startup_v2::selected_config(&anchor)
                .unwrap()
                .active_data_root,
            data_root::startup_v2::checked_path(&target.join("desktop")).unwrap()
        );
        assert!(!target.join("desktop/manager-location.json").exists());
        assert_eq!(
            data_root::load_runtime_config(&root)
                .unwrap()
                .active_data_root,
            root
        );
        let audit = audit_records(&root);
        assert_eq!(audit.len(), 1);
        assert_eq!(audit[0].result, super::super::AuditResult::Succeeded);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cli_update_query_survives_disconnect_through_commit_and_terminal_audit() {
        use crate::infrastructure::storage_writers::WriterGate;
        use std::time::Duration;
        for terminal in [0, 1, 2] {
            let root = tempfile::tempdir().unwrap();
            let writers = Arc::new(WriterGate::default());
            let worker_gate = writers.clone();
            let worker_root = root.path().to_path_buf();
            let (entered, entry) = tokio::sync::oneshot::channel();
            let (release, released) = tokio::sync::oneshot::channel();
            let observer = tokio::spawn(async move {
                run_ipc_update_owned(
                    worker_gate,
                    worker_root.clone(),
                    backup_request(IpcCommand::UpdateCheck),
                    chrono::Utc::now(),
                    move || async move {
                        entered.send(()).unwrap();
                        released.await.unwrap();
                        match terminal {
                            0 => {
                                crate::commands::run_readonly("fixture update cache", move || {
                                    let status =
                                        crate::modules::update::UpdateStatus::pending("0.1.9");
                                    crate::commands::update_schedule::complete_at_root(
                                        &worker_root,
                                        crate::modules::update::schedule::Target::ManagerStable,
                                        Some(&status),
                                        200,
                                    )
                                })
                                .await
                                .map_err(map_app_error)?;
                                Ok(serde_json::json!({"status":"up_to_date"}))
                            }
                            1 => Err(IpcErrorCode::ExecutionFailed),
                            _ => panic!("isolated CLI update panic"),
                        }
                    },
                )
                .await
            });
            tokio::time::timeout(Duration::from_secs(2), entry)
                .await
                .unwrap()
                .unwrap();
            observer.abort();
            assert!(observer.await.unwrap_err().is_cancelled());
            assert!(writers.freeze().unwrap().is_none());
            assert!(!root.path().join("audit.log").exists());
            tokio::time::timeout(Duration::from_secs(1), tokio::task::yield_now())
                .await
                .unwrap();
            release.send(()).unwrap();
            tokio::time::timeout(Duration::from_secs(2), async {
                writers.wait_until_idle_for_test().await.unwrap();
                assert!(writers.freeze().unwrap().is_some());
            })
            .await
            .unwrap();
            let records = audit_records(root.path());
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].command, IpcCommand::UpdateCheck);
            assert_eq!(
                records[0].error_code,
                match terminal {
                    0 => None,
                    1 => Some(IpcErrorCode::ExecutionFailed),
                    _ => Some(IpcErrorCode::InternalError),
                }
            );
            assert_eq!(
                root.path()
                    .join("cache/updates/manager-stable.json")
                    .exists(),
                terminal == 0
            );
        }
    }

    #[tokio::test]
    async fn cli_update_requires_real_gui_and_frozen_query_has_no_side_effects() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let root = tempfile::tempdir().unwrap();
        let mut service = IpcService::for_tests_with(root.path().into(), root.path().into());
        let mut request = backup_request(IpcCommand::UpdateCheck);
        request.confirm = false; // Metadata check remains a no-confirm CLI action.
        let response = service.execute(request).await;
        assert!(!response.ok);
        assert_eq!(response.error.unwrap().code, IpcErrorCode::TargetNotFound);
        assert_eq!(
            audit_records(root.path())[0].error_code,
            Some(IpcErrorCode::TargetNotFound)
        );
        let before = std::fs::read(root.path().join("audit.log")).unwrap();
        service.writers.freeze().unwrap().unwrap().commit();
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        assert_eq!(
            run_ipc_update_owned(
                service.writers.clone(),
                root.path().into(),
                backup_request(IpcCommand::UpdateCheck),
                chrono::Utc::now(),
                move || {
                    counter.fetch_add(1, Ordering::SeqCst);
                    async { Ok(serde_json::Value::Null) }
                }
            )
            .await,
            Err(IpcErrorCode::TargetStateConflict)
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            std::fs::read(root.path().join("audit.log")).unwrap(),
            before
        );
        assert!(!root.path().join("manager-state").exists());
        assert!(!root.path().join("cache").exists());
    }

    #[test]
    fn cli_update_projects_truthful_terminal_status_and_keeps_three_field_dto() {
        for terminal in [
            "available",
            "up_to_date",
            "failed",
            "superseded",
            "not_checked",
        ] {
            let mut status = crate::modules::update::UpdateStatus::pending("0.1.9");
            status.available_version = Some("0.1.10".into());
            status.notes = Some("never exposed".into());
            let projected = project_ipc_update(crate::types::update::CheckUpdateResultDto {
                status: terminal.into(),
                update: status.into(),
            });
            match terminal {
                "available" | "up_to_date" => {
                    let value = projected.unwrap();
                    assert_eq!(value.as_object().unwrap().len(), 3);
                    assert_eq!(value["status"], terminal);
                    assert_eq!(value["available_version"], "0.1.10");
                    assert!(value["error"].is_null());
                    assert!(!value.to_string().contains("never exposed"));
                }
                "failed" => assert_eq!(projected, Err(IpcErrorCode::ExecutionFailed)),
                "superseded" => assert_eq!(projected, Err(IpcErrorCode::TargetStateConflict)),
                _ => assert_eq!(projected, Err(IpcErrorCode::InternalError)),
            }
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cli_queries_survive_disconnect_and_audit_only_admitted_roots() {
        use crate::infrastructure::storage_writers::WriterGate;
        use std::time::{Duration, Instant};
        for command in [
            IpcCommand::Status,
            IpcCommand::DataRootShow,
            IpcCommand::BackupList,
        ] {
            for terminal in [0, 1, 2] {
                let root = tempfile::tempdir().unwrap();
                let writers = Arc::new(WriterGate::default());
                let (entered, entry) = tokio::sync::oneshot::channel();
                let (release, released) = std::sync::mpsc::channel();
                let worker_gate = writers.clone();
                let worker_root = root.path().to_path_buf();
                let observer = tokio::spawn(async move {
                    run_ipc_query_owned(
                        worker_gate,
                        worker_root,
                        backup_request(command),
                        chrono::Utc::now(),
                        move || {
                            entered.send(()).unwrap();
                            released.recv_timeout(Duration::from_secs(2)).unwrap();
                            match terminal {
                                0 => Ok(serde_json::json!({"status": "fixture"})),
                                1 => Err(IpcErrorCode::ExecutionFailed),
                                _ => panic!("isolated query panic"),
                            }
                        },
                    )
                    .await
                });
                tokio::time::timeout(Duration::from_secs(2), entry)
                    .await
                    .unwrap()
                    .unwrap();
                let timer = Instant::now();
                tokio::time::sleep(Duration::from_millis(20)).await;
                assert!(timer.elapsed() < Duration::from_secs(1));
                observer.abort();
                assert!(observer.await.unwrap_err().is_cancelled());
                assert!(writers.freeze().unwrap().is_none());
                release.send(()).unwrap();
                tokio::time::timeout(Duration::from_secs(2), async {
                    writers.wait_until_idle_for_test().await.unwrap();
                    assert!(writers.freeze().unwrap().is_some());
                })
                .await
                .unwrap();
                let records = audit_records(root.path());
                assert_eq!(records.len(), 1);
                assert_eq!(records[0].command, command);
                assert_eq!(
                    records[0].error_code,
                    match terminal {
                        0 => None,
                        1 => Some(IpcErrorCode::ExecutionFailed),
                        _ => Some(IpcErrorCode::InternalError),
                    }
                );
            }
        }
        let root = tempfile::tempdir().unwrap();
        let mut service = IpcService::for_tests_with(root.path().into(), root.path().into());
        service.writers.freeze().unwrap().unwrap().commit();
        for command in [IpcCommand::Status, IpcCommand::DataRootShow] {
            assert!(service.execute(backup_request(command)).await.ok);
        }
        assert_eq!(
            service
                .execute(backup_request(IpcCommand::UpdateCheck))
                .await
                .error
                .unwrap()
                .code,
            IpcErrorCode::TargetStateConflict
        );
        assert_eq!(
            service
                .execute(backup_request(IpcCommand::BackupList))
                .await
                .error
                .unwrap()
                .code,
            IpcErrorCode::TargetStateConflict
        );
        assert!(!root.path().join("audit.log").exists());
        assert!(!root.path().join(".backup-w2.lock").exists());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cli_binding_owns_runtime_and_exclusive_storage_until_terminal_audit() {
        use crate::infrastructure::storage_writers::WriterGate;
        use std::time::Duration;
        // A saved pending reference must stay frozen even if projection later
        // fails or panics. No-save success, failure and panic reopen writers.
        for saved in [false, true] {
            for terminal in [0, 1, 2] {
                let root = tempfile::tempdir().unwrap();
                let writers = Arc::new(WriterGate::default());
                let mutations = crate::state::SharedRuntimeInstall::default();
                let worker_gate = writers.clone();
                let worker_mutations = mutations.clone();
                let worker_root = root.path().to_path_buf();
                let (entered, entry) = tokio::sync::oneshot::channel();
                let (release, released) = std::sync::mpsc::channel();
                let observer = tokio::spawn(async move {
                    run_ipc_binding_owned(
                        worker_gate,
                        worker_mutations,
                        worker_root,
                        backup_request(IpcCommand::DataRootSwitch),
                        chrono::Utc::now(),
                        move |pending| {
                            *pending = saved;
                            entered.send(()).unwrap();
                            released.recv_timeout(Duration::from_secs(2)).unwrap();
                            match terminal {
                                0 => Ok(serde_json::Value::Null),
                                1 => Err(IpcErrorCode::ExecutionFailed),
                                _ => panic!("isolated binding projection panic"),
                            }
                        },
                    )
                    .await
                });
                tokio::time::timeout(Duration::from_secs(2), entry)
                    .await
                    .unwrap()
                    .unwrap();
                observer.abort();
                assert!(observer.await.unwrap_err().is_cancelled());
                assert!(writers.admit().is_err());
                assert!(writers.freeze().unwrap().is_none());
                assert!(mutations.acquire().is_none());
                release.send(()).unwrap();
                tokio::time::timeout(Duration::from_secs(2), async {
                    loop {
                        if mutations.acquire().is_some() {
                            break;
                        }
                        tokio::task::yield_now().await;
                    }
                })
                .await
                .unwrap();
                assert_eq!(writers.frozen(), saved);
                assert_eq!(writers.admit().is_err(), saved);
                let records = audit_records(root.path());
                assert_eq!(records.len(), 1);
                assert_eq!(records[0].command, IpcCommand::DataRootSwitch);
                assert_eq!(
                    records[0].error_code,
                    match terminal {
                        0 => None,
                        1 => Some(IpcErrorCode::ExecutionFailed),
                        _ => Some(IpcErrorCode::InternalError),
                    }
                );
            }
        }
    }

    #[tokio::test]
    async fn cli_binding_busy_frozen_and_unconfirmed_never_execute_or_audit() {
        use crate::infrastructure::storage_writers::WriterGate;
        use std::sync::atomic::{AtomicUsize, Ordering};
        let root = tempfile::tempdir().unwrap();
        let writers = Arc::new(WriterGate::default());
        let mutations = crate::state::SharedRuntimeInstall::default();
        let calls = Arc::new(AtomicUsize::new(0));
        for blocked in 0..3 {
            let runtime = if blocked == 0 {
                mutations.acquire()
            } else {
                None
            };
            let storage = if blocked == 1 {
                Some(writers.admit().unwrap())
            } else {
                None
            };
            let binding = if blocked == 2 {
                writers.freeze().unwrap()
            } else {
                None
            };
            let counter = calls.clone();
            assert_eq!(
                run_ipc_binding_owned(
                    writers.clone(),
                    mutations.clone(),
                    root.path().into(),
                    backup_request(IpcCommand::DataRootSwitch),
                    chrono::Utc::now(),
                    move |_| {
                        counter.fetch_add(1, Ordering::SeqCst);
                        Ok(serde_json::Value::Null)
                    },
                )
                .await,
                Err(IpcErrorCode::TargetStateConflict)
            );
            drop((runtime, storage, binding));
            assert!(mutations.acquire().is_some());
            assert!(writers.admit().is_ok());
        }
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let mut service = IpcService::for_tests_with(root.path().into(), root.path().into());
        let mut request = backup_request(IpcCommand::DataRootSwitch);
        request
            .args
            .insert("target".into(), root.path().display().to_string());
        request.confirm = false;
        assert_eq!(
            service.execute(request).await.error.unwrap().code,
            IpcErrorCode::RequireConfirm
        );
        assert!(!root.path().join("audit.log").exists());
    }

    /// 回归：CLI 切换数据根必须把引用写回固定的应用数据目录。
    ///
    /// 此前写的是「当前活动根」，于是第二次切换会落在旧目标里；
    /// 启动只从应用数据目录单跳读取，重启后第二次切换就丢了。
    #[cfg(unix)]
    #[tokio::test(flavor = "multi_thread")]
    async fn switch_data_root_uses_fixed_d_and_target_u_across_actual_ipc_restart() {
        use crate::modules::data_root;
        let t = tempfile::tempdir().unwrap();
        let user = t.path().join("user");
        let first = t.path().join("first");
        let second = t.path().join("second");
        for target in [&first, &second] {
            data_root::initialize(&target.join("desktop")).unwrap();
        }
        let mut service = v2_binding_service(&user, None);
        let anchor = service.dependencies.app_data_root.clone();
        assert!(service.execute(reference_request(&first)).await.ok);
        drop(service);
        let mut service = v2_binding_service(&user, None);
        let current_d = service.dependencies.active_data_root.clone();
        let current_h = service.dependencies.opencodex_home.clone();
        assert_eq!(
            current_d,
            data_root::startup_v2::checked_path(&first.join("desktop")).unwrap()
        );
        let metadata = std::fs::read(data_root::runtime_config_path(&current_d)).unwrap();
        let target_metadata =
            std::fs::read(data_root::runtime_config_path(&second.join("desktop"))).unwrap();
        let response = service.execute(reference_request(&second)).await;
        assert!(response.ok, "{:?}", response.error);
        assert_eq!(response.data.unwrap()["status"], "restart_required");
        assert!(service.writers.frozen());
        assert_eq!(
            std::fs::read(data_root::runtime_config_path(&current_d)).unwrap(),
            metadata
        );
        assert_eq!(
            std::fs::read(data_root::runtime_config_path(&second.join("desktop"))).unwrap(),
            target_metadata
        );
        assert!(!current_d.join("manager-location.json").exists());
        assert!(!second.join("desktop/manager-location.json").exists());
        let pointer: serde_json::Value =
            serde_json::from_slice(&std::fs::read(anchor.join("manager-location.json")).unwrap())
                .unwrap();
        assert_eq!(
            pointer["activeRoot"],
            data_root::startup_v2::checked_path(&second)
                .unwrap()
                .display()
                .to_string()
        );
        let before_audit = std::fs::read(current_d.join("audit.log")).unwrap();
        let show = service
            .execute(backup_request(IpcCommand::DataRootShow))
            .await;
        assert!(show.ok);
        assert_eq!(
            show.data.unwrap()["opencodex_home"],
            current_h.display().to_string()
        );
        assert_eq!(
            service
                .execute(backup_request(IpcCommand::BackupCreate))
                .await
                .error
                .unwrap()
                .code,
            IpcErrorCode::TargetStateConflict
        );
        assert_eq!(
            std::fs::read(current_d.join("audit.log")).unwrap(),
            before_audit
        );
        drop(service);
        let restarted = v2_binding_service(&user, None);
        assert_eq!(restarted.dependencies.app_data_root, anchor);
        assert_eq!(
            restarted.dependencies.active_data_root,
            data_root::startup_v2::checked_path(&second.join("desktop")).unwrap()
        );
        assert_eq!(
            restarted.dependencies.opencodex_home,
            data_root::startup_v2::checked_path(&second.join("opencodex/home")).unwrap()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn binding_refuses_busy_writer_and_validation_failure_reopens_admission() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("desktop");
        crate::modules::data_root::initialize(&root).unwrap();
        let mut service = IpcService::for_tests_with(root.clone(), root.clone());
        let target = t.path().join("missing");
        let active = service.writers.admit().unwrap();
        assert_eq!(
            service
                .execute(reference_request(&target))
                .await
                .error
                .unwrap()
                .code,
            IpcErrorCode::TargetStateConflict
        );
        assert!(!service.writers.frozen());
        drop(active);
        assert!(!service.execute(reference_request(&target)).await.ok);
        assert!(service.writers.admit().is_ok());
        assert!(!target.exists());
    }

    #[test]
    fn passphrase_required_is_reported_distinctly() {
        assert_eq!(
            map_app_error(crate::errors::AppError::PassphraseRequired),
            IpcErrorCode::PassphraseRequired
        );
        assert_eq!(
            map_app_error(crate::errors::AppError::NotFound {
                entity: "container".to_string()
            }),
            IpcErrorCode::TargetNotFound
        );
        assert_eq!(IpcErrorCode::PassphraseRequired.exit_code(), 13);
        assert!(
            crate::modules::ipc::error_message(IpcErrorCode::PassphraseRequired)
                .to_lowercase()
                .contains("passphrase")
        );
    }

    #[tokio::test]
    async fn cli_root_migrate_is_unsupported_before_status_audit_or_target_creation() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("not-created-desktop");
        let target = t.path().join("not-created-target");
        let mut service = IpcService::for_tests_with(root.clone(), root.clone());
        let binding = service.writers.freeze().unwrap().unwrap();
        let mut request = reference_request(&target);
        request.args.insert("migrate".into(), "true".into());
        let response = service.execute(request).await;
        assert_eq!(
            response.error.as_ref().unwrap().code,
            IpcErrorCode::ValidationFailed
        );
        assert!(response.error.unwrap().message.contains("unsupported"));
        assert!(!root.exists() && !target.exists());
        assert!(service.runtime_mutations.acquire().is_some());
        drop(binding);
        assert!(service.writers.admit().is_ok());
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "multi_thread")]
    async fn actual_ipc_reference_checks_sandbox_installation_update_and_current_locks() {
        use crate::modules::data_root;
        for rejected in [0, 1, 2, 3, 4] {
            let t = tempfile::tempdir().unwrap();
            let sandbox = t.path().canonicalize().unwrap().join("sandbox");
            let mut service = v2_binding_service(&t.path().join("unused-user"), Some(&sandbox));
            let anchor = service.dependencies.app_data_root.clone();
            let pointer = std::fs::read(anchor.join("manager-location.json")).unwrap();
            let target = if rejected == 0 {
                t.path().join("outside")
            } else if rejected == 1 {
                sandbox.join("program/forbidden-data")
            } else {
                sandbox.join("selected")
            };
            if rejected != 1 {
                data_root::initialize(&target.join("desktop")).unwrap();
            }
            if rejected == 2 {
                data_root::set_opencodex_home(
                    &target.join("desktop"),
                    data_root::OpenCodexHomeMode::External,
                    Some(&t.path().join("outside-home")),
                )
                .unwrap();
            }
            if rejected == 3 {
                service
                    .binding_context
                    .as_ref()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .update_pending = true;
            }
            if rejected == 4 {
                service
                    .binding_context
                    .as_ref()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .startup
                    .locks
                    .pop();
            }
            {
                let mut collector = service.collector.lock().unwrap();
                collector.refresh().unwrap();
                assert_eq!(
                    collector.matrix().runtime,
                    crate::types::status::RuntimeState::Stopped
                );
            }
            let response = service.execute(reference_request(&target)).await;
            assert!(!response.ok, "rejected case {rejected}");
            assert_eq!(
                response.error.unwrap().code,
                if rejected <= 2 {
                    IpcErrorCode::ValidationFailed
                } else {
                    IpcErrorCode::TargetStateConflict
                },
                "case {rejected}"
            );
            assert_eq!(
                std::fs::read(anchor.join("manager-location.json")).unwrap(),
                pointer
            );
            assert!(service.writers.admit().is_ok());
            assert!(!target.join("desktop/manager-location.json").exists());
            assert!(!t.path().join("outside-home").exists());
            assert!(!t.path().join("unused-user").exists());
            if rejected == 1 {
                assert!(!target.exists());
            }
        }
    }
}
