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
pub(crate) fn pin_parents(path: &Path) -> io::Result<Vec<File>> {
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
    verify_authority(file, directory, expected, true)
}
fn verify_authority(
    file: &File,
    directory: bool,
    expected: &str,
    require_system: bool,
) -> io::Result<()> {
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
        || !matches!(unsafe { (*dacl).AceCount }, 1 | 2)
        || require_system && unsafe { (*dacl).AceCount } != 2
    {
        return Err(denied(
            "private object requires an explicit protected two-principal DACL",
        ));
    }
    let mut user = false;
    let mut system = false;
    for i in 0..unsafe { (*dacl).AceCount } as u32 {
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
    if !user || require_system && !system {
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

/// The official runtime hardens its secret record to a protected owner-only
/// DACL. That is also private: SYSTEM is optional for this read-only consumer.
/// Owner, protection, exact full-control grants, ordinary single-link file and
/// rejection of every other principal remain identical. Never repairs an ACL.
pub(crate) fn verify_runtime_record_handle(file: &File) -> io::Result<()> {
    verify_authority(file, false, &user_sid()?, false)
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
#[path = "../../../../../test/smoke/windows023-lifetime-binding.rs"]
mod candidate_lifetime_binding;

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

    #[test]
    #[ignore = "requires ephemeral Windows Actions and the exact pinned runtime"]
    fn native_original_runtime_record_diagnostic() {
        assert_eq!(std::env::var("GITHUB_ACTIONS").as_deref(), Ok("true"));
        let package = std::path::PathBuf::from(
            std::env::var_os("OCXD_RECORD_PROBE_PACKAGE").expect("explicit pinned package"),
        );
        let bun = std::path::PathBuf::from(
            std::env::var_os("OCXD_RECORD_PROBE_BUN").expect("explicit pinned Bun"),
        );
        let evidence = std::path::PathBuf::from(
            std::env::var_os("OCXD_RECORD_PROBE_REPORT").expect("fresh evidence file"),
        );
        let boundary = std::path::PathBuf::from(std::env::var_os("RUNNER_TEMP").unwrap());
        assert!(package.is_absolute() && bun.is_absolute() && evidence.is_absolute());
        assert!(package.starts_with(&boundary) && bun.starts_with(&boundary));
        assert!(evidence.starts_with(&boundary) && !evidence.exists());
        let task = tempfile::Builder::new()
            .prefix("ocxd-original-record-")
            .tempdir_in(&boundary)
            .unwrap();
        let home = task.path().join("profile");
        let h = home.join(".codex-desktop/opencodex/home");
        std::fs::create_dir_all(&home).unwrap();
        // Call the unchanged atomic writer used by writeRuntimePort, with a
        // synthetic key. No registry, proxy, service or real client is touched.
        let original_writer = r#"
          const {mkdirSync}=await import('node:fs');
          const {join}=await import('node:path');
          const {pathToFileURL}=await import('node:url');
          const runtimePackage=process.env.OCXD_RECORD_PROBE_PACKAGE;
          const metadata=await Bun.file(join(runtimePackage,'package.json')).json();
          if(metadata.version!=='2.97.0-desktop023.names.2')throw Error('Pinned package required');
          const {atomicWriteFile}=await import(pathToFileURL(join(runtimePackage,'src/config/atomic-write.ts')).href);
          const h=process.env.OPENCODEX_HOME;
          mkdirSync(h,{recursive:true,mode:0o700});
          atomicWriteFile(join(h,'runtime-port.json'),JSON.stringify({pid:process.pid,port:1,attestationSecret:'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA'}));
        "#;
        let child = std::process::Command::new(&bun)
            .args(["--no-env-file", "-e", original_writer])
            .env("OCXD_RECORD_PROBE_PACKAGE", &package)
            .env("HOME", &home)
            .env("USERPROFILE", &home)
            .env("APPDATA", home.join("AppData/Roaming"))
            .env("LOCALAPPDATA", home.join("AppData/Local"))
            .env("CODEX_HOME", home.join(".codex"))
            .env("OPENCODEX_HOME", &h)
            .current_dir(task.path())
            .output()
            .expect("original atomic writer process");
        assert!(
            child.status.success(),
            "Original atomic writer failed; no acceptance claimed"
        );
        let file = crate::modules::backup::safety::open_file(&h.join("runtime-port.json"), false)
            .expect("read original record with product opener");
        let record_owner = owner(&file);
        let expected = user_sid().unwrap();
        let (elevated, default_owner) = token_evidence();
        let result = verify_runtime_record_handle(&file);
        let classification = match result.as_ref().err().map(|e| e.to_string()) {
            None => "accepted-by-original-validator",
            Some(s) if s == "private object owner differs from TokenUser" => "owner-mismatch",
            Some(s) if s.contains("DACL") => "dacl-contract",
            Some(s)
                if s.contains("ACE") || s.contains("access mask") || s.contains("principal") =>
            {
                "ace-contract"
            }
            Some(_) => "other-validation-error",
        };
        let mut report = serde_json::json!({
            "accepted":false,"diagnosticOnly":true,"runtimeChanged":false,
            "source":crate::build_commit(),"tokenElevated":elevated,
            "defaultOwnerIsAdministrators":default_owner==ADMINISTRATORS_SID,
            "recordOwnerMatchesTokenUser":record_owner==expected,
            "recordOwnerIsAdministrators":record_owner==ADMINISTRATORS_SID,
            "originalValidatorAccepted":result.is_ok(),"classification":classification,
            "productStarted":false,"businessNetworkUsed":false,"serviceMutation":false,
            "scope":"Original atomic writer and actual Desktop record validator only; not lifecycle acceptance"
        });
        drop(file);
        let mut prototype_failures: Vec<&str> = Vec::new();
        if std::env::var("OCXD_RECORD_PROBE_CANDIDATE").as_deref() == Ok("true") {
            let candidate_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let script = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../../test/smoke/windows023-native-private-record.mjs");
                let result = std::process::Command::new(&bun)
                    .arg("--no-env-file")
                    .arg(script)
                    .env("OCXD_RECORD_PROBE_PACKAGE", &package)
                    .env("OCXD_RECORD_PROBE_TASK", task.path())
                    .env("HOME", &home)
                    .env("USERPROFILE", &home)
                    .env("APPDATA", home.join("AppData/Roaming"))
                    .env("LOCALAPPDATA", home.join("AppData/Local"))
                    .env("CODEX_HOME", home.join(".codex"))
                    .env("OPENCODEX_HOME", &h)
                    .current_dir(task.path())
                    .output()
                    .expect("isolated new-owner prototype");
                std::fs::write(
                    evidence.with_file_name("candidate-record.stdout.log"),
                    &result.stdout,
                )
                .unwrap();
                std::fs::write(
                    evidence.with_file_name("candidate-record.stderr.log"),
                    &result.stderr,
                )
                .unwrap();
                assert!(
                    result.status.success(),
                    "Isolated native writer prototype failed"
                );
                let candidate = crate::modules::backup::safety::open_file(
                    &h.join("native-runtime-port.json"),
                    false,
                )
                .unwrap();
                assert_eq!(owner(&candidate), expected);
                verify_runtime_record_handle(&candidate)
                    .expect("original validator must accept prototype");
                report["candidateOriginalValidatorAccepted"] = serde_json::json!(true);
                report["candidateOwnerMatchesTokenUser"] = serde_json::json!(true);
                report["candidateReadback"] =
                    serde_json::from_slice::<serde_json::Value>(&result.stdout)
                        .expect("fixed synthetic prototype report");
                drop(candidate);
                let bridge = crate::modules::backup::safety::open_file(
                    &h.join("native-fd-runtime-port.json"),
                    false,
                )
                .unwrap();
                assert_eq!(owner(&bridge), expected);
                verify_runtime_record_handle(&bridge)
                    .expect("original validator must accept descriptor bridge");
                report["candidateDescriptorBridgeAccepted"] = serde_json::json!(true);
                if std::env::var("OCXD_RECORD_PROBE_HELPER").as_deref() == Ok("true") {
                    let helper = crate::modules::backup::safety::open_file(
                        &h.join("maintained-helper/runtime-port.json"),
                        false,
                    )
                    .unwrap();
                    assert_eq!(owner(&helper), expected);
                    verify_runtime_record_handle(&helper)
                        .expect("candidate module must satisfy original validator");
                    report["candidateMaintainedHelperAccepted"] = serde_json::json!(true);
                }
                if let Some(package_overlay) = std::env::var_os("OCXD_RECORD_PROBE_OVERLAY") {
                    let package_overlay = std::path::PathBuf::from(package_overlay);
                    assert!(package_overlay.starts_with(&boundary));
                    let overlay_home = task.path().join("atomic-overlay-home");
                    std::fs::create_dir(&overlay_home).unwrap();
                    let overlay_writer = r#"
                      const {mkdirSync,readFileSync,readdirSync}=await import('node:fs');
                      const {join}=await import('node:path');
                      const {pathToFileURL}=await import('node:url');
                      const runtimePackage=process.env.OCXD_RECORD_PROBE_PACKAGE;
                      const {atomicWriteFile}=await import(pathToFileURL(join(runtimePackage,'src/config/atomic-write.ts')).href);
                      const h=process.env.OPENCODEX_HOME;mkdirSync(h,{recursive:true});
                      const target=join(h,'runtime-port.json');
                      const first=JSON.stringify({pid:process.pid,port:1,attestationSecret:'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA'});
                      atomicWriteFile(target,first);
                      let failed=false;
                      try{atomicWriteFile(target,'synthetic-unpublished',undefined,{beforeRename(){throw Error('forced-before-publication');}});}catch(error){if(error.message!=='forced-before-publication')throw error;failed=true;}
                      if(!failed||readFileSync(target,'utf8')!==first||readdirSync(h).some(n=>n.startsWith('runtime-port.json.ocx.')))throw Error('Atomic pre-publication failure must preserve original and remove only own temp');
                      console.log(JSON.stringify({accepted:false,diagnosticOnly:true,actualAtomicEntryUsed:true,forcedPrePublicationFailurePreservedOriginal:true,noOwnTempResidue:true}));
                    "#;
                    let result = std::process::Command::new(&bun)
                        .args(["--no-env-file", "-e", overlay_writer])
                        .env("OCXD_RECORD_PROBE_PACKAGE", &package_overlay)
                        .env("HOME", &home)
                        .env("USERPROFILE", &home)
                        .env("APPDATA", home.join("AppData/Roaming"))
                        .env("LOCALAPPDATA", home.join("AppData/Local"))
                        .env("CODEX_HOME", home.join(".codex"))
                        .env("OPENCODEX_HOME", &overlay_home)
                        .current_dir(task.path())
                        .output()
                        .expect("isolated original atomic overlay");
                    std::fs::write(
                        evidence.with_file_name("candidate-atomic-overlay.stdout.log"),
                        &result.stdout,
                    )
                    .unwrap();
                    std::fs::write(
                        evidence.with_file_name("candidate-atomic-overlay.stderr.log"),
                        &result.stderr,
                    )
                    .unwrap();
                    assert!(
                        result.status.success(),
                        "Candidate atomic overlay must execute successfully"
                    );
                    let published = crate::modules::backup::safety::open_file(
                        &overlay_home.join("runtime-port.json"),
                        false,
                    )
                    .unwrap();
                    assert_eq!(owner(&published), expected);
                    verify_runtime_record_handle(&published)
                        .expect("actual atomic overlay must satisfy original validator");
                    report["candidateAtomicOverlayAccepted"] = serde_json::json!(true);
                    report["candidateAtomicOverlayReadback"] =
                        serde_json::from_slice::<serde_json::Value>(&result.stdout)
                            .expect("fixed atomic failure preservation report");
                    if let Some(archive_package) =
                        std::env::var_os("OCXD_RECORD_PROBE_ARCHIVE_PACKAGE")
                    {
                        let archive_package = std::path::PathBuf::from(archive_package);
                        assert!(archive_package.starts_with(&boundary));
                        let metadata: serde_json::Value = serde_json::from_slice(
                            &std::fs::read(archive_package.join("package.json")).unwrap(),
                        )
                        .unwrap();
                        assert_eq!(
                            metadata["version"],
                            "2.97.0-desktop023.lifecycle.1-record-prototype"
                        );
                        assert_eq!(metadata["desktop023RecordPrototype"]["candidateOnly"], true);
                        let archive_home = task.path().join("rebuilt-archive-home");
                        std::fs::create_dir(&archive_home).unwrap();
                        let result = std::process::Command::new(&bun)
                            .args(["--no-env-file", "-e", overlay_writer])
                            .env("OCXD_RECORD_PROBE_PACKAGE", &archive_package)
                            .env("HOME", &home)
                            .env("USERPROFILE", &home)
                            .env("APPDATA", home.join("AppData/Roaming"))
                            .env("LOCALAPPDATA", home.join("AppData/Local"))
                            .env("CODEX_HOME", home.join(".codex"))
                            .env("OPENCODEX_HOME", &archive_home)
                            .current_dir(task.path())
                            .output()
                            .expect("exact rebuilt prototype archive");
                        std::fs::write(
                            evidence.with_file_name("candidate-archive.stdout.log"),
                            &result.stdout,
                        )
                        .unwrap();
                        std::fs::write(
                            evidence.with_file_name("candidate-archive.stderr.log"),
                            &result.stderr,
                        )
                        .unwrap();
                        assert!(
                            result.status.success(),
                            "Rebuilt candidate archive actual atomic entry failed"
                        );
                        let published = crate::modules::backup::safety::open_file(
                            &archive_home.join("runtime-port.json"),
                            false,
                        )
                        .unwrap();
                        assert_eq!(owner(&published), expected);
                        verify_runtime_record_handle(&published)
                            .expect("actual rebuilt archive must satisfy unchanged validator");
                        report["candidateArchiveAccepted"] = serde_json::json!(true);
                        report["candidateArchiveReadback"] =
                            serde_json::from_slice::<serde_json::Value>(&result.stdout).unwrap();
                    }
                }
            }));
            if candidate_result.is_err() {
                prototype_failures.push("new-record-or-descriptor-bridge");
            }
            for (detached, self_job, candidate_lifetime) in [
                (false, false, false),
                (true, false, false),
                (false, true, false),
                (true, true, false),
                (false, true, true),
                (true, true, true),
            ] {
                let case_key = match (detached, self_job, candidate_lifetime) {
                    (false, false, false) => "leasePrototype",
                    (true, false, false) => "detachedLeasePrototype",
                    (false, true, false) => "selfJobLeasePrototype",
                    (true, true, false) => "detachedSelfJobLeasePrototype",
                    (false, _, true) => "candidateLifetimePrototype",
                    (true, _, true) => "detachedCandidateLifetimePrototype",
                };
                let lease_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let lease_root = task.path().join(case_key);
                    std::fs::create_dir(&lease_root).unwrap();
                    drop(
                        create_new_private_file(&lease_root.join("lifetime.lease"), 0o600).unwrap(),
                    );
                    let script = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("../../../test/smoke/windows023-native-lifecycle-lease.mjs");
                    let mut child = std::process::Command::new(&bun)
                        .arg("--no-env-file")
                        .arg(script)
                        .env("OCXD_RECORD_PROBE_PACKAGE", &package)
                        .env("OCXD_LEASE_PROBE_ROOT", &lease_root)
                        .env(
                            "OCXD_LEASE_PROBE_RUST_HANDSHAKE",
                            if candidate_lifetime { "true" } else { "false" },
                        )
                        .env(
                            "OCXD_LEASE_PROBE_CANDIDATE",
                            if candidate_lifetime { "true" } else { "false" },
                        )
                        .env(
                            "OCXD_LEASE_PROBE_SELF_JOB",
                            if self_job { "true" } else { "false" },
                        )
                        .env(
                            "OCXD_LEASE_PROBE_DETACHED",
                            if detached { "true" } else { "false" },
                        )
                        .env("HOME", &home)
                        .env("USERPROFILE", &home)
                        .env("APPDATA", home.join("AppData/Roaming"))
                        .env("LOCALAPPDATA", home.join("AppData/Local"))
                        .env("CODEX_HOME", home.join(".codex"))
                        .env("OPENCODEX_HOME", &h)
                        .current_dir(task.path())
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .spawn()
                        .expect("isolated native lifetime prototype");
                    let witness = if candidate_lifetime {
                        let until = std::time::Instant::now() + std::time::Duration::from_secs(12);
                        let result = loop {
                            if lease_root.join("awaiting-rust-live-witness").exists() {
                                let result=super::candidate_lifetime_binding::native::Witness::live(
                                    &lease_root.join("service-ready"), &lease_root, &expected, &bun,
                                ).and_then(|original| {
                                    let bytes=std::fs::read(lease_root.join("service-ready"))?;
                                    let value:serde_json::Value=serde_json::from_slice(&bytes).map_err(std::io::Error::other)?;
                                    let binding:crate::infrastructure::runtime_lifetime::Binding=serde_json::from_value(value["candidateBinding"].clone()).map_err(std::io::Error::other)?;
                                    let secret="A".repeat(43);
                                    use base64::Engine;
                                    let challenge=base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([1u8;32]);
                                    let proof=value["candidateProof"].as_str().ok_or_else(|| std::io::Error::other("Missing candidate proof"))?;
                                    let pid=value["pid"].as_u64().and_then(|p|u32::try_from(p).ok()).ok_or_else(||std::io::Error::other("Invalid candidate PID"))?;
                                    let signed=crate::infrastructure::runtime_lifetime::SignedChallenge{secret:&secret,challenge:&challenge,port:1,proof};
                                    if !binding.verify_proof(&lease_root,&expected,pid,&signed) { return Err(std::io::Error::other("Production protocol rejected live candidate")); }
                                    let install=bun.ancestors().nth(3).ok_or_else(||std::io::Error::other("Pinned Bun install unavailable"))?;
                                    let production=crate::infrastructure::runtime_lifetime::native::Witness::live(binding,&lease_root,&expected,install)?;
                                    Ok((original,production))
                                });
                                break result;
                            }
                            if child.try_wait().unwrap().is_some()
                                || std::time::Instant::now() >= until
                            {
                                break Err(std::io::Error::other(
                                    "Candidate live witness handshake did not arrive",
                                ));
                            }
                            std::thread::sleep(std::time::Duration::from_millis(20));
                        };
                        // Permit normal synthetic cleanup even when the consumer rejects.
                        // Failure is retained and asserted after collecting all child logs.
                        std::fs::write(lease_root.join("rust-live-witness-observed"), b"observed")
                            .unwrap();
                        Some(result)
                    } else {
                        None
                    };
                    let result = child
                        .wait_with_output()
                        .expect("collect exact native prototype exit");
                    std::fs::write(
                        evidence.with_file_name(format!("candidate-{case_key}.stdout.log")),
                        &result.stdout,
                    )
                    .unwrap();
                    std::fs::write(
                        evidence.with_file_name(format!("candidate-{case_key}.stderr.log")),
                        &result.stderr,
                    )
                    .unwrap();
                    assert!(
                        result.status.success(),
                        "Isolated native lifetime proof failed"
                    );
                    report[case_key] = serde_json::from_slice::<serde_json::Value>(&result.stdout)
                        .expect("fixed synthetic lifetime report");
                    if candidate_lifetime {
                        let (witness,mut production_witness) = witness.unwrap().expect(
                            "Independent and production live process/Job/lease witnesses must be valid",
                        );
                        witness.exited().expect("Same retained native handles must prove original process exit, emptyJob and freed lease");
                        assert!(production_witness.quiescent().expect("Production native exit proof must remain readable"),"Production witness must prove original exit/Job empty and hold exclusive lease");
                        report[case_key]["productionNativeLifetimeConsumerAccepted"] =
                            serde_json::json!(true);
                        report[case_key]["independentRustKernelWitnessAccepted"] =
                            serde_json::json!(true);
                        let lease = crate::modules::backup::safety::open_file(
                            &lease_root.join("runtime-lifetime.lease"),
                            false,
                        )
                        .unwrap();
                        assert_eq!(owner(&lease), expected);
                        verify_runtime_record_handle(&lease).expect(
                            "candidate lease must satisfy original private record validator",
                        );
                        assert!(super::candidate_lifetime_binding::verify_candidate_file(
                            &lease_root.join("service-ready"), &lease_root, &expected,
                        ), "Actual candidate TypeScript proof must verify in independent Rust reader; changed fields/key/challenge/port must refuse");
                        report[case_key]["independentRustBindingReaderAccepted"] =
                            serde_json::json!(true);
                    }
                }));
                if lease_result.is_err() {
                    prototype_failures.push(case_key);
                }
            }
        }
        if let Some(package_producer) = std::env::var_os("OCXD_PRODUCER_PROBE_PACKAGE")
            .filter(|_| prototype_failures.is_empty())
        {
            let producer_root = task.path().join("actual-producer");
            std::fs::create_dir(&producer_root).unwrap();
            let script = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../test/smoke/windows023-runtime-producer-probe.mjs");
            let result = std::process::Command::new(&bun)
                .arg("--no-env-file")
                .arg(script)
                .env("OCXD_PRODUCER_PROBE_PACKAGE", package_producer)
                .env("OCXD_PRODUCER_PROBE_ROOT", &producer_root)
                .current_dir(task.path())
                .output()
                .expect("actual isolated candidate producer");
            std::fs::write(
                evidence.with_file_name("actual-producer.stdout.log"),
                &result.stdout,
            )
            .unwrap();
            std::fs::write(
                evidence.with_file_name("actual-producer.stderr.log"),
                &result.stderr,
            )
            .unwrap();
            let readback = std::fs::read(producer_root.join("producer-report.json"))
                .ok()
                .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
            if !matches!(readback.as_ref(),Some(value) if value["fixtureSafeToRemove"]==true) {
                report["actualProducerReadback"] = readback
                    .unwrap_or(serde_json::json!({"accepted":false,"fixtureSafeToRemove":false}));
                report["fixtureRemoved"] = serde_json::json!(false);
                let mut file = OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(&evidence)
                    .unwrap();
                writeln!(file, "{report}").unwrap();
                let _retained = task.keep();
                panic!("Actual producer cleanup is unproved; exact temporary HOME retained, no force kill or false stopped state");
            }
            report["actualProducerReadback"] = readback.unwrap();
            if !result.status.success() {
                prototype_failures.push("actual-candidate-producer");
            }
        }
        report["prototypeFailures"] = serde_json::json!(prototype_failures);
        task.close().expect("exact fixture cleanup");
        report["fixtureRemoved"] = serde_json::json!(true);
        let mut output = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(evidence)
            .unwrap();
        writeln!(output, "{report}").unwrap();
        println!("original-record-diagnostic-only: {report}");
        assert!(
            prototype_failures.is_empty(),
            "Every independent prototype must pass; failures retained in report"
        );
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
