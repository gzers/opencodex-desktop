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
// The release seed owns identities, independent of installation time or locale.
const SEEDS: [(&str, &str, &str, &str); 3] = [
    (
        "837c8133-76e9-4bdf-a68c-9637331c4d43",
        "2f415ab0-e93f-400d-b4a8-8539bfc87903",
        "GPT",
        "gpt-",
    ),
    (
        "961df482-3cbe-421d-a268-caa768de2fc6",
        "51acc742-419e-43b2-898e-cdf05760554d",
        "Claude",
        "claude-",
    ),
    (
        "e4182850-f6b6-4597-b06a-935a23fc19de",
        "049145b7-8227-4c03-aec6-1915e84f7ce6",
        "Gemini",
        "gemini-",
    ),
];

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
    Library {
        schema_version: SCHEMA_VERSION,
        templates: SEEDS
            .iter()
            .map(|(id, revision, label, prefix)| {
                let mut content = TemplateContent::default();
                content.matching.include.push(Pattern {
                    kind: MatchKind::Prefix,
                    value: (*prefix).into(),
                });
                content.description = "内置匹配起点；能力字段默认继承，实际渠道需核验".into();
                let digest = content_digest(label, &content).expect("static seed is serializable");
                Template {
                    id: (*id).into(),
                    label: (*label).into(),
                    source: TemplateSource::Builtin,
                    default_revision: Some((*revision).into()),
                    revisions: vec![Revision {
                        id: (*revision).into(),
                        created_at: "2026-10-18T00:00:00Z".into(),
                        saved_label: (*label).into(),
                        content,
                        digest,
                        base_revision: None,
                        changes: vec![],
                    }],
                }
            })
            .collect(),
    }
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
        let guard = TargetFileLock::lock(&path)?;
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
        Ok((
            Snapshot {
                library,
                library_digest: sha256(bytes.as_deref().unwrap_or_default()),
                policy_digest: sha256(policy_bytes.as_deref().unwrap_or_default()),
            },
            policy,
        ))
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
            library_digest: sha256(&bytes),
            policy_digest: expected.policy_digest.clone(),
        })
    }
    pub fn save(&self, mut request: SaveTemplate) -> ModelResult<Commit> {
        let _guard = self.lock()?;
        let (snapshot, _) = self.load_unlocked()?;
        if request.expected_library != snapshot.library_digest {
            return Err(invalid("stale_baseline", "模版库已变化，请先比较最新版本"));
        }
        request.label = request.label.trim().into();
        if request.label.is_empty() || request.label.len() > 256 {
            return Err(invalid("label", "名称须为1至256字节"));
        }
        request.content.normalize();
        request.content.validate()?;
        let digest = content_digest(&request.label, &request.content)?;
        let mut library = snapshot.library.clone();
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
                return Ok(Commit {
                    snapshot,
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
        });
        let snapshot = self.persist(&library, &snapshot)?;
        Ok(Commit {
            snapshot,
            template_id,
            revision_id,
            changed: true,
        })
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
        template: &str,
        revisions: &[String],
    ) -> ModelResult<Snapshot> {
        let _guard = self.lock()?;
        let (snapshot, policy) = self.load_unlocked()?;
        if expected_library != snapshot.library_digest || expected_policy != snapshot.policy_digest
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
                || SEEDS.iter().any(|(t, r, _, _)| *t == template && *r == id)
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
            if used {
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
fn change_facts(
    base: Option<&Revision>,
    label: &str,
    content: &TemplateContent,
) -> ModelResult<Vec<ChangeFact>> {
    let old = base.map(|r| json!({"名称": r.saved_label, "说明": r.content.description, "匹配": r.content.matching, "通用配置": r.content.rules, "型号规则": r.content.variants, "人工锁": r.content.locked_fields})).unwrap_or(Value::Null);
    let new = json!({"名称": label, "说明": content.description, "匹配": content.matching, "通用配置": content.rules, "型号规则": content.variants, "人工锁": content.locked_fields});
    Ok(["名称", "说明", "匹配", "通用配置", "型号规则", "人工锁"]
        .iter()
        .filter(|key| old.get(**key) != new.get(**key))
        .map(|key| ChangeFact {
            field: (*key).into(),
            before: old.get(*key).cloned().unwrap_or(Value::Null),
            after: new.get(*key).cloned().unwrap_or(Value::Null),
        })
        .collect())
}
