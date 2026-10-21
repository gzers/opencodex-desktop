//! A citation can refer to an actual fetched snapshot, not client-supplied text
//! wearing a URL. This records provenance, not factual support for every claim.
use super::{
    agent::ResearchSource,
    schema::*,
    store::sha256,
    tasks::{Submission, Task},
};
use crate::{infrastructure::atomic_write::atomic_write, modules::backup::safety};
use std::path::{Path, PathBuf};
fn path(root: &Path, id: &str) -> ModelResult<PathBuf> {
    require_uuid(id)?;
    let path = root
        .join("manager-state/model-research-sources")
        .join(format!("{id}.json"));
    safety::check_path(root, &path, true)?;
    Ok(path)
}
fn digest(source: &ResearchSource) -> ModelResult<String> {
    Ok(sha256(&serde_json::to_vec(&(
        &source.url,
        &source.retrieved_at,
        &source.text,
        &source.links,
    ))?))
}
fn validate(source: &ResearchSource) -> ModelResult<()> {
    let url = reqwest::Url::parse(&source.url)
        .map_err(|_| invalid("research_receipt", "资料地址无效"))?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || source.text.len() > 128000
        || source.links.len() > 30
        || source.links.iter().any(|s| s.len() > 8192)
        || chrono::DateTime::parse_from_rfc3339(&source.retrieved_at).is_err()
    {
        return Err(invalid("research_receipt", "资料快照格式或范围无效"));
    }
    Ok(())
}
pub fn remember(root: &Path, mut sources: Vec<ResearchSource>) -> ModelResult<Vec<ResearchSource>> {
    if sources.len() > 5 {
        return Err(invalid("research_scope", "本次资料超过数量上限"));
    }
    for source in &mut sources {
        validate(source)?;
        let id = uuid::Uuid::new_v4().to_string();
        source.receipt_id = Some(id.clone());
        source.content_digest = Some(digest(source)?);
        let bytes = serde_json::to_vec(source)?;
        let path = path(root, &id)?;
        atomic_write(&path, &bytes, 0o600)?;
        if safety::read_file(&path, 1024 * 1024)? != bytes {
            return Err(invalid("research_readback", "资料快照未读回确认"));
        }
    }
    Ok(sources)
}
pub fn verify(root: &Path, sources: &[ResearchSource]) -> ModelResult<()> {
    if sources.len() > 5 {
        return Err(invalid("research_scope", "本次资料超过数量上限"));
    }
    for source in sources {
        validate(source)?;
        let id = source.receipt_id.as_deref().ok_or_else(|| {
            invalid(
                "research_receipt",
                "在线资料缺少实际读取回执，请重新读取资料",
            )
        })?;
        let saved: ResearchSource =
            serde_json::from_slice(&safety::read_file(&path(root, id)?, 1024 * 1024)?)?;
        if saved != *source || source.content_digest.as_deref() != Some(digest(source)?.as_str()) {
            return Err(invalid(
                "research_receipt",
                "资料正文、地址或时间与读取回执不同",
            ));
        }
    }
    Ok(())
}
pub fn citations(
    task: &Task,
    sources: &[ResearchSource],
    result: &mut Submission,
) -> ModelResult<()> {
    for proposal in &mut result.proposals {
        let target = task
            .targets
            .iter()
            .find(|t| Some(t.template_id.as_str()) == proposal.template_id.as_deref());
        let prior = target.map(|t| {
            t.buffer
                .as_ref()
                .map(|b| &b.content.sources)
                .unwrap_or(&t.revision.content.sources)
        });
        for evidence in &mut proposal.content.sources {
            if prior.is_some_and(|values| values.contains(evidence)) {
                continue;
            }
            if let Some(url) = &evidence.url {
                let source = sources.iter().find(|s| &s.url == url).ok_or_else(|| {
                    invalid(
                        "agent_source_evidence",
                        "建议引用了未读取或已改变的资料地址；原内容保留，请补充实际资料后重试",
                    )
                })?;
                evidence.recorded_at = source.retrieved_at.clone();
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn source() -> ResearchSource {
        ResearchSource {
            url: "https://official.example.test/model".into(),
            retrieved_at: "2026-10-19T00:00:00Z".into(),
            text: "Model capabilities fixture".into(),
            links: vec![],
            receipt_id: None,
            content_digest: None,
        }
    }
    #[test]
    fn proposed_citations_require_fetched_urls_and_get_the_actual_retrieval_time() {
        let root = tempfile::tempdir().unwrap();
        let local = super::super::ModelStore::new(root.path()).load().unwrap();
        let template = &local.library.templates[0];
        let target = TemplateRef {
            template_id: template.id.clone(),
            revision_id: template.revisions[0].id.clone(),
        };
        let task = super::super::tasks::Tasks::new(root.path())
            .prepare(super::super::tasks::CreateTask {
                scene: super::super::tasks::Scene::FromSources,
                targets: vec![target.clone()],
                allow_new: false,
                requirements: "source fixture".into(),
                expected_library: Some(local.library_digest),
                buffers: vec![],
            })
            .unwrap()
            .task;
        let mut content = template.revisions[0].content.clone();
        content.sources.push(SourceEvidence {
            field: "contextWindow".into(),
            url: Some("https://unread.example.test/model".into()),
            note: "生成的来源".into(),
            recorded_at: "2025-01-01T00:00:00Z".into(),
        });
        let mut result = Submission {
            submission_id: uuid::Uuid::new_v4().to_string(),
            contract_version: 1,
            proposals: vec![super::super::tasks::Proposal {
                template_id: Some(target.template_id),
                base_revision: Some(target.revision_id),
                label: template.label.clone(),
                content,
            }],
            report: None,
        };
        let sources = remember(root.path(), vec![source()]).unwrap();
        assert_eq!(
            citations(&task, &sources, &mut result).unwrap_err().code,
            "agent_source_evidence"
        );
        let evidence = result.proposals[0].content.sources.last_mut().unwrap();
        evidence.url = Some(sources[0].url.clone());
        citations(&task, &sources, &mut result).unwrap();
        assert_eq!(
            result.proposals[0]
                .content
                .sources
                .last()
                .unwrap()
                .recorded_at,
            sources[0].retrieved_at
        );
        // Existing unchanged historical evidence is retained without pretending it was fetched now.
        citations(
            &task,
            &[],
            &mut Submission {
                submission_id: uuid::Uuid::new_v4().to_string(),
                contract_version: 1,
                proposals: vec![super::super::tasks::Proposal {
                    template_id: Some(template.id.clone()),
                    base_revision: Some(template.revisions[0].id.clone()),
                    label: template.label.clone(),
                    content: template.revisions[0].content.clone(),
                }],
                report: None,
            },
        )
        .unwrap();
    }
    #[test]
    fn forged_client_body_timestamp_or_url_cannot_be_used_as_fetched_evidence() {
        let root = tempfile::tempdir().unwrap();
        let sources = remember(root.path(), vec![source()]).unwrap();
        verify(root.path(), &sources).unwrap();
        let mut changed = sources.clone();
        changed[0].text = "invented".into();
        assert_eq!(
            verify(root.path(), &changed).unwrap_err().code,
            "research_receipt"
        );
        changed = sources.clone();
        changed[0].retrieved_at = "2025-01-01T00:00:00Z".into();
        assert!(verify(root.path(), &changed).is_err());
        changed = sources;
        changed[0].url = "https://other.example.test".into();
        assert!(verify(root.path(), &changed).is_err());
        assert!(verify(root.path(), &[source()]).is_err());
    }
}
