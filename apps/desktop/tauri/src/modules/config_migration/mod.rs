//! 通用配置文档迁移引擎（U-09a 骨架）。
//!
//! 只建立「文档种类登记 / schema 识别 / 迁移链 / 未来版本拒绝 / 锁内备份与原子提交」
//! 这份可复用骨架；本次不改变磁盘布局，也不执行真实数据根迁移。
//!
//! 三种版本分别管理：应用版本、配置 schema 版本、默认配置版本。当前只为
//! preferences 文档登记一套 legacy v0 → 当前 schema 的转换，后续种类与步骤按同一接口追加。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::errors::AppError;
use crate::infrastructure::atomic_write::atomic_write;
use crate::infrastructure::locking::TargetFileLock;

/// 偏好文档当前 schema（无 schema_version 的旧文件视为 legacy v0）。
pub const PREFERENCES_CURRENT_SCHEMA: u32 = 1;

/// 配置文档种类；每种独立登记 schema 与迁移步骤。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentKind {
    Preferences,
}

impl DocumentKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Preferences => "preferences",
        }
    }

    fn current_schema(&self) -> u32 {
        match self {
            Self::Preferences => PREFERENCES_CURRENT_SCHEMA,
        }
    }
}

/// 迁移执行结果；调用方据此决定是否改写、是否提示用户或停止写入。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationOutcome {
    /// 已是当前 schema，只校验读取，不重写文件。
    Current,
    /// 已从旧 schema 在内存中转换完成，待调用方提交。
    Migrated,
    /// 需要外部输入（如主题历史值）才能完成，暂不落盘。
    NeedsInput,
    /// 版本高于当前应用支持，禁止保存/同步/清理等改写。
    UnsupportedFuture,
    /// 内容损坏或字段非法；保留原件，不静默重置。
    Invalid,
    /// 步骤缺失或内部失败。
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MigrationReport {
    pub document_kind: DocumentKind,
    pub from_schema: u32,
    pub to_schema: u32,
    pub outcome: MigrationOutcome,
    pub steps: Vec<String>,
    pub error: Option<String>,
}

impl MigrationReport {
    /// 是否需要在锁内备份并原子提交。
    pub fn needs_commit(&self) -> bool {
        matches!(self.outcome, MigrationOutcome::Migrated)
    }

    /// 是否允许后续写入；过新或损坏都禁止改写。
    pub fn write_allowed(&self) -> bool {
        matches!(
            self.outcome,
            MigrationOutcome::Current | MigrationOutcome::Migrated
        )
    }
}

/// 从原始字节识别 schema 版本；缺失或非整数按 legacy v0（0）识别。
fn detect_schema(value: &serde_json::Value) -> Result<u32, AppError> {
    let Some(field) = value.get("schema_version") else {
        return Ok(0);
    };
    field
        .as_u64()
        .and_then(|number| u32::try_from(number).ok())
        .ok_or_else(|| AppError::ConfigMigration {
            detail: "schema_version 不是合法整数".to_string(),
        })
}

