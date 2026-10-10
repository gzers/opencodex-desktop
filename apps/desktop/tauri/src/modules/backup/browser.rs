//! Bounded metadata-only browsing. Payload integrity is checked by restore, not listing.
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use super::{BackupManifest, MANIFEST_NAME, SCHEMA_VERSION};
use crate::errors::{AppError, AppResult};
use serde::Serialize;

const MAX_ENTRIES: usize = 5000;
const MAX_DEPTH: usize = 8;
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupFileNode {
    pub id: String,
    pub label: String,
    pub directory: bool,
    pub purpose: String,
    pub can_open: bool,
    pub children: Vec<BackupFileNode>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupFiles {
    pub root_path: String,
    pub nodes: Vec<BackupFileNode>,
}

fn fail(reason: &str) -> AppError {
    AppError::FileSystem {
        operation: "browse backup files".into(),
        detail: reason.into(),
    }
}

fn inspect(path: &Path) -> AppResult<std::fs::Metadata> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| fail("无法读取备份目录或文件"))?;
    if metadata.file_type().is_symlink() || !(metadata.is_dir() || metadata.is_file()) {
        return Err(fail("备份目录包含链接或不支持的文件类型"));
    }
    Ok(metadata)
}

fn text_file(path: &Path) -> bool {
    path.extension().and_then(|s| s.to_str()).is_some_and(|s| {
        matches!(
            s.to_ascii_lowercase().as_str(),
            "json" | "toml" | "yaml" | "yml" | "txt" | "md" | "ini" | "conf" | "log"
        )
    })
}

fn validate_manifest(path: &Path) -> AppResult<()> {
    inspect(path)?;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|_| fail("无法读取备份清单"))?
        .take(MAX_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| fail("无法读取备份清单"))?;
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(fail("备份清单超过读取上限"));
    }
    let manifest: BackupManifest =
        serde_json::from_slice(&bytes).map_err(|_| fail("备份清单格式不正确"))?;
    let record = path.parent().ok_or_else(|| fail("备份清单目录不正确"))?;
    if manifest.schema_version != SCHEMA_VERSION
        || record.file_name().and_then(|s| s.to_str()) != Some(manifest.backup_id.as_str())
        || record
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            != Some(manifest.action.as_str())
    {
        return Err(fail("备份清单版本或目录归属不正确"));
    }
    Ok(())
}

fn purpose(path: &Path, depth: usize, directory: bool) -> String {
    if !directory {
        return if path.file_name().is_some_and(|s| s == MANIFEST_NAME) {
            "备份范围、时间与完整性摘要；恢复前重新校验。"
        } else {
            "事务保存的文件副本；实际范围以本份清单为准。"
        }
        .into();
    }
    match depth {
        0 => "按 UTC 年月与事务分类保存备份。",
        1 => "UTC 年份。",
        2 => "UTC 月份。",
        3 => match path.file_name().and_then(|s| s.to_str()) {
            Some("upgrade") => "升级前备份。",
            Some("import") => "导入前备份。",
            Some("sync-overwrite") => "同步覆盖前备份。",
            Some("data-root-move") => "数据根迁移备份。",
            Some("extension-write") => "扩展写入前备份。",
            Some("runtime-uninstall") => "托管卸载前备份。",
            Some("manual-preferences") => "管理器偏好手动备份。",
            Some("restore-protection") => "恢复前保护备份。",
            Some("preferences-protection") => "变更前管理器偏好保护备份。",
            _ => "备份事务目录。",
        },
        4 => "一份备份；内容与用途以本份清单为准。",
        _ => "备份内的目录。",
    }
    .into()
}

fn visit(
    path: &Path,
    root: &Path,
    depth: usize,
    remaining: &mut usize,
) -> AppResult<BackupFileNode> {
    if *remaining == 0 || depth > MAX_DEPTH {
        return Err(fail("备份目录超过浏览上限，请缩小范围后重试"));
    }
    *remaining -= 1;
    let metadata = inspect(path)?;
    let directory = metadata.is_dir();
    let mut children = Vec::new();
    if directory {
        // Bound collection too: a huge flat directory must not allocate unbounded memory.
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(path).map_err(|_| fail("无法读取备份目录"))? {
            if entries.len() >= *remaining {
                return Err(fail("备份目录超过浏览上限，请缩小范围后重试"));
            }
            entries.push(entry.map_err(|_| fail("无法读取备份目录"))?.path());
        }
        entries.sort();
        for entry in entries {
            children.push(visit(&entry, root, depth + 1, remaining)?);
        }
    } else if path.file_name().is_some_and(|s| s == MANIFEST_NAME) {
        validate_manifest(path)?;
    }
    let relative = path
        .strip_prefix(root)
        .map_err(|_| fail("备份路径超出目录边界"))?;
    let id = relative
        .to_str()
        .ok_or_else(|| fail("备份名称编码不受支持"))?
        .replace('\\', "/");
    Ok(BackupFileNode {
        id: if id.is_empty() { ".".into() } else { id },
        label: path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| fail("备份名称编码不受支持"))?
            .into(),
        directory,
        purpose: purpose(path, depth, directory),
        can_open: directory || text_file(path),
        children,
    })
}

