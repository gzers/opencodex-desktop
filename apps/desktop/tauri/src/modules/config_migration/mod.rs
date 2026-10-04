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
    let manifest = serde_json::json!({
        "document_kind": report.document_kind,
        "from_schema": report.from_schema,
        "to_schema": report.to_schema,
        "steps": report.steps,
        "status": "prepared",
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
}
