//! 运行来源、托管安装与卸载命令（IMP Track B · B3 / B4 / B5）。
//!
//! 命令层只做三件事：解析输入、编排模块、把进度与终态转发给界面。
//! **安装进度不落盘**，只有终态才写 `runtime.json` 历史（`FZ-48`）。

use std::path::{Path, PathBuf};

use tauri::Manager;

use crate::errors::{AppError, AppResult};
use crate::modules::runtime::archive::{self, ArchiveRejection};
use crate::modules::runtime::install::{
    default_version, CancelFlag, InstallProgress, InstallProgressSink, InstallRequest,
    InstallSource, RuntimeInstaller,
};
use crate::modules::runtime::uninstall::{self, UninstallScope};
use crate::modules::runtime::{now_rfc3339, InstallHistoryEntry, RuntimeHandle, RuntimeSourceKind};
use crate::state::{
    SharedDataRoot, SharedHomeDir, SharedProcessContext, SharedProcessRunner, SharedRuntimeInstall,
    SharedStatusCollector,
};
use crate::types::runtime::{
    OfflinePackagePreviewDto, RuntimeInstallOutcomeDto, RuntimeInstallRequestDto, RuntimeSourceDto,
    RuntimeUninstallPlanDto, RuntimeUninstallRequestDto, RuntimeUninstallResidueDto,
    RuntimeUninstallResultDto, RuntimeUninstallStepDto,
};

/// 安装进度事件名；载荷形如 `{phase, percent, line?}`。
pub const RUNTIME_INSTALL_PROGRESS_EVENT: &str = "runtime-install-progress";
/// 运行来源变化事件名（安装 / 卸载 / 显式指定后广播，前端据此刷新）。
pub const RUNTIME_SOURCE_CHANGED_EVENT: &str = "runtime-source-changed";

/// One local startup retry on an admitted background worker. A malformed receipt
/// is retained; no package execution, networking, or polling is introduced.
pub(crate) fn reconcile_startup(root: PathBuf, install: SharedRuntimeInstall) {
    let Ok(storage) = crate::infrastructure::storage_writers::global().admit() else {
        return;
    };
    let Some(lease) = install.acquire() else {
        return;
    };
    tauri::async_runtime::spawn(async move {
        let result = crate::commands::run_blocking("reconcile panel protection", move || {
            let _storage = storage;
            let _lease = lease;
            if crate::modules::runtime::protection::PanelProtection::load(&root)?.is_none() {
                return Ok(());
            }
            let guard = crate::modules::backup::manager::acquire_preferences_transaction(&root)?;
            crate::modules::runtime::protection::reconcile_pending(&root, &guard)
        })
        .await;
        if result.is_err() {
            eprintln!("panel protection startup reconciliation remains pending");
        }
    });
}

/// 已发现的 node / npm 绝对路径（**不读 PATH**，`FZ-13` / `FZ-50`）。
fn discovered_executables() -> (Option<PathBuf>, Option<PathBuf>) {
    let Some(paths) = crate::types::discovery_paths::default_paths() else {
        return (None, None);
    };
    let validated = |path: &Path| {
        let path = std::fs::canonicalize(path).ok()?;
        crate::modules::runtime::paths::validate_executable(&path).ok()
    };
    (validated(&paths.node), validated(&paths.npm))
}

fn fixed_executables(root: &Path) -> AppResult<(PathBuf, PathBuf)> {
    let (node, npm) = crate::modules::runtime::install::managed_dependency_paths(root);
    for path in [&node, &npm] {
        crate::modules::backup::safety::check_path(root, path, false).map_err(|_| {
            AppError::RuntimeManaged {
                code: "managed_dependency_missing".into(),
                detail: format!("集成运行依赖缺失或路径不可信：{}", path.display()),
            }
        })?;
    }
    crate::modules::runtime::paths::validate_executable(&node).map_err(|_| {
        AppError::RuntimeManaged {
            code: "node_missing".into(),
            detail: "当前 Desktop 集成的 Node 缺失或不可执行".into(),
        }
    })?;
    crate::modules::backup::safety::read_file(&npm, 1024 * 1024)?;
    Ok((node, npm))
}

/// v2 never discovers dependencies. Legacy handles retain their test behavior.
pub fn managed_executables(
    handle: &RuntimeHandle,
) -> AppResult<(Option<PathBuf>, Option<PathBuf>)> {
    if handle.uses_versioned_programs() {
        let selected = handle
            .selected_program()?
            .ok_or_else(|| AppError::RuntimeManaged {
                code: "managed_dependency_missing".into(),
                detail: "集成运行程序尚未准备完成".into(),
            })?;
        return Ok((Some(selected.paths.node), Some(selected.paths.npm)));
    }
    if !handle.resolver_managed_only() {
        return Ok(discovered_executables());
    }
    let (node, npm) = fixed_executables(handle.installation_root())?;
    Ok((Some(node), Some(npm)))
}

/// 把安装进度转发成 Tauri 事件；凭据已在内核里掩码。
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelUpdateProgress {
    operation_id: String,
    candidate_version: String,
    sequence: u64,
    phase: crate::modules::runtime::install::InstallPhase,
    line: Option<String>,
}
struct EventProgressSink {
    app: tauri::AppHandle,
    channel: Option<tauri::ipc::Channel<PanelUpdateProgress>>,
    operation_id: String,
    candidate_version: String,
    sequence: std::sync::atomic::AtomicU64,
}

impl InstallProgressSink for EventProgressSink {
    fn emit(&self, progress: InstallProgress) {
        if crate::modules::notifications::registry::validate_signal(
            RUNTIME_INSTALL_PROGRESS_EVENT,
            crate::modules::notifications::registry::Job::Install,
            crate::modules::notifications::registry::Trigger::NativeCallback,
            crate::modules::notifications::registry::Channel::Local,
        )
        .is_err()
        {
            return;
        }
        if let Some(channel) = &self.channel {
            // npm stages have no trustworthy byte total; don't forward synthetic percentages.
            let _ = channel.send(PanelUpdateProgress {
                operation_id: self.operation_id.clone(),
                candidate_version: self.candidate_version.clone(),
                sequence: self
                    .sequence
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                    + 1,
                phase: progress.phase,
                line: progress.line,
            });
        } else {
            let _ = crate::commands::event_delivery::emit_signal(
                &self.app,
                RUNTIME_INSTALL_PROGRESS_EVENT,
                crate::modules::notifications::registry::Job::Install,
                crate::modules::notifications::registry::Trigger::NativeCallback,
                crate::modules::notifications::registry::Channel::Local,
                &progress,
            );
        }
    }
}

pub(crate) fn broadcast_source_changed(app: &tauri::AppHandle) {
    let _ = crate::commands::event_delivery::emit_signal(
        app,
        RUNTIME_SOURCE_CHANGED_EVENT,
        crate::modules::notifications::registry::Job::Source,
        crate::modules::notifications::registry::Trigger::Commit,
        crate::modules::notifications::registry::Channel::Local,
        (),
    );
}