pub fn list(data_root: &Path) -> AppResult<BackupFiles> {
    let root = data_root.join("backups");
    super::safety::check_path(data_root, &root, true)?;
    let mut remaining = MAX_ENTRIES;
    let nodes = match std::fs::symlink_metadata(&root) {
        Ok(_) => {
            if !inspect(&root)?.is_dir() {
                return Err(fail("备份根不是目录"));
            }
            vec![visit(&root, &root, 0, &mut remaining)?]
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(_) => return Err(fail("无法检查备份目录")),
    };
    Ok(BackupFiles {
        root_path: root.to_string_lossy().into_owned(),
        nodes,
    })
}

/// Revalidate on every open; the frontend passes only a relative node identifier.
pub fn resolve_open(data_root: &Path, id: &str) -> AppResult<PathBuf> {
    if id.is_empty() || id.contains('\\') || id.contains(':') {
        return Err(fail("备份路径不正确"));
    }
    let relative = Path::new(id);
    if id != "."
        && relative
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(fail("备份路径超出目录边界"));
    }
    let root = data_root.join("backups");
    super::safety::check_path(data_root, &root, true)?;
    if !inspect(&root)?.is_dir() {
        return Err(fail("备份根不是目录"));
    }
    let canonical_root = root.canonicalize().map_err(|_| fail("无法读取备份目录"))?;
    let mut path = root;
    if id != "." {
        for part in relative.components() {
            path.push(part);
            inspect(&path)?;
        }
    }
    let metadata = inspect(&path)?;
    let canonical = path.canonicalize().map_err(|_| fail("无法读取备份文件"))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(fail("备份路径超出目录边界"));
    }
    if metadata.is_file() {
        if !text_file(&canonical) {
            return Err(fail("此文件类型不支持直接打开，请打开所在目录"));
        }
        std::fs::File::open(&canonical).map_err(|_| fail("没有读取备份文件的权限"))?;
    } else {
        std::fs::read_dir(&canonical).map_err(|_| fail("没有读取备份目录的权限"))?;
    }
    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lists_real_files_without_verifying_payload_and_checks_manifest() {
        let temp = tempfile::tempdir().unwrap();
        let record = super::super::backup_file(
            temp.path(),
            super::super::BackupAction::Upgrade,
            Path::new("/config/preferences.json"),
            b"{}",
            chrono::Utc::now(),
            None,
        )
        .unwrap();
        std::fs::write(record.directory.join("preferences.json"), b"changed").unwrap();
        let files = list(temp.path()).unwrap();
        let record_node = &files.nodes[0].children[0].children[0].children[0].children[0];
        assert_eq!(record_node.children.len(), 2);
        let payload = record_node
            .children
            .iter()
            .find(|n| n.label == "preferences.json")
            .unwrap();
        assert!(payload.can_open);
        assert!(resolve_open(temp.path(), &payload.id).unwrap().is_file());
        std::fs::write(record.directory.join(MANIFEST_NAME), b"invalid").unwrap();
        assert!(list(temp.path()).is_err());
    }
    #[test]
    fn empty_missing_and_unsafe_paths() {
        let temp = tempfile::tempdir().unwrap();
        assert!(list(temp.path()).unwrap().nodes.is_empty());
        std::fs::create_dir(temp.path().join("backups")).unwrap();
        std::fs::write(temp.path().join("backups/script.cmd"), b"exit").unwrap();
        for id in [
            "../escape.json",
            "/tmp/file",
            "C:/file",
            "x/../file",
            "script.cmd",
            "missing.json",
        ] {
            assert!(resolve_open(temp.path(), id).is_err(), "{id}");
        }
        assert!(resolve_open(temp.path(), ".").unwrap().is_dir());
    }
    #[test]
    fn bounded_depth_and_manifest_size() {
        let temp = tempfile::tempdir().unwrap();
        let mut path = temp.path().join("backups");
        for _ in 0..10 {
            path = path.join("nested");
        }
        std::fs::create_dir_all(path).unwrap();
        assert!(list(temp.path()).is_err());
        std::fs::remove_dir_all(temp.path().join("backups")).unwrap();
        std::fs::create_dir(temp.path().join("backups")).unwrap();
        std::fs::write(
            temp.path().join("backups").join(MANIFEST_NAME),
            vec![b' '; 65537],
        )
        .unwrap();
        assert!(list(temp.path()).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn rejects_internal_and_external_symlinks_in_listing_and_open() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir(temp.path().join("backups")).unwrap();
        std::fs::write(temp.path().join("outside.json"), b"secret").unwrap();
        std::os::unix::fs::symlink(
            temp.path().join("outside.json"),
            temp.path().join("backups/link.json"),
        )
        .unwrap();
        assert!(list(temp.path()).is_err());
        assert!(resolve_open(temp.path(), "link.json").is_err());
        std::fs::remove_dir_all(temp.path().join("backups")).unwrap();
        std::os::unix::fs::symlink(temp.path().join("missing"), temp.path().join("backups"))
            .unwrap();
        assert!(
            list(temp.path()).is_err(),
            "dangling root link must not look like an empty backup directory"
        );
    }
}
