//! MOD-12 IPC 委托执行与命令验证。
//!
//! CLI 是运行中 GUI 的客户端：只读取共享快照，生命周期动作复用共享
//! runner；写路径在本阶段保留为无副作用的能力占位。

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
        }
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
                IpcCommand::BackupList => self.list_backups(),
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
                IpcCommand::BackupCreate => self.create_backup(),
                IpcCommand::Export => self.export_config(&request.args),
                IpcCommand::Import => self.import_config(&request.args, request.secret.as_deref()),
                IpcCommand::SyncRun => self.sync_run().await,
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

    fn list_backups(&self) -> Result<serde_json::Value, IpcErrorCode> {
        let records = crate::modules::backup::list_action_records(
            &self.dependencies.active_data_root.join("backups"),
            crate::modules::backup::BackupAction::Upgrade.as_str(),
        )
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

    fn create_backup(&self) -> Result<serde_json::Value, IpcErrorCode> {
        let target = self
            .dependencies
            .active_data_root
            .join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH);
        let payload = std::fs::read(&target).map_err(|_| IpcErrorCode::TargetNotFound)?;
        let record = crate::modules::backup::backup_file(
            &self.dependencies.active_data_root,
            crate::modules::backup::BackupAction::Upgrade,
            &target,
            &payload,
            chrono::Utc::now(),
            Some("cli-requested".to_string()),
        )
        .map_err(map_app_error)?;
        serde_json::to_value(IpcMessageData {
            status: "backed_up".to_string(),
            message: record.manifest.backup_id,
        })
        .map_err(|_| IpcErrorCode::InternalError)
    }

    fn export_config(
        &self,
        args: &BTreeMap<String, String>,
    ) -> Result<serde_json::Value, IpcErrorCode> {
        if args.contains_key("password") {
            return Err(IpcErrorCode::ValidationFailed);
        }
        let output = args.get("output").ok_or(IpcErrorCode::ValidationFailed)?;
        // 新版明文容器不需要口令；不再读取标准输入内容。
        let result = crate::modules::migration::export_with_container_file(
            &self.dependencies.active_data_root,
            std::path::Path::new(output),
            self.dependencies.current_version,
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
        &self,
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
        let home = self.dependencies.process_context.working_directory.clone();
        let result = crate::modules::migration::import_with_container_file(
            &self.dependencies.active_data_root,
            std::path::Path::new(input),
            &home,
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

    /// CLI 的同步与 GUI 走同一段真实执行体（`modules::sync::runner`）。
    ///
    /// 这里此前只读同步状态文件、回一个伪造的 `snapshot_id` 与 `artifacts: 0` 就返回成功，
    /// 从不接触网络——`ocxd sync run` 因此是「假成功」。现在改为真正执行同步，并把
    /// 端点缺失、冷同步、互斥占用、认证失败、远端不存在等如实映射为不同退出码。
    async fn sync_run(&self) -> Result<serde_json::Value, IpcErrorCode> {
        let data_root = self.dependencies.active_data_root.clone();
        let config = crate::modules::sync::config::SyncConfigStore::new(&data_root)
            .load()
            .map_err(|_| IpcErrorCode::ExecutionFailed)?;
        let endpoint = config
            .active()
            .cloned()
            .ok_or(IpcErrorCode::ExecutionFailed)?;
        let webdav =
            crate::modules::sync::runner::webdav_config(&endpoint).map_err(map_sync_run_failure)?;
        let outcome = crate::modules::sync::runner::run_sync(
            &data_root,
            &self.dependencies.home,
            &endpoint,
            &webdav,
        )
        .await
        .map_err(map_sync_run_failure)?;
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
        let mut service = IpcService::for_tests();
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