fn source_dto(handle: &RuntimeHandle, data_root: &Path) -> AppResult<RuntimeSourceDto> {
    let resolution = handle.current();
    let record = handle.store().load();
    let selected = handle.selected_program()?;
    // 卸载与「重装同版本」按**安装记录登记的**前缀定位（`FZ-51`）：
    // `FZ-47` 允许自定义前缀，那时前缀不再是数据根的默认位置。
    let managed_prefix = if handle.uses_versioned_programs() {
        selected
            .as_ref()
            .map(|p| p.paths.package_root.clone())
            .unwrap_or_else(|| handle.installation_root().join("runtime/versions"))
    } else if handle.resolver_managed_only() {
        handle
            .installation_root()
            .join(crate::modules::runtime::MANAGED_PREFIX_RELATIVE)
    } else {
        record.installed_prefix(data_root)
    };
    let managed_entry = handle.managed_entry().to_path_buf();
    Ok(RuntimeSourceDto {
        kind: resolution.kind,
        inside_data_root: resolution
            .path
            .as_ref()
            .map(|path| path.starts_with(data_root))
            .unwrap_or(false),
        path: resolution
            .path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned()),
        version: if handle.uses_versioned_programs() {
            selected.as_ref().map(|p| p.selection.version.clone())
        } else {
            record.resolved_version.clone()
        },
        resolved_at: record.resolved_at.clone(),
        managed_entry: matches!(resolution.kind, RuntimeSourceKind::Managed)
            .then(|| managed_entry.to_string_lossy().into_owned()),
        managed_prefix: managed_prefix.to_string_lossy().into_owned(),
        default_prefix: managed_prefix.to_string_lossy().into_owned(),
        explicit_path: handle
            .explicit_path()
            .map(|path| path.to_string_lossy().into_owned()),
        history: record.history,
    })
}

/// 代理进程是否在运行；运行中的来源切换要提示「需重启」（`FZ-48`）。
fn proxy_is_running(collector: &SharedStatusCollector) -> bool {
    use crate::types::status::RuntimeState;
    collector
        .lock()
        .map(|collector| {
            matches!(
                collector.matrix().runtime,
                RuntimeState::Running | RuntimeState::Starting | RuntimeState::Stopping
            )
        })
        .unwrap_or(false)
}

#[tauri::command]
pub async fn runtime_source(
    handle: tauri::State<'_, std::sync::Arc<RuntimeHandle>>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> AppResult<RuntimeSourceDto> {
    let handle = handle.inner().clone();
    let root = data_root.0.clone();
    crate::commands::run_blocking("read actual runtime source", move || {
        handle.observe_current();
        source_dto(&handle, &root)
    })
    .await
}

#[tauri::command]
pub async fn runtime_protection_status(
    data_root: tauri::State<'_, SharedDataRoot>,
) -> AppResult<crate::modules::runtime::protection::ProtectionStatus> {
    let root = data_root.0.clone();
    crate::commands::run_readonly("read panel protection", move || {
        Ok(crate::modules::runtime::protection::status(&root))
    })
    .await
}

/// Explicit local verification only. Worker ownership keeps all leases alive when
/// the observer closes; there is no stop, reinstall, receipt deletion override or timer.
fn reserve_protection_retry(
    runtime: &SharedRuntimeInstall,
    manager: &crate::commands::update::SharedUpdateStatus,
) -> AppResult<crate::state::RuntimeMutationLease> {
    let state = manager.lock().map_err(|_| AppError::NotConfigured)?;
    if state.installing || state.pending_restart.is_some() {
        return Err(AppError::NotConfigured);
    }
    runtime.acquire().ok_or(AppError::NotConfigured)
}

#[tauri::command]
pub async fn retry_runtime_protection(
    app: tauri::AppHandle,
    data_root: tauri::State<'_, SharedDataRoot>,
    runtime: tauri::State<'_, SharedRuntimeInstall>,
    manager: tauri::State<'_, crate::commands::update::SharedUpdateStatus>,
) -> AppResult<crate::modules::runtime::protection::ProtectionStatus> {
    let root = data_root.0.clone();
    let runtime = runtime.inner().clone();
    let manager = manager.inner().clone();
    crate::commands::run_blocking("retry panel protection", move || {
        let _reservation = reserve_protection_retry(&runtime, &manager)?;
        crate::modules::runtime::protection::retry(&root, |candidate, succeeded| {
            let identity = crate::commands::event_delivery::prepare(
                &root,
                "runtime-protection-reconcile-failed",
                crate::modules::notifications::registry::Channel::Local,
                candidate,
            );
            crate::commands::event_delivery::publish(
                &app,
                &root,
                if succeeded {
                    "runtime-protection-reconcile-succeeded"
                } else {
                    "runtime-protection-reconcile-failed"
                },
                identity,
                crate::modules::notifications::registry::Trigger::User,
            );
        })
    })
    .await
}

/// Source mutation shares the installation reservation; dismissal never releases a worker.
#[tauri::command]
pub async fn set_runtime_source(
    app: tauri::AppHandle,
    path: Option<String>,
    handle: tauri::State<'_, std::sync::Arc<RuntimeHandle>>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> AppResult<RuntimeSourceDto> {
    change_runtime_source(app, path, handle.inner().clone(), data_root.0.clone()).await
}

#[tauri::command]
pub async fn restore_discovered_runtime(
    app: tauri::AppHandle,
    handle: tauri::State<'_, std::sync::Arc<RuntimeHandle>>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> AppResult<RuntimeSourceDto> {
    change_runtime_source(app, None, handle.inner().clone(), data_root.0.clone()).await
}

async fn change_runtime_source(
    app: tauri::AppHandle,
    path: Option<String>,
    handle: std::sync::Arc<RuntimeHandle>,
    data_root: PathBuf,
) -> AppResult<RuntimeSourceDto> {
    let runtime = app.state::<SharedRuntimeInstall>().inner().clone();
    let manager = app
        .state::<crate::commands::update::SharedUpdateStatus>()
        .inner()
        .clone();
    crate::commands::run_blocking("change runtime source", move || {
        let _reservation = {
            let state = manager.lock().map_err(|_| AppError::NotConfigured)?;
            if state.installing || state.pending_restart.is_some() {
                return Err(AppError::NotConfigured);
            }
            runtime.acquire().ok_or(AppError::NotConfigured)?
        };
        crate::modules::runtime::protection::ensure_binding_ready(&data_root)?;
        let candidate = path
            .filter(|value| !value.trim().is_empty())
            .map(PathBuf::from);
        if let Some(candidate) = candidate.as_ref() {
            crate::modules::runtime::RuntimeResolver::validate_explicit(candidate).map_err(
                |rejection| AppError::RuntimeManaged {
                    code: rejection.code().to_string(),
                    detail: format!("指定的运行来源不可用：{}", rejection.code()),
                },
            )?;
        }
        handle.set_explicit(candidate);
        broadcast_source_changed(&app);
        source_dto(&handle, &data_root)
    })
    .await
}

