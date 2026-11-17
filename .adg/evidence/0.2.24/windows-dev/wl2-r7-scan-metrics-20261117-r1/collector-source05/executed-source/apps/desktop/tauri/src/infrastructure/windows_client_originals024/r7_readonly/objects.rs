//! Shared native observation DATA. No lease, cleanup, resume or restore API.
use super::super::{bounded::native_stamp, tree};
use crate::infrastructure::{hash, windows_official_runtime024 as native};
use serde_json::{json, Value};
use std::{
    fs::{self, OpenOptions},
    io::{self, Read},
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    path::Path,
    time::Instant,
};
fn invalid(reason: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, reason)
}
pub(super) fn observe(path: &Path, directory: bool, end: Instant) -> io::Result<Value> {
    native::validate_local_path(path)?;
    let mut file = OpenOptions::new()
        .access_mode(if directory { 0x20080 } else { 0x80020000 })
        .share_mode(7)
        .custom_flags(0x200000 | if directory { 0x2000000 } else { 0 })
        .open(path)?;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, GetFileType, GetFinalPathNameByHandleW,
        BY_HANDLE_FILE_INFORMATION, FILE_TYPE_DISK,
    };
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { GetFileType(file.as_raw_handle()) } != FILE_TYPE_DISK
        || info.dwFileAttributes & 0x400 != 0
        || (info.dwFileAttributes & 0x10 != 0) != directory
        || !directory && info.nNumberOfLinks != 1
    {
        return Err(invalid("reparse/multiple links/wrong kind refused"));
    }
    let mut final_path = vec![0u16; 32768];
    let n = unsafe {
        GetFinalPathNameByHandleW(file.as_raw_handle(), final_path.as_mut_ptr(), 32768, 0)
    };
    if n == 0 || n >= 32768 {
        return Err(invalid("native final path unavailable"));
    }
    let actual =
        String::from_utf16(&final_path[..n as usize]).map_err(|_| invalid("native path UTF16"))?;
    let actual = actual
        .strip_prefix(r"\\?\")
        .ok_or_else(|| invalid("local native path required"))?;
    if !actual.eq_ignore_ascii_case(&path.to_string_lossy().replace('/', r"\")) {
        return Err(invalid("native final path differs"));
    }
    tree::streams(path, directory)?;
    let security = native_stamp(&file, end).map_err(io::Error::other)?;
    let mut children = Vec::new();
    let mut bytes = Vec::new();
    if directory {
        for entry in fs::read_dir(path)? {
            if children.len() >= 20000 {
                return Err(invalid("children bound"));
            }
            children.push(
                entry?
                    .file_name()
                    .into_string()
                    .map_err(|_| invalid("child UTF16"))?,
            );
        }
        children.sort();
    } else {
        file.by_ref().take(4194305).read_to_end(&mut bytes)?;
        if bytes.len() > 4194304 {
            return Err(invalid("critical record bound"));
        }
    }
    if native_stamp(&file, end).map_err(io::Error::other)? != security {
        return Err(invalid("native object/security drift"));
    }
    tree::streams(path, directory)?;
    Ok(
        json!({"path":path,"directory":directory,"security":security,"children":children,"bytes":bytes.len(),"sha256":hash::sha256_hex(&bytes)}),
    )
}
pub(super) fn compare(expected: &Value, actual: &Value) -> io::Result<()> {
    if expected
        .get("security")
        .is_some_and(|s| s != &actual["security"])
        || expected
            .get("fileId")
            .is_some_and(|v| v != &actual["security"]["fileId"])
        || expected
            .get("volume")
            .is_some_and(|v| v != &actual["security"]["volume"])
        || expected
            .get("children")
            .is_some_and(|v| v != &actual["children"])
        || expected.get("bytes").is_some_and(|v| v != &actual["bytes"])
        || expected
            .get("sha256")
            .is_some_and(|v| v != &actual["sha256"])
    {
        return Err(invalid(
            "exact original expected object/bytes/8/children differ",
        ));
    }
    Ok(())
}
