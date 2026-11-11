//! Explicit completion/rollback of the current versioned installation journal.
//! IPC accepts only the reviewed operation identity and decision, never paths.
use super::{
    runtime_recovery_v2::{ServiceOwnerProof, ServiceOwnerProofProvider},
    runtime_recovery_v2_owner::SystemServiceOwnerProofProvider,
};
use crate::{
    errors::{AppError, AppResult},
    modules::{
        model_config::config_file::v2::native_import::backups,
        runtime::{
            program_activation::{self, Decision, PendingSummary},
            RuntimeHandle,
        },
    },
    state::{SharedDataRoot, SharedProcessContext, SharedRuntimeInstall},
};
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc};
use tauri::Manager;

fn fail(detail: &'static str) -> AppError {
    AppError::RuntimeManaged {
        code: "runtime_activation_recovery_refused".into(),
        detail: detail.into(),
    }
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryPreview {
    pub attempt_id: String,
    pub from: Option<String>,
    pub to: String,
    pub program_root: String,
    pub config_path: String,
    pub confirmation_digest: String,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecoveryRequest {
    pub attempt_id: String,
    pub confirmation_digest: String,
    pub decision: Decision,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryOutcome {
    pub attempt_id: String,
    pub decision: Decision,
    pub selected_version: Option<String>,
}
fn preview_value(
    program: &Path,
    data: &Path,
    home: &Path,
    summary: PendingSummary,
) -> AppResult<RecoveryPreview> {
    crate::modules::runtime::bundled::roots(program, home)?;
    let digest = crate::infrastructure::hash::sha256_hex(
        &serde_json::to_vec(&(program, data, home, &summary.confirmation_digest))
            .map_err(|_| fail("无法核对当前恢复上下文"))?,
    );
    Ok(RecoveryPreview {
        attempt_id: summary.attempt_id,
        from: summary.from,
        to: summary.to,
        program_root: program.to_string_lossy().into_owned(),
        config_path: home.join("config.json").to_string_lossy().into_owned(),
        confirmation_digest: digest,
    })
}
fn inspect(program: &Path, data: &Path, home: &Path) -> AppResult<Option<RecoveryPreview>> {
    program_activation::inspect_pending(program, data)?
        .map(|s| preview_value(program, data, home, s))
        .transpose()
}
fn apply(
    program: &Path,
    data: &Path,
    home: &Path,
    request: RecoveryRequest,
    require_stopped: impl Fn() -> AppResult<()>,
) -> AppResult<RecoveryOutcome> {
    let mutation = program_activation::Mutation::acquire(program)?;
    let summary = mutation.pending_summary(data)?;
    let preview = preview_value(program, data, home, summary.clone())?;
    if preview.attempt_id != request.attempt_id
        || preview.confirmation_digest != request.confirmation_digest
    {
        return Err(fail("安装恢复预览已变化，请重新核对原操作"));
    }
    require_stopped()?;
    let protected = backups::protect_upgrade(
        data,
        &backups::UpgradeRequest {
            home: home.into(),
            program_source: None,
        },
    )
    .map_err(|_| fail("当前配置的保护备份未能核验，安装恢复未执行"))?;
    let result = mutation.recover(
        data,
        &summary.attempt_id,
        &summary.confirmation_digest,
        request.decision,
        || {
            require_stopped()?;
            protected
                .verify()
                .map_err(|_| fail("配置或保护备份发生变化，安装恢复结果待核对"))
        },
    )?;
    let expected = if request.decision == Decision::Complete {
        Some(summary.to)
    } else {
        summary.from
    };
    if result.as_ref().map(|p| &p.selection.version) != expected.as_ref() {
        return Err(fail("恢复后版本读回不符，未确认完成"));
    }
    Ok(RecoveryOutcome {
        attempt_id: request.attempt_id,
        decision: request.decision,
        selected_version: expected,
    })
}

#[tauri::command]
pub async fn runtime_activation_preview(
    runtime: tauri::State<'_, Arc<RuntimeHandle>>,
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
) -> AppResult<Option<RecoveryPreview>> {
    if !runtime.uses_versioned_programs() {
        return Err(fail("当前运行来源不是新版受管版本目录"));
    }
    let program = runtime.installation_root().to_path_buf();
    let data = data.0.clone();
    let home = context.opencodex_home.clone();
    super::run_readonly("inspect pending runtime activation", move || {
        inspect(&program, &data, &home)
    })
    .await
}

#[tauri::command]
pub async fn runtime_activation_recover(
    request: RecoveryRequest,
    app: tauri::AppHandle,
    runtime: tauri::State<'_, Arc<RuntimeHandle>>,
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
    install: tauri::State<'_, SharedRuntimeInstall>,
) -> AppResult<RecoveryOutcome> {
    if !runtime.uses_versioned_programs() {
        return Err(fail("当前运行来源不是新版受管版本目录"));
    }
    let lease = {
        let manager = app.state::<crate::commands::update::SharedUpdateStatus>();
        let state = manager.lock().map_err(|_| fail("无法核对管理器更新状态"))?;
        if state.installing || state.pending_restart.is_some() {
            return Err(fail("管理器更新尚未结束，请完成后再恢复安装"));
        }
        install
            .acquire()
            .ok_or_else(|| fail("已有运行程序操作正在进行，请稍后核对"))?
    };
    let runtime = runtime.inner().clone();
    let data = data.0.clone();
    let context = context.inner().clone();
    super::run_blocking("recover pending runtime activation", move || {
        let _lease = lease;
        crate::modules::runtime::protection::ensure_binding_ready(&data)?;
        let owner = SystemServiceOwnerProofProvider::new(
            runtime.installation_root().into(),
            context.clone(),
        )?;
        let outcome = apply(
            runtime.installation_root(),
            &data,
            &context.opencodex_home,
            request,
            || {
                if owner.query(&context)? == ServiceOwnerProof::Stopped {
                    Ok(())
                } else {
                    Err(fail("请先停止当前OpenCodex并确认服务已退出，再恢复安装"))
                }
            },
        );
        runtime.observe_current();
        if outcome.is_ok() {
            super::runtime::broadcast_source_changed(&app);
        }
        outcome
    })
    .await
}

#[cfg(all(test, unix))]
#[path = "runtime_activation_tests.rs"]
mod tests;
