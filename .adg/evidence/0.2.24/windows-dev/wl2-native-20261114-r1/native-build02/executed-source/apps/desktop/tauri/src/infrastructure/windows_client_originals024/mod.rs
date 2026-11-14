//! Native prerequisites for same-object custody. A readable DACL alone is not
//! a complete original security descriptor. No mutation or recovery capability
//! is issued when the full descriptor cannot be read by the original token.
use std::{
    fs::{File, OpenOptions},
    io,
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    path::Path,
};
use windows_sys::Win32::{
    Security::GetKernelObjectSecurity,
    Storage::FileSystem::{GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION},
};

fn denied(reason: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, reason)
}

// READ_CONTROL cannot authorize SACL observation. Do not enable privileges,
// fall back to owner/group/DACL, or turn an unreadable SACL into an absent SACL.
const COMPLETE_SECURITY: u32 = 0x0000_01ff;
const READ_AND_SECURITY: u32 = 0x8002_0000 | 0x0100_0000;

/// Original handle + original native descriptor. Private, non-Clone, non-serde;
/// hashes of this object are audit facts, never a deserializable recovery proof.
pub(crate) struct InspectedOriginal {
    file: File,
    volume: u32,
    id: u64,
    security: Vec<u8>,
}
impl InspectedOriginal {
    pub(crate) fn open(path: &Path, directory: bool) -> io::Result<Self> {
        let file = OpenOptions::new()
            .access_mode(READ_AND_SECURITY)
            .share_mode(1)
            .custom_flags(0x0020_0000 | if directory { 0x0200_0000 } else { 0 })
            .open(path)?;
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
            return Err(io::Error::last_os_error());
        }
        if info.dwFileAttributes & 0x400 != 0
            || (info.dwFileAttributes & 0x10 != 0) != directory
            || !directory && info.nNumberOfLinks != 1
        {
            return Err(denied("original has reparse/multiple links/wrong kind"));
        }
        let security = descriptor(&file)?;
        Ok(Self {
            file,
            volume: info.dwVolumeSerialNumber,
            id: ((info.nFileIndexHigh as u64) << 32) | info.nFileIndexLow as u64,
            security,
        })
    }
    pub(crate) fn recheck(&self) -> io::Result<()> {
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        if unsafe { GetFileInformationByHandle(self.file.as_raw_handle(), &mut info) } == 0 {
            return Err(io::Error::last_os_error());
        }
        if info.dwVolumeSerialNumber != self.volume
            || ((info.nFileIndexHigh as u64) << 32) | info.nFileIndexLow as u64 != self.id
            || descriptor(&self.file)? != self.security
        {
            return Err(denied("original identity or complete security changed"));
        }
        Ok(())
    }
}
fn descriptor(file: &File) -> io::Result<Vec<u8>> {
    let mut needed = 0;
    let first = unsafe {
        GetKernelObjectSecurity(
            file.as_raw_handle(),
            COMPLETE_SECURITY,
            std::ptr::null_mut(),
            0,
            &mut needed,
        )
    };
    let error = io::Error::last_os_error();
    if first != 0 || error.raw_os_error() != Some(122) {
        return Err(error);
    }
    if !(20..=262144).contains(&needed) {
        return Err(denied("complete descriptor exceeds original bound"));
    }
    let mut bytes = vec![0u8; needed as usize];
    if unsafe {
        GetKernelObjectSecurity(
            file.as_raw_handle(),
            COMPLETE_SECURITY,
            bytes.as_mut_ptr().cast(),
            needed,
            &mut needed,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if needed as usize != bytes.len() {
        return Err(denied("complete descriptor changed while reading"));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wl2_complete_descriptor_never_falls_back_or_mutates() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("synthetic-original");
        std::fs::write(&p, b"synthetic, no credentials").unwrap();
        let before = std::fs::read(&p).unwrap();
        match InspectedOriginal::open(&p, false) {
            Ok(original) => original.recheck().unwrap(),
            Err(e) => {
                // This is prerequisite refusal, not a successful custody test.
                println!(
                    "complete-security prerequisite rejected: raw={:?} error={e}",
                    e.raw_os_error()
                );
                assert!(matches!(e.raw_os_error(), Some(5 | 1314)));
            }
        }
        assert_eq!(std::fs::read(&p).unwrap(), before);
    }
}
