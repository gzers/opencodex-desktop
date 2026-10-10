//! OpenCodeX Desktop 桌面壳。
//!
//! 当前提交只建立工程分层，不实现业务能力。业务模块先保持占位，
//! 后续工作包按 TASK 写入范围逐步填充。

pub mod commands;
pub mod errors;
pub mod infrastructure;
pub mod modules;
pub mod state;
pub mod types;

/// 构建时注入的源码提交（由 `build.rs` 写入 `OPENCODEX_BUILD_COMMIT`）。
/// 取不到时返回空串，调用方如实说明为「未知」，不伪造来源（F-09）。
pub fn build_commit() -> &'static str {
    option_env!("OPENCODEX_BUILD_COMMIT").unwrap_or("")
}

/// 把入口名写成一条非敏感运行日志；失败不阻断导航，也不携带窗口/凭据细节。
fn record_window_event(data_root: &std::path::Path, message: &str) {
    let _ = crate::infrastructure::runtime_log::RuntimeLog::new(data_root).append_event(message);
}

/// 统一的主窗口重开路径（F-03）：Dock reopen / 托盘 / 原生菜单走同一处，
/// 记录入口与每个原生调用的成败，避免各处各自 show/focus 且结果被静默忽略。
fn reveal_main_window<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    entry: &str,
    data_root: &std::path::Path,
) {
    use tauri::Manager;
    let Some(window) = app.get_window("main") else {
        record_window_event(data_root, &format!("window {entry}: main window missing"));
        return;
    };
    let show = window.show().is_ok();
    let focus = window.set_focus().is_ok();
    apply_dock_visibility(app, true);
    record_window_event(
        data_root,
        &format!("window {entry}: show={show} focus={focus} dock=regular"),
    );
}

/// 原生「重载主界面」（F-06）：直接调用主 WebView 的重载，不依赖前端轮询。
/// 与官方面板刷新、运行时启停区分；只记录成功与否，不宣称已修复绘制异常。
fn reload_main_webview<R: tauri::Runtime>(app: &tauri::AppHandle<R>, data_root: &std::path::Path) {
    use tauri::Manager;
    let Some(webview) = app.get_webview("main") else {
        record_window_event(data_root, "reload main: webview missing");
        return;
    };
    match webview.reload() {
        Ok(()) => record_window_event(data_root, "reload main: requested"),
        Err(error) => record_window_event(data_root, &format!("reload main: failed ({error})")),
    }
}

