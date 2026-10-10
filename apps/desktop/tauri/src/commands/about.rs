//! 关于页命令：只做 DTO 投影与受控来源编排，不访问文件或进程细节。

use std::sync::{Arc, Mutex};

use crate::errors::{AppError, AppResult};
use crate::modules::about::{OfficialProjectFacts, OfficialVersionSource};
use crate::state::SharedOfficialVersionSource;
use crate::types::about::{AboutAppDto, OfficialProjectDto};

#[tauri::command]
pub fn app_about(app: tauri::AppHandle) -> AppResult<AboutAppDto> {
    Ok(AboutAppDto {
        name: app.package_info().name.clone(),
        version: app.package_info().version.to_string(),
        identifier: app.config().identifier.clone(),
        platform: crate::infrastructure::platform::platform_label().into(),
        framework: "Tauri v2".into(),
        license: "MIT License".into(),
    })
}

#[tauri::command]
pub async fn official_project_facts(
    source: tauri::State<'_, SharedOfficialVersionSource>,
    runtime: tauri::State<'_, std::sync::Arc<crate::modules::runtime::RuntimeHandle>>,
) -> AppResult<OfficialProjectDto> {
    let source = source.inner().clone();
    let runtime = runtime.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let facts = official_project_facts_with_source(&source)?;
        // `FZ-48` / B1 口径：版本检查读成功后把版本回填到运行来源记录，
        // 这样 `runtime.json` 与卡片的「版本」对显式 / 发现来源也有事实可依。
        // 这里**不广播**：广播由前端在来源变化时触发的「读版本 → 重读来源」链路负责，
        // 避免 `runtime-source-changed` 与版本检查互相触发。
        if let Some(version) = facts.version.as_deref() {
            runtime.record_resolved_version(version);
        }
        Ok(facts)
    })
    .await
    .map_err(|_| AppError::NotConfigured)?
}

pub fn official_project_facts_with_source<S: OfficialVersionSource + ?Sized>(
    source: &Arc<Mutex<S>>,
) -> AppResult<OfficialProjectDto> {
    let guard = source.lock().map_err(|_poisoned| AppError::NotConfigured)?;
    Ok(OfficialProjectFacts::run(&*guard)?.into())
}

/// 远端元数据查询：并发调用复用结果，落盘缓存与退避，不安装。
#[tauri::command]
pub async fn official_remote_latest(
    trigger: Option<crate::commands::event_delivery::QueryTrigger>,
    app: tauri::AppHandle,
) -> AppResult<crate::modules::about::remote::OfficialRemoteLatest> {
    use crate::commands::update_schedule::{self, SharedPanelQuery};
    use crate::modules::update::schedule::Target;
    use std::sync::atomic::Ordering;
    use tauri::Manager;
    let gate = app.state::<SharedPanelQuery>();
    let sequence = gate.1.load(Ordering::Acquire);
    let mut query = gate.0.lock().await;
    if query.sequence != sequence {
        return query.value.clone().ok_or(AppError::NotConfigured);
    }
    if app
        .state::<crate::state::SharedRuntimeInstall>()
        .is_running()
    {
        return Err(AppError::NotConfigured);
    }
    let reserve_app = app.clone();
    let (query_root, identity) = crate::commands::run_blocking("reserve panel query", move || {
        let root = update_schedule::reserve(&reserve_app, Target::Panel)?;
        let identity = crate::commands::event_delivery::prepare(
            &root,
            "panel-check-failed",
            crate::modules::notifications::registry::Channel::Official,
            crate::modules::runtime::OFFICIAL_PACKAGE.as_bytes(),
        );
        Ok((root, identity))
    })
    .await?;
    let result = query_panel_metadata(&app).await;
    query.sequence = query.sequence.wrapping_add(1);
    query.value = result.as_ref().ok().cloned();
    gate.1.store(query.sequence, Ordering::Release);
    let value = query.value.clone();
    let succeeded = result.is_ok();
    let commit_app = app.clone();
    crate::commands::run_blocking("commit panel query", move || {
        update_schedule::complete(&commit_app, &query_root, Target::Panel, value.as_ref())?;
        crate::commands::event_delivery::publish(
            &commit_app,
            &query_root,
            if succeeded {
                "panel-check-succeeded"
            } else {
                "panel-check-failed"
            },
            identity,
            trigger.unwrap_or_default().into(),
        );
        Ok(())
    })
    .await?;
    result
}

async fn query_panel_metadata(
    app: &tauri::AppHandle,
) -> AppResult<crate::modules::about::remote::OfficialRemoteLatest> {
    use tauri::Manager;
    let Some(home) = app.try_state::<crate::state::SharedHomeDir>() else {
        return Err(AppError::NotConfigured);
    };
    let npm = crate::modules::about::remote::discovered_npm().ok_or(AppError::NotConfigured)?;
    let environment = crate::modules::preferences::network_environment_for_app(app, home.0.clone());
    let working = home.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        crate::modules::about::remote::query_remote_latest(&npm, &working, &environment, "latest")
    })
    .await
    .map_err(|_| AppError::NotConfigured)?
}
