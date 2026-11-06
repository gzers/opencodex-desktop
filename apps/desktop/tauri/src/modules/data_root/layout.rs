//! Version 2 splits immutable installed programs, manager state and instance HOME.
//! Pure path resolution: never scans legacy directories, initializes files or reads PATH.
use crate::errors::AppError;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Macos,
    Windows,
    Unix,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub aggregate_root: PathBuf,
    pub desktop_root: PathBuf,
    pub opencodex_home: PathBuf,
    pub installation_root: PathBuf,
    pub runtime_prefix: PathBuf,
    pub runtime_entry: PathBuf,
}

fn absolute_clean(path: &Path) -> bool {
    path.is_absolute() && !path.components().any(|p| matches!(p, Component::ParentDir))
}
fn invalid(detail: &str) -> AppError {
    AppError::FileSystem {
        operation: "resolve version 2 layout".into(),
        detail: detail.into(),
    }
}

impl Layout {
    /// Explicit install context permits isolated development without guessing cwd.
    pub fn new(
        user_home: &Path,
        installation_root: &Path,
        platform: Platform,
    ) -> Result<Self, AppError> {
        if !absolute_clean(user_home) || !absolute_clean(installation_root) {
            return Err(invalid("absolute user and installation roots are required"));
        }
        let aggregate_root = user_home.join(".codex-desktop");
        Self::with_aggregate_root(&aggregate_root, installation_root, platform)
    }

    pub fn with_aggregate_root(
        root: &Path,
        install: &Path,
        platform: Platform,
    ) -> Result<Self, AppError> {
        if !absolute_clean(root) || !absolute_clean(install) {
            return Err(invalid(
                "absolute aggregate and installation roots are required",
            ));
        }
        let root = super::startup_v2::checked_path(root)?;
        let install = super::startup_v2::checked_path(install)?;
        if root.starts_with(&install) || install.starts_with(&root) {
            return Err(invalid("installed programs and user data must be separate"));
        }
        Ok(Self {
            aggregate_root: root.clone(),
            desktop_root: root.join("desktop"),
            opencodex_home: root.join("opencodex/home"),
            installation_root: install.clone(),
            runtime_prefix: install.join("runtime/opencodex"),
            runtime_entry: install.join(if platform == Platform::Windows {
                "runtime/bin/ocx.cmd"
            } else {
                "runtime/bin/ocx"
            }),
        })
    }

    pub fn location_file(user_home: &Path) -> Result<PathBuf, AppError> {
        if !absolute_clean(user_home) {
            return Err(invalid("absolute current-user HOME is required"));
        }
        Ok(user_home.join(".codex-desktop/desktop/manager-location.json"))
    }

    /// Resolve only the running application location; no other installation fallback.
    pub fn installation_root(executable: &Path, platform: Platform) -> Result<PathBuf, AppError> {
        if !absolute_clean(executable) {
            return Err(invalid("absolute current executable is required"));
        }
        if platform == Platform::Macos {
            let macos = executable
                .parent()
                .ok_or_else(|| invalid("missing app executable directory"))?;
            let contents = macos
                .parent()
                .ok_or_else(|| invalid("missing app Contents"))?;
            let bundle = contents
                .parent()
                .ok_or_else(|| invalid("missing app bundle"))?;
            if macos.file_name().is_some_and(|v| v == "MacOS")
                && contents.file_name().is_some_and(|v| v == "Contents")
                && bundle.extension().is_some_and(|v| v == "app")
            {
                return bundle
                    .parent()
                    .map(Path::to_path_buf)
                    .ok_or_else(|| invalid("missing app installation root"));
            }
            return Err(invalid(
                "development executables require explicit installation context",
            ));
        }
        executable
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| invalid("missing executable installation directory"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distinct_users_share_program_but_never_state_or_home() {
        let temp = tempfile::tempdir().unwrap();
        let install = temp.path().join("program");
        let a = Layout::new(&temp.path().join("alice"), &install, Platform::Windows).unwrap();
        let b = Layout::new(&temp.path().join("bob"), &install, Platform::Windows).unwrap();
        assert_eq!(a.runtime_entry, b.runtime_entry);
        assert_ne!(a.desktop_root, b.desktop_root);
        assert_ne!(a.opencodex_home, b.opencodex_home);
        assert!(a.runtime_entry.ends_with("runtime/bin/ocx.cmd"));
        assert!(!a.aggregate_root.exists()); // resolution never creates real data
    }
    #[test]
    fn mac_runtime_is_sibling_of_signed_bundle() {
        let t = tempfile::tempdir().unwrap();
        let executable = t
            .path()
            .join("installation/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop");
        let install = Layout::installation_root(&executable, Platform::Macos).unwrap();
        let l = Layout::new(&t.path().join("user"), &install, Platform::Macos).unwrap();
        assert_eq!(
            l.runtime_prefix,
            super::super::startup_v2::checked_path(
                &t.path().join("installation/runtime/opencodex")
            )
            .unwrap()
        );
        assert_eq!(
            l.desktop_root,
            super::super::startup_v2::checked_path(&t.path().join("user/.codex-desktop/desktop"))
                .unwrap()
        );
        assert_eq!(
            l.opencodex_home,
            super::super::startup_v2::checked_path(
                &t.path().join("user/.codex-desktop/opencodex/home")
            )
            .unwrap()
        );
    }
    #[test]
    fn refuses_relative_parent_and_nested_program_roots() {
        let t = tempfile::tempdir().unwrap();
        assert!(Layout::new(Path::new("relative"), t.path(), Platform::Unix).is_err());
        assert!(Layout::new(
            &t.path().join("a/../b"),
            &t.path().join("install"),
            Platform::Unix
        )
        .is_err());
        let root = t.path().join("data");
        assert!(Layout::with_aggregate_root(&root, &root.join("runtime"), Platform::Unix).is_err());
        assert!(Layout::with_aggregate_root(&root.join("user"), &root, Platform::Unix).is_err());
        assert!(
            Layout::installation_root(&t.path().join("target/debug/app"), Platform::Macos).is_err()
        );
    }

    #[cfg(unix)]
    #[test]
    fn aliases_cannot_hide_nested_installation_or_user_data() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("data");
        std::fs::create_dir(&root).unwrap();
        let alias = t.path().join("alias");
        std::os::unix::fs::symlink(&root, &alias).unwrap();
        assert!(
            Layout::with_aggregate_root(&alias, &root.join("program"), Platform::Unix).is_err()
        );
        assert!(Layout::with_aggregate_root(&alias.join("data"), &root, Platform::Unix).is_err());
    }
}
