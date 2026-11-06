//! Version 2 commands use the shared matcher and never reinterpret UI rules.
use crate::modules::model_config::{
    matching_v2,
    schema::{ModelRules, ModelSelection},
    schema_v2::*,
};
use crate::state::SharedDataRoot;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryResponseV2 {
    pub dto_version: u32,
    pub library: LibraryV2,
    pub library_digest: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveRevisionInputV2 {
    pub expected_library: String,
    pub template_id: String,
    pub base_revision: Option<String>,
    pub label: String,
    pub content: TemplateContentV2,
    pub make_default: bool,
}
#[tauri::command]
pub async fn model_v2_library(
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<LibraryResponseV2> {
    let root = data_root.0.clone();
    let home = context.opencodex_home.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let snapshot =
            crate::modules::model_config::config_file::v2::library_snapshot(&root, &home)?;
        Ok(LibraryResponseV2 {
            dto_version: 2,
            library: snapshot.library,
            library_digest: snapshot.digests.library,
        })
    })
    .await
    .map_err(|_| error("model_storage"))?
}
#[tauri::command]
pub async fn model_v2_save_revision(
    request: SaveRevisionInputV2,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<LibraryResponseV2> {
    let admission = crate::infrastructure::storage_writers::global()
        .admit()
        .map_err(|_| error("model_storage"))?;
    let root = data_root.0.clone();
    let home = context.opencodex_home.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        let library = crate::modules::model_config::config_file::v2::save_revision(
            &root,
            &home,
            &SaveRevisionRequestV2 {
                expected_library: request.expected_library,
                template_id: request.template_id,
                base_revision: request.base_revision,
                revision_id: uuid::Uuid::new_v4().to_string(),
                created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                label: request.label,
                content: request.content,
                make_default: request.make_default,
                origin: crate::modules::model_config::schema::SaveOrigin::Manual,
            },
        )?;
        let library_digest = digest(&canonical_bytes(&library)?);
        Ok(LibraryResponseV2 {
            dto_version: 2,
            library,
            library_digest,
        })
    })
    .await
    .map_err(|_| error("model_storage"))?
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConditionPreviewRequest {
    pub content: TemplateContentV2,
    pub raw_model_ids: Vec<String>,
    pub channel_id: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionPreviewRow {
    pub raw_model_id: String,
    pub family_matched: bool,
    pub matched_condition_id: Option<String>,
    pub rules: ModelRules,
    pub protocol: ProtocolStateV2,
    pub field_origins: BTreeMap<String, FieldOriginV2>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionPreview {
    pub dto_version: u32,
    pub preview_only: bool,
    pub content_digest: String,
    pub rows: Vec<ConditionPreviewRow>,
}
pub fn preview_conditions(request: ConditionPreviewRequest) -> Result<ConditionPreview> {
    check_id(&request.channel_id)?;
    matching_v2::validate_content_v2(&request.content)?;
    if request.raw_model_ids.len() > 1024
        || request
            .raw_model_ids
            .iter()
            .any(|s| s.is_empty() || s.len() > 2048 || s.chars().any(char::is_control))
    {
        return Err(error("model_limit"));
    }
    // A content preview has no authority to select or persist a saved revision.
    // Reuse the exact effective-model resolver with one in-memory preview entry.
    let label = "内容预览";
    let content_digest = content_digest(label, &request.content)?;
    let reference = RefV2 {
        template_id: "11111111-1111-4111-8111-111111111111".into(),
        revision_id: "22222222-2222-4222-8222-222222222222".into(),
        digest: content_digest.clone(),
    };
    let library = LibraryV2 {
        schema_version: 2,
        submission_receipts: BTreeMap::new(),
        templates: vec![TemplateV2 {
            id: reference.template_id.clone(),
            label: label.into(),
            source: super::super::modules::model_config::schema::TemplateSource::User,
            default_revision: Some(reference.revision_id.clone()),
            revisions: vec![RevisionV2 {
                id: reference.revision_id.clone(),
                created_at: "2026-11-06T00:00:00Z".into(),
                saved_label: label.into(),
                content_version: 2,
                content: request.content.clone(),
                digest: content_digest.clone(),
                base_revision: None,
                changes: vec![],
                origin: super::super::modules::model_config::schema::SaveOrigin::Manual,
            }],
        }],
    };
    let channel = ChannelV2 {
        id: request.channel_id,
        label: label.into(),
        connection: ConnectionV2 {
            provider_id: "preview".into(),
            nodes: vec![],
            active_node_id: String::new(),
            credential_ref: None,
            enabled: false,
            allow_private_network: false,
            auth_mode: None,
            source_read_id: None,
            imported_config: None,
        },
        selection: ModelSelection::None,
        model_order: vec![],
        models: vec![],
        template_refs: vec![],
        defaults: ModelRules::default(),
        family_overrides: BTreeMap::new(),
        naming: BTreeMap::new(),
        namespace: None,
    };
    let mut rows = vec![];
    for raw in request.raw_model_ids {
        let matched = matching_v2::family_matches_v2(&request.content, &channel.id, &raw);
        let model = ModelConfigV2 {
            raw_model_id: raw.clone(),
            label: raw.clone(),
            observed: true,
            source: SourceModelV2 {
                raw_model_id: raw.clone(),
                fields: BTreeMap::new(),
            },
            explicit_overrides: ModelRules::default(),
            binding: matched.then(|| ModelBindingV2::Manual {
                reference: reference.clone(),
            }),
        };
        let effective = matching_v2::resolve_effective_model_v2(&library, &channel, &model)?;
        rows.push(ConditionPreviewRow {
            raw_model_id: raw,
            family_matched: matched,
            matched_condition_id: effective.matched_condition_id,
            rules: effective.rules,
            protocol: effective.protocol,
            field_origins: effective.field_origins,
        });
    }
    Ok(ConditionPreview {
        dto_version: 2,
        preview_only: true,
        content_digest,
        rows,
    })
}
#[tauri::command]
pub async fn model_v2_validate_content(content: TemplateContentV2) -> Result<()> {
    tauri::async_runtime::spawn_blocking(move || matching_v2::validate_content_v2(&content))
        .await
        .map_err(|_| error("model_source_invalid"))?
}
#[tauri::command]
pub async fn model_v2_preview_conditions(
    request: ConditionPreviewRequest,
) -> Result<ConditionPreview> {
    tauri::async_runtime::spawn_blocking(move || preview_conditions(request))
        .await
        .map_err(|_| error("model_source_invalid"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn content_preview_uses_engine_and_preserves_exact_model_ids() {
        let library = seed_library_v2().unwrap();
        let template = library
            .templates
            .iter()
            .find(|t| t.label.to_ascii_lowercase().contains("gpt"))
            .unwrap();
        let content = template.revisions[0].content.clone();
        let result = preview_conditions(ConditionPreviewRequest {
            content,
            raw_model_ids: vec![
                "gpt-6.1-sol".into(),
                "gpt-6-1-sol".into(),
                "unrelated".into(),
            ],
            channel_id: "33333333-3333-4333-8333-333333333333".into(),
        })
        .unwrap();
        assert!(result.preview_only);
        assert_eq!(result.rows[0].raw_model_id, "gpt-6.1-sol");
        assert_eq!(result.rows[1].raw_model_id, "gpt-6-1-sol");
        assert!(!result.rows[2].family_matched);
        assert!(result.rows[2].matched_condition_id.is_none());
        assert!(matches!(result.rows[2].protocol, ProtocolStateV2::Missing));
    }
}

async fn file_work<T: Send + 'static>(
    root: std::path::PathBuf,
    home: std::path::PathBuf,
    write: bool,
    work: impl FnOnce(&std::path::Path, &std::path::Path) -> Result<T> + Send + 'static,
) -> Result<T> {
    let admission = if write {
        Some(
            crate::infrastructure::storage_writers::global()
                .admit()
                .map_err(|_| error("model_storage"))?,
        )
    } else {
        None
    };
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        work(&root, &home)
    })
    .await
    .map_err(|_| error("model_storage"))?
}
#[tauri::command]
pub async fn model_v2_file_snapshot(
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::config_file::v2::FileSnapshotV2> {
    file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        false,
        crate::modules::model_config::config_file::v2::snapshot,
    )
    .await
}
#[tauri::command]
pub async fn model_v2_read_existing(
    request: crate::modules::model_config::config_file::v2::ReadRequestV2,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::config_file::v2::ReadResponseV2> {
    file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        true,
        move |root, home| {
            crate::modules::model_config::config_file::v2::read_existing(root, home, request)
        },
    )
    .await
}
#[tauri::command]
pub async fn model_v2_begin_draft(
    request: crate::modules::model_config::config_file::v2::ReadRequestV2,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::config_file::v2::FileSnapshotV2> {
    file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        true,
        move |root, home| {
            crate::modules::model_config::config_file::v2::begin_draft(root, home, request)
        },
    )
    .await
}
#[tauri::command]
pub async fn model_v2_stage_connection(
    request: crate::modules::model_config::config_file::v2::StageConnectionV2,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::config_file::v2::FileSnapshotV2> {
    file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        true,
        move |root, home| {
            crate::modules::model_config::config_file::v2::stage_connection(root, home, request)
        },
    )
    .await
}
#[tauri::command]
pub async fn model_v2_file_save(
    request: crate::modules::model_config::config_file::v2::SaveRequestV2,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::config_file::v2::FileSnapshotV2> {
    file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        true,
        move |root, home| crate::modules::model_config::config_file::v2::save(root, home, request),
    )
    .await
}
#[tauri::command]
pub async fn model_v2_copy_channel(
    request: crate::modules::model_config::config_file::v2::CopyChannelV2,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::config_file::v2::FileSnapshotV2> {
    file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        true,
        move |root, home| {
            crate::modules::model_config::config_file::v2::copy_channel(root, home, request)
        },
    )
    .await
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryKindV2 {
    OpenaiList,
    AnthropicList,
    GoogleList,
    OllamaList,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiscoverInputV2 {
    pub discovery_kind: DiscoveryKindV2,
    pub url: String,
    pub api_key: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredRowV2 {
    pub raw_model_id: String,
    pub label: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryResponseV2 {
    pub dto_version: u32,
    pub models: Vec<DiscoveredRowV2>,
    pub elapsed_ms: u64,
}
#[tauri::command]
pub async fn model_v2_discover(request: DiscoverInputV2) -> Result<DiscoveryResponseV2> {
    let protocol = match request.discovery_kind {
        DiscoveryKindV2::OpenaiList => "openai-chat",
        DiscoveryKindV2::AnthropicList => "anthropic",
        DiscoveryKindV2::GoogleList => "google",
        DiscoveryKindV2::OllamaList => "ollama-native",
    };
    let result = crate::modules::model_config::discovery::preview(
        crate::modules::model_config::discovery::DiscoveryRequest {
            protocol: protocol.into(),
            url: request.url,
            api_key: request.api_key,
            credential_ref: None,
            allow_private_network: false,
            provider_id: None,
            home: None,
        },
    )
    .await
    .map_err(|_| error("model_discovery_failed"))?;
    Ok(DiscoveryResponseV2 {
        dto_version: 2,
        models: result
            .models
            .into_iter()
            .map(|m| DiscoveredRowV2 {
                raw_model_id: m.id,
                label: m.label,
            })
            .collect(),
        elapsed_ms: result.elapsed_ms,
    })
}

#[tauri::command]
pub async fn model_v2_stage_credential(
    request: crate::modules::model_config::config_file::v2::StageCredentialV2,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::config_file::v2::FileSnapshotV2> {
    file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        true,
        move |root, home| {
            crate::modules::model_config::config_file::v2::stage_credential(root, home, request)
        },
    )
    .await
}
#[tauri::command]
pub async fn model_v2_credential_view(
    channel_id: String,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::config_file::CredentialView> {
    file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        false,
        move |root, home| {
            crate::modules::model_config::config_file::v2::credential_view(root, home, &channel_id)
        },
    )
    .await
}
#[tauri::command]
pub async fn model_v2_measure_nodes(
    request: crate::modules::model_config::config_file::v2::MeasurementIntentV2,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::measurements::ResultSet> {
    let prepared = file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        false,
        move |root, home| {
            crate::modules::model_config::config_file::v2::prepare_measurements(root, home, request)
        },
    )
    .await?;
    crate::modules::model_config::measurements::measure(prepared)
        .await
        .map_err(|e| {
            let mut err = error(&e.code);
            err.issues[0].message = e.message;
            err
        })
}

#[tauri::command]
pub async fn model_v2_change_library(
    request: crate::modules::model_config::config_file::v2::LibraryChangeV2,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<LibraryResponseV2> {
    file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        true,
        move |root, home| {
            let library =
                crate::modules::model_config::config_file::v2::change_library(root, home, request)?;
            let library_digest = digest(&canonical_bytes(&library)?);
            Ok(LibraryResponseV2 {
                dto_version: 2,
                library,
                library_digest,
            })
        },
    )
    .await
}
#[tauri::command]
pub async fn model_v2_template_usage(
    template_id: String,
    revision_id: String,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::template_ops_v2::UsageReportV2> {
    file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        false,
        move |root, home| {
            crate::modules::model_config::config_file::v2::template_usage(
                root,
                home,
                &template_id,
                &revision_id,
            )
        },
    )
    .await
}

#[tauri::command]
pub async fn model_v2_execution_directory(
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::execution_directory_v2::ExecutionDirectoryV2> {
    let root = data_root.0.clone();
    let context = context.inner().clone();
    let saved = {
        let c = context.clone();
        tauri::async_runtime::spawn_blocking(move || {
            crate::modules::model_config::execution_directory_v2::capture_saved(&root, &c)
        })
        .await
        .map_err(|_| error("model_execution_unavailable"))??
    };
    let live = crate::modules::model_config::upstream::snapshot(&context)
        .await
        .map_err(|e| error(&e.code))?;
    crate::modules::model_config::execution_directory_v2::build(&saved, &live)
}
#[tauri::command]
pub async fn model_v2_bind_agent(
    settings: crate::modules::model_config::agent::AgentSettings,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::agent::AgentSettings> {
    let root = data_root.0.clone();
    let context = context.inner().clone();
    let saved = {
        let r = root.clone();
        let c = context.clone();
        tauri::async_runtime::spawn_blocking(move || {
            crate::modules::model_config::execution_directory_v2::capture_saved(&r, &c)
        })
        .await
        .map_err(|_| error("model_execution_unavailable"))??
    };
    let live = crate::modules::model_config::upstream::snapshot(&context)
        .await
        .map_err(|e| error(&e.code))?;
    let bound =
        crate::modules::model_config::execution_directory_v2::bind(&saved, &live, settings)?;
    let settings = bound.settings().clone();
    file_work(
        root,
        context.opencodex_home.clone(),
        true,
        move |root, _| {
            crate::modules::model_config::agent::save_settings(root, settings)
                .map_err(|e| error(&e.code))
        },
    )
    .await
}
#[tauri::command]
pub async fn model_v2_agent_prepare(
    request: crate::modules::model_config::agent_runtime_v2::GenerateIntentV2,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::agent_runtime_v2::RunTicketV2> {
    let context = context.inner().clone();
    file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        true,
        move |root, _| {
            crate::modules::model_config::agent_runtime_v2::prepare_run(root, &context, request)
        },
    )
    .await
}
#[tauri::command]
pub async fn model_v2_agent_generate(
    run_id: String,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::agent_runtime_v2::GeneratedReviewV2> {
    let _admission = crate::infrastructure::storage_writers::global()
        .admit()
        .map_err(|_| error("model_storage"))?;
    crate::modules::model_config::agent_runtime_v2::generate(&data_root.0, &context, &run_id).await
}
#[tauri::command]
pub fn model_v2_agent_stop(run_id: String) -> Result<()> {
    crate::modules::model_config::agent_runtime_v2::stop(&run_id)
}
#[tauri::command]
pub async fn model_v2_agent_latest(
    template_id: Option<String>,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<Option<crate::modules::model_config::agent_runtime_v2::GeneratedReviewV2>> {
    file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        false,
        move |root, _| {
            crate::modules::model_config::agent_runtime_v2::latest(root, template_id.as_deref())
        },
    )
    .await
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcceptAgentV2 {
    pub run_id: String,
    pub expected_library: String,
    pub reviewed_response: String,
}
#[tauri::command]
pub async fn model_v2_agent_accept(
    request: AcceptAgentV2,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::agent_runtime_v2::GeneratedReviewV2> {
    let context = context.inner().clone();
    file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        true,
        move |root, home| {
            let accept =
                || -> Result<crate::modules::model_config::agent_runtime_v2::GeneratedReviewV2> {
                    let response_digest = digest(request.reviewed_response.as_bytes());
                    if let Ok(receipt) =
                        crate::modules::model_config::agent_runtime_v2::mark_accepted(
                            root,
                            &context,
                            &request.run_id,
                            &response_digest,
                        )
                    {
                        return Ok(receipt);
                    }
                    let plan = crate::modules::model_config::agent_runtime_v2::prepare_accept(
                        root,
                        &context,
                        &request.run_id,
                        &request.expected_library,
                        request.reviewed_response.as_bytes(),
                    )?;
                    crate::modules::model_config::config_file::v2::accept_agent_library(
                        root, home, &plan,
                    )?;
                    crate::modules::model_config::agent_runtime_v2::mark_accepted(
                        root,
                        &context,
                        &request.run_id,
                        &plan.response_digest,
                    )
                };
            if crate::modules::model_config::agent_runtime_v2::requires_external_authority(
                root,
                &request.run_id,
            )? {
                crate::modules::model_config::broker::with_external_authority(
                    root,
                    &context,
                    &request.run_id,
                    || {
                        accept().map_err(|e| {
                            crate::modules::model_config::schema::invalid(
                                &e.code,
                                "外部任务审阅或保存未完成，原记录保持",
                            )
                        })
                    },
                )
                .map_err(|e| error(&e.code))
            } else {
                accept()
            }
        },
    )
    .await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BufferRequestV2 {
    pub target: Option<RefV2>,
    pub label: String,
    pub content: TemplateContentV2,
    pub generation: u64,
}
#[tauri::command]
pub async fn model_v2_agent_buffer(
    request: BufferRequestV2,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::agent_runtime_v2::EditingBufferV2> {
    file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        false,
        move |root, home| {
            let state =
                crate::modules::model_config::config_file::v2::library_snapshot(root, home)?;
            if request.label.trim().is_empty() || request.label.len() > 1024 {
                return Err(error("model_source_invalid"));
            }
            matching_v2::validate_content_v2(&request.content)?;
            if let Some(reference) = &request.target {
                let saved = revision(&state.library, reference)?;
                crate::modules::model_config::schema_v2::validate_editing_buffer_v2(
                    Some(&saved.content),
                    &request.content,
                )?;
            }
            let mut buffer = crate::modules::model_config::agent_runtime_v2::EditingBufferV2 {
                target: request.target,
                label: request.label,
                content: request.content,
                digest: String::new(),
                generation: request.generation,
            };
            buffer.digest = crate::modules::model_config::agent_runtime_v2::buffer_digest(&buffer)?;
            Ok(buffer)
        },
    )
    .await
}

#[tauri::command]
pub async fn model_v2_external_prepare(
    request: crate::modules::model_config::agent_runtime_v2::GenerateIntentV2,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::broker::ExternalBrokerBundleV2> {
    let _admission = crate::infrastructure::storage_writers::global()
        .admit()
        .map_err(|_| error("model_storage"))?;
    crate::modules::model_config::broker::prepare_external_broker(
        data_root.0.clone(),
        context.inner().clone(),
        request,
    )
    .await
    .map_err(|e| error(&e.code))
}
#[tauri::command]
pub async fn model_v2_external_activate(
    task_id: String,
    prepared_ticket: String,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<()> {
    let context = context.inner().clone();
    file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        true,
        move |root, _| {
            crate::modules::model_config::broker::activate_external(
                root,
                &context,
                &task_id,
                &prepared_ticket,
            )
            .map_err(|e| error(&e.code))
        },
    )
    .await
}
#[tauri::command]
pub async fn model_v2_external_revoke(
    task_id: String,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<()> {
    file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        true,
        move |root, _| {
            crate::modules::model_config::broker::revoke_external(root, &task_id)
                .map_err(|e| error(&e.code))
        },
    )
    .await
}

#[tauri::command]
pub async fn model_v2_file_project(
    request: crate::modules::model_config::config_file::v2::SaveRequestV2,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::config_file::v2::export::DraftProjectionV2> {
    let root = data_root.0.clone();
    let home = context.opencodex_home.clone();
    tauri::async_runtime::spawn_blocking(move || {
        crate::modules::model_config::config_file::v2::export::project(&root, &home, request)
    })
    .await
    .map_err(|_| error("model_storage"))?
}

#[tauri::command]
pub async fn model_v2_external_run(
    task_id: String,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::agent_runtime_v2::ExternalRunViewV2> {
    file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        false,
        move |root, _| {
            crate::modules::model_config::agent_runtime_v2::external_run_view(root, &task_id)
        },
    )
    .await
}

#[tauri::command]
pub async fn model_v2_file_history(
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<Vec<crate::modules::model_config::config_file::v2::history::HistoryEntryV2>> {
    file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        false,
        move |root, home| crate::modules::model_config::config_file::v2::history::list(root, home),
    )
    .await
}
#[tauri::command]
pub async fn model_v2_file_history_prepare(
    request: crate::modules::model_config::config_file::v2::history::HistoryPrepareV2,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::config_file::v2::history::HistoryPlanV2> {
    file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        true,
        move |root, home| {
            crate::modules::model_config::config_file::v2::history::prepare(root, home, request)
        },
    )
    .await
}
#[tauri::command]
pub async fn model_v2_file_history_accept(
    recovery_id: String,
    data_root: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, crate::state::SharedProcessContext>,
) -> Result<crate::modules::model_config::config_file::v2::FileSnapshotV2> {
    file_work(
        data_root.0.clone(),
        context.opencodex_home.clone(),
        true,
        move |root, home| {
            crate::modules::model_config::config_file::v2::history::accept(root, home, &recovery_id)
        },
    )
    .await
}
