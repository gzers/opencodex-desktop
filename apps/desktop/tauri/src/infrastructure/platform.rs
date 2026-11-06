//! 跨平台文件系统辅助：权限位与符号链接。
//!
//! Windows 通过实际用户 token SID 校验拥有者并设置私有 ACL；
//! 不接管其它拥有者、不用环境变量代替账户身份。符号链接按
//! 平台选择 `symlink`（Unix）或 `symlink_file`（Windows）。

use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use std::path::PathBuf;

/// Business file sources use the explicit Sandbox home on every platform.
/// Keep the process HOME unchanged on macOS so Security.framework can locate
/// the user's native keychain; the Sandbox service/account namespace is separate.
pub fn home_dir() -> Option<PathBuf> {
    isolated_home(
        crate::modules::test_sandbox::enabled(),
        crate::modules::test_sandbox::sandbox_root().as_deref(),
        current_profile,
    )
}
fn current_profile() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        crate::infrastructure::windows_startup_root::current_user_profile().ok()
    }
    #[cfg(not(windows))]
    {
        dirs::home_dir()
    }
}
fn isolated_home(
    sandbox_enabled: bool,
    sandbox_root: Option<&Path>,
    daily_home: impl FnOnce() -> Option<PathBuf>,
) -> Option<PathBuf> {
    if sandbox_enabled {
        return sandbox_root
            .filter(|root| root.is_absolute())
            .map(|root| root.join("home"));
    }
    daily_home().filter(|path| path.is_absolute() && !path.as_os_str().is_empty())
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
        command.env("COMSPEC", root.join("System32").join("cmd.exe"));
        let temp = std::env::temp_dir();
        command.env("TEMP", &temp).env("TMP", &temp);
        if let Some(home) = home.map(PathBuf::from) {
            command.env("USERPROFILE", &home);
            // 尊重本机重定向的 AppData；隔离测试 HOME 则使用自身目录。
            let daily =
                !crate::modules::test_sandbox::enabled() && home_dir().as_ref() == Some(&home);
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

/// Official model commands share the same HOME and controlled tool discovery as
/// lifecycle/status commands. In particular, a sandbox must never re-add host HOME/PATH.
pub fn apply_runtime_environment(command: &mut std::process::Command, executable: &Path) {
    let home = home_dir();
    apply_user_environment(command, home.as_deref().map(Path::as_os_str));
    let discovered = home.as_deref().map(|home| {
        super::discovery_paths::process_path(home, &super::discovery_paths::paths_for_home(home))
    });
    command.env(
        "PATH",
        controlled_path(
            executable.parent().map(Path::to_path_buf),
            discovered.as_deref(),
        ),
    );
}

/// Invoke the Windows Shell with a Unicode object name, without command
/// interpretation or executable lookup through the application's working directory.
#[cfg(windows)]
pub fn shell_open(target: &std::ffi::OsStr) -> crate::errors::AppResult<()> {
    use std::os::windows::ffi::OsStrExt;
    let mut value = target.encode_wide().collect::<Vec<_>>();
    if value.contains(&0) {
        return Err(crate::errors::AppError::NotConfigured);
    }
    value.push(0);
    let result = unsafe {
        windows_sys::Win32::UI::Shell::ShellExecuteW(
            std::ptr::null_mut(),
            std::ptr::null(),
            value.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1,
        )
    } as isize;
    if result <= 32 {
        return Err(crate::errors::AppError::FileSystem {
            operation: "open Windows shell target".into(),
            detail: format!("Windows Shell rejected target (code {result})"),
        });
    }
    Ok(())
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

/// New private objects are owned by the actual TokenUser from creation time.
/// An elevated process may have Administrators as its default TokenOwner.
pub fn create_new_private_file(path: &Path, mode: u32) -> io::Result<std::fs::File> {
    #[cfg(windows)]
    {
        crate::infrastructure::windows_private_create::create_new_private_file(path, mode)
    }
    #[cfg(not(windows))]
    {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .create_mode(mode)
            .open(path)
    }
}
/// Existing private targets may be tightened only after their actual owner is
/// verified. Existing OS ancestors are never modified or taken over.
pub fn create_private_dir_all(path: &Path, mode: u32) -> io::Result<()> {
    #[cfg(windows)]
    {
        match std::fs::symlink_metadata(path) {
            Ok(_) => {
                crate::infrastructure::windows_startup_root::protect_current_user_path(path, mode)?
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(e),
        }
        crate::infrastructure::windows_private_create::create_private_dir_all(path, mode)
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(mode)
            .create(path)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = mode;
        std::fs::create_dir_all(path)
    }
}
/// Create the private lock once; reopening never truncates or rewrites it.
pub fn open_or_create_private_file(path: &Path, mode: u32) -> io::Result<std::fs::File> {
    #[cfg(windows)]
    {
        match create_new_private_file(path, mode) {
            Ok(file) => Ok(file),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                crate::infrastructure::windows_private_create::open_existing_private_file(
                    path, mode,
                )
            }
            Err(e) => Err(e),
        }
    }
    #[cfg(not(windows))]
    {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .create_mode(mode)
            .open(path)
    }
}

/// Unix permission bits; Windows enforces the current token's private ACL.
pub fn set_mode(path: &Path, mode: u32) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(path)?.permissions();
        permissions.set_mode(mode);
        std::fs::set_permissions(path, permissions)
    }
    #[cfg(windows)]
    {
        crate::infrastructure::windows_startup_root::protect_current_user_path(path, mode)
    }
    #[cfg(not(any(unix, windows)))]
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

#[cfg(test)]
mod home_tests {
    use super::*;

    #[test]
    fn sandbox_home_never_reads_the_daily_profile() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(
            isolated_home(true, Some(root.path()), || panic!("daily profile accessed")),
            Some(root.path().join("home"))
        );
        for invalid in [None, Some(Path::new("relative"))] {
            assert_eq!(
                isolated_home(true, invalid, || panic!("daily profile accessed")),
                None
            );
        }
    }

    #[test]
    fn daily_home_keeps_the_known_folder_contract() {
        let daily = tempfile::tempdir().unwrap();
        let ignored_sandbox = tempfile::tempdir().unwrap();
        assert_eq!(
            isolated_home(false, Some(ignored_sandbox.path()), || {
                Some(daily.path().to_path_buf())
            }),
            Some(daily.path().to_path_buf())
        );
        assert_eq!(
            isolated_home(false, None, || Some(PathBuf::from("relative"))),
            None
        );
    }
}
