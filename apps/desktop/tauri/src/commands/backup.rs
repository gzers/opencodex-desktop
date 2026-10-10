use crate::errors::{AppError, AppResult};
use crate::modules::backup::browser::{self, BackupFiles};
use crate::state::SharedDataRoot;

#[tauri::command]
pub async fn backup_files(root: tauri::State<'_, SharedDataRoot>) -> AppResult<BackupFiles> {
    let root = root.0.clone();
    tauri::async_runtime::spawn_blocking(move || browser::list(&root)).await
        .map_err(|_| AppError::NotConfigured)?
}

#[tauri::command]
pub async fn open_backup_file(root: tauri::State<'_, SharedDataRoot>, id: String) -> AppResult<()> {
    let root = root.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let path = browser::resolve_open(&root, &id)?;
        crate::commands::workspace::open_path(&path)
    }).await.map_err(|_| AppError::NotConfigured)?
}
