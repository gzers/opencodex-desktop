//! 跨平台文件系统辅助：权限位与符号链接。
//!
//! Windows 没有 Unix 权限位语义（访问控制由 ACL 按用户目录继承），
//! 因此权限相关调用在 Windows 上退化为 no-op，而不是报错；符号链接按
//! 平台选择 `symlink`（Unix）或 `symlink_file`（Windows）。

use std::fs::OpenOptions;
use std::io;
use std::path::Path;

/// `OpenOptions` 创建文件时的 Unix 权限位；Windows 上为 no-op。
pub trait OpenOptionsModeExt {
    fn create_mode(&mut self, mode: u32) -> &mut Self;
}

#[cfg(unix)]
impl OpenOptionsModeExt for OpenOptions {
    fn create_mode(&mut self, mode: u32) -> &mut Self {
        use std::os::unix::fs::OpenOptionsExt;
        self.mode(mode)
    }
}

#[cfg(not(unix))]
impl OpenOptionsModeExt for OpenOptions {
    fn create_mode(&mut self, _mode: u32) -> &mut Self {
        self
    }
}

/// 设置文件或目录的权限位；Windows 上 no-op。
pub fn set_mode(path: &Path, mode: u32) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(path)?.permissions();
        permissions.set_mode(mode);
        std::fs::set_permissions(path, permissions)
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mode);
        Ok(())
    }
}

/// 读取权限位的低 9 位；Windows 返回 `0`（无 Unix 权限语义）。
pub fn mode_of(path: &Path) -> io::Result<u32> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        Ok(std::fs::metadata(path)?.permissions().mode() & 0o777)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(0)
    }
}

/// 把 `from` 的权限位复制到 `to`；Windows 上 no-op。
pub fn copy_mode(from: &Path, to: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(from)?.permissions().mode();
        let mut permissions = std::fs::metadata(to)?.permissions();
        permissions.set_mode(mode);
        std::fs::set_permissions(to, permissions)
    }
    #[cfg(not(unix))]
    {
        let _ = (from, to);
        Ok(())
    }
}

/// 创建指向文件的符号链接。Windows 需要开发者模式或相应权限，
/// 调用方应把失败当作可回退路径（例如退回复制）。
#[cfg(unix)]
pub fn symlink_file(source: &Path, destination: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(source, destination)
}

#[cfg(windows)]
pub fn symlink_file(source: &Path, destination: &Path) -> io::Result<()> {
    std::os::windows::fs::symlink_file(source, destination)
}

#[cfg(not(any(unix, windows)))]
pub fn symlink_file(source: &Path, destination: &Path) -> io::Result<()> {
    let _ = (source, destination);
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "symbolic links are not supported on this platform",
    ))
}
