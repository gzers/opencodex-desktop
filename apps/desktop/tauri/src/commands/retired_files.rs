//! 已删除功能的残留文件清理。
//!
//! 为什么要做：删代码不等于删数据。0.2.12 删掉了客户端核对状态机，但用户机器上的
//! `model-client-state.json` 还在——它既不再被读取，也不会自己消失。留着会让人以为
//! 「是不是还有东西在核对」。
//!
//! 边界：只删**已确认无用**的固定文件名，不遍历目录、不删用户配置、不删历史备份。
use std::path::Path;

/// 已废弃的固定文件（相对数据根）。新增条目必须是「读取代码已完全删除」的文件。
const RETIRED_FILES: &[&str] = &[
    // 0.2.12 删除四态客户端状态机（UI 规范 §43 / 数据与状态 §43）。
    "manager-state/model-client-state.json",
];

/// 删除残留文件，返回实际删除数量。
///
/// 失败不阻断启动：删不掉只是留个无用文件，比让软件起不来好。
pub fn purge_retired(data_root: &Path) -> usize {
    let mut removed = 0;
    for relative in RETIRED_FILES {
        let path = data_root.join(relative);
        // 只处理数据根之内的普通文件；符号链接与目录一律跳过。
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        if std::fs::remove_file(&path).is_ok() {
            removed += 1;
        }
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn removes_only_the_retired_fixed_file() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("manager-state")).unwrap();
        let retired = root.path().join("manager-state/model-client-state.json");
        let keep = root.path().join("manager-state/model-policy.json");
        std::fs::write(&retired, b"{}").unwrap();
        std::fs::write(&keep, b"{}").unwrap();
        assert_eq!(purge_retired(root.path()), 1);
        assert!(!retired.exists());
        // 用户配置类文件绝不能碰。
        assert!(keep.exists());
    }

    #[test]
    fn missing_file_is_not_an_error() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(purge_retired(root.path()), 0);
    }

    #[test]
    fn never_follows_a_directory_or_symlink_at_the_retired_path() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("manager-state/model-client-state.json")).unwrap();
        // 同名目录不是我们的文件，跳过而不是递归删除。
        assert_eq!(purge_retired(root.path()), 0);
        assert!(root
            .path()
            .join("manager-state/model-client-state.json")
            .is_dir());
    }
}
