//! IPC binds the immutable startup data root; blocking writers retain admission.
use crate::modules::model_config::{matching, schema::*, ModelStore};
use crate::state::SharedDataRoot;
use std::path::PathBuf;

async fn execute<T: Send + 'static>(
    root: PathBuf,
    work: impl FnOnce(ModelStore) -> ModelResult<T> + Send + 'static,
) -> ModelResult<T> {
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        work(ModelStore::new(&root))
    })
    .await
    .map_err(|_| invalid("worker", "模型配置任务中断，结果需重新读取核对"))?
}

#[tauri::command]
pub async fn model_config_snapshot(
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Snapshot> {
    execute(data_root.0.clone(), |store| store.load()).await
}
#[tauri::command]
pub async fn model_template_save(
    request: SaveTemplate,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Commit> {
    execute(data_root.0.clone(), move |store| store.save(request)).await
}
#[tauri::command]
pub async fn model_template_set_default(
    expected_library: String,
    template_id: String,
    revision_id: String,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Snapshot> {
    execute(data_root.0.clone(), move |store| {
        store.set_default(&expected_library, &template_id, &revision_id)
    })
    .await
}
#[tauri::command]
pub async fn model_template_delete(
    expected_library: String,
    expected_policy: String,
    template_id: String,
    revision_ids: Vec<String>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Snapshot> {
    execute(data_root.0.clone(), move |store| {
        store.delete(
            &expected_library,
            &expected_policy,
            &template_id,
            &revision_ids,
        )
    })
    .await
}
#[tauri::command]
pub fn model_match_preview(
    content: TemplateContent,
    channel_id: String,
    model_id: String,
) -> ModelResult<matching::MatchPreview> {
    matching::preview(&content, &channel_id, &model_id)
}
