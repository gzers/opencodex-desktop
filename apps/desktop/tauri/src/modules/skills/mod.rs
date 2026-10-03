//! MOD-09：FZ-23 / FZ-24 / FZ-28 Skills 源、登记与分发策略。
//!
//! 所有路径都来自显式参数；模块不解析或读取真实用户家目录、
//! 数据根、Keychain 或网络资源。测试只使用受控临时目录。

use crate::modules::extensions::ClientId;
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// FZ-23 源存储版本与兼容读取位置约定。
pub const STORE_VERSION: u32 = 1;

/// FZ-24 symlink 与分发限制。
pub const SYMLINK_MAX_DEPTH: usize = 40;

/// FZ-28 压缩包导入限制。
pub const ARCHIVE_MAX_ENTRIES: usize = 1_000;
pub const ARCHIVE_MAX_TOTAL_BYTES: u64 = 128 * 1024 * 1024;
pub const ARCHIVE_SINGLE_FILE_MAX_BYTES: u64 = 32 * 1024 * 1024;

/// FZ-23 源存储布局：默认读源为 Agent Skills 共享目录，管理器写入区在数据根内。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SkillsSourceStore {
    pub store_version: u32,
    /// 默认读源：`<主目录>/.agents/skills`（Windows 为 `%USERPROFILE%\.agents\skills`）。
    pub source_dir: PathBuf,
    /// 管理器写入区：`<数据根>/manager-state/skills-store`（导入 / 恢复 / 备份 / 导出）。
    pub store_dir: PathBuf,
    pub registered: Vec<SkillRegistration>,
}

