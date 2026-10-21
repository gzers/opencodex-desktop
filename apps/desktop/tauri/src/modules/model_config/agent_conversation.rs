//! Local conversation/editor context only; never a template save or sync object.
use super::{
    agent::{AgentMessage, ResearchSource},
    research_receipts,
    schema::*,
    store::sha256,
};
use crate::{
    infrastructure::{atomic_write::atomic_write, locking::TargetFileLock},
    modules::backup::safety,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Content {
    pub base_revision: Option<String>,
    pub input: String,
    pub history: Vec<AgentMessage>,
    pub sources: Vec<ResearchSource>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Saved {
    pub template_id: Option<String>,
    pub digest: String,
    pub content: Content,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub template_id: Option<String>,
    pub expected_digest: Option<String>,
    pub content: Content,
}
fn path(root: &Path, template: Option<&str>) -> ModelResult<PathBuf> {
    if let Some(id) = template {
        require_uuid(id)?;
    }
    let path = root
        .join("manager-state/model-agent-conversations")
        .join(format!("{}.json", template.unwrap_or("new")));
    safety::check_path(root, &path, true)?;
    Ok(path)
}
fn validate(content: &Content) -> ModelResult<()> {
    if let Some(revision) = &content.base_revision {
        require_uuid(revision)?;
    }
    if content.history.len() > 500
        || content.input.len() > 32768
        || content.history.iter().any(|m| m.text.len() > 32768)
        || content.history.iter().map(|m| m.text.len()).sum::<usize>() > 1024 * 1024
    {
        return Err(invalid(
            "agent_conversation",
            "本机对话超过保存上限，当前输入保留",
        ));
    }
    Ok(())
}
pub fn load(root: &Path, template: Option<&str>) -> ModelResult<Option<Saved>> {
    let path = path(root, template)?;
    if !path.exists() {
        return Ok(None);
    }
    let saved: Saved = serde_json::from_slice(&safety::read_file(&path, 2 * 1024 * 1024)?)?;
    validate(&saved.content)?;
    if saved.template_id.as_deref() != template
        || saved.digest != sha256(&serde_json::to_vec(&saved.content)?)
    {
        return Err(invalid(
            "agent_conversation",
            "对话身份或摘要不同，未自动覆盖当前输入",
        ));
    }
    Ok(Some(saved))
}
pub fn save(root: &Path, request: Request) -> ModelResult<Saved> {
    validate(&request.content)?;
    research_receipts::verify(root, &request.content.sources)?;
    let path = path(root, request.template_id.as_deref())?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&path),
        true,
    )?;
    let _guard = TargetFileLock::lock(&path)?;
    let previous = load(root, request.template_id.as_deref())?;
    let digest = sha256(&serde_json::to_vec(&request.content)?);
    if previous.as_ref().map(|s| &s.digest) != request.expected_digest.as_ref() {
        if let Some(saved) = previous {
            if saved.digest == digest {
                return Ok(saved);
            }
        }
        return Err(invalid(
            "agent_conversation_changed",
            "另一窗口已保存此模版对话，当前输入保留，请先核对对话",
        ));
    }
    let saved = Saved {
        template_id: request.template_id,
        digest,
        content: request.content,
    };
    let bytes = serde_json::to_vec(&saved)?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err(invalid(
            "agent_conversation",
            "本机对话和资料超过2MiB，未截断保存",
        ));
    }
    atomic_write(&path, &bytes, 0o600)?;
    if safety::read_file(&path, 2 * 1024 * 1024)? != bytes {
        return Err(invalid("agent_conversation", "对话写后尚未读回确认"));
    }
    Ok(saved)
}
#[cfg(test)]
mod tests {
    use super::super::agent::MessageRole;
    use super::*;
    fn content(text: &str) -> Content {
        Content {
            base_revision: Some(uuid::Uuid::new_v4().to_string()),
            input: text.into(),
            history: vec![AgentMessage {
                role: MessageRole::User,
                text: "此前已发送的纠偏".into(),
            }],
            sources: vec![],
        }
    }
    #[test]
    fn keeps_unsent_input_and_sent_history_across_load_and_refuses_another_windows_stale_write() {
        let root = tempfile::tempdir().unwrap();
        let template = uuid::Uuid::new_v4().to_string();
        let first = save(
            root.path(),
            Request {
                template_id: Some(template.clone()),
                expected_digest: None,
                content: content("尚未发送"),
            },
        )
        .unwrap();
        assert_eq!(
            load(root.path(), Some(&template))
                .unwrap()
                .unwrap()
                .content
                .input,
            "尚未发送"
        );
        let mut later = first.content.clone();
        later.input = "另一窗口后来输入".into();
        let saved = save(
            root.path(),
            Request {
                template_id: Some(template.clone()),
                expected_digest: Some(first.digest.clone()),
                content: later,
            },
        )
        .unwrap();
        assert_eq!(
            save(
                root.path(),
                Request {
                    template_id: Some(template.clone()),
                    expected_digest: Some(first.digest),
                    content: first.content
                }
            )
            .unwrap_err()
            .code,
            "agent_conversation_changed"
        );
        assert_eq!(
            load(root.path(), Some(&template)).unwrap().unwrap().digest,
            saved.digest
        );
        assert!(!root.path().join(LIBRARY_PATH).exists());
        assert!(!root.path().join(POLICY_PATH).exists());
    }
}
