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

/// Read-only admission for a dedicated runtime program directory.
/// The signed .app is a read-only resource source, never this write target.
pub(crate) fn validate_macos_installation_root(
    install: &Path,
    user_home: Option<&Path>,
) -> Result<PathBuf, AppError> {
    let install = super::startup_v2::checked_path(install)?;
    let home = user_home
        .map(Path::to_path_buf)
        .or_else(crate::infrastructure::platform::home_dir)
        .ok_or_else(|| invalid("current-user HOME is required to verify installation root"))?;
    if !absolute_clean(&home) {
        return Err(invalid("absolute current-user HOME is required"));
    }
    let case_sensitive = macos_case_sensitive(&install);
    let same = |boundary: &Path| macos_same_location(&install, boundary, case_sensitive);
    let mut container = install.parent().is_none();
    for boundary in [
        "/Applications",
        "/System/Applications",
        "/System",
        "/Library",
        "/Users",
        "/Volumes",
        "/System/Volumes",
        "/System/Volumes/Data",
        "/System/Volumes/Data/Applications",
        "/usr",
        "/usr/local",
        "/opt",
        "/private",
        "/tmp",
        "/var",
    ] {
        container |= same(Path::new(boundary))?;
    }
    container |= same(&home)? || same(&home.join("Applications"))?;
    if let Some(parent) = install.parent() {
        let parent = super::startup_v2::checked_path(parent)?;
        // A second user's top-level Applications remains a container, while
        // its dedicated product subfolder is still a valid shared installation.
        let applications_name = install.file_name().is_some_and(|name| {
            name == "Applications" || (!case_sensitive && name.eq_ignore_ascii_case("Applications"))
        });
        if applications_name {
            if let Some(profile_parent) = parent.parent() {
                for boundary in ["/Users", "/Volumes"] {
                    container |=
                        macos_same_location(profile_parent, Path::new(boundary), case_sensitive)?;
                }
            }
        }
        // Standard volume and user-profile roots are containers, not product I.
        for boundary in ["/Volumes", "/Users"] {
            if Path::new(boundary).is_absolute() {
                container |= macos_same_location(&parent, Path::new(boundary), case_sensitive)?;
            }
        }
        #[cfg(unix)]
        if let (Ok(a), Ok(b)) = (std::fs::metadata(&install), std::fs::metadata(&parent)) {
            use std::os::unix::fs::MetadataExt;
            container |= a.dev() != b.dev(); // actual mounted-volume root
        }
    }
    if container {
        return Err(AppError::RuntimeManaged {
            code: "macos_install_container".into(),
            detail: "运行程序需要独立目录，不能直接使用 Applications、用户 HOME 或卷根。Desktop 应用可以直接放在 Applications；运行程序存放在当前用户的独立程序目录。原应用与配置未移动。".into(),
        });
    }
    Ok(install)
}

fn macos_same_location(a: &Path, b: &Path, case_sensitive: bool) -> Result<bool, AppError> {
    // Macos layout fixtures can run on Windows, whose static roots differ.
    if !b.is_absolute() {
        return Ok(false);
    }
    let b = super::startup_v2::checked_path(b)?;
    if a == b {
        return Ok(true);
    }
    #[cfg(unix)]
    if let (Ok(a), Ok(b)) = (std::fs::metadata(a), std::fs::metadata(&b)) {
        use std::os::unix::fs::MetadataExt;
        // Includes firmlinks/aliases that realpath need not spell identically.
        return Ok(a.dev() == b.dev() && a.ino() == b.ino());
    }
    Ok(!case_sensitive && a.as_os_str().eq_ignore_ascii_case(b.as_os_str()))
}