/// 启动事件里附构建来源；提交缺失时如实标注「未知」。
fn record_startup_event(data_root: &std::path::Path, version: &str) {
    let commit = build_commit();
    let commit = if commit.is_empty() { "unknown" } else { commit };
    record_window_event(
        data_root,
        &format!("startup version={version} commit={commit}"),
    );
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::Manager;
    let context = tauri::generate_context!();
    #[cfg(windows)]
    let (context, smoke_window) = {
        let mut context = context;
        let smoke = crate::infrastructure::windows_smoke::from_environment(context.config_mut())
            .expect("invalid Windows smoke configuration");
        (context, smoke)
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        // 目录选择交给系统原生对话框（FZ-23「自定义源目录」），不自造浏览器。
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(move |app| {
            use tauri::Manager;
            #[cfg(windows)]
            if let Some(smoke) = smoke_window {
                tauri::WebviewWindowBuilder::from_config(app, &smoke.config)?
                    .data_directory(smoke.data_directory)
                    .build()?;
            }

            let default_root = app.path().app_data_dir().map_err(|error| {
                Box::new(crate::errors::AppError::FileSystem {
                    operation: "resolve app data directory".to_string(),
                    detail: error.to_string(),
                }) as Box<dyn std::error::Error>
            })?;
            // 测试沙箱身份（配置规划§10）：启用时强制使用独立沙箱根；根缺失/非法即停止，
            // 绝不回退到日常目录。未启用时行为与此前一致。
            let bootstrap_root = if crate::modules::test_sandbox::enabled() {
                default_root.clone()
            } else {
                crate::modules::data_root::bootstrap::default_anchor(
                    &default_root, &std::env::current_exe()?, cfg!(windows),
                )?
            };
            let anchor = crate::modules::test_sandbox::resolve_data_root(&bootstrap_root)
                .map_err(|detail| {
                    Box::new(crate::errors::AppError::FileSystem {
                        operation: "resolve sandbox data root".to_string(),
                        detail,
                    }) as Box<dyn std::error::Error>
                })?;
            // FZ-02 首次启动先初始化或引用当前版本的数据根；失败阻断启动。
            let sandbox_boundary = crate::modules::test_sandbox::resolve_boundary(&anchor)
                .map_err(std::io::Error::other)?;
            let (runtime_config, locks) = crate::modules::data_root::bootstrap::resolve_locked_with_boundary(
                &anchor, sandbox_boundary.as_deref(),
            )?;
            let data_root = runtime_config.active_data_root.clone();
            app.manage(crate::state::SharedDataRootAnchor(anchor));
            app.manage(crate::state::SharedSandboxBoundary(sandbox_boundary));
            app.manage(crate::state::InstanceState { locks: std::sync::Mutex::new(locks) });
            // 配置格式自动转换（§5）：启动时按需迁移旧 schema 偏好并完成未提交事务；
            // 已是当前 schema 不写盘，损坏/过新不覆盖原件。失败只记日志，不阻断启动。
            if let Err(error) = crate::modules::config_migration::migrate_preferences_on_startup(&data_root)
            {
                record_window_event(&data_root, &format!("config migration: {error}"));
            }
            app.manage(crate::state::SharedDataRoot(data_root.clone()));
            // 启动事件附构建来源：维护事故可把本机安装映射回提交（F-09）。
            record_startup_event(&data_root, &app.package_info().version.to_string());
            app.manage(std::sync::Mutex::new(
                crate::modules::sync::SyncRun {
                    run_id: "sync-initial".to_string(),
                    direction: crate::modules::sync::SyncDirection::Upload,
                    connection_state: crate::types::status::ConnectionState::Unconfigured,
                    operation_state: crate::types::status::OperationState::Idle,
                    started_at: chrono::Utc::now(),
                    finished_at: None,
                    items_total: 0,
                    items_done: 0,
                    failure_reason: None,
                    snapshot_id: None,
                },
            ));
            app.manage(crate::state::SharedTrayRequests::new());
            // 「启动时清理」：开启时按保留策略做一次有界的轻量清理（日志截断、通知与
            // 通知保留期），必须在通知集合载入前完成；启动不扫描、校验或删除备份。
            let (startup_cleanup, cleanup_identity) =
                crate::commands::cleanup::run_startup_cleanup_registered(&data_root, chrono::Utc::now());
            match &startup_cleanup {
                Ok(outcome) if outcome.ran
                    && (outcome.trimmed_log_files > 0 || outcome.pruned_notifications > 0) => {
                        record_window_event(&data_root, &format!(
                            "startup cleanup: logs={} lines={} notifications={}",
                            outcome.trimmed_log_files, outcome.trimmed_log_lines, outcome.pruned_notifications,
                        ));
                }
                Err(_) => record_window_event(&data_root, "startup cleanup failed; retained unreadable files"),
                _ => {}
            }
            // 通知实体持久化在数据根的 `manager-state/notifications.json`，
            // 重启后按 read / resolved / deleted 原样恢复；正式运行不预置任何演示种子
            // （《数据与状态》§5.7、《领域模型》§16）。
            app.manage(std::sync::Arc::new(std::sync::Mutex::new(
                crate::commands::notifications::startup_notification_store(&data_root),
            )));
            // Terminal facts use the post-cleanup store, never an earlier in-memory copy.
            if match &startup_cleanup { Ok(outcome) => outcome.ran, Err(_) => true } {
                crate::commands::event_delivery::publish(
                    app.handle(), &data_root,
                    if startup_cleanup.is_ok() { "local-cleanup-succeeded" } else { "local-cleanup-failed" },
                    cleanup_identity, crate::modules::notifications::registry::Trigger::Startup,
                );
            }
            if crate::commands::update::reconcile_startup(app.handle(), &data_root).is_err() {
                record_window_event(&data_root, "manager installation reconciliation failed; receipt retained");
            }
            app.manage(std::sync::Arc::new(std::sync::Mutex::new(
                crate::modules::update::UpdateStatus::pending(app.package_info().version.to_string())
                    .with_channel(update_channel(&data_root)),
            )) as crate::commands::update::SharedUpdateStatus);

            // 状态采集仍只使用受控发现路径；不搜索 PATH，也不执行真实 ocx。
            let home = crate::infrastructure::platform::home_dir()
                .ok_or_else(|| {
                    Box::new(crate::errors::AppError::FileSystem {
                        operation: "resolve user home directory".to_string(),
                        detail: "no absolute user profile directory is available".to_string(),
                    }) as Box<dyn std::error::Error>
                })?;
            let discovery_paths = crate::infrastructure::discovery_paths::paths_for_home(&home);
            app.manage(crate::state::SharedHomeDir(home.clone()));
            // 运行来源解析层（FZ-48）：显式指定 > 托管安装 > 自动发现候选，不读 PATH。
            // 发现候选按文档口径给出（受控默认目录 + Homebrew 两个固定前缀），
            // 逐个做可执行校验，缺失即跳过。
            let runtime = crate::modules::runtime::RuntimeHandle::initialize(
                &data_root,
                crate::infrastructure::discovery_paths::runtime_candidates(&home),
            );
            let runtime_executable: crate::infrastructure::runtime_executable::SharedRuntimeExecutable =
                runtime.clone();
            app.manage(runtime.clone());
            // 托管安装 / 卸载的进行态（取消 + 同一时刻只允许一个写者）。
            let runtime_install = crate::state::SharedRuntimeInstall::new();
            app.manage(runtime_install.clone());
            crate::commands::runtime::reconcile_startup(data_root.clone(), runtime_install);
            #[cfg(windows)]
            app.manage(crate::commands::update::SharedPendingUpdate::new(None));
            app.manage(crate::commands::update_schedule::SharedSchedule::default());
            app.manage(crate::commands::update_schedule::SharedPanelQuery::default());
            let active_data_root = runtime_config.active_data_root.clone();
            let opencodex_home = crate::modules::data_root::resolve_opencodex_home(&runtime_config);
            let process_path = crate::infrastructure::discovery_paths::process_path(&home, &discovery_paths);
            let status_environment = crate::modules::process::EnvironmentPolicy {
                opencodex_home: opencodex_home.clone(),
                home: Some(home.clone().into_os_string()),
                path: Some(process_path.clone()),
                ..Default::default()
            };
            let status_source = crate::infrastructure::status_source::OfficialStatusSource::new(
                runtime_executable.clone(),
                home.clone(),
                status_environment.clone(),
            );
            let collector = std::sync::Arc::new(std::sync::Mutex::new(
                crate::modules::status::StatusCollector::new(status_source),
            ));
            app.manage(collector.clone());

            let cache_root = data_root.join("cache");
            let cli_enabled = cfg!(unix) && crate::modules::preferences::PreferencesStore::new(&data_root)
                .load()
                .map(|value| value.cli_enabled)
                .unwrap_or(false);
            if cli_enabled {
                let ipc_cache_root = cache_root.clone();
                let ipc_collector = collector.clone();
                let ipc_runner = std::sync::Arc::new(std::sync::Mutex::new(
                    crate::modules::process::ControlledProcessRunner::new(home.clone()),
                ));
                let ipc_dependencies = crate::modules::ipc::service::IpcDependencies {
                    process_context: crate::state::ProcessContext {
                        runtime: runtime_executable.clone(),
                        working_directory: home.clone(),
                        opencodex_home: opencodex_home.clone(),
                    },
                    runtime_log: crate::infrastructure::runtime_log::RuntimeLog::new(&data_root),
                    active_data_root: active_data_root.clone(),
                    app_data_root: data_root.clone(),
                    home: home.clone(),
                    opencodex_home: opencodex_home.clone(),
                    external_home: match runtime_config.opencodex_home_mode {
                        crate::modules::data_root::OpenCodexHomeMode::External => {
                            runtime_config.opencodex_home_path.clone()
                        }
                        _ => None,
                    },
                    current_version: app.package_info().version.to_string().leak(),
                };
                #[cfg(unix)]
                let ipc_app = app.handle().clone();
                #[cfg(unix)]
                tauri::async_runtime::spawn(async move {
                    let endpoint =
                        crate::modules::ipc::endpoint::IpcEndpoint::bind(&ipc_cache_root);
                    let Ok(endpoint) = endpoint else {
                        return;
                    };
                    let mut service = crate::modules::ipc::service::IpcService::new(
                        ipc_collector,
                        ipc_runner,
                        ipc_dependencies,
                    ).with_app_handle(ipc_app);
                    loop {
                        if endpoint.accept_once(&mut service).await.is_err() {
                            break;
                        }
                    }
                });
                // Windows 暂无本机 IPC 端点实现；显式消费这些值，避免未使用告警。
                #[cfg(not(unix))]
                let _ = (ipc_cache_root, ipc_collector, ipc_runner, ipc_dependencies);
            }

            let tray_menu = crate::infrastructure::tray_controller::TauriTrayController::build_menu(app.handle())?;
            // 「启动时打开主界面」（默认开启）：关闭时只启动代理托管，不显示主窗口。
            // 托盘与菜单仍可随时把窗口打开。
            if !launch_main(&data_root) {
                if let Some(window) = app.get_window("main") {
                    let _ = window.hide();
                    if let Some(activity) = window.app_handle().try_state::<crate::infrastructure::app_activity::AppActivity>() { activity.publish(window.app_handle(), false); }
                    // 启动就不显示窗口 = 只留托盘：此时不该占着 Dock 图标。
                    apply_dock_visibility(app.handle(), false);
                }
            }
            if let Some(tray) = app.tray_by_id(crate::modules::tray::TRAY_ID) {
                // Windows 托盘不会像 macOS 模板图标那样按主题反色，使用应用彩色图标。
                #[cfg(windows)]
                if let Some(icon) = app.default_window_icon() {
                    tray.set_icon(Some(icon.clone()))?;
                    tray.set_icon_as_template(false)?;
                }
                tray.set_menu(Some(tray_menu.clone()))?;
            }
            // Windows keeps one system caption row. Product actions stay in the
            // overview/sidebar/tray; WebView editing retains standard shortcuts.
            // macOS must retain Edit's responder-chain keyboard equivalents.
            #[cfg(windows)]
            let native_menu = None;
            #[cfg(not(windows))]
            let native_menu = {
                let menu = crate::infrastructure::tray_controller::build_app_menu(app.handle())?;
                app.set_menu(menu.clone())?;
                Some(menu)
            };
            // 托盘菜单与应用菜单是两个独立资源，控制器需要同时持有两者才能一起门控。
            let tray_controller = crate::infrastructure::tray_controller::TauriTrayController::new(
                app.handle().clone(),
                tray_menu,
                native_menu,
            );
            app.manage(std::sync::Arc::new(tray_controller) as crate::infrastructure::tray_controller::SharedTrayPresenter);

            // Doctor 只读来源与状态采集共用同一受控发现路径和冻结环境。
            let doctor_source = std::sync::Arc::new(std::sync::Mutex::new(
                crate::infrastructure::official_cli_source::OfficialDoctorSource::new(
                    runtime_executable.clone(),
                    home.clone(),
                    status_environment.clone(),
                ),
            ));
            app.manage(doctor_source as crate::state::SharedDoctorSource);

            // 关于页官方版本与 Doctor 复用受控发现路径，但不采集 stderr。
            let official_version_source = std::sync::Arc::new(std::sync::Mutex::new(
                crate::infrastructure::official_version_source::OfficialCliVersionSource::new(
                    runtime_executable.clone(),
                    home.clone(),
                    status_environment.clone(),
                ),
            ));
            app.manage(official_version_source as crate::state::SharedOfficialVersionSource);

            // 「随 Codex 启动 OpenCodex」开关的读写都经官方 shim（`ocx codex-shim`），
            // 管理器不旁路改写官方配置（AC-11）。
            let codex_shim_source = std::sync::Arc::new(std::sync::Mutex::new(
                crate::infrastructure::codex_shim_source::OfficialCodexShimSource::new(
                    runtime_executable.clone(),
                    home.clone(),
                    status_environment.clone(),
                ),
            ));
            app.manage(codex_shim_source as crate::state::SharedCodexShimSource);

            // 进程动作复用同一受控发现路径与启动数据根；不搜索 PATH。
            app.manage(std::sync::Mutex::new(
                crate::modules::process::ControlledProcessRunner::new(home.clone()),
            ));
            app.manage(crate::state::ProcessContext {
                runtime: runtime_executable,
                working_directory: home.clone(),
                opencodex_home,
            });

            app.manage(crate::infrastructure::app_activity::AppActivity::default());
            app.manage(crate::commands::panel::PanelLifetime::default());
            crate::infrastructure::app_activity::install(app.handle());
            crate::infrastructure::window_appearance::install(app.handle());
            // FZ-08 后台周期轮询；Tauri 事件只推送同一快照，前端不再自建定时器。
            let app_handle = app.handle().clone();
            let polling_collector = collector.clone();
            tauri::async_runtime::spawn(async move {
                struct TauriStatusEmitter(
                    tauri::AppHandle,
                    std::path::PathBuf,
                    // 上一次已观测到的运行状态：只有**真正变化**时才考虑发通知，
                    // 避免周期轮询把同一条关注态反复写回。
                    std::sync::Mutex<Option<crate::types::status::RuntimeState>>,
                );
                impl crate::modules::status::polling::StatusSnapshotEmitter for TauriStatusEmitter {
                    fn emit(&self, snapshot: &crate::types::runtime_status::StatusSnapshotDto) {
                        use tauri::Manager;
                        if let Some(tray) = self.0.try_state::<crate::infrastructure::tray_controller::SharedTrayPresenter>() {
                            let state = crate::infrastructure::tray_controller::tray_state_from_snapshot(snapshot, self.1.clone());
                            tray.update(&state);
                        }
                        if let Err(_error) = crate::commands::event_delivery::emit_signal(
                            &self.0,
                            crate::modules::status::polling::STATUS_SNAPSHOT_CHANGED_EVENT,
                            crate::modules::notifications::registry::Job::Observe,
                            crate::modules::notifications::registry::Trigger::StatusChange,
                            crate::modules::notifications::registry::Channel::Local,
                            snapshot,
                        ) {}
                        // 概览状态卡不再承载说明句（UI规范 §19）：需要关注的运行状态
                        // 由后端发布持久通知，稳定事实与瞬时进度不写。
                        let runtime = snapshot.matrix.runtime;
                        let changed = self
                            .2
                            .lock()
                            .map(|mut last| {
                                let changed = *last != Some(runtime);
                                *last = Some(runtime);
                                changed
                            })
                            .unwrap_or(false);
                        if !changed {
                            return;
                        }
                        let data_root = self.0.state::<crate::state::SharedDataRoot>();
                        let store = self.0.state::<crate::state::SharedNotificationStore>();
                        let enabled = crate::commands::lifecycle_notifications_enabled(&data_root.0);
                        // 发布器始终构造：「启停结果通知」偏好只约束启停结果类通知，
                        // 环境与配置所有权类通知（未发现 / 存在风险 / 外部接管）不受它影响，
                        // 是否跳过由 `publish_runtime_state_notification` 判定。
                        let publisher = crate::commands::notifications::NotificationPublisher {
                            store: store.inner(),
                            data_root: &data_root.0,
                        };
                        let runtime_log =
                            crate::infrastructure::runtime_log::RuntimeLog::new(&data_root.0);
                        let process_context = self.0.try_state::<crate::state::ProcessContext>();
                        let published = crate::commands::publish_runtime_state_notification_with_context(
                            Some(publisher),
                            runtime,
                            enabled,
                            &runtime_log,
                            process_context.as_deref(),
                        );
                        // 后端写入后必须广播，否则前端仍显示启动时那份旧列表。
                        if published {
                            if let Err(_error) = crate::commands::event_delivery::emit_signal(
                                &self.0,
                                crate::commands::notifications::NOTIFICATIONS_CHANGED_EVENT,
                                crate::modules::notifications::registry::Job::NotificationMutation,
                                crate::modules::notifications::registry::Trigger::Commit,
                                crate::modules::notifications::registry::Channel::Local,
                                (),
                            ) {}
                        }
                    }
                }

                struct TauriBackgroundState(tauri::AppHandle);
                impl crate::modules::status::polling::BackgroundStateProvider for TauriBackgroundState {
                    fn backgrounded(&self) -> bool {
                        use tauri::Manager;
                        !self.0.state::<crate::infrastructure::app_activity::AppActivity>().foreground()
                    }
                }

                let emitter = TauriStatusEmitter(
                    app_handle.clone(),
                    active_data_root.clone(),
                    std::sync::Mutex::new(None),
                );
                let background = TauriBackgroundState(app_handle.clone());
                let mut service = crate::modules::status::polling::StatusPollingService::new(
                    polling_collector,
                    emitter,
                    background,
                );
                loop {
                    service.poll_once().await;
                    let wait = service.next_wait();
                    let activity = app_handle.state::<crate::infrastructure::app_activity::AppActivity>();
                    tokio::select! {
                        _ = tokio::time::sleep(wait) => {},
                        _ = activity.wake.notified() => { service.invalidate_schedule(); }
                    }
                }
            });
            Ok(())
        })
        .on_menu_event(|app, event| {
            use crate::modules::tray::*;
            let action = crate::infrastructure::tray_controller::native_menu_action(event.id().as_ref())
                .or_else(|| match event.id().as_ref() {
                TRAY_MENU_START => Some(TrayAction::Start),
                TRAY_MENU_STOP => Some(TrayAction::Stop),
                TRAY_MENU_RESTART => Some(TrayAction::Restart),
                TRAY_MENU_OPEN_MAIN => Some(TrayAction::OpenMain),
                TRAY_MENU_OPEN_PANEL => Some(TrayAction::OpenPanel),
                TRAY_MENU_OPEN_LOGS => Some(TrayAction::OpenLogs),
                TRAY_MENU_OPEN_DATA_DIR => Some(TrayAction::OpenDataDir),
                TRAY_MENU_DOCTOR => Some(TrayAction::RunDoctor),
                TRAY_MENU_OPEN_SETTINGS => Some(TrayAction::OpenSettings),
                TRAY_MENU_QUIT => Some(TrayAction::Quit),
                _ => None,
            });
            let Some(action) = action else {
                return;
            };
            match action {
                TrayAction::Quit => app.exit(0),
                TrayAction::OpenMain
                | TrayAction::OpenPanel
                | TrayAction::OpenLogs
                | TrayAction::OpenDataDir
                | TrayAction::RunDoctor
                | TrayAction::OpenSettings => {
                    // 统一重开路径并记录入口与结果（F-03）；窗口回来后 Dock 图标也恢复。
                    let data_root = app.state::<crate::state::SharedDataRoot>();
                    reveal_main_window(app, "tray/menu", &data_root.0);
                    app.state::<crate::state::SharedTrayRequests>().push(action);
                    { let _ = crate::commands::event_delivery::emit_signal(app, "tray-requests-available", crate::modules::notifications::registry::Job::Tray, crate::modules::notifications::registry::Trigger::NativeCallback, crate::modules::notifications::registry::Channel::Local, ()); }
                }
                // 原生「重载主界面」（F-06）：直接重载主 WebView，不入前端请求队列。
                TrayAction::ReloadMain => {
                    let data_root = app.state::<crate::state::SharedDataRoot>();
                    reveal_main_window(app, "app-menu:reload", &data_root.0);
                    reload_main_webview(app, &data_root.0);
                }
                TrayAction::Start | TrayAction::Stop | TrayAction::Restart => {
                    app.state::<crate::state::SharedTrayRequests>().push(action);
                    { let _ = crate::commands::event_delivery::emit_signal(app, "tray-requests-available", crate::modules::notifications::registry::Job::Tray, crate::modules::notifications::registry::Trigger::NativeCallback, crate::modules::notifications::registry::Channel::Local, ()); }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::backup::backup_files,
            commands::backup::open_backup_file,
            commands::backup::create_preferences_backup,
            commands::backup::preferences_backup_policy,
            commands::backup::save_preferences_backup_policy,
            commands::backup::list_preferences_backups,
            commands::backup::set_preferences_backup_pinned,
            commands::backup::preview_preferences_backup_cleanup,
            commands::backup::cleanup_preferences_backups,
            commands::backup::restore_preferences_backup,
            commands::update::check_for_update,
            commands::update::install_update,
            commands::update::restart_after_update,
            commands::update::get_update_status,
            commands::update::set_update_channel,
            commands::update_schedule::update_schedule_plan,
            commands::update_schedule::official_remote_cache,
            commands::upgrade::create_upgrade_backup,
            commands::upgrade::create_restore_backup,
            commands::upgrade::restore_risk_summary,
            commands::about::app_about,
            commands::window::window_appearance,
            commands::about::official_project_facts,
            commands::about::official_remote_latest,
            commands::app_status,
            commands::discovery::discover_environment,
            commands::doctor::run_doctor,
            commands::codex_shim::codex_shim_status,
            commands::codex_shim::set_codex_shim,
            commands::data_root::get_data_root_config,
            commands::data_root::initialize_data_root,
            commands::data_root::set_opencodex_home_config,
            commands::data_root::switch_data_root,
            commands::data_root::validate_data_root_structure,
            commands::extensions::extension_config,
            commands::extensions::list_extensions,
            commands::extensions::read_skill_detail,
            commands::extensions::read_mcp_detail,
            commands::extensions::set_extension_client_enabled,
            commands::extensions::execute_extension_write,
            commands::skills::import_skill_archive,
            commands::get_status_snapshot,
            commands::restore_guidance,
            commands::logs::read_logs,
            commands::notifications::list_notifications,
            commands::notifications::mark_notification_read,
            commands::notifications::mark_all_notifications_read,
            commands::notifications::delete_notification,
            commands::notifications::mark_notification_resolved,
            commands::notifications::clear_notifications,
            commands::notifications::clear_read_notifications,
            commands::notifications::clear_resolved_notifications,
            commands::cleanup::cleanup_local_logs,
            commands::cleanup::cleanup_local_notifications,
            commands::preferences::get_preferences,
            commands::preferences::save_preferences,
            commands::preferences::restore_default_preferences,
            commands::network::check_network_proxy,
            commands::sync::get_sync_config,
            commands::sync::save_sync_endpoint,
            commands::sync::delete_sync_endpoint,
            commands::sync::test_sync_connection,
            commands::sync::get_sync_status,
            commands::sync::run_sync_now,
            commands::workspace::get_managed_path_targets,
            commands::workspace::open_managed_path,
            commands::workspace::open_external_link,
            commands::workspace::open_local_document,
            commands::migration::export_migration,
            commands::migration::import_migration,
            crate::infrastructure::app_activity::app_foreground,
            commands::drain_tray_requests,
            commands::runtime::runtime_source,
            commands::runtime::runtime_protection_status,
            commands::runtime::retry_runtime_protection,
            commands::runtime::set_runtime_source,
            commands::runtime::restore_discovered_runtime,
            commands::runtime::preview_offline_package,
            commands::runtime::install_runtime,
            commands::runtime::cancel_runtime_install,
            commands::runtime::install_official_update,
            commands::runtime::plan_runtime_uninstall,
            commands::runtime::uninstall_runtime,
            commands::runtime::official_uninstall_observation,
            commands::panel::sync_embedded_panel,
            commands::process_action,
            commands::tray::tray_state
        ])
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let Some(activity) = window.app_handle().try_state::<crate::infrastructure::app_activity::AppActivity>() {
                    match event {
                        tauri::WindowEvent::Focused(focused) => activity.publish(window.app_handle(), *focused),
                        tauri::WindowEvent::Resized(_) => activity.publish(window.app_handle(), window.is_focused().unwrap_or(false)),
                        _ => {},
                    }
                }
            }
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() != "main" {
                    return;
                }
                // 「关闭窗口后保持代理运行」为默认开启：关闭窗口应只隐藏桌面壳，
                // 托盘与托管代理继续存活。此前这里没有任何处理，点关闭按钮会直接
                // 退出整个应用（托盘图标一并消失），与设置里的说明不符。
                if close_action(keep_proxy_on_close(window.app_handle())) == CloseAction::Hide {
                    api.prevent_close();
                    let _ = window.hide();
                    if let Some(activity) = window.app_handle().try_state::<crate::infrastructure::app_activity::AppActivity>() { activity.publish(window.app_handle(), false); }
                    // 只隐藏窗口：Dock 图标随之消失（托盘仍在），再打开窗口时恢复。
                    apply_dock_visibility(window.app_handle(), false);
                }
            }
        })
        .build(context)
        .expect("failed to build OpenCodeX Desktop")
        .run(|app_handle, event| {
            // 关闭窗口只是隐藏（见上）。此时点击 Dock 图标必须能把窗口找回来，
            // 否则用户会以为应用「卡住」了——macOS 用 Reopen 事件表达这一意图。
            #[cfg(target_os = "macos")]
            {
                if let tauri::RunEvent::Reopen {
                    has_visible_windows,
                    ..
                } = event
                {
                    if !has_visible_windows {
                        let data_root = app_handle.state::<crate::state::SharedDataRoot>();
                        reveal_main_window(app_handle, "dock:reopen", &data_root.0);
                    }
                    }
            }
            #[cfg(not(target_os = "macos"))]
            let _ = (&app_handle, &event);
        });
}

