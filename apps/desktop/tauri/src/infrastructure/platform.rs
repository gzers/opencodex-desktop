//! 跨平台文件系统辅助：权限位与符号链接。
//!
//! Windows 没有 Unix 权限位语义（访问控制由 ACL 按用户目录继承），
//! 因此权限相关调用在 Windows 上退化为 no-op，而不是报错；符号链接按
//! 平台选择 `symlink`（Unix）或 `symlink_file`（Windows）。

use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use std::path::PathBuf;

/// Windows 使用用户配置目录 API / USERPROFILE；Unix 保留 HOME 语义。
/// 绝不把缺失或相对路径当作当前工作目录。
pub fn home_dir() -> Option<PathBuf> {
    dirs::home_dir().filter(|path| path.is_absolute() && !path.as_os_str().is_empty())
}

pub fn platform_label() -> &'static str {
    if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else {
        std::env::consts::OS
    }
}

/// Windows 短命探针/扫描器退出后可能尚在释放目录句柄。
/// 只对共享/访问冲突做有界重试，保持原子 rename，不先删除目标。
pub fn rename(source: &Path, target: &Path) -> io::Result<()> {
    #[cfg(windows)]
    {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        loop {
            match std::fs::rename(source, target) {
                Err(error)
                    if matches!(error.raw_os_error(), Some(5 | 32 | 33))
                        && std::time::Instant::now() < deadline =>
                {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                result => return result,
            }
        }
    }
    #[cfg(not(windows))]
    std::fs::rename(source, target)
}

/// 受控子进程所需的最小系统路径，不继承宿主 PATH。
pub fn system_path_dirs() -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        let root = windows_root();
        vec![root.join("System32"), root]
    }
    #[cfg(not(windows))]
    {
        ["/usr/bin", "/bin", "/usr/sbin", "/sbin"]
            .into_iter()
            .map(PathBuf::from)
            .collect()
    }
}

pub fn controlled_path(
    directories: impl IntoIterator<Item = PathBuf>,
    injected: Option<&std::ffi::OsStr>,
) -> std::ffi::OsString {
    let mut paths = Vec::new();
    for path in directories
        .into_iter()
        .chain(injected.into_iter().flat_map(std::env::split_paths))
        .chain(system_path_dirs())
    {
        if path.is_absolute() && std::env::join_paths([&path]).is_ok() && !paths.contains(&path) {
            paths.push(path);
        }
    }
    std::env::join_paths(paths).expect("validated controlled PATH components")
}

#[cfg(windows)]
fn windows_root() -> PathBuf {
    std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
}

/// env_clear 后补齐明确允许的用户和系统目录，不透传任意宿主环境或凭据。
pub fn apply_user_environment(command: &mut std::process::Command, home: Option<&std::ffi::OsStr>) {
    if let Some(home) = home {
        command.env("HOME", home);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW：后台 CLI 不闪控制台。
        let root = windows_root();
        command.env("SystemRoot", &root).env("WINDIR", &root);
        command.env("COMSPEC", root.join("System32/cmd.exe"));
        let temp = std::env::temp_dir();
        command.env("TEMP", &temp).env("TMP", &temp);
        if let Some(home) = home.map(PathBuf::from) {
            command.env("USERPROFILE", &home);
            // 尊重本机重定向的 AppData；隔离测试 HOME 则使用自身目录。
            let daily = home_dir().as_ref() == Some(&home);
            let roaming = daily
                .then(dirs::data_dir)
                .flatten()
                .unwrap_or_else(|| home.join("AppData/Roaming"));
            let local = daily
                .then(dirs::data_local_dir)
                .flatten()
                .unwrap_or_else(|| home.join("AppData/Local"));
            command.env("APPDATA", roaming).env("LOCALAPPDATA", local);
        }
    }
}

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
