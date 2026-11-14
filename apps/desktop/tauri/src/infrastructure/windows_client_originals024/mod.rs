//! Native prerequisites for same-object custody. A readable DACL alone is not
//! a complete original security descriptor. No mutation or recovery capability
//! is issued when the full descriptor cannot be read by the original token.
use crate::modules::runtime::windows_lifecycle024_contract::{
    FULL_DESCRIPTOR_QUERY_ACCESS, FULL_DESCRIPTOR_SECURITY_INFORMATION,
};
use std::{
    fs::{File, OpenOptions},
    io,
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    path::Path,
};
mod phase;
use windows_sys::Win32::{
    Foundation::LocalFree,
    Security::{
        GetSecurityDescriptorControl, GetSecurityDescriptorLength, IsValidSecurityDescriptor,
        MakeSelfRelativeSD,
    },
    Storage::FileSystem::{GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION},
};

#[derive(Debug)]
pub(crate) struct CaptureError {
    api: &'static str,
    cause: io::Error,
}
impl CaptureError {
    fn new(api: &'static str, cause: io::Error) -> Self {
        Self { api, cause }
    }
    pub(crate) fn api(&self) -> &'static str {
        self.api
    }
    pub(crate) fn raw_os_error(&self) -> Option<i32> {
        self.cause.raw_os_error()
    }
}
impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "api={} raw={:?}: {}",
            self.api,
            self.raw_os_error(),
            self.cause
        )
    }
}
impl std::error::Error for CaptureError {}
impl From<io::Error> for CaptureError {
    fn from(e: io::Error) -> Self {
        Self::new("native-original-path-observation", e)
    }
}
fn denied(reason: &'static str) -> CaptureError {
    CaptureError::new(
        "complete-descriptor-validation",
        io::Error::new(io::ErrorKind::PermissionDenied, reason),
    )
}

// READ_CONTROL cannot authorize SACL observation. Do not enable privileges,
// fall back to owner/group/DACL, or turn an unreadable SACL into an absent SACL.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SecurityFailure {
    Unreadable,
    CoverageUnknown,
}
pub(crate) fn classify_error(code: Option<i32>) -> SecurityFailure {
    match code {
        Some(5 | 1314) => SecurityFailure::Unreadable,
        _ => SecurityFailure::CoverageUnknown,
    }
}