/// 窗口是否可见 → 该不该占 Dock 图标。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockVisibility {
    /// 有窗口：`Regular`，显示 Dock 图标与菜单栏。
    Regular,
    /// 窗口已隐藏、只留托盘：`Accessory`，Dock 图标与菜单栏消失，托盘不受影响。
    Accessory,
}

/// 纯函数便于断言：只有「有可见窗口」才保留 Dock 图标。
pub fn dock_visibility(window_visible: bool) -> DockVisibility {
    if window_visible {
        DockVisibility::Regular
    } else {
        DockVisibility::Accessory
    }
}

/// 按窗口可见性切换 macOS 激活策略；非 macOS 平台是空实现（由托盘与窗口本身表达状态）。
#[cfg(target_os = "macos")]
fn apply_dock_visibility<R: tauri::Runtime>(app: &tauri::AppHandle<R>, window_visible: bool) {
    let policy = match dock_visibility(window_visible) {
        DockVisibility::Regular => tauri::ActivationPolicy::Regular,
        DockVisibility::Accessory => tauri::ActivationPolicy::Accessory,
    };
    let _ = app.set_activation_policy(policy);
}

#[cfg(not(target_os = "macos"))]
fn apply_dock_visibility<R: tauri::Runtime>(_app: &tauri::AppHandle<R>, _window_visible: bool) {}