/// 离线包预检：只校验不展开，让拖拽区能立刻给出「已选 / 校验失败 / 拒绝」。
#[tauri::command]
pub async fn preview_offline_package(path: String) -> AppResult<OfflinePackagePreviewDto> {
    crate::commands::run_readonly("preview offline package", move || {
        offline_package_preview(&path)
    })
    .await
}
fn offline_package_preview(path: &str) -> AppResult<OfflinePackagePreviewDto> {
    let archive_path = PathBuf::from(path);
    let file_name = archive_path
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| archive_path.display().to_string());
    let file_size = std::fs::metadata(&archive_path)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    match archive::inspect(&archive_path, None) {
        Ok(inspection) => Ok(OfflinePackagePreviewDto {
            ok: true,
            file_name,
            file_size,
            version: Some(inspection.version),
            sha256_prefix: Some(inspection.sha256[..16].to_string()),
            rejection_code: None,
            message: "离线包校验通过".to_string(),
        }),
        Err(rejection) => Ok(OfflinePackagePreviewDto {
            ok: false,
            file_name,
            file_size,
            version: None,
            sha256_prefix: None,
            rejection_code: Some(rejection.code().to_string()),
            message: describe_rejection(&rejection),
        }),
    }
}

#[tauri::command]
pub async fn stage_offline_package(
    request: tauri::ipc::Request<'_>,
    root: tauri::State<'_, SharedDataRoot>,
    upload: tauri::State<'_, crate::modules::runtime::offline_drop::SharedOfflineDrop>,
    runtime: tauri::State<'_, SharedRuntimeInstall>,
) -> AppResult<serde_json::Value> {
    #[derive(serde::Deserialize)]
    #[serde(tag = "action", rename_all = "snake_case")]
    enum Action {
        Begin { name: String, length: u64 },
        Finish { id: String },
        Clear { id: String },
    }
    enum Input {
        Action(Action),
        Chunk {
            id: String,
            offset: u64,
            bytes: Vec<u8>,
        },
    }
    let invalid = || AppError::FileSystem {
        operation: "receive offline package".into(),
        detail: "离线包读取请求无效。".into(),
    };
    let chunk = |bytes: Vec<u8>| -> AppResult<Input> {
        if bytes.is_empty() || bytes.len() > crate::modules::runtime::offline_drop::CHUNK_BYTES {
            return Err(invalid());
        }
        let id = request
            .headers()
            .get("x-ocx-drop-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(invalid)?
            .to_owned();
        let offset = request
            .headers()
            .get("x-ocx-drop-offset")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .ok_or_else(invalid)?;
        Ok(Input::Chunk { id, offset, bytes })
    };
    let input = match request.body() {
        tauri::ipc::InvokeBody::Json(value) if value.is_array() => chunk(
            crate::modules::runtime::offline_drop::fallback_bytes(value)?,
        )?,
        tauri::ipc::InvokeBody::Json(value) => {
            Input::Action(serde_json::from_value(value.clone()).map_err(|_| invalid())?)
        }
        tauri::ipc::InvokeBody::Raw(bytes) => chunk(bytes.clone())?,
    };
    let root = root.0.clone();
    let upload = upload.inner().clone();
    let runtime = runtime.inner().clone();
    crate::commands::run_blocking("receive offline package", move || {
        let _storage = crate::infrastructure::storage_writers::global().admit()?;
        let _lease = runtime.acquire().ok_or(AppError::NotConfigured)?;
        let mut buffer = upload.0.lock().map_err(|_| AppError::NotConfigured)?;
        match input {
            Input::Action(Action::Begin { name, length }) => Ok(serde_json::json!({"id": buffer.begin(&root, &name, length)?})),
            Input::Chunk { id, offset, bytes } => Ok(serde_json::json!({"received": buffer.append(&id, offset, &bytes)?})),
            Input::Action(Action::Finish { id }) => {
                let path = buffer.finish(&id)?;
                let preview = offline_package_preview(&path.to_string_lossy())?;
                if !preview.ok { buffer.clear(&id); }
                Ok(serde_json::json!({"path": preview.ok.then(|| path.to_string_lossy().into_owned()), "preview": preview}))
            }
            Input::Action(Action::Clear { id }) => { buffer.clear(&id); Ok(serde_json::json!({})) }
        }
    }).await
}

fn describe_rejection(rejection: &ArchiveRejection) -> String {
    match rejection {
        ArchiveRejection::NotGzip => "不是 gzip 归档；请选择官方 .tgz 离线包".to_string(),
        ArchiveRejection::BadFilename => {
            "文件名必须匹配 opencodex-<版本>.tgz（或 bitkyc08-opencodex-<版本>.tgz）".to_string()
        }
        ArchiveRejection::NotOfficialPackage { found } => {
            format!("包名必须是 @bitkyc08/opencodex（实际 {found}）")
        }
        ArchiveRejection::VersionMismatch { declared, found } => {
            format!("文件名声明的版本 {declared} 与包内 {found} 不一致")
        }
        ArchiveRejection::MissingPackageJson => "包内缺少 package.json".to_string(),
        ArchiveRejection::BadPackageJson => "包内 package.json 无法解析或缺少必要字段".to_string(),
        ArchiveRejection::TooManyEntries => "归档条目数超过上限".to_string(),
        ArchiveRejection::TooLarge => "解压总量超过上限".to_string(),
        ArchiveRejection::FileTooLarge => "单个文件超过上限".to_string(),
        ArchiveRejection::DepthExceeded => "归档目录嵌套深度超过上限".to_string(),
        ArchiveRejection::PathEscape { entry } => format!("归档条目路径越界：{entry}"),
        ArchiveRejection::LinkEntry { entry } => format!("归档含符号链接 / 硬链接条目：{entry}"),
        ArchiveRejection::SpecialEntry { entry } => {
            format!("归档含设备 / FIFO 等特殊条目：{entry}")
        }
        ArchiveRejection::Read => "读取归档失败".to_string(),
    }
}

/// 执行一次托管安装；进度就地推送，终态才写历史。
#[tauri::command]
pub async fn install_runtime(
    request: RuntimeInstallRequestDto,
    app: tauri::AppHandle,
    handle: tauri::State<'_, std::sync::Arc<RuntimeHandle>>,
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, SharedHomeDir>,
    collector: tauri::State<'_, SharedStatusCollector>,
    install: tauri::State<'_, SharedRuntimeInstall>,
) -> AppResult<RuntimeInstallOutcomeDto> {
    install_managed_runtime(
        request,
        app,
        RuntimeInstallContext {
            handle: handle.inner().clone(),
            data_root_path: data_root.0.clone(),
            home_path: home.0.clone(),
            collector: collector.inner().clone(),
        },
        install.inner(),
        false,
        None,
    )
    .await
}

struct RuntimeInstallContext {
    handle: std::sync::Arc<RuntimeHandle>,
    data_root_path: PathBuf,
    home_path: PathBuf,
    collector: SharedStatusCollector,
}

/// 受控安装核心（供 install_runtime 与代跑官方更新共用）；不直接接触 Tauri State。
async fn install_managed_runtime(
    request: RuntimeInstallRequestDto,
    app: tauri::AppHandle,
    context: RuntimeInstallContext,
    install: &SharedRuntimeInstall,
    protect_update: bool,
    on_progress: Option<tauri::ipc::Channel<PanelUpdateProgress>>,
) -> AppResult<RuntimeInstallOutcomeDto> {
    let _storage = crate::infrastructure::storage_writers::global().admit()?;
    let RuntimeInstallContext {
        handle,
        data_root_path,
        home_path,
        collector,
    } = context;
    let source_kind = request.source_kind().ok_or(AppError::RuntimeManaged {
        code: "bad_source".to_string(),
        detail: "安装源必须是 registry 或 offline".to_string(),
    })?;
    let prefix = if handle.uses_versioned_programs() {
        handle.observe_current();
        let selected = handle
            .selected_program()?
            .ok_or_else(|| AppError::RuntimeManaged {
                code: "managed_dependency_missing".into(),
                detail: "集成运行程序尚未准备完成，请先恢复内置资源".into(),
            })?;
        if request
            .prefix_path()
            .is_some_and(|p| p != selected.paths.package_root)
        {
            return Err(AppError::RuntimeManaged {
                code: "invalid_install_root".into(),
                detail: "运行包位置由当前版本选择确定，请刷新安装配置".into(),
            });
        }
        selected.paths.package_root
    } else if handle.resolver_managed_only() {
        let expected = handle.installation_root().join("runtime/opencodex");
        if request.prefix_path().is_some_and(|p| p != expected) {
            return Err(AppError::RuntimeManaged {
                code: "invalid_install_root".into(),
                detail: "运行程序必须安装在当前 Desktop 的 runtime/opencodex 目录".into(),
            });
        }
        expected
    } else {
        request
            .prefix_path()
            .unwrap_or_else(|| handle.store().load().installed_prefix(&data_root_path))
    };
    let requested_version = request
        .version
        .clone()
        .unwrap_or_else(|| default_version().to_string());
    // 网络代理（U-05）：本次安装未显式指定代理时，继承用户网络偏好里的手动 HTTP 代理。
    let proxy = request.proxy().or_else(
        || match crate::modules::preferences::proxy_policy_for_app(&app) {
            crate::modules::preferences::ProxyPolicy::Manual(url) => {
                let (scheme, host) = url.split_once("://")?;
                let scheme = crate::modules::runtime::install::ProxyScheme::parse(scheme)?;
                Some(crate::modules::runtime::install::ProxyConfig::new(
                    scheme, host,
                ))
            }
            _ => None,
        },
    );
    let source = match source_kind {
        crate::modules::runtime::install::InstallSourceKind::Registry => InstallSource::Registry {
            version: requested_version.clone(),
            proxy,
        },
        crate::modules::runtime::install::InstallSourceKind::Offline => InstallSource::Offline {
            archive: request.offline_archive().ok_or(AppError::RuntimeManaged {
                code: "offline_path_missing".to_string(),
                detail: "离线安装需要选择一个 .tgz 文件".to_string(),
            })?,
        },
    };
    let (node_path, npm_path) = managed_executables(&handle)?;
    let install_request = InstallRequest {
        prefix: prefix.clone(),
        source,
        allow_scripts: request.allow_scripts,
        npm_path,
        node_path: node_path.clone(),
    };

    let mutation = {
        let manager = app.state::<crate::commands::update::SharedUpdateStatus>();
        let guard = manager.lock().map_err(|_| AppError::NotConfigured)?;
        if guard.installing || guard.pending_restart.is_some() {
            return Err(AppError::NotConfigured);
        }
        install.acquire().ok_or(AppError::RuntimeManaged {
            code: "install_in_progress".to_string(),
            detail: "已有安装正在进行".to_string(),
        })?
    };
    let proxy_running = proxy_is_running(&collector);
    let sink = EventProgressSink {
        app: app.clone(),
        channel: on_progress,
        operation_id: format!("panel-{}", uuid::Uuid::new_v4()),
        candidate_version: requested_version.clone(),
        sequence: std::sync::atomic::AtomicU64::new(0),
    };
    let handle_ref = handle.clone();

    // Select only candidate fields; proxy credentials never enter event state/logs.
    let mut candidate = serde_json::json!({
        "source": request.source,
        "version": requested_version,
        "prefix": prefix,
        "offline": request.offline_path,
        "scripts": request.allow_scripts,
    });
    let worker_app = app.clone();
    let result = crate::commands::run_blocking("install managed runtime", move || {
        let mutation = mutation;
        if let InstallSource::Offline { archive } = &install_request.source {
            if let Ok(inspection) = archive::inspect(archive, None) {
                candidate["offline_sha256"] = serde_json::json!(inspection.sha256);
            }
        }
        let identity = crate::commands::event_delivery::prepare(
            &data_root_path,
            "runtime-install-failed",
            crate::modules::notifications::registry::Channel::Official,
            candidate.to_string().as_bytes(),
        );
        let result: AppResult<_> = (|| {
            if handle_ref.uses_versioned_programs() {
                crate::modules::runtime::protection::ensure_binding_ready(&data_root_path)?;
                let (target,platform)=crate::modules::runtime::program_layout::host_target()?;
                let selected=crate::modules::runtime::program_selection::read(handle_ref.installation_root(),platform,target)?.ok_or(AppError::NotConfigured)?;
                let npm=crate::modules::runtime::install::SystemNpmRunner::new(Some(home_path.clone()),data_root_path.join("cache/npm"));
                let probe=crate::modules::runtime::install::RealVersionProbe::default();
                let mut installer=RuntimeInstaller::new(&data_root_path,&npm,Some(home_path.clone()),&sink,&probe,node_path.clone());
                installer.cancel=mutation.cancel.clone();
                installer.config_backup=Some(crate::modules::model_config::config_file::v2::native_import::backups::UpgradeRequest {
                    home:worker_app.state::<SharedProcessContext>().opencodex_home.clone(),
                    program_source:Some(crate::commands::native_config_import::program_source(&selected)),
                });
                use crate::commands::runtime_recovery_v2::{ServiceOwnerProof,ServiceOwnerProofProvider};
                let context=worker_app.state::<SharedProcessContext>().inner().clone();
                let owner=crate::commands::runtime_recovery_v2_owner::SystemServiceOwnerProofProvider::new(handle_ref.installation_root().into(),context.clone())?;
                let outcome=installer.install_versioned(handle_ref.installation_root(),&selected,&install_request,||{
                    if owner.query(&context)?==ServiceOwnerProof::Stopped {Ok(())} else {Err(AppError::NotConfigured)}
                })?;
                handle_ref.observe_current();
                return Ok((outcome.installation,false));
            }
            // Reconcile before creating another protection backup or replacing the prefix.
            {
                let guard = crate::modules::backup::manager::acquire_preferences_transaction(&data_root_path)?;
                crate::modules::runtime::protection::reconcile_pending(&data_root_path, &guard)?;
            }
            let protection = if protect_update {
                crate::modules::backup::manager::begin_preferences_protection_observed(
                    &data_root_path,
                    true,
                    &mut crate::commands::event_delivery::BackupEvents {
                        app: &worker_app,
                        root: &data_root_path,
                        trigger: crate::modules::notifications::registry::Trigger::User,
                    },
                )?
            } else {
                crate::modules::backup::manager::acquire_preferences_transaction(&data_root_path)?
            };
            let mut receipt = crate::modules::runtime::protection::PanelProtection::arm(
                &data_root_path, &protection, &install_request.prefix,
                &requested_version, node_path.as_deref(),
            )?;
            let npm = crate::modules::runtime::install::SystemNpmRunner::new(
                Some(home_path.clone()),
                data_root_path.join("cache/npm"),
            );
            let probe = crate::modules::runtime::install::RealVersionProbe::default();
            let mut installer = RuntimeInstaller::new(
                &data_root_path,
                &npm,
                Some(home_path),
                &sink,
                &probe,
                node_path.clone(),
            );
            installer.installation_root = handle_ref.resolver_managed_only()
                .then(|| handle_ref.installation_root().to_path_buf());
            installer.cancel = mutation.cancel.clone();
            installer.attempt_id = receipt.as_ref().map(|r| r.attempt_id());
            {
                // Reinstalling through the installation dialog must not bypass
                // the same current-HOME protection used by official updates.
                let home = worker_app.state::<SharedProcessContext>().opencodex_home.clone();
                let program_source = handle_ref.selected_program().ok().flatten().as_ref()
                    .map(crate::commands::native_config_import::program_source);
                installer.config_backup = Some(crate::modules::model_config::config_file::v2::native_import::backups::UpgradeRequest { home, program_source });
            }
            match installer.install(&install_request) {
                Ok(outcome) => {
                    handle_ref.refresh();
                    // 安装刚落地，版本是确定事实：回填到运行来源记录，卡片与
                    // `runtime.json` 的 `resolved_version` 才不会停在「未知」（`FZ-48`）。
                    // Package, launcher and source history were committed by the installer.
                    // Do not duplicate success history or acknowledge an ignored write failure.
                    let protection_pending = receipt.as_mut().is_some_and(|r|
                        r.reconcile(&data_root_path, &protection).is_err());
                    if protection_pending {
                        eprintln!("managed runtime installed, but preference protection reconciliation is pending");
                    }
                    Ok((outcome, protection_pending))
                }
                Err(error) => {
                    if let Some(receipt) = &receipt {
                        // Forget only this failed attempt; its backup remains protected.
                        if receipt.discard(&data_root_path).is_err() {
                            eprintln!("failed panel attempt receipt retained for reconciliation");
                        }
                    }
                    // 终态失败：不改运行来源，但留一条可核对的历史（`FZ-48`）。
                    let _ = handle_ref.record_history(InstallHistoryEntry {
                        action: "install".to_string(),
                        result: "failed".to_string(),
                        package: crate::modules::runtime::OFFICIAL_PACKAGE.to_string(),
                        version: requested_version.clone(),
                        target: prefix.to_string_lossy().into_owned(),
                        at: now_rfc3339(),
                        reason: Some(error.message()),
                    });
                    Err(error.into())
                }
            }
        })();
        crate::commands::event_delivery::publish(
            &worker_app,
            &data_root_path,
            if result.is_ok() {
                "runtime-install-succeeded"
            } else {
                "runtime-install-failed"
            },
            identity,
            crate::modules::notifications::registry::Trigger::User,
        );
        if result.is_ok() {
            broadcast_source_changed(&worker_app);
        }
        result
    })
    .await;
    let (outcome, protection_reconciliation_pending) = result?;

    let needs_restart = outcome.restart_required(proxy_running);
    Ok(RuntimeInstallOutcomeDto {
        package: outcome.package,
        version: outcome.version,
        target: outcome.target.to_string_lossy().into_owned(),
        entry: outcome.entry.to_string_lossy().into_owned(),
        tarball_sha256: outcome.tarball_sha256,
        source: outcome.source,
        scripts_enabled: outcome.scripts_enabled,
        npm_path: outcome.npm_path,
        installed_at: outcome.installed_at,
        proxy_used: outcome.proxy_used,
        needs_restart,
        protection_reconciliation_pending,
    })
}

/// 请求取消当前安装（内核会终止 npm 子进程并清理临时前缀）。
#[tauri::command]
pub fn cancel_runtime_install(install: tauri::State<'_, SharedRuntimeInstall>) -> AppResult<bool> {
    Ok(install.cancel())
}

/// 代跑官方更新（U-04）：先只读解析远端确定版本，再复用受控安装把该版本装到当前登记前缀。
///
/// 不浮动 `latest`：查询与安装绑定同一确定版本与来源策略；不写全局 npm 前缀，不调用官方更新器。
/// 安装属写操作，前端需显式确认；本命令沿用 `install_runtime` 的离线包 / 代理 / 清单 / 重启提示路径。
// Tauri injects application states as distinct command arguments.
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn install_official_update(
    candidate_version: String,
    on_progress: tauri::ipc::Channel<PanelUpdateProgress>,
    app: tauri::AppHandle,
    handle: tauri::State<'_, std::sync::Arc<RuntimeHandle>>,
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, SharedHomeDir>,
    collector: tauri::State<'_, SharedStatusCollector>,
    install: tauri::State<'_, SharedRuntimeInstall>,
) -> AppResult<RuntimeInstallOutcomeDto> {
    // 1. 只读远端查询：与官方版本卡片（U-03）共用同一来源与代理策略，解析出确定版本。
    let query_handle = handle.inner().clone();
    let environment =
        crate::modules::preferences::network_environment_for_app(&app, home.0.clone());
    let working = home.0.clone();
    let cache = data_root.0.join("cache/npm");
    let logs = data_root.0.join("logs/runtime-query");
    // npm view may write its cache; the blocking worker must own admission.
    let remote = super::run_blocking("official update metadata", move || {
        query_runtime_remote_latest(
            &query_handle,
            &working,
            &cache,
            &logs,
            &environment,
            "latest",
        )
    })
    .await?;

    if remote.version != candidate_version {
        return Err(AppError::NotConfigured);
    }
    // 2. 复用受控安装，把查询到的确定版本装到当前登记前缀（`prefix: None` 取安装记录登记值）。
    let request = official_update_request(remote.version);
    install_managed_runtime(
        request,
        app,
        RuntimeInstallContext {
            handle: handle.inner().clone(),
            data_root_path: data_root.0.clone(),
            home_path: home.0.clone(),
            collector: collector.inner().clone(),
        },
        install.inner(),
        true,
        Some(on_progress),
    )
    .await
}

/// 组装代跑官方更新的受控安装请求（U-04/U-04b）：绑定查询到的**确定版本**，
/// 来源固定 `registry`，前缀留空以取当前登记前缀，脚本默认不执行。
pub(crate) fn official_update_request(version: String) -> RuntimeInstallRequestDto {
    RuntimeInstallRequestDto {
        prefix: None,
        source: "registry".to_string(),
        version: Some(version),
        offline_path: None,
        proxy_scheme: None,
        proxy_host: None,
        proxy_username: None,
        proxy_secret: None,
        allow_scripts: false,
    }
}

/// 卸载方案（只读）：界面先拿它渲染「将移除的对象」与备份检测，再决定是否执行。
#[tauri::command]
pub async fn plan_runtime_uninstall(
    handle: tauri::State<'_, std::sync::Arc<RuntimeHandle>>,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
) -> AppResult<RuntimeUninstallPlanDto> {
    let handle_ref = handle.inner().clone();
    let ocx = context.executable().ok();
    let data_root_path = data_root.0.clone();
    let home = context.opencodex_home.clone();
    let plan = crate::commands::run_readonly("plan runtime uninstall", move || {
        managed_uninstall_plan(&handle_ref, &data_root_path, &home)
    })
    .await?;
    Ok(RuntimeUninstallPlanDto::from_plan(&plan, ocx.as_deref()))
}

/// Validate before acquiring leases or stopping the proxy. Cancellation or an
/// unconfirmed request must leave the running process and storage untouched.
fn confirmed_uninstall_scope(request: &RuntimeUninstallRequestDto) -> AppResult<UninstallScope> {
    let scope = request.scope_kind().ok_or(AppError::RuntimeManaged {
        code: "bad_scope".to_string(),
        detail: "卸载范围必须是 body 或 full".to_string(),
    })?;
    if !request.confirmed() {
        return Err(uninstall::UninstallError::ConfirmationRequired.into());
    }
    Ok(scope)
}

/// 统一卸载（Revision 11）：覆盖**任意已解析来源**，由**应用执行**。
///
/// 顺序固定：停代理 → 备份（可选）→ 官方 `ocx uninstall`（完整卸载）→ 移除包体与入口 →
/// 清空 `OPENCODEX_HOME` 非自有残留（可选）→ 只读残留核验。任一步失败如实记录，不静默跳过。
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn uninstall_runtime(
    request: RuntimeUninstallRequestDto,
    app: tauri::AppHandle,
    handle: tauri::State<'_, std::sync::Arc<RuntimeHandle>>,
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, SharedHomeDir>,
    context: tauri::State<'_, SharedProcessContext>,
    install: tauri::State<'_, SharedRuntimeInstall>,
) -> AppResult<RuntimeUninstallResultDto> {
    let scope = confirmed_uninstall_scope(&request)?;
    let reservation = {
        let manager = app.state::<crate::commands::update::SharedUpdateStatus>();
        let state = manager.lock().map_err(|_| AppError::NotConfigured)?;
        if state.installing || state.pending_restart.is_some() {
            return Err(AppError::NotConfigured);
        }
        install.acquire().ok_or(AppError::RuntimeManaged {
            code: "install_in_progress".to_string(),
            detail: "已有安装、卸载或恢复正在进行".to_string(),
        })?
    };

    let ocx = context.executable().ok();
    let npm = if handle.resolver_managed_only() {
        None
    } else {
        discovered_executables().1
    };
    let opencodex_home = context.opencodex_home.clone();
    let data_root_path = data_root.0.clone();
    let home_dir = home.0.clone();
    let options = uninstall::UninstallOptions {
        scope,
        auto_backup: request.auto_backup,
        clean_data: request.clean_data,
        confirmed: request.confirmed(),
    };
    let plan = managed_uninstall_plan(&handle, &data_root_path, &opencodex_home)?;

    let commands = uninstall::SystemUninstallCommands {
        home: Some(home_dir),
    };
    let handle_ref = handle.inner().clone();
    let worker_app = app.clone();
    let worker_context = context.inner().clone();
    let outcome = crate::commands::run_blocking("uninstall runtime", move || {
        // Keep the reservation until the owned worker terminates, even if IPC is dropped.
        let _reservation = reservation;
        let source_version = handle_ref.store().load().resolved_version;
        let candidate = serde_json::json!({
            "scope": options.scope,
            "auto_backup": options.auto_backup,
            "clean_data": options.clean_data,
            "source_kind": plan.source_kind,
            "source_path": plan.source_path,
            "source_version": source_version,
            "remove_objects": plan.remove_objects,
            "runtime_objects": plan.runtime_objects,
        });
        let identity = crate::commands::event_delivery::prepare(
            &data_root_path,
            "runtime-uninstall-failed",
            crate::modules::notifications::registry::Channel::Local,
            candidate.to_string().as_bytes(),
        );
        let result: AppResult<_> = (|| {
            {
                let guard = crate::modules::backup::manager::acquire_preferences_transaction(
                    &data_root_path,
                )?;
                crate::modules::runtime::protection::reconcile_pending(&data_root_path, &guard)?;
            }
            let runner = worker_app.state::<SharedProcessRunner>();
            let stopped = crate::commands::process_action_with_runner(
                crate::types::process_action::ProcessActionRequest {
                    action: crate::modules::process::LifecycleAction::Stop,
                    confirm: true,
                },
                runner.inner(),
                &worker_context,
                &crate::infrastructure::runtime_log::RuntimeLog::new(&data_root_path),
                None,
            )?;
            if stopped.result != Some(crate::modules::process::LifecycleResult::Stopped) {
                return Err(AppError::RuntimeManaged {
                    code: "stop_unconfirmed".to_string(),
                    detail: "代理停止尚未确认，未进入卸载步骤".to_string(),
                });
            }
            if handle_ref.resolver_managed_only()
                && managed_uninstall_plan(&handle_ref, &data_root_path, &opencodex_home)? != plan
            {
                return Err(AppError::RuntimeManaged {
                    code: "uninstall_plan_changed".into(),
                    detail: "受管程序或卸载方案已改变，请重新预览".into(),
                });
            }
            let outcome = uninstall::execute_uninstall(
                &data_root_path,
                &opencodex_home,
                &plan,
                &options,
                ocx.as_deref(),
                npm.as_deref(),
                &commands,
            )?;
            let verified = outcome.verified_complete();
            let target = plan.source_path.clone().unwrap_or_default();
            let resolution_after = handle_ref.refresh();
            let version = source_version.as_deref().unwrap_or("unknown");
            let _ = handle_ref.record_history(uninstall::history_entry(
                if verified { "succeeded" } else { "failed" },
                version,
                Path::new(&target),
                (!verified).then(|| "uninstall_incomplete".to_string()),
            ));
            Ok((outcome, resolution_after))
        })();
        // Terminal delivery belongs to the worker, not its IPC observer.
        // A delivery error cannot undo an already completed removal.
        crate::commands::event_delivery::publish(
            &worker_app,
            &data_root_path,
            if result
                .as_ref()
                .is_ok_and(|(outcome, _)| outcome.verified_complete())
            {
                "runtime-uninstall-succeeded"
            } else {
                "runtime-uninstall-failed"
            },
            identity,
            crate::modules::notifications::registry::Trigger::User,
        );
        result
    })
    .await;
    let (outcome, resolution_after) = outcome?;
    broadcast_source_changed(&app);

    let failed_steps = outcome
        .steps
        .iter()
        .filter(|step| step.status == uninstall::StepStatus::Failed)
        .count();
    let message = if failed_steps > 0 {
        format!("卸载未核验完成：{failed_steps} 步失败；请按步骤详情处理")
    } else if !outcome.verified_complete() {
        "卸载未核验完成：仍有残留或缺少清除证据；请查看残留核验与诊断记录".to_string()
    } else {
        format!(
            "已按「{}」卸载；运行来源现在为「{}」",
            match scope {
                UninstallScope::Body => "仅移除包体与入口",
                UninstallScope::Full => "完整卸载",
            },
            uninstall::source_summary(resolution_after.kind),
        )
    };

    Ok(RuntimeUninstallResultDto {
        scope,
        source_kind: resolution_after.kind,
        source_path: resolution_after
            .path
            .map(|path| path.to_string_lossy().into_owned()),
        backup_id: outcome.backup_id,
        backup_directory: outcome
            .backup_directory
            .map(|path| path.to_string_lossy().into_owned()),
        steps: outcome
            .steps
            .into_iter()
            .map(|step| RuntimeUninstallStepDto {
                name: step.name,
                status: step.status.as_str().to_string(),
                detail: step.detail,
            })
            .collect(),
        residue: outcome
            .residue
            .into_iter()
            .map(|item| RuntimeUninstallResidueDto {
                path: item.path,
                status: item.status.as_str().to_string(),
            })
            .collect(),
        official_output: outcome.official_output,
        needs_restart: false,
        message,
    })
}

/// 只读展示官方卸载留下的（掩码后）结果；本产品不执行官方命令。
#[tauri::command]
pub fn official_uninstall_observation(
    context: tauri::State<'_, SharedProcessContext>,
) -> AppResult<Vec<String>> {
    Ok(uninstall::official_uninstall_observation(
        &context.opencodex_home,
    ))
}

/// 供命令层测试直接使用的取消标志类型别名，避免测试引入内核实现细节。
pub type RuntimeCancelFlag = CancelFlag;

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::modules::runtime::install::InstallSourceKind;

    #[test]
    fn protection_retry_refuses_manager_handoff_and_installation_without_runtime_effects() {
        let runtime = SharedRuntimeInstall::new();
        let manager = std::sync::Arc::new(std::sync::Mutex::new(
            crate::modules::update::UpdateStatus::pending("0.1.9"),
        ));
        manager.lock().unwrap().installing = true;
        assert!(reserve_protection_retry(&runtime, &manager).is_err());
        drop(runtime.acquire().expect("refusal must not reserve runtime"));
        manager.lock().unwrap().installing = false;
        manager.lock().unwrap().pending_restart = Some("0.1.10".into());
        assert!(reserve_protection_retry(&runtime, &manager).is_err());
        drop(
            runtime
                .acquire()
                .expect("handoff refusal must not reserve runtime"),
        );
    }

    #[test]
    fn protection_retry_owns_runtime_exclusion_until_worker_lease_drops() {
        let runtime = SharedRuntimeInstall::new();
        let manager = std::sync::Arc::new(std::sync::Mutex::new(
            crate::modules::update::UpdateStatus::pending("0.1.9"),
        ));
        let worker_lease = reserve_protection_retry(&runtime, &manager).unwrap();
        assert!(runtime.acquire().is_none());
        assert!(reserve_protection_retry(&runtime, &manager).is_err());
        drop(worker_lease);
        assert!(reserve_protection_retry(&runtime, &manager).is_ok());
    }

    fn handle_for(root: &Path) -> std::sync::Arc<RuntimeHandle> {
        RuntimeHandle::initialize(root, Vec::new())
    }

    #[test]
    fn uninstall_admission_requires_valid_scope_and_explicit_confirmation() {
        let mut request = RuntimeUninstallRequestDto {
            scope: "body".to_string(),
            auto_backup: true,
            clean_data: false,
            confirmation: None,
        };
        for confirmation in [None, Some("".to_string()), Some("  ".to_string())] {
            request.confirmation = confirmation;
            assert!(matches!(
                confirmed_uninstall_scope(&request),
                Err(AppError::RuntimeManaged { code, .. })
                    if code == "uninstall_confirmation_required"
            ));
        }
        request.confirmation = Some("confirmed".to_string());
        assert_eq!(
            confirmed_uninstall_scope(&request).unwrap(),
            UninstallScope::Body
        );
        request.scope = "full".to_string();
        assert_eq!(
            confirmed_uninstall_scope(&request).unwrap(),
            UninstallScope::Full
        );
        request.scope = "invalid".to_string();
        assert!(matches!(
            confirmed_uninstall_scope(&request),
            Err(AppError::RuntimeManaged { code, .. }) if code == "bad_scope"
        ));
    }

    #[test]
    fn source_dto_reports_managed_prefix_and_no_entry_until_managed() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let handle = handle_for(&data_root);
        let dto = source_dto(&handle, &data_root).unwrap();
        assert_eq!(dto.kind, RuntimeSourceKind::Unresolved);
        assert_eq!(dto.path, None);
        assert!(!dto.inside_data_root);
        assert!(dto.managed_entry.is_none());
        assert_eq!(
            dto.managed_prefix,
            data_root
                .join("runtime/opencodex")
                .to_string_lossy()
                .into_owned()
        );
        assert!(dto.history.is_empty());
    }

    #[test]
    fn source_dto_marks_entry_and_inside_root_when_managed() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let entry = data_root.join(crate::modules::runtime::MANAGED_ENTRY_RELATIVE);
        std::fs::create_dir_all(entry.parent().expect("parent")).expect("mkdir");
        std::fs::write(&entry, b"#!/bin/sh\nexit 0\n").expect("write");
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&entry, std::fs::Permissions::from_mode(0o755)).expect("chmod");

        let handle = handle_for(&data_root);
        let dto = source_dto(&handle, &data_root).unwrap();
        assert_eq!(dto.kind, RuntimeSourceKind::Managed);
        assert!(dto.inside_data_root);
        assert_eq!(
            dto.managed_entry.as_deref(),
            Some(entry.to_string_lossy().as_ref())
        );
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    #[test]
    fn source_dto_uses_selected_disk_version_and_keeps_invalid_history_untouched() {
        let f = crate::modules::runtime::program_selection::tests::Fixture::new();
        let t = tempfile::tempdir().unwrap();
        let data = t.path().canonicalize().unwrap().join("desktop");
        let store = crate::modules::runtime::RuntimeStore::new(&data);
        std::fs::create_dir_all(store.path().parent().unwrap()).unwrap();
        std::fs::write(store.path(), b"{invalid historical metadata").unwrap();
        let before = std::fs::read(store.path()).unwrap();
        let h = RuntimeHandle::initialize_versioned(&data, &f.root);
        let dto = source_dto(&h, &data).unwrap();
        assert_eq!(dto.version.as_deref(), Some("2.99.0"));
        assert_eq!(dto.managed_prefix, f.paths.package_root.to_string_lossy());
        assert!(dto
            .managed_entry
            .unwrap()
            .ends_with("runtime/bin/ocx-runtime"));
        assert_eq!(std::fs::read(store.path()).unwrap(), before);
        std::fs::write(f.root.join("runtime/current.json"), b"{").unwrap();
        assert!(source_dto(&h, &data).is_err());
        assert_eq!(std::fs::read(store.path()).unwrap(), before);
    }

    #[test]
    fn rejection_messages_are_human_readable_and_specific() {
        assert!(describe_rejection(&ArchiveRejection::NotGzip).contains("gzip"));
        assert!(describe_rejection(&ArchiveRejection::NotOfficialPackage {
            found: "evil".to_string()
        })
        .contains("evil"));
        assert!(describe_rejection(&ArchiveRejection::PathEscape {
            entry: "../x".to_string()
        })
        .contains("../x"));
    }

    #[test]
    fn install_source_kind_is_carried_through_dto() {
        let request = RuntimeInstallRequestDto {
            prefix: None,
            source: "offline".to_string(),
            version: None,
            offline_path: Some("/tmp/opencodex-0.3.1.tgz".to_string()),
            proxy_scheme: None,
            proxy_host: None,
            proxy_username: None,
            proxy_secret: None,
            allow_scripts: false,
        };
        assert_eq!(request.source_kind(), Some(InstallSourceKind::Offline));
    }

    #[test]
    fn official_update_binds_registry_version_and_current_prefix() {
        let request = official_update_request("2.66.0".to_string());
        assert_eq!(request.source_kind(), Some(InstallSourceKind::Registry));
        // U-04b：绑定到查询到的确定版本，不浮动 `latest`。
        assert_eq!(request.version.as_deref(), Some("2.66.0"));
        // 前缀留空：安装时取当前登记前缀，不写全局 npm 前缀。
        assert!(request.prefix_path().is_none());
        assert!(!request.allow_scripts);
        assert!(request.offline_archive().is_none());
    }
}

