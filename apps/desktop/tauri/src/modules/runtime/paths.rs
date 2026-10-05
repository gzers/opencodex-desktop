//! 运行来源涉及的两类路径校验，**口径不同，不可混用**。
//!
//! - `validate_executable`（读路径）：判断某个候选 `ocx` 能不能用。
//!   来源可能是 Homebrew（`/opt/homebrew/bin`、`/usr/local/bin`）或 npm 全局前缀，
//!   这些路径**通常就是符号链接**，所以这里跟随符号链接、也不把系统目录当禁区——
//!   否则会把用户正经装好的 OpenCodex 判成不可用。
//! - `validate_install_target`（写路径，IMP `FZ-47`）：判断我们能不能往某个目录**写入**。
//!   这里必须拒绝符号链接与系统保护目录；本函数在 B2 托管安装使用。

use std::path::{Path, PathBuf};

/// 候选被拒绝的原因。解析层只用它做诊断，不直接暴露给界面。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathRejection {
    NotAbsolute,
    Missing,
    NotRegularFile,
    NotExecutable,
    SymlinkComponent,
    SystemProtected,
    NotDirectory,
    NotWritable,
}

impl PathRejection {
    /// 稳定标识；用于日志与界面文案映射，不随中文文案变动。
    pub fn code(self) -> &'static str {
        match self {
            PathRejection::NotAbsolute => "not_absolute",
            PathRejection::Missing => "missing",
            PathRejection::NotRegularFile => "not_regular_file",
            PathRejection::NotExecutable => "not_executable",
            PathRejection::SymlinkComponent => "symlink_component",
            PathRejection::SystemProtected => "system_protected",
            PathRejection::NotDirectory => "not_directory",
            PathRejection::NotWritable => "not_writable",
        }
    }
}

/// 系统保护目录：托管安装**写入**时一律拒绝（`FZ-47`）。
///
/// 有意不含 `/var` 与 `/tmp`：macOS 上它们本身是系统符号链接，且 `/var/folders`
/// 是标准的按用户临时区，把它们当禁区既拦不住真实威胁、又会让合法临时路径无法使用。
///
/// 只用于写入判定。读取（发现 / 显式指定运行来源）不受此限，
/// 否则 Homebrew 与 npm 全局前缀下的正常安装会被误伤。
pub fn is_system_protected(path: &Path) -> bool {
    #[cfg(windows)]
    {
        if path.is_absolute() && path.parent().is_none() {
            return true;
        }
        let candidate = path.to_string_lossy().replace('/', "\\").to_lowercase();
        [
            "SystemRoot",
            "ProgramFiles",
            "ProgramFiles(x86)",
            "ProgramData",
        ]
        .into_iter()
        .filter_map(std::env::var_os)
        .map(|root| {
            root.to_string_lossy()
                .replace('/', "\\")
                .trim_end_matches('\\')
                .to_lowercase()
        })
        .any(|root| candidate == root || candidate.starts_with(&(root + "\\")))
    }
    #[cfg(not(windows))]
    {
        const PROTECTED: &[&str] = &[
            "/",
            "/bin",
            "/sbin",
            "/usr",
            "/etc",
            "/System",
            "/Library",
            "/private",
            "/Applications",
        ];
        PROTECTED.iter().any(|root| path == Path::new(root))
            || PROTECTED
                .iter()
                .any(|root| *root != "/" && path.starts_with(root))
    }
}

fn is_symlink(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .map(|meta| meta.file_type().is_symlink())
        .unwrap_or(false)
}

/// 路径自身或**受管范围内**的任一层父目录是否为符号链接。
///
/// `scope` 用于把检查限在用户自己的区域内（通常传 `HOME`）：系统前缀里
/// 由操作系统自己维护的符号链接（如 macOS 的 `/var` → `private/var`）不算逃逸，
/// 而用户区域里出现的符号链接必须当重定向处理。目标自身永远是检查对象。
pub fn has_symlink_component(path: &Path, scope: Option<&Path>) -> bool {
    if is_symlink(path) {
        return true;
    }
    let mut current = path.parent();
    while let Some(candidate) = current {
        if candidate.as_os_str().is_empty() {
            break;
        }
        let in_scope = scope
            .map(|scope| candidate.starts_with(scope))
            .unwrap_or(true);
        if in_scope && is_symlink(candidate) {
            return true;
        }
        current = candidate.parent().filter(|parent| *parent != candidate);
    }
    false
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|meta| {
            meta.is_file()
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| {
                        ["exe", "com", "cmd", "bat"].contains(&ext.to_ascii_lowercase().as_str())
                    })
        })
        .unwrap_or(false)
}

