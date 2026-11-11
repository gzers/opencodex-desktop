//! Product removal of admitted runtime images, preserving configuration/backups.
//! One-use in-memory review grants no arbitrary path/file access through IPC.
use super::runtime_recovery_v2_owner::SystemServiceOwnerProofProvider;
use crate::{
    errors::{AppError, AppResult},
    modules::runtime::{
        program_removal::{self, Outcome, Prepared, Preview},
        terminal_cli::{self, Context},
        RuntimeHandle,
    },
    state::{SharedDataRoot, SharedHomeDir, SharedProcessContext, SharedRuntimeInstall},
};
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tauri::Manager;
fn fail(text: &'static str) -> AppError {
    AppError::RuntimeManaged {
        code: "runtime_removal_review_refused".into(),
        detail: text.into(),
    }
}
#[derive(Default)]
pub struct Reviews(Mutex<Option<Ticket>>);
struct Ticket {
    id: String,
    selected_at: Instant,
    context: Context,
    shell: String,
    search: Vec<std::path::PathBuf>,
    prepared: Prepared,
    resume: bool,
    terminal: Option<terminal_cli::Preview>,
    confirmation: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    pub id: String,
    pub removal: Preview,
    pub resume: bool,
    pub command_path: Option<std::path::PathBuf>,
    pub shell_paths: Vec<std::path::PathBuf>,
    pub confirmation_digest: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub id: String,
    pub confirmation_digest: String,
}
fn prepare(
    context: Context,
    shell: String,
    search: Vec<std::path::PathBuf>,
    reviews: &Reviews,
) -> AppResult<Option<Review>> {
    if !cfg!(target_os = "macos") {
        return Err(fail("当前平台程序卸载的完整进程及终端撤销尚未接入"));
    }
    let (prepared, resume) = match program_removal::inspect_pending(
        &context.program_root,
        &context.desktop_root,
        &context.opencodex_home,
    )? {
        Some(original) => (original, true),
        None if program_removal::intentionally_removed(
            &context.program_root,
            &context.desktop_root,
        )? =>
        {
            return Ok(None)
        }
        None => (
            program_removal::prepare_registered(
                &context.program_root,
                &context.desktop_root,
                &context.opencodex_home,
            )?,
            false,
        ),
    };
    let removal = prepared.preview()?;
    let terminal = terminal_cli::removal_preview(&context, &shell, &search)?;
    let confirmation = crate::infrastructure::hash::sha256_hex(
        &serde_json::to_vec(&(&removal, &context, &terminal, resume))
            .map_err(|_| fail("卸载预览无法核验"))?,
    );
    let id = uuid::Uuid::new_v4().to_string();
    let response = Review {
        id: id.clone(),
        removal,
        resume,
        command_path: terminal.as_ref().map(|p| p.command_path.clone()),
        shell_paths: terminal
            .as_ref()
            .map(|p| p.profile_paths.clone())
            .unwrap_or_default(),
        confirmation_digest: confirmation.clone(),
    };
    *reviews.0.lock().map_err(|_| fail("卸载预览锁不可用"))? = Some(Ticket {
        id,
        selected_at: Instant::now(),
        context,
        shell,
        search,
        prepared,
        resume,
        terminal,
        confirmation,
    });
    Ok(Some(response))
}
fn consume(reviews: &Reviews, request: &Request, context: &Context) -> AppResult<Ticket> {
    let mut slot = reviews.0.lock().map_err(|_| fail("卸载预览锁不可用"))?;
    if slot.as_ref().is_none_or(|t| {
        t.id != request.id
            || t.confirmation != request.confirmation_digest
            || t.context != *context
            || t.selected_at.elapsed() > Duration::from_secs(600)
    }) {
        return Err(fail("卸载预览已过期或配置位置变化，请重新核对"));
    }
    Ok(slot.take().unwrap())
}
fn execute(ticket: Ticket, unused: impl Fn() -> AppResult<()>) -> AppResult<Outcome> {
    let c = &ticket.context;
    let original = ticket.prepared.preview()?;
    // Bind terminal bytes before any first-step change, including continuing a
    // previous half-finished removal. Never recompute a new permission on apply.
    let current_terminal = terminal_cli::removal_preview(c, &ticket.shell, &ticket.search)?;
    if current_terminal.as_ref().map(|p| &p.confirmation_digest)
        != ticket.terminal.as_ref().map(|p| &p.confirmation_digest)
    {
        return Err(fail("终端入口在卸载预览后变化，请重新核对"));
    }
    let revoke = || {
        terminal_cli::revoke_for_removal(
            c,
            &ticket.shell,
            &ticket.search,
            ticket
                .terminal
                .as_ref()
                .map(|p| p.confirmation_digest.as_str()),
        )
    };
    if ticket.resume {
        program_removal::resume(
            &c.program_root,
            &c.desktop_root,
            &c.opencodex_home,
            &original.confirmation_digest,
            unused,
            revoke,
        )
    } else {
        ticket
            .prepared
            .execute(&original.confirmation_digest, unused, revoke)
    }
}
#[tauri::command]
pub async fn runtime_removal_preview(
    runtime: tauri::State<'_, Arc<RuntimeHandle>>,
    data: tauri::State<'_, SharedDataRoot>,
    profile: tauri::State<'_, SharedHomeDir>,
    context: tauri::State<'_, SharedProcessContext>,
    reviews: tauri::State<'_, Arc<Reviews>>,
) -> AppResult<Option<Review>> {
    let runtime = runtime.inner().clone();
    let data = data.0.clone();
    let profile = profile.0.clone();
    let home = context.opencodex_home.clone();
    let reviews = reviews.inner().clone();
    super::run_readonly("preview versioned runtime removal", move || {
        let (c, shell, search) = super::terminal_cli::input(&runtime, data, profile, home)?;
        prepare(c, shell, search, &reviews)
    })
    .await
}
#[tauri::command]
pub async fn runtime_removal_apply(
    request: Request,
    app: tauri::AppHandle,
    runtime: tauri::State<'_, Arc<RuntimeHandle>>,
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
    reviews: tauri::State<'_, Arc<Reviews>>,
    install: tauri::State<'_, SharedRuntimeInstall>,
) -> AppResult<Outcome> {
    let lease = {
        let manager = app.state::<super::update::SharedUpdateStatus>();
        let state = manager.lock().map_err(|_| fail("无法核对管理器更新状态"))?;
        if state.installing || state.pending_restart.is_some() {
            return Err(fail("管理器更新尚未结束，请完成后再卸载运行程序"));
        }
        install
            .acquire()
            .ok_or_else(|| fail("已有安装、卸载或恢复正在进行"))?
    };
    let runtime = runtime.inner().clone();
    let data = data.0.clone();
    let profile = app.state::<SharedHomeDir>().0.clone();
    let context = context.inner().clone();
    let reviews = reviews.inner().clone();
    super::run_blocking("remove reviewed versioned runtime", move || {
        let _lease = lease;
        let (c, _, _) = super::terminal_cli::input(
            &runtime,
            data.clone(),
            profile,
            context.opencodex_home.clone(),
        )?;
        let ticket = consume(&reviews, &request, &c)?;
        crate::modules::runtime::protection::ensure_binding_ready(&data)?;
        let owner = SystemServiceOwnerProofProvider::new(
            runtime.installation_root().into(),
            context.clone(),
        )?;
        // Stops use the official lifecycle from the app's ordinary run control.
        // Removal never invents a signal/force-kill fallback for uncertain state.
        let result = execute(ticket, || owner.require_program_unused(&context));
        runtime.observe_current();
        if result.is_ok() {
            super::runtime::broadcast_source_changed(&app);
        }
        result
    })
    .await
}
#[cfg(all(test, target_os = "macos", target_arch = "aarch64"))]
#[path = "runtime_removal_tests.rs"]
mod tests;