/// 纯内存转换：识别 schema、执行已发布迁移链并校验，不做任何文件访问。
///
/// 返回报告与「迁移后的字节」；仅当 `outcome == Migrated` 时后者为 `Some`。
pub fn prepare(
    kind: DocumentKind,
    bytes: &[u8],
) -> Result<(MigrationReport, Option<Vec<u8>>), AppError> {
    let value: serde_json::Value = match serde_json::from_slice(bytes) {
        Ok(value) => value,
        Err(_) => {
            return Ok((
                MigrationReport {
                    document_kind: kind,
                    from_schema: 0,
                    to_schema: kind.current_schema(),
                    outcome: MigrationOutcome::Invalid,
                    steps: Vec::new(),
                    error: Some("配置不是合法 JSON".to_string()),
                },
                None,
            ))
        }
    };
    if !value.is_object() {
        return Ok((
            MigrationReport {
                document_kind: kind,
                from_schema: 0,
                to_schema: kind.current_schema(),
                outcome: MigrationOutcome::Invalid,
                steps: Vec::new(),
                error: Some("配置根节点不是对象".to_string()),
            },
            None,
        ));
    }

    let from_schema = match detect_schema(&value) {
        Ok(schema) => schema,
        Err(error) => {
            return Ok((
                MigrationReport {
                    document_kind: kind,
                    from_schema: 0,
                    to_schema: kind.current_schema(),
                    outcome: MigrationOutcome::Invalid,
                    steps: Vec::new(),
                    error: Some(error.to_string()),
                },
                None,
            ))
        }
    };
    let to_schema = kind.current_schema();

    if from_schema > to_schema {
        return Ok((
            MigrationReport {
                document_kind: kind,
                from_schema,
                to_schema,
                outcome: MigrationOutcome::UnsupportedFuture,
                steps: Vec::new(),
                error: Some("配置版本高于当前应用支持".to_string()),
            },
            None,
        ));
    }

    match kind {
        DocumentKind::Preferences => prepare_preferences(from_schema, to_schema, value),
    }
}

fn prepare_preferences(
    from_schema: u32,
    to_schema: u32,
    value: serde_json::Value,
) -> Result<(MigrationReport, Option<Vec<u8>>), AppError> {
    let kind = DocumentKind::Preferences;
    // 无论哪个版本，最终都必须能落到当前类型并通过校验。
    let mut preferences: crate::modules::preferences::Preferences =
        match serde_json::from_value(value.clone()) {
            Ok(value) => value,
            Err(_) => {
                return Ok((
                    MigrationReport {
                        document_kind: kind,
                        from_schema,
                        to_schema,
                        outcome: MigrationOutcome::Invalid,
                        steps: Vec::new(),
                        error: Some("偏好字段类型不合法".to_string()),
                    },
                    None,
                ))
            }
        };

    if from_schema == to_schema {
        preferences.schema_version = to_schema;
        let outcome = match crate::modules::preferences::validate(&preferences) {
            Ok(()) => MigrationOutcome::Current,
            Err(_) => MigrationOutcome::Invalid,
        };
        let report = MigrationReport {
            document_kind: kind,
            from_schema,
            to_schema,
            outcome,
            steps: Vec::new(),
            error: None,
        };
        return Ok((report, None));
    }

    // legacy v0 → 当前：旧复合更新枚举迁移 + 补 schema 版本。
    let mut steps = Vec::new();
    crate::modules::preferences::migrate_legacy_update_fields(&mut preferences);
    steps.push("preferences.v0.legacy_update_channel".to_string());
    preferences.schema_version = to_schema;

    if crate::modules::preferences::validate(&preferences).is_err() {
        return Ok((
            MigrationReport {
                document_kind: kind,
                from_schema,
                to_schema,
                outcome: MigrationOutcome::Invalid,
                steps,
                error: Some("迁移后偏好未通过校验".to_string()),
            },
            None,
        ));
    }

    let bytes = match serde_json::to_vec_pretty(&preferences) {
        Ok(bytes) => bytes,
        Err(error) => {
            return Ok((
                MigrationReport {
                    document_kind: kind,
                    from_schema,
                    to_schema,
                    outcome: MigrationOutcome::Failed,
                    steps,
                    error: Some(error.to_string()),
                },
                None,
            ))
        }
    };
    // 重新识别一次，保证提交内容确实带上当前 schema。
    let _ = detect_schema(&serde_json::from_slice::<serde_json::Value>(&bytes).unwrap_or_default());
    Ok((
        MigrationReport {
            document_kind: kind,
            from_schema,
            to_schema,
            outcome: MigrationOutcome::Migrated,
            steps,
            error: None,
        },
        Some(bytes),
    ))
}