#[cfg(all(test, windows))]
mod windows_tests {
    use super::*;

    #[test]
    fn windows_only_accepts_supported_command_entries() {
        let root = tempfile::tempdir().unwrap();
        for name in ["ocx.exe", "ocx.cmd", "ocx.bat"] {
            let path = root.path().join(name);
            std::fs::write(&path, b"fixture").unwrap();
            assert_eq!(validate_executable(&path), Ok(path));
        }
        let path = root.path().join("ocx");
        std::fs::write(&path, b"#!/bin/sh").unwrap();
        assert_eq!(
            validate_executable(&path),
            Err(PathRejection::NotExecutable)
        );
    }

    #[test]
    fn windows_install_rejects_system_and_drive_roots() {
        assert!(is_system_protected(Path::new(r"C:\")));
        let system = std::env::var_os("SystemRoot").unwrap();
        let path = PathBuf::from(system).join("OpenCodex");
        assert_eq!(
            validate_install_target(&path, None),
            Err(PathRejection::SystemProtected)
        );
        assert!(is_system_protected(Path::new(
            &path.to_string_lossy().to_uppercase()
        )));
        let root = tempfile::tempdir().unwrap();
        assert!(!is_system_protected(root.path()));
    }
}

/// 读路径校验：候选 `ocx` 是否为**可执行的普通文件**（跟随符号链接）。
///
/// 不检查系统目录、不拒绝符号链接——发现 Homebrew / npm 全局安装时这两种情况都很常见。
pub fn validate_executable(path: &Path) -> Result<PathBuf, PathRejection> {
    if !path.is_absolute() {
        return Err(PathRejection::NotAbsolute);
    }
    if std::fs::metadata(path).is_err() {
        return Err(PathRejection::Missing);
    }
    if !path.is_file() {
        return Err(PathRejection::NotRegularFile);
    }
    if !is_executable(path) {
        return Err(PathRejection::NotExecutable);
    }
    Ok(path.to_path_buf())
}

/// 写路径校验：托管安装的自定义前缀（`FZ-47`）。
///
/// 比读路径严格得多：绝对路径、非符号链接（目标自身，以及 `scope` 内的父目录）、
/// 非系统保护目录、目录可写。目标可以尚不存在（父目录存在即可）。
///
/// `scope` 传用户的主目录：系统前缀里的符号链接不判逃逸，用户区域里的判。
pub fn validate_install_target(
    path: &Path,
    scope: Option<&Path>,
) -> Result<PathBuf, PathRejection> {
    if !path.is_absolute() {
        return Err(PathRejection::NotAbsolute);
    }
    if has_symlink_component(path, scope) {
        return Err(PathRejection::SymlinkComponent);
    }
    if is_system_protected(path) {
        return Err(PathRejection::SystemProtected);
    }
    if path.exists() {
        if !path.is_dir() {
            return Err(PathRejection::NotDirectory);
        }
        let probe = path.join(".opencodex-write-probe");
        if std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&probe)
            .is_err()
        {
            return Err(PathRejection::NotWritable);
        }
        let _ = std::fs::remove_file(&probe);
        return Ok(path.to_path_buf());
    }
    let parent = path.parent().ok_or(PathRejection::NotAbsolute)?;
    if !parent.is_dir() {
        return Err(PathRejection::Missing);
    }
    Ok(path.to_path_buf())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;

    fn write_executable(path: &Path) {
        std::fs::write(path, b"#!/bin/sh\nexit 0\n").expect("write fixture");
        std::fs::set_permissions(path, Permissions::from_mode(0o755)).expect("chmod fixture");
    }

    #[test]
    fn executable_accepts_relative_rejection_for_non_absolute() {
        assert_eq!(
            validate_executable(Path::new("ocx")).unwrap_err(),
            PathRejection::NotAbsolute
        );
    }

    #[test]
    fn executable_rejects_missing_and_non_executable() {
        let root = tempfile::tempdir().expect("temp");
        let missing = root.path().join("ocx");
        assert_eq!(
            validate_executable(&missing).unwrap_err(),
            PathRejection::Missing
        );

        let plain = root.path().join("plain");
        std::fs::write(&plain, b"not executable").expect("write");
        assert_eq!(
            validate_executable(&plain).unwrap_err(),
            PathRejection::NotExecutable
        );
    }

    #[test]
    fn executable_follows_symlinks_and_allows_system_prefixes() {
        // 发现 Homebrew / npm 全局安装时，`ocx` 常常是符号链接；
        // 读路径必须接受，否则会把用户正当装好的 OpenCodex 判成不可用。
        let root = tempfile::tempdir().expect("temp");
        let real = root.path().join("real-ocx");
        write_executable(&real);
        let link = root.path().join("ocx-link");
        std::os::unix::fs::symlink(&real, &link).expect("symlink");

        assert_eq!(validate_executable(&link), Ok(link.clone()));
        // `/usr/local/bin` 这类路径属系统前缀，但读取不受限。
        assert!(is_system_protected(Path::new("/usr/local/bin")));
    }

    #[test]
    fn install_target_rejects_symlink_and_protected_dirs() {
        let root = tempfile::tempdir().expect("temp");
        let real = root.path().join("real");
        std::fs::create_dir(&real).expect("mkdir");
        let link = root.path().join("link");
        std::os::unix::fs::symlink(&real, &link).expect("symlink");

        assert_eq!(
            validate_install_target(&link, Some(root.path())).unwrap_err(),
            PathRejection::SymlinkComponent
        );
        assert_eq!(
            validate_install_target(Path::new("/usr/lib/opencodex"), None).unwrap_err(),
            PathRejection::SystemProtected
        );
        assert_eq!(
            validate_install_target(Path::new("/"), None).unwrap_err(),
            PathRejection::SystemProtected
        );
        assert_eq!(
            validate_install_target(Path::new("/Applications/OpenCodex"), None).unwrap_err(),
            PathRejection::SystemProtected
        );
    }

    #[test]
    fn install_target_accepts_writable_directory_and_creatable_child() {
        let root = tempfile::tempdir().expect("temp");
        let existing = root.path().join("prefix");
        std::fs::create_dir(&existing).expect("mkdir");
        assert_eq!(
            validate_install_target(&existing, Some(root.path())),
            Ok(existing.clone())
        );

        let creatable = root.path().join("not-yet");
        assert_eq!(
            validate_install_target(&creatable, Some(root.path())),
            Ok(creatable.clone())
        );
    }

    #[test]
    fn install_target_rejects_regular_file_and_non_absolute() {
        let root = tempfile::tempdir().expect("temp");
        let file = root.path().join("file");
        std::fs::write(&file, b"x").expect("write");
        assert_eq!(
            validate_install_target(&file, Some(root.path())).unwrap_err(),
            PathRejection::NotDirectory
        );
        assert_eq!(
            validate_install_target(Path::new("relative"), Some(root.path())).unwrap_err(),
            PathRejection::NotAbsolute
        );
    }

    #[test]
    fn symlink_component_detection_walks_in_scope_ancestors_only() {
        let root = tempfile::tempdir().expect("temp");
        let real = root.path().join("real");
        std::fs::create_dir_all(real.join("nested")).expect("mkdir");
        let link = root.path().join("link");
        std::os::unix::fs::symlink(&real, &link).expect("symlink");

        // 用户区域内：父目录的符号链接要判逃逸。
        assert!(has_symlink_component(
            &link.join("nested"),
            Some(root.path())
        ));
        assert!(!has_symlink_component(
            &real.join("nested"),
            Some(root.path())
        ));
        // 系统区域里的符号链接（如 macOS `/var` → `private/var`）不判逃逸，
        // 否则所有落在 /var 下的合法路径都会被误伤。
        assert!(!has_symlink_component(
            Path::new("/private/tmp/ocx-fixture"),
            Some(root.path())
        ));
    }
}