fn macos_case_sensitive(path: &Path) -> bool {
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::ffi::OsStrExt;
        let mut ancestor = path;
        while !ancestor.exists() {
            let Some(parent) = ancestor.parent() else {
                return false;
            };
            ancestor = parent;
        }
        let Ok(path) = std::ffi::CString::new(ancestor.as_os_str().as_bytes()) else {
            return false;
        };
        // SAFETY: a live NUL-terminated path; pathconf is read-only. Unknown
        // filesystems conservatively fold only these exact reserved locations.
        unsafe { libc::pathconf(path.as_ptr(), libc::_PC_CASE_SENSITIVE) > 0 }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        true
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
        if platform == Platform::Macos {
            validate_macos_installation_root(installation_root, Some(user_home))?;
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
        if platform == Platform::Macos {
            validate_macos_installation_root(&install, None)?;
        }
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
        let home = crate::infrastructure::platform::home_dir()
            .ok_or_else(|| invalid("absolute current-user HOME is required"))?;
        Self::installation_root_for_user(executable, &home, platform)
    }

    /// The program root is independent of both the .app location and the
    /// configurable data root. Passing HOME explicitly keeps startup, supply,
    /// installation and CLI consumers on the same current-user identity.
    pub fn installation_root_for_user(
        executable: &Path,
        user_home: &Path,
        platform: Platform,
    ) -> Result<PathBuf, AppError> {
        let locations = crate::modules::runtime::program_layout::ProgramLocations::resolve(
            executable, user_home, platform,
        )?;
        if platform == Platform::Macos {
            return validate_macos_installation_root(&locations.program_root, Some(user_home));
        }
        Ok(locations.program_root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mac_standard_app_uses_current_user_program_directory() {
        // Resolve only: never create or launch anything in the real container.
        let t = tempfile::tempdir().unwrap();
        let home = t.path().join("current-user");
        let program = Layout::installation_root_for_user(
            Path::new("/Applications/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop"),
            &home,
            Platform::Macos,
        )
        .unwrap();
        assert_eq!(
            program,
            home.join("Library/Application Support/OpenCodeX Desktop")
        );
        assert!(!home.exists());
    }
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
    fn mac_runtime_is_separate_from_signed_bundle_and_data() {
        let t = tempfile::tempdir().unwrap();
        let executable = t
            .path()
            .join("installation/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop");
        let home = t.path().join("user");
        let install =
            Layout::installation_root_for_user(&executable, &home, Platform::Macos).unwrap();
        let l = Layout::new(&home, &install, Platform::Macos).unwrap();
        assert_eq!(
            l.runtime_prefix,
            super::super::startup_v2::checked_path(
                &home.join("Library/Application Support/OpenCodeX Desktop/runtime/opencodex")
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

    #[cfg(target_os = "macos")]
    #[test]
    fn mac_shared_containers_and_volume_roots_are_refused_read_only() {
        let t = tempfile::tempdir().unwrap();
        let user = t.path().join("profile");
        for container in [
            "/",
            "/Applications",
            "/System/Applications",
            "/Users",
            "/Volumes",
            "/Volumes/ocxd023-unmounted-fixture",
            "/System/Volumes/Data",
            "/System/Volumes/Data/Applications",
            "/Users/ocxd023-nonexistent-profile/Applications",
            "/usr/local",
            "/private/tmp",
        ] {
            let error =
                validate_macos_installation_root(Path::new(container), Some(&user)).unwrap_err();
            assert!(
                matches!(error, AppError::RuntimeManaged { ref code, .. }
                if code == "macos_install_container"),
                "{container}: {error}"
            );
        }
        assert!(!user.exists());
    }

    #[test]
    fn mac_explicit_home_and_personal_applications_are_containers() {
        let t = tempfile::tempdir().unwrap();
        let user = t.path().join("profile");
        for install in [&user, &user.join("Applications")] {
            assert!(validate_macos_installation_root(install, Some(&user)).is_err());
        }
        assert!(!user.exists(), "admission must not initialize HOME");
        let install = user.join("Applications/OpenCodeX Desktop");
        assert!(Layout::new(&user, &install, Platform::Macos).is_ok());
        assert!(!user.exists());
    }

    #[cfg(unix)]
    #[test]
    fn mac_container_aliases_are_refused_without_touching_old_bundle_or_config() {
        let t = tempfile::tempdir().unwrap();
        let user = t.path().join("profile");
        let apps = user.join("Applications");
        let bundle = apps.join("OpenCodeX Desktop.app/Contents/MacOS");
        std::fs::create_dir_all(&bundle).unwrap();
        let old_app = bundle.join("opencodex-desktop");
        let old_config = user.join("old-config-sentinel");
        std::fs::write(&old_app, b"old app must stay").unwrap();
        std::fs::write(&old_config, b"old config must stay").unwrap();
        let alias = t.path().join("application-container-alias");
        std::os::unix::fs::symlink(&apps, &alias).unwrap();
        let home_alias = t.path().join("home-alias");
        std::os::unix::fs::symlink(&user, &home_alias).unwrap();
        for path in [&apps, &alias, &user, &home_alias] {
            assert!(validate_macos_installation_root(path, Some(&user)).is_err());
        }
        assert_eq!(std::fs::read(old_app).unwrap(), b"old app must stay");
        assert_eq!(std::fs::read(old_config).unwrap(), b"old config must stay");
        assert!(!apps.join("runtime").exists());
        assert!(!apps.join(".runtime-bundle.lock").exists());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn mac_container_case_comparison_follows_the_actual_volume() {
        let t = tempfile::tempdir().unwrap();
        let user = t.path().join("profile");
        std::fs::create_dir(&user).unwrap();
        // Test the missing reserved directory too; canonicalizing existing paths
        // alone would miss it on a case-insensitive volume.
        let lower = user.join("applications");
        // Independent filesystem observation, not the comparator's own answer.
        std::fs::write(user.join("Case-Probe"), b"temporary case probe").unwrap();
        let sensitive = !user.join("case-probe").exists();
        assert_eq!(macos_case_sensitive(&user), sensitive);
        assert_eq!(
            validate_macos_installation_root(&lower, Some(&user)).is_ok(),
            sensitive
        );
        std::fs::create_dir(user.join("Applications")).unwrap();
        assert_eq!(
            validate_macos_installation_root(&lower, Some(&user)).is_ok(),
            sensitive
        );
        assert!(!lower.join("runtime").exists());
    }

    #[cfg(unix)]
    #[test]
    fn mac_dedicated_custom_and_shared_read_only_folders_remain_valid() {
        use std::os::unix::fs::PermissionsExt;
        let t = tempfile::tempdir().unwrap();
        let user = t.path().join("profile");
        let i = t.path().join("custom programs/OpenCodeX Desktop");
        std::fs::create_dir_all(&i).unwrap();
        std::fs::set_permissions(&i, std::fs::Permissions::from_mode(0o555)).unwrap();
        let before = std::fs::metadata(&i).unwrap().permissions().mode();
        assert!(validate_macos_installation_root(&i, Some(&user)).is_ok());
        assert!(validate_macos_installation_root(&i, Some(&t.path().join("second-user"))).is_ok());
        assert_eq!(std::fs::metadata(&i).unwrap().permissions().mode(), before);
        assert_eq!(std::fs::read_dir(&i).unwrap().count(), 0);
        std::fs::set_permissions(&i, std::fs::Permissions::from_mode(0o755)).unwrap();
        // /tmp's OS alias is allowed for a dedicated descendant, not the container.
        assert!(Layout::new(&user, &i, Platform::Macos).is_ok());
        assert!(!user.exists());
    }

    #[test]
    fn windows_layout_does_not_gain_macos_container_policy() {
        let t = tempfile::tempdir().unwrap();
        let install = t.path().join("Applications");
        let layout = Layout::new(&t.path().join("profile"), &install, Platform::Windows).unwrap();
        assert!(layout.runtime_entry.ends_with("runtime/bin/ocx.cmd"));
        assert!(!install.exists());
    }
}
