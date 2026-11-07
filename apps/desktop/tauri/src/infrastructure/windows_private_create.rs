//! Create new private objects with an explicit TokenUser owner, including when
//! an elevated token's default owner is Administrators. Never repair ownership.
//! Register this module only on Windows. Existing objects are read-only inputs.
#![cfg(windows)]

use crate::infrastructure::windows_startup_root::user_scope::current_user_sid;
use std::{
    ffi::c_void,
    fs::{File, OpenOptions},
    io,
    os::windows::{
        ffi::OsStrExt,
        fs::OpenOptionsExt,
        io::{AsRawHandle, FromRawHandle},
    },
    path::{Component, Path, Prefix},
    ptr,
};
use windows_sys::Win32::{
    Foundation::{LocalFree, HANDLE, INVALID_HANDLE_VALUE},
    Security::{
        GetAce, GetSecurityDescriptorControl, GetSecurityDescriptorDacl, IsValidAcl, IsValidSid,
        ACCESS_ALLOWED_ACE, ACE_HEADER, ACL, SECURITY_ATTRIBUTES, SE_DACL_PROTECTED,
    },
    Storage::FileSystem::{
        CreateDirectoryW, CreateFileW, GetFileInformationByHandle, GetFileType,
        BY_HANDLE_FILE_INFORMATION, CREATE_NEW, FILE_ALL_ACCESS, FILE_ATTRIBUTE_DIRECTORY,
        FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS,
        FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_TYPE_DISK,
    },
};

// These SDK APIs are outside the app's existing windows-sys Authorization
// feature. Keep their real typed ACL/PSID/SD pointer signatures; no feature edit.
#[link(name = "advapi32")]
extern "system" {
    fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
        text: *const u16,
        revision: u32,
        descriptor: *mut *mut c_void,
        size: *mut u32,
    ) -> i32;
    fn ConvertSidToStringSidW(sid: *const c_void, text: *mut *mut u16) -> i32;
    fn GetSecurityInfo(
        handle: HANDLE,
        kind: i32,
        information: u32,
        owner: *mut *mut c_void,
        group: *mut *mut c_void,
        dacl: *mut *mut ACL,
        sacl: *mut *mut ACL,
        descriptor: *mut *mut c_void,
    ) -> u32;
}
const READ_CONTROL: u32 = 0x0002_0000;
const GENERIC_READ: u32 = 0x8000_0000;
const GENERIC_READ_WRITE: u32 = 0xc000_0000;
const OWNER_AND_DACL: u32 = 1 | 4;
const SYSTEM_SID: &str = "S-1-5-18";
const ADMINISTRATORS_SID: &str = "S-1-5-32-544";