/// 关闭主窗口时的处置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseAction {
    /// 只隐藏窗口；桌面壳与托盘继续存活，代理不受影响。
    Hide,
    /// 允许窗口关闭，桌面壳随之退出。
    Exit,
}

/// 依据「关闭窗口后保持代理运行」偏好决定关闭行为。
///
/// 读不到偏好时按默认（保持运行）处理：不因为一次读取失败就把用户的应用悄悄退掉。
pub fn close_action(keep_proxy_on_close: Option<bool>) -> CloseAction {
    match keep_proxy_on_close {
        Some(false) => CloseAction::Exit,
        _ => CloseAction::Hide,
    }
}

fn keep_proxy_on_close<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Option<bool> {
    use tauri::Manager;
    let root = app.try_state::<crate::state::SharedDataRoot>()?;
    crate::modules::preferences::PreferencesStore::new(&root.0)
        .load()
        .ok()
        .map(|preferences| preferences.keep_proxy_on_close)
}

/// 「启动时打开主界面」；读不到偏好时按默认（打开）处理。
fn launch_main(data_root: &std::path::Path) -> bool {
    crate::modules::preferences::PreferencesStore::new(data_root)
        .load()
        .map(|preferences| preferences.launch_main)
        .unwrap_or(true)
}

/// 更新通道（U-07）：偏好里已是解耦后的稳定/测试通道，检查/安装/调度共用同一解析。
fn update_channel(data_root: &std::path::Path) -> crate::modules::update::UpdateChannel {
    let value = crate::modules::preferences::PreferencesStore::new(data_root)
        .load()
        .map(|preferences| preferences.app_update_channel)
        .unwrap_or_else(|_| crate::modules::preferences::APP_UPDATE_CHANNEL_STABLE.to_string());
    if value == crate::modules::preferences::APP_UPDATE_CHANNEL_BETA {
        crate::modules::update::UpdateChannel::Beta
    } else {
        crate::modules::update::UpdateChannel::Stable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::preferences::{Preferences, PreferencesStore};
    use crate::modules::update::UpdateChannel;

    fn seeded(preferences: Preferences) -> tempfile::TempDir {
        let root = tempfile::tempdir().expect("temp root");
        crate::modules::data_root::initialize(root.path()).expect("initialize");
        PreferencesStore::new(root.path())
            .save(&preferences)
            .expect("save preferences");
        root
    }

    // 回归：解耦后的更新通道在启动时真正决定更新状态里的通道。
    #[test]
    fn update_channel_preference_seeds_update_status_channel() {
        let beta = seeded(Preferences {
            app_update_channel: "beta".to_string(),
            ..Default::default()
        });
        assert_eq!(update_channel(beta.path()), UpdateChannel::Beta);
        let stable = seeded(Preferences {
            app_update_channel: "stable".to_string(),
            ..Default::default()
        });
        assert_eq!(update_channel(stable.path()), UpdateChannel::Stable);
    }

    // 回归：旧复合枚举（stable-24h / beta-6h / manual）加载时迁移到通道/自动检查/间隔。
    // 直接写旧格式文件（保存路径会先校验，无法写回非法旧值）。
    #[test]
    fn legacy_update_channel_enum_migrates_on_load() {
        for (legacy, channel, auto, interval) in [
            ("beta-6h", "beta", true, 21600),
            ("manual", "stable", false, 86400),
        ] {
            let root = tempfile::tempdir().expect("temp root");
            crate::modules::data_root::initialize(root.path()).expect("initialize");
            let path = crate::modules::preferences::PreferencesStore::new(root.path()).path();
            std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
            std::fs::write(
                &path,
                format!(
                    r#"{{"interface_scale":120,"launch_main":true,"auto_panel":true,"panel_mode":"embedded","keep_proxy_on_close":true,"lifecycle_notifications":true,"sync_conflict_alerts":true,"launch_with_codex":true,"auto_backup_upgrade":true,"auto_backup_import":true,"auto_backup_sync":true,"backup_retention":"10","backup_integrity":"sha-256","backup_include_skills":true,"export_include_skills":true,"mcp_conflict_policy":"ask","mcp_mask":true,"backup_include_mcp":true,"export_include_mcp":true,"log_retention":"30d-10000","notification_retention":"30","startup_cleanup":true,"cleanup_backup_summary":true,"cli_enabled":false,"sync_conflict_policy":"ask","cold_sync":true,"backup_before_overwrite":true,"app_update_channel":"{legacy}"}}"#
                ),
            )
            .expect("write legacy fixture");

            let loaded = PreferencesStore::new(root.path()).load().expect("reload");
            assert_eq!(loaded.app_update_channel, channel);
            assert_eq!(loaded.app_update_auto_check, auto);
            assert_eq!(loaded.app_update_check_interval_seconds, interval);
        }
    }

    // 回归：「启动时打开主界面」在启动阶段被真实读取。
    #[test]
    fn launch_main_preference_is_consumed() {
        let shown = seeded(Preferences {
            launch_main: true,
            ..Default::default()
        });
        assert!(launch_main(shown.path()));
        let hidden = seeded(Preferences {
            launch_main: false,
            ..Default::default()
        });
        assert!(!launch_main(hidden.path()));
    }
}