/// 迁移原件与记录目录：`<数据根>/manager-state/config-migrations/<kind>/<txn>/`。
pub fn migration_dir(data_root: &Path, kind: DocumentKind, txn: &str) -> PathBuf {
    data_root
        .join("manager-state/config-migrations")
        .join(kind.as_str())
        .join(txn)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CommittedMigration {
    pub document_kind: DocumentKind,
    pub from_schema: u32,
    pub to_schema: u32,
    pub steps: Vec<String>,
    pub backup_dir: PathBuf,
}

/// 在目标锁内备份原件并原子提交迁移结果。
///
/// 顺序：锁 → 备份原件与摘要 → 原子替换 → 登记；任何一步失败都不替换目标。
pub fn commit(
    data_root: &Path,
    target: &Path,
    original: &[u8],
    report: &MigrationReport,
    migrated: &[u8],
) -> Result<CommittedMigration, AppError> {
    if !report.needs_commit() {
        return Err(AppError::ConfigMigration {
            detail: "非 Migrated 结果不进入提交".to_string(),
        });
    }
    let _lock = TargetFileLock::lock(target)?;
    let txn = format!("txn-{}", uuid::Uuid::new_v4());
    let dir = migration_dir(data_root, report.document_kind, &txn);
    std::fs::create_dir_all(&dir).map_err(|error| AppError::ConfigMigration {
        detail: format!("创建迁移目录失败: {error}"),
    })?;
    std::fs::write(dir.join("original.json"), original).map_err(|error| {
        AppError::ConfigMigration {
            detail: format!("备份原件失败: {error}"),
        }
    })?;
    // prepared 记录带原件与候选摘要：中断后据此判断目标是否已被替换（§6 步骤 6）。
    let manifest = serde_json::json!({
        "document_kind": report.document_kind,
        "from_schema": report.from_schema,
        "to_schema": report.to_schema,
        "steps": report.steps,
        "status": "prepared",
        "original_digest": crate::infrastructure::hash::sha256_hex(original),
        "candidate_digest": crate::infrastructure::hash::sha256_hex(migrated),
    });
    std::fs::write(
        dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|error| AppError::ConfigMigration {
            detail: format!("序列化迁移记录失败: {error}"),
        })?,
    )
    .map_err(|error| AppError::ConfigMigration {
        detail: format!("写入迁移记录失败: {error}"),
    })?;

    atomic_write(target, migrated, 0o600)?;

    let manifest = serde_json::json!({
        "document_kind": report.document_kind,
        "from_schema": report.from_schema,
        "to_schema": report.to_schema,
        "steps": report.steps,
        "status": "committed",
        "original_digest": crate::infrastructure::hash::sha256_hex(original),
        "candidate_digest": crate::infrastructure::hash::sha256_hex(migrated),
    });
    let _ = std::fs::write(
        dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap_or_default(),
    );

    Ok(CommittedMigration {
        document_kind: report.document_kind,
        from_schema: report.from_schema,
        to_schema: report.to_schema,
        steps: report.steps.clone(),
        backup_dir: dir,
    })
}

/// 单个迁移事务的恢复判定结果（迁移机制 §6 步骤 6）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryAction {
    /// 目标仍是原件摘要：prepared 未提交，可重新准备/提交。
    ReapplyPrepared,
    /// 目标已是候选摘要：提交已完成，补写 committed 记录即可。
    CompleteRecord,
    /// 两者都不等：视为外部变更，保留文件和证据，停止自动恢复。
    ExternalChange,
    /// 没有待恢复的事务。
    NoPendingTransaction,
}

/// 一个待核对的迁移事务与其判定。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryReport {
    pub document_kind: DocumentKind,
    pub transaction_dir: PathBuf,
    pub from_schema: u32,
    pub to_schema: u32,
    pub action: RecoveryAction,
}

fn read_manifest(dir: &Path) -> Option<serde_json::Value> {
    let bytes = std::fs::read(dir.join("manifest.json")).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// 找出最后一个尚未 committed 的事务；按目录名（含时间近似）稳定排序取最新的一个。
fn pending_transaction(data_root: &Path, kind: DocumentKind) -> Option<PathBuf> {
    let root = data_root
        .join("manager-state/config-migrations")
        .join(kind.as_str());
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&root)
        .ok()?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect();
    dirs.sort();
    for dir in dirs.into_iter().rev() {
        let Some(manifest) = read_manifest(&dir) else {
            continue;
        };
        if manifest.get("status").and_then(|value| value.as_str()) != Some("committed") {
            return Some(dir);
        }
    }
    None
}