fn denied(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, message)
}
fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
struct LocalMemory(*mut c_void);
impl Drop for LocalMemory {
    fn drop(&mut self) {
        // SAFETY: only LocalAlloc-owned SD/SID strings enter this wrapper.
        unsafe {
            LocalFree(self.0);
        }
    }
}
fn user_sid() -> io::Result<String> {
    let sid = current_user_sid()?;
    if sid == SYSTEM_SID || sid == ADMINISTRATORS_SID {
        return Err(denied(
            "private creation requires a user token, not SYSTEM or Administrators",
        ));
    }
    Ok(sid)
}
fn private_descriptor(sid: &str, directory: bool) -> io::Result<LocalMemory> {
    let inherit = if directory { "OICI" } else { "" };
    descriptor(&format!(
        "O:{sid}D:P(A;{inherit};FA;;;{sid})(A;{inherit};FA;;;SY)"
    ))
}
fn descriptor(sddl: &str) -> io::Result<LocalMemory> {
    let text: Vec<u16> = sddl.encode_utf16().chain(Some(0)).collect();
    let mut raw = ptr::null_mut();
    // SAFETY: nul-terminated SDDL; LocalMemory retains the resulting SD.
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            text.as_ptr(),
            1,
            &mut raw,
            ptr::null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if raw.is_null() {
        return Err(denied(
            "private creation security descriptor is unavailable",
        ));
    }
    Ok(LocalMemory(raw))
}
fn attributes(sd: &LocalMemory) -> SECURITY_ATTRIBUTES {
    SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: sd.0,
        bInheritHandle: 0,
    }
}
// The private Windows projection is User/SYSTEM FA, never group/world access.
// Creation returns a read/write File, so read-only or inaccessible Unix intents
// are unsupported and fail before creating anything, rather than becoming no-ops.
fn validate(path: &Path, mode: u32, directory: bool) -> io::Result<()> {
    let required = if directory { 0o700 } else { 0o600 };
    if mode & !0o777 != 0 || mode & required != required {
        return Err(invalid(
            "private creation requires owner rw (directories rwx) permission bits",
        ));
    }
    if !path.is_absolute() || path.file_name().is_none() {
        return Err(invalid(
            "private creation requires an absolute non-root path",
        ));
    }
    let mut components = path.components();
    if !matches!(components.next(), Some(Component::Prefix(p)) if matches!(p.kind(),
        Prefix::Disk(_) | Prefix::VerbatimDisk(_) | Prefix::UNC(_, _) | Prefix::VerbatimUNC(_, _)))
        || path.components().any(|c| matches!(c, Component::ParentDir))
    {
        return Err(invalid(
            "private creation requires a drive or UNC filesystem path",
        ));
    }
    // Components normalizes interior '.', so reject it before that normalization.
    let units: Vec<u16> = path.as_os_str().encode_wide().collect();
    if units.contains(&0)
        || units
            .split(|c| *c == 47 || *c == 92)
            .any(|p| p == [46] || p == [46, 46])
    {
        return Err(invalid(
            "private creation rejects nul and dot path segments",
        ));
    }
    for c in path.components() {
        if let Component::Normal(name) = c {
            let name: Vec<u16> = name.encode_wide().collect();
            if name
                .iter()
                .any(|c| *c < 32 || [58, 60, 62, 34, 124, 63, 42].contains(c))
                || matches!(name.last(), Some(32 | 46))
            {
                return Err(invalid(
                    "private creation rejects streams, ambiguous and device paths",
                ));
            }
            let stem: String = name
                .iter()
                .take_while(|c| **c != 46)
                .map(|c| char::from_u32(*c as u32).unwrap_or('\u{fffd}'))
                .collect();
            let stem = stem.to_ascii_uppercase();
            if ["CON", "PRN", "AUX", "NUL"].contains(&stem.as_str())
                || ["COM", "LPT"].iter().any(|p| {
                    stem.strip_prefix(p).is_some_and(|n| {
                        matches!(
                            n,
                            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                        )
                    })
                })
            {
                return Err(invalid("private creation rejects DOS device names"));
            }
        }
    }
    Ok(())
}
fn wide_path(path: &Path) -> Vec<u16> {
    // Extended filesystem syntax preserves Unicode/long paths; validation above
    // excludes NT device namespaces, ADS and Win32-normalized ambiguous names.
    let units: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .map(|c| if c == 47 { 92 } else { c })
        .collect();
    let mut result: Vec<u16> = match path.components().next() {
        Some(Component::Prefix(p)) if matches!(p.kind(), Prefix::Disk(_)) => {
            "\\\\?\\".encode_utf16().collect()
        }
        Some(Component::Prefix(p)) if matches!(p.kind(), Prefix::UNC(_, _)) => {
            let mut out: Vec<u16> = "\\\\?\\UNC\\".encode_utf16().collect();
            out.extend_from_slice(&units[2..]);
            out.push(0);
            return out;
        }
        _ => vec![],
    };
    result.extend(units);
    result.push(0);
    result
}
fn information(
    file: &File,
    directory: bool,
    check_links: bool,
) -> io::Result<BY_HANDLE_FILE_INFORMATION> {
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    // SAFETY: a retained File handle and a correctly sized output structure.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { GetFileType(file.as_raw_handle()) } != FILE_TYPE_DISK
        || info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || (info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0) != directory
        || check_links && info.nNumberOfLinks != 1
    {
        return Err(denied(
            "private object has an unexpected type, reparse point or link count",
        ));
    }
    Ok(info)
}
fn open_directory(path: &Path, security: bool) -> io::Result<File> {
    let file = OpenOptions::new()
        // Attribute-only opens do not participate in the data-access sharing
        // check: Windows can rename that directory despite no SHARE_DELETE.
        // GENERIC_READ maps to FILE_LIST_DIRECTORY plus read metadata/control,
        // so the retained handle participates in delete/rename share admission.
        .access_mode(GENERIC_READ | if security { READ_CONTROL } else { 0 })
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE) // Intentionally no FILE_SHARE_DELETE.
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    information(&file, true, false)?;
    Ok(file)
}
fn pin_parents(path: &Path) -> io::Result<Vec<File>> {
    let mut pins = Vec::new();
    for parent in path
        .ancestors()
        .skip(1)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        pins.push(open_directory(parent, false)?);
    }
    Ok(pins)
}
fn read_security(file: &File) -> io::Result<(LocalMemory, *mut c_void, *mut ACL)> {
    let mut owner = ptr::null_mut();
    let mut dacl = ptr::null_mut();
    let mut sd = ptr::null_mut();
    // SAFETY: output pointers receive fields inside the returned LocalAlloc SD.
    let result = unsafe {
        GetSecurityInfo(
            file.as_raw_handle(),
            1,
            OWNER_AND_DACL,
            &mut owner,
            ptr::null_mut(),
            &mut dacl,
            ptr::null_mut(),
            &mut sd,
        )
    };
    if result != 0 {
        return Err(io::Error::from_raw_os_error(result as i32));
    }
    if sd.is_null() {
        return Err(denied("private object security descriptor is unavailable"));
    }
    Ok((LocalMemory(sd), owner, dacl))
}
unsafe fn sid_string(sid: *const c_void) -> io::Result<String> {
    if sid.is_null() || unsafe { IsValidSid(sid.cast_mut()) } == 0 {
        return Err(denied("private object SID is unavailable or invalid"));
    }
    let mut raw = ptr::null_mut();
    if unsafe { ConvertSidToStringSidW(sid, &mut raw) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let _memory = LocalMemory(raw.cast());
    let mut len = 0;
    while len < 256 && unsafe { *raw.add(len) } != 0 {
        len += 1;
    }
    if len == 256 {
        return Err(denied("private object SID exceeds limit"));
    }
    String::from_utf16(unsafe { std::slice::from_raw_parts(raw, len) })
        .map_err(|_| denied("private object SID is invalid"))
}
fn verify(file: &File, directory: bool, expected: &str) -> io::Result<()> {
    information(file, directory, true)?;
    let (sd, owner, dacl) = read_security(file)?;
    if unsafe { sid_string(owner)? } != expected {
        return Err(denied("private object owner differs from TokenUser"));
    }
    let mut control = 0u16;
    let mut revision = 0;
    let mut present = 0;
    let mut defaulted = 0;
    let mut actual = ptr::null_mut();
    if unsafe { GetSecurityDescriptorControl(sd.0, &mut control, &mut revision) } == 0
        || unsafe { GetSecurityDescriptorDacl(sd.0, &mut present, &mut actual, &mut defaulted) }
            == 0
    {
        return Err(io::Error::last_os_error());
    }
    if control & SE_DACL_PROTECTED == 0
        || present == 0
        || defaulted != 0
        || dacl.is_null()
        || actual != dacl
        || unsafe { IsValidAcl(dacl) } == 0
        || unsafe { (*dacl).AceCount } != 2
    {
        return Err(denied(
            "private object requires an explicit protected two-principal DACL",
        ));
    }
    let mut user = false;
    let mut system = false;
    for i in 0..2 {
        let mut raw = ptr::null_mut();
        if unsafe { GetAce(dacl, i, &mut raw) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let header = unsafe { &*raw.cast::<ACE_HEADER>() };
        let flags = if directory { 3 } else { 0 }; // OI | CI, never INHERITED_ACE.
        if header.AceType != 0 || header.AceFlags != flags || header.AceSize < 16 {
            return Err(denied("private object contains an unexpected ACE"));
        }
        let ace = unsafe { &*raw.cast::<ACCESS_ALLOWED_ACE>() };
        let sid = ptr::addr_of!(ace.SidStart).cast::<c_void>();
        let count = unsafe { *sid.cast::<u8>().add(1) } as usize;
        if count > 15 || 8 + 8 + count * 4 != header.AceSize as usize || ace.Mask != FILE_ALL_ACCESS
        {
            return Err(denied(
                "private object contains an unexpected access mask or SID size",
            ));
        }
        let principal = unsafe { sid_string(sid)? };
        if principal == expected && !user {
            user = true;
        } else if principal == SYSTEM_SID && !system {
            system = true;
        } else {
            return Err(denied(
                "private object grants access to an unexpected principal",
            ));
        }
    }
    if !user || !system {
        return Err(denied(
            "private object does not match User/SYSTEM authority",
        ));
    }
    Ok(())
}

/// Read-only owner/DACL/type/link verification for an already retained handle.
/// Callers must open with READ_CONTROL and pin the pathname without share-delete
/// if they rely on that pathname. This never sets permissions or takes ownership.
pub(crate) fn verify_private_handle(file: &File, directory: bool) -> io::Result<()> {
    verify(file, directory, &user_sid()?)
}

/// CREATE_NEW only, read/write, non-inheritable handle. Parents must exist; every
/// ancestor is pinned and checked for reparse points throughout create+readback.
/// A verification failure leaves any uncertain new object untouched, not deleted
/// by a pathname that might now refer to a different object.
pub(crate) fn create_new_private_file(path: &Path, mode: u32) -> io::Result<File> {
    validate(path, mode, false)?;
    let sid = user_sid()?;
    let _parents = pin_parents(path)?;
    let sd = private_descriptor(&sid, false)?;
    let sa = attributes(&sd);
    let name = wide_path(path);
    // SAFETY: retained descriptor/UTF16 path, CREATE_NEW does not open or replace
    // an existing object. File owns the successful, non-inherited handle.
    let raw = unsafe {
        CreateFileW(
            name.as_ptr(),
            GENERIC_READ_WRITE | READ_CONTROL | 0x0004_0000, // WRITE_DAC for same-owner permission finalization
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            &sa,
            CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OPEN_REPARSE_POINT,
            ptr::null_mut(),
        )
    };
    if raw == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let file = unsafe { File::from_raw_handle(raw) };
    verify(&file, false, &sid)?;
    Ok(file)
}

/// Reopen an already private file without truncation while pinning ancestors.
/// This is used only after CREATE_NEW reported an existing lock pathname.
pub(crate) fn open_existing_private_file(path: &Path, mode: u32) -> io::Result<File> {
    validate(path, mode, false)?;
    let _parents = pin_parents(path)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(3)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    verify_private_handle(&file, false)?;
    Ok(file)
}

/// Create only missing directories; existing OS ancestors are pinned/read-only.
/// The target, and any racing object at a missing step, must already satisfy the
/// actual TokenUser owner and private DACL contract. No existing ACL is changed.
pub(crate) fn create_private_dir_all(path: &Path, mode: u32) -> io::Result<()> {
    validate(path, mode, true)?;
    let sid = user_sid()?;
    let sd = private_descriptor(&sid, true)?;
    let sa = attributes(&sd);
    let mut pins = Vec::new();
    for entry in path.ancestors().collect::<Vec<_>>().into_iter().rev() {
        let target = entry == path;
        match open_directory(entry, target) {
            Ok(file) => {
                if target {
                    verify(&file, true, &sid)?;
                }
                pins.push(file);
            }
            Err(error)
                if matches!(error.raw_os_error(), Some(2 | 3)) && entry.parent().is_some() =>
            {
                let name = wide_path(entry);
                if unsafe { CreateDirectoryW(name.as_ptr(), &sa) } == 0 {
                    let error = io::Error::last_os_error();
                    if error.raw_os_error() != Some(183) {
                        return Err(error);
                    }
                }
                let file = open_directory(entry, true)?;
                verify(&file, true, &sid)?;
                pins.push(file);
            }
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Seek, SeekFrom, Write};
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        Security::{
            GetSecurityDescriptorLength, GetTokenInformation, TokenElevation, TokenOwner,
            TOKEN_ELEVATION, TOKEN_OWNER, TOKEN_QUERY,
        },
        Storage::FileSystem::FILE_READ_ATTRIBUTES,
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    };

    #[link(name = "kernel32")]
    extern "system" {
        fn GetSystemDirectoryW(buffer: *mut u16, size: u32) -> u32;
    }
    fn owner(file: &File) -> String {
        let (_sd, sid, _) = read_security(file).unwrap();
        unsafe { sid_string(sid) }.unwrap()
    }
    fn security_bytes(file: &File) -> Vec<u8> {
        let (sd, _, _) = read_security(file).unwrap();
        let len = unsafe { GetSecurityDescriptorLength(sd.0) } as usize;
        assert!((20..=65536).contains(&len));
        unsafe { std::slice::from_raw_parts(sd.0.cast::<u8>(), len) }.to_vec()
    }
    fn token_evidence() -> (bool, String) {
        let mut raw = ptr::null_mut();
        assert_ne!(
            unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw) },
            0
        );
        struct Token(HANDLE);
        impl Drop for Token {
            fn drop(&mut self) {
                unsafe {
                    CloseHandle(self.0);
                }
            }
        }
        let token = Token(raw);
        let mut elevated = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut needed = 0;
        assert_ne!(
            unsafe {
                GetTokenInformation(
                    token.0,
                    TokenElevation,
                    (&mut elevated as *mut TOKEN_ELEVATION).cast(),
                    std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                    &mut needed,
                )
            },
            0
        );
        unsafe {
            GetTokenInformation(token.0, TokenOwner, ptr::null_mut(), 0, &mut needed);
        }
        assert!((std::mem::size_of::<TOKEN_OWNER>() as u32..=65536).contains(&needed));
        let mut memory = vec![0usize; (needed as usize).div_ceil(std::mem::size_of::<usize>())];
        assert_ne!(
            unsafe {
                GetTokenInformation(
                    token.0,
                    TokenOwner,
                    memory.as_mut_ptr().cast(),
                    needed,
                    &mut needed,
                )
            },
            0
        );
        let owner = unsafe { sid_string((*memory.as_ptr().cast::<TOKEN_OWNER>()).Owner) }.unwrap();
        (elevated.TokenIsElevated != 0, owner)
    }
    fn fixture_dir(t: &tempfile::TempDir, name: &str) -> std::path::PathBuf {
        let path = t.path().join(name);
        create_private_dir_all(&path, 0o700).unwrap();
        path
    }

    #[test]
    fn native_token_owner_readback_is_user_even_when_default_owner_is_admins() {
        let t = tempfile::tempdir().unwrap();
        let path = fixture_dir(&t, "private").join("new-file");
        let file = create_new_private_file(&path, 0o600).unwrap();
        let (elevated, default_owner) = token_evidence();
        let token_user = current_user_sid().unwrap();
        let actual_owner = owner(&file);
        eprintln!("token_user_sid={token_user} token_default_owner_sid={default_owner} elevated={elevated} created_owner_sid={actual_owner} owner_matches_user={}", actual_owner == token_user);
        assert_eq!(actual_owner, token_user);
        assert_ne!(actual_owner, ADMINISTRATORS_SID);
        assert_ne!(actual_owner, SYSTEM_SID);
        verify_private_handle(&file, false).unwrap();
    }

    #[test]
    fn native_file_is_read_write_exclusive_and_preserves_existing_bytes_and_acl() {
        let t = tempfile::tempdir().unwrap();
        let path = fixture_dir(&t, "private").join("unicode 路径.json");
        let mut file = create_new_private_file(&path, 0o644).unwrap();
        file.write_all(b"synthetic fixture").unwrap();
        file.sync_all().unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"synthetic fixture");
        let before = security_bytes(&file);
        assert_eq!(
            create_new_private_file(&path, 0o600).unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(security_bytes(&file), before);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }

    #[test]
    fn native_reopen_preserves_private_lock_bytes_without_truncation() {
        use std::io::{Read, Write};
        let temp = tempfile::tempdir().unwrap();
        let dir = fixture_dir(&temp, "reopen");
        let path = dir.join("private.lock");
        let mut file = create_new_private_file(&path, 0o600).unwrap();
        file.write_all(b"original-lock-owner").unwrap();
        drop(file);
        let before = std::fs::read(&path).unwrap();
        let mut opened = open_existing_private_file(&path, 0o600).unwrap();
        let mut actual = Vec::new();
        opened.read_to_end(&mut actual).unwrap();
        assert_eq!(actual, before);
        verify_private_handle(&opened, false).unwrap();
    }

    #[test]
    fn native_recursive_creation_preserves_os_parent_acl_and_existing_private_target() {
        let t = tempfile::tempdir().unwrap();
        // This parent may itself have default-owner=Administrators when elevated.
        // It is only pinned; no target-owner exception or ACL update is applied.
        let os_parent = open_directory(t.path(), true).unwrap();
        let before = security_bytes(&os_parent);
        let target = t.path().join("one/two/three");
        create_private_dir_all(&target, 0o755).unwrap();
        for entry in target.ancestors().take(3) {
            verify_private_handle(&open_directory(entry, true).unwrap(), true).unwrap();
        }
        assert_eq!(security_bytes(&os_parent), before);
        let existing = open_directory(&target, true).unwrap();
        let target_before = security_bytes(&existing);
        create_private_dir_all(&target, 0o700).unwrap();
        assert_eq!(security_bytes(&existing), target_before);
        assert_eq!(security_bytes(&os_parent), before);
    }

    #[test]
    fn native_owned_but_broad_target_is_rejected_without_acl_repair() {
        let t = tempfile::tempdir().unwrap();
        let path = t.path().join("owned-unproven");
        let sid = user_sid().unwrap();
        let sd = descriptor(&format!(
            "O:{sid}D:P(A;OICI;FA;;;{sid})(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)"
        ))
        .unwrap();
        assert_ne!(
            unsafe { CreateDirectoryW(wide_path(&path).as_ptr(), &attributes(&sd)) },
            0
        );
        let file = open_directory(&path, true).unwrap();
        assert_eq!(owner(&file), sid);
        let before = security_bytes(&file);
        assert_eq!(
            create_private_dir_all(&path, 0o700).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
        assert_eq!(security_bytes(&file), before);
    }

    #[test]
    fn native_foreign_os_target_is_rejected_without_taking_ownership() {
        let mut units = vec![0u16; 32768];
        let n = unsafe { GetSystemDirectoryW(units.as_mut_ptr(), units.len() as u32) } as usize;
        assert!(n > 0 && n < units.len());
        use std::os::windows::ffi::OsStringExt;
        let target = std::path::PathBuf::from(std::ffi::OsString::from_wide(&units[..n]));
        let file = open_directory(&target, true).unwrap();
        assert_ne!(owner(&file), current_user_sid().unwrap());
        let before = security_bytes(&file);
        assert_eq!(
            create_private_dir_all(&target, 0o700).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
        assert_eq!(security_bytes(&file), before);
    }

    #[test]
    fn native_existing_default_owner_is_evidence_not_permission_to_take_ownership() {
        let t = tempfile::tempdir().unwrap();
        let target = t.path().join("default-created");
        std::fs::create_dir(&target).unwrap();
        let file = open_directory(&target, true).unwrap();
        let actual_owner = owner(&file);
        let token_user = current_user_sid().unwrap();
        let (elevated, default_owner) = token_evidence();
        let before = security_bytes(&file);
        let already_private = verify_private_handle(&file, true).is_ok();
        let result = create_private_dir_all(&target, 0o700);
        eprintln!("token_user_sid={token_user} token_default_owner_sid={default_owner} elevated={elevated} existing_owner_sid={actual_owner} owner_matches_user={}", actual_owner == token_user);
        if actual_owner != token_user || !already_private {
            assert!(result.is_err());
        } else {
            result.unwrap();
        }
        assert_eq!(security_bytes(&file), before);
    }

    #[test]
    fn native_junction_in_parent_chain_and_leaf_is_rejected_without_target_changes() {
        let t = tempfile::tempdir().unwrap();
        let target = fixture_dir(&t, "retained");
        let file = open_directory(&target, true).unwrap();
        let before = security_bytes(&file);
        let junction = t.path().join("junction");
        // Junction creation does not need developer mode or symlink privilege.
        let mut system = vec![0u16; 32768];
        let n = unsafe { GetSystemDirectoryW(system.as_mut_ptr(), system.len() as u32) } as usize;
        assert!(n > 0 && n < system.len());
        use std::os::windows::ffi::OsStringExt;
        let powershell = std::path::PathBuf::from(std::ffi::OsString::from_wide(&system[..n]))
            .join("WindowsPowerShell/v1.0/powershell.exe");
        let status = std::process::Command::new(powershell)
            .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command",
                "$ErrorActionPreference='Stop'; New-Item -ItemType Junction -Path $env:OCXD_PRIVATE_CREATE_LINK -Target $env:OCXD_PRIVATE_CREATE_TARGET | Out-Null"])
            .env("OCXD_PRIVATE_CREATE_LINK", &junction)
            .env("OCXD_PRIVATE_CREATE_TARGET", &target)
            .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
            .status().unwrap();
        assert!(status.success());
        assert!(create_new_private_file(&junction.join("blocked"), 0o600).is_err());
        assert!(create_private_dir_all(&junction.join("blocked-dir"), 0o700).is_err());
        assert!(create_private_dir_all(&junction, 0o700).is_err());
        assert!(!target.join("blocked").exists());
        assert!(!target.join("blocked-dir").exists());
        assert_eq!(security_bytes(&file), before);
        std::fs::remove_dir(junction).unwrap();
    }

    #[test]
    fn native_hard_link_is_not_a_verified_private_single_link_file() {
        let t = tempfile::tempdir().unwrap();
        let path = fixture_dir(&t, "private").join("single");
        let file = create_new_private_file(&path, 0o600).unwrap();
        let before = security_bytes(&file);
        drop(file);
        std::fs::hard_link(&path, path.with_file_name("second-link")).unwrap();
        let file = OpenOptions::new()
            .access_mode(READ_CONTROL | FILE_READ_ATTRIBUTES)
            .share_mode(3)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&path)
            .unwrap();
        assert_eq!(
            verify_private_handle(&file, false).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
        assert_eq!(security_bytes(&file), before);
        assert!(create_new_private_file(&path, 0o600).is_err());
    }

    #[test]
    fn native_pinned_ancestor_cannot_be_renamed_during_create_or_validation() {
        let t = tempfile::tempdir().unwrap();
        let parent = fixture_dir(&t, "pinned");
        let leaf = parent.join("leaf");
        let moved = parent.with_file_name("moved");
        let pins = pin_parents(&leaf).unwrap();
        let held = pins.last().unwrap();
        let before = information(held, true, false).unwrap();
        let race_parent = parent.clone();
        let race_moved = moved.clone();
        // A real competing pathname rename must fail while the stronger handle
        // is held, not merely after a metadata-only path check.
        let rename = std::thread::spawn(move || std::fs::rename(race_parent, race_moved))
            .join()
            .unwrap();
        assert!(
            rename.is_err(),
            "ancestor rename escaped the retained directory handle"
        );
        let created = create_new_private_file(&leaf, 0o600).unwrap();
        verify_private_handle(&created, false).unwrap();
        let at_path = open_directory(&parent, false).unwrap();
        let after = information(&at_path, true, false).unwrap();
        assert_eq!(
            (
                before.dwVolumeSerialNumber,
                before.nFileIndexHigh,
                before.nFileIndexLow
            ),
            (
                after.dwVolumeSerialNumber,
                after.nFileIndexHigh,
                after.nFileIndexLow
            )
        );
        assert!(leaf.is_file());
        assert!(!moved.exists());
        assert!(!moved.join("leaf").exists());
        drop(created);
        drop(at_path);
        drop(pins);
        std::fs::rename(&parent, &moved).unwrap();
        assert!(moved.join("leaf").is_file());
    }

    #[test]
    fn native_system_directory_ancestors_allow_read_pins_without_security_changes() {
        let mut units = vec![0u16; 32768];
        let n = unsafe { GetSystemDirectoryW(units.as_mut_ptr(), units.len() as u32) } as usize;
        assert!(n > 0 && n < units.len());
        use std::os::windows::ffi::OsStringExt;
        let system = std::path::PathBuf::from(std::ffi::OsString::from_wide(&units[..n]));
        let directory = open_directory(&system, true).unwrap();
        let before = security_bytes(&directory);
        let pins = pin_parents(&system.join("read-only-pin-probe")).unwrap();
        let held = information(pins.last().unwrap(), true, false).unwrap();
        let original = information(&directory, true, false).unwrap();
        assert_eq!(
            (
                held.dwVolumeSerialNumber,
                held.nFileIndexHigh,
                held.nFileIndexLow
            ),
            (
                original.dwVolumeSerialNumber,
                original.nFileIndexHigh,
                original.nFileIndexLow
            )
        );
        assert_eq!(security_bytes(&directory), before);
        // No file or directory is created in the system-owned ancestor.
    }

    #[test]
    fn native_unicode_long_paths_use_explicit_owner_and_private_acl() {
        let t = tempfile::tempdir().unwrap();
        let mut directory = t.path().to_owned();
        for _ in 0..6 {
            directory.push("unicode-用户-long-component-0123456789");
        }
        create_private_dir_all(&directory, 0o700).unwrap();
        let path = directory.join("原始对象-private.json");
        assert!(path.as_os_str().encode_wide().count() > 260);
        let mut file = create_new_private_file(&path, 0o600).unwrap();
        file.write_all(b"synthetic fixture").unwrap();
        file.sync_all().unwrap();
        verify_private_handle(&file, false).unwrap();
        assert_eq!(owner(&file), current_user_sid().unwrap());
    }

    #[test]
    fn native_invalid_modes_and_ambiguous_paths_create_nothing() {
        let t = tempfile::tempdir().unwrap();
        let path = t.path().join("not-created");
        for mode in [0, 0o400, 0o200, 0o1000] {
            assert!(create_new_private_file(&path, mode).is_err());
        }
        for mode in [0, 0o600, 0o1000] {
            assert!(create_private_dir_all(&path, mode).is_err());
        }
        assert!(!path.exists());
        for path in [
            Path::new("relative"),
            Path::new(r"C:\"),
            Path::new(r"C:\parent\..\child"),
            Path::new(r"C:\parent\.\child"),
            Path::new(r"C:\parent\name:stream"),
            Path::new(r"C:\parent\NUL"),
            Path::new(r"C:\parent\trailing."),
            Path::new(r"\\.\pipe\private"),
        ] {
            assert!(validate(path, 0o600, false).is_err());
        }
    }
}
