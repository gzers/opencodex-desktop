//! Read-only saved revision matching and exact reference report. Matching is
//! never evidence of adoption, client activation or runtime validation.
use super::{matching, runtime, schema::*, store::ModelStore};
use serde::Serialize;
use std::{collections::BTreeSet, path::Path};
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceLocation {
    pub state: String,
    pub location: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelMatch {
    pub state: String,
    pub channel_id: String,
    pub channel_label: String,
    pub model_id: String,
    pub matched: bool,
    pub reason: String,
    pub adopted: bool,
    pub resolution_error: Option<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub template_id: String,
    pub revision_id: String,
    pub library_digest: String,
    pub policy_digest: String,
    pub workspace_digest: String,
    pub references: Vec<ReferenceLocation>,
    pub models: Vec<ModelMatch>,
    pub historical_dependency: bool,
}
pub fn report(
    root: &Path,
    template_id: &str,
    revision_id: &str,
    expected_library: &str,
) -> ModelResult<Report> {
    let snapshot = ModelStore::new(root).load()?;
    report_snapshot(
        root,
        snapshot,
        template_id,
        revision_id,
        expected_library,
        || ModelStore::new(root).load(),
    )
}
pub fn report_file(
    root: &Path,
    home: &Path,
    template_id: &str,
    revision_id: &str,
    expected_library: &str,
) -> ModelResult<Report> {
    let snapshot = super::config_file::snapshot(root, home)?;
    report_snapshot(
        root,
        snapshot,
        template_id,
        revision_id,
        expected_library,
        || super::config_file::snapshot(root, home),
    )
}
fn report_snapshot(
    root: &Path,
    snapshot: Snapshot,
    template_id: &str,
    revision_id: &str,
    expected_library: &str,
    read_current: impl FnOnce() -> ModelResult<Snapshot>,
) -> ModelResult<Report> {
    if snapshot.library_digest != expected_library {
        return Err(invalid(
            "stale_baseline",
            "模版库已变化，请重新读取版本后查看",
        ));
    }
    let reference = TemplateRef {
        template_id: template_id.into(),
        revision_id: revision_id.into(),
    };
    let revision = matching::revision(&snapshot.library, &reference)?;
    let mut references = vec![];
    let mut models = vec![];
    for (state, policy) in std::iter::once(("已提交策略", &snapshot.policy)).chain(
        snapshot
            .workspace
            .iter()
            .filter(|w| w.policy != snapshot.policy)
            .map(|w| ("工作区草稿", &w.policy)),
    ) {
        if policy.template_refs.contains(&reference) {
            references.push(ReferenceLocation {
                state: state.into(),
                location: "全局采用".into(),
            });
        }
        for channel in &policy.channels {
            if channel.template_refs.contains(&reference) {
                references.push(ReferenceLocation {
                    state: state.into(),
                    location: format!("{} · 渠道采用", channel.label),
                });
            }
            for (id, adopted) in &channel.model_refs {
                if adopted == &reference {
                    references.push(ReferenceLocation {
                        state: state.into(),
                        location: format!("{} · {} · 精确型号采用", channel.label, id),
                    });
                }
            }
            let ids = channel
                .discovered
                .iter()
                .map(|m| m.id.as_str())
                .chain(channel.model_order.iter().map(String::as_str))
                .chain(channel.model_refs.keys().map(String::as_str))
                .chain(
                    match &channel.selection {
                        ModelSelection::Whitelist(ids) => ids.as_slice(),
                        _ => &[],
                    }
                    .iter()
                    .map(String::as_str),
                )
                .collect::<BTreeSet<_>>();
            for id in ids {
                if models.len() >= 20000 {
                    return Err(invalid(
                        "usage_limit",
                        "匹配与使用范围超过2万项查询上限，未返回不完整结果",
                    ));
                }
                let preview = matching::preview(&revision.content, &channel.id, id)?;
                let (adopted, resolution_error) =
                    match matching::resolve(&snapshot.library, policy, &channel.id, id) {
                        Ok(value) => (value.is_some_and(|(value, _)| value == reference), None),
                        Err(error) => (false, Some(error.message)),
                    };
                models.push(ModelMatch {
                    state: state.into(),
                    channel_id: channel.id.clone(),
                    channel_label: channel.label.clone(),
                    model_id: id.into(),
                    matched: preview.matched,
                    reason: preview.reason,
                    adopted,
                    resolution_error,
                });
            }
        }
    }
    let historical_dependency = runtime::recovery_uses(root, template_id, revision_id)?;
    let current = read_current()?;
    if snapshot.library_digest != current.library_digest
        || snapshot.policy_digest != current.policy_digest
        || snapshot.workspace_digest != current.workspace_digest
    {
        return Err(invalid("stale_baseline", "查看期间配置已变化，请重新查询"));
    }
    Ok(Report {
        template_id: template_id.into(),
        revision_id: revision_id.into(),
        library_digest: snapshot.library_digest,
        policy_digest: snapshot.policy_digest,
        workspace_digest: snapshot.workspace_digest,
        references,
        models,
        historical_dependency,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matching_unadopted_version_is_separate_from_exact_saved_and_draft_usage() {
        let root = tempfile::tempdir().unwrap();
        let store = ModelStore::new(root.path());
        let before = store.load().unwrap();
        let template = &before.library.templates[0];
        let revision = &template.revisions[0];
        let mut channel = ChannelPolicy {
            id: uuid::Uuid::new_v4().to_string(),
            label: "Fixture".into(),
            connection: ChannelConnection {
                provider_id: "fixture".into(),
                protocol: "openai-chat".into(),
                nodes: vec![ServiceNode {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: "Node".into(),
                    url: "https://example.test".into(),
                }],
                ..Default::default()
            },
            discovered: vec![DiscoveredModel {
                id: "gpt-6-astra".into(),
                label: "gpt-6-astra".into(),
                rules: Default::default(),
            }],
            template_refs: vec![],
            model_refs: Default::default(),
            selection: ModelSelection::None,
            model_order: vec![],
            defaults: Default::default(),
            overrides: Default::default(),
            family_overrides: Default::default(),
            naming: Default::default(),
        };
        channel.connection.active_node_id = channel.connection.nodes[0].id.clone();
        let saved = store
            .commit_runtime_policy(
                &before.library_digest,
                &before.policy_digest,
                &before.workspace_digest,
                Policy {
                    channels: vec![channel],
                    ..Default::default()
                },
            )
            .unwrap();
        let unmatched_usage = report(
            root.path(),
            &template.id,
            &revision.id,
            &saved.library_digest,
        )
        .unwrap();
        assert!(unmatched_usage.references.is_empty());
        assert!(unmatched_usage
            .models
            .iter()
            .any(|m| m.matched && !m.adopted));
        let mut draft = saved.policy.clone();
        draft.channels[0].model_refs.insert(
            "gpt-6-astra".into(),
            TemplateRef {
                template_id: template.id.clone(),
                revision_id: revision.id.clone(),
            },
        );
        let staged = store
            .stage_policy(&saved.policy_digest, &saved.workspace_digest, draft)
            .unwrap();
        let used = report(
            root.path(),
            &template.id,
            &revision.id,
            &staged.library_digest,
        )
        .unwrap();
        assert_eq!(used.references.len(), 1);
        assert_eq!(used.references[0].state, "工作区草稿");
        assert!(used
            .models
            .iter()
            .any(|m| m.state == "工作区草稿" && m.adopted));
        assert!(used
            .models
            .iter()
            .any(|m| m.state == "已提交策略" && !m.adopted));
        assert_eq!(
            report(root.path(), &template.id, &revision.id, "old")
                .unwrap_err()
                .code,
            "stale_baseline"
        );
        assert_eq!(store.load().unwrap().workspace, staged.workspace);
        assert_eq!(store.load().unwrap().policy, saved.policy);
        let mut explicit = staged.workspace.as_ref().unwrap().policy.clone();
        explicit.channels[0].discovered.push(DiscoveredModel {
            id: "unmatched-exact".into(),
            label: "Unmatched".into(),
            rules: Default::default(),
        });
        explicit.channels[0]
            .model_refs
            .insert("unmatched-exact".into(), reference_for(template, revision));
        let updated = store
            .stage_policy(&staged.policy_digest, &staged.workspace_digest, explicit)
            .unwrap();
        let actual = report(
            root.path(),
            &template.id,
            &revision.id,
            &updated.library_digest,
        )
        .unwrap();
        assert!(actual
            .models
            .iter()
            .any(|m| m.model_id == "unmatched-exact" && !m.matched && m.adopted));
    }
    fn reference_for(template: &Template, revision: &Revision) -> TemplateRef {
        TemplateRef {
            template_id: template.id.clone(),
            revision_id: revision.id.clone(),
        }
    }
}
