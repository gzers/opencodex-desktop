//! 安装发现命令：只做输入校验、模块编排与 DTO 转换。

use crate::errors::AppResult;
use crate::modules::discovery::EnvironmentDiscovery;
use crate::types::discovery::{DiscoveryRequest, EnvironmentReportDto};

#[tauri::command]
pub async fn discover_environment(
    request: Option<DiscoveryRequest>,
) -> AppResult<EnvironmentReportDto> {
    let explicit_paths = request.and_then(|request| request.to_environment_paths());
    let paths = explicit_paths.unwrap_or_else(crate::types::discovery_paths::macos_default_paths);

    tauri::async_runtime::spawn_blocking(move || {
        Ok(EnvironmentDiscovery::new(paths).discover().into())
    })
    .await
    .map_err(|_| crate::errors::AppError::NotConfigured)?
}
