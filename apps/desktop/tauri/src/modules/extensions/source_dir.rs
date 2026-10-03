//! FZ-23 源目录：默认源目录与自定义源目录的解析与校验。
//!
//! 源目录是**配置值**而非常量：默认为 `<主目录>/.agents/skills`，可由用户改为
//! 自定义目录。自定义目录只读，且必须先通过本模块的校验——校验失败时调用方
//! **不得切换**，要保留原值并给出原因。

use std::path::{Path, PathBuf};

use super::{ClientTarget, ExtensionConfig};

/// 校验失败的稳定原因码；前端按码出提示，不解析文案。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceDirError {
    /// 不是绝对路径。
    NotAbsolute,
    /// 路径不存在。
    Missing,
    /// 入口本身是符号链接。
    Symlink,
    /// 不是目录。
    NotDirectory,
    /// 不可读（无权限或读取失败）。
    NotReadable,
    /// 无法解析为真实路径。
    Unresolvable,
    /// 与数据根、管理器写入区或任一客户端 Skills 目录重叠。
    OverlapsManaged,
}

impl SourceDirError {
    pub fn code(&self) -> &'static str {
        match self {
            SourceDirError::NotAbsolute => "not_absolute",
            SourceDirError::Missing => "missing",
            SourceDirError::Symlink => "symlink",
            SourceDirError::NotDirectory => "not_directory",
            SourceDirError::NotReadable => "not_readable",
            SourceDirError::Unresolvable => "unresolvable",
            SourceDirError::OverlapsManaged => "overlaps_managed",
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            SourceDirError::NotAbsolute => "请使用绝对路径。",
            SourceDirError::Missing => "目录不存在。",
            SourceDirError::Symlink => "该路径是符号链接；请选择真实目录。",
            SourceDirError::NotDirectory => "该路径不是目录。",
            SourceDirError::NotReadable => "目录不可读或没有访问权限。",
            SourceDirError::Unresolvable => "无法解析该目录的真实路径。",
            SourceDirError::OverlapsManaged => {
                "该目录与数据根、管理器写入区或客户端 Skills 目录重叠；请换一个目录。"
            }
        }
    }
}

/// 默认源目录（不校验，可能尚不存在）。
pub fn default_source_dir(home: &Path) -> PathBuf {
    super::discovery::agents_skills_dir(home)
}

/// 解析配置声明的源目录。`source_store` 为空指针或为空串时回落到默认目录。
///
/// 只做解析，不做校验——`configured_source_dir` 可能返回一个尚未通过校验的路径；
/// 需要保证合法性的调用方用 `validate_or_default`。
pub fn configured_source_dir(config: &ExtensionConfig, home: &Path) -> PathBuf {
    match config.source_store.as_deref().map(str::trim) {
        Some(value) if !value.is_empty() => PathBuf::from(value),
        _ => default_source_dir(home),
    }
}

/// 发现用：校验通过则用自定义目录；不通过则**回落默认目录**并在返回值里说明。
///
/// 发现是只读动作，不应因一个坏配置整体失败；写入动作请用 `validate_source_dir`
/// 并显式失败。
pub fn validate_or_default(
    config: &ExtensionConfig,
    data_root: &Path,
    home: &Path,
    targets: &[ClientTarget],
) -> (PathBuf, Option<SourceDirError>) {
    let declared = configured_source_dir(config, home);
    if config
        .source_store
        .as_deref()
        .map(str::trim)
        .is_none_or(str::is_empty)
    {
        return (declared, None);
    }
    match validate_source_dir(&declared, data_root, home, targets) {
        Ok(path) => (path, None),
        Err(error) => (default_source_dir(home), Some(error)),
    }
}

/// R-23 校验：通过时返回规范化后的真实路径。
pub fn validate_source_dir(
    candidate: &Path,
    data_root: &Path,
    home: &Path,
    targets: &[ClientTarget],
) -> Result<PathBuf, SourceDirError> {
    if !candidate.is_absolute() {
        return Err(SourceDirError::NotAbsolute);
    }
    let metadata = std::fs::symlink_metadata(candidate).map_err(|_| SourceDirError::Missing)?;
    if metadata.is_symlink() {
        return Err(SourceDirError::Symlink);
    }
    if !metadata.is_dir() {
        return Err(SourceDirError::NotDirectory);
    }
    // 可读：能列出目录即视为可读。
    std::fs::read_dir(candidate).map_err(|_| SourceDirError::NotReadable)?;
    let canonical = candidate
        .canonicalize()
        .map_err(|_| SourceDirError::Unresolvable)?;

    for managed in managed_roots(data_root, home, targets) {
        let Ok(managed) = managed.canonicalize() else {
            // 受管根尚不存在（如未初始化的写入区）：退化为字面比较，
            // 避免因「目录还没建」而漏判重叠。
            if overlaps_lexically(&canonical, &managed) {
                return Err(SourceDirError::OverlapsManaged);
            }
            continue;
        };
        if overlaps_lexically(&canonical, &managed) {
            return Err(SourceDirError::OverlapsManaged);
        }
    }
    Ok(canonical)
}

