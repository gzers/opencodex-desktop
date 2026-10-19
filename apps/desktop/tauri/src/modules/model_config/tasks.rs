//! Scoped external proposals use the same immutable version transaction as UI saves.
use super::{
    matching,
    schema::*,
    store::{sha256, ModelStore},
};
use crate::infrastructure::{atomic_write::atomic_write, locking::TargetFileLock};
use crate::modules::backup::safety;
use chrono::{Duration, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

const TASK_DIR: &str = "manager-state/model-template-tasks";
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Scene {
    FromSources,
    UpdateModels,
    AuditAndRevise,
    PrivateModels,
    Preferences,
    CheckOnly,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateTask {
    pub scene: Scene,
    pub targets: Vec<TemplateRef>,
    pub allow_new: bool,
    pub requirements: String,
    #[serde(default)]
    pub expected_library: Option<String>,
    #[serde(default)]
    pub buffers: Vec<EditingBuffer>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EditingBuffer {
    pub template_id: String,
    pub base_revision: String,
    pub label: String,
    pub content: TemplateContent,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Target {
    pub template_id: String,
    pub label: String,
    pub revision: Revision,
    pub buffer: Option<EditingBuffer>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Task {
    pub id: String,
    pub contract_version: u32,
    pub scene: Scene,
    pub targets: Vec<Target>,
    pub allow_new: bool,
    pub requirements: String,
    pub expected_library: String,
    pub expires_at: String,
    pub revoked: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TaskRecord {
    task: Task,
    token_digest: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Proposal {
    pub template_id: Option<String>,
    pub base_revision: Option<String>,
    pub label: String,
    pub content: TemplateContent,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Submission {
    pub submission_id: String,
    pub contract_version: u32,
    pub proposals: Vec<Proposal>,
    pub report: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Receipt {
    pub submission_id: String,
    pub digest: String,
    pub state: String,
    pub outcomes: Vec<CommitIdentity>,
    pub report: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskGrant {
    pub task: Task,
    pub token: String,
}

pub struct Tasks {
    root: PathBuf,
}
impl Tasks {
    pub fn new(root: &Path) -> Self {
        Self { root: root.into() }
    }
    fn path(&self, id: &str) -> ModelResult<PathBuf> {
        require_uuid(id)?;
        let path = self.root.join(TASK_DIR).join(format!("{id}.json"));
        safety::check_path(&self.root, &path, true)?;
        Ok(path)
    }
    fn record(&self, id: &str) -> ModelResult<TaskRecord> {
        let path = self.path(id)?;
        Ok(serde_json::from_slice(&safety::read_file(
            &path,
            4 * 1024 * 1024,
        )?)?)
    }
    fn auth(&self, id: &str, token: &str) -> ModelResult<Task> {
        let record = self.record(id)?;
        let supplied = sha256(token.as_bytes());
        let equal = supplied.len() == record.token_digest.len()
            && supplied
                .bytes()
                .zip(record.token_digest.bytes())
                .fold(0u8, |a, (b, c)| a | (b ^ c))
                == 0;
        if !equal || record.task.revoked {
            return Err(invalid("task_authorization", "外部任务能力无效或已撤销"));
        }
        Ok(record.task)
    }
    /// Authenticate the listener before the external host forwards its bearer.
    /// The verifier derives this key locally; the key is never returned by API.
    pub(super) fn broker_proof(
        &self,
        id: &str,
        challenge: &str,
        authority: &str,
        pid: u32,
    ) -> ModelResult<String> {
        use hmac::{Hmac, Mac};
        require_uuid(challenge)?;
        let record = self.record(id)?;
        if record.task.id != id || record.task.revoked {
            return Err(invalid("task_authorization", "任务已撤销或身份无效"));
        }
        let mut mac = Hmac::<sha2::Sha256>::new_from_slice(record.token_digest.as_bytes())
            .map_err(|_| invalid("task_proof", "任务入口核验失败"))?;
        mac.update(format!("ocxd-model-task-v1\n{id}\n{challenge}\n{authority}\n{pid}").as_bytes());
        Ok(format!("{:x}", mac.finalize().into_bytes()))
    }
    fn active(task: &Task) -> ModelResult<()> {
        if chrono::DateTime::parse_from_rfc3339(&task.expires_at)
            .map_err(|_| invalid("task_expired", "任务期限无效"))?
            < Utc::now()
        {
            return Err(invalid(
                "task_expired",
                "外部任务已过期，请重新复制当前范围",
            ));
        }
        Ok(())
    }
    pub fn prepare(&self, request: CreateTask) -> ModelResult<TaskGrant> {
        if request.scene == Scene::CheckOnly && request.targets.is_empty() {
            return Err(invalid("task_scope", "仅检查场景需要已有模版范围"));
        }
        if request.targets.len() > 256 || request.requirements.len() > 16384 {
            return Err(invalid("task_limit", "任务范围或补充要求过大"));
        }
        if request.targets.is_empty() && !request.allow_new && request.scene != Scene::CheckOnly {
            return Err(invalid("task_scope", "请选择已有模版或明确允许新建"));
        }
        let snapshot = ModelStore::new(&self.root).load()?;
        if request
            .expected_library
            .as_ref()
            .is_some_and(|v| v != &snapshot.library_digest)
        {
            return Err(invalid(
                "stale_baseline",
                "预览基准已变化，请重新读取后复制",
            ));
        }
        if request.buffers.len() > 1 {
            return Err(invalid("task_buffer", "一次只接入当前模版的编辑缓冲"));
        }
        let mut unique = BTreeSet::new();
        let mut targets = vec![];
        for reference in request.targets {
            if !unique.insert(reference.template_id.clone()) {
                return Err(invalid("task_scope", "同一任务不能重复指定模版"));
            }
            let revision = matching::revision(&snapshot.library, &reference)?.clone();
            let template = snapshot
                .library
                .templates
                .iter()
                .find(|t| t.id == reference.template_id)
                .ok_or_else(|| invalid("task_scope", "模版不存在"))?;
            targets.push(Target {
                buffer: request
                    .buffers
                    .iter()
                    .find(|b| {
                        b.template_id == reference.template_id
                            && b.base_revision == reference.revision_id
                    })
                    .cloned(),
                template_id: reference.template_id,
                label: template.label.clone(),
                revision,
            });
        }
        for buffer in &request.buffers {
            buffer.content.validate()?;
            if !targets.iter().any(|t| {
                t.template_id == buffer.template_id && t.revision.id == buffer.base_revision
            }) {
                return Err(invalid("task_buffer", "编辑缓冲不属于本次精确范围"));
            }
        }
        let id = uuid::Uuid::new_v4().to_string();
        let token = format!(
            "{}{}",
            uuid::Uuid::new_v4().simple(),
            uuid::Uuid::new_v4().simple()
        );
        let task = Task {
            id: id.clone(),
            contract_version: 1,
            scene: request.scene,
            targets,
            allow_new: request.allow_new,
            requirements: request.requirements,
            expected_library: snapshot.library_digest,
            expires_at: (Utc::now() + Duration::days(7)).to_rfc3339(),
            revoked: false,
        };
        Ok(TaskGrant { task, token })
    }
    pub fn activate(&self, grant: &TaskGrant) -> ModelResult<()> {
        require_uuid(&grant.task.id)?;
        Self::active(&grant.task)?;
        if grant.token.len() != 64
            || !grant.token.bytes().all(|c| c.is_ascii_hexdigit())
            || grant.task.contract_version != 1
            || grant.task.revoked
        {
            return Err(invalid("task_authorization", "任务能力无效"));
        }
        let checked = self.prepare(CreateTask {
            scene: grant.task.scene.clone(),
            targets: grant
                .task
                .targets
                .iter()
                .map(|t| TemplateRef {
                    template_id: t.template_id.clone(),
                    revision_id: t.revision.id.clone(),
                })
                .collect(),
            allow_new: grant.task.allow_new,
            requirements: grant.task.requirements.clone(),
            expected_library: Some(grant.task.expected_library.clone()),
            buffers: grant
                .task
                .targets
                .iter()
                .filter_map(|t| t.buffer.clone())
                .collect(),
        })?;
        if serde_json::to_value(&checked.task.targets)?
            != serde_json::to_value(&grant.task.targets)?
        {
            return Err(invalid("task_baseline", "准备中的目标快照已变化"));
        }
        if chrono::DateTime::parse_from_rfc3339(&grant.task.expires_at)
            .map_err(|_| invalid("task_expired", "任务期限无效"))?
            > Utc::now() + Duration::days(7)
        {
            return Err(invalid("task_expired", "任务期限超出支持范围"));
        }
        let path = self.path(&grant.task.id)?;
        let lock_path = crate::infrastructure::locking::lock_path_for(&path);
        safety::check_path(&self.root, &lock_path, true)?;
        let _guard = TargetFileLock::lock(&path)?;
        if path.exists() {
            let existing = self.record(&grant.task.id)?;
            if existing.token_digest == sha256(grant.token.as_bytes())
                && serde_json::to_value(&existing.task)? == serde_json::to_value(&grant.task)?
            {
                return Ok(());
            }
            return Err(invalid("task_collision", "任务身份已登记且内容不同"));
        }
        let record = TaskRecord {
            task: grant.task.clone(),
            token_digest: sha256(grant.token.as_bytes()),
        };
        atomic_write(&path, &serde_json::to_vec(&record)?, 0o600)?;
        Ok(())
    }
    pub fn create(&self, request: CreateTask) -> ModelResult<TaskGrant> {
        let grant = self.prepare(request)?;
        self.activate(&grant)?;
        Ok(grant)
    }
    pub fn context(&self, id: &str, token: &str) -> ModelResult<Task> {
        let task = self.auth(id, token)?;
        Self::active(&task)?;
        Ok(task)
    }
    pub fn revoke(&self, id: &str) -> ModelResult<()> {
        let path = self.path(id)?;
        safety::check_path(
            &self.root,
            &crate::infrastructure::locking::lock_path_for(&path),
            true,
        )?;
        let _guard = TargetFileLock::lock(&path)?;
        let mut record = self.record(id)?;
        record.task.revoked = true;
        atomic_write(&path, &serde_json::to_vec(&record)?, 0o600)?;
        Ok(())
    }
    pub fn contract(&self, id: &str, token: &str) -> ModelResult<Value> {
        self.context(id, token)?;
        Ok(contract_definition())
    }
    fn validate(
        &self,
        task: &Task,
        submission: &Submission,
        require_current: bool,
    ) -> ModelResult<()> {
        require_uuid(&submission.submission_id)?;
        if submission.contract_version != 1 {
            return Err(invalid("contract_version", "任务契约版本不受支持"));
        }
        if submission
            .report
            .as_ref()
            .is_some_and(|s| s.len() > 1024 * 1024)
        {
            return Err(invalid("report_limit", "报告过大"));
        }
        if task.scene == Scene::CheckOnly {
            if !submission.proposals.is_empty() {
                return Err(invalid("read_only_task", "仅检查任务禁止任何模版写入"));
            }
            if submission
                .report
                .as_ref()
                .is_none_or(|s| s.trim().is_empty())
            {
                return Err(invalid("report", "仅检查任务须提供报告"));
            }
            return Ok(());
        }
        if submission.proposals.is_empty() || submission.proposals.len() > 256 {
            return Err(invalid("task_scope", "提交须有1至256个明确模版结果"));
        }
        let mut targets = BTreeSet::new();
        for proposal in &submission.proposals {
            proposal.content.validate()?;
            if proposal.label.trim().is_empty() {
                return Err(invalid("label", "模版名称不能为空"));
            }
            if let Some(id) = &proposal.template_id {
                if !targets.insert(id) {
                    return Err(invalid("task_scope", "目标重复"));
                }
                let target = task
                    .targets
                    .iter()
                    .find(|t| &t.template_id == id)
                    .ok_or_else(|| invalid("task_scope", "提交包含未授权目标"))?;
                if proposal.base_revision.as_ref() != Some(&target.revision.id) {
                    return Err(invalid("task_baseline", "提交基准与任务不一致"));
                }
                check_locks(
                    target
                        .buffer
                        .as_ref()
                        .map(|b| &b.content)
                        .unwrap_or(&target.revision.content),
                    &proposal.content,
                )?;
            } else if !task.allow_new || proposal.base_revision.is_some() {
                return Err(invalid("task_scope", "任务未授权新增或新建基准非法"));
            }
        }
        if require_current
            && ModelStore::new(&self.root).load()?.library_digest != task.expected_library
        {
            return Err(invalid(
                "stale_baseline",
                "任务基准已变化，请保留结果并重新比较",
            ));
        }
        Ok(())
    }
    pub fn validate_prepared(&self, task: &Task, submission: &Submission) -> ModelResult<()> {
        Self::active(task)?;
        self.validate(task, submission, true)
    }
    pub(super) fn validate_generated(
        &self,
        task: &Task,
        submission: &Submission,
    ) -> ModelResult<()> {
        Self::active(task)?;
        self.validate(task, submission, false)
    }
    pub fn preflight(&self, id: &str, token: &str, submission: &Submission) -> ModelResult<Value> {
        let task = self.context(id, token)?;
        self.validate(&task, submission, true)?;
        Ok(
            json!({"valid":true,"writes":task.scene != Scene::CheckOnly,"targets":submission.proposals.len(),"saved":false}),
        )
    }
    pub fn receipt(&self, id: &str, token: &str, submission: &str) -> ModelResult<Option<Receipt>> {
        let task = self.auth(id, token)?;
        self.receipt_for(&task, submission)
    }
    pub fn owner_result(&self, id: &str, submission: &str) -> ModelResult<Value> {
        let task = self.record(id)?.task;
        let receipt = self.receipt_for(&task, submission)?;
        Ok(json!({"task":task,"receipt":receipt}))
    }
    fn receipt_for(&self, task: &Task, submission: &str) -> ModelResult<Option<Receipt>> {
        let id = task.id.as_str();
        require_uuid(submission)?;
        let path = self
            .root
            .join(TASK_DIR)
            .join(format!("{id}-{submission}.report.json"));
        safety::check_path(&self.root, &path, true)?;
        let stored: Option<Receipt> = if path.exists() {
            Some(serde_json::from_slice(&safety::read_file(
                &path,
                2 * 1024 * 1024,
            )?)?)
        } else {
            None
        };
        let key = format!("{id}/{submission}");
        if let Some(value) = ModelStore::new(&self.root)
            .load()?
            .library
            .submission_receipts
            .get(&key)
        {
            if let Some(stored) = &stored {
                if stored.submission_id != submission
                    || stored.digest != value.digest
                    || stored.outcomes != value.outcomes
                {
                    return Err(invalid(
                        "receipt_mismatch",
                        "回执与实际保存记录不一致，结果待核对",
                    ));
                }
            }
            return Ok(Some(Receipt {
                submission_id: submission.into(),
                digest: value.digest.clone(),
                state: if value.outcomes.iter().any(|v| v.changed) {
                    "saved".into()
                } else {
                    "unchanged".into()
                },
                outcomes: value.outcomes.clone(),
                report: stored.and_then(|r| r.report),
            }));
        }
        if let Some(receipt) = &stored {
            if !((task.scene == Scene::CheckOnly && receipt.state == "report")
                || receipt.state == "rejected")
                || !receipt.outcomes.is_empty()
                || receipt.submission_id != submission
            {
                return Err(invalid("receipt_mismatch", "未找到对应的实际保存记录"));
            }
        }
        Ok(stored)
    }
    pub fn submit(
        &self,
        id: &str,
        token: &str,
        mut submission: Submission,
    ) -> ModelResult<Receipt> {
        for proposal in &mut submission.proposals {
            proposal.content.normalize();
            proposal.label = proposal.label.trim().into();
        }
        let path = self.path(id)?;
        let lock_path = crate::infrastructure::locking::lock_path_for(&path);
        safety::check_path(&self.root, &lock_path, true)?;
        let _guard = TargetFileLock::lock(&path)?;
        let task = self.context(id, token)?;
        require_uuid(&submission.submission_id)?;
        let digest = sha256(&serde_json::to_vec(&submission)?);
        if let Some(receipt) = self.receipt(id, token, &submission.submission_id)? {
            if receipt.digest != digest {
                return Err(invalid(
                    "submission_conflict",
                    "同一提交ID内容不同，拒绝重复提交",
                ));
            }
            if receipt.state == "rejected" {
                return Err(invalid(
                    "submission_rejected",
                    "该提交此前已被拒绝；修改内容须使用新的提交身份",
                ));
            }
            return Ok(receipt);
        }
        if let Err(error) = self.validate(&task, &submission, true) {
            let rejected = Receipt {
                submission_id: submission.submission_id.clone(),
                digest,
                state: "rejected".into(),
                outcomes: vec![],
                report: Some(error.code.clone()),
            };
            let path = self
                .root
                .join(TASK_DIR)
                .join(format!("{id}-{}.report.json", submission.submission_id));
            safety::check_path(&self.root, &path, true)?;
            atomic_write(&path, &serde_json::to_vec(&rejected)?, 0o600)?;
            return Err(error);
        }
        let receipt = if task.scene == Scene::CheckOnly {
            Receipt {
                submission_id: submission.submission_id.clone(),
                digest,
                state: "report".into(),
                outcomes: vec![],
                report: submission.report,
            }
        } else {
            let requests = submission
                .proposals
                .into_iter()
                .map(|p| SaveTemplate {
                    expected_library: task.expected_library.clone(),
                    template_id: p.template_id,
                    base_revision: p.base_revision,
                    label: p.label,
                    content: p.content,
                })
                .collect();
            let result = ModelStore::new(&self.root).save_submission(
                &task.expected_library,
                requests,
                Some((&format!("{id}/{}", submission.submission_id), &digest)),
            )?;
            Receipt {
                submission_id: submission.submission_id.clone(),
                digest,
                state: if result.outcomes.iter().any(|v| v.changed) {
                    "saved".into()
                } else {
                    "unchanged".into()
                },
                outcomes: result.outcomes,
                report: submission.report,
            }
        };
        let receipt_path = self
            .root
            .join(TASK_DIR)
            .join(format!("{id}-{}.report.json", submission.submission_id));
        safety::check_path(&self.root, &receipt_path, true)?;
        // Saved versions + identities are already recoverable from the library receipt
        // if this auxiliary report write fails. A caller must query before retrying.
        atomic_write(&receipt_path, &serde_json::to_vec(&receipt)?, 0o600)?;
        Ok(receipt)
    }
}
fn check_locks(before: &TemplateContent, after: &TemplateContent) -> ModelResult<()> {
    let a = serde_json::to_value(before)?;
    let b = serde_json::to_value(after)?;
    for field in &before.locked_fields {
        let pointer = format!("/{}", field.replace('.', "/"));
        if a.pointer(&pointer) != b.pointer(&pointer) {
            return Err(invalid("locked_field", "外部结果修改了人工锁定字段"));
        }
    }
    for variant in &before.variants {
        if variant.locked_fields.is_empty() {
            continue;
        }
        let next = after
            .variants
            .iter()
            .find(|v| v.id == variant.id)
            .ok_or_else(|| invalid("locked_field", "外部结果删除了人工锁定型号"))?;
        let a = serde_json::to_value(&variant.rules)?;
        let b = serde_json::to_value(&next.rules)?;
        for field in &variant.locked_fields {
            let pointer = format!("/{}", field.replace('.', "/"));
            if a.pointer(&pointer) != b.pointer(&pointer) {
                return Err(invalid("locked_field", "外部结果修改了型号人工锁定字段"));
            }
        }
    }
    if before.locked_fields != after.locked_fields
        || before.variants.iter().any(|v| {
            after
                .variants
                .iter()
                .find(|n| n.id == v.id)
                .is_some_and(|n| n.locked_fields != v.locked_fields)
        })
    {
        return Err(invalid("locked_field", "外部结果不能修改人工锁集合"));
    }
    Ok(())
}

pub fn contract_definition() -> Value {
    json!({"contractVersion":1,"submission":schemars::schema_for!(Submission),"rules":{"matching":"only adopted templates; highest-priority ties reject; longest literal raw-id suffix","inheritance":"explicit inherit/custom; unknown is not false","identity":"software assigns new template/revision identities","atomicity":"whole batch or no changes","checkOnly":"reports only; proposals are forbidden"}})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn create(scene: Scene) -> CreateTask {
        CreateTask {
            scene,
            targets: vec![],
            allow_new: true,
            requirements: String::new(),
            expected_library: None,
            buffers: vec![],
        }
    }
    fn proposal() -> Proposal {
        let mut content = TemplateContent::default();
        content.matching.include.push(Pattern {
            kind: MatchKind::Prefix,
            value: "custom-".into(),
        });
        Proposal {
            template_id: None,
            base_revision: None,
            label: "外部结果".into(),
            content,
        }
    }
    fn submission() -> Submission {
        Submission {
            submission_id: uuid::Uuid::new_v4().to_string(),
            contract_version: 1,
            proposals: vec![proposal()],
            report: None,
        }
    }
    #[test]
    fn external_submission_is_scoped_atomic_and_replays_after_restart_without_a_second_version() {
        let root = tempfile::tempdir().unwrap();
        let tasks = Tasks::new(root.path());
        let grant = tasks.create(create(Scene::FromSources)).unwrap();
        let request = submission();
        assert!(tasks.context(&grant.task.id, "wrong-token").is_err());
        let preflight = tasks
            .preflight(&grant.task.id, &grant.token, &request)
            .unwrap();
        assert_eq!(preflight["saved"], false);
        assert!(!root.path().join(LIBRARY_PATH).exists());
        let first = tasks
            .submit(&grant.task.id, &grant.token, request.clone())
            .unwrap();
        let again = Tasks::new(root.path())
            .submit(&grant.task.id, &grant.token, request.clone())
            .unwrap();
        assert_eq!(first.outcomes, again.outcomes);
        let library = ModelStore::new(root.path()).load().unwrap().library;
        assert_eq!(
            library.templates.len(),
            super::super::store::seed_library().templates.len() + 1
        );
        let mut changed = request;
        changed.proposals[0].label = "不同内容".into();
        assert_eq!(
            tasks
                .submit(&grant.task.id, &grant.token, changed)
                .unwrap_err()
                .code,
            "submission_conflict"
        );
    }
    #[test]
    fn check_only_service_rejects_any_version_write_and_only_records_a_report() {
        let root = tempfile::tempdir().unwrap();
        let tasks = Tasks::new(root.path());
        let library = ModelStore::new(root.path()).load().unwrap().library;
        let target = TemplateRef {
            template_id: library.templates[0].id.clone(),
            revision_id: library.templates[0].revisions[0].id.clone(),
        };
        let mut scope = create(Scene::CheckOnly);
        scope.targets = vec![target];
        scope.allow_new = false;
        let grant = tasks.create(scope).unwrap();
        let mut request = submission();
        assert_eq!(
            tasks
                .submit(&grant.task.id, &grant.token, request.clone())
                .unwrap_err()
                .code,
            "read_only_task"
        );
        request.submission_id = uuid::Uuid::new_v4().to_string();
        request.proposals.clear();
        request.report = Some("已检查，待补公开资料".into());
        let receipt = tasks
            .submit(&grant.task.id, &grant.token, request.clone())
            .unwrap();
        assert_eq!(receipt.state, "report");
        assert!(!root.path().join(LIBRARY_PATH).exists());
        assert_eq!(
            tasks
                .receipt(&grant.task.id, &grant.token, &request.submission_id)
                .unwrap()
                .unwrap(),
            receipt
        );
    }
    #[test]
    fn bad_batch_stale_baseline_and_revocation_never_partially_save() {
        let root = tempfile::tempdir().unwrap();
        let tasks = Tasks::new(root.path());
        let grant = tasks.create(create(Scene::FromSources)).unwrap();
        let mut request = submission();
        let mut invalid = proposal();
        invalid.content.matching.include.clear();
        request.proposals.push(invalid);
        assert!(tasks.submit(&grant.task.id, &grant.token, request).is_err());
        assert!(!root.path().join(LIBRARY_PATH).exists());
        tasks.revoke(&grant.task.id).unwrap();
        assert_eq!(
            tasks
                .submit(&grant.task.id, &grant.token, submission())
                .unwrap_err()
                .code,
            "task_authorization"
        );
    }
    #[test]
    fn advanced_reasoning_fields_use_the_same_external_lock_boundary() {
        let mut content = proposal().content;
        content.rules.pinned_effort = Field::Custom("high".into());
        content.rules.effort_map = Field::Custom(std::collections::BTreeMap::from([(
            "high".into(),
            "__omit__".into(),
        )]));
        content.locked_fields.insert("rules.pinnedEffort".into());
        content.locked_fields.insert("rules.effortMap".into());
        content.validate().unwrap();
        let mut changed = content.clone();
        changed.rules.pinned_effort = Field::Custom("low".into());
        assert_eq!(
            super::check_locks(&content, &changed).unwrap_err().code,
            "locked_field"
        );
        let mut changed = content.clone();
        changed.rules.effort_map = Field::Custom(Default::default());
        assert_eq!(
            super::check_locks(&content, &changed).unwrap_err().code,
            "locked_field"
        );
    }
    #[test]
    fn external_proposals_cannot_change_locked_values_or_expand_scope() {
        let root = tempfile::tempdir().unwrap();
        let store = ModelStore::new(root.path());
        let before = store.load().unwrap();
        let mut original = proposal();
        original.content.rules.fast = Field::Custom(false);
        original.content.locked_fields.insert("rules.fast".into());
        let saved = store
            .save(SaveTemplate {
                expected_library: before.library_digest,
                template_id: None,
                base_revision: None,
                label: original.label.clone(),
                content: original.content.clone(),
            })
            .unwrap();
        let tasks = Tasks::new(root.path());
        let mut scope = create(Scene::AuditAndRevise);
        scope.allow_new = false;
        scope.targets.push(TemplateRef {
            template_id: saved.template_id.clone(),
            revision_id: saved.revision_id.clone(),
        });
        let grant = tasks.create(scope).unwrap();
        let mut request = submission();
        request.proposals[0] = original;
        request.proposals[0].template_id = Some(saved.template_id);
        request.proposals[0].base_revision = Some(saved.revision_id);
        request.proposals[0].content.rules.fast = Field::Custom(true);
        assert_eq!(
            tasks
                .submit(&grant.task.id, &grant.token, request)
                .unwrap_err()
                .code,
            "locked_field"
        );
        assert_eq!(store.load().unwrap(), saved.snapshot);
    }
}
