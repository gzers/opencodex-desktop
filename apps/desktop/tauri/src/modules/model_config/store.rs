use super::{matching, schema::*};
use crate::infrastructure::{atomic_write::atomic_write, locking::TargetFileLock};
use crate::modules::backup::{backup_file, safety, BackupAction};
use chrono::{SecondsFormat, Utc};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

const MAX_DOCUMENT: u64 = 16 * 1024 * 1024;
const TRANSACTION_PATH: &str = "manager-state/model-config-transaction";
const JOURNAL_PATH: &str = "manager-state/model-config-journal.json";
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PendingFiles {
    expected_policy: String,
    #[serde(default)]
    runtime_commit: bool,
    files: Vec<PendingFile>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PendingFile {
    path: String,
    before: Option<Vec<u8>>,
    after: Vec<u8>,
}
// The release catalog owns fixed full identities and provenance. Production
// never imports the documentation tree or expands display names into API IDs.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SeedCatalog {
    format: String,
    source_sha256: String,
    reviewed_at: String,
    families: Vec<SeedFamily>,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SeedFamily {
    id: String,
    revision_id: String,
    label: String,
    content: TemplateContent,
}

pub fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
/// Typed v1 canonical content: JSON object keys are sorted by serde_json::Map;
/// only integer numeric fields are admitted; sets normalize before serialization.
/// This is the model-content-v1 contract, not a claim of general RFC8785 support.
pub fn content_digest(label: &str, content: &TemplateContent) -> ModelResult<String> {
    let mut normalized = content.clone();
    normalized.normalize();
    Ok(sha256(&serde_json::to_vec(
        &json!({"label": label, "content": normalized}),
    )?))
}
pub fn seed_library() -> Library {
    static CATALOG: std::sync::OnceLock<Library> = std::sync::OnceLock::new();
    CATALOG
        .get_or_init(|| {
            let catalog: SeedCatalog = serde_json::from_str(include_str!("seed-catalog.json"))
                .expect("validated release seed catalog");
            assert_eq!(catalog.format, "ocxd-builtin-models-v1");
            assert_eq!(catalog.source_sha256.len(), 64);
            let created_at = format!("{}T00:00:00Z", catalog.reviewed_at);
            Library {
                schema_version: SCHEMA_VERSION,
                submission_receipts: Default::default(),
                templates: catalog
                    .families
                    .into_iter()
                    .map(|seed| {
                        let mut content = seed.content;
                        content.normalize();
                        content.validate().expect("validated seed content");
                        let digest = content_digest(&seed.label, &content)
                            .expect("static seed is serializable");
                        Template {
                            id: seed.id,
                            label: seed.label.clone(),
                            source: TemplateSource::Builtin,
                            default_revision: Some(seed.revision_id.clone()),
                            revisions: vec![Revision {
                                id: seed.revision_id,
                                created_at: created_at.clone(),
                                saved_label: seed.label,
                                content,
                                digest,
                                base_revision: None,
                                changes: vec![],
                                origin: SaveOrigin::Builtin,
                            }],
                        }
                    })
                    .collect(),
            }
        })
        .clone()
}

pub fn validate_library(library: &Library) -> ModelResult<()> {
    if library.schema_version != SCHEMA_VERSION {
        return Err(invalid("schema_version", "不支持的模版库结构版本"));
    }
    let trusted = seed_library();
    let mut templates = BTreeSet::new();
    let mut revisions = BTreeSet::new();
    for template in &library.templates {
        require_uuid(&template.id)?;
        if !templates.insert(&template.id)
            || template.label.trim().is_empty()
            || template.label.len() > 256
            || template.revisions.is_empty()
        {
            return Err(invalid("template", "模版身份重复、名称无效或没有保存版本"));
        }
        let baseline = trusted.templates.iter().find(|t| t.id == template.id);
        if template.source == TemplateSource::Builtin {
            let seed = baseline.ok_or_else(|| invalid("source", "无法核实内置来源"))?;
            if !template.revisions.contains(&seed.revisions[0]) {
                return Err(invalid("baseline", "内置初始基线缺失或被改写"));
            }
        } else if baseline.is_some() {
            return Err(invalid("source", "可信内置身份不能伪装为自建"));
        }
        for value in &template.revisions {
            require_uuid(&value.id)?;
            if !revisions.insert(&value.id) {
                return Err(invalid("revision_conflict", "保存版本身份重复"));
            }
            value.content.validate()?;
            if content_digest(&value.saved_label, &value.content)? != value.digest {
                return Err(invalid("digest_mismatch", "保存版本内容与摘要不一致"));
            }
            if chrono::DateTime::parse_from_rfc3339(&value.created_at).is_err() {
                return Err(invalid("timestamp", "保存时间无效"));
            }
        }
        if let Some(default) = &template.default_revision {
            if !template.revisions.iter().any(|r| &r.id == default) {
                return Err(invalid("default_revision", "默认指针缺少精确版本"));
            }
        }
    }
    for seed in &trusted.templates {
        if !library.templates.iter().any(|t| t.id == seed.id) {
            return Err(invalid("baseline", "可信内置模版缺失"));
        }
    }
    Ok(())
}
fn validate_policy(library: &Library, policy: &Policy) -> ModelResult<()> {
    if policy.schema_version != SCHEMA_VERSION {
        return Err(invalid("schema_version", "不支持的策略结构版本"));
    }
    let mut channel_ids = BTreeSet::new();
    for channel in &policy.channels {
        if channel.id.is_empty() || !channel_ids.insert(&channel.id) {
            return Err(invalid("channel", "渠道身份为空或重复"));
        }
    }
    for refs in std::iter::once(&policy.template_refs)
        .chain(policy.channels.iter().map(|c| &c.template_refs))
    {
        let mut ids = BTreeSet::new();
        for reference in refs {
            if !ids.insert(&reference.template_id) {
                return Err(invalid("reference", "同一作用域重复采用模版"));
            }
            matching::revision(library, reference)?;
        }
    }
    for channel in &policy.channels {
        for reference in channel.model_refs.values() {
            matching::revision(library, reference)?;
        }
    }
    Ok(())
}

pub struct ModelStore {
    root: PathBuf,
}
impl ModelStore {
    pub fn new(root: &Path) -> Self {
        Self { root: root.into() }
    }
    fn path(&self, relative: &str) -> ModelResult<PathBuf> {
        let path = self.root.join(relative);
        safety::check_path(&self.root, &path, true)?;
        Ok(path)
    }
    fn lock(&self) -> ModelResult<TargetFileLock> {
        let path = self.path(TRANSACTION_PATH)?;
        let lock_path = crate::infrastructure::locking::lock_path_for(&path);
        safety::check_path(&self.root, &lock_path, true)?;
        let guard = TargetFileLock::lock(&path)?;
        safety::check_path(&self.root, &lock_path, false)?;
        safety::check_path(&self.root, &path, true)?;
        Ok(guard)
    }
    fn bytes(&self, relative: &str) -> ModelResult<Option<Vec<u8>>> {
        let path = self.path(relative)?;
        match std::fs::symlink_metadata(&path) {
            Ok(_) => Ok(Some(safety::read_file(&path, MAX_DOCUMENT)?)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }
    fn load_unlocked(&self) -> ModelResult<(Snapshot, Policy)> {
        self.recover_pending()?;
        let bytes = self.bytes(LIBRARY_PATH)?;
        let library = match &bytes {
            Some(bytes) => serde_json::from_slice(bytes)?,
            None => seed_library(),
        };
        validate_library(&library)?;
        let policy_bytes = self.bytes(POLICY_PATH)?;
        let policy = match &policy_bytes {
            Some(bytes) => serde_json::from_slice(bytes)?,
            None => Policy::default(),
        };
        validate_policy(&library, &policy)?;
        let workspace_bytes = self.bytes(DRAFT_PATH)?;
        let workspace: Option<PolicyDraft> = workspace_bytes
            .as_ref()
            .map(|bytes| serde_json::from_slice(bytes))
            .transpose()?;
        Ok((
            Snapshot {
                library,
                policy: policy.clone(),
                library_digest: sha256(bytes.as_deref().unwrap_or_default()),
                policy_digest: sha256(policy_bytes.as_deref().unwrap_or_default()),
                workspace,
                workspace_digest: sha256(workspace_bytes.as_deref().unwrap_or_default()),
            },
            policy,
        ))
    }
    fn recover_pending(&self) -> ModelResult<()> {
        let path = self.path(JOURNAL_PATH)?;
        if !path.exists() {
            return Ok(());
        }
        let journal: PendingFiles =
            serde_json::from_slice(&safety::read_file(&path, 128 * 1024 * 1024)?)?;
        if journal.files.is_empty()
            || journal.files.len() > 2
            || journal.files.iter().any(|f| {
                !(if journal.runtime_commit {
                    [POLICY_PATH, DRAFT_PATH]
                } else {
                    [LIBRARY_PATH, DRAFT_PATH]
                })
                .contains(&f.path.as_str())
                    || f.after.len() as u64 > MAX_DOCUMENT
                    || f.before
                        .as_ref()
                        .is_some_and(|bytes| bytes.len() as u64 > MAX_DOCUMENT)
            })
            || journal
                .files
                .iter()
                .map(|f| &f.path)
                .collect::<BTreeSet<_>>()
                .len()
                != journal.files.len()
        {
            return Err(invalid("journal", "模型导入事务记录无效，未覆盖任何文件"));
        }
        let current_policy = self.bytes(POLICY_PATH)?;
        let intended_policy = journal.files.iter().find(|f| f.path == POLICY_PATH);
        if sha256(current_policy.as_deref().unwrap_or_default()) != journal.expected_policy
            && intended_policy.is_none_or(|f| current_policy.as_deref() != Some(f.after.as_slice()))
        {
            return Err(invalid(
                "journal_drift",
                "未完成事务期间当前策略改变，请保留事务并核对",
            ));
        }
        for file in &journal.files {
            let current = self.bytes(&file.path)?;
            if current != file.before && current.as_deref() != Some(file.after.as_slice()) {
                return Err(invalid(
                    "journal_drift",
                    "未完成事务检测到后续修改，拒绝覆盖",
                ));
            }
        }
        // Validate the complete intended pair before continuing any interrupted
        // write. A broken journal is diagnostic evidence, never an initializer.
        let library: Library = match journal.files.iter().find(|f| f.path == LIBRARY_PATH) {
            Some(file) => serde_json::from_slice(&file.after)?,
            None => self
                .bytes(LIBRARY_PATH)?
                .map(|b| serde_json::from_slice(&b))
                .transpose()?
                .unwrap_or_else(seed_library),
        };
        validate_library(&library)?;
        let workspace_base = if journal.runtime_commit {
            let policy_file =
                intended_policy.ok_or_else(|| invalid("journal", "运行提交事务缺少策略产物"))?;
            let policy: Policy = serde_json::from_slice(&policy_file.after)?;
            validate_policy(&library, &policy)?;
            super::compiler::project(&library, &policy)?;
            sha256(&policy_file.after)
        } else {
            journal.expected_policy.clone()
        };
        if let Some(file) = journal.files.iter().find(|f| f.path == DRAFT_PATH) {
            let workspace: PolicyDraft = serde_json::from_slice(&file.after)?;
            if workspace.base_policy != workspace_base {
                return Err(invalid("journal", "导入事务的草稿基准不一致"));
            }
            validate_policy(&library, &workspace.policy)?;
        }
        for file in &journal.files {
            let target = self.path(&file.path)?;
            if self.bytes(&file.path)?.as_deref() != Some(file.after.as_slice()) {
                atomic_write(&target, &file.after, 0o600)?;
            }
            if self.bytes(&file.path)?.as_deref() != Some(file.after.as_slice()) {
                return Err(invalid("journal_readback", "事务恢复读回未完成"));
            }
        }
        std::fs::remove_file(path)?;
        Ok(())
    }
    pub fn accept_import(
        &self,
        expected_library: &str,
        expected_policy: &str,
        expected_workspace: &str,
        library: Library,
        policy: Option<Policy>,
    ) -> ModelResult<Snapshot> {
        let _guard = self.lock()?;
        let (snapshot, _) = self.load_unlocked()?;
        if snapshot.library_digest != expected_library
            || snapshot.policy_digest != expected_policy
            || snapshot.workspace_digest != expected_workspace
        {
            return Err(invalid(
                "stale_baseline",
                "导入确认后本地内容已变化，未写入",
            ));
        }
        validate_library(&library)?;
        let mut files = vec![];
        let library_bytes = serde_json::to_vec_pretty(&library)?;
        if library != snapshot.library {
            files.push(PendingFile {
                path: LIBRARY_PATH.into(),
                before: self.bytes(LIBRARY_PATH)?,
                after: library_bytes,
            });
        }
        if let Some(policy) = policy {
            validate_policy(&library, &policy)?;
            let workspace = PolicyDraft {
                base_policy: expected_policy.into(),
                policy,
            };
            files.push(PendingFile {
                path: DRAFT_PATH.into(),
                before: self.bytes(DRAFT_PATH)?,
                after: serde_json::to_vec_pretty(&workspace)?,
            });
        }
        if files.is_empty() {
            return Ok(snapshot);
        }
        for file in &files {
            if file.after.len() as u64 > MAX_DOCUMENT {
                return Err(invalid("limit", "导入产物超过文件上限"));
            }
            if let Some(before) = &file.before {
                backup_file(
                    &self.root,
                    BackupAction::ModelConfig,
                    &self.path(&file.path)?,
                    before,
                    Utc::now(),
                    Some("接受模型导入前保护".into()),
                )?;
            }
        }
        let journal = PendingFiles {
            expected_policy: expected_policy.into(),
            runtime_commit: false,
            files,
        };
        atomic_write(
            &self.path(JOURNAL_PATH)?,
            &serde_json::to_vec(&journal)?,
            0o600,
        )?;
        self.recover_pending()?;
        self.load_unlocked().map(|v| v.0)
    }
    pub fn load(&self) -> ModelResult<Snapshot> {
        let _guard = self.lock()?;
        Ok(self.load_unlocked()?.0)
    }
    fn persist(&self, library: &Library, expected: &Snapshot) -> ModelResult<Snapshot> {
        validate_library(library)?;
        let before = self.bytes(LIBRARY_PATH)?;
        if sha256(before.as_deref().unwrap_or_default()) != expected.library_digest
            || sha256(self.bytes(POLICY_PATH)?.as_deref().unwrap_or_default())
                != expected.policy_digest
            || sha256(self.bytes(DRAFT_PATH)?.as_deref().unwrap_or_default())
                != expected.workspace_digest
        {
            return Err(invalid(
                "stale_baseline",
                "配置已在其他位置修改，请重新读取并比较",
            ));
        }
        let path = self.path(LIBRARY_PATH)?;
        if let Some(before) = before {
            backup_file(
                &self.root,
                BackupAction::ModelConfig,
                &path,
                &before,
                Utc::now(),
                Some("模型模版保存前保护".into()),
            )?;
        }
        let bytes = serde_json::to_vec_pretty(library)?;
        if bytes.len() as u64 > MAX_DOCUMENT {
            return Err(invalid("limit", "模版库超过文件大小上限"));
        }
        // Recheck after the backup operation, before replacing a concurrent external edit.
        if sha256(self.bytes(LIBRARY_PATH)?.as_deref().unwrap_or_default())
            != expected.library_digest
            || sha256(self.bytes(POLICY_PATH)?.as_deref().unwrap_or_default())
                != expected.policy_digest
        {
            return Err(invalid("stale_baseline", "备份期间检测到外部修改，未覆盖"));
        }
        atomic_write(&path, &bytes, 0o600)?;
        let actual = self
            .bytes(LIBRARY_PATH)?
            .ok_or_else(|| invalid("readback", "保存后文件不可读取，结果待核对"))?;
        if actual != bytes {
            return Err(invalid("readback", "保存后内容已变化，结果待核对"));
        }
        Ok(Snapshot {
            library: library.clone(),
            policy: expected.policy.clone(),
            library_digest: sha256(&bytes),
            policy_digest: expected.policy_digest.clone(),
            workspace: expected.workspace.clone(),
            workspace_digest: expected.workspace_digest.clone(),
        })
    }
    pub fn save(&self, request: SaveTemplate) -> ModelResult<Commit> {
        let expected = request.expected_library.clone();
        let result = self.save_batch(&expected, vec![request])?;
        let identity = result
            .outcomes
            .into_iter()
            .next()
            .ok_or_else(|| invalid("batch", "保存未返回结果，请重新读取核对"))?;
        Ok(Commit {
            snapshot: result.snapshot,
            template_id: identity.template_id,
            revision_id: identity.revision_id,
            changed: identity.changed,
        })
    }
    /// Incomplete editor values persist locally; they are never published or synced.
    pub fn stage_policy(
        &self,
        expected_policy: &str,
        expected_workspace: &str,
        policy: Policy,
    ) -> ModelResult<Snapshot> {
        let _guard = self.lock()?;
        let (mut snapshot, _) = self.load_unlocked()?;
        if snapshot.policy_digest != expected_policy
            || snapshot.workspace_digest != expected_workspace
        {
            return Err(invalid(
                "stale_workspace",
                "渠道工作区已变化，当前输入保留，请比较后继续",
            ));
        }
        if policy.schema_version != SCHEMA_VERSION {
            return Err(invalid("schema_version", "草稿结构版本无效"));
        }
        for reference in policy.template_refs.iter().chain(
            policy
                .channels
                .iter()
                .flat_map(|c| c.template_refs.iter().chain(c.model_refs.values())),
        ) {
            matching::revision(&snapshot.library, reference)?;
        }
        let workspace = PolicyDraft {
            base_policy: expected_policy.into(),
            policy,
        };
        let bytes = serde_json::to_vec_pretty(&workspace)?;
        if bytes.len() as u64 > MAX_DOCUMENT {
            return Err(invalid("limit", "渠道草稿超过文件大小上限"));
        }
        let path = self.path(DRAFT_PATH)?;
        atomic_write(&path, &bytes, 0o600)?;
        if self.bytes(DRAFT_PATH)?.as_deref() != Some(bytes.as_slice()) {
            return Err(invalid("readback", "草稿保存后待核对"));
        }
        snapshot.workspace = Some(workspace);
        snapshot.workspace_digest = sha256(&bytes);
        Ok(snapshot)
    }
    pub fn save_batch(
        &self,
        expected: &str,
        requests: Vec<SaveTemplate>,
    ) -> ModelResult<BatchCommit> {
        self.save_submission(expected, requests, None)
    }
    pub fn save_submission(
        &self,
        expected: &str,
        requests: Vec<SaveTemplate>,
        submission: Option<(&str, &str)>,
    ) -> ModelResult<BatchCommit> {
        self.save_with_origin(expected, requests, submission, None)
    }
    pub(super) fn save_internal_review(
        &self,
        expected: &str,
        requests: Vec<SaveTemplate>,
        operation: &str,
        digest: &str,
        run_id: &str,
    ) -> ModelResult<BatchCommit> {
        require_uuid(run_id)?;
        self.save_with_origin(
            expected,
            requests,
            Some((operation, digest)),
            Some(SaveOrigin::InternalAgent(run_id.into())),
        )
    }
    fn save_with_origin(
        &self,
        expected: &str,
        requests: Vec<SaveTemplate>,
        submission: Option<(&str, &str)>,
        origin: Option<SaveOrigin>,
    ) -> ModelResult<BatchCommit> {
        let _guard = self.lock()?;
        let (snapshot, _) = self.load_unlocked()?;
        if let Some((id, digest)) = submission {
            if let Some(receipt) = snapshot.library.submission_receipts.get(id) {
                if receipt.digest != digest {
                    return Err(invalid(
                        "submission_conflict",
                        "同一提交身份的内容不同，已拒绝重放",
                    ));
                }
                return Ok(BatchCommit {
                    outcomes: receipt.outcomes.clone(),
                    snapshot,
                });
            }
        }
        if expected != snapshot.library_digest
            || requests.iter().any(|r| r.expected_library != expected)
        {
            return Err(invalid(
                "stale_baseline",
                "模版基准已变化，请先比较最新内容",
            ));
        }
        if requests.is_empty() || requests.len() > 256 {
            return Err(invalid("batch", "批次须包含1至256个明确目标"));
        }
        let mut targets = BTreeSet::new();
        let mut library = snapshot.library.clone();
        let mut outcomes = vec![];
        for request in requests {
            if request
                .template_id
                .as_ref()
                .is_some_and(|id| !targets.insert(id.clone()))
            {
                return Err(invalid("batch", "同一模版不能在同一批次重复出现"));
            }
            outcomes.push(prepare_template(&mut library, request)?);
        }
        if let Some((id, _)) = submission {
            for outcome in &outcomes {
                if outcome.changed {
                    if let Some(revision) = library
                        .templates
                        .iter_mut()
                        .find(|t| t.id == outcome.template_id)
                        .and_then(|t| t.revisions.iter_mut().find(|r| r.id == outcome.revision_id))
                    {
                        revision.origin = origin.clone().unwrap_or_else(|| {
                            SaveOrigin::ExternalTask(id.split('/').next().unwrap_or(id).into())
                        });
                    }
                }
            }
        }
        if let Some((id, digest)) = submission {
            library.submission_receipts.insert(
                id.into(),
                SubmissionReceipt {
                    digest: digest.into(),
                    outcomes: outcomes.clone(),
                    saved_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                },
            );
        }
        let snapshot = if submission.is_some() || outcomes.iter().any(|v| v.changed) {
            self.persist(&library, &snapshot)?
        } else {
            snapshot
        };
        Ok(BatchCommit { snapshot, outcomes })
    }
    pub fn commit_runtime_policy(
        &self,
        expected_library: &str,
        expected_policy: &str,
        expected_workspace: &str,
        policy: Policy,
    ) -> ModelResult<Snapshot> {
        let _guard = self.lock()?;
        let (snapshot, _) = self.load_unlocked()?;
        let bytes = serde_json::to_vec_pretty(&policy)?;
        // A verified runtime result may be reconciled after the local pair was
        // committed but its plan receipt was not. Preserve any later editor.
        if snapshot.policy == policy && snapshot.policy_digest == sha256(&bytes) {
            return Ok(snapshot);
        }
        if snapshot.library_digest != expected_library
            || snapshot.policy_digest != expected_policy
            || snapshot.workspace_digest != expected_workspace
        {
            return Err(invalid(
                "stale_baseline",
                "运行写入后本地基准已变化，保留计划待核对",
            ));
        }
        validate_policy(&snapshot.library, &policy)?;
        super::compiler::project(&snapshot.library, &policy)?;
        let path = self.path(POLICY_PATH)?;
        if let Some(before) = self.bytes(POLICY_PATH)? {
            backup_file(
                &self.root,
                BackupAction::ModelConfig,
                &path,
                &before,
                Utc::now(),
                Some("渠道策略提交前保护".into()),
            )?;
        }
        if bytes.len() as u64 > MAX_DOCUMENT {
            return Err(invalid("limit", "渠道策略超过文件上限"));
        }
        if self.load_unlocked()?.0.library_digest != expected_library
            || sha256(self.bytes(POLICY_PATH)?.as_deref().unwrap_or_default()) != expected_policy
            || sha256(self.bytes(DRAFT_PATH)?.as_deref().unwrap_or_default()) != expected_workspace
        {
            return Err(invalid("stale_baseline", "写前本地基准已变化，未覆盖"));
        }
        let workspace = PolicyDraft {
            base_policy: sha256(&bytes),
            policy,
        };
        let workspace_bytes = serde_json::to_vec_pretty(&workspace)?;
        let journal = PendingFiles {
            expected_policy: expected_policy.into(),
            runtime_commit: true,
            files: vec![
                PendingFile {
                    path: POLICY_PATH.into(),
                    before: self.bytes(POLICY_PATH)?,
                    after: bytes,
                },
                PendingFile {
                    path: DRAFT_PATH.into(),
                    before: self.bytes(DRAFT_PATH)?,
                    after: workspace_bytes,
                },
            ],
        };
        atomic_write(
            &self.path(JOURNAL_PATH)?,
            &serde_json::to_vec(&journal)?,
            0o600,
        )?;
        self.recover_pending()?;
        self.load_unlocked().map(|v| v.0)
    }
    pub fn set_default(
        &self,
        expected: &str,
        template: &str,
        revision: &str,
    ) -> ModelResult<Snapshot> {
        let _guard = self.lock()?;
        let (snapshot, _) = self.load_unlocked()?;
        if expected != snapshot.library_digest {
            return Err(invalid("stale_baseline", "默认设置基准已变化"));
        }
        let mut library = snapshot.library.clone();
        let target = library
            .templates
            .iter_mut()
            .find(|t| t.id == template)
            .ok_or_else(|| invalid("not_found", "模版不存在"))?;
        if !target.revisions.iter().any(|r| r.id == revision) {
            return Err(invalid("not_found", "版本不存在"));
        }
        if target.default_revision.as_deref() == Some(revision) {
            return Ok(snapshot);
        }
        target.default_revision = Some(revision.into());
        self.persist(&library, &snapshot)
    }
    pub fn delete(
        &self,
        expected_library: &str,
        expected_policy: &str,
        expected_workspace: &str,
        template: &str,
        revisions: &[String],
    ) -> ModelResult<Snapshot> {
        let _guard = self.lock()?;
        let (snapshot, policy) = self.load_unlocked()?;
        if expected_library != snapshot.library_digest
            || expected_policy != snapshot.policy_digest
            || expected_workspace != snapshot.workspace_digest
        {
            return Err(invalid("stale_baseline", "删除确认已过期，请重新查看依赖"));
        }
        let ids: BTreeSet<_> = revisions.iter().cloned().collect();
        if ids.is_empty() || ids.len() != revisions.len() {
            return Err(invalid("selection", "请选择明确且不重复的版本"));
        }
        let mut library = snapshot.library.clone();
        let target = library
            .templates
            .iter_mut()
            .find(|t| t.id == template)
            .ok_or_else(|| invalid("not_found", "模版不存在"))?;
        for id in &ids {
            if !target.revisions.iter().any(|r| &r.id == id) {
                return Err(invalid("not_found", "所选版本不存在，整批未删除"));
            }
            if target.default_revision.as_ref() == Some(id)
                || seed_library()
                    .templates
                    .iter()
                    .any(|t| t.id == template && t.revisions.iter().any(|r| r.id == *id))
            {
                return Err(invalid(
                    "protected_revision",
                    "内置基线或默认版本不能删除，整批未删除",
                ));
            }
            let used = policy
                .template_refs
                .iter()
                .chain(
                    policy
                        .channels
                        .iter()
                        .flat_map(|c| c.template_refs.iter().chain(c.model_refs.values())),
                )
                .any(|r| r.template_id == template && r.revision_id == *id);
            let draft_used = snapshot.workspace.as_ref().is_some_and(|workspace| {
                workspace
                    .policy
                    .template_refs
                    .iter()
                    .chain(
                        workspace
                            .policy
                            .channels
                            .iter()
                            .flat_map(|c| c.template_refs.iter().chain(c.model_refs.values())),
                    )
                    .any(|r| r.template_id == template && r.revision_id == *id)
            });
            if used || draft_used || super::runtime::recovery_uses(&self.root, template, id)? {
                return Err(invalid(
                    "protected_revision",
                    "保存策略仍引用所选版本，整批未删除",
                ));
            }
        }
        target.revisions.retain(|r| !ids.contains(&r.id));
        library.templates.retain(|t| !t.revisions.is_empty());
        self.persist(&library, &snapshot)
    }
}

fn prepare_template(
    library: &mut Library,
    mut request: SaveTemplate,
) -> ModelResult<CommitIdentity> {
    request.label = request.label.trim().into();
    if request.label.is_empty() || request.label.len() > 256 {
        return Err(invalid("label", "名称须为1至256字节"));
    }
    request.content.normalize();
    request.content.validate()?;
    let digest = content_digest(&request.label, &request.content)?;
    let index = if let Some(id) = &request.template_id {
        library
            .templates
            .iter()
            .position(|t| &t.id == id)
            .ok_or_else(|| invalid("not_found", "模版已不存在"))?
    } else {
        if request.base_revision.is_some() {
            return Err(invalid("base_revision", "新模版不能指定其他模版的版本基准"));
        }
        let mut id = uuid::Uuid::new_v4().to_string();
        while library.templates.iter().any(|t| t.id == id) {
            id = uuid::Uuid::new_v4().to_string();
        }
        library.templates.push(Template {
            id,
            label: request.label.clone(),
            source: TemplateSource::User,
            default_revision: None,
            revisions: vec![],
        });
        library.templates.len() - 1
    };
    let template_id = library.templates[index].id.clone();
    let base = if request.template_id.is_some() {
        Some(
            library.templates[index]
                .revisions
                .iter()
                .find(|r| Some(&r.id) == request.base_revision.as_ref())
                .cloned()
                .ok_or_else(|| invalid("base_revision", "编辑基准版本不存在"))?,
        )
    } else {
        None
    };
    if let Some(base) = &base {
        if base.digest == digest {
            return Ok(CommitIdentity {
                template_id,
                revision_id: base.id.clone(),
                changed: false,
            });
        }
    }
    let facts = change_facts(base.as_ref(), &request.label, &request.content)?;
    let mut revision_id = uuid::Uuid::new_v4().to_string();
    while library
        .templates
        .iter()
        .flat_map(|t| &t.revisions)
        .any(|r| r.id == revision_id)
    {
        revision_id = uuid::Uuid::new_v4().to_string();
    }
    library.templates[index].label = request.label.clone();
    library.templates[index].revisions.push(Revision {
        id: revision_id.clone(),
        created_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        saved_label: request.label,
        content: request.content,
        digest,
        base_revision: request.base_revision,
        changes: facts,
        origin: SaveOrigin::Manual,
    });
    Ok(CommitIdentity {
        template_id,
        revision_id,
        changed: true,
    })
}

fn change_facts(
    base: Option<&Revision>,
    label: &str,
    content: &TemplateContent,
) -> ModelResult<Vec<ChangeFact>> {
    let old = base.map(|r| json!({"名称": r.saved_label, "说明": r.content.description, "匹配": r.content.matching, "通用配置": r.content.rules, "型号规则": r.content.variants, "人工锁": r.content.locked_fields, "来源": r.content.sources})).unwrap_or(Value::Null);
    let new = json!({"名称": label, "说明": content.description, "匹配": content.matching, "通用配置": content.rules, "型号规则": content.variants, "人工锁": content.locked_fields, "来源": content.sources});
    Ok([
        "名称",
        "说明",
        "匹配",
        "通用配置",
        "型号规则",
        "人工锁",
        "来源",
    ]
    .iter()
    .filter(|key| old.get(**key) != new.get(**key))
    .map(|key| ChangeFact {
        field: (*key).into(),
        before: old.get(*key).cloned().unwrap_or(Value::Null),
        after: new.get(*key).cloned().unwrap_or(Value::Null),
    })
    .collect())
}

#[cfg(test)]
mod journal_tests {
    use super::*;
    #[test]
    fn release_seeds_are_exact_sourced_unadopted_and_do_not_promote_unknown_alias_capabilities() {
        let library = seed_library();
        validate_library(&library).unwrap();
        assert_eq!(library.templates.len(), 17);
        for family in &library.templates {
            require_uuid(&family.id).unwrap();
            let revision = &family.revisions[0];
            require_uuid(&revision.id).unwrap();
            assert!(revision
                .content
                .matching
                .include
                .iter()
                .all(|pattern| pattern.kind == MatchKind::Exact));
            assert!(!revision.content.sources.is_empty());
            assert_eq!(revision.content.rules, ModelRules::default());
            assert!(revision
                .content
                .variants
                .iter()
                .all(|variant| variant.rules.fast == Field::Inherit));
        }
        let gpt = &library.templates[0].revisions[0].content;
        assert!(
            !matching::preview(gpt, "any", "gpt-unresearched")
                .unwrap()
                .matched
        );
        assert_eq!(
            matching::preview(gpt, "any", "gpt-6.1-sol")
                .unwrap()
                .rules
                .context_window,
            Field::Custom(1_050_000)
        );
        let mistral = library
            .templates
            .iter()
            .find(|t| t.label.contains("Mistral"))
            .unwrap();
        assert!(mistral.revisions[0].content.variants.is_empty());
        assert!(
            matching::resolve(&library, &Policy::default(), "any", "gpt-6.1-sol")
                .unwrap()
                .is_none()
        );
    }
    fn fixture(root: &Path) -> PendingFiles {
        let mut library = seed_library();
        let mut user = library.templates[0].clone();
        user.id = uuid::Uuid::new_v4().to_string();
        user.source = TemplateSource::User;
        user.default_revision = None;
        user.revisions[0].id = uuid::Uuid::new_v4().to_string();
        user.revisions[0].origin = SaveOrigin::Manual;
        library.templates.push(user.clone());
        let policy = Policy {
            template_refs: vec![TemplateRef {
                template_id: user.id,
                revision_id: user.revisions[0].id.clone(),
            }],
            ..Default::default()
        };
        let journal = PendingFiles {
            expected_policy: sha256(b""),
            runtime_commit: false,
            files: vec![
                PendingFile {
                    path: LIBRARY_PATH.into(),
                    before: None,
                    after: serde_json::to_vec(&library).unwrap(),
                },
                PendingFile {
                    path: DRAFT_PATH.into(),
                    before: None,
                    after: serde_json::to_vec(&PolicyDraft {
                        base_policy: sha256(b""),
                        policy,
                    })
                    .unwrap(),
                },
            ],
        };
        let store = ModelStore::new(root);
        atomic_write(
            &store.path(JOURNAL_PATH).unwrap(),
            &serde_json::to_vec(&journal).unwrap(),
            0o600,
        )
        .unwrap();
        journal
    }
    #[test]
    fn interrupted_runtime_policy_and_draft_pair_recovers_after_the_first_file() {
        let root = tempfile::tempdir().unwrap();
        let store = ModelStore::new(root.path());
        let library = seed_library();
        let policy = Policy {
            template_refs: vec![TemplateRef {
                template_id: library.templates[0].id.clone(),
                revision_id: library.templates[0].revisions[0].id.clone(),
            }],
            ..Default::default()
        };
        let bytes = serde_json::to_vec_pretty(&policy).unwrap();
        let draft = PolicyDraft {
            base_policy: sha256(&bytes),
            policy: policy.clone(),
        };
        let journal = PendingFiles {
            expected_policy: sha256(b""),
            runtime_commit: true,
            files: vec![
                PendingFile {
                    path: POLICY_PATH.into(),
                    before: None,
                    after: bytes.clone(),
                },
                PendingFile {
                    path: DRAFT_PATH.into(),
                    before: None,
                    after: serde_json::to_vec(&draft).unwrap(),
                },
            ],
        };
        atomic_write(
            &store.path(JOURNAL_PATH).unwrap(),
            &serde_json::to_vec(&journal).unwrap(),
            0o600,
        )
        .unwrap();
        atomic_write(&store.path(POLICY_PATH).unwrap(), &bytes, 0o600).unwrap();
        let snapshot = store.load().unwrap();
        assert_eq!(snapshot.policy, policy);
        assert_eq!(snapshot.workspace.unwrap(), draft);
        assert!(!store.path(JOURNAL_PATH).unwrap().exists());
    }
    #[test]
    fn reconcile_of_already_committed_policy_preserves_a_later_draft() {
        let root = tempfile::tempdir().unwrap();
        let store = ModelStore::new(root.path());
        let before = store.load().unwrap();
        let seed = &before.library.templates[0];
        let policy = Policy {
            template_refs: vec![TemplateRef {
                template_id: seed.id.clone(),
                revision_id: seed.revisions[0].id.clone(),
            }],
            ..Default::default()
        };
        let committed = store
            .commit_runtime_policy(
                &before.library_digest,
                &before.policy_digest,
                &before.workspace_digest,
                policy.clone(),
            )
            .unwrap();
        let next = store
            .stage_policy(
                &committed.policy_digest,
                &committed.workspace_digest,
                Policy::default(),
            )
            .unwrap();
        let repeated = store
            .commit_runtime_policy(
                &before.library_digest,
                &before.policy_digest,
                &before.workspace_digest,
                policy,
            )
            .unwrap();
        assert_eq!(repeated, next);
        assert!(repeated.workspace.unwrap().policy.template_refs.is_empty());
    }
    #[test]
    fn interrupted_pair_rolls_forward_only_its_original_or_intended_bytes() {
        let root = tempfile::tempdir().unwrap();
        let journal = fixture(root.path());
        let store = ModelStore::new(root.path());
        atomic_write(
            &store.path(LIBRARY_PATH).unwrap(),
            &journal.files[0].after,
            0o600,
        )
        .unwrap();
        let snapshot = store.load().unwrap();
        assert_eq!(
            snapshot.library.templates.len(),
            seed_library().templates.len() + 1
        );
        assert_eq!(snapshot.workspace.unwrap().policy.template_refs.len(), 1);
        assert!(!store.path(JOURNAL_PATH).unwrap().exists());
        let repeated = store.load().unwrap();
        assert_eq!(snapshot.library, repeated.library);
    }
    #[test]
    fn subsequent_editor_changes_and_invalid_intended_documents_remain_untouched() {
        let root = tempfile::tempdir().unwrap();
        let mut journal = fixture(root.path());
        let store = ModelStore::new(root.path());
        let changed = b"later-editor";
        atomic_write(&store.path(DRAFT_PATH).unwrap(), changed, 0o600).unwrap();
        assert_eq!(store.load().unwrap_err().code, "journal_drift");
        assert_eq!(
            std::fs::read(store.path(DRAFT_PATH).unwrap()).unwrap(),
            changed
        );
        assert!(!store.path(LIBRARY_PATH).unwrap().exists());
        std::fs::remove_file(store.path(DRAFT_PATH).unwrap()).unwrap();
        journal.files[1].after = b"not-json".to_vec();
        atomic_write(
            &store.path(JOURNAL_PATH).unwrap(),
            &serde_json::to_vec(&journal).unwrap(),
            0o600,
        )
        .unwrap();
        assert!(store.load().is_err());
        assert!(!store.path(LIBRARY_PATH).unwrap().exists());
    }
}
