use crate::{
    errors::{AppError, AppResult},
    modules::runtime::{
        terminal_cli::{self, Action, Context, Outcome, Preview},
        RuntimeHandle,
    },
    state::{SharedDataRoot, SharedHomeDir, SharedProcessContext, SharedRuntimeInstall},
};
use serde::Deserialize;
use std::{path::PathBuf, sync::Arc};
use tauri::Manager;

fn fail(message: &'static str) -> AppError {
    AppError::RuntimeManaged {
        code: "terminal_cli_refused".into(),
        detail: message.into(),
    }
}
pub(crate) fn input(
    runtime: &RuntimeHandle,
    data: PathBuf,
    profile: PathBuf,
    home: PathBuf,
) -> AppResult<(Context, String, Vec<PathBuf>)> {
    if !cfg!(target_os = "macos") {
        return Err(fail("当前平台终端注册尚未接入"));
    }
    if !runtime.uses_versioned_programs() {
        return Err(fail("终端命令需要新版受管运行程序"));
    }
    // CLI observes exactly the persisted active binding; a pending GUI restart
    // cannot silently bind a terminal to a different D/H.
    let selected = crate::modules::data_root::startup_v2::selected_config(
        &profile.join(".codex-desktop/desktop"),
    )?;
    if !crate::modules::data_root::runtime_binding_matches(&selected, &data, &home) {
        return Err(fail("配置位置待重启生效，请重启后设置终端入口"));
    }
    if std::env::var_os("ZDOTDIR").is_some_and(|path| !path.is_empty() && path != profile) {
        return Err(fail("当前ZDOTDIR与用户目录不同，请核对Shell配置后再启用"));
    }
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());
    let mut search = std::env::var_os("PATH")
        .as_deref()
        .map(std::env::split_paths)
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    for path in ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin", "/bin"] {
        let path = PathBuf::from(path);
        if !search.contains(&path) {
            search.push(path);
        }
    }
    Ok((
        Context {
            user_profile: profile,
            desktop_root: data,
            opencodex_home: home,
            program_root: runtime.installation_root().into(),
            manager_executable: std::env::current_exe()
                .map_err(|_| fail("当前应用入口不可确认"))?,
        },
        shell,
        search,
    ))
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub action: Action,
    pub confirmation_digest: String,
}
#[tauri::command]
pub async fn runtime_terminal_preview(
    runtime: tauri::State<'_, Arc<RuntimeHandle>>,
    data: tauri::State<'_, SharedDataRoot>,
    profile: tauri::State<'_, SharedHomeDir>,
    context: tauri::State<'_, SharedProcessContext>,
) -> AppResult<Preview> {
    let runtime = runtime.inner().clone();
    let data = data.0.clone();
    let profile = profile.0.clone();
    let home = context.opencodex_home.clone();
    super::run_readonly("inspect official terminal entry", move || {
        let (c, shell, search) = input(&runtime, data, profile, home)?;
        terminal_cli::inspect(&c, &shell, &search)
    })
    .await
}
#[tauri::command]
pub async fn runtime_terminal_apply(
    request: Request,
    app: tauri::AppHandle,
    runtime: tauri::State<'_, Arc<RuntimeHandle>>,
    data: tauri::State<'_, SharedDataRoot>,
    profile: tauri::State<'_, SharedHomeDir>,
    context: tauri::State<'_, SharedProcessContext>,
    install: tauri::State<'_, SharedRuntimeInstall>,
) -> AppResult<Outcome> {
    let lease = {
        let manager = app.state::<crate::commands::update::SharedUpdateStatus>();
        let state = manager.lock().map_err(|_| fail("无法核对管理器更新状态"))?;
        if state.installing || state.pending_restart.is_some() {
            return Err(fail("管理器更新尚未结束"));
        }
        install
            .acquire()
            .ok_or_else(|| fail("已有运行程序操作进行中"))?
    };
    let runtime = runtime.inner().clone();
    let data = data.0.clone();
    let profile = profile.0.clone();
    let home = context.opencodex_home.clone();
    super::run_blocking("change official terminal entry", move || {
        let _lease = lease;
        let (c, shell, search) = input(&runtime, data, profile, home)?;
        crate::modules::runtime::protection::ensure_binding_ready(&c.desktop_root)?;
        terminal_cli::apply(
            &c,
            request.action,
            &shell,
            &search,
            &request.confirmation_digest,
        )
    })
    .await
}
