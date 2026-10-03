//! 配置迁移前端 DTO；领域结果与 WebView 契约保持分离。

use serde::{Deserialize, Serialize};

use crate::modules::migration::{ExportResult, ImportResult, ImportSectionSkip};

/// 导入请求：新版明文容器不需要口令；仅旧版加密容器才需要。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationImportRequest {
    #[serde(default)]
    pub passphrase: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationExportResult {
    pub path: String,
    pub backup_id: Option<String>,
    pub document_sha256: String,
    pub format_version: u32,
    pub sections: Vec<String>,
    pub excluded: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationImportSectionSkip {
    pub section: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationImportResult {
    pub backup_id: String,
    pub document_sha256: String,
    pub format_version: u32,
    pub applied_sections: Vec<String>,
    pub skipped_sections: Vec<MigrationImportSectionSkip>,
    pub excluded: Vec<String>,
}

impl From<ExportResult> for MigrationExportResult {
    fn from(value: ExportResult) -> Self {
        Self {
            path: value.path.to_string_lossy().into_owned(),
            backup_id: value.backup_id,
            document_sha256: value.document_sha256,
            format_version: value.format_version,
            sections: value.sections,
            excluded: value.excluded,
        }
    }
}

impl From<ImportSectionSkip> for MigrationImportSectionSkip {
    fn from(value: ImportSectionSkip) -> Self {
        Self {
            section: value.section,
            reason: value.reason,
        }
    }
}

impl From<ImportResult> for MigrationImportResult {
    fn from(value: ImportResult) -> Self {
        Self {
            backup_id: value.backup_id,
            document_sha256: value.document_sha256,
            format_version: value.format_version,
            applied_sections: value.applied_sections,
            skipped_sections: value
                .skipped_sections
                .into_iter()
                .map(MigrationImportSectionSkip::from)
                .collect(),
            excluded: value.excluded,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn migration_dtos_use_camel_case() {
        let export = MigrationExportResult {
            path: "/tmp/opencodex/config.ocxdconf".to_string(),
            backup_id: Some("bk_01".to_string()),
            document_sha256: "digest".to_string(),
            format_version: 2,
            sections: vec!["preferences".to_string()],
            excluded: vec!["钥匙串密文".to_string()],
        };
        let payload = serde_json::to_value(&export).expect("serialize export DTO");
        assert_eq!(payload["backupId"], "bk_01");
        assert_eq!(payload["documentSha256"], "digest");
        assert_eq!(payload["formatVersion"], 2);
        assert_eq!(payload["sections"][0], "preferences");

        let import = MigrationImportResult {
            backup_id: "bk_02".to_string(),
            document_sha256: "digest".to_string(),
            format_version: 2,
            applied_sections: vec!["preferences".to_string()],
            skipped_sections: vec![MigrationImportSectionSkip {
                section: "asset_files".to_string(),
                reason: "不在容器内".to_string(),
            }],
            excluded: vec![],
        };
        let payload = serde_json::to_value(&import).expect("serialize import DTO");
        assert_eq!(payload["backupId"], "bk_02");
        assert_eq!(payload["appliedSections"][0], "preferences");
        assert_eq!(payload["skippedSections"][0]["section"], "asset_files");
    }

    #[test]
    fn import_request_passphrase_is_optional() {
        let empty: MigrationImportRequest = serde_json::from_str("{}").expect("parse empty");
        assert!(empty.passphrase.is_none());
        let with: MigrationImportRequest =
            serde_json::from_str("{\"passphrase\":\"x\"}").expect("parse value");
        assert_eq!(with.passphrase.as_deref(), Some("x"));
    }

    #[test]
    fn module_results_project_without_extra_fields() {
        let export = ExportResult {
            path: PathBuf::from("/tmp/config.ocxdconf"),
            backup_id: None,
            document_sha256: "digest".to_string(),
            format_version: 2,
            sections: vec![],
            excluded: vec![],
        };
        let value = MigrationExportResult::from(export);
        assert!(value.backup_id.is_none());
        assert_eq!(value.path, "/tmp/config.ocxdconf");
    }
}
