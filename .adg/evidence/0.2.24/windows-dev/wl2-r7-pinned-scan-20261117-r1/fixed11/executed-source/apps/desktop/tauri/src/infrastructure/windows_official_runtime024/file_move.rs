//! Single-file original-handle rename mechanism, not a ClientOriginals proof.
//! No SD coverage/custody journal/Closed authority is asserted here. A production
//! adapter must first prove full security and flush its exact move intent.
use super::{
    paths::{identity, validate, wide},
    security::{check, denied},
    FileIdentity, PinnedDirectory,
};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom},
    mem::{offset_of, size_of},
    os::windows::{ffi::OsStrExt, fs::OpenOptionsExt, io::AsRawHandle},
    path::{Path, PathBuf},
    time::Instant,
};
use windows_sys::Win32::{
    Foundation::INVALID_HANDLE_VALUE,
    Storage::FileSystem::{
        FileRenameInfo, FindClose, FindFirstStreamW, FindNextStreamW, FindStreamInfoStandard,
        SetFileInformationByHandle, FILE_RENAME_INFO, WIN32_FIND_STREAM_DATA,
    },
};
const LIMIT: usize = 2 * 1024 * 1024;
fn before(deadline: Instant) -> io::Result<()> {
    if Instant::now() >= deadline {
        Err(io::ErrorKind::TimedOut.into())
    } else {
        Ok(())
    }
}
fn unnamed_stream_only(path: &Path) -> io::Result<()> {
    let w = wide(path)?;
    let mut data = WIN32_FIND_STREAM_DATA::default();
    let h = unsafe {
        FindFirstStreamW(
            w.as_ptr(),
            FindStreamInfoStandard,
            (&mut data as *mut WIN32_FIND_STREAM_DATA).cast(),
            0,
        )
    };
    if h == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    struct Search(windows_sys::Win32::Foundation::HANDLE);
    impl Drop for Search {
        fn drop(&mut self) {
            unsafe {
                FindClose(self.0);
            }
        }
    }
    let _search = Search(h);
    let n = data
        .cStreamName
        .iter()
        .position(|c| *c == 0)
        .ok_or_else(|| denied("unbounded stream name"))?;
    if String::from_utf16(&data.cStreamName[..n]).map_err(|_| denied("stream unicode"))?
        != "::$DATA"
    {
        return Err(denied("named or unknown stream refuses original movement"));
    }
    if unsafe { FindNextStreamW(h, (&mut data as *mut WIN32_FIND_STREAM_DATA).cast()) } != 0 {
        return Err(denied("additional streams refuse original movement"));
    }
    let e = io::Error::last_os_error();
    if e.raw_os_error() != Some(38) {
        return Err(e);
    }
    Ok(())
}
fn bytes(file: &mut File) -> io::Result<Vec<u8>> {
    if file.metadata()?.len() > LIMIT as u64 {
        return Err(denied("original file content bound"));
    }
    file.seek(SeekFrom::Start(0))?;
    let mut b = Vec::new();
    file.take(LIMIT as u64 + 1).read_to_end(&mut b)?;
    if b.len() > LIMIT {
        return Err(denied("original file content bound changed"));
    }
    Ok(b)
}
/// Private original native handle and bounded bytes, not serializable/Clone.
/// The original path may be absent after rename; restore always uses this same
/// object with ReplaceIfExists=false. Errors preserve its last known location.
pub struct RetainedFileMove {
    file: File,
    original_parent: PinnedDirectory,
    original_path: PathBuf,
    current_path: PathBuf,
    target_parent: Option<PinnedDirectory>,
    original_id: FileIdentity,
    original_bytes: Vec<u8>,
}
impl RetainedFileMove {
    pub fn open(path: &Path, deadline: Instant) -> io::Result<Self> {
        before(deadline)?;
        validate(path)?;
        let parent = PinnedDirectory::open(
            path.parent()
                .ok_or_else(|| denied("original parent absent"))?,
        )?;
        let mut file = OpenOptions::new()
            .access_mode(0x80030000)
            .share_mode(1)
            .custom_flags(0x00200000)
            .open(path)?;
        let original_id = identity(&file, false)?;
        unnamed_stream_only(path)?;
        let original_bytes = bytes(&mut file)?;
        if identity(&file, false)? != original_id || bytes(&mut file)? != original_bytes {
            return Err(denied("original changed before move handle sealed"));
        }
        parent.recheck()?;
        before(deadline)?;
        Ok(Self {
            file,
            original_parent: parent,
            original_path: path.into(),
            current_path: path.into(),
            target_parent: None,
            original_id,
            original_bytes,
        })
    }
    pub fn identity(&self) -> FileIdentity {
        self.original_id
    }
    pub fn location(&self) -> &Path {
        &self.current_path
    }
    pub fn verify(&mut self, deadline: Instant) -> io::Result<()> {
        before(deadline)?;
        self.original_parent.recheck()?;
        if let Some(parent) = &self.target_parent {
            parent.recheck()?;
        }
        if identity(&self.file, false)? != self.original_id
            || bytes(&mut self.file)? != self.original_bytes
        {
            return Err(denied("original handle identity/content drift"));
        }
        // Reopen only for data identity with DELETE sharing compatible with the
        // already-retained original DELETE-capable descriptor. No new authority.
        let observed = OpenOptions::new()
            .access_mode(0x80020000)
            .share_mode(1 | 4)
            .custom_flags(0x00200000)
            .open(&self.current_path)?;
        if identity(&observed, false)? != self.original_id {
            return Err(denied("original path does not name retained object"));
        }
        unnamed_stream_only(&self.current_path)?;
        before(deadline)
    }
    pub fn move_into(&mut self, target: &Path, name: &str, deadline: Instant) -> io::Result<()> {
        self.verify(deadline)?;
        if self.target_parent.is_some() {
            return Err(denied("original already moved; no successor target"));
        }
        if name.is_empty() || name.contains(['/', '\\']) || name == "." || name == ".." {
            return Err(denied("one exact target leaf required"));
        }
        let destination = target.join(name);
        validate(&destination)?;
        let parent = PinnedDirectory::open_private(target)?;
        if parent.identity() == self.original_parent.identity() {
            return Err(denied(
                "custody target is the original parent object; same-path no-op refused",
            ));
        }
        if parent.identity().volume != self.original_id.volume {
            return Err(denied(
                "cross-volume original movement refused; no copy/delete fallback",
            ));
        }
        before(deadline)?;
        rename_handle(&self.file, &destination)?;
        // Update location immediately after native success, before any fallible
        // post-check. A later error must not drop knowledge of the moved object.
        self.current_path = destination;
        self.target_parent = Some(parent);
        self.verify(deadline)
    }
    pub fn restore(&mut self, deadline: Instant) -> io::Result<()> {
        self.verify(deadline)?;
        if self.target_parent.is_none() {
            return Err(denied("no moved original to restore"));
        }
        if self.current_path == self.original_path {
            // A previous native restore may have completed just as its caller's
            // deadline expired. Verify this same object; do not rename again.
            self.target_parent = None;
            return Ok(());
        }
        let path = self.original_path.clone();
        rename_handle(&self.file, &path)?;
        self.current_path = path;
        self.verify(deadline)?;
        self.target_parent = None;
        Ok(())
    }
}
pub(crate) fn rename_handle(file: &File, path: &Path) -> io::Result<()> {
    validate(path)?;
    let name: Vec<u16> = path.as_os_str().encode_wide().collect();
    if name.is_empty() || name.contains(&0) || name.len() > 32760 {
        return Err(denied("rename path bound"));
    }
    let offset = offset_of!(FILE_RENAME_INFO, FileName);
    let name_bytes = name
        .len()
        .checked_mul(2)
        .ok_or_else(|| denied("rename size overflow"))?;
    let length = offset
        .checked_add(name_bytes)
        // Keep FileNameLength at name bytes, and supply explicit zeroed WCHAR
        // storage after FileName. Never depend on usize allocation padding for
        // termination: some valid lengths fill that allocation exactly.
        .and_then(|length| length.checked_add(2))
        .ok_or_else(|| denied("rename size overflow"))?;
    let mut storage = vec![
        0usize;
        length
            .max(size_of::<FILE_RENAME_INFO>())
            .div_ceil(size_of::<usize>())
    ];
    let info = storage.as_mut_ptr().cast::<FILE_RENAME_INFO>();
    unsafe {
        (*info).Anonymous.ReplaceIfExists = false;
        (*info).RootDirectory = std::ptr::null_mut();
        (*info).FileNameLength = name_bytes as u32;
        std::ptr::copy_nonoverlapping(
            name.as_ptr(),
            storage.as_mut_ptr().cast::<u8>().add(offset).cast(),
            name.len(),
        );
        check(SetFileInformationByHandle(
            file.as_raw_handle(),
            FileRenameInfo,
            info.cast(),
            length as u32,
        ))
    }
}
