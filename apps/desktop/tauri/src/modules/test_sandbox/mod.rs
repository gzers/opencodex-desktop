//! 测试沙箱身份（配置规划§10）。
//!
//! 开发/集成测试默认使用独立数据身份，与日常使用身份分离；沙箱根选择失败即停止，
//! 不回退到日常目录。日常发布构建默认不启用沙箱，行为与此前一致。

use std::path::{Path, PathBuf};

/// 沙箱身份标识（拟议值）。
pub const SANDBOX_IDENTIFIER: &str = "com.gzers.opencodex.desktop.sandbox";

/// 沙箱根覆盖环境变量；测试脚本用它指向每次运行的临时根。
const SANDBOX_ROOT_ENV: &str = "OPENCODEX_SANDBOX_ROOT";
/// Explicit isolated container for sibling migration destinations. Omission
/// retains the original anchor-only boundary; never infer the parent directory.
const SANDBOX_BOUNDARY_ENV: &str = "OPENCODEX_SANDBOX_BOUNDARY";

/// 构建/启动策略：是否运行在沙箱身份。
///
/// 只有显式设置 ${B}OPENCODEX_SANDBOX=${B}（或等价 dev 标记）才启用；未设置时保持日常身份。
pub fn enabled() -> bool {
    match std::env::var("OPENCODEX_SANDBOX") {
        Ok(value) => matches!(value.trim(), "1" | "true" | "yes" | "on"),
        Err(_) => false,
    }
}

/// 沙箱数据根：要求 ${B}OPENCODEX_SANDBOX_ROOT${B} 为绝对路径；缺失或非法即返回 None，由调用方停止。
pub fn sandbox_root() -> Option<PathBuf> {
    if !enabled() {
        return None;
    }
    let raw = std::env::var_os(SANDBOX_ROOT_ENV)?;
    if raw.is_empty() {
        return None;
    }
    let path = PathBuf::from(raw);
    if !path.is_absolute() {
        return None;
    }
    Some(path)
}

/// 解析应用数据根：沙箱启用时使用沙箱根，否则沿用 Tauri 提供的日常路径。
///
/// 沙箱启用但根缺失/非法时返回 Err，调用方必须停止启动，不能回退到日常目录。
pub fn resolve_data_root(default_root: &Path) -> Result<PathBuf, String> {
    if !enabled() {
        return Ok(default_root.to_path_buf());
    }
    match sandbox_root() {
        Some(root) => Ok(root),
        None => Err(format!(
            "沙箱已启用但 {SANDBOX_ROOT_ENV} 缺失或不是绝对路径；拒绝回退到日常目录"
        )),
    }
}

/// 可以写入沙箱的活动数据根：沙箱启用时强制使用沙箱内管理器状态目录。
pub fn resolve_active_data_root(sandbox: &Path) -> PathBuf {
    sandbox.join("manager-state").join("active-root")
}

/// Resolve once at startup and retain it in application state. Commands must not
/// re-read an environment variable to broaden an already-running sandbox.
pub fn resolve_boundary(anchor: &Path) -> Result<Option<PathBuf>, String> {
    if !enabled() {
        return Ok(None);
    }
    let explicit = std::env::var_os(SANDBOX_BOUNDARY_ENV).map(PathBuf::from);
    isolated_boundary(anchor, explicit.as_deref()).map(Some)
}

