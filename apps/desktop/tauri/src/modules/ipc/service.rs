//! MOD-12 IPC 委托执行与命令验证。
//!
//! CLI 是运行中 GUI 的客户端：只读取共享快照，生命周期动作复用共享
//! runner；同步由持有存储与同步准入的后台执行者完成核验、反馈及审计。

use serde::Serialize;
use std::collections::BTreeMap;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

use crate::modules::ipc::{
    validate_request, IpcCommand, IpcError, IpcErrorCode, IpcRequest, IpcResponse,
};
use crate::modules::process::{LifecycleAction, LifecycleResult, ProcessCommand, ProcessRunner};
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
    /// 固定的应用数据目录（`app_data_dir`）。运行期配置只从这里读取，
    /// 因此切换数据根时也必须写回这里，否则第二次切换重启后会丢失。
    pub app_data_root: std::path::PathBuf,
    /// 用户主目录：同步要把 Skills 软链接投射到各客户端的真实目录。
    pub home: std::path::PathBuf,
    pub opencodex_home: std::path::PathBuf,
    pub external_home: Option<std::path::PathBuf>,
    pub current_version: &'static str,
    pub runtime_log: crate::infrastructure::runtime_log::RuntimeLog,
}

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
    audit: super::audit::AuditStore,
    writers: Arc<crate::infrastructure::storage_writers::WriterGate>,
    sync_operations: Arc<tokio::sync::Mutex<()>>,
    app: Option<tauri::AppHandle>,
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
        let audit =
            super::audit::AuditStore::with_writers(&dependencies.active_data_root, writers.clone());
        Self {
            collector,
            runner,
            dependencies,
            audit,
            writers,
            sync_operations: crate::commands::sync::sync_operation_gate(),
            app: None,
        }
    }

    /// Bind the running GUI's shared status and notification store. The CLI must
    /// never instantiate its own GUI state or a second notification publisher.
    #[cfg(unix)]
    pub(crate) fn with_app_handle(mut self, app: tauri::AppHandle) -> Self {
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
        service.audit = super::audit::AuditStore::with_writers(
            &service.dependencies.active_data_root,
            service.writers.clone(),
        );
        service
    }

    pub async fn execute(&mut self, request: IpcRequest) -> IpcResponse<serde_json::Value> {
        let started = chrono::Utc::now();
        let request = match validate_request(request) {
            Ok(value) => value,
            Err(response) => return response,
        };
        // A disconnected/cancelled observer must not cancel an admitted mutation or
        // release its old-root admission before terminal feedback and audit.
        if request.command == IpcCommand::SyncRun {
            let request_id = request.request_id.clone();
            return owned_response(request_id, self.sync_run(request, started).await);
        }
        if request.command == IpcCommand::BackupCreate {
            let request_id = request.request_id.clone();
            return owned_response(request_id, self.create_backup(request, started).await);
        }
        if matches!(request.command, IpcCommand::Export | IpcCommand::Import) {
            let request_id = request.request_id.clone();
            return owned_response(request_id, self.migrate_config(request, started).await);
        }
        let request_id = request.request_id.clone();
        let request_id = if request_id.starts_with("req_") {
            request_id
        } else {
            format!("req_{request_id}")
        };
        let is_binding = request.command == IpcCommand::DataRootSwitch;
        let read_only = matches!(
            request.command,
            IpcCommand::Status
                | IpcCommand::DataRootShow
                | IpcCommand::BackupList
                | IpcCommand::UpdateCheck
        );
        let binding = if is_binding {
            self.writers.freeze().ok().flatten()
        } else {
            None
        };
        let admission = if is_binding {
            None
        } else {
            self.writers.admit().ok()
        };
        let blocked =
            (is_binding && binding.is_none()) || (!is_binding && !read_only && admission.is_none());
        let mut saved_pending = false;
        let result: Result<serde_json::Value, IpcErrorCode> = if blocked {
            Err(IpcErrorCode::TargetStateConflict)
        } else {
            match request.command {
                IpcCommand::Status => self.read_status(),
                IpcCommand::DataRootShow => self.read_data_root(),
                IpcCommand::BackupList => self.list_backups().await,
                IpcCommand::UpdateCheck => self.update_check(),
                IpcCommand::Start | IpcCommand::Stop | IpcCommand::Restart => {
                    let action = match request.command {
                        IpcCommand::Start => LifecycleAction::Start,
                        IpcCommand::Stop => LifecycleAction::Stop,
                        _ => LifecycleAction::Restart,
                    };
                    self.run_lifecycle(action, request.args.clone())
                }
                IpcCommand::DataRootSwitch => {
                    self.switch_data_root(&request.args, &mut saved_pending)
                }
                IpcCommand::BackupCreate => unreachable!("backup uses its owned execution path"),
                IpcCommand::Export | IpcCommand::Import => {
                    unreachable!("migration uses its owned execution path")
                }
                IpcCommand::SyncRun => unreachable!("sync uses its owned execution path"),
            }
        };
        let (result, error_code) = match result {
            Ok(data) => (Some(data), None),
            Err(error_code) => (None, Some(error_code)),
        };
        let finished = chrono::Utc::now();
        let audit = super::AuditRecord::from_request(
            &request,
            super::AuditSource::Cli,
            started,
            finished,
            if error_code.is_some() {
                super::AuditResult::Failed
            } else {
                super::AuditResult::Succeeded
            },
            error_code,
        );
        if let Some(binding) = binding {
            // Audit is the final old-root write within the exclusive binding transaction.
            let _ = self.audit.record_during_binding(&audit, &binding);
            if saved_pending {
                binding.commit();
            }
        } else if admission.is_some() {
            // Read-only requests remain usable after save, without writing old-root audit.
            let _ = self.audit.record(&audit);
        }

        match result {
            Some(data) => IpcResponse {
                request_id,
                ok: true,
                data: Some(data),
                error: None,
            },
            None => IpcResponse {
                request_id,
                ok: false,
                data: None,
                error: Some(IpcError {
                    code: error_code.unwrap_or(IpcErrorCode::InternalError),
                    message: crate::modules::ipc::error_message(
                        error_code.unwrap_or(IpcErrorCode::InternalError),
                    )
                    .to_string(),
                }),
            },
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

    async fn list_backups(&self) -> Result<serde_json::Value, IpcErrorCode> {
        // Creating the cooperative lock is a write even for this metadata query.
        // Lock waits and the bounded manifest scan stay off the async executor.
        // The worker owns admission even when its query observer disconnects.
        let root = self.dependencies.active_data_root.clone();
        let records = crate::commands::run_blocking_with_gate(
            self.writers.clone(),
            "CLI backup list",
            move || crate::modules::backup::manager::list_records(&root),
        )
        .await
        .map_err(map_backup_worker_error)?;
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

    fn update_check(&self) -> Result<serde_json::Value, IpcErrorCode> {
        let status =
            crate::modules::update::UpdateStatus::pending(self.dependencies.current_version);
        serde_json::to_value(IpcUpdateData {
            status: "not_checked".to_string(),
            available_version: status.available_version,
            error: status.error,
        })
        .map_err(|_| IpcErrorCode::InternalError)
    }

    fn switch_data_root(
        &self,
        args: &BTreeMap<String, String>,
        saved_pending: &mut bool,
    ) -> Result<serde_json::Value, IpcErrorCode> {
        let target = args.get("target").ok_or(IpcErrorCode::ValidationFailed)?;
        let mode = match args.get("migrate").map(String::as_str) {
            Some("true") => crate::modules::data_root::DataRootSwitchMode::MigrateData,
            Some("false") | None => crate::modules::data_root::DataRootSwitchMode::ReferenceOnly,
            Some(_) => crate::modules::data_root::DataRootSwitchMode::MigrateData,
        };
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
        let _transaction = crate::modules::backup::manager::acquire_preferences_transaction(
            &self.dependencies.active_data_root,
        )
        .map_err(map_app_error)?;
        // 与 GUI 命令保持一致：把引用写回固定的应用数据目录。
        // 写到「当前活动根」会让第二次切换落在旧目标里，重启后按单跳读取就丢失。
        let config = crate::modules::data_root::switch_reference(
            &self.dependencies.app_data_root,
            std::path::Path::new(target),
            mode,
        )
        .map_err(map_app_error)?;
        *saved_pending = !crate::modules::data_root::runtime_binding_matches(
            &config,
            &self.dependencies.active_data_root,
            &self.dependencies.opencodex_home,
        );
        serde_json::to_value(IpcMessageData {
            status: "switched".to_string(),
            message: if matches!(
                mode,
                crate::modules::data_root::DataRootSwitchMode::ReferenceOnly
            ) {
                "Data root reference switched; restart is required."
            } else {
                "Data root switched; restart is required."
            }
            .to_string(),
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

    async fn migrate_config(
        &self,
        request: IpcRequest,
        started: chrono::DateTime<chrono::Utc>,
    ) -> Result<serde_json::Value, IpcErrorCode> {
        let dependencies = self.dependencies.clone();
        let worker_request = request.clone();
        let operation = match request.command {
            IpcCommand::Export => "CLI config export",
            IpcCommand::Import => "CLI config import",
            _ => unreachable!("migration accepts only export/import"),
        };
        run_ipc_storage_owned(
            self.writers.clone(),
            dependencies.active_data_root.clone(),
            request,
            started,
            operation,
            move || match worker_request.command {
                IpcCommand::Export => Self::export_config(&dependencies, &worker_request.args),
                IpcCommand::Import => Self::import_config(
                    &dependencies,
                    &worker_request.args,
                    worker_request.secret.as_deref(),
                ),
                _ => unreachable!("migration accepts only export/import"),
            },
        )
        .await
    }

    fn export_config(
        dependencies: &IpcDependencies,
        args: &BTreeMap<String, String>,
    ) -> Result<serde_json::Value, IpcErrorCode> {
        if args.contains_key("password") {
            return Err(IpcErrorCode::ValidationFailed);
        }
        let output = args.get("output").ok_or(IpcErrorCode::ValidationFailed)?;
        // 新版明文容器不需要口令；不再读取标准输入内容。
        let result = crate::modules::migration::export_with_container_file(
            &dependencies.active_data_root,
            std::path::Path::new(output),
            dependencies.current_version,
        )
        .map_err(map_app_error)?;
        Ok(serde_json::to_value(IpcExportData {
            path: result.path.display().to_string(),
            backup_id: result.backup_id,
            document_sha256: result.document_sha256,
            format_version: result.format_version,
            sections: result.sections,
        })
        .unwrap_or(serde_json::Value::Null))
    }

    fn import_config(
        dependencies: &IpcDependencies,
        args: &BTreeMap<String, String>,
        secret: Option<&str>,
    ) -> Result<serde_json::Value, IpcErrorCode> {
        if args.contains_key("password") {
            return Err(IpcErrorCode::ValidationFailed);
        }
        let input = args.get("input").ok_or(IpcErrorCode::ValidationFailed)?;
        // 只有旧版加密容器才需要口令；未提供时由迁移模块返回 PassphraseRequired，
        // 调用方据此提示输入，而不是把它当成普通失败（此前被压成 ExecutionFailed）。
        let passphrase = secret
            .map(str::to_string)
            .or_else(|| std::env::var("OCXD_PASSPHRASE").ok());
        let result = crate::modules::migration::import_with_container_file(
            &dependencies.active_data_root,
            std::path::Path::new(input),
            &dependencies.home,
            passphrase.as_deref(),
        )
        .map_err(map_app_error)?;
        Ok(serde_json::to_value(IpcImportData {
            backup_id: result.backup_id,
            document_sha256: result.document_sha256,
            format_version: result.format_version,
            applied_sections: result.applied_sections,
            skipped_sections: result
                .skipped_sections
                .into_iter()
                .map(|item| item.section)
                .collect(),
        })
        .unwrap_or(serde_json::Value::Null))
    }

    async fn sync_run(
        &self,
        request: IpcRequest,
        started: chrono::DateTime<chrono::Utc>,
    ) -> Result<serde_json::Value, IpcErrorCode> {
        let root = self.dependencies.active_data_root.clone();
        let home = self.dependencies.home.clone();
        let app = self.app.clone();
        let worker_root = root.clone();
        run_ipc_sync_owned(
            self.writers.clone(),
            self.sync_operations.clone(),
            root,
            request,
            started,
            move || {
                project_ipc_sync(crate::commands::sync::execute_sync_now(
                    app.as_ref(),
                    &worker_root,
                    &home,
                ))
            },
        )
        .await
    }

    fn run_lifecycle(
        &self,
        action: LifecycleAction,
        args: BTreeMap<String, String>,
    ) -> Result<serde_json::Value, IpcErrorCode> {
        if args.contains_key("force") || args.contains_key("skip_backup") {
            return Err(IpcErrorCode::ValidationFailed);
        }
        let mut guard = self
            .runner
            .lock()
            .map_err(|_| IpcErrorCode::InternalError)?;
        let runner = guard.deref_mut();
        let command = ProcessCommand::new(
            action,
            self.dependencies
                .process_context
                .executable()
                .map_err(|_| IpcErrorCode::InternalError)?,
            self.dependencies.process_context.working_directory.clone(),
            self.dependencies.process_context.opencodex_home.clone(),
        );
        let started_at = std::time::Instant::now();
        let outcome = runner.execute(&command);
        let duration_ms = started_at.elapsed().as_millis();
        let log_result = match &outcome {
            Ok(LifecycleResult::Started) => Ok("started"),
            Ok(LifecycleResult::Stopped) => Ok("stop completed"),
            Ok(LifecycleResult::Cancelled) => Err("cancelled".to_string()),
            Ok(LifecycleResult::Failed) => Err("failed".to_string()),
            Err(error) => Err(error.to_string()),
        };
        let _ = self.dependencies.runtime_log.append_result(
            action_log_label(action),
            log_result.map_err(|error| format!("{error}; duration_ms={duration_ms}")),
        );
        // 只有真正完成才回成功。取消/失败此前也被当成 ok:true，
        // 说明「CLI 退出码与结构化输出可信」并未成立。
        match outcome {
            Ok(LifecycleResult::Started) | Ok(LifecycleResult::Stopped) => {
                Ok(serde_json::to_value(IpcMessageData {
                    status: action_to_status(action).to_string(),
                    message: "Lifecycle command delegated to the running manager.".to_string(),
                })
                .unwrap_or(serde_json::Value::Null))
            }
            Ok(_) | Err(_) => Err(IpcErrorCode::ExecutionFailed),
        }
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

/// 同步失败按「原因」映射退出码，而不是一律 `ExecutionFailed`。
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

fn action_log_label(action: LifecycleAction) -> &'static str {
    match action {
        LifecycleAction::Start => "start",
        LifecycleAction::Stop => "stop",
        LifecycleAction::Restart => "restart",
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
            Some(IpcErrorCode::ExecutionFailed)
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
                loop {
                    if writers.freeze().unwrap().is_some() {
                        break;
                    }
                    tokio::task::yield_now().await;
                }
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
    async fn cli_migration_roundtrip_uses_explicit_home_and_preserves_wire_contract() {
        use crate::modules::extensions::{ClientId, ClientTarget, ExtensionConfig, CLIENT_IDS};
        use crate::modules::preferences::{Preferences, PreferencesStore};
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");
        let home = temp.path().join("explicit-home");
        let cwd = temp.path().join("missing-working-directory");
        for root in [&source, &target] {
            crate::modules::data_root::initialize(root).unwrap();
            preferences_fixture(root);
        }
        std::fs::create_dir(&home).unwrap();
        PreferencesStore::new(&source)
            .save(&Preferences {
                interface_scale: 150,
                ..Default::default()
            })
            .unwrap();
        // Force the extension section to validate client destinations. An absent
        // process cwd must not invalidate the valid, explicitly supplied HOME.
        let mut config = ExtensionConfig {
            enablement: CLIENT_IDS
                .map(|client| (client, client == ClientId::Codex))
                .into_iter()
                .collect(),
            sync_method: "copy".into(),
            ..Default::default()
        };
        let targets = CLIENT_IDS.map(|client| ClientTarget::user_target(client, &home, true, true));
        crate::modules::extensions::projection::save_with_config(&source, &mut config, &targets)
            .unwrap();
        let output = temp.path().join("exports/config.ocx");
        let mut exporter = IpcService::for_tests_with(source.clone(), source.clone());
        let exported = exporter
            .execute(migration_request(IpcCommand::Export, &output))
            .await;
        assert!(exported.ok, "{:?}", exported.error);
        let data = exported.data.unwrap();
        assert_eq!(data["path"], output.display().to_string());
        assert!(data["backup_id"].is_null());
        assert_eq!(data["format_version"], 2);
        assert!(data["sections"]
            .as_array()
            .unwrap()
            .contains(&"preferences".into()));
        assert!(data.get("documentSha256").is_none());
        let digest = data["document_sha256"].clone();
        let mut importer = IpcService::for_tests_with(target.clone(), target.clone());
        importer.dependencies.home = home;
        importer.dependencies.process_context.working_directory = cwd.clone();
        let imported = importer
            .execute(migration_request(IpcCommand::Import, &output))
            .await;
        assert!(imported.ok, "{:?}", imported.error);
        let data = imported.data.unwrap();
        assert_eq!(data["document_sha256"], digest);
        assert_eq!(data["format_version"], 2);
        assert!(data["backup_id"].as_str().unwrap().starts_with("bk_"));
        assert!(data["applied_sections"]
            .as_array()
            .unwrap()
            .contains(&"extension_config".into()));
        assert!(data["skipped_sections"]
            .as_array()
            .unwrap()
            .contains(&"asset_files".into()));
        assert_eq!(
            PreferencesStore::new(&target)
                .load()
                .unwrap()
                .interface_scale,
            150
        );
        assert_eq!(
            crate::modules::extensions::projection::load_config_lenient(&target).sync_method,
            "copy"
        );
        assert!(!cwd.exists());
        for (root, command) in [(&source, IpcCommand::Export), (&target, IpcCommand::Import)] {
            let audit = audit_records(root);
            assert_eq!(audit.len(), 1);
            assert_eq!(audit[0].command, command);
            assert_eq!(audit[0].result, super::super::AuditResult::Succeeded);
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cli_migration_lock_wait_keeps_executor_responsive_and_owns_audit_after_disconnect() {
        use crate::infrastructure::locking::TargetFileLock;
        use std::time::{Duration, Instant};
        for command in [IpcCommand::Export, IpcCommand::Import] {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("root");
            crate::modules::data_root::initialize(&root).unwrap();
            preferences_fixture(&root);
            let container = temp.path().join("config.ocx");
            if command == IpcCommand::Import {
                crate::modules::migration::export_with_container_file(&root, &container, "0.1.9")
                    .unwrap();
            }
            let target = if command == IpcCommand::Export {
                container.clone()
            } else {
                root.join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH)
            };
            let guard = TargetFileLock::lock(&target).unwrap();
            let (release, released) = std::sync::mpsc::channel();
            // Bounded watchdog also releases on assertion failure, avoiding an
            // orphaned lock. This tests async liveness, not native performance.
            let holder = std::thread::spawn(move || {
                let _ = released.recv_timeout(Duration::from_secs(2));
                drop(guard);
            });
            let mut service = IpcService::for_tests_with(root.clone(), root.clone());
            service.dependencies.home = temp.path().to_path_buf();
            let writers = service.writers.clone();
            let started = Instant::now();
            let request = migration_request(command, &container);
            let observer = tokio::spawn(async move { service.execute(request).await });
            tokio::time::sleep(Duration::from_millis(20)).await;
            assert!(started.elapsed() < Duration::from_secs(1));
            assert!(!observer.is_finished());
            assert!(writers.freeze().unwrap().is_none());
            observer.abort();
            assert!(observer.await.unwrap_err().is_cancelled());
            assert!(writers.freeze().unwrap().is_none());
            assert!(!root.join("audit.log").exists());
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
            let audit = audit_records(&root);
            assert_eq!(audit.len(), 1);
            assert_eq!(audit[0].command, command);
            assert_eq!(audit[0].result, super::super::AuditResult::Succeeded);
            assert!(container.is_file());
            assert_eq!(
                crate::modules::preferences::PreferencesStore::new(&root)
                    .load()
                    .unwrap(),
                crate::modules::preferences::Preferences::default()
            );
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
                IpcErrorCode::TargetStateConflict
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
            let audit = audit_records(root.path());
            assert_eq!(audit.len(), 1);
            assert_eq!(audit[0].command, command);
            assert_eq!(audit[0].result, super::super::AuditResult::Failed);
            assert!(!std::fs::read_to_string(root.path().join("audit.log"))
                .unwrap()
                .contains("test-only-secret-not-for-audit"));
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
                loop {
                    if writers.freeze().unwrap().is_some() {
                        break;
                    }
                    tokio::task::yield_now().await;
                }
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
        let lock = std::fs::File::create(root.path().join(".backup-w2.lock")).unwrap();
        lock.lock().unwrap();
        let (release, released) = std::sync::mpsc::channel();
        // A watchdog releases the real lock even if a regression blocks this
        // single-thread async executor; this is not a native performance budget.
        let holder = std::thread::spawn(move || {
            let _ = released.recv_timeout(Duration::from_secs(2));
            drop(lock);
        });
        let service = IpcService::for_tests_with(root.path().into(), root.path().into());
        let writers = service.writers.clone();
        let started = Instant::now();
        let observer = tokio::spawn(async move { service.list_backups().await });
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
        assert!(!root.path().join("audit.log").exists());
        assert!(!root.path().join("backups").exists());
    }

    /// 回归：CLI 切换数据根必须把引用写回固定的应用数据目录。
    ///
    /// 此前写的是「当前活动根」，于是第二次切换会落在旧目标里；
    /// 启动只从应用数据目录单跳读取，重启后第二次切换就丢了。
    #[cfg(unix)]
    #[tokio::test(flavor = "multi_thread")]
    async fn switch_data_root_records_reference_in_the_fixed_app_data_root() {
        let temp = tempfile::tempdir().expect("temp");
        let app_root = temp.path().join("app-data");
        let first = temp.path().join("root-a");
        let second = temp.path().join("root-b");
        for path in [&app_root, &first, &second] {
            crate::modules::data_root::initialize(path).expect("init");
        }
        // 模拟「已经切过一次」的运行实例：活动根是 first。
        let mut service = IpcService::for_tests_with(first.clone(), app_root.clone());
        use std::os::unix::fs::PermissionsExt;
        let executable = temp.path().join("stopped-ocx");
        std::fs::write(
            &executable,
            b"#!/bin/sh\necho '{\"status\":\"stopped\",\"dataRoot\":\"/tmp/fixture\"}'\n",
        )
        .unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
        service.collector = Arc::new(Mutex::new(crate::modules::status::StatusCollector::new(
            crate::infrastructure::status_source::OfficialStatusSource::new(
                crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(
                    executable,
                ),
                temp.path(),
                crate::modules::process::EnvironmentPolicy {
                    opencodex_home: temp.path().join("opencodex-home"),
                    ..Default::default()
                },
            ),
        )));
        let mut args = std::collections::BTreeMap::new();
        args.insert("target".to_string(), second.display().to_string());
        let request = IpcRequest {
            request_id: "req_00000000-0000-0000-0000-0000000000fe".to_string(),
            command: IpcCommand::DataRootSwitch,
            args,
            confirm: true,
            contract_version: 1,
            secret: None,
        };
        let response = service.execute(request).await;
        assert!(response.ok, "切换应成功：{:?}", response.error);

        assert!(service.writers.frozen());
        let audit_before = std::fs::read(service.audit.path()).expect("binding audit retained");
        let mut read = IpcRequest {
            request_id: "req_00000000-0000-0000-0000-0000000000fc".into(),
            command: IpcCommand::DataRootShow,
            args: BTreeMap::new(),
            confirm: false,
            contract_version: 1,
            secret: None,
        };
        read.request_id = "req_00000000-0000-0000-0000-0000000000fd".into();
        assert!(
            service.execute(read).await.ok,
            "pending binding remains readable"
        );
        let mut write = IpcRequest {
            request_id: "req_00000000-0000-0000-0000-0000000000fb".into(),
            command: IpcCommand::BackupCreate,
            args: BTreeMap::new(),
            confirm: false,
            contract_version: 1,
            secret: None,
        };
        write.confirm = true;
        assert_eq!(
            service.execute(write).await.error.unwrap().code,
            IpcErrorCode::TargetStateConflict
        );
        assert_eq!(
            std::fs::read(service.audit.path()).unwrap(),
            audit_before,
            "no post-save old-root audit writes"
        );

        let config = crate::modules::data_root::load_runtime_config(&app_root)
            .expect("app data root config");
        assert_eq!(
            config.active_data_root, second,
            "引用必须记录在应用数据目录，供重启时单跳读取"
        );
        assert!(
            crate::modules::data_root::load_runtime_config(&first)
                .map(|config| config.active_data_root != second)
                .unwrap_or(true),
            "不得把引用写进旧的活动根"
        );
    }
    #[tokio::test(flavor = "multi_thread")]
    async fn binding_refuses_busy_writer_and_validation_failure_reopens_admission() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("root");
        let target = temp.path().join("target");
        for path in [&root, &target] {
            crate::modules::data_root::initialize(path).unwrap();
        }
        let mut service = IpcService::for_tests_with(root.clone(), root.clone());
        let request = |target: &std::path::Path| IpcRequest {
            request_id: "req_00000000-0000-0000-0000-0000000000fa".into(),
            command: IpcCommand::DataRootSwitch,
            args: BTreeMap::from([("target".into(), target.display().to_string())]),
            confirm: true,
            contract_version: 1,
            secret: None,
        };
        let active = service.writers.admit().unwrap();
        assert_eq!(
            service.execute(request(&target)).await.error.unwrap().code,
            IpcErrorCode::TargetStateConflict
        );
        assert!(!service.writers.frozen());
        assert_eq!(
            crate::modules::data_root::load_runtime_config(&root)
                .unwrap()
                .active_data_root,
            root
        );
        drop(active);
        assert!(
            !service
                .execute(request(&temp.path().join("missing")))
                .await
                .ok
        );
        assert!(
            service.writers.admit().is_ok(),
            "failed validation reopens writers"
        );
    }

    /// 回归：旧版容器缺口令是「可以让用户补口令」的状态，不是普通执行失败。
    ///
    /// 修复前 `map_app_error` 把任何 AppError 一律压成 ExecutionFailed，
    /// CLI/GUI 都看不到「需要原口令」，只能显示一句笼统的失败。
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
}
