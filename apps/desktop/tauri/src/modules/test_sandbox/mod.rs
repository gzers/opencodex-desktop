//! 测试沙箱身份（配置规划§10）。
//!
//! 开发/集成测试默认使用独立数据身份，与日常使用身份分离；沙箱根选择失败即停止，
//! 不回退到日常目录。日常发布构建默认不启用沙箱，行为与此前一致。

use std::path::{Path, PathBuf};

/// 沙箱身份标识（拟议值）。
pub const SANDBOX_IDENTIFIER: &str = "com.gzers.opencodex.desktop.sandbox";

/// 沙箱根覆盖环境变量；测试脚本用它指向每次运行的临时根。
const SANDBOX_ROOT_ENV: &str = "OPENCODEX_SANDBOX_ROOT";

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

#[cfg(test)]
mod tests {
    use super::*;

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