/// 重启恢复入口：核对 prepared 事务与目标摘要，决定重做、补记录还是停止（不盲目覆盖）。
pub fn recover(
    data_root: &Path,
    target: &Path,
    kind: DocumentKind,
) -> Result<RecoveryReport, AppError> {
    let Some(dir) = pending_transaction(data_root, kind) else {
        return Ok(RecoveryReport {
            document_kind: kind,
            transaction_dir: PathBuf::new(),
            from_schema: kind.current_schema(),
            to_schema: kind.current_schema(),
            action: RecoveryAction::NoPendingTransaction,
        });
    };
    let manifest = read_manifest(&dir).ok_or_else(|| AppError::ConfigMigration {
        detail: "迁移事务缺少可读记录".to_string(),
    })?;
    let from_schema = manifest
        .get("from_schema")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let to_schema = manifest
        .get("to_schema")
        .and_then(|v| v.as_u64())
        .unwrap_or(kind.current_schema() as u64) as u32;
    let original_digest = manifest.get("original_digest").and_then(|v| v.as_str());
    let candidate_digest = manifest.get("candidate_digest").and_then(|v| v.as_str());

    let current = std::fs::read(target).map_err(|error| AppError::ConfigMigration {
        detail: format!("读取迁移目标失败: {error}"),
    })?;
    let current_digest = crate::infrastructure::hash::sha256_hex(&current);
    let action = if Some(current_digest.as_str()) == candidate_digest {
        RecoveryAction::CompleteRecord
    } else if Some(current_digest.as_str()) == original_digest {
        RecoveryAction::ReapplyPrepared
    } else {
        RecoveryAction::ExternalChange
    };
    if action == RecoveryAction::CompleteRecord {
        let committed = serde_json::json!({
            "document_kind": kind,
            "from_schema": from_schema,
            "to_schema": to_schema,
            "steps": manifest.get("steps").cloned().unwrap_or_default(),
            "status": "committed",
            "original_digest": original_digest,
            "candidate_digest": candidate_digest,
        });
        let _ = std::fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec_pretty(&committed).unwrap_or_default(),
        );
    }
    Ok(RecoveryReport {
        document_kind: kind,
        transaction_dir: dir,
        from_schema,
        to_schema,
        action,
    })
}

/// 启动迁移入口（§5）：对某个已知文档按需转换。缺文件或已是当前 schema 时不写盘。
pub fn migrate_document_on_startup(
    data_root: &Path,
    kind: DocumentKind,
    target: &Path,
) -> Result<Option<CommittedMigration>, AppError> {
    let bytes = match std::fs::read(target) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(AppError::ConfigMigration {
                detail: format!("读取待迁移文档失败: {error}"),
            })
        }
    };
    let (report, migrated) = prepare(kind, &bytes)?;
    if !report.needs_commit() {
        return Ok(None);
    }
    let migrated = migrated.ok_or_else(|| AppError::ConfigMigration {
        detail: "Migrated 结果缺少候选字节".to_string(),
    })?;
    commit(data_root, target, &bytes, &report, &migrated).map(Some)
}

