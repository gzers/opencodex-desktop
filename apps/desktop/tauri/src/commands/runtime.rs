//! 运行来源、托管安装与卸载命令（IMP Track B · B3 / B4 / B5）。
//!
//! 命令层只做三件事：解析输入、编排模块、把进度与终态转发给界面。
//! **安装进度不落盘**，只有终态才写 `runtime.json` 历史（`FZ-48`）。

use std::path::{Path, PathBuf};

use tauri::Emitter;

use crate::errors::{AppError, AppResult};
use crate::modules::runtime::archive::{self, ArchiveRejection};
use crate::modules::runtime::install::{
    CancelFlag, InstallProgress, InstallProgressSink, InstallRequest, InstallSource,
    RuntimeInstaller, DEFAULT_VERSION,
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

/// 已发现的 node / npm 绝对路径（**不读 PATH**，`FZ-13` / `FZ-50`）。
fn discovered_executables() -> (Option<PathBuf>, Option<PathBuf>) {
    let paths = crate::types::discovery_paths::macos_default_paths();
    let validated = |path: &Path| crate::modules::runtime::paths::validate_executable(path).ok();
    (validated(&paths.node), validated(&paths.npm))
}

/// 把安装进度转发成 Tauri 事件；凭据已在内核里掩码。
struct EventProgressSink {
    app: tauri::AppHandle,
}

impl InstallProgressSink for EventProgressSink {
    fn emit(&self, progress: InstallProgress) {
        let _ = self.app.emit(RUNTIME_INSTALL_PROGRESS_EVENT, &progress);
    }
}

fn broadcast_source_changed(app: &tauri::AppHandle) {
    let _ = app.emit(RUNTIME_SOURCE_CHANGED_EVENT, ());
}

fn source_dto(handle: &RuntimeHandle, data_root: &Path) -> RuntimeSourceDto {
    let resolution = handle.current();
    let record = handle.store().load();
    // 卸载与「重装同版本」按**安装记录登记的**前缀定位（`FZ-51`）：
    // `FZ-47` 允许自定义前缀，那时前缀不再是数据根的默认位置。
    let managed_prefix = record.installed_prefix(data_root);
    let managed_entry = data_root.join(crate::modules::runtime::MANAGED_ENTRY_RELATIVE);
    RuntimeSourceDto {
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
        version: record.resolved_version.clone(),
        resolved_at: record.resolved_at.clone(),
        managed_entry: matches!(resolution.kind, RuntimeSourceKind::Managed)
            .then(|| managed_entry.to_string_lossy().into_owned()),
        managed_prefix: managed_prefix.to_string_lossy().into_owned(),
        default_prefix: data_root
            .join(crate::modules::runtime::MANAGED_PREFIX_RELATIVE)
            .to_string_lossy()
            .into_owned(),
        explicit_path: handle
            .explicit_path()
            .map(|path| path.to_string_lossy().into_owned()),
        history: record.history,
    }
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
pub fn runtime_source(
    handle: tauri::State<'_, std::sync::Arc<RuntimeHandle>>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> AppResult<RuntimeSourceDto> {
    Ok(source_dto(handle.inner(), &data_root.0))
}

/// 用户显式指定运行来源；路径必须通过可执行校验，否则保持原选择。
#[tauri::command]
pub fn set_runtime_source(
    app: tauri::AppHandle,
    path: Option<String>,
    handle: tauri::State<'_, std::sync::Arc<RuntimeHandle>>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> AppResult<RuntimeSourceDto> {
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
    Ok(source_dto(handle.inner(), &data_root.0))
}

/// 恢复自动发现（清除显式指定）。
#[tauri::command]
pub fn restore_discovered_runtime(
    app: tauri::AppHandle,
    handle: tauri::State<'_, std::sync::Arc<RuntimeHandle>>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> AppResult<RuntimeSourceDto> {
    handle.set_explicit(None);
    broadcast_source_changed(&app);
    Ok(source_dto(handle.inner(), &data_root.0))
}

/// 离线包预检：只校验不展开，让拖拽区能立刻给出「已选 / 校验失败 / 拒绝」。
#[tauri::command]
pub async fn preview_offline_package(path: String) -> AppResult<OfflinePackagePreviewDto> {
    crate::commands::run_blocking("preview offline package", move || {
        let archive_path = PathBuf::from(&path);
        let file_name = archive_path
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.clone());
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
    })
    .await
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
    let source_kind = request.source_kind().ok_or(AppError::RuntimeManaged {
        code: "bad_source".to_string(),
        detail: "安装源必须是 registry 或 offline".to_string(),
    })?;
    let prefix = request
        .prefix_path()
        // 缺省沿用安装记录登记的前缀（自定义前缀安装后不要悄悄回到默认位置）。
        .unwrap_or_else(|| handle.store().load().installed_prefix(&data_root.0));
    let requested_version = request
        .version
        .clone()
        .unwrap_or_else(|| DEFAULT_VERSION.to_string());
    let source = match source_kind {
        crate::modules::runtime::install::InstallSourceKind::Registry => InstallSource::Registry {
            version: requested_version.clone(),
            proxy: request.proxy(),
        },
        crate::modules::runtime::install::InstallSourceKind::Offline => InstallSource::Offline {
            archive: request.offline_archive().ok_or(AppError::RuntimeManaged {
                code: "offline_path_missing".to_string(),
                detail: "离线安装需要选择一个 .tgz 文件".to_string(),
            })?,
        },
    };
    let (node_path, npm_path) = discovered_executables();
    let install_request = InstallRequest {
        prefix: prefix.clone(),
        source,
        allow_scripts: request.allow_scripts,
        npm_path,
        node_path: node_path.clone(),
    };

    let cancel = install.begin().ok_or(AppError::RuntimeManaged {
        code: "install_in_progress".to_string(),
        detail: "已有安装正在进行".to_string(),
    })?;
    let proxy_running = proxy_is_running(&collector);
    let data_root_path = data_root.0.clone();
    let home_path = home.0.clone();
    let sink = EventProgressSink { app: app.clone() };
    let handle_ref = handle.inner().clone();

    let result = crate::commands::run_blocking("install managed runtime", move || {
        let npm = crate::modules::runtime::install::SystemNpmRunner::new(Some(home_path.clone()));
        let probe = crate::modules::runtime::install::RealVersionProbe::default();
        let mut installer = RuntimeInstaller::new(
            &data_root_path,
            &npm,
            Some(home_path),
            &sink,
            &probe,
            node_path.clone(),
        );
        installer.cancel = cancel.clone();
        match installer.install(&install_request) {
            Ok(outcome) => {
                handle_ref.refresh();
                // 安装刚落地，版本是确定事实：回填到运行来源记录，卡片与
                // `runtime.json` 的 `resolved_version` 才不会停在「未知」（`FZ-48`）。
                handle_ref.record_resolved_version(&outcome.version);
                let entry = outcome.history_entry("succeeded", None);
                let _ = handle_ref.record_history(entry);
                Ok(outcome)
            }
            Err(error) => {
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
    })
    .await;
    // 无论成败都要放掉进行态，否则后续安装会被自己挡住。
    install.finish();
    let outcome = result?;

    broadcast_source_changed(&app);
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
    })
}

/// 请求取消当前安装（内核会终止 npm 子进程并清理临时前缀）。
#[tauri::command]
pub fn cancel_runtime_install(install: tauri::State<'_, SharedRuntimeInstall>) -> AppResult<bool> {
    Ok(install.cancel())
}

/// 卸载方案（只读）：界面先拿它渲染「将移除的对象」与备份检测，再决定是否执行。
#[tauri::command]
pub async fn plan_runtime_uninstall(
    handle: tauri::State<'_, std::sync::Arc<RuntimeHandle>>,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
) -> AppResult<RuntimeUninstallPlanDto> {
    let resolution = handle.current();
    let ocx = context.executable().ok();
    let data_root_path = data_root.0.clone();
    let home = context.opencodex_home.clone();
    let plan = crate::commands::run_blocking("plan runtime uninstall", move || {
        Ok(uninstall::plan_uninstall(
            &data_root_path,
            &home,
            resolution.kind,
            resolution.path.as_deref(),
        ))
    })
    .await?;
    Ok(RuntimeUninstallPlanDto::from_plan(&plan, ocx.as_deref()))
}

/// 统一卸载（Revision 11）：覆盖**任意已解析来源**，由**应用执行**。
///
/// 顺序固定：停代理 → 备份（可选）→ 移除包体与入口 → 官方 `ocx uninstall`（完整卸载）→
/// 清空 `OPENCODEX_HOME` 非自有残留（可选）→ 只读残留核验。任一步失败如实记录，不静默跳过。
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn uninstall_runtime(
    request: RuntimeUninstallRequestDto,
    app: tauri::AppHandle,
    handle: tauri::State<'_, std::sync::Arc<RuntimeHandle>>,
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, SharedHomeDir>,
    runner: tauri::State<'_, SharedProcessRunner>,
    context: tauri::State<'_, SharedProcessContext>,
    install: tauri::State<'_, SharedRuntimeInstall>,
) -> AppResult<RuntimeUninstallResultDto> {
    let scope = request.scope_kind().ok_or(AppError::RuntimeManaged {
        code: "bad_scope".to_string(),
        detail: "卸载范围必须是 body 或 full".to_string(),
    })?;
    let _guard = install.begin().ok_or(AppError::RuntimeManaged {
        code: "install_in_progress".to_string(),
        detail: "已有安装或卸载正在进行".to_string(),
    })?;

    // 停代理：本产品托管的子进程必须先停，未停则不进入删除步骤（与官方 §3008 同源）。
    let _ = crate::commands::process_action_with_runner(
        crate::types::process_action::ProcessActionRequest {
            action: crate::modules::process::LifecycleAction::Stop,
            confirm: true,
        },
        runner.inner(),
        context.inner(),
        &crate::infrastructure::runtime_log::RuntimeLog::new(&data_root.0),
        None,
    );

    let resolution = handle.current();
    let ocx = context.executable().ok();
    let npm = discovered_executables().1;
    let opencodex_home = context.opencodex_home.clone();
    let data_root_path = data_root.0.clone();
    let home_dir = home.0.clone();
    let options = uninstall::UninstallOptions {
        scope,
        auto_backup: request.auto_backup,
        clean_data: request.clean_data,
        confirmed: request.confirmed(),
    };
    let plan = uninstall::plan_uninstall(
        &data_root_path,
        &opencodex_home,
        resolution.kind,
        resolution.path.as_deref(),
    );

    let commands = uninstall::SystemUninstallCommands {
        home: Some(home_dir),
    };
    let handle_ref = handle.inner().clone();
    let outcome = crate::commands::run_blocking("uninstall runtime", move || {
        let outcome = uninstall::execute_uninstall(
            &data_root_path,
            &opencodex_home,
            &plan,
            &options,
            ocx.as_deref(),
            npm.as_deref(),
            &commands,
        )?;
        let failed = outcome
            .steps
            .iter()
            .any(|step| step.status == uninstall::StepStatus::Failed);
        let target = plan.source_path.clone().unwrap_or_default();
        let resolution_after = handle_ref.refresh();
        let version = handle_ref
            .store()
            .load()
            .resolved_version
            .unwrap_or_else(|| DEFAULT_VERSION.to_string());
        let _ = handle_ref.record_history(uninstall::history_entry(
            if failed { "failed" } else { "succeeded" },
            &version,
            Path::new(&target),
            None,
        ));
        Ok((outcome, resolution_after))
    })
    .await;
    install.finish();
    let (outcome, resolution_after) = outcome?;
    broadcast_source_changed(&app);

    let failed_steps = outcome
        .steps
        .iter()
        .filter(|step| step.status == uninstall::StepStatus::Failed)
        .count();
    let residue_present = outcome
        .residue
        .iter()
        .any(|item| item.status != uninstall::ResidueStatus::Cleared);
    let message = if failed_steps > 0 {
        format!("卸载完成但 {failed_steps} 步失败；请按步骤详情处理，未清理项已列入残留核验")
    } else {
        format!(
            "已按「{}」卸载；运行来源现在为「{}」{}",
            match scope {
                UninstallScope::Body => "仅移除包体与入口",
                UninstallScope::Full => "完整卸载",
            },
            uninstall::source_summary(resolution_after.kind),
            if residue_present {
                "（残留核验有未清除项）"
            } else {
                ""
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::runtime::install::InstallSourceKind;

    fn handle_for(root: &Path) -> std::sync::Arc<RuntimeHandle> {
        RuntimeHandle::initialize(root, Vec::new())
    }

    #[test]
    fn source_dto_reports_managed_prefix_and_no_entry_until_managed() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let handle = handle_for(&data_root);
        let dto = source_dto(&handle, &data_root);
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
        let dto = source_dto(&handle, &data_root);
        assert_eq!(dto.kind, RuntimeSourceKind::Managed);
        assert!(dto.inside_data_root);
        assert_eq!(
            dto.managed_entry.as_deref(),
            Some(entry.to_string_lossy().as_ref())
        );
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
}
