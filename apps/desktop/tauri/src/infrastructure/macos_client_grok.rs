//! The controlled Mac runner clears GROK_HOME and fixes HOME. Official2.99
//! resolveGrokHome therefore uses that profile's .grok. No directory is created
//! for observation; missing is independently retained, unreadable is not empty.
use super::*;
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum GrokSnapshot {
    Absent,
    Directory { tree: TreeSnapshot },
}
impl std::fmt::Debug for GrokSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Absent => f.write_str("GrokAbsent"),
            Self::Directory { tree } => f.debug_tuple("GrokTree").field(tree).finish(),
        }
    }
}

#[cfg(test)]
#[path = "macos_client_grok_tests.rs"]
mod tests;
fn lookup(profile: &PinnedDirectory) -> io::Result<Option<libc::stat>> {
    let mut out = std::mem::MaybeUninit::uninit();
    if unsafe {
        libc::fstatat(
            profile.file().as_raw_fd(),
            c".grok".as_ptr(),
            out.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    } != 0
    {
        let e = io::Error::last_os_error();
        return if e.kind() == io::ErrorKind::NotFound {
            Ok(None)
        } else {
            Err(e)
        };
    }
    Ok(Some(unsafe { out.assume_init() }))
}
impl GrokSnapshot {
    pub(super) fn backup_remainder_matches(&self, current: &Self) -> bool {
        match (self, current) {
            (Self::Directory { tree: a }, Self::Directory { tree: b }) => {
                a.grok_backup_remainder_matches(b)
            }
            _ => false,
        }
    }

    pub(super) fn restored_matches(&self, current: &Self) -> bool {
        match (self, current) {
            (Self::Absent, Self::Absent) => true,
            (Self::Directory { tree: a }, Self::Directory { tree: b }) => {
                a.restored_matches(b, false)
            }
            _ => false,
        }
    }
    pub fn read(profile: &PinnedDirectory) -> io::Result<Self> {
        profile.recheck()?;
        match lookup(profile)? {
            None => {
                profile.recheck()?;
                if lookup(profile)?.is_some() {
                    return Err(denied("Grok absence changed"));
                }
                Ok(Self::Absent)
            }
            Some(before) => {
                if before.st_mode & libc::S_IFMT != libc::S_IFDIR
                    || before.st_uid != unsafe { libc::geteuid() }
                {
                    return Err(denied("Grok root kind or user unknown"));
                }
                let root = PinnedDirectory::open(&profile.path().join(".grok"))?;
                if root.identity() != &identity(&before) {
                    return Err(denied("Grok directory replaced before capture"));
                }
                let tree = TreeSnapshot::read_mode(&root, false)?;
                root.recheck()?;
                profile.recheck()?;
                if lookup(profile)?.is_none_or(|s| version(&s) != version(&before)) {
                    return Err(denied("Grok directory changed during capture"));
                }
                Ok(Self::Directory { tree })
            }
        }
    }
    pub(super) fn archive(
        &self,
        profile: &PinnedDirectory,
        vault: &PinnedDirectory,
    ) -> io::Result<()> {
        if Self::read(profile)? != *self {
            return Err(denied("Grok source changed before archival"));
        }
        if let Self::Directory { tree } = self {
            let root = PinnedDirectory::open(&profile.path().join(".grok"))?;
            tree.archive(&root, vault)?;
        }
        if Self::read(profile)? != *self {
            return Err(denied("Grok source changed during archival"));
        }
        Ok(())
    }
    pub(super) fn verify_archive(&self, vault: &PinnedDirectory) -> io::Result<()> {
        match self {
            Self::Absent => Ok(()),
            Self::Directory { tree } => {
                if tree.coordinated() {
                    return Err(denied(
                        "Grok tree cannot claim Codex coordination semantics",
                    ));
                }
                tree.verify_archive(vault)
            }
        }
    }
    pub(super) fn archived_file(
        &self,
        vault: &PinnedDirectory,
        path: &str,
    ) -> io::Result<zeroize::Zeroizing<Vec<u8>>> {
        match self {
            Self::Absent => Err(denied("Grok root was absent in original phase")),
            Self::Directory { tree } => tree.archived_file(vault, path),
        }
    }
}