/// v2 plans use I for program objects and D for backup/state. Never classify a
/// legacy/external program or derive an installation prefix from runtime history.
fn managed_uninstall_plan(
    handle: &RuntimeHandle,
    data_root: &Path,
    home: &Path,
) -> AppResult<uninstall::UninstallPlan> {
    let resolution = handle.current();
    if !handle.resolver_managed_only() {
        return Ok(uninstall::plan_uninstall(
            data_root,
            home,
            resolution.kind,
            resolution.path.as_deref(),
        ));
    }
    let root = handle.installation_root();
    let (node, _) = crate::modules::runtime::install::managed_dependency_paths(root);
    crate::modules::runtime::install::validate_managed_installation(root, Some(&node))?;
    let entry = root.join(crate::modules::runtime::MANAGED_ENTRY_RELATIVE);
    if resolution.kind != RuntimeSourceKind::Managed
        || resolution.path.as_deref() != Some(entry.as_path())
    {
        return Err(AppError::RuntimeManaged {
            code: "uninstall_source_unresolved".into(),
            detail: "当前受管来源未核验；请先诊断或恢复安装".into(),
        });
    }
    let prefix = root.join(crate::modules::runtime::MANAGED_PREFIX_RELATIVE);
    let mut plan = uninstall::plan_uninstall(data_root, home, RuntimeSourceKind::Unresolved, None);
    plan.source_kind = RuntimeSourceKind::Managed;
    plan.source_path = Some(entry.to_string_lossy().into_owned());
    plan.remove_objects = vec![
        prefix.to_string_lossy().into_owned(),
        entry.to_string_lossy().into_owned(),
    ];
    plan.owner = uninstall::BodyOwner::Managed { prefix, entry };
    Ok(plan)
}

