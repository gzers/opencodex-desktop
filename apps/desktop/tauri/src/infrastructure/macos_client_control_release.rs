//! Descriptor-relative, create-only publication of the Desktop control nonce.
//! The final name is never visible with incomplete contents.
use super::*;
impl PinnedDirectory {
    pub(crate) fn publish_complete_release(
        &self,
        operation: &str,
        checkpoint: &impl Fn() -> io::Result<()>,
    ) -> io::Result<()> {
        use std::os::unix::fs::MetadataExt;
        self.private()?;
        if uuid::Uuid::parse_str(operation)
            .map(|v| v.to_string() != operation)
            .unwrap_or(true)
        {
            return Err(denied("release notification unsafe or changed"));
        }
        let name = std::ffi::CString::new(format!(".release-{operation}.tmp"))?;
        let fd = unsafe {
            libc::openat(
                self.file().as_raw_fd(),
                name.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                0o600,
            )
        };
        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }
        let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
        let before = file.metadata()?;
        if !before.is_file()
            || before.uid() != unsafe { libc::geteuid() }
            || before.nlink() != 1
            || before.mode() & 0o777 != 0o600
        {
            return Err(denied("release notification unsafe or changed"));
        }
        // The consumer's final path remains ABSENT until every byte is complete
        // and durable. A visible empty/partial nonce is an immediate helper refusal.
        checkpoint()?;
        writeln!(file, "{operation}")?;
        file.sync_all()?;
        self.private()?;
        if unsafe {
            libc::renameatx_np(
                self.file().as_raw_fd(),
                name.as_ptr(),
                self.file().as_raw_fd(),
                c"release".as_ptr(),
                libc::RENAME_EXCL,
            )
        } != 0
        {
            return Err(std::io::Error::last_os_error());
        }
        self.file().sync_all()?;
        self.private()?;
        let mut current = std::mem::MaybeUninit::<libc::stat>::uninit();
        if unsafe {
            libc::fstatat(
                self.file().as_raw_fd(),
                c"release".as_ptr(),
                current.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } != 0
        {
            return Err(std::io::Error::last_os_error());
        }
        let current = unsafe { current.assume_init() };
        if current.st_dev as u64 != before.dev()
            || current.st_ino != before.ino()
            || current.st_nlink != 1
            || current.st_mode & 0o777 != 0o600
            || current.st_uid != before.uid()
            || current.st_size != operation.len() as i64 + 1
        {
            return Err(denied("release notification unsafe or changed"));
        }
        Ok(())
    }
}