/// 启动时处理偏好文档：先完成未提交事务，再按需迁移旧 schema。
pub fn migrate_preferences_on_startup(
    data_root: &Path,
) -> Result<Option<CommittedMigration>, AppError> {
    let target = data_root.join(crate::modules::preferences::PREFERENCES_RELATIVE_PATH);
    let _ = recover(data_root, &target, DocumentKind::Preferences)?;
    migrate_document_on_startup(data_root, DocumentKind::Preferences, &target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_preferences_migrate_to_current_schema() {
        let legacy =
            br#"{"interface_scale":120,"launch_main":true,"app_update_channel":"beta-6h"}"#;
        let (report, bytes) = prepare(DocumentKind::Preferences, legacy).expect("prepare");
        assert_eq!(report.outcome, MigrationOutcome::Migrated);
        assert_eq!(report.from_schema, 0);
        assert_eq!(report.to_schema, PREFERENCES_CURRENT_SCHEMA);
        assert_eq!(report.steps, vec!["preferences.v0.legacy_update_channel"]);
        let value: serde_json::Value = serde_json::from_slice(&bytes.expect("bytes")).unwrap();
        assert_eq!(value["schema_version"], PREFERENCES_CURRENT_SCHEMA);
        assert_eq!(value["app_update_channel"], "beta");
        assert_eq!(value["app_update_auto_check"], true);
        assert_eq!(value["app_update_check_interval_seconds"], 21600);
    }

    #[test]
    fn current_schema_is_not_rewritten() {
        let current = serde_json::json!({
            "schema_version": PREFERENCES_CURRENT_SCHEMA,
            "interface_scale": 100,
        });
        let bytes = serde_json::to_vec(&current).unwrap();
        let (report, migrated) = prepare(DocumentKind::Preferences, &bytes).expect("prepare");
        assert_eq!(report.outcome, MigrationOutcome::Current);
        assert!(!report.needs_commit());
        assert!(migrated.is_none());
    }

    #[test]
    fn future_schema_is_rejected_without_transform() {
        let future = serde_json::json!({ "schema_version": PREFERENCES_CURRENT_SCHEMA + 5 });
        let bytes = serde_json::to_vec(&future).unwrap();
        let (report, migrated) = prepare(DocumentKind::Preferences, &bytes).expect("prepare");
        assert_eq!(report.outcome, MigrationOutcome::UnsupportedFuture);
        assert!(!report.write_allowed());
        assert!(migrated.is_none());
    }

    #[test]
    fn corrupted_bytes_are_invalid() {
        let (report, migrated) = prepare(DocumentKind::Preferences, b"{broken").expect("prepare");
        assert_eq!(report.outcome, MigrationOutcome::Invalid);
        assert!(migrated.is_none());
    }

    #[test]
    fn commit_backs_up_original_and_writes_current_schema() {
        let root = tempfile::tempdir().expect("temp root");
        let target = root.path().join("manager-state/preferences.json");
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        let legacy = br#"{"interface_scale":120,"app_update_channel":"manual"}"#.to_vec();
        std::fs::write(&target, &legacy).unwrap();
        let (report, migrated) = prepare(DocumentKind::Preferences, &legacy).expect("prepare");
        let committed = commit(
            root.path(),
            &target,
            &legacy,
            &report,
            &migrated.expect("migrated bytes"),
        )
        .expect("commit");
        assert_eq!(committed.from_schema, 0);
        assert!(committed.backup_dir.join("original.json").exists());
        let written: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&target).unwrap()).unwrap();
        assert_eq!(written["schema_version"], PREFERENCES_CURRENT_SCHEMA);
        assert_eq!(written["app_update_channel"], "stable");
        assert_eq!(written["app_update_auto_check"], false);
    }

    #[test]
    fn recovery_completes_record_when_target_is_candidate() {
        let root = tempfile::tempdir().expect("temp root");
        let target = root.path().join("manager-state/preferences.json");
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        let legacy = br#"{"interface_scale":120,"app_update_channel":"manual"}"#.to_vec();
        std::fs::write(&target, &legacy).unwrap();
        let (report, migrated) = prepare(DocumentKind::Preferences, &legacy).expect("prepare");
        let migrated = migrated.expect("migrated bytes");
        let committed = commit(root.path(), &target, &legacy, &report, &migrated).expect("commit");

        // 手动回退 manifest 到 prepared，模拟提交后崩溃前未写 committed。
        let mut manifest: serde_json::Value = serde_json::from_slice(
            &std::fs::read(committed.backup_dir.join("manifest.json")).unwrap(),
        )
        .unwrap();
        manifest["status"] = serde_json::json!("prepared");
        std::fs::write(
            committed.backup_dir.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let recovered = recover(root.path(), &target, DocumentKind::Preferences).expect("recover");
        assert_eq!(recovered.action, RecoveryAction::CompleteRecord);
    }

    #[test]
    fn recovery_reapplies_when_target_is_still_original() {
        let root = tempfile::tempdir().expect("temp root");
        let target = root.path().join("manager-state/preferences.json");
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        let legacy = br#"{"interface_scale":120,"app_update_channel":"manual"}"#.to_vec();
        std::fs::write(&target, &legacy).unwrap();
        let (report, migrated) = prepare(DocumentKind::Preferences, &legacy).expect("prepare");
        let migrated = migrated.expect("migrated bytes");
        let committed = commit(root.path(), &target, &legacy, &report, &migrated).expect("commit");

        // 目标回到原件字节，manifest 保持 prepared：应判定可重做。
        std::fs::write(&target, &legacy).unwrap();
        let mut manifest: serde_json::Value = serde_json::from_slice(
            &std::fs::read(committed.backup_dir.join("manifest.json")).unwrap(),
        )
        .unwrap();
        manifest["status"] = serde_json::json!("prepared");
        std::fs::write(
            committed.backup_dir.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let recovered = recover(root.path(), &target, DocumentKind::Preferences).expect("recover");
        assert_eq!(recovered.action, RecoveryAction::ReapplyPrepared);
    }

    #[test]
    fn recovery_stops_on_external_change() {
        let root = tempfile::tempdir().expect("temp root");
        let target = root.path().join("manager-state/preferences.json");
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        let legacy = br#"{"interface_scale":120,"app_update_channel":"manual"}"#.to_vec();
        std::fs::write(&target, &legacy).unwrap();
        let (report, migrated) = prepare(DocumentKind::Preferences, &legacy).expect("prepare");
        let migrated = migrated.expect("migrated bytes");
        let committed = commit(root.path(), &target, &legacy, &report, &migrated).expect("commit");
        let mut manifest: serde_json::Value = serde_json::from_slice(
            &std::fs::read(committed.backup_dir.join("manifest.json")).unwrap(),
        )
        .unwrap();
        manifest["status"] = serde_json::json!("prepared");
        std::fs::write(
            committed.backup_dir.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        std::fs::write(&target, r#"{"interface_scale":99}"#.as_bytes()).unwrap();

        let recovered = recover(root.path(), &target, DocumentKind::Preferences).expect("recover");
        assert_eq!(recovered.action, RecoveryAction::ExternalChange);
    }

    #[test]
    fn startup_migration_skips_when_current_schema() {
        let root = tempfile::tempdir().expect("temp root");
        let target = root.path().join("manager-state/preferences.json");
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        let current = serde_json::to_vec(&serde_json::json!({
            "schema_version": PREFERENCES_CURRENT_SCHEMA,
            "interface_scale": 100,
        }))
        .unwrap();
        std::fs::write(&target, &current).unwrap();
        let before = std::fs::read(&target).unwrap();
        let uploaded = migrate_preferences_on_startup(root.path()).expect("startup migration");
        assert!(uploaded.is_none());
        assert_eq!(std::fs::read(&target).unwrap(), before);
        assert!(!root.path().join("manager-state/config-migrations").exists());
    }

    #[test]
    fn startup_migration_converts_legacy_preferences() {
        let root = tempfile::tempdir().expect("temp root");
        let target = root.path().join("manager-state/preferences.json");
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(
            &target,
            br#"{"interface_scale":120,"app_update_channel":"beta-6h"}"#,
        )
        .unwrap();
        let committed = migrate_preferences_on_startup(root.path())
            .expect("startup migration")
            .expect("committed");
        assert_eq!(committed.from_schema, 0);
        let written: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&target).unwrap()).unwrap();
        assert_eq!(written["schema_version"], PREFERENCES_CURRENT_SCHEMA);
        assert_eq!(written["app_update_channel"], "beta");
        assert!(committed.backup_dir.join("original.json").exists());
    }
}