// Pure backend entry points. Main registers IPC wrappers and holds operation
// objects by ID, shared install/storage/preferences leases and process ownership.
// A client must never supply a deserialized RuntimeRecoveryOperation.
pub fn prepare_runtime_recovery(
    installer: &RuntimeInstaller<'_>,
    source: crate::modules::runtime::install::RuntimeRecoverySource,
    npm: PathBuf,
    service: &dyn crate::modules::runtime::install::RecoveryServiceBoundary,
) -> AppResult<crate::modules::runtime::install::RuntimeRecoveryOperation> {
    validate_recovery_dependencies(installer, Some(&npm))?;
    installer
        .prepare_recovery(source, npm, service)
        .map_err(AppError::from)
}
pub fn apply_runtime_recovery(
    installer: &RuntimeInstaller<'_>,
    operation: &mut crate::modules::runtime::install::RuntimeRecoveryOperation,
    confirmation: &str,
    service: &dyn crate::modules::runtime::install::RecoveryServiceBoundary,
) -> AppResult<crate::modules::runtime::install::InstallOutcome> {
    let (node, npm) = operation.dependency_paths();
    validate_recovery_dependencies(installer, Some(npm))?;
    if installer.node_path.as_deref() != Some(node) {
        return Err(AppError::RuntimeManaged {
            code: "managed_dependency_mismatch".into(),
            detail: "冻结恢复操作的 Node 与当前集成依赖不一致".into(),
        });
    }
    installer
        .apply_recovery(operation, confirmation, service)
        .map_err(AppError::from)
}
pub fn runtime_recovery_status(
    data_root: &Path,
    operation_id: &str,
) -> AppResult<crate::modules::runtime::install::RuntimeRecoveryStatus> {
    crate::modules::runtime::install::RuntimeRecoveryOperation::read_status(data_root, operation_id)
        .map_err(AppError::from)
}
pub fn cancel_runtime_recovery(
    operation: &mut crate::modules::runtime::install::RuntimeRecoveryOperation,
) -> AppResult<crate::modules::runtime::install::RuntimeRecoveryStatus> {
    operation.cancel().map_err(AppError::from)
}