/// 需要与源目录互斥的受管根：数据根、管理器写入区、各客户端 Skills 目录。
fn managed_roots(data_root: &Path, home: &Path, targets: &[ClientTarget]) -> Vec<PathBuf> {
    let mut roots = vec![
        data_root.to_path_buf(),
        super::discovery::skills_store_dir(data_root),
    ];
    if !home.as_os_str().is_empty() {
        roots.push(home.to_path_buf());
    }
    roots.extend(targets.iter().map(|target| target.skills_dir.clone()));
    roots
}

/// 两个路径是否相等或互为祖先/后代。
fn overlaps_lexically(left: &Path, right: &Path) -> bool {
    left == right || left.starts_with(right) || right.starts_with(left)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::modules::extensions::ClientId;

    fn targets(home: &Path) -> Vec<ClientTarget> {
        [
            ClientId::Codex,
            ClientId::Claude,
            ClientId::Gemini,
            ClientId::Grok,
            ClientId::Opencode,
            ClientId::Hermes,
        ]
        .map(|client| ClientTarget::user_target(client, home, true, true))
        .into_iter()
        .collect()
    }

    #[test]
    fn default_source_dir_is_the_agents_standard_directory() {
        let home = tempfile::tempdir().unwrap();
        assert_eq!(
            default_source_dir(home.path()),
            home.path().join(".agents/skills")
        );
    }

    #[test]
    fn configured_source_dir_falls_back_to_default_when_unset() {
        let home = tempfile::tempdir().unwrap();
        let config = ExtensionConfig::default();
        assert_eq!(
            configured_source_dir(&config, home.path()),
            home.path().join(".agents/skills")
        );
    }

    #[test]
    fn custom_source_dir_is_accepted_when_it_is_a_plain_readable_directory() {
        let home = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        let custom = tempfile::tempdir().unwrap();
        let resolved = validate_source_dir(
            custom.path(),
            root.path(),
            home.path(),
            &targets(home.path()),
        )
        .expect("custom dir should validate");
        assert_eq!(resolved, custom.path().canonicalize().unwrap());
    }

    #[test]
    fn relative_and_missing_paths_are_rejected() {
        let home = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        let list = targets(home.path());
        assert_eq!(
            validate_source_dir(
                Path::new("relative/skills"),
                root.path(),
                home.path(),
                &list
            ),
            Err(SourceDirError::NotAbsolute)
        );
        assert_eq!(
            validate_source_dir(
                &home.path().join("no-such-dir"),
                root.path(),
                home.path(),
                &list
            ),
            Err(SourceDirError::Missing)
        );
    }

    #[test]
    fn symlink_entry_is_rejected() {
        let home = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        let real = tempfile::tempdir().unwrap();
        let link = home.path().join("linked-skills");
        std::os::unix::fs::symlink(real.path(), &link).unwrap();
        assert_eq!(
            validate_source_dir(&link, root.path(), home.path(), &targets(home.path())),
            Err(SourceDirError::Symlink)
        );
    }

    #[test]
    fn overlapping_managed_roots_are_rejected() {
        let home = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        let list = targets(home.path());
        // 写入区按需创建，确保命中「重叠」而非「不存在」。
        let store = root.path().join("manager-state/skills-store");
        std::fs::create_dir_all(&store).unwrap();
        std::fs::create_dir_all(home.path().join(".codex/skills")).unwrap();
        // 数据根本身、写入区、客户端 Skills 目录、以及它们的祖先（含 home）都不能当源目录。
        for candidate in [
            root.path().to_path_buf(),
            store.clone(),
            home.path().join(".codex/skills"),
            home.path().to_path_buf(),
        ] {
            assert_eq!(
                validate_source_dir(&candidate, root.path(), home.path(), &list),
                Err(SourceDirError::OverlapsManaged),
                "{candidate:?} 应被判为重叠"
            );
        }
    }

    #[test]
    fn non_existent_candidate_reports_missing_before_overlap() {
        let home = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        let list = targets(home.path());
        // 写入区尚未创建：先报「不存在」，这是对用户更有用的原因。
        assert_eq!(
            validate_source_dir(
                &root.path().join("manager-state/skills-store"),
                root.path(),
                home.path(),
                &list
            ),
            Err(SourceDirError::Missing)
        );
    }

    #[test]
    fn missing_client_directory_does_not_hide_overlap() {
        let home = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        let list = targets(home.path());
        // `.codex/skills` 不存在时仍要判定与 `.codex` 重叠。
        let candidate = home.path().join(".codex");
        std::fs::create_dir_all(&candidate).unwrap();
        assert_eq!(
            validate_source_dir(&candidate, root.path(), home.path(), &list),
            Err(SourceDirError::OverlapsManaged)
        );
    }

    #[test]
    fn validate_or_default_reports_why_the_custom_dir_was_ignored() {
        let home = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();
        crate::modules::data_root::initialize(root.path()).unwrap();
        let list = targets(home.path());
        let mut config = ExtensionConfig::default();
        config.set_source_store(Some(home.path().join("missing").display().to_string()));
        let (resolved, error) = validate_or_default(&config, root.path(), home.path(), &list);
        assert_eq!(resolved, default_source_dir(home.path()));
        assert_eq!(error, Some(SourceDirError::Missing));
    }
}