fn prospective_canonical(path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute()
        || path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err("isolated path must be absolute without parent traversal".into());
    }
    let mut ancestor = path;
    let mut missing = Vec::new();
    loop {
        match std::fs::symlink_metadata(ancestor) {
            Ok(_) => break,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                missing.push(ancestor.file_name().ok_or("invalid isolated path")?);
                ancestor = ancestor.parent().ok_or("invalid isolated path")?;
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    let mut resolved = ancestor.canonicalize().map_err(|e| e.to_string())?;
    if !missing.is_empty() && !resolved.is_dir() {
        return Err("isolated path ancestor is not a directory".into());
    }
    for part in missing.into_iter().rev() {
        resolved.push(part);
    }
    Ok(resolved)
}

fn isolated_boundary(anchor: &Path, explicit: Option<&Path>) -> Result<PathBuf, String> {
    let anchor = prospective_canonical(anchor)?;
    let Some(explicit) = explicit else {
        return Ok(anchor);
    };
    let boundary = prospective_canonical(explicit)?;
    if !boundary.is_dir()
        || boundary.parent().is_none()
        || anchor == boundary
        || !anchor.starts_with(&boundary)
    {
        return Err(
            "sandbox boundary must be an existing isolated container enclosing the anchor".into(),
        );
    }
    Ok(boundary)
}

pub fn check_boundary(boundary: Option<&Path>, path: &Path) -> Result<(), String> {
    if let Some(boundary) = boundary {
        if !prospective_canonical(path)?.starts_with(boundary) {
            return Err("path escapes the isolated sandbox container".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_container_allows_siblings_but_never_infers_parent_scope() {
        let temp = tempfile::tempdir().unwrap();
        let container = temp.path().join("isolated");
        std::fs::create_dir(&container).unwrap();
        let anchor = container.join("source");
        let target = container.join("target 中文");
        let boundary = isolated_boundary(&anchor, Some(&container)).unwrap();
        assert!(check_boundary(Some(&boundary), &anchor).is_ok());
        assert!(check_boundary(Some(&boundary), &target).is_ok());
        let default = isolated_boundary(&anchor, None).unwrap();
        assert!(check_boundary(Some(&default), &target).is_err());
        assert!(check_boundary(Some(&boundary), &temp.path().join("daily")).is_err());
        assert!(check_boundary(Some(&boundary), &container.join("../daily")).is_err());
        assert!(isolated_boundary(&anchor, Some(Path::new("relative"))).is_err());
        assert!(isolated_boundary(&anchor, Some(&container.join("missing"))).is_err());
        assert!(isolated_boundary(&container, Some(&container)).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn linked_or_dangling_paths_cannot_hide_container_escape() {
        let temp = tempfile::tempdir().unwrap();
        let container = temp.path().join("isolated");
        let outside = temp.path().join("daily");
        std::fs::create_dir(&container).unwrap();
        std::fs::create_dir(&outside).unwrap();
        let boundary = isolated_boundary(&container.join("source"), Some(&container)).unwrap();
        std::os::unix::fs::symlink(&outside, container.join("linked")).unwrap();
        std::os::unix::fs::symlink(outside.join("missing"), container.join("dangling")).unwrap();
        assert!(check_boundary(Some(&boundary), &container.join("linked/target")).is_err());
        assert!(check_boundary(Some(&boundary), &container.join("dangling/target")).is_err());
        assert!(isolated_boundary(&container.join("linked/source"), Some(&container)).is_err());
    }

    // 单测共享进程环境变量，合并为一个测试避免并发互相干扰。
    #[test]
    fn sandbox_identity_and_root_contract() {
        std::env::remove_var("OPENCODEX_SANDBOX");
        std::env::remove_var(SANDBOX_ROOT_ENV);
        assert!(!enabled());
        assert!(resolve_data_root(Path::new("/daily/root")).is_ok());

        std::env::set_var("OPENCODEX_SANDBOX", "1");
        std::env::remove_var(SANDBOX_ROOT_ENV);
        assert!(enabled());
        assert!(sandbox_root().is_none());
        // 沙箱启用但根缺失：拒绝回退到日常目录。
        assert!(resolve_data_root(Path::new("/daily/root")).is_err());

        std::env::set_var(SANDBOX_ROOT_ENV, "relative/path");
        assert!(sandbox_root().is_none());
        assert!(resolve_data_root(Path::new("/daily/root")).is_err());

        let dir = tempfile::tempdir().expect("temp");
        std::env::set_var(SANDBOX_ROOT_ENV, dir.path());
        assert_eq!(sandbox_root().as_deref(), Some(dir.path()));
        assert_eq!(
            resolve_data_root(Path::new("/daily/root")).expect("sandbox root"),
            dir.path()
        );

        std::env::remove_var(SANDBOX_ROOT_ENV);
        std::env::remove_var("OPENCODEX_SANDBOX");
    }
}
