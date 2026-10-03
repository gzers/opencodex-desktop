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
        platform: "macOS".into(),
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
