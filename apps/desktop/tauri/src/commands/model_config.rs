//! IPC binds the immutable startup data root; blocking writers retain admission.
use crate::modules::model_config::compiler::{project, DraftProjection};
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
    expected_workspace: String,
    template_id: String,
    revision_ids: Vec<String>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Snapshot> {
    execute(data_root.0.clone(), move |store| {
        store.delete(
            &expected_library,
            &expected_policy,
            &expected_workspace,
            &template_id,
            &revision_ids,
        )
    })
    .await
}
#[tauri::command]
pub async fn model_policy_stage(
    expected_policy: String,
    expected_workspace: String,
    policy: Policy,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Snapshot> {
    execute(data_root.0.clone(), move |store| {
        store.stage_policy(&expected_policy, &expected_workspace, policy)
    })
    .await
}
#[tauri::command]
pub async fn model_policy_prepare(
    policy: Policy,
    expected_library: String,
    expected_policy: String,
    expected_workspace: String,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> ModelResult<crate::modules::model_config::runtime::SavePlanView> {
    let _admission = crate::infrastructure::storage_writers::global().admit()?;
    crate::modules::model_config::runtime::prepare(
        &data_root.0,
        &context,
        policy,
        expected_library,
        expected_policy,
        expected_workspace,
    )
    .await
}
#[tauri::command]
pub async fn model_policy_apply(
    plan_id: String,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> ModelResult<crate::modules::model_config::runtime::SavePlanView> {
    let _admission = crate::infrastructure::storage_writers::global().admit()?;
    observe_policy(
        &data_root.0,
        &plan_id,
        crate::modules::model_config::runtime::apply(&data_root.0, &context, &plan_id),
    )
    .await
}
#[tauri::command]
pub async fn model_policy_result(
    plan_id: String,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> ModelResult<crate::modules::model_config::runtime::SavePlanView> {
    let _admission = crate::infrastructure::storage_writers::global().admit()?;
    observe_policy(
        &data_root.0,
        &plan_id,
        crate::modules::model_config::runtime::reconcile(&data_root.0, &context, &plan_id),
    )
    .await
}
#[tauri::command]
pub async fn model_policy_latest(
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Option<crate::modules::model_config::runtime::SavePlanView>> {
    let root = data_root.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        crate::modules::model_config::runtime::latest(&root)
    })
    .await
    .map_err(|_| invalid("plan_worker", "读取原保存计划中断"))?
}
#[tauri::command]
pub async fn model_policy_history(
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Vec<crate::modules::model_config::runtime::HistoryEntry>> {
    let root = data_root.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        crate::modules::model_config::runtime::history(&root)
    })
    .await
    .map_err(|_| invalid("history_worker", "读取保存记录中断"))?
}
#[tauri::command]
pub async fn model_policy_client_status(
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<crate::modules::model_config::client_state::View> {
    let root = data_root.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        crate::modules::model_config::client_state::status(&root)
    })
    .await
    .map_err(|_| invalid("client_state", "读取客户端核对状态中断"))?
}
#[tauri::command]
pub async fn model_policy_client_check(
    request: crate::modules::model_config::client_state::Request,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> ModelResult<crate::modules::model_config::client_state::View> {
    let _admission = crate::infrastructure::storage_writers::global().admit()?;
    crate::modules::model_config::client_state::check(&data_root.0, &context, request).await
}
#[tauri::command]
pub async fn model_policy_recovery_prepare(
    request: crate::modules::model_config::recovery::Request,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<crate::modules::model_config::recovery::Preview> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        crate::modules::model_config::runtime::prepare_recovery(&root, request)
    })
    .await
    .map_err(|_| invalid("history_worker", "恢复比较中断，未写入运行配置"))?
}
#[tauri::command]
pub async fn model_policy_recovery_accept(
    plan_id: String,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Snapshot> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        crate::modules::model_config::runtime::accept_recovery(&root, &plan_id)
    })
    .await
    .map_err(|_| invalid("history_worker", "恢复草稿结果需重新核对，请保留原计划"))?
}
#[tauri::command]
pub async fn model_policy_compensate(
    plan_id: String,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> ModelResult<crate::modules::model_config::runtime::SavePlanView> {
    let _admission = crate::infrastructure::storage_writers::global().admit()?;
    observe_policy(
        &data_root.0,
        &plan_id,
        crate::modules::model_config::runtime::compensate(&data_root.0, &context, &plan_id),
    )
    .await
}
async fn observe_policy(
    root: &std::path::Path,
    id: &str,
    operation: impl std::future::Future<
        Output = ModelResult<crate::modules::model_config::runtime::SavePlanView>,
    >,
) -> ModelResult<crate::modules::model_config::runtime::SavePlanView> {
    use crate::modules::model_config::feedback::{self, OperationKind, OperationPhase};
    let observation = uuid::Uuid::new_v4().to_string();
    feedback::progress(
        root,
        &observation,
        OperationKind::PolicySave,
        OperationPhase::Running,
    );
    let result = operation.await;
    let success = result
        .as_ref()
        .is_ok_and(|plan| ["committed", "compensated"].contains(&plan.phase.as_str()));
    let phase = if result.as_ref().is_ok_and(|plan| plan.phase == "committed") {
        OperationPhase::Review
    } else if success {
        OperationPhase::Succeeded
    } else {
        OperationPhase::Failed
    };
    feedback::progress(root, &observation, OperationKind::PolicySave, phase);
    if crate::modules::model_config::runtime::query(root, id)
        .is_ok_and(|plan| plan.phase != "prepared")
    {
        feedback::policy_terminal(root, id, success);
    }
    result
}
#[tauri::command]
pub fn model_match_preview(
    content: TemplateContent,
    channel_id: String,
    model_id: String,
) -> ModelResult<matching::MatchPreview> {
    matching::preview(&content, &channel_id, &model_id)
}

#[tauri::command]
pub async fn model_policy_preview(
    policy: Policy,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<DraftProjection> {
    execute(data_root.0.clone(), move |store| {
        let snapshot = store.load()?;
        project(&snapshot.library, &policy)
    })
    .await
}

#[tauri::command]
pub async fn model_upstream_snapshot(
    context: tauri::State<'_, crate::state::SharedProcessContext>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<crate::modules::model_config::channel_actions::ReadView> {
    let _admission = crate::infrastructure::storage_writers::global().admit()?;
    let snapshot = crate::modules::model_config::upstream::snapshot(&context).await?;
    crate::modules::model_config::channel_actions::remember(&data_root.0, &context, snapshot)
}
#[tauri::command]
pub async fn model_channel_discover(
    request: crate::modules::model_config::discovery::DiscoveryRequest,
) -> ModelResult<crate::modules::model_config::discovery::DiscoveryResult> {
    crate::modules::model_config::discovery::discover(request).await
}
#[tauri::command]
pub async fn model_channel_forget_credential(reference: String) -> ModelResult<()> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::modules::model_config::discovery::forget(&reference)
    })
    .await
    .map_err(|_| invalid("credential_store", "凭据清理未完成"))?
}
#[tauri::command]
pub async fn model_node_measure(
    request: crate::modules::model_config::measurements::Request,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<crate::modules::model_config::measurements::ResultSet> {
    use crate::modules::model_config::feedback::{self, OperationKind, OperationPhase};
    let id = request.run_id.clone();
    feedback::progress(
        &data_root.0,
        &id,
        OperationKind::NodeMeasurement,
        OperationPhase::Running,
    );
    let result = crate::modules::model_config::measurements::measure(request).await;
    let phase = match &result {
        Ok(value) if value.stopped => OperationPhase::Stopped,
        Err(error) if error.code == "operation_stopped" => OperationPhase::Stopped,
        Ok(value) if value.results.iter().all(|r| r.error.is_none()) => OperationPhase::Succeeded,
        _ => OperationPhase::Failed,
    };
    feedback::progress(&data_root.0, &id, OperationKind::NodeMeasurement, phase);
    result
}
#[tauri::command]
pub fn model_node_measure_stop(run_id: String) -> ModelResult<()> {
    crate::modules::model_config::measurements::stop(&run_id)
}
#[tauri::command]
pub async fn model_agent_accept_review(
    request: crate::modules::model_config::agent::ReviewRequest,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<BatchCommit> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        let result = crate::modules::model_config::agent::accept_review(&root, request)?;
        crate::modules::model_config::broker::announce(&root);
        Ok(result)
    })
    .await
    .map_err(|_| {
        invalid(
            "agent_review_worker",
            "建议保存结果待核对，请保留原确认身份",
        )
    })?
}
#[tauri::command]
pub async fn model_agent_latest_result(
    template_id: Option<String>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Option<crate::modules::model_config::agent::PendingSuggestion>> {
    let root = data_root.0.clone();
    let admission = crate::infrastructure::storage_writers::global().admit()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        crate::modules::model_config::agent::latest_result(&root, template_id.as_deref())
    })
    .await
    .map_err(|_| invalid("agent_result_worker", "上次建议读取中断"))?
}
#[tauri::command]
pub async fn model_external_task(
    request: crate::modules::model_config::tasks::CreateTask,
    data_root: tauri::State<'_, SharedDataRoot>,
    app: tauri::AppHandle,
) -> ModelResult<crate::modules::model_config::broker::TaskBundle> {
    let _admission = crate::infrastructure::storage_writers::global().admit()?;
    crate::modules::model_config::broker::configure(app);
    crate::modules::model_config::broker::create_bundle(data_root.0.clone(), request).await
}
#[tauri::command]
pub async fn model_external_activate(
    grant: crate::modules::model_config::tasks::TaskGrant,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<()> {
    let root = data_root.0.clone();
    execute(root.clone(), move |_store| {
        crate::modules::model_config::tasks::Tasks::new(&root).activate(&grant)
    })
    .await
}
#[tauri::command]
pub async fn model_external_result(
    task_id: String,
    submission_id: String,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<serde_json::Value> {
    let root = data_root.0.clone();
    execute(root.clone(), move |_| {
        crate::modules::model_config::tasks::Tasks::new(&root)
            .owner_result(&task_id, &submission_id)
    })
    .await
}
#[tauri::command]
pub async fn model_agent_preferences(
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Option<crate::modules::model_config::agent::AgentSettings>> {
    let root = data_root.0.clone();
    execute(root.clone(), move |_| {
        crate::modules::model_config::agent::settings(&root)
    })
    .await
}
#[tauri::command]
pub async fn model_agent_preferences_save(
    settings: crate::modules::model_config::agent::AgentSettings,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<crate::modules::model_config::agent::AgentSettings> {
    let root = data_root.0.clone();
    execute(root.clone(), move |_| {
        crate::modules::model_config::agent::save_settings(&root, settings)
    })
    .await
}
#[tauri::command]
pub async fn model_research_sources(
    urls: Vec<String>,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Vec<crate::modules::model_config::agent::ResearchSource>> {
    use crate::modules::model_config::feedback::{self, OperationKind, OperationPhase};
    let id = uuid::Uuid::new_v4().to_string();
    feedback::progress(
        &data_root.0,
        &id,
        OperationKind::Research,
        OperationPhase::Running,
    );
    let result = crate::modules::model_config::agent::research(urls).await;
    feedback::progress(
        &data_root.0,
        &id,
        OperationKind::Research,
        if result.is_ok() {
            OperationPhase::Succeeded
        } else {
            OperationPhase::Failed
        },
    );
    result
}
#[tauri::command]
pub async fn model_research_search(
    query: String,
    data_root: tauri::State<'_, SharedDataRoot>,
) -> ModelResult<Vec<crate::modules::model_config::agent::ResearchSource>> {
    use crate::modules::model_config::feedback::{self, OperationKind, OperationPhase};
    let id = uuid::Uuid::new_v4().to_string();
    feedback::progress(
        &data_root.0,
        &id,
        OperationKind::Research,
        OperationPhase::Running,
    );
    let result = crate::modules::model_config::agent::search(query).await;
    feedback::progress(
        &data_root.0,
        &id,
        OperationKind::Research,
        if result.is_ok() {
            OperationPhase::Succeeded
        } else {
            OperationPhase::Failed
        },
    );
    result
}
#[tauri::command]
pub async fn model_agent_generate(
    request: crate::modules::model_config::agent::GenerateRequest,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> ModelResult<crate::modules::model_config::tasks::Submission> {
    let _admission = crate::infrastructure::storage_writers::global().admit()?;
    use crate::modules::model_config::feedback::{self, OperationKind, OperationPhase};
    let id = request.run_id.clone();
    feedback::progress(
        &data_root.0,
        &id,
        OperationKind::Generation,
        OperationPhase::Running,
    );
    let result =
        crate::modules::model_config::agent::generate(&data_root.0, &context, request).await;
    feedback::progress(
        &data_root.0,
        &id,
        OperationKind::Generation,
        match &result {
            Ok(_) => OperationPhase::Review,
            Err(error) if ["agent_stopped", "operation_stopped"].contains(&error.code.as_str()) => {
                OperationPhase::Stopped
            }
            Err(_) => OperationPhase::Failed,
        },
    );
    result
}
#[tauri::command]
pub fn model_agent_stop(run_id: String) -> ModelResult<()> {
    crate::modules::model_config::agent::stop(&run_id)
}
