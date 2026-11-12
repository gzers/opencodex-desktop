//! Create-only content objects in the original private attempt directory.
//! Names are derived only from independently bound tree SHA fields. Partial or
//! corrupt objects stay retained and refuse reuse; source bytes never overwrite
//! a prior object, and none of these methods publish into the client tree.
use super::*;

fn blob_name(sha: &str) -> io::Result<String> {
    if sha.len() != 64
        || !sha
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(denied("invalid tree content reference"));
    }
    Ok(format!("tree-content-{sha}.blob"))
}
fn hash_file(
    file: &File,
    expected: u64,
    mut write: impl FnMut(&[u8]) -> io::Result<()>,
) -> io::Result<String> {
    if expected > MAX_FILE {
        return Err(denied("tree content size over limit"));
    }
    let mut buffer = zeroize::Zeroizing::new(vec![0; 64 * 1024]);
    let mut total = 0u64;
    let mut hash = Sha256::new();
    let mut input = file;
    loop {
        let n = input.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        total += n as u64;
        if total > expected {
            return Err(denied("tree content grew while copying"));
        }
        hash.update(&buffer[..n]);
        write(&buffer[..n])?;
    }
    if total != expected {
        return Err(denied("tree content size differs"));
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn verify_blob(vault: &PinnedDirectory, sha: &str, bytes: u64) -> io::Result<()> {
    vault.private()?;
    let child = blob_name(sha)?;
    let file = open_at(vault.file(), OsStr::new(&child), false)?;
    let before = stat(&file)?;
    let p = protection(&file, &before)?;
    require_private(&p, false)?;
    if before.st_size < 0
        || before.st_size as u64 != bytes
        || hash_file(&file, bytes, |_| Ok(()))? != sha
    {
        return Err(denied("tree archived content differs"));
    }
    stable(vault.file(), OsStr::new(&child), &file, &before, &p)?;
    vault.private()?;
    Ok(())
}
impl TreeSnapshot {
    fn source_file(&self, root: &PinnedDirectory, path: &str) -> io::Result<(Vec<File>, File)> {
        let parts: Vec<_> = path.split('/').collect();
        let mut chain = Vec::<File>::new();
        let mut relative = String::new();
        for part in &parts[..parts.len() - 1] {
            let parent = chain.last().unwrap_or(root.file());
            let file = open_at(parent, OsStr::new(part), true)?;
            if !relative.is_empty() {
                relative.push('/');
            }
            relative.push_str(part);
            let Some(Entry::Directory {
                identity: id,
                protection: p,
                ..
            }) = self.entries.iter().find(|e| e.path() == relative)
            else {
                return Err(denied("tree ancestor not independently referenced"));
            };
            let s = stat(&file)?;
            if identity(&s) != *id || protection(&file, &s)? != *p {
                return Err(denied("tree ancestor changed before archive"));
            }
            chain.push(file);
        }
        let parent = chain.last().unwrap_or(root.file());
        let child = OsStr::new(parts.last().unwrap());
        let observed = lookup(parent, child)?;
        if observed.st_mode & libc::S_IFMT != libc::S_IFREG || observed.st_nlink != 1 {
            return Err(denied("tree archive source is not single ordinary file"));
        }
        let file = open_at(parent, child, false)?;
        if version(&stat(&file)?) != version(&observed) {
            return Err(denied("tree archive source replaced before open"));
        }
        Ok((chain, file))
    }
    pub(crate) fn archive(
        &self,
        root: &PinnedDirectory,
        vault: &PinnedDirectory,
    ) -> io::Result<()> {
        self.archive_checked(root, vault, &|_| Ok(()))
    }
    fn archive_checked(
        &self,
        root: &PinnedDirectory,
        vault: &PinnedDirectory,
        checkpoint: &impl Fn(&str) -> io::Result<()>,
    ) -> io::Result<()> {
        if !self.valid() || TreeSnapshot::read_mode(root, self.coordination)? != *self {
            return Err(denied("tree changed before byte archive"));
        }
        if vault.path().starts_with(root.path()) || root.path().starts_with(vault.path()) {
            return Err(denied("tree archive roots overlap"));
        }
        vault.private()?;
        for entry in &self.entries {
            let Entry::File {
                path,
                identity: id,
                protection: p,
                bytes,
                sha256,
            } = entry
            else {
                continue;
            };
            let (chain, file) = self.source_file(root, path)?;
            let before = stat(&file)?;
            if identity(&before) != *id
                || protection(&file, &before)? != *p
                || before.st_size < 0
                || before.st_size as u64 != *bytes
            {
                return Err(denied("tree archive source changed"));
            }
            let child = blob_name(sha256)?;
            let c = name(OsStr::new(&child))?;
            let raw = unsafe {
                libc::openat(
                    vault.file().as_raw_fd(),
                    c.as_ptr(),
                    libc::O_RDWR
                        | libc::O_CREAT
                        | libc::O_EXCL
                        | libc::O_NOFOLLOW
                        | libc::O_CLOEXEC,
                    0o400,
                )
            };
            if raw >= 0 {
                let mut blob = unsafe { File::from_raw_fd(raw) };
                require_private(&protection(&blob, &stat(&blob)?)?, false)?;
                let hash = hash_file(&file, *bytes, |data| blob.write_all(data))?;
                blob.sync_all()?;
                vault.sync()?;
                checkpoint("after_blob")?;
                if hash != *sha256 {
                    return Err(denied("tree source hash changed during archive"));
                }
            } else {
                let e = io::Error::last_os_error();
                if e.kind() != io::ErrorKind::AlreadyExists {
                    return Err(e);
                }
                // A retained object may come from a failed earlier capture.
                // Its original successful fsync is not assumed from existence.
                // Re-read and sync this exact protected fd before a new frame
                // may reference it; never replace or narrow its permissions.
                verify_blob(vault, sha256, *bytes)?;
                let retained = open_at(vault.file(), OsStr::new(&child), false)?;
                let metadata = stat(&retained)?;
                let security = protection(&retained, &metadata)?;
                require_private(&security, false)?;
                if metadata.st_size < 0
                    || metadata.st_size as u64 != *bytes
                    || hash_file(&retained, *bytes, |_| Ok(()))? != *sha256
                {
                    return Err(denied("retained tree content changed before sync"));
                }
                retained.sync_all()?;
                vault.sync()?;
                stable(
                    vault.file(),
                    OsStr::new(&child),
                    &retained,
                    &metadata,
                    &security,
                )?;
            }
            let name = OsStr::new(path.rsplit('/').next().unwrap());
            let parent = chain.last().unwrap_or(root.file());
            stable(parent, name, &file, &before, p)?;
            verify_blob(vault, sha256, *bytes)?;
        }
        if TreeSnapshot::read_mode(root, self.coordination)? != *self {
            return Err(denied("tree changed while archiving bytes"));
        }
        self.verify_archive(vault)
    }
    pub(crate) fn verify_archive(&self, vault: &PinnedDirectory) -> io::Result<()> {
        if !self.valid() {
            return Err(denied("invalid archived tree inventory"));
        }
        let mut checked = std::collections::BTreeMap::new();
        for entry in &self.entries {
            if let Entry::File { sha256, bytes, .. } = entry {
                if let Some(previous) = checked.insert(sha256, *bytes) {
                    if previous != *bytes {
                        return Err(denied("tree hash has contradictory sizes"));
                    }
                } else {
                    verify_blob(vault, sha256, *bytes)?;
                }
            }
        }
        Ok(())
    }
    pub(crate) fn archived_file(
        &self,
        vault: &PinnedDirectory,
        path: &str,
    ) -> io::Result<zeroize::Zeroizing<Vec<u8>>> {
        let Some(Entry::File { bytes, sha256, .. }) =
            self.entries.iter().find(|entry| entry.path() == path)
        else {
            return Err(denied("tree file not independently archived"));
        };
        verify_blob(vault, sha256, *bytes)?;
        let file = open_at(vault.file(), OsStr::new(&blob_name(sha256)?), false)?;
        let before = stat(&file)?;
        let p = protection(&file, &before)?;
        require_private(&p, false)?;
        let mut out = zeroize::Zeroizing::new(Vec::with_capacity(*bytes as usize));
        (&file).take(*bytes + 1).read_to_end(&mut out)?;
        if out.len() as u64 != *bytes || digest(&out) != *sha256 {
            return Err(denied("tree archived readback changed"));
        }
        stable(
            vault.file(),
            OsStr::new(&blob_name(sha256)?),
            &file,
            &before,
            &p,
        )?;
        Ok(out)
    }
}

#[cfg(test)]
#[path = "macos_client_tree_archive_tests.rs"]
mod tests;
