//! Token-bound Windows identity and private security descriptors.
//! Raw bindings below cover SDK APIs absent from the existing windows-sys features;
//! signatures use the same Win32 types and do not require widening Cargo features.
use std::os::windows::{
    ffi::OsStringExt,
    fs::{MetadataExt, OpenOptionsExt},
    io::AsRawHandle,
};
use std::{
    ffi::c_void,
    fs::OpenOptions,
    io,
    path::{Component, Path, PathBuf},
    ptr,
};
#[cfg(test)]
use windows_sys::Win32::UI::Shell::{FOLDERID_LocalAppData, SHGetKnownFolderPath};
use windows_sys::Win32::{
    Foundation::{CloseHandle, LocalFree, HANDLE},
    Security::{
        GetSecurityDescriptorDacl, GetTokenInformation, TokenUser, ACL, SECURITY_ATTRIBUTES,
        TOKEN_QUERY, TOKEN_USER,
    },
    System::Threading::{GetCurrentProcess, GetCurrentThread, OpenProcessToken, OpenThreadToken},
    UI::Shell::GetUserProfileDirectoryW,
};

#[link(name = "advapi32")]
extern "system" {
    fn ConvertSidToStringSidW(sid: *const c_void, text: *mut *mut u16) -> i32;
    fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
        text: *const u16,
        revision: u32,
        descriptor: *mut *mut c_void,
        size: *mut u32,
    ) -> i32;
    fn GetSecurityInfo(
        handle: HANDLE,
        kind: i32,
        info: u32,
        owner: *mut *mut c_void,
        group: *mut *mut c_void,
        dacl: *mut *mut ACL,
        sacl: *mut *mut ACL,
        descriptor: *mut *mut c_void,
    ) -> u32;
    fn SetSecurityInfo(
        handle: HANDLE,
        kind: i32,
        info: u32,
        owner: *mut c_void,
        group: *mut c_void,
        dacl: *const ACL,
        sacl: *const ACL,
    ) -> u32;
    fn ImpersonateNamedPipeClient(pipe: HANDLE) -> i32;
    fn RevertToSelf() -> i32;
}
#[link(name = "ole32")]
extern "system" {
    #[cfg(test)]
    fn CoTaskMemFree(value: *const c_void);
}
#[link(name = "kernel32")]
extern "system" {
    fn GetNamedPipeServerProcessId(pipe: HANDLE, process: *mut u32) -> i32;
}