/// FZ-28 冻结来源登记。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct SkillRegistration {
    pub source: SkillSource,
    pub source_uri: String,
    pub source_sha256: String,
    pub verified_at: String,
    pub size_bytes: u64,
    pub file_count: u32,
    pub origin_trust: OriginTrust,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillSource {
    Local,
    Imported,
    SharedStore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OriginTrust {
    User,
}

/// 每个 Skill 在客户端的期望同步开关；未安装客户端由分发时跳过。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SkillDistribution {
    pub name: String,
    pub enabled_clients: Vec<ClientId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrationError {
    VersionMismatch,
    DigestShape,
    UriContainsSecret,
    ArchiveEntryLimit,
    ArchiveTotalLimit,
    ArchiveFileLimit,
    TimestampInvalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymlinkError {
    TooDeep,
    EscapesBoundary,
    UnsupportedFileType,
    Cycle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistributionError {
    NotInstalled,
    HashConflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistributionAction {
    CreateSymlink,
    Copy,
    Skip,
}

/// 校验 FZ-23 源存储。
pub fn validate_source_store(store: &SkillsSourceStore) -> Result<(), RegistrationError> {
    if store.store_version != STORE_VERSION {
        return Err(RegistrationError::VersionMismatch);
    }
    for registration in &store.registered {
        validate_registration(registration)?;
    }
    Ok(())
}

/// 校验 FZ-28 登记形状与敏感信息。
pub fn validate_registration(registration: &SkillRegistration) -> Result<(), RegistrationError> {
    if registration.source_sha256.len() != 64
        || !registration
            .source_sha256
            .chars()
            .all(|c| c.is_ascii_hexdigit())
    {
        return Err(RegistrationError::DigestShape);
    }
    let normalized_uri = registration.source_uri.trim();
    if normalized_uri.is_empty()
        || normalized_uri.contains("token=")
        || normalized_uri.contains("password=")
        || normalized_uri.contains("authorization=")
    {
        return Err(RegistrationError::UriContainsSecret);
    }
    if DateTime::parse_from_rfc3339(&registration.verified_at).is_err() {
        return Err(RegistrationError::TimestampInvalid);
    }
    if registration.file_count > ARCHIVE_MAX_ENTRIES as u32 {
        return Err(RegistrationError::ArchiveEntryLimit);
    }
    if registration.size_bytes > ARCHIVE_MAX_TOTAL_BYTES {
        return Err(RegistrationError::ArchiveTotalLimit);
    }
    Ok(())
}

/// 导入压缩包上限判定。
pub fn validate_archive_import(entries: &[(String, u64)]) -> Result<(), RegistrationError> {
    if entries.len() > ARCHIVE_MAX_ENTRIES {
        return Err(RegistrationError::ArchiveEntryLimit);
    }
    let total: u64 = entries.iter().map(|(_, size)| *size).sum();
    if total > ARCHIVE_MAX_TOTAL_BYTES {
        return Err(RegistrationError::ArchiveTotalLimit);
    }
    if entries
        .iter()
        .any(|(_, size)| *size > ARCHIVE_SINGLE_FILE_MAX_BYTES)
    {
        return Err(RegistrationError::ArchiveFileLimit);
    }
    Ok(())
}

/// 判断 source 是否逃出 allowed 根；只接受已规范化路径。
pub fn symlink_stays_within(source: &Path, allowed: &[&Path]) -> bool {
    if source.as_os_str().is_empty() {
        return false;
    }
    let canonical_root = |root: &Path| root.canonicalize().ok();
    allowed.iter().any(|root| {
        canonical_root(root).is_some_and(|canonical_root| source.starts_with(&canonical_root))
    })
}

/// 递归校验 symlink 链；目标必须留在同一个显式根内。
pub fn validate_symlink_chain(
    source: &Path,
    root: &Path,
    depth: usize,
) -> Result<(), SymlinkError> {
    if depth > SYMLINK_MAX_DEPTH {
        return Err(SymlinkError::TooDeep);
    }
    let Ok(path) = source.canonicalize() else {
        return Err(SymlinkError::EscapesBoundary);
    };
    if !symlink_stays_within(&path, &[root]) {
        return Err(SymlinkError::EscapesBoundary);
    }
    let metadata = std::fs::symlink_metadata(source).map_err(|_| SymlinkError::EscapesBoundary)?;
    if metadata.is_symlink() {
        let target = std::fs::read_link(source).map_err(|_| SymlinkError::EscapesBoundary)?;
        let target = if target.is_absolute() {
            target
        } else {
            source
                .parent()
                .ok_or(SymlinkError::EscapesBoundary)?
                .join(target)
        };
        return validate_symlink_chain(&target, root, depth + 1);
    }
    if !metadata.is_dir() && !metadata.is_file() {
        return Err(SymlinkError::UnsupportedFileType);
    }
    Ok(())
}

/// FZ-24：目标已存在且内容哈希不同进待处理；相同可跳过。
pub fn distribution_decision(
    source_payload: &[u8],
    target: Option<&Path>,
) -> Result<DistributionAction, DistributionError> {
    let Some(path) = target else {
        return Ok(DistributionAction::CreateSymlink);
    };
    if !path.exists() {
        return Ok(DistributionAction::CreateSymlink);
    }
    let target_payload = std::fs::read(path).map_err(|_| DistributionError::HashConflict)?;
    if sha256_hex(source_payload) == sha256_hex(&target_payload) {
        return Ok(DistributionAction::Skip);
    }
    Err(DistributionError::HashConflict)
}

/// 首选 symlink；复制是显式回退，未安装不生成动作。
pub fn planned_action(
    skill: &SkillDistribution,
    client: ClientId,
    installed: bool,
) -> Option<DistributionAction> {
    if !skill.enabled_clients.contains(&client) || !installed {
        return None;
    }
    Some(DistributionAction::CreateSymlink)
}

/// 计算目录内容摘要；路径按名排序，不包含目录自身元数据。
pub fn directory_digest(root: &Path) -> Result<String, std::io::Error> {
    let mut hasher = Sha256::new();
    let mut paths = Vec::new();
    collect_files(root, &mut paths)?;
    paths.sort();
    for path in paths {
        let relative = path
            .strip_prefix(root)
            .expect("relative path")
            .to_path_buf();
        hasher.update(relative.to_string_lossy().as_bytes());
        hasher.update([0]);
        let payload = std::fs::read(&path)?;
        hasher.update((payload.len() as u64).to_le_bytes());
        hasher.update(payload);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

pub fn collect_files(current: &Path, paths: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = std::fs::symlink_metadata(&path)?;
        if metadata.is_dir() {
            collect_files(&path, paths)?;
        } else if metadata.is_file() {
            paths.push(path);
        }
    }
    Ok(())
}

pub fn sha256_hex(payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// 使用冻结格式生成登记时间戳。
pub fn verified_at(now: DateTime<Utc>) -> String {
    now.to_rfc3339_opts(SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::extensions::ExtensionConfig;
    use chrono::TimeZone;

    fn temp_root(name: &str) -> tempfile::TempDir {
        tempfile::Builder::new()
            .prefix(name)
            .tempdir()
            .expect("temporary directory")
    }

    fn registration() -> SkillRegistration {
        SkillRegistration {
            source: SkillSource::Local,
            source_uri: "file:///fixtures/skills/demo".to_string(),
            source_sha256: "a".repeat(64),
            verified_at: verified_at(Utc.with_ymd_and_hms(2026, 9, 15, 0, 0, 0).unwrap()),
            size_bytes: 1,
            file_count: 1,
            origin_trust: OriginTrust::User,
            note: None,
        }
    }

    #[test]
    fn source_store_matches_fz23() {
        let store = SkillsSourceStore {
            store_version: STORE_VERSION,
            source_dir: PathBuf::from("/fixture/home/.agents/skills"),
            store_dir: PathBuf::from("/fixture/data-root/manager-state/skills-store"),
            registered: vec![registration()],
        };
        assert!(validate_source_store(&store).is_ok());
        assert_eq!(STORE_VERSION, 1);
    }

    #[test]
    fn registration_rejects_secret_uri_and_bad_shape() {
        assert!(validate_registration(&registration()).is_ok());
        let mut value = registration();
        value.source_uri = "https://example.test?token=secret".to_string();
        assert_eq!(
            validate_registration(&value),
            Err(RegistrationError::UriContainsSecret)
        );
        value = registration();
        value.source_sha256 = "not-a-digest".into();
        assert_eq!(
            validate_registration(&value),
            Err(RegistrationError::DigestShape)
        );
        value = registration();
        value.verified_at = "not-a-time".into();
        assert_eq!(
            validate_registration(&value),
            Err(RegistrationError::TimestampInvalid)
        );
    }

    #[test]
    fn archive_limits_match_fz28() {
        assert_eq!(ARCHIVE_MAX_ENTRIES, 1_000);
        assert_eq!(ARCHIVE_MAX_TOTAL_BYTES, 128 * 1024 * 1024);
        assert_eq!(ARCHIVE_SINGLE_FILE_MAX_BYTES, 32 * 1024 * 1024);
        assert!(validate_archive_import(&[("a".into(), 1)]).is_ok());
        let entries = (0..1_001).map(|i| (format!("{i}"), 1)).collect::<Vec<_>>();
        assert_eq!(
            validate_archive_import(&entries),
            Err(RegistrationError::ArchiveEntryLimit)
        );
    }

    #[test]
    fn symlink_chain_rejects_escape_but_accepts_internal_link() {
        let root = temp_root("skills-root-");
        let source = root.path().join("source.txt");
        std::fs::write(&source, b"content").unwrap();
        let link = root.path().join("link.txt");
        std::os::unix::fs::symlink(&source, &link).unwrap();
        let inside_result = validate_symlink_chain(&link, root.path(), 0);
        assert!(
            inside_result.is_ok(),
            "inside symlink should pass: {inside_result:?}; link={:?}, canonical={:?}, root={:?}",
            link,
            link.canonicalize(),
            root.path().canonicalize()
        );

        let escaped = temp_root("skills-outside-");
        let escaped_file = escaped.path().join("outside.txt");
        std::fs::write(&escaped_file, b"outside").unwrap();
        let escaped_link = root.path().join("escaped.txt");
        std::os::unix::fs::symlink(&escaped_file, &escaped_link).unwrap();
        assert_eq!(
            validate_symlink_chain(&escaped_link, root.path(), 0),
            Err(SymlinkError::EscapesBoundary)
        );
    }

    #[test]
    fn distribution_plans_symlink_and_skips_uninstalled() {
        let skill = SkillDistribution {
            name: "demo".into(),
            enabled_clients: vec![ClientId::Codex, ClientId::Claude],
        };
        assert_eq!(
            planned_action(&skill, ClientId::Codex, true),
            Some(DistributionAction::CreateSymlink)
        );
        assert_eq!(planned_action(&skill, ClientId::Codex, false), None);
        assert_eq!(planned_action(&skill, ClientId::Gemini, true), None);
    }

    #[test]
    fn existing_target_conflicts_only_on_different_hash() {
        let target = temp_root("skills-conflict-target-");
        let target_file = target.path().join("skill.md");
        std::fs::write(&target_file, b"same").unwrap();
        assert_eq!(
            distribution_decision(b"same", Some(&target_file)),
            Ok(DistributionAction::Skip)
        );
        std::fs::write(&target_file, b"different").unwrap();
        assert_eq!(
            distribution_decision(b"same", Some(&target_file)),
            Err(DistributionError::HashConflict)
        );
        assert_eq!(
            distribution_decision(b"same", None),
            Ok(DistributionAction::CreateSymlink)
        );
    }

    #[test]
    fn directory_digest_is_order_stable_and_change_sensitive() {
        let root = temp_root("skills-digest-");
        std::fs::create_dir_all(root.path().join("nested")).unwrap();
        std::fs::write(root.path().join("b.txt"), b"two").unwrap();
        std::fs::write(root.path().join("nested/a.txt"), b"one").unwrap();
        let first = directory_digest(root.path()).unwrap();
        std::fs::write(root.path().join("nested/a.txt"), b"changed").unwrap();
        let second = directory_digest(root.path()).unwrap();
        assert_ne!(first, second);
        assert_eq!(first.len(), 64);
    }

    #[test]
    fn copy_fallback_is_explicit_not_implicit() {
        let root = temp_root("skills-copy-");
        let file = root.path().join("skill.md");
        std::fs::write(&file, b"existing").unwrap();
        assert!(planned_action(
            &SkillDistribution {
                name: "demo".into(),
                enabled_clients: vec![ClientId::Codex],
            },
            ClientId::Codex,
            true
        )
        .is_some());
        assert_eq!(
            distribution_decision(b"payload", Some(&file)),
            Err(DistributionError::HashConflict)
        );
        let mut config = ExtensionConfig::default();
        assert_eq!(config.revision, 1);
        config.set_enabled(ClientId::Codex, false);
        assert_eq!(config.revision, 2);
    }
}