/// Original handle + original native descriptor. Private, non-Clone, non-serde;
/// hashes of this object are audit facts, never a deserializable recovery proof.
pub(crate) struct InspectedOriginal {
    file: File,
    volume: u32,
    id: u64,
    security: Vec<u8>,
}
impl InspectedOriginal {
    pub(crate) fn open(path: &Path, directory: bool) -> Result<Self, CaptureError> {
        let file = OpenOptions::new()
            .access_mode(FULL_DESCRIPTOR_QUERY_ACCESS)
            .share_mode(1)
            .custom_flags(0x0020_0000 | if directory { 0x0200_0000 } else { 0 })
            .open(path)
            .map_err(|e| CaptureError::new("CreateFileW(FULL_DESCRIPTOR_QUERY_ACCESS)", e))?;
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
            return Err(CaptureError::new(
                "GetFileInformationByHandle",
                io::Error::last_os_error(),
            ));
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
    pub(crate) fn recheck(&self) -> Result<(), CaptureError> {
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        if unsafe { GetFileInformationByHandle(self.file.as_raw_handle(), &mut info) } == 0 {
            return Err(CaptureError::new(
                "GetFileInformationByHandle",
                io::Error::last_os_error(),
            ));
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
fn descriptor(file: &File) -> Result<Vec<u8>, CaptureError> {
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn GetSecurityInfo(
            handle: *mut std::ffi::c_void,
            kind: i32,
            requested: u32,
            owner: *mut *mut std::ffi::c_void,
            group: *mut *mut std::ffi::c_void,
            dacl: *mut *mut windows_sys::Win32::Security::ACL,
            sacl: *mut *mut windows_sys::Win32::Security::ACL,
            descriptor: *mut *mut std::ffi::c_void,
        ) -> u32;
    }
    let mut raw = std::ptr::null_mut();
    let code = unsafe {
        GetSecurityInfo(
            file.as_raw_handle(),
            1,
            FULL_DESCRIPTOR_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut raw,
        )
    };
    if code != 0 {
        return Err(CaptureError::new(
            "GetSecurityInfo(SE_FILE_OBJECT,BACKUP_SECURITY_INFORMATION)",
            io::Error::from_raw_os_error(code as i32),
        ));
    }
    if raw.is_null() {
        return Err(denied("complete query returned no descriptor"));
    }
    struct Local(*mut std::ffi::c_void);
    impl Drop for Local {
        fn drop(&mut self) {
            unsafe {
                LocalFree(self.0);
            }
        }
    }
    let _allocated = Local(raw);
    if unsafe { IsValidSecurityDescriptor(raw) } == 0 {
        return Err(denied("invalid complete descriptor"));
    }
    let length = unsafe { GetSecurityDescriptorLength(raw) };
    if !(20..=262144).contains(&length) {
        return Err(denied("complete descriptor length bound"));
    }
    let (mut control, mut revision) = (0u16, 0u32);
    if unsafe { GetSecurityDescriptorControl(raw, &mut control, &mut revision) } == 0 {
        return Err(CaptureError::new(
            "GetSecurityDescriptorControl",
            io::Error::last_os_error(),
        ));
    }
    if revision != 1 {
        return Err(denied("unknown descriptor revision"));
    }
    if control & 0x8000 != 0 {
        // Self-relative output preserves every OS-returned ACE/control byte,
        // including unrecognized extension ACEs; never reorder/filter them.
        return Ok(
            unsafe { std::slice::from_raw_parts(raw.cast::<u8>(), length as usize) }.to_vec(),
        );
    }
    let mut required = 0;
    let ok = unsafe { MakeSelfRelativeSD(raw, std::ptr::null_mut(), &mut required) };
    if ok != 0
        || io::Error::last_os_error().raw_os_error() != Some(122)
        || !(20..=262144).contains(&required)
    {
        return Err(denied("absolute descriptor conversion coverage unknown"));
    }
    let mut relative = vec![0u8; required as usize];
    if unsafe { MakeSelfRelativeSD(raw, relative.as_mut_ptr().cast(), &mut required) } == 0 {
        return Err(CaptureError::new(
            "MakeSelfRelativeSD",
            io::Error::last_os_error(),
        ));
    }
    if relative.len() != required as usize
        || unsafe { IsValidSecurityDescriptor(relative.as_ptr().cast_mut().cast()) } == 0
    {
        return Err(denied("relative descriptor drift"));
    }
    Ok(relative)
}

/// Read-only full-tree admission. Complete SD failure happens before the first
/// client mutation or custody creation or official process construction.
pub(crate) fn preflight(profile: &Path, deadline: std::time::Instant) -> Result<(), CaptureError> {
    let profile_pin = super::windows_official_runtime024::PinnedDirectory::open(profile)?;
    let mut queue = vec![profile.join(".codex"), profile.join(".grok")];
    let mut count = 0;
    while let Some(path) = queue.pop() {
        if std::time::Instant::now() >= deadline {
            return Err(CaptureError::new(
                "original-preflight-deadline",
                io::ErrorKind::TimedOut.into(),
            ));
        }
        count += 1;
        if count > 512 {
            return Err(denied("original tree exceeds bound"));
        }
        match std::fs::symlink_metadata(&path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // Absence still requires full security/identity of its original
                // parent. Do not treat an absent root as permission to mutate.
                InspectedOriginal::open(
                    path.parent().ok_or_else(|| denied("parent absent"))?,
                    true,
                )?
                .recheck()?;
            }
            Err(e) => return Err(CaptureError::new("symlink_metadata(original)", e)),
            Ok(m) => {
                let inspected = InspectedOriginal::open(&path, m.is_dir())?;
                inspected.recheck()?;
                if m.is_dir() {
                    for child in std::fs::read_dir(&path)? {
                        queue.push(child?.path());
                    }
                }
                inspected.recheck()?;
            }
        }
    }
    profile_pin.recheck().map_err(CaptureError::from)
}
pub(crate) fn require_durable_capture() -> crate::errors::AppResult<()> {
    Err(crate::errors::AppError::RuntimeManaged {
        code: "windows024_originals_custody_required".into(),
        detail: "完整安全前提读回后仍需同卷原件持久保管；未创建官方进程或宣告捕获成功".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wl2_r3_complete_query_masks_and_error_classification() {
        assert_eq!(FULL_DESCRIPTOR_QUERY_ACCESS, 0x01020000);
        assert_eq!(FULL_DESCRIPTOR_SECURITY_INFORMATION, 0x10000);
        for code in [5, 1314] {
            assert_eq!(classify_error(Some(code)), SecurityFailure::Unreadable);
        }
        for code in [0, 50, 87, 122, 1338, 1460] {
            assert_eq!(classify_error(Some(code)), SecurityFailure::CoverageUnknown);
        }
        assert_eq!(classify_error(None), SecurityFailure::CoverageUnknown);
        let wrapped = CaptureError::new(
            "CreateFileW(FULL_DESCRIPTOR_QUERY_ACCESS)",
            io::Error::from_raw_os_error(1314),
        );
        assert_eq!(wrapped.api(), "CreateFileW(FULL_DESCRIPTOR_QUERY_ACCESS)");
        assert_eq!(wrapped.raw_os_error(), Some(1314));
        let query = CaptureError::new(
            "GetSecurityInfo(SE_FILE_OBJECT,BACKUP_SECURITY_INFORMATION)",
            io::Error::from_raw_os_error(5),
        );
        assert_eq!(query.raw_os_error(), Some(5));
        assert!(query.to_string().contains("GetSecurityInfo"));
        // The known ordinary-token1314 is reused. No live full-SD request is
        // repeated without a new environment; only new pure classification is tested.
    }
}