/// CLI callers must authenticate the connected server before sending a request
/// (which may contain secrets). A process handle retains the queried identity;
/// a dead/inaccessible server is an error, never a reason to trust the name.
pub(crate) fn verify_pipe_server(pipe: HANDLE) -> io::Result<()> {
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    let mut pid = 0;
    if unsafe { GetNamedPipeServerProcessId(pipe, &mut pid) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if process.is_null() {
        return Err(io::Error::last_os_error());
    }
    let process = Token(process);
    let mut token = ptr::null_mut();
    if unsafe { OpenProcessToken(process.0, TOKEN_QUERY, &mut token) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let token = Token(token);
    let mut still_pid = 0;
    if unsafe { GetNamedPipeServerProcessId(pipe, &mut still_pid) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if pid != still_pid || token_sid(token.0)? != current_user_sid()? {
        return Err(denied("IPC server SID differs from current user SID"));
    }
    Ok(())
}

struct Token(HANDLE);
impl Drop for Token {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
fn process_token() -> io::Result<Token> {
    let mut token = ptr::null_mut();
    // SAFETY: query-only process token; Token owns and closes the returned handle.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(Token(token))
}
fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn denied(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, message)
}
unsafe fn sid_text(sid: *const c_void) -> io::Result<String> {
    let mut text = ptr::null_mut();
    // SAFETY: callers retain the security descriptor/token containing a valid SID.
    if unsafe { ConvertSidToStringSidW(sid, &mut text) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let memory = SecurityDescriptor(text.cast());
    let mut len = 0;
    while len < 256 && unsafe { *text.add(len) } != 0 {
        len += 1;
    }
    if len == 256 {
        return Err(invalid("SID string exceeds bound"));
    }
    let result = String::from_utf16(unsafe { std::slice::from_raw_parts(text, len) })
        .map_err(|_| invalid("invalid SID string"));
    drop(memory);
    result
}
fn token_sid(token: HANDLE) -> io::Result<String> {
    let mut needed = 0;
    // SAFETY: size query; aligned storage below retains TOKEN_USER and its SID.
    unsafe {
        GetTokenInformation(token, TokenUser, ptr::null_mut(), 0, &mut needed);
    }
    if !(std::mem::size_of::<TOKEN_USER>() as u32..=65536).contains(&needed) {
        return Err(invalid("invalid token user size"));
    }
    let mut user = vec![0usize; (needed as usize).div_ceil(std::mem::size_of::<usize>())];
    if unsafe {
        GetTokenInformation(
            token,
            TokenUser,
            user.as_mut_ptr().cast(),
            needed,
            &mut needed,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    unsafe { sid_text((*user.as_ptr().cast::<TOKEN_USER>()).User.Sid) }
}
pub(crate) fn current_user_sid() -> io::Result<String> {
    token_sid(process_token()?.0)
}

/// Uses the real process token, independent of HOME/USERPROFILE/USERNAME/cwd.
pub(crate) fn current_user_profile() -> io::Result<PathBuf> {
    let token = process_token()?;
    let mut needed = 0;
    unsafe {
        GetUserProfileDirectoryW(token.0, ptr::null_mut(), &mut needed);
    }
    if !(2..=32768).contains(&needed) {
        return Err(invalid("invalid OS Profile size"));
    }
    let mut path = vec![0u16; needed as usize];
    if unsafe { GetUserProfileDirectoryW(token.0, path.as_mut_ptr(), &mut needed) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if needed as usize > path.len() || needed < 2 || path[needed as usize - 1] != 0 {
        return Err(invalid("invalid OS Profile framing"));
    }
    checked_folder(&path[..needed as usize - 1])
}
#[cfg(test)]
pub(crate) fn current_user_local_app_data() -> io::Result<PathBuf> {
    let mut path = ptr::null_mut();
    let token = process_token()?;
    // KF_FLAG_DONT_VERIFY (0x4000) retrieves identity without creating a directory.
    let status =
        unsafe { SHGetKnownFolderPath(&FOLDERID_LocalAppData, 0x4000, token.0, &mut path) };
    if status < 0 {
        return Err(io::Error::other(format!(
            "LocalAppData lookup failed: {status:#x}"
        )));
    }
    let result = (|| {
        if path.is_null() {
            return Err(invalid("missing OS LocalAppData"));
        }
        let mut len = 0;
        while len < 32768 && unsafe { *path.add(len) } != 0 {
            len += 1;
        }
        if len == 32768 {
            return Err(invalid("OS LocalAppData exceeds bound"));
        }
        checked_folder(unsafe { std::slice::from_raw_parts(path, len) })
    })();
    unsafe {
        CoTaskMemFree(path.cast());
    }
    result
}
fn checked_folder(units: &[u16]) -> io::Result<PathBuf> {
    if units.is_empty() || units.contains(&0) {
        return Err(invalid("invalid OS folder"));
    }
    let path = PathBuf::from(std::ffi::OsString::from_wide(units));
    if !path.is_absolute() {
        return Err(invalid("OS folder must be absolute"));
    }
    Ok(path)
}

/// LocalAlloc ownership, also used for SID strings returned by Windows.
pub(crate) struct SecurityDescriptor(*mut c_void);
impl Drop for SecurityDescriptor {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.0);
        }
    }
}
impl SecurityDescriptor {
    fn private(sid: &str, directory: bool) -> io::Result<Self> {
        let flags = if directory { "OICI" } else { "" };
        let sddl: Vec<u16> = format!("D:P(A;{flags};FA;;;SY)(A;{flags};FA;;;{sid})")
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let mut descriptor = ptr::null_mut();
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                1,
                &mut descriptor,
                ptr::null_mut(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(Self(descriptor))
    }
    pub(crate) fn attributes(&self) -> SECURITY_ATTRIBUTES {
        SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: self.0,
            bInheritHandle: 0,
        }
    }
    fn dacl(&self) -> io::Result<*mut ACL> {
        let mut present = 0;
        let mut defaulted = 0;
        let mut acl = ptr::null_mut();
        if unsafe { GetSecurityDescriptorDacl(self.0, &mut present, &mut acl, &mut defaulted) } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if present == 0 || acl.is_null() {
            return Err(invalid("private DACL must not be null"));
        }
        Ok(acl)
    }
}
pub(crate) fn private_pipe_security() -> io::Result<SecurityDescriptor> {
    SecurityDescriptor::private(&current_user_sid()?, false)
}

/// Protect an existing *owned* object; never take ownership or create a path.
/// Owner-only access plus SYSTEM is retained even for Unix modes with group bits.
/// MAXIMUM_ALLOWED is intentional: SetSecurityInfo documents that handles opened
/// this way do not propagate ACEs to existing children. The operation still needs
/// WRITE_DAC granted by the existing owner/DACL; it never takes ownership.
pub(crate) fn protect_current_user_path(path: &Path, mode: u32) -> io::Result<()> {
    if mode & !0o777 != 0
        || !path.is_absolute()
        || path.parent().is_none()
        || path.components().any(|x| matches!(x, Component::ParentDir))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "absolute non-root path and Unix permission bits required",
        ));
    }
    // Pin ancestors from the volume downward without FILE_SHARE_DELETE so a
    // checked parent cannot be renamed into a junction before the target opens.
    // Retain every handle until the final handle-based security update finishes.
    let mut parents = Vec::new();
    for ancestor in path
        .ancestors()
        .skip(1)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        let parent = OpenOptions::new()
            .access_mode(0x8000_0000) // GENERIC_READ includes directory-list access for delete sharing.
            .share_mode(3)
            .custom_flags(0x0200_0000 | 0x0020_0000)
            .open(ancestor)?;
        let metadata = parent.metadata()?;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(denied("private path cannot contain reparse points"));
        }
        parents.push(parent);
    }
    let file = OpenOptions::new()
        .access_mode(0x0200_0000) // MAXIMUM_ALLOWED; see the no-propagation contract.
        .share_mode(0)
        .custom_flags(0x0200_0000 | 0x0020_0000)
        .open(path)?;
    let metadata = file.metadata()?;
    if metadata.file_attributes() & 0x400 != 0 || !(metadata.is_file() || metadata.is_dir()) {
        return Err(denied("private target must be a regular owned object"));
    }
    let mut owner = ptr::null_mut();
    let mut raw = ptr::null_mut();
    let status = unsafe {
        GetSecurityInfo(
            file.as_raw_handle(),
            1,
            1,
            &mut owner,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut raw,
        )
    };
    if status != 0 {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    let descriptor = SecurityDescriptor(raw);
    if owner.is_null() || unsafe { sid_text(owner)? } != current_user_sid()? {
        return Err(denied("private object belongs to another user"));
    }
    let private = SecurityDescriptor::private(&current_user_sid()?, metadata.is_dir())?;
    let status = unsafe {
        SetSecurityInfo(
            file.as_raw_handle(),
            1,
            4 | 0x8000_0000,
            ptr::null_mut(),
            ptr::null_mut(),
            private.dacl()?,
            ptr::null(),
        )
    };
    drop(descriptor);
    if status != 0 {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    Ok(())
}

/// Must follow reading a bounded request from this connection. No await occurs
/// while impersonating; even a rejected peer is reverted before returning.
pub(crate) fn verify_pipe_client(pipe: HANDLE) -> io::Result<()> {
    let expected = current_user_sid()?;
    if unsafe { ImpersonateNamedPipeClient(pipe) } == 0 {
        return Err(io::Error::last_os_error());
    }
    struct Revert;
    impl Drop for Revert {
        fn drop(&mut self) {
            // Continuing on a Tokio worker with a client token is unsafe.
            if unsafe { RevertToSelf() } == 0 {
                std::process::abort();
            }
        }
    }
    let revert = Revert;
    let result = (|| {
        let mut token = ptr::null_mut();
        if unsafe { OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, 1, &mut token) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let token = Token(token);
        if token_sid(token.0)? != expected {
            return Err(denied("IPC client SID differs from manager SID"));
        }
        Ok(())
    })();
    drop(revert);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn security_bytes(path: &Path) -> Vec<u8> {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Security::GetFileSecurityW;
        let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut needed = 0;
        unsafe {
            GetFileSecurityW(path.as_ptr(), 4, ptr::null_mut(), 0, &mut needed);
        }
        assert!((1..65536).contains(&needed));
        let mut bytes = vec![0u8; needed as usize];
        assert_ne!(
            unsafe {
                GetFileSecurityW(
                    path.as_ptr(),
                    4,
                    bytes.as_mut_ptr().cast(),
                    needed,
                    &mut needed,
                )
            },
            0
        );
        bytes.truncate(needed as usize);
        bytes
    }
    #[test]
    fn token_folders_and_sid_are_real_absolute_os_values() {
        assert!(current_user_sid().unwrap().starts_with("S-1-5-"));
        assert!(current_user_profile().unwrap().is_dir());
        assert!(current_user_local_app_data().unwrap().is_absolute());
    }
    #[test]
    fn private_file_remains_readable_and_missing_path_is_not_created() {
        let t = tempfile::tempdir().unwrap();
        let file = t.path().join("秘密 %.json");
        std::fs::write(&file, b"synthetic").unwrap();
        protect_current_user_path(&file, 0o600).unwrap();
        assert_eq!(std::fs::read(&file).unwrap(), b"synthetic");
        let missing = t.path().join("missing");
        assert_eq!(
            protect_current_user_path(&missing, 0o700)
                .unwrap_err()
                .kind(),
            io::ErrorKind::NotFound
        );
        assert!(!missing.exists());
    }
    #[test]
    fn private_directory_does_not_rewrite_existing_child_security() {
        let t = tempfile::tempdir().unwrap();
        let child = t.path().join("retained");
        std::fs::write(&child, b"retained").unwrap();
        let before = security_bytes(&child);
        protect_current_user_path(t.path(), 0o700).unwrap();
        assert_eq!(
            security_bytes(&child),
            before,
            "directory ACL protection must not rewrite existing child DACLs"
        );
        assert_eq!(std::fs::read(child).unwrap(), b"retained");
    }
    #[test]
    fn invalid_path_and_pipe_fail_closed() {
        assert!(protect_current_user_path(Path::new("relative"), 0o700).is_err());
        assert!(verify_pipe_client(ptr::null_mut()).is_err());
        assert!(verify_pipe_server(ptr::null_mut()).is_err());
    }
    #[test]
    fn junction_cannot_redirect_permission_changes_into_another_tree() {
        use std::os::windows::process::CommandExt;
        let t = tempfile::tempdir().unwrap();
        let target = t.path().join("retained-target");
        std::fs::create_dir(&target).unwrap();
        let file = target.join("private.json");
        std::fs::write(&file, b"retain").unwrap();
        let before = security_bytes(&file);
        let link = t.path().join("junction");
        let powershell = PathBuf::from(std::env::var_os("SystemRoot").unwrap())
            .join("System32/WindowsPowerShell/v1.0/powershell.exe");
        let output = std::process::Command::new(powershell)
            .creation_flags(0x0800_0000)
            .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command",
                "$ErrorActionPreference='Stop'; New-Item -ItemType Junction -Path $env:OCX_WIN23_JUNCTION -Target $env:OCX_WIN23_TARGET | Out-Null"])
            .env("OCX_WIN23_JUNCTION", &link).env("OCX_WIN23_TARGET", &target)
            .output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            protect_current_user_path(&link.join("private.json"), 0o600)
                .unwrap_err()
                .kind(),
            io::ErrorKind::PermissionDenied
        );
        assert_eq!(security_bytes(&file), before);
        assert_eq!(std::fs::read(&file).unwrap(), b"retain");
        std::fs::remove_dir(link).unwrap();
    }
    #[test]
    fn spoofed_user_environment_cannot_change_token_identity() {
        const CHILD: &str = "OCX_WIN23_IDENTITY_CHILD";
        let profile = current_user_profile().unwrap();
        let local = current_user_local_app_data().unwrap();
        let sid = current_user_sid().unwrap();
        if let Some(expected) = std::env::var_os(CHILD) {
            assert_eq!(
                expected,
                format!("{sid}|{}|{}", profile.display(), local.display()).as_str()
            );
            return;
        }
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "spoofed_user_environment_cannot_change_token_identity",
                "--nocapture",
            ])
            .env(
                CHILD,
                format!("{sid}|{}|{}", profile.display(), local.display()),
            )
            .env("USERNAME", "different-user")
            .env("USERPROFILE", r"C:\synthetic-other-user")
            .env("HOME", r"C:\synthetic-other-user")
            .env("LOCALAPPDATA", r"C:\synthetic-other-user\local")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