fn validate_recovery_dependencies(
    installer: &RuntimeInstaller<'_>,
    requested_npm: Option<&Path>,
) -> AppResult<()> {
    let root = installer
        .installation_root
        .as_deref()
        .ok_or_else(|| AppError::RuntimeManaged {
            code: "installation_context_missing".into(),
            detail: "恢复需要可信 Desktop 安装上下文".into(),
        })?;
    let (node, npm) = fixed_executables(root)?;
    if installer.node_path.as_deref() != Some(node.as_path())
        || requested_npm.is_some_and(|path| path != npm)
    {
        return Err(AppError::RuntimeManaged {
            code: "managed_dependency_mismatch".into(),
            detail: "恢复只能使用当前 Desktop 集成的固定 Node/npm".into(),
        });
    }
    Ok(())
}

/// The remote query engine accepts a launcher, so bind the selected Node/CLI in
/// a private short-lived launcher while retaining its timeout, logs and proxy.
/// No CLI shebang, shell lookup, discovered Node or inherited PATH is used.
pub fn query_runtime_remote_latest(
    handle: &RuntimeHandle,
    working_directory: &Path,
    cache: &Path,
    logs: &Path,
    environment: &crate::modules::process::EnvironmentPolicy,
    tag: &str,
) -> AppResult<crate::modules::about::remote::OfficialRemoteLatest> {
    let (node, npm) = managed_executables(handle)?;
    let npm = npm.ok_or(AppError::NotConfigured)?;
    if !handle.resolver_managed_only() {
        return crate::modules::about::remote::query_remote_latest(
            &npm,
            working_directory,
            cache,
            logs,
            environment,
            tag,
        );
    }
    let node = node.ok_or(AppError::NotConfigured)?;
    let desktop_root = handle
        .store()
        .path()
        .parent()
        .and_then(Path::parent)
        .ok_or(AppError::NotConfigured)?;
    crate::modules::backup::safety::check_path(desktop_root, cache, true)?;
    std::fs::create_dir_all(cache).map_err(|e| AppError::FileSystem {
        operation: "create query cache".into(),
        detail: e.to_string(),
    })?;
    let launcher_directory = tempfile::Builder::new()
        .prefix("managed-npm-query-")
        .tempdir_in(cache)
        .map_err(|e| AppError::FileSystem {
            operation: "create fixed npm launcher".into(),
            detail: e.to_string(),
        })?;
    let launcher = launcher_directory
        .path()
        .join(if cfg!(windows) { "npm.cmd" } else { "npm.sh" });
    let script = crate::modules::runtime::install::entry_script(Some(&node), &npm);
    crate::infrastructure::atomic_write::atomic_write(&launcher, script.as_bytes(), 0o700)?;
    crate::modules::about::remote::query_remote_latest(
        &launcher,
        working_directory,
        cache,
        logs,
        environment,
        tag,
    )
}
