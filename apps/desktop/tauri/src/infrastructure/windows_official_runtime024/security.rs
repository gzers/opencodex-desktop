use std::{
    ffi::c_void,
    io,
    mem::size_of,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    ptr,
};
use windows_sys::Win32::{
    Foundation::{LocalFree, HANDLE},
    Security::{
        GetAce, GetTokenInformation, TokenElevation, TokenElevationType, TokenSessionId, TokenUser,
        ACCESS_ALLOWED_ACE, ACL, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER,
    },
    System::Threading::{GetCurrentProcess, OpenProcessToken},
};

#[link(name = "advapi32")]
unsafe extern "system" {
    fn ConvertSidToStringSidW(sid: *const c_void, text: *mut *mut u16) -> i32;
    fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
        text: *const u16,
        revision: u32,
        sd: *mut *mut c_void,
        length: *mut u32,
    ) -> i32;
    fn GetSecurityInfo(
        handle: HANDLE,
        kind: i32,
        requested: u32,
        owner: *mut *mut c_void,
        group: *mut *mut c_void,
        dacl: *mut *mut ACL,
        sacl: *mut *mut ACL,
        descriptor: *mut *mut c_void,
    ) -> u32;
    fn GetSecurityDescriptorControl(
        descriptor: *const c_void,
        control: *mut u16,
        revision: *mut u32,
    ) -> i32;
    fn IsValidSid(sid: *const c_void) -> i32;
    fn GetLengthSid(sid: *const c_void) -> u32;
}

pub(super) fn denied(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, message)
}
pub(super) fn check(ok: i32) -> io::Result<()> {
    if ok == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
pub(super) struct Local(pub *mut c_void);
impl Drop for Local {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.0);
        }
    }
}

fn sid_text(sid: *const c_void) -> io::Result<String> {
    if sid.is_null() || unsafe { IsValidSid(sid) } == 0 {
        return Err(denied("invalid or missing SID"));
    }
    let mut text = ptr::null_mut();
    check(unsafe { ConvertSidToStringSidW(sid, &mut text) })?;
    let _memory = Local(text.cast());
    let length = (0..256)
        .find(|&n| unsafe { *text.add(n) } == 0)
        .ok_or_else(|| denied("SID string exceeds bound"))?;
    String::from_utf16(unsafe { std::slice::from_raw_parts(text, length) })
        .map_err(|_| denied("invalid SID string"))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenIdentity {
    pub sid: String,
    pub session_id: u32,
    pub elevated: bool,
    pub elevation_type: u32,
}

pub(super) fn process_identity(process: HANDLE) -> io::Result<TokenIdentity> {
    let mut raw = ptr::null_mut();
    check(unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut raw) })?;
    let token = unsafe { OwnedHandle::from_raw_handle(raw) };
    let mut required = 0;
    unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            ptr::null_mut(),
            0,
            &mut required,
        );
    }
    if required < size_of::<TOKEN_USER>() as u32 || required > 65536 {
        return Err(denied("invalid TokenUser size"));
    }
    // Word alignment is required for TOKEN_USER and the embedded SID pointer.
    let mut storage = vec![0usize; (required as usize).div_ceil(size_of::<usize>())];
    check(unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            storage.as_mut_ptr().cast(),
            required,
            &mut required,
        )
    })?;
    let user = unsafe { &*storage.as_ptr().cast::<TOKEN_USER>() };
    let start = storage.as_ptr() as usize;
    let end = start + required as usize;
    let p = user.User.Sid as usize;
    if p < start || p.checked_add(8).is_none_or(|v| v > end) {
        return Err(denied("TokenUser SID outside buffer"));
    }
    let subauthorities = unsafe { *user.User.Sid.cast::<u8>().add(1) } as usize;
    if p.checked_add(8 + 4 * subauthorities)
        .is_none_or(|v| v > end)
    {
        return Err(denied("TokenUser SID truncated"));
    }
    let sid = sid_text(user.User.Sid)?;
    fn integer(token: HANDLE, kind: i32) -> io::Result<u32> {
        let mut value = 0u32;
        let mut length = 0;
        check(unsafe {
            GetTokenInformation(token, kind, (&mut value as *mut u32).cast(), 4, &mut length)
        })?;
        if length != 4 {
            return Err(denied("unexpected token information length"));
        }
        Ok(value)
    }
    let elevated = integer(raw, TokenElevation)?;
    let elevation_type = integer(raw, TokenElevationType)?;
    if elevated > 1 || !(1..=3).contains(&elevation_type) {
        return Err(denied("unknown elevation information"));
    }
    Ok(TokenIdentity {
        sid,
        session_id: integer(raw, TokenSessionId)?,
        elevated: elevated != 0,
        elevation_type,
    })
}
pub fn current_identity() -> io::Result<TokenIdentity> {
    process_identity(unsafe { GetCurrentProcess() })
}

