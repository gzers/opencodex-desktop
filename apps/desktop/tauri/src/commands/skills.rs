//! Skills 压缩包导入命令层；路径边界在 projection 内统一执行。

use std::path::Path;

use crate::commands::extensions::execute_extension_write_with_paths;
use crate::commands::extensions::ExtensionWriteCommand;
use crate::types::extensions::ExtensionWriteResultDto;

use crate::errors::AppResult;
use crate::state::{SharedDataRoot, SharedHomeDir};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSkillArchiveRequest {
    pub archive_name: String,
    pub archive_payload: String,
}

#[tauri::command]
pub async fn import_skill_archive(
    request: ImportSkillArchiveRequest,
    data_root: tauri::State<'_, SharedDataRoot>,
    home: tauri::State<'_, SharedHomeDir>,
) -> AppResult<ExtensionWriteResultDto> {
    let root = data_root.0.clone();
    let home = home.0.clone();
    let name = request.archive_name;
    let payload = request.archive_payload;
    crate::commands::run_blocking("import skill archive", move || {
        import_skill_archive_with_paths(&root, &home, &name, &payload)
    })
    .await
}

pub fn import_skill_archive_with_paths(
    data_root: &Path,
    home: &Path,
    archive_name: &str,
    archive_payload: &str,
) -> AppResult<ExtensionWriteResultDto> {
    execute_extension_write_with_paths(
        data_root,
        home,
        ExtensionWriteCommand::ImportSkillArchive {
            archive_name: archive_name.to_owned(),
            archive_payload: archive_payload.to_owned(),
        },
    )
}
