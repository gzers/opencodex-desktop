//! A durable gate outside D keeps admission closed while D/H are removed.
//! Never unlink this file: replacing its inode would split the lock authority.
use crate::errors::{AppError, AppResult};
use std::{
    fs::File,
    path::{Path, PathBuf},
};

const NAME: &str = ".desktop-removal.lock";
fn fail(error: impl ToString) -> AppError {
    AppError::FileSystem {
        operation: "admit user data removal".into(),
        detail: error.to_string(),
    }
}
fn gate(desktop: &Path) -> AppResult<(PathBuf, File)> {
    let parent = desktop
        .parent()
        .ok_or_else(|| fail("missing explicit aggregate"))?;
    if !desktop.is_absolute() || desktop.file_name().is_none() {
        return Err(fail("absolute explicit desktop root required"));
    }
    // OS profile aliases are normalized; managed aggregate links are refused.
    if std::fs::symlink_metadata(parent).is_ok() {
        crate::modules::backup::safety::inspect(parent)?;
    }
    let parent = super::startup_v2::checked_path(parent)?;
    super::startup_v2::check_owner(&parent)?;
    if !parent.exists() {
        crate::infrastructure::platform::create_private_dir_all(&parent, 0o700).map_err(fail)?;
    }
    let path = parent.join(NAME);
    #[cfg(windows)]
    let file =
        crate::infrastructure::platform::open_or_create_private_file(&path, 0o600).map_err(fail)?;
    #[cfg(not(windows))]
    let file = {
        use std::fs::OpenOptions;
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC);
        }
        options.open(&path).map_err(fail)?
    };
    let metadata = file.metadata().map_err(fail)?;
    if !metadata.is_file() {
        return Err(fail("gate is not a regular file"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.uid() != unsafe { libc::geteuid() }
            || metadata.mode() & 0o7777 != 0o600
            || metadata.nlink() != 1
        {
            return Err(fail("gate owner, mode or link count differs"));
        }
    }
    Ok((parent.join(desktop.file_name().unwrap()), file))
}

#[derive(Debug)]
pub(crate) struct SharedLease {
    _file: File,
}
pub(crate) fn shared(desktop: &Path) -> AppResult<SharedLease> {
    let (_, file) = gate(desktop)?;
    file.try_lock_shared()
        .map_err(|_| AppError::InstanceLockConflict)?;
    Ok(SharedLease { _file: file })
}

// Construction is private to this module. Only an exclusive lease can authorize
// taking the inner locks without reentering the shared admission.
#[cfg(any(windows, test))]
pub(crate) struct RemovalLeaseSet {
    roots: Vec<PathBuf>,
    _files: Vec<File>,
}
#[cfg(any(windows, test))]
impl RemovalLeaseSet {
    pub(crate) fn try_exclusive(roots: impl IntoIterator<Item = PathBuf>) -> AppResult<Self> {
        let mut gates = roots
            .into_iter()
            .map(|root| gate(&root))
            .collect::<AppResult<Vec<_>>>()?;
        gates.sort_by(|a, b| a.0.cmp(&b.0));
        gates.dedup_by(|a, b| a.0 == b.0);
        let mut result = Self {
            roots: vec![],
            _files: vec![],
        };
        for (root, file) in gates {
            file.try_lock()
                .map_err(|_| AppError::InstanceLockConflict)?;
            result.roots.push(root);
            result._files.push(file);
        }
        Ok(result)
    }
    pub(crate) fn require(&self, desktop: &Path) -> AppResult<()> {
        let normalized = super::startup_v2::checked_path(desktop)?;
        if self.roots.contains(&normalized) {
            Ok(())
        } else {
            Err(fail("desktop is outside exclusive removal capability"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exclusive_gate_blocks_new_admission_until_inner_handles_are_gone() {
        let t = tempfile::tempdir().unwrap();
        let desktop = t.path().canonicalize().unwrap().join("aggregate/desktop");
        let live = shared(&desktop).unwrap();
        assert!(RemovalLeaseSet::try_exclusive([desktop.clone()]).is_err());
        drop(live);
        let removal = RemovalLeaseSet::try_exclusive([desktop.clone()]).unwrap();
        assert!(shared(&desktop).is_err());
        assert!(crate::modules::instance::AppInstanceLock::acquire(&desktop).is_err());
        assert!(
            crate::modules::backup::manager::acquire_preferences_transaction(&desktop).is_err()
        );
        assert!(
            !desktop.exists(),
            "refused entrants must not recreate removed D"
        );
        removal.require(&desktop).unwrap();
        assert!(removal
            .require(&desktop.with_file_name("unselected"))
            .is_err());
        assert!(!desktop.exists());
        drop(removal);
        shared(&desktop).unwrap();
        assert!(desktop.parent().unwrap().join(NAME).is_file());
    }
    #[cfg(unix)]
    #[test]
    fn linked_or_broad_gate_is_refused_without_repair() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let t = tempfile::tempdir().unwrap();
        let desktop = t.path().canonicalize().unwrap().join("desktop");
        let path = t.path().join(NAME);
        let outside = t.path().join("outside");
        std::fs::write(&outside, b"retain").unwrap();
        symlink(&outside, &path).unwrap();
        assert!(shared(&desktop).is_err());
        assert_eq!(std::fs::read(&outside).unwrap(), b"retain");
        std::fs::remove_file(&path).unwrap();
        std::fs::write(&path, b"retain").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(shared(&desktop).is_err());
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o644
        );
    }
}
