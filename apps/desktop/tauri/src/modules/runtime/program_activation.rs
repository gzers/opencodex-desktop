//! One recoverable selection transaction. Images are immutable inputs and are
//! never deleted by activation or recovery. Source admission belongs to installers.
use super::{
    bundled, install, program_layout,
    program_selection::{self, CurrentSelection, SelectedProgram, ACTIVATION_PENDING},
    RuntimeSourceKind, RuntimeSourceRecord, RuntimeStore,
};
use crate::{
    errors::{AppError, AppResult},
    infrastructure::{atomic_write::atomic_write, hash, locking::TargetFileLock, platform},
    modules::backup::safety,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

const MAX_RECORD: u64 = 1024 * 1024;
const MAX_JOURNAL: u64 = 16 * 1024 * 1024;

fn fail(detail: &'static str) -> AppError {
    AppError::RuntimeManaged {
        code: "runtime_activation_unconfirmed".into(),
        detail: detail.into(),
    }
}
fn io(_: std::io::Error) -> AppError {
    fail("程序版本事务文件操作失败，原件和恢复记录保留")
}
fn encode(value: &impl Serialize) -> AppResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|_| fail("程序版本事务无法编码"))
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> AppResult<T> {
    let value = crate::modules::model_config::schema_v2::parse_strict_json_v2(bytes)
        .map_err(|_| fail("程序版本事务不是唯一、合法的JSON"))?;
    serde_json::from_value(value).map_err(|_| fail("程序版本事务结构无法核验"))
}
fn read(root: &Path, path: &Path, limit: u64) -> AppResult<Option<Vec<u8>>> {
    safety::check_path(root, path, true)?;
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(io(e)),
        Ok(_) => {
            let file = safety::open_file(path, false)?;
            safety::require_single_link(&file)?;
            let mut bytes = Vec::new();
            file.take(limit + 1).read_to_end(&mut bytes).map_err(io)?;
            if bytes.len() as u64 > limit {
                return Err(fail("程序版本事务文件超过读取上限"));
            }
            Ok(Some(bytes))
        }
    }
}
fn create(path: &Path, bytes: &[u8]) -> AppResult<()> {
    let mut file = platform::create_new_private_file(path, 0o600).map_err(io)?;
    file.write_all(bytes).map_err(io)?;
    file.sync_all().map_err(io)
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ImageChange {
    before: Option<Vec<u8>>,
    after: Option<Vec<u8>>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Journal {
    schema_version: u32,
    attempt_id: String,
    desktop_root: PathBuf,
    before: Option<CurrentSelection>,
    after: CurrentSelection,
    current: ImageChange,
    entry: ImageChange,
    record: ImageChange,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Envelope {
    sha256: String,
    journal: Journal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Complete,
    Rollback,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Checkpoint {
    Journal,
    Entry,
    Current,
    Record,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingSummary {
    pub attempt_id: String,
    pub from: Option<String>,
    pub to: String,
}

/// OS-level serialization for I. Read-only consumers never acquire this lock.
pub struct Mutation {
    root: PathBuf,
    _file: fs::File,
}
impl Mutation {
    pub fn acquire(root: &Path) -> AppResult<Self> {
        let lock_path = root.join(".runtime-program.lock");
        safety::check_path(root, &lock_path, true)?;
        let file = safety::open_file(&lock_path, true)?;
        safety::require_single_link(&file)?;
        file.try_lock()
            .map_err(|_| fail("运行程序正在操作，请稍后重试"))?;
        Ok(Self {
            root: root.into(),
            _file: file,
        })
    }
    fn pending(&self) -> PathBuf {
        self.root.join("runtime").join(ACTIVATION_PENDING)
    }
    fn prepare(
        &self,
        data: &Path,
        expected: Option<&CurrentSelection>,
        after: &CurrentSelection,
    ) -> AppResult<Journal> {
        bundled::roots(&self.root, data)?;
        if read(&self.root, &self.pending(), MAX_JOURNAL)?.is_some() {
            return Err(fail("上次版本事务尚未明确恢复"));
        }
        let (target, platform) = program_layout::host_target()?;
        let before = program_selection::read(&self.root, platform, target)?;
        if before.as_ref().map(|s| &s.selection) != expected {
            return Err(fail("当前版本已变化，未切换程序"));
        }
        let candidate = program_selection::read_image(&self.root, platform, target, after)?;
        let current_path = self
            .root
            .join("runtime")
            .join(program_layout::CURRENT_FILENAME);
        let old_entry = read(&self.root, &candidate.stable_entry, 64 * 1024)?;
        if before.is_none() && old_entry.is_some() {
            return Err(fail("未知程序入口存在，未接管或覆盖"));
        }
        let record = read(data, RuntimeStore::new(data).path(), MAX_RECORD)?;
        let mut next: RuntimeSourceRecord = record
            .as_deref()
            .map(decode)
            .transpose()?
            .unwrap_or_default();
        if next.schema_version != super::RUNTIME_SCHEMA_VERSION
            || (record.is_some() && !record_matches(&next, expected, &candidate.stable_entry))
        {
            return Err(fail("本机运行来源记录不属于当前版本，未覆盖"));
        }
        next.source = RuntimeSourceKind::Managed;
        next.path = Some(candidate.stable_entry.to_string_lossy().into_owned());
        next.resolved_version = Some(after.version.clone());
        next.resolved_at = Some(super::now_rfc3339());
        let mut record_document = record
            .as_deref()
            .map(decode::<serde_json::Value>)
            .transpose()?
            .unwrap_or(
                serde_json::to_value(RuntimeSourceRecord::default())
                    .map_err(|_| fail("来源记录无法编码"))?,
            );
        let updated = serde_json::to_value(&next).map_err(|_| fail("目标来源记录无法编码"))?;
        for key in ["source", "path", "resolved_version", "resolved_at"] {
            record_document[key] = updated[key].clone();
        }
        let journal = Journal {
            schema_version: 2,
            attempt_id: uuid::Uuid::new_v4().to_string(),
            desktop_root: data.into(),
            before: before.map(|s| s.selection),
            after: after.clone(),
            current: ImageChange {
                before: read(&self.root, &current_path, 64 * 1024)?,
                after: Some(encode(after)?),
            },
            entry: ImageChange {
                before: old_entry,
                after: Some(
                    install::entry_script(Some(&candidate.paths.node), &candidate.script)
                        .into_bytes(),
                ),
            },
            record: ImageChange {
                before: record,
                after: Some(encode(&record_document)?),
            },
        };
        self.validate(data, &journal)?;
        Ok(journal)
    }
    fn validate(&self, data: &Path, journal: &Journal) -> AppResult<()> {
        bundled::roots(&self.root, data)?;
        if journal.schema_version != 2
            || journal.desktop_root != data
            || uuid::Uuid::parse_str(&journal.attempt_id).is_err()
        {
            return Err(fail("恢复记录不属于当前程序或用户数据根"));
        }
        let (target, platform) = program_layout::host_target()?;
        let candidate =
            program_selection::read_image(&self.root, platform, target, &journal.after)?;
        let selected: CurrentSelection = decode(
            journal
                .current
                .after
                .as_deref()
                .ok_or_else(|| fail("恢复记录缺少目标选择"))?,
        )?;
        if selected != journal.after
            || journal.entry.after.as_deref()
                != Some(
                    install::entry_script(Some(&candidate.paths.node), &candidate.script)
                        .as_bytes(),
                )
        {
            return Err(fail("恢复目标选择与入口身份不符"));
        }
        match &journal.before {
            Some(before) => {
                let old = program_selection::read_image(&self.root, platform, target, before)?;
                let selection: CurrentSelection = decode(
                    journal
                        .current
                        .before
                        .as_deref()
                        .ok_or_else(|| fail("缺少原版本原件"))?,
                )?;
                if selection != *before
                    || journal.entry.before.as_deref()
                        != Some(
                            install::entry_script(Some(&old.paths.node), &old.script).as_bytes(),
                        )
                {
                    return Err(fail("原版本选择或入口无法核验"));
                }
            }
            None if journal.current.before.is_some() || journal.entry.before.is_some() => {
                return Err(fail("未安装状态不能包含已有程序入口"))
            }
            None => (),
        }
        if let Some(bytes) = &journal.record.before {
            let record: RuntimeSourceRecord = decode(bytes)?;
            if record.schema_version != super::RUNTIME_SCHEMA_VERSION
                || !record_matches(&record, journal.before.as_ref(), &candidate.stable_entry)
            {
                return Err(fail("原本机来源记录无法核验"));
            }
        }
        let record: RuntimeSourceRecord = decode(
            journal
                .record
                .after
                .as_deref()
                .ok_or_else(|| fail("缺少目标来源记录"))?,
        )?;
        if record.schema_version != super::RUNTIME_SCHEMA_VERSION
            || !record_matches(&record, Some(&journal.after), &candidate.stable_entry)
        {
            return Err(fail("目标来源记录无法核验"));
        }
        Ok(())
    }
    fn load(&self, data: &Path) -> AppResult<(Journal, Vec<u8>)> {
        let bytes = read(&self.root, &self.pending(), MAX_JOURNAL)?
            .ok_or_else(|| fail("没有待恢复的版本事务"))?;
        let envelope: Envelope = decode(&bytes)?;
        if hash::sha256_hex(&encode(&envelope.journal)?) != envelope.sha256 {
            return Err(fail("版本恢复记录摘要不符"));
        }
        self.validate(data, &envelope.journal)?;
        Ok((envelope.journal, bytes))
    }
    pub fn pending_summary(&self, data: &Path) -> AppResult<PendingSummary> {
        let (journal, _) = self.load(data)?;
        Ok(PendingSummary {
            attempt_id: journal.attempt_id,
            from: journal.before.map(|s| s.version),
            to: journal.after.version,
        })
    }
    pub(super) fn activate(
        &self,
        data: &Path,
        expected: Option<&CurrentSelection>,
        after: &CurrentSelection,
        admit: impl Fn() -> AppResult<()>,
    ) -> AppResult<SelectedProgram> {
        self.activate_inner(data, expected, after, admit, |_| Ok(()))
    }
    pub(super) fn activate_inner(
        &self,
        data: &Path,
        expected: Option<&CurrentSelection>,
        after: &CurrentSelection,
        admit: impl Fn() -> AppResult<()>,
        checkpoint: impl Fn(Checkpoint) -> AppResult<()>,
    ) -> AppResult<SelectedProgram> {
        let record_path = RuntimeStore::new(data).path().to_path_buf();
        safety::check_path(
            data,
            &crate::infrastructure::locking::lock_path_for(&record_path),
            true,
        )?;
        let _record = TargetFileLock::lock(&record_path)?;
        let journal = self.prepare(data, expected, after)?;
        admit()?;
        let bytes = encode(&Envelope {
            sha256: hash::sha256_hex(&encode(&journal)?),
            journal: journal.clone(),
        })?;
        if bytes.len() as u64 > MAX_JOURNAL {
            return Err(fail("版本恢复记录超过上限"));
        }
        create(&self.pending(), &bytes)?;
        safety::sync_directories(&self.root, &self.root.join("runtime"))?;
        checkpoint(Checkpoint::Journal)?;
        self.apply(
            data,
            &journal,
            &bytes,
            Decision::Complete,
            &admit,
            &checkpoint,
        )?
        .ok_or_else(|| fail("目标版本激活回执缺失"))
    }
    /// Explicit recovery only. A pending file is not permission to resume at startup.
    pub fn recover(
        &self,
        data: &Path,
        attempt: &str,
        decision: Decision,
        admit: impl Fn() -> AppResult<()>,
    ) -> AppResult<Option<SelectedProgram>> {
        bundled::roots(&self.root, data)?;
        // Reject another user's D before creating even a stable target lock there.
        let (journal, bytes) = self.load(data)?;
        if journal.attempt_id != attempt {
            return Err(fail("恢复确认不属于原版本事务"));
        }
        let record_path = RuntimeStore::new(data).path().to_path_buf();
        safety::check_path(
            data,
            &crate::infrastructure::locking::lock_path_for(&record_path),
            true,
        )?;
        let _record = TargetFileLock::lock(&record_path)?;
        if read(&self.root, &self.pending(), MAX_JOURNAL)?.as_deref() != Some(bytes.as_slice()) {
            return Err(fail("取得来源锁期间版本事务已变化"));
        }
        self.apply(data, &journal, &bytes, decision, &admit, &|_| Ok(()))
    }
    fn apply(
        &self,
        data: &Path,
        journal: &Journal,
        bytes: &[u8],
        decision: Decision,
        admit: &impl Fn() -> AppResult<()>,
        checkpoint: &impl Fn(Checkpoint) -> AppResult<()>,
    ) -> AppResult<Option<SelectedProgram>> {
        self.validate(data, journal)?;
        let (target, platform) = program_layout::host_target()?;
        let entry = self
            .root
            .join(program_layout::runtime_entry_relative(platform));
        let current = self
            .root
            .join("runtime")
            .join(program_layout::CURRENT_FILENAME);
        let record = RuntimeStore::new(data).path().to_path_buf();
        let changes = [
            (&self.root, &entry, &journal.entry, 0o755, Checkpoint::Entry),
            (
                &self.root,
                &current,
                &journal.current,
                0o600,
                Checkpoint::Current,
            ),
            (
                &journal.desktop_root,
                &record,
                &journal.record,
                0o600,
                Checkpoint::Record,
            ),
        ];
        // Audit all objects before the first recovery mutation. Unknown values
        // are never overwritten or used to synthesize a replacement baseline.
        for (root, path, change, _, _) in &changes {
            let value = read(root, path, MAX_RECORD)?;
            if value != change.before && value != change.after {
                return Err(fail("版本事务目标包含外部修改，原件和恢复记录保留"));
            }
        }
        for (root, path, change, mode, phase) in &changes {
            admit()?;
            if read(&self.root, &self.pending(), MAX_JOURNAL)?.as_deref() != Some(bytes) {
                return Err(fail("版本事务恢复记录已变化"));
            }
            let desired = if decision == Decision::Complete {
                &change.after
            } else {
                &change.before
            };
            let observed = read(root, path, MAX_RECORD)?;
            if observed != change.before && observed != change.after {
                return Err(fail("版本事务目标在恢复期间变化"));
            }
            if &observed != desired {
                if let Some(value) = desired {
                    atomic_write(path, value, *mode)?;
                } else {
                    fs::remove_file(path).map_err(io)?;
                }
                safety::sync_directories(
                    root,
                    path.parent()
                        .ok_or_else(|| fail("版本事务目标缺少父目录"))?,
                )?;
            }
            if read(root, path, MAX_RECORD)?.as_ref() != desired.as_ref() {
                return Err(fail("版本事务写后读回不符"));
            }
            checkpoint(*phase)?;
        }
        admit()?;
        let selected = program_selection::read_activating(&self.root, platform, target)?;
        let expected = if decision == Decision::Complete {
            Some(&journal.after)
        } else {
            journal.before.as_ref()
        };
        if selected.as_ref().map(|s| &s.selection) != expected {
            return Err(fail("最终程序版本与本次决定不符"));
        }
        for (root, path, change, _, _) in &changes {
            let desired = if decision == Decision::Complete {
                &change.after
            } else {
                &change.before
            };
            if read(root, path, MAX_RECORD)?.as_ref() != desired.as_ref() {
                return Err(fail("版本事务最终读回包含外部修改"));
            }
        }
        if read(&self.root, &self.pending(), MAX_JOURNAL)?.as_deref() != Some(bytes) {
            return Err(fail("最终版本事务身份无法核验"));
        }
        // Retain the immutable transaction and the terminal decision as evidence.
        let receipts = data.join("manager-state/runtime-activations");
        safety::check_path(data, &receipts, true)?;
        platform::create_private_dir_all(&receipts, 0o700).map_err(io)?;
        let receipt = receipts.join(format!(
            "{}-{}.json",
            journal.attempt_id,
            if decision == Decision::Complete {
                "complete"
            } else {
                "rollback"
            }
        ));
        let receipt_bytes = encode(
            &serde_json::json!({"schemaVersion":1,"decision":decision,"transactionSha256":hash::sha256_hex(bytes),"journal":journal}),
        )?;
        if let Some(existing) = read(data, &receipt, MAX_JOURNAL)? {
            if existing != receipt_bytes {
                return Err(fail("已有版本事务回执与本次决定不符"));
            }
        } else {
            create(&receipt, &receipt_bytes)?;
        }
        safety::sync_directories(data, &receipts)?;
        fs::remove_file(self.pending()).map_err(io)?;
        safety::sync_directories(&self.root, &self.root.join("runtime"))?;
        program_selection::read(&self.root, platform, target)
    }
}
fn record_matches(
    record: &RuntimeSourceRecord,
    selected: Option<&CurrentSelection>,
    entry: &Path,
) -> bool {
    match selected {
        Some(selected) => {
            record.source == RuntimeSourceKind::Managed
                && record.path.as_deref() == entry.to_str()
                && record.resolved_version.as_deref() == Some(selected.version.as_str())
        }
        None => {
            record.source == RuntimeSourceKind::Unresolved
                && record.path.is_none()
                && record.resolved_version.is_none()
        }
    }
}

#[cfg(all(test, unix))]
#[path = "program_activation_tests.rs"]
mod tests;