pub(super) struct PrivateDescriptor(Local);
impl PrivateDescriptor {
    pub fn new(sid: &str, directory: bool) -> io::Result<Self> {
        if sid != current_identity()?.sid || matches!(sid, "S-1-5-18" | "S-1-5-32-544") {
            return Err(denied("TokenUser owner required"));
        }
        let inheritance = if directory { "OICI" } else { "" };
        let text: Vec<u16> =
            format!("O:{sid}D:P(A;{inheritance};FA;;;{sid})(A;{inheritance};FA;;;SY)")
                .encode_utf16()
                .chain(Some(0))
                .collect();
        let mut sd = ptr::null_mut();
        check(unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                text.as_ptr(),
                1,
                &mut sd,
                ptr::null_mut(),
            )
        })?;
        if sd.is_null() {
            return Err(denied("missing private descriptor"));
        }
        Ok(Self(Local(sd)))
    }
    pub fn attributes(&self) -> SECURITY_ATTRIBUTES {
        SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: self.0 .0,
            bInheritHandle: 0,
        }
    }
}

/// Read-only validation: never repairs old ownership or permissions. Accept only
/// TokenUser/SYSTEM allow ACEs, with owner full access and no null/empty ACL.
pub(super) fn verify_private(handle: HANDLE, sid: &str, protected: bool) -> io::Result<()> {
    let mut owner = ptr::null_mut();
    let mut acl = ptr::null_mut();
    let mut sd = ptr::null_mut();
    let code = unsafe {
        GetSecurityInfo(
            handle,
            1,
            1 | 4,
            &mut owner,
            ptr::null_mut(),
            &mut acl,
            ptr::null_mut(),
            &mut sd,
        )
    };
    if code != 0 {
        return Err(io::Error::from_raw_os_error(code as i32));
    }
    let _sd = Local(sd);
    if sid_text(owner)? != sid || acl.is_null() {
        return Err(denied("private owner/DACL differs"));
    }
    let mut control = 0;
    let mut revision = 0;
    check(unsafe { GetSecurityDescriptorControl(sd, &mut control, &mut revision) })?;
    if protected && control & 0x1000 == 0 {
        return Err(denied("private root must have protected DACL"));
    }
    let header = unsafe { &*acl };
    let count = header.AceCount as u32;
    if count == 0 || count > 64 {
        return Err(denied("invalid private ACE count"));
    }
    let acl_start = acl as usize;
    let acl_end = acl_start
        .checked_add(header.AclSize as usize)
        .ok_or_else(|| denied("ACL overflow"))?;
    let mut owner_full = false;
    for i in 0..count {
        let mut ace = ptr::null_mut();
        check(unsafe { GetAce(acl, i, &mut ace) })?;
        let start = ace as usize;
        if start < acl_start + size_of::<ACL>() || start + 8 > acl_end {
            return Err(denied("invalid ACE boundary"));
        }
        let allowed = unsafe { &*ace.cast::<ACCESS_ALLOWED_ACE>() };
        let end = start
            .checked_add(allowed.Header.AceSize as usize)
            .ok_or_else(|| denied("ACE overflow"))?;
        if allowed.Header.AceType != 0
            || allowed.Header.AceFlags & 8 != 0
            || end > acl_end
            || end < start + 16
        {
            return Err(denied("unsupported private ACE"));
        }
        let raw_sid = unsafe { ace.cast::<u8>().add(8) }.cast::<c_void>();
        let subauthorities = unsafe { *raw_sid.cast::<u8>().add(1) } as usize;
        if 8 + 4 * subauthorities > end - (start + 8) {
            return Err(denied("SID truncated inside ACE"));
        }
        let who = sid_text(raw_sid)?;
        if unsafe { GetLengthSid(raw_sid) } as usize > end - (start + 8) {
            return Err(denied("SID exceeds ACE"));
        }
        if who != sid && who != "S-1-5-18" {
            return Err(denied("unexpected private DACL principal"));
        }
        if who == sid && allowed.Mask & 0x001f01ff == 0x001f01ff {
            owner_full = true;
        }
    }
    if !owner_full {
        return Err(denied("owner full-access grant required"));
    }
    Ok(())
}
