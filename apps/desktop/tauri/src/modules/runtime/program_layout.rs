//! One program-location input for supply, installation, launch and user CLI.
//! User configuration roots and runtime observations do not select programs.
use crate::{errors::AppError, modules::data_root::layout::Platform};
use std::path::{Component, Path, PathBuf};

pub const CURRENT_FILENAME: &str = "current.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramLocations {
    pub desktop_application: PathBuf,
    pub program_root: PathBuf,
    pub runtime_root: PathBuf,
    pub stable_entry: PathBuf,
    pub versions_root: PathBuf,
    pub current_record: PathBuf,
    platform: Platform,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeVersionPaths {
    pub version_root: PathBuf,
    pub package_root: PathBuf,
    pub node: PathBuf,
    pub npm: PathBuf,
    pub bun: PathBuf,
}

fn invalid(detail: &str) -> AppError {
    AppError::RuntimeManaged {
        code: "runtime_program_path".into(),
        detail: detail.into(),
    }
}

fn clean_absolute(path: &Path) -> bool {
    path.is_absolute()
        && path.to_str().is_some()
        && !path.to_string_lossy().chars().any(char::is_control)
        && !path
            .to_string_lossy()
            .split(['/', '\\'])
            .any(|s| matches!(s, "." | ".."))
        && !path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
}

impl ProgramLocations {
    /// Resolve read-only. No legacy discovery, directory creation or PATH read.
    pub fn resolve(
        executable: &Path,
        user_home: &Path,
        platform: Platform,
    ) -> Result<Self, AppError> {
        if !clean_absolute(executable) || !clean_absolute(user_home) {
            return Err(invalid("程序位置需要明确的绝对路径与当前用户目录"));
        }
        let (desktop_application, program_root) = if platform == Platform::Macos {
            let macos = executable
                .parent()
                .ok_or_else(|| invalid("缺少应用程序目录"))?;
            let contents = macos
                .parent()
                .ok_or_else(|| invalid("缺少 Contents 目录"))?;
            let app = contents.parent().ok_or_else(|| invalid("缺少应用包目录"))?;
            if macos.file_name().is_none_or(|v| v != "MacOS")
                || contents.file_name().is_none_or(|v| v != "Contents")
                || app.extension().is_none_or(|v| v != "app")
            {
                return Err(invalid("开发程序需要显式安装上下文"));
            }
            (
                app.to_path_buf(),
                user_home.join("Library/Application Support/OpenCodeX Desktop"),
            )
        } else {
            (
                executable.to_path_buf(),
                executable
                    .parent()
                    .ok_or_else(|| invalid("缺少安装根"))?
                    .to_path_buf(),
            )
        };
        let runtime_root = program_root.join("runtime");
        Ok(Self {
            desktop_application,
            program_root,
            stable_entry: runtime_root.join(if platform == Platform::Windows {
                "bin/ocx.cmd"
            } else {
                "bin/ocx"
            }),
            versions_root: runtime_root.join("versions"),
            current_record: runtime_root.join(CURRENT_FILENAME),
            runtime_root,
            platform,
        })
    }

    /// Official release version is one immutable directory component. Package
    /// identity/digest verification remains the installer's responsibility.
    pub fn version(&self, version: &str) -> Result<RuntimeVersionPaths, AppError> {
        RuntimeVersionPaths::new(&self.versions_root, version, self.platform)
    }
}

impl RuntimeVersionPaths {
    pub fn new(versions_root: &Path, version: &str, platform: Platform) -> Result<Self, AppError> {
        if !clean_absolute(versions_root) {
            return Err(invalid("版本目录需要明确的绝对路径"));
        }
        let parts = version.split('.').collect::<Vec<_>>();
        if parts.len() != 3
            || parts.iter().any(|p| {
                p.is_empty()
                    || p.len() > 10
                    || !p.bytes().all(|c| c.is_ascii_digit())
                    || (p.len() > 1 && p.starts_with('0'))
                    || p.parse::<u32>().is_err()
            })
        {
            return Err(invalid("运行包版本需要完整的官方发布版本号"));
        }
        let version_root = versions_root.join(version);
        let dependencies = version_root.join("dependencies");
        let windows = platform == Platform::Windows;
        Ok(RuntimeVersionPaths {
            package_root: version_root.join("opencodex"),
            node: dependencies.join(if windows {
                "node/node.exe"
            } else {
                "node/bin/node"
            }),
            npm: dependencies.join("npm/bin/npm-cli.js"),
            bun: dependencies.join(if windows { "bun/bun.exe" } else { "bun/bun" }),
            version_root,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moving_a_mac_app_does_not_move_runtime_or_user_state() {
        let t = tempfile::tempdir().unwrap();
        let home = t.path().join("alice");
        let a = ProgramLocations::resolve(
            &t.path()
                .join("Applications/OpenCodeX Desktop.app/Contents/MacOS/desktop"),
            &home,
            Platform::Macos,
        )
        .unwrap();
        let b = ProgramLocations::resolve(
            &t.path()
                .join("other/OpenCodeX Desktop.app/Contents/MacOS/desktop"),
            &home,
            Platform::Macos,
        )
        .unwrap();
        assert_ne!(a.desktop_application, b.desktop_application);
        assert_eq!(a.program_root, b.program_root);
        assert_eq!(a.version("2.99.0").unwrap(), b.version("2.99.0").unwrap());
        let bob = ProgramLocations::resolve(
            &t.path()
                .join("Applications/OpenCodeX Desktop.app/Contents/MacOS/desktop"),
            &t.path().join("bob"),
            Platform::Macos,
        )
        .unwrap();
        assert_ne!(a.program_root, bob.program_root);
        assert!(!home.exists());
    }

    #[test]
    fn windows_uses_actual_custom_installation_without_guessing_default() {
        let t = tempfile::tempdir().unwrap();
        let custom = t.path().join("custom program location");
        let p = ProgramLocations::resolve(
            &custom.join("opencodex-desktop.exe"),
            &t.path().join("user"),
            Platform::Windows,
        )
        .unwrap();
        assert_eq!(p.program_root, custom);
        assert_eq!(p.stable_entry, custom.join("runtime/bin/ocx.cmd"));
        assert_eq!(
            p.version("2.99.0").unwrap().node,
            custom.join("runtime/versions/2.99.0/dependencies/node/node.exe")
        );
    }

    #[test]
    fn version_and_source_inputs_cannot_escape_the_program_directory() {
        let t = tempfile::tempdir().unwrap();
        let p = ProgramLocations::resolve(
            &t.path().join("program/desktop"),
            &t.path().join("user"),
            Platform::Unix,
        )
        .unwrap();
        for value in [
            "../2.99.0",
            "2.99.0/../other",
            "2.99.0\\other",
            "2.99.0:stream",
            "02.99.0",
            "2.99",
            "2.99.0-dev",
            "4294967296.0.0",
        ] {
            assert!(p.version(value).is_err(), "{value}");
        }
        assert!(ProgramLocations::resolve(
            &t.path().join("a/../desktop"),
            t.path(),
            Platform::Unix
        )
        .is_err());
        assert!(!p.runtime_root.exists());
    }
}
